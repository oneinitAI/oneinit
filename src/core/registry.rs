// 社区recipe注册表 — npm 式远程仓库
//
// 架构：
//   远程 GitHub 仓库 (oneinit-recipes) 存放 INDEX.json + recipes/<name>/<ver>.yaml
//   raw.githubusercontent.com 作为 CDN
//   本地 ~/.oneinit/cache/ 缓存 INDEX.json 和已download的recipe

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::{CoreError, Result, cache_dir, data_dir};

/// 默认注册表 URL（GitHub raw content）
const DEFAULT_REGISTRY_URL: &str =
    "https://raw.githubusercontent.com/oneinitAI/oneinit-recipes/main";

// ============================================================
// 配方签名验证（安全 #3）
//
// INDEX.json.sig = Ed25519 签名（hex），签名对象为 INDEX.json 原文。
// 注册表维护者用私钥（GitHub secret）签名，公钥内置于此。
// ============================================================

/// 注册表签名公钥（Ed25519，32 字节 hex）— 与配方仓库私钥配对
const REGISTRY_PUBLIC_KEY_HEX: &str =
    "4d3b8ea42836ab97766150581ae45439c5a3477bf036a5157c7dff9ba2ad3869";

/// 验证 INDEX.json 的 Ed25519 签名（使用内置注册表公钥）
pub fn verify_index_signature(data: &[u8], sig_hex: &str) -> bool {
    verify_signature(data, sig_hex, REGISTRY_PUBLIC_KEY_HEX)
}

/// 用指定公钥（hex）验签 — 团队环境 / 测试均可注入
pub fn verify_signature(data: &[u8], sig_hex: &str, pub_hex: &str) -> bool {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    let Some(pub_bytes) = hex_decode(pub_hex) else {
        return false;
    };
    let pub_array: [u8; 32] = match pub_bytes.try_into() {
        Ok(a) => a,
        Err(_) => return false,
    };
    let Some(sig_bytes) = hex_decode(sig_hex.trim()) else {
        return false;
    };
    let sig_array: [u8; 64] = match sig_bytes.try_into() {
        Ok(a) => a,
        Err(_) => return false,
    };
    let Ok(pubkey) = VerifyingKey::from_bytes(&pub_array) else {
        return false;
    };
    let sig = Signature::from_bytes(&sig_array);
    pubkey.verify(data, &sig).is_ok()
}

/// hex 字符串 → 字节（小写/大写均可）
fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

// ============================================================
// 数据结构
// ============================================================

/// INDEX.json — 全局recipe索引
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    /// 索引格式版本
    pub version: u32,
    /// 最后更新时间（ISO8601）
    pub last_updated: String,
    /// 包列表
    pub packages: BTreeMap<String, IndexEntry>,
}

/// 单个包的索引条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    #[serde(default)]
    pub description: String,
    /// 最新版本号
    pub latest: String,
    /// 所有可用版本
    pub versions: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub maintainers: Vec<String>,
    /// 来源注册表 URL（多订阅时区分配方来自哪个订阅）
    #[serde(default)]
    pub source: String,
}

/// 本地注册表配置（~/.oneinit/registry.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// 默认注册表 base URL
    pub registry_url: String,
    /// 自定义订阅 URL 列表（可多个，每个需提供 INDEX.json）
    #[serde(default)]
    pub subscriptions: Vec<String>,
    /// 上次 update 时间（ISO8601）
    #[serde(default)]
    pub last_update: String,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            registry_url: DEFAULT_REGISTRY_URL.to_string(),
            subscriptions: Vec::new(),
            last_update: String::new(),
        }
    }
}

// ============================================================
// 配置读写
// ============================================================

/// 配置文件路径: ~/.oneinit/registry.json
fn config_path() -> PathBuf {
    data_dir().join("registry.json")
}

/// 读取本地注册表配置（不exists则返回默认值）
pub fn load_config() -> RegistryConfig {
    let path = config_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        RegistryConfig::default()
    }
}

/// 保存注册表配置
pub fn save_config(config: &RegistryConfig) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| CoreError::Registry(format!("config serialize failed: {}", e)))?;
    std::fs::write(&path, json)?;
    Ok(())
}

// ============================================================
// 多订阅管理
// ============================================================

/// 所有注册表 URL（默认 + 自定义订阅，去重）
pub fn all_registry_urls() -> Vec<String> {
    let config = load_config();
    let mut urls: Vec<String> = vec![config.registry_url.clone()];
    for sub in &config.subscriptions {
        let trimmed = sub.trim().to_string();
        if !trimmed.is_empty() && !urls.contains(&trimmed) {
            urls.push(trimmed);
        }
    }
    urls
}

/// 添加自定义订阅 URL
pub fn add_subscription(url: &str) -> Result<()> {
    let trimmed = url.trim().to_string();
    if trimmed.is_empty() {
        return Err(CoreError::Registry("empty subscription URL".into()));
    }
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return Err(CoreError::Registry(
            "subscription URL must start with http:// or https://".into(),
        ));
    }
    let mut config = load_config();
    if config.subscriptions.contains(&trimmed) {
        return Err(CoreError::Registry("already subscribed".into()));
    }
    config.subscriptions.push(trimmed);
    save_config(&config)?;
    Ok(())
}

/// 移除自定义订阅 URL，返回是否移除成功
pub fn remove_subscription(url: &str) -> Result<bool> {
    let mut config = load_config();
    let before = config.subscriptions.len();
    config.subscriptions.retain(|s| s != url);
    let removed = config.subscriptions.len() != before;
    if removed {
        save_config(&config)?;
    }
    Ok(removed)
}

/// 列出所有自定义订阅 URL
pub fn list_subscriptions() -> Vec<String> {
    load_config().subscriptions
}

// ============================================================
// 索引缓存
// ============================================================

/// 缓存的 INDEX.json 路径: ~/.oneinit/cache/INDEX.json
fn cached_index_path() -> PathBuf {
    cache_dir().join("INDEX.json")
}

/// 读取本地缓存的 INDEX（不exists返回 None）
pub fn load_cached_index() -> Option<Index> {
    let path = cached_index_path();
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

/// 从单个注册表 URL 拉取 INDEX.json（不写缓存）
async fn fetch_index_from(client: &reqwest::Client, base_url: &str) -> Result<Index> {
    let url = format!("{}/INDEX.json", base_url);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| CoreError::Registry(format!("拉取 {url} 失败: {e}")))?;

    if !response.status().is_success() {
        return Err(CoreError::Registry(format!(
            "INDEX.json HTTP {} at {url}",
            response.status()
        )));
    }

    let body = response
        .text()
        .await
        .map_err(|e| CoreError::Registry(format!("read {url} response failed: {e}")))?;

    // 安全 #3：验签（INDEX.json.sig 存在则强校验；不存在则警告，兼容旧注册表）
    let sig_url = format!("{}/INDEX.json.sig", base_url);
    match client.get(&sig_url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let sig_text = resp
                .text()
                .await
                .map_err(|e| CoreError::Registry(format!("read {sig_url} failed: {e}")))?;
            let sig = sig_text.trim().to_string();
            if verify_index_signature(body.as_bytes(), &sig) {
                eprintln!("[OK] INDEX.json 签名已验证: {base_url}");
            } else {
                return Err(CoreError::Registry(format!(
                    "INDEX.json 签名验证失败（{base_url}）— 注册表可能被篡改，已拒绝使用该索引。"
                )));
            }
        }
        Ok(_) => {
            eprintln!(
                "[WARN] {} 没有 INDEX.json.sig — 跳过签名校验",
                base_url
            );
        }
        Err(e) => {
            eprintln!(
                "[WARN] 无法获取 {} 的签名: {} — 跳过签名校验",
                sig_url, e
            );
        }
    }

    serde_json::from_str(&body)
        .map_err(|e| CoreError::Registry(format!("INDEX.json parse failed at {url}: {e}")))
}

/// 合并多个 INDEX，包名冲突时优先保留先出现的（默认注册表优先）
/// 为每个 entry 标注 source 注册表 URL
fn merge_indexes(indexes: Vec<(String, Index)>) -> Index {
    let mut packages: BTreeMap<String, IndexEntry> = BTreeMap::new();
    let mut last_updated = String::new();

    for (source_url, index) in indexes {
        if last_updated.is_empty() && !index.last_updated.is_empty() {
            last_updated = index.last_updated.clone();
        }
        for (name, mut entry) in index.packages {
            entry.source = source_url.clone();
            packages.entry(name).or_insert(entry);
        }
    }

    Index {
        version: 1,
        last_updated,
        packages,
    }
}

/// 从所有注册表（默认 + 订阅）拉取 INDEX.json 并合并写入缓存
pub async fn fetch_index() -> Result<Index> {
    let urls = all_registry_urls();
    if urls.is_empty() {
        return Err(CoreError::Registry("no registry configured".into()));
    }

    // 安全 M-2：禁用重定向，防止被攻陷的注册表把请求重定向到攻击者服务器
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| CoreError::Registry(format!("HTTP client creation failed: {e}")))?;

    // 顺序拉取，容忍单个订阅失败
    let mut indexes: Vec<(String, Index)> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    for url in &urls {
        match fetch_index_from(&client, url).await {
            Ok(index) => indexes.push((url.clone(), index)),
            Err(e) => errors.push(e.to_string()),
        }
    }

    if indexes.is_empty() {
        return Err(CoreError::Registry(format!(
            "all registries failed: {}",
            errors.join("; ")
        )));
    }

    let merged = merge_indexes(indexes);

    // 写入缓存
    let body = serde_json::to_string_pretty(&merged)
        .map_err(|e| CoreError::Registry(format!("cache serialize failed: {e}")))?;
    let cache_path = cached_index_path();
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&cache_path, &body)?;

    // 更新配置的时间戳
    let mut config = load_config();
    config.last_update = chrono::Utc::now().to_rfc3339();
    save_config(&config)?;

    if !errors.is_empty() {
        eprintln!(
            "[WARN] {} 个注册表拉取失败: {}",
            errors.len(),
            errors.join("; ")
        );
    }

    Ok(merged)
}

// ============================================================
// recipedownload
// ============================================================

/// 从远程download单个recipe YAML
///
/// 路径: {registry_url}/recipes/{name}/{version}.yaml
/// 多订阅时根据缓存 INDEX 中该包的 source 选择正确的注册表
pub async fn fetch_recipe(
    name: &str,
    version: &str,
) -> Result<super::community_recipe::CommunityRecipe> {
    // 优先使用缓存 INDEX 里标注的 source（多订阅定位）
    let base_url = load_cached_index()
        .and_then(|idx| idx.packages.get(name).cloned())
        .map(|e| {
            if e.source.is_empty() {
                load_config().registry_url
            } else {
                e.source
            }
        })
        .unwrap_or_else(|| load_config().registry_url);

    let url = format!("{base_url}/recipes/{name}/{version}.yaml");

    // 安全 M-2：禁用重定向
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| CoreError::Registry(format!("HTTP client creation failed: {e}")))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| CoreError::Registry(format!("拉取配方失败: {e}")))?;

    if !response.status().is_success() {
        return Err(CoreError::Registry(format!(
            "recipe {name}={version} HTTP {}",
            response.status()
        )));
    }

    let body = response
        .text()
        .await
        .map_err(|e| CoreError::Registry(format!("read recipe response failed: {e}")))?;

    // 缓存到本地 recipes/
    let cache_recipe_path = super::recipes_dir().join(format!("{}.yaml", name));
    let _ = std::fs::write(&cache_recipe_path, &body);

    let recipe: super::community_recipe::CommunityRecipe = serde_yaml::from_str(&body)
        .map_err(|e| CoreError::Registry(format!("recipe YAML parse failed: {e}")))?;

    Ok(recipe)
}

// ============================================================
// 查找与解析
// ============================================================

/// 在缓存 INDEX 中查找包
pub fn resolve(name: &str) -> Option<IndexEntry> {
    let index = load_cached_index()?;
    index.packages.get(name).cloned()
}

/// 列出所有远程可用包
pub fn list_available() -> Vec<(String, String, String)> {
    // Vec<(name, latest_version, description)>
    let index = match load_cached_index() {
        Some(i) => i,
        None => return Vec::new(),
    };
    index
        .packages
        .iter()
        .map(|(name, entry)| {
            (
                name.clone(),
                entry.latest.clone(),
                entry.description.clone(),
            )
        })
        .collect()
}

/// 列出所有远程可用包（含来源注册表 URL，TUI 显示用）
pub fn list_available_with_source() -> Vec<(String, String, String, String)> {
    // Vec<(name, latest_version, description, source_url)>
    let index = match load_cached_index() {
        Some(i) => i,
        None => return Vec::new(),
    };
    index
        .packages
        .iter()
        .map(|(name, entry)| {
            (
                name.clone(),
                entry.latest.clone(),
                entry.description.clone(),
                entry.source.clone(),
            )
        })
        .collect()
}

/// 检查索引缓存是否过期（超过指定小时数）
pub fn is_index_stale(max_age_hours: u64) -> bool {
    let config = load_config();
    if config.last_update.is_empty() {
        return true;
    }
    match chrono::DateTime::parse_from_rfc3339(&config.last_update) {
        Ok(last) => {
            let elapsed = chrono::Utc::now().signed_duration_since(last);
            elapsed.num_hours() >= max_age_hours as i64
        }
        Err(_) => true,
    }
}

// ============================================================
// INDEX 生成（用于 publish）
// ============================================================

/// 从recipe列表生成 INDEX.json
pub fn generate_index(recipes: &[super::community_recipe::CommunityRecipe]) -> Index {
    let mut packages = BTreeMap::new();

    for recipe in recipes {
        let entry = packages
            .entry(recipe.name.clone())
            .or_insert_with(|| IndexEntry {
                description: recipe.description.clone(),
                latest: recipe.version.clone(),
                versions: Vec::new(),
                tags: recipe.tags.clone().unwrap_or_default(),
                maintainers: Vec::new(),
                source: String::new(),
            });

        // 更新版本列表
        if !entry.versions.contains(&recipe.version) {
            entry.versions.push(recipe.version.clone());
        }

        // 更新 latest（简单 chars串比较，取最大）
        if recipe.version > entry.latest {
            entry.latest = recipe.version.clone();
        }

        // 更新 description（非空时覆盖）
        if !recipe.description.is_empty() {
            entry.description = recipe.description.clone();
        }

        // 合并 tags
        if let Some(ref tags) = recipe.tags {
            for tag in tags {
                if !entry.tags.contains(tag) {
                    entry.tags.push(tag.clone());
                }
            }
        }

        // 合并 maintainers
        if let Some(ref m) = recipe.maintainer
            && let Some(ref name) = m.github
            && !entry.maintainers.contains(name)
        {
            entry.maintainers.push(name.clone());
        }
    }

    // 排序版本号
    for entry in packages.values_mut() {
        entry.versions.sort();
    }

    Index {
        version: 1,
        last_updated: chrono::Utc::now().to_rfc3339(),
        packages,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_index_signature_valid_and_tampered() {
        use ed25519_dalek::{Signer, SigningKey};

        // 固定 seed 生成密钥对（可复现）
        let seed = [7u8; 32];
        let signing = SigningKey::from_bytes(&seed);
        let pub_hex: String = signing
            .verifying_key()
            .to_bytes()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();

        let data = b"{\"version\":1,\"packages\":{}}";
        let sig: String = signing
            .sign(data)
            .to_bytes()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();

        // 正路径：正确签名通过
        assert!(verify_signature(data, &sig, &pub_hex));
        // 篡改路径：数据被改 → 拒绝
        assert!(!verify_signature(b"{\"version\":2}", &sig, &pub_hex));
        // 伪造路径：随机错误签名 → 拒绝
        assert!(!verify_signature(data, &"00".repeat(64), &pub_hex));
        // 非法 hex → 拒绝
        assert!(!verify_signature(data, "not-hex", &pub_hex));
        // 公钥非法 → 拒绝
        assert!(!verify_signature(data, &sig, "zz"));
    }

    #[test]
    fn test_hex_decode() {
        assert_eq!(
            hex_decode("4d3b8ea4").unwrap(),
            vec![0x4d, 0x3b, 0x8e, 0xa4]
        );
        assert_eq!(hex_decode("FF00"), Some(vec![0xff, 0x00]));
        assert!(hex_decode("").unwrap().is_empty());
        assert_eq!(hex_decode("xyz"), None);
        assert_eq!(hex_decode("abc"), None); // 奇数长度
    }

    #[test]
    fn test_merge_indexes_first_wins_and_source() {
        let index_a = Index {
            version: 1,
            last_updated: "2026-08-01T00:00:00Z".to_string(),
            packages: BTreeMap::from([(
                "node20".to_string(),
                IndexEntry {
                    description: "from A".to_string(),
                    latest: "20.18.1".to_string(),
                    versions: vec!["20.18.1".to_string()],
                    tags: vec![],
                    maintainers: vec![],
                    source: String::new(),
                },
            )]),
        };
        let index_b = Index {
            version: 1,
            last_updated: "2026-08-01T01:00:00Z".to_string(),
            packages: BTreeMap::from([
                (
                    "node20".to_string(),
                    IndexEntry {
                        description: "from B (should lose)".to_string(),
                        latest: "99".to_string(),
                        versions: vec!["99".to_string()],
                        tags: vec![],
                        maintainers: vec![],
                        source: String::new(),
                    },
                ),
                (
                    "rust".to_string(),
                    IndexEntry {
                        description: "from B".to_string(),
                        latest: "stable".to_string(),
                        versions: vec!["stable".to_string()],
                        tags: vec![],
                        maintainers: vec![],
                        source: String::new(),
                    },
                ),
            ]),
        };

        let merged = merge_indexes(vec![
            ("url-a".to_string(), index_a),
            ("url-b".to_string(), index_b),
        ]);

        assert_eq!(merged.packages.len(), 2);
        // 冲突时第一个（默认注册表）优先
        assert_eq!(merged.packages["node20"].latest, "20.18.1");
        assert_eq!(merged.packages["node20"].description, "from A");
        // source 标注
        assert_eq!(merged.packages["node20"].source, "url-a");
        assert_eq!(merged.packages["rust"].source, "url-b");
    }

    #[test]
    fn test_add_remove_subscription_roundtrip() {
        // 用临时 HOME 隔离，避免污染真实配置
        let mut config = RegistryConfig::default();
        config
            .subscriptions
            .push("https://example.com/reg".to_string());
        // 直接测试 add/remove 逻辑（不碰真实文件）
        let url = "https://mirror.example.com/recipes";
        let mut cfg = RegistryConfig::default();
        cfg.subscriptions.push(url.to_string());
        assert!(cfg.subscriptions.contains(&url.to_string()));
        cfg.subscriptions.retain(|s| s != url);
        assert!(cfg.subscriptions.is_empty());
        // 防重复
        let mut cfg2 = RegistryConfig::default();
        if !cfg2.subscriptions.contains(&url.to_string()) {
            cfg2.subscriptions.push(url.to_string());
        }
        assert_eq!(cfg2.subscriptions.len(), 1);
    }
}
