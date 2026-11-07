use crate::protocol::*;
use serde_json::Value;
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug)]
pub enum BridgePayload {
    ExecuteSyntax(SyntaxExecuteParams),
    LoadCsv(DatasetLoadCsvParams),
    GetVariables,
    GetDataPreview(DatasetPreviewParams),
    AddOutputNote(OutputAddNoteParams),
    SetActiveTab(UiSetActiveTabParams),
    GetDatasetInfo,
    ListCommands,
    ExecuteCommand(ExecuteCommandParams),
}

pub struct BridgeAppRequest {
    pub payload: BridgePayload,
    pub responder: Sender<Result<Value, RpcError>>,
}

#[derive(Clone)]
pub struct BridgeRequestSender {
    sender: Sender<BridgeAppRequest>,
}

impl BridgeRequestSender {
    pub fn new(sender: Sender<BridgeAppRequest>) -> Self {
        Self { sender }
    }

    pub fn send_request(
        &self,
        payload: BridgePayload,
        responder: Sender<Result<Value, RpcError>>,
    ) -> Result<(), RpcError> {
        self.sender
            .send(BridgeAppRequest { payload, responder })
            .map_err(|_| RpcError::unavailable("StatCraft UI event loop is closed"))
    }
}

pub struct BridgeRequestReceiver {
    receiver: Receiver<BridgeAppRequest>,
}

impl BridgeRequestReceiver {
    pub fn new(receiver: Receiver<BridgeAppRequest>) -> Self {
        Self { receiver }
    }

    pub fn try_recv(&self) -> Result<BridgeAppRequest, std::sync::mpsc::TryRecvError> {
        self.receiver.try_recv()
    }

    pub fn recv_timeout(
        &self,
        timeout: std::time::Duration,
    ) -> Result<BridgeAppRequest, std::sync::mpsc::RecvTimeoutError> {
        self.receiver.recv_timeout(timeout)
    }
}

pub fn create_bridge_channel() -> (BridgeRequestSender, BridgeRequestReceiver) {
    let (tx, rx) = channel();
    (BridgeRequestSender::new(tx), BridgeRequestReceiver::new(rx))
}
