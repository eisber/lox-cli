# Private, case-local payload. No old helper is sourced or invoked.
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

function Deny-Case01([string]$Code) { throw "GuardSourceExact:$Code" }

function Assert-Case01ControlOwner {
    # ControlStatus is independently sampled by the dedicated bounded caller.
    # The current bridge must itself gate this PowerShell dispatch. Its exact
    # enforcing source/executable/ancestry require NEW independent admission.
    $bridge = $Case01Request.bridge
    if (!$bridge -or !$bridge.guard_path -or !$bridge.guard_sha256 -or
        $null -eq $Case01Request.control_generation) { Deny-Case01 'SUPPORTED_CONTROL_API_UNAVAILABLE' }
    $now = ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds() / 1000.0)
    if ($now -lt $Case01Request.control_observed_unix -or
        $now - $Case01Request.control_observed_unix -gt 2) { Deny-Case01 'CONTROL_OBSERVATION_STALE' }
    $process = Get-Process -Id $bridge.pid -ErrorAction Stop
    if ($process.StartTime.ToUniversalTime().ToString('o') -cne $bridge.start_utc -or
        (Get-Case01Sha $process.Path) -cne $bridge.exe_sha256 -or
        (Get-Case01Sha $bridge.guard_path) -cne $bridge.guard_sha256) {
        Deny-Case01 'SUPPORTED_BRIDGE_RUNTIME_SOURCE_MISMATCH'
    }
    $ancestor = $PID
    $seen = @{}
    while ($ancestor -ne $bridge.pid) {
        if ($ancestor -le 0 -or $seen.ContainsKey($ancestor) -or $seen.Count -gt 32) {
            Deny-Case01 'SUPPORTED_BRIDGE_ANCESTRY_UNAVAILABLE'
        }
        $seen[$ancestor] = $true
        $record = Get-CimInstance Win32_Process -Filter "ProcessId=$ancestor" -ErrorAction Stop
        if (!$record) { Deny-Case01 'SUPPORTED_BRIDGE_ANCESTRY_UNAVAILABLE' }
        $ancestor = [int]$record.ParentProcessId
    }
    # Adjacent ControlStatus + source-on-disk/ancestry do not establish which
    # generation/exclusive owner the bridge actually enforced at this effect.
    # The reviewed sources expose no supported atomic receipt for that fact.
    Deny-Case01 'CURRENT_GENERATION_AT_EFFECT_API_UNAVAILABLE'
}

function Initialize-Case01Native {
    if ('Case01Native' -as [type]) { return }
    Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class Case01Native {
    [StructLayout(LayoutKind.Sequential)] public struct Point {public int X,Y;}
    [StructLayout(LayoutKind.Sequential)] public struct Rect {public int L,T,R,B;}
    [StructLayout(LayoutKind.Sequential)] public struct Mouse {
        public int dx,dy; public uint data,flags,time; public UIntPtr extra;
    }
    [StructLayout(LayoutKind.Explicit)] public struct Union {
        [FieldOffset(0)] public Mouse mouse;
    }
    [StructLayout(LayoutKind.Sequential)] public struct Input {public uint type;public Union value;}
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern IntPtr GetAncestor(IntPtr h,uint flags);
    [DllImport("user32.dll")] public static extern IntPtr WindowFromPhysicalPoint(Point p);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h,out uint pid);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll")] public static extern bool GetCursorPos(out Point p);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x,int y);
    [DllImport("user32.dll")] public static extern IntPtr GetThreadDpiAwarenessContext();
    [DllImport("user32.dll")] public static extern int GetAwarenessFromDpiAwarenessContext(IntPtr context);
    [DllImport("user32.dll")] static extern IntPtr GetWindow(IntPtr h,uint command);
    [DllImport("user32.dll",CharSet=CharSet.Unicode)] static extern int GetClassName(IntPtr h,StringBuilder s,int n);
    [DllImport("user32.dll",SetLastError=true)] public static extern uint SendInput(uint count,Input[] inputs,int size);
    [DllImport("kernel32.dll")] public static extern uint WTSGetActiveConsoleSessionId();
    [DllImport("user32.dll")] static extern IntPtr OpenInputDesktop(uint f,bool inherit,uint access);
    [DllImport("user32.dll")] static extern bool CloseDesktop(IntPtr h);
    [DllImport("user32.dll",CharSet=CharSet.Unicode)] static extern bool GetUserObjectInformation(
        IntPtr h,int index,StringBuilder text,uint size,out uint needed);
    [DllImport("dwmapi.dll",EntryPoint="DwmGetWindowAttribute")]
        public static extern int Cloak(IntPtr h,int a,out int value,int size);
    [DllImport("dwmapi.dll",EntryPoint="DwmGetWindowAttribute")]
        public static extern int Frame(IntPtr h,int a,out Rect value,int size);
    public static uint Owner(IntPtr h) {uint p;GetWindowThreadProcessId(h,out p);return p;}
    public static string Desktop() {
        IntPtr h=OpenInputDesktop(0,false,1);
        if(h==IntPtr.Zero)return "<unavailable>";
        try {uint n;var s=new StringBuilder(256);
            return GetUserObjectInformation(h,2,s,512,out n)?s.ToString():"<unavailable>";
        } finally {CloseDesktop(h);}
    }
    public static uint CompleteClick() {
        var inputs=new Input[2];
        inputs[0].value.mouse.flags=2;
        inputs[1].value.mouse.flags=4;
        return SendInput(2,inputs,Marshal.SizeOf(typeof(Input)));
    }
    public static string ClassName(IntPtr h) {
        var s=new StringBuilder(256);GetClassName(h,s,256);return s.ToString();
    }
    public static bool Unoccluded(IntPtr root,Rect area) {
        var visited=new System.Collections.Generic.HashSet<IntPtr>();
        for(var h=GetWindow(root,3);h!=IntPtr.Zero;h=GetWindow(h,3)) {
            if(!visited.Add(h) || visited.Count>4096)return false;
            if(!IsWindowVisible(h))continue;
            int cloak;Rect other;
            if(Cloak(h,14,out cloak,4)!=0 || Frame(h,9,out other,16)!=0)return false;
            if(cloak==0 && other.L<area.R && other.R>area.L &&
                other.T<area.B && other.B>area.T)return false;
        }
        return true;
    }
}
'@
}

function Get-Case01Sha([string]$Path) {
    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Assert-Case01Deadline {
    $now = ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds() / 1000.0)
    if ($now -ge $Case01Request.deadline_unix) { Deny-Case01 'DEADLINE_EXPIRED' }
}

function Get-Case01Fence([string]$Exe) {
    # Read effective policy, not named persistent rules. Conservatively reject
    # every effective allow/bypass that might apply, including broad app rules.
    $profiles = @(Get-NetFirewallProfile -PolicyStore ActiveStore -ErrorAction Stop)
    if ($profiles.Count -ne 3 -or @($profiles | Where-Object { "$($_.Enabled)" -ne 'True' }).Count) {
        Deny-Case01 'FIREWALL_PROFILE_DISABLED_OR_UNKNOWN'
    }
    $rules = @(Get-NetFirewallRule -PolicyStore ActiveStore -ErrorAction Stop |
        Where-Object { "$($_.Enabled)" -eq 'True' -and "$($_.Direction)" -eq 'Outbound' })
    $blocks = @()
    $competing = @()
    foreach ($rule in $rules) {
        $apps = @($rule | Get-NetFirewallApplicationFilter -ErrorAction Stop)
        if ($apps.Count -ne 1) { Deny-Case01 'FIREWALL_APP_FILTER_AMBIGUOUS' }
        $program = [Environment]::ExpandEnvironmentVariables([string]$apps[0].Program)
        $applies = $program -ieq $Exe -or $program -in @('Any', '*', 'System')
        # Packaged-app and service/WFP paths are not silently treated as excluded.
        if (!$applies -and ($program.Contains('*') -or !$program)) {
            Deny-Case01 'FIREWALL_PROGRAM_SCOPE_UNSUPPORTED'
        }
        if (!$applies) { continue }
        if ("$($rule.Action)" -ne 'Block' -or "$($rule.OverrideBlockRules)" -eq 'True') {
            $competing += [string]$rule.Name
            continue
        }
        if ($program -ine $Exe) { continue }
        $address = @($rule | Get-NetFirewallAddressFilter -ErrorAction Stop)
        $port = @($rule | Get-NetFirewallPortFilter -ErrorAction Stop)
        $service = @($rule | Get-NetFirewallServiceFilter -ErrorAction Stop)
        $interface = @($rule | Get-NetFirewallInterfaceFilter -ErrorAction Stop)
        $interfaceType = @($rule | Get-NetFirewallInterfaceTypeFilter -ErrorAction Stop)
        $security = @($rule | Get-NetFirewallSecurityFilter -ErrorAction Stop)
        if ($address.Count -ne 1 -or $port.Count -ne 1 -or $service.Count -ne 1 -or
            $interface.Count -ne 1 -or $interfaceType.Count -ne 1 -or $security.Count -ne 1) {
            Deny-Case01 'FIREWALL_FILTER_SCHEMA_UNSUPPORTED'
        }
        $all = "$($rule.Profile)" -eq 'Any' -and "$($rule.PrimaryStatus)" -eq 'OK' -and
            "$($rule.EnforcementStatus)" -eq 'Full' -and
            "$($address[0].LocalAddress)" -eq 'Any' -and "$($address[0].RemoteAddress)" -eq 'Any' -and
            "$($port[0].Protocol)" -eq 'Any' -and "$($port[0].LocalPort)" -eq 'Any' -and
            "$($port[0].RemotePort)" -eq 'Any' -and "$($service[0].Service)" -eq 'Any' -and
            "$($interface[0].InterfaceAlias)" -eq 'Any' -and
            "$($interfaceType[0].InterfaceType)" -eq 'Any' -and
            "$($security[0].Authentication)" -eq 'NotRequired' -and
            "$($security[0].LocalUser)" -eq 'Any' -and "$($security[0].RemoteUser)" -eq 'Any' -and
            "$($security[0].RemoteMachine)" -eq 'Any'
        if ($all) {
            $blocks += @{rule = $rule; address = $address[0]; port = $port[0]}
        }
    }
    if ($blocks.Count -ne 1 -or $competing.Count) { Deny-Case01 'EFFECTIVE_FENCE_MISMATCH_OR_ALLOW_BYPASS' }
    $ipsec = @(Get-NetIPsecRule -PolicyStore ActiveStore -ErrorAction Stop |
        Where-Object { "$($_.Enabled)" -eq 'True' })
    if ($ipsec.Count) { Deny-Case01 'EFFECTIVE_IPSEC_BYPASS_SCOPE_UNSUPPORTED' }
    $chosen = $blocks[0]
    # NetSecurity ActiveStore alone does not enumerate effective third-party
    # WFP callouts/filters. Do not silently promote that partial proof.
    Deny-Case01 'EFFECTIVE_WFP_BYPASS_PROOF_API_UNIMPLEMENTED'
    return @{
        store = 'ActiveStore'; profiles_enabled = @('Domain', 'Private', 'Public')
        program = $Exe; direction = [string]$chosen.rule.Direction; action = [string]$chosen.rule.Action
        enabled = $true; profiles = [string]$chosen.rule.Profile
        protocol = [string]$chosen.port.Protocol
        local_address = [string]$chosen.address.LocalAddress; remote_address = [string]$chosen.address.RemoteAddress
        local_port = [string]$chosen.port.LocalPort; remote_port = [string]$chosen.port.RemotePort
        competing_allow_or_bypass = @($competing); policy_status = [string]$chosen.rule.PrimaryStatus
    }
}

function Get-Case01Capture($Frame) {
    Add-Type -AssemblyName System.Drawing
    $width = $Frame.R - $Frame.L
    $height = $Frame.B - $Frame.T
    if ($width -le 0 -or $height -le 0 -or [long]$width*$height -gt 12000000) {
        Deny-Case01 'CAPTURE_SIZE_UNSUPPORTED'
    }
    $bitmap = [Drawing.Bitmap]::new($width, $height)
    $graphics = [Drawing.Graphics]::FromImage($bitmap)
    $memory = [IO.MemoryStream]::new()
    try {
        $graphics.CopyFromScreen($Frame.L, $Frame.T, 0, 0, $bitmap.Size)
        $bitmap.Save($memory, [Drawing.Imaging.ImageFormat]::Png)
        $bytes = $memory.ToArray()
        if ($bytes.Length -gt 4000000) { Deny-Case01 'CAPTURE_TOO_LARGE' }
        $hash = [Security.Cryptography.SHA256]::Create()
        try {
            return @{
                screenshot_sha256 = ([BitConverter]::ToString($hash.ComputeHash($bytes))).Replace('-', '').ToLowerInvariant()
                screenshot_base64 = [Convert]::ToBase64String($bytes)
            }
        } finally { $hash.Dispose() }
    } finally { $memory.Dispose(); $graphics.Dispose(); $bitmap.Dispose() }
}

function GuardSourceExact([bool]$RequireWindow) {
    Assert-Case01Deadline
    Initialize-Case01Native
    $actor = [Security.Principal.WindowsIdentity]::GetCurrent()
    $session = [Diagnostics.Process]::GetCurrentProcess().SessionId
    $console = [Case01Native]::WTSGetActiveConsoleSessionId()
    $desktop = [Case01Native]::Desktop()
    if ($actor.User.Value -cne $Case01Request.actor_sid -or $session -eq 0 -or
        $session -ne $console -or $desktop -cne 'Default') { Deny-Case01 'ACTOR_SESSION_DESKTOP_MISMATCH' }
    $awareness = [Case01Native]::GetAwarenessFromDpiAwarenessContext(
        [Case01Native]::GetThreadDpiAwarenessContext())
    if ($awareness -ne 2) { Deny-Case01 'PHYSICAL_COORDINATE_CONTEXT_UNSUPPORTED' }
    $exe = [string]$Case01Request.application_path
    if ((Get-Case01Sha $exe) -cne '7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3') {
        Deny-Case01 'APPLICATION_HASH_MISMATCH'
    }
    if ([Diagnostics.FileVersionInfo]::GetVersionInfo($exe).FileVersion -cne '17.2.10.21') {
        Deny-Case01 'APPLICATION_VERSION_MISMATCH'
    }
    $pe = [IO.File]::OpenRead($exe)
    $reader = [IO.BinaryReader]::new($pe)
    try {
        $pe.Position = 0x3c
        $offset = $reader.ReadInt32()
        $pe.Position = $offset
        if ($reader.ReadUInt32() -ne 0x4550 -or $reader.ReadUInt16() -ne 0x8664) {
            Deny-Case01 'APPLICATION_NOT_AMD64'
        }
    } finally { $reader.Dispose(); $pe.Dispose() }
    $fixtureHash = Get-Case01Sha ([string]$Case01Request.fixture_path)
    if ($fixtureHash -cne '674905cda0c10ffff853e7090507f696747c8b0fb8f3149479b96cff924823ce') {
        Deny-Case01 'FIXTURE_CHANGED'
    }
    $fence = Get-Case01Fence $exe
    $result = @{
        observed_unix = ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds() / 1000.0)
        actor_sid = $actor.User.Value; actor_session = $session; active_console_session = $console
        desktop = $desktop; dpi_awareness = $awareness; application_sha256 = Get-Case01Sha $exe
        fixture_sha256 = $fixtureHash; fence = $fence
        control_binding = 'EXTERNAL_EXACT_BRIDGE_ADMISSION_PLUS_FRESH_SUPPORTED_TOOL_REQUIRED'
    }
    if (!$RequireWindow) { return $result }
    $owned = $Case01Request.owned
    if (!$owned) { Deny-Case01 'OWNED_IDENTITY_REQUIRED' }
    $p = Get-Process -Id $owned.pid -ErrorAction Stop
    if ($p.Path -ine $exe -or $p.SessionId -ne $session -or
        $p.StartTime.ToUniversalTime().ToString('o') -cne $owned.start_utc -or
        (Get-Case01Sha $p.Path) -cne $owned.sha256) { Deny-Case01 'OWNED_IDENTITY_MISMATCH' }
    $root = [IntPtr]$p.MainWindowHandle
    $foreground = [Case01Native]::GetForegroundWindow()
    $target = $Case01Request.target
    $point = [Case01Native+Point]::new()
    $hit = [IntPtr]::Zero
    if ($target) {
        if ($target.point.Count -ne 2) { Deny-Case01 'VISUAL_SELECTION_REQUIRED' }
        $point.X = $target.point[0]; $point.Y = $target.point[1]
        $hit = [Case01Native]::WindowFromPhysicalPoint($point)
    } elseif ($Case01Request.operation -notin @('capture', 'cleanup_probe', 'read_fixture')) {
        Deny-Case01 'VISUAL_SELECTION_REQUIRED'
    }
    $frame = [Case01Native+Rect]::new()
    $cloaked = 0
    $cloakHr = [Case01Native]::Cloak($root, 14, [ref]$cloaked, 4)
    $frameHr = [Case01Native]::Frame($root, 9, [ref]$frame, 16)
    if ($root -eq [IntPtr]::Zero -or $foreground -ne $root -or
        [Case01Native]::Owner($foreground) -ne $p.Id -or
        [Case01Native]::GetAncestor($root, 2) -ne $root -or
        [Case01Native]::ClassName($root) -cne 'Qt6102QWindowIcon') {
        Deny-Case01 'PHYSICAL_FOREGROUND_ROOT_MISMATCH'
    }
    if ($target -and ([Case01Native]::Owner($hit) -ne $p.Id -or
        [Case01Native]::GetAncestor($hit, 2) -ne $root)) { Deny-Case01 'PHYSICAL_HIT_ROOT_MISMATCH' }
    if ($cloakHr -ne 0 -or $frameHr -ne 0 -or $cloaked -ne 0 -or
        ![Case01Native]::IsWindowVisible($root)) { Deny-Case01 'DWM_UNAVAILABLE_OR_CLOAKED' }
    if ($target -and ($point.X -lt $frame.L -or $point.X -ge $frame.R -or
        $point.Y -lt $frame.T -or $point.Y -ge $frame.B)) { Deny-Case01 'POINT_OUTSIDE_DWM_FRAME' }
    if (![Case01Native]::Unoccluded($root, $frame)) { Deny-Case01 'CAPTURE_OCCLUDED_OR_UNSUPPORTED' }
    $capture = Get-Case01Capture $frame
    if (![Case01Native]::Unoccluded($root, $frame) -or
        [Case01Native]::GetForegroundWindow() -ne $root) { Deny-Case01 'CAPTURE_CHANGED_DURING_OBSERVATION' }
    if ($target -and $target.frame_sha256 -cne $capture.screenshot_sha256) { Deny-Case01 'EXPIRED_FRAME' }
    $result.owned = @{
        pid = $p.Id; start_utc = $p.StartTime.ToUniversalTime().ToString('o')
        session = $p.SessionId; path = $p.Path; sha256 = Get-Case01Sha $p.Path
    }
    $result.foreground = [long]$foreground; $result.foreground_pid = [Case01Native]::Owner($foreground)
    $result.root = [long]$root; $result.hit = [long]$hit; $result.hit_pid = [Case01Native]::Owner($hit)
    $result.hit_root = [long][Case01Native]::GetAncestor($hit, 2)
    $result.cloaked = $cloaked; $result.cloak_hresult = $cloakHr; $result.frame_hresult = $frameHr
    $result.dwm_frame = @($frame.L, $frame.T, $frame.R, $frame.B)
    $result.point = @($point.X, $point.Y); $result.visible = $true
    $result.screenshot_sha256 = $capture.screenshot_sha256
    $result.screenshot_base64 = $capture.screenshot_base64
    $result.observed_unix = ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds() / 1000.0)
    return $result
}

try {
    $operation = [string]$Case01Request.operation
    $reads = @('prelaunch', 'capture', 'read_fixture', 'cleanup_probe')
    $effects = @('launch', 'navigate', 'move', 'hover', 'glyph', 'sim_start', 'sim_stop', 'close')
    if ($operation -notin ($reads + $effects)) { Deny-Case01 'OPERATION_NOT_CASE01' }
    if ($operation -in $effects) { Assert-Case01ControlOwner }
    $observation = GuardSourceExact ($null -ne $Case01Request.owned)
    $receipt = @{ok = $true; observation = $observation; effect = 'NOT_STARTED'}
    if ($operation -eq 'read_fixture') {
        $bytes = [IO.File]::ReadAllBytes([string]$Case01Request.fixture_path)
        if ($bytes.Length -gt 4000000) { Deny-Case01 'TRANSFER_SIZE_UNSUPPORTED' }
        $receipt.fixture_base64 = [Convert]::ToBase64String($bytes)
    }
    if ($operation -in @('navigate', 'glyph', 'sim_start', 'sim_stop')) {
        $cursor = [Case01Native+Point]::new()
        if (![Case01Native]::GetCursorPos([ref]$cursor) -or
            $cursor.X -ne $observation.point[0] -or $cursor.Y -ne $observation.point[1]) {
            Deny-Case01 'CURSOR_NOT_AT_SEPARATELY_GUARDED_VISIBLE_TARGET'
        }
        Assert-Case01Deadline
        $count = [Case01Native]::CompleteClick()
        $receipt.completion = @{inserted = $count; down = ($count -ge 1); up = ($count -eq 2)}
        if ($count -ne 2) { Deny-Case01 'INCOMPLETE_CLICK_REMOTE_COMPLETION_UNKNOWN' }
    } elseif ($operation -in @('move', 'hover')) {
        Assert-Case01Deadline
        if (![Case01Native]::SetCursorPos($observation.point[0], $observation.point[1])) {
            Deny-Case01 'MOVE_COMPLETION_UNKNOWN'
        }
        $receipt.completion = @{kind = $operation; completed = $true}
    } elseif ($operation -eq 'launch') {
        Assert-Case01Deadline
        if (@(Get-Process | Where-Object { $_.Path -ieq $Case01Request.application_path }).Count) {
            Deny-Case01 'EXISTING_APPLICATION'
        }
        $p = Start-Process -FilePath $Case01Request.application_path -PassThru `
            -ArgumentList ('"' + $Case01Request.fixture_path + '"')
        $receipt.launched_identity = @{
            pid = $p.Id; start_utc = $p.StartTime.ToUniversalTime().ToString('o')
            session = $p.SessionId; path = $p.Path; sha256 = Get-Case01Sha $p.Path
        }
        $receipt.owned_window = 'UNKNOWN_REQUIRES_NEW_POSTLAUNCH_OBSERVATION'
    } elseif ($operation -eq 'close') {
        Assert-Case01Deadline
        $p = Get-Process -Id $Case01Request.owned.pid -ErrorAction Stop
        $receipt.close_requested = $p.CloseMainWindow()
        $exited = $p.WaitForExit(2000)
        $remaining = Get-Process -Id $Case01Request.owned.pid -ErrorAction SilentlyContinue
        $receipt.absence_observed = $exited -and !$remaining
        $receipt.absence_pid = $p.Id
        $receipt.absence_start_utc = $Case01Request.owned.start_utc
        $receipt.absence_observed_unix = ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds() / 1000.0)
        $receipt.fixture_sha256 = Get-Case01Sha ([string]$Case01Request.fixture_path)
    }
    $receipt.completed_utc = [DateTime]::UtcNow.ToString('o')
    $receipt | ConvertTo-Json -Depth 20 -Compress
} catch {
    # Backend refusal is explicit and terminal, not a false/empty guard result.
    @{ok = $false; error = $_.Exception.Message; effect = 'UNKNOWN_NOT_INFERRED';
      partial_receipt = $receipt;
      completed_utc = [DateTime]::UtcNow.ToString('o')} | ConvertTo-Json -Depth 5 -Compress
}
