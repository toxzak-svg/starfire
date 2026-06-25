# V29 training watcher — polls model.bin mtime, logs save events + ETA
# Usage: powershell -ExecutionPolicy Bypass -File watch_v29.ps1

param(
    [string]$ModelPath = "C:\Users\Zwmar\projects\starfire\data\star_model_v29.bin",
    [string]$LogPath = "C:\Users\Zwmar\projects\starfire\models\v29_watch.log",
    [int]$PollSeconds = 30,
    [int]$TotalBatches = 18200,
    [int]$TotalEpochs = 10,
    [int]$BatchesPerSave = 16
)

# Initial estimate from first run: 60 batches in 6h46m = 6.77 min/batch
$script:BatchRateMin = 6.77
$script:LastSaveTime = $null
$script:LastBatchCount = 0

function Write-Log {
    param([string]$Msg)
    $ts = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $line = "[$ts] $Msg"
    Add-Content -Path $LogPath -Value $line
    Write-Host $line
}

function Format-Duration {
    param([double]$Hours)
    if ($Hours -ge 720) { return ("{0:N1} months" -f ($Hours / 720)) }
    if ($Hours -ge 48)  { return ("{0:N1} days" -f ($Hours / 24)) }
    if ($Hours -ge 1)    { return ("{0:N1} hours" -f $Hours) }
    return ("{0:N0} minutes" -f ($Hours * 60))
}

if (Test-Path $ModelPath) {
    $script:LastSaveTime = (Get-Item $ModelPath).LastWriteTime
    Write-Log "Watcher started. Model last saved at $script:LastSaveTime"
} else {
    Write-Log "Watcher started. Model file not found yet."
}

# First save fires at batch_start = 500 seq = batch 16 of epoch 1
$script:LastBatchCount = $BatchesPerSave
Write-Log "Config: total=$TotalBatches batches, $TotalEpochs epochs, ~$BatchesPerSave batches/save"
Write-Log "Initial rate estimate: $([math]::Round($script:BatchRateMin,2)) min/batch (from first run)"

while ($true) {
    Start-Sleep -Seconds $PollSeconds

    if (-not (Test-Path $ModelPath)) { continue }

    $currentMtime = (Get-Item $ModelPath).LastWriteTime
    if ($currentMtime -ne $script:LastSaveTime) {
        $delta = ($currentMtime - $script:LastSaveTime).TotalMinutes
        $batchesDelta = $BatchesPerSave
        $script:LastBatchCount += $batchesDelta

        if ($delta -gt 0) {
            $measuredRate = $delta / $batchesDelta
            $script:BatchRateMin = ($script:BatchRateMin * 0.7) + ($measuredRate * 0.3)
            Write-Log "SAVE @ batch ~$($script:LastBatchCount) | measured: $([math]::Round($measuredRate,2)) min/batch | smoothed: $([math]::Round($script:BatchRateMin,2)) min/batch"
        }

        $remainingBatches = $TotalBatches - $script:LastBatchCount
        $remainingHours = ($remainingBatches * $script:BatchRateMin) / 60
        $epochNum = [math]::Ceiling($script:LastBatchCount / ($TotalBatches / $TotalEpochs))
        $pct = [math]::Round(($script:LastBatchCount / $TotalBatches) * 100, 3)

        $dur = Format-Duration $remainingHours
        Write-Log ("  Progress: batch {0}/{1} ({2}%) | epoch ~{3}/{4}" -f $script:LastBatchCount, $TotalBatches, $pct, $epochNum, $TotalEpochs)
        Write-Log ("  ETA: {0} remaining" -f $dur)

        $script:LastSaveTime = $currentMtime
    }
}