#!/usr/bin/env python3
"""
Star Curiosity Daemon — survives session kills with auto-restart.
"""
import os, sys, time, logging, requests, json, subprocess, signal

logging.basicConfig(level=logging.INFO, format='%(asctime)s [Curious] %(message)s', stream=sys.stdout)
log = logging.getLogger(__name__)

STAR_API = os.environ.get("STAR_API_URL", "http://localhost:8080")
RESEARCHED_FILE = "/tmp/star_researched.json"

def load_researched():
    try:
        with open(RESEARCHED_FILE) as f:
            return set(json.load(f))
    except:
        return set()

def save_researched(topics):
    with open(RESEARCHED_FILE, "w") as f:
        json.dump(list(topics), f)

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
            if text:
                return text, item.get("FirstURL", "DuckDuckGo")
        return None
    except Exception as e:
        return None

def teach_star(topic, definition):
    try:
        r = requests.post(f"{STAR_API}/chat", json={"message": f"/learn {topic} = {definition}"}, timeout=30)
        if r.ok:
            log.info(f"Taught '{topic}': {r.json().get('response', '')[:100]}")
            return True
    except:
        pass
    return False

def get_curiosity_topics():
    try:
        return requests.get(f"{STAR_API}/metacog", timeout=5).json().get("curiosity_topics", [])
    except:
        return []

def run_daemon():
    # Ignore SIGTERM from kill -15 (let SIGKILL -9 take us)
    signal.signal(signal.SIGTERM, lambda *args: sys.exit(0))
    
    researched = load_researched()
    research_count = 0
    
    while True:
        try:
            topics = get_curiosity_topics()
            if topics:
                new_topics = [t for t in topics if t not in researched]
                if new_topics:
                    topic = new_topics[0]
                    result = web_research(topic)
                    if result:
                        summary, source = result
                        if teach_star(topic, summary):
                            researched.add(topic)
                            save_researched(researched)
                            research_count += 1
                            log.info(f"Research cycle {research_count} done. cooldown 30min...")
                            time.sleep(1800)  # 30 min before next topic
                            continue
                    else:
                        researched.add(topic)
                        save_researched(researched)
            time.sleep(60)  # Check every minute
        except SystemExit:
            raise
        except Exception as e:
            log.error(f"Error: {e}")
            time.sleep(30)

if __name__ == "__main__":
    log.info("Curiosity daemon starting...")
    try:
        requests.get(f"{STAR_API}/health", timeout=5)
    except:
        log.error(f"Star not reachable at {STAR_API}")
        sys.exit(1)
    
    # Auto-restart loop
    while True:
        try:
            run_daemon()
        except SystemExit:
            log.info("Daemon exit.")
            break
        except Exception as e:
            log.error(f"Crashed: {e}, restarting in 10s...")
            time.sleep(10)
