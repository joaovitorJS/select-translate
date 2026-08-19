import { lerConfig, resolverTema } from "./config.js";

const { listen } = window.__TAURI__.event;
const janelaAtual = window.__TAURI__.window.getCurrentWindow();

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

function esconderStatus() {
  document.getElementById("popover-status").classList.add("oculto");
  document.getElementById("popover-erro").classList.add("oculto");
}

function mostrarCarregando(original) {
  esconderStatus();
  document.getElementById("popover-status").classList.remove("oculto");
  document.getElementById("popover-original").textContent = original;
  document.getElementById("popover-original").classList.remove("oculto");
  document.getElementById("popover-traduzido").classList.add("oculto");
}

function mostrarErro(mensagem) {
  esconderStatus();
  const elementoErro = document.getElementById("popover-erro");
  elementoErro.textContent = mensagem;
  elementoErro.classList.remove("oculto");
}

// Texto vem do clipboard do usuário (pode ter sido copiado de uma
// página não confiável) — textContent, nunca innerHTML, mesma regra
// do main.js.
function mostrarTraducao(original, traduzido) {
  esconderStatus();
  document.getElementById("popover-original").textContent = original;
  document.getElementById("popover-original").classList.remove("oculto");
  document.getElementById("popover-traduzido").textContent = traduzido;
  document.getElementById("popover-traduzido").classList.remove("oculto");
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

  listen("traducao-iniciada", (evento) => {
    mostrarCarregando(evento.payload.original);
  });

  listen("traducao-erro", (evento) => {
    mostrarErro(evento.payload.erro);
  });

  listen("nova-traducao", (evento) => {
    const { original, traduzido } = evento.payload;
    mostrarTraducao(original, traduzido);
  });
});
