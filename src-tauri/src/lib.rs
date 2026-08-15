mod captura;
mod traducao;

use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// Fixo por enquanto — vira configurável na Fase 5.
const ATALHO_PADRAO: &str = "CommandOrControl+Alt+T";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            app.global_shortcut()
                .on_shortcut(ATALHO_PADRAO, move |app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        captura::capturar_e_traduzir(app.clone());
                    }
                })?;
            println!("[select-translate] Atalho global registrado: {ATALHO_PADRAO}");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
