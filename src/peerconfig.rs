use axum::{
    extract::Path,
    http::{header, HeaderMap, StatusCode},
    Extension,
};

use crate::{state::SharedState, wghelper};

pub async fn get_config(
    Path((iface_id, peer_id)): Path<(usize, usize)>,
    Extension(state): Extension<SharedState>,
) -> Result<(HeaderMap, String), StatusCode> {
    let state = state.read().await;
    if state.interfaces.len() > iface_id && state.interfaces[iface_id].peer.len() > peer_id {
        let peer_config = wghelper::generate_peer_config(&state.interfaces[iface_id], peer_id);
        let mut headers = HeaderMap::new();
        // "Access-Control-Expose-Headers", "Content-Disposition"
        headers.insert(header::ACCESS_CONTROL_EXPOSE_HEADERS, "Content-Disposition".parse().unwrap());
        headers.insert(
            header::CONTENT_DISPOSITION,
            format!(
                "attachment; filename=\"{}.conf\"",
                state.interfaces[iface_id].peer[peer_id].name
            )
            .parse()
            .unwrap(),
        );
        headers.insert(header::CONTENT_TYPE, "text/plain".parse().unwrap());
        return Ok((headers, peer_config));
    }
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}
