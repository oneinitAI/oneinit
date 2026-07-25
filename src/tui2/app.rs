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
        Target::Install(recipe_name) => match crate::core::recipe::resolve(&recipe_name) {
            Some(recipe) => match crate::core::recipe::install(&recipe, &formatter).await {
                Ok(()) => format!("[OK] {} installation complete", recipe.display_name),
                Err(e) => format!("[ERROR] installation failed: {}", e),
            },
            None => format!("[ERROR] recipe not found: {}", recipe_name),
        },
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
