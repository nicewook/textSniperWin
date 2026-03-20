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
        .setup(move |app| {
            // 3. 트레이 아이콘 생성
            let tray = TrayManager::setup_tray(app.handle())?;

            // 4. 글로벌 단축키 등록
            setup_global_shortcut(app, tray.clone())?;

            // 5. 첫 실행 안내
            if config.first_run {
                tray.set_tooltip(Some("TextSniper - Shift+Alt+T로 캡처"))?;
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
    use tauri_plugin_global_shortcut::{
        Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
    };

    let shortcut = Shortcut::new(Some(Modifiers::SHIFT | Modifiers::ALT), Code::KeyT);

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

    // 5. OCR (스레드 + 채널, 5초 타임아웃)
    let pixels_clone = pixels;
    let width = rect.width;
    let height = rect.height;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let result = (|| {
            let bmp = ocr::create_bitmap_from_rgba(&pixels_clone, width, height)?;
            ocr::recognize_text(&bmp)
        })();
        let _ = tx.send(result);
    });

    let text = match rx.recv_timeout(std::time::Duration::from_secs(5)) {
        Ok(Ok(t)) => t,
        Ok(Err(_)) => {
            TrayManager::flash_state(tray, TrayState::Error, app);
            return;
        }
        Err(_) => {
            eprintln!("OCR timeout (5s exceeded)");
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
