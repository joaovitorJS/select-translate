use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct DeepLRequest {
    text: Vec<String>,
    target_lang: String,
}

#[derive(Deserialize)]
struct DeepLResponse {
    translations: Vec<Traducao>,
}

#[derive(Deserialize)]
struct Traducao {
    text: String,
}

/// Contas Free do DeepL usam um endpoint diferente de contas Pro;
/// a chave de contas Free sempre termina em ":fx".
fn endpoint(api_key: &str) -> &'static str {
    if api_key.trim().ends_with(":fx") {
        "https://api-free.deepl.com/v2/translate"
    } else {
        "https://api.deepl.com/v2/translate"
    }
}

fn extrair_traducao(corpo_json: &str) -> Result<String, String> {
    let resposta: DeepLResponse = serde_json::from_str(corpo_json)
        .map_err(|e| format!("Resposta inesperada da API do DeepL: {e}"))?;

    resposta
        .translations
        .first()
        .map(|t| t.text.clone())
        .ok_or_else(|| "Nenhuma tradução retornada pela API do DeepL".to_string())
}

pub async fn traduzir(texto: &str, idioma_destino: &str, api_key: &str) -> Result<String, String> {
    let cliente = reqwest::Client::new();
    let resposta = cliente
        .post(endpoint(api_key))
        .header("Authorization", format!("DeepL-Auth-Key {api_key}"))
        .json(&DeepLRequest {
            text: vec![texto.to_string()],
            target_lang: idioma_destino.to_string(),
        })
        .send()
        .await
        .map_err(|e| format!("Falha ao conectar com o DeepL: {e}"))?;

    if !resposta.status().is_success() {
        return Err(format!("Erro da API do DeepL: {}", resposta.status()));
    }

    let corpo = resposta
        .text()
        .await
        .map_err(|e| format!("Falha ao ler resposta do DeepL: {e}"))?;

    extrair_traducao(&corpo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_conta_free_usa_api_free() {
        assert_eq!(endpoint("abc123:fx"), "https://api-free.deepl.com/v2/translate");
    }

    #[test]
    fn endpoint_conta_pro_usa_api_padrao() {
        assert_eq!(endpoint("abc123"), "https://api.deepl.com/v2/translate");
    }

    #[test]
    fn extrair_traducao_extrai_o_primeiro_texto() {
        let json = r#"{"translations":[{"text":"Olá, mundo!"}]}"#;
        assert_eq!(extrair_traducao(json).unwrap(), "Olá, mundo!");
    }

    #[test]
    fn extrair_traducao_falha_com_lista_vazia() {
        let json = r#"{"translations":[]}"#;
        assert!(extrair_traducao(json).is_err());
    }

    #[test]
    fn extrair_traducao_falha_com_json_invalido() {
        assert!(extrair_traducao("isso não é json").is_err());
    }
}
