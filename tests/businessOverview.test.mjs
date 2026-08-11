import assert from "node:assert/strict";
import test from "node:test";
import { buildBusinessOverview } from "../src/businessOverview.ts";
import { nextPurchaseSplitNumber } from "../src/numbering.ts";

const businessCase = {
  id: "case-1", number: "TD-1", customerId: "customer-1", customerName: "Buyer",
  stage: "production", currency: "USD", incoterm: "FOB", paymentTerms: "T/T",
  shipmentDate: "2026-08-10", notes: "", totalAmountMinor: 100_000,
  lines: [
    { id: "line-1", sourceType: "product", productId: "p1", sku: "A", nameZh: "A", nameEn: "A", quantity: 10, unit: "pcs", unitPriceMinor: 5_000, amountMinor: 50_000 },
    { id: "line-2", sourceType: "product", productId: "p2", sku: "B", nameZh: "B", nameEn: "B", quantity: 20, unit: "pcs", unitPriceMinor: 2_500, amountMinor: 50_000 },
  ],
};

function order(id, currency, sourceCaseLineId, quantity, amountMinor, milestoneStatus = "completed") {
  return {
    id, number: id, businessCaseId: "case-1", businessCaseNumber: "TD-1", supplierId: id,
    supplierName: id, status: "in_production", currency, expectedDate: "2026-08-01", notes: "",
    totalAmountMinor: amountMinor, completedQuantity: 0, readyQuantity: 0,
    lines: [{
      id: `${id}-line`, sourceCaseLineId, productId: sourceCaseLineId, sku: sourceCaseLineId,
      nameZh: sourceCaseLineId, nameEn: sourceCaseLineId, quantity, unit: "pcs",
      unitCostMinor: Math.round(amountMinor / quantity), amountMinor,
      milestones: [{ id: `${id}-milestone`, purchaseOrderLineId: `${id}-line`, stage: "production",
        label: "生产", plannedDate: "2026-08-01", actualDate: "", progress: 50,
        completedQuantity: 0, status: milestoneStatus, issue: milestoneStatus === "blocked" ? "缺料" : "" }],
    }],
  };
}

test("aggregates profit, coverage and cross-process risks for one business case", () => {
  const orders = [
    order("po-usd", "USD", "line-1", 10, 40_000, "blocked"),
    order("po-eur", "EUR", "line-2", 20, 10_000),
  ];
  const shipments = [{ id: "ship-1", number: "S1", businessCaseId: "case-1", businessCaseNumber: "TD-1",
    partnerId: "f1", partnerName: "Forwarder", status: "shipped", plannedDate: "2026-08-01",
    actualDate: "2026-08-01", trackingNumber: "", notes: "",
    lines: [{ id: "sl1", businessCaseLineId: "line-1", sku: "A", productName: "A", quantity: 10, unit: "pcs" }] }];
  const payments = [{ id: "pay-1", number: "PAY1", businessCaseId: "case-1", businessCaseNumber: "TD-1",
    paymentType: "deposit", dueDate: "2026-08-01", currency: "USD", amountMinor: 100_000,
    receivedAmountMinor: 30_000, receivedDate: "", status: "partial", notes: "" }];
  const documents = [{ id: "doc-1", businessCaseId: "case-1", status: "draft",
    validationIssues: [{ severity: "error", code: "quantity", message: "mismatch" }] }];

  const result = buildBusinessOverview(businessCase, orders, shipments, payments, documents, [], "2026-08-11");
  assert.equal(result.purchaseTotalMinor, 40_000);
  assert.equal(result.grossProfitMinor, 60_000);
  assert.equal(result.margin, 60);
  assert.equal(result.purchaseCoverage, 100);
  assert.equal(result.shipmentCoverage, 50);
  assert.equal(result.receivedPercent, 30);
  assert.equal(result.foreignCurrencyOrders.length, 1);
  assert.ok(result.risks.some((risk) => risk.category === "成本币种"));
  assert.ok(result.risks.some((risk) => risk.category === "生产异常"));
  assert.ok(result.risks.some((risk) => risk.category === "收款逾期"));
  assert.ok(result.risks.some((risk) => risk.category === "单证校验"));
});

test("does not flag future fulfillment work while a case is still in quotation", () => {
  const quotation = { ...businessCase, stage: "quotation", shipmentDate: "" };
  const result = buildBusinessOverview(quotation, [], [], [], [], [], "2026-08-11");
  assert.equal(result.purchaseCoverage, 0);
  assert.equal(result.risks.length, 0);
});

test("uses the latest complete cost estimate for profit and target quote risk", () => {
  const estimates = [{
    id: "cost-1", number: "CST-1", businessCaseId: "case-1", businessCaseNumber: "TD-1",
    customerName: "Buyer", currency: "USD", targetMarginBps: 3000, notes: "",
    totalCostMinor: 80_000, suggestedPriceMinor: 114_286, updatedAt: "2026-08-11 10:00:00", lines: [],
  }];
  const result = buildBusinessOverview(businessCase, [], [], [], [], estimates, "2026-08-11");
  assert.equal(result.latestEstimate?.number, "CST-1");
  assert.equal(result.grossProfitMinor, 20_000);
  assert.equal(result.margin, 20);
  assert.ok(result.risks.some((risk) => risk.category === "报价毛利"));
  assert.ok(!result.risks.some((risk) => risk.category === "成本估算"));
});

test("numbers supplier-split purchase orders inside their business case only", () => {
  const caseRecord = { id: "case-1", number: "TD-20260810-0001" };
  const orders = [
    { businessCaseId: "case-1", number: "PO-20260810-0001-1" },
    { businessCaseId: "case-1", number: "PO-20260810-0001-2" },
    { businessCaseId: "case-2", number: "PO-20260810-0001-9" },
    { businessCaseId: "case-1", number: "QUO-20260810-0001" },
  ];
  assert.equal(nextPurchaseSplitNumber(caseRecord, orders), "PO-20260810-0001-3");
});
