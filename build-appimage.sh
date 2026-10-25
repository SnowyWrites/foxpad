#!/bin/sh
set -e

cargo build --release

rm -rf FoxPad.AppDir
mkdir -p FoxPad.AppDir/usr/bin
cp target/release/foxpad FoxPad.AppDir/usr/bin/
cp assets/foxpad.png FoxPad.AppDir/foxpad.png
ln -s foxpad.png FoxPad.AppDir/.DirIcon

cat > FoxPad.AppDir/foxpad.desktop <<'EOF'
[Desktop Entry]
Type=Application
Name=FoxPad
Exec=foxpad
Icon=foxpad
Categories=Utility;TextEditor;
StartupWMClass=foxpad
EOF

cat > FoxPad.AppDir/AppRun <<'EOF'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
exec "$HERE/usr/bin/foxpad" "$@"
EOF
chmod +x FoxPad.AppDir/AppRun

ARCH=x86_64 ./appimagetool-x86_64.AppImage FoxPad.AppDir FoxPad-x86_64.AppImage
echo "Built FoxPad-x86_64.AppImage"
