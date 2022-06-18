use axum::{routing::get, Extension, Router};
use state::SharedState;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use wghelper::Wg;

mod interface;
mod peer;
mod peerconfig;
mod state;
mod wghelper;

#[tokio::main]
async fn main() {
    let interface_conf: Wg = Wg::read_state();
    let shared_state: SharedState = Arc::new(RwLock::new(interface_conf));

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route(
            "/interface",
            get(interface::get_servers).post(interface::create_server),
        )
        .route(
            "/interface/:iface",
            get(interface::get_server)
                //.patch(interface::update_server)
                .delete(interface::delete_server),
        )
        .route("/interface/:iface/start", get(interface::start_server))
        .route("/interface/:iface/stop", get(interface::stop_server))
        .route("/interface/:iface/refresh", get(interface::refresh_server))
        .route(
            "/interface/:iface/peer",
            get(peer::get_peers).post(peer::create_peer),
        )
        .route(
            "/interface/:iface/peer/:peer",
            get(peer::get_peer)
                //.patch(peer::update_peer)
                .delete(peer::delete_peer),
        )
        .route(
            "/interface/:iface/peer/:peer/config",
            get(peerconfig::get_config),
        )
        .layer(Extension(shared_state))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
