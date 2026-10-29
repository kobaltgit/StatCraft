pub mod dataset;
pub mod split;
pub mod variable;

pub use dataset::{CellCoord, CellEdit, DataSet, DeltaBuffer};
pub use split::{SplitFileConfig, SplitFileMode, SplitKeyValue, SplitSubgroup};
pub use statcraft_io::{RawDataType, RawTable, RawValue};
pub use variable::{
    Alignment, CoreError, DataType, Measure, MissingValues, Role, ValueLabels, VariableDictionary,
    VariableMetadata,
};
