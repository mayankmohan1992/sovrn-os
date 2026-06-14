import { useState, useEffect } from 'preact/hooks';
import { api, wsConnect, wsOn } from '../lib/api';

interface Conversation {
  peer_id: string;
  last_message_at: number;
  unread_count: number;
}

interface Message {
  id: string;
  from_id: string;
  to_id: string;
  content: string;
  created_at: number;
  delivery_status: string;
}

export function Messages({ peerId }: { peerId?: string }) {
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [messages, setMessages] = useState<Message[]>([]);
  const [selectedPeer, setSelectedPeer] = useState(peerId || '');
  const [newMessage, setNewMessage] = useState('');

  useEffect(() => {
    loadConversations();
    wsConnect();
    const unsub = wsOn('messages.update', () => {
      loadConversations();
      if (selectedPeer) loadMessages(selectedPeer);
    });
    return () => { unsub(); };
  }, []);

  useEffect(() => {
    if (selectedPeer) loadMessages(selectedPeer);
  }, [selectedPeer]);

  async function loadConversations() {
    try {
      const data = await api.getConversations();
      setConversations(data.conversations || []);
    } catch (e) {
      console.error('Failed to load conversations:', e);
    }
  }

  async function loadMessages(peer: string) {
    try {
      const data = await api.getMessages(peer);
      setMessages(data.messages || []);
    } catch (e) {
      console.error('Failed to load messages:', e);
    }
  }

  async function handleSend() {
    if (!newMessage.trim() || !selectedPeer) return;
    try {
      await api.sendMessage(selectedPeer, newMessage.trim());
      setNewMessage('');
      loadMessages(selectedPeer);
    } catch (e) {
      console.error('Failed to send message:', e);
    }
  }

  return (
    <div class="messages-layout">
      <div class="messages-sidebar card">
        <h3>Messages</h3>
        {conversations.length === 0 ? (
          <p class="messages-empty">No conversations yet</p>
        ) : (
          conversations.map(conv => (
            <div
              class={`messages-item ${conv.peer_id === selectedPeer ? 'messages-item--active' : ''}`}
              onClick={() => setSelectedPeer(conv.peer_id)}
            >
              <div class="avatar avatar-sm">{conv.peer_id[0]}</div>
              <div class="messages-item-info">
                <span class="messages-item-peer">{conv.peer_id.slice(0, 16)}...</span>
              </div>
              {conv.unread_count > 0 && (
                <span class="messages-badge">{conv.unread_count}</span>
              )}
            </div>
          ))
        )}
      </div>
      <div class="messages-main card">
        {selectedPeer ? (
          <>
            <div class="messages-header">
              <span>{selectedPeer.slice(0, 16)}...</span>
            </div>
            <div class="messages-list">
              {messages.map(msg => (
                <div class={`message ${msg.from_id === 'me' ? 'message--sent' : 'message--received'}`} key={msg.id}>
                  <p>{msg.content}</p>
                  <span class="message-time">{new Date(msg.created_at * 1000).toLocaleTimeString()}</span>
                </div>
              ))}
            </div>
            <div class="messages-compose">
              <input
                class="input"
                placeholder="Type a message..."
                value={newMessage}
                onInput={(e) => setNewMessage((e.target as HTMLInputElement).value)}
                onKeyDown={(e) => e.key === 'Enter' && handleSend()}
              />
              <button class="btn btn-primary" onClick={handleSend}>Send</button>
            </div>
          </>
        ) : (
          <div class="messages-empty-main">Select a conversation</div>
        )}
      </div>
    </div>
  );
}
