use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use regex::Regex;
use std::error::Error;
use directories::UserDirs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalForward {
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host_tag: String,
    pub user: String,
    pub hostname: String,
    pub port: u16,
    pub group: String,
    pub tags: Vec<String>,
    pub forward_agent: bool,
    pub dynamic_forward: Option<String>,
    pub local_forward: Option<LocalForward>,
    pub proxy_jump: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedConfig {
    pub global: Vec<(String, String)>,
    pub servers: Vec<ServerConfig>,
}

/// 解析SSH配置文件
pub fn parse_ssh_config(path: &Path) -> Result<ParsedConfig, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    // 预编译正则表达式
    let re_host = Regex::new(r"(?i)^host\s+(\S+)")?;
    let re_config = Regex::new(r"(?i)^\s*(\S+)\s+(.+)")?;
    let re_group = Regex::new(r"(?i)^\s*#\s*group\s+([^\s#]+)")?;
    let re_tags = Regex::new(r"(?i)^\s*#\s*tags\s+([^#]+)")?;
    
    let mut global_config = Vec::new();
    let mut servers = Vec::new();
    let mut current = None;
    let mut in_global_section = false;
    
    for line in reader.lines() {
        let line = line?.trim().to_string();
        if line.is_empty() {
            continue;
        }

        // 处理新Host声明
        if let Some(caps) = re_host.captures(&line) {
            let host_tag = &caps[1];
            
            // 保存前一个配置
            if let Some(server) = current.take() {
                servers.push(server);
            }
            
            // 检查是否为全局配置
            if host_tag == "*" {
                in_global_section = true;
                continue;
            }
            
            // 创建新服务器配置
            in_global_section = false;
            current = Some(ServerConfig {
                host_tag: host_tag.to_string(),
                user: String::new(),
                hostname: String::new(),
                port: 22,
                group: String::new(),
                tags: Vec::new(),
                forward_agent: false,
                dynamic_forward: None,
                local_forward: None,
                proxy_jump: None,
            });
            continue;
        }

        // 处理全局配置项
        if in_global_section {
            if let Some(caps) = re_config.captures(&line) {
                global_config.push((caps[1].to_string(), caps[2].to_string()));
            }
            continue;
        }

        // 处理服务器配置项
        if let Some(ref mut server) = current {
            if let Some(caps) = re_group.captures(&line) {
                server.group = caps[1].trim().to_string();
            }
            else if let Some(caps) = re_tags.captures(&line) {
                server.tags = caps[1].split_whitespace()
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();
            }
            else if let Some(caps) = re_config.captures(&line) {
                let key = caps[1].to_lowercase();
                let value = caps[2].to_string();
                
                match key.as_str() {
                    "user" => server.user = value,
                    "hostname" => server.hostname = value,
                    "port" => server.port = value.parse().unwrap_or(22),
                    "forwardagent" => server.forward_agent = value.to_lowercase() == "yes",
                    "dynamicforward" => server.dynamic_forward = Some(value),
                    "localforward" => {
                        // println!("Parsing LocalForward: {}", value);
                        let parts: Vec<&str> = value.splitn(2, ' ').collect();
                        // println!("LocalForward parts: {:?}", parts);
                        if parts.len() == 2 {
                            if let Ok(local_port) = parts[0].parse::<u16>() {
                                let remote_parts: Vec<&str> = parts[1].splitn(2, ':').collect();
                                if remote_parts.len() == 2 {
                                    if let Ok(remote_port) = remote_parts[1].parse::<u16>() {
                                        // println!("Parsed LocalForward: {} {}:{}", local_port, remote_parts[0], remote_port);
                                        server.local_forward = Some(LocalForward {
                                            local_port,
                                            remote_host: remote_parts[0].to_string(),
                                            remote_port,
                                        });
                                    }
                                }
                            }
                        }
                    },
                    "proxyjump" => server.proxy_jump = Some(value),
                    _ => {} // 忽略其他配置
                }
            }
        }
    }
    
    // 保存最后一个配置
    if let Some(server) = current {
        servers.push(server);
    }
    
    Ok(ParsedConfig { global: global_config, servers })
}

/// 保存SSH配置
pub fn save_ssh_config(path: &Path, config: &ParsedConfig) -> Result<(), Box<dyn Error>> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)?;
    
    // 写入全局配置
    if !config.global.is_empty() {
        writeln!(file, "Host *")?;
        for (key, value) in &config.global {
            writeln!(file, "    {} {}", key, value)?;
        }
        writeln!(file)?;
    }
    
    // 写入服务器配置
    for server in &config.servers {
        writeln!(file, "Host {}", server.host_tag)?;
        if !server.user.is_empty() {
            writeln!(file, "    User {}", server.user)?;
        }
        if !server.hostname.is_empty() {
            writeln!(file, "    Hostname {}", server.hostname)?;
        }
        if server.port != 22 {
            writeln!(file, "    Port {}", server.port)?;
        }
        if server.forward_agent {
            writeln!(file, "    ForwardAgent yes")?;
        }
        if let Some(dynamic) = &server.dynamic_forward {
            writeln!(file, "    DynamicForward {}", dynamic)?;
        }
        if let Some(local) = &server.local_forward {
            writeln!(file, "    LocalForward {} {}:{}",
                local.local_port, local.remote_host, local.remote_port)?;
        }
        if let Some(proxy) = &server.proxy_jump {
            writeln!(file, "    ProxyJump {}", proxy)?;
        }
        if !server.group.is_empty() {
            writeln!(file, "# Group {}", server.group)?;
        }
        if !server.tags.is_empty() {
            let tags = server.tags.join(" ");
            writeln!(file, "# Tags {}", tags)?;
        }
        writeln!(file)?;
    }
    
    Ok(())
}


/// 过滤服务器列表
pub fn filter_servers(servers: &ParsedConfig, group: Option<&str>, tags: Option<&str>) -> Vec<ServerConfig> {
    servers.servers
        .iter()
        .filter(|s| !s.host_tag.is_empty())
        .filter(|s| {
            let has_annotations = !s.group.is_empty() || !s.tags.is_empty();
            // 只有当有注释时才进行过滤匹配
            has_annotations
            && group.map_or(true, |g| !s.group.is_empty() && s.group.contains(g))
            && tags.map_or(true, |t| !s.tags.is_empty() && s.tags.iter().any(|tag| tag == t))
        })
        .cloned()
        .collect::<Vec<_>>()
}

/// 获取SSH配置文件路径
pub fn get_ssh_config_path() -> Result<std::path::PathBuf, Box<dyn Error>> {
    let user_dirs = UserDirs::new().ok_or("无法获取用户目录")?;
    Ok(Path::new(&user_dirs.home_dir()).join(".ssh/config"))
}

/// 连接到指定服务器
pub fn connect_to_server(server: &ServerConfig) -> Result<(), Box<dyn Error>> {
    println!("连接到 {}@{}:{}(通过 {})", 
        server.user, server.hostname, server.port, server.host_tag);
    
    let status = std::process::Command::new("ssh")
        .arg(&server.host_tag)
        .status()?;
        
    if !status.success() {
        return Err(format!("SSH连接失败: {}", status).into());
    }
    Ok(())
}