//! Dynamic recipe builder for the non-exact-match system.
//!
//! Given a versioned family + a concrete version, build a
//! [`CommunityRecipe`] whose URL / checksum are version-parameterized and
//! verified against official sources. Installers reuse the normal
//! plan → execute path.

use super::community_recipe::{
    CommunityRecipe, ConfigFile, PlatformConfig, Platforms, PostInstallConfig,
};
use super::{CoreError, Result};

/// Build a dynamic recipe for a versioned family at a concrete version.
///
/// Returns Err when the platform is unsupported or the checksum could not be
/// resolved (unless `no_checksum`). `refresh` forces re-fetching checksums.
pub async fn build(
    family: &str,
    version: &str,
    refresh: bool,
    no_checksum: bool,
) -> Result<CommunityRecipe> {
    let recipe = match family {
        "node" => node_recipe(version).await?,
        "go" => go_recipe(version).await?,
        "python" => python_recipe(version).await?,
        "java" => java_recipe(version).await?,
        "rust" => rust_recipe(version)?,
        other => {
            return Err(CoreError::Other(format!(
                "暂不支持 '{other}' 的动态安装"
            )));
        }
    };

    // Attach checksums (per platform), verifying against official sources.
    // Builders that already resolved checksums (java via the Adoptium API,
    // rust via pinned rustup-init hashes) are skipped.
    let mut out = recipe;

    // checksums
    for platform in ["windows", "linux", "darwin"] {
        let Some(cfg) = platform_cfg_mut(&mut out, platform) else {
            continue;
        };
        if !cfg.sha256.is_empty() {
            continue; // already resolved by the builder
        }
        let filename = cfg.url.rsplit('/').next().unwrap_or("").to_string();
        let hash = super::checksum::resolve(family, version, &filename, refresh).await?;
        match hash {
            Some(h) => cfg.sha256 = h,
            None if no_checksum => {
                cfg.sha256 = String::new(); // planner treats empty as "skip verify"
            }
            None => {
                return Err(CoreError::Other(format!(
                     "无法解析 {family}@{version} ({filename}) 的校验和。\
                     使用 --no-checksum 重试可跳过校验（有风险）。"
                )));
            }
        }
    }

    Ok(out)
}

fn platform_cfg_mut<'a>(
    recipe: &'a mut CommunityRecipe,
    platform: &str,
) -> Option<&'a mut PlatformConfig> {
    match platform {
        "windows" => recipe.platforms.windows.as_mut(),
        "linux" => recipe.platforms.linux.as_mut(),
        "darwin" => recipe.platforms.darwin.as_mut(),
        _ => None,
    }
}

/// node — nodejs.org/dist/v{ver}/node-v{ver}-{os}-{arch}.{ext}
async fn node_recipe(version: &str) -> Result<CommunityRecipe> {
    let arch = if std::env::consts::ARCH == "aarch64" {
        "arm64"
    } else {
        "x64"
    };
    let base = format!("https://nodejs.org/dist/v{version}");

    // extension / install type are per-platform: windows ships .zip,
    // unix ships .tar.gz — independent of the host we're building on.
    let filename = |os: &str, a: &str| {
        let ext = if os == "win" { "zip" } else { "tar.gz" };
        format!("node-v{version}-{os}-{a}.{ext}")
    };
    let install_type = |os: &str| {
        if os == "win" {
            "zip_extract"
        } else {
            "tar_extract"
        }
    };
    let install_path = format!("node-{version}");
    let platform_cfg = |os: &str, a: &str| PlatformConfig {
        url: format!("{base}/{}", filename(os, a)),
        sha256: String::new(), // filled by build()
        install_type: install_type(os).to_string(),
        install_args: None,
        install_path: install_path.clone(),
        path_add: vec![format!("{{{{install_dir}}}}/node-v{version}-{os}-{a}")],
    };

    Ok(CommunityRecipe {
        name: format!("node@{version}"),
        version: version.to_string(),
        description: format!("Node.js {version}"),
        license: Some("MIT".into()),
        license_url: Some("https://github.com/nodejs/node/blob/main/LICENSE".into()),
        platforms: Platforms {
            windows: Some(platform_cfg("win", "x64")),
            linux: Some(platform_cfg("linux", arch)),
            darwin: Some(platform_cfg("darwin", arch)),
        },
        post_install: None,
        depends: None,
        tags: Some(vec!["runtime".into(), "javascript".into()]),
        maintainer: None,
    })
}

/// go — go.dev/dl/go{ver}.{os}-{arch}.{ext}
async fn go_recipe(version: &str) -> Result<CommunityRecipe> {
    let os = checksum_os();
    let arch = std::env::consts::ARCH;
    let (os_name, a_name, ext, install_type) = match (os, arch) {
        ("win", "x86_64") => ("windows", "amd64", "zip", "zip_extract"),
        ("linux", "x86_64") => ("linux", "amd64", "tar.gz", "tar_extract"),
        ("linux", "aarch64") => ("linux", "arm64", "tar.gz", "tar_extract"),
        ("darwin", "x86_64") => ("darwin", "amd64", "tar.gz", "tar_extract"),
        ("darwin", "aarch64") => ("darwin", "arm64", "tar.gz", "tar_extract"),
        (o, a) => {
            return Err(CoreError::Other(format!(
                "不支持 go 的平台: {o}/{a}"
            )));
        }
    };
    let filename = format!("go{version}.{os_name}-{a_name}.{ext}");
    let url = format!("https://go.dev/dl/{filename}");

    Ok(CommunityRecipe {
        name: format!("go@{version}"),
        version: version.to_string(),
        description: format!("Go {version}"),
        license: Some("BSD-3-Clause".into()),
        license_url: Some("https://go.dev/LICENSE".into()),
        platforms: Platforms {
            windows: if os == "win" {
                Some(PlatformConfig {
                    url: url.clone(),
                    sha256: String::new(),
                    install_type: install_type.to_string(),
                    install_args: None,
                    install_path: format!("go-{version}"),
                    path_add: vec![format!("{{{{install_dir}}}}/go/bin")],
                })
            } else {
                None
            },
            linux: if os == "linux" {
                Some(PlatformConfig {
                    url: url.clone(),
                    sha256: String::new(),
                    install_type: install_type.to_string(),
                    install_args: None,
                    install_path: format!("go-{version}"),
                    path_add: vec![format!("{{{{install_dir}}}}/go/bin")],
                })
            } else {
                None
            },
            darwin: if os == "darwin" {
                Some(PlatformConfig {
                    url: url.clone(),
                    sha256: String::new(),
                    install_type: install_type.to_string(),
                    install_args: None,
                    install_path: format!("go-{version}"),
                    path_add: vec![format!("{{{{install_dir}}}}/go/bin")],
                })
            } else {
                None
            },
        },
        post_install: None,
        depends: None,
        tags: Some(vec!["runtime".into(), "go".into()]),
        maintainer: None,
    })
}

/// python — Windows embeddable zip + pip bootstrap (mirrors the builtin
/// python3.11 recipe). Linux/macOS dynamic python is not supported yet
/// (needs build toolchain).
async fn python_recipe(version: &str) -> Result<CommunityRecipe> {
    if !cfg!(target_os = "windows") {
        return Err(CoreError::Other(
            "动态 python 安装目前仅支持 Windows（embeddable 包）；\
             其他平台请使用内置 python3.11"
                .into(),
        ));
    }
    let arch = std::env::consts::ARCH;
    let a = if arch == "aarch64" { "arm64" } else { "amd64" };
    let filename = format!("python-{version}-embed-{a}.zip");
    let url = format!("https://www.python.org/ftp/python/{version}/{filename}");

    // major.minor without dots ("3.11.9" → "311"), used for the ._pth file
    let short_version: String = version.split('.').take(2).collect::<Vec<_>>().join("");
    // pip bootstrap (same semantics as the builtin python3.11 recipe):
    // 1) uncomment `import site` in the ._pth — pip needs site-packages
    // 2) download + run get-pip.py from the official PyPI bootstrap source
    let uncomment_site = format!(
        "powershell -NoProfile -Command \"(Get-Content '{{{{install_dir}}}}\\python{s}.pth') \
         -replace '^#import site','import site' | Set-Content '{{{{install_dir}}}}\\python{s}.pth'\"",
        s = short_version,
    );
    let bootstrap_pip =
        "curl -fsSL https://bootstrap.pypa.io/get-pip.py -o \"{{install_dir}}\\get-pip.py\" \
         && \"{{install_dir}}\\python.exe\" \"{{install_dir}}\\get-pip.py\" \
         --index-url https://pypi.org/simple && del \"{{install_dir}}\\get-pip.py\""
            .to_string();

    Ok(CommunityRecipe {
        name: format!("python@{version}"),
        version: version.to_string(),
        description: format!("Python {version} (embeddable + pip)"),
        license: Some("PSF-2.0".into()),
        license_url: Some("https://docs.python.org/3/license.html".into()),
        platforms: Platforms {
            windows: Some(PlatformConfig {
                url: url.clone(),
                sha256: String::new(),
                install_type: "zip_extract".into(),
                install_args: None,
                install_path: format!("python-{version}"),
                path_add: vec![format!("{{{{install_dir}}}}")],
            }),
            linux: None,
            darwin: None,
        },
        post_install: Some(PostInstallConfig {
            env_vars: None,
            config_files: Some(vec![ConfigFile {
                path: "{{install_dir}}/pip.conf".into(),
                template: "[global]\nindex-url = https://pypi.tuna.tsinghua.edu.cn/simple\ntrusted-host = pypi.tuna.tsinghua.edu.cn\n".into(),
            }]),
            commands: Some(vec![uncomment_site, bootstrap_pip]),
        }),
        depends: None,
        tags: Some(vec!["runtime".into(), "python".into()]),
        maintainer: None,
    })
}

/// java — Temurin JDK via the Adoptium API. One `feature_releases` call
/// returns every platform's binary (link + sha256) for the requested line, so
/// checksums are filled directly and need no separate resolver.
async fn java_recipe(version: &str) -> Result<CommunityRecipe> {
    let feature = version.split('.').next().unwrap_or(version);
    let url = format!("https://api.adoptium.net/v3/assets/feature_releases/{feature}/ga");
    let json = fetch_json(&url).await?;
    let releases = json
        .as_array()
        .ok_or_else(|| CoreError::Download("Adoptium API: 返回的不是数组".into()))?;
    let release = java_release_for(version, releases)?;
    let semver = release["version_data"]["semver"]
        .as_str()
        .ok_or_else(|| CoreError::Download("Adoptium API: 发布缺少 semver".into()))?;
    let release_name = release["release_name"].as_str().unwrap_or(semver);
    let binaries = release["binaries"]
        .as_array()
        .ok_or_else(|| CoreError::Download("Adoptium API: 发布缺少二进制列表".into()))?;

    let arch = if std::env::consts::ARCH == "aarch64" {
        "aarch64"
    } else {
        "x64"
    };
    // Temurin archives unpack to a top-level dir named after the release
    // (e.g. `jdk-17.0.20+8/`); the executables live in its `bin/`.
    let install_path = format!("jdk-{semver}");
    let platform_cfg = |os: &str, a: &str| -> Option<PlatformConfig> {
        let b = binaries.iter().find(|b| {
            b["os"].as_str() == Some(os)
                && b["architecture"].as_str() == Some(a)
                && b["image_type"].as_str() == Some("jdk")
        })?;
        let pkg = &b["package"];
        Some(PlatformConfig {
            url: pkg["link"].as_str()?.to_string(),
            sha256: pkg["checksum"].as_str().unwrap_or("").to_string(),
            install_type: if os == "windows" {
                "zip_extract"
            } else {
                "tar_extract"
            }
            .to_string(),
            install_args: None,
            install_path: install_path.clone(),
            path_add: vec![format!("{{{{install_dir}}}}/{release_name}/bin")],
        })
    };

    Ok(CommunityRecipe {
        name: format!("java@{version}"),
        version: semver.to_string(),
        description: format!("Temurin JDK {semver}"),
        license: Some("GPL-2.0-with-classpath-exception".into()),
        license_url: Some("https://adoptium.net/temurin/compatibility/".into()),
        platforms: Platforms {
            windows: platform_cfg("windows", "x64"),
            linux: platform_cfg("linux", arch),
            darwin: platform_cfg("darwin", arch),
        },
        post_install: None,
        depends: None,
        tags: Some(vec!["runtime".into(), "java".into()]),
        maintainer: None,
    })
}

/// Find the release matching a requested java version from a newest-first
/// `feature_releases` list: `"17"` → newest 17.x; `"17.0.20"` → newest
/// 17.0.20; `"17.0.20+8"` → the exact build.
fn java_release_for<'a>(
    version: &str,
    releases: &'a [serde_json::Value],
) -> Result<&'a serde_json::Value> {
    for rel in releases {
        let semver = rel["version_data"]["semver"].as_str().unwrap_or("");
        let matches = if version.contains('+') {
            semver == version
        } else {
            semver.starts_with(version)
        };
        if matches {
            return Ok(rel);
        }
    }
    Err(CoreError::Other(format!(
        "Adoptium (Temurin) 没有版本 '{version}' — \
         试试 `oneinit list versions java`"
    )))
}

/// rust — rustup-init with the requested toolchain. The installer binary is
/// the same for every version (pinned sha256, verified), so only the
/// `--default-toolchain` argument varies. Installing executes the installer,
/// so it is gated by `--allow-exec` (H-4), like the static rust recipe.
fn rust_recipe(version: &str) -> Result<CommunityRecipe> {
    let rustup_args = vec![
        "-y".to_string(),
        "--default-toolchain".to_string(),
        version.to_string(),
        "--profile".to_string(),
        "minimal".to_string(),
    ];
    let command = format!(
        "{{{{install_dir}}}}/rustup-init -y --default-toolchain {version} --profile minimal"
    );
    Ok(CommunityRecipe {
        name: format!("rust@{version}"),
        version: version.to_string(),
        description: format!("Rust toolchain via rustup ({version})"),
        license: Some("MIT OR Apache-2.0".into()),
        license_url: Some("https://github.com/rust-lang/rustup#license".into()),
        platforms: Platforms {
            windows: Some(PlatformConfig {
                url: "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe"
                    .into(),
                sha256: "86478e53f769379d7f0ebfa7c9aa97cb76ca92233f79aa2cc0dbee2efaac73c7".into(),
                install_type: "exe_silent".into(),
                install_args: Some(rustup_args),
                install_path: format!("rust-{version}"),
                path_add: vec!["{{user_home}}/.cargo/bin".into()],
            }),
            linux: Some(PlatformConfig {
                url: "https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init"
                    .into(),
                sha256: "4acc9acc76d5079515b46346a485974457b5a79893cfb01112423c89aeb5aa10".into(),
                install_type: "binary_copy".into(),
                install_args: None,
                install_path: format!("rust-{version}"),
                path_add: vec!["{{user_home}}/.cargo/bin".into()],
            }),
            darwin: Some(PlatformConfig {
                url: "https://static.rust-lang.org/rustup/dist/x86_64-apple-darwin/rustup-init"
                    .into(),
                sha256: "33cf85df9142bc6d29cbc62fa5ca1d4c29622cddb55213a4c1a43c457fb9b2d7".into(),
                install_type: "binary_copy".into(),
                install_args: None,
                install_path: format!("rust-{version}"),
                path_add: vec!["{{user_home}}/.cargo/bin".into()],
            }),
        },
        post_install: Some(PostInstallConfig {
            env_vars: None,
            config_files: Some(vec![ConfigFile {
                path: "{{user_home}}/.cargo/config.toml".into(),
                template: "[source.crates-io]\nreplace-with = \"rsproxy-sparse\"\n\n[source.rsproxy-sparse]\nregistry = \"sparse+https://rsproxy.cn/index/\"\n\n[registries.rsproxy]\nindex = \"https://rsproxy.cn/crates.io-index\"\n\n[net]\ngit-fetch-with-cli = true\n".into(),
            }]),
            commands: Some(vec![command]),
        }),
        depends: None,
        tags: Some(vec!["runtime".into(), "rust".into()]),
        maintainer: None,
    })
}

async fn fetch_json(url: &str) -> Result<serde_json::Value> {
    let client = super::self_update::http_client()?;
    let resp = client
        .get(url)
        .header("User-Agent", "oneinit-dynamic-recipe")
        .send()
        .await
        .map_err(|e| CoreError::Download(format!("GET {url} failed: {e}")))?;
    if !resp.status().is_success() {
        return Err(CoreError::Download(format!(
            "GET {url} -> HTTP {}",
            resp.status()
        )));
    }
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| CoreError::Download(format!("read {url} failed: {e}")))?;
    serde_json::from_slice(&bytes)
        .map_err(|e| CoreError::Download(format!("解析 {url} 失败: {e}")))
}

fn checksum_os() -> &'static str {
    if cfg!(target_os = "windows") {
        "win"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else {
        "linux"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn releases() -> Vec<serde_json::Value> {
        serde_json::from_str(
            r#"[
            {"release_name": "jdk-17.0.20+8", "version_data": {"semver": "17.0.20+8"}},
            {"release_name": "jdk-17.0.19+7", "version_data": {"semver": "17.0.19+7"}},
            {"release_name": "jdk-17.0.18+5", "version_data": {"semver": "17.0.18+5"}}
          ]"#,
        )
        .unwrap()
    }

    #[test]
    fn test_java_release_for() {
        let r = releases();
        // feature line → newest of the line
        assert_eq!(
            java_release_for("17", &r).unwrap()["release_name"],
            "jdk-17.0.20+8"
        );
        // partial patch → newest matching patch
        assert_eq!(
            java_release_for("17.0.19", &r).unwrap()["release_name"],
            "jdk-17.0.19+7"
        );
        // exact build
        assert_eq!(
            java_release_for("17.0.20+8", &r).unwrap()["release_name"],
            "jdk-17.0.20+8"
        );
        // unknown → error
        assert!(java_release_for("18", &r).is_err());
        assert!(java_release_for("17.0.21", &r).is_err());
    }

    #[test]
    fn test_rust_recipe_exec_gated() {
        // rust installs by executing rustup-init → must be exec-gated
        let r = rust_recipe("1.82.0").unwrap();
        assert_eq!(
            r.platforms.windows.as_ref().unwrap().install_type,
            "exe_silent"
        );
        assert!(r.post_install.as_ref().unwrap().commands.is_some());
        assert!(r.platforms.windows.as_ref().unwrap().sha256.len() == 64);
    }
}
