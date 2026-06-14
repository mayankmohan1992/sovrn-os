import { useState } from 'preact/hooks';
import { authToken, currentUserId } from '../App';
import { api } from '../lib/api';

export function Login() {
  const [seedPhrase, setSeedPhrase] = useState('');
  const [mode, setMode] = useState<'login' | 'create'>('login');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  async function handleLogin() {
    if (!seedPhrase.trim()) return;
    setLoading(true);
    setError('');
    try {
      const result = await api.login(seedPhrase.trim());
      authToken.value = result.token;
      currentUserId.value = result.profile?.public_key || '';
      localStorage.setItem('sovrn_token', result.token);
    } catch (e: any) {
      setError(e.message || 'Login failed');
    } finally {
      setLoading(false);
    }
  }

  async function handleCreate() {
    setLoading(true);
    setError('');
    try {
      const result = await api.login('new');
      authToken.value = result.token;
      currentUserId.value = result.profile?.public_key || '';
      localStorage.setItem('sovrn_token', result.token);
    } catch (e: any) {
      setError(e.message || 'Failed to create identity');
    } finally {
      setLoading(false);
    }
  }

  return (
    <div class="login">
      <div class="login-card card">
        <div class="login-logo">⬡</div>
        <h1>Sovrn</h1>
        <p class="login-tagline">Your mesh. Your rules.</p>

        {error && <div class="login-error">{error}</div>}

        {mode === 'login' ? (
          <div class="login-form">
            <label class="login-label">Enter your seed phrase</label>
            <textarea
              class="input login-seed"
              placeholder="your seed phrase words here..."
              value={seedPhrase}
              onInput={(e) => setSeedPhrase((e.target as HTMLTextAreaElement).value)}
              rows={3}
            />
            <button class="btn btn-primary login-btn" onClick={handleLogin} disabled={loading}>
              {loading ? 'Signing in...' : 'Sign In'}
            </button>
            <p class="login-switch">
              New to Sovrn? <a href="#" onClick={(e) => { e.preventDefault(); setMode('create'); }}>Create an identity</a>
            </p>
          </div>
        ) : (
          <div class="login-form">
            <p class="login-create-text">
              This will generate a new identity on the Sovrn mesh network.
              You'll receive a 24-word seed phrase — save it securely!
            </p>
            <button class="btn btn-primary login-btn" onClick={handleCreate} disabled={loading}>
              {loading ? 'Creating...' : 'Create New Identity'}
            </button>
            <p class="login-switch">
              Already have an identity? <a href="#" onClick={(e) => { e.preventDefault(); setMode('login'); }}>Sign in</a>
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
