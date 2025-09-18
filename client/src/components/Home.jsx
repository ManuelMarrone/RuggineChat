
import Card from 'react-bootstrap/Card';
import { Button, Form, Badge, Row, Col } from 'react-bootstrap';
import 'bootstrap-icons/font/bootstrap-icons.css';
import { useState, useEffect } from 'react';
import { useWebSocket } from '../contexts/WebSocketContext';
import ChatInvites from './ChatInvites';


function Home({ user }) {
  const { sendChatInvite, connectedUsers } = useWebSocket();
  const [chatMode, setChatMode] = useState(''); // '', 'group', 'private'
  const [selectedUsers, setSelectedUsers] = useState(new Set());
  const [privateChatUser, setPrivateChatUser] = useState('');

  // Calcola utenti disponibili dai connectedUsers in tempo reale
  const availableUsers = connectedUsers.filter(u =>
    u.is_available && u.username !== user.username
  );

  const handleUserToggle = (username) => {
    const newSelected = new Set(selectedUsers);
    if (newSelected.has(username)) {
      newSelected.delete(username);
    } else {
      newSelected.add(username);
    }
    setSelectedUsers(newSelected);
  };

  const handleSelectPrivateUser = (username) => {
    setPrivateChatUser(username);
  };

  const handleCreateGroup = () => {
    if (selectedUsers.size >= 2) {
      const selectedUsernames = availableUsers
        .filter(u => selectedUsers.has(u.username))
        .map(u => u.username);

      // Aggiungi l'utente corrente al gruppo
      const groupMembers = [user.username, ...selectedUsernames];

      // Invia inviti invece di navigare direttamente
      const result = sendChatInvite('group', '', groupMembers,
        `${user.username} ti ha invitato in un gruppo di ${groupMembers.length} membri`);

      if (result.success) {
        // Reset il modulo dopo l'invio
        resetChatMode();
        alert('Inviti inviati! Aspetta che almeno un membro accetti per aprire la chat.');
      } else {
        alert('Errore durante l\'invio degli inviti. Riprova.');
      }
    }
  };

  const handleStartPrivateChat = () => {
    if (privateChatUser.trim()) {
      // Verifica che l'utente selezionato sia disponibile
      const targetUser = availableUsers.find(u => u.username === privateChatUser.trim());

      if (!targetUser) {
        alert('Utente non trovato o non disponibile. Seleziona un utente dalla lista degli utenti attivi.');
        return;
      }

      // Invia invito invece di navigare direttamente
      const result = sendChatInvite('private', privateChatUser.trim(), [],
        `${user.username} ti ha invitato in una chat privata`);

      if (result.success) {
        // Reset il modulo dopo l'invio
        resetChatMode();
        alert('Invito inviato! Aspetta che l\'utente accetti per aprire la chat.');
      } else {
        alert('Errore durante l\'invio dell\'invito. Riprova.');
      }
    }
  };

  const resetChatMode = () => {
    setChatMode('');
    setSelectedUsers(new Set());
    setPrivateChatUser('');
  };
  return (
    <div style={{ paddingTop: '2rem' }}>
      {/* Pannello inviti/notifications: visibile solo nella Home */}
      <ChatInvites />
      {/* Dashboard principale */}
      <div style={{
        maxWidth: 950,
        margin: '2rem auto',
        padding: '2rem 2rem 3rem 2rem',
        background: '#292929',
        borderRadius: 16,
        boxShadow: '0 8px 32px rgba(0, 0, 0, 0.3)'
      }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '1.5rem' }}>
          <div>
            <h2 style={{ marginBottom: 0, color: "#dfdfdf" }}>
              <i className="bi bi-chat-dots-fill" style={{ marginRight: 10, color: '#C9462A' }}></i>
              Inizia Nuova Chat
            </h2>
            <div style={{ color: "#b8b8b8", fontSize: 18, marginTop: '0.5rem' }}>
              Benvenuto, <span style={{ fontWeight: 600, color: "#C9462A" }}>{user.username}</span>! Scegli il tipo di chat da avviare.
            </div>
          </div>
        </div>
        <hr style={{ borderColor: '#444' }} />

        {/* Selezione modalità chat */}
        {!chatMode && (
          <Card className="shadow-sm my-4" style={{ border: 'none', background: '#1c1c1c' }}>
            <Card.Body>
              <Card.Title as="h4" className="mb-3" style={{ color: "#dfdfdf" }}>
                <i className="bi bi-chat-square-dots" style={{ marginRight: 8, color: '#C9462A' }}></i>
                Seleziona Modalità Chat
              </Card.Title>
              <Row className="g-3">
                <Col md={6}>
                  <Card
                    className="h-100"
                    style={{
                      cursor: 'pointer',
                      border: '2px solid transparent',
                      transition: 'all 0.3s ease',
                      background: '#333'
                    }}
                    onMouseEnter={(e) => {
                      e.currentTarget.style.borderColor = '#90AE85';
                      e.currentTarget.style.transform = 'translateY(-4px)';
                      e.currentTarget.style.boxShadow = '0 8px 24px rgba(144, 174, 133, 0.3)';
                    }}
                    onMouseLeave={(e) => {
                      e.currentTarget.style.borderColor = 'transparent';
                      e.currentTarget.style.transform = 'translateY(0)';
                      e.currentTarget.style.boxShadow = 'none';
                    }}
                    onClick={() => setChatMode('group')}
                  >
                    <Card.Body className="text-center" style={{ padding: '2rem' }}>
                      <i className="bi bi-people-fill" style={{ fontSize: '3rem', color: '#90AE85', marginBottom: '1rem' }}></i>
                      <Card.Title style={{ color: '#90AE85', fontSize: '1.25rem' }}>Chat di Gruppo</Card.Title>
                      <Card.Text style={{ color: '#dfdfdf', fontSize: '0.95rem' }}>
                        Crea una conversazione con più utenti contemporaneamente.
                        Perfetta per discussioni di team o progetti collaborativi.
                      </Card.Text>
                    </Card.Body>
                  </Card>
                </Col>
                <Col md={6}>
                  <Card
                    className="h-100"
                    style={{
                      cursor: 'pointer',
                      border: '2px solid transparent',
                      transition: 'all 0.3s ease',
                      background: '#333'
                    }}
                    onMouseEnter={(e) => {
                      e.currentTarget.style.borderColor = '#C9462A';
                      e.currentTarget.style.transform = 'translateY(-4px)';
                      e.currentTarget.style.boxShadow = '0 8px 24px rgba(201, 70, 42, 0.3)';
                    }}
                    onMouseLeave={(e) => {
                      e.currentTarget.style.borderColor = 'transparent';
                      e.currentTarget.style.transform = 'translateY(0)';
                      e.currentTarget.style.boxShadow = 'none';
                    }}
                    onClick={() => setChatMode('private')}
                  >
                    <Card.Body className="text-center" style={{ padding: '2rem' }}>
                      <i className="bi bi-person-fill" style={{ fontSize: '3rem', color: '#C9462A', marginBottom: '1rem' }}></i>
                      <Card.Title style={{ color: '#C9462A', fontSize: '1.25rem' }}>Chat Privata</Card.Title>
                      <Card.Text style={{ color: '#dfdfdf', fontSize: '0.95rem' }}>
                        Avvia una conversazione uno-a-uno con un utente specifico.
                        Ideale per comunicazioni dirette e riservate.
                      </Card.Text>
                    </Card.Body>
                  </Card>
                </Col>
              </Row>
            </Card.Body>
          </Card>
        )}

        {/* Chat di Gruppo */}
        {chatMode === 'group' && (
          <Card className="shadow-sm my-4" style={{ border: 'none', background: '#1c1c1c' }}>
            <Card.Body>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
                <Card.Title as="h4" style={{ color: "#90AE85", marginBottom: 0 }}>
                  <i className="bi bi-people-fill" style={{ marginRight: 8 }}></i>
                  Crea Chat di Gruppo
                </Card.Title>
                <Button
                  style={{
                    background: 'transparent',
                    border: '2px solid #555',
                    color: '#dfdfdf',
                    borderRadius: '8px'
                  }}
                  size="sm"
                  onClick={resetChatMode}
                >
                  <i className="bi bi-arrow-left" style={{ marginRight: 4 }}></i>
                  Indietro
                </Button>
              </div>

              <Card.Text style={{ marginBottom: '1.5rem', color: '#b8b8b8' }}>
                Seleziona gli utenti disponibili per creare una chat di gruppo.
                <Badge
                  style={{
                    background: 'linear-gradient(135deg, #90AE85, #7a9b70)',
                    marginLeft: '0.5rem',
                    borderRadius: '12px'
                  }}
                >
                  {selectedUsers.size} selezionati
                </Badge>
              </Card.Text>

              <Row className="g-2 mb-3">
                {availableUsers.length === 0 ? (
                  <Col xs={12}>
                    <div style={{
                      textAlign: 'center',
                      padding: '2rem',
                      color: '#888',
                      background: '#333',
                      borderRadius: '8px',
                      border: '1px solid #444'
                    }}>
                      <i className="bi bi-people-x" style={{ fontSize: '2rem', marginBottom: '0.5rem' }}></i>
                      <div>Nessun utente disponibile per chat di gruppo</div>
                      <small>Gli altri utenti devono essere online e disponibili</small>
                    </div>
                  </Col>
                ) : (
                  availableUsers.map(member => (
                    <Col key={member.username} md={6} lg={4}>
                      <div
                        style={{
                          padding: '0.75rem',
                          background: selectedUsers.has(member.username)
                            ? 'linear-gradient(135deg, rgba(144, 174, 133, 0.2), rgba(144, 174, 133, 0.1))'
                            : '#333',
                          border: selectedUsers.has(member.username)
                            ? '2px solid #90AE85'
                            : '1px solid #555',
                          borderRadius: '8px',
                          cursor: 'pointer',
                          transition: 'all 0.2s ease',
                          color: selectedUsers.has(member.username) ? '#90AE85' : '#dfdfdf'
                        }}
                        onClick={() => handleUserToggle(member.username)}
                      >
                        <div style={{ display: 'flex', alignItems: 'center' }}>
                          <i className="bi bi-circle-fill" style={{ color: '#10b981', fontSize: '10px', marginRight: '8px' }}></i>
                          <span style={{ fontWeight: selectedUsers.has(member.username) ? '600' : '400' }}>
                            {member.username}
                          </span>
                          {selectedUsers.has(member.username) && (
                            <i className="bi bi-check-circle-fill" style={{ color: '#90AE85', marginLeft: 'auto' }}></i>
                          )}
                        </div>
                      </div>
                    </Col>
                  ))
                )}
              </Row>

              <div style={{ textAlign: 'center' }}>
                <Button
                  size="lg"
                  disabled={selectedUsers.size < 2}
                  onClick={handleCreateGroup}
                  style={{
                    padding: '0.75rem 2rem',
                    background: selectedUsers.size >= 2
                      ? 'linear-gradient(135deg, #90AE85, #7a9b70)'
                      : '#555',
                    border: 'none',
                    borderRadius: '8px',
                    color: selectedUsers.size >= 2 ? 'white' : '#888',
                    fontWeight: '500'
                  }}
                >
                  <i className="bi bi-plus-circle" style={{ marginRight: 8 }}></i>
                  Crea Gruppo ({selectedUsers.size} membri)
                </Button>
                {selectedUsers.size < 2 && (
                  <div style={{ marginTop: '0.5rem', color: '#888', fontSize: '0.9rem' }}>
                    Seleziona almeno 2 utenti per creare un gruppo
                  </div>
                )}
              </div>
            </Card.Body>
          </Card>
        )}

        {/* Chat Privata */}
        {chatMode === 'private' && (
          <Card className="shadow-sm my-4" style={{ border: 'none', background: '#1c1c1c' }}>
            <Card.Body>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
                <Card.Title as="h4" style={{ color: "#C9462A", marginBottom: 0 }}>
                  <i className="bi bi-person-fill" style={{ marginRight: 8 }}></i>
                  Avvia Chat Privata
                </Card.Title>
                <Button
                  style={{
                    background: 'transparent',
                    border: '2px solid #555',
                    color: '#dfdfdf',
                    borderRadius: '8px'
                  }}
                  size="sm"
                  onClick={resetChatMode}
                >
                  <i className="bi bi-arrow-left" style={{ marginRight: 4 }}></i>
                  Indietro
                </Button>
              </div>

              <Card.Text style={{ marginBottom: '1.5rem', color: '#b8b8b8' }}>
                Seleziona un utente disponibile per iniziare una chat privata.
              </Card.Text>

              {/* Lista utenti disponibili per chat privata */}
              <div style={{ marginBottom: '1.5rem' }}>
                <h6 style={{ color: '#C9462A', marginBottom: '0.75rem' }}>
                  <i className="bi bi-people" style={{ marginRight: 6 }}></i>
                  Utenti Disponibili ({availableUsers.length})
                </h6>

                {availableUsers.length === 0 ? (
                  <div style={{
                    textAlign: 'center',
                    padding: '2rem',
                    color: '#888',
                    background: '#333',
                    borderRadius: '8px',
                    border: '1px solid #444'
                  }}>
                    <i className="bi bi-person-x" style={{ fontSize: '2rem', marginBottom: '0.5rem' }}></i>
                    <div>Nessun utente disponibile al momento</div>
                  </div>
                ) : (
                  <Row className="g-2">
                    {availableUsers.map(member => (
                      <Col key={member.username} md={6} lg={4}>
                        <div
                          style={{
                            padding: '0.75rem',
                            background: privateChatUser === member.username
                              ? 'linear-gradient(135deg, rgba(201, 70, 42, 0.2), rgba(201, 70, 42, 0.1))'
                              : '#333',
                            border: privateChatUser === member.username
                              ? '2px solid #C9462A'
                              : '1px solid #555',
                            borderRadius: '8px',
                            cursor: 'pointer',
                            transition: 'all 0.2s ease',
                            color: privateChatUser === member.username ? '#C9462A' : '#dfdfdf'
                          }}
                          onClick={() => handleSelectPrivateUser(member.username)}
                        >
                          <div style={{ display: 'flex', alignItems: 'center' }}>
                            <i className="bi bi-circle-fill" style={{ color: '#10b981', fontSize: '10px', marginRight: '8px' }}></i>
                            <span style={{ fontWeight: privateChatUser === member.username ? '600' : '400' }}>
                              {member.username}
                            </span>
                            {privateChatUser === member.username && (
                              <i className="bi bi-check-circle-fill" style={{ color: '#C9462A', marginLeft: 'auto' }}></i>
                            )}
                          </div>
                        </div>
                      </Col>
                    ))}
                  </Row>
                )}
              </div>

              <div style={{ textAlign: 'center' }}>
                <Button
                  size="lg"
                  disabled={!privateChatUser.trim()}
                  style={{
                    padding: '0.75rem 2rem',
                    background: privateChatUser.trim()
                      ? 'linear-gradient(135deg, #C9462A, #b03d24)'
                      : '#555',
                    border: 'none',
                    borderRadius: '8px',
                    color: privateChatUser.trim() ? 'white' : '#888',
                    fontWeight: '500'
                  }}
                  onClick={handleStartPrivateChat}
                >
                  <i className="bi bi-chat-fill" style={{ marginRight: 8 }}></i>
                  Avvia Chat Privata
                  {privateChatUser && (
                    <span style={{ marginLeft: '8px', fontWeight: 'normal' }}>
                      con {privateChatUser}
                    </span>
                  )}
                </Button>
              </div>
            </Card.Body>
          </Card>
        )}
      </div>
    </div>
  );
}

export default Home;