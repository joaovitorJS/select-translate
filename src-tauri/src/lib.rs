mod captura;
mod traducao;

use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use tauri_plugin_sql::{Migration, MigrationKind};

/// Fixo por enquanto — vira configurável na Fase 5.
const ATALHO_PADRAO: &str = "CommandOrControl+Alt+T";

/// Mesmo caminho usado pelo frontend (src/historico.js) ao chamar os
/// commands `plugin:sql|execute`/`plugin:sql|select`.
const BANCO_HISTORICO: &str = "sqlite:historico.db";

fn migracoes_banco() -> Vec<Migration> {
    vec![Migration {
        version: 1,
        description: "cria tabela de historico",
        sql: "CREATE TABLE historico (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                texto_original TEXT NOT NULL,
                texto_traduzido TEXT NOT NULL,
                idioma_destino TEXT NOT NULL,
                criado_em TEXT NOT NULL
              );",
        kind: MigrationKind::Up,
    }]
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations(BANCO_HISTORICO, migracoes_banco())
                .build(),
        )
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
