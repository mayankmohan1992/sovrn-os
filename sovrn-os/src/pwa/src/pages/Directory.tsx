import { useState, useEffect } from 'preact/hooks';
import { api } from '../lib/api';

interface UserItem {
  public_key: string;
  name: string;
  domain: string;
}

interface DomainItem {
  name: string;
  domain: string;
}

export function Directory() {
  const [activeTab, setActiveTab] = useState<'users' | 'domains'>('users');
  const [users, setUsers] = useState<UserItem[]>([]);
  const [domains, setDomains] = useState<DomainItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  useEffect(() => {
    loadData();
  }, [activeTab]);

  async function loadData() {
    setLoading(true);
    setError('');
    try {
      if (activeTab === 'users') {
        const data = await api.getDirectoryUsers();
        setUsers(data.identities || []);
      } else {
        const data = await api.getDirectoryDomains();
        setDomains(data.domains || []);
      }
    } catch (e: any) {
      console.error('Failed to load directory data:', e);
      setError(e.message || 'Failed to load data from network');
    } finally {
      setLoading(false);
    }
  }

  return (
    <div class="directory">
      <h2>Network Directory</h2>
      <p class="settings-description">Lookup active nodes, profiles, and registered domains on the Sovrn mesh.</p>

      <div class="directory-tabs">
        <button
          class={`btn btn-secondary ${activeTab === 'users' ? 'btn-primary' : ''}`}
          onClick={() => setActiveTab('users')}
        >
          Registered Users ({loading && activeTab === 'users' ? '...' : users.length})
        </button>
        <button
          class={`btn btn-secondary ${activeTab === 'domains' ? 'btn-primary' : ''}`}
          onClick={() => setActiveTab('domains')}
        >
          Registered Domains ({loading && activeTab === 'domains' ? '...' : domains.length})
        </button>
      </div>

      {error && <div class="login-error">{error}</div>}

      {loading ? (
        <p>Loading directory information...</p>
      ) : activeTab === 'users' ? (
        users.length === 0 ? (
          <p>No registered users found on the network.</p>
        ) : (
          <div class="directory-grid">
            {users.map(u => (
              <div class="directory-item card" key={u.public_key}>
                <div class="avatar avatar-md">{u.name[0]}</div>
                <div class="directory-item-info">
                  <span class="directory-item-name">{u.name}</span>
                  {u.domain && <span class="directory-item-domain">{u.domain}</span>}
                  <span class="directory-item-sub" title={u.public_key}>Key: {u.public_key}</span>
                </div>
              </div>
            ))}
          </div>
        )
      ) : (
        domains.length === 0 ? (
          <p>No registered domains found on the network.</p>
        ) : (
          <div class="directory-grid">
            {domains.map(d => (
              <div class="directory-item card" key={d.domain}>
                <div class="avatar avatar-md">⬡</div>
                <div class="directory-item-info">
                  <span class="directory-item-name">{d.domain}</span>
                  <span class="directory-item-sub">Name: {d.name}</span>
                </div>
              </div>
            ))}
          </div>
        )
      )}
    </div>
  );
}
