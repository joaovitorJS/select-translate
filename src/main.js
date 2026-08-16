import {
  carregarHistorico,
  excluirDoHistorico,
  formatarDataHistorico,
  inserirNoHistorico,
  limparHistorico,
} from "./historico.js";
import {
  IDIOMAS,
  lerConfig,
  montarAtalhoDoEvento,
  resolverTema,
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

// ---------------------------------------------------------------------
// Tema (claro/escuro/automático)
// ---------------------------------------------------------------------

const consultaTemaEscuro = window.matchMedia("(prefers-color-scheme: dark)");

function aplicarTema(preferencia) {
  const resolvido = resolverTema(preferencia, consultaTemaEscuro.matches);
  document.documentElement.dataset.theme = resolvido;
}

async function carregarTemaNaTela() {
  const preferencia = await lerConfig("tema", "automatico");
  document.getElementById("select-tema").value = preferencia;
  aplicarTema(preferencia);
}

async function alternarTema(evento) {
  await salvarConfig("tema", evento.target.value);
  aplicarTema(evento.target.value);
}

// ---------------------------------------------------------------------
// Aba Tradução
// ---------------------------------------------------------------------

function mostrarTraducao(original, traduzido) {
  const elementoOriginal = document.getElementById("texto-original");
  const elementoTraduzido = document.getElementById("texto-traduzido");

  elementoOriginal.textContent = original;
  elementoOriginal.classList.remove("texto-vazio");

  elementoTraduzido.textContent = traduzido;
  elementoTraduzido.classList.remove("texto-vazio");

  esconderCarregandoTraducao();
  esconderErroTraducao();
  mostrarAba("traducao");
}

function mostrarCarregandoTraducao(original) {
  document.getElementById("traducao-status").classList.remove("oculto");
  esconderErroTraducao();

  const elementoOriginal = document.getElementById("texto-original");
  elementoOriginal.textContent = original;
  elementoOriginal.classList.remove("texto-vazio");

  mostrarAba("traducao");
}

function esconderCarregandoTraducao() {
  document.getElementById("traducao-status").classList.add("oculto");
}

function mostrarErroTraducao(mensagem) {
  esconderCarregandoTraducao();
  const elemento = document.getElementById("traducao-erro");
  elemento.textContent = mensagem;
  elemento.classList.remove("oculto");
}

function esconderErroTraducao() {
  document.getElementById("traducao-erro").classList.add("oculto");
}

// ---------------------------------------------------------------------
// Aba Histórico
// ---------------------------------------------------------------------

function criarBlocoHistorico(rotulo, texto, classe) {
  const bloco = document.createElement("div");
  bloco.className = `historico-bloco ${classe}`;

  const label = document.createElement("span");
  label.className = "historico-rotulo";
  label.textContent = rotulo;

  const paragrafo = document.createElement("p");
  paragrafo.textContent = texto;

  bloco.append(label, paragrafo);
  return bloco;
}

function criarItemHistorico(linha) {
  const item = document.createElement("li");

  const cabecalho = document.createElement("div");
  cabecalho.className = "historico-cabecalho";

  const data = document.createElement("time");
  data.textContent = formatarDataHistorico(linha.criado_em);

  const botaoExcluir = document.createElement("button");
  botaoExcluir.type = "button";
  botaoExcluir.className = "historico-excluir";
  botaoExcluir.setAttribute("aria-label", "Excluir esta tradução do histórico");
  botaoExcluir.textContent = "×";
  botaoExcluir.addEventListener("click", async () => {
    await excluirDoHistorico(linha.id);
    await atualizarHistorico();
  });

  cabecalho.append(data, botaoExcluir);

  const original = criarBlocoHistorico("Original", linha.texto_original, "historico-original");
  const traduzido = criarBlocoHistorico("Tradução", linha.texto_traduzido, "historico-traduzido");

  item.append(cabecalho, original, traduzido);
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

// "Limpar histórico" apaga tudo (irreversível), então exige dois
// cliques: o primeiro vira um pedido de confirmação por alguns
// segundos, o segundo (dentro do prazo) executa de fato. Evita usar
// window.confirm() — o app não usa diálogos nativos do navegador em
// nenhum outro lugar.
let limpezaPendente = null;

async function alternarLimparHistorico(evento) {
  const botao = evento.currentTarget;

  if (limpezaPendente) {
    clearTimeout(limpezaPendente);
    limpezaPendente = null;
    botao.textContent = "Limpar histórico";
    botao.classList.remove("confirmando");
    await limparHistorico();
    await atualizarHistorico();
    return;
  }

  botao.textContent = "Confirmar: apagar tudo?";
  botao.classList.add("confirmando");
  limpezaPendente = setTimeout(() => {
    limpezaPendente = null;
    botao.textContent = "Limpar histórico";
    botao.classList.remove("confirmando");
  }, 4000);
}

// ---------------------------------------------------------------------
// Aba Configurações
// ---------------------------------------------------------------------

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

// Qual provedor usar é uma escolha "do momento", não algo que precisa de
// "Salvar" — aplica na hora e mantém os dois seletores (aba Tradução e
// Configurações) sincronizados. As credenciais de cada provedor
// continuam guardadas independentemente, então alternar não perde nada.
async function definirProvedorAtivo(provedor) {
  await salvarConfig("provedor", provedor);
  document.getElementById("select-provedor-rapido").value = provedor;
  document.getElementById("select-provedor").value = provedor;
  alternarCamposProvedor(provedor);
}

function mostrarMensagemConfig(texto, tipo) {
  const elemento = document.getElementById("config-mensagem");
  elemento.textContent = texto;
  elemento.className = `mensagem ${tipo}`;
}

// O campo de atalho é `readonly` (não dá pra digitar direto) — em vez
// disso, ele "grava" a combinação de teclas pressionada enquanto está
// focado, parecido com o editor de atalhos do VS Code/Discord.
function capturarTeclaDoAtalho(evento) {
  evento.preventDefault();

  if (evento.key === "Escape") {
    evento.target.blur();
    return;
  }

  const atalho = montarAtalhoDoEvento(evento);
  if (atalho) {
    evento.target.value = atalho;
  }
}

async function carregarConfigNaTela() {
  const atalho = await lerConfig("atalho", "CommandOrControl+Alt+T");
  const idioma = await lerConfig("idioma", "pt-br");
  const provedor = await lerConfig("provedor", "deepl");
  const modoAutomatico = await lerConfig("modo_automatico", false);

  document.getElementById("input-atalho").value = atalho;
  document.getElementById("input-modo-automatico").checked = modoAutomatico;
  document.getElementById("input-autostart").checked = await invoke("autostart_esta_ativo");
  document.getElementById("input-manter-no-topo").checked = await lerConfig(
    "manter_no_topo",
    false,
  );
  document.getElementById("select-idioma").value = idioma;
  document.getElementById("select-provedor").value = provedor;
  document.getElementById("select-provedor-rapido").value = provedor;
  document.getElementById("input-deepl-key").value = await lerConfig("deepl_api_key", "");
  document.getElementById("input-azure-key").value = await lerConfig("azure_api_key", "");
  document.getElementById("input-azure-regiao").value = await lerConfig("azure_regiao", "");

  alternarCamposProvedor(provedor);
}

// Modo automático é salvo assim que o checkbox muda — diferente dos
// outros campos, não precisa clicar em "Salvar" para ter efeito (o
// backend confere o valor salvo a cada ~800ms).
async function alternarModoAutomatico(evento) {
  await salvarConfig("modo_automatico", evento.target.checked);
  mostrarMensagemConfig(
    evento.target.checked ? "Modo automático ligado." : "Modo automático desligado.",
    "sucesso",
  );
}

// Igual ao modo automático: efeito imediato ao mudar, sem precisar de
// "Salvar". O estado de verdade fica no registro do Windows (via
// tauri-plugin-autostart), não em config.json.
async function alternarAutostart(evento) {
  try {
    await invoke("definir_autostart", { ativo: evento.target.checked });
    mostrarMensagemConfig(
      evento.target.checked ? "Vai iniciar com o Windows." : "Não inicia mais com o Windows.",
      "sucesso",
    );
  } catch (erro) {
    evento.target.checked = !evento.target.checked;
    mostrarMensagemConfig(`Não foi possível alterar isso: ${erro}`, "erro");
  }
}

// Mesmo padrão do autostart: aplica na janela e salva no mesmo command,
// efeito imediato sem precisar de "Salvar".
async function alternarManterNoTopo(evento) {
  try {
    await invoke("definir_manter_no_topo", { ativo: evento.target.checked });
    mostrarMensagemConfig(
      evento.target.checked
        ? "A janela vai ficar sempre em cima das outras."
        : "A janela não fica mais sempre em cima.",
      "sucesso",
    );
  } catch (erro) {
    evento.target.checked = !evento.target.checked;
    mostrarMensagemConfig(`Não foi possível alterar isso: ${erro}`, "erro");
  }
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

  // "provedor" não entra aqui: definirProvedorAtivo já salva na hora,
  // assim que o seletor muda (na aba Tradução ou aqui).
  await salvarConfig("idioma", estado.idioma);
  await salvarConfig("deepl_api_key", estado.deeplKey);
  await salvarConfig("azure_api_key", estado.azureKey);
  await salvarConfig("azure_regiao", estado.azureRegiao);

  mostrarMensagemConfig("Configurações salvas.", "sucesso");
}

// ---------------------------------------------------------------------

window.addEventListener("DOMContentLoaded", () => {
  document.querySelectorAll("[data-aba]").forEach((botao) => {
    botao.addEventListener("click", () => mostrarAba(botao.dataset.aba));
  });

  atualizarHistorico();
  document
    .getElementById("btn-limpar-historico")
    .addEventListener("click", alternarLimparHistorico);

  carregarTemaNaTela();
  document.getElementById("select-tema").addEventListener("change", alternarTema);
  consultaTemaEscuro.addEventListener("change", async () => {
    const preferencia = await lerConfig("tema", "automatico");
    if (preferencia === "automatico") aplicarTema(preferencia);
  });

  popularSelectIdiomas();
  carregarConfigNaTela();
  const inputAtalho = document.getElementById("input-atalho");
  inputAtalho.addEventListener("keydown", capturarTeclaDoAtalho);
  inputAtalho.addEventListener("focus", () => inputAtalho.classList.add("gravando"));
  inputAtalho.addEventListener("blur", () => inputAtalho.classList.remove("gravando"));
  document
    .getElementById("select-provedor")
    .addEventListener("change", (evento) => definirProvedorAtivo(evento.target.value));
  document
    .getElementById("select-provedor-rapido")
    .addEventListener("change", (evento) => definirProvedorAtivo(evento.target.value));
  document.getElementById("form-config").addEventListener("submit", salvarConfigDaTela);
  document
    .getElementById("input-modo-automatico")
    .addEventListener("change", alternarModoAutomatico);
  document.getElementById("input-autostart").addEventListener("change", alternarAutostart);
  document
    .getElementById("input-manter-no-topo")
    .addEventListener("change", alternarManterNoTopo);

  listen("traducao-iniciada", (evento) => {
    mostrarCarregandoTraducao(evento.payload.original);
  });

  listen("traducao-erro", (evento) => {
    mostrarErroTraducao(evento.payload.erro);
  });

  listen("nova-traducao", async (evento) => {
    const { original, traduzido, idioma } = evento.payload;
    mostrarTraducao(original, traduzido);
    await inserirNoHistorico(original, traduzido, idioma);
    await atualizarHistorico();
  });
});
