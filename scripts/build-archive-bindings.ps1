param(
    [Parameter(Mandatory)][string]$Maturin,
    [Parameter(Mandatory)][string]$JavaGenerator,
    [Parameter(Mandatory)][string]$JavaHome,
    [Parameter(Mandatory)][string]$KotlinCompiler,
    [string]$VcVars,
    [string]$Python = 'python',
    [string]$OutputDirectory = 'target\archive-bindings',
    [string]$CargoTargetDirectory
)

$ErrorActionPreference = 'Stop'
if (-not [OperatingSystem]::IsWindows() -or -not [Environment]::Is64BitProcess) {
    throw 'This artifact qualification script currently supports Windows AMD64 only.'
}
$Repository = Split-Path $PSScriptRoot -Parent
Set-Location $Repository
if (Test-Path $OutputDirectory) { throw 'Use a fresh output directory to preserve previous evidence.' }
$Output = (New-Item -ItemType Directory $OutputDirectory).FullName
$Steps = [System.Collections.Generic.List[object]]::new()

Add-Type @'
using System;
using System.Text;
using System.Runtime.InteropServices;
public class LoxBindingBuildPath {
    [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
    public static extern uint GetShortPathNameW(string path, StringBuilder result, uint length);
}
'@
function Short-Path([string]$Path) {
    $buffer = [System.Text.StringBuilder]::new(4096)
    if ([LoxBindingBuildPath]::GetShortPathNameW($Path, $buffer, 4096) -eq 0) {
        throw "Cannot shorten $Path; use a short local build/output directory."
    }
    $buffer.ToString()
}
function Step([string]$Name, [string]$Executable, [string[]]$Arguments) {
    $start = [DateTime]::UtcNow.ToString('o')
    & $Executable @Arguments *> "$Output\$Name.log"
    $code = $LASTEXITCODE
    $Steps.Add([ordered]@{
        name = $Name; executable = $Executable; arguments = $Arguments
        started_utc = $start; ended_utc = [DateTime]::UtcNow.ToString('o'); exit_code = $code
    })
    $Steps | ConvertTo-Json -Depth 6 | Set-Content "$Output\commands.json" -Encoding utf8
    Get-Content "$Output\$Name.log" -Tail 12
    if ($code -ne 0) { throw "$Name failed ($code); see $Output\$Name.log" }
}

$Output = Short-Path $Output
if (-not $CargoTargetDirectory) { $CargoTargetDirectory = "$Output\target" }
New-Item -ItemType Directory -Force $CargoTargetDirectory, "$Output\temp", "$Output\tools" | Out-Null
$env:CARGO_TARGET_DIR = Short-Path (Resolve-Path $CargoTargetDirectory).Path
$env:TEMP = Short-Path "$Output\temp"
$env:TMP = $env:TEMP
Remove-Item Env:RUST_MIN_STACK -ErrorAction SilentlyContinue
$env:CARGO_BUILD_JOBS = '2'
$env:LOX_TELEMETRY = '0'
$env:CARGO_NET_OFFLINE = 'true'
$env:JAVA_HOME = (Resolve-Path $JavaHome).Path
$env:PATH = "$Output\tools;$env:JAVA_HOME\bin;$env:PATH"
$Java = "$env:JAVA_HOME\bin\java.exe"
$Javac = "$env:JAVA_HOME\bin\javac.exe"
$Jar = "$env:JAVA_HOME\bin\jar.exe"
$Maturin = (Resolve-Path $Maturin).Path
$JavaGenerator = (Resolve-Path $JavaGenerator).Path
$KotlinCompiler = Short-Path (Resolve-Path $KotlinCompiler).Path
Set-Location (Short-Path $Repository)

Step 'maturin-version' $Maturin @('--version')
Step 'java-generator-version' $JavaGenerator @('--version')
if ((Get-Content -Raw "$Output\maturin-version.log") -notmatch '\b1\.16\.0\b' -or
    (Get-Content -Raw "$Output\java-generator-version.log") -notmatch '\b0\.5\.2\b') {
    throw 'Expected maturin 1.16.0 and uniffi-bindgen-java 0.5.2.'
}
Step 'rust-version' 'rustc' @('-Vv')
Step 'java-version' $Java @('-version')
Step 'kotlin-version' $KotlinCompiler @('-version')
Step 'python-version' $Python @('-c', 'import platform,sys,sysconfig; print(sys.version, platform.machine(), "GIL-disabled=", sysconfig.get_config_var("Py_GIL_DISABLED")); assert sys.platform == "win32" and platform.machine() == "AMD64" and not sysconfig.get_config_var("Py_GIL_DISABLED")')
Step 'control-core-focused' 'cargo' @('test', '-p', 'lox-core', '--test', 'control_resolution', '--locked', '--offline')
Step 'numeric-state-core-focused' 'cargo' @('test', '-p', 'lox-core', '--test', 'numeric_state_payload', '--locked', '--offline')
Step 'text-state-core-focused' 'cargo' @('test', '-p', 'lox-core', '--test', 'text_state_payload', '--locked', '--offline')
Step 'device-inventory-core-focused' 'cargo' @('test', '-p', 'lox-core', '--features', 'configuration-xml', '--test', 'device_inventory', '--locked', '--offline')
Step 'rust-focused' 'cargo' @('test', '-p', 'lox-bindings', '--locked', '--offline')
Step 'generator-build' 'cargo' @('build', '-p', 'lox-bindings', '--bin', 'lox-uniffi-bindgen', '--features', 'bindgen', '--release', '--locked', '--offline')
# maturin's workspace-target discovery otherwise invokes the root package's features.
Copy-Item -Force "$env:CARGO_TARGET_DIR\release\lox-uniffi-bindgen.exe" "$Output\tools\uniffi-bindgen.exe"
Step 'wheel-build' $Maturin @('build', '--release', '--locked', '--offline', '--out', "$Output\wheels")
$Wheels = @(Get-ChildItem "$Output\wheels\*.whl")
if ($Wheels.Count -ne 1) { throw 'Expected exactly one native wheel.' }
Step 'venv-create' $Python @('-m', 'venv', "$Output\venv")
$VenvPython = "$Output\venv\Scripts\python.exe"
Step 'wheel-install' $VenvPython @('-m', 'pip', '--disable-pip-version-check', 'install', '--no-index', '--no-deps', $Wheels[0].FullName)
Step 'python-smoke' $VenvPython @('-I', 'tests\bindings\archive_smoke.py', '--fixtures', "$Output\fixtures")
Step 'python-control-smoke' $VenvPython @('-I', '-B', 'tests\bindings\control_smoke.py')
Step 'python-config-inspection-smoke' $VenvPython @('-I', '-B', 'tests\bindings\config_inspection_smoke.py')
$InstalledDll = "$Output\venv\Lib\site-packages\lox_archive\lox_archive.dll"
Step 'java-generate' $JavaGenerator @('generate', '--no-format', '--out-dir', "$Output\generated-java", $InstalledDll)
Step 'java-correction-guards' $Python @('-I', '-B', 'tests\bindings\test_java_generator_correction.py', '--generated-dir', "$Output\generated-java", '--scratch', "$Output\correction-guard-tests")
Step 'java-generator-correction' $Python @('-I', '-B', 'scripts\patch-archive-java-generator.py', '--generated-dir', "$Output\generated-java", '--generator-version', '0.5.2', '--receipt', "$Output\java-generator-correction.json")

New-Item -ItemType Directory -Force "$Output\jvm\classes", "$Output\jvm\test-classes", "$Output\jvm\native", "$Output\jvm\metadata\META-INF\licenses", "$Output\jvm\metadata\META-INF\lox-build" | Out-Null
foreach ($file in @('LICENSE', 'LICENSE-AGPL', 'LICENSE-COMMERCIAL', 'NOTICE')) {
    Copy-Item -Force $file "$Output\jvm\metadata\META-INF\licenses\$file"
}
Copy-Item -Force 'crates\lox-bindings\THIRD_PARTY_NOTICES' "$Output\jvm\metadata\META-INF\licenses"
Copy-Item -Force 'crates\lox-bindings\licenses\MPL-2.0.txt' "$Output\jvm\metadata\META-INF\licenses"
Copy-Item -Force 'crates\lox-bindings\patches\uniffi-bindgen-java-0.5.2-rustbuffer-free.json' "$Output\jvm\metadata\META-INF\lox-build\PATCH-SPEC.json"
Copy-Item -Force "$Output\java-generator-correction.json" "$Output\jvm\metadata\META-INF\lox-build\PATCH-RECEIPT.json"
$Sources = Get-ChildItem "$Output\generated-java", 'bindings\java' -Recurse -Filter '*.java' |
    ForEach-Object { '"' + $_.FullName.Replace('\', '\\') + '"' }
[IO.File]::WriteAllLines("$Output\jvm\sources.args", [string[]]$Sources, [Text.UTF8Encoding]::new($false))
Step 'javac-api' $Javac @('--release', '22', '-d', "$Output\jvm\classes", "@$Output\jvm\sources.args")
$ApiJar = "$Output\jvm\lox-archive-0.1.0.jar"
Step 'jar-api' $Jar @('--create', '--file', $ApiJar, '-C', "$Output\jvm\classes", '.', '-C', "$Output\jvm\metadata", '.')
Step 'jar-sources' $Jar @('--create', '--file', "$Output\jvm\lox-archive-0.1.0-sources.jar", '-C', "$Output\generated-java", '.', '-C', 'bindings\java', '.', '-C', "$Output\jvm\metadata", '.')
Copy-Item -Force $InstalledDll "$Output\jvm\native\lox_archive.dll"
if ((Get-FileHash $InstalledDll).Hash -ne (Get-FileHash "$Output\jvm\native\lox_archive.dll").Hash) {
    throw 'Python and JVM native artifacts differ.'
}
Step 'javac-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\ArchiveSmoke.java', 'tests\bindings\NativeArchiveReleaseProbe.java', 'tests\bindings\ArchiveAllocationSmoke.java', 'tests\bindings\NativeClockStatusProbe.java', 'tests\bindings\ClockTestSupport.java', 'tests\bindings\ClockSmoke.java', 'tests\bindings\ControlSmoke.java', 'tests\bindings\ConfigInspectionSmoke.java', 'tests\bindings\NativeConfigInspectionProbe.java', 'tests\bindings\XmlControlsSmoke.java', 'tests\bindings\ColorSmoke.java', 'tests\bindings\NativeColorProbe.java')
Step 'javac-catalog-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\CatalogSmoke.java', 'tests\bindings\NativeCatalogProbe.java')
$NativeAccess = @('--enable-native-access=ALL-UNNAMED', "-Djava.library.path=$Output\jvm\native")
Step 'java-smoke' $Java ($NativeAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'ArchiveSmoke', "$Output\fixtures"))
Step 'java-control-smoke' $Java ($NativeAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'ControlSmoke'))
$VsWhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
if (-not $VcVars) {
    if (-not (Test-Path $VsWhere)) { throw 'Pass -VcVars pointing to an existing vcvars64.bat.' }
    $VsInstall = & $VsWhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if ($LASTEXITCODE -ne 0 -or -not $VsInstall) { throw 'Existing MSVC x64 build tools not found.' }
    $VcVars = Join-Path $VsInstall 'VC\Auxiliary\Build\vcvars64.bat'
}
if (Test-Path $VsWhere) { $env:PATH = "$(Split-Path $VsWhere);$env:PATH" }
$VcVars = (Resolve-Path $VcVars).Path
New-Item -ItemType Directory -Force "$Output\jvm\test-native" | Out-Null
$Probe = "$Output\jvm\test-native\windows_heap_probe"
$NativeCommand = 'call "' + $VcVars + '" >nul && cl /nologo /W4 /WX /O2 /LD /MT /Fo"' + $Probe + '.obj" /Fe"' + $Probe + '.dll" tests\bindings\windows_heap_probe.c /link /IMPLIB:"' + $Probe + '.lib"'
Step 'native-heap-probe-build' $env:ComSpec @('/d', '/c', $NativeCommand)
$AllocationAccess = $NativeAccess + @("-Dlox.archive.heapProbe=$Probe.dll")
Step 'python-xml-controls-smoke' $VenvPython @('-I', '-B', 'tests\bindings\xml_controls_smoke.py', "$Probe.dll")
Step 'java-xml-controls-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'XmlControlsSmoke', 'crates\lox-bindings\tests\fixtures\xml_controls.xml'))
Step 'python-catalog-smoke' $VenvPython @('-I', '-B', 'tests\bindings\catalog_smoke.py', "$Probe.dll")
Step 'java-catalog-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'CatalogSmoke', 'crates\lox-bindings\tests\fixtures\control_catalog.json'))
Step 'python-devices-smoke' $VenvPython @('-I', '-B', 'tests\bindings\devices_smoke.py', "$Probe.dll")
Step 'javac-devices-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\DevicesSmoke.java', 'tests\bindings\NativeDeviceProbe.java')
Step 'java-devices-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'DevicesSmoke', 'crates\lox-bindings\tests\fixtures\xml_devices.xml'))
Step 'python-color-smoke' $VenvPython @('-I', '-B', 'tests\bindings\color_smoke.py', "$Probe.dll", 'crates\lox-bindings\tests\fixtures\color_profiles.tsv')
Step 'java-color-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'net.eisber.lox.archive.ColorSmoke', 'crates\lox-bindings\tests\fixtures\color_profiles.tsv'))
Step 'javac-numeric-state-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\NumericStateSmoke.java', 'tests\bindings\NativeNumericStateProbe.java')
Step 'python-numeric-state-smoke' $VenvPython @('-I', '-B', 'tests\bindings\numeric_state_smoke.py', "$Probe.dll")
Step 'java-numeric-state-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'NumericStateSmoke'))
Step 'javac-text-state-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\TextStateSmoke.java', 'tests\bindings\NativeTextStateProbe.java')
Step 'python-text-state-smoke' $VenvPython @('-I', '-B', 'tests\bindings\text_state_smoke.py', "$Probe.dll")
Step 'java-text-state-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'TextStateSmoke'))
Step 'java-allocation-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'ArchiveAllocationSmoke', "$Output\fixtures"))
Step 'java-config-inspection-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'ConfigInspectionSmoke', 'crates\lox-bindings\tests\fixtures\config_inspection.xml'))
Step 'kotlinc-smoke' $KotlinCompiler @('-jvm-target', '22', '-classpath', $ApiJar, '-include-runtime', '-d', "$Output\jvm\kotlin-smoke.jar", 'tests\bindings\ArchiveSmoke.kt')
Step 'kotlin-smoke' $Java ($NativeAccess + @('-cp', "$ApiJar;$Output\jvm\kotlin-smoke.jar", 'ArchiveSmokeKt', "$Output\fixtures"))
Step 'kotlinc-control-smoke' $KotlinCompiler @('-jvm-target', '22', '-classpath', $ApiJar, '-include-runtime', '-d', "$Output\jvm\kotlin-control-smoke.jar", 'tests\bindings\ControlSmoke.kt')
Step 'kotlin-control-smoke' $Java ($NativeAccess + @('-cp', "$ApiJar;$Output\jvm\kotlin-control-smoke.jar", 'ControlSmokeKt'))
Step 'jar-allocation-test-support' $Jar @('--create', '--file', "$Output\jvm\allocation-test-support.jar", '-C', "$Output\jvm\test-classes", '.')
$DeviceKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\allocation-test-support.jar".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-devices-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\DevicesSmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-devices.args", $DeviceKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-devices-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-devices.args")
Step 'kotlin-devices-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-devices-smoke.jar", 'DevicesSmokeKt', 'crates\lox-bindings\tests\fixtures\xml_devices.xml'))
$ColorKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\allocation-test-support.jar".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-color-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\ColorSmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-color.args", $ColorKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-color-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-color.args")
Step 'kotlin-color-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-color-smoke.jar", 'ColorSmokeKt', 'crates\lox-bindings\tests\fixtures\color_profiles.tsv'))
$ConfigKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\allocation-test-support.jar".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-config-inspection-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\ConfigInspectionSmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-config-inspection.args", $ConfigKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-config-inspection-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-config-inspection.args")
Step 'kotlin-config-inspection-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-config-inspection-smoke.jar", 'ConfigInspectionSmokeKt', 'crates\lox-bindings\tests\fixtures\config_inspection.xml'))
$ControlsKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\allocation-test-support.jar".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-xml-controls-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\XmlControlsSmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-xml-controls.args", $ControlsKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-xml-controls-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-xml-controls.args")
Step 'kotlin-xml-controls-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-xml-controls-smoke.jar", 'XmlControlsSmokeKt', 'crates\lox-bindings\tests\fixtures\xml_controls.xml'))
$CatalogKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\allocation-test-support.jar".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-catalog-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\CatalogSmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-catalog.args", $CatalogKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-catalog-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-catalog.args")
Step 'kotlin-catalog-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-catalog-smoke.jar", 'CatalogSmokeKt', 'crates\lox-bindings\tests\fixtures\control_catalog.json'))
$NumericKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\allocation-test-support.jar".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-numeric-state-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\NumericStateSmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-numeric-state.args", $NumericKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-numeric-state-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-numeric-state.args")
Step 'kotlin-numeric-state-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-numeric-state-smoke.jar", 'NumericStateSmokeKt'))
$TextKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\allocation-test-support.jar".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-text-state-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\TextStateSmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-text-state.args", $TextKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-text-state-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-text-state.args")
Step 'kotlin-text-state-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-text-state-smoke.jar", 'TextStateSmokeKt'))
$KotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\allocation-test-support.jar".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-allocation-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\ArchiveAllocationSmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-allocation.args", $KotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-allocation-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-allocation.args")
Step 'kotlin-allocation-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-allocation-smoke.jar", 'ArchiveAllocationSmokeKt', "$Output\fixtures"))
$ClockKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\allocation-test-support.jar".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-clock-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\ClockSmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-clock.args", $ClockKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-clock-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-clock.args")
Step 'clock-head-smokes' $VenvPython @('-I', '-B', 'tests\bindings\clock_smoke.py', '--signals', "$Output\clock-signals",
    '--java', $Java, '--api-jar', $ApiJar, '--classes', "$Output\jvm\test-classes",
    '--kotlin-jar', "$Output\jvm\kotlin-clock-smoke.jar", '--native-dir', "$Output\jvm\native",
    '--heap-probe', "$Probe.dll")

Step 'python-config-diff-smoke' $VenvPython @('-I', '-B', 'tests\bindings\config_diff_smoke.py', "$Probe.dll")
Step 'javac-config-diff-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\ConfigDiffSmoke.java', 'tests\bindings\NativeConfigDiffProbe.java')
Step 'java-config-diff-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'ConfigDiffSmoke'))
$DiffKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\test-classes".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-config-diff-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\ConfigDiffSmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-config-diff.args", $DiffKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-config-diff-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-config-diff.args")
Step 'kotlin-config-diff-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-config-diff-smoke.jar", 'ConfigDiffSmokeKt'))

$Artifacts = @(Get-ChildItem "$Output\wheels", "$Output\generated-java", "$Output\jvm", "$Output\fixtures" -Recurse -File)
$Artifacts += Get-Item $InstalledDll, "$Output\venv\Lib\site-packages\lox_archive\lox_archive.py", "$Output\tools\uniffi-bindgen.exe"
$Artifacts += Get-Item "$Output\venv\Lib\site-packages\lox_archive_async.py"
$Artifacts += Get-Item "$Output\java-generator-correction.json"
$Artifacts += Get-Item "$Output\CLOCK-PROOF.json", "$Output\clock-java.log", "$Output\clock-kotlin.log"
$Artifacts += Get-Item "$Output\python-control-smoke.log", "$Output\java-control-smoke.log", "$Output\kotlin-control-smoke.log"
$Artifacts += Get-Item "$Output\python-config-inspection-smoke.log", "$Output\java-config-inspection-smoke.log", "$Output\kotlin-config-inspection-smoke.log"
$Artifacts += Get-Item "$Output\python-xml-controls-smoke.log", "$Output\java-xml-controls-smoke.log", "$Output\kotlin-xml-controls-smoke.log"
$Artifacts += Get-Item "$Output\python-catalog-smoke.log", "$Output\java-catalog-smoke.log", "$Output\kotlin-catalog-smoke.log"
$Artifacts += Get-Item "$Output\python-color-smoke.log", "$Output\java-color-smoke.log", "$Output\kotlin-color-smoke.log"
$Artifacts += Get-Item "$Output\python-numeric-state-smoke.log", "$Output\java-numeric-state-smoke.log", "$Output\kotlin-numeric-state-smoke.log"
$Artifacts += Get-Item "$Output\python-text-state-smoke.log", "$Output\java-text-state-smoke.log", "$Output\kotlin-text-state-smoke.log"
$Artifacts += Get-Item "$Output\python-devices-smoke.log", "$Output\java-devices-smoke.log", "$Output\kotlin-devices-smoke.log"
$Artifacts += Get-Item "$Output\python-config-diff-smoke.log", "$Output\java-config-diff-smoke.log", "$Output\kotlin-config-diff-smoke.log"
$Manifest = foreach ($file in $Artifacts) {
    [ordered]@{
        path = [IO.Path]::GetRelativePath($Output, $file.FullName)
        bytes = $file.Length; sha256 = (Get-FileHash $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    }
}
[ordered]@{
    schema_version = 1; completed_utc = [DateTime]::UtcNow.ToString('o')
    source_head = (git rev-parse HEAD); source_index_tree = (git write-tree)
    source_status = @(git --no-pager status --short)
    platform = 'Windows AMD64 only; see actual version logs'
    contract = 'Archive, raw-byte read-only XML device/control/room/summary inspection, typed supplied-summary structural comparison delegated to Core, stateless typed LoxApp3 catalog parsing/filtering, payload-only numeric state decoding with exact UUID/IEEE bits, strict payload-only text state decoding with raw/native35 state/icon UUIDs and exact UTF8 including NUL, strict production HTTP clock, pure typed control selection, and offline typed color encoding/explicit legacy observations. Current loopback clock cancellation/close only; no Lighting eligibility, full XML/semantic diff/editor/auth/control I/O/statistics/subscriptions/full-runtime/platform parity. Candidate composition remains publication-gated.'
    native_rust_min_stack = $env:RUST_MIN_STACK
    generators = @{ uniffi = '0.32.1'; java = '0.5.2'; maturin = '1.16.0' }
    artifact_manifest = $Manifest
    excluded = @('target', 'temp', 'venv except tested native DLL/generated module/async adapter')
} | ConvertTo-Json -Depth 8 | Set-Content "$Output\ARTIFACTS.json" -Encoding utf8
Write-Output "Archive artifact proof complete: $Output\ARTIFACTS.json"
