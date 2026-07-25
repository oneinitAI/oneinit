#![allow(dead_code)]

mod cli;
mod core;
mod output;
mod security;
mod skill_mgr;
mod tui2;

use clap::{Parser, Subcommand};

/// OneInit — One command to init your dev machine
///
/// The first tool to install on a new machine.
/// Developer-ready after one command.
#[derive(Parser)]
#[command(name = "oneinit", version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Output in JSON format (AI-friendly)
    #[arg(
        global = true,
        long = "json",
        help = "Output in JSON format (AI-friendly)"
    )]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize dev environment with presets
    Init {
        /// 预置套装名称（如 "python", "frontend", "ai"）
        #[arg(short, long)]
        preset: Option<String>,
    },

    /// Install a tool（如 python3.7, node18）
    Install {
        /// 要安装的工具包名称
        package: String,
    },

    /// Uninstall a tool
    Uninstall {
        /// 要卸载的工具包名称
        package: String,
    },

    /// List installed tools
    List,

    /// Search available tools
    Search {
        /// 搜索关键词
        keyword: Option<String>,
    },

    /// Sync from oneinit.yaml
    Sync,

    /// Capture environment to oneinit.yaml
    Capture {
        /// 输出文件路径（默认 oneinit.yaml）
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Validate community recipe YAML
    Verify {
        /// recipe文件路径
        file: String,
    },

    /// Update remote recipe index (like apt update)
    Update,

    /// Publish recipe to remote registry
    Publish {
        /// recipe文件路径
        file: String,
    },

    /// Export environment as tar.gz
    Export {
        /// 输出文件路径（默认 oneinit-backup.tar.gz）
        #[arg(short, long, default_value = "oneinit-backup.tar.gz")]
        output: String,
        /// 包含已安装的工具目录（~/.oneinit/envs/）
        #[arg(long)]
        include_envs: bool,
    },

    /// 从 tar.gz 包导入环境
    Import {
        /// 备份文件路径
        file: String,
        /// 只预览不实际执行
        #[arg(long)]
        dry_run: bool,
        /// 强制覆盖已exists的文件
        #[arg(long)]
        force: bool,
    },

    /// Launch interactive TUI
    Tui,

    /// Environment health check（PATH 残留、清单漂移、磁盘空间）
    Doctor,

    /// Export installed tools (like pip freeze)
    Freeze {
        /// 输出文件路径（默认 oneinit.yaml）
        #[arg(short, long, default_value = "oneinit.yaml")]
        output: String,
    },

    /// Generate shell completion script
    Completions {
        /// Shell 类型（bash, zsh, powershell, fish, elvish）
        shell: String,
    },

    /// Install/manage AI Skill
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },
}

#[derive(clap::Subcommand)]
enum SkillAction {
    /// Install Skill to detected AI agents
    Install {
        /// 指定 AI 助手（zcode, codex, claude, agents, all）。默认 all
        #[arg(short, long, default_value = "all")]
        target: String,
    },
    /// Show Skill installation status
    Status,
    /// Uninstall Skill
    Uninstall,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let formatter = output::OutputFormatter::new(cli.json);

    // 高风险操作前显示免责声明
    let is_dangerous = matches!(
        cli.command,
        Commands::Install { .. } | Commands::Sync | Commands::Import { dry_run: false, .. }
    );
    if is_dangerous {
        security::print_disclaimer(&formatter);
    }

    match cli.command {
        Commands::Init { preset } => cli::run_init(&formatter, preset.as_deref()).await,
        Commands::Install { package } => cli::run_install(&formatter, &package).await,
        Commands::Uninstall { package } => cli::run_uninstall(&formatter, &package).await,
        Commands::List => cli::run_list(&formatter).await,
        Commands::Search { keyword } => cli::run_search(&formatter, keyword.as_deref()).await,
        Commands::Sync => cli::run_sync(&formatter).await,
        Commands::Capture { output } => {
            cli::run_capture(&formatter, output.as_deref().unwrap_or("oneinit.yaml")).await
        }
        Commands::Verify { file } => cli::run_verify(&formatter, &file).await,
        Commands::Update => cli::run_update(&formatter).await,
        Commands::Publish { file } => cli::run_publish(&formatter, &file).await,
        Commands::Export {
            output,
            include_envs,
        } => cli::run_export(&formatter, &output, include_envs).await,
        Commands::Import {
            file,
            dry_run,
            force,
        } => cli::run_import(&formatter, &file, dry_run, force).await,
        Commands::Tui => {
            if let Err(e) = tui2::run_tui(&formatter).await {
                eprintln!("TUI error: {}", e);
            }
        }
        Commands::Doctor => cli::run_doctor(&formatter).await,
        Commands::Freeze { output } => cli::run_freeze(&formatter, &output).await,
        Commands::Completions { shell } => {
            use clap::CommandFactory;
            let mut cmd = Cli::command();
            let shell_name = shell.as_str();
            let shell = match shell_name {
                "bash" => clap_complete::Shell::Bash,
                "zsh" => clap_complete::Shell::Zsh,
                "powershell" => clap_complete::Shell::PowerShell,
                "fish" => clap_complete::Shell::Fish,
                "elvish" => clap_complete::Shell::Elvish,
                other => {
                    eprintln!(
                        "[ERROR] 不支持的 shell: {} (支持: bash, zsh, powershell, fish, elvish)",
                        other
                    );
                    return;
                }
            };
            clap_complete::generate(shell, &mut cmd, "oneinit", &mut std::io::stdout());
        }
        Commands::Skill { action } => match action {
            SkillAction::Install { target } => cli::run_skill_install(&formatter, &target).await,
            SkillAction::Status => cli::run_skill_status(&formatter).await,
            SkillAction::Uninstall => cli::run_skill_uninstall(&formatter).await,
        },
    }
}
