//! The few things the dock's menu and previews ask of other programs' windows: a title, whether it is minimised, a request to close.
//! None of them waits for the other program: a window that hangs must not hang the dock.
//!
//! A window is named by a [`WinRef`]: its handle AND the process it belonged to when the dock looked at it. Window handles are reused
//! by Windows after a window is destroyed, and a menu can stay open (or a preview) for a long time after the list of the app's windows
//! was taken; acting on a bare handle could then close a window of another program. An action therefore checks, at the moment it is
//! asked for, that the handle still belongs to the same process (and is still a visible window) before it does anything.

use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, WPARAM};
use windows::Win32::System::Threading::{GetCurrentProcessId, OpenProcess, PROCESS_TERMINATE, TerminateProcess};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowExW, GetClassNameW, GetShellWindow, GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindow, IsWindowVisible, PostMessageW, SC_CLOSE, WM_SYSCOMMAND,
};

fn h(hwnd: isize) -> HWND {
    HWND(hwnd as *mut _)
}

/// The window's title; empty when it has none or is gone. For a window of another process Windows answers from its own copy of the
/// caption, without sending the window a message, so a program that does not respond cannot hold this up.
pub fn window_title(hwnd: isize) -> String {
    if hwnd == 0 {
        return String::new();
    }
    let mut buf = [0u16; 512];
    let n = unsafe { GetWindowTextW(h(hwnd), &mut buf) };
    if n <= 0 { String::new() } else { String::from_utf16_lossy(&buf[..n as usize]) }
}

/// Whether the window is minimised.
pub fn is_minimized(hwnd: isize) -> bool {
    hwnd != 0 && unsafe { IsIconic(h(hwnd)).as_bool() }
}

/// A window as the dock saw it: its handle and the process it belonged to then.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WinRef {
    pub hwnd: isize,
    pub pid: u32,
}

/// What [`WinRef`] checks and does, so that the decision can be tested without another program's windows.
pub trait WinApi {
    /// The process the window belongs to now; `None` when there is no such window.
    fn owner(&self, hwnd: isize) -> Option<u32>;
    fn visible(&self, hwnd: isize) -> bool;
    /// Posts the window the command its own close button gives (`WM_SYSCOMMAND` with `SC_CLOSE`). False when it could not be posted.
    fn post_close(&self, hwnd: isize) -> bool;
}

/// The real windows.
pub struct Win32;

impl WinApi for Win32 {
    fn owner(&self, hwnd: isize) -> Option<u32> {
        if hwnd == 0 || !unsafe { IsWindow(Some(h(hwnd))).as_bool() } {
            return None;
        }
        let mut pid = 0u32;
        unsafe { GetWindowThreadProcessId(h(hwnd), Some(&mut pid)) };
        (pid != 0).then_some(pid)
    }

    fn visible(&self, hwnd: isize) -> bool {
        hwnd != 0 && unsafe { IsWindowVisible(h(hwnd)).as_bool() }
    }

    fn post_close(&self, hwnd: isize) -> bool {
        unsafe { PostMessageW(Some(h(hwnd)), WM_SYSCOMMAND, WPARAM(SC_CLOSE as usize), LPARAM(0)).is_ok() }
    }
}

impl WinRef {
    /// The window now, or `None` when `hwnd` is no window.
    pub fn of_with(api: &dyn WinApi, hwnd: isize) -> Option<WinRef> {
        api.owner(hwnd).map(|pid| WinRef { hwnd, pid })
    }

    pub fn of(hwnd: isize) -> Option<WinRef> {
        Self::of_with(&Win32, hwnd)
    }

    /// Whether the handle still names the window that was seen: it exists, belongs to the same process and is visible (a minimised
    /// window is visible).
    pub fn is_still_with(&self, api: &dyn WinApi) -> bool {
        api.owner(self.hwnd) == Some(self.pid) && api.visible(self.hwnd)
    }

    pub fn is_still(&self) -> bool {
        self.is_still_with(&Win32)
    }
}

/// The windows now, those that are gone left out.
pub fn refs(hwnds: &[isize]) -> Vec<WinRef> {
    hwnds.iter().filter_map(|&w| WinRef::of(w)).collect()
}

/// Asks a window to close, the way its own close button does (`WM_SYSCOMMAND` with `SC_CLOSE`, posted: the program decides, may ask the
/// user about unsaved work, and a window that has disabled its close command, or has no close button, is not sent a request the
/// taskbar would not have sent either). Only when the handle is still the window that was seen (see the module's note). False when
/// nothing was posted.
pub fn close_window_with(api: &dyn WinApi, w: WinRef) -> bool {
    w.is_still_with(api) && api.post_close(w.hwnd)
}

pub fn close_window(w: WinRef) -> bool {
    close_window_with(&Win32, w)
}

// ---------------------------------------------------------------- end task

/// The window class of the frame Windows puts around every packaged (Store) app's own window. All such frames belong to ONE
/// process (ApplicationFrameHost): ending it would end every Store app at once.
const FRAME_CLASS: &str = "ApplicationFrameWindow";
/// The app's own window inside such a frame.
const CORE_CLASS: &str = "Windows.UI.Core.CoreWindow";

/// What a window says about the process behind it, for "end task".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskOwner {
    /// The process the window belongs to.
    pub pid: u32,
    /// The window is a packaged app's frame (it belongs to the host of all such apps, not to the app).
    pub frame: bool,
    /// For a frame: the process of the app's own window inside it, when there is one (a suspended app's window is not there).
    pub inner: Option<u32>,
}

impl TaskOwner {
    /// The app's own process: the window's, and for a Store app's frame the process of the window inside it (never the host of all
    /// frames; `None` while there is nothing inside).
    pub fn app_pid(&self) -> Option<u32> {
        if self.frame { self.inner.filter(|&p| p != self.pid) } else { Some(self.pid) }
    }
}

/// The processes "end task" ends for these windows (pure). Each once; a frame counts as the app inside it and never as the host;
/// never this process, never the shell's (Explorer's windows belong to the process that is the desktop and the taskbar), never the
/// system's two (0 and 4).
pub fn end_targets(owners: &[TaskOwner], own: u32, shell: Option<u32>) -> Vec<u32> {
    let mut out: Vec<u32> = Vec::new();
    for o in owners {
        if let Some(pid) = o.app_pid().filter(|&p| p > 4 && p != own && Some(p) != shell && !out.contains(&p)) {
            out.push(pid);
        }
    }
    out
}

fn class_of(hwnd: isize) -> String {
    let mut buf = [0u16; 128];
    let n = unsafe { GetClassNameW(h(hwnd), &mut buf) };
    if n <= 0 { String::new() } else { String::from_utf16_lossy(&buf[..n as usize]) }
}

/// The process behind a window as the dock saw it; `None` when the handle is no longer that window.
pub fn task_owner(w: WinRef) -> Option<TaskOwner> {
    if !w.is_still() {
        return None;
    }
    let frame = class_of(w.hwnd) == FRAME_CLASS;
    let inner = frame
        .then(|| {
            let class = crate::wide::wide(CORE_CLASS);
            let child = unsafe { FindWindowExW(Some(h(w.hwnd)), None, windows::core::PCWSTR(class.as_ptr()), None) }.ok()?;
            let mut pid = 0u32;
            unsafe { GetWindowThreadProcessId(child, Some(&mut pid)) };
            (pid != 0).then_some(pid)
        })
        .flatten();
    Some(TaskOwner { pid: w.pid, frame, inner })
}

/// The process of the shell's desktop window (Explorer's, while it is the shell); `None` when there is none.
pub fn shell_pid() -> Option<u32> {
    let shell = unsafe { GetShellWindow() };
    if shell.0.is_null() {
        return None;
    }
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(shell, Some(&mut pid)) };
    (pid != 0).then_some(pid)
}

/// The processes "end task" would end for these windows now ([`end_targets`], with this process and the shell left out).
pub fn end_task_processes(windows: &[WinRef]) -> Vec<u32> {
    let owners: Vec<TaskOwner> = windows.iter().filter_map(|w| task_owner(*w)).collect();
    end_targets(&owners, unsafe { GetCurrentProcessId() }, shell_pid())
}

/// Ends the processes behind these windows, without asking them: what Windows' "End task" does, for a program that does not answer
/// its close button. Unsaved work in them is lost; the user chose the row that says so. Returns how many were ended and how many
/// could not be (a program running with administrator rights cannot be ended by this process).
pub fn end_task(windows: &[WinRef]) -> (usize, usize) {
    let (mut ended, mut refused) = (0, 0);
    for pid in end_task_processes(windows) {
        let ok = unsafe {
            match OpenProcess(PROCESS_TERMINATE, false, pid) {
                Ok(handle) => {
                    let ok = TerminateProcess(handle, 1).is_ok();
                    let _ = CloseHandle(handle);
                    ok
                }
                Err(_) => false,
            }
        };
        if ok { ended += 1 } else { refused += 1 }
    }
    (ended, refused)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    /// Windows that exist only in the test: (handle, process, visible).
    struct Fake {
        windows: Vec<(isize, u32, bool)>,
        posted: RefCell<Vec<isize>>,
    }

    impl WinApi for Fake {
        fn owner(&self, hwnd: isize) -> Option<u32> {
            self.windows.iter().find(|w| w.0 == hwnd).map(|w| w.1)
        }
        fn visible(&self, hwnd: isize) -> bool {
            self.windows.iter().any(|w| w.0 == hwnd && w.2)
        }
        fn post_close(&self, hwnd: isize) -> bool {
            self.posted.borrow_mut().push(hwnd);
            true
        }
    }

    fn fake(windows: Vec<(isize, u32, bool)>) -> Fake {
        Fake { windows, posted: RefCell::new(Vec::new()) }
    }

    #[test]
    fn end_task_ends_the_apps_own_processes_and_never_the_shell_the_host_or_itself() {
        let own = 500;
        let shell = Some(600);
        let plain = |pid| TaskOwner { pid, frame: false, inner: None };
        // two windows of one process, one of another: each process once, in the windows' order
        assert_eq!(end_targets(&[plain(10), plain(10), plain(11)], own, shell), [10, 11]);
        // Explorer's folder windows belong to the shell's process; the dock's own process is never a target; nor are 0 and 4
        assert_eq!(end_targets(&[plain(600), plain(500), plain(0), plain(4), plain(12)], own, shell), [12]);
        assert_eq!(end_targets(&[plain(600)], own, None), [600], "with no shell window there is nothing to protect");
        // a Store app: the app inside the frame, never the host of all frames
        let frame = |inner| TaskOwner { pid: 77, frame: true, inner };
        assert_eq!(end_targets(&[frame(Some(90))], own, shell), [90]);
        assert!(end_targets(&[frame(None)], own, shell).is_empty(), "a suspended app has no window inside: nothing is ended");
        assert!(end_targets(&[frame(Some(77))], own, shell).is_empty(), "the host named as its own inside is still the host");
        assert_eq!(end_targets(&[frame(Some(90)), frame(Some(91)), plain(77)], own, shell), [90, 91, 77], "the host itself only when a plain window of it is asked for");
        assert!(end_targets(&[], own, shell).is_empty());
    }

    #[test]
    fn nothing_is_ended_for_a_handle_that_is_no_window() {
        assert_eq!(task_owner(WinRef { hwnd: 0x7FFF_FFF1, pid: 1 }), None);
        assert!(end_task_processes(&[WinRef { hwnd: 0x7FFF_FFF1, pid: 1 }, WinRef { hwnd: 0, pid: 2 }]).is_empty());
        assert_eq!(end_task(&[WinRef { hwnd: 0x7FFF_FFF1, pid: 1 }]), (0, 0));
    }

    #[test]
    fn a_window_that_is_not_there_has_no_title_is_not_minimised_and_is_not_seen() {
        assert_eq!(window_title(0), "");
        assert!(!is_minimized(0));
        assert_eq!(WinRef::of(0), None);
        // a number that is no window (never a real handle)
        assert_eq!(WinRef::of(0x7FFF_FFF1), None);
        assert!(!close_window(WinRef { hwnd: 0x7FFF_FFF1, pid: 1 }));
        assert!(!close_window(WinRef { hwnd: 0, pid: 1 }));
        assert_eq!(window_title(0x7FFF_FFF1), "");
    }

    #[test]
    fn a_window_that_is_still_the_one_seen_is_asked_to_close() {
        let api = fake(vec![(10, 500, true)]);
        let w = WinRef::of_with(&api, 10).unwrap();
        assert_eq!(w, WinRef { hwnd: 10, pid: 500 });
        assert!(close_window_with(&api, w));
        assert_eq!(*api.posted.borrow(), vec![10]);
    }

    #[test]
    fn a_handle_that_another_program_has_taken_over_is_never_sent_a_close() {
        // the menu saw window 10 of process 500; the window went and Windows gave the number to a window of process 900
        let seen = WinRef { hwnd: 10, pid: 500 };
        let api = fake(vec![(10, 900, true)]);
        assert!(!seen.is_still_with(&api));
        assert!(!close_window_with(&api, seen));
        assert!(api.posted.borrow().is_empty(), "nothing is posted to a window that is not the one that was seen");
    }

    #[test]
    fn a_window_that_is_gone_or_hidden_is_not_sent_a_close_either() {
        let seen = WinRef { hwnd: 10, pid: 500 };
        let gone = fake(vec![]);
        assert!(!close_window_with(&gone, seen));
        let hidden = fake(vec![(10, 500, false)]);
        assert!(!close_window_with(&hidden, seen));
        assert!(gone.posted.borrow().is_empty() && hidden.posted.borrow().is_empty());
    }

    #[test]
    fn the_request_is_the_command_of_the_close_button_not_a_bare_close_message() {
        // WM_SYSCOMMAND / SC_CLOSE is what the title bar's button and the taskbar send; a window that disabled its close command
        // refuses it, which a posted WM_CLOSE would not
        assert_eq!(SC_CLOSE, 0xF060);
        assert_eq!(WM_SYSCOMMAND, 0x0112);
    }
}
