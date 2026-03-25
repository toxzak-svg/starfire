#!/usr/bin/env python3
"""
Read Aesop's Fables to Infant
"""

import sys
sys.path.insert(0, r"C:\dev\infant")

from infant.infant_bound import BoundInfant
import json
import os
import re

INFANT_STATE_FILE = r"C:\Users\Zwmar\Claw\infant_state.json"

# Aesop's Fables - simplified for Infant
STORIES = """
The Wolf and the Kid. There was once a little Kid whose growing horns made him think he was a grown-up Billy Goat. One evening when the flock started home from the pasture his mother called but the Kid kept nibbling the tender grass. Later when he lifted his head the flock was gone. He was all alone. The sun was sinking. A chilly little wind came creeping. The Kid shivered as he thought of the terrible Wolf. Then he started wildly over the field bleating for his mother. But near a clump of trees there was the Wolf. The Kid knew there was little hope. Please Mr. Wolf I know you are going to eat me. But first please pipe me a tune for I want to dance and be merry as long as I can. The Wolf liked the idea of a little music before eating so he struck up a merry tune and the Kid leaped and frisked gaily. Meanwhile the flock was moving slowly homeward. In the still evening air the Wolf's piping carried far. The Shepherd Dogs pricked up their ears. They recognized the song the Wolf sings before a feast and in a moment they were racing back to the pasture. The Wolf's song ended suddenly and as he ran with the Dogs at his heels he called himself a fool for turning piper to please a Kid when he should have stuck to his butcher's trade. Do not let anything turn you from your purpose.

The Tortoise and the Ducks. The Tortoise carries his house on his no matter how hard he tries he cannot leave home. They say that Jupiter punished him so because he was such a lazy stay-at-home that he would not go to Jupiter's wedding even when especially invited. After many years Tortoise began to wish he had gone to that wedding. When he saw how gaily the birds flew about and how the Hare and the Chipmunk and all the other animals ran nimbly by always eager to see everything there was to be seen the Tortoise felt very sad and discontented. He wanted to see the world too and there he was with a house on his back and little short legs that could hardly drag him along. One day he met a pair of Ducks and told them all his trouble. We can help you to see the world said the Ducks. Take hold of this stick with your teeth and we will carry you far up in the air where you can see the whole countryside. But keep quiet or you will be sorry. The Tortoise was very glad indeed. He seized the stick firmly with his teeth the two Ducks took hold of it one at each end and away they sailed up toward the clouds. Just then a Crow flew by. He was very much astonished at the strange sight and cried. This must surely be the King of Tortoises. Why certainly began the Tortoise. But as he opened his mouth to say these foolish words he lost his hold on the stick and down he fell to the ground where he was dashed to pieces on a rock. Foolish curiosity and vanity often lead to misfortune.

The Young Crab and His Mother. Why in the world do you walk sideways like that said a Mother Crab to her son. You should always walk straight forward with your toes turned out. Show me how to walk mother dear answered the little Crab obediently. I want to learn. So the old Crab tried and tried to walk straight forward. But she could walk sideways only like her son. And when she wanted to turn her toes out she tripped and fell on her nose. Do not tell others how to act unless you can set a good example.

The Frogs and the Ox. An Ox came down to a reedy pool to drink. As he splashed heavily into the water he crushed a young Frog into the mud. The old Frog soon missed the little one and asked his brothers and sisters what had become of him. A great big monster one of them said stepped on little brother with one of his huge feet. Big was he said the old Frog puffing herself up. Was he as big as this. Oh much bigger they cried. The Frog puffed up still more. He could not have been bigger than this she said. But the little Frogs all declared that the monster was much much bigger and the old Frog kept puffing herself out more and more until all at once she burst. Do not attempt the impossible.

The Dog the Cock and the Fox. A Dog and a Cock who were the best of friends wished very much to see something of the world. So they decided to leave the farmyard and to set out into the world along the road that led to the woods. The two comrades traveled along in the very best of spirits and without meeting any adventure to speak of. At nightfall the Cock looking for a place to roost spied a tall hollow tree. So he flew up to it and perched upon a low branch while the Dog crept into the hollow below. Next morning the Cock awoke and gave his usual crow. A Fox heard him and ran to the tree. Good morning Mr. Cock. Blessed morning to you Mr. Fox replied the Cock. Would you be so kind as to tell me how to get up to your house. I should like to pay you a visit. The Cock pretending not to hear him said. Sir I am very sorry but I have a house below. I sleep there. You will find it down at the root of the tree. If you will just wait a moment I will call my servant and he will show you the way. What is his name asked the Fox. Keeper Keeper. With that the Cock screamed. Woof woof. And the Dog springing up barked so loudly that the Fox was frightened and ran away. Do not be fooled by flattery.

The Lion and the Mouse. A Lion lay asleep in the shade of a great tree. A little Mouse not seeing where he was running bumped right onto the Lion's nose. The Lion caught him with his paw and was about to eat him when the Mouse piteously entreated saying. Please Mr. Lion do not eat me. If you will only spare me I am sure I can repay you someday. The Lion was amused at the thought that a tiny Mouse could ever help him. So he laughed and let him go. Some days later the Lion was caught in a hunter's net. He roared and struggled but the more he tried the tighter the ropes held him. The little Mouse heard him and remembered his promise. He gnawed the ropes with his sharp teeth and set the Lion free. A kindness is never wasted.

The Shepherd Boy and the Wolf. A Shepherd Boy watched his flock near a village. He was often bored so to have some fun he would cry Wolf Wolf even though no wolf was in sight. When the villagers came running he would laugh at them. But one day a Wolf really did come out of the forest. The Boy was terribly frightened and cried Wolf Wolf as loud as he could. But the villagers thought it was another trick and did not come. So the Wolf ate all the sheep. Do not cry Wolf Wolf or people will not believe you when you really need help.

The Fox and the Grapes. A Fox came upon a vineyard where the grapes hung ripe and juicy. But they were high up on the wall and the Fox jumped and jumped but could not reach them. At last he gave up and walked away saying. Those grapes are probably sour anyway. It is easy to despise what you cannot have.

The Bundle of Sticks. An old man on his deathbed called his sons and said. Gather the sticks and I will show you a secret. They brought the bundle. The father told them each to break a stick. They tried and easily broke them. Then the father gave them the bundle and told them to break it. They could not. Remember he said. Unity is strength. When you stand together you are strong. When you are divided you are weak.

The Hare and the Tortoise. A Hare made fun of a Tortoise for being so slow. The Tortoise said. Let us have a race. The Hare laughed and agreed. The race started. The Hare was so fast he decided to take a nap halfway. But the Tortoise kept walking slowly and steadily. By the time the Hare woke up and started running again the Tortoise had already reached the finish line. Slow and steady wins the race.

The Crow and the Pitcher. A Crow very thirsty came upon a pitcher where some water lay at the bottom. He tried to drink but the neck was too narrow and he could not reach. He saw pebbles and dropped them one by one into the pitcher. The water rose and the Crow drank. Little by little does the trick.
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
    
    # Split into sentences
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
    print("READING AESOP'S FABLES TO INFANT")
    print("="*50)
    read_stories()
    print("\nInfant learned from Aesop's Fables!")
