/// Deslocamento do popover em relação ao cursor (mesma convenção visual de
/// um tooltip/context menu — nunca nasce exatamente em cima do ponteiro).
const DESLOCAMENTO_X: i32 = 16;
const DESLOCAMENTO_Y: i32 = 16;

/// Calcula onde posicionar a janela do popover perto do cursor, sem deixar
/// nenhuma parte dela sair da área útil da tela — importante perto das
/// bordas/cantos (ex: usuário seleciona texto no canto inferior direito do
/// monitor).
pub fn calcular_posicao_popover(
    cursor: (i32, i32),
    tamanho_popover: (u32, u32),
    tela: (i32, i32, u32, u32),
) -> (i32, i32) {
    let (cursor_x, cursor_y) = cursor;
    let (largura_popover, altura_popover) = (tamanho_popover.0 as i32, tamanho_popover.1 as i32);
    let (tela_x, tela_y, tela_largura, tela_altura) = tela;

    let x_desejado = cursor_x + DESLOCAMENTO_X;
    let y_desejado = cursor_y + DESLOCAMENTO_Y;

    let x_maximo = tela_x + tela_largura as i32 - largura_popover;
    let y_maximo = tela_y + tela_altura as i32 - altura_popover;

    (
        x_desejado.clamp(tela_x, x_maximo.max(tela_x)),
        y_desejado.clamp(tela_y, y_maximo.max(tela_y)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const TELA_FULL_HD: (i32, i32, u32, u32) = (0, 0, 1920, 1080);
    const POPOVER: (u32, u32) = (360, 160);

    #[test]
    fn desloca_do_cursor_quando_ha_espaco_de_sobra() {
        let (x, y) = calcular_posicao_popover((500, 500), POPOVER, TELA_FULL_HD);
        assert_eq!((x, y), (516, 516));
    }

    #[test]
    fn nao_ultrapassa_a_borda_direita() {
        let (x, _) = calcular_posicao_popover((1900, 500), POPOVER, TELA_FULL_HD);
        assert_eq!(x, 1920 - 360);
    }

    #[test]
    fn nao_ultrapassa_a_borda_inferior() {
        let (_, y) = calcular_posicao_popover((500, 1070), POPOVER, TELA_FULL_HD);
        assert_eq!(y, 1080 - 160);
    }

    #[test]
    fn nao_ultrapassa_o_canto_inferior_direito() {
        let (x, y) = calcular_posicao_popover((1919, 1079), POPOVER, TELA_FULL_HD);
        assert_eq!((x, y), (1920 - 360, 1080 - 160));
    }

    #[test]
    fn respeita_origem_de_tela_fora_de_zero_multi_monitor() {
        // Segundo monitor à direita do principal, começando em x=1920.
        let tela_segundo_monitor = (1920, 0, 1920, 1080);
        let (x, y) = calcular_posicao_popover((3800, 500), POPOVER, tela_segundo_monitor);
        assert_eq!((x, y), (1920 + 1920 - 360, 516));
    }

    #[test]
    fn nao_ultrapassa_a_borda_esquerda_ou_superior_mesmo_com_deslocamento_negativo_hipotetico() {
        // Cursor já no canto (0,0) da tela: o deslocamento positivo não
        // deveria nunca colocar o popover fora da tela por cima/esquerda.
        let (x, y) = calcular_posicao_popover((0, 0), POPOVER, TELA_FULL_HD);
        assert_eq!((x, y), (16, 16));
    }
}
