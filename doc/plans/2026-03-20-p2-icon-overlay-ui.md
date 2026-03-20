# P2: 아이콘 & 오버레이 UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 미니멀 단색 트레이 아이콘 4개를 생성하고, 오버레이에 커서 근처 안내 툴팁을 추가한다.

**Architecture:** 아이콘은 Rust 빌드 스크립트(build.rs)로 생성하거나 수동 생성 후 교체. 오버레이 툴팁은 WM_PAINT에서 GDI `DrawTextW`로 커서 근처에 안내 텍스트를 렌더링. 드래그 시작 전에만 표시하고, 드래그 중에는 숨긴다.

**Tech Stack:** Rust, Win32 GDI (DrawTextW, CreateFontW), ICO 파일 생성 (`image` crate 또는 수동)

---

## 파일 맵

- **Create:** `src-tauri/icons/icon.ico` — 메인 트레이 아이콘 (교체)
- **Create:** `src-tauri/icons/icon-loading.ico` — 로딩 상태 (교체)
- **Create:** `src-tauri/icons/icon-success.ico` — 성공 상태 (교체)
- **Create:** `src-tauri/icons/icon-error.ico` — 에러 상태 (교체)
- **Modify:** `src-tauri/src/overlay.rs` — WM_PAINT에 커서 근처 툴팁 렌더링 추가

---

**병렬 실행:** Task 1(아이콘)과 Task 2(툴팁)는 서로 다른 파일을 건드리므로 병렬 실행 가능.

---

### Task 1: 미니멀 단색 트레이 아이콘 4개 생성

**Files:**
- Create/Replace: `src-tauri/icons/icon.ico`
- Create/Replace: `src-tauri/icons/icon-loading.ico`
- Create/Replace: `src-tauri/icons/icon-success.ico`
- Create/Replace: `src-tauri/icons/icon-error.ico`

**디자인 스펙:**
- 크기: 32x32 (시스템 트레이 표준)
- 스타일: 미니멀 단색, 배경 투명
- icon.ico: 흰색 "T" 글자 (텍스트 캡처를 상징) — 배경 투명
- icon-loading.ico: 흰색 "..." 또는 회전 표시 — 배경 투명
- icon-success.ico: 녹색 체크마크 "✓" — 배경 투명
- icon-error.ico: 빨간색 "✕" — 배경 투명

**접근법:** Python 스크립트로 ICO 생성 (Pillow 사용). 빌드 의존성을 추가하지 않기 위해 일회성 스크립트로 생성 후 아이콘 파일만 커밋.

- [ ] **Step 1: 아이콘 생성 스크립트 작성 및 실행**

Python + Pillow로 4개 ICO 파일 생성. 스크립트는 커밋하지 않음 (일회성).

```python
# scripts/generate_icons.py
from PIL import Image, ImageDraw, ImageFont
import os

ICON_DIR = os.path.join(os.path.dirname(__file__), '..', 'src-tauri', 'icons')
SIZE = 32

def create_icon(name, draw_fn):
    img = Image.new('RGBA', (SIZE, SIZE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    draw_fn(draw, img)
    path = os.path.join(ICON_DIR, name)
    img.save(path, format='ICO', sizes=[(16, 16), (SIZE, SIZE)])
    print(f"Created {path}")

def draw_main(draw, img):
    """흰색 T 글자"""
    try:
        font = ImageFont.truetype("arial.ttf", 26)
    except:
        font = ImageFont.load_default()
    draw.text((8, 1), "T", fill=(255, 255, 255, 255), font=font)

def draw_loading(draw, img):
    """흰색 ... 점 3개"""
    for i, x in enumerate([8, 14, 20]):
        draw.ellipse([x, 14, x+4, 18], fill=(255, 255, 255, 200))

def draw_success(draw, img):
    """녹색 체크마크"""
    draw.line([(8, 16), (14, 22), (24, 8)], fill=(0, 200, 0, 255), width=3)

def draw_error(draw, img):
    """빨간색 X"""
    draw.line([(8, 8), (24, 24)], fill=(220, 50, 50, 255), width=3)
    draw.line([(24, 8), (8, 24)], fill=(220, 50, 50, 255), width=3)

create_icon('icon.ico', draw_main)
create_icon('icon-loading.ico', draw_loading)
create_icon('icon-success.ico', draw_success)
create_icon('icon-error.ico', draw_error)
```

Run: `pip install Pillow && python scripts/generate_icons.py`

- [ ] **Step 2: 생성된 아이콘 파일 확인**

각 ICO 파일이 생성되었고 32x32 크기인지 확인.

- [ ] **Step 3: 빌드 확인**

Run: `cd src-tauri && cargo check 2>&1`
Expected: 아이콘이 `include_bytes!`로 참조되므로 빌드 성공 확인

- [ ] **Step 4: 수동 테스트**

앱 실행 후 트레이 아이콘이 "T"로 표시되는지 확인.
캡처 실행 후 success 아이콘(녹색 체크) 표시 확인.

- [ ] **Step 5: 커밋**

```bash
git add src-tauri/icons/icon.ico src-tauri/icons/icon-loading.ico src-tauri/icons/icon-success.ico src-tauri/icons/icon-error.ico
git commit -m "feat: replace placeholder tray icons with minimal mono design"
```

---

### Task 2: 오버레이 커서 근처 안내 툴팁

**Files:**
- Modify: `src-tauri/src/overlay.rs:76-82` — thread_local에 MOUSE_POS 추가
- Modify: `src-tauri/src/overlay.rs:99-106` — WM_MOUSEMOVE에서 마우스 위치 항상 저장
- Modify: `src-tauri/src/overlay.rs:138-181` — WM_PAINT에서 툴팁 렌더링

**설계:**
- 드래그 시작 전: 커서 근처에 "드래그하여 선택 / ESC 취소" 텍스트 표시
- 드래그 중: 툴팁 숨김 (선택 영역만 표시)
- 텍스트: Win32 GDI `DrawTextW` + `CreateFontW`
- 배경: 반투명 다크 박스, 흰색 텍스트
- 위치: 커서 오른쪽 아래 20px 오프셋 (화면 밖으로 나가면 보정)

- [ ] **Step 1: thread_local에 MOUSE_POS 추가**

```rust
// overlay.rs thread_local! 블록에 추가
static MOUSE_POS: Cell<(i32, i32)> = const { Cell::new((0, 0)) };
```

- [ ] **Step 2: WM_MOUSEMOVE에서 마우스 위치 항상 저장**

현재 WM_MOUSEMOVE는 드래그 중에만 좌표를 저장한다. 드래그 전에도 마우스 위치를 알아야 툴팁을 그릴 수 있으므로, `if` 블록 밖에서도 좌표를 저장하도록 변경.

```rust
WM_MOUSEMOVE => {
    let x = (lparam.0 & 0xFFFF) as i16 as i32;
    let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
    MOUSE_POS.set((x, y));
    if DRAG_START.get().is_some() {
        DRAG_CURRENT.set(Some((x, y)));
    }
    let _ = unsafe { InvalidateRect(Some(hwnd), None, true) };
    LRESULT(0)
}
```

**주의:** `InvalidateRect`를 `if` 밖으로 이동하여 드래그 전에도 화면을 갱신 (커서 이동 시 툴팁 위치 업데이트).

- [ ] **Step 3: WM_PAINT에서 툴팁 렌더링 구현**

WM_PAINT 핸들러의 마지막, `EndPaint` 직전에 툴팁 렌더링 추가.

```rust
// 드래그 전에만 툴팁 표시
if DRAG_START.get().is_none() {
    let (mx, my) = MOUSE_POS.get();
    let tip_str = "드래그하여 선택 / ESC 취소";
    let mut text_buf: Vec<u16> = tip_str.encode_utf16().collect();

    // 폰트 생성 (DPI 스케일 적용)
    let dpi = DPI_SCALE.get();
    let font_height = (16.0 * dpi) as i32;
    let font = unsafe {
        CreateFontW(
            font_height, 0, 0, 0,
            FW_NORMAL.0 as i32,
            0, 0, 0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            (DEFAULT_PITCH.0 | FF_SWISS.0) as u32,
            w!("Segoe UI"),
        )
    };
    let old_font = unsafe { SelectObject(hdc, font.into()) };

    // 텍스트 크기 측정
    let mut text_rect = RECT::default();
    unsafe {
        DrawTextW(
            hdc,
            &mut text_buf,
            &mut text_rect,
            DT_CALCRECT | DT_SINGLELINE,
        );
    }

    let tw = text_rect.right - text_rect.left;
    let th = text_rect.bottom - text_rect.top;
    let pad = 8;

    // 툴팁 위치 계산 (커서 오른쪽 아래 20px, 화면 밖 보정)
    let mut tx = mx + 20;
    let mut ty = my + 20;

    let mut client_rect2 = RECT::default();
    let _ = unsafe { GetClientRect(hwnd, &mut client_rect2) };
    if tx + tw + pad * 2 > client_rect2.right {
        tx = mx - tw - pad * 2 - 10;
    }
    if ty + th + pad * 2 > client_rect2.bottom {
        ty = my - th - pad * 2 - 10;
    }

    // 배경 박스
    let bg_rect = RECT {
        left: tx,
        top: ty,
        right: tx + tw + pad * 2,
        bottom: ty + th + pad * 2,
    };
    let bg_brush = unsafe { CreateSolidBrush(COLORREF(0x00202020)) };
    unsafe { FillRect(hdc, &bg_rect, bg_brush) };
    let _ = unsafe { DeleteObject(bg_brush.into()) };

    // 텍스트 그리기
    let mut draw_rect = RECT {
        left: tx + pad,
        top: ty + pad,
        right: tx + pad + tw,
        bottom: ty + pad + th,
    };
    unsafe {
        SetTextColor(hdc, COLORREF(0x00FFFFFF));
        SetBkMode(hdc, TRANSPARENT);
        DrawTextW(
            hdc,
            &mut text_buf,
            &mut draw_rect,
            DT_SINGLELINE,
        );
    }

    // 정리
    unsafe {
        SelectObject(hdc, old_font);
        let _ = DeleteObject(font.into());
    }
}
```

**참고:** 오버레이 전체에 `LWA_ALPHA(100)` ≈ 40% 불투명도가 적용되어 있으므로 툴팁도 반투명으로 보임. 가독성이 부족하면 alpha 값을 150~180으로 올리는 것을 고려.

- [ ] **Step 4: show_overlay에서 MOUSE_POS 초기화**

`show_overlay` 함수 상단, 다른 thread_local 리셋과 함께:

```rust
MOUSE_POS.set((0, 0));
```

- [ ] **Step 5: 필요한 Win32 import 추가**

overlay.rs 상단에 필요한 추가 import 확인:
- `DrawTextW`, `DT_CALCRECT`, `DT_SINGLELINE` — `windows::Win32::Graphics::Gdi::*`에 이미 포함
- `CreateFontW`, `FW_NORMAL`, `DEFAULT_CHARSET`, 등 — 역시 Gdi에 포함
- `SetTextColor`, `SetBkMode`, `TRANSPARENT` — Gdi에 포함
- `SelectObject` — Gdi에 포함

기존 `use windows::Win32::Graphics::Gdi::*;`로 충분할 가능성이 높지만, 컴파일 시 누락되는 것이 있으면 추가.

- [ ] **Step 6: 빌드 확인**

Run: `cd src-tauri && cargo check 2>&1`
Expected: 에러 없음, 새 경고 없음

- [ ] **Step 7: 수동 테스트**

Run: `cargo tauri dev` (또는 직접 exe 실행)
테스트:
1. Shift+Alt+T → 오버레이 표시 → 커서 근처에 "드래그하여 선택 / ESC 취소" 텍스트 보임
2. 마우스 이동 → 텍스트가 커서를 따라다님
3. 드래그 시작 → 텍스트 사라짐, 선택 영역만 표시
4. 화면 모서리에서 → 텍스트가 화면 안으로 보정됨

- [ ] **Step 8: 커밋**

```bash
git add src-tauri/src/overlay.rs
git commit -m "feat: add cursor tooltip guide text on overlay"
```

---

### Task 3: 최종 통합 검증

- [ ] **Step 1: cargo check — warning 0개 확인**

Run: `cd src-tauri && cargo check 2>&1 | grep "^warning:" | grep -v "generated"`
Expected: 출력 없음

- [ ] **Step 2: cargo test — 전체 통과 확인**

Run: `cd src-tauri && cargo test 2>&1 | tail -5`
Expected: 30 passed (또는 그 이상)

- [ ] **Step 3: 수동 통합 테스트**

1. 앱 실행
2. 트레이 아이콘: "T" 단색 아이콘 표시 확인
3. Shift+Alt+T → 오버레이 + 커서 툴팁 확인
4. 드래그 → 선택 영역 표시 (툴팁 숨김)
5. 릴리즈 → 캡처 → 트레이 success(녹색 체크) → 1초 후 "T" 복귀
6. 빈 영역 캡처 → 트레이 error(빨간 X) → 1초 후 복귀
7. ESC → 취소

- [ ] **Step 4: 최종 커밋 (TODO.md 업데이트)**

```bash
git add doc/TODO.md
git commit -m "docs: mark P2 complete"
```
