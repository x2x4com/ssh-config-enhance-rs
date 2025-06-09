use clap::Parser;
use prettytable::{Table, row, format};
use crossterm::{
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, ClearType},
    ExecutableCommand,
    style::{Attribute, Color},
    QueueableCommand,
    style::Stylize
};
use std::error::Error;
use std::process::exit;
use ssh_config_enhance::{ServerConfig, parse_ssh_config, filter_servers, get_ssh_config_path, connect_to_server};
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(version = "3.3", about = "SSH配置管理工具")]
struct Cli {
    #[clap(short = 'g', long, help = "按组过滤服务器")]
    group: Option<String>,
    
    #[clap(short = 't', long, help = "按标签过滤服务器")]
    tags: Option<String>,

    #[clap(long = "sshconfig", help = "ssh config位置", default_value = get_ssh_config_path().unwrap().into_os_string())]
    ssh_config: PathBuf,
    
    #[clap(short = 'c', long, help = "使用选择器界面", default_value_t = false)]
    choose: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    
    // 获取SSH配置文件路径
    let ssh_config = args.ssh_config;
    
    // 解析配置文件
    let servers = parse_ssh_config(&ssh_config)?;
    
    // 过滤服务器列表
    let filtered = filter_servers(
        &servers,
        args.group.as_deref(),
        args.tags.as_deref()
    );
    
    let selection: usize;
    // 显示服务器列表
    if args.choose {
        selection = display_curses(&filtered)?;
    } else {
        display_table(&filtered)?;
        selection = prompt_selection(&filtered)?;
    }
    
    connect_to_server(&filtered[selection])?;
    
    Ok(())
}

fn display_table(servers: &[ServerConfig]) -> Result<(), Box<dyn Error>> {
    let mut t = Table::new();
    t.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    t.set_titles(row!["ID", "连接目标", "组", "标签", "主机标签", "ForwardAgent", "DynamicForward", "LocalForward", "ProxyJump"]);
    for (i, server) in servers.iter().enumerate() {
        let tags = &server.tags.join(",");
        t.add_row(row![
            i.to_string(),
            format!("{}@{}:{}", server.user, server.hostname, server.port),
            if server.group.is_empty() { "N/A" } else { &server.group }.to_string(),
            if server.tags.is_empty() { "N/A" } else { tags }.to_string(),
            server.host_tag.clone(),
            if server.forward_agent { "Yes" } else { "No" }.to_string(),
            server.dynamic_forward.clone().unwrap_or_else(|| "N/A".to_string()),
            server.local_forward.as_ref().map_or("N/A".to_string(), |lf| format!("{} {}:{}", lf.local_port, lf.remote_host, lf.remote_port)),
            server.proxy_jump.clone().unwrap_or_else(|| "N/A".to_string())
        ]);
    }
    t.printstd();
    Ok(())
}

fn prompt_selection(filtered: &[ServerConfig]) -> Result<usize, Box<dyn Error>> {
    use std::io::{self, Write};
    
    loop {
        let mut input = String::new();
        print!("请输入要连接的服务器ID (0-{}): ", filtered.len().saturating_sub(1));
        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;
        
        match input.trim().parse() {
            Ok(selection) if selection < filtered.len() => return Ok(selection),
            Ok(_) => eprintln!("ID 必须在 0-{} 之间，请重新输入", filtered.len().saturating_sub(1)),
            Err(_) => eprintln!("无效的数字输入，请重新输入"),
        }
    }
}

fn display_curses(servers: &[ServerConfig]) -> Result<usize, Box<dyn Error>> {
    let mut stdout = std::io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    crossterm::terminal::enable_raw_mode()?;
    
    let mut selection = 0;
    let _ = crossterm::terminal::size()?; // 明确忽略未使用变量
    stdout.queue(crossterm::cursor::Hide)?;
    stdout.queue(crossterm::terminal::Clear(ClearType::All))?;

    loop {
        // 绘制标题
        stdout.queue(crossterm::cursor::MoveTo(0, 0))?;
        stdout.queue(crossterm::style::PrintStyledContent(
            crossterm::style::style(" SSH服务器列表 (↑/↓选择，↲确认, ESC退出) ")
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
            let tags = &server.tags.join(",");
            let line = format!("| {:2} | {:<50} | {:<20} | {:<50} | {:<50} |",
                i,
                format!("{}@{}:{}", server.user, server.hostname, server.port),
                if server.group.is_empty() { "" } else { &server.group },
                if server.tags.is_empty() { "" } else { tags },
                server.host_tag
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
                match event {
                    crossterm::event::KeyEvent {
                        code: crossterm::event::KeyCode::Up,
                        ..
                    } => {
                        if selection > 0 {
                            selection -= 1;
                        }
                    }
                    crossterm::event::KeyEvent {
                        code: crossterm::event::KeyCode::Down,
                        ..
                    } => {
                        if selection < servers.len() - 1 {
                            selection += 1;
                        }
                    }
                    crossterm::event::KeyEvent {
                        code: crossterm::event::KeyCode::Enter,
                        ..
                    } => break,
                    crossterm::event::KeyEvent {
                        code: crossterm::event::KeyCode::Esc,
                        ..
                    } |
                    crossterm::event::KeyEvent {
                        code: crossterm::event::KeyCode::Char('c'),
                        modifiers: crossterm::event::KeyModifiers::CONTROL,
                        ..
                    } => {
                        stdout.execute(LeaveAlternateScreen)?;
                        crossterm::terminal::disable_raw_mode()?;
                        stdout.queue(crossterm::cursor::Show)?;
                        println!("退出程序");
                        exit(0);
                    }
                    _ => {}
                }
            }
        }
    }
    
    // 恢复终端状态
    stdout.execute(LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    stdout.queue(crossterm::cursor::Show)?;
    
    Ok(selection)
}
