use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::Instant;
use tower_http::cors::CorsLayer;

// Importa i moduli
mod chat;
mod cpu_log;
mod invites;
mod notifications;
mod performance;
mod routes;
mod state;
mod tracking;
mod types;
mod user;
mod websocket;

// Importa le strutture e funzioni necessarie dai moduli
use performance::update_cpu_time;
use routes::{get_users, login_user, root, update_user_availability};
use state::AppState;
use websocket::websocket_handler;

#[tokio::main]
async fn main() {
    //inizializzazione tempo totale di uso di CPU
    let total_cpu_time: Arc<Mutex<Duration>> = Arc::new(Mutex::new(Duration::ZERO));

    //avvia il thread di Log
    cpu_log::start_log(Arc::clone(&total_cpu_time));
    //misura l'inizio di uso di cpu
    let start = Instant::now();

    //bypass del blocco del browser per richieste tra origini diverse(client/server)
    let cors = CorsLayer::new()
        .allow_origin(
            "http://localhost:5173"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
        )
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
        ])
        .allow_credentials(true);

    // Inizializza stato condiviso usando il modulo state
    let app_state = AppState::new(total_cpu_time.clone());

    let app = Router::new()
        .route("/", get(root)) //root endpoint
        .route("/ws", get(websocket_handler)) //websocket endpoint
        .route("/api/users", get(get_users)) //users list endpoint
        .route("/api/login", post(login_user)) //login validation endpoint
        .route(
            "/api/users/:username/availability", //user availability update
            post(update_user_availability),
        )
        .with_state(app_state) //state injection per la condivisione tra endpoint
        .layer(cors);

    //crea l'indirizzo del server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server listening on {}", addr);
    println!("WebSocket endpoint: ws://127.0.0.1:3000/ws");

    //aggiorna il tempo di CPU
    update_cpu_time(total_cpu_time.clone(), start);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    // tests placeholder
}
