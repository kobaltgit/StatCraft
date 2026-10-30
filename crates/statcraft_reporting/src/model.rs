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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CellAlignment {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Clone, Debug, Default)]
pub struct ReportTable {
    pub title: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub alignments: Vec<CellAlignment>,
    pub footnotes: Vec<String>,
}

impl ReportTable {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            headers: Vec::new(),
            rows: Vec::new(),
            alignments: Vec::new(),
            footnotes: Vec::new(),
        }
    }
}
