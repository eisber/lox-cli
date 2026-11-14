param(
    [Parameter(Mandatory)][string]$Maturin,
    [Parameter(Mandatory)][string]$JavaGenerator,
    [Parameter(Mandatory)][string]$JavaHome,
    [Parameter(Mandatory)][string]$KotlinCompiler,
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
$env:RUST_MIN_STACK = '16777216'
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
$InstalledDll = "$Output\venv\Lib\site-packages\lox_archive\lox_archive.dll"
Step 'java-generate' $JavaGenerator @('generate', '--no-format', '--out-dir', "$Output\generated-java", $InstalledDll)

New-Item -ItemType Directory -Force "$Output\jvm\classes", "$Output\jvm\test-classes", "$Output\jvm\native", "$Output\jvm\metadata\META-INF\licenses" | Out-Null
foreach ($file in @('LICENSE', 'LICENSE-AGPL', 'LICENSE-COMMERCIAL', 'NOTICE')) {
    Copy-Item -Force $file "$Output\jvm\metadata\META-INF\licenses\$file"
}
Copy-Item -Force 'crates\lox-bindings\THIRD_PARTY_NOTICES' "$Output\jvm\metadata\META-INF\licenses"
Copy-Item -Force 'crates\lox-bindings\licenses\MPL-2.0.txt' "$Output\jvm\metadata\META-INF\licenses"
$Sources = Get-ChildItem "$Output\generated-java" -Recurse -Filter '*.java' |
    ForEach-Object { '"' + $_.FullName.Replace('\', '\\') + '"' }
[IO.File]::WriteAllLines("$Output\jvm\sources.args", [string[]]$Sources, [Text.UTF8Encoding]::new($false))
Step 'javac-api' $Javac @('--release', '22', '-d', "$Output\jvm\classes", "@$Output\jvm\sources.args")
$ApiJar = "$Output\jvm\lox-archive-0.1.0.jar"
Step 'jar-api' $Jar @('--create', '--file', $ApiJar, '-C', "$Output\jvm\classes", '.', '-C', "$Output\jvm\metadata", '.')
Step 'jar-sources' $Jar @('--create', '--file', "$Output\jvm\lox-archive-0.1.0-sources.jar", '-C', "$Output\generated-java", '.', '-C', "$Output\jvm\metadata", '.')
Copy-Item -Force $InstalledDll "$Output\jvm\native\lox_archive.dll"
if ((Get-FileHash $InstalledDll).Hash -ne (Get-FileHash "$Output\jvm\native\lox_archive.dll").Hash) {
    throw 'Python and JVM native artifacts differ.'
}
Step 'javac-smoke' $Javac @('--release', '22', '-cp', $ApiJar, '-d', "$Output\jvm\test-classes", 'tests\bindings\ArchiveSmoke.java')
$NativeAccess = @('--enable-native-access=ALL-UNNAMED', "-Djava.library.path=$Output\jvm\native")
Step 'java-smoke' $Java ($NativeAccess + @('-cp', "$ApiJar;$Output\jvm\test-classes", 'ArchiveSmoke', "$Output\fixtures"))
Step 'kotlinc-smoke' $KotlinCompiler @('-jvm-target', '22', '-classpath', $ApiJar, '-include-runtime', '-d', "$Output\jvm\kotlin-smoke.jar", 'tests\bindings\ArchiveSmoke.kt')
Step 'kotlin-smoke' $Java ($NativeAccess + @('-cp', "$ApiJar;$Output\jvm\kotlin-smoke.jar", 'ArchiveSmokeKt', "$Output\fixtures"))

$Artifacts = @(Get-ChildItem "$Output\wheels", "$Output\generated-java", "$Output\jvm", "$Output\fixtures" -Recurse -File)
$Artifacts += Get-Item $InstalledDll, "$Output\venv\Lib\site-packages\lox_archive\lox_archive.py", "$Output\tools\uniffi-bindgen.exe"
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
    contract = 'archive functions only; no runtime/lifecycle qualification'
    generators = @{ uniffi = '0.32.1'; java = '0.5.2'; maturin = '1.16.0' }
    artifact_manifest = $Manifest
    excluded = @('target', 'temp', 'venv except tested native DLL/generated module')
} | ConvertTo-Json -Depth 8 | Set-Content "$Output\ARTIFACTS.json" -Encoding utf8
Write-Output "Archive artifact proof complete: $Output\ARTIFACTS.json"
