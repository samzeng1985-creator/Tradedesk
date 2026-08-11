import assert from "node:assert/strict";
import test from "node:test";
import { performance } from "node:perf_hooks";
import {
  filterBusinessCases,
  filterConfigurations,
  filterCustomers,
  filterDocuments,
  filterProducts,
  filterPurchaseOrders,
} from "../src/uiQueries.ts";

const count = 10_000;
const products = Array.from({ length: count }, (_, index) => ({
  id: `p-${index}`, sku: `SKU-${index}`, nameZh: `产品 ${index}`, nameEn: `Generator ${index}`,
  model: `MODEL-${index}`, hsCode: `8502${index}`, unit: "Set", grossWeightKg: 1, active: true,
}));
const customers = Array.from({ length: count }, (_, index) => ({
  id: `c-${index}`, code: `CUS-${index}`, legalName: `Customer ${index}`, market: "EU", currency: "USD",
  paymentTerms: "T/T", address: `Address ${index}`, shippingAddress: `Warehouse ${index}`,
  billingAddress: `Billing ${index}`, purchaseIntent: `Intent ${index}`, customerAnalysis: "",
  strengths: "", weaknesses: "", contacts: `Contact ${index}`, active: true,
}));
const configurations = Array.from({ length: count }, (_, index) => ({
  id: `cfg-${index}`, code: `CFG-${index}`, name: `Configured generator ${index}`, model: `M-${index}`,
  currency: "USD", exchangeRate: 0.14, exchangeRateDate: "2026-08-11", notes: `Project ${index}`,
  totalAmountMinor: index, active: true, lines: [{ category: "Engine", name: `Engine ${index}`,
    specification: `Spec ${index}`, brand: `Brand ${index}` }],
}));
const cases = Array.from({ length: count }, (_, index) => ({
  id: `case-${index}`, number: `TD-20260811-${String(index).padStart(4, "0")}`,
  customerName: `Customer ${index}`, currency: "USD", incoterm: index % 2 ? "FOB" : "CIF", lines: [],
}));
const orders = Array.from({ length: count }, (_, index) => ({
  id: `po-${index}`, number: `PO-20260811-${String(index).padStart(4, "0")}`,
  businessCaseNumber: cases[index].number, supplierName: `Supplier ${index}`,
}));
const documents = Array.from({ length: count }, (_, index) => ({
  id: `doc-${index}`, number: `INV-20260811-${String(index).padStart(4, "0")}`,
  businessCaseNumber: cases[index].number, customerName: `Customer ${index}`,
  documentType: index % 2 ? "commercial_invoice" : "packing_list",
  status: index % 3 ? "issued" : "draft",
}));

test("10,000-record UI searches stay within the interaction budget", () => {
  const started = performance.now();
  let matches = 0;
  for (const query of ["9999", "customer 1234", "generator 7777", "20260811-0042", "not-found"]) {
    matches += filterProducts(products, query).length;
    matches += filterCustomers(customers, query).length;
    matches += filterConfigurations(configurations, query).length;
    matches += filterBusinessCases(cases, query).length;
    matches += filterPurchaseOrders(orders, query).length;
    matches += filterDocuments(documents, { query, type: "all", status: "all" }).length;
  }
  const elapsedMs = performance.now() - started;
  console.log(`UI interaction benchmark: 30 searches across 10,000 records in ${elapsedMs.toFixed(1)} ms`);
  assert.ok(matches > 0, "benchmark queries should exercise successful matches");
  assert.ok(elapsedMs < 2_500, `UI filtering took ${elapsedMs.toFixed(1)} ms, exceeding the 2,500 ms CI budget`);
});
