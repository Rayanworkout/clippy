# Save this as install.ps1 and run it in PowerShell as Administrator.
$scriptDir = $PSScriptRoot

# Define the expected binary filenames (assuming they are built for Windows)
$daemonFile = Join-Path $scriptDir "daemon.exe"
$uiFile = Join-Path $scriptDir "ui.exe"

# Check if daemon.exe exists
if (!(Test-Path $daemonFile)) {
    Write-Error "Error: 'daemon.exe' not found in $scriptDir. Exiting."
    exit 1
}

# Check if ui.exe exists
if (!(Test-Path $uiFile)) {
    Write-Error "Error: 'ui.exe' not found in $scriptDir. Exiting."
    exit 1
}

# Destination directory (adjust as needed)
$destDir = "C:\Program Files\Clippy"

# Ensure the destination directory exists; create it if it doesn't
if (!(Test-Path $destDir)) {
    Write-Host "Destination directory $destDir does not exist. Creating it..."
    New-Item -ItemType Directory -Path $destDir -Force | Out-Null
}

# Copy the binaries to the destination directory
Write-Host "Installing daemon.exe to $destDir..."
Copy-Item -Path $daemonFile -Destination (Join-Path $destDir "daemon.exe") -Force

Write-Host "Installing ui.exe to $destDir..."
Copy-Item -Path $uiFile -Destination (Join-Path $destDir "ui.exe") -Force

Write-Host "Installation complete!"
