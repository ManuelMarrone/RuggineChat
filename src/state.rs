use crate::types::{ChatUsersCount, User};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

// Stato globale per memorizzare utenti connessi e loro WebSocket
#[derive(Debug)]
pub struct ConnectedUser {
    pub user: User,
    pub sender: tokio::sync::mpsc::UnboundedSender<String>, //canale websocket dell'utente
    pub session_id: String,                                 // Identificatore univoco della sessione
}

//struttura di condivisione dello stato tra tutti i thread, connessioni websocket e operazioni http
#[derive(Clone)]
pub struct AppState {
    pub connected_users: Arc<Mutex<HashMap<String, ConnectedUser>>>,
    pub total_cpu_time: Arc<Mutex<Duration>>,
    pub chat_tracking: Arc<Mutex<HashMap<String, ChatUsersCount>>>, // traccia utenti attivi in chat
    pub private_chats_with_both_users: Arc<Mutex<HashSet<String>>>, // Set di chat_id private dove entrambi gli utenti sono stati presenti almeno una volta. Utilizzato per evitare notifiche di abbandono premature in chat incomplete.
}

impl AppState {
    pub fn new(total_cpu_time: Arc<Mutex<Duration>>) -> Self {
        let connected_users = Arc::new(Mutex::new(HashMap::new()));

        AppState {
            connected_users: connected_users.clone(),
            total_cpu_time: total_cpu_time.clone(),
            chat_tracking: Arc::new(Mutex::new(HashMap::new())),
            private_chats_with_both_users: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}
