import {
  carregarHistorico,
  formatarDataHistorico,
  inserirNoHistorico,
} from "./historico.js";

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

function criarItemHistorico(linha) {
  const item = document.createElement("li");

  const data = document.createElement("time");
  data.textContent = formatarDataHistorico(linha.criado_em);

  const original = document.createElement("p");
  original.className = "historico-original";
  original.textContent = linha.texto_original;

  const traduzido = document.createElement("p");
  traduzido.className = "historico-traduzido";
  traduzido.textContent = linha.texto_traduzido;

  item.append(data, original, traduzido);
  return item;
}

function renderizarHistorico(linhas) {
  const lista = document.getElementById("lista-historico");
  lista.replaceChildren();

  if (linhas.length === 0) {
    const vazio = document.createElement("li");
    vazio.className = "texto-vazio";
    vazio.textContent = "Nenhuma tradução ainda.";
    lista.appendChild(vazio);
    return;
  }

  // Texto original/traduzido vem do clipboard do usuário (pode ter sido
  // copiado de uma página web não confiável) — usar textContent em vez de
  // innerHTML evita que HTML/JS embutido no texto seja interpretado.
  for (const linha of linhas) {
    lista.appendChild(criarItemHistorico(linha));
  }
}

async function atualizarHistorico() {
  const linhas = await carregarHistorico();
  renderizarHistorico(linhas);
}

window.addEventListener("DOMContentLoaded", () => {
  document.querySelectorAll("[data-aba]").forEach((botao) => {
    botao.addEventListener("click", () => mostrarAba(botao.dataset.aba));
  });

  atualizarHistorico();

  listen("nova-traducao", async (evento) => {
    const { original, traduzido, idioma } = evento.payload;
    mostrarTraducao(original, traduzido);
    await inserirNoHistorico(original, traduzido, idioma);
    await atualizarHistorico();
  });
});
