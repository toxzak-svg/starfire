"""
Conversation Dataset Builder

Collects, spell-checks, and grammar-checks conversation data
for personal dataset creation.
"""

import json
import os
from datetime import datetime
import re

DATA_DIR = r"C:\Users\Zwmar\Claw\conversation-data"
RAW_FILE = os.path.join(DATA_DIR, "raw_conversations.jsonl")
CLEAN_FILE = os.path.join(DATA_DIR, "cleaned_conversations.jsonl")
METADATA_FILE = os.path.join(DATA_DIR, "metadata.json")

# Simple spell/grammar corrections (expandable)
CORRECTIONS = {
    "thats": "that's",
    "im": "I'm",
    "dont": "don't",
    "cant": "can't",
    "wont": "won't",
    "isnt": "isn't",
    "didnt": "didn't",
    "doesnt": "doesn't",
    "youre": "you're",
    "theyre": "they're",
    "were": "we're",
    "couldnt": "couldn't",
    "shouldnt": "shouldn't",
    "wouldnt": "wouldn't",
    "thats": "that's",
    "hes": "he's",
    "shes": "she's",
    "lets": "let's",
    "thats": "that's",
    "rn": "right now",
    "idk": "I don't know",
    "btw": "by the way",
    "imo": "in my opinion",
    "tbh": "to be honest",
    "lol": "laugh out loud",
    "aka": "also known as",
    "eg": "for example",
    "ie": "that is",
    "etc": "and so on",
    "asap": "as soon as possible",
    "fyi": "for your information",
    "nvm": "never mind",
    "smth": "something",
    "sb": "somebody",
    "tho": "though",
    "kinda": "kind of",
    "sorta": "sort of",
    "gotta": "got to",
    "wanna": "want to",
    "gonna": "going to",
    "lemme": "let me",
    "gotta": "got to",
    "ya": "you",
    "yall": "you all",
    "ain't": "isn't",
    "finna": "fixing to",
    "lit": "lit",
    "salty": "salty",
    "shook": "shocked",
    "tea": "gossip",
    "v": "very",
    "r": "are",
    "u": "you",
    "ur": "your",
    "b": "be",
    "c": "see",
    "y": "why",
    "n": "and",
    "w": "with",
    "m": "am",
    "plz": "please",
    "thx": "thanks",
    "thnx": "thanks",
    "pls": "please",
    "ok": "okay",
    "yeah": "yes",
    "yep": "yes",
    "nope": "no",
    "maybe": "maybe",
    "cool": "cool",
    "nice": "nice",
    "great": "great",
    "awesome": "awesome",
    "terrible": "terrible",
    "horrible": "horrible",
    "weird": "weird",
    "strange": "strange",
    "love": "love",
    "hate": "hate",
    "like": "like",
    "dislike": "dislike",
    "want": "want",
    "need": "need",
    "wish": "wish",
    "hope": "hope",
    "think": "think",
    "believe": "believe",
    "know": "know",
    "remember": "remember",
    "forget": "forget",
    "understand": "understand",
    "realize": "realize",
    "feel": "feel",
    "seem": "seem",
    "look": "look",
    "find": "find",
    "thing": "thing",
    "things": "things",
    "stuff": "stuff",
    "way": "way",
    "ways": "ways",
    "lot": "lot",
    "lots": "lots",
    "bit": "bit",
    "little": "little",
    "much": "much",
    "many": "many",
    "most": "most",
    "some": "some",
    "any": "any",
    "every": "every",
    "each": "each",
    "either": "either",
    "neither": "neither",
    "both": "both",
    "few": "few",
    "several": "several",
    "enough": "enough",
    "plenty": "plenty",
    "tonight": "tonight",
    "today": "today",
    "tomorrow": "tomorrow",
    "yesterday": "yesterday",
    "sometimes": "sometimes",
    "always": "always",
    "never": "never",
    "often": "often",
    "usually": "usually",
    "again": "again",
    "still": "still",
    "already": "already",
    "also": "also",
    "just": "just",
    "even": "even",
    "ever": "ever",
    "once": "once",
    "twice": "twice",
    "actually": "actually",
    "probably": "probably",
    "definitely": "definitely",
    "maybe": "maybe",
    "perhaps": "perhaps",
    "certainly": "certainly",
    "sure": "sure",
    "exactly": "exactly",
    "especially": "especially",
    "specifically": "specifically",
    "particularly": "particularly",
    "generally": "generally",
    "usually": "usually",
    "normally": "normally",
    "basically": "basically",
    "literally": "literally",
    "basically": "basically",
    "obviously": "obviously",
    "clearly": "clearly",
    "simply": "simply",
    "absolutely": "absolutely",
    "completely": "completely",
    "totally": "totally",
    "entirely": "entirely",
    "very": "very",
    "really": "really",
    "extremely": "extremely",
    "incredibly": "incredibly",
    "especially": "especially",
    "particularly": "particularly",
    "definitely": "definitely",
    "certainly": "certainly",
    "surely": "surely",
    "perhaps": "perhaps",
    "maybe": "maybe",
    "possibly": "possibly",
    "probably": "probably",
    "likely": "likely",
    "unlikely": "unlikely",
    "almost": "almost",
    "nearly": "nearly",
    "around": "around",
    "approximately": "approximately",
    "about": "about",
    "roughly": "roughly",
    "close": "close",
    "far": "far",
    "near": "near",
    "here": "here",
    "there": "there",
    "where": "where",
    "when": "when",
    "how": "how",
    "why": "why",
    "what": "what",
    "which": "which",
    "who": "who",
    "whom": "whom",
    "whose": "whose",
    "whether": "whether",
    "whatever": "whatever",
    "whichever": "whichever",
    "whenever": "whenever",
    "wherever": "wherever",
}


def fix_spelling(text):
    """Apply spelling corrections."""
    # Don't correct inside words (use word boundaries)
    for wrong, correct in CORRECTIONS.items():
        # Use word boundary to avoid partial replacements
        pattern = r'\b' + re.escape(wrong) + r'\b'
        text = re.sub(pattern, correct, text, flags=re.IGNORECASE)
    return text


def fix_grammar(text):
    """Apply basic grammar fixes."""
    # Double negatives to single
    text = re.sub(r'\bdon\'t have no\b', "don't have any", text, flags=re.IGNORECASE)
    text = re.sub(r'\bcan\'t hardly\b', "can hardly", text, flags=re.IGNORECASE)
    text = re.sub(r'\bcould of\b', "could have", text, flags=re.IGNORECASE)
    text = re.sub(r'\bwould of\b', "would have", text, flags=re.IGNORECASE)
    text = re.sub(r'\bshould of\b', "should have", text, flags=re.IGNORECASE)
    text = re.sub(r'\bmight of\b', "might have", text, flags=re.IGNORECASE)
    text = re.sub(r'\bwouldn\'t of\b', "wouldn't have", text, flags=re.IGNORECASE)
    text = re.sub(r'\bshouldn\'t of\b', "shouldn't have", text, flags=re.IGNORECASE)
    text = re.sub(r'\bcouldn\'t of\b', "couldn't have", text, flags=re.IGNORECASE)
    
    # Their/there/they're (simple cases)
    # These are context-dependent, so we'll be conservative
    
    # Its/it's (simple - it's = it is, its = possessive)
    # Be conservative here too
    
    # Your/you're
    # Be conservative
    
    # Lie/lay (conservative)
    # Lay/lie
    
    # Subject-verb agreement (basic)
    text = re.sub(r'\bhe don\'t\b', "he doesn't", text, flags=re.IGNORECASE)
    text = re.sub(r'\bshe don\'t\b', "she doesn't", text, flags=re.IGNORECASE)
    text = re.sub(r'\bthey was\b', "they were", text, flags=re.IGNORECASE)
    
    # Fewer/less (conservative - only for "less" before numbers)
    # Be careful here
    
    # Then/than (easy cases)
    text = re.sub(r'\brather then\b', "rather than", text, flags=re.IGNORECASE)
    text = re.sub(r'\bbetter then\b', "better than", text, flags=re.IGNORECASE)
    text = re.sub(r'\bmore then\b', "more than", text, flags=re.IGNORECASE)
    text = re.sub(r'\bno then\b', "no than", text, flags=re.IGNORECASE)  # oops
    
    # To/too/two (conservative)
    # Be careful
    
    # A lot (two words)
    text = re.sub(r'\balot\b', "a lot", text, flags=re.IGNORECASE)
    
    # Slight grammar improvements
    text = re.sub(r'\b(good|bad) then\b', r'\1 than', text, flags=re.IGNORECASE)
    
    return text


def clean_text(text):
    """Clean and normalize text."""
    # Fix spelling
    text = fix_spelling(text)
    # Fix grammar
    text = fix_grammar(text)
    
    # Fix multiple spaces
    text = re.sub(r' +', ' ', text)
    
    # Fix multiple newlines
    text = re.sub(r'\n\n+', '\n\n', text)
    
    # Capitalize first letter after newlines
    lines = text.split('\n')
    cleaned_lines = []
    for line in lines:
        if line:
            line = line.strip()
            if line:
                line = line[0].upper() + line[1:] if len(line) > 1 else line.upper()
        cleaned_lines.append(line)
    text = '\n'.join(cleaned_lines)
    
    return text.strip()


def add_conversation(user_message, assistant_message, metadata=None):
    """Add a conversation pair to the dataset."""
    timestamp = datetime.now().isoformat()
    
    # Clean the messages
    user_clean = clean_text(user_message)
    assistant_clean = clean_text(assistant_message)
    
    # Raw entry
    raw_entry = {
        "timestamp": timestamp,
        "user": user_message,
        "assistant": assistant_message,
        "metadata": metadata or {}
    }
    
    # Cleaned entry
    clean_entry = {
        "timestamp": timestamp,
        "user": user_clean,
        "assistant": assistant_clean,
        "user_original": user_message,
        "assistant_original": assistant_message,
        "metadata": metadata or {}
    }
    
    # Append to raw file
    with open(RAW_FILE, 'a', encoding='utf-8') as f:
        f.write(json.dumps(raw_entry, ensure_ascii=False) + '\n')
    
    # Append to cleaned file
    with open(CLEAN_FILE, 'a', encoding='utf-8') as f:
        f.write(json.dumps(clean_entry, ensure_ascii=False) + '\n')
    
    print(f"Added conversation at {timestamp}")
    return clean_entry


def get_stats():
    """Get dataset statistics."""
    stats = {
        "raw_count": 0,
        "clean_count": 0,
        "last_updated": None
    }
    
    if os.path.exists(RAW_FILE):
        with open(RAW_FILE, 'r', encoding='utf-8') as f:
            stats["raw_count"] = sum(1 for _ in f)
            # Get last timestamp
            for line in f:
                pass
    
    if os.path.exists(METADATA_FILE):
        with open(METADATA_FILE, 'r', encoding='utf-8') as f:
            stats.update(json.load(f))
    
    return stats


if __name__ == "__main__":
    # Test
    test = "thats really cool tho, idrc lol"
    print(f"Original: {test}")
    print(f"Cleaned:  {clean_text(test)}")
