use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const DEFAULT_BRIDGE_PORT: u16 = 18493;
pub const DEFAULT_TIMEOUT_SECONDS: u64 = 45;
pub const MAX_TIMEOUT_SECONDS: u64 = 300;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    #[serde(default = "default_jsonrpc")]
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

fn default_jsonrpc() -> String {
    "2.0".to_string()
}

impl RpcRequest {
    pub fn new(method: impl Into<String>, params: Option<Value>, id: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.into(),
            params,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

impl RpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, error: RpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl RpcError {
    pub fn parse_error(msg: impl Into<String>) -> Self {
        Self {
            code: -32700,
            message: msg.into(),
            data: None,
        }
    }

    pub fn method_not_found(method: impl Into<String>) -> Self {
        Self {
            code: -32601,
            message: format!("Method not found: {}", method.into()),
            data: None,
        }
    }

    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self {
            code: -32602,
            message: msg.into(),
            data: None,
        }
    }

    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self {
            code: -32603,
            message: msg.into(),
            data: None,
        }
    }

    pub fn unavailable(msg: impl Into<String>) -> Self {
        Self {
            code: -32000,
            message: msg.into(),
            data: None,
        }
    }
}

// ----------------------------------------------------------------------------
// Typed DTOs for Code Mode protocol
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxExecuteParams {
    pub syntax: String,
    #[serde(default)]
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxExecuteResult {
    pub executed_commands: usize,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetLoadCsvParams {
    pub csv_content: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetLoadResult {
    pub rows: usize,
    pub cols: usize,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DatasetVariablesResult {
    pub count: usize,
    pub variables: Vec<VariableInfoSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueLabelSummary {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableInfoSummary {
    pub index: usize,
    pub name: String,
    pub label: Option<String>,
    pub data_type: String,
    pub measure: String,
    pub value_labels: Vec<ValueLabelSummary>,
    pub missing_values: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DatasetPreviewParams {
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetPreviewResult {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub total_rows: usize,
    pub total_cols: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputAddNoteParams {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputAddNoteResult {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSetActiveTabParams {
    pub tab: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSetActiveTabResult {
    pub success: bool,
    pub active_tab: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetInfoResult {
    pub has_dataset: bool,
    pub rows: usize,
    pub cols: usize,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSpecSummary {
    pub id: String,
    pub menu_path: String,
    pub shortcut: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub syntax_names: Vec<String>,
    pub schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteCommandParams {
    pub command_id: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteCommandResult {
    pub command_id: String,
    pub success: bool,
    pub result: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_text: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_request_response_roundtrip() {
        let req = RpcRequest::new(
            "syntax.execute",
            Some(serde_json::json!({
                "syntax": "FREQUENCIES /VARIABLES=x.",
                "timeout_seconds": 30
            })),
            Some(serde_json::json!(1)),
        );
        let serialized = serde_json::to_string(&req).unwrap();
        let parsed: RpcRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(parsed.method, "syntax.execute");

        let res = RpcResponse::success(
            parsed.id,
            serde_json::json!(SyntaxExecuteResult {
                executed_commands: 1,
                message: "OK".to_string(),
                output_text: None,
            }),
        );
        let res_json = serde_json::to_string(&res).unwrap();
        let res_parsed: RpcResponse = serde_json::from_str(&res_json).unwrap();
        assert!(res_parsed.result.is_some());
    }
}
