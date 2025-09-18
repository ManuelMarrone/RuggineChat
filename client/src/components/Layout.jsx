import Navbar from "./Navbar";
import { useWebSocket } from "../contexts/WebSocketContext";

export default function Layout({ user, children }) {
  const { handleLogout, loginError } = useWebSocket();

  return (
    <>
      <Navbar user={user} onLogout={handleLogout} />
      {loginError && (
        <div style={{
          background: '#ff5252',
          color: '#fff',
          padding: '1rem',
          textAlign: 'center',
          fontWeight: 'bold',
          fontSize: '1rem'
        }}>
          🚫 {loginError} - Verrai disconnesso automaticamente...
        </div>
      )}
      
      <main style={{ padding: "0px" }}>
        {children}
      </main>
    </>
  );
}