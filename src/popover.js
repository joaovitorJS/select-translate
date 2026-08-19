import { lerConfig, resolverTema, truncarTexto } from "./config.js";

const { listen } = window.__TAURI__.event;
const { LogicalSize } = window.__TAURI__.window;
const janelaAtual = window.__TAURI__.window.getCurrentWindow();

// Largura fixa (bate com o "width" inicial em tauri.conf.json) — só a
// altura se ajusta ao conteúdo. Uma bolha que também mudasse de
// largura ficaria mais fácil de estourar a borda da tela (a posição já
// foi calculada pro tamanho antigo, do lado Rust, antes de mostrar).
const LARGURA = 320;
const ALTURA_MINIMA = 70;
const ALTURA_MAXIMA = 400;

// Evita a bolha crescer sem limite com uma seleção enorme — também
// limita, na prática, até onde ALTURA_MAXIMA precisa dar conta.
const LIMITE_CARACTERES = 500;

// Mesma lógica de tema do main.js (aplicarTema/carregarTemaNaTela) —
// duplicada porque cada janela do Tauri carrega seu próprio documento,
// sem estado JS compartilhado entre elas. `resolverTema` é reaproveitada
// de config.js, só a parte de "onde aplicar no DOM" é local.
const consultaTemaEscuro = window.matchMedia("(prefers-color-scheme: dark)");

function aplicarTema(preferencia) {
  document.documentElement.dataset.theme = resolverTema(preferencia, consultaTemaEscuro.matches);
}

async function carregarTema() {
  aplicarTema(await lerConfig("tema", "automatico"));
}

function esconder() {
  janelaAtual.hide();
}

// Redimensiona a janela pra caber o conteúdo atual do popover (só a
// altura, ver LARGURA acima). Medir `scrollHeight` logo depois de mudar
// o `textContent` já força o navegador a recalcular o layout na hora
// (leitura de scrollHeight é síncrona) — nada de `requestAnimationFrame`
// aqui: como o evento de tradução chega **antes** do `show()` do lado
// Rust (a janela ainda está escondida nesse momento), rAF fica parado
// até a janela aparecer e o callback nunca roda a tempo.
function ajustarTamanhoAoConteudo() {
  const altura = document.getElementById("popover-shell").scrollHeight;
  const alturaFinal = Math.min(Math.max(altura, ALTURA_MINIMA), ALTURA_MAXIMA);
  janelaAtual.setSize(new LogicalSize(LARGURA, alturaFinal));
}

function esconderStatus() {
  document.getElementById("popover-status").classList.add("oculto");
  document.getElementById("popover-erro").classList.add("oculto");
}

function mostrarCarregando() {
  esconderStatus();
  document.getElementById("popover-status").classList.remove("oculto");
  document.getElementById("popover-traduzido").classList.add("oculto");
  ajustarTamanhoAoConteudo();
}

function mostrarErro(mensagem) {
  esconderStatus();
  const elementoErro = document.getElementById("popover-erro");
  elementoErro.textContent = mensagem;
  elementoErro.classList.remove("oculto");
  ajustarTamanhoAoConteudo();
}

// Só a tradução é mostrada (sem o texto original) — o popover é pra
// uma consulta rápida, não pra comparar os dois textos como na janela
// principal. Texto vem do clipboard do usuário (pode ter sido copiado
// de uma página não confiável) — textContent, nunca innerHTML, mesma
// regra do main.js.
function mostrarTraducao(traduzido) {
  esconderStatus();
  const elementoTraduzido = document.getElementById("popover-traduzido");
  elementoTraduzido.textContent = truncarTexto(traduzido, LIMITE_CARACTERES);
  elementoTraduzido.classList.remove("oculto");
  ajustarTamanhoAoConteudo();
}

window.addEventListener("DOMContentLoaded", () => {
  carregarTema();
  consultaTemaEscuro.addEventListener("change", async () => {
    const preferencia = await lerConfig("tema", "automatico");
    if (preferencia === "automatico") aplicarTema(preferencia);
  });

  window.addEventListener("keydown", (evento) => {
    if (evento.key === "Escape") {
      esconder();
    }
  });

  listen("traducao-iniciada", () => {
    mostrarCarregando();
  });

  listen("traducao-erro", (evento) => {
    mostrarErro(evento.payload.erro);
  });

  listen("nova-traducao", (evento) => {
    mostrarTraducao(evento.payload.traduzido);
  });
});
