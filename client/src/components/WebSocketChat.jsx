import React, { useState, useEffect, useRef, useMemo } from 'react';
import { Container, Row, Col, Card, Form, Button, ListGroup, Badge } from 'react-bootstrap';
import { useWebSocket } from '../contexts/WebSocketContext';
import { useLocation, useNavigate } from 'react-router-dom';

function Chat() {
  const [newMessage, setNewMessage] = useState('');
  const [hasEnteredChat, setHasEnteredChat] = useState(false);
  const messagesEndRef = useRef(null);
  const location = useLocation();
  const navigate = useNavigate();

  // Ottieni parametri della chat dalla navigazione
  const chatParams = location.state || {};
  const {
    chatType = 'private',
    members = [],
    targetUser = '',
    groupName = '',
    chatId = null
  } = chatParams;

  // Ottieni tutto dal WebSocket context
  const {
    connectedUsers,  //array di oggetti User
    messages,        //array di oggetti ChatMessage
    sendMessage,     //funzione di invio messaggio
    enterChat,       //funzione di ingresso chat
    leaveChat,       //funzione di uscita chat
    aloneInChatStatus,   //stato per notificare utente da solo in chat
    chatUsersCount,      //dettagli della chat
    chatDeclined,       //array di inviti rifutati
    chatLeftUsers,      //array di utenti usciti
    chatAbandonedStatus,   //stato di abbandono chat
    user                   //utente corrente
  } = useWebSocket();

  const inChatMembers = useMemo(() => {
    return connectedUsers
      .filter(member => member.chat_id === chatId)
      .map(member => member.username);
  }, [connectedUsers, chatId]);

  const notInChatMembers = useMemo(() => {
    return members.filter(member => !inChatMembers.includes(member));
  }, [members, inChatMembers]);

  //Ottieni dati tracking per la chat corrente
  const currentChatCount = useMemo(() => {
    return chatId ? chatUsersCount[chatId] : null;
  }, [chatUsersCount, chatId]);

  //Ottieni stato abbandono per la chat corrente
  const currentChatAbandoned = useMemo(() => {
    return chatId ? chatAbandonedStatus[chatId] : null;
  }, [chatAbandonedStatus, chatId]);

  // Entra automaticamente in chat quando il componente si monta
  useEffect(() => {
    if ((chatType === 'group' || chatType === 'private') && !hasEnteredChat) {
      enterChat(chatType, targetUser, members, chatId);
      setHasEnteredChat(true);
    }

    // Cleanup: esce dalla chat quando il componente si smonta
    return () => {
      if ((chatType === 'group' || chatType === 'private') && hasEnteredChat) {
        leaveChat();
      }
    };
  }, [chatType, targetUser, hasEnteredChat]);

  // Auto scroll to bottom when new message arrives
  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  const handleSendMessage = () => {
    if (sendMessage(newMessage, chatType, targetUser, inChatMembers, chatId)) {
      setNewMessage('');
    }
  };

  const handleLeaveChat = () => {
    // Esce dalla chat e rende l'utente disponibile
    leaveChat();

    // Naviga alla home
    navigate('/', { replace: true });
  };

  const getChatTitle = () => {
    switch (chatType) {
      case 'group':
        return `🦀 ${groupName || `Gruppo (${members.length} membri)`}`;
      case 'private':
        return `🦀 Chat con ${targetUser}`;
      default:
        return '🦀 Chat';
    }
  };

  const getChatSubtitle = () => {
    switch (chatType) {
      case 'group':
        return `Membri: ${members.join(', ')}`;
      case 'private':
        return `Chat privata con ${targetUser}`;
      default:
        return 'Chat';
    }
  };

  const formatTime = (timestamp) => {
    return new Date(timestamp).toLocaleTimeString('it-IT', {
      hour: '2-digit',
      minute: '2-digit'
    });
  };

  const getMessageStyle = (type) => {
    switch (type) {
      case 'own':
        return {
          background: 'linear-gradient(135deg, #C9462A, #ab7468ff)',
          color: '#fff',
          marginLeft: 'auto',
          borderRadius: '20px 20px 5px 20px',
          maxWidth: '75%',
        };
      case 'other':
        return {
          background: '#444',
          color: '#dfdfdf',
          marginRight: 'auto',
          borderRadius: '20px 20px 20px 5px',
          maxWidth: '75%',
          border: '1px solid #d1d3d5ff'
        };
      case 'system':
        return {
          background: 'linear-gradient(135deg, #ffecd2 0%, #fcb69f 100%)',
          color: '#8b4513',
          margin: '0 auto',
          borderRadius: '15px',
          maxWidth: '80%',
          textAlign: 'center',
          fontStyle: 'italic'
        };
      default:
        return {};
    }
  };

  // Controlla se l'utente è solo in questa chat
  const isAloneInChat = chatId && aloneInChatStatus[chatId];

  return (
    <Container fluid className="p-4">
      <Row className="h-100">
        {/* Chat Area */}
        <Col md={8} className="h-100">
          {/* Avviso quando l'utente è solo (solo se NON c'è abbandono definitivo) */}
          {isAloneInChat && !currentChatAbandoned && (
            <div className="alert alert-warning d-flex align-items-center m-3 mb-2" role="alert">
              <i className="bi bi-exclamation-triangle-fill me-2"></i>
              <div>
                <strong>Attenzione:</strong> Sei solo in questa chat. I tuoi messaggi non arriveranno a nessuno.
              </div>
            </div>
          )}

          {/* Avviso abbandono definitivo chat privata (priorità alta) */}
          {currentChatAbandoned && (
            <div className="alert alert-danger d-flex align-items-center justify-content-between m-3 mb-0" role="alert" style={{ backgroundColor: '#dc3545', borderColor: '#dc3545', color: 'white' }}>
              <div className="d-flex align-items-center">
                <i className="bi bi-exclamation-triangle-fill me-2"></i>
                <div>
                  <strong>Chat Abbandonata:</strong> {currentChatAbandoned.message}
                </div>
              </div>
              <Button
                variant="outline-light"
                size="sm"
                onClick={() => {
                  leaveChat();
                  navigate('/dashboard');
                }}
                style={{ marginLeft: '1rem' }}
              >
                Abbandona Chat
              </Button>
            </div>
          )}
          <Card style={{ height: '85vh', boxShadow: '0 8px 32px 0 rgba(31, 38, 135, 0.2)', borderRadius: '20px', overflow: 'hidden' }}>


            {/* Chat Header */}
            <Card.Header style={{
              background: '#292929',
              color: '#fff',
              borderRadius: '20px 20px 0 0',
              padding: '1rem 1.5rem'
            }}>
              <div className="d-flex justify-content-between align-items-center">
                <div>
                  <h4 className="mb-0">{getChatTitle()}</h4>
                  <small style={{ opacity: 0.9 }}>{getChatSubtitle()}</small>
                </div>
                <div className="d-flex align-items-center gap-3">
                  {(chatType === 'group' || chatType === 'private') && (
                    <Button
                      variant="outline-warning"
                      size="sm"
                      onClick={handleLeaveChat}
                      style={{ borderRadius: '20px' }}
                    >
                      Abbandona Chat
                    </Button>
                  )}
                </div>
              </div>
            </Card.Header>

            {/* Messages */}
            <Card.Body style={{
              overflowY: 'auto',
              background: '#fff',
              padding: '1.5rem'
            }}>
              {messages.length === 0 ? (
                <div style={{ textAlign: 'center', color: '#6c757d', marginTop: '2rem' }}>
                  <p>💬 Benvenuto nella chat! Inizia una conversazione...</p>
                </div>
              ) : (
                messages.map((message) => (
                  <div
                    key={message.id}
                    style={{
                      display: 'flex',
                      flexDirection: 'column',
                      marginBottom: '1rem',
                      animation: 'slideIn 0.3s ease-out'
                    }}
                  >
                    <div
                      style={{
                        ...getMessageStyle(message.type),
                        padding: '0.8rem 1.2rem',
                        wordWrap: 'break-word'
                      }}
                    >
                      {message.type !== 'own' && message.type !== 'system' && (
                        <small style={{ fontWeight: 'bold', marginBottom: '0.3rem', display: 'block', color: '#7ab066ff' }}>
                          {message.sender}
                        </small>
                      )}
                      <div>{message.message}</div>
                      <small style={{
                        opacity: 0.7,
                        fontSize: '0.75rem',
                        marginTop: '0.3rem',
                        display: 'block',
                        textAlign: message.type === 'own' ? 'right' : 'left'
                      }}>
                        {formatTime(message.timestamp)}
                      </small>
                    </div>
                  </div>
                ))
              )}
              <div ref={messagesEndRef} />
            </Card.Body>

            {/* Message Input */}
            <Card.Footer style={{ background: '#292929', padding: '1rem 1.5rem', border: 'none' }}>
              <Form
                onSubmit={(e) => {
                  e.preventDefault();
                  handleSendMessage();
                }}
              >
                <div className="d-flex gap-2">
                  <Form.Control
                    type="text"
                    value={newMessage}
                    onChange={(e) => setNewMessage(e.target.value)}
                    placeholder="Scrivi un messaggio..."
                    style={{
                      borderRadius: '20px',
                      border: '2px solid #e9ecef',
                      padding: '0.7rem 1rem'
                    }}
                  />
                  <Button
                    type="submit"
                    disabled={!newMessage.trim()}
                    style={{
                      background: newMessage.trim()
                        ? 'linear-gradient(135deg, #C9462A, #b03d24)'
                        : '#555',
                      border: 'none',
                      borderRadius: '20px',
                      padding: '0.7rem 1.5rem',
                      fontWeight: '600',
                      transition: 'all 0.3s ease',
                      boxShadow: newMessage.trim()
                        ? '0 2px 8px rgba(201, 70, 42, 0.3)'
                        : 'none',
                      transform: newMessage.trim() ? 'scale(1.02)' : 'scale(1)'
                    }}
                  >
                    <i className="bi bi-arrow-right-circle-fill" style={{ fontSize: '1.1rem' }}></i>
                  </Button>
                </div>
              </Form>
            </Card.Footer>
          </Card>
        </Col>

        {/* Users Sidebar */}
        <Col md={4}>
          <Card style={{ height: '85vh', boxShadow: '0 8px 32px 0 rgba(31, 38, 135, 0.2)', borderRadius: '20px' }}>
            <Card.Header style={{
              background: '#292929',
              borderRadius: '20px 20px 0 0',
              padding: '1rem 1.5rem'
            }}>
              <h5 className="mb-0" style={{ color: '#fff' }}>
                <i className="bi bi-people-fill me-2" style={{ fontSize: '1.3rem', color: '#fff', verticalAlign: 'middle', marginRight: '1rem' }}></i>
                Membri del {chatType === 'group' ? 'Gruppo' : 'Chat'}
              </h5>
            </Card.Header>
            <Card.Body style={{ padding: '1rem', overflowY: 'auto' }}>
              {/* Sezione Utenti in Chat */}
              <div className="mb-4">
                <div className="d-flex align-items-center gap-2 mb-3">
                  <Badge bg="success" style={{ fontSize: '0.8rem', padding: '0.5rem 0.8rem' }}>
                    {currentChatCount ? (
                      `In Chat (${currentChatCount.in_chat_count}/${currentChatCount.invited_count})`
                    ) : (
                      `In Chat (${inChatMembers.length})`
                    )}
                  </Badge>
                  {currentChatCount && (
                    <Badge bg="info" style={{ fontSize: '0.7rem', padding: '0.3rem 0.6rem' }}>
                      📊 Invitati: {currentChatCount.invited_count}
                    </Badge>
                  )}
                </div>
                {(currentChatCount ? currentChatCount.users_in_chat.length === 0 : inChatMembers.length === 0) ? (
                  <p style={{ color: '#6c757d', fontSize: '0.9rem', fontStyle: 'italic' }}>
                    Nessun membro attualmente in chat
                  </p>
                ) : (
                  <div>
                    {(currentChatCount ? currentChatCount.users_in_chat : inChatMembers).map((memberName, index) => (
                      <div
                        key={`in-chat-${index}`}
                        style={{
                          padding: '0.6rem 0.8rem',
                          marginBottom: '0.3rem',
                          backgroundColor: '#e2ffe2ff',
                          borderRadius: '10px',
                          borderLeft: '4px solid #28a745'
                        }}
                      >
                        <div className="d-flex align-items-center gap-2">
                          <div
                            style={{
                              width: '8px',
                              height: '8px',
                              borderRadius: '50%',
                              backgroundColor: '#28a745'
                            }}
                          />
                          <span style={{ fontWeight: memberName === user.username ? 'bold' : 'normal' }}>
                            {memberName}
                            {memberName === user.username && ' (tu)'}
                          </span>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>

              {/* Sezione utenti non in chat: separa in 'In attesa' e 'Non disponibili' */}
              {(() => {
                const invited = currentChatCount ? currentChatCount.invited_users : members;
                const inChat = currentChatCount ? currentChatCount.users_in_chat : inChatMembers;
                const notInChat = invited.filter(u => !inChat.includes(u));

                const onlineUsernames = new Set(connectedUsers.map(u => u.username));
                // chatDeclined/left arrivano dal context
                const declinedSet = new Set((chatDeclined || [])
                  .filter(d => (d.chat_id === chatId || !d.chat_id))
                  .map(d => d.responding_user));
                const leftSet = new Set((chatLeftUsers && chatLeftUsers[chatId]) || []);

                const pending = notInChat.filter(u => onlineUsernames.has(u) && !declinedSet.has(u) && !leftSet.has(u));
                const unavailable = notInChat.filter(u => !pending.includes(u));

                return (pending.length > 0 || unavailable.length > 0) && (
                  <div>
                    <div className="d-flex align-items-center gap-2 mb-3">
                      <Badge bg="secondary" style={{ fontSize: '0.8rem', padding: '0.5rem 0.8rem' }}>
                        Non in Chat ({notInChat.length})
                      </Badge>
                    </div>

                    {pending.length > 0 && (
                      <div className="mb-3">
                        <div className="d-flex align-items-center gap-2 mb-2">
                          <Badge bg="warning" text="dark" style={{ fontSize: '0.75rem' }}>In attesa</Badge>
                          <small style={{ color: '#6c757d' }}>Connessi, non sono ancora entrati</small>
                        </div>
                        {pending.map((memberName, index) => (
                          <div key={`pending-${index}`} style={{ padding: '0.6rem 0.8rem', marginBottom: '0.3rem', backgroundColor: '#fffbe6', borderRadius: '10px', borderLeft: '4px solid #ffc107' }}>
                            <span>{memberName}</span>
                          </div>
                        ))}
                      </div>
                    )}

                    {unavailable.length > 0 && (
                      <div>
                        <div className="d-flex align-items-center gap-2 mb-2">
                          <Badge bg="secondary" style={{ fontSize: '0.75rem' }}>Non disponibili</Badge>
                          <small style={{ color: '#6c757d' }}>Hanno rifiutato, sono usciti o offline</small>
                        </div>
                        {unavailable.map((memberName, index) => (
                          <div key={`unavailable-${index}`} style={{ padding: '0.6rem 0.8rem', marginBottom: '0.3rem', backgroundColor: '#f8f9fa', borderRadius: '10px', borderLeft: '4px solid #6c757d' }}>
                            <span>{memberName}
                              {declinedSet.has(memberName) && <span style={{ color: '#c53030', marginLeft: 8 }}>(rifiutato)</span>}
                              {leftSet.has(memberName) && !declinedSet.has(memberName) && <span style={{ color: '#6c757d', marginLeft: 8 }}>(uscito)</span>}
                              {!onlineUsernames.has(memberName) && !declinedSet.has(memberName) && !leftSet.has(memberName) && <span style={{ color: '#6c757d', marginLeft: 8 }}>(offline)</span>}
                            </span>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                );
              })()}

              {/* Messaggio se non ci sono membri */}
              {members.length === 0 && (
                <div style={{ textAlign: 'center', color: '#6c757d', marginTop: '2rem' }}>
                  <p>Nessun membro nella chat</p>
                </div>
              )}
            </Card.Body>
          </Card>
        </Col>
      </Row>

      <style jsx>{`
        @keyframes slideIn {
          from {
            opacity: 0;
            transform: translateY(20px);
          }
          to {
            opacity: 1;
            transform: translateY(0);
          }
        }
      `}</style>
    </Container>
  );
}

export default Chat;
