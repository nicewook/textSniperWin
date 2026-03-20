# P1: 코드 정리 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 디버그 로그를 조건부로 전환하고, cargo warnings 11개를 모두 제거하여 깨끗한 빌드를 달성한다.

**Architecture:** 매크로 기반 조건부 로깅 + 개별 warning 수정. 변경은 기계적이고 독립적이므로 순차 실행.

**Tech Stack:** Rust, Tauri v2, windows crate 0.62

---

## 현재 상황

| 항목 | 수량 | 위치 |
|------|------|------|
| `eprintln!` 디버그 로그 | 31개 | lib.rs(22), ocr.rs(6), tray.rs(3) |
| unused `BOOL` must_use | 8개 | overlay.rs |
| unused import `HWND` | 1개 | capture.rs:2 |
| dead_code: `state` 필드 | 1개 | tray.rs:18 |
| dead_code: `new()` 함수 | 1개 | tray.rs:22 |

---

### Task 1: 조건부 디버그 로그 매크로 도입

**Files:**
- Create: `src-tauri/src/debug_log.rs` — 매크로 정의
- Modify: `src-tauri/src/lib.rs` — `mod debug_log;` 추가 + `eprintln!` → `debug_log!` 교체
- Modify: `src-tauri/src/ocr.rs` — `eprintln!` → `debug_log!` 교체
- Modify: `src-tauri/src/tray.rs` — 에러 로그는 `eprintln!` 유지, 디버그 로그만 교체

**설계 결정:**
- `tray.rs`의 에러 로그 (`[tray] auto_start error`, `[tray] config save error`, `[tray] could not find tray`)는 **릴리즈에서도 유지** — 실제 에러 상황
- `lib.rs`의 `[pipeline]` 로그와 `ocr.rs`의 `[ocr-thread]` 로그는 **디버그 전용**

- [ ] **Step 1: `debug_log.rs` 매크로 파일 생성**

```rust
/// 디버그 빌드에서만 출력되는 로그 매크로
macro_rules! debug_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        eprintln!($($arg)*);
    };
}

pub(crate) use debug_log;
```

- [ ] **Step 2: `lib.rs`에 모듈 등록 + `eprintln!` → `debug_log!` 교체**

`mod debug_log;` 추가. `lib.rs`의 22개 `eprintln!` 중:
- `"TextSniper is already running."` (line 19) → `eprintln!` 유지 (사용자 피드백)
- `"Failed to register shortcut"` (line 81) → `eprintln!` 유지 (에러)
- 나머지 `[pipeline]` 로그 20개 → `debug_log::debug_log!` 교체

- [ ] **Step 3: `ocr.rs`의 `eprintln!` → `debug_log!` 교체**

파일 상단에 `use crate::debug_log::debug_log;` import 추가.
6개 전부 `[ocr-thread]` 디버그 로그이므로 모두 교체.

- [ ] **Step 4: 빌드 확인**

Run: `cd src-tauri && cargo check 2>&1`
Expected: 에러 없음

- [ ] **Step 5: 커밋**

```bash
git add src-tauri/src/debug_log.rs src-tauri/src/lib.rs src-tauri/src/ocr.rs
git commit -m "refactor: conditional debug logging with debug_log! macro"
```

---

### Task 2: cargo warnings 정리 — overlay.rs unused BOOL (8개)

**Files:**
- Modify: `src-tauri/src/overlay.rs` — `let _ =` 추가

- [ ] **Step 1: 8개 unused BOOL 경고에 `let _ =` 추가**

대상 라인:
- `overlay.rs:104` — `InvalidateRect`
- `overlay.rs:148` — `DeleteObject(dark_brush)`
- `overlay.rs:161` — `DeleteObject(sel_brush)`
- `overlay.rs:177` — `DeleteObject(border_brush)`
- `overlay.rs:180` — `EndPaint`
- `overlay.rs:246` — `ShowWindow`
- `overlay.rs:247` — `SetForegroundWindow`
- `overlay.rs:252` — `TranslateMessage`

각각 `let _ = ...;` 패턴으로 감싸기.

- [ ] **Step 2: 빌드 확인**

Run: `cd src-tauri && cargo check 2>&1 | grep "warning:"`
Expected: BOOL 관련 경고 0개

- [ ] **Step 3: 커밋**

```bash
git add src-tauri/src/overlay.rs
git commit -m "fix: suppress unused BOOL warnings in overlay.rs"
```

---

### Task 3: cargo warnings 정리 — capture.rs unused import + tray.rs dead_code

**Files:**
- Modify: `src-tauri/src/capture.rs:2` — `use windows::Win32::Foundation::HWND;` 제거
- Modify: `src-tauri/src/tray.rs:17-26` — `state` 필드 + `new()` 함수 제거

- [ ] **Step 1: capture.rs에서 미사용 HWND import 제거**

```rust
// capture.rs:2 — 제거
use windows::Win32::Foundation::HWND;
```

- [ ] **Step 2: tray.rs에서 TrayManager의 `state` 필드와 `new()` 제거**

```rust
// 변경 전
pub struct TrayManager {
    state: Mutex<TrayState>,
}

impl TrayManager {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(TrayState::Idle),
        }
    }

// 변경 후
pub struct TrayManager;

impl TrayManager {
```

또한 `use std::sync::Mutex;` import도 제거 (더 이상 사용 안 함).

- [ ] **Step 3: 빌드 확인 — warning 0개 목표**

Run: `cd src-tauri && cargo check 2>&1 | grep "^warning:" | grep -v "generated"`
Expected: 출력 없음 (0 warnings)

- [ ] **Step 4: 테스트 확인**

Run: `cd src-tauri && cargo test 2>&1 | tail -5`
Expected: 28 passed

- [ ] **Step 5: 커밋**

```bash
git add src-tauri/src/capture.rs src-tauri/src/tray.rs
git commit -m "fix: remove unused import, dead code in TrayManager"
```
