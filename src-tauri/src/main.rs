#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    ssh_config_manager::run();
}
