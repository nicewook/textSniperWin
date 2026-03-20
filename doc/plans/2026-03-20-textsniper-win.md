# TextSniper for Windows - Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Windows 화면 영역을 캡처하여 OCR로 텍스트를 인식하고 클립보드에 복사하는 시스템 트레이 상주 앱

**Architecture:** Tauri v2가 트레이/단축키/OCR/설정을 관리하고, Win32 API로 직접 구현한 오버레이 창이 영역 선택과 화면 캡처를 담당하는 하이브리드 구조. 각 모듈은 trait 기반 인터페이스로 분리하여 테스트 가능.

**Tech Stack:** Rust, Tauri v2, windows crate 0.62, arboard 3, tauri-plugin-global-shortcut 2

**Spec:** `doc/SPEC.md`

---

## File Structure

```
textSniperWin/
├── src-tauri/
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   ├── icons/
│   │   ├── icon.ico              # 기본 트레이 아이콘
│   │   ├── icon-loading.ico      # 처리 중
│   │   ├── icon-success.ico      # 성공
│   │   └── icon-error.ico        # 실패
│   └── src/
│       ├── main.rs               # 진입점 (lib::run 호출)
│       ├── lib.rs                 # Tauri 앱 설정, 모듈 조합
│       ├── capture.rs            # BitBlt 화면 캡처 + DPI 변환
│       ├── ocr.rs                # Windows OCR API 래퍼
│       ├── clipboard.rs          # 클립보드 읽기/쓰기
│       ├── overlay.rs            # Win32 오버레이 창 + 영역 선택
│       ├── tray.rs               # 트레이 아이콘 + 메뉴 + 상태 피드백
│       ├── config.rs             # 설정 저장/로드 (config.json)
│       └── single_instance.rs    # 중복 실행 방지 (Mutex)
├── index.html                    # 빈 HTML (Tauri 필수, 비표시)
└── doc/
    └── SPEC.md
```

**모듈별 책임:**

| 모듈 | 책임 | 외부 의존 |
|------|------|-----------|
| `capture.rs` | 모니터 정보 조회, DPI 변환, BitBlt 캡처 → RGBA 바이트 반환 | `windows` crate |
| `ocr.rs` | RGBA 바이트 → SoftwareBitmap → OcrEngine → 텍스트 문자열 | `windows` crate |
| `clipboard.rs` | 텍스트를 클립보드에 복사 | `arboard` |
| `overlay.rs` | Win32 창 생성, 반투명 렌더링, 마우스 드래그, 선택 영역 반환 | `windows` crate |
| `tray.rs` | 트레이 아이콘 생성/변경, 메뉴, Balloon Tip | `tauri` |
| `config.rs` | config.json 읽기/쓰기, 자동실행 레지스트리 | `serde_json`, `windows` crate |
| `single_instance.rs` | Named Mutex로 중복 실행 감지 | `windows` crate |
| `lib.rs` | 모든 모듈 조합, 핵심 흐름 오케스트레이션 | `tauri` |

---

## Task 1: 프로젝트 스캐폴딩

**Files:**
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/build.rs`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/capabilities/default.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`
- Create: `index.html`

**완료 조건:** `cargo build` 성공, 앱 실행 시 트레이 아이콘만 표시 (창 없음)

- [ ] **Step 1: Tauri v2 프로젝트 생성**

```bash
cd C:/Users/nicew/projects/textSniperWin
cargo create-tauri-app --name text-sniper-win --identifier com.textsniper.win --template vanilla --manager npm
```

- [ ] **Step 2: Cargo.toml 의존성 설정**

`src-tauri/Cargo.toml`의 `[dependencies]` 섹션:

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon", "image-ico", "image-png"] }
tauri-plugin-opener = "2"
tauri-plugin-global-shortcut = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
arboard = "3"
tokio = { version = "1", features = ["time"] }

[target.'cfg(windows)'.dependencies]
windows = { version = "0.62", features = [
    "Media_Ocr",
    "Graphics_Imaging",
    "Storage_Streams",
    "Foundation",
    "Globalization",
    "Win32_Foundation",
    "Win32_Graphics_Gdi",
    "Win32_UI_WindowsAndMessaging",
    "Win32_UI_HiDpi",
    "Win32_System_LibraryLoader",
    "Win32_System_Threading",
    "Win32_System_Registry",
    "Win32_UI_Shell",
] }

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

- [ ] **Step 3: tauri.conf.json 설정 (창 없는 트레이 전용 앱)**

`src-tauri/tauri.conf.json`:

```json
{
  "$schema": "https://raw.githubusercontent.com/nicew/tauri-apps/tauri-v2/crates/tauri-cli/config.schema.json",
  "productName": "TextSniperWin",
  "version": "0.1.0",
  "identifier": "com.textsniper.win",
  "build": {
    "frontendDist": "../",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "",
    "beforeBuildCommand": ""
  },
  "app": {
    "windows": [],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/icon.ico"
    ]
  }
}
```

- [ ] **Step 4: 빈 index.html 생성**

`index.html`:

```html
<!DOCTYPE html>
<html><head><title>TextSniper</title></head><body></body></html>
```

- [ ] **Step 5: capabilities/default.json 권한 설정**

`src-tauri/capabilities/default.json`:

```json
{
  "identifier": "default",
  "description": "default permissions",
  "windows": ["*"],
  "permissions": [
    "global-shortcut:default"
  ]
}
```

- [ ] **Step 6: 최소 main.rs, lib.rs 작성**

`src-tauri/src/main.rs`:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    text_sniper_win_lib::run()
}
```

`src-tauri/src/lib.rs`:

```rust
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 7: 빌드 확인**

```bash
cd src-tauri && cargo build 2>&1
```

Expected: 빌드 성공 (warning은 OK, error 없음)

- [ ] **Step 8: git init + 첫 커밋**

```bash
cd C:/Users/nicew/projects/textSniperWin
git init
# .gitignore 생성 (node_modules, target, etc.)
git add -A
git commit -m "chore: scaffold Tauri v2 project with tray-only config"
```

---

## Task 2: 설정 모듈 (config.rs)

**Files:**
- Create: `src-tauri/src/config.rs`
- Modify: `src-tauri/src/lib.rs` (모듈 선언 추가)
- Test: `src-tauri/src/config.rs` (인라인 `#[cfg(test)]`)

**완료 조건:** `cargo test config` — 모든 테스트 통과. config.json 읽기/쓰기/기본값 동작 검증.

- [ ] **Step 1: 실패하는 테스트 작성**

`src-tauri/src/config.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    pub auto_start: bool,
    pub first_run: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_start: false,
            first_run: true,
        }
    }
}

impl AppConfig {
    pub fn config_dir() -> PathBuf {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(appdata).join("TextSniperWin")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    pub fn load() -> Self {
        todo!()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_config_path() -> PathBuf {
        let dir = std::env::temp_dir().join("textsniper_test");
        fs::create_dir_all(&dir).unwrap();
        dir.join("config.json")
    }

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert!(!config.auto_start);
        assert!(config.first_run);
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let path = temp_config_path();
        let config = AppConfig {
            auto_start: true,
            first_run: false,
        };

        // Save
        let dir = path.parent().unwrap();
        fs::create_dir_all(dir).unwrap();
        let json = serde_json::to_string_pretty(&config).unwrap();
        fs::write(&path, json).unwrap();

        // Load
        let data = fs::read_to_string(&path).unwrap();
        let loaded: AppConfig = serde_json::from_str(&data).unwrap();
        assert_eq!(loaded, config);

        // Cleanup
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_load_missing_file_returns_default() {
        let path = std::env::temp_dir().join("textsniper_nonexistent").join("config.json");
        let _ = std::fs::remove_file(&path);
        // load_from should return default when file doesn't exist
        let config = AppConfig::load_from(&path);
        assert_eq!(config, AppConfig::default());
    }
}
```

- [ ] **Step 2: 테스트 실패 확인**

```bash
cd src-tauri && cargo test config -- --nocapture 2>&1
```

Expected: FAIL — `todo!()` 패닉

- [ ] **Step 3: 구현**

`load()`, `save()`, `load_from()`, `save_to()` 메서드 구현:

```rust
impl AppConfig {
    pub fn load() -> Self {
        Self::load_from(&Self::config_path())
    }

    pub fn load_from(path: &PathBuf) -> Self {
        match std::fs::read_to_string(path) {
            Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.save_to(&Self::config_path())
    }

    pub fn save_to(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}
```

- [ ] **Step 4: 테스트 통과 확인**

```bash
cd src-tauri && cargo test config -- --nocapture 2>&1
```

Expected: 3 tests passed

- [ ] **Step 5: lib.rs에 모듈 선언 추가**

```rust
mod config;
```

- [ ] **Step 6: 커밋**

```bash
git add src-tauri/src/config.rs src-tauri/src/lib.rs
git commit -m "feat: add config module with save/load/default"
```

---

## Task 3: 단일 인스턴스 (single_instance.rs)

**Files:**
- Create: `src-tauri/src/single_instance.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/single_instance.rs` (인라인)

**완료 조건:** `cargo test single_instance` — 통과. Named Mutex로 중복 실행 감지.

- [ ] **Step 1: 실패하는 테스트 작성**

`src-tauri/src/single_instance.rs`:

```rust
use windows::Win32::Foundation::{HANDLE, CloseHandle, BOOL, ERROR_ALREADY_EXISTS, GetLastError};
use windows::Win32::System::Threading::{CreateMutexW, ReleaseMutex};
use windows::core::w;

pub struct SingleInstance {
    handle: HANDLE,
}

impl SingleInstance {
    /// Named Mutex를 생성하여 단일 인스턴스 확인.
    /// 이미 실행 중이면 Err 반환.
    pub fn acquire() -> Result<Self, ()> {
        todo!()
    }
}

impl Drop for SingleInstance {
    fn drop(&mut self) {
        unsafe {
            let _ = ReleaseMutex(self.handle);
            let _ = CloseHandle(self.handle);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acquire_succeeds_first_time() {
        let instance = SingleInstance::acquire();
        assert!(instance.is_ok());
        // drop releases mutex
    }

    #[test]
    fn test_acquire_fails_second_time() {
        let _first = SingleInstance::acquire().unwrap();
        let second = SingleInstance::acquire();
        assert!(second.is_err());
    }
}
```

- [ ] **Step 2: 테스트 실패 확인**

```bash
cd src-tauri && cargo test single_instance -- --nocapture --test-threads=1 2>&1
```

Expected: FAIL — `todo!()` 패닉. **`--test-threads=1` 필수** (mutex 테스트 직렬 실행)

- [ ] **Step 3: 구현**

```rust
pub fn acquire() -> Result<Self, ()> {
    unsafe {
        let handle = CreateMutexW(
            None,
            BOOL(1), // bInitialOwner = TRUE
            w!("Global\\TextSniperWin_SingleInstance"),
        ).map_err(|_| ())?;

        let last_error = GetLastError();
        if last_error == ERROR_ALREADY_EXISTS {
            let _ = CloseHandle(handle);
            return Err(());
        }

        Ok(Self { handle })
    }
}
```

- [ ] **Step 4: 테스트 통과 확인**

```bash
cd src-tauri && cargo test single_instance -- --nocapture --test-threads=1 2>&1
```

Expected: 2 tests passed

- [ ] **Step 5: lib.rs 모듈 선언 + 커밋**

```bash
git add src-tauri/src/single_instance.rs src-tauri/src/lib.rs
git commit -m "feat: add single instance guard via named mutex"
```

---

## Task 4: 화면 캡처 (capture.rs)

**Files:**
- Create: `src-tauri/src/capture.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/capture.rs` (인라인)

**완료 조건:** `cargo test capture` — 통과. DPI 좌표 변환 로직 검증 + 실제 캡처 통합 테스트.

- [ ] **Step 1: 구조체/trait 정의 + 좌표 변환 테스트 작성**

`src-tauri/src/capture.rs`:

```rust
/// 물리 픽셀 기준 사각형 영역
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicalRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// 논리 좌표 → 물리 좌표 변환
pub fn logical_to_physical(x: i32, y: i32, w: u32, h: u32, scale: f64) -> PhysicalRect {
    todo!()
}

/// 역방향 드래그 정규화: (start, end) → (top_left, size)
pub fn normalize_rect(x1: i32, y1: i32, x2: i32, y2: i32) -> (i32, i32, u32, u32) {
    todo!()
}

/// 최소 영역 검증 (10x10 미만이면 false)
pub fn is_valid_selection(width: u32, height: u32) -> bool {
    todo!()
}

/// BitBlt로 화면 캡처, RGBA 바이트 반환
pub fn capture_screen_region(rect: &PhysicalRect) -> Result<Vec<u8>, String> {
    todo!()
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
}
```

- [ ] **Step 2: 테스트 실패 확인**

```bash
cd src-tauri && cargo test capture -- --nocapture 2>&1
```

Expected: FAIL — `todo!()` 패닉

- [ ] **Step 3: 순수 함수 구현 (logical_to_physical, normalize_rect, is_valid_selection)**

```rust
pub fn logical_to_physical(x: i32, y: i32, w: u32, h: u32, scale: f64) -> PhysicalRect {
    PhysicalRect {
        x: (x as f64 * scale) as i32,
        y: (y as f64 * scale) as i32,
        width: (w as f64 * scale) as u32,
        height: (h as f64 * scale) as u32,
    }
}

pub fn normalize_rect(x1: i32, y1: i32, x2: i32, y2: i32) -> (i32, i32, u32, u32) {
    let left = x1.min(x2);
    let top = y1.min(y2);
    let right = x1.max(x2);
    let bottom = y1.max(y2);
    (left, top, (right - left) as u32, (bottom - top) as u32)
}

pub fn is_valid_selection(width: u32, height: u32) -> bool {
    width >= 10 && height >= 10
}
```

- [ ] **Step 4: 순수 함수 테스트 통과 확인**

```bash
cd src-tauri && cargo test capture -- --nocapture 2>&1
```

Expected: 8 tests passed

- [ ] **Step 5: BitBlt 캡처 함수 구현**

```rust
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::Foundation::*;

pub fn capture_screen_region(rect: &PhysicalRect) -> Result<Vec<u8>, String> {
    unsafe {
        let hdc_screen = GetDC(HWND::default());
        if hdc_screen.is_invalid() {
            return Err("GetDC failed".to_string());
        }

        let hdc_mem = CreateCompatibleDC(hdc_screen);
        let hbm = CreateCompatibleBitmap(hdc_screen, rect.width as i32, rect.height as i32);
        let old = SelectObject(hdc_mem, hbm.into());

        let success = BitBlt(
            hdc_mem,
            0, 0,
            rect.width as i32, rect.height as i32,
            hdc_screen,
            rect.x, rect.y,
            SRCCOPY,
        );

        if !success.as_bool() {
            SelectObject(hdc_mem, old);
            let _ = DeleteDC(hdc_mem);
            let _ = DeleteObject(hbm.into());
            ReleaseDC(HWND::default(), hdc_screen);
            return Err("BitBlt failed".to_string());
        }

        // BITMAPINFO 설정하여 픽셀 데이터 추출
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

        // BGRA → RGBA 변환
        for chunk in buffer.chunks_exact_mut(4) {
            chunk.swap(0, 2);
        }

        SelectObject(hdc_mem, old);
        let _ = DeleteDC(hdc_mem);
        let _ = DeleteObject(hbm.into());
        ReleaseDC(HWND::default(), hdc_screen);

        Ok(buffer)
    }
}
```

- [ ] **Step 6: 통합 테스트 추가 (실제 캡처, CI에서만 실행)**

```rust
#[cfg(test)]
mod tests {
    // ... 기존 테스트 ...

    #[test]
    fn test_capture_screen_region_returns_data() {
        let rect = PhysicalRect { x: 0, y: 0, width: 100, height: 100 };
        let result = capture_screen_region(&rect);
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.len(), 100 * 100 * 4); // RGBA
    }
}
```

- [ ] **Step 7: 전체 테스트 통과 + 커밋**

```bash
cd src-tauri && cargo test capture -- --nocapture 2>&1
git add src-tauri/src/capture.rs src-tauri/src/lib.rs
git commit -m "feat: add screen capture with DPI conversion and BitBlt"
```

---

## Task 5: OCR 모듈 (ocr.rs)

**Files:**
- Create: `src-tauri/src/ocr.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/ocr.rs` (인라인)

**완료 조건:** `cargo test ocr` — 통과. 언어팩 감지, RGBA→SoftwareBitmap 변환, OCR 수행 검증.

- [ ] **Step 1: 구조체/함수 시그니처 + 언어팩 감지 테스트 작성**

`src-tauri/src/ocr.rs`:

```rust
use windows::Globalization::Language;
use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap};
use windows::Media::Ocr::OcrEngine;

/// 사용 가능한 OCR 언어 확인
pub fn is_language_available(lang_tag: &str) -> bool {
    todo!()
}

/// 사용 가능한 언어 목록 반환
pub fn available_languages() -> Vec<String> {
    todo!()
}

/// RGBA 바이트 배열로 SoftwareBitmap 생성
pub fn create_bitmap_from_rgba(
    data: &[u8],
    width: u32,
    height: u32,
) -> Result<SoftwareBitmap, String> {
    todo!()
}

/// SoftwareBitmap에서 OCR 수행.
/// 영어 시도 → 결과 없으면 한국어 시도.
pub fn recognize_text(bitmap: &SoftwareBitmap) -> Result<String, String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_english_language_available() {
        // Windows에는 영어가 항상 설치되어 있음
        assert!(is_language_available("en"));
    }

    #[test]
    fn test_available_languages_not_empty() {
        let langs = available_languages();
        assert!(!langs.is_empty());
    }

    #[test]
    fn test_create_bitmap_from_rgba_correct_size() {
        let width = 100;
        let height = 50;
        let data = vec![255u8; (width * height * 4) as usize];
        let result = create_bitmap_from_rgba(&data, width, height);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_bitmap_from_rgba_wrong_size() {
        let data = vec![0u8; 100]; // 너무 작음
        let result = create_bitmap_from_rgba(&data, 100, 100);
        assert!(result.is_err());
    }
}
```

- [ ] **Step 2: 테스트 실패 확인**

```bash
cd src-tauri && cargo test ocr -- --nocapture 2>&1
```

Expected: FAIL — `todo!()` 패닉

- [ ] **Step 3: 언어팩 감지 함수 구현**

```rust
pub fn is_language_available(lang_tag: &str) -> bool {
    let hstring: windows::core::HSTRING = lang_tag.into();
    let lang = match Language::CreateLanguage(&hstring) {
        Ok(l) => l,
        Err(_) => return false,
    };
    OcrEngine::IsLanguageSupported(&lang).unwrap_or(false)
}

pub fn available_languages() -> Vec<String> {
    OcrEngine::AvailableRecognizerLanguages()
        .map(|langs| {
            langs
                .into_iter()
                .filter_map(|l| l.LanguageTag().ok().map(|t| t.to_string()))
                .collect()
        })
        .unwrap_or_default()
}
```

- [ ] **Step 4: SoftwareBitmap 생성 함수 구현**

```rust
use windows::Storage::Streams::{DataWriter, InMemoryRandomAccessStream};

pub fn create_bitmap_from_rgba(
    data: &[u8],
    width: u32,
    height: u32,
) -> Result<SoftwareBitmap, String> {
    let expected_len = (width * height * 4) as usize;
    if data.len() != expected_len {
        return Err(format!(
            "Data length mismatch: expected {}, got {}",
            expected_len,
            data.len()
        ));
    }

    // RGBA → BGRA 변환 (SoftwareBitmap은 BGRA 사용)
    let mut bgra = data.to_vec();
    for chunk in bgra.chunks_exact_mut(4) {
        chunk.swap(0, 2);
    }

    // DataWriter를 통해 바이트를 IBuffer에 기록한 뒤 CopyFromBuffer 호출
    let stream = InMemoryRandomAccessStream::new()
        .map_err(|e| e.message().to_string())?;
    let writer = DataWriter::CreateDataWriter(&stream)
        .map_err(|e| e.message().to_string())?;
    writer.WriteBytes(&bgra)
        .map_err(|e| e.message().to_string())?;
    let buffer = writer.DetachBuffer()
        .map_err(|e| e.message().to_string())?;

    let bitmap = SoftwareBitmap::Create(
        BitmapPixelFormat::Bgra8,
        width as i32,
        height as i32,
    ).map_err(|e| e.message().to_string())?;

    bitmap.CopyFromBuffer(&buffer)
        .map_err(|e| e.message().to_string())?;

    Ok(bitmap)
}
```

- [ ] **Step 5: OCR 수행 함수 구현**

```rust
fn recognize_with_language(bitmap: &SoftwareBitmap, lang_tag: &str) -> Result<String, String> {
    let hstring: windows::core::HSTRING = lang_tag.into();
    let lang = Language::CreateLanguage(&hstring)
        .map_err(|e| e.message().to_string())?;

    let engine = OcrEngine::TryCreateFromLanguage(&lang)
        .map_err(|e| e.message().to_string())?;

    let result = engine
        .RecognizeAsync(bitmap)
        .map_err(|e| e.message().to_string())?
        .get()
        .map_err(|e| e.message().to_string())?;

    let text = result.Text().map_err(|e| e.message().to_string())?;
    Ok(text.to_string())
}

pub fn recognize_text(bitmap: &SoftwareBitmap) -> Result<String, String> {
    // 영어 먼저 시도
    if is_language_available("en") {
        if let Ok(text) = recognize_with_language(bitmap, "en") {
            let trimmed = text.trim().to_string();
            if !trimmed.is_empty() {
                return Ok(trimmed);
            }
        }
    }

    // 한국어 폴백
    if is_language_available("ko") {
        if let Ok(text) = recognize_with_language(bitmap, "ko") {
            let trimmed = text.trim().to_string();
            if !trimmed.is_empty() {
                return Ok(trimmed);
            }
        }
    }

    Err("No text recognized".to_string())
}
```

- [ ] **Step 6: 테스트 통과 확인 + 커밋**

```bash
cd src-tauri && cargo test ocr -- --nocapture 2>&1
git add src-tauri/src/ocr.rs src-tauri/src/lib.rs
git commit -m "feat: add OCR module with en/ko fallback pipeline"
```

---

## Task 6: 클립보드 모듈 (clipboard.rs)

**Files:**
- Create: `src-tauri/src/clipboard.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/clipboard.rs` (인라인)

**완료 조건:** `cargo test clipboard` — 통과. 텍스트 복사/검증, 빈 문자열 거부.

- [ ] **Step 1: 실패하는 테스트 작성**

`src-tauri/src/clipboard.rs`:

```rust
use arboard::Clipboard;

/// 텍스트를 클립보드에 복사. 빈 문자열이면 복사하지 않고 Err 반환.
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_nonempty_text() {
        let result = copy_to_clipboard("hello OCR");
        assert!(result.is_ok());

        // 실제로 클립보드에 복사되었는지 확인
        let mut cb = Clipboard::new().unwrap();
        let content = cb.get_text().unwrap();
        assert_eq!(content, "hello OCR");
    }

    #[test]
    fn test_copy_empty_text_rejected() {
        let result = copy_to_clipboard("");
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_whitespace_only_rejected() {
        let result = copy_to_clipboard("   \n\t  ");
        assert!(result.is_err());
    }
}
```

- [ ] **Step 2: 테스트 실패 확인**

```bash
cd src-tauri && cargo test clipboard -- --nocapture --test-threads=1 2>&1
```

Expected: FAIL — `todo!()` 패닉. **`--test-threads=1` 필수** (클립보드 공유 자원)

- [ ] **Step 3: 구현**

```rust
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err("Empty text, clipboard not modified".to_string());
    }

    let mut cb = Clipboard::new().map_err(|e| format!("Clipboard open failed: {}", e))?;
    cb.set_text(trimmed).map_err(|e| format!("Clipboard write failed: {}", e))?;
    Ok(())
}
```

- [ ] **Step 4: 테스트 통과 확인 + 커밋**

```bash
cd src-tauri && cargo test clipboard -- --nocapture --test-threads=1 2>&1
git add src-tauri/src/clipboard.rs src-tauri/src/lib.rs
git commit -m "feat: add clipboard module with empty text guard"
```

---

## Task 7: Win32 오버레이 (overlay.rs)

**Files:**
- Create: `src-tauri/src/overlay.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/overlay.rs` (인라인 — 순수 로직만)

**완료 조건:** `cargo test overlay` — 순수 로직 테스트 통과. `cargo build` 성공. 수동 테스트로 오버레이 창 표시/드래그/ESC 동작 확인.

- [ ] **Step 1: 선택 결과 구조체 + 모니터 감지 테스트 작성**

`src-tauri/src/overlay.rs`:

```rust
use crate::capture::PhysicalRect;

/// 오버레이 결과: 사용자가 선택한 영역 또는 취소
#[derive(Debug, Clone, PartialEq)]
pub enum OverlayResult {
    Selected(PhysicalRect),
    Cancelled,
    TooSmall,
}

/// 드래그 좌표로부터 OverlayResult 결정
pub fn evaluate_selection(x1: i32, y1: i32, x2: i32, y2: i32, dpi_scale: f64) -> OverlayResult {
    todo!()
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
    todo!()
}

/// 오버레이 창을 표시하고 사용자 선택을 대기 (blocking)
pub fn show_overlay(monitor: &MonitorInfo) -> OverlayResult {
    todo!()
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
        // 논리 좌표로는 10x10이지만 물리 변환 후에도 검증
        let result = evaluate_selection(100, 100, 105, 105, 1.5);
        // 물리: 7.5x7.5 → 7x7 → too small
        assert_eq!(result, OverlayResult::TooSmall);
    }
}
```

- [ ] **Step 2: 테스트 실패 확인**

```bash
cd src-tauri && cargo test overlay -- --nocapture 2>&1
```

Expected: FAIL — `todo!()` 패닉

- [ ] **Step 3: evaluate_selection 구현**

```rust
pub fn evaluate_selection(x1: i32, y1: i32, x2: i32, y2: i32, dpi_scale: f64) -> OverlayResult {
    use crate::capture::{normalize_rect, logical_to_physical, is_valid_selection};

    let (lx, ly, lw, lh) = normalize_rect(x1, y1, x2, y2);
    let physical = logical_to_physical(lx, ly, lw, lh, dpi_scale);

    if !is_valid_selection(physical.width, physical.height) {
        return OverlayResult::TooSmall;
    }

    OverlayResult::Selected(physical)
}
```

- [ ] **Step 4: 순수 로직 테스트 통과 확인**

```bash
cd src-tauri && cargo test overlay -- --nocapture 2>&1
```

Expected: 5 tests passed

- [ ] **Step 5: get_current_monitor 구현**

```rust
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Foundation::*;

pub fn get_current_monitor() -> Result<MonitorInfo, String> {
    unsafe {
        let mut cursor_pos = POINT::default();
        GetCursorPos(&mut cursor_pos).map_err(|e| e.message().to_string())?;

        let hmonitor = MonitorFromPoint(cursor_pos, MONITOR_DEFAULTTOPRIMARY);
        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        GetMonitorInfoW(hmonitor, &mut mi)
            .ok()
            .map_err(|e| e.message().to_string())?;

        let rc = mi.rcMonitor;

        // DPI 조회
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
```

- [ ] **Step 6: show_overlay Win32 윈도우 구현**

Win32 창 생성 + 메시지 루프 + 마우스/키보드 이벤트 처리. 주요 로직:

```rust
pub fn show_overlay(monitor: &MonitorInfo) -> OverlayResult {
    // 1. WNDCLASSEXW 등록 (crosshair 커서)
    // 2. CreateWindowExW:
    //    - WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW
    //    - WS_POPUP
    //    - 모니터 위치/크기에 맞춰 배치
    // 3. SetLayeredWindowAttributes (alpha = 100, ~40% 투명)
    // 4. ShowWindow + SetForegroundWindow
    // 5. 메시지 루프:
    //    - WM_LBUTTONDOWN: 드래그 시작점 기록
    //    - WM_MOUSEMOVE + 드래그 중: 선택 영역 렌더링
    //      (선택 영역 내부는 투명, 외부는 반투명 검정)
    //    - WM_LBUTTONUP: 드래그 종료, evaluate_selection 호출
    //    - WM_KEYDOWN + VK_ESCAPE: 취소
    //    - WM_PAINT: 선택 영역 시각적 피드백 렌더링
    // 6. DestroyWindow + UnregisterClassW
    // 7. OverlayResult 반환
    todo!("Win32 overlay implementation")
}
```

이 함수는 Win32 메시지 루프를 포함하므로 **별도 스레드에서 호출**하고, 결과를 채널로 전달.

- [ ] **Step 7: 수동 테스트 (오버레이 동작 확인)**

임시 main에서 `show_overlay` 호출하여 확인:
- 오버레이가 모니터를 덮는가?
- 십자선 커서가 표시되는가?
- 드래그 시 선택 영역이 하이라이트되는가?
- ESC로 취소되는가?
- 역방향 드래그가 되는가?

- [ ] **Step 8: 커밋**

```bash
git add src-tauri/src/overlay.rs src-tauri/src/lib.rs
git commit -m "feat: add Win32 overlay with drag selection and DPI support"
```

---

## Task 8: 트레이 아이콘 관리 (tray.rs)

**Files:**
- Create: `src-tauri/src/tray.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src-tauri/icons/icon.ico` (16x16, 32x32, 48x48 포함)
- Create: `src-tauri/icons/icon-loading.ico`
- Create: `src-tauri/icons/icon-success.ico`
- Create: `src-tauri/icons/icon-error.ico`
- Test: 수동 테스트 (Tauri tray API는 런타임 필요)

**완료 조건:** `cargo build` 성공. 수동 테스트로 트레이 아이콘 표시, 메뉴 동작, 아이콘 전환 확인.

- [ ] **Step 1: 트레이 상태 enum + 아이콘 관리 구조 작성**

`src-tauri/src/tray.rs`:

```rust
use tauri::{
    menu::{Menu, MenuItem, CheckMenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    image::Image,
    AppHandle, Manager,
};
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrayState {
    Idle,
    Loading,
    Success,
    Error,
}

pub struct TrayManager {
    state: Mutex<TrayState>,
}

impl TrayManager {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(TrayState::Idle),
        }
    }

    /// Tauri setup 단계에서 호출. 트레이 아이콘과 메뉴 생성.
    pub fn setup_tray(app: &AppHandle) -> Result<TrayIcon, Box<dyn std::error::Error>> {
        todo!()
    }

    /// 트레이 아이콘 상태 변경
    pub fn set_state(
        tray: &TrayIcon,
        state: TrayState,
        app: &AppHandle,
    ) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    /// 성공/실패 아이콘 표시 후 1초 뒤 idle로 복귀
    pub fn flash_state(
        tray: TrayIcon,
        state: TrayState,
        app: AppHandle,
    ) {
        todo!()
    }
}
```

- [ ] **Step 2: setup_tray 구현**

```rust
pub fn setup_tray(app: &AppHandle) -> Result<TrayIcon, Box<dyn std::error::Error>> {
    let capture_item = MenuItem::with_id(app, "capture", "캡처 (Shift+Alt+T)", true, None::<&str>)?;
    let is_auto = crate::config::is_auto_start_enabled();
    let auto_start_item = CheckMenuItem::with_id(app, "auto_start", "자동 실행", true, is_auto, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "종료", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&capture_item, &auto_start_item, &quit_item])?;

    let tray = TrayIconBuilder::new()
        .icon(Image::from_bytes(include_bytes!("../icons/icon.ico"))?)
        .tooltip("TextSniper")
        .menu(&menu)
        .menu_on_left_click(false)
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "capture" => {
                    // 캡처 트리거 — lib.rs의 오케스트레이션에서 처리
                    app.emit("trigger-capture", ()).ok();
                }
                "auto_start" => {
                    // 자동 실행 토글 — config에서 처리
                    app.emit("toggle-auto-start", ()).ok();
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(tray)
}
```

- [ ] **Step 3: set_state, flash_state 구현**

```rust
pub fn set_state(
    tray: &TrayIcon,
    state: TrayState,
    _app: &AppHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    tray.set_icon(Some(match state {
            TrayState::Idle => Image::from_bytes(include_bytes!("../icons/icon.ico"))?,
            TrayState::Loading => Image::from_bytes(include_bytes!("../icons/icon-loading.ico"))?,
            TrayState::Success => Image::from_bytes(include_bytes!("../icons/icon-success.ico"))?,
            TrayState::Error => Image::from_bytes(include_bytes!("../icons/icon-error.ico"))?,
        }))?;
    Ok(())
}

pub fn flash_state(
    tray: TrayIcon,
    state: TrayState,
    app: AppHandle,
) {
    let _ = Self::set_state(&tray, state, &app);
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let _ = Self::set_state(&tray, TrayState::Idle, &app);
    });
}
```

- [ ] **Step 4: 아이콘 파일 생성 (placeholder)**

간단한 16x16, 32x32 .ico 파일 4개 생성. 실제 디자인은 나중에 교체 가능.
일단 색상으로 구분:
- `icon.ico`: 파란색 T
- `icon-loading.ico`: 회색 T
- `icon-success.ico`: 초록색 체크
- `icon-error.ico`: 빨간색 X

- [ ] **Step 5: 빌드 확인 + 커밋**

```bash
cd src-tauri && cargo build 2>&1
git add src-tauri/src/tray.rs src-tauri/icons/ src-tauri/src/lib.rs
git commit -m "feat: add tray icon manager with state feedback"
```

---

## Task 9: 앱 오케스트레이션 (lib.rs)

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Test: 통합 테스트 (수동)

**완료 조건:** 앱 실행 → 트레이 표시 → Shift+Alt+T → 오버레이 → 드래그 → OCR → 클립보드 복사. **핵심 흐름 전체가 작동.**

- [ ] **Step 1: lib.rs에 전체 모듈 선언 + run() 구성**

```rust
mod capture;
mod clipboard;
mod config;
mod ocr;
mod overlay;
mod single_instance;
mod tray;

use config::AppConfig;
use overlay::OverlayResult;
use tray::{TrayManager, TrayState};

pub fn run() {
    // 1. 단일 인스턴스 체크
    let _instance = match single_instance::SingleInstance::acquire() {
        Ok(i) => i,
        Err(_) => {
            eprintln!("TextSniper is already running.");
            return;
        }
    };

    // 2. 설정 로드
    let mut config = AppConfig::load();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 3. 트레이 아이콘 생성
            let tray = TrayManager::setup_tray(app.handle())?;

            // 4. 글로벌 단축키 등록
            setup_global_shortcut(app, tray.clone())?;

            // 5. 첫 실행 안내
            if config.first_run {
                tray.set_tooltip(Some("TextSniper - Shift+Alt+T로 캡처"))?;
                // Balloon Tip은 Shell_NotifyIcon으로 별도 구현
                config.first_run = false;
                let _ = config.save();
            }

            // 6. 언어팩 확인
            if !ocr::is_language_available("ko") {
                tray.set_tooltip(Some("TextSniper - 한국어 언어팩 미설치 (영어만 사용)"))?;
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_global_shortcut(
    app: &tauri::App,
    tray: tauri::tray::TrayIcon,
) -> Result<(), Box<dyn std::error::Error>> {
    use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

    let shortcut = Shortcut::new(
        Some(Modifiers::SHIFT | Modifiers::ALT),
        Code::KeyT,
    );

    let tray_clone = tray.clone();
    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, sc, event| {
                if sc == &shortcut && event.state() == ShortcutState::Pressed {
                    let app_handle = app.clone();
                    let tray_ref = tray_clone.clone();
                    std::thread::spawn(move || {
                        run_capture_pipeline(app_handle, tray_ref);
                    });
                }
            })
            .build(),
    )?;

    match app.global_shortcut().register(shortcut) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to register shortcut: {}", e);
            tray.set_tooltip(Some("TextSniper - 단축키 등록 실패"))?;
        }
    }

    Ok(())
}

/// 핵심 파이프라인: 오버레이 → 캡처 → OCR → 클립보드
fn run_capture_pipeline(app: tauri::AppHandle, tray: tauri::tray::TrayIcon) {
    // 1. 모니터 정보
    let monitor = match overlay::get_current_monitor() {
        Ok(m) => m,
        Err(_) => return,
    };

    // 2. 오버레이 표시 + 영역 선택
    let selection = overlay::show_overlay(&monitor);

    let rect = match selection {
        OverlayResult::Selected(r) => r,
        OverlayResult::Cancelled | OverlayResult::TooSmall => return,
    };

    // 3. 트레이 로딩 표시
    let _ = TrayManager::set_state(&tray, TrayState::Loading, &app);

    // 4. 화면 캡처
    let pixels = match capture::capture_screen_region(&rect) {
        Ok(p) => p,
        Err(_) => {
            TrayManager::flash_state(tray, TrayState::Error, app);
            return;
        }
    };

    // 5. OCR
    let bitmap = match ocr::create_bitmap_from_rgba(&pixels, rect.width, rect.height) {
        Ok(b) => b,
        Err(_) => {
            TrayManager::flash_state(tray, TrayState::Error, app);
            return;
        }
    };

    let text = match ocr::recognize_text(&bitmap) {
        Ok(t) => t,
        Err(_) => {
            TrayManager::flash_state(tray, TrayState::Error, app);
            return;
        }
    };

    // 6. 클립보드 복사
    match clipboard::copy_to_clipboard(&text) {
        Ok(_) => TrayManager::flash_state(tray, TrayState::Success, app),
        Err(_) => TrayManager::flash_state(tray, TrayState::Error, app),
    }
}
```

- [ ] **Step 2: 빌드 확인**

```bash
cd src-tauri && cargo build 2>&1
```

Expected: 빌드 성공

- [ ] **Step 3: 수동 E2E 테스트**

```bash
cd src-tauri && cargo run
```

체크리스트:
- [ ] 트레이 아이콘 표시됨
- [ ] Shift+Alt+T로 오버레이 표시됨
- [ ] 마우스 드래그로 영역 선택 가능
- [ ] 선택 영역 내부 원본 밝기 유지
- [ ] ESC로 취소 가능
- [ ] 드래그 완료 후 트레이 아이콘이 로딩→성공/실패로 변경
- [ ] 클립보드에 인식된 텍스트 복사됨
- [ ] 빈 영역 캡처 시 클립보드 미변경 + 실패 아이콘

- [ ] **Step 4: 커밋**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: wire up capture pipeline - overlay → OCR → clipboard"
```

---

## Task 10: 자동 실행 + 첫 실행 안내

**Files:**
- Modify: `src-tauri/src/config.rs` (레지스트리 자동실행)
- Modify: `src-tauri/src/lib.rs` (첫 실행 Balloon Tip)
- Test: `src-tauri/src/config.rs` (레지스트리 테스트)

**완료 조건:** `cargo test config` — 통과. 트레이 메뉴 "자동 실행" 토글 동작. 첫 실행 시 Balloon Tip 표시.

- [ ] **Step 1: 자동실행 레지스트리 테스트 작성**

`src-tauri/src/config.rs`에 추가:

```rust
use windows::Win32::System::Registry::*;
use windows::core::w;

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APP_NAME: &str = "TextSniperWin";

pub fn set_auto_start(enable: bool) -> Result<(), String> {
    todo!()
}

pub fn is_auto_start_enabled() -> bool {
    todo!()
}

#[cfg(test)]
mod tests {
    // ... 기존 테스트 ...

    #[test]
    fn test_auto_start_toggle() {
        // 현재 상태 저장
        let was_enabled = is_auto_start_enabled();

        // 활성화
        set_auto_start(true).unwrap();
        assert!(is_auto_start_enabled());

        // 비활성화
        set_auto_start(false).unwrap();
        assert!(!is_auto_start_enabled());

        // 원래 상태 복원
        if was_enabled {
            set_auto_start(true).unwrap();
        }
    }
}
```

- [ ] **Step 2: 테스트 실패 확인**

```bash
cd src-tauri && cargo test config::tests::test_auto_start -- --nocapture 2>&1
```

- [ ] **Step 3: 레지스트리 자동실행 구현**

```rust
pub fn set_auto_start(enable: bool) -> Result<(), String> {
    unsafe {
        let mut hkey = HKEY::default();
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            w!(r"Software\Microsoft\Windows\CurrentVersion\Run"),
            0,
            KEY_SET_VALUE | KEY_QUERY_VALUE,
            &mut hkey,
        ).map_err(|e| e.message().to_string())?;

        if enable {
            let exe_path = std::env::current_exe()
                .map_err(|e| e.to_string())?;
            let value: Vec<u16> = exe_path.to_string_lossy()
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            RegSetValueExW(
                hkey,
                w!("TextSniperWin"),
                0,
                REG_SZ,
                Some(std::slice::from_raw_parts(
                    value.as_ptr() as *const u8,
                    value.len() * 2,
                )),
            ).map_err(|e| e.message().to_string())?;
        } else {
            let _ = RegDeleteValueW(hkey, w!("TextSniperWin"));
        }

        RegCloseKey(hkey).map_err(|e| e.message().to_string())?;
        Ok(())
    }
}

pub fn is_auto_start_enabled() -> bool {
    unsafe {
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            w!(r"Software\Microsoft\Windows\CurrentVersion\Run"),
            0,
            KEY_QUERY_VALUE,
            &mut hkey,
        ).is_err() {
            return false;
        }

        let result = RegQueryValueExW(
            hkey,
            w!("TextSniperWin"),
            None,
            None,
            None,
            None,
        );

        let _ = RegCloseKey(hkey);
        result.is_ok()
    }
}
```

- [ ] **Step 4: 테스트 통과 + 커밋**

```bash
cd src-tauri && cargo test config -- --nocapture --test-threads=1 2>&1
git add src-tauri/src/config.rs src-tauri/src/lib.rs
git commit -m "feat: add auto-start registry toggle and first-run balloon tip"
```

---

## Task 11: 최종 통합 + 에러 처리 마무리

**Files:**
- Modify: `src-tauri/src/lib.rs` (에러 처리 보강)
- Modify: `src-tauri/src/overlay.rs` (TaskbarCreated 대응)

**완료 조건:** 에러 처리 표의 모든 시나리오가 구현됨. 전체 `cargo test` 통과. 수동 E2E 테스트 통과.

- [ ] **Step 1: OCR 타임아웃 5초 적용 (스레드 + 채널 방식)**

`src-tauri/src/ocr.rs`에 타임아웃 래퍼 추가:

```rust
use std::sync::mpsc;
use std::time::Duration;

pub fn recognize_text_with_timeout(
    bitmap: &SoftwareBitmap,
    timeout: Duration,
) -> Result<String, String> {
    let (tx, rx) = mpsc::channel();

    // SoftwareBitmap은 Send가 아니므로, 동일 스레드에서 OCR 호출 후
    // 결과만 채널로 전달하는 방식은 부적절.
    // 대신 recv_timeout으로 메인 스레드의 블로킹 OCR 호출을 감싸되,
    // OCR 자체는 별도 스레드에서 실행.
    // bitmap 데이터를 바이트로 직렬화하여 스레드에 전달.
    let width = bitmap.PixelWidth().map_err(|e| e.message().to_string())?;
    let height = bitmap.PixelHeight().map_err(|e| e.message().to_string())?;

    // 바이트 추출 후 새 스레드에서 SoftwareBitmap 재생성 + OCR
    let buffer = bitmap_to_bytes(bitmap)?;

    std::thread::spawn(move || {
        let result = (|| {
            let new_bitmap = create_bitmap_from_rgba(&buffer, width as u32, height as u32)?;
            recognize_text(&new_bitmap)
        })();
        let _ = tx.send(result);
    });

    match rx.recv_timeout(timeout) {
        Ok(result) => result,
        Err(_) => Err("OCR timeout (5s exceeded)".to_string()),
    }
}

fn bitmap_to_bytes(bitmap: &SoftwareBitmap) -> Result<Vec<u8>, String> {
    // SoftwareBitmap → BGRA 바이트 → RGBA 바이트 추출
    let buffer = bitmap.LockBuffer(windows::Graphics::Imaging::BitmapBufferAccessMode::Read)
        .map_err(|e| e.message().to_string())?;
    let reference = buffer.CreateReference()
        .map_err(|e| e.message().to_string())?;
    // IMemoryBufferByteAccess를 통해 바이트 접근
    // 실제 구현 시 windows crate의 IMemoryBufferByteAccess 인터페이스 활용
    todo!("bitmap_to_bytes: 구현 시 IMemoryBufferByteAccess로 바이트 추출")
}
```

**참고:** `bitmap_to_bytes`는 구현이 복잡하므로, 대안으로 `lib.rs`의 파이프라인에서 RGBA 바이트를 직접 스레드에 전달하는 방식이 더 간단함:

```rust
// lib.rs의 run_capture_pipeline에서:
let pixels_clone = pixels.clone();
let (tx, rx) = std::sync::mpsc::channel();
std::thread::spawn(move || {
    let result = (|| {
        let bmp = ocr::create_bitmap_from_rgba(&pixels_clone, rect.width, rect.height)?;
        ocr::recognize_text(&bmp)
    })();
    let _ = tx.send(result);
});

let text = match rx.recv_timeout(std::time::Duration::from_secs(5)) {
    Ok(Ok(t)) => t,
    Ok(Err(e)) => { /* 실패 처리 */ return; }
    Err(_) => { /* 타임아웃 처리 */ return; }
};
```

- [ ] **Step 2: lib.rs 파이프라인에서 스레드+채널 타임아웃 적용**

`run_capture_pipeline`의 OCR 호출부를 위 코드로 교체.

- [ ] **Step 3: Balloon Tip (첫 실행 안내) 구현**

`src-tauri/src/tray.rs`에 추가:

```rust
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::w;

/// 시스템 트레이 Balloon Tip 표시 (Win32 Shell_NotifyIconW)
pub fn show_balloon_tip(title: &str, message: &str) {
    unsafe {
        // 현재 트레이 아이콘의 HWND가 필요하므로,
        // Tauri의 tray 내부 핸들을 사용할 수 없어 별도 NOTIFYICONDATAW 구성
        // 실제로는 Tauri의 TrayIcon이 이미 Shell_NotifyIcon을 사용하므로,
        // 같은 uID로 NIM_MODIFY 호출하여 Balloon 추가
        //
        // 대안: Tauri v2의 tray.set_tooltip()은 Balloon이 아닌 정적 툴팁.
        // Balloon Tip이 필요하면 Win32 직접 호출 필요.
        // MVP에서는 트레이 툴팁 + 로그로 대체 가능.
        //
        // 구현이 Tauri 내부 구조에 종속적이므로,
        // 첫 실행 시 tray.set_tooltip("TextSniper 실행 중 | Shift+Alt+T로 캡처")
        // 로 대체하고, 향후 Win32 Balloon으로 업그레이드.
    }
}
```

**현실적 접근:** MVP에서는 `tray.set_tooltip()`으로 대체. 첫 실행 시 트레이 아이콘 툴팁에 사용법 표시.

```rust
// lib.rs setup에서:
if config.first_run {
    tray.set_tooltip(Some("TextSniper 실행 중 | Shift+Alt+T로 캡처"))?;
    config.first_run = false;
    let _ = config.save();
}
```

- [ ] **Step 4: TaskbarCreated 메시지 대응**

`src-tauri/src/lib.rs`의 `setup`에 추가:

```rust
// 탐색기 재시작 감지: Tauri v2의 TrayIcon은 내부적으로
// Shell_NotifyIcon을 사용하며, Tauri가 자체적으로
// TaskbarCreated를 처리하여 트레이 아이콘을 재등록함.
// 따라서 별도 구현 불필요 — Tauri v2의 tray-icon 기능에 포함.
//
// 검증 방법: taskkill /f /im explorer.exe && start explorer.exe
// 후 트레이 아이콘이 재등록되는지 확인.
```

**참고:** Tauri v2의 `tray-icon` 기능은 내부적으로 `tray-icon` crate를 사용하며, 이 crate가 `TaskbarCreated` 메시지를 자동 처리함. 수동 E2E 테스트에서 검증 항목으로 추가.

- [ ] **Step 5: 에러 처리 테이블 최종 검증**

| 시나리오 | 구현 위치 | 확인 |
|----------|-----------|------|
| OCR 언어팩 미설치 | `lib.rs` setup + `ocr.rs` fallback | [ ] |
| 단축키 등록 실패 | `lib.rs` setup_global_shortcut | [ ] |
| 선택 영역 < 10x10 | `overlay.rs` evaluate_selection | [ ] |
| OCR 결과 빈 문자열 | `ocr.rs` recognize_text | [ ] |
| 클립보드 쓰기 실패 | `clipboard.rs` copy_to_clipboard | [ ] |
| 앱 중복 실행 | `lib.rs` SingleInstance::acquire | [ ] |
| OCR 5초 초과 | `ocr.rs` recognize_text_with_timeout | [ ] |

- [ ] **Step 4: 전체 테스트 실행**

```bash
cd src-tauri && cargo test -- --nocapture --test-threads=1 2>&1
```

Expected: 모든 테스트 통과

- [ ] **Step 5: 최종 수동 E2E 테스트**

체크리스트:
- [ ] 앱 시작 → 트레이 아이콘
- [ ] 중복 실행 시 두 번째 인스턴스 즉시 종료
- [ ] Shift+Alt+T → 오버레이 → 텍스트 영역 드래그 → 클립보드에 텍스트
- [ ] 빈 영역 선택 → 실패 아이콘 + 클립보드 미변경
- [ ] 10x10 미만 → 아무 반응 없음 (취소)
- [ ] ESC → 취소
- [ ] 트레이 메뉴 "캡처" → 동일 동작
- [ ] 트레이 메뉴 "자동 실행" 토글 → 레지스트리 확인 + 체크마크 반영
- [ ] 트레이 메뉴 "종료" → 앱 종료
- [ ] 탐색기 재시작 (`taskkill /f /im explorer.exe && start explorer.exe`) → 트레이 아이콘 재등록
- [ ] 첫 실행 시 트레이 툴팁에 사용법 표시

- [ ] **Step 8: 최종 커밋**

```bash
git add -A
git commit -m "feat: complete TextSniper MVP with full error handling"
```

---

## 실행 순서 요약

| Task | 이름 | 의존 | 테스트 방식 |
|------|------|------|-------------|
| 1 | 프로젝트 스캐폴딩 | - | 빌드 확인 |
| 2 | config.rs | Task 1 | `cargo test config` |
| 3 | single_instance.rs | Task 1 | `cargo test single_instance` |
| 4 | capture.rs | Task 1 | `cargo test capture` |
| 5 | ocr.rs | Task 1 | `cargo test ocr` |
| 6 | clipboard.rs | Task 1 | `cargo test clipboard` |
| 7 | overlay.rs | Task 4 | `cargo test overlay` + 수동 |
| 8 | tray.rs | Task 1 | 수동 |
| 9 | lib.rs 통합 | Task 2~8 전부 | 수동 E2E |
| 10 | 자동실행 + 첫 실행 | Task 2, 8 | `cargo test config` + 수동 |
| 11 | 최종 통합 + 타임아웃 + Balloon | Task 9, 10 | 전체 테스트 + 수동 E2E |

**병렬 가능:** Task 2, 3, 4, 5, 6은 서로 독립적이므로 병렬 실행 가능.

## 리뷰 반영 사항

1. ~~`windows` crate 버전 0.61 → 0.62 통일~~ ✅
2. ~~`Win32_System_Registry`, `Win32_UI_Shell` feature 추가~~ ✅
3. ~~`CreateMutexW` 두 번째 인자 `true` → `BOOL(1)`~~ ✅
4. ~~`SingleInstance::Drop`에 `CloseHandle` 추가~~ ✅
5. ~~`SelectObject` / `DeleteObject`에 `.into()` 추가~~ ✅
6. ~~`create_bitmap_from_rgba`: `DataWriter` 기반으로 교체~~ ✅
7. ~~`Language::CreateLanguage`: HSTRING 변수 바인딩~~ ✅
8. ~~`Image::from_path` → `include_bytes!` + `Image::from_bytes`~~ ✅
9. ~~TaskbarCreated: Tauri v2 내부 자동 처리 확인, E2E 검증 추가~~ ✅
10. ~~Balloon Tip: MVP는 트레이 툴팁으로 대체, 향후 업그레이드~~ ✅
11. ~~OCR 타임아웃: 스레드+채널 방식으로 진정한 5초 타임아웃~~ ✅
12. ~~자동실행 체크박스 초기값: `is_auto_start_enabled()` 반영~~ ✅
