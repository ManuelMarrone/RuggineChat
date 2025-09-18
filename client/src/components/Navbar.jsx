import React from 'react';
import { Navbar as BootstrapNavbar, Nav, Container, Button, Badge } from 'react-bootstrap';
import { Link, useLocation, useNavigate } from 'react-router-dom';
import { useWebSocket } from '../contexts/WebSocketContext';

function Navbar({ user, onLogout }) {
  const location = useLocation();

  const navigate = useNavigate();
  const { connectedUsers } = useWebSocket();

  const navigateToActiveUsers = () => {
    navigate('/users');
  };

  return (
    <BootstrapNavbar bg="dark" variant="dark" expand="lg" className="mb-3">
      <Container>


        <BootstrapNavbar.Toggle aria-controls="basic-navbar-nav" />
        <BootstrapNavbar.Collapse id="basic-navbar-nav">
          <Nav className="me-auto">

            <>
              <Nav.Link
                as={Link}
                to="/"
                active={location.pathname === '/'}
              >
                Home
              </Nav.Link>
              <Nav.Link

                as="button"
                onClick={navigateToActiveUsers}
                style={{
                  background: 'none',
                  border: 'none',
                  color: location.pathname === '/' ? '#fff' : 'rgba(255,255,255,.55)',
                  cursor: 'pointer'
                }}
              >
                Utenti Attivi
                <Badge
                  className="badge-pulse ms-2"
                  style={{
                    background: 'linear-gradient(135deg, #10b981, #059669)',
                    fontSize: '0.75rem',
                    padding: '0.25rem 0.5rem',
                    borderRadius: '12px',
                    boxShadow: '0 2px 4px rgba(16, 185, 129, 0.3)',
                    border: 'none',
                    marginLeft: '0.25rem'
                  }}
                >
                  {connectedUsers.length}
                </Badge>
              </Nav.Link>
            </>

          </Nav>

          <Nav>
            <BootstrapNavbar.Text className="me-3">
              Benvenut*, {user?.username || 'User'}
            </BootstrapNavbar.Text>
            <Button variant="outline-light" size="sm" onClick={onLogout}>
              Logout
            </Button>
          </Nav>
        </BootstrapNavbar.Collapse>
      </Container>
    </BootstrapNavbar>
  );
}

export default Navbar;