//! A vsock echo server for the guest side of the macOS tests.
//!
//! Build it on the Mac, copy it into a guest, and run it there with a port:
//! `guest_vsock_echo 5252`. It echoes each connection until it closes.

#[cfg(target_os = "macos")]
fn main() -> std::io::Result<()> {
    use std::io::{Read, Write};
    use std::os::fd::FromRawFd;
    use std::os::unix::net::UnixStream;

    let port: u32 = std::env::args()
        .nth(1)
        .and_then(|port| port.parse().ok())
        .expect("usage: guest_vsock_echo <port>");
    // SAFETY: Plain socket calls with a correctly sized `sockaddr_vm`.
    let listener = unsafe {
        let listener = libc::socket(libc::AF_VSOCK, libc::SOCK_STREAM, 0);
        if listener < 0 {
            return Err(std::io::Error::last_os_error());
        }
        let mut address: libc::sockaddr_vm = std::mem::zeroed();
        address.svm_len = std::mem::size_of::<libc::sockaddr_vm>() as u8;
        address.svm_family = libc::AF_VSOCK as u8;
        address.svm_port = port;
        address.svm_cid = libc::VMADDR_CID_ANY;
        let size = std::mem::size_of::<libc::sockaddr_vm>() as libc::socklen_t;
        if libc::bind(listener, (&raw const address).cast(), size) != 0
            || libc::listen(listener, 8) != 0
        {
            return Err(std::io::Error::last_os_error());
        }
        listener
    };
    loop {
        // SAFETY: `accept` on a listening socket, with no address out.
        let client = unsafe { libc::accept(listener, std::ptr::null_mut(), std::ptr::null_mut()) };
        if client < 0 {
            continue;
        }
        // SAFETY: `client` is a new descriptor that nothing else owns.
        let mut stream = unsafe { UnixStream::from_raw_fd(client) };
        let mut buffer = [0_u8; 4096];
        while let Ok(count) = stream.read(&mut buffer) {
            if count == 0 || stream.write_all(&buffer[..count]).is_err() {
                break;
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("guest_vsock_echo runs only in a macOS guest");
}
