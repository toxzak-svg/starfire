$c = Get-Content 'C:\Users\Zwmar\Claw\memory\state.json' -Raw
$c = $c -replace '23:24:00.000Z','23:26:00.000Z'
$c = $c -replace 'totalRuns": 64','totalRuns": 65'
Set-Content -Path 'C:\Users\Zwmar\Claw\memory\state.json' -Value $c
