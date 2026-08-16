# Fases de Desenvolvimento — Select Translate

> Roteiro de execução baseado no [`GUIA.md`](./GUIA.md). Cada fase entrega algo **testável** antes de passar para a próxima — a ideia é nunca ficar mais de uma fase sem conseguir rodar o app e ver algo funcionando. As referências entre parênteses apontam para a seção correspondente do guia técnico.

## Como usar este documento

- Marque as caixas conforme for concluindo.
- Cada fase tem um **critério de pronto** — não avance para a próxima fase sem satisfazê-lo, mesmo que o código pareça "quase lá".
- Fases 0 a 5 formam o **MVP** (produto mínimo utilizável no dia a dia). Fases 6 em diante são incrementos sobre o MVP.

---

## Fase 0 — Ambiente preparado
*(GUIA.md §2)*

Objetivo: ter tudo instalado e validado antes de escrever qualquer código do projeto.

- [x] Rust instalado via rustup (`rustc --version` funciona) — `rustc 1.97.1`, `cargo 1.97.1`
- [x] Visual Studio Build Tools com workload C++ Desktop instalado — `cl.exe` confirmado em `VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64\`
- [x] WebView2 Runtime confirmado no Windows — já vinha pré-instalado
- [x] Node.js LTS instalado (`node --version`, `npm --version`) — `v24.19.0` / `npm 11.17.0`
- [x] Tauri CLI instalado (`cargo tauri --version`) — `tauri-cli 2.11.4`
- [x] VS Code + extensões rust-analyzer e Tauri — `rust-lang.rust-analyzer` e `tauri-apps.tauri-vscode` instaladas

**Critério de pronto:** `npm create tauri-app@latest` roda sem erros de dependência faltando. ✅ Ferramentas validadas em 2026-08-15.

**Observações:**
- A workload C++ do Build Tools, com `--add Microsoft.VisualStudio.Workload.VCTools` sozinho, instalou só os componentes *obrigatórios* e ficou sem o compilador (`cl.exe`), pois ele é *recomendado*, não obrigatório, dentro da workload. Foi necessário reinstalar com a flag `--includeRecommended` para trazer o `Microsoft.VisualStudio.Component.VC.Tools.x86.x64`. Se for reinstalar em outra máquina, já use `--includeRecommended` de primeira.
- O instalador do Node.js deixou o `npm` bloqueado no PowerShell pela política de execução de scripts padrão (`Restricted`). Foi ajustado para `RemoteSigned` no escopo do usuário atual (`Set-ExecutionPolicy -Scope CurrentUser -ExecutionPolicy RemoteSigned`), que é o mínimo necessário para os scripts `.ps1` do npm rodarem.

---

## Fase 1 — Scaffold e "Hello World" nativo
*(GUIA.md §3, §4)*

Objetivo: projeto Tauri criado, rodando em modo dev, com a janela padrão abrindo.

- [x] Projeto criado com template `vanilla` + JavaScript — via `npm create tauri-app@latest -- select-translate-scaffold --manager npm --template vanilla -y`
- [x] `npm run tauri dev` abre a janela padrão sem erros — compilou em 3m58s (`dev profile`) e a janela abriu, confirmado pelo usuário
- [x] Estrutura de pastas (`src/`, `src-tauri/`) entendida e explorada
- [x] Primeiro commit do projeto — commit do scaffold nesta mesma fase

**Critério de pronto:** você consegue editar `src/index.html`, salvar, e ver a mudança refletida na janela automaticamente (hot reload). ✅ Janela abriu sem erros, confirmado pelo usuário em 2026-08-15. O teste explícito de hot-reload (editar e salvar `index.html` com o dev server rodando) não foi verificado passo a passo nesta sessão — vale confirmar na prática antes da Fase 3.

**Observações:**
- O projeto foi movido de `/home/joaovitor/www/experiments/select-translate` (WSL) para `C:\Users\joaov\projetos\select-translate` (disco nativo do Windows, também acessível pelo WSL em `/mnt/c/Users/joaov/projetos/select-translate`). Motivo: ferramentas Windows (`npm`, `cargo`) spawnam sub-processos via `cmd.exe`, que **não suporta caminho UNC** (`\\wsl.localhost\...`) como diretório de trabalho — silenciosamente cai para `C:\Windows`. Trabalhar num caminho nativo do Windows evita esse problema. O Git funciona normalmente a partir de `/mnt/c/...` pelo WSL.
- `git config core.fileMode false` foi definido neste repositório porque o mount do Windows (`drvfs`) marca todos os arquivos como executáveis, gerando diffs de modo de arquivo sem mudança real de conteúdo.
- `create-tauri-app` sempre cria uma subpasta nova com o nome do projeto; o scaffold foi gerado numa pasta temporária irmã (`select-translate-scaffold`) e depois movido para a raiz do projeto. Os nomes internos (`package.json`, `src-tauri/Cargo.toml`, `tauri.conf.json`, `main.rs`) foram renomeados de `select-translate-scaffold`/`select_translate_scaffold_lib` para `select-translate`/`select_translate_lib`.

---

## Fase 2 — MVP de captura e tradução (sem UI, sem histórico)
*(GUIA.md §6, §7, §8)*

Objetivo: provar que o núcleo funciona — selecionar texto em qualquer app, apertar um atalho fixo, ver a tradução aparecer (mesmo que só num `println!`/`console.log` por enquanto). Esta é a fase de **maior risco técnico** do projeto — se algo não vai funcionar como esperado, é aqui que vai aparecer.

- [x] Atalho global fixo registrado (`CommandOrControl+Alt+T`, ainda não configurável)
- [x] Simulação de `Ctrl+C` com `enigo` funcionando
- [x] Leitura do clipboard após a simulação funcionando
- [x] Módulo de tradução com suporte a múltiplos provedores (`src-tauri/src/traducao/`: `deepl.rs`, `azure.rs`, dispatcher em `mod.rs`)
- [x] Conta e API key obtidas do DeepL (Azure Translator implementado e testado só via unit tests — sem credencial real testada ainda)
- [x] Chamada funcionando via `reqwest` ao provedor configurado, com credenciais lidas de variáveis de ambiente (`TRANSLATION_PROVIDER`, `DEEPL_API_KEY` ou `AZURE_TRANSLATOR_KEY`/`AZURE_TRANSLATOR_REGION`) — nunca hardcoded no código, nunca commitado
- [x] Resultado aparece em qualquer lugar visível (terminal com `println!` de debug + `alert()` na janela via evento `nova-traducao`)

**Critério de pronto:** ✅ Validado em 2026-08-15 com o Bloco de Notas e uma página web no Edge, usando DeepL — texto selecionado, `Ctrl+Alt+T`, tradução em português apareceu no `alert()`. Word e PDF ainda não testados manualmente; Azure Translator não testado com credencial real (só unit tests). Vale confirmar isso antes da Fase 10.

**Observações:**
- Decisão tomada durante a implementação: o app suporta **múltiplos provedores de tradução** desde já (não só na Fase 12 como estava planejado originalmente) — motivo prático: a conta DeepL do usuário ficou temporariamente inacessível, então o Microsoft Azure Translator foi adicionado como alternativa. A arquitetura usa um enum `ConfiguracaoProvedor` (`DeepL { api_key }` / `AzureTranslator { api_key, regiao }`) com um `match` despachando para o submódulo certo — adicionar um novo provedor no futuro segue o mesmo padrão.
- **Dois bugs reais encontrados e corrigidos na simulação de Ctrl+C** (não estavam previstos no `GUIA.md` original — o guia foi atualizado com o código corrigido):
  1. `enigo` tem dois jeitos de simular uma tecla: `Key::Unicode(char)` (injeta o caractere diretamente, ignorando modificadores como Ctrl) e `Key::Other(codigo_vk)` (simula a tecla física de verdade). Usar `Key::Unicode('c')` fazia o app de destino receber um "c" digitado (ou um símbolo, dependendo do layout de teclado) em vez do atalho de copiar. Corrigido usando `Key::Other(0x43)` (VK_C do Windows).
  2. Mesmo com a tecla certa, o atalho ainda falhava: no instante em que o atalho global (`Ctrl+Alt+T`) dispara, as teclas físicas ainda estão pressionadas pelo usuário. Simular Ctrl+C nesse momento fazia o app de destino receber `Ctrl+Alt+C` (Alt real ainda down), não `Ctrl+C`. Corrigido soltando explicitamente Alt/Shift/Ctrl (via `Direction::Release`) antes de simular o Ctrl+C de verdade. Esse é um padrão a repetir em qualquer automação de teclado disparada a partir de um atalho global.
- Logs de debug (`[select-translate] [debug] Clipboard antes/depois: ...`) foram deixados no código propositalmente — úteis para diagnosticar a Fase 6 (modo automático) também. Podem ser removidos/reduzidos quando a Fase 3 substituir esse fluxo por UI de verdade.

---

## Fase 3 — Interface real (abas Tradução / Histórico / Configurações)
*(GUIA.md §9)*

Objetivo: substituir os placeholders da Fase 2 por uma interface de verdade.

- [x] Layout com as três abas implementado (HTML/CSS)
- [x] Evento `nova-traducao` emitido pelo Rust e escutado pelo JS
- [x] Aba "Tradução" mostra texto original e traduzido lado a lado
- [x] Janela é trazida para frente automaticamente após uma tradução
- [x] Abas "Histórico" e "Configurações" existem na tela (ainda vazias/sem função — só o esqueleto visual)

**Critério de pronto:** ✅ Validado em 2026-08-15 — testado com um texto longo/multi-linha real (trecho de um livro técnico), capturado e traduzido corretamente, aparecendo formatado na aba Tradução, com a janela vindo para frente sozinha.

**Observações:**
- O `greet` (command de demonstração do template Tauri) e os assets `tauri.svg`/`javascript.svg` foram removidos — não tinham mais uso depois que a interface real substituiu a página de exemplo do scaffold.
- Sem testes unitários de JS nesta fase: a lógica adicionada em `main.js` é só alternância declarativa de abas e escrita de texto no DOM (sem parsing, formatação ou regra de negócio não-trivial), o que se encaixa na exceção do `CLAUDE.md` para UI puramente declarativa.

**Critério de pronto:** repetir o teste da Fase 2, mas agora o resultado aparece formatado na aba Tradução da janela do app, não em `console.log`.

---

## Fase 4 — Histórico persistente
*(GUIA.md §10)*

Objetivo: toda tradução feita fica salva e consultável, mesmo depois de fechar e abrir o app de novo.

- [x] Plugin `tauri-plugin-sql` instalado e configurado
- [x] Migração cria a tabela `historico` na primeira execução
- [x] Cada tradução bem-sucedida é gravada no banco
- [x] Aba "Histórico" lista as traduções salvas (mais recente primeiro)
- [x] Fechar e reabrir o app preserva o histórico

**Critério de pronto:** ✅ Validado em 2026-08-15 — 3 traduções feitas, app fechado (janela fechada = processo encerrado, já que a Fase 7 ainda não implementou "fechar = esconder"), reaberto, e as 3 traduções continuavam na aba Histórico.

**Observações:**
- **`execute`/`select` do `tauri-plugin-sql` não conectam ao banco sob demanda** — é preciso chamar `plugin:sql|load` explicitamente antes (mesmo usando o "caminho preguiçoso" documentado como `Database.get()` no pacote oficial). Sem isso, o erro é `database sqlite:historico.db not loaded`. Implementado em `src/historico.js` como uma função `garantirBancoCarregado()` que faz `load` uma vez (guardando a Promise, não só um booleano, para não disparar `load` duas vezes em chamadas concorrentes) antes de qualquer `execute`/`select`.
- **`sql:default` não inclui a permissão de escrita** — precisa declarar `sql:allow-load`, `sql:allow-execute` e `sql:allow-select` explicitamente em `capabilities/default.json`, senão o erro (`sql.execute not allowed`) só aparece no console do DevTools do webview, nunca no terminal onde roda o processo Rust — reforça o alerta do `GUIA.md §13` sobre falhas silenciosas de permissão.
- Como não existe bundler (projeto vanilla, sem Vite — decisão da Fase 1), não dava para `import Database from '@tauri-apps/plugin-sql'` direto (é um pacote npm, sem um jeito de resolver o import no navegador sem um bundler). Em vez de adicionar um bundler só por causa disso, `src/historico.js` chama `window.__TAURI__.core.invoke("plugin:sql|execute"/"plugin:sql|select"/"plugin:sql|load", ...)` diretamente — os mesmos commands que o pacote oficial usa por baixo dos panos. O pacote `@tauri-apps/plugin-sql` foi instalado e depois desinstalado do `package.json` por não ser necessário.
- Corrigido um risco de XSS local: o texto original/traduzido vem do clipboard do usuário (pode ter sido copiado de uma página web não confiável). `renderizarHistorico` em `main.js` monta os elementos via `textContent`/DOM em vez de interpolar strings em `innerHTML`, evitando que HTML/JS embutido no texto copiado seja executado dentro do app.
- Testes JS: primeira vez que o projeto tem lógica de frontend não-trivial (`formatarDataHistorico`), então foi adicionado `node --test` (test runner nativo do Node, sem dependência nova) como `npm test`, cobrindo data válida, inválida e string vazia.

---

## Fase 5 — Configurações (fecha o MVP)
*(GUIA.md §11)*

Objetivo: remover os valores fixos da Fase 2 (atalho fixo, credenciais por variável de ambiente) e tornar tudo configurável pelo usuário — **este é o fim do MVP**.

- [x] Plugin `tauri-plugin-store` instalado
- [x] Tela de configurações funcional: campo de atalho, idioma de destino, **seletor de provedor de tradução** (DeepL / Azure Translator) com os campos de credencial correspondentes (API key para DeepL; API key + região para Azure)
- [x] Salvar configurações persiste entre reinícios do app
- [x] Trocar o atalho na tela realmente re-registra o atalho global (chamando o command `registrar_atalho`)
- [x] Erro de atalho em conflito é exibido de forma amigável (não trava o app) — implementado (try/catch no `main.js` + `Result` propagado do Rust sem panic) mas **não testado empiricamente** com um conflito real, por ser difícil de reproduzir de propósito; revisar na Fase 10.
- [x] Credenciais deixam de vir de variável de ambiente — passam a vir do `store` configurado pela tela

**Critério de pronto:** ✅ Validado em 2026-08-15 — configurado tudo pela tela (atalho, idioma, provedor DeepL, chave), salvo, `Ctrl+Alt+T` funcionando; app fechado e reaberto **sem nenhuma variável de ambiente**, atalho e configurações continuaram aplicados, tradução funcionando normalmente.

> ✅ **MVP completo.** Fases 0 a 5 concluídas — o app já é usável no dia a dia via `npm run tauri dev`.

**Observações:**
- Mesma pegadinha do `tauri-plugin-sql` na Fase 4, agora com o `tauri-plugin-store`: `set`/`get`/`save` exigem um `rid` de uma store já carregada via `plugin:store|load` (que retorna um resource id, não só confirma um caminho como o `sql`). `src/config.js` guarda essa Promise de `load` em `ridCarregado` e reusa, no mesmo padrão do `historico.js`.
- `app.store(path)` do lado Rust (via `tauri_plugin_store::StoreExt`) é **síncrono** e compartilha o mesmo estado em memória que o frontend usa via `plugin:store|*` — não precisou de nenhuma ponte extra para o backend ler credenciais/atalho/idioma salvos pela tela.
- Erro de compilação real (não o falso-positivo do PowerShell): `tauri_plugin_global_shortcut::Result<T>` é um type alias **privado** do crate — precisa usar `Result<T, tauri_plugin_global_shortcut::Error>` explicitamente na assinatura de funções próprias.
- O idioma de destino agora é uma lista canônica pequena (`pt-br`, `pt-pt`, `en`, `es`, `fr`, `de`, `it`) mapeada para o código específico de cada provedor em `traducao/mod.rs`; adicionar um idioma novo é só uma linha ali e uma linha equivalente em `src/config.js`.
- Os pacotes npm `@tauri-apps/plugin-store` (e antes `@tauri-apps/plugin-sql`) foram instalados só temporariamente para inspecionar o `dist-js` e confirmar os nomes exatos dos commands/parâmetros, depois desinstalados — o projeto não depende de nenhum dos dois em tempo de execução, só chama `invoke` diretamente (decisão da Fase 4, mantida por não termos bundler).

> ✅ **Ao final da Fase 5, você tem um MVP completo e usável no dia a dia**, mesmo rodando via `npm run tauri dev`. As fases seguintes são refinamentos de experiência (segundo plano, modo automático) e distribuição (instalador).

---

## Fase 6 — Modo automático (clipboard) e alternância
*(GUIA.md §7.3)*

Objetivo: adicionar o segundo modo de captura, com toggle nas configurações.

- [x] Loop de monitoramento do clipboard implementado (thread separada)
- [x] Flag "modo automático" lida da configuração a cada ciclo do loop
- [x] Checkbox na tela de Configurações liga/desliga o modo automático
- [x] Com o modo automático ligado, copiar qualquer texto (`Ctrl+C` normal) dispara tradução
- [x] Com o modo automático desligado, só o atalho manual dispara tradução

**Critério de pronto:** ✅ Validado em 2026-08-15 — checkbox ligado, `Ctrl+C` normal traduziu sozinho; checkbox desligado, parou de traduzir automaticamente, sem precisar reiniciar o app.

**Observações:**
- `capturar_e_traduzir` (atalho) e o novo loop de monitoramento (`iniciar_monitoramento_automatico`) foram unificados num único `traduzir_e_notificar(app, texto)` compartilhado — evita duplicar a lógica de "chamar o provedor configurado, emitir o evento, trazer a janela pra frente" entre os dois modos de captura.
- O checkbox de modo automático salva sozinho no `change` (sem precisar clicar em "Salvar", diferente dos outros campos da tela) — o backend confere o valor a cada ~800ms, então o efeito é quase imediato.
- Cuidado que valeu a pena documentar: a thread de monitoramento inicializa `ultimo_valor` com o conteúdo **atual** do clipboard ao iniciar (não com string vazia). Sem isso, a primeira checagem depois de ligar o app compararia contra `""` e traduziria de sopetão qualquer coisa que já estivesse copiada antes — um falso positivo bem confuso de depurar.

---

## Fase 7 — Bandeja do sistema e execução em segundo plano
*(GUIA.md §5, §12)*

Objetivo: o app se comporta como um utilitário de fundo de verdade, não como uma janela comum.

- [x] Ícone na bandeja com menu "Abrir" / "Sair"
- [x] Fechar a janela (X) esconde em vez de encerrar o processo
- [x] `tauri-plugin-single-instance` impede abrir o app duas vezes
- [x] `tauri-plugin-autostart` implementado, com toggle nas configurações para iniciar com o Windows

**Critério de pronto:** ✅ Validado em 2026-08-15 — ícone na bandeja com menu Abrir/Sair funcionando; fechar a janela mantém o app rodando (confirmado com o usuário); tentativa de abrir uma segunda instância (`Start-Process` no mesmo `.exe`) não criou um segundo processo — só o original continuou rodando; checkbox de autostart testado na tela de Configurações.

**Observações:**
- `tauri-plugin-single-instance` **precisa ser o primeiro `.plugin(...)` registrado** no builder — é uma exigência do próprio plugin, não uma preferência de estilo.
- A feature `tray-icon` do crate `tauri` **não vem habilitada por padrão** — precisou adicionar `features = ["tray-icon"]` no `Cargo.toml` (erro de compilação real: `unresolved import 'tauri::tray'`, com a mensagem apontando exatamente qual feature faltava).
- O estado do autostart não é guardado em `config.json` — o `tauri-plugin-autostart` mexe direto no registro do Windows (`HKCU\...\Run`), então essa é a fonte de verdade; o frontend consulta o estado atual via o command `autostart_esta_ativo` em vez de duplicar essa informação na store.
- Conferido manualmente que o teste do autostart não deixou nenhuma entrada residual em `HKCU:\Software\Microsoft\Windows\CurrentVersion\Run` apontando para o build de desenvolvimento.

---

## Fase 8 — Auditoria de permissions/capabilities
*(GUIA.md §13)*

Objetivo: revisão dedicada de segurança/configuração antes de empacotar — não é uma feature nova, é uma checagem.

- [x] `capabilities/default.json` revisado contra a lista de plugins realmente usados
- [x] Cada funcionalidade (atalho, clipboard, sql, store, autostart) testada isoladamente para confirmar que não há falha silenciosa de permissão
- [x] Nenhuma API key ou segredo commitado no repositório

**Critério de pronto:** ✅ Validado em 2026-08-15 — depois de reduzir as permissões, testado atalho global, modo automático, histórico e salvar configurações; tudo continuou funcionando. Buscado em todo o histórico do git (`git log -p --all`) por padrões de chave de API — nada encontrado (só um placeholder de exemplo no GUIA.md).

**Observações:**
- Achado real da auditoria: `clipboard-manager:default` e `global-shortcut:allow-register`/`allow-unregister` estavam nas capabilities desde as Fases 2/4, mas **nunca foram necessários** — todo uso de clipboard/atalho global acontece só do lado Rust (via `ClipboardExt`/`GlobalShortcutExt`, chamado diretamente em `captura.rs`/`lib.rs`), nunca via `invoke("plugin:clipboard-manager|...")`/`invoke("plugin:global-shortcut|...")` do frontend. O sistema de permissions do Tauri só protege a ponte de IPC frontend→backend para commands de **plugin**; código Rust que usa a extension trait diretamente não passa por ali. Removidas as duas entradas.
- `tauri-plugin-opener` (do template da Fase 1) nunca foi usado em lugar nenhum — nem no Rust, nem no frontend (não há `<a target="_blank">` na UI atual). Removido por completo: dependência do `Cargo.toml`, registro do plugin em `lib.rs`, e a permissão `opener:default` das capabilities.
- Capabilities finais: `core:default` + só `sql:*` e `store:*` (as únicas que o frontend de fato invoca via `plugin:sql|...`/`plugin:store|...`).

---

## Fase 9 — Empacotamento e instalador
*(GUIA.md §14)*

Objetivo: gerar o artefato final distribuível.

- [x] Ícones do app gerados em todos os tamanhos — mantidos os do scaffold da Fase 1 (logo padrão do Tauri; usuário optou por não trocar por enquanto, dá pra rodar `npm run tauri icon` com uma imagem própria depois)
- [x] `bundle.targets` configurado para `nsis` e `msi` (era `"all"`; trocado para ser explícito e não surpreender na Fase 11 quando entrar Linux)
- [x] `npm run tauri build` gera o instalador sem erros — 13min53s, gerou `select-translate_0.1.0_x64-setup.exe` (~4,6 MB) e `select-translate_0.1.0_x64_en-US.msi` (~6,6 MB)
- [x] Instalador testado — **ressalva:** testado na própria máquina de desenvolvimento (não havia VM limpa disponível), instalando de verdade via `/S` (silencioso) e rodando o `.exe` instalado isoladamente (não via `cargo`/`npm`). Não é o mesmo que testar numa máquina 100% sem Rust/Node, mas confirma que o binário standalone funciona sem depender do ambiente de dev ativo.
- [x] App instalado aparece corretamente no menu Iniciar e desinstala sem deixar resíduos

**Critério de pronto:** ✅ Validado em 2026-08-15 (com a ressalva acima sobre a máquina de teste) — instalado via `select-translate_0.1.0_x64-setup.exe`, atalho apareceu no Menu Iniciar (`select-translate.lnk`), app abriu, Ctrl+Alt+T traduziu normalmente (confirmado pelo usuário), desinstalado depois e conferido que pasta de instalação, atalho e entrada de registro sumiram completamente.

**Observações:**
- Instala em `%LOCALAPPDATA%\select-translate\` (modo `installMode: currentUser`, sem precisar de admin).
- Os dados do usuário (`config.json`, `historico.db`, em `%APPDATA%\com.joaov.select-translate\`) **não são apagados** ao desinstalar — comportamento padrão/esperado (evita perda de dados se o usuário reinstalar depois), não é um resíduo real.
- Como a pasta de dados é baseada no `identifier` do app (não no perfil de build), o app instalado (release) enxergou automaticamente a chave DeepL e as configurações já salvas durante os testes em modo `dev` — não precisou reconfigurar nada.

---

## Fase 10 — Validação final de ponta a ponta
*(GUIA.md §15)*

Objetivo: passar pelo checklist completo dos 7 requisitos originais antes de considerar "pronto para uso real".

- [x] Checklist completo da seção 15 do GUIA.md executado — 10 de 12 itens aprovados, 2 não testados por falta de app disponível (ver observações)
- [x] Teste em pelo menos: navegador, Bloco de Notas, ~~Word~~, um PDF, um editor de código, ~~um app de mensagens~~ — Word e app de mensagens pulados (usuário não tem nenhum instalado); todos os outros passaram

**Critério de pronto:** ✅ Validado em 2026-08-15, com ressalva — todos os itens testáveis no ambiente atual passaram. Word e app de mensagens (WhatsApp/Telegram) ficam pendentes por falta de app disponível para testar, não por falha do app.

**Observações:**
- Testes novos feitos nesta fase (os demais já tinham sido validados em fases anteriores e foram reaproveitados como evidência): **PDF** (Edge, texto selecionado e traduzido), **VS Code** (comentário/código selecionado e traduzido), e **trocar o atalho global** (mudado de `CommandOrControl+Alt+T` para `CommandOrControl+Alt+Y` pela tela de Configurações — confirmado no log `Atalho global registrado: CommandOrControl+Alt+Y` seguido de uma tradução funcionando com o atalho novo).
- Usuário optou por manter o atalho em `CommandOrControl+Alt+Y` depois do teste, em vez de voltar para o padrão.
- **Pendência real, não urgente:** testar em Word e num app de mensagens quando/se o usuário tiver algum instalado. Como a captura funciona via clipboard (Ctrl+C simulado) e já foi validada em navegador, Bloco de Notas, PDF e VS Code — uma variedade grande de tipos de app (nativo Win32, webview, editor de texto rico, editor de código) — o risco de Word/apps de mensagens se comportarem diferente é baixo, mas não é zero (ex: Word às vezes copia RTF/HTML além de texto puro; alguns apps Electron de mensagens têm comportamento de clipboard peculiar).

---

## Fase 11 — (Futuro) Portabilidade para Linux
*(GUIA.md §16)*

Fora do escopo imediato — só iniciar depois que o app estiver estável e em uso real no Windows.

- [ ] Ambiente de build Linux configurado
- [ ] `enigo` validado em X11 e, separadamente, em Wayland
- [ ] Bundler configurado para `deb`/`appimage`
- [ ] Ajustes de atalho global e bandeja documentados por ambiente gráfico

---

## Fase 12 — (Futuro) Melhorias
*(GUIA.md §17)*

Backlog de ideias, sem compromisso de prazo: mais provedores de tradução (Google Translate, Microsoft Translator já resolvido na Fase 2/5 como Azure Translator, LibreTranslate self-hosted etc.), fallback automático entre provedores se um estiver fora do ar, detecção automática de idioma, atalhos por idioma de destino, exportação de histórico, tema claro/escuro, leitura em voz alta.

---

## Resumo visual

```
Fase 0  Ambiente preparado
Fase 1  Scaffold "Hello World"
Fase 2  Núcleo: captura + tradução (maior risco técnico)
Fase 3  Interface (abas)
Fase 4  Histórico persistente
Fase 5  Configurações                    ← fim do MVP
Fase 6  Modo automático (clipboard)
Fase 7  Bandeja + segundo plano
Fase 8  Auditoria de permissions
Fase 9  Empacotamento/instalador
Fase 10 Validação final
Fase 11 (futuro) Linux
Fase 12 (futuro) Melhorias
```
