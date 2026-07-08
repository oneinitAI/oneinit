// TUI 模块入口 — 交互式终端界面
//
// 架构（参见 tui.md）：
//   入口层 → 应用层(App+屏幕) → 组件层 → 状态层 → 基础设施层
//
// 异步事件驱动：EventStream + mpsc 通道，事件循环独立于渲染。
// 退出TUI执行模式：Enter 触发安装/卸载时，停止事件循环 → 恢复终端 →
// 执行操作 → 按任意键 → 重新进入 TUI → 重启事件循环。

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

    // 创建应用（不启动事件循环，稍后启动）
    let mut app_state = state::AppState::new();

    // 主循环
    let result = main_loop(&mut terminal, &mut app_state).await;

    // 恢复终端
    backend::restore(&mut terminal)?;

    result
}

/// TUI 主循环
async fn main_loop(
    terminal: &mut backend::Tui,
    app_state: &mut state::AppState,
) -> std::io::Result<()> {
    loop {
        // 启动事件循环（每次渲染循环开始时确保有一个活跃的事件循环）
        let (event_tx, mut event_rx) = event::start();

        // 渲染并处理事件
        loop {
            // 渲染当前帧
            terminal.draw(|frame| screens::draw(frame, app_state))?;

            // 等待事件（200ms 超时后继续渲染）
            match tokio::time::timeout(Duration::from_millis(200), event_rx.recv()).await {
                Ok(Some(ev)) => {
                    // 处理事件，可能返回操作目标
                    if let Some(target) = handle_event(app_state, ev) {
                        // 停止事件循环：drop sender 让事件循环任务退出
                        drop(event_tx);
                        // drain 剩余事件
                        while event_rx.try_recv().is_ok() {}

                        // 执行操作（恢复终端 → 安装/卸载 → 按任意键 → 重新进入 TUI）
                        if let Err(e) =
                            app::execute_action(terminal, app_state, target).await
                        {
                            app_state.message = Some(format!("❌ 操作失败: {}", e));
                            *terminal = backend::init()?;
                        }
                        // 跳出内层循环，重新启动事件循环
                        break;
                    }
                }
                Ok(None) => {
                    // 通道关闭（事件循环任务结束）
                    return Ok(());
                }
                Err(_) => {
                    // 超时，继续渲染
                }
            }

            if app_state.should_quit {
                drop(event_tx);
                return Ok(());
            }
        }
    }
}

/// 处理单个事件
fn handle_event(state: &mut state::AppState, ev: event::AppEvent) -> Option<state::Target> {
    use event::AppEvent;
    use state::{Screen, Target};

    // 帮助弹窗优先：任意键关闭
    if state.current_screen == Screen::Help {
        if let AppEvent::Key(_) = ev {
            state.current_screen = Screen::PackageList;
            return None;
        }
    }

    match ev {
        AppEvent::Quit => {
            state.should_quit = true;
        }
        AppEvent::Key(key) => {
            return handle_key(state, key);
        }
        AppEvent::Tick | AppEvent::Resize(_, _) => {}
        AppEvent::InstallProgress(pkg, pct) => {
            state.install_progress.insert(pkg, pct);
        }
        AppEvent::InstallComplete(pkg, success) => {
            if success {
                state.message = Some(format!("✅ {} 安装成功", pkg));
            } else {
                state.message = Some(format!("❌ {} 安装失败", pkg));
            }
            state.installing = None;
            state.refresh();
        }
    }

    None
}

/// 处理按键
fn handle_key(state: &mut state::AppState, key: crossterm::event::KeyCode) -> Option<state::Target> {
    use crossterm::event::KeyCode;

    if state.current_screen == state::Screen::Install {
        return None;
    }

    match key {
        KeyCode::Tab => {
            state.toggle_pane();
            state.message = None;
        }
        KeyCode::Up => {
            state.move_cursor(-1);
            state.message = None;
        }
        KeyCode::Down => {
            state.move_cursor(1);
            state.message = None;
        }
        KeyCode::Char('?') => {
            state.current_screen = state::Screen::Help;
        }
        KeyCode::Char('r') => {
            state.refresh();
            state.message = Some("已刷新".to_string());
        }
        KeyCode::Enter => {
            return state.current_target();
        }
        _ => {}
    }

    None
}
