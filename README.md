# Kallyup

Kallyup is the KLC-first bootstrapper for Kalcite. Its profile policy is
compiled from `src/kallyup_core.klc`; Rust is restricted to host actions that
KLC cannot yet safely perform: starting Cargo/Git processes and installing
binaries.

```sh
cargo install --git https://github.com/Kalcite-Engine/kallyup.git
kallyup list
kallyup install minimal
kallyup install developer --root "$HOME/.local"
kallyup install full
```

Profiles are `minimal` (Kalcite + Kally), `developer` (+ LSP), and `full`
(+ editor). Kallyup never requests privilege escalation or edits `PATH`; it
prints the required path action after installation.

## One-command bootstrap

The launchers install their host requirements when possible (Git, a native C
build toolchain, Rustup and Cargo), install or update Kallyup, then run it.
Pass the normal Kallyup command after the script; omitting it displays the
available profiles.

```sh
# Linux and macOS
curl -fsSL https://raw.githubusercontent.com/Kalcite-Engine/kallyup/main/scripts/kallyup-bootstrap.sh | sh -s -- install developer
```

```powershell
# Windows PowerShell: install the developer profile in the current session
irm https://raw.githubusercontent.com/Kalcite-Engine/kallyup/main/scripts/kallyup-bootstrap.ps1 | iex; kallyup install developer
```

Replace `developer` with `minimal` or `full` to select another profile. Linux
package managers supported by the shell launcher are APT, DNF, Pacman, and
Zypper. On macOS, Apple requires the Command Line Tools dialog to be completed
once before the script can continue.

## Platform notes

Kallyup runs anywhere Cargo and Git are available: Windows, macOS, Linux, and
Nix environments. It installs each component from its `main` Git branch. Use
`--root DIR` for a user-managed location; add `DIR/bin` to `PATH` afterwards.
For reproducible releases, use the platform packages or Nix flakes documented
in the Kalcite workspace installation guide.
