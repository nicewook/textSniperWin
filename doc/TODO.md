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
- [x] placeholder 아이콘 4개 → macOS 스타일 미니멀 단색 아이콘 (16x16+32x32 멀티사이즈)
- [x] 오버레이 커서 근처 툴팁 ("드래그하여 선택 / ESC 취소") + DPI 대응

## P3 — 릴리즈 준비
- [ ] `tauri.conf.json` 릴리즈 설정 (버전, 설명, 퍼블리셔)
- [ ] NSIS/WiX 인스톨러 구성
- [ ] WebView2 런타임 번들링 결정 (Win10 대응)
- [ ] 앱 아이콘 (taskbar, 설치 프로그램용)

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
