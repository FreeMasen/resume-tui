use std::{
    env, fs,
    path::{Path, PathBuf},
};

use proc_macro2::{Span, TokenStream};
use serde::Deserialize;
use syn::{punctuated::Punctuated, Data, Lit, LitStr, Token};

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("source_data.rs");
    fs::write(&dest_path, generate_from_configuration()).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=resume-tui-data/src/codegen.rs");
    std::process::Command::new("rustfmt").arg(&dest_path).spawn().unwrap().wait_with_output().unwrap();
}

fn generate_from_configuration() -> String {
    let path = if let Ok(cfg) = env::var("RESUME_DATA_PATH") {
        PathBuf::from(cfg)
    } else {
        PathBuf::from("data")
    };
    let info_text = std::fs::read_to_string(path.join("info.toml")).unwrap();
    let info: Info = toml::from_str(&info_text).unwrap();
    let name = LitStr::new(&info.name, Span::call_site());
    let tag_line = LitStr::new(&info.tag_line, Span::call_site());

    let mut jobs = Punctuated::<TokenStream, Token![,]>::new();
    let jobs_text = std::fs::read_to_string(path.join("jobs.toml")).unwrap();
    let jobs_value: Jobs = toml::from_str(&jobs_text).unwrap();
    for job in jobs_value.jobs {
        jobs.push(TokenStream::from(job));
    }
    quote::quote! {
        use resume_tui_data::*;
        static DATABASE: Database = Database {
            name: #name,
            tag_line: #tag_line,
            jobs: &[#jobs],
            open_source: &[],
            education: &[]
        };
    }
    .to_string()
}

#[derive(Deserialize)]
pub struct Info {
    name: String,
    tag_line: String,
}
#[derive(Debug, Deserialize)]
pub struct Jobs {
    #[serde(rename = "job")]
    jobs: Vec<Job>
}

#[derive(Debug, Deserialize)]
pub struct Job {
    company: String,
    title: String,
    start: String,
    end: Option<String>,
    detail: Vec<Detail>,
}
#[derive(Debug, Deserialize)]
pub struct Detail {
    short: String,
    long: String,
    detail: String,
}

impl From<Job> for TokenStream {
    fn from(value: Job) -> Self {
        let Job {
            company,
            title,
            start,
            end,
            detail,
        } = value;
        let company = LitStr::new(&company, Span::call_site());
        let title = LitStr::new(&title, Span::call_site());
        let start = LitStr::new(&start, Span::call_site());
        let end = end.map(|end| {
            let end = LitStr::new(&end, Span::call_site());
            quote::quote! {
                Some(#end)
            }
        }).unwrap_or_else(|| {
            quote::quote!(None)
        });
        let mut details = Punctuated::<TokenStream, Token![,]>::new();
        for detail in detail {
            details.push(TokenStream::from(detail));
        }
        quote::quote! {
            Workplace {
                name: #company,
                title: #title,
                start: #start,
                end: #end,
                details: &[#details],
            }
        }
    }
}

impl From<Detail> for TokenStream {
    fn from(value: Detail) -> Self {
        let Detail {
            short,
            long,
            detail,
        } = value;
        let short = LitStr::new(&short, Span::call_site());
        let long = LitStr::new(&long, Span::call_site());
        let detail = LitStr::new(&detail, Span::call_site());
        quote::quote! {
            Detail {
                short: #short,
                long: #long,
                detail: #detail,
            }
        }
    }
}
