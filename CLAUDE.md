# TextSniper for Windows - CLAUDE.md

## 프로젝트 개요

Windows 화면 영역을 캡처하여 OCR로 텍스트를 인식하고 클립보드에 복사하는 시스템 트레이 상주 앱.
단축키: `Shift+Alt+T`

**상세 스펙:** `doc/SPEC.md`
**구현 계획:** `doc/plans/2026-03-20-textsniper-win.md`

## 기술 스택

- **언어:** Rust (Tauri v2)
- **오버레이:** Win32 API 직접 구현 (`CreateWindowEx` + `WS_EX_LAYERED`)
- **화면 캡처:** GDI `BitBlt`
- **OCR:** Windows OCR API (`windows` crate 0.62, `Windows.Media.Ocr`)
- **클립보드:** `arboard` 3
- **단축키:** `tauri-plugin-global-shortcut` 2

## 소스 구조

```
src-tauri/src/
├── lib.rs            # Tauri 앱 진입점, 파이프라인 조율
├── main.rs           # binary entry
├── config.rs         # 설정 로드/저장 (JSON, AppData/Roaming)
├── single_instance.rs# Named Mutex로 단일 인스턴스 보장
├── capture.rs        # GDI BitBlt 화면 캡처 → RGBA 픽셀
├── ocr.rs            # WinRT OCR API 래퍼, 언어 감지
├── clipboard.rs      # arboard 클립보드 쓰기
├── overlay.rs        # Win32 반투명 선택 오버레이 창
├── tray.rs           # 트레이 아이콘 + 메뉴 이벤트 처리 (캡처/자동실행/종료)
└── debug_log.rs      # debug_log! 매크로 (debug 빌드 전용 eprintln)
```

## 핵심 파이프라인 (`lib.rs::run_capture_pipeline`)

1. 현재 커서 위치 모니터 감지 (`overlay::get_current_monitor`)
2. Win32 오버레이 창으로 영역 선택 (`overlay::show_overlay`)
3. 오버레이 닫힘 후 150ms 대기 (화면 복구)
4. GDI BitBlt로 캡처 (`capture::capture_screen_region`)
5. 전용 스레드에서 WinRT OCR 실행 (5초 타임아웃, COM MTA 초기화 필수)
6. 클립보드 복사 (`clipboard::copy_to_clipboard`)

## 주요 Known Issues & 버그 수정 이력

- **COM 초기화:** WinRT OCR 호출 스레드에서 반드시 `CoInitializeEx(COINIT_MULTITHREADED)` 호출해야 함
- **DPI 이중 스케일링:** 캡처 좌표는 물리 픽셀 기준. 오버레이에서 DPI 스케일 적용 후 캡처 시 재스케일 금지
- **캡처 타이밍:** 오버레이 닫힌 직후 캡처 시 오버레이가 찍힘 → 150ms sleep 필요
- **단일 인스턴스:** 개발 시 구 프로세스 kill 후 테스트 필수 (`taskkill /F /IM text-sniper-win.exe`)
- **Tauri emit/listen:** 트레이 전용 앱(WebView 없음)에서 emit은 무의미. 직접 함수 호출 사용
- **muda CheckMenuItem:** 클릭 시 자동 토글됨. 핸들러 내 is_checked()는 이미 새 값. set_checked 수동 호출 금지 (에러 복구 시만)

## 빌드 & 실행

```bash
# 개발 실행 (npx 방식 — cargo-tauri 미설치 환경)
npx @tauri-apps/cli dev

# 또는 cargo-tauri가 PATH에 있으면
cd src-tauri && cargo tauri dev

# 빌드
cd src-tauri && cargo tauri build
```

## 테스트

```bash
cd src-tauri && cargo test
```

총 30개 테스트 (각 모듈 단위 테스트 포함).
