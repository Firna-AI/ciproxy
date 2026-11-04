//! Local Markdown destination classification and path resolution.

use std::path::{Component, Path, PathBuf};

/// A local destination with its fragment and GitHub plain-view mode preserved.
pub(super) struct DestinationParts<'a> {
    pub(super) path: &'a str,
    pub(super) fragment: Option<&'a str>,
    plain: bool,
}

impl DestinationParts<'_> {
    /// Plain-view line numbers are not Markdown heading IDs.
    pub(super) fn is_line_link(&self) -> bool {
        self.plain
            && self
                .fragment
                .is_some_and(|fragment| is_line_fragment(&percent_decode(fragment)))
    }
}

fn is_line_fragment(fragment: &str) -> bool {
    let is_number =
        |value: &str| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit());
    let Some(fragment) = fragment.strip_prefix('L') else {
        return false;
    };
    match fragment.split_once("-L") {
        Some((start, end)) => [start, end].into_iter().all(|position| {
            let (line, column) = position
                .split_once('C')
                .map_or((position, None), |(line, column)| (line, Some(column)));
            is_number(line) && column.is_none_or(is_number)
        }),
        None => is_number(fragment),
    }
}

pub(super) fn ignored_destination(destination: &str) -> bool {
    destination.starts_with('/') || has_url_scheme(destination)
}

pub(super) fn destination_parts(destination: &str) -> DestinationParts<'_> {
    let (path_and_query, fragment) = destination
        .split_once('#')
        .map_or((destination, None), |(path, fragment)| {
            (path, Some(fragment))
        });
    let (path, query) = path_and_query
        .split_once('?')
        .map_or((path_and_query, None), |(path, query)| (path, Some(query)));
    DestinationParts {
        path,
        fragment,
        plain: query.is_some_and(|query| query.split('&').any(|part| part == "plain=1")),
    }
}

pub(super) fn resolve_target_path(source_path: &str, destination_path: &str) -> Option<String> {
    if destination_path.is_empty() {
        return Some(source_path.to_owned());
    }
    let mut resolved = PathBuf::from(source_path);
    resolved.pop();
    for component in Path::new(destination_path).components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => resolved.push(part),
            Component::ParentDir => {
                if !resolved.pop() {
                    return None;
                }
            }
            Component::Prefix(_) | Component::RootDir => return None,
        }
    }
    Some(
        resolved
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/"),
    )
}

pub(super) fn percent_decode(value: &str) -> String {
    let mut decoded = Vec::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%'
            && let Some(pair) = bytes.get(index + 1..index + 3)
            && let Ok(hex) = std::str::from_utf8(pair)
            && let Ok(byte) = u8::from_str_radix(hex, 16)
        {
            decoded.push(byte);
            index += 3;
            continue;
        }
        decoded.push(bytes[index]);
        index += 1;
    }
    match String::from_utf8(decoded) {
        Ok(decoded) => decoded,
        Err(_) => value.to_owned(),
    }
}

fn has_url_scheme(destination: &str) -> bool {
    let Some(colon) = destination.find(':') else {
        return false;
    };
    let scheme = &destination[..colon];
    !scheme.is_empty()
        && scheme.chars().enumerate().all(|(index, character)| {
            character.is_ascii_alphabetic()
                || (index > 0
                    && (character.is_ascii_digit() || matches!(character, '+' | '-' | '.')))
        })
}
