//! Version resolution for the non-exact-match recipe system.
//!
//! A user asks for `python@3.11` / `node@lts` / `go@latest`; this module
//! resolves the spec to a concrete version using an embedded catalog, refreshed
//! from official APIs (nodejs.org / go.dev) and cached in SQLite.

use super::{CoreError, Result};

/// A versioned tool family known to the resolver.
pub const FAMILIES: [&str; 5] = ["python", "node", "go", "java", "rust"];

/// Default + LTS versions (hand-maintained fallback; refreshed from APIs).
///
/// Only versions the dynamic installer can actually fetch are listed: python
/// is limited to 3.11+ because python.org stopped publishing Windows
/// embeddable zips for 3.10 and older.
fn embedded_catalog(recipe: &str) -> Option<(&'static str, &'static str, &'static [&'static str])> {
    match recipe {
        "python" => Some(("3.12.4", "3.12.4", &["3.13.0", "3.12.4", "3.11.9"])),
        "node" => Some((
            "20.18.1",
            "22.11.0",
            &["22.11.0", "21.7.3", "20.18.1", "18.20.4"],
        )),
        "go" => Some(("1.23.4", "1.23.4", &["1.24.0", "1.23.4", "1.22.10"])),
        // java (Temurin) — LTS lines as of 2026; refreshed from Adoptium.
        "java" => Some((
            "21.0.12",
            "25.0.4",
            &["25.0.4", "21.0.12", "17.0.20", "11.0.32"],
        )),
        "rust" => Some(("stable", "stable", &["stable", "1.82.0"])),
        _ => None,
    }
}

/// Is `recipe` a versioned family with non-exact-match support?
pub fn is_versioned(recipe: &str) -> bool {
    embedded_catalog(recipe).is_some()
}

/// Resolve a version spec to a concrete version.
///
/// spec rules:
/// - exact ("3.11.9") → as-is if it looks like a version
/// - partial ("3.11") → newest catalog version with that prefix
/// - "latest" → newest catalog version
/// - "lts" → the LTS version
/// - None → default_version
///
/// Cached versions (from a prior `--refresh`) are merged with the embedded
/// catalog and take part in resolution (never replace it).
pub fn resolve(recipe: &str, spec: Option<&str>) -> Result<String> {
    resolve_with(recipe, spec, None)
}

/// Test seam: `versions_override` replaces the catalog + cache entirely.
fn resolve_with(
    recipe: &str,
    spec: Option<&str>,
    versions_override: Option<&[&str]>,
) -> Result<String> {
    let (default_v, lts_v, embedded) = embedded_catalog(recipe)
        .ok_or_else(|| CoreError::Other(format!("recipe '{recipe}' is not a versioned family")))?;

    // catalog = embedded, merged with cached versions seen before
    let versions: Vec<String> = match versions_override {
        Some(v) => v.iter().map(|s| s.to_string()).collect(),
        None => {
            let mut versions: Vec<String> = embedded.iter().map(|s| s.to_string()).collect();
            if let Ok(cached) = super::cache_db::cached_versions(recipe) {
                for v in cached {
                    if !versions.contains(&v) {
                        versions.push(v);
                    }
                }
            }
            versions
        }
    };

    match spec.unwrap_or("") {
        "" => Ok(default_v.to_string()),
        "latest" => versions
            .iter()
            .max_by(|a, b| compare_versions(a, b))
            .cloned()
            .ok_or_else(|| CoreError::Other(format!("没有 '{recipe}' 的已知版本"))),
        "lts" => Ok(lts_v.to_string()),
        partial => {
            // Prefix match on the catalog: "3.11" → newest "3.11.x",
            // exact specs ("3.11.9") match themselves. Non-semver values
            // like rust's "stable" also match by prefix.
            let matches: Vec<&String> =
                versions.iter().filter(|v| v.starts_with(partial)).collect();
            matches
                .into_iter()
                .max_by(|a, b| compare_versions(a, b))
                .cloned()
                .ok_or_else(|| {
                    CoreError::Other(format!(
                        "未找到 '{recipe}' 的版本 '{partial}' — 试试 `oneinit list versions {recipe}`"
                    ))
                })
        }
    }
}

/// Compare two dotted versions (fallback: plain string compare).
fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let pa: Vec<u64> = a.split('.').filter_map(|p| p.parse::<u64>().ok()).collect();
    let pb: Vec<u64> = b.split('.').filter_map(|p| p.parse::<u64>().ok()).collect();
    if pa.is_empty() || pb.is_empty() {
        return a.cmp(b);
    }
    for (x, y) in pa.iter().zip(pb.iter()) {
        match x.cmp(y) {
            std::cmp::Ordering::Equal => continue,
            o => return o,
        }
    }
    pa.len().cmp(&pb.len())
}

/// Refresh the version catalog from official APIs, caching into SQLite.
/// Supports node (index.json), go (go.dev JSON) and java (Adoptium
/// feature_releases for the LTS lines). Others keep embedded.
pub async fn refresh(recipe: &str) -> Result<usize> {
    match recipe {
        "node" => {
            let versions = fetch_node_versions().await?;
            let n = versions.len();
            for v in versions {
                super::cache_db::cache_version("node", &v, "nodejs.org")?;
            }
            Ok(n)
        }
        "go" => {
            let versions = fetch_go_versions().await?;
            let n = versions.len();
            for v in versions {
                super::cache_db::cache_version("go", &v, "go.dev")?;
            }
            Ok(n)
        }
        "java" => {
            let mut n = 0;
            for feature in ["11", "17", "21", "25"] {
                let versions = fetch_adoptium_versions(feature).await?;
                for v in versions {
                    super::cache_db::cache_version("java", &v, "adoptium")?;
                    n += 1;
                }
            }
            Ok(n)
        }
        other => Err(CoreError::Other(format!(
            "'{other}' 不支持在线版本刷新（仅内置目录）"
        ))),
    }
}

async fn fetch_node_versions() -> Result<Vec<String>> {
    let client = super::self_update::http_client().map_err(|e| CoreError::Other(e.to_string()))?;
    let resp = client
        .get("https://nodejs.org/dist/index.json")
        .header("User-Agent", "oneinit-version-resolver")
        .send()
        .await
        .map_err(|e| CoreError::Download(format!("拉取 node 版本失败: {e}")))?;
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| CoreError::Download(format!("read node versions failed: {e}")))?;
    let json: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|e| CoreError::Download(format!("解析 node 版本失败: {e}")))?;
    let mut versions = Vec::new();
    if let Some(arr) = json.as_array() {
        for item in arr {
            if let Some(v) = item["version"].as_str() {
                versions.push(v.trim_start_matches('v').to_string());
            }
        }
    }
    if versions.is_empty() {
        return Err(CoreError::Download(
            "nodejs.org 索引中没有版本".into(),
        ));
    }
    Ok(versions)
}

async fn fetch_go_versions() -> Result<Vec<String>> {
    let client = super::self_update::http_client().map_err(|e| CoreError::Other(e.to_string()))?;
    let resp = client
        .get("https://go.dev/dl/?mode=json")
        .header("User-Agent", "oneinit-version-resolver")
        .send()
        .await
        .map_err(|e| CoreError::Download(format!("拉取 go 版本失败: {e}")))?;
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| CoreError::Download(format!("read go versions failed: {e}")))?;
    let json: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|e| CoreError::Download(format!("解析 go 版本失败: {e}")))?;
    let mut versions = Vec::new();
    if let Some(arr) = json.as_array() {
        for item in arr {
            if let Some(v) = item["version"].as_str() {
                versions.push(v.trim_start_matches("go").to_string());
            }
        }
    }
    if versions.is_empty() {
        return Err(CoreError::Download("go.dev 索引中没有版本".into()));
    }
    Ok(versions)
}

/// Adoptium `feature_releases` — concrete semvers (e.g. `21.0.12+8`) for a
/// feature line, newest first.
async fn fetch_adoptium_versions(feature: &str) -> Result<Vec<String>> {
    let client = super::self_update::http_client().map_err(|e| CoreError::Other(e.to_string()))?;
    let url = format!("https://api.adoptium.net/v3/assets/feature_releases/{feature}/ga");
    let resp = client
        .get(&url)
        .header("User-Agent", "oneinit-version-resolver")
        .send()
        .await
        .map_err(|e| CoreError::Download(format!("拉取 adoptium 版本失败: {e}")))?;
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| CoreError::Download(format!("read adoptium versions failed: {e}")))?;
    let json: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|e| CoreError::Download(format!("解析 adoptium 版本失败: {e}")))?;
    let mut versions = Vec::new();
    if let Some(arr) = json.as_array() {
        for rel in arr {
            if let Some(v) = rel["version_data"]["semver"].as_str() {
                versions.push(v.to_string());
            }
        }
    }
    if versions.is_empty() {
        return Err(CoreError::Download(format!(
            "no releases in Adoptium feature_releases/{feature}"
        )));
    }
    Ok(versions)
}

/// List all known versions for a recipe (cached + embedded, deduped, newest first).
pub fn list(recipe: &str) -> Result<Vec<String>> {
    let (_, _, embedded) = embedded_catalog(recipe)
        .ok_or_else(|| CoreError::Other(format!("recipe '{recipe}' is not a versioned family")))?;
    let mut versions: Vec<String> = embedded.iter().map(|s| s.to_string()).collect();
    if let Ok(cached) = super::cache_db::cached_versions(recipe) {
        for v in cached {
            if !versions.contains(&v) {
                versions.push(v);
            }
        }
    }
    versions.sort_by(|a, b| compare_versions(b, a));
    Ok(versions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_exact_and_partial() {
        let py = ["3.13.0", "3.12.4", "3.11.9", "3.10.14", "3.9.19"];
        let nd = ["22.11.0", "21.7.3", "20.18.1", "18.20.4"];
        assert_eq!(
            resolve_with("python", Some("3.11.9"), Some(&py)).unwrap(),
            "3.11.9"
        );
        assert_eq!(
            resolve_with("python", Some("3.11"), Some(&py)).unwrap(),
            "3.11.9"
        );
        assert_eq!(
            resolve_with("python", Some("3"), Some(&py)).unwrap(),
            "3.13.0"
        );
        assert_eq!(resolve_with("python", None, Some(&py)).unwrap(), "3.12.4");
        assert_eq!(
            resolve_with("node", Some("lts"), Some(&nd)).unwrap(),
            "22.11.0"
        );
        assert_eq!(
            resolve_with("node", Some("20"), Some(&nd)).unwrap(),
            "20.18.1"
        );
    }

    #[test]
    fn test_resolve_unknown() {
        let py = ["3.13.0", "3.12.4", "3.11.9", "3.10.14", "3.9.19"];
        assert!(resolve_with("python", Some("9.99"), Some(&py)).is_err());
        assert!(resolve("unknown", None).is_err());
    }

    #[test]
    fn test_compare_versions() {
        assert_eq!(
            compare_versions("3.11.9", "3.11.10"),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            compare_versions("3.13.0", "3.11.9"),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            compare_versions("20.18.1", "20.18.1"),
            std::cmp::Ordering::Equal
        );
    }

    #[test]
    fn test_is_versioned() {
        assert!(is_versioned("python"));
        assert!(is_versioned("node"));
        assert!(!is_versioned("dotnet8"));
    }
}
