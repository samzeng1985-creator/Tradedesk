import { useMemo, useState } from "react";
import { AttachmentPanel } from "./AttachmentPanel";
import { formatMoney } from "./currencies";
import type {
  BusinessCase,
  CostCategory,
  CostEstimate,
  CostEstimateInput,
  CostEstimateLineInput,
  PurchaseOrder,
} from "./domain";
import { nextDatedNumber } from "./numbering";

const categoryLabels: Record<CostCategory, string> = {
  material: "材料 / 产品",
  processing: "加工费",
  packaging: "包装费",
  domestic_logistics: "国内物流",
  international_freight: "国际运费",
  duty_tax: "税费 / 关税",
  commission: "佣金",
  insurance: "保险费",
  certification: "认证 / 检验",
  other: "其他",
};

type DraftLine = CostEstimateLineInput & { key: string };

function draftKey() {
  return globalThis.crypto?.randomUUID?.() ?? `${Date.now()}-${Math.random()}`;
}

function emptyLine(category: CostCategory = "material"): DraftLine {
  return {
    key: draftKey(), category, description: "", specification: "", quantity: 1,
    unit: "项", unitCostMinor: 0, notes: "",
  };
}

function CostEstimateEditor({ record, estimates, cases, purchaseOrders, onClose, onSave }: {
  record: CostEstimate | null;
  estimates: CostEstimate[];
  cases: BusinessCase[];
  purchaseOrders: PurchaseOrder[];
  onClose: () => void;
  onSave: (input: CostEstimateInput) => Promise<void>;
}) {
  const [number, setNumber] = useState(record?.number ?? nextDatedNumber(estimates.map((item) => item.number), "CST"));
  const [caseId, setCaseId] = useState(record?.businessCaseId ?? cases[0]?.id ?? "");
  const [targetMargin, setTargetMargin] = useState(String((record?.targetMarginBps ?? 2500) / 100));
  const [notes, setNotes] = useState(record?.notes ?? "");
  const [lines, setLines] = useState<DraftLine[]>(record?.lines.map((line) => ({
    key: line.id,
    id: line.id,
    category: line.category,
    description: line.description,
    specification: line.specification,
    quantity: line.quantity,
    unit: line.unit,
    unitCostMinor: line.unitCostMinor,
    notes: line.notes,
  })) ?? [emptyLine()]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const selectedCase = cases.find((item) => item.id === caseId);
  const totalMinor = lines.reduce((sum, line) => sum + Math.round(line.quantity * line.unitCostMinor), 0);
  const marginBps = Math.round(Number(targetMargin) * 100);
  const suggestedMinor = marginBps >= 0 && marginBps < 10_000
    ? Math.round(totalMinor / (1 - marginBps / 10_000))
    : 0;

  function patchLine(index: number, patch: Partial<DraftLine>) {
    setLines((current) => current.map((line, lineIndex) => lineIndex === index ? { ...line, ...patch } : line));
  }

  function importPurchases() {
    if (!selectedCase) return;
    const matching = purchaseOrders.filter((order) =>
      order.businessCaseId === selectedCase.id
      && order.status !== "cancelled"
      && order.currency === selectedCase.currency,
    );
    if (!matching.length) {
      setError(`没有可带入的 ${selectedCase.currency} 采购明细。异币采购需先折算后再录入。`);
      return;
    }
    if (lines.some((line) => line.description.trim()) && !window.confirm("用当前业务单的同币采购明细替换现有成本行？")) return;
    setLines(matching.flatMap((order) => order.lines.map((line) => ({
      key: draftKey(),
      category: "material" as CostCategory,
      description: `${line.sku} · ${line.nameZh || line.nameEn}`,
      specification: order.supplierName,
      quantity: line.quantity,
      unit: line.unit,
      unitCostMinor: line.unitCostMinor,
      notes: `采购单 ${order.number}`,
    }))));
    setError("");
  }

  async function submit(event: React.FormEvent) {
    event.preventDefault();
    if (!selectedCase) { setError("请选择业务单。"); return; }
    if (!Number.isFinite(marginBps) || marginBps < 0 || marginBps > 9500) {
      setError("目标毛利率需在 0% 至 95% 之间。"); return;
    }
    if (!lines.length || lines.some((line) => !line.description.trim() || !line.unit.trim() || line.quantity <= 0 || line.unitCostMinor < 0)) {
      setError("请完整填写每项成本的名称、数量、单位和单价。"); return;
    }
    setBusy(true); setError("");
    try {
      await onSave({
        id: record?.id,
        number,
        businessCaseId: selectedCase.id,
        targetMarginBps: marginBps,
        notes,
        lines: lines.map(({ key: _key, ...line }) => line),
      });
      onClose();
    } catch (reason) { setError(String(reason)); } finally { setBusy(false); }
  }

  return <div className="modal-backdrop" onMouseDown={onClose}>
    <section className="modal-card cost-estimate-editor" onMouseDown={(event) => event.stopPropagation()}>
      <div className="panel-heading"><div><span className="eyebrow">报价成本依据</span><h2>{record ? "编辑成本估算" : "新建成本估算"}</h2></div><button className="icon-button" onClick={onClose}>×</button></div>
      <form onSubmit={submit}>
        <div className="editor-form cost-estimate-fields">
          <label>估算编号 *<input required value={number} onChange={(event) => setNumber(event.target.value)} /></label>
          <label>业务单 *<select required disabled={!!record} value={caseId} onChange={(event) => setCaseId(event.target.value)}><option value="">请选择</option>{cases.map((item) => <option value={item.id} key={item.id}>{item.number} · {item.customerName}</option>)}</select></label>
          <label>估算币种<input readOnly value={selectedCase?.currency ?? record?.currency ?? "—"} /></label>
          <label>目标毛利率（%）<input required type="number" min="0" max="95" step="0.01" value={targetMargin} onChange={(event) => setTargetMargin(event.target.value)} /></label>
          <label className="field-wide">说明<input value={notes} onChange={(event) => setNotes(event.target.value)} placeholder="报价范围、汇率口径或未决事项" /></label>
        </div>
        <div className="cost-line-heading"><div><h3>成本明细</h3><p>所有金额均按业务单币种录入，避免不同币种直接相加</p></div><div><button type="button" className="button button-secondary" disabled={!selectedCase} onClick={importPurchases}>带入同币采购</button><button type="button" className="button button-secondary" onClick={() => setLines((current) => [...current, emptyLine("other")])}>添加成本项</button></div></div>
        <div className="table-wrap cost-line-table"><table><thead><tr><th>类别</th><th>成本项目</th><th>规格 / 供应商</th><th>数量</th><th>单位</th><th>单价</th><th>金额</th><th>备注</th><th>操作</th></tr></thead><tbody>{lines.map((line, index) => <tr key={line.key}>
          <td><select value={line.category} onChange={(event) => patchLine(index, { category: event.target.value as CostCategory })}>{Object.entries(categoryLabels).map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></td>
          <td><input required value={line.description} onChange={(event) => patchLine(index, { description: event.target.value })} /></td>
          <td><input value={line.specification} onChange={(event) => patchLine(index, { specification: event.target.value })} /></td>
          <td><input required type="number" min="0.001" step="0.001" value={line.quantity} onChange={(event) => patchLine(index, { quantity: Number(event.target.value) })} /></td>
          <td><input required value={line.unit} onChange={(event) => patchLine(index, { unit: event.target.value })} /></td>
          <td><input required type="number" min="0" step="0.01" value={line.unitCostMinor / 100} onChange={(event) => patchLine(index, { unitCostMinor: Math.round(Number(event.target.value) * 100) })} /></td>
          <td><strong>{formatMoney(Math.round(line.quantity * line.unitCostMinor), selectedCase?.currency ?? "CNY")}</strong></td>
          <td><input value={line.notes} onChange={(event) => patchLine(index, { notes: event.target.value })} /></td>
          <td><button type="button" className="danger-link" disabled={lines.length === 1} onClick={() => setLines((current) => current.filter((_, lineIndex) => lineIndex !== index))}>移除</button></td>
        </tr>)}</tbody></table></div>
        {error && <div className="form-error cost-estimate-error">{error}</div>}
        <div className="cost-estimate-footer"><div><span>成本合计</span><strong>{formatMoney(totalMinor, selectedCase?.currency ?? "CNY")}</strong></div><div><span>建议最低报价（目标毛利 {Number(targetMargin || 0).toFixed(2)}%）</span><strong>{formatMoney(suggestedMinor, selectedCase?.currency ?? "CNY")}</strong></div><div className="modal-actions"><button type="button" className="button button-secondary" onClick={onClose}>取消</button><button className="button button-primary" disabled={busy}>{busy ? "保存中…" : "保存成本估算"}</button></div></div>
      </form>
    </section>
  </div>;
}

export function CostEstimateCenter({ estimates, cases, purchaseOrders, onSave, onArchive }: {
  estimates: CostEstimate[];
  cases: BusinessCase[];
  purchaseOrders: PurchaseOrder[];
  onSave: (input: CostEstimateInput) => Promise<void>;
  onArchive: (id: string) => Promise<void>;
}) {
  const [query, setQuery] = useState("");
  const [editing, setEditing] = useState<CostEstimate | "new" | null>(null);
  const [attachmentRecord, setAttachmentRecord] = useState<CostEstimate | null>(null);
  const normalized = query.trim().toLocaleLowerCase();
  const filtered = useMemo(() => estimates.filter((item) =>
    [item.number, item.businessCaseNumber, item.customerName, item.currency, item.notes]
      .some((value) => value.toLocaleLowerCase().includes(normalized))), [estimates, normalized]);
  const currentCurrency = estimates[0]?.currency ?? "CNY";
  const comparable = estimates.filter((item) => item.currency === currentCurrency);

  async function archive(item: CostEstimate) {
    if (!window.confirm(`归档成本估算“${item.number}”？历史业务资料不会删除。`)) return;
    await onArchive(item.id);
  }

  return <>
    <section className="panel cost-estimate-center">
      <div className="panel-heading"><div><h2>成本估算中心</h2><p>把完整成本、目标毛利与报价底线放在同一张业务快照中</p></div><button className="button button-primary" disabled={!cases.length} onClick={() => setEditing("new")}>新建成本估算</button></div>
      {!cases.length && <div className="empty-callout">请先建立业务单，再创建成本估算。</div>}
      <div className="cost-summary-grid"><article><span>有效估算</span><strong>{estimates.length}</strong></article><article><span>{currentCurrency} 成本合计</span><strong>{formatMoney(comparable.reduce((sum, item) => sum + item.totalCostMinor, 0), currentCurrency)}</strong></article><article><span>平均目标毛利</span><strong>{comparable.length ? (comparable.reduce((sum, item) => sum + item.targetMarginBps, 0) / comparable.length / 100).toFixed(1) : "0.0"}%</strong></article></div>
      <div className="table-toolbar"><label><span className="sr-only">搜索成本估算</span><input placeholder="搜索估算编号、业务单或客户" value={query} onChange={(event) => setQuery(event.target.value)} /></label><span className="record-count">{filtered.length} 张成本估算</span></div>
      <div className="table-wrap"><table><thead><tr><th>估算编号</th><th>业务单</th><th>客户</th><th>币种</th><th>成本合计</th><th>目标毛利</th><th>建议最低报价</th><th>操作</th></tr></thead><tbody>{filtered.map((item) => <tr key={item.id}><td><strong>{item.number}</strong><small className="table-subtitle">{item.lines.length} 个成本项</small></td><td>{item.businessCaseNumber}</td><td>{item.customerName}</td><td>{item.currency}</td><td>{formatMoney(item.totalCostMinor, item.currency)}</td><td>{(item.targetMarginBps / 100).toFixed(2)}%</td><td><strong>{formatMoney(item.suggestedPriceMinor, item.currency)}</strong></td><td><div className="row-actions"><button onClick={() => setEditing(item)}>编辑</button><button onClick={() => setAttachmentRecord(item)}>附件</button><button onClick={() => void archive(item)}>归档</button></div></td></tr>)}</tbody></table>{!filtered.length && <div className="empty-table">暂无成本估算</div>}</div>
    </section>
    {editing && <CostEstimateEditor record={editing === "new" ? null : editing} estimates={estimates} cases={cases} purchaseOrders={purchaseOrders} onClose={() => setEditing(null)} onSave={onSave} />}
    {attachmentRecord && <div className="modal-backdrop" onMouseDown={() => setAttachmentRecord(null)}><section className="modal-card attachment-modal" onMouseDown={(event) => event.stopPropagation()}><div className="panel-heading"><div><span className="eyebrow">成本估算附件</span><h2>{attachmentRecord.number}</h2></div><button className="icon-button" onClick={() => setAttachmentRecord(null)}>×</button></div><AttachmentPanel entityType="cost_estimate" entityId={attachmentRecord.id} entityLabel={attachmentRecord.number} /></section></div>}
  </>;
}
