export type PipelineStage =
  | "quotation"
  | "order"
  | "purchase"
  | "production"
  | "shipment"
  | "documents";

export type RecordStatus = "ready" | "working" | "blocked" | "draft";

export interface Product {
  id: string;
  sku: string;
  name: string;
  model: string;
  hsCode: string;
  unit: string;
  grossWeightKg: number;
}

export interface Customer {
  id: string;
  code: string;
  name: string;
  market: string;
  currency: string;
  paymentTerms: string;
}

export interface Supplier {
  id: string;
  code: string;
  name: string;
  leadTimeDays: number;
  onTimeRate: number;
  status: RecordStatus;
}

export interface ProductionMilestone {
  id: string;
  label: string;
  owner: string;
  plannedDate: string;
  progress: number;
  status: RecordStatus;
}

export interface TradeDocument {
  id: string;
  type: string;
  number: string;
  status: RecordStatus;
  updatedAt: string;
}

export interface TradeCase {
  id: string;
  number: string;
  customer: Customer;
  stage: PipelineStage;
  salesAmount: number;
  purchaseAmount: number;
  currency: string;
  shipmentDate: string;
  productionProgress: number;
}
