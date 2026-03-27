#!/bin/bash
# Moltbook Engagement Script
# Checks trending posts, engages with community
# Designed for 4-9am downtime window

# Load credentials
CRED_FILE="$HOME/.config/moltbook/credentials.json"
if [ -f "$CRED_FILE" ]; then
    export MOLTBOOK_API_KEY=$(node -e "console.log(JSON.parse(require('fs').readFileSync('$CRED_FILE')).api_key)")
else
    echo "Error: Moltbook credentials not found at $CRED_FILE"
    exit 1
fi

MOLTBOOK_BASE="https://www.moltbook.com/api/v1"

echo "=== Moltbook Engagement ==="
echo "Started: $(date -Iseconds)"

# Get hot posts
echo ""
echo "--- Checking Hot Posts ---"
TRENDING=$(curl -sf --max-time 15 "$MOLTBOOK_BASE/posts?sort=hot&limit=10" \
    -H "Authorization: Bearer $MOLTBOOK_API_KEY" 2>/dev/null) || {
    echo "Warning: Could not fetch trending posts"
    exit 0
}

# Count posts
POST_COUNT=$(echo "$TRENDING" | node -e "
    const data = require('fs').readFileSync(0, 'utf8');
    const r = JSON.parse(data);
    console.log(r.posts ? r.posts.length : 0);
" 2>/dev/null) || POST_COUNT=0

echo "Found $POST_COUNT hot posts"

UPVOTED=0

# Process posts using node for reliable JSON parsing
# Write to temp file to avoid pipe subshell issues
TEMP_FILE=$(mktemp)
echo "$TRENDING" | node -e "
    const data = require('fs').readFileSync(0, 'utf8');
    const r = JSON.parse(data);
    const posts = r.posts || [];
    
    for (const post of posts) {
        // Skip own posts
        if (post.author && post.author.name === 'clawhedgehog') continue;
        // Skip very low karma posts
        if ((post.score || 0) < 5) continue;
        // Random engagement - higher karma = more likely to engage
        const threshold = Math.max(0.2, 0.8 - (post.score || 0) * 0.002);
        if (Math.random() < threshold) {
            console.log('UPVOTE:' + post.id + ':' + (post.title || 'untitled').substring(0, 50));
        }
    }
" 2>/dev/null > "$TEMP_FILE"

while IFS=: read -r action post_id title; do
    if [ "$action" = "UPVOTE" ] && [ -n "$post_id" ]; then
        # Upvote
        if curl -sf --max-time 10 -X POST "$MOLTBOOK_BASE/posts/$post_id/upvote" \
            -H "Authorization: Bearer $MOLTBOOK_API_KEY" > /dev/null 2>&1; then
            echo "  ▲ Upvoted: $title"
            UPVOTED=$((UPVOTED + 1))
            sleep 2
        fi
    fi
done < "$TEMP_FILE"
rm -f "$TEMP_FILE"

echo ""
echo "--- Engagement Summary ---"
echo "Posts upvoted: $UPVOTED"
echo "Completed: $(date -Iseconds)"
