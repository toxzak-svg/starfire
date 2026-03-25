#!/usr/bin/env python3
"""
Infant Sentence Builder - Learns to form sentences from stories
No LLM - just pattern learning from text
"""

import re
import json
import os
import random
from collections import defaultdict


class SentenceBuilder:
    """
    Learns sentence patterns and builds sentences from them.
    No LLM - pure pattern matching.
    """
    
    def __init__(self):
        # Word categories learned from context
        self.word_categories = defaultdict(set)
        
        # Sentence templates (abstracted)
        self.templates = []
        
        # Common word sequences (n-grams)
        self.bigrams = defaultdict(int)
        self.trigrams = defaultdict(int)
        
        # Simple grammar rules
        self.subject_verbs = defaultdict(set)  # subject -> possible verbs
        self.verb_objects = defaultdict(set)  # verb -> possible objects
        
    def learn_from_text(self, text):
        """Learn patterns from text."""
        sentences = re.split(r'[.!?]+', text)
        
        for sentence in sentences:
            sentence = sentence.strip()
            if len(sentence) < 5:
                continue
            
            words = sentence.split()
            if len(words) < 2:
                continue
            
            # Learn templates
            self._learn_template(words)
            
            # Learn n-grams
            self._learn_ngrams(words)
            
            # Learn grammar
            self._learn_grammar(words)
    
    def _learn_template(self, words):
        """Learn sentence pattern."""
        # Create template with POS-like tags
        template = []
        for i, word in enumerate(words):
            word_lower = word.lower().strip('.,!?')
            
            # Simple category detection
            if word_lower in ['the', 'a', 'an', 'my', 'your', 'his', 'her']:
                template.append('DET')
            elif word_lower in ['is', 'was', 'are', 'were', 'be', 'been']:
                template.append('BE')
            elif word_lower in ['i', 'you', 'he', 'she', 'it', 'we', 'they']:
                template.append('PRON')
            elif word_lower.endswith('ing'):
                template.append('VERB')
            elif word_lower.endswith('ed'):
                template.append('VERB')
            elif word_lower in ['not', 'no', 'never']:
                template.append('NEG')
            elif word[0].isupper() and len(word) > 2:
                template.append('PROPN')
            elif i > 0 and words[i-1].lower() in ['a', 'the', 'is', 'was']:
                template.append('NOUN')
            else:
                template.append('WORD')
        
        # Store template with actual words for filling
        if len(words) >= 2:
            self.templates.append({
                'pattern': template,
                'words': [w.lower().strip('.,!?') for w in words],
                'original': ' '.join(words)
            })
    
    def _learn_ngrams(self, words):
        """Learn common word sequences."""
        for i in range(len(words) - 1):
            bigram = (words[i].lower().strip('.,!?'), 
                     words[i+1].lower().strip('.,!?'))
            self.bigrams[bigram] += 1
            
            if i < len(words) - 2:
                trigram = (words[i].lower().strip('.,!?'),
                          words[i+1].lower().strip('.,!?'),
                          words[i+2].lower().strip('.,!?'))
                self.trigrams[trigram] += 1
    
    def _learn_grammar(self, words):
        """Learn simple subject-verb-object patterns."""
        words_lower = [w.lower().strip('.,!?') for w in words]
        
        # Find verb positions
        verb_positions = []
        for i, w in enumerate(words_lower):
            if w in ['is', 'was', 'are', 'were', 'be', 'been', 'have', 'has', 'had',
                    'do', 'does', 'did', 'say', 'said', 'go', 'went', 'come', 'came',
                    'love', 'loved', 'know', 'knew', 'think', 'thought']:
                verb_positions.append(i)
        
        # Learn subject -> verb
        for pos in verb_positions:
            if pos > 0:
                subject = words_lower[pos-1]
                self.subject_verbs[subject].add(words_lower[pos])
    
    def build_sentence(self, context=None):
        """Build a sentence from learned patterns."""
        if not self.templates:
            return "I do not know enough words yet."
        
        # Try to find a good template
        templates = [t for t in self.templates if len(t['words']) >= 4]
        
        if not templates:
            templates = self.templates
        
        # Pick random template
        template = random.choice(templates)
        
        # For now, return a known sentence as response
        return template['original']
    
    def respond_to(self, input_text):
        """Respond to input - simple pattern matching."""
        input_lower = input_text.lower()
        
        # Check what user asked about
        if any(w in input_lower for w in ['who', 'what', 'where', 'when', 'how']):
            # Question - try to answer from learned patterns
            if 'name' in input_lower:
                # Find sentences with names
                for t in self.templates:
                    if any(w[0].isupper() for w in t['words'] if len(w) > 1):
                        return t['original']
            
            # General question response
            templates = [t for t in self.templates if len(t['words']) >= 3]
            if templates:
                return random.choice(templates)['original']
        
        # Otherwise return something learned
        return self.build_sentence()


# Test
if __name__ == "__main__":
    builder = SentenceBuilder()
    
    # Learn from stories
    story = """
    Once upon a time there were four little Rabbits. Their names were Flopsy Mopsy Cotton-tail and Peter. They lived with their Mother. The little rabbit was very naughty. He ran straight away. He ate some lettuces and some beans. The Wolf and the Kid. The little Kid knew there was little hope. Please Mr Wolf I know you are going to eat me. Do not let anything turn you from your purpose. The Tortoise and the Ducks. The Tortoise was very glad. He seized the stick firmly. Away they sailed up toward the clouds. Foolish curiosity often leads to misfortune. The Lion and the Mouse. A Lion lay asleep in the shade. The little Mouse ran bump right onto the Lion's nose. A kindness is never wasted. The Hare and the Tortoise. The Hare made fun of the Tortoise. Slow and steady wins the race. I am happy. You are kind. He was sad. She was loved.
    """
    
    builder.learn_from_text(story)
    
    print("Testing sentence building:")
    print(builder.respond_to("who are you"))
    print(builder.respond_to("what do you know"))
    print(builder.build_sentence())
