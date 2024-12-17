use std::path::PathBuf;

use anyhow::{bail, Result};
use eframe::egui::{self, ViewportClass, ViewportId};
use serde::{Deserialize, Serialize};
use tokio::{
    io::AsyncWriteExt,
    sync::mpsc::{channel, Receiver},
    task::spawn_blocking,
};
use tokio_stream::StreamExt;

use crate::{emulator::get_emulator_path, utils, MyApp};

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
}

#[derive(Debug, Deserialize)]
struct Asset {
    url: String,
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
    pub fn update_update_status(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        match self.update_status {
            UpdateStatusType::UNCHECKED => {
                self.update_status = UpdateStatusType::CHECKING;
                let (tx, mut rx) = channel::<Result<LatestEmulatorAsset>>(1);
                self.rx_latest_info = Some(rx);
                tokio::spawn(async {
                    tx.send(get_latest_info().await).await.unwrap();
                });
                return;
            }
            UpdateStatusType::CHECKING => {
                if let Some(rx) = self.rx_latest_info {
                    match rx.try_recv() {
                        Ok(result) => {
                            if let Ok(asset) = result {
                                self.asset = Some(asset);
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
                let asset = if let Some(asset) = self.asset {
                    asset
                } else {
                    log::error!("Could not get asset");
                    self.update_status = UpdateStatusType::COMPLETED;
                    return;
                };
                if let Some(current_version) = self.current_version {
                    Updater::ui(ctx, |_ctx, _class| {
                        ui.label("Download emulator");
                        if ui.button("Ok").clicked() {
                            self.update_status = UpdateStatusType::DOWNLOADING;
                            let _asset = asset.clone();
                            tokio::spawn(async move {
                                update_emulator(&_asset).await;
                            });
                        };
                    })
                } else {
                    Updater::ui(ctx, |ctx, class| {
                        ui.label("");
                    })
                };
            }
        }
    }

    fn ui<T>(ctx: &egui::Context, viewport_ui_cb: impl FnMut(&egui::Context, ViewportClass) -> T) {
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("update_window"),
            egui::ViewportBuilder::default()
                .with_title("Update")
                .with_inner_size([200.0, 100.0]),
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
                url: asset.url.clone(),
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

pub async fn update_emulator(asset: &LatestEmulatorAsset) -> Result<()> {
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
    spawn_blocking(move || {
        let zipfile = std::fs::File::open(downloaded_path).unwrap();
        let mut archive = zip::ZipArchive::new(zipfile).unwrap();

        archive.extract(archive_path).unwrap();
    })
    .await?;

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
