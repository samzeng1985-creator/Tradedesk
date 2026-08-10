import { useMemo, useState } from "react";
import type { FormEvent } from "react";
import type {
  BusinessCase,
  BusinessCaseInput,
  Customer,
  PipelineStage,
  Product,
} from "./domain";

interface BusinessCaseCenterProps {
  cases: BusinessCase[];
  customers: Customer[];
  products: Product[];
  onSave: (input: BusinessCaseInput) => Promise<void>;
  onArchive: (id: string) => Promise<void>;
}

interface DraftLine {
  id?: string;
  productId: string;
  quantity: string;
  unitPrice: string;
}

const stageLabels: Record<PipelineStage, string> = {
  quotation: "报价中",
  order: "已确认",
  purchase: "采购中",
  production: "生产中",
  shipment: "待发货",
  documents: "制单中",
};

function formatMoney(minor: number, currency: string) {
  try {
    return new Intl.NumberFormat("zh-CN", {
      style: "currency",
      currency: currency || "USD",
    }).format(minor / 100);
  } catch {
    return `${currency || "USD"} ${(minor / 100).toFixed(2)}`;
  }
}

function nextNumber(cases: BusinessCase[]) {
  const year = new Date().getFullYear();
  return `TD-${year}-${String(cases.length + 1).padStart(4, "0")}`;
}

function CaseEditor({
  record,
  cases,
  customers,
  products,
  onClose,
  onSave,
}: {
  record: BusinessCase | null;
  cases: BusinessCase[];
  customers: Customer[];
  products: Product[];
  onClose: () => void;
  onSave: (input: BusinessCaseInput) => Promise<void>;
}) {
  const [number, setNumber] = useState(record?.number ?? nextNumber(cases));
  const [customerId, setCustomerId] = useState(record?.customerId ?? "");
  const [stage, setStage] = useState<PipelineStage>(record?.stage ?? "quotation");
  const [currency, setCurrency] = useState(record?.currency ?? "USD");
  const [incoterm, setIncoterm] = useState(record?.incoterm ?? "FOB");
  const [paymentTerms, setPaymentTerms] = useState(record?.paymentTerms ?? "");
  const [shipmentDate, setShipmentDate] = useState(record?.shipmentDate ?? "");
  const [notes, setNotes] = useState(record?.notes ?? "");
  const [lines, setLines] = useState<DraftLine[]>(
    record?.lines.map((line) => ({
      id: line.id,
      productId: line.productId,
      quantity: String(line.quantity),
      unitPrice: (line.unitPriceMinor / 100).toFixed(2),
    })) ?? [{ productId: "", quantity: "1", unitPrice: "0.00" }],
  );
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");

  const totalMinor = useMemo(
    () => lines.reduce((total, line) => {
      const amount = Number(line.quantity) * Math.round(Number(line.unitPrice) * 100);
      return total + (Number.isFinite(amount) ? Math.round(amount) : 0);
    }, 0),
    [lines],
  );

  function updateLine(index: number, patch: Partial<DraftLine>) {
    setLines((current) => current.map((line, currentIndex) =>
      currentIndex === index ? { ...line, ...patch } : line,
    ));
  }

  function selectCustomer(id: string) {
    setCustomerId(id);
    const customer = customers.find((item) => item.id === id);
    if (customer) {
      setCurrency(customer.currency);
      setPaymentTerms(customer.paymentTerms);
    }
  }

  async function submit(event: FormEvent) {
    event.preventDefault();
    if (lines.some((line) => !line.productId || Number(line.quantity) <= 0 || Number(line.unitPrice) < 0)) {
      setError("请完整填写产品、数量和单价。");
      return;
    }
    setSaving(true);
    setError("");
    try {
      await onSave({
        id: record?.id,
        number,
        customerId,
        stage,
        currency,
        incoterm,
        paymentTerms,
        shipmentDate,
        notes,
        lines: lines.map((line) => ({
          id: line.id,
          productId: line.productId,
          quantity: Number(line.quantity),
          unitPriceMinor: Math.round(Number(line.unitPrice) * 100),
        })),
      });
      onClose();
    } catch (reason) {
      setError(String(reason));
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="modal-backdrop" role="presentation" onMouseDown={onClose}>
      <section className="modal-card case-editor" role="dialog" aria-modal="true" aria-labelledby="case-editor-title" onMouseDown={(event) => event.stopPropagation()}>
        <div className="panel-heading">
          <div><span className="eyebrow">业务单</span><h2 id="case-editor-title">{record ? "编辑业务单" : "新建业务单"}</h2></div>
          <button className="icon-button" onClick={onClose} aria-label="关闭">×</button>
        </div>
        <form onSubmit={submit}>
          <div className="editor-form case-fields">
            <label>业务单号 *<input required value={number} onChange={(event) => setNumber(event.target.value)} autoFocus /></label>
            <label>客户 *<select required value={customerId} onChange={(event) => selectCustomer(event.target.value)}><option value="">请选择客户</option>{customers.map((customer) => <option key={customer.id} value={customer.id}>{customer.code} · {customer.legalName}</option>)}</select></label>
            <label>状态<select value={stage} onChange={(event) => setStage(event.target.value as PipelineStage)}>{Object.entries(stageLabels).map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label>
            <label>币种 *<input required maxLength={3} value={currency} onChange={(event) => setCurrency(event.target.value.toUpperCase())} /></label>
            <label>贸易术语<input value={incoterm} onChange={(event) => setIncoterm(event.target.value.toUpperCase())} placeholder="FOB / CIF / EXW" /></label>
            <label>计划发货日<input type="date" value={shipmentDate} onChange={(event) => setShipmentDate(event.target.value)} /></label>
            <label className="field-wide">付款条款<input value={paymentTerms} onChange={(event) => setPaymentTerms(event.target.value)} /></label>
          </div>

          <div className="line-editor-heading"><div><h3>订单产品</h3><p>保存时固化产品名称、SKU 和单位快照</p></div><button type="button" className="button button-secondary" onClick={() => setLines((current) => [...current, { productId: "", quantity: "1", unitPrice: "0.00" }])}>添加产品</button></div>
          <div className="line-editor">
            {lines.map((line, index) => {
              const product = products.find((item) => item.id === line.productId);
              const amount = Math.round(Number(line.quantity) * Math.round(Number(line.unitPrice) * 100));
              return <div className="line-row" key={line.id ?? `${index}-${record?.id ?? "new"}`}>
                <label>产品<select required value={line.productId} onChange={(event) => updateLine(index, { productId: event.target.value })}><option value="">请选择产品</option>{products.map((item) => <option value={item.id} key={item.id}>{item.sku} · {item.nameEn}</option>)}</select></label>
                <label>数量<input required type="number" min="0.001" step="0.001" value={line.quantity} onChange={(event) => updateLine(index, { quantity: event.target.value })} /></label>
                <label>单位<input value={product?.unit ?? "—"} readOnly /></label>
                <label>单价<input required type="number" min="0" step="0.01" value={line.unitPrice} onChange={(event) => updateLine(index, { unitPrice: event.target.value })} /></label>
                <div className="line-amount"><span>金额</span><strong>{formatMoney(Number.isFinite(amount) ? amount : 0, currency)}</strong></div>
                <button type="button" className="remove-line" disabled={lines.length === 1} onClick={() => setLines((current) => current.filter((_, currentIndex) => currentIndex !== index))}>移除</button>
              </div>;
            })}
          </div>
          <label className="case-notes">备注<input value={notes} onChange={(event) => setNotes(event.target.value)} /></label>
          {error && <div className="form-error" role="alert">{error}</div>}
          <div className="case-editor-footer"><div><span>订单总额</span><strong>{formatMoney(totalMinor, currency)}</strong></div><div className="modal-actions"><button type="button" className="button button-secondary" onClick={onClose}>取消</button><button className="button button-primary" disabled={saving}>{saving ? "保存中…" : "保存业务单"}</button></div></div>
        </form>
      </section>
    </div>
  );
}

export function BusinessCaseCenter({ cases, customers, products, onSave, onArchive }: BusinessCaseCenterProps) {
  const [editing, setEditing] = useState<BusinessCase | "new" | null>(null);
  const [query, setQuery] = useState("");
  const normalizedQuery = query.trim().toLocaleLowerCase();
  const filteredCases = cases.filter((item) =>
    [item.number, item.customerName, item.currency, item.incoterm].some((value) =>
      value.toLocaleLowerCase().includes(normalizedQuery),
    ),
  );

  async function archive(record: BusinessCase) {
    if (!window.confirm(`归档业务单“${record.number}”？历史订单行仍会保留。`)) return;
    await onArchive(record.id);
  }

  return <>
    <section className="panel business-case-panel">
      <div className="panel-heading"><div><h2>业务单中心</h2><p>客户和产品只选择一次，后续采购与单证复用同一业务快照</p></div><button className="button button-primary" disabled={!customers.length || !products.length} onClick={() => setEditing("new")}>新建业务单</button></div>
      {(!customers.length || !products.length) && <div className="empty-callout">请先在“主数据”中至少录入一个客户和一个产品。</div>}
      <div className="table-toolbar"><label><span className="sr-only">搜索业务单</span><input placeholder="按单号、客户、币种或贸易术语搜索" value={query} onChange={(event) => setQuery(event.target.value)} /></label><span className="record-count">{filteredCases.length} 条业务单</span></div>
      <div className="table-wrap"><table><thead><tr><th>业务单号</th><th>客户</th><th>状态</th><th>贸易术语</th><th>计划发货</th><th>金额</th><th>操作</th></tr></thead><tbody>{filteredCases.map((item) => <tr key={item.id}><td><strong>{item.number}</strong><small className="table-subtitle">{item.lines.length} 个产品行</small></td><td>{item.customerName}</td><td><span className="case-stage">{stageLabels[item.stage]}</span></td><td>{item.incoterm || "—"}</td><td>{item.shipmentDate || "—"}</td><td>{formatMoney(item.totalAmountMinor, item.currency)}</td><td><div className="row-actions"><button onClick={() => setEditing(item)}>编辑</button><button onClick={() => archive(item)}>归档</button></div></td></tr>)}</tbody></table></div>
      {!filteredCases.length && <div className="empty-table">{cases.length ? "没有符合条件的业务单" : "还没有业务单，请从第一笔真实订单开始"}</div>}
    </section>
    {editing && <CaseEditor record={editing === "new" ? null : editing} cases={cases} customers={customers} products={products} onClose={() => setEditing(null)} onSave={onSave} />}
  </>;
}
