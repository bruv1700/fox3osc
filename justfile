set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]
default_features := "15tet 17tet 19tet 22tet 23tet 24tet"

build features=default_features $RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none": fetch clippy
    cargo +nightly build \
    -Z build-std=std,panic_abort \
    -Z build-std-features="optimize_for_size" \
    -Z build-std-features=panic_immediate_abort \
    --features "{{features}}" \
    --release
# --frozen is weird with -Z build-std

build-debug features=default_features: fetch-debug clippy-debug
    cargo build \
    --features "{{features}}" \
    --frozen

fetch:
    rustup component add rust-src --toolchain nightly
    cargo +nightly fetch

fetch-debug:
    cargo fetch

clippy: fetch
    cargo +nightly clippy -- -Dwarnings

clippy-debug: fetch-debug
    cargo clippy -- -Dwarnings

clean: fetch-debug
    cargo clean --frozen

clap_path := if os() == "windows" {
    `[System.Environment]::ExpandEnvironmentVariables("%LOCALAPPDATA%") + "\Programs\Common\CLAP"`
} else {
    "$HOME/.clap"
}

[windows]
install: clap_folder (_uninstall "pdb")
    Copy-Item -Path "target\release\fox3osc.dll" -Destination "{{clap_path}}\fox3osc.clap" -Force
    
[windows]
install-debug: clap_folder
    Copy-Item -Path "target\debug\fox3osc.dll" -Destination "{{clap_path}}\fox3osc.clap" -Force
    Copy-Item -Path "target\debug\fox3osc.pdb" -Destination "{{clap_path}}\fox3osc.pdb" -Force

[windows]
uninstall: (_uninstall "clap") (_uninstall "pdb")

[windows]
_uninstall extension:
    If (Test-Path "{{clap_path}}\fox3osc.{{extension}}") { \
    Remove-Item "{{clap_path}}\fox3osc.{{extension}}" \
    }

[private, windows]
clap_folder:
    New-Item -Path {{clap_path}} -Type Directory -Force > $null

[linux]
install: clap_folder (_install "release")

[linux]
install-debug: clap_folder (_install "debug")

[linux]
uninstall:
    rm -rf {{clap_path}}/fox3osc.clap

[private, linux]
clap_folder:
    mkdir -p {{clap_path}}

[linux]
_install profile:
    cp -f target/{{profile}}/libfox3osc.so {{clap_path}}/fox3osc.clap
