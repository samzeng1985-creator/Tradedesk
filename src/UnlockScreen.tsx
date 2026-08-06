import { useState } from "react";
import type { FormEvent } from "react";

interface UnlockScreenProps {
  checking: boolean;
  existing: boolean;
  onUnlock: (password: string, companyName?: string) => Promise<void>;
}

export function UnlockScreen({ checking, existing, onUnlock }: UnlockScreenProps) {
  const [password, setPassword] = useState("");
  const [confirmation, setConfirmation] = useState("");
  const [companyName, setCompanyName] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

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

  return (
    <main className="unlock-screen">
      <section className="unlock-card">
        <div className="unlock-brand"><span className="brand-mark">TD</span><strong>TradeDesk Local</strong></div>
        <span className="eyebrow">SQLCipher 本地加密</span>
        <h1>{checking ? "正在检查工作区" : existing ? "解锁工作区" : "创建加密工作区"}</h1>
        <p>业务资料只保存在这台电脑。密码不会写入数据库或日志。</p>
        {!checking && (
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
        <small>请妥善保管密码。当前版本无法找回遗失的工作区密码。</small>
      </section>
    </main>
  );
}
