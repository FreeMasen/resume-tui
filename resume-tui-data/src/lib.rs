

#[derive(Debug, Clone)]
pub struct Database {
    pub name: &'static str,
    pub tag_line: &'static str,
    pub jobs: &'static [Workplace],
    pub open_source: &'static [()],
    pub education: &'static [()],
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
    pub short: &'static str,
    pub long: &'static str,
    pub detail: &'static str,
}
