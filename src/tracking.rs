use crate::performance::update_cpu_time;
use crate::state::AppState;
use crate::types::{
    AloneInChatNotification, ChatAbandonedNotification, ChatUsersCount, MessageType,
    WebSocketMessage,
};
use std::time::Instant;

/// Inizializza o aggiorna il tracking per una chat
pub fn init_chat_tracking(state: &AppState, chat_id: &str, invited_users: Vec<String>) {
    let mut tracking = state.chat_tracking.lock().unwrap();
    let start = Instant::now();
    tracking.insert(
        chat_id.to_string(),
        ChatUsersCount {
            chat_id: chat_id.to_string(),
            invited_users: invited_users.clone(),
            users_in_chat: Vec::new(),
            invited_count: invited_users.len(),
            in_chat_count: 0,
        },
    );
    update_cpu_time(state.total_cpu_time.clone(), start);
}

// Aggiunge un utente alla chat e aggiorna il conteggio
pub async fn add_user_to_chat_tracking(state: &AppState, chat_id: &str, username: &str) {
    let start;
    let (should_broadcast, is_private_chat_full) = {
        let mut tracking = state.chat_tracking.lock().unwrap();   
        start = Instant::now(); 
        if let Some(chat_count) = tracking.get_mut(chat_id) {
            if !chat_count.users_in_chat.contains(&username.to_string()) {
                chat_count.users_in_chat.push(username.to_string());
                chat_count.in_chat_count = chat_count.users_in_chat.len();

                //CONTROLLO SPECIALE: Se è una chat privata e abbiamo raggiunto 2 utenti
                let is_private_full =
                    chat_count.invited_count == 2 && chat_count.in_chat_count == 2;
                (true, is_private_full)
            } else {
                (false, false)
            }
        } else {
            (false, false)
        }
    };

    update_cpu_time(state.total_cpu_time.clone(), start);

    //Se è una chat privata con entrambi gli utenti, memorizzala
    if is_private_chat_full {
        let mut private_chats = state.private_chats_with_both_users.lock().unwrap();
        private_chats.insert(chat_id.to_string());
    }

    if should_broadcast {
        broadcast_chat_users_count(state, chat_id).await;
    }
}

// Rimuove un utente dalla chat e aggiorna il conteggio
pub async fn remove_user_from_chat_tracking(state: &AppState, chat_id: &str, username: &str) {
    let mut start; 
    let (should_broadcast, should_send_abandonment_notice, remaining_user) = {
        let mut tracking = state.chat_tracking.lock().unwrap();
        start = Instant::now();
        if let Some(chat_count) = tracking.get_mut(chat_id) {
            if let Some(pos) = chat_count.users_in_chat.iter().position(|u| u == username) {
                chat_count.users_in_chat.remove(pos);
                chat_count.in_chat_count = chat_count.users_in_chat.len();
                
                update_cpu_time(state.total_cpu_time.clone(), start);
                //CONTROLLO SPECIALE: Chat privata con abbandono definitivo
                let is_private_chat_abandonment = {
                    let private_chats = state.private_chats_with_both_users.lock().unwrap();
                    start = Instant::now();
                    let is_private = chat_count.invited_count == 2;
                    let has_one_left = chat_count.in_chat_count == 1;
                    let had_both_users = private_chats.contains(chat_id);

                    is_private && has_one_left && had_both_users
                };

                let remaining_user =
                    if is_private_chat_abandonment && !chat_count.users_in_chat.is_empty() {
                        Some(chat_count.users_in_chat[0].clone())
                    } else {
                        None
                    };
                update_cpu_time(state.total_cpu_time.clone(), start);    
                (true, is_private_chat_abandonment, remaining_user)
            } else {
                update_cpu_time(state.total_cpu_time.clone(), start);
                (false, false, None)
            }
        } else {
            update_cpu_time(state.total_cpu_time.clone(), start);
            (false, false, None)
        }
    };

    if should_broadcast {
        broadcast_chat_users_count(state, chat_id).await;
    }

    start = Instant::now();
    // NUOVO: Invia notifica di abbandono definitivo per chat private
    if should_send_abandonment_notice {
        if let Some(remaining_user_name) = remaining_user {
            update_cpu_time(state.total_cpu_time.clone(), start);
            send_chat_abandoned_notification(state, chat_id, username, &remaining_user_name).await;
        }
    }
}

/// Rimuove un utente dagli "invited" quando rifiuta l'invito (o non entrerà più)
pub async fn remove_user_from_invited(state: &AppState, chat_id: &str, username: &str) {
    let mut tracking = state.chat_tracking.lock().unwrap();
    let start = Instant::now();
    if let Some(chat_count) = tracking.get_mut(chat_id) {
        chat_count.invited_users.retain(|u| u != username);
        chat_count.invited_count = chat_count.invited_users.len();
        // Sicurezza: assicurati che non risulti in chat
        chat_count.users_in_chat.retain(|u| u != username);
        chat_count.in_chat_count = chat_count.users_in_chat.len();
    }
    update_cpu_time(state.total_cpu_time.clone(), start);
}

// Invia notifica di abbandono definitivo per chat private
pub async fn send_chat_abandoned_notification(
    state: &AppState,
    chat_id: &str,
    abandoned_by: &str,
    remaining_user: &str,
) {
    let mut start = Instant::now();
    let abandoned_notification = ChatAbandonedNotification {
        chat_id: chat_id.to_string(),
        abandoned_by: abandoned_by.to_string(),
        remaining_user: remaining_user.to_string(),
        message: format!(
            "{} ha abbandonato la chat definitivamente e non potrà rientrare",
            abandoned_by
        ),
        is_private_chat: true,
    };

    let message = WebSocketMessage {
        message_type: MessageType::ChatAbandoned,
        data: serde_json::to_string(&abandoned_notification).unwrap(),
    };
    let message_json = serde_json::to_string(&message).unwrap();

    update_cpu_time(state.total_cpu_time.clone(), start);
    // Invia solo all'utente rimasto
    let users = state.connected_users.lock().unwrap();
    start = Instant::now();
    if let Some(connected_user) = users.get(remaining_user) {
        let _ = connected_user.sender.send(message_json);
    }
    update_cpu_time(state.total_cpu_time.clone(), start);
}

// Invia aggiornamento del conteggio utenti a tutti i partecipanti della chat
pub async fn broadcast_chat_users_count(state: &AppState, chat_id: &str) {
    let chat_count = {
        let tracking = state.chat_tracking.lock().unwrap();
        tracking.get(chat_id).cloned()
    };
    let mut start = Instant::now(); 
    if let Some(count_data) = chat_count {
        let message = WebSocketMessage {
            message_type: MessageType::ChatUsersCount,
            data: serde_json::to_string(&count_data).unwrap(),
        };
        let message_json = serde_json::to_string(&message).unwrap();
        update_cpu_time(state.total_cpu_time.clone(), start);
        // Invia a tutti gli utenti invitati (che potrebbero essere in chat o meno)
        let users = state.connected_users.lock().unwrap();
        start = Instant::now();
        for invited_user in &count_data.invited_users {
            if let Some(connected_user) = users.get(invited_user) {
                let _ = connected_user.sender.send(message_json.clone());
                }
        }
    }
    update_cpu_time(state.total_cpu_time.clone(), start);   
}

pub async fn check_and_notify_alone_in_chat(state: &AppState, chat_id: &str) {
    let mut start ;
    // Conta gli utenti nella chat specifica
    let users_in_chat: Vec<String> = {
        let users = state.connected_users.lock().unwrap();
        start = Instant::now(); 
        users
            .values()
            .filter(|user| user.user.chat_id.as_ref().map(|s| s.as_str()) == Some(chat_id))
            .map(|user| user.user.username.clone())
            .collect()
    };
    let count = users_in_chat.len();

    // Se c'è esattamente un utente, è solo
    if count == 1 {
        let alone_user = &users_in_chat[0];

        let alone_notification = AloneInChatNotification {
            chat_id: chat_id.to_string(),
            message: "Sei solo in questa chat. I tuoi messaggi non arriveranno a nessuno."
                .to_string(),
            is_alone: true,
        };

        let message = WebSocketMessage {
            message_type: MessageType::AloneInChat,
            data: serde_json::to_string(&alone_notification).unwrap(),
        };
        let message_json = serde_json::to_string(&message).unwrap();

        update_cpu_time(state.total_cpu_time.clone(), start);
        // Invia notifica solo all'utente che è rimasto solo
        let users = state.connected_users.lock().unwrap();
        start = Instant::now();
        if let Some(connected_user) = users.get(alone_user) {
            let _ = connected_user.sender.send(message_json);
        }
    } else if count > 1 {
        // Se ci sono più utenti, assicurati che nessuno abbia la notifica di solitudine
        let not_alone_notification = AloneInChatNotification {
            chat_id: chat_id.to_string(),
            message: " Altri utenti sono presenti nella chat.".to_string(),
            is_alone: false,
        };

        let message = WebSocketMessage {
            message_type: MessageType::AloneInChat,
            data: serde_json::to_string(&not_alone_notification).unwrap(),
        };
        let message_json = serde_json::to_string(&message).unwrap();

        update_cpu_time(state.total_cpu_time.clone(), start);
        // Invia a tutti gli utenti della chat
        let users = state.connected_users.lock().unwrap();
        start = Instant::now(); 
        for username in users_in_chat {
            if let Some(connected_user) = users.get(&username) {
                let _ = connected_user.sender.send(message_json.clone());
                }
        }
    }

    update_cpu_time(state.total_cpu_time.clone(), start);
}
