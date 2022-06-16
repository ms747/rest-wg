use crate::peer::PeerConf;
use crate::state::SharedState;
use crate::wghelper;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use axum::{extract::Path, Extension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceConf {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub enabled: bool,
    pub ifup: String,
    pub ifdown: String,
    pub publickey: String,
    pub privatekey: String,
    pub peer: Vec<PeerConf>,
}

impl Default for InterfaceConf {
    fn default() -> Self {
        Self {
            name: "".into(),
            address: "".into(),
            port: 51820,
            enabled: true,
            ifup: "iptables -A FORWARD -i %i -j ACCEPT; iptables -t nat -A POSTROUTING -o enp0s3 -j MASQUERADE".into(),
            ifdown: "iptables -D FORWARD -i %i -j ACCEPT; iptables -t nat -D POSTROUTING -o enp0s3 -j MASQUERADE".into(),
            publickey: "".into(),
            privatekey: "".into(),
            peer: vec![]
        }
    }
}

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
pub struct CreateInterfaceConf {
    name: String,
    port: u16,
    address: String,
}

pub async fn start_interface(
    Path(iface_id): Path<usize>,
    Extension(state): Extension<SharedState>,
) -> Result<StatusCode, (StatusCode, String)> {
    let state = state.read().await;
    let iface = state.interfaces.get(iface_id);
    if let Some(iface) = iface {
        return match wghelper::start(iface).await {
            Err(msg) => Err((StatusCode::INTERNAL_SERVER_ERROR, msg)),
            Ok(_) => Ok(StatusCode::OK),
        };
    }

    Err((
        StatusCode::INTERNAL_SERVER_ERROR,
        "Interface not found".into(),
    ))
}

pub async fn stop_interface(
    Path(iface_id): Path<usize>,
    Extension(state): Extension<SharedState>,
) -> Result<StatusCode, StatusCode> {
    let state = state.read().await;
    let iface = state.interfaces.get(iface_id);
    if let Some(iface) = iface {
        wghelper::stop(&iface.name).await;
        return Ok(StatusCode::OK);
    }
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn refresh_interface(
    Path(iface_id): Path<usize>,
    Extension(state): Extension<SharedState>,
) -> Result<StatusCode, StatusCode> {
    let state = state.read().await;
    let iface = state.interfaces.get(iface_id);
    if let Some(iface) = iface {
        wghelper::refresh(iface).await;
        return Ok(StatusCode::OK);
    }
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_interfaces(Extension(state): Extension<SharedState>) -> impl IntoResponse {
    let state = state.read().await;
    let ifaces: Vec<String> = state
        .interfaces
        .iter()
        .map(|iface| iface.name.clone())
        .collect();

    axum::Json(ifaces)
}

pub async fn create_interface(
    Json(create_iface): Json<CreateInterfaceConf>,
    Extension(state): Extension<SharedState>,
) -> Result<StatusCode, StatusCode> {
    let mut state = state.write().await;
    let (privatekey, publickey) = wghelper::get_keys().await;
    let new_interface = InterfaceConf {
        name: create_iface.name,
        port: create_iface.port,
        address: create_iface.address,
        publickey,
        privatekey,
        ..InterfaceConf::default()
    };
    (*state).interfaces.push(new_interface);
    wghelper::write_config(&state).await;
    Ok(StatusCode::OK)
}

pub async fn get_interface(
    Path(id): Path<usize>,
    Extension(state): Extension<SharedState>,
) -> Result<Json<InterfaceConf>, StatusCode> {
    let state = state.read().await;
    let iface = state.interfaces.get(id);
    if let Some(iface) = iface {
        Ok(Json(iface.clone()))
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

pub async fn update_interface(
    Json(updated_json): Json<UpdateInterfaceConf>,
    Path(id): Path<usize>,
    Extension(state): Extension<SharedState>,
) -> Result<StatusCode, StatusCode> {
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
}

pub async fn delete_interface(
    Path(id): Path<usize>,
    Extension(state): Extension<SharedState>,
) -> Result<StatusCode, StatusCode> {
    let mut state = state.write().await;
    if state.interfaces.len() > id {
        state.interfaces.remove(id);
        wghelper::write_config(&state).await;
        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
