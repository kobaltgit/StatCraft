use egui::{vec2, Align2, Context, Id, Ui, Vec2, Window};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DialogSize {
    /// 560 x 430 px - Type 1: Statistical Procedure dialogs
    Standard,
    /// 360 x 300 px - Type 2: Subdialogs / Options
    Subdialog,
    /// 460 x 380 px - Type 3: Variable Properties
    Properties,
    /// 620 x 480 px - Type 4: Compute & Transform
    Compute,
    /// Custom dimensions
    Custom(Vec2),
}

impl DialogSize {
    pub fn to_vec2(self) -> Vec2 {
        match self {
            DialogSize::Standard => vec2(560.0, 430.0),
            DialogSize::Subdialog => vec2(360.0, 300.0),
            DialogSize::Properties => vec2(460.0, 380.0),
            DialogSize::Compute => vec2(620.0, 480.0),
            DialogSize::Custom(v) => v,
        }
    }
}

pub struct DialogShell {
    title: String,
    size: DialogSize,
    resizable: bool,
    collapsible: bool,
    id: Option<Id>,
}

impl DialogShell {
    pub fn new(title: impl Into<String>, size: DialogSize) -> Self {
        Self {
            title: title.into(),
            size,
            resizable: false,
            collapsible: false,
            id: None,
        }
    }

    pub fn standard(title: impl Into<String>) -> Self {
        Self::new(title, DialogSize::Standard)
    }

    pub fn subdialog(title: impl Into<String>) -> Self {
        Self::new(title, DialogSize::Subdialog)
    }

    pub fn properties(title: impl Into<String>) -> Self {
        Self::new(title, DialogSize::Properties)
    }

    pub fn compute(title: impl Into<String>) -> Self {
        Self::new(title, DialogSize::Compute)
    }

    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn show<R>(
        self,
        ctx: &Context,
        open: &mut bool,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> Option<R> {
        let size = self.size.to_vec2();
        let mut window = Window::new(&self.title)
            .open(open)
            .resizable(self.resizable)
            .collapsible(self.collapsible)
            .fixed_size(size)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0]);

        if let Some(id) = self.id {
            window = window.id(id);
        }

        let inner_response = window.show(ctx, |ui| {
            ui.set_width(size.x - 24.0);
            add_contents(ui)
        });

        inner_response.and_then(|r| r.inner)
    }
}
