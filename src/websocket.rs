use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::time::Instant;

use crate::chat::{broadcast_chat_message, broadcast_user_left};
use crate::invites::{handle_invite_response, send_chat_invite};
use crate::notifications::invalidate_chat_ready_notifications;
use crate::performance::update_cpu_time;
use crate::state::{AppState, ConnectedUser};
use crate::tracking::{
    add_user_to_chat_tracking, check_and_notify_alone_in_chat, remove_user_from_chat_tracking,
};
use crate::types::{
    ChatInvite, ChatInviteResponse, ChatMessage, LoginRequest, MessageType, User, WebSocketMessage,
};
use crate::user::{
    broadcast_user_joined, broadcast_user_status_changed, send_users_list, send_users_list_to_all,
};

//WebSocket handler principale
pub async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

//Gestisce una singola connessione WebSocket toclean
pub async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    let mut username: Option<String> = None;
    let state_clone = state.clone();

    // Loop unico: gestisce sia invii che ricezioni senza spawn
    let _ws_closed = false;
    loop {
        tokio::select! {
            maybe_out = rx.recv() => {
                match maybe_out {
                    Some(msg) => {
                        if sender.send(Message::Text(msg)).await.is_err() {
                            break;
                        }
                    },
                    None => { /* canale chiuso */ }
                }
            },
            incoming = receiver.next() => {
                if let Some(msg) = incoming {
            let start = Instant::now();
            match msg {
                Ok(Message::Text(text)) => {
                    match serde_json::from_str::<WebSocketMessage>(&text) {
                        Ok(ws_msg) => {
                            match ws_msg.message_type {
                                MessageType::Login => {
                                    handle_login_message(&state_clone, &tx, &ws_msg, &mut username, start).await;
                                }
                                MessageType::ChatMessage => {
                                    handle_chat_message(&state_clone, &username, &ws_msg, start).await;
                                }
                                MessageType::UserStatusChanged => {
                                    handle_user_status_changed(&state_clone, &username, &ws_msg, start).await;
                                }
                                MessageType::ChatInvite => {
                                    handle_chat_invite(&state_clone, &username, &ws_msg, start).await;
                                }
                                MessageType::ChatInviteResponse => {
                                    handle_chat_invite_response(&state_clone, &username, &ws_msg, start).await;
                                }
                                _ => {}
                            }
                        }
                        Err(e) => {
                            println!("Error parsing WebSocket message: {}", e);
                            //aggiorna il tempo di CPU//
                            update_cpu_time(state.total_cpu_time.clone(), start);
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    //aggiorna il tempo di CPU//
                    update_cpu_time(state.total_cpu_time.clone(), start);
                    break;
                }
                Err(e) => {
                    println!("WebSocket error: {}", e);
                    //aggiorna il tempo di CPU//
                    update_cpu_time(state.total_cpu_time.clone(), start);
                    break;
                }
                _ => {}
            }
                } else {
                    let _ = _ws_closed;
                    break;
                }
            }
        }
    }

    //misura l'inizio di uso di cpu//
    let mut start = Instant::now();

    // Cleanup: rimuovi utente quando si disconnette (LOGOUT AUTOMATICO)
    let mut chat_id: Option<String> = None;
    if let Some(disconnected_username) = username {
        // Prima di rimuovere l'utente, assicurati che sia disponibile e fai il broadcast
        let updated_user = {
            //aggiorna il tempo di CPU//
            update_cpu_time(state.total_cpu_time.clone(), start);
            let mut users = state_clone.connected_users.lock().unwrap();
            start = Instant::now();
            if let Some(connected_user) = users.get_mut(&disconnected_username) {
                // Imposta l'utente come disponibile prima di rimuoverlo
                connected_user.user.is_available = true;
                let user_to_broadcast = connected_user.user.clone();

                if connected_user.user.chat_id.is_some() {
                    chat_id = connected_user.user.chat_id.clone();
                    connected_user.user.chat_id = None;
                }

                // Ora rimuovi l'utente
                users.remove(&disconnected_username);
                Some(user_to_broadcast)
            } else {
                // L'utente non esiste più, rimuovi solo per sicurezza
                users.remove(&disconnected_username);
                None
            }
        };
        //aggiorna il tempo di CPU//
        update_cpu_time(state.total_cpu_time.clone(), start);

        // Fai il broadcast dell'aggiornamento di stato prima di notificare la disconnessione
        if let Some(user_to_update) = updated_user {
            broadcast_user_status_changed(&state_clone, &user_to_update).await;
        }

        // Notifica tutti dell'uscita dell'utente
        broadcast_user_left(&state_clone, &disconnected_username, chat_id).await;
    }
}

async fn handle_login_message(
    state: &AppState,
    tx: &tokio::sync::mpsc::UnboundedSender<String>,
    ws_msg: &WebSocketMessage,
    username: &mut Option<String>,
    mut start: Instant,
) {
    if let Ok(login_req) = serde_json::from_str::<LoginRequest>(&ws_msg.data) {
        //CONTROLLO DUPLICATI - Verifica se l'username è già in uso
        {
            //aggiorna il tempo di CPU//
            update_cpu_time(state.total_cpu_time.clone(), start);

            let users = state.connected_users.lock().unwrap();
            start = Instant::now();
            if users.contains_key(&login_req.username) {
                // Invia errore al client
                let error_msg = WebSocketMessage {
                    message_type: MessageType::LoginError,
                    data: format!(
                        "Username '{}' è già in uso. Scegli un altro nome.",
                        login_req.username
                    ),
                };
                if let Ok(error_json) = serde_json::to_string(&error_msg) {
                    let _ = tx.send(error_json);
                }
                return;
            }
        }

        // Username disponibile - procedi con il login
        *username = Some(login_req.username.clone());

        // Genera session_id qui
        let session_id = uuid::Uuid::new_v4().to_string();

        let user = User {
            username: login_req.username.clone(),
            is_available: true,
            chat_id: None,
        };
        //aggiorna il tempo di CPU//
        update_cpu_time(state.total_cpu_time.clone(), start);
        // Registra l'utente
        {
            let mut users = state.connected_users.lock().unwrap();
            start = Instant::now();
            users.insert(
                login_req.username.clone(),
                ConnectedUser {
                    user: user.clone(),
                    sender: tx.clone(),
                    session_id: session_id.clone(),
                },
            );
        }

        // Conferma login riuscito
        let success_msg = WebSocketMessage {
            message_type: MessageType::LoginSuccess,
            data: format!(
                "Username '{}' è disponibile; session_id: {}",
                login_req.username, session_id
            ),
        };
        if let Ok(success_json) = serde_json::to_string(&success_msg) {
            let _ = tx.send(success_json);
        }
        //aggiorna il tempo di CPU//
        update_cpu_time(state.total_cpu_time.clone(), start);

        // Notifica tutti dell'ingresso del nuovo utente
        broadcast_user_joined(state, &user).await;

        // Invia lista utenti al nuovo utente
        send_users_list(tx, state).await;
    }
}

async fn handle_chat_message(
    state: &AppState,
    username: &Option<String>,
    ws_msg: &WebSocketMessage,
    start: Instant,
) {
    if let Some(ref current_username) = username {
        if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(&ws_msg.data) {
            //aggiorna il tempo di CPU//
            update_cpu_time(state.total_cpu_time.clone(), start);
            broadcast_chat_message(state, current_username, &chat_msg).await;
        }
    }
}

async fn handle_user_status_changed(
    state: &AppState,
    username: &Option<String>,
    ws_msg: &WebSocketMessage,
    mut start: Instant,
) {
    if let Some(ref current_username) = username {
        //Gestione stati avanzata con JSON
        if let Ok(status_data) = serde_json::from_str::<serde_json::Value>(&ws_msg.data) {
            let updated_user = {
                //aggiorna il tempo di CPU//
                update_cpu_time(state.total_cpu_time.clone(), start);

                let mut users = state.connected_users.lock().unwrap();
                start = Instant::now();
                if let Some(connected_user) = users.get_mut(current_username) {
                    //Gestione stato disponibilità
                    if let Some(available) = status_data.get("available").and_then(|v| v.as_bool())
                    {
                        connected_user.user.is_available = available;
                    }

                    if let Some(chat_id) = status_data.get("chatId") {
                        if chat_id.is_null() {
                            connected_user.user.chat_id = None;
                        } else if let Some(chat_id_str) = chat_id.as_str() {
                            connected_user.user.chat_id = Some(chat_id_str.to_string());
                        }
                    }

                    // Gestione uscita da chat (torna disponibile)
                    if let Some(in_chat) = status_data.get("inChat").and_then(|v| v.as_bool()) {
                        if !in_chat {
                            let old_chat_id = connected_user.user.chat_id.clone();
                            connected_user.user.is_available = true;
                            connected_user.user.chat_id = None;

                            // Invalida ChatReady esistenti per questa chat
                            if let Some(chat_id_str) = &old_chat_id {
                                let chat_id_for_invalidation = chat_id_str.clone();
                                let username_for_invalidation = current_username.clone();
                                tokio::spawn({
                                    let state_clone = state.clone();
                                    async move {
                                        let reason =
                                            format!("User {} left chat", username_for_invalidation);
                                        invalidate_chat_ready_notifications(
                                            &state_clone,
                                            &chat_id_for_invalidation,
                                            &reason,
                                        )
                                        .await;
                                    }
                                });
                            }

                            // NUOVO: Rimuovi utente dal tracking della chat
                            if let Some(chat_id_str) = &old_chat_id {
                                let chat_id_for_tracking = chat_id_str.clone();
                                let username_for_tracking = current_username.clone();
                                tokio::spawn({
                                    let state_clone = state.clone();
                                    async move {
                                        remove_user_from_chat_tracking(
                                            &state_clone,
                                            &chat_id_for_tracking,
                                            &username_for_tracking,
                                        )
                                        .await;
                                    }
                                });
                            }

                            // CONTROLLO: Controlla solitudine quando qualcuno esce
                            if let Some(chat_id_str) = old_chat_id {
                                //aggiorna il tempo di CPU//
                                update_cpu_time(state.total_cpu_time.clone(), start);
                                // Chiamata asincrona alla fine del blocco
                                tokio::spawn({
                                    let state_clone = state.clone();
                                    let chat_id_str = chat_id_str.clone();
                                    async move {
                                        check_and_notify_alone_in_chat(&state_clone, &chat_id_str)
                                            .await;
                                    }
                                });
                                start = Instant::now();
                            }
                        } else {
                            connected_user.user.is_available = false;

                            //NUOVO: Aggiungi utente al tracking della chat
                            if let Some(chat_id_str) = &connected_user.user.chat_id {
                                let chat_id_for_tracking = chat_id_str.clone();
                                let username_for_tracking = current_username.clone();
                                tokio::spawn({
                                    let state_clone = state.clone();
                                    async move {
                                        add_user_to_chat_tracking(
                                            &state_clone,
                                            &chat_id_for_tracking,
                                            &username_for_tracking,
                                        )
                                        .await;
                                    }
                                });
                            }

                            // CONTROLLO: Controlla solitudine quando qualcuno entra
                            if let Some(chat_id_str) = &connected_user.user.chat_id {
                                let chat_id_for_check = chat_id_str.clone();
                                //aggiorna il tempo di CPU//
                                update_cpu_time(state.total_cpu_time.clone(), start);
                                // Chiamata asincrona alla fine del blocco
                                tokio::spawn({
                                    let state_clone = state.clone();
                                    async move {
                                        check_and_notify_alone_in_chat(
                                            &state_clone,
                                            &chat_id_for_check,
                                        )
                                        .await;
                                    }
                                });
                                start = Instant::now();
                            }
                        }
                    }
                    //aggiorna il tempo di CPU//
                    update_cpu_time(state.total_cpu_time.clone(), start);
                    connected_user.user.clone()
                } else {
                    return; // Utente non trovato
                }
            };

            // Broadcast aggiornamento stato a tutti
            broadcast_user_status_changed(state, &updated_user).await;
            send_users_list_to_all(state).await;
        }
        // Aggiungi qui la gestione degli altri casi per status_data...
    }
}

async fn handle_chat_invite(
    state: &AppState,
    username: &Option<String>,
    ws_msg: &WebSocketMessage,
    start: Instant,
) {
    if let Some(ref current_username) = username {
        if let Ok(invite) = serde_json::from_str::<ChatInvite>(&ws_msg.data) {
            //aggiorna il tempo di CPU//
            update_cpu_time(state.total_cpu_time.clone(), start);
            // Invia l'invito ai destinatari
            send_chat_invite(state, current_username, &invite).await;
        }
    }
}

async fn handle_chat_invite_response(
    state: &AppState,
    username: &Option<String>,
    ws_msg: &WebSocketMessage,
    start: Instant,
) {
    if let Some(ref current_username) = username {
        if let Ok(response) = serde_json::from_str::<ChatInviteResponse>(&ws_msg.data) {
            //aggiorna il tempo di CPU//
            update_cpu_time(state.total_cpu_time.clone(), start);
            // Gestisce la risposta all'invito
            handle_invite_response(state, current_username, &response).await;
        }
    }
}
