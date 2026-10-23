# App updates through GitHub Releases

System settings provides manual checks, release notes, signed download/install,
and a restart action. Nothing checks the network at startup or on a timer.
The current version comes from the running binary. Discovery uses the public
`releases/latest` redirect, avoiding the anonymous REST API quota. GitHub's latest stable release
is compared using SemVer; equal/older versions never offer a downgrade. Releases
without an updater manifest still work through the GitHub download button.
Network, manifest, download, signature and installation failures remain errors,
not an "up to date" result. Notes are displayed as text, not executable HTML.

## Signing and building

Tauri updater signatures are separate from macOS Developer ID signing and
notarization. A signed update does not resolve the current ad-hoc macOS app's
Accessibility permission reset on replacement.

The public key is committed in `src-tauri/Tauri.toml`. Keep the matching private
key outside the repository and back it up securely: losing it prevents updates
to apps trusting this public key. This Mac's initial key is at
`~/.tauri/screen-buoy-updater.key` with mode 0600.

GitHub Actions requires repository secret `TAURI_SIGNING_PRIVATE_KEY` (the key's
contents) and optionally `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. The initial key has
an empty password; the password secret may be absent. Do not generate a new key
on each CI run. An ordinary local build needs no signing key; only artifact
creation opts in with the updater config:

```sh
TAURI_SIGNING_PRIVATE_KEY="$HOME/.tauri/screen-buoy-updater.key" \
TAURI_SIGNING_PRIVATE_KEY_PASSWORD='' APPLE_SIGNING_IDENTITY=- \
npm run tauri build -- --target universal-apple-darwin --bundles app \
  --config src-tauri/tauri.updater.conf.json
```

Keep JS and Rust `tauri-plugin-updater` on the same tested version. Version 2.8.1
matches the repository's existing Tauri 2.5 backend; upgrading Tauri is a separate
change. Official reference: <https://v2.tauri.app/plugin/updater/>.

## Release assets and publication

The tag workflow preserves the two-platform build/test gate and creates a draft
only after both jobs finish. It includes the existing macOS/Windows ZIPs plus:

- `screen-buoy-macos.app.tar.gz` and `.sig`: one universal app for both Mac architectures.
- `screen-buoy-windows-setup.exe` and `.sig`: NSIS installer for x86_64 Windows.
- `latest.json`: stable version, release URL, and inline signatures for all three targets.
- `SHA256SUMS`: both ZIPs, both updater packages and the manifest.

`scripts/create-update-manifest.mjs` refuses incomplete artifacts or signatures.
Inspect the draft and verify downloads before publishing it as latest. Publishing
the draft makes the static GitHub endpoint discoverable; no server is needed.
Never publish a manifest before all referenced assets are uploaded.

Existing 1.2.3 apps do not contain an updater and require one manual upgrade to
a release containing this feature. Do not claim an end-to-end public update test
until two updater-enabled versions have been installed in sequence. Windows
installer/runtime validation requires Windows CI and a Windows desktop.

## Configuration survival

Release builds keep writable settings outside the app installation:

- macOS: `~/Library/Application Support/com.screen-buoy.dev/config_macos.toml`
- Windows: `%APPDATA%/com.screen-buoy.dev/config.toml`

First run migrates an existing sidecar; otherwise it writes embedded platform
defaults. Existing user files are never overwritten. The Windows ZIP remains
available, but its new release binary also migrates to the user directory.
Development builds retain repository/sidecar lookup.

New installations use a 250 ms completion delay, 60000 ms cache TTL, logging
`none`, debug off, start-in-tray/show-tray on, start-at-login off, and Alt+H.
These defaults do not overwrite upgraded users' choices.

## Validation

```sh
npm run build
npm run test:updater
npm run test:overlay
npm run check:version
cargo test --locked --manifest-path src-tauri/Cargo.toml --lib
```

Manual acceptance covers current-version checks, a newer release, timeout,
absent manifest, failed download/signature, installation and relaunch.
Tests cover version ordering, stable releases, legacy releases, complete release
manifests, and configuration migration/preservation. A successful same-version
check alone does not prove installation or failure recovery.

A live discovery smoke test is opt-in: `cargo test --locked --manifest-path src-tauri/Cargo.toml --lib updates::tests::public_github_release_check -- --ignored --nocapture`.
