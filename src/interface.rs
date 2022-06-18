use crate::state::SharedState;
use crate::wghelper::{Server, Wg};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use axum::{extract::Path, Extension};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UpdateInterfaceConf {
    name: Option<String>,
    address: Option<String>,
    port: Option<u16>,
    enabled: Option<bool>,
    ifup: Option<String>,
    ifdown: Option<String>,
    publickey: Option<String>,
    privatekey: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateServer {
    name: String,
    port: u16,
    cidr: String,
}

pub async fn start_server(
    Path(server_id): Path<usize>,
    Extension(state): Extension<SharedState>,
) -> Result<StatusCode, StatusCode> {
    let state = state.read().await;
    if let Some(_) = state.servers.get(server_id) {
        state.start(server_id).await;
        return Ok(StatusCode::OK);
    }
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn stop_server(
    Path(server_id): Path<usize>,
    Extension(state): Extension<SharedState>,
) -> Result<StatusCode, StatusCode> {
    let state = state.read().await;
    if let Some(_) = state.servers.get(server_id) {
        state.stop(server_id).await;
        return Ok(StatusCode::OK);
    }
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn refresh_server(
    Path(server_id): Path<usize>,
    Extension(state): Extension<SharedState>,
) -> Result<StatusCode, StatusCode> {
    let state = state.read().await;
    if let Some(_) = state.servers.get(server_id) {
        state.hot_reload(server_id).await;
        return Ok(StatusCode::OK);
    }
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_servers(Extension(state): Extension<SharedState>) -> impl IntoResponse {
    let state = state.read().await;
    let ifaces: Vec<String> = state
        .servers
        .iter()
        .map(|server| server.name.clone())
        .collect();
    Json(ifaces)
}

pub async fn create_server(
    Json(create_server): Json<CreateServer>,
    Extension(state): Extension<SharedState>,
) -> Result<StatusCode, StatusCode> {
    let mut state = state.write().await;
    state
        .new(&create_server.name, &create_server.cidr, create_server.port)
        .await;

    Wg::dump_state(&state).await;
    Ok(StatusCode::OK)
}

pub async fn get_server(
    Path(server_id): Path<usize>,
    Extension(state): Extension<SharedState>,
) -> Result<Json<Server>, StatusCode> {
    let state = state.read().await;
    if let Some(server) = state.servers.get(server_id) {
        Ok(Json(server.clone()))
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

pub async fn update_server(
    Json(updated_json): Json<UpdateInterfaceConf>,
    Path(server_id): Path<usize>,
    Extension(state): Extension<SharedState>,
) -> Result<StatusCode, StatusCode> {
    /*
    let mut state = state.write().await;
    dbg!(&updated_json);
    if state.interfaces.len() > id {
        // name
        if let Some(name) = updated_json.name {
            state.interfaces[id].name = name;
        }
        // address
        if let Some(address) = updated_json.address {
            state.interfaces[id].address = address;
        }
        // port
        if let Some(port) = updated_json.port {
            state.interfaces[id].port = port;
        }
        // enabled
        if let Some(enabled) = updated_json.enabled {
            state.interfaces[id].enabled = enabled;
        }
        // ifup
        if let Some(ifup) = updated_json.ifup {
            state.interfaces[id].ifup = ifup;
        }
        // ifdown
        if let Some(ifdown) = updated_json.ifdown {
            state.interfaces[id].ifdown = ifdown;
        }
        // publickey
        if let Some(publickey) = updated_json.publickey {
            state.interfaces[id].publickey = publickey;
        }
        // privatekey
        if let Some(privatekey) = updated_json.privatekey {
            state.interfaces[id].privatekey = privatekey;
        }
        wghelper::write_config(&state).await;
        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
    */
    todo!()
}

pub async fn delete_server(
    Path(server_id): Path<usize>,
    Extension(state): Extension<SharedState>,
) -> Result<StatusCode, StatusCode> {
    let mut state = state.write().await;
    if let Some(_) = state.servers.get(server_id) {
        state.servers.remove(server_id);
        Wg::dump_state(&state).await;
        return Ok(StatusCode::OK);
    }
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}
