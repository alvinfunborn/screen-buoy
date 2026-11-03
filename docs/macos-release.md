# macOS build and release

## Local candidate

```sh
npm ci
npm run check:version
rustup target add aarch64-apple-darwin x86_64-apple-darwin
APPLE_SIGNING_IDENTITY=- npm run tauri build -- --target universal-apple-darwin --bundles app
cargo test --locked --manifest-path src-tauri/Cargo.toml --lib
```

Output: `src-tauri/target/universal-apple-darwin/release/bundle/macos/ScreenBuoy.app`.

Verify the actual artifact:

```sh
APP=src-tauri/target/universal-apple-darwin/release/bundle/macos/ScreenBuoy.app
lipo "$APP/Contents/MacOS/screen-buoy" -verify_arch arm64 x86_64
codesign --verify --deep --strict "$APP"
/usr/libexec/PlistBuddy -c 'Print CFBundleShortVersionString' "$APP/Contents/Info.plist"
ditto -c -k --keepParent "$APP" screen-buoy-macos.zip
```

An ad-hoc signature provides no Developer ID identity and no notarization. The current tag workflow deliberately produces this development/community distribution when credentials are absent. Do not call it notarized or Gatekeeper-approved.

## Developer ID distribution

Use a **Developer ID Application** certificate with its private key in the build machine's keychain. Apple Development and Apple Distribution certificates are not substitutes for this distribution identity.

Set `APPLE_SIGNING_IDENTITY` to that identity. Supply notarization credentials using either:

- `APPLE_ID`, `APPLE_PASSWORD` (app-specific password), and `APPLE_TEAM_ID`; or
- `APPLE_API_ISSUER`, `APPLE_API_KEY`, and `APPLE_API_KEY_PATH`.

Then run the same universal Tauri build without the `APPLE_SIGNING_IDENTITY=-` override. Store credentials in the runner's secret store, never in the repository. Before uploading the artifact, validate the signature, stapled ticket, and Gatekeeper assessment:

```sh
codesign --verify --deep --strict "$APP"
xcrun stapler validate "$APP"
spctl --assess --type execute --verbose=2 "$APP"
```

A signed release remains blocked until those validations succeed. Reference: [Tauri macOS signing and notarization](https://v2.tauri.app/distribute/sign/macos/).

## Versions and installation

Update `package.json`, both root versions in `package-lock.json`, `src-tauri/Cargo.toml`, the `screen-buoy` entry in `Cargo.lock`, and `src-tauri/Tauri.toml` together. `npm run check:version` checks consistency; tag CI also checks `v<version>` against the bundle version.

For 1.2.1+, place the `.app` in `/Applications`. Release preferences live at `~/Library/Application Support/com.screen-buoy.dev/config_macos.toml`. On first run, a legacy sidecar configuration beside the `.app` is copied there if present; otherwise embedded macOS defaults are used. An existing user config is never overwritten by app replacement. Development mode continues to use the repository config.

Replacing an ad-hoc signed binary may require re-adding the installed app in Privacy & Security permissions. The settings UI shows the current executable path so the user can identify the correct copy.

For repeated local installations, use the same valid signing certificate instead of `APPLE_SIGNING_IDENTITY=-`. Inspect the installed requirement with `codesign -d -r- /Applications/ScreenBuoy.app` and available certificates with `security find-identity -v -p codesigning`. Set `APPLE_SIGNING_IDENTITY` to the same certificate name or fingerprint on every build. Apple Development is suitable for local testing, but does not provide Developer ID distribution or notarization. Switching away from the old ad-hoc identity may require one permission migration; do not reset TCC grants on every update.

Install a **release** bundle. A `--debug` bundle uses the development working-directory configuration and can exit with `please check the config file: config.toml exists` when launched from `/Applications`. Release builds load the user configuration described above. After replacing the app, validate the installed process's Accessibility and keyboard-hook status, not just the toggles shown in System Settings.

The application name is `ScreenBuoy`; its bundle identifier remains
`com.screen-buoy.dev` and its executable remains `screen-buoy`. Install it as
`/Applications/ScreenBuoy.app`. When replacing an older `screen-buoy.app`, migrate
the bundle filename as well: Finder, Spotlight, and launchers can keep showing
`screen-buoy` when only the internal bundle name changes. Stop the old instance,
move the verified replacement to `ScreenBuoy.app`, refresh its LaunchServices
registration, relaunch, and verify the login item targets the new path. Preserve
the existing user configuration and a recoverable backup of the old app.
The updater replaces the app at its existing installation path and does not
perform this one-time filename migration automatically.
On startup, the app applies the saved login-start preference to the `ScreenBuoy`
LaunchAgent and then removes the legacy `screen-buoy` LaunchAgent.
Keeping the identifier and installation path does not guarantee permission
continuity with ad-hoc signatures. Validate that separately across two installed
versions. Community releases continue to use ad-hoc signing and manual updates,
with no proactive update reminders or forced upgrades. Developer ID signing is
optional if the distribution policy changes later.

## GitHub release procedure

The tag workflow builds and tests Windows and macOS independently. Only after both jobs succeed does a single job create a draft containing both archives, signed updater artifacts, `latest.json` and `SHA256SUMS`. This prevents a partial public release and duplicate generated notes. Configure the updater signing secrets before tagging; see [App updates](app-updates.md).

Before publishing the draft, verify the tagged commit, download both archives, check their checksums and executable versions, and inspect the macOS universal binary and signature. Add release notes that distinguish tested runtime behavior from unverified hardware/OS combinations, then publish the draft as the latest release.
