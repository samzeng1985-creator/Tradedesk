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
import { CurrencySelect } from "./currencies";

export type MasterTab = "products" | "configurable" | "components" | "customers" | "suppliers";
export type MasterRecord = Product | Customer | Supplier;
export type MasterInput = ProductInput | CustomerInput | SupplierInput;

interface MasterEditorProps {
  tab: MasterTab;
  record: MasterRecord | null;
  products: Product[];
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
      paymentTerms: item?.paymentTerms ?? "", address: item?.address ?? "",
      shippingAddress: item?.shippingAddress ?? "", billingAddress: item?.billingAddress ?? "",
      purchaseIntent: item?.purchaseIntent ?? "", customerAnalysis: item?.customerAnalysis ?? "",
      strengths: item?.strengths ?? "", weaknesses: item?.weaknesses ?? "",
      contacts: item?.contacts ?? "",
    };
  }
  const item = record as Supplier | null;
  return {
    id: item?.id ?? "", code: item?.code ?? "", legalName: item?.legalName ?? "",
    address: item?.address ?? "", contacts: item?.contacts ?? "",
    bankDetails: item?.bankDetails ?? "", currency: item?.currency ?? "CNY",
    paymentTerms: item?.paymentTerms ?? "", qualificationNotes: item?.qualificationNotes ?? "",
    leadTimeDays: String(item?.leadTimeDays ?? 0), onTimeRate: String(item?.onTimeRate ?? 0),
  };
}

export function MasterEditor({ tab, record, products, saving, onClose, onSave }: MasterEditorProps) {
  const [values, setValues] = useState(() => initialValues(tab, record));
  const [supplierTerms, setSupplierTerms] = useState(() =>
    tab === "suppliers" ? structuredClone((record as Supplier | null)?.productTerms ?? []) : [],
  );
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
          address: values.address, shippingAddress: values.shippingAddress,
          billingAddress: values.billingAddress, purchaseIntent: values.purchaseIntent,
          customerAnalysis: values.customerAnalysis, strengths: values.strengths,
          weaknesses: values.weaknesses, contacts: values.contacts,
        });
      } else {
        const selectedProducts = supplierTerms.map((term) => term.productId).filter(Boolean);
        if (new Set(selectedProducts).size !== selectedProducts.length) {
          setError("同一供应商不能重复添加同一个产品。");
          return;
        }
        await onSave({
          id: values.id || undefined, code: values.code, legalName: values.legalName,
          address: values.address, contacts: values.contacts, bankDetails: values.bankDetails,
          currency: values.currency, paymentTerms: values.paymentTerms,
          leadTimeDays: Number(values.leadTimeDays), onTimeRate: Number(values.onTimeRate),
          qualificationNotes: values.qualificationNotes, productTerms: supplierTerms,
        });
      }
    } catch (reason) {
      setError(String(reason));
    }
  }

  return (
    <div className="modal-backdrop" role="presentation" onMouseDown={onClose}>
      <section className={`modal-card ${tab === "customers" || tab === "suppliers" ? "customer-editor" : ""}`} role="dialog" aria-modal="true" aria-labelledby="editor-title" onMouseDown={(event) => event.stopPropagation()}>
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
            <div className="editor-section-heading field-wide"><h3>基本信息</h3><p>用于报价、订单和后续单证复用</p></div>
            <label>客户编码 *<input required value={values.code} onChange={(event) => set("code", event.target.value)} autoFocus /></label>
            <label>客户法定名称 *<input required value={values.legalName} onChange={(event) => set("legalName", event.target.value)} /></label>
            <label>国家/市场<input value={values.market} onChange={(event) => set("market", event.target.value)} placeholder="例如：美国" /></label>
            <label>默认币种 *<CurrencySelect value={values.currency} onChange={(value) => set("currency", value)} /></label>
            <label className="field-wide">默认付款条款<input value={values.paymentTerms} onChange={(event) => set("paymentTerms", event.target.value)} /></label>
            <label className="field-wide">客户地址<textarea rows={2} value={values.address} onChange={(event) => set("address", event.target.value)} placeholder="公司注册地址或主要办公地址" /></label>
            <label>收货地址<textarea rows={3} value={values.shippingAddress} onChange={(event) => set("shippingAddress", event.target.value)} placeholder="货物实际送达地址、仓库或港口信息" /></label>
            <label>账单地址<textarea rows={3} value={values.billingAddress} onChange={(event) => set("billingAddress", event.target.value)} placeholder="发票和账单使用的地址" /></label>

            <div className="editor-section-heading field-wide"><h3>购买意向</h3><p>记录客户关注的产品、采购计划和合作偏好</p></div>
            <label className="field-wide">购买意向<textarea rows={4} value={values.purchaseIntent} onChange={(event) => set("purchaseIntent", event.target.value)} placeholder="目标产品、规格、预计数量、预算、采购频率、期望交期等" /></label>

            <div className="editor-section-heading field-wide"><h3>客户分析</h3><p>沉淀客户背景、决策方式、成交条件和风险判断</p></div>
            <label className="field-wide">客户分析<textarea rows={4} value={values.customerAnalysis} onChange={(event) => set("customerAnalysis", event.target.value)} placeholder="业务规模、销售渠道、决策流程、信用情况、合作阶段和跟进建议" /></label>

            <div className="editor-section-heading field-wide"><h3>优劣势分析</h3><p>用于判断合作价值和需要提前控制的风险</p></div>
            <label>优势<textarea rows={4} value={values.strengths} onChange={(event) => set("strengths", event.target.value)} placeholder="渠道、规模、付款能力、增长潜力等" /></label>
            <label>劣势与风险<textarea rows={4} value={values.weaknesses} onChange={(event) => set("weaknesses", event.target.value)} placeholder="价格敏感、账期、认证要求、沟通或交付风险等" /></label>

            <div className="editor-section-heading field-wide"><h3>主要人员和联系方式</h3><p>每位联系人一行，便于快速复制和查找</p></div>
            <label className="field-wide">联系人<textarea rows={5} value={values.contacts} onChange={(event) => set("contacts", event.target.value)} placeholder={"姓名｜职务｜邮箱｜电话｜WhatsApp/微信\n例如：Jane Smith｜采购经理｜jane@example.com｜+1 206 555 0100｜WhatsApp 同号"} /></label>
          </>}
          {tab === "suppliers" && <>
            <div className="editor-section-heading field-wide"><h3>基本信息</h3><p>采购单和供应商评估会复用以下资料</p></div>
            <label>供应商编码 *<input required value={values.code} onChange={(event) => set("code", event.target.value)} autoFocus /></label>
            <label>供应商法定名称 *<input required value={values.legalName} onChange={(event) => set("legalName", event.target.value)} /></label>
            <label>默认币种 *<CurrencySelect value={values.currency} onChange={(value) => set("currency", value)} /></label>
            <label>默认付款条款<input value={values.paymentTerms} onChange={(event) => set("paymentTerms", event.target.value)} placeholder="例如：30% 预付，70% 发货前" /></label>
            <label>默认交期（天）<input type="number" min="0" value={values.leadTimeDays} onChange={(event) => set("leadTimeDays", event.target.value)} /></label>
            <label>准时率（%）<input type="number" min="0" max="100" value={values.onTimeRate} onChange={(event) => set("onTimeRate", event.target.value)} /></label>
            <label className="field-wide">地址<textarea rows={2} value={values.address} onChange={(event) => set("address", event.target.value)} /></label>
            <label className="field-wide">联系人<textarea rows={3} value={values.contacts} onChange={(event) => set("contacts", event.target.value)} placeholder="姓名｜职务｜电话｜邮箱" /></label>
            <label className="field-wide">银行资料<textarea rows={3} value={values.bankDetails} onChange={(event) => set("bankDetails", event.target.value)} placeholder="开户名、银行、账号、SWIFT 等；属于敏感资料" /></label>
            <label className="field-wide">资质、质量与评估备注<textarea rows={3} value={values.qualificationNotes} onChange={(event) => set("qualificationNotes", event.target.value)} /></label>

            <div className="editor-section-heading field-wide"><h3>供应产品与采购条件</h3><p>维护每个产品的采购价、MOQ 和交期，便于后续采购复用</p></div>
            <div className="field-wide supplier-product-terms">
              {supplierTerms.map((term, index) => <div className="supplier-product-term" key={term.id}>
                <label>产品 *<select required value={term.productId} onChange={(event) => {
                  const product = products.find((item) => item.id === event.target.value);
                  setSupplierTerms((current) => current.map((item, itemIndex) => itemIndex === index ? { ...item, productId: product?.id ?? "", productSku: product?.sku ?? "", productName: product?.nameEn || product?.nameZh || "" } : item));
                }}><option value="">请选择产品</option>{products.map((product) => <option value={product.id} key={product.id}>{product.sku} · {product.nameEn || product.nameZh}</option>)}</select></label>
                <label>币种 *<CurrencySelect value={term.currency} onChange={(value) => setSupplierTerms((current) => current.map((item, itemIndex) => itemIndex === index ? { ...item, currency: value } : item))} /></label>
                <label>采购单价 *<input required type="number" min="0.01" step="0.01" value={(term.unitPriceMinor / 100).toFixed(2)} onChange={(event) => setSupplierTerms((current) => current.map((item, itemIndex) => itemIndex === index ? { ...item, unitPriceMinor: Math.round(Number(event.target.value) * 100) } : item))} /></label>
                <label>MOQ *<input required type="number" min="0.001" step="0.001" value={term.moq} onChange={(event) => setSupplierTerms((current) => current.map((item, itemIndex) => itemIndex === index ? { ...item, moq: Number(event.target.value) } : item))} /></label>
                <label>交期（天）<input type="number" min="0" value={term.leadTimeDays} onChange={(event) => setSupplierTerms((current) => current.map((item, itemIndex) => itemIndex === index ? { ...item, leadTimeDays: Number(event.target.value) } : item))} /></label>
                <button type="button" className="danger-link supplier-term-remove" onClick={() => setSupplierTerms((current) => current.filter((_, itemIndex) => itemIndex !== index))}>移除</button>
              </div>)}
              <button type="button" className="button button-secondary" disabled={!products.length} onClick={() => setSupplierTerms((current) => [...current, { id: crypto.randomUUID(), productId: "", productSku: "", productName: "", currency: values.currency || "CNY", unitPriceMinor: 0, moq: 1, leadTimeDays: Number(values.leadTimeDays) || 0 }])}>添加供应产品</button>
              {!products.length && <small>请先录入产品，再维护供应商采购条件。</small>}
            </div>
          </>}
          {error && <div className="form-error field-wide" role="alert">{error}</div>}
          <div className="modal-actions field-wide"><button type="button" className="button button-secondary" onClick={onClose}>取消</button><button className="button button-primary" disabled={saving}>{saving ? "保存中…" : "保存"}</button></div>
        </form>
      </section>
    </div>
  );
}
