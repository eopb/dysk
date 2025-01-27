mod args;
mod col;
mod col_expr;
mod cols;
mod csv;
mod filter;
mod json;
mod list_cols;
mod normal;
mod order;
mod sorting;
mod table;
mod units;

use {
    crate::{
        args::*,
        normal::*,
    },
    std::{
        fs,
        os::unix::fs::MetadataExt,
    },
};

#[allow(clippy::match_like_matches_macro)]
fn main() {
    let args: Args = argh::from_env();
    if args.version {
        println!("dysk {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    if args.list_cols {
        list_cols::print(args.color());
        return;
    }
    let mut options = lfs_core::ReadOptions::default();
    options.remote_stats(match args.remote_stats.value() {
        Some(false) => false,
        _ => true,
    });
    let mut mounts = match lfs_core::read_mounts(&options) {
        Ok(mounts) => mounts,
        Err(e) => {
            eprintln!("Error reading mounts: {}", e);
            return;
        }
    };
    if !args.all {
        mounts.retain(is_normal);
    }
    if let Some(path) = &args.path {
        let md = match fs::metadata(path) {
            Ok(md) => md,
            Err(e) => {
                eprintln!("Can't read {:?} : {}", path, e);
                return;
            }
        };
        let dev = lfs_core::DeviceId::from(md.dev());
        mounts.retain(|m| m.info.dev == dev);
    }
    args.sort.sort(&mut mounts);
    let mounts = match args.filter.filter(&mounts) {
        Ok(mounts) => mounts,
        Err(e) => {
            eprintln!("Error in filter evaluation: {}", e);
            return;
        }
    };
    if args.csv {
        csv::print(&mounts, &args).expect("writing csv failed");
        return;
    }
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json::output_value(&mounts, args.units)).unwrap()
        );
        return;
    }
    if mounts.is_empty() {
        println!("no mount to display - try\n    dysk -a");
        return;
    }
    table::print(&mounts, args.color(), &args);
}

