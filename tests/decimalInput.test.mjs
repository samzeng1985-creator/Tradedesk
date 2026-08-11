import assert from "node:assert/strict";
import test from "node:test";
import {
  MAX_MONEY_INPUT,
  moneyInputToMinor,
  normalizeMoneyInput,
} from "../src/decimalInput.ts";

test("money input keeps uninterrupted integer and decimal editing", () => {
  assert.equal(normalizeMoneyInput("9"), "9");
  assert.equal(normalizeMoneyInput("99"), "99");
  assert.equal(normalizeMoneyInput("999999.99"), "999999.99");
  assert.equal(normalizeMoneyInput("9."), "9.");
  assert.equal(normalizeMoneyInput("12,50"), "12.50");
});

test("money input rejects invalid precision and converts large prices to minor units", () => {
  assert.equal(normalizeMoneyInput("1.234"), null);
  assert.equal(normalizeMoneyInput("abc"), null);
  assert.equal(moneyInputToMinor("123456789.12"), 12_345_678_912);
  assert.equal(moneyInputToMinor("0"), null);
  assert.equal(moneyInputToMinor(String(MAX_MONEY_INPUT + 1)), null);
});
