import { useEffect, useMemo, useRef, useState } from "react";
import { documentDraftApi } from "./api";
import { AttachmentPanel } from "./AttachmentPanel";
import { nextDatedNumber, todayIso } from "./numbering";
import type {
  BusinessCase,
  CompanyRecord,
  CompanyRegistry,
  CompanySigningAsset,
  ConvertDocumentInput,
  CreateDocumentInput,
  DocumentLineSnapshot,
  DocumentPayload,
  DocumentStatus,
  DocumentType,
  SaveDocumentInput,
  TradeDocument,
} from "./domain";

interface DocumentCenterProps {
  companyRegistry: CompanyRegistry | null;
  documents: TradeDocument[];
  cases: BusinessCase[];
  onCreate: (input: CreateDocumentInput) => Promise<TradeDocument>;
  onConvert: (input: ConvertDocumentInput) => Promise<TradeDocument>;
  onSave: (input: SaveDocumentInput) => Promise<TradeDocument>;
  onIssue: (id: string) => Promise<TradeDocument>;
  onVoid: (id: string, reason: string) => Promise<TradeDocument>;
  onNewVersion: (id: string) => Promise<TradeDocument>;
  onExportPdf: (id: string, companyId: string, signingAssetId: string) => Promise<string>;
  onExportCsv: (id: string) => Promise<string>;
  onPrint: (id: string, companyId: string, signingAssetId: string) => Promise<string>;
  onOpenPdf: (id: string) => Promise<void>;
}

const typeLabels: Record<DocumentType, string> = {
  commercial_quotation: "商业报价单",
  proforma_invoice: "形式发票",
  commercial_invoice: "商业发票",
  packing_list: "详细装箱单",
  trade_contract: "外贸合同",
};

const statusLabels: Record<DocumentStatus, string> = {
  draft: "草稿",
  issued: "已签发",
  voided: "已作废",
};

const prefixes: Record<DocumentType, string> = {
  commercial_quotation: "QUO",
  proforma_invoice: "PI",
  commercial_invoice: "INV",
  packing_list: "PKL",
  trade_contract: "CT",
};

const documentTypes: DocumentType[] = [
  "commercial_quotation",
  "proforma_invoice",
  "trade_contract",
  "commercial_invoice",
  "packing_list",
];

function nextNumber(documents: TradeDocument[], type: DocumentType, issueDate = todayIso()) {
  return nextDatedNumber(
    documents.filter((item) => item.documentType === type).map((item) => item.number),
    prefixes[type],
    issueDate,
  );
}

function money(value: number, currency: string) {
  try {
    return new Intl.NumberFormat("en-US", {
      style: "currency",
      currency: currency || "USD",
    }).format(value / 100);
  } catch {
    return `${currency || "USD"} ${(value / 100).toFixed(2)}`;
  }
}

function numberValue(value: string) {
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : 0;
}

function CreateDocumentModal({
  documents,
  cases,
  onClose,
  onCreate,
}: {
  documents: TradeDocument[];
  cases: BusinessCase[];
  onClose: () => void;
  onCreate: (input: CreateDocumentInput) => Promise<void>;
}) {
  const [type, setType] = useState<DocumentType>("commercial_quotation");
  const [caseId, setCaseId] = useState(cases[0]?.id ?? "");
  const [number, setNumber] = useState(nextNumber(documents, type));
  const [issueDate, setIssueDate] = useState(todayIso());
  const [language, setLanguage] = useState("zh_en");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");

  async function submit(event: React.FormEvent) {
    event.preventDefault();
    setSaving(true);
    setError("");
    try {
      await onCreate({ businessCaseId: caseId, documentType: type, number, issueDate, language });
    } catch (reason) {
      setError(String(reason));
    } finally {
      setSaving(false);
    }
  }

  return <div className="modal-backdrop" onMouseDown={onClose}>
    <form className="modal-card document-create" onSubmit={submit} onMouseDown={(event) => event.stopPropagation()}>
      <div className="panel-heading"><div><span className="eyebrow">业务单证包</span><h2>从业务单生成单证</h2></div><button type="button" className="icon-button" onClick={onClose}>×</button></div>
      <div className="form-grid two-columns">
        <label>单证类型<select value={type} onChange={(event) => { const next = event.target.value as DocumentType; setType(next); setNumber(nextNumber(documents, next, issueDate)); }}>{documentTypes.map((item) => <option value={item} key={item}>{typeLabels[item]}</option>)}</select></label>
        <label>来源业务单<select required value={caseId} onChange={(event) => setCaseId(event.target.value)}>{cases.map((item) => <option value={item.id} key={item.id}>{item.number} · {item.customerName}</option>)}</select></label>
        <label>单证编号<input required value={number} onChange={(event) => setNumber(event.target.value)} /></label>
        <label>签发日期<input required type="date" value={issueDate} onChange={(event) => { setIssueDate(event.target.value); setNumber(nextNumber(documents, type, event.target.value)); }} /></label>
        <label>输出语言<select value={language} onChange={(event) => setLanguage(event.target.value)}><option value="zh_en">中英双语</option><option value="en">英文</option><option value="ru">俄文</option></select></label>
      </div>
      {error && <div className="form-error">{error}</div>}
      <div className="modal-actions"><button type="button" className="button button-secondary" onClick={onClose}>取消</button><button className="button button-primary" disabled={saving || !caseId}>{saving ? "生成中…" : "生成草稿"}</button></div>
    </form>
  </div>;
}

function ConvertDocumentModal({ source, documents, onClose, onConvert }: {
  source: TradeDocument;
  documents: TradeDocument[];
  onClose: () => void;
  onConvert: (input: ConvertDocumentInput) => Promise<void>;
}) {
  const targets: DocumentType[] = source.documentType === "commercial_quotation"
    ? ["proforma_invoice", "trade_contract"]
    : ["trade_contract", "commercial_invoice"];
  const [target, setTarget] = useState<DocumentType>(targets[0]);
  const [number, setNumber] = useState(nextNumber(documents, targets[0]));
  const [issueDate, setIssueDate] = useState(todayIso());
  const [language, setLanguage] = useState(source.language);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");

  async function submit(event: React.FormEvent) {
    event.preventDefault();
    setSaving(true); setError("");
    try {
      await onConvert({ sourceDocumentId: source.id, targetDocumentType: target, number, issueDate, language });
    } catch (reason) {
      setError(String(reason));
    } finally {
      setSaving(false);
    }
  }

  return <div className="modal-backdrop" onMouseDown={onClose}>
    <form className="modal-card document-create" onSubmit={submit} onMouseDown={(event) => event.stopPropagation()}>
      <div className="panel-heading"><div><span className="eyebrow">复用已签发快照</span><h2>转换 {source.number}</h2></div><button type="button" className="icon-button" onClick={onClose}>×</button></div>
      <p className="empty-callout">客户、产品、价格、折扣和贸易条款将原样复制，不读取已变化的主数据。</p>
      <div className="form-grid two-columns">
        <label>目标单证<select value={target} onChange={(event) => { const next = event.target.value as DocumentType; setTarget(next); setNumber(nextNumber(documents, next, issueDate)); }}>{targets.map((item) => <option value={item} key={item}>{typeLabels[item]}</option>)}</select></label>
        <label>单证编号<input required value={number} onChange={(event) => setNumber(event.target.value)} /></label>
        <label>签发日期<input required type="date" value={issueDate} onChange={(event) => { setIssueDate(event.target.value); setNumber(nextNumber(documents, target, event.target.value)); }} /></label>
        <label>输出语言<select value={language} onChange={(event) => setLanguage(event.target.value)}><option value="zh_en">中英双语</option><option value="en">英文</option><option value="ru">俄文</option></select></label>
      </div>
      {error && <div className="form-error">{error}</div>}
      <div className="modal-actions"><button type="button" className="button button-secondary" onClick={onClose}>取消</button><button className="button button-primary" disabled={saving}>{saving ? "转换中…" : "生成转换草稿"}</button></div>
    </form>
  </div>;
}

function Preview({ document, payload, company, signingAsset }: { document: TradeDocument; payload: DocumentPayload; company?: CompanyRecord; signingAsset?: CompanySigningAsset }) {
  const total = payload.lines.reduce((sum, line) => sum + line.amountMinor, 0);
  const payable = total - payload.discountMinor;
  const packing = document.documentType === "packing_list";
  const contract = document.documentType === "trade_contract";
  const quotation = document.documentType === "commercial_quotation";
  const proforma = document.documentType === "proforma_invoice";
  const title = contract ? "SALES CONTRACT" : packing ? "DETAILED PACKING LIST" : quotation ? "COMMERCIAL QUOTATION" : proforma ? "PROFORMA INVOICE" : "COMMERCIAL INVOICE";
  return <div className="document-paper landscape">
    <header>{company?.logoDataUrl && <img className="preview-logo" src={company.logoDataUrl} alt="公司 Logo" />}<h2>{title}</h2><p>{typeLabels[document.documentType]} · {document.number} · V{document.version}</p>{document.status === "draft" && <strong>DRAFT / 草稿</strong>}</header>
    <div className="preview-parties"><div><b>{packing ? "SHIPPER" : "SELLER / EXPORTER"}</b><span>{payload.seller}</span><small>{payload.sellerAddress}</small></div><div><b>{packing ? "CONSIGNEE" : "BUYER / CONSIGNEE"}</b><span>{payload.buyer}</span><small>{payload.buyerAddress}</small></div></div>
    <div className="preview-meta"><span><b>No.</b>{document.number}</span><span><b>Date</b>{document.issueDate}</span><span><b>Reference</b>{document.businessCaseNumber}</span><span><b>Incoterm</b>{payload.incoterm}</span>{quotation ? <span><b>Valid until</b>{payload.validUntil}</span> : <><span><b>Loading</b>{payload.portOfLoading}</span><span><b>Discharge</b>{payload.portOfDischarge}</span></>}</div>
    <table><thead><tr><th>No.</th><th>SKU / Description</th><th>Qty</th>{packing ? <><th>Packages</th><th>Net kg</th><th>Gross kg</th><th>CBM</th></> : <><th>Unit price</th><th>Amount</th></>}</tr></thead><tbody>{payload.lines.map((line, index) => <tr key={`${line.productId}-${index}`}><td>{index + 1}</td><td><b>{line.sku}</b> · {line.description}{line.model && <small> · {line.model}</small>}</td><td>{line.quantity} {line.unit}</td>{packing ? <><td>{line.packages} {line.packageType}</td><td>{line.netWeightKg}</td><td>{line.grossWeightKg}</td><td>{line.cbm}</td></> : <><td>{money(line.unitPriceMinor, document.currency)}</td><td>{money(line.amountMinor, document.currency)}</td></>}</tr>)}</tbody>{!packing && <tfoot><tr><td colSpan={4}>TOTAL</td><td>{money(payable, document.currency)}</td></tr></tfoot>}</table>
    {!packing && payload.discountMinor > 0 && <section><h3>DISCOUNT</h3><p>{money(payload.discountMinor, document.currency)} · Total after discount: {money(payable, document.currency)}</p></section>}
    {contract && <section><h3>GENERAL TERMS</h3><p>{payload.contractTerms || "General trade terms shall be confirmed in writing by both parties."}</p></section>}
    {payload.notes && <section><h3>NOTES</h3><p>{payload.notes}</p></section>}
    <footer><span>{company?.companyName || payload.seller} · Encrypted local snapshot</span><span className={`preview-signature ${signingAsset?.kind === "stamp" ? "stamp" : ""}`}>{signingAsset?.dataUrl && <img src={signingAsset.dataUrl} alt={signingAsset.kind === "stamp" ? "电子章" : "电子签名"} />}<b>Authorized Signature / Stamp</b></span></footer>
  </div>;
}

function LineEditor({ line, packing, onChange }: { line: DocumentLineSnapshot; packing: boolean; onChange: (line: DocumentLineSnapshot) => void }) {
  const set = (patch: Partial<DocumentLineSnapshot>) => onChange({ ...line, ...patch });
  return <div className="document-line-form">
    <div className="document-line-name"><strong>{line.sku}</strong><input value={line.description} onChange={(event) => set({ description: event.target.value })} /><span>{line.model || "无型号"} · HS {line.hsCode || "未填写"}</span></div>
    <label>数量<input type="number" min="0.001" step="0.001" value={line.quantity} onChange={(event) => set({ quantity: numberValue(event.target.value) })} /></label>
    {packing ? <>
      <label>箱数<input type="number" min="1" step="1" value={line.packages} onChange={(event) => set({ packages: numberValue(event.target.value) })} /></label>
      <label>包装<input value={line.packageType} onChange={(event) => set({ packageType: event.target.value })} /></label>
      <label>净重 kg<input type="number" min="0" step="0.001" value={line.netWeightKg} onChange={(event) => set({ netWeightKg: numberValue(event.target.value) })} /></label>
      <label>毛重 kg<input type="number" min="0" step="0.001" value={line.grossWeightKg} onChange={(event) => set({ grossWeightKg: numberValue(event.target.value) })} /></label>
      <label>CBM<input type="number" min="0" step="0.001" value={line.cbm} onChange={(event) => set({ cbm: numberValue(event.target.value) })} /></label>
    </> : <label>单价<input type="number" min="0" step="0.01" value={(line.unitPriceMinor / 100).toFixed(2)} onChange={(event) => set({ unitPriceMinor: Math.round(numberValue(event.target.value) * 100) })} /></label>}
  </div>;
}

function DocumentEditor({ initial, companyRegistry, onClose, onSave, onIssue, onExportPdf, onExportCsv, onPrint }: {
  initial: TradeDocument;
  companyRegistry: CompanyRegistry | null;
  onClose: () => void;
  onSave: (input: SaveDocumentInput) => Promise<TradeDocument>;
  onIssue: (id: string) => Promise<TradeDocument>;
  onExportPdf: (id: string, companyId: string, signingAssetId: string) => Promise<string>;
  onExportCsv: (id: string) => Promise<string>;
  onPrint: (id: string, companyId: string, signingAssetId: string) => Promise<string>;
}) {
  const [document, setDocument] = useState(initial);
  const [number, setNumber] = useState(initial.number);
  const [issueDate, setIssueDate] = useState(initial.issueDate);
  const [language, setLanguage] = useState(initial.language);
  const [payload, setPayload] = useState<DocumentPayload>(() => structuredClone(initial.payload));
  const [busy, setBusy] = useState("");
  const [message, setMessage] = useState("");
  const [companyId, setCompanyId] = useState(companyRegistry?.defaultCompanyId ?? "");
  const [signingAssetId, setSigningAssetId] = useState("");
  const [draftReady, setDraftReady] = useState(false);
  const [autosaveState, setAutosaveState] = useState("");
  const lastDraftSignature = useRef(JSON.stringify({ id: initial.id, number: initial.number, issueDate: initial.issueDate, language: initial.language, payload: initial.payload }));
  const autosaveTimer = useRef<number | null>(null);
  const autosavePromise = useRef<Promise<unknown> | null>(null);
  const selectedCompany = companyRegistry?.companies.find((item) => item.id === companyId) ?? companyRegistry?.companies[0];
  const selectedAsset = selectedCompany?.signingAssets.find((item) => item.id === signingAssetId);
  const editable = document.status === "draft";
  const packing = document.documentType === "packing_list";
  const quotation = document.documentType === "commercial_quotation";
  const invoiceLike = document.documentType === "commercial_invoice" || document.documentType === "proforma_invoice";
  const setPayloadField = (patch: Partial<DocumentPayload>) => setPayload((current) => ({ ...current, ...patch }));
  const draftInput = (): SaveDocumentInput => ({ id: document.id, number, issueDate, language, payload });

  useEffect(() => {
    if (!editable) { setDraftReady(true); return; }
    let cancelled = false;
    documentDraftApi.load(document.id).then((draft) => {
      if (cancelled || !draft) return;
      setNumber(draft.input.number);
      setIssueDate(draft.input.issueDate);
      setLanguage(draft.input.language);
      setPayload(structuredClone(draft.input.payload));
      lastDraftSignature.current = JSON.stringify(draft.input);
      setMessage(`已恢复 ${draft.updatedAt} 自动保存的编辑内容`);
    }).catch((reason) => {
      if (!cancelled) setMessage(`读取自动草稿失败：${String(reason)}`);
    }).finally(() => {
      if (!cancelled) setDraftReady(true);
    });
    return () => { cancelled = true; };
  }, [document.id, editable]);

  useEffect(() => {
    if (!editable || !draftReady) return;
    const input = draftInput();
    const signature = JSON.stringify(input);
    if (signature === lastDraftSignature.current) return;
    setAutosaveState("等待自动保存");
    autosaveTimer.current = window.setTimeout(() => {
      setAutosaveState("自动保存中…");
      const pending = documentDraftApi.save(input).then((draft) => {
        lastDraftSignature.current = JSON.stringify(draft.input);
        setAutosaveState(`已自动保存 ${draft.updatedAt}`);
      }).catch((reason) => setAutosaveState(`自动保存失败：${String(reason)}`));
      autosavePromise.current = pending;
      void pending.finally(() => {
        if (autosavePromise.current === pending) autosavePromise.current = null;
      });
    }, 900);
    return () => {
      if (autosaveTimer.current !== null) window.clearTimeout(autosaveTimer.current);
      autosaveTimer.current = null;
    };
  }, [document.id, draftReady, editable, issueDate, language, number, payload]);

  async function finishPendingAutosave() {
    if (autosaveTimer.current !== null) {
      window.clearTimeout(autosaveTimer.current);
      autosaveTimer.current = null;
    }
    if (autosavePromise.current) await autosavePromise.current;
  }

  async function save() {
    setBusy("save"); setMessage("");
    try {
      await finishPendingAutosave();
      const input = draftInput();
      const updated = await onSave(input);
      await documentDraftApi.delete(document.id);
      lastDraftSignature.current = JSON.stringify(input);
      setDocument(updated); setPayload(structuredClone(updated.payload)); setMessage("草稿已保存");
      return updated;
    } catch (reason) { setMessage(String(reason)); throw reason; } finally { setBusy(""); }
  }

  async function issue() {
    setBusy("issue"); setMessage("");
    try {
      await finishPendingAutosave();
      const saved = await onSave({ id: document.id, number, issueDate, language, payload });
      const updated = await onIssue(saved.id);
      await documentDraftApi.delete(document.id);
      setDocument(updated); setPayload(structuredClone(updated.payload)); setMessage("已签发并冻结为只读版本");
    } catch (reason) { setMessage(String(reason)); } finally { setBusy(""); }
  }

  async function closeEditor() {
    if (editable && draftReady) {
      const input = draftInput();
      const signature = JSON.stringify(input);
      if (signature !== lastDraftSignature.current) {
        try {
          await finishPendingAutosave();
          await documentDraftApi.save(input);
        } catch (reason) {
          setMessage(`关闭前自动保存失败：${String(reason)}`);
          return;
        }
      }
    }
    onClose();
  }

  async function output(action: "pdf" | "csv" | "print") {
    setBusy(action); setMessage("");
    try {
      if (editable) await save();
      const path = action === "pdf" ? await onExportPdf(document.id, companyId, signingAssetId) : action === "csv" ? await onExportCsv(document.id) : await onPrint(document.id, companyId, signingAssetId);
      setMessage(`${action === "print" ? "已打开打印用 PDF" : "已导出"}：${path}`);
    } catch (reason) { setMessage(String(reason)); } finally { setBusy(""); }
  }

  return <div className="document-editor-shell">
    <header className="document-editor-toolbar"><div><span className="eyebrow">{typeLabels[document.documentType]} · V{document.version}</span><h2>{document.number}</h2>{editable && autosaveState && <small className="autosave-state">{autosaveState}</small>}</div><div className="document-toolbar-actions">{editable && <button className="button button-secondary" disabled={!!busy} onClick={() => void save().catch(() => undefined)}>保存草稿</button>}{editable && <button className="button button-primary" disabled={!!busy} onClick={() => void issue()}>签发冻结</button>}<button className="button button-secondary" disabled={!!busy} onClick={() => void output("pdf")}>PDF</button><button className="button button-secondary" disabled={!!busy} onClick={() => void output("csv")}>CSV</button><button className="button button-secondary" disabled={!!busy} onClick={() => void output("print")}>打印</button><button className="icon-button" onClick={() => void closeEditor()}>×</button></div></header>
    {message && <div className="document-message">{message}</div>}
    <div className="document-editor-layout">
      <div className="document-form-panel">
        <div className="document-brand-selectors"><label>导出公司<select value={companyId} onChange={(event) => { setCompanyId(event.target.value); setSigningAssetId(""); }}>{companyRegistry?.companies.map((item) => <option value={item.id} key={item.id}>{item.companyName}</option>)}</select></label><label>签章（可不选）<select value={signingAssetId} onChange={(event) => setSigningAssetId(event.target.value)}><option value="">不使用签章</option>{selectedCompany?.signingAssets.map((item) => <option value={item.id} key={item.id}>{item.kind === "stamp" ? "电子章" : "电子签名"} · {item.name}</option>)}</select></label></div>
        <AttachmentPanel entityType="document" entityId={document.id} entityLabel={`${document.number} V${document.version}`} title="单证附件" />
        {!editable && <div className="locked-callout">该版本已{document.status === "issued" ? "签发" : "作废"}，内容只读；修改请创建新版本。</div>}
        {document.validationIssues.length > 0 && <div className="validation-list">{document.validationIssues.map((issue, index) => <span className={issue.severity} key={`${issue.code}-${index}`}>{issue.severity === "error" ? "错误" : "提醒"} · {issue.message}</span>)}</div>}
        <fieldset disabled={!editable}><div className="form-grid two-columns"><label>单证编号<input value={number} onChange={(event) => setNumber(event.target.value)} /></label><label>签发日期<input type="date" value={issueDate} onChange={(event) => setIssueDate(event.target.value)} /></label><label>输出语言<select value={language} onChange={(event) => setLanguage(event.target.value)}><option value="zh_en">中英双语</option><option value="en">英文</option><option value="ru">俄文</option></select></label><label>来源业务单<input value={document.businessCaseNumber} readOnly /></label></div>
        <h3>买卖双方</h3><div className="form-grid two-columns"><label>卖方/出口商<input value={payload.seller} onChange={(event) => setPayloadField({ seller: event.target.value })} /></label><label>买方/收货人<input value={payload.buyer} onChange={(event) => setPayloadField({ buyer: event.target.value })} /></label><label>卖方地址<textarea value={payload.sellerAddress} onChange={(event) => setPayloadField({ sellerAddress: event.target.value })} /></label><label>买方地址<textarea value={payload.buyerAddress} onChange={(event) => setPayloadField({ buyerAddress: event.target.value })} /></label></div>
        <h3>贸易与运输</h3><div className="form-grid two-columns"><label>原产国<input value={payload.originCountry} onChange={(event) => setPayloadField({ originCountry: event.target.value })} /></label><label>目的国<input value={payload.destinationCountry} onChange={(event) => setPayloadField({ destinationCountry: event.target.value })} /></label><label>装运港<input value={payload.portOfLoading} onChange={(event) => setPayloadField({ portOfLoading: event.target.value })} /></label><label>目的港<input value={payload.portOfDischarge} onChange={(event) => setPayloadField({ portOfDischarge: event.target.value })} /></label><label>贸易术语<input value={payload.incoterm} onChange={(event) => setPayloadField({ incoterm: event.target.value })} /></label><label>付款条款<input value={payload.paymentTerms} onChange={(event) => setPayloadField({ paymentTerms: event.target.value })} /></label><label>装运日期<input type="date" value={payload.shipmentDate} onChange={(event) => setPayloadField({ shipmentDate: event.target.value })} /></label><label>客户 PO<input value={payload.poReference} onChange={(event) => setPayloadField({ poReference: event.target.value })} /></label>{quotation && <label>报价有效期<input type="date" value={payload.validUntil} onChange={(event) => setPayloadField({ validUntil: event.target.value })} /></label>}</div>
        <h3>产品明细</h3><div className="document-lines">{payload.lines.map((line, index) => <LineEditor line={line} packing={packing} onChange={(updated) => setPayloadField({ lines: payload.lines.map((item, itemIndex) => itemIndex === index ? updated : item) })} key={`${line.productId}-${index}`} />)}</div>
        {!packing && <label>折扣金额（{document.currency}）<input type="number" min="0" step="0.01" value={(payload.discountMinor / 100).toFixed(2)} onChange={(event) => setPayloadField({ discountMinor: Math.round(numberValue(event.target.value) * 100) })} /></label>}
        {invoiceLike && <label>银行资料<textarea rows={3} value={payload.bankDetails} onChange={(event) => setPayloadField({ bankDetails: event.target.value })} /></label>}
        {document.documentType === "trade_contract" && <label>合同通用条款<textarea rows={6} value={payload.contractTerms} onChange={(event) => setPayloadField({ contractTerms: event.target.value })} /></label>}
        <label>备注<textarea rows={4} value={payload.notes} onChange={(event) => setPayloadField({ notes: event.target.value })} /></label></fieldset>
      </div>
      <div className="document-preview-panel"><Preview document={{ ...document, number, issueDate, language }} payload={payload} company={selectedCompany} signingAsset={selectedAsset} /></div>
    </div>
  </div>;
}

export function DocumentCenter(props: DocumentCenterProps) {
  const [creating, setCreating] = useState(false);
  const [converting, setConverting] = useState<TradeDocument | null>(null);
  const [editing, setEditing] = useState<TradeDocument | null>(null);
  const [query, setQuery] = useState("");
  const [type, setType] = useState<"all" | DocumentType>("all");
  const [status, setStatus] = useState<"all" | DocumentStatus>("all");
  const [message, setMessage] = useState("");
  const filtered = useMemo(() => props.documents.filter((document) => {
    const text = `${document.number} ${document.customerName} ${document.businessCaseNumber}`.toLowerCase();
    return (type === "all" || document.documentType === type) && (status === "all" || document.status === status) && text.includes(query.toLowerCase());
  }), [props.documents, query, status, type]);

  async function create(input: CreateDocumentInput) {
    const document = await props.onCreate(input);
    setCreating(false); setEditing(document);
  }

  async function convert(input: ConvertDocumentInput) {
    const document = await props.onConvert(input);
    setConverting(null); setEditing(document);
  }

  async function voidDocument(document: TradeDocument) {
    const reason = window.prompt("请输入作废原因（旧版本仍会保留）");
    if (!reason) return;
    try { await props.onVoid(document.id, reason); setMessage(`${document.number} V${document.version} 已作废`); } catch (error) { setMessage(String(error)); }
  }

  async function newVersion(document: TradeDocument) {
    try { const created = await props.onNewVersion(document.id); setEditing(created); } catch (error) { setMessage(String(error)); }
  }

  if (editing) return <DocumentEditor initial={editing} companyRegistry={props.companyRegistry} onClose={() => setEditing(null)} onSave={props.onSave} onIssue={props.onIssue} onExportPdf={props.onExportPdf} onExportCsv={props.onExportCsv} onPrint={props.onPrint} />;

  return <section className="panel document-center">
    <div className="panel-heading"><div><h2>单证中心</h2><p>从报价、PI 到履约单证，复用同一份加密业务快照</p></div><button className="button button-primary" disabled={!props.cases.length} onClick={() => setCreating(true)}>新建单证</button></div>
    {!props.cases.length && <div className="empty-callout">请先建立报价阶段的业务单，再生成报价、形式发票、合同及出货单证。</div>}
    {message && <div className="document-message">{message}</div>}
    <div className="document-filters"><input placeholder="搜索单号、业务单或客户" value={query} onChange={(event) => setQuery(event.target.value)} /><select value={type} onChange={(event) => setType(event.target.value as typeof type)}><option value="all">全部类型</option>{documentTypes.map((item) => <option value={item} key={item}>{typeLabels[item]}</option>)}</select><select value={status} onChange={(event) => setStatus(event.target.value as typeof status)}><option value="all">全部状态</option><option value="draft">草稿</option><option value="issued">已签发</option><option value="voided">已作废</option></select><span>{filtered.length} 个版本</span></div>
    <div className="document-history-list">{filtered.map((document) => <article className="document-history-card" key={document.id}><div className="document-type-mark">{prefixes[document.documentType]}</div><div className="document-history-main"><span className="eyebrow">{typeLabels[document.documentType]} · {document.businessCaseNumber}</span><h3>{document.number} <small>V{document.version}</small></h3><p>{document.customerName} · {document.issueDate} · 模板 {document.templateVersion}</p><div className="document-chips"><span className={`document-status ${document.status}`}>{statusLabels[document.status]}</span>{document.validationIssues.filter((item) => item.severity === "error").length > 0 && <span className="validation-chip">{document.validationIssues.filter((item) => item.severity === "error").length} 个错误</span>}{document.pdfSha256 && <span>PDF {document.pdfSha256.slice(0, 10)}…</span>}</div></div><div className="document-card-actions"><button onClick={() => setEditing(document)}>{document.status === "draft" ? "编辑" : "查看"}</button>{document.pdfPath && <button onClick={() => void props.onOpenPdf(document.id)}>打开 PDF</button>}{document.status === "issued" && (document.documentType === "commercial_quotation" || document.documentType === "proforma_invoice") && <button onClick={() => setConverting(document)}>转换单证</button>}{document.status === "issued" && <button onClick={() => void newVersion(document)}>创建新版本</button>}{document.status === "issued" && <button className="danger-link" onClick={() => void voidDocument(document)}>作废</button>}</div></article>)}</div>
    {!filtered.length && <div className="empty-table">{props.documents.length ? "没有符合筛选条件的单证" : "还没有历史单证，可从业务单生成第一份草稿"}</div>}
    {creating && <CreateDocumentModal documents={props.documents} cases={props.cases} onClose={() => setCreating(false)} onCreate={create} />}
    {converting && <ConvertDocumentModal source={converting} documents={props.documents} onClose={() => setConverting(null)} onConvert={convert} />}
  </section>;
}
