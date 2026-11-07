use crate::channel::{BridgePayload, BridgeRequestSender};
use crate::protocol::*;
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BridgeServerError {
    #[error("Bridge port {port} is already in use by another process: {source}")]
    PortInUse {
        port: u16,
        #[source]
        source: std::io::Error,
    },
    #[error("Failed to bind server on {addr}: {source}")]
    IoError {
        addr: String,
        #[source]
        source: std::io::Error,
    },
}

pub struct BridgeServer {
    running: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<()>>,
    local_addr: String,
}

impl BridgeServer {
    pub fn start(
        port: u16,
        request_sender: BridgeRequestSender,
    ) -> Result<Self, BridgeServerError> {
        let addr_str = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr_str).map_err(|e| {
            if e.kind() == std::io::ErrorKind::AddrInUse {
                BridgeServerError::PortInUse { port, source: e }
            } else {
                BridgeServerError::IoError {
                    addr: addr_str.clone(),
                    source: e,
                }
            }
        })?;

        let bound_addr = listener
            .local_addr()
            .map(|a| a.to_string())
            .unwrap_or(addr_str.clone());

        // Non-blocking so we can check running flag periodically
        let _ = listener.set_nonblocking(true);

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);

        let thread_handle = thread::Builder::new()
            .name("statcraft-bridge-server".to_string())
            .spawn(move || {
                run_server_loop(listener, request_sender, running_clone);
            })
            .map_err(|e| BridgeServerError::IoError {
                addr: bound_addr.clone(),
                source: e,
            })?;

        Ok(Self {
            running,
            thread_handle: Some(thread_handle),
            local_addr: bound_addr,
        })
    }

    pub fn local_addr(&self) -> &str {
        &self.local_addr
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn stop(&mut self) {
        if self.running.swap(false, Ordering::SeqCst) {
            if let Some(handle) = self.thread_handle.take() {
                let _ = handle.join();
            }
        }
    }
}

impl Drop for BridgeServer {
    fn drop(&mut self) {
        self.stop();
    }
}

fn run_server_loop(
    listener: TcpListener,
    request_sender: BridgeRequestSender,
    running: Arc<AtomicBool>,
) {
    while running.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, _peer_addr)) => {
                let req_sender = request_sender.clone();
                let running_sub = Arc::clone(&running);
                // Handle client session in a lightweight thread
                let _ = thread::Builder::new()
                    .name("statcraft-bridge-client".to_string())
                    .spawn(move || {
                        handle_client_connection(stream, req_sender, running_sub);
                    });
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(_) => {
                thread::sleep(Duration::from_millis(50));
            }
        }
    }
}

fn handle_client_connection(
    mut stream: TcpStream,
    request_sender: BridgeRequestSender,
    running: Arc<AtomicBool>,
) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));

    let reader_stream = match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    };
    let mut reader = BufReader::new(reader_stream);

    let mut line = String::new();
    while running.load(Ordering::SeqCst) {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break, // EOF, client disconnected
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let response = match serde_json::from_str::<RpcRequest>(trimmed) {
                    Ok(req) => process_rpc_request(req, &request_sender),
                    Err(e) => RpcResponse::error(None, RpcError::parse_error(e.to_string())),
                };

                let mut out_bytes = match serde_json::to_vec(&response) {
                    Ok(b) => b,
                    Err(_) => continue,
                };
                out_bytes.push(b'\n');

                if stream.write_all(&out_bytes).is_err() {
                    break;
                }
                let _ = stream.flush();
            }
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                // Timeout on read, continue checking running
                continue;
            }
            Err(_) => break,
        }
    }
}

fn process_rpc_request(req: RpcRequest, request_sender: &BridgeRequestSender) -> RpcResponse {
    let id = req.id.clone();
    let method = req.method.as_str();

    match method {
        "ping" => RpcResponse::success(id, Value::String("pong".to_string())),
        "syntax.execute" => {
            let params: SyntaxExecuteParams = match req.params {
                Some(p) => match serde_json::from_value(p) {
                    Ok(v) => v,
                    Err(e) => {
                        return RpcResponse::error(id, RpcError::invalid_params(e.to_string()))
                    }
                },
                None => {
                    return RpcResponse::error(
                        id,
                        RpcError::invalid_params("Missing parameters for syntax.execute"),
                    )
                }
            };

            let timeout_secs = params
                .timeout_seconds
                .unwrap_or(DEFAULT_TIMEOUT_SECONDS)
                .min(MAX_TIMEOUT_SECONDS);

            let (tx, rx) = channel();
            if let Err(e) = request_sender.send_request(BridgePayload::ExecuteSyntax(params), tx) {
                return RpcResponse::error(id, e);
            }

            match rx.recv_timeout(Duration::from_secs(timeout_secs)) {
                Ok(Ok(val)) => RpcResponse::success(id, val),
                Ok(Err(err)) => RpcResponse::error(id, err),
                Err(_) => RpcResponse::error(
                    id,
                    RpcError::unavailable(
                        "Timed out waiting for syntax execution in StatCraft GUI",
                    ),
                ),
            }
        }
        "dataset.load_csv" => {
            let params: DatasetLoadCsvParams = match req.params {
                Some(p) => match serde_json::from_value(p) {
                    Ok(v) => v,
                    Err(e) => {
                        return RpcResponse::error(id, RpcError::invalid_params(e.to_string()))
                    }
                },
                None => {
                    return RpcResponse::error(
                        id,
                        RpcError::invalid_params("Missing parameters for dataset.load_csv"),
                    )
                }
            };

            let (tx, rx) = channel();
            if let Err(e) = request_sender.send_request(BridgePayload::LoadCsv(params), tx) {
                return RpcResponse::error(id, e);
            }

            match rx.recv_timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECONDS)) {
                Ok(Ok(val)) => RpcResponse::success(id, val),
                Ok(Err(err)) => RpcResponse::error(id, err),
                Err(_) => RpcResponse::error(
                    id,
                    RpcError::unavailable("Timed out waiting for dataset load in StatCraft GUI"),
                ),
            }
        }
        "dataset.get_variables" => {
            let (tx, rx) = channel();
            if let Err(e) = request_sender.send_request(BridgePayload::GetVariables, tx) {
                return RpcResponse::error(id, e);
            }

            match rx.recv_timeout(Duration::from_secs(10)) {
                Ok(Ok(val)) => RpcResponse::success(id, val),
                Ok(Err(err)) => RpcResponse::error(id, err),
                Err(_) => RpcResponse::error(
                    id,
                    RpcError::unavailable("Timed out waiting for variables in StatCraft GUI"),
                ),
            }
        }
        "dataset.get_data_preview" => {
            let params: DatasetPreviewParams = match req.params {
                Some(p) => match serde_json::from_value(p) {
                    Ok(v) => v,
                    Err(e) => {
                        return RpcResponse::error(id, RpcError::invalid_params(e.to_string()))
                    }
                },
                None => DatasetPreviewParams::default(),
            };

            let (tx, rx) = channel();
            if let Err(e) = request_sender.send_request(BridgePayload::GetDataPreview(params), tx) {
                return RpcResponse::error(id, e);
            }

            match rx.recv_timeout(Duration::from_secs(10)) {
                Ok(Ok(val)) => RpcResponse::success(id, val),
                Ok(Err(err)) => RpcResponse::error(id, err),
                Err(_) => RpcResponse::error(
                    id,
                    RpcError::unavailable("Timed out waiting for data preview in StatCraft GUI"),
                ),
            }
        }
        "output.add_note" => {
            let params: OutputAddNoteParams = match req.params {
                Some(p) => match serde_json::from_value(p) {
                    Ok(v) => v,
                    Err(e) => {
                        return RpcResponse::error(id, RpcError::invalid_params(e.to_string()))
                    }
                },
                None => {
                    return RpcResponse::error(
                        id,
                        RpcError::invalid_params("Missing parameters for output.add_note"),
                    )
                }
            };

            let (tx, rx) = channel();
            if let Err(e) = request_sender.send_request(BridgePayload::AddOutputNote(params), tx) {
                return RpcResponse::error(id, e);
            }

            match rx.recv_timeout(Duration::from_secs(10)) {
                Ok(Ok(val)) => RpcResponse::success(id, val),
                Ok(Err(err)) => RpcResponse::error(id, err),
                Err(_) => RpcResponse::error(
                    id,
                    RpcError::unavailable("Timed out waiting to add output note"),
                ),
            }
        }
        "ui.set_active_tab" => {
            let params: UiSetActiveTabParams = match req.params {
                Some(p) => match serde_json::from_value(p) {
                    Ok(v) => v,
                    Err(e) => {
                        return RpcResponse::error(id, RpcError::invalid_params(e.to_string()))
                    }
                },
                None => {
                    return RpcResponse::error(
                        id,
                        RpcError::invalid_params("Missing parameters for ui.set_active_tab"),
                    )
                }
            };

            let (tx, rx) = channel();
            if let Err(e) = request_sender.send_request(BridgePayload::SetActiveTab(params), tx) {
                return RpcResponse::error(id, e);
            }

            match rx.recv_timeout(Duration::from_secs(10)) {
                Ok(Ok(val)) => RpcResponse::success(id, val),
                Ok(Err(err)) => RpcResponse::error(id, err),
                Err(_) => RpcResponse::error(
                    id,
                    RpcError::unavailable("Timed out waiting to set active tab"),
                ),
            }
        }
        "dataset.get_info" => {
            let (tx, rx) = channel();
            if let Err(e) = request_sender.send_request(BridgePayload::GetDatasetInfo, tx) {
                return RpcResponse::error(id, e);
            }

            match rx.recv_timeout(Duration::from_secs(10)) {
                Ok(Ok(val)) => RpcResponse::success(id, val),
                Ok(Err(err)) => RpcResponse::error(id, err),
                Err(_) => RpcResponse::error(
                    id,
                    RpcError::unavailable("Timed out waiting for dataset info"),
                ),
            }
        }
        "commands.list" => {
            let (tx, rx) = channel();
            if let Err(e) = request_sender.send_request(BridgePayload::ListCommands, tx) {
                return RpcResponse::error(id, e);
            }

            match rx.recv_timeout(Duration::from_secs(10)) {
                Ok(Ok(val)) => RpcResponse::success(id, val),
                Ok(Err(err)) => RpcResponse::error(id, err),
                Err(_) => RpcResponse::error(
                    id,
                    RpcError::unavailable("Timed out waiting for commands list from StatCraft"),
                ),
            }
        }
        "command.execute" => {
            let params: ExecuteCommandParams = match req.params {
                Some(p) => match serde_json::from_value(p) {
                    Ok(v) => v,
                    Err(e) => {
                        return RpcResponse::error(id, RpcError::invalid_params(e.to_string()))
                    }
                },
                None => {
                    return RpcResponse::error(
                        id,
                        RpcError::invalid_params("Missing parameters for command.execute"),
                    )
                }
            };

            let (tx, rx) = channel();
            if let Err(e) = request_sender.send_request(BridgePayload::ExecuteCommand(params), tx) {
                return RpcResponse::error(id, e);
            }

            match rx.recv_timeout(Duration::from_secs(MAX_TIMEOUT_SECONDS)) {
                Ok(Ok(val)) => RpcResponse::success(id, val),
                Ok(Err(err)) => RpcResponse::error(id, err),
                Err(_) => RpcResponse::error(
                    id,
                    RpcError::unavailable("Timed out waiting for command execution in StatCraft"),
                ),
            }
        }
        other => RpcResponse::error(id, RpcError::method_not_found(other)),
    }
}
