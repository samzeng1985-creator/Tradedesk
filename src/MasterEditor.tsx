import { useState } from "react";
import type { FormEvent } from "react";
import type {
  Customer,
  CustomerInput,
  Product,
  ProductInput,
  Supplier,
  SupplierInput,
} from "./domain";

export type MasterTab = "products" | "customers" | "suppliers";
export type MasterRecord = Product | Customer | Supplier;
export type MasterInput = ProductInput | CustomerInput | SupplierInput;

interface MasterEditorProps {
  tab: MasterTab;
  record: MasterRecord | null;
  saving: boolean;
  onClose: () => void;
  onSave: (input: MasterInput) => Promise<void>;
}

function initialValues(tab: MasterTab, record: MasterRecord | null): Record<string, string> {
  if (tab === "products") {
    const item = record as Product | null;
    return {
      id: item?.id ?? "", sku: item?.sku ?? "", nameZh: item?.nameZh ?? "",
      nameEn: item?.nameEn ?? "", model: item?.model ?? "", hsCode: item?.hsCode ?? "",
      unit: item?.unit ?? "pcs", grossWeightKg: String(item?.grossWeightKg ?? 0),
    };
  }
  if (tab === "customers") {
    const item = record as Customer | null;
    return {
      id: item?.id ?? "", code: item?.code ?? "", legalName: item?.legalName ?? "",
      market: item?.market ?? "", currency: item?.currency ?? "USD",
      paymentTerms: item?.paymentTerms ?? "",
    };
  }
  const item = record as Supplier | null;
  return {
    id: item?.id ?? "", code: item?.code ?? "", legalName: item?.legalName ?? "",
    leadTimeDays: String(item?.leadTimeDays ?? 0), onTimeRate: String(item?.onTimeRate ?? 0),
  };
}

export function MasterEditor({ tab, record, saving, onClose, onSave }: MasterEditorProps) {
  const [values, setValues] = useState(() => initialValues(tab, record));
  const [error, setError] = useState("");
  const set = (key: string, value: string) => setValues((current) => ({ ...current, [key]: value }));

  async function submit(event: FormEvent) {
    event.preventDefault();
    setError("");
    try {
      if (tab === "products") {
        await onSave({
          id: values.id || undefined, sku: values.sku, nameZh: values.nameZh,
          nameEn: values.nameEn, model: values.model, hsCode: values.hsCode,
          unit: values.unit, grossWeightKg: Number(values.grossWeightKg),
        });
      } else if (tab === "customers") {
        await onSave({
          id: values.id || undefined, code: values.code, legalName: values.legalName,
          market: values.market, currency: values.currency, paymentTerms: values.paymentTerms,
        });
      } else {
        await onSave({
          id: values.id || undefined, code: values.code, legalName: values.legalName,
          leadTimeDays: Number(values.leadTimeDays), onTimeRate: Number(values.onTimeRate),
        });
      }
    } catch (reason) {
      setError(String(reason));
    }
  }

  return (
    <div className="modal-backdrop" role="presentation" onMouseDown={onClose}>
      <section className="modal-card" role="dialog" aria-modal="true" aria-labelledby="editor-title" onMouseDown={(event) => event.stopPropagation()}>
        <div className="panel-heading">
          <div><span className="eyebrow">主数据</span><h2 id="editor-title">{record ? "编辑" : "新建"}{tab === "products" ? "产品" : tab === "customers" ? "客户" : "供应商"}</h2></div>
          <button className="icon-button" onClick={onClose} aria-label="关闭">×</button>
        </div>
        <form className="editor-form" onSubmit={submit}>
          {tab === "products" && <>
            <label>SKU *<input required value={values.sku} onChange={(event) => set("sku", event.target.value)} autoFocus /></label>
            <label>英文名称 *<input required value={values.nameEn} onChange={(event) => set("nameEn", event.target.value)} /></label>
            <label>中文名称<input value={values.nameZh} onChange={(event) => set("nameZh", event.target.value)} /></label>
            <label>型号<input value={values.model} onChange={(event) => set("model", event.target.value)} /></label>
            <label>HS 编码<input value={values.hsCode} onChange={(event) => set("hsCode", event.target.value)} /></label>
            <label>单位 *<input required value={values.unit} onChange={(event) => set("unit", event.target.value)} /></label>
            <label>单件毛重（kg）<input type="number" min="0" step="0.001" value={values.grossWeightKg} onChange={(event) => set("grossWeightKg", event.target.value)} /></label>
          </>}
          {tab === "customers" && <>
            <label>客户编码 *<input required value={values.code} onChange={(event) => set("code", event.target.value)} autoFocus /></label>
            <label>客户法定名称 *<input required value={values.legalName} onChange={(event) => set("legalName", event.target.value)} /></label>
            <label>国家/市场<input value={values.market} onChange={(event) => set("market", event.target.value)} placeholder="例如：美国" /></label>
            <label>默认币种 *<input required maxLength={3} value={values.currency} onChange={(event) => set("currency", event.target.value.toUpperCase())} /></label>
            <label className="field-wide">默认付款条款<input value={values.paymentTerms} onChange={(event) => set("paymentTerms", event.target.value)} /></label>
          </>}
          {tab === "suppliers" && <>
            <label>供应商编码 *<input required value={values.code} onChange={(event) => set("code", event.target.value)} autoFocus /></label>
            <label>供应商法定名称 *<input required value={values.legalName} onChange={(event) => set("legalName", event.target.value)} /></label>
            <label>默认交期（天）<input type="number" min="0" value={values.leadTimeDays} onChange={(event) => set("leadTimeDays", event.target.value)} /></label>
            <label>准时率（%）<input type="number" min="0" max="100" value={values.onTimeRate} onChange={(event) => set("onTimeRate", event.target.value)} /></label>
          </>}
          {error && <div className="form-error field-wide" role="alert">{error}</div>}
          <div className="modal-actions field-wide"><button type="button" className="button button-secondary" onClick={onClose}>取消</button><button className="button button-primary" disabled={saving}>{saving ? "保存中…" : "保存"}</button></div>
        </form>
      </section>
    </div>
  );
}
