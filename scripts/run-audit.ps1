Write-Host "Running cargo audit"
if (-not (Get-Command cargo-audit -ErrorAction SilentlyContinue)) {
  Write-Host "cargo-audit not found. Installing..."
  cargo install cargo-audit
}

cargo audit
