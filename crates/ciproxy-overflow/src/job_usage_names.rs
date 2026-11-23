//! Retain remote action references, not text from script commands.

/// Replace script text while keeping the runner's prefix.
pub(super) fn redact_script_name(name: &mut String) {
    for (prefix, replacement) in [
        ("Run ", "Run script"),
        ("Pre Run ", "Pre Run script"),
        ("Post Run ", "Post Run script"),
    ] {
        if let Some(reference) = name.strip_prefix(prefix) {
            if !remote_action(reference) {
                name.replace_range(.., replacement);
            }
            return;
        }
    }
}

fn remote_action(reference: &str) -> bool {
    if let Some(image) = reference.strip_prefix("docker://") {
        return docker_image(image);
    }
    let Some((path, revision)) = reference.split_once('@') else {
        return false;
    };
    let Some((owner, repository)) = path.split_once('/') else {
        return false;
    };
    owner
        .as_bytes()
        .first()
        .is_some_and(u8::is_ascii_alphanumeric)
        && owner
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        && repository.split('/').all(reference_segment)
        && revision.split('/').all(reference_segment)
}

fn docker_image(image: &str) -> bool {
    let path = if let Some((path, digest)) = image.split_once('@') {
        if !digest.split_once(':').is_some_and(|(algorithm, hash)| {
            reference_segment(algorithm) && reference_segment(hash)
        }) {
            return false;
        }
        path
    } else {
        image
    };
    let tail = if let Some((first, rest)) = path.split_once('/') {
        let valid_first = first.split_once(':').map_or_else(
            || reference_segment(first),
            |(host, port)| {
                reference_segment(host)
                    && !port.is_empty()
                    && port.bytes().all(|byte| byte.is_ascii_digit())
            },
        );
        if !valid_first {
            return false;
        }
        rest
    } else {
        path
    };
    let mut parts = tail.rsplit('/');
    parts.next().is_some_and(|name| {
        name.split_once(':').map_or_else(
            || reference_segment(name),
            |(repository, tag)| reference_segment(repository) && reference_segment(tag),
        )
    }) && parts.all(reference_segment)
}

fn reference_segment(segment: &str) -> bool {
    !matches!(segment, "" | "." | "..")
        && segment
            .chars()
            .all(|symbol| symbol.is_alphanumeric() || matches!(symbol, '.' | '-' | '_' | '+'))
}
