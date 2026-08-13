use std::path::Path;

use super::{CoreError, Result};

/// 获取当前系统 PATH 环境变量值
fn get_path() -> Result<String> {
    std::env::var("PATH").map_err(|_| CoreError::PathOp("无法读取 PATH 环境变量".to_string()))
}

/// 将目录添加到用户级 PATH
pub fn add(directory: &Path) -> Result<()> {
    let dir_str = directory.to_string_lossy().to_string();
    let current = get_path()?;

    // 检查是否已在 PATH 中
    let parts: Vec<&str> = split_path(&current);
    if parts.iter().any(|p| paths_equal(p, &dir_str)) {
        return Ok(()); // 已exists，跳过
    }

    let new_path = if current.ends_with(';') || current.ends_with(':') {
        format!("{}{}", current, dir_str)
    } else {
        format!("{}{}{}", current, PATH_SEP, dir_str)
    };

    set_path(&new_path)
}

/// 判断目录是否已在当前进程 PATH 中（viz 等只读场景用）
pub fn is_in_path(directory: &Path) -> bool {
    let Ok(current) = get_path() else {
        return false;
    };
    let dir_str = directory.to_string_lossy().to_string();
    split_path(&current)
        .iter()
        .any(|p| paths_equal(p, &dir_str))
}

/// 从 PATH 中移除指定目录
pub fn remove(directory: &Path) -> Result<()> {
    let dir_str = directory.to_string_lossy().to_string();
    let current = get_path()?;

    let parts: Vec<&str> = split_path(&current);
    let filtered: Vec<&str> = parts
        .into_iter()
        .filter(|p| !paths_equal(p, &dir_str))
        .collect();

    let new_path = filtered.join(&PATH_SEP.to_string());
    set_path(&new_path)
}

/// 备份当前 PATH（安装前调用）
pub fn backup() -> Result<String> {
    get_path()
}

/// 恢复 PATH 到备份状态
pub fn restore(backup: &str) -> Result<()> {
    set_path(backup)
}

// 平台常量
#[cfg(target_os = "windows")]
const PATH_SEP: char = ';';

#[cfg(not(target_os = "windows"))]
const PATH_SEP: char = ':';

// 平台特定 PATH 分割
fn split_path(path: &str) -> Vec<&str> {
    path.split(PATH_SEP).collect()
}

// 路径比较（忽略大小写在 Windows）
fn paths_equal(a: &str, b: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        a.trim_matches('\"').to_lowercase() == b.trim_matches('\"').to_lowercase()
    }
    #[cfg(not(target_os = "windows"))]
    {
        a == b
    }
}

// ============================================================
// 平台特定实现：设置 PATH
// ============================================================

/// 将 PATH 写入系统（平台特定）
fn set_path(new_path: &str) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        set_path_windows(new_path)
    }
    #[cfg(not(target_os = "windows"))]
    {
        set_path_unix(new_path)
    }
}

/// Windows: 写入注册表 + 广播 WM_SETTINGCHANGE
///
/// 保持原有值类型（REG_EXPAND_SZ / REG_SZ），避免 `%VAR%` 被字面化；
/// PATH 超过传统 2047 字符上限时给出警告（注册表可存更长，但部分旧程序会截断）。
#[cfg(target_os = "windows")]
fn set_path_windows(new_path: &str) -> Result<()> {
    use winreg::RegKey;
    use winreg::enums::*;

    // 软限制：旧程序按 2047 字符截断 PATH
    const WINDOWS_PATH_LIMIT: usize = 2047;
    if new_path.len() > WINDOWS_PATH_LIMIT {
        eprintln!(
            "[WARN] PATH 长度 {} 超过传统上限 {} 字符 — 部分旧程序可能无法读取完整 PATH",
            new_path.len(),
            WINDOWS_PATH_LIMIT
        );
    }

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .map_err(|e| CoreError::PathOp(format!("cannot open registry: {}", e)))?;

    // 保持原有值类型：PATH 若为 REG_EXPAND_SZ（含 %VAR%），写成 REG_SZ 会导致变量字面化
    let vtype = match env.get_raw_value("PATH") {
        Ok(v) => v.vtype,
        Err(_) => REG_SZ,
    };
    // UTF-16LE 编码 + null 结尾（REG_SZ / REG_EXPAND_SZ 需要）
    let mut bytes: Vec<u8> = new_path
        .encode_utf16()
        .flat_map(|u| u.to_le_bytes())
        .collect();
    bytes.extend_from_slice(&[0u8, 0u8]);
    let value = winreg::RegValue { bytes, vtype };
    env.set_raw_value("PATH", &value)
        .map_err(|e| CoreError::PathOp(format!("cannot write PATH registry: {}", e)))?;

    // 广播 WM_SETTINGCHANGE 通知其他进程
    broadcast_setting_change();

    // SAFETY: 在安装/卸载操作中修改 PATH 是预期行为
    unsafe { std::env::set_var("PATH", new_path) };

    Ok(())
}

/// Windows: 广播 WM_SETTINGCHANGE
///
/// 用 PostMessage（非阻塞）而非 SendMessage：SendMessage 到 HWND_BROADCAST
/// 会在存在不响应窗口时无限阻塞（已知 Windows 陷阱），导致安装/同步卡死。
#[cfg(target_os = "windows")]
fn broadcast_setting_change() {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    unsafe {
        let env: Vec<u16> = OsStr::new("Environment")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let hwnd = winapi::um::winuser::HWND_BROADCAST;
        winapi::um::winuser::PostMessageW(
            hwnd,
            winapi::um::winuser::WM_SETTINGCHANGE,
            0,
            env.as_ptr() as winapi::shared::minwindef::LPARAM,
        );
    }
}

/// Unix: 写入 shell 配置文件
///
/// 只写检测到的默认 shell 对应的 profile（`$SHELL` 决定；缺失时兜底 .bashrc），
/// 避免多 shell 文件内容不一致。
#[cfg(not(target_os = "windows"))]
fn set_path_unix(new_path: &str) -> Result<()> {
    let home = dirs::home_dir()
        .ok_or_else(|| CoreError::PathOp("Cannot determine home directory".to_string()))?;

    let shell = std::env::var("SHELL").unwrap_or_default();
    let shell_lower = shell.to_ascii_lowercase();

    if shell_lower.contains("fish") {
        let fish_config = home.join(".config/fish/config.fish");
        if let Some(parent) = fish_config.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        write_path_to_fish_file(&fish_config)?;
    } else if shell_lower.contains("zsh") {
        // zsh 兼容 bash 的 export 语法
        let zshrc = home.join(".zshrc");
        write_path_to_shell_file(&zshrc)?;
    } else {
        // 默认 bash（含 $SHELL 为空 / 未知）
        let bashrc = home.join(".bashrc");
        write_path_to_shell_file(&bashrc)?;
    }

    // SAFETY: 在安装/卸载操作中修改 PATH 是预期行为
    unsafe { std::env::set_var("PATH", new_path) };

    Ok(())
}

/// 写入 export PATH=... 到 shell 文件（检查重复，避免多次追加）
#[cfg(not(target_os = "windows"))]
fn write_path_to_shell_file(path: &Path) -> Result<()> {
    use std::io::Write;

    let path_val = std::env::var("PATH").unwrap_or_default();
    let marker = "# Added by OneInit";

    // 检查是否已有 OneInit 写入的 PATH 行，避免重复追加
    let existing = std::fs::read_to_string(path).unwrap_or_default();
    if existing.contains(marker) {
        return Ok(());
    }

    let export_line = format!("\n{}\nexport PATH=\"{}\"\n", marker, path_val);

    let mut file = std::fs::OpenOptions::new().append(true).open(path)?;
    file.write_all(export_line.as_bytes())?;
    Ok(())
}

/// 写入 fish 格式的 set -gx PATH（检查重复，避免多次追加）
#[cfg(not(target_os = "windows"))]
fn write_path_to_fish_file(path: &Path) -> Result<()> {
    use std::io::Write;

    let path_val = std::env::var("PATH").unwrap_or_default();
    let marker = "# Added by OneInit";

    let existing = std::fs::read_to_string(path).unwrap_or_default();
    if existing.contains(marker) {
        return Ok(());
    }

    let fish_path = format!(
        "\n{}\nset -gx PATH {}\n",
        marker,
        path_val.replace(':', " ")
    );

    let mut file = std::fs::OpenOptions::new().append(true).open(path)?;
    file.write_all(fish_path.as_bytes())?;
    Ok(())
}
