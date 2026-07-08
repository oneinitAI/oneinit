// 应用生命周期 — 事件分发 + 安装/卸载执行
//
// 退出TUI执行模式：当用户按 Enter 触发操作时，暂停 TUI 渲染，
// 调用现有 backend（recipe::install/uninstall），进度条直接输出到终端，
// 完成后按回车返回 TUI 并刷新状态。

use std::io::{self, Write};

use crossterm::event::KeyCode;
use ratatui::Frame;
use tokio::sync::mpsc::UnboundedSender;

use crate::output::OutputFormatter;

use super::backend::{self, Tui};
use super::event::AppEvent;
use super::screens;
use super::state::{AppState, Screen, Target};

/// 应用实例
pub struct App {
    pub state: AppState,
    pub event_tx: UnboundedSender<AppEvent>,
}

impl App {
    /// 创建应用（初始化状态 + 启动事件循环）
    pub fn new() -> (Self, tokio::sync::mpsc::UnboundedReceiver<AppEvent>) {
        let (event_tx, event_rx) = super::event::start();
        let app = App {
            state: AppState::new(),
            event_tx,
        };
        (app, event_rx)
    }

    /// 渲染当前帧
    pub fn render(&mut self, frame: &mut Frame) {
        screens::draw(frame, &mut self.state);
    }

    /// 处理事件，返回可能需要执行的操作目标
    pub fn handle_event(&mut self, event: AppEvent) -> Option<Target> {
        // 帮助弹窗优先处理：任意键关闭
        if self.state.current_screen == Screen::Help {
            if let AppEvent::Key(_) = event {
                self.state.current_screen = Screen::PackageList;
                return None;
            }
        }

        match event {
            AppEvent::Quit => {
                self.state.should_quit = true;
            }
            AppEvent::Key(key) => {
                return self.handle_key(key);
            }
            AppEvent::Tick | AppEvent::Resize(_, _) => {}
            AppEvent::InstallProgress(pkg, pct) => {
                self.state.install_progress.insert(pkg, pct);
            }
            AppEvent::InstallComplete(pkg, success) => {
                if success {
                    self.state.message = Some(format!("✅ {} 安装成功", pkg));
                } else {
                    self.state.message = Some(format!("❌ {} 安装失败", pkg));
                }
                self.state.installing = None;
                self.state.refresh();
            }
        }

        None
    }

    /// 处理按键，返回操作目标（仅 Enter 时）
    fn handle_key(&mut self, key: KeyCode) -> Option<Target> {
        if self.state.current_screen == Screen::Install {
            return None;
        }

        match key {
            KeyCode::Tab => {
                self.state.toggle_pane();
                self.state.message = None;
            }
            KeyCode::Up => {
                self.state.move_cursor(-1);
                self.state.message = None;
            }
            KeyCode::Down => {
                self.state.move_cursor(1);
                self.state.message = None;
            }
            KeyCode::Char('?') => {
                self.state.current_screen = Screen::Help;
            }
            KeyCode::Char('r') => {
                self.state.refresh();
                self.state.message = Some("已刷新".to_string());
            }
            KeyCode::Enter => {
                // 返回操作目标，由主循环执行（需要 async + 终端暂停）
                return self.state.current_target();
            }
            _ => {}
        }

        None
    }
}

/// 执行操作（在 TUI 外运行）
///
/// 暂停 TUI → 调用 recipe::install/uninstall → 等待回车 → 返回
pub async fn execute_action(
    terminal: &mut Tui,
    state: &mut AppState,
    target: Target,
) -> io::Result<()> {
    // 1. 恢复终端（退出 TUI 画面）
    backend::restore(terminal)?;

    // 2. 执行操作
    let formatter = OutputFormatter::new(false);
    let result_msg = match target {
        Target::Install(recipe_name) => {
            match crate::core::recipe::resolve(&recipe_name) {
                Some(recipe) => {
                    match crate::core::recipe::install(&recipe, &formatter).await {
                        Ok(()) => format!("✅ {} 安装成功", recipe.display_name),
                        Err(e) => format!("❌ 安装失败: {}", e),
                    }
                }
                None => format!("❌ 未找到配方: {}", recipe_name),
            }
        }
        Target::Uninstall(package_name) => {
            match crate::core::recipe::uninstall(&package_name, &formatter).await {
                Ok(()) => format!("✅ {} 卸载完成", package_name),
                Err(e) => format!("❌ 卸载失败: {}", e),
            }
        }
    };

    println!("\n{}", result_msg);
    print!("\n按回车键返回 TUI...");
    io::stdout().flush()?;
    wait_for_enter()?;

    // 3. 重新进入 TUI
    *terminal = backend::init()?;

    // 4. 更新状态
    state.message = Some(result_msg);
    state.refresh();

    Ok(())
}

/// 等待用户按回车（普通终端模式）
fn wait_for_enter() -> io::Result<()> {
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(())
}
