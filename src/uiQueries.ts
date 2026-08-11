import type {
  BusinessCase,
  ConfigComponent,
  ConfigurableProduct,
  Customer,
  DocumentStatus,
  DocumentType,
  Product,
  PurchaseOrder,
  Supplier,
  TradeDocument,
} from "./domain";

function normalized(value: string): string {
  return value.trim().toLocaleLowerCase();
}

function containsAny(query: string, values: string[]): boolean {
  const needle = normalized(query);
  return !needle || values.some((value) => value.toLocaleLowerCase().includes(needle));
}

export function filterProducts(items: Product[], query: string): Product[] {
  return items.filter((item) => containsAny(query, [item.sku, item.nameZh, item.nameEn, item.model, item.hsCode]));
}

export function filterCustomers(items: Customer[], query: string): Customer[] {
  return items.filter((item) => containsAny(query, [
    item.code, item.legalName, item.market, item.currency, item.address,
    item.shippingAddress, item.billingAddress, item.purchaseIntent,
    item.customerAnalysis, item.contacts,
  ]));
}

export function filterSuppliers(items: Supplier[], query: string): Supplier[] {
  return items.filter((item) => containsAny(query, [
    item.code, item.legalName, item.address, item.contacts, item.currency,
    item.paymentTerms, item.qualificationNotes,
    ...item.productTerms.flatMap((term) => [term.productSku, term.productName]),
  ]));
}

export function filterBusinessCases(items: BusinessCase[], query: string): BusinessCase[] {
  return items.filter((item) => containsAny(query, [item.number, item.customerName, item.currency, item.incoterm]));
}

export function filterPurchaseOrders(items: PurchaseOrder[], query: string): PurchaseOrder[] {
  return items.filter((item) => containsAny(query, [item.number, item.businessCaseNumber, item.supplierName]));
}

export function filterComponents(items: ConfigComponent[], query: string): ConfigComponent[] {
  return items.filter((item) => containsAny(query, [
    item.code, item.category, item.name, item.specification, item.brand, item.notes,
  ]));
}

export function filterConfigurations(items: ConfigurableProduct[], query: string): ConfigurableProduct[] {
  return items.filter((item) => containsAny(query, [
    item.code, item.name, item.model, item.notes,
    ...item.lines.flatMap((line) => [line.category, line.name, line.specification, line.brand]),
  ]));
}

export interface DocumentFilters {
  query: string;
  type: "all" | DocumentType;
  status: "all" | DocumentStatus;
}

export function filterDocuments(items: TradeDocument[], filters: DocumentFilters): TradeDocument[] {
  return items.filter((item) =>
    (filters.type === "all" || item.documentType === filters.type)
    && (filters.status === "all" || item.status === filters.status)
    && containsAny(filters.query, [item.number, item.customerName, item.businessCaseNumber]),
  );
}
