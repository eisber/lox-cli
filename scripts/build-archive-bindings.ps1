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
Step 'custom-listing-core-focused' 'cargo' @('test', '-p', 'lox-core', '--test', 'custom_listing', '--locked', '--offline')
Step 'runtime-overlay-core-focused' 'cargo' @('test', '-p', 'lox-core', '--features', 'configuration-xml', '--test', 'runtime_overlay', '--locked', '--offline')
Step 'statistics-identity-core-focused' 'cargo' @('test', '-p', 'lox-core', '--features', 'statistics-xml', '--test', 'statistics_series', 'identity_', '--locked', '--offline')
Step 'statistics-identity-cli-focused' 'cargo' @('test', '--test', 'statistics_series_cli', 'identity_', '--locked', '--offline')
Step 'numeric-state-core-focused' 'cargo' @('test', '-p', 'lox-core', '--test', 'numeric_state_payload', '--locked', '--offline')
Step 'text-state-core-focused' 'cargo' @('test', '-p', 'lox-core', '--test', 'text_state_payload', '--locked', '--offline')
Step 'daytimer-state-core-focused' 'cargo' @('test', '-p', 'lox-core', '--test', 'daytimer_state_payload', '--locked', '--offline')
Step 'weather-state-core-focused' 'cargo' @('test', '-p', 'lox-core', '--test', 'weather_state_payload', '--locked', '--offline')
Step 'statistics-series-core-focused' 'cargo' @('test', '-p', 'lox-core', '--features', 'statistics-xml', '--test', 'statistics_series', '--locked', '--offline')
Step 'device-inventory-core-focused' 'cargo' @('test', '-p', 'lox-core', '--features', 'configuration-xml', '--test', 'device_inventory', '--locked', '--offline')
Step 'user-inventory-core-focused' 'cargo' @('test', '-p', 'lox-core', '--features', 'configuration-xml', '--test', 'user_inventory', '--locked', '--offline')
Step 'program-text-core-focused' 'cargo' @('test', '-p', 'lox-core', '--features', 'configuration-editor', '--test', 'configuration_editor', 'program_reader_', '--locked', '--offline')
Step 'mood-inventory-core-focused' 'cargo' @('test', '-p', 'lox-core', '--features', 'configuration-editor', '--test', 'configuration_editor', 'mood_inventory_', '--locked', '--offline')
Step 'status-monitor-core-focused' 'cargo' @('test', '-p', 'lox-core', '--features', 'configuration-editor', '--test', 'configuration_editor', 'status_monitor_inspection_', '--locked', '--offline')
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
Step 'python-runtime-overlay-smoke' $VenvPython @('-I', '-B', 'tests\bindings\runtime_overlay_smoke.py', "$Probe.dll")
Step 'javac-runtime-overlay-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\RuntimeOverlaySmoke.java', 'tests\bindings\NativeRuntimeOverlayProbe.java')
Step 'java-runtime-overlay-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'RuntimeOverlaySmoke'))
$OverlayKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\test-classes".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-runtime-overlay-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\RuntimeOverlaySmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-runtime-overlay.args", $OverlayKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-runtime-overlay-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-runtime-overlay.args")
Step 'kotlin-runtime-overlay-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-runtime-overlay-smoke.jar", 'RuntimeOverlaySmokeKt'))
@{
    actual_all_three_established_utc = [DateTime]::UtcNow.ToString('o')
    dll_sha256 = (Get-FileHash $InstalledDll -Algorithm SHA256).Hash.ToLowerInvariant()
    proof = 'Actual three-way success/error RustBuffers live1/free0,64 calls per branch per language retained0 without requested GC; Core-only selected raw bytes and typed ordered MergedConsistency then DuplicateUuid{id} cause frames, typed native parser/UTF8 boundaries. Python and compiledJava22/Kotlin SAME API. No acquisition/schema/baseline/deployment claims.'
} | ConvertTo-Json | Set-Content "$Output\RUNTIME-OVERLAY-MECHANISM.json" -Encoding utf8
Step 'python-custom-listing-smoke' $VenvPython @('-I', '-B', 'tests\bindings\custom_listing_smoke.py', "$Probe.dll")
Step 'javac-custom-listing-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\CustomListingSmoke.java', 'tests\bindings\NativeCustomListingProbe.java')
Step 'java-custom-listing-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'CustomListingSmoke'))
$CustomListingKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\test-classes".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-custom-listing-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\CustomListingSmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-custom-listing.args", $CustomListingKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-custom-listing-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-custom-listing.args")
Step 'kotlin-custom-listing-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-custom-listing-smoke.jar", 'CustomListingSmokeKt'))
@{
    actual_all_three_established_utc = [DateTime]::UtcNow.ToString('o')
    dll_sha256 = (Get-FileHash $InstalledDll -Algorithm SHA256).Hash.ToLowerInvariant()
    proof = 'Actual custom-listing success/Malformed/Unsupported RustBuffers live1/free0,64 calls per branch per language retained0 without requested GC; shared Core successful-listing parser, raw sorted names/duplicates/legacy quirks and atomic data-free zero-based row errors. Python and compiledJava22/Kotlin SAME API; no acquisition or fragment-filename validation.'
} | ConvertTo-Json | Set-Content "$Output\CUSTOM-LISTING-MECHANISM.json" -Encoding utf8
Step 'python-statistics-identity-smoke' $VenvPython @('-I', '-B', 'tests\bindings\statistics_identity_smoke.py', "$Probe.dll")
Step 'javac-statistics-identity-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\StatisticsIdentitySmoke.java', 'tests\bindings\NativeStatisticsIdentityProbe.java')
Step 'java-statistics-identity-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'StatisticsIdentitySmoke'))
$StatisticsIdentityKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\test-classes".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-statistics-identity-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\StatisticsIdentitySmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-statistics-identity.args", $StatisticsIdentityKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-statistics-identity-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-statistics-identity.args")
Step 'kotlin-statistics-identity-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-statistics-identity-smoke.jar", 'StatisticsIdentitySmokeKt'))
@{
    actual_all_three_established_utc = [DateTime]::UtcNow.ToString('o')
    dll_sha256 = (Get-FileHash $InstalledDll -Algorithm SHA256).Hash.ToLowerInvariant()
    proof = 'Actual statistics-identity String success/NotFound/Ambiguous RustBuffers live1/free0,64 calls per branch per language retained0 without requested GC; shared Core raw-UUID/exact-name/substring precedence, raw selected UUID, sorted/deduplicated owned typed diagnostics and distinct raw UUID ambiguity count. Existing StatEntry input; Python and compiledJava22/Kotlin SAME API. No index/series parsing, group/month filtering or acquisition in this facade.'
} | ConvertTo-Json | Set-Content "$Output\STATISTICS-IDENTITY-MECHANISM.json" -Encoding utf8
Step 'python-daytimer-state-smoke' $VenvPython @('-I', '-B', 'tests\bindings\daytimer_state_smoke.py', "$Probe.dll")
Step 'javac-daytimer-state-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\DaytimerStateSmoke.java', 'tests\bindings\NativeDaytimerStateProbe.java')
Step 'java-daytimer-state-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'DaytimerStateSmoke'))
$DaytimerKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\test-classes".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-daytimer-state-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\DaytimerStateSmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-daytimer-state.args", $DaytimerKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-daytimer-state-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-daytimer-state.args")
Step 'kotlin-daytimer-state-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-daytimer-state-smoke.jar", 'DaytimerStateSmokeKt'))
@{
    actual_all_three_established_utc = [DateTime]::UtcNow.ToString('o')
    dll_sha256 = (Get-FileHash $InstalledDll -Algorithm SHA256).Hash.ToLowerInvariant()
    proof = 'Actual Daytimer success/error RustBuffers live1/free0,64 calls per branch per language retained0 without requested GC; Core payload decoder, typed signed fields, authoritative u64 bits, raw/native35 UUIDs, order/duplicates and atomic errors. Python and compiledJava22/Kotlin SAME API.'
} | ConvertTo-Json | Set-Content "$Output\DAYTIMER-STATE-MECHANISM.json" -Encoding utf8
Step 'python-weather-state-smoke' $VenvPython @('-I', '-B', 'tests\bindings\weather_state_smoke.py', "$Probe.dll")
Step 'javac-weather-state-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\WeatherStateSmoke.java', 'tests\bindings\NativeWeatherStateProbe.java')
Step 'java-weather-state-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'WeatherStateSmoke'))
$WeatherKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\test-classes".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-weather-state-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\WeatherStateSmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-weather-state.args", $WeatherKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-weather-state-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-weather-state.args")
Step 'kotlin-weather-state-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-weather-state-smoke.jar", 'WeatherStateSmokeKt'))
@{
    actual_all_three_established_utc = [DateTime]::UtcNow.ToString('o')
    dll_sha256 = (Get-FileHash $InstalledDll -Algorithm SHA256).Hash.ToLowerInvariant()
    proof = 'Actual Weather success/error RustBuffers live1/free0,64 calls per branch per language retained0 without requested GC; Core payload decoder, unsigned last-update, signed fields, six authoritative u64 bits, raw/native35 UUIDs, order/duplicates and atomic typed errors. Python and compiledJava22/Kotlin SAME API.'
} | ConvertTo-Json | Set-Content "$Output\WEATHER-STATE-MECHANISM.json" -Encoding utf8
Step 'python-mqtt-catalog-smoke' $VenvPython @('-I', '-B', 'tests\bindings\mqtt_catalog_smoke.py', "$Probe.dll")
Step 'javac-mqtt-catalog-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\MqttCatalogSmoke.java', 'tests\bindings\NativeMqttCatalogProbe.java')
Step 'java-mqtt-catalog-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'MqttCatalogSmoke'))
$MqttKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\test-classes".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-mqtt-catalog-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\MqttCatalogSmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-mqtt-catalog.args", $MqttKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-mqtt-catalog-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-mqtt-catalog.args")
Step 'kotlin-mqtt-catalog-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-mqtt-catalog-smoke.jar", 'MqttCatalogSmokeKt'))
@{
    actual_all_three_established_utc = [DateTime]::UtcNow.ToString('o')
    dll_sha256 = (Get-FileHash $InstalledDll -Algorithm SHA256).Hash.ToLowerInvariant()
    proof = 'Actual MQTT success/load-error RustBuffers live1/free0,64 public calls per branch per language retained0 without requested GC; retained owned results/exceptions; unchanged Core shared loader including lox-n37w; Python and compiledJava22/Kotlin SAME API'
} | ConvertTo-Json | Set-Content "$Output\MQTT-CATALOG-MECHANISM.json" -Encoding utf8
Step 'python-backup-catalog-smoke' $VenvPython @('-I', '-B', 'tests\bindings\backup_catalog_smoke.py', "$Probe.dll")
Step 'javac-backup-catalog-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\BackupCatalogSmoke.java', 'tests\bindings\NativeBackupCatalogProbe.java')
Step 'java-backup-catalog-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'BackupCatalogSmoke'))
$BackupKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\test-classes".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-backup-catalog-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\BackupCatalogSmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-backup-catalog.args", $BackupKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-backup-catalog-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-backup-catalog.args")
Step 'kotlin-backup-catalog-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-backup-catalog-smoke.jar", 'BackupCatalogSmokeKt'))
@{
    original_receipt_utc = '2026-11-16T04:54:08.592Z'
    original_deadline_utc = '2026-11-16T05:14:08.592Z'
    actual_all_three_established_utc = [DateTime]::UtcNow.ToString('o')
    dll_sha256 = (Get-FileHash $InstalledDll -Algorithm SHA256).Hash.ToLowerInvariant()
    proof = 'Actual backup name Some success RustBuffer live1/free0,64 public calls per language retained0 without requested GC; unchanged matcher. None functional only, Vec inherits generated ownership; infallible domain has no error branch. Python and compiledJava22/Kotlin SAME API.'
} | ConvertTo-Json | Set-Content "$Output\BACKUP-CATALOG-MECHANISM.json" -Encoding utf8
Step 'python-firmware-metadata-smoke' $VenvPython @('-I', '-B', 'tests\bindings\firmware_metadata_smoke.py', "$Probe.dll")
Step 'javac-firmware-metadata-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\FirmwareMetadataSmoke.java', 'tests\bindings\NativeFirmwareMetadataProbe.java')
Step 'java-firmware-metadata-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'FirmwareMetadataSmoke'))
$FirmwareKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\test-classes".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-firmware-metadata-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\FirmwareMetadataSmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-firmware-metadata.args", $FirmwareKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-firmware-metadata-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-firmware-metadata.args")
Step 'kotlin-firmware-metadata-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-firmware-metadata-smoke.jar", 'FirmwareMetadataSmokeKt'))
@{
    original_receipt_utc = '2026-11-16T02:42:37.8875211Z'
    original_deadline_utc = '2026-11-16T03:02:37.8875211Z'
    actual_all_three_established_utc = [DateTime]::UtcNow.ToString('o')
    dll_sha256 = (Get-FileHash $InstalledDll -Algorithm SHA256).Hash.ToLowerInvariant()
    proof = 'Actual firmware XML export success/error RustBuffers live1/free0,64 public calls per branch per language retained0 without requested GC; Core InvalidVersion Display echo and unchanged prefix matcher; generated String lowering; Python and compiledJava22/Kotlin SAME API'
} | ConvertTo-Json | Set-Content "$Output\FIRMWARE-METADATA-MECHANISM.json" -Encoding utf8
Step 'python-statistics-index-smoke' $VenvPython @('-I', '-B', 'tests\bindings\statistics_index_smoke.py', "$Probe.dll")
Step 'javac-statistics-index-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\StatisticsIndexSmoke.java', 'tests\bindings\NativeStatisticsIndexProbe.java')
Step 'java-statistics-index-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'StatisticsIndexSmoke'))
$StatisticsIndexKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\test-classes".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-statistics-index-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\StatisticsIndexSmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-statistics-index.args", $StatisticsIndexKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-statistics-index-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-statistics-index.args")
Step 'kotlin-statistics-index-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-statistics-index-smoke.jar", 'StatisticsIndexSmokeKt'))
@{
    original_receipt_utc = '2026-11-15T21:11:59.512Z'
    original_deadline_utc = '2026-11-15T21:31:59.512Z'
    actual_all_three_established_utc = [DateTime]::UtcNow.ToString('o')
    dll_sha256 = (Get-FileHash $InstalledDll -Algorithm SHA256).Hash.ToLowerInvariant()
    proof = 'Actual infallible statistics index result RustBuffer live1/free0,64 public calls per language retained0 without requested GC; generated String lowering; Python and compiledJava22/Kotlin SAME API'
} | ConvertTo-Json | Set-Content "$Output\STATISTICS-INDEX-MECHANISM.json" -Encoding utf8
Step 'python-statistics-series-smoke' $VenvPython @('-I', '-B', 'tests\bindings\statistics_series_smoke.py', "$Probe.dll")
Step 'javac-statistics-series-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\StatisticsSeriesSmoke.java', 'tests\bindings\NativeStatisticsSeriesProbe.java')
Step 'java-statistics-series-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'StatisticsSeriesSmoke'))
$StatisticsSeriesKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\test-classes".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-statistics-series-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\StatisticsSeriesSmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-statistics-series.args", $StatisticsSeriesKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-statistics-series-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-statistics-series.args")
Step 'kotlin-statistics-series-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-statistics-series-smoke.jar", 'StatisticsSeriesSmokeKt'))
@{
    original_source_receipt_utc = '2026-11-18T03:26:01.673Z'
    actual_all_three_established_utc = [DateTime]::UtcNow.ToString('o')
    dll_sha256 = (Get-FileHash $InstalledDll -Algorithm SHA256).Hash.ToLowerInvariant()
    proof = 'Actual monthly statistics XML success/InvalidValue error RustBuffers live1/free0,64 public calls per branch per language retained0 without requested GC; retained owned rows/exceptions, Core-only parse_series, opaque timestamps, finite f64 signed-zero/subnormal/extreme/underflow bits and exact typed error causes; generated String lowering; Python and compiledJava22/Kotlin SAME API. No index/identity/merge/acquisition expansion.'
} | ConvertTo-Json | Set-Content "$Output\STATISTICS-SERIES-MECHANISM.json" -Encoding utf8
Step 'python-autopilot-catalog-smoke' $VenvPython @('-I', '-B', 'tests\bindings\autopilot_catalog_smoke.py', "$Probe.dll")
Step 'javac-autopilot-catalog-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\AutopilotCatalogSmoke.java', 'tests\bindings\NativeAutopilotProbe.java')
Step 'java-autopilot-catalog-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'AutopilotCatalogSmoke'))
$AutopilotKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\test-classes".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-autopilot-catalog-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\AutopilotCatalogSmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-autopilot-catalog.args", $AutopilotKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-autopilot-catalog-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-autopilot-catalog.args")
Step 'kotlin-autopilot-catalog-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-autopilot-catalog-smoke.jar", 'AutopilotCatalogSmokeKt'))
@{
    original_receipt_utc = '2026-11-15T19:32:55.067Z'
    original_deadline_utc = '2026-11-15T19:52:55.067Z'
    actual_all_three_established_utc = [DateTime]::UtcNow.ToString('o')
    dll_sha256 = (Get-FileHash $InstalledDll -Algorithm SHA256).Hash.ToLowerInvariant()
    proof = 'Actual Autopilot export success/error RustBuffers live1/free0,64 public calls per branch per language retained0; Python + compiledJava22 + compiledKotlin SAME API'
} | ConvertTo-Json | Set-Content "$Output\AUTOPILOT-MECHANISM.json" -Encoding utf8
Step 'python-autopilot-rule-smoke' $VenvPython @('-I', '-B', 'tests\bindings\autopilot_rule_smoke.py', "$Probe.dll")
Step 'javac-autopilot-rule-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\AutopilotRuleSmoke.java', 'tests\bindings\NativeAutopilotRuleProbe.java')
Step 'java-autopilot-rule-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'AutopilotRuleSmoke'))
$AutopilotRuleKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\test-classes".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-autopilot-rule-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\AutopilotRuleSmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-autopilot-rule.args", $AutopilotRuleKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-autopilot-rule-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-autopilot-rule.args")
Step 'kotlin-autopilot-rule-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-autopilot-rule-smoke.jar", 'AutopilotRuleSmokeKt'))
@{
    actual_all_three_established_utc = [DateTime]::UtcNow.ToString('o')
    dll_sha256 = (Get-FileHash $InstalledDll -Algorithm SHA256).Hash.ToLowerInvariant()
    proof = 'Actual supplied-row Autopilot selector success/UuidNotFound/NotFound/Ambiguous RustBuffers live1/free0,64 calls per branch per language retained0 without requested GC; Core loose raw UUID/name substring selection, owned four-field rows/query/ordered duplicate candidates. Python and compiledJava22/Kotlin SAME API; no acquisition or whole-CLI migration.'
} | ConvertTo-Json | Set-Content "$Output\AUTOPILOT-RULE-MECHANISM.json" -Encoding utf8
Step 'python-structure-metadata-smoke' $VenvPython @('-I', '-B', 'tests\bindings\structure_metadata_smoke.py', "$Probe.dll")
Step 'javac-structure-metadata-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\StructureMetadataSmoke.java', 'tests\bindings\NativeMetadataProbe.java')
Step 'java-structure-metadata-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'StructureMetadataSmoke'))
$MetadataKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\test-classes".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-structure-metadata-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\StructureMetadataSmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-structure-metadata.args", $MetadataKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-structure-metadata-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-structure-metadata.args")
Step 'kotlin-structure-metadata-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-structure-metadata-smoke.jar", 'StructureMetadataSmokeKt'))
@{
    actual_all_three_established_utc = [DateTime]::UtcNow.ToString('o')
    dll_sha256 = (Get-FileHash $InstalledDll -Algorithm SHA256).Hash.ToLowerInvariant()
    proof = 'Actual new metadata export success/error RustBuffers live1/free0,64 public calls per branch per language retained0; Python + compiledJava22 + compiledKotlin same API'
} | ConvertTo-Json | Set-Content "$Output\METADATA-MECHANISM.json" -Encoding utf8
Step 'python-xml-controls-smoke' $VenvPython @('-I', '-B', 'tests\bindings\xml_controls_smoke.py', "$Probe.dll")
Step 'java-xml-controls-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'XmlControlsSmoke', 'crates\lox-bindings\tests\fixtures\xml_controls.xml'))
Step 'python-catalog-smoke' $VenvPython @('-I', '-B', 'tests\bindings\catalog_smoke.py', "$Probe.dll")
Step 'java-catalog-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'CatalogSmoke', 'crates\lox-bindings\tests\fixtures\control_catalog.json'))
Step 'python-devices-smoke' $VenvPython @('-I', '-B', 'tests\bindings\devices_smoke.py', "$Probe.dll")
Step 'javac-devices-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\DevicesSmoke.java', 'tests\bindings\NativeDeviceProbe.java')
Step 'java-devices-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'DevicesSmoke', 'crates\lox-bindings\tests\fixtures\xml_devices.xml'))
Step 'python-users-smoke' $VenvPython @('-I', '-B', 'tests\bindings\users_smoke.py', "$Probe.dll")
Step 'javac-users-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\UsersSmoke.java', 'tests\bindings\NativeUserProbe.java')
Step 'java-users-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'UsersSmoke', 'crates\lox-bindings\tests\fixtures\xml_users.xml'))
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
$UserKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\allocation-test-support.jar".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-users-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\UsersSmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-users.args", $UserKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-users-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-users.args")
Step 'kotlin-users-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-users-smoke.jar", 'UsersSmokeKt', 'crates\lox-bindings\tests\fixtures\xml_users.xml'))
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

Step 'python-statistics-month-merge-smoke' $VenvPython @('-I', '-B', 'tests\bindings\statistics_month_merge_smoke.py', "$Probe.dll")
Step 'javac-statistics-month-merge-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\StatisticsMonthMergeSmoke.java', 'tests\bindings\NativeStatisticsMonthMergeProbe.java')
Step 'java-statistics-month-merge-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'StatisticsMonthMergeSmoke'))
$StatisticsMonthMergeKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\test-classes".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-statistics-month-merge-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\StatisticsMonthMergeSmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-statistics-month-merge.args", $StatisticsMonthMergeKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-statistics-month-merge-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-statistics-month-merge.args")
Step 'kotlin-statistics-month-merge-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-statistics-month-merge-smoke.jar", 'StatisticsMonthMergeSmokeKt'))
@{
    actual_all_three_established_utc = [DateTime]::UtcNow.ToString('o')
    dll_sha256 = (Get-FileHash $InstalledDll -Algorithm SHA256).Hash.ToLowerInvariant()
    proof = 'Actual caller-ordered month merge success/OutputMismatch RustBuffers: unchanged prefix128/minimum64000 matcher, live1/free0,64 distinct retained host objects per branch per language and native_retained=0 without requested GC. Generated Python and SAME Java22/Kotlin APIs passed exact typed fields, order, and raw binary64 witnesses including signed zero, subnormal, infinities and two quiet NaN payloads.'
} | ConvertTo-Json | Set-Content "$Output\STATISTICS-MONTH-MERGE-MECHANISM.json" -Encoding utf8

Step 'python-fragment-application-smoke' $VenvPython @('-I', '-B', 'tests\bindings\fragment_application_smoke.py', "$Probe.dll")
Step 'javac-fragment-application-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\FragmentApplicationSmoke.java')
Step 'java-fragment-application-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'net.eisber.lox.archive.FragmentApplicationSmoke'))
$FragmentApplicationKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\test-classes".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-fragment-application-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\FragmentApplicationSmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-fragment-application.args", $FragmentApplicationKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-fragment-application-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-fragment-application.args")
Step 'kotlin-fragment-application-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-fragment-application-smoke.jar", 'FragmentApplicationSmokeKt'))
@{
    actual_all_three_established_utc = [DateTime]::UtcNow.ToString('o')
    dll_sha256 = (Get-FileHash $InstalledDll -Algorithm SHA256).Hash.ToLowerInvariant()
    proof = 'Actual nonempty fragment replacement success and named FragmentRead/Archive (four codec causes), FragmentRead/FragmentIntegrity and UuidConsistency/DuplicateUuid failures: unchanged HeapWalk, live1/free0,64 distinct retained host objects per branch per language and native_retained=0 without requested GC. Generated installed Python and SAME Java22/Kotlin APIs passed exact byte, filename order, duplicate, placement, bypass, and typed cause checks. Small bypass and other short/payload-free branches have no separate allocation-attribution claim.'
} | ConvertTo-Json | Set-Content "$Output\FRAGMENT-APPLICATION-MECHANISM.json" -Encoding utf8

Step 'python-program-text-smoke' $VenvPython @('-I', '-B', 'tests\bindings\program_text_smoke.py', "$Probe.dll")
Step 'javac-program-text-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\ProgramTextSmoke.java')
Step 'java-program-text-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'net.eisber.lox.archive.ProgramTextSmoke'))
$ProgramTextKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\test-classes".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-program-text-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\ProgramTextSmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-program-text.args", $ProgramTextKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-program-text-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-program-text.args")
Step 'kotlin-program-text-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-program-text-smoke.jar", 'ProgramTextSmokeKt'))
@{
    actual_all_three_established_utc = [DateTime]::UtcNow.ToString('o')
    dll_sha256 = (Get-FileHash $InstalledDll -Algorithm SHA256).Hash.ToLowerInvariant()
    proof = 'Actual selected SequenceController text success and Load/NotFound/Ambiguous/Unsupported RustBuffers from the same export: live address and prefix count 1, freed address and count 0, 64 distinct retained host objects per branch and native_retained=0 per language without requested GC. The marked ProgramTextRequest input buffer calibrates live1/free0, then 64 host surrogate rejections per language retain no matching native input. Installed Python and SAME Java22/Kotlin APIs passed exact/empty text and owned ordered diagnostics with unchanged Core loader (lox-n37w). Empty success has no separate allocation claim.'
} | ConvertTo-Json | Set-Content "$Output\PROGRAM-TEXT-MECHANISM.json" -Encoding utf8

Step 'python-mood-inventory-smoke' $VenvPython @('-I', '-B', 'tests\bindings\mood_inventory_smoke.py', "$Probe.dll")
Step 'javac-mood-inventory-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\MoodInventorySmoke.java')
Step 'java-mood-inventory-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'net.eisber.lox.archive.MoodInventorySmoke'))
$MoodInventoryKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\test-classes".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-mood-inventory-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\MoodInventorySmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-mood-inventory.args", $MoodInventoryKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-mood-inventory-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-mood-inventory.args")
Step 'kotlin-mood-inventory-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-mood-inventory-smoke.jar", 'MoodInventorySmokeKt'))
@{
    actual_all_three_established_utc = [DateTime]::UtcNow.ToString('o')
    dll_sha256 = (Get-FileHash $InstalledDll -Algorithm SHA256).Hash.ToLowerInvariant()
    proof = 'Installed list_moods nonempty success and Load/NotFound/Ambiguous/NotLightController RustBuffers: calibrated live address and prefix count 1, freed address and count 0, and 64 distinct retained host values per branch with zero matching native allocations, independently in Python, Java and Kotlin without requested GC. A distinct MoodInventoryRequest XML prefix calibrated live1/free0, then 64 pre-call lone-surrogate rejections per language retained zero matching native inputs. Empty inventory is a value check only.'
} | ConvertTo-Json | Set-Content "$Output\MOOD-INVENTORY-MECHANISM.json" -Encoding utf8

Step 'python-status-monitor-smoke' $VenvPython @('-I', '-B', 'tests\bindings\status_monitor_smoke.py', "$Probe.dll")
Step 'javac-status-monitor-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\StatusMonitorSmoke.java')
Step 'java-status-monitor-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'net.eisber.lox.archive.StatusMonitorSmoke'))
$StatusMonitorKotlinArguments = @('-jvm-target', '22', '-classpath',
    ('"' + "$ApiJar;$Output\jvm\test-classes".Replace('\', '\\') + '"'),
    '-include-runtime', '-d', ('"' + "$Output\jvm\kotlin-status-monitor-smoke.jar".Replace('\', '\\') + '"'),
    'tests\bindings\StatusMonitorSmoke.kt')
[IO.File]::WriteAllLines("$Output\jvm\kotlin-status-monitor.args", $StatusMonitorKotlinArguments, [Text.UTF8Encoding]::new($false))
Step 'kotlinc-status-monitor-smoke' $KotlinCompiler @("@$Output\jvm\kotlin-status-monitor.args")
Step 'kotlin-status-monitor-smoke' $Java ($AllocationAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes;$Output\jvm\kotlin-status-monitor-smoke.jar", 'StatusMonitorSmokeKt'))
@{
    actual_all_three_established_utc = [DateTime]::UtcNow.ToString('o')
    dll_sha256 = (Get-FileHash $InstalledDll -Algorithm SHA256).Hash.ToLowerInvariant()
    proof = 'Actual stored Status Monitor inspection success and Ambiguous/InvalidValue errors from the same export: live address and prefix count 1, freed address and count 0, 64 distinct retained host objects per branch and native_retained=0 per language without requested GC. Each language also calibrated the marked StatusMonitorRequest input buffer live1/free0, then rejected 64 lone-surrogate selectors through the public entry with no matching retained native input. Installed CPython 3.13 GIL and SAME Java22/Kotlin APIs checked ordered owned fields, raw text, missing/empty attributes and typed refusals. Other branches have no separate allocation-attribution claim.'
} | ConvertTo-Json | Set-Content "$Output\STATUS-MONITOR-MECHANISM.json" -Encoding utf8

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
$Artifacts += Get-Item "$Output\python-custom-listing-smoke.log", "$Output\java-custom-listing-smoke.log", "$Output\kotlin-custom-listing-smoke.log", "$Output\CUSTOM-LISTING-MECHANISM.json"
$Artifacts += Get-Item "$Output\runtime-overlay-core-focused.log", "$Output\python-runtime-overlay-smoke.log", "$Output\javac-runtime-overlay-smoke.log", "$Output\java-runtime-overlay-smoke.log", "$Output\kotlinc-runtime-overlay-smoke.log", "$Output\kotlin-runtime-overlay-smoke.log", "$Output\RUNTIME-OVERLAY-MECHANISM.json"
$Artifacts += Get-Item "$Output\python-statistics-identity-smoke.log", "$Output\java-statistics-identity-smoke.log", "$Output\kotlin-statistics-identity-smoke.log", "$Output\STATISTICS-IDENTITY-MECHANISM.json"
$Artifacts += Get-Item "$Output\statistics-identity-core-focused.log", "$Output\statistics-identity-cli-focused.log", "$Output\javac-statistics-identity-smoke.log", "$Output\kotlinc-statistics-identity-smoke.log"
$Artifacts += Get-Item "$Output\python-text-state-smoke.log", "$Output\java-text-state-smoke.log", "$Output\kotlin-text-state-smoke.log"
$Artifacts += Get-Item "$Output\python-daytimer-state-smoke.log", "$Output\java-daytimer-state-smoke.log", "$Output\kotlin-daytimer-state-smoke.log", "$Output\DAYTIMER-STATE-MECHANISM.json"
$Artifacts += Get-Item "$Output\python-weather-state-smoke.log", "$Output\java-weather-state-smoke.log", "$Output\kotlin-weather-state-smoke.log", "$Output\WEATHER-STATE-MECHANISM.json"
$Artifacts += Get-Item "$Output\python-devices-smoke.log", "$Output\java-devices-smoke.log", "$Output\kotlin-devices-smoke.log"
$Artifacts += Get-Item "$Output\python-users-smoke.log", "$Output\java-users-smoke.log", "$Output\kotlin-users-smoke.log"
$Artifacts += Get-Item "$Output\python-config-diff-smoke.log", "$Output\java-config-diff-smoke.log", "$Output\kotlin-config-diff-smoke.log"
$Artifacts += Get-Item "$Output\python-structure-metadata-smoke.log", "$Output\java-structure-metadata-smoke.log", "$Output\kotlin-structure-metadata-smoke.log", "$Output\METADATA-MECHANISM.json"
$Artifacts += Get-Item "$Output\python-autopilot-catalog-smoke.log", "$Output\java-autopilot-catalog-smoke.log", "$Output\kotlin-autopilot-catalog-smoke.log", "$Output\AUTOPILOT-MECHANISM.json"
$Artifacts += Get-Item "$Output\python-autopilot-rule-smoke.log", "$Output\java-autopilot-rule-smoke.log", "$Output\kotlin-autopilot-rule-smoke.log", "$Output\AUTOPILOT-RULE-MECHANISM.json"
$Artifacts += Get-Item "$Output\python-statistics-index-smoke.log", "$Output\java-statistics-index-smoke.log", "$Output\kotlin-statistics-index-smoke.log", "$Output\STATISTICS-INDEX-MECHANISM.json"
$Artifacts += Get-Item "$Output\statistics-series-core-focused.log", "$Output\python-statistics-series-smoke.log", "$Output\javac-statistics-series-smoke.log", "$Output\java-statistics-series-smoke.log", "$Output\kotlinc-statistics-series-smoke.log", "$Output\kotlin-statistics-series-smoke.log", "$Output\STATISTICS-SERIES-MECHANISM.json", "$Output\jvm\kotlin-statistics-series.args"
$Artifacts += Get-Item "$Output\rust-focused.log", "$Output\python-statistics-month-merge-smoke.log", "$Output\javac-statistics-month-merge-smoke.log", "$Output\java-statistics-month-merge-smoke.log", "$Output\kotlinc-statistics-month-merge-smoke.log", "$Output\kotlin-statistics-month-merge-smoke.log", "$Output\STATISTICS-MONTH-MERGE-MECHANISM.json"
$Artifacts += Get-Item "$Output\python-fragment-application-smoke.log", "$Output\javac-fragment-application-smoke.log", "$Output\java-fragment-application-smoke.log", "$Output\kotlinc-fragment-application-smoke.log", "$Output\kotlin-fragment-application-smoke.log", "$Output\FRAGMENT-APPLICATION-MECHANISM.json"
$Artifacts += Get-Item "$Output\program-text-core-focused.log", "$Output\python-program-text-smoke.log", "$Output\javac-program-text-smoke.log", "$Output\java-program-text-smoke.log", "$Output\kotlinc-program-text-smoke.log", "$Output\kotlin-program-text-smoke.log", "$Output\PROGRAM-TEXT-MECHANISM.json"
$Artifacts += Get-Item "$Output\mood-inventory-core-focused.log", "$Output\python-mood-inventory-smoke.log", "$Output\javac-mood-inventory-smoke.log", "$Output\java-mood-inventory-smoke.log", "$Output\kotlinc-mood-inventory-smoke.log", "$Output\kotlin-mood-inventory-smoke.log", "$Output\MOOD-INVENTORY-MECHANISM.json"
$Artifacts += Get-Item "$Output\status-monitor-core-focused.log", "$Output\python-status-monitor-smoke.log", "$Output\javac-status-monitor-smoke.log", "$Output\java-status-monitor-smoke.log", "$Output\kotlinc-status-monitor-smoke.log", "$Output\kotlin-status-monitor-smoke.log", "$Output\STATUS-MONITOR-MECHANISM.json"
$Artifacts += Get-Item "$Output\python-firmware-metadata-smoke.log", "$Output\java-firmware-metadata-smoke.log", "$Output\kotlin-firmware-metadata-smoke.log", "$Output\FIRMWARE-METADATA-MECHANISM.json"
$Artifacts += Get-Item "$Output\python-backup-catalog-smoke.log", "$Output\java-backup-catalog-smoke.log", "$Output\kotlin-backup-catalog-smoke.log", "$Output\BACKUP-CATALOG-MECHANISM.json"
$Artifacts += Get-Item "$Output\python-mqtt-catalog-smoke.log", "$Output\java-mqtt-catalog-smoke.log", "$Output\kotlin-mqtt-catalog-smoke.log", "$Output\MQTT-CATALOG-MECHANISM.json"
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
    contract = 'Archive, raw-byte read-only XML user/device/control/room/summary inspection, typed supplied-summary structural comparison delegated to Core, offline Core-only byte-preserving three-way merge and application of supplied filename-keyed LoxCC fragments with ordered typed domain cause frames and explicit native parser/UTF8 boundaries, stateless typed LoxApp3 catalog parsing/filtering, supplied typed Autopilot rule selection delegated to Core with owned query/candidate errors, infallible HTML statistics index catalog delegated to Core, supplied statistics-index identity resolution delegated to Core with raw UUID result and typed sorted diagnostics, strict supplied successful custom FTP listing interpretation with raw sorted names/duplicates and data-free typed row errors, atomic monthly statistics XML series delegated to Core with opaque timestamps/owned finite f64 rows/typed errors and original source text, caller-ordered merge of supplied statistics series delegated to Core with owned rows and typed output mismatch, payload-only numeric state decoding with exact UUID/IEEE bits, strict payload-only text state decoding with raw/native35 state/icon UUIDs and exact UTF8 including NUL, strict payload-only Daytimer-State decoding with signed observations/raw activation word/two authoritative IEEE bit fields and exact typed atomic errors, strict payload-only Weather-State decoding with unsigned update/signed observations/six authoritative IEEE bit fields and exact typed atomic errors, strict production HTTP clock, pure typed control selection, and offline typed color encoding/explicit legacy observations. User inventory is raw owned fields only, never credentials/permissions/authentication. Current loopback clock cancellation/close only; no overlay acquisition/schema/baseline/deployment guarantee, custom-listing acquisition or validation of filenames at listing time, statistics group/month selection or acquisition, Lighting eligibility, framing, HTTP weather layout, full XML/semantic diff, editor/auth/control I/O/binary statistics/subscriptions/full-runtime/platform parity. Candidate composition remains publication-gated.'
    native_rust_min_stack = $env:RUST_MIN_STACK
    generators = @{ uniffi = '0.32.1'; java = '0.5.2'; maturin = '1.16.0' }
    artifact_manifest = $Manifest
    excluded = @('target', 'temp', 'venv except tested native DLL/generated module/async adapter')
} | ConvertTo-Json -Depth 8 | Set-Content "$Output\ARTIFACTS.json" -Encoding utf8
Write-Output "Archive artifact proof complete: $Output\ARTIFACTS.json"
