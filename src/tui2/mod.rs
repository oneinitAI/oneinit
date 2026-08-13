// TUI module entry — interactive terminal UI
//
// 架构（参见 tui.md）：
//   入口层 → 应用层(App+屏幕) → 组件层 → 状态层 → 基础设施层
//
// Async event-driven: EventStream + mpsc, event loop decoupled from render.
// QuitTUI执行模式：Enter 触发安装/卸载时，停止事件循环 → 恢复终端 →
// 执行操作 → 按任意键 → re-enter TUI → 重启事件循环。

pub mod app;
pub mod backend;
pub mod event;
pub mod screens;
pub mod state;

use std::time::Duration;

use crate::output::OutputFormatter;

/// Start TUI main loop
pub async fn run_tui(_formatter: &OutputFormatter) -> std::io::Result<()> {
    // 安装 panic hook（确保崩溃时恢复终端）
    backend::setup_panic_hook();

    // 初始化终端
    let mut terminal = backend::init()?;

    // 启动时自动拉取配方索引（缓存缺失或过期时，失败静默）
    {
        use crate::core::registry;
        if registry::load_cached_index().is_none() || registry::is_index_stale(24) {
            let _ = registry::fetch_index().await;
        }
    }

    // Create app (event loop starts later)
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
        // Start event loop (ensure active on each render cycle start)
        let (event_tx, mut event_rx) = event::start();

        // 渲染并处理事件
        loop {
            // 渲染当前帧
            terminal.draw(|frame| screens::draw(frame, app_state))?;

            // Wait for event (200ms timeout, then re-render)
            match tokio::time::timeout(Duration::from_millis(200), event_rx.recv()).await {
                Ok(Some(ev)) => {
                    // 处理事件，可能返回操作目标
                    if let Some(target) = handle_event(app_state, ev) {
                        // 停止事件循环：drop sender 让事件循环任务Quit
                        drop(event_tx);
                        // drain 剩余事件
                        while event_rx.try_recv().is_ok() {}

                        // 执行操作（恢复终端 → 安装/卸载 → 按任意键 → re-enter TUI）
                        if let Err(e) = app::execute_action(terminal, app_state, target).await {
                            app_state.message = Some(format!("[ERROR] operation failed: {}", e));
                            *terminal = backend::init()?;
                        }
                        // Break inner loop, restart event loop
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
    use state::Screen;

    // 帮助弹窗优先：任意键关闭
    if state.current_screen == Screen::Help
        && let AppEvent::Key(_) = ev
    {
        state.current_screen = Screen::PackageList;
        return None;
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
                state.message = Some(format!("[OK] {} installation complete", pkg));
            } else {
                state.message = Some(format!("[ERROR] {} installation failed", pkg));
            }
            state.installing = None;
            state.refresh();
        }
    }

    None
}

/// 处理按键
fn handle_key(
    state: &mut state::AppState,
    key: crossterm::event::KeyCode,
) -> Option<state::Target> {
    use crossterm::event::KeyCode;

    // Capture screen: Esc to return
    if state.current_screen == state::Screen::Capture {
        if key == KeyCode::Esc || key == KeyCode::Enter || key == KeyCode::Char('q') {
            state.current_screen = state::Screen::PackageList;
        }
        return None;
    }

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
        KeyCode::Char('c') => {
            // Run capture
            state.message = Some("[SCAN] 正在扫描...".to_string());
            run_capture_to_state(state);
        }
        KeyCode::Char('r') => {
            state.refresh();
            state.message = Some("Refreshed".to_string());
        }
        KeyCode::Enter => {
            return state.current_target();
        }
        _ => {}
    }

    None
}

/// Run capture并写入 state
fn run_capture_to_state(state: &mut state::AppState) {
    let mut scheduler = crate::core::capture::detector::DetectorScheduler::new();
    scheduler.register_defaults();
    let results = scheduler.scan();

    let mut detected = Vec::new();
    for (name, opt_env) in &results {
        if let Some(env) = opt_env {
            detected.push((
                env.name.clone(),
                Some(env.version.clone()),
                env.install_path.clone(),
            ));
        } else {
            detected.push((name.clone(), None, "Not detected".to_string()));
        }
    }

    state.capture_result = Some(detected);
    state.current_screen = state::Screen::Capture;
}
