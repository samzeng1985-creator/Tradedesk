import { invoke } from "@tauri-apps/api/core";
import type {
  BusinessCase,
  BusinessCaseInput,
  ComponentOption,
  ComponentOptionInput,
  ComponentOptionTranslationInput,
  ConfigurationLanguage,
  ConfigComponent,
  ConfigComponentInput,
  ConfigurableProduct,
  ConfigurableProductInput,
  ConvertDocumentInput,
  CreateDocumentInput,
  Customer,
  CustomerInput,
  DocumentExportResult,
  Product,
  ProductInput,
  ProductionMilestone,
  ProductionMilestoneInput,
  PurchaseOrder,
  PurchaseOrderInput,
  PurchaseStatus,
  Supplier,
  SupplierInput,
  SaveDocumentInput,
  TradeDocument,
  WorkspaceSummary,
} from "./domain";

export const workspaceApi = {
  exists: () => invoke<boolean>("workspace_exists"),
  unlock: (password: string, companyName?: string) =>
    invoke<WorkspaceSummary>("unlock_workspace", { password, companyName }),
  lock: () => invoke<void>("lock_workspace"),
  summary: () => invoke<WorkspaceSummary>("workspace_summary"),
};

export const documentApi = {
  list: () => invoke<TradeDocument[]>("list_documents"),
  create: (input: CreateDocumentInput) =>
    invoke<TradeDocument>("create_document", { input }),
  convert: (input: ConvertDocumentInput) =>
    invoke<TradeDocument>("convert_document", { input }),
  save: (input: SaveDocumentInput) =>
    invoke<TradeDocument>("save_document", { input }),
  issue: (id: string) => invoke<TradeDocument>("issue_document", { id }),
  void: (id: string, reason: string) =>
    invoke<TradeDocument>("void_document", { id, reason }),
  newVersion: (id: string) =>
    invoke<TradeDocument>("create_document_version", { id }),
  exportPdf: (id: string) =>
    invoke<DocumentExportResult>("export_document_pdf", { id }),
  exportCsv: (id: string) => invoke<string>("export_document_csv", { id }),
  print: (id: string) => invoke<DocumentExportResult>("print_document", { id }),
  openPdf: (id: string) => invoke<void>("open_document_pdf", { id }),
};

export const masterApi = {
  listProducts: () => invoke<Product[]>("list_products"),
  saveProduct: (input: ProductInput) => invoke<Product>("save_product", { input }),
  listConfigComponents: () => invoke<ConfigComponent[]>("list_config_components"),
  saveConfigComponent: (input: ConfigComponentInput) => invoke<ConfigComponent>("save_config_component", { input }),
  listComponentOptions: () => invoke<ComponentOption[]>("list_component_options"),
  saveComponentOption: (input: ComponentOptionInput) => invoke<ComponentOption>("save_component_option", { input }),
  saveComponentOptionTranslation: (input: ComponentOptionTranslationInput) => invoke<ComponentOption>("save_component_option_translation", { input }),
  listConfigurableProducts: () => invoke<ConfigurableProduct[]>("list_configurable_products"),
  saveConfigurableProduct: (input: ConfigurableProductInput) => invoke<ConfigurableProduct>("save_configurable_product", { input }),
  exportConfigurationPdf: (id: string, language: ConfigurationLanguage) => invoke<DocumentExportResult>("export_configuration_pdf", { id, language }),
  exportConfigurationCsv: (id: string, language: ConfigurationLanguage) => invoke<string>("export_configuration_csv", { id, language }),
  printConfiguration: (id: string, language: ConfigurationLanguage) => invoke<DocumentExportResult>("print_configuration", { id, language }),
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
  archive: (id: string) => invoke<void>("archive_business_case", { id }),
};

export const fulfillmentApi = {
  list: () => invoke<PurchaseOrder[]>("list_purchase_orders"),
  create: (input: PurchaseOrderInput) =>
    invoke<PurchaseOrder>("create_purchase_order", { input }),
  updateStatus: (id: string, status: PurchaseStatus) =>
    invoke<PurchaseOrder>("update_purchase_order_status", { id, status }),
  updateMilestone: (input: ProductionMilestoneInput) =>
    invoke<ProductionMilestone>("update_production_milestone", { input }),
};
