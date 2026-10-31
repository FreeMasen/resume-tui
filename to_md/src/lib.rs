use std::{
    fmt,
    fs::File,
    io::{BufWriter, Write},
    iter,
    path::Path,
};

use pulldown_cmark::{Alignment, CowStr, Event, HeadingLevel, LinkType, Parser, Tag, TagEnd};
use resume_tui_data::data::{source::DATABASE, Database, Detail, Project, School, Workplace};

pub fn generate_md(path: impl AsRef<Path>) -> Result<(), String> {
    let f = BufWriter::new(
        File::options()
            .read(false)
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)
            .map_err(|e| e.to_string())?,
    );
    let _ = pulldown_cmark_to_cmark::cmark(
        DATABASE.into_markdown(),
        FmtFile(f),
        pulldown_cmark::Options::all(),
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

struct FmtFile(BufWriter<File>);

impl fmt::Write for FmtFile {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.write_all(s.as_bytes()).map_err(|_| std::fmt::Error)
    }
}

pub trait IntoMarkdown {
    fn into_markdown<'a>(&'a self) -> impl Iterator<Item = Event<'a>>;
}

impl IntoMarkdown for Database {
    fn into_markdown<'a>(&'a self) -> impl Iterator<Item = Event<'a>> {
        [
            Event::Start(Tag::Heading {
                level: HeadingLevel::H1,
                id: None,
                classes: Vec::new(),
                attrs: Vec::new(),
            }),
            Event::Text(CowStr::Borrowed(DATABASE.name)),
            Event::End(TagEnd::Heading(HeadingLevel::H1)),
        ]
        .into_iter()
        .chain(database_sub_header(&DATABASE))
        .chain(self.jobs.into_markdown())
        .chain(self.open_source.into_markdown())
        .chain(self.education.into_markdown())
    }
}

fn database_sub_header<'a>(db: &Database) -> impl Iterator<Item = Event<'a>> {
    fn generate_email<'a>(email: &'a str) -> impl Iterator<Item = Event<'a>> {
        wrap_in_tags(
            Tag::Link {
                link_type: LinkType::Email,
                dest_url: format!("<{email}>").into(),
                title: "email".into(),
                id: "email".into(),
            },
            text(email),
            TagEnd::Link,
        )
    }
    fn generate_link<'a>(
        username: &'a str,
        link_prefix: &'a str,
    ) -> impl Iterator<Item = Event<'a>> {
        let id = link_prefix
            .find(".com")
            .and_then(|idx| link_prefix.get(..idx))
            .unwrap_or(link_prefix);
        wrap_in_tags(
            Tag::Link {
                link_type: LinkType::Inline,
                dest_url: format!("https://{link_prefix}/{username}").into(),
                title: id.into(),
                id: id.into(),
            },
            text(username),
            TagEnd::Link,
        )
    }

    fn generate_github<'a>(username: &'a str) -> impl Iterator<Item = Event<'a>> {
        generate_link(username, "github.com/")
    }
    fn generate_linkedin<'a>(username: &'a str) -> impl Iterator<Item = Event<'a>> {
        generate_link(username, "linkedin.com/in/")
    }

    let head = wrap_in_tags(Tag::TableCell, text("Email"), TagEnd::TableCell)
        .chain(wrap_in_tags(
            Tag::TableCell,
            text("Github"),
            TagEnd::TableCell,
        ))
        .chain(wrap_in_tags(
            Tag::TableCell,
            text("LinkedIn"),
            TagEnd::TableCell,
        ));
    let head = wrap_in_tags(Tag::TableHead, head, TagEnd::TableHead);
    let email_cell = wrap_in_tags(
        Tag::TableCell,
        db.email.into_iter().flat_map(generate_email),
        TagEnd::TableCell,
    );
    let gh_cell = wrap_in_tags(
        Tag::TableCell,
        db.github.into_iter().flat_map(generate_github),
        TagEnd::TableCell,
    );
    let li_cell = wrap_in_tags(
        Tag::TableCell,
        db.linkedin.into_iter().flat_map(generate_linkedin),
        TagEnd::TableCell,
    );

    wrap_in_tags(
        Tag::Table(vec![Alignment::Left, Alignment::Center, Alignment::Right]),
        head.chain(wrap_in_tags(
            Tag::TableRow,
            email_cell.chain(gh_cell).chain(li_cell),
            TagEnd::TableRow,
        )),
        TagEnd::Table,
    )
}

impl IntoMarkdown for [Workplace] {
    fn into_markdown<'a>(&'a self) -> impl Iterator<Item = Event<'a>> {
        [
            Event::Start(Tag::Heading {
                level: HeadingLevel::H2,
                id: None,
                classes: Vec::new(),
                attrs: Vec::new(),
            }),
            Event::Text(CowStr::Borrowed("Work")),
            Event::End(TagEnd::Heading(HeadingLevel::H2)),
        ]
        .into_iter()
        .chain(self.into_iter().flat_map(IntoMarkdown::into_markdown))
    }
}

impl IntoMarkdown for Workplace {
    fn into_markdown<'a>(&'a self) -> impl Iterator<Item = Event<'a>> {
        let head = Tag::Heading {
            level: HeadingLevel::H3,
            id: None,
            classes: Vec::new(),
            attrs: Vec::new(),
        };
        let head = Event::Start(head);
        let name = self.name;
        let name = CowStr::Borrowed(name);
        let name = Event::Text(name);
        let base = [head, name, Event::End(TagEnd::Heading(HeadingLevel::H3))];
        let dates = wrap_in_tags(
            Tag::Paragraph,
            text(self.start)
                .chain(text(" - "))
                .chain(text(self.end.unwrap_or("Current"))),
            TagEnd::Paragraph,
        );

        base.into_iter()
            .chain(dates)
            .chain(wrap_in(
                Event::Start(Tag::Strong),
                self.title.into_markdown(),
                Event::End(TagEnd::Strong),
            ))
            .chain(
                self.details
                    .into_iter()
                    .flat_map(IntoMarkdown::into_markdown),
            )
    }
}

impl IntoMarkdown for Detail {
    fn into_markdown<'a>(&'a self) -> impl Iterator<Item = Event<'a>> {
        let headline = self.headline.into_markdown();
        let headline = wrap_in_tags(Tag::Strong, headline, TagEnd::Strong);
        list(list_item(
            headline
                .chain(wrap_in_tags(
                    Tag::Emphasis,
                    self.snippet.into_markdown(),
                    TagEnd::Emphasis,
                ))
                .chain(nested_header(self.detail.into_markdown())),
        ))
    }
}

impl IntoMarkdown for [Project] {
    fn into_markdown<'a>(&'a self) -> impl Iterator<Item = Event<'a>> {
        wrap_in_tags(
            Tag::Heading {
                level: HeadingLevel::H2,
                id: None,
                classes: Vec::new(),
                attrs: Vec::new(),
            },
            "Open Source".into_markdown(),
            TagEnd::Heading(HeadingLevel::H2),
        )
        .into_iter()
        .chain(list(self.into_iter().flat_map(IntoMarkdown::into_markdown)))
    }
}

impl IntoMarkdown for Project {
    fn into_markdown<'a>(&'a self) -> impl Iterator<Item = Event<'a>> {
        fn project_contents<'a>(
            name: &'a str,
            short_desc: &'a str,
            long_desc: &'a str,
        ) -> impl Iterator<Item = Event<'a>> {
            let header = wrap_in_tags(
                Tag::Paragraph,
                wrap_in_tags(Tag::Strong, text(name), TagEnd::Strong),
                TagEnd::Paragraph,
            );
            let sub_head = wrap_in_tags(
                Tag::Paragraph,
                wrap_in_tags(Tag::Emphasis, text(short_desc), TagEnd::Emphasis),
                TagEnd::Paragraph,
            );
            let desc = long_desc.into_markdown();
            header.chain(sub_head).chain(desc)
        }
        fn full_project<'a>(
            name: &'a str,
            short_desc: &'a str,
            long_desc: &'a str,
            sub_projects: &'a [Project],
        ) -> impl Iterator<Item = Event<'a>> {
            let base = project_contents(name, short_desc, long_desc);
            let subs = sub_projects
                .iter()
                .flat_map(|p| list_item(project_contents(p.name, p.short_desc, p.long_desc)));
            base.chain(list(subs))
        }
        list_item(full_project(
            self.name,
            self.short_desc,
            self.long_desc,
            self.sub_projects,
        ))
    }
}

impl IntoMarkdown for [School] {
    fn into_markdown<'a>(&'a self) -> impl Iterator<Item = Event<'a>> {
        wrap_in_tags(
            Tag::Heading {
                level: HeadingLevel::H2,
                id: None,
                classes: Vec::new(),
                attrs: Vec::new(),
            },
            "Education".into_markdown(),
            TagEnd::Heading(HeadingLevel::H2),
        )
        .chain(self.into_iter().enumerate().flat_map(|(i, s)| {
            (i > 0)
                .then_some(Event::HardBreak)
                .into_iter()
                .chain(s.into_markdown())
        }))
    }
}

impl IntoMarkdown for School {
    fn into_markdown<'a>(&'a self) -> impl Iterator<Item = Event<'a>> {
        [
            Event::Start(Tag::Strong),
            Event::Text(CowStr::Borrowed(self.name)),
            Event::End(TagEnd::Strong),
            Event::HardBreak,
            Event::Text(CowStr::Borrowed(self.desc)),
        ]
        .into_iter()
    }
}

impl IntoMarkdown for str {
    fn into_markdown<'a>(&'a self) -> impl Iterator<Item = Event<'a>> {
        let mut p = Parser::new(self.trim());
        if self.trim().is_empty() {
            while p.next().is_some() {}
        }
        p
    }
}

fn list<'a>(contents: impl Iterator<Item = Event<'a>>) -> impl Iterator<Item = Event<'a>> {
    wrap_in_tags(Tag::List(None), contents, TagEnd::List(false))
}

fn list_item<'a>(contents: impl Iterator<Item = Event<'a>>) -> impl Iterator<Item = Event<'a>> {
    wrap_in_tags(Tag::Item, contents, TagEnd::Item)
}

fn wrap_in_tags<'a>(
    start: Tag<'a>,
    contents: impl Iterator<Item = Event<'a>>,
    end: TagEnd,
) -> impl Iterator<Item = Event<'a>> {
    wrap_in(Event::Start(start), contents, Event::End(end))
}

fn wrap_in<'a>(
    start: Event<'a>,
    contents: impl Iterator<Item = Event<'a>>,
    end: Event<'a>,
) -> impl Iterator<Item = Event<'a>> {
    iter::once(start).chain(contents).chain(iter::once(end))
}

fn text<'a>(content: &'a str) -> impl Iterator<Item = Event<'a>> {
    iter::once(Event::Text(CowStr::Borrowed(content)))
}

fn nested_header<'a>(contents: impl Iterator<Item = Event<'a>>) -> impl Iterator<Item = Event<'a>> {
    fn header_map(level: HeadingLevel) -> HeadingLevel {
        let level2 = match level {
            HeadingLevel::H1 => HeadingLevel::H3,
            HeadingLevel::H2 => HeadingLevel::H4,
            HeadingLevel::H3 => HeadingLevel::H5,
            _ => HeadingLevel::H6,
        };

        println!("setting {level} to {level2}");
        level2
    }
    contents.map(|event| match event {
        Event::Start(Tag::Heading {
            level,
            id,
            classes,
            attrs,
        }) => Event::Start(Tag::Heading {
            level: header_map(level),
            id,
            classes,
            attrs,
        }),
        Event::End(TagEnd::Heading(level)) => Event::End(TagEnd::Heading(header_map(level))),
        ev => ev,
    })
}
