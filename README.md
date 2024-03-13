# Resume TUI

This project is an attempt to make an interactive resume as a TUI application similar to efforts
like [This 2d animation portfolio](https://github.com/JSLegendDev/2d-portfolio-kaboom).

## Installation

TODO

## How it works

The application itself is built with the data needed, to achieve this I've setup a build script
to generate a rust module that is only present at built time named `source_data.rs`

In a `build.rs`, the `DATABASE` static variable is generated by parsing a handful of toml files in a
specific directory layout corrisponding to the details of each view. The layout is outlined below.
The default location for this is a `data` directory in the current working directory when building,
this can be overridden by using the `RESUME_DATA_PATH` environment variable.

### Directory Layout

```text
./data
├── edu.toml
├── info.toml
├── job_details
│   └── <Job Name or Id>
│       ├── bullet-1.toml
│       └── bullet-2.toml
├── jobs.toml
├── oss.toml
└── oss_details
    └── <Project Name or Id>
        ├── subproject-1.toml
        ├── subproject-2.toml
        └── subproject-4.toml
```

#### `info.toml`

The primary entrypoint for the Home page is the `info.toml` file where the base information is
stored; it includes a name and a "tag line".

<details>

<summary>JSON Schema</summary>

```json
{
    "title": "Info",
    "description": "Basic information",
    "type": "object",
    "properties": {
        "name": {
            "description": "The name displayed on the Home page.",
            "type": "string",
        },
        "tag_line": {
            "description": "The tagline to display below the name on the Home page.",
            "type": "string"
        }
    }
}
```

</details>

#### `jobs.toml`

This file is the entry point for the professional experience portion of the resume. The top level is
an array of `Job` objects under the key `job` or `jobs` [^1].

<details>

<summary>JSON Schema</summary>

```json
{
    "title": "Jobs",
    "description": "List of job details",
    "type": "object",
    "properties": {
        "jobs": {
            "description": "list of jobs",
            "type": "array",
            "items": {
                "type": "Job"
            },
        }
    }
}
```

</details>

Each `Job` object contains some basic information about the position including company name, title,
start date, optional end date, a list of details and an optional id. The list of details can either
be included directly in the toml file or can be stored in the `job_details` directory. `job_details`
is layed as directories, each directory will be named have either the `company` or `id`. The job
details when in the TOML file should be rendered in the order they are defined and any additional
details in teh `job_details` directory will be appended to this list in the order they are returned
from `read_dir`.

<details>

<summary>JSON Schema</summary>

```json
{
    "title": "Job",
    "description": "Overview of each job",
    "type": "object",
    "patternProperties": {
        "id": {
            "description": "An optional unique ID for a job used for file based details when representing multiple jobs at the same company.",
            "type": "string"
        },
        "company": {
            "description": "The name of the company",
            "type": "string"
        },
        "title": {
            "description": "Job title at this company",
            "type": "string"
        },
        "start": {
            "description": "The date this position started",
            "type": "string"
        }
        ,
        "end": {
            "description": "The date this position ended, if not provided it will display 'current'",
            "type": "string"
        },
        "details?": {
            "description": "A list of job details",
            "type": "array",
            "items": {
                "type": "JobDetail"
            },
        }
    },
    "required": [ "company", "title", "start" ]
}
```

</details>

Each `Detail` is essentially a bullet point for this job, it will contain a headline, snippet and
long form description.

<details>

<summary>JSON Schema</summary>

```json
{
    "title": "JobDetail",
    "description": "A highlight from a job",
    "type": "object",
    "properties": {
        "headline": {
            "description": "The headline",
            "type": "string"
        },
        "snippet": {
            "description": "A snippet describing this detail",
            "type": "string"
        },
        "detail": {
            "description": "The long form description, Commonmark markdown can be used to style this content",
            "type": "string"
        }
    }
}
```

</details>

#### `oss.toml`

This file is the entry point for the open source work portion of the resume. The top level is
an array of `Project` objects under the key `projects` or `project` [^1].

<details>

<summary>JSON Schema</summary>

```json
{
    "title": "Projects",
    "description": "List of oss projects",
    "type": "object",
    "patternProperties": {
        "projects?": {
            "type": "array",
            "items": {
                "type": "Project"
            },
        }
    }
}
```

</detail>

A `Project` is a recursive data structure for describing open source contribution it contains a
project name, short description, long description and a list of sub projects. This structure
makes it easier to represent GitHub Organizations and their repositories and/or crates
that have workspace crates that deserve additional details.

<details>

<summary>JSON Schema</summary>

```json
{
    "title": "Project",
    "description": "A Project outline",
    "type": "object",
    "properties": {
        "name": {
            "description": "The name of the project",
            "type": "string",
        },
        "short_desc": {
            "description": "A snippet about this project",
            "type": "string"
        },
        "long_desc": {
            "description": "A long form overview of this project, Commonmark markdown can be used to style this content",
            "type": "string"
        },
        "sub_projects": {
            "description": "A list of sub-projects related to this project, this is recursive in nature so these sub projects can also have sub-projects",
            "type": "array",
            "items": {
                "type": "Project"
            }
        }
    },
    "required": ["name", "short_desc", "long_desc"]
}
```

</details>

#### `edu.toml`

This file is the entry point for the open source work portion of the resume. The top level is
an array of `School` objects under the key `schools` or `school` [^1].

<details>

<summary>JSON Schema</summary>

```json
{
    "title": "Education",
    "description": "List of school details",
    "type": "object",
    "patternProperties": {
        "schools?": {
            "type": "array"
            "items": {
                "type": "School"
            },
        },
    }
}
```

</detail>

A `School` is a breif description of an educational experience including the name of the institution
a description of the course of study and an optional graduation date.

<details>

<summary>JSON Schema</summary>

```json
{
    "title": "School",
    "description": "Description of schooling",
    "type": "object",
    "properties": {
        "name": {
            "description": "The name of the institution",
            "type": "string"
        },
        "desc": {
            "description": "A description of this course of study",
            "type": "string"
        },
        "graduation_date": {
            "description": "If completed, when that happened",
            "type": "string",
        }
    },
    "required": ["name", "desc"]
}
```

</details>

[^1]: Because toml allows for 2 array syntaxes, array properties have a serde `alias` to allow
  them to be formatted as either an inline array (`<list-name> = []`) or with the `[[<list-name>]]` syntax. I personally
  find it to be more plesent to use the plural name for the former and non-plural for the latter.
