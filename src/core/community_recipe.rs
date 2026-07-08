// 社区配方系统 — 按 社区配方.md 实现
//
// 声明式 YAML 配方格式，支持多平台、模板变量、安全提醒。
// 配方文件存放在 ~/.oneinit/recipes/*.yaml

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::{db_dir, envs_dir, CoreError, Result};
use crate::output::OutputFormatter;

// ============================================================
// 配方 DTO（严格对应 社区配方.md 1.1 节 YAML 格式）
// ============================================================

/// 社区配方（从 YAML 反序列化）
#[derive(Debug, Clone, Deserialize)]
pub struct CommunityRecipe {
    pub name: String,
    pub version: String,
    #[allow(dead_code)]
    pub description: String,
    pub platforms: Platforms,
    pub post_install: Option<PostInstallConfig>,
    #[allow(dead_code)]
    pub depends: Option<Vec<String>>,
    #[allow(dead_code)]
    pub tags: Option<Vec<String>>,
    #[allow(dead_code)]
    pub maintainer: Option<Maintainer>,
}

/// 平台配置集合
#[derive(Debug, Clone, Deserialize)]
pub struct Platforms {
    pub windows: Option<PlatformConfig>,
    pub linux: Option<PlatformConfig>,
    pub darwin: Option<PlatformConfig>,
}

/// 单平台配置
#[derive(Debug, Clone, Deserialize)]
pub struct PlatformConfig {
    pub url: String,
    pub sha256: String,
    pub install_type: String,
    pub install_args: Option<Vec<String>>,
    pub install_path: String,
    pub path_add: Vec<String>,
}

/// 后置配置
#[derive(Debug, Clone, Deserialize)]
pub struct PostInstallConfig {
    #[allow(dead_code)]
    pub env_vars: Option<BTreeMap<String, String>>,
    pub config_files: Option<Vec<ConfigFile>>,
    pub commands: Option<Vec<String>>,
}

/// 配置文件定义（模板）
#[derive(Debug, Clone, Deserialize)]
pub struct ConfigFile {
    pub path: String,
    pub template: String,
}

/// 维护者信息
#[derive(Debug, Clone, Deserialize)]
pub struct Maintainer {
    #[allow(dead_code)]
    pub name: Option<String>,
    #[allow(dead_code)]
    pub github: Option<String>,
    #[allow(dead_code)]
    pub email: Option<String>,
}

/// 合法的 install_type 值
const VALID_INSTALL_TYPES: &[&str] = &[
    "zip_extract",
    "tar_extract",
    "exe_silent",
    "msi_install",
    "pkg_install",
    "binary_copy",
];

// ============================================================
// 目录助手
// ============================================================

/// 社区配方存放目录: ~/.oneinit/recipes/
pub fn recipes_dir() -> PathBuf {
    super::data_dir().join("recipes")
}

// ============================================================
// 加载与查找
// ============================================================

/// 从 ~/.oneinit/recipes/*.yaml 加载所有社区配方
pub fn load_all() -> Vec<CommunityRecipe> {
    let dir = recipes_dir();
    let mut recipes = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("yaml") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(recipe) = serde_yaml::from_str::<CommunityRecipe>(&content) {
                        recipes.push(recipe);
                    }
                }
            }
        }
    }

    recipes
}

/// 按名称查找社区配方
pub fn resolve(name: &str) -> Option<CommunityRecipe> {
    load_all().into_iter().find(|r| r.name == name)
}

/// 列出所有社区配方名
pub fn list_names() -> Vec<String> {
    load_all().iter().map(|r| r.name.clone()).collect()
}

/// 获取当前平台的配置
pub fn current_platform_config(recipe: &CommunityRecipe) -> Option<&PlatformConfig> {
    #[cfg(target_os = "windows")]
    {
        recipe.platforms.windows.as_ref()
    }
    #[cfg(target_os = "linux")]
    {
        recipe.platforms.linux.as_ref()
    }
    #[cfg(target_os = "macos")]
    {
        recipe.platforms.darwin.as_ref()
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        None
    }
}

// ============================================================
// 模板变量渲染
// ============================================================

/// 渲染模板变量
///
/// 支持的变量（按 社区配方.md）：
/// - {{install_dir}} — 安装目录绝对路径
/// - {{user_home}} — 用户主目录
/// - {{mirror_pip}} — pip 清华源 URL
/// - {{mirror_pip_host}} — pip 清华源主机名
/// - {{mirror_npm}} — npm 淘宝源 URL
pub fn render_template(template: &str, install_dir: &Path) -> String {
    let install_dir_str = install_dir.to_string_lossy().to_string();
    let user_home = dirs::home_dir()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_default();

    template
        .replace("{{install_dir}}", &install_dir_str)
        .replace("{{user_home}}", &user_home)
        .replace("{{mirror_pip}}", "https://pypi.tuna.tsinghua.edu.cn/simple")
        .replace("{{mirror_pip_host}}", "pypi.tuna.tsinghua.edu.cn")
        .replace("{{mirror_npm}}", "https://registry.npmmirror.com")
}

// ============================================================
// 验证
// ============================================================

/// 验证结果
#[derive(Debug)]
pub struct VerifyResult {
    pub valid: bool,
    pub checks: Vec<(String, bool, String)>, // (检查项, 是否通过, 说明)
}

/// 验证配方文件
pub fn verify(yaml_path: &Path) -> Result<VerifyResult> {
    let content = std::fs::read_to_string(yaml_path)?;
    let mut checks = Vec::new();

    // 1. YAML 语法
    let recipe: CommunityRecipe = match serde_yaml::from_str(&content) {
        Ok(r) => {
            checks.push(("YAML 语法".to_string(), true, "解析成功".to_string()));
            r
        }
        Err(e) => {
            checks.push(("YAML 语法".to_string(), false, e.to_string()));
            return Ok(VerifyResult {
                valid: false,
                checks,
            });
        }
    };

    // 2. name 非空
    let ok = !recipe.name.is_empty();
    checks.push((
        "name 字段".to_string(),
        ok,
        if ok {
            recipe.name.clone()
        } else {
            "为空".to_string()
        },
    ));

    // 3. version 非空
    let ok = !recipe.version.is_empty();
    checks.push((
        "version 字段".to_string(),
        ok,
        if ok {
            recipe.version.clone()
        } else {
            "为空".to_string()
        },
    ));

    // 4. description 非空
    let ok = !recipe.description.is_empty();
    checks.push((
        "description 字段".to_string(),
        ok,
        if ok { "存在".to_string() } else { "为空".to_string() },
    ));

    // 5. 至少一个平台已配置
    let platform_count = [
        recipe.platforms.windows.is_some(),
        recipe.platforms.linux.is_some(),
        recipe.platforms.darwin.is_some(),
    ]
    .iter()
    .filter(|&&b| b)
    .count();
    checks.push((
        "平台覆盖".to_string(),
        platform_count > 0,
        format!("{} 个平台", platform_count),
    ));

    // 6. 逐平台检查 url/sha256/install_type
    let platforms: Vec<(&str, Option<&PlatformConfig>)> = vec![
        ("windows", recipe.platforms.windows.as_ref()),
        ("linux", recipe.platforms.linux.as_ref()),
        ("darwin", recipe.platforms.darwin.as_ref()),
    ];

    for (os, cfg) in &platforms {
        if let Some(cfg) = cfg {
            // url
            let url_ok = !cfg.url.is_empty();
            checks.push((
                format!("{}.url", os),
                url_ok,
                if url_ok { cfg.url.clone() } else { "为空".to_string() },
            ));

            // sha256 长度
            let sha_ok = cfg.sha256.len() == 64;
            checks.push((
                format!("{}.sha256", os),
                sha_ok,
                format!("{} 字符", cfg.sha256.len()),
            ));

            // install_type 合法
            let type_ok = VALID_INSTALL_TYPES.contains(&cfg.install_type.as_str());
            checks.push((
                format!("{}.install_type", os),
                type_ok,
                cfg.install_type.clone(),
            ));
        }
    }

    // 7. maintainer 警告（非阻塞）
    if recipe.maintainer.is_none() {
        checks.push((
            "maintainer".to_string(),
            true, // 不影响 valid
            "[WARN] 未填写维护者信息，社区配方建议填写".to_string(),
        ));
    }

    let valid = checks.iter().all(|(_, ok, _)| *ok);

    Ok(VerifyResult { valid, checks })
}
