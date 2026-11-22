//! The control lines stay exact, so an agent and an older helper agree.

use super::vm_control::{Answer, ControlError, Request, StopReason, VmPhase};

const REQUESTS: [Request; 5] = [
    Request::State,
    Request::Connect(5252),
    Request::Stop,
    Request::Save,
    Request::Wait,
];

#[test]
fn requests_keep_their_version_one_lines() {
    let lines: Vec<String> = REQUESTS.iter().map(|request| request.line()).collect();
    assert_eq!(
        lines,
        [
            "CIPROXY-VM 1 STATE\n",
            "CIPROXY-VM 1 CONNECT 5252\n",
            "CIPROXY-VM 1 STOP\n",
            "CIPROXY-VM 1 SAVE\n",
            "CIPROXY-VM 1 WAIT\n",
        ]
    );
    for request in REQUESTS {
        assert_eq!(Request::parse(&request.line()), Some(request));
    }
}

#[test]
fn parse_refuses_other_versions_words_and_forms() {
    for line in [
        "CIPROXY-VM 1 STATE",
        "CIPROXY-VM 2 STATE\n",
        "CIPROXY-VM 1 STATE extra\n",
        "CIPROXY-VM 1 CONNECT\n",
        "CIPROXY-VM 1 CONNECT -1\n",
        "CIPROXY-VM 1 CONNECT 5252 1\n",
        "CIPROXY-VM 1 SAVE /etc/passwd\n",
        "CIPROXY-VM 1 state\n",
        "CIPROXY-VM 1  STATE\n",
        "",
    ] {
        assert_eq!(Request::parse(line), None, "{line:?}");
    }
}

#[test]
fn answers_parse_only_in_the_form_of_their_request() {
    let running = Answer::Phase(VmPhase::Running).line();
    let stopped = Answer::Stopped(StopReason::GuestShutdown).line();
    assert_eq!(running, "OK running\n");
    assert_eq!(stopped, "OK guest-shutdown\n");
    assert_eq!(
        Answer::parse(Request::State, &running),
        Some(Answer::Phase(VmPhase::Running))
    );
    assert_eq!(
        Answer::parse(Request::Wait, &stopped),
        Some(Answer::Stopped(StopReason::GuestShutdown))
    );
    assert_eq!(Answer::parse(Request::Stop, "OK\n"), Some(Answer::Done));
    assert_eq!(Answer::parse(Request::Stop, &running), None);
    assert_eq!(Answer::parse(Request::State, "OK\n"), None);
    assert_eq!(Answer::parse(Request::Wait, &running), None);
    assert_eq!(Answer::parse(Request::Save, "OK"), None);
    assert_eq!(Answer::parse(Request::Save, "ERR secret\n"), None);
}

#[test]
fn only_a_guest_that_does_not_listen_is_retryable() {
    let line = Answer::Refused(ControlError::GuestNotListening).line();
    assert_eq!(line, "ERR guest-not-listening\n");
    assert_eq!(
        Answer::parse(Request::Connect(5252), &line),
        Some(Answer::Refused(ControlError::GuestNotListening))
    );
    let retryable: Vec<bool> = [
        ControlError::GuestNotListening,
        ControlError::Stopped,
        ControlError::Unsupported,
        ControlError::Failed,
    ]
    .into_iter()
    .map(ControlError::retryable)
    .collect();
    assert_eq!(retryable, [true, false, false, false]);
}
