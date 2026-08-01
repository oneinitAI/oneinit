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
// recipe定义
// ============================================================

/// 安装recipe — 描述一个工具的完整安装过程
#[derive(Debug, Clone)]
pub struct Recipe {
    /// 包标识符（如 "python3.11"）
    pub name: String,
    /// 版本号（如 "3.11.9"）
    pub version: String,
    /// 显示名称（如 "Python 3.11.9"）
    pub display_name: String,
    /// download URL
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
    /// download文件并执行（如 get-pip.py）
    DownloadAndRun { url: String, args: Vec<String> },
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
// recipe注册表
// ============================================================

/// 根据包名查找recipe
pub fn resolve(name: &str) -> Option<Recipe> {
    match name {
        "python3.11" => Some(python311_recipe()),
        "node20" => Some(node20_recipe()),
        "go" => Some(go_recipe()),
        "java17" => Some(java17_recipe()),
        _ => None,
    }
}

/// 列出所有已知recipe
pub fn list_recipes() -> Vec<Recipe> {
    vec![
        python311_recipe(),
        node20_recipe(),
        go_recipe(),
        java17_recipe(),
    ]
}

// ============================================================
// 安装执行器
// ============================================================

/// 执行recipe安装（完整流程）
pub async fn install(recipe: &Recipe, formatter: &OutputFormatter) -> Result<()> {
    let start = Instant::now();
    let install_dir = envs_dir().join(&recipe.name);

    // 安全提醒：内置recipe仍需告知用户将download和修改的内容
    formatter.output("", Some(serde_json::Value::Null));
    formatter.output(
        "[SECURITY] 即将安装内置recipe，以下操作将被执行:",
        Some(serde_json::Value::Null),
    );
    formatter.output(
        &format!("[SECURITY]   tool: {} v{}", recipe.name, recipe.version),
        Some(serde_json::Value::Null),
    );
    formatter.output(
        &format!("[SECURITY]   download: {}", recipe.download_url),
        Some(serde_json::Value::Null),
    );
    formatter.output(
        &format!("[SECURITY]   dir: {}", install_dir.display()),
        Some(serde_json::Value::Null),
    );
    formatter.output(
        "[SECURITY]   操作: 修改 PATH 环境变量、写入配置文件",
        Some(serde_json::Value::Null),
    );

    // 1. create install directory
    if install_dir.exists() {
        fs::remove_dir_all(&install_dir)?;
    }
    fs::create_dir_all(&install_dir)?;

    // 2. 备份当前 PATH
    let path_backup = path_mgr::backup()?;

    // 3. download压缩包
    let archive_name = recipe.download_url.rsplit('/').next().unwrap_or("archive");
    let temp_archive = temp_dir().join(archive_name);
    let dl_result = downloader::download(&recipe.download_url, &temp_archive).await?;
    formatter.output(
        &format!(
            "[OK] download complete: {} ({:.1} MB)",
            archive_name,
            dl_result.file_size as f64 / 1_048_576.0
        ),
        Some(serde_json::json!({"message": "download_complete"})),
    );

    // 4. verify SHA256
    downloader::verify_sha256(&temp_archive, &recipe.sha256)?;
    formatter.output("[OK] SHA256 verified", Some(serde_json::Value::Null));

    // 5. 解压到安装目录
    let extracted = downloader::extract(&temp_archive, &install_dir)?;
    formatter.output(
        &format!("[OK] Extraction complete: {}  files", extracted.len()),
        Some(serde_json::Value::Null),
    );

    // 清理临时压缩包
    let _ = fs::remove_file(&temp_archive);

    // 6. execute install后处理
    if let Some(ref post) = recipe.post_install {
        execute_post_install(post, &install_dir, formatter).await?;
    }

    // 7. generate config files
    let config_files = apply_configs(&install_dir, &recipe.configs)?;
    for cf in &config_files {
        formatter.output(
            &format!("[OK] config file: {}", cf.display()),
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
        config_files: config_files
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
        installed_at: chrono::Utc::now().to_rfc3339(),
        original_path: Some(path_backup),
        env_vars_backup: serde_json::json!({}),
    };
    let record_id = manifest.add(&record)?;

    let duration = start.elapsed();
    formatter.output(
        &format!(
            "🎉 {} installation complete！\n  路径: {}\n  耗时: {:.1}s",
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
        .ok_or_else(|| CoreError::Other(format!("'{}' not installed", package)))?;

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

    // 4. delete install directory
    if install_path.exists() {
        fs::remove_dir_all(install_path)?;
    }

    // 5. 从清单中删除记录
    manifest.remove(package)?;

    formatter.output(
        &format!("'{}' uninstalled, all files cleaned.", package),
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

/// execute install后处理步骤（异步，因为可能需要download）
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

/// download文件并执行
async fn execute_download_and_run(
    url: &str,
    args: &[String],
    install_dir: &Path,
    formatter: &OutputFormatter,
) -> Result<()> {
    let file_name = url.rsplit('/').next().unwrap_or("script");
    let dest = install_dir.join(file_name);

    // download脚本
    downloader::download(url, &dest).await?;

    formatter.output(
        &format!("[OK] script downloaded: {}", file_name),
        Some(serde_json::json!({"message": "script_downloaded"})),
    );

    // 构建命令
    let python_exe = install_dir.join("python.exe");
    let mut cmd = Command::new(&python_exe);
    cmd.arg(&dest);
    cmd.args(args);
    cmd.current_dir(install_dir);

    formatter.output(
        &format!("[WAIT] running: python {} {}", file_name, args.join(" ")),
        Some(serde_json::Value::Null),
    );

    let output = cmd
        .output()
        .map_err(|e| CoreError::Other(format!("script execution failed: {}", e)))?;

    if output.status.success() {
        formatter.output("[OK] script completed", Some(serde_json::Value::Null));
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CoreError::Other(format!(
            "script execution failed: {}",
            stderr
        )));
    }

    // 清理脚本文件
    let _ = fs::remove_file(&dest);

    Ok(())
}

/// 执行文件修改操作
fn execute_modify_file(rel_path: &str, action: &ModifyAction, install_dir: &Path) -> Result<()> {
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
                        format!(
                            "{}{}",
                            &line[..line.len() - trimmed.len()],
                            after_hash.trim_start()
                        )
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
// Python 3.11.9 recipe（Windows embeddable + get-pip）
// ============================================================

/// Python 3.11.9 安装recipe
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
        // 此值为占位，如果首次downloadverification failed需要更新
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
                // download并运行 get-pip.py 安装 pip（使用官方 PyPI 源引导）
                PostInstallStep::DownloadAndRun {
                    url: "https://bootstrap.pypa.io/get-pip.py".to_string(),
                    args: vec![
                        "--index-url".to_string(),
                        "https://pypi.org/simple".to_string(),
                    ],
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

// ============================================================
// Node.js 20.18.1 recipe（npm 淘宝源自动配置）
// ============================================================

/// Node.js 20 LTS 安装recipe
///
/// 官方二进制分发包，解压后自动写入 .npmrc 使用 npmmirror 镜像。
fn node20_recipe() -> Recipe {
    let (url, sha256, bin_dir) = node20_artifact();
    Recipe {
        name: "node20".to_string(),
        version: "20.18.1".to_string(),
        display_name: "Node.js 20.18.1".to_string(),
        download_url: url.to_string(),
        sha256: sha256.to_string(),
        bin_dir: bin_dir.to_string(),
        env_vars: vec![],
        configs: vec![super::config_gen::npm_mirror_config()],
        post_install: None,
    }
}

/// 当前平台对应的 Node.js 20.18.1 下载信息 (url, sha256, bin_dir)
fn node20_artifact() -> (&'static str, &'static str, &'static str) {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    match (os, arch) {
        ("windows", _) => (
            "https://nodejs.org/dist/v20.18.1/node-v20.18.1-win-x64.zip",
            "56e5aacdeee7168871721b75819ccacf2367de8761b78eaceacdecd41e04ca03",
            "node-v20.18.1-win-x64",
        ),
        ("linux", _) => (
            "https://nodejs.org/dist/v20.18.1/node-v20.18.1-linux-x64.tar.gz",
            "259e5a8bf2e15ecece65bd2a47153262eda71c0b2c9700d5e703ce4951572784",
            "node-v20.18.1-linux-x64",
        ),
        ("macos", "aarch64") => (
            "https://nodejs.org/dist/v20.18.1/node-v20.18.1-darwin-arm64.tar.gz",
            "9e92ce1032455a9cc419fe71e908b27ae477799371b45a0844eedb02279922a4",
            "node-v20.18.1-darwin-arm64",
        ),
        ("macos", _) => (
            "https://nodejs.org/dist/v20.18.1/node-v20.18.1-darwin-x64.tar.gz",
            "c5497dd17c8875b53712edaf99052f961013cedc203964583fc0cfc0aaf93581",
            "node-v20.18.1-darwin-x64",
        ),
        _ => (
            "https://nodejs.org/dist/v20.18.1/node-v20.18.1-linux-x64.tar.gz",
            "259e5a8bf2e15ecece65bd2a47153262eda71c0b2c9700d5e703ce4951572784",
            "node-v20.18.1-linux-x64",
        ),
    }
}

// ============================================================
// Go 1.23.4 recipe
// ============================================================

/// Go 1.23.4 安装recipe
///
/// 官方二进制分发包，二进制位于 go/bin（go 工具链标准布局）。
fn go_recipe() -> Recipe {
    let (url, sha256, _bin_dir) = go_artifact();
    Recipe {
        name: "go".to_string(),
        version: "1.23.4".to_string(),
        display_name: "Go 1.23.4".to_string(),
        download_url: url.to_string(),
        sha256: sha256.to_string(),
        bin_dir: "go/bin".to_string(),
        env_vars: vec![],
        configs: vec![],
        post_install: None,
    }
}

/// 当前平台对应的 Go 1.23.4 下载信息 (url, sha256, bin_dir)
fn go_artifact() -> (&'static str, &'static str, &'static str) {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    match (os, arch) {
        ("windows", _) => (
            "https://dl.google.com/go/go1.23.4.windows-amd64.zip",
            "16c59ac9196b63afb872ce9b47f945b9821a3e1542ec125f16f6085a1c0f3c39",
            "go/bin",
        ),
        ("linux", _) => (
            "https://dl.google.com/go/go1.23.4.linux-amd64.tar.gz",
            "6924efde5de86fe277676e929dc9917d466efa02fb934197bc2eba35d5680971",
            "go/bin",
        ),
        ("macos", "aarch64") => (
            "https://dl.google.com/go/go1.23.4.darwin-arm64.tar.gz",
            "87d2bb0ad4fe24d2a0685a55df321e0efe4296419a9b3de03369dbe60b8acd3a",
            "go/bin",
        ),
        ("macos", _) => (
            "https://dl.google.com/go/go1.23.4.darwin-amd64.tar.gz",
            "6700067389a53a1607d30aa8d6e01d198230397029faa0b109e89bc871ab5a0e",
            "go/bin",
        ),
        _ => (
            "https://dl.google.com/go/go1.23.4.linux-amd64.tar.gz",
            "6924efde5de86fe277676e929dc9917d466efa02fb934197bc2eba35d5680971",
            "go/bin",
        ),
    }
}

// ============================================================
// Java 17 (Temurin) recipe
// ============================================================

/// Temurin JDK 17 安装recipe
///
/// Eclipse Temurin 官方构建，macOS 使用 Contents/Home 布局。
fn java17_recipe() -> Recipe {
    let (url, sha256, bin_dir) = java17_artifact();
    Recipe {
        name: "java17".to_string(),
        version: "17.0.20+8".to_string(),
        display_name: "Temurin JDK 17.0.20".to_string(),
        download_url: url.to_string(),
        sha256: sha256.to_string(),
        bin_dir: bin_dir.to_string(),
        env_vars: vec![],
        configs: vec![],
        post_install: None,
    }
}

/// 当前平台对应的 Temurin JDK 17.0.20+8 下载信息 (url, sha256, bin_dir)
fn java17_artifact() -> (&'static str, &'static str, &'static str) {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    match (os, arch) {
        ("windows", _) => (
            "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.20%2B8/OpenJDK17U-jdk_x64_windows_hotspot_17.0.20_8.zip",
            "418497be5cf585bdd2203d6486a565d66d3f5e992d5630d45104cb873fab8122",
            "jdk-17.0.20+8/bin",
        ),
        ("linux", _) => (
            "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.20%2B8/OpenJDK17U-jdk_x64_linux_hotspot_17.0.20_8.tar.gz",
            "be7668bc030d578b83d6d5ef9221d6d6729bbbca8cf94a7d52e16ac68b5a5a35",
            "jdk-17.0.20+8/bin",
        ),
        ("macos", "aarch64") => (
            "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.20%2B8/OpenJDK17U-jdk_aarch64_mac_hotspot_17.0.20_8.tar.gz",
            "524850138c742324fb21fca4ff6ef68ea25f25bf59366a864e45b4a0c45ed0df",
            "jdk-17.0.20+8/Contents/Home/bin",
        ),
        ("macos", _) => (
            "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.20%2B8/OpenJDK17U-jdk_x64_mac_hotspot_17.0.20_8.tar.gz",
            "3710c3131c5d7c090582b357f1310133a90bf701183d065223f1a0b90b9ed5ae",
            "jdk-17.0.20+8/Contents/Home/bin",
        ),
        _ => (
            "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.20%2B8/OpenJDK17U-jdk_x64_linux_hotspot_17.0.20_8.tar.gz",
            "be7668bc030d578b83d6d5ef9221d6d6729bbbca8cf94a7d52e16ac68b5a5a35",
            "jdk-17.0.20+8/bin",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_python311() {
        let recipe = resolve("python3.11");
        assert!(recipe.is_some());
        let r = recipe.unwrap();
        assert_eq!(r.name, "python3.11");
        assert_eq!(r.version, "3.11.9");
        assert!(!r.download_url.is_empty());
        assert!(!r.sha256.is_empty());
    }

    #[test]
    fn test_resolve_node20() {
        let recipe = resolve("node20");
        assert!(recipe.is_some());
        let r = recipe.unwrap();
        assert_eq!(r.version, "20.18.1");
        assert!(r.download_url.contains("node-v20.18.1"));
        assert_eq!(r.sha256.len(), 64);
        assert!(!r.configs.is_empty()); // .npmrc 镜像配置
    }

    #[test]
    fn test_resolve_go() {
        let recipe = resolve("go");
        assert!(recipe.is_some());
        let r = recipe.unwrap();
        assert_eq!(r.version, "1.23.4");
        assert!(r.download_url.contains("go1.23.4"));
        assert_eq!(r.sha256.len(), 64);
        assert_eq!(r.bin_dir, "go/bin");
    }

    #[test]
    fn test_resolve_java17() {
        let recipe = resolve("java17");
        assert!(recipe.is_some());
        let r = recipe.unwrap();
        assert_eq!(r.version, "17.0.20+8");
        assert!(r.download_url.contains("temurin17"));
        assert_eq!(r.sha256.len(), 64);
    }

    #[test]
    fn test_all_sha256_valid_length() {
        for r in list_recipes() {
            assert_eq!(
                r.sha256.len(),
                64,
                "{} sha256 must be 64 hex chars",
                r.name
            );
        }
    }

    #[test]
    fn test_resolve_unknown() {
        assert!(resolve("nonexistent_package").is_none());
    }

    #[test]
    fn test_list_recipes_not_empty() {
        let recipes = list_recipes();
        assert!(!recipes.is_empty());
        for name in ["python3.11", "node20", "go", "java17"] {
            assert!(recipes.iter().any(|r| r.name == name), "missing {name}");
        }
    }

    #[test]
    fn test_python311_recipe_fields() {
        let recipe = python311_recipe();
        assert_eq!(recipe.bin_dir, ".");
        assert!(recipe.post_install.is_some());
        let post = recipe.post_install.unwrap();
        assert_eq!(post.steps.len(), 2);
    }
}
