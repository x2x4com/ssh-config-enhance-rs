#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::path::Path;
use tauri_plugin_shell::ShellExt;
use ssh_config_enhance::{get_ssh_config_path, parse_ssh_config, save_ssh_config, filter_servers};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      get_ssh_config,
      connect_to_host,
      get_servers,
      save_servers
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[tauri::command]
fn get_ssh_config() -> Result<String, String> {
    let ssh_config = get_ssh_config_path().map_err(|e| e.to_string())?;
    let servers = parse_ssh_config(&ssh_config).map_err(|e| e.to_string())?;

    serde_json::to_string(&servers).map_err(|e| e.to_string())
}



/// 连接到指定主机
#[tauri::command]
async fn connect_to_host(app_handle: tauri::AppHandle, host: String) -> Result<(), String> {
    let shell = app_handle.shell();
    
    match shell.command("ssh").args([&host]).spawn() {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("执行 SSH 命令失败: {}", e)),
    }
}


/// Tauri命令：获取服务器配置
#[tauri::command]
fn get_servers(group: Option<&str>, tags: Option<&str>) -> Result<ssh_config_enhance::ParsedConfig, String> {
    let path = Path::new("./config");
    let parsed_ssh_config = parse_ssh_config(path).map_err(|e| e.to_string());
    let filtered_servers = filter_servers(&parsed_ssh_config.clone()?, group, tags);
    let config = ssh_config_enhance::ParsedConfig {
        servers: filtered_servers,
        global: parsed_ssh_config?.global,
    };
    Ok(config)
}

/// Tauri命令：保存服务器配置
#[tauri::command]
fn save_servers(config: ssh_config_enhance::ParsedConfig) -> Result<(), String> {
    let path = Path::new("./config");
    save_ssh_config(path, &config).map_err(|e| e.to_string())
}