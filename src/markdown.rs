use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span, Text},
};

pub fn convert_md(s: &'static str) -> Text<'static> {
    convert(s).unwrap_or_else(|| {
        log::debug!("Plain text!");
        Text::raw(s)
    })
}

const fn default_style() -> Style {
    crate::DEFAULT_STYLE
}

fn convert(s: &'static str) -> Option<Text<'static>> {
    let parser = Parser::new(s);
    let mut wrapper = Wrapper::new();
    for event in parser {
        log::trace!("Event: {event:#?}");
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {}
                Tag::Heading { .. } => {
                    wrapper.clear_style();
                    wrapper.modify_style(Modifier::BOLD)
                }
                Tag::BlockQuote(_) => {
                    wrapper.line_prefix = Some(Span::raw("| ").style(default_style()));
                    wrapper.push_symbol("| ");
                }
                Tag::CodeBlock(kind) => {
                    wrapper.push_text_with_style("```", default_style());
                    if let CodeBlockKind::Fenced(name) = kind {
                        wrapper.push_text_with_style(name, default_style())
                    }
                    wrapper.new_line();
                }
                Tag::HtmlBlock => {}
                Tag::List(idx) => {
                    wrapper.list_number = idx;
                }
                Tag::Item => {
                    let ch = if let Some(i) = wrapper.list_number {
                        wrapper.list_number = Some(i + 1);
                        i.to_string()
                    } else {
                        '-'.to_string()
                    };
                    wrapper.push_symbol(ch);
                }
                Tag::FootnoteDefinition(_)
                | Tag::Table(_)
                | Tag::TableHead
                | Tag::TableRow
                | Tag::TableCell => {}
                Tag::Emphasis => wrapper.modify_style(Modifier::ITALIC),
                Tag::Strong => wrapper.modify_style(Modifier::BOLD),
                Tag::Strikethrough => {
                    wrapper.modify_style(Modifier::CROSSED_OUT);
                }
                Tag::Link { dest_url, .. } => {
                    wrapper.link_url = Some(dest_url.to_string());
                    wrapper.push_symbol('[');
                }
                Tag::Image { .. } => return None,
                Tag::MetadataBlock(_) => return None,
                Tag::DefinitionList => return None,
                Tag::DefinitionListTitle => return None,
                Tag::DefinitionListDefinition => return None,
            },
            Event::End(tag) => {
                match tag {
                    TagEnd::Paragraph => {
                        wrapper.new_line();
                        wrapper.new_line();
                    }
                    TagEnd::Heading(_)
                    | TagEnd::HtmlBlock
                    | TagEnd::List(_)
                    | TagEnd::Item
                    | TagEnd::FootnoteDefinition => {
                        wrapper.new_line();
                    }
                    TagEnd::Link => {
                        wrapper.push_symbol(']');
                        if let Some(url) = wrapper.link_url.take() {
                            wrapper.push_symbol('(');
                            wrapper.push_text(url);
                            wrapper.push_symbol(')');
                        }
                    }
                    TagEnd::CodeBlock => {
                        wrapper.push_text_with_style("```", default_style());
                        wrapper.new_line();
                    }
                    TagEnd::BlockQuote(_) => {
                        // If there is only 1 span in the line, it is probably the
                        // prefix, we don't want to render that but just to be defensive
                        // we check to see if the prefix exists and if the last element
                        // in the line is the prefix, if not then we render whatever
                        // is in the line.
                        wrapper.clear_prefix();
                        wrapper.new_line();
                    }
                    _ => {}
                }
                wrapper.clear_style();
            }
            Event::Code(content) => {
                wrapper.push_symbol('`');
                wrapper.push_text_with_style(content, default_style());
                wrapper.push_symbol('`');
            }
            Event::Text(content)
            | Event::Html(content)
            | Event::InlineHtml(content)
            | Event::FootnoteReference(content) => {
                wrapper.push_text(content);
            }
            Event::SoftBreak => {
                wrapper.push_symbol(' ');
            }
            Event::HardBreak => {
                wrapper.new_line();
            }
            Event::Rule => {
                wrapper.push_text_with_style("-".repeat(5), default_style());
                wrapper.new_line();
                wrapper.new_line();
            }
            Event::TaskListMarker(complete) => {
                let text = if complete { "- [x] " } else { "- [ ]" };
                wrapper.push_text_with_style(text, default_style());
            }
            Event::DisplayMath(_) | Event::InlineMath(_) => {}
        }
    }
    Some(wrapper.finish())
}

#[derive(Debug, Default)]
struct Wrapper {
    line: Vec<Span<'static>>,
    lines: Vec<Line<'static>>,
    style: Style,
    line_prefix: Option<Span<'static>>,
    list_number: Option<u64>,
    link_url: Option<String>,
}

impl Wrapper {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Push a character into the current line, this assumes the provided value
    /// shouldn't need to worry about line wrapping internally and is primarily
    /// for dealing with lists and other similar text
    fn push_symbol(&mut self, ch: impl ToString) {
        self.line.push(ch.to_string().into());
    }

    /// Push text into the current line, wrapping as needed
    fn push_text(&mut self, content: impl ToString) {
        self.push_text_with_style(content, self.style);
    }

    /// Push text into the current line, wrapping as needed
    fn push_text_with_style(&mut self, content: impl ToString, style: Style) {
        let content = content.to_string();
        let span = Span::from(content).style(style);
        self.line.push(span);
    }

    fn new_line(&mut self) {
        let prev_lines = core::mem::take(&mut self.line);
        self.lines.push(Line::from(prev_lines));
        if let Some(prefix) = self.line_prefix.clone() {
            // self.current_width += prefix.width();
            self.line.push(prefix);
        }
    }

    fn clear_style(&mut self) {
        self.set_style(default_style());
    }

    fn set_style(&mut self, style: Style) {
        self.style = style;
    }

    fn modify_style(&mut self, modif: Modifier) {
        self.style = self.style.add_modifier(modif);
    }

    fn finish(mut self) -> Text<'static> {
        if !self.line.is_empty() {
            self.new_line()
        }

        Text::from(self.lines).left_aligned()
    }

    /// Clear the line prefix and any potentially empty lines with that prefix
    fn clear_prefix(&mut self) {
        let Some(prefix) = self.line_prefix.take() else {
            log::warn!("Taking prefix w/o a prefix!");
            return;
        };
        if let Some(last_span) = self.line.pop() {
            if last_span != prefix {
                self.line.push(last_span);
            }
        }
        while let Some(last_line) = self.lines.pop() {
            if last_line.spans.len() == 1 {
                if !last_line
                    .spans
                    .last()
                    .map(|s| s == &prefix)
                    .unwrap_or(false)
                {
                    self.lines.push(last_line);
                    break;
                }
            } else {
                self.lines.push(last_line);
                break;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use ratatui::{
        backend::{Backend, TestBackend},
        Terminal,
    };

    use super::*;

    macro_rules! assert_rendered {
        ($text:expr) => {
            let backend = TestBackend::new(45, $text.height() as _);
            let mut term = Terminal::new(backend).unwrap();
            term.draw(|f| f.render_widget($text, f.area())).unwrap();
            term.backend_mut().flush().unwrap();
            insta::assert_snapshot!(term.backend());
        };
    }

    #[test]
    fn convert_markdown() {
        env_logger::builder().is_test(true).try_init().ok();
        let md = r#"# this is a heading and should be bold
        
this is a line after a _hard_ break
and a __soft__ break

this line should be split around here but the text will continue on

- this is a list item
- this is another list item

[link!](https://example.com)



***

```rust
fn main() {
}
```

> This is an important block quote
> don't forget that
> these also auto-wrap
        "#;
        assert_rendered!(convert_md(md));
    }

    #[test]
    fn convert_markdown_one_line() {
        env_logger::builder().is_test(true).try_init().ok();
        let md = r#"just text"#;
        assert_rendered!(convert_md(md));
    }

    #[test]
    fn convert_markdown_breaks() {
        env_logger::builder().is_test(true).try_init().ok();
        let md = r#"one
soft


hard



double hard"#;
        assert_rendered!(convert_md(md));
    }

    #[test]
    fn convert_markdown_bock_quote() {
        env_logger::builder().is_test(true).try_init().ok();
        let md = r#"> super important
> block quote"#;
        assert_rendered!(convert_md(md));
    }
}
