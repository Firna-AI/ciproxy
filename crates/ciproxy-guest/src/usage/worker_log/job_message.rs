//! Extract action references from the pinned runner's Worker job message.

use std::collections::HashSet;
use std::io::{self, BufRead, Read};

use serde::Deserialize;

const MAX_JOB_MESSAGE_BYTES: usize = 1024 * 1024;

#[derive(Deserialize)]
struct JobMessage {
    steps: Vec<Step>,
}

#[derive(Deserialize)]
struct Step {
    reference: ActionReference,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum ActionReference {
    Repository {
        name: Option<String>,
        path: Option<String>,
        #[serde(rename = "ref")]
        revision: Option<String>,
    },
    ContainerRegistry {
        image: String,
    },
    Script,
}

impl ActionReference {
    fn name(self) -> Option<String> {
        match self {
            Self::Repository {
                name: Some(mut name),
                path,
                revision: Some(revision),
            } if !name.is_empty() && !revision.is_empty() => {
                if let Some(path) = path.filter(|path| !path.is_empty()) {
                    name.push('/');
                    name.push_str(&path);
                }
                name.push('@');
                name.push_str(&revision);
                Some(name)
            }
            Self::ContainerRegistry { image } if !image.is_empty() => Some(image),
            _ => None,
        }
    }
}

/// The job's remote action names. No other part of the message is retained.
#[derive(Default)]
pub(super) struct JobActions(HashSet<String>);

impl JobActions {
    /// Match the Info record that precedes the job message body.
    pub(super) fn is_header(line: &str) -> bool {
        line.strip_prefix('[')
            .and_then(|line| line.split_once("] "))
            .is_some_and(|(header, message)| {
                header.ends_with(" INFO Worker") && message == "Job message:"
            })
    }

    /// Read the indented JSON body. Return `None` while the record is incomplete.
    pub(super) fn read(reader: &mut dyn BufRead) -> io::Result<Option<Self>> {
        let mut body = Vec::new();
        loop {
            match reader.fill_buf()?.first() {
                None => return Ok(None),
                Some(b'[') => return Ok(Some(Self::default())),
                _ => {}
            }
            let line_start = body.len();
            (&mut *reader)
                .take((MAX_JOB_MESSAGE_BYTES + 1 - line_start) as u64)
                .read_until(b'\n', &mut body)?;
            if body.len() > MAX_JOB_MESSAGE_BYTES {
                if !body.ends_with(b"\n") {
                    reader.skip_until(b'\n')?;
                }
                return skip_to_trace(reader);
            }
            let line = &body[line_start..];
            if !line.ends_with(b"\n") {
                return Ok(None);
            }
            if matches!(line, b"}\n" | b"}\r\n") {
                return Ok(Some(Self::parse(&body)));
            }
        }
    }

    fn parse(body: &[u8]) -> Self {
        let Ok(message) = serde_json::from_slice::<JobMessage>(body) else {
            return Self::default();
        };
        Self(
            message
                .steps
                .into_iter()
                .filter_map(|step| step.reference.name())
                .collect(),
        )
    }

    /// Keep prefixed names only when the job names the same action.
    pub(super) fn filter_name<'name>(actions: Option<&Self>, name: &'name str) -> &'name str {
        for (prefix, replacement) in [
            ("Run ", "Run script"),
            ("Pre Run ", "Pre Run script"),
            ("Post Run ", "Post Run script"),
        ] {
            if let Some(reference) = name.strip_prefix(prefix)
                && !actions.is_some_and(|actions| actions.0.contains(reference))
            {
                return replacement;
            }
        }
        name
    }
}

fn skip_to_trace(reader: &mut dyn BufRead) -> io::Result<Option<JobActions>> {
    loop {
        match reader.fill_buf()?.first() {
            None => return Ok(None),
            Some(b'[') => return Ok(Some(JobActions::default())),
            _ => {
                reader.skip_until(b'\n')?;
            }
        }
    }
}
