const CAMINHO_BANCO = "sqlite:historico.db";

export function formatarDataHistorico(isoString) {
  const data = new Date(isoString);
  if (Number.isNaN(data.getTime())) {
    return isoString;
  }
  return data.toLocaleString("pt-BR", {
    day: "2-digit",
    month: "2-digit",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

// `execute`/`select` exigem que o banco já tenha sido carregado via
// `plugin:sql|load` nesta sessão — não é criado sob demanda. Guardamos
// a Promise (não só um booleano) para que chamadas concorrentes esperem
// a mesma conexão em vez de disparar `load` mais de uma vez.
let bancoCarregado = null;

function garantirBancoCarregado() {
  const { invoke } = window.__TAURI__.core;
  if (!bancoCarregado) {
    bancoCarregado = invoke("plugin:sql|load", { db: CAMINHO_BANCO });
  }
  return bancoCarregado;
}

export async function inserirNoHistorico(original, traduzido, idioma) {
  const { invoke } = window.__TAURI__.core;
  await garantirBancoCarregado();
  await invoke("plugin:sql|execute", {
    db: CAMINHO_BANCO,
    query:
      "INSERT INTO historico (texto_original, texto_traduzido, idioma_destino, criado_em) VALUES ($1, $2, $3, $4)",
    values: [original, traduzido, idioma, new Date().toISOString()],
  });
}

export async function carregarHistorico() {
  const { invoke } = window.__TAURI__.core;
  await garantirBancoCarregado();
  return await invoke("plugin:sql|select", {
    db: CAMINHO_BANCO,
    query: "SELECT * FROM historico ORDER BY id DESC LIMIT 200",
    values: [],
  });
}
