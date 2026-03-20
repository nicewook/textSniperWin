# TextSniper 다음 세션 TODO

## P0 — 기능 미작동 수정
- [ ] 트레이 메뉴 "캡처" 클릭 이벤트 리스너 연결 (emit만 있고 수신 없음)
- [ ] 트레이 메뉴 "자동 실행" 토글 이벤트 → `config::set_auto_start()` 호출 연결
- [ ] CheckMenuItem 체크 상태 토글 반영

## P1 — 코드 정리
- [ ] `eprintln!` 디버그 로그 → `#[cfg(debug_assertions)]` 조건부 전환
- [ ] cargo warnings 12개 정리 (unused imports, unused_must_use, dead_code)
- [ ] `TrayManager::new()`, `state` 필드 미사용 코드 제거

## P2 — 아이콘 & UI
- [ ] placeholder 아이콘 4개 실제 디자인으로 교체 (icon, loading, success, error)
- [ ] 오버레이 안내 텍스트 ("드래그하여 선택 / ESC 취소")

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
