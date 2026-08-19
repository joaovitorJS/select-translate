import { test } from "node:test";
import assert from "node:assert/strict";
import {
  montarAtalhoDoEvento,
  resolverTema,
  truncarTexto,
  validarConfigFormulario,
} from "./config.js";

const base = {
  atalho: "CommandOrControl+Alt+T",
  provedor: "deepl",
  deeplKey: "chave",
  azureKey: "",
  azureRegiao: "",
};

test("aceita configuração válida do DeepL", () => {
  assert.equal(validarConfigFormulario(base), null);
});

test("rejeita quando o atalho está vazio", () => {
  assert.match(validarConfigFormulario({ ...base, atalho: "" }), /atalho/i);
});

test("aceita atalho do popover vazio quando o popover está desativado", () => {
  assert.equal(validarConfigFormulario({ ...base, popoverAtivo: false, atalhoPopover: "" }), null);
});

test("rejeita atalho do popover vazio quando o popover está ativado", () => {
  assert.match(
    validarConfigFormulario({ ...base, popoverAtivo: true, atalhoPopover: "" }),
    /popover/i,
  );
});

test("aceita atalho do popover preenchido quando o popover está ativado", () => {
  assert.equal(
    validarConfigFormulario({
      ...base,
      popoverAtivo: true,
      atalhoPopover: "CommandOrControl+Alt+P",
    }),
    null,
  );
});

test("rejeita DeepL sem chave de API", () => {
  assert.match(validarConfigFormulario({ ...base, deeplKey: "" }), /DeepL/);
});

test("rejeita Azure sem chave ou região", () => {
  const semChave = { ...base, provedor: "azure", deeplKey: "", azureRegiao: "brazilsouth" };
  assert.match(validarConfigFormulario(semChave), /Azure/);

  const semRegiao = { ...base, provedor: "azure", deeplKey: "", azureKey: "chave" };
  assert.match(validarConfigFormulario(semRegiao), /Azure/);
});

test("aceita configuração válida do Azure", () => {
  const valido = {
    ...base,
    provedor: "azure",
    deeplKey: "",
    azureKey: "chave",
    azureRegiao: "brazilsouth",
  };
  assert.equal(validarConfigFormulario(valido), null);
});

test("truncarTexto mantém o texto igual quando já cabe no limite", () => {
  assert.equal(truncarTexto("abc", 5), "abc");
  assert.equal(truncarTexto("abcde", 5), "abcde");
});

test("truncarTexto corta e acrescenta reticências quando passa do limite", () => {
  assert.equal(truncarTexto("abcdef", 5), "abcde…");
});

test("resolverTema respeita preferência explícita, ignorando o sistema", () => {
  assert.equal(resolverTema("claro", true), "claro");
  assert.equal(resolverTema("escuro", false), "escuro");
});

test("resolverTema segue o sistema quando a preferência é automática", () => {
  assert.equal(resolverTema("automatico", true), "escuro");
  assert.equal(resolverTema("automatico", false), "claro");
});

const semModificador = { ctrlKey: false, altKey: false, shiftKey: false, metaKey: false };

test("montarAtalhoDoEvento monta Ctrl+Alt+T a partir do evento", () => {
  const evento = { ...semModificador, ctrlKey: true, altKey: true, key: "t", code: "KeyT" };
  assert.equal(montarAtalhoDoEvento(evento), "CommandOrControl+Alt+T");
});

test("montarAtalhoDoEvento usa 'code' pra ignorar layout de teclado em dígitos", () => {
  const evento = { ...semModificador, ctrlKey: true, key: "1", code: "Digit1" };
  assert.equal(montarAtalhoDoEvento(evento), "CommandOrControl+1");
});

test("montarAtalhoDoEvento reconhece teclas de função sem precisar de modificador", () => {
  const evento = { ...semModificador, key: "F9", code: "F9" };
  assert.equal(montarAtalhoDoEvento(evento), "F9");
});

test("montarAtalhoDoEvento retorna null enquanto só um modificador está pressionado", () => {
  const evento = { ...semModificador, ctrlKey: true, key: "Control", code: "ControlLeft" };
  assert.equal(montarAtalhoDoEvento(evento), null);
});

test("montarAtalhoDoEvento retorna null para Escape (cancela a gravação)", () => {
  const evento = { ...semModificador, key: "Escape", code: "Escape" };
  assert.equal(montarAtalhoDoEvento(evento), null);
});

test("montarAtalhoDoEvento junta vários modificadores na ordem esperada", () => {
  const evento = {
    ctrlKey: true,
    altKey: true,
    shiftKey: true,
    metaKey: false,
    key: "y",
    code: "KeyY",
  };
  assert.equal(montarAtalhoDoEvento(evento), "CommandOrControl+Alt+Shift+Y");
});
