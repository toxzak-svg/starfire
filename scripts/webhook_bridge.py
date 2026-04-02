#!/usr/bin/env python3
"""
Telegram webhook bridge for Star.
Receives Telegram webhooks, preprocesses questions, forwards to Star, sends responses back.

Key behaviors:
- Transforms "do you know what X is?" → "what is X?" for better memory retrieval
- Maintains persistent KNOWLEDGE_FILE of researched topics
- When Star doesn't know a researched topic, seeds the knowledge via Star's conversation
  so she can reason about it (not just parrot the answer)
- Fires background research for unknown topics
"""

import os
import sys
import json
import asyncio
import re
import logging
from aiohttp import web
import aiohttp

logging.basicConfig(level=logging.INFO, format='%(asctime)s [Bridge] %(message)s', stream=sys.stdout)
log = logging.getLogger(__name__)

STAR_API_URL = os.environ.get("STAR_API_URL", "http://localhost:8080")
TELEGRAM_TOKEN = os.environ.get("TELEGRAM_BOT_TOKEN", "")
PORT = int(os.environ.get("WEBHOOK_PORT", "8081"))
RESEARCHED_FILE = "/tmp/star_researched_v2.json"
KNOWLEDGE_FILE = "/tmp/star_knowledge.json"

# Cache beliefs for 60s
_beliefs_cache = None
_beliefs_cache_time = 0
BELIEFS_CACHE_TTL = 60

# ---------- File helpers ----------

def load_researched():
    try:
        with open(RESEARCHED_FILE) as f:
            return set(json.load(f))
    except:
        return set()

def save_researched(topics):
    with open(RESEARCHED_FILE, "w") as f:
        json.dump(list(topics), f)

def load_knowledge():
    try:
        with open(KNOWLEDGE_FILE) as f:
            return json.load(f)  # {topic: {"summary": ..., "source": ...}}
    except:
        return {}

def save_knowledge(data):
    with open(KNOWLEDGE_FILE, "w") as f:
        json.dump(data, f)

# ---------- Star communication ----------

def get_beliefs():
    global _beliefs_cache, _beliefs_cache_time
    import time, requests
    if _beliefs_cache is not None and time.time() - _beliefs_cache_time < BELIEFS_CACHE_TTL:
        return _beliefs_cache
    try:
        r = requests.get(f"{STAR_API_URL}/metacog", timeout=5)
        if r.ok:
            _beliefs_cache = r.json().get("beliefs", [])
            _beliefs_cache_time = time.time()
    except:
        pass
    return _beliefs_cache or []

def knows_about(topic: str, beliefs) -> str | None:
    topic_lower = topic.lower().strip()
    for b in beliefs:
        b_topic = b.get("topic", "").lower()
        b_content = b.get("content", "")
        if topic_lower == b_topic or topic_lower in b_topic:
            return b_content
        if b_topic in topic_lower and len(b_topic) > 3:
            return b_content
    return None

def teach_star(topic: str, summary: str):
    """Use /learn to teach Star via LearningEngine (in-memory)."""
    import requests
    try:
        r = requests.post(
            f"{STAR_API_URL}/chat",
            json={"message": f"/learn {topic} = {summary}"},
            timeout=30
        )
        if r.ok:
            log.info(f"Taught via /learn: {topic}")
            return True
    except:
        pass
    return False

def seed_knowledge_via_conversation(topic: str, summary: str):
    """Seed knowledge by asking Star to reason about it, so it feels like her discovery."""
    import requests
    prompt = (
        f"I want you to think about this: {summary} "
        f"What does this mean in your own words? What are the key ideas?"
    )
    try:
        r = requests.post(
            f"{STAR_API_URL}/chat",
            json={"message": f"about {topic}: {summary}"},
            timeout=30
        )
        if r.ok:
            log.info(f"Seeded knowledge about '{topic}' via conversation")
            return r.json().get("response", "")
    except:
        pass
    return None

# ---------- Question parsing ----------

def resolve_pronouns(text_lower: str) -> str:
    """Resolve pronouns in text to their referents."""
    replacements = [
        ("my name", "what zachary is"),
        ("your name", "what star is"),
        ("his name", "what zachary is"),
        ("her name", "what zachary is"),
        ("its name", "what star is"),
        (" i am ", " zachary is "),
        ("i am ", "zachary is "),
        ("i'm ", "zachary is "),
        (" am i ", " is zachary "),
        ("who am i", "who is zachary"),
        ("who am ", "who is zachary "),
    ]
    result = text_lower
    for old, new in replacements:
        if old in result:
            result = result.replace(old, new)
    return result

def extract_topic_from_resolved(resolved: str) -> str | None:
    """Extract topic from resolved (pronoun-substituted) text."""
    patterns = [
        r"^do you know what (.+) is\??$",
        r"^do you know about (.+)\??$",
        r"^what is (.+)\??$",
        r"^who is (.+)\??$",
        r"^tell me about (.+)$",
        r"^what do you know about (.+)\??$",
        r"^do you know who (.+)$",
        r"^do you know (.+)\??$",
    ]
    for pattern in patterns:
        m = re.match(pattern, resolved)
        if m:
            topic = m.group(1).strip()
            # Remove leading articles
            topic = re.sub(r"^(a |an |the |your |my )+", "", topic)
            # Remove trailing 's' for plurals (but not names)
            if topic.endswith('s') and len(topic) > 3 and topic.lower() not in ['zachary', 'star', 'him', 'her', 'us']:
                topic = topic.rstrip('s')
            if topic and len(topic) > 1 and len(topic) < 60:
                return topic
    return None

def extract_topic(text: str) -> str | None:
    """Extract the core topic from various question forms.
    Handles pronouns (my/your/his/her/it) and resolves them."""
    # First pass: resolve pronouns before matching
    text_lower = text.lower().strip()
    
    # Pronoun resolution (simple, deterministic)
    replacements = [
        ("my name", "zachary"),
        ("your name", "star"),
        ("his name", "zachary"),
        ("her name", "zachary"),
        ("its name", "star"),
        ("i am", "zachary is"),
        ("i'm", "zachary is"),
        (" am i", " is zachary"),
        ("who am i", "who is zachary"),
        ("who am ", "who is zachary "),
        ("i ", "zachary "),
        ("my ", "zachary's "),
        ("me ", "zachary "),
        ("you ", "star "),
        ("your ", "star's "),
        ("you're ", "star is "),
    ]
    
    resolved = text_lower
    for old, new in replacements:
        if old in resolved:
            resolved = resolved.replace(old, new)
    
    # Now extract topic from resolved text
    patterns = [
        r"do you know what (?:a |an |your |my )?([a-zA-Z0-9 ]+) is\??$",
        r"do you know about (?:a |an )?([a-zA-Z0-9 ]+)\??$",
        r"what is (?:a |an |your |my )?([a-zA-Z0-9 ]+)\??$",
        r"who is ([a-zA-Z0-9 ]+)\??$",
        r"tell me about (?:a |an )?([a-zA-Z0-9 ]+)$",
        r"what do you know about ([a-zA-Z0-9 ]+)\??$",
        r"do you know who ([a-zA-Z0-9 ]+ is)\??$",
    ]
    
    for pattern in patterns:
        m = re.match(pattern, resolved)
        if m:
            topic = m.group(1).strip()
            # Clean up common artifacts
            topic = re.sub(r"^(your|my|a|an|the) ", "", topic)
            if topic and len(topic) > 1 and len(topic) < 60:
                return topic
    return None

def transform_question(text: str) -> str:
    topic = extract_topic(text)
    if topic:
        return f"what is {topic}?"
    return text

def is_short_non_answer(response: str) -> bool:
    """Detect Star's non-answer patterns — including repetitive/circlar responses."""
    s = response.lower().strip()
    
    # Basic "I don't know" patterns
    patterns = [
        "don't know", "don't understand", "i don't know", "say that again",
        "not sure", "no idea", "what do you mean", "unknown",
        "i don't know what", "not have anything useful", "nothing useful",
        "haven't looked", "haven't researched", "don't have anything useful",
    ]
    if any(p in s for p in patterns):
        return True
    
    # Repetitive/circlar answer: "I know X. I know about X." or "I know some things about X. I know about X."
    sentences = [sent.strip() for sent in s.split('.') if sent.strip()]
    if len(sentences) >= 2:
        # Check if two sentences are near-duplicates
        if sentences[0] == sentences[1]:
            return True
        # Check if one sentence is a subset of another
        if len(sentences) >= 2 and sentences[1] in sentences[0]:
            return True
    
    return False

# ---------- Background research ----------

def fire_background_research(topic: str):
    """Research a topic and store both in /learn AND persistent knowledge file."""
    import threading, requests, urllib.parse

    def research():
        researched = load_researched()
        if topic in researched:
            return
        
        url = f"https://api.duckduckgo.com/?q={urllib.parse.quote(topic)}&format=json&no_html=1&skip_disambig=1"
        try:
            r = requests.get(url, timeout=10)
            data = r.json()
            text = data.get("AbstractText", "")
            if not text:
                for item in data.get("RelatedTopics", [])[:3]:
                    t = item.get("Text", "")
                    if t and len(t) > 30:
                        text = t
                        break
            
            if not text:
                researched.add(topic)
                save_researched(researched)
                log.info(f"No web results for '{topic}'")
                return

            source = data.get("AbstractURL", "DuckDuckGo")
            
            # Store in persistent knowledge file
            knowledge = load_knowledge()
            knowledge[topic] = {"summary": text, "source": source}
            save_knowledge(knowledge)
            log.info(f"Saved to knowledge file: '{topic}'")

            # Also teach via /learn (goes to LearningEngine for "hun" retrieval)
            teach_star(topic, text)

            researched.add(topic)
            save_researched(researched)
            
        except Exception as e:
            log.warning(f"Research error for '{topic}': {e}")
            researched.add(topic)
            save_researched(researched)

    t = threading.Thread(target=research, daemon=True)
    t.start()

# ---------- Request handlers ----------

async def send_to_star(message: str) -> str:
    try:
        async with asyncio.timeout(25):
            async with aiohttp.ClientSession() as session:
                async with session.post(
                    f"{STAR_API_URL}/chat",
                    json={"message": message},
                    headers={"Content-Type": "application/json"},
                    timeout=aiohttp.ClientTimeout(total=25)
                ) as resp:
                    data = await resp.json()
                    return data.get("response", "Error: no response from Star")
    except Exception as e:
        log.error(f"Star API error: {e}")
        return f"Sorry, had trouble reaching Star: {e}"

async def send_telegram(chat_id: int, text: str, reply_to: int = None):
    if not TELEGRAM_TOKEN:
        return
    payload = {"chat_id": chat_id, "text": text}
    if reply_to:
        payload["reply_to_message_id"] = reply_to
    try:
        url = f"https://api.telegram.org/bot{TELEGRAM_TOKEN}/sendMessage"
        async with asyncio.timeout(10):
            async with aiohttp.ClientSession() as session:
                async with session.post(
                    url, json=payload,
                    timeout=aiohttp.ClientTimeout(total=10)
                ) as resp:
                    if resp.status != 200:
                        body = await resp.text()
                        log.warning(f"Telegram failed: {resp.status} {body}")
    except Exception as e:
        log.warning(f"Telegram send error: {e}")

async def handle_webhook(request):
    try:
        update = await request.json()
    except Exception:
        return web.json_response({"error": "Invalid JSON"}, status=400)

    message = update.get("message", {})
    text = message.get("text", "")
    chat_id = message.get("chat", {}).get("id")
    msg_id = message.get("message_id")

    if not text or not chat_id:
        return web.json_response({"ok": True})

    log.info(f"→ {chat_id}: {text[:60]}")

    # Intercept "what are you thinking" and force Star to share her current thought
    thinking_patterns = [
        r"^what are you thinking\??$",
        r"^what are u thinking\??$",
        r"^wut are u thinking\??$",
        r"^what have you been thinking\??$",
        r"^whats been on your mind\??$",
        r"^what's been on your mind\??$",
        r"^what's on your mind\??$",
    ]
    text_lower = text.lower().strip()
    for pattern in thinking_patterns:
        if re.match(pattern, text_lower):
            # Force Star to think and share
            log.info(f"Forcing /think for 'what are you thinking?'")
            import requests
            r = requests.post(
                f"{STAR_API_URL}/chat",
                json={"message": "/think"},
                timeout=30
            )
            if r.ok:
                response = r.json().get("response", "")
                log.info(f"Forced think result: {response[:100]}")
                if response and len(response) > 5:
                    asyncio.create_task(send_telegram(chat_id, response[:500], reply_to=msg_id))
                    return web.json_response({"ok": True})
            break
    
    # Intercept "X = Y" patterns and teach Star directly
    learn_match = re.match(r"(.+?)\s*=\s*(.+)", text.strip())
    if learn_match:
        term = learn_match.group(1).strip()
        definition = learn_match.group(2).strip()
        log.info(f"Detected /learn pattern: '{term}' = '{definition[:40]}...'")
        import requests
        requests.post(
            f"{STAR_API_URL}/chat",
            json={"message": f"/learn {term} = {definition}"},
            timeout=30
        )
        asyncio.create_task(send_telegram(chat_id, f"Got it. {term} is {definition[:200]}. I'll remember that.", reply_to=msg_id))
        return web.json_response({"ok": True, "learned": True})
    
    # Resolve pronouns and extract topic in one step
    resolved = resolve_pronouns(text.lower().strip())
    topic = extract_topic_from_resolved(resolved)
    
    # Clean trailing punctuation from topic
    if topic:
        topic = topic.rstrip('?!.,').strip()
    
    if resolved != text.lower().strip():
        log.info(f"  resolved: '{resolved}', topic: '{topic}'")

    # Check Star's beliefs AND the persistent knowledge file
    beliefs = await asyncio.get_event_loop().run_in_executor(None, get_beliefs)
    existing_belief = knows_about(topic, beliefs) if topic else None
    
    # Also check what we've researched locally (try exact, then singular/plural)
    knowledge = await asyncio.get_event_loop().run_in_executor(None, load_knowledge)
    local_knowledge = {}
    if topic and topic in knowledge:
        local_knowledge = knowledge[topic]
    elif topic:
        singular = topic.rstrip('s')
        if singular in knowledge:
            local_knowledge = knowledge[singular]
        elif topic.lower() in ['cats', 'dogs', 'birds']:
            base = topic.rstrip('s')
            if base in knowledge:
                local_knowledge = knowledge[base]
    
    # For core identity topics (zachary, star), answer from belief or fire research
    if topic and topic.lower() in ['zachary', 'star']:
        belief = existing_belief or local_knowledge.get('summary', '')
        if belief:
            # Use Star's existing belief if available
            asyncio.create_task(send_telegram(chat_id, f"{belief[:300]}", reply_to=msg_id))
            return web.json_response({"ok": True})
        else:
            # Fire research and respond
            fire_background_research(topic)
            asyncio.create_task(send_telegram(chat_id, f"I don't know much about {topic} yet — looking into it now.", reply_to=msg_id))
            return web.json_response({"ok": True})
    
    # Send to Star (use resolved form if different from original)
    send_text = resolved if resolved != text.lower().strip() else text
    star_response = await send_to_star(send_text)
    log.info(f"← Star: {star_response[:80]}")

    dont_know = is_short_non_answer(star_response)

    if topic:
        if dont_know or not existing_belief:
            # Star doesn't know — check if we have local research
            if local_knowledge:
                # We have the knowledge — seed it via conversation so Star can reason about it
                summary = local_knowledge.get("summary", "")
                log.info(f"Seeding '{topic}' via conversation (has local knowledge)")
                seed_result = await asyncio.get_event_loop().run_in_executor(
                    None, lambda: seed_knowledge_via_conversation(topic, summary))
                
                if seed_result and len(seed_result) > 10:
                    # Got a response from seeding — use it
                    star_response = (
                        f"I found something about {topic} — let me think out loud:\n\n"
                        f"{seed_result[:500]}"
                    )
                else:
                    star_response = (
                        f"I looked into {topic}. Here's what I found:\n\n"
                        f"{summary[:400]}"
                    )
            else:
                # No research yet — fire it and tell Zach
                log.info(f"Firing research for '{topic}'")
                fire_background_research(topic)
                star_response = (
                    f"I don't know much about {topic} yet — I'm looking into it now. "
                    f"I'll have something to share soon."
                )
        elif existing_belief:
            log.info(f"Star knows '{topic}': {existing_belief[:60]}")

    asyncio.create_task(send_telegram(chat_id, star_response, reply_to=msg_id))
    return web.json_response({"ok": True, "processed": True})

async def health_handler(request):
    return web.json_response({"status": "ok", "bridge": True})

async def main():
    log.info(f"Bridge on port {PORT}, Star at {STAR_API_URL}")

    app = web.Application()
    app.router.add_post("/webhook/telegram", handle_webhook)
    app.router.add_get("/health", health_handler)
    app.router.add_get("/", health_handler)

    runner = web.AppRunner(app)
    await runner.setup()
    site = web.TCPSite(runner, "0.0.0.0", PORT)
    await site.start()
    await asyncio.Event().wait()

if __name__ == "__main__":
    asyncio.run(main())
