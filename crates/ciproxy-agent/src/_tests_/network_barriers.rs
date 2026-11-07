//! A held host deletion proves progress without real timers or host commands.

use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use tokio::sync::{self, oneshot};

use crate::error::Result;
use crate::process::{Commands, UnitState};

pub(crate) struct HeldDeletion {
    pub(crate) delegate: Arc<dyn Commands>,
    pub(crate) inspection_started: sync::Mutex<Option<oneshot::Sender<()>>>,
    pub(crate) inspection_release: sync::Mutex<Option<oneshot::Receiver<()>>>,
    pub(crate) started: sync::Mutex<Option<oneshot::Sender<()>>>,
    pub(crate) release: sync::Mutex<Option<oneshot::Receiver<()>>>,
}

#[async_trait]
impl Commands for HeldDeletion {
    async fn namespaces(&self) -> Result<Vec<String>> {
        let release = self.inspection_release.lock().await.take();
        if let Some(release) = release {
            self.inspection_started
                .lock()
                .await
                .take()
                .unwrap()
                .send(())
                .unwrap();
            release.await.unwrap();
        }
        self.delegate.namespaces().await
    }
    async fn unit(&self, name: &str) -> Result<UnitState> {
        self.delegate.unit(name).await
    }
    async fn check_filesystem(&self, path: &str) -> Result<()> {
        self.delegate.check_filesystem(path).await
    }
    async fn run(&self, program: &str, args: &[String], input: &[u8]) -> Result<Vec<u8>> {
        if args.iter().any(|arg| arg == "delete") {
            let release = self.release.lock().await.take();
            if let Some(release) = release {
                self.started.lock().await.take().unwrap().send(()).unwrap();
                release.await.unwrap();
            }
        }
        self.delegate.run(program, args, input).await
    }
}

#[derive(Clone, Default)]
pub(crate) struct Capture(pub(crate) Arc<Mutex<Vec<u8>>>);

impl Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
