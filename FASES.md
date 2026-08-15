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

- [ ] Atalho global fixo registrado (ex: `Ctrl+Alt+T`, ainda não configurável)
- [ ] Simulação de `Ctrl+C` com `enigo` funcionando
- [ ] Leitura do clipboard após a simulação funcionando
- [ ] Conta DeepL criada e API key obtida
- [ ] Chamada à API do DeepL funcionando via `reqwest`, com a chave hardcoded temporariamente (nunca vai para o instalador final assim — é só para validar o fluxo)
- [ ] Resultado aparece em qualquer lugar visível (terminal, `alert()`, ou texto simples na janela)

**Critério de pronto:** selecionar um texto no navegador, no Bloco de Notas e no Word, apertar o atalho, e ver a tradução em português correta nos três casos. Se falhar em algum app específico, anote — vira item de troubleshooting mais adiante.

---

## Fase 3 — Interface real (abas Tradução / Histórico / Configurações)
*(GUIA.md §9)*

Objetivo: substituir os placeholders da Fase 2 por uma interface de verdade.

- [ ] Layout com as três abas implementado (HTML/CSS)
- [ ] Evento `nova-traducao` emitido pelo Rust e escutado pelo JS
- [ ] Aba "Tradução" mostra texto original e traduzido lado a lado
- [ ] Janela é trazida para frente automaticamente após uma tradução
- [ ] Abas "Histórico" e "Configurações" existem na tela (ainda vazias/sem função — só o esqueleto visual)

**Critério de pronto:** repetir o teste da Fase 2, mas agora o resultado aparece formatado na aba Tradução da janela do app, não em `console.log`.

---

## Fase 4 — Histórico persistente
*(GUIA.md §10)*

Objetivo: toda tradução feita fica salva e consultável, mesmo depois de fechar e abrir o app de novo.

- [ ] Plugin `tauri-plugin-sql` instalado e configurado
- [ ] Migração cria a tabela `historico` na primeira execução
- [ ] Cada tradução bem-sucedida é gravada no banco
- [ ] Aba "Histórico" lista as traduções salvas (mais recente primeiro)
- [ ] Fechar e reabrir o app preserva o histórico

**Critério de pronto:** fazer 3 traduções, fechar o app completamente, reabrir, e ver as 3 no histórico.

---

## Fase 5 — Configurações (fecha o MVP)
*(GUIA.md §11)*

Objetivo: remover os valores fixos da Fase 2 (atalho hardcoded, API key hardcoded) e tornar tudo configurável pelo usuário — **este é o fim do MVP**.

- [ ] Plugin `tauri-plugin-store` instalado
- [ ] Tela de configurações funcional: campo de atalho, idioma de destino, campo de API key
- [ ] Salvar configurações persiste entre reinícios do app
- [ ] Trocar o atalho na tela realmente re-registra o atalho global (chamando o command `registrar_atalho`)
- [ ] Erro de atalho em conflito é exibido de forma amigável (não trava o app)
- [ ] API key deixa de estar hardcoded no código-fonte

**Critério de pronto:** apagar toda referência hardcoded de atalho/API key do código; configurar tudo pela tela; reiniciar o app; confirmar que as configurações continuam aplicadas.

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

Backlog de ideias, sem compromisso de prazo: múltiplos serviços de tradução, detecção automática de idioma, atalhos por idioma de destino, exportação de histórico, tema claro/escuro, leitura em voz alta.

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
