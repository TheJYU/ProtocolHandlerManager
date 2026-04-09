use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn get_applications_dir() -> Result<PathBuf, String> {
    let home = env::var("HOME").map_err(|_| "HOME environment variable not found".to_string())?;
    let mut path = PathBuf::from(home);
    path.push(".local");
    path.push("share");
    path.push("applications");
    fs::create_dir_all(&path).map_err(|e| format!("Failed to create applications directory: {}", e))?;
    Ok(path)
}

fn get_desktop_file_path(protocol: &str) -> Result<PathBuf, String> {
    let dir = get_applications_dir()?;
    Ok(dir.join(format!("protocol-handler-manager-{}.desktop", protocol)))
}

pub fn register_protocol(protocol: &str) -> Result<(), String> {
    let exe_path = env::current_exe().map_err(|e| format!("Failed to get executable path: {}", e))?;
    let desktop_file_path = get_desktop_file_path(protocol)?;

    // Properly escape any double quotes in the path according to XDG spec
    let safe_exe_path = exe_path.display().to_string().replace("\"", "\\\"");

    let content = format!(
        "[Desktop Entry]\n\
        Type=Application\n\
        Name=Protocol Handler Manager ({})\n\
        Exec=\"{}\" %u\n\
        MimeType=x-scheme-handler/{};\n\
        NoDisplay=true\n",
        protocol,
        safe_exe_path,
        protocol
    );

    fs::write(&desktop_file_path, content).map_err(|e| format!("Failed to write .desktop file: {}", e))?;

    let _ = update_desktop_database();
    let _ = set_default_mime(protocol, &desktop_file_path);
    Ok(())
}

pub fn unregister_protocol(protocol: &str) -> Result<(), String> {
    let desktop_file_path = get_desktop_file_path(protocol)?;
    if desktop_file_path.exists() {
        let _ = fs::remove_file(desktop_file_path);
    }
    let _ = update_desktop_database();
    Ok(())
}

fn update_desktop_database() -> Result<(), String> {
    let dir = get_applications_dir()?;
    Command::new("update-desktop-database")
        .arg(&dir)
        .status()
        .map_err(|e| format!("Failed to update desktop database: {}", e))?;
    Ok(())
}

fn set_default_mime(protocol: &str, desktop_file_path: &PathBuf) -> Result<(), String> {
    let desktop_file_name = desktop_file_path.file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid desktop file name")?;
    let mime_type = format!("x-scheme-handler/{}", protocol);

    Command::new("xdg-mime")
        .args(["default", desktop_file_name, &mime_type])
        .status()
        .map_err(|e| format!("Failed to set default mime: {}", e))?;
    Ok(())
}
