use crate::variable::CoreError;
use statcraft_io::RawValue;
use std::collections::HashMap;

/// 1-bit bitmask tracking null entries to minimize memory footprint.
/// Each byte tracks 8 rows. Bit = 1 represents NULL; Bit = 0 represents present value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NullBitmask {
    data: Vec<u8>,
    len: usize,
}

impl NullBitmask {
    pub fn new(len: usize) -> Self {
        let byte_count = len.div_ceil(8);
        Self {
            data: vec![0u8; byte_count],
            len,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn is_null(&self, index: usize) -> bool {
        if index >= self.len {
            return true;
        }
        let byte_idx = index / 8;
        let bit_idx = index % 8;
        (self.data[byte_idx] & (1 << bit_idx)) != 0
    }

    #[inline]
    pub fn set_null(&mut self, index: usize, is_null: bool) -> Result<(), CoreError> {
        if index >= self.len {
            return Err(CoreError::IndexOutOfBounds {
                index,
                total: self.len,
            });
        }
        let byte_idx = index / 8;
        let bit_idx = index % 8;
        if is_null {
            self.data[byte_idx] |= 1 << bit_idx;
        } else {
            self.data[byte_idx] &= !(1 << bit_idx);
        }
        Ok(())
    }

    pub fn memory_bytes(&self) -> usize {
        self.data.len() + std::mem::size_of::<Self>()
    }
}

/// Lossless dictionary-encoded column for string and categorical values.
/// String values are stored once in a unique dictionary pool, while individual rows
/// store 32-bit integer indexes. Missing values are tracked via 1-bit `NullBitmask`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictionaryEncodedColumn {
    dictionary: Vec<String>,
    indices: Vec<u32>,
    null_mask: NullBitmask,
}

impl DictionaryEncodedColumn {
    pub fn encode(values: &[RawValue]) -> Result<Self, CoreError> {
        let len = values.len();
        let mut null_mask = NullBitmask::new(len);
        let mut dict_map: HashMap<String, u32> = HashMap::new();
        let mut dictionary: Vec<String> = Vec::new();
        let mut indices: Vec<u32> = Vec::with_capacity(len);

        for (idx, val) in values.iter().enumerate() {
            match val {
                RawValue::Null => {
                    null_mask.set_null(idx, true)?;
                    indices.push(0);
                }
                RawValue::Text(s) => {
                    null_mask.set_null(idx, false)?;
                    let dict_idx = if let Some(&existing) = dict_map.get(s.as_str()) {
                        existing
                    } else {
                        let new_idx = dictionary.len() as u32;
                        dict_map.insert(s.clone(), new_idx);
                        dictionary.push(s.clone());
                        new_idx
                    };
                    indices.push(dict_idx);
                }
                RawValue::Number(num) => {
                    // Numeric formatted as string if forced into dictionary column
                    null_mask.set_null(idx, false)?;
                    let s = num.to_string();
                    let dict_idx = if let Some(&existing) = dict_map.get(s.as_str()) {
                        existing
                    } else {
                        let new_idx = dictionary.len() as u32;
                        dict_map.insert(s.clone(), new_idx);
                        dictionary.push(s.clone());
                        new_idx
                    };
                    indices.push(dict_idx);
                }
            }
        }

        Ok(Self {
            dictionary,
            indices,
            null_mask,
        })
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    #[inline]
    pub fn is_null(&self, index: usize) -> bool {
        self.null_mask.is_null(index)
    }

    #[inline]
    pub fn get_str(&self, index: usize) -> Option<&str> {
        if self.null_mask.is_null(index) || index >= self.indices.len() {
            return None;
        }
        let dict_idx = self.indices[index] as usize;
        self.dictionary.get(dict_idx).map(|s| s.as_str())
    }

    pub fn decode(&self) -> Vec<RawValue> {
        let mut out = Vec::with_capacity(self.indices.len());
        for (i, &dict_idx) in self.indices.iter().enumerate() {
            if self.null_mask.is_null(i) {
                out.push(RawValue::Null);
            } else if let Some(s) = self.dictionary.get(dict_idx as usize) {
                out.push(RawValue::Text(s.clone()));
            } else {
                out.push(RawValue::Null);
            }
        }
        out
    }

    pub fn memory_bytes(&self) -> usize {
        let dict_heap: usize = self.dictionary.iter().map(|s| s.capacity()).sum();
        let dict_overhead = self.dictionary.len() * std::mem::size_of::<String>();
        let indices_bytes = self.indices.capacity() * std::mem::size_of::<u32>();
        dict_heap
            + dict_overhead
            + indices_bytes
            + self.null_mask.memory_bytes()
            + std::mem::size_of::<Self>()
    }
}

/// Lossless compressed column container supporting both pure 64-bit float numbers
/// (never downcast to f32) and dictionary-encoded strings.
#[derive(Debug, Clone, PartialEq)]
pub enum LosslessCompressedColumn {
    Numeric {
        values: Vec<f64>,
        null_mask: NullBitmask,
    },
    Dictionary(DictionaryEncodedColumn),
}

impl LosslessCompressedColumn {
    pub fn encode_numeric(raw: &[RawValue]) -> Result<Self, CoreError> {
        let len = raw.len();
        let mut null_mask = NullBitmask::new(len);
        let mut values = Vec::with_capacity(len);

        for (i, val) in raw.iter().enumerate() {
            match val {
                RawValue::Number(num) => {
                    null_mask.set_null(i, false)?;
                    values.push(*num);
                }
                RawValue::Null => {
                    null_mask.set_null(i, true)?;
                    values.push(0.0);
                }
                RawValue::Text(s) => {
                    if let Ok(parsed) = s.trim().parse::<f64>() {
                        null_mask.set_null(i, false)?;
                        values.push(parsed);
                    } else {
                        null_mask.set_null(i, true)?;
                        values.push(0.0);
                    }
                }
            }
        }

        Ok(Self::Numeric { values, null_mask })
    }

    pub fn encode_text(raw: &[RawValue]) -> Result<Self, CoreError> {
        let dict = DictionaryEncodedColumn::encode(raw)?;
        Ok(Self::Dictionary(dict))
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::Numeric { values, .. } => values.len(),
            Self::Dictionary(d) => d.len(),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn is_null(&self, index: usize) -> bool {
        match self {
            Self::Numeric { null_mask, .. } => null_mask.is_null(index),
            Self::Dictionary(d) => d.is_null(index),
        }
    }

    #[inline]
    pub fn get_numeric(&self, index: usize) -> Option<f64> {
        match self {
            Self::Numeric { values, null_mask } => {
                if null_mask.is_null(index) || index >= values.len() {
                    None
                } else {
                    Some(values[index])
                }
            }
            Self::Dictionary(d) => d.get_str(index).and_then(|s| s.trim().parse::<f64>().ok()),
        }
    }

    #[inline]
    pub fn get_str(&self, index: usize) -> Option<&str> {
        match self {
            Self::Numeric { .. } => None,
            Self::Dictionary(d) => d.get_str(index),
        }
    }

    pub fn decode(&self) -> Vec<RawValue> {
        match self {
            Self::Numeric { values, null_mask } => values
                .iter()
                .enumerate()
                .map(|(i, &v)| {
                    if null_mask.is_null(i) {
                        RawValue::Null
                    } else {
                        RawValue::Number(v)
                    }
                })
                .collect(),
            Self::Dictionary(d) => d.decode(),
        }
    }

    pub fn memory_bytes(&self) -> usize {
        match self {
            Self::Numeric { values, null_mask } => {
                values.capacity() * std::mem::size_of::<f64>()
                    + null_mask.memory_bytes()
                    + std::mem::size_of::<Self>()
            }
            Self::Dictionary(d) => d.memory_bytes() + std::mem::size_of::<Self>(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_bitmask_operations() {
        let mut mask = NullBitmask::new(20);
        assert!(!mask.is_null(0));
        assert!(!mask.is_null(19));

        mask.set_null(3, true).unwrap();
        mask.set_null(15, true).unwrap();

        assert!(mask.is_null(3));
        assert!(mask.is_null(15));
        assert!(!mask.is_null(2));
        assert!(!mask.is_null(14));

        mask.set_null(3, false).unwrap();
        assert!(!mask.is_null(3));

        assert!(mask.set_null(100, true).is_err());
    }

    #[test]
    fn test_dictionary_encoding_roundtrip() {
        let raw = vec![
            RawValue::Text("Category A".to_string()),
            RawValue::Null,
            RawValue::Text("Category B".to_string()),
            RawValue::Text("Category A".to_string()),
            RawValue::Text("Category C".to_string()),
            RawValue::Text("Category B".to_string()),
            RawValue::Null,
        ];

        let col = DictionaryEncodedColumn::encode(&raw).unwrap();
        assert_eq!(col.len(), 7);
        assert_eq!(col.dictionary.len(), 3);
        assert_eq!(col.get_str(0), Some("Category A"));
        assert_eq!(col.get_str(1), None);
        assert!(col.is_null(1));
        assert_eq!(col.get_str(2), Some("Category B"));
        assert_eq!(col.get_str(3), Some("Category A"));

        let decoded = col.decode();
        assert_eq!(decoded, raw);
    }

    #[test]
    fn test_lossless_numeric_preserves_f64_precision() {
        let raw = vec![
            RawValue::Number(123_456_789.987_654_33),
            RawValue::Null,
            RawValue::Number(-0.000_000_001_234_567_9),
            RawValue::Number(1.0 / 3.0),
        ];

        let col = LosslessCompressedColumn::encode_numeric(&raw).unwrap();
        assert_eq!(col.len(), 4);
        assert_eq!(col.get_numeric(0), Some(123_456_789.987_654_33));
        assert_eq!(col.get_numeric(1), None);
        assert_eq!(col.get_numeric(2), Some(-0.000_000_001_234_567_9));
        assert_eq!(col.get_numeric(3), Some(1.0 / 3.0));

        let decoded = col.decode();
        assert_eq!(decoded, raw);
    }
}
