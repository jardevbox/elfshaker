//! SPDX-License-Identifier: Apache-2.0
//! Copyright (C) 2021 Arm Limited or its affiliates and Contributors. All rights reserved.

use clap::{App, Arg, ArgMatches};
use std::{error::Error, ops::ControlFlow};

use super::utils::{format_size, open_repo_from_cwd};
use elfshaker::repo::{PackId, Repository};

pub(crate) const SUBCOMMAND: &str = "list";

pub(crate) fn run(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let packs = matches.values_of_lossy("pack");
    let format = matches
        .value_of_lossy("format")
        .expect("<format> not provided");

    let repo = open_repo_from_cwd()?;

    let packs = packs
        .map(|packs| packs.iter().cloned().map(PackId::Pack).collect())
        .unwrap_or(repo.packs()?);

    print_snapshots(&repo, &packs, &format)?;

    Ok(())
}

pub(crate) fn get_app() -> App<'static, 'static> {
    App::new(SUBCOMMAND)
        .about("Prints the list of snapshots available in the repository.")
        .arg(
            Arg::with_name("pack")
                .index(1)
                .required(false)
                .multiple(true)
                .help("Prints the contents of the specified packs."),
        )
        .arg(
            Arg::with_name("format")
                .long("format")
                .default_value("%s")
                .help(
                    "Pretty-print each result in the given format, where \
                    <format> is a string containing one or more of the \
                    following placeholders:\n\
                    \t%s - fully-qualified snapshot\n\
                    \t%t - snapshot tag\n\
                    \t%h - human-readable size\n\
                    \t%b - size in bytes\n\
                    \t%n - number of files\n",
                ),
        )
}

fn format_snapshot_row(
    fmt: &str,
    pack_id: &PackId,
    snapshot: &str,
    size: u64,
    file_count: usize,
) -> String {
    fmt.to_owned()
        .replace("%s", &format!("{}:{}", pack_id, snapshot))
        .replace("%t", snapshot)
        .replace("%h", &format_size(size))
        .replace("%b", &size.to_string())
        .replace("%n", &file_count.to_string())
}

fn print_snapshots(
    repo: &Repository,
    pack_ids: &[PackId],
    fmt: &str,
) -> Result<(), Box<dyn Error>> {
    let mut lines = vec![];

    for pack_id in pack_ids {
        repo.load_index(pack_id)?
            .for_each_snapshot(|snapshot, entries| {
                let file_count = entries.len();
                let file_size = entries.iter().map(|entry| entry.metadata.size).sum();

                lines.push(format_snapshot_row(
                    fmt, pack_id, snapshot, file_size, file_count,
                ));
                ControlFlow::<(), ()>::Continue(())
            })?;
    }

    lines.sort();

    for line in lines {
        println!("{}", line);
    }

    Ok(())
}
