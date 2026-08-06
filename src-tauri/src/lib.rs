mod document;
mod domain;
mod storage;

use std::{path::PathBuf, sync::Mutex};

use domain::{
    BusinessCase, BusinessCaseInput, Customer, CustomerInput, Product, ProductInput,
    ProductionMilestone, ProductionMilestoneInput, PurchaseOrder, PurchaseOrderInput,
    PurchaseStatus, Supplier, SupplierInput, WorkspaceSummary,
};
use storage::EncryptedDatabase;
use tauri::{Manager, State};
use zeroize::Zeroizing;

struct AppState {
    database_path: PathBuf,
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let database_path = app.path().app_data_dir()?.join("workspace.tdesk");
            app.manage(AppState {
                database_path,
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
        ])
        .run(tauri::generate_context!())
        .expect("failed to run TradeDesk Local");
}
