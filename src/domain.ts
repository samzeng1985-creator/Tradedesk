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
  nameZh: string;
  nameEn: string;
  model: string;
  hsCode: string;
  unit: string;
  grossWeightKg: number;
  active: boolean;
}

export type ProductInput = Omit<Product, "active" | "id"> & { id?: string };

export interface Customer {
  id: string;
  code: string;
  legalName: string;
  market: string;
  currency: string;
  paymentTerms: string;
  active: boolean;
}

export type CustomerInput = Omit<Customer, "active" | "id"> & { id?: string };

export interface Supplier {
  id: string;
  code: string;
  legalName: string;
  leadTimeDays: number;
  onTimeRate: number;
  active: boolean;
}

export type SupplierInput = Omit<Supplier, "active" | "id"> & { id?: string };

export interface WorkspaceSummary {
  companyName: string;
  encrypted: boolean;
  products: number;
  customers: number;
  suppliers: number;
  activeCases: number;
}

export interface BusinessCaseLine {
  id: string;
  productId: string;
  sku: string;
  nameZh: string;
  nameEn: string;
  quantity: number;
  unit: string;
  unitPriceMinor: number;
  amountMinor: number;
}

export interface BusinessCaseLineInput {
  productId: string;
  quantity: number;
  unitPriceMinor: number;
}

export interface BusinessCase {
  id: string;
  number: string;
  customerId: string;
  customerName: string;
  stage: PipelineStage;
  currency: string;
  incoterm: string;
  paymentTerms: string;
  shipmentDate: string;
  notes: string;
  totalAmountMinor: number;
  lines: BusinessCaseLine[];
}

export interface BusinessCaseInput {
  id?: string;
  number: string;
  customerId: string;
  stage: PipelineStage;
  currency: string;
  incoterm: string;
  paymentTerms: string;
  shipmentDate: string;
  notes: string;
  lines: BusinessCaseLineInput[];
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
