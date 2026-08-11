#!/usr/bin/env bash
set -euo pipefail

project_root="$(cd "$(dirname "$0")/.." && pwd)"
bundle_root="$project_root/src-tauri/target/release/bundle"
app_path="$(find "$bundle_root/macos" -maxdepth 1 -name '*.app' -print -quit)"
dmg_path="$(find "$bundle_root/dmg" -maxdepth 1 -name '*.dmg' -print -quit)"

[[ -n "$app_path" && -d "$app_path" ]] || { echo "macOS app bundle not found" >&2; exit 1; }
[[ -n "$dmg_path" && -f "$dmg_path" ]] || { echo "macOS DMG not found" >&2; exit 1; }

main_executable="$app_path/Contents/MacOS/trade-desk-local"
resource_dir="$app_path/Contents/Resources"
for required in \
  "$main_executable" \
  "$resource_dir/typst" \
  "$resource_dir/TYPST-LICENSE.txt" \
  "$resource_dir/TYPST-NOTICE.txt"; do
  [[ -f "$required" ]] || { echo "Missing bundle resource: $required" >&2; exit 1; }
done

[[ -x "$main_executable" ]] || { echo "Application binary is not executable" >&2; exit 1; }
[[ -x "$resource_dir/typst" ]] || { echo "Bundled Typst is not executable" >&2; exit 1; }

expected_version="$(node -p "require('$project_root/package.json').version")"
bundle_version="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$app_path/Contents/Info.plist")"
[[ "$bundle_version" == "$expected_version" ]] || {
  echo "Bundle version mismatch: $bundle_version / $expected_version" >&2
  exit 1
}

"$resource_dir/typst" --version
hdiutil verify "$dmg_path"
printf 'Verified TradeDesk %s: %s\n' "$bundle_version" "$dmg_path"
