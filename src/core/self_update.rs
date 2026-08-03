//! Self-update: fetch the latest GitHub release for this platform, verify the
//! archive against the release SHA256SUMS.txt, and replace the running binary.
//!
//! No third-party crate is used: the GitHub API + raw asset download +
//! checksum verification matches the project's existing supply-chain model.

use std::collections::BTreeMap;
use std::path::PathBuf;

use super::{CoreError, Result};

const REPO: &str = "oneinitAI/oneinit";
const UA: &str = "oneinit-self-update/1.0";

/// Check for updates and install the newest release.
/// Returns `Ok(true)` when an update was applied.
pub async fn run_self_update(formatter: &crate::output::OutputFormatter) -> Result<bool> {
    let current = env!("CARGO_PKG_VERSION");

    formatter.output(
        &format!("[UPDATE] Checking for updates... current v{}", current),
        None::<serde_json::Value>,
    );

    let latest = latest_release_tag().await?;
    if latest.trim_start_matches('v') == current {
        formatter.output(
            &format!("[OK] Already up to date (v{})", current),
            None::<serde_json::Value>,
        );
        return Ok(false);
    }
    formatter.output(
        &format!("[UPDATE] New version found: v{} → v{}", current, latest),
        None::<serde_json::Value>,
    );

    let asset_name = platform_asset(&latest)?;
    let base = format!("https://github.com/{REPO}/releases/download/{latest}");

    // Download the checksum file first — never install an unverified binary.
    let sums_text = fetch_text(&format!("{base}/SHA256SUMS.txt")).await?;
    let expected = parse_sums(&sums_text)
        .get(&asset_name)
        .cloned()
        .ok_or_else(|| CoreError::Other(format!("SHA256SUMS.txt has no entry for {asset_name}")))?;

    let archive_bytes = fetch_bytes(&format!("{base}/{asset_name}")).await?;
    let actual = sha256_hex(&archive_bytes);
    if actual != expected {
        return Err(CoreError::Checksum {
            file: asset_name.clone(),
            expected,
        });
    }
    formatter.output(
        &format!("[OK] SHA256 verified: {asset_name}"),
        None::<serde_json::Value>,
    );

    // Extract the archive and locate the binary
    let tmp_dir = std::env::temp_dir().join(format!("oneinit-update-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&tmp_dir)?;
    let archive_path = tmp_dir.join(&asset_name);
    std::fs::write(&archive_path, &archive_bytes)?;
    super::downloader::extract(&archive_path, &tmp_dir)?;

    let bin_name = if cfg!(target_os = "windows") {
        "oneinit.exe"
    } else {
        "oneinit"
    };
    let new_bin = find_binary(&tmp_dir, bin_name).ok_or_else(|| {
        CoreError::Other(format!("binary {bin_name} not found in release archive"))
    })?;

    replace_binary(&new_bin)?;
    let _ = std::fs::remove_dir_all(&tmp_dir);

    formatter.output(
        &format!("[OK] Updated to v{latest} — restart terminals to use it"),
        Some(serde_json::json!({
            "status": "success",
            "action": "self_update",
            "from": current,
            "to": latest,
        })),
    );
    Ok(true)
}

/// Latest release tag from the GitHub API (e.g. "v0.1.0-beta.2").
async fn latest_release_tag() -> Result<String> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let client = http_client()?;
    let resp = client
        .get(&url)
        .header("User-Agent", UA)
        .send()
        .await
        .map_err(|e| CoreError::Download(format!("GET {url} failed: {e}")))?;
    if !resp.status().is_success() {
        return Err(CoreError::Download(format!(
            "GET {url} -> HTTP {}",
            resp.status()
        )));
    }
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| CoreError::Download(format!("read latest release failed: {e}")))?;
    let json: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|e| CoreError::Download(format!("parse latest release failed: {e}")))?;
    json["tag_name"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| CoreError::Download("release has no tag_name".to_string()))
}

/// Platform asset name, e.g. `oneinit-v0.1.0-beta.2-windows-x86_64.zip`.
fn platform_asset(tag: &str) -> Result<String> {
    let (os, arch, ext) = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => ("windows", "x86_64", "zip"),
        ("linux", "x86_64") => ("linux", "x86_64", "tar.gz"),
        ("macos", "x86_64") => ("macos", "x86_64", "tar.gz"),
        ("macos", "aarch64") => ("macos", "aarch64", "tar.gz"),
        (os, arch) => {
            return Err(CoreError::Other(format!(
                "no pre-built binary for {os}/{arch} — build from source instead"
            )));
        }
    };
    Ok(format!("oneinit-{tag}-{os}-{arch}.{ext}"))
}

fn http_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| CoreError::Download(format!("build http client failed: {e}")))
}

async fn fetch_text(url: &str) -> Result<String> {
    let bytes = fetch_bytes(url).await?;
    String::from_utf8(bytes).map_err(|e| CoreError::Download(format!("{url} is not UTF-8: {e}")))
}

async fn fetch_bytes(url: &str) -> Result<Vec<u8>> {
    let client = http_client()?;
    let resp = client
        .get(url)
        .header("User-Agent", UA)
        .send()
        .await
        .map_err(|e| CoreError::Download(format!("GET {url} failed: {e}")))?;
    if !resp.status().is_success() {
        return Err(CoreError::Download(format!(
            "GET {url} -> HTTP {}",
            resp.status()
        )));
    }
    resp.bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| CoreError::Download(format!("read {url} failed: {e}")))
}

/// Parse SHA256SUMS.txt content → { filename: lowercase hex }.
pub fn parse_sums(text: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        if let (Some(hash), Some(name)) = (parts.next(), parts.next())
            && hash.len() == 64
        {
            map.insert(name.to_string(), hash.to_lowercase());
        }
    }
    map
}

fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(data);
    h.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}

/// Find the binary in an extracted archive (depth ≤ 3).
fn find_binary(dir: &std::path::Path, bin_name: &str) -> Option<PathBuf> {
    fn walk(dir: &std::path::Path, bin_name: &str, depth: usize) -> Option<PathBuf> {
        if depth > 3 {
            return None;
        }
        for entry in std::fs::read_dir(dir).ok()?.flatten() {
            let p = entry.path();
            if p.is_file() && p.file_name().map(|n| n == bin_name).unwrap_or(false) {
                return Some(p);
            }
            if p.is_dir()
                && let Some(found) = walk(&p, bin_name, depth + 1)
            {
                return Some(found);
            }
        }
        None
    }
    walk(dir, bin_name, 0)
}

/// Replace the running binary.
///
/// Unix: write to a temp file in the same dir, then rename over the current
/// executable (the running process keeps the old inode).
///
/// Windows: the running exe is locked, so write the new binary next to it and
/// launch a detached batch that swaps files after this process exits.
fn replace_binary(new_bin: &std::path::Path) -> Result<()> {
    let current_exe = std::env::current_exe()?;

    if cfg!(target_os = "windows") {
        let new_path = current_exe.with_extension("oneinit-new.exe");
        std::fs::copy(new_bin, &new_path)?;

        let bat_path = current_exe.with_extension("update.bat");
        let bat = format!(
            "@echo off\r\n\
             ping -n 3 127.0.0.1 > nul\r\n\
             del /f /q \"{old}\"\r\n\
             ren \"{new}\" \"{old}\"\r\n",
            old = current_exe.display(),
            new = new_path.display(),
        );
        std::fs::write(&bat_path, bat)?;
        // Detached: the batch waits ~2s, by which time this process has exited.
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", &bat_path.to_string_lossy()])
            .spawn();
        Ok(())
    } else {
        let tmp = current_exe.with_extension("oneinit-new");
        std::fs::copy(new_bin, &tmp)?;
        std::fs::rename(&tmp, &current_exe)?;
        // SAFETY: replacing our own binary is the point of this command
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&current_exe, std::fs::Permissions::from_mode(0o755));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sums() {
        let text = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  oneinit-v0.1.0-beta.2-linux-x86_64.tar.gz\n";
        let map = parse_sums(text);
        assert_eq!(
            map.get("oneinit-v0.1.0-beta.2-linux-x86_64.tar.gz")
                .unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn test_platform_asset() {
        let name = platform_asset("v0.1.0-beta.2").unwrap();
        assert!(name.starts_with("oneinit-v0.1.0-beta.2-"));
    }

    #[test]
    fn test_sha256_known() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
