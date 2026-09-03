[CmdletBinding()]
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]] $KallyupArguments
)

$ErrorActionPreference = 'Stop'
$KallyupRepository = 'https://github.com/Kalcite-Engine/kallyup.git'
$CargoHome = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { Join-Path $HOME '.cargo' }
$env:Path = "$(Join-Path $CargoHome 'bin');$env:Path"

function Test-Command([string] $Name) {
    return $null -ne (Get-Command $Name -ErrorAction SilentlyContinue)
}

function Install-Git {
    if (Test-Command git) { return }

    if (Test-Command winget) {
        winget install --id Git.Git --exact --silent --accept-package-agreements --accept-source-agreements
        $gitBin = Join-Path $env:ProgramFiles 'Git\cmd'
        if (Test-Path $gitBin) { $env:Path = "$gitBin;$env:Path" }
    } elseif (Test-Command choco) {
        choco install git -y
    } elseif (Test-Command scoop) {
        scoop install git
    } else {
        throw 'Git is missing and Winget, Chocolatey, and Scoop are unavailable. Install Git, then run this script again.'
    }

    if (-not (Test-Command git)) {
        throw 'Git was installed but is not available in this session. Open a new PowerShell window and run this script again.'
    }
}

function Install-Rust {
    if (Test-Command cargo) { return }

    $rustupInit = Join-Path $env:TEMP 'rustup-init.exe'
    Invoke-WebRequest -UseBasicParsing -Uri 'https://win.rustup.rs/x86_64' -OutFile $rustupInit
    & $rustupInit -y --profile minimal --default-toolchain stable
    $env:Path = "$(Join-Path $CargoHome 'bin');$env:Path"

    if (-not (Test-Command cargo)) {
        throw 'Rustup completed but Cargo was not found. Open a new PowerShell window and run this script again.'
    }
}

Install-Git
Install-Rust
cargo install --git $KallyupRepository --branch main --locked --force

$kallyup = Join-Path $CargoHome 'bin\kallyup.exe'
if (-not (Test-Path $kallyup)) { throw 'Kallyup was not installed.' }
if ($KallyupArguments.Count -eq 0) { $KallyupArguments = @('list') }
& $kallyup @KallyupArguments
exit $LASTEXITCODE
