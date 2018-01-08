//! This tiny build script ensures that the crate is not compiled with an
//! incompatible version of rust.
//! This scipt was stolen from `rocket_codegen`.

extern crate ansi_term;
extern crate version_check;

use ansi_term::Color::{Red, Yellow, Blue, White};
use version_check::{is_nightly, is_min_version, is_min_date};

// Specifies the minimum nightly version that is targetted
// Note that sometimes the `rustc` date might be older than the nightly version,
// usually one day older
const MIN_DATE: &'static str = "2017-12-17";
const MIN_VERSION: &'static str = "1.24.0-nightly";

// Convenience macro for writing to stderr.
macro_rules! printerr {
    ($($arg:tt)*) => ({
        use std::io::prelude::*;
        write!(&mut ::std::io::stderr(), "{}\n", format_args!($($arg)*))
            .expect("Failed to write to stderr.")
    })
}

fn main() {
    let ok_nightly = is_nightly();
    let ok_version = is_min_version(MIN_VERSION);
    let ok_date = is_min_date(MIN_DATE);

    let print_version_err = |version: &str, date: &str| {
        printerr!(
            "{} {}. {} {}.",
            White.paint("Installed version is:"),
            Yellow.paint(format!("{} ({})", version, date)),
            White.paint("Minimum required:"),
            Yellow.paint(format!("{} ({})", MIN_VERSION, MIN_DATE))
        );
    };

    match (ok_nightly, ok_version, ok_date) {
        (Some(is_nightly), Some((ok_version, version)), Some((ok_date, date))) => {
            if !is_nightly {
                printerr!(
                    "{} {}",
                    Red.bold().paint("Error:"),
                    White.paint("rocket_cors requires a nightly version of Rust.")
                );
                print_version_err(&*version, &*date);
                printerr!(
                    "{}{}{}",
                    Blue.paint("See the README ("),
                    White.paint("https://github.com/lawliet89/rocket_cors"),
                    Blue.paint(") for more information.")
                );
                panic!("Aborting compilation due to incompatible compiler.")
            }

            if !ok_version || !ok_date {
                printerr!(
                    "{} {}",
                    Red.bold().paint("Error:"),
                    White.paint("rocket_cors requires a more recent version of rustc.")
                );
                printerr!(
                    "{}{}{}",
                    Blue.paint("Use `"),
                    White.paint("rustup update"),
                    Blue.paint("` or your preferred method to update Rust.")
                );
                print_version_err(&*version, &*date);
                panic!("Aborting compilation due to incompatible compiler.")
            }
        }
        _ => {
            println!(
                "cargo:warning={}",
                "rocket_cors was unable to check rustc compatibility."
            );
            println!(
                "cargo:warning={}",
                "Build may fail due to incompatible rustc version."
            );
        }
    }
}
