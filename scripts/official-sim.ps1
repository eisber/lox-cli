$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

function Write-OracleJson($Path, $Value) {
    if (Test-Path -LiteralPath $Path) { throw 'Immutable artifact already exists.' }
    [IO.File]::WriteAllText($Path, ($Value | ConvertTo-Json -Depth 35 -Compress), [Text.UTF8Encoding]::new($false))
}

function Write-OracleEvent([string]$Event) {
    [IO.File]::AppendAllText((Join-Path $script:OracleRoot 'events.jsonl'),
        ((@{event=$Event;utc=[DateTime]::UtcNow.ToString('o')} | ConvertTo-Json -Compress)+"`n"),
        [Text.UTF8Encoding]::new($false))
}

function Get-OracleSamplingPlan($Request) {
    $phases = @($Request.sampling_phases)
    if ($null -eq $Request.sampling_phases) { $phases = @(@{name='after_first_calculation';at_seconds=$Request.wait_seconds}) }
    if ($phases.Count -lt 1 -or $phases.Count -gt 6) { throw 'Native sampling phase count is not bounded.' }
    $names = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $previousEnd = 0
    foreach ($phase in $phases) {
        if ($phase.name -cnotmatch '^[a-z][a-z0-9_]{0,63}$' -or !$names.Add([string]$phase.name) -or
            ($phase.at_seconds -isnot [int] -and $phase.at_seconds -isnot [long]) -or
            $phase.at_seconds -lt 20 -or $phase.at_seconds -gt 200 -or $phase.at_seconds -lt $previousEnd) {
            throw 'Native sampling phase name or time is outside the bounded protocol.'
        }
        $previousEnd = $phase.at_seconds
        if ($null -ne $phase.latest_seconds) {
            if (($phase.latest_seconds -isnot [int] -and $phase.latest_seconds -isnot [long]) -or
                $phase.latest_seconds -le $phase.at_seconds -or $phase.latest_seconds -gt 210 -or
                $phase.latest_seconds -gt $phase.at_seconds+30) {
                throw 'Native sampling window exceeds its finite end budget.'
            }
            $previousEnd = $phase.latest_seconds
        }
    }
    return ,$phases
}

function Get-OracleMainFromNative($Windows) {
    $found = @()
    foreach ($native in $Windows) {
        # Qt's Win32 class is not the QObject class returned by UI Automation.
        if ($native.ClassName -eq 'CLxMainWindow' -or $native.ClassName -match '^Qt[0-9]+QWindowIcon') {
            $element = [Windows.Automation.AutomationElement]::FromHandle([IntPtr]$native.Handle)
            if ($element.Current.ProcessId -eq $script:UxProcessId -and $element.Current.ClassName -eq 'CLxMainWindow') {
                $found += $element
            }
        }
    }
    if ($found.Count -gt 1) { throw 'Owned native main window is ambiguous.' }
    if ($found.Count -eq 1) { return $found[0] }
    return $null
}

function Initialize-OracleNative {
    if ('OracleNative' -as [type]) { return }
    Add-Type @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
public static class OracleNative {
    public static WindowInfo[] VerifiedNoncomposited = new WindowInfo[0];
    public static WindowInfo[] CheckedForeignWindows = new WindowInfo[0];
    public delegate bool Callback(IntPtr handle,IntPtr data);
    [StructLayout(LayoutKind.Sequential)] public struct Rect {public int Left,Top,Right,Bottom;}
    public class WindowInfo {
        public long Handle;public uint ProcessId;public string ClassName;public int[] Rectangle;
        public int Cloaked;public long ExtendedStyle;public int LayeredAlpha=-1;public uint LayeredFlags;
        public bool IntersectsCapture;
    }
    [DllImport("user32.dll")] static extern bool EnumWindows(Callback callback,IntPtr data);
    [DllImport("user32.dll")] static extern IntPtr GetWindow(IntPtr handle,uint command);
    [DllImport("user32.dll")] static extern bool IsWindow(IntPtr handle);
    [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr handle);
    [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr handle,out uint pid);
    [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr handle,out Rect rect);
    [DllImport("user32.dll",CharSet=CharSet.Unicode)] static extern int GetClassName(IntPtr handle,StringBuilder text,int count);
    [DllImport("user32.dll",EntryPoint="GetWindowLongPtrW")] static extern IntPtr GetWindowLongPtr(IntPtr handle,int index);
    [DllImport("user32.dll")] static extern bool GetLayeredWindowAttributes(IntPtr handle,out uint key,out byte alpha,out uint flags);
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr handle);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr handle,uint message,IntPtr wp,IntPtr lp);
    [DllImport("user32.dll")] public static extern IntPtr GetAncestor(IntPtr handle,uint flags);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("kernel32.dll")] public static extern uint WTSGetActiveConsoleSessionId();
    [DllImport("kernel32.dll",SetLastError=true)] static extern IntPtr OpenProcess(uint access,bool inherit,uint pid);
    [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr handle);
    [DllImport("advapi32.dll",SetLastError=true)] static extern bool OpenProcessToken(IntPtr process,uint access,out IntPtr token);
    [DllImport("advapi32.dll",SetLastError=true)] static extern bool GetTokenInformation(IntPtr token,int info,IntPtr data,uint size,out uint required);
    [DllImport("advapi32.dll")] static extern IntPtr GetSidSubAuthorityCount(IntPtr sid);
    [DllImport("advapi32.dll")] static extern IntPtr GetSidSubAuthority(IntPtr sid,uint index);
    [DllImport("user32.dll",SetLastError=true)] static extern IntPtr OpenInputDesktop(uint flags,bool inherit,uint access);
    [DllImport("user32.dll",CharSet=CharSet.Unicode)] static extern bool GetUserObjectInformation(IntPtr handle,int index,StringBuilder text,uint size,out uint needed);
    [DllImport("user32.dll")] static extern bool CloseDesktop(IntPtr handle);
    public static uint Owner(IntPtr handle){
        if(handle==IntPtr.Zero)return 0;
        uint pid=0;GetWindowThreadProcessId(handle,out pid);return pid;
    }
    public static string InputDesktop() {
        IntPtr desktop=OpenInputDesktop(0,false,1);
        if(desktop==IntPtr.Zero)return "<unavailable>";
        try {
            var name=new StringBuilder(256);uint needed;
            return GetUserObjectInformation(desktop,2,name,512,out needed)?name.ToString():"<unavailable>";
        } finally {CloseDesktop(desktop);}
    }
    public static int Integrity(uint pid) {
        IntPtr process=OpenProcess(0x1000,false,pid);
        if(process==IntPtr.Zero)return -1;
        IntPtr token=IntPtr.Zero,data=IntPtr.Zero;
        try {
            if(!OpenProcessToken(process,8,out token))return -1;
            uint size;GetTokenInformation(token,25,IntPtr.Zero,0,out size);
            if(size==0 || size>65536)return -1;
            data=Marshal.AllocHGlobal((int)size);
            if(!GetTokenInformation(token,25,data,size,out size))return -1;
            IntPtr sid=Marshal.ReadIntPtr(data);
            byte count=Marshal.ReadByte(GetSidSubAuthorityCount(sid));
            if(count==0)return -1;
            return Marshal.ReadInt32(GetSidSubAuthority(sid,(uint)(count-1)));
        } finally {
            if(data!=IntPtr.Zero)Marshal.FreeHGlobal(data);
            if(token!=IntPtr.Zero)CloseHandle(token);
            CloseHandle(process);
        }
    }
    [DllImport("dwmapi.dll")] static extern int DwmGetWindowAttribute(IntPtr handle,int attribute,out int value,int size);
    [DllImport("user32.dll",CharSet=CharSet.Unicode)] static extern IntPtr SendMessageTimeout(IntPtr handle,uint message,UIntPtr size,StringBuilder text,uint flags,uint timeout,out UIntPtr result);
    public static string ReadText(IntPtr handle) {
        var text=new StringBuilder(8193);UIntPtr result;
        if(SendMessageTimeout(handle,13,(UIntPtr)8193,text,2,2000,out result)==IntPtr.Zero)return "<native text unavailable>";
        return text.ToString();
    }
    public static bool Ping(IntPtr handle) {
        UIntPtr result;
        return SendMessageTimeout(handle,0,UIntPtr.Zero,null,2,1000,out result)!=IntPtr.Zero;
    }
    public static WindowInfo Describe(IntPtr handle) {
        Rect r;
        if(!IsWindow(handle) || !GetWindowRect(handle,out r))return null;
        return Details(handle,Owner(handle),r);
    }
    static WindowInfo Details(IntPtr handle,uint owner,Rect r) {
        var name=new StringBuilder(256);GetClassName(handle,name,256);
        int cloaked=0;DwmGetWindowAttribute(handle,14,out cloaked,4);
        long style=GetWindowLongPtr(handle,-20).ToInt64();
        int opacity=-1;uint flags=0,key=0;byte alpha=0;
        if((style & 0x80000)!=0 && GetLayeredWindowAttributes(handle,out key,out alpha,out flags))opacity=alpha;
        return new WindowInfo{Handle=handle.ToInt64(),ProcessId=owner,ClassName=name.ToString(),
            Rectangle=new[]{r.Left,r.Top,r.Right,r.Bottom},Cloaked=cloaked,ExtendedStyle=style,
            LayeredAlpha=opacity,LayeredFlags=flags};
    }
    public static WindowInfo[] Windows(uint wanted) {
        var windows=new List<WindowInfo>();
        Callback callback=delegate(IntPtr handle,IntPtr data) {
            uint owner;GetWindowThreadProcessId(handle,out owner);
            if(owner==wanted && IsWindowVisible(handle)){
                Rect r;
                if(GetWindowRect(handle,out r))
                    windows.Add(Details(handle,owner,r));
            }
            return true;
        };
        EnumWindows(callback,IntPtr.Zero);
        GC.KeepAlive(callback);
        return windows.ToArray();
    }
    public static WindowInfo FirstOccluder(IntPtr selected,uint wanted,int[] area) {
        var ignored=new List<WindowInfo>();
        var checkedWindows=new List<WindowInfo>();
        VerifiedNoncomposited=new WindowInfo[0];
        CheckedForeignWindows=new WindowInfo[0];
        if(!IsWindow(selected) || Owner(selected)!=wanted)
            return new WindowInfo{ClassName="<selected-root-not-found>"};
        var visited=new HashSet<IntPtr>();
        // GW_HWNDPREV walks only windows above this exact root in Z order.
        // EnumWindows enumeration order is not used as an occlusion contract.
        for(IntPtr handle=GetWindow(selected,3);handle!=IntPtr.Zero;handle=GetWindow(handle,3)) {
            if(!visited.Add(handle) || visited.Count>4096)
                return new WindowInfo{ClassName="<unstable-window-z-order>"};
            uint owner;GetWindowThreadProcessId(handle,out owner);
            Rect r;
            if(owner!=wanted && IsWindowVisible(handle) && GetWindowRect(handle,out r) &&
               r.Right>r.Left && r.Bottom>r.Top){
                var details=Details(handle,owner,r);
                details.IntersectsCapture=r.Left<area[2] && r.Right>area[0] && r.Top<area[3] && r.Bottom>area[1];
                checkedWindows.Add(details);CheckedForeignWindows=checkedWindows.ToArray();
                if(!details.IntersectsCapture)continue;
                bool invisible=details.Cloaked!=0 ||
                    ((details.LayeredFlags & 2)!=0 && details.LayeredAlpha==0);
                if(invisible){
                    ignored.Add(details);VerifiedNoncomposited=ignored.ToArray();continue;
                }
                return details;
            }
        }
        return null;
    }
    public static bool Unoccluded(IntPtr selected,uint wanted,int[] area) {
        return FirstOccluder(selected,wanted,area)==null;
    }
}
'@
}

function Save-OracleNativeWindowScreen($Window, [string]$Path) {
    [void](Assert-OracleIdentity)
    $r = $Window.Rectangle
    $width = $r[2]-$r[0]
    $height = $r[3]-$r[1]
    if ($width -lt 1 -or $height -lt 1 -or $width*$height -gt 12000000) { throw 'Unsupported owned capture size.' }
    $occluder = [OracleNative]::FirstOccluder([IntPtr]$Window.Handle,[uint32]$script:UxProcessId,$r)
    if ($occluder) {
        $processName = if ($occluder.ProcessId) { (Get-Process -Id $occluder.ProcessId -ErrorAction SilentlyContinue).ProcessName } else { $null }
        Write-OracleJson ($Path+'.occlusion.json') @{
            captured=$false;root_handle=$Window.Handle;occluder=$occluder;process_name=$processName;rectangle=$r
            verified_noncomposited=@([OracleNative]::VerifiedNoncomposited)
            scope='Read-only first-overlap PID/class/geometry/cloaking/alpha metadata; no foreign window text or pixels.'
        }
        throw "Foreign top-level window PID $($occluder.ProcessId), class $($occluder.ClassName), overlaps the exact owned native capture."
    }
    foreach ($fx in @(0.01,0.2,0.4,0.6,0.8,0.99)) {
        foreach ($fy in @(0.01,0.2,0.4,0.6,0.8,0.99)) {
            [uint32]$owner = 0
            [void][LoxUxNative]::GetWindowThreadProcessId(
                [LoxUxNative]::WindowFromPoint([LoxUxNative+Point]::new([int]($r[0]+$fx*$width),[int]($r[1]+$fy*$height))),[ref]$owner)
            if ($owner -ne $script:UxProcessId) { throw 'A foreign window occludes the owned native capture.' }
        }
    }
    if (Test-Path -LiteralPath $Path) { throw 'Immutable capture already exists.' }
    $bitmap = [Drawing.Bitmap]::new($width,$height)
    $graphics = [Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.CopyFromScreen($r[0],$r[1],0,0,$bitmap.Size)
        $bitmap.Save($Path,[Drawing.Imaging.ImageFormat]::Png)
    } finally { $graphics.Dispose();$bitmap.Dispose() }
    return @{backend='owned-native-window-CopyFromScreen';rectangle=$r;occlusion_grid=36}
}

function Get-OracleControl([string[]]$Suffixes, [switch]$Optional) {
    $main = Get-UxWindow 'CLxMainWindow'
    $found = @($main.FindAll([Windows.Automation.TreeScope]::Descendants, [Windows.Automation.Condition]::TrueCondition) |
        Where-Object {
            $id = $_.Current.AutomationId
            @($Suffixes | Where-Object { $id.EndsWith($_, [StringComparison]::Ordinal) }).Count -gt 0
        })
    if ($Optional -and !$found.Count) { return $null }
    if ($found.Count -ne 1) { throw 'Native simulator control binding is not unique.' }
    Assert-UxOwned $found[0]
    return $found[0]
}

function Read-OracleRecoveryModal($Dialog) {
    $script:OracleModalText = [Collections.Generic.List[object]]::new()
    $modalCallback = [LoxUxNative+EnumProc]{
        param($handle,$unused)
        [uint32]$owner = 0
        [void][LoxUxNative]::GetWindowThreadProcessId($handle,[ref]$owner)
        if ($owner -eq $script:UxProcessId) {
            $class = [Text.StringBuilder]::new(256)
            [void][LoxUxNative]::GetClassName($handle,$class,256)
            $script:OracleModalText.Add(@{
                handle=$handle.ToInt64();class=$class.ToString();id=[LoxUxNative]::GetDlgCtrlID($handle)
                text=$(if($class.ToString() -cin @('Static','Button')){[OracleNative]::ReadText($handle)}else{$null})
            })
        }
        return $true
    }
    [void][LoxUxNative]::EnumChildWindows([IntPtr]$Dialog.Handle,$modalCallback,[IntPtr]::Zero)
    $process = Assert-OracleIdentity
    $fixture = Join-Path $script:OracleRoot $script:OracleRequest.fixture_filename
    return @{
        window=$Dialog;root_handle=[OracleNative]::GetAncestor([IntPtr]$Dialog.Handle,2).ToInt64()
        already_handled=[bool]$script:OracleRecoveryHandled
        identity=@{
            owned_pid=$process.Id;session=$process.SessionId;actor_session=(Get-Process -Id $PID).SessionId
            application_sha256=(Get-FileHash -LiteralPath $process.Path).Hash.ToLowerInvariant()
            fixture_sha256=(Get-FileHash -LiteralPath $fixture).Hash.ToLowerInvariant()
        }
        native_text=@($script:OracleModalText.ToArray())
    }
}

function Skip-OracleOwnedRecovery($Dialog, [string]$RequestPath) {
    if ($script:OracleRequest.recovery_policy -cne 'owned-skip' -or $script:OracleRecoveryHandled) {
        throw 'An owned startup modal is blocked by policy or repeats a previously handled recovery question.'
    }
    $record = Read-OracleRecoveryModal $Dialog
    $beforePath = Join-Path $script:OracleRoot 'startup-recovery.before-action.json'
    Write-OracleJson $beforePath $record
    $validationText = & python (Join-Path $script:OracleRoot 'official_sim_runner.py') $RequestPath --recovery-dialog $beforePath `
        2> (Join-Path $script:OracleRoot 'startup-recovery.validation.stderr.txt')
    if ($LASTEXITCODE -ne 0) { throw 'Startup modal is not the exact reviewed owned-fixture recovery question; no action was taken.' }
    $validation = $validationText | ConvertFrom-Json
    Write-OracleJson (Join-Path $script:OracleRoot 'startup-recovery.validation.json') $validation
    if (!$validation.accepted -or $validation.button_id -ne 7 -or $validation.root_handle -ne $Dialog.Handle) {
        throw 'Owned recovery validation did not authorize the exact No control.'
    }
    $element = [Windows.Automation.AutomationElement]::FromHandle([IntPtr]$Dialog.Handle)
    Assert-UxOwned $element
    if ($element.Current.ClassName -cne '#32770' -or $element.Current.NativeWindowHandle -ne $Dialog.Handle -or
        [OracleNative]::GetAncestor([IntPtr]$element.Current.NativeWindowHandle,2).ToInt64() -ne $Dialog.Handle) {
        throw 'Owned recovery dialog root identity changed before activation.'
    }
    $no = Get-UxDialogControl $element @(7) 'Button'
    [void](Get-UxDialogControl $element @(6) 'Button')
    if ([OracleNative]::GetAncestor([IntPtr]$no.Current.NativeWindowHandle,2).ToInt64() -ne $Dialog.Handle) {
        throw 'Actual No button is not a child of the exact owned recovery root.'
    }
    Focus-Ux $element
    Assert-UxWindowForeground $element
    $fresh = Read-OracleRecoveryModal ([OracleNative]::Describe([IntPtr]$Dialog.Handle))
    if (($fresh.native_text | ConvertTo-Json -Depth 8 -Compress) -cne ($record.native_text | ConvertTo-Json -Depth 8 -Compress) -or
        $fresh.identity.fixture_sha256 -cne $script:OracleRequest.fixture_sha256 -or
        $fresh.identity.session -ne $fresh.identity.actor_session) {
        throw 'Owned recovery content, fixture bytes or session changed before No; action refused.'
    }
    $script:OracleRecoveryHandled = $true
    Write-OracleJson (Join-Path $script:OracleRoot 'startup-recovery.action-intent.json') @{
        policy='owned-skip';action='decline_restore';button_id=7;dialog_handle=$Dialog.Handle
        button=(Read-UxElement $no);foreground_handle=[OracleNative]::GetForegroundWindow().ToInt64()
        before_fixture_sha256=$fresh.identity.fixture_sha256
        backup_copy='Not attempted: no exact owned backup path was exposed; no unrelated project/cache search.'
        scope='Actual native No only. Native UI may clear this lab recovery entry; cache preservation is not claimed.'
    }
    Click-Ux $no
    $until = [DateTime]::UtcNow.AddSeconds(5)
    do {
        [void](Assert-OracleIdentity)
        $windows = @([OracleNative]::Windows([uint32]$script:OracleTarget.pid))
        if (!@($windows | Where-Object { $_.Handle -eq $Dialog.Handle }).Count) { break }
        Start-Sleep -Milliseconds 100
    } while ([DateTime]::UtcNow -lt $until)
    $afterHash = (Get-FileHash -LiteralPath (Join-Path $script:OracleRoot $script:OracleRequest.fixture_filename)).Hash.ToLowerInvariant()
    $gone = !@($windows | Where-Object { $_.Handle -eq $Dialog.Handle }).Count
    Write-OracleJson (Join-Path $script:OracleRoot 'startup-recovery.after-action.json') @{
        action='decline_restore';button_id=7;dialog_gone=$gone;owned_windows=$windows
        before_fixture_sha256=$fresh.identity.fixture_sha256;after_fixture_sha256=$afterHash
    }
    if (!$gone -or $afterHash -cne $script:OracleRequest.fixture_sha256) {
        throw 'Native recovery decline did not close the exact dialog with unchanged pinned fixture bytes.'
    }
    Write-OracleEvent 'owned-recovery-declined'
}

function Assert-OracleDesktopReady {
    Initialize-OracleNative
    $actorSession = (Get-Process -Id $PID).SessionId
    $foreground = [OracleNative]::GetForegroundWindow()
    $foregroundPid = [OracleNative]::Owner($foreground)
    $foregroundName = if ($foregroundPid) { (Get-Process -Id $foregroundPid -ErrorAction SilentlyContinue).ProcessName } else { $null }
    $actorIntegrity = [OracleNative]::Integrity([uint32]$PID)
    $foregroundIntegrity = if ($foregroundPid) { [OracleNative]::Integrity($foregroundPid) } else { 0 }
    $desktop = [OracleNative]::InputDesktop()
    $consoleSession = [OracleNative]::WTSGetActiveConsoleSessionId()
    $reason = $null
    if ($actorSession -ne $consoleSession -or $desktop -cne 'Default') {
        $reason = 'Notebook console/input desktop is locked, unavailable or noninteractive; an operator must restore its normal unlocked desktop.'
    } elseif ($actorIntegrity -lt 0 -or ($foregroundPid -and $foregroundIntegrity -lt 0)) {
        $reason = 'Foreground integrity could not be verified; no GUI activation was attempted.'
    } elseif ($foregroundPid -and $foregroundIntegrity -gt $actorIntegrity) {
        $reason = "A higher-integrity foreground input owner ($foregroundName, PID $foregroundPid) blocks safe notebook activation. An operator must release input sharing or focus its normal user desktop; no foreground-policy/security bypass was attempted."
    }
    Write-OracleJson (Join-Path $script:OracleRoot 'desktop-readiness.json') @{
        actor_session=$actorSession;active_console_session=$consoleSession;input_desktop=$desktop
        actor_integrity_rid=$actorIntegrity;foreground_pid=$foregroundPid;foreground_process_name=$foregroundName;foreground_handle=$foreground.ToInt64()
        foreground_integrity_rid=$foregroundIntegrity;allowed=(!$reason);reason=$reason
        scope='Read-only metadata, no foreground-window text, capture, input or security changes.'
    }
    if ($reason) { throw $reason }
}

function Focus-OracleMain($Main, [string]$Label='initial') {
    $windows = @([OracleNative]::Windows([uint32]$script:OracleTarget.pid))
    $uiaHandle = [IntPtr]$Main.Current.NativeWindowHandle
    $rootHandle = [OracleNative]::GetAncestor($uiaHandle,2)
    if ($rootHandle -eq [IntPtr]::Zero -or [OracleNative]::Owner($rootHandle) -ne $script:OracleTarget.pid) {
        throw 'UIA main window does not resolve to an exact owned GA_ROOT.'
    }
    $chain = [Collections.Generic.List[long]]::new()
    $ancestor = $uiaHandle
    for ($depth=0;$depth -lt 16;$depth++) {
        if ([OracleNative]::Owner($ancestor) -ne $script:OracleTarget.pid) { throw 'Main window parent chain leaves the owned process.' }
        $chain.Add($ancestor.ToInt64())
        if ($ancestor -eq $rootHandle) { break }
        $ancestor = [OracleNative]::GetAncestor($ancestor,1)
        if ($ancestor -eq [IntPtr]::Zero) { break }
    }
    if ($ancestor -ne $rootHandle) { throw 'Owned GA_ROOT parent chain could not be verified.' }
    [uint32]$foregroundOwner = 0
    [void][LoxUxNative]::GetWindowThreadProcessId([LoxUxNative]::GetForegroundWindow(),[ref]$foregroundOwner)
    Write-OracleJson (Join-Path $script:OracleRoot ('main-window.'+$Label+'.identity.json')) @{
        uia=(Read-UxElement $Main);uia_native_handle=$uiaHandle.ToInt64();validated_root_handle=$rootHandle.ToInt64()
        parent_chain=@($chain.ToArray());root_policy='GA_ROOT=2 only; never GA_ROOTOWNER or same-PID substitution.'
        native_windows=$windows;foreground_handle=[LoxUxNative]::GetForegroundWindow().ToInt64()
        foreground_pid=$foregroundOwner
    }
    $ours = @($windows | Where-Object { $_.Handle -eq $rootHandle.ToInt64() -and
        ($_.ClassName -eq 'CLxMainWindow' -or $_.ClassName -match '^Qt[0-9]+QWindowIcon') })
    if ($ours.Count -ne 1 -or $ours[0].Cloaked -ne 0) {
        throw 'The exact owned native window is cloaked or not on the visible notebook desktop.'
    }
    for ($attempt=0;$attempt -lt 3;$attempt++) {
        try {
            if ($rootHandle -eq $uiaHandle) {
                Focus-Ux $Main
            } else {
                $until = [DateTime]::UtcNow.AddSeconds(2)
                do {
                    [void][LoxUxNative]::Activate($rootHandle)
                    if ([LoxUxNative]::GetForegroundWindow() -eq $rootHandle -and
                        [OracleNative]::Owner($rootHandle) -eq $script:OracleTarget.pid) { break }
                    Start-Sleep -Milliseconds 80
                } while ([DateTime]::UtcNow -lt $until)
                if ([LoxUxNative]::GetForegroundWindow() -ne $rootHandle) { throw 'Exact validated GA_ROOT could not obtain the foreground.' }
            }
            return
        }
        catch {
            $handle = [LoxUxNative]::GetForegroundWindow()
            [uint32]$owner = 0
            [void][LoxUxNative]::GetWindowThreadProcessId($handle,[ref]$owner)
            $visible = @([OracleNative]::Windows([uint32]$script:OracleTarget.pid))
            Write-OracleJson (Join-Path $script:OracleRoot ('focus-'+$Label+'-attempt-'+$attempt+'.json')) @{
                requested_handle=$rootHandle.ToInt64();foreground_handle=$handle.ToInt64()
                foreground_pid=$owner;owned_windows=$visible
            }
            if ($attempt -eq 2) {
                $tabs = Get-OracleControl @('.CLxTitleBar.CTitleBarTabs') -Optional
                if ($tabs -and $tabs.Current.ClassName -eq 'CTitleBarTabs') {
                    $r = $tabs.Current.BoundingRectangle
                    foreach ($fraction in @(0.85,0.7,0.95)) {
                        $x = [int]($r.Left+$fraction*$r.Width)
                        $y = [int]($r.Top+$r.Height/2)
                        $hit = Get-UxHitElement $x $y
                        [uint32]$hitOwner = 0
                        [void][LoxUxNative]::GetWindowThreadProcessId([LoxUxNative]::WindowFromPoint([LoxUxNative+Point]::new($x,$y)),[ref]$hitOwner)
                        Write-OracleJson (Join-Path $script:OracleRoot ('focus-'+$Label+'-candidate-'+([int]($fraction*100))+'.json')) @{
                            x=$x;y=$y;hit_owner=$hitOwner
                            hit=$(if($hitOwner -eq $script:OracleTarget.pid){Read-UxElement $hit}else{$null})
                        }
                        if ($hitOwner -ne $script:OracleTarget.pid -or $hit.Current.ProcessId -ne $script:OracleTarget.pid) { continue }
                        $blank = $hit.Current.AutomationId -ceq $tabs.Current.AutomationId
                        if (!$blank -and $hit.Current.AutomationId.EndsWith('.CLxTitleBar')) {
                            $buttons = @($tabs.FindAll([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.Condition]::TrueCondition) |
                                Where-Object { $_.Current.ClassName -match 'Button' -and
                                    $_.Current.BoundingRectangle.Contains([Windows.Point]::new($x,$y)) })
                            $blank = $buttons.Count -eq 0
                        }
                        if (!$blank) { continue }
                        [void](Assert-OracleIdentity)
                        [void][LoxUxNative]::SetCursorPos($x,$y)
                        Assert-UxHit $hit $x $y
                        Write-OracleJson (Join-Path $script:OracleRoot ('focus-'+$Label+'-owned-titlebar-click.json')) @{
                            pid=$script:OracleTarget.pid;hwnd=$rootHandle.ToInt64();x=$x;y=$y
                            binding='Only the hit-tested blank title-tab container; never a tab/button, Alt key, taskbar or foreign window.'
                        }
                        [LoxUxNative]::mouse_event(2,0,0,0,[UIntPtr]::Zero)
                        [LoxUxNative]::mouse_event(4,0,0,0,[UIntPtr]::Zero)
                        Start-Sleep -Milliseconds 150
                        [void][LoxUxNative]::Activate($rootHandle)
                        if ([LoxUxNative]::GetForegroundWindow() -ne $rootHandle -or
                            [OracleNative]::Owner($rootHandle) -ne $script:OracleTarget.pid) {
                            throw 'Owned title activation did not yield the exact validated GA_ROOT.'
                        }
                        return
                    }
                }
            }
            if ($attempt -eq 2) { throw }
            Start-Sleep -Milliseconds 1500
        }
    }
}

function Save-OracleScreen([string]$Path) {
    return (Save-OracleWindowScreen (Get-UxWindow 'CLxMainWindow') $Path)
}

function Save-OracleWindowScreen($Window, [string]$Path) {
    Assert-UxForeground
    Assert-UxOwned $Window
    $rectangle = $Window.Current.BoundingRectangle
    $canvasRecord = $null
    $inset = 0
    $virtual = [Windows.Forms.SystemInformation]::VirtualScreen
    $captureDpi = [OracleNative]::GetDpiForWindow([OracleNative]::GetAncestor([IntPtr]$Window.Current.NativeWindowHandle,2))
    if ($script:OracleRequest.capture_scope -eq 'canvas') {
        $views = @($Window.FindAll([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.Condition]::TrueCondition) |
            Where-Object { $_.Current.ProcessId -eq $script:UxProcessId -and
                $_.Current.ClassName -eq 'AfxFrameOrView140u' -and !$_.Current.IsOffscreen })
        if ($views.Count -ne 1) { throw 'Owned native programming canvas is not uniquely exposed by UI Automation.' }
        Assert-UxOwned $views[0]
        $canvas = $views[0].Current.BoundingRectangle
        $canvasHandle = [IntPtr]$views[0].Current.NativeWindowHandle
        $mainRoot = [OracleNative]::GetAncestor([IntPtr]$Window.Current.NativeWindowHandle,2)
        if ($canvasHandle -eq [IntPtr]::Zero -or [OracleNative]::GetAncestor($canvasHandle,2) -ne $mainRoot -or
            [OracleNative]::Owner($canvasHandle) -ne $script:UxProcessId) {
            throw 'Native canvas HWND does not belong to the exact owned main root.'
        }
        $inset = 64
        $left = [Math]::Ceiling([Math]::Max($rectangle.Left,$canvas.Left)+$inset)
        $top = [Math]::Ceiling([Math]::Max($rectangle.Top,$canvas.Top)+$inset)
        $right = [Math]::Floor([Math]::Min($rectangle.Right,$canvas.Right)-$inset)
        $bottom = [Math]::Floor([Math]::Min($rectangle.Bottom,$canvas.Bottom)-$inset)
        $left = [Math]::Max($left,$virtual.Left)
        $top = [Math]::Max($top,$virtual.Top)
        $right = [Math]::Min($right,$virtual.Right)
        $bottom = [Math]::Min($bottom,$virtual.Bottom)
        if ($right -le $left -or $bottom -le $top) { throw 'Owned canvas capture rectangle is empty.' }
        $canvasRecord = @{
            uia=(Read-UxElement $views[0]);native_handle=$canvasHandle.ToInt64();root_handle=$mainRoot.ToInt64()
            bounds=@($canvas.Left,$canvas.Top,$canvas.Right,$canvas.Bottom)
            main_bounds=@($rectangle.Left,$rectangle.Top,$rectangle.Right,$rectangle.Bottom)
        }
        $rectangle = [Windows.Rect]::new($left,$top,$right-$left,$bottom-$top)
    }
    if ($rectangle.Width * $rectangle.Height -gt 12000000 -or $rectangle.Width -lt 400) {
        throw 'Owned application capture extent is unsupported.'
    }
    $root = [OracleNative]::GetAncestor([IntPtr]$Window.Current.NativeWindowHandle,2)
    Write-OracleJson ($Path+'.geometry.json') @{
        root_handle=$root.ToInt64();dpi=$captureDpi;owned_canvas=$canvasRecord;inset_pixels=$inset
        rectangle=@($rectangle.Left,$rectangle.Top,$rectangle.Width,$rectangle.Height)
        origin_screen=@($rectangle.Left,$rectangle.Top)
        virtual_screen_bounds=@($virtual.Left,$virtual.Top,$virtual.Right,$virtual.Bottom)
        scope=$(if($canvasRecord){'owned-native-programming-canvas'}else{'owned-application'})
    }
    if ($root -eq [IntPtr]::Zero -or [OracleNative]::Owner($root) -ne $script:UxProcessId -or
        [LoxUxNative]::GetForegroundWindow() -ne $root) {
        throw 'The exact owned GA_ROOT must be foreground before application capture.'
    }
    $occluder = [OracleNative]::FirstOccluder($root,[uint32]$script:UxProcessId,
        [int[]]@($rectangle.Left,$rectangle.Top,$rectangle.Right,$rectangle.Bottom))
    Write-OracleJson ($Path+'.occlusion-check.json') @{
        root_handle=$root.ToInt64();rectangle=@($rectangle.Left,$rectangle.Top,$rectangle.Right,$rectangle.Bottom)
        checked_foreign_windows=@([OracleNative]::CheckedForeignWindows)
        verified_noncomposited=@([OracleNative]::VerifiedNoncomposited);occluder=$occluder
        scope='Exact Z-order checks; outside-ROI windows are measured, not hidden or whitelisted. No foreign text/pixels.'
    }
    if ($occluder) {
        $processName = if ($occluder.ProcessId) { (Get-Process -Id $occluder.ProcessId -ErrorAction SilentlyContinue).ProcessName } else { $null }
        Write-OracleJson ($Path+'.occlusion.json') @{
            captured=$false;root_handle=$root.ToInt64();occluder=$occluder;process_name=$processName
            verified_noncomposited=@([OracleNative]::VerifiedNoncomposited)
            rectangle=@($rectangle.Left,$rectangle.Top,$rectangle.Right,$rectangle.Bottom)
            scope='Read-only first-overlap PID/class/geometry/cloaking/alpha metadata; no foreign window text or pixels.'
        }
        throw "Foreign top-level window PID $($occluder.ProcessId), class $($occluder.ClassName), overlaps the exact owned application capture."
    }
    foreach ($fx in @(0.01,0.2,0.4,0.6,0.8,0.99)) {
        foreach ($fy in @(0.01,0.2,0.4,0.6,0.8,0.99)) {
            $x = [int]($rectangle.Left + $rectangle.Width * $fx)
            $y = [int]($rectangle.Top + $rectangle.Height * $fy)
            [uint32]$owner = 0
            [void][LoxUxNative]::GetWindowThreadProcessId([LoxUxNative]::WindowFromPoint([LoxUxNative+Point]::new($x,$y)), [ref]$owner)
            if ($owner -ne $script:UxProcessId) { throw 'A foreign window occludes the owned capture.' }
        }
    }
    if (Test-Path -LiteralPath $Path) { throw 'Capture already exists.' }
    $bitmap = [Drawing.Bitmap]::new([int]$rectangle.Width, [int]$rectangle.Height)
    $graphics = [Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.CopyFromScreen([int]$rectangle.Left, [int]$rectangle.Top, 0, 0, $bitmap.Size)
        $bitmap.Save($Path, [Drawing.Imaging.ImageFormat]::Png)
    } finally { $graphics.Dispose(); $bitmap.Dispose() }
    return @{
        backend='owned-window-CopyFromScreen';rectangle=@($rectangle.Left,$rectangle.Top,$rectangle.Width,$rectangle.Height)
        occlusion_grid=36;verified_noncomposited=@([OracleNative]::VerifiedNoncomposited)
        scope=$(if($canvasRecord){'owned-native-programming-canvas'}else{'owned-application'})
        origin_screen=@($rectangle.Left,$rectangle.Top);owned_canvas=$canvasRecord;inset_pixels=$inset
        root_handle=$root.ToInt64();dpi=$captureDpi
        virtual_screen_bounds=@($virtual.Left,$virtual.Top,$virtual.Right,$virtual.Bottom)
        inset_basis=$(if($canvasRecord){'fixed-64-physical-pixels'}else{'not-applicable'})
        checked_foreign_windows=@([OracleNative]::CheckedForeignWindows)
    }
}

function Initialize-OracleOcr {
    Add-Type -AssemblyName System.Runtime.WindowsRuntime
    [Windows.Media.Ocr.OcrEngine,Windows.Foundation,ContentType=WindowsRuntime] | Out-Null
    [Windows.Storage.StorageFile,Windows.Storage,ContentType=WindowsRuntime] | Out-Null
    [Windows.Graphics.Imaging.BitmapDecoder,Windows.Foundation,ContentType=WindowsRuntime] | Out-Null
    [Windows.Graphics.Imaging.SoftwareBitmap,Windows.Foundation,ContentType=WindowsRuntime] | Out-Null
    [Windows.Storage.Streams.IRandomAccessStream,Windows.Storage.Streams,ContentType=WindowsRuntime] | Out-Null
    $script:OracleAsTask = ([System.WindowsRuntimeSystemExtensions].GetMethods() | Where-Object {
        $_.Name -eq 'AsTask' -and $_.IsGenericMethod -and $_.GetParameters().Count -eq 1 -and
        $_.GetParameters()[0].ParameterType.Name -eq 'IAsyncOperation`1'
    })[0]
    $script:OracleOcrEngine = [Windows.Media.Ocr.OcrEngine]::TryCreateFromUserProfileLanguages()
    if (!$script:OracleOcrEngine) { throw 'No installed Windows OCR recognizer is available.' }
}

function Wait-OracleWinRt($Operation, [Type]$Type) {
    $task = $script:OracleAsTask.MakeGenericMethod($Type).Invoke($null, @($Operation))
    if (!$task.Wait(20000)) { throw 'Windows OCR operation timed out.' }
    return $task.Result
}

function Read-OracleOcr([string]$Image) {
    $file = Wait-OracleWinRt ([Windows.Storage.StorageFile]::GetFileFromPathAsync($Image)) ([Windows.Storage.StorageFile])
    $stream = Wait-OracleWinRt ($file.OpenAsync([Windows.Storage.FileAccessMode]::Read)) ([Windows.Storage.Streams.IRandomAccessStream])
    try {
        $decoder = Wait-OracleWinRt ([Windows.Graphics.Imaging.BitmapDecoder]::CreateAsync($stream)) ([Windows.Graphics.Imaging.BitmapDecoder])
        $bitmap = Wait-OracleWinRt ($decoder.GetSoftwareBitmapAsync()) ([Windows.Graphics.Imaging.SoftwareBitmap])
        $result = Wait-OracleWinRt ($script:OracleOcrEngine.RecognizeAsync($bitmap)) ([Windows.Media.Ocr.OcrResult])
        return @{
            engine='Windows.Media.Ocr';language=$script:OracleOcrEngine.RecognizerLanguage.LanguageTag
            lines=@($result.Lines | ForEach-Object {
                @{text=$_.Text;words=@($_.Words | ForEach-Object {
                    @{text=$_.Text;rect=@($_.BoundingRect.X,$_.BoundingRect.Y,$_.BoundingRect.Width,$_.BoundingRect.Height)}
                })}
            })
        }
    } finally { $stream.Dispose() }
}

function Read-OracleOutput($Ocr, [string]$Image, [string]$Phase) {
    $headers = @($Ocr.lines | Where-Object { ($_.text -replace '\s','') -ceq 'EnergyManager' })
    if ($headers.Count -ne 1 -or $headers[0].words.Count -lt 2) {
        return @{outputs=@{};artifacts=@();reason='Native Energy Manager header is not uniquely readable.'}
    }
    $words = $headers[0].words
    $left = ($words | ForEach-Object { $_.rect[0] } | Measure-Object -Minimum).Minimum
    $top = ($words | ForEach-Object { $_.rect[1] } | Measure-Object -Minimum).Minimum
    $right = ($words | ForEach-Object { $_.rect[0]+$_.rect[2] } | Measure-Object -Maximum).Maximum
    $bottom = ($words | ForEach-Object { $_.rect[1]+$_.rect[3] } | Measure-Object -Maximum).Maximum
    $width = $right-$left
    $height = $bottom-$top
    # Windows OCR locates the header only. Output admission uses the separately
    # pinned German Tesseract recipe, never colour or alternative OCR variants.
    $roi = [Drawing.Rectangle]::new(
        [int]($left+2.34*$width),[int]($top+2.6*$height),
        [int](0.9*$width),[int](1.5*$height))
    $original = [Drawing.Bitmap]::new($Image)
    if (!$original.Size.Width -or $roi.Right -gt $original.Width -or $roi.Bottom -gt $original.Height -or
        $roi.Left -lt 0 -or $roi.Top -lt 0 -or $roi.Width -lt 12 -or $roi.Height -lt 8) {
        $original.Dispose()
        return @{outputs=@{};artifacts=@();reason='Native output ROI is outside the actual owned bitmap.'}
    }
    $anchorName = $Phase+'.output-anchor.json'
    Write-OracleJson (Join-Path $script:OracleRoot $anchorName) @{
        header=$headers[0];source_bitmap=$Image
        roi=@($roi.X,$roi.Y,$roi.Width,$roi.Height);scale=4
        binding='Config 17.2.10.21 public energy fixture OL1 pin and wired State.I1 reference; native raster text only.'
    }
    $original.Dispose()
    return @{outputs=@{};artifacts=@($anchorName);reason='Output admission requires the pinned file-only Tesseract recipe and independent word agreement at confidence >=90.'}
}

function Save-OracleSample([string]$Phase) {
    $main = Get-UxWindow 'CLxMainWindow'
    $clock = Get-OracleControl @('.CSettingEditorTimeFancy.CSettingEditorTime')
    $pause = Get-OracleControl @('.Testing.CRibbonWidget::CRibbonButton')
    $live = Get-OracleControl @('.Testing.LIVE')
    $uia = @($main.FindAll([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.Condition]::TrueCondition) |
        ForEach-Object { Read-UxElement $_ })
    Write-OracleJson (Join-Path $script:OracleRoot ($Phase + '.uia.json')) $uia
    $image = Join-Path $script:OracleRoot ($Phase + '.png')
    $timeBefore = Get-UxValue $clock
    $pauseEnabled = $pause.Current.IsEnabled
    $liveEnabled = $live.Current.IsEnabled
    $captureBegin = $script:OracleClock.Elapsed.TotalMilliseconds
    $capture = Save-OracleScreen $image
    $captureEnd = $script:OracleClock.Elapsed.TotalMilliseconds
    $timeAfter = Get-UxValue $clock
    $ocr = $null
    $ocrError = $null
    $output = @{outputs=@{};artifacts=@();reason='Output OCR was unavailable.'}
    try {
        $ocr = Read-OracleOcr $image
        Write-OracleJson (Join-Path $script:OracleRoot ($Phase + '.ocr.json')) $ocr
        $output = Read-OracleOutput $ocr $image $Phase
    } catch { $ocrError = $_.Exception.Message }
    return @{
        phase=$Phase;elapsed_ms=[Math]::Round(($captureBegin+$captureEnd)/2)
        capture_interval_ms=@([Math]::Round($captureBegin),[Math]::Round($captureEnd))
        simulator_time=$timeBefore;simulator_time_after_capture=$timeAfter;pause_enabled=$pauseEnabled;live_enabled=$liveEnabled
        capture=$capture;ocr_error=$ocrError;outputs=$output.outputs;readback_reason=$output.reason
        source=@{kind='ocr';artifacts=@(($Phase+'.png'),($Phase+'.uia.json')) + $(if($ocr){@($Phase+'.ocr.json')}else{@()}) + $output.artifacts}
    }
}

function Assert-OracleClockFocus($Clock) {
    Assert-UxOwned $Clock
    Assert-UxForeground
    $main = Get-UxWindow 'CLxMainWindow'
    $root = [OracleNative]::GetAncestor([IntPtr]$main.Current.NativeWindowHandle,2)
    if ($root -eq [IntPtr]::Zero -or [OracleNative]::Owner($root) -ne $script:OracleTarget.pid -or
        [OracleNative]::GetForegroundWindow() -ne $root) { throw 'Clock editing lost the exact owned foreground root.' }
    $wanted = $Clock.GetRuntimeId() -join ','
    $focused = [Windows.Automation.AutomationElement]::FocusedElement
    for ($depth=0;$focused -and $depth -lt 16;$depth++) {
        Assert-UxOwned $focused
        if (($focused.GetRuntimeId() -join ',') -ceq $wanted) { return }
        $focused = Get-UxParent $focused
    }
    throw 'Keyboard focus is not the selected native simulator clock or its child.'
}

function Wait-OracleClockReadback($Clock, [string]$Expected) {
    $readings = [Collections.Generic.List[object]]::new()
    $until = [DateTime]::UtcNow.AddSeconds(2)
    do {
        $actual = Get-UxValue $Clock
        $readings.Add(@{utc=[DateTime]::UtcNow.ToString('o');value=$actual})
        if (($actual -replace '\s','') -ceq $Expected) { break }
        Start-Sleep -Milliseconds 100
    } while ([DateTime]::UtcNow -lt $until)
    return @{actual=$actual;readings=@($readings.ToArray());matched=(($actual -replace '\s','') -ceq $Expected)}
}

function Set-OracleClockSections($Clock, [string]$Value) {
    $target = @($Value.Split(':') | ForEach-Object { [int]$_ })
    Click-Ux $Clock
    Assert-OracleClockFocus $Clock
    for ($section=0;$section -lt 3;$section++) {
        Assert-OracleClockFocus $Clock
        Send-UxKeys '{HOME}'
        if ($section) { Assert-OracleClockFocus $Clock; Send-UxKeys ('{RIGHT}'*(3*$section)) }
        $before = (Get-UxValue $Clock) -replace '\s',''
        if ($before -cnotmatch '^(?:[01][0-9]|2[0-3]):[0-5][0-9]:[0-5][0-9]$') {
            throw 'Native clock is not an exact HH:MM:SS value before section editing.'
        }
        $parts = @($before.Split(':') | ForEach-Object { [int]$_ })
        $limit = if($section -eq 0){23}else{59}
        $probeDelta = if($parts[$section] -eq $limit){-1}else{1}
        $parts[$section] += $probeDelta
        $probeExpected = '{0:D2}:{1:D2}:{2:D2}' -f $parts[0],$parts[1],$parts[2]
        Assert-OracleClockFocus $Clock
        Send-UxKeys $(if($probeDelta -gt 0){'{UP}'}else{'{DOWN}'})
        $probe = Wait-OracleClockReadback $Clock $probeExpected
        Write-OracleJson (Join-Path $script:OracleRoot ('simulator-start-time.section-'+$section+'.probe.json')) @{
            section=$section;before=$before;expected_probe=$probeExpected;actual_probe=$probe
            probe_delta=$probeDelta
            binding='HOME plus 0/3/6 character moves and one nonwrapping arrow probe; only the intended native field may change.'
        }
        if (!$probe.matched) { throw 'Native clock keyboard section probe did not change exactly the intended field.' }
        $delta = $target[$section]-$parts[$section]
        if ($delta) {
            Assert-OracleClockFocus $Clock
            $key = if($delta -gt 0){'{UP}'}else{'{DOWN}'}
            Send-UxKeys ($key*[Math]::Abs($delta))
        }
        $parts[$section] = $target[$section]
        $expected = '{0:D2}:{1:D2}:{2:D2}' -f $parts[0],$parts[1],$parts[2]
        $readback = Wait-OracleClockReadback $Clock $expected
        Write-OracleJson (Join-Path $script:OracleRoot ('simulator-start-time.section-'+$section+'.result.json')) @{
            section=$section;delta=$delta;expected=$expected;actual=$readback
        }
        if (!$readback.matched) { throw 'Native clock section did not reach its declared value with other fields unchanged.' }
    }
    return Get-UxValue $Clock
}

function Set-OracleSimulatorStartTime([string]$Value) {
    if ($Value -cnotmatch '^(?:[01][0-9]|2[0-3]):[0-5][0-9]:[0-5][0-9]$') {
        throw 'Simulator-local start time must be exact HH:MM:SS.'
    }
    $currentTime = Get-OracleControl @('.CActionSimulationSettings::CActionSimulationSettingsWidget.CSettingEditorCheckBox')
    $toggle = $null
    if (!$currentTime.TryGetCurrentPattern([Windows.Automation.TogglePattern]::Pattern,[ref]$toggle)) {
        throw 'Native simulator current-time checkbox has no verified TogglePattern.'
    }
    $beforeToggle = $toggle.Current.ToggleState
    Write-OracleJson (Join-Path $script:OracleRoot 'simulator-start-time.intent.json') @{
        requested=$Value;current_time_before=$beforeToggle.ToString();control=(Read-UxElement $currentTime)
        scope='Owned native simulator-local controls before Start; one guarded checkbox click at most.'
    }
    if ($beforeToggle -eq [Windows.Automation.ToggleState]::On) { Click-Ux $currentTime }
    $toggleReads = [Collections.Generic.List[object]]::new()
    $until = [DateTime]::UtcNow.AddSeconds(2)
    do {
        $actualToggle = $toggle.Current.ToggleState
        $toggleReads.Add(@{utc=[DateTime]::UtcNow.ToString('o');state=$actualToggle.ToString()})
        if ($actualToggle -eq [Windows.Automation.ToggleState]::Off) { break }
        Start-Sleep -Milliseconds 100
    } while ([DateTime]::UtcNow -lt $until)
    Write-OracleJson (Join-Path $script:OracleRoot 'simulator-start-time.toggle-readback.json') @{
        requested_current_time='Off';readings=@($toggleReads.ToArray())
    }
    if ($actualToggle -ne [Windows.Automation.ToggleState]::Off) {
        throw 'Native simulator current-time checkbox is not definitely OFF.'
    }
    $clock = Get-OracleControl @('.CSettingEditorTimeFancy.CSettingEditorTime')
    $before = Get-UxValue $clock
    $method = $script:OracleRequest.simulator_time_edit
    if (!$method) { $method='value-pattern' }
    if ($method -ceq 'verified-sections') {
        if (!$clock.Current.IsEnabled) { throw 'Native simulator clock is disabled.' }
        $after = Set-OracleClockSections $clock $Value
        $readback = @{actual=$after}
    } elseif ($method -ceq 'value-pattern') {
        $pattern = $null
        if (!$clock.Current.IsEnabled -or
            !$clock.TryGetCurrentPattern([Windows.Automation.ValuePattern]::Pattern,[ref]$pattern) -or
            $pattern.Current.IsReadOnly) {
            throw 'Native simulator time editor has no writable ValuePattern; recover a guarded native edit binding.'
        }
        Assert-UxOwned $clock
        Assert-UxForeground
        $pattern.SetValue($Value)
        $readback = Wait-OracleClockReadback $clock $Value
        $after = $readback.actual
    } else { throw 'Unsupported simulator-local time edit method.' }
    Write-OracleJson (Join-Path $script:OracleRoot 'simulator-start-time.json') @{
        requested=$Value;before=$before;after=$after;current_time_before=$beforeToggle.ToString()
        current_time_after=$toggle.Current.ToggleState.ToString()
        readback=$readback
        scope='Native simulator-local time control before Start only; no OS clock/date change.'
        method=$method;coordinate_policy='Owned controls and verified keyboard navigation; no guessed segment coordinates.'
    }
    if (($after -replace '\s','') -cne $Value) {
        throw 'Actual native simulator start-time readback does not match; no clock setting was inferred.'
    }
}

function Hide-OracleDocks {
    $main = Get-UxWindow 'CLxMainWindow'
    foreach ($suffix in @('.DocumentsPage.TreeWindow','.DocumentsPage.PropertyWindow')) {
        $controls = @($main.FindAll([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.Condition]::TrueCondition) |
            Where-Object { $_.Current.AutomationId.EndsWith($suffix) -and !$_.Current.IsOffscreen })
        if ($controls.Count -gt 1) { throw 'Dock binding is ambiguous.' }
        if (!$controls.Count) { continue }
        $element = $controls[0]
        $r = $element.Current.BoundingRectangle
        if ($r.Width -lt 100 -or $r.Height -lt 100) { continue }
        # The pinned native dock title bar has a 12-logical-pixel pin inset.
        $inset = 12 * [OracleNative]::GetDpiForWindow([IntPtr]$main.Current.NativeWindowHandle) / 96.0
        $x = [int]($r.Right - $inset)
        $y = [int]($r.Top + $inset)
        Assert-UxHit $element $x $y
        [void][LoxUxNative]::SetCursorPos($x,$y)
        Assert-UxForeground
        [LoxUxNative]::mouse_event(2,0,0,0,[UIntPtr]::Zero)
        [LoxUxNative]::mouse_event(4,0,0,0,[UIntPtr]::Zero)
        Start-Sleep -Milliseconds 250
    }
}

function Assert-OracleIdentity {
    $process = Get-Process -Id $script:OracleTarget.pid -ErrorAction Stop
    if ($process.StartTime.ToUniversalTime().ToString('o') -ne $script:OracleTarget.start_utc -or
        (Get-FileHash -LiteralPath $process.Path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $script:OracleRequest.application.sha256) {
        throw 'Owned native process identity changed; cleanup was refused.'
    }
    return $process
}

function Cache-OracleNativeObserver($Main, $CanvasCapture) {
    if ($CanvasCapture.scope -ne 'owned-native-programming-canvas') { throw 'Native observer requires the verified canvas capture.' }
    $root = [OracleNative]::GetAncestor([IntPtr]$Main.Current.NativeWindowHandle,2)
    $clock = Get-OracleControl @('.CSettingEditorTimeFancy.CSettingEditorTime')
    $r = $clock.Current.BoundingRectangle
    $mainBounds = $CanvasCapture.owned_canvas.main_bounds
    $clockRect = @(
        [int][Math]::Max($mainBounds[0],[Math]::Floor($r.Left-4)),
        [int][Math]::Max($mainBounds[1],[Math]::Floor($r.Top-4)),
        [int][Math]::Min($mainBounds[2],[Math]::Ceiling($r.Right+4)),
        [int][Math]::Min($mainBounds[3],[Math]::Ceiling($r.Bottom+4))
    )
    $rootNative = [OracleNative]::Describe($root)
    $canvasNative = [OracleNative]::Describe([IntPtr]$CanvasCapture.owned_canvas.native_handle)
    if (!$rootNative -or !$canvasNative -or $rootNative.ProcessId -ne $script:OracleTarget.pid -or
        $canvasNative.ProcessId -ne $script:OracleTarget.pid -or
        [OracleNative]::GetAncestor([IntPtr]$canvasNative.Handle,2) -ne $root) {
        throw 'Pre-attach native observer HWND ownership is invalid.'
    }
    $script:OracleObserverCache = @{
        schema_version=1;observer_mode='native-win32';cached_before_attach_utc=[DateTime]::UtcNow.ToString('o')
        target=$script:OracleTarget;root=$rootNative;canvas=$canvasNative
        canvas_capture=$CanvasCapture;clock_uia=(Read-UxElement $clock);clock_rectangle=$clockRect
        header_geometry_source='progress_running.ocr.json'
        post_attach_readiness='Two actual post-attach clock OCR reads; no WM_NULL/Ping liveness proxy.'
        scope='No UIA provider/tree calls or input after attachment; cached geometry is not an output observation.'
    }
    Write-OracleJson (Join-Path $script:OracleRoot 'native-observer-cache.json') $script:OracleObserverCache
}

function Assert-OracleNativeObserver {
    $process = Assert-OracleIdentity
    $cache = $script:OracleObserverCache
    if (!$cache -or $process.SessionId -ne (Get-Process -Id $PID).SessionId) { throw 'Native observer session/cache is invalid.' }
    $root = [OracleNative]::Describe([IntPtr]$cache.root.Handle)
    $canvas = [OracleNative]::Describe([IntPtr]$cache.canvas.Handle)
    if (!$root -or !$canvas -or $root.ProcessId -ne $script:OracleTarget.pid -or
        $canvas.ProcessId -ne $script:OracleTarget.pid -or $root.ClassName -cne $cache.root.ClassName -or
        $canvas.ClassName -cne $cache.canvas.ClassName -or $root.Cloaked -ne 0 -or
        (($root.Rectangle -join ',') -cne ($cache.root.Rectangle -join ',')) -or
        (($canvas.Rectangle -join ',') -cne ($cache.canvas.Rectangle -join ',')) -or
        [OracleNative]::GetAncestor([IntPtr]$canvas.Handle,2).ToInt64() -ne $root.Handle -or
        [OracleNative]::GetForegroundWindow().ToInt64() -ne $root.Handle) {
        throw 'Exact native observer PID/root/foreground/geometry changed after attachment.'
    }
}

function Save-OracleCachedRegion([int[]]$Rectangle, [string]$Path) {
    Assert-OracleNativeObserver
    $bounds = $script:OracleObserverCache.canvas_capture.virtual_screen_bounds
    $main = $script:OracleObserverCache.canvas_capture.owned_canvas.main_bounds
    if ($Rectangle.Count -ne 4 -or $Rectangle[0] -lt $bounds[0] -or $Rectangle[1] -lt $bounds[1] -or
        $Rectangle[2] -gt $bounds[2] -or $Rectangle[3] -gt $bounds[3] -or
        $Rectangle[0] -lt $main[0] -or $Rectangle[1] -lt $main[1] -or
        $Rectangle[2] -gt $main[2] -or $Rectangle[3] -gt $main[3] -or
        $Rectangle[2] -le $Rectangle[0] -or $Rectangle[3] -le $Rectangle[1]) {
        throw 'Cached observer ROI leaves its verified main/virtual-screen bounds.'
    }
    $window = [pscustomobject]@{Handle=$script:OracleObserverCache.root.Handle;Rectangle=$Rectangle}
    $capture = Save-OracleNativeWindowScreen $window $Path
    Write-OracleJson ($Path+'.native-capture.json') @{
        root_handle=$window.Handle;rectangle=$Rectangle;origin_screen=@($Rectangle[0],$Rectangle[1])
        checked_foreign_windows=@([OracleNative]::CheckedForeignWindows)
        verified_noncomposited=@([OracleNative]::VerifiedNoncomposited)
        capture_unix_ms=[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds();observer_mode='native-win32'
    }
    return $capture
}

function Save-OracleNativeSample([string]$Phase) {
    Assert-OracleNativeObserver
    $cache = $script:OracleObserverCache
    $xywh = $cache.canvas_capture.rectangle
    $region = [int[]]@($xywh[0],$xywh[1],($xywh[0]+$xywh[2]),($xywh[1]+$xywh[3]))
    $begin = $script:OracleClock.Elapsed.TotalMilliseconds
    [void](Save-OracleCachedRegion $region (Join-Path $script:OracleRoot ($Phase+'.png')))
    $end = $script:OracleClock.Elapsed.TotalMilliseconds
    [void](Save-OracleCachedRegion ([int[]]$cache.clock_rectangle) (Join-Path $script:OracleRoot ($Phase+'.clock.png')))
    $record = @{
        phase=$Phase;elapsed_ms=[Math]::Round(($begin+$end)/2);capture_interval_ms=@($begin,$end)
        capture=$cache.canvas_capture;simulator_time=$null;outputs=@{}
        observer_mode='native-win32';clock_image=($Phase+'.clock.png')
        clock_capture_unix_ms=[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
        cached_geometry_source='native-observer-cache.json';native_identity_geometry_verified=$true
        readiness_mode='direct-post-attach-clock-ocr'
        source=@{kind='ocr';artifacts=@(($Phase+'.png'),($Phase+'.clock.png'),($Phase+'.png.native-capture.json'),($Phase+'.clock.png.native-capture.json'),'native-observer-cache.json')}
    }
    Write-OracleJson (Join-Path $script:OracleRoot ($Phase+'.native-observation.json')) $record
    return $record
}

function Invoke-OracleCachedAttachment([string]$RequestPath) {
    [void](Assert-OracleIdentity)
    Write-OracleEvent 'post-start-attach-started'
    $text = & python (Join-Path $script:OracleRoot 'official_sim_runner.py') --attach-owned $RequestPath
    if ($LASTEXITCODE -ne 0) { throw 'Independently bounded post-start attacher failed.' }
    $record = $text | ConvertFrom-Json
    if ($record.status -ne 'client_log_opened') { throw 'Post-start attacher failed or reached its independent deadline.' }
    Start-Sleep -Seconds 2
    Assert-OracleNativeObserver
    Write-OracleEvent 'post-start-attach-ready'
    return $record
}

function Stop-OracleNativeObserved {
    $process = Get-Process -Id $script:OracleTarget.pid -ErrorAction SilentlyContinue
    if (!$process) { return @{status='already_exited';pid=$script:OracleTarget.pid;observer_mode='native-win32'} }
    [void](Assert-OracleIdentity)
    $nudge = $null
    $errorReason = $null
    try {
        $text = & python (Join-Path $script:OracleRoot 'official_sim_runner.py') --exit-dump-owned (Join-Path $script:OracleRoot 'request.json')
        if ($LASTEXITCODE -ne 0) { throw 'Bounded pinned exit-and-dump helper failed.' }
        $nudge = $text | ConvertFrom-Json
    } catch { $errorReason=$_.Exception.Message }
    $forced = $false
    $process = Get-Process -Id $script:OracleTarget.pid -ErrorAction SilentlyContinue
    if ($process) {
        $process = Assert-OracleIdentity
        Stop-Process -Id $process.Id -Force
        $forced = $true
    }
    $cleanup = @{
        status='closed';pid=$script:OracleTarget.pid;observer_mode='native-win32'
        exit_dump=$nudge;exit_dump_error=$errorReason;graceful=$false;vendor_graceful_exit_claimed=$false
        kind=$(if(!$forced -and $nudge -and $nudge.status -eq 'exited_and_dumped'){'client-requested-exit-and-dump'}else{'owned-fallback-cleanup'})
        client_requested_exit_and_dump_verified=(!$forced -and $nudge -and $nudge.status -eq 'exited_and_dumped')
        forced_termination_may_invalidate_coverage=$forced
        scope='No post-attach UIA calls; verified public exit-and-dump first, exact owned terminate only as fallback.'
    }
    Write-OracleJson (Join-Path $script:OracleRoot 'native-observer-cleanup.json') $cleanup
    return $cleanup
}

function Stop-OracleOwned {
    if (!$script:OracleTarget) { return @{status='no_target'} }
    $process = Get-Process -Id $script:OracleTarget.pid -ErrorAction SilentlyContinue
    if (!$process) { return @{status='already_exited'} }
    $process = Assert-OracleIdentity
    if ($script:OracleStartupModalBlocked) {
        Stop-Process -Id $process.Id -Force
        return @{
            status='closed';pid=$process.Id;graceful=$false;kind='owned-startup-modal-fallback'
            forced_termination_may_invalidate_coverage=$true
            scope='Unreviewed/failed startup modal: exact owned termination only, no generic dialog button or restore input.'
        }
    }
    $graceful = $false
    $gracefulError = $null
    $gracefulErrorArtifact = $null
    $forced = $false
    try {
        $windows = @([OracleNative]::Windows([uint32]$script:OracleTarget.pid))
        $main = Get-OracleMainFromNative $windows
        if ($main) {
            Focus-OracleMain $main 'cleanup'
            $pause = Get-OracleControl @('.Testing.CRibbonWidget::CRibbonButton') -Optional
            if ($pause -and $pause.Current.IsEnabled) {
                Click-Ux (Get-OracleControl @('.Testing.Start Simulation','.Testing.Simulation starten'))
            }
        } else {
            foreach ($window in $windows) {
                if ($window.ClassName -eq '#32770') {
                    [void][OracleNative]::PostMessage([IntPtr]$window.Handle,0x10,[IntPtr]::Zero,[IntPtr]::Zero)
                }
            }
        }
        [void]$process.CloseMainWindow()
        $until = [DateTime]::UtcNow.AddSeconds(12)
        while (!$process.HasExited -and [DateTime]::UtcNow -lt $until) {
            if (!$main) {
                foreach ($native in @([OracleNative]::Windows([uint32]$script:OracleTarget.pid))) {
                    if ($native.ClassName -in @('#32770','CLxMainWindow') -or $native.ClassName -match '^Qt[0-9]+QWindowIcon') {
                        [void][OracleNative]::PostMessage([IntPtr]$native.Handle,0x10,[IntPtr]::Zero,[IntPtr]::Zero)
                    }
                }
                Start-Sleep -Milliseconds 300
                $process.Refresh()
                continue
            }
            foreach ($window in @(Get-UxWindows)) {
                if ($window.Current.ClassName -eq '#32770') {
                    $no = Get-UxDialogControl $window @(7) 'Button'
                    Focus-Ux $window
                    Click-Ux $no
                } elseif ($window.Current.ClassName -eq 'QMessageBox') {
                    $buttons = @(Find-UxElements $window ([pscustomobject]@{control_type='Button'}) |
                        Where-Object { $_.Current.Name -cin @('Nein','Nicht speichern',"Don't Save",'No') })
                    if ($buttons.Count -eq 1) { Focus-Ux $window; Click-Ux $buttons[0] }
                }
            }
            Start-Sleep -Milliseconds 300
            $process.Refresh()
        }
        $graceful = $process.HasExited
    } catch {
        $gracefulError = $_.Exception.Message
        $gracefulErrorArtifact = 'cleanup-graceful-error-'+[Guid]::NewGuid().ToString('N')+'.json'
        Write-OracleJson (Join-Path $script:OracleRoot $gracefulErrorArtifact) @{
            pid=$script:OracleTarget.pid;reason=$gracefulError;exception_type=$_.Exception.GetType().FullName
        }
    }
    if (!$graceful -and !$process.HasExited) {
        $process = Assert-OracleIdentity
        Stop-Process -Id $process.Id -Force
        $forced = $true
    }
    return @{
        status='closed';graceful=$graceful;pid=$script:OracleTarget.pid
        graceful_error=$gracefulError;graceful_error_artifact=$gracefulErrorArtifact
        forced_termination_may_invalidate_coverage=$forced
    }
}

function Stop-OracleAttacher {
    $path = Join-Path $script:OracleRoot 'attached.json'
    if (Test-Path -LiteralPath $path) {
        $record = [IO.File]::ReadAllText($path) | ConvertFrom-Json
    } else {
        $watchPath = Join-Path $script:OracleRoot 'attach-watchdog-owned.json'
        if (!(Test-Path -LiteralPath $watchPath)) { return @{status='not_started'} }
        $watch = [IO.File]::ReadAllText($watchPath) | ConvertFrom-Json
        $record = @{attacher_pid=$watch.target_pid;attacher_creation_ticks=$watch.target_creation_ticks}
    }
    $process = Get-Process -Id $record.attacher_pid -ErrorAction SilentlyContinue
    if (!$process) { return @{status='already_exited';pid=$record.attacher_pid} }
    if ($process.SessionId -ne (Get-Process -Id $PID).SessionId -or
        $process.StartTime.ToUniversalTime().ToFileTimeUtc() -ne $record.attacher_creation_ticks -or
        (Get-FileHash -LiteralPath $process.Path).Hash.ToLowerInvariant() -ne $script:OracleRequest.instrumentation.drrun_sha256) {
        throw 'Owned attacher identity changed; launcher cleanup refused.'
    }
    $forced = !$process.WaitForExit(3000)
    if ($forced) { Stop-Process -Id $process.Id -Force }
    return @{status='closed';pid=$record.attacher_pid;forced=$forced}
}

function Invoke-OfficialReplay([string]$RequestPath) {
    $script:OracleRequest = Get-Content -LiteralPath $RequestPath -Raw | ConvertFrom-Json
    $request = $script:OracleRequest
    $script:OracleRoot = $request.run_dir
    $script:OracleTarget = $null
    $script:OracleClock = [Diagnostics.Stopwatch]::StartNew()
    $observations = [Collections.Generic.List[object]]::new()
    $errorData = $null
    $cleanup = $null
    $nativeVerified = $false
    $attachment = $null
    $preAttachVerified = $false
    $clockProofPending = $false
    $script:OracleNativeAttachAttempted = $false
    $script:OracleObserverCache = $null
    $script:OracleRecoveryHandled = $false
    $script:OracleStartupModalBlocked = $false
    try {
        $phases = Get-OracleSamplingPlan $request
        if ([Environment]::UserInteractive -ne $true -or (Get-Process -Id $PID).SessionId -eq 0) {
            throw 'The replay must execute in an authenticated interactive desktop session.'
        }
        if (!$request.expected_desktop_host -or $env:COMPUTERNAME -ine $request.expected_desktop_host -or
            !$request.controller_host -or $env:COMPUTERNAME -ieq $request.controller_host) {
            throw 'Replay desktop identity is not the explicitly selected separate notebook.'
        }
        Assert-OracleDesktopReady
        if ((Get-FileHash -LiteralPath $request.application.path).Hash.ToLowerInvariant() -ne $request.application.sha256) {
            throw 'Application hash mismatch.'
        }
        if (@(Get-Process LoxoneConfig -ErrorAction SilentlyContinue).Count) { throw 'An existing native application must not be adopted.' }
        $rules = @(Get-NetFirewallRule -DisplayName 'LoxOracle-Config-Offline' -ErrorAction Stop)
        if ($rules.Count -ne 1 -or $rules[0].Enabled -ne 'True' -or $rules[0].Direction -ne 'Outbound' -or
            $rules[0].Action -ne 'Block' -or $rules[0].Profile -ne 'Any') {
            throw 'Required retained outbound firewall fence is missing.'
        }
        $filter = @($rules[0] | Get-NetFirewallApplicationFilter)
        if ($filter.Count -ne 1 -or $filter[0].Program -ine $request.application.path) { throw 'Firewall fence targets another executable.' }
        $address = @($rules[0] | Get-NetFirewallAddressFilter)
        $ports = @($rules[0] | Get-NetFirewallPortFilter)
        if ($address.Count -ne 1 -or @($address[0].RemoteAddress).Count -ne 1 -or $address[0].RemoteAddress -ne 'Any' -or
            $ports.Count -ne 1 -or $ports[0].Protocol -ne 'Any' -or $ports[0].RemotePort -ne 'Any') {
            throw 'Firewall fence does not block all outbound destinations/protocols.'
        }
        New-Item -ItemType Directory -Path (Join-Path $request.run_dir 'coverage') | Out-Null
        $launchedText = & python (Join-Path $request.run_dir 'official_sim_runner.py') $RequestPath
        if ($LASTEXITCODE -ne 0) { throw 'Sanitized native launcher failed.' }
        $launched = $launchedText | ConvertFrom-Json
        $process = Get-Process -Id $launched.target_pid -ErrorAction Stop
        if ($process.SessionId -ne (Get-Process -Id $PID).SessionId) { throw 'Target is not in this interactive session.' }
        if ($request.mode -eq 'drcov') {
            $current = $process.Id
            $ancestors = @()
            for ($n=0; $n -lt 8; $n++) {
                $info = Get-CimInstance Win32_Process -Filter "ProcessId=$current"
                if (!$info) { break }
                $ancestors += [int]$info.ParentProcessId
                $current = [int]$info.ParentProcessId
                if ($current -eq $launched.launcher_pid) { break }
            }
            if ($launched.launcher_pid -notin $ancestors -and $launched.launcher_pid -ne $process.Id) {
                throw 'Target PID is not descended from the owned instrumentation launcher.'
            }
        }
        $script:OracleTarget = @{pid=$process.Id;start_utc=$process.StartTime.ToUniversalTime().ToString('o');launcher_pid=$launched.launcher_pid}
        Write-OracleJson (Join-Path $request.run_dir 'owned.json') $script:OracleTarget
        Write-OracleEvent 'target-verified'
        . $request.ux_backend
        Initialize-OracleNative
        $uxRequest = @{
            schema_version=1;target=@{pid=$process.Id;expected_exe_sha256=$request.application.sha256}
            artifact_dir=(Join-Path $request.run_dir 'ux-init');steps=@(@{op='status'})
        }
        $ux = Invoke-LoxoneUx -RequestJson ($uxRequest | ConvertTo-Json -Depth 10 -Compress) | ConvertFrom-Json
        if (!$ux.ok) { throw 'Native UX ownership initialization failed.' }
        Write-OracleEvent 'ux-initialized'
        $until = [DateTime]::UtcNow.AddSeconds(90)
        do {
            $windows = @([OracleNative]::Windows([uint32]$process.Id))
            $dialogs = @($windows | Where-Object { $_.ClassName -eq '#32770' })
            if ($dialogs.Count) {
                $script:OracleStartupModalBlocked = $true
                $dialog = $dialogs[0]
                Write-OracleEvent 'startup-modal-detected'
                $label = if($script:OracleRecoveryHandled){'startup-modal-repeat'}else{'startup-modal'}
                Write-OracleJson (Join-Path $request.run_dir ($label+'.native.json')) (Read-OracleRecoveryModal $dialog)
                try {
                    $capture = Save-OracleNativeWindowScreen $dialog (Join-Path $request.run_dir ($label+'.png'))
                    Write-OracleJson (Join-Path $request.run_dir ($label+'.capture.json')) $capture
                } catch {
                    Write-OracleJson (Join-Path $request.run_dir ($label+'.capture-error.json')) @{reason=$_.Exception.Message}
                    throw 'Owned startup modal capture was blocked; no recovery input was attempted.'
                }
                if ($dialogs.Count -ne 1 -or $request.recovery_policy -cne 'owned-skip' -or $script:OracleRecoveryHandled) {
                    throw 'An owned native startup modal blocked project loading; its actual text/capture were retained.'
                }
                Skip-OracleOwnedRecovery $dialog $RequestPath
                $script:OracleStartupModalBlocked = $false
                continue
            }
            $main = Get-OracleMainFromNative $windows
            $testing = if ($main) { Get-OracleControl @('.CategoryButtonContainer.Testing') -Optional } else { $null }
            if ($testing) { break }
            Start-Sleep -Milliseconds 400
        } while ([DateTime]::UtcNow -lt $until)
        if (!$testing) {
            Write-OracleJson (Join-Path $request.run_dir 'startup-native-windows.json') $windows
            throw 'Native offline project did not finish loading.'
        }
        Write-OracleEvent 'project-loaded'
        Focus-OracleMain $main
        Hide-OracleDocks
        Click-Ux $testing
        Write-OracleEvent 'offline-testing-selected'
        $live = Get-OracleControl @('.Testing.LIVE')
        $pause = Get-OracleControl @('.Testing.CRibbonWidget::CRibbonButton')
        if ($live.Current.IsEnabled -or $pause.Current.IsEnabled) { throw 'Expected disconnected, stopped native simulator.' }
        if ($request.simulator_start_time) { Set-OracleSimulatorStartTime $request.simulator_start_time }
        try { Initialize-OracleOcr } catch {
            Write-OracleJson (Join-Path $request.run_dir 'ocr-unavailable.json') @{reason=$_.Exception.Message}
        }
        $observations.Add((Save-OracleSample 'before_start'))
        Write-OracleEvent 'before-start-sampled'
        if ($request.mode -eq 'drcov' -and $request.instrumentation_start -eq 'post-init') {
            $process = Assert-OracleIdentity
            if ($process.SessionId -ne (Get-Process -Id $PID).SessionId) { throw 'Pre-attach native session changed.' }
            Write-OracleEvent 'post-init-attach-started'
            $ownedMainHandle = $main.Current.NativeWindowHandle
            $attachmentText = & python (Join-Path $request.run_dir 'official_sim_runner.py') --attach-owned $RequestPath
            if ($LASTEXITCODE -ne 0) { throw 'Independent bounded post-init attach helper failed.' }
            $attachment = $attachmentText | ConvertFrom-Json
            if ($attachment.status -ne 'client_log_opened') { throw 'Post-init attachment failed or reached its independent deadline.' }
            [void](Assert-OracleIdentity)
            $native = @([OracleNative]::Windows([uint32]$script:OracleTarget.pid))
            $modal = @($native | Where-Object { $_.ClassName -eq '#32770' })
            if ($modal.Count) {
                Write-OracleJson (Join-Path $request.run_dir 'attach-modal.native.json') $modal
                try {
                    [void](Save-OracleNativeWindowScreen $modal[0] (Join-Path $request.run_dir 'attach-modal.png'))
                } catch {
                    Write-OracleJson (Join-Path $request.run_dir 'attach-modal.capture-error.json') @{
                        reason=$_.Exception.Message;exception_type=$_.Exception.GetType().FullName
                        target_pid=$script:OracleTarget.pid;scope='Owned post-init modal capture only.'
                    }
                }
                throw 'An owned native modal blocked post-init attachment.'
            }
            $attachedMain = @($native | Where-Object { $_.Handle -eq $ownedMainHandle })
            if ($attachedMain.Count -ne 1 -or ![OracleNative]::Ping([IntPtr]$attachedMain[0].Handle)) {
                throw 'The exact owned native application is unresponsive after attachment.'
            }
            Write-OracleEvent 'post-init-attach-ready'
        }
        Click-Ux (Get-OracleControl @('.Testing.Start Simulation','.Testing.Simulation starten'))
        Write-OracleEvent 'native-simulation-started'
        $script:OracleClock.Restart()
        Start-Sleep -Seconds 3
        $observations.Add((Save-OracleSample 'initial_running'))
        Start-Sleep -Seconds 3
        $observations.Add((Save-OracleSample 'progress_running'))
        $a = $observations[1]; $b = $observations[2]
        $nativeVerified = $a.pause_enabled -and $b.pause_enabled -and !$a.live_enabled -and !$b.live_enabled -and
            $a.simulator_time -ne $b.simulator_time
        if (!$nativeVerified) { throw 'Native simulator progress was not positively verified.' }
        Write-OracleEvent 'native-progress-verified'
        if ($request.observer_mode -eq 'native-win32') {
            if ($request.mode -ne 'drcov' -or $request.instrumentation_start -ne 'post-start') {
                throw 'Native-only observation requires explicit post-start instrumentation.'
            }
            $preAttachVerified = $nativeVerified
            $nativeVerified = $false
            $clockProofPending = $true
            Cache-OracleNativeObserver $main $b.capture
            $script:OracleNativeAttachAttempted = $true
            $attachment = Invoke-OracleCachedAttachment $RequestPath
            Start-Sleep -Seconds 3
            $observations.Add((Save-OracleNativeSample 'post_attach_clock_1'))
            Start-Sleep -Seconds 3
            $observations.Add((Save-OracleNativeSample 'post_attach_clock_2'))
        }
        foreach ($phase in $phases) {
            $remaining = $phase.at_seconds - $script:OracleClock.Elapsed.TotalSeconds
            if ($remaining -gt 0) { Start-Sleep -Milliseconds ([int]($remaining * 1000)) }
            $sample = if ($request.observer_mode -eq 'native-win32') {
                Save-OracleNativeSample $phase.name
            } else { Save-OracleSample $phase.name }
            $observations.Add($sample)
            Write-OracleEvent ($phase.name+'-sampled')
            if ($phase.latest_seconds -and $sample.capture_interval_ms[1] -gt $phase.latest_seconds*1000) {
                throw 'Actual native capture missed its finite sampling window; retained observation is not a timing pass.'
            }
        }
    } catch { $errorData = @{code='native_replay_blocked';message=$_.Exception.Message} }
    finally {
        Write-OracleEvent 'cleanup-started'
        try {
            $cleanup = if ($script:OracleNativeAttachAttempted) { Stop-OracleNativeObserved } else { Stop-OracleOwned }
        }
        catch { $cleanup = @{status='blocked';reason=$_.Exception.Message} }
        try { $cleanup.attacher = Stop-OracleAttacher }
        catch { $cleanup.attacher = @{status='blocked';reason=$_.Exception.Message};$cleanup.status='blocked' }
        Write-OracleEvent 'cleanup-finished'
    }
    if ($request.mode -eq 'drcov' -and !$errorData) {
        $logs = @(Get-ChildItem (Join-Path $request.run_dir 'coverage') -Filter '*.log' -File -ErrorAction SilentlyContinue |
            Where-Object { $_.Length -gt 0 })
        if (!$logs.Count) { $errorData = @{code='coverage_log_empty';message='No nonempty actual native drcov log was flushed; output success cannot admit instrumentation.'} }
    }
    $report = @{
        schema_version=1;kind='official-native-replay-remote';target=$script:OracleTarget
        run_id=$request.run_id
        attachment=$attachment
        observer_mode=$request.observer_mode
        pre_attach_running_verified=[bool]$preAttachVerified;post_attach_clock_proof_pending=[bool]$clockProofPending
        coverage_scope=$(if($request.instrumentation_start -eq 'post-start'){'Application startup and SimulationStart excluded; attachment occurred during an already verified native simulation.'}else{'See attachment metadata.'})
        native_running_verified=[bool]$nativeVerified;observations=@($observations.ToArray())
        error=$errorData;cleanup=$cleanup;remote_run_dir=$request.run_dir
        timing='Sampled native wall-time execution, not a simulator step API or complete timing trace; internal dt is unknown.'
    }
    Write-OracleJson (Join-Path $request.run_dir 'remote-report.json') $report
    $report | ConvertTo-Json -Depth 30 -Compress
}

function Invoke-OfficialReplayRecovery([string]$RequestPath) {
    $script:OracleRequest = Get-Content -LiteralPath $RequestPath -Raw | ConvertFrom-Json
    $request = $script:OracleRequest
    $script:OracleRoot = $request.run_dir
    $script:OracleTarget = Get-Content -LiteralPath (Join-Path $request.run_dir 'owned.json') -Raw | ConvertFrom-Json
    $process = Get-Process -Id $script:OracleTarget.pid -ErrorAction SilentlyContinue
    if (!$process) {
        $result = @{status='already_exited';pid=$script:OracleTarget.pid;attacher=(Stop-OracleAttacher)}
        Write-OracleJson (Join-Path $request.run_dir 'recovery.json') $result
        return $result
    }
    [void](Assert-OracleIdentity)
    if ($request.observer_mode -eq 'native-win32') {
        Initialize-OracleNative
        $cleanup = Stop-OracleNativeObserved
        $cleanup.attacher = Stop-OracleAttacher
        Write-OracleJson (Join-Path $request.run_dir 'recovery.json') $cleanup
        return $cleanup
    }
    . $request.ux_backend
    Initialize-OracleNative
    $uxRequest = @{
        schema_version=1;target=@{pid=$process.Id;expected_exe_sha256=$request.application.sha256}
        artifact_dir=(Join-Path $request.run_dir ('ux-recovery-'+[Guid]::NewGuid().ToString('N')))
        steps=@(@{op='status'})
    }
    $ux = Invoke-LoxoneUx -RequestJson ($uxRequest | ConvertTo-Json -Depth 10 -Compress) | ConvertFrom-Json
    if (!$ux.ok) { throw 'Recovery ownership initialization failed.' }
    $cleanup = Stop-OracleOwned
    $cleanup.attacher = Stop-OracleAttacher
    Write-OracleJson (Join-Path $request.run_dir 'recovery.json') $cleanup
    return $cleanup
}
