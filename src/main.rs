use axum::{
    routing::{get, post},
    extract::Json,
    Router,
};
use penumbra_nft::{
    mint::mint_nft,
    transfer::transfer_nft,
    view::reveal_nft,
    staking::{stake_nft, unstake_nft},
    airdrop::airdrop_nft,
    ibc::{export_nft_for_ibc, import_nft_from_ibc},
    types::{NFTMetadata, NFT},
    state::NFTState,
};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(NFTState::new()));

    let app = Router::new()
        .route("/mint", post(mint_handler))
        .route("/transfer", post(transfer_handler))
        .route("/view/:id", get(view_handler))
        .route("/stake/:id", post(stake_handler))
        .route("/unstake/:id", post(unstake_handler))
        .route("/airdrop", post(airdrop_handler))
        .route("/ibc/export/:id", get(ibc_export_handler))
        .route("/ibc/import", post(ibc_import_handler))
        .with_state(state);

    println!("Listening on http://127.0.0.1:3000");
    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// POST /mint
async fn mint_handler(
    state: axum::extract::State<Arc<Mutex<NFTState>>>,
    Json(req): Json<MintRequest>,
) -> Json<MintResponse> {
    let mut state = state.lock().unwrap();
    let metadata = NFTMetadata {
        name: req.name,
        description: req.description,
        image_cid: req.image_cid,
        attributes: req.attributes,
        shielded: true,
    };
    let id = mint_nft(&mut state, req.owner, metadata, Some(5));
    Json(MintResponse { id })
}

// POST /transfer
async fn transfer_handler(
    state: axum::extract::State<Arc<Mutex<NFTState>>>,
    Json(req): Json<TransferRequest>,
) -> Json<GenericResponse> {
    let mut state = state.lock().unwrap();
    let result = transfer_nft(&mut state, &req.id, &req.to);
    Json(GenericResponse {
        status: result.map(|_| "ok".into()).unwrap_or_else(|e| e),
    })
}

// GET /view/:id
async fn view_handler(
    state: axum::extract::State<Arc<Mutex<NFTState>>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<Option<NFT>> {
    let state = state.lock().unwrap();
    Json(reveal_nft(&state, &id, None))
}

// POST /stake/:id
async fn stake_handler(
    state: axum::extract::State<Arc<Mutex<NFTState>>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<GenericResponse> {
    let mut state = state.lock().unwrap();
    let result = stake_nft(&mut state, &id);
    Json(GenericResponse {
        status: result.map(|_| "staked".into()).unwrap_or_else(|e| e),
    })
}

// POST /unstake/:id
async fn unstake_handler(
    state: axum::extract::State<Arc<Mutex<NFTState>>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<GenericResponse> {
    let mut state = state.lock().unwrap();
    let result = unstake_nft(&mut state, &id);
    Json(GenericResponse {
        status: result.map(|_| "unstaked".into()).unwrap_or_else(|e| e),
    })
}

// POST /airdrop
async fn airdrop_handler(
    state: axum::extract::State<Arc<Mutex<NFTState>>>,
    Json(req): Json<AirdropRequest>,
) -> Json<GenericResponse> {
    let mut state = state.lock().unwrap();
    let result = airdrop_nft(&mut state, &req.id, req.recipients);
    Json(GenericResponse {
        status: result.map(|_| "airdropped".into()).unwrap_or_else(|e| e),
    })
}

// GET /ibc/export/:id
async fn ibc_export_handler(
    state: axum::extract::State<Arc<Mutex<NFTState>>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<Option<String>> {
    let state = state.lock().unwrap();
    Json(state.get_nft(&id).map(export_nft_for_ibc))
}

// POST /ibc/import
async fn ibc_import_handler(
    state: axum::extract::State<Arc<Mutex<NFTState>>>,
    Json(req): Json<IBCImportRequest>,
) -> Json<GenericResponse> {
    let mut state = state.lock().unwrap();
    let nft = import_nft_from_ibc(&req.serialized);
    let id = nft.id.clone();
    state.nfts.insert(id.clone(), nft);
    Json(GenericResponse {
        status: format!("imported {}", id),
    })
}

// Request/Response structs

#[derive(serde::Deserialize)]
struct MintRequest {
    owner: String,
    name: String,
    description: String,
    image_cid: String,
    attributes: String,
}

#[derive(serde::Serialize)]
struct MintResponse {
    id: String,
}

#[derive(serde::Deserialize)]
struct TransferRequest {
    id: String,
    to: String,
}

#[derive(serde::Deserialize)]
struct AirdropRequest {
    id: String,
    recipients: Vec<String>,
}

#[derive(serde::Deserialize)]
struct IBCImportRequest {
    serialized: String,
}

#[derive(serde::Serialize)]
struct GenericResponse {
    status: String,
}
