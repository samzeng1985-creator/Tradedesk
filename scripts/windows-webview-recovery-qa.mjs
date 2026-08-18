import fs from "node:fs";

const [, , operation, portText, outputPath, backupPath, recoveryKey] = process.argv;
const port = Number(portText);
if (!operation || !port || !outputPath) {
  throw new Error("Usage: node windows-webview-recovery-qa.mjs <seed|restore|shutdown> <port> <output.json> [backup.tdbackup] [recovery-key]");
}

async function waitForTarget() {
  const deadline = Date.now() + 30_000;
  while (Date.now() < deadline) {
    try {
      const targets = await fetch(`http://127.0.0.1:${port}/json/list`).then((response) => response.json());
      const target = targets.find((item) => item.type === "page" && item.webSocketDebuggerUrl);
      if (target) return target;
    } catch {}
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  throw new Error(`WebView2 debug target did not appear on port ${port}`);
}

const target = await waitForTarget();
const socket = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((resolve, reject) => {
  socket.addEventListener("open", resolve, { once: true });
  socket.addEventListener("error", reject, { once: true });
});

let sequence = 0;
const pending = new Map();
socket.addEventListener("message", (event) => {
  const message = JSON.parse(String(event.data));
  if (!message.id || !pending.has(message.id)) return;
  const { resolve, reject } = pending.get(message.id);
  pending.delete(message.id);
  if (message.error) reject(new Error(JSON.stringify(message.error)));
  else resolve(message.result);
});

function send(method, params = {}) {
  const id = ++sequence;
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject });
    socket.send(JSON.stringify({ id, method, params }));
  });
}

async function evaluate(expression) {
  const result = await send("Runtime.evaluate", {
    expression,
    awaitPromise: true,
    returnByValue: true,
    userGesture: true,
  });
  if (result.exceptionDetails) {
    throw new Error(result.exceptionDetails.exception?.description || result.exceptionDetails.text);
  }
  return result.result.value;
}

async function invoke(command, args = {}) {
  const expression = `(async () => {
    try {
      const value = await window.__TAURI_INTERNALS__.invoke(${JSON.stringify(command)}, ${JSON.stringify(args)});
      return { ok: true, value };
    } catch (error) {
      return { ok: false, error: String(error) };
    }
  })()`;
  const result = await evaluate(expression);
  if (!result?.ok) throw new Error(`${command}: ${result?.error || "unknown invoke error"}`);
  return result.value;
}

const password = "TradeDesk-RC1-QA-2026!";
let report;

if (operation === "seed") {
  const existedBefore = await invoke("workspace_exists");
  if (existedBefore) throw new Error("Isolated QA workspace already exists before seed operation");
  const unlocked = await invoke("unlock_workspace", { password, companyName: "TradeDesk RC1 QA Company" });
  const product = await invoke("save_product", { input: {
    id: null, sku: "QA-PRODUCT-001", nameZh: "RC1恢复测试产品", nameEn: "RC1 recovery test product",
    model: "QA-2026", hsCode: "85022000", unit: "set", grossWeightKg: 123.45,
  }});
  const customer = await invoke("save_customer", { input: {
    id: null, code: "QA-CUSTOMER-001", legalName: "RC1 Recovery Customer LLC", market: "Russia",
    currency: "USD", paymentTerms: "T/T", address: "QA address", shippingAddress: "QA shipping address",
    billingAddress: "QA billing address", purchaseIntent: "Recovery acceptance", customerAnalysis: "QA only",
    strengths: "Test", weaknesses: "Test", contacts: "qa@example.invalid",
  }});
  const supplier = await invoke("save_supplier", { input: {
    id: null, code: "QA-SUPPLIER-001", legalName: "RC1 Recovery Supplier", address: "Wuhan QA",
    contacts: "qa-supplier@example.invalid", bankDetails: "QA ONLY", currency: "CNY", paymentTerms: "T/T",
    leadTimeDays: 30, onTimeRate: 95, qualificationNotes: "Recovery acceptance", productTerms: [],
  }});
  const attachmentBytes = Array.from(new TextEncoder().encode("TradeDesk RC1 encrypted recovery attachment"));
  const attachment = await invoke("save_attachment", { input: {
    entityType: "product", entityId: product.id, entityLabel: product.sku,
    fileName: "rc1-recovery-proof.txt", mimeType: "text/plain", bytes: attachmentBytes,
  }});
  const summaryBefore = await invoke("workspace_summary");
  const backup = await invoke("create_workspace_backup");
  await invoke("lock_workspace");
  report = { operation, existedBefore, unlocked, product, customer, supplier, attachment, summaryBefore, backup };
} else if (operation === "restore") {
  if (!backupPath || !recoveryKey) throw new Error("Restore operation requires a backup path and recovery key");
  await invoke("lock_workspace");
  const bytes = Array.from(fs.readFileSync(backupPath));
  await invoke("restore_workspace_backup", { bytes });
  const pendingBeforeUnlock = await invoke("workspace_restore_pending");
  let wrongPasswordRejected = false;
  try {
    await invoke("unlock_workspace", { password: "wrong-RC1-password", companyName: null });
  } catch {
    wrongPasswordRejected = true;
  }
  const summaryAfter = await invoke("unlock_workspace", { password, companyName: null });
  const pendingAfterUnlock = await invoke("workspace_restore_pending");
  const products = await invoke("list_products");
  const customers = await invoke("list_customers");
  const suppliers = await invoke("list_suppliers");
  const attachments = await invoke("list_attachments");
  const exportedAttachmentPath = await invoke("export_attachment", { id: attachments[0].id });
  await invoke("lock_workspace");
  const recoverySummary = await invoke("unlock_workspace_with_recovery", { recoveryKey });
  report = {
    operation, pendingBeforeUnlock, wrongPasswordRejected, summaryAfter, pendingAfterUnlock,
    products: products.map(({ id, sku, nameEn }) => ({ id, sku, nameEn })),
    customers: customers.map(({ id, code, legalName }) => ({ id, code, legalName })),
    suppliers: suppliers.map(({ id, code, legalName }) => ({ id, code, legalName })),
    attachments: attachments.map(({ id, entityType, entityId, fileName, sizeBytes, sha256 }) => ({ id, entityType, entityId, fileName, sizeBytes, sha256 })),
    exportedAttachmentPath,
    recoverySummary,
  };
} else if (operation === "shutdown") {
  await invoke("lock_workspace");
  report = { operation, result: "locked" };
} else {
  throw new Error(`Unknown operation: ${operation}`);
}

fs.writeFileSync(outputPath, `${JSON.stringify(report, null, 2)}\n`);
if (operation === "shutdown") {
  await send("Browser.close");
}
socket.close();
