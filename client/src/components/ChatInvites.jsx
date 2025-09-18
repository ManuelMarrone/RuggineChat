import React from 'react';
import { Button, Badge } from 'react-bootstrap';
import { useWebSocket } from '../contexts/WebSocketContext';
import { useNavigate } from 'react-router-dom';

function ChatInvites() {
  const navigate = useNavigate();
  const { chatInvites, chatReady, chatDeclined, respondToChatInvite, removeChatReady, removeChatDeclined, sendRawMessage, user, clearAllNotifications } = useWebSocket();



  // Gestisce la risposta agli inviti
  const handleInviteResponse = (invite, accepted) => {
    const success = respondToChatInvite(invite.id, accepted, invite.from, invite.chat_type, invite.chat_id, invite.from_session_id);

    if (success && accepted) {
      // Se accettato, naviga alla chat
      const chatState = {
        chatType: invite.chat_type.Private ? 'private' : 'group',
        //Per chat private, l'utente invitato deve chattare con chi ha inviato l'invito
        targetUser: invite.chat_type.Private ? invite.from : '',
        members: invite.chat_type.Group?.members || [user.username, invite.from],
        groupName: invite.chat_type.Group ? `Gruppo di ${invite.chat_type.Group.members.length} membri` : '',
        chatId: invite.chat_id
      };

      navigate('/chat', { state: chatState });
    }

  };

  // Gestisce l'entrata nella chat quando pronta
  const handleEnterReadyChat = (readyChat) => {
    // Rimuovi la notifica di chat pronta
    removeChatReady(readyChat.chat_id);

    // Naviga alla chat
    const chatState = {
      chatType: readyChat.chat_type.Private ? 'private' : 'group',
      targetUser: readyChat.chat_type.Private ? readyChat.accepted_by : '',
      members: readyChat.chat_type.Group?.members || [user.username, readyChat.accepted_by],
      groupName: readyChat.chat_type.Group ? `Gruppo di ${readyChat.chat_type.Group.members.length} membri` : '',
      chatId: readyChat.chat_id
    };

    navigate('/chat', { state: chatState });

    const systemMessage = {
      message_type: 'ChatMessage',
      data: JSON.stringify({
        id: crypto.randomUUID(),
        chat_id: readyChat.chat_id,
        username: "Sistema",
        content: `${readyChat.inviter} (Host) è entrato nella chat`,
        timestamp: new Date().toISOString(),
        chat_type: readyChat.chat_type.Private
          ? { Private: { target: readyChat.accepted_by } }
          : { Group: { members: readyChat.chat_type.Group?.members || [user.username, readyChat.accepted_by] } }
      })
    };

    sendRawMessage(systemMessage);

  };



  return (
    <div>
      {chatReady.length > 0 && (
        <div style={{
          maxWidth: 950,
          margin: '1rem auto',
          padding: '1rem',
          background: 'linear-gradient(135deg, #90AE85, #7a9b70)',
          borderRadius: 12,
          boxShadow: '0 4px 20px rgba(144, 174, 133, 0.3)'
        }}>
          <h5 style={{ marginBottom: '1rem', color: '#fff', fontWeight: '600' }}>
            <i className="bi bi-check-circle-fill" style={{ marginRight: 8 }}></i>
            Chat Pronte ({chatReady.length})
          </h5>
          {chatReady.map(ready => (
            <div key={ready.chat_id} style={{
              background: 'rgba(255, 255, 255, 0.95)',
              padding: '1rem',
              marginBottom: '0.5rem',
              borderRadius: 8,
              boxShadow: '0 2px 8px rgba(0, 0, 0, 0.1)'
            }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <div>
                  <strong style={{ color: '#2d3748' }}>{ready.accepted_by} ha accettato il tuo invito!</strong>
                  <div style={{ fontSize: '0.9rem', color: '#718096', marginTop: '0.25rem' }}>
                    {ready.chat_type.Private ? 'Chat Privata' : `Chat di Gruppo`} - Pronta per essere aperta
                  </div>
                </div>
                <div>
                  <Button
                    style={{
                      background: 'linear-gradient(135deg, #C9462A, #b03d24)',
                      border: 'none',
                      borderRadius: '8px',
                      fontWeight: '500',
                      boxShadow: '0 2px 8px rgba(201, 70, 42, 0.3)'
                    }}
                    size="sm"
                    onClick={() => handleEnterReadyChat(ready)}
                  >
                    Entra in Chat
                  </Button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Inviti rifiutati */}
      {chatDeclined && chatDeclined.length > 0 && (
        <div style={{
          maxWidth: 950,
          margin: '1rem auto',
          padding: '1rem',
          background: 'linear-gradient(135deg, #c53030, #9b2c2c)',
          borderRadius: 12,
          boxShadow: '0 4px 20px rgba(197, 48, 48, 0.3)'
        }}>
          <h5 style={{ marginBottom: '1rem', color: '#fff', fontWeight: '600' }}>
            <i className="bi bi-x-circle-fill" style={{ marginRight: 8 }}></i>
            Inviti Rifiutati ({chatDeclined.length})
          </h5>
          {chatDeclined.map(item => (
            <div key={item.invite_id} style={{
              background: 'rgba(255, 255, 255, 0.95)',
              padding: '1rem',
              marginBottom: '0.5rem',
              borderRadius: 8,
              boxShadow: '0 2px 8px rgba(0, 0, 0, 0.1)'
            }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <div>
                  <strong style={{ color: '#2d3748' }}>{item.responding_user} ha rifiutato il tuo invito.</strong>
                  <div style={{ fontSize: '0.9rem', color: '#718096', marginTop: '0.25rem' }}>
                    {item.chat_type?.Private ? 'Chat Privata' : 'Chat di Gruppo'} - L'invito è stato rifiutato
                  </div>
                </div>
                <div>
                  <Button
                    style={{
                      background: '#6c757d',
                      border: 'none',
                      borderRadius: '8px',
                      fontWeight: '500'
                    }}
                    size="sm"
                    onClick={() => removeChatDeclined(item.invite_id)}
                  >
                    Chiudi
                  </Button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Inviti ricevuti */}
      {chatInvites.length > 0 && (
        <div style={{
          maxWidth: 950,
          margin: '1rem auto',
          padding: '1rem',
          background: 'linear-gradient(135deg, #C9462A, #b03d24)',
          borderRadius: 12,
          boxShadow: '0 2px 8px rgba(0, 0, 0, 0.1)'
        }}>
          <h5 style={{ marginBottom: '1rem', color: '#fff', fontWeight: '600' }}>
            <i className="bi bi-envelope-fill" style={{ marginRight: 8 }}></i>
            Inviti Chat Ricevuti ({chatInvites.length})
          </h5>
          {chatInvites.map(invite => (
            <div key={invite.id} style={{
              background: 'rgba(231, 229, 229, 1)',
              padding: '1rem',
              marginBottom: '0.5rem',
              borderRadius: 8,
              border: '1px solid rgba(255, 255, 255, 0.2)',
              boxShadow: '0 2px 8px rgba(0, 0, 0, 0.1)'
            }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <div>
                  <strong style={{ color: '#2d3748' }}>{invite.message}</strong>
                  <div style={{ fontSize: '0.9rem', color: '#718096', marginTop: '0.25rem' }}>
                    {invite.chat_type.Private ? 'Chat Privata' : `Chat di Gruppo (${invite.chat_type.Group?.members.length} membri)`}
                  </div>
                </div>
                <div>
                  <Button
                    style={{
                      background: 'linear-gradient(135deg, #90AE85, #7a9b70)',
                      border: 'none',
                      borderRadius: '8px',
                      marginRight: '0.5rem',
                      fontWeight: '500',
                      boxShadow: '0 2px 8px rgba(144, 174, 133, 0.3)'
                    }}
                    size="sm"
                    onClick={() => handleInviteResponse(invite, true)}
                  >
                    Accetta
                  </Button>
                  <Button
                    style={{
                      background: '#6c757d',
                      border: 'none',
                      borderRadius: '8px',
                      fontWeight: '500'
                    }}
                    size="sm"
                    onClick={() => handleInviteResponse(invite, false)}
                  >
                    Rifiuta
                  </Button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export default ChatInvites;