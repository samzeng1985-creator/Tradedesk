import type {
  BusinessCase,
  CostEstimate,
  PaymentPlan,
  ProductionMilestone,
  PurchaseOrder,
  ShipmentBatch,
  TradeDocument,
} from "./domain";

export interface OverviewRisk {
  kind: "critical" | "warning" | "info";
  category: string;
  title: string;
  detail: string;
}

export type OverviewMilestone = ProductionMilestone & { supplierName: string; sku: string };

export interface BusinessOverview {
  orders: PurchaseOrder[];
  foreignCurrencyOrders: PurchaseOrder[];
  unconvertedOrders: PurchaseOrder[];
  milestones: OverviewMilestone[];
  shipments: ShipmentBatch[];
  latestEstimate: CostEstimate | null;
  purchaseTotalMinor: number;
  grossProfitMinor: number;
  margin: number;
  plannedPaymentMinor: number;
  receivedPaymentMinor: number;
  receivedPercent: number;
  purchaseCoverage: number;
  productionProgress: number;
  shipmentCoverage: number;
  blockedMilestones: OverviewMilestone[];
  risks: OverviewRisk[];
}

const stageOrder = ["quotation", "order", "purchase", "production", "shipment", "documents"];

function formatAmount(valueMinor: number, currency: string) {
  return `${currency} ${(valueMinor / 100).toFixed(2)}`;
}

function lineCoverage(caseRecord: BusinessCase, allocated: Map<string, number>) {
  if (!caseRecord.lines.length) return 0;
  const total = caseRecord.lines.reduce((sum, line) => {
    if (line.quantity <= 0) return sum;
    return sum + Math.min(1, (allocated.get(line.id) ?? 0) / line.quantity);
  }, 0);
  return Math.round((total / caseRecord.lines.length) * 100);
}

export function buildBusinessOverview(
  caseRecord: BusinessCase | null,
  purchaseOrders: PurchaseOrder[],
  shipmentBatches: ShipmentBatch[],
  paymentPlans: PaymentPlan[],
  documents: TradeDocument[],
  costEstimates: CostEstimate[],
  today = new Date().toISOString().slice(0, 10),
): BusinessOverview {
  if (!caseRecord) return {
    orders: [], foreignCurrencyOrders: [], unconvertedOrders: [], milestones: [], shipments: [], purchaseTotalMinor: 0,
    grossProfitMinor: 0, margin: 0, plannedPaymentMinor: 0, receivedPaymentMinor: 0,
    receivedPercent: 0, purchaseCoverage: 0, productionProgress: 0, shipmentCoverage: 0,
    blockedMilestones: [], risks: [], latestEstimate: null,
  };

  const orders = purchaseOrders.filter((order) =>
    order.businessCaseId === caseRecord.id && order.status !== "cancelled",
  );
  const sameCurrencyOrders = orders.filter((order) => order.currency === caseRecord.currency);
  const foreignCurrencyOrders = orders.filter((order) => order.currency !== caseRecord.currency);
  const unconvertedOrders = foreignCurrencyOrders.filter((order) => !Number.isFinite(order.exchangeRate) || order.exchangeRate <= 0);
  const milestones = orders.flatMap((order) => order.lines.flatMap((line) =>
    line.milestones.map((milestone) => ({ ...milestone, supplierName: order.supplierName, sku: line.sku })),
  ));
  const shipments = shipmentBatches.filter((batch) =>
    batch.businessCaseId === caseRecord.id && batch.status !== "cancelled",
  );
  const payments = paymentPlans.filter((payment) =>
    payment.businessCaseId === caseRecord.id && payment.status !== "cancelled",
  );
  const caseDocuments = documents.filter((document) =>
    document.businessCaseId === caseRecord.id && document.status !== "voided",
  );
  const latestEstimate = costEstimates
    .filter((estimate) => estimate.businessCaseId === caseRecord.id && estimate.currency === caseRecord.currency)
    .sort((left, right) => right.updatedAt.localeCompare(left.updatedAt))[0] ?? null;

  const purchaseAllocation = new Map<string, number>();
  orders.forEach((order) => order.lines.forEach((line) => {
    purchaseAllocation.set(line.sourceCaseLineId, (purchaseAllocation.get(line.sourceCaseLineId) ?? 0) + line.quantity);
  }));
  const shippedAllocation = new Map<string, number>();
  shipments.filter((batch) => batch.status === "shipped" || batch.status === "delivered").forEach((batch) => {
    batch.lines.forEach((line) => {
      shippedAllocation.set(line.businessCaseLineId, (shippedAllocation.get(line.businessCaseLineId) ?? 0) + line.quantity);
    });
  });

  const purchaseTotalMinor = sameCurrencyOrders.reduce((sum, order) => sum + order.totalAmountMinor, 0)
    + foreignCurrencyOrders.reduce((sum, order) => order.exchangeRate > 0
      ? sum + Math.round(order.totalAmountMinor / order.exchangeRate)
      : sum, 0);
  const effectiveCostMinor = latestEstimate?.totalCostMinor ?? purchaseTotalMinor;
  const grossProfitMinor = caseRecord.totalAmountMinor - effectiveCostMinor;
  const margin = caseRecord.totalAmountMinor
    ? Math.round((grossProfitMinor / caseRecord.totalAmountMinor) * 100)
    : 0;
  const plannedPaymentMinor = payments.reduce((sum, payment) => sum + payment.amountMinor, 0);
  const receivedPaymentMinor = payments.reduce((sum, payment) => sum + payment.receivedAmountMinor, 0);
  const receivedPercent = caseRecord.totalAmountMinor
    ? Math.min(100, Math.round((receivedPaymentMinor / caseRecord.totalAmountMinor) * 100))
    : 0;
  const purchaseCoverage = lineCoverage(caseRecord, purchaseAllocation);
  const shipmentCoverage = lineCoverage(caseRecord, shippedAllocation);
  const productionProgress = milestones.length
    ? Math.round(milestones.reduce((sum, milestone) => sum + milestone.progress, 0) / milestones.length)
    : 0;
  const blockedMilestones = milestones.filter((milestone) => milestone.status === "blocked");
  const overdueMilestones = milestones.filter((milestone) =>
    milestone.status !== "completed" && milestone.plannedDate !== "" && milestone.plannedDate < today,
  );
  const overduePayments = payments.filter((payment) =>
    payment.status !== "received" && payment.dueDate !== "" && payment.dueDate < today,
  );
  const documentErrors = caseDocuments.reduce((sum, document) =>
    sum + document.validationIssues.filter((issue) => issue.severity === "error").length, 0);
  const stageIndex = stageOrder.indexOf(caseRecord.stage);
  const risks: OverviewRisk[] = [];

  if (!latestEstimate && stageIndex >= 1) risks.push({ kind: "info", category: "成本估算", title: "尚未建立完整成本估算", detail: "当前毛利仅参考已录及已折算采购成本，尚未覆盖包装、物流、税费、佣金等间接成本。" });
  if (unconvertedOrders.length) risks.push({ kind: "critical", category: "成本币种", title: `${unconvertedOrders.length} 张采购单尚未折算`, detail: `采购币种与业务单 ${caseRecord.currency} 不一致，请编辑采购单并补充汇率后计入工作台。` });
  if (purchaseCoverage < 100 && (stageIndex >= 2 || orders.length > 0)) risks.push({ kind: purchaseCoverage === 0 ? "critical" : "warning", category: "采购覆盖", title: `采购覆盖率 ${purchaseCoverage}%`, detail: purchaseCoverage === 0 ? "业务单尚未下推采购，无法形成可靠成本和交付计划。" : "仍有产品数量未分配采购，请检查供应商拆单。" });
  if ((latestEstimate || (orders.length && !unconvertedOrders.length)) && grossProfitMinor < 0) risks.push({ kind: "critical", category: "利润", title: "预计毛利为负", detail: `当前成本已超过销售金额 ${formatAmount(Math.abs(grossProfitMinor), caseRecord.currency)}。` });
  if (latestEstimate && !unconvertedOrders.length && purchaseTotalMinor > latestEstimate.totalCostMinor) risks.push({ kind: "warning", category: "成本偏差", title: "实际采购已超过完整成本估算", detail: `折算采购成本比成本估算高 ${formatAmount(purchaseTotalMinor - latestEstimate.totalCostMinor, caseRecord.currency)}，请更新估算或复核采购价。` });
  if (latestEstimate && latestEstimate.suggestedPriceMinor > caseRecord.totalAmountMinor && grossProfitMinor >= 0) risks.push({ kind: "warning", category: "报价毛利", title: "销售金额低于建议最低报价", detail: `按目标毛利率计算，当前报价尚差 ${formatAmount(latestEstimate.suggestedPriceMinor - caseRecord.totalAmountMinor, caseRecord.currency)}。` });
  blockedMilestones.slice(0, 2).forEach((milestone) => risks.push({ kind: "critical", category: "生产异常", title: `${milestone.sku} · ${milestone.label}`, detail: milestone.issue || "生产节点已阻断，请向供应商确认恢复日期。" }));
  if (overdueMilestones.length) risks.push({ kind: "warning", category: "生产延期", title: `${overdueMilestones.length} 个生产节点已超过计划日期`, detail: "请更新实际进度、完成数量或延期原因。" });
  if (caseRecord.shipmentDate && caseRecord.shipmentDate < today && shipmentCoverage < 100) risks.push({ kind: "critical", category: "装运", title: `计划发货日已到，发运覆盖率 ${shipmentCoverage}%`, detail: "仍有数量未记录为已发运或已交付。" });
  if (overduePayments.length) risks.push({ kind: "critical", category: "收款逾期", title: `${overduePayments.length} 个收款节点逾期`, detail: `逾期未收 ${formatAmount(overduePayments.reduce((sum, payment) => sum + Math.max(0, payment.amountMinor - payment.receivedAmountMinor), 0), caseRecord.currency)}。` });
  if (plannedPaymentMinor < caseRecord.totalAmountMinor && (stageIndex >= 1 || payments.length > 0)) risks.push({ kind: "info", category: "收款计划", title: "收款计划尚未覆盖销售总额", detail: `尚差 ${formatAmount(caseRecord.totalAmountMinor - plannedPaymentMinor, caseRecord.currency)} 未安排收款节点。` });
  if (documentErrors) risks.push({ kind: "critical", category: "单证校验", title: `${documentErrors} 个阻断错误待处理`, detail: "存在跨单证数量、金额或运输信息不一致，处理后才能签发。" });

  return {
    orders, foreignCurrencyOrders, unconvertedOrders, milestones, shipments, latestEstimate, purchaseTotalMinor, grossProfitMinor,
    margin, plannedPaymentMinor, receivedPaymentMinor, receivedPercent, purchaseCoverage,
    productionProgress, shipmentCoverage, blockedMilestones, risks,
  };
}
