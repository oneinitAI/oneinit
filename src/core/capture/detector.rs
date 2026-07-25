// Detector trait + scheduler + cross-platform command helpers
//
// find_command 采用多策略查找：where/which -> PATH 手动遍历 -> exe 扩展名补全

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

use super::{Result, RuntimeEnv};

// ============================================================
// 检测器 trait（同步，无 async-trait）
// ============================================================

/// Environment detector trait
///
/// Design: synchronous trait, Command::output() is blocking,
/// async wrapping adds complexity unnecessarily（需要 async-trait crate 做 dyn dispatch）。
pub trait EnvDetector: Send + Sync {
    /// 检测目标环境是否已安装，返回 None 表示not detected
    fn detect(&self) -> Result<Option<RuntimeEnv>>;

    /// detector name (for logging and envs map key)
    fn name(&self) -> &str;

    /// priority (lower = higher priority, default 50)
    fn priority(&self) -> u8 {
        50
    }
}

// ============================================================
// 检测器调度器
// ============================================================

/// Detector scheduler — register and run all detectors
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
        self.detectors.push(Box::new(super::rust::RustDetector));
        self.detectors.push(Box::new(super::go::GoDetector));
        self.detectors.push(Box::new(super::java::JavaDetector));
        self.detectors.push(Box::new(super::docker::DockerDetector));
        // 注册自定义检测器
        self.register_custom();
        // 按优先级排序
        self.detectors.sort_by_key(|d| d.priority());
    }

    /// 从 ~/.oneinit/scan_config.yaml 加载自定义检测器
    fn register_custom(&mut self) {
        let config_path = crate::core::data_dir().join("scan_config.yaml");
        if let Ok(content) = std::fs::read_to_string(&config_path)
            && let Ok(config) = serde_yaml::from_str::<CustomDetectorConfig>(&content)
        {
            for def in config.custom_detectors {
                self.detectors.push(Box::new(CustomDetector::new(def)));
            }
        }
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
                Err(_e) => {
                    // 检测出错不中断，记录为not detected
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
// 自定义检测器
// ============================================================

/// 自定义检测器配置（从 scan_config.yaml 反序列化）
#[derive(Debug, serde::Deserialize)]
struct CustomDetectorConfig {
    custom_detectors: Vec<CustomDetectorDef>,
}

/// 单个自定义检测器定义
#[derive(Debug, serde::Deserialize)]
struct CustomDetectorDef {
    name: String,
    command: String,
    version_prefix: String,
}

/// 自定义检测器（运行用户指定的命令）
struct CustomDetector {
    def: CustomDetectorDef,
}

impl CustomDetector {
    fn new(def: CustomDetectorDef) -> Self {
        Self { def }
    }
}

impl EnvDetector for CustomDetector {
    fn name(&self) -> &str {
        &self.def.name
    }

    fn detect(&self) -> Result<Option<RuntimeEnv>> {
        // 解析命令（支持带参数，如 "flutter --version"）
        let parts: Vec<&str> = self.def.command.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(None);
        }

        let program = parts[0];
        let args = &parts[1..];

        // 先检查命令是否exists
        if find_command(program).is_none() {
            return Ok(None);
        }

        // 运行命令获取输出
        let output = run_command(program, args);
        let combined = output.or_else(|| run_command_with_stderr(program, args));

        let version = combined
            .as_deref()
            .and_then(|s| {
                if self.def.version_prefix.is_empty() {
                    Some(s.lines().next()?.trim().to_string())
                } else {
                    extract_version(s, &self.def.version_prefix)
                }
            })
            .unwrap_or_else(|| "unknown".to_string());

        let install_path = find_command(program)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| program.to_string());

        Ok(Some(RuntimeEnv {
            name: self.def.name.clone(),
            version,
            install_path,
            mirrors: BTreeMap::new(),
            global_packages: Vec::new(),
        }))
    }

    fn priority(&self) -> u8 {
        90 // 自定义检测器优先级最低
    }
}

// ============================================================
// 跨平台命令查找工具（多策略）
// ============================================================

/// 查找命令的完整路径（多策略查找）
///
/// 策略1: Windows uses `where`，Unix uses `which`
/// 策略2: PATH 手动遍历回退（where/which 失败或不可用时）
/// 策略3: Windows 额外尝试 .exe/.bat/.cmd 扩展名
pub fn find_command(name: &str) -> Option<PathBuf> {
    // 策略1: where/which
    if let Some(path) = find_via_system(name) {
        return Some(path);
    }

    // 策略2: PATH 手动遍历
    if let Some(path) = find_in_path(name) {
        return Some(path);
    }

    // 策略3: Windows 扩展名补全
    if cfg!(windows) {
        for ext in exe_extensions() {
            let name_with_ext = format!("{}{}", name, ext);
            if let Some(path) = find_via_system(&name_with_ext) {
                return Some(path);
            }
            if let Some(path) = find_in_path(&name_with_ext) {
                return Some(path);
            }
        }
    }

    None
}

/// 通过系统命令 (where/which) 查找
fn find_via_system(name: &str) -> Option<PathBuf> {
    let lookup_cmd = if cfg!(windows) { "where" } else { "which" };

    let output = Command::new(lookup_cmd).arg(name).output().ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8(output.stdout)
        .ok()?
        .lines()
        .next()
        .map(|s| PathBuf::from(s.trim()))
        .filter(|p| p.exists())
}

/// 在 PATH 环境变量中手动查找命令
///
/// 当 where/which 不可用或找不到时，拆分 PATH 逐目录检查。
fn find_in_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var("PATH").ok()?;
    let sep = if cfg!(windows) { ';' } else { ':' };

    for dir in path_var.split(sep) {
        if dir.is_empty() {
            continue;
        }
        let dir_path = PathBuf::from(dir);

        // 直接检查 name
        let candidate = dir_path.join(name);
        if candidate.exists() {
            return Some(candidate);
        }

        // Windows: 检查带扩展名的变体
        if cfg!(windows) {
            for ext in exe_extensions() {
                let candidate_ext = dir_path.join(format!("{}{}", name, ext));
                if candidate_ext.exists() {
                    return Some(candidate_ext);
                }
            }
        }
    }

    None
}

/// Windows 可执行文件扩展名
fn exe_extensions() -> &'static [&'static str] {
    if cfg!(windows) {
        &[".exe", ".bat", ".cmd", ".ps1"]
    } else {
        &[""]
    }
}

/// run command and return stdout (trimmed)
pub fn run_command(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// 运行命令并返回 stderr（某些程序如 java -version 输出到 stderr）
pub fn run_command_with_stderr(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    // 有些命令即使成功 exit code 也可能非 0，这里宽松处理
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let trimmed = combined.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// 运行命令，合并 stdout 和 stderr 输出
pub fn run_command_combined(program: &str, args: &[&str]) -> Option<String> {
    run_command(program, args).or_else(|| run_command_with_stderr(program, args))
}

/// extract version from output
///
/// 简单 chars串解析，不使用 regex。
/// 例: extract_version("Python 3.11.9", "Python ") -> "3.11.9"
pub fn extract_version(output: &str, prefix: &str) -> Option<String> {
    output
        .lines()
        .find(|l| l.contains(prefix))
        .and_then(|l| l.split(prefix).nth(1))
        .map(|s| s.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_version_python() {
        assert_eq!(
            extract_version("Python 3.11.9\n", "Python "),
            Some("3.11.9".to_string())
        );
    }

    #[test]
    fn test_extract_version_git() {
        assert_eq!(
            extract_version("git version 2.43.0.windows.1\n", "git version "),
            Some("2.43.0.windows.1".to_string())
        );
    }

    #[test]
    fn test_extract_version_not_found() {
        assert_eq!(extract_version("hello world", "Python "), None);
    }

    #[test]
    fn test_extract_version_multiline() {
        let output = "some line\nPython 3.8.10\nanother line";
        assert_eq!(
            extract_version(output, "Python "),
            Some("3.8.10".to_string())
        );
    }

    #[test]
    fn test_run_command_nonexistent() {
        assert!(run_command("this_command_does_not_exist_12345", &[]).is_none());
    }

    #[test]
    fn test_find_command_existing() {
        // git 应该在大多数开发环境中可用
        let result = find_command("git");
        // 不做硬断言（CI 可能没有 git），只确保不 panic
        let _ = result;
    }
}
