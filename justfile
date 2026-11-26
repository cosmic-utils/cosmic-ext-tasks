name := 'tasks'
export APPID := 'dev.edfloreshz.Tasks'

rootdir := ''
prefix := '/usr'
flatpak-prefix := '/app'

base-dir := absolute_path(clean(rootdir / prefix))
flatpak-base-dir := absolute_path(clean(rootdir / flatpak-prefix))

export INSTALL_DIR := base-dir / 'share'

cargo-target-dir := env('CARGO_TARGET_DIR', 'target')
bin-src := cargo-target-dir / 'release' / name
bin-dst := base-dir / 'bin' / name
flatpak-bin-dst := flatpak-base-dir / 'bin' / name

desktop := APPID + '.desktop'
desktop-src := cargo-target-dir / 'xdgen' / 'app.desktop'
desktop-dst := clean(rootdir / prefix) / 'share' / 'applications' / desktop
flatpak-desktop-dst := clean(rootdir / flatpak-prefix) / 'share' / 'applications' / desktop

metainfo := APPID + '.metainfo.xml'
metainfo-src := cargo-target-dir / 'xdgen' / 'app.metainfo.xml'
metainfo-dst := clean(rootdir / prefix) / 'share' / 'metainfo' / metainfo
flatpak-metainfo-dst := clean(rootdir / flatpak-prefix) / 'share' / 'metainfo' / metainfo

icon-dir := 'icons' / 'hicolor' / 'scalable' / 'apps'
icon-src := 'resources' / icon-dir / 'icon.svg'
icon-dst := base-dir / 'share' / icon-dir / (APPID + '.svg')
flatpak-icon-dst := flatpak-base-dir / 'share' / icon-dir / (APPID + '.svg')

# Default recipe which runs `just build-release`
default: build-release

# Runs `cargo clean`
clean:
    cargo clean

# Removes vendored dependencies
clean-vendor:
    rm -rf .cargo vendor vendor.tar

# `cargo clean` and removes vendored dependencies
clean-dist: clean clean-vendor

# Compiles with debug profile
build-debug *args:
    cargo build --locked {{args}}

# Compiles with release profile
build-release *args: (build-debug '--release' args)

# Compiles release profile with vendored dependencies
build-vendored *args: vendor-extract (build-release '--frozen --offline' args)

# Runs a clippy check
check *args:
    cargo clippy --all-features --locked {{args}} -- -W clippy::pedantic

# Runs a clippy check with JSON message format
check-json: (check '--message-format=json')

dev *args:
    cargo fmt
    just run {{args}}

# Run with debug logs
run *args:
    env RUST_LOG=tasks=info RUST_BACKTRACE=full cargo run --release --locked {{args}}

# Installs files
install:
    install -Dm0755 {{bin-src}} {{bin-dst}}
    install -Dm0644 {{desktop-src}} {{desktop-dst}}
    install -Dm0644 {{metainfo-src}} {{metainfo-dst}}
    install -Dm0644 {{icon-src}} {{icon-dst}}

# Installs files
flatpak:
    install -Dm0755 {{bin-src}} {{flatpak-bin-dst}}
    install -Dm0644 {{desktop-src}} {{flatpak-desktop-dst}}
    install -Dm0644 {{metainfo-src}} {{flatpak-metainfo-dst}}
    install -Dm0644 {{icon-src}} {{flatpak-icon-dst}}

# Bump the Cargo package version, commit it, and create an annotated tag.
tag version:
    #!/usr/bin/env bash
    set -euo pipefail
    version={{quote(version)}}

    if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$ ]]; then
        echo "Expected a Cargo version such as 1.2.3." >&2
        exit 1
    fi
    if git show-ref --verify --quiet "refs/tags/$version"; then
        echo "Tag $version already exists." >&2
        exit 1
    fi
    if [[ -n "$(git status --porcelain -- Cargo.toml Cargo.lock)" ]]; then
        echo "Commit or stash pending Cargo.toml and Cargo.lock changes before tagging." >&2
        exit 1
    fi

    sed -i "0,/^version = /s/^version = .*/version = \"$version\"/" Cargo.toml
    if ! cargo update --workspace; then
        git restore --worktree -- Cargo.toml Cargo.lock
        exit 1
    fi
    git commit --only -m "release: $version" -- Cargo.toml Cargo.lock
    git tag -a "$version" -m "Release $version"

# Regenerates the Flatpak dependency manifest from Cargo.lock.
flatpak-cargo-sources:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p .flatpak-builder
    scratch=$(mktemp -d .flatpak-builder/cargo-sources.XXXXXX)
    trap 'rm -rf "$scratch"' EXIT
    curl -fsSL https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py -o "$scratch/generator.py"
    python3 -m venv "$scratch/.venv"
    "$scratch/.venv/bin/python" -m pip install 'aiohttp>=3.9.5,<4' 'tomlkit>=0.13.3,<1'
    "$scratch/.venv/bin/python" "$scratch/generator.py" Cargo.lock -o "$scratch/cargo-sources.json"
    mv "$scratch/cargo-sources.json" flatpak/cargo-sources.json

# Builds an installable Flatpak bundle under dist.
flatpak-bundle *args:
    mkdir -p dist
    flatpak-builder --user --force-clean --repo=.flatpak-builder/repo {{args}} .flatpak-builder/build-dir flatpak/{{APPID}}.json
    flatpak build-bundle .flatpak-builder/repo dist/{{APPID}}.flatpak {{APPID}}

# Uninstalls installed files
uninstall:
    rm {{bin-dst}}
    rm {{desktop-dst}}
    rm {{metainfo-dst}}
    rm {{icon-dst}}

# Vendor dependencies locally
vendor:
    #!/usr/bin/env bash
    mkdir -p .cargo
    cargo vendor --sync Cargo.toml | head -n -1 > .cargo/config.toml
    echo 'directory = "vendor"' >> .cargo/config.toml
    echo >> .cargo/config.toml
    echo '[env]' >> .cargo/config.toml
    if [ -n "${SOURCE_DATE_EPOCH}" ]
    then
        source_date="$(date -d "@${SOURCE_DATE_EPOCH}" "+%Y-%m-%d")"
        echo "VERGEN_GIT_COMMIT_DATE = \"${source_date}\"" >> .cargo/config.toml
    fi
    if [ -n "${SOURCE_GIT_HASH}" ]
    then
        echo "VERGEN_GIT_SHA = \"${SOURCE_GIT_HASH}\"" >> .cargo/config.toml
    fi
    tar pcf vendor.tar .cargo vendor
    rm -rf .cargo vendor

# Extracts vendored dependencies
vendor-extract:
    rm -rf vendor
    tar pxf vendor.tar
