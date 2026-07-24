// 操作执行 — 安装/卸载（在 TUI 外运行）
//
// 退出TUI执行模式流程：
//   1. 主循环检测到 Enter → 返回 Target
//   2. 主循环 drop event_tx（停止事件循环任务，释放 EventStream 对 stdin 的占用）
//   3. 调用 execute_action：恢复终端 → 执行安装/卸载 → 按任意键 → 重新进入 TUI
//   4. 主循环重启事件循环
//
// 关键：execute_action 必须在事件循环停止后调用，否则 EventStream 会
// 和 stdin 读取竞争，导致"按回车无反应"。

use std::io::{self, Read, Write};

use crate::output::OutputFormatter;

use super::backend::{self, Tui};
use super::state::{AppState, Target};

/// 执行操作（在 TUI 外运行）
pub async fn execute_action(
    terminal: &mut Tui,
    state: &mut AppState,
    target: Target,
) -> io::Result<()> {
    // 1. 恢复终端（退出 raw mode + 备用屏幕）
    backend::restore(terminal)?;

    // 2. 执行操作（OutputFormatter 输出到普通终端）
    let formatter = OutputFormatter::new(false);
    let result_msg = match target {
        Target::Install(recipe_name) => match crate::core::recipe::resolve(&recipe_name) {
            Some(recipe) => match crate::core::recipe::install(&recipe, &formatter).await {
                Ok(()) => format!("[OK] {} 安装成功", recipe.display_name),
                Err(e) => format!("[ERROR] 安装失败: {}", e),
            },
            None => format!("[ERROR] 未找到配方: {}", recipe_name),
        },
        Target::Uninstall(package_name) => {
            match crate::core::recipe::uninstall(&package_name, &formatter).await {
                Ok(()) => format!("[OK] {} 卸载完成", package_name),
                Err(e) => format!("[ERROR] 卸载失败: {}", e),
            }
        }
    };

    println!("\n{}", result_msg);
    println!("\n按任意键返回 TUI...");
    io::stdout().flush()?;

    // 3. 等待按键（此时 raw mode 已关闭，事件循环已停止）
    //    逐字节读取，读到一个字节即返回
    wait_for_any_key();

    // 4. 重新进入 TUI
    *terminal = backend::init()?;

    // 5. 更新状态
    state.message = Some(result_msg);
    state.refresh();

    Ok(())
}

/// 等待任意按键（恢复终端后使用）
///
/// 此时 raw mode 已关闭，事件循环已停止，直接用 std::io 读取。
fn wait_for_any_key() {
    let mut buf = [0u8; 1];
    let _ = io::stdin().read(&mut buf);
}
