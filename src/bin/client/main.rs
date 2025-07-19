use clap::{Parser, Subcommand};
use luxnulla::{
    CommandRequest, CommandResponse, ErrorCommandResponse, OkCommandResponse, SOCKET_NAME,
};
use std::{path::PathBuf, str::FromStr};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

mod ui;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Edit { target: EditTarget },
    Start,
    Status,
    Restart,
    Tui,
}

#[derive(Debug, Clone)]
enum EditTarget {
    Xray,
    Luxnulla,
}

impl FromStr for EditTarget {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "xray" => Ok(EditTarget::Xray),
            "luxnulla" => Ok(EditTarget::Luxnulla),
            _ => Err(format!(
                "Неизвестный аргумент для 'edit': {}. Ожидается 'xray' или 'luxnulla'.",
                s
            )),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut sock = UnixStream::connect(PathBuf::from("/tmp").join(SOCKET_NAME)).await?;

    let tui = match args.command {
        Commands::Tui => {
            ui::table::init();
        }
        _ => {}
    };

    let cmd: CommandRequest = request_action(args);

    let out = serde_json::to_vec(&cmd)?;
    sock.write_all(&out).await?;

    let mut buf = vec![0u8; 1024];
    let n = sock.read(&mut buf).await?;

    let resp: CommandResponse = serde_json::from_slice(&buf[..n])?;
    response_action(resp);

    Ok(())
}

fn request_action(args: Args) -> CommandRequest {
    match args.command {
        Commands::Edit { target } => match target {
            EditTarget::Xray => CommandRequest::EditXray,
            EditTarget::Luxnulla => CommandRequest::EditLuxnulla,
        },

        Commands::Start => CommandRequest::Start,
        Commands::Status => CommandRequest::Status,
        Commands::Restart => CommandRequest::Restart,
        _ => {
            eprintln!("Usage: client status|restart");
            std::process::exit(1);
        }
    }
}

fn response_action(res: CommandResponse) {
    match res {
        CommandResponse::Ok(res) => match res {
            OkCommandResponse::Message(msg) => {
                println!("Ok: {}", msg);
            }
            OkCommandResponse::GetSubs(msg) => {
                // println!("Error: {}", msg);
            }
        },

        CommandResponse::Err(res) => match res {
            ErrorCommandResponse::Message(err) => {
                println!("Error: {}", err);
            }
            ErrorCommandResponse::GetSubs(msg) => {
                println!("Error: {}", msg);
            }
        },
    }
}
