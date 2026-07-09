// 应用状态 — 全局状态 + 屏幕路由
//
// AppState 持有所有数据：当前屏幕、包列表、选中项、搜索词、安装进度。
// 屏幕切换通过 current_screen 字段驱动渲染。

use std::collections::HashMap;

use crate::core::manifest::InstallRecord;
use crate::core::recipe::Recipe;

/// 所有屏幕类型
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
    /// 环境捕获结果
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
    /// 当前面板（主屏幕）
    pub active_pane: Pane,
    /// 可用配方列表
    pub available: Vec<Recipe>,
    /// 已安装记录列表
    pub installed: Vec<InstallRecord>,
    /// 已安装面板选中索引
    pub selected_installed: usize,
    /// 可安装面板选中索引
    pub selected_available: usize,
    /// 搜索关键词（预留）
    pub search_query: String,
    /// 安装进度（包名 → 百分比）
    pub install_progress: HashMap<String, u8>,
    /// 当前安装中的包名
    pub installing: Option<String>,
    /// 状态消息（操作结果 / 错误）
    pub message: Option<String>,
    /// 环境捕获结果（Capture 屏幕用）
    pub capture_result: Option<Vec<(String, Option<String>, String)>>, // (name, version, path)
    /// 是否应退出
    pub should_quit: bool,
}

impl AppState {
    /// 创建新状态并加载数据
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

    /// 刷新数据（从 Manifest + recipe 注册表重新读取）
    pub fn refresh(&mut self) {
        self.installed = crate::core::manifest::Manifest::open()
            .ok()
            .and_then(|m| m.list().ok())
            .unwrap_or_default();
        self.available = crate::core::recipe::list_recipes();
        self.clamp_selections();
    }

    /// 切换面板
    pub fn toggle_pane(&mut self) {
        self.active_pane = match self.active_pane {
            Pane::Installed => Pane::Available,
            Pane::Available => Pane::Installed,
        };
    }

    /// 移动光标（-1 上移，+1 下移）
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

    /// 获取当前选中的操作目标
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

    /// 约束选中索引
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
    /// 安装指定配方
    Install(String),
    /// 卸载指定包
    Uninstall(String),
}
