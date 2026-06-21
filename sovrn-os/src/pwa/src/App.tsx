import { Router, Route } from 'preact-router';
import { Header } from './components/Header';
import { Sidebar } from './components/Sidebar';
import { Timeline } from './pages/Timeline';
import { Messages } from './pages/Messages';
import { Profile } from './pages/Profile';
import { Settings } from './pages/Settings';
import { Login } from './pages/Login';
import { Directory } from './pages/Directory';
import { useState, useCallback } from 'preact/hooks';
import { signal } from '@preact/signals';

// Auth state (global signal)
export const authToken = signal<string | null>(null);
export const currentUserId = signal<string | null>(null);
export const currentUserDomain = signal<string | null>(null);

export function App() {
  const [sidebarOpen, setSidebarOpen] = useState(true);

  const toggleSidebar = useCallback(() => {
    setSidebarOpen(prev => !prev);
  }, []);

  // If not authenticated, show login
  if (!authToken.value) {
    return <Login />;
  }

  return (
    <div class="app-layout">
      <Header onToggleSidebar={toggleSidebar} />
      <div class="app-body">
        <Sidebar open={sidebarOpen} />
        <main class="app-main">
          <Router>
            <Route path="/" component={Timeline} />
            <Route path="/messages/:peerId?" component={Messages} />
            <Route path="/profile/:pubkey?" component={Profile} />
            <Route path="/settings" component={Settings} />
            <Route path="/directory" component={Directory} />
          </Router>
        </main>
      </div>
    </div>
  );
}
