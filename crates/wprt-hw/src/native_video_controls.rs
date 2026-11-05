// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted port of pristine platform/video.cpp bound_key/side_key/note_press/note_key/window_proc.
// Integrated source;runtime acceptance tracked in HANDOFF; native Win32/menu/settings/Bindings/LiveFlag leaves retained.
// Exact SDK constants below avoid changing shared core dependency features for these native-leaf calls.
use std::ffi::CStr;
const HTCLIENT:u32=1u32;
// Values extracted from pinned official windows-sys0.61.2 UI SDK bindings;Win32 calls remain native leaves.
const VK_CONTROL:u16=17u16;
const VK_F10:u16=121u16;
const VK_F11:u16=122u16;
const VK_F12:u16=123u16;
const VK_F4:u16=115u16;
const VK_LBUTTON:u16=1u16;
const VK_LCONTROL:u16=162u16;
const VK_LMENU:u16=164u16;
const VK_LSHIFT:u16=160u16;
const VK_MBUTTON:u16=4u16;
const VK_MENU:u16=18u16;
const VK_RBUTTON:u16=2u16;
const VK_RCONTROL:u16=163u16;
const VK_RMENU:u16=165u16;
const VK_RSHIFT:u16=161u16;
const VK_SHIFT:u16=16u16;
const VK_XBUTTON1:u16=5u16;
const VK_XBUTTON2:u16=6u16;
const WM_APP:u32=32768u32;
const WM_CLOSE:u32=16u32;
const WM_DESTROY:u32=2u32;
const WM_ERASEBKGND:u32=20u32;
const WM_KEYDOWN:u32=256u32;
const WM_LBUTTONDOWN:u32=513u32;
const WM_MBUTTONDOWN:u32=519u32;
const WM_MOUSEMOVE:u32=512u32;
const WM_MOUSEWHEEL:u32=522u32;
const WM_PAINT:u32=15u32;
const WM_RBUTTONDOWN:u32=516u32;
const WM_SETCURSOR:u32=32u32;
const WM_SETTINGCHANGE:u32=26u32;
const WM_SIZE:u32=5u32;
const WM_SYSKEYDOWN:u32=260u32;
const WM_SYSKEYUP:u32=261u32;
const WM_TIMER:u32=275u32;
const WM_XBUTTONDOWN:u32=523u32;
const XBUTTON1:u16=1u16;

extern "C" {
    fn wphw_vc_bound(code: i32) -> bool;
    fn wphw_vc_screenshot_binding(code: i32) -> bool;
    fn wphw_vc_request_screenshot();
    fn wphw_vc_map_shift(scan: u32) -> i32;
    fn wphw_vc_exit() -> !;
    fn wphw_vc_title(window: usize);
    fn wphw_vc_capture();
    fn wphw_vc_fullscreen() -> bool;
    fn wphw_vc_set_fullscreen(window: usize, enabled: bool);
    fn wphw_vc_store_fullscreen();
    fn wphw_vc_refresh_menu(window: usize);
    fn wphw_vc_menu_open() -> bool;
    fn wphw_vc_menu_key(window: usize, key: u32, repeat: bool);
    fn wphw_vc_toggle_menu(window: usize);
    fn wphw_vc_poll_gamepads(window: usize);
    fn wphw_vc_menu_click(window: usize, x: i32, y: i32, right: bool);
    fn wphw_vc_menu_hover(window: usize, x: i32, y: i32);
    fn wphw_vc_hide_cursor() -> bool;
    fn wphw_vc_set_null_cursor();
    fn wphw_vc_menu_wheel(window: usize, delta: i32);
    fn wphw_vc_note_wheel(delta: i32);
    fn wphw_vc_wheel_key(up: bool) -> i32;
    fn wphw_vc_theme(window: usize);
    fn wphw_vc_paint(window: usize);
    fn wphw_vc_default(window: usize, message: u32, wparam: usize, lparam: isize) -> isize;
}
// port of video.cpp bound_key. Do not truncate WPARAM before source switch comparisons.
unsafe fn bound_key(key: usize) -> bool {
    let code = key as i32;
    if key == VK_SHIFT as usize {
        wphw_vc_bound(code) || wphw_vc_bound(VK_LSHIFT as i32) || wphw_vc_bound(VK_RSHIFT as i32)
    } else if key == VK_CONTROL as usize {
        wphw_vc_bound(code)
            || wphw_vc_bound(VK_LCONTROL as i32)
            || wphw_vc_bound(VK_RCONTROL as i32)
    } else if key == VK_MENU as usize {
        wphw_vc_bound(code) || wphw_vc_bound(VK_LMENU as i32) || wphw_vc_bound(VK_RMENU as i32)
    } else {
        wphw_vc_bound(code)
    }
}
// port of video.cpp side_key. Native MapVirtualKeyA preserves the source MAPVK_VSC_TO_VK_EX call.
unsafe fn side_key(key: usize, lparam: isize) -> i32 {
    let extended = lparam & (1 << 24) != 0;
    if key == VK_SHIFT as usize {
        wphw_vc_map_shift(((lparam >> 16) & 0xff) as u32)
    } else if key == VK_CONTROL as usize {
        if extended {
            VK_RCONTROL as i32
        } else {
            VK_LCONTROL as i32
        }
    } else if key == VK_MENU as usize {
        if extended {
            VK_RMENU as i32
        } else {
            VK_LMENU as i32
        }
    } else {
        key as i32
    }
}
// port of video.cpp note_press/note_key.
unsafe fn note_press(code: i32, side: i32) {
    if wphw_vc_screenshot_binding(code) || (side != 0 && wphw_vc_screenshot_binding(side)) {
        wphw_vc_request_screenshot();
    }
}
unsafe fn note_key(key: usize, lparam: isize) {
    if lparam & (1 << 30) == 0 {
        note_press(key as i32, side_key(key, lparam));
    }
}
// Native windowsx GET_* macros sign-extend the low/high SHORT mouse coordinates and wheel delta.
fn mouse_x(lparam: isize) -> i32 {
    (lparam as u16 as i16) as i32
}
fn mouse_y(lparam: isize) -> i32 {
    ((lparam >> 16) as u16 as i16) as i32
}
// port of video.cpp window_proc. Every source handled message/early return and default call is retained.
#[no_mangle]
pub unsafe extern "C" fn wprt_video_window_proc(
    window: usize,
    message: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    match message {
        WM_CLOSE | WM_DESTROY => wphw_vc_exit(),
        m if m == WM_APP + 1 => {
            wphw_vc_title(window);
            0
        }
        WM_KEYDOWN => {
            let repeat = lparam & (1 << 30) != 0;
            if wparam == VK_F12 as usize && !repeat {
                wphw_vc_capture();
                return 0;
            }
            if wparam == VK_F11 as usize && !repeat {
                let enabled = !wphw_vc_fullscreen();
                wphw_vc_set_fullscreen(window, enabled);
                wphw_vc_store_fullscreen();
                wphw_vc_refresh_menu(window);
                return 0;
            }
            if wphw_vc_menu_open() {
                wphw_vc_menu_key(window, wparam as u32, repeat);
                return 0;
            }
            note_key(wparam, lparam);
            wphw_vc_default(window, message, wparam, lparam)
        }
        WM_SYSKEYDOWN | WM_SYSKEYUP => {
            if wparam == VK_F10 as usize {
                if message == WM_SYSKEYDOWN && lparam & (1 << 30) == 0 {
                    wphw_vc_toggle_menu(window);
                }
                return 0;
            }
            if message == WM_SYSKEYDOWN && !wphw_vc_menu_open() {
                note_key(wparam, lparam);
            }
            if wparam != VK_F4 as usize && bound_key(wparam) {
                return 0;
            }
            wphw_vc_default(window, message, wparam, lparam)
        }
        WM_TIMER => {
            if wparam == 1 {
                wphw_vc_poll_gamepads(window);
                return 0;
            }
            wphw_vc_default(window, message, wparam, lparam)
        }
        WM_LBUTTONDOWN | WM_RBUTTONDOWN => {
            if wphw_vc_menu_open() {
                wphw_vc_menu_click(
                    window,
                    mouse_x(lparam),
                    mouse_y(lparam),
                    message == WM_RBUTTONDOWN,
                );
                return 0;
            }
            note_press(
                if message == WM_LBUTTONDOWN {
                    VK_LBUTTON as i32
                } else {
                    VK_RBUTTON as i32
                },
                0,
            );
            wphw_vc_default(window, message, wparam, lparam)
        }
        WM_MOUSEMOVE => {
            if wphw_vc_menu_open() {
                wphw_vc_menu_hover(window, mouse_x(lparam), mouse_y(lparam));
                return 0;
            }
            wphw_vc_default(window, message, wparam, lparam)
        }
        WM_MBUTTONDOWN => {
            if !wphw_vc_menu_open() {
                note_press(VK_MBUTTON as i32, 0);
            }
            wphw_vc_default(window, message, wparam, lparam)
        }
        WM_XBUTTONDOWN => {
            if !wphw_vc_menu_open() {
                note_press(
                    if ((wparam >> 16) & 0xffff) == XBUTTON1 as usize {
                        VK_XBUTTON1 as i32
                    } else {
                        VK_XBUTTON2 as i32
                    },
                    0,
                );
            }
            1
        }
        WM_SIZE => {
            wphw_vc_refresh_menu(window);
            wphw_vc_default(window, message, wparam, lparam)
        }
        WM_SETCURSOR => {
            if (lparam as u16) as u32 == HTCLIENT && wphw_vc_hide_cursor() && !wphw_vc_menu_open() {
                wphw_vc_set_null_cursor();
                return 1;
            }
            wphw_vc_default(window, message, wparam, lparam)
        }
        WM_MOUSEWHEEL => {
            let delta = ((wparam >> 16) as u16 as i16) as i32;
            if wphw_vc_menu_open() {
                wphw_vc_menu_wheel(window, delta);
                return 0;
            }
            wphw_vc_note_wheel(delta);
            if delta != 0 {
                note_press(wphw_vc_wheel_key(delta > 0), 0);
            }
            0
        }
        WM_SETTINGCHANGE => {
            if lparam != 0
                && CStr::from_ptr(lparam as *const std::ffi::c_char).to_bytes()
                    == b"ImmersiveColorSet"
            {
                wphw_vc_theme(window);
            }
            wphw_vc_default(window, message, wparam, lparam)
        }
        WM_ERASEBKGND => 1,
        WM_PAINT => {
            wphw_vc_paint(window);
            0
        }
        _ => wphw_vc_default(window, message, wparam, lparam),
    }
}
