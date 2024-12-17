use std::path::PathBuf;

use anyhow::{bail, Result};
use eframe::egui::{self, ViewportClass};
use serde::Deserialize;
use tokio::{
    io::AsyncWriteExt,
    sync::mpsc::{channel, Receiver, Sender},
    task::spawn_blocking,
};
use tokio_stream::StreamExt;

use crate::{
    emulator::{self, get_emulator_path},
    utils,
};

const USER_AGENT: &str = "h8pks";
const LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/Kogepan229/Koge29_H8-3069F_Emulator/releases/latest";

pub enum UpdateStatusType {
    UNCHECKED,
    CHECKING,
    CHECKED,
    DOWNLOADING,
    DOWNLOADED,
    COMPLETED,
}

pub struct Updater {
    current_version: Option<String>,
    update_status: UpdateStatusType,
    rx_latest_info: Option<Receiver<Result<LatestEmulatorAsset>>>,
    asset: Option<LatestEmulatorAsset>,
    // rx_download_progress: Receiver<String>,
    rs_download_notice: Option<Receiver<()>>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    browser_download_url: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct LatestRelease {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Clone)]
pub struct LatestEmulatorAsset {
    version: String,
    url: String,
    name: String,
}

impl Updater {
    pub fn new(current_emulator_version: Option<String>) -> Self {
        Updater {
            current_version: current_emulator_version,
            update_status: UpdateStatusType::UNCHECKED,
            rx_latest_info: None,
            asset: None,
            // rx_download_progress: Receiver<String>,
            rs_download_notice: None,
        }
    }

    pub fn update(&mut self, _ui: &mut egui::Ui, ctx: &egui::Context) {
        match self.update_status {
            UpdateStatusType::UNCHECKED => {
                self.update_status = UpdateStatusType::CHECKING;
                let (tx, rx) = channel::<Result<LatestEmulatorAsset>>(1);
                self.rx_latest_info = Some(rx);
                tokio::spawn(async move {
                    tx.send(get_latest_info().await).await.unwrap();
                });
                return;
            }
            UpdateStatusType::CHECKING => {
                if let Some(rx) = self.rx_latest_info.as_mut() {
                    match rx.try_recv() {
                        Ok(result) => {
                            if let Ok(asset) = result {
                                self.asset = Some(asset);
                                self.update_status = UpdateStatusType::CHECKED;
                            } else {
                                self.update_status = UpdateStatusType::COMPLETED;
                                log::warn!("Failed to get latest emulator info.");
                                return;
                            }
                        }
                        Err(e) => match e {
                            tokio::sync::mpsc::error::TryRecvError::Empty => return,
                            tokio::sync::mpsc::error::TryRecvError::Disconnected => {
                                self.update_status = UpdateStatusType::COMPLETED;
                                log::error!("Unexpected chunnel receive error occured when checking update.");
                                return;
                            }
                        },
                    }
                }
            }
            UpdateStatusType::CHECKED => {
                let asset = if let Some(asset) = &self.asset {
                    asset
                } else {
                    log::error!("Could not get asset");
                    self.update_status = UpdateStatusType::COMPLETED;
                    return;
                };
                if let Some(current_version) = &self.current_version {
                    if current_version == &asset.version {
                        log::info!("Using latest emulator");
                        self.update_status = UpdateStatusType::COMPLETED;
                        return;
                    }
                }

                Updater::ui(ctx, |ctx, _class| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        if let Some(current_version) = &self.current_version {
                            ui.label("Update emulator");
                            ui.label(format!("v{} -> v{}", current_version, asset.version));
                        } else {
                            ui.label("Download emulator");
                            ui.label(format!("v{}", asset.version));
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            let button_text = if self.current_version.is_some() {
                                "Update"
                            } else {
                                "Download"
                            };
                            if ui.button(button_text).clicked() {
                                self.update_status = UpdateStatusType::DOWNLOADING;
                                let (tx, rx) = channel::<()>(1);
                                self.rs_download_notice = Some(rx);
                                let _asset = asset.clone();
                                let _ctx = ctx.clone();
                                tokio::spawn(async move {
                                    update_emulator(&_asset, tx).await.unwrap();
                                    _ctx.request_repaint();
                                });
                                return;
                            };
                            if self.current_version.is_some() {
                                if ui.button("Skip").clicked() {
                                    self.update_status = UpdateStatusType::COMPLETED;
                                    return;
                                }
                            }
                        })
                    })
                });
            }
            UpdateStatusType::DOWNLOADING => {
                Updater::ui(ctx, |_ctx, _class| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.label("Downloading emulator...");
                    })
                });
                if let Some(rx) = self.rs_download_notice.as_mut() {
                    match rx.try_recv() {
                        Ok(_) => {
                            self.update_status = UpdateStatusType::DOWNLOADED;
                        }
                        Err(e) => match e {
                            tokio::sync::mpsc::error::TryRecvError::Empty => return,
                            tokio::sync::mpsc::error::TryRecvError::Disconnected => {
                                self.update_status = UpdateStatusType::COMPLETED;
                                log::error!("Unexpected chunnel receive error occured when checking update.");
                                return;
                            }
                        },
                    }
                }
            }
            UpdateStatusType::DOWNLOADED => {
                let version = if let Some(current_version) = &self.current_version {
                    current_version.clone()
                } else {
                    if let Some(v) = emulator::check_version() {
                        v
                    } else {
                        self.update_status = UpdateStatusType::COMPLETED;
                        log::error!("Could not check emulator version.");
                        return;
                    }
                };
                Updater::ui(ctx, |ctx, _class| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.label("Download completed!");
                        ui.label(format!("Emulator version: {}", version));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            if ui.button("Close").clicked() {
                                self.update_status = UpdateStatusType::COMPLETED;
                                return;
                            }
                        })
                    });
                });
            }
            UpdateStatusType::COMPLETED => (),
        }
    }

    fn ui<T>(ctx: &egui::Context, viewport_ui_cb: impl FnMut(&egui::Context, ViewportClass) -> T) {
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("update_window"),
            egui::ViewportBuilder::default()
                .with_title("Update")
                .with_inner_size([400.0, 200.0]),
            viewport_ui_cb,
        );
    }
}

pub async fn get_latest_info() -> Result<LatestEmulatorAsset> {
    let info: LatestRelease = reqwest::Client::new()
        .get(LATEST_RELEASE_URL)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()
        .await?
        .json()
        .await?;

    for asset in info.assets.iter() {
        if asset
            .name
            .starts_with(&("koge29_h8-3069f_emulator-".to_string() + get_target()))
        {
            let version = if info.tag_name.starts_with("v") {
                info.tag_name[1..].to_string()
            } else {
                info.tag_name.clone()
            };
            return Ok(LatestEmulatorAsset {
                version,
                name: asset.name.clone(),
                url: asset.browser_download_url.clone(),
            });
        }
    }

    bail!("Not found latest emulator info")
}

pub async fn download_file(url: String, filename: String) -> Result<PathBuf> {
    utils::create_tmp_dir().await?;
    let mut filepath = utils::get_tmp_dir_path()?;
    filepath.push(filename);
    if filepath.exists() {
        tokio::fs::remove_file(filepath.clone()).await?;
    }

    let client = reqwest::Client::new();
    let mut stream = client
        .get(url)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()
        .await?
        .bytes_stream();

    let mut file = tokio::fs::File::create(filepath.clone()).await?;
    while let Some(chunk_result) = stream.next().await {
        let chunk = &chunk_result?;
        file.write_all(&chunk).await?;
    }
    file.flush().await?;

    Ok(filepath)
}

pub async fn update_emulator(asset: &LatestEmulatorAsset, tx_notice: Sender<()>) -> Result<()> {
    if !asset.name.ends_with(".zip") {
        bail!("File is not zip.");
    }

    let emulator_path = get_emulator_path()?;
    if emulator_path.exists() {
        tokio::fs::remove_file(emulator_path).await?;
    }
    let downloaded_path = download_file(asset.url.clone(), asset.name.clone()).await?;

    // Unzip
    let mut archive_path = utils::get_tmp_dir_path().unwrap();
    archive_path.push(&asset.name[..asset.name.len() - 4]);
    let _downloaded_path = downloaded_path.clone();
    let _archive_path = archive_path.clone();
    spawn_blocking(move || {
        let zipfile = std::fs::File::open(_downloaded_path).unwrap();
        let mut archive = zip::ZipArchive::new(zipfile).unwrap();

        archive.extract(_archive_path).unwrap();
    })
    .await?;
    tokio::fs::remove_file(downloaded_path).await?;

    // Move emulator
    let mut archived_emulator_path = archive_path.clone();
    archived_emulator_path.push(emulator::EMULATOR_FILE_NAME);
    if cfg!(windows) {
        archived_emulator_path.set_extension("exe");
    }
    emulator::create_emulator_dir().await?;
    tokio::fs::rename(archived_emulator_path, emulator::get_emulator_path()?).await?;
    tokio::fs::remove_dir_all(archive_path).await?;

    tx_notice.send(()).await?;

    Ok(())
}

#[allow(unreachable_code)]
pub fn get_target() -> &'static str {
    #[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
    return "x86_64-pc-windows-msvc";

    #[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "musl"))]
    return "x86_64-unknown-linux-musl";

    #[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
    return "x86_64-unknown-linux-gnu";

    return "unknown";
}
