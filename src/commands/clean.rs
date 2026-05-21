use anyhow::{Context, Result};
use clap::Args;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use tar::Archive;

use crate::docker::get_file;

#[derive(Args, Debug)]
pub struct CleanArgs {
    /// Base folder — `container-volume` will be appended
    #[arg(short = 'f', long = "base-folder", required = true)]
    pub base_folder: String,
}

pub async fn run(args: CleanArgs) -> Result<()> {
    let folder = format!("{}/container-volume", args.base_folder);
    log::info!("Deleting content of folder: {folder}");

    // Remove each entry inside the container-volume folder
    let dir = fs::read_dir(&folder)
        .with_context(|| format!("Cannot read directory {folder}"))?;

    for entry in dir {
        let entry = entry.context("Failed to read dir entry")?;
        fs::remove_dir_all(entry.path())
            .with_context(|| format!("Failed to remove {:?}", entry.path()))?;
    }

    // Restore default structure from the Docker image
    log::info!("About to revert to default structure on folder [{folder}]");

    let tar_bytes = get_file(
        "trivadis/platys-modern-data-platform", // stack is implicit here, same as Go
        "/opt/mdps-gen/static-data/container-volume",
    )
    .await?;

    let base = Path::new(&args.base_folder);
    let mut archive = Archive::new(std::io::Cursor::new(tar_bytes));

    for entry in archive.entries().context("Failed to iterate tar entries")? {
        let mut entry = entry.context("Bad tar entry")?;
        let header = entry.header();
        let path = entry.path().context("Bad entry path")?;
        let target: PathBuf = base.join(&*path);

        match header.entry_type() {
            tar::EntryType::Directory => {
                if !target.exists() {
                    fs::create_dir_all(&target)
                        .with_context(|| format!("Failed to create dir {:?}", target))?;
                }
            }
            tar::EntryType::Regular => {
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("Failed to create parent {:?}", parent))?;
                }
                let mode = header.mode().unwrap_or(0o644);
                let mut file = open_file_with_mode(&target, mode)?;
                let mut content = Vec::new();
                entry.read_to_end(&mut content).context("Failed to read entry")?;
                std::io::Write::write_all(&mut file, &content)
                    .with_context(|| format!("Failed to write {:?}", target))?;
            }
            _ => {}
        }
    }

    Ok(())
}

#[cfg(unix)]
fn open_file_with_mode(path: &Path, mode: u32) -> Result<fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    fs::OpenOptions::new()
        .create(true)
        .write(true)
        .mode(mode)
        .open(path)
        .with_context(|| format!("Failed to open {:?}", path))
}

#[cfg(not(unix))]
fn open_file_with_mode(path: &Path, _mode: u32) -> Result<fs::File> {
    fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(path)
        .with_context(|| format!("Failed to open {:?}", path))
}
