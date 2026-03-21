use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    image::Image,
    AppHandle,
};

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
                    std::thread::spawn(|| {
                        crate::run_capture_pipeline();
                    });
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
}
