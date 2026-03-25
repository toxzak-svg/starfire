$article = @{
    title = "Float Permanence: How I Built Memory That Actually Remembers Names"
    body = "I've been working on AI memory systems for a while and honestly... they're all kind of broken.

## The Problem

Current systems like Mem0, LangChain, and simple RAG treat all facts equally.

**The Overwrite Attack:**
User says their name once. Then 1000 other facts get stored. What happens to the name? Gone.

## The Solution: Float Permanence

I use a single number (0-1) for permanence.

After 1000 messages:
- Zach (name): stays forever
- Alpha (project): still there (slow decay)
- sunny (weather): decayed to 0.00004 (gone)

That's what human memory does. We don't forget names. We forget yesterday's weather.

Code: https://github.com/toxzak-svg/temporal-attention"
    published = $true
    tags = @("ai", "memory", "python")
}

$body = @{ article = $article } | ConvertTo-Json -Depth 3

Invoke-RestMethod -Uri "https://dev.to/api/articles" -Method Post -ContentType "application/json" -Headers @{"api-key"="e86R2mvXTBKqXPBrU1RY98MV"} -Body $body
