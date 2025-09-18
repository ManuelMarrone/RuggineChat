use crate::performance::update_cpu_time;
use crate::state::AppState;
use crate::types::{LoginRequest, User};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::time::Instant;

//handlers HTTP REST del server

pub async fn root() -> &'static str {
    "Rust WebSocket Chat Server is running!"
}

//get lista utenti connessi
pub async fn get_users(State(users): State<AppState>) -> impl IntoResponse {
    let users_guard = users.connected_users.lock().unwrap();
    let start = Instant::now(); 
    let users_vec: Vec<User> = users_guard.values().map(|cu| cu.user.clone()).collect();
    //aggiorna il tempo di CPU//
    update_cpu_time(users.total_cpu_time.clone(), start);
    (StatusCode::OK, Json(users_vec))
}

//validazione username
pub async fn login_user(
    State(users): State<AppState>,
    Json(login_req): Json<LoginRequest>,
) -> impl IntoResponse {
    // Controlla se l'username è già in uso
    let users_guard = users.connected_users.lock().unwrap();
    let start = Instant::now(); 
    if users_guard.contains_key(&login_req.username) {
        //aggiorna il tempo di CPU
        update_cpu_time(users.total_cpu_time.clone(), start);
        return (
            StatusCode::CONFLICT,
            Json(format!(
                "Username '{}' è già in uso. Scegli un altro nome.",
                login_req.username
            )),
        );
    }

    //aggiorna il tempo di CPU
    update_cpu_time(users.total_cpu_time.clone(), start);
    // Username disponibile
    (
        StatusCode::OK,
        Json(format!("Username '{}' è disponibile", login_req.username)),
    )
}

//aggiorna stato utente
pub async fn update_user_availability(
    State(users): State<AppState>,
    Path(username): Path<String>,
    Json(available): Json<bool>,
) -> impl IntoResponse {
    let mut users_map = users.connected_users.lock().unwrap();
    let start = Instant::now();

    if let Some(connected_user) = users_map.get_mut(&username) {
        connected_user.user.is_available = available;

        if available {
            connected_user.user.chat_id = None;
        }
        //aggiorna il tempo di CPU//
        update_cpu_time(users.total_cpu_time.clone(), start);
        (StatusCode::OK, Json("Disponibilità aggiornata"))
    } else {
        //aggiorna il tempo di CPU//
        update_cpu_time(users.total_cpu_time.clone(), start);
        (StatusCode::NOT_FOUND, Json("Utente non trovato"))
    }
}
