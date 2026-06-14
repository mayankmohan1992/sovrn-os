import { useState, useEffect } from 'preact/hooks';
import { api, wsConnect, wsOn } from '../lib/api';

interface FeedEvent {
  id: string;
  kind: number;
  author: string;
  content: string;
  created_at: number;
  sig: string;
}

export function Timeline() {
  const [events, setEvents] = useState<FeedEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [newPost, setNewPost] = useState('');

  useEffect(() => {
    loadTimeline();
    wsConnect();
    const unsub = wsOn('feed.update', () => loadTimeline());
    return () => { unsub(); };
  }, []);

  async function loadTimeline() {
    try {
      const data = await api.getTimeline();
      setEvents(data.events || []);
    } catch (e) {
      console.error('Failed to load timeline:', e);
    } finally {
      setLoading(false);
    }
  }

  async function handlePost() {
    if (!newPost.trim()) return;
    try {
      await api.createEvent({
        kind: 1,
        content: newPost.trim(),
        author: 'me',
      });
      setNewPost('');
      loadTimeline();
    } catch (e) {
      console.error('Failed to create post:', e);
    }
  }

  function timeAgo(ts: number) {
    const diff = Math.floor(Date.now() / 1000) - ts;
    if (diff < 60) return 'just now';
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    return `${Math.floor(diff / 86400)}d ago`;
  }

  return (
    <div class="timeline">
      <div class="timeline-compose card">
        <textarea
          class="input"
          placeholder="What's happening on the mesh?"
          value={newPost}
          onInput={(e) => setNewPost((e.target as HTMLTextAreaElement).value)}
          rows={3}
        />
        <div class="timeline-compose-actions">
          <button class="btn btn-primary" onClick={handlePost} disabled={!newPost.trim()}>
            Post
          </button>
        </div>
      </div>

      {loading ? (
        <div class="timeline-empty">Loading timeline...</div>
      ) : events.length === 0 ? (
        <div class="timeline-empty">
          <p>No posts yet. Be the first to post on the mesh!</p>
        </div>
      ) : (
        <div class="timeline-events">
          {events.map(event => (
            <article class="card timeline-event" key={event.id}>
              <div class="timeline-event-header">
                <span class="timeline-event-author">{event.author.slice(0, 12)}...</span>
                <time class="timeline-event-time">{timeAgo(event.created_at)}</time>
              </div>
              <p class="timeline-event-content">{event.content}</p>
            </article>
          ))}
        </div>
      )}
    </div>
  );
}
