use crate::core::{
    community_recipe, ensure_dirs,
    manifest::Manifest,
    preset,
    recipe::{self, resolve},
    registry,
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
                    &format!(
                        "[WARN] 套装 '{}' 暂无可用的配方。({})",
                        preset.display_name, preset.description
                    ),
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
                &format!(
                    "🚀 开始初始化 '{}' ({})...",
                    preset.display_name, preset.description
                ),
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
            &format!("  📦 {} — {} ({})", p.name, p.display_name, p.description),
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
        if let Ok(manifest) = Manifest::open()
            && let Ok(Some(_)) = manifest.get(pkg_name)
        {
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

/// oneinit install <package[@version]> — 安装指定工具
///
/// 支持版本语法：
///   oneinit install python          # 安装默认/最新版
///   oneinit install python@3.11.9   # 安装精确版本
///   oneinit install node@latest     # 安装最新版
pub async fn run_install(formatter: &OutputFormatter, package: &str) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    // 解析 name@version 语法
    let (name, version_spec) = parse_package_spec(package);

    // 递归安装（处理依赖）
    install_recursive(&name, version_spec.as_deref(), formatter, &mut Vec::new()).await;
}

/// 解析 name@version 语法
/// 返回 (name, Option<version>)
/// "python@3.11.9" -> ("python", Some("3.11.9"))
/// "python@latest" -> ("python", Some("latest"))
/// "python" -> ("python", None)
fn parse_package_spec(spec: &str) -> (String, Option<String>) {
    if let Some(idx) = spec.find('@') {
        let name = spec[..idx].to_string();
        let ver = spec[idx + 1..].to_string();
        (name, Some(ver))
    } else {
        (spec.to_string(), None)
    }
}

/// 递归安装（处理依赖）
///
/// installing_stack 防止循环依赖。
/// 使用 BoxFuture 支持 async 递归。
fn install_recursive<'a>(
    name: &'a str,
    version_spec: Option<&'a str>,
    formatter: &'a OutputFormatter,
    installing_stack: &'a mut Vec<String>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + 'a>> {
    Box::pin(async move {
        // 防止循环依赖
        if installing_stack.iter().any(|n| n == name) {
            formatter.output(
                &format!("[WARN] 跳过循环依赖: {}", name),
                Some(serde_json::Value::Null),
            );
            return;
        }
        installing_stack.push(name.to_string());

        // 检查是否已安装
        if let Ok(manifest) = Manifest::open()
            && let Ok(Some(record)) = manifest.get(name)
        {
            formatter.output(
                &format!(
                    "[OK] '{}' 已安装 v{}",
                    name,
                    record.version.as_deref().unwrap_or("?")
                ),
                Some(serde_json::json!({
                    "status": "success", "action": "install",
                    "package": name, "already_installed": true,
                })),
            );
            installing_stack.pop();
            return;
        }

        // 查找配方（内置 -> 本地社区 -> 远程），同时获取依赖信息
        let recipe_info = resolve_recipe_with_deps(name, version_spec, formatter).await;

        match recipe_info {
            RecipeResolution::Builtin(rec) => {
                if let Err(e) = recipe::install(&rec, formatter).await {
                    formatter.error(&e);
                }
            }
            RecipeResolution::Community(rec) => {
                // 先安装依赖
                install_dependencies(&rec, formatter, installing_stack).await;
                if let Err(e) = community_recipe::install(&rec, formatter).await {
                    formatter.error(&e);
                }
            }
            RecipeResolution::NotFound(hint) => {
                formatter.output(
                    &format!("[ERROR] 未找到 '{}' 的安装配方。{}", name, hint),
                    Some(serde_json::json!({
                        "status": "error", "action": "install",
                        "package": name, "message": "Recipe not found",
                    })),
                );
            }
        }

        installing_stack.pop();
    })
}

/// 配方解析结果
enum RecipeResolution {
    Builtin(crate::core::recipe::Recipe),
    Community(Box<crate::core::community_recipe::CommunityRecipe>),
    NotFound(String),
}

/// 三层查找配方（内置 -> 本地 -> 远程），返回配方和依赖信息
async fn resolve_recipe_with_deps(
    name: &str,
    version_spec: Option<&str>,
    formatter: &OutputFormatter,
) -> RecipeResolution {
    // 1. 内置配方（@latest 或无版本时尝试）
    if (version_spec.is_none() || version_spec == Some("latest"))
        && let Some(rec) = resolve(name)
    {
        return RecipeResolution::Builtin(rec);
    }

    // 2. 本地社区配方
    if let Some(rec) = community_recipe::resolve(name) {
        // 如果指定了版本且不匹配，跳过
        if let Some(ver) = version_spec {
            if ver != "latest" && ver != rec.version {
                // 版本不匹配，继续查找远程
            } else {
                return RecipeResolution::Community(Box::new(rec));
            }
        } else {
            return RecipeResolution::Community(Box::new(rec));
        }
    }

    // 3. 远程注册表
    if let Some(entry) = registry::resolve(name) {
        let target_version = match version_spec {
            Some("latest") | None => entry.latest.clone(),
            Some(v) => {
                if entry.versions.contains(&v.to_string()) {
                    v.to_string()
                } else {
                    formatter.output(
                        &format!("[WARN] 版本 {} 不可用，可用: {:?}", v, entry.versions),
                        Some(serde_json::Value::Null),
                    );
                    entry.latest.clone()
                }
            }
        };

        formatter.output(
            &format!("[REMOTE] 获取 {} v{}...", name, target_version),
            Some(serde_json::json!({
                "status": "fetching", "source": "remote",
                "package": name, "version": target_version,
            })),
        );

        match registry::fetch_recipe(name, &target_version).await {
            Ok(recipe) => return RecipeResolution::Community(Box::new(recipe)),
            Err(e) => {
                formatter.output(
                    &format!("[ERROR] 远程获取失败: {}", e),
                    Some(serde_json::Value::Null),
                );
            }
        }
    }

    // 未找到
    let hint = if registry::load_cached_index().is_none() {
        " 提示: 运行 'oneinit update' 获取远程配方索引。".to_string()
    } else {
        String::new()
    };
    RecipeResolution::NotFound(hint)
}

/// 递归安装配方的依赖
async fn install_dependencies(
    recipe: &crate::core::community_recipe::CommunityRecipe,
    formatter: &OutputFormatter,
    installing_stack: &mut Vec<String>,
) {
    if let Some(ref deps) = recipe.depends {
        if deps.is_empty() {
            return;
        }
        formatter.output(
            &format!("[DEPS] 检查依赖: {:?}", deps),
            Some(serde_json::Value::Null),
        );
        for dep in deps {
            install_recursive(dep, None, formatter, installing_stack).await;
        }
    }
}

/// oneinit uninstall <package> -- 卸载指定工具
pub async fn run_uninstall(formatter: &OutputFormatter, package: &str) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    // 先尝试内置配方卸载
    if recipe::uninstall(package, formatter).await.is_err() {
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
        Ok(manifest) => match manifest.list() {
            Ok(records) => {
                let names: Vec<&str> = records.iter().map(|r| r.name.as_str()).collect();
                formatter.output(
                    &if names.is_empty() {
                        "📋 尚未安装任何工具。使用 oneinit install <package> 开始。".to_string()
                    } else {
                        format!(
                            "📋 已安装 {} 个工具:\n{}",
                            names.len(),
                            names
                                .iter()
                                .map(|n| format!("  - {}", n))
                                .collect::<Vec<_>>()
                                .join("\n")
                        )
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
        },
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
            keyword.is_none_or(|kw| {
                r.name.contains(kw) || r.display_name.to_lowercase().contains(&kw.to_lowercase())
            })
        })
        .map(|r| {
            serde_json::json!({
                "name": r.name,
                "version": r.version,
                "display_name": r.display_name,
                "source": "builtin",
            })
        })
        .collect();

    // 社区配方
    let community: Vec<serde_json::Value> = crate::core::community_recipe::load_all()
        .iter()
        .filter(|r| {
            keyword.is_none_or(|kw| {
                r.name.contains(kw) || r.description.to_lowercase().contains(&kw.to_lowercase())
            })
        })
        .map(|r| {
            serde_json::json!({
                "name": r.name,
                "version": r.version,
                "display_name": r.description,
                "source": "community",
            })
        })
        .collect();

    // 远程配方（从缓存 INDEX）
    let remote: Vec<serde_json::Value> = crate::core::registry::list_available()
        .iter()
        .filter(|(name, _, desc)| {
            keyword.is_none_or(|kw| {
                name.contains(kw) || desc.to_lowercase().contains(&kw.to_lowercase())
            })
        })
        .map(|(name, ver, desc)| {
            serde_json::json!({
                "name": name,
                "version": ver,
                "display_name": desc,
                "source": "remote",
            })
        })
        .collect();

    let total = builtin.len() + community.len() + remote.len();

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
            human.push_str(&format!(
                "  - {} v{} [community]\n",
                r["name"], r["version"]
            ));
        }
        for r in &remote {
            human.push_str(&format!("  - {} v{} [remote]\n", r["name"], r["version"]));
        }
    }

    let mut all_results = builtin.clone();
    all_results.extend(community);
    all_results.extend(remote);

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
        &format!(
            "📦 读取 oneinit.yaml: {} 个工具, {} 个镜像, {} 条后置命令",
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
    if let Some(ref commands) = config.post_install
        && !commands.is_empty()
    {
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
                &format!(
                    "\n[{}] 验证完成: {}/{} 项通过",
                    if result.valid { "OK" } else { "FAIL" },
                    passed,
                    total
                ),
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
    if let Err(e) =
        crate::core::migration::run_import(formatter, file, dry_run, force, skip_checksum)
    {
        formatter.error(&e);
    }
}

/// oneinit update -- 更新远程配方索引
pub async fn run_update(formatter: &OutputFormatter) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    use crate::core::registry;

    let config = registry::load_config();
    formatter.output(
        &format!("[UPDATE] 正在从 {} 获取配方索引...", config.registry_url),
        Some(serde_json::json!({
            "status": "fetching",
            "action": "update",
            "registry_url": config.registry_url,
        })),
    );

    match registry::fetch_index().await {
        Ok(index) => {
            let count = index.packages.len();
            formatter.output(
                &format!(
                    "[OK] 索引更新完成: {} 个可用包 (更新于 {})",
                    count, index.last_updated
                ),
                Some(serde_json::json!({
                    "status": "success",
                    "action": "update",
                    "package_count": count,
                    "last_updated": index.last_updated,
                    "packages": index.packages.keys().collect::<Vec<_>>(),
                })),
            );
        }
        Err(e) => {
            formatter.output(
                &format!("[ERROR] 索引更新失败: {}", e),
                Some(serde_json::json!({
                    "status": "error",
                    "action": "update",
                    "error": e.to_string(),
                    "hint": "If 404, the registry repo may not exist yet. Use oneinit publish.",
                })),
            );
        }
    }
}

/// oneinit publish <file> -- 发布配方到远程仓库
pub async fn run_publish(formatter: &OutputFormatter, file: &str) {
    use crate::core::community_recipe;
    use crate::core::registry;

    let path = std::path::PathBuf::from(file);
    if !path.exists() {
        formatter.output(
            &format!("[ERROR] 文件不存在: {}", file),
            Some(serde_json::json!({
                "status": "error", "action": "publish", "file": file,
                "message": "File not found"
            })),
        );
        return;
    }

    // 1. 验证配方
    formatter.output("[PUBLISH] 正在验证配方...", Some(serde_json::Value::Null));
    let verify_result = match community_recipe::verify(&path) {
        Ok(r) => r,
        Err(e) => {
            formatter.error(&e);
            return;
        }
    };
    if !verify_result.valid {
        formatter.output(
            "[ERROR] 配方验证未通过",
            Some(serde_json::json!({
                "status": "error", "action": "publish", "message": "Validation failed"
            })),
        );
        return;
    }

    // 2. 解析配方
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            formatter.error(&e.into());
            return;
        }
    };
    let recipe: community_recipe::CommunityRecipe = match serde_yaml::from_str(&content) {
        Ok(r) => r,
        Err(e) => {
            formatter.output(
                &format!("[ERROR] YAML 解析失败: {}", e),
                Some(serde_json::Value::Null),
            );
            return;
        }
    };

    // 3. 安全提醒
    formatter.output("", Some(serde_json::Value::Null));
    formatter.output(
        "========== [SECURITY] 发布确认 ==========",
        Some(serde_json::Value::Null),
    );
    formatter.output(
        &format!("[SECURITY] 配方名称: {} v{}", recipe.name, recipe.version),
        Some(serde_json::Value::Null),
    );

    let recipe_dir = format!("recipes/{}", recipe.name);
    let recipe_filename = format!("{}.yaml", recipe.version);
    let config = registry::load_config();

    formatter.output(
        "========================================",
        Some(serde_json::Value::Null),
    );

    // 4. 生成发布步骤
    formatter.output("", Some(serde_json::Value::Null));
    formatter.output("[INFO] 完成发布的步骤:", Some(serde_json::Value::Null));
    formatter.output(
        "  1. git clone https://github.com/BG4JTS/oneinit-recipes.git",
        Some(serde_json::Value::Null),
    );
    formatter.output(
        &format!("  2. mkdir -p {}", recipe_dir),
        Some(serde_json::Value::Null),
    );
    formatter.output(
        &format!("  3. cp {} {}/{}", file, recipe_dir, recipe_filename),
        Some(serde_json::Value::Null),
    );
    formatter.output("  4. 更新 INDEX.json", Some(serde_json::Value::Null));
    formatter.output(
        "  5. git add . && git commit && git push",
        Some(serde_json::Value::Null),
    );
    formatter.output("  6. 创建 Pull Request", Some(serde_json::Value::Null));

    formatter.output(
        &format!("\n[PUBLISH] {} v{} 已准备就绪", recipe.name, recipe.version),
        Some(serde_json::json!({
            "status": "ready", "action": "publish",
            "recipe_name": recipe.name, "recipe_version": recipe.version,
            "target_path": format!("{}/{}", recipe_dir, recipe_filename),
            "registry_url": config.registry_url,
        })),
    );
}

/// oneinit doctor -- 环境健康检查
pub async fn run_doctor(formatter: &OutputFormatter) {
    use crate::core::manifest::Manifest;
    use std::path::Path;

    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    let mut checks: Vec<(String, bool, String)> = Vec::new();

    // 1. 数据目录
    let data_dir = crate::core::data_dir();
    let ok = data_dir.exists();
    checks.push((
        "data_dir".to_string(),
        ok,
        if ok {
            data_dir.display().to_string()
        } else {
            "不存在".to_string()
        },
    ));

    // 2. SQLite manifest 可读
    let manifest_ok = Manifest::open().is_ok();
    checks.push((
        "manifest_db".to_string(),
        manifest_ok,
        if manifest_ok {
            "可读".to_string()
        } else {
            "无法打开".to_string()
        },
    ));

    // 3. manifest vs 实际安装目录一致性
    if manifest_ok
        && let Ok(manifest) = Manifest::open()
        && let Ok(records) = manifest.list()
    {
        let mut orphan_paths = 0;
        let mut orphan_path_entries = 0;
        for record in &records {
            let install_path = Path::new(&record.install_path);
            if !install_path.exists() {
                orphan_paths += 1;
            }
            for entry in &record.path_entries {
                if !Path::new(entry).exists() {
                    orphan_path_entries += 1;
                }
            }
        }
        let ok = orphan_paths == 0 && orphan_path_entries == 0;
        let detail = if ok {
            format!("{} 个记录全部一致", records.len())
        } else {
            format!(
                "{} 个安装目录缺失, {} 个 PATH 条目指向不存在的路径",
                orphan_paths, orphan_path_entries
            )
        };
        checks.push(("consistency".to_string(), ok, detail));
    }

    // 4. PATH 中是否有 oneinit 条目（正常情况）
    let path_var = std::env::var("PATH").unwrap_or_default();
    let has_oneinit = path_var.contains(".oneinit");
    checks.push((
        "path_entries".to_string(),
        true, // 信息性检查，不算错误
        if has_oneinit {
            "PATH 中包含 oneinit 管理的条目".to_string()
        } else {
            "PATH 中无 oneinit 条目（正常，如果未安装工具）".to_string()
        },
    ));

    // 5. 磁盘空间
    let envs_dir = crate::core::envs_dir();
    let disk_ok = envs_dir.exists();
    checks.push((
        "envs_dir".to_string(),
        disk_ok,
        envs_dir.display().to_string(),
    ));

    // 6. 缓存索引
    let cache_index = crate::core::registry::load_cached_index();
    let index_ok = cache_index.is_some();
    let index_detail = if index_ok {
        format!("已缓存 ({} 个包)", cache_index.unwrap().packages.len())
    } else {
        "未缓存（运行 oneinit update 获取）".to_string()
    };
    checks.push(("registry_cache".to_string(), true, index_detail));

    // 输出结果
    let total = checks.len();
    let passed = checks.iter().filter(|(_, ok, _)| *ok).count();

    for (name, ok, detail) in &checks {
        let tag = if *ok { "[OK]" } else { "[FAIL]" };
        formatter.output(
            &format!("  {} {} - {}", tag, name, detail),
            Some(serde_json::json!({
                "check": name, "passed": *ok, "detail": detail,
            })),
        );
    }

    let healthy = passed == total;
    formatter.output(
        &format!(
            "\n[{}] {} 项检查: {}/{} 通过",
            if healthy { "OK" } else { "FAIL" },
            if healthy {
                "环境健康"
            } else {
                "发现问题"
            },
            passed,
            total
        ),
        Some(serde_json::json!({
            "status": if healthy { "healthy" } else { "issues" },
            "action": "doctor",
            "total_checks": total,
            "passed": passed,
            "healthy": healthy,
        })),
    );
}

/// oneinit freeze [-o file] -- 从 manifest 导出已安装工具为 oneinit.yaml
pub async fn run_freeze(formatter: &OutputFormatter, output: &str) {
    use crate::core::manifest::Manifest;
    use std::collections::BTreeMap;

    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    let manifest = match Manifest::open() {
        Ok(m) => m,
        Err(e) => {
            formatter.error(&e);
            return;
        }
    };

    let records = match manifest.list() {
        Ok(r) => r,
        Err(e) => {
            formatter.error(&e);
            return;
        }
    };

    if records.is_empty() {
        formatter.output(
            "[INFO] 尚未安装任何工具，无可导出内容",
            Some(serde_json::json!({
                "status": "empty", "action": "freeze", "count": 0,
            })),
        );
        return;
    }

    // 构建 envs map: tool_name -> version
    let mut envs: BTreeMap<String, String> = BTreeMap::new();
    for record in &records {
        let version = record.version.as_deref().unwrap_or("latest");
        // 从 name 中提取工具类型（如 python3.11 -> python, node20 -> node）
        let tool_name = extract_tool_name(&record.name);
        envs.insert(tool_name, version.to_string());
    }

    // 生成 YAML
    let mut yaml = String::new();
    yaml.push_str("# 由 oneinit freeze 生成\n");
    yaml.push_str("# 在新机器上运行 oneinit sync 即可恢复\n\n");

    yaml.push_str("envs:\n");
    for (tool, version) in &envs {
        yaml.push_str(&format!("  {}: {}\n", tool, version));
    }

    // 写入文件
    std::fs::write(output, &yaml).unwrap_or_else(|e| {
        formatter.output(
            &format!("[ERROR] 写入失败: {}", e),
            Some(serde_json::Value::Null),
        );
    });

    formatter.output(
        &format!(
            "[OK] 已导出 {} 个工具到 {}（在新机器上运行 oneinit sync 恢复）",
            records.len(),
            output
        ),
        Some(serde_json::json!({
            "status": "success", "action": "freeze",
            "output": output, "count": records.len(),
            "tools": envs.keys().collect::<Vec<_>>(),
        })),
    );
}

/// 从包名提取工具类型名
/// python3.11 -> python, node20 -> node, rust-stable -> rust
fn extract_tool_name(name: &str) -> String {
    // 找到第一个数字的位置
    let pos = name
        .find(|c: char| c.is_ascii_digit())
        .unwrap_or(name.len());
    name[..pos].trim_end_matches('-').to_string()
}

/// oneinit skill install [--target <agent>] -- 安装 AI Skill
pub async fn run_skill_install(formatter: &OutputFormatter, target: &str) {
    if target == "all" {
        crate::skill_mgr::install_all(formatter);
    } else {
        crate::skill_mgr::install_to(target, formatter);
    }
}

/// oneinit skill status -- 查看 Skill 安装状态
pub async fn run_skill_status(formatter: &OutputFormatter) {
    crate::skill_mgr::status(formatter);
}

/// oneinit skill uninstall -- 卸载 AI Skill
pub async fn run_skill_uninstall(formatter: &OutputFormatter) {
    crate::skill_mgr::uninstall(formatter);
}
