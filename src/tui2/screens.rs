// Render layer — dispatch by current_screen
//
// 布局：Title bar + 内容区 + 底部帮助栏
// Main screen split into two panes (installed / available)

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph};

use super::state::{AppState, Pane, Screen};

/// Main render entry
pub fn draw(frame: &mut Frame, state: &mut AppState) {
    let area = frame.area();

    // Vertical layout: title(3) / content(flex) / footer(3)
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

/// Title bar
fn draw_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("OneInit — One command to init your dev machine")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL).title(" OneInit "));
    frame.render_widget(title, area);
}

/// Content area (routed by current screen)
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

/// Main screen: dual panes
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

    let title = format!(" Installed ({}) ", state.installed.len());
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

    let title = format!(" Available ({}) ", state.available.len());
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

/// 安装屏幕：显示Progress bar
fn draw_install_screen(frame: &mut Frame, area: Rect, state: &AppState) {
    let pkg = state.installing.as_deref().unwrap_or("Unknown");

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    // Progress bar
    let progress = state.install_progress.get(pkg).copied().unwrap_or(0);
    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Installing: {} ", pkg)),
        )
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent(progress.into());
    frame.render_widget(gauge, chunks[0]);

    // Status message
    let msg = state
        .message
        .as_deref()
        .unwrap_or("Downloading and extracting...");
    let info = Paragraph::new(msg).block(Block::default().borders(Borders::ALL).title(" Status "));
    frame.render_widget(info, chunks[1]);
}

/// Capture 屏幕：显示环境检测结果
fn draw_capture_screen(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        "Capture Results",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    if let Some(ref results) = state.capture_result {
        for (name, version, path) in results {
            let status_tag = if version.is_some() { "[OK]" } else { "[--]" };
            let ver_str = version.as_deref().unwrap_or("Not detected");
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
            "Scanning...",
            Style::default().fg(Color::DarkGray),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[Esc]/[q] Back to main",
        Style::default().fg(Color::DarkGray),
    )));

    let content =
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Capture "));
    frame.render_widget(content, area);
}

/// Footer + status message
fn draw_footer(frame: &mut Frame, area: Rect, state: &AppState) {
    let help_text = "[Tab]Pane [Arrows]Move [Enter]Action [c]Capture [?]Help [q]Quit";
    let message = state.message.as_deref().unwrap_or("");

    let content = format!("{}\n{}", help_text, message);
    let footer = Paragraph::new(content)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title(" Help "));
    frame.render_widget(footer, area);
}

/// 帮助弹窗（居中覆盖）
fn draw_help_popup(frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(60, 50, area);

    let help = Paragraph::new(vec![
        Line::from(Span::styled(
            "OneInit Help",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  Tab     Switch pane (installed / available)"),
        Line::from("  Arrow keys  Move selection"),
        Line::from("  Enter   Install (available pane) or Uninstall (installed pane)"),
        Line::from("  ?       Show/hide help"),
        Line::from("  q / Esc Quit"),
        Line::from(""),
        Line::from(Span::styled(
            "按任意键关闭帮助",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Help ")
            .style(Style::default().fg(Color::White)),
    );

    frame.render_widget(help, popup_area);
}

/// 计算Centered rect (for popups)
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
