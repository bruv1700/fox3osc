set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

build $RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none": fetch clippy
    cargo +nightly build \
    -Z build-std=std,panic_abort \
    -Z build-std-features="optimize_for_size" \
    -Z build-std-features=panic_immediate_abort \
    --release
# --frozen is weird with -Z build-std

build-debug: fetch-debug clippy-debug
    cargo build --frozen

fetch:
    rustup toolchain install nightly
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
} else if os() == "macos" {
    "$HOME/Library/Audio/Plug-ins/CLAP"
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

[linux, macos]
install: clap_folder (_install "release")

[linux, macos]
install-debug: clap_folder (_install "debug")

[linux, macos]
uninstall:
    rm -rf {{clap_path}}/fox3osc.clap

[private, linux]
clap_folder:
    mkdir -p {{clap_path}}

[linux]
_install profile:
    cp -f target/{{profile}}/libfox3osc.so {{clap_path}}/fox3osc.clap

[private, macos]
clap_folder:
    mkdir -p {{clap_path}}/fox3osc.clap/Contents/MacOS

[macos]
_install profile: contents_plist
    cp -f target/{{profile}}/libfox3osc.dylib {{clap_path}}/fox3osc.clap/Contents/MacOS

[private, macos]
contents_plist:
    echo -e \
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n" \
    "<"'!'"DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n" \
    "<plist version=\"1.0\">\n" \
    "<dict>\n" \
    "    <key>CFBundleName</key>\n" \
    "    <string>fox3osc</string>\n" \
    "    <key>CFBundleExecutable</key>\n" \
    "    <string>libfox3osc.dylib</string>\n" \
    "    <key>CFBundleIdentifier</key>\n" \
    "    <string>com.bruvy.fox3osc</string>\n" \
    "</dict>\n" \
    "</plist>" > {{clap_path}}/fox3osc.clap/Contents/contents.plist
