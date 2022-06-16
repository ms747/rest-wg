use axum::{routing::get, Extension, Router};
use state::{SharedState, State};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

mod interface;
mod peer;
mod peerconfig;
mod state;
mod wghelper;

#[tokio::main]
async fn main() {
    let interface_conf: State = wghelper::read_config();
    let shared_state: SharedState = Arc::new(RwLock::new(interface_conf));

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route(
            "/interface",
            get(interface::get_interfaces).post(interface::create_interface),
        )
        .route(
            "/interface/:iface",
            get(interface::get_interface)
                .patch(interface::update_interface)
                .delete(interface::delete_interface),
        )
        .route("/interface/:iface/start", get(interface::start_interface))
        .route("/interface/:iface/stop", get(interface::stop_interface))
        .route(
            "/interface/:iface/refresh",
            get(interface::refresh_interface),
        )
        .route(
            "/interface/:iface/peer",
            get(peer::get_peers).post(peer::create_peer),
        )
        .route(
            "/interface/:iface/peer/:peer",
            get(peer::get_peer)
                .patch(peer::update_peer)
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
