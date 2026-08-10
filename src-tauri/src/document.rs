use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use sha2::{Digest, Sha256};

use crate::domain::{
    ConfigurableProduct, DocumentExportResult, DocumentType, DocumentValidationIssue,
    TradeDocument, ValidationSeverity,
};

const COMMERCIAL_INVOICE_TEMPLATE: &str =
    include_str!("../../templates/base/commercial-invoice.typ");
const COMMERCIAL_QUOTATION_TEMPLATE: &str =
    include_str!("../../templates/base/commercial-quotation.typ");
const PROFORMA_INVOICE_TEMPLATE: &str = include_str!("../../templates/base/proforma-invoice.typ");
const PACKING_LIST_TEMPLATE: &str = include_str!("../../templates/base/packing-list.typ");
const TRADE_CONTRACT_TEMPLATE: &str = include_str!("../../templates/base/trade-contract.typ");
const CONFIGURATION_SHEET_TEMPLATE: &str =
    include_str!("../../templates/base/configuration-sheet.typ");

pub fn validate(document: &TradeDocument) -> Vec<DocumentValidationIssue> {
    let mut issues = Vec::new();
    let mut error = |code: &str, message: &str| {
        issues.push(DocumentValidationIssue {
            severity: ValidationSeverity::Error,
            code: code.to_owned(),
            message: message.to_owned(),
        });
    };
    if document.number.trim().is_empty() {
        error("number_required", "单证编号不能为空");
    }
    if document.issue_date.trim().is_empty() {
        error("date_required", "签发日期不能为空");
    }
    if document.payload.seller.trim().is_empty() {
        error("seller_required", "卖方/出口商不能为空");
    }
    if document.payload.buyer.trim().is_empty() {
        error("buyer_required", "买方/收货人不能为空");
    }
    if document.payload.lines.is_empty() {
        error("lines_required", "至少需要一个产品明细");
    }
    if document.document_type == DocumentType::CommercialQuotation
        && document.payload.valid_until.trim().is_empty()
    {
        error("valid_until_required", "报价有效期不能为空");
    }
    if document.document_type == DocumentType::CommercialQuotation
        && !document.payload.valid_until.trim().is_empty()
        && document.payload.valid_until < document.issue_date
    {
        error("invalid_valid_until", "报价有效期不能早于签发日期");
    }
    for (index, line) in document.payload.lines.iter().enumerate() {
        if line.description.trim().is_empty() || line.quantity <= 0.0 || !line.quantity.is_finite()
        {
            error(
                "invalid_line",
                &format!("第 {} 行的品名或数量不完整", index + 1),
            );
        }
        if line.amount_minor < 0 || line.unit_price_minor < 0 {
            error(
                "invalid_amount",
                &format!("第 {} 行金额不能为负数", index + 1),
            );
        }
        if document.document_type == DocumentType::PackingList {
            if line.packages <= 0 {
                error(
                    "packages_required",
                    &format!("第 {} 行箱数必须大于 0", index + 1),
                );
            }
            if line.gross_weight_kg <= 0.0 || line.gross_weight_kg < line.net_weight_kg {
                error(
                    "invalid_weight",
                    &format!("第 {} 行毛重必须大于 0 且不能小于净重", index + 1),
                );
            }
        }
    }
    let subtotal = document
        .payload
        .lines
        .iter()
        .map(|line| line.amount_minor)
        .sum::<i64>();
    if document.payload.discount_minor < 0 || document.payload.discount_minor > subtotal {
        error("invalid_discount", "折扣不能为负数或超过产品小计");
    }
    if document.document_type != DocumentType::PackingList {
        for (index, line) in document.payload.lines.iter().enumerate() {
            if line.hs_code.trim().is_empty() {
                issues.push(DocumentValidationIssue {
                    severity: ValidationSeverity::Warning,
                    code: "hs_code_missing".to_owned(),
                    message: format!("第 {} 行缺少 HS 编码", index + 1),
                });
            }
        }
    }
    if document.document_type == DocumentType::ProformaInvoice
        && document.payload.bank_details.trim().is_empty()
    {
        issues.push(DocumentValidationIssue {
            severity: ValidationSeverity::Warning,
            code: "bank_details_missing".to_owned(),
            message: "形式发票尚未填写收款银行资料".to_owned(),
        });
    }
    issues
}

pub fn has_blocking_errors(document: &TradeDocument) -> bool {
    validate(document)
        .iter()
        .any(|issue| issue.severity == ValidationSeverity::Error)
}

pub fn export_pdf(
    document: &TradeDocument,
    typst_path: &Path,
    work_dir: &Path,
    output_dir: &Path,
) -> Result<DocumentExportResult, String> {
    fs::create_dir_all(work_dir).map_err(|error| format!("无法创建单证临时目录：{error}"))?;
    fs::create_dir_all(output_dir).map_err(|error| format!("无法创建单证导出目录：{error}"))?;
    let data_path = work_dir.join("document.json");
    let template_path = work_dir.join("document.typ");
    let output_path = output_dir.join(format!("{}.pdf", export_stem(document)));
    let payload = serde_json::to_vec_pretty(document)
        .map_err(|error| format!("无法生成单证快照：{error}"))?;
    fs::write(&data_path, payload).map_err(|error| format!("无法写入单证快照：{error}"))?;
    fs::write(&template_path, template(document))
        .map_err(|error| format!("无法写入单证模板：{error}"))?;
    let output = Command::new(typst_path)
        .arg("compile")
        .arg("--root")
        .arg(work_dir)
        .arg("--pdf-standard")
        .arg("1.7")
        .arg(&template_path)
        .arg(&output_path)
        .output()
        .map_err(|error| format!("无法启动 Typst PDF 渲染器：{error}"))?;
    if !output.status.success() {
        return Err(format!(
            "PDF 生成失败：{}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let bytes = fs::read(&output_path).map_err(|error| format!("无法读取生成的 PDF：{error}"))?;
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    Ok(DocumentExportResult {
        path: output_path.to_string_lossy().into_owned(),
        sha256,
        exported_at: String::new(),
    })
}

pub fn export_csv(document: &TradeDocument, output_dir: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(output_dir).map_err(|error| format!("无法创建单证导出目录：{error}"))?;
    let output_path = output_dir.join(format!("{}.csv", export_stem(document)));
    let mut rows = vec![
        "line,sku,description,model,hs_code,quantity,unit,unit_price,amount,packages,package_type,net_weight_kg,gross_weight_kg,cbm".to_owned(),
    ];
    for (index, line) in document.payload.lines.iter().enumerate() {
        rows.push(
            [
                (index + 1).to_string(),
                csv(&line.sku),
                csv(&line.description),
                csv(&line.model),
                csv(&line.hs_code),
                line.quantity.to_string(),
                csv(&line.unit),
                minor(line.unit_price_minor),
                minor(line.amount_minor),
                line.packages.to_string(),
                csv(&line.package_type),
                line.net_weight_kg.to_string(),
                line.gross_weight_kg.to_string(),
                line.cbm.to_string(),
            ]
            .join(","),
        );
    }
    fs::write(&output_path, format!("\u{feff}{}\r\n", rows.join("\r\n")))
        .map_err(|error| format!("无法写入 CSV：{error}"))?;
    Ok(output_path)
}

pub fn export_configuration_pdf(
    configuration: &ConfigurableProduct,
    typst_path: &Path,
    work_dir: &Path,
    output_dir: &Path,
) -> Result<DocumentExportResult, String> {
    fs::create_dir_all(work_dir).map_err(|error| format!("无法创建配置单临时目录：{error}"))?;
    fs::create_dir_all(output_dir).map_err(|error| format!("无法创建配置单导出目录：{error}"))?;
    let data_path = work_dir.join("configuration.json");
    let template_path = work_dir.join("configuration.typ");
    let output_path = output_dir.join(format!("{}.pdf", configuration_stem(configuration)));
    let payload = serde_json::to_vec_pretty(configuration)
        .map_err(|error| format!("无法生成配置清单快照：{error}"))?;
    fs::write(&data_path, payload).map_err(|error| format!("无法写入配置清单快照：{error}"))?;
    fs::write(&template_path, CONFIGURATION_SHEET_TEMPLATE)
        .map_err(|error| format!("无法写入配置单模板：{error}"))?;
    let output = Command::new(typst_path)
        .arg("compile")
        .arg("--root")
        .arg(work_dir)
        .arg("--pdf-standard")
        .arg("1.7")
        .arg(&template_path)
        .arg(&output_path)
        .output()
        .map_err(|error| format!("无法启动 Typst PDF 渲染器：{error}"))?;
    if !output.status.success() {
        return Err(format!(
            "配置单 PDF 生成失败：{}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let bytes =
        fs::read(&output_path).map_err(|error| format!("无法读取生成的配置单 PDF：{error}"))?;
    Ok(DocumentExportResult {
        path: output_path.to_string_lossy().into_owned(),
        sha256: format!("{:x}", Sha256::digest(&bytes)),
        exported_at: String::new(),
    })
}

pub fn export_configuration_csv(
    configuration: &ConfigurableProduct,
    output_dir: &Path,
) -> Result<PathBuf, String> {
    fs::create_dir_all(output_dir).map_err(|error| format!("无法创建配置单导出目录：{error}"))?;
    let output_path = output_dir.join(format!("{}.csv", configuration_stem(configuration)));
    let mut rows = vec![
        [
            csv("配置编号"),
            csv(&configuration.code),
            csv("产品名称"),
            csv(&configuration.name),
            csv("型号"),
            csv(&configuration.model),
            csv("币种"),
            csv(&configuration.currency),
        ]
        .join(","),
        [csv("配置说明"), csv(&configuration.notes)].join(","),
        String::new(),
        "序号,组件类别,品名,型号/规格/材质,数量,单位,单价,总价,币种,品牌,备注".to_owned(),
    ];
    for (index, line) in configuration.lines.iter().enumerate() {
        rows.push(
            [
                (index + 1).to_string(),
                csv(&line.category),
                csv(&line.name),
                csv(&line.specification),
                line.quantity.to_string(),
                csv(&line.unit),
                minor(line.unit_price_minor),
                minor(line.amount_minor),
                csv(&configuration.currency),
                csv(&line.brand),
                csv(&line.notes),
            ]
            .join(","),
        );
    }
    rows.push(
        [
            String::new(),
            String::new(),
            csv("配置总价"),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            minor(configuration.total_amount_minor),
            csv(&configuration.currency),
            String::new(),
            String::new(),
        ]
        .join(","),
    );
    fs::write(&output_path, format!("\u{feff}{}\r\n", rows.join("\r\n")))
        .map_err(|error| format!("无法写入配置单 CSV：{error}"))?;
    Ok(output_path)
}

pub fn find_typst(executable_dir: &Path) -> Option<PathBuf> {
    let file_name = if cfg!(target_os = "windows") {
        "typst.exe"
    } else {
        "typst"
    };
    let candidates = [
        executable_dir.join(file_name),
        executable_dir.join("resources").join(file_name),
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("tools")
            .join("typst")
            .join("typst-x86_64-pc-windows-msvc")
            .join("typst.exe"),
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("tools")
            .join("typst")
            .join("typst-aarch64-apple-darwin")
            .join("typst"),
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("tools")
            .join("typst")
            .join("typst-x86_64-apple-darwin")
            .join("typst"),
    ];
    candidates.into_iter().find(|path| path.is_file())
}

pub fn open_file(path: &Path) -> Result<(), String> {
    let mut command = if cfg!(target_os = "windows") {
        let mut command = Command::new("explorer.exe");
        command.arg(path);
        command
    } else if cfg!(target_os = "macos") {
        let mut command = Command::new("open");
        command.arg(path);
        command
    } else {
        let mut command = Command::new("xdg-open");
        command.arg(path);
        command
    };
    command
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("无法打开导出文件：{error}"))
}

fn template(document: &TradeDocument) -> &'static str {
    match document.document_type {
        DocumentType::CommercialQuotation => COMMERCIAL_QUOTATION_TEMPLATE,
        DocumentType::ProformaInvoice => PROFORMA_INVOICE_TEMPLATE,
        DocumentType::CommercialInvoice => COMMERCIAL_INVOICE_TEMPLATE,
        DocumentType::PackingList => PACKING_LIST_TEMPLATE,
        DocumentType::TradeContract => TRADE_CONTRACT_TEMPLATE,
    }
}

fn export_stem(document: &TradeDocument) -> String {
    let kind = match document.document_type {
        DocumentType::CommercialQuotation => "CommercialQuotation",
        DocumentType::ProformaInvoice => "ProformaInvoice",
        DocumentType::CommercialInvoice => "CommercialInvoice",
        DocumentType::PackingList => "PackingList",
        DocumentType::TradeContract => "TradeContract",
    };
    let raw = format!(
        "{}_{}_{}_V{}_{}",
        document.customer_name, kind, document.number, document.version, document.issue_date
    );
    raw.chars()
        .map(|character| match character {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            _ => character,
        })
        .collect::<String>()
        .trim_matches([' ', '.'])
        .to_owned()
}

fn configuration_stem(configuration: &ConfigurableProduct) -> String {
    sanitize_stem(&format!(
        "ConfigurationSheet_{}_{}",
        configuration.code, configuration.name
    ))
}

fn sanitize_stem(raw: &str) -> String {
    raw.chars()
        .map(|character| match character {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            _ => character,
        })
        .collect::<String>()
        .trim_matches([' ', '.'])
        .to_owned()
}

fn csv(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn minor(value: i64) -> String {
    format!("{}.{:02}", value / 100, value.unsigned_abs() % 100)
}

#[cfg(test)]
mod tests {
    use super::csv;

    #[test]
    fn csv_escapes_quotes() {
        assert_eq!(csv("A, \"B\""), "\"A, \"\"B\"\"\"");
    }
}
