import { test } from "node:test";
import assert from "node:assert/strict";
import { formatarDataHistorico } from "./historico.js";

test("formata uma data ISO válida no padrão dd/mm/aaaa", () => {
  const resultado = formatarDataHistorico("2026-08-15T19:05:00.000Z");
  assert.match(resultado, /\d{2}\/\d{2}\/\d{4}/);
});

test("retorna o valor original quando a data não pode ser interpretada", () => {
  assert.equal(formatarDataHistorico("não é uma data"), "não é uma data");
});

test("retorna o valor original para string vazia", () => {
  assert.equal(formatarDataHistorico(""), "");
});
