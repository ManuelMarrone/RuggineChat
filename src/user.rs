use crate::performance::update_cpu_time;
use crate::state::AppState;
use crate::types::{MessageType, User, WebSocketMessage};
use std::time::Instant;

pub async fn broadcast_user_joined(state: &AppState, user: &User) {
    let start = Instant::now();
    let message = WebSocketMessage {
        message_type: MessageType::UserJoined,
        data: serde_json::to_string(user).unwrap(),
    };
    //aggiorna il tempo di CPU//
    update_cpu_time(state.total_cpu_time.clone(), start);
    broadcast_to_all(state, &message).await;
}

pub async fn broadcast_user_status_changed(state: &AppState, updated_user: &User) {
    let start = Instant::now();
    let message = WebSocketMessage {
        message_type: MessageType::UserStatusChanged,
        data: serde_json::to_string(&updated_user).unwrap(),
    };
    //aggiorna il tempo di CPU//
    update_cpu_time(state.total_cpu_time.clone(), start);
    broadcast_to_all(state, &message).await;
}

//Funzione per inviare la lista aggiornata a tutti gli utenti
pub async fn send_users_list_to_all(state: &AppState) {
    let mut start;
    let users_list = {
        let users = state.connected_users.lock().unwrap();
        start = Instant::now();
        users
            .values()
            .map(|connected_user| connected_user.user.clone())
            .collect::<Vec<_>>()
    };

    let users_msg = WebSocketMessage {
        message_type: MessageType::UsersList,
        data: serde_json::to_string(&users_list).unwrap_or_default(),
    };

    if let Ok(msg_json) = serde_json::to_string(&users_msg) {
        //aggiorna il tempo di CPU//
        update_cpu_time(state.total_cpu_time.clone(), start);
        let users = state.connected_users.lock().unwrap();
        start = Instant::now();
        for connected_user in users.values() {
            let _ = connected_user.sender.send(msg_json.clone());
        }
    }

    //aggiorna il tempo di CPU//
    update_cpu_time(state.total_cpu_time.clone(), start);
}

pub async fn send_users_list(tx: &tokio::sync::mpsc::UnboundedSender<String>, state: &AppState) {
    let users: Vec<User> = {
        let users_guard = state.connected_users.lock().unwrap();
        users_guard.values().map(|cu| cu.user.clone()).collect()
    };
    let start = Instant::now();

    let message = WebSocketMessage {
        message_type: MessageType::UsersList,
        data: serde_json::to_string(&users).unwrap(),
    };

    let _ = tx.send(serde_json::to_string(&message).unwrap());
    //aggiorna il tempo di CPU//
    update_cpu_time(state.total_cpu_time.clone(), start);
}

pub async fn broadcast_to_all(state: &AppState, message: &WebSocketMessage) {
    let mut start = Instant::now();
    let message_json = serde_json::to_string(message).unwrap();

    //aggiorna il tempo di CPU//
    update_cpu_time(state.total_cpu_time.clone(), start);

    let users = state.connected_users.lock().unwrap();
    start = Instant::now();
    for connected_user in users.values() {
        let _ = connected_user.sender.send(message_json.clone());
    }
    //aggiorna il tempo di CPU//
    update_cpu_time(state.total_cpu_time.clone(), start);
}
