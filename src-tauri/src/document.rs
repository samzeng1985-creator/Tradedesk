use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::domain::{
    CompanyProfile, CompanyRegistry, ConfigurableProduct, DocumentExportResult, DocumentType,
    DocumentValidationIssue, PurchaseOrder, PurchaseStatus, TradeDocument, ValidationSeverity,
};

const COMMERCIAL_INVOICE_TEMPLATE: &str =
    include_str!("../../templates/base/commercial-invoice.typ");
const COMMERCIAL_QUOTATION_TEMPLATE: &str =
    include_str!("../../templates/base/commercial-quotation.typ");
const PROFORMA_INVOICE_TEMPLATE: &str = include_str!("../../templates/base/proforma-invoice.typ");
const PACKING_LIST_TEMPLATE: &str = include_str!("../../templates/base/packing-list.typ");
const TRADE_CONTRACT_TEMPLATE: &str = include_str!("../../templates/base/trade-contract.typ");
const SHIPPING_MARKS_TEMPLATE: &str = include_str!("../../templates/base/shipping-marks.typ");
const SHIPPER_INSTRUCTION_TEMPLATE: &str =
    include_str!("../../templates/base/shipper-instruction.typ");
const CUSTOMS_DECLARATION_TEMPLATE: &str =
    include_str!("../../templates/base/customs-declaration.typ");
const BILL_OF_LADING_TEMPLATE: &str = include_str!("../../templates/base/bill-of-lading.typ");
const INSURANCE_POLICY_TEMPLATE: &str = include_str!("../../templates/base/insurance-policy.typ");
const CERTIFICATE_OF_ORIGIN_TEMPLATE: &str =
    include_str!("../../templates/base/certificate-of-origin.typ");
const INSPECTION_CERTIFICATE_TEMPLATE: &str =
    include_str!("../../templates/base/inspection-certificate.typ");
const FUMIGATION_CERTIFICATE_TEMPLATE: &str =
    include_str!("../../templates/base/fumigation-certificate.typ");
const BENEFICIARY_CERTIFICATE_TEMPLATE: &str =
    include_str!("../../templates/base/beneficiary-certificate.typ");
const CONFIGURATION_SHEET_TEMPLATE: &str =
    include_str!("../../templates/base/configuration-sheet.typ");
const PURCHASE_ORDER_TEMPLATE: &str = include_str!("../../templates/base/purchase-order.typ");
const TEMPLATE_HELPERS: &str = include_str!("../../templates/base/helpers.typ");

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfigurationLabels {
    title: &'static str,
    code: &'static str,
    product_name: &'static str,
    currency: &'static str,
    exchange_rate: &'static str,
    rate_date: &'static str,
    model: &'static str,
    component_count: &'static str,
    configuration_total: &'static str,
    number: &'static str,
    item_name: &'static str,
    specification: &'static str,
    quantity: &'static str,
    unit: &'static str,
    unit_price: &'static str,
    amount: &'static str,
    brand: &'static str,
    notes: &'static str,
    prepared_by: &'static str,
    snapshot_notice: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BrandingPayload {
    company_name: String,
    logo_path: String,
    signature_path: String,
    signing_asset_kind: String,
}

const MAX_BRAND_ASSET_BYTES: usize = 3 * 1024 * 1024;

fn decode_brand_asset(value: &str) -> Result<Option<(&'static str, Vec<u8>)>, String> {
    if value.trim().is_empty() {
        return Ok(None);
    }
    let (header, encoded) = value.split_once(',').ok_or("企业图片数据格式不正确")?;
    let extension = match header {
        "data:image/png;base64" => "png",
        "data:image/jpeg;base64" => "jpg",
        "data:image/webp;base64" => "webp",
        _ => return Err("Logo 和电子签名仅支持 PNG、JPG 或 WebP 图片".to_owned()),
    };
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|_| "企业图片数据无法读取".to_owned())?;
    if bytes.len() > MAX_BRAND_ASSET_BYTES {
        return Err("每张企业图片不能超过 3 MB".to_owned());
    }
    Ok(Some((extension, bytes)))
}

pub fn validate_company_profile(profile: &CompanyProfile) -> Result<(), String> {
    if profile.company_name.trim().is_empty() {
        return Err("公司名称不能为空".to_owned());
    }
    decode_brand_asset(&profile.logo_data_url)?;
    decode_brand_asset(&profile.signature_data_url)?;
    Ok(())
}

pub fn validate_company_registry(registry: &CompanyRegistry) -> Result<(), String> {
    use std::collections::HashSet;

    if registry.companies.is_empty() {
        return Err("至少需要保留一家出口公司".to_owned());
    }
    let mut company_ids = HashSet::new();
    for company in &registry.companies {
        if company.id.trim().is_empty() || company.company_name.trim().is_empty() {
            return Err("公司名称不能为空".to_owned());
        }
        if !company_ids.insert(company.id.trim()) {
            return Err("公司编号不能重复".to_owned());
        }
        decode_brand_asset(&company.logo_data_url)?;
        let mut asset_ids = HashSet::new();
        for asset in &company.signing_assets {
            if asset.id.trim().is_empty() || asset.name.trim().is_empty() {
                return Err("电子签名或电子章的名称不能为空".to_owned());
            }
            if !matches!(asset.kind.as_str(), "signature" | "stamp") {
                return Err("签章类型仅支持电子签名或电子章".to_owned());
            }
            if !asset_ids.insert(asset.id.trim()) {
                return Err("同一公司内的签章编号不能重复".to_owned());
            }
            if asset.data_url.trim().is_empty() {
                return Err(format!("请为“{}”上传图片", asset.name));
            }
            decode_brand_asset(&asset.data_url)?;
        }
    }
    if !company_ids.contains(registry.default_company_id.trim()) {
        return Err("默认公司不存在".to_owned());
    }
    Ok(())
}

fn prepare_branding(profile: &CompanyProfile, work_dir: &Path) -> Result<BrandingPayload, String> {
    validate_company_profile(profile)?;
    let write_asset = |value: &str, stem: &str| -> Result<String, String> {
        let Some((extension, bytes)) = decode_brand_asset(value)? else {
            return Ok(String::new());
        };
        let filename = format!("{stem}.{extension}");
        fs::write(work_dir.join(&filename), bytes)
            .map_err(|error| format!("无法准备企业图片：{error}"))?;
        Ok(filename)
    };
    Ok(BrandingPayload {
        company_name: profile.company_name.trim().to_owned(),
        logo_path: write_asset(&profile.logo_data_url, "company-logo")?,
        signature_path: write_asset(&profile.signature_data_url, "company-signature")?,
        signing_asset_kind: profile.signing_asset_kind.clone(),
    })
}

fn cleanup_branding_assets(work_dir: &Path) {
    let Ok(entries) = fs::read_dir(work_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("company-logo.") || name.starts_with("company-signature.") {
            let _ = fs::remove_file(entry.path());
        }
    }
}

fn branded_json<T: Serialize>(value: &T, branding: BrandingPayload) -> Result<Vec<u8>, String> {
    let mut value =
        serde_json::to_value(value).map_err(|error| format!("无法生成导出数据：{error}"))?;
    value.as_object_mut().ok_or("无法生成导出数据")?.insert(
        "branding".to_owned(),
        serde_json::to_value(branding).map_err(|error| format!("无法生成企业资料：{error}"))?,
    );
    serde_json::to_vec_pretty(&value).map_err(|error| format!("无法生成导出数据：{error}"))
}

pub fn validate(document: &TradeDocument) -> Vec<DocumentValidationIssue> {
    let mut issues = Vec::new();
    let mut hs_warnings = Vec::new();
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
    let packing_fields = matches!(
        document.document_type,
        DocumentType::PackingList
            | DocumentType::ShippingMarks
            | DocumentType::ShipperInstruction
            | DocumentType::CustomsDeclaration
            | DocumentType::BillOfLading
            | DocumentType::InspectionCertificate
            | DocumentType::FumigationCertificate
    );
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
        if packing_fields {
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
    if document.document_type == DocumentType::ShippingMarks
        && document.payload.shipping_marks.trim().is_empty()
    {
        error("shipping_marks_required", "唛头内容不能为空");
    }
    if matches!(
        document.document_type,
        DocumentType::ShipperInstruction
            | DocumentType::CustomsDeclaration
            | DocumentType::BillOfLading
            | DocumentType::InsurancePolicy
            | DocumentType::CertificateOfOrigin
            | DocumentType::InspectionCertificate
            | DocumentType::FumigationCertificate
            | DocumentType::BeneficiaryCertificate
    ) && document.payload.transport_mode.trim().is_empty()
    {
        error("transport_mode_required", "运输方式不能为空");
    }
    if matches!(
        document.document_type,
        DocumentType::ShipperInstruction
            | DocumentType::BillOfLading
            | DocumentType::InsurancePolicy
            | DocumentType::CertificateOfOrigin
            | DocumentType::InspectionCertificate
            | DocumentType::FumigationCertificate
            | DocumentType::BeneficiaryCertificate
    ) && (document.payload.port_of_loading.trim().is_empty()
        || document.payload.port_of_discharge.trim().is_empty())
    {
        error("ports_required", "运输或保险资料必须填写装运港和目的港");
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
    if document.document_type == DocumentType::BillOfLading {
        if document.payload.carrier.trim().is_empty() {
            error("carrier_required", "提单补料必须填写承运人或船公司");
        }
        if document.payload.bill_of_lading_type.trim().is_empty() {
            error("bill_type_required", "提单补料必须选择提单类型");
        }
    }
    if document.document_type == DocumentType::InsurancePolicy {
        if document.payload.insurance_company.trim().is_empty() {
            error("insurance_company_required", "保险申请必须填写保险公司");
        }
        if document.payload.insurance_coverage.trim().is_empty() {
            error("insurance_coverage_required", "保险申请必须填写承保险别");
        }
        if document.payload.insured_value_minor <= 0 {
            error("insured_value_required", "保险金额必须大于 0");
        }
        let incoterm = document.payload.incoterm.trim().to_ascii_uppercase();
        if (incoterm.starts_with("CIF") || incoterm.starts_with("CIP"))
            && document.payload.insured_value_minor < subtotal - document.payload.discount_minor
        {
            error("insured_value_too_low", "CIF/CIP 保险金额不能低于货值");
        }
    }
    if document.document_type == DocumentType::CertificateOfOrigin {
        if document.payload.origin_country.trim().is_empty() {
            error("origin_country_required", "原产地证明必须填写原产国");
        }
        if document.payload.certificate_type.trim().is_empty() {
            error("certificate_type_required", "原产地证明必须选择证书类型");
        }
        if document.payload.certification_authority.trim().is_empty() {
            error(
                "certification_authority_required",
                "原产地证明必须填写签证机构",
            );
        }
    }
    if document.document_type == DocumentType::InspectionCertificate {
        for (code, value, message) in [
            (
                "inspection_authority_required",
                document.payload.certification_authority.as_str(),
                "检验证书必须填写检验机构",
            ),
            (
                "manufacturer_required",
                document.payload.manufacturer.as_str(),
                "检验证书必须填写制造商",
            ),
            (
                "batch_number_required",
                document.payload.batch_number.as_str(),
                "检验证书必须填写批次号",
            ),
            (
                "inspection_standard_required",
                document.payload.inspection_standard.as_str(),
                "检验证书必须填写检验标准",
            ),
            (
                "inspection_date_required",
                document.payload.inspection_date.as_str(),
                "检验证书必须填写检验日期",
            ),
            (
                "inspection_result_required",
                document.payload.inspection_result.as_str(),
                "检验证书必须填写检验结果",
            ),
        ] {
            if value.trim().is_empty() {
                error(code, message);
            }
        }
    }
    if document.document_type == DocumentType::FumigationCertificate {
        for (code, value, message) in [
            (
                "fumigation_authority_required",
                document.payload.certification_authority.as_str(),
                "熏蒸证书申请必须填写服务或签证机构",
            ),
            (
                "fumigation_agent_required",
                document.payload.fumigation_agent.as_str(),
                "熏蒸证书申请必须填写熏蒸剂",
            ),
            (
                "fumigation_method_required",
                document.payload.fumigation_method.as_str(),
                "熏蒸证书申请必须填写处理方法",
            ),
            (
                "fumigation_date_required",
                document.payload.fumigation_date.as_str(),
                "熏蒸证书申请必须填写处理日期",
            ),
            (
                "fumigation_place_required",
                document.payload.fumigation_place.as_str(),
                "熏蒸证书申请必须填写处理地点",
            ),
            (
                "fumigation_operator_required",
                document.payload.fumigation_operator.as_str(),
                "熏蒸证书申请必须填写操作人员",
            ),
            (
                "fumigation_license_required",
                document.payload.fumigation_license_number.as_str(),
                "熏蒸证书申请必须填写机构或人员许可证号",
            ),
        ] {
            if value.trim().is_empty() {
                error(code, message);
            }
        }
        if document.payload.fumigation_duration_hours <= 0.0
            || !document.payload.fumigation_duration_hours.is_finite()
        {
            error(
                "fumigation_duration_required",
                "熏蒸持续时间必须大于 0 小时",
            );
        }
    }
    if document.document_type == DocumentType::BeneficiaryCertificate {
        for (code, value, message) in [
            (
                "letter_of_credit_number_required",
                document.payload.letter_of_credit_number.as_str(),
                "受益人证明必须填写信用证号码",
            ),
            (
                "issuing_bank_required",
                document.payload.issuing_bank.as_str(),
                "受益人证明必须填写开证行",
            ),
            (
                "letter_of_credit_expiry_required",
                document.payload.letter_of_credit_expiry_date.as_str(),
                "受益人证明必须填写信用证有效期",
            ),
            (
                "presentation_deadline_required",
                document.payload.presentation_deadline.as_str(),
                "受益人证明必须填写交单截止日",
            ),
            (
                "beneficiary_certificate_type_required",
                document.payload.beneficiary_certificate_type.as_str(),
                "受益人证明必须填写证明类型",
            ),
            (
                "beneficiary_statement_required",
                document.payload.beneficiary_statement.as_str(),
                "受益人证明必须填写证明声明",
            ),
        ] {
            if value.trim().is_empty() {
                error(code, message);
            }
        }
        if !document
            .payload
            .letter_of_credit_issue_date
            .trim()
            .is_empty()
            && !document
                .payload
                .letter_of_credit_expiry_date
                .trim()
                .is_empty()
            && document.payload.letter_of_credit_expiry_date
                < document.payload.letter_of_credit_issue_date
        {
            error(
                "invalid_letter_of_credit_dates",
                "信用证有效期不能早于开证日期",
            );
        }
        if !document.payload.presentation_deadline.trim().is_empty()
            && !document
                .payload
                .letter_of_credit_expiry_date
                .trim()
                .is_empty()
            && document.payload.presentation_deadline
                > document.payload.letter_of_credit_expiry_date
        {
            error(
                "invalid_presentation_deadline",
                "交单截止日不能晚于信用证有效期",
            );
        }
        for (code, value, message) in [
            (
                "letter_of_credit_issue_date_missing",
                document.payload.letter_of_credit_issue_date.as_str(),
                "尚未填写信用证开证日期，请与银行正本核对",
            ),
            (
                "letter_of_credit_terms_missing",
                document.payload.letter_of_credit_terms.as_str(),
                "尚未记录信用证条款清单，请与银行正本核对",
            ),
            (
                "required_documents_missing",
                document.payload.required_documents.as_str(),
                "尚未记录所需交单文件清单",
            ),
        ] {
            if value.trim().is_empty() {
                hs_warnings.push(DocumentValidationIssue {
                    severity: ValidationSeverity::Warning,
                    code: code.to_owned(),
                    message: message.to_owned(),
                });
            }
        }
    }
    if !matches!(
        document.document_type,
        DocumentType::PackingList | DocumentType::ShippingMarks
    ) {
        for (index, line) in document.payload.lines.iter().enumerate() {
            if line.hs_code.trim().is_empty() {
                if matches!(
                    document.document_type,
                    DocumentType::CustomsDeclaration
                        | DocumentType::InsurancePolicy
                        | DocumentType::CertificateOfOrigin
                ) {
                    error(
                        "hs_code_missing",
                        &format!("报关、保险或原产地资料第 {} 行必须填写 HS 编码", index + 1),
                    );
                } else {
                    hs_warnings.push(DocumentValidationIssue {
                        severity: ValidationSeverity::Warning,
                        code: "hs_code_missing".to_owned(),
                        message: format!("第 {} 行缺少 HS 编码", index + 1),
                    });
                }
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
    issues.extend(hs_warnings);
    issues
}

pub fn has_blocking_errors(document: &TradeDocument) -> bool {
    validate(document)
        .iter()
        .chain(document.validation_issues.iter())
        .any(|issue| issue.severity == ValidationSeverity::Error)
}

pub fn cross_validate(
    document: &TradeDocument,
    peers: &[TradeDocument],
) -> Vec<DocumentValidationIssue> {
    let peer = |document_type: DocumentType| {
        peers.iter().find(|item| {
            item.id != document.id
                && item.business_case_id == document.business_case_id
                && item.document_type == document_type
                && item.status != crate::domain::DocumentStatus::Voided
        })
    };
    let mut pairs = Vec::new();
    match document.document_type {
        DocumentType::CommercialInvoice => {
            if let Some(item) = peer(DocumentType::PackingList) {
                pairs.push((item, false, false, false, false));
            }
            if let Some(item) = peer(DocumentType::CustomsDeclaration) {
                pairs.push((item, true, true, true, false));
            }
            if let Some(item) = peer(DocumentType::InsurancePolicy) {
                pairs.push((item, false, false, true, false));
            }
            if let Some(item) = peer(DocumentType::CertificateOfOrigin) {
                pairs.push((item, false, false, true, false));
            }
            if let Some(item) = peer(DocumentType::BeneficiaryCertificate) {
                pairs.push((item, false, true, true, false));
            }
        }
        DocumentType::PackingList => {
            if let Some(item) = peer(DocumentType::CommercialInvoice) {
                pairs.push((item, false, false, false, false));
            }
            if let Some(item) = peer(DocumentType::ShipperInstruction) {
                pairs.push((item, true, false, false, true));
            }
            if let Some(item) = peer(DocumentType::ShippingMarks) {
                pairs.push((item, false, false, false, false));
            }
            if let Some(item) = peer(DocumentType::CustomsDeclaration) {
                pairs.push((item, true, false, false, false));
            }
            if let Some(item) = peer(DocumentType::BillOfLading) {
                pairs.push((item, true, false, false, true));
            }
            if let Some(item) = peer(DocumentType::InspectionCertificate) {
                pairs.push((item, true, false, false, false));
            }
            if let Some(item) = peer(DocumentType::FumigationCertificate) {
                pairs.push((item, true, false, false, true));
            }
        }
        DocumentType::CustomsDeclaration => {
            if let Some(item) = peer(DocumentType::CommercialInvoice) {
                pairs.push((item, false, true, true, false));
            }
            if let Some(item) = peer(DocumentType::PackingList) {
                pairs.push((item, true, false, false, false));
            }
            if let Some(item) = peer(DocumentType::CertificateOfOrigin) {
                pairs.push((item, false, false, true, false));
            }
            if let Some(item) = peer(DocumentType::FumigationCertificate) {
                pairs.push((item, true, false, false, true));
            }
        }
        DocumentType::ShipperInstruction | DocumentType::ShippingMarks => {
            if let Some(item) = peer(DocumentType::PackingList) {
                pairs.push((
                    item,
                    document.document_type == DocumentType::ShipperInstruction,
                    false,
                    false,
                    document.document_type == DocumentType::ShipperInstruction,
                ));
            }
            if document.document_type == DocumentType::ShipperInstruction
                && let Some(item) = peer(DocumentType::BillOfLading)
            {
                pairs.push((item, true, false, false, true));
            }
            if document.document_type == DocumentType::ShipperInstruction
                && let Some(item) = peer(DocumentType::FumigationCertificate)
            {
                pairs.push((item, true, false, false, true));
            }
        }
        DocumentType::BillOfLading => {
            if let Some(item) = peer(DocumentType::PackingList) {
                pairs.push((item, true, false, false, true));
            }
            if let Some(item) = peer(DocumentType::ShipperInstruction) {
                pairs.push((item, true, false, false, true));
            }
            if let Some(item) = peer(DocumentType::CertificateOfOrigin) {
                pairs.push((item, false, false, false, true));
            }
            if let Some(item) = peer(DocumentType::InspectionCertificate) {
                pairs.push((item, true, false, false, true));
            }
            if let Some(item) = peer(DocumentType::FumigationCertificate) {
                pairs.push((item, true, false, false, true));
            }
            if let Some(item) = peer(DocumentType::BeneficiaryCertificate) {
                pairs.push((item, false, false, false, true));
            }
        }
        DocumentType::InsurancePolicy => {
            if let Some(item) = peer(DocumentType::CommercialInvoice) {
                pairs.push((item, false, false, true, false));
            }
            if let Some(item) = peer(DocumentType::BillOfLading) {
                pairs.push((item, false, false, false, true));
            }
        }
        DocumentType::CertificateOfOrigin => {
            if let Some(item) = peer(DocumentType::CommercialInvoice) {
                pairs.push((item, false, false, true, false));
            }
            if let Some(item) = peer(DocumentType::CustomsDeclaration) {
                pairs.push((item, false, false, true, false));
            }
            if let Some(item) = peer(DocumentType::BillOfLading) {
                pairs.push((item, false, false, false, true));
            }
        }
        DocumentType::InspectionCertificate => {
            if let Some(item) = peer(DocumentType::PackingList) {
                pairs.push((item, true, false, false, false));
            }
            if let Some(item) = peer(DocumentType::BillOfLading) {
                pairs.push((item, true, false, false, true));
            }
        }
        DocumentType::FumigationCertificate => {
            if let Some(item) = peer(DocumentType::PackingList) {
                pairs.push((item, true, false, false, true));
            }
            if let Some(item) = peer(DocumentType::BillOfLading) {
                pairs.push((item, true, false, false, true));
            }
        }
        DocumentType::BeneficiaryCertificate => {
            if let Some(item) = peer(DocumentType::CommercialInvoice) {
                pairs.push((item, false, true, true, false));
            }
            if let Some(item) = peer(DocumentType::BillOfLading) {
                pairs.push((item, false, false, false, true));
            }
        }
        _ => {}
    }
    let mut issues = Vec::new();
    for (other, compare_physical, compare_amount, compare_hs, compare_transport) in pairs {
        compare_document_pair(
            document,
            other,
            compare_physical,
            compare_amount,
            compare_hs,
            compare_transport,
            &mut issues,
        );
    }
    issues
}

fn compare_document_pair(
    document: &TradeDocument,
    other: &TradeDocument,
    compare_physical: bool,
    compare_amount: bool,
    compare_hs: bool,
    compare_transport: bool,
    issues: &mut Vec<DocumentValidationIssue>,
) {
    let quantities = |value: &TradeDocument| {
        let mut totals = std::collections::HashMap::<String, f64>::new();
        for line in &value.payload.lines {
            *totals.entry(line.product_id.clone()).or_default() += line.quantity;
        }
        totals
    };
    let current_quantities = quantities(document);
    let other_quantities = quantities(other);
    if current_quantities.len() != other_quantities.len()
        || current_quantities.iter().any(|(product_id, quantity)| {
            (other_quantities
                .get(product_id)
                .copied()
                .unwrap_or_default()
                - quantity)
                .abs()
                > 0.000_001
        })
    {
        issues.push(DocumentValidationIssue {
            severity: ValidationSeverity::Error,
            code: "cross_document_quantity_mismatch".to_owned(),
            message: format!("与单证 {} 的产品数量不一致，请核对后再签发", other.number),
        });
    }
    if compare_physical {
        let physical = |value: &TradeDocument| {
            value
                .payload
                .lines
                .iter()
                .fold((0_i64, 0.0, 0.0, 0.0), |sum, line| {
                    (
                        sum.0 + line.packages,
                        sum.1 + line.net_weight_kg,
                        sum.2 + line.gross_weight_kg,
                        sum.3 + line.cbm,
                    )
                })
        };
        let current = physical(document);
        let compared = physical(other);
        if (current.1 - compared.1).abs() > 0.001 || (current.2 - compared.2).abs() > 0.001 {
            issues.push(DocumentValidationIssue {
                severity: ValidationSeverity::Error,
                code: "cross_document_weight_mismatch".to_owned(),
                message: format!("与单证 {} 的净重或毛重不一致", other.number),
            });
        }
        if current.0 != compared.0 || (current.3 - compared.3).abs() > 0.001 {
            issues.push(DocumentValidationIssue {
                severity: ValidationSeverity::Error,
                code: "cross_document_package_mismatch".to_owned(),
                message: format!("与单证 {} 的总件数或 CBM 不一致", other.number),
            });
        }
    }
    if compare_amount {
        let amount = |value: &TradeDocument| {
            value
                .payload
                .lines
                .iter()
                .map(|line| line.amount_minor)
                .sum::<i64>()
                - value.payload.discount_minor
        };
        if document.currency != other.currency || amount(document) != amount(other) {
            issues.push(DocumentValidationIssue {
                severity: ValidationSeverity::Error,
                code: "cross_document_amount_mismatch".to_owned(),
                message: format!("与单证 {} 的币种或申报总额不一致", other.number),
            });
        }
    }
    if compare_hs {
        for line in &document.payload.lines {
            if let Some(other_line) = other
                .payload
                .lines
                .iter()
                .find(|item| item.product_id == line.product_id)
                && !line.hs_code.trim().is_empty()
                && !other_line.hs_code.trim().is_empty()
                && line.hs_code.trim() != other_line.hs_code.trim()
            {
                issues.push(DocumentValidationIssue {
                    severity: ValidationSeverity::Error,
                    code: "cross_document_hs_mismatch".to_owned(),
                    message: format!("产品 {} 与单证 {} 的 HS 编码不一致", line.sku, other.number),
                });
            }
        }
    }
    if compare_transport {
        let fields = [
            (
                "装运港",
                document.payload.port_of_loading.trim(),
                other.payload.port_of_loading.trim(),
            ),
            (
                "目的港",
                document.payload.port_of_discharge.trim(),
                other.payload.port_of_discharge.trim(),
            ),
            (
                "船名/航次",
                document.payload.vessel_voyage.trim(),
                other.payload.vessel_voyage.trim(),
            ),
            (
                "装运日期",
                document.payload.shipment_date.trim(),
                other.payload.shipment_date.trim(),
            ),
            (
                "唛头",
                document.payload.shipping_marks.trim(),
                other.payload.shipping_marks.trim(),
            ),
        ];
        let mismatches = fields
            .iter()
            .filter(|(_, current, compared)| {
                !current.is_empty() && !compared.is_empty() && current != compared
            })
            .map(|(label, _, _)| *label)
            .collect::<Vec<_>>();
        if !mismatches.is_empty() {
            issues.push(DocumentValidationIssue {
                severity: ValidationSeverity::Error,
                code: "cross_document_transport_mismatch".to_owned(),
                message: format!(
                    "与单证 {} 的运输字段不一致：{}",
                    other.number,
                    mismatches.join("、")
                ),
            });
        }
    }
    let origin_pair = (document.document_type == DocumentType::CertificateOfOrigin
        && matches!(
            other.document_type,
            DocumentType::CommercialInvoice | DocumentType::CustomsDeclaration
        ))
        || (other.document_type == DocumentType::CertificateOfOrigin
            && matches!(
                document.document_type,
                DocumentType::CommercialInvoice | DocumentType::CustomsDeclaration
            ));
    if origin_pair
        && !document.payload.origin_country.trim().is_empty()
        && !other.payload.origin_country.trim().is_empty()
        && !document
            .payload
            .origin_country
            .trim()
            .eq_ignore_ascii_case(other.payload.origin_country.trim())
    {
        issues.push(DocumentValidationIssue {
            severity: ValidationSeverity::Error,
            code: "cross_document_origin_mismatch".to_owned(),
            message: format!("与单证 {} 的原产国不一致", other.number),
        });
    }
}

pub fn export_pdf(
    document: &TradeDocument,
    company_profile: &CompanyProfile,
    typst_path: &Path,
    work_dir: &Path,
    output_dir: &Path,
) -> Result<DocumentExportResult, String> {
    fs::create_dir_all(work_dir).map_err(|error| format!("无法创建单证临时目录：{error}"))?;
    fs::create_dir_all(output_dir).map_err(|error| format!("无法创建单证导出目录：{error}"))?;
    let data_path = work_dir.join("document.json");
    let template_path = work_dir.join("document.typ");
    let output_path = output_dir.join(format!("{}.pdf", export_stem(document)));
    let payload = branded_json(document, prepare_branding(company_profile, work_dir)?)
        .map_err(|error| format!("无法生成单证快照：{error}"))?;
    fs::write(&data_path, payload).map_err(|error| format!("无法写入单证快照：{error}"))?;
    fs::write(work_dir.join("helpers.typ"), TEMPLATE_HELPERS)
        .map_err(|error| format!("无法写入单证排版助手：{error}"))?;
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
    cleanup_branding_assets(work_dir);
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
        "document_type,document_number,business_case,issue_date,currency,transport_mode,vessel_voyage,booking_reference,freight_terms,bill_of_lading_type,bill_of_lading_number,carrier,notify_party,place_of_receipt,place_of_delivery,container_numbers,seal_numbers,shipping_marks,insurance_company,policy_number,insured_value,insurance_markup_percent,premium_rate_percent,premium,insurance_coverage,claims_payable_at,customs_supervision_code,customs_declaration_elements,certificate_number,certificate_type,certification_authority,manufacturer,manufacturer_address,batch_number,inspection_standard,inspection_date,inspection_place,inspection_result,fumigation_agent,fumigation_method,fumigation_temperature_celsius,fumigation_duration_hours,fumigation_date,fumigation_place,fumigation_operator,fumigation_license_number,letter_of_credit_number,issuing_bank,letter_of_credit_issue_date,letter_of_credit_expiry_date,presentation_deadline,beneficiary_certificate_type,beneficiary_statement,letter_of_credit_terms,required_documents".to_owned(),
        [
            csv(document.document_type.as_str()),
            csv(&document.number),
            csv(&document.business_case_number),
            csv(&document.issue_date),
            csv(&document.currency),
            csv(&document.payload.transport_mode),
            csv(&document.payload.vessel_voyage),
            csv(&document.payload.booking_reference),
            csv(&document.payload.freight_terms),
            csv(&document.payload.bill_of_lading_type),
            csv(&document.payload.bill_of_lading_number),
            csv(&document.payload.carrier),
            csv(&document.payload.notify_party),
            csv(&document.payload.place_of_receipt),
            csv(&document.payload.place_of_delivery),
            csv(&document.payload.container_numbers),
            csv(&document.payload.seal_numbers),
            csv(&document.payload.shipping_marks),
            csv(&document.payload.insurance_company),
            csv(&document.payload.policy_number),
            minor(document.payload.insured_value_minor),
            document.payload.insurance_markup_percent.to_string(),
            document.payload.premium_rate_percent.to_string(),
            minor(document.payload.premium_minor),
            csv(&document.payload.insurance_coverage),
            csv(&document.payload.claims_payable_at),
            csv(&document.payload.customs_supervision_code),
            csv(&document.payload.customs_declaration_elements),
            csv(&document.payload.certificate_number),
            csv(&document.payload.certificate_type),
            csv(&document.payload.certification_authority),
            csv(&document.payload.manufacturer),
            csv(&document.payload.manufacturer_address),
            csv(&document.payload.batch_number),
            csv(&document.payload.inspection_standard),
            csv(&document.payload.inspection_date),
            csv(&document.payload.inspection_place),
            csv(&document.payload.inspection_result),
            csv(&document.payload.fumigation_agent),
            csv(&document.payload.fumigation_method),
            document.payload.fumigation_temperature_celsius.to_string(),
            document.payload.fumigation_duration_hours.to_string(),
            csv(&document.payload.fumigation_date),
            csv(&document.payload.fumigation_place),
            csv(&document.payload.fumigation_operator),
            csv(&document.payload.fumigation_license_number),
            csv(&document.payload.letter_of_credit_number),
            csv(&document.payload.issuing_bank),
            csv(&document.payload.letter_of_credit_issue_date),
            csv(&document.payload.letter_of_credit_expiry_date),
            csv(&document.payload.presentation_deadline),
            csv(&document.payload.beneficiary_certificate_type),
            csv(&document.payload.beneficiary_statement),
            csv(&document.payload.letter_of_credit_terms),
            csv(&document.payload.required_documents),
        ]
        .join(","),
        String::new(),
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
    language: &str,
    company_profile: &CompanyProfile,
    typst_path: &Path,
    work_dir: &Path,
    output_dir: &Path,
) -> Result<DocumentExportResult, String> {
    fs::create_dir_all(work_dir).map_err(|error| format!("无法创建配置单临时目录：{error}"))?;
    fs::create_dir_all(output_dir).map_err(|error| format!("无法创建配置单导出目录：{error}"))?;
    let data_path = work_dir.join("configuration.json");
    let template_path = work_dir.join("configuration.typ");
    let output_path = output_dir.join(format!("{}.pdf", configuration_stem(configuration)));
    let branding = prepare_branding(company_profile, work_dir)?;
    let payload = serde_json::to_vec_pretty(&serde_json::json!({
        "configuration": configuration,
        "labels": configuration_labels(language)?,
        "language": language,
        "rtl": language == "ar",
        "branding": branding,
    }))
    .map_err(|error| format!("无法生成配置清单快照：{error}"))?;
    fs::write(&data_path, payload).map_err(|error| format!("无法写入配置清单快照：{error}"))?;
    fs::write(work_dir.join("helpers.typ"), TEMPLATE_HELPERS)
        .map_err(|error| format!("无法写入配置单排版助手：{error}"))?;
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
    cleanup_branding_assets(work_dir);
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
    language: &str,
    output_dir: &Path,
) -> Result<PathBuf, String> {
    fs::create_dir_all(output_dir).map_err(|error| format!("无法创建配置单导出目录：{error}"))?;
    let output_path = output_dir.join(format!("{}.csv", configuration_stem(configuration)));
    let labels = configuration_labels(language)?;
    let mut rows = vec![
        [
            csv(labels.code),
            csv(&configuration.code),
            csv(labels.product_name),
            csv(&configuration.name),
            csv(labels.model),
            csv(&configuration.model),
            csv(labels.currency),
            csv(&configuration.currency),
        ]
        .join(","),
        [
            csv(labels.exchange_rate),
            csv(&format!(
                "1 CNY = {} {}",
                configuration.exchange_rate, configuration.currency
            )),
            csv(labels.rate_date),
            csv(&configuration.exchange_rate_date),
        ]
        .join(","),
        [csv(labels.notes), csv(&configuration.notes)].join(","),
        String::new(),
        [
            labels.number,
            labels.item_name,
            labels.specification,
            labels.quantity,
            labels.unit,
            labels.unit_price,
            labels.amount,
            labels.currency,
            labels.brand,
            labels.notes,
        ]
        .map(csv)
        .join(","),
    ];
    for (index, line) in configuration.lines.iter().enumerate() {
        rows.push(
            [
                (index + 1).to_string(),
                csv(&format!("{} / {}", line.category, line.name)),
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
            csv(labels.configuration_total),
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

pub fn export_purchase_order_pdf(
    order: &PurchaseOrder,
    sales_currency: &str,
    company_profile: &CompanyProfile,
    typst_path: &Path,
    work_dir: &Path,
    output_dir: &Path,
) -> Result<DocumentExportResult, String> {
    fs::create_dir_all(work_dir).map_err(|error| format!("无法创建采购单临时目录：{error}"))?;
    fs::create_dir_all(output_dir).map_err(|error| format!("无法创建采购单导出目录：{error}"))?;
    let data_path = work_dir.join("purchase-order.json");
    let template_path = work_dir.join("purchase-order.typ");
    let output_path = output_dir.join(format!("{}.pdf", purchase_order_stem(order)));
    let branding = prepare_branding(company_profile, work_dir)?;
    let payload = serde_json::to_vec_pretty(&serde_json::json!({
        "purchaseOrder": order,
        "salesCurrency": sales_currency,
        "statusLabel": purchase_status_label(&order.status),
        "branding": branding,
    }))
    .map_err(|error| format!("无法生成采购单快照：{error}"))?;
    fs::write(&data_path, payload).map_err(|error| format!("无法写入采购单快照：{error}"))?;
    fs::write(work_dir.join("helpers.typ"), TEMPLATE_HELPERS)
        .map_err(|error| format!("无法写入采购单排版助手：{error}"))?;
    fs::write(&template_path, PURCHASE_ORDER_TEMPLATE)
        .map_err(|error| format!("无法写入采购单模板：{error}"))?;
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
    cleanup_branding_assets(work_dir);
    if !output.status.success() {
        return Err(format!(
            "采购单 PDF 生成失败：{}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let bytes =
        fs::read(&output_path).map_err(|error| format!("无法读取生成的采购单 PDF：{error}"))?;
    Ok(DocumentExportResult {
        path: output_path.to_string_lossy().into_owned(),
        sha256: format!("{:x}", Sha256::digest(&bytes)),
        exported_at: String::new(),
    })
}

pub fn export_purchase_order_csv(
    order: &PurchaseOrder,
    sales_currency: &str,
    output_dir: &Path,
) -> Result<PathBuf, String> {
    fs::create_dir_all(output_dir).map_err(|error| format!("无法创建采购单导出目录：{error}"))?;
    let output_path = output_dir.join(format!("{}.csv", purchase_order_stem(order)));
    let fx_snapshot = if order.exchange_rate_date.is_empty() {
        String::new()
    } else {
        format!(
            "1 {sales_currency} = {} {}",
            order.exchange_rate, order.currency
        )
    };
    let mut rows = vec![
        [
            "purchase_order_number",
            "supplier",
            "status",
            "sales_reference",
            "expected_date",
            "currency",
        ]
        .map(csv)
        .join(","),
        [
            csv(&order.number),
            csv(&order.supplier_name),
            csv(purchase_status_label(&order.status)),
            csv(&order.business_case_number),
            csv(&order.expected_date),
            csv(&order.currency),
        ]
        .join(","),
        [
            csv("exchange_rate"),
            csv(&fx_snapshot),
            csv("exchange_rate_date"),
            csv(&order.exchange_rate_date),
            csv("notes"),
            csv(&order.notes),
        ]
        .join(","),
        String::new(),
        [
            "line",
            "sku",
            "name_zh",
            "name_en",
            "quantity",
            "unit",
            "unit_cost",
            "amount",
            "currency",
        ]
        .map(csv)
        .join(","),
    ];
    for (index, line) in order.lines.iter().enumerate() {
        rows.push(
            [
                (index + 1).to_string(),
                csv(&line.sku),
                csv(&line.name_zh),
                csv(&line.name_en),
                line.quantity.to_string(),
                csv(&line.unit),
                minor(line.unit_cost_minor),
                minor(line.amount_minor),
                csv(&order.currency),
            ]
            .join(","),
        );
    }
    rows.push(
        [
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            csv("TOTAL"),
            String::new(),
            minor(order.total_amount_minor),
            csv(&order.currency),
        ]
        .join(","),
    );
    fs::write(&output_path, format!("\u{feff}{}\r\n", rows.join("\r\n")))
        .map_err(|error| format!("无法写入采购单 CSV：{error}"))?;
    Ok(output_path)
}

fn purchase_status_label(status: &PurchaseStatus) -> &'static str {
    match status {
        PurchaseStatus::Draft => "草稿 / Draft",
        PurchaseStatus::PendingConfirmation => "待确认 / Pending Confirmation",
        PurchaseStatus::Confirmed => "已确认 / Confirmed",
        PurchaseStatus::InProduction => "生产中 / In Production",
        PurchaseStatus::ReadyToShip => "可发货 / Ready to Ship",
        PurchaseStatus::Completed => "已完成 / Completed",
        PurchaseStatus::Cancelled => "已取消 / Cancelled",
    }
}

fn configuration_labels(language: &str) -> Result<ConfigurationLabels, String> {
    let labels = match language {
        "en" => ConfigurationLabels {
            title: "PRODUCT CONFIGURATION SHEET",
            code: "Configuration Code",
            product_name: "Product Name",
            currency: "Currency",
            exchange_rate: "Exchange Rate",
            rate_date: "Rate Date",
            model: "Model",
            component_count: "Component Count",
            configuration_total: "Configuration Total",
            number: "No.",
            item_name: "Item Name",
            specification: "Model / Specification / Material",
            quantity: "Quantity",
            unit: "Unit",
            unit_price: "Unit Price",
            amount: "Amount",
            brand: "Brand",
            notes: "Notes",
            prepared_by: "Prepared by",
            snapshot_notice: "Prices are based on the saved configuration snapshot.",
        },
        "ru" => ConfigurationLabels {
            title: "ЛИСТ КОМПЛЕКТАЦИИ ИЗДЕЛИЯ",
            code: "Код конфигурации",
            product_name: "Наименование изделия",
            currency: "Валюта",
            exchange_rate: "Курс обмена",
            rate_date: "Дата курса",
            model: "Модель",
            component_count: "Количество компонентов",
            configuration_total: "Итоговая стоимость",
            number: "№",
            item_name: "Наименование",
            specification: "Модель / спецификация / материал",
            quantity: "Количество",
            unit: "Ед.",
            unit_price: "Цена за единицу",
            amount: "Сумма",
            brand: "Марка",
            notes: "Примечания",
            prepared_by: "Составил",
            snapshot_notice: "Цены указаны по сохраненному снимку конфигурации.",
        },
        "fr" => ConfigurationLabels {
            title: "FICHE DE CONFIGURATION DU PRODUIT",
            code: "Référence configuration",
            product_name: "Nom du produit",
            currency: "Devise",
            exchange_rate: "Taux de change",
            rate_date: "Date du taux",
            model: "Modèle",
            component_count: "Nombre de composants",
            configuration_total: "Total configuration",
            number: "N°",
            item_name: "Désignation",
            specification: "Modèle / Spécification / Matériau",
            quantity: "Quantité",
            unit: "Unité",
            unit_price: "Prix unitaire",
            amount: "Montant",
            brand: "Marque",
            notes: "Remarques",
            prepared_by: "Préparé par",
            snapshot_notice: "Les prix correspondent à la configuration enregistrée.",
        },
        "es" => ConfigurationLabels {
            title: "HOJA DE CONFIGURACIÓN DEL PRODUCTO",
            code: "Código de configuración",
            product_name: "Nombre del producto",
            currency: "Moneda",
            exchange_rate: "Tipo de cambio",
            rate_date: "Fecha del tipo",
            model: "Modelo",
            component_count: "Número de componentes",
            configuration_total: "Total de configuración",
            number: "N.º",
            item_name: "Nombre del artículo",
            specification: "Modelo / Especificación / Material",
            quantity: "Cantidad",
            unit: "Unidad",
            unit_price: "Precio unitario",
            amount: "Importe",
            brand: "Marca",
            notes: "Observaciones",
            prepared_by: "Preparado por",
            snapshot_notice: "Los precios corresponden a la configuración guardada.",
        },
        "pt" => ConfigurationLabels {
            title: "FICHA DE CONFIGURAÇÃO DO PRODUTO",
            code: "Código da configuração",
            product_name: "Nome do produto",
            currency: "Moeda",
            exchange_rate: "Taxa de câmbio",
            rate_date: "Data da taxa",
            model: "Modelo",
            component_count: "Número de componentes",
            configuration_total: "Total da configuração",
            number: "Nº",
            item_name: "Nome do item",
            specification: "Modelo / Especificação / Material",
            quantity: "Quantidade",
            unit: "Unidade",
            unit_price: "Preço unitário",
            amount: "Valor total",
            brand: "Marca",
            notes: "Observações",
            prepared_by: "Preparado por",
            snapshot_notice: "Os preços correspondem à configuração guardada.",
        },
        "ar" => ConfigurationLabels {
            title: "ورقة تكوين المنتج",
            code: "رمز التكوين",
            product_name: "اسم المنتج",
            currency: "العملة",
            exchange_rate: "سعر الصرف",
            rate_date: "تاريخ سعر الصرف",
            model: "الطراز",
            component_count: "عدد المكونات",
            configuration_total: "إجمالي التكوين",
            number: "الرقم",
            item_name: "اسم الصنف",
            specification: "الطراز / المواصفات / المادة",
            quantity: "الكمية",
            unit: "الوحدة",
            unit_price: "سعر الوحدة",
            amount: "الإجمالي",
            brand: "العلامة التجارية",
            notes: "ملاحظات",
            prepared_by: "إعداد",
            snapshot_notice: "تعتمد الأسعار على نسخة التكوين المحفوظة.",
        },
        _ => return Err("不支持的配置单语言。".to_owned()),
    };
    Ok(labels)
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
        DocumentType::ShippingMarks => SHIPPING_MARKS_TEMPLATE,
        DocumentType::ShipperInstruction => SHIPPER_INSTRUCTION_TEMPLATE,
        DocumentType::CustomsDeclaration => CUSTOMS_DECLARATION_TEMPLATE,
        DocumentType::BillOfLading => BILL_OF_LADING_TEMPLATE,
        DocumentType::InsurancePolicy => INSURANCE_POLICY_TEMPLATE,
        DocumentType::CertificateOfOrigin => CERTIFICATE_OF_ORIGIN_TEMPLATE,
        DocumentType::InspectionCertificate => INSPECTION_CERTIFICATE_TEMPLATE,
        DocumentType::FumigationCertificate => FUMIGATION_CERTIFICATE_TEMPLATE,
        DocumentType::BeneficiaryCertificate => BENEFICIARY_CERTIFICATE_TEMPLATE,
    }
}

fn export_stem(document: &TradeDocument) -> String {
    let kind = match document.document_type {
        DocumentType::CommercialQuotation => "CommercialQuotation",
        DocumentType::ProformaInvoice => "ProformaInvoice",
        DocumentType::CommercialInvoice => "CommercialInvoice",
        DocumentType::PackingList => "PackingList",
        DocumentType::TradeContract => "TradeContract",
        DocumentType::ShippingMarks => "ShippingMarks",
        DocumentType::ShipperInstruction => "ShipperInstruction",
        DocumentType::CustomsDeclaration => "CustomsDeclaration",
        DocumentType::BillOfLading => "BillOfLading",
        DocumentType::InsurancePolicy => "InsuranceApplication",
        DocumentType::CertificateOfOrigin => "CertificateOfOriginApplication",
        DocumentType::InspectionCertificate => "InspectionCertificateApplication",
        DocumentType::FumigationCertificate => "FumigationCertificateApplication",
        DocumentType::BeneficiaryCertificate => "BeneficiaryCertificate",
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

fn purchase_order_stem(order: &PurchaseOrder) -> String {
    sanitize_stem(&format!(
        "PurchaseOrder_{}_{}",
        order.number, order.supplier_name
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
    use super::{csv, validate_company_profile};
    use crate::domain::CompanyProfile;

    #[test]
    fn csv_escapes_quotes() {
        assert_eq!(csv("A, \"B\""), "\"A, \"\"B\"\"\"");
    }

    #[test]
    fn validates_encrypted_company_assets() {
        let profile = CompanyProfile {
            company_name: "Example Export Co., Ltd.".to_owned(),
            logo_data_url: "data:image/png;base64,iVBORw0KGgo=".to_owned(),
            signature_data_url: String::new(),
            signing_asset_kind: String::new(),
        };
        assert!(validate_company_profile(&profile).is_ok());
        assert!(
            validate_company_profile(&CompanyProfile {
                logo_data_url: "data:image/svg+xml;base64,PHN2Zz4=".to_owned(),
                ..profile
            })
            .is_err()
        );
    }
}
