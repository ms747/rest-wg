use std::sync::Arc;

use crate::interface::InterfaceConf;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

pub type SharedState = Arc<RwLock<State>>;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct State {
    #[serde(rename = "interface", skip_serializing_if = "Vec::is_empty", default)]
    pub interfaces: Vec<InterfaceConf>,
}
