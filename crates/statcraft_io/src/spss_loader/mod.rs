pub mod decompress;
pub mod header;
pub mod records;
#[cfg(test)]
mod tests;

use crate::csv_loader::{IoError, RawColumn, RawDataType, RawTable, RawValue};
use decompress::{decompress_zsav_stream, BytecodeReader, DecodedSlot};
use header::{parse_header, SystemHeader};
use records::{
    parse_extension_record, parse_value_labels, parse_variable_record, ExtensionData, RawVarRecord,
    ReaderHelper,
};
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek};
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct SpssVariableInfo {
    pub name: String,
    pub original_name: String,
    pub var_type: RawDataType,
    pub string_len: usize,
    pub print_format: u32,
    pub write_format: u32,
    pub decimals: u8,
    pub width: usize,
    pub label: String,
    pub measure: u8,   // 1: Nominal, 2: Ordinal, 3: Scale
    pub alignment: u8, // 0: Left, 1: Right, 2: Center
    pub display_width: usize,
    pub value_labels: BTreeMap<String, String>,
    pub missing_discrete: Vec<String>,
    pub missing_range: Option<(f64, f64)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpssData {
    pub raw_table: RawTable,
    pub variables: Vec<SpssVariableInfo>,
    pub weight_var_name: Option<String>,
}

#[derive(Debug, Clone)]
struct LogicalVarSlotMap {
    logical_index: usize,
    is_string: bool,
}

pub fn load_spss_from_path<P: AsRef<Path>>(path: P) -> Result<SpssData, IoError> {
    let p = path.as_ref();
    let file = File::open(p)
        .map_err(|e| IoError::Io(format!("Cannot open SPSS file '{}': {}", p.display(), e)))?;
    let mut reader = BufReader::new(file);
    load_spss_from_reader(&mut reader)
}

pub fn load_spss_from_reader<R: Read + Seek>(reader: &mut R) -> Result<SpssData, IoError> {
    let header = parse_header(reader)?;

    let mut raw_vars: Vec<RawVarRecord> = Vec::new();
    let mut val_labels_map: HashMap<usize, BTreeMap<String, String>> = HashMap::new();
    let mut ext_data = ExtensionData::default();

    loop {
        let mut rec_type_b = [0u8; 4];
        reader
            .read_exact(&mut rec_type_b)
            .map_err(|e| IoError::Io(format!("Failed to read record type: {}", e)))?;
        let rec_type = if header.is_little_endian {
            i32::from_le_bytes(rec_type_b)
        } else {
            i32::from_be_bytes(rec_type_b)
        };

        match rec_type {
            2 => {
                let mut helper = ReaderHelper {
                    reader,
                    is_little_endian: header.is_little_endian,
                };
                let var_rec = parse_variable_record(&mut helper)?;
                raw_vars.push(var_rec);
            }
            3 => {
                let mut helper = ReaderHelper {
                    reader,
                    is_little_endian: header.is_little_endian,
                };
                let (labels, var_indices) = parse_value_labels(&mut helper)?;
                for idx in var_indices {
                    val_labels_map.insert(idx, labels.clone());
                }
            }
            6 => {
                let mut helper = ReaderHelper {
                    reader,
                    is_little_endian: header.is_little_endian,
                };
                let n_lines = helper.read_i32()?;
                if n_lines > 0 {
                    helper.skip((n_lines as usize).saturating_mul(80))?;
                }
            }
            7 => {
                let mut helper = ReaderHelper {
                    reader,
                    is_little_endian: header.is_little_endian,
                };
                parse_extension_record(&mut helper, &mut ext_data)?;
            }
            999 => {
                let mut helper = ReaderHelper {
                    reader,
                    is_little_endian: header.is_little_endian,
                };
                let _filler = helper.read_i32()?;
                break;
            }
            other => {
                return Err(IoError::MalformedSpss(format!(
                    "Unexpected SPSS record type: {}",
                    other
                )));
            }
        }
    }

    let (variables, slot_map, weight_var_name) =
        build_variables(&raw_vars, &val_labels_map, &ext_data, header.weight_index);

    let raw_table = read_table_data(reader, &header, &ext_data, &variables, &slot_map)?;

    Ok(SpssData {
        raw_table,
        variables,
        weight_var_name,
    })
}

fn build_variables(
    raw_vars: &[RawVarRecord],
    val_labels: &HashMap<usize, BTreeMap<String, String>>,
    ext: &ExtensionData,
    weight_index: i32,
) -> (
    Vec<SpssVariableInfo>,
    Vec<LogicalVarSlotMap>,
    Option<String>,
) {
    let mut variables: Vec<SpssVariableInfo> = Vec::new();
    let mut slot_map: Vec<LogicalVarSlotMap> = Vec::new();
    let mut slot_1based = 1usize;
    let mut weight_var_name = None;

    let mut current_var_idx = 0usize;
    let mut display_param_idx = 0usize;

    for (raw_idx, raw) in raw_vars.iter().enumerate() {
        if raw.type_code == -1 {
            // Continuation slot for a long string
            if let Some(last_slot) = slot_map.last() {
                slot_map.push(LogicalVarSlotMap {
                    logical_index: last_slot.logical_index,
                    is_string: true,
                });
            }
            slot_1based += 1;
            continue;
        }

        let is_string = raw.type_code > 0;
        let string_len = if is_string { raw.type_code as usize } else { 0 };
        let var_type = if is_string {
            RawDataType::Text
        } else {
            RawDataType::Numeric
        };

        let orig_name = raw.name.clone();
        let name = ext
            .long_names
            .get(&orig_name.to_uppercase())
            .cloned()
            .unwrap_or_else(|| orig_name.clone());

        if (slot_1based as i32) == weight_index {
            weight_var_name = Some(name.clone());
        }

        let decimals = (raw.print_format & 0xFF) as u8;
        let width = ((raw.print_format >> 8) & 0xFF) as usize;

        let (measure, display_width, alignment) =
            if let Some(dp) = ext.display_params.get(display_param_idx) {
                display_param_idx += 1;
                *dp
            } else {
                let m = if is_string { 1 } else { 3 };
                let a = if is_string { 0 } else { 1 };
                let w = width.max(8);
                (m, w, a)
            };

        let labels = val_labels
            .get(&(raw_idx + 1))
            .or_else(|| val_labels.get(&slot_1based))
            .cloned()
            .unwrap_or_default();

        variables.push(SpssVariableInfo {
            name,
            original_name: orig_name,
            var_type,
            string_len,
            print_format: raw.print_format,
            write_format: raw.write_format,
            decimals,
            width,
            label: raw.label.clone(),
            measure,
            alignment,
            display_width,
            value_labels: labels,
            missing_discrete: raw.missing_discrete.clone(),
            missing_range: raw.missing_range,
        });

        slot_map.push(LogicalVarSlotMap {
            logical_index: current_var_idx,
            is_string,
        });

        current_var_idx += 1;
        slot_1based += 1;
    }

    (variables, slot_map, weight_var_name)
}

fn read_table_data<R: Read + Seek>(
    reader: &mut R,
    header: &SystemHeader,
    ext: &ExtensionData,
    variables: &[SpssVariableInfo],
    slot_map: &[LogicalVarSlotMap],
) -> Result<RawTable, IoError> {
    let mut columns: Vec<RawColumn> = variables
        .iter()
        .map(|v| RawColumn {
            name: v.name.clone(),
            data_type: v.var_type,
            values: Vec::new(),
        })
        .collect();

    if variables.is_empty() {
        return Ok(RawTable {
            columns: vec![],
            row_count: 0,
        });
    }

    let mut row_count = 0usize;

    if header.compression == 2 {
        let decompressed = decompress_zsav_stream(
            reader,
            ext.zheader_ofs,
            ext.ztrailer_ofs,
            ext.ztrailer_len,
            header.is_little_endian,
        )?;
        let mut bytecode_reader = BytecodeReader::new(
            Cursor::new(decompressed),
            header.bias,
            header.is_little_endian,
        );
        read_cases_bytecode(
            &mut bytecode_reader,
            header.case_cnt,
            slot_map,
            variables,
            &mut columns,
            &mut row_count,
        )?;
    } else if header.compression == 1 {
        let mut bytecode_reader = BytecodeReader::new(reader, header.bias, header.is_little_endian);
        read_cases_bytecode(
            &mut bytecode_reader,
            header.case_cnt,
            slot_map,
            variables,
            &mut columns,
            &mut row_count,
        )?;
    } else {
        read_cases_uncompressed(
            reader,
            header.case_cnt,
            header.is_little_endian,
            slot_map,
            variables,
            &mut columns,
            &mut row_count,
        )?;
    }

    Ok(RawTable { columns, row_count })
}

fn read_cases_bytecode<R: Read>(
    b_reader: &mut BytecodeReader<R>,
    max_cases: i64,
    slot_map: &[LogicalVarSlotMap],
    variables: &[SpssVariableInfo],
    columns: &mut [RawColumn],
    row_count: &mut usize,
) -> Result<(), IoError> {
    let limit = if max_cases >= 0 {
        max_cases as usize
    } else {
        usize::MAX
    };

    while *row_count < limit {
        let mut case_values: Vec<Option<RawValue>> = vec![None; variables.len()];
        let mut case_string_buffers: Vec<Vec<u8>> = vec![Vec::new(); variables.len()];
        let mut hit_eof = false;

        for (slot_idx, slot) in slot_map.iter().enumerate() {
            let decoded = b_reader.next_slot(slot.is_string)?;
            match decoded {
                DecodedSlot::Eof => {
                    hit_eof = true;
                    if slot_idx == 0 {
                        return Ok(());
                    }
                    break;
                }
                DecodedSlot::Numeric(num_opt) => {
                    let val = match num_opt {
                        Some(n) => RawValue::Number(n),
                        None => RawValue::Null,
                    };
                    case_values[slot.logical_index] = Some(val);
                }
                DecodedSlot::StringBytes(bytes) => {
                    case_string_buffers[slot.logical_index].extend_from_slice(&bytes);
                }
            }
        }

        if hit_eof && case_values.iter().all(|v| v.is_none()) {
            break;
        }

        for (var_idx, var) in variables.iter().enumerate() {
            if var.var_type == RawDataType::Text {
                let buf = &case_string_buffers[var_idx];
                let len = var.string_len.min(buf.len());
                let s = String::from_utf8_lossy(&buf[..len]).trim_end().to_string();
                columns[var_idx].values.push(RawValue::Text(s));
            } else {
                let v = case_values[var_idx].take().unwrap_or(RawValue::Null);
                columns[var_idx].values.push(v);
            }
        }

        *row_count += 1;
        if hit_eof {
            break;
        }
    }

    Ok(())
}

fn read_cases_uncompressed<R: Read>(
    reader: &mut R,
    max_cases: i64,
    is_little_endian: bool,
    slot_map: &[LogicalVarSlotMap],
    variables: &[SpssVariableInfo],
    columns: &mut [RawColumn],
    row_count: &mut usize,
) -> Result<(), IoError> {
    let limit = if max_cases >= 0 {
        max_cases as usize
    } else {
        usize::MAX
    };

    while *row_count < limit {
        let mut case_values: Vec<Option<RawValue>> = vec![None; variables.len()];
        let mut case_string_buffers: Vec<Vec<u8>> = vec![Vec::new(); variables.len()];
        let mut hit_eof = false;

        for (slot_idx, slot) in slot_map.iter().enumerate() {
            let mut raw = [0u8; 8];
            match reader.read_exact(&mut raw) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    hit_eof = true;
                    if slot_idx == 0 {
                        return Ok(());
                    }
                    break;
                }
                Err(e) => {
                    return Err(IoError::Io(format!(
                        "Error reading uncompressed data: {}",
                        e
                    )))
                }
            }

            if slot.is_string {
                case_string_buffers[slot.logical_index].extend_from_slice(&raw);
            } else {
                let num = if is_little_endian {
                    f64::from_le_bytes(raw)
                } else {
                    f64::from_be_bytes(raw)
                };
                let val = if num.is_nan() || num <= -1.79e308 {
                    RawValue::Null
                } else {
                    RawValue::Number(num)
                };
                case_values[slot.logical_index] = Some(val);
            }
        }

        if hit_eof {
            break;
        }

        for (var_idx, var) in variables.iter().enumerate() {
            if var.var_type == RawDataType::Text {
                let buf = &case_string_buffers[var_idx];
                let len = var.string_len.min(buf.len());
                let s = String::from_utf8_lossy(&buf[..len]).trim_end().to_string();
                columns[var_idx].values.push(RawValue::Text(s));
            } else {
                let v = case_values[var_idx].take().unwrap_or(RawValue::Null);
                columns[var_idx].values.push(v);
            }
        }

        *row_count += 1;
    }

    Ok(())
}
