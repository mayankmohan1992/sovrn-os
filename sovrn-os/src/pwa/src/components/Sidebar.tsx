import { Link } from 'preact-router/match';
import { useState, useEffect } from 'preact/hooks';
import { currentUserId } from '../App';
import { api } from '../lib/api';

interface SidebarProps {
  open: boolean;
}

export function Sidebar({ open }: SidebarProps) {
  const [profile, setProfile] = useState<any>(null);

  useEffect(() => {
    if (currentUserId.value) {
      loadUserProfile();
    }
  }, [currentUserId.value]);

  async function loadUserProfile() {
    try {
      const data = await api.getProfile(currentUserId.value!);
      setProfile(data);
    } catch (e) {
      console.error('Failed to load user profile for sidebar:', e);
    }
  }

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
        <Link href="/directory" class="sidebar-item" activeClassName="sidebar-item--active">
          <svg width="20" height="20" viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M12 2a5 5 0 11-10 0 5 5 0 0110 0zm0 13.5a5.5 5.5 0 00-11 0V16a1 1 0 001 1h9a1 1 0 001-1v-.5zM17.5 3a.5.5 0 01.5.5v1.5a.5.5 0 01-.5.5h-1.5a.5.5 0 01-.5-.5V3.5a.5.5 0 01.5-.5h1.5zm0 4.5a.5.5 0 01.5.5v1.5a.5.5 0 01-.5.5h-1.5a.5.5 0 01-.5-.5V8a.5.5 0 01.5-.5h1.5zm.5 5a.5.5 0 00-.5-.5h-1.5a.5.5 0 00-.5.5v1.5a.5.5 0 00.5.5h1.5a.5.5 0 00.5-.5V13.5z" clip-rule="evenodd"/></svg>
          <span>Directory</span>
        </Link>
        <Link href="/settings" class="sidebar-item" activeClassName="sidebar-item--active">
          <svg width="20" height="20" viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M11.49 3.17c-.38-1.56-2.6-1.56-2.98 0a1.532 1.532 0 01-2.286.948c-1.372-.836-2.942.734-2.106 2.106.54.886.061 2.042-.947 2.287-1.561.379-1.561 2.6 0 2.978a1.532 1.532 0 01.947 2.287c-.836 1.372.734 2.942 2.106 2.106a1.532 1.532 0 012.287.947c.379 1.561 2.6 1.561 2.978 0a1.533 1.533 0 012.287-.947c1.372.836 2.942-.734 2.106-2.106a1.533 1.533 0 01.947-2.287c1.561-.379 1.561-2.6 0-2.978a1.532 1.532 0 01-.947-2.287c.836-1.372-.734-2.942-2.106-2.106a1.532 1.532 0 01-2.287-.947zM10 13a3 3 0 100-6 3 3 0 000 6z" clip-rule="evenodd"/></svg>
          <span>Settings</span>
        </Link>
      </div>
      <div class="sidebar-footer">
        <div class="sidebar-user">
          <div class="avatar avatar-sm">{profile?.name?.[0] || '?'}</div>
          <span class="sidebar-user-name" title={profile?.domain || 'Anonymous'}>
            {profile?.name || 'Anonymous'}
          </span>
        </div>
      </div>
    </nav>
  );
}
