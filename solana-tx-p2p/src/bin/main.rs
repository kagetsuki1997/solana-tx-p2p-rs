// BEGIN LINTS
#![allow(unexpected_cfgs)]
#![cfg_attr(
    feature = "clippy",
    cfg_attr(feature = "c_unwind", deny(ffi_unwind_calls)),
    cfg_attr(feature = "strict_provenance", deny(fuzzy_provenance_casts, lossy_provenance_casts)),
    cfg_attr(feature = "multiple_supertrait_upcastable", deny(multiple_supertrait_upcastable)),
    cfg_attr(feature = "must_not_suspend", deny(must_not_suspend)),
    cfg_attr(feature = "type_privacy_lints", deny(unnameable_types)),
    deny(
      // move to workspace after stabilize
      report-in-external-macro,
      warnings,
    )
)]
// END LINTS
mod cli;
mod command;
mod env;
mod error;
mod tracing;

use std::process;

use clap::Parser;

use self::cli::Cli;

#[cfg(not(miri))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    if let Err(err) = Cli::parse().run() {
        let message = err.to_string();
        eprintln!("{message}");
        process::exit(-1);
    }
}
