// 安全提示与免责声明模块
//
// 在执行高风险操作前调用 print_disclaimer() 向用户展示安全信息。
// 所有安全提示使用 [SECURITY] 前缀，方便用户识别。

use crate::output::OutputFormatter;

/// 项目版本号
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 免责声明（简短版，安装/导入时显示）
pub const DISCLAIMER_SHORT: &str = "\
[SECURITY] ============================================
[SECURITY] OneInit 安全提示
[SECURITY]
[SECURITY] OneInit 将从网络下载文件、修改 PATH 环境变量、
[SECURITY] 写入配置文件、并可能执行安装脚本。
[SECURITY]
[SECURITY] 请确认你信任配方的来源和下载地址。
[SECURITY] 社区配方未经审计，使用风险自负。
[SECURITY] ============================================";

/// 免责声明（完整版，首次运行时显示）
pub const DISCLAIMER_FULL: &str = "\
[SECURITY] ============================================
[SECURITY] OneInit v{ver} 安全与免责声明
[SECURITY]
[SECURITY] OneInit 是开源软件 (GPL-3.0)，不提供任何担保。
[SECURITY] 使用 OneInit 安装的工具和配置由用户自行承担风险。
[SECURITY]
[SECURITY] OneInit 会执行以下操作:
[SECURITY]   1. 从网络下载文件到 ~/.oneinit/ 目录
[SECURITY]   2. 修改系统 PATH 环境变量
[SECURITY]   3. 写入配置文件 (如 pip.conf, .npmrc)
[SECURITY]   4. 可能执行安装脚本 (如 get-pip.py)
[SECURITY]
[SECURITY] 安全建议:
[SECURITY]   - 仅安装你信任来源的配方
[SECURITY]   - 安装前查看 [SECURITY] 提示的下载地址和命令
[SECURITY]   - 使用 oneinit verify 验证社区配方
[SECURITY]   - 定期使用 oneinit doctor 检查环境健康
[SECURITY]
[SECURITY] 完整许可证: https://www.gnu.org/licenses/gpl-3.0.html
[SECURITY] ============================================";

/// 打印简短免责声明
pub fn print_disclaimer(formatter: &OutputFormatter) {
    for line in DISCLAIMER_SHORT.lines() {
        formatter.output(line, Some(serde_json::Value::Null));
    }
}

/// 打印完整免责声明（首次运行时）
pub fn print_full_disclaimer(formatter: &OutputFormatter) {
    let text = DISCLAIMER_FULL.replace("{ver}", VERSION);
    for line in text.lines() {
        formatter.output(line, Some(serde_json::Value::Null));
    }
}
