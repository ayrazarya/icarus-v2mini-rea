/// Helper to trigger native Linux file dialog via zenity if available
pub fn pick_file_zenity(save: bool, default_name: &str) -> Option<String> {
    let mut cmd = std::process::Command::new("zenity");
    cmd.arg("--file-selection");
    if save {
        cmd.arg("--save").arg("--confirm-overwrite");
    }
    cmd.arg("--file-filter=TOML Profile (*.toml) | *.toml");
    cmd.arg("--file-filter=All Files (*) | *");
    if !default_name.is_empty() {
        cmd.arg(format!("--filename={}", default_name));
    }
    let output = cmd.output().ok()?;
    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Some(path);
        }
    }
    None
}
