// Node.js 环境检测器

use std::collections::BTreeMap;

use super::detector::{find_command, run_command, EnvDetector};
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
        let node_cmd = if cfg!(windows) { "node" } else { "node" };

        let node_path = match find_command(node_cmd) {
            Some(p) => p,
            None => return Ok(None),
        };

        let node_str = node_path.to_string_lossy().to_string();

        // 2. 获取版本号 (输出如 "v18.19.0")
        let version_str = run_command(&node_str, &["--version"]);
        let version = version_str
            .as_deref()
            .and_then(|s| {
                // 去掉 "v" 前缀
                s.strip_prefix('v')
                    .map(|v| v.trim().to_string())
                    .or_else(|| Some(s.trim().to_string()))
            })
            .unwrap_or_else(|| "unknown".to_string());

        // 3. 解析 npm 路径（通常在 node 同目录或 ../npm.cmd）
        let npm_cmd = resolve_npm(&node_path);

        // 4. 检测 npm 镜像源
        let mut mirrors = BTreeMap::new();
        if let Some(registry) = run_command(&npm_cmd, &["config", "get", "registry"]) {
            if !registry.is_empty() && registry != "undefined" {
                mirrors.insert("npm".to_string(), registry);
            }
        }

        // 5. 获取全局包列表
        let global_packages =
            run_command(&npm_cmd, &["ls", "-g", "--depth=0", "--parseable"])
                .map(|output| {
                    // --parseable 输出绝对路径，最后一段是包名
                    output
                        .lines()
                        .skip(1) // 跳过 npm 根目录
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
            install_path: node_str,
            mirrors,
            global_packages,
        }))
    }
}

/// 解析 npm 命令路径
///
/// npm 通常和 node 在同一目录（Unix）或用 npm.cmd 包装（Windows）。
/// 回退：npm.cmd / npm / npx
fn resolve_npm(node_path: &std::path::Path) -> String {
    // 尝试从 node 同目录找 npm
    if let Some(dir) = node_path.parent() {
        let candidates = if cfg!(windows) {
            vec!["npm.cmd", "npx.cmd", "npm"]
        } else {
            vec!["npm", "npx"]
        };
        for name in candidates {
            let npm_path = dir.join(name);
            if npm_path.exists() {
                return npm_path.to_string_lossy().to_string();
            }
        }
    }
    // 回退到 PATH 查找
    find_command("npm")
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "npm".to_string())
}
