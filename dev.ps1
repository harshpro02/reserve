param(
    [ValidateSet('run', 'serve', 'checkout', 'test', 'fmt', 'clippy')]
    [string]$Task = 'run'
)

# Rust is installed in the existing Ubuntu WSL distribution.
# This small launcher lets you use it from a Windows PowerShell terminal.
$projectWindowsPath = $PSScriptRoot
$projectWslPath = (& wsl.exe -d Ubuntu --exec wslpath -u $projectWindowsPath)
if ($LASTEXITCODE -ne 0) { throw 'Could not find the project in Ubuntu WSL.' }
$projectWslPath = $projectWslPath.Trim()

$cargoArgs = @($Task)
if ($Task -eq 'run') { $cargoArgs = @('run', '--bin', 'reserve') }
if ($Task -eq 'serve') { $cargoArgs = @('run', '--bin', 'server') }
if ($Task -eq 'checkout') { $cargoArgs = @('run', '--example', 'checkout') }
if ($Task -eq 'fmt') { $cargoArgs += '--check' }
if ($Task -eq 'clippy') { $cargoArgs += @('--all-targets', '--', '-D', 'warnings') }

& wsl.exe -d Ubuntu --cd $projectWslPath --exec env CARGO_TARGET_DIR=/root/portfolio-target /root/.cargo/bin/cargo @cargoArgs
exit $LASTEXITCODE
