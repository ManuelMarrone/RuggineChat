// API functions per la comunicazione con il backend Rust
const BASE_URL = 'http://localhost:3000';

// Funzioni per la gestione degli utenti
const signup = async (username) => {
  try {
    const response = await fetch(`${BASE_URL}/api/login`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ username }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(errorText || 'Signup failed');
    }

    return await response.json();
  } catch (error) {
    console.error('Signup error:', error);
    throw error;
  }
};

const login = async (credentials) => {
  try {
    // Verifica direttamente tramite l'endpoint di login
    const response = await fetch(`${BASE_URL}/api/login`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ username: credentials.username }),
    });

    if (response.status === 409) { // Conflict - username già in uso
      const errorText = await response.json();
      throw new Error(errorText);
    }

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(errorText || 'Login failed');
    }

    // Username disponibile, restituisci i dati utente
    return { username: credentials.username, available: true };
  } catch (error) {
    console.error('Login error:', error);
    throw error;
  }
};

const getAllUsers = async () => {
  try {
    const response = await fetch(`${BASE_URL}/api/users`);

    if (!response.ok) {
      throw new Error('Failed to fetch users');
    }

    return await response.json();
  } catch (error) {
    console.error('Get users error:', error);
    throw error;
  }
};

const updateUserAvailability = async (username, available) => {
  try {
    const response = await fetch(`${BASE_URL}/api/users/${username}/availability`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(available),
    });

    if (!response.ok) {
      throw new Error('Failed to update availability');
    }

    return await response.json();
  } catch (error) {
    console.error('Update availability error:', error);
    throw error;
  }
};

// Named exports
export {
  login,
  signup,
  getAllUsers,
  updateUserAvailability
};