#!/usr/bin/env python3
"""
Star Learning Wrapper — sits in front of Star's API

Handles the full learn loop:
1. Zach asks Star something
2. Wrapper checks if Star knows enough via /metacog beliefs
3. If NOT → fires background research → teaches Star via /learn
4. Responds to Zach: "I don't know yet, but I'm looking into it"
5. On NEXT message from Zach, Star has learned it and shares it

Also fires research on Star's curiosity topics every 30min.
"""

import os, sys, time, logging, requests, json, threading

logging.basicConfig(level=logging.INFO, format='%(asctime)s [StarLearn] %(message)s', stream=sys.stdout)
log = logging.getLogger(__name__)

STAR_API = os.environ.get("STAR_API_URL", "http://localhost:8080")
RESEARCHED_FILE = "/tmp/star_researched_v2.json"
PENDING_FILE = "/tmp/star_pending.json"
COOLDOWN = 1800  # 30 min between research topics

def load_researched():
    try:
        with open(RESEARCHED_FILE) as f:
            return set(json.load(f))
    except:
        return set()

def save_researched(topics):
    with open(RESEARCHED_FILE, "w") as f:
        json.dump(list(topics), f)

def load_pending():
    try:
        with open(PENDING_FILE) as f:
            return set(json.load(f))
    except:
        return set()

def save_pending(topics):
    with open(PENDING_FILE, "w") as f:
        json.dump(list(topics), f)

def get_beliefs():
    """Get Star's beliefs from metacog."""
    try:
        r = requests.get(f"{STAR_API}/metacog", timeout=5)
        return r.json().get("beliefs", [])
    except:
        return []

def knows_about(topic: str, beliefs) -> bool:
    """Check if Star has a belief about this topic."""
    topic_lower = topic.lower()
    for b in beliefs:
        if topic_lower in b.get("topic", "").lower() or topic_lower in b.get("content", "").lower():
            return True
    return False

def web_research(topic: str):
    """Search DuckDuckGo for topic info."""
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
    except Exception as e:
        log.warning(f"Search failed for '{topic}': {e}")
        return None

def teach_star(topic: str, definition: str):
    """Teach Star via /learn."""
    try:
        # Use chat to /learn
        r = requests.post(
            f"{STAR_API}/chat",
            json={"message": f"/learn {topic} = {definition}"},
            timeout=30
        )
        if r.ok:
            resp = r.json().get("response", "")
            log.info(f"Taught '{topic}': {resp[:80]}")
            return True
        log.warning(f"Teach failed: {r.status_code}")
    except Exception as e:
        log.error(f"Teach error: {e}")
    return False

def research_topic(topic: str, researched: set, pending: set):
    """Research a topic and teach Star. Runs in background thread."""
    log.info(f"Researching: '{topic}'")
    result = web_research(topic)
    
    if result:
        summary, source = result
        if teach_star(topic, summary):
            researched.add(topic)
            pending.discard(topic)
            save_researched(researched)
            save_pending(pending)
            log.info(f"Learned '{topic}' from research")
            return True
    else:
        # Mark as researched even on failure to avoid retry loops
        researched.add(topic)
        pending.discard(topic)
        save_researched(researched)
        save_pending(pending)
        log.info(f"No results for '{topic}', marked as researched")
    return False

def get_curiosity_topics():
    """Get Star's curiosity topics."""
    try:
        r = requests.get(f"{STAR_API}/metacog", timeout=5)
        return r.json().get("curiosity_topics", [])
    except:
        return []

def get_pending():
    """Get topics Star said she's looking into."""
    try:
        with open(PENDING_FILE) as f:
            data = json.load(f)
            if isinstance(data, list):
                return set(data)
            return set()
    except:
        return set()

def save_pending(topics):
    with open(PENDING_FILE, "w") as f:
        json.dump(list(topics), f)

def research_in_background(topic: str, researched: set, pending: set):
    """Fire research in a background thread."""
    pending.add(topic)
    save_pending(pending)
    researched_copy = researched.copy()
    pending_copy = pending.copy()
    
    t = threading.Thread(target=research_topic, args=(topic, researched_copy, pending_copy))
    t.daemon = True
    t.start()
    log.info(f"Background research started: '{topic}'")

def check_curiosity_and_research():
    """Background: research curiosity topics."""
    researched = load_researched()
    pending = get_pending()
    topics = get_curiosity_topics()
    
    for topic in topics:
        if topic not in researched and topic not in pending:
            research_in_background(topic, researched, pending)
            break  # One at a time

def main():
    log.info(f"Star Learning Wrapper starting... API: {STAR_API}")
    
    if not requests.get(f"{STAR_API}/health", timeout=5).ok:
        log.error(f"Star not reachable at {STAR_API}")
        sys.exit(1)
    
    # Load state
    researched = load_researched()
    pending = get_pending()
    last_research_check = 0
    
    while True:
        try:
            now = time.time()
            
            # Every 30min: check curiosity topics for research
            if now - last_research_check > COOLDOWN:
                check_curiosity_and_research()
                last_research_check = now
            
            time.sleep(60)
            
        except KeyboardInterrupt:
            break
        except Exception as e:
            log.error(f"Error: {e}")
            time.sleep(30)

if __name__ == "__main__":
    main()
