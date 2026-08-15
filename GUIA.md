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
8. [Integrando com a API do DeepL](#8-integrando-com-a-api-do-deepl)
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

Ou seja: o Tauri **não é** "um site rodando no navegador". É um programa `.exe` de verdade, que só usa a tecnologia web (HTML/CSS/JS) para desenhar a tela, da mesma forma que o próprio Windows usa HTML internamente em partes do Explorer. Toda a lógica pesada (capturar texto, falar com a API do DeepL, ler/gravar banco de dados, atalho global, bandeja do sistema) roda em **Rust**, compilado nativamente.

### Arquitetura em alto nível

```
                     ┌─────────────────────────────────────────┐
                     │              APLICATIVO (.exe)            │
                     │                                            │
  Atalho global ────►│  ┌──────────────┐                          │
  (Ctrl+Alt+T)        │  │  Rust (core)  │                          │
                     │  │              │      ┌─────────────┐    │
  Clipboard ─────────►│  │  1. Captura   ├─────►│  DeepL API   │    │
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
| Enviar para tradução | Chamada HTTP à API do DeepL, feita em Rust com a crate `reqwest`. |
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

## 8. Integrando com a API do DeepL

### 8.1. Obtendo a API key

1. Crie uma conta em [deepl.com/pro-api](https://www.deepl.com/pro-api) (plano **Free** tem 500.000 caracteres/mês, sem custo).
2. Copie a **Authentication Key** no painel da conta.
3. **Não deixe essa chave fixa no código-fonte.** Ela deve ser digitada pelo usuário na tela de configurações (seção 11) e salva localmente (ex: no SQLite ou em um arquivo de config local do Tauri).

### 8.2. Chamando a API a partir do Rust

Adicione a crate de requisições HTTP:

```
cargo add reqwest --features json
cargo add serde --features derive
cargo add tokio --features full
```

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

#[tauri::command]
async fn traduzir(texto: String, idioma_destino: String, api_key: String) -> Result<String, String> {
    let cliente = reqwest::Client::new();
    let resposta = cliente
        .post("https://api-free.deepl.com/v2/translate")
        .header("Authorization", format!("DeepL-Auth-Key {}", api_key))
        .json(&DeepLRequest {
            text: vec![texto],
            target_lang: idioma_destino,
        })
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resposta.status().is_success() {
        return Err(format!("Erro da API DeepL: {}", resposta.status()));
    }

    let corpo: DeepLResponse = resposta.json().await.map_err(|e| e.to_string())?;
    corpo.translations.first()
        .map(|t| t.text.clone())
        .ok_or_else(|| "Nenhuma tradução retornada".to_string())
}
```

> **Nota:** contas **Free** do DeepL usam o endpoint `api-free.deepl.com`; contas **Pro** usam `api.deepl.com`. Deixe isso configurável, ou detecte pelo formato da chave (chaves free terminam em `:fx`).

Trate erros comuns: chave inválida (`403`), limite de caracteres excedido (`456`), texto vazio. Mostre essas mensagens de forma amigável na interface, não como texto técnico cru.

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

O Tauri v2 mudou o modelo de segurança: por padrão, **nada é permitido** — cada plugin que a janela usa precisa ser explicitamente listado em `src-tauri/capabilities/default.json`. Esquecer isso é a causa mais comum de "meu código está certo mas não funciona" — a chamada falha **silenciosamente** no frontend.

Exemplo de `capabilities/default.json` cobrindo tudo que usamos neste guia:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Permissões da janela principal",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "clipboard-manager:default",
    "global-shortcut:allow-register",
    "global-shortcut:allow-unregister",
    "sql:default",
    "store:default",
    "autostart:default"
  ]
}
```

**Checklist ao adicionar um novo plugin:** (1) `cargo add`, (2) registrar no `.plugin()` do `main.rs`, (3) adicionar a permissão correspondente aqui. Os três passos são obrigatórios — pular qualquer um deles quebra silenciosamente.

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

- [ ] Selecionar texto no **navegador** e traduzir pelo atalho → aparece na aba Tradução.
- [ ] Selecionar texto no **Bloco de Notas** → mesmo resultado.
- [ ] Selecionar texto no **Word** → mesmo resultado.
- [ ] Selecionar texto num **PDF** (visualizador comum, não protegido) → mesmo resultado.
- [ ] Selecionar texto num **editor de código** (ex: VS Code) → mesmo resultado.
- [ ] Selecionar texto num **app de mensagens** (ex: WhatsApp Desktop, Telegram) → mesmo resultado.
- [ ] Trocar o atalho global nas configurações e confirmar que o novo atalho funciona (e o antigo não).
- [ ] Ativar o **modo automático** e confirmar que copiar texto com `Ctrl+C` dispara a tradução sozinho.
- [ ] Desativar o modo automático e confirmar que só o atalho manual funciona.
- [ ] Fechar a janela (X) e confirmar que o app continua rodando na bandeja (ícone visível, `Ctrl+Alt+T` continua funcionando).
- [ ] Abrir a aba **Histórico** e ver as traduções anteriores, mesmo após reiniciar o app.
- [ ] Rodar `npm run tauri build` e instalar o `.exe`/`.msi` gerado numa máquina limpa (ou máquina virtual) para confirmar que o instalador funciona sem as ferramentas de desenvolvimento.

---

## 16. Caminho futuro para Linux

Boa parte do código (toda a lógica de tradução, histórico, interface) é reaproveitada quase sem mudanças, porque Tauri e a maioria dos plugins usados aqui (`sql`, `store`, `clipboard-manager`, `global-shortcut`, `autostart`, `single-instance`) já são multiplataforma. O que precisa de atenção ao portar:

- **Empacotamento**: trocar `targets: ["nsis", "msi"]` por `["deb", "appimage"]` (ou rodar o build direto numa máquina/CI Linux — o Tauri não faz cross-compilation de instalador nativo entre SOs facilmente).
- **Captura via `enigo`**: a biblioteca suporta Linux, mas o comportamento pode variar entre **X11** e **Wayland** (no Wayland, simulação de teclado global tem mais restrições de segurança por design do protocolo). Vale testar cedo num ambiente Linux real.
- **Atalhos globais**: no Linux, dependendo do ambiente gráfico (GNOME, KDE, etc.), pode ser necessário orientar o usuário a registrar o atalho manualmente nas configurações do sistema, já que nem todo compositor Wayland permite atalhos globais arbitrários de apps de terceiros.
- **Ícone de bandeja**: no Linux, a bandeja do sistema depende de extensões específicas em alguns ambientes (ex: GNOME precisa de uma extensão para mostrar ícones de bandeja). Vale documentar isso para o usuário final Linux.

---

## 17. Próximos passos

Ideias para depois que a versão inicial estiver funcionando (não fazem parte do escopo original, mas são evoluções naturais):

- Suporte a múltiplos serviços de tradução (Google Translate, Microsoft Translator) com fallback automático se um estiver fora do ar.
- Detecção automática do idioma de origem (a própria API do DeepL suporta isso).
- Atalhos diferentes por idioma de destino (ex: um atalho para traduzir para inglês, outro para português).
- Exportar o histórico (CSV/JSON).
- Tema claro/escuro na interface.
- Pronúncia em áudio da tradução (text-to-speech).
- Sincronização do histórico entre dispositivos (exigiria um backend próprio — fora do escopo de "app local").
