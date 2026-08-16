use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::{thread, time::Duration};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_store::StoreExt;

use crate::traducao;

/// Intervalo entre checagens do clipboard no modo automático.
const INTERVALO_MONITORAMENTO: Duration = Duration::from_millis(800);

/// Decide se o texto lido do clipboard é, de fato, uma novidade (e não
/// apenas o que já estava lá antes, ou uma seleção vazia/só espaços).
pub fn houve_novo_texto(clipboard_original: &str, clipboard_capturado: &str) -> bool {
    !clipboard_capturado.trim().is_empty() && clipboard_capturado != clipboard_original
}

/// Código de tecla virtual do Windows (VK_C). Precisa ser `Key::Other`
/// (tecla física de verdade), não `Key::Unicode('c')` — o modo Unicode
/// injeta o caractere diretamente e ignora o Ctrl que está pressionado
/// ao mesmo tempo, então o app de destino recebe só um "c" digitado
/// em vez do atalho de copiar.
const VK_C: u32 = 0x43;

fn simular_copiar() -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;

    // No instante em que o atalho global dispara, as teclas físicas do
    // atalho (ex: Ctrl+Alt+T) ainda podem estar pressionadas. Se não
    // soltarmos os modificadores antes, o app de destino recebe
    // Ctrl+Alt+C (ou pior) em vez de só Ctrl+C.
    enigo
        .key(Key::Alt, Direction::Release)
        .map_err(|e| e.to_string())?;
    enigo
        .key(Key::Shift, Direction::Release)
        .map_err(|e| e.to_string())?;
    enigo
        .key(Key::Control, Direction::Release)
        .map_err(|e| e.to_string())?;
    thread::sleep(Duration::from_millis(50));

    enigo
        .key(Key::Control, Direction::Press)
        .map_err(|e| e.to_string())?;
    enigo
        .key(Key::Other(VK_C), Direction::Click)
        .map_err(|e| e.to_string())?;
    enigo
        .key(Key::Control, Direction::Release)
        .map_err(|e| e.to_string())?;
    thread::sleep(Duration::from_millis(250));
    Ok(())
}

/// Dispara pelo atalho global: captura o texto selecionado em
/// qualquer app (via clipboard, simulando Ctrl+C) e envia para tradução.
pub fn capturar_e_traduzir(app: AppHandle) {
    let clipboard_original = app.clipboard().read_text().unwrap_or_default();

    if let Err(erro) = simular_copiar() {
        println!("[select-translate] Falha ao simular Ctrl+C: {erro}");
        return;
    }

    let texto_capturado = app.clipboard().read_text().unwrap_or_default();

    if !houve_novo_texto(&clipboard_original, &texto_capturado) {
        println!("[select-translate] Nenhum texto novo selecionado.");
        return;
    }

    println!("[select-translate] Texto capturado: {texto_capturado}");
    traduzir_e_notificar(app, texto_capturado);
}

/// Lê `modo_automatico` da store (`config.json`). Ausente ou de tipo
/// inesperado conta como desligado — mais seguro do que assumir ligado.
fn interpretar_modo_automatico(valor: Option<serde_json::Value>) -> bool {
    valor.and_then(|v| v.as_bool()).unwrap_or(false)
}

fn modo_automatico_ativo(app: &AppHandle) -> bool {
    let valor = app.store("config.json").ok().and_then(|store| store.get("modo_automatico"));
    interpretar_modo_automatico(valor)
}

/// Roda para sempre em segundo plano (uma thread própria, sem bloquear
/// o resto do app) checando o clipboard periodicamente. Só dispara
/// tradução quando `modo_automatico` está ligado nas Configurações — a
/// checagem é feita a cada volta do loop, então ligar/desligar o
/// checkbox tem efeito na próxima checagem, sem precisar reiniciar o app.
pub fn iniciar_monitoramento_automatico(app: AppHandle) {
    thread::spawn(move || {
        // Começa com o que já está no clipboard para não traduzir, na
        // primeira checagem, algo que o usuário copiou antes de abrir
        // o app (ou antes de ligar o modo automático).
        let mut ultimo_valor = app.clipboard().read_text().unwrap_or_default();

        loop {
            thread::sleep(INTERVALO_MONITORAMENTO);

            if !modo_automatico_ativo(&app) {
                continue;
            }

            let Ok(atual) = app.clipboard().read_text() else {
                continue;
            };

            if houve_novo_texto(&ultimo_valor, &atual) {
                ultimo_valor = atual.clone();
                println!("[select-translate] (modo automático) Texto capturado: {atual}");
                traduzir_e_notificar(app.clone(), atual);
            }
        }
    });
}

/// Compartilhado pelos dois modos de captura: chama o provedor de
/// tradução configurado e notifica a UI (evento + trazer janela pra
/// frente) quando a tradução termina.
fn traduzir_e_notificar(app: AppHandle, texto: String) {
    tauri::async_runtime::spawn(async move {
        let (config, idioma) = match traducao::configuracao_do_store(&app) {
            Ok(resultado) => resultado,
            Err(erro) => {
                println!("[select-translate] {erro}");
                return;
            }
        };

        match traducao::traduzir(&config, &texto, &idioma).await {
            Ok(traduzido) => {
                println!("[select-translate] Tradução: {traduzido}");
                let _ = app.emit(
                    "nova-traducao",
                    serde_json::json!({
                        "original": texto,
                        "traduzido": traduzido,
                        "idioma": idioma,
                    }),
                );

                if let Some(janela) = app.get_webview_window("main") {
                    let _ = janela.show();
                    let _ = janela.set_focus();
                }
            }
            Err(erro) => println!("[select-translate] Erro ao traduzir: {erro}"),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detecta_texto_novo_diferente_do_original() {
        assert!(houve_novo_texto("abc", "xyz"));
    }

    #[test]
    fn ignora_quando_capturado_igual_ao_original() {
        assert!(!houve_novo_texto("abc", "abc"));
    }

    #[test]
    fn ignora_quando_capturado_esta_vazio_ou_so_espacos() {
        assert!(!houve_novo_texto("abc", ""));
        assert!(!houve_novo_texto("abc", "   "));
    }

    #[test]
    fn interpretar_modo_automatico_true_quando_true() {
        assert!(interpretar_modo_automatico(Some(serde_json::json!(true))));
    }

    #[test]
    fn interpretar_modo_automatico_false_quando_false() {
        assert!(!interpretar_modo_automatico(Some(serde_json::json!(false))));
    }

    #[test]
    fn interpretar_modo_automatico_false_quando_ausente_ou_tipo_errado() {
        assert!(!interpretar_modo_automatico(None));
        assert!(!interpretar_modo_automatico(Some(serde_json::json!("ligado"))));
    }
}
