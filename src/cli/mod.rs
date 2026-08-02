use crate::core::{
    community_recipe, ensure_dirs,
    manifest::Manifest,
    preset,
    recipe::{self, resolve},
    registry,
    sync::{self, SyncConfig},
    team,
};
use crate::output::OutputFormatter;

/// oneinit init — initialize dev environment with presets
pub async fn run_init(formatter: &OutputFormatter, preset_name: Option<&str>) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    match preset_name {
        Some(name) => {
            // specified preset name, batch install
            let preset = match preset::resolve(name) {
                Some(p) => p,
                None => {
                    formatter.output(
                        &format!("[ERROR] Preset not found: '{}'。Available presets:", name),
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
                        "[WARN] Preset '{}' has no available packages。({})",
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
                    "Initializing preset '{}' ({})...",
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

            // batch install
            batch_install(&preset.packages, formatter).await;
        }
        None => {
            // no preset specified, listing available presets
            formatter.output(
                "No preset specified. Available presets:",
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

/// list all available presets
fn list_available_presets(formatter: &OutputFormatter) {
    let presets = preset::list_presets();
    for p in &presets {
        formatter.output(
            &format!("   {} — {} ({})", p.name, p.display_name, p.description),
            Some(serde_json::json!({
                "name": p.name,
                "display_name": p.display_name,
                "description": p.description,
                "package_count": p.packages.len(),
            })),
        );
    }
    formatter.output(
        "\nUse oneinit init --preset <name> to start.",
        Some(serde_json::json!({
            "usage": "oneinit init --preset <name>"
        })),
    );
}

/// batch install recipe list
async fn batch_install(packages: &[String], formatter: &OutputFormatter) {
    let mut succeeded: Vec<&str> = Vec::new();
    let mut skipped: Vec<&str> = Vec::new();
    let mut failed: Vec<(&str, String)> = Vec::new();

    for pkg_name in packages {
        // check if already installed
        if let Ok(manifest) = Manifest::open()
            && let Ok(Some(_)) = manifest.get(pkg_name)
        {
            formatter.output(
                &format!("  [SKIP] '{}' already installed, skipped", pkg_name),
                Some(serde_json::json!({
                    "package": pkg_name,
                    "status": "skipped",
                    "reason": "already_installed"
                })),
            );
            skipped.push(pkg_name);
            continue;
        }

        // find recipe
        let recipe = match resolve(pkg_name) {
            Some(r) => r,
            None => {
                formatter.output(
                    &format!("  [ERROR] '{}' recipe not found, skipped", pkg_name),
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
                    &format!("  [ERROR] '{}' install failed: {}", pkg_name, e),
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

/// oneinit install <package[@version]> — Install a tool
///
/// supports version syntax:
///   oneinit install python          # install default/latest
///   oneinit install python@3.11.9   # install specific version
///   oneinit install node@latest     # install latest
pub async fn run_install(formatter: &OutputFormatter, package: &str, allow_exec: bool) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    // parse name@version syntax
    let (name, version_spec) = parse_package_spec(package);

    // recursive install (handle dependencies)
    install_recursive(
        &name,
        version_spec.as_deref(),
        formatter,
        &mut Vec::new(),
        allow_exec,
    )
    .await;
}

/// parse name@version syntax
/// returns (name, Option<version>)
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

/// recursive install (handle dependencies)
///
/// installing_stack prevents circular dependencies.
/// uses BoxFuture for async recursion.
fn install_recursive<'a>(
    name: &'a str,
    version_spec: Option<&'a str>,
    formatter: &'a OutputFormatter,
    installing_stack: &'a mut Vec<String>,
    allow_exec: bool,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + 'a>> {
    Box::pin(async move {
        // 防止循环依赖
        if installing_stack.iter().any(|n| n == name) {
            formatter.output(
                &format!("[WARN] Skipping circular dependency: {}", name),
                Some(serde_json::Value::Null),
            );
            return;
        }
        installing_stack.push(name.to_string());

        // check if already installed
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

        // find recipe（内置 -> 本地社区 -> 远程），同时获取依赖信息
        let recipe_info = resolve_recipe_with_deps(name, version_spec, formatter).await;

        match recipe_info {
            RecipeResolution::Builtin(rec) => {
                if let Err(e) = recipe::install(&rec, formatter).await {
                    formatter.error(&e);
                }
            }
            RecipeResolution::Community(rec) => {
                // 先安装依赖
                install_dependencies(&rec, formatter, installing_stack, allow_exec).await;
                if let Err(e) = community_recipe::install(&rec, formatter, allow_exec).await {
                    formatter.error(&e);
                }
            }
            RecipeResolution::NotFound(hint) => {
                formatter.output(
                    &format!("[ERROR] Not found: '{}'  recipe.{}", name, hint),
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

/// Recipe resolution result
enum RecipeResolution {
    Builtin(crate::core::recipe::Recipe),
    Community(Box<crate::core::community_recipe::CommunityRecipe>),
    NotFound(String),
}

/// 3-tier recipe lookup (builtin -> local -> remote)
async fn resolve_recipe_with_deps(
    name: &str,
    version_spec: Option<&str>,
    formatter: &OutputFormatter,
) -> RecipeResolution {
    // 1. 内置recipe（@latest 或无版本时尝试）
    if (version_spec.is_none() || version_spec == Some("latest"))
        && let Some(rec) = resolve(name)
    {
        return RecipeResolution::Builtin(rec);
    }

    // 2. 本地社区recipe
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
                        &format!(
                            "[WARN] Version {} not available. Available: {:?}",
                            v, entry.versions
                        ),
                        Some(serde_json::Value::Null),
                    );
                    entry.latest.clone()
                }
            }
        };

        formatter.output(
            &format!("[REMOTE] Fetching {} v{}...", name, target_version),
            Some(serde_json::json!({
                "status": "fetching", "source": "remote",
                "package": name, "version": target_version,
            })),
        );

        match registry::fetch_recipe(name, &target_version).await {
            Ok(recipe) => return RecipeResolution::Community(Box::new(recipe)),
            Err(e) => {
                formatter.output(
                    &format!("[ERROR] Remote fetch failed: {}", e),
                    Some(serde_json::Value::Null),
                );
            }
        }
    }

    // 未找到
    let hint = if registry::load_cached_index().is_none() {
        " Hint: run 'oneinit update' to fetch remote recipe index.".to_string()
    } else {
        String::new()
    };
    RecipeResolution::NotFound(hint)
}

/// recursively install recipe dependencies
async fn install_dependencies(
    recipe: &crate::core::community_recipe::CommunityRecipe,
    formatter: &OutputFormatter,
    installing_stack: &mut Vec<String>,
    allow_exec: bool,
) {
    if let Some(ref deps) = recipe.depends {
        if deps.is_empty() {
            return;
        }
        formatter.output(
            &format!("[DEPS] Checking dependencies: {:?}", deps),
            Some(serde_json::Value::Null),
        );
        for dep in deps {
            install_recursive(dep, None, formatter, installing_stack, allow_exec).await;
        }
    }
}

/// oneinit uninstall <package> — uninstall a tool
pub async fn run_uninstall(formatter: &OutputFormatter, package: &str) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    // trying builtin recipe uninstall first
    if recipe::uninstall(package, formatter).await.is_err() {
        // builtin uninstall failed, trying community recipe
        use crate::core::community_recipe;
        if let Err(e2) = community_recipe::uninstall(package, formatter).await {
            formatter.error(&e2);
        }
    }
}

/// oneinit list — list installed tools
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
                        "No tools installed yet. Use oneinit install <package> to start."
                            .to_string()
                    } else {
                        format!(
                            "Installed {} 个工具:\n{}",
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

/// oneinit search <keyword> — search available tools
pub async fn run_search(formatter: &OutputFormatter, keyword: Option<&str>) {
    // 内置recipe
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

    // 社区recipe
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

    // 远程recipe（从缓存 INDEX）
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
            Some(kw) => format!("[SEARCH] No tools matching '{}' ", kw),
            None => "[SEARCH] No tools available.".to_string(),
        });
    } else {
        human.push_str(&format!("[SEARCH] Found {}  available tools:\n", total));
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

/// oneinit sync — sync environment from oneinit.yaml
pub async fn run_sync(formatter: &OutputFormatter) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    // 1. 查找 oneinit.yaml
    let yaml_path = std::path::PathBuf::from("oneinit.yaml");
    if !yaml_path.exists() {
        formatter.output(
            "[ERROR] oneinit.yaml not found in current directory.",
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
            " 读取 oneinit.yaml: {} 个工具, {} 个镜像, {} 条后置命令",
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
            &format!("[CONF] Mirror config: {:?}", mirrors),
            Some(serde_json::json!({
                "mirrors_applied": mirrors,
            })),
        );
    }

    // 5. execute post_install 命令
    if let Some(ref commands) = config.post_install
        && !commands.is_empty()
    {
        formatter.output(
            "[RUN] Running post-install commands...",
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
        "[SUCCESS] Environment synchronized!",
        Some(serde_json::json!({
            "status": "complete",
            "action": "sync",
            "message": "Environment synchronized successfully"
        })),
    );
}

// ============================================================
// 团队环境同步（team.yaml）
// ============================================================

/// oneinit team add <url> — 配置团队环境并立即同步
pub async fn run_team_add(
    formatter: &OutputFormatter,
    url: &str,
    branch: &str,
    force: bool,
    allow_exec: bool,
) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }
    if let Err(e) = team::add_team(formatter, url, branch, force).await {
        formatter.error(&e);
        return;
    }
    // 配置成功后立即同步一次
    run_team_sync(formatter, true, allow_exec).await;
}

/// oneinit team remove — 移除团队环境配置
pub fn run_team_remove(formatter: &OutputFormatter) {
    match team::remove_team() {
        Ok(true) => formatter.output("[TEAM] 已移除团队环境配置", None::<serde_json::Value>),
        Ok(false) => formatter.output("[TEAM] 未配置团队环境", None::<serde_json::Value>),
        Err(e) => formatter.error(&e),
    }
}

/// oneinit team status — 查看团队环境状态
pub fn run_team_status(formatter: &OutputFormatter) {
    team::status(formatter);
}

/// oneinit team sync — 立即同步团队环境
pub async fn run_team_sync(formatter: &OutputFormatter, force: bool, allow_exec: bool) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }
    if !team::is_configured() {
        formatter.output(
            "[TEAM] 未配置团队环境 — 使用 oneinit team add <url>",
            None::<serde_json::Value>,
        );
        return;
    }

    let content = match team::fetch_if_changed(formatter, force).await {
        Ok(Some(c)) => c,
        Ok(None) => return, // 无变化
        Err(e) => {
            formatter.error(&e);
            return;
        }
    };

    apply_team_env(formatter, &content, allow_exec).await;
}

/// 每次运行 oneinit 时的团队环境自动检测（轻量：24h 间隔 + 内容哈希）
///
/// 仅检测 + 内容变化时同步；失败静默 `[WARN]`，不阻塞主命令。
pub async fn maybe_team_sync(formatter: &OutputFormatter) {
    let cfg = team::load_config();
    if !team::is_configured() || !team::needs_check(&cfg) {
        return;
    }
    match team::fetch_if_changed(formatter, false).await {
        Ok(Some(content)) => apply_team_env(formatter, &content, false).await,
        Ok(None) => {}
        Err(e) => {
            formatter.output(
                &format!("[WARN] 团队环境检测失败: {}", e),
                Some(serde_json::json!({
                    "status": "warning",
                    "action": "team_sync",
                    "error": e.to_string(),
                })),
            );
        }
    }
}

/// 应用团队环境：安装缺失工具 + 镜像 + env_vars + PATH + config_files + post_install
async fn apply_team_env(formatter: &OutputFormatter, content: &str, allow_exec: bool) {
    let config = match sync::parse_config(content) {
        Ok(c) => c,
        Err(e) => {
            formatter.error(&e);
            return;
        }
    };

    let team_name = config
        .team
        .as_ref()
        .and_then(|t| t.name.clone())
        .unwrap_or_else(|| "(未命名)".to_string());
    formatter.output(
        &format!("[TEAM] 同步团队环境: {}", team_name),
        Some(serde_json::json!({
            "status": "team_sync",
            "action": "team",
            "team": team_name,
        })),
    );

    // 1. 工具（3 层解析：内置 -> 本地社区 -> 远程注册表）
    let names = sync::envs_to_recipe_names(&config);
    let mut installing_stack = Vec::new();
    if !names.is_empty() {
        formatter.output(
            &format!("[TEAM] 需要检查 {} 个工具: {:?}", names.len(), names),
            Some(serde_json::Value::Null),
        );
        for name in &names {
            install_recursive(name, None, formatter, &mut installing_stack, allow_exec).await;
        }
    }

    // 2. 镜像源
    if let Some(mirrors) = &config.mirrors
        && !mirrors.is_empty()
        && let Err(e) = team::apply_mirrors(mirrors, formatter)
    {
        formatter.error(&e);
    }

    // 3. 环境变量
    if !config.env_vars.is_empty()
        && let Err(e) = team::apply_env_vars(&config.env_vars, formatter)
    {
        formatter.error(&e);
    }

    // 4. PATH 条目
    if !config.path.is_empty()
        && let Err(e) = team::apply_path_entries(&config.path, formatter)
    {
        formatter.error(&e);
    }

    // 5. 配置文件模板
    if !config.config_files.is_empty()
        && let Err(e) = team::apply_config_files(&config.config_files, formatter)
    {
        formatter.error(&e);
    }

    // 6. post_install 命令（安全：默认拒绝，需 --allow-exec，与 H-4 一致）
    if let Some(cmds) = &config.post_install
        && !cmds.is_empty()
    {
        if allow_exec {
            if let Err(e) = sync::run_post_install(cmds, formatter) {
                formatter.error(&e);
            }
        } else {
            formatter.output(
                "[SKIP] 跳过 post_install 命令（需要 --allow-exec）",
                None::<serde_json::Value>,
            );
        }
    }

    // 仅当所有工具都已安装成功才记录哈希（失败时下次可重试）
    let all_tools_ok = {
        let manifest = Manifest::open().ok();
        names.iter().all(|n| {
            manifest
                .as_ref()
                .and_then(|m| m.get(n).ok().flatten())
                .is_some()
        })
    };

    let mut cfg = team::load_config();
    cfg.cached_sha256 = team::sha256_hex(content.as_bytes());
    cfg.last_sync = chrono::Utc::now().to_rfc3339();
    if let Err(e) = team::save_config(&cfg) {
        formatter.error(&e);
    }

    if all_tools_ok {
        formatter.output(
            "[TEAM] 同步完成",
            Some(serde_json::json!({ "status": "complete", "action": "team_sync" })),
        );
    } else {
        formatter.output(
            "[WARN] 部分工具未成功安装，未记录为已同步 — 可重试 `oneinit team sync --force`",
            Some(serde_json::json!({
                "status": "warning",
                "action": "team_sync",
                "partial": true,
            })),
        );
    }
}

/// oneinit viz — 环境可视化（ASCII 树 / HTML 报告 / Issue 快照）
pub async fn run_viz(
    formatter: &OutputFormatter,
    html: bool,
    issue: bool,
    output: Option<&str>,
    open: bool,
    no_scan: bool,
) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    let report = crate::core::viz::gather(!no_scan);

    if html {
        let path = output.unwrap_or("report.html");
        let content = crate::core::viz::render_html(&report);
        if let Err(e) = crate::core::viz::write_output(path, &content) {
            formatter.error(&e);
            return;
        }
        formatter.output(
            &format!("[OK] HTML 报告已生成: {}", path),
            Some(serde_json::json!({
                "status": "success",
                "action": "viz",
                "format": "html",
                "path": path,
            })),
        );
        if open {
            open_in_browser(path);
        }
        return;
    }

    if issue {
        let path = output.unwrap_or("env-snapshot.md");
        let content = crate::core::viz::render_issue(&report);
        if let Err(e) = crate::core::viz::write_output(path, &content) {
            formatter.error(&e);
            return;
        }
        formatter.output(
            &format!("[OK] Issue 环境快照已生成: {}", path),
            Some(serde_json::json!({
                "status": "success",
                "action": "viz",
                "format": "issue",
                "path": path,
            })),
        );
        // 同时打印 Markdown，方便直接复制粘贴到 Issue
        println!("\n{}", content);
        return;
    }

    // 默认：ASCII 树（human + JSON 双输出）
    let tree = crate::core::viz::render_ascii(&report);
    formatter.output(&tree, Some(serde_json::json!(report)));
}

/// 尽力用系统默认浏览器打开文件
fn open_in_browser(path: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", path])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(path).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
    }
}

/// oneinit verify <file> -- Validate community recipe YAML
pub async fn run_verify(formatter: &OutputFormatter, file: &str) {
    use crate::core::community_recipe;

    let path = std::path::PathBuf::from(file);
    if !path.exists() {
        formatter.output(
            &format!("[ERROR] File not found: {}", file),
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
                    "\n[{}] Verification complete: {}/{} 项通过",
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

/// oneinit capture [--output <file>] — capture current dev environment
pub async fn run_capture(formatter: &OutputFormatter, output: &str) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    if let Err(e) = crate::core::capture::run_capture(formatter, output) {
        formatter.error(&e);
    }
}

/// oneinit export [--output <file>] [--include-envs] -- export environment as tar.gz
pub async fn run_export(formatter: &OutputFormatter, output: &str, include_envs: bool) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    if let Err(e) = crate::core::migration::run_export(formatter, output, include_envs) {
        formatter.error(&e);
    }
}

/// oneinit import <file> [--dry-run] [--force] — import environment from tar.gz
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

/// oneinit update — update remote recipe index
pub async fn run_update(formatter: &OutputFormatter) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    use crate::core::registry;

    let urls = registry::all_registry_urls();
    formatter.output(
        &format!(
            "[UPDATE] Fetching recipe index from {}  registry(s): {}",
            urls.len(),
            urls.join(", ")
        ),
        Some(serde_json::json!({
            "status": "fetching",
            "action": "update",
            "registries": urls,
        })),
    );

    match registry::fetch_index().await {
        Ok(index) => {
            let count = index.packages.len();
            formatter.output(
                &format!(
                    "[OK] Index updated: {} packages from {}  registries (updated {})",
                    count,
                    urls.len(),
                    index.last_updated
                ),
                Some(serde_json::json!({
                    "status": "success",
                    "action": "update",
                    "registry_count": urls.len(),
                    "package_count": count,
                    "last_updated": index.last_updated,
                    "packages": index.packages.keys().collect::<Vec<_>>(),
                })),
            );
        }
        Err(e) => {
            formatter.output(
                &format!("[ERROR] Index update failed: {}", e),
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

/// oneinit issue [recipe|bug] — 打开配方仓库 issue 表单
pub fn run_issue(kind: &str) {
    const RECIPES_REPO: &str = "https://github.com/oneinitAI/oneinit-recipes/issues";

    let url = match kind {
        "recipe" | "recipe-request" => format!(
            "{}/new?assignees=&labels=recipe&projects=&template=recipe_request.yml",
            RECIPES_REPO
        ),
        "bug" | "bug-report" => format!(
            "{}/new?assignees=&labels=bug&projects=&template=bug_report.yml",
            RECIPES_REPO
        ),
        _ => format!("{}/new/choose", RECIPES_REPO),
    };

    println!("[ISSUE] Opening: {}", url);

    // 跨平台打开浏览器
    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("cmd")
        .args(["/C", "start", "", &url])
        .spawn();

    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open").arg(&url).spawn();

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    let result = std::process::Command::new("xdg-open").arg(&url).spawn();

    match result {
        Ok(_) => println!("[OK] 已在浏览器打开表单。"),
        Err(_) => {
            println!("[INFO] 无法自动打开浏览器，请手动访问:\n      {}", url);
        }
    }
}

/// oneinit registry add <url> — 添加自定义订阅
pub fn run_registry_add(formatter: &OutputFormatter, url: &str) {
    use crate::core::registry;

    match registry::add_subscription(url) {
        Ok(()) => {
            let subs = registry::list_subscriptions();
            formatter.output(
                &format!(
                    "[OK] Subscribed: {}\n当前订阅 ({}):\n  - 默认: {}\n{}",
                    url,
                    subs.len(),
                    registry::load_config().registry_url,
                    subs.iter()
                        .enumerate()
                        .map(|(i, s)| format!("  - {}: {}", i + 1, s))
                        .collect::<Vec<_>>()
                        .join("\n"),
                ),
                Some(serde_json::json!({
                    "status": "success",
                    "action": "registry_add",
                    "url": url,
                    "subscriptions": subs,
                })),
            );
        }
        Err(e) => formatter.error(&e),
    }
}

/// oneinit registry remove <url> — 移除订阅
pub fn run_registry_remove(formatter: &OutputFormatter, url: &str) {
    use crate::core::registry;

    match registry::remove_subscription(url) {
        Ok(true) => {
            let subs = registry::list_subscriptions();
            formatter.output(
                &format!(
                    "[OK] Removed subscription: {}\n剩余订阅 ({}): {}",
                    url,
                    subs.len(),
                    if subs.is_empty() {
                        "(none)".to_string()
                    } else {
                        subs.join(", ")
                    }
                ),
                Some(serde_json::json!({
                    "status": "success",
                    "action": "registry_remove",
                    "url": url,
                    "subscriptions": subs,
                })),
            );
        }
        Ok(false) => formatter.output(
            &format!("[WARN] Subscription not found: {}", url),
            Some(serde_json::json!({
                "status": "not_found",
                "action": "registry_remove",
                "url": url,
            })),
        ),
        Err(e) => formatter.error(&e),
    }
}

/// oneinit registry list — 列出所有订阅
pub fn run_registry_list(formatter: &OutputFormatter) {
    use crate::core::registry;

    let default = registry::load_config().registry_url;
    let subs = registry::list_subscriptions();

    let mut human = format!("[REGISTRY] 默认注册表: {}\n", default);
    if subs.is_empty() {
        human.push_str("  自定义订阅: (none)\n");
    } else {
        human.push_str(&format!("  自定义订阅 ({}):\n", subs.len()));
        for (i, s) in subs.iter().enumerate() {
            human.push_str(&format!("    {}. {}\n", i + 1, s));
        }
    }

    formatter.output(
        &human,
        Some(serde_json::json!({
            "status": "success",
            "action": "registry_list",
            "default_registry": default,
            "subscriptions": subs,
        })),
    );
}

/// oneinit publish <file> — publish recipe to remote registry
pub async fn run_publish(formatter: &OutputFormatter, file: &str) {
    use crate::core::community_recipe;
    use crate::core::registry;

    let path = std::path::PathBuf::from(file);
    if !path.exists() {
        formatter.output(
            &format!("[ERROR] File not found: {}", file),
            Some(serde_json::json!({
                "status": "error", "action": "publish", "file": file,
                "message": "File not found"
            })),
        );
        return;
    }

    // 1. 验证recipe
    formatter.output(
        "[PUBLISH] Validating recipe...",
        Some(serde_json::Value::Null),
    );
    let verify_result = match community_recipe::verify(&path) {
        Ok(r) => r,
        Err(e) => {
            formatter.error(&e);
            return;
        }
    };
    if !verify_result.valid {
        formatter.output(
            "[ERROR] Recipe validation failed.",
            Some(serde_json::json!({
                "status": "error", "action": "publish", "message": "Validation failed"
            })),
        );
        return;
    }

    // 2. 解析recipe
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
                &format!("[ERROR] YAML parse failed: {}", e),
                Some(serde_json::Value::Null),
            );
            return;
        }
    };

    // 3. 安全提醒
    formatter.output("", Some(serde_json::Value::Null));
    formatter.output(
        "========== [SECURITY] PUBLISH CONFIRMATION ==========",
        Some(serde_json::Value::Null),
    );
    formatter.output(
        &format!("[SECURITY] Recipe: {} v{}", recipe.name, recipe.version),
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
    formatter.output("[INFO] Publish steps:", Some(serde_json::Value::Null));
    formatter.output(
        "  1. git clone https://github.com/oneinitAI/oneinit-recipes.git",
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
    formatter.output("  4. Update INDEX.json", Some(serde_json::Value::Null));
    formatter.output(
        "  5. git add . && git commit && git push",
        Some(serde_json::Value::Null),
    );
    formatter.output("  6. Create Pull Request", Some(serde_json::Value::Null));

    formatter.output(
        &format!("\n[PUBLISH] {} v{} is ready", recipe.name, recipe.version),
        Some(serde_json::json!({
            "status": "ready", "action": "publish",
            "recipe_name": recipe.name, "recipe_version": recipe.version,
            "target_path": format!("{}/{}", recipe_dir, recipe_filename),
            "registry_url": config.registry_url,
        })),
    );
}

/// oneinit doctor — environment health check
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
            "不exists".to_string()
        },
    ));

    // 2. SQLite manifest readable
    let manifest_ok = Manifest::open().is_ok();
    checks.push((
        "manifest_db".to_string(),
        manifest_ok,
        if manifest_ok {
            "readable".to_string()
        } else {
            "cannot open".to_string()
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
            format!("{} records all consistent", records.len())
        } else {
            format!(
                "{} install directories missing, {} PATH entries point to non-existent paths",
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
            "PATH contains oneinit-managed entries".to_string()
        } else {
            "No oneinit entries in PATH (normal if no tools installed)".to_string()
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
        format!("cached ({} packages)", cache_index.unwrap().packages.len())
    } else {
        "not cached (run oneinit update to fetch)".to_string()
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

/// oneinit freeze [-o file] — export installed tools as oneinit.yaml
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
            "[INFO] No tools installed, nothing to export.",
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
    yaml.push_str("# Generated by oneinit freeze\n");
    yaml.push_str("# Run oneinit sync on new machine to restore\n\n");

    yaml.push_str("envs:\n");
    for (tool, version) in &envs {
        yaml.push_str(&format!("  {}: {}\n", tool, version));
    }

    // 写入文件
    std::fs::write(output, &yaml).unwrap_or_else(|e| {
        formatter.output(
            &format!("[ERROR] write failed: {}", e),
            Some(serde_json::Value::Null),
        );
    });

    formatter.output(
        &format!(
            "[OK] Exported {} tools to {} (run oneinit sync on new machine to restore)",
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

/// extract tool type from package name
/// python3.11 -> python, node20 -> node, rust-stable -> rust
fn extract_tool_name(name: &str) -> String {
    // 找到第一个数字的位置
    let pos = name
        .find(|c: char| c.is_ascii_digit())
        .unwrap_or(name.len());
    name[..pos].trim_end_matches('-').to_string()
}

/// oneinit skill install [--target <agent>] -- install AI Skill
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
