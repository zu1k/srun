#!pwsh
<#
    OpenSSL is already installed on windows-latest virtual environment.
    If you need OpenSSL, consider install it by:

    choco install openssl
#>

param(
    [Parameter(HelpMessage = "Specify the target triple directly")]
    [Alias('t')]
    [string]$TargetTriple,

    [Parameter(HelpMessage = "extra features")]
    [Alias('f')]
    [string]$Features
)

$ErrorActionPreference = "Stop"


if (-not $PSBoundParameters.ContainsKey('TargetTriple')) {
    try {
        $TargetTriple = (rustc -Vv | Select-String -Pattern "host: (.*)" | ForEach-Object { $_.Matches.Value }).split()[-1]
    } catch {
        Write-Error "Unable to determine TargetTriple automatically"
    }
}

Write-Host "Started building release for ${TargetTriple} ..."

if ([string]::IsNullOrEmpty($Features)) {
    cargo build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --release --target $TargetTriple
}
else {
    cargo build -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --release --target $TargetTriple --features "${Features}"
}

if (!$?) {
    exit $LASTEXITCODE
}

$Version = (Select-String -Pattern '^version *= *"([^"]*)"$' -Path "${PSScriptRoot}\..\Cargo.toml" | ForEach-Object { $_.Matches.Value }).split()[-1]
$Version = $Version -replace '"'

$PackageReleasePath = "${PSScriptRoot}\release"
$PackageName = "srun-${Version}.${TargetTriple}.zip"
$PackagePath = "${PackageReleasePath}\${PackageName}"

Write-Host $Version
Write-Host $PackageReleasePath
Write-Host $PackageName
Write-Host $PackagePath

Push-Location "${PSScriptRoot}\..\target\$TargetTriple\release"

$ProgressPreference = "SilentlyContinue"
New-Item "${PackageReleasePath}" -ItemType Directory -ErrorAction SilentlyContinue
$CompressParam = @{
    LiteralPath     = "srun.exe"
    DestinationPath = "${PackagePath}"
}
Compress-Archive @CompressParam

Write-Host "Created release packet ${PackagePath}"

$PackageChecksumPath = "${PackagePath}.sha256"
$PackageHash = (Get-FileHash -Path "${PackagePath}" -Algorithm SHA256).Hash
"${PackageHash}  ${PackageName}" | Out-File -FilePath "${PackageChecksumPath}"

Write-Host "Created release packet checksum ${PackageChecksumPath}"
