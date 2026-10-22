param([string]$DatabaseUrl=$env:DATABASE_URL)
$ErrorActionPreference='Stop'
$projectRoot=Split-Path -Parent $PSScriptRoot
if(-not $DatabaseUrl){$DatabaseUrl='postgres://atelier:atelier_local_only@127.0.0.1:5432/atelier'}
$env:DATABASE_URL=$DatabaseUrl
$env:DEV_AUTH='true'
if(-not $env:BIND_ADDRESS){$env:BIND_ADDRESS='127.0.0.1:8080'}
if(-not $env:ATELIER_CREDENTIAL_KEY){
 $keyDirectory=Join-Path $projectRoot '.atelier'
 New-Item -ItemType Directory -Force -Path $keyDirectory | Out-Null
 $keyFile=Join-Path $keyDirectory 'credential.key'
 if(-not (Test-Path -LiteralPath $keyFile)){
  $bytes=New-Object byte[] 32;$rng=[Security.Cryptography.RandomNumberGenerator]::Create()
  try{$rng.GetBytes($bytes)}finally{$rng.Dispose()}
  $encoded=[Text.Encoding]::ASCII.GetBytes([Convert]::ToBase64String($bytes))
  try{$stream=[IO.File]::Open($keyFile,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::None);try{$stream.Write($encoded,0,$encoded.Length)}finally{$stream.Dispose()}}
  catch [IO.IOException]{if(-not (Test-Path -LiteralPath $keyFile)){throw}}
 }
 $env:ATELIER_CREDENTIAL_KEY=[IO.File]::ReadAllText($keyFile).Trim()
 Write-Output 'Using local development credential encryption key; keep .atelier/credential.key private and backed up.'
}
Push-Location $projectRoot
try{& cargo run --manifest-path services/api/Cargo.toml;if($LASTEXITCODE -ne 0){throw 'Rust API exited with an error'}}finally{Pop-Location}