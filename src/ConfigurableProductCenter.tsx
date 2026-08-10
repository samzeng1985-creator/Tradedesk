import { useMemo, useState } from "react";
import type { FormEvent } from "react";
import type {
  ComponentOption,
  ComponentOptionInput,
  ComponentOptionKind,
  ComponentOptionTranslationInput,
  ConfigurationLanguage,
  ConfigComponent,
  ConfigComponentInput,
  ConfigurableProduct,
  ConfigurableProductInput,
} from "./domain";

function formatMoney(valueMinor: number, currency: string) {
  return new Intl.NumberFormat("zh-CN", {
    style: "currency",
    currency,
    minimumFractionDigits: 2,
  }).format(valueMinor / 100);
}

interface ComponentLibraryProps {
  components: ConfigComponent[];
  options: ComponentOption[];
  onSave: (input: ConfigComponentInput) => Promise<void>;
  onArchive: (id: string) => Promise<void>;
  onSaveOption: (input: ComponentOptionInput) => Promise<void>;
  onSaveOptionTranslation: (input: ComponentOptionTranslationInput) => Promise<void>;
  onArchiveOption: (id: string) => Promise<void>;
}

const optionLabels: Record<ComponentOptionKind, string> = {
  category: "组件类别",
  name: "品名",
  brand: "品牌",
  specification: "规格/材质",
  unit: "单位",
  notes: "组件备注",
  product_name: "配置产品名",
  configuration_notes: "配置说明",
};

export const configurationLanguageLabels: Record<ConfigurationLanguage, string> = {
  en: "English · 英语",
  ru: "Русский · 俄语",
  fr: "Français · 法语",
  es: "Español · 西班牙语",
  pt: "Português · 葡萄牙语",
  ar: "العربية · 阿拉伯语",
};

function OptionTranslationRow({
  option,
  language,
  onSave,
  onArchive,
}: {
  option: ComponentOption;
  language: ConfigurationLanguage;
  onSave: (input: ComponentOptionTranslationInput) => Promise<void>;
  onArchive: (option: ComponentOption) => Promise<void>;
}) {
  const [value, setValue] = useState(option.translations[language] ?? "");
  const [busy, setBusy] = useState(false);

  async function save() {
    if (!value.trim()) return;
    setBusy(true);
    try {
      await onSave({ optionId: option.id, language, value });
    } finally {
      setBusy(false);
    }
  }

  return <div className="option-translation-row">
    <span title={option.value}>{option.value}</span>
    <input value={value} onChange={(event) => setValue(event.target.value)} placeholder={`${configurationLanguageLabels[language]}译文`} dir={language === "ar" ? "rtl" : "ltr"} />
    <button type="button" className="text-button" disabled={busy || !value.trim()} onClick={() => void save()}>{busy ? "保存中…" : "保存译文"}</button>
    <button type="button" className="danger-link" onClick={() => void onArchive(option)}>停用</button>
  </div>;
}

function RememberedInput({
  label,
  value,
  suggestions,
  required,
  placeholder,
  onChange,
}: {
  label: string;
  value: string;
  suggestions: string[];
  required?: boolean;
  placeholder?: string;
  onChange: (value: string) => void;
}) {
  const normalized = value.trim().toLocaleLowerCase();
  const matches = suggestions
    .filter((item) => !normalized || item.toLocaleLowerCase().includes(normalized))
    .slice(0, 8);
  const [focused, setFocused] = useState(false);

  return <label className="remembered-field">{label}{required ? " *" : ""}
    <input
      required={required}
      value={value}
      placeholder={placeholder}
      autoComplete="off"
      onFocus={() => setFocused(true)}
      onBlur={() => setFocused(false)}
      onChange={(event) => onChange(event.target.value)}
    />
    {focused && matches.length > 0 && <span className="remembered-suggestions" role="listbox">
      {matches.map((item) => <button type="button" role="option" key={item} onMouseDown={(event) => event.preventDefault()} onClick={() => { onChange(item); setFocused(false); }}>{item}</button>)}
    </span>}
  </label>;
}

function ComponentOptionManager({
  options,
  onClose,
  onSave,
  onSaveTranslation,
  onArchive,
}: {
  options: ComponentOption[];
  onClose: () => void;
  onSave: (input: ComponentOptionInput) => Promise<void>;
  onSaveTranslation: (input: ComponentOptionTranslationInput) => Promise<void>;
  onArchive: (id: string) => Promise<void>;
}) {
  const [kind, setKind] = useState<ComponentOptionKind>("category");
  const [language, setLanguage] = useState<ConfigurationLanguage>("en");
  const [value, setValue] = useState("");
  const [query, setQuery] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const normalized = query.trim().toLocaleLowerCase();
  const filtered = options.filter((item) => item.kind === kind && item.value.toLocaleLowerCase().includes(normalized));

  async function submit(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setError("");
    try {
      await onSave({ kind, value });
      setValue("");
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(false);
    }
  }

  async function archive(option: ComponentOption) {
    if (!window.confirm(`停用词库选项“${option.value}”？现有组件资料不会改变。`)) return;
    await onArchive(option.id);
  }

  return <div className="modal-backdrop" role="presentation" onMouseDown={onClose}>
    <section className="modal-card option-manager" role="dialog" aria-modal="true" onMouseDown={(event) => event.stopPropagation()}>
      <div className="panel-heading"><div><span className="eyebrow">多语录入词库</span><h2>组件与配置术语设置</h2><p>先选择输出语种，再为中文基础词录入经过确认的专业译文。</p></div><button className="icon-button" onClick={onClose} aria-label="关闭">×</button></div>
      <label className="option-language">译文语种<select value={language} onChange={(event) => setLanguage(event.target.value as ConfigurationLanguage)}>{Object.entries(configurationLanguageLabels).map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label>
      <div className="option-kind-tabs">{(Object.keys(optionLabels) as ComponentOptionKind[]).map((item) => <button type="button" className={kind === item ? "selected" : ""} key={item} onClick={() => { setKind(item); setQuery(""); }}>{optionLabels[item]} {options.filter((option) => option.kind === item).length}</button>)}</div>
      <form className="option-add-form" onSubmit={submit}><input required value={value} onChange={(event) => setValue(event.target.value)} placeholder={`新增${optionLabels[kind]}`} /><button className="button button-primary" disabled={busy}>{busy ? "保存中…" : "加入词库"}</button></form>
      <input className="option-search" value={query} onChange={(event) => setQuery(event.target.value)} placeholder={`模糊搜索${optionLabels[kind]}`} />
      {error && <div className="form-error">{error}</div>}
      <div className="option-list option-translation-list">{filtered.map((option) => <OptionTranslationRow key={`${option.id}-${language}-${option.translations[language] ?? ""}`} option={option} language={language} onSave={onSaveTranslation} onArchive={archive} />)}</div>
      {!filtered.length && <div className="empty-table">暂无符合条件的选项</div>}
    </section>
  </div>;
}

function ComponentEditor({
  record,
  options,
  onClose,
  onSave,
}: {
  record: ConfigComponent | null;
  options: ComponentOption[];
  onClose: () => void;
  onSave: (input: ConfigComponentInput) => Promise<void>;
}) {
  const [values, setValues] = useState(() => ({
    code: record?.code ?? "",
    category: record?.category ?? "",
    name: record?.name ?? "",
    specification: record?.specification ?? "",
    defaultQuantity: String(record?.defaultQuantity ?? 1),
    unit: record?.unit ?? "套",
    unitPrice: record ? String(record.unitPriceMinor / 100) : "0",
    currency: record?.currency ?? "CNY",
    brand: record?.brand ?? "",
    notes: record?.notes ?? "",
  }));
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");
  const set = (key: keyof typeof values, value: string) =>
    setValues((current) => ({ ...current, [key]: value }));
  const suggestions = (kind: ComponentOptionKind) => options.filter((item) => item.kind === kind).map((item) => item.value);

  async function submit(event: FormEvent) {
    event.preventDefault();
    setSaving(true);
    setError("");
    try {
      await onSave({
        id: record?.id,
        code: values.code,
        category: values.category,
        name: values.name,
        specification: values.specification,
        defaultQuantity: Number(values.defaultQuantity),
        unit: values.unit,
        unitPriceMinor: Math.round(Number(values.unitPrice) * 100),
        currency: values.currency,
        brand: values.brand,
        notes: values.notes,
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
      <section className="modal-card component-editor" role="dialog" aria-modal="true" onMouseDown={(event) => event.stopPropagation()}>
        <div className="panel-heading"><div><span className="eyebrow">组件库</span><h2>{record ? "编辑组件" : "新建组件"}</h2></div><button className="icon-button" onClick={onClose} aria-label="关闭">×</button></div>
        <form className="editor-form" onSubmit={submit}>
          <label>组件编号 *<input required value={values.code} onChange={(event) => set("code", event.target.value)} autoFocus /></label>
          <RememberedInput label="组件类别" required value={values.category} suggestions={suggestions("category")} onChange={(value) => set("category", value)} placeholder="输入关键字，例如：冷却" />
          <RememberedInput label="品名" required value={values.name} suggestions={suggestions("name")} onChange={(value) => set("name", value)} placeholder="输入关键字搜索历史品名" />
          <RememberedInput label="品牌" value={values.brand} suggestions={suggestions("brand")} onChange={(value) => set("brand", value)} placeholder="输入关键字搜索品牌" />
          <label className="field-wide">型号 / 规格 / 材质<textarea rows={3} value={values.specification} onChange={(event) => set("specification", event.target.value)} /></label>
          <label>默认数量 *<input required type="number" min="0.001" step="0.001" value={values.defaultQuantity} onChange={(event) => set("defaultQuantity", event.target.value)} /></label>
          <label>单位 *<input required value={values.unit} onChange={(event) => set("unit", event.target.value)} /></label>
          <label>单价 *<input required type="number" min="0" step="0.01" value={values.unitPrice} onChange={(event) => set("unitPrice", event.target.value)} /></label>
          <label>币种 *<input required maxLength={3} value={values.currency} onChange={(event) => set("currency", event.target.value.toUpperCase())} /></label>
          <label className="field-wide">备注<textarea rows={3} value={values.notes} onChange={(event) => set("notes", event.target.value)} placeholder="例如：含在机组报价中、业主现场采购、不包含" /></label>
          {error && <div className="form-error field-wide">{error}</div>}
          <div className="modal-actions field-wide"><button type="button" className="button button-secondary" onClick={onClose}>取消</button><button className="button button-primary" disabled={saving}>{saving ? "保存中…" : "保存组件"}</button></div>
        </form>
      </section>
    </div>
  );
}

export function ComponentLibrary({ components, options, onSave, onArchive, onSaveOption, onSaveOptionTranslation, onArchiveOption }: ComponentLibraryProps) {
  const [query, setQuery] = useState("");
  const [editing, setEditing] = useState<ConfigComponent | "new" | null>(null);
  const [managingOptions, setManagingOptions] = useState(false);
  const normalized = query.trim().toLocaleLowerCase();
  const filtered = components.filter((item) =>
    [item.code, item.category, item.name, item.specification, item.brand, item.notes]
      .some((value) => value.toLocaleLowerCase().includes(normalized)),
  );

  async function archive(item: ConfigComponent) {
    if (!window.confirm(`停用组件“${item.name}”？已保存配置中的组件快照不会改变。`)) return;
    await onArchive(item.id);
  }

  return <>
    <div className="table-toolbar"><label><span className="sr-only">搜索组件</span><input placeholder="搜索组件编号、类别、品名、规格或品牌" value={query} onChange={(event) => setQuery(event.target.value)} /></label><div className="toolbar-buttons"><button className="button button-secondary" onClick={() => setManagingOptions(true)}>词库设置</button><button className="button button-primary" onClick={() => setEditing("new")}>新建组件</button></div></div>
    <div className="table-wrap"><table><thead><tr><th>组件编号</th><th>类别</th><th>品名</th><th>型号/规格/材质</th><th>默认数量</th><th>单价</th><th>品牌</th><th>备注</th><th>操作</th></tr></thead><tbody>{filtered.map((item) => <tr key={item.id}><td>{item.code}</td><td>{item.category}</td><td><strong>{item.name}</strong></td><td>{item.specification || "—"}</td><td>{item.defaultQuantity} {item.unit}</td><td>{formatMoney(item.unitPriceMinor, item.currency)}</td><td>{item.brand || "—"}</td><td>{item.notes || "—"}</td><td><div className="row-actions"><button onClick={() => setEditing(item)}>编辑</button><button onClick={() => void archive(item)}>停用</button></div></td></tr>)}</tbody></table></div>
    {!filtered.length && <div className="empty-table">{components.length ? "没有符合条件的组件" : "还没有组件，请先录入可选组件和价格"}</div>}
    {editing && <ComponentEditor record={editing === "new" ? null : editing} options={options} onClose={() => setEditing(null)} onSave={onSave} />}
    {managingOptions && <ComponentOptionManager options={options} onClose={() => setManagingOptions(false)} onSave={onSaveOption} onSaveTranslation={onSaveOptionTranslation} onArchive={onArchiveOption} />}
  </>;
}

interface DraftLine {
  componentId: string;
  quantity: string;
  unitPrice: string;
  snapshot?: ConfigurableProduct["lines"][number];
}

function ConfigurationEditor({
  record,
  components,
  options,
  onClose,
  onSave,
}: {
  record: ConfigurableProduct | null;
  components: ConfigComponent[];
  options: ComponentOption[];
  onClose: () => void;
  onSave: (input: ConfigurableProductInput) => Promise<void>;
}) {
  const [code, setCode] = useState(record?.code ?? "");
  const [name, setName] = useState(record?.name ?? "");
  const [model, setModel] = useState(record?.model ?? "");
  const [currency, setCurrency] = useState(record?.currency ?? "CNY");
  const [notes, setNotes] = useState(record?.notes ?? "");
  const [lines, setLines] = useState<DraftLine[]>(() => record?.lines.map((line) => ({ componentId: line.componentId, quantity: String(line.quantity), unitPrice: String(line.unitPriceMinor / 100), snapshot: line })) ?? []);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");

  const totalMinor = useMemo(() => lines.reduce((sum, line) => sum + Math.round(Number(line.quantity || 0) * Number(line.unitPrice || 0) * 100), 0), [lines]);
  const componentFor = (line: DraftLine) => components.find((item) => item.id === line.componentId);

  function addLine() {
    const selected = new Set(lines.map((line) => line.componentId));
    const component = components.find((item) => item.currency === currency && !selected.has(item.id));
    if (!component) return;
    setLines((current) => [...current, { componentId: component.id, quantity: String(component.defaultQuantity), unitPrice: String(component.unitPriceMinor / 100) }]);
  }

  function updateLine(index: number, patch: Partial<DraftLine>) {
    setLines((current) => current.map((line, lineIndex) => lineIndex === index ? { ...line, ...patch } : line));
  }

  function selectComponent(index: number, componentId: string) {
    const component = components.find((item) => item.id === componentId);
    if (!component) return;
    updateLine(index, { componentId, quantity: String(component.defaultQuantity), unitPrice: String(component.unitPriceMinor / 100), snapshot: undefined });
  }

  async function submit(event: FormEvent) {
    event.preventDefault();
    setSaving(true);
    setError("");
    try {
      await onSave({
        id: record?.id,
        code,
        name,
        model,
        currency,
        notes,
        lines: lines.map((line) => ({ componentId: line.componentId, quantity: Number(line.quantity), unitPriceMinor: Math.round(Number(line.unitPrice) * 100) })),
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
      <section className="modal-card configuration-editor" role="dialog" aria-modal="true" onMouseDown={(event) => event.stopPropagation()}>
        <div className="panel-heading"><div><span className="eyebrow">自选配置产品</span><h2>{record ? "编辑配置清单" : "新建配置清单"}</h2><p>字段和计算方式参照天然气发电机组配置报价样本</p></div><button className="icon-button" onClick={onClose} aria-label="关闭">×</button></div>
        <form onSubmit={submit}>
          <div className="editor-form configuration-header-fields">
            <label>配置编号 *<input required value={code} onChange={(event) => setCode(event.target.value)} autoFocus /></label>
            <RememberedInput label="产品名称" required value={name} suggestions={options.filter((item) => item.kind === "product_name").map((item) => item.value)} onChange={setName} placeholder="搜索已记忆的配置产品名称" />
            <label>型号<input value={model} onChange={(event) => setModel(event.target.value)} /></label>
            <label>币种 *<input required maxLength={3} value={currency} onChange={(event) => setCurrency(event.target.value.toUpperCase())} /></label>
            <label className="field-wide">报价说明<textarea rows={2} value={notes} onChange={(event) => setNotes(event.target.value)} /></label>
          </div>
          <div className="configuration-line-heading"><div><h3>产品配置报价清单</h3><p>同一组件只能选择一次；单价可按本次配置调整并冻结</p></div><button type="button" className="button button-secondary" onClick={addLine} disabled={!components.some((item) => item.currency === currency && !lines.some((line) => line.componentId === item.id))}>添加组件</button></div>
          <div className="table-wrap configuration-line-table"><table><thead><tr><th>序号</th><th>组件</th><th>型号/规格/材质</th><th>数量</th><th>单位</th><th>单价</th><th>总价</th><th>品牌</th><th>备注</th><th></th></tr></thead><tbody>{lines.map((line, index) => { const component = componentFor(line); const snapshot = line.snapshot; const category = component?.category ?? snapshot?.category ?? "历史组件"; const itemName = component?.name ?? snapshot?.name ?? "已停用组件"; const specification = component?.specification ?? snapshot?.specification ?? ""; const unit = component?.unit ?? snapshot?.unit ?? ""; const brand = component?.brand ?? snapshot?.brand ?? ""; const lineNotes = component?.notes ?? snapshot?.notes ?? ""; const amountMinor = Math.round(Number(line.quantity || 0) * Number(line.unitPrice || 0) * 100); return <tr key={`${line.componentId}-${index}`}><td>{index + 1}</td><td><small className="component-category">{category}</small><select value={line.componentId} onChange={(event) => selectComponent(index, event.target.value)}>{snapshot && !component && <option value={snapshot.componentId}>{snapshot.name}（历史）</option>}{components.filter((item) => item.currency === currency && (item.id === line.componentId || !lines.some((current) => current.componentId === item.id))).map((item) => <option value={item.id} key={item.id}>{item.code} · {item.name}</option>)}</select><strong>{itemName}</strong></td><td className="wrap-cell">{specification || "—"}</td><td><input type="number" min="0.001" step="0.001" value={line.quantity} onChange={(event) => updateLine(index, { quantity: event.target.value })} /></td><td>{unit}</td><td><input type="number" min="0" step="0.01" value={line.unitPrice} onChange={(event) => updateLine(index, { unitPrice: event.target.value })} /></td><td>{formatMoney(amountMinor, currency)}</td><td>{brand || "—"}</td><td className="wrap-cell">{lineNotes || "—"}</td><td><button type="button" className="danger-link" onClick={() => setLines((current) => current.filter((_, lineIndex) => lineIndex !== index))}>移除</button></td></tr>; })}</tbody></table></div>
          {!lines.length && <div className="empty-callout">请至少添加一个组件。组件需先在“组件库”中录入。</div>}
          <div className="configuration-footer"><div><span>配置总价</span><strong>{formatMoney(totalMinor, currency)}</strong></div>{error && <div className="form-error">{error}</div>}<div className="modal-actions"><button type="button" className="button button-secondary" onClick={onClose}>取消</button><button className="button button-primary" disabled={saving || !lines.length}>{saving ? "保存中…" : "保存配置清单"}</button></div></div>
        </form>
      </section>
    </div>
  );
}

export function ConfigurableProductLibrary({
  configurations,
  components,
  options,
  onSave,
  onArchive,
  onExportPdf,
  onExportCsv,
  onPrint,
}: {
  configurations: ConfigurableProduct[];
  components: ConfigComponent[];
  options: ComponentOption[];
  onSave: (input: ConfigurableProductInput) => Promise<void>;
  onArchive: (id: string) => Promise<void>;
  onExportPdf: (id: string, language: ConfigurationLanguage) => Promise<string>;
  onExportCsv: (id: string, language: ConfigurationLanguage) => Promise<string>;
  onPrint: (id: string, language: ConfigurationLanguage) => Promise<string>;
}) {
  const [query, setQuery] = useState("");
  const [editing, setEditing] = useState<ConfigurableProduct | "new" | null>(null);
  const [busy, setBusy] = useState("");
  const [message, setMessage] = useState("");
  const [language, setLanguage] = useState<ConfigurationLanguage>("en");
  const normalized = query.trim().toLocaleLowerCase();
  const filtered = configurations.filter((item) => [item.code, item.name, item.model, item.notes, ...item.lines.flatMap((line) => [line.category, line.name, line.specification, line.brand])].some((value) => value.toLocaleLowerCase().includes(normalized)));

  async function archive(item: ConfigurableProduct) {
    if (!window.confirm(`停用配置“${item.name}”？已形成的清单数据会保留。`)) return;
    await onArchive(item.id);
  }

  async function output(item: ConfigurableProduct, action: "pdf" | "csv" | "print") {
    setBusy(`${item.id}-${action}`);
    setMessage("");
    try {
      const path = action === "pdf" ? await onExportPdf(item.id, language) : action === "csv" ? await onExportCsv(item.id, language) : await onPrint(item.id, language);
      setMessage(`${action === "print" ? "已打开打印用 PDF" : "已导出配置单"}：${path}`);
    } catch (reason) {
      setMessage(`导出失败：${String(reason)}`);
    } finally {
      setBusy("");
    }
  }

  return <>
    <div className="table-toolbar"><label><span className="sr-only">搜索配置</span><input placeholder="搜索配置编号、产品、型号或组件" value={query} onChange={(event) => setQuery(event.target.value)} /></label><div className="toolbar-buttons"><label className="export-language">导出语种<select value={language} onChange={(event) => setLanguage(event.target.value as ConfigurationLanguage)}>{Object.entries(configurationLanguageLabels).map(([value, label]) => <option value={value} key={value}>{label}</option>)}</select></label><button className="button button-primary" disabled={!components.length} onClick={() => setEditing("new")}>新建自选配置</button></div></div>
    {!components.length && <div className="empty-callout">请先进入“组件库”，录入至少一个可选组件。</div>}
    {message && <div className="document-message">{message}</div>}
    <div className="table-wrap"><table><thead><tr><th>配置编号</th><th>产品</th><th>型号</th><th>组件数</th><th>币种</th><th>配置总价</th><th>说明</th><th>操作</th></tr></thead><tbody>{filtered.map((item) => <tr key={item.id}><td>{item.code}</td><td><strong>{item.name}</strong></td><td>{item.model || "—"}</td><td>{item.lines.length}</td><td>{item.currency}</td><td><strong>{formatMoney(item.totalAmountMinor, item.currency)}</strong></td><td>{item.notes || "—"}</td><td><div className="row-actions configuration-actions"><button onClick={() => setEditing(item)}>配置/查看</button><button disabled={!!busy} onClick={() => void output(item, "pdf")}>{busy === `${item.id}-pdf` ? "导出中…" : "PDF"}</button><button disabled={!!busy} onClick={() => void output(item, "csv")}>{busy === `${item.id}-csv` ? "导出中…" : "CSV"}</button><button disabled={!!busy} onClick={() => void output(item, "print")}>{busy === `${item.id}-print` ? "打开中…" : "打印"}</button><button className="danger-link" onClick={() => void archive(item)}>停用</button></div></td></tr>)}</tbody></table></div>
    {!filtered.length && <div className="empty-table">{configurations.length ? "没有符合条件的配置" : "还没有自选配置产品"}</div>}
    {editing && <ConfigurationEditor record={editing === "new" ? null : editing} components={components} options={options} onClose={() => setEditing(null)} onSave={onSave} />}
  </>;
}
