use crate::core::{
    ensure_dirs, manifest::Manifest, recipe::{self, resolve},
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
    if let Ok(manifest) = Manifest::open() {
        if let Ok(Some(record)) = manifest.get(package) {
            formatter.output(
                &format!("📦 '{}' 已安装 ({})", package, record.install_path),
                Some(serde_json::json!({
                    "status": "success",
                    "action": "install",
                    "package": package,
                    "already_installed": true,
                    "install_path": record.install_path,
                })),
            );
            return;
        }
    }

    // 查找配方
    let rec = match resolve(package) {
        Some(r) => r,
        None => {
            formatter.output(
                &format!("❌ 未找到 '{}' 的安装配方。使用 oneinit search 查看可用工具。", package),
                Some(serde_json::json!({
                    "status": "error",
                    "action": "install",
                    "package": package,
                    "message": "Recipe not found"
                })),
            );
            return;
        }
    };

    // 执行安装
    if let Err(e) = recipe::install(&rec, formatter).await {
        formatter.error(&e);
    }
}

/// oneinit uninstall <package> — 卸载指定工具
pub async fn run_uninstall(formatter: &OutputFormatter, package: &str) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    if let Err(e) = recipe::uninstall(package, formatter).await {
        formatter.error(&e);
    }
}

/// oneinit list — 列出已安装的工具
pub async fn run_list(formatter: &OutputFormatter) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    use crate::core::manifest::Manifest;

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
    let all = recipe::list_recipes();

    let results: Vec<_> = all
        .iter()
        .filter(|r| {
            keyword.map_or(true, |kw| {
                r.name.contains(kw)
                    || r.display_name.to_lowercase().contains(&kw.to_lowercase())
            })
        })
        .collect();

    formatter.output(
        &if results.is_empty() {
            match keyword {
                Some(kw) => format!("🔍 未找到与 '{}' 相关的工具。", kw),
                None => "🔍 暂无可用工具。".to_string(),
            }
        } else {
            format!(
                "🔍 找到 {} 个可用工具:\n{}",
                results.len(),
                results
                    .iter()
                    .map(|r| format!("  - {} ({})", r.name, r.display_name))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        },
        Some(serde_json::json!({
            "status": "success",
            "action": "search",
            "keyword": keyword,
            "results": results.iter().map(|r| serde_json::json!({
                "name": r.name,
                "version": r.version,
                "display_name": r.display_name,
            })).collect::<Vec<_>>(),
            "count": results.len()
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
