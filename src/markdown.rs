use ratatui::{style::{Color, Modifier, Style}, text::{Line, Span, Text}};
use pulldown_cmark::{Event, Parser, Tag};


pub fn convert_md(s: &'static str, width: usize) -> Text<'static> {
    convert(s, width).unwrap_or_else(|| {
        eprintln!("Plain text!");
        Text::raw(s)
    })
}

fn default_style() -> Style {
    Style::new().fg(Color::Green).bg(Color::Black)
}

fn convert(s: &'static str, width: usize) -> Option<Text<'static>> {    
    let parser = Parser::new(s);
    let mut lines = Vec::new();
    let mut style = default_style();
    let mut spans = Vec::new();
    let mut list_idx = None;
    let mut line_length = 0;
    for event in parser {
        match event {
            Event::Start(tag) => {
                match tag {
                    Tag::Paragraph => {},
                    Tag::Heading { .. } => {
                        style = default_style().add_modifier(Modifier::BOLD);
                    },
                    Tag::BlockQuote => {
                        spans.push(Span::raw("| ").style(default_style()));
                    },
                    Tag::CodeBlock(_) => {
                        
                    },
                    Tag::HtmlBlock => {
                        return None
                    },
                    Tag::List(idx) => {
                        list_idx = idx;
                    },
                    Tag::Item => {
                        let span = if let Some(i) = list_idx {
                            let ret = Span::raw(i.to_string()).style(default_style());
                            list_idx = Some(i+1);
                            ret
                        } else {
                            Span::raw("- ").style(default_style())
                        };
                        line_length += span.width();
                        spans.push(span);
                    },
                    Tag::FootnoteDefinition(_)
                     | Tag::Table(_)
                     | Tag::TableHead
                     | Tag::TableRow
                     | Tag::TableCell => {
                        return None;
                    }
                    Tag::Emphasis => style = style.add_modifier(Modifier::ITALIC),
                    Tag::Strong => style = style.add_modifier(Modifier::BOLD),
                    Tag::Strikethrough => todo!(),
                    Tag::Link {dest_url, title: _title, .. } => {
                        let mut span = Span::from(dest_url.to_string()).style(style);
                        
                        while line_length + span.width() > width {
                            let idx = width - line_length + span.width();
                            let (left, right) = split_span(span, idx);
                            span = right;
                            spans.push(left);
                            lines.push(Line::from(spans));
                            line_length = 0;
                            spans = Vec::new();    
                        }
                        line_length += span.width();
                        spans.push(span);
                    },
                    Tag::Image { .. } => return None,
                    Tag::MetadataBlock(_) => return None,
                }
            },
            Event::End(_tag) => {
                style = default_style();
            },
            Event::Text(content)
            | Event::Code(content) 
            | Event::Html(content)
            | Event::InlineHtml(content)
            | Event::FootnoteReference(content) => {
                let mut span = Span::from(content.to_string()).style(style);
                while line_length + span.width() > width {
                    let idx: usize = width - line_length + span.width();
                    let (left, right) = split_span(span, idx);
                    span = right;
                    spans.push(left);
                    lines.push(Line::from(spans));
                    line_length = 0;
                    spans = Vec::new();    
                }
                line_length += span.width();
                spans.push(span);
            },
            Event::SoftBreak => {
                spans.push(" ".into());
            },
            Event::HardBreak => {
                lines.push(Line::from(spans));
                spans = Vec::new();
                style = default_style();
            },
            Event::Rule => {
                eprintln!("RULE!!!");
                return None
            },
            Event::TaskListMarker(_) => {
                eprintln!("LIST_MARKER");
                return None
            },
        }
    }
    if !spans.is_empty() {
        lines.push(Line::from(spans));
    }
    Some(Text::from(lines).left_aligned())
}

fn wrap_span(first_split: usize, max_width: usize, span: Span<'static>) -> WrappedSpans {  
    let (pre, mut post) = split_span(span, first_split);
    let mut lines = Vec::new();
    while post.width() >= max_width {
        let (pre, new_post) = split_span(post, max_width);
        lines.push(Line::from(pre));
        post = new_post;
    }
    WrappedSpans {
        pre,
        lines,
        post,
    }
}

struct WrappedSpans {
    pre: Span<'static>,
    lines: Vec<Line<'static>>,
    post: Span<'static>
}

fn split_span(span: Span<'static>, idx: usize) -> (Span<'static>, Span<'static>) {
    let start = &span.content[..idx];
    let last_space = start.rfind(' ').unwrap_or(idx);
    let start = span.content[..last_space].to_string();
    let end = span.content[last_space..].to_string();
    (Span::from(start).style(span.style), Span::from(end).style(span.style))
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::source::DATABASE;

    #[test]
    fn convert_some_details() {
        let oss = DATABASE.open_source.first().unwrap();
        let out = convert_md(oss.long_desc, 50);
        panic!("{}", out.to_string());
    }
}
