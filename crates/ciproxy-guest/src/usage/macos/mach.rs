//! Read whole-VM CPU and memory counters from the Mach host port.

use std::io::{self, ErrorKind};
use std::mem::{MaybeUninit, offset_of, size_of};
use std::thread::available_parallelism;

use ciproxy_protocol::platform::Platform;

use crate::usage::{CpuReading, Reading, Source};

use super::memory_mib;

/// The revision 1 prefix ends before `swapped_count` and has all needed counts.
const VM_INFO_COUNT: libc::mach_msg_type_number_t =
    (offset_of!(libc::vm_statistics64, swapped_count) / size_of::<libc::integer_t>())
        as libc::mach_msg_type_number_t;

unsafe extern "C" {
    /// Get the kernel page size. `size` must point to writable storage.
    fn host_page_size(host: libc::host_t, size: *mut libc::vm_size_t) -> libc::kern_return_t;
    /// Release one reference to a port right in the given task's IPC space.
    fn mach_port_deallocate(
        task: libc::mach_port_t,
        port: libc::mach_port_t,
    ) -> libc::kern_return_t;
}

/// The usage source for an own-machine macOS VM.
pub(crate) struct Mach;

impl Source for Mach {
    fn platform(&self) -> Platform {
        Platform::MacosArm64
    }

    fn read(&self) -> io::Result<Reading> {
        let at_ms = Reading::now_ms()?;
        let host = HostPort::acquire()?;
        let cpu = host.cpu()?;
        let max_cpu_milli_vcpu = u32::try_from(available_parallelism()?.get())
            .ok()
            .and_then(|cpus| cpus.checked_mul(1_000))
            .ok_or(ErrorKind::InvalidData)?;
        Ok(Reading {
            at_ms,
            cpu,
            max_cpu_milli_vcpu,
            memory_mib: host.memory()?,
            oom_kills: 0,
        })
    }

    fn worker_present(&self, _runner_pid: u32) -> io::Result<bool> {
        Err(ErrorKind::Unsupported.into())
    }
}

struct HostPort(libc::host_t);

impl HostPort {
    /// The Mach call takes no inputs. This wrapper owns its send reference.
    #[expect(deprecated, reason = "use the Mach binding in the existing libc crate")]
    fn acquire() -> io::Result<Self> {
        let port = unsafe { libc::mach_host_self() };
        if port == libc::MACH_PORT_NULL as libc::mach_port_t {
            return Err(io::Error::other("Mach host port unavailable"));
        }
        Ok(Self(port))
    }

    /// The CPU flavor writes up to `count` Mach integers. Read only a full result.
    fn cpu(&self) -> io::Result<CpuReading> {
        let mut info = MaybeUninit::<libc::host_cpu_load_info>::uninit();
        let mut count = libc::HOST_CPU_LOAD_INFO_COUNT;
        let status = unsafe {
            libc::host_statistics(
                self.0,
                libc::HOST_CPU_LOAD_INFO,
                info.as_mut_ptr().cast(),
                &mut count,
            )
        };
        if status != libc::KERN_SUCCESS || count != libc::HOST_CPU_LOAD_INFO_COUNT {
            return Err(ErrorKind::InvalidData.into());
        }
        let ticks = unsafe { info.assume_init() }.cpu_ticks;
        Ok(CpuReading::HostTicks {
            busy: [
                ticks[libc::CPU_STATE_USER as usize],
                ticks[libc::CPU_STATE_NICE as usize],
                ticks[libc::CPU_STATE_SYSTEM as usize],
            ],
            idle: ticks[libc::CPU_STATE_IDLE as usize],
        })
    }

    /// Limit the write to the revision 1 prefix. The remaining fields stay zero.
    fn memory(&self) -> io::Result<u32> {
        let mut info = MaybeUninit::<libc::vm_statistics64>::zeroed();
        let mut count = VM_INFO_COUNT;
        let status = unsafe {
            libc::host_statistics64(
                self.0,
                libc::HOST_VM_INFO64,
                info.as_mut_ptr().cast(),
                &mut count,
            )
        };
        if status != libc::KERN_SUCCESS || count != VM_INFO_COUNT {
            return Err(ErrorKind::InvalidData.into());
        }
        let info = unsafe { info.assume_init() };
        memory_mib(
            info.internal_page_count,
            info.purgeable_count,
            info.wire_count,
            info.compressor_page_count,
            self.page_size()?,
        )
        .ok_or_else(|| ErrorKind::InvalidData.into())
    }

    /// The output points to one `vm_size_t` on the stack.
    fn page_size(&self) -> io::Result<usize> {
        let mut size: libc::vm_size_t = 0;
        let status = unsafe { host_page_size(self.0, &mut size) };
        if status != libc::KERN_SUCCESS || size == 0 {
            return Err(ErrorKind::InvalidData.into());
        }
        Ok(size)
    }
}

impl Drop for HostPort {
    /// Release the send reference that `acquire` got for this task.
    #[expect(deprecated, reason = "use the Mach binding in the existing libc crate")]
    fn drop(&mut self) {
        let _ = unsafe { mach_port_deallocate(libc::mach_task_self(), self.0) };
    }
}
