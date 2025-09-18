// Test di integrazione per il server WebSocket (Axum)
// ---------------------------------------------------
// Obiettivi:
// - Verificare il rifiuto del login con username duplicato
// - Verificare l'invio e la ricezione di un invito privato
// - Verificare il broadcast di un messaggio di gruppo ai membri
//
// Strategia:
// - Avvio di un processo reale del server (`fullstack-app`) con PORT=0 (porta effimera)
// - Lettura dello stdout per ricavare l'endpoint WebSocket effettivo (es. ws://127.0.0.1:12345/ws)
// - Connessione di client WebSocket reali (tokio-tungstenite) per scambiare messaggi come farebbe il frontend
//
// Note su Windows:
// - Lo stdout del processo viene drenato continuamente in background per evitare errori di "pipe chiusa"
// - Se un processo server rimane attivo tra un run e l'altro, può bloccare l'aggiornamento del binario: terminare i processi residui

use futures_util::{SinkExt, StreamExt};
use std::{clone, net::SocketAddr, sync::{Arc, Mutex}, time::Duration};
use tokio::{sync::mpsc, net::TcpListener};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use axum::Router;
use tower_http::cors::CorsLayer;
use fullstack_app::{create_app, AppState};
use fullstack_app::types; // importiamo i tipi dal crate invece di duplicarli

// Semplice wrapper per ogni client connesso ai fini del test:
// - `sender`: la metà di scrittura del WebSocket per inviare frame testuali
// - `rx`: canale che riceve i messaggi JSON già deserializzati in `WebSocketMessage`

struct TestClient {
    sender: futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        WsMessage,
    >,
    rx: mpsc::UnboundedReceiver<types::WebSocketMessage>,
}

// Avvia un server Axum in-process su porta effimera e ritorna l'URL WS.
async fn start_test_server() -> (String, tokio::task::JoinHandle<()>) {
    // Stato condiviso con timer CPU inizializzato a ZERO
    let total_cpu_time: Arc<Mutex<Duration>> = Arc::new(Mutex::new(Duration::ZERO));
    let state = AppState::new(total_cpu_time.clone());

    // CORS minimale, come nel main
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

    let app: Router = create_app(state, cors);
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    let ws_url = format!("ws://{}/ws", addr);
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (ws_url, handle)
}

// Apre una connessione WebSocket al server, splitta lettura/scrittura e
// inoltra i frame testuali in un canale tipizzato con `WebSocketMessage`.
async fn connect_client(ws_url: &str) -> TestClient {
    let (stream, _) = tokio_tungstenite::connect_async(ws_url).await.unwrap();
    let (write, mut read) = stream.split();
    let (tx, rx) = mpsc::unbounded_channel::<types::WebSocketMessage>();
    tokio::spawn(async move {
        while let Some(Ok(msg)) = read.next().await {
            if let WsMessage::Text(txt) = msg {
                if let Ok(parsed) = serde_json::from_str::<types::WebSocketMessage>(&txt) {
                    let _ = tx.send(parsed);
                }
            }
        }
    });
    TestClient { sender: write, rx }
}

// Helper per inviare un messaggio WS serializzato come
// { message_type, data: json(payload) }.
async fn send_ws<T: serde::Serialize>(client: &mut TestClient, message_type: types::MessageType, payload: T) {
    let msg = types::WebSocketMessage { message_type, data: serde_json::to_string(&payload).unwrap() };
    let txt = serde_json::to_string(&msg).unwrap();
    client.sender.send(WsMessage::Text(txt)).await.unwrap();
}

/// Riceve in modo asincrono i messaggi dal canale `rx` e restituisce il primo
/// che soddisfa il predicato `pred`, oppure `None` se scade `timeout_ms` o se
/// il canale viene chiuso prima di trovare una corrispondenza.
///
/// Note importanti:
/// - Consuma i messaggi dal canale: quelli che non soddisfano `pred` vengono scartati (non vengono reinseriti).
/// - `timeout_ms` è espresso in millisecondi ed è un limite massimo di attesa.
/// - `pred` riceve un riferimento al `WebSocketMessage` già deserializzato.
///
/// Parametri:
/// - `rx`: canale unbounded che riceve i messaggi per quel client.
/// - `pred`: funzione che ritorna `true` quando il messaggio desiderato è stato visto.
/// - `timeout_ms`: tempo massimo di attesa in millisecondi.
///
/// Ritorno:
/// - `Some(msg)` se un messaggio soddisfa `pred` entro il timeout.
/// - `None` se scade il timeout o se il canale si chiude prima.
async fn recv_until<F>(rx: &mut mpsc::UnboundedReceiver<types::WebSocketMessage>, mut pred: F, timeout_ms: u64) -> Option<types::WebSocketMessage>
where
    F: FnMut(&types::WebSocketMessage) -> bool,
{
    let deadline = tokio::time::Duration::from_millis(timeout_ms);
    let fut = async {
        loop {
            if let Some(msg) = rx.recv().await {
                if pred(&msg) { return Some(msg); }
            } else { return None; }
        }
    };
    tokio::time::timeout(deadline, fut).await.ok().flatten()
}

// Attende che il server confermi (tramite `UserStatusChanged`) che l'utente
// `username` sia effettivamente nella chat con `chat_id` e non disponibile.
async fn wait_user_in_chat(
    rx: &mut mpsc::UnboundedReceiver<types::WebSocketMessage>,
    username: &str,
    chat_id: &str,
) {
    let got = recv_until(
        rx,
        |m| {
            if let types::MessageType::UserStatusChanged = m.message_type {
                if let Ok(u) = serde_json::from_str::<types::User>(&m.data) {
                    return u.username == username && u.chat_id.as_deref() == Some(chat_id) && !u.is_available;
                }
            }
            false
        },
        3000,
    ).await;
    assert!(got.is_some(), "{} should be marked in chat {}", username, chat_id);
}

// Test 1: rifiuto del login con username duplicato
// Passi:
// - Avvio server e connessione di due client
// - Il primo fa login con "mario" e riceve LoginSuccess
// - Il secondo tenta lo stesso username e deve ricevere LoginError
#[tokio::test]
async fn test_duplicate_login_rejected() {
    let (ws_url, _handle) = start_test_server().await;
    let mut c1 = connect_client(&ws_url).await;
    let mut c2 = connect_client(&ws_url).await;

    send_ws(&mut c1, types::MessageType::Login, types::LoginRequest { username: "mario".into() }).await;
    let _ = recv_until(&mut c1.rx, |m| matches!(m.message_type, types::MessageType::LoginSuccess), 2000).await;

    send_ws(&mut c2, types::MessageType::Login, types::LoginRequest { username: "mario".into() }).await;
    let got_error = recv_until(&mut c2.rx, |m| matches!(m.message_type, types::MessageType::LoginError), 2000).await;
    assert!(got_error.is_some());
}

// Test 2: invio e ricezione di un invito privato
// Passi:
// - Avvio server e login di alice e bob
// - alice invia ChatInvite(private) a bob
// - bob deve ricevere un ChatInvite con lo stesso id
#[tokio::test]
async fn test_invite_delivery_private() {
    let (ws_url, _handle) = start_test_server().await;
    let mut alice = connect_client(&ws_url).await;
    let mut bob = connect_client(&ws_url).await;

    send_ws(&mut alice, types::MessageType::Login, types::LoginRequest { username: "alice".into() }).await;
    let _ = recv_until(&mut alice.rx, |m| matches!(m.message_type, types::MessageType::LoginSuccess), 2000).await;
    send_ws(&mut bob, types::MessageType::Login, types::LoginRequest { username: "bob".into() }).await;
    let _ = recv_until(&mut bob.rx, |m| matches!(m.message_type, types::MessageType::LoginSuccess), 2000).await;

    let invite = types::ChatInvite {
        id: "inv1".into(),
        chat_id: Some("chat-1".into()),
        from: "alice".into(),
        from_session_id: "dummy".into(),
        chat_type: types::ChatType::Private { target: "bob".into() },
        message: "Join me".into(),
        timestamp: chrono::Utc::now(),
    };
    send_ws(&mut alice, types::MessageType::ChatInvite, invite.clone()).await;

    let got_invite = recv_until(&mut bob.rx, |m| matches!(m.message_type, types::MessageType::ChatInvite), 2000).await
        .expect("bob should receive invite");
    let payload: types::ChatInvite = serde_json::from_str(&got_invite.data).unwrap();
    assert_eq!(payload.id, invite.id);
}

// Test 3: broadcast di un messaggio di gruppo a tutti i membri
// Passi:
// - Avvio server, login di alice, bob, carol
// - Tutti segnalano `UserStatusChanged` con lo stesso `chatId` (group-1)
// - Attendo conferma per ciascuno (UserStatusChanged coerente)
// - alice invia ChatMessage(Group) con chatId=group-1
// - bob e carol devono ricevere il medesimo ChatMessage
#[tokio::test]
async fn test_group_message_broadcast_to_all_members() {
    let (ws_url, _handle) = start_test_server().await;
    let mut a = connect_client(&ws_url).await;
    let mut b = connect_client(&ws_url).await;
    let mut c = connect_client(&ws_url).await;

    send_ws(&mut a, types::MessageType::Login, types::LoginRequest { username: "alice".into() }).await;
    let _ = recv_until(&mut a.rx, |m| matches!(m.message_type, types::MessageType::LoginSuccess), 2000).await;
    send_ws(&mut b, types::MessageType::Login, types::LoginRequest { username: "bob".into() }).await;
    let _ = recv_until(&mut b.rx, |m| matches!(m.message_type, types::MessageType::LoginSuccess), 2000).await;
    send_ws(&mut c, types::MessageType::Login, types::LoginRequest { username: "carol".into() }).await;
    let _ = recv_until(&mut c.rx, |m| matches!(m.message_type, types::MessageType::LoginSuccess), 2000).await;

    let status = serde_json::json!({
        "available": false,
        "inChat": true,
        "chatId": "group-1",
        "members": ["alice", "bob", "carol"]
    });
    send_ws(&mut a, types::MessageType::UserStatusChanged, status.clone()).await;
    send_ws(&mut b, types::MessageType::UserStatusChanged, status.clone()).await;
    send_ws(&mut c, types::MessageType::UserStatusChanged, status.clone()).await;

    wait_user_in_chat(&mut a.rx, "alice", "group-1").await;
    wait_user_in_chat(&mut b.rx, "bob", "group-1").await;
    wait_user_in_chat(&mut c.rx, "carol", "group-1").await;

    let chat_msg = types::ChatMessage {
        id: uuid::Uuid::new_v4(),
        chat_id: Some("group-1".into()),
        username: "alice".into(),
        content: "ciao gruppo".into(),
        timestamp: chrono::Utc::now(),
        chat_type: types::ChatType::Group { members: vec!["alice".into(), "bob".into(), "carol".into()] },
    };
    send_ws(&mut a, types::MessageType::ChatMessage, chat_msg.clone()).await;

    let msg_b = recv_until(&mut b.rx, |m| matches!(m.message_type, types::MessageType::ChatMessage), 3000).await
        .expect("bob should receive group message");
    let parsed_b: types::ChatMessage = serde_json::from_str(&msg_b.data).unwrap();
    assert_eq!(parsed_b.content, chat_msg.content);

    let msg_c = recv_until(&mut c.rx, |m| matches!(m.message_type, types::MessageType::ChatMessage), 3000).await
        .expect("carol should receive group message");
    let parsed_c: types::ChatMessage = serde_json::from_str(&msg_c.data).unwrap();
    assert_eq!(parsed_c.content, chat_msg.content);
}

//Test 4: ricezione di un messaggio solo dai componenti della stessa chat
// Passi:
// - Avvio server, login di alice, bob, carol, dave
// - alice e bob segnalano `UserStatusChanged` con lo stesso `chatId` (group-1)
// - carol e dave segnalano `UserStatusChanged` con lo stesso `chatId` (group-2)
// - Attendo conferma per ciascuno (UserStatusChanged coerente)
// - alice invia ChatMessage(Group) con chatId=group-1
// - bob deve ricevere il messaggio
// - carol e dave non devono ricevere il messaggio
#[tokio::test]
async fn test_multiple_chats_correct_deliver() {
    let (ws_url, _handle) = start_test_server().await;
    let mut a = connect_client(&ws_url).await;
    let mut b = connect_client(&ws_url).await;
    let mut c = connect_client(&ws_url).await;
    let mut d = connect_client(&ws_url).await;

    send_ws(&mut a, types::MessageType::Login, types::LoginRequest { username: "alice".into() }).await;
    let _ = recv_until(&mut a.rx, |m| matches!(m.message_type, types::MessageType::LoginSuccess), 2000).await;
    send_ws(&mut b, types::MessageType::Login, types::LoginRequest { username: "bob".into() }).await;
    let _ = recv_until(&mut b.rx, |m| matches!(m.message_type, types::MessageType::LoginSuccess), 2000).await;
    send_ws(&mut c, types::MessageType::Login, types::LoginRequest { username: "carol".into() }).await;
    let _ = recv_until(&mut c.rx, |m| matches!(m.message_type, types::MessageType::LoginSuccess), 2000).await;
    send_ws(&mut d, types::MessageType::Login, types::LoginRequest { username: "dave".into() }).await;
    let _ = recv_until(&mut d.rx, |m| matches!(m.message_type, types::MessageType::LoginSuccess), 2000).await;

    let status1 = serde_json::json!({
        "available": false,
        "inChat": true,
        "chatId": "group-1",
        "members": ["alice", "bob"]
    });
    send_ws(&mut a, types::MessageType::UserStatusChanged, status1.clone()).await;
    send_ws(&mut b, types::MessageType::UserStatusChanged, status1.clone()).await;

    let status2 = serde_json::json!({
        "available": false,
        "inChat": true,
        "chatId": "group-2",
        "members": ["carol", "dave"]
    });
    send_ws(&mut c, types::MessageType::UserStatusChanged, status2.clone()).await;
    send_ws(&mut d, types::MessageType::UserStatusChanged, status2.clone()).await;
    wait_user_in_chat(&mut a.rx, "alice", "group-1").await;
    wait_user_in_chat(&mut b.rx, "bob", "group-1").await;
    wait_user_in_chat(&mut c.rx, "carol", "group-2").await;
    wait_user_in_chat(&mut d.rx, "dave", "group-2").await;
    let chat_msg1 = types::ChatMessage {
        id: uuid::Uuid::new_v4(),
        chat_id: Some("group-1".into()),
        username: "alice".into(),
        content: "ciao gruppo 1".into(),
        timestamp: chrono::Utc::now(),
        chat_type: types::ChatType::Group { members: vec!["alice".into(), "bob".into()] },
    };
    send_ws(&mut a, types::MessageType::ChatMessage, chat_msg1.clone()).await;
    let msg_b = recv_until(&mut b.rx, |m| matches!(m.message_type, types::MessageType::ChatMessage), 3000).await
        .expect("bob should receive group message");
    let parsed_b: types::ChatMessage = serde_json::from_str(&msg_b.data).unwrap();
    assert_eq!(parsed_b.content, chat_msg1.content);
    let no_msg_c = recv_until(&mut c.rx, |m| matches!(m.message_type, types::MessageType::ChatMessage), 1000).await;
    assert!(no_msg_c.is_none(), "carol should NOT receive group-1 message");
    let no_msg_d = recv_until(&mut d.rx, |m| matches!(m.message_type, types::MessageType::ChatMessage), 1000).await;
    assert!(no_msg_d.is_none(), "dave should NOT receive group-1 message");
}

//* Performance test (PTest) *//

//PTest 1 latenza di invio-recezione di un messaggio
#[tokio::test]
async fn test_group_message_latency() {
    let (ws_url, _handle) = start_test_server().await;
    let mut sender = connect_client(&ws_url).await;
    let mut receiver = connect_client(&ws_url).await;

    // Login
    send_ws(&mut sender, types::MessageType::Login, types::LoginRequest { username: "alice".into() }).await;
    let _ = recv_until(&mut sender.rx, |m| matches!(m.message_type, types::MessageType::LoginSuccess), 2000).await;
    send_ws(&mut receiver, types::MessageType::Login, types::LoginRequest { username: "bob".into() }).await;
    let _ = recv_until(&mut receiver.rx, |m| matches!(m.message_type, types::MessageType::LoginSuccess), 2000).await;

    // Entrambi entrano nella stessa chat
    let status = serde_json::json!({
        "available": false,
        "inChat": true,
        "chatId": "group-latency",
        "members": ["alice", "bob"]
    });
    send_ws(&mut sender, types::MessageType::UserStatusChanged, status.clone()).await;
    send_ws(&mut receiver, types::MessageType::UserStatusChanged, status.clone()).await;
    wait_user_in_chat(&mut sender.rx, "alice", "group-latency").await;
    wait_user_in_chat(&mut receiver.rx, "bob", "group-latency").await;

    // Misura tempo di invio/ricezione
    let chat_msg = types::ChatMessage {
        id: uuid::Uuid::new_v4(),
        chat_id: Some("group-latency".into()),
        username: "alice".into(),
        content: "misura latenza".into(),
        timestamp: chrono::Utc::now(),
        chat_type: types::ChatType::Group { members: vec!["alice".into(), "bob".into()] },
    };

    let start = std::time::Instant::now();
    send_ws(&mut sender, types::MessageType::ChatMessage, chat_msg.clone()).await;
    let msg = recv_until(&mut receiver.rx, |m| matches!(m.message_type, types::MessageType::ChatMessage), 3000).await
        .expect("bob should receive group message");
    let elapsed_ms = start.elapsed().as_micros();

    println!("Tempo di latenza: {} µs", elapsed_ms);
    assert!(elapsed_ms < 3000, "Messaggio ricevuto troppo tardi");
}

//PTest 2 latenza di invio-recezione di un numero arbitrario messaggi
//con 1000    159.65 µs
//con 10000   156.73 µs
//con 1000000 155.38 µs
#[tokio::test]
async fn test_group_message_average_latency() {
    const N_MESSAGES: usize = 1000; //Numero di iterazioni

    let (ws_url, _handle) = start_test_server().await;
    let mut sender = connect_client(&ws_url).await;
    let mut receiver = connect_client(&ws_url).await;

    // Login
    send_ws(&mut sender, types::MessageType::Login, types::LoginRequest { username: "alice".into() }).await;
    let _ = recv_until(&mut sender.rx, |m| matches!(m.message_type, types::MessageType::LoginSuccess), 2000).await;
    send_ws(&mut receiver, types::MessageType::Login, types::LoginRequest { username: "bob".into() }).await;
    let _ = recv_until(&mut receiver.rx, |m| matches!(m.message_type, types::MessageType::LoginSuccess), 2000).await;

    // Entrambi entrano nella stessa chat
    let status = serde_json::json!({
        "available": false,
        "inChat": true,
        "chatId": "group-latency",
        "members": ["alice", "bob"]
    });
    send_ws(&mut sender, types::MessageType::UserStatusChanged, status.clone()).await;
    send_ws(&mut receiver, types::MessageType::UserStatusChanged, status.clone()).await;
    wait_user_in_chat(&mut sender.rx, "alice", "group-latency").await;
    wait_user_in_chat(&mut receiver.rx, "bob", "group-latency").await;

    // Invio multiplo e calcolo media
    let mut latencies = Vec::with_capacity(N_MESSAGES);
    for i in 0..N_MESSAGES {
        let chat_msg = types::ChatMessage {
            id: uuid::Uuid::new_v4(),
            chat_id: Some("group-latency".into()),
            username: "alice".into(),
            content: format!("msg {}", i),
            timestamp: chrono::Utc::now(),
            chat_type: types::ChatType::Group { members: vec!["alice".into(), "bob".into()] },
        };

        let start = std::time::Instant::now();
        send_ws(&mut sender, types::MessageType::ChatMessage, chat_msg.clone()).await;
        let msg = recv_until(&mut receiver.rx, |m| matches!(m.message_type, types::MessageType::ChatMessage), 3000).await
            .expect("bob should receive group message");
        let elapsed_ms = start.elapsed().as_micros();
        latencies.push(elapsed_ms);
    }

    let avg_latency = latencies.iter().copied().sum::<u128>() as f64 / N_MESSAGES as f64;
    println!("Tempo medio di latenza su {} messaggi: {:.2} µs", N_MESSAGES, avg_latency);
    assert!(avg_latency < 3000_000.0, "Messaggi ricevuti troppo tardi in in media");
}

//PTest 3 misura latenza media con un numero arbitrario di byte inviati
//Latenza media invio/ricezione per    10 byte su 10000 iterazioni: 162.23 µs
//Latenza media invio/ricezione per  1000 byte su 10000 iterazioni: 291.38 µs
//Latenza media invio/ricezione per 10000 byte su 10000 iterazioni: 1416.41 µs

#[tokio::test]
async fn test_group_message_average_latency_with_arbitrary_size() {
    const N_BYTES: usize = 10000; // Dimensione del messaggio
    const N_ITER: usize = 10000;   // Numero di iterazioni

    let (ws_url, _handle) = start_test_server().await;
    let mut sender = connect_client(&ws_url).await;
    let mut receiver = connect_client(&ws_url).await;

    // Login
    send_ws(&mut sender, types::MessageType::Login, types::LoginRequest { username: "alice".into() }).await;
    let _ = recv_until(&mut sender.rx, |m| matches!(m.message_type, types::MessageType::LoginSuccess), 2000).await;
    send_ws(&mut receiver, types::MessageType::Login, types::LoginRequest { username: "bob".into() }).await;
    let _ = recv_until(&mut receiver.rx, |m| matches!(m.message_type, types::MessageType::LoginSuccess), 2000).await;

    // Entrambi entrano nella stessa chat
    let status = serde_json::json!({
        "available": false,
        "inChat": true,
        "chatId": "group-arbitrary-size",
        "members": ["alice", "bob"]
    });
    send_ws(&mut sender, types::MessageType::UserStatusChanged, status.clone()).await;
    send_ws(&mut receiver, types::MessageType::UserStatusChanged, status.clone()).await;
    wait_user_in_chat(&mut sender.rx, "alice", "group-arbitrary-size").await;
    wait_user_in_chat(&mut receiver.rx, "bob", "group-arbitrary-size").await;

    // Crea un payload di N_BYTES
    let big_content = "X".repeat(N_BYTES);

    let mut latencies = Vec::with_capacity(N_ITER);
    for _ in 0..N_ITER {
        let chat_msg = types::ChatMessage {
            id: uuid::Uuid::new_v4(),
            chat_id: Some("group-arbitrary-size".into()),
            username: "alice".into(),
            content: big_content.clone(),
            timestamp: chrono::Utc::now(),
            chat_type: types::ChatType::Group { members: vec!["alice".into(), "bob".into()] },
        };

        let start = std::time::Instant::now();
        send_ws(&mut sender, types::MessageType::ChatMessage, chat_msg.clone()).await;
        let _ = recv_until(&mut receiver.rx, |m| matches!(m.message_type, types::MessageType::ChatMessage), 5000).await
            .expect("bob should receive group message");
        let elapsed_us = start.elapsed().as_micros();
        latencies.push(elapsed_us);
    }

    let avg_latency = latencies.iter().copied().sum::<u128>() as f64 / N_ITER as f64;
    println!("Latenza media invio/ricezione per {} byte su {} iterazioni: {:.2} µs", N_BYTES, N_ITER, avg_latency);
    assert!(avg_latency < 5_000_000.0, "Messaggi ricevuti troppo tardi in media");
}


//PTest 4 test di misurazione 
/*
Statistiche latenza con 1 utenti contemporanei:
- Messaggi per iterazione: 1
- Dimensione messaggi: 1024 bytes
- Numero iterazioni: 100
- Latenza media: 362.62 µs
- Latenza minima: 274 µs
- Latenza massima: 595 µs 
*/
/*
Statistiche latenza con 10 utenti contemporanei:
- Dimensione messaggi: 1024 bytes
- Numero iterazioni: 100
- Latenza media: 5305.48 µs
- Latenza minima: 5051 µs
- Latenza massima: 5548 µs
*/
/*
Statistiche latenza con 100 utenti contemporanei:
- Messaggi per iterazione: 100
- Dimensione messaggi: 1024 bytes
- Numero iterazioni: 100
- Latenza media: 711187.78 µs
- Latenza minima: 699307 µs
- Latenza massima: 752172 µs
*/

#[tokio::test]
async fn test_multiple_concurrent_users_latency() {
    const NUM_USERS: usize = 100;    
    const N_BYTES: usize = 90;     
    const N_ITER: usize = 100;        

    let (ws_url, _handle) = start_test_server().await;

    // Creazione degli utenti e login
    let mut users = Vec::with_capacity(NUM_USERS);
    for i in 0..NUM_USERS {
        let mut client = connect_client(&ws_url).await;
        send_ws(&mut client, types::MessageType::Login, types::LoginRequest { 
            username: format!("user{}", i) 
        }).await;
        let _ = recv_until(&mut client.rx, |m| {
            matches!(m.message_type, types::MessageType::LoginSuccess)
        }, 2000).await;
        users.push(client);
    }

    // Tutti entrano nella stessa chat
    let members: Vec<String> = (0..NUM_USERS).map(|i| format!("user{}", i)).collect();
    let status = serde_json::json!({
        "available": false,
        "inChat": true,
        "chatId": "concurrent-test",
        "members": members.clone()
    });

    for user in &mut users {
        send_ws(user, types::MessageType::UserStatusChanged, status.clone()).await;
    }

    // Attendi che tutti siano nella chat
    for (i, user) in users.iter_mut().enumerate() {
        wait_user_in_chat(&mut user.rx, &format!("user{}", i), "concurrent-test").await;
    }

    let content = "X".repeat(N_BYTES);
    let mut iteration_latencies = Vec::with_capacity(N_ITER);

    // Esegui N_ITER volte il test di invio contemporaneo
    for iter in 0..N_ITER {
        let start = std::time::Instant::now();
        

        // Invia messaggi in parallelo
        for (i, user) in users.iter_mut().enumerate() {
            let chat_msg = types::ChatMessage {
                id: uuid::Uuid::new_v4(),
                chat_id: Some("concurrent-test".into()),
                username: format!("user{}", i),
                content: format!("{}-iter{}", content, iter),
                timestamp: chrono::Utc::now(),
                chat_type: types::ChatType::Group { members: members.clone() },
            };
            
            send_ws(user, types::MessageType::ChatMessage, chat_msg.clone()).await;
        }

        // Attendi che ogni utente riceva NUM_USERS messaggi
        for user in users.iter_mut() {
            for _ in 0..NUM_USERS {
                let _ = recv_until(&mut user.rx, |m| {
                    matches!(m.message_type, types::MessageType::ChatMessage)
                }, 10000).await.expect("Message not received in time");
            }
        }

        let elapsed = start.elapsed();
        iteration_latencies.push(elapsed.as_micros());
        
        println!("Iterazione {} completata in {} µs", iter + 1, elapsed.as_micros());
    }

    // Calcola e mostra le statistiche
    let avg_latency = iteration_latencies.iter().sum::<u128>() as f64 / N_ITER as f64;
    let min_latency = iteration_latencies.iter().min().unwrap();
    let max_latency = iteration_latencies.iter().max().unwrap();

    println!("\nStatistiche latenza con {} utenti contemporanei:", NUM_USERS);
    println!("- Messaggi per iterazione: {}", NUM_USERS);
    println!("- Dimensione messaggi: {} bytes", N_BYTES);
    println!("- Numero iterazioni: {}", N_ITER);
    println!("- Latenza media: {:.2} µs", avg_latency);
    println!("- Latenza minima: {} µs", min_latency);
    println!("- Latenza massima: {} µs", max_latency);

    assert!(avg_latency < 5_000_000.0, "Latenza media troppo alta");
}