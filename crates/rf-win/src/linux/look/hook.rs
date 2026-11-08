//! The system-wide keyboard hook (Space in Explorer opens DirectLook). Wayland has no global
//! key hooks: `start` installs nothing and never calls back.

use super::keys::LookAction;
use super::target::TargetKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LookEvent {
    pub action: LookAction,
    pub target: TargetKind,
    pub hwnd: isize,
}

pub fn set_own_windows(_hwnds: Vec<isize>) {}

pub struct HookHandle(());

impl HookHandle {
    pub fn stop(self) {}
}

pub fn start(_on_event: impl Fn(LookEvent) + Send + 'static) -> HookHandle {
    HookHandle(())
}
