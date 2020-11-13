@echo off

rustup toolchain install stable-i686-pc-windows-msvc || echo "RUSTUP NOT INSTALLED" && exit 1
cargo build --target i686-pc-windows-msvc --release || echo "BUILD FAILED" && exit 1
copy "target\release\m2_driver.dll" "C:\Program Files (x86)\macchina\passthru\driver.dll" || echo "FAILED TO COPY DLL" && exit 1

regedit.exe /S "%~dp0\driver.reg" || echo "FAILED TO MERGE REGISTRY" && exit 1

echo "Install complete!"