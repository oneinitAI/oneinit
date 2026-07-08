// 终端后端 — 初始化/恢复 + 能力检测
//
// 跨平台适配核心：Windows/Linux/macOS 行为一致。

use std::io::{self, Stdout};

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

/// TUI 终端类型别名
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// 初始化终端（适配 Windows / Linux / macOS）
pub fn init() -> io::Result<Tui> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

/// 恢复终端状态（退出时调用）
pub fn restore(terminal: &mut Tui) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

/// 检测终端是否支持下划线颜色
///
/// 旧版 conhost（传统 CMD）不支持 underline-color，会报错。
/// Windows Terminal (WT_SESSION) 和 VS Code (TERM_PROGRAM=vscode) 支持。
pub fn supports_underline_color() -> bool {
    if cfg!(windows) {
        std::env::var("WT_SESSION").is_ok()
            || std::env::var("TERM_PROGRAM").map_or(false, |v| v == "vscode")
    } else {
        true
    }
}

/// 安装 panic hook — 确保 panic 时恢复终端状态
///
/// TUI 应用崩溃若不恢复 raw mode，终端会变得不可用。
pub fn setup_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // 尽力恢复终端
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);
        // 调用默认 hook 打印 panic 信息
        default_hook(panic_info);
    }));
}
