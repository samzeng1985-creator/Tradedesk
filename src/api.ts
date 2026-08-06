import { invoke } from "@tauri-apps/api/core";
import type {
  BusinessCase,
  BusinessCaseInput,
  Customer,
  CustomerInput,
  Product,
  ProductInput,
  Supplier,
  SupplierInput,
  WorkspaceSummary,
} from "./domain";

export const workspaceApi = {
  exists: () => invoke<boolean>("workspace_exists"),
  unlock: (password: string, companyName?: string) =>
    invoke<WorkspaceSummary>("unlock_workspace", { password, companyName }),
  lock: () => invoke<void>("lock_workspace"),
  summary: () => invoke<WorkspaceSummary>("workspace_summary"),
};

export const masterApi = {
  listProducts: () => invoke<Product[]>("list_products"),
  saveProduct: (input: ProductInput) => invoke<Product>("save_product", { input }),
  listCustomers: () => invoke<Customer[]>("list_customers"),
  saveCustomer: (input: CustomerInput) => invoke<Customer>("save_customer", { input }),
  listSuppliers: () => invoke<Supplier[]>("list_suppliers"),
  saveSupplier: (input: SupplierInput) => invoke<Supplier>("save_supplier", { input }),
  archive: (entity: "product" | "customer" | "supplier", id: string) =>
    invoke<void>("archive_master", { entity, id }),
};

export const businessCaseApi = {
  list: () => invoke<BusinessCase[]>("list_business_cases"),
  save: (input: BusinessCaseInput) =>
    invoke<BusinessCase>("save_business_case", { input }),
  archive: (id: string) => invoke<void>("archive_business_case", { id }),
};
