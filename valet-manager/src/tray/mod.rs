use tray_icon::{menu::{Menu, MenuItem, PredefinedMenuItem, MenuEvent}, TrayIcon, TrayIconBuilder};
use tray_icon::Icon;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TrayEvent {
    Open,
    Quit,
    SwitchPhp(String),
    RestartServices,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum HealthState { Ok, Degraded, Down }

#[allow(dead_code)]
pub fn build_icon_image(state: HealthState) -> Icon {
    // 16×16 RGBA buffer with solid color circle on transparent background.
    let (r, g, b) = match state {
        HealthState::Ok       => (0x5D, 0xCA, 0xA5),
        HealthState::Degraded => (0xEF, 0x9F, 0x27),
        HealthState::Down     => (0xE2, 0x4B, 0x4A),
    };
    let w = 16; let h = 16;
    let mut rgba = vec![0u8; w * h * 4];
    let cx = w as f32 / 2.0; let cy = h as f32 / 2.0; let radius = 6.0;
    for y in 0..h {
        for x in 0..w {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            let idx = (y * w + x) * 4;
            if dist <= radius {
                rgba[idx] = r; rgba[idx+1] = g; rgba[idx+2] = b; rgba[idx+3] = 255;
            }
        }
    }
    Icon::from_rgba(rgba, w as u32, h as u32).expect("valid tray icon")
}

#[allow(dead_code)]
pub fn build_tray(active_php: &str, php_versions: &[String], health: HealthState) -> anyhow::Result<TrayIcon> {
    let menu = Menu::new();
    let _ = menu.append(&MenuItem::new(format!("PHP: {}", active_php), false, None));
    let _ = menu.append(&PredefinedMenuItem::separator());
    for v in php_versions {
        let _ = menu.append(&MenuItem::new(format!("Switch to {}", v), true, None));
    }
    let _ = menu.append(&PredefinedMenuItem::separator());
    let _ = menu.append(&MenuItem::new("Restart services", true, None));
    let _ = menu.append(&MenuItem::new("Open Valet Manager", true, None));
    let _ = menu.append(&PredefinedMenuItem::separator());
    let _ = menu.append(&MenuItem::new("Quit", true, None));

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Valet Manager")
        .with_icon(build_icon_image(health))
        .build()?;
    Ok(tray)
}

/// Drain menu events into a tokio mpsc::Sender<TrayEvent>.
/// Spawn this once at startup. Maps menu item labels back to TrayEvent variants.
#[allow(dead_code)]
pub fn spawn_menu_drain(tx: tokio::sync::mpsc::Sender<TrayEvent>) {
    let receiver = MenuEvent::receiver().clone();
    std::thread::spawn(move || {
        while let Ok(ev) = receiver.recv() {
            // We can't directly inspect the label here without storing menu IDs;
            // for now, just emit Open for any click. Real wiring is a follow-up.
            let _ = ev;
            let _ = tx.blocking_send(TrayEvent::Open);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn health_states_distinct() {
        assert_ne!(HealthState::Ok, HealthState::Down);
    }
    #[test]
    fn build_icon_image_ok_state_returns_icon() {
        // Must not panic
        let _ = build_icon_image(HealthState::Ok);
    }
}
