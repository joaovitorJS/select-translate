use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::{thread, time::Duration};
use tauri::{AppHandle, Emitter};
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::traducao;

/// Decide se o texto lido do clipboard depois do Ctrl+C simulado é,
/// de fato, uma nova seleção (e não apenas o que já estava no
/// clipboard antes, ou uma seleção vazia/só espaços).
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
/// qualquer app (via clipboard) e envia para tradução.
pub fn capturar_e_traduzir(app: AppHandle) {
    let leitura_original = app.clipboard().read_text();
    println!("[select-translate] [debug] Clipboard antes: {leitura_original:?}");
    let clipboard_original = leitura_original.unwrap_or_default();

    if let Err(erro) = simular_copiar() {
        println!("[select-translate] Falha ao simular Ctrl+C: {erro}");
        return;
    }

    let leitura_capturada = app.clipboard().read_text();
    println!("[select-translate] [debug] Clipboard depois: {leitura_capturada:?}");
    let texto_capturado = leitura_capturada.unwrap_or_default();

    if !houve_novo_texto(&clipboard_original, &texto_capturado) {
        println!("[select-translate] Nenhum texto novo selecionado.");
        return;
    }

    println!("[select-translate] Texto capturado: {texto_capturado}");

    tauri::async_runtime::spawn(async move {
        let config = match traducao::configuracao_do_ambiente() {
            Ok(config) => config,
            Err(erro) => {
                println!("[select-translate] {erro}");
                return;
            }
        };

        match traducao::traduzir(&config, &texto_capturado).await {
            Ok(traduzido) => {
                println!("[select-translate] Tradução: {traduzido}");
                let _ = app.emit(
                    "nova-traducao",
                    serde_json::json!({ "original": texto_capturado, "traduzido": traduzido }),
                );
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
}
