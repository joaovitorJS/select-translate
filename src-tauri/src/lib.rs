mod captura;
mod traducao;

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager, WindowEvent,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use tauri_plugin_sql::{Migration, MigrationKind};
use tauri_plugin_store::StoreExt;

/// Usado só se a store ainda não tiver um atalho salvo (primeira execução).
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

/// Desregistra qualquer atalho anterior e registra `atalho` para
/// disparar a captura+tradução. Usado tanto na inicialização (com o
/// atalho salvo, ou o padrão na primeira execução) quanto pelo command
/// `registrar_atalho` quando o usuário troca o atalho nas Configurações.
fn registrar_atalho_no_backend(
    app: &AppHandle,
    atalho: &str,
) -> Result<(), tauri_plugin_global_shortcut::Error> {
    app.global_shortcut().unregister_all()?;
    app.global_shortcut()
        .on_shortcut(atalho, move |app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                captura::capturar_e_traduzir(app.clone());
            }
        })
}

/// Lê o atalho salvo em `config.json`, ou `ATALHO_PADRAO` se não houver
/// nenhum salvo ainda (primeira execução).
fn atalho_salvo(app: &AppHandle) -> String {
    app.store("config.json")
        .ok()
        .and_then(|store| store.get("atalho"))
        .and_then(|valor| valor.as_str().map(str::to_string))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| ATALHO_PADRAO.to_string())
}

/// Chamado pela tela de Configurações quando o usuário salva um novo
/// atalho. Falha de forma amigável (em vez de travar o app) quando a
/// combinação já está em uso por outro programa.
#[tauri::command]
fn registrar_atalho(app: AppHandle, atalho: String) -> Result<(), String> {
    registrar_atalho_no_backend(&app, &atalho).map_err(|erro| erro.to_string())?;

    if let Ok(store) = app.store("config.json") {
        store.set("atalho", serde_json::json!(atalho));
        let _ = store.save();
    }

    println!("[select-translate] Atalho global registrado: {atalho}");
    Ok(())
}

/// Chamado quando o campo de atalho nas Configurações ganha foco (o
/// usuário começou a "gravar" um novo atalho). Desregistra o atalho
/// atual temporariamente — senão, se o usuário apertar a combinação que
/// já está ativa (ou ela disparar por qualquer outro motivo enquanto o
/// campo está focado), o Ctrl+C simulado pela captura normal cai
/// direto nesse campo e é interpretado como se fosse a nova gravação.
#[tauri::command]
fn pausar_atalho_global(app: AppHandle) -> Result<(), String> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|erro| erro.to_string())
}

/// Chamado quando o campo de atalho perde o foco (gravação cancelada
/// ou terminada sem clicar em "Salvar" — quem salva de verdade é
/// `registrar_atalho`, que já re-registra com o valor novo). Volta a
/// registrar o atalho que estava salvo antes de começar a gravação.
#[tauri::command]
fn retomar_atalho_global(app: AppHandle) -> Result<(), String> {
    let atalho = atalho_salvo(&app);
    registrar_atalho_no_backend(&app, &atalho).map_err(|erro| erro.to_string())
}

/// Traz a janela principal para frente e dá foco a ela — usado tanto
/// pelo menu da bandeja quanto pela segunda instância do app (via
/// tauri-plugin-single-instance) e pelo command de autostart.
fn mostrar_janela_principal(app: &AppHandle) {
    if let Some(janela) = app.get_webview_window("main") {
        let _ = janela.show();
        let _ = janela.set_focus();
    }
}

/// Liga/desliga iniciar o app junto com o Windows. O estado "de
/// verdade" vive no registro do Windows (via tauri-plugin-autostart),
/// não em config.json — por isso o frontend consulta
/// `autostart_esta_ativo` em vez de guardar sua própria cópia.
#[tauri::command]
fn definir_autostart(app: AppHandle, ativo: bool) -> Result<(), String> {
    let gerenciador = app.autolaunch();
    if ativo {
        gerenciador.enable().map_err(|e| e.to_string())
    } else {
        gerenciador.disable().map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn autostart_esta_ativo(app: AppHandle) -> Result<bool, String> {
    app.autolaunch().is_enabled().map_err(|e| e.to_string())
}

/// Liga/desliga manter a janela sempre em cima das outras (nunca ficar
/// escondida atrás do que o usuário está usando). Aplica na janela
/// imediatamente e salva em config.json para reaplicar na próxima
/// abertura do app.
#[tauri::command]
fn definir_manter_no_topo(app: AppHandle, ativo: bool) -> Result<(), String> {
    if let Some(janela) = app.get_webview_window("main") {
        janela.set_always_on_top(ativo).map_err(|e| e.to_string())?;
    }

    if let Ok(store) = app.store("config.json") {
        store.set("manter_no_topo", serde_json::json!(ativo));
        let _ = store.save();
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Precisa ser o primeiro plugin registrado (exigência do
        // próprio tauri-plugin-single-instance).
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            mostrar_janela_principal(app);
        }))
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations(BANCO_HISTORICO, migracoes_banco())
                .build(),
        )
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .setup(|app| {
            let atalho = atalho_salvo(app.handle());

            if let Err(erro) = registrar_atalho_no_backend(&app.handle().clone(), &atalho) {
                println!("[select-translate] Falha ao registrar atalho '{atalho}': {erro}");
            } else {
                println!("[select-translate] Atalho global registrado: {atalho}");
            }

            captura::iniciar_monitoramento_automatico(app.handle().clone());

            // Ícone na bandeja com menu "Abrir"/"Sair" — o app continua
            // rodando em segundo plano mesmo com a janela fechada.
            let abrir = MenuItem::with_id(app, "abrir", "Abrir", true, None::<&str>)?;
            let sair = MenuItem::with_id(app, "sair", "Sair", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&abrir, &sair])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "abrir" => mostrar_janela_principal(app),
                    "sair" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            // Fechar a janela (X) esconde em vez de encerrar o processo
            // — o app continua vivo na bandeja.
            if let Some(janela) = app.get_webview_window("main") {
                let janela_para_esconder = janela.clone();
                janela.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = janela_para_esconder.hide();
                    }
                });

                let manter_no_topo_salvo = app
                    .store("config.json")?
                    .get("manter_no_topo")
                    .and_then(|valor| valor.as_bool())
                    .unwrap_or(false);
                if manter_no_topo_salvo {
                    let _ = janela.set_always_on_top(true);
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            registrar_atalho,
            pausar_atalho_global,
            retomar_atalho_global,
            definir_autostart,
            autostart_esta_ativo,
            definir_manter_no_topo
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
