use crate::chat::broadcast_chat_message;
use crate::performance::update_cpu_time;
use crate::state::AppState;
use crate::tracking::{broadcast_chat_users_count, init_chat_tracking, remove_user_from_invited};
use crate::types::{
    ChatInvite, ChatInviteResponse, ChatInviteResponseNotify, ChatMessage, ChatReady, MessageType,
    WebSocketMessage,
};
use std::time::Instant;
use uuid;

//Gestione inviti chat
pub async fn send_chat_invite(state: &AppState, from_username: &str, invite: &ChatInvite) {
    let mut start = Instant::now();
    let message = WebSocketMessage {
        message_type: MessageType::ChatInvite,
        data: serde_json::to_string(invite).unwrap(),
    };
    let message_json = serde_json::to_string(&message).unwrap();
    //aggiorna il tempo di CPU//
    update_cpu_time(state.total_cpu_time.clone(), start);
    let users = state.connected_users.lock().unwrap();
    start = Instant::now();

    //Inizializza tracking utenti per questa chat
    let invited_users = match &invite.chat_type {
        crate::types::ChatType::Private { target } => {
            vec![from_username.to_string(), target.clone()]
        }
        crate::types::ChatType::Group { members } => members.clone(),
        _ => vec![],
    };

    if let Some(chat_id) = &invite.chat_id {
        init_chat_tracking(state, chat_id, invited_users);
    }

    //Determina i destinatari dell'invito in base al tipo di chat
    match &invite.chat_type {
        crate::types::ChatType::Private { target } => {
            // Invito per chat privata: invia solo al target
            if let Some(connected_user) = users.get(target) {
                let _ = connected_user.sender.send(message_json);
            }
        }
        crate::types::ChatType::Group { members } => {
            // Invito per chat di gruppo: invia a tutti i membri tranne il mittente
            for member in members {
                if member != from_username {
                    if let Some(connected_user) = users.get(member) {
                        let _ = connected_user.sender.send(message_json.clone());
                    }
                }
            }
        }
        _ => {}
    }
    //aggiorna il tempo di CPU//
    update_cpu_time(state.total_cpu_time.clone(), start);
}

pub async fn handle_invite_response(
    state: &AppState,
    responding_user: &str,
    response: &ChatInviteResponse,
) {
    let mut start = Instant::now();

    if response.accepted {
        // Quando qualcuno accetta, invia una notifica al mittente dell'invito
        // che la chat è pronta per essere aperta
        let chat_ready = ChatReady {
            chat_id: response
                .chat_id
                .clone()
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            inviter: response.from_user.clone(),
            inviter_session_id: response.from_session_id.clone(),
            chat_type: response.chat_type.clone(),
            accepted_by: responding_user.to_string(),
        };

        let ready_message = WebSocketMessage {
            message_type: MessageType::ChatReady,
            data: serde_json::to_string(&chat_ready).unwrap(),
        };
        let ready_message_json = serde_json::to_string(&ready_message).unwrap();

        let system_message = ChatMessage {
            id: uuid::Uuid::new_v4(),
            chat_id: response.chat_id.clone(),
            username: "Sistema".to_string(),
            content: format!("{} è entrato nella chat", responding_user),
            timestamp: chrono::Utc::now(),
            chat_type: crate::types::ChatType::System,
        };
        
        update_cpu_time(state.total_cpu_time.clone(), start);
        broadcast_chat_message(state, "Sistema", &system_message).await;
        start = Instant::now(); 
        // Invia la notifica "chat pronta" al mittente dell'invito
        let users = state.connected_users.lock().unwrap();

        // Cerca il mittente con session_id corrispondente
        let inviter_user = users
            .get(&response.from_user)
            .filter(|cu| cu.session_id == response.from_session_id);
        if let Some(inviter_user) = inviter_user {
            let _ = inviter_user.sender.send(ready_message_json);
        }

        // Invia conferma di accettazione a chi ha risposto
        let response_message = WebSocketMessage {
            message_type: MessageType::ChatInviteResponse,
            data: serde_json::to_string(response).unwrap(),
        };
        let response_json = serde_json::to_string(&response_message).unwrap();

        if let Some(responding_user_conn) = users.get(responding_user) {
            let _ = responding_user_conn.sender.send(response_json);
        }
        update_cpu_time(state.total_cpu_time.clone(), start);
    } else {
        // Se rifiutato, invia solo la risposta negativa al mittente
        // Estende il payload con chi ha rifiutato così il client può mostrarlo
        let notify = ChatInviteResponseNotify {
            invite_id: response.invite_id.clone(),
            chat_id: response.chat_id.clone(),
            accepted: false,
            from_user: response.from_user.clone(),
            from_session_id: response.from_session_id.clone(),
            chat_type: response.chat_type.clone(),
            responding_user: responding_user.to_string(),
        };
        let response_message = WebSocketMessage {
            message_type: MessageType::ChatInviteResponse,
            data: serde_json::to_string(&notify).unwrap(),
        };
        let response_json = serde_json::to_string(&response_message).unwrap();
        update_cpu_time(state.total_cpu_time.clone(), start);

        {
            let users = state.connected_users.lock().unwrap();
            start = Instant::now();
            let inviter_user = users
                .get(&response.from_user)
                .filter(|cu| cu.session_id == response.from_session_id);
            if let Some(inviter_user) = inviter_user {
                let _ = inviter_user.sender.send(response_json);
            }
            update_cpu_time(state.total_cpu_time.clone(), start);
        }
        
        
        // Aggiorna i partecipanti invitati con il conteggio utenti della chat (se disponibile)
        if let Some(ref chat_id) = response.chat_id {
            // Rimuovi chi ha rifiutato dagli invitati di questa chat
            remove_user_from_invited(state, chat_id, responding_user).await;
            // Poi notifica il nuovo conteggio ai rimanenti invitati
            broadcast_chat_users_count(state, chat_id).await;
            
        }
    }
}
