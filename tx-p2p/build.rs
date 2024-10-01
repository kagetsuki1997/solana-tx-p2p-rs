use std::{env, path::PathBuf};

const CARGO_OUT_DIR_ENV: &str = "OUT_DIR";

macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning=\r\x1B[32;1m    {} {}\x1B[0m", env!("CARGO_PKG_NAME"), format!($($tokens)*))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(
        env::var(CARGO_OUT_DIR_ENV)
            .expect("cargo must provide `{CARGO_OUT_DIR_ENV}` environment variable; qed"),
    );
    p!("{CARGO_OUT_DIR_ENV}={}", out_dir.display());

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("file_descriptor_set.pb"))
        .compile(&["../proto/p2p/p2p.proto", "../proto/p2p/v1/p2p_service.proto"], &[
            "../proto", "proto",
        ])?;

    Ok(())
}
