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
const DEFAULT_REGISTRY_URL: &str = "https://raw.githubusercontent.com/oneinitAI/oneinit-recipes/main";

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
}

/// 本地注册表配置（~/.oneinit/registry.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// 注册表 base URL
    pub registry_url: String,
    /// 上次 update 时间（ISO8601）
    #[serde(default)]
    pub last_update: String,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            registry_url: DEFAULT_REGISTRY_URL.to_string(),
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

/// 从远程download INDEX.json 并缓存
pub async fn fetch_index() -> Result<Index> {
    let config = load_config();
    let url = format!("{}/INDEX.json", config.registry_url);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| CoreError::Registry(format!("HTTP client creation failed: {}", e)))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| CoreError::Registry(format!("fetch INDEX.json failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(CoreError::Registry(format!(
            "INDEX.json HTTP {} — 仓库可能不exists或empty。先用 oneinit publish 添加recipe。",
            response.status()
        )));
    }

    let body = response
        .text()
        .await
        .map_err(|e| CoreError::Registry(format!("read INDEX.json response failed: {}", e)))?;

    let index: Index = serde_json::from_str(&body)
        .map_err(|e| CoreError::Registry(format!("INDEX.json parse failed: {}", e)))?;

    // 写入缓存
    let cache_path = cached_index_path();
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&cache_path, &body)?;

    // 更新配置的时间戳
    let mut config = config;
    config.last_update = chrono::Utc::now().to_rfc3339();
    save_config(&config)?;

    Ok(index)
}

// ============================================================
// recipedownload
// ============================================================

/// 从远程download单个recipe YAML
///
/// 路径: {registry_url}/recipes/{name}/{version}.yaml
pub async fn fetch_recipe(
    name: &str,
    version: &str,
) -> Result<super::community_recipe::CommunityRecipe> {
    let config = load_config();
    let url = format!("{}/recipes/{}/{}.yaml", config.registry_url, name, version);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| CoreError::Registry(format!("HTTP client creation failed: {}", e)))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| CoreError::Registry(format!("fetch recipe failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(CoreError::Registry(format!(
            "recipe {}={} HTTP {}",
            name,
            version,
            response.status()
        )));
    }

    let body = response
        .text()
        .await
        .map_err(|e| CoreError::Registry(format!("read recipe response failed: {}", e)))?;

    // 缓存到本地 recipes/
    let cache_recipe_path = super::recipes_dir().join(format!("{}.yaml", name));
    let _ = std::fs::write(&cache_recipe_path, &body);

    let recipe: super::community_recipe::CommunityRecipe = serde_yaml::from_str(&body)
        .map_err(|e| CoreError::Registry(format!("recipe YAML parse failed: {}", e)))?;

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
