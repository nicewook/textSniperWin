# P3 릴리즈 준비 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** TextSniper for Windows를 설치 가능한 릴리즈 바이너리로 빌드할 수 있도록 설정, 인스톨러, 아이콘을 완성한다.

**Architecture:** Tauri v2의 NSIS 번들러를 활용하여 Windows 인스톨러를 생성한다 (WiX 대비 NSIS가 Tauri v2 기본이며 언어 선택, 설치 경로 커스텀이 간편). WebView2는 트레이 전용 앱이지만 Tauri 런타임 의존성이므로 embedBootstrapper로 임베드한다 (Tauri 기본값 downloadBootstrapper 대비 오프라인 설치 시작 가능). 앱 아이콘은 `cargo tauri icon`으로 마스터 PNG에서 모든 필요 크기를 자동 생성한다.

**Note:** `productName`을 "TextSniperWin" → "TextSniper"로 변경하지만, 단일 인스턴스 뮤텍스는 `"Global\\TextSniperWin_SingleInstance"`로 하드코딩되어 있어 영향 없음.

**Tech Stack:** Tauri v2 bundle config (NSIS), ICO/PNG 아이콘 생성

---

## File Structure

```
src-tauri/
├── tauri.conf.json          # 수정: 릴리즈 메타데이터 + 번들 설정
├── Cargo.toml               # 수정: package 메타데이터
├── icons/
│   ├── icon.ico             # 교체: 앱 아이콘 (16~256px 멀티사이즈)
│   ├── icon.png             # 교체: 512x512 마스터 PNG
│   ├── 32x32.png            # 교체: 트레이용
│   ├── 128x128.png          # 교체
│   ├── 128x128@2x.png       # 교체: 256x256
│   ├── StoreLogo.png        # 교체: 50x50
│   ├── Square*Logo.png      # 교체: MSIX 타일 아이콘들
│   ├── icon.icns            # 자동생성: macOS (사용 안 하지만 빌드 참조)
│   └── app-icon-master.png  # 생성: 1024x1024 마스터 원본 (cargo tauri icon 입력)
```

---

### Task 1: tauri.conf.json 릴리즈 메타데이터 설정

**Files:**
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: 현재 설정 확인**

현재 상태:
```json
{
  "productName": "TextSniperWin",
  "version": "0.1.0",
  "identifier": "com.textsniper.win",
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/128x128@2x.png", "icons/icon.icns", "icons/icon.ico"]
  }
}
```

- [ ] **Step 2: 릴리즈 메타데이터 업데이트**

`tauri.conf.json`을 다음과 같이 수정:

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "TextSniper",
  "version": "1.0.0",
  "identifier": "com.textsniper.win",
  "build": {
    "frontendDist": "../src"
  },
  "app": {
    "withGlobalTauri": false,
    "windows": [],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": ["nsis"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "publisher": "정현석",
    "copyright": "Copyright © 2026 정현석",
    "shortDescription": "화면 영역 OCR 텍스트 캡처 도구",
    "longDescription": "Windows 화면의 특정 영역을 캡처하여 OCR로 텍스트를 인식하고 클립보드에 복사하는 경량 데스크톱 앱. 단축키 Shift+Alt+T로 빠르게 캡처할 수 있습니다.",
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": null,
      "timestampUrl": null,
      "webviewInstallMode": {
        "type": "embedBootstrapper"
      },
      "nsis": {
        "displayLanguageSelector": false,
        "installerIcon": "icons/icon.ico",
        "languages": ["Korean", "English"],
        "startMenuFolder": "TextSniper"
      }
    }
  }
}
```

변경 사항:
- `productName`: "TextSniperWin" → "TextSniper" (사용자에게 보이는 이름)
- `version`: "0.1.0" → "1.0.0"
- `targets`: "all" → `["nsis"]` (Windows 전용이므로 NSIS만)
- `publisher`, `copyright`, `shortDescription`, `longDescription` 추가
- `webviewInstallMode`: `embedBootstrapper` — Win10에서 WebView2 미설치 시 자동 다운로드/설치
- `nsis`: 인스톨러 아이콘, 언어, 시작 메뉴 폴더 설정

- [ ] **Step 3: 빌드 검증**

Run: `cd src-tauri && cargo tauri build 2>&1 | head -50`
Expected: 빌드 시작 성공 (아이콘 관련 경고는 Task 4에서 해결)

- [ ] **Step 4: Commit**

```bash
git add src-tauri/tauri.conf.json
git commit -m "chore: configure release metadata and NSIS installer settings"
```

---

### Task 2: Cargo.toml 패키지 메타데이터 정리

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 메타데이터 업데이트**

`Cargo.toml`의 `[package]` 섹션 수정:

```toml
[package]
name = "text-sniper-win"
version = "1.0.0"
description = "Screen region OCR text capture tool for Windows"
authors = ["정현석"]
edition = "2021"
license = "MIT"
```

변경:
- `version`: "0.1.0" → "1.0.0" (tauri.conf.json과 동기화)
- `description`: 실제 설명으로 변경
- `authors`: 실제 이름으로 변경
- `license` 추가

- [ ] **Step 2: 빌드 검증**

Run: `cd src-tauri && cargo check`
Expected: 경고/에러 없이 성공

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml
git commit -m "chore: update Cargo.toml package metadata for release"
```

---

### Task 3: WebView2 런타임 번들링 결정 및 설정

**Files:**
- Modify: `src-tauri/tauri.conf.json` (Task 1에서 이미 설정됨)

이 Task는 Task 1에서 `webviewInstallMode: embedBootstrapper`로 설정 완료.

배경 설명:
- **Windows 11**: WebView2가 OS에 기본 포함 → 추가 설치 불필요
- **Windows 10**: WebView2 미포함 가능 → bootstrapper가 설치 시 자동 다운로드
- `embedBootstrapper`: 인스톨러에 ~1.8MB bootstrapper를 임베드. 사용자가 오프라인이 아닌 한 동작
- `offlineInstaller`: ~160MB의 전체 WebView2 런타임 임베드 (오프라인 지원, 인스톨러 크기 큼)
- 결정: **embedBootstrapper** — 인스톨러 크기 최소화, Win10 사용자 대부분 온라인 환경

- [ ] **Step 1: 설정 확인**

Task 1 완료 후 `tauri.conf.json`에서 다음 확인:
```json
"webviewInstallMode": {
  "type": "embedBootstrapper"
}
```

- [ ] **Step 2: TODO.md 업데이트**

WebView2 결정을 TODO.md에 반영:
- `- [ ] WebView2 런타임 번들링 결정 (Win10 대응)` → `- [x] WebView2 런타임 번들링 결정: embedBootstrapper (Win10 대응)`

---

### Task 4: 앱 아이콘 생성 (taskbar, 인스톨러용)

**Files:**
- Modify: `src-tauri/icons/` 디렉토리 내 모든 아이콘 파일
- Reference: `src-tauri/src/tray.rs:24` (트레이 아이콘 로드 — `include_bytes!("../icons/icon.ico")`)

현재 `src-tauri/icons/`에는 Tauri 기본 placeholder 또는 이전에 만든 트레이 아이콘이 있음.
트레이 아이콘(32x32 단색)은 P2에서 완성됨. 이 Task에서는 **앱 아이콘** (taskbar, 인스톨러, 시작 메뉴)을 만든다.

**중요:** `icon.ico`는 트레이 아이콘(`tray.rs:24`)에서도 사용됨. 교체 후 트레이 표시 정상 확인 필요.

- [ ] **Step 1: 앱 아이콘 디자인 결정**

앱 아이콘 요구사항:
- 트레이 아이콘과 동일한 디자인 계열 (십자선 + T 심볼)
- 배경색: 브랜드 컬러 (파란색 계열 추천, 예: #2563EB)
- 크기: 1024x1024 이상 마스터 PNG 1장 준비
- `cargo tauri icon`이 나머지 모든 사이즈를 자동 생성

- [ ] **Step 2: 마스터 PNG (1024x1024) 생성**

이미지 편집 도구(Figma, Photoshop 등)로 1024x1024 PNG 생성.
저장: `src-tauri/icons/app-icon-master.png` (원본 보존용)

- [ ] **Step 3: `cargo tauri icon`으로 모든 아이콘 자동 생성**

```bash
cd src-tauri
npx @tauri-apps/cli icon icons/app-icon-master.png
```

이 명령은 `src-tauri/icons/` 아래에 다음을 자동 생성/덮어씀:
- `icon.ico` (멀티사이즈: 16~256px)
- `icon.icns` (macOS)
- `icon.png` (512x512)
- `32x32.png`, `128x128.png`, `128x128@2x.png`
- `Square*.png` (모든 MSIX 타일 사이즈)
- `StoreLogo.png`

- [ ] **Step 4: 트레이 아이콘 동작 확인**

`tray.rs:24`에서 `include_bytes!("../icons/icon.ico")`로 트레이 아이콘을 로드함.
`cargo tauri icon`이 `icon.ico`를 덮어쓰므로, 트레이에 새 아이콘이 표시됨.

Run: `npx @tauri-apps/cli dev`
확인: 시스템 트레이에 새 아이콘이 정상 표시되는지 확인

- [ ] **Step 5: 빌드 검증**

Run: `cd src-tauri && cargo tauri build`
Expected:
- NSIS 인스톨러 생성 성공
- 인스톨러에 올바른 아이콘 표시
- 출력 경로: `src-tauri/target/release/bundle/nsis/TextSniper_1.0.0_x64-setup.exe`

- [ ] **Step 6: 인스톨러 테스트**

수동 검증:
1. 생성된 `.exe` 인스톨러 실행
2. 설치 화면에 올바른 아이콘 표시 확인
3. 시작 메뉴에 "TextSniper" 폴더 생성 확인
4. 설치된 앱 실행 → 트레이 아이콘 정상 표시 확인
5. taskbar에 올바른 아이콘 확인
6. 프로그램 추가/제거에 올바른 이름, 퍼블리셔, 아이콘 확인

- [ ] **Step 7: Commit**

```bash
git add src-tauri/icons/
git commit -m "feat: add production app icons for installer and taskbar"
```

---

### Task 5: TODO.md 업데이트 및 최종 빌드 검증

**Files:**
- Modify: `doc/TODO.md`

- [ ] **Step 1: TODO.md P3 항목 완료 처리**

```markdown
## P3 — 릴리즈 준비 ✅
- [x] `tauri.conf.json` 릴리즈 설정 (버전, 설명, 퍼블리셔)
- [x] NSIS/WiX 인스톨러 구성 → NSIS 선택 (Tauri v2 기본, WiX 대비 설정 간편)
- [x] WebView2 런타임 번들링 결정: embedBootstrapper (Win10 대응)
- [x] 앱 아이콘 (taskbar, 설치 프로그램용)
```

- [ ] **Step 2: 클린 빌드 최종 검증**

```bash
cd src-tauri
cargo clean
cargo tauri build
```

Expected:
- 경고 0개
- `target/release/bundle/nsis/TextSniper_1.0.0_x64-setup.exe` 생성
- 인스톨러 크기: ~5-10MB (bootstrapper 포함)

- [ ] **Step 3: Commit**

```bash
git add doc/TODO.md
git commit -m "docs: mark P3 release preparation as complete"
```
