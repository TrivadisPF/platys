use anyhow::{Context, Result};
use bollard::container::{
    Config, CreateContainerOptions, LogsOptions, StartContainerOptions,
};
use bollard::image::CreateImageOptions;
use clap::Args;
use futures_util::StreamExt;
use regex::Regex;

use crate::cli::DEFAULT_STACK;
use crate::docker::{stop_remove_container, wait_for_container};

#[derive(Args, Debug)]
pub struct StacksArgs {
    /// Version of the stack
    #[arg(short = 'w', long = "stack-version", default_value = "latest")]
    pub stack_version: String,
}

pub async fn run(args: StacksArgs) -> Result<()> {
    let stack = DEFAULT_STACK;
    let version = &args.stack_version;
    let image_ref = format!("{stack}:{version}");

    let docker = bollard::Docker::connect_with_local_defaults()
        .context("Failed to connect to Docker")?;

    // Pull the image
    let mut pull_stream = docker.create_image(
        Some(CreateImageOptions {
            from_image: image_ref.as_str(),
            ..Default::default()
        }),
        None,
        None,
    );
    while let Some(item) = pull_stream.next().await {
        if let Ok(info) = item {
            if let Some(status) = info.status {
                println!("{status}");
            }
        }
    }

    // Create container, run `ls /opt/mdps-gen/seed-stacks`
    let resp = docker
        .create_container(
            Some(CreateContainerOptions {
                name: "platys",
                platform: None,
            }),
            Config {
                image: Some(image_ref.as_str()),
                tty: Some(true),
                cmd: Some(vec!["ls", "/opt/mdps-gen/seed-stacks"]),
                ..Default::default()
            },
        )
        .await
        .context("Failed to create container")?;

    docker
        .start_container(&resp.id, None::<StartContainerOptions<String>>)
        .await
        .context("Failed to start container")?;   

    wait_for_container(&docker, &image_ref).await?;

    let mut log_stream = docker.logs(
        &resp.id,
        Some(LogsOptions::<String> {
            stdout: true,
            stderr: false,
            ..Default::default()
        }),
    );

    let re = Regex::new(r"([A-Z0-9_-]+)\.yml").expect("valid regex");
    let mut stacks: Vec<String> = Vec::new();

    while let Some(item) = log_stream.next().await {
        if let Ok(chunk) = item {
            let line = chunk.to_string();
            for cap in re.captures_iter(&line) {
                stacks.push(cap[1].to_string());
            }
        }
    }

    stop_remove_container(&docker, &resp.id).await?;

    println!("Available stacks:");
    for s in stacks {
        println!("  {s}");
    }

    Ok(())
}
