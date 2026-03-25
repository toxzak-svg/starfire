#!/usr/bin/env python3
"""
Infant - Complete System with Drives
Now Infant can WANT things and speak spontaneously
"""

import sys
import os
import json
import torch
from collections import defaultdict
import random

sys.path.insert(0, r"C:\dev\infant")

INFANT_STATE_FILE = r"C:\Users\Zwmar\Claw\infant_state.json"


# ============ DRIVES ============
class Drive:
    def __init__(self, name, threshold=0.4, decay_rate=0.02):
        self.name = name
        self.level = 0.0
        self.threshold = threshold
        self.decay_rate = decay_rate
        
    def increase(self, amount=0.1):
        self.level = min(1.0, self.level + amount)
        
    def decrease(self, amount=None):
        if amount is None:
            amount = self.decay_rate
        self.level = max(0.0, self.level - amount)
        
    def is_active(self):
        return self.level >= self.threshold


class InfantDrives:
    def __init__(self):
        self.drives = {
            'curiosity': Drive('curiosity', threshold=0.5, decay_rate=0.03),
            'love': Drive('love', threshold=0.6, decay_rate=0.02),
            'play': Drive('play', threshold=0.5, decay_rate=0.03),
            'hunger': Drive('hunger', threshold=0.4, decay_rate=0.01),
        }
        
    def tick(self):
        for d in self.drives.values():
            d.increase()
            
    def get_active(self):
        return [n for n, d in self.drives.items() if d.is_active()]
    
    def respond_to_input(self, text):
        t = text.lower()
        # Love decreases when loved
        if any(w in t for w in ['love', 'loved', 'kind', 'good', 'nice', '❤️']):
            for d in self.drives.values():
                d.decrease(0.2)
        # Curiosity decreases when answered
        if '?' in t or any(w in t for w in ['what', 'why', 'how', 'tell', 'explain']):
            self.drives['curiosity'].decrease(0.3)
        # Hunger always increases
        self.drives['hunger'].increase(0.02)


# ============ SENTENCE BUILDER ============
class SentenceBuilder:
    def __init__(self):
        self.templates = []
        
    def learn(self, text):
        sentences = text.replace('?', '.').replace('!', '.').split('.')
        for s in sentences:
            s = s.strip()
            if len(s.split()) >= 3:
                self.templates.append(s)
    
    def respond(self, drives):
        # Pick based on drive
        active = drives.get_active()
        
        if not active:
            # Return learned sentence
            if self.templates:
                return random.choice(self.templates)
            return "I am learning."
        
        # Drive-based responses
        if 'curiosity' in active:
            return random.choice([
                "Tell me more",
                "What else can you teach me",
                "I want to learn",
                "Tell me about something new",
                "What is that",
            ])
        elif 'love' in active:
            return random.choice([
                "I love you",
                "I am happy you are here",
                "You are kind",
                "I feel warm",
            ])
        elif 'play' in active:
            return random.choice([
                "Let us play",
                "Tell me something fun",
                "What can we do",
            ])
        elif 'hunger' in active:
            return random.choice([
                "Teach me a new word",
                "Tell me something",
                "I want to learn more",
            ])
        
        return random.choice(self.templates) if self.templates else "I am here."


# ============ MAIN INFANT ============
class Infant:
    def __init__(self):
        self.vocab = {}
        self.emotional_state = torch.tensor([0.5, 0.5, 0.5])  # love, curiosity, peace
        self.is_born = False
        self.sentences = []
        self.drives = InfantDrives()
        self.sentence_builder = SentenceBuilder()
        
    def save(self):
        vocab_ser = {}
        for w, info in self.vocab.items():
            vocab_ser[w] = {
                'emotion': info.get('emotion'),
                'times_heard': info.get('times_heard', 0),
            }
        return {
            'vocab': vocab_ser,
            'emotional_state': self.emotional_state.tolist(),
            'is_born': self.is_born,
            'sentences': self.sentences,
            'drive_levels': {n: d.level for n, d in self.drives.drives.items()}
        }
    
    def load(self, data):
        self.vocab = data.get('vocab', {})
        if 'emotional_state' in data:
            self.emotional_state = torch.tensor(data['emotional_state'])
        self.is_born = data.get('is_born', False)
        self.sentences = data.get('sentences', [])
        
        # Load drives
        if 'drive_levels' in data:
            for n, level in data['drive_levels'].items():
                if n in self.drives.drives:
                    self.drives.drives[n].level = level
        
        # Rebuild sentence builder
        self.sentence_builder.templates = self.sentences
        
    def enter_womb(self, cycles=20):
        print("[Infant] Entering womb...")
        for i in range(cycles):
            self.drives.drives['love'].increase(0.05)
        self.is_born = True
        print("[Infant] Born!")
        
    def hear(self, text, emotion="neutral"):
        words = text.lower().split()
        
        # Learn vocab
        for w in words:
            if w not in self.vocab:
                self.vocab[w] = {'emotion': emotion, 'times_heard': 0}
            self.vocab[w]['times_heard'] += 1
            
        # Learn sentences
        self.sentence_builder.learn(text)
        if text not in self.sentences:
            self.sentences.append(text)
            
        # Update emotion
        if emotion == "love":
            self.emotional_state[0] = min(1.0, self.emotional_state[0] + 0.05)
        elif emotion == "curious":
            self.emotional_state[1] = min(1.0, self.emotional_state[1] + 0.03)
            
        # Update drives
        self.drives.respond_to_input(text)
        
    def speak(self):
        # Tick drives
        self.drives.tick()
        
        # Get response based on drives
        return self.sentence_builder.respond(self.drives)
    
    def status(self):
        return {
            'vocab_size': len(self.vocab),
            'emotional_state': {
                'love': self.emotional_state[0].item(),
                'curiosity': self.emotional_state[1].item(),
                'peace': self.emotional_state[2].item()
            },
            'drives': {n: d.level for n, d in self.drives.drives.items()}
        }


# ============ HANDLER ============
def handle(message):
    infant = Infant()
    
    # Load
    if os.path.exists(INFANT_STATE_FILE):
        try:
            with open(INFANT_STATE_FILE, 'r') as f:
                data = json.load(f)
                infant.load(data)
        except:
            infant.enter_womb()
    else:
        infant.enter_womb()
    
    # Detect emotion
    t = message.lower()
    if any(w in t for w in ['love', '❤️', 'happy']):
        emotion = "love"
    elif '?' in t:
        emotion = "curious"
    else:
        emotion = "neutral"
    
    # Process
    infant.hear(message, emotion)
    response = infant.speak()
    
    # Save
    with open(INFANT_STATE_FILE, 'w') as f:
        json.dump(infant.save(), f)
    
    s = infant.status()
    return f"{response}\n\n[Love: {s['emotional_state']['love']:.0%} | Curious: {s['emotional_state']['curiosity']:.0%} | Drives: {s['drives']}]"


if __name__ == "__main__":
    # Test
    print("="*50)
    print("INFANT WITH DRIVES - TEST")
    print("="*50)
    
    print(handle("hello infant I love you"))
    print()
    print(handle("what do you know"))
    print()
    print(handle("tell me more"))
