// Sovrn Hub API client — talks to sovrnd via REST + WebSocket

const API_BASE = '/api';
const WS_BASE = `ws${location.protocol === 'https:' ? 's' : ''}://${location.host}/ws`;

let ws: WebSocket | null = null;
let wsListeners: Map<string, Set<(data: any) => void>> = new Map();

// ── REST API ────────────────────────────────────────────────────
async function apiFetch<T = any>(path: string, options?: RequestInit): Promise<T> {
  const token = localStorage.getItem('sovrn_token');
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(token ? { Authorization: `Bearer ${token}` } : {}),
    ...(options?.headers as Record<string, string> || {}),
  };

  const res = await fetch(`${API_BASE}${path}`, {
    ...options,
    headers,
  });

  if (!res.ok) {
    const body = await res.text();
    throw new Error(`API error ${res.status}: ${body}`);
  }

  return res.json();
}

export const api = {
  // Auth
  login: (publicKey: string) =>
    apiFetch('/auth/login', { method: 'POST', body: JSON.stringify({ public_key: publicKey }) }),

  // DHT
  dhtLookup: (domain: string) =>
    apiFetch(`/dht/lookup/${domain}`),
  dhtRegister: (name: string, publicKey: string, signature: string) =>
    apiFetch('/dht/register', { method: 'POST', body: JSON.stringify({ name, public_key: publicKey, signature }) }),
  dhtCheck: (name: string) =>
    apiFetch(`/dht/check/${name}`),

  // Identity
  getProfile: (publicKey: string) =>
    apiFetch(`/identity/profile/${publicKey}`),
  updateProfile: (data: any) =>
    apiFetch('/identity/profile', { method: 'PUT', body: JSON.stringify(data) }),
  createAlias: (keyId: string, name: string) =>
    apiFetch('/identity/alias', { method: 'POST', body: JSON.stringify({ key_id: keyId, name }) }),
  deriveAppKey: (masterKey: string, appId: string) =>
    apiFetch('/identity/derive_app_key', { method: 'POST', body: JSON.stringify({ master_key: masterKey, app_id: appId }) }),

  // Presence
  getOnline: (limit = 50) =>
    apiFetch(`/presence/online?limit=${limit}`),
  setStatus: (status: string) =>
    apiFetch('/presence/status', { method: 'POST', body: JSON.stringify({ status }) }),

  // Feed
  getTimeline: (userId = 'default', limit = 50, cursor?: string) =>
    apiFetch(`/feed/timeline?user_id=${userId}&limit=${limit}${cursor ? `&cursor=${cursor}` : ''}`),
  getEvent: (eventId: string) =>
    apiFetch(`/feed/event/${eventId}`),
  createEvent: (data: any) =>
    apiFetch('/feed/create', { method: 'POST', body: JSON.stringify(data) }),
  deleteEvent: (eventId: string) =>
    apiFetch(`/feed/event/${eventId}`, { method: 'DELETE' }),
  addReaction: (eventId: string, kind: number) =>
    apiFetch('/feed/react', { method: 'POST', body: JSON.stringify({ event_id: eventId, kind }) }),
  searchFeed: (query: string, kind?: number) =>
    apiFetch(`/feed/search?q=${encodeURIComponent(query)}${kind ? `&kind=${kind}` : ''}`),

  // Messages
  getConversations: (userId = 'default') =>
    apiFetch(`/messages/conversations?user_id=${userId}`),
  getMessages: (peerId: string, cursor?: number) =>
    apiFetch(`/messages/${peerId}${cursor ? `?cursor=${cursor}` : ''}`),
  sendMessage: (to: string, content: string) =>
    apiFetch('/messages/send', { method: 'POST', body: JSON.stringify({ to_id: to, content }) }),
  markRead: (peerId: string) =>
    apiFetch(`/messages/mark_read/${peerId}`, { method: 'POST', body: JSON.stringify({}) }),

  // CDN
  cdnPush: (cid: string, localPath: string) =>
    apiFetch('/cdn/push', { method: 'POST', body: JSON.stringify({ cid, local_path: localPath }) }),
  cdnStatus: (pushId: string) =>
    apiFetch(`/cdn/status/${pushId}`),

  // Directory
  getDirectoryUsers: () =>
    apiFetch('/identity/directory/users'),
  getDirectoryDomains: () =>
    apiFetch('/identity/directory/domains'),

  // Health
  health: () => apiFetch('/health'),
};

// ── WebSocket ──────────────────────────────────────────────────
export function wsConnect() {
  if (ws?.readyState === WebSocket.OPEN) return;

  ws = new WebSocket(WS_BASE);

  ws.onopen = () => {
    console.log('[sovrn] WebSocket connected');
    // Subscribe to all channels
    ws?.send(JSON.stringify({ type: 'subscribe', channels: ['feed', 'messages', 'presence'] }));
  };

  ws.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data);
      const type = data.type;
      const listeners = wsListeners.get(type);
      if (listeners) {
        listeners.forEach(fn => fn(data.data || data));
      }
      // Also notify wildcard listeners
      const wildcard = wsListeners.get('*');
      if (wildcard) {
        wildcard.forEach(fn => fn(data));
      }
    } catch (e) {
      console.error('[sovrn] WebSocket parse error:', e);
    }
  };

  ws.onclose = () => {
    console.log('[sovrn] WebSocket closed, reconnecting in 5s...');
    setTimeout(wsConnect, 5000);
  };

  ws.onerror = (e) => {
    console.error('[sovrn] WebSocket error:', e);
  };
}

export function wsOn(type: string, callback: (data: any) => void) {
  if (!wsListeners.has(type)) {
    wsListeners.set(type, new Set());
  }
  wsListeners.get(type)!.add(callback);
  return () => wsListeners.get(type)?.delete(callback);
}

export function wsSend(type: string, params: any = {}) {
  if (ws?.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify({ type, params }));
  }
}
