use std::{
    env, fs,
    path::{Path, PathBuf},
};

use proc_macro2::{Span, TokenStream};
use serde::Deserialize;
use syn::{punctuated::Punctuated, LitStr, Token};

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("source_data.rs");
    
    let path = if let Ok(cfg) = env::var("RESUME_DATA_PATH") {
        PathBuf::from(cfg)
    } else {
        PathBuf::from("data")
    };
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", path.display());
    fs::write(&dest_path, generate_from_toml_files(path)).unwrap();
    std::process::Command::new("rustfmt").arg(&dest_path).spawn().unwrap().wait_with_output().unwrap();
}

fn generate_from_toml_files(path: PathBuf) -> String {
    
    let info_text = std::fs::read_to_string(path.join("info.toml")).unwrap();
    let info: Info = toml::from_str(&info_text).unwrap();
    let name = LitStr::new(&info.name, Span::call_site());
    let tag_line = LitStr::new(&info.tag_line, Span::call_site());

    let mut jobs = Punctuated::<TokenStream, Token![,]>::new();
    let jobs_text = std::fs::read_to_string(path.join("jobs.toml")).unwrap();
    let mut jobs_value: Jobs = toml::from_str(&jobs_text).unwrap();
    collect_jobs(&path, &mut jobs_value);
    for job in jobs_value.jobs {
        jobs.push(TokenStream::from(job));
    }
    quote::quote! {
        use resume_tui_data::*;
        pub static DATABASE: Database = Database {
            name: #name,
            tag_line: #tag_line,
            jobs: &[#jobs],
            open_source: &[],
            education: &[]
        };
    }
    .to_string()
}

fn collect_jobs(base_path: impl AsRef<Path>, jobs: &mut Jobs) {
    for job in jobs.jobs.iter_mut() {
        let job_dir = job.id.as_ref().unwrap_or(&job.company);
        let maybe_job_path = base_path.as_ref().join("job_details").join(job_dir);
        if !maybe_job_path.exists() {
            continue;
        }
        for file in std::fs::read_dir(&maybe_job_path).unwrap() {
            let Ok(file) = file else {
                continue;
            };
            if !file.file_type().map(|t| t.is_file()).unwrap_or(false) {
                continue;
            }
            let details_text = std::fs::read_to_string(file.path()).unwrap();
            let detail: Detail = toml::from_str(&details_text).unwrap();
            job.detail.push(detail);
        }
        
    }
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
    #[serde(default)]
    id: Option<String>,
    company: String,
    title: String,
    start: String,
    #[serde(default)]
    end: Option<String>,
    #[serde(default)]
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
            id: _,
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
