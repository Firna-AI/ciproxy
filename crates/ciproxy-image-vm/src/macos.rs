//! The macOS commands over `ciproxy-vz`.

use std::path::Path;
use std::process::ExitCode;
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use ciproxy_vz::lease::{LEASE_FILE, lookup};
use ciproxy_vz::{Bundle, Display, RestoreImage, StopReason, Vm, VmSettings, install};
use futures_executor::block_on;

use crate::{Command, Machine};

const GIB: u64 = 1024 * 1024 * 1024;

pub fn run(command: Command) -> Result<ExitCode, String> {
    match command {
        Command::Install {
            restore_image,
            bundle,
            disk_bytes,
            machine,
        } => {
            let image = block_on(RestoreImage::load(&restore_image)).map_err(text)?;
            eprintln!("installing macOS {}", image.build_version());
            let started = Instant::now();
            block_on(install(
                &Bundle::new(bundle),
                &image,
                &settings(machine),
                disk_bytes,
                |done| eprintln!("install {:.0}%", done * 100.0),
            ))
            .map_err(text)?;
            eprintln!("installed in {} s", started.elapsed().as_secs());
            Ok(ExitCode::SUCCESS)
        }
        Command::Run {
            bundle,
            recovery,
            vnc_password_file,
            status_file,
            stop_file,
            machine,
        } => run_vm(
            &Bundle::new(bundle),
            recovery,
            &vnc_password_file,
            &status_file,
            &stop_file,
            machine,
        ),
        Command::Ip {
            bundle,
            wait_seconds,
        } => {
            let mac = Bundle::new(bundle).read_mac_address().map_err(text)?;
            let deadline = Instant::now() + Duration::from_secs(wait_seconds);
            loop {
                if let Some(address) = lookup(LEASE_FILE.as_ref(), mac).map_err(text)? {
                    println!("{address}");
                    return Ok(ExitCode::SUCCESS);
                }
                if Instant::now() >= deadline {
                    return Err(format!("no DHCP lease for {mac}"));
                }
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
}

fn run_vm(
    bundle: &Bundle,
    recovery: bool,
    password_file: &Path,
    status_file: &Path,
    stop_file: &Path,
    machine: Machine,
) -> Result<ExitCode, String> {
    let password = std::fs::read_to_string(password_file).map_err(text)?;
    let vm = Arc::new(Vm::new(bundle, &settings(machine)).map_err(text)?);
    if recovery {
        block_on(vm.start_recovery()).map_err(text)?;
    } else {
        block_on(vm.start()).map_err(text)?;
    }
    let vnc = vm.start_vnc(0, Some(password.trim())).map_err(text)?;
    let mut port = vnc.port();
    for _ in 0..50 {
        if port != 0 {
            break;
        }
        thread::sleep(Duration::from_millis(100));
        port = vnc.port();
    }
    let status = format!(
        "{{\"vnc_port\":{port},\"mac_address\":\"{}\"}}\n",
        vm.mac_address()
    );
    std::fs::write(status_file, status).map_err(text)?;
    eprintln!("running, VNC port {port}");
    let (sender, receiver) = mpsc::channel();
    let waiter = Arc::clone(&vm);
    thread::spawn(move || {
        let _ignored = sender.send(block_on(waiter.stopped()));
    });
    let mut stopping = false;
    loop {
        match receiver.recv_timeout(Duration::from_secs(1)) {
            Ok(reason) => {
                drop(vnc);
                eprintln!("stopped: {reason:?}");
                return Ok(match reason {
                    StopReason::Guest | StopReason::Host => ExitCode::SUCCESS,
                    StopReason::Error(_) => ExitCode::FAILURE,
                });
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err("the stop waiter ended".to_owned());
            }
        }
        if !stopping && stop_file.exists() {
            stopping = true;
            block_on(vm.stop()).map_err(text)?;
        }
    }
}

fn settings(machine: Machine) -> VmSettings {
    VmSettings {
        cpus: machine.cpus,
        memory_bytes: machine.memory_gib * GIB,
        display: Some(Display {
            width: machine.width,
            height: machine.height,
            pixels_per_inch: 80,
        }),
    }
}

fn text(error: impl std::fmt::Display) -> String {
    error.to_string()
}
