import type {
  Customer,
  Product,
  ProductionMilestone,
  Supplier,
  TradeCase,
  TradeDocument,
} from "./domain";

export const products: Product[] = [
  {
    id: "prd-1",
    sku: "TS-1001",
    nameZh: "不锈钢保温杯",
    nameEn: "Stainless Steel Tumbler",
    model: "750ml",
    hsCode: "9617.00",
    unit: "pcs",
    grossWeightKg: 0.48,
    active: true,
  },
  {
    id: "prd-2",
    sku: "PK-2208",
    nameZh: "食品级硅胶密封圈",
    nameEn: "Silicone Seal Ring",
    model: "82mm",
    hsCode: "3926.90",
    unit: "pcs",
    grossWeightKg: 0.04,
    active: true,
  },
];

export const customers: Customer[] = [
  {
    id: "cus-1",
    code: "CUS-US-018",
    legalName: "Northstar Trading LLC",
    market: "美国",
    currency: "USD",
    paymentTerms: "30% T/T, 70% before shipment",
    active: true,
  },
  {
    id: "cus-2",
    code: "CUS-RU-006",
    legalName: "ООО Север Импорт",
    market: "俄罗斯",
    currency: "USD",
    paymentTerms: "50% / 50%",
    active: true,
  },
];

export const suppliers: Supplier[] = [
  {
    id: "sup-1",
    code: "SUP-ZJ-012",
    legalName: "浙江星河金属制品有限公司",
    leadTimeDays: 25,
    onTimeRate: 96,
    active: true,
  },
  {
    id: "sup-2",
    code: "SUP-GD-021",
    legalName: "东莞远成硅胶科技有限公司",
    leadTimeDays: 18,
    onTimeRate: 91,
    active: true,
  },
];

export const initialCase: TradeCase = {
  id: "case-1",
  number: "TD-2026-0001",
  customer: customers[0],
  stage: "production",
  salesAmount: 42800,
  purchaseAmount: 29650,
  currency: "USD",
  shipmentDate: "2026-09-18",
  productionProgress: 68,
};

export const initialMilestones: ProductionMilestone[] = [
  {
    id: "m-1",
    label: "杯体生产",
    owner: "浙江星河金属制品",
    plannedDate: "2026-08-28",
    progress: 82,
    status: "working",
  },
  {
    id: "m-2",
    label: "密封圈生产",
    owner: "东莞远成硅胶",
    plannedDate: "2026-08-22",
    progress: 64,
    status: "working",
  },
  {
    id: "m-3",
    label: "成品质检",
    owner: "内部质检",
    plannedDate: "2026-09-05",
    progress: 0,
    status: "blocked",
  },
];

export const initialDocuments: TradeDocument[] = [
  {
    id: "doc-1",
    type: "商业报价单",
    number: "QT-2026-0001",
    status: "ready",
    updatedAt: "2026-08-03",
  },
  {
    id: "doc-2",
    type: "外贸合同",
    number: "CT-2026-0001",
    status: "ready",
    updatedAt: "2026-08-04",
  },
  {
    id: "doc-3",
    type: "商业发票",
    number: "INV-2026-0001",
    status: "draft",
    updatedAt: "2026-08-05",
  },
  {
    id: "doc-4",
    type: "详细装箱单",
    number: "PKL-2026-0001",
    status: "blocked",
    updatedAt: "等待最终装箱数据",
  },
];
