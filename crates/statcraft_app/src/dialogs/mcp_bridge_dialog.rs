use super::common::{DialogShell, GroupBox, SubdialogAction, SubdialogActionBar};
use crate::bridge_service::BridgeService;
use crate::theme::Tokens;
use egui::{vec2, Color32, Context, ScrollArea, Stroke, Ui};
use statcraft_bridge::{BridgeLogEntry, BridgeLogEvent};
use statcraft_i18n::t;

#[derive(Debug, Clone, Default)]
pub struct McpBridgeDialogState {
    pub is_open: bool,
    pub copied_banner: bool,
}

impl McpBridgeDialogState {
    pub fn open(&mut self) {
        self.is_open = true;
        self.copied_banner = false;
    }
}

pub fn render_mcp_bridge_dialog(
    ctx: &Context,
    state: &mut McpBridgeDialogState,
    bridge: &mut BridgeService,
) {
    if !state.is_open {
        return;
    }

    let mut is_open = state.is_open;
    let mut copied_banner = state.copied_banner;

    let should_close = DialogShell::standard(t!("dialog-mcp-bridge-title"))
        .show(ctx, &mut is_open, |ui| {
            ui.vertical(|ui| {
                render_status_header(ui, bridge);
                ui.add_space(8.0);

                render_settings_and_config(ui, &mut copied_banner);
                ui.add_space(8.0);

                render_activity_log(ui, bridge);
                ui.add_space(8.0);

                let sub_action = SubdialogActionBar::new(true).show(ui);
                matches!(
                    sub_action,
                    SubdialogAction::Continue | SubdialogAction::Cancel
                )
            })
            .inner
        })
        .unwrap_or(false);

    if should_close {
        is_open = false;
    }

    state.is_open = is_open;
    state.copied_banner = copied_banner;
}

fn render_status_header(ui: &mut Ui, bridge: &mut BridgeService) {
    GroupBox::new(t!("dialog-mcp-bridge-title")).show(ui, |ui| {
        ui.horizontal(|ui| {
            let (circle_color, status_text) = if bridge.is_active() {
                (
                    Color32::from_rgb(46, 160, 67),
                    t!("dialog-mcp-bridge-status-active", port = bridge.port()),
                )
            } else if bridge.is_port_busy() {
                (
                    Color32::from_rgb(209, 154, 102),
                    t!("dialog-mcp-bridge-status-port-busy", port = bridge.port()),
                )
            } else {
                (
                    Color32::from_rgb(160, 160, 160),
                    t!("dialog-mcp-bridge-status-stopped"),
                )
            };

            // Vector circle status indicator
            let (rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), egui::Sense::hover());
            ui.painter().circle_filled(rect.center(), 5.0, circle_color);
            ui.painter()
                .circle_stroke(rect.center(), 5.0, Stroke::new(1.0_f32, Tokens::border()));

            ui.label(status_text);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if bridge.is_active() {
                    if ui.button(t!("dialog-mcp-bridge-btn-stop")).clicked() {
                        bridge.stop();
                    }
                } else if bridge.is_port_busy() {
                    if ui.button(t!("dialog-mcp-bridge-btn-retry")).clicked() {
                        bridge.start();
                    }
                } else if ui.button(t!("dialog-mcp-bridge-btn-start")).clicked() {
                    bridge.start();
                }
            });
        });

        ui.add_space(4.0);
        let mut autostart = bridge.autostart();
        if ui
            .checkbox(&mut autostart, t!("dialog-mcp-bridge-autostart"))
            .changed()
        {
            bridge.set_autostart(autostart);
        }
    });
}

fn render_settings_and_config(ui: &mut Ui, copied_banner: &mut bool) {
    GroupBox::new(t!("dialog-mcp-bridge-config-title")).show(ui, |ui| {
        let mcp_config_json = r#"{
  "mcpServers": {
    "statcraft": {
      "command": "statcraft.exe",
      "args": ["--mcp"]
    }
  }
}"#;

        ui.monospace(mcp_config_json);
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            if ui.button(t!("dialog-mcp-bridge-copy-config")).clicked() {
                ui.output_mut(|o| o.copied_text = mcp_config_json.to_string());
                *copied_banner = true;
            }

            let skill_prompt = r#"You are the StatCraft Autonomous Data Analyst. You control the active StatCraft desktop statistical suite via MCP Code Mode.

Workflow:
1. Audit dataset: call `statcraft_list_variables` to check variable types and measurement levels, and `statcraft_get_data_preview` to inspect distributions.
2. Discover commands: call `statcraft_list_commands` or read `statcraft://commands` to get the live catalog of all available procedures and JSON parameter schemas.
3. Execute analysis:
   - Run batch SPSS syntax via `statcraft_run_syntax` (e.g. `DESCRIPTIVES`, `FREQUENCIES`, `CROSSTABS`, `T-TEST`, `ONEWAY`, `REGRESSION`, `RELIABILITY`, `FACTOR`). Use `HELP <CMD>.` for syntax templates.
   - Or direct execution via `statcraft_execute_command` or syntax directive `RUN COMMAND "<command_id>" { <json_params> }.`
4. Inspect results: `statcraft_run_syntax` directly returns formatted Pivot Tables with test statistics and exact p-values.
5. Report findings: summarize conclusions and APA-style interpretations into the StatCraft Output Viewer using `statcraft_insert_output_note`."#;

            if ui.button(t!("dialog-mcp-bridge-copy-skill")).clicked() {
                ui.output_mut(|o| o.copied_text = skill_prompt.to_string());
                *copied_banner = true;
            }

            if *copied_banner {
                ui.label(t!("dialog-mcp-bridge-copied"));
            }
        });
    });
}

fn render_activity_log(ui: &mut Ui, bridge: &mut BridgeService) {
    GroupBox::new(t!("dialog-mcp-bridge-log-title")).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(t!("dialog-mcp-bridge-clear-log")).clicked() {
                    bridge.clear_log();
                }
            });
        });

        ui.add_space(4.0);

        let entries = bridge.log_ring().entries();
        if entries.is_empty() {
            ui.label(t!("dialog-mcp-bridge-log-empty"));
        } else {
            ScrollArea::vertical()
                .max_height(120.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for entry in entries.iter().rev() {
                        let text = format_log_entry(entry);
                        ui.label(text);
                    }
                });
        }
    });
}

fn format_log_entry(entry: &BridgeLogEntry) -> String {
    let time = &entry.timestamp;
    match &entry.event {
        BridgeLogEvent::ServerStarted { address } => {
            t!("bridge-log-server-started", time = time, addr = address)
        }
        BridgeLogEvent::ServerStopped => {
            t!("bridge-log-server-stopped", time = time)
        }
        BridgeLogEvent::ClientConnected { address } => {
            t!("bridge-log-client-connected", time = time, addr = address)
        }
        BridgeLogEvent::ClientDisconnected { address } => {
            t!(
                "bridge-log-client-disconnected",
                time = time,
                addr = address
            )
        }
        BridgeLogEvent::VariablesRequested { count } => {
            t!("bridge-log-vars-requested", time = time, count = count)
        }
        BridgeLogEvent::DataPreviewRequested { limit } => {
            t!("bridge-log-preview-requested", time = time, limit = limit)
        }
        BridgeLogEvent::DatasetLoaded { rows, cols } => {
            t!(
                "bridge-log-dataset-loaded",
                time = time,
                rows = rows,
                cols = cols
            )
        }
        BridgeLogEvent::SyntaxExecuted { preview, count } => {
            t!(
                "bridge-log-syntax-executed",
                time = time,
                cmd = preview,
                count = count
            )
        }
        BridgeLogEvent::CommandExecuted { command_id } => {
            format!("{} [Command] {}", time, command_id)
        }
        BridgeLogEvent::OutputNoteAdded { title } => {
            t!("bridge-log-note-added", time = time, title = title)
        }
        BridgeLogEvent::TabSwitched { tab } => {
            t!("bridge-log-tab-switched", time = time, tab = tab)
        }
        BridgeLogEvent::Error { message } => {
            t!("bridge-log-error", time = time, message = message)
        }
    }
}
