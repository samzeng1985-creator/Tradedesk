import { useEffect, useState } from "react";
import type { ChangeEvent } from "react";
import type { CompanyProfile, CompanyProfileInput } from "./domain";

const acceptedImageTypes = ["image/png", "image/jpeg", "image/webp"];

function readImage(file: File): Promise<string> {
  if (!acceptedImageTypes.includes(file.type)) {
    return Promise.reject(new Error("仅支持 PNG、JPG 或 WebP 图片"));
  }
  if (file.size > 3 * 1024 * 1024) {
    return Promise.reject(new Error("每张图片不能超过 3 MB"));
  }
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result || ""));
    reader.onerror = () => reject(new Error("无法读取所选图片"));
    reader.readAsDataURL(file);
  });
}

function AssetField({
  title,
  hint,
  value,
  onChange,
}: {
  title: string;
  hint: string;
  value: string;
  onChange: (value: string) => void;
}) {
  const [error, setError] = useState("");

  async function choose(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    event.target.value = "";
    if (!file) return;
    try {
      onChange(await readImage(file));
      setError("");
    } catch (reason) {
      setError(String(reason));
    }
  }

  return <section className="brand-asset-card">
    <div className="brand-asset-preview">
      {value ? <img src={value} alt={`${title}预览`} /> : <span>尚未上传</span>}
    </div>
    <div>
      <h3>{title}</h3>
      <p>{hint}</p>
      <div className="brand-asset-actions">
        <label className="button button-secondary">
          选择图片
          <input className="sr-only" type="file" accept="image/png,image/jpeg,image/webp" onChange={choose} />
        </label>
        {value && <button type="button" className="danger-link" onClick={() => onChange("")}>移除</button>}
      </div>
      {error && <div className="form-error">{error}</div>}
    </div>
  </section>;
}

export function CompanySettings({
  profile,
  onSave,
}: {
  profile: CompanyProfile;
  onSave: (input: CompanyProfileInput) => Promise<void>;
}) {
  const [values, setValues] = useState(profile);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState("");

  useEffect(() => setValues(profile), [profile]);

  async function submit(event: React.FormEvent) {
    event.preventDefault();
    setSaving(true);
    setMessage("");
    try {
      await onSave({ ...values, companyName: values.companyName.trim() });
      setMessage("企业资料已保存，新的 PDF 导出将自动使用这些内容。");
    } catch (reason) {
      setMessage(String(reason));
    } finally {
      setSaving(false);
    }
  }

  return <form className="panel company-settings" onSubmit={submit}>
    <div className="panel-heading">
      <div><h2>企业资料与单证标识</h2><p>资料保存在 SQLCipher 加密工作区中，并统一用于配置单和单证 PDF。</p></div>
      <button className="button button-primary" disabled={saving || !values.companyName.trim()}>{saving ? "保存中…" : "保存设置"}</button>
    </div>
    {message && <div className="document-message">{message}</div>}
    <label className="company-name-field">
      <span>公司名称</span>
      <input required maxLength={160} value={values.companyName} onChange={(event) => setValues({ ...values, companyName: event.target.value })} placeholder="用于单证卖方名称和页脚" />
      <small>页脚格式：公司名称 - 单证编号 - 版本/页码。</small>
    </label>
    <div className="brand-assets-grid">
      <AssetField title="公司 Logo" hint="建议使用透明背景 PNG，横向比例约 3:1；每张不超过 3 MB。" value={values.logoDataUrl} onChange={(logoDataUrl) => setValues({ ...values, logoDataUrl })} />
      <AssetField title="电子签名" hint="建议使用透明背景 PNG，仅保留签名或签章图像。" value={values.signatureDataUrl} onChange={(signatureDataUrl) => setValues({ ...values, signatureDataUrl })} />
    </div>
    <p className="settings-note">未上传图片时仍可正常导出，系统会保留文字签名位置；上传后将自动嵌入 PDF。</p>
  </form>;
}
