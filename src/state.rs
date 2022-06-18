use std::sync::Arc;

use crate::wghelper::Wg;
use tokio::sync::RwLock;

pub type SharedState = Arc<RwLock<Wg>>;
