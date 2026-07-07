use crate::core::{
    ensure_dirs, manifest::Manifest,
};
use crate::output::OutputFormatter;

/// oneinit init — 一键初始化开发环境（Phase 2 完整实现）
pub async fn run_init(formatter: &OutputFormatter, _preset: Option<&str>) {
    formatter.output(
        "⏳ init 命令将在 Phase 2 实现。届时将根据套装一键初始化整台电脑的开发环境。",
        Some(serde_json::json!({
            "status": "success",
            "action": "init",
            "message": "Phase 2: not yet implemented"
        })),
    );
}

/// oneinit install <package> — 安装指定工具
pub async fn run_install(formatter: &OutputFormatter, package: &str) {
    // 安装前确保目录存在
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    // 检查是否已安装
    match Manifest::open() {
        Ok(manifest) => {
            if let Ok(Some(record)) = manifest.get(package) {
                formatter.output(
                    &format!("📦 '{}' 已安装 ({})", package, record.install_path),
                    Some(serde_json::json!({
                        "status": "success",
                        "action": "install",
                        "package": package,
                        "already_installed": true,
                        "install_path": record.install_path,
                        "message": "Package already installed"
                    })),
                );
                return;
            }
        }
        Err(e) => {
            formatter.error(&e);
            return;
        }
    }

    // 核心安装流程将在配方系统（Task 6）中实现
    // 当前仅验证核心引擎基础设施是否正常工作
    formatter.output(
        &format!("⏳ install '{}' 的核心引擎已就绪，等待配方系统完成后实现完整安装流程。", package),
        Some(serde_json::json!({
            "status": "success",
            "action": "install",
            "package": package,
            "installed": false,
            "message": "Core engine ready, waiting for recipe system"
        })),
    );
}

/// oneinit uninstall <package> — 卸载指定工具
pub async fn run_uninstall(formatter: &OutputFormatter, package: &str) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    match Manifest::open() {
        Ok(manifest) => {
            match manifest.remove(package) {
                Ok(Some(record)) => {
                    formatter.output(
                        &format!("🗑️ 已移除 '{}' 的安装记录（实际文件未删除，配方系统完成后实现完整卸载）", package),
                        Some(serde_json::json!({
                            "status": "success",
                            "action": "uninstall",
                            "package": package,
                            "removed_record": record,
                            "message": "Record removed, full rollback pending"
                        })),
                    );
                }
                Ok(None) => {
                    formatter.output(
                        &format!("📦 '{}' 未安装，无需卸载。", package),
                        Some(serde_json::json!({
                            "status": "success",
                            "action": "uninstall",
                            "package": package,
                            "not_installed": true,
                            "message": "Package not installed"
                        })),
                    );
                }
                Err(e) => {
                    formatter.error(&e);
                }
            }
        }
        Err(e) => {
            formatter.error(&e);
        }
    }
}

/// oneinit list — 列出已安装的工具
pub async fn run_list(formatter: &OutputFormatter) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    match Manifest::open() {
        Ok(manifest) => {
            match manifest.list() {
                Ok(records) => {
                    let names: Vec<&str> = records.iter().map(|r| r.name.as_str()).collect();
                    formatter.output(
                        &if names.is_empty() {
                            "📋 尚未安装任何工具。使用 oneinit install <package> 开始。".to_string()
                        } else {
                            format!("📋 已安装 {} 个工具:\n{}", names.len(), names.iter().map(|n| format!("  - {}", n)).collect::<Vec<_>>().join("\n"))
                        },
                        Some(serde_json::json!({
                            "status": "success",
                            "action": "list",
                            "installed": records,
                            "count": records.len()
                        })),
                    );
                }
                Err(e) => {
                    formatter.error(&e);
                }
            }
        }
        Err(e) => {
            formatter.error(&e);
        }
    }
}

/// oneinit search <keyword> — 搜索可用工具
pub async fn run_search(formatter: &OutputFormatter, keyword: Option<&str>) {
    let msg = match keyword {
        Some(kw) => format!("⏳ search '{}' 将在配方系统完成后实现。", kw),
        None => "⏳ search 将在配方系统完成后实现。".to_string(),
    };
    formatter.output(
        &msg,
        Some(serde_json::json!({
            "status": "success",
            "action": "search",
            "keyword": keyword,
            "results": [],
            "message": "Not yet implemented"
        })),
    );
}

/// oneinit sync — 从 oneinit.yaml 同步环境
pub async fn run_sync(formatter: &OutputFormatter) {
    formatter.output(
        "⏳ sync 将在核心引擎完成后实现。届时将读取 oneinit.yaml 批量同步环境。",
        Some(serde_json::json!({
            "status": "success",
            "action": "sync",
            "message": "Not yet implemented"
        })),
    );
}
