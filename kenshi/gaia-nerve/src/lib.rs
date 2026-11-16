//! gaia_nerve.dll: Kenshi's in-process nerve, the successor of kenshi-bridge.
//!
//! kenshi-bridge proved a Rust DLL can live inside Kenshi but had to pose as an
//! Ogre RenderSystem. Now RE_Kenshi + KenshiLua run the game, and KenshiLua's
//! LuaJIT has FFI, so GaiaMind's Lua loads this DLL directly and calls plain C.
//!
//! The DLL listens on 127.0.0.1 only. Every newline-terminated line a client
//! writes is queued for Lua (`gaia_nerve_recv`); every line Lua sends
//! (`gaia_nerve_send`) goes to all connected clients. Lua runs on the game
//! thread and never blocks here: accept/read happen on background threads and
//! writes carry a short timeout.

use std::collections::VecDeque;
use std::ffi::{c_char, CStr};
use std::io::{BufRead, BufReader, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

const VERSION: &[u8] = b"gaia-nerve 0.1.0\0";
const MAX_QUEUE: usize = 1024;
const MAX_LINE: usize = 4096;
const WRITE_TIMEOUT: Duration = Duration::from_millis(50);

#[derive(Default)]
struct Nerve {
    started: bool,
    next_id: u64,
    inbound: VecDeque<String>,
    clients: Vec<(u64, TcpStream)>,
}

fn nerve() -> &'static Mutex<Nerve> {
    static NERVE: OnceLock<Mutex<Nerve>> = OnceLock::new();
    NERVE.get_or_init(|| Mutex::new(Nerve::default()))
}

fn with<T>(f: impl FnOnce(&mut Nerve) -> T) -> T {
    let mut guard = nerve().lock().unwrap_or_else(|p| p.into_inner());
    f(&mut guard)
}

fn read_client(id: u64, stream: TcpStream) {
    for line in BufReader::new(stream).lines() {
        let Ok(mut line) = line else { break };
        line.truncate(MAX_LINE);
        with(|n| {
            if n.inbound.len() >= MAX_QUEUE {
                n.inbound.pop_front();
            }
            n.inbound.push_back(line);
        });
    }
    with(|n| n.clients.retain(|(cid, _)| *cid != id));
}

fn accept_loop(listener: TcpListener) {
    for stream in listener.incoming().flatten() {
        let _ = stream.set_nodelay(true);
        let _ = stream.set_write_timeout(Some(WRITE_TIMEOUT));
        if let Ok(reader) = stream.try_clone() {
            let id = with(|n| {
                n.next_id += 1;
                n.clients.push((n.next_id, stream));
                n.next_id
            });
            thread::spawn(move || read_client(id, reader));
        }
    }
}

/// Start listening on 127.0.0.1:`port`. Returns 1 on success or if already
/// started, 0 if the port could not be bound.
#[no_mangle]
pub extern "C" fn gaia_nerve_start(port: u16) -> i32 {
    if with(|n| n.started) {
        return 1;
    }
    let Ok(listener) = TcpListener::bind((Ipv4Addr::LOCALHOST, port)) else { return 0 };
    with(|n| n.started = true);
    thread::spawn(move || accept_loop(listener));
    1
}

/// Send one line (without newline) to every connected client. Returns the
/// number of clients that received it; dead clients are dropped.
///
/// # Safety
/// `line` must be null or a valid NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn gaia_nerve_send(line: *const c_char) -> i32 {
    if line.is_null() {
        return 0;
    }
    let mut bytes = unsafe { CStr::from_ptr(line) }.to_bytes().to_vec();
    bytes.retain(|&b| b != b'\n' && b != b'\r');
    bytes.push(b'\n');
    with(|n| {
        n.clients.retain_mut(|(_, c)| c.write_all(&bytes).is_ok());
        n.clients.len() as i32
    })
}

/// Pop the next inbound line into `buf` (NUL-terminated). Returns its length,
/// 0 when the queue is empty, or -1 if `cap` is too small (the line stays queued).
///
/// # Safety
/// `buf` must point to at least `cap` writable bytes.
#[no_mangle]
pub unsafe extern "C" fn gaia_nerve_recv(buf: *mut c_char, cap: usize) -> i32 {
    if buf.is_null() || cap == 0 {
        return -1;
    }
    with(|n| {
        let Some(line) = n.inbound.front() else { return 0 };
        if line.len() + 1 > cap {
            return -1;
        }
        let len = line.len();
        unsafe {
            std::ptr::copy_nonoverlapping(line.as_ptr(), buf as *mut u8, len);
            *buf.add(len) = 0;
        }
        n.inbound.pop_front();
        len as i32
    })
}

/// Number of connected clients.
#[no_mangle]
pub extern "C" fn gaia_nerve_clients() -> i32 {
    with(|n| n.clients.len() as i32)
}

#[no_mangle]
pub extern "C" fn gaia_nerve_version() -> *const c_char {
    VERSION.as_ptr() as *const c_char
}
