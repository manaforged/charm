#!/usr/bin/env bash
set -euo pipefail

quick=0
only=""
live=0
while [[ $# -gt 0 ]]; do
    case "$1" in
        --quick) quick=1; shift ;;
        --full) quick=0; shift ;;
        --live) live=1; shift ;;
        --only) only="${2:?--only needs a comma-separated gate list}"; shift 2 ;;
        *) echo "unknown flag: $1" >&2; exit 2 ;;
    esac
done

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"
if [[ -n "${USERPROFILE:-}" ]] && command -v cygpath >/dev/null 2>&1; then
    export PATH="$(cygpath "$USERPROFILE")/.cargo/bin:$PATH"
fi
export PATH="$HOME/.cargo/bin:$PATH"
if ! command -v cargo >/dev/null 2>&1 && command -v cargo.exe >/dev/null 2>&1; then
    cargo() { cargo.exe "$@"; }
fi
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$repo_root/target/verify}"

step() { printf '\n\033[1;34m== %s ==\033[0m\n' "$*"; }
ok()   { printf '\033[1;32m✓ %s\033[0m\n'  "$*"; }
fail() { printf '\033[1;31m✗ %s\033[0m\n'  "$*" >&2; exit 1; }
skipped=""
current_gate=""
skip() {
    [[ -z "${CI:-}" ]] || fail "$1"
    printf '  (%s; skipping)\n' "$1"
    [[ " $skipped " == *" $current_gate "* ]] || skipped="${skipped:+$skipped }$current_gate"
}

msrv="$(awk -F'"' '/^rust-version *= *"/{print $2; exit}' Cargo.toml)"
crate_version="$(awk -F'"' '/^\[/{section=$0} section=="[package]" && /^version *= *"/{print $2; exit}' Cargo.toml)"

g_comments() {
    step "rust comments (one-line rustdoc or // SAFETY:)"
    python3 scripts/check-comments.py || fail "comments must be one-line rustdoc or // SAFETY:"
    ok "comment lint"
}

g_fmt() {
    step "cargo fmt --all --check"
    cargo fmt --all -- --check || fail "rustfmt found formatting issues"
    ok "format clean"
}

g_clippy() {
    step "cargo clippy --all-targets -D warnings"
    cargo clippy --all-targets --no-deps -- -D warnings || fail "clippy produced warnings"
    ok "clippy clean"
}

g_test() {
    step "cargo test"
    cargo test || fail "tests failed"
    ok "tests pass"
}

g_msrv() {
    [[ -n "$msrv" ]] || fail "Cargo.toml has no rust-version"
    step "cargo +$msrv check"
    rustup toolchain list | grep -q "^$msrv" || rustup toolchain install "$msrv" --profile minimal \
        || fail "could not install Rust $msrv"
    cargo +"$msrv" check --all-targets || fail "MSRV cargo check failed"
    ok "MSRV $msrv compiles"
}

g_api() {
    step "cargo truesight check"
    if ! cargo truesight --help >/dev/null 2>&1; then
        skip "cargo-truesight not installed"
        return 0
    fi
    cargo truesight check || fail "run cargo truesight sync"
    ok "API reference current"
}

g_deny() {
    step "cargo deny check"
    if ! command -v cargo-deny >/dev/null 2>&1; then
        skip "cargo-deny not installed: cargo install --locked cargo-deny"
        return 0
    fi
    cargo deny check || fail "cargo-deny found issues"
    ok "cargo-deny clean"
}

g_live() {
    step "cargo test --test live -- --ignored (installed Chrome)"
    if [[ -n "${CHROME_EXE:-}" ]]; then
        echo "  CHROME_EXE=$CHROME_EXE"
    fi
    if [[ "$(uname -s)" == Linux && -z "${CHARM_CHROME_NO_SANDBOX:-}" ]]; then
        echo "  Linux: set CHARM_CHROME_NO_SANDBOX=1 when the user namespace sandbox is unavailable"
    fi
    cargo test --test live -- --ignored --nocapture || fail "live suite failed"
    ok "live pass"
}

g_release() {
    step "release qualification (tag, manifest, changelog)"
    local tag="${GITHUB_REF_NAME:-}"
    [[ -n "$tag" ]] || fail "GITHUB_REF_NAME is unset; set it to the release tag (v<semver>)"
    [[ "$tag" =~ ^v([0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?)$ ]] || fail "tag $tag is not v<semver>"
    local version="${BASH_REMATCH[1]}"
    [[ "$crate_version" == "$version" ]] || fail "tag $tag does not match Cargo.toml version $crate_version"
    grep -qE '^publish *= *false' Cargo.toml && fail "Cargo.toml sets publish = false"
    [[ -f CHANGELOG.md ]] || fail "CHANGELOG.md is missing"
    grep -qE "^## ${version//./\\.} - [0-9]{4}-[0-9]{2}-[0-9]{2}$" CHANGELOG.md \
        || fail "CHANGELOG.md has no '## $version - YYYY-MM-DD' heading"
    git fetch --quiet origin main 2>/dev/null || true
    git rev-parse --quiet --verify origin/main >/dev/null || fail "origin/main is unknown; fetch it before qualifying a release"
    git merge-base --is-ancestor HEAD origin/main || fail "tag $tag is on a commit that is not on main"
    mkdir -p "$CARGO_TARGET_DIR"
    awk -v head="## $version " 'index($0, head) == 1 {on = 1; next} on && /^## / {exit} on' CHANGELOG.md \
        >"$CARGO_TARGET_DIR/release-notes.md"
    ok "release $tag qualified"
}

g_package() {
    step "cargo package"
    [[ -z "$(git status --porcelain --untracked-files=no)" ]] || fail "the tree has uncommitted changes; packages build from HEAD"
    rm -rf "$CARGO_TARGET_DIR/package"
    cargo package || fail "cargo package failed"
    ok "crate packaged"
}

g_publish() {
    step "cargo publish"
    [[ -n "${CARGO_REGISTRY_TOKEN:-}" ]] || fail "CARGO_REGISTRY_TOKEN is unset; publish runs from the release workflow"
    [[ "${GITHUB_REF:-}" == refs/tags/v* ]] || fail "publish runs only from a v* tag"
    if curl -fsS "https://index.crates.io/ch/ar/charm-cdp" 2>/dev/null | grep -q "\"vers\":\"$crate_version\""; then
        ok "charm-cdp $crate_version is already on crates.io"
        return 0
    fi
    cargo publish || fail "cargo publish failed"
    ok "crate published"
}

gate_order=(comments fmt clippy test msrv api deny live release package publish)
quick_gates=(comments fmt clippy test)
full_gates=(comments fmt clippy test msrv api deny)

if [[ -n "$only" ]]; then
    IFS=',' read -ra want <<<"$only"
    for g in "${want[@]}"; do
        [[ " ${gate_order[*]} " == *" $g "* ]] \
            || { echo "unknown gate: $g (gates: ${gate_order[*]})" >&2; exit 2; }
    done
elif [[ $quick -eq 1 ]]; then
    want=("${quick_gates[@]}")
else
    want=("${full_gates[@]}")
fi
[[ $live -eq 1 ]] && want+=(live)

for g in "${gate_order[@]}"; do
    if [[ " ${want[*]} " == *" $g "* ]]; then
        current_gate="$g"
        "g_${g//-/_}"
    fi
done

if [[ -n "$skipped" ]]; then
    printf '\n\033[1;33mVerify incomplete: skipped %s\033[0m\n' "$skipped" >&2
    exit 2
fi

printf '\n\033[1;32mAll selected verify gates passed.\033[0m\n'
