以下是 OneInit TUI 的完整架构设计，重点覆盖 **Ratatui 组件化架构**、**多屏幕路由** 以及 **VS Code / PowerShell / CMD / Windows Terminal 的跨终端适配**。

---

## 一、总体架构

OneInit TUI 采用 **分层架构 + 组件化设计**，核心原则是 **“关注点分离”** 和 **“可测试性”**。

```
┌─────────────────────────────────────────────────────────────────┐
│                        入口层 (main.rs)                        │
│             参数解析 → 初始化 → 启动 TUI 主循环                  │
├─────────────────────────────────────────────────────────────────┤
│                     应用层 (App + 屏幕)                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐ │
│  │ 主屏幕        │  │ 安装屏幕      │  │ 详情屏幕 / 配置屏幕   │ │
│  │ (PackageList) │  │ (Install)    │  │ (Info/Config)        │ │
│  └──────────────┘  └──────────────┘  └──────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                      组件层 (Components)                       │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────────┐ │
│  │ 列表组件  │ │ 搜索框   │ │ 进度条   │ │ 状态栏 / 帮助栏   │ │
│  │ (List)   │ │ (Input)  │ │ (Gauge)  │ │ (Status/Footer)  │ │
│  └──────────┘ └──────────┘ └──────────┘ └──────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                     状态层 (State)                              │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  AppState (全局状态: 当前屏幕、包列表、安装队列、配置)     │  │
│  └──────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                     基础设施层 (Infrastructure)                 │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌───────────┐ │
│  │ 事件循环    │ │ 终端后端   │ │ 核心引擎   │ │ 日志系统  │ │
│  │ (EventLoop)│ │ (Backend)  │ │ (Core)    │ │ (Logger)  │ │
│  └────────────┘ └────────────┘ └────────────┘ └───────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

---

## 二、核心依赖

```toml
[dependencies]
# TUI 核心
ratatui = { version = "0.29", features = ["crossterm"] }

# 终端后端（跨平台）
crossterm = { version = "0.28", features = ["event-stream"] }

# 异步运行时
tokio = { version = "1", features = ["full"] }

# 事件通道
tokio-stream = "0.1"

# 日志（调试专用，生产环境可关闭）
tracing = "0.1"
tracing-subscriber = "0.3"

# 序列化（配置读取）
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"

# OneInit 核心库（已有的 MVP 逻辑）
oneinit-core = { path = "../core" }
```

**选择 `crossterm` 作为后端的原因**：它是 Ratatui 官方推荐的跨平台后端，同时支持 Windows、macOS、Linux，且 Ratatui 默认启用 crossterm。

---

## 三、核心模块详解

### 3.1 终端后端初始化（跨平台适配核心）

```rust
// src/backend.rs
use crossterm::{
    execute, terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{EnableMouseCapture, DisableMouseCapture},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{stdout, Stdout};

pub type TuiTerminal = Terminal<CrosstermBackend<Stdout>>;

/// 初始化终端（适配 Windows / Linux / macOS）
pub fn init_terminal() -> Result<TuiTerminal> {
    // 1. 启用 raw mode（禁用行缓冲、信号处理）
    enable_raw_mode()?;
    
    // 2. 进入备用屏幕（Alternate Screen）
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    
    // 3. 创建 Ratatui 终端
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    
    Ok(terminal)
}

/// 恢复终端状态（退出时调用）
pub fn restore_terminal(mut terminal: TuiTerminal) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
```

**关键点**：
- `enable_raw_mode()` 让按键事件直接传递给程序，无需按回车。
- `EnterAlternateScreen` 进入备用屏幕，退出时恢复原终端内容。
- 这三个步骤在 Windows、Linux、macOS 上行为一致，crossterm 已做好抽象。

---

### 3.2 事件循环架构（异步 + 组件化）

OneInit TUI 采用 **异步事件驱动 + 即时渲染** 模式。事件循环与渲染分离，确保 UI 不阻塞后台操作（如包下载）。

```rust
// src/event.rs
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender, UnboundedReceiver};

/// 应用事件（封装 crossterm 事件 + 自定义事件）
#[derive(Debug, Clone)]
pub enum AppEvent {
    Tick,                          // 定时刷新
    Key(KeyCode),                  // 键盘事件
    Resize(u16, u16),              // 窗口大小变化
    InstallProgress(String, u8),   // 安装进度（来自核心引擎）
    InstallComplete(String, bool), // 安装完成
    Quit,                          // 退出
}

/// 事件循环（运行在独立 tokio 任务中）
pub fn start_event_loop(tx: UnboundedSender<AppEvent>) {
    tokio::spawn(async move {
        let mut reader = crossterm::event::EventStream::new();
        let mut tick_interval = tokio::time::interval(Duration::from_millis(100));
        
        loop {
            tokio::select! {
                // 定时 Tick
                _ = tick_interval.tick() => {
                    let _ = tx.send(AppEvent::Tick);
                }
                // 终端事件
                Some(Ok(event)) = reader.next() => {
                    match event {
                        CrosstermEvent::Key(key) => {
                            if key.code == KeyCode::Char('q') {
                                let _ = tx.send(AppEvent::Quit);
                            } else {
                                let _ = tx.send(AppEvent::Key(key.code));
                            }
                        }
                        CrosstermEvent::Resize(w, h) => {
                            let _ = tx.send(AppEvent::Resize(w, h));
                        }
                        _ => {}
                    }
                }
            }
        }
    });
}
```

**设计要点**：
- 使用 `tokio::sync::mpsc` 通道传递事件。
- Tick 定时器保证 UI 持续刷新（即时渲染模式）。
- `EventStream` 支持异步读取终端事件。

---

### 3.3 应用状态与屏幕管理

OneInit TUI 需要支持 **多屏幕切换**（主列表 → 安装详情 → 配置屏幕），采用 **屏幕枚举 + 状态驱动** 的方式。

```rust
// src/screen/mod.rs
use ratatui::{layout::Rect, Frame};

/// 所有屏幕类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    PackageList,   // 主屏幕：包列表
    Install,       // 安装屏幕：显示进度
    PackageInfo,   // 包详情
    Config,        // 配置屏幕
    Help,          // 帮助弹窗
}

/// 屏幕 trait（每个屏幕实现自己的渲染和事件处理）
pub trait ScreenRender {
    fn render(&self, f: &mut Frame, area: Rect, state: &AppState);
}

/// 应用状态（全局）
#[derive(Default)]
pub struct AppState {
    pub current_screen: Screen,
    pub packages: Vec<Package>,          // 可用包列表
    pub installed: Vec<InstalledPackage>, // 已安装包
    pub selected_index: usize,           // 当前选中项
    pub search_query: String,            // 搜索关键词
    pub install_queue: Vec<String>,      // 待安装队列
    pub install_progress: HashMap<String, u8>, // 安装进度
    pub should_quit: bool,
}
```

**屏幕切换逻辑**：

```rust
// src/app.rs
impl App {
    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Key(KeyCode::Enter) => {
                // 进入详情屏幕
                self.state.current_screen = Screen::PackageInfo;
            }
            AppEvent::Key(KeyCode::Esc) => {
                // 返回主屏幕
                self.state.current_screen = Screen::PackageList;
            }
            AppEvent::InstallProgress(pkg, progress) => {
                // 更新进度（来自核心引擎）
                self.state.install_progress.insert(pkg, progress);
            }
            AppEvent::InstallComplete(pkg, success) => {
                // 安装完成，刷新已安装列表
                self.refresh_installed();
            }
            _ => {}
        }
    }
}
```

---

### 3.4 组件化渲染（Component Architecture）

采用 Ratatui 官方推荐的 **Component 架构**，每个组件封装自己的状态、事件处理和渲染逻辑。

```rust
// src/components/mod.rs
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

/// 组件 trait（封装状态 + 渲染）
pub trait Component {
    /// 渲染组件
    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &AppState);
    
    /// 处理事件（返回 true 表示事件已消费）
    fn handle_event(&mut self, event: &AppEvent, state: &mut AppState) -> bool {
        false // 默认不处理
    }
}

/// 包列表组件
pub struct PackageListComponent {
    pub scroll_offset: usize,
    pub focused: bool,
}

impl Component for PackageListComponent {
    fn render(&mut self, area: Rect, buf: &mut Buffer, state: &AppState) {
        let items: Vec<String> = state.packages
            .iter()
            .filter(|p| p.name.contains(&state.search_query))
            .map(|p| format!("{}  {}", p.name, p.version))
            .collect();
        
        // 使用 Ratatui List 组件渲染
        let list = List::new(items)
            .highlight_style(Style::default().bg(Color::Blue))
            .block(Block::default().borders(Borders::ALL).title("Packages"));
        
        // 处理滚动
        let list_state = &mut self.list_state;
        list_state.select(Some(state.selected_index - self.scroll_offset));
        
        list.render(area, buf, list_state);
    }
    
    fn handle_event(&mut self, event: &AppEvent, state: &mut AppState) -> bool {
        match event {
            AppEvent::Key(KeyCode::Down) => {
                state.selected_index = (state.selected_index + 1).min(state.packages.len() - 1);
                true
            }
            AppEvent::Key(KeyCode::Up) => {
                state.selected_index = state.selected_index.saturating_sub(1);
                true
            }
            _ => false,
        }
    }
}
```

---

### 3.5 主循环（整合所有模块）

```rust
// src/main.rs
#[tokio::main]
async fn main() -> Result<()> {
    // 1. 初始化日志（调试用）
    tracing_subscriber::fmt().with_env_filter("info").init();
    
    // 2. 初始化终端
    let mut terminal = backend::init_terminal()?;
    
    // 3. 创建事件通道
    let (event_tx, mut event_rx) = unbounded_channel::<AppEvent>();
    
    // 4. 启动事件循环（独立任务）
    start_event_loop(event_tx);
    
    // 5. 初始化应用状态
    let mut app = App::new()?;
    
    // 6. 主循环
    while !app.state.should_quit {
        // 渲染
        terminal.draw(|f| app.render(f))?;
        
        // 处理事件（非阻塞）
        if let Ok(event) = event_rx.try_recv() {
            app.handle_event(event);
        }
        
        // 小睡一下避免 CPU 空转
        tokio::time::sleep(Duration::from_millis(16)).await;
    }
    
    // 7. 恢复终端
    backend::restore_terminal(terminal)?;
    
    Ok(())
}
```

---

## 四、跨终端适配（Windows 重点）

### 4.1 已知兼容性问题与解决方案

| 问题 | 表现 | 解决方案 |
| :--- | :--- | :--- |
| **下划线颜色不支持** | 在外部 PowerShell 报 `SetUnderlineColor not supported` | 禁用 `underline-color` 特性 |
| **键盘事件重复触发** | 按一次键触发两次事件 | 在事件处理中做去重 |
| **VT 序列支持差异** | VSCode 终端正常，外部 CMD 异常 | 显式启用 VT 处理 |
| **窗口大小变化卡顿** | 最大化窗口时 TUI 冻结 | 监听 Resize 事件并触发重绘 |

### 4.2 Cargo.toml 兼容性配置

```toml
[dependencies]
# 禁用默认特性，避免 underline-color 在旧 Windows 上出问题
ratatui = { version = "0.29", default-features = false, features = ["crossterm"] }
crossterm = { version = "0.28", features = ["event-stream"] }
```

### 4.3 终端能力检测（运行时降级）

```rust
// src/backend/capability.rs
/// 检测终端是否支持下划线颜色
pub fn supports_underline_color() -> bool {
    // 检测 Windows Terminal 或较新的 conhost
    if cfg!(windows) {
        // 检查环境变量: WT_SESSION 表示 Windows Terminal
        std::env::var("WT_SESSION").is_ok()
            || std::env::var("TERM_PROGRAM").map_or(false, |v| v == "vscode")
    } else {
        true // Linux/macOS 通常支持
    }
}

/// 根据终端能力调整样式
pub fn adaptive_style(base: Style) -> Style {
    if !supports_underline_color() {
        // 降级：用颜色替代下划线
        base.fg(Color::Cyan)
    } else {
        base.underlined()
    }
}
```

### 4.4 各终端测试建议

| 终端 | 测试重点 | 备注 |
| :--- | :--- | :--- |
| **Windows Terminal** | 完整功能测试 | 最佳体验，推荐开发环境 |
| **VS Code 集成终端** | 颜色、键盘事件 | 通常表现良好 |
| **PowerShell (外部)** | VT 序列、下划线 | 可能需要显式启用 VT |
| **CMD (传统)** | 基础功能降级 | 颜色和样式受限 |

---

## 五、错误处理与日志

TUI 应用崩溃会导致终端状态混乱（raw mode 未恢复）。需要 **Panic Hook** 确保退出时恢复终端。

```rust
// src/main.rs
fn setup_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // 恢复终端
        let _ = backend::restore_terminal();
        // 调用默认 hook
        default_hook(panic_info);
    }));
}
```

**日志策略**：
- 开发阶段：`tracing` 输出到文件（`/tmp/oneinit-tui.log`）
- 生产阶段：默认关闭日志，`--verbose` 开启

---

## 六、目录结构

```
oneinit-tui/
├── Cargo.toml
├── src/
│   ├── main.rs              # 入口 + 主循环
│   ├── app.rs               # App 结构 + 生命周期
│   ├── backend/
│   │   ├── mod.rs           # 终端初始化/恢复
│   │   └── capability.rs    # 终端能力检测
│   ├── event.rs             # 事件定义 + 事件循环
│   ├── state.rs             # AppState 定义
│   ├── screen/
│   │   ├── mod.rs           # Screen 枚举 + trait
│   │   ├── package_list.rs  # 主屏幕渲染
│   │   ├── install.rs       # 安装进度屏幕
│   │   └── help.rs          # 帮助弹窗
│   ├── components/
│   │   ├── mod.rs           # Component trait
│   │   ├── list.rs          # 可滚动列表组件
│   │   ├── search.rs        # 搜索输入组件
│   │   ├── progress.rs      # 进度条组件
│   │   └── footer.rs        # 底部状态栏
│   └── utils/
│       └── layout.rs        # 布局辅助函数
└── examples/
    └── demo.rs              # 开发演示
```

---

## 七、开发路线

1. ：终端初始化 + 事件循环骨架
2.：主屏幕（包列表 + 搜索 + 导航）
1. ：安装屏幕（进度展示 + 状态反馈）
2. ：跨终端适配测试（Windows / Linux / macOS）
3. ：集成 OneInit Core（调用 `install` / `uninstall` / `list`）
4. ：错误处理 + 日志 + 发布准备

---