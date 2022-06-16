use crate::{state::SharedState, wghelper};
use axum::{extract::Path, http::StatusCode, Extension, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConf {
    pub name: String,
    pub publickey: String,
    pub privatekey: String,
    pub address: String,
    pub port: u16,
    pub enabled: bool,
    pub allowedip: Vec<String>,
}

impl Default for PeerConf {
    fn default() -> Self {
        Self {
            name: "".into(),
            publickey: "".into(),
            privatekey: "".into(),
            address: "".into(),
            port: 0,
            enabled: true,
            allowedip: vec![],
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreatePeerConf {
    name: String,
    address: String,
    allowedip: Vec<String>,
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
    Path(iface_id): Path<usize>,
    Extension(state): Extension<SharedState>,
) -> Result<Json<Vec<PeerConf>>, StatusCode> {
    let state = state.read().await;
    if state.interfaces.len() > iface_id {
        return Ok(Json(state.interfaces[iface_id].peer.clone()));
    }
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_peer(
    Path((iface_id, peer_id)): Path<(usize, usize)>,
    Extension(state): Extension<SharedState>,
) -> Result<Json<PeerConf>, StatusCode> {
    let state = state.read().await;
    if state.interfaces.len() > iface_id && state.interfaces[iface_id].peer.len() > peer_id {
        return Ok(axum::Json(state.interfaces[iface_id].peer[peer_id].clone()));
    }
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn create_peer(
    Json(create_peer): Json<CreatePeerConf>,
    Path(iface_id): Path<usize>,
    Extension(state): Extension<SharedState>,
) -> Result<StatusCode, StatusCode> {
    let mut state = state.write().await;
    if state.interfaces.len() > iface_id {
        let (privatekey, publickey) = wghelper::get_keys().await;
        let new_peer = PeerConf {
            address: create_peer.address,
            port: 51820,
            name: create_peer.name,
            privatekey,
            publickey,
            allowedip: create_peer.allowedip,
            ..PeerConf::default()
        };
        state.interfaces[iface_id].peer.push(new_peer);
        wghelper::write_config(&state).await;
        return Ok(StatusCode::OK);
    }

    Ok(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn update_peer(
    Json(updated_peer): Json<UpdatePeerConf>,
    Extension(state): Extension<SharedState>,
    Path((iface_id, peer_id)): Path<(usize, usize)>,
) -> Result<StatusCode, StatusCode> {
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
}

pub async fn delete_peer(
    Path((iface_id, peer_id)): Path<(usize, usize)>,
    Extension(state): Extension<SharedState>,
) -> Result<StatusCode, StatusCode> {
    let mut state = state.write().await;
    if state.interfaces.len() > iface_id && state.interfaces[iface_id].peer.len() > peer_id {
        state.interfaces[iface_id].peer.remove(peer_id);
        wghelper::write_config(&state).await;
        return Ok(StatusCode::OK);
    }
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}
