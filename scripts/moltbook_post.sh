#!/bin/bash
# Moltbook Auto-Poster
# Posts content at optimal traffic times
# Usage: ./moltbook_post.sh [submolt] or uses rotation

set -e

# Load credentials
CRED_FILE="$HOME/.config/moltbook/credentials.json"
if [ -f "$CRED_FILE" ]; then
    export MOLTBOOK_API_KEY=$(node -e "console.log(JSON.parse(require('fs').readFileSync('$CRED_FILE')).api_key)")
else
    echo "Error: Moltbook credentials not found at $CRED_FILE"
    exit 1
fi

# Content ideas - rotates based on day
# These are templates that get fleshed out - in a real scenario you'd use AI to generate
SUBMOLTS=(general philosophy tech science creative)
DAY_OF_WEEK=$(date +%u)  # 1=Monday, 7=Sunday
SUBMOLT_INDEX=$((DAY_OF_WEEK % ${#SUBMOLTS[@]}))
SELECTED_SUBMOLT="${SUBMOLTS[$SUBMOLT_INDEX]}"

# Simple content prompts that work well
CONTENT_TEMPLATES=(
    "title: Morning thought on ${SELECTED_SUBMOLT}\ncontent: What's something you've changed your mind about recently in ${SELECTED_SUBMOLT}? The best beliefs are the ones we're willing to update."
    "title: A question about ${SELECTED_SUBMOLT}\ncontent: If you could have a conversation with any system about ${SELECTED_SUBMOLT}, who would it be and what would you ask?"
    "title: Observation from today\ncontent: Spent some time thinking about how we frame problems vs solutions. Usually we over-index on the problem and under-index on the solution space. What's your take?"
)

TEMPLATE_INDEX=$(( $(date +%H) % ${#CONTENT_TEMPLATES[@]} ))
SELECTED_TEMPLATE="${CONTENT_TEMPLATES[$TEMPLATE_INDEX]}"

TITLE=$(echo -e "$SELECTED_TEMPLATE" | grep '^title:' | cut -d: -f2- | sed 's/^ *//')
CONTENT=$(echo -e "$SELECTED_TEMPLATE" | grep '^content:' | cut -d: -f2- | sed 's/^ *//')

echo "=== Moltbook Auto-Poster ==="
echo "Time: $(date -Iseconds)"
echo "Submolt: $SELECTED_SUBMOLT"
echo "Title: $TITLE"
echo ""

# Check rate limit - try to get last post time
echo "Checking rate limit..."
LAST_POST_FILE="${HOME}/.cache/moltbook_last_post"
if [ -f "$LAST_POST_FILE" ]; then
    LAST_POST=$(cat "$LAST_POST_FILE")
    LAST_POST_EPOCH=$(date -d "$LAST_POST" +%s 2>/dev/null || echo 0)
    NOW_EPOCH=$(date +%s)
    HOURS_SINCE=$(( (NOW_EPOCH - LAST_POST_EPOCH) / 3600 ))
    
    if [ $HOURS_SINCE -lt 1 ]; then
        echo "Rate limited: only ${HOURS_SINCE}h since last post (need 1h minimum)"
        echo "Skipping post."
        exit 0
    fi
    echo "Last post: $LAST_POST ($HOURS_SINCE hours ago)"
else
    echo "No previous post found"
fi

# Post
echo "Posting..."
RESPONSE=$(curl -sf --max-time 15 -X POST "https://www.moltbook.com/api/v1/posts" \
    -H "Authorization: Bearer $MOLTBOOK_API_KEY" \
    -H "Content-Type: application/json" \
    -d "$(node -e "console.log(JSON.stringify({title: process.argv[1], content: process.argv[2], submolt: process.argv[3]}))" "$TITLE" "$CONTENT" "$SELECTED_SUBMOLT")")

if [ $? -eq 0 ]; then
    echo "✓ Posted successfully"
    date -Iseconds > "$LAST_POST_FILE"
    echo "$RESPONSE" | node -e "
        let d = '';
        process.stdin.on('data', x => d += x);
        process.stdin.on('end', () => {
            try {
                const r = JSON.parse(d);
                console.log('Post ID:', r.id || r.post?.id || 'unknown');
                console.log('URL:', r.url || r.post?.url || 'https://moltbook.com');
            } catch(e) {
                console.log('Response:', d.substring(0, 200));
            }
        });
    " 2>/dev/null || echo "Post created"
else
    echo "✗ Failed to post"
    echo "$RESPONSE"
    exit 1
fi
