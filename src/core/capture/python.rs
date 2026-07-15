// Python 环境检测器

use std::collections::BTreeMap;

use super::detector::{extract_version, find_command, run_command, run_command_combined, EnvDetector};
use super::RuntimeEnv;
use crate::core::Result;

pub struct PythonDetector;

impl EnvDetector for PythonDetector {
    fn name(&self) -> &str {
        "python"
    }

    fn priority(&self) -> u8 {
        10
    }

    fn detect(&self) -> Result<Option<RuntimeEnv>> {
        // 1. 查找 python 命令
        // Windows: 先试 python，再试 py launcher
        // Unix: 先试 python3，再试 python
        let python_path = if cfg!(windows) {
            find_command("python")
                .or_else(|| find_py_launcher())
        } else {
            find_command("python3").or_else(|| find_command("python"))
        };

        let python_path = match python_path {
            Some(p) => p,
            None => return Ok(None),
        };

        let python_str = python_path.to_string_lossy().to_string();

        // 2. 获取版本号（Python 2 的 --version 输出到 stderr，Python 3 输出到 stdout）
        let version_output = run_command_combined(&python_str, &["--version"]);
        let version = version_output
            .as_deref()
            .and_then(|s| extract_version(s, "Python "))
            .unwrap_or_else(|| "unknown".to_string());

        // 3. 检测 pip 镜像源
        // 优先用 <python> -m pip（更可靠），回退 pip3、pip
        let mut mirrors = BTreeMap::new();
        if let Some(mirror) = run_command(&python_str, &["-m", "pip", "config", "get", "index-url"])
            .or_else(|| run_command("pip3", &["config", "get", "index-url"]))
            .or_else(|| run_command("pip", &["config", "get", "index-url"]))
        {
            if !mirror.is_empty() {
                mirrors.insert("pip".to_string(), mirror);
            }
        }

        // 4. 获取全局包列表
        let global_packages = run_command(&python_str, &["-m", "pip", "list", "--format=freeze"])
            .or_else(|| run_command("pip3", &["list", "--format=freeze"]))
            .or_else(|| run_command("pip", &["list", "--format=freeze"]))
            .map(|output| {
                output
                    .lines()
                    .filter(|l| !l.is_empty())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        Ok(Some(RuntimeEnv {
            name: "python".to_string(),
            version,
            install_path: python_str,
            mirrors,
            global_packages,
        }))
    }
}

/// 在 Windows 上查找 py launcher
fn find_py_launcher() -> Option<std::path::PathBuf> {
    if !cfg!(windows) {
        return None;
    }
    let output = std::process::Command::new("py")
        .arg("-0p")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    // py -0p 输出格式: " -V:3.13 *        C:\Users\...\python.exe"
    String::from_utf8(output.stdout)
        .ok()?
        .lines()
        .next()
        .and_then(|line| {
            // 取最后一列路径（去掉前缀标记）
            let parts: Vec<&str> = line.split_whitespace().collect();
            parts.last().map(std::path::PathBuf::from)
        })
        .filter(|p| p.exists())
}
