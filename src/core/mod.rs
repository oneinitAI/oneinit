pub mod capture;
pub mod community_recipe;
pub mod config_gen;
pub mod downloader;
pub mod manifest;
pub mod migration;
pub mod path_mgr;
pub mod preset;
pub mod recipe;
pub mod registry;
pub mod sync;

use std::path::PathBuf;

/// 获取 OneInit 数据根目录 ~/.oneinit/
///
/// 如果Cannot determine home directory（$HOME 未设置），返回当前目录下的 .oneinit 作为回退。
#[allow(dead_code)]
pub fn data_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| {
        // 回退：使用当前工作目录
        eprintln!("[WARN] Cannot determine home directory，falling back to current dir");
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
