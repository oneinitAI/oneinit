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
pub async fn run_init(
    formatter: &OutputFormatter,
    preset_name: Option<&str>,
    dry_run: bool,
    allow_exec: bool,
    project: Option<&str>,
) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    // Project-aware install: scan a project dir and install detected toolchains
    if let Some(dir) = project {
        run_init_project(formatter, dir, dry_run, allow_exec).await;
        return;
    }

    match preset_name {
        Some(name) => {
            // specified preset name, batch install
            let preset = match preset::resolve(name) {
                Some(p) => p,
                None => {
                    formatter.begin_document("init");
                    formatter.output(
                        &format!("[ERROR] 未找到预置套装: '{}'。可用套装:", name),
                        Some(serde_json::json!({
                            "status": "error",
                            "action": "init",
                            "preset": name,
                            "message": "未找到预置套装",
                            "available": preset::list_presets().iter().map(|p| p.name.clone()).collect::<Vec<_>>()
                        })),
                    );
                    list_available_presets(formatter);
                    formatter.end_document();
                    return;
                }
            };

            if preset.packages.is_empty() {
                formatter.output(
                    &format!(
                        "[WARN] 预置套装 '{}' 没有可用包。({})",
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

            // batch install（--dry-run 只预览）
            if dry_run {
                dry_run_packages(
                    formatter,
                    "init",
                    &preset.packages,
                    allow_exec,
                    false,
                    false,
                )
                .await;
                return;
            }
            batch_install(&preset.packages, formatter, allow_exec, false, false).await;
        }
        None => {
            // no preset specified, listing available presets
            formatter.begin_document("init");
            formatter.output(
                "未指定预置套装。可用套装:",
                Some(serde_json::json!({
                    "status": "success",
                    "action": "init",
                    "message": "未指定预置套装，列出可用项"
                })),
            );
            list_available_presets(formatter);
            formatter.end_document();
        }
    }
}

/// 单包 dry-run 结果（渲染文本 + 结构化摘要）
struct DryRunResult {
    text: String,
    total_ops: usize,
    operations: Vec<String>,
}

/// 单包 dry-run：四级解析配方并生成操作计划（不执行）。
/// 返回 `Some` 表示成功规划（含渲染文本与结构化摘要）；`None` 表示已安装 / 未找到 / 规划失败。
async fn dry_run_single(
    formatter: &OutputFormatter,
    name: &str,
    version_spec: Option<&str>,
    allow_exec: bool,
    refresh: bool,
    no_checksum: bool,
) -> Option<DryRunResult> {
    dry_run_single_inner(
        formatter,
        name,
        version_spec,
        allow_exec,
        refresh,
        no_checksum,
        0,
    )
    .await
}

/// dry_run_single 内部实现；`depth` 用于依赖递归的循环保护（超过 10 层停止展开）。
/// 返回 BoxFuture 以支持 async 递归（Rust 中 async fn 不能直接递归）。
fn dry_run_single_inner<'a>(
    formatter: &'a OutputFormatter,
    name: &'a str,
    version_spec: Option<&'a str>,
    allow_exec: bool,
    refresh: bool,
    no_checksum: bool,
    depth: usize,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<DryRunResult>> + 'a>> {
    Box::pin(async move {
        use crate::core::planner;

        // 依赖循环保护
        if depth > 10 {
            return None;
        }

        // 已安装 → 提示并跳过
        if let Ok(manifest) = Manifest::open()
            && let Ok(Some(record)) = manifest.get(name)
        {
            formatter.output(
                &format!(
                    "[SKIP] '{}' 已安装 v{}",
                    name,
                    record.version.as_deref().unwrap_or("?")
                ),
                None::<serde_json::Value>,
            );
            return None;
        }

        match resolve_recipe_with_deps(name, version_spec, formatter, refresh, no_checksum).await {
            RecipeResolution::Builtin(rec) => match planner::plan_builtin_install(&rec) {
                Ok(plan) => Some(DryRunResult {
                    operations: plan.operations.iter().map(|op| op.describe()).collect(),
                    total_ops: plan.summary.total_ops,
                    text: planner::render_plan(&plan, &format!("Install {name}")),
                }),
                Err(e) => {
                    formatter.error(&e);
                    None
                }
            },
            RecipeResolution::Community(rec) => {
                // 先递归渲染依赖计划
                let mut dep_text = String::new();
                if let Some(deps) = &rec.depends {
                    for dep in deps {
                        if let Some(r) = dry_run_single_inner(
                            formatter,
                            dep,
                            None,
                            allow_exec,
                            refresh,
                            no_checksum,
                            depth + 1,
                        )
                        .await
                        {
                            dep_text.push_str(&r.text);
                            dep_text.push('\n');
                        }
                    }
                }
                match planner::plan_community_install(&rec, allow_exec) {
                    Ok(plan) => {
                        let main_text = planner::render_plan(&plan, &format!("Install {name}"));
                        Some(DryRunResult {
                            operations: plan.operations.iter().map(|op| op.describe()).collect(),
                            total_ops: plan.summary.total_ops,
                            text: format!("{dep_text}{main_text}"),
                        })
                    }
                    Err(e) => {
                        formatter.error(&e);
                        None
                    }
                }
            }
            RecipeResolution::NotFound(hint) => {
                formatter.output(
                    &format!("[ERROR] 未找到: '{}' 配方。{}", name, hint),
                    None::<serde_json::Value>,
                );
                None
            }
        }
    })
}

/// Render the plan for a list of package names (four-tier resolution,
/// matching `batch_install` semantics). Used by --dry-run.
async fn dry_run_packages(
    formatter: &OutputFormatter,
    action: &str,
    packages: &[String],
    allow_exec: bool,
    refresh: bool,
    no_checksum: bool,
) {
    let mut skipped = 0usize;
    let mut planned = 0usize;
    let mut rendered = String::new();
    for name in packages {
        // already installed → skip
        if let Ok(manifest) = Manifest::open()
            && let Ok(Some(_)) = manifest.get(name)
        {
            skipped += 1;
            continue;
        }
        match dry_run_single(formatter, name, None, allow_exec, refresh, no_checksum).await {
            Some(result) => {
                rendered.push_str(&result.text);
                rendered.push('\n');
                planned += 1;
            }
            None => {}
        }
    }
    formatter.output(
        &format!(
            "[PLAN] {action} --dry-run: {} 待安装, {} 已安装跳过\n\n{}",
            planned, skipped, rendered
        ),
        Some(serde_json::json!({
            "status": "dry_run",
            "action": action,
            "planned": planned,
            "skipped": skipped,
        })),
    );
}

/// Project-aware install: scan a project directory for manifest files and
/// install the detected toolchains (init --project).
async fn run_init_project(formatter: &OutputFormatter, dir: &str, dry_run: bool, allow_exec: bool) {
    let path = std::path::Path::new(dir);
    if !path.is_dir() {
        formatter.output(
            &format!("[ERROR] 项目目录不存在: {}", dir),
            None::<serde_json::Value>,
        );
        return;
    }

    let detected = detect_project_toolchains(path);
    if detected.is_empty() {
        formatter.output(
            &format!(
                "[INFO] 在 {} 中未检测到项目清单文件（requirements.txt / pyproject.toml / package.json / Cargo.toml / go.mod）",
                path.display()
            ),
            Some(serde_json::json!({
                "status": "info",
                "action": "init_project",
                "detected": [],
            })),
        );
        return;
    }

    let mut lines = format!("[PROJECT] 检测到 {} 个项目清单:\n", detected.len());
    for (recipe, source) in &detected {
        lines.push_str(&format!("  - {}  ←  {}\n", recipe, source));
    }
    formatter.output(
        &lines,
        Some(serde_json::json!({
            "status": "success",
            "action": "init_project",
            "detected": detected,
        })),
    );

    // Install each toolchain (3-tier resolution, skips already installed)
    let packages: Vec<String> = detected.iter().map(|(r, _)| r.clone()).collect();
    if dry_run {
        dry_run_packages(
            formatter,
            "init --project",
            &packages,
            allow_exec,
            false,
            false,
        )
        .await;
        return;
    }
    let mut stack = Vec::new();
    for pkg in &packages {
        install_recursive(
            pkg, None, formatter, &mut stack, allow_exec, false, false, false,
        )
        .await;
    }
    formatter.output(
        "[PROJECT] 项目环境就绪 ✓",
        Some(serde_json::json!({
            "status": "complete",
            "action": "init_project",
            "packages": packages,
        })),
    );
}

/// Detect toolchains required by a project from common manifest files.
/// Returns (recipe_name, source_file) pairs.
fn detect_project_toolchains(dir: &std::path::Path) -> Vec<(String, String)> {
    let mut result = Vec::new();
    if dir.join("Cargo.toml").exists() {
        result.push(("rust".to_string(), "Cargo.toml".to_string()));
    }
    if dir.join("go.mod").exists() {
        result.push(("go".to_string(), "go.mod".to_string()));
    }
    if dir.join("package.json").exists() {
        result.push(("node20".to_string(), "package.json".to_string()));
    }
    if dir.join("requirements.txt").exists()
        || dir.join("pyproject.toml").exists()
        || dir.join("setup.py").exists()
    {
        result.push((
            "python3.11".to_string(),
            "requirements.txt / pyproject.toml".to_string(),
        ));
    }
    result
}

/// list all available presets
fn list_available_presets(formatter: &OutputFormatter) {
    formatter.begin_document("init_presets");
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
    formatter.end_document();
}

/// batch install recipe list（与单包 install 一致的四级解析）
async fn batch_install(
    packages: &[String],
    formatter: &OutputFormatter,
    allow_exec: bool,
    refresh: bool,
    no_checksum: bool,
) {
    let mut succeeded: Vec<&str> = Vec::new();
    let mut skipped: Vec<&str> = Vec::new();
    let mut failed: Vec<(&str, String)> = Vec::new();

    let mut installing_stack = Vec::new();
    for pkg_name in packages {
        let outcome = install_recursive(
            pkg_name,
            None,
            formatter,
            &mut installing_stack,
            allow_exec,
            refresh,
            no_checksum,
            false,
        )
        .await;
        match outcome {
            InstallOutcome::Installed => succeeded.push(pkg_name),
            InstallOutcome::AlreadyInstalled | InstallOutcome::Skipped(_) => skipped.push(pkg_name),
            InstallOutcome::Failed(e) => failed.push((pkg_name, e)),
        }
    }

    // 输出总结
    formatter.output(
        &format!(
            "\n[SUMMARY] 初始化完成: {} 成功, {} 跳过, {} 失败",
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
pub async fn run_install(
    formatter: &OutputFormatter,
    package: &str,
    allow_exec: bool,
    dry_run: bool,
    refresh: bool,
    no_checksum: bool,
    no_rollback: bool,
) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    // parse name@version syntax
    let (name, version_spec) = parse_package_spec(package);

    // --dry-run: resolve and preview the plan without executing
    if dry_run {
        dry_run_install(
            formatter,
            &name,
            version_spec.as_deref(),
            allow_exec,
            refresh,
            no_checksum,
        )
        .await;
        return;
    }

    // recursive install (handle dependencies)
    install_recursive(
        &name,
        version_spec.as_deref(),
        formatter,
        &mut Vec::new(),
        allow_exec,
        refresh,
        no_checksum,
        no_rollback,
    )
    .await;
}

/// Preview the operations a single install would perform (no execution).
async fn dry_run_install(
    formatter: &OutputFormatter,
    name: &str,
    version_spec: Option<&str>,
    allow_exec: bool,
    refresh: bool,
    no_checksum: bool,
) {
    match dry_run_single(
        formatter,
        name,
        version_spec,
        allow_exec,
        refresh,
        no_checksum,
    )
    .await
    {
        Some(result) => {
            formatter.output(
                &result.text,
                Some(serde_json::json!({
                    "status": "dry_run",
                    "action": "install",
                    "package": name,
                    "total_ops": result.total_ops,
                    "operations": result.operations,
                })),
            );
        }
        None => {}
    }
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

/// 单次安装的结果（供批量安装与依赖安装汇总）
#[derive(Debug, Clone)]
pub enum InstallOutcome {
    /// 安装成功
    Installed,
    /// 已安装，跳过
    AlreadyInstalled,
    /// 跳过（循环依赖等）
    Skipped(String),
    /// 安装失败
    Failed(String),
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
    refresh: bool,
    no_checksum: bool,
    no_rollback: bool,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = InstallOutcome> + 'a>> {
    Box::pin(async move {
        // 防止循环依赖
        if installing_stack.iter().any(|n| n == name) {
            formatter.output(
                &format!("[WARN] 跳过循环依赖: {}", name),
                Some(serde_json::Value::Null),
            );
            return InstallOutcome::Skipped(format!("circular dependency: {name}"));
        }
        installing_stack.push(name.to_string());

        // check if already installed (by the requested name)
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
            return InstallOutcome::AlreadyInstalled;
        }

        // find recipe（内置 -> 本地社区 -> 远程 -> 动态非完全匹配），同时获取依赖信息
        let recipe_info =
            resolve_recipe_with_deps(name, version_spec, formatter, refresh, no_checksum).await;

        let outcome = match recipe_info {
            RecipeResolution::Builtin(rec) => {
                match recipe::install(&rec, formatter, no_rollback).await {
                    Ok(()) => InstallOutcome::Installed,
                    Err(e) => {
                        formatter.error(&e);
                        InstallOutcome::Failed(e.to_string())
                    }
                }
            }
            RecipeResolution::Community(rec) => {
                // 动态配方可能以 family@version 命名 — 检查是否已装
                if rec.name != name
                    && let Ok(manifest) = Manifest::open()
                    && let Ok(Some(_)) = manifest.get(&rec.name)
                {
                    formatter.output(
                        &format!("[OK] '{}' 已安装 v{}", rec.name, rec.version),
                        None::<serde_json::Value>,
                    );
                    installing_stack.pop();
                    return InstallOutcome::AlreadyInstalled;
                }
                // 先安装依赖；依赖失败则中止主包安装
                let deps_ok = install_dependencies(
                    &rec,
                    formatter,
                    installing_stack,
                    allow_exec,
                    refresh,
                    no_checksum,
                    no_rollback,
                )
                .await;
                if !deps_ok {
                    installing_stack.pop();
                    return InstallOutcome::Failed(format!("依赖安装失败: {}", rec.name));
                }
                match community_recipe::install(&rec, formatter, allow_exec, no_rollback).await {
                    Ok(()) => InstallOutcome::Installed,
                    Err(e) => {
                        formatter.error(&e);
                        InstallOutcome::Failed(e.to_string())
                    }
                }
            }
            RecipeResolution::NotFound(hint) => {
                formatter.output(
                    &format!("[ERROR] 未找到: '{}' 配方。{}", name, hint),
                    Some(serde_json::json!({
                        "status": "error", "action": "install",
                        "package": name, "message": "Recipe not found",
                    })),
                );
                InstallOutcome::Failed(format!("未找到配方: {name}"))
            }
        };

        installing_stack.pop();
        outcome
    })
}

/// Recipe resolution result
enum RecipeResolution {
    Builtin(crate::core::recipe::Recipe),
    Community(Box<crate::core::community_recipe::CommunityRecipe>),
    NotFound(String),
}

/// 3-tier recipe lookup (builtin -> local community -> remote -> dynamic)
async fn resolve_recipe_with_deps(
    name: &str,
    version_spec: Option<&str>,
    formatter: &OutputFormatter,
    refresh: bool,
    no_checksum: bool,
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

    // 3. 远程注册表（versioned families 交给动态配方系统处理，避免注册表
    //    版本回退覆盖 @version 语义）
    if !crate::core::version::is_versioned(name)
        && let Some(entry) = registry::resolve(name)
    {
        let target_version = match version_spec {
            Some("latest") | None => entry.latest.clone(),
            Some(v) => {
                if entry.versions.contains(&v.to_string()) {
                    v.to_string()
                } else {
                    formatter.output(
                        &format!("[WARN] 版本 {} 不可用。可用版本: {:?}", v, entry.versions),
                        Some(serde_json::Value::Null),
                    );
                    entry.latest.clone()
                }
            }
        };

        formatter.output(
            &format!("[REMOTE] 拉取 {} v{}...", name, target_version),
            Some(serde_json::json!({
                "status": "fetching", "source": "remote",
                "package": name, "version": target_version,
            })),
        );

        match registry::fetch_recipe(name, &target_version).await {
            Ok(recipe) => return RecipeResolution::Community(Box::new(recipe)),
            Err(e) => {
                formatter.output(
                    &format!("[ERROR] 远程拉取失败: {}", e),
                    Some(serde_json::Value::Null),
                );
            }
        }
    }

    // 4. 动态非完全匹配（versioned family + @version / 默认版本 / 旧名重定向）
    if let Some(resolution) = try_dynamic(name, version_spec, refresh, no_checksum).await {
        return resolution;
    }

    // 未找到
    let hint = if registry::load_cached_index().is_none() {
        " Hint: run 'oneinit update' to fetch remote recipe index.".to_string()
    } else {
        String::new()
    };
    RecipeResolution::NotFound(hint)
}

/// Try to resolve a non-exact-match (dynamic) recipe:
/// - `python@3.11` / `node@lts` / `go@latest` (versioned family + spec)
/// - `python` (family, default version)
/// - old-style names `python3.12` / `node18` (family + version suffix)
async fn try_dynamic(
    name: &str,
    version_spec: Option<&str>,
    refresh: bool,
    no_checksum: bool,
) -> Option<RecipeResolution> {
    use crate::core::{dynamic, version};

    // Determine (family, spec): explicit spec wins; else old-name redirect
    let (family, spec): (String, Option<String>) = if version_spec.is_some() {
        (name.to_string(), version_spec.map(|s| s.to_string()))
    } else if let Some((f, v)) = old_name_redirect(name) {
        (f, Some(v))
    } else if version::is_versioned(name) {
        (name.to_string(), None) // default version
    } else {
        return None;
    };

    if !version::is_versioned(&family) {
        return None;
    }

    // Resolve the concrete version
    let resolved = match version::resolve(&family, spec.as_deref()) {
        Ok(v) => v,
        Err(e) => {
            return Some(RecipeResolution::NotFound(format!(" {}", e)));
        }
    };

    if refresh {
        let _ = version::refresh(&family).await;
    }

    match dynamic::build(&family, &resolved, refresh, no_checksum).await {
        Ok(recipe) => {
            crate::core::cache_db::cache_version(&family, &resolved, "resolved").ok();
            Some(RecipeResolution::Community(Box::new(recipe)))
        }
        Err(e) => Some(RecipeResolution::NotFound(format!(" 动态配方失败: {}", e))),
    }
}

/// Old-style recipe name → (family, version suffix):
/// `python3.12` → ("python", "3.12"); `node18` → ("node", "18");
/// `java17` → ("java", "17"); `go1.23` → ("go", "1.23").
fn old_name_redirect(name: &str) -> Option<(String, String)> {
    const FAMILIES: [&str; 5] = ["python", "node", "go", "java", "rust"];
    for family in FAMILIES {
        if let Some(rest) = name.strip_prefix(family)
            && !rest.is_empty()
            && rest
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            return Some((family.to_string(), rest.to_string()));
        }
    }
    None
}

/// recursively install recipe dependencies
///
/// 返回是否全部成功；任一依赖失败即返回 `false`（调用方应中止主包安装）。
async fn install_dependencies(
    recipe: &crate::core::community_recipe::CommunityRecipe,
    formatter: &OutputFormatter,
    installing_stack: &mut Vec<String>,
    allow_exec: bool,
    refresh: bool,
    no_checksum: bool,
    no_rollback: bool,
) -> bool {
    let Some(ref deps) = recipe.depends else {
        return true;
    };
    if deps.is_empty() {
        return true;
    }
    formatter.output(
        &format!("[DEPS] 检查依赖: {:?}", deps),
        Some(serde_json::Value::Null),
    );
    for dep in deps {
        let outcome = install_recursive(
            dep,
            None,
            formatter,
            installing_stack,
            allow_exec,
            refresh,
            no_checksum,
            no_rollback,
        )
        .await;
        if matches!(outcome, InstallOutcome::Failed(_)) {
            formatter.output(
                &format!("[ERROR] 依赖 '{}' 安装失败，中止安装主包", dep),
                Some(serde_json::json!({
                    "status": "error",
                    "action": "install",
                    "package": dep,
                    "message": "Dependency install failed, aborting parent install",
                })),
            );
            return false;
        }
    }
    true
}

/// oneinit uninstall <package> — uninstall a tool
pub async fn run_uninstall(formatter: &OutputFormatter, package: &str, dry_run: bool) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    // --dry-run: preview what would be removed
    if dry_run {
        if let Ok(manifest) = Manifest::open()
            && let Ok(Some(record)) = manifest.get(package)
        {
            let plan = crate::core::planner::plan_uninstall(&record);
            formatter.output(
                &crate::core::planner::render_plan(&plan, &format!("Uninstall {package}")),
                Some(serde_json::json!({
                    "status": "dry_run",
                    "action": "uninstall",
                    "package": package,
                    "total_ops": plan.summary.total_ops,
                })),
            );
        } else {
            formatter.output(
                &format!("[ERROR] '{}' 未安装", package),
                None::<serde_json::Value>,
            );
        }
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

/// oneinit list — list installed tools (--format table|csv)
pub async fn run_list(formatter: &OutputFormatter, format: Option<&str>) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    use crate::core::manifest::Manifest;

    match Manifest::open() {
        Ok(manifest) => match manifest.list() {
            Ok(records) => {
                if format == Some("csv") {
                    // CSV with header
                    let mut csv = String::from("name,version,status,install_path\n");
                    for r in &records {
                        csv.push_str(&format!(
                            "{},{},{},{}\n",
                            r.name,
                            r.version.as_deref().unwrap_or("?"),
                            "installed",
                            r.install_path
                        ));
                    }
                    formatter.output(
                        &csv,
                        Some(serde_json::json!({
                            "status": "success",
                            "action": "list",
                            "format": "csv",
                            "count": records.len()
                        })),
                    );
                    return;
                }
                let rows: Vec<Vec<String>> = records
                    .iter()
                    .map(|r| {
                        vec![
                            r.name.clone(),
                            r.version.clone().unwrap_or_else(|| "?".to_string()),
                            "installed".to_string(),
                            r.install_path.clone(),
                        ]
                    })
                    .collect();
                let rendered = render_table(
                    &["名称", "版本", "状态", "路径"],
                    &rows,
                    if records.is_empty() {
                        Some("尚未安装任何工具。使用 oneinit install <包名> 开始。")
                    } else {
                        None
                    },
                );
                formatter.output(
                    &rendered,
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
                "verified": r.verified.unwrap_or(false),
                "license": r.license,
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

    // versioned families（动态配方：支持 name@version / @latest / @lts）
    let families: Vec<serde_json::Value> = crate::core::version::FAMILIES
        .iter()
        .filter(|f| keyword.is_none_or(|kw| f.contains(&kw.to_lowercase())))
        .map(|f| {
            serde_json::json!({
                "name": f,
                "version": "@latest",
                "display_name": format!("{} (versioned — install {}@3.x / @lts / @latest)", f, f),
                "source": "dynamic",
            })
        })
        .collect();

    let total = builtin.len() + community.len() + remote.len() + families.len();

    let mut human = String::new();
    if total == 0 {
        human.push_str(&match keyword {
            Some(kw) => format!("[SEARCH] 没有匹配 '{}' 的工具 ", kw),
            None => "[SEARCH] 没有可用工具。".to_string(),
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
        for r in &families {
            human.push_str(&format!("  - {} v{} [dynamic]\n", r["name"], r["version"]));
        }
    }

    let mut all_results = builtin.clone();
    all_results.extend(community);
    all_results.extend(remote);
    all_results.extend(families);

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
pub async fn run_sync(formatter: &OutputFormatter, dry_run: bool, allow_exec: bool) {
    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    // 1. 查找 oneinit.yaml
    let yaml_path = std::path::PathBuf::from("oneinit.yaml");
    if !yaml_path.exists() {
        formatter.output(
            "[ERROR] 当前目录未找到 oneinit.yaml。",
            Some(serde_json::json!({
                "status": "error",
                "action": "sync",
                "message": "当前目录未找到 oneinit.yaml"
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
    if dry_run {
        dry_run_packages(formatter, "sync", &recipe_names, allow_exec, false, false).await;
        return;
    }
    batch_install(&recipe_names, formatter, allow_exec, false, false).await;

    // 4. 应用镜像配置（记录日志，未来扩展覆盖默认镜像）
    if let Some(ref mirrors) = config.mirrors {
        formatter.output(
            &format!("[CONF] 镜像配置: {:?}", mirrors),
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
            "[RUN] 正在执行安装后命令...",
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
    run_team_sync(formatter, true, allow_exec, false).await;
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
pub async fn run_team_sync(
    formatter: &OutputFormatter,
    force: bool,
    allow_exec: bool,
    dry_run: bool,
) {
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

    if dry_run {
        dry_run_team_env(formatter, &content, allow_exec);
        return;
    }
    apply_team_env(formatter, &content, allow_exec).await;
}

/// Preview a team env sync: list missing tools and the env/config operations
/// that would be applied (no execution).
fn dry_run_team_env(formatter: &OutputFormatter, content: &str, allow_exec: bool) {
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

    let mut lines = format!("[PLAN] Team env sync --dry-run ({team_name}):\n\n");
    let mut planned = 0usize;
    let mut skipped = 0usize;

    // 1. missing tools (builtin-only plan for preview)
    let names = sync::envs_to_recipe_names(&config);
    for name in &names {
        if let Ok(manifest) = Manifest::open()
            && let Ok(Some(_)) = manifest.get(name)
        {
            skipped += 1;
            continue;
        }
        match recipe::resolve(name) {
            Some(recipe) => match crate::core::planner::plan_builtin_install(&recipe) {
                Ok(plan) => {
                    lines.push_str(&format!("  [EXTRACT] Install {name}: {} operations\n", plan.summary.total_ops));
                    planned += 1;
                }
                Err(e) => lines.push_str(&format!("  [WARN] cannot plan {name}: {e}\n")),
            },
            None => lines.push_str(&format!(
                "  [EXTRACT] Install {name}: (remote/community recipe — run `oneinit install {} --dry-run` for detail)\n",
                name
            )),
        }
    }
    if !names.is_empty() {
        lines.push('\n');
    }

    // 2. mirrors
    if let Some(mirrors) = &config.mirrors
        && !mirrors.is_empty()
    {
        for (k, v) in mirrors {
            lines.push_str(&format!("  [ENV] Apply mirror {k} = {v}\n"));
            planned += 1;
        }
    }
    // 3. env vars
    for (k, v) in &config.env_vars {
        lines.push_str(&format!("  [ENV] Set env {k} = {v}\n"));
        planned += 1;
    }
    // 4. PATH
    for p in &config.path {
        lines.push_str(&format!("  [PATH+] PATH += {p}\n"));
        planned += 1;
    }
    // 5. config files
    for cf in &config.config_files {
        lines.push_str(&format!("  [WRITE] Write config file {}\n", cf.path));
        planned += 1;
    }
    // 6. post_install
    if let Some(cmds) = &config.post_install
        && !cmds.is_empty()
    {
        if allow_exec {
            for c in cmds {
                lines.push_str(&format!("  [SCRIPT] Run: {c}\n"));
                planned += 1;
            }
        } else {
            lines.push_str("  [SKIP] 跳过 post_install 命令（需要 --allow-exec）\n");
        }
    }

    lines.push_str(&format!(
        "\n[SUMMARY] 共 {planned} 项操作，{skipped} 个工具已安装\n"
    ));
    formatter.output(
        &lines,
        Some(serde_json::json!({
            "status": "dry_run",
            "action": "team_sync",
            "team": team_name,
            "planned": planned,
            "already_installed": skipped,
        })),
    );
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
            install_recursive(
                name,
                None,
                formatter,
                &mut installing_stack,
                allow_exec,
                false,
                false,
                false,
            )
            .await;
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

            formatter.begin_document("verify");
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
            formatter.end_document();
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
    formatter.begin_document("update");
    formatter.output(
        &format!(
            "[UPDATE] 从 {} 个注册表拉取配方索引: {}",
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
                    "[OK] 索引已更新: {} 个包，来自 {} 个注册表（更新于 {}）",
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
    formatter.end_document();
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
            &format!("[WARN] 未找到订阅: {}", url),
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

/// oneinit recipe new <name> — 生成配方模板文件
pub fn run_recipe_new(formatter: &OutputFormatter, name: &str) {
    let valid = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if !valid {
        formatter.output(
            &format!("[ERROR] 无效的配方名: '{}'（仅允许字母/数字/-/_）", name),
            Some(serde_json::json!({
                "status": "error", "action": "recipe_new", "name": name,
                "message": "Invalid recipe name",
            })),
        );
        return;
    }

    let path = format!("{name}.yaml");
    if std::path::Path::new(&path).exists() {
        formatter.output(
            &format!("[ERROR] 文件已存在: {path}"),
            Some(serde_json::json!({
                "status": "error", "action": "recipe_new", "name": name,
                "message": "File already exists",
            })),
        );
        return;
    }

    let template = crate::core::community_recipe::recipe_template(name);
    if let Err(e) = std::fs::write(&path, template) {
        formatter.output(
            &format!("[ERROR] 写入失败: {}", e),
            Some(serde_json::json!({
                "status": "error", "action": "recipe_new", "name": name,
                "message": e.to_string(),
            })),
        );
        return;
    }

    formatter.output(
        &format!(
            "[OK] 配方模板已生成: {}（填写 TODO 后运行 `oneinit verify {}` 校验）",
            path, path
        ),
        Some(serde_json::json!({
            "status": "success", "action": "recipe_new", "name": name,
            "file": path,
        })),
    );
}

/// oneinit publish <file> — publish recipe to remote registry
pub async fn run_publish(formatter: &OutputFormatter, file: &str, pr: bool) {
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

    // 1. 验证recipe
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
            "[ERROR] 配方验证失败。",
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
                &format!("[ERROR] YAML 解析失败: {}", e),
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
    formatter.output("[INFO] 发布步骤:", Some(serde_json::Value::Null));
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
        &format!("\n[PUBLISH] {} v{} 已就绪", recipe.name, recipe.version),
        Some(serde_json::json!({
            "status": "ready", "action": "publish",
            "recipe_name": recipe.name, "recipe_version": recipe.version,
            "target_path": format!("{}/{}", recipe_dir, recipe_filename),
            "registry_url": config.registry_url,
        })),
    );

    // --pr：自动提交配方到 oneinit-recipes 并创建 PR
    if pr {
        publish_with_pr(
            formatter,
            file,
            &recipe.name,
            &recipe.version,
            &recipe.description,
        );
    }
}

/// publish --pr：在临时目录 clone oneinit-recipes，复制配方、更新 INDEX.json、
/// 提交并 `gh pr create`。gh 不可用或任一步骤失败时给出明确提示（不静默失败）。
fn publish_with_pr(
    formatter: &OutputFormatter,
    file: &str,
    name: &str,
    version: &str,
    description: &str,
) {
    use crate::core::registry::{Index, IndexEntry};

    // 0. 前置检查：gh CLI
    let gh_ok = std::process::Command::new("gh")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !gh_ok {
        formatter.output(
            "[ERROR] 未检测到 gh CLI。请安装 GitHub CLI（https://cli.github.com）后重试，或按上面的手动步骤操作。",
            Some(serde_json::json!({
                "status": "error", "action": "publish_pr", "message": "gh CLI not found",
            })),
        );
        return;
    }

    // 1. temp 目录 clone
    let work = std::env::temp_dir().join(format!("oneinit-publish-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&work).unwrap();
    let clone_ok = std::process::Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "https://github.com/oneinitAI/oneinit-recipes.git",
        ])
        .current_dir(&work)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !clone_ok {
        formatter.output(
            "[ERROR] clone oneinit-recipes 失败（网络或权限）。请按手动步骤操作。",
            Some(serde_json::json!({
                "status": "error", "action": "publish_pr", "message": "git clone failed",
            })),
        );
        let _ = std::fs::remove_dir_all(&work);
        return;
    }
    let repo = work.join("oneinit-recipes");

    // 2. 复制配方
    let recipe_dest = repo
        .join("recipes")
        .join(name)
        .join(format!("{version}.yaml"));
    if let Some(parent) = recipe_dest.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    if std::fs::copy(file, &recipe_dest).is_err() {
        formatter.output(
            "[ERROR] 复制配方失败。",
            Some(serde_json::json!({
                "status": "error", "action": "publish_pr", "message": "copy failed",
            })),
        );
        let _ = std::fs::remove_dir_all(&work);
        return;
    }

    // 3. 更新 INDEX.json（保持 BTreeMap 排序）
    let index_path = repo.join("INDEX.json");
    if let Ok(content) = std::fs::read_to_string(&index_path)
        && let Ok(mut index) = serde_json::from_str::<Index>(&content)
    {
        let entry = index
            .packages
            .entry(name.to_string())
            .or_insert_with(|| IndexEntry {
                description: description.to_string(),
                latest: version.to_string(),
                versions: Vec::new(),
                tags: Vec::new(),
                maintainers: Vec::new(),
                source: String::new(),
            });
        if !entry.versions.contains(&version.to_string()) {
            entry.versions.push(version.to_string());
            entry.versions.sort();
        }
        if version > entry.latest.as_str() {
            entry.latest = version.to_string();
        }
        if !description.is_empty() {
            entry.description = description.to_string();
        }
        if let Ok(json) = serde_json::to_string_pretty(&index) {
            let _ = std::fs::write(&index_path, format!("{json}\n"));
        }
    }

    // 4. 提交 + 推分支 + gh pr create
    let branch = format!("recipes/{name}-{version}");
    let steps_ok = [
        std::process::Command::new("git")
            .args(["checkout", "-b", &branch])
            .current_dir(&repo)
            .status()
            .map(|s| s.success())
            .unwrap_or(false),
        std::process::Command::new("git")
            .args(["add", "-A"])
            .current_dir(&repo)
            .status()
            .map(|s| s.success())
            .unwrap_or(false),
        std::process::Command::new("git")
            .args([
                "commit",
                "-m",
                &format!("feat: add {name} {version} recipe"),
            ])
            .current_dir(&repo)
            .status()
            .map(|s| s.success())
            .unwrap_or(false),
        std::process::Command::new("git")
            .args(["push", "origin", &branch])
            .current_dir(&repo)
            .status()
            .map(|s| s.success())
            .unwrap_or(false),
    ]
    .iter()
    .all(|ok| *ok);

    if !steps_ok {
        formatter.output(
            &format!(
                "[ERROR] 提交/推送失败（可能需要 fork 或写权限）。分支保留在: {}，请手动 `gh pr create --repo oneinitAI/oneinit-recipes --head {branch}`",
                repo.display()
            ),
            Some(serde_json::json!({
                "status": "error", "action": "publish_pr", "message": "git push failed",
                "branch": branch,
            })),
        );
        return;
    }

    // 5. gh pr create
    let pr_title = format!("feat: add {name} {version} recipe");
    let pr_body = format!(
        "Add `{name}` {version} community recipe.\n\n- source: {}\n- description: {}",
        name, description
    );
    let pr = std::process::Command::new("gh")
        .args([
            "pr",
            "create",
            "--repo",
            "oneinitAI/oneinit-recipes",
            "--base",
            "main",
            "--head",
            &branch,
            "--title",
            &pr_title,
            "--body",
            &pr_body,
        ])
        .current_dir(&repo)
        .output();
    match pr {
        Ok(out) if out.status.success() => {
            let url = String::from_utf8_lossy(&out.stdout).trim().to_string();
            formatter.output(
                &format!("[OK] PR 已创建: {url}"),
                Some(serde_json::json!({
                    "status": "success", "action": "publish_pr",
                    "pr_url": url, "branch": branch,
                })),
            );
        }
        _ => {
            formatter.output(
                &format!(
                    "[WARN] 分支已推送但 PR 创建失败。手动执行: gh pr create --repo oneinitAI/oneinit-recipes --head {branch} --base main --title \"{pr_title}\""
                ),
                Some(serde_json::json!({
                    "status": "warning", "action": "publish_pr",
                    "message": "PR creation failed", "branch": branch,
                })),
            );
        }
    }

    // 清理临时 clone
    let _ = std::fs::remove_dir_all(&work);
}

/// oneinit doctor — environment health check
pub async fn run_doctor(formatter: &OutputFormatter) {
    use crate::core::doctor::{Severity, category_order, is_healthy, run_all, warning_count};

    if let Err(e) = ensure_dirs() {
        formatter.error(&e);
        return;
    }

    let results = run_all().await;

    formatter.begin_document("doctor");

    // Group by category, in the engine's canonical order.
    for &cat in category_order() {
        let rows: Vec<_> = results.iter().filter(|r| r.category == cat).collect();
        if rows.is_empty() {
            continue;
        }
        formatter.output(&format!("== {} ==", cat), Some(serde_json::Value::Null));
        for r in rows {
            let tag = match (r.passed, r.severity) {
                (_, Severity::Info) => "[INFO]",
                (true, _) => "[OK]",
                (false, Severity::Critical) => "[FAIL]",
                (false, Severity::Warning) => "[WARN]",
            };
            // Multi-line detail (e.g. license list) indents nicely under the tag.
            let indented = r.detail.replace('\n', "\n        ");
            formatter.output(
                &format!("  {}   {}", tag, indented),
                Some(serde_json::json!({
                    "category": r.category,
                    "check": r.name,
                    "passed": r.passed,
                    "severity": r.severity,
                    "detail": r.detail,
                })),
            );
        }
        formatter.output("", Some(serde_json::Value::Null));
    }

    let healthy = is_healthy(&results);
    let warnings = warning_count(&results);
    let critical_failures = results
        .iter()
        .filter(|r| !r.passed && r.severity == Severity::Critical)
        .count();
    let total = results.len();

    formatter.output(
        &format!(
            "[{}] 环境: {} 项检查, {} 警告, {} 严重问题",
            if healthy { "OK" } else { "FAIL" },
            total,
            warnings,
            critical_failures,
        ),
        Some(serde_json::json!({
            "status": if healthy { "healthy" } else { "issues" },
            "action": "doctor",
            "total_checks": total,
            "warnings": warnings,
            "critical_failures": critical_failures,
            "healthy": healthy,
        })),
    );
    formatter.end_document();
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
            "[INFO] 未安装任何工具，无需导出。",
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
            "[OK] 已导出 {} 个工具到 {}（在新机器上运行 oneinit sync 恢复）",
            records.len(),
            output
        ),
        Some(serde_json::json!({
            "status": "success", "action": "freeze",
            "output": output, "count": records.len(),
            "tools": envs.keys().collect::<Vec<_>>(),
            "envs": envs,
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

/// oneinit skill list -- 列出各 AI 助手的 Skill 安装情况
pub async fn run_skill_list(formatter: &OutputFormatter) {
    crate::skill_mgr::status(formatter);
}

/// oneinit skill status -- 查看 Skill 安装状态
pub async fn run_skill_status(formatter: &OutputFormatter) {
    crate::skill_mgr::status(formatter);
}

/// oneinit skill uninstall -- 卸载 AI Skill
pub async fn run_skill_uninstall(formatter: &OutputFormatter) {
    crate::skill_mgr::uninstall(formatter);
}

/// Render a simple ASCII table (no external dependency).
/// `empty_msg` replaces the table when `rows` is empty.
fn render_table(headers: &[&str], rows: &[Vec<String>], empty_msg: Option<&str>) -> String {
    if rows.is_empty() {
        return empty_msg.unwrap_or("(空)").to_string();
    }
    // column widths
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }
    let border = |left: &str, mid: &str, right: &str| -> String {
        let mut s = String::from(left);
        for (i, w) in widths.iter().enumerate() {
            s.push_str(&"─".repeat(w + 2));
            s.push_str(if i + 1 == widths.len() { right } else { mid });
        }
        s
    };
    let mut out = String::new();
    out.push_str(&border("┌", "┬", "┐"));
    out.push('\n');
    // header
    out.push('│');
    for (i, h) in headers.iter().enumerate() {
        out.push_str(&format!(" {} ", h));
        out.push_str(&" ".repeat(widths[i] - h.len()));
        out.push('│');
    }
    out.push('\n');
    out.push_str(&border("├", "┼", "┤"));
    out.push('\n');
    // rows
    for row in rows {
        out.push('│');
        for (i, cell) in row.iter().enumerate() {
            out.push_str(&format!(" {} ", cell));
            if i < widths.len() {
                out.push_str(&" ".repeat(widths[i].saturating_sub(cell.len())));
            }
            out.push('│');
        }
        out.push('\n');
    }
    out.push_str(&border("└", "┴", "┘"));
    out.push('\n');
    out
}

/// oneinit self-update — update OneInit itself to the latest release
pub async fn run_self_update(formatter: &OutputFormatter) {
    match crate::core::self_update::run_self_update(formatter).await {
        Ok(_) => {}
        Err(e) => formatter.error(&e),
    }
}

/// oneinit list versions <recipe> — list available versions for a family
pub async fn run_list_versions(formatter: &OutputFormatter, recipe: &str) {
    use crate::core::version;

    if !version::is_versioned(recipe) {
        formatter.output(
            &format!(
                "[ERROR] '{recipe}' 不是可版本化配方（支持: python / node / go / java / rust）"
            ),
            None::<serde_json::Value>,
        );
        return;
    }
    match version::list(recipe) {
        Ok(versions) => {
            let text = format!(
                "{} 可用版本 ({}):\n  {}",
                recipe,
                versions.len(),
                versions
                    .iter()
                    .map(|v| format!("  - {}", v))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            formatter.output(
                &text,
                Some(serde_json::json!({
                    "status": "success",
                    "action": "list_versions",
                    "recipe": recipe,
                    "versions": versions,
                })),
            );
        }
        Err(e) => formatter.error(&e),
    }
}

/// oneinit info <package[@version]> — show resolved version details
pub async fn run_info(formatter: &OutputFormatter, package: &str) {
    use crate::core::version;

    let (name, spec) = parse_package_spec(package);
    if !version::is_versioned(&name) {
        // fall back to showing the recipe info via search
        formatter.output(
            &format!("[INFO] '{name}' 不是可版本化配方 — 用 `oneinit search {name}` 查看"),
            None::<serde_json::Value>,
        );
        return;
    }

    match version::resolve(&name, spec.as_deref()) {
        Ok(resolved) => {
            let default_v = version::resolve(&name, None).unwrap_or_default();
            let lts_v = version::resolve(&name, Some("lts")).unwrap_or_default();
            formatter.output(
                &format!(
                    "[INFO] {name}@{}\n  解析结果: {}（{}）\n  默认版本: {}\n  LTS 版本: {}\n  安装: `oneinit install {}@{}`",
                    spec.as_deref().unwrap_or("default"),
                    resolved,
                    if resolved == default_v { "默认" } else { "指定" },
                    default_v,
                    lts_v,
                    name,
                    resolved
                ),
                Some(serde_json::json!({
                    "status": "success",
                    "action": "info",
                    "package": name,
                    "requested": spec,
                    "resolved": resolved,
                    "default": default_v,
                    "lts": lts_v,
                })),
            );
        }
        Err(e) => formatter.error(&e),
    }
}

#[cfg(test)]
mod tests {
    use super::{detect_project_toolchains, render_table};
    use std::path::Path;

    #[test]
    fn test_detect_project_toolchains() {
        let dir = std::env::temp_dir().join(format!("oneinit-proj-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("package.json"), "{}").unwrap();
        std::fs::write(dir.join("requirements.txt"), "requests\n").unwrap();
        std::fs::write(dir.join("go.mod"), "module demo\n").unwrap();

        let detected = detect_project_toolchains(Path::new(&dir));
        let recipes: Vec<&str> = detected.iter().map(|(r, _)| r.as_str()).collect();
        assert!(recipes.contains(&"node20"));
        assert!(recipes.contains(&"python3.11"));
        assert!(recipes.contains(&"go"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_detect_project_toolchains_empty() {
        let dir = std::env::temp_dir().join(format!("oneinit-proj-empty-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let detected = detect_project_toolchains(Path::new(&dir));
        assert!(detected.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_render_table() {
        let rows = vec![
            vec!["python3.11".to_string(), "3.11.9".to_string()],
            vec!["node20".to_string(), "20.18.1".to_string()],
        ];
        let t = render_table(&["Name", "Version"], &rows, None);
        assert!(t.contains("python3.11"));
        assert!(t.contains("┌"));
        assert!(t.contains("└"));
        // column alignment: node20 row aligns under header
        assert!(t.contains("node20   "));
    }

    #[test]
    fn test_render_table_empty() {
        let t = render_table(&["A"], &[], Some("nothing"));
        assert_eq!(t, "nothing");
    }
}
