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
  recoveryKey: string;
  recoveryReady: boolean;
}

export interface BackupResult {
  path: string;
  sizeBytes: number;
  createdAt: string;
}

export interface AttachmentRecord {
  id: string;
  entityType: string;
  entityId: string;
  entityLabel: string;
  fileName: string;
  mimeType: string;
  sizeBytes: number;
  sha256: string;
  createdAt: string;
}

export interface AttachmentInput {
  entityType: string;
  entityId: string;
  entityLabel: string;
  fileName: string;
  mimeType: string;
  bytes: number[];
}

export interface CompanySigningAsset {
  id: string;
  name: string;
  kind: "signature" | "stamp";
  dataUrl: string;
}

export interface CompanyRecord {
  id: string;
  companyName: string;
  logoDataUrl: string;
  signingAssets: CompanySigningAsset[];
}

export interface CompanyRegistry {
  defaultCompanyId: string;
  companies: CompanyRecord[];
}

export type CompanyRegistryInput = CompanyRegistry;

export interface MasterImportResult {
  products: number;
  customers: number;
  suppliers: number;
  components: number;
  configurations: number;
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
  exchangeRate: number;
  exchangeRateDate: string;
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
  exchangeRate: number;
  exchangeRateDate: string;
  expectedDate: string;
  notes: string;
  lines: PurchaseOrderLineInput[];
}

export interface PurchaseOrderUpdateInput {
  id: string;
  supplierId: string;
  currency: string;
  exchangeRate: number;
  exchangeRateDate: string;
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

export type CostCategory =
  | "material"
  | "processing"
  | "packaging"
  | "domestic_logistics"
  | "international_freight"
  | "duty_tax"
  | "commission"
  | "insurance"
  | "certification"
  | "other";

export interface CostEstimateLine {
  id: string;
  category: CostCategory;
  description: string;
  specification: string;
  quantity: number;
  unit: string;
  unitCostMinor: number;
  amountMinor: number;
  notes: string;
}

export interface CostEstimateLineInput {
  id?: string;
  category: CostCategory;
  description: string;
  specification: string;
  quantity: number;
  unit: string;
  unitCostMinor: number;
  notes: string;
}

export interface CostEstimate {
  id: string;
  number: string;
  businessCaseId: string;
  businessCaseNumber: string;
  customerName: string;
  currency: string;
  targetMarginBps: number;
  notes: string;
  totalCostMinor: number;
  suggestedPriceMinor: number;
  updatedAt: string;
  lines: CostEstimateLine[];
}

export interface CostEstimateInput {
  id?: string;
  number: string;
  businessCaseId: string;
  targetMarginBps: number;
  notes: string;
  lines: CostEstimateLineInput[];
}

export type PartnerType = "freight_forwarder" | "customs_broker" | "insurer" | "inspection" | "other";

export interface Partner {
  id: string;
  code: string;
  legalName: string;
  partnerType: PartnerType;
  contact: string;
  address: string;
  active: boolean;
}

export type PartnerInput = Omit<Partner, "id" | "active"> & { id?: string };

export type ShipmentStatus = "planned" | "booked" | "shipped" | "delivered" | "cancelled";

export interface ShipmentLine {
  id: string;
  businessCaseLineId: string;
  sku: string;
  productName: string;
  quantity: number;
  unit: string;
}

export interface ShipmentLineInput {
  businessCaseLineId: string;
  quantity: number;
}

export interface ShipmentBatch {
  id: string;
  number: string;
  businessCaseId: string;
  businessCaseNumber: string;
  partnerId: string;
  partnerName: string;
  status: ShipmentStatus;
  plannedDate: string;
  actualDate: string;
  trackingNumber: string;
  notes: string;
  lines: ShipmentLine[];
}

export interface ShipmentBatchInput {
  id?: string;
  number: string;
  businessCaseId: string;
  partnerId: string;
  status: ShipmentStatus;
  plannedDate: string;
  actualDate: string;
  trackingNumber: string;
  notes: string;
  lines: ShipmentLineInput[];
}

export type PaymentStatus = "planned" | "partial" | "received" | "cancelled";
export type PaymentType = "deposit" | "balance" | "installment" | "other";

export interface PaymentPlan {
  id: string;
  number: string;
  businessCaseId: string;
  businessCaseNumber: string;
  paymentType: PaymentType;
  dueDate: string;
  currency: string;
  amountMinor: number;
  receivedAmountMinor: number;
  receivedDate: string;
  status: PaymentStatus;
  notes: string;
}

export interface PaymentPlanInput {
  id?: string;
  number: string;
  businessCaseId: string;
  paymentType: PaymentType;
  dueDate: string;
  amountMinor: number;
  receivedAmountMinor: number;
  receivedDate: string;
  status: PaymentStatus;
  notes: string;
}

export type DocumentType =
  | "commercial_quotation"
  | "proforma_invoice"
  | "commercial_invoice"
  | "packing_list"
  | "trade_contract"
  | "shipping_marks"
  | "shipper_instruction"
  | "customs_declaration"
  | "bill_of_lading"
  | "insurance_policy"
  | "certificate_of_origin"
  | "inspection_certificate"
  | "fumigation_certificate"
  | "beneficiary_certificate";
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
  shippingMarks: string;
  transportMode: string;
  vesselVoyage: string;
  bookingReference: string;
  freightTerms: string;
  billOfLadingType: string;
  customsSupervisionCode: string;
  customsDeclarationElements: string;
  notifyParty: string;
  notifyPartyAddress: string;
  carrier: string;
  billOfLadingNumber: string;
  placeOfReceipt: string;
  placeOfDelivery: string;
  containerNumbers: string;
  sealNumbers: string;
  insuranceCompany: string;
  policyNumber: string;
  insuredValueMinor: number;
  insuranceMarkupPercent: number;
  premiumRatePercent: number;
  premiumMinor: number;
  insuranceCoverage: string;
  claimsPayableAt: string;
  certificateNumber: string;
  certificateType: string;
  certificationAuthority: string;
  manufacturer: string;
  manufacturerAddress: string;
  batchNumber: string;
  inspectionStandard: string;
  inspectionDate: string;
  inspectionPlace: string;
  inspectionResult: string;
  fumigationAgent: string;
  fumigationMethod: string;
  fumigationTemperatureCelsius: number;
  fumigationDurationHours: number;
  fumigationDate: string;
  fumigationPlace: string;
  fumigationOperator: string;
  fumigationLicenseNumber: string;
  letterOfCreditNumber: string;
  issuingBank: string;
  letterOfCreditIssueDate: string;
  letterOfCreditExpiryDate: string;
  presentationDeadline: string;
  beneficiaryCertificateType: string;
  beneficiaryStatement: string;
  letterOfCreditTerms: string;
  requiredDocuments: string;
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

export interface DocumentDraft {
  input: SaveDocumentInput;
  updatedAt: string;
}

export interface DocumentExportResult {
  path: string;
  sha256: string;
  exportedAt: string;
}
