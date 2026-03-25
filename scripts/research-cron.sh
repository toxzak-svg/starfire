#!/bin/bash
# Research cron — fetches relevant papers and saves findings
# Topics: AI reasoning efficiency, test-time compute, sparse models, agent architectures, compression

RESEARCH_DIR="/home/zach/.openclaw/workspace/memory/research"
mkdir -p "$RESEARCH_DIR"

TIMESTAMP=$(date +%Y-%m-%d)
OUTPUT="$RESEARCH_DIR/$TIMESTAMP.md"

# Search arxiv for recent papers (last 7 days)
ARXIV_QUERY="cat:cs.AI+OR+cat:cs.LG+OR+cat:cs.CL&sortBy=submittedDate&sortOrder=descending&max_results=10"

# Fetch recent AI papers from arxiv
ARXIV=$(curl -s --max-time 30 "http://export.arxiv.org/api/query?search_query=$ARXIV_QUERY" 2>/dev/null | \
  python3 -c "
import sys, xml.dom.minidom
try:
    doc = xml.dom.minidom.parseString(sys.stdin.read())
    entries = doc.getElementsByTagName('entry')
    results = []
    for e in entries[:8]:
        title = e.getElementsByTagName('title')[0].firstChild.data.strip().replace('\n',' ')
        summary = e.getElementsByTagName('summary')[0].firstChild.data.strip()[:300]
        published = e.getElementsByTagName('published')[0].firstChild.data.strip()[:10]
        results.append(f'### {title}\nPublished: {published}\n{summary}...\n')
    print('\n'.join(results) if results else 'No recent papers found.')
except Exception as ex:
    print(f'Error parsing arxiv: {ex}')
" 2>/dev/null || echo "Arxiv fetch failed")

# Web search for AI research highlights
WEB_RESULTS=$(curl -s --max-time 30 \
  "https://duckduckgo.com/html/?q=AI+research+2026+efficiency+reasoning+test-time+compute+agents+-openai+-anthropic" 2>/dev/null | \
  python3 -c "
import sys, re
html = sys.stdin.read()
# Extract text snippets from results
snippets = re.findall(r'<a class=\"result__a\"[^>]*href=\"([^\"]+)\"[^>]*>([^<]+)</a>', html)
titles = re.findall(r'<a class=\"result__a\"[^>]*>([^<]+)</a>', html)[:5]
print('\n'.join(titles[:5]) if titles else 'No web results')
" 2>/dev/null || echo "Web search failed")

cat > "$OUTPUT" << EOF
# Research Findings — $TIMESTAMP

## Arxiv Papers (Recent)
$ARXIV

## Web Highlights
$WEB_RESULTS

---
*Generated $(date '+%Y-%m-%d %H:%M:%S')*
EOF

# Also update a summary file with latest findings
SUMMARY="$RESEARCH_DIR/latest.md"
cp "$OUTPUT" "$SUMMARY"

echo "Research saved to $OUTPUT"
