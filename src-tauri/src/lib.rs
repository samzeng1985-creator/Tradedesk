mod document;
mod domain;
mod security;
mod spreadsheet;
mod storage;

use std::{
    path::PathBuf,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use domain::{
    AttachmentInput, AttachmentRecord, BackupResult, BusinessCase, BusinessCaseInput,
    CompanyRegistry, ComponentOption, ComponentOptionInput, ComponentOptionTranslationInput,
    ConfigComponent, ConfigComponentInput, ConfigurableProduct, ConfigurableProductInput,
    ConvertDocumentInput, CreateDocumentInput, Customer, CustomerInput, DocumentDraft,
    DocumentExportResult, Partner, PartnerInput, PaymentPlan, PaymentPlanInput, Product,
    ProductInput, ProductionMilestone, ProductionMilestoneInput, PurchaseOrder, PurchaseOrderInput,
    PurchaseStatus, SaveDocumentInput, ShipmentBatch, ShipmentBatchInput, Supplier, SupplierInput,
    TradeDocument, WorkspaceSummary,
};
use storage::EncryptedDatabase;
use tauri::{Manager, State};
use uuid::Uuid;
use zeroize::Zeroizing;

struct AppState {
    database_path: PathBuf,
    recovery_path: PathBuf,
    backup_dir: PathBuf,
    backup_cache_dir: PathBuf,
    attachment_export_dir: PathBuf,
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

fn open_workspace(
    password: Zeroizing<String>,
    company_name: Option<String>,
    state: &State<'_, AppState>,
) -> Result<WorkspaceSummary, String> {
    if password.chars().count() < 8 {
        return Err("工作区密码至少需要 8 个字符。".to_owned());
    }
    if let Some(parent) = state.database_path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| "无法创建本地数据目录。")?;
    }
    let database =
        EncryptedDatabase::open(&state.database_path, password).map_err(database_error)?;
    if let Some(name) = company_name {
        database.initialize_company(&name).map_err(database_error)?;
    }
    let mut summary = database.summary().map_err(database_error)?;
    if security::recovery_vault_exists(&state.recovery_path) {
        summary.recovery_ready = true;
    } else {
        summary.recovery_key = security::create_recovery_vault(
            &state.recovery_path,
            database.recovery_secret().as_str(),
        )?;
        summary.recovery_ready = true;
    }
    security::commit_restored_workspace(&state.database_path, &state.recovery_path)?;
    let mut guard = state
        .database
        .lock()
        .map_err(|_| "工作区状态异常，请重启软件。")?;
    *guard = Some(database);
    Ok(summary)
}

#[tauri::command]
fn unlock_workspace(
    password: String,
    company_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<WorkspaceSummary, String> {
    open_workspace(Zeroizing::new(password), company_name, &state)
}

#[tauri::command]
fn unlock_workspace_with_recovery(
    recovery_key: String,
    state: State<'_, AppState>,
) -> Result<WorkspaceSummary, String> {
    let password = security::recover_password(&state.recovery_path, Zeroizing::new(recovery_key))?;
    open_workspace(password, None, &state)
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
    let mut summary = with_database(state.clone(), EncryptedDatabase::summary)?;
    summary.recovery_ready = security::recovery_vault_exists(&state.recovery_path);
    Ok(summary)
}

#[tauri::command]
fn rotate_recovery_key(state: State<'_, AppState>) -> Result<String, String> {
    let password = with_database(state.clone(), |database| Ok(database.recovery_secret()))?;
    security::create_recovery_vault(&state.recovery_path, password.as_str())
}

#[tauri::command]
fn create_workspace_backup(state: State<'_, AppState>) -> Result<BackupResult, String> {
    std::fs::create_dir_all(&state.backup_dir)
        .map_err(|error| format!("无法创建备份目录：{error}"))?;
    std::fs::create_dir_all(&state.backup_cache_dir)
        .map_err(|error| format!("无法创建备份缓存目录：{error}"))?;
    if !security::recovery_vault_exists(&state.recovery_path) {
        return Err("请先生成恢复密钥，再创建备份。".to_owned());
    }
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "系统时间无效，无法创建备份。".to_owned())?
        .as_secs();
    let database_backup = state
        .backup_cache_dir
        .join(format!("workspace-{stamp}.tdesk"));
    let output = state
        .backup_dir
        .join(format!("TradeDesk-backup-{stamp}.tdbackup"));
    with_database(state.clone(), |database| {
        database.backup_to(&database_backup)
    })?;
    let size_bytes =
        security::create_backup_package(&database_backup, &state.recovery_path, &output);
    let _ = std::fs::remove_file(&database_backup);
    Ok(BackupResult {
        path: output.to_string_lossy().into_owned(),
        size_bytes: size_bytes?,
        created_at: stamp.to_string(),
    })
}

#[tauri::command]
fn restore_workspace_backup(bytes: Vec<u8>, state: State<'_, AppState>) -> Result<(), String> {
    let guard = state
        .database
        .lock()
        .map_err(|_| "工作区状态异常，请重启软件。")?;
    if guard.is_some() {
        return Err("恢复备份前请先锁定工作区。".to_owned());
    }
    drop(guard);
    security::restore_backup_package(&bytes, &state.database_path, &state.recovery_path)
}

#[tauri::command]
fn workspace_restore_pending(state: State<'_, AppState>) -> bool {
    security::restore_pending(&state.database_path)
}

#[tauri::command]
fn rollback_workspace_restore(state: State<'_, AppState>) -> Result<(), String> {
    let guard = state
        .database
        .lock()
        .map_err(|_| "工作区状态异常，请重启软件。")?;
    if guard.is_some() {
        return Err("撤销恢复前请先锁定工作区。".to_owned());
    }
    drop(guard);
    security::rollback_restored_workspace(&state.database_path, &state.recovery_path)
}

#[tauri::command]
fn list_attachments(state: State<'_, AppState>) -> Result<Vec<AttachmentRecord>, String> {
    with_database(state, EncryptedDatabase::list_attachments)
}

#[tauri::command]
fn list_entity_attachments(
    entity_type: String,
    entity_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<AttachmentRecord>, String> {
    with_database(state, |database| {
        database.list_entity_attachments(&entity_type, &entity_id)
    })
}

#[tauri::command]
fn save_attachment(
    input: AttachmentInput,
    state: State<'_, AppState>,
) -> Result<AttachmentRecord, String> {
    with_database(state, |database| database.save_attachment(input))
}

fn safe_file_name(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if matches!(
                character,
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
            ) {
                '_'
            } else {
                character
            }
        })
        .collect::<String>()
        .trim()
        .to_owned()
}

#[tauri::command]
fn export_attachment(id: String, state: State<'_, AppState>) -> Result<String, String> {
    let (file_name, bytes) =
        with_database(state.clone(), |database| database.attachment_content(&id))?;
    std::fs::create_dir_all(&state.attachment_export_dir)
        .map_err(|error| format!("无法创建附件导出目录：{error}"))?;
    let safe_name = safe_file_name(&file_name);
    let prefix = id.get(..8).unwrap_or(&id);
    let output = state
        .attachment_export_dir
        .join(format!("{prefix}-{safe_name}"));
    std::fs::write(&output, bytes).map_err(|error| format!("无法导出附件：{error}"))?;
    Ok(output.to_string_lossy().into_owned())
}

#[tauri::command]
fn delete_attachment(id: String, state: State<'_, AppState>) -> Result<(), String> {
    with_database(state, |database| database.delete_attachment(&id))
}

#[tauri::command]
fn save_document_draft(
    input: SaveDocumentInput,
    state: State<'_, AppState>,
) -> Result<DocumentDraft, String> {
    with_database(state, |database| database.save_document_draft(input))
}

#[tauri::command]
fn load_document_draft(
    id: String,
    state: State<'_, AppState>,
) -> Result<Option<DocumentDraft>, String> {
    with_database(state, |database| database.load_document_draft(&id))
}

#[tauri::command]
fn delete_document_draft(id: String, state: State<'_, AppState>) -> Result<(), String> {
    with_database(state, |database| database.delete_document_draft(&id))
}

#[tauri::command]
fn get_company_registry(state: State<'_, AppState>) -> Result<CompanyRegistry, String> {
    with_database(state, EncryptedDatabase::company_registry)
}

#[tauri::command]
fn save_company_registry(
    input: CompanyRegistry,
    state: State<'_, AppState>,
) -> Result<CompanyRegistry, String> {
    document::validate_company_registry(&input)?;
    with_database(state, |database| database.save_company_registry(input))
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
fn list_config_components(state: State<'_, AppState>) -> Result<Vec<ConfigComponent>, String> {
    with_database(state, EncryptedDatabase::list_config_components)
}

#[tauri::command]
fn save_config_component(
    input: ConfigComponentInput,
    state: State<'_, AppState>,
) -> Result<ConfigComponent, String> {
    with_database(state, |database| database.save_config_component(input))
}

#[tauri::command]
fn list_component_options(state: State<'_, AppState>) -> Result<Vec<ComponentOption>, String> {
    with_database(state, EncryptedDatabase::list_component_options)
}

#[tauri::command]
fn save_component_option(
    input: ComponentOptionInput,
    state: State<'_, AppState>,
) -> Result<ComponentOption, String> {
    with_database(state, |database| database.save_component_option(input))
}

#[tauri::command]
fn save_component_option_translation(
    input: ComponentOptionTranslationInput,
    state: State<'_, AppState>,
) -> Result<ComponentOption, String> {
    with_database(state, |database| {
        database.save_component_option_translation(input)
    })
}

#[tauri::command]
fn list_configurable_products(
    state: State<'_, AppState>,
) -> Result<Vec<ConfigurableProduct>, String> {
    with_database(state, EncryptedDatabase::list_configurable_products)
}

#[tauri::command]
fn save_configurable_product(
    input: ConfigurableProductInput,
    state: State<'_, AppState>,
) -> Result<ConfigurableProduct, String> {
    with_database(state, |database| database.save_configurable_product(input))
}

fn export_configuration_pdf_file(
    id: &str,
    language: &str,
    company_id: &str,
    signing_asset_id: &str,
    state: &State<'_, AppState>,
) -> Result<DocumentExportResult, String> {
    let (configuration, missing, company_profile) = with_database(state.clone(), |database| {
        let (configuration, missing) = database.configuration_for_export(id, language)?;
        Ok((
            configuration,
            missing,
            database.resolve_company_profile(company_id, signing_asset_id)?,
        ))
    })?;
    if !missing.is_empty() {
        return Err(format!(
            "请先在“组件库 → 词库设置”补齐所选语言译文：{}",
            missing.join("；")
        ));
    }
    let typst_path = state
        .typst_path
        .as_ref()
        .ok_or("未找到 Typst PDF 渲染器，请重新运行开发环境安装脚本。")?;
    document::export_configuration_pdf(
        &configuration,
        language,
        &company_profile,
        typst_path,
        &state.render_cache_dir.join("configuration").join(id),
        &state.export_dir,
    )
}

#[tauri::command]
fn export_configuration_pdf(
    id: String,
    language: String,
    company_id: String,
    signing_asset_id: String,
    state: State<'_, AppState>,
) -> Result<DocumentExportResult, String> {
    export_configuration_pdf_file(&id, &language, &company_id, &signing_asset_id, &state)
}

#[tauri::command]
fn export_configuration_csv(
    id: String,
    language: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let (configuration, missing) = with_database(state.clone(), |database| {
        database.configuration_for_export(&id, &language)
    })?;
    if !missing.is_empty() {
        return Err(format!(
            "请先在“组件库 → 词库设置”补齐所选语言译文：{}",
            missing.join("；")
        ));
    }
    document::export_configuration_csv(&configuration, &language, &state.export_dir)
        .map(|path| path.to_string_lossy().into_owned())
}

#[tauri::command]
fn print_configuration(
    id: String,
    language: String,
    company_id: String,
    signing_asset_id: String,
    state: State<'_, AppState>,
) -> Result<DocumentExportResult, String> {
    let result =
        export_configuration_pdf_file(&id, &language, &company_id, &signing_asset_id, &state)?;
    document::open_file(std::path::Path::new(&result.path))?;
    Ok(result)
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
fn export_master_data(template_only: bool, state: State<'_, AppState>) -> Result<String, String> {
    let (products, customers, suppliers, components, configurations) =
        with_database(state.clone(), |database| {
            Ok((
                database.list_products()?,
                database.list_customers()?,
                database.list_suppliers()?,
                database.list_config_components()?,
                database.list_configurable_products()?,
            ))
        })?;
    std::fs::create_dir_all(&state.export_dir)
        .map_err(|error| format!("无法创建导出目录：{error}"))?;
    let filename = if template_only {
        "TradeDesk_主数据导入模板.xlsx"
    } else {
        "TradeDesk_主数据.xlsx"
    };
    let path = state.export_dir.join(filename);
    spreadsheet::export_master_workbook(
        &path,
        template_only,
        &products,
        &customers,
        &suppliers,
        &components,
        &configurations,
    )?;
    Ok(path.to_string_lossy().into_owned())
}

#[tauri::command]
fn import_master_data(
    bytes: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<spreadsheet::MasterImportResult, String> {
    let mut data = spreadsheet::parse_master_workbook(&bytes)?;
    let existing_components =
        with_database(state.clone(), EncryptedDatabase::list_config_components)?;
    with_database(state.clone(), |database| {
        for item in &mut data.products {
            item.id = Some(
                database
                    .master_record_id("product", &item.sku)?
                    .unwrap_or_else(|| Uuid::new_v4().to_string()),
            );
        }
        for item in &mut data.customers {
            item.id = Some(
                database
                    .master_record_id("customer", &item.code)?
                    .unwrap_or_else(|| Uuid::new_v4().to_string()),
            );
        }
        for item in &mut data.suppliers {
            item.id = Some(
                database
                    .master_record_id("supplier", &item.code)?
                    .unwrap_or_else(|| Uuid::new_v4().to_string()),
            );
        }
        for item in &mut data.components {
            item.id = Some(
                database
                    .master_record_id("config_component", &item.code)?
                    .unwrap_or_else(|| Uuid::new_v4().to_string()),
            );
        }
        Ok(())
    })?;
    let available_component_codes = existing_components
        .iter()
        .map(|item| item.code.to_lowercase())
        .chain(data.components.iter().map(|item| item.code.to_lowercase()))
        .collect::<std::collections::HashSet<_>>();
    for configuration in &data.configurations {
        for line in &configuration.lines {
            if !available_component_codes.contains(&line.component_code.to_lowercase()) {
                return Err(format!(
                    "配置“{}”引用了不存在的组件编号“{}”",
                    configuration.code, line.component_code
                ));
            }
        }
    }
    let result = spreadsheet::MasterImportResult {
        products: data.products.len(),
        customers: data.customers.len(),
        suppliers: data.suppliers.len(),
        components: data.components.len(),
        configurations: data.configurations.len(),
    };
    with_database(state.clone(), |database| {
        for item in data.products {
            database.save_product(item)?;
        }
        for item in data.customers {
            database.save_customer(item)?;
        }
        for item in data.suppliers {
            database.save_supplier(item)?;
        }
        for item in data.components {
            database.save_config_component(item)?;
        }
        Ok(())
    })?;
    let components = with_database(state.clone(), EncryptedDatabase::list_config_components)?;
    for configuration in data.configurations {
        let id = with_database(state.clone(), |database| {
            database.master_record_id("configurable_product", &configuration.code)
        })?;
        let input = spreadsheet::build_configuration_input(configuration, id, &components)?;
        with_database(state.clone(), |database| {
            database.save_configurable_product(input.clone())
        })?;
    }
    Ok(result)
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
fn list_partners(state: State<'_, AppState>) -> Result<Vec<Partner>, String> {
    with_database(state, EncryptedDatabase::list_partners)
}

#[tauri::command]
fn save_partner(input: PartnerInput, state: State<'_, AppState>) -> Result<Partner, String> {
    with_database(state, |database| database.save_partner(input))
}

#[tauri::command]
fn archive_partner(id: String, state: State<'_, AppState>) -> Result<(), String> {
    with_database(state, |database| database.archive_partner(&id))
}

#[tauri::command]
fn list_shipment_batches(state: State<'_, AppState>) -> Result<Vec<ShipmentBatch>, String> {
    with_database(state, EncryptedDatabase::list_shipment_batches)
}

#[tauri::command]
fn save_shipment_batch(
    input: ShipmentBatchInput,
    state: State<'_, AppState>,
) -> Result<ShipmentBatch, String> {
    with_database(state, |database| database.save_shipment_batch(input))
}

#[tauri::command]
fn list_payment_plans(state: State<'_, AppState>) -> Result<Vec<PaymentPlan>, String> {
    with_database(state, EncryptedDatabase::list_payment_plans)
}

#[tauri::command]
fn save_payment_plan(
    input: PaymentPlanInput,
    state: State<'_, AppState>,
) -> Result<PaymentPlan, String> {
    with_database(state, |database| database.save_payment_plan(input))
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

fn export_pdf(
    id: &str,
    company_id: &str,
    signing_asset_id: &str,
    state: &State<'_, AppState>,
) -> Result<DocumentExportResult, String> {
    let (document, company_profile) = with_database(state.clone(), |database| {
        Ok((
            database.get_document(id)?,
            database.resolve_company_profile(company_id, signing_asset_id)?,
        ))
    })?;
    let typst_path = state
        .typst_path
        .as_ref()
        .ok_or("未找到 Typst PDF 渲染器，请重新运行开发环境安装脚本。")?;
    let mut result = document::export_pdf(
        &document,
        &company_profile,
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
    company_id: String,
    signing_asset_id: String,
    state: State<'_, AppState>,
) -> Result<DocumentExportResult, String> {
    export_pdf(&id, &company_id, &signing_asset_id, &state)
}

#[tauri::command]
fn export_document_csv(id: String, state: State<'_, AppState>) -> Result<String, String> {
    let document = with_database(state.clone(), |database| database.get_document(&id))?;
    document::export_csv(&document, &state.export_dir)
        .map(|path| path.to_string_lossy().into_owned())
}

#[tauri::command]
fn print_document(
    id: String,
    company_id: String,
    signing_asset_id: String,
    state: State<'_, AppState>,
) -> Result<DocumentExportResult, String> {
    let result = export_pdf(&id, &company_id, &signing_asset_id, &state)?;
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
            let recovery_path = app_data_dir.join("workspace.recovery.tdesk");
            let export_dir = app
                .path()
                .document_dir()
                .unwrap_or_else(|_| app_data_dir.clone())
                .join("TradeDesk Exports");
            let render_cache_dir = app.path().app_cache_dir()?.join("document-render");
            let backup_cache_dir = app.path().app_cache_dir()?.join("backup");
            let backup_dir = app
                .path()
                .document_dir()
                .unwrap_or_else(|_| app_data_dir.clone())
                .join("TradeDesk Backups");
            let attachment_export_dir = app
                .path()
                .document_dir()
                .unwrap_or_else(|_| app_data_dir.clone())
                .join("TradeDesk Attachments");
            let executable_dir = std::env::current_exe()?
                .parent()
                .map(PathBuf::from)
                .unwrap_or_else(|| app_data_dir.clone());
            app.manage(AppState {
                database_path,
                recovery_path,
                backup_dir,
                backup_cache_dir,
                attachment_export_dir,
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
            unlock_workspace_with_recovery,
            lock_workspace,
            workspace_summary,
            rotate_recovery_key,
            create_workspace_backup,
            restore_workspace_backup,
            workspace_restore_pending,
            rollback_workspace_restore,
            list_attachments,
            list_entity_attachments,
            save_attachment,
            export_attachment,
            delete_attachment,
            save_document_draft,
            load_document_draft,
            delete_document_draft,
            get_company_registry,
            save_company_registry,
            list_products,
            save_product,
            list_config_components,
            save_config_component,
            list_component_options,
            save_component_option,
            save_component_option_translation,
            list_configurable_products,
            save_configurable_product,
            export_configuration_pdf,
            export_configuration_csv,
            print_configuration,
            list_customers,
            save_customer,
            list_suppliers,
            save_supplier,
            archive_master,
            export_master_data,
            import_master_data,
            list_business_cases,
            save_business_case,
            archive_business_case,
            list_purchase_orders,
            create_purchase_order,
            update_purchase_order_status,
            update_production_milestone,
            list_partners,
            save_partner,
            archive_partner,
            list_shipment_batches,
            save_shipment_batch,
            list_payment_plans,
            save_payment_plan,
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
