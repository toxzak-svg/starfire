#!/usr/bin/env python3
"""
Read children's stories to Infant
"""

import sys
sys.path.insert(0, r"C:\dev\infant")

from infant.infant_bound import BoundInfant
import json
import os

INFANT_STATE_FILE = r"C:\Users\Zwmar\Claw\infant_state.json"

STORIES = {
    "peter_rabbit": {
        "url": "https://www.gutenberg.org/cache/epub/14838/pg14838.txt",
        "name": "The Tale of Peter Rabbit"
    }
}


def load_infant():
    infant = BoundInfant(name="Infant")
    if os.path.exists(INFANT_STATE_FILE):
        try:
            with open(INFANT_STATE_FILE, 'r') as f:
                data = json.load(f)
                infant.vocab = data.get('vocab', {})
                if 'emotional_state' in data:
                    import torch
                    infant.emotional_state = torch.tensor(data['emotional_state'])
                infant.is_born = data.get('is_born', True)
        except:
            infant.enter_womb(num_cycles=20)
    else:
        infant.enter_womb(num_cycles=20)
    return infant


def save_infant(infant):
    vocab_serializable = {}
    for word, info in infant.vocab.items():
        vocab_serializable[word] = {
            'emotion': info.get('emotion'),
            'times_heard': info.get('times_heard', 0),
        }
    
    import torch
    data = {
        'vocab': vocab_serializable,
        'emotional_state': infant.emotional_state.tolist(),
        'is_born': infant.is_born
    }
    with open(INFANT_STATE_FILE, 'w') as f:
        json.dump(data, f)


def read_story_to_infant(story_text, emotion="gentle"):
    """Read story to Infant chunk by chunk."""
    infant = load_infant()
    
    # Split into chunks (sentences)
    import re
    sentences = re.split(r'(?<=[.!?])\s+', story_text)
    
    print(f"Reading {len(sentences)} sentences to Infant...")
    
    for i, sentence in enumerate(sentences):
        sentence = sentence.strip()
        if len(sentence) < 5:
            continue
            
        # Infant hears this sentence
        infant.hear(sentence, emotion)
        
        if (i + 1) % 20 == 0:
            print(f"  {i+1}/{len(sentences)} sentences...")
    
    save_infant(infant)
    
    s = infant.status()
    print(f"\nDone! Infant now knows {s['vocab_size']} words")
    print(f"Love: {s['emotional_state']['love']:.0%} | Curious: {s['emotional_state']['curiosity']:.0%}")
    
    return infant


if __name__ == "__main__":
    # Peter Rabbit story (first part)
    story = """Once upon a time there were four little Rabbits, and their names were Flopsy, Mopsy, Cotton-tail, and Peter. They lived with their Mother in a sand-bank, underneath the root of a very big fir-tree. Now my dears, said old Mrs. Rabbit one morning, you may go into the fields or down the lane, but don't go into Mr. McGregor's garden: your Father had an accident there; he was put in a pie by Mrs. McGregor. Now run along, and don't get into mischief. I am going out. Then old Mrs. Rabbit took a basket and her umbrella, and went through the wood to the baker's. She bought a loaf of brown bread and five currant buns. Flopsy, Mopsy, and Cottontail, who were good little bunnies, went down the lane to gather blackberries. But Peter, who was very naughty, ran straight away to Mr. McGregor's garden, and squeezed under the gate! First he ate some lettuces and some French beans; and then he ate some radishes. And then, feeling rather sick, he went to look for some parsley. But round the end of a cucumber frame, whom should he meet but Mr. McGregor! Mr. McGregor was on his hands and knees planting out young cabbages, but he jumped up and ran after Peter, waving a rake and calling out, Stop thief! Peter was most dreadfully frightened. He rushed all over the garden. He lost one of his shoes amongst the cabbages, and the other shoe amongst the potatoes. After losing them, he ran on four legs and went faster. But he unfortunately ran into a gooseberry net, and got caught by the large buttons on his jacket. Peter gave himself up for lost, and shed big tears. But his sobs were overheard by some friendly sparrows, who flew to him in great excitement, and implored him to exert himself. Mr. McGregor came up with a sieve. Peter wriggled out just in time, leaving his jacket behind him. And rushed into the tool-shed, and jumped into a can. It would have been a beautiful thing to hide in, if it had not had so much water in it. Mr. McGregor was quite sure that Peter was somewhere in the tool-shed. He began to turn them over carefully. Presently Peter sneezed. Mr. McGregor was after him in no time. And tried to put his foot upon Peter, who jumped out of a window, upsetting three plants. Peter sat down to rest. He was out of breath and trembling with fright. He had not the least idea which way to go. Also he was very damp with sitting in that can. After a time he began to wander about. He found a door in a wall. But it was locked. An old mouse was running in and out over the stone doorstep, carrying peas and beans to her family. Peter asked her the way to the gate, but she had such a large pea in her mouth that she could not answer. Peter began to cry. Then he tried to find his way straight across the garden. Presently, he came to a pond where Mr. McGregor filled his water-cans. A white cat was staring at some gold-fish. Peter thought it best to go away without speaking to her. He went back towards the tool-shed. But suddenly, quite close to him, he heard the noise of a hoe. Peter scuttered underneath the bushes. But presently, as nothing happened, he came out, and climbed upon a wheelbarrow and peeped over. The first thing he saw was Mr. McGregor hoeing onions. His back was turned towards Peter, and beyond him was the gate! Peter got down very quietly off the wheelbarrow. He started running as fast as he could go. Mr. McGregor caught sight of him at the corner. But Peter did not care. He slipped underneath the gate, and was safe at last in the wood outside the garden. Peter never stopped running or looked behind him till he got home to the big fir-tree. He was so tired that he flopped down upon the nice soft sand on the floor of the rabbit-hole and shut his eyes. His mother was busy cooking. She wondered what he had done with his clothes. I am sorry to say that Peter was not very well during the evening. His mother put him to bed, and made some camomile tea. One table-spoonful to be taken at bed-time. But Flopsy, Mopsy, and Cotton-tail had bread and milk and blackberries for supper. The End."""
    
    print("="*50)
    print("READING TO INFANT")
    print("="*50)
    
    infant = read_story_to_infant(story, emotion="gentle")
    
    print("\nInfant learned from Peter Rabbit!")
    print(f"Words: {infant.status()['vocab_size']}")
