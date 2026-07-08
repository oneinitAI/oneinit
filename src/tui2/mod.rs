// TUI 模块入口 — 交互式终端界面
//
// 架构（参见 tui.md）：
//   入口层 → 应用层(App+屏幕) → 组件层 → 状态层 → 基础设施层
//
// 异步事件驱动：EventStream + mpsc 通道，事件循环独立于渲染。
// 退出TUI执行模式：Enter 触发安装/卸载时，暂停渲染→执行→按回车返回。

pub mod app;
pub mod backend;
pub mod event;
pub mod screens;
pub mod state;

use std::time::Duration;

use crate::output::OutputFormatter;

/// 启动 TUI 主循环
pub async fn run_tui(_formatter: &OutputFormatter) -> std::io::Result<()> {
    // 安装 panic hook（确保崩溃时恢复终端）
    backend::setup_panic_hook();

    // 初始化终端
    let mut terminal = backend::init()?;

    // 创建应用 + 启动事件循环
    let (mut app, mut event_rx) = app::App::new();

    // 主循环
    let result = main_loop(&mut terminal, &mut app, &mut event_rx).await;

    // 恢复终端
    backend::restore(&mut terminal)?;

    result
}

/// TUI 主循环
async fn main_loop(
    terminal: &mut backend::Tui,
    app: &mut app::App,
    event_rx: &mut tokio::sync::mpsc::UnboundedReceiver<event::AppEvent>,
) -> std::io::Result<()> {
    loop {
        // 渲染当前帧
        terminal.draw(|frame| app.render(frame))?;

        // 非阻塞检查是否有事件
        match tokio::time::timeout(Duration::from_millis(200), event_rx.recv()).await {
            Ok(Some(event)) => {
                // 处理事件，可能返回操作目标
                if let Some(target) = app.handle_event(event) {
                    // 执行操作（暂停 TUI）
                    if let Err(e) = app::execute_action(terminal, &mut app.state, target).await {
                        app.state.message = Some(format!("❌ 操作失败: {}", e));
                        // 尝试恢复终端
                        *terminal = backend::init()?;
                    }
                }
            }
            Ok(None) => {
                // 通道关闭（事件循环任务结束）
                break;
            }
            Err(_) => {
                // 超时，继续渲染（处理 should_quit 检查）
            }
        }

        // 检查是否应退出
        if app.state.should_quit {
            break;
        }
    }

    Ok(())
}
