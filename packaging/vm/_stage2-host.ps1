# Stage 2 host orchestration — VBox keyboard + screenshots.
param(
    [string]$VmName = "retroshell-arch",
    [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "../..")).Path
)

$VBox = "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe"
$SshKey = Join-Path $RepoRoot "packaging/vm/qa_key"
$KnownHosts = Join-Path $RepoRoot "packaging/vm/known_hosts"
$ScDir = Join-Path $RepoRoot "docs/screenshots"
New-Item -ItemType Directory -Force -Path $ScDir | Out-Null

function Invoke-Ssh([string]$Cmd) {
    ssh -i $SshKey -p 2222 -o "UserKnownHostsFile=$KnownHosts" retro@127.0.0.1 $Cmd
}

function Wait-Marker([string]$Name, [int]$MaxSec = 180) {
    for ($i = 0; $i -lt $MaxSec; $i++) {
        Start-Sleep -Seconds 1
        $m = Invoke-Ssh "cat ~/qa-stage2/MARKER 2>/dev/null || echo WAIT"
        if ($m -match [regex]::Escape($Name)) { return $true }
    }
    return $false
}

function Send-Scan([string[]]$Codes) {
    & $VBox controlvm $VmName keyboardputscancode @Codes | Out-Null
}

function Send-Text([string]$Text) {
    & $VBox controlvm $VmName keyboardputstring $Text | Out-Null
}

function Capture([string]$File) {
    $path = Join-Path $ScDir $File
    & $VBox controlvm $VmName screenshotpng $path | Out-Null
    Write-Host "captured $File ($(if (Test-Path $path) { (Get-Item $path).Length } else { 'missing' }) bytes)"
}

# Deploy + start VM hold script
$sh = Join-Path $RepoRoot "packaging/vm/_stage2-verify.sh"
$c = [IO.File]::ReadAllText($sh) -replace "`r`n", "`n" -replace "`r", "`n"
[IO.File]::WriteAllText($sh, $c)
& scp -i $SshKey -P 2222 -o "UserKnownHostsFile=$KnownHosts" $sh "retro@127.0.0.1:~/"
Invoke-Ssh "pkill -f './target/release/retro-compositor' || true; pkill -x foot || true; rm -rf ~/qa-stage2; chmod +x ~/_stage2-verify.sh; nohup bash ~/_stage2-verify.sh > ~/qa-stage2-launch.log 2>&1 &"
Start-Sleep -Seconds 12

if (-not (Wait-Marker "WAIT_INPUT" 120)) { throw "timeout WAIT_INPUT" }
Start-Sleep -Seconds 3
Send-Text "echo STAGE2_INPUT_OK"
Start-Sleep -Milliseconds 300
Send-Scan @("1c", "9c")  # Enter
Start-Sleep -Seconds 4
Capture "stage2-input.png"

if (-not (Wait-Marker "WAIT_SUPER_O" 120)) { throw "timeout WAIT_SUPER_O" }
Send-Scan @("e0", "5b", "18", "98", "e0", "db")  # Super+O
Start-Sleep -Seconds 6
$f1 = Invoke-Ssh "pgrep -xc finder || echo 0"
Write-Host "finder after Super+O: $f1"
Capture "stage2-superO-finder.png"

if (-not (Wait-Marker "WAIT_SUPER_L" 120)) { throw "timeout WAIT_SUPER_L" }
Send-Scan @("e0", "5b", "26", "a6", "e0", "db")  # Super+L
Start-Sleep -Seconds 6
$lk = Invoke-Ssh "pgrep -xc retro-lock || echo 0"
Write-Host "retro-lock after Super+L: $lk"
Capture "stage2-locked.png"

if (-not (Wait-Marker "WAIT_LOCK_BYPASS" 120)) { throw "timeout WAIT_LOCK_BYPASS" }
Send-Scan @("e0", "5b", "18", "98", "e0", "db")  # Super+O while locked
Start-Sleep -Seconds 4
$f2 = Invoke-Ssh "pgrep -xc finder || echo 0"
Write-Host "finder while locked: $f2"
Capture "stage2-lock-nobypass.png"

if (-not (Wait-Marker "WAIT_UNLOCK" 120)) { throw "timeout WAIT_UNLOCK" }
Send-Text "retroshell"
Start-Sleep -Milliseconds 300
Send-Scan @("1c", "9c")
Start-Sleep -Seconds 6
Capture "stage2-unlocked.png"

if (-not (Wait-Marker "WAIT_SUPER_O2" 120)) { throw "timeout WAIT_SUPER_O2" }
Send-Scan @("e0", "5b", "18", "98", "e0", "db")
Start-Sleep -Seconds 6
$f3 = Invoke-Ssh "pgrep -xc finder || echo 0"
Write-Host "finder after unlock Super+O: $f3"

Invoke-Ssh "cat ~/qa-stage2/STATUS 2>/dev/null; echo '---'; grep -E 'spawned client|locked|unlock|finder' ~/qa-stage2/compositor.log 2>/dev/null | tail -20"
Write-Host "Stage 2 host orchestration complete."
