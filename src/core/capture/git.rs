// Git 环境检测器

use std::collections::BTreeMap;

use super::RuntimeEnv;
use super::detector::{EnvDetector, extract_version, find_command, run_command};
use crate::core::Result;

pub struct GitDetector;

impl EnvDetector for GitDetector {
    fn name(&self) -> &str {
        "git"
    }

    fn priority(&self) -> u8 {
        30
    }

    fn detect(&self) -> Result<Option<RuntimeEnv>> {
        // 1. find git command
        let git_path = match find_command("git") {
            Some(p) => p,
            None => return Ok(None),
        };

        let git_str = git_path.to_string_lossy().to_string();

        // 2. get version (输出如 "git version 2.43.0.windows.1")
        let version_str = run_command(&git_str, &["--version"]);
        let version = version_str
            .as_deref()
            .and_then(|s| extract_version(s, "git version "))
            .unwrap_or_else(|| "unknown".to_string());

        // 3. 检测 git 用户配置（用已解析的 git 路径）
        let mut mirrors = BTreeMap::new();
        if let Some(user_name) = run_command(&git_str, &["config", "user.name"])
            && !user_name.is_empty()
        {
            mirrors.insert("user.name".to_string(), user_name);
        }
        if let Some(user_email) = run_command(&git_str, &["config", "user.email"])
            && !user_email.is_empty()
        {
            mirrors.insert("user.email".to_string(), user_email);
        }

        Ok(Some(RuntimeEnv {
            name: "git".to_string(),
            version,
            install_path: git_str,
            mirrors,
            global_packages: Vec::new(),
        }))
    }
}
