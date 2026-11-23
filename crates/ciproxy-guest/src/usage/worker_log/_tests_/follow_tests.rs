//! Page selection and following at report boundaries.

use std::io::Cursor;

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

fn ran(name: &str, start: i64, end: Option<i64>) -> JobStep {
    JobStep {
        name: name.into(),
        timing: JobStepTiming::Ran {
            started_at_ms: Some(start),
            ended_at_ms: end,
        },
    }
}
