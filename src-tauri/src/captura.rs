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

/// Código de tecla virtual do Windows (VK_C). Só serve nesse SO: no
/// Windows, `Key::Other(v)` manda `v` direto como Virtual-Key, então
/// tem que ser exatamente VK_C (0x43). Precisa ser `Key::Other`, não
/// `Key::Unicode('c')` — o modo Unicode no Windows injeta o caractere
/// diretamente e ignora o Ctrl que está pressionado ao mesmo tempo,
/// então o app de destino recebe só um "c" digitado em vez do atalho
/// de copiar.
#[cfg(target_os = "windows")]
const VK_C: u32 = 0x43;

/// Tecla enviada para simular o "C" de Ctrl+C, específica por SO
/// (testado empiricamente na Fase 11, ao portar para Linux).
///
/// No Windows precisa ser `Key::Other(VK_C)` (ver comentário de
/// `VK_C`). No Linux (e nas demais plataformas), `Key::Other(v)` vira
/// um keysym X11/Wayland, não um Virtual-Key — e `VK_C` (`0x43`)
/// também é, por coincidência, o keysym de "C" **maiúsculo**
/// (`XK_C`), então usá-lo aqui fazia o enigo sintetizar Shift+C,
/// virando Ctrl+Shift+C (um atalho diferente, não ligado a copiar na
/// maioria dos apps) em vez de Ctrl+C — o clipboard nunca mudava e a
/// captura falhava silenciosamente ("Nenhum texto novo selecionado").
/// `Key::Unicode('c')` resolve para o keysym minúsculo correto sem
/// precisar de Shift, e — diferente do Windows — não tem o problema
/// de ignorar o Ctrl pressionado, porque no Linux tanto `Unicode`
/// quanto `Other` passam pelo mesmo caminho de resolução de keysym.
fn tecla_copiar() -> Key {
    #[cfg(target_os = "windows")]
    {
        Key::Other(VK_C)
    }
    #[cfg(not(target_os = "windows"))]
    {
        Key::Unicode('c')
    }
}

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
        .key(tecla_copiar(), Direction::Click)
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

/// Interpreta um valor booleano opcional lido da store (`config.json`).
/// Ausente ou de tipo inesperado conta como desligado — mais seguro do
/// que assumir ligado. Usado por qualquer flag booleana da store
/// (modo automático, manter atrás, etc).
fn interpretar_booleano_store(valor: Option<serde_json::Value>) -> bool {
    valor.and_then(|v| v.as_bool()).unwrap_or(false)
}

fn flag_da_store_ativa(app: &AppHandle, chave: &str) -> bool {
    let valor = app.store("config.json").ok().and_then(|store| store.get(chave));
    interpretar_booleano_store(valor)
}

fn modo_automatico_ativo(app: &AppHandle) -> bool {
    flag_da_store_ativa(app, "modo_automatico")
}

fn manter_no_topo_ativo(app: &AppHandle) -> bool {
    flag_da_store_ativa(app, "manter_no_topo")
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
/// tradução configurado e notifica a UI (eventos de início/erro/sucesso
/// + trazer janela pra frente) durante o processo.
fn traduzir_e_notificar(app: AppHandle, texto: String) {
    let _ = app.emit("traducao-iniciada", serde_json::json!({ "original": texto }));

    tauri::async_runtime::spawn(async move {
        let (config, idioma) = match traducao::configuracao_do_store(&app) {
            Ok(resultado) => resultado,
            Err(erro) => {
                println!("[select-translate] {erro}");
                let _ = app.emit("traducao-erro", serde_json::json!({ "erro": erro }));
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
                    // Com "manter no topo" ligado, a janela já fica sempre
                    // visível acima das outras — não precisa roubar o foco
                    // do que o usuário está digitando a cada tradução.
                    if !manter_no_topo_ativo(&app) {
                        let _ = janela.set_focus();
                    }
                }
            }
            Err(erro) => {
                println!("[select-translate] Erro ao traduzir: {erro}");
                let _ = app.emit("traducao-erro", serde_json::json!({ "erro": erro }));
            }
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
    fn interpretar_booleano_store_true_quando_true() {
        assert!(interpretar_booleano_store(Some(serde_json::json!(true))));
    }

    #[test]
    fn interpretar_booleano_store_false_quando_false() {
        assert!(!interpretar_booleano_store(Some(serde_json::json!(false))));
    }

    #[test]
    fn interpretar_booleano_store_false_quando_ausente_ou_tipo_errado() {
        assert!(!interpretar_booleano_store(None));
        assert!(!interpretar_booleano_store(Some(serde_json::json!("ligado"))));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn tecla_copiar_usa_virtual_key_c_no_windows() {
        assert_eq!(tecla_copiar(), Key::Other(VK_C));
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn tecla_copiar_usa_c_minusculo_fora_do_windows() {
        // Ver comentário de `tecla_copiar`: `Key::Other(VK_C)` do Windows
        // equivale ao keysym de "C" maiúsculo no Linux, que quebra o
        // Ctrl+C (vira Ctrl+Shift+C). `Key::Unicode('c')` minúsculo é o
        // que resolve pro keysym certo sem exigir Shift.
        assert_eq!(tecla_copiar(), Key::Unicode('c'));
    }
}
