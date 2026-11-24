use crate::styling::CellStyleFormat;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct ReportDocument {
    pub title: String,
    pub sections: Vec<ReportSection>,
}

impl ReportDocument {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            sections: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.sections.is_empty()
    }

    pub fn add_section(&mut self, section: ReportSection) {
        self.sections.push(section);
    }
}

#[derive(Clone, Debug)]
pub struct ReportSection {
    pub title: String,
    pub items: Vec<ReportItem>,
}

impl ReportSection {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            items: Vec::new(),
        }
    }

    pub fn add_item(&mut self, item: ReportItem) {
        self.items.push(item);
    }
}

#[derive(Clone, Debug)]
pub enum ReportItem {
    Log(String),
    Notes(ReportNotes),
    Table(ReportTable),
    Text { title: String, content: String },
    Warning(String),
}

#[derive(Clone, Debug, Default)]
pub struct ReportNotes {
    pub dataset_name: String,
    pub timestamp: String,
    pub active_cases: usize,
    pub active_vars: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CellAlignment {
    #[default]
    Left,
    Center,
    Right,
}

pub mod cell_styles_serde {
    use super::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(
        styles: &HashMap<(usize, usize), CellStyleFormat>,
        s: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut list: Vec<((usize, usize), &CellStyleFormat)> =
            styles.iter().map(|(k, v)| (*k, v)).collect();
        list.sort_by_key(|(k, _)| *k);
        list.serialize(s)
    }

    pub fn deserialize<'de, D>(d: D) -> Result<HashMap<(usize, usize), CellStyleFormat>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let list = Vec::<((usize, usize), CellStyleFormat)>::deserialize(d)?;
        Ok(list.into_iter().collect())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ReportTable {
    pub title: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub alignments: Vec<CellAlignment>,
    pub footnotes: Vec<String>,
    #[serde(default, with = "cell_styles_serde")]
    pub cell_styles: HashMap<(usize, usize), CellStyleFormat>,
}

impl ReportTable {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            headers: Vec::new(),
            rows: Vec::new(),
            alignments: Vec::new(),
            footnotes: Vec::new(),
            cell_styles: HashMap::new(),
        }
    }
}
