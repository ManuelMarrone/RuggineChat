import React, { createContext, useContext, useEffect, useRef, useState } from 'react';

const WebSocketContext = createContext();

// Contex: Singleton WebSocket ed Event Hub centralizzato per state sync real-time
export const useWebSocket = () => {
  const context = useContext(WebSocketContext);
  if (!context) {
    throw new Error('useWebSocket must be used within a WebSocketProvider');
  }
  return context;
};

export const WebSocketProvider = ({ children, user, onLogout }) => {
  const [isConnected, setIsConnected] = useState(false);
  const [connectedUsers, setConnectedUsers] = useState([]);
  const [messages, setMessages] = useState([]);
  const [loginError, setLoginError] = useState('');
  const [chatInvites, setChatInvites] = useState([]);
  const [chatReady, setChatReady] = useState([]); // stato per chat pronte
  const [chatDeclined, setChatDeclined] = useState([]); // notifiche inviti rifiutati
  const [aloneInChatStatus, setAloneInChatStatus] = useState({}); // chat_id -> boolean
  const [chatUsersCount, setChatUsersCount] = useState({}); // chat_id -> ChatUsersCount
  const [chatAbandonedStatus, setChatAbandonedStatus] = useState({}); //  chat_id -> ChatAbandonedNotification
  const [chatLeftUsers, setChatLeftUsers] = useState({}); //chat_id -> utenti che sono usciti dalla chat
  const [lastUsersUpdate, setLastUsersUpdate] = useState(new Date()); //Stato aggiornamento
  const wsRef = useRef(null);
  const chatStateRef = useRef({ inChat: false, chatType: '', members: [], chatId: null });
  const leavingChatRef = useRef(false); // Previene chiamate multiple di leaveChat
  const sessionIdRef = useRef(''); // Sessione utente
  // Pulisci lo stato legato alla sessione quando cambia utente
  useEffect(() => {
    // Reset completo delle notifiche e stati chat per evitare "bleed" tra utenti
    setMessages([]);
    setChatInvites([]);
    setChatReady([]);
    setChatDeclined([]);
    setAloneInChatStatus({});
    setChatUsersCount({});
    setChatAbandonedStatus({});
    setChatLeftUsers({});
  }, [user?.username]);

  useEffect(() => {
    if (!user?.username) {
      return;
    }

    const connectWebSocket = () => {
      try {
        const ws = new WebSocket('ws://localhost:3000/ws');
        wsRef.current = ws;

        //si attiva quando la connesione ws viene stabilita
        ws.onopen = () => {
          setIsConnected(true);

          // Invia immediatamente il messaggio di login
          const loginMessage = {
            message_type: 'Login',
            data: JSON.stringify({ username: user.username })
          };
          ws.send(JSON.stringify(loginMessage));
        };

        //si attiva ogni volta che il server invia un messaggio al client
        ws.onmessage = (event) => {
          try {
            const wsMessage = JSON.parse(event.data);

            switch (wsMessage.message_type) {
              case 'LoginSuccess': {
                // Estrai session_id dalla risposta del backend
                if (typeof wsMessage.data === 'string' && wsMessage.data.includes('session_id')) {
                  const match = wsMessage.data.match(/session_id: ([a-f0-9\-]+)/);
                  if (match) {
                    sessionIdRef.current = match[1];
                  }
                }
                break;
              }
                setLoginError('');
                break;

              case 'LoginError':
                setLoginError(wsMessage.data);
                if (wsMessage.data.includes('già in uso') || wsMessage.data.includes('Username taken')) {
                  setTimeout(() => {
                    handleLogout();
                  }, 3000);
                }
                break;

              case 'ChatMessage':
                const chatMsg = JSON.parse(wsMessage.data);

                // Semplice filtro: mostra tutti i messaggi quando siamo in chat
                setMessages(prev => [...prev, {
                  id: chatMsg.id,
                  chat_id: chatMsg.chat_id,
                  sender: chatMsg.username,
                  message: chatMsg.content,
                  timestamp: new Date(chatMsg.timestamp),
                  type: chatMsg.username === 'Sistema' ? 'system' :
                    chatMsg.username === user.username ? 'own' : 'other'
                }]);
                break;

              case 'ChatInvite':
                const invite = JSON.parse(wsMessage.data);
                setChatInvites(prev => [...prev, invite]);
                break;

              case 'ChatInviteResponse':
                const response = JSON.parse(wsMessage.data);
                if (response.accepted) {
                } else {
                  // Notifica all'invitante che l'utente ha rifiutato
                  // Payload atteso: ChatInviteResponseNotify
                  setChatDeclined(prev => [...prev, response]);
                }
                break;

              case 'ChatReady':
                const chatReadyData = JSON.parse(wsMessage.data);
                setChatReady(prev => [...prev, chatReadyData]);
                break;

              case 'AloneInChat':
                const aloneData = JSON.parse(wsMessage.data);
                setAloneInChatStatus(prev => ({
                  ...prev,
                  [aloneData.chat_id]: aloneData.is_alone
                }));

                break;

              case 'ChatUsersCount':
                const countData = JSON.parse(wsMessage.data);
                setChatUsersCount(prev => {
                  const prevInChat = prev[countData.chat_id]?.users_in_chat || [];
                  const removed = prevInChat.filter(u => !countData.users_in_chat.includes(u));
                  if (removed.length > 0) {
                    setChatLeftUsers(prevLeft => {
                      const existing = new Set(prevLeft[countData.chat_id] || []);
                      removed.forEach(u => existing.add(u));
                      return { ...prevLeft, [countData.chat_id]: Array.from(existing) };
                    });
                  }
                  return {
                    ...prev,
                    [countData.chat_id]: countData
                  };
                });
                break;

              case 'ChatAbandoned':
                const abandonedData = JSON.parse(wsMessage.data);
                setChatAbandonedStatus(prev => ({
                  ...prev,
                  [abandonedData.chat_id]: abandonedData
                }));
                break;

              case 'ChatInvalidated':
                const invalidatedData = JSON.parse(wsMessage.data);
                // Rimuovi tutte le notifiche ChatReady per questo chat_id
                setChatReady(prev =>
                  prev.filter(notification => notification.chat_id !== invalidatedData.chat_id)
                );
                break;

              case 'UserJoined':
                const joinedUser = JSON.parse(wsMessage.data);

                setConnectedUsers(prev => {
                  const exists = prev.some(u => u.username === joinedUser.username);
                  if (!exists) {
                    const updated = [...prev, joinedUser];
                    setLastUsersUpdate(new Date()); //Aggiorna timestamp
                    return updated;
                  }
                  return prev;
                });

                break;

              case 'UserLeft':
                const leftUsername = wsMessage.data;

                setConnectedUsers(prev => {
                  const updated = prev.filter(u => u.username !== leftUsername);
                  setLastUsersUpdate(new Date()); //Aggiorna timestamp
                  return updated;
                });
                break;

              case 'UserStatusChanged':
                const userStatusData = JSON.parse(wsMessage.data);

                setConnectedUsers(prev => {
                  const updated = prev.map(u =>
                    u.username === userStatusData.username
                      ? {
                        ...u, is_available: userStatusData.is_available,
                        chat_id: userStatusData.is_available ? null : userStatusData.chat_id
                      }
                      : u
                  );
                  setLastUsersUpdate(new Date()); //Aggiorna timestamp
                  return updated;
                });
                break;

              case 'UsersList':
                const usersList = JSON.parse(wsMessage.data);

                setConnectedUsers(usersList);
                setLastUsersUpdate(new Date()); //Aggiorna timestamp
                break;

              case 'Error':
                break;

              default:
            }
          } catch (error) {
          }
        };

        //si attiva quando la connessione ws viene chiusa
        ws.onclose = () => {
          setIsConnected(false);
          setConnectedUsers([]);
          setLastUsersUpdate(new Date()); //Aggiorna timestamp
        };

        ws.onerror = (error) => {
          setIsConnected(false);
        };
      } catch (error) {
      }
    };

    connectWebSocket();

    return () => {
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
    };
  }, [user?.username]);

  const handleLogout = () => {
    if (wsRef.current) {
      // Chiudi la connessione WebSocket - il backend gestirà automaticamente
      // la rimozione dell'utente e invierà UserLeft agli altri client
      wsRef.current.close();
      wsRef.current = null;
    }
    setIsConnected(false);
    // Pulisci tutti gli stati locali per evitare che le notifiche del vecchio utente persistano
    setConnectedUsers([]);
    setMessages([]);
    setLoginError('');
    setChatInvites([]);
    setChatReady([]);
    setChatDeclined([]);
    setAloneInChatStatus({});
    setChatUsersCount({});
    setChatAbandonedStatus({});
    setChatLeftUsers({});
    onLogout();
  };

  const sendMessage = (content, chatType = 'private', targetUser = '', members = [], chatId = null) => {
    if (!wsRef.current || !isConnected || !content.trim()) return false;

    let chat_type_obj;

    switch (chatType) {
      case 'private':
        if (!targetUser) {
          return false;
        }
        chat_type_obj = { Private: { target: targetUser } };
        break;
      case 'group':
        if (!members || members.length === 0) {
          return false;
        }
        chat_type_obj = { Group: { members: members } };
        break;
      default:
        return false;
    }

    const chatMessage = {
      id: crypto.randomUUID(),
      chat_id: chatId,
      username: user.username,
      content: content.trim(),
      timestamp: new Date().toISOString(),
      chat_type: chat_type_obj
    };

    const wsMessage = {
      message_type: 'ChatMessage',
      data: JSON.stringify(chatMessage)
    };

    try {
      wsRef.current.send(JSON.stringify(wsMessage));

      // RIMOZIONE DUPLICATI: Non aggiungiamo il messaggio localmente
      // Il messaggio arriverà dal backend e verrà aggiunto tramite il listener WebSocket

      return true;
    } catch (error) {
      return false;
    }
  };

  const sendRawMessage = (messageObject) => {
    if (!wsRef.current || !isConnected) return false;

    try {
      wsRef.current.send(JSON.stringify(messageObject));
      return true;
    } catch (error) {
      return false;
    }
  };

  const enterChat = (chatType, targetUser = '', members = [], chatId) => {

    if (!chatId) {
      return;
    }

    if (!wsRef.current || !isConnected) {
      return;
    }

    // Reset messaggi per nuova chat
    setMessages([]);
    chatStateRef.current = { inChat: true, chatType, members, chatId };


    const message = {
      message_type: 'UserStatusChanged',
      data: JSON.stringify({
        available: false,
        inChat: true,
        chatType,
        targetUser,
        members,
        chatId
      })
    };

    try {
      wsRef.current.send(JSON.stringify(message));

      // Aggiorna immediatamente lo stato locale
      setConnectedUsers(prev =>
        prev.map(u =>
          u.username === user.username
            ? { ...u, is_available: false, chat_id: chatId }
            : u
        )
      );
      setLastUsersUpdate(new Date()); // Aggiorna timestamp
      return chatId;

    } catch (error) {
    }
  };

  const leaveChat = () => {

    if (leavingChatRef.current || !chatStateRef.current.inChat) {
      return;
    }

    leavingChatRef.current = true;

    if (!wsRef.current || !isConnected || wsRef.current.readyState !== WebSocket.OPEN) {
      chatStateRef.current = { inChat: false, chatType: '', members: [], chatId: null };

      // Aggiorna stato locale immediatamente
      setConnectedUsers(prev =>
        prev.map(u =>
          u.username === user.username
            ? { ...u, is_available: true, chat_id: null }
            : u
        )
      );

      // Fallback: usa API HTTP per aggiornare lo stato nel backend
      import('../API/API.mjs').then(({ updateUserAvailability }) => {
        updateUserAvailability(user.username, true)
          .catch(err => console.error('Errore aggiornamento API HTTP:', err));
      });

      leavingChatRef.current = false;
      return;
    }

    // Salva i membri della chat prima di resettare lo stato
    const currentMembers = chatStateRef.current.members || [];
    const currentChatType = chatStateRef.current.chatType;
    const leavingChatId = chatStateRef.current.chatId;

    // Invia messaggio di abbandono chat a tutti i membri
    const leaveMessage = {
      message_type: 'ChatMessage',
      data: JSON.stringify({
        id: crypto.randomUUID(),
        username: 'Sistema',
        content: `${user.username} ha abbandonato la chat`,
        timestamp: new Date().toISOString(),
        chat_type: currentChatType === 'private'
          ? { Private: { target: currentMembers.find(m => m !== user.username) || '' } }
          : { Group: { members: currentMembers } }
      })
    };

    // Invia messaggio di status per aggiornare disponibilità
    const statusMessage = {
      message_type: 'UserStatusChanged',
      data: JSON.stringify({
        available: true,
        inChat: false,
        members: currentMembers,
        userLeft: user.username,
        chatId: leavingChatId
      })
    };

    try {
      // Invia prima il messaggio di abbandono
      wsRef.current.send(JSON.stringify(leaveMessage));

      // Poi aggiorna lo status
      wsRef.current.send(JSON.stringify(statusMessage));

      // Aggiorna immediatamente lo stato locale dell'utente corrente
      setConnectedUsers(prev =>
        prev.map(u =>
          u.username === user.username
            ? { ...u, is_available: true, chat_id: null }
            : u
        )
      );

    } catch (error) {
      console.error('Errore invio messaggio leaveChat:', error);
    } finally {
      // Reset dello stato e del guard sempre
      chatStateRef.current = { inChat: false, chatType: '', members: [], chatId: null };
      leavingChatRef.current = false;
    }
  };

  const sendChatInvite = (chatType, targetUser = '', members = [], message = '') => {
    if (!wsRef.current || !isConnected) return false;

    const chatId = crypto.randomUUID();

    const invite = {
      id: crypto.randomUUID(),
      chat_id: chatId,
      from: user.username,
      from_session_id: sessionIdRef.current,
      chat_type: chatType === 'private'
        ? { Private: { target: targetUser } }
        : { Group: { members: members } },
      message: message || `${user.username} ti ha invitato in una chat ${chatType === 'private' ? 'privata' : 'di gruppo'}`,
      timestamp: new Date().toISOString()
    };

    const wsMessage = {
      message_type: 'ChatInvite',
      data: JSON.stringify(invite)
    };

    try {
      wsRef.current.send(JSON.stringify(wsMessage));
      return { success: true, chatId: chatId };
    } catch (error) {
      console.error('Errore invio invito:', error);
      return { success: false, chatId: null };
    }
  };

  const respondToChatInvite = (inviteId, accepted, fromUser, chatType, chatId, fromSessionId) => {
    if (!wsRef.current || !isConnected) return false;

    const response = {
      invite_id: inviteId,
      accepted: accepted,
      from_user: fromUser,
      from_session_id: fromSessionId, // Usa il session_id dell'invitante originale
      chat_type: chatType,
      chat_id: chatId
    };

    const wsMessage = {
      message_type: 'ChatInviteResponse',
      data: JSON.stringify(response)
    };

    try {
      wsRef.current.send(JSON.stringify(wsMessage));

      // Rimuovi l'invito dalla lista
      setChatInvites(prev => prev.filter(invite => invite.id !== inviteId));

      return true;
    } catch (error) {
      console.error('Errore risposta invito:', error);
      return false;
    }
  };

  const removeChatReady = (chatId) => {
    setChatReady(prev => prev.filter(ready => ready.chat_id !== chatId));
  };
  const removeChatDeclined = (idOrChatId) => {
    setChatDeclined(prev => prev.filter(item => (item.invite_id !== idOrChatId && item.chat_id !== idOrChatId)));
  };

  const clearAllNotifications = () => {
    setChatInvites([]);
    setChatReady([]);
    setChatDeclined([]);
  };

  const contextValue = {
    isConnected,
    connectedUsers,
    messages,
    loginError,
    chatInvites,
    chatReady, //notifiche chat pronte
    chatDeclined, //notifiche inviti rifiutati
    aloneInChatStatus, // stato solitudine chat
    chatUsersCount, // conteggio utenti per chat
    chatLeftUsers, // utenti che hanno lasciato la chat
    chatAbandonedStatus, // stato abbandono definitivo chat
    lastUsersUpdate, // timestamp ultimo aggiornamento
    sendMessage,
    sendRawMessage,
    enterChat,
    leaveChat,
    sendChatInvite,
    respondToChatInvite,
    removeChatReady, // funzione per rimuovere notifiche
    removeChatDeclined, //rimuovi notifica rifiuto
    clearAllNotifications,
    handleLogout,
    user
  };

  return (
    <WebSocketContext.Provider value={contextValue}>
      {children}
    </WebSocketContext.Provider>
  );
};
