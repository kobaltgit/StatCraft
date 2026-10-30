use crate::csv_loader::IoError;
use std::collections::{BTreeMap, HashMap};
use std::io::Read;

#[derive(Debug, Clone, PartialEq)]
pub struct RawVarRecord {
    pub type_code: i32,
    pub has_var_label: bool,
    pub missing_code: i32,
    pub print_format: u32,
    pub write_format: u32,
    pub name: String,
    pub label: String,
    pub missing_discrete: Vec<String>,
    pub missing_range: Option<(f64, f64)>,
}

#[derive(Debug, Clone, Default)]
pub struct ExtensionData {
    pub long_names: HashMap<String, String>,
    pub display_params: Vec<(u8, usize, u8)>, // (measure, display_width, alignment)
    pub encoding: String,
    pub zheader_ofs: i64,
    pub ztrailer_ofs: i64,
    pub ztrailer_len: i64,
}

pub struct ReaderHelper<'a, R: Read> {
    pub reader: &'a mut R,
    pub is_little_endian: bool,
}

impl<'a, R: Read> ReaderHelper<'a, R> {
    pub fn read_i32(&mut self) -> Result<i32, IoError> {
        let mut b = [0u8; 4];
        self.reader
            .read_exact(&mut b)
            .map_err(|e| IoError::Io(format!("Failed to read i32: {}", e)))?;
        Ok(if self.is_little_endian {
            i32::from_le_bytes(b)
        } else {
            i32::from_be_bytes(b)
        })
    }

    pub fn read_u32(&mut self) -> Result<u32, IoError> {
        let mut b = [0u8; 4];
        self.reader
            .read_exact(&mut b)
            .map_err(|e| IoError::Io(format!("Failed to read u32: {}", e)))?;
        Ok(if self.is_little_endian {
            u32::from_le_bytes(b)
        } else {
            u32::from_be_bytes(b)
        })
    }

    pub fn read_i64(&mut self) -> Result<i64, IoError> {
        let mut b = [0u8; 8];
        self.reader
            .read_exact(&mut b)
            .map_err(|e| IoError::Io(format!("Failed to read i64: {}", e)))?;
        Ok(if self.is_little_endian {
            i64::from_le_bytes(b)
        } else {
            i64::from_be_bytes(b)
        })
    }

    pub fn read_f64(&mut self) -> Result<f64, IoError> {
        let mut b = [0u8; 8];
        self.reader
            .read_exact(&mut b)
            .map_err(|e| IoError::Io(format!("Failed to read f64: {}", e)))?;
        Ok(if self.is_little_endian {
            f64::from_le_bytes(b)
        } else {
            f64::from_be_bytes(b)
        })
    }

    pub fn skip(&mut self, len: usize) -> Result<(), IoError> {
        let mut remaining = len;
        let mut buf = [0u8; 1024];
        while remaining > 0 {
            let to_read = remaining.min(buf.len());
            self.reader
                .read_exact(&mut buf[..to_read])
                .map_err(|e| IoError::Io(format!("Failed to skip bytes: {}", e)))?;
            remaining -= to_read;
        }
        Ok(())
    }
}

pub fn parse_variable_record<R: Read>(
    helper: &mut ReaderHelper<R>,
) -> Result<RawVarRecord, IoError> {
    let type_code = helper.read_i32()?;
    let has_var_label = helper.read_i32()? == 1;
    let missing_code = helper.read_i32()?;
    let print_format = helper.read_u32()?;
    let write_format = helper.read_u32()?;

    let mut name_buf = [0u8; 8];
    helper
        .reader
        .read_exact(&mut name_buf)
        .map_err(|e| IoError::Io(format!("Failed to read variable name: {}", e)))?;
    let name = String::from_utf8_lossy(&name_buf).trim().to_string();

    let mut label = String::new();
    if has_var_label {
        let label_len = helper.read_i32()?;
        if !(0..=1024 * 1024).contains(&label_len) {
            return Err(IoError::MalformedSpss(format!(
                "Invalid variable label length: {}",
                label_len
            )));
        }
        let mut l_buf = vec![0u8; label_len as usize];
        helper
            .reader
            .read_exact(&mut l_buf)
            .map_err(|e| IoError::Io(format!("Failed to read variable label: {}", e)))?;
        label = String::from_utf8_lossy(&l_buf).trim().to_string();

        let pad = (4 - (label_len % 4)) % 4;
        if pad > 0 {
            helper.skip(pad as usize)?;
        }
    }

    let mut missing_discrete = Vec::new();
    let mut missing_range = None;

    if (1..=3).contains(&missing_code) {
        for _ in 0..missing_code {
            if type_code == 0 {
                let v = helper.read_f64()?;
                let s = if v.fract() == 0.0 {
                    format!("{:.0}", v)
                } else {
                    format!("{}", v)
                };
                missing_discrete.push(s);
            } else {
                let mut b = [0u8; 8];
                helper
                    .reader
                    .read_exact(&mut b)
                    .map_err(|e| IoError::Io(format!("Failed to read missing string: {}", e)))?;
                missing_discrete.push(String::from_utf8_lossy(&b).trim().to_string());
            }
        }
    } else if missing_code == -2 {
        let low = helper.read_f64()?;
        let high = helper.read_f64()?;
        missing_range = Some((low, high));
    } else if missing_code == -3 {
        let low = helper.read_f64()?;
        let high = helper.read_f64()?;
        let disc = helper.read_f64()?;
        missing_range = Some((low, high));
        let s = if disc.fract() == 0.0 {
            format!("{:.0}", disc)
        } else {
            format!("{}", disc)
        };
        missing_discrete.push(s);
    }

    Ok(RawVarRecord {
        type_code,
        has_var_label,
        missing_code,
        print_format,
        write_format,
        name,
        label,
        missing_discrete,
        missing_range,
    })
}

pub fn parse_value_labels<R: Read>(
    helper: &mut ReaderHelper<R>,
) -> Result<(BTreeMap<String, String>, Vec<usize>), IoError> {
    let label_count = helper.read_i32()?;
    if !(0..=1_000_000).contains(&label_count) {
        return Err(IoError::MalformedSpss(format!(
            "Invalid value label count: {}",
            label_count
        )));
    }

    let mut labels = BTreeMap::new();
    for _ in 0..label_count {
        let val = helper.read_f64()?;
        let mut len_b = [0u8; 1];
        helper
            .reader
            .read_exact(&mut len_b)
            .map_err(|e| IoError::Io(format!("Failed to read value label length: {}", e)))?;
        let len = len_b[0] as usize;

        let mut text_b = vec![0u8; len];
        helper
            .reader
            .read_exact(&mut text_b)
            .map_err(|e| IoError::Io(format!("Failed to read value label text: {}", e)))?;
        let label_text = String::from_utf8_lossy(&text_b).trim().to_string();

        let rem = (1 + len) % 8;
        if rem != 0 {
            helper.skip(8 - rem)?;
        }

        let key = if val.fract() == 0.0 {
            format!("{:.0}", val)
        } else {
            format!("{}", val)
        };
        labels.insert(key, label_text);
    }

    let rec4_type = helper.read_i32()?;
    if rec4_type != 4 {
        return Err(IoError::MalformedSpss(format!(
            "Expected Record Type 4 following Record Type 3, found {}",
            rec4_type
        )));
    }

    let var_count = helper.read_i32()?;
    if !(0..=100_000).contains(&var_count) {
        return Err(IoError::MalformedSpss(format!(
            "Invalid var count in Record 4: {}",
            var_count
        )));
    }

    let mut var_indices = Vec::with_capacity(var_count as usize);
    for _ in 0..var_count {
        let idx = helper.read_i32()?;
        if idx > 0 {
            var_indices.push(idx as usize);
        }
    }

    Ok((labels, var_indices))
}

pub fn parse_extension_record<R: Read>(
    helper: &mut ReaderHelper<R>,
    ext: &mut ExtensionData,
) -> Result<(), IoError> {
    let subtype = helper.read_i32()?;
    let size = helper.read_i32()?;
    let count = helper.read_i32()?;

    if size < 0 || count < 0 {
        return Err(IoError::MalformedSpss(format!(
            "Negative extension size/count: size={}, count={}",
            size, count
        )));
    }

    let total_len = (size as usize).saturating_mul(count as usize);
    if total_len > 64 * 1024 * 1024 {
        return Err(IoError::MalformedSpss(format!(
            "Extension record payload too large: {} bytes",
            total_len
        )));
    }

    let mut payload = vec![0u8; total_len];
    helper
        .reader
        .read_exact(&mut payload)
        .map_err(|e| IoError::Io(format!("Failed to read extension record: {}", e)))?;

    match subtype {
        4 => {
            // Display parameters: size=4, 3 ints (measure, width, alignment) or 2 ints
            if size == 4 && total_len >= 8 {
                let n_ints = total_len / 4;
                let mut ints = Vec::with_capacity(n_ints);
                let (chunks, _) = payload.as_chunks::<4>();
                for chunk in chunks {
                    let val = if helper.is_little_endian {
                        i32::from_le_bytes(*chunk)
                    } else {
                        i32::from_be_bytes(*chunk)
                    };
                    ints.push(val);
                }

                if ints.len() % 3 == 0 {
                    let (triplets, _) = ints.as_chunks::<3>();
                    for chunk in triplets {
                        let measure = chunk[0].clamp(1, 3) as u8;
                        let width = (chunk[1].max(1) as usize).min(255);
                        let align = chunk[2].clamp(0, 2) as u8;
                        ext.display_params.push((measure, width, align));
                    }
                } else if ints.len() % 2 == 0 {
                    let (pairs, _) = ints.as_chunks::<2>();
                    for chunk in pairs {
                        let measure = chunk[0].clamp(1, 3) as u8;
                        let width = (chunk[1].max(1) as usize).min(255);
                        ext.display_params.push((measure, width, 1));
                    }
                }
            }
        }
        13 => {
            // Long variable names: "SHORT=LongName\tSHORT2=LongName2"
            let text = String::from_utf8_lossy(&payload);
            for pair in text.split('\t') {
                let trimmed = pair.trim_matches('\0').trim();
                if let Some((short_name, long_name)) = trimmed.split_once('=') {
                    ext.long_names.insert(
                        short_name.trim().to_uppercase(),
                        long_name.trim().to_string(),
                    );
                }
            }
        }
        20 => {
            // Character encoding: e.g. "UTF-8"
            ext.encoding = String::from_utf8_lossy(&payload)
                .trim_matches('\0')
                .trim()
                .to_string();
        }
        24 if total_len >= 24 => {
            // ZLIB header/trailer offsets: 3 i64s
            let read_i64_at = |offset: usize| -> i64 {
                let b = [
                    payload[offset],
                    payload[offset + 1],
                    payload[offset + 2],
                    payload[offset + 3],
                    payload[offset + 4],
                    payload[offset + 5],
                    payload[offset + 6],
                    payload[offset + 7],
                ];
                if helper.is_little_endian {
                    i64::from_le_bytes(b)
                } else {
                    i64::from_be_bytes(b)
                }
            };
            ext.zheader_ofs = read_i64_at(0);
            ext.ztrailer_ofs = read_i64_at(8);
            ext.ztrailer_len = read_i64_at(16);
        }
        _ => {}
    }

    Ok(())
}
