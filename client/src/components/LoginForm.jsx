import { useState } from 'react';
import { login } from '../API/API.mjs';

export default function LoginForm({ onLogin }) {
  const [username, setUsername] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError('');
    setLoading(true);

    try {
      // Il login HTTP è solo per validazione
      await login({ username });

      // Passiamo solo l'username, la connessione WebSocket avverrà automaticamente
      onLogin({ username });
    } catch (err) {
      setError(err.message || 'Errore durante il login');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{
      minHeight: '100vh',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      background: 'linear-gradient(180deg, #C9462A 0%, #90AE85 100%)'
    }}>
      <form
        onSubmit={handleSubmit}
        style={{
          background: '#292929',
          padding: '2.5rem 2rem',
          borderRadius: '16px',
          boxShadow: '0 8px 32px 0 rgba(31, 38, 135, 0.2)',
          minWidth: 320,
          display: 'flex',
          flexDirection: 'column',
          gap: '1.2rem'
        }}
      >
        <h2 style={{ textAlign: 'center', color: '#C9462A', marginBottom: 0 }}>
          Ruggine Chat
        </h2>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
          <label htmlFor="login-username" style={{ color: '#dfdfdf', fontWeight: 500 }}>
            Username
          </label>
          <input
            id="login-username"
            value={username}
            onChange={e => setUsername(e.target.value)}
            placeholder="Inserisci il tuo username"
            autoComplete="username"
            style={{
              padding: '0.5rem',
              borderRadius: 6,
              border: '1px solid #bdbdbd',
              fontSize: 16
            }}
            required
            disabled={loading}
          />
          <small style={{ color: '#b8b8b8ff', fontSize: '0.85rem' }}>
            💡 Scegli un username unico non ancora in uso
          </small>
        </div>
        <button
          type="submit"
          disabled={loading}
          style={{
            background: loading ? '#555' : 'linear-gradient(135deg, #C9462A, #b0604eff)',
            color: '#fff',
            border: 'none',
            borderRadius: 6,
            padding: '0.7rem',
            fontWeight: 600,
            fontSize: 16,
            cursor: loading ? 'not-allowed' : 'pointer',
            marginTop: 8,
            boxShadow: loading ? 'none' : '0 2px 8px rgba(201, 70, 42, 0.3)'
          }}
        >
          {loading ? 'Accesso in corso...' : 'Accedi'}
        </button>
        {error && (
          <div style={{
            color: '#fff',
            background: '#ff5252',
            borderRadius: 6,
            padding: '0.5rem',
            textAlign: 'center',
            fontWeight: 500
          }}>
            {error}
          </div>
        )}
      </form>
    </div>
  );
}