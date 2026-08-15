import {
  carregarHistorico,
  formatarDataHistorico,
  inserirNoHistorico,
} from "./historico.js";
import {
  IDIOMAS,
  lerConfig,
  salvarConfig,
  validarConfigFormulario,
} from "./config.js";

const { listen } = window.__TAURI__.event;
const { invoke } = window.__TAURI__.core;

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

function popularSelectIdiomas() {
  const select = document.getElementById("select-idioma");
  select.replaceChildren(
    ...IDIOMAS.map(([valor, rotulo]) => {
      const opcao = document.createElement("option");
      opcao.value = valor;
      opcao.textContent = rotulo;
      return opcao;
    }),
  );
}

function alternarCamposProvedor(provedor) {
  document.getElementById("campos-deepl").classList.toggle("oculto", provedor !== "deepl");
  document.getElementById("campos-azure").classList.toggle("oculto", provedor !== "azure");
}

function mostrarMensagemConfig(texto, tipo) {
  const elemento = document.getElementById("config-mensagem");
  elemento.textContent = texto;
  elemento.className = tipo;
}

async function carregarConfigNaTela() {
  const atalho = await lerConfig("atalho", "CommandOrControl+Alt+T");
  const idioma = await lerConfig("idioma", "pt-br");
  const provedor = await lerConfig("provedor", "deepl");

  document.getElementById("input-atalho").value = atalho;
  document.getElementById("select-idioma").value = idioma;
  document.getElementById("select-provedor").value = provedor;
  document.getElementById("input-deepl-key").value = await lerConfig("deepl_api_key", "");
  document.getElementById("input-azure-key").value = await lerConfig("azure_api_key", "");
  document.getElementById("input-azure-regiao").value = await lerConfig("azure_regiao", "");

  alternarCamposProvedor(provedor);
}

async function salvarConfigDaTela(evento) {
  evento.preventDefault();

  const estado = {
    atalho: document.getElementById("input-atalho").value.trim(),
    idioma: document.getElementById("select-idioma").value,
    provedor: document.getElementById("select-provedor").value,
    deeplKey: document.getElementById("input-deepl-key").value.trim(),
    azureKey: document.getElementById("input-azure-key").value.trim(),
    azureRegiao: document.getElementById("input-azure-regiao").value.trim(),
  };

  const erro = validarConfigFormulario(estado);
  if (erro) {
    mostrarMensagemConfig(erro, "erro");
    return;
  }

  try {
    // Salva e re-registra o atalho global no mesmo comando — se a
    // combinação já estiver em uso por outro programa, o registro
    // falha e a Promise rejeita, sem travar o app.
    await invoke("registrar_atalho", { atalho: estado.atalho });
  } catch (erroAtalho) {
    mostrarMensagemConfig(`Não foi possível registrar esse atalho: ${erroAtalho}`, "erro");
    return;
  }

  await salvarConfig("idioma", estado.idioma);
  await salvarConfig("provedor", estado.provedor);
  await salvarConfig("deepl_api_key", estado.deeplKey);
  await salvarConfig("azure_api_key", estado.azureKey);
  await salvarConfig("azure_regiao", estado.azureRegiao);

  mostrarMensagemConfig("Configurações salvas.", "sucesso");
}

window.addEventListener("DOMContentLoaded", () => {
  document.querySelectorAll("[data-aba]").forEach((botao) => {
    botao.addEventListener("click", () => mostrarAba(botao.dataset.aba));
  });

  atualizarHistorico();

  popularSelectIdiomas();
  carregarConfigNaTela();
  document
    .getElementById("select-provedor")
    .addEventListener("change", (evento) => alternarCamposProvedor(evento.target.value));
  document.getElementById("form-config").addEventListener("submit", salvarConfigDaTela);

  listen("nova-traducao", async (evento) => {
    const { original, traduzido, idioma } = evento.payload;
    mostrarTraducao(original, traduzido);
    await inserirNoHistorico(original, traduzido, idioma);
    await atualizarHistorico();
  });
});
