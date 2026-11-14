//! macOS: run as our own "responsible" process.
//!
//! macOS asks the *responsible* app for its Bluetooth permission note. When the
//! tester is started by another app (Craft Anything, an editor…), that other app
//! would be asked, and macOS kills the tester if it has no Bluetooth note. So the
//! tester starts itself once more with "disclaim" set; the copy is then
//! responsible for itself and uses the note embedded in its own Info.plist.

use std::ffi::CString;
use std::os::raw::c_int;

const MARK: &str = "LICHTERKETTE_DISCLAIMED";

unsafe extern "C" {
    fn responsibility_spawnattrs_setdisclaim(attr: *mut libc::posix_spawnattr_t, disclaim: c_int) -> c_int;
}

/// Returns `Some(exit_code)` when this process only started the disclaimed copy.
pub fn relaunch_disclaimed() -> Option<i32> {
    if std::env::var_os(MARK).is_some() {
        return None;
    }
    let exe = std::env::current_exe().ok()?;
    let exe_c = CString::new(exe.to_string_lossy().as_bytes()).ok()?;
    let args: Vec<CString> = std::env::args()
        .filter_map(|a| CString::new(a).ok())
        .collect();
    let mut argv: Vec<*mut libc::c_char> = args.iter().map(|a| a.as_ptr() as *mut _).collect();
    argv.push(std::ptr::null_mut());

    // Environment plus the marker, so the copy doesn't relaunch again.
    let mut env: Vec<CString> = std::env::vars()
        .filter_map(|(k, v)| CString::new(format!("{k}={v}")).ok())
        .collect();
    env.push(CString::new(format!("{MARK}=1")).ok()?);
    let mut envp: Vec<*mut libc::c_char> = env.iter().map(|e| e.as_ptr() as *mut _).collect();
    envp.push(std::ptr::null_mut());

    unsafe {
        let mut attr: libc::posix_spawnattr_t = std::mem::zeroed();
        if libc::posix_spawnattr_init(&mut attr) != 0 {
            return None;
        }
        if responsibility_spawnattrs_setdisclaim(&mut attr, 1) != 0 {
            libc::posix_spawnattr_destroy(&mut attr);
            return None;
        }
        let mut pid: libc::pid_t = 0;
        let rc = libc::posix_spawn(
            &mut pid,
            exe_c.as_ptr(),
            std::ptr::null(),
            &attr,
            argv.as_ptr(),
            envp.as_ptr(),
        );
        libc::posix_spawnattr_destroy(&mut attr);
        if rc != 0 {
            return None; // fall back to running here
        }
        let mut status: c_int = 0;
        while libc::waitpid(pid, &mut status, 0) < 0 {
            if std::io::Error::last_os_error().raw_os_error() != Some(libc::EINTR) {
                return Some(1);
            }
        }
        Some(if libc::WIFEXITED(status) { libc::WEXITSTATUS(status) } else { 1 })
    }
}
