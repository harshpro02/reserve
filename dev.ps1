param(
    [ValidateSet('run', 'test', 'fmt', 'clippy')]
    [string]$Task = 'run'
)

# Rust is installed in the existing Ubuntu WSL distribution.
# This small launcher lets you use it from a Windows PowerShell terminal.
$projectWindowsPath = $PSScriptRoot
$projectWslPath = (& wsl.exe -d Ubuntu --exec wslpath -u $projectWindowsPath)
if ($LASTEXITCODE -ne 0) { throw 'Could not find the project in Ubuntu WSL.' }
$projectWslPath = $projectWslPath.Trim()

$cargoArgs = @($Task)
if ($Task -eq 'fmt') { $cargoArgs += '--check' }
if ($Task -eq 'clippy') { $cargoArgs += @('--all-targets', '--', '-D', 'warnings') }

& wsl.exe -d Ubuntu --cd $projectWslPath --exec bash -lc 'source "$HOME/.cargo/env"; exec cargo "$@"' reserve @cargoArgs
exit $LASTEXITCODE
