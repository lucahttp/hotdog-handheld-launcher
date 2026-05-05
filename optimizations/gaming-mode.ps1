#Requires -RunAsAdministrator

<#
.SYNOPSIS
    Panchito Launcher - Gaming Mode Toggle
.DESCRIPTION
    Applies or reverts system optimizations for gaming on Windows 10/11 handhelds.
    No external tools required — uses built-in PowerShell and Windows commands.
.PARAMETER Rollback
    Switch to revert all optimizations back to default Windows settings.
.PARAMETER DryRun
    Switch to show what would be changed without actually applying changes.
.EXAMPLE
    .\gaming-mode.ps1
    .\gaming-mode.ps1 -Rollback
    .\gaming-mode.ps1 -DryRun
#>

param(
    [switch]$Rollback,
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"

# ---------- helpers ----------
function Write-Header($s) {
    Write-Host "`n================================================" -ForegroundColor Cyan
    Write-Host " $s" -ForegroundColor Cyan
    Write-Host "================================================" -ForegroundColor Cyan
}

function Write-Ok($s) { Write-Host "  [OK] $s" -ForegroundColor Green }
function Write-Skip($s) { Write-Host "  [SKIP] $s" -ForegroundColor Yellow }
function Write-Fail($s) { Write-Host "  [FAIL] $s" -ForegroundColor Red }

function Invoke-ServiceOp {
    param([string]$Name, [string]$Action)  # Action = Stop, Start, Disable, Enable-Auto, Enable-Demand, Enable-Delayed
    if ($DryRun) { Write-Skip "Would $Action service '$Name'"; return }
    try {
        switch ($Action) {
            "Stop"   { Stop-Service -Name $Name -Force -ErrorAction SilentlyContinue; Write-Ok "Stopped '$Name'" }
            "Start"  { Start-Service -Name $Name -ErrorAction SilentlyContinue; Write-Ok "Started '$Name'" }
            "Disable" { Set-Service -Name $Name -StartupType Disabled -ErrorAction SilentlyContinue; Write-Ok "Disabled '$Name'" }
            "Enable-Auto" { Set-Service -Name $Name -StartupType Automatic -ErrorAction SilentlyContinue; Write-Ok "Enabled auto '$Name'" }
            "Enable-Demand" { Set-Service -Name $Name -StartupType Manual -ErrorAction SilentlyContinue; Write-Ok "Enabled manual '$Name'" }
            "Enable-Delayed" { Set-Service -Name $Name -StartupType AutomaticDelayedStart -ErrorAction SilentlyContinue; Write-Ok "Enabled delayed '$Name'" }
        }
    } catch { Write-Fail "$Action '$Name': $_" }
}

function Set-Reg {
    param([string]$Path, [string]$Name, [string]$Type, [object]$Value)
    if ($DryRun) { Write-Skip "Would set '$Path\$Name' = $Value"; return }
    try {
        # Ensure parent key exists
        $parent = Split-Path $Path -Parent
        $leaf   = Split-Path $Path -Leaf
        if (-not (Test-Path "Registry::$parent\$leaf")) {
            New-Item -Path "Registry::$parent" -Name $leaf -Force | Out-Null
        }
        Set-ItemProperty -Path "Registry::$Path" -Name $Name -Type $Type -Value $Value -Force
        Write-Ok "$Path\$Name = $Value"
    } catch { Write-Fail "Registry $Path\$Name : $_" }
}

function Del-Reg {
    param([string]$Path, [string]$Name)
    if ($DryRun) { Write-Skip "Would delete '$Path\$Name'"; return }
    try {
        Remove-ItemProperty -Path "Registry::$Path" -Name $Name -Force -ErrorAction SilentlyContinue
        Write-Ok "Deleted $Path\$Name"
    } catch { Write-Fail "Delete $Path\$Name : $_" }
}

function Del-RegKey {
    param([string]$Path)
    if ($DryRun) { Write-Skip "Would delete key '$Path'"; return }
    try {
        Remove-Item -Path "Registry::$Path" -Recurse -Force -ErrorAction SilentlyContinue
        Write-Ok "Deleted key $Path"
    } catch { Write-Fail "Delete key $Path : $_" }
}

# ---------- OS check ----------
$os = (Get-CimInstance Win32_OperatingSystem).Version
$isWin11 = $os -ge "10.0.22000"
$isWin10 = $os -ge "10.0.10240"
Write-Host "Detected OS: Windows $($isWin11 ? '11' : ($isWin10 ? '10' : 'Unknown'))" -ForegroundColor Magenta

if ($Rollback) {
    # =============================================
    # ROLLBACK — revert all changes
    # =============================================
    Write-Header "ROLLBACK: Restoring default Windows settings"

    # --- Services ---
    Write-Header "Services"
    Invoke-ServiceOp -Name DiagTrack          -Action Enable-Demand
    Invoke-ServiceOp -Name dmwappushservice   -Action Enable-Demand
    Invoke-ServiceOp -Name DcpSvc             -Action Enable-Demand
    Invoke-ServiceOp -Name DiagTrack          -Action Start
    Invoke-ServiceOp -Name SysMain            -Action Enable-Auto
    Invoke-ServiceOp -Name SysMain            -Action Start
    Invoke-ServiceOp -Name WSearch             -Action Enable-Delayed
    Invoke-ServiceOp -Name WSearch             -Action Start
    Invoke-ServiceOp -Name wuauserv           -Action Enable-Demand
    Invoke-ServiceOp -Name UsoSvc             -Action Enable-Demand
    Invoke-ServiceOp -Name WaaSMedicSvc       -Action Enable-Demand
    Invoke-ServiceOp -Name wuauserv           -Action Start
    Invoke-ServiceOp -Name Spooler            -Action Enable-Auto
    Invoke-ServiceOp -Name Spooler            -Action Start
    Invoke-ServiceOp -Name seclogon           -Action Enable-Demand

    # --- Registry ---
    Write-Header "Registry changes"
    Set-Reg "HKLM\SOFTWARE\Policies\Microsoft\SQMClient\Windows"           CEIPEnable            DWord 1
    Del-Reg "HKLM\SOFTWARE\Policies\Microsoft\Windows\AppCompat"           AITEnable
    Del-Reg "HKLM\SOFTWARE\Policies\Microsoft\Windows\AppCompat"           DisableInventory
    Del-Reg "HKLM\SOFTWARE\Policies\Microsoft\Windows\AppCompat"           DisablePCA
    Set-Reg "HKLM\SYSTEM\CurrentControlSet\Control\WMI\AutoLogger\SQMLogger" Start DWord 1
    Del-Reg "HKLM\SOFTWARE\Policies\Microsoft\Windows\DataCollection"      AllowTelemetry

    Set-Reg "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management\PrefetchParameters" EnableSuperfetch DWord 1
    Set-Reg "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management\PrefetchParameters" EnablePrefetcher  DWord 1
    Del-Reg "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management\PrefetchParameters" SfTracingState

    Del-RegKey "HKCU\Software\Policies\Microsoft\Windows\WindowsCopilot"
    Del-RegKey "HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsCopilot"
    Set-Reg "HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced" ShowCopilotButton DWord 1

    Set-Reg "HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search" AllowCortana DWord 1
    Del-Reg "HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search" DisableWebSearch
    Del-Reg "HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search" ConnectedSearchUseWeb
    Set-Reg "HKCU\Software\Microsoft\Windows\CurrentVersion\Search"   CortanaConsent  DWord 1
    Set-Reg "HKCU\Software\Microsoft\Windows\CurrentVersion\Search"   BingSearchEnabled DWord 1

    Del-RegKey "HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU"
    Del-Reg "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\DeliveryOptimization\Config" DODownloadMode

    Del-Reg "HKLM\SOFTWARE\Policies\Microsoft\Windows\OneDrive" DisableFileSyncNGSC

    Set-Reg "HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile" NetworkThrottlingIndex DWord 10
    Del-RegKey "HKLM\SOFTWARE\Policies\Microsoft\Windows\Psched"
    Set-Reg "HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize" EnableTransparency DWord 1
    Set-Reg "HKCU\Control Panel\Desktop" AutoEndTasks        String "0"
    Set-Reg "HKCU\Control Panel\Desktop" HungAppTimeout      String "5000"
    Set-Reg "HKCU\Control Panel\Desktop" WaitToKillAppTimeout String "20000"
    Set-Reg "HKLM\System\CurrentControlSet\Control\Remote Assistance" fAllowToGetHelp DWord 1
    Set-Reg "HKLM\SYSTEM\CurrentControlSet\Control" WaitToKillServiceTimeout String "200000"

    Set-Reg "HKCU\Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo" Enabled DWord 1
    Set-Reg "HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager" DisableWindowsSpotlightFeatures              DWord 0
    Set-Reg "HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager" DisableTailoredExperiencesWithDiagnosticData DWord 0
    Del-Reg "HKLM\SOFTWARE\Policies\Microsoft\Windows\AdvertisingInfo" DisabledByGroupPolicy
    Del-Reg "HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Error Reporting" Disabled
    Del-Reg "HKLM\SOFTWARE\Policies\Microsoft\PCHealth\ErrorReporting" DoReport
    Del-RegKey "HKLM\SOFTWARE\Policies\Microsoft\Windows\GameDVR"

    if ($isWin11) {
        Set-Reg "HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced" TaskbarAl DWord 1
        Set-Reg "HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced" TaskbarMn DWord 1
        Set-Reg "HKCU\Control Panel\Desktop" DockMoving String "1"
        Del-Reg "HKCU\Software\Microsoft\Windows\CurrentVersion\Policies\Explorer" HideSCAMeetNow
    }

    Set-Reg "HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR"       GameDVR_Enabled DWord 1
    Set-Reg "HKCU\Software\Microsoft\Windows\CurrentVersion\GameConfigStore" GameDVR_Enabled DWord 1

} else {
    # =============================================
    # APPLY — gaming optimizations
    # =============================================
    Write-Header "APPLY: Gaming Mode Optimizations"

    # --- 1. Telemetry ---
    Write-Header "1. Disable Telemetry"
    Invoke-ServiceOp -Name DiagTrack        -Action Stop
    Invoke-ServiceOp -Name dmwappushservice -Action Stop
    Invoke-ServiceOp -Name DcpSvc           -Action Stop
    Invoke-ServiceOp -Name DiagTrack        -Action Disable
    Invoke-ServiceOp -Name dmwappushservice -Action Disable
    Invoke-ServiceOp -Name DcpSvc           -Action Disable

    Set-Reg "HKLM\SOFTWARE\Policies\Microsoft\SQMClient\Windows"                               CEIPEnable       DWord 0
    Set-Reg "HKLM\SOFTWARE\Policies\Microsoft\Windows\AppCompat"                                AITEnable        DWord 0
    Set-Reg "HKLM\SOFTWARE\Policies\Microsoft\Windows\AppCompat"                                DisableInventory  DWord 1
    Set-Reg "HKLM\SOFTWARE\Policies\Microsoft\Windows\AppCompat"                                DisablePCA       DWord 1
    Set-Reg "HKLM\SYSTEM\CurrentControlSet\Control\WMI\AutoLogger\SQMLogger"                  Start            DWord 0
    Set-Reg "HKLM\SOFTWARE\Policies\Microsoft\Windows\DataCollection"                           AllowTelemetry   DWord 0

    # --- 2. Superfetch/SysMain ---
    Write-Header "2. Disable Superfetch/SysMain"
    Invoke-ServiceOp -Name SysMain -Action Stop
    Invoke-ServiceOp -Name SysMain -Action Disable
    Set-Reg "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management\PrefetchParameters" EnableSuperfetch DWord 0
    Set-Reg "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management\PrefetchParameters" EnablePrefetcher  DWord 0
    Set-Reg "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management\PrefetchParameters" SfTracingState   DWord 1

    # --- 3. Windows Search ---
    Write-Header "3. Disable Windows Search"
    Invoke-ServiceOp -Name WSearch -Action Stop
    Invoke-ServiceOp -Name WSearch -Action Disable

    # --- 4. Copilot / Cortana / AI ---
    Write-Header "4. Disable AI Services"
    Set-Reg "HKCU\Software\Policies\Microsoft\Windows\WindowsCopilot"                           TurnOffWindowsCopilot DWord 1
    Set-Reg "HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsCopilot"                           TurnOffWindowsCopilot DWord 1
    Set-Reg "HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced"                   ShowCopilotButton    DWord 0

    Set-Reg "HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search" AllowCortana          DWord 0
    Set-Reg "HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search" DisableWebSearch      DWord 1
    Set-Reg "HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Search" ConnectedSearchUseWeb DWord 0
    Set-Reg "HKCU\Software\Microsoft\Windows\CurrentVersion\Search"   CortanaConsent        DWord 0
    Set-Reg "HKCU\Software\Microsoft\Windows\CurrentVersion\Search"   BingSearchEnabled     DWord 0

    # --- 5. Windows Updates ---
    Write-Header "5. Disable Windows Updates"
    Invoke-ServiceOp -Name wuauserv     -Action Stop
    Invoke-ServiceOp -Name UsoSvc       -Action Stop
    Invoke-ServiceOp -Name WaaSMedicSvc -Action Stop
    Invoke-ServiceOp -Name wuauserv     -Action Disable
    Invoke-ServiceOp -Name UsoSvc       -Action Disable
    Invoke-ServiceOp -Name WaaSMedicSvc -Action Disable

    Set-Reg "HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU" NoAutoUpdate DWord 1
    Set-Reg "HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate\AU" AUOptions    DWord 1
    Set-Reg "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\DeliveryOptimization\Config" DODownloadMode DWord 0

    # --- 6. OneDrive ---
    Write-Header "6. Disable OneDrive"
    Set-Reg "HKLM\SOFTWARE\Policies\Microsoft\Windows\OneDrive" DisableFileSyncNGSC DWord 1

    # --- 7. Performance Tweaks ---
    Write-Header "7. Performance Tweaks"
    Set-Reg "HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile" NetworkThrottlingIndex DWord 4294967295
    Set-Reg "HKLM\SOFTWARE\Policies\Microsoft\Windows\Psched"                            NonBestEffortLimit    DWord 0
    Set-Reg "HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize"           EnableTransparency    DWord 0
    Set-Reg "HKCU\Control Panel\Desktop" AutoEndTasks        String "1"
    Set-Reg "HKCU\Control Panel\Desktop" HungAppTimeout      String "1000"
    Set-Reg "HKCU\Control Panel\Desktop" WaitToKillAppTimeout String "2000"
    Set-Reg "HKLM\System\CurrentControlSet\Control\Remote Assistance" fAllowToGetHelp DWord 0
    Set-Reg "HKLM\SYSTEM\CurrentControlSet\Control" WaitToKillServiceTimeout String "2000"

    # --- 8. Privacy ---
    Write-Header "8. Privacy Hardening"
    Set-Reg "HKCU\Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo" Enabled DWord 0
    Set-Reg "HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager" DisableWindowsSpotlightFeatures              DWord 1
    Set-Reg "HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager" DisableTailoredExperiencesWithDiagnosticData DWord 1
    Set-Reg "HKLM\SOFTWARE\Policies\Microsoft\Windows\AdvertisingInfo" DisabledByGroupPolicy DWord 1
    Set-Reg "HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Error Reporting" Disabled DWord 1
    Set-Reg "HKLM\SOFTWARE\Policies\Microsoft\PCHealth\ErrorReporting" DoReport DWord 0

    # --- 9. Windows 11 UI ---
    if ($isWin11) {
        Write-Header "9. Simplify Windows 11 UI"
        Set-Reg "HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced" TaskbarAl DWord 0
        Set-Reg "HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced" TaskbarMn DWord 0
        Set-Reg "HKCU\Control Panel\Desktop" DockMoving String "0"
        Set-Reg "HKCU\Software\Microsoft\Windows\CurrentVersion\Policies\Explorer" HideSCAMeetNow DWord 1
    }

    # --- 10. GameDVR ---
    Write-Header "10. Disable GameDVR"
    Set-Reg "HKLM\SOFTWARE\Policies\Microsoft\Windows\GameDVR" AllowGameDVR DWord 0
    Set-Reg "HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR" GameDVR_Enabled DWord 0
    Set-Reg "HKCU\Software\Microsoft\Windows\CurrentVersion\GameConfigStore" GameDVR_Enabled DWord 0

    # --- 11. Print Spooler ---
    Write-Header "11. Disable Print Spooler"
    Invoke-ServiceOp -Name Spooler -Action Stop
    Invoke-ServiceOp -Name Spooler -Action Disable

    # --- 12. Secondary Logon ---
    Write-Header "12. Disable Secondary Logon"
    Invoke-ServiceOp -Name seclogon -Action Stop
    Invoke-ServiceOp -Name seclogon -Action Disable
}

# ---------- done ----------
Write-Host ""
if ($DryRun) {
    Write-Host "Dry-run complete. No changes were made." -ForegroundColor Yellow
} elseif ($Rollback) {
    Write-Host "Rollback complete. Restart recommended." -ForegroundColor Green
} else {
    Write-Host "Gaming Mode applied. Restart recommended." -ForegroundColor Green
}
