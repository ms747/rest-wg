use axum::extract::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PeerConf {
    name: String,
    publickey: String,
    privatekey: String,
    address: String,
    enabled: bool,
    allowedip: Vec<String>,
}

pub async fn get_peer(Path((interface, peer_id)): Path<(String, String)>) -> String {
    format!("{} {}\n", interface, peer_id)
}

pub async fn create_peer(Path((interface, peer_id)): Path<(String, String)>) -> String {
    format!("{} {}\n", interface, peer_id)
}

pub async fn update_peer(Path((interface, peer_id)): Path<(String, String)>) -> String {
    format!("{} {}\n", interface, peer_id)
}

pub async fn delete_peer(Path((interface, peer_id)): Path<(String, String)>) -> String {
    format!("{} {}\n", interface, peer_id)
}
