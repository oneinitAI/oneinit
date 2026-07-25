// App state — global state + screen routing
//
// AppState holds: current screen, packages, selection, search, progress.
// Screen switching via current_screen field drives rendering.

use std::collections::HashMap;

use crate::core::manifest::InstallRecord;
use crate::core::recipe::Recipe;

/// All screen types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// 主屏幕：包列表
    PackageList,
    /// 安装屏幕：显示进度
    Install,
    /// 包详情
    PackageInfo,
    /// 帮助弹窗
    Help,
    /// Capture Results
    Capture,
}

/// 当前面板焦点（主屏幕用）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    /// 已安装面板
    Installed,
    /// 可安装面板
    Available,
}

/// 应用状态（全局）
pub struct AppState {
    /// 当前屏幕
    pub current_screen: Screen,
    /// Current pane (main screen)
    pub active_pane: Pane,
    /// Available recipe list
    pub available: Vec<Recipe>,
    /// Installed records list
    pub installed: Vec<InstallRecord>,
    /// 已安装面板选中索引
    pub selected_installed: usize,
    /// 可安装面板选中索引
    pub selected_available: usize,
    /// Search query (reserved)
    pub search_query: String,
    /// 安装进度（包名 → 百分比）
    pub install_progress: HashMap<String, u8>,
    /// Currently installing package
    pub installing: Option<String>,
    /// Status message（操作结果 / 错误）
    pub message: Option<String>,
    /// Capture Results (for Capture screen)
    pub capture_result: Option<Vec<(String, Option<String>, String)>>, // (name, version, path)
    /// 是否应Quit
    pub should_quit: bool,
}

impl AppState {
    /// Create new state and load data
    pub fn new() -> Self {
        let mut state = Self {
            current_screen: Screen::PackageList,
            active_pane: Pane::Available,
            available: Vec::new(),
            installed: Vec::new(),
            selected_installed: 0,
            selected_available: 0,
            search_query: String::new(),
            install_progress: HashMap::new(),
            installing: None,
            message: None,
            capture_result: None,
            should_quit: false,
        };
        state.refresh();
        state
    }

    /// Refresh data (reload from Manifest + recipe registry)
    pub fn refresh(&mut self) {
        self.installed = crate::core::manifest::Manifest::open()
            .ok()
            .and_then(|m| m.list().ok())
            .unwrap_or_default();
        self.available = crate::core::recipe::list_recipes();
        self.clamp_selections();
    }

    /// Toggle pane
    pub fn toggle_pane(&mut self) {
        self.active_pane = match self.active_pane {
            Pane::Installed => Pane::Available,
            Pane::Available => Pane::Installed,
        };
    }

    /// Move cursor (-1 up, +1 down)
    pub fn move_cursor(&mut self, delta: i32) {
        match self.active_pane {
            Pane::Installed => {
                let len = self.installed.len();
                if len == 0 {
                    return;
                }
                let new = self.selected_installed as i32 + delta;
                self.selected_installed = if new < 0 {
                    len - 1
                } else {
                    (new as usize) % len
                };
            }
            Pane::Available => {
                let len = self.available.len();
                if len == 0 {
                    return;
                }
                let new = self.selected_available as i32 + delta;
                self.selected_available = if new < 0 {
                    len - 1
                } else {
                    (new as usize) % len
                };
            }
        }
    }

    /// Get current selection target
    pub fn current_target(&self) -> Option<Target> {
        match self.active_pane {
            Pane::Installed => self
                .installed
                .get(self.selected_installed)
                .map(|r| Target::Uninstall(r.name.clone())),
            Pane::Available => self
                .available
                .get(self.selected_available)
                .map(|r| Target::Install(r.name.clone())),
        }
    }

    /// Clamp selection index
    fn clamp_selections(&mut self) {
        if !self.installed.is_empty() {
            self.selected_installed = self.selected_installed.min(self.installed.len() - 1);
        } else {
            self.selected_installed = 0;
        }
        if !self.available.is_empty() {
            self.selected_available = self.selected_available.min(self.available.len() - 1);
        } else {
            self.selected_available = 0;
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// 操作目标（从 TUI 触发）
#[derive(Debug, Clone)]
pub enum Target {
    /// 安装指定recipe
    Install(String),
    /// 卸载指定包
    Uninstall(String),
}
