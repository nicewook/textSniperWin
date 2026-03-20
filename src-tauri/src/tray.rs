use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    image::Image,
    AppHandle, Emitter,
};
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrayState {
    Idle,
    Loading,
    Success,
    Error,
}

pub struct TrayManager {
    state: Mutex<TrayState>,
}

impl TrayManager {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(TrayState::Idle),
        }
    }

    /// Tauri setup 단계에서 호출. 트레이 아이콘과 메뉴 생성.
    pub fn setup_tray(app: &AppHandle) -> Result<TrayIcon, Box<dyn std::error::Error>> {
        let capture_item =
            MenuItem::with_id(app, "capture", "캡처 (Shift+Alt+T)", true, None::<&str>)?;
        let is_auto = crate::config::is_auto_start_enabled();
        let auto_start_item =
            CheckMenuItem::with_id(app, "auto_start", "자동 실행", true, is_auto, None::<&str>)?;
        let quit_item = MenuItem::with_id(app, "quit", "종료", true, None::<&str>)?;
        let menu = Menu::with_items(app, &[&capture_item, &auto_start_item, &quit_item])?;

        let tray = TrayIconBuilder::new()
            .icon(Image::from_bytes(include_bytes!("../icons/icon.ico"))?)
            .tooltip("TextSniper")
            .menu(&menu)
            .show_menu_on_left_click(false)
            .on_menu_event(|app, event| match event.id.as_ref() {
                "capture" => {
                    app.emit("trigger-capture", ()).ok();
                }
                "auto_start" => {
                    app.emit("toggle-auto-start", ()).ok();
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
