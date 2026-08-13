// Action execution — install/uninstall (outside TUI)
//
// Exit-TUI execution flow:
//   1. main loop detects Enter -> returns Target
//   2. main loop drops event_tx (stops event loop, frees stdin)
//   3. call execute_action: restore terminal -> install/uninstall -> press any key -> re-enter TUI
//   4. main loop restarts event loop
//
// CRITICAL: execute_action must be called after event loop stops, otherwise EventStream
// races with stdin reads, causing unresponsive prompt.

use std::io::{self, Read, Write};

use crate::output::OutputFormatter;

use super::backend::{self, Tui};
use super::state::{AppState, Target};

/// Execute action (outside TUI)
pub async fn execute_action(
    terminal: &mut Tui,
    state: &mut AppState,
    target: Target,
) -> io::Result<()> {
    // 1. restore terminal (exit raw mode + alternate screen)
    backend::restore(terminal)?;

    // 2. execute (OutputFormatter writes to normal terminal)
    let formatter = OutputFormatter::new(false);
    let result_msg = match target {
        Target::Install(recipe_name) => install_from_tui(&recipe_name, &formatter).await,
        Target::Uninstall(package_name) => {
            match crate::core::recipe::uninstall(&package_name, &formatter).await {
                Ok(()) => format!("[OK] {} uninstalled", package_name),
                Err(e) => format!("[ERROR] uninstall failed: {}", e),
            }
        }
    };

    println!("\n{}", result_msg);
    println!("\nPress any key to return to TUI...");
    io::stdout().flush()?;

    // 3. wait for keypress (raw mode off, event loop stopped)
    //    read one byte and return
    wait_for_any_key();

    // 4. re-enter TUI
    *terminal = backend::init()?;

    // 5. update state
    state.message = Some(result_msg);
    state.refresh();

    Ok(())
}

/// Wait for any key (after terminal restore)
///
/// raw mode disabled, event loop stopped, direct std::io read.
fn wait_for_any_key() {
    let mut buf = [0u8; 1];
    let _ = io::stdin().read(&mut buf);
}

/// TUI 安装：三级解析（内置 → 本地社区 → 远程注册表）
async fn install_from_tui(name: &str, formatter: &OutputFormatter) -> String {
    use crate::core::{community_recipe, recipe, registry};

    // 1. 内置
    if let Some(r) = recipe::resolve(name) {
        return match recipe::install(&r, formatter).await {
            Ok(()) => format!("[OK] {} installation complete", r.display_name),
            Err(e) => format!("[ERROR] installation failed: {}", e),
        };
    }

    // 2. 本地社区
    if let Some(r) = community_recipe::resolve(name) {
        // TUI 默认拒绝执行类配方（安全），提示用 CLI --allow-exec
        return match community_recipe::install(&r, formatter, false).await {
            Ok(()) => format!("[OK] {} installation complete", r.name),
            Err(e) => format!(
                "[ERROR] installation failed: {}\n       Hint: 含命令的配方请在 CLI 用 `oneinit install --allow-exec {}`",
                e, name
            ),
        };
    }

    // 3. 远程注册表（先确保有缓存索引，缺失则拉取）
    if registry::load_cached_index().is_none() {
        formatter.output(
            "[REMOTE] 拉取配方索引...",
            Some(serde_json::Value::Null),
        );
        if let Err(e) = registry::fetch_index().await {
            return format!("[ERROR] index refresh failed: {}", e);
        }
    }

    if let Some(entry) = registry::resolve(name) {
        let target_version = entry.latest.clone();
        formatter.output(
            &format!("[REMOTE] 拉取 {} v{}...", name, target_version),
            Some(serde_json::Value::Null),
        );
        return match registry::fetch_recipe(name, &target_version).await {
            Ok(r) => match community_recipe::install(&r, formatter, false).await {
                Ok(()) => format!("[OK] {} installation complete", r.name),
                Err(e) => format!(
                    "[ERROR] installation failed: {}\n       Hint: 含命令的配方请在 CLI 用 `oneinit install --allow-exec {}`",
                    e, name
                ),
            },
            Err(e) => format!("[ERROR] remote fetch failed: {}", e),
        };
    }

    format!("[ERROR] recipe not found: {}", name)
}
