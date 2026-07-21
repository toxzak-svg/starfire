"use client";

import "./console.css";
import Link from "next/link";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  Download,
  Eye,
  EyeOff,
  Send,
  SlidersHorizontal,
  Square,
  Trash2,
} from "lucide-react";
import { getHealth, sendMessage } from "@/lib/api";
import { ConsoleControls, ConsoleMessage } from "./components";
import {
  compactLive,
  currentTime,
  DEFAULT_SETTINGS,
  restoreConsole,
  STORAGE_KEY,
} from "./utils";

export default function ConsolePage() {
  const [messages, setMessages] = useState([]);
  const [settings, setSettings] = useState(DEFAULT_SETTINGS);
  const [restored, setRestored] = useState(false);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const [connected, setConnected] = useState(null);
  const [showControls, setShowControls] = useState(true);
  const abortRef = useRef(null);
  const endRef = useRef(null);

  useEffect(() => {
    const saved = restoreConsole();
    setMessages(saved.messages);
    setSettings(saved.settings);
    setRestored(true);
  }, []);

  useEffect(() => {
    getHealth()
      .then(() => setConnected(true))
      .catch(() => setConnected(false));
  }, []);

  useEffect(() => {
    if (!restored || typeof window === "undefined") return;
    if (!settings.persist) {
      window.localStorage.removeItem(STORAGE_KEY);
      return;
    }
    try {
      window.localStorage.setItem(
        STORAGE_KEY,
        JSON.stringify({
          messages: messages.slice(-100).map((message) => ({
            ...message,
            animate: false,
            live: compactLive(message.live),
          })),
          settings,
        }),
      );
    } catch {
      // Browser storage is optional.
    }
  }, [messages, restored, settings]);

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, loading]);

  const latestStarIndex = useMemo(() => {
    for (let index = messages.length - 1; index >= 0; index -= 1) {
      if (messages[index].role === "star") return index;
    }
    return -1;
  }, [messages]);

  const run = useCallback(
    async (text, appendUser = true, baseMessages = messages) => {
      const clean = text.trim();
      if (!clean || loading) return;

      const user = {
        id: `u-${Date.now()}`,
        role: "user",
        text: clean,
        time: currentTime(),
      };
      const source = appendUser ? [...baseMessages, user] : baseMessages;
      if (appendUser) setMessages((current) => [...current, user]);
      setInput("");
      setLoading(true);
      const controller = new AbortController();
      abortRef.current = controller;

      try {
        const history = source.map((message) => ({
          role: message.role === "user" ? "user" : "assistant",
          content: message.text,
        }));
        const response = await sendMessage(clean, history, {
          signal: controller.signal,
        });
        setConnected(true);
        setMessages((current) => [
          ...current,
          {
            id: `s-${Date.now()}`,
            role: "star",
            text: response.response || response.message || "I'm here.",
            time: currentTime(),
            live: response.live || null,
            animate: settings.stream,
            prompt: clean,
          },
        ]);
      } catch (error) {
        setMessages((current) => [
          ...current,
          {
            id: `e-${Date.now()}`,
            role: "star",
            text:
              error?.name === "AbortError"
                ? "Stopped locally. The server may still finish, but this browser discarded the result."
                : "I couldn't reach myself just now. Is the server still awake?",
            time: currentTime(),
            live: null,
            animate: false,
          },
        ]);
        if (error?.name !== "AbortError") setConnected(false);
      } finally {
        abortRef.current = null;
        setLoading(false);
      }
    },
    [loading, messages, settings.stream],
  );

  function submit(event) {
    event.preventDefault();
    run(input);
  }

  function keyDown(event) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      run(input);
    }
    if (event.key === "Escape" && loading) abortRef.current?.abort();
  }

  function regenerate() {
    const latest = messages[latestStarIndex];
    if (!latest?.prompt || loading) return;
    const baseMessages = messages.filter(
      (_, index) => index !== latestStarIndex,
    );
    setMessages(baseMessages);
    run(latest.prompt, false, baseMessages);
  }

  function clear() {
    setMessages([]);
    if (typeof window !== "undefined") {
      window.localStorage.removeItem(STORAGE_KEY);
    }
  }

  function exportTranscript() {
    if (!messages.length || typeof window === "undefined") return;
    const text = messages
      .map(
        (message) =>
          `${message.role === "star" ? "Star" : "Zach"} [${message.time}]\n${message.text}`,
      )
      .join("\n\n");
    const url = URL.createObjectURL(
      new Blob([text], { type: "text/plain;charset=utf-8" }),
    );
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = `starfire-chat-${new Date().toISOString().slice(0, 10)}.txt`;
    anchor.click();
    URL.revokeObjectURL(url);
  }

  return (
    <main className="console-shell">
      <header className="console-header">
        <div className="console-brand">
          <b>★</b>
          <span>
            <strong>Starfire Console</strong>
            <small>inspectable live chat</small>
          </span>
        </div>
        <div className="console-header-actions">
          <span
            className={`console-status ${connected === false ? "offline" : ""}`}
          >
            <i />
            {connected === null ? "checking" : connected ? "online" : "offline"}
          </span>
          <button
            type="button"
            onClick={() => setShowControls((value) => !value)}
            title="Toggle controls"
          >
            <SlidersHorizontal size={15} />
          </button>
          <Link href="/">classic UI</Link>
        </div>
      </header>

      {showControls && (
        <ConsoleControls settings={settings} onChange={setSettings} />
      )}

      <div className="console-toolbar">
        <span>
          {settings.traceMode === "off" ? (
            <EyeOff size={12} />
          ) : (
            <Eye size={12} />
          )}
          trace {settings.traceMode} · stream{" "}
          {settings.stream ? settings.speed : "off"}
        </span>
        <span>
          <button
            type="button"
            disabled={!messages.length}
            onClick={exportTranscript}
            title="Export transcript"
          >
            <Download size={13} />
          </button>
          <button
            type="button"
            disabled={!messages.length || loading}
            onClick={clear}
            title="Clear chat"
          >
            <Trash2 size={13} />
          </button>
        </span>
      </div>

      <section className="console-messages">
        {!messages.length && (
          <div className="console-empty">
            <b>★</b>
            <p>
              Star is awake. This console streams completed replies and exposes
              a structured process summary from the live typed metadata.
            </p>
          </div>
        )}
        {messages.map((message, index) => (
          <ConsoleMessage
            key={message.id || index}
            message={message}
            settings={settings}
            latest={index === latestStarIndex}
            onRegenerate={regenerate}
          />
        ))}
        {loading && (
          <div className="console-thinking">
            <b>★</b>
            <span>
              <i />
              <i />
              <i />
            </span>
            <small>running protected cognition and typed rendering</small>
          </div>
        )}
        <div ref={endRef} />
      </section>

      <form className="console-input" onSubmit={submit}>
        <textarea
          rows={1}
          value={input}
          onChange={(event) => setInput(event.target.value)}
          onKeyDown={keyDown}
          placeholder="Say something..."
        />
        {loading ? (
          <button
            type="button"
            className="stop"
            onClick={() => abortRef.current?.abort()}
            title="Stop locally (Esc)"
          >
            <Square size={15} fill="currentColor" />
          </button>
        ) : (
          <button type="submit" disabled={!input.trim()}>
            <Send size={16} />
          </button>
        )}
        <small>Enter sends · Shift+Enter adds a line · Esc stops locally</small>
      </form>
    </main>
  );
}
