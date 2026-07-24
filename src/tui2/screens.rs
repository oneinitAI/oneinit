// 渲染层 — 根据 current_screen 渲染不同界面
//
// 布局：标题栏 + 内容区 + 底部帮助栏
// 主屏幕内容区分为左右两个面板（已安装 / 可安装）

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph};

use super::state::{AppState, Pane, Screen};

/// 主渲染入口
pub fn draw(frame: &mut Frame, state: &mut AppState) {
    let area = frame.area();

    // 垂直布局：标题栏(3) / 内容区(弹性) / 底部帮助栏(3)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    draw_title(frame, chunks[0]);
    draw_content(frame, chunks[1], state);
    draw_footer(frame, chunks[2], state);
}

/// 标题栏
fn draw_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("OneInit — 一条命令，初始化整台电脑")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL).title(" OneInit "));
    frame.render_widget(title, area);
}

/// 内容区（根据当前屏幕路由）
fn draw_content(frame: &mut Frame, area: Rect, state: &mut AppState) {
    match state.current_screen {
        Screen::PackageList => draw_main_screen(frame, area, state),
        Screen::Install => draw_install_screen(frame, area, state),
        Screen::PackageInfo => draw_main_screen(frame, area, state),
        Screen::Capture => draw_capture_screen(frame, area, state),
        Screen::Help => {
            draw_main_screen(frame, area, state);
            draw_help_popup(frame, frame.area());
        }
    }
}

/// 主屏幕：左右双面板
fn draw_main_screen(frame: &mut Frame, area: Rect, state: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    draw_installed_pane(frame, chunks[0], state);
    draw_available_pane(frame, chunks[1], state);
}

/// 已安装面板
fn draw_installed_pane(frame: &mut Frame, area: Rect, state: &mut AppState) {
    let is_active =
        state.active_pane == Pane::Installed && state.current_screen == Screen::PackageList;
    let border_color = if is_active {
        Color::Green
    } else {
        Color::DarkGray
    };

    let title = format!(" 📦 已安装 ({}) ", state.installed.len());
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(border_color));

    let items: Vec<ListItem> = state
        .installed
        .iter()
        .map(|r| {
            let version = r.version.as_deref().unwrap_or("?");
            ListItem::new(Line::from(vec![
                Span::styled(&r.name, Style::default().fg(Color::Yellow)),
                Span::raw(" "),
                Span::styled(
                    format!("v{}", version),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let mut list_state = ListState::default();
    if is_active && !state.installed.is_empty() {
        list_state.select(Some(state.selected_installed));
    }

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">");

    frame.render_stateful_widget(list, area, &mut list_state);
}

/// 可安装面板
fn draw_available_pane(frame: &mut Frame, area: Rect, state: &mut AppState) {
    let is_active =
        state.active_pane == Pane::Available && state.current_screen == Screen::PackageList;
    let border_color = if is_active {
        Color::Green
    } else {
        Color::DarkGray
    };

    let title = format!(" 🔍 可安装 ({}) ", state.available.len());
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(border_color));

    let items: Vec<ListItem> = state
        .available
        .iter()
        .map(|r| {
            let is_installed = state.installed.iter().any(|i| i.name == r.name);
            let prefix = if is_installed { "v " } else { "  " };
            ListItem::new(Line::from(vec![
                Span::raw(prefix),
                Span::styled(
                    &r.display_name,
                    if is_installed {
                        Style::default().fg(Color::DarkGray)
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
            ]))
        })
        .collect();

    let mut list_state = ListState::default();
    if is_active && !state.available.is_empty() {
        list_state.select(Some(state.selected_available));
    }

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">");

    frame.render_stateful_widget(list, area, &mut list_state);
}

/// 安装屏幕：显示进度条
fn draw_install_screen(frame: &mut Frame, area: Rect, state: &AppState) {
    let pkg = state.installing.as_deref().unwrap_or("未知包");

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    // 进度条
    let progress = state.install_progress.get(pkg).copied().unwrap_or(0);
    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" 正在安装: {} ", pkg)),
        )
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent(progress.into());
    frame.render_widget(gauge, chunks[0]);

    // 状态消息
    let msg = state.message.as_deref().unwrap_or("正在下载和解压...");
    let info = Paragraph::new(msg).block(Block::default().borders(Borders::ALL).title(" 状态 "));
    frame.render_widget(info, chunks[1]);
}

/// Capture 屏幕：显示环境检测结果
fn draw_capture_screen(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        "环境捕获结果",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    if let Some(ref results) = state.capture_result {
        for (name, version, path) in results {
            let status_tag = if version.is_some() { "[OK]" } else { "[--]" };
            let ver_str = version.as_deref().unwrap_or("未检测到");
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    status_tag,
                    Style::default().fg(if version.is_some() {
                        Color::Green
                    } else {
                        Color::DarkGray
                    }),
                ),
                Span::raw(" "),
                Span::styled(name, Style::default().fg(Color::Yellow)),
                Span::raw(" "),
                Span::styled(ver_str, Style::default().fg(Color::White)),
            ]));
            lines.push(Line::from(Span::styled(
                format!("       {}", path),
                Style::default().fg(Color::DarkGray),
            )));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "正在扫描...",
            Style::default().fg(Color::DarkGray),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[Esc]/[q] 返回主菜单",
        Style::default().fg(Color::DarkGray),
    )));

    let content =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Capture "));
    frame.render_widget(content, area);
}

/// 底部帮助栏 + 状态消息
fn draw_footer(frame: &mut Frame, area: Rect, state: &AppState) {
    let help_text = "[Tab]切换 [↑↓]移动 [Enter]安装/卸载 [c]捕获环境 [?]帮助 [q]退出";
    let message = state.message.as_deref().unwrap_or("");

    let content = format!("{}\n{}", help_text, message);
    let footer = Paragraph::new(content)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title(" 帮助 "));
    frame.render_widget(footer, area);
}

/// 帮助弹窗（居中覆盖）
fn draw_help_popup(frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(60, 50, area);

    let help = Paragraph::new(vec![
        Line::from(Span::styled(
            "OneInit 帮助",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  Tab     切换面板（已安装 ↔ 可安装）"),
        Line::from("  ↑ / ↓   上下移动选择"),
        Line::from("  Enter   安装（可安装面板）或卸载（已安装面板）"),
        Line::from("  ?       显示/隐藏帮助"),
        Line::from("  q / Esc 退出"),
        Line::from(""),
        Line::from(Span::styled(
            "按任意键关闭帮助",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" 帮助 ")
            .style(Style::default().fg(Color::White)),
    );

    frame.render_widget(help, popup_area);
}

/// 计算居中矩形（用于弹窗）
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
