//! The Mac host: the root check, the host name, and the hardware report.

use std::ffi::{CStr, CString};
use std::sync::{Arc, OnceLock};

use ciproxy_protocol::reports::HardwareReport;

use crate::boundaries::Host;
use crate::error::{Error, ErrorResultExt, Result};
use crate::macos::hardware::{
    AreaSpace, ImageNeeds, MacFacts, MacosVersion, report, virtualization,
};
use crate::macos::paths::AREA;

/// What the newest local image needs from its host, if the Mac has one.
pub(crate) trait NewestImage: Send + Sync {
    fn needs(&self) -> Option<ImageNeeds>;
}

pub(crate) struct MacHost {
    pub(crate) image: Arc<dyn NewestImage>,
    entitlement: OnceLock<bool>,
}

impl MacHost {
    pub(crate) fn new(image: Arc<dyn NewestImage>) -> Self {
        Self {
            image,
            entitlement: OnceLock::new(),
        }
    }

    fn facts(&self) -> Result<MacFacts> {
        Ok(MacFacts {
            processor: sysctl_text("machdep.cpu.brand_string")?,
            threads: u32::try_from(sysctl_number("hw.logicalcpu")?).context("read threads")?,
            memory_bytes: sysctl_number("hw.memsize")?,
            version: macos_version()?,
            framework_supported: sysctl_number("kern.hv_support").unwrap_or(0) == 1,
            entitlement: *self.entitlement.get_or_init(has_entitlement),
        })
    }
}

impl Host for MacHost {
    fn require_root(&self) -> Result<()> {
        let arm64 = sysctl_number("hw.optional.arm64").unwrap_or(0) == 1;
        if rustix::process::geteuid().as_raw() != 0 || !arm64 {
            return Err(Error::MacHostUnsupported);
        }
        Ok(())
    }

    fn hostname(&self) -> Result<String> {
        let mut buffer = [0_u8; 256];
        // SAFETY: The buffer is valid for its length, and gethostname
        // writes at most that many bytes.
        let status = unsafe { libc::gethostname(buffer.as_mut_ptr().cast(), buffer.len()) };
        if status != 0 {
            return Err(std::io::Error::last_os_error()).context("read host name");
        }
        let name = CStr::from_bytes_until_nul(&buffer).context("read host name")?;
        Ok(name.to_string_lossy().trim().to_owned())
    }

    fn hardware(&self) -> Result<HardwareReport> {
        let facts = self.facts()?;
        let state = virtualization(&facts, self.image.needs());
        Ok(report(&facts, state, area_space()))
    }
}

pub(crate) fn macos_version() -> Result<MacosVersion> {
    MacosVersion::parse(&sysctl_text("kern.osproductversion")?).ok_or(Error::MacHostUnsupported)
}

/// The space of the area volume, or none before install creates it.
fn area_space() -> Option<AreaSpace> {
    let stat = rustix::fs::statvfs(AREA).ok()?;
    let block = stat.f_frsize;
    Some(AreaSpace {
        capacity_bytes: stat.f_blocks.saturating_mul(block),
        free_bytes: stat.f_bavail.saturating_mul(block),
    })
}

/// Ask `codesign` once whether the running binary has the virtualization
/// entitlement. Without it, every VM start fails.
fn has_entitlement() -> bool {
    let Ok(binary) = std::env::current_exe() else {
        return false;
    };
    std::process::Command::new("codesign")
        .args(["-d", "--entitlements", "-", "--xml"])
        .arg(binary)
        .output()
        .is_ok_and(|output| {
            String::from_utf8_lossy(&output.stdout).contains("com.apple.security.virtualization")
        })
}

fn sysctl_raw(name: &str) -> Result<Vec<u8>> {
    let name = CString::new(name).context("name a sysctl")?;
    let mut length = 0_usize;
    // SAFETY: A null buffer asks only for the length.
    let status = unsafe {
        libc::sysctlbyname(
            name.as_ptr(),
            std::ptr::null_mut(),
            &mut length,
            std::ptr::null_mut(),
            0,
        )
    };
    if status != 0 {
        return Err(std::io::Error::last_os_error()).context("read a sysctl length");
    }
    let mut buffer = vec![0_u8; length];
    // SAFETY: The buffer is valid for `length` bytes.
    let status = unsafe {
        libc::sysctlbyname(
            name.as_ptr(),
            buffer.as_mut_ptr().cast(),
            &mut length,
            std::ptr::null_mut(),
            0,
        )
    };
    if status != 0 {
        return Err(std::io::Error::last_os_error()).context("read a sysctl");
    }
    buffer.truncate(length);
    Ok(buffer)
}

fn sysctl_text(name: &str) -> Result<String> {
    let bytes = sysctl_raw(name)?;
    let text = CStr::from_bytes_until_nul(&bytes)
        .map(|text| text.to_string_lossy().into_owned())
        .unwrap_or_else(|_| String::from_utf8_lossy(&bytes).into_owned());
    Ok(text.trim().to_owned())
}

/// Read a 32-bit or 64-bit integer sysctl.
fn sysctl_number(name: &str) -> Result<u64> {
    let bytes = sysctl_raw(name)?;
    match bytes.len() {
        4 => Ok(u64::from(u32::from_ne_bytes(
            bytes.try_into().map_err(|_| Error::MacHostUnsupported)?,
        ))),
        8 => Ok(u64::from_ne_bytes(
            bytes.try_into().map_err(|_| Error::MacHostUnsupported)?,
        )),
        _ => Err(Error::MacHostUnsupported),
    }
}
