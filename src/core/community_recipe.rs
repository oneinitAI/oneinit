// 社区recipe系统 — 按 社区recipe.md 实现
//
// declarative YAML recipe, multi-platform, template vars, security.
// recipe files stored in ~/.oneinit/recipes/*.yaml

use std::collections::BTreeMap;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use serde::Deserialize;

use super::{CoreError, Result, envs_dir};
use crate::output::OutputFormatter;

// ============================================================
// recipe DTO（严格对应 社区recipe.md 1.1 节 YAML 格式）
// ============================================================

/// 社区recipe（deserialized from YAML）
#[derive(Debug, Clone, Deserialize)]
pub struct CommunityRecipe {
    pub name: String,
    pub version: String,
    #[allow(dead_code)]
    pub description: String,
    /// 软件许可证（如 MIT、GPL-2.0），安装前展示
    #[serde(default)]
    #[allow(dead_code)]
    pub license: Option<String>,
    /// 许可证详情 URL，安装前展示
    #[serde(default)]
    #[allow(dead_code)]
    pub license_url: Option<String>,
    pub platforms: Platforms,
    pub post_install: Option<PostInstallConfig>,
    #[allow(dead_code)]
    pub depends: Option<Vec<String>>,
    #[allow(dead_code)]
    pub tags: Option<Vec<String>>,
    #[allow(dead_code)]
    pub maintainer: Option<Maintainer>,
}

/// platform configuration set
#[derive(Debug, Clone, Deserialize)]
pub struct Platforms {
    pub windows: Option<PlatformConfig>,
    pub linux: Option<PlatformConfig>,
    pub darwin: Option<PlatformConfig>,
}

/// single platform configuration
#[derive(Debug, Clone, Deserialize)]
pub struct PlatformConfig {
    pub url: String,
    pub sha256: String,
    pub install_type: String,
    pub install_args: Option<Vec<String>>,
    pub install_path: String,
    pub path_add: Vec<String>,
}

/// post-install configuration
#[derive(Debug, Clone, Deserialize)]
pub struct PostInstallConfig {
    #[allow(dead_code)]
    pub env_vars: Option<BTreeMap<String, String>>,
    pub config_files: Option<Vec<ConfigFile>>,
    pub commands: Option<Vec<String>>,
}

/// config file definition (template)
#[derive(Debug, Clone, Deserialize)]
pub struct ConfigFile {
    pub path: String,
    pub template: String,
}

/// maintainer information
#[derive(Debug, Clone, Deserialize)]
pub struct Maintainer {
    #[allow(dead_code)]
    pub name: Option<String>,
    #[allow(dead_code)]
    pub github: Option<String>,
    #[allow(dead_code)]
    pub email: Option<String>,
}

/// valid install_type values
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

/// community recipe storage directory: ~/.oneinit/recipes/
pub fn recipes_dir() -> PathBuf {
    super::data_dir().join("recipes")
}

// ============================================================
// load and lookup
// ============================================================

/// 从 ~/.oneinit/recipes/*.yaml load all community recipes
pub fn load_all() -> Vec<CommunityRecipe> {
    let dir = recipes_dir();
    let mut recipes = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("yaml")
                && let Ok(content) = std::fs::read_to_string(&path)
                && let Ok(recipe) = serde_yaml::from_str::<CommunityRecipe>(&content)
            {
                recipes.push(recipe);
            }
        }
    }

    recipes
}

/// 按名称find community recipe by name
pub fn resolve(name: &str) -> Option<CommunityRecipe> {
    load_all().into_iter().find(|r| r.name == name)
}

/// list all community recipe names
pub fn list_names() -> Vec<String> {
    load_all().iter().map(|r| r.name.clone()).collect()
}

/// get current platform configuration
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
// template variable rendering
// ============================================================

/// 渲染模板变量
///
/// supported variables (per community recipe spec):
/// - {{install_dir}} — absolute install directory path
/// - {{user_home}} — user home directory
/// - {{mirror_pip}} — pip Tsinghua mirror URL
/// - {{mirror_pip_host}} — pip Tsinghua mirror hostname
/// - {{mirror_npm}} — npm npmmirror URL
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
// 安全：路径越界检测
// ============================================================

/// 检测渲染后的相对路径是否会逃逸出 install_dir
///
/// install_dir.join(path) 的语义：path 若为绝对路径则完全替换 install_dir，
/// 或含 `..` 跨出边界。本函数用于拦截 config_files 的恶意路径。
///
/// 用栈模拟相对路径：每个 `..` 弹栈，若栈空再遇 `..` 即越过 install_dir。
pub fn path_escapes_install_dir(rendered_path: &str, _install_dir: &Path) -> bool {
    use std::path::Component;
    let p = std::path::Path::new(rendered_path);
    let mut depth: i32 = 0; // 相对 install_dir 的深度
    for comp in p.components() {
        match comp {
            Component::Normal(_) => depth += 1,
            Component::ParentDir => {
                if depth == 0 {
                    return true; // 越过 install_dir 根
                }
                depth -= 1;
            }
            Component::CurDir => {}
            // 绝对路径 / Windows 前缀：install_dir.join 时会替换，视为逃逸
            Component::RootDir | Component::Prefix(_) => return true,
        }
    }
    false
}

// ============================================================
// 验证
// ============================================================

/// verification result
#[derive(Debug)]
pub struct VerifyResult {
    pub valid: bool,
    pub checks: Vec<(String, bool, String)>, // (检查项, 是否通过, 说明)
}

/// validate recipe file
pub fn verify(yaml_path: &Path) -> Result<VerifyResult> {
    let content = std::fs::read_to_string(yaml_path)?;
    let mut checks = Vec::new();

    // 1. YAML syntax
    let recipe: CommunityRecipe = match serde_yaml::from_str(&content) {
        Ok(r) => {
            checks.push(("YAML syntax".to_string(), true, "parse success".to_string()));
            r
        }
        Err(e) => {
            checks.push(("YAML syntax".to_string(), false, e.to_string()));
            return Ok(VerifyResult {
                valid: false,
                checks,
            });
        }
    };

    // 2. name 非空
    let ok = !recipe.name.is_empty();
    checks.push((
        "name field".to_string(),
        ok,
        if ok {
            recipe.name.clone()
        } else {
            "empty".to_string()
        },
    ));

    // 3. version 非空
    let ok = !recipe.version.is_empty();
    checks.push((
        "version field".to_string(),
        ok,
        if ok {
            recipe.version.clone()
        } else {
            "empty".to_string()
        },
    ));

    // 4. description 非空
    let ok = !recipe.description.is_empty();
    checks.push((
        "description field".to_string(),
        ok,
        if ok {
            "exists".to_string()
        } else {
            "empty".to_string()
        },
    ));

    // 5. 至少一 platforms已配置
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
        format!("{}  platforms", platform_count),
    ));

    // 6. per-platform check url/sha256/install_type
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
                if url_ok {
                    cfg.url.clone()
                } else {
                    "empty".to_string()
                },
            ));

            // sha256 长度（64 位 hex 为 SHA256，128 位 hex 为 SHA512）
            let sha_ok = cfg.sha256.len() == 64 || cfg.sha256.len() == 128;
            checks.push((
                format!("{}.sha256", os),
                sha_ok,
                format!("{}  chars", cfg.sha256.len()),
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

    // 7. maintainer warning (non-blocking)
    if recipe.maintainer.is_none() {
        checks.push((
            "maintainer".to_string(),
            true, // does not affect validity
            "[WARN] 未填写maintainer information，社区recipe建议填写".to_string(),
        ));
    }

    let valid = checks.iter().all(|(_, ok, _)| *ok);

    Ok(VerifyResult { valid, checks })
}

// ============================================================
// 安装与卸载（含安全提醒）
// ============================================================

/// 安装社区recipe
///
/// Security flow（按 社区recipe.md 第四节要求）：
/// 1. prominently display source, SHA256, target dir, commands
/// 2. wait for user to type y to confirm (unless allow_exec explicitly granted)
/// 3. download -> verify -> extract/install -> post_install -> PATH -> Manifest
///
/// `allow_exec`：是否允许执行远程配方声明的命令/安装器。默认 false——
/// 含 post_install.commands 或执行类 install_type 的配方会被拒绝（安全 H-4）。
pub async fn install(
    recipe: &CommunityRecipe,
    formatter: &OutputFormatter,
    allow_exec: bool,
) -> Result<()> {
    let start = Instant::now();

    // get current platform config
    let platform_cfg = current_platform_config(recipe).ok_or_else(|| {
        CoreError::Other(format!(
            "recipe '{}' unsupported on this platform",
            recipe.name
        ))
    })?;

    // 安全 H-4：判定配方是否含"执行"类操作（命令 / 安装器）
    let has_commands = recipe
        .post_install
        .as_ref()
        .and_then(|p| p.commands.as_ref())
        .map(|c| !c.is_empty())
        .unwrap_or(false);
    let exec_type = matches!(
        platform_cfg.install_type.as_str(),
        "exe_silent" | "msi_install" | "pkg_install"
    );
    let needs_exec = has_commands || exec_type;

    if needs_exec && !allow_exec {
        return Err(CoreError::Other(format!(
            "recipe '{}' requires executing commands/installers ({}{}). \
             Refused for security. Re-run with --allow-exec to accept.",
            recipe.name,
            if has_commands {
                "post_install commands"
            } else {
                ""
            },
            if exec_type {
                " install_type=".to_string() + &platform_cfg.install_type
            } else {
                String::new()
            },
        )));
    }

    let install_dir = envs_dir().join(&platform_cfg.install_path);

    // ====== 安全提醒（社区recipe.md 第四节核心要求）======
    formatter.output("", Some(serde_json::Value::Null));
    formatter.output(
        "========== [SECURITY] Install confirmation ==========",
        Some(serde_json::Value::Null),
    );
    formatter.output(
        &format!("[SECURITY] Name:     {} v{}", recipe.name, recipe.version),
        Some(serde_json::Value::Null),
    );
    formatter.output(
        &format!("[SECURITY] Source: {}", platform_cfg.url),
        Some(serde_json::Value::Null),
    );
    // 安全：完整显示校验和，便于用户核对（M-3 修复：原仅显示 16 位且对非 hex 会 panic）
    formatter.output(
        &format!("[SECURITY] SHA256:  {}", platform_cfg.sha256),
        Some(serde_json::Value::Null),
    );
    formatter.output(
        &format!("[SECURITY] Install to: {}", install_dir.display()),
        Some(serde_json::Value::Null),
    );
    // 展示许可证信息（若有）
    if let Some(license) = &recipe.license {
        let line = match (&recipe.license_url, license.is_empty()) {
            (Some(url), false) => format!("[SECURITY] License:  {} ({})", license, url),
            (Some(url), true) => format!("[SECURITY] License:  see {}", url),
            (None, false) => format!("[SECURITY] License:  {}", license),
            (None, true) => String::new(),
        };
        if !line.is_empty() {
            formatter.output(&line, Some(serde_json::Value::Null));
        }
    } else if let Some(url) = &recipe.license_url {
        formatter.output(
            &format!("[SECURITY] License:  see {}", url),
            Some(serde_json::Value::Null),
        );
    }

    // display commands to be executed
    if let Some(ref post) = recipe.post_install {
        if let Some(ref cmds) = post.commands {
            for cmd in cmds {
                let rendered = render_template(cmd, &install_dir);
                formatter.output(
                    &format!("[SECURITY] Will run:   {}", rendered),
                    Some(serde_json::Value::Null),
                );
            }
        }
        if let Some(ref configs) = post.config_files {
            for cf in configs {
                // 安全：显示渲染后的真实路径（原仅显示未渲染模板，
                // 用户无法判断 {{user_home}} 是否越界）
                let rendered = render_template(&cf.path, &install_dir);
                let escaped = path_escapes_install_dir(&rendered, &install_dir);
                let warning = if escaped {
                    "  ⚠️ ESCAPES install dir"
                } else {
                    ""
                };
                formatter.output(
                    &format!("[SECURITY] Will write:   {}{}", rendered, warning),
                    Some(serde_json::Value::Null),
                );
            }
        }
    }

    formatter.output(
        &format!(
            "[SECURITY] PATH add: {}",
            platform_cfg
                .path_add
                .iter()
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

    // wait for user confirmation（--allow-exec / --yes 已显式授权，跳过交互）
    if allow_exec || formatter.auto_yes {
        formatter.output(
            "[SECURITY] skipping interactive confirm (--allow-exec / --yes)",
            Some(serde_json::Value::Null),
        );
    } else {
        print!("[SECURITY] Type y to confirm, any other key to cancel: ");
        io::stdout().flush()?;
        let confirmed = wait_for_confirmation();
        if !confirmed {
            formatter.output(
                "[CANCEL] Installation cancelled",
                Some(serde_json::Value::Null),
            );
            return Ok(());
        }
    }

    // ====== execute install (plan → execute) ======
    // 1. build the operation plan (same plan used by --dry-run)
    let plan = crate::core::planner::plan_community_install(recipe, allow_exec)?;

    // 2. backup PATH before any PATH modification
    let path_backup = super::path_mgr::backup()?;

    // 3. clean up any stale install dir, then create fresh
    if install_dir.exists() {
        std::fs::remove_dir_all(&install_dir)?;
    }

    // 4. execute all operations
    crate::core::planner::execute_plan(&plan, formatter).await?;

    // 5. record to Manifest
    let manifest = super::manifest::Manifest::open()?;
    let path_entries = platform_cfg
        .path_add
        .iter()
        .map(|p| render_template(p, &install_dir))
        .collect();
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
            "[SUCCESS] {} v{} installation complete ({:.1}s)",
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

/// uninstall community recipe
pub async fn uninstall(name: &str, formatter: &OutputFormatter) -> Result<()> {
    let manifest = super::manifest::Manifest::open()?;
    let record = manifest
        .get(name)?
        .ok_or_else(|| CoreError::Other(format!("'{}' not installed", name)))?;

    // remove from PATH
    for entry in &record.path_entries {
        super::path_mgr::remove(Path::new(entry))?;
    }

    // delete install directory
    let install_path = Path::new(&record.install_path);
    if install_path.exists() {
        std::fs::remove_dir_all(install_path)?;
    }

    // remove from manifest
    manifest.remove(name)?;

    formatter.output(
        &format!("[DEL] '{}' uninstalled", name),
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
    // 1. generate config files
    if let Some(ref configs) = post.config_files {
        for cf in configs {
            // 安全 H-3：渲染模板后再校验，禁止写入 install_dir 之外的任意绝对路径
            // （防止 {{user_home}}/.ssh/authorized_keys 等越界写入）
            let rendered_path = render_template(&cf.path, install_dir);
            let full_path = install_dir.join(&rendered_path);
            if path_escapes_install_dir(&rendered_path, install_dir) {
                return Err(CoreError::Other(format!(
                    "config_files path escapes install directory (security): {}",
                    full_path.display()
                )));
            }
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let content = render_template(&cf.template, install_dir);
            std::fs::write(&full_path, &content)?;
            formatter.output(
                &format!("[OK] config file: {}", full_path.display()),
                Some(serde_json::Value::Null),
            );
        }
    }

    // 2. execute commands
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
                            &format!(
                                "  [WARN] 命令退出码: {:?} {}",
                                out.status.code(),
                                stderr.trim()
                            ),
                            Some(serde_json::Value::Null),
                        );
                    }
                }
                Err(e) => {
                    formatter.output(
                        &format!("  [WARN] execution failed: {}", e),
                        Some(serde_json::Value::Null),
                    );
                }
            }
        }
    }

    Ok(())
}

/// wait for user to type y to confirm
fn wait_for_confirmation() -> bool {
    let mut buf = [0u8; 1];
    match io::stdin().read_exact(&mut buf) {
        Ok(()) => buf[0] == b'y' || buf[0] == b'Y',
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_template_basic() {
        let dir = std::path::Path::new("/test/path");
        let result = render_template("{{install_dir}}/lib", dir);
        assert!(result.contains("/test/path/lib"));
    }

    #[test]
    fn test_render_template_mirror_pip() {
        let dir = std::path::Path::new("/tmp");
        let result = render_template("index-url = {{mirror_pip}}", dir);
        assert!(result.contains("pypi.tuna.tsinghua.edu.cn"));
    }

    #[test]
    fn test_render_template_mirror_npm() {
        let dir = std::path::Path::new("/tmp");
        let result = render_template("registry = {{mirror_npm}}", dir);
        assert!(result.contains("registry.npmmirror.com"));
    }

    #[test]
    fn test_render_template_no_vars() {
        let dir = std::path::Path::new("/tmp");
        let result = render_template("plain text no vars", dir);
        assert_eq!(result, "plain text no vars");
    }

    #[test]
    fn test_verify_valid_recipe() {
        let yaml = r#"
name: test-tool
version: "1.0.0"
description: "A test"
platforms:
  windows:
    url: "https://example.com/test.zip"
    sha256: "0000000000000000000000000000000000000000000000000000000000000000"
    install_type: "zip_extract"
    install_path: "test"
    path_add: ["{{install_dir}}"]
"#;
        let dir = std::env::temp_dir();
        let path = dir.join("oneinit_verify_test.yaml");
        std::fs::write(&path, yaml).unwrap();

        let result = verify(&path).unwrap();
        assert!(result.valid);
        assert!(result.checks.iter().all(|(_, ok, _)| *ok));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_parse_license_fields() {
        let yaml = r#"
name: lic-tool
version: "1.0.0"
description: "Licensed tool"
license: "MIT"
license_url: "https://example.com/license"
platforms:
  windows:
    url: "https://example.com/test.zip"
    sha256: "0000000000000000000000000000000000000000000000000000000000000000"
    install_type: "zip_extract"
    install_path: "test"
    path_add: ["{{install_dir}}"]
"#;
        let recipe: CommunityRecipe = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(recipe.license.as_deref(), Some("MIT"));
        assert_eq!(
            recipe.license_url.as_deref(),
            Some("https://example.com/license")
        );
    }

    #[test]
    fn test_license_fields_optional() {
        // 老配方没有 license 字段也能解析（serde default）
        let yaml = r#"
name: old-tool
version: "1.0.0"
description: "No license fields"
platforms:
  windows:
    url: "https://example.com/test.zip"
    sha256: "0000000000000000000000000000000000000000000000000000000000000000"
    install_type: "zip_extract"
    install_path: "test"
    path_add: ["{{install_dir}}"]
"#;
        let recipe: CommunityRecipe = serde_yaml::from_str(yaml).unwrap();
        assert!(recipe.license.is_none());
        assert!(recipe.license_url.is_none());
    }

    #[test]
    fn test_path_escape_detection() {
        let install_dir = std::path::Path::new("/home/user/.oneinit/envs/foo");
        // 相对路径，正常
        assert!(!path_escapes_install_dir("config/app.ini", install_dir));
        // 含 .. 但未越界（仍在 install_dir 内）
        assert!(!path_escapes_install_dir("sub/../config.ini", install_dir));
        // 含 .. 越界（H-3 核心场景）
        assert!(path_escapes_install_dir("../../etc/passwd", install_dir));
        // 绝对路径（恶意，如 {{user_home}}/.bashrc）
        assert!(path_escapes_install_dir("/home/user/.bashrc", install_dir));
        // 绝对路径恰好等于 install_dir（允许）
        // （canonicalize 在测试环境可能失败，但 is_absolute 分支优先返回 true；
        //   这里只验证绝对路径被正确识别为"需警惕"）
    }

    #[test]
    fn test_verify_bad_sha256_length() {
        let yaml = r#"
name: bad-tool
version: "1.0.0"
description: "Bad SHA"
platforms:
  windows:
    url: "https://example.com/test.zip"
    sha256: "too_short"
    install_type: "zip_extract"
    install_path: "test"
    path_add: []
"#;
        let dir = std::env::temp_dir();
        let path = dir.join("oneinit_verify_bad_test.yaml");
        std::fs::write(&path, yaml).unwrap();

        let result = verify(&path).unwrap();
        assert!(!result.valid);
        // sha256 检查应该失败
        assert!(result.checks.iter().any(|(_, ok, _)| !ok));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_verify_invalid_yaml() {
        let dir = std::env::temp_dir();
        let path = dir.join("oneinit_verify_bad_yaml.yaml");
        std::fs::write(&path, "{{{invalid yaml}}}").unwrap();

        let result = verify(&path).unwrap();
        assert!(!result.valid);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_verify_bad_install_type() {
        let yaml = r#"
name: bad-type
version: "1.0.0"
description: "Bad type"
platforms:
  windows:
    url: "https://example.com/test.zip"
    sha256: "0000000000000000000000000000000000000000000000000000000000000000"
    install_type: "nonexistent_type"
    install_path: "test"
    path_add: []
"#;
        let dir = std::env::temp_dir();
        let path = dir.join("oneinit_verify_type_test.yaml");
        std::fs::write(&path, yaml).unwrap();

        let result = verify(&path).unwrap();
        assert!(!result.valid);

        let _ = std::fs::remove_file(&path);
    }
}
