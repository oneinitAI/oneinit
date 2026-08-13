//! 安装收尾公共流程 — 写入清单记录
//!
//! 内置配方（`recipe::install`）与社区配方（`community_recipe::install`）
//! 共用此收尾步骤：执行完操作计划后，把安装结果写入 SQLite 清单。

use std::path::Path;

use super::manifest::{InstallRecord, Manifest};
use super::Result;

/// 构建并写入安装清单记录，返回记录 ID。
///
/// `path_entries` 为写入 PATH 的条目；`config_files` 为生成/修改的配置文件
/// 绝对路径列表；`path_backup` 为安装前备份的 PATH（用于卸载回滚）。
pub fn add_manifest_record(
    name: &str,
    version: Option<String>,
    install_path: &Path,
    archive_url: Option<String>,
    sha256: Option<String>,
    path_entries: Vec<String>,
    config_files: Vec<String>,
    path_backup: String,
) -> Result<String> {
    let manifest = Manifest::open()?;
    let record = InstallRecord {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.to_string(),
        version,
        install_path: install_path.to_string_lossy().to_string(),
        archive_url,
        sha256,
        path_entries,
        config_files,
        installed_at: chrono::Utc::now().to_rfc3339(),
        original_path: Some(path_backup),
        env_vars_backup: serde_json::json!({}),
    };
    let record_id = manifest.add(&record)?;
    Ok(record_id)
}
