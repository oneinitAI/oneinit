//! 安装收尾公共流程 — 写入清单记录
//!
//! 内置配方（`recipe::install`）与社区配方（`community_recipe::install`）
//! 共用此收尾步骤：执行完操作计划后，把安装结果写入 SQLite 清单。

use std::path::Path;

use super::manifest::{InstallRecord, Manifest};
use super::Result;
use crate::output::OutputFormatter;

/// 安装失败回滚：恢复 PATH 备份并清理本次创建的安装目录。
///
/// 安装流程在操作执行前会清空并重建安装目录，失败时其中的内容均为本次
/// 安装的半成品，直接删除可让环境回到"未安装"状态（PATH 已恢复、清单未写）。
/// 调试时可用 `--no-rollback` 跳过。
pub fn rollback_install(
    formatter: &OutputFormatter,
    install_dir: &Path,
    path_backup: &str,
) {
    match super::path_mgr::restore(path_backup) {
        Ok(()) => formatter.output(
            "[ROLLBACK] 已恢复 PATH",
            Some(serde_json::json!({ "status": "rolled_back", "action": "rollback", "step": "path" })),
        ),
        Err(e) => formatter.output(
            &format!("[WARN] 回滚 PATH 失败: {}", e),
            Some(serde_json::Value::Null),
        ),
    }
    if install_dir.exists() {
        match std::fs::remove_dir_all(install_dir) {
            Ok(()) => formatter.output(
                &format!(
                    "[ROLLBACK] 已清理安装目录: {}",
                    install_dir.display()
                ),
                Some(serde_json::json!({ "status": "rolled_back", "action": "rollback", "step": "dir" })),
            ),
            Err(e) => formatter.output(
                &format!("[WARN] 清理安装目录失败: {}", e),
                Some(serde_json::Value::Null),
            ),
        }
    }
}

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

/// 从配方名推断可执行文件名（去 @version / 数字后缀；Windows 加 .exe）
///
/// 例：`python3.11` → `python(.exe)`；`node@20` → `node(.exe)`；`go` → `go(.exe)`。
pub fn exe_name(recipe_name: &str) -> String {
    let base = recipe_name
        .split('@')
        .next()
        .unwrap_or(recipe_name)
        .trim_end_matches(|c: char| c.is_ascii_digit());
    if cfg!(target_os = "windows") {
        format!("{}.exe", base)
    } else {
        base.to_string()
    }
}

/// 安装后二进制可用性验证：在 PATH 条目（bin 目录）中查找可执行文件并探测 `--version`。
///
/// 返回版本输出首行；找不到可执行文件或探测失败返回 `None`（不阻断安装，仅提示）。
pub fn verify_installed_binary(path_entries: &[String], recipe_name: &str) -> Option<String> {
    use crate::core::capture::detector::{run_command, run_command_with_stderr};

    let exe = exe_name(recipe_name);
    for entry in path_entries {
        let candidate = Path::new(entry).join(&exe);
        if !candidate.exists() {
            continue;
        }
        let probe = candidate.to_string_lossy().to_string();
        let output = run_command_with_stderr(&probe, &["--version"])
            .or_else(|| run_command(&probe, &["--version"]));
        if let Some(ver) = output {
            return ver.lines().next().map(|s| s.to_string());
        }
    }
    None
}
