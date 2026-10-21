# Run from any directory after syncing the project. No production build or packaging.
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
foreach ($commandName in @('node','npm','cargo')) {
    if (-not (Get-Command $commandName -ErrorAction SilentlyContinue)) {
        throw "Required command missing: $commandName. Install project dependencies first; see docs/AGENT_REUSE_PLAN.md."
    }
}
Push-Location -LiteralPath $projectRoot
try {
    & npm --prefix apps/web test
    if ($LASTEXITCODE -ne 0) { throw 'Frontend regression tests failed.' }
    & npm --prefix apps/web run typecheck
    if ($LASTEXITCODE -ne 0) { throw 'Frontend typecheck failed.' }
    & cargo test --locked --offline --manifest-path crates/atelier-agent-core/Cargo.toml
    if ($LASTEXITCODE -ne 0) { throw 'Agent core tests failed (or dependencies are unavailable offline).' }
    & cargo check --locked --offline --manifest-path services/api/Cargo.toml
    if ($LASTEXITCODE -ne 0) { throw 'API compile failed (or dependencies are unavailable offline).' }
    & cargo test --locked --offline --manifest-path services/api/Cargo.toml
    if ($LASTEXITCODE -ne 0) { throw 'API tests failed (or dependencies are unavailable offline).' }
    Write-Output 'Agent checks passed. Database integration and browser acceptance require separate verification.'
}
finally { Pop-Location }