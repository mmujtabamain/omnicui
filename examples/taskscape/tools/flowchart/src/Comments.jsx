import React, { useState } from 'react';

const readName = () => {
  try { return localStorage.getItem('flow.author') ?? ''; } catch { return ''; }
};
const saveName = (n) => {
  try { localStorage.setItem('flow.author', n); } catch {}
};
const ago = (t) => {
  const m = Math.round((Date.now() - t) / 60000);
  return m < 1 ? 'just now' : m < 60 ? `${m}m ago` : m < 1440 ? `${Math.round(m / 60)}h ago` : `${Math.round(m / 1440)}d ago`;
};

export function Comments({ node, comments, reload }) {
  const [author, setAuthor] = useState(readName);
  const [tag, setTag] = useState('general');
  const [text, setText] = useState('');
  const [replyTo, setReplyTo] = useState(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const here = comments.filter((c) => c.node === node);
  const roots = here.filter((c) => !c.parent);
  const replies = (id) => here.filter((c) => c.parent === id);

  async function post(e) {
    e.preventDefault();
    if (!text.trim()) return;
    setBusy(true);
    setError('');
    saveName(author);
    const r = await fetch('api/comments', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ node, author, tag, text, parent: replyTo }),
    }).catch(() => null);
    setBusy(false);
    if (!r || !r.ok) return setError('Could not save the comment. Try again.');
    setText('');
    setReplyTo(null);
    reload();
  }
  async function resolve(c) {
    await fetch(`api/comments/${c.id}`, { method: 'PATCH', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ resolved: !c.resolved }) });
    reload();
  }
  const one = (c, depth = 0) => (
    <li key={c.id} className={`comment ${c.resolved ? 'resolved' : ''}`} style={{ marginLeft: depth * 14 }}>
      <div className="meta">
        <b>{c.author}</b> <span className={`chip ${c.tag}`}>{c.tag}</span> <span className="muted">{ago(c.created_at)}</span>
      </div>
      <div className="text">{c.text}</div>
      <div className="acts">
        <button className="link" onClick={() => setReplyTo(c.id)}>Reply</button>
        {!c.parent && <button className="link" onClick={() => resolve(c)}>{c.resolved ? 'Reopen' : 'Resolve'}</button>}
      </div>
      {replies(c.id).length > 0 && <ul>{replies(c.id).map((r) => one(r, depth + 1))}</ul>}
    </li>
  );

  return (
    <section className="comments">
      <h3>Comments on <code>{node}</code> ({roots.filter((c) => !c.resolved).length} open)</h3>
      <ul>{roots.map((c) => one(c))}</ul>
      <form onSubmit={post}>
        {replyTo && <div className="muted small">Replying · <button type="button" className="link" onClick={() => setReplyTo(null)}>cancel</button></div>}
        <div className="row">
          <input placeholder="Your name" value={author} onChange={(e) => setAuthor(e.target.value)} />
          <select value={tag} onChange={(e) => setTag(e.target.value)} title="Which version this is about">
            <option value="general">general</option>
            <option value="web">web</option>
            <option value="desktop">desktop</option>
            <option value="both">both</option>
          </select>
        </div>
        <textarea placeholder="Add a comment…" value={text} onChange={(e) => setText(e.target.value)} rows={3} />
        {error && <div className="error">{error}</div>}
        <button type="submit" disabled={busy || !text.trim()}>{busy ? 'Saving…' : 'Comment'}</button>
      </form>
    </section>
  );
}
