"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import {
  Activity,
  Check,
  Copy,
  Eye,
  EyeOff,
  Gauge,
  RotateCcw,
  SlidersHorizontal,
} from "lucide-react";
import { humanize, processTraceSteps } from "./utils";

const SPEEDS = { slow: 34, normal: 18, fast: 7 };

function Typewriter({ text, enabled, speed }) {
  const [visible, setVisible] = useState(enabled ? 0 : text.length);

  useEffect(() => {
    if (!enabled) {
      setVisible(text.length);
      return undefined;
    }

    setVisible(0);
    const chunk = text.length > 1200 ? 8 : text.length > 500 ? 4 : 2;
    const timer = window.setInterval(() => {
      setVisible((current) => {
        const next = Math.min(text.length, current + chunk);
        if (next >= text.length) window.clearInterval(timer);
        return next;
      });
    }, SPEEDS[speed] ?? SPEEDS.normal);
    return () => window.clearInterval(timer);
  }, [enabled, speed, text]);

  return (
    <>
      {text.slice(0, visible)}
      {visible < text.length && <span className="console-caret">▍</span>}
    </>
  );
}

function ProcessTrace({ live, mode, animate }) {
  const steps = useMemo(() => processTraceSteps(live, mode), [live, mode]);
  const [visible, setVisible] = useState(animate ? 0 : steps.length);

  useEffect(() => {
    if (!animate) {
      setVisible(steps.length);
      return undefined;
    }

    setVisible(0);
    const timer = window.setInterval(() => {
      setVisible((current) => {
        const next = Math.min(steps.length, current + 1);
        if (next >= steps.length) window.clearInterval(timer);
        return next;
      });
    }, 250);
    return () => window.clearInterval(timer);
  }, [animate, steps]);

  if (!steps.length) return null;
  return (
    <div className="console-trace">
      <div
        className="console-trace-head"
        title="Structured metadata summaries, not hidden chain-of-thought."
      >
        <span>
          <Activity size={12} /> process trace
        </span>
        <small>summary, not private chain-of-thought</small>
      </div>
      <div className="console-trace-list">
        {steps.slice(0, visible).map((step, index) => (
          <div className="console-trace-step" key={`${step.title}-${index}`}>
            <b>{index + 1}</b>
            <span>
              <strong>{step.title}</strong>
              <small>{step.detail}</small>
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}

export function ConsoleMessage({ message, settings, latest, onRegenerate }) {
  const [copied, setCopied] = useState(false);
  const copy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(message.text);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1000);
    } catch {
      setCopied(false);
    }
  }, [message.text]);

  return (
    <article className={`console-message ${message.role}`}>
      <div className="console-avatar">
        {message.role === "star" ? "★" : "Z"}
      </div>
      <div className="console-bubble">
        {message.role === "star" ? (
          <Typewriter
            text={message.text}
            enabled={settings.stream && message.animate}
            speed={settings.speed}
          />
        ) : (
          message.text
        )}
      </div>
      <div className="console-message-footer">
        <span>
          {message.role === "star" ? "Star" : "Zach"} · {message.time}
        </span>
        <span className="console-message-actions">
          <button type="button" onClick={copy} title="Copy message">
            {copied ? <Check size={12} /> : <Copy size={12} />}
          </button>
          {latest && message.role === "star" && message.prompt && (
            <button type="button" onClick={onRegenerate} title="Regenerate">
              <RotateCcw size={12} />
            </button>
          )}
        </span>
      </div>
      {message.role === "star" && message.live && (
        <div className={`console-live ${message.live.enabled ? "on" : "off"}`}>
          <i />
          {message.live.enabled
            ? `live · turn ${message.live.turn ?? "?"} · ${humanize(message.live.intent, "planned response")}`
            : "legacy fallback"}
        </div>
      )}
      {message.role === "star" && settings.traceMode !== "off" && (
        <ProcessTrace
          live={message.live}
          mode={settings.traceMode}
          animate={settings.stream && message.animate}
        />
      )}
    </article>
  );
}

export function ConsoleControls({ settings, onChange }) {
  const update = (patch) => onChange({ ...settings, ...patch });
  return (
    <section className="console-controls">
      <div className="console-control-row">
        <span>
          <Eye size={13} /> Process trace
        </span>
        <div className="console-segments">
          {["off", "summary", "detailed"].map((value) => (
            <button
              key={value}
              type="button"
              className={settings.traceMode === value ? "active" : ""}
              onClick={() => update({ traceMode: value })}
            >
              {value}
            </button>
          ))}
        </div>
      </div>
      <div className="console-control-row">
        <span>
          {settings.stream ? <Gauge size={13} /> : <EyeOff size={13} />} Stream
          reply
        </span>
        <label className="console-switch">
          <input
            type="checkbox"
            checked={settings.stream}
            onChange={(event) => update({ stream: event.target.checked })}
          />
          <i />
        </label>
      </div>
      <div className="console-control-row">
        <span>
          <Gauge size={13} /> Stream speed
        </span>
        <div className="console-segments">
          {["slow", "normal", "fast"].map((value) => (
            <button
              key={value}
              type="button"
              disabled={!settings.stream}
              className={settings.speed === value ? "active" : ""}
              onClick={() => update({ speed: value })}
            >
              {value}
            </button>
          ))}
        </div>
      </div>
      <div className="console-control-row">
        <span>
          <SlidersHorizontal size={13} /> Remember chat
        </span>
        <label className="console-switch">
          <input
            type="checkbox"
            checked={settings.persist}
            onChange={(event) => update({ persist: event.target.checked })}
          />
          <i />
        </label>
      </div>
    </section>
  );
}
