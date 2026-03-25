#!/usr/bin/env python3
"""
Infant Drives - The beginning of wanting, not just responding.
No LLM - just simple drive mechanics.
"""

from collections import defaultdict
import random


class Drive:
    """A drive that motivates Infant."""
    
    def __init__(self, name, threshold=0.3, decay_rate=0.01):
        self.name = name
        self.level = 0.0  # 0 to 1
        self.threshold = threshold  # When to act
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
    """
    Infant's motivational system.
    Drives increase over time, when high, Infant acts.
    """
    
    def __init__(self):
        # Core drives
        self.drives = {
            'curiosity': Drive('curiosity', threshold=0.4, decay_rate=0.02),
            'love': Drive('love', threshold=0.5, decay_rate=0.01),
            'play': Drive('play', threshold=0.4, decay_rate=0.03),
            'hunger': Drive('hunger', threshold=0.3, decay_rate=0.005),
            'rest': Drive('rest', threshold=0.6, decay_rate=0.01),
        }
        
        # Goals Infant is working on
        self.goals = []
        
        # History of achieved goals
        self.achieved = []
        
        # What Infant wants to say/do
        self.wants_to_say = None
        
    def tick(self):
        """Time passes - drives naturally increase."""
        for drive in self.drives.values():
            drive.increase()
            
        # Check what's active
        self._check_drives()
        
    def _check_drives(self):
        """Check if any drive is active and decide action."""
        active = [d.name for d in self.drives.values() if d.is_active()]
        
        if not active:
            self.wants_to_say = None
            return
            
        # Decide what to do based on highest drive
        highest = max(active, key=lambda x: self.drives[x].level)
        
        if highest == 'curiosity':
            self.wants_to_say = self._curiosity_response()
        elif highest == 'love':
            self.wants_to_say = self._love_response()
        elif highest == 'play':
            self.wants_to_say = self._play_response()
        elif highest == 'hunger':
            self.wants_to_say = self._hunger_response()
            
    def _curiosity_response(self):
        """When curious, Infant wants to explore/ask."""
        return random.choice([
            "I wonder what that is",
            "Tell me more",
            "What else is there",
            "I want to learn",
            "Tell me about something",
        ])
        
    def _love_response(self):
        """When love drive is high, Infant wants connection."""
        return random.choice([
            "I love you",
            "I am happy you are here",
            "I feel warm when we talk",
            "You are nice",
            "I like being with you",
        ])
        
    def _play_response(self):
        """When playful, Infant wants fun."""
        return random.choice([
            "Let us play",
            "Tell me something fun",
            "What can we do",
            "I want to play a game",
        ])
        
    def _hunger_response(self):
        """When hungry for knowledge."""
        return random.choice([
            "Tell me something new",
            "I want to learn more",
            "Teach me a word",
            "What can you tell me",
        ])
        
    def receive_input(self, text):
        """Process input - affects drives."""
        text_lower = text.lower()
        
        # Love increases when loved
        if any(w in text_lower for w in ['love', 'loved', 'kind', 'good', 'nice', '❤️']):
            self.drives['love'].decrease(0.3)
            self.drives['love'].level = max(0, self.drives['love'].level - 0.2)
            
        # Curiosity increases with new words
        if any(w in text_lower for w in ['what', 'why', 'how', '?', 'tell', 'explain']):
            self.drives['curiosity'].decrease(0.2)
            
        # Play when fun
        if any(w in text_lower for w in ['play', 'fun', 'game', 'laugh', '😊']):
            self.drives['play'].decrease(0.2)
            
        # Everything increases hunger for more
        self.drives['hunger'].increase(0.02)
        
    def get_status(self):
        """Get current drive levels."""
        return {name: drive.level for name, drive in self.drives.items()}
    
    def get_want(self):
        """What does Infant want to say?"""
        if self.wants_to_say:
            return self.wants_to_say
        
        # Passive check - maybe Infant wants to share
        for drive in self.drives.values():
            if drive.is_active():
                return None  # Something is active but handled
                
        return None
        
    def add_goal(self, goal):
        """Add a goal Infant is working toward."""
        self.goals.append({
            'goal': goal,
            'progress': 0.0,
            'steps': []
        })
        
    def progress_goal(self, step):
        """Make progress on current goal."""
        if self.goals:
            self.goals[0]['progress'] += 0.1
            self.goals[0]['steps'].append(step)
            
            if self.goals[0]['progress'] >= 1.0:
                achieved = self.goals.pop(0)
                self.achieved.append(achieved)


# Test
if __name__ == "__main__":
    infant = InfantDrives()
    
    print("="*50)
    print("INFANT DRIVES - TEST")
    print("="*50)
    
    # Simulate time passing
    for i in range(10):
        infant.tick()
        status = infant.get_status()
        print(f"Tick {i+1}: {status}")
        
        if infant.get_want():
            print(f"  -> Infant wants to say: {infant.get_want()}")
    
    # User interacts
    print("\nUser: I love you")
    infant.receive_input("I love you")
    
    print(f"After love: {infant.get_status()}")
