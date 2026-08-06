import { useMemo, useState } from "react";
import {
  customers,
  initialCase,
  initialDocuments,
  initialMilestones,
  products,
  suppliers,
} from "./demo";
import type {
  PipelineStage,
  ProductionMilestone,
  RecordStatus,
  TradeDocument,
} from "./domain";

type View = "overview" | "masters" | "fulfillment" | "documents";
type MasterTab = "products" | "customers" | "suppliers";

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
  }).format(value);
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
  const [view, setView] = useState<View>("overview");
  const [masterTab, setMasterTab] = useState<MasterTab>("products");
  const [tradeCase, setTradeCase] = useState(initialCase);
  const [milestones, setMilestones] = useState(initialMilestones);
  const [documents, setDocuments] = useState(initialDocuments);
  const [masterQuery, setMasterQuery] = useState("");

  const margin = useMemo(
    () => Math.round(((tradeCase.salesAmount - tradeCase.purchaseAmount) / tradeCase.salesAmount) * 100),
    [tradeCase],
  );

  const normalizedQuery = masterQuery.trim().toLocaleLowerCase();
  const filteredProducts = products.filter((item) =>
    [item.sku, item.name, item.model, item.hsCode].some((value) =>
      value.toLocaleLowerCase().includes(normalizedQuery),
    ),
  );
  const filteredCustomers = customers.filter((item) =>
    [item.code, item.name, item.market, item.currency].some((value) =>
      value.toLocaleLowerCase().includes(normalizedQuery),
    ),
  );
  const filteredSuppliers = suppliers.filter((item) =>
    [item.code, item.name].some((value) =>
      value.toLocaleLowerCase().includes(normalizedQuery),
    ),
  );

  function advanceStage() {
    const index = pipeline.findIndex((item) => item.key === tradeCase.stage);
    const next = pipeline[Math.min(index + 1, pipeline.length - 1)];
    setTradeCase((current) => ({ ...current, stage: next.key }));
  }

  function completeMilestone(id: string) {
    setMilestones((current) =>
      current.map((item) =>
        item.id === id ? { ...item, progress: 100, status: "ready" as const } : item,
      ),
    );
  }

  function prepareDocuments() {
    setDocuments((current) =>
      current.map((item) =>
        item.status === "blocked"
          ? { ...item, status: "draft" as const, updatedAt: "刚刚生成" }
          : item,
      ),
    );
    setTradeCase((current) => ({ ...current, stage: "documents" }));
    setView("documents");
  }

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <span className="brand-mark">TD</span>
          <span>
            <strong>TradeDesk</strong>
            <small>Local</small>
          </span>
        </div>

        <nav aria-label="主导航">
          <button className={view === "overview" ? "selected" : ""} onClick={() => setView("overview")}>
            工作台
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
            <strong>本地工作区</strong>
            <small>加密边界已预留</small>
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
            <span className="demo-badge">演示工作区</span>
            <button className="button button-secondary">全局搜索</button>
          </div>
        </header>

        {view === "overview" && (
          <div className="page-stack">
            <section className="case-hero">
              <div className="case-heading">
                <div>
                  <span className="eyebrow">当前业务单</span>
                  <h2>{tradeCase.number}</h2>
                  <p>{tradeCase.customer.name} · 计划发货 {tradeCase.shipmentDate}</p>
                </div>
                <button className="button button-primary" onClick={advanceStage}>
                  推进下一节点
                </button>
              </div>
              <Pipeline stage={tradeCase.stage} />
            </section>

            <section className="metric-grid" aria-label="业务指标">
              <article>
                <span>销售金额</span>
                <strong>{money(tradeCase.salesAmount, tradeCase.currency)}</strong>
                <small>合同已确认</small>
              </article>
              <article>
                <span>采购成本</span>
                <strong>{money(tradeCase.purchaseAmount, tradeCase.currency)}</strong>
                <small>预计毛利率 {margin}%</small>
              </article>
              <article>
                <span>生产进度</span>
                <strong>{tradeCase.productionProgress}%</strong>
                <small>2 个供应商</small>
              </article>
              <article>
                <span>单证状态</span>
                <strong>{documents.filter((item) => item.status === "ready").length}/{documents.length}</strong>
                <small>2 份待完善</small>
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
                  {milestones.map((item) => (
                    <div className="milestone" key={item.id}>
                      <div className="milestone-row">
                        <div>
                          <strong>{item.label}</strong>
                          <span>{item.owner}</span>
                        </div>
                        <Status status={item.status} />
                      </div>
                      <div className="progress-track" aria-label={`${item.label} ${item.progress}%`}>
                        <span style={{ width: `${item.progress}%` }} />
                      </div>
                      <div className="milestone-meta">
                        <span>{item.progress}%</span>
                        <span>计划 {item.plannedDate}</span>
                      </div>
                    </div>
                  ))}
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
                  <article className="issue issue-warning">
                    <span>交付风险</span>
                    <strong>密封圈生产低于计划 8%</strong>
                    <p>建议今天向供应商确认可恢复日期。</p>
                  </article>
                  <article className="issue">
                    <span>单证资料</span>
                    <strong>详细装箱单等待箱规</strong>
                    <p>生产包装完成后可从产品和批次自动生成。</p>
                  </article>
                  <button className="button button-primary button-wide" onClick={prepareDocuments}>
                    生成待制单证
                  </button>
                </div>
              </section>
            </div>
          </div>
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
              <button className="button button-primary">新建记录</button>
            </div>

            <div className="table-wrap">
              {masterTab === "products" && (
                <table>
                  <thead>
                    <tr><th>SKU</th><th>产品</th><th>型号</th><th>HS 编码</th><th>单位</th><th>毛重</th></tr>
                  </thead>
                  <tbody>
                    {filteredProducts.map((item) => (
                      <tr key={item.id}><td>{item.sku}</td><td>{item.name}</td><td>{item.model}</td><td>{item.hsCode}</td><td>{item.unit}</td><td>{item.grossWeightKg} kg</td></tr>
                    ))}
                  </tbody>
                </table>
              )}
              {masterTab === "customers" && (
                <table>
                  <thead>
                    <tr><th>客户编码</th><th>客户</th><th>市场</th><th>币种</th><th>默认付款条款</th></tr>
                  </thead>
                  <tbody>
                    {filteredCustomers.map((item) => (
                      <tr key={item.id}><td>{item.code}</td><td>{item.name}</td><td>{item.market}</td><td>{item.currency}</td><td>{item.paymentTerms}</td></tr>
                    ))}
                  </tbody>
                </table>
              )}
              {masterTab === "suppliers" && (
                <table>
                  <thead>
                    <tr><th>供应商编码</th><th>供应商</th><th>默认交期</th><th>准时率</th><th>状态</th></tr>
                  </thead>
                  <tbody>
                    {filteredSuppliers.map((item) => (
                      <tr key={item.id}><td>{item.code}</td><td>{item.name}</td><td>{item.leadTimeDays} 天</td><td>{item.onTimeRate}%</td><td><Status status={item.status} /></td></tr>
                    ))}
                  </tbody>
                </table>
              )}
            </div>
          </section>
        )}

        {view === "fulfillment" && (
          <div className="page-stack">
            <section className="panel">
              <div className="panel-heading">
                <div>
                  <span className="eyebrow">采购单 PO-2026-0001</span>
                  <h2>生产执行</h2>
                  <p>业务单 {tradeCase.number} · 可发货前必须完成质检与包装</p>
                </div>
                <Status status="working" />
              </div>
              <div className="milestone-list fulfillment-list">
                {milestones.map((item: ProductionMilestone) => (
                  <div className="milestone" key={item.id}>
                    <div className="milestone-row">
                      <div>
                        <strong>{item.label}</strong>
                        <span>{item.owner} · 计划 {item.plannedDate}</span>
                      </div>
                      {item.progress < 100 ? (
                        <button className="button button-secondary" onClick={() => completeMilestone(item.id)}>
                          标记完成
                        </button>
                      ) : <Status status="ready" />}
                    </div>
                    <div className="progress-track"><span style={{ width: `${item.progress}%` }} /></div>
                  </div>
                ))}
              </div>
            </section>
          </div>
        )}

        {view === "documents" && (
          <section className="panel">
            <div className="panel-heading">
              <div>
                <h2>业务单证包</h2>
                <p>{tradeCase.number} · 同一份业务快照生成，减少重复录入</p>
              </div>
              <button className="button button-primary">新建单证</button>
            </div>
            <div className="document-grid">
              {documents.map((item: TradeDocument) => (
                <article key={item.id}>
                  <div className="document-topline"><span>{item.type}</span><Status status={item.status} /></div>
                  <strong>{item.number}</strong>
                  <small>{item.updatedAt}</small>
                  <button className="button button-secondary">打开</button>
                </article>
              ))}
            </div>
          </section>
        )}
      </main>
    </div>
  );
}
