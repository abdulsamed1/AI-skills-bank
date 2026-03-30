$src = Join-Path (Get-Location) 'skills-aggregated'
$targets = @('.copilot\skills','.gemini\skills','.gemini\antigravity\skills')
foreach ($rel in $targets) {
  $dest = Join-Path $env:USERPROFILE $rel
  Write-Host "`n== Attempting target: $dest =="
  try {
    New-Item -ItemType Directory -Path $dest -Force | Out-Null
    Copy-Item -Path (Join-Path $src '*') -Destination $dest -Recurse -Force -ErrorAction Stop
    Write-Host "Copied -> $dest"
  } catch {
    Write-Host "ERROR copying to $dest : $($_.Exception.Message)"
  }
}
