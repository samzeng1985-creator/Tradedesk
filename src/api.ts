import { invoke } from "@tauri-apps/api/core";
import type {
  BusinessCase,
  BusinessCaseInput,
  AttachmentInput,
  AttachmentRecord,
  BackupResult,
  CompanyRegistry,
  CompanyRegistryInput,
  ComponentOption,
  ComponentOptionInput,
  ComponentOptionTranslationInput,
  ConfigurationLanguage,
  ConfigComponent,
  ConfigComponentInput,
  ConfigurableProduct,
  ConfigurableProductInput,
  CostEstimate,
  CostEstimateInput,
  ConvertDocumentInput,
  CreateDocumentInput,
  Customer,
  CustomerInput,
  DocumentExportResult,
  DocumentDraft,
  MasterImportResult,
  Partner,
  PartnerInput,
  PaymentPlan,
  PaymentPlanInput,
  PipelineStage,
  Product,
  ProductInput,
  ProductionMilestone,
  ProductionMilestoneInput,
  PurchaseOrder,
  PurchaseOrderInput,
  PurchaseOrderUpdateInput,
  PurchaseStatus,
  Supplier,
  SupplierInput,
  SaveDocumentInput,
  ShipmentBatch,
  ShipmentBatchInput,
  TradeDocument,
  WorkspaceSummary,
} from "./domain";

export const workspaceApi = {
  exists: () => invoke<boolean>("workspace_exists"),
  unlock: (password: string, companyName?: string) =>
    invoke<WorkspaceSummary>("unlock_workspace", { password, companyName }),
  unlockWithRecovery: (recoveryKey: string) =>
    invoke<WorkspaceSummary>("unlock_workspace_with_recovery", { recoveryKey }),
  lock: () => invoke<void>("lock_workspace"),
  summary: () => invoke<WorkspaceSummary>("workspace_summary"),
  companyRegistry: () => invoke<CompanyRegistry>("get_company_registry"),
  saveCompanyRegistry: (input: CompanyRegistryInput) =>
    invoke<CompanyRegistry>("save_company_registry", { input }),
  rotateRecoveryKey: () => invoke<string>("rotate_recovery_key"),
  createBackup: () => invoke<BackupResult>("create_workspace_backup"),
  restoreBackup: (bytes: number[]) => invoke<void>("restore_workspace_backup", { bytes }),
  restorePending: () => invoke<boolean>("workspace_restore_pending"),
  rollbackRestore: () => invoke<void>("rollback_workspace_restore"),
};

export const attachmentApi = {
  list: () => invoke<AttachmentRecord[]>("list_attachments"),
  listFor: (entityType: string, entityId: string) => invoke<AttachmentRecord[]>("list_entity_attachments", { entityType, entityId }),
  save: (input: AttachmentInput) => invoke<AttachmentRecord>("save_attachment", { input }),
  export: (id: string) => invoke<string>("export_attachment", { id }),
  delete: (id: string) => invoke<void>("delete_attachment", { id }),
};

export const logisticsApi = {
  listPartners: () => invoke<Partner[]>("list_partners"),
  savePartner: (input: PartnerInput) => invoke<Partner>("save_partner", { input }),
  archivePartner: (id: string) => invoke<void>("archive_partner", { id }),
  listShipments: () => invoke<ShipmentBatch[]>("list_shipment_batches"),
  saveShipment: (input: ShipmentBatchInput) => invoke<ShipmentBatch>("save_shipment_batch", { input }),
  listPayments: () => invoke<PaymentPlan[]>("list_payment_plans"),
  savePayment: (input: PaymentPlanInput) => invoke<PaymentPlan>("save_payment_plan", { input }),
};

export const documentApi = {
  list: () => invoke<TradeDocument[]>("list_documents"),
  create: (input: CreateDocumentInput) =>
    invoke<TradeDocument>("create_document", { input }),
  convert: (input: ConvertDocumentInput) =>
    invoke<TradeDocument>("convert_document", { input }),
  save: (input: SaveDocumentInput) =>
    invoke<TradeDocument>("save_document", { input }),
  review: (id: string) => invoke<TradeDocument>("review_document", { id }),
  issue: (id: string) => invoke<TradeDocument>("issue_document", { id }),
  void: (id: string, reason: string) =>
    invoke<TradeDocument>("void_document", { id, reason }),
  archive: (id: string) => invoke<TradeDocument>("archive_document", { id }),
  newVersion: (id: string) =>
    invoke<TradeDocument>("create_document_version", { id }),
  exportPdf: (id: string, companyId: string, signingAssetId: string) =>
    invoke<DocumentExportResult>("export_document_pdf", { id, companyId, signingAssetId }),
  exportCsv: (id: string) => invoke<string>("export_document_csv", { id }),
  print: (id: string, companyId: string, signingAssetId: string) => invoke<DocumentExportResult>("print_document", { id, companyId, signingAssetId }),
  openPdf: (id: string) => invoke<void>("open_document_pdf", { id }),
};

export const documentDraftApi = {
  save: (input: SaveDocumentInput) => invoke<DocumentDraft>("save_document_draft", { input }),
  load: (id: string) => invoke<DocumentDraft | null>("load_document_draft", { id }),
  delete: (id: string) => invoke<void>("delete_document_draft", { id }),
};

export const masterApi = {
  exportWorkbook: (templateOnly = false) => invoke<string>("export_master_data", { templateOnly }),
  importWorkbook: (bytes: number[]) => invoke<MasterImportResult>("import_master_data", { bytes }),
  listProducts: () => invoke<Product[]>("list_products"),
  saveProduct: (input: ProductInput) => invoke<Product>("save_product", { input }),
  listConfigComponents: () => invoke<ConfigComponent[]>("list_config_components"),
  saveConfigComponent: (input: ConfigComponentInput) => invoke<ConfigComponent>("save_config_component", { input }),
  listComponentOptions: () => invoke<ComponentOption[]>("list_component_options"),
  saveComponentOption: (input: ComponentOptionInput) => invoke<ComponentOption>("save_component_option", { input }),
  saveComponentOptionTranslation: (input: ComponentOptionTranslationInput) => invoke<ComponentOption>("save_component_option_translation", { input }),
  listConfigurableProducts: () => invoke<ConfigurableProduct[]>("list_configurable_products"),
  saveConfigurableProduct: (input: ConfigurableProductInput) => invoke<ConfigurableProduct>("save_configurable_product", { input }),
  exportConfigurationPdf: (id: string, language: ConfigurationLanguage, companyId: string, signingAssetId: string) => invoke<DocumentExportResult>("export_configuration_pdf", { id, language, companyId, signingAssetId }),
  exportConfigurationCsv: (id: string, language: ConfigurationLanguage) => invoke<string>("export_configuration_csv", { id, language }),
  printConfiguration: (id: string, language: ConfigurationLanguage, companyId: string, signingAssetId: string) => invoke<DocumentExportResult>("print_configuration", { id, language, companyId, signingAssetId }),
  listCustomers: () => invoke<Customer[]>("list_customers"),
  saveCustomer: (input: CustomerInput) => invoke<Customer>("save_customer", { input }),
  listSuppliers: () => invoke<Supplier[]>("list_suppliers"),
  saveSupplier: (input: SupplierInput) => invoke<Supplier>("save_supplier", { input }),
  archive: (entity: "product" | "config_component" | "component_option" | "configurable_product" | "customer" | "supplier", id: string) =>
    invoke<void>("archive_master", { entity, id }),
};

export const businessCaseApi = {
  list: () => invoke<BusinessCase[]>("list_business_cases"),
  save: (input: BusinessCaseInput) =>
    invoke<BusinessCase>("save_business_case", { input }),
  updateStage: (id: string, stage: PipelineStage) =>
    invoke<BusinessCase>("update_business_case_stage", { id, stage }),
  archive: (id: string) => invoke<void>("archive_business_case", { id }),
};

export const costEstimateApi = {
  list: () => invoke<CostEstimate[]>("list_cost_estimates"),
  save: (input: CostEstimateInput) =>
    invoke<CostEstimate>("save_cost_estimate", { input }),
  archive: (id: string) => invoke<void>("archive_cost_estimate", { id }),
};

export const fulfillmentApi = {
  list: () => invoke<PurchaseOrder[]>("list_purchase_orders"),
  create: (input: PurchaseOrderInput) =>
    invoke<PurchaseOrder>("create_purchase_order", { input }),
  update: (input: PurchaseOrderUpdateInput) =>
    invoke<PurchaseOrder>("update_purchase_order", { input }),
  exportPdf: (id: string, companyId: string, signingAssetId: string) =>
    invoke<DocumentExportResult>("export_purchase_order_pdf", { id, companyId, signingAssetId }),
  exportCsv: (id: string) => invoke<string>("export_purchase_order_csv", { id }),
  print: (id: string, companyId: string, signingAssetId: string) =>
    invoke<DocumentExportResult>("print_purchase_order", { id, companyId, signingAssetId }),
  updateStatus: (id: string, status: PurchaseStatus) =>
    invoke<PurchaseOrder>("update_purchase_order_status", { id, status }),
  updateMilestone: (input: ProductionMilestoneInput) =>
    invoke<ProductionMilestone>("update_production_milestone", { input }),
};
