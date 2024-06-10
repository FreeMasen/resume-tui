use std::{
    env, fs,
    path::{Path, PathBuf},
};

use proc_macro2::{Span, TokenStream};
use quote::quote;
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
    emit_rebuild_directives(&path);

    let rust = generate_from_toml_files(path);
    fs::write(dest_path, &rust).unwrap();
    fs::write("debug.rs", &rust).unwrap();
    std::process::Command::new("rustfmt")
        .arg("debug.rs")
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
}

fn emit_rebuild_directives(path: impl AsRef<Path>) {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", path.as_ref().display());
}

fn generate_from_toml_files(path: PathBuf) -> String {
    let info_text = std::fs::read_to_string(path.join("info.toml")).unwrap();
    let info: Info = toml::from_str(&info_text).unwrap();
    let name = LitStr::new(&info.name, Span::call_site());
    let tag_line = LitStr::new(&info.tag_line, Span::call_site());
    let github = optional_str(info.github);
    let linkedin = optional_str(info.linkedin);
    let email = optional_str(info.email);
    let phone = optional_str(info.phone);
    let jobs_text = std::fs::read_to_string(path.join("jobs.toml")).unwrap();
    let mut jobs_value: Jobs = toml::from_str(&jobs_text).unwrap();
    collect_jobs(&path, &mut jobs_value);
    let jobs = TokenStream::from(jobs_value);
    let oss_text = std::fs::read_to_string(path.join("oss.toml")).unwrap();
    let mut oss: Projects = toml::from_str(&oss_text).unwrap();
    collect_oss(&path, &mut oss);
    let oss = TokenStream::from(oss);
    let edu_text = std::fs::read_to_string(path.join("edu.toml")).unwrap();
    let edu: Education = toml::from_str(&edu_text).unwrap();
    let edu = TokenStream::from(edu);
    quote::quote! {
        pub static DATABASE: Database = Database {
            name: #name,
            tag_line: #tag_line,
            github: #github,
            linkedin: #linkedin,
            email: #email,
            phone: #phone,
            jobs: #jobs,
            open_source: #oss,
            education: #edu,
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
            job.details.push(detail);
        }
    }
}
fn collect_oss(base_path: impl AsRef<Path>, projects: &mut Projects) {
    for proj in projects.projects.iter_mut() {
        let proj_dir = proj.id.as_ref().unwrap_or(&proj.name);
        let maybe_proj_path = base_path.as_ref().join("oss_details").join(proj_dir);
        if !maybe_proj_path.exists() {
            continue;
        }
        for file in std::fs::read_dir(&maybe_proj_path).unwrap() {
            let Ok(file) = file else {
                continue;
            };
            if !file.file_type().map(|t| t.is_file()).unwrap_or(false) {
                continue;
            }
            let details_text = std::fs::read_to_string(file.path()).unwrap();
            let detail: Project = toml::from_str(&details_text).unwrap();
            proj.sub_projects.push(detail);
        }
    }
}

#[derive(Deserialize)]
pub struct Info {
    name: String,
    tag_line: String,
    github: Option<String>,
    linkedin: Option<String>,
    email: Option<String>,
    phone: Option<String>,
}
#[derive(Debug, Deserialize)]
pub struct Jobs {
    #[serde(rename = "job")]
    jobs: Vec<Job>,
}

impl From<Jobs> for TokenStream {
    fn from(value: Jobs) -> Self {
        let Jobs { jobs } = value;
        let jobs: Punctuated<TokenStream, Token![,]> =
            jobs.into_iter().map(TokenStream::from).collect();
        quote! {
            &[#jobs]
        }
    }
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
    #[serde(default, alias = "detail")]
    details: Vec<Detail>,
}
#[derive(Debug, Deserialize)]
pub struct Detail {
    headline: String,
    snippet: String,
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
            details: detail,
        } = value;
        let company = LitStr::new(&company, Span::call_site());
        let title = LitStr::new(&title, Span::call_site());
        let start = LitStr::new(&start, Span::call_site());
        let end = optional_str(end);
        let details: Punctuated<TokenStream, Token![,]> =
            detail.into_iter().map(TokenStream::from).collect();
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
            headline,
            snippet,
            detail,
        } = value;
        let headline = LitStr::new(&headline, Span::call_site());
        let snippet = LitStr::new(&snippet, Span::call_site());
        let detail = LitStr::new(&detail, Span::call_site());
        quote::quote! {
            Detail {
                headline: #headline,
                snippet: #snippet,
                detail: #detail,
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Projects {
    #[serde(default, rename = "project")]
    projects: Vec<Project>,
}

impl From<Projects> for TokenStream {
    fn from(value: Projects) -> Self {
        let projects: Punctuated<TokenStream, Token![,]> =
            value.projects.into_iter().map(TokenStream::from).collect();
        quote::quote!(&[#projects])
    }
}

impl From<Project> for TokenStream {
    fn from(value: Project) -> Self {
        let Project {
            long_desc,
            name,
            short_desc,
            sub_projects,
            id: _,
        } = value;
        let long_desc = LitStr::new(&long_desc, Span::call_site());
        let name = LitStr::new(&name, Span::call_site());
        let short_desc = LitStr::new(&short_desc, Span::call_site());

        let sub_projects: Punctuated<TokenStream, Token![,]> =
            sub_projects.into_iter().map(TokenStream::from).collect();
        quote! {
            Project {
                name: #name,
                short_desc: #short_desc,
                long_desc: #long_desc,
                sub_projects: &[#sub_projects],
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Project {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    pub short_desc: String,
    pub long_desc: String,
    #[serde(default, alias = "sub_project")]
    pub sub_projects: Vec<Project>,
}

impl From<School> for TokenStream {
    fn from(value: School) -> Self {
        let School {
            name,
            graduated,
            desc,
        } = value;
        let name = LitStr::new(&name, Span::call_site());
        let graduated = optional_str(graduated);
        let desc = LitStr::new(&desc, Span::call_site());

        quote! {
            School {
                name: #name,
                graduated: #graduated,
                desc: #desc,
            }
        }
    }
}

impl From<Education> for TokenStream {
    fn from(value: Education) -> Self {
        let schools: Punctuated<TokenStream, Token![,]> =
            value.schools.into_iter().map(TokenStream::from).collect();
        quote::quote!(&[#schools])
    }
}
#[derive(Debug, Clone, Deserialize)]
pub struct Education {
    #[serde(alias = "school")]
    schools: Vec<School>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct School {
    pub name: String,
    pub graduated: Option<String>,
    pub desc: String,
}

fn optional_str(value: Option<String>) -> TokenStream {
    value
        .map(|s| {
            let end = LitStr::new(&s, Span::call_site());
            quote::quote! {
                Some(#end)
            }
        })
        .unwrap_or_else(|| quote::quote!(None))
}
