//! Checksum resolution for dynamic (non-exact-match) recipes.
//!
//! Sources tried in order: SQLite cache → official source (node SHASUMS256.txt,
//! python.org SPDX SBOM / pinned hashes, dl.google.com `.sha256` sidecar /
//! go.dev dl JSON) → embedded map.

use super::{CoreError, Result};

/// Pinned SHA256 for python.org Windows embeddable zips (amd64) that lack an
/// SPDX SBOM. Computed from the official files over TLS; 3.12.4 was
/// cross-checked against both its SPDX SBOM ("CPython" package) and the MD5
/// published on the release page. python.org retired its JSON checksum API and
/// never shipped SHA256SUMS, so these cover the embedded catalog's older
/// series.
const PYTHON_EMBEDDED_SHA256: &[(&str, &str)] = &[
    (
        "3.11.9",
        "009d6bf7e3b2ddca3d784fa09f90fe54336d5b60f0e0f305c37f400bf83cfd3b",
    ),
    (
        "3.12.4",
        "15fea3c9367653a85086fe37216b4d1a1c78688fa5e1587e1db0b0f658856564",
    ),
    (
        "3.13.0",
        "01c32d0737432240adcf0bbc1d32327f0976d3a1e1427774bc8febc8f1c03111",
    ),
];

/// Platform names used by the resolver (mirror the recipe platform keys).
pub fn platform_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else {
        "linux"
    }
}

/// Resolve the SHA256 checksum for a dynamic recipe file.
///
/// `filename` is the archive's basename; `url` is used to pick the right
/// official source. Returns None when unavailable and `--no-checksum` is set.
pub async fn resolve(
    recipe: &str,
    version: &str,
    filename: &str,
    refresh: bool,
) -> Result<Option<String>> {
    let platform = platform_name();

    // 1. cache (unless --refresh)
    if !refresh
        && let Some(h) = super::cache_db::cached_checksum(recipe, version, platform, filename)
    {
        return Ok(Some(h));
    }

    // 2. official sources
    let fetched = match recipe {
        "node" => node_checksum(version, filename).await?,
        "python" => python_checksum(version, filename).await?,
        "go" => go_checksum(version, filename).await?,
        _ => None,
    };

    if let Some(h) = fetched {
        super::cache_db::cache_checksum(recipe, version, platform, filename, &h, "official")?;
        return Ok(Some(h));
    }

    Ok(None)
}

/// nodejs.org SHASUMS256.txt — one file per version, lists every asset.
async fn node_checksum(version: &str, filename: &str) -> Result<Option<String>> {
    let url = format!("https://nodejs.org/dist/v{version}/SHASUMS256.txt");
    let text = fetch_text(&url).await?;
    Ok(parse_shasums(&text, filename))
}

/// python.org release checksums.
///
/// The JSON API and SHA256SUMS were retired; the official sources left are the
/// per-file SPDX SBOM (3.12+) — the archive's own hash sits under the
/// "CPython" package — and pinned hashes for the catalog's amd64 zips.
async fn python_checksum(version: &str, filename: &str) -> Result<Option<String>> {
    // 1. per-file SPDX SBOM: .../{filename}.spdx.json
    let spdx_url = format!("https://www.python.org/ftp/python/{version}/{filename}.spdx.json");
    if let Ok(bytes) = fetch_bytes(&spdx_url).await
        && let Ok(json) = serde_json::from_slice::<serde_json::Value>(&bytes)
        && let Some(pkgs) = json["packages"].as_array()
    {
        for p in pkgs {
            if p["name"].as_str() == Some("CPython")
                && let Some(cs) = p["checksums"].as_array()
            {
                for c in cs {
                    if c["algorithm"].as_str() == Some("SHA256")
                        && let Some(h) = c["checksumValue"].as_str()
                    {
                        return Ok(Some(h.to_lowercase()));
                    }
                }
            }
        }
    }

    // 2. pinned fallback for the embedded catalog's amd64 embeddable zips
    //    (pre-3.12 releases have no SBOM)
    if filename.ends_with("-embed-amd64.zip") {
        for (v, h) in PYTHON_EMBEDDED_SHA256 {
            if *v == version {
                return Ok(Some(h.to_string()));
            }
        }
    }

    Ok(None)
}

/// go — dl.google.com serves a per-file `.sha256` sidecar for every release
/// file (older versions drop out of the dl JSON index but keep their sidecars),
/// so the sidecar is the primary source; the dl JSON is the fallback.
async fn go_checksum(version: &str, filename: &str) -> Result<Option<String>> {
    // 1. per-file sidecar on the CDN: plain hex, no filename, just the hash
    let sidecar_url = format!("https://dl.google.com/go/{filename}.sha256");
    if let Ok(text) = fetch_text(&sidecar_url).await {
        let h = text.trim();
        if h.len() == 64 && h.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(Some(h.to_lowercase()));
        }
    } // no sidecar → fall through to the JSON index

    // 2. dl JSON index (recent releases only)
    let bytes = fetch_bytes("https://go.dev/dl/?mode=json").await?;
    let json: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|e| CoreError::Download(format!("parse go dl failed: {e}")))?;
    let target = format!("go{version}");
    if let Some(arr) = json.as_array() {
        for rel in arr {
            if rel["version"].as_str() == Some(target.as_str())
                && let Some(files) = rel["files"].as_array()
            {
                for f in files {
                    if f["filename"].as_str() == Some(filename)
                        && let Some(h) = f["sha256"].as_str()
                    {
                        return Ok(Some(h.to_string()));
                    }
                }
            }
        }
    }
    Ok(None)
}

/// Parse `hash  filename` (shasum format) lines.
fn parse_shasums(text: &str, filename: &str) -> Option<String> {
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        if let (Some(hash), Some(name)) = (parts.next(), parts.next())
            && name == filename
            && hash.len() == 64
        {
            return Some(hash.to_lowercase());
        }
    }
    None
}

async fn fetch_text(url: &str) -> Result<String> {
    let bytes = fetch_bytes(url).await?;
    String::from_utf8(bytes).map_err(|e| CoreError::Download(format!("{url} is not UTF-8: {e}")))
}

async fn fetch_bytes(url: &str) -> Result<Vec<u8>> {
    let client = super::self_update::http_client()?;
    let resp = client
        .get(url)
        .header("User-Agent", "oneinit-checksum-resolver")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_shasums() {
        let text = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  node-v20.18.1-linux-x64.tar.gz\n";
        assert_eq!(
            parse_shasums(text, "node-v20.18.1-linux-x64.tar.gz").unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(parse_shasums(text, "other"), None);
    }
}
