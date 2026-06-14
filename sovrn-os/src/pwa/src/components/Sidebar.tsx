import { Link } from 'preact-router/match';

interface SidebarProps {
  open: boolean;
}

export function Sidebar({ open }: SidebarProps) {
  return (
    <nav class={`sidebar ${open ? 'sidebar--open' : 'sidebar--closed'}`}>
      <div class="sidebar-nav">
        <Link href="/" class="sidebar-item" activeClassName="sidebar-item--active">
          <svg width="20" height="20" viewBox="0 0 20 20" fill="currentColor"><path d="M10.707 2.293a1 1 0 00-1.414 0l-7 7a1 1 0 001.414 1.414L4 10.414V17a1 1 0 001 1h2a1 1 0 001-1v-2a1 1 0 011-1h2a1 1 0 011 1v2a1 1 0 001 1h2a1 1 0 001-1v-6.586l.293.293a1 1 0 001.414-1.414l-7-7z"/></svg>
          <span>Home</span>
        </Link>
        <Link href="/messages" class="sidebar-item" activeClassName="sidebar-item--active">
          <svg width="20" height="20" viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M18 10c0 3.866-3.582 7-8 7a8.841 8.841 0 01-4.083-.98L2 17l1.338-3.123C2.493 12.767 2 11.434 2 10c0-3.866 3.582-7 8-7s8 3.134 8 7zM7 9H5v2h2V9zm8 0h-2v2h2V9zm-4 0H9v2h2V9z" clip-rule="evenodd"/></svg>
          <span>Messages</span>
        </Link>
        <Link href="/profile" class="sidebar-item" activeClassName="sidebar-item--active">
          <svg width="20" height="20" viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z" clip-rule="evenodd"/></svg>
          <span>Profile</span>
        </Link>
        <Link href="/settings" class="sidebar-item" activeClassName="sidebar-item--active">
          <svg width="20" height="20" viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M11.49 3.17c-.38-1.56-2.6-1.56-2.98 0a1.532 1.532 0 01-2.286.948c-1.372-.836-2.942.734-2.106 2.106.54.886.061 2.042-.947 2.287-1.561.379-1.561 2.6 0 2.978a1.532 1.532 0 01.947 2.287c-.836 1.372.734 2.942 2.106 2.106a1.532 1.532 0 012.287.947c.379 1.561 2.6 1.561 2.978 0a1.533 1.533 0 012.287-.947c1.372.836 2.942-.734 2.106-2.106a1.533 1.533 0 01.947-2.287c1.561-.379 1.561-2.6 0-2.978a1.532 1.532 0 01-.947-2.287c.836-1.372-.734-2.942-2.106-2.106a1.532 1.532 0 01-2.287-.947zM10 13a3 3 0 100-6 3 3 0 000 6z" clip-rule="evenodd"/></svg>
          <span>Settings</span>
        </Link>
      </div>
      <div class="sidebar-footer">
        <div class="sidebar-user">
          <div class="avatar avatar-sm">?</div>
          <span class="sidebar-user-name">Anonymous</span>
        </div>
      </div>
    </nav>
  );
}
