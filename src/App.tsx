import { useEffect, useState } from "react";
import { businessCaseApi, costEstimateApi, documentApi, fulfillmentApi, logisticsApi, masterApi, workspaceApi } from "./api";
import { buildBusinessOverview } from "./businessOverview";
import { BusinessCaseCenter } from "./BusinessCaseCenter";
import { CompanySettings } from "./CompanySettings";
import { CostEstimateCenter } from "./CostEstimateCenter";
import { ComponentLibrary, ConfigurableProductLibrary } from "./ConfigurableProductCenter";
import { DataSecurityCenter, RecoveryKeyNotice } from "./DataSecurityCenter";
import { DocumentCenter } from "./DocumentCenter";
import { FulfillmentCenter } from "./FulfillmentCenter";
import { LogisticsCenter } from "./LogisticsCenter";
import { MasterEditor } from "./MasterEditor";
import type { MasterInput, MasterRecord, MasterTab } from "./MasterEditor";
import { UnlockScreen } from "./UnlockScreen";
import type {
  BusinessCase,
  BusinessCaseInput,
  CompanyRegistry,
  CompanyRegistryInput,
  ComponentOption,
  ComponentOptionInput,
  ComponentOptionTranslationInput,
  ConfigurationLanguage,
  ConfigComponent,
  ConfigComponentInput,
  ConfigurableProduct,
  ConfigurableProductInput,
  ConvertDocumentInput,
  CreateDocumentInput,
  Customer,
  CostEstimate,
  CostEstimateInput,
  PipelineStage,
  Partner,
  PartnerInput,
  PaymentPlan,
  PaymentPlanInput,
  Product,
  ProductionMilestoneInput,
  PurchaseOrder,
  PurchaseOrderInput,
  PurchaseStatus,
  RecordStatus,
  SaveDocumentInput,
  ShipmentBatch,
  ShipmentBatchInput,
  Supplier,
  TradeDocument,
  WorkspaceSummary,
} from "./domain";

type View = "overview" | "cases" | "masters" | "costs" | "fulfillment" | "logistics" | "documents" | "settings" | "security";

const pipeline: Array<{ key: PipelineStage; label: string }> = [
  { key: "quotation", label: "报价" },
  { key: "order", label: "订单" },
  { key: "purchase", label: "采购" },
  { key: "production", label: "生产" },
  { key: "shipment", label: "发货" },
  { key: "documents", label: "单证" },
];

const viewTitles: Record<View, { title: string; subtitle: string }> = {
  overview: { title: "业务工作台", subtitle: "从订单到单证的轻量闭环" },
  cases: { title: "业务单", subtitle: "客户、产品与商业条款的统一业务快照" },
  masters: { title: "主数据", subtitle: "一次建档，多处复用" },
  costs: { title: "成本估算", subtitle: "完整成本、目标毛利与报价底线" },
  fulfillment: { title: "采购与生产", subtitle: "只跟踪关键里程碑，不做复杂排产" },
  logistics: { title: "装运与收款", subtitle: "分批发货、物流合作方与收款节点统一管理" },
  documents: { title: "单证中心", subtitle: "版本、状态与跨单证一致性" },
  settings: { title: "企业设置", subtitle: "统一配置公司名称、Logo 和电子签名" },
  security: { title: "数据与安全", subtitle: "恢复密钥、加密备份与附件保护" },
};

const statusText: Record<RecordStatus, string> = {
  ready: "已完成",
  working: "进行中",
  blocked: "待处理",
  draft: "草稿",
};

function money(value: number, currency: string) {
  try {
    return new Intl.NumberFormat("zh-CN", {
      style: "currency",
      currency,
      maximumFractionDigits: 0,
    }).format(value / 100);
  } catch {
    return `${currency || "CNY"} ${(value / 100).toFixed(0)}`;
  }
}

function milestoneRecordStatus(status: "pending" | "in_progress" | "completed" | "blocked"): RecordStatus {
  if (status === "completed") return "ready";
  if (status === "in_progress") return "working";
  if (status === "blocked") return "blocked";
  return "draft";
}

function Status({ status }: { status: RecordStatus }) {
  return <span className={`status status-${status}`}>{statusText[status]}</span>;
}

function Pipeline({ stage }: { stage: PipelineStage }) {
  const activeIndex = pipeline.findIndex((item) => item.key === stage);
  return (
    <ol className="pipeline" aria-label="业务流程进度">
      {pipeline.map((item, index) => (
        <li
          className={index < activeIndex ? "done" : index === activeIndex ? "active" : ""}
          key={item.key}
        >
          <span className="pipeline-index">{index + 1}</span>
          <span>{item.label}</span>
        </li>
      ))}
    </ol>
  );
}

export default function App() {
  const [workspace, setWorkspace] = useState<WorkspaceSummary | null>(null);
  const [companyRegistry, setCompanyRegistry] = useState<CompanyRegistry | null>(null);
  const [workspaceExists, setWorkspaceExists] = useState(false);
  const [workspaceChecking, setWorkspaceChecking] = useState(true);
  const [view, setView] = useState<View>("overview");
  const [masterTab, setMasterTab] = useState<MasterTab>("products");
  const [products, setProducts] = useState<Product[]>([]);
  const [configComponents, setConfigComponents] = useState<ConfigComponent[]>([]);
  const [componentOptions, setComponentOptions] = useState<ComponentOption[]>([]);
  const [configurableProducts, setConfigurableProducts] = useState<ConfigurableProduct[]>([]);
  const [customers, setCustomers] = useState<Customer[]>([]);
  const [suppliers, setSuppliers] = useState<Supplier[]>([]);
  const [businessCases, setBusinessCases] = useState<BusinessCase[]>([]);
  const [costEstimates, setCostEstimates] = useState<CostEstimate[]>([]);
  const [purchaseOrders, setPurchaseOrders] = useState<PurchaseOrder[]>([]);
  const [partners, setPartners] = useState<Partner[]>([]);
  const [shipmentBatches, setShipmentBatches] = useState<ShipmentBatch[]>([]);
  const [paymentPlans, setPaymentPlans] = useState<PaymentPlan[]>([]);
  const [documents, setDocuments] = useState<TradeDocument[]>([]);
  const [overviewCaseId, setOverviewCaseId] = useState("");
  const [masterQuery, setMasterQuery] = useState("");
  const [editorOpen, setEditorOpen] = useState(false);
  const [editingRecord, setEditingRecord] = useState<MasterRecord | null>(null);
  const [saving, setSaving] = useState(false);
  const [masterTransferBusy, setMasterTransferBusy] = useState(false);
  const [masterTransferMessage, setMasterTransferMessage] = useState("");
  const [recoveryKeyNotice, setRecoveryKeyNotice] = useState("");
  const [restorePending, setRestorePending] = useState(false);

  useEffect(() => {
    Promise.all([workspaceApi.exists(), workspaceApi.restorePending()])
      .then(([exists, pending]) => { setWorkspaceExists(exists); setRestorePending(pending); })
      .finally(() => setWorkspaceChecking(false));
  }, []);

  useEffect(() => {
    if (businessCases.some((item) => item.id === overviewCaseId)) return;
    setOverviewCaseId(businessCases[0]?.id ?? "");
  }, [businessCases, overviewCaseId]);

  async function loadMasterData() {
    const [nextProducts, nextComponents, nextComponentOptions, nextConfigurations, nextCustomers, nextSuppliers, nextCases, nextCostEstimates, nextOrders, nextPartners, nextShipments, nextPayments, nextDocuments, summary] = await Promise.all([
      masterApi.listProducts(),
      masterApi.listConfigComponents(),
      masterApi.listComponentOptions(),
      masterApi.listConfigurableProducts(),
      masterApi.listCustomers(),
      masterApi.listSuppliers(),
      businessCaseApi.list(),
      costEstimateApi.list(),
      fulfillmentApi.list(),
      logisticsApi.listPartners(),
      logisticsApi.listShipments(),
      logisticsApi.listPayments(),
      documentApi.list(),
      workspaceApi.summary(),
    ]);
    setProducts(nextProducts);
    setConfigComponents(nextComponents);
    setComponentOptions(nextComponentOptions);
    setConfigurableProducts(nextConfigurations);
    setCustomers(nextCustomers);
    setSuppliers(nextSuppliers);
    setBusinessCases(nextCases);
    setCostEstimates(nextCostEstimates);
    setPurchaseOrders(nextOrders);
    setPartners(nextPartners);
    setShipmentBatches(nextShipments);
    setPaymentPlans(nextPayments);
    setDocuments(nextDocuments);
    setWorkspace(summary);
  }

  async function unlock(password: string, companyName?: string) {
    const summary = await workspaceApi.unlock(password, companyName);
    if (summary.recoveryKey) setRecoveryKeyNotice(summary.recoveryKey);
    await loadMasterData();
    setCompanyRegistry(await workspaceApi.companyRegistry());
    setWorkspaceExists(true);
    setRestorePending(false);
  }

  async function recover(recoveryKey: string) {
    await workspaceApi.unlockWithRecovery(recoveryKey);
    await loadMasterData();
    setCompanyRegistry(await workspaceApi.companyRegistry());
    setWorkspaceExists(true);
    setRestorePending(false);
  }

  async function restoreBackup(bytes: number[]) {
    await workspaceApi.restoreBackup(bytes);
    setWorkspaceExists(true);
    setRestorePending(await workspaceApi.restorePending());
  }

  async function rollbackRestore() {
    await workspaceApi.rollbackRestore();
    setRestorePending(false);
    setWorkspaceExists(await workspaceApi.exists());
  }

  async function lock() {
    await workspaceApi.lock();
    setWorkspace(null);
    setCompanyRegistry(null);
    setWorkspaceExists(true);
  }

  async function saveCompanyRegistry(input: CompanyRegistryInput) {
    const saved = await workspaceApi.saveCompanyRegistry(input);
    setCompanyRegistry(saved);
    setWorkspace(await workspaceApi.summary());
  }

  function openEditor(record: MasterRecord | null = null) {
    setEditingRecord(record);
    setEditorOpen(true);
  }

  async function saveMaster(input: MasterInput) {
    setSaving(true);
    try {
      if (masterTab === "products") await masterApi.saveProduct(input as Parameters<typeof masterApi.saveProduct>[0]);
      if (masterTab === "customers") await masterApi.saveCustomer(input as Parameters<typeof masterApi.saveCustomer>[0]);
      if (masterTab === "suppliers") await masterApi.saveSupplier(input as Parameters<typeof masterApi.saveSupplier>[0]);
      await loadMasterData();
      setEditorOpen(false);
    } finally {
      setSaving(false);
    }
  }

  async function exportMasterWorkbook(templateOnly: boolean) {
    setMasterTransferBusy(true); setMasterTransferMessage("");
    try {
      const path = await masterApi.exportWorkbook(templateOnly);
      setMasterTransferMessage(`${templateOnly ? "导入模板" : "主数据"}已导出：${path}`);
    } catch (reason) { setMasterTransferMessage(String(reason)); } finally { setMasterTransferBusy(false); }
  }

  async function importMasterWorkbook(file: File) {
    setMasterTransferBusy(true); setMasterTransferMessage("");
    try {
      if (!file.name.toLowerCase().endsWith(".xlsx")) throw new Error("请选择 .xlsx 格式的 Excel 文件");
      const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));
      const result = await masterApi.importWorkbook(bytes);
      await loadMasterData();
      setMasterTransferMessage(`导入完成：产品 ${result.products}、客户 ${result.customers}、供应商 ${result.suppliers}、组件 ${result.components}、自选配置 ${result.configurations}`);
    } catch (reason) { setMasterTransferMessage(String(reason)); } finally { setMasterTransferBusy(false); }
  }

  async function archiveMaster(entity: "product" | "customer" | "supplier", record: MasterRecord) {
    if (!window.confirm(`停用“${"sku" in record ? record.sku : record.legalName}”？历史资料不会被删除。`)) return;
    await masterApi.archive(entity, record.id);
    await loadMasterData();
  }

  async function saveConfigComponent(input: ConfigComponentInput) {
    await masterApi.saveConfigComponent(input);
    await loadMasterData();
  }

  async function saveConfigurableProduct(input: ConfigurableProductInput) {
    await masterApi.saveConfigurableProduct(input);
    await loadMasterData();
  }

  async function saveComponentOption(input: ComponentOptionInput) {
    await masterApi.saveComponentOption(input);
    await loadMasterData();
  }

  async function archiveComponentOption(id: string) {
    await masterApi.archive("component_option", id);
    await loadMasterData();
  }

  async function saveComponentOptionTranslation(input: ComponentOptionTranslationInput) {
    await masterApi.saveComponentOptionTranslation(input);
    await loadMasterData();
  }

  async function exportConfigurationPdf(id: string, language: ConfigurationLanguage, companyId: string, signingAssetId: string) {
    const result = await masterApi.exportConfigurationPdf(id, language, companyId, signingAssetId);
    return result.path;
  }

  async function exportConfigurationCsv(id: string, language: ConfigurationLanguage) {
    return masterApi.exportConfigurationCsv(id, language);
  }

  async function printConfiguration(id: string, language: ConfigurationLanguage, companyId: string, signingAssetId: string) {
    const result = await masterApi.printConfiguration(id, language, companyId, signingAssetId);
    return result.path;
  }

  async function archiveConfigMaster(entity: "config_component" | "configurable_product", id: string) {
    await masterApi.archive(entity, id);
    await loadMasterData();
  }

  async function saveBusinessCase(input: BusinessCaseInput) {
    await businessCaseApi.save(input);
    await loadMasterData();
  }

  async function updateBusinessCaseStage(id: string, stage: PipelineStage) {
    await businessCaseApi.updateStage(id, stage);
    await loadMasterData();
  }

  async function archiveBusinessCase(id: string) {
    await businessCaseApi.archive(id);
    await loadMasterData();
  }

  async function saveCostEstimate(input: CostEstimateInput) {
    await costEstimateApi.save(input);
    await loadMasterData();
  }

  async function archiveCostEstimate(id: string) {
    await costEstimateApi.archive(id);
    await loadMasterData();
  }

  async function createPurchaseOrder(input: PurchaseOrderInput) {
    await fulfillmentApi.create(input);
    await loadMasterData();
  }

  async function updatePurchaseStatus(id: string, status: PurchaseStatus) {
    await fulfillmentApi.updateStatus(id, status);
    await loadMasterData();
  }

  async function updateProductionMilestone(input: ProductionMilestoneInput) {
    await fulfillmentApi.updateMilestone(input);
    await loadMasterData();
  }

  async function savePartner(input: PartnerInput) {
    await logisticsApi.savePartner(input);
    await loadMasterData();
  }

  async function archivePartner(id: string) {
    await logisticsApi.archivePartner(id);
    await loadMasterData();
  }

  async function saveShipment(input: ShipmentBatchInput) {
    await logisticsApi.saveShipment(input);
    await loadMasterData();
  }

  async function savePayment(input: PaymentPlanInput) {
    await logisticsApi.savePayment(input);
    await loadMasterData();
  }

  async function createDocument(input: CreateDocumentInput) {
    const document = await documentApi.create(input);
    await loadMasterData();
    return document;
  }

  async function convertDocument(input: ConvertDocumentInput) {
    const document = await documentApi.convert(input);
    await loadMasterData();
    return document;
  }

  async function saveDocument(input: SaveDocumentInput) {
    const document = await documentApi.save(input);
    await loadMasterData();
    return document;
  }

  async function issueDocument(id: string) {
    const document = await documentApi.issue(id);
    await loadMasterData();
    return document;
  }

  async function voidDocument(id: string, reason: string) {
    const document = await documentApi.void(id, reason);
    await loadMasterData();
    return document;
  }

  async function createDocumentVersion(id: string) {
    const document = await documentApi.newVersion(id);
    await loadMasterData();
    return document;
  }

  async function exportDocumentPdf(id: string, companyId: string, signingAssetId: string) {
    const result = await documentApi.exportPdf(id, companyId, signingAssetId);
    await loadMasterData();
    return result.path;
  }

  async function exportDocumentCsv(id: string) {
    return documentApi.exportCsv(id);
  }

  async function printDocument(id: string, companyId: string, signingAssetId: string) {
    const result = await documentApi.print(id, companyId, signingAssetId);
    await loadMasterData();
    return result.path;
  }

  const currentCase = businessCases.find((item) => item.id === overviewCaseId) ?? businessCases[0] ?? null;
  const overview = buildBusinessOverview(currentCase, purchaseOrders, shipmentBatches, paymentPlans, documents, costEstimates);
  const {
    orders: currentOrders, foreignCurrencyOrders, milestones: currentMilestones,
    shipments: currentShipments, purchaseTotalMinor, grossProfitMinor, margin,
    plannedPaymentMinor, receivedPaymentMinor, receivedPercent, purchaseCoverage, latestEstimate,
    productionProgress, shipmentCoverage, blockedMilestones, risks: overviewRisks,
  } = overview;

  const normalizedQuery = masterQuery.trim().toLocaleLowerCase();
  const filteredProducts = products.filter((item) =>
    [item.sku, item.nameZh, item.nameEn, item.model, item.hsCode].some((value) =>
      value.toLocaleLowerCase().includes(normalizedQuery),
    ),
  );
  const filteredCustomers = customers.filter((item) =>
    [
      item.code, item.legalName, item.market, item.currency, item.address,
      item.shippingAddress, item.billingAddress, item.purchaseIntent,
      item.customerAnalysis, item.contacts,
    ].some((value) =>
      value.toLocaleLowerCase().includes(normalizedQuery),
    ),
  );
  const filteredSuppliers = suppliers.filter((item) =>
    [item.code, item.legalName].some((value) =>
      value.toLocaleLowerCase().includes(normalizedQuery),
    ),
  );

  function prepareDocuments() {
    setView("documents");
  }

  if (!workspace) {
    return <UnlockScreen checking={workspaceChecking} existing={workspaceExists} restorePending={restorePending} onUnlock={unlock} onRecover={recover} onRestore={restoreBackup} onRollbackRestore={rollbackRestore} />;
  }

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <span className="brand-mark">TD</span>
          <span>
            <strong>TradeDesk</strong>
            <small>Local · 0.21.0</small>
          </span>
        </div>

        <nav aria-label="主导航">
          <button className={view === "overview" ? "selected" : ""} onClick={() => setView("overview")}>
            工作台
          </button>
          <button className={view === "cases" ? "selected" : ""} onClick={() => setView("cases")}>
            业务单
          </button>
          <button className={view === "masters" ? "selected" : ""} onClick={() => setView("masters")}>
            主数据
          </button>
          <button className={view === "costs" ? "selected" : ""} onClick={() => setView("costs")}>
            成本估算
          </button>
          <button
            className={view === "fulfillment" ? "selected" : ""}
            onClick={() => setView("fulfillment")}
          >
            采购与生产
          </button>
          <button className={view === "logistics" ? "selected" : ""} onClick={() => setView("logistics")}>
            装运与收款
          </button>
          <button className={view === "documents" ? "selected" : ""} onClick={() => setView("documents")}>
            单证中心
          </button>
          <button className={view === "settings" ? "selected" : ""} onClick={() => setView("settings")}>
            企业设置
          </button>
          <button className={view === "security" ? "selected" : ""} onClick={() => setView("security")}>
            数据与安全
          </button>
        </nav>

        <div className="workspace-state">
          <span className="lock-dot" aria-hidden="true" />
          <div>
            <strong>{workspace.companyName}</strong>
            <small>SQLCipher 已加密</small>
          </div>
        </div>
      </aside>

      <main>
        <header className="topbar">
          <div>
            <h1>{viewTitles[view].title}</h1>
            <p>{viewTitles[view].subtitle}</p>
          </div>
          <div className="top-actions">
            <span className="demo-badge">本地加密工作区</span>
            <button className="button button-secondary" onClick={lock}>锁定</button>
          </div>
        </header>

        {view === "overview" && (
          <div className="page-stack">
            <section className="case-hero">
              {currentCase ? <>
                <div className="case-heading">
                  <div>
                    <span className="eyebrow">业务利润与风险</span>
                    <h2>{currentCase.number}</h2>
                    <p>{currentCase.customerName} · 计划发货 {currentCase.shipmentDate || "未设置"}</p>
                  </div>
                  <div className="overview-case-actions">
                    <label><span>查看业务单</span><select value={currentCase.id} onChange={(event) => setOverviewCaseId(event.target.value)}>{businessCases.map((item) => <option value={item.id} key={item.id}>{item.number} · {item.customerName}</option>)}</select></label>
                    <button className="button button-primary" onClick={() => setView("cases")}>打开业务单</button>
                  </div>
                </div>
                <Pipeline stage={currentCase.stage} />
              </> : <div className="empty-overview"><span className="eyebrow">开始第一笔业务</span><h2>建立业务单后即可跟踪采购与生产</h2><button className="button button-primary" onClick={() => setView("cases")}>前往业务单</button></div>}
            </section>

            <section className="metric-grid overview-metrics" aria-label="业务利润与履约指标">
              <article>
                <span>销售金额</span>
                <strong>{money(currentCase?.totalAmountMinor ?? 0, currentCase?.currency ?? "USD")}</strong>
                <small>{currentCase?.currency ?? "暂无业务单"}</small>
              </article>
              <article>
                <span>{latestEstimate ? "完整估算成本" : "同币采购成本"}</span>
                <strong>{money(latestEstimate?.totalCostMinor ?? purchaseTotalMinor, currentCase?.currency ?? "USD")}</strong>
                <small>{latestEstimate ? `${latestEstimate.number} · 目标毛利 ${(latestEstimate.targetMarginBps / 100).toFixed(2)}%` : foreignCurrencyOrders.length ? `${foreignCurrencyOrders.length} 张异币采购未计入` : `${currentOrders.length} 张有效采购单`}</small>
              </article>
              <article className={grossProfitMinor < 0 ? "metric-danger" : ""}>
                <span>暂估毛利</span>
                <strong>{latestEstimate ? money(grossProfitMinor, currentCase?.currency ?? "USD") : foreignCurrencyOrders.length ? "待折算" : currentOrders.length ? money(grossProfitMinor, currentCase?.currency ?? "USD") : "待估算"}</strong>
                <small>{latestEstimate ? `完整成本口径 · 暂估毛利率 ${margin}%` : currentOrders.length && !foreignCurrencyOrders.length ? purchaseCoverage < 100 ? `仅基于已录成本 · 采购覆盖 ${purchaseCoverage}%` : `暂估毛利率 ${margin}%` : "建立成本估算后计算"}</small>
              </article>
              <article>
                <span>已收款</span>
                <strong>{money(receivedPaymentMinor, currentCase?.currency ?? "USD")}</strong>
                <small>{receivedPercent}% · 计划 {money(plannedPaymentMinor, currentCase?.currency ?? "USD")}</small>
              </article>
              <article>
                <span>采购覆盖</span>
                <strong>{purchaseCoverage}%</strong>
                <small>{new Set(currentOrders.map((order) => order.supplierId)).size} 个供应商</small>
              </article>
              <article>
                <span>生产进度</span>
                <strong>{productionProgress}%</strong>
                <small>{blockedMilestones.length ? `${blockedMilestones.length} 个阻断节点` : "暂无阻断节点"}</small>
              </article>
              <article>
                <span>已发运覆盖</span>
                <strong>{shipmentCoverage}%</strong>
                <small>{currentShipments.length} 个装运批次</small>
              </article>
              <article className={overviewRisks.some((risk) => risk.kind === "critical") ? "metric-danger" : ""}>
                <span>风险事项</span>
                <strong>{overviewRisks.length}</strong>
                <small>{overviewRisks.filter((risk) => risk.kind === "critical").length} 个阻断风险</small>
              </article>
            </section>

            <div className="two-column">
              <section className="panel">
                <div className="panel-heading">
                  <div>
                    <h2>生产里程碑</h2>
                    <p>只显示影响交付的关键节点</p>
                  </div>
                  <button className="text-button" onClick={() => setView("fulfillment")}>
                    查看全部
                  </button>
                </div>
                <div className="milestone-list">
                  {currentMilestones.slice(0, 5).map((item) => (
                    <div className="milestone" key={item.id}>
                      <div className="milestone-row">
                        <div>
                          <strong>{item.sku} · {item.label}</strong>
                          <span>{item.supplierName}</span>
                        </div>
                        <Status status={milestoneRecordStatus(item.status)} />
                      </div>
                      <div className="progress-track" aria-label={`${item.label} ${item.progress}%`}>
                        <span style={{ width: `${item.progress}%` }} />
                      </div>
                      <div className="milestone-meta">
                        <span>{item.progress}%</span>
                        <span>计划 {item.plannedDate || "未设置"}</span>
                      </div>
                    </div>
                  ))}
                  {!currentMilestones.length && <div className="empty-table">创建采购单后自动生成生产里程碑</div>}
                </div>
              </section>

              <section className="panel">
                <div className="panel-heading">
                  <div>
                    <h2>待处理事项</h2>
                    <p>按业务阻断程度排序</p>
                  </div>
                </div>
                <div className="issue-list">
                  {overviewRisks.slice(0, 6).map((risk, index) => <article className={`issue issue-${risk.kind}`} key={`${risk.category}-${index}`}><span>{risk.category}</span><strong>{risk.title}</strong><p>{risk.detail}</p></article>)}
                  {!overviewRisks.length && <article className="issue issue-success"><span>业务健康</span><strong>当前没有待处理风险</strong><p>采购、生产、装运、收款和单证校验均未发现阻断事项。</p></article>}
                  <button className="button button-primary button-wide" onClick={prepareDocuments}>
                    生成待制单证
                  </button>
                </div>
              </section>
            </div>
          </div>
        )}

        {view === "cases" && (
          <BusinessCaseCenter
            cases={businessCases}
            customers={customers}
            products={products}
            configurableProducts={configurableProducts}
            onSave={saveBusinessCase}
            onUpdateStage={updateBusinessCaseStage}
            onArchive={archiveBusinessCase}
          />
        )}

        {view === "masters" && (
          <section className="panel master-panel">
            <div className="tabs" role="tablist" aria-label="主数据类型">
              <button role="tab" aria-selected={masterTab === "products"} onClick={() => setMasterTab("products")}>
                产品 {products.length}
              </button>
              <button role="tab" aria-selected={masterTab === "configurable"} onClick={() => setMasterTab("configurable")}>
                自选配置 {configurableProducts.length}
              </button>
              <button role="tab" aria-selected={masterTab === "components"} onClick={() => setMasterTab("components")}>
                组件库 {configComponents.length}
              </button>
              <button role="tab" aria-selected={masterTab === "customers"} onClick={() => setMasterTab("customers")}>
                客户 {customers.length}
              </button>
              <button role="tab" aria-selected={masterTab === "suppliers"} onClick={() => setMasterTab("suppliers")}>
                供应商 {suppliers.length}
              </button>
            </div>
            <div className="master-transfer-toolbar"><div><strong>Excel 批量维护</strong><span>一次导出全部主数据，或按模板校验后批量导入</span></div><div className="toolbar-buttons"><button className="button button-secondary" disabled={masterTransferBusy} onClick={() => void exportMasterWorkbook(false)}>导出主数据</button><button className="button button-secondary" disabled={masterTransferBusy} onClick={() => void exportMasterWorkbook(true)}>下载导入模板</button><label className="button button-primary">{masterTransferBusy ? "处理中…" : "导入 Excel"}<input className="sr-only" type="file" accept=".xlsx,application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" disabled={masterTransferBusy} onChange={(event) => { const file = event.target.files?.[0]; event.target.value = ""; if (file) void importMasterWorkbook(file); }} /></label></div></div>
            {masterTransferMessage && <div className="document-message">{masterTransferMessage}</div>}

            {(masterTab === "products" || masterTab === "customers" || masterTab === "suppliers") && <div className="table-toolbar">
              <label>
                <span className="sr-only">搜索主数据</span>
                <input
                  placeholder={masterTab === "customers" ? "搜索客户、地址、联系人或购买意向" : "按编号、名称或 HS 编码搜索"}
                  value={masterQuery}
                  onChange={(event) => setMasterQuery(event.target.value)}
                />
              </label>
              <button className="button button-primary" onClick={() => openEditor()}>新建记录</button>
            </div>}

            {(masterTab === "products" || masterTab === "customers" || masterTab === "suppliers") && <div className="table-wrap">
              {masterTab === "products" && (
                <table>
                  <thead>
                    <tr><th>SKU</th><th>产品</th><th>型号</th><th>HS 编码</th><th>单位</th><th>毛重</th><th>操作</th></tr>
                  </thead>
                  <tbody>
                    {filteredProducts.map((item) => (
                      <tr key={item.id}><td>{item.sku}</td><td><strong>{item.nameEn}</strong><small className="table-subtitle">{item.nameZh}</small></td><td>{item.model || "—"}</td><td>{item.hsCode || "—"}</td><td>{item.unit}</td><td>{item.grossWeightKg} kg</td><td><div className="row-actions"><button onClick={() => openEditor(item)}>编辑</button><button onClick={() => archiveMaster("product", item)}>停用</button></div></td></tr>
                    ))}
                  </tbody>
                </table>
              )}
              {masterTab === "customers" && (
                <table>
                  <thead>
                    <tr><th>客户编码</th><th>客户</th><th>市场</th><th>币种</th><th>默认付款条款</th><th>操作</th></tr>
                  </thead>
                  <tbody>
                    {filteredCustomers.map((item) => (
                      <tr key={item.id}><td>{item.code}</td><td>{item.legalName}</td><td>{item.market || "—"}</td><td>{item.currency}</td><td>{item.paymentTerms || "—"}</td><td><div className="row-actions"><button onClick={() => openEditor(item)}>编辑</button><button onClick={() => archiveMaster("customer", item)}>停用</button></div></td></tr>
                    ))}
                  </tbody>
                </table>
              )}
              {masterTab === "suppliers" && (
                <table>
                  <thead>
                    <tr><th>供应商编码</th><th>供应商</th><th>默认交期</th><th>准时率</th><th>状态</th><th>操作</th></tr>
                  </thead>
                  <tbody>
                    {filteredSuppliers.map((item) => (
                      <tr key={item.id}><td>{item.code}</td><td>{item.legalName}</td><td>{item.leadTimeDays} 天</td><td>{item.onTimeRate}%</td><td><Status status="ready" /></td><td><div className="row-actions"><button onClick={() => openEditor(item)}>编辑</button><button onClick={() => archiveMaster("supplier", item)}>停用</button></div></td></tr>
                    ))}
                  </tbody>
                </table>
              )}
            </div>}
            {masterTab === "configurable" && companyRegistry && <ConfigurableProductLibrary companyRegistry={companyRegistry} configurations={configurableProducts} components={configComponents} options={componentOptions} onSave={saveConfigurableProduct} onArchive={(id) => archiveConfigMaster("configurable_product", id)} onExportPdf={exportConfigurationPdf} onExportCsv={exportConfigurationCsv} onPrint={printConfiguration} />}
            {masterTab === "components" && <ComponentLibrary components={configComponents} options={componentOptions} onSave={saveConfigComponent} onArchive={(id) => archiveConfigMaster("config_component", id)} onSaveOption={saveComponentOption} onSaveOptionTranslation={saveComponentOptionTranslation} onArchiveOption={archiveComponentOption} />}
          </section>
        )}

        {view === "fulfillment" && (
          <FulfillmentCenter
            orders={purchaseOrders}
            cases={businessCases}
            suppliers={suppliers}
            onCreate={createPurchaseOrder}
            onStatus={updatePurchaseStatus}
            onMilestone={updateProductionMilestone}
          />
        )}

        {view === "costs" && (
          <CostEstimateCenter
            estimates={costEstimates}
            cases={businessCases}
            purchaseOrders={purchaseOrders}
            onSave={saveCostEstimate}
            onArchive={archiveCostEstimate}
          />
        )}

        {view === "documents" && (
          <DocumentCenter
            companyRegistry={companyRegistry}
            documents={documents}
            cases={businessCases}
            onCreate={createDocument}
            onConvert={convertDocument}
            onSave={saveDocument}
            onIssue={issueDocument}
            onVoid={voidDocument}
            onNewVersion={createDocumentVersion}
            onExportPdf={exportDocumentPdf}
            onExportCsv={exportDocumentCsv}
            onPrint={printDocument}
            onOpenPdf={documentApi.openPdf}
          />
        )}

        {view === "logistics" && (
          <LogisticsCenter
            cases={businessCases}
            partners={partners}
            shipments={shipmentBatches}
            payments={paymentPlans}
            onSavePartner={savePartner}
            onArchivePartner={archivePartner}
            onSaveShipment={saveShipment}
            onSavePayment={savePayment}
          />
        )}

        {view === "settings" && companyRegistry && (
          <CompanySettings registry={companyRegistry} onSave={saveCompanyRegistry} />
        )}
        {view === "security" && (
          <DataSecurityCenter recoveryReady={workspace.recoveryReady} onRecoveryKey={setRecoveryKeyNotice} />
        )}
      </main>
      {editorOpen && (masterTab === "products" || masterTab === "customers" || masterTab === "suppliers") && <MasterEditor tab={masterTab} record={editingRecord} saving={saving} onClose={() => setEditorOpen(false)} onSave={saveMaster} />}
      {recoveryKeyNotice && <RecoveryKeyNotice recoveryKey={recoveryKeyNotice} onClose={() => setRecoveryKeyNotice("")} />}
    </div>
  );
}
