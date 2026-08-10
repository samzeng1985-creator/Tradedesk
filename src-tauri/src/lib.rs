mod document;
mod domain;
mod storage;

use std::{path::PathBuf, sync::Mutex};

use domain::{
    BusinessCase, BusinessCaseInput, ConvertDocumentInput, CreateDocumentInput, Customer,
    CustomerInput, DocumentExportResult, Product, ProductInput, ProductionMilestone,
    ProductionMilestoneInput, PurchaseOrder, PurchaseOrderInput, PurchaseStatus, SaveDocumentInput,
    Supplier, SupplierInput, TradeDocument, WorkspaceSummary,
};
use storage::EncryptedDatabase;
use tauri::{Manager, State};
use zeroize::Zeroizing;

struct AppState {
    database_path: PathBuf,
    export_dir: PathBuf,
    render_cache_dir: PathBuf,
    typst_path: Option<PathBuf>,
    database: Mutex<Option<EncryptedDatabase>>,
}

fn database_error(error: rusqlite::Error) -> String {
    match error {
        rusqlite::Error::SqliteFailure(details, _)
            if details.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            if details.extended_code == 787 {
                "记录已被采购资料引用，不能删除或替换；可以保留原行并调整未分配数量。".to_owned()
            } else {
                "编号已经存在，请使用其他编号。".to_owned()
            }
        }
        rusqlite::Error::SqliteFailure(details, _)
            if matches!(
                details.code,
                rusqlite::ErrorCode::NotADatabase | rusqlite::ErrorCode::DatabaseCorrupt
            ) =>
        {
            "无法打开加密工作区，请检查密码或数据文件。".to_owned()
        }
        rusqlite::Error::InvalidQuery => "输入内容不完整或格式不正确。".to_owned(),
        rusqlite::Error::QueryReturnedNoRows => "记录不存在或已经停用。".to_owned(),
        other => format!("本地数据库操作失败：{other}"),
    }
}

fn with_database<T>(
    state: State<'_, AppState>,
    action: impl FnOnce(&EncryptedDatabase) -> rusqlite::Result<T>,
) -> Result<T, String> {
    let guard = state
        .database
        .lock()
        .map_err(|_| "工作区状态异常，请重启软件。")?;
    let database = guard.as_ref().ok_or("请先解锁本地工作区。")?;
    action(database).map_err(database_error)
}

#[tauri::command]
fn workspace_exists(state: State<'_, AppState>) -> bool {
    state.database_path.exists()
}

#[tauri::command]
fn unlock_workspace(
    password: String,
    company_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<WorkspaceSummary, String> {
    if password.chars().count() < 8 {
        return Err("工作区密码至少需要 8 个字符。".to_owned());
    }
    if let Some(parent) = state.database_path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| "无法创建本地数据目录。")?;
    }
    let database = EncryptedDatabase::open(&state.database_path, Zeroizing::new(password))
        .map_err(database_error)?;
    if let Some(name) = company_name {
        database.initialize_company(&name).map_err(database_error)?;
    }
    let summary = database.summary().map_err(database_error)?;
    let mut guard = state
        .database
        .lock()
        .map_err(|_| "工作区状态异常，请重启软件。")?;
    *guard = Some(database);
    Ok(summary)
}

#[tauri::command]
fn lock_workspace(state: State<'_, AppState>) -> Result<(), String> {
    let mut guard = state
        .database
        .lock()
        .map_err(|_| "工作区状态异常，请重启软件。")?;
    *guard = None;
    Ok(())
}

#[tauri::command]
fn workspace_summary(state: State<'_, AppState>) -> Result<WorkspaceSummary, String> {
    with_database(state, EncryptedDatabase::summary)
}

#[tauri::command]
fn list_products(state: State<'_, AppState>) -> Result<Vec<Product>, String> {
    with_database(state, EncryptedDatabase::list_products)
}

#[tauri::command]
fn save_product(input: ProductInput, state: State<'_, AppState>) -> Result<Product, String> {
    with_database(state, |database| database.save_product(input))
}

#[tauri::command]
fn list_customers(state: State<'_, AppState>) -> Result<Vec<Customer>, String> {
    with_database(state, EncryptedDatabase::list_customers)
}

#[tauri::command]
fn save_customer(input: CustomerInput, state: State<'_, AppState>) -> Result<Customer, String> {
    with_database(state, |database| database.save_customer(input))
}

#[tauri::command]
fn list_suppliers(state: State<'_, AppState>) -> Result<Vec<Supplier>, String> {
    with_database(state, EncryptedDatabase::list_suppliers)
}

#[tauri::command]
fn save_supplier(input: SupplierInput, state: State<'_, AppState>) -> Result<Supplier, String> {
    with_database(state, |database| database.save_supplier(input))
}

#[tauri::command]
fn archive_master(entity: String, id: String, state: State<'_, AppState>) -> Result<(), String> {
    with_database(state, |database| database.archive(&entity, &id))
}

#[tauri::command]
fn list_business_cases(state: State<'_, AppState>) -> Result<Vec<BusinessCase>, String> {
    with_database(state, EncryptedDatabase::list_business_cases)
}

#[tauri::command]
fn save_business_case(
    input: BusinessCaseInput,
    state: State<'_, AppState>,
) -> Result<BusinessCase, String> {
    with_database(state, |database| database.save_business_case(input))
}

#[tauri::command]
fn archive_business_case(id: String, state: State<'_, AppState>) -> Result<(), String> {
    with_database(state, |database| database.archive_business_case(&id))
}

#[tauri::command]
fn list_purchase_orders(state: State<'_, AppState>) -> Result<Vec<PurchaseOrder>, String> {
    with_database(state, EncryptedDatabase::list_purchase_orders)
}

#[tauri::command]
fn create_purchase_order(
    input: PurchaseOrderInput,
    state: State<'_, AppState>,
) -> Result<PurchaseOrder, String> {
    with_database(state, |database| database.create_purchase_order(input))
}

#[tauri::command]
fn update_purchase_order_status(
    id: String,
    status: PurchaseStatus,
    state: State<'_, AppState>,
) -> Result<PurchaseOrder, String> {
    with_database(state, |database| {
        database.update_purchase_order_status(&id, status)
    })
}

#[tauri::command]
fn update_production_milestone(
    input: ProductionMilestoneInput,
    state: State<'_, AppState>,
) -> Result<ProductionMilestone, String> {
    with_database(state, |database| {
        database.update_production_milestone(input)
    })
}

#[tauri::command]
fn list_documents(state: State<'_, AppState>) -> Result<Vec<TradeDocument>, String> {
    with_database(state, EncryptedDatabase::list_documents)
}

#[tauri::command]
fn create_document(
    input: CreateDocumentInput,
    state: State<'_, AppState>,
) -> Result<TradeDocument, String> {
    with_database(state, |database| database.create_document(input))
}

#[tauri::command]
fn convert_document(
    input: ConvertDocumentInput,
    state: State<'_, AppState>,
) -> Result<TradeDocument, String> {
    with_database(state, |database| database.convert_document(input))
}

#[tauri::command]
fn save_document(
    input: SaveDocumentInput,
    state: State<'_, AppState>,
) -> Result<TradeDocument, String> {
    with_database(state, |database| database.save_document(input))
}

#[tauri::command]
fn issue_document(id: String, state: State<'_, AppState>) -> Result<TradeDocument, String> {
    with_database(state, |database| database.issue_document(&id))
}

#[tauri::command]
fn void_document(
    id: String,
    reason: String,
    state: State<'_, AppState>,
) -> Result<TradeDocument, String> {
    with_database(state, |database| database.void_document(&id, &reason))
}

#[tauri::command]
fn create_document_version(
    id: String,
    state: State<'_, AppState>,
) -> Result<TradeDocument, String> {
    with_database(state, |database| database.create_document_version(&id))
}

fn export_pdf(id: &str, state: &State<'_, AppState>) -> Result<DocumentExportResult, String> {
    let document = with_database(state.clone(), |database| database.get_document(id))?;
    let typst_path = state
        .typst_path
        .as_ref()
        .ok_or("未找到 Typst PDF 渲染器，请重新运行开发环境安装脚本。")?;
    let mut result = document::export_pdf(
        &document,
        typst_path,
        &state.render_cache_dir.join(id),
        &state.export_dir,
    )?;
    let updated = with_database(state.clone(), |database| {
        database.update_document_export(id, &result.path, &result.sha256)
    })?;
    result.exported_at = updated.exported_at;
    Ok(result)
}

#[tauri::command]
fn export_document_pdf(
    id: String,
    state: State<'_, AppState>,
) -> Result<DocumentExportResult, String> {
    export_pdf(&id, &state)
}

#[tauri::command]
fn export_document_csv(id: String, state: State<'_, AppState>) -> Result<String, String> {
    let document = with_database(state.clone(), |database| database.get_document(&id))?;
    document::export_csv(&document, &state.export_dir)
        .map(|path| path.to_string_lossy().into_owned())
}

#[tauri::command]
fn print_document(id: String, state: State<'_, AppState>) -> Result<DocumentExportResult, String> {
    let result = export_pdf(&id, &state)?;
    document::open_file(std::path::Path::new(&result.path))?;
    Ok(result)
}

#[tauri::command]
fn open_document_pdf(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let document = with_database(state, |database| database.get_document(&id))?;
    if document.pdf_path.trim().is_empty() {
        return Err("该版本尚未导出 PDF。".to_owned());
    }
    document::open_file(std::path::Path::new(&document.pdf_path))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let database_path = app_data_dir.join("workspace.tdesk");
            let export_dir = app
                .path()
                .document_dir()
                .unwrap_or_else(|_| app_data_dir.clone())
                .join("TradeDesk Exports");
            let render_cache_dir = app.path().app_cache_dir()?.join("document-render");
            let executable_dir = std::env::current_exe()?
                .parent()
                .map(PathBuf::from)
                .unwrap_or_else(|| app_data_dir.clone());
            app.manage(AppState {
                database_path,
                export_dir,
                render_cache_dir,
                typst_path: document::find_typst(&executable_dir),
                database: Mutex::new(None),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            workspace_exists,
            unlock_workspace,
            lock_workspace,
            workspace_summary,
            list_products,
            save_product,
            list_customers,
            save_customer,
            list_suppliers,
            save_supplier,
            archive_master,
            list_business_cases,
            save_business_case,
            archive_business_case,
            list_purchase_orders,
            create_purchase_order,
            update_purchase_order_status,
            update_production_milestone,
            list_documents,
            create_document,
            convert_document,
            save_document,
            issue_document,
            void_document,
            create_document_version,
            export_document_pdf,
            export_document_csv,
            print_document,
            open_document_pdf,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run TradeDesk Local");
}
