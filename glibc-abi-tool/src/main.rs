// Copyright 2022 Gregory Szorc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub mod abilist;
pub mod repo;
pub mod report;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// A fictional versioning CLI
#[derive(Debug, Parser)]
#[command(name = "libc-targeting")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    GlibcAbilistSync {
        /// Path to local glibc Git clone.
        glibc_repo: PathBuf,
        dest_dir: PathBuf,
    },

    GlibcSymbolReport {
        /// Path to local glibc Git clone.
        glibc_repo: PathBuf,
        dest_dir: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::GlibcAbilistSync {
            glibc_repo,
            dest_dir,
        } => report::write_json_metadata(glibc_repo, dest_dir),
        Commands::GlibcSymbolReport {
            glibc_repo,
            dest_dir,
        } => {
            let repo = repo::Repo::open(glibc_repo)?;
            report::write_report(&repo, &dest_dir)
        }
    }
}
