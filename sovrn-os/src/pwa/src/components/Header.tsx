import { useCallback } from 'preact/hooks';

interface HeaderProps {
  onToggleSidebar: () => void;
}

export function Header({ onToggleSidebar }: HeaderProps) {
  return (
    <header class="header">
      <div class="header-left">
        <button class="btn btn-ghost" onClick={onToggleSidebar} aria-label="Toggle sidebar">
          <svg width="20" height="20" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M3 5a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM3 10a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM3 15a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1z" clip-rule="evenodd"/>
          </svg>
        </button>
        <h1 class="header-title">
          <span class="header-logo">⬡</span> Sovrn
        </h1>
      </div>
      <div class="header-right">
        <button class="btn btn-ghost" title="Notifications">
          <svg width="18" height="18" viewBox="0 0 20 20" fill="currentColor">
            <path d="M10 2a6 6 0 00-6 6v3.58l-1.7 1.707A1 1 0 004 15h12a1 1 0 00.707-1.707L15 11.58V8a6 6 0 00-6-6zM10 18a3 3 0 01-3-3h6a3 3 0 01-3 3z"/>
          </svg>
        </button>
        <button class="btn btn-primary btn-sm" title="New Post">
          Post
        </button>
      </div>
    </header>
  );
}
