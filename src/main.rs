use axum::{routing::get, Extension, Router};
use state::{SharedState, State};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;

mod interface;
mod peer;
mod state;

#[tokio::main]
async fn main() {
    let interface_conf = std::fs::read_to_string("./interfaces.toml").unwrap();

    let interface_conf: State = toml::from_str(&interface_conf).unwrap();
    let shared_state: SharedState = Arc::new(RwLock::new(interface_conf));

    let app = Router::new()
        .route(
            "/interface",
            get(interface::get_interfaces).post(interface::create_interface),
        )
        .route(
            "/interface/:name",
            get(interface::get_interface)
                .patch(interface::update_interface)
                .delete(interface::delete_interface),
        )
        .route(
            "/interface/:name/peer/:id",
            get(peer::get_peer)
                .post(peer::create_peer)
                .patch(peer::update_peer)
                .delete(peer::delete_peer),
        )
        .layer(Extension(shared_state));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
