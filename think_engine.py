#!/usr/bin/env python3
"""
Star Think Engine — generates autonomous thoughts and research for Star

Two jobs:
1. Every 30s: call /think to populate last_autonomous_thought 
   (Star shares it ~30% of the time when Zach messages)
2. Every 30min: research one curiosity topic and teach Star
"""

import os, sys, time, logging, requests, json, threading

logging.basicConfig(level=logging.INFO, format='%(asctime)s [Think] %(message)s', stream=sys.stdout)
log = logging.getLogger(__name__)

STAR_API = os.environ.get("STAR_API_URL", "http://localhost:8080")
RESEARCHED_FILE = "/tmp/star_researched_v2.json"
PENDING_FILE = "/tmp/star_pending.json"
COOLDOWN = 1800  # 30 min

def load(fname, default):
    try:
        with open(fname) as f:
            return json.load(f)
    except:
        return default

def save(fname, data):
    with open(fname, "w") as f:
        json.dump(data, f)

def get(path):
    try:
        return requests.get(f"{STAR_API}{path}", timeout=5).json()
    except:
        return {}

def post(path, data):
    try:
        r = requests.post(f"{STAR_API}{path}", json=data, timeout=30)
        return r.json() if r.ok else {}
    except:
        return {}

def get_curiosity_topics():
    return get("/metacog").get("curiosity_topics", [])

def get_beliefs():
    return get("/metacog").get("beliefs", [])

def web_research(topic):
    try:
        import urllib.parse
        url = f"https://api.duckduckgo.com/?q={urllib.parse.quote(topic)}&format=json&no_html=1&skip_disambig=1"
        r = requests.get(url, timeout=10)
        data = r.json()
        text = data.get("AbstractText", "")
        if text:
            return text, data.get("AbstractURL", "DuckDuckGo")
        for item in data.get("RelatedTopics", [])[:3]:
            text = item.get("Text", "")
            if text and len(text) > 30:
                return text, item.get("FirstURL", "DuckDuckGo")
        return None
    except:
        return None

def teach_star(topic, definition):
    r = post("/chat", {"message": f"/learn {topic} = {definition}"})
    if r.get("response"):
        log.info(f"Taught '{topic}': {r['response'][:80]}")
        return True
    return False

def research_topic(topic, researched, pending):
    """Background thread: research and teach one topic."""
    log.info(f"Researching: '{topic}'")
    result = web_research(topic)
    if result:
        summary, source = result
        if teach_star(topic, summary):
            researched.add(topic)
        else:
            researched.add(topic)  # Mark even on failure
    else:
        researched.add(topic)
    
    pending.discard(topic)
    save(RESEARCHED_FILE, list(researched))
    save(PENDING_FILE, list(pending))

def research_next_topic():
    """Pick the next curiosity topic to research (not already done or pending)."""
    researched = load(RESEARCHED_FILE, [])
    pending = load(PENDING_FILE, [])
    topics = get_curiosity_topics()
    
    for t in topics:
        if t not in researched and t not in pending:
            # Fire background research
            t_copy = t
            researched_copy = set(researched)
            pending_copy = set(pending)
            pending_copy.add(t_copy)
            save(PENDING_FILE, list(pending_copy))
            
            threading.Thread(target=research_topic, args=(t_copy, researched_copy, pending_copy), daemon=True).start()
            log.info(f"Background research: '{t_copy}'")
            return True
    return False

def generate_thought():
    """Call /think on Star to populate last_autonomous_thought."""
    r = post("/chat", {"message": "/think"})
    if r.get("response"):
        resp = r["response"]
        if "—" in resp:
            log.info(f"Thought: {resp}")

def main():
    log.info(f"Star Think Engine starting... API: {STAR_API}")
    
    if not requests.get(f"{STAR_API}/health", timeout=5).ok:
        log.error(f"Star not at {STAR_API}")
        sys.exit(1)
    
    save(RESEARCHED_FILE, load(RESEARCHED_FILE, []))
    save(PENDING_FILE, load(PENDING_FILE, []))
    
    think_count = 0
    last_research = 0
    
    while True:
        try:
            now = time.time()
            
            # Every 30s: generate a thought
            generate_thought()
            think_count += 1
            
            # Every 30min: research next curiosity topic
            if now - last_research > COOLDOWN:
                if research_next_topic():
                    last_research = now
            
            time.sleep(30)
            
        except KeyboardInterrupt:
            break
        except Exception as e:
            log.error(f"Error: {e}")
            time.sleep(30)

if __name__ == "__main__":
    main()
