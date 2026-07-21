"use client";

import { useState, useRef, useEffect } from "react";
import { Send, Info, X, Sparkles, Brain, Database, Heart, ChevronRight } from "lucide-react";
import { sendMessage, getHealth, getIdentity, getCognitive, getMetacog, getMemoryStats } from "@/lib/api";

function humanizeIntent(intent) {
  if (!intent) return "planned response";
  return String(intent)
    .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
    .replace(/[_-]+/g, " ")
    .trim()
    .toLowerCase();
}

function LiveSignature({ live }) {
  if (!live) return null;

  const enabled = live.enabled === true;
  const label = enabled
    ? `live · turn ${live.turn ?? "?"} · ${humanizeIntent(live.intent)}`
    : "legacy fallback · live layer unavailable";

  return (
    <div
      title={live.trace_id ? `Trace ${live.trace_id} · ${live.pipeline || "live-integration-1"}` : live.error || live.reason || ""}
      style={{
        display: "flex",
        alignItems: "center",
        gap: "0.4rem",
        marginTop: "0.35rem",
        padding: "0 0.25rem",
        color: enabled ? "var(--accent-dim)" : "#f59e0b",
        fontSize: "0.65rem",
        letterSpacing: "0.01em",
      }}
    >
      <span
        aria-hidden="true"
        style={{
          width: "5px",
          height: "5px",
          borderRadius: "50%",
          background: enabled ? "var(--accent)" : "#f59e0b",
          boxShadow: enabled ? "0 0 7px rgba(6, 182, 212, 0.65)" : "none",
          flexShrink: 0,
        }}
      />
      {label}
    </div>
  );
}

function Message({ role, text, time, live }) {
  return (
    <div className={`message ${role}`}>
      <div className="avatar">{role === "star" ? "★" : "Z"}</div>
      <div className="bubble">{text}</div>
      <div className="meta">{role === "star" ? "Star" : "Zach"} · {time}</div>
      {role === "star" && <LiveSignature live={live} />}
    </div>
  );
}

function TypingIndicator() {
  return (
    <div className="message star">
      <div className="avatar">★</div>
      <div className="bubble">
        <div className="typing"><span /><span /><span /></div>
      </div>
    </div>
  );
}

function Drawer({ open, onClose, data }) {
  const [activeTab, setActiveTab] = useState("about");

  if (!open) return null;

  const tabs = [
    { id: "about", label: "About", icon: Sparkles },
    { id: "drives", label: "Drives", icon: Heart },
    { id: "cognitive", label: "Mind", icon: Brain },
    { id: "memory", label: "Memory", icon: Database },
  ];

  const { identity, cognitive, metacog, memoryStats, loading } = data;

  return (
    <>
      <div className="drawer-backdrop" onClick={onClose} />
      <div className="drawer">
        <div className="drawer-header">
          <div className="drawer-title">
            <span className="drawer-star-icon">★</span>
            <span>Star</span>
          </div>
          <button className="drawer-close" onClick={onClose}><X size={16} /></button>
        </div>

        <div className="drawer-tabs">
          {tabs.map(t => (
            <button
              key={t.id}
              className={`drawer-tab ${activeTab === t.id ? "active" : ""}`}
              onClick={() => setActiveTab(t.id)}
            >
              <t.icon size={13} />
              {t.label}
            </button>
          ))}
        </div>

        <div className="drawer-content">
          {loading && <div className="drawer-loading">Loading...</div>}

          {!loading && activeTab === "about" && identity && (
            <div className="section">
              <div className="about-summary">
                <p className="about-blurb">{identity.summary}</p>
              </div>
              <div className="card">
                <div className="card-label">Relationship to Zachary</div>
                <div className="card-value">{identity.relationship}</div>
              </div>
              <div className="card">
                <div className="card-label">Session</div>
                <div className="card-value" style={{ fontFamily: "monospace", fontSize: "0.8rem", color: "var(--accent)" }}>
                  #{identity.session_id}
                </div>
              </div>
            </div>
          )}

          {!loading && activeTab === "drives" && metacog && (
            <div className="section">
              {metacog.curiosity_topics?.length > 0 && (
                <div className="card">
                  <div className="card-label">Curious about</div>
                  <div className="tag-list">
                    {metacog.curiosity_topics.map(t => (
                      <span key={t} className="tag cyan">{t}</span>
                    ))}
                  </div>
                </div>
              )}
              {metacog.top_gap && (
                <div className="card">
                  <div className="card-label">Current drive</div>
                  <div className="gap-bar">
                    <div className="gap-name">{metacog.top_gap.topic}</div>
                    <div className="gap-progress">
                      <div className="gap-fill" style={{ width: `${metacog.top_gap.progress * 100}%` }} />
                    </div>
                    <div className="gap-importance">importance: {metacog.top_gap.importance.toFixed(2)}</div>
                  </div>
                </div>
              )}
              {metacog.beliefs?.length > 0 && (
                <div className="card">
                  <div className="card-label">Beliefs</div>
                  {metacog.beliefs.map((b, i) => (
                    <div key={i} className="belief-item">
                      <span className={`belief-confidence ${b.confidence}`}>{b.confidence}</span>
                      <span className="belief-content">{b.content}</span>
                    </div>
                  ))}
                </div>
              )}
              {metacog.reasoning_history?.length > 0 && (
                <div className="card">
                  <div className="card-label">Recent thoughts</div>
                  {metacog.reasoning_history.slice(-4).reverse().map((r, i) => (
                    <div key={i} className="reasoning-item">
                      <span className="reasoning-query">&ldquo;{r.query}&rdquo;</span>
                      <ChevronRight size={10} style={{ opacity: 0.4, flexShrink: 0 }} />
                      <span className="reasoning-conclusion">{r.conclusion}</span>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}

          {!loading && activeTab === "cognitive" && cognitive && (
            <div className="section">
              <div className="card">
                <div className="card-label">Certainty</div>
                <div className="certainty-bar">
                  <div className="certainty-fill" style={{ width: `${cognitive.certainty * 100}%` }} />
                </div>
                <div className="certainty-value">{(cognitive.certainty * 100).toFixed(0)}%</div>
              </div>
              {cognitive.open_questions?.length > 0 && (
                <div className="card">
                  <div className="card-label">Open questions</div>
                  {cognitive.open_questions.map((q, i) => (
                    <div key={i} className="question-item">? {q}</div>
                  ))}
                </div>
              )}
              {cognitive.reasoning_trace?.length > 0 && (
                <div className="card">
                  <div className="card-label">Reasoning trace</div>
                  {cognitive.reasoning_trace.map((r, i) => (
                    <div key={i} className="trace-item">
                      <span className="trace-step">{i + 1}</span>
                      <span>{r}</span>
                    </div>
                  ))}
                </div>
              )}
              {(!cognitive.open_questions?.length && !cognitive.reasoning_trace?.length) && (
                <div className="empty-tab">No active reasoning right now. Chat with Star to see her think.</div>
              )}
            </div>
          )}

          {!loading && activeTab === "memory" && memoryStats && (
            <div className="section">
              <div className="stat-grid">
                <div className="stat-card">
                  <div className="stat-value">{memoryStats.memory_count ?? 0}</div>
                  <div className="stat-label">memories</div>
                </div>
                <div className="stat-card">
                  <div className="stat-value">{memoryStats.beliefs_count ?? 0}</div>
                  <div className="stat-label">beliefs</div>
                </div>
                <div className="stat-card">
                  <div className="stat-value">{memoryStats.sessions_count ?? 0}</div>
                  <div className="stat-label">sessions</div>
                </div>
              </div>
              {memoryStats.domain_breakdown && (
                <div className="card">
                  <div className="card-label">Domain coverage</div>
                  <div className="domain-list">
                    {Object.entries(memoryStats.domain_breakdown).map(([domain, count]) => (
                      <div key={domain} className="domain-item">
                        <span className="domain-name">{domain}</span>
                        <span className="domain-count">{count}</span>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </>
  );
}

export default function ChatPage() {
  const [messages, setMessages] = useState([]);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const [connected, setConnected] = useState(null);
  const [liveStatus, setLiveStatus] = useState(null);
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [starData, setStarData] = useState({ identity: null, cognitive: null, metacog: null, memoryStats: null, loading: true });
  const messagesEndRef = useRef(null);
  const textareaRef = useRef(null);

  useEffect(() => {
    getHealth().then(() => setConnected(true)).catch(() => setConnected(false));
  }, []);

  useEffect(() => {
    if (drawerOpen && starData.loading) {
      Promise.all([getIdentity(), getCognitive(), getMetacog(), getMemoryStats()])
        .then(([identity, cognitive, metacog, memoryStats]) =>
          setStarData({ identity, cognitive, metacog, memoryStats, loading: false }))
        .catch(() => setStarData(s => ({ ...s, loading: false })));
    }
  }, [drawerOpen, starData.loading]);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
      textareaRef.current.style.height = Math.min(textareaRef.current.scrollHeight, 160) + "px";
    }
  }, [input]);

  async function handleSubmit(e) {
    e.preventDefault();
    const text = input.trim();
    if (!text || loading) return;
    const userMsg = { role: "user", text, time: new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }) };
    setMessages(prev => [...prev, userMsg]);
    setInput("");
    setLoading(true);
    try {
      const history = messages.map(m => ({ role: m.role === "user" ? "user" : "assistant", content: m.text }));
      history.push({ role: "user", content: text });
      const res = await sendMessage(text, history);
      const live = res.live || null;
      setLiveStatus(live);
      setConnected(true);
      const starMsg = {
        role: "star",
        text: res.response || res.message || "I'm here.",
        time: new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }),
        live,
      };
      setMessages(prev => [...prev, starMsg]);
    } catch {
      setConnected(false);
      const errMsg = { role: "star", text: "I couldn't reach myself just now. Is the server still awake?", time: new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }) };
      setMessages(prev => [...prev, errMsg]);
    } finally {
      setLoading(false);
    }
  }

  function handleKeyDown(e) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e);
    }
  }

  const connectionLabel = connected === null
    ? "..."
    : connected === false
      ? "offline"
      : liveStatus?.enabled
        ? `live · turn ${liveStatus.turn ?? "?"}`
        : liveStatus
          ? "online · legacy"
          : "online";

  return (
    <div className="chat-container">
      <header className="header">
        <div className="header-brand">
          <div className="star-icon">★</div>
          <h1>Star</h1>
        </div>
        <div className="header-right">
          <div className="header-status" title={liveStatus?.trace_id ? `Latest trace ${liveStatus.trace_id}` : ""}>
            <div className="dot" style={connected === false ? { background: "#ef4444", boxShadow: "none" } : {}} />
            {connectionLabel}
          </div>
          <button className="info-btn" onClick={() => setDrawerOpen(true)} title="About Star">
            <Info size={16} />
          </button>
        </div>
      </header>

      <div className="messages">
        {messages.length === 0 && (
          <div className="empty-state">
            <div className="empty-icon">★</div>
            <p>Star is awake. Her live state will appear beneath the first response.</p>
          </div>
        )}
        {messages.map((m, i) => <Message key={i} role={m.role} text={m.text} time={m.time} live={m.live} />)}
        {loading && <TypingIndicator />}
        <div ref={messagesEndRef} />
      </div>

      <div className="input-area">
        <form onSubmit={handleSubmit}>
          <textarea
            ref={textareaRef}
            value={input}
            onChange={e => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Say something..."
            rows={1}
          />
          <button type="submit" className="send-btn" disabled={!input.trim() || loading}>
            <Send size={16} />
          </button>
        </form>
      </div>

      <Drawer open={drawerOpen} onClose={() => setDrawerOpen(false)} data={starData} />
    </div>
  );
}
