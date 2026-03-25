# Infant Sentence Builder Plan

## Goal: Make Infant speak in full sentences without traditional LLM

## Current State
- Word knowledge: 1000+ words
- No sentence formation
- Just lists words

## Approach: Pattern-Based Sentence Building

### 1. Learn Sentence Templates (from stories)
- Extract common sentence patterns
- Store as templates with placeholders

Example patterns learned from stories:
- "The [animal] [verb] [action]"
- "[Subject] was [adjective]"
- "I am [state]"

### 2. N-gram Word Sequences
- Learn common 2-3 word sequences
- "the rabbit" "was very" "so happy"

### 3. Grammar Tags
- Tag each word: noun, verb, adjective, etc.
- Use tags to build valid sentences

### 4. Response Generation
- Match input to learned patterns
- Fill with known words

## Implementation Steps

### Phase 1: Template Learning
- Parse stories for sentence patterns
- Store templates with word categories

### Phase 2: Response Builder
- Input → categorize → match template → fill → output

### Phase 3: Refinement
- Learn from corrections
- Improve grammar

## Target Output
User: "how are you?"
Infant: "I am happy to see you" (full sentence)
