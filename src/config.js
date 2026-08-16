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
export function validarConfigFormulario({ atalho, provedor, deeplKey, azureKey, azureRegiao }) {
  if (!atalho) {
    return "Informe um atalho.";
  }
  if (provedor === "deepl" && !deeplKey) {
    return "Informe a chave da API do DeepL.";
  }
  if (provedor === "azure" && (!azureKey || !azureRegiao)) {
    return "Informe a chave e a região do Azure Translator.";
  }
  return null;
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
