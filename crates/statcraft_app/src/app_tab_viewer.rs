use crate::app::{is_tab_closeable, Tab};
use crate::views::{
    render_data_view, render_output_view_with_workspace, render_syntax_editor,
    render_variable_view, DataViewAction, DataViewState, DocumentTabAction, DocumentTabStrip,
    OutputViewerState, SyntaxEditorAction, SyntaxEditorState, VariableViewAction,
    VariableViewState,
};
use egui::Ui;
use egui_dock::TabViewer;
use statcraft_core::{DataSet, WorkspaceManager};
use statcraft_i18n::t;

pub struct StatCraftTabViewer<'a> {
    pub workspace: &'a WorkspaceManager,
    pub document_tab_scroll_offset: &'a mut f32,
    pub pending_document_tab_action: &'a mut DocumentTabAction,
    pub dataset: &'a mut Option<DataSet>,
    pub data_view_state: &'a mut DataViewState,
    pub variable_view_state: &'a mut VariableViewState,
    pub output_view_state: &'a mut OutputViewerState,
    pub syntax_editor_state: &'a mut SyntaxEditorState,
    pub pending_data_action: &'a mut DataViewAction,
    pub pending_variable_action: &'a mut VariableViewAction,
    pub pending_syntax_action: &'a mut SyntaxEditorAction,
}

impl<'a> TabViewer for StatCraftTabViewer<'a> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            Tab::DataView => t!("menu-view-data").into(),
            Tab::VariableView => t!("menu-view-variables").into(),
            Tab::OutputViewer => t!("menu-view-output").into(),
            Tab::SyntaxEditor => t!("menu-file-new-syntax").into(),
        }
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::DataView => {
                let tab_action =
                    DocumentTabStrip::render(ui, self.workspace, self.document_tab_scroll_offset);
                if tab_action != DocumentTabAction::None {
                    *self.pending_document_tab_action = tab_action;
                }
                let action = render_data_view(ui, self.dataset, self.data_view_state);
                if action != DataViewAction::None {
                    *self.pending_data_action = action;
                }
            }
            Tab::VariableView => {
                let tab_action =
                    DocumentTabStrip::render(ui, self.workspace, self.document_tab_scroll_offset);
                if tab_action != DocumentTabAction::None {
                    *self.pending_document_tab_action = tab_action;
                }
                let action = render_variable_view(ui, self.dataset, self.variable_view_state);
                if action != VariableViewAction::None {
                    *self.pending_variable_action = action;
                }
            }
            Tab::OutputViewer => {
                render_output_view_with_workspace(ui, self.output_view_state, Some(self.workspace));
            }
            Tab::SyntaxEditor => {
                let action = render_syntax_editor(ui, self.syntax_editor_state);
                if action != SyntaxEditorAction::None {
                    *self.pending_syntax_action = action;
                }
            }
        }
    }

    fn scroll_bars(&self, tab: &Self::Tab) -> [bool; 2] {
        match tab {
            Tab::DataView | Tab::VariableView | Tab::OutputViewer | Tab::SyntaxEditor => {
                [false, false]
            }
        }
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        is_tab_closeable(*tab)
    }
}
