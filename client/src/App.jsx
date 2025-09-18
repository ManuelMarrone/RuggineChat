import './App.css'
import { Routes, Route, Navigate } from "react-router-dom";
import { useState } from "react";
import Layout from "./components/Layout";
import LoginForm from "./components/LoginForm";
import { Outlet } from "react-router-dom";
import Home from "./components/Home";
import WebSocketChat from "./components/WebSocketChat";
import UtentiAttivi from './components/ActiveUsers';
import { WebSocketProvider } from "./contexts/WebSocketContext";



function App() {
  const [user, setUser] = useState(null);

  const handleLogin = (userData) => {
    setUser(userData);
  };

  const handleLogout = async () => {

    setUser(null);
  };

  return (
    <WebSocketProvider key={user?.username || 'anon'} user={user} onLogout={handleLogout}>
      <Routes>
        {/* Route di login */}
        <Route
          path="/login"
          element={
            !user ? (
              <LoginForm onLogin={handleLogin} />
            ) : (
              <Navigate to="/" replace />
            )
          }
        />

        {/* Route protette con layout */}
        <Route
          path="/"
          element={
            user ? (
              <Layout user={user}>
                <Outlet />
              </Layout>
            ) : (
              <Navigate to="/login" replace />
            )
          }
        >
          <Route index element={<Home user={user} />} />
          <Route path="users" element={<UtentiAttivi />} />
          <Route path="chat" element={<WebSocketChat />} />
        </Route>

        {/* Catch-all per route non esistenti */}
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </WebSocketProvider>
  );
}

export default App;