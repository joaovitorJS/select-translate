mod azure;
mod deepl;

use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

/// Chave canônica de idioma (ex: "pt-br") -> label + código específico
/// de cada provedor. Adicionar um idioma novo é só acrescentar uma
/// entrada aqui e no <select> de src/config.js.
const IDIOMAS: &[(&str, &str, &str)] = &[
    // (valor_canonico, codigo_deepl, codigo_azure)
    ("pt-br", "PT-BR", "pt"),
    ("pt-pt", "PT-PT", "pt-pt"),
    ("en", "EN-US", "en"),
    ("es", "ES", "es"),
    ("fr", "FR", "fr"),
    ("de", "DE", "de"),
    ("it", "IT", "it"),
];

const IDIOMA_PADRAO: &str = "pt-br";

/// Qual provedor de tradução usar e as credenciais dele. Lido da tela
/// de Configurações (via tauri-plugin-store), nunca hardcoded.
pub enum ConfiguracaoProvedor {
    DeepL { api_key: String },
    AzureTranslator { api_key: String, regiao: String },
}

/// Código do idioma de destino no formato esperado pelo provedor em uso.
fn codigo_idioma(config: &ConfiguracaoProvedor, idioma_canonico: &str) -> &'static str {
    let entrada = IDIOMAS
        .iter()
        .find(|(valor, _, _)| *valor == idioma_canonico)
        .unwrap_or_else(|| IDIOMAS.iter().find(|(v, _, _)| *v == IDIOMA_PADRAO).unwrap());

    match config {
        ConfiguracaoProvedor::DeepL { .. } => entrada.1,
        ConfiguracaoProvedor::AzureTranslator { .. } => entrada.2,
    }
}

pub async fn traduzir(
    config: &ConfiguracaoProvedor,
    texto: &str,
    idioma_canonico: &str,
) -> Result<String, String> {
    let idioma = codigo_idioma(config, idioma_canonico);
    match config {
        ConfiguracaoProvedor::DeepL { api_key } => deepl::traduzir(texto, idioma, api_key).await,
        ConfiguracaoProvedor::AzureTranslator { api_key, regiao } => {
            azure::traduzir(texto, idioma, api_key, regiao).await
        }
    }
}

fn ler_texto(store: &tauri_plugin_store::Store<impl tauri::Runtime>, chave: &str) -> Option<String> {
    store
        .get(chave)
        .and_then(|valor| valor.as_str().map(str::to_string))
        .filter(|s| !s.is_empty())
}

/// Lê o provedor, as credenciais e o idioma salvos pela tela de
/// Configurações (armazenados em `config.json` via tauri-plugin-store).
pub fn configuracao_do_store(app: &AppHandle) -> Result<(ConfiguracaoProvedor, String), String> {
    let store = app.store("config.json").map_err(|e| e.to_string())?;

    let idioma = ler_texto(&store, "idioma").unwrap_or_else(|| IDIOMA_PADRAO.to_string());
    let provedor = ler_texto(&store, "provedor").unwrap_or_else(|| "deepl".to_string());

    let config = match provedor.as_str() {
        "azure" => {
            let api_key = ler_texto(&store, "azure_api_key").ok_or_else(|| {
                "Configure a API key do Azure Translator na tela de Configurações.".to_string()
            })?;
            let regiao = ler_texto(&store, "azure_regiao").ok_or_else(|| {
                "Configure a região do Azure Translator na tela de Configurações.".to_string()
            })?;
            ConfiguracaoProvedor::AzureTranslator { api_key, regiao }
        }
        _ => {
            let api_key = ler_texto(&store, "deepl_api_key").ok_or_else(|| {
                "Configure a API key do DeepL na tela de Configurações.".to_string()
            })?;
            ConfiguracaoProvedor::DeepL { api_key }
        }
    };

    Ok((config, idioma))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codigo_idioma_deepl() {
        let config = ConfiguracaoProvedor::DeepL {
            api_key: "x".to_string(),
        };
        assert_eq!(codigo_idioma(&config, "pt-br"), "PT-BR");
        assert_eq!(codigo_idioma(&config, "en"), "EN-US");
    }

    #[test]
    fn codigo_idioma_azure() {
        let config = ConfiguracaoProvedor::AzureTranslator {
            api_key: "x".to_string(),
            regiao: "brazilsouth".to_string(),
        };
        assert_eq!(codigo_idioma(&config, "pt-br"), "pt");
        assert_eq!(codigo_idioma(&config, "fr"), "fr");
    }

    #[test]
    fn codigo_idioma_desconhecido_cai_no_padrao() {
        let config = ConfiguracaoProvedor::DeepL {
            api_key: "x".to_string(),
        };
        assert_eq!(codigo_idioma(&config, "klingon"), "PT-BR");
    }
}
