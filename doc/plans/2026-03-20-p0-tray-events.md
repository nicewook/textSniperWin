# P0: 트레이 메뉴 이벤트 연결 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 트레이 메뉴의 "캡처", "자동 실행" 클릭이 실제 동작하도록 이벤트 핸들러를 연결한다.

**Architecture:** 현재 `on_menu_event`에서 Tauri 이벤트를 emit만 하고 수신하지 않는 구조를 제거하고, 클로저 안에서 직접 로직을 호출한다. `run_capture_pipeline`을 `pub`으로 노출하여 `tray.rs`에서 호출 가능하게 하고, `config::set_auto_start` + CheckMenuItem 상태 갱신을 직접 수행한다.

**Tech Stack:** Rust, Tauri v2, windows crate 0.62

---

## 현재 문제 분석

| # | 문제 | 원인 위치 |
|---|------|-----------|
| 1 | 트레이 "캡처" 클릭 무반응 | `tray.rs:45` — `emit("trigger-capture")` 하지만 리스너 없음 |
| 2 | 트레이 "자동 실행" 토글 무반응 | `tray.rs:48` — `emit("toggle-auto-start")` 하지만 리스너 없음 |
| 3 | CheckMenuItem 체크 상태 미반영 | 토글 시 레지스트리/config 동기화 없음 (muda가 UI 체크는 자동 토글) |

## 파일 맵

- **Modify:** `src-tauri/src/lib.rs` — `run_capture_pipeline`을 `pub`로 변경
- **Modify:** `src-tauri/src/tray.rs` — `on_menu_event` 클로저에서 직접 로직 호출
- **Test:** `src-tauri/src/tray.rs` — 기존 테스트 + 수동 검증 (트레이 UI는 단위 테스트 불가)

## 설계 결정

**emit/listen 제거, 직접 호출 방식 선택 이유:**
- Tauri 이벤트 시스템은 webview ↔ backend 통신용. 트레이 → backend는 같은 Rust 프로세스 내이므로 직접 호출이 자연스럽다.
- 리스너 등록 코드가 불필요해지고, 디버깅이 쉬워진다.

**muda CheckMenuItem 자동 토글 동작:**
- muda (Tauri 내부 메뉴 라이브러리)는 CheckMenuItem 클릭 시 **자동으로 체크 상태를 토글**한다.
- `on_menu_event` 콜백이 호출될 때 `is_checked()`는 이미 **토글된 새 값**을 반환한다.
- 따라서 `set_checked()`로 UI를 수동 갱신할 필요 없음. 레지스트리/config 동기화만 하면 된다.
- 에러 시에만 `set_checked(!new_state)`로 **되돌리기**가 필요하다.

---

### Task 1: `run_capture_pipeline`을 `pub`로 노출

**Files:**
- Modify: `src-tauri/src/lib.rs:90` — `fn run_capture_pipeline` → `pub fn run_capture_pipeline`

- [ ] **Step 1: `run_capture_pipeline` 함수 시그니처를 `pub`으로 변경**

```rust
// lib.rs:90 — 변경 전
fn run_capture_pipeline(app: tauri::AppHandle, tray: tauri::tray::TrayIcon) {

// lib.rs:90 — 변경 후
pub fn run_capture_pipeline(app: tauri::AppHandle, tray: tauri::tray::TrayIcon) {
```

- [ ] **Step 2: 빌드 확인**

Run: `cd src-tauri && cargo check 2>&1`
Expected: 경고만 있고 에러 없음 (기존 warnings은 P1 범위)

- [ ] **Step 3: 커밋**

```bash
git add src-tauri/src/lib.rs
git commit -m "refactor: make run_capture_pipeline pub for tray access"
```

---

### Task 2: 트레이 "캡처" 클릭 → `run_capture_pipeline` 호출

**Files:**
- Modify: `src-tauri/src/tray.rs:43-54` — `on_menu_event` 클로저

- [ ] **Step 1: "capture" 분기 수정 — emit 제거, 직접 호출**

`on_menu_event` 클로저 안에서 `app.tray_by_id("main")`으로 트레이를 가져와 `run_capture_pipeline`을 새 스레드에서 호출한다.

```rust
"capture" => {
    let app_handle = app.clone();
    if let Some(tray) = app.tray_by_id("main") {
        std::thread::spawn(move || {
            crate::run_capture_pipeline(app_handle, tray);
        });
    }
}
```

**주의:** `tray_by_id("main")`을 사용하려면 TrayIconBuilder에 `.id("main")`을 추가해야 한다.

- [ ] **Step 2: TrayIconBuilder에 `.id("main")` 추가**

```rust
// tray.rs:38 — 변경 전
let tray = TrayIconBuilder::new()

// tray.rs:38 — 변경 후
let tray = TrayIconBuilder::new()
    .id("main")
```

- [ ] **Step 3: 빌드 확인**

Run: `cd src-tauri && cargo check 2>&1`
Expected: 에러 없음

- [ ] **Step 4: 수동 테스트**

Run: `cd src-tauri && cargo tauri dev`
테스트: 트레이 우클릭 → "캡처" 클릭 → 오버레이 나타나는지 확인

- [ ] **Step 5: 커밋**

```bash
git add src-tauri/src/tray.rs
git commit -m "feat: wire tray capture menu to run_capture_pipeline"
```

---

### Task 3: 트레이 "자동 실행" 토글 → `config::set_auto_start` + config 동기화

**Files:**
- Modify: `src-tauri/src/tray.rs:47-49` — `"auto_start"` 분기

**핵심 포인트:**
- muda가 CheckMenuItem 클릭 시 **자동으로 체크 상태를 토글**함
- `on_menu_event` 호출 시점에 `is_checked()`는 이미 **새 값** 반환
- `TrayIcon`에 `menu()` getter가 없으므로, `auto_start_item`을 클로저에 **직접 캡처**해야 함

- [ ] **Step 1: `auto_start_item`을 클로저에 캡처하도록 구조 변경**

`setup_tray` 함수에서 `auto_start_item`을 clone하여 `on_menu_event` 클로저에 move로 캡처한다.

```rust
// tray.rs — setup_tray 함수, TrayIconBuilder 직전에 clone 추가
let auto_start_clone = auto_start_item.clone();

let tray = TrayIconBuilder::new()
    .id("main")
    .icon(Image::from_bytes(include_bytes!("../icons/icon.ico"))?)
    .tooltip("TextSniper")
    .menu(&menu)
    .show_menu_on_left_click(false)
    .on_menu_event(move |app, event| match event.id.as_ref() {
        "capture" => {
            let app_handle = app.clone();
            if let Some(tray) = app.tray_by_id("main") {
                std::thread::spawn(move || {
                    crate::run_capture_pipeline(app_handle, tray);
                });
            }
        }
        "auto_start" => {
            // muda가 이미 체크 상태를 토글했으므로 is_checked()는 새 값
            let new_state = auto_start_clone.is_checked().unwrap_or(false);
            match crate::config::set_auto_start(new_state) {
                Ok(()) => {
                    // config.json 동기화
                    let mut cfg = crate::config::AppConfig::load();
                    cfg.auto_start = new_state;
                    let _ = cfg.save();
                }
                Err(e) => {
                    // 레지스트리 실패 시 체크 상태 되돌리기
                    let _ = auto_start_clone.set_checked(!new_state);
                    eprintln!("[tray] auto_start error: {}", e);
                }
            }
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    })
    .build(app)?;
```

- [ ] **Step 2: 빌드 확인**

Run: `cd src-tauri && cargo check 2>&1`
Expected: 에러 없음

- [ ] **Step 3: 수동 테스트**

Run: `cd src-tauri && cargo tauri dev`
테스트 순서:
1. 트레이 우클릭 → "자동 실행" 클릭 → 체크마크 표시 확인
2. 다시 클릭 → 체크마크 해제 확인
3. 레지스트리 확인: `reg query "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v TextSniperWin`
4. 앱 재시작 → 체크 상태 유지 확인

- [ ] **Step 4: 커밋**

```bash
git add src-tauri/src/tray.rs
git commit -m "feat: wire tray auto-start toggle to registry and config"
```

---

### Task 4: 정리 — 미사용 emit 제거 및 최종 검증

**Files:**
- Modify: `src-tauri/src/tray.rs` — 불필요한 `Emitter` import 제거

- [ ] **Step 1: 미사용 import 정리**

Task 2, 3 완료 후 `app.emit()` 호출이 모두 제거되었으므로:

```rust
// tray.rs:1 — 변경 전
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    image::Image,
    AppHandle, Emitter,
};

// tray.rs:1 — 변경 후
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    image::Image,
    AppHandle,
};
```

- [ ] **Step 2: cargo check로 Emitter가 실제로 미사용인지 확인**

Run: `cd src-tauri && cargo check 2>&1 | grep -i emitter`
Expected: 관련 에러/경고 없음

- [ ] **Step 3: 전체 테스트 실행**

Run: `cd src-tauri && cargo test 2>&1`
Expected: 28개 테스트 전부 통과

- [ ] **Step 4: 수동 통합 테스트**

1. `cargo tauri dev`로 앱 실행
2. Shift+Alt+T → 캡처 동작 확인 (기존 기능 회귀 없음)
3. 트레이 우클릭 → "캡처" → 캡처 동작 확인
4. 트레이 우클릭 → "자동 실행" → 토글 동작 확인
5. 트레이 우클릭 → "종료" → 앱 종료 확인

- [ ] **Step 5: 최종 커밋**

```bash
git add src-tauri/src/tray.rs
git commit -m "chore: remove unused Emitter import"
```
