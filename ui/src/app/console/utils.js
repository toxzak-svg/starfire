export const STORAGE_KEY = "starfire-console-v1";
export const DEFAULT_SETTINGS = {
  stream: true,
  speed: "normal",
  persist: true,
};

export function humanize(value, fallback = "unknown") {
  if (value === null || value === undefined || value === "") return fallback;
  return String(value)
    .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
    .replace(/[_-]+/g, " ")
    .trim()
    .toLowerCase();
}

function displayValue(value) {
  if (value === null || value === undefined) return "unknown";
  if (typeof value === "number") {
    return Number.isInteger(value) ? String(value) : value.toFixed(2);
  }
  if (typeof value === "string") return humanize(value);
  if (typeof value === "object") {
    return humanize(
      value.level || value.kind || value.value || value.label,
      "typed",
    );
  }
  return String(value);
}

export function currentTime() {
  return new Date().toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function restoreConsole() {
  if (typeof window === "undefined") {
    return { messages: [], settings: DEFAULT_SETTINGS };
  }

  try {
    const parsed = JSON.parse(
      window.localStorage.getItem(STORAGE_KEY) || "null",
    );
    return {
      messages: Array.isArray(parsed?.messages)
        ? parsed.messages.slice(-100).map((message) => ({
            ...message,
            animate: false,
          }))
        : [],
      settings: { ...DEFAULT_SETTINGS, ...(parsed?.settings || {}) },
    };
  } catch {
    return { messages: [], settings: DEFAULT_SETTINGS };
  }
}

export function processTraceSteps(live, mode) {
  if (!live) return [];
  if (live.enabled !== true) {
    return [
      {
        title: "Live layer unavailable",
        detail: live.reason || live.error || "Protected runtime fallback",
      },
    ];
  }

  const plan = live.semantic_plan || {};
  const operations = Array.isArray(plan.operations) ? plan.operations : [];
  const prohibited = Array.isArray(plan.prohibited_implications)
    ? plan.prohibited_implications
    : [];
  const summary = [
    {
      title: "Interpreted request",
      detail: `${humanize(live.intent, "planned response")} intent`,
    },
    {
      title: "Built typed response plan",
      detail: `${operations.length} operation${operations.length === 1 ? "" : "s"} · ${displayValue(plan.detail_budget)} detail`,
    },
    {
      title: "Rendered reply",
      detail: `${humanize(live.pipeline, "live pipeline")} · turn ${live.turn ?? "?"}`,
    },
  ];

  if (mode !== "detailed") return summary;
  return [
    summary[0],
    {
      title: "Selected stance",
      detail: `${displayValue(plan.stance)} · confidence ${displayValue(plan.confidence)}`,
    },
    summary[1],
    {
      title: "Applied constraints",
      detail: `${prohibited.length} prohibited implication${prohibited.length === 1 ? "" : "s"} checked`,
    },
    {
      title: "Committed bounded voice revision",
      detail: live.voice_after
        ? `Persistent state advanced to turn ${live.turn ?? "?"}`
        : "No projection returned",
    },
    summary[2],
  ];
}

export function compactLive(live) {
  if (!live) return null;
  const plan = live.semantic_plan || {};
  return {
    enabled: live.enabled === true,
    pipeline: live.pipeline,
    trace_id: live.trace_id,
    turn: live.turn,
    intent: live.intent,
    reason: live.reason,
    error: live.error,
    voice_after: live.voice_after ? { present: true } : null,
    semantic_plan: {
      intent: plan.intent,
      operations: Array.isArray(plan.operations) ? plan.operations : [],
      detail_budget: plan.detail_budget,
      stance: plan.stance,
      confidence: plan.confidence,
      prohibited_implications: Array.isArray(plan.prohibited_implications)
        ? plan.prohibited_implications
        : [],
    },
  };
}
