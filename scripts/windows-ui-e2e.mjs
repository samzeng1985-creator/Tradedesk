import fs from "node:fs";
import path from "node:path";

const [, , operation, portText, outputPath, screenshotDirectory] = process.argv;
const port = Number(portText);
if (!operation || !port || !outputPath || !screenshotDirectory) {
  throw new Error("Usage: node windows-ui-e2e.mjs <seed|verify> <port> <output.json> <screenshot-directory>");
}

const password = "TradeDesk-RC2-UI-QA-2026!";
const expected = {
  productSku: "RC2-ENGINE-1000",
  customerCode: "RC2-CUSTOMER-001",
  supplierCode: "RC2-SUPPLIER-001",
  businessNumber: "TD-20260818-2801",
  supplierPrice: "1234567890.12",
  purchaseTotal: "600,000.00",
};

function delay(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

async function waitForTarget() {
  const deadline = Date.now() + 45_000;
  while (Date.now() < deadline) {
    try {
      const targets = await fetch(`http://127.0.0.1:${port}/json/list`).then((response) => response.json());
      const target = targets.find((item) => item.type === "page" && item.webSocketDebuggerUrl);
      if (target) return target;
    } catch {}
    await delay(250);
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

async function pageCall(method, ...args) {
  return evaluate(`window.__tradeDeskQa.${method}(...${JSON.stringify(args)})`);
}

async function waitUntil(description, predicate, timeout = 15_000) {
  const deadline = Date.now() + timeout;
  let lastValue;
  while (Date.now() < deadline) {
    lastValue = await predicate();
    if (lastValue) return lastValue;
    await delay(100);
  }
  const body = await pageCall("bodyText");
  throw new Error(`Timed out waiting for ${description}. Last value: ${JSON.stringify(lastValue)}. Page: ${body.slice(0, 1200)}`);
}

async function clickAndWait(text, expectedText, selector = "button") {
  const started = performance.now();
  await pageCall("clickText", selector, text, true);
  await waitUntil(expectedText, () => pageCall("hasText", expectedText));
  return Math.round((performance.now() - started) * 10) / 10;
}

async function screenshot(name) {
  fs.mkdirSync(screenshotDirectory, { recursive: true });
  const result = await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: false });
  const destination = path.join(screenshotDirectory, `${name}.png`);
  fs.writeFileSync(destination, Buffer.from(result.data, "base64"));
  return destination;
}

await send("Page.enable");
await send("Runtime.enable");
await waitUntil("React application body", async () => (await evaluate("document.body?.innerText?.length || 0")) > 0);
await evaluate(`(() => {
  const normalize = (value) => String(value ?? "").replace(/\\s+/g, " ").trim();
  const visible = (element) => !!(element && (element.offsetWidth || element.offsetHeight || element.getClientRects().length));
  const ensureVisible = (element) => {
    const rectangle = element.getBoundingClientRect();
    if (rectangle.top < 0 || rectangle.bottom > innerHeight || rectangle.left < 0 || rectangle.right > innerWidth) {
      element.scrollIntoView({ block: "center", inline: "nearest" });
    }
  };
  const nativeSetter = (element, value) => {
    const prototype = element instanceof HTMLTextAreaElement
      ? HTMLTextAreaElement.prototype
      : element instanceof HTMLSelectElement
        ? HTMLSelectElement.prototype
        : HTMLInputElement.prototype;
    const setter = Object.getOwnPropertyDescriptor(prototype, "value")?.set;
    if (!setter) throw new Error("Native value setter was not found");
    setter.call(element, value);
    element.dispatchEvent(new Event("input", { bubbles: true }));
    element.dispatchEvent(new Event("change", { bubbles: true }));
  };
  const labelControl = (prefix, root = document) => {
    const label = [...root.querySelectorAll("label")].find((item) => visible(item) && normalize(item.textContent).startsWith(prefix));
    const control = label?.querySelector("input, textarea, select");
    if (!control) throw new Error('Control for label "' + prefix + '" was not found');
    return control;
  };
  const textElement = (selector, text, exact = true, root = document) => {
    const candidates = [...root.querySelectorAll(selector)].filter(visible);
    const found = candidates.find((item) => exact ? normalize(item.textContent) === text : normalize(item.textContent).includes(text));
    if (!found) throw new Error('Visible ' + selector + ' with text "' + text + '" was not found');
    return found;
  };
  window.__tradeDeskQa = {
    bodyText: () => normalize(document.body.innerText),
    hasText: (text) => normalize(document.body.innerText).includes(text),
    clickText: (selector, text, exact = true) => {
      const element = textElement(selector, text, exact);
      if (element.disabled) throw new Error('Element "' + text + '" is disabled');
      ensureVisible(element);
      element.click();
      return true;
    },
    clickDialogText: (text) => {
      const dialog = document.querySelector('[role="dialog"]');
      if (!dialog) throw new Error("Dialog was not found");
      const element = textElement("button", text, true, dialog);
      if (element.disabled) throw new Error('Dialog button "' + text + '" is disabled');
      ensureVisible(element);
      element.click();
      return true;
    },
    setLabel: (prefix, value) => {
      const control = labelControl(prefix);
      ensureVisible(control);
      nativeSetter(control, value);
      return control.value;
    },
    setWithin: (rootSelector, prefix, value) => {
      const root = document.querySelector(rootSelector);
      if (!root) throw new Error('Root "' + rootSelector + '" was not found');
      const control = labelControl(prefix, root);
      ensureVisible(control);
      nativeSetter(control, value);
      return control.value;
    },
    selectOption: (prefix, optionText, rootSelector = "") => {
      const root = rootSelector ? document.querySelector(rootSelector) : document;
      if (!root) throw new Error('Root "' + rootSelector + '" was not found');
      const control = labelControl(prefix, root);
      if (!(control instanceof HTMLSelectElement)) throw new Error('Label "' + prefix + '" is not a select');
      const option = [...control.options].find((item) => normalize(item.textContent).includes(optionText));
      if (!option) throw new Error('Option containing "' + optionText + '" was not found for "' + prefix + '"');
      nativeSetter(control, option.value);
      return { value: control.value, text: normalize(option.textContent) };
    },
    valueForLabel: (prefix, rootSelector = "") => {
      const root = rootSelector ? document.querySelector(rootSelector) : document;
      return labelControl(prefix, root).value;
    },
    setAriaSelect: (labelPart, value) => {
      const control = [...document.querySelectorAll("select[aria-label]")].find((item) => visible(item) && item.getAttribute("aria-label").includes(labelPart));
      if (!control) throw new Error('Select with aria-label containing "' + labelPart + '" was not found');
      nativeSetter(control, value);
      return control.value;
    },
    ariaSelectState: (labelPart) => {
      const control = [...document.querySelectorAll("select[aria-label]")].find((item) => item.getAttribute("aria-label").includes(labelPart));
      return control ? { value: control.value, disabled: control.disabled } : null;
    },
    metric: (label) => {
      const article = [...document.querySelectorAll("article")].find((item) => normalize(item.querySelector("span")?.textContent) === label);
      return normalize(article?.querySelector("strong")?.textContent);
    },
    milestoneState: (label) => {
      const button = [...document.querySelectorAll(".milestone-card")].find((item) => normalize(item.textContent).includes(label));
      return button ? normalize(button.textContent) : "";
    },
    clickMilestone: (label) => {
      const button = [...document.querySelectorAll(".milestone-card")].find((item) => normalize(item.textContent).includes(label));
      if (!button) throw new Error('Milestone "' + label + '" was not found');
      ensureVisible(button);
      button.click();
      return true;
    },
    dialogOpen: () => !!document.querySelector('[role="dialog"]'),
    topbarTitle: () => normalize(document.querySelector(".topbar h1")?.textContent),
    rowContains: (text) => [...document.querySelectorAll("tbody tr")].some((row) => normalize(row.textContent).includes(text)),
    elementRect: (selector) => {
      const element = document.querySelector(selector);
      if (!element) return null;
      const rectangle = element.getBoundingClientRect();
      return { width: rectangle.width, height: rectangle.height, whiteSpace: getComputedStyle(element).whiteSpace };
    },
    resetScroll: () => {
      window.scrollTo(0, 0);
      for (const element of document.querySelectorAll(".sidebar, main, .modal-card")) element.scrollTop = 0;
      return true;
    },
  };
})()`);

const timings = {};
const assertions = [];
function assertEqual(name, actual, expectedValue) {
  if (actual !== expectedValue) throw new Error(`${name}: expected ${JSON.stringify(expectedValue)}, got ${JSON.stringify(actual)}`);
  assertions.push({ name, result: "passed", actual });
}
function assertIncludes(name, actual, expectedPart) {
  if (!String(actual).includes(expectedPart)) throw new Error(`${name}: expected ${JSON.stringify(actual)} to include ${JSON.stringify(expectedPart)}`);
  assertions.push({ name, result: "passed", actual: String(actual).length > 200 ? expectedPart : actual });
}

async function saveDialog(waitText) {
  await pageCall("clickDialogText", "保存");
  await waitUntil(`dialog close after saving ${waitText}`, async () => !(await pageCall("dialogOpen")));
  await waitUntil(`${waitText} visible`, () => pageCall("hasText", waitText));
}

async function runSeed() {
  await waitUntil("create workspace screen", () => pageCall("hasText", "创建加密工作区"));
  await pageCall("setLabel", "公司或工作区名称", "TradeDesk RC2 UI QA");
  await pageCall("setLabel", "工作区密码", password);
  await pageCall("setLabel", "确认密码", password);
  await pageCall("clickText", "button", "创建并进入", true);
  await waitUntil("recovery key notice", () => pageCall("hasText", "我已安全保存"), 30_000);
  await pageCall("clickText", "button", "我已安全保存", true);
  await waitUntil("workbench", async () => (await pageCall("topbarTitle")) === "业务工作台");

  timings.openMasterDataMs = await clickAndWait("主数据", "一次建档，多处复用");
  await pageCall("clickText", "button", "新建记录", true);
  await waitUntil("new product dialog", () => pageCall("hasText", "新建产品"));
  await pageCall("setLabel", "SKU", expected.productSku);
  await pageCall("setLabel", "英文名称", "RC2 Natural Gas Generator");
  await pageCall("setLabel", "中文名称", "RC2天然气发电机组");
  await pageCall("setLabel", "型号", "RC2-1000KW");
  await pageCall("setLabel", "HS 编码", "85022000");
  await pageCall("setLabel", "单位", "set");
  await pageCall("setLabel", "单件毛重", "8200.5");
  await saveDialog(expected.productSku);

  await pageCall("clickText", "button[role=tab]", "客户", false);
  await pageCall("clickText", "button", "新建记录", true);
  await waitUntil("new customer dialog", () => pageCall("hasText", "新建客户"));
  await pageCall("setLabel", "客户编码", expected.customerCode);
  await pageCall("setLabel", "客户法定名称", "RC2 Moscow Energy LLC");
  await pageCall("setLabel", "国家/市场", "Russia");
  await pageCall("setLabel", "默认付款条款", "T/T 50% + 50%");
  await pageCall("setLabel", "客户地址", "Moscow registered office");
  await pageCall("setLabel", "收货地址", "Moscow project warehouse");
  await pageCall("setLabel", "账单地址", "Moscow billing office");
  await pageCall("setLabel", "购买意向", "1000kW natural gas generator, 2 sets");
  await pageCall("setLabel", "客户分析", "RC2 synthetic acceptance customer");
  await pageCall("setLabel", "优势", "Defined project schedule");
  await pageCall("setLabel", "劣势与风险", "Exchange-rate exposure");
  await pageCall("setLabel", "联系人", "QA Buyer | qa-buyer@example.invalid");
  await saveDialog(expected.customerCode);

  await pageCall("clickText", "button[role=tab]", "供应商", false);
  await pageCall("clickText", "button", "新建记录", true);
  await waitUntil("new supplier dialog", () => pageCall("hasText", "新建供应商"));
  await pageCall("setLabel", "供应商编码", expected.supplierCode);
  await pageCall("setLabel", "供应商法定名称", "RC2 Wuhan Generator Works");
  await pageCall("setLabel", "默认付款条款", "30% advance, 70% before shipment");
  await pageCall("setLabel", "默认交期", "45");
  await pageCall("setLabel", "准时率", "96");
  await pageCall("setLabel", "地址", "Wuhan QA industrial park");
  await pageCall("setLabel", "联系人", "QA Supplier | qa-supplier@example.invalid");
  await pageCall("setLabel", "银行资料", "SYNTHETIC QA DATA ONLY");
  await pageCall("setLabel", "资质、质量与评估备注", "RC2 interaction acceptance");
  await pageCall("clickDialogText", "添加供应产品");
  await pageCall("selectOption", "产品", expected.productSku, ".supplier-product-term");
  await pageCall("setWithin", ".supplier-product-term", "采购单价", expected.supplierPrice);
  await pageCall("setWithin", ".supplier-product-term", "MOQ", "1");
  await pageCall("setWithin", ".supplier-product-term", "交期", "45");
  const supplierPrice = await pageCall("valueForLabel", "采购单价", ".supplier-product-term");
  assertEqual("12-digit supplier price input", supplierPrice, expected.supplierPrice);
  await saveDialog(expected.supplierCode);

  timings.openBusinessCasesMs = await clickAndWait("业务单", "客户、产品与商业条款的统一业务快照");
  await pageCall("clickText", "button", "新建业务单", true);
  await waitUntil("new business case dialog", () => pageCall("hasText", "新建业务单"));
  await pageCall("setLabel", "业务单号", expected.businessNumber);
  await pageCall("selectOption", "客户", expected.customerCode);
  await pageCall("setLabel", "贸易术语", "CIF");
  await pageCall("setLabel", "计划发货日", "2026-12-20");
  await pageCall("selectOption", "产品 / 自选配置", expected.productSku);
  await pageCall("setLabel", "数量", "2");
  await pageCall("setLabel", "单价", "50000.00");
  await pageCall("setLabel", "备注", "RC2 real UI interaction order");
  await pageCall("clickDialogText", "保存业务单");
  await waitUntil("business case saved", async () => !(await pageCall("dialogOpen")) && await pageCall("rowContains", expected.businessNumber));
  await pageCall("setAriaSelect", expected.businessNumber, "order");
  await waitUntil("inline business stage persisted", async () => {
    const state = await pageCall("ariaSelectState", expected.businessNumber);
    return state?.value === "order" && !state.disabled;
  });

  timings.openFulfillmentMs = await clickAndWait("采购与生产", "只跟踪关键里程碑，不做复杂排产");
  await pageCall("clickText", "button", "新建采购单", true);
  await waitUntil("new purchase order dialog", () => pageCall("hasText", "新建采购单"));
  await pageCall("selectOption", "来源业务单", expected.businessNumber);
  await pageCall("selectOption", "供应商", expected.supplierCode);
  await pageCall("setLabel", "采购币种", "CNY");
  await pageCall("setLabel", "采购折算汇率", "7.2");
  await pageCall("setLabel", "汇率日期", "2026-08-18");
  await pageCall("setLabel", "预计交货日", "2026-11-30");
  await pageCall("setWithin", ".purchase-draft-line", "采购数量", "2");
  await pageCall("setWithin", ".purchase-draft-line", "采购单价", "300000.00");
  await pageCall("clickDialogText", "创建采购单");
  await waitUntil("purchase order saved", async () => !(await pageCall("dialogOpen")) && await pageCall("hasText", expected.purchaseTotal));

  await pageCall("clickMilestone", "原料准备");
  await waitUntil("milestone dialog", () => pageCall("hasText", "生产节点"));
  await pageCall("setLabel", "状态", "completed");
  await pageCall("setLabel", "实际日期", "2026-08-18");
  await pageCall("setLabel", "完成数量", "2");
  await pageCall("setLabel", "异常或备注", "RC2 completed through real UI");
  await pageCall("clickDialogText", "保存节点");
  await waitUntil("milestone saved", async () => !(await pageCall("dialogOpen")) && (await pageCall("milestoneState", "原料准备")).includes("100%"));
  assertIncludes("milestone completion", await pageCall("milestoneState", "原料准备"), "100%");

  timings.openWorkbenchMs = await clickAndWait("工作台", "从订单到单证的轻量闭环");
  assertIncludes("sales amount", await pageCall("metric", "销售金额"), "100,000");
  assertIncludes("converted purchase cost", await pageCall("metric", "折算采购成本"), "83,333");
  assertEqual("purchase coverage", await pageCall("metric", "采购覆盖"), "100%");

  const navigation = [
    ["业务单", "业务单"], ["主数据", "主数据"], ["成本估算", "成本估算"],
    ["采购与生产", "采购与生产"], ["装运与收款", "装运与收款"],
    ["单证中心", "单证中心"], ["企业设置", "企业设置"], ["数据与安全", "数据与安全"],
  ];
  timings.navigationMs = [];
  for (const [button, title] of navigation) {
    const started = performance.now();
    await pageCall("clickText", "nav button", button, true);
    await waitUntil(`topbar ${title}`, async () => (await pageCall("topbarTitle")) === title);
    timings.navigationMs.push(Math.round((performance.now() - started) * 10) / 10);
  }
  const maximumNavigation = Math.max(...timings.navigationMs);
  if (maximumNavigation > 1000) throw new Error(`Navigation interaction exceeded 1000 ms: ${maximumNavigation} ms`);
  assertions.push({ name: "navigation latency under 1000 ms", result: "passed", actual: maximumNavigation });

  await pageCall("clickText", "nav button", "主数据", true);
  await waitUntil("master data page", async () => (await pageCall("topbarTitle")) === "主数据");
  await pageCall("clickText", "button[role=tab]", "产品", false);
  const searchStarted = performance.now();
  await pageCall("setLabel", "搜索主数据", expected.productSku);
  await waitUntil("master search result", () => pageCall("rowContains", expected.productSku));
  timings.masterSearchMs = Math.round((performance.now() - searchStarted) * 10) / 10;
  if (timings.masterSearchMs > 1000) throw new Error(`Master search exceeded 1000 ms: ${timings.masterSearchMs} ms`);
  assertions.push({ name: "master search latency under 1000 ms", result: "passed", actual: timings.masterSearchMs });

  await pageCall("resetScroll");
  const screenshotPath = await screenshot("rc2-seed-complete");
  await pageCall("clickText", "button", "锁定", true);
  await waitUntil("locked workspace", () => pageCall("hasText", "解锁工作区"));
  return { screenshotPath };
}

async function runVerify() {
  await waitUntil("existing workspace unlock", () => pageCall("hasText", "解锁工作区"));
  await pageCall("setLabel", "工作区密码", password);
  const started = performance.now();
  await pageCall("clickText", "button", "解锁", true);
  await waitUntil("workbench after relaunch", async () => (await pageCall("topbarTitle")) === "业务工作台", 30_000);
  timings.unlockMs = Math.round((performance.now() - started) * 10) / 10;
  assertIncludes("persisted sales amount", await pageCall("metric", "销售金额"), "100,000");
  assertIncludes("persisted converted purchase cost", await pageCall("metric", "折算采购成本"), "83,333");

  await pageCall("clickText", "nav button", "主数据", true);
  await waitUntil("master data after relaunch", async () => (await pageCall("topbarTitle")) === "主数据");
  assertEqual("persisted product", await pageCall("rowContains", expected.productSku), true);
  await pageCall("clickText", "button[role=tab]", "客户", false);
  assertEqual("persisted customer", await pageCall("rowContains", expected.customerCode), true);
  await pageCall("clickText", "button[role=tab]", "供应商", false);
  assertEqual("persisted supplier", await pageCall("rowContains", expected.supplierCode), true);

  await pageCall("clickText", "nav button", "业务单", true);
  await waitUntil("business cases after relaunch", async () => (await pageCall("topbarTitle")) === "业务单");
  assertEqual("persisted business case", await pageCall("rowContains", expected.businessNumber), true);

  await pageCall("clickText", "nav button", "采购与生产", true);
  await waitUntil("fulfillment after relaunch", async () => (await pageCall("topbarTitle")) === "采购与生产");
  assertIncludes("persisted purchase total", await pageCall("bodyText"), expected.purchaseTotal);
  assertIncludes("persisted milestone", await pageCall("milestoneState", "原料准备"), "100%");
  const purchaseCompanyLabel = await pageCall("elementRect", ".purchase-output-toolbar .export-language");
  if (!purchaseCompanyLabel || purchaseCompanyLabel.height > 60 || purchaseCompanyLabel.whiteSpace !== "nowrap") {
    throw new Error(`Purchase output label wrapped unexpectedly: ${JSON.stringify(purchaseCompanyLabel)}`);
  }
  assertions.push({ name: "purchase output labels stay on one line", result: "passed", actual: purchaseCompanyLabel });
  await pageCall("resetScroll");
  const screenshotPath = await screenshot("rc2-relaunch-persistence");
  await pageCall("clickText", "button", "锁定", true);
  await waitUntil("locked after verification", () => pageCall("hasText", "解锁工作区"));
  return { screenshotPath };
}

let details;
try {
  details = operation === "seed" ? await runSeed() : operation === "verify" ? await runVerify() : (() => { throw new Error(`Unknown operation: ${operation}`); })();
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, `${JSON.stringify({
    result: "passed", operation, port, timings, assertions, expected, ...details,
    completedAtUtc: new Date().toISOString(),
  }, null, 2)}\n`);
} catch (error) {
  let failureScreenshot = "";
  try { failureScreenshot = await screenshot(`rc2-${operation}-failure`); } catch {}
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, `${JSON.stringify({
    result: "failed", operation, port, timings, assertions, expected,
    error: String(error?.stack || error), failureScreenshot,
    completedAtUtc: new Date().toISOString(),
  }, null, 2)}\n`);
  throw error;
} finally {
  socket.close();
}
