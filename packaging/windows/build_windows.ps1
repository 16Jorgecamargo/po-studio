$ErrorActionPreference = "Stop"

$Root = Resolve-Path (Join-Path $PSScriptRoot "..\..")
Set-Location $Root

$Version = (Select-String -Path "Cargo.toml" -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1).Matches[0].Groups[1].Value

cargo build --release
if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }

New-Item -ItemType Directory -Force -Path "dist" | Out-Null
Copy-Item "target\release\po-studio.exe" "dist\PO-Studio-$Version-windows-portable.exe" -Force

$Iscc = Get-Command "iscc.exe" -ErrorAction SilentlyContinue
if (-not $Iscc) {
    $Candidates = @("${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe", "$env:ProgramFiles\Inno Setup 6\ISCC.exe")
    $Found = $Candidates | Where-Object { Test-Path $_ } | Select-Object -First 1
    if ($Found) { $Iscc = $Found } else { $Iscc = $null }
} else {
    $Iscc = $Iscc.Source
}

if ($Iscc) {
    & $Iscc "/DAppVersion=$Version" "packaging\windows\po_studio.iss"
    if ($LASTEXITCODE -ne 0) { throw "Inno Setup failed" }
} else {
    Write-Warning "Inno Setup 6 not found: only the portable exe was made. Install it with: winget install JRSoftware.InnoSetup"
}

Get-ChildItem "dist" | Where-Object { $_.Name -like "PO-Studio-*windows*" } | ForEach-Object { $_.FullName }
