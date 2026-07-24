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
        return Ok(()); // 已存在，跳过
    }

    let new_path = if current.ends_with(';') || current.ends_with(':') {
        format!("{}{}", current, dir_str)
    } else {
        format!("{}{}{}", current, PATH_SEP, dir_str)
    };

    set_path(&new_path)
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
#[cfg(target_os = "windows")]
fn set_path_windows(new_path: &str) -> Result<()> {
    use winreg::RegKey;
    use winreg::enums::*;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags("Environment", KEY_WRITE)
        .map_err(|e| CoreError::PathOp(format!("无法打开注册表: {}", e)))?;

    env.set_value("PATH", &new_path)
        .map_err(|e| CoreError::PathOp(format!("无法写入 PATH 注册表: {}", e)))?;

    // 广播 WM_SETTINGCHANGE 通知其他进程
    broadcast_setting_change();

    // SAFETY: 在安装/卸载操作中修改 PATH 是预期行为
    unsafe { std::env::set_var("PATH", new_path) };

    Ok(())
}

/// Windows: 广播 WM_SETTINGCHANGE
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
        winapi::um::winuser::SendMessageW(
            hwnd,
            winapi::um::winuser::WM_SETTINGCHANGE,
            0,
            env.as_ptr() as winapi::shared::minwindef::LPARAM,
        );
    }
}

/// Unix: 写入 shell 配置文件
#[cfg(not(target_os = "windows"))]
fn set_path_unix(new_path: &str) -> Result<()> {
    let home =
        dirs::home_dir().ok_or_else(|| CoreError::PathOp("无法获取用户主目录".to_string()))?;

    // 检测可用的 shell 并写入对应配置文件
    let mut written = false;

    let bashrc = home.join(".bashrc");
    if bashrc.exists() {
        write_path_to_shell_file(&bashrc)?;
        written = true;
    }

    let zshrc = home.join(".zshrc");
    if zshrc.exists() {
        write_path_to_shell_file(&zshrc)?;
        written = true;
    }

    let fish_config = home.join(".config/fish/config.fish");
    if fish_config.exists() {
        write_path_to_fish_file(&fish_config)?;
        written = true;
    }

    if !written {
        // 如果都没找到，尝试创建 .bashrc
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
        path_val.replace(':', ' ')
    );

    let mut file = std::fs::OpenOptions::new().append(true).open(path)?;
    file.write_all(fish_path.as_bytes())?;
    Ok(())
}
