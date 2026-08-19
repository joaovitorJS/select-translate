mod captura;
mod popover;
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

/// Idem, para o atalho do popover (Melhoria — Popover de tradução).
const ATALHO_POPOVER_PADRAO: &str = "CommandOrControl+Alt+P";

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

/// Desregistra os atalhos anteriores e registra de novo os dois — o
/// principal (sempre) e o do popover (só se `popover_ativo` estiver
/// ligado na store). Precisa reconstruir os dois juntos porque
/// `unregister_all()` do plugin desregistra **todos** os atalhos
/// globais do processo, não só um — chamar isso pra atualizar um dos
/// dois (ex: usuário troca o atalho principal) apagaria o outro sem
/// querer se cada atalho fosse registrado de forma independente. Usado
/// na inicialização e por todo command que mexe em algum dos atalhos
/// (`registrar_atalho`, `registrar_atalho_popover`,
/// `pausar_atalho_global`/`retomar_atalho_global`, `definir_popover_ativo`).
fn sincronizar_atalhos_no_backend(
    app: &AppHandle,
) -> Result<(), tauri_plugin_global_shortcut::Error> {
    app.global_shortcut().unregister_all()?;

    let atalho = atalho_salvo(app);
    app.global_shortcut()
        .on_shortcut(atalho.as_str(), move |app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                captura::capturar_e_traduzir(app.clone());
            }
        })?;

    if popover_ativo(app) {
        let atalho_popover = atalho_popover_salvo(app);
        app.global_shortcut()
            .on_shortcut(atalho_popover.as_str(), move |app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    captura::capturar_e_traduzir_popover(app.clone());
                }
            })?;
    }

    Ok(())
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

/// Idem, para o atalho do popover.
fn atalho_popover_salvo(app: &AppHandle) -> String {
    app.store("config.json")
        .ok()
        .and_then(|store| store.get("atalho_popover"))
        .and_then(|valor| valor.as_str().map(str::to_string))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| ATALHO_POPOVER_PADRAO.to_string())
}

/// O popover vem desativado por padrão (igual ao modo automático) — o
/// usuário liga de propósito nas Configurações.
fn popover_ativo(app: &AppHandle) -> bool {
    app.store("config.json")
        .ok()
        .and_then(|store| store.get("popover_ativo"))
        .and_then(|valor| valor.as_bool())
        .unwrap_or(false)
}

/// Chamado pela tela de Configurações quando o usuário salva um novo
/// atalho principal. Falha de forma amigável (em vez de travar o app)
/// quando a combinação já está em uso por outro programa.
#[tauri::command]
fn registrar_atalho(app: AppHandle, atalho: String) -> Result<(), String> {
    if let Ok(store) = app.store("config.json") {
        store.set("atalho", serde_json::json!(atalho));
        let _ = store.save();
    }

    sincronizar_atalhos_no_backend(&app).map_err(|erro| erro.to_string())?;

    println!("[select-translate] Atalho global registrado: {atalho}");
    Ok(())
}

/// Idem, para o atalho do popover.
#[tauri::command]
fn registrar_atalho_popover(app: AppHandle, atalho: String) -> Result<(), String> {
    if let Ok(store) = app.store("config.json") {
        store.set("atalho_popover", serde_json::json!(atalho));
        let _ = store.save();
    }

    sincronizar_atalhos_no_backend(&app).map_err(|erro| erro.to_string())?;

    println!("[select-translate] Atalho do popover registrado: {atalho}");
    Ok(())
}

/// Liga/desliga o atalho do popover. Aplica na hora (sem precisar
/// reiniciar o app), igual ao "manter no topo".
#[tauri::command]
fn definir_popover_ativo(app: AppHandle, ativo: bool) -> Result<(), String> {
    if let Ok(store) = app.store("config.json") {
        store.set("popover_ativo", serde_json::json!(ativo));
        let _ = store.save();
    }

    sincronizar_atalhos_no_backend(&app).map_err(|erro| erro.to_string())
}

/// Chamado quando um dos dois campos de atalho nas Configurações ganha
/// foco (o usuário começou a "gravar" um novo atalho). Desregistra os
/// dois atalhos temporariamente — senão, se algum deles disparar
/// enquanto o campo está focado (ex: usuário aperta a combinação já
/// ativa), o Ctrl+C simulado pela captura normal cai direto nesse
/// campo e é interpretado como se fosse a nova gravação.
#[tauri::command]
fn pausar_atalho_global(app: AppHandle) -> Result<(), String> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|erro| erro.to_string())
}

/// Chamado quando um dos campos de atalho perde o foco (gravação
/// cancelada ou terminada sem clicar em "Salvar" — quem salva de
/// verdade é `registrar_atalho`/`registrar_atalho_popover`, que já
/// re-sincronizam com o valor novo). Volta a registrar os atalhos que
/// estavam salvos antes de começar a gravação.
#[tauri::command]
fn retomar_atalho_global(app: AppHandle) -> Result<(), String> {
    sincronizar_atalhos_no_backend(&app).map_err(|erro| erro.to_string())
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
            if let Err(erro) = sincronizar_atalhos_no_backend(&app.handle().clone()) {
                println!("[select-translate] Falha ao registrar atalhos globais: {erro}");
            } else {
                println!(
                    "[select-translate] Atalho global registrado: {}",
                    atalho_salvo(app.handle())
                );
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

            // Popover: some sozinho ao perder o foco (clicar fora, Alt+Tab
            // etc.) — não tem borda nem botão de fechar, então é a única
            // forma "natural" de dispensá-lo além do Esc (tratado no
            // frontend, ver popover.js).
            if let Some(janela_popover) = app.get_webview_window("popover") {
                let janela_popover_para_esconder = janela_popover.clone();
                janela_popover.on_window_event(move |event| {
                    if let WindowEvent::Focused(false) = event {
                        let _ = janela_popover_para_esconder.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            registrar_atalho,
            registrar_atalho_popover,
            definir_popover_ativo,
            pausar_atalho_global,
            retomar_atalho_global,
            definir_autostart,
            autostart_esta_ativo,
            definir_manter_no_topo
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
