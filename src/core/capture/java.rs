// Java 环境检测器

use std::collections::BTreeMap;

use super::RuntimeEnv;
use super::detector::{EnvDetector, extract_version, find_command, run_command_with_stderr};
use crate::core::Result;

pub struct JavaDetector;

impl EnvDetector for JavaDetector {
    fn name(&self) -> &str {
        "java"
    }

    fn priority(&self) -> u8 {
        42
    }

    fn detect(&self) -> Result<Option<RuntimeEnv>> {
        // 1. 查找 java 命令
        let java_path = match find_command("java") {
            Some(p) => p,
            None => return Ok(None),
        };

        let java_str = java_path.to_string_lossy().to_string();

        // 2. 获取版本号
        // java -version 输出到 stderr！格式如:
        //   openjdk version "17.0.1" 2021-10-19
        //   java version "1.8.0_291"
        let version_output = run_command_with_stderr(&java_str, &["-version"]);

        let version = version_output
            .as_deref()
            .and_then(|s| {
                // 匹配 version "X.Y.Z" 或 version "X.Y.Z_WWW"
                if let Some(line) = s.lines().find(|l| l.contains("version")) {
                    // 提取引号内的版本号
                    if let Some(start) = line.find('"') {
                        let rest = &line[start + 1..];
                        if let Some(end) = rest.find('"') {
                            return Some(rest[..end].to_string());
                        }
                    }
                }
                None
            })
            .unwrap_or_else(|| "unknown".to_string());

        // 3. 检测 JDK（javac）和 JAVA_HOME
        let mut mirrors = BTreeMap::new();

        if let Some(javac_version) = find_command("javac")
            .and_then(|p| run_command_with_stderr(&p.to_string_lossy(), &["-version"]))
        {
            // javac -version 输出如 "javac 17.0.1"
            let jv = extract_version(&javac_version, "javac ").unwrap_or_else(|| {
                javac_version
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string()
            });
            mirrors.insert("javac".to_string(), jv);
        }

        if let Ok(java_home) = std::env::var("JAVA_HOME")
            && !java_home.is_empty()
        {
            mirrors.insert("JAVA_HOME".to_string(), java_home);
        }

        Ok(Some(RuntimeEnv {
            name: "java".to_string(),
            version,
            install_path: java_str,
            mirrors,
            global_packages: Vec::new(),
        }))
    }
}
