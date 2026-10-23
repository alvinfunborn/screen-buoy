//! Manual release discovery. Downloads and installation use Tauri's signed updater.
use serde::Serialize;
use std::time::Duration;

const RELEASES_PAGE: &str = "https://github.com/alvinfunborn/screen-buoy/releases/latest";
const MANIFEST: &str =
    "https://github.com/alvinfunborn/screen-buoy/releases/latest/download/latest.json";
const TAG_PATH: &str = "/alvinfunborn/screen-buoy/releases/tag/";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseStatus {
    current_version: String,
    version: String,
    notes: String,
    newer: bool,
    can_install: bool,
}

fn release_tag(url: &reqwest::Url) -> Result<&str, String> {
    if url.scheme() != "https" || url.host_str() != Some("github.com") {
        return Err("Unexpected GitHub release destination".into());
    }
    url.path()
        .strip_prefix(TAG_PATH)
        .filter(|tag| !tag.is_empty() && !tag.contains('/'))
        .ok_or_else(|| "GitHub did not resolve the latest release".into())
}

fn release_status(current: &str, tag: &str, can_install: bool) -> Result<ReleaseStatus, String> {
    let version = tag.strip_prefix('v').unwrap_or(tag);
    let current_version = semver::Version::parse(current).map_err(|e| e.to_string())?;
    let latest_version = semver::Version::parse(version)
        .map_err(|_| "GitHub returned an invalid release version".to_owned())?;
    if !latest_version.pre.is_empty() {
        return Err("GitHub returned a non-stable release".into());
    }
    Ok(ReleaseStatus {
        current_version: current.to_owned(),
        version: version.to_owned(),
        notes: String::new(),
        newer: latest_version > current_version,
        can_install,
    })
}

#[tauri::command]
pub async fn get_release_status(app: tauri::AppHandle) -> Result<ReleaseStatus, String> {
    discover_release(&app.package_info().version.to_string()).await
}

async fn discover_release(current: &str) -> Result<ReleaseStatus, String> {
    // The public latest-release redirect avoids the unauthenticated API quota.
    // No startup request, timer, credentials, or persistent background task.
    let client = reqwest::Client::builder()
        .user_agent(concat!("screen-buoy/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;
    let response = client
        .head(RELEASES_PAGE)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    let mut release = release_status(current, release_tag(response.url())?, false)?;
    if release.newer {
        let manifest = client
            .head(MANIFEST)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        // Only a missing manifest means a legacy/manual release. Network and
        // server failures must remain errors, never an "up to date" result.
        release.can_install = match manifest.status() {
            reqwest::StatusCode::NOT_FOUND => false,
            status if status.is_success() => true,
            status => return Err(format!("GitHub update manifest returned HTTP {status}")),
        };
    }
    Ok(release)
}

#[tauri::command]
pub fn open_releases_page() -> Result<(), String> {
    // Fixed destination: the frontend cannot run arbitrary programs or URLs.
    #[cfg(target_os = "macos")]
    let status = std::process::Command::new("/usr/bin/open")
        .arg(RELEASES_PAGE)
        .status();
    #[cfg(target_os = "windows")]
    let status = std::process::Command::new("explorer.exe")
        .arg(RELEASES_PAGE)
        .status();
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let status = std::process::Command::new("xdg-open")
        .arg(RELEASES_PAGE)
        .status();
    status.map_err(|e| e.to_string()).and_then(|status| {
        if status.success() {
            Ok(())
        } else {
            Err("Could not open GitHub Releases".into())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "manual smoke test against public GitHub; no API token or quota"]
    fn public_github_release_check() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let release = runtime
            .block_on(discover_release(env!("CARGO_PKG_VERSION")))
            .unwrap();
        assert!(semver::Version::parse(&release.version).is_ok());
        println!("Public GitHub version check: {release:?}");
    }

    #[test]
    fn legacy_release_is_current_without_an_update_manifest() {
        let status = release_status("1.2.3", "v1.2.3", false).unwrap();
        assert!(!status.newer);
        assert!(!status.can_install);
    }

    #[test]
    fn compares_semver_and_never_offers_a_downgrade() {
        assert!(release_status("1.2.9", "v1.2.10", true).unwrap().newer);
        assert!(!release_status("1.3.0", "v1.2.10", true).unwrap().newer);
        assert!(
            release_status("1.3.0-beta.1", "v1.3.0", true)
                .unwrap()
                .newer
        );
        let legacy = release_status("1.2.3", "v1.2.4", false).unwrap();
        assert!(legacy.newer && !legacy.can_install);
    }

    #[test]
    fn rejects_invalid_or_non_stable_releases() {
        assert!(release_status("1.2.3", "latest", true).is_err());
        assert!(release_status("1.2.3", "v2.0.0-beta.1", true).is_err());
    }

    #[test]
    fn only_accepts_the_expected_repository_release_redirect() {
        let valid =
            reqwest::Url::parse("https://github.com/alvinfunborn/screen-buoy/releases/tag/v1.2.3")
                .unwrap();
        assert_eq!(release_tag(&valid).unwrap(), "v1.2.3");
        for invalid in [
            RELEASES_PAGE,
            "https://github.com/other/repo/releases/tag/v1.2.3",
            "https://example.com/alvinfunborn/screen-buoy/releases/tag/v1.2.3",
        ] {
            assert!(release_tag(&reqwest::Url::parse(invalid).unwrap()).is_err());
        }
    }
}
