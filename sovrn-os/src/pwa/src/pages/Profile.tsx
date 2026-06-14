import { useState, useEffect } from 'preact/hooks';
import { api } from '../lib/api';

export function Profile({ pubkey }: { pubkey?: string }) {
  const [profile, setProfile] = useState<any>(null);
  const [aliases, setAliases] = useState<any[]>([]);
  const [domainName, setDomainName] = useState('');
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadProfile();
  }, [pubkey]);

  async function loadProfile() {
    setLoading(true);
    try {
      const data = await api.getProfile(pubkey || 'me');
      setProfile(data);
    } catch (e) {
      console.error('Failed to load profile:', e);
    } finally {
      setLoading(false);
    }
  }

  async function handleRegisterDomain() {
    if (!domainName.trim()) return;
    try {
      const result = await api.dhtCheck(domainName.trim());
      if (result.available) {
        await api.createAlias('me', domainName.trim());
        alert(`Domain ${domainName}.sovrn registered!`);
        setDomainName('');
        loadProfile();
      } else {
        alert(`Domain not available. Suggestions: ${(result.suggestions || []).join(', ')}`);
      }
    } catch (e) {
      console.error('Failed to register domain:', e);
    }
  }

  return (
    <div class="profile">
      <div class="card profile-header">
        <div class="avatar avatar-lg">{profile?.name?.[0] || '?'}</div>
        <div class="profile-info">
          <h2>{profile?.name || 'Anonymous'}</h2>
          <p class="profile-pubkey">{profile?.public_key || 'No key'}</p>
          <p class="profile-domain">{profile?.domain || 'No domain'}</p>
        </div>
      </div>

      <div class="card profile-section">
        <h3>Register Domain</h3>
        <div class="profile-register">
          <input
            class="input"
            placeholder="your-name"
            value={domainName}
            onInput={(e) => setDomainName((e.target as HTMLInputElement).value)}
          />
          <span class="profile-domain-suffix">.sovrn</span>
          <button class="btn btn-primary" onClick={handleRegisterDomain}>Register</button>
        </div>
      </div>

      {loading && <p>Loading profile...</p>}
    </div>
  );
}
