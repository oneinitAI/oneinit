#![allow(dead_code)]

mod cli;
mod core;
mod output;
mod tui2;

use clap::{Parser, Subcommand};

/// OneInit — 一条命令，初始化整台电脑
///
/// 拿到一台新电脑后，第一个要装的工具。
/// 装完它，这台电脑就是开发者就绪的机器。
#[derive(Parser)]
#[command(name = "oneinit", version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// 全局开关：所有命令输出 JSON 格式（AI 模式）
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
    /// 一键初始化开发环境
    Init {
        /// 预置套装名称（如 "python", "frontend", "ai"）
        #[arg(short, long)]
        preset: Option<String>,
    },

    /// 安装指定工具（如 python3.7, node18）
    Install {
        /// 要安装的工具包名称
        package: String,
    },

    /// 卸载指定工具
    Uninstall {
        /// 要卸载的工具包名称
        package: String,
    },

    /// 列出已安装的工具
    List,

    /// 搜索可用工具
    Search {
        /// 搜索关键词
        keyword: Option<String>,
    },

    /// 从 oneinit.yaml 同步环境
    Sync,

    /// 捕获当前环境生成 oneinit.yaml
    Capture {
        /// 输出文件路径（默认 oneinit.yaml）
        #[arg(short, long)]
        output: Option<String>,
    },

    /// 验证社区配方文件
    Verify {
        /// 配方文件路径
        file: String,
    },

    /// 更新远程配方索引（类似 apt update）
    Update,

    /// 发布配方到远程仓库
    Publish {
        /// 配方文件路径
        file: String,
    },

    /// 导出环境为 tar.gz 包
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
        /// 强制覆盖已存在的文件
        #[arg(long)]
        force: bool,
    },

    /// 启动交互式 TUI 界面
    Tui,

    /// 环境健康检查（PATH 残留、清单漂移、磁盘空间）
    Doctor,

    /// 导出已安装工具为 oneinit.yaml（类似 pip freeze）
    Freeze {
        /// 输出文件路径（默认 oneinit.yaml）
        #[arg(short, long, default_value = "oneinit.yaml")]
        output: String,
    },

    /// 生成 shell 自动补全脚本
    Completions {
        /// Shell 类型（bash, zsh, powershell, fish, elvish）
        shell: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let formatter = output::OutputFormatter::new(cli.json);

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
                eprintln!("TUI 错误: {}", e);
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
    }
}
