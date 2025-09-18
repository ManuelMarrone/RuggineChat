# Ruggine: App di Chat Testuale

## Introduzione

Il presente documento descrive il **Progetto 2.1 – Ruggine: App di Chat Testuale**, sviluppato per il corso di **Programmazione di sistema**.  
L’obiettivo del progetto è la realizzazione di un’applicazione client/server per la gestione di una chat testuale, con funzionalità di iscrizione, creazione e gestione di gruppi, e monitoraggio delle prestazioni del server.  

L’applicazione, chiamata **Ruggine Chat**, è stata progettata con particolare attenzione all’efficienza in termini di consumo di CPU e dimensioni dell’eseguibile, garantendo inoltre portabilità su più piattaforme.  

### Partecipanti al gruppo
- **Manuel Marrone**  
- **Edoardo Cecchini**  
- **Antonio Ceglia**  
- **Antonino Labate**  

La documentazione prodotta è articolata in più sezioni, ciascuna finalizzata a descrivere il progetto da punti di vista differenti: utilizzo, progettazione e verifica tramite test.  

---

## Indice

1. [Manuale Utente](#manuale-utente)  
2. [Manuale del Progettista](#manuale-del-progettista)  
3. [Documentazione e Strategia dei Test](#documentazione-e-strategia-dei-test)

---

## Manuale Utente – Ruggine Chat

### 1. Introduzione

L’applicazione **Ruggine Chat** è una piattaforma di chat testuale progettata per lo scambio di messaggi all’interno di un’organizzazione, come ad esempio un’azienda o un’università.  

Il sistema consente agli utenti di:
- Creare conversazioni individuali o di gruppo  
- Inviare e ricevere messaggi in tempo reale  

#### Requisiti minimi

**Sistema operativo**  
- Windows 10 o successivo  
- macOS 12 o successivo   

**Browser supportati**  
- Chrome, Firefox, Edge, Safari (versioni aggiornate)  

**Connessione di rete**  
- Rete locale aziendale/universitaria/casalinga  

**Requisiti software**  
- Node.js: ≥ 18.18.0 (alcune dipendenze ESLint e Vite lo richiedono)  
- npm: ≥ 9.x (incluso con Node 18)  
- Rust: ≥ 1.63 (consigliato installare la versione LTS più recente, es. Rust 1.80+, tramite *rustup*)  

---

### 2. Installazione

#### 2.1 Download e Setup
- Scaricare l’applicazione dal repository ufficiale: [https://github.com/PdS2425-C2/G53.git](https://github.com/PdS2425-C2/G53.git)  
- Se fornita come archivio `.zip` o `.tar.gz`, estrarre il pacchetto  
- Il progetto è organizzato in due cartelle principali:  
  - `client/` → applicazione client (React + Vite)  
  - `src/` → server API (Rust + Axum)  

#### 2.2 Installazione dipendenze

**Frontend**  
```bash
cd client
npm install
```

**Backend**  
Non richiede installazioni manuali: Cargo scaricherà automaticamente le dipendenze indicate in `Cargo.toml` al primo `cargo run`.

#### 2.3 Avvio dell’applicazione

**Modalità sviluppo**

_Backend:_  
```bash
cd src
cargo run
```

_Frontend:_  
```bash
cd client
npm run dev
```

Il frontend sarà accessibile su [http://localhost:5173](http://localhost:5173) (porta Vite predefinita) e comunicherà con il backend su [http://localhost:3000](http://localhost:3000).  

### 3. Utilizzo

#### 3.1 Avvio e accesso
1. Avviare sia backend che frontend come descritto nella sezione Installazione.  
2. Aprire il browser e collegarsi a:  
   - [http://localhost:5173](http://localhost:5173) (sviluppo)  
3. Inserire un **username univoco** (non sono ammessi duplicati).  

#### 3.2 Navigazione principale

- **Home**  
  Panoramica delle funzioni principali: utenti attivi, notifiche ricevute, inviti, creazione chat.  

- **Utenti Attivi**  
  Lista degli utenti attualmente connessi.  
  Ogni utente ha lo stato: *Disponibile* o *Occupato*.  

- **Chat di Gruppo**  
  1. Cliccare sul pulsante dedicato.  
  2. Selezionare almeno 2 utenti e inviare un invito.  
  3. Se uno degli invitati accetta, il creatore riceve una notifica.  
  4. Tramite la notifica si accede alla chat di gruppo.  

- **Chat Privata**  
  1. Cliccare sul pulsante dedicato.  
  2. Selezionare un utente e inviare un invito.  
  3. Se l’invitato accetta, il creatore riceve una notifica.  
  4. Tramite la notifica si accede alla chat privata.  

- **Logout**  
  Al click sul pulsante *Logout*:  
  - L’utente si disconnette da eventuali chat attive.
  - L’account viene eliminato dalla sessione corrente. 
  - Non risulta più nella lista utenti connessi.

#### 3.3 Flusso di utilizzo tipico
1. Avviare l’applicazione (frontend + backend)  
2. Accedere con un username univoco  
3. Consultare la lista utenti attivi  
4. Avviare una chat privata o di gruppo  
5. Scambiare messaggi in tempo reale  
6. Abbandonare la chat al termine della conversazione  
7. Effettuare il logout oppure avviare una nuova chat  

---

### 4. Risoluzione Problemi Comuni (FAQ)

- **L’app non si avvia** → Controllare che sia frontend che backend siano avviati correttamente  
- **Non vedo altri utenti online** → Verificare che ci siano effettivamente altre sessioni attive collegate  
- **Non ricevo notifiche di invito** → Controllare che il backend sia in esecuzione (WebSocket attivo)  
- **Errore di accesso** → Verificare di aver scelto un username non già in uso  
- **Connessione persa** → Controllare la rete o eventuali firewall che bloccano la comunicazione tra frontend e backend  


---

## Manuale del Progettista

### 1. Architettura del sistema

#### Schema generale

```
┌─────────────────────┐                    ┌─────────────────────┐
│                     │                    │                     │
│     FRONTEND        │                    │      BACKEND        │
│                     │                    │                     │
│  React + Vite       │◄──────────────────►│   Rust + Axum       │
│                     │   WebSocket        │                     │
│  ┌───────────────┐  │   HTTP REST        │  ┌───────────────┐  │
│  │ WebSocketChat │  │                    │  │ WebSocket     │  │
│  │ ChatInvites   │  │                    │  │ Handler       │  │
│  │ ActiveUsers   │  │                    │  │               │  │
│  │ LoginForm     │  │                    │  │ Chat Manager  │  │
│  └───────────────┘  │                    │  │               │  │
│  ┌───────────────┐  │                    │  │ User Manager  │  │
│  │ WebSocket     │  │                    │  │               │  │
│  │ Context       │  │                    │  │ State Manager │  │
│  └───────────────┘  │                    │  └───────────────┘  │
│                     │                    │                     │
│  Port: 5173         │                    │  Port: 3000         │
└─────────────────────┘                    └─────────────────────┘
```

#### Tecnologie usate:

- Backend: Rust con Axum framework
- Frontend: React con Vite
- Comunicazione: WebSocket + HTTP REST API
- Serializzazione: JSON (serde)
- CORS: Abilitato per sviluppo cross-origin

### 2. Struttura dei componenti

#### Frontend (cartella client/)

- `package.json`: definisce le dipendenze NPM necessarie al progetto.
- `vite.config.js`: contiene la configurazione di Vite per il build e lo sviluppo.

##### Cartella src
- `App.jsx`: rappresenta il componente root dell’app React.
- `main.jsx`: è l’entry point che inizializza l’applicazione.
- `index.css`: raccoglie gli stili globali.

##### Cartella API
- `API.mjs`: file che raccoglie le funzioni per la comunicazione REST con il backend.

##### Cartella components
- `Layout.jsx`: definisce il layout principale dell’applicazione.
- `Navbar.jsx`: gestisce la barra di navigazione.
- `Home.jsx`: funge da dashboard per la creazione di nuove chat.
  - Funzionalità: selezione utenti disponibili, creazione chat private o di gruppo, invio inviti, interfaccia intuitiva per la scelta della modalità.
- `LoginForm.jsx`: implementa il form di autenticazione.
- `ActiveUsers.jsx`: permette il monitoraggio degli utenti connessi.
  - Funzionalità: lista utenti online in tempo reale, indicatori di stato (disponibile/occupato), statistiche sulle connessioni, ordinamento con l’utente corrente in primo piano.
- `ChatInvites.jsx`: gestisce il sistema di inviti alle chat.
  - Funzionalità: ricezione e gestione inviti, accettazione/rifiuto, notifiche chat pronte, navigazione automatica alla chat.
- `WebSocketChat.jsx`: rappresenta l’interfaccia principale per le conversazioni.
  - Funzionalità: visualizzazione messaggi in tempo reale, invio messaggi, gestione chat private e di gruppo, tracking utenti in chat, notifiche di abbandono, indicatori “solo in chat”.

##### Cartella contexts
- `WebSocketContext.jsx`: definisce il context per la gestione dei WebSocket, permettendo di condividere lo stato e le funzioni tra i componenti dell’app.

#### Backend (cartella src/)

- `main.rs`: Entry Point e orchestrazione
  - Inizializza e avvia il server Axum
  - Configura le policy CORS per il frontend React
  - Definisce tutti gli endpoint REST API (es. /api/login, /api/users)
  - Configura l’endpoint WebSocket /ws
  - Inizializza il sistema di tracking CPU
  - Crea e distribuisce lo stato condiviso dell’applicazione

- `state.rs`: Gestione dello stato condiviso
  - Strutture principali: ConnectedUser, AppState
  - Gestisce utenti connessi, tracking CPU, inviti in attesa, conteggio utenti per chat attive, set delle chat private già avviate
  - Dati accessibili thread-safe tramite Arc<Mutex<>>

- `websocket.rs`: Comunicazioni real-time
  - Gestisce tutte le connessioni WebSocket e il routing dei messaggi
  - Funzioni principali: websocket_handler, handle_socket, smistamento messaggi per MessageType
  - Cleanup automatico alla disconnessione

- `chat.rs`: Logica dei messaggi
  - Gestisce invio e broadcasting dei messaggi tra utenti
  - Funzioni: broadcast_chat_message, broadcast_user_left
  - Filtering intelligente basato su chat_id, integrazione monitoraggio CPU

- `user.rs`: Gestione utenti e broadcasting generale
  - Funzioni: broadcast_user_joined, broadcast_user_status_changed, send_users_list, broadcast_to_all

- `invites.rs`: Sistema inviti chat
  - Funzioni: send_chat_invite, handle_invite_response
  - Routing intelligente per inviti privati o di gruppo, gestione session ID e notifiche “chat ready”

- `tracking.rs`: Monitoraggio chat e utenti
  - Funzioni: init_chat_tracking, add_user_to_chat_tracking, remove_user_from_chat_tracking, check_and_notify_alone_in_chat

- `notifications.rs`: Sistema notifiche
  - Funzioni: invalidate_chat_ready_notifications

- `routes.rs`: Endpoint HTTP REST
  - GET /: health check
  - GET /api/users: lista utenti connessi
  - POST /api/login: validazione username
  - POST /api/users/:username/availability: aggiornamento disponibilità

- `performance.rs`: Monitoraggio performance
  - Funzione update_cpu_time

- `cpu_log.rs`: Logging performance
  - Salvataggio asincrono su file ogni 2 minuti

- `types.rs`: Definizioni tipi e strutture
  - User, ChatMessage, ChatInvite, ChatInviteResponse, WebSocketMessage, MessageType

### 3. Flusso dei dati

#### Autenticazione
```
Client                               Server
------                               ------
|                                      |
|--- WebSocket Connect ---------------->|
|                                      |
|--- MessageType::Login --------------->|
|    {username: "alice"}              |
|                                      |
|                                      |-- Verifica unicità username
|                                      |-- Genera session_id univoco
|<-- LoginSuccess/LoginError -----------|
|    {session_id: "uuid-123"}         |
|<-- UserJoined broadcast -------------|
|<-- UsersList aggiornata -------------|
```

#### Meccanismo di Validazione
- Controllo Duplicati: server verifica unicità username
- Generazione Session ID: UUID univoco per ogni connessione
- Registrazione Utente: memorizzato in AppState
- Broadcast: notifica a tutti gli utenti connessi

#### Lista Utenti Attivi
- Stato Globale Condiviso: AppState
- Eventi che Aggiornano la Lista: UserJoined, UserLeft, UserStatusChanged

#### Creazione Chat
```
Inviter (Alice)         Server              Invitee (Bob)
       |                   |                      |
       |--- ChatInvite ---->|                      |
       |   {target: "bob", chat_type: Private}|
       |                   |--- ChatInvite ------>|
       |                   |                      |
       |                   |<-- ChatInviteResponse|
       |                   |    {accepted: true}  |
       |<-- ChatReady ------|                      |
       |   {chat_id, inviter, accepted_by}        |
       |                   |--- Sistema message -->|
       |<-- Sistema message|    "Bob è entrato"   |
```

#### Gestione Messaggi Chat
```
Sender                  Server                  Receiver
  |                       |                       |
  |--- ChatMessage ------>|                       |
  |                       |--- Broadcast -------->|
  |<-- Echo (if same chat)|                       |
```

#### Tipi di Chat Supportati
- Chat Privata: 2 utenti, tracking abbandono definitivo
- Chat di Gruppo: ≥3 utenti, gestione inviti multipli e abbandoni

### 4. Scelte progettuali e motivazioni

- Rust per backend: Zero-cost Abstractions, Memory Safety, Predictable Performance
- React + Vite per frontend: Startup rapido, HMR, build ottimizzate, architettura a componenti
- WebSocket per chat: comunicazione bidirezionale, messaggi istantanei, aggiornamenti presenza, gestione inviti e notifiche in tempo reale

### 5. Analisi e valutazione

#### Punti di forza
- Performance e leggerezza
- Architettura Real-Time efficiente
- Scalabilità e robustezza
- Semplicità d’uso e manutenibilità

#### Limiti attuali
- Persistenza e storage in memoria
- Autenticazione basica username-only
- Sicurezza e privacy non implementate

#### Possibili estensioni future
- Persistenza su database
- Autenticazione completa e profili utenti
- Sicurezza avanzata (end-to-end encryption, rate limiting)
- Performance testing per N utenti simultanei

#### Dimensioni e performance
- Backend Rust: 2,02 MB
- Frontend bundle Vite: 0,88 MB

##### -Tempi medi per invio messaggio singolo: 
- Latenza media: 10 Byte 162.23 µs
- Latenza media: 1000 Byte 291.38 µs
- Latenza media: 10000 Byte 1416.41 µs

#### -Statistiche di invio di messaggi in contemporanea:

###### Latenza con 10 messaggi contemporanei da 90 bytes (dimensione messaggio medio in italia):
- Latenza media: 3404.77 µs
- Latenza minima: 3115 µs
- Latenza massima: 4894 µs

###### Latenza con 100 messaggi contemporanei da 90 bytes(dimensione messaggio medio in italia):
- Latenza media: 590.438.18 ms
- Latenza minima: 575.902 ms
- Latenza massima: 656.374 ms

###### Latenza con 10 messaggi contemporanei da 1024 bytes:
- Latenza media: 5305.48 µs
- Latenza minima: 5051 µs
- Latenza massima: 5548 µs

###### Latenza con 100 messaggi contemporanei da 1024 bytes:
- Latenza media: 711.18778 ms
- Latenza minima: 699.307 ms
- Latenza massima: 752.172 ms



### 6. Conclusioni

#### 6.1 Sintesi delle Scelte Tecniche
- Stack Tecnologico: Rust + Axum + WebSocket (backend), React + Vite (frontend)
- Architettura del Sistema: stato condiviso thread-safe, sistema canali per messaging, protocollo WebSocket custom, tracking intelligente chat

#### 6.2 Considerazioni sul Processo di Sviluppo
- Approccio iterativo, milestone incrementali
- Divisione chiara responsabilità frontend/backend
- Pair programming e code review sistematiche

#### 6.3 Lezioni Apprese e Margini di Miglioramento
- Lezioni: Rust garantisce sicurezza, monitoraggio integrato facilita ottimizzazioni
- Miglioramenti: persistenza, scalabilità orizzontale, sicurezza, monitoring, implementazione CI/CD, rilevamento regressioni performance

#### 6.4 Conclusione Finale
Il progetto **Ruggine Chat** ha raggiunto tutti gli obiettivi prefissati, offrendo un sistema robusto, performante e maintainibile, con un team capace di affrontare contesti professionali reali.


---
## Documentazione e Strategia dei Test

### 1. Documentazione Test/Benchmark

Il progetto **Ruggine Chat** implementa test di integrazione end-to-end che verificano il funzionamento dei principali componenti del sistema in scenari realistici.

### Struttura dei test
- Avvio di un server Axum reale (porta effimera)  
- Connessione di client WebSocket multipli simulando il comportamento del frontend  
- Scambio di messaggi JSON tra client e server  
- Verifica della correttezza degli output ricevuti  

#### Strategia di testing
- Test di integrazione backend con server WebSocket in-process  
- Ogni test utilizza un server dedicato con stato isolato  
- Connessioni client simulate tramite `tokio-tungstenite`  
- Messaggi JSON inviati e ricevuti secondo protocollo di produzione  

#### Test case implementati
1. **test_duplicate_login_rejected**  
   - Obiettivo: verificare il rifiuto di login con username duplicato  
   - Scenario: due client tentano login con lo stesso username  
   - Verifica: il secondo client riceve `LoginError`  

2. **test_invite_delivery_private**  
   - Obiettivo: verificare l'invio e ricezione di inviti privati  
   - Scenario: Alice invia invito privato a Bob  
   - Verifica: Bob riceve l'invito con ID corretto  

3. **test_group_message_broadcast_to_all_members**  
   - Obiettivo: verificare il broadcast di messaggi di gruppo  
   - Scenario: Alice, Bob, Carol in gruppo; Alice invia messaggio  
   - Verifica: Bob e Carol ricevono il messaggio  

4. **test_multiple_chats_correct_deliver**  
   - Obiettivo: verificare l'isolamento tra chat diverse  
   - Scenario: due gruppi separati (Alice-Bob, Carol-Dave); Alice invia messaggio  
   - Verifica: Bob riceve il messaggio; Carol e Dave non ricevono il messaggio  

---

### 2. Funzioni Utility implementate a supporto del testing

#### async fn start_test_server()
- Inizializza un server Axum con stato isolato  
- Bind su porta 0 per evitare conflitti  
- Configurazione CORS minimale simulando il server di produzione  
- Ritorna URL WebSocket e handle del task server  

**Vantaggi:**  
- Isolamento: ogni test ha server dedicato  
- Realismo: server identico a quello reale  
- Port-free: nessun conflitto di porte tra test  

#### async fn connect_client(ws_url: &str) -> TestClient
- Stabilisce connessione WebSocket al server di test  
- Divide stream in lettura/scrittura per gestione asincrona  
- Crea canale tipizzato per messaggi deserializzati  
- Spawna task per conversione automatica di WsMessage in WebSocketMessage  

**Vantaggi:**  
- Tipizzazione: messaggi JSON automaticamente deserializzati  

#### async fn send_ws<T: serde::Serialize>(...)
- Serializza payload del messaggio in formato WebSocket standard  
- Wrappa in struttura `{ message_type, data }`  
- Invia tramite WebSocket come frame testuale  

**Vantaggi:**  
- Protocollo: formato identico a client di produzione  
- Semplicità: API semplificata per invio messaggi  

#### async fn recv_until<F>(...) -> Option<WebSocketMessage> where F: FnMut(&WebSocketMessage) -> bool
- Riceve messaggi asincroni con predicato di filtraggio  
- Timeout configurabile per prevenire deadlock  
- Consuma messaggi non corrispondenti (drop semantics)  
- Ritorna primo messaggio che soddisfa il predicato  

**Vantaggi:**  
- Predicati flessibili: closure per matching custom  
- Resource management: cleanup automatico messaggi  

#### async fn wait_user_in_chat(...)
- Attende conferma server che utente sia entrato in chat  
- Verifica messaggi `UserStatusChanged` con stato corretto  

**Vantaggi:**  
- Sincronizzazione: evita race conditions  

---

### 3. Test di integrazione backend (Rust)

#### Dove sono i test
- File: `tests/chat_flow.rs`  
- Tipologia: test di integrazione con server in-process (usano `create_app` della crate e WebSocket reali)  

I test avviano un server Axum in-process su porta effimera, si connettono via WebSocket e verificano i flussi scambiando messaggi JSON come farebbe il client.

#### Funzionalità testate
1. **Rifiuto del login con nome duplicato**  
   - Primo client: `Login` → atteso `LoginSuccess`  
   - Secondo client stesso username → atteso `LoginError`  

2. **Invio e ricezione di un invito privato**  
   - `alice` e `bob` fanno `Login`  
   - `alice` invia `ChatInvite` di tipo `Private { target: "bob" }`  
   - `bob` riceve `ChatInvite` con lo stesso `id`  

3. **Broadcast messaggio di gruppo**  
   - `alice`, `bob`, `carol` fanno `Login`  
   - Tutti inviano `UserStatusChanged` con `chatId = "group-1"`  
   - `alice` invia `ChatMessage` di tipo `Group` con `chat_id = group-1`  
   - `bob` e `carol` ricevono il messaggio  

#### Architettura dei test
- Avvio server in-process:  
  - Creazione `AppState` e `Router` con `fullstack_app::create_app`  
  - TcpListener su porta effimera, server servito con `axum::serve` in un task  
  - URL WS usato dai client: `ws://{addr}/ws`  

- Connessione e I/O WebSocket:  
  - Client usano `tokio-tungstenite` per aprire connessione e leggere/scrivere frame  
  - Helper:  
    - `connect_client(ws_url)` apre connessione e inoltra messaggi JSON  
    - `send_ws(mt, payload)` invia `WebSocketMessage { message_type, data }`  
    - `recv_until(pred, timeout)` attende messaggio che soddisfa predicato  
    - `wait_user_in_chat(u, chatId)` attende `UserStatusChanged` coerente  

#### Flussi di messaggi (schemi sintetici)
**Login duplicato**  
- C1 → Server: `Login { username: "mario" }`  
- Server → C1: `LoginSuccess`  
- C2 → Server: `Login { username: "mario" }`  
- Server → C2: `LoginError`  

**Invito privato**  
- A → Server: `Login { "alice" }`  
- B → Server: `Login { "bob" }`  
- A → Server: `ChatInvite { Private { target: "bob" }, id, chat_id, ... }`  
- Server → B: `ChatInvite { id, ... }`  

**Messaggio di gruppo**  
- A/B/C → Server: `Login`  
- A/B/C → Server: `UserStatusChanged { chatId: "group-1", inChat: true }`  
- A → Server: `ChatMessage { Group { members: [...] }, chat_id: "group-1" }`  
- Server → B,C: `ChatMessage { ... }`  

#### Librerie usate
- Runtime e I/O: `tokio`, `futures-util`  
- WebSocket client: `tokio-tungstenite` (dev-dependency)  
- Serializzazione/utility: `serde`, `serde_json`, `uuid`, `chrono`  
- Server: `axum`, `tower-http` (CORS)  

#### Esecuzione
Prerequisiti: Rust e Cargo installati  

Comandi (PowerShell):
```powershell
cargo test
cargo test --test chat_flow
```

#### Note
- I test avviano il server in-process su porta libera; non serve alcun server esterno  
- I timeout sono conservativi per stabilità dei test  
