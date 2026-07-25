// Rust 环境检测器

use std::collections::BTreeMap;

use super::RuntimeEnv;
use super::detector::{EnvDetector, extract_version, find_command, run_command};
use crate::core::Result;

pub struct RustDetector;

impl EnvDetector for RustDetector {
    fn name(&self) -> &str {
        "rust"
    }

    fn priority(&self) -> u8 {
        40
    }

    fn detect(&self) -> Result<Option<RuntimeEnv>> {
        // 1. find rustc command
        let rustc_path = match find_command("rustc") {
            Some(p) => p,
            None => return Ok(None),
        };

        let rustc_str = rustc_path.to_string_lossy().to_string();

        // 2. 获取 rustc 版本 (输出如 "rustc 1.75.0 (82e160899 2023-12-04)")
        let version_str = run_command(&rustc_str, &["--version"]);
        let version = version_str
            .as_deref()
            .and_then(|s| extract_version(s, "rustc "))
            .unwrap_or_else(|| "unknown".to_string());

        // 3. get cargo version
        let mut mirrors = BTreeMap::new();
        if let Some(cargo_version) =
            find_command("cargo").and_then(|p| run_command(&p.to_string_lossy(), &["--version"]))
        {
            mirrors.insert("cargo".to_string(), cargo_version);
        }

        // 4. get rustup toolchain info
        if let Some(toolchain) = run_command("rustup", &["show", "active-toolchain"]) {
            // 输出如 "stable-x86_64-pc-windows-msvc (default)"
            let tc = toolchain.split_whitespace().next().unwrap_or("");
            if !tc.is_empty() {
                mirrors.insert("toolchain".to_string(), tc.to_string());
            }
        }

        Ok(Some(RuntimeEnv {
            name: "rust".to_string(),
            version,
            install_path: rustc_str,
            mirrors,
            global_packages: Vec::new(),
        }))
    }
}
