use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub username: String,
    pub is_available: bool,
    pub chat_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginRequest {
    pub username: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessage {
    pub id: Uuid,
    pub chat_id: Option<String>,
    pub username: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub chat_type: ChatType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ChatType {
    Private { target: String },
    Group { members: Vec<String> },
    System,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatInvite {
    pub id: String,
    pub chat_id: Option<String>,
    pub from: String,
    pub from_session_id: String, // Sessione del mittente
    pub chat_type: ChatType,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatInviteResponse {
    pub invite_id: String,
    pub chat_id: Option<String>,
    pub accepted: bool,
    pub from_user: String,
    pub from_session_id: String, // Sessione del mittente
    pub chat_type: ChatType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatReady {
    pub chat_id: String,
    pub inviter: String,
    pub inviter_session_id: String, // Sessione del mittente
    pub chat_type: ChatType,
    pub accepted_by: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AloneInChatNotification {
    pub chat_id: String,
    pub message: String,
    pub is_alone: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatUsersCount {
    pub chat_id: String,
    pub invited_users: Vec<String>, // Tutti gli utenti invitati (incluso chi invita)
    pub users_in_chat: Vec<String>, // Solo utenti effettivamente in chat
    pub invited_count: usize,       // Numero totale invitati
    pub in_chat_count: usize,       // Numero effettivamente in chat
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatAbandonedNotification {
    pub chat_id: String,
    pub abandoned_by: String,   // Chi ha abbandonato la chat
    pub remaining_user: String, // Chi è rimasto
    pub message: String,        // Messaggio da mostrare
    pub is_private_chat: bool,  // Se è una chat privata
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatInvalidated {
    pub chat_id: String,
    pub reason: String,
}

// Notifica estesa per informare l'invitante dell'esito (accettato/rifiutato),
// includendo chi ha risposto.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatInviteResponseNotify {
    pub invite_id: String,
    pub chat_id: Option<String>,
    pub accepted: bool,
    pub from_user: String,
    pub from_session_id: String,
    pub chat_type: ChatType,
    pub responding_user: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebSocketMessage {
    pub message_type: MessageType,
    pub data: String, // JSON serialized data
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageType {
    Login,
    LoginSuccess,
    LoginError,
    ChatMessage,
    UserJoined,
    UserLeft,
    UserStatusChanged,
    UsersList,
    ChatInvite,
    ChatInviteResponse,
    ChatReady,
    AloneInChat,
    ChatUsersCount,  //aggiornamenti conteggio utenti chat
    ChatAbandoned,   // notifica abbandono definitivo chat privata
    ChatInvalidated, // invalida ChatReady obsolete
    Error,
}
