//! The "shell" part of the context menu. Windows merges in Explorer's menu (IContextMenu);
//! Linux has no such thing. TODO(linux, phase 5): items from `.desktop` actions and
//! "Open with" apps for the selection's MIME type. Until then the menu has no shell items and
//! shows only the app's own entries.

use std::path::{Path, PathBuf};

use crate::compat::HWND;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuOutcome {
    Dismissed,
    /// User chose "Rename": the app should start inline rename.
    Rename,
    /// A shell verb ran (e.g. "delete", "copy", "properties", or a handler without a verb).
    Invoked(String),
    /// One of the app's own items (see `ExtraItem`) was chosen.
    Custom(u32),
}

#[derive(Debug, Clone)]
pub struct ExtraItem {
    pub id: u32,
    pub label: String,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub enum ShellTarget {
    /// Items of one folder (the selection).
    Items(Vec<PathBuf>),
    /// A single item through its real parent (sidebar entries, drive roots).
    One(PathBuf),
    /// The empty area of a folder.
    Background(PathBuf),
}

#[derive(Debug, Clone)]
pub struct ShellIcon {
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ShellItem {
    /// Command id for [`invoke`]; 0 for separators and submenus.
    pub id: u32,
    pub label: String,
    /// Text after a tab in the label (shortcut hint), if any.
    pub hint: Option<String>,
    /// Canonical verb ("open", "properties"…), empty if the handler has none.
    pub verb: String,
    pub icon: Option<ShellIcon>,
    pub enabled: bool,
    pub separator: bool,
    pub children: Vec<ShellItem>,
}

pub fn show_for_items(_owner: HWND, _items: &[PathBuf], _screen: (i32, i32), _extended: bool, _extra: &[ExtraItem]) -> anyhow::Result<MenuOutcome> {
    Err(super::unsupported("native context menu"))
}

pub fn show_for_one(_owner: HWND, _item: &Path, _screen: (i32, i32), _extended: bool, _extra: &[ExtraItem]) -> anyhow::Result<MenuOutcome> {
    Err(super::unsupported("native context menu"))
}

pub fn show_for_background(_owner: HWND, _dir: &Path, _screen: (i32, i32), _extended: bool, _extra: &[ExtraItem]) -> anyhow::Result<MenuOutcome> {
    Err(super::unsupported("native context menu"))
}

/// The shell items for `target`: none yet (see the module docs). Session 0 needs no release.
pub fn build(_owner: HWND, _target: &ShellTarget, _extended: bool) -> anyhow::Result<(u64, Vec<ShellItem>)> {
    Ok((0, Vec::new()))
}

pub fn invoke(_owner: HWND, _session: u64, _id: u32, _screen: (i32, i32)) -> anyhow::Result<MenuOutcome> {
    Ok(MenuOutcome::Dismissed)
}

pub fn release(_session: u64) {}

/// Runs a verb on the target. "open" works; the rest needs the phase 5 menu.
pub fn invoke_verb(_owner: HWND, target: &ShellTarget, verb: &str, _screen: (i32, i32)) -> anyhow::Result<()> {
    match (verb, target) {
        ("open", ShellTarget::One(p)) => super::shell::open(p),
        ("open", ShellTarget::Items(items)) => items.iter().try_for_each(|p| super::shell::open(p)),
        _ => Err(super::unsupported(&format!("shell verb \"{verb}\""))),
    }
}

/// "&Open\tCtrl+O" → ("Open", Some("Ctrl+O")); "&&" stays a literal ampersand.
pub fn clean_label(raw: &str) -> (String, Option<String>) {
    let (text, hint) = match raw.split_once('\t') {
        Some((t, h)) => (t, Some(h.trim().to_string()).filter(|h| !h.is_empty())),
        None => (raw, None),
    };
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '&' {
            if chars.peek() == Some(&'&') {
                chars.next();
                out.push('&');
            }
            continue;
        }
        out.push(c);
    }
    (out.trim().to_string(), hint)
}

/// A small native popup menu of the app's own items. None here: the app uses its own menus.
pub fn show_simple(_owner: HWND, _screen: (i32, i32), _items: &[ExtraItem]) -> Option<u32> {
    None
}
