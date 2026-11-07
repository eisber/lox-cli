[CmdletBinding(SupportsShouldProcess)]
param(
    [Parameter(Mandatory)]
    [string] $ReaRoot,
    [Parameter(Mandatory)]
    [string] $GhidraDir,
    [Parameter(Mandatory)]
    [string] $JavaHome,
    [string] $InstallDir = 'C:\Program Files (x86)\Loxone\LoxoneConfig',
    [string] $Python = 'python',
    [string] $Node = 'node',
    [ValidateRange(1, 24)]
    [int] $IntervalHours = 1,
    [string] $TaskName = 'LoxCli-LoxoneConfig-Analysis'
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$script = Join-Path $PSScriptRoot 'reverse-loxone.py'
$pythonPath = @(Get-Command $Python -CommandType Application -ErrorAction Stop)[0].Source
$nodePath = @(Get-Command $Node -CommandType Application -ErrorAction Stop)[0].Source
$resolved = @($ReaRoot, $GhidraDir, $JavaHome, $InstallDir) | ForEach-Object {
    (Resolve-Path -LiteralPath $_ -ErrorAction Stop).Path
}
$log = Join-Path $root '.reverse-engineering\scheduled.log'
$paths = @($script, $resolved[0], $resolved[1], $resolved[2], $resolved[3], $nodePath, $log)
if ($paths | Where-Object { $_.Contains('"') -or $_.EndsWith('\') }) {
    throw 'Task paths must not contain quotes or a trailing backslash.'
}
$arguments = '"{0}" --rea-root "{1}" --ghidra-dir "{2}" --java-home "{3}" --install-dir "{4}" --node "{5}" --log-file "{6}"' -f $paths
$action = New-ScheduledTaskAction -Execute $pythonPath -Argument $arguments -WorkingDirectory $root
$trigger = New-ScheduledTaskTrigger -Once -At (Get-Date).AddMinutes(5) `
    -RepetitionInterval (New-TimeSpan -Hours $IntervalHours)
$settings = New-ScheduledTaskSettingsSet -MultipleInstances IgnoreNew `
    -ExecutionTimeLimit (New-TimeSpan -Days 7) -StartWhenAvailable `
    -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries
$principal = New-ScheduledTaskPrincipal `
    -UserId ([System.Security.Principal.WindowsIdentity]::GetCurrent().Name) `
    -LogonType Interactive -RunLevel Limited
$task = New-ScheduledTask -Action $action -Trigger $trigger -Settings $settings `
    -Principal $principal -Description 'Local-only, versioned REA analysis after Loxone Config updates. Never runs the target executable or uploads results.'
if ($PSCmdlet.ShouldProcess($TaskName, 'Register hourly Loxone Config analysis')) {
    $existing = Get-ScheduledTask -TaskPath '\' | Where-Object TaskName -EQ $TaskName
    if ($existing) {
        $backup = Join-Path $root '.reverse-engineering'
        New-Item -ItemType Directory -Path $backup -Force | Out-Null
        $backupPath = Join-Path $backup ("scheduled-task-{0}.xml" -f (Get-Date -Format 'yyyyMMdd-HHmmssfff'))
        Export-ScheduledTask -TaskName $TaskName -TaskPath '\' | Set-Content -LiteralPath $backupPath -Encoding utf8
    }
    Register-ScheduledTask -TaskName $TaskName -TaskPath '\' -InputObject $task -Force | Out-Null
    Get-ScheduledTask -TaskName $TaskName -TaskPath '\' | Select-Object TaskName, State
}
