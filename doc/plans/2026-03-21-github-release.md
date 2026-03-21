# GitHub v1.0.0 릴리즈 배포 계획

**Goal:** TextSniper v1.0.0 인스톨러를 GitHub Releases에 배포하여 누구나 다운로드/설치 가능하게 한다.

**현재 상태:**
- GitHub repo: `nicewook/textSniperWin` (public)
- gh CLI 인증 완료
- 인스톨러: `src-tauri/target/release/bundle/nsis/TextSniper_1.0.0_x64-setup.exe` (3.5MB)
- 태그: 없음
- 미커밋 파일: `Cargo.lock` 변경, P3 계획 문서, 불필요한 android/ios 아이콘

---

## Task 1: 릴리즈 전 정리

### Step 1: 불필요한 파일 정리

`cargo tauri icon`이 자동 생성한 android/ios 아이콘은 Windows 전용 프로젝트에서 불필요.

```bash
# .gitignore에 추가
echo "src-tauri/icons/android/" >> .gitignore
echo "src-tauri/icons/ios/" >> .gitignore
```

### Step 2: 미커밋 변경사항 커밋

```bash
# Cargo.lock 변경 (version 1.0.0 반영)
git add src-tauri/Cargo.lock .gitignore
git commit -m "chore: update Cargo.lock for v1.0.0 and gitignore mobile icons"

# P3 계획 문서
git add doc/plans/2026-03-21-p3-release-prep.md
git commit -m "docs: add P3 release preparation plan"
```

### Step 3: remote에 push

```bash
git push origin master
```

---

## Task 2: GitHub Release 생성

### Step 1: v1.0.0 태그 생성 + Release 생성 + 인스톨러 업로드

```bash
gh release create v1.0.0 \
  "src-tauri/target/release/bundle/nsis/TextSniper_1.0.0_x64-setup.exe" \
  --title "TextSniper v1.0.0" \
  --notes "$(cat <<'EOF'
## TextSniper v1.0.0 — 첫 릴리즈

Windows 화면의 특정 영역을 캡처하여 OCR로 텍스트를 인식하고 클립보드에 복사하는 경량 데스크톱 앱.

### 주요 기능
- **단축키 `Shift+Alt+T`**로 빠른 캡처
- Windows OCR API 활용 (영어 + 한국어)
- 시스템 트레이 상주, 자동 실행 지원
- NSIS 인스톨러 (WebView2 bootstrapper 포함, Win10/11 지원)

### 설치 방법
1. `TextSniper_1.0.0_x64-setup.exe` 다운로드
2. 실행하여 설치 (관리자 권한 불필요)
3. 시스템 트레이에서 TextSniper 아이콘 확인
4. `Shift+Alt+T`로 캡처 시작

### 시스템 요구사항
- Windows 10 버전 1903 이상
- 한국어 OCR: Windows 한국어 언어팩 필요

### 알려진 제한사항
- 관리자 권한 앱 화면 캡처 불가
- DRM 보호 콘텐츠 캡처 불가
- 미서명 인스톨러 — SmartScreen 경고 발생 가능 ("자세한 정보" → "실행")
EOF
)"
```

### Step 2: 릴리즈 확인

```bash
gh release view v1.0.0
```

확인 사항:
- Release 페이지에 인스톨러 파일 표시
- Release notes 정상 렌더링
- 다운로드 링크 동작

---

## Task 3: 검증

### Step 1: 다운로드 테스트

```bash
# 임시 디렉토리에 다운로드
gh release download v1.0.0 -D /tmp/textsniper-release
ls -la /tmp/textsniper-release/
```

### Step 2: 파일 무결성 확인

```bash
# 원본과 다운로드 파일 크기 비교
ls -la src-tauri/target/release/bundle/nsis/TextSniper_1.0.0_x64-setup.exe
ls -la /tmp/textsniper-release/TextSniper_1.0.0_x64-setup.exe
```

### Step 3: README 업데이트 (선택)

GitHub repo 메인 페이지에 설치 안내 추가 고려.
