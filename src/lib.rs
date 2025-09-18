// Libreria del server per riuso in test di integrazione

// Reimportiamo i moduli in modo che siano disponibili anche come crate libreria
pub mod chat;
pub mod cpu_log;
pub mod invites;
pub mod notifications;
pub mod performance;
pub mod routes;
pub mod state;
pub mod tracking;
pub mod types;
pub mod user;
pub mod websocket;

pub use state::{AppState, ConnectedUser};
pub use types::*;

use axum::{routing::{get, post}, Router};
use tower_http::cors::CorsLayer;
use routes::{get_users, login_user, root, update_user_availability};
use websocket::websocket_handler;

// Costruisce il Router Axum come fa il main
pub fn create_app(state: AppState, cors: CorsLayer) -> Router {
	Router::new()
		.route("/", get(root))
		.route("/ws", get(websocket_handler))
		.route("/api/users", get(get_users))
		.route("/api/login", post(login_user))
		.route(
			"/api/users/:username/availability",
			post(update_user_availability),
		)
		.with_state(state)
		.layer(cors)
}
