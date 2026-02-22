use crate::utils::{ProgressBar, ProgressReader};
use anyhow::{Context, Result};
use bon::Builder;
use clap::Args;
use flate2::bufread::GzDecoder;
use kdam::term;
use reqwest::blocking::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self},
    io::{BufReader, IsTerminal},
    path::PathBuf,
};
use tar::Archive;
#[derive(Args, Debug)]
pub struct BenchPullArgs {
    #[arg(
        short,
        long,
        help = "Path to where the benchmark is to be saved.",
        value_name = "DIR"
    )]
    pub dest: PathBuf,

    #[arg(
        long,
        help = "Version of scarfbench to pull.",
        value_name = "VERSION"
    )]
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Release {
    assets: Vec<Asset>,
}
#[derive(Debug, Serialize, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Builder)]
pub struct PullScarfBench {
    /// Name (prefix) of the benchmark
    #[builder(default= "benchmark-v".to_string())]
    pub asset_name_prefix: String,

    /// Version of scarfbench to pull. Default is latest.
    pub version: Option<String>,

    /// Where to download the benchmark to.
    pub dest_dir: PathBuf,
}

impl PullScarfBench {
    fn github_token() -> Option<String> {
        std::env::var("SCARF_BENCH_GITHUB_TOKEN")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .or_else(|| {
                std::env::var("GITHUB_TOKEN")
                    .ok()
                    .filter(|v| !v.trim().is_empty())
            })
    }

    fn maybe_auth(
        request: RequestBuilder,
        token: Option<&str>,
    ) -> RequestBuilder {
        match token {
            Some(token) => request.bearer_auth(token),
            None => request,
        }
    }

    pub fn exec(&self) -> anyhow::Result<i32> {
        let token = Self::github_token();
        let client = Client::builder().user_agent("scarf-cli").build()?;

        // Get the download URL
        let api_url = match self.version.as_deref() {
            Some(v) => format!(
                "https://api.github.com/repos/scarfbench/benchmark/releases/tags/{}",
                v
            ),
            None => format!(
                "https://api.github.com/repos/scarfbench/benchmark/releases/latest"
            ),
        };
        log::info!("Downloading from {api_url}");

        // Get releases
        let release_response = Self::maybe_auth(client.get(&api_url), token.as_deref())
            .header("User-Agent", "scarf")
            .send()
            .with_context(|| format!("Unable to fetch the release metadata from {api_url}"))?
            .error_for_status()
            .with_context(|| {
                format!(
                    "GitHub API returned an error status while reading {api_url}. If this repo is private, set SCARF_BENCH_GITHUB_TOKEN (or GITHUB_TOKEN) with contents:read access to scarfbench/benchmark."
                )
            })?;

        let releases: Release =
            release_response.json().context("Failed to parse release JSON")?;

        // Get the asset to download
        let asset = releases
            .assets
            .into_iter()
            .find(|predicate| predicate.name.contains(&self.asset_name_prefix))
            .with_context(|| {
                return format!(
                    "There are no release assets that start with {} available at {}",
                    &self.asset_name_prefix, api_url
                );
            })?;

        // Create the save destination
        fs::create_dir_all(&self.dest_dir).with_context(|| {
            format!(
                "Failed to create a directory at {}",
                &self.dest_dir.to_string_lossy()
            )
        })?;

        let response = Self::maybe_auth(client.get(&asset.browser_download_url), token.as_deref())
            .send()
            .with_context(|| {
                format!(
                    "Failed to download the asset from {}",
                    asset.browser_download_url
                )
            })?
            .error_for_status()
            .with_context(|| {
                format!(
                    "Asset download returned an error status from {}. If this release is private, set SCARF_BENCH_GITHUB_TOKEN (or GITHUB_TOKEN) with access to scarfbench/benchmark.",
                    asset.browser_download_url
                )
            })?;

        // Get the total size of the payload (our benchmark tar.gz we are downloading)
        let total_size =
            response.content_length().map(|s| s as usize).unwrap_or(0);

        // Set up terminal to tell kdam if we are in a active terminal (for colors and ansi stuff)
        term::init(std::io::stderr().is_terminal());
        term::hide_cursor()?;

        // initialize our progress bar
        let pb = total_size.progress("Downloading scarfbench");

        let pr =
            ProgressReader::new(BufReader::new(response), pb, Some(total_size));

        // Extract by streaming the response to tar directly
        let tar = GzDecoder::new(pr);
        let mut archive = Archive::new(tar);
        archive.unpack(&self.dest_dir).with_context(|| {
            format!("Failed to extract into {}", self.dest_dir.display())
        })?;
        term::show_cursor()?;
        Ok(0)
    }
}

/// A simple list subcommand that lists all the benchmark applications as a table.
pub fn run(bench_pull_args: BenchPullArgs) -> Result<i32> {
    PullScarfBench::builder()
        .maybe_version(bench_pull_args.version)
        .dest_dir(bench_pull_args.dest)
        .build()
        .exec()
}
