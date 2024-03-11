use std::{
    env, fs,
    path::{Path, PathBuf},
};

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use resume_tui_data::{Database, Detail, Workplace};
use syn::{Data, Lit, LitStr};

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("source_data.rs");
    fs::write(&dest_path, generate_from_configuration()).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=resume-tui-data/src/codegen.rs");
}

fn generate_from_configuration() -> String {
    let path = if let Ok(cfg) = env::var("RESUME_DATA_FILE_PATH") {
        PathBuf::from(cfg)
    } else {
        PathBuf::from("resume.toml")
    };
    let contents = std::fs::read_to_string(path).expect("file unreadable");
    let database: Database<String> = toml::from_str(&contents).expect("invalid toml");
    let database: Database<LitStr> = database.into();
    let database: TokenStream = database.into();
    quote::quote! {
        use super::*;
        static DATABASE: Database<&'static str> = #database;
    }
    .to_string()
}
