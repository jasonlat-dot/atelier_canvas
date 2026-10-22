param([string]$PgBin = $env:PG_BIN)
$ErrorActionPreference='Stop'
$projectRoot=Split-Path -Parent $PSScriptRoot
$previousUrl=$env:ATELIER_TEST_DATABASE_URL
$ownedCluster=$false
$testRoot=$null
try {
 if (-not $previousUrl) {
  if (-not $PgBin) { $command=Get-Command initdb -ErrorAction SilentlyContinue;if($command){$PgBin=Split-Path $command.Source} }
  if (-not $PgBin -or -not (Test-Path (Join-Path $PgBin 'initdb.exe'))) {throw 'Provide -PgBin or PG_BIN for a portable isolated cluster, or set ATELIER_TEST_DATABASE_URL to a dedicated atelier_test_* database.'}
  $probe=[Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,0);$probe.Start();$testPort=$probe.LocalEndpoint.Port;$probe.Stop()
  $testRoot=Join-Path ([IO.Path]::GetTempPath()) ('atelier-s1-'+[guid]::NewGuid().ToString('N'))
  New-Item -ItemType Directory -Path $testRoot | Out-Null
  $testData=Join-Path $testRoot 'data'
  & (Join-Path $PgBin 'initdb.exe') -D $testData -U atelier_test --auth=trust --encoding=UTF8 --locale=C | Out-Null
  if($LASTEXITCODE -ne 0){throw 'Isolated initdb failed'}
  $launcher=Start-Process -FilePath (Join-Path $PgBin 'pg_ctl.exe') -ArgumentList @('-D',"`"$testData`"",'-l',"`"$(Join-Path $testRoot 'server.log')`"",'-o',"`"-h 127.0.0.1 -p $testPort`"",'-w','start') -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $testRoot 'start.out') -RedirectStandardError (Join-Path $testRoot 'start.err')
  $ownedCluster=$true
  if(-not $launcher.WaitForExit(30000)){throw 'Isolated PostgreSQL launcher timed out'}
  if($launcher.ExitCode -ne 0){throw 'Isolated PostgreSQL start failed; inspect start.err'}
  $ownedCluster=$true
  $database='atelier_test_'+[guid]::NewGuid().ToString('N')
  & (Join-Path $PgBin 'createdb.exe') -h 127.0.0.1 -p $testPort -U atelier_test $database
  if($LASTEXITCODE -ne 0){throw 'Test database creation failed'}
  $env:ATELIER_TEST_DATABASE_URL="postgres://atelier_test@127.0.0.1:$testPort/$database"
 }
 Push-Location $projectRoot
 try {
  & cargo test --offline --manifest-path services/api/Cargo.toml providers::tests::postgres_provider_lifecycle -- --ignored --nocapture
  if($LASTEXITCODE -ne 0){throw 'Provider integration test failed'}
 } finally {Pop-Location}
} finally {
 $env:ATELIER_TEST_DATABASE_URL=$previousUrl
 if($ownedCluster){
  & (Join-Path $PgBin 'pg_ctl.exe') -D $testData -m fast -w stop | Out-Null
  if($LASTEXITCODE -ne 0){Write-Warning 'Isolated PostgreSQL did not stop; inspect its own temp directory.'}
  if($LASTEXITCODE -eq 0){Write-Output "Isolated PostgreSQL stopped; diagnostic files retained at $testRoot"}
 }
}