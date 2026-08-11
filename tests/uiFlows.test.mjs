import assert from "node:assert/strict";
import test from "node:test";
import {
  filterBusinessCases,
  filterComponents,
  filterConfigurations,
  filterCustomers,
  filterDocuments,
  filterProducts,
  filterPurchaseOrders,
  filterSuppliers,
} from "../src/uiQueries.ts";

const product = {
  id: "product-1", sku: "NE300KW", nameZh: "天然气发电机组", nameEn: "Gas Genset",
  model: "K19N", hsCode: "850220", unit: "Set", grossWeightKg: 2_000, active: true,
};
const customer = {
  id: "customer-1", code: "CUS-001", legalName: "NGO LLC", market: "Russia", currency: "USD",
  paymentTerms: "T/T", address: "Moscow office", shippingAddress: "Kazan warehouse",
  billingAddress: "Moscow billing center", purchaseIntent: "Gas power project",
  customerAnalysis: "Long-term buyer", strengths: "Clear specification", weaknesses: "Long approval",
  contacts: "Ivan +7 000", active: true,
};
const supplier = {
  id: "supplier-1", code: "SUP-001", legalName: "TIDE", address: "Wuhan", contacts: "Amy",
  bankDetails: "", currency: "CNY", paymentTerms: "30% deposit", leadTimeDays: 45,
  onTimeRate: 95, qualificationNotes: "ISO audited", active: true,
  productTerms: [{ id: "term-1", productId: product.id, productSku: product.sku,
    productName: product.nameEn, currency: "CNY", unitPriceMinor: 2_000_000, moq: 1, leadTimeDays: 45 }],
};
const component = {
  id: "component-1", code: "ENG-001", category: "Engine", name: "Cummins engine",
  specification: "K19N natural gas", defaultQuantity: 1, unit: "Set", unitPriceMinor: 1_000_000,
  currency: "CNY", brand: "Cummins", notes: "Main component", active: true,
};
const configuration = {
  id: "configuration-1", code: "CFG-20260811-0001", name: "300kW Generator Set", model: "NE300KW",
  currency: "USD", exchangeRate: 0.14, exchangeRateDate: "2026-08-11", notes: "Russia project",
  totalAmountMinor: 4_935_000, active: true,
  lines: [{ id: "line-1", componentId: component.id, category: component.category, name: component.name,
    specification: component.specification, quantity: 1, unit: component.unit, unitPriceMinor: 4_935_000,
    brand: component.brand, notes: component.notes, amountMinor: 4_935_000 }],
};
const businessCase = {
  id: "case-1", number: "TD-20260811-0001", customerId: customer.id, customerName: customer.legalName,
  stage: "purchase", currency: "USD", incoterm: "FOB", paymentTerms: "T/T",
  shipmentDate: "2026-10-30", notes: "", totalAmountMinor: 4_935_000,
  lines: [{ id: "case-line-1", sourceType: "configurable_product", productId: configuration.id,
    sku: configuration.code, nameZh: configuration.name, nameEn: configuration.name,
    quantity: 1, unit: "Set", unitPriceMinor: 4_935_000, amountMinor: 4_935_000 }],
};
const purchaseOrder = {
  id: "po-1", number: "PO-20260811-0001-1", businessCaseId: businessCase.id,
  businessCaseNumber: businessCase.number, supplierId: supplier.id, supplierName: supplier.legalName,
  status: "confirmed", currency: "CNY", exchangeRate: 0.14, exchangeRateDate: "2026-08-11",
  expectedDate: "2026-10-01", notes: "", totalAmountMinor: 2_000_000,
  completedQuantity: 0, readyQuantity: 0, lines: [],
};
const document = {
  id: "document-1", documentType: "commercial_invoice", number: "INV-20260811-0001",
  businessCaseId: businessCase.id, businessCaseNumber: businessCase.number, customerName: customer.legalName,
  version: 1, status: "issued", language: "en", issueDate: "2026-08-11", currency: "USD",
  templateVersion: "base-1", payload: {}, validationIssues: [], voidReason: "", pdfPath: "invoice.pdf",
  pdfSha256: "abc", exportedAt: "2026-08-11", createdAt: "2026-08-11", updatedAt: "2026-08-11",
};

test("master data searches the fields users actually enter", () => {
  assert.deepEqual(filterProducts([product], "k19n"), [product]);
  assert.deepEqual(filterCustomers([customer], "kazan"), [customer]);
  assert.deepEqual(filterCustomers([customer], "billing center"), [customer]);
  assert.deepEqual(filterSuppliers([supplier], "ne300kw"), [supplier]);
  assert.deepEqual(filterComponents([component], "natural gas"), [component]);
  assert.deepEqual(filterConfigurations([configuration], "cummins"), [configuration]);
});

test("one configured product remains discoverable through business, purchase and document flows", () => {
  assert.deepEqual(filterConfigurations([configuration], "cfg-20260811"), [configuration]);
  assert.deepEqual(filterBusinessCases([businessCase], "ngo llc"), [businessCase]);
  assert.deepEqual(filterPurchaseOrders([purchaseOrder], "td-20260811"), [purchaseOrder]);
  assert.deepEqual(filterPurchaseOrders([purchaseOrder], "tide"), [purchaseOrder]);
  assert.deepEqual(filterDocuments([document], { query: "ngo", type: "commercial_invoice", status: "issued" }), [document]);
  assert.deepEqual(filterDocuments([document], { query: "", type: "packing_list", status: "all" }), []);
});

test("blank and whitespace searches do not hide records", () => {
  assert.equal(filterProducts([product], "   ").length, 1);
  assert.equal(filterBusinessCases([businessCase], "").length, 1);
  assert.equal(filterDocuments([document], { query: " ", type: "all", status: "all" }).length, 1);
});
