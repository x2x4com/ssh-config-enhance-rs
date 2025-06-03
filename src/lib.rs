use serde::Serialize;
use std::path::Path;
use std::fs::File;
use std::io::BufRead;
use regex::Regex;
use std::error::Error;
use directories::UserDirs;

#[derive(Debug, Clone, Serialize)]
pub struct ServerConfig {
    pub host_tag: String,
    pub user: String,
    pub hostname: String,
    pub port: u16,
    pub group: String,
    pub tags: Vec<String>,
}

/// 解析SSH配置文件
pub fn parse_ssh_config(path: &Path) -> Result<Vec<ServerConfig>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = std::io::BufReader::new(file);
    
    // 预编译正则表达式
    let re_host = Regex::new(r"(?i)^host\s+(\S+)")?;
    let re_user = Regex::new(r"(?i)^\s*user\s+(\S+)")?;
    let re_hostname = Regex::new(r"(?i)^\s*hostname\s+(\S+)")?;
    let re_port = Regex::new(r"(?i)^\s*port\s+(\S+)")?;
    let re_group = Regex::new(r"(?i)^\s*#\s*group\s+([^\s#]+)")?;
    let re_tags = Regex::new(r"(?i)^\s*#\s*tags\s+([^#]+)")?;
    
    let mut servers = Vec::new();
    let mut current = ServerConfig {
        host_tag: String::new(),
        user: String::new(),
        hostname: String::new(),
        port: 22,
        group: String::new(),
        tags: Vec::new(),
    };
    
    let mut current_host = String::new();
    
    for line in reader.lines() {
        let line = line?.trim().to_string();
        if line.is_empty() {
            continue;
        }

        // 处理新Host声明
        if let Some(caps) = re_host.captures(&line) {
            let host_tag = &caps[1];
            
            // 保存前一个有效配置（非通配符）
            if !current_host.is_empty() && current_host != "*" {
                servers.push(current.clone());
            }
            
            // 重置临时变量但保留group/tags
            current_host = host_tag.to_string();
            current.host_tag = host_tag.to_string();
            current.user.clear();
            current.hostname.clear();
            current.port = 22;
            current.group.clear();  // 重置group
            current.tags.clear();   // 重置tags
            continue;
        }

        // 仅在当前Host有效时处理配置项
        if current_host != "*" {
            if let Some(caps) = re_group.captures(&line) {
                current.group = caps[1].trim().to_string();
            }
            else if let Some(caps) = re_tags.captures(&line) {
                current.tags = caps[1].split_whitespace()
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();
            }
            else if let Some(caps) = re_user.captures(&line) {
                current.user = caps[1].trim().to_string();
            }
            else if let Some(caps) = re_hostname.captures(&line) {
                current.hostname = caps[1].trim().to_string();
            }
            else if let Some(caps) = re_port.captures(&line) {
                current.port = caps[1].trim().parse()?;
            }
        }
    }
    
    // 保存最后一个有效配置
    if !current_host.is_empty() && current_host != "*" {
        servers.push(current);
    }
    
    Ok(servers)
}

/// 过滤服务器列表
pub fn filter_servers(servers: &[ServerConfig], group: Option<&str>, tags: Option<&str>) -> Vec<ServerConfig> {
    servers
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