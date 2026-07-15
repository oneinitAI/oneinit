// Docker 环境检测器

use std::collections::BTreeMap;

use super::detector::{extract_version, find_command, run_command, EnvDetector};
use super::RuntimeEnv;
use crate::core::Result;

pub struct DockerDetector;

impl EnvDetector for DockerDetector {
    fn name(&self) -> &str {
        "docker"
    }

    fn priority(&self) -> u8 {
        43
    }

    fn detect(&self) -> Result<Option<RuntimeEnv>> {
        // 1. 查找 docker 命令
        let docker_path = match find_command("docker") {
            Some(p) => p,
            None => return Ok(None),
        };

        let docker_str = docker_path.to_string_lossy().to_string();

        // 2. 获取版本 (输出如 "Docker version 24.0.7, build afdd53b")
        let version_str = run_command(&docker_str, &["--version"]);
        let version = version_str
            .as_deref()
            .and_then(|s| {
                // "Docker version 24.0.7, build ..." -> "24.0.7"
                extract_version(s, "Docker version ")
                    .map(|v| v.split(',').next().unwrap_or(&v).trim().to_string())
            })
            .unwrap_or_else(|| "unknown".to_string());

        // 3. 检测 compose 版本
        let mut mirrors = BTreeMap::new();

        if let Some(compose_version) = run_command(&docker_str, &["compose", "version"]) {
            // "Docker Compose version v2.23.0" -> "v2.23.0"
            let cv = extract_version(&compose_version, "Docker Compose version ")
                .unwrap_or_else(|| compose_version.lines().next().unwrap_or("").trim().to_string());
            mirrors.insert("compose".to_string(), cv);
        }

        // 4. 获取容器数和镜像数
        if let Some(containers) = run_command(&docker_str, &["ps", "-q"]) {
            let count = containers.lines().count();
            mirrors.insert("containers".to_string(), count.to_string());
        }

        if let Some(images) = run_command(&docker_str, &["images", "-q"]) {
            let count = images.lines().count();
            mirrors.insert("images".to_string(), count.to_string());
        }

        Ok(Some(RuntimeEnv {
            name: "docker".to_string(),
            version,
            install_path: docker_str,
            mirrors,
            global_packages: Vec::new(),
        }))
    }
}
