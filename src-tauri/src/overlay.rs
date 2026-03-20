use crate::capture::PhysicalRect;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::w;
use std::cell::Cell;

/// 오버레이 결과: 사용자가 선택한 영역 또는 취소
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OverlayResult {
    Selected(PhysicalRect),
    Cancelled,
    TooSmall,
}

/// 드래그 좌표로부터 OverlayResult 결정
pub fn evaluate_selection(x1: i32, y1: i32, x2: i32, y2: i32, dpi_scale: f64) -> OverlayResult {
    use crate::capture::{normalize_rect, logical_to_physical, is_valid_selection};

    let (lx, ly, lw, lh) = normalize_rect(x1, y1, x2, y2);
    let physical = logical_to_physical(lx, ly, lw, lh, dpi_scale);

    if !is_valid_selection(physical.width, physical.height) {
        return OverlayResult::TooSmall;
    }

    OverlayResult::Selected(physical)
}

/// 마우스 커서 위치의 모니터 정보 (위치, 크기, DPI 스케일)
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub dpi_scale: f64,
}

/// 마우스 커서가 있는 모니터 정보 가져오기
pub fn get_current_monitor() -> Result<MonitorInfo, String> {
    unsafe {
        let mut cursor_pos = POINT::default();
        GetCursorPos(&mut cursor_pos).map_err(|e| e.message().to_string())?;

        let hmonitor = MonitorFromPoint(cursor_pos, MONITOR_DEFAULTTOPRIMARY);
        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(hmonitor, &mut mi).as_bool() {
            return Err("GetMonitorInfoW failed".to_string());
        }

        let rc = mi.rcMonitor;

        let mut dpi_x: u32 = 96;
        let mut dpi_y: u32 = 96;
        let _ = GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);
        let dpi_scale = dpi_x as f64 / 96.0;

        Ok(MonitorInfo {
            x: rc.left,
            y: rc.top,
            width: (rc.right - rc.left) as u32,
            height: (rc.bottom - rc.top) as u32,
            dpi_scale,
        })
    }
}

// Thread-local state for the overlay window procedure
thread_local! {
    static DRAG_START: Cell<Option<(i32, i32)>> = const { Cell::new(None) };
    static DRAG_CURRENT: Cell<Option<(i32, i32)>> = const { Cell::new(None) };
    static OVERLAY_RESULT: Cell<Option<OverlayResult>> = const { Cell::new(None) };
    static DPI_SCALE: Cell<f64> = const { Cell::new(1.0) };
    static MONITOR_OFFSET: Cell<(i32, i32)> = const { Cell::new((0, 0)) };
}

unsafe extern "system" fn overlay_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_LBUTTONDOWN => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
            DRAG_START.set(Some((x, y)));
            DRAG_CURRENT.set(Some((x, y)));
            unsafe { SetCapture(hwnd) };
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            if DRAG_START.get().is_some() {
                let x = (lparam.0 & 0xFFFF) as i16 as i32;
                let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
                DRAG_CURRENT.set(Some((x, y)));
                let _ = unsafe { InvalidateRect(Some(hwnd), None, true) };
            }
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            if let Some((sx, sy)) = DRAG_START.get() {
                let x = (lparam.0 & 0xFFFF) as i16 as i32;
                let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
                let _ = unsafe { ReleaseCapture() };

                let (ox, oy) = MONITOR_OFFSET.get();
                // 오버레이 창이 Per-Monitor DPI Aware 컨텍스트에서 동작하므로
                // lparam 좌표는 이미 물리 픽셀 기준. DPI 스케일 적용 불필요.
                let result = evaluate_selection(
                    sx + ox, sy + oy,
                    x + ox, y + oy,
                    1.0,
                );
                OVERLAY_RESULT.set(Some(result));
                DRAG_START.set(None);
                DRAG_CURRENT.set(None);
                unsafe { PostQuitMessage(0) };
            }
            LRESULT(0)
        }
        WM_KEYDOWN => {
            if wparam.0 == VK_ESCAPE.0 as usize {
                OVERLAY_RESULT.set(Some(OverlayResult::Cancelled));
                DRAG_START.set(None);
                DRAG_CURRENT.set(None);
                unsafe { PostQuitMessage(0) };
            }
            LRESULT(0)
        }
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = unsafe { BeginPaint(hwnd, &mut ps) };

            // Fill entire window with semi-transparent dark overlay
            let mut client_rect = RECT::default();
            let _ = unsafe { GetClientRect(hwnd, &mut client_rect) };

            let dark_brush = unsafe { CreateSolidBrush(COLORREF(0x00000000)) };
            unsafe { FillRect(hdc, &client_rect, dark_brush) };
            let _ = unsafe { DeleteObject(dark_brush.into()) };

            // If dragging, clear the selection area (draw white rect to show "selected" area)
            if let (Some((sx, sy)), Some((cx, cy))) = (DRAG_START.get(), DRAG_CURRENT.get()) {
                let sel_rect = RECT {
                    left: sx.min(cx),
                    top: sy.min(cy),
                    right: sx.max(cx),
                    bottom: sy.max(cy),
                };
                // Draw selection area with a lighter brush to indicate selection
                let sel_brush = unsafe { CreateSolidBrush(COLORREF(0x00404040)) };
                unsafe { FillRect(hdc, &sel_rect, sel_brush) };
                let _ = unsafe { DeleteObject(sel_brush.into()) };

                // Draw border around selection
                let border_brush = unsafe { CreateSolidBrush(COLORREF(0x0000FF00)) };
                // Top border
                let top = RECT { left: sel_rect.left, top: sel_rect.top, right: sel_rect.right, bottom: sel_rect.top + 2 };
                unsafe { FillRect(hdc, &top, border_brush) };
                // Bottom border
                let bottom = RECT { left: sel_rect.left, top: sel_rect.bottom - 2, right: sel_rect.right, bottom: sel_rect.bottom };
                unsafe { FillRect(hdc, &bottom, border_brush) };
                // Left border
                let left = RECT { left: sel_rect.left, top: sel_rect.top, right: sel_rect.left + 2, bottom: sel_rect.bottom };
                unsafe { FillRect(hdc, &left, border_brush) };
                // Right border
                let right = RECT { left: sel_rect.right - 2, top: sel_rect.top, right: sel_rect.right, bottom: sel_rect.bottom };
                unsafe { FillRect(hdc, &right, border_brush) };
                let _ = unsafe { DeleteObject(border_brush.into()) };
            }

            let _ = unsafe { EndPaint(hwnd, &ps) };
            LRESULT(0)
        }
        WM_ERASEBKGND => {
            LRESULT(1) // Prevent flickering
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

/// 오버레이 창을 표시하고 사용자 선택을 대기 (blocking)
pub fn show_overlay(monitor: &MonitorInfo) -> OverlayResult {
    // Reset thread-local state
    DRAG_START.set(None);
    DRAG_CURRENT.set(None);
    OVERLAY_RESULT.set(None);
    DPI_SCALE.set(monitor.dpi_scale);
    MONITOR_OFFSET.set((monitor.x, monitor.y));

    unsafe {
        let hinstance = GetModuleHandleW(None).unwrap_or_default();
        let hinstance_val = HINSTANCE(hinstance.0);

        let class_name = w!("TextSniperOverlay");

        let cursor = LoadCursorW(None, IDC_CROSS).unwrap_or_default();

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(overlay_wndproc),
            hInstance: hinstance_val,
            hCursor: cursor,
            hbrBackground: HBRUSH::default(),
            lpszClassName: class_name,
            ..Default::default()
        };

        RegisterClassExW(&wc);

        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            class_name,
            w!("TextSniper Overlay"),
            WS_POPUP,
            monitor.x,
            monitor.y,
            monitor.width as i32,
            monitor.height as i32,
            None,
            None,
            Some(hinstance_val),
            None,
        );

        let hwnd = match hwnd {
            Ok(h) => h,
            Err(_) => {
                let _ = UnregisterClassW(class_name, Some(hinstance_val));
                return OverlayResult::Cancelled;
            }
        };

        // Set semi-transparent (alpha ~40% = 100/255)
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 100, LWA_ALPHA);

        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);

        // Message loop
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let _ = DestroyWindow(hwnd);
        let _ = UnregisterClassW(class_name, Some(hinstance_val));

        OVERLAY_RESULT.get().unwrap_or(OverlayResult::Cancelled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_selection_normal() {
        let result = evaluate_selection(100, 100, 250, 300, 1.0);
        assert_eq!(result, OverlayResult::Selected(PhysicalRect {
            x: 100, y: 100, width: 150, height: 200,
        }));
    }

    #[test]
    fn test_evaluate_selection_reverse_drag() {
        let result = evaluate_selection(250, 300, 100, 100, 1.0);
        assert_eq!(result, OverlayResult::Selected(PhysicalRect {
            x: 100, y: 100, width: 150, height: 200,
        }));
    }

    #[test]
    fn test_evaluate_selection_too_small() {
        let result = evaluate_selection(100, 100, 105, 105, 1.0);
        assert_eq!(result, OverlayResult::TooSmall);
    }

    #[test]
    fn test_evaluate_selection_with_dpi_150() {
        let result = evaluate_selection(100, 100, 200, 200, 1.5);
        assert_eq!(result, OverlayResult::Selected(PhysicalRect {
            x: 150, y: 150, width: 150, height: 150,
        }));
    }

    #[test]
    fn test_evaluate_selection_too_small_after_dpi() {
        let result = evaluate_selection(100, 100, 105, 105, 1.5);
        // 물리: 7.5x7.5 -> 7x7 -> too small
        assert_eq!(result, OverlayResult::TooSmall);
    }
}
