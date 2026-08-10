import { useEffect, useMemo, useState } from "react";
import type { ChangeEvent } from "react";
import type { CompanyRecord, CompanyRegistry, CompanyRegistryInput, CompanySigningAsset } from "./domain";

const acceptedImageTypes = ["image/png", "image/jpeg", "image/webp"];

function uid(prefix: string) {
  return `${prefix}-${globalThis.crypto?.randomUUID?.() ?? `${Date.now()}-${Math.random().toString(16).slice(2)}`}`;
}

function readImage(file: File): Promise<string> {
  if (!acceptedImageTypes.includes(file.type)) return Promise.reject(new Error("仅支持 PNG、JPG 或 WebP 图片"));
  if (file.size > 3 * 1024 * 1024) return Promise.reject(new Error("每张图片不能超过 3 MB"));
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result || ""));
    reader.onerror = () => reject(new Error("无法读取所选图片"));
    reader.readAsDataURL(file);
  });
}

function AssetField({ title, hint, value, onChange }: { title: string; hint: string; value: string; onChange: (value: string) => void }) {
  const [error, setError] = useState("");
  async function choose(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    event.target.value = "";
    if (!file) return;
    try { onChange(await readImage(file)); setError(""); } catch (reason) { setError(String(reason)); }
  }
  return <section className="brand-asset-card">
    <div className="brand-asset-preview">{value ? <img src={value} alt={`${title}预览`} /> : <span>尚未上传</span>}</div>
    <div><h3>{title}</h3><p>{hint}</p><div className="brand-asset-actions">
      <label className="button button-secondary">选择图片<input className="sr-only" type="file" accept="image/png,image/jpeg,image/webp" onChange={choose} /></label>
      {value && <button type="button" className="danger-link" onClick={() => onChange("")}>移除</button>}
    </div>{error && <div className="form-error">{error}</div>}</div>
  </section>;
}

export function CompanySettings({ registry, onSave }: { registry: CompanyRegistry; onSave: (input: CompanyRegistryInput) => Promise<void> }) {
  const [values, setValues] = useState(registry);
  const [selectedId, setSelectedId] = useState(registry.defaultCompanyId);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState("");
  useEffect(() => { setValues(registry); setSelectedId(registry.defaultCompanyId); }, [registry]);

  const company = useMemo(() => values.companies.find((item) => item.id === selectedId) ?? values.companies[0], [values, selectedId]);
  function updateCompany(patch: Partial<CompanyRecord>) {
    if (!company) return;
    setValues((current) => ({ ...current, companies: current.companies.map((item) => item.id === company.id ? { ...item, ...patch } : item) }));
  }
  function updateAsset(assetId: string, patch: Partial<CompanySigningAsset>) {
    updateCompany({ signingAssets: company.signingAssets.map((asset) => asset.id === assetId ? { ...asset, ...patch } : asset) });
  }
  function addCompany() {
    const id = uid("company");
    setValues((current) => ({ ...current, companies: [...current.companies, { id, companyName: "新公司", logoDataUrl: "", signingAssets: [] }] }));
    setSelectedId(id);
  }
  function removeCompany() {
    if (!company || values.companies.length === 1 || !window.confirm(`删除公司“${company.companyName}”及其签章设置？`)) return;
    const companies = values.companies.filter((item) => item.id !== company.id);
    const defaultCompanyId = values.defaultCompanyId === company.id ? companies[0].id : values.defaultCompanyId;
    setValues({ defaultCompanyId, companies });
    setSelectedId(defaultCompanyId);
  }
  function addAsset(kind: "signature" | "stamp") {
    updateCompany({ signingAssets: [...company.signingAssets, { id: uid(kind), name: kind === "stamp" ? "电子章" : "电子签名", kind, dataUrl: "" }] });
  }
  async function submit(event: React.FormEvent) {
    event.preventDefault(); setSaving(true); setMessage("");
    try {
      await onSave({ ...values, companies: values.companies.map((item) => ({ ...item, companyName: item.companyName.trim(), signingAssets: item.signingAssets.map((asset) => ({ ...asset, name: asset.name.trim() })) })) });
      setMessage("公司、Logo 与签章已保存。导出单证时可自由选择公司，并可选择不盖章、电子签名或电子章。");
    } catch (reason) { setMessage(String(reason)); } finally { setSaving(false); }
  }

  return <form className="panel company-settings" onSubmit={submit}>
    <div className="panel-heading"><div><h2>多公司与电子签章</h2><p>资料保存在 SQLCipher 加密工作区中；每次导出可选择对应公司及签章。</p></div><button className="button button-primary" disabled={saving || values.companies.some((item) => !item.companyName.trim())}>{saving ? "保存中…" : "保存设置"}</button></div>
    {message && <div className="document-message">{message}</div>}
    <div className="company-registry-layout">
      <aside className="company-registry-list"><div className="company-list-heading"><strong>出口公司</strong><button type="button" onClick={addCompany}>+ 新增</button></div>{values.companies.map((item) => <button type="button" className={item.id === company?.id ? "active" : ""} onClick={() => setSelectedId(item.id)} key={item.id}><span>{item.companyName || "未命名公司"}</span>{values.defaultCompanyId === item.id && <small>默认</small>}</button>)}</aside>
      {company && <div className="company-registry-editor">
        <div className="company-editor-actions"><button type="button" className="button button-secondary" onClick={() => setValues((current) => ({ ...current, defaultCompanyId: company.id }))} disabled={values.defaultCompanyId === company.id}>{values.defaultCompanyId === company.id ? "当前默认公司" : "设为默认公司"}</button><button type="button" className="danger-link" disabled={values.companies.length === 1} onClick={removeCompany}>删除公司</button></div>
        <label className="company-name-field"><span>公司名称</span><input required maxLength={160} value={company.companyName} onChange={(event) => updateCompany({ companyName: event.target.value })} placeholder="用于单证卖方名称和页脚" /></label>
        <AssetField title="公司 Logo" hint="建议透明背景 PNG，横向比例约 3:1；不超过 3 MB。" value={company.logoDataUrl} onChange={(logoDataUrl) => updateCompany({ logoDataUrl })} />
        <div className="signing-assets-heading"><div><h3>电子签名与电子章</h3><p>同一公司可保存多个资产，导出时按单证选择，也可不选择。</p></div><div><button type="button" className="button button-secondary" onClick={() => addAsset("signature")}>+ 电子签名</button><button type="button" className="button button-secondary" onClick={() => addAsset("stamp")}>+ 电子章</button></div></div>
        <div className="signing-assets-list">{company.signingAssets.map((asset) => <div className="signing-asset-row" key={asset.id}><div className="signing-asset-fields"><label>名称<input required value={asset.name} onChange={(event) => updateAsset(asset.id, { name: event.target.value })} /></label><label>类型<select value={asset.kind} onChange={(event) => updateAsset(asset.id, { kind: event.target.value as "signature" | "stamp" })}><option value="signature">电子签名</option><option value="stamp">电子章</option></select></label></div><AssetField title={asset.kind === "stamp" ? "印章图片" : "签名图片"} hint="建议透明背景 PNG；印章将按方形显示，签名按横向显示。" value={asset.dataUrl} onChange={(dataUrl) => updateAsset(asset.id, { dataUrl })} /><button type="button" className="danger-link" onClick={() => updateCompany({ signingAssets: company.signingAssets.filter((item) => item.id !== asset.id) })}>移除此签章</button></div>)}</div>
        {!company.signingAssets.length && <div className="empty-callout">尚未添加签章。PDF 仍可正常导出，并保留签章位置。</div>}
      </div>}
    </div>
  </form>;
}
