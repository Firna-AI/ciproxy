//! Markdown link destinations and GitHub-style heading anchors.

use std::collections::BTreeSet;

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use regex_syntax::is_word_character;

pub(super) fn link_destinations(source: &str) -> Vec<(String, usize)> {
    Parser::new_ext(source, parser_options())
        .into_offset_iter()
        .filter_map(|(event, range)| match event {
            Event::Start(Tag::Link { dest_url, .. }) => Some((
                dest_url.into_string(),
                source[..range.start]
                    .bytes()
                    .filter(|byte| *byte == b'\n')
                    .count()
                    + 1,
            )),
            _ => None,
        })
        .collect()
}

pub(super) fn heading_anchors(source: &str) -> BTreeSet<String> {
    let mut anchors = BTreeSet::new();
    let mut heading = None::<(String, Option<String>)>;
    for event in Parser::new_ext(source, parser_options()) {
        match event {
            Event::Start(Tag::Heading { id, .. }) => {
                heading = Some((String::new(), id.map(|value| value.into_string())));
            }
            Event::Text(text) | Event::Code(text) => {
                if let Some((heading_text, _id)) = &mut heading {
                    heading_text.push_str(&text);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some((heading_text, _id)) = &mut heading {
                    heading_text.push('\n');
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some((heading_text, explicit_id)) = heading.take() {
                    insert_anchor(
                        &mut anchors,
                        explicit_id.unwrap_or_else(|| github_heading_slug(&heading_text)),
                    );
                }
            }
            _ => {}
        }
    }
    anchors
}

fn parser_options() -> Options {
    let mut options = Options::all();
    options.remove(Options::ENABLE_WIKILINKS);
    options
}

fn insert_anchor(anchors: &mut BTreeSet<String>, base: String) {
    let mut candidate = base.clone();
    let mut counter = 0;
    while anchors.contains(&candidate) {
        counter += 1;
        candidate = format!("{base}-{counter}");
    }
    anchors.insert(candidate);
}

fn github_heading_slug(heading: &str) -> String {
    heading
        .to_lowercase()
        .chars()
        .filter_map(|character| {
            if character == ' ' {
                Some('-')
            } else if character == '-' || is_word_character(character) {
                Some(character)
            } else {
                None
            }
        })
        .collect()
}
