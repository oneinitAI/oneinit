// Node.js 环境检测器

use std::collections::BTreeMap;

use super::detector::{extract_version, find_command, run_command, EnvDetector};
use super::RuntimeEnv;
use crate::core::Result;

pub struct NodeDetector;

impl EnvDetector for NodeDetector {
    fn name(&self) -> &'static str {
        "node"
    }

    fn priority(&self) -> u8 {
        20
    }

    fn detect(&self) -> Result<Option<RuntimeEnv>> {
        // 1. 查找 node 命令
        let node_cmd = if cfg!(windows) { "node.exe" } else { "node" };

        let node_path = match find_command(node_cmd) {
            Some(p) => p,
            None => return Ok(None),
        };

        // 2. 获取版本号 (输出如 "v18.19.0")
        let version_str = run_command(&node_path.to_string_lossy(), &["--version"]);
        let version = version_str
            .as_deref()
            .and_then(|s| {
                // 去掉 "v" 前缀
                s.strip_prefix('v').map(|v| v.trim().to_string()).or_else(|| Some(s.trim().to_string()))
            })
            .unwrap_or_else(|| "unknown".to_string());

        // 3. 检测 npm 镜像源
        let mut mirrors = BTreeMap::new();
        if let Some(registry) = run_command("npm", &["config", "get", "registry"]) {
            if !registry.is_empty() {
                mirrors.insert("npm".to_string(), registry);
            }
        }

        // 4. 获取全局包列表
        // npm ls -g --depth=0 输出格式: "  +-- package@version"
        let global_packages = run_command("npm", &["ls", "-g", "--depth=0", "--parseable"])
            .map(|output| {
                // --parseable 输出绝对路径，我们取最后一段作为包名
                output
                    .lines()
                    .skip(1) // 跳过第一行（npm 根目录）
                    .filter_map(|line| {
                        let name = line.rsplit(['/', '\\']).next()?;
                        if name.is_empty() || name.contains("node_modules") {
                            None
                        } else {
                            Some(name.to_string())
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(Some(RuntimeEnv {
            name: "node".to_string(),
            version,
            install_path: node_path.to_string_lossy().to_string(),
            mirrors,
            global_packages,
        }))
    }
}
