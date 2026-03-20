use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    image::Image,
    AppHandle,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrayState {
    Idle,
    Loading,
    Success,
    Error,
}

pub struct TrayManager;

impl TrayManager {
    /// Tauri setup 단계에서 호출. 트레이 아이콘과 메뉴 생성.
    pub fn setup_tray(app: &AppHandle) -> Result<TrayIcon, Box<dyn std::error::Error>> {
        let capture_item =
            MenuItem::with_id(app, "capture", "캡처 (Shift+Alt+T)", true, None::<&str>)?;
        let is_auto = crate::config::is_auto_start_enabled();
        let auto_start_item =
            CheckMenuItem::with_id(app, "auto_start", "자동 실행", true, is_auto, None::<&str>)?;
        let quit_item = MenuItem::with_id(app, "quit", "종료", true, None::<&str>)?;
        let menu = Menu::with_items(app, &[&capture_item, &auto_start_item, &quit_item])?;

        let auto_start_clone = auto_start_item.clone();

        let tray = TrayIconBuilder::with_id("main")
            .icon(Image::from_bytes(include_bytes!("../icons/icon.ico"))?)
            .tooltip("TextSniper")
            .menu(&menu)
            .show_menu_on_left_click(false)
            .on_menu_event(move |app: &AppHandle, event: tauri::menu::MenuEvent| match event.id.as_ref() {
                "capture" => {
                    let app_handle = app.clone();
                    if let Some(tray) = app.tray_by_id("main") {
                        std::thread::spawn(move || {
                            crate::run_capture_pipeline(app_handle, tray);
                        });
                    } else {
                        eprintln!("[tray] could not find tray 'main'");
                    }
                }
                "auto_start" => {
                    let new_state = auto_start_clone.is_checked().unwrap_or(false);
                    match crate::config::set_auto_start(new_state) {
                        Ok(()) => {
                            let mut cfg = crate::config::AppConfig::load();
                            cfg.auto_start = new_state;
                            if let Err(e) = cfg.save() {
                                eprintln!("[tray] config save error: {}", e);
                            }
                        }
                        Err(e) => {
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

        Ok(tray)
    }

    /// 트레이 아이콘 상태 변경
    pub fn set_state(
        tray: &TrayIcon,
        state: TrayState,
        _app: &AppHandle,
    ) -> Result<(), Box<dyn std::error::Error>> {
        tray.set_icon(Some(match state {
            TrayState::Idle => Image::from_bytes(include_bytes!("../icons/icon.ico"))?,
            TrayState::Loading => Image::from_bytes(include_bytes!("../icons/icon-loading.ico"))?,
            TrayState::Success => Image::from_bytes(include_bytes!("../icons/icon-success.ico"))?,
            TrayState::Error => Image::from_bytes(include_bytes!("../icons/icon-error.ico"))?,
        }))?;
        Ok(())
    }

    /// 성공/실패 아이콘 표시 후 1초 뒤 idle로 복귀
    pub fn flash_state(tray: TrayIcon, state: TrayState, app: AppHandle) {
        let _ = Self::set_state(&tray, state, &app);
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(1));
            let _ = Self::set_state(&tray, TrayState::Idle, &app);
        });
    }
}
