$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes, System.Windows.Forms, System.Drawing

if (!('LoxUxNative' -as [type])) {
    Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class LoxUxNative {
    public delegate bool EnumProc(IntPtr h, IntPtr p);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc callback, IntPtr p);
    [DllImport("user32.dll")] public static extern bool EnumChildWindows(IntPtr h, EnumProc callback, IntPtr p);
    [DllImport("user32.dll")] public static extern int GetDlgCtrlID(IntPtr h);
    [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetClassName(IntPtr h,StringBuilder name,int size);
    [DllImport("user32.dll",EntryPoint="GetWindowLongPtrW")] static extern IntPtr GetWindowLongPtr(IntPtr h,int index);
    [DllImport("user32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern IntPtr SendMessageTimeout(IntPtr h,uint message,UIntPtr size,StringBuilder text,uint flags,uint timeout,out UIntPtr result);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] static extern bool BringWindowToTop(IntPtr h);
    [DllImport("user32.dll")] static extern bool IsIconic(IntPtr h);
    [DllImport("user32.dll")] static extern bool ShowWindow(IntPtr h, int state);
    [DllImport("kernel32.dll")] static extern uint GetCurrentThreadId();
    [DllImport("user32.dll")] static extern bool AttachThreadInput(uint from, uint to, bool attach);
    [DllImport("user32.dll")] static extern bool SetWindowPos(IntPtr h,IntPtr after,int x,int y,int w,int height,uint flags);
    [StructLayout(LayoutKind.Sequential)] struct Message { public IntPtr h; public uint id; public UIntPtr wp; public IntPtr lp; public uint time; public Point point; }
    [DllImport("user32.dll")] static extern bool PeekMessage(out Message message,IntPtr h,uint min,uint max,uint flags);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(Point p);
    [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint x, uint y, uint d, UIntPtr e);
    [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
    [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint flags);
    [StructLayout(LayoutKind.Sequential)] public struct Point { public int X, Y; public Point(int x,int y) { X=x; Y=y; } }
    [StructLayout(LayoutKind.Sequential)] struct Keyboard { public ushort vk, scan; public uint flags, time; public UIntPtr extra; }
    [StructLayout(LayoutKind.Sequential)] struct Mouse { public int x,y; public uint data,flags,time; public UIntPtr extra; }
    [StructLayout(LayoutKind.Explicit)] struct Union { [FieldOffset(0)] public Keyboard keyboard; [FieldOffset(0)] public Mouse mouse; }
    [StructLayout(LayoutKind.Sequential)] struct Input { public uint type; public Union data; }
    [DllImport("user32.dll", SetLastError=true)] static extern uint SendInput(uint count, Input[] input, int size);
    public static bool Activate(IntPtr handle) {
        uint ignored;
        uint current = GetCurrentThreadId();
        uint foreground = GetWindowThreadProcessId(GetForegroundWindow(), out ignored);
        uint target = GetWindowThreadProcessId(handle, out ignored);
        Message message;
        PeekMessage(out message, IntPtr.Zero, 0, 0, 0);
        bool attachedForeground = false, attachedTarget = false;
        try {
            if (IsIconic(handle)) ShowWindow(handle, 9);
            if (foreground != 0 && foreground != current)
                attachedForeground = AttachThreadInput(current, foreground, true);
            if (target != 0 && target != current && target != foreground)
                attachedTarget = AttachThreadInput(current, target, true);
            SetForegroundWindow(handle);
            BringWindowToTop(handle);
            SetWindowPos(handle, IntPtr.Zero, 0, 0, 0, 0, 0x43);
            return GetForegroundWindow() == handle;
        } finally {
            if (attachedTarget) AttachThreadInput(current, target, false);
            if (attachedForeground) AttachThreadInput(current, foreground, false);
        }
    }
    public static string ReadEdit(IntPtr handle) {
        if ((GetWindowLongPtr(handle,-16).ToInt64() & 0x20) != 0)
            throw new InvalidOperationException("Password edit controls are not supported.");
        var text = new StringBuilder(8193);
        UIntPtr result;
        if (SendMessageTimeout(handle,13,(UIntPtr)(uint)text.Capacity,text,2,2000,out result) == IntPtr.Zero)
            throw new InvalidOperationException("The native filename edit did not respond.");
        return text.ToString();
    }
    public static void TypeLiteral(string text) {
        foreach (char c in text) {
            var input = new Input[2];
            input[0].type = input[1].type = 1;
            input[0].data.keyboard.scan = input[1].data.keyboard.scan = c;
            input[0].data.keyboard.flags = 4;
            input[1].data.keyboard.flags = 6;
            if (SendInput(2, input, Marshal.SizeOf(typeof(Input))) != 2)
                throw new InvalidOperationException("Unicode keyboard input was rejected.");
        }
    }
}
'@
}

function Stop-Ux([string]$Code, [string]$Message) {
    $failure = [InvalidOperationException]::new($Message)
    $failure.Data['ux_code'] = $Code
    throw $failure
}

function Get-UxField($Object, [string]$Name, $Default = $null) {
    if ($null -eq $Object) { return $Default }
    $property = $Object.PSObject.Properties[$Name]
    if ($null -eq $property) { return $Default }
    $property.Value
}

function Assert-UxFields($Object, [string[]]$Allowed) {
    if ($null -eq $Object -or $Object -isnot [pscustomobject]) {
        Stop-Ux 'invalid_request' 'Expected a JSON object.'
    }
    foreach ($property in $Object.PSObject.Properties) {
        if ($property.Name -notin $Allowed) {
            Stop-Ux 'invalid_request' "Unknown field: $($property.Name)."
        }
    }
}

function Assert-UxInteger($Value, [long]$Minimum, [long]$Maximum, [string]$Label) {
    if (($Value -isnot [int] -and $Value -isnot [long]) -or $Value -lt $Minimum -or $Value -gt $Maximum) {
        Stop-Ux 'invalid_request' "$Label must be an integer in $Minimum..$Maximum."
    }
}

function Get-UxWindows {
    $script:UxHandles = [Collections.Generic.List[IntPtr]]::new()
    $callback = [LoxUxNative+EnumProc]{
        param($handle, $unused)
        [uint32]$owner = 0
        [void][LoxUxNative]::GetWindowThreadProcessId($handle, [ref]$owner)
        if ($owner -eq $script:UxProcessId -and [LoxUxNative]::IsWindowVisible($handle)) {
            $script:UxHandles.Add($handle)
        }
        return $true
    }
    [void][LoxUxNative]::EnumWindows($callback, [IntPtr]::Zero)
    foreach ($handle in $script:UxHandles) {
        [Windows.Automation.AutomationElement]::FromHandle($handle)
    }
}

function Assert-UxOwned($Element) {
    if ($Element.Current.ProcessId -ne $script:UxProcessId) {
        Stop-Ux 'foreign_process' 'The selected element does not belong to the requested process.'
    }
}

function Assert-UxForeground {
    if ($script:UxProcess.HasExited) { Stop-Ux 'target_exited' 'The owned process exited; no further input is allowed.' }
    [uint32]$owner = 0
    [void][LoxUxNative]::GetWindowThreadProcessId([LoxUxNative]::GetForegroundWindow(), [ref]$owner)
    if ($owner -ne $script:UxProcessId) {
        Stop-Ux 'foreground_changed' 'Another process owns the foreground; input was stopped.'
    }
}

function Find-UxElements($Root, $Selector) {
    if ($null -eq $Selector) { $Root; return }
    Assert-UxFields $Selector @('name', 'automation_id', 'automation_id_suffix', 'control_type')
    if (!$Selector.PSObject.Properties.Count) { Stop-Ux 'invalid_request' 'An empty selector is ambiguous.' }
    $queue = [Collections.Generic.Queue[object]]::new()
    $queue.Enqueue(@($Root, 0))
    $walker = [Windows.Automation.TreeWalker]::ControlViewWalker
    $visited = 0
    while ($queue.Count) {
        $entry = $queue.Dequeue()
        $element = $entry[0]
        $depth = $entry[1]
        $script:UxNodes++
        $visited++
        if ($visited -gt 10000) { Stop-Ux 'uia_budget' 'Owned UI traversal exceeded its node budget.' }
        $current = $element.Current
        if ($current.ProcessId -ne $script:UxProcessId) { continue }
        $matches = $true
        foreach ($property in $Selector.PSObject.Properties) {
            if ($property.Value -isnot [string] -or !$property.Value) {
                Stop-Ux 'invalid_request' 'Selectors require nonempty strings.'
            }
            switch ($property.Name) {
                'name' { if ($current.Name -cne $property.Value) { $matches = $false } }
                'automation_id' { if ($current.AutomationId -cne $property.Value) { $matches = $false } }
                'automation_id_suffix' { if (!$current.AutomationId.EndsWith($property.Value, [StringComparison]::Ordinal)) { $matches = $false } }
                'control_type' {
                    if ($current.ControlType.ProgrammaticName -cne $property.Value -and
                        $current.ControlType.ProgrammaticName -cne "ControlType.$($property.Value)") { $matches = $false }
                }
            }
        }
        if ($matches) { $element }
        if ($depth -ge 20) { continue }
        $child = $walker.GetFirstChild($element)
        while ($child) {
            $queue.Enqueue(@($child, ($depth + 1)))
            $child = $walker.GetNextSibling($child)
        }
    }
}

function Get-UxWindow([string]$Class, [switch]$AllowMissing) {
    $windows = @(Get-UxWindows)
    if ($Class) {
        $direct = @($windows | Where-Object { $_.Current.ClassName -ceq $Class })
        if ($direct.Count -eq 1) { return $direct[0] }
        $found = foreach ($window in $windows) {
            Find-UxElements $window ([pscustomobject]@{control_type='Window'}) |
                Where-Object { $_.Current.ClassName -ceq $Class }
        }
        $windows = @($found | Group-Object { $_.Current.NativeWindowHandle } | ForEach-Object { $_.Group[0] })
    } else {
        $foreground = [LoxUxNative]::GetForegroundWindow()
        $active = @($windows | Where-Object { $_.Current.NativeWindowHandle -eq $foreground.ToInt64() })
        if ($active.Count -eq 1) { return $active[0] }
        $windows = @($windows | Where-Object { $_.Current.IsEnabled })
    }
    if ($windows.Count -eq 0) {
        if ($AllowMissing) { return $null }
        Stop-Ux 'window_missing' 'No matching owned window is available.'
    }
    if ($windows.Count -ne 1) {
        Stop-Ux 'window_ambiguous' "Expected one owned window, found $($windows.Count); specify window_class."
    }
    $windows[0]
}

function Get-UxElement($Step) {
    $class = Get-UxField $Step 'window_class'
    $selector = Get-UxField $Step 'selector'
    $roots = if ($class -or !$selector) { @(Get-UxWindow $class) } else { @(Get-UxWindows) }
    $seen = [Collections.Generic.HashSet[string]]::new()
    $elements = @(foreach ($root in $roots) {
        foreach ($element in @(Find-UxElements $root $selector)) {
            if ($seen.Add(($element.GetRuntimeId() -join ','))) { $element }
        }
    })
    if ($elements.Count -ne 1) {
        Stop-Ux 'selector_ambiguous' "Expected one owned element, found $($elements.Count)."
    }
    $elements[0]
}

function Focus-Ux($Element) {
    Assert-UxOwned $Element
    $handle = [IntPtr]$Element.Current.NativeWindowHandle
    if ($handle -eq [IntPtr]::Zero) { Stop-Ux 'focus_failed' 'The selected window has no native handle.' }
    if ([LoxUxNative]::GetForegroundWindow() -eq $handle) { return }
    $until = [DateTime]::UtcNow.AddSeconds(2)
    do {
        [void][LoxUxNative]::Activate($handle)
        Start-Sleep -Milliseconds 80
        [uint32]$owner = 0
        [void][LoxUxNative]::GetWindowThreadProcessId([LoxUxNative]::GetForegroundWindow(), [ref]$owner)
        if ($owner -eq $script:UxProcessId -and [LoxUxNative]::GetForegroundWindow() -eq $handle) { return }
    } while ([DateTime]::UtcNow -lt $until)
    Stop-Ux 'focus_failed' 'The owned process could not obtain the foreground.'
}

function Get-UxHitElement([int]$X, [int]$Y) {
    [Windows.Automation.AutomationElement]::FromPoint([Windows.Point]::new($X,$Y))
}

function Get-UxParent($Element) {
    [Windows.Automation.TreeWalker]::ControlViewWalker.GetParent($Element)
}

function Assert-UxHit($Element, [int]$X, [int]$Y) {
    $wanted = $Element.GetRuntimeId() -join ','
    $hit = Get-UxHitElement $X $Y
    for ($depth=0; $hit -and $depth -lt 40; $depth++) {
        if ($hit.Current.ProcessId -ne $script:UxProcessId) { break }
        if (($hit.GetRuntimeId() -join ',') -ceq $wanted) { return }
        $hit = Get-UxParent $hit
    }
    Stop-Ux 'occluded' 'The hit-tested control is not the selected element or its descendant.'
}

function Get-UxForegroundHandle {
    [LoxUxNative]::GetForegroundWindow()
}

function Assert-UxWindowForeground($Window) {
    Assert-UxForeground
    $handle = [IntPtr]$Window.Current.NativeWindowHandle
    if ($handle -eq [IntPtr]::Zero -or (Get-UxForegroundHandle) -ne $handle) {
        Stop-Ux 'foreground_changed' 'The requested window is not the foreground window; keys were refused.'
    }
}

function Click-Ux($Element, [switch]$Double) {
    Assert-UxOwned $Element
    Assert-UxForeground
    if (!$Element.Current.IsEnabled -or $Element.Current.IsOffscreen) {
        Stop-Ux 'element_unavailable' 'The selected control is disabled or off-screen.'
    }
    if ($Element.Current.Name -match '^(?i:Connect to Miniserver|LIVE|Transfer|Upload|Reboot|Restart Miniserver|Write to Miniserver)$') {
        Stop-Ux 'online_action_refused' 'This driver only supports offline project work.'
    }
    $rectangle = $Element.Current.BoundingRectangle
    if ($rectangle.IsEmpty -or $rectangle.Width -le 0 -or $rectangle.Height -le 0) {
        Stop-Ux 'empty_rectangle' 'The control has no usable click rectangle.'
    }
    $x = [int]($rectangle.Left + $rectangle.Width / 2)
    $y = [int]($rectangle.Top + $rectangle.Height / 2)
    [uint32]$owner = 0
    [void][LoxUxNative]::GetWindowThreadProcessId([LoxUxNative]::WindowFromPoint([LoxUxNative+Point]::new($x,$y)), [ref]$owner)
    if ($owner -ne $script:UxProcessId) { Stop-Ux 'occluded' 'Another process occludes the click point.' }
    Assert-UxHit $Element $x $y
    [void][LoxUxNative]::SetCursorPos($x,$y)
    Assert-UxForeground
    Assert-UxHit $Element $x $y
    foreach ($click in 1..$(if ($Double) { 2 } else { 1 })) {
        [LoxUxNative]::mouse_event(2,0,0,0,[UIntPtr]::Zero)
        [LoxUxNative]::mouse_event(4,0,0,0,[UIntPtr]::Zero)
        if ($Double) { Start-Sleep -Milliseconds 80 }
    }
    $script:UxInputs++
}

function Send-UxKeys([string]$Value) {
    Assert-UxForeground
    $focused = [Windows.Automation.AutomationElement]::FocusedElement
    if ($focused -and $focused.Current.IsPassword) { Stop-Ux 'protected_input' 'Password controls are not supported.' }
    [Windows.Forms.SendKeys]::SendWait($Value)
    $script:UxInputs++
}

function Write-UxLiteral([string]$Value) {
    Assert-UxForeground
    [LoxUxNative]::TypeLiteral($Value)
    $script:UxInputs++
}

function Read-UxElement($Element) {
    Assert-UxOwned $Element
    $current = $Element.Current
    $value = $null
    $pattern = $null
    $valueSupported = $false
    if ($current.IsPassword) { $value = '<protected>' }
    elseif ($Element.TryGetCurrentPattern([Windows.Automation.ValuePattern]::Pattern, [ref]$pattern)) {
        $value = $pattern.Current.Value
        $valueSupported = $true
    }
    $truncated = $false
    if ($value -and $value.Length -gt 256) { $value = $value.Substring(0,256); $truncated = $true }
    [ordered]@{
        name=$current.Name
        class=$current.ClassName
        automation_id=$current.AutomationId
        control_type=$current.ControlType.ProgrammaticName
        enabled=$current.IsEnabled
        offscreen=$current.IsOffscreen
        value=$value
        value_supported=$valueSupported
        value_truncated=$truncated
        rect=@($current.BoundingRectangle.Left,$current.BoundingRectangle.Top,$current.BoundingRectangle.Right,$current.BoundingRectangle.Bottom)
    }
}

function Get-UxValue($Element) {
    Assert-UxOwned $Element
    if ($Element.Current.IsPassword) { Stop-Ux 'protected_input' 'Password controls are not supported.' }
    $pattern = $null
    if (!$Element.TryGetCurrentPattern([Windows.Automation.ValuePattern]::Pattern,[ref]$pattern)) {
        Stop-Ux 'value_pattern_unavailable' 'The selected control does not expose a value.'
    }
    [string]$pattern.Current.Value
}

function Get-UxDialogControl($Dialog, [int[]]$Ids, [string]$Class) {
    Assert-UxOwned $Dialog
    $script:UxNativeControls = [Collections.Generic.List[IntPtr]]::new()
    $script:UxNativeIds = $Ids
    $script:UxNativeClass = $Class
    $callback = [LoxUxNative+EnumProc]{
        param($handle,$unused)
        $name = [Text.StringBuilder]::new(256)
        [void][LoxUxNative]::GetClassName($handle,$name,256)
        if ([LoxUxNative]::GetDlgCtrlID($handle) -in $script:UxNativeIds -and
            $name.ToString() -ceq $script:UxNativeClass -and [LoxUxNative]::IsWindowVisible($handle)) {
            $script:UxNativeControls.Add($handle)
        }
        return $true
    }
    [void][LoxUxNative]::EnumChildWindows([IntPtr]$Dialog.Current.NativeWindowHandle,$callback,[IntPtr]::Zero)
    if ($script:UxNativeControls.Count -ne 1) {
        Stop-Ux 'selector_ambiguous' "Expected one visible native $Class control, found $($script:UxNativeControls.Count)."
    }
    $element = [Windows.Automation.AutomationElement]::FromHandle($script:UxNativeControls[0])
    Assert-UxOwned $element
    $element
}

function Get-UxEnergyCell([int]$Row, [int]$Column) {
    if ($script:UxTarget.exe_sha256 -ne '7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3') {
        Stop-Ux 'unsupported_energy_build' 'Energy table recipes are pinned to Config 17.2.10.21; recover a new binding before replaying another build.'
    }
    Assert-UxInteger $Row 1 12 'Energy row'
    Assert-UxInteger $Column 0 10 'Energy column'
    $window = Get-UxWindow 'CEnergyManagerConfigurationDlg'
    $tables = @(Find-UxElements $window ([pscustomobject]@{control_type='Table'}) |
        Where-Object { $_.Current.ClassName -eq 'GenericView::CGenericViewTable' })
    if ($tables.Count -ne 1) { Stop-Ux 'energy_shape_changed' 'The Energy table is not unique.' }
    $grid = $null
    if (!$tables[0].TryGetCurrentPattern([Windows.Automation.GridPattern]::Pattern,[ref]$grid) -or
        $grid.Current.RowCount -ne 12 -or $grid.Current.ColumnCount -ne 11) {
        Stop-Ux 'energy_shape_changed' 'Expected a native GridPattern with 12 rows and 11 columns.'
    }
    $labels = @{
        0=@('Priority',('Priorit'+[char]0xe4+'t')); 1=@('Output','Ausgang')
        2=@('Type','Typ'); 3=@('Icon','Symbol'); 4=@('Name','Bezeichnung')
        5=@('Preparation time [s]','Vorbereitungszeit [s]')
        6=@('Minimum run-time [min]','Mindestlaufzeit [min]')
        7=@('Minimum run-time / day [min]','Mindestlaufzeit / Tag [min]')
        8=@('Min run-time/day until...','Mindestlaufzeit/Tag bis...')
        9=@('Switch-on power [kW]','Einschaltleistung [kW]')
        10=@('Rated power [kW]','Nennleistung [kW]')
    }
    $headers = @(Find-UxElements $tables[0] ([pscustomobject]@{control_type='Header'}))
    if ($headers.Count -ne 11 -or $headers[$Column].Current.Name -cnotin $labels[$Column]) {
        Stop-Ux 'energy_columns_changed' 'The requested column no longer matches the verified English/German header binding.'
    }
    $cell = $grid.GetItem($Row - 1, $Column)
    Assert-UxOwned $cell
    $item = $null
    if (!$cell.TryGetCurrentPattern([Windows.Automation.GridItemPattern]::Pattern,[ref]$item) -or
        $item.Current.Row -ne ($Row - 1) -or $item.Current.Column -ne $Column) {
        Stop-Ux 'energy_shape_changed' 'The returned grid cell does not match the requested row and column.'
    }
    $cell
}

function Get-UxSettingGroup($Element) {
    $parent = $Element
    for ($depth=0; $parent -and $depth -lt 12; $depth++) {
        if ($parent.Current.ClassName -eq 'CCollapseableContainer') {
            $headers = @(Find-UxElements $parent ([pscustomobject]@{automation_id_suffix='.Header.Default'}))
            if ($headers.Count -eq 1) { return $headers[0].Current.Name }
            return $null
        }
        $parent = Get-UxParent $parent
    }
}

function Open-UxEnergy {
    if ($script:UxTarget.exe_sha256 -ne '7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3') {
        Stop-Ux 'unsupported_energy_build' 'Recover the Energy dialog binding before replaying another Config build.'
    }
    $open = @(Get-UxWindows | Where-Object { $_.Current.ClassName -eq 'CEnergyManagerConfigurationDlg' })
    if ($open.Count -eq 1) { return @{already_open=$true} }
    $main = Get-UxWindow 'CLxMainWindow'
    Focus-Ux $main
    $bars = @(Find-UxElements $main ([pscustomobject]@{automation_id_suffix='.QPropWindowScrollArea.qt_scrollarea_vcontainer.QScrollBar'}))
    if ($bars.Count -gt 1) { Stop-Ux 'selector_ambiguous' 'Property scroll bar is not unique.' }
    if ($bars.Count -eq 1) {
        $range = $null
        if (!$bars[0].TryGetCurrentPattern([Windows.Automation.RangeValuePattern]::Pattern,[ref]$range) -or $range.Current.IsReadOnly) {
            Stop-Ux 'scroll_unavailable' 'The property pane does not expose a writable scroll range.'
        }
        Assert-UxForeground
        $range.SetValue($range.Current.Maximum)
        $script:UxInputs++
    }
    $until = [DateTime]::UtcNow.AddSeconds(3)
    do {
        $buttons = @(Find-UxElements $main ([pscustomobject]@{automation_id_suffix='.CSettingEditorButton.QPushButton'}) |
            Where-Object { (Get-UxSettingGroup $_) -cin @('Settings','Einstellungen') })
        if ($buttons.Count -ne 1) { Stop-Ux 'selector_ambiguous' 'Select an Energy Manager with one supported Settings configuration button.' }
        $rectangle = $buttons[0].Current.BoundingRectangle
        if ($main.Current.BoundingRectangle.Contains($rectangle)) { break }
        Start-Sleep -Milliseconds 100
    } while ([DateTime]::UtcNow -lt $until)
    if (!$main.Current.BoundingRectangle.Contains($rectangle)) { Stop-Ux 'element_unavailable' 'The Energy configuration button remains outside its owned window.' }
    Click-Ux $buttons[0]
    $until = [DateTime]::UtcNow.AddSeconds(5)
    do {
        $dialog = Get-UxWindow 'CEnergyManagerConfigurationDlg' -AllowMissing
        if ($dialog) { return @{opened=$true;window_class='CEnergyManagerConfigurationDlg'} }
        Start-Sleep -Milliseconds 100
    } while ([DateTime]::UtcNow -lt $until)
    Stop-Ux 'wait_timeout' 'The selected block did not open the expected Energy configuration dialog.'
}

function Save-UxFailureCapture([string]$Directory) {
    [uint32]$owner = 0
    $handle = [LoxUxNative]::GetForegroundWindow()
    [void][LoxUxNative]::GetWindowThreadProcessId($handle, [ref]$owner)
    if ($owner -ne $script:UxProcessId) { return @{capture_skipped='Foreground is not owned.'} }
    $window = [Windows.Automation.AutomationElement]::FromHandle($handle)
    $rectangle = $window.Current.BoundingRectangle
    if ($rectangle.Width -le 0 -or $rectangle.Height -le 0 -or $rectangle.Width * $rectangle.Height -gt 25000000) {
        return @{capture_skipped='Invalid or oversized owned window.'}
    }
    $bitmap = [Drawing.Bitmap]::new([int]$rectangle.Width,[int]$rectangle.Height)
    $graphics = [Drawing.Graphics]::FromImage($bitmap)
    $dc = $graphics.GetHdc()
    try { $captured = [LoxUxNative]::PrintWindow($handle,$dc,2) }
    finally { $graphics.ReleaseHdc($dc); $graphics.Dispose() }
    try {
        if (!$captured) { return @{capture_skipped='PrintWindow failed; no desktop fallback was used.'} }
        $path = Join-Path $Directory 'failure.png'
        $bitmap.Save($path,[Drawing.Imaging.ImageFormat]::Png)
        return @{failure_screenshot=$path;capture_backend='PrintWindow'}
    } finally { $bitmap.Dispose() }
}

function Invoke-UxStep($Step) {
    $op = Get-UxField $Step 'op'
    switch ($op) {
        'status' { return $script:UxTarget }
        'windows' { return ,@((Get-UxWindows) | ForEach-Object { Read-UxElement $_ }) }
        'inspect' {
            $limit = Get-UxField $Step 'limit' 12
            $maximumDepth = Get-UxField $Step 'depth' 2
            Assert-UxInteger $limit 1 50 'Inspection limit'
            Assert-UxInteger $maximumDepth 0 5 'Inspection depth'
            $queue = [Collections.Generic.Queue[object]]::new()
            $selector = Get-UxField $Step 'selector'
            if ($selector) {
                $roots = if (Get-UxField $Step 'window_class') { @(Get-UxWindow $Step.window_class) } else { @(Get-UxWindows) }
                foreach ($root in $roots) {
                    foreach ($element in @(Find-UxElements $root $selector)) { $queue.Enqueue(@($element,0)) }
                }
            } else {
                $queue.Enqueue(@((Get-UxWindow (Get-UxField $Step 'window_class')),0))
            }
            $items = [Collections.Generic.List[object]]::new()
            $seen = [Collections.Generic.HashSet[string]]::new()
            $walker = [Windows.Automation.TreeWalker]::ControlViewWalker
            while ($queue.Count -and $items.Count -lt $limit) {
                $entry = $queue.Dequeue()
                $element = $entry[0]
                if ($element.Current.ProcessId -ne $script:UxProcessId) { continue }
                if (!$seen.Add(($element.GetRuntimeId() -join ','))) { continue }
                $script:UxNodes++
                $items.Add((Read-UxElement $element))
                if ($entry[1] -ge $maximumDepth) { continue }
                $child = $walker.GetFirstChild($element)
                while ($child) { $queue.Enqueue(@($child,($entry[1]+1))); $child=$walker.GetNextSibling($child) }
            }
            return @{elements=@($items.ToArray());truncated=($queue.Count -gt 0)}
        }
        'wait' {
            $timeout = Get-UxField $Step 'timeout_ms' 5000
            Assert-UxInteger $timeout 1 30000 'Wait timeout'
            if (!(Get-UxField $Step 'selector')) { Stop-Ux 'invalid_request' 'Wait requires a selector.' }
            $until = [DateTime]::UtcNow.AddMilliseconds($timeout)
            do {
                $window = Get-UxWindow (Get-UxField $Step 'window_class') -AllowMissing
                $found = @(if ($window) { Find-UxElements $window $Step.selector })
                if ($found.Count -gt 1) { Stop-Ux 'selector_ambiguous' 'Wait matched multiple controls.' }
                if ($found.Count -eq 1 -and $found[0].Current.IsEnabled) {
                    if ($null -eq $Step.PSObject.Properties['value'] -or (Get-UxValue $found[0]) -ceq $Step.value) {
                        return (Read-UxElement $found[0])
                    }
                }
                Start-Sleep -Milliseconds 100
            } while ([DateTime]::UtcNow -lt $until)
            Stop-Ux 'wait_timeout' 'The selected control did not become ready.'
        }
        'focus' {
            $selected = if (Get-UxField $Step 'selector') { Get-UxElement $Step } else { Get-UxWindow (Get-UxField $Step 'window_class') }
            $window = $selected
            while ($window -and ($window.Current.ControlType -ne [Windows.Automation.ControlType]::Window -or !$window.Current.NativeWindowHandle)) {
                $window = [Windows.Automation.TreeWalker]::ControlViewWalker.GetParent($window)
            }
            if (!$window) { Stop-Ux 'focus_failed' 'No native owned parent window was found.' }
            Focus-Ux $window
            if ($selected.Current.ControlType -ne [Windows.Automation.ControlType]::Window) { $selected.SetFocus(); Assert-UxForeground }
            return @{focused=$true}
        }
        'invoke' {
            Click-Ux (Get-UxElement $Step)
            return @{invoked=$true}
        }
        'set_value' {
            $element = Get-UxElement $Step
            Assert-UxForeground
            if ($element.Current.IsPassword -or !$element.Current.IsEnabled) { Stop-Ux 'element_unavailable' 'The input is protected or disabled.' }
            $pattern = $null
            if (!$element.TryGetCurrentPattern([Windows.Automation.ValuePattern]::Pattern,[ref]$pattern) -or $pattern.Current.IsReadOnly) {
                Stop-Ux 'value_pattern_unavailable' 'The control has no writable ValuePattern; use an explicit guarded key recipe.'
            }
            $pattern.SetValue([string]$Step.value)
            $script:UxInputs++
            return (Read-UxElement $element)
        }
        'keys' {
            if (Get-UxField $Step 'selector') { Click-Ux (Get-UxElement $Step) }
            if (Get-UxField $Step 'window_class') { Assert-UxWindowForeground (Get-UxWindow $Step.window_class) }
            Send-UxKeys ([string]$Step.value)
            return @{sent=$true}
        }
        'energy.open' { return (Open-UxEnergy) }
        { $_ -in @('energy.read','energy.set','energy.select') } {
            Assert-UxInteger $Step.row 1 12 'Energy row'
            Assert-UxInteger $Step.column 0 10 'Energy column'
            $cell = Get-UxEnergyCell $Step.row $Step.column
            if ($op -ne 'energy.read') {
                if ($op -eq 'energy.set' -and $Step.column -notin @(4,5,6,7,9,10)) {
                    Stop-Ux 'unsupported_energy_column' 'Text edits support name, preparation, runtimes and power columns only.'
                }
                if ($op -eq 'energy.select') {
                    $maximum = if ($Step.column -eq 2) { 1 } elseif ($Step.column -eq 8) { 27 } else { -1 }
                    Assert-UxInteger $Step.index 0 $maximum 'Energy selection'
                }
                Click-Ux $cell -Double
                if ($op -eq 'energy.set') {
                    Send-UxKeys '^a{BACKSPACE}'
                    Write-UxLiteral ([string]$Step.value)
                } else {
                    Send-UxKeys ('{HOME}' + ('{DOWN}' * $Step.index))
                }
                Send-UxKeys '{ENTER}'
            }
            $display = (Get-UxEnergyCell $Step.row $Step.column).Current.Name
            return @{row=$Step.row;column=$Step.column;display=$display;authoritative_storage=$false;note='Qt cell labels may lag; verify the GUI-saved project with the native CLI.'}
        }
        { $_ -in @('energy.accept','energy.cancel') } {
            [void](Get-UxEnergyCell 1 0)
            $names = if ($op -eq 'energy.accept') { @('OK') } else { @('Cancel','Abbrechen') }
            $dialog = Get-UxWindow 'CEnergyManagerConfigurationDlg'
            $buttons = @(Find-UxElements $dialog ([pscustomobject]@{control_type='Button'}) | Where-Object { $_.Current.Name -cin $names })
            if ($buttons.Count -ne 1) { Stop-Ux 'selector_ambiguous' 'The supported dialog action is not unique.' }
            $clickedName = $buttons[0].Current.Name
            Click-Ux $buttons[0]
            $until = [DateTime]::UtcNow.AddSeconds(5)
            do {
                if (!(Get-UxWindow 'CEnergyManagerConfigurationDlg' -AllowMissing)) { return @{clicked=$clickedName;closed=$true} }
                Start-Sleep -Milliseconds 100
            } while ([DateTime]::UtcNow -lt $until)
            Stop-Ux 'dialog_not_closed' 'The Energy dialog remains open; subsequent input was stopped.'
        }
        'save_as' {
            $path = [string]$Step.path
            if ($path -notmatch '^[A-Za-z]:\\' -or $path -match '[\r\n*?"<>|]' -or $path -match '\\\.\.(\\|$)') {
                Stop-Ux 'invalid_path' 'Save As requires an absolute Windows drive path without traversal or wildcards.'
            }
            if (Test-Path -LiteralPath $path) { Stop-Ux 'output_exists' 'Refusing to overwrite an existing oracle output.' }
            if (!(Test-Path -LiteralPath ([IO.Path]::GetDirectoryName($path)) -PathType Container)) {
                Stop-Ux 'missing_parent' 'Create the explicit output directory before Save As.'
            }
            Assert-UxWindowForeground (Get-UxWindow 'CLxMainWindow')
            Send-UxKeys '^+s'
            $until = [DateTime]::UtcNow.AddSeconds(5)
            $dialog = $null
            do {
                $found = @(Get-UxWindows | Where-Object { $_.Current.ClassName -eq '#32770' })
                if ($found.Count -eq 1) { $dialog=$found[0]; break }
                if ($found.Count -gt 1) { Stop-Ux 'window_ambiguous' 'Multiple common dialogs appeared.' }
                Start-Sleep -Milliseconds 100
            } while ([DateTime]::UtcNow -lt $until)
            if (!$dialog) { Stop-Ux 'wait_timeout' 'Save As did not open an owned common dialog.' }
            $edit = Get-UxDialogControl $dialog @(1001,1148) 'Edit'
            Click-Ux $edit
            Send-UxKeys '^a{BACKSPACE}'
            Write-UxLiteral $path
            $until=[DateTime]::UtcNow.AddSeconds(3)
            do {
                $filename = [LoxUxNative]::ReadEdit([IntPtr]$edit.Current.NativeWindowHandle)
                if ($filename -ceq $path) { break }
                Start-Sleep -Milliseconds 100
            } while ([DateTime]::UtcNow -lt $until)
            if ($filename -cne $path) {
                Stop-Ux 'filename_mismatch' 'The dialog did not retain the exact requested path; Save was refused.'
            }
            if (Test-Path -LiteralPath $path) { Stop-Ux 'output_exists' 'The output appeared while the dialog was open.' }
            $save = Get-UxDialogControl $dialog @(1) 'Button'
            Click-Ux $save
            $until=[DateTime]::UtcNow.AddSeconds(10)
            do {
                if (Test-Path -LiteralPath $path -PathType Leaf) {
                    $stream=$null
                    try { $stream=[IO.File]::Open($path,[IO.FileMode]::Open,[IO.FileAccess]::Read,[IO.FileShare]::None) }
                    catch [IO.IOException] { Start-Sleep -Milliseconds 100; continue }
                    try {
                        if ($stream.Length -eq 0) { Stop-Ux 'empty_output' 'The saved output is empty.' }
                        $hash=[Security.Cryptography.SHA256]::Create()
                        try { $digest=[BitConverter]::ToString($hash.ComputeHash($stream)).Replace('-','').ToLowerInvariant() }
                        finally { $hash.Dispose() }
                    } finally { $stream.Dispose() }
                    return @{path=$path;sha256=$digest;filename_backend='native-common-dialog';storage_verified=$false}
                }
                Start-Sleep -Milliseconds 100
            } while ([DateTime]::UtcNow -lt $until)
            Stop-Ux 'save_timeout' 'Save As did not produce a closed, readable output file.'
        }
        default { Stop-Ux 'unknown_operation' "Unsupported operation: $op." }
    }
}

function Invoke-LoxoneUx {
    param([Parameter(Mandatory=$true)][string]$RequestJson)
    $clock=[Diagnostics.Stopwatch]::StartNew()
    $script:UxProcessId=0
    $script:UxNodes=0
    $script:UxInputs=0
    $script:UxTarget=@{}
    $script:UxIdentityVerified=$false
    $script:UxProcess=$null
    $completed=0
    $failed=$null
    $errorData=$null
    $results=[Collections.Generic.List[object]]::new()
    $artifacts=@{}
    $directory=$null
    $steps=@()
    try {
        if ($RequestJson.Length -gt 262144) { Stop-Ux 'invalid_request' 'Request is too large.' }
        $request=$RequestJson | ConvertFrom-Json
        Assert-UxFields $request @('schema_version','target','artifact_dir','steps')
        Assert-UxInteger $request.schema_version 1 1 'Request schema'
        Assert-UxFields $request.target @('pid','expected_exe_sha256')
        Assert-UxInteger $request.target.pid 1 2147483647 'Owned PID'
        $script:UxProcessId=[int]$request.target.pid
        if ($request.steps -isnot [Array]) { Stop-Ux 'invalid_request' 'Steps must be a JSON array.' }
        $steps=@($request.steps)
        if ($steps.Count -lt 1 -or $steps.Count -gt 64) { Stop-Ux 'invalid_request' 'Provide 1..64 steps.' }
        $readOps=@('status','windows','inspect','energy.read')
        $allowedOps=@($readOps)+@('focus','invoke','set_value','keys','wait','energy.open','energy.set','energy.select','energy.accept','energy.cancel','save_as')
        foreach ($step in $steps) {
            $fields = switch ($step.op) {
                { $_ -in @('status','windows','energy.open','energy.accept','energy.cancel') } { @('op') }
                'inspect' { @('op','window_class','selector','limit','depth') }
                { $_ -in @('focus','invoke') } { @('op','window_class','selector') }
                { $_ -in @('set_value','keys') } { @('op','window_class','selector','value') }
                'wait' { @('op','window_class','selector','timeout_ms','value') }
                'energy.read' { @('op','row','column') }
                'energy.set' { @('op','row','column','value') }
                'energy.select' { @('op','row','column','index') }
                'save_as' { @('op','path') }
                default { @('op') }
            }
            Assert-UxFields $step $fields
            if ($step.op -notin $allowedOps) { Stop-Ux 'unknown_operation' 'The request contains an unsupported operation.' }
            if ($step.op -in @('set_value','keys','energy.set')) {
                if ($step.value -isnot [string] -or $step.value.Length -gt 8192 -or $step.value.Contains([char]0)) {
                    Stop-Ux 'invalid_request' 'Input values must be bounded strings without NUL.'
                }
                if ($step.op -eq 'wait' -and $null -ne $step.PSObject.Properties['value']) {
                    if ($step.value -isnot [string] -or $step.value.Length -gt 8192 -or $step.value.Contains([char]0)) {
                        Stop-Ux 'invalid_request' 'Wait values must be bounded strings without NUL.'
                    }
                }
            }
        }
        $pin=Get-UxField $request.target 'expected_exe_sha256'
        if ($null -ne $pin -and ($pin -isnot [string] -or $pin -notmatch '^[a-fA-F0-9]{64}$')) { Stop-Ux 'invalid_request' 'Executable SHA-256 pin is malformed.' }
        if (@($steps | Where-Object {$_.op -notin $readOps}).Count -and !$pin) {
            Stop-Ux 'version_pin_required' 'Input steps require an executable SHA-256 pin.'
        }
        $directory=[string]$request.artifact_dir
        if ($directory -notmatch '^[A-Za-z]:\\' -or $directory -match '[\r\n*?"<>|]' -or $directory -match '\\\.\.(\\|$)') {
            Stop-Ux 'invalid_path' 'Use an absolute, traversal-free artifact directory.'
        }
        if (Test-Path -LiteralPath $directory) { $directory=$null; Stop-Ux 'artifact_exists' 'An existing run directory cannot be overwritten.' }
        New-Item -ItemType Directory -Path $directory | Out-Null
        $artifacts=@{actions=(Join-Path $directory 'actions.jsonl');report=(Join-Path $directory 'report.json')}
        [IO.File]::WriteAllText($artifacts.actions,'',[Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText((Join-Path $directory 'request.json'),$RequestJson,[Text.UTF8Encoding]::new($false))
        $process=Get-Process -Id $script:UxProcessId -ErrorAction Stop
        [void]$process.Handle
        $script:UxProcess=$process
        if ($process.SessionId -ne [Diagnostics.Process]::GetCurrentProcess().SessionId) {
            Stop-Ux 'foreign_session' 'The target is not in this interactive desktop session.'
        }
        $image=Get-Item -LiteralPath $process.Path
        $script:UxTarget=@{exe_sha256=(Get-FileHash -LiteralPath $image.FullName -Algorithm SHA256).Hash.ToLowerInvariant();version=$image.VersionInfo.FileVersion}
        if ($pin -and $script:UxTarget.exe_sha256 -cne $pin.ToLowerInvariant()) {
            Stop-Ux 'version_mismatch' 'The executable does not match the requested pin; no GUI action was performed.'
        }
        $script:UxIdentityVerified=[bool]$pin
        [void][LoxUxNative]::SetThreadDpiAwarenessContext([IntPtr](-4))
        for ($index=0;$index -lt $steps.Count;$index++) {
            $failed=$index
            $step=$steps[$index]
            $value=Invoke-UxStep $step
            $entry=@{index=$index;op=$step.op;value=$value}
            $results.Add($entry)
            [IO.File]::AppendAllText($artifacts.actions, (($entry | ConvertTo-Json -Depth 15 -Compress)+"`n"), [Text.UTF8Encoding]::new($false))
            $completed++
        }
        $failed=$null
    } catch {
        $exception=$_.Exception
        while ($exception.InnerException -and !$exception.Data['ux_code']) { $exception=$exception.InnerException }
        $code=if($exception.Data['ux_code']){[string]$exception.Data['ux_code']}else{'ux_error'}
        $errorData=@{code=$code;message=$exception.Message}
        if ($script:UxIdentityVerified -and $directory -and (Test-Path -LiteralPath $directory -PathType Container)) {
            try {
                $capture=Save-UxFailureCapture $directory
                foreach($key in $capture.Keys){$artifacts[$key]=$capture[$key]}
            } catch { $artifacts.capture_error=$_.Exception.Message }
        } elseif ($directory) {
            $artifacts.capture_skipped='Executable identity is not pinned and verified.'
        }
    }
    $reply=[ordered]@{
        schema_version=1
        ok=($null -eq $errorData)
        pid=$script:UxProcessId
        target=$script:UxTarget
        completed_steps=$completed
        failed_step=$failed
        error=$errorData
        results=@($results.ToArray())
        elapsed_ms=$clock.Elapsed.TotalMilliseconds
        artifacts=$artifacts
        metrics=@{steps_requested=$steps.Count;uia_nodes_seen=$script:UxNodes;input_actions=$script:UxInputs}
    }
    $json=$reply | ConvertTo-Json -Depth 20 -Compress
    if ($directory -and $artifacts.report) {
        [IO.File]::WriteAllText($artifacts.report,$json,[Text.UTF8Encoding]::new($false))
    }
    [regex]::Replace($json,'[^\u0000-\u007f]',{param($match) '\u{0:x4}' -f [int][char]$match.Value})
}
