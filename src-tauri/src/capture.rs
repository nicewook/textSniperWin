use windows::Win32::Graphics::Gdi::*;

/// 물리 픽셀 기준 사각형 영역
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicalRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// 논리 좌표 -> 물리 좌표 변환
pub fn logical_to_physical(x: i32, y: i32, w: u32, h: u32, scale: f64) -> PhysicalRect {
    PhysicalRect {
        x: (x as f64 * scale) as i32,
        y: (y as f64 * scale) as i32,
        width: (w as f64 * scale) as u32,
        height: (h as f64 * scale) as u32,
    }
}

/// 역방향 드래그 정규화: (start, end) -> (top_left, size)
pub fn normalize_rect(x1: i32, y1: i32, x2: i32, y2: i32) -> (i32, i32, u32, u32) {
    let left = x1.min(x2);
    let top = y1.min(y2);
    let right = x1.max(x2);
    let bottom = y1.max(y2);
    (left, top, (right - left) as u32, (bottom - top) as u32)
}

/// 최소 영역 검증 (10x10 미만이면 false)
pub fn is_valid_selection(width: u32, height: u32) -> bool {
    width >= 10 && height >= 10
}

/// BitBlt로 화면 캡처, RGBA 바이트 반환
pub fn capture_screen_region(rect: &PhysicalRect) -> Result<Vec<u8>, String> {
    unsafe {
        let hdc_screen = GetDC(None);
        if hdc_screen.is_invalid() {
            return Err("GetDC failed".to_string());
        }

        let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
        let hbm = CreateCompatibleBitmap(hdc_screen, rect.width as i32, rect.height as i32);
        let old = SelectObject(hdc_mem, hbm.into());

        let blt_result = BitBlt(
            hdc_mem,
            0, 0,
            rect.width as i32, rect.height as i32,
            Some(hdc_screen),
            rect.x, rect.y,
            SRCCOPY,
        );

        if blt_result.is_err() {
            SelectObject(hdc_mem, old);
            let _ = DeleteDC(hdc_mem);
            let _ = DeleteObject(hbm.into());
            ReleaseDC(None, hdc_screen);
            return Err("BitBlt failed".to_string());
        }

        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: rect.width as i32,
                biHeight: -(rect.height as i32), // top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let buf_size = (rect.width * rect.height * 4) as usize;
        let mut buffer: Vec<u8> = vec![0u8; buf_size];

        GetDIBits(
            hdc_mem,
            hbm,
            0,
            rect.height,
            Some(buffer.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        // BGRA -> RGBA 변환
        for chunk in buffer.chunks_exact_mut(4) {
            chunk.swap(0, 2);
        }

        SelectObject(hdc_mem, old);
        let _ = DeleteDC(hdc_mem);
        let _ = DeleteObject(hbm.into());
        ReleaseDC(None, hdc_screen);

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logical_to_physical_100_percent() {
        let r = logical_to_physical(100, 200, 300, 400, 1.0);
        assert_eq!(r, PhysicalRect { x: 100, y: 200, width: 300, height: 400 });
    }

    #[test]
    fn test_logical_to_physical_150_percent() {
        let r = logical_to_physical(100, 200, 300, 400, 1.5);
        assert_eq!(r, PhysicalRect { x: 150, y: 300, width: 450, height: 600 });
    }

    #[test]
    fn test_logical_to_physical_125_percent() {
        let r = logical_to_physical(100, 100, 200, 200, 1.25);
        assert_eq!(r, PhysicalRect { x: 125, y: 125, width: 250, height: 250 });
    }

    #[test]
    fn test_normalize_rect_normal_direction() {
        let (x, y, w, h) = normalize_rect(10, 20, 110, 120);
        assert_eq!((x, y, w, h), (10, 20, 100, 100));
    }

    #[test]
    fn test_normalize_rect_reverse_direction() {
        let (x, y, w, h) = normalize_rect(110, 120, 10, 20);
        assert_eq!((x, y, w, h), (10, 20, 100, 100));
    }

    #[test]
    fn test_normalize_rect_partial_reverse() {
        let (x, y, w, h) = normalize_rect(200, 50, 100, 150);
        assert_eq!((x, y, w, h), (100, 50, 100, 100));
    }

    #[test]
    fn test_is_valid_selection_valid() {
        assert!(is_valid_selection(10, 10));
        assert!(is_valid_selection(100, 50));
    }

    #[test]
    fn test_is_valid_selection_too_small() {
        assert!(!is_valid_selection(9, 9));
        assert!(!is_valid_selection(5, 100));
        assert!(!is_valid_selection(100, 5));
        assert!(!is_valid_selection(0, 0));
    }

    #[test]
    fn test_capture_screen_region_returns_data() {
        let rect = PhysicalRect { x: 0, y: 0, width: 100, height: 100 };
        let result = capture_screen_region(&rect);
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.len(), 100 * 100 * 4);
    }
}
