use dirs::config_dir;
use eyre::OptionExt;
use luxnulla::{
    CONFIG_DIR, CommandRequest, CommandResponse, EDITOR_NAME, ErrorCommandResponse,
    LUXNULLA_CONFIG_FILE, OkCommandResponse, SOCKET_NAME, XRAY_CONFIG_FILE,
};
use std::{fs, path::PathBuf, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
};

mod subscribe_parse;
mod xray_parser;

struct Application {
    config_dir: PathBuf,
}

impl Application {
    async fn handle_client(&self, mut sock: UnixStream) {
        let mut buf = vec![0u8; 1024];
        match sock.read(&mut buf).await {
            Ok(n) if n > 0 => {
                let req: Result<CommandRequest, _> = serde_json::from_slice(&buf[..n]);

                let resp = match req {
                    Ok(CommandRequest::Status) => CommandResponse::Ok(OkCommandResponse::Message(
                        "Luxnulla-core is running".to_string(),
                    )),
                    Ok(CommandRequest::Restart) => {
                        let mock_plain_text_url = "https://raw.githubusercontent.com/Epodonios/v2ray-configs/refs/heads/main/Base64/Config%20list10_base64.txt";

                        println!(
                            "--- Fetching from plain text URL: {} ---",
                            mock_plain_text_url
                        );

                        match subscribe_parse::fetch_and_parse_configs(mock_plain_text_url).await {
                            Ok(configs) => {
                                println!("Successfully parsed {} configs.", configs.len());
                                xray_parser::work(configs);
                            }
                            Err(e) => {
                                eprintln!("Error: {}", e);
                            }
                        }

                        CommandResponse::Ok(OkCommandResponse::Message(String::from("restart")))
                    }
                    Ok(CommandRequest::EditXray) => {
                        tokio::process::Command::new(EDITOR_NAME)
                            .arg(&self.config_dir.join(XRAY_CONFIG_FILE))
                            .spawn()
                            .unwrap();

                        CommandResponse::Ok(OkCommandResponse::Message(String::from(
                            "zeditor is running",
                        )))
                    }
                    Ok(CommandRequest::EditLuxnulla) => {
                        tokio::process::Command::new(EDITOR_NAME)
                            .arg(&self.config_dir.join(LUXNULLA_CONFIG_FILE))
                            .spawn()
                            .unwrap();

                        CommandResponse::Ok(OkCommandResponse::Message(String::from(
                            "zeditor is running",
                        )))
                    }
                    Ok(CommandRequest::Start) => {
                        let config_path_buf = self.config_dir.join(XRAY_CONFIG_FILE);

                        let xray_config = match config_path_buf.to_str() {
                            Some(s) => String::from(s),
                            None => String::from("pizda pathbuffer"),
                        };

                        tokio::process::Command::new("xray")
                            .args(&["run", "-c", &xray_config])
                            .spawn()
                            .unwrap();

                        CommandResponse::Ok(OkCommandResponse::Message(String::from(
                            "xray is started",
                        )))
                    }

                    Err(e) => CommandResponse::Err(ErrorCommandResponse::Message(format!(
                        "bad request: {}",
                        e
                    ))),
                };
                let out = serde_json::to_vec(&resp).unwrap();
                let _ = sock.write_all(&out).await;
            }
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let application = Arc::new(Application {
        config_dir: config_dir()
            .ok_or_eyre("cannot get a dir")?
            .join(CONFIG_DIR),
    });

    if !application.config_dir.exists() {
        std::fs::create_dir(&application.config_dir)?;
    }

    if !application.config_dir.join(XRAY_CONFIG_FILE).exists() {
        std::fs::File::create(&application.config_dir.join(XRAY_CONFIG_FILE)).unwrap();
    }

    let sock_path = PathBuf::from("/tmp/").join(SOCKET_NAME);
    if sock_path.exists() {
        fs::remove_file(&sock_path)?;
    }

    let listener = UnixListener::bind(&sock_path)?;
    println!("Luxnulla listening on {:?}", sock_path);

    loop {
        let app_clone = application.clone();

        let (sock, _) = listener.accept().await?;

        tokio::spawn(async move { app_clone.handle_client(sock).await });
    }
}
