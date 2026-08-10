import { useMemo, useState } from "react";
import type { FormEvent } from "react";
import type {
  BusinessCase,
  MilestoneStatus,
  ProductionMilestone,
  ProductionMilestoneInput,
  PurchaseOrder,
  PurchaseOrderInput,
  PurchaseStatus,
  Supplier,
} from "./domain";
import { CurrencySelect, formatMoney } from "./currencies";

interface FulfillmentCenterProps {
  orders: PurchaseOrder[];
  cases: BusinessCase[];
  suppliers: Supplier[];
  onCreate: (input: PurchaseOrderInput) => Promise<void>;
  onStatus: (id: string, status: PurchaseStatus) => Promise<void>;
  onMilestone: (input: ProductionMilestoneInput) => Promise<void>;
}

const purchaseStatusLabels: Record<PurchaseStatus, string> = {
  draft: "草稿",
  pending_confirmation: "待确认",
  confirmed: "已确认",
  in_production: "生产中",
  ready_to_ship: "可发货",
  completed: "已完成",
  cancelled: "已取消",
};

const milestoneStatusLabels: Record<MilestoneStatus, string> = {
  pending: "未开始",
  in_progress: "进行中",
  completed: "已完成",
  blocked: "异常",
};

function nextNumber(orders: PurchaseOrder[]) {
  const year = new Date().getFullYear();
  const maximum = orders.reduce((current, order) => {
    const match = order.number.match(new RegExp(`^PO-${year}-(\\d+)$`));
    return Math.max(current, match ? Number(match[1]) : 0);
  }, 0);
  return `PO-${year}-${String(maximum + 1).padStart(4, "0")}`;
}

interface DraftPurchaseLine {
  sourceCaseLineId: string;
  sku: string;
  name: string;
  unit: string;
  available: number;
  selected: boolean;
  quantity: string;
  unitCost: string;
}

function PurchaseEditor({ orders, cases, suppliers, onClose, onCreate }: {
  orders: PurchaseOrder[];
  cases: BusinessCase[];
  suppliers: Supplier[];
  onClose: () => void;
  onCreate: (input: PurchaseOrderInput) => Promise<void>;
}) {
  const [number, setNumber] = useState(nextNumber(orders));
  const [caseId, setCaseId] = useState("");
  const [supplierId, setSupplierId] = useState("");
  const [currency, setCurrency] = useState("USD");
  const [expectedDate, setExpectedDate] = useState("");
  const [notes, setNotes] = useState("");
  const [lines, setLines] = useState<DraftPurchaseLine[]>([]);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");

  const allocated = useMemo(() => {
    const result = new Map<string, number>();
    orders.filter((order) => order.status !== "cancelled").forEach((order) => {
      order.lines.forEach((line) => {
        result.set(line.sourceCaseLineId, (result.get(line.sourceCaseLineId) ?? 0) + line.quantity);
      });
    });
    return result;
  }, [orders]);

  function selectCase(id: string) {
    setCaseId(id);
    const selected = cases.find((item) => item.id === id);
    if (!selected) {
      setLines([]);
      return;
    }
    setCurrency(selected.currency);
    setExpectedDate(selected.shipmentDate);
    setLines(selected.lines.map((line) => {
      const available = Math.max(0, line.quantity - (allocated.get(line.id) ?? 0));
      return {
        sourceCaseLineId: line.id,
        sku: line.sku,
        name: line.nameZh || line.nameEn,
        unit: line.unit,
        available,
        selected: available > 0,
        quantity: String(available),
        unitCost: "0.00",
      };
    }));
  }

  function updateLine(index: number, patch: Partial<DraftPurchaseLine>) {
    setLines((current) => current.map((line, currentIndex) =>
      currentIndex === index ? { ...line, ...patch } : line,
    ));
  }

  const totalMinor = lines.reduce((total, line) => {
    if (!line.selected) return total;
    const amount = Number(line.quantity) * Math.round(Number(line.unitCost) * 100);
    return total + (Number.isFinite(amount) ? Math.round(amount) : 0);
  }, 0);

  async function submit(event: FormEvent) {
    event.preventDefault();
    const selectedLines = lines.filter((line) => line.selected);
    if (!selectedLines.length) {
      setError("请至少选择一个仍有未采购数量的业务单产品行。");
      return;
    }
    if (selectedLines.some((line) => Number(line.quantity) <= 0
      || Number(line.quantity) > line.available
      || Number(line.unitCost) < 0)) {
      setError("采购数量必须大于 0 且不能超过业务单未分配数量。");
      return;
    }
    setSaving(true);
    setError("");
    try {
      await onCreate({
        number,
        businessCaseId: caseId,
        supplierId,
        currency,
        expectedDate,
        notes,
        lines: selectedLines.map((line) => ({
          sourceCaseLineId: line.sourceCaseLineId,
          quantity: Number(line.quantity),
          unitCostMinor: Math.round(Number(line.unitCost) * 100),
        })),
      });
      onClose();
    } catch (reason) {
      setError(String(reason));
    } finally {
      setSaving(false);
    }
  }

  return <div className="modal-backdrop" role="presentation" onMouseDown={onClose}>
    <section className="modal-card purchase-editor" role="dialog" aria-modal="true" aria-labelledby="purchase-editor-title" onMouseDown={(event) => event.stopPropagation()}>
      <div className="panel-heading"><div><span className="eyebrow">采购下推</span><h2 id="purchase-editor-title">新建采购单</h2></div><button className="icon-button" onClick={onClose} aria-label="关闭">×</button></div>
      <form onSubmit={submit}>
        <div className="editor-form">
          <label>采购单号 *<input required value={number} onChange={(event) => setNumber(event.target.value)} autoFocus /></label>
          <label>来源业务单 *<select required value={caseId} onChange={(event) => selectCase(event.target.value)}><option value="">请选择业务单</option>{cases.map((item) => <option value={item.id} key={item.id}>{item.number} · {item.customerName}</option>)}</select></label>
          <label>供应商 *<select required value={supplierId} onChange={(event) => setSupplierId(event.target.value)}><option value="">请选择供应商</option>{suppliers.map((item) => <option value={item.id} key={item.id}>{item.code} · {item.legalName}</option>)}</select></label>
          <label>采购币种 *<CurrencySelect value={currency} onChange={setCurrency} /></label>
          <label>预计交货日<input type="date" value={expectedDate} onChange={(event) => setExpectedDate(event.target.value)} /></label>
          <label>备注<input value={notes} onChange={(event) => setNotes(event.target.value)} /></label>
        </div>
        <div className="line-editor-heading"><div><h3>采购产品</h3><p>可按产品拆分给不同供应商，系统阻止超量采购</p></div></div>
        <div className="purchase-line-editor">
          {caseId && !lines.some((line) => line.available > 0) && <div className="empty-callout">该业务单所有产品数量都已分配采购。</div>}
          {lines.map((line, index) => <div className={`purchase-draft-line ${line.available <= 0 ? "unavailable" : ""}`} key={line.sourceCaseLineId}>
            <label className="line-check"><input type="checkbox" disabled={line.available <= 0} checked={line.selected} onChange={(event) => updateLine(index, { selected: event.target.checked })} /><span><strong>{line.sku}</strong><small>{line.name} · 未分配 {line.available} {line.unit}</small></span></label>
            <label>采购数量<input type="number" min="0.001" max={line.available} step="0.001" disabled={!line.selected} value={line.quantity} onChange={(event) => updateLine(index, { quantity: event.target.value })} /></label>
            <label>采购单价<input type="number" min="0" step="0.01" disabled={!line.selected} value={line.unitCost} onChange={(event) => updateLine(index, { unitCost: event.target.value })} /></label>
          </div>)}
        </div>
        {error && <div className="form-error" role="alert">{error}</div>}
        <div className="case-editor-footer"><div><span>采购总额</span><strong>{formatMoney(totalMinor, currency)}</strong></div><div className="modal-actions"><button type="button" className="button button-secondary" onClick={onClose}>取消</button><button className="button button-primary" disabled={saving}>{saving ? "创建中…" : "创建采购单"}</button></div></div>
      </form>
    </section>
  </div>;
}

function MilestoneEditor({ milestone, lineQuantity, onClose, onSave }: {
  milestone: ProductionMilestone;
  lineQuantity: number;
  onClose: () => void;
  onSave: (input: ProductionMilestoneInput) => Promise<void>;
}) {
  const [plannedDate, setPlannedDate] = useState(milestone.plannedDate);
  const [actualDate, setActualDate] = useState(milestone.actualDate);
  const [progress, setProgress] = useState(String(milestone.progress));
  const [completedQuantity, setCompletedQuantity] = useState(String(milestone.completedQuantity));
  const [status, setStatus] = useState(milestone.status);
  const [issue, setIssue] = useState(milestone.issue);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");

  async function submit(event: FormEvent) {
    event.preventDefault();
    const quantity = Number(completedQuantity);
    if (quantity < 0 || quantity > lineQuantity || Number(progress) < 0 || Number(progress) > 100) {
      setError("进度需为 0–100，完成数量不能超过采购数量。");
      return;
    }
    setSaving(true);
    setError("");
    try {
      await onSave({
        id: milestone.id,
        plannedDate,
        actualDate,
        progress: status === "completed" ? 100 : Number(progress),
        completedQuantity: quantity,
        status,
        issue,
      });
      onClose();
    } catch (reason) {
      setError(String(reason));
    } finally {
      setSaving(false);
    }
  }

  return <div className="modal-backdrop" role="presentation" onMouseDown={onClose}>
    <section className="modal-card milestone-editor-card" role="dialog" aria-modal="true" aria-labelledby="milestone-editor-title" onMouseDown={(event) => event.stopPropagation()}>
      <div className="panel-heading"><div><span className="eyebrow">生产节点</span><h2 id="milestone-editor-title">{milestone.label}</h2></div><button className="icon-button" onClick={onClose} aria-label="关闭">×</button></div>
      <form className="editor-form" onSubmit={submit}>
        <label>状态<select value={status} onChange={(event) => setStatus(event.target.value as MilestoneStatus)}>{Object.entries(milestoneStatusLabels).map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label>
        <label>进度 %<input type="number" min="0" max="100" value={status === "completed" ? "100" : progress} disabled={status === "completed"} onChange={(event) => setProgress(event.target.value)} /></label>
        <label>计划日期<input type="date" value={plannedDate} onChange={(event) => setPlannedDate(event.target.value)} /></label>
        <label>实际日期<input type="date" value={actualDate} onChange={(event) => setActualDate(event.target.value)} /></label>
        <label>完成数量<input type="number" min="0" max={lineQuantity} step="0.001" value={completedQuantity} onChange={(event) => setCompletedQuantity(event.target.value)} /></label>
        <label>采购数量<input value={lineQuantity} readOnly /></label>
        <label className="field-wide">异常或备注<input value={issue} onChange={(event) => setIssue(event.target.value)} placeholder="延期、缺料、返工、质检异常…" /></label>
        {error && <div className="form-error field-wide" role="alert">{error}</div>}
        <div className="modal-actions field-wide"><button type="button" className="button button-secondary" onClick={onClose}>取消</button><button className="button button-primary" disabled={saving}>{saving ? "保存中…" : "保存节点"}</button></div>
      </form>
    </section>
  </div>;
}

export function FulfillmentCenter({ orders, cases, suppliers, onCreate, onStatus, onMilestone }: FulfillmentCenterProps) {
  const [creating, setCreating] = useState(false);
  const [editingMilestone, setEditingMilestone] = useState<{ milestone: ProductionMilestone; quantity: number } | null>(null);
  const [query, setQuery] = useState("");
  const normalizedQuery = query.trim().toLocaleLowerCase();
  const filtered = orders.filter((order) => [order.number, order.businessCaseNumber, order.supplierName]
    .some((value) => value.toLocaleLowerCase().includes(normalizedQuery)));

  async function changeStatus(id: string, status: PurchaseStatus) {
    try {
      await onStatus(id, status);
    } catch (reason) {
      window.alert(String(reason));
    }
  }

  return <>
    <section className="panel fulfillment-center">
      <div className="panel-heading"><div><h2>采购与生产中心</h2><p>业务单按供应商拆分采购，并跟踪影响发货的六个关键节点</p></div><button className="button button-primary" disabled={!cases.length || !suppliers.length} onClick={() => setCreating(true)}>新建采购单</button></div>
      {(!cases.length || !suppliers.length) && <div className="empty-callout">请先录入业务单和供应商，再创建采购单。</div>}
      <div className="table-toolbar"><label><span className="sr-only">搜索采购单</span><input placeholder="按采购单号、业务单或供应商搜索" value={query} onChange={(event) => setQuery(event.target.value)} /></label><span className="record-count">{filtered.length} 张采购单</span></div>
      <div className="purchase-order-list">
        {filtered.map((order) => {
          const totalQuantity = order.lines.reduce((sum, line) => sum + line.quantity, 0);
          return <article className="purchase-order-card" key={order.id}>
            <header className="purchase-order-header">
              <div><span className="eyebrow">{order.businessCaseNumber}</span><h3>{order.number}</h3><p>{order.supplierName} · 预计交货 {order.expectedDate || "未设置"}</p></div>
              <div className="purchase-order-actions"><strong>{formatMoney(order.totalAmountMinor, order.currency)}</strong><select aria-label={`${order.number}状态`} value={order.status} onChange={(event) => changeStatus(order.id, event.target.value as PurchaseStatus)}>{Object.entries(purchaseStatusLabels).map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></div>
            </header>
            <div className="purchase-summary"><span>采购 {totalQuantity}</span><span>生产完成 {order.completedQuantity}</span><span>可发货 {order.readyQuantity}</span><span>{order.lines.length} 个产品行</span></div>
            <div className="purchase-products">
              {order.lines.map((line) => <section className="purchase-product" key={line.id}>
                <div className="purchase-product-heading"><div><strong>{line.sku} · {line.nameZh || line.nameEn}</strong><span>{line.quantity} {line.unit} × {formatMoney(line.unitCostMinor, order.currency)}</span></div><strong>{formatMoney(line.amountMinor, order.currency)}</strong></div>
                <div className="milestone-grid">
                  {line.milestones.map((milestone) => <button className={`milestone-card milestone-${milestone.status}`} key={milestone.id} onClick={() => setEditingMilestone({ milestone, quantity: line.quantity })}>
                    <span>{milestone.label}</span><strong>{milestone.progress}%</strong><small>{milestoneStatusLabels[milestone.status]}{milestone.issue ? ` · ${milestone.issue}` : ""}</small>
                  </button>)}
                </div>
              </section>)}
            </div>
          </article>;
        })}
      </div>
      {!filtered.length && <div className="empty-table">{orders.length ? "没有符合条件的采购单" : "还没有采购单，可从已确认业务单开始拆分"}</div>}
    </section>
    {creating && <PurchaseEditor orders={orders} cases={cases} suppliers={suppliers} onClose={() => setCreating(false)} onCreate={onCreate} />}
    {editingMilestone && <MilestoneEditor milestone={editingMilestone.milestone} lineQuantity={editingMilestone.quantity} onClose={() => setEditingMilestone(null)} onSave={onMilestone} />}
  </>;
}
