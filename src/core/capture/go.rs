// Go 环境检测器

use std::collections::BTreeMap;

use super::RuntimeEnv;
use super::detector::{EnvDetector, extract_version, find_command, run_command};
use crate::core::Result;

pub struct GoDetector;

impl EnvDetector for GoDetector {
    fn name(&self) -> &str {
        "go"
    }

    fn priority(&self) -> u8 {
        41
    }

    fn detect(&self) -> Result<Option<RuntimeEnv>> {
        // 1. find go command
        let go_path = match find_command("go") {
            Some(p) => p,
            None => return Ok(None),
        };

        let go_str = go_path.to_string_lossy().to_string();

        // 2. get version (输出如 "go version go1.21.5 windows/amd64")
        let version_str = run_command(&go_str, &["version"]);
        let version = version_str
            .as_deref()
            .and_then(|s| extract_version(s, "go version go"))
            .unwrap_or_else(|| "unknown".to_string());

        // 3. get GOPATH and GOROOT
        let mut mirrors = BTreeMap::new();
        if let Some(gopath) = run_command(&go_str, &["env", "GOPATH"])
            && !gopath.is_empty()
        {
            mirrors.insert("GOPATH".to_string(), gopath);
        }
        if let Some(goroot) = run_command(&go_str, &["env", "GOROOT"])
            && !goroot.is_empty()
        {
            mirrors.insert("GOROOT".to_string(), goroot);
        }

        Ok(Some(RuntimeEnv {
            name: "go".to_string(),
            version,
            install_path: go_str,
            mirrors,
            global_packages: Vec::new(),
        }))
    }
}
