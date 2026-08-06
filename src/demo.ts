import type { TradeDocument } from "./domain";

// 单证中心将在下一阶段接入真实快照；当前仅保留四类纵向切片入口。
export const initialDocuments: TradeDocument[] = [
  { id: "doc-1", type: "商业报价单", number: "QT-待生成", status: "draft", updatedAt: "从业务单生成" },
  { id: "doc-2", type: "外贸合同", number: "CT-待生成", status: "draft", updatedAt: "从业务单生成" },
  { id: "doc-3", type: "商业发票", number: "INV-待生成", status: "draft", updatedAt: "等待装运资料" },
  { id: "doc-4", type: "详细装箱单", number: "PKL-待生成", status: "blocked", updatedAt: "等待最终装箱数据" },
];
