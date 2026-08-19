const CAMINHO_CONFIG = "config.json";

// Idiomas suportados: (valor_canonico, rótulo). Precisa bater com a
// lista IDIOMAS de src-tauri/src/traducao/mod.rs.
export const IDIOMAS = [
  ["pt-br", "Português (Brasil)"],
  ["pt-pt", "Português (Portugal)"],
  ["en", "Inglês"],
  ["es", "Espanhol"],
  ["fr", "Francês"],
  ["de", "Alemão"],
  ["it", "Italiano"],
];

// `set`/`get`/`save` do tauri-plugin-store exigem um `rid` de uma store
// já carregada via `plugin:store|load` — não conectam sob demanda
// (mesma pegadinha do plugin-sql na Fase 4).
let ridCarregado = null;

function garantirStoreCarregado() {
  const { invoke } = window.__TAURI__.core;
  if (!ridCarregado) {
    ridCarregado = invoke("plugin:store|load", { path: CAMINHO_CONFIG });
  }
  return ridCarregado;
}

export async function salvarConfig(chave, valor) {
  const { invoke } = window.__TAURI__.core;
  const rid = await garantirStoreCarregado();
  await invoke("plugin:store|set", { rid, key: chave, value: valor });
  await invoke("plugin:store|save", { rid });
}

/**
 * Regra de validação do formulário de Configurações: cada provedor
 * exige campos diferentes. Retorna a mensagem de erro, ou `null` se
 * estiver tudo certo.
 */
export function validarConfigFormulario({
  atalho,
  atalhoPopover,
  popoverAtivo,
  provedor,
  deeplKey,
  azureKey,
  azureRegiao,
}) {
  if (!atalho) {
    return "Informe um atalho.";
  }
  if (popoverAtivo && !atalhoPopover) {
    return "Informe um atalho para o popover (ou desative o popover).";
  }
  if (provedor === "deepl" && !deeplKey) {
    return "Informe a chave da API do DeepL.";
  }
  if (provedor === "azure" && (!azureKey || !azureRegiao)) {
    return "Informe a chave e a região do Azure Translator.";
  }
  return null;
}

/**
 * Corta `texto` em `limite` caracteres, acrescentando "…" quando corta
 * de verdade (não quando o texto já cabia). Usado pelo popover pra não
 * deixar a bolha crescer sem limite com uma seleção enorme.
 */
export function truncarTexto(texto, limite) {
  if (texto.length <= limite) {
    return texto;
  }
  return `${texto.slice(0, limite)}…`;
}

export async function lerConfig(chave, valorPadrao) {
  const { invoke } = window.__TAURI__.core;
  const rid = await garantirStoreCarregado();
  const [valor, existe] = await invoke("plugin:store|get", { rid, key: chave });
  return existe ? valor : valorPadrao;
}

/**
 * Decide qual tema aplicar de fato: "automatico" segue a preferência do
 * Windows (prefers-color-scheme); "claro"/"escuro" sempre vencem,
 * independente do sistema.
 */
export function resolverTema(preferencia, sistemaPrefereEscuro) {
  if (preferencia === "claro" || preferencia === "escuro") {
    return preferencia;
  }
  return sistemaPrefereEscuro ? "escuro" : "claro";
}

// Teclas que já têm um nome amigável diferente do `event.key` cru.
const NOMES_TECLA_ESPECIAL = {
  " ": "Space",
  Escape: "Escape",
  Enter: "Enter",
  Tab: "Tab",
  Backspace: "Backspace",
  Delete: "Delete",
  ArrowUp: "Up",
  ArrowDown: "Down",
  ArrowLeft: "Left",
  ArrowRight: "Right",
  Home: "Home",
  End: "End",
  PageUp: "PageUp",
  PageDown: "PageDown",
};

const MODIFICADORES_PUROS = new Set(["Control", "Alt", "Shift", "Meta"]);

/**
 * Nome da tecla no formato que o `tauri-plugin-global-shortcut` espera
 * (ex: "T", "1", "F5", "Space"). Usa `code` (posição física, não afetada
 * por layout de teclado) quando dá, e cai para `key` em teclas especiais.
 */
function nomeDaTecla({ key, code }) {
  if (NOMES_TECLA_ESPECIAL[key]) {
    return NOMES_TECLA_ESPECIAL[key];
  }
  if (/^F([1-9]|1\d|2[0-4])$/.test(key)) {
    return key; // F1–F24
  }
  if (code?.startsWith("Key")) {
    return code.slice(3); // "KeyT" -> "T"
  }
  if (code?.startsWith("Digit")) {
    return code.slice(5); // "Digit1" -> "1"
  }
  if (key.length === 1) {
    return key.toUpperCase();
  }
  return key; // melhor esforço para teclas não mapeadas explicitamente
}

/**
 * Converte um evento de teclado (ou objeto com o mesmo formato, em
 * testes) na string de atalho que o backend espera, tipo
 * "CommandOrControl+Alt+T". Retorna `null` quando só um modificador foi
 * pressionado até agora (ainda não é um atalho completo) ou quando a
 * tecla é Escape (usada pra cancelar a gravação, não pra virar atalho).
 */
export function montarAtalhoDoEvento(evento) {
  const { ctrlKey, altKey, shiftKey, metaKey, key } = evento;

  if (key === "Escape" || MODIFICADORES_PUROS.has(key)) {
    return null;
  }

  const partes = [];
  if (ctrlKey) partes.push("CommandOrControl");
  if (altKey) partes.push("Alt");
  if (shiftKey) partes.push("Shift");
  if (metaKey) partes.push("Super");
  partes.push(nomeDaTecla(evento));

  return partes.join("+");
}
