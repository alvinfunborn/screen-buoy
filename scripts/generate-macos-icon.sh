#!/bin/bash
# Regenerate the macOS assets using Apple's native icon silhouette.
# Requires Xcode with Icon Composer; checked-in assets are used by CI builds.
set -euo pipefail

project_dir="$(cd "$(dirname "$0")/.." && pwd)"
developer_dir="${DEVELOPER_DIR:-$(xcode-select -p)}"
ictool="$developer_dir/../Applications/Icon Composer.app/Contents/Executables/ictool"
if [[ ! -x "$ictool" ]]; then
  echo "Icon Composer is required to regenerate the macOS icon." >&2
  exit 1
fi

mkdir -p "$project_dir/logs/macos-icon"
work_dir="$(mktemp -d "$project_dir/logs/macos-icon/generate.XXXXXX")"
trap 'rm -rf "$work_dir"' EXIT
mkdir -p "$work_dir/ScreenBuoy.icon/Assets" "$work_dir/screen-buoy.iconset"

# Keep the existing artwork. Icon Composer places images at their pixel size.
sips -z 1024 1024 "$project_dir/src-tauri/icons/icon.png" \
  --out "$work_dir/ScreenBuoy.icon/Assets/icon.png" >/dev/null
cat > "$work_dir/ScreenBuoy.icon/icon.json" <<'EOF'
{
  "fill": {"solid": "srgb:0.04706,0.00000,0.25098,1.00000"},
  "groups": [{"layers": [{"image-name": "icon.png", "name": "Screen Buoy"}]}],
  "supported-platforms": {"squares": "shared"}
}
EOF

# Apple's 824px artwork sits inside a 1024px transparent canvas. Arbitrary
# rounded rectangles can trigger the system's extra silver backing plate.
case "$("$ictool" --help 2>&1)" in
  *--export-image*)
    "$ictool" "$work_dir/ScreenBuoy.icon" --export-image \
      --output-file "$work_dir/artwork.png" --platform macOS \
      --rendition Default --width 824 --height 824 --scale 1
    ;;
  *)
    "$ictool" "$work_dir/ScreenBuoy.icon" --export-preview \
      macOS Default 824 824 1 -45 "$work_dir/artwork.png"
    ;;
esac
sips --padToHeightWidth 1024 1024 "$work_dir/artwork.png" \
  --out "$project_dir/src-tauri/icons/icon-macos.png" >/dev/null

for size in 16 32 128 256 512; do
  sips -z "$size" "$size" "$project_dir/src-tauri/icons/icon-macos.png" \
    --out "$work_dir/screen-buoy.iconset/icon_${size}x${size}.png" >/dev/null
  pixels=$((size * 2))
  sips -z "$pixels" "$pixels" "$project_dir/src-tauri/icons/icon-macos.png" \
    --out "$work_dir/screen-buoy.iconset/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$work_dir/screen-buoy.iconset" \
  -o "$project_dir/src-tauri/icons/icon.icns"
echo "Generated src-tauri/icons/icon-macos.png and icon.icns"
