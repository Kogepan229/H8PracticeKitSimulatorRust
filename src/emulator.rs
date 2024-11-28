use eframe::egui;
use std::{process::Stdio, sync::Arc};
use tokio::{
    net::{tcp::OwnedWriteHalf, TcpStream},
    sync::Mutex,
};
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, LinesCodec};

pub static EMULATOR_PATH: &str = "./emulator/koge29_h8-3069f_emulator";

pub fn check_version() -> Option<String> {
    let output = std::process::Command::new(EMULATOR_PATH)
        .arg("--version")
        .output();
    if let Ok(o) = output {
        if let Ok(os) = String::from_utf8(o.stdout) {
            if os.starts_with("koge29_h8-3069f_emulator ") {
                let version = os
                    .replace("koge29_h8-3069f_emulator ", "")
                    .trim()
                    .to_string();
                return Some(version);
            }
        }
    }
    return None;
}

pub struct Emulator {
    pub process: tokio::process::Child,
    // process_reader: FramedRead<ChildStdout, LinesCodec>,
    // stream: Box<TcpStream>,
    pub socket_received: Arc<Mutex<Vec<String>>>,
    pub socket_writer: OwnedWriteHalf,
}

impl Emulator {
    pub async fn execute(elf_path: String, ctx: egui::Context) -> Result<Emulator, String> {
        let process = tokio::process::Command::new(EMULATOR_PATH)
            .kill_on_drop(true)
            .args(["--elf", &elf_path, "-r", "-s"])
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start emulator.");

        let stream: TcpStream;
        loop {
            let _stream = TcpStream::connect("127.0.0.1:12345").await;
            match _stream {
                Ok(s) => {
                    stream = s;
                    break;
                }
                Err(e) => {
                    if process.id().is_none() {
                        return Err(e.to_string());
                    }
                }
            }
        }

        let (socket_reader, socket_writer) = stream.into_split();

        let socket_received = Arc::new(Mutex::new(Vec::<String>::new()));
        let _socket_received = socket_received.clone();
        tokio::spawn(async move {
            loop {
                let mut msg = vec![0; 1024];
                socket_reader.readable().await.unwrap();
                match socket_reader.try_read(&mut msg) {
                    Ok(n) => {
                        if n == 0 {
                            break;
                        }
                        msg.truncate(n);
                        _socket_received
                            .lock()
                            .await
                            .push(String::from_utf8(msg.clone()).unwrap().trim().to_string());
                        ctx.request_repaint();
                        println!(
                            "r: {}",
                            String::from_utf8(msg.clone()).unwrap().trim().to_string()
                        )
                    }
                    Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        println!("{}", e.to_string());
                        return;
                    }
                }
            }
        });

        Ok(Emulator {
            process,
            socket_received,
            socket_writer,
        })
    }
}
