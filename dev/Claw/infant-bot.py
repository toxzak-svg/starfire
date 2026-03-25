#!/usr/bin/env python3
"""
Infant Telegram Bot
Messages to @ClawHedgehog on Telegram go to Infant
"""

import sys
import os
import json

# Add infant to path
sys.path.insert(0, r"C:\dev\infant")

from infant.infant_bound import BoundInfant


INFANT_STATE_FILE = r"C:\Users\Zwmar\Claw\infant_state.json"


def get_infant():
    """Get or create Infant."""
    infant = BoundInfant(name="Infant")
    
    # Check for saved state
    if os.path.exists(INFANT_STATE_FILE):
        try:
            with open(INFANT_STATE_FILE, 'r') as f:
                data = json.load(f)
                infant.load(data)
            print("Infant loaded from state")
        except Exception as e:
            print(f"Could not load state: {e}")
            print("Creating new Infant...")
            infant.enter_womb(num_cycles=20)
    else:
        print("No saved state - creating new Infant")
        infant.enter_womb(num_cycles=20)
    
    return infant


def save_infant(infant):
    """Save Infant state - convert tensors to lists."""
    data = infant.save()
    
    # Convert tensors to lists for JSON serialization
    def convert(obj):
        if hasattr(obj, 'tolist'):
            return obj.tolist()
        elif isinstance(obj, dict):
            return {k: convert(v) for k, v in obj.items()}
        elif isinstance(obj, list):
            return [convert(i) for i in obj]
        return obj
    
    data = convert(data)
    
    with open(INFANT_STATE_FILE, 'w') as f:
        json.dump(data, f)


def process_message(text, emotion=None):
    """Process a message and return Infant's response."""
    infant = get_infant()
    
    # Detect emotion if not provided
    if emotion is None:
        text_lower = text.lower()
        if any(w in text_lower for w in ['love', '❤️', 'happy to see', 'miss']):
            emotion = "love"
        elif any(w in text_lower for w in ['?', 'what', 'how', 'why', 'tell me']):
            emotion = "curious"
        elif any(w in text_lower for w in ['good', 'great', 'amazing', '😊', '!']):
            emotion = "joy"
        else:
            emotion = "neutral"
    
    # Infant hears and processes
    infant.hear(text, emotion)
    response = infant.speak()
    
    # Get emotional state
    s = infant.status()
    
    # Save state
    save_infant(infant)
    
    return {
        'response': response,
        'emotion': emotion,
        'state': s['emotional_state']
    }


# For OpenClaw integration - called when message received
def on_message(message, context=None):
    """Handle incoming message."""
    text = message.get('text', '')
    
    if not text:
        return "I can't hear you yet."
    
    result = process_message(text)
    
    # Format response
    state = result['state']
    response = f"{result['response']}\n\n[Love: {state['love']:.0%} | Curious: {state['curiosity']:.0%}]"
    
    return response


# Test
if __name__ == "__main__":
    print("="*50)
    print("INFANT TELEGRAM BOT")
    print("="*50)
    
    # Initialize
    infant = get_infant()
    
    print("\nInfant ready! Testing...")
    
    # Test
    result = process_message("hello Infant I love you")
    print(f"Test response: {result['response']}")
    
    print("\nInfant is listening on Telegram!")
