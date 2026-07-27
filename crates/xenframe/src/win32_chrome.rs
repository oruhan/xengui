// SPDX-License-Identifier: Apache-2.0
#![cfg(target_os = "windows")]

use std::sync::Arc;
use raw_window_handle::{ HasWindowHandle, RawWindowHandle };
use winit::window::Window;
use windows_sys::Win32::Foundation::{ HWND, LPARAM, LRESULT, RECT, WPARAM };
use windows_sys::Win32::UI::Shell::{ DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass };
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetWindowRect,
    HTBOTTOM,
    HTBOTTOMLEFT,
    HTBOTTOMRIGHT,
    HTCLIENT,
    HTLEFT,
    HTRIGHT,
    HTTOP,
    HTTOPLEFT,
    HTTOPRIGHT,
    IsZoomed,
    KillTimer,
    NCCALCSIZE_PARAMS,
    SWP_FRAMECHANGED,
    SWP_NOACTIVATE,
    SWP_NOMOVE,
    SWP_NOSIZE,
    SWP_NOZORDER,
    SetTimer,
    SetWindowPos,
    WINDOWPOS,
    WM_DESTROY,
    WM_ENTERSIZEMOVE,
    WM_ERASEBKGND,
    WM_EXITSIZEMOVE,
    WM_NCCALCSIZE,
    WM_NCHITTEST,
    WM_PAINT,
    WM_SIZE,
    WM_TIMER,
    WM_WINDOWPOSCHANGED,
    WVR_REDRAW,
};
use windows_sys::Win32::Graphics::Dwm::{
    DWMWA_TRANSITIONS_FORCEDISABLED,
    DWMWA_USE_IMMERSIVE_DARK_MODE,
    DWMWA_WINDOW_CORNER_PREFERENCE,
    DWMWCP_ROUND,
    DwmExtendFrameIntoClientArea,
    DwmSetWindowAttribute,
};
use windows_sys::Win32::UI::Controls::MARGINS;
use windows_sys::Win32::Graphics::Dwm::DwmFlush;

const SUBCLASS_ID: usize = 1;
const RESIZE_TIMER_ID: usize = 1;
// ~120Hz cap: fast enough to feel responsive during a drag, without
// flooding the GPU with more presents than a modal WM_SIZE loop needs.
const RESIZE_TIMER_INTERVAL_MS: u32 = 8;

thread_local! {
    static RESIZE_TICK: std::cell::RefCell<Option<Box<dyn FnMut()>>> = const {
        std::cell::RefCell::new(None)
    };
}

/// Registers the closure called on every `WM_TIMER` tick while a modal
/// resize/move loop is active. The caller (xenframe::App) should capture
/// whatever it needs to re-measure the client area and repaint - Win32
/// delivers `WM_SIZE` synchronously inside that loop, so without this
/// timer nothing else in the app gets a chance to run until the drag ends.
pub fn set_resize_tick_callback(f: impl FnMut() + 'static) {
    RESIZE_TICK.with(|cell| {
        *cell.borrow_mut() = Some(Box::new(f));
    });
}

pub fn flush_dwm() {
    unsafe {
        DwmFlush();
    }
}

unsafe extern "system" fn custom_chrome_subclass(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _uidsubclass: usize,
    _dwrefdata: usize
) -> LRESULT {
    match msg {
        WM_NCCALCSIZE if wparam != 0 => {
            let params = unsafe { &mut *(lparam as *mut NCCALCSIZE_PARAMS) };
            if (unsafe { IsZoomed(hwnd) }) != 0 {
                let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                unsafe {
                    GetWindowRect(hwnd, &mut rect);
                }
                params.rgrc[0] = rect;
                return WVR_REDRAW as LRESULT;
            }
        }
        WM_NCHITTEST => {
            // Call default proc first to let OS evaluate base hit areas
            let hit = unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
            if hit != (HTCLIENT as LRESULT) {
                return hit;
            }

            let x = (lparam & 0xffff) as i16 as i32;
            let y = ((lparam >> 16) & 0xffff) as i16 as i32;

            let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            unsafe {
                GetWindowRect(hwnd, &mut rect);
            }

            let border_width = 8;
            let left = x < rect.left + border_width;
            let right = x >= rect.right - border_width;
            let top = y < rect.top + border_width;
            let bottom = y >= rect.bottom - border_width;

            let custom_hit = match (left, right, top, bottom) {
                (true, _, true, _) => HTTOPLEFT,
                (_, true, true, _) => HTTOPRIGHT,
                (true, _, _, true) => HTBOTTOMLEFT,
                (_, true, _, true) => HTBOTTOMRIGHT,
                (true, _, _, _) => HTLEFT,
                (_, true, _, _) => HTRIGHT,
                (_, _, true, _) => HTTOP,
                (_, _, _, true) => HTBOTTOM,
                _ => HTCLIENT,
            };

            if custom_hit != HTCLIENT {
                return custom_hit as LRESULT;
            }
        }
        WM_ENTERSIZEMOVE => {
            xengui::devtools::clear();
            xengui::devtools::record("WM_ENTERSIZEMOVE");
            let disable: i32 = 1;
            unsafe {
                DwmSetWindowAttribute(
                    hwnd,
                    DWMWA_TRANSITIONS_FORCEDISABLED as u32,
                    &disable as *const _ as *const _,
                    std::mem::size_of_val(&disable) as u32
                );
                SetTimer(hwnd, RESIZE_TIMER_ID, RESIZE_TIMER_INTERVAL_MS, None);
            }
        }
        WM_TIMER if wparam == RESIZE_TIMER_ID => {
            let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            unsafe {
                GetWindowRect(hwnd, &mut rect);
            }
            xengui::devtools::record_size(
                "WM_TIMER",
                (rect.right - rect.left).max(0) as u32,
                (rect.bottom - rect.top).max(0) as u32
            );
            RESIZE_TICK.with(|cell| {
                if let Some(tick) = cell.borrow_mut().as_mut() {
                    tick();
                }
            });
            return 0;
        }
        WM_EXITSIZEMOVE => {
            unsafe {
                KillTimer(hwnd, RESIZE_TIMER_ID);
                let disable: i32 = 0;
                DwmSetWindowAttribute(
                    hwnd,
                    DWMWA_TRANSITIONS_FORCEDISABLED as u32,
                    &disable as *const _ as *const _,
                    std::mem::size_of_val(&disable) as u32
                );
            }
            xengui::devtools::record("WM_EXITSIZEMOVE");
            xengui::devtools::dump("resize gesture ended");
        }
        WM_WINDOWPOSCHANGED => {
            let pos = unsafe { &*(lparam as *const WINDOWPOS) };
            xengui::devtools::record_size_note(
                "WM_WINDOWPOSCHANGED",
                pos.cx.max(0) as u32,
                pos.cy.max(0) as u32,
                format!("at ({}, {})", pos.x, pos.y)
            );
        }
        WM_SIZE => {
            let width = (lparam & 0xffff) as u16 as u32;
            let height = ((lparam >> 16) & 0xffff) as u16 as u32;
            xengui::devtools::record_size("WM_SIZE", width, height);
        }
        WM_ERASEBKGND => {
            xengui::devtools::record("WM_ERASEBKGND");
        }
        WM_PAINT => {
            xengui::devtools::record("WM_PAINT");
        }
        WM_DESTROY => {
            // Remove subclass hook when window is destroyed
            unsafe {
                KillTimer(hwnd, RESIZE_TIMER_ID);
                RemoveWindowSubclass(hwnd, Some(custom_chrome_subclass), SUBCLASS_ID);
            }
        }
        _ => {}
    }

    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

pub fn install_for_window(window: &Arc<Window>) {
    let Ok(handle) = window.window_handle() else {
        return;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return;
    };

    unsafe {
        let hwnd = handle.hwnd.get() as HWND;

        // Force frame recalculation without stripping WS_CAPTION
        SetWindowPos(
            hwnd,
            std::ptr::null_mut(),
            0,
            0,
            0,
            0,
            SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE
        );

        // Extend DWM frame for native shadows
        let margins = MARGINS {
            cxLeftWidth: 1,
            cxRightWidth: 1,
            cyTopHeight: 1,
            cyBottomHeight: 1,
        };
        let _ = DwmExtendFrameIntoClientArea(hwnd, &margins);

        let corner_pref = DWMWCP_ROUND as u32;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE as u32,
            &corner_pref as *const _ as *const _,
            std::mem::size_of_val(&corner_pref) as u32
        );

        let dark: i32 = 1;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE as u32,
            &dark as *const _ as *const _,
            std::mem::size_of_val(&dark) as u32
        );
        // Attach subclassing via comctl32 safely
        SetWindowSubclass(hwnd, Some(custom_chrome_subclass), SUBCLASS_ID, 0);
    }
}
