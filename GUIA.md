# Guia Completo: App Nativo Windows de Tradução por Seleção de Texto

> Guia passo a passo para construir, do zero, um aplicativo desktop nativo para Windows que traduz qualquer texto selecionado em qualquer programa, usando **Tauri v2** (Rust + interface local, sem depender de navegador). Escrito para quem está começando agora — cada etapa explica não só *o que* fazer, mas *por quê*.

## Sumário

1. [Visão geral do sistema](#1-visão-geral-do-sistema)
2. [Pré-requisitos](#2-pré-requisitos)
3. [Criando o projeto](#3-criando-o-projeto)
4. [Estrutura de pastas explicada](#4-estrutura-de-pastas-explicada)
5. [Ícone na bandeja do sistema](#5-ícone-na-bandeja-do-sistema-system-tray)
6. [Atalho global configurável](#6-atalho-global-configurável)
7. [Capturando o texto selecionado](#7-capturando-o-texto-selecionado)
8. [Integrando com provedores de tradução (DeepL e Azure Translator)](#8-integrando-com-provedores-de-tradução-deepl-e-azure-translator)
9. [Interface do app](#9-interface-do-app)
10. [Histórico persistente](#10-histórico-persistente)
11. [Tela de configurações](#11-tela-de-configurações)
12. [Rodando em segundo plano](#12-rodando-em-segundo-plano)
13. [Permissions/capabilities do Tauri v2](#13-permissionscapabilities-do-tauri-v2)
14. [Empacotando o instalável](#14-empacotando-o-instalável)
15. [Testando de ponta a ponta](#15-testando-de-ponta-a-ponta)
16. [Caminho futuro para Linux](#16-caminho-futuro-para-linux)
17. [Próximos passos](#17-próximos-passos)

---

## 1. Visão geral do sistema

### Por que Tauri?

Você pediu um app **nativo**, instalável, que **não** dependa de navegador nem seja um site. Isso descarta soluções puramente web. As alternativas nativas mais realistas eram:

| Opção | Problema para o seu caso |
|---|---|
| C# + WPF | Interface 100% nativa, mas **só roda no Windows** — você mencionou querer suporte a Linux no futuro. |
| Python + PyQt | Empacotamento em `.exe` instalável é frágil e a interface fica menos polida. |
| **Tauri (Rust)** | Roda a interface num **webview local** (o motor de renderização já embutido no seu sistema operacional — no Windows é o WebView2 da Microsoft), sem servidor, sem internet para a UI, sem navegador externo. Gera instalador nativo (`.exe`/`.msi`) no Windows e (`.deb`/`.AppImage`) no Linux, a partir de praticamente o mesmo código. |

Ou seja: o Tauri **não é** "um site rodando no navegador". É um programa `.exe` de verdade, que só usa a tecnologia web (HTML/CSS/JS) para desenhar a tela, da mesma forma que o próprio Windows usa HTML internamente em partes do Explorer. Toda a lógica pesada (capturar texto, falar com o provedor de tradução, ler/gravar banco de dados, atalho global, bandeja do sistema) roda em **Rust**, compilado nativamente.

### Arquitetura em alto nível

```
                     ┌─────────────────────────────────────────┐
                     │              APLICATIVO (.exe)            │
                     │                                            │
  Atalho global ────►│  ┌──────────────┐                          │
  (Ctrl+Alt+T)        │  │  Rust (core)  │                          │
                     │  │              │      ┌─────────────┐    │
  Clipboard ─────────►│  │  1. Captura   ├─────►│ DeepL / Azure│    │
  (modo automático)   │  │     texto     │      │  (internet)  │    │
                     │  │  2. Chama     │◄─────┤             │    │
                     │  │     DeepL     │      └─────────────┘    │
                     │  │  3. Salva no  │                          │
                     │  │     SQLite    │                          │
                     │  └──────┬───────┘                          │
                     │         │                                  │
                     │  ┌──────▼───────┐                          │
                     │  │  Janela       │  Abas: Tradução |        │
                     │  │  (webview)    │  Histórico | Config      │
                     │  └──────────────┘                          │
                     │                                            │
                     │  Ícone na bandeja do sistema (tray)        │
                     └─────────────────────────────────────────┘
```

Fluxo típico de uso (modo atalho): você seleciona um texto em qualquer app → pressiona `Ctrl+Alt+T` → o app simula um "Ctrl+C" para capturar a seleção → lê o clipboard → envia para o DeepL → mostra a tradução na janela → salva no histórico local.

Fluxo alternativo (modo automático): toda vez que você copia algo (`Ctrl+C` normal), o app detecta a mudança no clipboard e traduz automaticamente.

### Peça de tecnologia por requisito

| Seu requisito | Como resolvemos |
|---|---|
| Selecionar texto em qualquer app | Não existe API universal para "ler a seleção atual" de outro programa. Usamos o truque real que apps como PopClip/QTranslate usam: simular `Ctrl+C` (crate `enigo`) e ler o clipboard. |
| Atalho global configurável | Plugin oficial `tauri-plugin-global-shortcut`. |
| Captura automática (opcional) | Monitoramento do clipboard (polling) via `tauri-plugin-clipboard-manager`. |
| Enviar para tradução | Chamada HTTP a um provedor plugável (DeepL ou Azure Translator), feita em Rust com a crate `reqwest`. |
| Mostrar resultado numa janela própria | Janela do Tauri com HTML/CSS/JS simples (sem framework). |
| Histórico consultável | `tauri-plugin-sql` (SQLite local). |
| Rodar em segundo plano / bandeja | Tray API nativa do Tauri v2 + `tauri-plugin-single-instance`. |
| Instalador nativo | Bundler embutido do Tauri (NSIS/MSI). |

---

## 2. Pré-requisitos

Você vai instalar, nesta ordem, no Windows:

### 2.1. Rust (via rustup)

1. Acesse [rustup.rs](https://rustup.rs) e baixe o instalador (`rustup-init.exe`).
2. Execute e escolha a opção padrão (`1) Proceed with installation`).
3. Feche e reabra o terminal. Confirme com:
   ```
   rustc --version
   cargo --version
   ```

### 2.2. Microsoft C++ Build Tools

O Rust no Windows precisa do linker/compilador da Microsoft para gerar `.exe`.

1. Baixe o [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/).
2. No instalador, marque o workload **"Desenvolvimento para desktop com C++"** (Desktop development with C++).
3. Certifique-se de que o **Windows 10/11 SDK** está marcado dentro desse workload.
4. Instale (pode levar 15-30 minutos).

### 2.3. WebView2 Runtime

Esse é o motor que renderiza a interface do app. No Windows 10 (build 1803+) e Windows 11 ele **já vem pré-instalado**. Se quiser garantir, baixe o [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/) (o "Evergreen Bootstrapper").

### 2.4. Node.js

Usado só para as ferramentas de scaffolding/build do frontend (não é um servidor rodando no app final).

1. Baixe a versão **LTS** em [nodejs.org](https://nodejs.org).
2. Confirme:
   ```
   node --version
   npm --version
   ```

### 2.5. Tauri CLI

```
cargo install tauri-cli --version "^2.0.0"
```

Isso instala o comando `cargo tauri`.

### 2.6. Editor recomendado

- [VS Code](https://code.visualstudio.com/)
- Extensões: **rust-analyzer** (autocomplete/erros de Rust) e **Tauri** (oficial, syntax highlighting de `tauri.conf.json`).

---

## 3. Criando o projeto

Abra o terminal na pasta onde quer criar o projeto (por exemplo `select-translate`) e rode:

```
npm create tauri-app@latest
```

O assistente vai perguntar:
- **Project name**: `select-translate`
- **Package manager**: `npm` (padrão, sem mistério)
- **UI template**: escolha **Vanilla** — ou seja, HTML/CSS/JS puro, sem React/Vue/Svelte. Como você está começando, isso evita aprender Rust **e** um framework de frontend ao mesmo tempo.
- **UI flavor**: **TypeScript** ou **JavaScript** — pode escolher JavaScript puro para simplificar.

Entre na pasta gerada e rode o app em modo desenvolvimento (recompila e recarrega automaticamente quando você edita o código):

```
cd select-translate
npm install
npm run tauri dev
```

Na primeira vez, o Rust vai compilar todas as dependências — isso demora alguns minutos. Se uma janela abrir com a página padrão do Tauri, seu ambiente está pronto.

---

## 4. Estrutura de pastas explicada

```
select-translate/
├── src/                      # Frontend (HTML/CSS/JS) — o que aparece na janela
│   ├── index.html
│   ├── main.js
│   └── styles.css
├── src-tauri/                 # Backend em Rust — a "lógica de verdade"
│   ├── src/
│   │   └── main.rs            # Ponto de entrada do app
│   ├── capabilities/
│   │   └── default.json       # Permissões que a janela tem (ver seção 13)
│   ├── icons/                 # Ícones do app/instalador/tray
│   ├── Cargo.toml             # "package.json" do Rust — lista as dependências
│   └── tauri.conf.json        # Configuração central: janela, bundler, plugins
└── package.json
```

Regra mental simples: **tudo que precisa "conversar com o Windows"** (clipboard, atalho global, bandeja, banco de dados, chamadas HTTP) vai em `src-tauri/` (Rust). **Tudo que é tela** (o que o usuário vê e clica) vai em `src/` (HTML/CSS/JS).

---

## 5. Ícone na bandeja do sistema (system tray)

No Tauri v2, a bandeja do sistema não é mais um plugin separado — faz parte do próprio `tauri` (feature `tray-icon`, já habilitada por padrão no template).

Em `src-tauri/src/main.rs`, dentro da função `setup()` do `Builder`:

```rust
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let abrir = MenuItem::with_id(app, "abrir", "Abrir", true, None::<&str>)?;
            let sair = MenuItem::with_id(app, "sair", "Sair", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&abrir, &sair])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "abrir" => {
                        if let Some(janela) = app.get_webview_window("main") {
                            let _ = janela.show();
                            let _ = janela.set_focus();
                        }
                    }
                    "sair" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("erro ao rodar o app");
}
```

**O que isso faz:** cria um ícone na bandeja com um menu de contexto (botão direito) com "Abrir" e "Sair". Clicar em "Abrir" traz a janela de volta; "Sair" encerra o processo de verdade.

### Fechar a janela não deve encerrar o app

Por padrão, fechar a janela (o "X") mataria o processo. Queremos que ela apenas se esconda, continuando na bandeja. Interceptamos o evento de fechamento da janela:

```rust
use tauri::WindowEvent;

// dentro do .setup(), após pegar a janela "main":
let janela = app.get_webview_window("main").unwrap();
let janela_clone = janela.clone();
janela.on_window_event(move |event| {
    if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        let _ = janela_clone.hide();
    }
});
```

---

## 6. Atalho global configurável

Instale o plugin:

```
cargo add tauri-plugin-global-shortcut
npm install @tauri-apps/plugin-global-shortcut
```

Registre o plugin no `main.rs`:

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_global_shortcut::Builder::new().build())
    // ... resto do builder
```

### Registrando/trocando o atalho em runtime

Como o atalho precisa ser **configurável pelo usuário**, o registro deve acontecer dinamicamente (não fixo no código), disparado a partir da tela de configurações (seção 11). No lado Rust, expomos um *command* que o frontend chama:

```rust
use tauri_plugin_global_shortcut::GlobalShortcutExt;

#[tauri::command]
fn registrar_atalho(app: tauri::AppHandle, atalho: String) -> Result<(), String> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| e.to_string())?;

    app.global_shortcut()
        .on_shortcut(atalho.as_str(), move |app, _shortcut, _event| {
            // Aqui chamamos a função de captura de texto (seção 7)
            capturar_e_traduzir(app.clone());
        })
        .map_err(|e| e.to_string())?;

    Ok(())
}
```

E registre o command no builder: `.invoke_handler(tauri::generate_handler![registrar_atalho, /* outros commands */])`.

No frontend (`main.js`), ao salvar a configuração:

```js
import { invoke } from '@tauri-apps/api/core';

await invoke('registrar_atalho', { atalho: 'CommandOrControl+Alt+T' });
```

**Ponto de atenção importante:** se outra combinação já estiver registrada por outro programa do Windows (ex: uma tecla usada por outro app), o registro **falha silenciosamente** — não trava, mas o atalho simplesmente não funciona. No guia de UI (seção 11), trate o retorno de erro do `invoke` e mostre uma mensagem tipo "Não foi possível registrar esse atalho, tente outra combinação".

---

## 7. Capturando o texto selecionado

Esta é a parte mais "artesanal" do projeto, porque **não existe** uma API do Windows que devolva "qual texto está selecionado agora em qualquer programa". A solução prática (usada por ferramentas reais como PopClip e QTranslate) é:

1. Guardar o conteúdo atual do clipboard (para não perder o que o usuário tinha copiado antes).
2. Simular o atalho `Ctrl+C` via automação de teclado.
3. Esperar um pequeno intervalo (o sistema operacional precisa de alguns milissegundos para processar a cópia).
4. Ler o novo conteúdo do clipboard — esse é o texto que estava selecionado.
5. (Opcional) Restaurar o clipboard original depois de usar o texto.

### 7.1. Simulando o Ctrl+C

Adicione a crate `enigo` (biblioteca Rust multiplataforma — Windows, Linux e macOS — para simular teclado/mouse):

```
cargo add enigo
```

```rust
use enigo::{Enigo, Key, Keyboard, Settings};
use std::{thread, time::Duration};

fn simular_copiar() {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    enigo.key(Key::Control, enigo::Direction::Press).unwrap();
    enigo.key(Key::Unicode('c'), enigo::Direction::Click).unwrap();
    enigo.key(Key::Control, enigo::Direction::Release).unwrap();
    thread::sleep(Duration::from_millis(150)); // dá tempo do SO processar
}
```

### 7.2. Lendo o clipboard

Instale o plugin oficial de clipboard:

```
cargo add tauri-plugin-clipboard-manager
npm install @tauri-apps/plugin-clipboard-manager
```

Registre no builder: `.plugin(tauri_plugin_clipboard_manager::init())`.

```rust
use tauri_plugin_clipboard_manager::ClipboardExt;

fn capturar_e_traduzir(app: tauri::AppHandle) {
    let clipboard_original = app.clipboard().read_text().unwrap_or_default();

    simular_copiar();

    let texto_capturado = app.clipboard().read_text().unwrap_or_default();

    if !texto_capturado.trim().is_empty() && texto_capturado != clipboard_original {
        // chama a tradução (seção 8) com texto_capturado
        // depois, se quiser, restaura o clipboard original:
        let _ = app.clipboard().write_text(clipboard_original);
    }
}
```

> **Limitações a saber:** essa técnica funciona na grande maioria dos apps (navegadores, Word, Bloco de Notas, editores de código, VS Code, apps de mensagens). Pode falhar em: PDFs protegidos contra cópia, alguns apps com proteção anti-automação (jogos, alguns terminais remotos), ou quando a janela do app de destino não está em foco no momento exato do atalho. Isso é uma limitação inerente da abordagem (não existe alternativa universal), então vale documentar isso claramente na tela de ajuda do app.

### 7.3. Modo automático: monitorar o clipboard

Para o modo "automático" (traduzir sempre que o usuário copiar algo com `Ctrl+C` normal, sem precisar de atalho), rodamos um loop em segundo plano que verifica periodicamente se o clipboard mudou:

```rust
fn iniciar_monitoramento_clipboard(app: tauri::AppHandle) {
    thread::spawn(move || {
        let mut ultimo_valor = String::new();
        loop {
            thread::sleep(Duration::from_millis(800));

            let ativo = /* ler flag de configuração "modo automático ligado" */;
            if !ativo { continue; }

            if let Ok(atual) = app.clipboard().read_text() {
                if atual != ultimo_valor && !atual.trim().is_empty() {
                    ultimo_valor = atual.clone();
                    // chama a tradução com `atual`
                }
            }
        }
    });
}
```

O **toggle** entre modo manual (atalho) e automático (clipboard) é simplesmente uma flag booleana salva nas configurações (seção 11): quando o modo automático está desligado, o loop acima ignora as mudanças de clipboard e só a função do atalho (seção 6) dispara a tradução.

---

## 8. Integrando com provedores de tradução (DeepL e Azure Translator)

O app não fica travado a um único serviço de tradução: o módulo `src-tauri/src/traducao/` define um **provedor plugável**. Cada provedor vira um submódulo com sua própria chamada HTTP e seu próprio jeito de "montar o payload" e "extrair a tradução da resposta"; um dispatcher central decide qual provedor chamar. Isso resolve dois problemas de uma vez: (1) contas de API às vezes ficam temporariamente indisponíveis (foi o motivo real de adicionar um segundo provedor durante a Fase 2 deste projeto), e (2) é o mesmo ponto de extensão que a Fase 12 já previa para "múltiplos serviços de tradução" — só que resolvido cedo, de forma simples.

```
src-tauri/src/traducao/
├── mod.rs    # enum ConfiguracaoProvedor + dispatcher traduzir()
├── deepl.rs  # chamada à API do DeepL
└── azure.rs  # chamada à API do Microsoft Azure Translator (Cognitive Services)
```

### 8.1. DeepL

1. Crie uma conta em [deepl.com/pro-api](https://www.deepl.com/pro-api) (plano **Free** tem 500.000 caracteres/mês, sem custo).
2. Copie a **Authentication Key** no painel da conta.

Adicione as crates de requisições HTTP e serialização:

```
cargo add reqwest --features json
cargo add serde --features derive
```

`src-tauri/src/traducao/deepl.rs`:

```rust
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

    resposta.translations.first()
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

    let corpo = resposta.text().await.map_err(|e| format!("Falha ao ler resposta do DeepL: {e}"))?;
    extrair_traducao(&corpo)
}
```

Erros comuns: chave inválida (`403`), limite de caracteres excedido (`456`), texto vazio.

### 8.2. Microsoft Azure Translator

1. No [portal do Azure](https://portal.azure.com), crie um recurso **Translator** (Cognitive Services). O tier **F0 (gratuito)** cobre 2 milhões de caracteres/mês.
2. Na página do recurso, em "Keys and Endpoint", copie uma das chaves e a **região** (ex: `brazilsouth`) — o Azure Translator exige os dois, diferente do DeepL que só usa a chave.

`src-tauri/src/traducao/azure.rs`:

```rust
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

    resposta.first()
        .and_then(|item| item.translations.first())
        .map(|t| t.text.clone())
        .ok_or_else(|| "Nenhuma tradução retornada pelo Azure Translator".to_string())
}

pub async fn traduzir(texto: &str, idioma_destino: &str, api_key: &str, regiao: &str) -> Result<String, String> {
    let cliente = reqwest::Client::new();
    let resposta = cliente
        .post(montar_url(idioma_destino))
        .header("Ocp-Apim-Subscription-Key", api_key)
        .header("Ocp-Apim-Subscription-Region", regiao)
        .header("Content-Type", "application/json")
        .json(&vec![AzureRequestItem { text: texto.to_string() }])
        .send()
        .await
        .map_err(|e| format!("Falha ao conectar com o Azure Translator: {e}"))?;

    if !resposta.status().is_success() {
        return Err(format!("Erro da API do Azure Translator: {}", resposta.status()));
    }

    let corpo = resposta.text().await.map_err(|e| format!("Falha ao ler resposta do Azure Translator: {e}"))?;
    extrair_traducao(&corpo)
}
```

Repare que a API do Azure espera um **array** de objetos (`[{"Text": "..."}]`) tanto no request quanto na resposta, diferente do DeepL — por isso `extrair_traducao` aqui indexa `resposta.first()` antes de `.translations`.

### 8.3. O dispatcher

`src-tauri/src/traducao/mod.rs` decide qual provedor usar e traduz o idioma de destino para o formato que cada API espera (o DeepL usa `"PT-BR"`; o Azure usa `"pt"`):

```rust
mod azure;
mod deepl;

pub enum ConfiguracaoProvedor {
    DeepL { api_key: String },
    AzureTranslator { api_key: String, regiao: String },
}

impl ConfiguracaoProvedor {
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
        ConfiguracaoProvedor::AzureTranslator { api_key, regiao } =>
            azure::traduzir(texto, idioma, api_key, regiao).await,
    }
}
```

Até a Fase 5 (tela de Configurações), a escolha do provedor e as credenciais vêm de variáveis de ambiente lidas em runtime — **nunca commitadas**:

```rust
pub fn configuracao_do_ambiente() -> Result<ConfiguracaoProvedor, String> {
    let provedor = std::env::var("TRANSLATION_PROVIDER").unwrap_or_else(|_| "deepl".to_string());
    match provedor.to_lowercase().as_str() {
        "azure" => Ok(ConfiguracaoProvedor::AzureTranslator {
            api_key: std::env::var("AZURE_TRANSLATOR_KEY").map_err(|_| "AZURE_TRANSLATOR_KEY não definida".to_string())?,
            regiao: std::env::var("AZURE_TRANSLATOR_REGION").map_err(|_| "AZURE_TRANSLATOR_REGION não definida".to_string())?,
        }),
        "deepl" => Ok(ConfiguracaoProvedor::DeepL {
            api_key: std::env::var("DEEPL_API_KEY").map_err(|_| "DEEPL_API_KEY não definida".to_string())?,
        }),
        outro => Err(format!("Provedor de tradução desconhecido: '{outro}'. Use 'deepl' ou 'azure'.")),
    }
}
```

No PowerShell, antes de rodar `npm run tauri dev`:

```powershell
$env:TRANSLATION_PROVIDER = "azure"
$env:AZURE_TRANSLATOR_KEY = "sua-chave-aqui"
$env:AZURE_TRANSLATOR_REGION = "brazilsouth"
```

**Adicionando um terceiro provedor no futuro:** crie `traducao/novo_provedor.rs` com uma `pub async fn traduzir(texto, idioma, ...)`, adicione uma variante no enum `ConfiguracaoProvedor`, e um braço no `match` de `traduzir()` e de `idioma_destino_padrao()`. O resto do app (captura, UI, histórico) não muda nada.

---

## 9. Interface do app

Mantendo simples (vanilla JS), a janela principal tem três abas: **Tradução**, **Histórico**, **Configurações**.

`src/index.html` (esqueleto):

```html
<!doctype html>
<html lang="pt-br">
<head>
  <meta charset="UTF-8" />
  <title>Select Translate</title>
  <link rel="stylesheet" href="styles.css" />
</head>
<body>
  <nav class="abas">
    <button data-aba="traducao" class="ativa">Tradução</button>
    <button data-aba="historico">Histórico</button>
    <button data-aba="config">Configurações</button>
  </nav>

  <section id="aba-traducao" class="painel">
    <h2>Texto original</h2>
    <p id="texto-original">—</p>
    <h2>Tradução</h2>
    <p id="texto-traduzido">—</p>
  </section>

  <section id="aba-historico" class="painel oculto">
    <ul id="lista-historico"></ul>
  </section>

  <section id="aba-config" class="painel oculto">
    <label>Atalho global: <input id="input-atalho" value="Ctrl+Alt+T" /></label>
    <label>Idioma de destino: <select id="select-idioma"></select></label>
    <label><input type="checkbox" id="check-automatico" /> Modo automático (traduzir ao copiar)</label>
    <label>Chave da API DeepL: <input id="input-api-key" type="password" /></label>
    <button id="btn-salvar-config">Salvar</button>
  </section>

  <script type="module" src="main.js"></script>
</body>
</html>
```

`src/main.js` escuta um evento do backend (disparado toda vez que uma tradução termina) e atualiza a aba "Tradução":

```js
import { listen } from '@tauri-apps/api/event';

listen('nova-traducao', (evento) => {
  document.getElementById('texto-original').textContent = evento.payload.original;
  document.getElementById('texto-traduzido').textContent = evento.payload.traduzido;
});
```

No Rust, depois de chamar `traduzir()` com sucesso, emitimos esse evento e trazemos a janela para frente:

```rust
use tauri::Emitter;

app.emit("nova-traducao", serde_json::json!({
    "original": texto_original,
    "traduzido": texto_traduzido,
})).ok();

if let Some(janela) = app.get_webview_window("main") {
    let _ = janela.show();
    let _ = janela.set_focus();
}
```

---

## 10. Histórico persistente

Instale o plugin oficial de SQLite:

```
cargo add tauri-plugin-sql --features sqlite
npm install @tauri-apps/plugin-sql
```

Registre no builder e defina a migração inicial (cria a tabela na primeira execução):

```rust
use tauri_plugin_sql::{Migration, MigrationKind};

let migracoes = vec![Migration {
    version: 1,
    description: "cria tabela de historico",
    sql: "CREATE TABLE historico (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            texto_original TEXT NOT NULL,
            texto_traduzido TEXT NOT NULL,
            idioma_destino TEXT NOT NULL,
            criado_em TEXT NOT NULL
          );",
    kind: MigrationKind::Up,
}];

tauri::Builder::default()
    .plugin(
        tauri_plugin_sql::Builder::default()
            .add_migrations("sqlite:historico.db", migracoes)
            .build(),
    )
    // ...
```

No frontend, gravar e consultar:

```js
import Database from '@tauri-apps/plugin-sql';

const db = await Database.load('sqlite:historico.db');

async function salvarNoHistorico(original, traduzido, idioma) {
  await db.execute(
    'INSERT INTO historico (texto_original, texto_traduzido, idioma_destino, criado_em) VALUES ($1, $2, $3, $4)',
    [original, traduzido, idioma, new Date().toISOString()]
  );
}

async function carregarHistorico() {
  return await db.select('SELECT * FROM historico ORDER BY id DESC LIMIT 200');
}
```

Popule a aba "Histórico" chamando `carregarHistorico()` quando o usuário clicar nessa aba, renderizando uma lista simples com data, texto original e traduzido.

---

## 11. Tela de configurações

As configurações (atalho, idioma alvo, modo automático, API key) precisam ser **persistidas** entre execuções do app. A forma mais simples é guardá-las na mesma base SQLite (uma tabela `configuracoes` com pares chave/valor), ou usar o plugin `tauri-plugin-store` (armazenamento de JSON simples em disco) — para um iniciante, o `store` é mais direto:

```
cargo add tauri-plugin-store
npm install @tauri-apps/plugin-store
```

```js
import { Store } from '@tauri-apps/plugin-store';

const config = await Store.load('config.json');

document.getElementById('btn-salvar-config').addEventListener('click', async () => {
  const atalho = document.getElementById('input-atalho').value;
  const idioma = document.getElementById('select-idioma').value;
  const automatico = document.getElementById('check-automatico').checked;
  const apiKey = document.getElementById('input-api-key').value;

  await config.set('atalho', atalho);
  await config.set('idioma_destino', idioma);
  await config.set('modo_automatico', automatico);
  await config.set('api_key', apiKey);
  await config.save();

  try {
    await invoke('registrar_atalho', { atalho });
    alert('Configurações salvas!');
  } catch (erro) {
    alert('Atalho inválido ou já em uso: ' + erro);
  }
});
```

Ao iniciar o app, carregue essas configurações e aplique-as (registre o atalho salvo, ligue/desligue o loop de monitoramento automático, etc.) — isso garante que as preferências do usuário sejam mantidas entre uma abertura e outra do programa.

---

## 12. Rodando em segundo plano

### 12.1. Evitar múltiplas instâncias

Instale:

```
cargo add tauri-plugin-single-instance
```

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
        // se o usuário tentar abrir o app de novo, apenas mostra a janela existente
        if let Some(janela) = app.get_webview_window("main") {
            let _ = janela.show();
            let _ = janela.set_focus();
        }
    }))
    // IMPORTANTE: este deve ser o PRIMEIRO plugin registrado no builder
    .plugin(tauri_plugin_global_shortcut::Builder::new().build())
    // ... demais plugins
```

### 12.2. Iniciar com o Windows (opcional, mas recomendado para um app de bandeja)

```
cargo add tauri-plugin-autostart
npm install @tauri-apps/plugin-autostart
```

```rust
.plugin(tauri_plugin_autostart::init(
    tauri_plugin_autostart::MacosLauncher::LaunchAgent,
    None,
))
```

Exponha um toggle na tela de configurações para o usuário ativar/desativar isso via `enable()`/`disable()` do plugin no frontend.

---

## 13. Permissions/capabilities do Tauri v2

O Tauri v2 mudou o modelo de segurança: por padrão, **nada é permitido** — cada *command de plugin chamado pelo frontend* precisa ser explicitamente listado em `src-tauri/capabilities/default.json`. Esquecer isso é a causa mais comum de "meu código está certo mas não funciona" — a chamada falha **silenciosamente** no frontend.

**Ponto importante (só descoberto na prática, na Fase 8 deste projeto):** essa permissão protege a ponte de IPC **frontend → backend**, isto é, só entra em jogo quando o JavaScript chama `invoke("plugin:xxx|yyy")`. Se o seu código Rust usa a extension trait de um plugin diretamente (ex: `app.clipboard()...` do `tauri-plugin-clipboard-manager`, ou `app.global_shortcut()...` do `tauri-plugin-global-shortcut`) e o frontend **nunca** invoca aquele plugin diretamente, você não precisa da permissão dele nas capabilities — só dos plugins cujos commands o `invoke()` do frontend realmente chama (no nosso caso, `sql` e `store`; `clipboard-manager` e `global-shortcut` continuam registrados como plugin no `.plugin()`, só não aparecem nas capabilities). Vale a pena revisar isso periodicamente (a Fase 8 do `FASES.md` é dedicada a essa auditoria) — não custa nada ter uma permissão sobrando, mas também não serve pra nada além de aumentar a superfície de IPC exposta ao frontend.

Exemplo de `capabilities/default.json` deste projeto, já depois dessa auditoria — note que só lista os plugins de fato invocados via `invoke()` no frontend:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Permissões da janela principal",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "sql:default",
    "sql:allow-load",
    "sql:allow-execute",
    "sql:allow-select",
    "store:default",
    "store:allow-load",
    "store:allow-get",
    "store:allow-set",
    "store:allow-save"
  ]
}
```

**Checklist ao adicionar um novo plugin:** (1) `cargo add`, (2) registrar no `.plugin()` do `main.rs`, (3) se (e só se) o **frontend** for chamar `invoke("plugin:nome|comando")` diretamente, adicionar a permissão correspondente aqui — teste sempre, porque os nomes exatos de permissão (`allow-execute`, `allow-select` etc.) nem sempre estão cobertos por `plugin:default`, como aconteceu com `sql`/`store` neste projeto (ver Fases 4 e 5 do `FASES.md`).

---

## 14. Empacotando o instalável

No `src-tauri/tauri.conf.json`, configure a seção `bundle`:

```json
{
  "bundle": {
    "active": true,
    "targets": ["nsis", "msi"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.ico"
    ],
    "windows": {
      "nsis": {
        "installMode": "currentUser",
        "languages": ["Portuguese", "English"]
      }
    }
  }
}
```

- `nsis` gera um instalador `.exe` (mais comum, mais flexível visualmente).
- `msi` gera um `.msi` (formato padrão Windows Installer, útil se sua empresa/ambiente exigir).

Gere os ícones em vários tamanhos a partir de uma imagem única com a própria CLI do Tauri:

```
npm run tauri icon caminho/para/seu-icone.png
```

Para gerar o instalador final:

```
npm run tauri build
```

O(s) instalador(es) aparecem em `src-tauri/target/release/bundle/nsis/` e `.../msi/`. Esse é o arquivo que você distribui e executa para instalar o app numa máquina Windows — sem precisar de Rust, Node ou nada do ambiente de desenvolvimento instalado nela.

---

## 15. Testando de ponta a ponta

Checklist manual para validar cada requisito original antes de considerar o app pronto:

- [x] Selecionar texto no **navegador** e traduzir pelo atalho → aparece na aba Tradução. *(validado nas Fases 2-4)*
- [x] Selecionar texto no **Bloco de Notas** → mesmo resultado. *(validado nas Fases 2-4)*
- [ ] Selecionar texto no **Word** → mesmo resultado. **Não testado** — usuário não tem Word instalado.
- [x] Selecionar texto num **PDF** (visualizador comum, não protegido) → mesmo resultado. *(validado na Fase 10, Edge)*
- [x] Selecionar texto num **editor de código** (ex: VS Code) → mesmo resultado. *(validado na Fase 10)*
- [ ] Selecionar texto num **app de mensagens** (ex: WhatsApp Desktop, Telegram) → mesmo resultado. **Não testado** — usuário não tem nenhum dos dois instalado.
- [x] Trocar o atalho global nas configurações e confirmar que o novo atalho funciona (e o antigo não). *(validado na Fase 10 — trocado para `CommandOrControl+Alt+Y`, confirmado nos logs e pelo usuário; mantido assim por escolha do usuário)*
- [x] Ativar o **modo automático** e confirmar que copiar texto com `Ctrl+C` dispara a tradução sozinho. *(validado na Fase 6)*
- [x] Desativar o modo automático e confirmar que só o atalho manual funciona. *(validado na Fase 6)*
- [x] Fechar a janela (X) e confirmar que o app continua rodando na bandeja (ícone visível, o atalho continua funcionando). *(validado na Fase 7)*
- [x] Abrir a aba **Histórico** e ver as traduções anteriores, mesmo após reiniciar o app. *(validado na Fase 4)*
- [x] Rodar `npm run tauri build` e instalar o `.exe`/`.msi` gerado (mesma máquina de dev, sem VM limpa disponível) para confirmar que o instalador funciona standalone. *(validado na Fase 9, com a ressalva sobre a máquina de teste)*

---

## 16. Caminho futuro para Linux

*(Validado na Fase 11, num Ubuntu 26.04 dentro do WSL2 com WSLg — dá um compositor Wayland de verdade com bridge X11/XWayland, então dá pra testar as duas pontas sem precisar de uma máquina Linux física.)*

Boa parte do código (toda a lógica de tradução, histórico, interface) é reaproveitada quase sem mudanças, porque Tauri e a maioria dos plugins usados aqui (`sql`, `store`, `clipboard-manager`, `global-shortcut`, `autostart`, `single-instance`) já são multiplataforma — confirmado: `cargo build`/`cargo test` rodam limpos no Linux sem alterar nada da lógica de negócio (traducao/, historico), e todos os testes Rust (18) e JS (16) passam sem mudança.

- **Ambiente de build**: precisa do toolchain Rust (`rustup`) e das libs de sistema do Tauri v2 — `libwebkit2gtk-4.1-dev`, `build-essential`, `curl`, `wget`, `file`, `libxdo-dev`, `libssl-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev` (via `apt`). O CLI do Tauri via `npm`/`node_modules` só carrega o binding nativo do SO em que rodou `npm install` — numa máquina Windows+WSL com o projeto no `/mnt/c` (`node_modules` compartilhado entre os dois lados), rodar `npm run tauri` no Linux quebra com "Cannot find native binding"; a solução é instalar o CLI separado via `cargo install tauri-cli --version "^2" --locked` e usar `cargo tauri build`/`cargo tauri dev`, que não depende do `node_modules`. **Atenção com o mount 9p do `/mnt/c`**: o cache incremental do Cargo não confia nos mtimes desse filesystem, então builds sucessivos (mesmo sem mudar nada) recompilam tudo do zero — leva minutos, não é bug.
- **Empacotamento**: em vez de trocar `targets: ["nsis", "msi"]` por `["deb", "appimage"]` (o que quebraria o build no Windows), o `tauri.conf.json` usa `"targets": "all"` — o Tauri resolve sozinho pra `nsis`+`msi` no Windows e `deb`+`appimage` no Linux a partir do **mesmo config**, sem precisar de arquivo `tauri.linux.conf.json` separado. Validado: `cargo tauri build --bundles deb,appimage` gera os dois pacotes; o `.deb` foi inspecionado com `dpkg-deb -I/-c` (dependências corretas: `libayatana-appindicator3-1`, `libwebkit2gtk-4.1-0`, `libgtk-3-0`) e o `.AppImage` roda standalone (`--appimage-extract-and-run`, útil em ambientes sem FUSE como WSL) — os dois testados de ponta a ponta (atalho global + captura, ver item abaixo).
- **Captura via `enigo` — bug real encontrado e corrigido**: o código da Fase 2 usava `Key::Other(VK_C)` com `VK_C = 0x43`, um Virtual-Key code do Windows. No Linux, `Key::Other(v)` não é um scancode — vira um **keysym** X11/Wayland, e por coincidência `0x43` também é o keysym de "C" **maiúsculo** (`XK_C`). O enigo sintetiza isso pressionando Shift junto, então o app de destino recebia **Ctrl+Shift+C** em vez de **Ctrl+C** — um atalho diferente (não ligado a copiar na maioria dos apps), e a captura falhava silenciosamente (log "Nenhum texto novo selecionado", clipboard nunca mudava). Corrigido isolando a escolha da tecla numa função `tecla_copiar()` (`src-tauri/src/captura.rs`) com `#[cfg(target_os = "windows")]`: Windows continua com `Key::Other(VK_C)` (motivo documentado na Fase 2 — `Key::Unicode` no Windows ignora o Ctrl pressionado); as demais plataformas usam `Key::Unicode('c')` minúsculo, que no Linux resolve pro keysym certo sem precisar de Shift. Validado ponta a ponta: janela Tk (X11 via XWayland) com texto pré-selecionado, atalho global disparado de verdade via `xdotool key ctrl+alt+t`, clipboard conferido com `xclip` — o texto certo foi capturado.
- **X11 vs Wayland — o que foi validado e o que não**: a suíte padrão do enigo (feature default `x11rb`, a única habilitada aqui — `enigo = "0.6.1"` sem features extras no `Cargo.toml`) só sabe entregar teclas simuladas via **XTest**, que só alcança clientes X11 "de verdade" ou clientes Wayland rodando através do **XWayland** (a maioria dos apps GTK3/Qt5/Electron mais antigos, e basicamente tudo que roda numa sessão X11 pura). Isso foi validado com sucesso. O que **não** foi validado — e é uma limitação estrutural, não um bug: um cliente **Wayland nativo** (sem XWayland) não recebe nada via XTest, porque o XTest fica só na "bolha" do X11; pra alcançar esse caso o enigo precisaria da feature `wayland` (protocolo `virtual-keyboard-unstable-v1`), que a maioria dos compositores desktop populares (GNOME em primeiro lugar) **não implementa** por ser um protocolo privilegiado — restrição de segurança por design do Wayland, não uma questão de código. Não havia um app Wayland nativo disponível neste ambiente de teste (WSLg) pra confirmar isso na prática. Recomendação pra quando for revisitar: se aparecer relato de usuário Linux onde a captura simplesmente não funciona em determinado app (sem erro, só "Nenhum texto novo selecionado"), o primeiro suspeito é esse — o app alvo rodando Wayland nativo.
- **Atalhos globais**: `tauri-plugin-global-shortcut` (via `global-hotkey`) registrou e disparou `CommandOrControl+Alt+T` normalmente sob X11/XWayland (mesmo ambiente de teste acima). Sob um compositor Wayland "de verdade" com um shell completo (GNOME, KDE — diferente do WSLg, que não tem shell), o mesmo tipo de restrição do item anterior pode se aplicar: nem todo compositor deixa um app de terceiros capturar uma combinação de teclas globalmente, podendo exigir que o usuário registre o atalho manualmente nas configurações do sistema. Não testado num compositor com shell completo.
- **Ícone de bandeja**: o `TrayIconBuilder`/`libayatana-appindicator` não deu erro fatal, mas também não apareceu em lugar nenhum — o WSLg não tem um host de bandeja (nenhum shell rodando, só o compositor). Isso é consistente com o aviso original de que GNOME precisa de uma extensão pra mostrar ícones de bandeja: a causa raiz é a mesma em qualquer ambiente sem um host de "system tray" ativo (`StatusNotifierWatcher`), não é exclusivo do GNOME. Vale documentar isso pro usuário final Linux (o app continua funcionando normalmente sem a bandeja visível — só perde o acesso rápido por ali).
- **Metadados do pacote**: `Cargo.toml` tinha `description`/`authors` com texto específico do Windows (`"...em qualquer programa do Windows"`) e um placeholder (`authors = ["you"]`), herdados da Fase 9. Isso vira `Description`/`Maintainer` no `.deb` de verdade — corrigido na Fase 11 pra texto neutro de plataforma e o handle do GitHub do autor.
- **glibc: builda sempre na distro mais antiga que você quer suportar, não na sua** — os pacotes `.deb`/`.AppImage` da v0.2.1 foram compilados direto no Ubuntu 26.04 (o host de desenvolvimento) e vieram exigindo `GLIBC_2.38`/`2.39`; num Ubuntu 22.04 LTS de verdade (glibc 2.35) o binário nem iniciava — `select-translate: /lib/x86_64-linux-gnu/libc.so.6: version 'GLIBC_2.38' not found`. glibc é retrocompatível (um binário linkado contra uma versão mais velha roda em qualquer sistema com uma versão igual ou mais nova), nunca o contrário — então a distro onde você compila vira, na prática, a versão mínima suportada. Corrigido buildando dentro de um container Docker baseado em `ubuntu:22.04` (`docker/linux-builder.Dockerfile`, na raiz do repo) — o binário resultante passou a exigir só até `GLIBC_2.34` (confirmado com `objdump -T select-translate | grep GLIBC_ | sort -V`), compatível com Ubuntu 22.04+ e equivalentes. Vale usar esse Dockerfile em toda release futura com artefato Linux, em vez de buildar direto no host.

---

## 17. Próximos passos

Ideias para depois que a versão inicial estiver funcionando (não fazem parte do escopo original, mas são evoluções naturais):

- Mais um provedor de tradução (ex: Google Translate) seguindo o mesmo padrão de `traducao/` (§8.3), com fallback automático se um estiver fora do ar.
- Detecção automática do idioma de origem (a própria API do DeepL suporta isso).
- Atalhos diferentes por idioma de destino (ex: um atalho para traduzir para inglês, outro para português).
- Exportar o histórico (CSV/JSON).
- Tema claro/escuro na interface.
- Pronúncia em áudio da tradução (text-to-speech).
- Sincronização do histórico entre dispositivos (exigiria um backend próprio — fora do escopo de "app local").
