use crate::dataset::DataSet;
use crate::stc_codec::load_stc_from_path;
use crate::variable::CoreError;
use statcraft_io::{load_csv_from_path, load_excel_from_path, CsvOptions};
use std::path::Path;

/// Loads a `DataSet` from an external file (.sav, .zsav, .csv, .stc, .xlsx).
pub fn load_dataset_from_file<P: AsRef<Path>>(path: P) -> Result<DataSet, CoreError> {
    let p = path.as_ref();
    let ext = p
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "sav" | "zsav" => DataSet::load_spss_from_path(p),
        "csv" => {
            let table = load_csv_from_path(p, CsvOptions::default()).map_err(CoreError::Io)?;
            DataSet::from_raw_table(table)
        }
        "stc" => load_stc_from_path(p).map_err(CoreError::Io),
        "xlsx" | "xls" => {
            let table = load_excel_from_path(p, None).map_err(CoreError::Io)?;
            DataSet::from_raw_table(table)
        }
        other => Err(CoreError::MergeError(format!(
            "Unsupported file format for merge: .{}",
            other
        ))),
    }
}
