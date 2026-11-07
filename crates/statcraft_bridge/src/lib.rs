pub mod channel;
pub mod client;
pub mod events;
pub mod mcp;
pub mod protocol;
pub mod server;

pub use channel::{
    create_bridge_channel, BridgeAppRequest, BridgePayload, BridgeRequestReceiver,
    BridgeRequestSender,
};
pub use client::BridgeClient;
pub use events::{current_time_string, BridgeLogEntry, BridgeLogEvent, BridgeLogRing};
pub use protocol::{
    CommandSpecSummary, DatasetInfoResult, DatasetLoadCsvParams, DatasetLoadResult,
    DatasetPreviewParams, DatasetPreviewResult, DatasetVariablesResult, ExecuteCommandParams,
    ExecuteCommandResult, OutputAddNoteParams, OutputAddNoteResult, RpcError, RpcRequest,
    RpcResponse, SyntaxExecuteParams, SyntaxExecuteResult, UiSetActiveTabParams,
    UiSetActiveTabResult, ValueLabelSummary, VariableInfoSummary, DEFAULT_BRIDGE_PORT,
    DEFAULT_TIMEOUT_SECONDS, MAX_TIMEOUT_SECONDS,
};
pub use server::{BridgeServer, BridgeServerError};
