use crate::csv_loader::IoError;
use flate2::read::ZlibDecoder;
use std::io::{Read, Seek, SeekFrom};

pub enum DecodedSlot {
    Numeric(Option<f64>),
    StringBytes([u8; 8]),
    Eof,
}

pub struct BytecodeReader<R: Read> {
    reader: R,
    bias: f64,
    is_little_endian: bool,
    current_codes: [u8; 8],
    current_literals: Vec<[u8; 8]>,
    code_index: usize,
    literal_index: usize,
    reached_eof: bool,
}

impl<R: Read> BytecodeReader<R> {
    pub fn new(reader: R, bias: f64, is_little_endian: bool) -> Self {
        Self {
            reader,
            bias,
            is_little_endian,
            current_codes: [0u8; 8],
            current_literals: Vec::new(),
            code_index: 8,
            literal_index: 0,
            reached_eof: false,
        }
    }

    fn read_next_chunk(&mut self) -> Result<bool, IoError> {
        let mut codes = [0u8; 8];
        match self.reader.read_exact(&mut codes) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                self.reached_eof = true;
                return Ok(false);
            }
            Err(e) => return Err(IoError::Io(format!("Error reading bytecode group: {}", e))),
        }

        let num_literals = codes.iter().filter(|&&c| c == 253).count();
        let mut literals = Vec::with_capacity(num_literals);

        for _ in 0..num_literals {
            let mut lit = [0u8; 8];
            self.reader
                .read_exact(&mut lit)
                .map_err(|e| IoError::Io(format!("Error reading bytecode literal: {}", e)))?;
            literals.push(lit);
        }

        self.current_codes = codes;
        self.current_literals = literals;
        self.code_index = 0;
        self.literal_index = 0;
        Ok(true)
    }

    pub fn next_slot(&mut self, is_string: bool) -> Result<DecodedSlot, IoError> {
        if self.reached_eof {
            return Ok(DecodedSlot::Eof);
        }

        loop {
            if self.code_index >= 8 && !self.read_next_chunk()? {
                return Ok(DecodedSlot::Eof);
            }

            let code = self.current_codes[self.code_index];
            self.code_index += 1;

            match code {
                0 => {
                    // Padding/no-op, skip to next instruction
                    continue;
                }
                1..=251 => {
                    let val = (code as f64) - self.bias;
                    if is_string {
                        let mut b = [0u8; 8];
                        let s = format!("{:.0}", val);
                        let bytes = s.as_bytes();
                        let len = bytes.len().min(8);
                        b[..len].copy_from_slice(&bytes[..len]);
                        b[len..8].fill(b' ');
                        return Ok(DecodedSlot::StringBytes(b));
                    } else {
                        return Ok(DecodedSlot::Numeric(Some(val)));
                    }
                }
                252 => {
                    self.reached_eof = true;
                    return Ok(DecodedSlot::Eof);
                }
                253 => {
                    if self.literal_index >= self.current_literals.len() {
                        return Err(IoError::MalformedSpss(
                            "Bytecode literal index out of range".to_string(),
                        ));
                    }
                    let raw = self.current_literals[self.literal_index];
                    self.literal_index += 1;

                    if is_string {
                        return Ok(DecodedSlot::StringBytes(raw));
                    } else {
                        let num = if self.is_little_endian {
                            f64::from_le_bytes(raw)
                        } else {
                            f64::from_be_bytes(raw)
                        };

                        if num.is_nan() || num <= -1.79e308 {
                            return Ok(DecodedSlot::Numeric(None));
                        } else {
                            return Ok(DecodedSlot::Numeric(Some(num)));
                        }
                    }
                }
                254 => {
                    if is_string {
                        return Ok(DecodedSlot::StringBytes([b' '; 8]));
                    } else {
                        return Ok(DecodedSlot::Numeric(None));
                    }
                }
                255 => {
                    if is_string {
                        return Ok(DecodedSlot::StringBytes([b' '; 8]));
                    } else {
                        return Ok(DecodedSlot::Numeric(None));
                    }
                }
            }
        }
    }
}

pub fn decompress_zsav_stream<R: Read + Seek>(
    reader: &mut R,
    zheader_ofs: i64,
    ztrailer_ofs: i64,
    ztrailer_len: i64,
    is_little_endian: bool,
) -> Result<Vec<u8>, IoError> {
    if ztrailer_ofs <= 0 || ztrailer_len < 24 {
        return Err(IoError::MalformedSpss(
            "Invalid .zsav trailer offset or length".to_string(),
        ));
    }

    reader
        .seek(SeekFrom::Start(ztrailer_ofs as u64))
        .map_err(|e| IoError::Io(format!("Failed to seek to .zsav trailer: {}", e)))?;

    let mut trailer_hdr = [0u8; 24];
    reader
        .read_exact(&mut trailer_hdr)
        .map_err(|e| IoError::Io(format!("Failed to read .zsav trailer header: {}", e)))?;

    let n_blocks = if is_little_endian {
        i32::from_le_bytes([
            trailer_hdr[20],
            trailer_hdr[21],
            trailer_hdr[22],
            trailer_hdr[23],
        ])
    } else {
        i32::from_be_bytes([
            trailer_hdr[20],
            trailer_hdr[21],
            trailer_hdr[22],
            trailer_hdr[23],
        ])
    };

    if !(0..=1_000_000).contains(&n_blocks) {
        return Err(IoError::MalformedSpss(format!(
            "Invalid block count in .zsav trailer: {}",
            n_blocks
        )));
    }

    let mut block_descriptors = Vec::with_capacity(n_blocks as usize);
    for _ in 0..n_blocks {
        let mut desc = [0u8; 16];
        reader
            .read_exact(&mut desc)
            .map_err(|e| IoError::Io(format!("Failed to read block descriptor: {}", e)))?;

        let (uncompressed_size, compressed_size) = if is_little_endian {
            (
                i64::from_le_bytes([
                    desc[0], desc[1], desc[2], desc[3], desc[4], desc[5], desc[6], desc[7],
                ]),
                i64::from_le_bytes([
                    desc[8], desc[9], desc[10], desc[11], desc[12], desc[13], desc[14], desc[15],
                ]),
            )
        } else {
            (
                i64::from_be_bytes([
                    desc[0], desc[1], desc[2], desc[3], desc[4], desc[5], desc[6], desc[7],
                ]),
                i64::from_be_bytes([
                    desc[8], desc[9], desc[10], desc[11], desc[12], desc[13], desc[14], desc[15],
                ]),
            )
        };

        if uncompressed_size < 0 || compressed_size < 0 || uncompressed_size > 10 * 1024 * 1024 {
            return Err(IoError::MalformedSpss(format!(
                "Invalid block sizes in .zsav: uncompressed={}, compressed={}",
                uncompressed_size, compressed_size
            )));
        }

        block_descriptors.push((uncompressed_size as usize, compressed_size as usize));
    }

    reader
        .seek(SeekFrom::Start(zheader_ofs as u64))
        .map_err(|e| IoError::Io(format!("Failed to seek to .zsav blocks: {}", e)))?;

    let mut decompressed_data = Vec::new();
    for (uncomp_sz, comp_sz) in block_descriptors {
        let mut comp_buf = vec![0u8; comp_sz];
        reader
            .read_exact(&mut comp_buf)
            .map_err(|e| IoError::Io(format!("Failed to read compressed block: {}", e)))?;

        let mut decoder = ZlibDecoder::new(&comp_buf[..]);
        let mut block_uncomp = Vec::with_capacity(uncomp_sz);
        decoder
            .read_to_end(&mut block_uncomp)
            .map_err(|e| IoError::MalformedSpss(format!("Zlib decompression failed: {}", e)))?;

        decompressed_data.extend_from_slice(&block_uncomp);
    }

    Ok(decompressed_data)
}
