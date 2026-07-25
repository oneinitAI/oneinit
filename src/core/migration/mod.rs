// 数据迁移模块 — export / import
//
// 按 数据的采集与迁移.md 第四章实现。
// 导出：扫描环境 -> 生成 oneinit.yaml -> 可选打包 envs/ -> tar.gz
// 导入：解压 tar.gz -> verify SHA256 -> 恢复recipe/环境 -> 安装全局包

pub mod manifest;
pub mod packer;
pub mod unpacker;

use serde::{Deserialize, Serialize};

use super::Result;
use crate::output::OutputFormatter;

/// 导出结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    /// 输出文件路径
    pub path: String,
    /// 包总大小（字节）
    pub total_size: u64,
    /// 检测到的环境数量
    pub env_count: usize,
    /// 包含的缓存文件数量
    pub cache_count: usize,
}

/// 导入结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    /// 恢复的recipe路径
    pub recipe_path: String,
    /// 恢复的缓存文件数
    pub cache_restored: usize,
    /// 恢复的全局包数
    pub packages_restored: usize,
    /// 是否为 dry_run（只预览未实际执行）
    pub dry_run: bool,
}

/// 执行导出
///
/// 流程：扫描环境 -> 序列化 YAML -> 可选打包 envs/ -> 生成 manifest.json -> tar.gz
pub fn run_export(
    formatter: &OutputFormatter,
    output: &str,
    include_envs: bool,
) -> Result<ExportResult> {
    formatter.output(
        "[EXPORT] 开始导出环境...",
        Some(serde_json::json!({
            "status": "exporting",
            "action": "export",
            "output": output,
            "include_envs": include_envs,
        })),
    );

    let result = packer::export(formatter, output, include_envs)?;

    formatter.output(
        &format!(
            "[OK] 导出完成: {} ({:.1} KB, {} 个环境, {} 个缓存文件)",
            result.path,
            result.total_size as f64 / 1024.0,
            result.env_count,
            result.cache_count,
        ),
        Some(serde_json::json!({
            "status": "success",
            "action": "export",
            "path": result.path,
            "total_size": result.total_size,
            "env_count": result.env_count,
            "cache_count": result.cache_count,
        })),
    );

    Ok(result)
}

/// 执行导入
///
/// 流程：解压 -> 解析 manifest -> verify SHA256 -> 恢复recipe/环境
pub fn run_import(
    formatter: &OutputFormatter,
    archive: &str,
    dry_run: bool,
    force: bool,
    skip_checksum: bool,
) -> Result<ImportResult> {
    // 安全提醒
    if !dry_run {
        formatter.output("", Some(serde_json::Value::Null));
        formatter.output(
            "[SECURITY] 即将导入环境备份，以下操作将被执行:",
            Some(serde_json::Value::Null),
        );
        formatter.output(
            &format!("[SECURITY]   file: {}", archive),
            Some(serde_json::Value::Null),
        );
        formatter.output(
            "[SECURITY]   操作: 恢复recipe文件、恢复工具目录、恢复全局包列表",
            Some(serde_json::Value::Null),
        );
        formatter.output(
            if force {
                "[SECURITY]   模式: --force 已启用，将覆盖现有文件"
            } else {
                "[SECURITY]   模式: 已exists文件将被跳过（使用 --force 覆盖）"
            },
            Some(serde_json::Value::Null),
        );
        formatter.output(
            "[SECURITY] 建议先使用 --dry-run 预览导入内容",
            Some(serde_json::Value::Null),
        );
    }

    formatter.output(
        &format!("[IMPORT] importing: {} (dry_run={})", archive, dry_run),
        Some(serde_json::json!({
            "status": "importing",
            "action": "import",
            "file": archive,
            "dry_run": dry_run,
            "force": force,
            "skip_checksum": skip_checksum,
        })),
    );

    let result = unpacker::import(formatter, archive, dry_run, force, skip_checksum)?;

    if dry_run {
        formatter.output(
            &format!(
                "[OK] 预览完成: 将恢复 {} 个缓存, {} 个包 (dry run)",
                result.cache_restored, result.packages_restored,
            ),
            Some(serde_json::json!({
                "status": "success",
                "action": "import",
                "dry_run": true,
                "cache_restored": result.cache_restored,
                "packages_restored": result.packages_restored,
            })),
        );
    } else {
        formatter.output(
            &format!(
                "[OK] 导入完成: {} 个缓存恢复, {} 个包恢复",
                result.cache_restored, result.packages_restored,
            ),
            Some(serde_json::json!({
                "status": "success",
                "action": "import",
                "dry_run": false,
                "recipe_path": result.recipe_path,
                "cache_restored": result.cache_restored,
                "packages_restored": result.packages_restored,
            })),
        );
    }

    Ok(result)
}
