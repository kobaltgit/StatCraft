use crate::protocol::*;
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

pub struct BridgeClient {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
    next_id: u64,
}

impl BridgeClient {
    pub fn connect(addr: &str) -> Result<Self, std::io::Error> {
        let stream = TcpStream::connect(addr)?;
        stream.set_read_timeout(Some(Duration::from_secs(MAX_TIMEOUT_SECONDS + 10)))?;
        stream.set_write_timeout(Some(Duration::from_secs(30)))?;
        let reader_stream = stream.try_clone()?;
        let reader = BufReader::new(reader_stream);

        Ok(Self {
            stream,
            reader,
            next_id: 1,
        })
    }

    pub fn call(&mut self, method: &str, params: Option<Value>) -> Result<Value, RpcError> {
        let id = self.next_id;
        self.next_id += 1;

        let req = RpcRequest::new(method, params, Some(Value::from(id)));
        let mut req_bytes = serde_json::to_vec(&req)
            .map_err(|e| RpcError::internal_error(format!("Serialization error: {}", e)))?;
        req_bytes.push(b'\n');

        self.stream
            .write_all(&req_bytes)
            .map_err(|e| RpcError::unavailable(format!("Failed to send request: {}", e)))?;
        self.stream
            .flush()
            .map_err(|e| RpcError::unavailable(format!("Failed to flush stream: {}", e)))?;

        let mut line = String::new();
        self.reader
            .read_line(&mut line)
            .map_err(|e| RpcError::unavailable(format!("Failed to read response: {}", e)))?;

        let resp: RpcResponse = serde_json::from_str(line.trim())
            .map_err(|e| RpcError::parse_error(format!("Failed to parse response: {}", e)))?;

        if let Some(err) = resp.error {
            Err(err)
        } else if let Some(res) = resp.result {
            Ok(res)
        } else {
            Err(RpcError::internal_error(
                "Empty response from bridge server",
            ))
        }
    }

    pub fn ping(&mut self) -> Result<String, RpcError> {
        let val = self.call("ping", None)?;
        val.as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| RpcError::internal_error("Expected string response for ping"))
    }

    pub fn execute_syntax(
        &mut self,
        syntax: &str,
        timeout_seconds: Option<u64>,
    ) -> Result<SyntaxExecuteResult, RpcError> {
        let val = self.call(
            "syntax.execute",
            Some(
                serde_json::to_value(SyntaxExecuteParams {
                    syntax: syntax.to_string(),
                    timeout_seconds,
                })
                .map_err(|e| RpcError::internal_error(e.to_string()))?,
            ),
        )?;
        serde_json::from_value(val).map_err(|e| RpcError::internal_error(e.to_string()))
    }

    pub fn load_csv(
        &mut self,
        csv_content: &str,
        name: Option<&str>,
    ) -> Result<DatasetLoadResult, RpcError> {
        let val = self.call(
            "dataset.load_csv",
            Some(
                serde_json::to_value(DatasetLoadCsvParams {
                    csv_content: csv_content.to_string(),
                    name: name.map(|s| s.to_string()),
                })
                .map_err(|e| RpcError::internal_error(e.to_string()))?,
            ),
        )?;
        serde_json::from_value(val).map_err(|e| RpcError::internal_error(e.to_string()))
    }

    pub fn get_variables(&mut self) -> Result<DatasetVariablesResult, RpcError> {
        let val = self.call("dataset.get_variables", None)?;
        serde_json::from_value(val).map_err(|e| RpcError::internal_error(e.to_string()))
    }

    pub fn get_data_preview(
        &mut self,
        limit: Option<usize>,
    ) -> Result<DatasetPreviewResult, RpcError> {
        let val = self.call(
            "dataset.get_data_preview",
            Some(
                serde_json::to_value(DatasetPreviewParams { limit })
                    .map_err(|e| RpcError::internal_error(e.to_string()))?,
            ),
        )?;
        serde_json::from_value(val).map_err(|e| RpcError::internal_error(e.to_string()))
    }

    pub fn add_output_note(
        &mut self,
        title: &str,
        content: &str,
    ) -> Result<OutputAddNoteResult, RpcError> {
        let val = self.call(
            "output.add_note",
            Some(
                serde_json::to_value(OutputAddNoteParams {
                    title: title.to_string(),
                    content: content.to_string(),
                })
                .map_err(|e| RpcError::internal_error(e.to_string()))?,
            ),
        )?;
        serde_json::from_value(val).map_err(|e| RpcError::internal_error(e.to_string()))
    }

    pub fn set_active_tab(&mut self, tab: &str) -> Result<UiSetActiveTabResult, RpcError> {
        let val = self.call(
            "ui.set_active_tab",
            Some(
                serde_json::to_value(UiSetActiveTabParams {
                    tab: tab.to_string(),
                })
                .map_err(|e| RpcError::internal_error(e.to_string()))?,
            ),
        )?;
        serde_json::from_value(val).map_err(|e| RpcError::internal_error(e.to_string()))
    }

    pub fn get_dataset_info(&mut self) -> Result<DatasetInfoResult, RpcError> {
        let val = self.call("dataset.get_info", None)?;
        serde_json::from_value(val).map_err(|e| RpcError::internal_error(e.to_string()))
    }

    pub fn list_commands(&mut self) -> Result<Vec<CommandSpecSummary>, RpcError> {
        let val = self.call("commands.list", None)?;
        serde_json::from_value(val).map_err(|e| RpcError::internal_error(e.to_string()))
    }

    pub fn execute_command(
        &mut self,
        command_id: &str,
        params: Value,
    ) -> Result<ExecuteCommandResult, RpcError> {
        let val = self.call(
            "command.execute",
            Some(
                serde_json::to_value(ExecuteCommandParams {
                    command_id: command_id.to_string(),
                    params,
                })
                .map_err(|e| RpcError::internal_error(e.to_string()))?,
            ),
        )?;
        serde_json::from_value(val).map_err(|e| RpcError::internal_error(e.to_string()))
    }
}
