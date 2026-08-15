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

- [ ] Loop de monitoramento do clipboard implementado (thread separada)
- [ ] Flag "modo automático" lida da configuração a cada ciclo do loop
- [ ] Checkbox na tela de Configurações liga/desliga o modo automático
- [ ] Com o modo automático ligado, copiar qualquer texto (`Ctrl+C` normal) dispara tradução
- [ ] Com o modo automático desligado, só o atalho manual dispara tradução

**Critério de pronto:** alternar o checkbox e confirmar visualmente que o comportamento muda imediatamente, sem precisar reiniciar o app.

---

## Fase 7 — Bandeja do sistema e execução em segundo plano
*(GUIA.md §5, §12)*

Objetivo: o app se comporta como um utilitário de fundo de verdade, não como uma janela comum.

- [ ] Ícone na bandeja com menu "Abrir" / "Sair"
- [ ] Fechar a janela (X) esconde em vez de encerrar o processo
- [ ] `tauri-plugin-single-instance` impede abrir o app duas vezes
- [ ] `tauri-plugin-autostart` implementado, com toggle nas configurações para iniciar com o Windows

**Critério de pronto:** fechar a janela, confirmar que o ícone continua na bandeja e que o atalho global continua funcionando; tentar abrir o app de novo pelo atalho de desktop e confirmar que só a janela existente é trazida para frente (não abre um segundo processo).

---

## Fase 8 — Auditoria de permissions/capabilities
*(GUIA.md §13)*

Objetivo: revisão dedicada de segurança/configuração antes de empacotar — não é uma feature nova, é uma checagem.

- [ ] `capabilities/default.json` revisado contra a lista de plugins realmente usados
- [ ] Cada funcionalidade (atalho, clipboard, sql, store, autostart) testada isoladamente para confirmar que não há falha silenciosa de permissão
- [ ] Nenhuma API key ou segredo commitado no repositório

**Critério de pronto:** rodar o app do zero (perfil de usuário limpo, se possível) e confirmar que nenhuma funcionalidade falha silenciosamente por falta de permissão.

---

## Fase 9 — Empacotamento e instalador
*(GUIA.md §14)*

Objetivo: gerar o artefato final distribuível.

- [ ] Ícones do app gerados em todos os tamanhos (`npm run tauri icon`)
- [ ] `bundle.targets` configurado para `nsis` e/ou `msi`
- [ ] `npm run tauri build` gera o instalador sem erros
- [ ] Instalador testado numa máquina/VM sem as ferramentas de desenvolvimento instaladas
- [ ] App instalado aparece corretamente no menu Iniciar e desinstala sem deixar resíduos

**Critério de pronto:** instalar o app a partir do `.exe`/`.msi` gerado numa máquina limpa e usar todas as funcionalidades sem ter Rust/Node instalado nela.

---

## Fase 10 — Validação final de ponta a ponta
*(GUIA.md §15)*

Objetivo: passar pelo checklist completo dos 7 requisitos originais antes de considerar "pronto para uso real".

- [ ] Checklist completo da seção 15 do GUIA.md executado e aprovado
- [ ] Teste em pelo menos: navegador, Bloco de Notas, Word, um PDF, um editor de código, um app de mensagens

**Critério de pronto:** todos os itens do checklist marcados como OK.

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
