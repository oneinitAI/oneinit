// 检测器 trait + 调度器 + 跨平台命令查找工具

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

use super::{RuntimeEnv, Result};

// ============================================================
// 检测器 trait（同步，无 async-trait）
// ============================================================

/// 环境检测器 Trait
///
/// 设计决策：使用同步方法而非 async，因为 Command::output() 本身是阻塞调用，
/// async 包装无意义且会增加复杂度（需要 async-trait crate 做 dyn dispatch）。
pub trait EnvDetector: Send + Sync {
    /// 检测目标环境是否已安装，返回 None 表示未检测到
    fn detect(&self) -> Result<Option<RuntimeEnv>>;

    /// 检测器名称（用于日志和 envs map key）
    fn name(&self) -> &'static str;

    /// 检测优先级（数字越小越优先，默认 50）
    fn priority(&self) -> u8 {
        50
    }
}

// ============================================================
// 检测器调度器
// ============================================================

/// 检测器调度器 — 注册并运行所有检测器
pub struct DetectorScheduler {
    detectors: Vec<Box<dyn EnvDetector>>,
}

impl DetectorScheduler {
    pub fn new() -> Self {
        Self {
            detectors: Vec::new(),
        }
    }

    /// 注册所有内置检测器
    pub fn register_defaults(&mut self) {
        self.detectors.push(Box::new(super::python::PythonDetector));
        self.detectors.push(Box::new(super::node::NodeDetector));
        self.detectors.push(Box::new(super::git::GitDetector));
        // 按优先级排序
        self.detectors.sort_by_key(|d| d.priority());
    }

    /// 执行所有检测
    pub fn scan(&self) -> BTreeMap<String, Option<RuntimeEnv>> {
        let mut results = BTreeMap::new();
        for detector in &self.detectors {
            let name = detector.name();
            match detector.detect() {
                Ok(Some(env)) => {
                    results.insert(name.to_string(), Some(env));
                }
                Ok(None) => {
                    results.insert(name.to_string(), None);
                }
                Err(_) => {
                    results.insert(name.to_string(), None);
                }
            }
        }
        results
    }
}

impl Default for DetectorScheduler {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 跨平台命令查找工具
// ============================================================

/// 查找命令的完整路径
///
/// Windows 用 `where`，Unix 用 `which`。
/// 返回找到的第一个路径，或 None。
pub fn find_command(name: &str) -> Option<PathBuf> {
    let lookup_cmd = if cfg!(windows) { "where" } else { "which" };

    let output = Command::new(lookup_cmd)
        .arg(name)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8(output.stdout)
        .ok()?
        .lines()
        .next()
        .map(|s| PathBuf::from(s.trim()))
}

/// 运行命令并返回 stdout（去除首尾空白）
pub fn run_command(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// 从输出中提取版本号
///
/// 简单字符串解析，不使用 regex。
/// 例: extract_version("Python 3.11.9", "Python ") -> "3.11.9"
pub fn extract_version(output: &str, prefix: &str) -> Option<String> {
    output
        .lines()
        .find(|l| l.contains(prefix))
        .and_then(|l| l.split(prefix).nth(1))
        .map(|s| s.trim().to_string())
}
