//! The text protocol of a VM helper's control socket. One request per connection.
//!
//! Each request is one line: the version, a word, and an optional argument.
//! Each answer is one line: `OK`, `OK WORD`, or `ERR CODE`. A `CONNECT`
//! answer carries the vsock stream as one `SCM_RIGHTS` fd. Only enums build
//! the words, so no request or answer can carry free text or a secret.

/// The version prefix of every request. A new agent must still speak to an
/// older VM helper after an update, so change this only with a new version.
pub(crate) const VERSION: &str = "CIPROXY-VM 1";

/// The longest request or answer line, newline included.
pub(crate) const LINE_LIMIT: usize = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Request {
    /// Read the VM phase.
    State,
    /// Connect to a guest vsock port and pass the stream fd.
    Connect(u32),
    /// Ask the guest to shut down.
    Stop,
    /// Pause the VM and save its state into the VM folder.
    Save,
    /// Answer once the VM stops.
    Wait,
}

impl Request {
    pub(crate) fn line(self) -> String {
        match self {
            Self::State => format!("{VERSION} STATE\n"),
            Self::Connect(port) => format!("{VERSION} CONNECT {port}\n"),
            Self::Stop => format!("{VERSION} STOP\n"),
            Self::Save => format!("{VERSION} SAVE\n"),
            Self::Wait => format!("{VERSION} WAIT\n"),
        }
    }

    pub(crate) fn parse(line: &str) -> Option<Self> {
        let rest = line
            .strip_suffix('\n')?
            .strip_prefix(VERSION)?
            .strip_prefix(' ')?;
        let mut words = rest.split(' ');
        let request = match (words.next()?, words.next()) {
            ("STATE", None) => Self::State,
            ("CONNECT", Some(port)) => Self::Connect(port.parse().ok()?),
            ("STOP", None) => Self::Stop,
            ("SAVE", None) => Self::Save,
            ("WAIT", None) => Self::Wait,
            _ => return None,
        };
        words.next().is_none().then_some(request)
    }
}

macro_rules! words {
    ($(#[$meta:meta])* $name:ident { $($variant:ident = $word:literal,)+ }) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name { $($variant,)+ }

        impl $name {
            pub(crate) fn word(self) -> &'static str {
                match self { $(Self::$variant => $word,)+ }
            }

            pub(crate) fn from_word(word: &str) -> Option<Self> {
                match word { $($word => Some(Self::$variant),)+ _ => None }
            }
        }
    };
}

words!(
    /// The phase of a helper's VM.
    VmPhase {
        Starting = "starting",
        Running = "running",
        Paused = "paused",
        Stopping = "stopping",
        Stopped = "stopped",
    }
);

words!(
    /// Why a helper's VM stopped.
    StopReason {
        GuestShutdown = "guest-shutdown",
        HostStop = "host-stop",
        Failure = "failure",
    }
);

words!(
    /// Why a helper refused a request.
    ControlError {
        GuestNotListening = "guest-not-listening",
        Stopped = "stopped",
        Unsupported = "unsupported",
        Failed = "failed",
    }
);

impl ControlError {
    /// A refused vsock connect means that the guest does not listen yet.
    pub(crate) fn retryable(self) -> bool {
        self == Self::GuestNotListening
    }
}

/// One answer line, before the client reads its words.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Answer {
    Done,
    Phase(VmPhase),
    Stopped(StopReason),
    Refused(ControlError),
}

impl Answer {
    pub(crate) fn line(self) -> String {
        match self {
            Self::Done => "OK\n".to_owned(),
            Self::Phase(phase) => format!("OK {}\n", phase.word()),
            Self::Stopped(reason) => format!("OK {}\n", reason.word()),
            Self::Refused(error) => format!("ERR {}\n", error.word()),
        }
    }

    /// Parse the answer to `request`. Each request has its own answer form.
    pub(crate) fn parse(request: Request, line: &str) -> Option<Self> {
        let line = line.strip_suffix('\n')?;
        if let Some(code) = line.strip_prefix("ERR ") {
            return ControlError::from_word(code).map(Self::Refused);
        }
        match (request, line.strip_prefix("OK")?) {
            (Request::Connect(_) | Request::Stop | Request::Save, "") => Some(Self::Done),
            (Request::State, word) => VmPhase::from_word(word.strip_prefix(' ')?).map(Self::Phase),
            (Request::Wait, word) => {
                StopReason::from_word(word.strip_prefix(' ')?).map(Self::Stopped)
            }
            _ => None,
        }
    }
}
