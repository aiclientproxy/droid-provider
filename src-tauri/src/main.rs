//! Droid Provider CLI - OAuth Provider Plugin for ProxyCast
//!
//! 这是一个独立的 CLI 工具，通过 JSON-RPC 与 ProxyCast 通信。
//! 支持 WorkOS OAuth 和 API Key 两种认证方式。

mod auth;
mod credentials;
mod provider;
mod token_refresh;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};
use tracing::{debug, info};

/// Droid Provider CLI
#[derive(Parser)]
#[command(name = "droid-provider-cli")]
#[command(about = "Droid Provider Plugin - Factory.ai Droid 平台支持")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Run in JSON-RPC mode (stdin/stdout)
    #[arg(long)]
    json_rpc: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Get plugin info
    Info,
    /// List supported models
    Models,
    /// Validate a credential
    Validate {
        #[arg(long)]
        credential_id: String,
    },
    /// Refresh token
    Refresh {
        #[arg(long)]
        credential_id: String,
    },
}

/// JSON-RPC Request
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: serde_json::Value,
}

/// JSON-RPC Response
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
    id: serde_json::Value,
}

/// JSON-RPC Error
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<serde_json::Value>,
}

impl JsonRpcResponse {
    fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    fn error(id: serde_json::Value, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
            id,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("droid_provider=debug".parse().unwrap()),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    if cli.json_rpc {
        run_json_rpc_mode().await?;
    } else if let Some(command) = cli.command {
        match command {
            Commands::Info => {
                let info = get_plugin_info();
                println!("{}", serde_json::to_string_pretty(&info)?);
            }
            Commands::Models => {
                let models = provider::list_models();
                println!("{}", serde_json::to_string_pretty(&models)?);
            }
            Commands::Validate { credential_id } => {
                info!("Validating credential: {}", credential_id);
                match provider::validate_credential(&credential_id).await {
                    Ok(result) => println!("{}", serde_json::to_string_pretty(&result)?),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            Commands::Refresh { credential_id } => {
                info!("Refreshing token for: {}", credential_id);
                match provider::refresh_token(&credential_id).await {
                    Ok(result) => println!("{}", serde_json::to_string_pretty(&result)?),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        }
    } else {
        // Default: print info
        let info = get_plugin_info();
        println!("{}", serde_json::to_string_pretty(&info)?);
    }

    Ok(())
}

/// Run in JSON-RPC mode
async fn run_json_rpc_mode() -> anyhow::Result<()> {
    info!("Starting Droid Provider in JSON-RPC mode");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        debug!("Received: {}", line);

        let response = match serde_json::from_str::<JsonRpcRequest>(&line) {
            Ok(request) => handle_request(request).await,
            Err(e) => JsonRpcResponse::error(
                serde_json::Value::Null,
                -32700,
                format!("Parse error: {}", e),
            ),
        };

        let response_str = serde_json::to_string(&response)?;
        debug!("Sending: {}", response_str);

        writeln!(stdout, "{}", response_str)?;
        stdout.flush()?;
    }

    Ok(())
}

/// Handle a JSON-RPC request
async fn handle_request(request: JsonRpcRequest) -> JsonRpcResponse {
    let id = request.id.clone();

    match request.method.as_str() {
        "get_info" => {
            let info = get_plugin_info();
            JsonRpcResponse::success(id, serde_json::to_value(info).unwrap())
        }
        "list_models" => {
            let models = provider::list_models();
            JsonRpcResponse::success(id, serde_json::to_value(models).unwrap())
        }
        "supports_model" => {
            let model = request.params["model"].as_str().unwrap_or("");
            let supports = provider::supports_model(model);
            JsonRpcResponse::success(id, serde_json::json!({ "supports": supports }))
        }
        "acquire_credential" => {
            let model = request.params["model"].as_str().unwrap_or("");
            match provider::acquire_credential(model).await {
                Ok(credential) => {
                    JsonRpcResponse::success(id, serde_json::to_value(credential).unwrap())
                }
                Err(e) => JsonRpcResponse::error(id, -32000, e.to_string()),
            }
        }
        "release_credential" => {
            let credential_id = request.params["credential_id"].as_str().unwrap_or("");
            let result = &request.params["result"];
            match provider::release_credential(credential_id, result.clone()).await {
                Ok(_) => JsonRpcResponse::success(id, serde_json::json!({})),
                Err(e) => JsonRpcResponse::error(id, -32000, e.to_string()),
            }
        }
        "validate_credential" => {
            let credential_id = request.params["credential_id"].as_str().unwrap_or("");
            match provider::validate_credential(credential_id).await {
                Ok(result) => JsonRpcResponse::success(id, serde_json::to_value(result).unwrap()),
                Err(e) => JsonRpcResponse::error(id, -32000, e.to_string()),
            }
        }
        "refresh_token" => {
            let credential_id = request.params["credential_id"].as_str().unwrap_or("");
            match provider::refresh_token(credential_id).await {
                Ok(result) => JsonRpcResponse::success(id, serde_json::to_value(result).unwrap()),
                Err(e) => JsonRpcResponse::error(id, -32000, e.to_string()),
            }
        }
        "create_credential" => {
            let auth_type = request.params["auth_type"].as_str().unwrap_or("oauth");
            let config = request.params["config"].clone();
            match provider::create_credential(auth_type, config).await {
                Ok(credential_id) => {
                    JsonRpcResponse::success(id, serde_json::json!({ "credential_id": credential_id }))
                }
                Err(e) => JsonRpcResponse::error(id, -32000, e.to_string()),
            }
        }
        "transform_request" => {
            let request_body = request.params["request"].clone();
            match provider::transform_request(request_body).await {
                Ok(transformed) => {
                    JsonRpcResponse::success(id, serde_json::json!({ "request": transformed }))
                }
                Err(e) => JsonRpcResponse::error(id, -32000, e.to_string()),
            }
        }
        "transform_response" => {
            let response_body = request.params["response"].clone();
            match provider::transform_response(response_body).await {
                Ok(transformed) => {
                    JsonRpcResponse::success(id, serde_json::json!({ "response": transformed }))
                }
                Err(e) => JsonRpcResponse::error(id, -32000, e.to_string()),
            }
        }
        "apply_risk_control" => {
            let mut request_body = request.params["request"].clone();
            let credential_id = request.params["credential_id"].as_str().unwrap_or("");
            match provider::apply_risk_control(&mut request_body, credential_id).await {
                Ok(_) => {
                    JsonRpcResponse::success(id, serde_json::json!({ "request": request_body }))
                }
                Err(e) => JsonRpcResponse::error(id, -32000, e.to_string()),
            }
        }
        "parse_error" => {
            let status = request.params["status"].as_u64().unwrap_or(0) as u16;
            let body = request.params["body"].as_str().unwrap_or("");
            let error = provider::parse_error(status, body);
            JsonRpcResponse::success(id, serde_json::to_value(error).unwrap_or_default())
        }
        _ => JsonRpcResponse::error(id, -32601, format!("Method not found: {}", request.method)),
    }
}

/// Get plugin info
fn get_plugin_info() -> serde_json::Value {
    serde_json::json!({
        "id": "droid",
        "display_name": "Droid (Factory.ai)",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Factory.ai Droid 平台支持，支持 WorkOS OAuth 和 API Key 认证",
        "target_protocol": "anthropic",
        "category": "oauth",
        "auth_types": [
            {
                "id": "oauth",
                "display_name": "WorkOS OAuth",
                "description": "使用 WorkOS OAuth 授权登录 Factory.ai",
                "category": "oauth",
                "icon": "Key"
            },
            {
                "id": "api_key",
                "display_name": "API Key",
                "description": "使用 Factory.ai API Key 认证",
                "category": "api_key",
                "icon": "KeyRound"
            }
        ],
        "model_families": [
            {
                "name": "opus",
                "pattern": "claude-opus-*",
                "tier": 3,
                "description": "Claude Opus - 最强能力"
            },
            {
                "name": "sonnet",
                "pattern": "claude-*-sonnet*",
                "tier": 2,
                "description": "Claude Sonnet - 均衡选择"
            },
            {
                "name": "gpt",
                "pattern": "gpt-*",
                "tier": 3,
                "description": "GPT 系列模型"
            },
            {
                "name": "all",
                "pattern": "*",
                "tier": null,
                "description": "所有支持的模型"
            }
        ],
        "endpoints": {
            "anthropic": "/a/v1/messages",
            "openai": "/o/v1/responses",
            "comm": "/o/v1/chat/completions"
        }
    })
}
