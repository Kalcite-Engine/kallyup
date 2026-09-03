#!/usr/bin/env sh
# Bootstrap Kallyup on Linux and macOS, then forward all arguments to it.
set -eu

KALLYUP_REPOSITORY="https://github.com/Kalcite-Engine/kallyup.git"
CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
export PATH="$CARGO_HOME/bin:$PATH"

fail() {
    printf '%s\n' "kallyup bootstrap: $*" >&2
    exit 1
}

run_privileged() {
    if [ "$(id -u)" -eq 0 ]; then
        "$@"
    elif command -v sudo >/dev/null 2>&1; then
        sudo "$@"
    else
        fail "administrator access is needed to install $1"
    fi
}

install_linux_requirements() {
    if command -v apt-get >/dev/null 2>&1; then
        run_privileged apt-get update
        run_privileged apt-get install -y git curl build-essential pkg-config
    elif command -v dnf >/dev/null 2>&1; then
        run_privileged dnf install -y git curl gcc gcc-c++ make pkgconf-pkg-config
    elif command -v pacman >/dev/null 2>&1; then
        run_privileged pacman -Sy --needed --noconfirm git curl base-devel pkgconf
    elif command -v zypper >/dev/null 2>&1; then
        run_privileged zypper --non-interactive install git curl gcc gcc-c++ make pkg-config
    else
        fail "unsupported Linux package manager; install Git, curl and a C build toolchain first"
    fi
}

has_c_compiler() {
    command -v cc >/dev/null 2>&1 || command -v clang >/dev/null 2>&1 || command -v gcc >/dev/null 2>&1
}

ensure_system_requirements() {
    if command -v git >/dev/null 2>&1 && command -v curl >/dev/null 2>&1 && has_c_compiler; then
        return
    fi

    case "$(uname -s)" in
        Linux) install_linux_requirements ;;
        Darwin)
            if ! command -v git >/dev/null 2>&1 || ! has_c_compiler; then
                xcode-select --install || true
                fail "macOS is opening the Command Line Tools installer; finish it, then run this script again"
            fi
            fail "install curl, then run this script again"
            ;;
        *) fail "this launcher supports Linux and macOS only" ;;
    esac
}

ensure_rust() {
    if command -v cargo >/dev/null 2>&1; then
        return
    fi
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
    export PATH="$CARGO_HOME/bin:$PATH"
    command -v cargo >/dev/null 2>&1 || fail "Rustup completed but Cargo was not found"
}

ensure_system_requirements
ensure_rust

cargo install --git "$KALLYUP_REPOSITORY" --branch main --locked --force

if [ "$#" -eq 0 ]; then
    set -- list
fi
exec "$CARGO_HOME/bin/kallyup" "$@"
