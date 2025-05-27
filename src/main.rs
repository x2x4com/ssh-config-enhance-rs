use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use regex::Regex;
use clap::Parser;
use prettytable::{Table, row, format};
use directories::UserDirs;
use crossterm::{
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, ClearType}, 
    ExecutableCommand, 
    // event, 
    style::{Attribute, Color, Stylize},
    QueueableCommand
};
use std::error::Error;
use std::process::Command;

#[derive(Parser, Debug)]
#[clap(version = "3.3", about = "SSH配置管理工具")]
struct Cli {
    #[clap(short = 'g', long, help = "按组过滤服务器")]
    group: Option<String>,
    
    #[clap(short = 't', long, help = "按标签过滤服务器")]
    tags: Option<String>,
    
    #[clap(long, help = "使用终端TEXT", default_value_t = false)]
    text: bool,
}

#[derive(Debug, Clone)]
struct ServerConfig {
    host_tag: String,
    user: String,
    hostname: String,
    port: u16,
    group: String,
    tags: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    
    // 获取SSH配置文件路径
    let user_dirs = UserDirs::new().ok_or("无法获取用户目录")?;
    let ssh_config = Path::new(&user_dirs.home_dir()).join(".ssh/config");
    
    // 解析配置文件
    let servers = parse_ssh_config(&ssh_config)?;
    
    // 过滤服务器列表
    let filtered = filter_servers(&servers, &args);
    let selection: usize;
    // 显示服务器列表
    if !args.text {
        selection = display_curses(&filtered)?;
    } else {
        display_table(&filtered)?;
        selection = prompt_selection(&filtered)?;
    }
    connect_to_server(&filtered[selection])?;
    // display_table(&filtered)?;
    // let selection = prompt_selection(&filtered)?;
    // connect_to_server(&filtered[selection])?;
    
    Ok(())
}

fn parse_ssh_config(path: &Path) -> Result<Vec<ServerConfig>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
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
        group: "UNDEFINED".to_string(),
        tags: "UNDEFINED".to_string(),
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
                    .collect::<Vec<_>>()
                    .join(" ");
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

fn filter_servers(servers: &[ServerConfig], args: &Cli) -> Vec<ServerConfig> {
    let mut result = Vec::new();
    
    for server in servers {
        // 处理未定义的group和tags
        let server_group = if server.group.is_empty() { "UNDEFINED" } else { &server.group };
        let server_tags = if server.tags.is_empty() { "UNDEFINED" } else { &server.tags };
        
        // 实现过滤逻辑
        if args.group.as_deref() == Some("ALL") {
            result.push(server.clone());
        } else if let Some(group) = &args.group {
            if server_group.contains(group) {
                if args.tags.as_deref() == Some("NULL") {
                    result.push(server.clone());
                } else if let Some(tags) = &args.tags {
                    if server_tags.contains(tags) {
                        result.push(server.clone());
                    }
                }
            }
        } else if let Some(tags) = &args.tags {
            if server_tags.contains(tags) {
                result.push(server.clone());
            }
        } else {
            result.push(server.clone());
        }
    }
    
    servers
        .iter()
        .filter(|s| !s.host_tag.is_empty())
        .filter(|s| {
            args.group.as_deref().map_or(true, |g| s.group.contains(g)) &&
            args.tags.as_deref().map_or(true, |t| s.tags.contains(t))
        })
        .cloned()
        .collect::<Vec<_>>()
}

fn display_table(servers: &[ServerConfig]) -> Result<(), Box<dyn Error>> {
    let mut t = Table::new();
    t.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    t.set_titles(row!["ID", "Link Target", "Group", "Tags", "Host Info"]);
    for (i, server) in servers.iter().enumerate() {
        t.add_row(row![
            i.to_string(),
            format!("{}@{}:{}", server.user, server.hostname, server.port),
            server.group.clone(),
            server.tags.clone(),
            server.host_tag.clone(),
        ]);
    }
    t.printstd();
    Ok(())
}

fn prompt_selection(filtered: &[ServerConfig]) -> Result<usize, Box<dyn Error>> {
    use std::io::{self, Write};
    
    let mut input = String::new();
    print!("请输入要连接的服务器ID (0-{}): ", filtered.len().saturating_sub(1));
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    let selection = input.trim().parse()?;
    
    if selection >= filtered.len() {
        return Err(format!("ID 必须在 0-{} 之间", filtered.len().saturating_sub(1)).into());
    }
    
    Ok(selection)
}

fn display_curses(servers: &[ServerConfig]) -> Result<usize, Box<dyn Error>> {
    let mut stdout = std::io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    crossterm::terminal::enable_raw_mode()?;
    
    let mut selection = 0;
    let _ = crossterm::terminal::size()?; // 明确忽略未使用变量
    
    loop {
        stdout.queue(crossterm::cursor::Hide)?;
        stdout.queue(crossterm::terminal::Clear(ClearType::All))?;
        
        // 绘制标题
        stdout.queue(crossterm::cursor::MoveTo(0, 0))?;
        stdout.queue(crossterm::style::PrintStyledContent(
            crossterm::style::style(" SSH服务器列表 (↑/↓选择，↲确认) ")
                .with(Color::DarkGreen)
        ))?;
        
        // 绘制服务器列表
        for (i, server) in servers.iter().enumerate() {
            let y = 2 + i as u16;
            stdout.queue(crossterm::cursor::MoveTo(0, y))?;
            
            if i == selection {
                stdout.queue(crossterm::style::SetAttribute(
                    Attribute::Reverse
                ))?;
            }
            
            let line = format!(" {:2} {:<25} {:<15} {:<20}",
                i,
                server.host_tag,
                format!("{}@{}", server.user, server.hostname),
                format!(":{}", server.port)
            );
            
            stdout.queue(crossterm::style::Print(line))?;
            
            if i == selection {
                stdout.queue(crossterm::style::SetAttribute(
                    Attribute::NoReverse
                ))?;
            }
        }
        
        stdout.flush()?;
        
        // 处理键盘事件
        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let crossterm::event::Event::Key(event) = crossterm::event::read()? {
                match event.code {
                    crossterm::event::KeyCode::Up => {
                        if selection > 0 {
                            selection -= 1;
                        }
                    }
                    crossterm::event::KeyCode::Down => {
                        if selection < servers.len() - 1 {
                            selection += 1;
                        }
                    }
                    crossterm::event::KeyCode::Enter => break,
                    crossterm::event::KeyCode::Esc => {
                        selection = 0;
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
    
    // 恢复终端状态
    stdout.execute(crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;

    stdout.execute(LeaveAlternateScreen)?;
    stdout.queue(crossterm::cursor::Show)?;
    
    Ok(selection)
}

fn connect_to_server(server: &ServerConfig) -> Result<(), Box<dyn Error>> {
    // 实现SSH连接逻辑
    // println!("info: {:#?}", server);
    println!("Connecting to {}@{}:{}", server.user, server.hostname, server.port);
    // 这里可以使用ssh命令或其他库来实现实际的连接
    // 例如使用 std::process::Command 来调用 ssh 命令
    
    let status = Command::new("ssh")
        .arg("-p").arg(server.port.to_string())
        .arg(format!("{}@{}", server.user, server.hostname))
        .status()?;
    if !status.success() {
        return Err(format!("SSH Failed: {}", status).into());
    }
    Ok(())
}
