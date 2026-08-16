import { test } from "node:test";
import assert from "node:assert/strict";
import { resolverTema, validarConfigFormulario } from "./config.js";

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

test("resolverTema respeita preferência explícita, ignorando o sistema", () => {
  assert.equal(resolverTema("claro", true), "claro");
  assert.equal(resolverTema("escuro", false), "escuro");
});

test("resolverTema segue o sistema quando a preferência é automática", () => {
  assert.equal(resolverTema("automatico", true), "escuro");
  assert.equal(resolverTema("automatico", false), "claro");
});
