use std::{str::FromStr, sync::Arc};

use anchor_client::anchor_lang::InstructionData;
use askama::Template;
use askama_axum::{IntoResponse, Response};
use axum::{
    extract::Query,
    http::StatusCode,
    routing::{get, post},
    Extension, Form, Json, Router,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::engine::Engine as _;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use solana_client::{rpc_client::RpcClient, rpc_config::RpcSendTransactionConfig};
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel}, instruction::Instruction, message::Message, pubkey::Pubkey, signature::Signature, transaction::Transaction
};

use t_vault::instruction;

struct Config {
    rpc_url: String,
}

impl Config {
    pub fn new() -> Self {
        Config {
            rpc_url: std::env::var("SOLANA_RPC_URL").expect("SOLANA_RPC_URL not set in env."),
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let config = Config::new();
    let rpc_client = Arc::new(RpcClient::new_with_commitment(config.rpc_url, CommitmentConfig {commitment:  CommitmentLevel::Processed}));

    let app = Router::new()
        // No Auth
        .route("/styles.css", get(styles))
        .route("/script.js", get(script))
        .route("/", get(index))
        .route("/tx-modal", get(handle_get_tx_modal))
        .route("/initialize", post(handle_initialize))
        .route("/tx-status", get(handle_get_tx_status))
        .route("/tx-submit", post(handle_submit_tx))
        .route("/tx-status-data", get(handle_get_tx_status_data))
        .layer(Extension(rpc_client));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listener bound to port 3000");
    println!("Serving listener..");
    axum::serve(listener, app).await.unwrap();
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

async fn index() -> impl IntoResponse {
    return IndexTemplate;
}

async fn styles() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/css")
        .body(include_str!("../templates/styles.css").to_owned())
        .unwrap()
}

async fn script() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/javascript")
        .body(include_str!("../templates/script.js").to_owned())
        .unwrap()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Base64EncodedTransaction {
    encoded_tx: String,
}

#[derive(Deserialize)]
struct InitializePayload {
    public_key: String,
}

async fn handle_initialize(
    Extension(rpc_client): Extension<Arc<RpcClient>>,
    Json(payload): Json<InitializePayload>,
) -> impl IntoResponse {
    let to_pubkey: Pubkey = Pubkey::from_str(&payload.public_key).unwrap();
    println!("Found pubkey: {}", to_pubkey);

    let system_program_id = Pubkey::from_str(&t_vault::id().to_string()).unwrap();

    let ix_data = instruction::Initialize {};

    let ix = Instruction::new_with_bytes(system_program_id, &ix_data.data(), Vec::new());

    println!("Created ix...");

    //let signers = &[&keypair];

    let blockhash = rpc_client.get_latest_blockhash().unwrap();
    println!("Got latest blockhash");

    let message = Message::new_with_blockhash(&[ix], Some(&to_pubkey), &blockhash);

    let tx = Transaction::new_unsigned(message);
    let serialized_tx = bincode::serialize(&tx).unwrap();

    let encoded_tx = BASE64.encode(serialized_tx.clone());
    //let config = RpcSendTransactionConfig::default();

    //println!("Sending transaction...");
    //let sx = rpc.send_transaction_with_config(&tx, config).unwrap();
    //println!("Transaction send: {}", sx);

    (
        StatusCode::OK,
        Json(Base64EncodedTransaction { encoded_tx }),
    )
}

#[derive(Template)]
#[template(path = "tx-modal.html")]
struct TxModalTemplate {
    transaction_name: String,
    button_id: String,
    data_endpoint: String,
    encoded_tx: String,
}

#[derive(Deserialize)]
struct TxModalQueryParams {
    tx_type: String,
    pubkey: String,
}

// Building the tx
async fn handle_get_tx_modal(
    Extension(rpc_client): Extension<Arc<RpcClient>>,
    Query(query_params): Query<TxModalQueryParams>,
) -> impl IntoResponse {
    let tx_type = query_params.tx_type;
    let pubkey = Pubkey::from_str(&query_params.pubkey);

    if let Ok(pubkey) = pubkey {
        if tx_type == "initialize" {
            let system_program_id = Pubkey::from_str(&t_vault::id().to_string()).unwrap();

            let ix_data = instruction::Initialize {};

            let ix = Instruction::new_with_bytes(system_program_id, &ix_data.data(), Vec::new());

            println!("Created ix...");

            //let signers = &[&keypair];

            let blockhash = rpc_client.get_latest_blockhash().unwrap();
            println!("Got latest blockhash");

            let message = Message::new_with_blockhash(&[ix], Some(&pubkey), &blockhash);

            let tx = Transaction::new_unsigned(message);
            let serialized_tx = bincode::serialize(&tx).unwrap();

            let encoded_tx = BASE64.encode(serialized_tx);
            return (
                StatusCode::OK,
                TxModalTemplate {
                    transaction_name: "Initialize".to_string(),
                    button_id: "initialize-button".to_string(),
                    data_endpoint: "/initialize".to_string(),
                    encoded_tx,
                }
                .to_string(),
            );
        }

        return (StatusCode::BAD_REQUEST, "Invalid tx_type".to_string());
    } else {
        (StatusCode::BAD_REQUEST, "Invalid pubkey".to_string())
    }
}

#[derive(Template)]
#[template(path = "tx-status.html")]
struct TxStatusTemplate {
    tx_signature: String,
}

#[derive(Deserialize)]
struct TxStatusQueryParams {
    tx_signature: String,
}

async fn handle_get_tx_status(
    Query(query_params): Query<TxStatusQueryParams>,
) -> impl IntoResponse {
    let tx_signature = query_params.tx_signature;

    (
        StatusCode::OK,
        TxStatusTemplate { tx_signature }.to_string(),
    )
}

async fn handle_get_tx_status_data(
    Extension(rpc_client): Extension<Arc<RpcClient>>,
    Query(query_params): Query<TxStatusQueryParams>,
) -> impl IntoResponse {
    let tx_signature = query_params.tx_signature;

    let sig = Signature::from_str(&tx_signature).unwrap();

    let transaction_status = rpc_client.get_signature_statuses(&[sig]);

    if let Ok(tx_status_response) = &transaction_status {
        let statuses = &tx_status_response.value;

        if let Some(status) = statuses[0].clone() {
            println!("{:?}", status);
            if let Some(confirmation) = status.confirmation_status.clone() {
                return (StatusCode::OK, format!("{:?}", confirmation))
            }
            return (StatusCode::OK, format!("{:?}", status))
        }
    }

    (StatusCode::OK, "Signature status not found".to_string())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubmitTxPayload {
    encoded_serialized_tx: String,
}

async fn handle_submit_tx(
    Extension(rpc_client): Extension<Arc<RpcClient>>,
    Form(tx_data): Form<SubmitTxPayload>,
) -> impl IntoResponse {
    let serialized_tx = BASE64.decode(tx_data.encoded_serialized_tx).unwrap();
    let tx: Transaction = bincode::deserialize(&serialized_tx).unwrap();

    let send_config = RpcSendTransactionConfig {
        skip_preflight: false,
        preflight_commitment: Some(CommitmentLevel::Processed),
        encoding: None,
        max_retries: None,
        min_context_slot: None,

    };

    let signature_result = rpc_client.send_transaction_with_config(&tx, send_config);

    if let Ok(signature) = signature_result {
        return (
            StatusCode::OK,
            TxStatusTemplate {
                tx_signature: signature.to_string(),
            }
            .to_string(),
        );
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Failed to submit tx".to_string(),
    )
}
