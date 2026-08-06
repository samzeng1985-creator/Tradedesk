import { useEffect, useMemo, useState } from "react";
import { businessCaseApi, documentApi, fulfillmentApi, masterApi, workspaceApi } from "./api";
import { BusinessCaseCenter } from "./BusinessCaseCenter";
import { DocumentCenter } from "./DocumentCenter";
import { FulfillmentCenter } from "./FulfillmentCenter";
import { MasterEditor } from "./MasterEditor";
import type { MasterInput, MasterRecord, MasterTab } from "./MasterEditor";
import { UnlockScreen } from "./UnlockScreen";
import type {
  BusinessCase,
  BusinessCaseInput,
  CreateDocumentInput,
  Customer,
  PipelineStage,
  Product,
  ProductionMilestoneInput,
  PurchaseOrder,
  PurchaseOrderInput,
  PurchaseStatus,
  RecordStatus,
  SaveDocumentInput,
  Supplier,
  TradeDocument,
  WorkspaceSummary,
} from "./domain";

type View = "overview" | "cases" | "masters" | "fulfillment" | "documents";

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
  fulfillment: { title: "采购与生产", subtitle: "只跟踪关键里程碑，不做复杂排产" },
  documents: { title: "单证中心", subtitle: "版本、状态与跨单证一致性" },
};

const statusText: Record<RecordStatus, string> = {
  ready: "已完成",
  working: "进行中",
  blocked: "待处理",
  draft: "草稿",
};

function money(value: number, currency: string) {
  return new Intl.NumberFormat("zh-CN", {
    style: "currency",
    currency,
    maximumFractionDigits: 0,
  }).format(value / 100);
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
  const [workspaceExists, setWorkspaceExists] = useState(false);
  const [workspaceChecking, setWorkspaceChecking] = useState(true);
  const [view, setView] = useState<View>("overview");
  const [masterTab, setMasterTab] = useState<MasterTab>("products");
  const [products, setProducts] = useState<Product[]>([]);
  const [customers, setCustomers] = useState<Customer[]>([]);
  const [suppliers, setSuppliers] = useState<Supplier[]>([]);
  const [businessCases, setBusinessCases] = useState<BusinessCase[]>([]);
  const [purchaseOrders, setPurchaseOrders] = useState<PurchaseOrder[]>([]);
  const [documents, setDocuments] = useState<TradeDocument[]>([]);
  const [masterQuery, setMasterQuery] = useState("");
  const [editorOpen, setEditorOpen] = useState(false);
  const [editingRecord, setEditingRecord] = useState<MasterRecord | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    workspaceApi.exists()
      .then(setWorkspaceExists)
      .finally(() => setWorkspaceChecking(false));
  }, []);

  async function loadMasterData() {
    const [nextProducts, nextCustomers, nextSuppliers, nextCases, nextOrders, nextDocuments, summary] = await Promise.all([
      masterApi.listProducts(),
      masterApi.listCustomers(),
      masterApi.listSuppliers(),
      businessCaseApi.list(),
      fulfillmentApi.list(),
      documentApi.list(),
      workspaceApi.summary(),
    ]);
    setProducts(nextProducts);
    setCustomers(nextCustomers);
    setSuppliers(nextSuppliers);
    setBusinessCases(nextCases);
    setPurchaseOrders(nextOrders);
    setDocuments(nextDocuments);
    setWorkspace(summary);
  }

  async function unlock(password: string, companyName?: string) {
    await workspaceApi.unlock(password, companyName);
    await loadMasterData();
    setWorkspaceExists(true);
  }

  async function lock() {
    await workspaceApi.lock();
    setWorkspace(null);
    setWorkspaceExists(true);
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

  async function archiveMaster(entity: "product" | "customer" | "supplier", record: MasterRecord) {
    if (!window.confirm(`停用“${"sku" in record ? record.sku : record.legalName}”？历史资料不会被删除。`)) return;
    await masterApi.archive(entity, record.id);
    await loadMasterData();
  }

  async function saveBusinessCase(input: BusinessCaseInput) {
    await businessCaseApi.save(input);
    await loadMasterData();
  }

  async function archiveBusinessCase(id: string) {
    await businessCaseApi.archive(id);
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

  async function createDocument(input: CreateDocumentInput) {
    const document = await documentApi.create(input);
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

  async function exportDocumentPdf(id: string) {
    const result = await documentApi.exportPdf(id);
    await loadMasterData();
    return result.path;
  }

  async function exportDocumentCsv(id: string) {
    return documentApi.exportCsv(id);
  }

  async function printDocument(id: string) {
    const result = await documentApi.print(id);
    await loadMasterData();
    return result.path;
  }

  const currentCase = businessCases[0] ?? null;
  const currentOrders = currentCase
    ? purchaseOrders.filter((order) => order.businessCaseId === currentCase.id && order.status !== "cancelled")
    : [];
  const currentMilestones = currentOrders.flatMap((order) => order.lines.flatMap((line) =>
    line.milestones.map((milestone) => ({ ...milestone, supplierName: order.supplierName, sku: line.sku })),
  ));
  const purchaseTotalMinor = currentOrders.reduce((sum, order) => sum + order.totalAmountMinor, 0);
  const productionProgress = currentMilestones.length
    ? Math.round(currentMilestones.reduce((sum, milestone) => sum + milestone.progress, 0) / currentMilestones.length)
    : 0;

  const margin = useMemo(() => {
    if (!currentCase?.totalAmountMinor) return 0;
    return Math.round(((currentCase.totalAmountMinor - purchaseTotalMinor) / currentCase.totalAmountMinor) * 100);
  }, [currentCase, purchaseTotalMinor]);

  const normalizedQuery = masterQuery.trim().toLocaleLowerCase();
  const filteredProducts = products.filter((item) =>
    [item.sku, item.nameZh, item.nameEn, item.model, item.hsCode].some((value) =>
      value.toLocaleLowerCase().includes(normalizedQuery),
    ),
  );
  const filteredCustomers = customers.filter((item) =>
    [item.code, item.legalName, item.market, item.currency].some((value) =>
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
    return <UnlockScreen checking={workspaceChecking} existing={workspaceExists} onUnlock={unlock} />;
  }

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <span className="brand-mark">TD</span>
          <span>
            <strong>TradeDesk</strong>
            <small>Local · 0.5.0</small>
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
          <button
            className={view === "fulfillment" ? "selected" : ""}
            onClick={() => setView("fulfillment")}
          >
            采购与生产
          </button>
          <button className={view === "documents" ? "selected" : ""} onClick={() => setView("documents")}>
            单证中心
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
                    <span className="eyebrow">最近业务单</span>
                    <h2>{currentCase.number}</h2>
                    <p>{currentCase.customerName} · 计划发货 {currentCase.shipmentDate || "未设置"}</p>
                  </div>
                  <button className="button button-primary" onClick={() => setView("cases")}>打开业务单</button>
                </div>
                <Pipeline stage={currentCase.stage} />
              </> : <div className="empty-overview"><span className="eyebrow">开始第一笔业务</span><h2>建立业务单后即可跟踪采购与生产</h2><button className="button button-primary" onClick={() => setView("cases")}>前往业务单</button></div>}
            </section>

            <section className="metric-grid" aria-label="业务指标">
              <article>
                <span>销售金额</span>
                <strong>{money(currentCase?.totalAmountMinor ?? 0, currentCase?.currency ?? "USD")}</strong>
                <small>{currentCase ? currentCase.number : "暂无业务单"}</small>
              </article>
              <article>
                <span>采购成本</span>
                <strong>{money(purchaseTotalMinor, currentCase?.currency ?? "USD")}</strong>
                <small>{currentOrders.length ? `预计毛利率 ${margin}%` : "尚未下推采购"}</small>
              </article>
              <article>
                <span>生产进度</span>
                <strong>{productionProgress}%</strong>
                <small>{new Set(currentOrders.map((order) => order.supplierId)).size} 个供应商</small>
              </article>
              <article>
                <span>生产风险</span>
                <strong>{workspace.productionRisks}</strong>
                <small>{workspace.productionRisks ? "存在阻断节点" : "暂无异常节点"}</small>
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
                  {currentMilestones.filter((item) => item.status === "blocked").slice(0, 2).map((item) => <article className="issue issue-warning" key={item.id}><span>生产异常</span><strong>{item.sku} · {item.label}</strong><p>{item.issue || "请向供应商确认恢复日期。"}</p></article>)}
                  <article className="issue">
                    <span>采购覆盖</span>
                    <strong>{currentOrders.length ? `${currentOrders.length} 张采购单正在执行` : "业务单尚未下推采购"}</strong>
                    <p>{currentOrders.length ? `当前可发货数量 ${currentOrders.reduce((sum, order) => sum + order.readyQuantity, 0)}` : "按供应商拆分产品行后即可开始生产跟踪。"}</p>
                  </article>
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
            onSave={saveBusinessCase}
            onArchive={archiveBusinessCase}
          />
        )}

        {view === "masters" && (
          <section className="panel master-panel">
            <div className="tabs" role="tablist" aria-label="主数据类型">
              <button role="tab" aria-selected={masterTab === "products"} onClick={() => setMasterTab("products")}>
                产品 {products.length}
              </button>
              <button role="tab" aria-selected={masterTab === "customers"} onClick={() => setMasterTab("customers")}>
                客户 {customers.length}
              </button>
              <button role="tab" aria-selected={masterTab === "suppliers"} onClick={() => setMasterTab("suppliers")}>
                供应商 {suppliers.length}
              </button>
            </div>

            <div className="table-toolbar">
              <label>
                <span className="sr-only">搜索主数据</span>
                <input
                  placeholder="按编号、名称或 HS 编码搜索"
                  value={masterQuery}
                  onChange={(event) => setMasterQuery(event.target.value)}
                />
              </label>
              <button className="button button-primary" onClick={() => openEditor()}>新建记录</button>
            </div>

            <div className="table-wrap">
              {masterTab === "products" && (
                <table>
                  <thead>
                    <tr><th>SKU</th><th>产品</th><th>型号</th><th>HS 编码</th><th>单位</th><th>毛重</th><th>操作</th></tr>
                  </thead>
                  <tbody>
                    {filteredProducts.map((item) => (
                      <tr key={item.id}><td>{item.sku}</td><td><strong>{item.nameEn}</strong><small className="table-subtitle">{item.nameZh}</small></td><td>{item.model || "—"}</td><td>{item.hsCode || "—"}</td><td>{item.unit}</td><td>{item.grossWeightKg} kg</td><td className="row-actions"><button onClick={() => openEditor(item)}>编辑</button><button onClick={() => archiveMaster("product", item)}>停用</button></td></tr>
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
                      <tr key={item.id}><td>{item.code}</td><td>{item.legalName}</td><td>{item.market || "—"}</td><td>{item.currency}</td><td>{item.paymentTerms || "—"}</td><td className="row-actions"><button onClick={() => openEditor(item)}>编辑</button><button onClick={() => archiveMaster("customer", item)}>停用</button></td></tr>
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
                      <tr key={item.id}><td>{item.code}</td><td>{item.legalName}</td><td>{item.leadTimeDays} 天</td><td>{item.onTimeRate}%</td><td><Status status="ready" /></td><td className="row-actions"><button onClick={() => openEditor(item)}>编辑</button><button onClick={() => archiveMaster("supplier", item)}>停用</button></td></tr>
                    ))}
                  </tbody>
                </table>
              )}
            </div>
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

        {view === "documents" && (
          <DocumentCenter
            documents={documents}
            cases={businessCases}
            onCreate={createDocument}
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
      </main>
      {editorOpen && <MasterEditor tab={masterTab} record={editingRecord} saving={saving} onClose={() => setEditorOpen(false)} onSave={saveMaster} />}
    </div>
  );
}
