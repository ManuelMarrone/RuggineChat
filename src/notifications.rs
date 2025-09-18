use crate::performance::update_cpu_time;
use crate::state::AppState;
use crate::types::{ChatInvalidated, MessageType, WebSocketMessage};
use crate::user::broadcast_to_all;
use std::time::Instant;

/// Invalida le notifiche ChatReady obsolete per una specifica chat.
/// Viene chiamata quando una chat non è più disponibile (disconnessioni, abbandoni).
/// Invia un messaggio di invalidazione a tutti i client per rimuovere notifiche obsolete.
pub async fn invalidate_chat_ready_notifications(state: &AppState, chat_id: &str, reason: &str) {
    let start = Instant::now();

    let chat_invalidated = ChatInvalidated {
        chat_id: chat_id.to_string(),
        reason: reason.to_string(),
    };

    let invalidation_message = WebSocketMessage {
        message_type: MessageType::ChatInvalidated,
        data: serde_json::to_string(&chat_invalidated).unwrap(),
    };
    
    update_cpu_time(state.total_cpu_time.clone(), start);
    // Invia a tutti gli utenti connessi (le notifiche ChatReady potrebbero essere su qualsiasi client)
    broadcast_to_all(state, &invalidation_message).await;

}
