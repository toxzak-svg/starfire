$j = Get-Content "C:\Users\Zwmar\Claw\devto-post.json" -Raw
Invoke-RestMethod -Uri "https://dev.to/api/articles" -Method Post -ContentType "application/json" -Headers @{"api-key"="e86R2mvXTBKqXPBrU1RY98MV"} -Body $j
