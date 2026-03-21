# TextSniper 다음 세션 TODO

## P0 — 기능 미작동 수정 ✅
- [x] 트레이 메뉴 "캡처" 클릭 → `run_capture_pipeline` 직접 호출
- [x] 트레이 메뉴 "자동 실행" 토글 → `config::set_auto_start()` + config.json 동기화
- [x] CheckMenuItem 체크 상태 토글 반영 (muda 자동 토글 활용)

## P1 — 코드 정리 ✅
- [x] `eprintln!` 디버그 로그 → `debug_log!` 매크로 (debug 빌드 전용)
- [x] cargo warnings 11개 정리 → 0개 (unused BOOL, imports, dead_code)
- [x] `TrayManager::new()`, `state` 필드 미사용 코드 제거 → unit struct

## P2 — 아이콘 & UI ✅
- [x] placeholder 아이콘 → macOS 스타일 미니멀 단색 아이콘 (16x16+32x32)
- [x] ~~트레이 상태 아이콘(loading/success/error)~~ 제거 — "의미없음" 결정
- [x] ~~오버레이 커서 툴팁~~ 제거 — "의미없음" 결정

## P3 — 릴리즈 준비 ✅
- [x] `tauri.conf.json` 릴리즈 설정 (버전, 설명, 퍼블리셔)
- [x] NSIS/WiX 인스톨러 구성 → NSIS 선택 (Tauri v2 기본, WiX 대비 설정 간편)
- [x] WebView2 런타임 번들링 결정: embedBootstrapper (Win10 대응)
- [x] 앱 아이콘 (taskbar, 설치 프로그램용)

## P3.5 — 배포 인프라
- [ ] 코드 서명 — EV 인증서 적용 시 SmartScreen 경고 제거 (연 $200~400)
- [ ] 자동 업데이트 — `tauri-plugin-updater` + GitHub Releases 연동, 후속 버전 자동 배포

## P4 — 기능 확장 (MVP 범위 외)
- [ ] 복사 전 미리보기 팝업
- [ ] 줄바꿈 자동 정리 옵션
- [ ] 단축키 커스터마이징 (설정 UI)
- [ ] 멀티 모니터 전체 커버
- [ ] 히스토리 저장

## 참고 파일
- 스펙: `doc/SPEC.md`
- 구현 계획: `doc/plans/2026-03-20-textsniper-win.md`
- 프로젝트 컨텍스트: `CLAUDE.md`
- v1.0.0 릴리즈: https://github.com/nicewook/textSniperWin/releases/tag/v1.0.0
