mod azure;
mod deepl;

/// Qual provedor de tradução usar e as credenciais dele. A Fase 5
/// substitui a leitura por variável de ambiente por uma tela de
/// Configurações de verdade, mas o formato de dados continua o mesmo.
pub enum ConfiguracaoProvedor {
    DeepL { api_key: String },
    AzureTranslator { api_key: String, regiao: String },
}

impl ConfiguracaoProvedor {
    /// Código do idioma de destino no formato esperado por cada provedor.
    /// Fixo em português do Brasil por enquanto — vira configurável na Fase 5.
    fn idioma_destino_padrao(&self) -> &'static str {
        match self {
            ConfiguracaoProvedor::DeepL { .. } => "PT-BR",
            ConfiguracaoProvedor::AzureTranslator { .. } => "pt",
        }
    }
}

pub async fn traduzir(config: &ConfiguracaoProvedor, texto: &str) -> Result<String, String> {
    let idioma = config.idioma_destino_padrao();
    match config {
        ConfiguracaoProvedor::DeepL { api_key } => deepl::traduzir(texto, idioma, api_key).await,
        ConfiguracaoProvedor::AzureTranslator { api_key, regiao } => {
            azure::traduzir(texto, idioma, api_key, regiao).await
        }
    }
}

/// Lê o provedor e as credenciais de variáveis de ambiente:
/// - `TRANSLATION_PROVIDER`: "deepl" (padrão) ou "azure"
/// - `DEEPL_API_KEY` (quando o provedor é deepl)
/// - `AZURE_TRANSLATOR_KEY` e `AZURE_TRANSLATOR_REGION` (quando o provedor é azure)
pub fn configuracao_do_ambiente() -> Result<ConfiguracaoProvedor, String> {
    let provedor = std::env::var("TRANSLATION_PROVIDER").unwrap_or_else(|_| "deepl".to_string());

    match provedor.to_lowercase().as_str() {
        "azure" => {
            let api_key = std::env::var("AZURE_TRANSLATOR_KEY")
                .map_err(|_| "AZURE_TRANSLATOR_KEY não definida".to_string())?;
            let regiao = std::env::var("AZURE_TRANSLATOR_REGION")
                .map_err(|_| "AZURE_TRANSLATOR_REGION não definida".to_string())?;
            Ok(ConfiguracaoProvedor::AzureTranslator { api_key, regiao })
        }
        "deepl" => {
            let api_key = std::env::var("DEEPL_API_KEY")
                .map_err(|_| "DEEPL_API_KEY não definida".to_string())?;
            Ok(ConfiguracaoProvedor::DeepL { api_key })
        }
        outro => Err(format!(
            "Provedor de tradução desconhecido: '{outro}'. Use 'deepl' ou 'azure'."
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idioma_destino_padrao_deepl_usa_pt_br() {
        let config = ConfiguracaoProvedor::DeepL {
            api_key: "x".to_string(),
        };
        assert_eq!(config.idioma_destino_padrao(), "PT-BR");
    }

    #[test]
    fn idioma_destino_padrao_azure_usa_pt() {
        let config = ConfiguracaoProvedor::AzureTranslator {
            api_key: "x".to_string(),
            regiao: "brazilsouth".to_string(),
        };
        assert_eq!(config.idioma_destino_padrao(), "pt");
    }
}
