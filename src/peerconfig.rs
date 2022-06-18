use axum::{
    extract::Path,
    http::{header, HeaderMap, StatusCode},
    Extension,
};

use crate::state::SharedState;

pub async fn get_config(
    Path((server_id, peer_id)): Path<(usize, usize)>,
    Extension(state): Extension<SharedState>,
) -> Result<(HeaderMap, String), StatusCode> {
    let state = state.read().await;

    if let Some(server) = state.servers.get(server_id) {
        if let Some(peer) = server.peers.get(peer_id) {
            let peer_config = state.peer_config(server_id, peer_id);
            let mut headers = HeaderMap::new();
            headers.insert(
                header::ACCESS_CONTROL_EXPOSE_HEADERS,
                "Content-Disposition".parse().unwrap(),
            );
            headers.insert(
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}.conf\"", peer.name)
                    .parse()
                    .unwrap(),
            );
            headers.insert(header::CONTENT_TYPE, "text/plain".parse().unwrap());
            return Ok((headers, peer_config));
        }
    }
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}
