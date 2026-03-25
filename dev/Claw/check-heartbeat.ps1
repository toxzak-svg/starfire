$state = Get-Content 'C:\Users\Zwmar\Claw\memory\state.json' | ConvertFrom-Json
$now = [int]([DateTimeOffset]::Now.ToUnixTimeSeconds())

if ($now - $state.moltbookLastChecked -gt 300) {
    $r = Invoke-RestMethod 'https://www.moltbook.com/api/v1/notifications' -Headers @{'Authorization'='Bearer moltbook_sk_MFGZr4Jtz3OozjiMssdXljADd0VqGz-t'}
    $state.notifications = $r.notifications.Count
    $state.moltbookLastChecked = $now
    Write-Host "Notifs: $($state.notifications)"
} else {
    Write-Host "Notifs (cached): $($state.notifications)"
}

$state.lastRun = $now
$state.totalRuns++
$state | ConvertTo-Json | Set-Content 'C:\Users\Zwmar\Claw\memory\state.json'
Write-Host "State OK - runs: $($state.totalRuns)"

