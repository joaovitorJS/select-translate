use enigo::{Direction, Enigo, Key, Keyboard, Mouse, Settings};
use std::{thread, time::Duration};
use tauri::{AppHandle, Emitter, EventTarget, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_store::StoreExt;

use crate::popover;
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

/// Pra onde mandar o resultado de uma tradução: a janela principal (de
/// sempre) ou o popover perto do cursor (Melhoria — Popover de tradução).
/// `Popover` carrega a posição do cursor no instante da captura — não
/// dá pra reler o cursor só quando a tradução termina, porque o
/// usuário já pode ter movido o mouse até lá (a chamada à API de
/// tradução é assíncrona e pode levar um tempo).
#[derive(Clone, Copy)]
pub enum DestinoTraducao {
    Principal,
    Popover { cursor: (i32, i32) },
}

/// Compartilhada pelos dois atalhos globais (principal e popover) e
/// pelo modo automático: captura o texto selecionado em qualquer app
/// (via clipboard, simulando Ctrl+C) e envia para tradução.
fn capturar_e_traduzir_com_destino(app: AppHandle, destino: DestinoTraducao) {
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
    traduzir_e_notificar(app, texto_capturado, destino);
}

/// Dispara pelo atalho global principal: mostra o resultado na janela
/// principal, como sempre.
pub fn capturar_e_traduzir(app: AppHandle) {
    capturar_e_traduzir_com_destino(app, DestinoTraducao::Principal);
}

/// Dispara pelo atalho global do popover: mostra o resultado numa
/// bolha pequena perto de onde o cursor estava na hora da captura, sem
/// levantar a janela principal.
pub fn capturar_e_traduzir_popover(app: AppHandle) {
    let cursor = Enigo::new(&Settings::default())
        .ok()
        .and_then(|enigo| enigo.location().ok())
        .unwrap_or((0, 0));
    capturar_e_traduzir_com_destino(app, DestinoTraducao::Popover { cursor });
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
                traduzir_e_notificar(app.clone(), atual, DestinoTraducao::Principal);
            }
        }
    });
}

/// No Linux, `show()` só enfileira um pedido pra thread principal do
/// GTK (roda numa próxima volta do loop de eventos) — quem chama
/// `show()` é a task async da tradução, numa thread diferente da do
/// GTK, então a chamada retorna antes da janela virar visível de
/// verdade. Sem essa pausa, o `set_focus()` logo em seguida ainda lê
/// a janela como "não visível" e descarta o pedido de foco em
/// silêncio — a janela não trava, só nunca aparece sozinha depois de
/// escondida (issue #2). Causa raiz confirmada lendo o código do
/// `tao` (dependência do Tauri): `set_focus()` só manda o pedido de
/// fato se `!minimized && get_visible()`. No Windows a chamada nativa
/// já é síncrona, não precisa dessa pausa.
#[cfg(target_os = "linux")]
async fn aguardar_janela_aparecer() {
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[cfg(not(target_os = "linux"))]
async fn aguardar_janela_aparecer() {}

/// Rótulo da janela e evento a usar conforme o destino — os dois
/// eventos (`emit_to`, não `emit` geral) só chegam na janela que
/// efetivamente vai mostrar o resultado, senão o popover reagiria a
/// traduções disparadas pelo atalho principal (e vice-versa).
fn janela_do_destino(destino: DestinoTraducao) -> &'static str {
    match destino {
        DestinoTraducao::Principal => "main",
        DestinoTraducao::Popover { .. } => "popover",
    }
}

/// Acha os limites (posição + tamanho) do monitor que contém o
/// `cursor`, ou cai pro monitor atual da janela como aproximação
/// razoável se nenhum bater exatamente (ex: coordenada um pixel fora
/// por arredondamento em telas com escala fracionária).
fn limites_do_monitor_do_cursor(
    janela: &tauri::WebviewWindow,
    cursor: (i32, i32),
) -> (i32, i32, u32, u32) {
    let monitores = janela.available_monitors().unwrap_or_default();

    let encontrado = monitores.iter().find(|monitor| {
        let posicao = monitor.position();
        let tamanho = monitor.size();
        cursor.0 >= posicao.x
            && cursor.0 < posicao.x + tamanho.width as i32
            && cursor.1 >= posicao.y
            && cursor.1 < posicao.y + tamanho.height as i32
    });

    let monitor = encontrado
        .cloned()
        .or_else(|| janela.current_monitor().ok().flatten());

    match monitor {
        Some(monitor) => {
            let posicao = monitor.position();
            let tamanho = monitor.size();
            (posicao.x, posicao.y, tamanho.width, tamanho.height)
        }
        // Fallback bruto (Full HD) só pro caso extremo de não haver
        // nenhum monitor detectável — não deveria acontecer na prática.
        None => (0, 0, 1920, 1080),
    }
}

/// Mostra a janela do popover perto do `cursor` já com o indicador de
/// carregando visível — chamado **antes** da chamada à API de
/// tradução, não depois. Se esperássemos a tradução terminar pra só
/// então mostrar a janela (como a `main` faz), o usuário quase nunca
/// veria o "carregando": a API costuma responder rápido o bastante
/// pra a janela já apareceria com o resultado pronto, sem feedback
/// visual nenhum de que algo estava acontecendo.
async fn mostrar_popover(app: &AppHandle, cursor: (i32, i32)) {
    if let Some(janela) = app.get_webview_window("popover") {
        let tamanho = janela
            .outer_size()
            .map(|s| (s.width, s.height))
            .unwrap_or((320, 70));
        let tela = limites_do_monitor_do_cursor(&janela, cursor);
        let (x, y) = popover::calcular_posicao_popover(cursor, tamanho, tela);

        let _ = janela.set_position(tauri::PhysicalPosition::new(x, y));
        let _ = janela.show();
        aguardar_janela_aparecer().await;
        // O popover só some quando perde o foco (ver handler de
        // Focused(false) em lib.rs) — sem pegar o foco de verdade
        // aqui, ele nunca some sozinho ao clicar em outro lugar.
        let _ = janela.set_focus();
    }
}

/// Compartilhado pelos dois atalhos globais e pelo modo automático:
/// chama o provedor de tradução configurado e notifica a UI (eventos
/// de início/erro/sucesso + mostrar o resultado na janela certa)
/// durante o processo.
fn traduzir_e_notificar(app: AppHandle, texto: String, destino: DestinoTraducao) {
    let janela_alvo = janela_do_destino(destino);
    let _ = app.emit_to(
        EventTarget::labeled(janela_alvo),
        "traducao-iniciada",
        serde_json::json!({ "original": texto }),
    );

    tauri::async_runtime::spawn(async move {
        if let DestinoTraducao::Popover { cursor } = destino {
            mostrar_popover(&app, cursor).await;
        }

        let (config, idioma) = match traducao::configuracao_do_store(&app) {
            Ok(resultado) => resultado,
            Err(erro) => {
                println!("[select-translate] {erro}");
                let _ = app.emit_to(
                    EventTarget::labeled(janela_alvo),
                    "traducao-erro",
                    serde_json::json!({ "erro": erro }),
                );
                return;
            }
        };

        match traducao::traduzir(&config, &texto, &idioma).await {
            Ok(traduzido) => {
                println!("[select-translate] Tradução: {traduzido}");
                let _ = app.emit_to(
                    EventTarget::labeled(janela_alvo),
                    "nova-traducao",
                    serde_json::json!({
                        "original": texto,
                        "traduzido": traduzido,
                        "idioma": idioma,
                    }),
                );

                // Popover já foi mostrado antes da chamada à API (ver
                // `mostrar_popover`) — só a principal precisa ser
                // trazida pra frente agora, depois de saber que deu certo.
                if let DestinoTraducao::Principal = destino {
                    if let Some(janela) = app.get_webview_window("main") {
                        let _ = janela.show();
                        aguardar_janela_aparecer().await;
                        // Com "manter no topo" ligado, a janela já fica
                        // sempre visível acima das outras — não precisa
                        // roubar o foco do que o usuário está digitando
                        // a cada tradução.
                        if !manter_no_topo_ativo(&app) {
                            let _ = janela.set_focus();
                        }
                    }
                }
            }
            Err(erro) => {
                println!("[select-translate] Erro ao traduzir: {erro}");
                let _ = app.emit_to(
                    EventTarget::labeled(janela_alvo),
                    "traducao-erro",
                    serde_json::json!({ "erro": erro }),
                );
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
