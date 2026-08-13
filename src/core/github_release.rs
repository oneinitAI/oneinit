//! GitHub Release 动态解析 —— dynamic 配方的版本 / 资产 / 校验和解析
//!
//! 支持从 GitHub releases API 实时解析：
//! - `releases/latest` → 最新 tag
//! - `releases/tags/{version}` → 指定版本
//! - 资产按 `asset_pattern`（{version}/{os}/{arch}/{ext}）精确匹配
//! - 校验和从 `.sha256` 资产或 `checksums.txt` 解析

use serde::Deserialize;

use super::{CoreError, Result};

/// 平台标识 → 资产名中的 os 段
pub fn os_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else {
        "linux"
    }
}

/// 默认归档扩展名（按平台）
pub fn default_ext() -> &'static str {
    if cfg!(target_os = "windows") {
        "zip"
    } else {
        "tar.gz"
    }
}

/// 架构别名表（{arch} 占位符展开，按优先级）
pub fn arch_aliases() -> Vec<&'static str> {
    if cfg!(target_arch = "aarch64") {
        vec!["aarch64", "arm64"]
    } else {
        vec!["x86_64", "x64", "amd64"]
    }
}

/// GitHub release 资产（只取需要的字段）
#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
}

/// GitHub release 响应（只取需要的字段）
#[derive(Debug, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub assets: Vec<Asset>,
}

/// HTTP 客户端（禁重定向，与注册表/团队同步一致的安全策略）
fn client() -> Result<reqwest::Client> {
    super::self_update::http_client()
}

async fn fetch_bytes(url: &str) -> Result<Vec<u8>> {
    let client = client()?;
    let resp = client
        .get(url)
        .header("User-Agent", "oneinit-github-release")
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

async fn fetch_text(url: &str) -> Result<String> {
    let bytes = fetch_bytes(url).await?;
    String::from_utf8(bytes).map_err(|e| CoreError::Download(format!("{url} not UTF-8: {e}")))
}

/// 拉取资产内容（校验和文件等）。
///
/// 与 API 请求不同：`browser_download_url` 会 302 到 objects.githubusercontent.com，
/// 这是 GitHub 的标准资产托管行为（URL 本身来自官方 API），需允许重定向。
async fn fetch_asset_bytes(url: &str) -> Result<Vec<u8>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| CoreError::Download(format!("build http client failed: {e}")))?;
    let resp = client
        .get(url)
        .header("User-Agent", "oneinit-github-release")
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

/// 拉取并反序列化一个 release（latest 或指定 tag）
pub async fn fetch_release(repo: &str, version_spec: Option<&str>) -> Result<Release> {
    let url = match version_spec {
        None | Some("latest") | Some("") => {
            format!("https://api.github.com/repos/{repo}/releases/latest")
        }
        Some(v) => format!("https://api.github.com/repos/{repo}/releases/tags/{v}"),
    };
    let bytes = fetch_bytes(&url).await?;
    serde_json::from_slice(&bytes)
        .map_err(|e| CoreError::Download(format!("parse {url} failed: {e}")))
}

/// 解析最新（或指定）版本的 tag 名
pub async fn resolve_tag(repo: &str, version_spec: Option<&str>) -> Result<String> {
    let release = fetch_release(repo, version_spec).await?;
    Ok(release.tag_name)
}

/// 资产候选名：占位符替换 + 架构别名展开
pub fn asset_candidates(pattern: &str, version: &str, os: &str, ext: &str) -> Vec<String> {
    let base = pattern
        .replace("{version}", version)
        .replace("{os}", os)
        .replace("{ext}", ext);
    if base.contains("{arch}") {
        arch_aliases()
            .into_iter()
            .map(|a| base.replace("{arch}", a))
            .collect()
    } else {
        vec![base]
    }
}

/// 平台 os 关键字（资产名中可能的变体，小写）
fn os_keywords(os: &str) -> Vec<&'static str> {
    match os {
        "windows" => vec!["windows", "win32", "win"],
        "darwin" => vec!["darwin", "macos", "mac"],
        _ => vec!["linux"],
    }
}

/// 架构关键字（含别名，小写）
fn arch_keywords(arch: &str) -> Vec<&'static str> {
    match arch {
        "aarch64" => vec!["aarch64", "arm64"],
        _ => vec!["x86_64", "x64", "amd64"],
    }
}

/// 当前主机架构（asset 匹配用）
pub fn runtime_arch() -> &'static str {
    if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "x86_64"
    }
}

/// 模糊匹配：资产名包含 version + os 关键字 + 架构关键字 + 扩展名
///
/// 用于 ripgrep 等 rust-triple 命名的资产（`ripgrep-14.1.0-x86_64-pc-windows-msvc.zip`），
/// 精确模板匹配不适用时的回退。
fn find_asset_fuzzy(
    assets: &[Asset],
    version: &str,
    os: &str,
    ext: &str,
) -> Option<(String, String)> {
    let os_keys = os_keywords(os);
    let arch_keys = arch_keywords(runtime_arch());
    let ver_lower = version.to_lowercase();
    for asset in assets {
        let name_lower = asset.name.to_lowercase();
        if !name_lower.contains(&ver_lower) {
            continue;
        }
        if !os_keys.iter().any(|k| name_lower.contains(k)) {
            continue;
        }
        if !arch_keys.iter().any(|k| name_lower.contains(k)) {
            continue;
        }
        if !asset.name.ends_with(ext) {
            continue;
        }
        return Some((asset.name.clone(), asset.browser_download_url.clone()));
    }
    None
}

/// 在 release 资产中匹配目标资产，返回 (资产名, 下载 URL)
///
/// 策略：先精确模板匹配（`asset_pattern` 占位符替换，适合 gh 等规则命名），
/// 未命中再模糊关键字匹配（适合 ripgrep 等 rust-triple 命名）。
pub fn find_asset(
    release: &Release,
    pattern: &str,
    version: &str,
    os: &str,
    ext: &str,
) -> Option<(String, String)> {
    let candidates = asset_candidates(pattern, version, os, ext);
    for asset in &release.assets {
        if candidates.iter().any(|c| c == &asset.name) {
            return Some((asset.name.clone(), asset.browser_download_url.clone()));
        }
    }
    find_asset_fuzzy(&release.assets, version, os, ext)
}

/// 解析校验和
///
/// - `asset.sha256`：拉取 `{资产名}.sha256`，取其首个 token（hash）
/// - `checksums.txt`：拉取 `*checksums*` 资产，按 `hash  filename` 行匹配
/// - 空字符串：跳过（返回空，由调用方按 --no-checksum 处理）
pub async fn resolve_checksum(
    checksum_spec: &str,
    release: &Release,
    asset_name: &str,
) -> Result<String> {
    if checksum_spec.is_empty() {
        return Ok(String::new());
    }

    let url = match checksum_spec {
        "asset.sha256" => release
            .assets
            .iter()
            .find(|a| a.name == format!("{asset_name}.sha256"))
            .map(|a| a.browser_download_url.clone()),
        "checksums.txt" => release
            .assets
            .iter()
            .find(|a| a.name.to_lowercase().contains("checksums"))
            .map(|a| a.browser_download_url.clone()),
        other => {
            return Err(CoreError::Download(format!(
                "dynamic 配方校验和来源不支持: {other}（支持 asset.sha256 / checksums.txt / 空）"
            )));
        }
    };

    let Some(url) = url else {
        return Err(CoreError::Download(format!(
            "找不到校验和来源 '{checksum_spec}'（资产 {asset_name}）— 可检查配方或使用 --no-checksum"
        )));
    };

    let bytes = fetch_asset_bytes(&url).await?;
    let text = String::from_utf8(bytes)
        .map_err(|e| CoreError::Download(format!("{url} not UTF-8: {e}")))?;

    match checksum_spec {
        "asset.sha256" => {
            // 文件格式多样：纯 hash / "hash  filename" / "SHA256 hash of X:\n<hash>\n..."
            // 取第一个 64/128 位 hex token
            match text
                .split_whitespace()
                .find(|t| t.len() == 64 || t.len() == 128)
            {
                Some(h) => Ok(h.to_lowercase()),
                None => Err(CoreError::Download(format!(
                    "{url} 中未找到有效校验和（格式不支持）"
                ))),
            }
        }
        "checksums.txt" => {
            for line in text.lines() {
                let mut parts = line.split_whitespace();
                if let (Some(hash), Some(name)) = (parts.next(), parts.next())
                    && name == asset_name
                    && (hash.len() == 64 || hash.len() == 128)
                {
                    return Ok(hash.to_lowercase());
                }
            }
            Err(CoreError::Download(format!(
                "{asset_name} 未出现在 {url} 中"
            )))
        }
        _ => unreachable!("validated by match above"),
    }
}

/// 拉取指定版本（或 latest）的全部资产（供测试/诊断）
pub async fn release_assets(
    repo: &str,
    version_spec: Option<&str>,
) -> Result<Vec<(String, String)>> {
    let release = fetch_release(repo, version_spec).await?;
    Ok(release
        .assets
        .iter()
        .map(|a| (a.name.clone(), a.browser_download_url.clone()))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_candidates_replaces_placeholders() {
        let cands = asset_candidates(
            "ripgrep-{version}-{os}-{arch}.{ext}",
            "14.1.0",
            "windows",
            "zip",
        );
        // x86_64 主机的架构别名
        assert!(!cands.is_empty());
        assert!(cands.iter().any(|c| c.contains("14.1.0")));
        assert!(cands.iter().any(|c| c.contains("windows")));
        assert!(cands.iter().any(|c| c.ends_with(".zip")));
    }

    #[test]
    fn test_asset_candidates_arch_aliases() {
        let cands = asset_candidates("tool-{version}-{arch}.zip", "1.0.0", "linux", "zip");
        // 至少包含主架构名
        assert!(cands.iter().any(|c| c == "tool-1.0.0-x86_64.zip"));
    }

    #[test]
    fn test_find_asset_exact_match() {
        let release = Release {
            tag_name: "14.1.0".to_string(),
            assets: vec![
                Asset {
                    name: "ripgrep-14.1.0-x86_64-pc-windows-msvc.zip".to_string(),
                    browser_download_url: "https://example.com/a.zip".to_string(),
                },
                Asset {
                    name: "ripgrep-14.1.0-windows-x86_64.zip".to_string(),
                    browser_download_url: "https://example.com/b.zip".to_string(),
                },
            ],
        };
        let found = find_asset(
            &release,
            "ripgrep-{version}-{os}-{arch}.{ext}",
            "14.1.0",
            "windows",
            "zip",
        );
        assert!(found.is_some());
        assert_eq!(found.unwrap().0, "ripgrep-14.1.0-windows-x86_64.zip");
    }

    #[test]
    fn test_find_asset_missing() {
        let release = Release {
            tag_name: "1.0.0".to_string(),
            assets: vec![Asset {
                name: "other.zip".to_string(),
                browser_download_url: "https://example.com/o.zip".to_string(),
            }],
        };
        assert!(find_asset(&release, "tool-{version}-{os}.zip", "1.0.0", "linux", "zip").is_none());
    }

    #[test]
    fn test_find_asset_fuzzy_rust_triple() {
        // ripgrep 风格：x86_64-pc-windows-msvc（精确模板无法匹配 → 模糊回退）
        let release = Release {
            tag_name: "14.1.0".to_string(),
            assets: vec![
                Asset {
                    name: "ripgrep-14.1.0-x86_64-pc-windows-msvc.zip".to_string(),
                    browser_download_url: "https://example.com/win.zip".to_string(),
                },
                Asset {
                    name: "ripgrep-14.1.0-x86_64-unknown-linux-musl.tar.gz".to_string(),
                    browser_download_url: "https://example.com/linux.tgz".to_string(),
                },
            ],
        };
        // 精确模板（{arch}→x86_64）不匹配 ripgrep 的 triple 名 → 模糊命中
        let found = find_asset(
            &release,
            "ripgrep-{version}-{os}-{arch}.{ext}",
            "14.1.0",
            "windows",
            "zip",
        );
        assert!(found.is_some());
        assert_eq!(found.unwrap().1, "https://example.com/win.zip");
    }

    #[test]
    fn test_find_asset_fuzzy_gh_macos() {
        // gh 风格：macOS 大写 + amd64 架构别名
        let release = Release {
            tag_name: "2.97.0".to_string(),
            assets: vec![Asset {
                name: "gh_2.97.0_macOS_amd64.zip".to_string(),
                browser_download_url: "https://example.com/gh.zip".to_string(),
            }],
        };
        let found = find_asset(
            &release,
            "gh_{version}_macOS_amd64.zip",
            "2.97.0",
            "darwin",
            "zip",
        );
        assert!(found.is_some());
    }

    #[test]
    fn test_checksums_txt_parse() {
        // 纯函数模拟 resolve_checksum 的 checksums.txt 行解析
        let text = "a2c9b8497e1f85b1ad0dfcb78b5a622e098801b8e461e459e88e1ee12f018112  gh_2.97.0_linux_amd64.tar.gz\n";
        let mut found = None;
        for line in text.lines() {
            let mut parts = line.split_whitespace();
            if let (Some(hash), Some(name)) = (parts.next(), parts.next())
                && name == "gh_2.97.0_linux_amd64.tar.gz"
                && hash.len() == 64
            {
                found = Some(hash.to_string());
            }
        }
        assert_eq!(
            found.as_deref(),
            Some("a2c9b8497e1f85b1ad0dfcb78b5a622e098801b8e461e459e88e1ee12f018112")
        );
    }
}
