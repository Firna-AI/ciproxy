//! Real e2fsck and files exercise corruption recovery without KVM or root.

use std::fs::{self, File};
use std::sync::Arc;

use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{Cache, CacheMock};
use crate::docker::{DockerDisks, LinuxDockers};
use crate::docker_files;
use crate::error::Error;
use crate::fixtures::{TemporaryLayout, active_lease, facts, scope, vm};
use crate::process::LinuxCommands;

#[tokio::test]
async fn offline_corruption_discards_idle_sources_and_preserves_a_live_writer() {
    for live_writer in [false, true] {
        let temporary = TemporaryLayout::new();
        let repository = temporary.repository().await;
        let directory = temporary.layout.data.join("docker").join(&repository);
        fs::create_dir_all(&directory).unwrap();
        let base = directory.join("base.ext4");
        File::create(&base)
            .unwrap()
            .set_len(16 * 1024 * 1024)
            .unwrap();
        let record = vm("corrupt-source");
        fs::create_dir_all(temporary.layout.jail_root(&record)).unwrap();
        if live_writer {
            let mut writer_lease = active_lease(repository.clone(), true);
            writer_lease.bytes = 16_777_216;
            docker_files::save(&temporary.layout, &vm("live-writer"), &writer_lease).unwrap();
            fs::copy(
                &base,
                temporary.layout.jail_root(&record).join("docker-job.ext4"),
            )
            .unwrap();
            let mut copy_lease = active_lease(repository, false);
            copy_lease.bytes = 16_777_216;
            docker_files::save(&temporary.layout, &record, &copy_lease).unwrap();
        }
        let cache: Arc<dyn Cache> = if live_writer {
            Arc::new(Unimock::new(()))
        } else {
            Arc::new(Unimock::new(
                CacheMock::budget.some_call(matching!(_)).returns(Ok(())),
            ))
        };
        let adapter = LinuxDockers {
            layout: temporary.layout.clone(),
            commands: Arc::new(LinuxCommands),
            cache,
        };
        let result = if live_writer {
            adapter.failed(&record).unwrap();
            adapter.discard(&record).await
        } else {
            adapter
                .prepare(
                    &record,
                    &facts(),
                    &scope(),
                    4096,
                    64 * 1024 * 1024,
                    64 * 1024 * 1024,
                )
                .await
                .map(|_| ())
        };
        assert_eq!(matches!(result, Err(Error::DockerFilesystem)), !live_writer);
        assert_eq!(
            base.exists(),
            live_writer,
            "Offline failure may retain only a source owned by a live writer"
        );
        assert_eq!(adapter.reserved(&record).unwrap(), live_writer);
        assert_eq!(
            adapter.report().unwrap().leased_bytes,
            if live_writer { 33_554_432 } else { 0 }
        );
    }
}
