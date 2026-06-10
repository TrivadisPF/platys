use anyhow::{Context, Result, bail};
use bollard::Docker;
use bollard::container::{
    Config, CreateContainerOptions, DownloadFromContainerOptions, InspectContainerOptions,
    LogsOptions, RemoveContainerOptions, StartContainerOptions, StopContainerOptions,
};
use bollard::image::CreateImageOptions;
use futures_util::stream::StreamExt;
use std::io::{self, Read};
use std::time::Duration;
use tar::Archive;
use tokio::time::{sleep, timeout};

pub const CONTAINER_NAME: &str = "platys";
pub const CONFIG_FILE_PATH: &str = "/opt/mdps-gen/vars/config.yml";

/// Creates and returns a connected Docker client, pulling the given image first.
pub async fn init_client(stack: &str, version: &str) -> Result<Docker> {
    let docker =
        Docker::connect_with_local_defaults().context("Failed to connect to Docker daemon")?;

    let image_ref = format!("{stack}:{version}");
    log::info!("Pulling image {image_ref} ...");

    let mut pull_stream = docker.create_image(
        Some(CreateImageOptions {
            from_image: image_ref.as_str(),
            ..Default::default()
        }),
        None,
        None,
    );

    while let Some(item) = pull_stream.next().await {
        match item {
            Ok(info) => {
                if let Some(status) = info.status {
                    log::info!("{status}");
                }
            }
            Err(e) => bail!("Image pull error: {e}"),
        }
    }

    Ok(docker)
}

/// Stops then removes a container by ID.
pub async fn stop_remove_container(docker: &Docker, id: &str) -> Result<()> {
    docker
        .stop_container(id, Some(StopContainerOptions { t: 0 }))
        .await
        .context("Failed to stop container")?;

    docker
        .remove_container(
            id,
            Some(RemoveContainerOptions {
                force: true, //remove the container even if it's still running
                ..Default::default()
            }),
        )
        .await
        .context("Failed to remove container")?;

    Ok(())
}

/// Waits for a container to finish, then returns its stdout/stderr logs as a String.
pub async fn wait_and_collect_logs(docker: &Docker, id: &str) -> Result<String> {
    wait_for_container(docker, id).await?;

    let mut log_stream = docker.logs(
        id,
        Some(LogsOptions::<String> {
            stdout: true,
            stderr: true,
            ..Default::default()
        }),
    );

    let mut output = String::new();
    while let Some(item) = log_stream.next().await {
        match item {
            Ok(chunk) => output.push_str(&chunk.to_string()),
            Err(e) => bail!("Log error: {e}"),
        }
    }

    Ok(output)
}

/// Pulls the config.yml from inside a freshly-created container and returns it as a String.
pub async fn pull_config(stack: &str, version: &str) -> Result<String> {
    let docker = init_client(stack, version).await?;

    // Force-remove any existing container with this name first
    let _ = docker
        .remove_container(
            CONTAINER_NAME,
            Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            }),
        )
        .await; // ignore error — container may not exist

    let resp = docker
        .create_container(
            Some(CreateContainerOptions {
                name: CONTAINER_NAME,
                platform: None,
            }),
            Config {
                image: Some(format!("{stack}:{version}")),
                tty: Some(true),
                ..Default::default()
            },
        )
        .await
        .context("Failed to create container")?;

    let container_id = resp.id.clone();

    // Run everything in a closure so we can always clean up
    let result = async {
        docker
            .start_container(&container_id, None::<StartContainerOptions<String>>)
            .await
            .context("Failed to start container")?;

        wait_for_container(&docker, &container_id).await?;

        let logs = wait_and_collect_logs(&docker, &container_id).await?;
        if !logs.is_empty() {
            print!("{logs}");
        }

        download_file_as_string(&docker, &container_id, CONFIG_FILE_PATH).await
    }
    .await;

    // Always clean up, regardless of success or failure
    let _ = docker
        .remove_container(
            &container_id,
            Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            }),
        )
        .await;

    result
}

/// Copies a file/folder out of a container (without version in the image tag).
/// Returns the raw tar bytes.
pub async fn get_file(stack: &str, file_path: &str) -> Result<Vec<u8>> {
    let docker = init_client(stack, "latest").await?;

    let resp = docker
        .create_container(
            Some(CreateContainerOptions {
                name: CONTAINER_NAME,
                platform: None,
            }),
            Config {
                image: Some(stack.to_string()),
                tty: Some(true),
                ..Default::default()
            },
        )
        .await
        .context("Failed to create container")?;

    docker
        .start_container(&resp.id, None::<StartContainerOptions<String>>)
        .await
        .context("Failed to start container")?;

    let logs = wait_and_collect_logs(&docker, &resp.id).await?;
    if !logs.is_empty() {
        print!("{logs}");
    }

    let mut byte_stream = docker.download_from_container(
        &resp.id,
        Some(DownloadFromContainerOptions { path: file_path }),
    );

    let mut tar_bytes: Vec<u8> = Vec::new();
    while let Some(chunk) = byte_stream.next().await {
        tar_bytes.extend_from_slice(&chunk.context("Failed to read tar stream")?);
    }

    stop_remove_container(&docker, &resp.id).await?;

    Ok(tar_bytes)
}

/// Extracts the first file entry from a Docker `download_from_container`
/// tar stream and returns its contents as a String.
fn extract_single_file_tar(tar_bytes: Vec<u8>) -> Result<String> {
    let mut archive = Archive::new(io::Cursor::new(tar_bytes));
    let entry = archive
        .entries()
        .context("Failed to iterate tar entries")?
        .next()
        .context("Tar file was empty")?;

    let mut entry = entry.context("Bad tar entry")?;
    let mut content = String::new();
    entry
        .read_to_string(&mut content)
        .context("Failed to read config entry from tar")?;

    Ok(content)
}

async fn download_file_as_string(
    docker: &Docker,
    container_id: &str,
    path: &str,
) -> Result<String> {
    let mut byte_stream =
        docker.download_from_container(container_id, Some(DownloadFromContainerOptions { path }));
    let mut tar_bytes: Vec<u8> = Vec::new();
    while let Some(chunk) = byte_stream.next().await {
        tar_bytes.extend_from_slice(&chunk.context("Failed to read tar stream")?);
    }

    extract_single_file_tar(tar_bytes)
}

pub async fn wait_for_container(docker: &Docker, id: &str) -> Result<()> {
    timeout(Duration::from_secs(120), async {
        loop {
            let info = docker
                .inspect_container(id, None::<InspectContainerOptions>)
                .await
                .context("Failed to inspect container")?;

            let running = info.state.and_then(|s| s.running).unwrap_or(false);

            if !running {
                break;
            }

            sleep(Duration::from_millis(500)).await;
        }
        Ok(())
    })
    .await
    .context("Container timed out after 120 seconds")?
}
