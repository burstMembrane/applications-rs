use crate::common::App;

use regex::Regex;
use std::io;
use std::process::Command;

use anyhow::Error;
use anyhow::Result;
use ini::ini;

use std::collections::HashSet;
use std::path::PathBuf;
use walkdir::WalkDir;
pub fn parse_desktop_file(desktop_file_path: PathBuf) -> App {
    let mut app = App::default();

    if desktop_file_path.is_dir() {
        return app;
    }
    app.app_desktop_path = desktop_file_path.clone();
    let desktop_file_path_str = desktop_file_path.to_str().unwrap();
    let map = ini!(desktop_file_path_str);
    let desktop_entry_exists = map.contains_key("desktop entry");
    if desktop_entry_exists {
        let desktop_entry = map["desktop entry"].clone();
        if desktop_entry.contains_key("exec") {
            let exec = desktop_entry["exec"].clone();
            app.app_path_exe = Some(PathBuf::from(exec.unwrap()));
        }
        if desktop_entry.contains_key("icon") {
            let icon = desktop_entry["icon"].clone();
            app.icon_path = Some(PathBuf::from(icon.unwrap()));
        }
        if desktop_entry.contains_key("name") {
            let name = desktop_entry["name"].clone();
            app.name = name.unwrap();
        }
    }
    return app;
}

pub fn get_all_apps() -> Result<Vec<App>> {
    // read XDG_DATA_DIRS env var
    let xdg_data_dirs = std::env::var("XDG_DATA_DIRS").unwrap_or("/usr/share".to_string());
    let xdg_data_dirs: Vec<&str> = xdg_data_dirs.split(':').collect();
    // make a string sett from xdg_data_dirs
    let mut search_dirs: HashSet<&str> = xdg_data_dirs.iter().cloned().collect();

    // get home dir of current user
    let home_dir = std::env::var("HOME").unwrap();
    let home_path = PathBuf::from(home_dir);
    let local_share_apps = home_path.join(".local/share/applications");

    search_dirs.insert(local_share_apps.to_str().unwrap());
    search_dirs.insert("/usr/share/applications");
    search_dirs.insert("/usr/share/xsessions");
    search_dirs.insert("/etc/xdg/autostart");
    search_dirs.insert("/var/lib/snapd/desktop/applications");

    // for each dir, search for .desktop files
    let mut apps: Vec<App> = Vec::new();
    for dir in search_dirs {
        let dir = PathBuf::from(dir);
        if !dir.exists() {
            continue;
        }
        for entry in WalkDir::new(dir.clone()) {
            if entry.is_err() {
                continue;
            }
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().is_none() {
                continue;
            }

            if path.extension().unwrap() == "desktop" {
                let app = parse_desktop_file(path.to_path_buf());
                apps.push(app);
            }
        }
    }
    Ok(apps)
}

pub fn open_file_with(file_path: PathBuf, app: App) {
    let exe_path = app.app_path_exe.unwrap();
    let exec_path_str = exe_path.to_str().unwrap();
    let file_path_str = file_path.to_str().unwrap();
    let output = std::process::Command::new(exec_path_str)
        .arg(file_path_str)
        .output();
    // match output
    match output {
        Ok(_) => println!("File opened successfully"),
        Err(e) => println!("Error opening file: {}", e),
    }
}

// match app name using either app name or .desktop file name
fn match_app_name(app: &App, app_name: &str) -> bool {
    app.name == *app_name
        || app_name.to_lowercase() == app.name.to_lowercase()
        || app
            .app_desktop_path
            .file_stem()
            .unwrap_or("".as_ref())
            .to_string_lossy()
            == *app_name.to_lowercase()
}
pub fn get_running_apps(apps: &Vec<App>) -> Result<Vec<App>> {
    let mut applications = Vec::new();

    // Run `wmctrl -l` to list all open windows
    let wmctrl_output = Command::new("wmctrl").arg("-l").output()?;

    if !wmctrl_output.status.success() {
        return Err(io::Error::from(io::ErrorKind::Other).into());
    }

    let wmctrl_stdout = String::from_utf8_lossy(&wmctrl_output.stdout);

    // Regular expression to extract strings within double quotes
    let re = Regex::new(r#""([^"]*)""#).unwrap();

    // Iterate over each window ID
    for line in wmctrl_stdout.lines() {
        let win_id = line.split_whitespace().next().unwrap_or_default();

        // Check if the window is a normal window
        let xprop_output = Command::new("xprop")
            .args(&["-id", win_id, "_NET_WM_WINDOW_TYPE"])
            .output()?;

        if !xprop_output.status.success() {
            continue;
        }

        let xprop_stdout = String::from_utf8_lossy(&xprop_output.stdout);
        if xprop_stdout.contains("_NET_WM_WINDOW_TYPE_NORMAL") {
            // Get the application name from WM_CLASS property
            let class_output = Command::new("xprop")
                .args(&["-id", win_id, "WM_CLASS"])
                .output()?;

            if !class_output.status.success() {
                continue;
            }

            let class_stdout = String::from_utf8_lossy(&class_output.stdout);
            let captures: Vec<_> = re.captures_iter(&class_stdout).collect();

            if let Some(capture) = captures.get(1) {
                let appname = &capture[1];
                applications.push(appname.to_string());
            }
        }
    }

    // Sort and remove duplicates
    applications.sort();
    applications.dedup();

    let running_apps: Vec<App> = applications
        .iter()
        .filter_map(|app_name| apps.iter().find(|app| match_app_name(app, app_name)))
        .cloned()
        .collect();

    Ok(running_apps)
}

/// If I need to compare app name with app apps, then this function should be moved to AppInfoContext where there is a `cached_apps`
pub fn get_frontmost_application(apps: &Vec<App>) -> Result<App> {
    let output = std::process::Command::new("xprop")
        .arg("-root")
        .arg("_NET_ACTIVE_WINDOW")
        .output()
        .expect("failed to execute process");

    let output = std::str::from_utf8(&output.stdout).unwrap();
    let id = output.split_whitespace().last().unwrap();

    let output = std::process::Command::new("xprop")
        .arg("-id")
        .arg(id)
        .arg("WM_CLASS")
        .output()
        .expect("failed to execute process");

    let output = std::str::from_utf8(&output.stdout).unwrap();

    let app_name = output.split('"').nth(1).unwrap();

    for app in apps {
        if match_app_name(&app, app_name) {
            return Ok(app.clone());
        }
    }

    Err(Error::msg(
        "Failed to find frontmost application. Maybe it's not a .desktop app?",
    ))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_get_apps() {
        let apps = get_all_apps().unwrap();
        // println!("App: {:#?}", apps);
        assert!(apps.len() > 0);
        // iterate through apps and find the onces whose name contains "terminal"
        for app in apps {
            if app.name.to_lowercase().contains("code") {
                println!("App: {:#?}", app);
            }
        }
    }

    #[test]
    fn test_parse_desktop_file() {
        // let applications path

        let output = Path::new("/usr/share/applications/")
            .read_dir()
            // get the first .desktop file
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().unwrap() == "desktop")
            .next()
            .unwrap()
            .path();
        let app = parse_desktop_file(output);
        println!("App: {:#?}", app);
    }
}
