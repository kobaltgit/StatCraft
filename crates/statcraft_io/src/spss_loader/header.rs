use crate::csv_loader::IoError;
use std::io::Read;

#[derive(Debug, Clone, PartialEq)]
pub struct SystemHeader {
    pub magic: [u8; 4],
    pub prod_name: String,
    pub layout_code: i32,
    pub nominal_case_size: usize,
    pub compression: i32,
    pub weight_index: i32,
    pub case_cnt: i64,
    pub bias: f64,
    pub creation_date: String,
    pub creation_time: String,
    pub file_label: String,
    pub is_little_endian: bool,
}

pub fn parse_header<R: Read>(reader: &mut R) -> Result<SystemHeader, IoError> {
    let mut buf = [0u8; 176];
    reader
        .read_exact(&mut buf)
        .map_err(|e| IoError::Io(format!("Failed to read SPSS system header: {}", e)))?;

    let magic: [u8; 4] = [buf[0], buf[1], buf[2], buf[3]];
    if &magic != b"$FL2" && &magic != b"$FL3" {
        return Err(IoError::MalformedSpss(format!(
            "Invalid SPSS magic signature: {:?}",
            String::from_utf8_lossy(&magic)
        )));
    }

    let raw_layout_le = i32::from_le_bytes([buf[64], buf[65], buf[66], buf[67]]);
    let raw_layout_be = i32::from_be_bytes([buf[64], buf[65], buf[66], buf[67]]);

    let is_little_endian = if raw_layout_le == 2 {
        true
    } else if raw_layout_be == 2 {
        false
    } else {
        return Err(IoError::MalformedSpss(format!(
            "Unsupported SPSS layout code (LE={}, BE={})",
            raw_layout_le, raw_layout_be
        )));
    };

    let read_i32 = |bytes: &[u8]| -> i32 {
        let arr = [bytes[0], bytes[1], bytes[2], bytes[3]];
        if is_little_endian {
            i32::from_le_bytes(arr)
        } else {
            i32::from_be_bytes(arr)
        }
    };

    let read_f64 = |bytes: &[u8]| -> f64 {
        let arr = [
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ];
        if is_little_endian {
            f64::from_le_bytes(arr)
        } else {
            f64::from_be_bytes(arr)
        }
    };

    let nominal_case_size_raw = read_i32(&buf[68..72]);
    if nominal_case_size_raw <= 0 {
        return Err(IoError::MalformedSpss(format!(
            "Invalid nominal case size: {}",
            nominal_case_size_raw
        )));
    }
    let nominal_case_size = nominal_case_size_raw as usize;

    let compression = read_i32(&buf[72..76]);
    let weight_index = read_i32(&buf[76..80]);
    let raw_cases = read_i32(&buf[80..84]);
    let case_cnt = if raw_cases < 0 { -1 } else { raw_cases as i64 };
    let bias = read_f64(&buf[84..92]);

    if bias <= 0.0 || !bias.is_finite() {
        return Err(IoError::MalformedSpss(format!(
            "Invalid SPSS compression bias: {}",
            bias
        )));
    }

    let prod_name = String::from_utf8_lossy(&buf[4..64]).trim().to_string();
    let creation_date = String::from_utf8_lossy(&buf[92..101]).trim().to_string();
    let creation_time = String::from_utf8_lossy(&buf[101..109]).trim().to_string();
    let file_label = String::from_utf8_lossy(&buf[109..173]).trim().to_string();

    Ok(SystemHeader {
        magic,
        prod_name,
        layout_code: 2,
        nominal_case_size,
        compression,
        weight_index,
        case_cnt,
        bias,
        creation_date,
        creation_time,
        file_label,
        is_little_endian,
    })
}
