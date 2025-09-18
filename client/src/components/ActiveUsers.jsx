import React, { useState, useEffect } from 'react';
import { Container, Row, Col, Card, ListGroup, Badge } from 'react-bootstrap';
import { useWebSocket } from '../contexts/WebSocketContext';
import ChatInvites from './ChatInvites';

function UtentiAttivi() {
  //Stato locale per gli utenti attivi
  const [localUsers, setLocalUsers] = useState([]);
  const [lastUpdate, setLastUpdate] = useState(new Date());

  // WebSocket context con nuovo stato di aggiornamento
  const { connectedUsers, isConnected, user, lastUsersUpdate } = useWebSocket();

  //Sincronizzazione con WebSocket 
  useEffect(() => {
    setLocalUsers(connectedUsers);
    setLastUpdate(new Date());

    // Animazione visiva
    const element = document.getElementById('users-list');
    if (element) {
      element.style.transform = 'scale(0.98)';
      element.style.transition = 'transform 0.2s ease';
      setTimeout(() => {
        element.style.transform = 'scale(1)';
      }, 100);
    }

  }, [connectedUsers, lastUsersUpdate]); //due dipendenze per aggiornamenti

  //Funzione per ordinare gli utenti (il proprio utente sempre primo)
  const getSortedUsers = () => {
    return [...localUsers].sort((a, b) => {
      // Il proprio utente sempre per primo
      if (a.username === user?.username) return -1;
      if (b.username === user?.username) return 1;

      // Poi per disponibilità (disponibili prima)
      if (a.is_available && !b.is_available) return -1;
      if (!a.is_available && b.is_available) return 1;

      // Infine alfabetico
      return a.username.localeCompare(b.username);
    });
  };

  //Statistiche utenti 
  const availableUsers = localUsers.filter(u => u.is_available).length;
  const busyUsers = localUsers.filter(u => !u.is_available).length;

  return (
    <div style={{ paddingTop: '2rem' }}>
      <ChatInvites />
      <Container fluid style={{ maxWidth: 950, margin: '0 auto' }}>
        {/* Card principale con tema scuro */}
        <Card style={{
          background: '#1c1c1c',
          border: 'none',
          borderRadius: '16px',
          boxShadow: '0 8px 32px rgba(0, 0, 0, 0.3)',
          overflow: 'hidden'
        }}>
          {/* Header della card */}
          <div style={{
            background: '#292929',
            color: '#fff',
            padding: '1.5rem'
          }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <h4 style={{ marginBottom: 0, display: 'flex', alignItems: 'center' }}>
                <i className="bi bi-people-fill" style={{ marginRight: '0.5rem' }}></i>
                Lista Utenti
              </h4>

              {/* Stato connessione e conteggio */}
              <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
                <Badge
                  style={{
                    background: 'rgba(255, 255, 255, 0.2)',
                    fontSize: '0.9rem',
                    padding: '0.4rem 0.8rem',
                    borderRadius: '12px',
                    border: '1px solid rgba(255, 255, 255, 0.3)'
                  }}
                >
                  {localUsers.length} totali
                </Badge>
              </div>
            </div>

            {/* Statistiche dettagliate */}
            {localUsers.length > 0 && (
              <div style={{ marginTop: '1rem', display: 'flex', gap: '1rem' }}>
                <Badge style={{
                  background: 'rgba(144, 174, 133, 0.9)',
                  padding: '0.4rem 0.8rem',
                  borderRadius: '10px',
                  fontSize: '0.8rem'
                }}>
                  <i className="bi bi-check-circle" style={{ marginRight: '0.3rem' }}></i>
                  {availableUsers} Disponibili
                </Badge>
                <Badge style={{
                  background: 'rgba(253, 126, 20, 0.9)',
                  padding: '0.4rem 0.8rem',
                  borderRadius: '10px',
                  fontSize: '0.8rem'
                }}>
                  <i className="bi bi-chat-dots" style={{ marginRight: '0.3rem' }}></i>
                  {busyUsers} In Chat
                </Badge>
              </div>
            )}
          </div>

          {/* Body della card */}
          <div style={{ padding: '1.5rem' }}>
            {!isConnected && (
              <div style={{
                textAlign: 'center',
                color: '#b8b8b8',
                padding: '3rem',
                background: '#1c1c1c',
                borderRadius: '12px',
                border: '1px solid #444'
              }}>
                <i className="bi bi-wifi-off" style={{ fontSize: '3rem', color: '#C9462A', marginBottom: '1rem' }}></i>
                <div style={{ fontSize: '1.2rem', marginBottom: '0.5rem' }}>Connessione WebSocket non disponibile</div>
                <small style={{ color: '#888' }}>Verifica la connessione al server</small>
              </div>
            )}

            {isConnected && localUsers.length === 0 && (
              <div style={{
                textAlign: 'center',
                color: '#b8b8b8',
                padding: '3rem',
                background: '#1c1c1c',
                borderRadius: '12px',
                border: '1px solid #444'
              }}>
                <i className="bi bi-person-x" style={{ fontSize: '3rem', color: '#888', marginBottom: '1rem' }}></i>
                <div style={{ fontSize: '1.2rem', marginBottom: '0.5rem' }}>Nessun utente connesso</div>
                <small style={{ color: '#888' }}>Aspetta che altri utenti si colleghino</small>
              </div>
            )}

            {isConnected && localUsers.length > 0 && (
              <div id="users-list">
                {getSortedUsers().map((connectedUser, index) => (
                  <div
                    key={`${connectedUser.username}-${index}`}
                    style={{
                      background: connectedUser.username === user?.username ? '#1c1c1c' : '#333',
                      border: connectedUser.username === user?.username ? '2px solid #C9462A' : '1px solid #555',
                      borderRadius: '12px',
                      padding: '1rem',
                      marginBottom: index < localUsers.length - 1 ? '0.75rem' : '0',
                      transition: 'all 0.3s ease',
                      position: 'relative'
                    }}
                  >
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                      <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
                        {/* Indicatore stato */}
                        <div
                          style={{
                            width: '14px',
                            height: '14px',
                            borderRadius: '50%',
                            background: connectedUser.is_available ? '#10b981' : '#fd7e14',
                            boxShadow: connectedUser.is_available
                              ? '0 0 12px rgba(16, 185, 129, 0.6)'
                              : '0 0 12px rgba(253, 126, 20, 0.6)',
                            transition: 'all 0.3s ease'
                          }}
                        />

                        {/* Info utente */}
                        <div>
                          <div style={{
                            fontWeight: connectedUser.username === user?.username ? '700' : '500',
                            fontSize: '1.1rem',
                            color: connectedUser.username === user?.username ? '#C9462A' : '#dfdfdf',
                            display: 'flex',
                            alignItems: 'center',
                            gap: '0.5rem'
                          }}>
                            <i className="bi bi-person-circle" style={{ fontSize: '1.2rem' }}></i>
                            {connectedUser.username}
                            {connectedUser.username === user?.username && (
                              <Badge
                                style={{
                                  background: 'linear-gradient(135deg, #C9462A, #b03d24)',
                                  fontSize: '0.7rem',
                                  padding: '0.2rem 0.5rem',
                                  borderRadius: '8px'
                                }}
                              >
                                Tu
                              </Badge>
                            )}
                          </div>

                          {/* Timestamp connessione */}
                          {connectedUser.connected_at && (
                            <div style={{
                              color: '#888',
                              fontSize: '0.8rem',
                              marginTop: '0.25rem',
                              display: 'flex',
                              alignItems: 'center',
                              gap: '0.3rem'
                            }}>
                              <i className="bi bi-clock" style={{ fontSize: '0.7rem' }}></i>
                              Connesso alle {new Date(connectedUser.connected_at).toLocaleTimeString()}
                            </div>
                          )}
                        </div>
                      </div>

                      {/* Badge stato */}
                      <Badge
                        style={{
                          background: connectedUser.is_available
                            ? 'linear-gradient(135deg, #90AE85, #7a9b70)'
                            : 'linear-gradient(135deg, #fd7e14, #e5690a)',
                          padding: '0.5rem 1rem',
                          borderRadius: '20px',
                          fontSize: '0.85rem',
                          border: 'none',
                          boxShadow: connectedUser.is_available
                            ? '0 2px 8px rgba(144, 174, 133, 0.3)'
                            : '0 2px 8px rgba(253, 126, 20, 0.3)'
                        }}
                      >
                        <i className={`bi ${connectedUser.is_available ? 'bi-check-circle' : 'bi-chat-dots'}`}
                          style={{ marginRight: '0.4rem', fontSize: '0.8rem' }}></i>
                        {connectedUser.is_available ? 'Disponibile' : 'In Chat'}
                      </Badge>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Footer con informazioni */}
          {isConnected && (
            <div style={{
              background: '#292929',
              padding: '1rem 1.5rem',
              borderTop: '1px solid #444',
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center'
            }}>
              <div style={{ color: '#888', fontSize: '0.9rem', display: 'flex', alignItems: 'center', gap: '1rem' }}>
                <span>
                  <i className="bi bi-info-circle" style={{ marginRight: '0.3rem' }}></i>
                  Stati: Disponibile • In Chat
                </span>
              </div>
              <small style={{ color: '#666', fontSize: '0.8rem' }}>
                <i className="bi bi-arrow-clockwise" style={{ marginRight: '0.3rem' }}></i>
                Aggiornato: {lastUpdate.toLocaleTimeString()}
              </small>
            </div>
          )}
        </Card>
      </Container>
    </div>
  );
}

export default UtentiAttivi;