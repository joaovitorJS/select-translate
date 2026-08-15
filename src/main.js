const { listen } = window.__TAURI__.event;

function mostrarAba(nome) {
  document.querySelectorAll("[data-aba]").forEach((botao) => {
    botao.classList.toggle("ativa", botao.dataset.aba === nome);
  });
  document.querySelectorAll(".painel").forEach((painel) => {
    painel.classList.toggle("oculto", painel.id !== `aba-${nome}`);
  });
}

function mostrarTraducao(original, traduzido) {
  const elementoOriginal = document.getElementById("texto-original");
  const elementoTraduzido = document.getElementById("texto-traduzido");

  elementoOriginal.textContent = original;
  elementoOriginal.classList.remove("texto-vazio");

  elementoTraduzido.textContent = traduzido;
  elementoTraduzido.classList.remove("texto-vazio");

  mostrarAba("traducao");
}

window.addEventListener("DOMContentLoaded", () => {
  document.querySelectorAll("[data-aba]").forEach((botao) => {
    botao.addEventListener("click", () => mostrarAba(botao.dataset.aba));
  });

  listen("nova-traducao", (evento) => {
    const { original, traduzido } = evento.payload;
    mostrarTraducao(original, traduzido);
  });
});
