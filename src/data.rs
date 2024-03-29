#[derive(Debug, Clone)]
pub struct Database {
    pub name: &'static str,
    pub tag_line: &'static str,
    pub github: Option<&'static str>,
    pub linkedin: Option<&'static str>,
    pub jobs: &'static [Workplace],
    pub open_source: &'static [Project],
    pub education: &'static [School],
}

#[derive(Debug, Clone)]
pub struct Workplace {
    pub name: &'static str,
    pub title: &'static str,
    pub start: &'static str,
    pub end: Option<&'static str>,
    pub details: &'static [Detail],
}

#[derive(Debug, Clone)]
pub struct Detail {
    pub headline: &'static str,
    pub snippet: &'static str,
    pub detail: &'static str,
}

#[derive(Debug, Clone)]
pub struct Project {
    pub name: &'static str,
    pub short_desc: &'static str,
    pub long_desc: &'static str,
    pub sub_projects: &'static [Project],
}

#[derive(Debug, Clone)]
pub struct School {
    pub name: &'static str,
    pub graduated: Option<&'static str>,
    pub desc: &'static str,
}

pub mod source {
    use super::*;
    include!(concat!(env!("OUT_DIR"), "/source_data.rs"));
}
