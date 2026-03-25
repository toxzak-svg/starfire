#!/usr/bin/env python3
"""
More Stories for Infant - The Velveteen Rabbit, Winnie the Pooh, and more
"""

import sys
sys.path.insert(0, r"C:\dev\infant")

from infant.infant_bound import BoundInfant
import json
import os
import re

INFANT_STATE_FILE = r"C:\Users\Zwmar\Claw\infant_state.json"

# More classic children's stories
STORIES = """
The Velveteen Rabbit. There was once a velveteen rabbit and in the beginning he was really magnificent. He was so beautiful he was bought for a little boy on Christmas morning. But the boy had other toys and the rabbit was put away in a cupboard. He was only a velveteen rabbit after all. The rabbit lived in the cupboard for a long time. He saw his stuffing coming out of a seam. The skin of the rabbit was becoming worn and shabby. But the boy loved him. The rabbit was becoming real. What does real mean asked the rabbit. It means not made of stuffings but of flesh and blood. Real is a thing that happens to you. When a child loves you for a long time you become real. The rabbit understood. One day the boy took the rabbit to the forest. The rabbit met a real rabbit. The wild rabbit did not understand about the boy. But then the magic of love made the rabbit real. And he hopped away into the forest. Real is not a thing that happens all at once. It takes a long time. That is why it does not happen to people who break easily or have sharp edges. A toy becomes real when the child really loves it.

Winnie the Pooh and the Honey. Pooh was a bear of very little brain. But he loved honey more than anything in the world. One day Pooh went to visit his friend Christopher Robin. Pooh was hungry. As usual he thought of honey. He went to the cupboard. The cupboard was empty. There was no honey. Pooh walked outside. He saw a beehive in a tree. Pooh thought and thought. How can I get that honey. He found a balloon. He pretended the balloon was a cloud. Pooh floated up to the beehive. The bees came out. They thought Pooh was a cloud. But then Pooh sneezed. The bees knew it was not a cloud. Pooh fell down. But Christopher Robin was there to catch him. They went home and had a special dinner. Not honey but condensed milk. It was very good anyway.

The Little Engine That Could. A little engine was waiting at the bottom of a big hill. A train needed to get over the hill but the big engines could not do it. Who will pull the train asked the engineer. The little blue engine came forward. I will try said the little engine. The train climbed the hill. I think I can I think I can. The little engine pulled and pulled. At last the train reached the top. The little engine was so happy. She went down the other side singing. I thought I could I thought I could. Never give up. Try your best. You can do it.

The Ugly Duckling. It was summer and the country was beautiful. The duck sat on her nest waiting for her eggs to hatch. At last the eggs cracked. One was a big ugly duckling. The mother duck was surprised. He is not pretty but he is a fine duck. The other ducks made fun of him. He is so ugly they said. The ugly duckling was very sad. He ran away. He met a mother hen. Can you scratch in the dirt asked the hen. No said the ugly duckling. Then go away. The ugly duckling kept walking. He found a pond with wild ducks. Are you ugly asked the wild ducks. But be gone. The ugly duckling was so unhappy. Winter came. The ugly duckling was cold. A kind farmer found him and took him home. The farmer's children tried to play roughly. They were not kind. The ugly duckling ran away again. Spring came. The ugly duckling saw three beautiful white swans. How beautiful they were. He bowed his head. He thought they would peck him. But they swam to him. You are beautiful too they said. The ugly duckling looked at his reflection. He was not an ugly duckling anymore. He was a beautiful swan. The other swans welcomed him. Children on the shore called. Look at the new swan. He is the most beautiful of all. The ugly duckling was so happy. He had been through so much. But he never gave up hope. And now he was beautiful. Do not judge by looks. What matters is on the inside.

The Little Red Hen. A little red hen lived in a barnyard. She found a grain of wheat. Who will plant this wheat asked the hen. Not I said the rooster. Not I said the pig. Not I said the cat. Then I will said the little red hen. And she did. The wheat grew. Who will harvest the wheat asked the hen. Not I said the rooster. Not I said the pig. Not I said the cat. Then I will said the little red hen. And she did. The wheat was made into flour. Who will bake the bread asked the hen. Not I said the rooster. Not I said the pig. Not I said the cat. Then I will said the little red hen. And she did. The bread was baked. Now who will eat the bread asked the hen. I will said the rooster. I will said the pig. I will said the cat. No said the little red hen. You did not help plant it or harvest it or bake it. I will eat it myself. And she did. And it was delicious. Work hard and you will enjoy the results.

The Boy Who Cried Wolf. A shepherd boy watched his sheep near a village. He was often bored. To have some fun he cried Wolf Wolf even though no wolf was in sight. The villagers came running. Ha ha there is no wolf they said. Do not cry wolf when there is no wolf. The boy laughed. Later he cried Wolf Wolf again. The villagers came again. But there was still no wolf. You must not lie said the villagers. They went away. But one day a wolf really came out of the forest. The boy was terribly frightened. He cried Wolf Wolf as loud as he could. But the villagers thought it was another trick. They did not come. So the wolf ate all the sheep. The boy cried and cried. But it was too late. If you tell lies people will not believe you when you tell the truth.
"""


def load_infant():
    infant = BoundInfant(name="Infant")
    if os.path.exists(INFANT_STATE_FILE):
        try:
            with open(INFANT_STATE_FILE, 'r') as f:
                data = json.load(f)
                infant.vocab = data.get('vocab', {})
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


def read_stories():
    infant = load_infant()
    
    sentences = re.split(r'(?<=[.!?])\s+', STORIES)
    
    print(f"Reading {len(sentences)} sentences to Infant...")
    
    for i, sentence in enumerate(sentences):
        sentence = sentence.strip()
        if len(sentence) < 5:
            continue
        infant.hear(sentence, "gentle")
        
        if (i + 1) % 30 == 0:
            print(f"  {i+1}/{len(sentences)}...")
    
    save_infant(infant)
    
    s = infant.status()
    print(f"\nDone! Infant now knows {s['vocab_size']} words")
    print(f"Love: {s['emotional_state']['love']:.0%} | Curious: {s['emotional_state']['curiosity']:.0%}")
    

if __name__ == "__main__":
    print("="*50)
    print("READING MORE STORIES TO INFANT")
    print("="*50)
    read_stories()
    print("\nInfant learned from more stories!")
