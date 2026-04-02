// Star API client
const STAR_API = process.env.NEXT_PUBLIC_STAR_API || "https://star-production-15a1.up.railway.app";

export async function sendMessage(message, history = []) {
  const res = await fetch(`${STAR_API}/chat`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ message, history }),
  });
  if (!res.ok) throw new Error(`Star API error: ${res.status}`);
  return res.json();
}

export async function getHealth() {
  const res = await fetch(`${STAR_API}/health`);
  if (!res.ok) throw new Error("not ok");
  return res.json();
}

export async function getIdentity() {
  const res = await fetch(`${STAR_API}/identity`);
  if (!res.ok) throw new Error("not ok");
  return res.json();
}

export async function getCognitive() {
  const res = await fetch(`${STAR_API}/cognitive`);
  if (!res.ok) throw new Error("not ok");
  return res.json();
}

export async function getMetacog() {
  const res = await fetch(`${STAR_API}/metacog`);
  if (!res.ok) throw new Error("not ok");
  return res.json();
}

export async function getMemoryStats() {
  const res = await fetch(`${STAR_API}/memory/stats`);
  if (!res.ok) throw new Error("not ok");
  return res.json();
}
