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
        other => {
            return Err(CoreError::Other(format!(
                "dynamic install not supported for '{other}' yet"
            )));
        }
    };

    // Attach checksums (per platform), verifying against official sources.
    let mut out = recipe;

    // checksums
    for platform in ["windows", "linux", "darwin"] {
        let Some(cfg) = platform_cfg_mut(&mut out, platform) else {
            continue;
        };
        let filename = cfg.url.rsplit('/').next().unwrap_or("").to_string();
        let hash = super::checksum::resolve(family, version, &filename, refresh).await?;
        match hash {
            Some(h) => cfg.sha256 = h,
            None if no_checksum => {
                cfg.sha256 = String::new(); // planner treats empty as "skip verify"
            }
            None => {
                return Err(CoreError::Other(format!(
                    "cannot resolve checksum for {family}@{version} ({filename}). \
                     Re-run with --no-checksum to skip verification (risky)."
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
                "unsupported platform for go: {o}/{a}"
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

/// python — Windows embeddable zip (mirrors the builtin python3.11 recipe).
/// Linux/macOS dynamic python is not supported yet (needs build toolchain).
async fn python_recipe(version: &str) -> Result<CommunityRecipe> {
    if !cfg!(target_os = "windows") {
        return Err(CoreError::Other(
            "dynamic python install currently supports Windows only \
             (embeddable zip); use the builtin python3.11 on other platforms"
                .into(),
        ));
    }
    let arch = std::env::consts::ARCH;
    let a = if arch == "aarch64" { "arm64" } else { "amd64" };
    let filename = format!("python-{version}-embed-{a}.zip");
    let url = format!("https://www.python.org/ftp/python/{version}/{filename}");

    Ok(CommunityRecipe {
        name: format!("python@{version}"),
        version: version.to_string(),
        description: format!("Python {version} (embeddable)"),
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
            commands: None,
        }),
        depends: None,
        tags: Some(vec!["runtime".into(), "python".into()]),
        maintainer: None,
    })
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
