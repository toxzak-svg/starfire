"use client";

import { useState, useRef, useEffect } from "react";
import { Send, Sparkles } from "lucide-react";
import { sendMessage, getHealth } from "@/lib/api";

function Message({ role, text, time }) {
  return (
    <div className={`message ${role}`}>
      <div className="avatar">
        {role === "star" ? "★" : "Z"}
      </div>
      <div className="bubble">{text}</div>
      <div className="meta">
        {role === "star" ? "Star" : "Zach"} · {time}
      </div>
    </div>
  );
}

function TypingIndicator() {
  return (
    <div className="message star">
      <div className="avatar">★</div>
      <div className="bubble">
        <div className="typing">
          <span /><span /><span />
        </div>
      </div>
    </div>
  );
}

export default function ChatPage() {
  const [messages, setMessages] = useState([]);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const [connected, setConnected] = useState(null);
  const messagesEndRef = useRef(null);
  const textareaRef = useRef(null);

  useEffect(() => {
    getHealth()
      .then(() => setConnected(true))
      .catch(() => setConnected(false));
  }, []);

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
      const starMsg = {
        role: "star",
        text: res.response || res.message || "I'm here.",
        time: new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }),
      };
      setMessages(prev => [...prev, starMsg]);
    } catch (err) {
      const errMsg = {
        role: "star",
        text: "I couldn't reach myself just now. Is the server still awake?",
        time: new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }),
      };
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

  return (
    <div className="chat-container">
      {/* Header */}
      <header className="header">
        <div className="header-brand">
          <div className="star-icon">★</div>
          <h1>Star</h1>
        </div>
        <div className="header-status">
          <div className="dot" style={connected === false ? { background: "#ef4444", boxShadow: "none" } : {}} />
          {connected === null ? "connecting..." : connected ? "online" : "offline"}
        </div>
      </header>

      {/* Messages */}
      <div className="messages">
        {messages.length === 0 && (
          <div className="empty-state">
            <div className="empty-icon">★</div>
            <p>Star is awake. She remembers everything you've ever talked about.</p>
          </div>
        )}
        {messages.map((m, i) => (
          <Message key={i} role={m.role} text={m.text} time={m.time} />
        ))}
        {loading && <TypingIndicator />}
        <div ref={messagesEndRef} />
      </div>

      {/* Input */}
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
    </div>
  );
}
