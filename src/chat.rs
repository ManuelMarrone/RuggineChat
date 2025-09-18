use crate::performance::update_cpu_time;
use crate::state::AppState;
use crate::tracking::{check_and_notify_alone_in_chat, remove_user_from_chat_tracking};
use crate::types::{ChatMessage, MessageType, WebSocketMessage};
use crate::user::broadcast_to_all;
use std::time::Instant;
use uuid;

pub async fn broadcast_chat_message(
    state: &AppState,
    sender_username: &str,
    chat_msg: &ChatMessage,
) {
    //misura l'inizio di uso di cpu//
    let mut start = Instant::now();

    //serializza messaggio
    let message = WebSocketMessage {
        message_type: MessageType::ChatMessage,
        data: serde_json::to_string(chat_msg).unwrap(),
    };
    let message_json = serde_json::to_string(&message).unwrap();

    //determina a quale chat appartiene il messaggio
    let target_chat_id = if let Some(explicit_chat_id) = chat_msg.chat_id.clone() {
        //aggiorna il tempo di CPU//
        update_cpu_time(state.total_cpu_time.clone(), start);
        Some(explicit_chat_id)
    } else {
        //aggiorna il tempo di CPU//
        update_cpu_time(state.total_cpu_time.clone(), start);
        // Fallback: prendi chat_id del sender
        let users = state.connected_users.lock().unwrap();
        start = Instant::now();
        let sender_chat_id = users
            .get(sender_username)
            .and_then(|u| u.user.chat_id.as_ref())
            .map(|id| id.clone());

        //aggiorna il tempo di CPU//
        update_cpu_time(state.total_cpu_time.clone(), start);
        sender_chat_id
    };

    //invio messaggio
    if let Some(chat_id) = target_chat_id {
        // FILTRA: Invia solo agli utenti con stesso chatId
        let users = state.connected_users.lock().unwrap();
        start = Instant::now();

        for (_, connected_user) in users.iter() {
            if let Some(user_chat_id) = &connected_user.user.chat_id {
                if user_chat_id == &chat_id {
                    let _ = connected_user.sender.send(message_json.clone());
                }
            }
        }

        //aggiorna il tempo di CPU//
        update_cpu_time(state.total_cpu_time.clone(), start);
    } else {
        // FALLBACK: Per messaggi di sistema o utenti senza chatId
        let users = state.connected_users.lock().unwrap();
        start = Instant::now();

        match &chat_msg.chat_type {
            crate::types::ChatType::Private { target } => {
                let recipients = vec![sender_username, target.as_str()];
                for recipient in recipients {
                    if let Some(connected_user) = users.get(recipient) {
                        let _ = connected_user.sender.send(message_json.clone());
                    }
                }
            }
            crate::types::ChatType::Group { members } => {
                for member in members {
                    if let Some(connected_user) = users.get(member) {
                        let _ = connected_user.sender.send(message_json.clone());
                    }
                }
            }
            _ => {}
        }
        //aggiorna il tempo di CPU//
        update_cpu_time(state.total_cpu_time.clone(), start);
    }
}

//disconnesione utente dal sistema
pub async fn broadcast_user_left(state: &AppState, username: &str, chat_id: Option<String>) {
    //misura l'inizio di uso di cpu//
    let start = Instant::now();

    let message = WebSocketMessage {
        message_type: MessageType::UserLeft,
        data: username.to_string(),
    };

    //aggiorna il tempo di CPU//
    update_cpu_time(state.total_cpu_time.clone(), start);

    broadcast_to_all(state, &message).await;

    //misura l'inizio di uso di cpu//
    let start = Instant::now();

    if let Some(ref chat_id_str) = chat_id {
        let system_chat_message = ChatMessage {
            id: uuid::Uuid::new_v4(),
            chat_id: chat_id.clone(),
            username: "Sistema".to_string(),
            content: format!("{} ha abbandonato la chat", username),
            timestamp: chrono::Utc::now(),
            chat_type: crate::types::ChatType::System,
        };

        //aggiorna il tempo di CPU//
        update_cpu_time(state.total_cpu_time.clone(), start);
        broadcast_chat_message(state, "Sistema", &system_chat_message).await;

        // Rimuovi utente dal tracking quando si disconnette
        remove_user_from_chat_tracking(state, chat_id_str, username).await;

        // Controlla se qualcuno è rimasto solo nella chat
        check_and_notify_alone_in_chat(state, chat_id_str).await;
    }
}
