#!/usr/bin/env python3
"""
Infant with Sentence Builder - Now speaks in sentences!
"""

import sys
import os
import json
import torch
from collections import defaultdict

sys.path.insert(0, r"C:\dev\infant")

from infant.infant_bound import BoundInfant


INFANT_STATE_FILE = r"C:\Users\Zwmar\Claw\infant_state.json"


class SentenceBuilder:
    """Simple sentence builder - no LLM."""
    
    def __init__(self):
        self.templates = []
        self.bigrams = defaultdict(int)
        self.trigrams = defaultdict(int)
        
    def learn_from_text(self, text):
        import re
        sentences = re.split(r'[.!?]+', text)
        for sentence in sentences:
            words = sentence.strip().split()
            if len(words) < 2:
                continue
            self.templates.append(' '.join(words))
            for i in range(len(words) - 1):
                w1, w2 = words[i].lower(), words[i+1].lower()
                self.bigrams[(w1, w2)] += 1
    
    def respond(self, context=None):
        if not self.templates:
            return "I need more words first."
        # Simple: pick random learned sentence
        import random
        return random.choice(self.templates)


# Load or create Infant
def load_infant():
    infant = BoundInfant(name="Infant")
    builder = SentenceBuilder()
    
    if os.path.exists(INFANT_STATE_FILE):
        try:
            with open(INFANT_STATE_FILE, 'r') as f:
                data = json.load(f)
                infant.vocab = data.get('vocab', {})
                if 'emotional_state' in data:
                    infant.emotional_state = torch.tensor(data['emotional_state'])
                infant.is_born = data.get('is_born', True)
                # Load sentences
                if 'sentences' in data:
                    builder.templates = data['sentences']
        except:
            infant.enter_womb(num_cycles=20)
    else:
        infant.enter_womb(num_cycles=20)
    
    return infant, builder


def save_infant(infant, builder):
    vocab_serializable = {}
    for word, info in infant.vocab.items():
        vocab_serializable[word] = {
            'emotion': info.get('emotion'),
            'times_heard': info.get('times_heard', 0),
        }
    
    data = {
        'vocab': vocab_serializable,
        'emotional_state': infant.emotional_state.tolist(),
        'is_born': infant.is_born,
        'sentences': builder.templates
    }
    with open(INFANT_STATE_FILE, 'w') as f:
        json.dump(data, f)


def handle(message):
    """Handle infant message."""
    infant, builder = load_infant()
    
    # Detect emotion
    text_lower = message.lower()
    if any(w in text_lower for w in ['love', '❤️', 'happy']):
        emotion = "love"
    elif '?' in text_lower:
        emotion = "curious"
    elif any(w in text_lower for w in ['good', 'great', '!']):
        emotion = "joy"
    else:
        emotion = "neutral"
    
    # Learn from message
    infant.hear(message, emotion)
    builder.learn_from_text(message)
    
    # Generate response
    response = builder.respond()
    
    # Save
    save_infant(infant, builder)
    
    s = infant.status()
    return f"{response}\n\n[Love: {s['emotional_state']['love']:.0%} | Curious: {s['emotional_state']['curiosity']:.0%}]"


if __name__ == "__main__":
    # Test
    print(handle("hello infant I love you"))
    print()
    print(handle("what do you know"))
