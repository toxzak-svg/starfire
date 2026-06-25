$benchLog = "C:\Users\Zwmar\projects\starfire\models\release_bench_" + (Get-Date -Format "HHmmss") + ".log"
$benchErr = $benchLog -replace "\.log$", ".err"
$proc = Start-Process -FilePath "C:\Users\Zwmar\projects\starfire\target\release\train_model.exe" `
    -ArgumentList "--data", "data\star_corpus_v1.txt", "--epochs", "1", "--output", "data\star_model_release_bench.bin", "--save-every", "999999" `
    -RedirectStandardOutput $benchLog -RedirectStandardError $benchErr `
    -PassThru -NoNewWindow -WorkingDirectory "C:\Users\Zwmar\projects\starfire"
Write-Host "Started PID $($proc.Id) at $(Get-Date -Format 'HH:mm:ss')"
Write-Host "Log: $benchLog"
$proc | Export-Clixml "C:\Users\Zwmar\projects\starfire\models\release_bench_proc.xml"
