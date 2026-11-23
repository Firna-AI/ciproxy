//! Page selection and following at report boundaries.

use std::io::Cursor;
use std::sync::Arc;

use ciproxy_protocol::usage::{JobStep, JobStepTiming};
use unimock::{MockFn, Unimock, matching};

use super::{LogPagesMock, WorkerLog};

const OLD: &str = "Worker_20231114-210000-utc.log";
const FIRST: &str = "Worker_20231114-221318-utc.log";
const SECOND: &str = "Worker_20231114-221322-utc.log";
const FIRST_LINES: &str = concat!(
    "[2023-11-14 22:13:18Z INFO StepsRunner] Processing step: DisplayName='Earlier step'\n",
    "[2023-11-14 22:13:18Z INFO StepsRunner] Skipping step due to condition evaluation.\n",
    "[2023-11-14 22:13:19Z INFO StepsRunner] Processing step: DisplayName='Crosses baseline'\n",
    "[2023-11-14 22:13:19Z INFO StepsRunner] Starting the step.\n",
    "[2023-11-14 22:13:20Z INFO StepsRunner] Step result: Succeeded\n",
    "[2023-11-14 22:13:20Z INFO StepsRunner] Processing step: DisplayName='Active step'\n",
    "[2023-11-14 22:13:20Z INFO StepsRunner] Starting the step.\n",
);
const LAST_LINE: &str = "[2023-11-14 22:13:21Z INFO StepsRunner] Step result: \n";
const NEXT_PAGE: &str = concat!(
    "[2023-11-14 22:13:22Z INFO StepsRunner] Processing step: DisplayName='Post action'\n",
    "[2023-11-14 22:13:22Z INFO StepsRunner] Starting the step.\n",
    "[2023-11-14 22:13:23Z INFO StepsRunner] Step result: Succeeded\n",
);
const SCRIPT_NAMES: &str = concat!(
    "[2023-11-14 22:13:19Z INFO StepsRunner] Processing step: DisplayName='Pre Run foo/bar@baz'\n",
    "[2023-11-14 22:13:20Z INFO StepsRunner] Processing step: DisplayName='Run foo/bar@baz'\n",
    "[2023-11-14 22:13:21Z INFO StepsRunner] Processing step: DisplayName='Post Run foo/bar@baz'\n",
);
const JOB_HEADER: &str = "[2023-11-14 22:13:18Z INFO Worker] Job message:\n";

#[test]
fn follows_new_pages_in_order_and_waits_for_a_complete_line() {
    let mut log = WorkerLog::new(Box::new(Unimock::new((
        LogPagesMock::list
            .next_call(matching!())
            .returns(Ok(vec![OLD.into()])),
        LogPagesMock::list
            .next_call(matching!())
            .returns(Ok(vec![FIRST.into(), OLD.into()])),
        LogPagesMock::open
            .next_call(matching!((name, 0) if *name == FIRST))
            .answers(&|_, _, _| {
                Ok(Box::new(Cursor::new(format!(
                    "{FIRST_LINES}{}",
                    &LAST_LINE[..25]
                ))))
            }),
        LogPagesMock::list.next_call(matching!()).returns(Ok(vec![
            SECOND.into(),
            FIRST.into(),
            OLD.into(),
        ])),
        LogPagesMock::open
            .next_call(
                matching!((name, offset) if *name == FIRST && *offset == FIRST_LINES.len() as u64),
            )
            .answers(&|_, _, _| Ok(Box::new(Cursor::new(LAST_LINE)))),
        LogPagesMock::open
            .next_call(matching!((name, 0) if *name == SECOND))
            .answers(&|_, _, _| Ok(Box::new(Cursor::new(NEXT_PAGE)))),
    ))));
    log.exclude_existing_pages().unwrap();

    let earlier = JobStep {
        name: "Earlier step".into(),
        timing: JobStepTiming::Skipped,
    };
    let crossing = ran(
        "Crosses baseline",
        1_699_999_999_000,
        Some(1_700_000_000_000),
    );
    assert_eq!(
        log.report().unwrap(),
        [
            earlier.clone(),
            crossing.clone(),
            ran("Active step", 1_700_000_000_000, None)
        ]
    );
    assert_eq!(
        log.report().unwrap(),
        [
            earlier,
            crossing,
            ran("Active step", 1_700_000_000_000, Some(1_700_000_001_000)),
            ran("Post action", 1_700_000_002_000, Some(1_700_000_003_000)),
        ]
    );
}

#[test]
fn redacts_an_action_shaped_command_without_the_action() {
    assert_eq!(
        names(replay(include_str!(
            "fixtures/worker-action-shaped-script.txt"
        ))),
        ["Run script"]
    );
}

#[test]
fn keeps_the_names_of_remote_actions_in_the_job() {
    assert_eq!(
        names(replay(include_str!("fixtures/worker-remote-actions.txt"))),
        [
            "Pre Run foo/bar@baz",
            "Run foo/bar@baz",
            "Run other/action/nested@release/stable",
            "Run alpine:3.20",
            "Run script",
            "Post Run foo/bar@baz",
        ]
    );
}

#[test]
fn redacts_script_names_without_a_job_message() {
    assert_eq!(
        names(replay(SCRIPT_NAMES)),
        ["Pre Run script", "Run script", "Post Run script"]
    );
}

#[test]
fn redacts_script_names_with_a_malformed_job_message() {
    let contents = format!(
        "{JOB_HEADER} {{\n  \"steps\": [{{\"reference\": {{\"type\": \"repository\", \"name\": \"foo/bar\", \"ref\": \"baz\"}}}}],\n}}\n{SCRIPT_NAMES}"
    );
    assert_eq!(
        names(replay(&contents)),
        ["Pre Run script", "Run script", "Post Run script"]
    );
}

#[test]
fn redacts_script_names_with_an_oversized_job_message() {
    let contents = format!(
        "{JOB_HEADER} {{\n  \"steps\": [{{\"reference\": {{\"type\": \"repository\", \"name\": \"foo/bar\", \"ref\": \"baz\"}}}}],\n  \"padding\": \"{}\"\n}}\n{SCRIPT_NAMES}",
        "x".repeat(1024 * 1024)
    );
    assert_eq!(
        names(replay(&contents)),
        ["Pre Run script", "Run script", "Post Run script"]
    );
}

fn names(steps: Vec<JobStep>) -> Vec<String> {
    steps.into_iter().map(|step| step.name).collect()
}

pub(super) fn replay(contents: &str) -> Vec<JobStep> {
    let bytes = contents.as_bytes().to_vec();
    let pages = Unimock::new((
        LogPagesMock::list
            .some_call(matching!())
            .returns(Ok(vec![FIRST.into()])),
        LogPagesMock::open
            .some_call(matching!((name, 0) if *name == FIRST))
            .answers_arc(Arc::new(move |_, _, _| {
                Ok(Box::new(Cursor::new(bytes.clone())))
            })),
    ));
    WorkerLog::new(Box::new(pages)).report().unwrap()
}

fn ran(name: &str, start: i64, end: Option<i64>) -> JobStep {
    JobStep {
        name: name.into(),
        timing: JobStepTiming::Ran {
            started_at_ms: Some(start),
            ended_at_ms: end,
        },
    }
}
