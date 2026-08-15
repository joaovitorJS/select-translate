# CLAUDE.md

Instruções para o Claude Code ao trabalhar neste repositório.

## Sobre o projeto

**Select Translate** é um aplicativo desktop nativo para Windows (com portabilidade futura para Linux) que traduz texto selecionado em qualquer programa. O usuário seleciona um texto em qualquer app (navegador, Bloco de Notas, Word, PDF, editor de código, app de mensagens), captura via atalho global configurável ou modo automático (monitoramento de clipboard), e recebe a tradução (API DeepL) numa janela própria, com histórico persistente. O app roda em segundo plano, minimizado na bandeja do sistema, e é distribuído como instalador nativo (`.exe`/`.msi`) — nunca como site ou algo dependente de navegador.

## Documentos de referência

- **[`GUIA.md`](./GUIA.md)** — guia técnico completo, passo a passo, com a arquitetura, snippets de código e explicação de cada peça (Tauri v2, Rust, plugins). Consulte antes de implementar qualquer fase.
- **[`FASES.md`](./FASES.md)** — roteiro de execução dividido em fases sequenciais, cada uma com checklist e critério de pronto. É a fonte de verdade de **o que falta fazer** e **o que já foi feito**.

Sempre que for implementar algo, localize a fase correspondente em `FASES.md` e a seção correspondente em `GUIA.md` antes de escrever código.

## Stack técnica (resumo)

- **Backend**: Rust (`src-tauri/`) — Tauri v2, `enigo` (simulação de teclado), `reqwest` (HTTP/DeepL), plugins oficiais Tauri (`global-shortcut`, `clipboard-manager`, `sql`, `store`, `autostart`, `single-instance`).
- **Frontend**: HTML/CSS/JS vanilla (`src/`) — sem framework, por design (ver GUIA.md §1).
- **Banco de dados**: SQLite local via `tauri-plugin-sql`.
- **Tradução**: API DeepL.

## Fluxo de trabalho por fase (IMPORTANTE)

Este projeto é desenvolvido fase por fase, seguindo a ordem de `FASES.md`. **Ao concluir o trabalho de uma fase, siga sempre esta sequência:**

1. **Criar uma branch para a fase** antes de começar a implementar (não trabalhar direto na branch principal). Convenção de nome: `fase-N-descricao-curta` (ex: `fase-2-nucleo-captura-traducao`, `fase-5-configuracoes`). A branch deve conter, ao final, todas as alterações de código daquela fase já commitadas.
2. **Implementar as tarefas da fase** conforme descrito em `FASES.md` e detalhado em `GUIA.md`.
3. **Escrever testes unitários para as funcionalidades novas daquela fase**, não deixar para depois:
   - Código Rust (`src-tauri/`): testes com `#[cfg(test)]` / `#[test]`, rodáveis via `cargo test`. Priorize testar lógica pura (parsing, formatação, regras de negócio) e isole efeitos colaterais (chamadas HTTP, I/O do sistema, clipboard) atrás de funções pequenas e testáveis — não é necessário (nem sempre possível) testar automação de teclado/mouse ou chamada real à API do DeepL; nesses casos, use mocks/fakes ou teste a lógica ao redor (ex: montagem do payload, tratamento de erro da resposta).
   - Código JS (`src/`): se a fase adicionar lógica não-trivial no frontend (parsing, formatação, regras de exibição), adicione testes correspondentes; UI puramente declarativa não precisa de teste unitário dedicado.
   - Toda fase que introduzir uma funcionalidade nova só é considerada concluída se vier acompanhada de teste(s) cobrindo o caminho principal e pelo menos um caso de erro relevante.
4. **Rodar os testes** (`cargo test` dentro de `src-tauri/`, e o runner de testes JS se houver) e confirmar que passam antes de prosseguir.
5. **Dar baixa em `FASES.md`**: marcar (`[x]`) cada item da checklist da fase que foi concluído. Se algum item não pôde ser concluído, deixá-lo desmarcado e anotar o motivo como observação na própria fase (não marcar como feito por otimismo).
6. **Commitar** as mudanças de código, os testes e a atualização de `FASES.md` juntos (ou em commits lógicos separados dentro da mesma branch) — a branch da fase deve refletir o estado completo daquela fase quando o trabalho terminar.

Nunca avance para a fase seguinte sem que a fase atual esteja com os itens marcados em `FASES.md`, os testes passando, e as mudanças commitadas na branch da fase.

## Notas adicionais

- Segredos (API key do DeepL etc.) nunca devem ser commitados hardcoded no código-fonte além do uso temporário descrito explicitamente na Fase 2 do `FASES.md` — a partir da Fase 5, a chave passa a vir exclusivamente da tela de Configurações.
- Ao adicionar um novo plugin Tauri, sempre atualizar `src-tauri/capabilities/default.json` na mesma alteração (ver GUIA.md §13) — plugin sem permissão declarada falha silenciosamente.
