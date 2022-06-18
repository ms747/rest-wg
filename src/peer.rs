use crate::{
    state::SharedState,
    wghelper::{Peer, Wg},
};
use axum::{extract::Path, http::StatusCode, Extension, Json};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreatePeer {
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePeerConf {
    name: Option<String>,
    publickey: Option<String>,
    privatekey: Option<String>,
    address: Option<String>,
    enabled: Option<bool>,
    allowedip: Option<Vec<String>>,
}

pub async fn get_peers(
    Path(server_id): Path<usize>,
    Extension(state): Extension<SharedState>,
) -> Result<Json<Vec<Peer>>, StatusCode> {
    let state = state.read().await;
    if let Some(server) = state.servers.get(server_id) {
        return Ok(Json(server.peers.clone()));
    }
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_peer(
    Path((server_id, peer_id)): Path<(usize, usize)>,
    Extension(state): Extension<SharedState>,
) -> Result<Json<Peer>, StatusCode> {
    let state = state.read().await;
    if let Some(server) = state.servers.get(server_id) {
        if let Some(peer) = server.peers.get(peer_id) {
            return Ok(axum::Json(peer.clone()));
        }
    }
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn create_peer(
    Json(create_peer): Json<CreatePeer>,
    Path(server_id): Path<usize>,
    Extension(state): Extension<SharedState>,
) -> Result<StatusCode, StatusCode> {
    let mut state = state.write().await;
    if let Some(_) = state.servers.get(server_id) {
        state.create_peer(&create_peer.name, server_id).await;
        Wg::dump_state(&state).await;
        return Ok(StatusCode::OK);
    }

    Ok(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn update_peer(
    Json(updated_peer): Json<UpdatePeerConf>,
    Extension(state): Extension<SharedState>,
    Path((server_id, peer_id)): Path<(usize, usize)>,
) -> Result<StatusCode, StatusCode> {
    todo!()
    /*
    let mut state = state.write().await;
    if state.interfaces.len() > iface_id && state.interfaces[iface_id].peer.len() > peer_id {
        // name
        if let Some(name) = updated_peer.name {
            state.interfaces[iface_id].peer[peer_id].name = name;
        }
        // publickey
        if let Some(publickey) = updated_peer.publickey {
            state.interfaces[iface_id].peer[peer_id].publickey = publickey;
        }
        // privatekey
        if let Some(privatekey) = updated_peer.privatekey {
            state.interfaces[iface_id].peer[peer_id].privatekey = privatekey;
        }
        // address
        if let Some(address) = updated_peer.address {
            state.interfaces[iface_id].peer[peer_id].address = address;
        }
        // enabled
        if let Some(enabled) = updated_peer.enabled {
            state.interfaces[iface_id].peer[peer_id].enabled = enabled;
        }
        // allowedip
        if let Some(allowedip) = updated_peer.allowedip {
            state.interfaces[iface_id].peer[peer_id].allowedip = allowedip;
        }

        wghelper::write_config(&state).await;
        return Ok(StatusCode::OK);
    }
    Err(StatusCode::INTERNAL_SERVER_ERROR)
    */
}

pub async fn delete_peer(
    Path((server_id, peer_id)): Path<(usize, usize)>,
    Extension(state): Extension<SharedState>,
) -> Result<StatusCode, StatusCode> {
    let mut state = state.write().await;
    if let Some(server) = state.servers.get_mut(server_id) {
        if let Some(_) = server.peers.get(peer_id) {
            server.peers.remove(peer_id);
            Wg::dump_state(&state).await;
            return Ok(StatusCode::OK);
        }
    }
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}
