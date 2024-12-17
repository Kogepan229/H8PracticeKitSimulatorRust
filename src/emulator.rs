use anyhow::Result;
use eframe::egui;
use std::path::PathBuf;
use tokio::{
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    sync::mpsc::{channel, Receiver, Sender},
};

pub const EMULATOR_FILE_NAME: &str = "koge29_h8-3069f_emulator";

pub fn get_emulator_dir_path() -> Result<PathBuf> {
    let mut path = std::env::current_exe()?;
    path.pop();
    path.push("emulator");
    return Ok(path);
}

pub async fn create_emulator_dir() -> Result<()> {
    let path = get_emulator_dir_path()?;
    if !path.exists() {
        tokio::fs::create_dir(path).await?;
    }
    return Ok(());
}

pub fn get_emulator_path() -> Result<PathBuf> {
    let mut path = get_emulator_dir_path()?;
    path.push(EMULATOR_FILE_NAME);
    if cfg!(windows) {
        path.set_extension("exe");
    }
    Ok(path)
}

pub fn check_version() -> Option<String> {
    let emulator_path = get_emulator_path().unwrap();
    let output = std::process::Command::new(emulator_path)
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
    message_rx: Receiver<String>,
    message_tx: Sender<String>,
}

impl Emulator {
    pub async fn execute(
        elf_path: String,
        elf_args: String,
        ctx: egui::Context,
    ) -> Result<Emulator, String> {
        let emulator_path = get_emulator_path().unwrap();
        let arg = "-a=".to_string() + &elf_args;
        let process = tokio::process::Command::new(emulator_path)
            .kill_on_drop(true)
            .args(["--elf", &elf_path, "-w", "-s", arg.as_str()])
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
        let message_rx = Emulator::spawn_receive_worker(socket_reader, ctx);
        let message_tx = Emulator::spawn_send_worker(socket_writer);

        Ok(Emulator {
            process,
            message_rx,
            message_tx,
        })
    }

    fn spawn_send_worker(socket_writer: OwnedWriteHalf) -> Sender<String> {
        let (message_tx, mut message_rx) = channel(32);
        tokio::spawn(async move {
            while let Some(message) = message_rx.recv().await {
                let _msg: String = message + "\n";
                let str_bytes = _msg.as_bytes();
                let mut written_bytes = 0;
                loop {
                    socket_writer.writable().await.unwrap();
                    match socket_writer.try_write(str_bytes) {
                        Ok(n) => {
                            written_bytes += n;
                        }
                        Err(_) => {}
                    }
                    if written_bytes == str_bytes.len() {
                        break;
                    }
                }
            }
        });

        message_tx
    }

    fn spawn_receive_worker(socket_reader: OwnedReadHalf, ctx: egui::Context) -> Receiver<String> {
        let (message_tx, message_rx) = channel(64);
        tokio::spawn(async move {
            let mut message: Vec<u8> = Vec::new();
            loop {
                let mut received = vec![0; 128];
                socket_reader.readable().await.unwrap();
                match socket_reader.try_read(&mut received) {
                    Ok(n) => {
                        if n == 0 {
                            break;
                        }
                        received.truncate(n);

                        for ch in received {
                            if ch == b'\n' {
                                message_tx
                                    .send(
                                        String::from_utf8(message.clone())
                                            .unwrap()
                                            .trim()
                                            .to_string(),
                                    )
                                    .await
                                    .unwrap();
                                message.clear();
                            } else {
                                message.push(ch);
                            }
                        }

                        ctx.request_repaint();
                        // println!(
                        //     "r: {}",
                        //     String::from_utf8(msg.clone()).unwrap().trim().to_string()
                        // )
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
        message_rx
    }

    pub fn pop_messages(&mut self) -> Vec<String> {
        let mut messages = Vec::new();
        loop {
            if let Ok(message) = self.message_rx.try_recv() {
                messages.push(message);
            } else {
                break;
            }
        }
        messages
    }

    pub fn send_message<T: Into<String>>(&self, message: T) {
        let tx = self.message_tx.clone();
        let _message = message.into();
        tokio::spawn(async move {
            tx.send(_message).await.unwrap();
        });
    }
}
