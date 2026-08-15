const { invoke } = window.__TAURI__.core;

let greetInputEl;
let greetMsgEl;

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-form").addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });

  // Temporário da Fase 2 — a Fase 3 substitui isso por uma aba de
  // Tradução de verdade. Por enquanto só mostra o resultado num alert.
  window.__TAURI__.event.listen("nova-traducao", (evento) => {
    const { original, traduzido } = evento.payload;
    alert(`Original:\n${original}\n\nTradução:\n${traduzido}`);
  });
});
