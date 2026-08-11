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
  onReview: (id: string) => Promise<TradeDocument>;
  onIssue: (id: string) => Promise<TradeDocument>;
  onVoid: (id: string, reason: string) => Promise<TradeDocument>;
  onArchive: (id: string) => Promise<TradeDocument>;
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
  shipping_marks: "运输唛头",
  shipper_instruction: "发货人委托书",
  customs_declaration: "报关资料",
  bill_of_lading: "提单补料",
  insurance_policy: "保险申请资料",
  certificate_of_origin: "原产地证明申请",
  inspection_certificate: "检验证书申请",
  fumigation_certificate: "熏蒸证书申请",
  beneficiary_certificate: "受益人证明",
};

const statusLabels: Record<DocumentStatus, string> = {
  draft: "草稿",
  reviewed: "已审核",
  issued: "已签发",
  voided: "已作废",
  archived: "已归档",
};

const prefixes: Record<DocumentType, string> = {
  commercial_quotation: "QUO",
  proforma_invoice: "PI",
  commercial_invoice: "INV",
  packing_list: "PKL",
  trade_contract: "CT",
  shipping_marks: "MARK",
  shipper_instruction: "SI",
  customs_declaration: "CUS",
  bill_of_lading: "BL",
  insurance_policy: "INS",
  certificate_of_origin: "COO",
  inspection_certificate: "IC",
  fumigation_certificate: "FUM",
  beneficiary_certificate: "BC",
};

const documentTypes: DocumentType[] = [
  "commercial_quotation",
  "proforma_invoice",
  "trade_contract",
  "commercial_invoice",
  "packing_list",
  "shipping_marks",
  "shipper_instruction",
  "customs_declaration",
  "bill_of_lading",
  "insurance_policy",
  "certificate_of_origin",
  "inspection_certificate",
  "fumigation_certificate",
  "beneficiary_certificate",
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

function canConvertDocument(type: DocumentType) {
  return ["commercial_quotation", "proforma_invoice", "commercial_invoice", "packing_list", "shipper_instruction", "customs_declaration", "bill_of_lading"].includes(type);
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
    : source.documentType === "proforma_invoice"
      ? ["trade_contract", "commercial_invoice"]
      : source.documentType === "commercial_invoice"
        ? ["packing_list", "shipping_marks", "shipper_instruction", "customs_declaration", "bill_of_lading", "insurance_policy", "certificate_of_origin", "inspection_certificate", "fumigation_certificate", "beneficiary_certificate"]
        : source.documentType === "packing_list"
          ? ["shipping_marks", "shipper_instruction", "customs_declaration", "bill_of_lading", "insurance_policy", "certificate_of_origin", "inspection_certificate", "fumigation_certificate", "beneficiary_certificate"]
          : source.documentType === "shipper_instruction"
            ? ["bill_of_lading", "insurance_policy", "certificate_of_origin", "inspection_certificate", "fumigation_certificate", "beneficiary_certificate"]
            : source.documentType === "customs_declaration"
              ? ["certificate_of_origin", "inspection_certificate", "fumigation_certificate", "beneficiary_certificate"]
              : ["insurance_policy", "certificate_of_origin", "inspection_certificate", "fumigation_certificate", "beneficiary_certificate"];
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
  const packing = ["packing_list", "shipping_marks", "shipper_instruction", "customs_declaration", "bill_of_lading", "certificate_of_origin", "inspection_certificate", "fumigation_certificate"].includes(document.documentType);
  const contract = document.documentType === "trade_contract";
  const quotation = document.documentType === "commercial_quotation";
  const proforma = document.documentType === "proforma_invoice";
  const title = document.documentType === "shipping_marks" ? "SHIPPING MARKS" : document.documentType === "shipper_instruction" ? "SHIPPER'S INSTRUCTION" : document.documentType === "customs_declaration" ? "CUSTOMS DECLARATION DATA" : document.documentType === "bill_of_lading" ? "BILL OF LADING INSTRUCTIONS" : document.documentType === "insurance_policy" ? "CARGO INSURANCE APPLICATION" : document.documentType === "certificate_of_origin" ? "CERTIFICATE OF ORIGIN APPLICATION" : document.documentType === "inspection_certificate" ? "INSPECTION CERTIFICATE APPLICATION" : document.documentType === "fumigation_certificate" ? "FUMIGATION CERTIFICATE APPLICATION" : document.documentType === "beneficiary_certificate" ? "BENEFICIARY'S CERTIFICATE" : contract ? "SALES CONTRACT" : packing ? "DETAILED PACKING LIST" : quotation ? "COMMERCIAL QUOTATION" : proforma ? "PROFORMA INVOICE" : "COMMERCIAL INVOICE";
  return <div className="document-paper landscape">
    <header><div className="preview-logo-slot">{company?.logoDataUrl && <img className="preview-logo" src={company.logoDataUrl} alt="公司 Logo" />}</div><div className="preview-heading"><h2>{title}</h2><p>{typeLabels[document.documentType]} · {document.number} · V{document.version}</p></div><div className="preview-header-status">{document.status === "draft" && <strong>DRAFT / 草稿</strong>}</div></header>
    <div className="preview-parties"><div><b>{packing ? "SHIPPER" : "SELLER / EXPORTER"}</b><span>{payload.seller}</span><small>{payload.sellerAddress}</small></div><div><b>{packing ? "CONSIGNEE" : "BUYER / CONSIGNEE"}</b><span>{payload.buyer}</span><small>{payload.buyerAddress}</small></div></div>
    <div className="preview-meta"><span><b>No.</b>{document.number}</span><span><b>Date</b>{document.issueDate}</span><span><b>Reference</b>{document.businessCaseNumber}</span><span><b>Incoterm</b>{payload.incoterm}</span>{quotation ? <span><b>Valid until</b>{payload.validUntil}</span> : <><span><b>Loading</b>{payload.portOfLoading}</span><span><b>Discharge</b>{payload.portOfDischarge}</span></>}</div>
    {document.documentType === "shipping_marks" && <section><h3>SHIPPING MARKS</h3><p>{payload.shippingMarks}</p></section>}
    {["shipper_instruction", "customs_declaration", "bill_of_lading", "insurance_policy"].includes(document.documentType) && <div className="preview-meta"><span><b>Transport</b>{payload.transportMode}</span><span><b>Vessel / Voyage</b>{payload.vesselVoyage || "—"}</span>{document.documentType === "customs_declaration" ? <><span><b>Supervision</b>{payload.customsSupervisionCode || "—"}</span><span><b>Declaration</b>{payload.customsDeclarationElements || "—"}</span></> : document.documentType === "insurance_policy" ? <><span><b>Insurer</b>{payload.insuranceCompany || "—"}</span><span><b>Insured value</b>{money(payload.insuredValueMinor, document.currency)}</span></> : <><span><b>Booking / B/L</b>{payload.bookingReference || "—"} / {payload.billOfLadingNumber || "—"}</span><span><b>Freight / B/L type</b>{payload.freightTerms} / {payload.billOfLadingType}</span></>}</div>}
    {document.documentType === "certificate_of_origin" && <div className="preview-meta"><span><b>Certificate type</b>{payload.certificateType}</span><span><b>Issuing authority</b>{payload.certificationAuthority || "—"}</span><span><b>Country of origin</b>{payload.originCountry}</span><span><b>Transport</b>{payload.transportMode} / {payload.vesselVoyage || "—"}</span></div>}
    {document.documentType === "inspection_certificate" && <div className="preview-meta"><span><b>Inspection body</b>{payload.certificationAuthority || "—"}</span><span><b>Manufacturer</b>{payload.manufacturer}</span><span><b>Batch / Standard</b>{payload.batchNumber || "—"} / {payload.inspectionStandard || "—"}</span><span><b>Result</b>{payload.inspectionResult || "—"}</span></div>}
    {document.documentType === "fumigation_certificate" && <div className="preview-meta"><span><b>Service provider</b>{payload.certificationAuthority || "—"}</span><span><b>Agent / Method</b>{payload.fumigationAgent || "—"} / {payload.fumigationMethod || "—"}</span><span><b>Treatment</b>{payload.fumigationTemperatureCelsius} °C / {payload.fumigationDurationHours} h</span><span><b>Date / Place</b>{payload.fumigationDate || "—"} / {payload.fumigationPlace || "—"}</span></div>}
    {document.documentType === "beneficiary_certificate" && <div className="preview-meta"><span><b>L/C No.</b>{payload.letterOfCreditNumber || "—"}</span><span><b>Issuing bank</b>{payload.issuingBank || "—"}</span><span><b>Expiry</b>{payload.letterOfCreditExpiryDate || "—"}</span><span><b>Presentation deadline</b>{payload.presentationDeadline || "—"}</span></div>}
    <table><thead><tr><th>No.</th><th>SKU / Description</th><th>Qty</th>{packing ? <><th>Packages</th><th>Net kg</th><th>Gross kg</th><th>CBM</th></> : <><th>Unit price</th><th>Amount</th></>}</tr></thead><tbody>{payload.lines.map((line, index) => <tr key={`${line.productId}-${index}`}><td>{index + 1}</td><td><b>{line.sku}</b> · {line.description}{line.model && <small> · {line.model}</small>}</td><td>{line.quantity} {line.unit}</td>{packing ? <><td>{line.packages} {line.packageType}</td><td>{line.netWeightKg}</td><td>{line.grossWeightKg}</td><td>{line.cbm}</td></> : <><td>{money(line.unitPriceMinor, document.currency)}</td><td>{money(line.amountMinor, document.currency)}</td></>}</tr>)}</tbody>{!packing && <tfoot><tr><td colSpan={4}>TOTAL</td><td>{money(payable, document.currency)}</td></tr></tfoot>}</table>
    {!packing && payload.discountMinor > 0 && <section><h3>DISCOUNT</h3><p>{money(payload.discountMinor, document.currency)} · Total after discount: {money(payable, document.currency)}</p></section>}
    {contract && <section><h3>GENERAL TERMS</h3><p>{payload.contractTerms || "General trade terms shall be confirmed in writing by both parties."}</p></section>}
    {document.documentType === "beneficiary_certificate" && <section><h3>BENEFICIARY'S STATEMENT</h3><p>{payload.beneficiaryStatement || "—"}</p></section>}
    {payload.notes && <section><h3>NOTES</h3><p>{payload.notes}</p></section>}
    <footer><span>{company?.companyName || payload.seller} · Encrypted local snapshot</span><span className={`preview-signature ${signingAsset?.kind === "stamp" ? "stamp" : ""}`}>{signingAsset?.dataUrl && <img src={signingAsset.dataUrl} alt={signingAsset.kind === "stamp" ? "电子章" : "电子签名"} />}<b>Authorized Signature / Stamp</b></span></footer>
  </div>;
}

function LineEditor({ line, packing, onChange }: { line: DocumentLineSnapshot; packing: boolean; onChange: (line: DocumentLineSnapshot) => void }) {
  const set = (patch: Partial<DocumentLineSnapshot>) => onChange({ ...line, ...patch });
  return <div className="document-line-form">
    <div className="document-line-name"><label>品名 / 描述<input value={line.description} onChange={(event) => set({ description: event.target.value })} /></label><span><strong>{line.sku}</strong> · {line.model || "无型号"} · HS {line.hsCode || "未填写"}</span></div>
    <label>数量<input type="number" min="0.001" step="0.001" value={line.quantity} onChange={(event) => set({ quantity: numberValue(event.target.value) })} /></label>
    <label>HS 编码<input value={line.hsCode} onChange={(event) => set({ hsCode: event.target.value })} /></label>
    {packing ? <>
      <label>箱数<input type="number" min="1" step="1" value={line.packages} onChange={(event) => set({ packages: numberValue(event.target.value) })} /></label>
      <label>包装<input value={line.packageType} onChange={(event) => set({ packageType: event.target.value })} /></label>
      <label>净重 kg<input type="number" min="0" step="0.001" value={line.netWeightKg} onChange={(event) => set({ netWeightKg: numberValue(event.target.value) })} /></label>
      <label>毛重 kg<input type="number" min="0" step="0.001" value={line.grossWeightKg} onChange={(event) => set({ grossWeightKg: numberValue(event.target.value) })} /></label>
      <label>CBM<input type="number" min="0" step="0.001" value={line.cbm} onChange={(event) => set({ cbm: numberValue(event.target.value) })} /></label>
    </> : <label>单价<input type="number" min="0" step="0.01" value={(line.unitPriceMinor / 100).toFixed(2)} onChange={(event) => set({ unitPriceMinor: Math.round(numberValue(event.target.value) * 100) })} /></label>}
  </div>;
}

function DocumentEditor({ initial, companyRegistry, onClose, onSave, onReview, onIssue, onExportPdf, onExportCsv, onPrint }: {
  initial: TradeDocument;
  companyRegistry: CompanyRegistry | null;
  onClose: () => void;
  onSave: (input: SaveDocumentInput) => Promise<TradeDocument>;
  onReview: (id: string) => Promise<TradeDocument>;
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
  const packing = ["packing_list", "shipping_marks", "shipper_instruction", "customs_declaration", "bill_of_lading", "certificate_of_origin", "inspection_certificate", "fumigation_certificate"].includes(document.documentType);
  const quotation = document.documentType === "commercial_quotation";
  const invoiceLike = document.documentType === "commercial_invoice" || document.documentType === "proforma_invoice";
  const shippingMarks = document.documentType === "shipping_marks";
  const shipperInstruction = document.documentType === "shipper_instruction";
  const customsDeclaration = document.documentType === "customs_declaration";
  const billOfLading = document.documentType === "bill_of_lading";
  const insurancePolicy = document.documentType === "insurance_policy";
  const certificateOfOrigin = document.documentType === "certificate_of_origin";
  const inspectionCertificate = document.documentType === "inspection_certificate";
  const fumigationCertificate = document.documentType === "fumigation_certificate";
  const beneficiaryCertificate = document.documentType === "beneficiary_certificate";
  const amountDocument = !["packing_list", "shipping_marks", "shipper_instruction", "bill_of_lading", "certificate_of_origin", "inspection_certificate", "fumigation_certificate", "beneficiary_certificate"].includes(document.documentType);
  const cargoValueMinor = payload.lines.reduce((sum, line) => sum + line.amountMinor, 0) - payload.discountMinor;
  const setPayloadField = (patch: Partial<DocumentPayload>) => setPayload((current) => ({ ...current, ...patch }));
  const updateInsurance = (insuredValueMinor: number, ratePercent = payload.premiumRatePercent) => setPayloadField({
    insuredValueMinor,
    premiumRatePercent: ratePercent,
    premiumMinor: Math.round(insuredValueMinor * ratePercent / 100),
  });
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
      const updated = await onIssue(document.id);
      await documentDraftApi.delete(document.id);
      setDocument(updated); setPayload(structuredClone(updated.payload)); setMessage("已签发并冻结为只读版本");
    } catch (reason) { setMessage(String(reason)); } finally { setBusy(""); }
  }

  async function review() {
    setBusy("review"); setMessage("");
    try {
      await finishPendingAutosave();
      const saved = await onSave({ id: document.id, number, issueDate, language, payload });
      const updated = await onReview(saved.id);
      await documentDraftApi.delete(document.id);
      setDocument(updated); setPayload(structuredClone(updated.payload)); setMessage("审核通过；确认无误后可签发冻结");
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
    <header className="document-editor-toolbar"><div><span className="eyebrow">{typeLabels[document.documentType]} · V{document.version}</span><h2>{document.number}</h2>{editable && autosaveState && <small className="autosave-state">{autosaveState}</small>}</div><div className="document-toolbar-actions">{editable && <button className="button button-secondary" disabled={!!busy} onClick={() => void save().catch(() => undefined)}>保存草稿</button>}{editable && <button className="button button-primary" disabled={!!busy} onClick={() => void review()}>提交审核</button>}{document.status === "reviewed" && <button className="button button-primary" disabled={!!busy} onClick={() => void issue()}>签发冻结</button>}<button className="button button-secondary" disabled={!!busy} onClick={() => void output("pdf")}>PDF</button><button className="button button-secondary" disabled={!!busy} onClick={() => void output("csv")}>CSV</button><button className="button button-secondary" disabled={!!busy} onClick={() => void output("print")}>打印</button><button className="icon-button" onClick={() => void closeEditor()}>×</button></div></header>
    {message && <div className="document-message">{message}</div>}
    <div className="document-editor-layout">
      <div className="document-form-panel">
        <div className="document-brand-selectors"><label>导出公司<select value={companyId} onChange={(event) => { setCompanyId(event.target.value); setSigningAssetId(""); }}>{companyRegistry?.companies.map((item) => <option value={item.id} key={item.id}>{item.companyName}</option>)}</select></label><label>签章（可不选）<select value={signingAssetId} onChange={(event) => setSigningAssetId(event.target.value)}><option value="">不使用签章</option>{selectedCompany?.signingAssets.map((item) => <option value={item.id} key={item.id}>{item.kind === "stamp" ? "电子章" : "电子签名"} · {item.name}</option>)}</select></label></div>
        <AttachmentPanel entityType="document" entityId={document.id} entityLabel={`${document.number} V${document.version}`} title="单证附件" />
        {!editable && <div className="locked-callout">该版本{statusLabels[document.status]}，内容只读；{document.status === "reviewed" ? "可直接签发，若需修改请创建新版本。" : "修改请创建新版本。"}</div>}
        {document.validationIssues.length > 0 && <div className="validation-list">{document.validationIssues.map((issue, index) => <span className={issue.severity} key={`${issue.code}-${index}`}>{issue.severity === "error" ? "错误" : "提醒"} · {issue.message}</span>)}</div>}
        <fieldset disabled={!editable}><div className="form-grid two-columns"><label>单证编号<input value={number} onChange={(event) => setNumber(event.target.value)} /></label><label>签发日期<input type="date" value={issueDate} onChange={(event) => setIssueDate(event.target.value)} /></label><label>输出语言<select value={language} onChange={(event) => setLanguage(event.target.value)}><option value="zh_en">中英双语</option><option value="en">英文</option><option value="ru">俄文</option></select></label><label>来源业务单<input value={document.businessCaseNumber} readOnly /></label></div>
        <h3>买卖双方</h3><div className="form-grid two-columns"><label>卖方/出口商<input value={payload.seller} onChange={(event) => setPayloadField({ seller: event.target.value })} /></label><label>买方/收货人<input value={payload.buyer} onChange={(event) => setPayloadField({ buyer: event.target.value })} /></label><label>卖方地址<textarea value={payload.sellerAddress} onChange={(event) => setPayloadField({ sellerAddress: event.target.value })} /></label><label>买方地址<textarea value={payload.buyerAddress} onChange={(event) => setPayloadField({ buyerAddress: event.target.value })} /></label></div>
        <h3>贸易与运输</h3><div className="form-grid two-columns"><label>原产国<input value={payload.originCountry} onChange={(event) => setPayloadField({ originCountry: event.target.value })} /></label><label>目的国<input value={payload.destinationCountry} onChange={(event) => setPayloadField({ destinationCountry: event.target.value })} /></label><label>装运港<input value={payload.portOfLoading} onChange={(event) => setPayloadField({ portOfLoading: event.target.value })} /></label><label>目的港<input value={payload.portOfDischarge} onChange={(event) => setPayloadField({ portOfDischarge: event.target.value })} /></label><label>贸易术语<input value={payload.incoterm} onChange={(event) => setPayloadField({ incoterm: event.target.value })} /></label><label>付款条款<input value={payload.paymentTerms} onChange={(event) => setPayloadField({ paymentTerms: event.target.value })} /></label><label>装运日期<input type="date" value={payload.shipmentDate} onChange={(event) => setPayloadField({ shipmentDate: event.target.value })} /></label><label>客户 PO<input value={payload.poReference} onChange={(event) => setPayloadField({ poReference: event.target.value })} /></label>{quotation && <label>报价有效期<input type="date" value={payload.validUntil} onChange={(event) => setPayloadField({ validUntil: event.target.value })} /></label>}</div>
        {(shippingMarks || shipperInstruction || customsDeclaration || billOfLading || insurancePolicy) && <><h3>履约资料</h3><div className="form-grid two-columns">{(shippingMarks || billOfLading) && <label className="field-wide">正唛内容<textarea rows={4} value={payload.shippingMarks} onChange={(event) => setPayloadField({ shippingMarks: event.target.value })} /></label>}{(shipperInstruction || customsDeclaration || billOfLading || insurancePolicy) && <label>运输方式<input value={payload.transportMode} onChange={(event) => setPayloadField({ transportMode: event.target.value })} /></label>}{(shipperInstruction || customsDeclaration || billOfLading || insurancePolicy) && <label>船名/航次<input value={payload.vesselVoyage} onChange={(event) => setPayloadField({ vesselVoyage: event.target.value })} /></label>}{(shipperInstruction || billOfLading) && <label>订舱参考号<input value={payload.bookingReference} onChange={(event) => setPayloadField({ bookingReference: event.target.value })} /></label>}{(shipperInstruction || billOfLading) && <label>运费条款<input value={payload.freightTerms} onChange={(event) => setPayloadField({ freightTerms: event.target.value })} /></label>}{(shipperInstruction || billOfLading) && <label>提单类型<select value={payload.billOfLadingType} onChange={(event) => setPayloadField({ billOfLadingType: event.target.value })}><option>Original B/L</option><option>Telex Release</option><option>Sea Waybill</option></select></label>}{billOfLading && <><label>提单号/草稿号<input value={payload.billOfLadingNumber} onChange={(event) => setPayloadField({ billOfLadingNumber: event.target.value })} /></label><label>承运人/船公司<input value={payload.carrier} onChange={(event) => setPayloadField({ carrier: event.target.value })} /></label><label>通知方<input value={payload.notifyParty} onChange={(event) => setPayloadField({ notifyParty: event.target.value })} /></label><label>通知方地址<input value={payload.notifyPartyAddress} onChange={(event) => setPayloadField({ notifyPartyAddress: event.target.value })} /></label><label>收货地<input value={payload.placeOfReceipt} onChange={(event) => setPayloadField({ placeOfReceipt: event.target.value })} /></label><label>交货地<input value={payload.placeOfDelivery} onChange={(event) => setPayloadField({ placeOfDelivery: event.target.value })} /></label><label>集装箱号<input value={payload.containerNumbers} onChange={(event) => setPayloadField({ containerNumbers: event.target.value })} /></label><label>封条号<input value={payload.sealNumbers} onChange={(event) => setPayloadField({ sealNumbers: event.target.value })} /></label></>}{insurancePolicy && <><label>保险公司<input value={payload.insuranceCompany} onChange={(event) => setPayloadField({ insuranceCompany: event.target.value })} /></label><label>保单号/申请号<input value={payload.policyNumber} onChange={(event) => setPayloadField({ policyNumber: event.target.value })} /></label><label>承保险别<input value={payload.insuranceCoverage} onChange={(event) => setPayloadField({ insuranceCoverage: event.target.value })} /></label><label>赔款偿付地点<input value={payload.claimsPayableAt} onChange={(event) => setPayloadField({ claimsPayableAt: event.target.value })} /></label><label>保险加成 %<input type="number" min="0" step="0.01" value={payload.insuranceMarkupPercent} onChange={(event) => { const markup = numberValue(event.target.value); const insured = Math.round(cargoValueMinor * (1 + markup / 100)); setPayloadField({ insuranceMarkupPercent: markup, insuredValueMinor: insured, premiumMinor: Math.round(insured * payload.premiumRatePercent / 100) }); }} /></label><label>保险金额（{document.currency}）<input type="number" min="0" step="0.01" value={(payload.insuredValueMinor / 100).toFixed(2)} onChange={(event) => updateInsurance(Math.round(numberValue(event.target.value) * 100))} /></label><label>保险费率 %<input type="number" min="0" step="0.0001" value={payload.premiumRatePercent} onChange={(event) => updateInsurance(payload.insuredValueMinor, numberValue(event.target.value))} /></label><label>预计保费（{document.currency}）<input value={(payload.premiumMinor / 100).toFixed(2)} readOnly /></label></>}{customsDeclaration && <label>监管方式代码<input value={payload.customsSupervisionCode} onChange={(event) => setPayloadField({ customsSupervisionCode: event.target.value })} /></label>}{customsDeclaration && <label className="field-wide">申报要素<input value={payload.customsDeclarationElements} onChange={(event) => setPayloadField({ customsDeclarationElements: event.target.value })} placeholder="品牌、用途、材质、规格等" /></label>}</div></>}
        {(certificateOfOrigin || inspectionCertificate || fumigationCertificate) && <><h3>认证资料</h3><div className="form-grid two-columns">
          <label>运输方式<input value={payload.transportMode} onChange={(event) => setPayloadField({ transportMode: event.target.value })} /></label><label>船名/航次<input value={payload.vesselVoyage} onChange={(event) => setPayloadField({ vesselVoyage: event.target.value })} /></label>
          <label>证书号/申请号<input value={payload.certificateNumber} onChange={(event) => setPayloadField({ certificateNumber: event.target.value })} /></label><label>{certificateOfOrigin ? "签证机构" : inspectionCertificate ? "检验机构" : "熏蒸服务/签证机构"}<input value={payload.certificationAuthority} onChange={(event) => setPayloadField({ certificationAuthority: event.target.value })} /></label>
          {certificateOfOrigin && <><label>证书类型<select value={payload.certificateType} onChange={(event) => setPayloadField({ certificateType: event.target.value })}><option>General Certificate of Origin</option><option>Form E</option><option>Form A</option><option>RCEP Certificate of Origin</option><option>China-Pakistan FTA Certificate</option><option>China-Chile FTA Certificate</option></select></label><label>原产国<input value={payload.originCountry} onChange={(event) => setPayloadField({ originCountry: event.target.value })} /></label><label className="field-wide">唛头<textarea rows={3} value={payload.shippingMarks} onChange={(event) => setPayloadField({ shippingMarks: event.target.value })} /></label></>}
          {inspectionCertificate && <><label>制造商<input value={payload.manufacturer} onChange={(event) => setPayloadField({ manufacturer: event.target.value })} /></label><label>制造商地址<input value={payload.manufacturerAddress} onChange={(event) => setPayloadField({ manufacturerAddress: event.target.value })} /></label><label>批次号<input value={payload.batchNumber} onChange={(event) => setPayloadField({ batchNumber: event.target.value })} /></label><label>检验标准<input value={payload.inspectionStandard} onChange={(event) => setPayloadField({ inspectionStandard: event.target.value })} /></label><label>检验日期<input type="date" value={payload.inspectionDate} onChange={(event) => setPayloadField({ inspectionDate: event.target.value })} /></label><label>检验地点<input value={payload.inspectionPlace} onChange={(event) => setPayloadField({ inspectionPlace: event.target.value })} /></label><label className="field-wide">检验结果<textarea rows={3} value={payload.inspectionResult} onChange={(event) => setPayloadField({ inspectionResult: event.target.value })} /></label></>}
          {fumigationCertificate && <><label>熏蒸剂<input value={payload.fumigationAgent} onChange={(event) => setPayloadField({ fumigationAgent: event.target.value })} /></label><label>处理方法<input value={payload.fumigationMethod} onChange={(event) => setPayloadField({ fumigationMethod: event.target.value })} /></label><label>处理温度 °C<input type="number" step="0.1" value={payload.fumigationTemperatureCelsius} onChange={(event) => setPayloadField({ fumigationTemperatureCelsius: numberValue(event.target.value) })} /></label><label>持续时间（小时）<input type="number" min="0" step="0.1" value={payload.fumigationDurationHours} onChange={(event) => setPayloadField({ fumigationDurationHours: numberValue(event.target.value) })} /></label><label>处理日期<input type="date" value={payload.fumigationDate} onChange={(event) => setPayloadField({ fumigationDate: event.target.value })} /></label><label>处理地点<input value={payload.fumigationPlace} onChange={(event) => setPayloadField({ fumigationPlace: event.target.value })} /></label><label>操作人员<input value={payload.fumigationOperator} onChange={(event) => setPayloadField({ fumigationOperator: event.target.value })} /></label><label>许可证号<input value={payload.fumigationLicenseNumber} onChange={(event) => setPayloadField({ fumigationLicenseNumber: event.target.value })} /></label><label className="field-wide">唛头<textarea rows={3} value={payload.shippingMarks} onChange={(event) => setPayloadField({ shippingMarks: event.target.value })} /></label></>}
        </div></>}
        {beneficiaryCertificate && <><h3>信用证与交单资料</h3><div className="form-grid two-columns">
          <label>运输方式<input value={payload.transportMode} onChange={(event) => setPayloadField({ transportMode: event.target.value })} /></label><label>船名/航次<input value={payload.vesselVoyage} onChange={(event) => setPayloadField({ vesselVoyage: event.target.value })} /></label><label>证明编号<input value={payload.certificateNumber} onChange={(event) => setPayloadField({ certificateNumber: event.target.value })} /></label><label>证明类型<input value={payload.beneficiaryCertificateType} onChange={(event) => setPayloadField({ beneficiaryCertificateType: event.target.value })} /></label>
          <label>信用证号码<input value={payload.letterOfCreditNumber} onChange={(event) => setPayloadField({ letterOfCreditNumber: event.target.value })} /></label><label>开证行<input value={payload.issuingBank} onChange={(event) => setPayloadField({ issuingBank: event.target.value })} /></label><label>开证日期<input type="date" value={payload.letterOfCreditIssueDate} onChange={(event) => setPayloadField({ letterOfCreditIssueDate: event.target.value })} /></label><label>信用证有效期<input type="date" value={payload.letterOfCreditExpiryDate} onChange={(event) => setPayloadField({ letterOfCreditExpiryDate: event.target.value })} /></label><label>交单截止日<input type="date" value={payload.presentationDeadline} onChange={(event) => setPayloadField({ presentationDeadline: event.target.value })} /></label>
          <label className="field-wide">受益人声明<textarea rows={4} value={payload.beneficiaryStatement} onChange={(event) => setPayloadField({ beneficiaryStatement: event.target.value })} /></label><label className="field-wide">信用证条款清单<textarea rows={4} value={payload.letterOfCreditTerms} onChange={(event) => setPayloadField({ letterOfCreditTerms: event.target.value })} /></label><label className="field-wide">所需交单文件<textarea rows={4} value={payload.requiredDocuments} onChange={(event) => setPayloadField({ requiredDocuments: event.target.value })} /></label>
        </div></>}
        <h3>产品明细</h3><div className="document-lines">{payload.lines.map((line, index) => <LineEditor line={line} packing={packing} onChange={(updated) => setPayloadField({ lines: payload.lines.map((item, itemIndex) => itemIndex === index ? updated : item) })} key={`${line.productId}-${index}`} />)}</div>
        {amountDocument && <label>折扣金额（{document.currency}）<input type="number" min="0" step="0.01" value={(payload.discountMinor / 100).toFixed(2)} onChange={(event) => { const discountMinor = Math.round(numberValue(event.target.value) * 100); if (insurancePolicy) { const value = payload.lines.reduce((sum, line) => sum + line.amountMinor, 0) - discountMinor; const insured = Math.round(value * (1 + payload.insuranceMarkupPercent / 100)); setPayloadField({ discountMinor, insuredValueMinor: insured, premiumMinor: Math.round(insured * payload.premiumRatePercent / 100) }); } else { setPayloadField({ discountMinor }); } }} /></label>}
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

  async function archiveDocument(document: TradeDocument) {
    if (!window.confirm(`请确认归档单证：${document.number}（版本 V${document.version}）。\n归档后仍可查看和导出。`)) return;
    try { await props.onArchive(document.id); setMessage(`${document.number} V${document.version} 已归档`); } catch (error) { setMessage(String(error)); }
  }

  async function newVersion(document: TradeDocument) {
    try { const created = await props.onNewVersion(document.id); setEditing(created); } catch (error) { setMessage(String(error)); }
  }

  if (editing) return <DocumentEditor initial={editing} companyRegistry={props.companyRegistry} onClose={() => setEditing(null)} onSave={props.onSave} onReview={props.onReview} onIssue={props.onIssue} onExportPdf={props.onExportPdf} onExportCsv={props.onExportCsv} onPrint={props.onPrint} />;

  return <section className="panel document-center">
    <div className="panel-heading"><div><h2>单证中心</h2><p>从报价、PI 到履约单证，复用同一份加密业务快照</p></div><button className="button button-primary" disabled={!props.cases.length} onClick={() => setCreating(true)}>新建单证</button></div>
    {!props.cases.length && <div className="empty-callout">请先建立报价阶段的业务单，再生成报价、形式发票、合同及出货单证。</div>}
    {message && <div className="document-message">{message}</div>}
    <div className="document-filters"><input placeholder="搜索单号、业务单或客户" value={query} onChange={(event) => setQuery(event.target.value)} /><select value={type} onChange={(event) => setType(event.target.value as typeof type)}><option value="all">全部类型</option>{documentTypes.map((item) => <option value={item} key={item}>{typeLabels[item]}</option>)}</select><select value={status} onChange={(event) => setStatus(event.target.value as typeof status)}><option value="all">全部状态</option><option value="draft">草稿</option><option value="reviewed">已审核</option><option value="issued">已签发</option><option value="voided">已作废</option><option value="archived">已归档</option></select><span>{filtered.length} 个版本</span></div>
    <div className="document-history-list">{filtered.map((document) => <article className="document-history-card" key={document.id}><div className="document-type-mark">{prefixes[document.documentType]}</div><div className="document-history-main"><span className="eyebrow">{typeLabels[document.documentType]} · {document.businessCaseNumber}</span><h3>{document.number} <small>V{document.version}</small></h3><p>{document.customerName} · {document.issueDate} · 模板 {document.templateVersion}</p><div className="document-chips"><span className={`document-status ${document.status}`}>{statusLabels[document.status]}</span>{document.validationIssues.filter((item) => item.severity === "error").length > 0 && <span className="validation-chip">{document.validationIssues.filter((item) => item.severity === "error").length} 个错误</span>}{document.pdfSha256 && <span>PDF {document.pdfSha256.slice(0, 10)}…</span>}</div></div><div className="document-card-actions"><button onClick={() => setEditing(document)}>{document.status === "draft" ? "编辑" : "查看"}</button>{document.pdfPath && <button onClick={() => void props.onOpenPdf(document.id)}>打开 PDF</button>}{document.status === "issued" && canConvertDocument(document.documentType) && <button onClick={() => setConverting(document)}>转换单证</button>}{document.status !== "draft" && document.status !== "archived" && <button onClick={() => void newVersion(document)}>创建新版本</button>}{document.status === "issued" && <button className="danger-link" onClick={() => void voidDocument(document)}>作废</button>}{document.status === "issued" && <button onClick={() => void archiveDocument(document)}>归档</button>}</div></article>)}</div>
    {!filtered.length && <div className="empty-table">{props.documents.length ? "没有符合筛选条件的单证" : "还没有历史单证，可从业务单生成第一份草稿"}</div>}
    {creating && <CreateDocumentModal documents={props.documents} cases={props.cases} onClose={() => setCreating(false)} onCreate={create} />}
    {converting && <ConvertDocumentModal source={converting} documents={props.documents} onClose={() => setConverting(null)} onConvert={convert} />}
  </section>;
}
