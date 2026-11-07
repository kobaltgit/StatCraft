use crate::app::Tab;
use crate::syntax_runner;
use crate::views::output_view::{OutputItem, OutputItemKind, OutputProcedure, OutputViewerState};
use egui_dock::DockState;
use statcraft_bridge::{
    create_bridge_channel, BridgeAppRequest, BridgeLogEvent, BridgeLogRing, BridgePayload,
    BridgeRequestReceiver, BridgeRequestSender, BridgeServer, BridgeServerError, DatasetInfoResult,
    DatasetLoadResult, DatasetPreviewResult, DatasetVariablesResult, ExecuteCommandResult,
    OutputAddNoteResult, RpcError, SyntaxExecuteResult, UiSetActiveTabResult, ValueLabelSummary,
    VariableInfoSummary, DEFAULT_BRIDGE_PORT,
};
use statcraft_core::{DataSet, Measure};
use statcraft_io::{load_csv_from_reader, CsvOptions};
use statcraft_syntax::CommandBus;
use std::io::Cursor;

pub struct BridgeService {
    server: Option<BridgeServer>,
    sender: BridgeRequestSender,
    receiver: BridgeRequestReceiver,
    log_ring: BridgeLogRing,
    autostart: bool,
    port_busy: bool,
    port: u16,
}

impl Default for BridgeService {
    fn default() -> Self {
        Self::new(true)
    }
}

impl BridgeService {
    pub fn new(autostart: bool) -> Self {
        let (sender, receiver) = create_bridge_channel();
        let mut service = Self {
            server: None,
            sender,
            receiver,
            log_ring: BridgeLogRing::new(100),
            autostart,
            port_busy: false,
            port: DEFAULT_BRIDGE_PORT,
        };

        if autostart {
            service.start();
        }

        service
    }

    pub fn start(&mut self) {
        if self.server.is_some() {
            return;
        }

        match BridgeServer::start(self.port, self.sender.clone()) {
            Ok(server) => {
                let addr = server.local_addr().to_string();
                self.server = Some(server);
                self.port_busy = false;
                self.log_ring
                    .push(BridgeLogEvent::ServerStarted { address: addr });
            }
            Err(BridgeServerError::PortInUse { port, .. }) => {
                self.port_busy = true;
                self.log_ring.push(BridgeLogEvent::Error {
                    message: format!("Port {} is already in use by another instance", port),
                });
            }
            Err(e) => {
                self.port_busy = false;
                self.log_ring.push(BridgeLogEvent::Error {
                    message: e.to_string(),
                });
            }
        }
    }

    pub fn stop(&mut self) {
        if let Some(mut server) = self.server.take() {
            server.stop();
            self.log_ring.push(BridgeLogEvent::ServerStopped);
        }
        self.port_busy = false;
    }

    pub fn is_active(&self) -> bool {
        self.server
            .as_ref()
            .map(|s| s.is_running())
            .unwrap_or(false)
    }

    pub fn is_port_busy(&self) -> bool {
        self.port_busy
    }

    pub fn autostart(&self) -> bool {
        self.autostart
    }

    pub fn set_autostart(&mut self, autostart: bool) {
        self.autostart = autostart;
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn log_ring(&self) -> &BridgeLogRing {
        &self.log_ring
    }

    pub fn clear_log(&mut self) {
        self.log_ring.clear();
    }

    pub fn process_requests(
        &mut self,
        dataset: &mut Option<DataSet>,
        bus: &mut CommandBus,
        output_view: &mut OutputViewerState,
        dock_state: &mut DockState<Tab>,
    ) {
        while let Ok(req) = self.receiver.try_recv() {
            self.handle_single_request(req, dataset, bus, output_view, dock_state);
        }
    }

    fn handle_single_request(
        &mut self,
        req: BridgeAppRequest,
        dataset: &mut Option<DataSet>,
        bus: &mut CommandBus,
        output_view: &mut OutputViewerState,
        dock_state: &mut DockState<Tab>,
    ) {
        let responder = req.responder;
        match req.payload {
            BridgePayload::ExecuteSyntax(params) => {
                let preview = if params.syntax.len() > 60 {
                    format!("{}...", &params.syntax[..57])
                } else {
                    params.syntax.clone()
                };

                match syntax_runner::run_syntax_batch(&params.syntax, dataset, bus, output_view) {
                    Ok(outcome) => {
                        self.log_ring.push(BridgeLogEvent::SyntaxExecuted {
                            preview,
                            count: outcome.count,
                        });
                        let _ = responder.send(Ok(serde_json::to_value(SyntaxExecuteResult {
                            executed_commands: outcome.count,
                            message: "Commands executed successfully".to_string(),
                            output_text: if outcome.output_text.is_empty() {
                                None
                            } else {
                                Some(outcome.output_text)
                            },
                        })
                        .unwrap_or_default()));
                    }
                    Err(err) => {
                        let err_msg = err.to_string();
                        self.log_ring.push(BridgeLogEvent::Error {
                            message: err_msg.clone(),
                        });
                        let _ = responder.send(Err(RpcError::internal_error(err_msg)));
                    }
                }
            }
            BridgePayload::LoadCsv(params) => {
                let ds_name = params.name.unwrap_or_else(|| "ImportedDataset".to_string());
                let cursor = Cursor::new(params.csv_content.as_bytes());
                match load_csv_from_reader(cursor, CsvOptions::default()) {
                    Ok(raw_table) => match DataSet::from_raw_table(raw_table) {
                        Ok(new_ds) => {
                            let rows = new_ds.row_count();
                            let cols = new_ds.col_count();
                            *dataset = Some(new_ds);
                            self.log_ring
                                .push(BridgeLogEvent::DatasetLoaded { rows, cols });
                            let _ = responder.send(Ok(serde_json::to_value(DatasetLoadResult {
                                rows,
                                cols,
                                name: ds_name,
                            })
                            .unwrap_or_default()));
                        }
                        Err(e) => {
                            let err_msg = format!("Failed to create dataset: {}", e);
                            self.log_ring.push(BridgeLogEvent::Error {
                                message: err_msg.clone(),
                            });
                            let _ = responder.send(Err(RpcError::internal_error(err_msg)));
                        }
                    },
                    Err(e) => {
                        let err_msg = format!("CSV parse error: {}", e);
                        self.log_ring.push(BridgeLogEvent::Error {
                            message: err_msg.clone(),
                        });
                        let _ = responder.send(Err(RpcError::invalid_params(err_msg)));
                    }
                }
            }
            BridgePayload::GetVariables => {
                let res = if let Some(ds) = dataset.as_ref() {
                    let vars = extract_variables_summary(ds);
                    let count = vars.len();
                    self.log_ring
                        .push(BridgeLogEvent::VariablesRequested { count });
                    DatasetVariablesResult {
                        count,
                        variables: vars,
                    }
                } else {
                    DatasetVariablesResult::default()
                };
                let _ = responder.send(Ok(serde_json::to_value(res).unwrap_or_default()));
            }
            BridgePayload::GetDataPreview(params) => {
                let limit = params.limit.unwrap_or(10);
                let res = if let Some(ds) = dataset.as_ref() {
                    self.log_ring
                        .push(BridgeLogEvent::DataPreviewRequested { limit });
                    extract_data_preview(ds, limit)
                } else {
                    DatasetPreviewResult {
                        headers: vec![],
                        rows: vec![],
                        total_rows: 0,
                        total_cols: 0,
                    }
                };
                let _ = responder.send(Ok(serde_json::to_value(res).unwrap_or_default()));
            }
            BridgePayload::AddOutputNote(params) => {
                let proc_id = output_view.alloc_id();
                let item_id = output_view.alloc_id();
                let proc = OutputProcedure {
                    id: proc_id,
                    title_key: "output-notes",
                    custom_title: Some(params.title.clone()),
                    is_expanded: true,
                    items: vec![OutputItem {
                        id: item_id,
                        title_key: "output-notes",
                        custom_title: Some(params.title.clone()),
                        kind: OutputItemKind::CustomText {
                            title: params.title.clone(),
                            content: params.content,
                        },
                    }],
                };
                output_view.procedures.push(proc);
                self.log_ring.push(BridgeLogEvent::OutputNoteAdded {
                    title: params.title,
                });
                let _ = responder.send(Ok(serde_json::to_value(OutputAddNoteResult {
                    success: true,
                    message: "Note successfully added to Output Viewer".to_string(),
                })
                .unwrap_or_default()));
            }
            BridgePayload::SetActiveTab(params) => {
                let target_tab = match params.tab.to_ascii_lowercase().as_str() {
                    "data" => Some(Tab::DataView),
                    "variable" => Some(Tab::VariableView),
                    "output" => Some(Tab::OutputViewer),
                    "syntax" => Some(Tab::SyntaxEditor),
                    _ => None,
                };

                if let Some(tab) = target_tab {
                    if let Some(pos) = dock_state.find_tab(&tab) {
                        dock_state.set_active_tab(pos);
                    }
                    self.log_ring.push(BridgeLogEvent::TabSwitched {
                        tab: params.tab.clone(),
                    });
                    let _ = responder.send(Ok(serde_json::to_value(UiSetActiveTabResult {
                        success: true,
                        active_tab: params.tab,
                    })
                    .unwrap_or_default()));
                } else {
                    let _ = responder.send(Err(RpcError::invalid_params(format!(
                        "Invalid tab '{}'. Allowed: 'data', 'variable', 'output', 'syntax'",
                        params.tab
                    ))));
                }
            }
            BridgePayload::GetDatasetInfo => {
                let res = if let Some(ds) = dataset.as_ref() {
                    DatasetInfoResult {
                        has_dataset: true,
                        rows: ds.row_count(),
                        cols: ds.col_count(),
                        name: None,
                    }
                } else {
                    DatasetInfoResult {
                        has_dataset: false,
                        rows: 0,
                        cols: 0,
                        name: None,
                    }
                };
                let _ = responder.send(Ok(serde_json::to_value(res).unwrap_or_default()));
            }
            BridgePayload::ListCommands => {
                let specs = bus.list_specs();
                let _ = responder.send(Ok(serde_json::to_value(specs).unwrap_or_default()));
            }
            BridgePayload::ExecuteCommand(params) => {
                let initial_proc_count = output_view.procedures.len();
                match bus.execute(&params.command_id, dataset, params.params.clone()) {
                    Ok(res_val) => {
                        let raw_log =
                            format!("RUN COMMAND \"{}\" {}", params.command_id, params.params);
                        output_view.add_generic_result(&params.command_id, &raw_log, &res_val);

                        let output_text = if output_view.procedures.len() > initial_proc_count {
                            crate::views::output_view::export_procedures_to_txt(
                                &output_view.procedures[initial_proc_count..],
                            )
                        } else {
                            String::new()
                        };

                        self.log_ring.push(BridgeLogEvent::SyntaxExecuted {
                            preview: format!("CMD: {}", params.command_id),
                            count: 1,
                        });

                        let _ = responder.send(Ok(serde_json::to_value(ExecuteCommandResult {
                            command_id: params.command_id,
                            success: true,
                            result: res_val,
                            output_text: if output_text.is_empty() {
                                None
                            } else {
                                Some(output_text)
                            },
                        })
                        .unwrap_or_default()));
                    }
                    Err(err) => {
                        let err_msg = err.to_string();
                        output_view.add_log(format!("Error: {}", err_msg));
                        self.log_ring.push(BridgeLogEvent::Error {
                            message: err_msg.clone(),
                        });
                        let _ = responder.send(Err(RpcError::internal_error(err_msg)));
                    }
                }
            }
        }
    }
}

fn extract_variables_summary(ds: &DataSet) -> Vec<VariableInfoSummary> {
    let mut vars = Vec::with_capacity(ds.col_count());
    for i in 0..ds.col_count() {
        if let Some(meta) = ds.dictionary.get(i) {
            let measure_str = match meta.measure {
                Measure::Scale => "Scale",
                Measure::Ordinal => "Ordinal",
                Measure::Nominal => "Nominal",
            };

            let val_labels: Vec<ValueLabelSummary> = meta
                .values
                .mapping
                .iter()
                .map(|(val, lbl)| ValueLabelSummary {
                    value: val.clone(),
                    label: lbl.clone(),
                })
                .collect();

            let missing_str = if !meta.missing.discrete.is_empty() {
                Some(meta.missing.discrete.join(", "))
            } else if let Some((low, high)) = meta.missing.range {
                Some(format!("{} THRU {}", low, high))
            } else {
                None
            };

            vars.push(VariableInfoSummary {
                index: i,
                name: meta.name.clone(),
                label: if meta.label.is_empty() {
                    None
                } else {
                    Some(meta.label.clone())
                },
                data_type: format!("{:?}", meta.var_type),
                measure: measure_str.to_string(),
                value_labels: val_labels,
                missing_values: missing_str,
            });
        }
    }
    vars
}

fn extract_data_preview(ds: &DataSet, limit: usize) -> DatasetPreviewResult {
    let col_count = ds.col_count();
    let total_rows = ds.row_count();
    let preview_rows_count = total_rows.min(limit);

    let mut headers = Vec::with_capacity(col_count);
    for i in 0..col_count {
        if let Some(meta) = ds.dictionary.get(i) {
            headers.push(meta.name.clone());
        } else {
            headers.push(format!("VAR{:05}", i + 1));
        }
    }

    let mut rows = Vec::with_capacity(preview_rows_count);
    for r in 0..preview_rows_count {
        let mut row_vals = Vec::with_capacity(col_count);
        for c in 0..col_count {
            row_vals.push(ds.get_cell_display(r, c, false));
        }
        rows.push(row_vals);
    }

    DatasetPreviewResult {
        headers,
        rows,
        total_rows,
        total_cols: col_count,
    }
}
