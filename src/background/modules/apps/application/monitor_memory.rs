//! Event-driven, local app-to-monitor memory. No Windows startup or scheduled job.
use std::{
    collections::HashMap,
    sync::{LazyLock, mpsc},
    time::{Duration, Instant},
};

use windows::Win32::{
    Foundation::RECT,
    UI::WindowsAndMessaging::{
        GetWindowPlacement, SW_RESTORE, SW_SHOWMAXIMIZED, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE,
        SWP_NOZORDER, WINDOWPLACEMENT,
    },
};

use crate::{
    error::Result,
    utils::{atomic_write_file, constants::SEELEN_COMMON, spawn_named_thread},
    windows_api::{Com, MonitorEnumerator, WindowsApi, window::Window},
};

use super::monitor_geometry::relocate;

enum Event {
    Added(isize, u32),
    Observed(isize, u32, bool),
    Removed(isize),
}

static EVENTS: LazyLock<mpsc::Sender<Event>> = LazyLock::new(|| {
    let (sender, receiver) = mpsc::channel();
    spawn_named_thread("MonitorMemory", move || {
        if let Err(err) = Com::run_with_context(|| {
            run(receiver);
            Ok(())
        }) {
            log::error!("Could not initialize monitor memory: {err}");
        }
    });
    sender
});

pub(super) fn added(window: &Window) {
    let _ = EVENTS.send(Event::Added(window.address(), window.process().id()));
}

pub(super) fn observed(window: &Window, deliberate_move: bool) {
    if deliberate_move || window.is_focused() {
        let _ = EVENTS.send(Event::Observed(
            window.address(),
            window.process().id(),
            deliberate_move,
        ));
    }
}

pub(super) fn removed(window: &Window) {
    let _ = EVENTS.send(Event::Removed(window.address()));
}

fn app_key(window: &Window) -> Option<String> {
    window
        .app_user_model_id()
        .map(|id| format!("umid:{id}"))
        .or_else(|| {
            window
                .process()
                .program_path()
                .ok()
                .map(|path| format!("exe:{}", path.to_string_lossy().to_lowercase()))
        })
}

fn run(receiver: mpsc::Receiver<Event>) {
    let path = SEELEN_COMMON.app_data_dir().join("last-monitors.json");
    let mut saved: HashMap<String, String> = match std::fs::read(&path) {
        Ok(bytes) => match serde_json::from_slice(&bytes) {
            Ok(saved) => saved,
            Err(_) => {
                log::error!(
                    "Monitor memory file is invalid; leaving it untouched and disabling memory."
                );
                return;
            }
        },
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => HashMap::new(),
        Err(_) => {
            log::error!("Cannot read monitor memory; disabling memory.");
            return;
        }
    };
    let mut pending: HashMap<isize, (u32, Instant)> = HashMap::new();
    let mut dirty = false;
    let mut last_write = Instant::now();
    loop {
        match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(Event::Added(hwnd, pid)) => {
                pending.insert(hwnd, (pid, Instant::now() + Duration::from_millis(800)));
            }
            Ok(Event::Removed(hwnd)) => {
                pending.remove(&hwnd);
            }
            Ok(Event::Observed(hwnd, pid, deliberate_move)) => {
                let window = Window::from(hwnd);
                if deliberate_move {
                    pending.remove(&hwnd);
                }
                if !pending.contains_key(&hwnd)
                    && window.is_window()
                    && window.process().id() == pid
                    && !window.is_minimized()
                    && !window.is_fullscreen()
                    && let Some(key) = app_key(&window)
                    && let Ok(monitor_id) = window.monitor().stable_id()
                {
                    let current = monitor_id.0;
                    // An unplugged display must not erase its saved assignment on fallback focus.
                    let can_replace = deliberate_move
                        || saved.get(&key).is_none_or(|previous| {
                            previous == &current || monitor_is_connected(previous)
                        });
                    if current != "null" && can_replace && saved.get(&key) != Some(&current) {
                        saved.insert(key, current);
                        dirty = true;
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }

        let due: Vec<_> = pending
            .iter()
            .filter(|(_, (_, at))| *at <= Instant::now())
            .map(|(&hwnd, &(pid, _))| (hwnd, pid))
            .collect();
        for (hwnd, pid) in due {
            let window = Window::from(hwnd);
            if window.is_window() && window.process().id() == pid {
                if let Some(key) = app_key(&window) {
                    if let Some(target) = saved.get(&key) {
                        if let Err(err) = restore(window, target) {
                            log::debug!("Monitor restore skipped: {err}");
                        }
                    } else if !window.is_minimized()
                        && !window.is_fullscreen()
                        && let Ok(monitor_id) = window.monitor().stable_id()
                        && monitor_id.0 != "null"
                    {
                        saved.insert(key, monitor_id.0);
                        dirty = true;
                    }
                }
            }
            pending.remove(&hwnd);
        }
        if dirty && last_write.elapsed() >= Duration::from_secs(1) {
            match serde_json::to_vec_pretty(&saved)
                .map_err(Into::into)
                .and_then(|bytes| atomic_write_file(&path, &bytes))
            {
                Ok(()) => dirty = false,
                Err(_) => log::error!("Could not save monitor memory."),
            }
            last_write = Instant::now();
        }
    }
}

fn monitor_is_connected(id: &str) -> bool {
    MonitorEnumerator::enumerate_win32()
        .unwrap_or_default()
        .iter()
        .any(|monitor| monitor.stable_id().is_ok_and(|value| value.0 == id))
}

fn restore(window: Window, id: &str) -> Result<()> {
    if crate::state::application::FULL_STATE
        .load()
        .settings
        .by_widget
        .wm
        .enabled
    {
        return Ok(());
    }
    if window.is_minimized()
        || window.is_fullscreen()
        || window.is_dragging()
        || !window.is_visible()
    {
        return Ok(());
    }
    // Explicit user rules take priority over automatic memory.
    if window
        .get_app_config()?
        .is_some_and(|config| config.bound_monitor.is_some())
    {
        return Ok(());
    }
    let source = window.monitor();
    if source.stable_id()?.0 == id {
        return Ok(());
    }
    let Some(target) = MonitorEnumerator::enumerate_win32()?
        .into_iter()
        .find(|monitor| monitor.stable_id().is_ok_and(|value| value.0 == id))
    else {
        return Ok(());
    };
    let source_info = source.info()?.monitorInfo;
    let target_info = target.info()?.monitorInfo;
    if target_info.rcWork.right <= target_info.rcWork.left
        || target_info.rcWork.bottom <= target_info.rcWork.top
    {
        return Ok(());
    }
    let mut placement = WINDOWPLACEMENT {
        length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
        ..Default::default()
    };
    unsafe {
        GetWindowPlacement(window.hwnd(), &mut placement)?;
    }
    let mut rect = placement.rcNormalPosition;
    // WINDOWPLACEMENT uses workspace coordinates for ordinary top-level app windows.
    rect.left += source_info.rcWork.left - source_info.rcMonitor.left;
    rect.right += source_info.rcWork.left - source_info.rcMonitor.left;
    rect.top += source_info.rcWork.top - source_info.rcMonitor.top;
    rect.bottom += source_info.rcWork.top - source_info.rcMonitor.top;
    let array = |r: RECT| [r.left, r.top, r.right, r.bottom];
    let [left, top, right, bottom] = relocate(
        array(rect),
        array(source_info.rcWork),
        array(target_info.rcWork),
        WindowsApi::get_monitor_scale_factor(target.handle())?
            / WindowsApi::get_monitor_scale_factor(source.handle())?,
    );
    let moved = RECT {
        left,
        top,
        right,
        bottom,
    };
    let was_maximized = window.is_maximized();
    if was_maximized {
        // Changing rcNormalPosition alone does not move a currently maximized window.
        // Queue restore, move, and maximize on its owning thread, in that order.
        window.show_window_async(SW_RESTORE)?;
    }
    window.set_position(&moved, SWP_NOACTIVATE | SWP_NOZORDER | SWP_ASYNCWINDOWPOS)?;
    if was_maximized {
        window.show_window_async(SW_SHOWMAXIMIZED)?;
    }
    Ok(())
}
