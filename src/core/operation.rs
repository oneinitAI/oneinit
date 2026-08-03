//! Atomic operation model for plan → execute (dry-run preview).
//!
//! Destructive commands (install/uninstall/init/sync/team sync) build an
//! [`OperationPlan`] first. With `--dry-run` the plan is rendered and the
//! command exits without executing anything; otherwise [`crate::core::planner::execute_plan`]
//! runs the operations in order.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A single atomic, user-visible operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Operation {
    /// Download a file (optionally verify its checksum after download)
    Download {
        url: String,
        dest: PathBuf,
        size: Option<u64>,
        sha256: Option<String>,
    },
    /// Extract an archive into a directory
    Extract { source: PathBuf, dest: PathBuf },
    /// Create a directory
    CreateDir { path: PathBuf, mode: u32 },
    /// Write a file (create or overwrite)
    WriteFile {
        path: PathBuf,
        content: String,
        overwrite: bool,
    },
    /// Append content to a file
    AppendToFile { path: PathBuf, content: String },
    /// Delete a file or directory
    Delete { path: PathBuf, recursive: bool },
    /// Run a program
    Exec {
        cmd: String,
        args: Vec<String>,
        cwd: Option<PathBuf>,
    },
    /// Set an environment variable (persisted to user profile)
    SetEnv { key: String, value: String },
    /// Remove an environment variable from the user profile
    UnsetEnv { key: String },
    /// Run a shell script (user-declared hook)
    ShellCommand { script: String, shell: String },
    /// Add a directory to the user PATH (via path_mgr)
    PathAdd { dir: PathBuf },
    /// Remove a directory from the user PATH (via path_mgr)
    PathRemove { dir: PathBuf },
    /// Copy a file (binary_copy installs, post-install scripts)
    CopyFile { from: PathBuf, to: PathBuf },
    /// Read a text file, apply a modification, write it back
    ModifyFile {
        path: PathBuf,
        action: crate::core::recipe::ModifyAction,
    },
}

impl Operation {
    /// Human-readable one-line description of this operation.
    pub fn describe(&self) -> String {
        match self {
            Operation::Download {
                url, dest, size, ..
            } => {
                let sz = size
                    .map(|s| format!(" ({})", crate::core::viz::human_bytes(s)))
                    .unwrap_or_default();
                format!("📥 Download {url} → {}{sz}", dest.display())
            }
            Operation::Extract { source, dest } => {
                format!("📦 Extract {} → {}", source.display(), dest.display())
            }
            Operation::CreateDir { path, mode } => {
                format!("📁 Create dir {} (mode {:#o})", path.display(), mode)
            }
            Operation::WriteFile {
                path,
                content,
                overwrite,
            } => {
                let action = if *overwrite { "write" } else { "create" };
                format!(
                    "📝 {action} file {} ({} bytes)",
                    path.display(),
                    content.len()
                )
            }
            Operation::AppendToFile { path, .. } => {
                format!("📎 Append to {}", path.display())
            }
            Operation::Delete { path, recursive } => {
                let flag = if *recursive { " (recursive)" } else { "" };
                format!("🗑️  Delete {}{flag}", path.display())
            }
            Operation::Exec { cmd, args, cwd } => {
                let cwd_str = cwd
                    .as_ref()
                    .map(|p| format!(" (in {})", p.display()))
                    .unwrap_or_default();
                let arg_str = if args.is_empty() {
                    String::new()
                } else {
                    format!(" {}", args.join(" "))
                };
                format!("▶️  Run {cmd}{arg_str}{cwd_str}")
            }
            Operation::SetEnv { key, value } => {
                format!("🔧 Set env {key} = {value}")
            }
            Operation::UnsetEnv { key } => {
                format!("🔧 Unset env {key}")
            }
            Operation::ShellCommand { script, shell } => {
                format!("📜 Run {shell} script ({} lines)", script.lines().count())
            }
            Operation::PathAdd { dir } => {
                format!("➕ PATH += {}", dir.display())
            }
            Operation::PathRemove { dir } => {
                format!("➖ PATH -= {}", dir.display())
            }
            Operation::CopyFile { from, to } => {
                format!("📋 Copy {} → {}", from.display(), to.display())
            }
            Operation::ModifyFile { path, .. } => {
                format!("✏️  Modify file {}", path.display())
            }
        }
    }
}

/// Summary counters over an operation plan.
#[derive(Debug, Clone, Default, Serialize)]
pub struct PlanSummary {
    pub total_ops: usize,
    pub downloads: usize,
    pub extracts: usize,
    pub dirs_created: usize,
    pub files_written: usize,
    pub files_deleted: usize,
    pub env_changes: usize,
    pub execs: usize,
}

/// A set of operations produced by a planner.
#[derive(Debug, Clone, Default)]
pub struct OperationPlan {
    pub operations: Vec<Operation>,
    pub summary: PlanSummary,
}

impl OperationPlan {
    /// Build the plan and compute its summary.
    pub fn new(operations: Vec<Operation>) -> Self {
        let summary = Self::summarize(&operations);
        OperationPlan {
            operations,
            summary,
        }
    }

    fn summarize(ops: &[Operation]) -> PlanSummary {
        let mut s = PlanSummary {
            total_ops: ops.len(),
            ..PlanSummary::default()
        };
        for op in ops {
            match op {
                Operation::Download { .. } => s.downloads += 1,
                Operation::Extract { .. } => s.extracts += 1,
                Operation::CreateDir { .. } => s.dirs_created += 1,
                Operation::WriteFile { .. }
                | Operation::AppendToFile { .. }
                | Operation::CopyFile { .. }
                | Operation::ModifyFile { .. } => s.files_written += 1,
                Operation::Delete { .. } => s.files_deleted += 1,
                Operation::SetEnv { .. } | Operation::UnsetEnv { .. } => s.env_changes += 1,
                Operation::Exec { .. } | Operation::ShellCommand { .. } => s.execs += 1,
                _ => {}
            }
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_describe() {
        let op = Operation::Download {
            url: "https://example.com/a.tgz".into(),
            dest: PathBuf::from("/tmp/a.tgz"),
            size: Some(1024),
            sha256: None,
        };
        assert!(op.describe().contains("https://example.com/a.tgz"));
        assert!(op.describe().contains("1.0 KB"));

        let del = Operation::Delete {
            path: PathBuf::from("/tmp/x"),
            recursive: true,
        };
        assert!(del.describe().contains("recursive"));
    }

    #[test]
    fn test_plan_summary() {
        let ops = vec![
            Operation::CreateDir {
                path: PathBuf::from("/x"),
                mode: 0o755,
            },
            Operation::Download {
                url: "u".into(),
                dest: PathBuf::from("/x/a"),
                size: None,
                sha256: None,
            },
            Operation::SetEnv {
                key: "K".into(),
                value: "V".into(),
            },
        ];
        let plan = OperationPlan::new(ops);
        assert_eq!(plan.summary.total_ops, 3);
        assert_eq!(plan.summary.dirs_created, 1);
        assert_eq!(plan.summary.downloads, 1);
        assert_eq!(plan.summary.env_changes, 1);
    }
}
