Param(
  [string]$profile = 'smoke'
)

Write-Host "Running k6 profile: $profile"

if ($profile -eq 'smoke') {
  k6 run tests/load_tests/create-and-get.js --vus 5 --duration 30s
} elseif ($profile -eq 'full') {
  # stages are configured inside the script; run full scenario
  k6 run tests/load_tests/create-and-get.js
} else {
  Write-Host "Unknown profile: $profile. Use 'smoke' or 'full'"
}
