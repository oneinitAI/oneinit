// Python 环境检测器

use std::collections::BTreeMap;

use super::detector::{extract_version, find_command, run_command, EnvDetector};
use super::RuntimeEnv;
use crate::core::Result;

pub struct PythonDetector;

impl EnvDetector for PythonDetector {
    fn name(&self) -> &'static str {
        "python"
    }

    fn priority(&self) -> u8 {
        10
    }

    fn detect(&self) -> Result<Option<RuntimeEnv>> {
        // 1. 查找 python 命令
        let python_cmd = if cfg!(windows) { "python" } else { "python3" };

        let python_path = match find_command(python_cmd) {
            Some(p) => p,
            None => return Ok(None),
        };

        // 2. 获取版本号
        let version_str = run_command(&python_path.to_string_lossy(), &["--version"]);
        // Python 2 的 --version 输出到 stderr，Python 3 输出到 stdout
        let version = version_str
            .as_deref()
            .and_then(|s| extract_version(s, "Python "))
            .unwrap_or_else(|| "unknown".to_string());

        // 3. 检测 pip 镜像源
        let mut mirrors = BTreeMap::new();
        if let Some(mirror) = run_command("pip", &["config", "get", "index-url"]) {
            if !mirror.is_empty() {
                mirrors.insert("pip".to_string(), mirror);
            }
        }

        // 4. 获取全局包列表
        let global_packages = run_command("pip", &["list", "--format=freeze"])
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
            install_path: python_path.to_string_lossy().to_string(),
            mirrors,
            global_packages,
        }))
    }
}
