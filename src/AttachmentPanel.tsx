import { useEffect, useState } from "react";
import { attachmentApi } from "./api";
import type { AttachmentRecord } from "./domain";

function sizeText(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

export function AttachmentPanel({ entityType, entityId, entityLabel, title = "附件" }: {
  entityType: "business_case" | "purchase_order" | "production_milestone" | "document" | "cost_estimate";
  entityId: string;
  entityLabel: string;
  title?: string;
}) {
  const [items, setItems] = useState<AttachmentRecord[]>([]);
  const [busy, setBusy] = useState("");
  const [message, setMessage] = useState("");

  async function load() {
    setItems(await attachmentApi.listFor(entityType, entityId));
  }

  useEffect(() => {
    load().catch((error) => setMessage(String(error)));
  }, [entityId, entityType]);

  async function upload(file: File) {
    if (file.size > 20 * 1024 * 1024) { setMessage("单个附件不能超过 20 MB。"); return; }
    setBusy("upload"); setMessage("");
    try {
      const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));
      await attachmentApi.save({ entityType, entityId, entityLabel, fileName: file.name, mimeType: file.type, bytes });
      await load();
      setMessage(`${file.name} 已加密保存`);
    } catch (error) { setMessage(String(error)); } finally { setBusy(""); }
  }

  async function exportFile(item: AttachmentRecord) {
    setBusy(item.id); setMessage("");
    try { setMessage(`已导出：${await attachmentApi.export(item.id)}`); }
    catch (error) { setMessage(String(error)); } finally { setBusy(""); }
  }

  async function remove(item: AttachmentRecord) {
    if (!window.confirm(`确定删除“${item.fileName}”吗？`)) return;
    setBusy(item.id); setMessage("");
    try { await attachmentApi.delete(item.id); await load(); }
    catch (error) { setMessage(String(error)); } finally { setBusy(""); }
  }

  return <section className="embedded-attachments">
    <div className="embedded-attachment-heading"><div><strong>{title}</strong><span>{items.length} 个加密文件 · 单个最大 20 MB</span></div><label className="button button-secondary attachment-upload">{busy === "upload" ? "上传中…" : "添加附件"}<input className="sr-only" type="file" disabled={!!busy} onChange={(event) => { const file = event.target.files?.[0]; event.target.value = ""; if (file) void upload(file); }} /></label></div>
    {message && <div className="attachment-message">{message}</div>}
    <div className="embedded-attachment-list">{items.map((item) => <article key={item.id}><div><strong>{item.fileName}</strong><span>{sizeText(item.sizeBytes)} · {item.createdAt} · {item.sha256.slice(0, 10)}…</span></div><div className="row-actions"><button disabled={!!busy} onClick={() => void exportFile(item)}>导出</button><button className="danger-link" disabled={!!busy} onClick={() => void remove(item)}>删除</button></div></article>)}</div>
    {!items.length && <small className="attachment-empty">暂无附件</small>}
  </section>;
}
