// Star API client
// Railway URL — change this to your deployed URL or localhost for dev
const STAR_API = process.env.NEXT_PUBLIC_STAR_API || "https://star-production-6458.up.railway.app";

export async function sendMessage(message, history = []) {
  const res = await fetch(`${STAR_API}/chat`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ message, history }),
  });

  if (!res.ok) {
    throw new Error(`Star API error: ${res.status}`);
  }

  return res.json();
}

export async function getHealth() {
  const res = await fetch(`${STAR_API}/health`);
  return res.json();
}
