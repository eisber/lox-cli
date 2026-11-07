$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
. "$PSScriptRoot\loxone-ux.ps1"

function Assert-Test($Condition, [string]$Message) {
    if (!$Condition) { throw $Message }
}

function Assert-ErrorCode([scriptblock]$Action, [string]$Expected) {
    $actual = $null
    try { & $Action | Out-Null }
    catch {
        $exception = $_.Exception
        while ($exception.InnerException -and !$exception.Data['ux_code']) { $exception = $exception.InnerException }
        $actual = $exception.Data['ux_code']
    }
    Assert-Test ($actual -eq $Expected) "Expected $Expected, received $actual."
}

$originalWindow = ${function:Get-UxWindow}
$originalFind = ${function:Find-UxElements}
$originalRead = ${function:Read-UxElement}
$script:lookups = 0
function Get-UxWindow([string]$Class, [switch]$AllowMissing) {
    $script:lookups++
    if ($script:lookups -eq 1) {
        if (!$AllowMissing) { Stop-Ux 'window_missing' 'Dialog is not created yet.' }
        return $null
    }
    [pscustomobject]@{Current=[pscustomobject]@{Name='ready';IsEnabled=$true}}
}
function Find-UxElements($Root, $Selector) { $Root }
function Read-UxElement($Element) { @{name=$Element.Current.Name} }
$result = Invoke-UxStep ([pscustomobject]@{op='wait';window_class='Delayed';selector=[pscustomobject]@{name='ready'};timeout_ms=1000})
Assert-Test ($script:lookups -eq 2 -and $result.name -eq 'ready') 'Wait did not retry a not-yet-created window.'
$originalValue = ${function:Get-UxValue}
$script:valueReads = 0
function Get-UxValue($Element) {
    $script:valueReads++
    if ($script:valueReads -eq 1) { return 'old' }
    'expected'
}
[void](Invoke-UxStep ([pscustomobject]@{op='wait';selector=[pscustomobject]@{name='ready'};value='expected';timeout_ms=1000}))
Assert-Test ($script:valueReads -eq 2) 'Wait returned before the exact requested value was observed.'
Set-Item Function:\Get-UxValue $originalValue
Set-Item Function:\Get-UxWindow $originalWindow
Set-Item Function:\Find-UxElements $originalFind
Set-Item Function:\Read-UxElement $originalRead

$originalForeground = ${function:Assert-UxForeground}
$originalHandle = ${function:Get-UxForegroundHandle}
$originalKeys = ${function:Send-UxKeys}
$script:lookups = 0
$script:sent = 0
function Get-UxWindow([string]$Class, [switch]$AllowMissing) {
    $script:lookups++
    [pscustomobject]@{Current=[pscustomobject]@{NativeWindowHandle=1111}}
}
function Assert-UxForeground {}
function Get-UxForegroundHandle { [IntPtr]2222 }
function Send-UxKeys([string]$Value) { $script:sent++ }
Assert-ErrorCode { Invoke-UxStep ([pscustomobject]@{op='keys';window_class='ExpectedDialog';value='{ENTER}'}) } 'foreground_changed'
Assert-Test ($script:lookups -eq 1 -and $script:sent -eq 0) 'Keys ignored the requested window class.'
Set-Item Function:\Get-UxWindow $originalWindow
Set-Item Function:\Assert-UxForeground $originalForeground
Set-Item Function:\Get-UxForegroundHandle $originalHandle
Set-Item Function:\Send-UxKeys $originalKeys

function New-TestElement([int]$Identity) {
    $element = [pscustomobject]@{Current=[pscustomobject]@{ProcessId=17};Identity=$Identity}
    $element | Add-Member -MemberType ScriptMethod -Name GetRuntimeId -Value { @($this.Identity) }
    $element
}
$originalHit = ${function:Get-UxHitElement}
$originalParent = ${function:Get-UxParent}
$script:UxProcessId = 17
$script:target = New-TestElement 1
$script:hit = New-TestElement 2
function Get-UxHitElement([int]$X, [int]$Y) { $script:hit }
function Get-UxParent($Element) { $null }
Assert-ErrorCode { Assert-UxHit $script:target 10 10 } 'occluded'
$script:hit = $script:target
Assert-UxHit $script:target 10 10
$script:hit = New-TestElement 2
function Get-UxParent($Element) { $script:target }
Assert-UxHit $script:target 10 10
Set-Item Function:\Get-UxHitElement $originalHit
Set-Item Function:\Get-UxParent $originalParent

Write-Output 'PASS: delayed window wait; window-bound keys; same-process occlusion.'
