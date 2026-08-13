pub mod cache_db;
pub mod capture;
pub mod checksum;
pub mod community_recipe;
pub mod config_gen;
pub mod doctor;
pub mod downloader;
pub mod dynamic;
pub mod install;
pub mod manifest;
pub mod migration;
pub mod operation;
pub mod path_mgr;
pub mod planner;
pub mod preset;
pub mod recipe;
pub mod registry;
pub mod self_update;
pub mod sync;
pub mod team;
pub mod version;
pub mod viz;

use std::path::PathBuf;

/// 获取 OneInit 数据根目录（默认 ~/.oneinit/）
///
/// 若设置了 `ONEINIT_HOME` 环境变量则直接使用（测试隔离 / 便携部署）。
/// 如果Cannot determine home directory（$HOME 未设置），返回当前目录下的 .oneinit 作为回退。
#[allow(dead_code)]
pub fn data_dir() -> PathBuf {
    // ONEINIT_HOME 覆盖默认数据目录（集成测试用它隔离，避免触碰真实 ~/.oneinit）
    if let Ok(dir) = std::env::var("ONEINIT_HOME") {
        return PathBuf::from(dir);
    }
    let home = dirs::home_dir().unwrap_or_else(|| {
        // 回退：使用当前工作目录
        eprintln!("[WARN] 无法确定主目录，回退到当前目录");
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
    });
    home.join(".oneinit")
}

/// 获取工具安装目录 ~/.oneinit/envs/
#[allow(dead_code)]
pub fn envs_dir() -> PathBuf {
    data_dir().join("envs")
}

/// 获取数据库目录 ~/.oneinit/db/
#[allow(dead_code)]
pub fn db_dir() -> PathBuf {
    data_dir().join("db")
}

/// 获取临时download目录 ~/.oneinit/temp/
#[allow(dead_code)]
pub fn temp_dir() -> PathBuf {
    data_dir().join("temp")
}

/// 获取社区recipe目录 ~/.oneinit/recipes/
#[allow(dead_code)]
pub fn recipes_dir() -> PathBuf {
    data_dir().join("recipes")
}

/// 获取缓存目录 ~/.oneinit/cache/
#[allow(dead_code)]
pub fn cache_dir() -> PathBuf {
    data_dir().join("cache")
}

/// 确保所有必要目录exists
#[allow(dead_code)]
pub fn ensure_dirs() -> Result<()> {
    std::fs::create_dir_all(envs_dir())?;
    std::fs::create_dir_all(db_dir())?;
    std::fs::create_dir_all(temp_dir())?;
    std::fs::create_dir_all(recipes_dir())?;
    std::fs::create_dir_all(cache_dir())?;
    Ok(())
}

/// OneInit 统一错误类型
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum CoreError {
    #[error("Download error: {0}")]
    Download(String),

    #[error("Checksum error: file {file}  SHA256 mismatch (expected: {expected})")]
    Checksum { file: String, expected: String },

    #[error("Extract error: {0}")]
    Extract(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("PATH operation error: {0}")]
    PathOp(String),

    #[error("Config generation error: {0}")]
    ConfigGen(String),

    #[error("Capture error: {0}")]
    Capture(String),

    #[error("Registry error: {0}")]
    Registry(String),

    #[error("数据迁移失败: {0}")]
    Migration(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

impl CoreError {
    /// Actionable suggestion for the error (shown as [HINT] / JSON "suggestion").
    pub fn suggestion(&self) -> Option<String> {
        let msg = self.to_string().to_lowercase();
        match self {
            CoreError::Download(_) => Some(
                "Check your network connection, or set a proxy (HTTP_PROXY/HTTPS_PROXY). If the URL is stale, run `oneinit update` to refresh the registry.".to_string(),
            ),
            CoreError::Checksum { .. } => Some(
                "The downloaded file does not match the expected checksum — it may be corrupted or tampered. Re-run the install to retry.".to_string(),
            ),
            CoreError::Extract(_) => Some(
                "The archive could not be extracted. The download may be corrupt; re-run the install.".to_string(),
            ),
            CoreError::Registry(_) => Some(
                "Registry fetch failed. Run `oneinit update` to retry, or `oneinit registry list` to check your subscriptions.".to_string(),
            ),
            CoreError::Database(_) => Some(
                "The local manifest database is unreadable. Run `oneinit doctor` to diagnose.".to_string(),
            ),
            CoreError::PathOp(_) => Some(
                "PATH update failed. OneInit is designed to run without admin rights — make sure you are not using sudo.".to_string(),
            ),
            CoreError::Other(_) if msg.contains("not found") && msg.contains("recipe") => Some(
                "The recipe was not found. Use `oneinit search <name>` to find alternatives, or `oneinit update` to refresh the remote index.".to_string(),
            ),
            CoreError::Other(_) if msg.contains("not found") && msg.contains("registry") => Some(
                "The registry may be unreachable or the package is missing. Run `oneinit update` and try again.".to_string(),
            ),
            CoreError::Other(_) if msg.contains("permission") || msg.contains("denied") => Some(
                "Permission denied. OneInit installs into your home directory (~/.oneinit) and should not need admin rights.".to_string(),
            ),
            CoreError::Other(_) if msg.contains("signature") || msg.contains("tampered") => Some(
                "Signature verification failed — the content may have been tampered. If you changed your signing key, re-add with --force.".to_string(),
            ),
            CoreError::Other(_) if msg.contains("yaml") || msg.contains("parse failed") => Some(
                "The config file (oneinit.yaml / team.yaml) has a YAML syntax error. Check the file with a YAML linter or editor.".to_string(),
            ),
            _ => None,
        }
    }
}

/// 核心引擎统一 Result 类型
#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, CoreError>;

// 让 CoreError 可以从 reqwest 错误转换
impl From<reqwest::Error> for CoreError {
    fn from(e: reqwest::Error) -> Self {
        CoreError::Download(e.to_string())
    }
}

// 让 CoreError 可以从 rusqlite 错误转换
impl From<rusqlite::Error> for CoreError {
    fn from(e: rusqlite::Error) -> Self {
        CoreError::Database(e.to_string())
    }
}

// 让 CoreError 可以从 zip 错误转换
impl From<zip::result::ZipError> for CoreError {
    fn from(e: zip::result::ZipError) -> Self {
        CoreError::Extract(e.to_string())
    }
}
