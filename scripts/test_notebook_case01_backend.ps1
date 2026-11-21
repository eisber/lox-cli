# Synthetic only: extract definitions without running the native dispatcher.
$ErrorActionPreference = 'Stop'
$errors = $null
$tokens = $null
$ast = [Management.Automation.Language.Parser]::ParseFile(
    (Join-Path $PSScriptRoot 'notebook_case01.ps1'), [ref]$tokens, [ref]$errors)
if ($errors.Count) { throw 'Case01 payload does not parse' }
foreach ($name in @('Deny-Case01', 'Assert-Case01ControlOwner', 'Get-Case01Fence', 'Initialize-Case01Native')) {
    $definition = $ast.Find({
        param($node)
        $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq $name
    }, $true)
    if (!$definition) { throw "Missing definition $name" }
    . ([ScriptBlock]::Create($definition.Extent.Text))
}

# Compiles public API declarations only. No declared Windows API is invoked.
Initialize-Case01Native

$script:protocol = 'Any'
$script:remotePort = 'Any'
$script:cloakedProfile = $false
$script:allow = $false
function Get-NetFirewallProfile {
    param($PolicyStore)
    if ($PolicyStore -cne 'ActiveStore') { throw 'Not effective profile policy' }
    foreach ($name in @('Domain', 'Private', 'Public')) {
        [pscustomobject]@{Name = $name; Enabled = !$script:cloakedProfile}
    }
}
function Get-NetFirewallRule {
    param($PolicyStore)
    if ($PolicyStore -cne 'ActiveStore') { throw 'Not effective rule policy' }
    [pscustomobject]@{
        Name = 'synthetic-block'; Enabled = $true; Direction = 'Outbound'; Action = 'Block'
        Profile = 'Any'; PrimaryStatus = 'OK'; EnforcementStatus = 'Full'; OverrideBlockRules = $false
    }
    if ($script:allow) {
        [pscustomobject]@{
            Name = 'synthetic-allow'; Enabled = $true; Direction = 'Outbound'; Action = 'Allow'
            Profile = 'Any'; PrimaryStatus = 'OK'; EnforcementStatus = 'Full'; OverrideBlockRules = $true
        }
    }
}
function Get-NetFirewallApplicationFilter { process { [pscustomobject]@{Program = 'synthetic.exe'} } }
function Get-NetFirewallAddressFilter { process { [pscustomobject]@{LocalAddress = 'Any'; RemoteAddress = 'Any'} } }
function Get-NetFirewallPortFilter {
    process { [pscustomobject]@{Protocol = $script:protocol; LocalPort = 'Any'; RemotePort = $script:remotePort} }
}
function Get-NetFirewallServiceFilter { process { [pscustomobject]@{Service = 'Any'} } }
function Get-NetFirewallInterfaceFilter { process { [pscustomobject]@{InterfaceAlias = 'Any'} } }
function Get-NetFirewallInterfaceTypeFilter { process { [pscustomobject]@{InterfaceType = 'Any'} } }
function Get-NetFirewallSecurityFilter {
    process { [pscustomobject]@{Authentication = 'NotRequired'; LocalUser = 'Any'; RemoteUser = 'Any'; RemoteMachine = 'Any'} }
}
function Get-NetIPsecRule { param($PolicyStore) }

function Expect-Refusal([scriptblock]$Action, [string]$Code) {
    try { & $Action | Out-Null } catch {
        if ($_.Exception.Message -cne "GuardSourceExact:$Code") { throw }
        return
    }
    throw "Synthetic guard unexpectedly passed: $Code"
}

Expect-Refusal { Get-Case01Fence 'synthetic.exe' } 'EFFECTIVE_WFP_BYPASS_PROOF_API_UNIMPLEMENTED'
$script:remotePort = '443'
Expect-Refusal { Get-Case01Fence 'synthetic.exe' } 'EFFECTIVE_FENCE_MISMATCH_OR_ALLOW_BYPASS'
$script:remotePort = 'Any'
$script:allow = $true
Expect-Refusal { Get-Case01Fence 'synthetic.exe' } 'EFFECTIVE_FENCE_MISMATCH_OR_ALLOW_BYPASS'
$script:allow = $false
$script:cloakedProfile = $true
Expect-Refusal { Get-Case01Fence 'synthetic.exe' } 'FIREWALL_PROFILE_DISABLED_OR_UNKNOWN'

# Fake source-on-disk/ancestry metadata must NOT become at-effect control proof.
$syntheticStart = [DateTime]::SpecifyKind([DateTime]'2000-01-01', [DateTimeKind]::Utc)
function Get-Process { param($Id) [pscustomobject]@{StartTime = $syntheticStart; Path = 'synthetic-bridge.exe'} }
function Get-Case01Sha { param($Path) return ('a' * 64) }
$Case01Request = [pscustomobject]@{
    control_observed_unix = ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds() / 1000.0)
    control_generation = 7
    bridge = [pscustomobject]@{
        pid = $PID; start_utc = $syntheticStart.ToString('o')
        exe_sha256 = ('a' * 64); guard_path = 'synthetic-guard.py'; guard_sha256 = ('a' * 64)
    }
}
Expect-Refusal { Assert-Case01ControlOwner } 'CURRENT_GENERATION_AT_EFFECT_API_UNAVAILABLE'
'5 synthetic backend refusals; public API declarations compiled; native API calls=0'
