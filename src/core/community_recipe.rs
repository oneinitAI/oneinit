// 社区配方系统 — 按 社区配方.md 实现
//
// 声明式 YAML 配方格式，支持多平台、模板变量、安全提醒。
// 配方文件存放在 ~/.oneinit/recipes/*.yaml

use std::collections::BTreeMap;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

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

// ============================================================
// 安装与卸载（含安全提醒）
// ============================================================

/// 安装社区配方
///
/// 安全流程（按 社区配方.md 第四节要求）：
/// 1. 醒目显示下载来源、SHA256、写入目录、执行的命令
/// 2. 等待用户输入 y 确认
/// 3. 下载 -> 校验 -> 解压/安装 -> post_install -> PATH -> Manifest
pub async fn install(
    recipe: &CommunityRecipe,
    formatter: &OutputFormatter,
) -> Result<()> {
    let start = Instant::now();

    // 获取当前平台配置
    let platform_cfg = current_platform_config(recipe).ok_or_else(|| {
        CoreError::Other(format!("配方 '{}' 不支持当前平台", recipe.name))
    })?;

    let install_dir = envs_dir().join(&platform_cfg.install_path);

    // ====== 安全提醒（社区配方.md 第四节核心要求）======
    formatter.output("", Some(serde_json::Value::Null));
    formatter.output(
        "========== [SECURITY] 安装确认 ==========",
        Some(serde_json::Value::Null),
    );
    formatter.output(
        &format!("[SECURITY] 名称:     {} v{}", recipe.name, recipe.version),
        Some(serde_json::Value::Null),
    );
    formatter.output(
        &format!("[SECURITY] 下载来源: {}", platform_cfg.url),
        Some(serde_json::Value::Null),
    );
    formatter.output(
        &format!("[SECURITY] SHA256:   {}", &platform_cfg.sha256[..16]),
        Some(serde_json::Value::Null),
    );
    formatter.output(
        &format!("[SECURITY] 安装目录: {}", install_dir.display()),
        Some(serde_json::Value::Null),
    );

    // 显示将要执行的命令
    if let Some(ref post) = recipe.post_install {
        if let Some(ref cmds) = post.commands {
            for cmd in cmds {
                let rendered = render_template(cmd, &install_dir);
                formatter.output(
                    &format!("[SECURITY] 将执行:   {}", rendered),
                    Some(serde_json::Value::Null),
                );
            }
        }
        if let Some(ref configs) = post.config_files {
            for cf in configs {
                formatter.output(
                    &format!("[SECURITY] 将写入:   {}", cf.path),
                    Some(serde_json::Value::Null),
                );
            }
        }
    }

    formatter.output(
        &format!("[SECURITY] 安装路径: {}",
            platform_cfg.path_add.iter()
                .map(|p| render_template(p, &install_dir))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Some(serde_json::Value::Null),
    );
    formatter.output(
        "========================================",
        Some(serde_json::Value::Null),
    );

    // 等待用户确认
    print!("[SECURITY] 输入 y 确认安装，其他键取消: ");
    io::stdout().flush()?;
    let confirmed = wait_for_confirmation();
    if !confirmed {
        formatter.output("[CANCEL] 已取消安装", Some(serde_json::Value::Null));
        return Ok(());
    }

    // ====== 执行安装 ======
    // 1. 创建安装目录
    if install_dir.exists() {
        std::fs::remove_dir_all(&install_dir)?;
    }
    std::fs::create_dir_all(&install_dir)?;

    // 2. 备份 PATH
    let path_backup = super::path_mgr::backup()?;

    // 3. 下载
    let archive_name = platform_cfg.url.rsplit('/').next().unwrap_or("archive");
    let temp_archive = super::temp_dir().join(archive_name);
    let dl_result = super::downloader::download(&platform_cfg.url, &temp_archive).await?;
    formatter.output(
        &format!("[OK] 下载完成: {} ({:.1} MB)", archive_name, dl_result.file_size as f64 / 1_048_576.0),
        Some(serde_json::Value::Null),
    );

    // 4. SHA256 校验
    super::downloader::verify_sha256(&temp_archive, &platform_cfg.sha256)?;
    formatter.output("[OK] SHA256 校验通过", Some(serde_json::Value::Null));

    // 5. 按 install_type 分派安装
    match platform_cfg.install_type.as_str() {
        "zip_extract" | "tar_extract" => {
            super::downloader::extract(&temp_archive, &install_dir)?;
            formatter.output("[OK] 解压完成", Some(serde_json::Value::Null));
        }
        "exe_silent" => {
            // 静默安装：运行 exe 并等待
            let args = platform_cfg.install_args.clone().unwrap_or_default();
            let status = Command::new(&temp_archive)
                .args(&args)
                .status()
                .map_err(|e| CoreError::Other(format!("执行安装程序失败: {}", e)))?;
            if !status.success() {
                return Err(CoreError::Other(format!(
                    "安装程序退出码: {:?}",
                    status.code()
                )));
            }
            formatter.output("[OK] 静默安装完成", Some(serde_json::Value::Null));
        }
        "binary_copy" => {
            std::fs::copy(&temp_archive, install_dir.join(archive_name))?;
            formatter.output("[OK] 文件复制完成", Some(serde_json::Value::Null));
        }
        other => {
            return Err(CoreError::Other(format!(
                "install_type '{}' 暂不支持（当前支持: zip_extract, tar_extract, exe_silent, binary_copy）",
                other
            )));
        }
    }

    // 清理临时文件
    let _ = std::fs::remove_file(&temp_archive);

    // 6. 执行 post_install
    if let Some(ref post) = recipe.post_install {
        execute_post_install(post, &install_dir, formatter)?;
    }

    // 7. 添加 path_add 到 PATH
    let mut path_entries = Vec::new();
    for path_template in &platform_cfg.path_add {
        let rendered = render_template(path_template, &install_dir);
        let path = PathBuf::from(&rendered);
        super::path_mgr::add(&path)?;
        path_entries.push(rendered);
    }

    // 8. 记录到 Manifest
    let manifest = super::manifest::Manifest::open()?;
    let record = super::manifest::InstallRecord {
        id: uuid::Uuid::new_v4().to_string(),
        name: recipe.name.clone(),
        version: Some(recipe.version.clone()),
        install_path: install_dir.to_string_lossy().to_string(),
        archive_url: Some(platform_cfg.url.clone()),
        sha256: Some(platform_cfg.sha256.clone()),
        path_entries,
        config_files: vec![],
        installed_at: chrono::Utc::now().to_rfc3339(),
        original_path: Some(path_backup),
        env_vars_backup: serde_json::json!({}),
    };
    let record_id = manifest.add(&record)?;

    let duration = start.elapsed();
    formatter.output(
        &format!(
            "[SUCCESS] {} v{} 安装成功 ({:.1}s)",
            recipe.name,
            recipe.version,
            duration.as_secs_f64()
        ),
        Some(serde_json::json!({
            "status": "success",
            "action": "install",
            "package": recipe.name,
            "version": recipe.version,
            "install_path": record.install_path,
            "manifest_id": record_id,
            "duration_ms": duration.as_millis() as u64,
        })),
    );

    Ok(())
}

/// 卸载社区配方
pub async fn uninstall(name: &str, formatter: &OutputFormatter) -> Result<()> {
    let manifest = super::manifest::Manifest::open()?;
    let record = manifest
        .get(name)?
        .ok_or_else(|| CoreError::Other(format!("'{}' 未安装", name)))?;

    // 从 PATH 移除
    for entry in &record.path_entries {
        super::path_mgr::remove(Path::new(entry))?;
    }

    // 删除安装目录
    let install_path = Path::new(&record.install_path);
    if install_path.exists() {
        std::fs::remove_dir_all(install_path)?;
    }

    // 从清单删除
    manifest.remove(name)?;

    formatter.output(
        &format!("[DEL] '{}' 卸载完成", name),
        Some(serde_json::json!({
            "status": "success",
            "action": "uninstall",
            "package": name,
            "removed_path": record.install_path,
        })),
    );

    Ok(())
}

// ============================================================
// post_install 执行
// ============================================================

fn execute_post_install(
    post: &PostInstallConfig,
    install_dir: &Path,
    formatter: &OutputFormatter,
) -> Result<()> {
    // 1. 生成配置文件
    if let Some(ref configs) = post.config_files {
        for cf in configs {
            let full_path = install_dir.join(&cf.path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let content = render_template(&cf.template, install_dir);
            std::fs::write(&full_path, &content)?;
            formatter.output(
                &format!("[OK] 配置文件: {}", full_path.display()),
                Some(serde_json::Value::Null),
            );
        }
    }

    // 2. 执行命令
    if let Some(ref commands) = post.commands {
        for cmd in commands {
            let rendered = render_template(cmd, install_dir);
            formatter.output(
                &format!("[RUN] {}", rendered),
                Some(serde_json::Value::Null),
            );

            let output = if cfg!(target_os = "windows") {
                Command::new("cmd").args(["/C", &rendered]).output()
            } else {
                Command::new("sh").args(["-c", &rendered]).output()
            };

            match output {
                Ok(out) => {
                    if !out.status.success() {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        formatter.output(
                            &format!("  [WARN] 命令退出码: {:?} {}", out.status.code(), stderr.trim()),
                            Some(serde_json::Value::Null),
                        );
                    }
                }
                Err(e) => {
                    formatter.output(
                        &format!("  [WARN] 执行失败: {}", e),
                        Some(serde_json::Value::Null),
                    );
                }
            }
        }
    }

    Ok(())
}

/// 等待用户输入 y 确认
fn wait_for_confirmation() -> bool {
    let mut buf = [0u8; 1];
    match io::stdin().read(&mut buf) {
        Ok(_) => buf[0] == b'y' || buf[0] == b'Y',
        Err(_) => false,
    }
}
