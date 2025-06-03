#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use tauri_plugin_shell::ShellExt;
use ssh_config_enhance::{get_ssh_config_path, parse_ssh_config, ServerConfig};

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
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}


fn main() {
    /*
    #[cfg(debug_assertions)] // only enable instrumentation in development builds
    let devtools = tauri_plugin_devtools::init();

    let mut builder = tauri::Builder::default();

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(devtools);
    }
    //tauri::Builder::default()
        builder
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_ssh_config,
            connect_to_host,
            save_ssh_config
        ])
        .run(tauri::generate_context!())
        .expect("运行Tauri应用时出错");
    */
    run();
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

/// 保存SSH配置
#[tauri::command]
fn save_ssh_config(config: String) -> Result<(), String> {
    // TODO: 实现保存到文件系统
    println!("保存配置: {}", config);
    Ok(())
}
