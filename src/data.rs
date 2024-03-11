pub static WORK: &[Workplace<&'static str>] = &[
    // Workplace {
    //     name: "SmartThings",
    //     title: "Staff Software Engineer",
    //     start: "March 2020",
    //     end: None,
    //     details: [
    //         Detail {
    //             short: "Software Achievements",
    //             long: "What I accomplished at this job",
    //             detail: "",
    //         },
    //         Detail {
    //             short: "Leadership Achievements",
    //             long: "What I accomplished at this job",
    //             detail: "",
    //         },
    //     ]
    //     .into_iter()
    //     .collect(),
    // },
    // Workplace {
    //     name: "Solution Design Group",
    //     title: "Software Engineer",
    //     start: "",
    //     end: Some("March 2020"),
    //     details: [].into_iter().collect(),
    // },
    // Workplace {
    //     name: "United Subcontractors",
    //     title: "Software Engineer",
    //     start: "",
    //     end: Some(""),
    //     details: [].into_iter().collect(),
    // },
    // Workplace {
    //     name: "Professional Data Analysts",
    //     title: "Business Analyst",
    //     start: "",
    //     end: Some(""),
    //     details: [].into_iter().collect(),
    // },
];
pub use resume_tui_data::*;

mod source {
    include!(concat!(env!("OUT_DIR"), "/source_data.rs"));
}
