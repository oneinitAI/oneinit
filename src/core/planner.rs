//! Operation planners: turn recipes / install records into an
//! [`OperationPlan`] for preview (--dry-run) and for execution.
//!
//! The plan is the single source of truth for *what will happen*: the same
//! plan is rendered for `--dry-run` and executed by [`execute_plan`].

use std::collections::BTreeMap;
use std::path::PathBuf;

use super::operation::{Operation, OperationPlan};
use super::recipe::Recipe;
use super::{CoreError, Result, envs_dir, temp_dir};

/// Build a plan for installing a builtin recipe (mirrors `recipe::install`).
pub fn plan_builtin_install(recipe: &Recipe) -> Result<OperationPlan> {
    let install_dir = envs_dir().join(&recipe.name);
    let archive_name = recipe.download_url.rsplit('/').next().unwrap_or("archive");
    let temp_archive = temp_dir().join(archive_name);
    let bin_path = install_dir.join(&recipe.bin_dir);

    let mut ops = vec![Operation::CreateDir {
        path: install_dir.clone(),
        mode: 0o755,
    }];
    ops.push(Operation::Download {
        url: recipe.download_url.clone(),
        dest: temp_archive.clone(),
        size: None,
        sha256: Some(recipe.sha256.clone()),
    });
    ops.push(Operation::Extract {
        source: temp_archive.clone(),
        dest: install_dir.clone(),
    });
    ops.push(Operation::Delete {
        path: temp_archive,
        recursive: false,
    });

    // config files
    for cfg in &recipe.configs {
        ops.push(Operation::WriteFile {
            path: install_dir.join(&cfg.rel_path),
            content: cfg.content.clone(),
            overwrite: true,
        });
    }

    // env vars (builtins currently declare none; this keeps the field alive)
    for (k, v) in &recipe.env_vars {
        ops.push(Operation::SetEnv {
            key: k.clone(),
            value: v.clone(),
        });
    }

    // post-install steps
    if let Some(post) = &recipe.post_install {
        for step in &post.steps {
            match step {
                super::recipe::PostInstallStep::DownloadAndRun { url, args } => {
                    let file_name = url.rsplit('/').next().unwrap_or("script");
                    let dest = install_dir.join(file_name);
                    ops.push(Operation::Download {
                        url: url.clone(),
                        dest: dest.clone(),
                        size: None,
                        sha256: None,
                    });
                    // Same semantics as execute_download_and_run: run with the
                    // installed python executable.
                    let mut run_args = vec![file_name.to_string()];
                    run_args.extend(args.clone());
                    ops.push(Operation::Exec {
                        cmd: install_dir.join("python.exe").to_string_lossy().to_string(),
                        args: run_args,
                        cwd: Some(install_dir.clone()),
                    });
                    ops.push(Operation::Delete {
                        path: dest,
                        recursive: false,
                    });
                }
                super::recipe::PostInstallStep::ModifyFile { rel_path, action } => {
                    ops.push(Operation::ModifyFile {
                        path: install_dir.join(rel_path),
                        action: action.clone(),
                    });
                }
            }
        }
    }

    ops.push(Operation::PathAdd { dir: bin_path });
    Ok(OperationPlan::new(ops))
}

/// Build a plan for installing a community recipe (mirrors
/// `community_recipe::install`). Applies the H-4 exec gate up front so that
/// `--dry-run` refuses exec-requiring recipes exactly like a real install.
pub fn plan_community_install(
    recipe: &super::community_recipe::CommunityRecipe,
    allow_exec: bool,
) -> Result<OperationPlan> {
    let platform_cfg =
        super::community_recipe::current_platform_config(recipe).ok_or_else(|| {
            CoreError::Other(format!(
                "recipe '{}' unsupported on this platform",
                recipe.name
            ))
        })?;

    // Security H-4: refuse recipes that require executing commands/installers
    let has_commands = recipe
        .post_install
        .as_ref()
        .and_then(|p| p.commands.as_ref())
        .map(|c| !c.is_empty())
        .unwrap_or(false);
    let exec_type = matches!(
        platform_cfg.install_type.as_str(),
        "exe_silent" | "msi_install" | "pkg_install"
    );
    let needs_exec = has_commands || exec_type;
    if needs_exec && !allow_exec {
        return Err(CoreError::Other(format!(
            "recipe '{}' requires executing commands/installers ({}{}). \
             Refused for security. Re-run with --allow-exec to accept.",
            recipe.name,
            if has_commands {
                "post_install commands"
            } else {
                ""
            },
            if exec_type {
                " install_type=".to_string() + &platform_cfg.install_type
            } else {
                String::new()
            },
        )));
    }

    let install_dir = envs_dir().join(&platform_cfg.install_path);
    let archive_name = platform_cfg.url.rsplit('/').next().unwrap_or("archive");
    let temp_archive = temp_dir().join(archive_name);

    let mut ops = Vec::new();
    ops.push(Operation::CreateDir {
        path: install_dir.clone(),
        mode: 0o755,
    });
    ops.push(Operation::Download {
        url: platform_cfg.url.clone(),
        dest: temp_archive.clone(),
        size: None,
        sha256: Some(platform_cfg.sha256.clone()),
    });

    // install_type dispatch
    match platform_cfg.install_type.as_str() {
        "zip_extract" | "tar_extract" => ops.push(Operation::Extract {
            source: temp_archive.clone(),
            dest: install_dir.clone(),
        }),
        "exe_silent" => {
            let args = platform_cfg.install_args.clone().unwrap_or_default();
            ops.push(Operation::Exec {
                cmd: temp_archive.to_string_lossy().to_string(),
                args,
                cwd: Some(install_dir.clone()),
            });
        }
        "binary_copy" => ops.push(Operation::CopyFile {
            from: temp_archive.clone(),
            to: install_dir.join(archive_name),
        }),
        "msi_install" => {
            let args = platform_cfg
                .install_args
                .clone()
                .unwrap_or_else(|| vec!["/qn".to_string(), "/norestart".to_string()]);
            let mut msiexec_args =
                vec!["/i".to_string(), temp_archive.to_string_lossy().to_string()];
            msiexec_args.extend(args);
            ops.push(Operation::Exec {
                cmd: "msiexec".to_string(),
                args: msiexec_args,
                cwd: None,
            });
        }
        "pkg_install" => {
            let home = dirs::home_dir()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_else(|| "/".to_string());
            ops.push(Operation::Exec {
                cmd: "installer".to_string(),
                args: vec![
                    "-pkg".to_string(),
                    temp_archive.to_string_lossy().to_string(),
                    "-target".to_string(),
                    home,
                ],
                cwd: None,
            });
        }
        other => {
            return Err(CoreError::Other(format!(
                "install_type '{}' 暂不支持（当前支持: zip_extract, tar_extract, exe_silent, binary_copy, msi_install, pkg_install）",
                other
            )));
        }
    }
    ops.push(Operation::Delete {
        path: temp_archive,
        recursive: false,
    });

    // post-install: config files, env vars, commands
    if let Some(post) = &recipe.post_install {
        if let Some(configs) = &post.config_files {
            for cf in configs {
                let path = super::community_recipe::render_template(&cf.path, &install_dir);
                let content = super::community_recipe::render_template(&cf.template, &install_dir);
                ops.push(Operation::WriteFile {
                    path: PathBuf::from(path),
                    content,
                    overwrite: true,
                });
            }
        }
        if let Some(env_vars) = &post.env_vars {
            for (k, v) in env_vars {
                ops.push(Operation::SetEnv {
                    key: k.clone(),
                    value: v.clone(),
                });
            }
        }
        if let Some(commands) = &post.commands {
            for cmd in commands {
                let script = super::community_recipe::render_template(cmd, &install_dir);
                ops.push(Operation::ShellCommand {
                    script,
                    shell: default_shell(),
                });
            }
        }
    }

    // PATH entries
    for path_template in &platform_cfg.path_add {
        let rendered = super::community_recipe::render_template(path_template, &install_dir);
        ops.push(Operation::PathAdd {
            dir: PathBuf::from(rendered),
        });
    }

    Ok(OperationPlan::new(ops))
}

/// Build a plan for uninstalling a tool from its manifest record.
pub fn plan_uninstall(record: &super::manifest::InstallRecord) -> OperationPlan {
    let mut ops = Vec::new();
    for entry in &record.path_entries {
        ops.push(Operation::PathRemove {
            dir: PathBuf::from(entry),
        });
    }
    ops.push(Operation::Delete {
        path: PathBuf::from(&record.install_path),
        recursive: true,
    });
    OperationPlan::new(ops)
}

/// Default shell name for ShellCommand ops.
pub fn default_shell() -> String {
    if cfg!(target_os = "windows") {
        "cmd".to_string()
    } else {
        "sh".to_string()
    }
}

/// Render a plan as human-readable lines + summary (used by --dry-run and the
/// pre-install preview). Returns the rendered string.
pub fn render_plan(plan: &OperationPlan, title: &str) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        "🔍 {title} — operations that will be executed:\n\n"
    ));
    for (i, op) in plan.operations.iter().enumerate() {
        s.push_str(&format!("  {}. {}\n", i + 1, op.describe()));
    }
    let sm = &plan.summary;
    s.push_str(&format!("\n📊 Total: {} operations\n", sm.total_ops));
    s.push_str(&format!("   ├── 📥 download: {}\n", sm.downloads));
    s.push_str(&format!("   ├── 📦 extract: {}\n", sm.extracts));
    s.push_str(&format!("   ├── 📁 create dir: {}\n", sm.dirs_created));
    s.push_str(&format!("   ├── 📝 files written: {}\n", sm.files_written));
    s.push_str(&format!("   ├── 🗑️  delete: {}\n", sm.files_deleted));
    s.push_str(&format!("   ├── 🔧 env changes: {}\n", sm.env_changes));
    s.push_str(&format!("   └── ▶️  run scripts: {}\n", sm.execs));
    s
}

/// Execute all operations in a plan, in order. Stops at the first failure.
pub async fn execute_plan(
    plan: &OperationPlan,
    formatter: &crate::output::OutputFormatter,
) -> Result<()> {
    for op in &plan.operations {
        match op {
            Operation::Download {
                url, dest, sha256, ..
            } => {
                formatter.debug_line(&format!("download {url} → {}", dest.display()));
                let dl = super::downloader::download(url, dest).await?;
                if let Some(expected) = sha256 {
                    super::downloader::verify_sha256(dest, expected)?;
                    formatter.output(
                        &format!("[OK] SHA256 verified: {}", dest.display()),
                        None::<serde_json::Value>,
                    );
                } else {
                    formatter.output(
                        &format!(
                            "[OK] downloaded: {} ({} bytes)",
                            dest.display(),
                            dl.file_size
                        ),
                        None::<serde_json::Value>,
                    );
                }
            }
            Operation::Extract { source, dest } => {
                let files = super::downloader::extract(source, dest)?;
                formatter.output(
                    &format!("[OK] extracted {} files → {}", files.len(), dest.display()),
                    None::<serde_json::Value>,
                );
            }
            Operation::CreateDir { path, .. } => {
                std::fs::create_dir_all(path)?;
            }
            Operation::WriteFile {
                path,
                content,
                overwrite: _,
            } => {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(path, content)?;
                formatter.output(
                    &format!("[OK] wrote {}", path.display()),
                    None::<serde_json::Value>,
                );
            }
            Operation::AppendToFile { path, content } => {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                use std::io::Write;
                let mut f = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?;
                f.write_all(content.as_bytes())?;
            }
            Operation::Delete { path, recursive } => {
                if path.exists() {
                    if *recursive {
                        std::fs::remove_dir_all(path)?;
                    } else {
                        std::fs::remove_file(path)?;
                    }
                }
            }
            Operation::Exec { cmd, args, cwd } => {
                formatter.debug_line(&format!("exec {cmd} {}", args.join(" ")));
                let mut c = std::process::Command::new(cmd);
                c.args(args);
                if let Some(d) = cwd {
                    c.current_dir(d);
                }
                let status = c
                    .status()
                    .map_err(|e| CoreError::Other(format!("exec {cmd} failed: {e}")))?;
                if !status.success() {
                    return Err(CoreError::Other(format!(
                        "command exited with {:?}",
                        status.code()
                    )));
                }
            }
            Operation::SetEnv { key, value } => {
                let mut vars = BTreeMap::new();
                vars.insert(key.clone(), value.clone());
                super::team::apply_env_vars(&vars, formatter)?;
            }
            Operation::UnsetEnv { key } => {
                // Best-effort: nothing generates this today; keep a debug note.
                formatter.debug_line(&format!("unset env {key} (no-op)"));
            }
            Operation::ShellCommand { script, shell: _ } => {
                super::sync::run_post_install(std::slice::from_ref(script), formatter)?;
            }
            Operation::PathAdd { dir } => {
                super::path_mgr::add(dir)?;
            }
            Operation::PathRemove { dir } => {
                super::path_mgr::remove(dir)?;
            }
            Operation::CopyFile { from, to } => {
                if let Some(parent) = to.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(from, to)?;
            }
            Operation::ModifyFile { path, action } => {
                let content = std::fs::read_to_string(path)?;
                let new_content = super::recipe::apply_modify_action(action, &content);
                std::fs::write(path, new_content)?;
                formatter.debug_line(&format!("modified {}", path.display()));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_builtin_has_download_extract_pathadd() {
        let recipe = super::super::recipe::resolve("node20").expect("node20 recipe");
        let plan = plan_builtin_install(&recipe).unwrap();
        let kinds: Vec<&str> = plan
            .operations
            .iter()
            .map(|op| match op {
                Operation::Download { .. } => "download",
                Operation::Extract { .. } => "extract",
                Operation::PathAdd { .. } => "pathadd",
                _ => "other",
            })
            .collect();
        assert!(kinds.contains(&"download"));
        assert!(kinds.contains(&"extract"));
        assert!(kinds.contains(&"pathadd"));
        assert!(plan.summary.total_ops >= 4);
    }

    #[test]
    fn test_plan_community_exec_gate() {
        let yaml = r#"
name: test-tool
version: "1.0.0"
description: "A test"
platforms:
  windows:
    url: "https://example.com/tool.zip"
    sha256: "0000000000000000000000000000000000000000000000000000000000000000"
    install_type: "exe_silent"
    install_path: "test"
    path_add: ["{{install_dir}}"]
"#;
        let recipe: super::super::community_recipe::CommunityRecipe =
            serde_yaml::from_str(yaml).unwrap();
        // exec-type recipe denied without --allow-exec
        assert!(plan_community_install(&recipe, false).is_err());
        // allowed with --allow-exec
        assert!(plan_community_install(&recipe, true).is_ok());
    }

    #[test]
    fn test_plan_uninstall_paths() {
        let record = super::super::manifest::InstallRecord {
            id: "x".into(),
            name: "t".into(),
            version: Some("1".into()),
            install_path: "/tmp/t".into(),
            archive_url: None,
            sha256: None,
            path_entries: vec!["/tmp/t/bin".into()],
            config_files: vec![],
            installed_at: String::new(),
            original_path: None,
            env_vars_backup: serde_json::json!({}),
        };
        let plan = plan_uninstall(&record);
        assert_eq!(plan.summary.files_deleted, 1);
        assert!(
            plan.operations
                .iter()
                .any(|op| matches!(op, Operation::PathRemove { .. }))
        );
    }

    #[test]
    fn test_render_plan_contains_summary() {
        let recipe = super::super::recipe::resolve("node20").expect("node20 recipe");
        let plan = plan_builtin_install(&recipe).unwrap();
        let text = render_plan(&plan, "Install node20");
        assert!(text.contains("Install node20"));
        assert!(text.contains("📊 Total:"));
    }
}
