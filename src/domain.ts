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

export interface ConfigComponent {
  id: string;
  code: string;
  category: string;
  name: string;
  specification: string;
  defaultQuantity: number;
  unit: string;
  unitPriceMinor: number;
  currency: string;
  brand: string;
  notes: string;
  active: boolean;
}

export type ConfigComponentInput = Omit<ConfigComponent, "active" | "id"> & { id?: string };

export type ComponentOptionKind =
  | "category"
  | "name"
  | "brand"
  | "specification"
  | "unit"
  | "notes"
  | "product_name"
  | "configuration_notes";

export type ConfigurationLanguage = "en" | "ru" | "fr" | "es" | "pt" | "ar";

export interface ComponentOption {
  id: string;
  kind: ComponentOptionKind;
  value: string;
  active: boolean;
  translations: Partial<Record<ConfigurationLanguage, string>>;
}

export interface ComponentOptionInput {
  id?: string;
  kind: ComponentOptionKind;
  value: string;
}

export interface ComponentOptionTranslationInput {
  optionId: string;
  language: ConfigurationLanguage;
  value: string;
}

export interface ConfigurableProductLine {
  id: string;
  componentId: string;
  category: string;
  name: string;
  specification: string;
  quantity: number;
  unit: string;
  unitPriceMinor: number;
  brand: string;
  notes: string;
  amountMinor: number;
}

export interface ConfigurableProductLineInput {
  componentId: string;
  quantity: number;
  unitPriceMinor: number;
}

export interface ConfigurableProduct {
  id: string;
  code: string;
  name: string;
  model: string;
  currency: string;
  exchangeRate: number;
  exchangeRateDate: string;
  notes: string;
  totalAmountMinor: number;
  active: boolean;
  lines: ConfigurableProductLine[];
}

export interface ConfigurableProductInput {
  id?: string;
  code: string;
  name: string;
  model: string;
  currency: string;
  exchangeRate: number;
  exchangeRateDate: string;
  notes: string;
  lines: ConfigurableProductLineInput[];
}

export interface Customer {
  id: string;
  code: string;
  legalName: string;
  market: string;
  currency: string;
  paymentTerms: string;
  address: string;
  shippingAddress: string;
  billingAddress: string;
  purchaseIntent: string;
  customerAnalysis: string;
  strengths: string;
  weaknesses: string;
  contacts: string;
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
  purchaseOrders: number;
  productionRisks: number;
  documents: number;
}

export interface BusinessCaseLine {
  id: string;
  sourceType: "product" | "configurable_product";
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
  id?: string;
  sourceType: "product" | "configurable_product";
  productId: string;
  quantity: number;
  unitPriceMinor: number;
}

export type PurchaseStatus =
  | "draft"
  | "pending_confirmation"
  | "confirmed"
  | "in_production"
  | "ready_to_ship"
  | "completed"
  | "cancelled";

export type MilestoneStatus = "pending" | "in_progress" | "completed" | "blocked";

export interface ProductionMilestone {
  id: string;
  purchaseOrderLineId: string;
  stage: string;
  label: string;
  plannedDate: string;
  actualDate: string;
  progress: number;
  completedQuantity: number;
  status: MilestoneStatus;
  issue: string;
}

export interface ProductionMilestoneInput {
  id: string;
  plannedDate: string;
  actualDate: string;
  progress: number;
  completedQuantity: number;
  status: MilestoneStatus;
  issue: string;
}

export interface PurchaseOrderLine {
  id: string;
  sourceCaseLineId: string;
  productId: string;
  sku: string;
  nameZh: string;
  nameEn: string;
  quantity: number;
  unit: string;
  unitCostMinor: number;
  amountMinor: number;
  milestones: ProductionMilestone[];
}

export interface PurchaseOrderLineInput {
  sourceCaseLineId: string;
  quantity: number;
  unitCostMinor: number;
}

export interface PurchaseOrder {
  id: string;
  number: string;
  businessCaseId: string;
  businessCaseNumber: string;
  supplierId: string;
  supplierName: string;
  status: PurchaseStatus;
  currency: string;
  expectedDate: string;
  notes: string;
  totalAmountMinor: number;
  completedQuantity: number;
  readyQuantity: number;
  lines: PurchaseOrderLine[];
}

export interface PurchaseOrderInput {
  number: string;
  businessCaseId: string;
  supplierId: string;
  currency: string;
  expectedDate: string;
  notes: string;
  lines: PurchaseOrderLineInput[];
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

export type DocumentType =
  | "commercial_quotation"
  | "proforma_invoice"
  | "commercial_invoice"
  | "packing_list"
  | "trade_contract";
export type DocumentStatus = "draft" | "issued" | "voided";
export type ValidationSeverity = "error" | "warning";

export interface DocumentValidationIssue {
  severity: ValidationSeverity;
  code: string;
  message: string;
}

export interface DocumentLineSnapshot {
  productId: string;
  sku: string;
  description: string;
  model: string;
  hsCode: string;
  quantity: number;
  unit: string;
  unitPriceMinor: number;
  amountMinor: number;
  packages: number;
  packageType: string;
  netWeightKg: number;
  grossWeightKg: number;
  cbm: number;
}

export interface DocumentPayload {
  seller: string;
  sellerAddress: string;
  buyer: string;
  buyerAddress: string;
  originCountry: string;
  destinationCountry: string;
  portOfLoading: string;
  portOfDischarge: string;
  incoterm: string;
  paymentTerms: string;
  shipmentDate: string;
  poReference: string;
  validUntil: string;
  discountMinor: number;
  bankDetails: string;
  notes: string;
  declaration: string;
  contractTerms: string;
  lines: DocumentLineSnapshot[];
}

export interface TradeDocument {
  id: string;
  documentType: DocumentType;
  number: string;
  businessCaseId: string;
  businessCaseNumber: string;
  customerName: string;
  version: number;
  status: DocumentStatus;
  language: string;
  issueDate: string;
  currency: string;
  templateVersion: string;
  payload: DocumentPayload;
  validationIssues: DocumentValidationIssue[];
  voidReason: string;
  pdfPath: string;
  pdfSha256: string;
  exportedAt: string;
  createdAt: string;
  updatedAt: string;
}

export interface CreateDocumentInput {
  businessCaseId: string;
  documentType: DocumentType;
  number: string;
  language: string;
  issueDate: string;
}

export interface ConvertDocumentInput {
  sourceDocumentId: string;
  targetDocumentType: DocumentType;
  number: string;
  language: string;
  issueDate: string;
}

export interface SaveDocumentInput {
  id: string;
  number: string;
  language: string;
  issueDate: string;
  payload: DocumentPayload;
}

export interface DocumentExportResult {
  path: string;
  sha256: string;
  exportedAt: string;
}
