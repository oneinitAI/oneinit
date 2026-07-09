use crate::core::{
    ensure_dirs, manifest::Manifest, preset, recipe::{self, resolve},
    sync::{self, SyncConfig},
};
use crate::output::OutputFormatter;

/// oneinit init — 一键初始化开发环境
pub async fn run_init(formatter: &OutputFormatter, preset_name: Option<&str>) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    match preset_name {
        Some(name) => {
            // 指定了套装名，执行批量安装
            let preset = match preset::resolve(name) {
                Some(p) => p,
                None => {
                    formatter.output(
                        &format!("[ERROR] 未找到套装 '{}'。可用套装：", name),
                        Some(serde_json::json!({
                            "status": "error",
                            "action": "init",
                            "preset": name,
                            "message": "Preset not found",
                            "available": preset::list_presets().iter().map(|p| p.name.clone()).collect::<Vec<_>>()
                        })),
                    );
                    list_available_presets(formatter);
                    return;
                }
            };

            if preset.packages.is_empty() {
                formatter.output(
                    &format!("[WARN] 套装 '{}' 暂无可用的配方。({})", preset.display_name, preset.description),
                    Some(serde_json::json!({
                        "status": "success",
                        "action": "init",
                        "preset": preset.name,
                        "installed": 0,
                        "message": "No available packages in preset"
                    })),
                );
                return;
            }

            formatter.output(
                &format!("🚀 开始初始化 '{}' ({})...", preset.display_name, preset.description),
                Some(serde_json::json!({
                    "status": "success",
                    "action": "init",
                    "preset": preset.name,
                    "display_name": preset.display_name,
                    "packages": preset.packages,
                })),
            );

            // 批量安装
            batch_install(&preset.packages, formatter).await;
        }
        None => {
            // 未指定套装，列出可用套装
            formatter.output(
                "📋 未指定套装。可用的预置套装：",
                Some(serde_json::json!({
                    "status": "success",
                    "action": "init",
                    "message": "No preset specified, listing available"
                })),
            );
            list_available_presets(formatter);
        }
    }
}

/// 列出所有可用套装
fn list_available_presets(formatter: &OutputFormatter) {
    let presets = preset::list_presets();
    for p in &presets {
        formatter.output(
            &format!(
                "  📦 {} — {} ({})",
                p.name, p.display_name, p.description
            ),
            Some(serde_json::json!({
                "name": p.name,
                "display_name": p.display_name,
                "description": p.description,
                "package_count": p.packages.len(),
            })),
        );
    }
    formatter.output(
        "\n使用 oneinit init --preset <名称> 开始初始化。",
        Some(serde_json::json!({
            "usage": "oneinit init --preset <name>"
        })),
    );
}

/// 批量安装配方列表
async fn batch_install(packages: &[String], formatter: &OutputFormatter) {
    let mut succeeded: Vec<&str> = Vec::new();
    let mut skipped: Vec<&str> = Vec::new();
    let mut failed: Vec<(&str, String)> = Vec::new();

    for pkg_name in packages {
        // 检查是否已安装
        if let Ok(manifest) = Manifest::open() {
            if let Ok(Some(_)) = manifest.get(pkg_name) {
                formatter.output(
                    &format!("  [SKIP] '{}' 已安装，跳过", pkg_name),
                    Some(serde_json::json!({
                        "package": pkg_name,
                        "status": "skipped",
                        "reason": "already_installed"
                    })),
                );
                skipped.push(pkg_name);
                continue;
            }
        }

        // 查找配方
        let recipe = match resolve(pkg_name) {
            Some(r) => r,
            None => {
                formatter.output(
                    &format!("  [ERROR] '{}' 未找到配方，跳过", pkg_name),
                    Some(serde_json::json!({
                        "package": pkg_name,
                        "status": "failed",
                        "reason": "recipe_not_found"
                    })),
                );
                failed.push((pkg_name, "recipe_not_found".to_string()));
                continue;
            }
        };

        // 安装
        match recipe::install(&recipe, formatter).await {
            Ok(()) => succeeded.push(pkg_name),
            Err(e) => {
                formatter.output(
                    &format!("  [ERROR] '{}' 安装失败: {}", pkg_name, e),
                    Some(serde_json::json!({
                        "package": pkg_name,
                        "status": "failed",
                        "error": e.to_string()
                    })),
                );
                failed.push((pkg_name, e.to_string()));
            }
        }
    }

    // 输出总结
    formatter.output(
        &format!(
            "\n📊 初始化完成: {} 成功, {} 跳过, {} 失败",
            succeeded.len(),
            skipped.len(),
            failed.len()
        ),
        Some(serde_json::json!({
            "status": "complete",
            "succeeded": succeeded,
            "skipped": skipped,
            "failed": failed.iter().map(|(n, e)| serde_json::json!({"package": n, "error": e})).collect::<Vec<_>>(),
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

    // 查找内置配方
    if let Some(rec) = resolve(package) {
        if let Err(e) = recipe::install(&rec, formatter).await {
            formatter.error(&e);
        }
        return;
    }

    // 未找到内置配方，尝试社区配方
    use crate::core::community_recipe;
    if let Some(rec) = community_recipe::resolve(package) {
        if let Err(e) = community_recipe::install(&rec, formatter).await {
            formatter.error(&e);
        }
        return;
    }

    // 都没找到
    formatter.output(
        &format!("[ERROR] 未找到 '{}' 的安装配方。使用 oneinit search 查看可用工具。", package),
        Some(serde_json::json!({
            "status": "error",
            "action": "install",
            "package": package,
            "message": "Recipe not found"
        })),
    );
}

/// oneinit uninstall <package> -- 卸载指定工具
pub async fn run_uninstall(formatter: &OutputFormatter, package: &str) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    // 先尝试内置配方卸载
    if let Err(e) = recipe::uninstall(package, formatter).await {
        // 内置卸载失败，尝试社区配方卸载
        use crate::core::community_recipe;
        if let Err(e2) = community_recipe::uninstall(package, formatter).await {
            formatter.error(&e2);
        }
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

/// oneinit search <keyword> -- 搜索可用工具
pub async fn run_search(formatter: &OutputFormatter, keyword: Option<&str>) {
    // 内置配方
    let builtin: Vec<serde_json::Value> = recipe::list_recipes()
        .iter()
        .filter(|r| {
            keyword.map_or(true, |kw| {
                r.name.contains(kw)
                    || r.display_name.to_lowercase().contains(&kw.to_lowercase())
            })
        })
        .map(|r| serde_json::json!({
            "name": r.name,
            "version": r.version,
            "display_name": r.display_name,
            "source": "builtin",
        }))
        .collect();

    // 社区配方
    let community: Vec<serde_json::Value> = crate::core::community_recipe::load_all()
        .iter()
        .filter(|r| {
            keyword.map_or(true, |kw| {
                r.name.contains(kw) || r.description.to_lowercase().contains(&kw.to_lowercase())
            })
        })
        .map(|r| serde_json::json!({
            "name": r.name,
            "version": r.version,
            "display_name": r.description,
            "source": "community",
        }))
        .collect();

    let total = builtin.len() + community.len();

    let mut human = String::new();
    if total == 0 {
        human.push_str(&match keyword {
            Some(kw) => format!("[SEARCH] 未找到与 '{}' 相关的工具。", kw),
            None => "[SEARCH] 暂无可用工具。".to_string(),
        });
    } else {
        human.push_str(&format!("[SEARCH] 找到 {} 个可用工具:\n", total));
        for r in &builtin {
            human.push_str(&format!("  - {} v{} [builtin]\n", r["name"], r["version"]));
        }
        for r in &community {
            human.push_str(&format!("  - {} v{} [community]\n", r["name"], r["version"]));
        }
    }

    let mut all_results = builtin.clone();
    all_results.extend(community);

    formatter.output(
        human.trim(),
        Some(serde_json::json!({
            "status": "success",
            "action": "search",
            "keyword": keyword,
            "results": all_results,
            "count": total,
        })),
    );
}

/// oneinit sync — 从 oneinit.yaml 同步环境
pub async fn run_sync(formatter: &OutputFormatter) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    // 1. 查找 oneinit.yaml
    let yaml_path = std::path::PathBuf::from("oneinit.yaml");
    if !yaml_path.exists() {
        formatter.output(
            "[ERROR] 当前目录未找到 oneinit.yaml 文件。",
            Some(serde_json::json!({
                "status": "error",
                "action": "sync",
                "message": "oneinit.yaml not found in current directory"
            })),
        );
        return;
    }

    // 2. 解析配置
    let config: SyncConfig = match sync::load_config(&yaml_path) {
        Ok(c) => c,
        Err(e) => {
            formatter.error(&e);
            return;
        }
    };

    formatter.output(
        &format!("📦 读取 oneinit.yaml: {} 个工具, {} 个镜像, {} 条后置命令",
            config.envs.len(),
            config.mirrors.as_ref().map_or(0, |m| m.len()),
            config.post_install.as_ref().map_or(0, |c| c.len()),
        ),
        Some(serde_json::json!({
            "status": "success",
            "action": "sync",
            "envs_count": config.envs.len(),
            "mirrors": config.mirrors,
            "post_install_count": config.post_install.as_ref().map_or(0, |c| c.len()),
        })),
    );

    // 3. 批量安装 envs
    let recipe_names = sync::envs_to_recipe_names(&config);
    batch_install(&recipe_names, formatter).await;

    // 4. 应用镜像配置（记录日志，未来扩展覆盖默认镜像）
    if let Some(ref mirrors) = config.mirrors {
        formatter.output(
            &format!("[CONF] 镜像配置: {:?}", mirrors),
            Some(serde_json::json!({
                "mirrors_applied": mirrors,
            })),
        );
    }

    // 5. 执行 post_install 命令
    if let Some(ref commands) = config.post_install {
        if !commands.is_empty() {
            formatter.output(
                "[RUN] 开始执行安装后命令...",
                Some(serde_json::json!({
                    "phase": "post_install",
                    "command_count": commands.len(),
                })),
            );
            if let Err(e) = sync::run_post_install(commands, formatter) {
                formatter.error(&e);
                return;
            }
        }
    }

    // 6. 同步完成
    formatter.output(
        "[SUCCESS] 环境同步完成!",
        Some(serde_json::json!({
            "status": "complete",
            "action": "sync",
            "message": "Environment synchronized successfully"
        })),
    );
}

/// oneinit verify <file> -- 验证社区配方文件
pub async fn run_verify(formatter: &OutputFormatter, file: &str) {
    use crate::core::community_recipe;

    let path = std::path::PathBuf::from(file);
    if !path.exists() {
        formatter.output(
            &format!("[ERROR] 文件不存在: {}", file),
            Some(serde_json::json!({
                "status": "error",
                "action": "verify",
                "file": file,
                "message": "File not found"
            })),
        );
        return;
    }

    match community_recipe::verify(&path) {
        Ok(result) => {
            let total = result.checks.len();
            let passed = result.checks.iter().filter(|(_, ok, _)| *ok).count();

            for (name, ok, detail) in &result.checks {
                let tag = if detail.starts_with("[WARN]") {
                    "[WARN]"
                } else if *ok {
                    "[OK]"
                } else {
                    "[FAIL]"
                };
                formatter.output(
                    &format!("  {} {} - {}", tag, name, detail),
                    Some(serde_json::json!({
                        "check": name,
                        "passed": *ok,
                        "detail": detail,
                    })),
                );
            }

            formatter.output(
                &format!("\n[{}] 验证完成: {}/{} 项通过", if result.valid { "OK" } else { "FAIL" }, passed, total),
                Some(serde_json::json!({
                    "status": if result.valid { "valid" } else { "invalid" },
                    "action": "verify",
                    "file": file,
                    "total_checks": total,
                    "passed": passed,
                    "valid": result.valid,
                })),
            );
        }
        Err(e) => {
            formatter.error(&e);
        }
    }
}

/// oneinit capture [--output <file>] -- 捕获当前开发环境
pub async fn run_capture(formatter: &OutputFormatter, output: &str) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    if let Err(e) = crate::core::capture::run_capture(formatter, output) {
        formatter.error(&e);
    }
}

/// oneinit export [--output <file>] [--include-envs] -- 导出环境为 tar.gz
pub async fn run_export(formatter: &OutputFormatter, output: &str, include_envs: bool) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    if let Err(e) = crate::core::migration::run_export(formatter, output, include_envs) {
        formatter.error(&e);
    }
}

/// oneinit import <file> [--dry-run] [--force] -- 从 tar.gz 导入环境
pub async fn run_import(formatter: &OutputFormatter, file: &str, dry_run: bool, force: bool) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    let skip_checksum = false;
    if let Err(e) = crate::core::migration::run_import(formatter, file, dry_run, force, skip_checksum) {
        formatter.error(&e);
    }
}
