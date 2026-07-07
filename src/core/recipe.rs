use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use super::config_gen::{AppConfig, apply_configs, remove_configs};
use super::downloader;
use super::manifest::{InstallRecord, Manifest};
use super::path_mgr;
use super::{CoreError, Result, envs_dir, temp_dir};
use crate::output::OutputFormatter;

// ============================================================
// 配方定义
// ============================================================

/// 安装配方 — 描述一个工具的完整安装过程
#[derive(Debug, Clone)]
pub struct Recipe {
    /// 包标识符（如 "python3.11"）
    pub name: String,
    /// 版本号（如 "3.11.9"）
    pub version: String,
    /// 显示名称（如 "Python 3.11.9"）
    pub display_name: String,
    /// 下载 URL
    pub download_url: String,
    /// 预期 SHA256（小写十六进制）
    pub sha256: String,
    /// 二进制文件所在目录（相对于安装目录，如 "."）
    pub bin_dir: String,
    /// 需要设置的环境变量（变量名, 值模板）
    pub env_vars: Vec<(String, String)>,
    /// 需要生成的配置文件
    pub configs: Vec<AppConfig>,
    /// 安装后处理步骤
    pub post_install: Option<PostInstall>,
}

/// 安装后处理步骤集合
#[derive(Debug, Clone)]
pub struct PostInstall {
    pub steps: Vec<PostInstallStep>,
}

/// 安装后处理步骤
#[derive(Debug, Clone)]
pub enum PostInstallStep {
    /// 下载文件并执行（如 get-pip.py）
    DownloadAndRun {
        url: String,
        args: Vec<String>,
    },
    /// 修改已有文件
    ModifyFile {
        rel_path: String,
        action: ModifyAction,
    },
}

/// 文件修改操作
#[derive(Debug, Clone)]
pub enum ModifyAction {
    /// 取消注释包含指定模式的行（删除行首的 #）
    UncommentLine { pattern: String },
    /// 追加一行内容
    AppendLine { content: String },
    /// 替换文件全部内容
    ReplaceContent { content: String },
}

// ============================================================
// 配方注册表
// ============================================================

/// 根据包名查找配方
pub fn resolve(name: &str) -> Option<Recipe> {
    match name {
        "python3.11" => Some(python311_recipe()),
        _ => None,
    }
}

/// 列出所有已知配方
pub fn list_recipes() -> Vec<Recipe> {
    vec![python311_recipe()]
}

// ============================================================
// 安装执行器
// ============================================================

/// 执行配方安装（完整流程）
pub async fn install(recipe: &Recipe, formatter: &OutputFormatter) -> Result<()> {
    let start = Instant::now();
    let install_dir = envs_dir().join(&recipe.name);

    // 1. 创建安装目录
    if install_dir.exists() {
        fs::remove_dir_all(&install_dir)?;
    }
    fs::create_dir_all(&install_dir)?;

    // 2. 备份当前 PATH
    let path_backup = path_mgr::backup()?;

    // 3. 下载压缩包
    let archive_name = recipe
        .download_url
        .rsplit('/')
        .next()
        .unwrap_or("archive");
    let temp_archive = temp_dir().join(archive_name);
    let dl_result = downloader::download(&recipe.download_url, &temp_archive).await?;
    formatter.output(
        &format!("✅ 下载完成: {} ({:.1} MB)", archive_name, dl_result.file_size as f64 / 1_048_576.0),
        Some(serde_json::json!({"message": "download_complete"})),
    );

    // 4. 校验 SHA256
    downloader::verify_sha256(&temp_archive, &recipe.sha256)?;
    formatter.output("✅ SHA256 校验通过", Some(serde_json::Value::Null));

    // 5. 解压到安装目录
    let extracted = downloader::extract(&temp_archive, &install_dir)?;
    formatter.output(
        &format!("✅ 解压完成: {} 个文件", extracted.len()),
        Some(serde_json::Value::Null),
    );

    // 清理临时压缩包
    let _ = fs::remove_file(&temp_archive);

    // 6. 执行安装后处理
    if let Some(ref post) = recipe.post_install {
        execute_post_install(post, &install_dir, formatter).await?;
    }

    // 7. 生成配置文件
    let config_files = apply_configs(&install_dir, &recipe.configs)?;
    for cf in &config_files {
        formatter.output(
            &format!("✅ 配置文件: {}", cf.display()),
            Some(serde_json::Value::Null),
        );
    }

    // 8. 添加到 PATH
    let bin_path = install_dir.join(&recipe.bin_dir);
    path_mgr::add(&bin_path)?;

    // 9. 记录到清单
    let manifest = Manifest::open()?;
    let record = InstallRecord {
        id: uuid::Uuid::new_v4().to_string(),
        name: recipe.name.clone(),
        version: Some(recipe.version.clone()),
        install_path: install_dir.to_string_lossy().to_string(),
        archive_url: Some(recipe.download_url.clone()),
        sha256: Some(recipe.sha256.clone()),
        path_entries: vec![bin_path.to_string_lossy().to_string()],
        config_files: config_files.iter().map(|p| p.to_string_lossy().to_string()).collect(),
        installed_at: chrono::Utc::now().to_rfc3339(),
        original_path: Some(path_backup),
        env_vars_backup: serde_json::json!({}),
    };
    let record_id = manifest.add(&record)?;

    let duration = start.elapsed();
    formatter.output(
        &format!(
            "🎉 {} 安装成功！\n  路径: {}\n  耗时: {:.1}s",
            recipe.display_name,
            install_dir.display(),
            duration.as_secs_f64()
        ),
        Some(serde_json::json!({
            "status": "success",
            "action": "install",
            "package": recipe.name,
            "version": recipe.version,
            "install_path": install_dir.to_string_lossy(),
            "path_entries": record.path_entries,
            "config_files": record.config_files,
            "manifest_id": record_id,
            "duration_ms": duration.as_millis() as u64,
        })),
    );

    Ok(())
}

// ============================================================
// 卸载执行器
// ============================================================

/// 执行完整卸载（按清单回滚）
pub async fn uninstall(package: &str, formatter: &OutputFormatter) -> Result<()> {
    let manifest = Manifest::open()?;

    // 1. 获取安装记录
    let record = manifest
        .get(package)?
        .ok_or_else(|| CoreError::Other(format!("'{}' 未安装", package)))?;

    let install_path = Path::new(&record.install_path);

    // 2. 从 PATH 中移除
    for entry in &record.path_entries {
        let path = Path::new(entry);
        path_mgr::remove(path)?;
    }

    // 3. 删除配置文件
    if let Some(recipe) = resolve(package) {
        remove_configs(install_path, &recipe.configs)?;
    }

    // 4. 删除安装目录
    if install_path.exists() {
        fs::remove_dir_all(install_path)?;
    }

    // 5. 从清单中删除记录
    manifest.remove(package)?;

    formatter.output(
        &format!("🗑️ '{}' 卸载完成，所有文件已清理。", package),
        Some(serde_json::json!({
            "status": "success",
            "action": "uninstall",
            "package": package,
            "removed_path": record.install_path,
        })),
    );

    Ok(())
}

// ============================================================
// 安装后处理执行
// ============================================================

/// 执行安装后处理步骤（异步，因为可能需要下载）
async fn execute_post_install(
    post: &PostInstall,
    install_dir: &Path,
    formatter: &OutputFormatter,
) -> Result<()> {
    for step in &post.steps {
        match step {
            PostInstallStep::DownloadAndRun { url, args } => {
                execute_download_and_run(url, args, install_dir, formatter).await?;
            }
            PostInstallStep::ModifyFile { rel_path, action } => {
                execute_modify_file(rel_path, action, install_dir)?;
            }
        }
    }
    Ok(())
}

/// 下载文件并执行
async fn execute_download_and_run(
    url: &str,
    args: &[String],
    install_dir: &Path,
    formatter: &OutputFormatter,
) -> Result<()> {
    let file_name = url.rsplit('/').next().unwrap_or("script");
    let dest = install_dir.join(file_name);

    // 下载脚本
    downloader::download(url, &dest).await?;

    formatter.output(
        &format!("✅ 下载脚本: {}", file_name),
        Some(serde_json::json!({"message": "script_downloaded"})),
    );

    // 构建命令
    let python_exe = install_dir.join("python.exe");
    let mut cmd = Command::new(&python_exe);
    cmd.arg(&dest);
    cmd.args(args);
    cmd.current_dir(install_dir);

    formatter.output(
        &format!("⏳ 执行: python {} {}", file_name, args.join(" ")),
        Some(serde_json::Value::Null),
    );

    let output = cmd
        .output()
        .map_err(|e| CoreError::Other(format!("执行脚本失败: {}", e)))?;

    if output.status.success() {
        formatter.output("✅ 脚本执行完成", Some(serde_json::Value::Null));
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CoreError::Other(format!("脚本执行失败: {}", stderr)));
    }

    // 清理脚本文件
    let _ = fs::remove_file(&dest);

    Ok(())
}

/// 执行文件修改操作
fn execute_modify_file(
    rel_path: &str,
    action: &ModifyAction,
    install_dir: &Path,
) -> Result<()> {
    let file_path = install_dir.join(rel_path);

    let content = fs::read_to_string(&file_path)?;
    let new_content = match action {
        ModifyAction::UncommentLine { pattern } => {
            content
                .lines()
                .map(|line| {
                    let trimmed = line.trim_start();
                    if trimmed.starts_with('#') && trimmed[1..].trim_start().starts_with(pattern) {
                        // 取消注释：删除行首的 # 和紧随的空格
                        let after_hash = &trimmed[1..];
                        format!("{}{}", &line[..line.len() - trimmed.len()], after_hash.trim_start())
                    } else {
                        line.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        ModifyAction::AppendLine { content: line } => {
            if content.ends_with('\n') {
                format!("{}{}", content, line)
            } else {
                format!("{}\n{}", content, line)
            }
        }
        ModifyAction::ReplaceContent { content: new } => new.clone(),
    };

    fs::write(&file_path, new_content)?;
    Ok(())
}

// ============================================================
// Python 3.11.9 配方（Windows embeddable + get-pip）
// ============================================================

/// Python 3.11.9 安装配方
///
/// 使用 Windows embeddable zip 包，安装后通过 get-pip.py 引导安装 pip。
fn python311_recipe() -> Recipe {
    let version = "3.11.9";
    let short_version = "311"; // major.minor 无点，用于 ._pth 和 dll 文件名

    Recipe {
        name: "python3.11".to_string(),
        version: version.to_string(),
        display_name: format!("Python {}", version),
        download_url: format!(
            "https://www.python.org/ftp/python/{}/python-{}-embed-amd64.zip",
            version, version
        ),
        // 注意：首次安装时会计算实际 SHA256 并验证
        // 此值为占位，如果首次下载校验失败需要更新
        sha256: python311_sha256(),
        bin_dir: ".".to_string(),
        env_vars: vec![],
        configs: vec![super::config_gen::pip_mirror_config()],
        post_install: Some(PostInstall {
            steps: vec![
                // 修改 ._pth 文件，启用 import site（pip 需要此功能）
                PostInstallStep::ModifyFile {
                    rel_path: format!("python{}._pth", short_version),
                    action: ModifyAction::UncommentLine {
                        pattern: "import site".to_string(),
                    },
                },
                // 下载并运行 get-pip.py 安装 pip（使用官方 PyPI 源引导）
                PostInstallStep::DownloadAndRun {
                    url: "https://bootstrap.pypa.io/get-pip.py".to_string(),
                    args: vec!["--index-url".to_string(), "https://pypi.org/simple".to_string()],
                },
            ],
        }),
    }
}

/// Python 3.11.9 embeddable zip SHA256
/// URL: https://www.python.org/ftp/python/3.11.9/python-3.11.9-embed-amd64.zip
#[cfg(target_os = "windows")]
fn python311_sha256() -> String {
    // https://www.python.org/ftp/python/3.11.9/python-3.11.9-embed-amd64.zip
    "009d6bf7e3b2ddca3d784fa09f90fe54336d5b60f0e0f305c37f400bf83cfd3b".to_string()
}

#[cfg(not(target_os = "windows"))]
fn python311_sha256() -> String {
    // 非 Windows 平台暂不支持 embeddable 包
    "NOT_AVAILABLE_FOR_NON_WINDOWS".to_string()
}
