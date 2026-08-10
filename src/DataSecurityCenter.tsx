import { useEffect, useState } from "react";
import { attachmentApi, workspaceApi } from "./api";
import type { AttachmentRecord, BackupResult } from "./domain";

function fileSize(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

function entityLabel(value: string) {
  return {
    workspace: "工作区",
    business_case: "业务单",
    purchase_order: "采购单",
    document: "单证",
    production: "生产记录",
  }[value] ?? value;
}

export function RecoveryKeyNotice({ recoveryKey, onClose }: { recoveryKey: string; onClose: () => void }) {
  const [copied, setCopied] = useState(false);

  async function copy() {
    await navigator.clipboard.writeText(recoveryKey);
    setCopied(true);
  }

  return <div className="modal-backdrop recovery-backdrop">
    <section className="modal-card recovery-card" role="dialog" aria-modal="true" aria-labelledby="recovery-title">
      <span className="eyebrow">只显示这一次</span>
      <h2 id="recovery-title">保存工作区恢复密钥</h2>
      <p>密码遗失时，这串密钥可以解锁工作区。请复制到密码管理器或打印后离线保存，不要只存放在本机。</p>
      <code className="recovery-key">{recoveryKey}</code>
      <div className="modal-actions"><button className="button button-secondary" onClick={() => void copy()}>{copied ? "已复制" : "复制密钥"}</button><button className="button button-primary" onClick={onClose}>我已安全保存</button></div>
    </section>
  </div>;
}

export function DataSecurityCenter({ recoveryReady, onRecoveryKey }: {
  recoveryReady: boolean;
  onRecoveryKey: (key: string) => void;
}) {
  const [attachments, setAttachments] = useState<AttachmentRecord[]>([]);
  const [entityType, setEntityType] = useState("workspace");
  const [entityId, setEntityId] = useState("");
  const [busy, setBusy] = useState("");
  const [message, setMessage] = useState("");

  useEffect(() => {
    attachmentApi.list().then(setAttachments).catch((error) => setMessage(String(error)));
  }, []);

  async function rotateRecoveryKey() {
    if (!window.confirm("生成新密钥后，旧恢复密钥会立即失效。确定继续吗？")) return;
    setBusy("recovery"); setMessage("");
    try {
      const key = await workspaceApi.rotateRecoveryKey();
      onRecoveryKey(key);
      setMessage("新恢复密钥已生成，旧密钥已经失效。");
    } catch (error) { setMessage(String(error)); } finally { setBusy(""); }
  }

  async function createBackup() {
    setBusy("backup"); setMessage("");
    try {
      const result: BackupResult = await workspaceApi.createBackup();
      setMessage(`加密备份已创建：${result.path}（${fileSize(result.sizeBytes)}）`);
    } catch (error) { setMessage(String(error)); } finally { setBusy(""); }
  }

  async function upload(file: File) {
    if (file.size > 20 * 1024 * 1024) {
      setMessage("单个附件不能超过 20 MB。");
      return;
    }
    setBusy("attachment"); setMessage("");
    try {
      const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));
      await attachmentApi.save({ entityType, entityId: entityId.trim(), fileName: file.name, mimeType: file.type, bytes });
      setAttachments(await attachmentApi.list());
      setMessage(`${file.name} 已写入 SQLCipher 加密附件库。`);
    } catch (error) { setMessage(String(error)); } finally { setBusy(""); }
  }

  async function exportAttachment(item: AttachmentRecord) {
    setBusy(item.id); setMessage("");
    try { setMessage(`附件已导出：${await attachmentApi.export(item.id)}`); }
    catch (error) { setMessage(String(error)); } finally { setBusy(""); }
  }

  async function deleteAttachment(item: AttachmentRecord) {
    if (!window.confirm(`确定删除加密附件“${item.fileName}”吗？此操作不可撤销。`)) return;
    setBusy(item.id); setMessage("");
    try {
      await attachmentApi.delete(item.id);
      setAttachments((current) => current.filter((candidate) => candidate.id !== item.id));
      setMessage(`${item.fileName} 已删除。`);
    } catch (error) { setMessage(String(error)); } finally { setBusy(""); }
  }

  return <div className="page-stack security-center">
    {message && <div className="document-message">{message}</div>}
    <section className="security-grid">
      <article className="panel security-card">
        <span className="eyebrow">密码遗失保护</span><h2>恢复密钥</h2>
        <p>{recoveryReady ? "恢复密钥库已建立。TradeDesk 不会保存可直接查看的恢复密钥。" : "尚未建立恢复密钥。"}</p>
        <button className="button button-secondary" disabled={!!busy} onClick={() => void rotateRecoveryKey()}>{busy === "recovery" ? "生成中…" : "重新生成恢复密钥"}</button>
      </article>
      <article className="panel security-card">
        <span className="eyebrow">数据库与恢复信息</span><h2>加密备份</h2>
        <p>备份包保存到“文档/TradeDesk Backups”，包含加密数据库及恢复密钥库，不包含明文密码。</p>
        <button className="button button-primary" disabled={!!busy || !recoveryReady} onClick={() => void createBackup()}>{busy === "backup" ? "备份中…" : "立即创建备份"}</button>
        <small>恢复方法：锁定工作区，在解锁页选择 .tdbackup 文件。</small>
      </article>
    </section>

    <section className="panel attachment-library">
      <div className="panel-heading"><div><h2>加密附件库</h2><p>文件内容作为 SQLCipher BLOB 保存，导出时才写入普通文件</p></div></div>
      <div className="attachment-toolbar">
        <label>资料类型<select value={entityType} onChange={(event) => setEntityType(event.target.value)}><option value="workspace">工作区</option><option value="business_case">业务单</option><option value="purchase_order">采购单</option><option value="production">生产记录</option><option value="document">单证</option></select></label>
        <label>关联编号（可选）<input value={entityId} onChange={(event) => setEntityId(event.target.value)} placeholder="例如 TD-2026-0001" /></label>
        <label className="button button-primary attachment-upload">{busy === "attachment" ? "上传中…" : "添加附件"}<input className="sr-only" type="file" disabled={!!busy} onChange={(event) => { const file = event.target.files?.[0]; event.target.value = ""; if (file) void upload(file); }} /></label>
      </div>
      <div className="table-wrap"><table><thead><tr><th>文件</th><th>关联资料</th><th>大小</th><th>校验值</th><th>添加时间</th><th>操作</th></tr></thead><tbody>{attachments.map((item) => <tr key={item.id}><td><strong>{item.fileName}</strong><small className="table-subtitle">{item.mimeType || "未知类型"}</small></td><td>{entityLabel(item.entityType)}{item.entityId ? ` · ${item.entityId}` : ""}</td><td>{fileSize(item.sizeBytes)}</td><td><code>{item.sha256.slice(0, 12)}…</code></td><td>{item.createdAt}</td><td><div className="row-actions"><button disabled={!!busy} onClick={() => void exportAttachment(item)}>导出</button><button className="danger-link" disabled={!!busy} onClick={() => void deleteAttachment(item)}>删除</button></div></td></tr>)}</tbody></table></div>
      {!attachments.length && <div className="empty-table">暂无附件。可先保存合同附件、采购确认文件或生产照片。</div>}
    </section>
  </div>;
}
