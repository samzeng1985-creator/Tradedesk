mod document;
mod domain;
mod storage;

use domain::WorkspaceSummary;

#[tauri::command]
fn implementation_status() -> WorkspaceSummary {
    WorkspaceSummary {
        company_name: "演示工作区".to_owned(),
        encrypted: true,
        products: 2,
        customers: 2,
        suppliers: 2,
        active_cases: 1,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![implementation_status])
        .run(tauri::generate_context!())
        .expect("failed to run TradeDesk Local");
}
