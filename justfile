#!/usr/bin/env just --justfile

main_crate := 'bindgen_helpers'
# How to call the current just executable. Note that just_executable() may have `\` in Windows paths, so we need to quote it.
just := quote(just_executable())
# cargo-binstall needs a workaround due to caching when used in CI
binstall_args := if env('CI', '') != '' {'--no-confirm --no-track --disable-telemetry'} else {''}

# if running in CI, treat warnings as errors by setting CARGO_BUILD_WARNINGS to 'deny' unless it is already set
# Use `CI=true just ci-test` to run the same tests as in GitHub CI.
# Use `just env-info` to see the current value of CARGO_BUILD_WARNINGS
ci_mode := if env('CI', '') != '' {'1'} else {''}
export CARGO_BUILD_WARNINGS := env('CARGO_BUILD_WARNINGS', if ci_mode == '1' {'deny'} else {'warn'})
export RUST_BACKTRACE := env('RUST_BACKTRACE', if ci_mode == '1' {'1'} else {'0'})
# Where `dist` puts the release asset: bindgen-helper-<target>.tar.gz with the binary and the licenses, as
# cli/Cargo.toml's binstall metadata expects
dist_dir := 'target/dist'
# Cargo lock behavior - enabled in CI.
locked := if env('CI', '') != '' {'--locked'} else {''}

@_default:
    {{just}} --list

# Run integration tests and save its output as the new expected output
bless *args:  (cargo-install 'cargo-insta')
    cargo insta test --accept --unreferenced=delete -p bindgen_helpers_tests {{args}}

# Build the project
build:
    cargo build {{locked}} --workspace --all-features --all-targets

# Quick compile without building a binary. Also checks the library with its default features, i.e. without `config`.
# Not with `--no-default-features`: without bindgen's `runtime`, clang-sys links libclang at build time, which CI lacks.
check *args:
    cargo check {{locked}} --workspace --all-features --all-targets {{args}}
    cargo check {{locked}} --package {{main_crate}} --all-targets

# Generate code coverage report to upload to codecov.io
ci-coverage: env-info && \
            (coverage '--codecov --output-path target/llvm-cov/codecov.info')
    # ATTENTION: the full file path above is used in the CI workflow
    mkdir -p target/llvm-cov

# Run all checks locally. Cargo.lock may be created or updated.
local-test: env-info test-fmt clippy check test test-doc

# Run all tests as expected by CI, using the committed Cargo.lock.
ci-test:
    {{just}} local-test
    {{just}} assert-git-is-clean

# Check that everything but the CLI, which has its own newer MSRV, compiles with the MSRV and the minimal dependency
# versions in Cargo.msrv.lock. Fails if Cargo.msrv.lock would change.
ci-test-msrv:
    RUSTUP_TOOLCHAIN="$({{just}} get-msrv)" {{just}} _use_msrv {{just}} env-info _check-msrv _assert-msrv-lock
    {{just}} assert-git-is-clean

# Check everything the MSRV applies to: all crates but the CLI.
_check-msrv: (check '--exclude' 'bindgen_helpers_cli')

# Run MSRV check locally and copy resolved lockfile changes back to Cargo.msrv.lock.
msrv-check:
    {{just}} _use_msrv {{just}} _check-msrv _save-msrv-lock

# Run a command with Cargo.msrv.lock copied to Cargo.lock, then restore the original Cargo.lock.
_use_msrv *args:
    #!/usr/bin/env bash
    set -euo pipefail
    command=({{args}})
    if [ ${#command[@]} -eq 0 ]; then
        >&2 echo 'ERROR: _use_msrv requires a command to run'
        exit 1
    fi
    backup=''
    trap 'rm -f Cargo.lock; if [ -n "$backup" ]; then mv "$backup" Cargo.lock; fi' EXIT
    if [ -f Cargo.lock ]; then
        backup="$(mktemp Cargo.lock.bak.XXXXXX)"
        mv Cargo.lock "$backup"
    fi
    cp Cargo.msrv.lock Cargo.lock
    {{just}} _sync-lock-versions Cargo.lock
    "${command[@]}"

# Set the versions of the workspace's own crates in `lockfile` to those in their Cargo.toml. A release PR bumps them in
# Cargo.toml and Cargo.lock but not in Cargo.msrv.lock, which `--locked` would then reject.
_sync-lock-versions lockfile:  (assert-cmd 'jq')
    #!/usr/bin/env bash
    set -euo pipefail
    VERSIONS="$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | "\(.name) \(.version)"')" \
        awk 'BEGIN { n = split(ENVIRON["VERSIONS"], lines, "\n"); for (i = 1; i <= n; i++) { split(lines[i], kv, " "); ver[kv[1]] = kv[2] } }
            /^name = / { name = $3; gsub(/"/, "", name); member = (name in ver) }
            /^version = / && member { $0 = "version = \"" ver[name] "\""; member = 0 }
            { print }' {{quote(lockfile)}} > {{quote(lockfile + '.tmp')}}
    mv {{quote(lockfile + '.tmp')}} {{quote(lockfile)}}

# Check that Cargo.msrv.lock is identical to Cargo.lock, apart from the workspace crates' own versions, and fail if not.
_assert-msrv-lock:
    #!/usr/bin/env bash
    set -euo pipefail
    expected="$(mktemp Cargo.msrv.lock.XXXXXX)"
    trap 'rm -f "$expected"' EXIT
    cp Cargo.msrv.lock "$expected"
    {{just}} _sync-lock-versions "$expected"
    if ! cmp -s Cargo.lock "$expected"; then
        >&2 echo 'ERROR: Cargo.msrv.lock is not up to date. Run `just msrv-check` locally.'
        diff -u "$expected" Cargo.lock || true
        exit 1
    fi

# Copy Cargo.lock to Cargo.msrv.lock if they differ. This is used to save changes to the lockfile caused by MSRV check.
_save-msrv-lock:
    @if ! cmp -s Cargo.lock Cargo.msrv.lock; then \
        >&2 echo 'Updating Cargo.msrv.lock' ;\
        cp Cargo.lock Cargo.msrv.lock ;\
    fi

# Clean all build artifacts
clean:
    cargo clean

# Run cargo clippy to lint the code
clippy *args:
    cargo clippy {{locked}} --workspace --all-features --all-targets {{args}}

# Generate code coverage report. Will install `cargo llvm-cov` if missing.
coverage *args='--open':  (cargo-install 'cargo-llvm-cov')
    cargo llvm-cov clean --workspace
    cargo llvm-cov --workspace --all-features --all-targets --include-build-script {{args}}

# Build the CLI's release binary for `target`, and pack it as the release asset in `target/dist`.
# The binary is `bindgen-helper.exe` on Windows; every target's archive is a `.tar.gz`, as binstall expects.
dist target:
    rustup target add {{quote(target)}}
    cargo build --release --locked --package bindgen_helpers_cli --bin bindgen-helper --target {{quote(target)}}
    rm -rf {{quote(dist_dir / 'bindgen-helper-' + target)}}* && mkdir -p {{quote(dist_dir / 'bindgen-helper-' + target)}}
    cp {{quote('target' / target / 'release/bindgen-helper' + if target =~ 'windows' { '.exe' } else { '' })}} LICENSE-MIT LICENSE-APACHE {{quote(dist_dir / 'bindgen-helper-' + target)}}/
    cd {{quote(dist_dir / 'bindgen-helper-' + target)}} && tar czf {{quote('../bindgen-helper-' + target + '.tar.gz')}} {{quote('bindgen-helper' + if target =~ 'windows' { '.exe' } else { '' })}} LICENSE-MIT LICENSE-APACHE

# Build and open code documentation
docs *args='--open':
    DOCS_RS=1 cargo doc {{locked}} --no-deps {{args}} --workspace --all-features

# Print environment info
env-info:
    @echo "Running for '{{main_crate}}' crate {{if ci_mode == '1' {'in CI mode'} else {'in dev mode'} }} on {{os()}} / {{arch()}}"
    @echo "PWD {{justfile_directory()}}"
    {{just}} --version
    rustc --version
    cargo --version
    rustup --version
    @echo "CARGO_BUILD_WARNINGS='$CARGO_BUILD_WARNINGS'"
    @echo "RUST_BACKTRACE='$RUST_BACKTRACE'"

# Reformat all code `cargo fmt`. If nightly is available, use it for better results
fmt:
    #!/usr/bin/env bash
    set -euo pipefail
    if (rustup toolchain list | grep nightly && rustup component list --toolchain nightly | grep rustfmt) &> /dev/null; then
        echo 'Reformatting Rust code using nightly Rust fmt to sort imports'
        cargo +nightly fmt --all -- --config imports_granularity=Module,group_imports=StdExternalCrate
    else
        echo 'Reformatting Rust with the stable cargo fmt.  Install nightly with `rustup install nightly` for better results'
        cargo fmt --all
    fi

# Reformat all Cargo.toml files using cargo-sort
fmt-toml *args:  (cargo-install 'cargo-sort')
    cargo sort --workspace --grouped {{args}}

# Get a package field from the metadata
get-crate-field field package=main_crate:  (assert-cmd 'jq')
    cargo metadata --no-deps --format-version 1 | jq -e -r '.packages | map(select(.name == "{{package}}")) | first | .{{field}} // error("Field \"{{field}}\" is missing in Cargo.toml for package {{package}}")'

# Get the minimum supported Rust version (MSRV) for the crate
get-msrv package=main_crate:  (get-crate-field 'rust_version' package)

# Find the minimum supported Rust version (MSRV) using cargo-msrv extension, and update Cargo.toml
msrv:  (cargo-install 'cargo-msrv')
    cargo msrv find --write-msrv -- {{just}} ci-test-msrv

# Initialize Cargo.msrv.lock with the minimal versions of dependencies.
# Cargo.lock is set aside and restored afterwards. Run `just msrv-check` afterwards to add the dev-dependencies.
msrv-init:  (cargo-install 'cargo-minimal-versions')
    #!/usr/bin/env bash
    set -euo pipefail
    backup="$(mktemp Cargo.lock.bak.XXXXXX)"
    mv Cargo.lock "$backup"
    trap 'mv "$backup" Cargo.lock' EXIT
    rm -f Cargo.msrv.lock
    if ! cargo minimal-versions check --workspace ; then
        echo "ERROR: Could not generate minimal Cargo.msrv.lock"
        echo "       fix it with 'cargo update ... --precise ...' until 'just check' passes,"
        echo "       then rename Cargo.lock to Cargo.msrv.lock"
        trap - EXIT
        echo "       The original Cargo.lock is in $backup"
        exit 1
    fi
    mv Cargo.lock Cargo.msrv.lock

# Run cargo-release
release *args='':  (cargo-install 'release-plz')
    release-plz {{args}}

# Check semver compatibility with prior published version. Install it with `cargo install cargo-semver-checks`
semver *args:  (cargo-install 'cargo-semver-checks')
    cargo semver-checks --all-features {{args}}

# Run all unit and integration tests
test:
    cargo test {{locked}} --workspace --all-features --all-targets
    cargo test {{locked}} --doc --workspace --all-features

# Test documentation generation
test-doc:  (docs '')

# Test code formatting
test-fmt: && (fmt-toml '--check' '--check-format')
    cargo fmt --all -- --check

# Find unused dependencies. Uses `cargo-udeps`
udeps:  (cargo-install 'cargo-udeps')
    cargo +nightly udeps --workspace --all-features --all-targets

# Update all dependencies, including breaking changes. Requires nightly toolchain (install with `rustup install nightly`)
update:  (cargo-install 'cargo-upgrade' 'cargo-edit')
    cargo upgrade --incompatible --compatible ignore
    cargo update

# Ensure that a certain command is available
[private]
assert-cmd command:
    @if ! type {{command}} > /dev/null; then \
        echo "Command '{{command}}' could not be found. Please make sure it has been installed on your computer." ;\
        exit 1 ;\
    fi

# Make sure the git repo has no uncommitted changes
[private]
assert-git-is-clean:
    @if [ -n "$(git status --untracked-files --porcelain)" ]; then \
        >&2 echo "ERROR: git repo is no longer clean. Make sure compilation and tests artifacts are in the .gitignore, and no repo files are modified." ;\
        >&2 echo "######### git status ##########" ;\
        git status ;\
        git --no-pager diff ;\
        exit 1 ;\
    fi

# Check if a certain Cargo command is installed, and install it if needed
[private]
cargo-install $COMMAND $INSTALL_CMD='' *args='':
    #!/usr/bin/env bash
    set -euo pipefail
    unset CARGO_BUILD_WARNINGS
    if ! command -v $COMMAND > /dev/null; then
        echo "$COMMAND could not be found. Installing..."
        if ! command -v cargo-binstall > /dev/null; then
            set -x
            cargo install ${INSTALL_CMD:-$COMMAND} --locked {{args}}
            { set +x; } 2>/dev/null
        else
            set -x
            cargo binstall ${INSTALL_CMD:-$COMMAND} {{binstall_args}} --locked
            { set +x; } 2>/dev/null
        fi
    fi
