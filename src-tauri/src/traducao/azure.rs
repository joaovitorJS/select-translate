use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct AzureRequestItem {
    #[serde(rename = "Text")]
    text: String,
}

#[derive(Deserialize)]
struct AzureResponseItem {
    translations: Vec<Traducao>,
}

#[derive(Deserialize)]
struct Traducao {
    text: String,
}

fn montar_url(idioma_destino: &str) -> String {
    format!("https://api.cognitive.microsofttranslator.com/translate?api-version=3.0&to={idioma_destino}")
}

fn extrair_traducao(corpo_json: &str) -> Result<String, String> {
    let resposta: Vec<AzureResponseItem> = serde_json::from_str(corpo_json)
        .map_err(|e| format!("Resposta inesperada do Azure Translator: {e}"))?;

    resposta
        .first()
        .and_then(|item| item.translations.first())
        .map(|t| t.text.clone())
        .ok_or_else(|| "Nenhuma tradução retornada pelo Azure Translator".to_string())
}

pub async fn traduzir(
    texto: &str,
    idioma_destino: &str,
    api_key: &str,
    regiao: &str,
) -> Result<String, String> {
    let cliente = reqwest::Client::new();
    let resposta = cliente
        .post(montar_url(idioma_destino))
        .header("Ocp-Apim-Subscription-Key", api_key)
        .header("Ocp-Apim-Subscription-Region", regiao)
        .header("Content-Type", "application/json")
        .json(&vec![AzureRequestItem {
            text: texto.to_string(),
        }])
        .send()
        .await
        .map_err(|e| format!("Falha ao conectar com o Azure Translator: {e}"))?;

    if !resposta.status().is_success() {
        return Err(format!("Erro da API do Azure Translator: {}", resposta.status()));
    }

    let corpo = resposta
        .text()
        .await
        .map_err(|e| format!("Falha ao ler resposta do Azure Translator: {e}"))?;

    extrair_traducao(&corpo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn montar_url_inclui_idioma_de_destino() {
        assert_eq!(
            montar_url("pt"),
            "https://api.cognitive.microsofttranslator.com/translate?api-version=3.0&to=pt"
        );
    }

    #[test]
    fn extrair_traducao_extrai_o_primeiro_texto() {
        let json = r#"[{"translations":[{"text":"Olá, mundo!","to":"pt"}]}]"#;
        assert_eq!(extrair_traducao(json).unwrap(), "Olá, mundo!");
    }

    #[test]
    fn extrair_traducao_falha_com_lista_vazia() {
        let json = r#"[]"#;
        assert!(extrair_traducao(json).is_err());
    }

    #[test]
    fn extrair_traducao_falha_com_json_invalido() {
        assert!(extrair_traducao("isso não é json").is_err());
    }
}
