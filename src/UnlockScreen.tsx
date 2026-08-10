import { useState } from "react";
import type { FormEvent } from "react";

interface UnlockScreenProps {
  checking: boolean;
  existing: boolean;
  restorePending: boolean;
  onUnlock: (password: string, companyName?: string) => Promise<void>;
  onRecover: (recoveryKey: string) => Promise<void>;
  onRestore: (bytes: number[]) => Promise<void>;
  onRollbackRestore: () => Promise<void>;
}

export function UnlockScreen({ checking, existing, restorePending, onUnlock, onRecover, onRestore, onRollbackRestore }: UnlockScreenProps) {
  const [password, setPassword] = useState("");
  const [confirmation, setConfirmation] = useState("");
  const [companyName, setCompanyName] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [recoveryMode, setRecoveryMode] = useState(false);
  const [recoveryKey, setRecoveryKey] = useState("");
  const [message, setMessage] = useState("");

  async function submit(event: FormEvent) {
    event.preventDefault();
    if (password.length < 8) {
      setError("工作区密码至少需要 8 个字符。");
      return;
    }
    if (!existing && password !== confirmation) {
      setError("两次输入的密码不一致。");
      return;
    }
    setBusy(true);
    setError("");
    try {
      await onUnlock(password, existing ? undefined : companyName.trim() || "本地工作区");
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(false);
    }
  }

  async function recover(event: FormEvent) {
    event.preventDefault();
    if (!recoveryKey.trim()) { setError("请输入完整的恢复密钥。"); return; }
    setBusy(true); setError(""); setMessage("");
    try { await onRecover(recoveryKey.trim()); }
    catch (reason) { setError(String(reason)); }
    finally { setBusy(false); }
  }

  async function restore(file: File) {
    if (existing && !window.confirm("恢复备份将替换当前工作区。系统会先保留临时安全副本，确定继续吗？")) return;
    setBusy(true); setError(""); setMessage("");
    try {
      const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));
      await onRestore(bytes);
      setMessage("备份已经恢复。现在可使用备份时的密码或恢复密钥解锁。");
    } catch (reason) { setError(String(reason)); }
    finally { setBusy(false); }
  }

  async function rollbackRestore() {
    if (!window.confirm("确定撤销刚才的备份恢复并回到原工作区吗？")) return;
    setBusy(true); setError(""); setMessage("");
    try {
      await onRollbackRestore();
      setMessage("已撤销备份恢复，原工作区已经还原。");
    } catch (reason) { setError(String(reason)); }
    finally { setBusy(false); }
  }

  return (
    <main className="unlock-screen">
      <section className="unlock-card">
        <div className="unlock-brand"><span className="brand-mark">TD</span><strong>TradeDesk Local</strong></div>
        <span className="eyebrow">SQLCipher 本地加密</span>
        <h1>{checking ? "正在检查工作区" : recoveryMode ? "使用恢复密钥" : existing ? "解锁工作区" : "创建加密工作区"}</h1>
        <p>{recoveryMode ? "恢复密钥会在本机解密工作区密码，密钥本身不会保存。" : "业务资料只保存在这台电脑。密码不会写入业务数据库或日志。"}</p>
        {!checking && !recoveryMode && (
          <form onSubmit={submit} className="unlock-form">
            {!existing && (
              <label>公司或工作区名称<input value={companyName} onChange={(event) => setCompanyName(event.target.value)} placeholder="例如：星海进出口" autoFocus /></label>
            )}
            <label>工作区密码<input type="password" value={password} onChange={(event) => setPassword(event.target.value)} autoFocus={existing} autoComplete={existing ? "current-password" : "new-password"} /></label>
            {!existing && (
              <label>确认密码<input type="password" value={confirmation} onChange={(event) => setConfirmation(event.target.value)} autoComplete="new-password" /></label>
            )}
            {error && <div className="form-error" role="alert">{error}</div>}
            <button className="button button-primary button-wide" disabled={busy}>{busy ? "处理中…" : existing ? "解锁" : "创建并进入"}</button>
          </form>
        )}
        {!checking && recoveryMode && <form onSubmit={recover} className="unlock-form"><label>恢复密钥<textarea rows={4} value={recoveryKey} onChange={(event) => setRecoveryKey(event.target.value)} placeholder="TDK-…" autoFocus /></label>{error && <div className="form-error" role="alert">{error}</div>}<button className="button button-primary button-wide" disabled={busy}>{busy ? "恢复中…" : "恢复并解锁"}</button></form>}
        {message && <div className="document-message">{message}</div>}
        {restorePending && <div className="restore-pending"><strong>备份尚未验证</strong><span>成功解锁后系统才会清理原工作区安全副本；如果备份无法解锁，可撤销恢复。</span><button className="button button-secondary button-wide" disabled={busy} onClick={() => void rollbackRestore()}>撤销上次恢复</button></div>}
        {!checking && existing && <div className="unlock-alternatives"><button className="text-button" onClick={() => { setRecoveryMode((value) => !value); setError(""); }}>{recoveryMode ? "返回密码解锁" : "忘记密码？使用恢复密钥"}</button><label className="text-button restore-upload">从加密备份恢复<input className="sr-only" type="file" accept=".tdbackup" disabled={busy} onChange={(event) => { const file = event.target.files?.[0]; event.target.value = ""; if (file) void restore(file); }} /></label></div>}
        {!checking && !existing && <label className="text-button restore-upload create-restore">已有备份？从 .tdbackup 恢复<input className="sr-only" type="file" accept=".tdbackup" disabled={busy} onChange={(event) => { const file = event.target.files?.[0]; event.target.value = ""; if (file) void restore(file); }} /></label>}
        <small>首次创建或升级工作区时会生成一次性显示的恢复密钥，请离线保存。</small>
      </section>
    </main>
  );
}
