use std::{collections::HashSet, io::Cursor, path::Path};

use calamine::{Data, Reader, Xlsx};
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook, Worksheet};
use serde::Serialize;

use crate::domain::{
    ConfigComponent, ConfigComponentInput, ConfigurableProduct, ConfigurableProductInput,
    ConfigurableProductLineInput, Customer, CustomerInput, Product, ProductInput, Supplier,
    SupplierInput, SupplierProductTermInput,
};

const SHEET_PRODUCTS: &str = "产品";
const SHEET_CUSTOMERS: &str = "客户";
const SHEET_SUPPLIERS: &str = "供应商";
const SHEET_SUPPLIER_PRODUCTS: &str = "供应商产品";
const SHEET_COMPONENTS: &str = "组件库";
const SHEET_CONFIGURATIONS: &str = "自选配置";
const SHEET_CONFIGURATION_LINES: &str = "配置明细";

#[derive(Debug)]
pub struct ConfigurationImport {
    pub code: String,
    pub name: String,
    pub model: String,
    pub currency: String,
    pub exchange_rate: f64,
    pub exchange_rate_date: String,
    pub notes: String,
    pub lines: Vec<ConfigurationLineImport>,
}

#[derive(Debug)]
pub struct ConfigurationLineImport {
    pub component_code: String,
    pub quantity: f64,
    pub unit_price_minor: i64,
}

#[derive(Debug, Default)]
pub struct MasterImportData {
    pub products: Vec<ProductInput>,
    pub customers: Vec<CustomerInput>,
    pub suppliers: Vec<SupplierInput>,
    pub components: Vec<ConfigComponentInput>,
    pub configurations: Vec<ConfigurationImport>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MasterImportResult {
    pub products: usize,
    pub customers: usize,
    pub suppliers: usize,
    pub components: usize,
    pub configurations: usize,
}

fn header_format() -> Format {
    Format::new()
        .set_bold()
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0x6D28D9))
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center)
        .set_text_wrap()
}

fn setup_sheet<'a>(
    workbook: &'a mut Workbook,
    name: &str,
    headers: &[&str],
    widths: &[f64],
) -> Result<&'a mut Worksheet, String> {
    let header = header_format();
    let sheet = workbook.add_worksheet();
    sheet.set_name(name).map_err(|error| error.to_string())?;
    sheet
        .set_freeze_panes(1, 0)
        .map_err(|error| error.to_string())?;
    for (column, title) in headers.iter().enumerate() {
        sheet
            .write_string_with_format(0, column as u16, *title, &header)
            .map_err(|error| error.to_string())?;
        sheet
            .set_column_width(column as u16, widths.get(column).copied().unwrap_or(16.0))
            .map_err(|error| error.to_string())?;
    }
    Ok(sheet)
}

fn write_text(
    sheet: &mut Worksheet,
    row: u32,
    col: u16,
    value: impl Into<String>,
) -> Result<(), String> {
    sheet
        .write_string(row, col, value)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn write_number(sheet: &mut Worksheet, row: u32, col: u16, value: f64) -> Result<(), String> {
    sheet
        .write_number(row, col, value)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

pub fn export_master_workbook(
    output: &Path,
    template_only: bool,
    products: &[Product],
    customers: &[Customer],
    suppliers: &[Supplier],
    components: &[ConfigComponent],
    configurations: &[ConfigurableProduct],
) -> Result<(), String> {
    let mut workbook = Workbook::new();
    let instructions = workbook.add_worksheet();
    instructions
        .set_name("导入说明")
        .map_err(|error| error.to_string())?;
    instructions
        .set_column_width(0, 24)
        .map_err(|error| error.to_string())?;
    instructions
        .set_column_width(1, 90)
        .map_err(|error| error.to_string())?;
    for (row, (title, note)) in [
        ("TradeDesk 主数据模板", "请保留工作表名称和第一行字段。带 * 的字段必填；空白数据行会被忽略。"),
        ("更新规则", "系统按 SKU、客户编号、供应商编号、组件编号、配置编号匹配；相同编号会更新，新增编号会创建。"),
        ("金额", "组件和配置明细中的单价按元填写，例如 1234.56；系统内部自动转换为分。"),
        ("自选配置", "先填写组件库，再填写自选配置和配置明细；配置明细通过配置编号、组件编号建立关联。"),
        ("数据安全", "导入前会校验全部工作表。若发现必填项、数字范围或关联错误，不会开始写入。"),
    ].iter().enumerate() {
        write_text(instructions, row as u32, 0, *title)?;
        write_text(instructions, row as u32, 1, *note)?;
    }

    let sheet = setup_sheet(
        &mut workbook,
        SHEET_PRODUCTS,
        &[
            "SKU *",
            "中文名称",
            "英文名称 *",
            "型号",
            "HS 编码",
            "单位 *",
            "毛重 kg",
        ],
        &[18.0, 24.0, 28.0, 18.0, 16.0, 12.0, 14.0],
    )?;
    if !template_only {
        for (index, item) in products.iter().enumerate() {
            let row = index as u32 + 1;
            write_text(sheet, row, 0, &item.sku)?;
            write_text(sheet, row, 1, &item.name_zh)?;
            write_text(sheet, row, 2, &item.name_en)?;
            write_text(sheet, row, 3, &item.model)?;
            write_text(sheet, row, 4, &item.hs_code)?;
            write_text(sheet, row, 5, &item.unit)?;
            write_number(sheet, row, 6, item.gross_weight_kg)?;
        }
    }

    let sheet = setup_sheet(
        &mut workbook,
        SHEET_CUSTOMERS,
        &[
            "客户编号 *",
            "公司全称 *",
            "市场",
            "币种 *",
            "付款条款",
            "客户地址",
            "收货地址",
            "账单地址",
            "购买意向",
            "客户分析",
            "优势",
            "劣势",
            "主要人员和联系方式",
        ],
        &[
            16.0, 28.0, 16.0, 12.0, 20.0, 32.0, 32.0, 32.0, 28.0, 28.0, 24.0, 24.0, 32.0,
        ],
    )?;
    if !template_only {
        for (index, item) in customers.iter().enumerate() {
            let row = index as u32 + 1;
            for (col, value) in [
                &item.code,
                &item.legal_name,
                &item.market,
                &item.currency,
                &item.payment_terms,
                &item.address,
                &item.shipping_address,
                &item.billing_address,
                &item.purchase_intent,
                &item.customer_analysis,
                &item.strengths,
                &item.weaknesses,
                &item.contacts,
            ]
            .iter()
            .enumerate()
            {
                write_text(sheet, row, col as u16, (*value).clone())?;
            }
        }
    }

    let sheet = setup_sheet(
        &mut workbook,
        SHEET_SUPPLIERS,
        &[
            "供应商编号 *",
            "公司全称 *",
            "地址",
            "联系人",
            "银行资料",
            "默认币种 *",
            "付款条款",
            "默认交期（天）",
            "准时率（0-100）",
            "资质/质量/评估备注",
        ],
        &[18.0, 30.0, 30.0, 28.0, 32.0, 14.0, 24.0, 18.0, 18.0, 32.0],
    )?;
    if !template_only {
        for (index, item) in suppliers.iter().enumerate() {
            let row = index as u32 + 1;
            write_text(sheet, row, 0, &item.code)?;
            write_text(sheet, row, 1, &item.legal_name)?;
            write_text(sheet, row, 2, &item.address)?;
            write_text(sheet, row, 3, &item.contacts)?;
            write_text(sheet, row, 4, &item.bank_details)?;
            write_text(sheet, row, 5, &item.currency)?;
            write_text(sheet, row, 6, &item.payment_terms)?;
            write_number(sheet, row, 7, item.lead_time_days as f64)?;
            write_number(sheet, row, 8, item.on_time_rate as f64)?;
            write_text(sheet, row, 9, &item.qualification_notes)?;
        }
    }

    let sheet = setup_sheet(
        &mut workbook,
        SHEET_SUPPLIER_PRODUCTS,
        &[
            "供应商编号 *",
            "产品 SKU *",
            "币种 *",
            "采购单价 *",
            "MOQ *",
            "交期（天）",
        ],
        &[18.0, 20.0, 14.0, 18.0, 14.0, 18.0],
    )?;
    if !template_only {
        let mut row = 1_u32;
        for supplier in suppliers {
            for term in &supplier.product_terms {
                write_text(sheet, row, 0, &supplier.code)?;
                write_text(sheet, row, 1, &term.product_sku)?;
                write_text(sheet, row, 2, &term.currency)?;
                write_number(sheet, row, 3, term.unit_price_minor as f64 / 100.0)?;
                write_number(sheet, row, 4, term.moq)?;
                write_number(sheet, row, 5, term.lead_time_days as f64)?;
                row += 1;
            }
        }
    }

    let sheet = setup_sheet(
        &mut workbook,
        SHEET_COMPONENTS,
        &[
            "组件编号 *",
            "组件类别 *",
            "品名 *",
            "型号/规格/材质",
            "默认数量",
            "单位 *",
            "人民币单价",
            "币种 *",
            "品牌",
            "备注",
        ],
        &[18.0, 18.0, 24.0, 28.0, 14.0, 12.0, 16.0, 12.0, 18.0, 28.0],
    )?;
    if !template_only {
        for (index, item) in components.iter().enumerate() {
            let row = index as u32 + 1;
            write_text(sheet, row, 0, &item.code)?;
            write_text(sheet, row, 1, &item.category)?;
            write_text(sheet, row, 2, &item.name)?;
            write_text(sheet, row, 3, &item.specification)?;
            write_number(sheet, row, 4, item.default_quantity)?;
            write_text(sheet, row, 5, &item.unit)?;
            write_number(sheet, row, 6, item.unit_price_minor as f64 / 100.0)?;
            write_text(sheet, row, 7, &item.currency)?;
            write_text(sheet, row, 8, &item.brand)?;
            write_text(sheet, row, 9, &item.notes)?;
        }
    }

    let sheet = setup_sheet(
        &mut workbook,
        SHEET_CONFIGURATIONS,
        &[
            "配置编号 *",
            "产品名称 *",
            "型号",
            "报价币种 *",
            "1 CNY 换算汇率",
            "汇率日期",
            "报价说明",
        ],
        &[18.0, 28.0, 20.0, 14.0, 18.0, 16.0, 30.0],
    )?;
    if !template_only {
        for (index, item) in configurations.iter().enumerate() {
            let row = index as u32 + 1;
            write_text(sheet, row, 0, &item.code)?;
            write_text(sheet, row, 1, &item.name)?;
            write_text(sheet, row, 2, &item.model)?;
            write_text(sheet, row, 3, &item.currency)?;
            write_number(sheet, row, 4, item.exchange_rate)?;
            write_text(sheet, row, 5, &item.exchange_rate_date)?;
            write_text(sheet, row, 6, &item.notes)?;
        }
    }

    let component_codes = components
        .iter()
        .map(|item| (item.id.as_str(), item.code.as_str()))
        .collect::<std::collections::HashMap<_, _>>();
    let sheet = setup_sheet(
        &mut workbook,
        SHEET_CONFIGURATION_LINES,
        &["配置编号 *", "序号", "组件编号 *", "数量 *", "报价单价 *"],
        &[18.0, 10.0, 18.0, 14.0, 16.0],
    )?;
    if !template_only {
        let mut row = 1;
        for configuration in configurations {
            for (index, line) in configuration.lines.iter().enumerate() {
                write_text(sheet, row, 0, &configuration.code)?;
                write_number(sheet, row, 1, (index + 1) as f64)?;
                write_text(
                    sheet,
                    row,
                    2,
                    component_codes
                        .get(line.component_id.as_str())
                        .copied()
                        .unwrap_or(""),
                )?;
                write_number(sheet, row, 3, line.quantity)?;
                write_number(sheet, row, 4, line.unit_price_minor as f64 / 100.0)?;
                row += 1;
            }
        }
    }

    workbook.save(output).map_err(|error| error.to_string())
}

fn text(row: &[Data], index: usize) -> String {
    row.get(index)
        .map(ToString::to_string)
        .unwrap_or_default()
        .trim()
        .to_owned()
}
fn number(
    row: &[Data],
    index: usize,
    sheet: &str,
    row_number: usize,
    default: Option<f64>,
) -> Result<f64, String> {
    let value = text(row, index);
    if value.is_empty() {
        return default.ok_or_else(|| {
            format!(
                "{sheet} 第 {row_number} 行：第 {} 列需要填写数字",
                index + 1
            )
        });
    }
    value.parse::<f64>().map_err(|_| {
        format!(
            "{sheet} 第 {row_number} 行：第 {} 列不是有效数字",
            index + 1
        )
    })
}
fn required(value: String, sheet: &str, row: usize, field: &str) -> Result<String, String> {
    if value.is_empty() {
        Err(format!("{sheet} 第 {row} 行：{field}不能为空"))
    } else {
        Ok(value)
    }
}
fn unique(keys: impl Iterator<Item = String>, sheet: &str) -> Result<(), String> {
    let mut seen = HashSet::new();
    for key in keys {
        if !seen.insert(key.to_lowercase()) {
            return Err(format!("{sheet} 中的编号“{key}”重复"));
        }
    }
    Ok(())
}

pub fn parse_master_workbook(bytes: &[u8]) -> Result<MasterImportData, String> {
    if bytes.len() > 15 * 1024 * 1024 {
        return Err("Excel 文件不能超过 15 MB".to_owned());
    }
    let mut workbook: Xlsx<_> =
        Xlsx::new(Cursor::new(bytes)).map_err(|error| format!("无法读取 Excel：{error}"))?;
    let mut data = MasterImportData::default();

    let range = workbook
        .worksheet_range(SHEET_PRODUCTS)
        .map_err(|_| format!("缺少工作表：{SHEET_PRODUCTS}"))?;
    for (index, row) in range.rows().skip(1).enumerate() {
        if row.iter().all(|cell| cell.to_string().trim().is_empty()) {
            continue;
        }
        let row_no = index + 2;
        let weight = number(row, 6, SHEET_PRODUCTS, row_no, Some(0.0))?;
        if weight < 0.0 {
            return Err(format!("{SHEET_PRODUCTS} 第 {row_no} 行：毛重不能为负数"));
        }
        data.products.push(ProductInput {
            id: None,
            sku: required(text(row, 0), SHEET_PRODUCTS, row_no, "SKU")?,
            name_zh: text(row, 1),
            name_en: required(text(row, 2), SHEET_PRODUCTS, row_no, "英文名称")?,
            model: text(row, 3),
            hs_code: text(row, 4),
            unit: required(text(row, 5), SHEET_PRODUCTS, row_no, "单位")?,
            gross_weight_kg: weight,
        });
    }

    let range = workbook
        .worksheet_range(SHEET_CUSTOMERS)
        .map_err(|_| format!("缺少工作表：{SHEET_CUSTOMERS}"))?;
    for (index, row) in range.rows().skip(1).enumerate() {
        if row.iter().all(|cell| cell.to_string().trim().is_empty()) {
            continue;
        }
        let row_no = index + 2;
        data.customers.push(CustomerInput {
            id: None,
            code: required(text(row, 0), SHEET_CUSTOMERS, row_no, "客户编号")?,
            legal_name: required(text(row, 1), SHEET_CUSTOMERS, row_no, "公司全称")?,
            market: text(row, 2),
            currency: required(text(row, 3), SHEET_CUSTOMERS, row_no, "币种")?.to_uppercase(),
            payment_terms: text(row, 4),
            address: text(row, 5),
            shipping_address: text(row, 6),
            billing_address: text(row, 7),
            purchase_intent: text(row, 8),
            customer_analysis: text(row, 9),
            strengths: text(row, 10),
            weaknesses: text(row, 11),
            contacts: text(row, 12),
        });
    }

    let range = workbook
        .worksheet_range(SHEET_SUPPLIERS)
        .map_err(|_| format!("缺少工作表：{SHEET_SUPPLIERS}"))?;
    for (index, row) in range.rows().skip(1).enumerate() {
        if row.iter().all(|cell| cell.to_string().trim().is_empty()) {
            continue;
        }
        let row_no = index + 2;
        let expanded = row.len() >= 10;
        let lead_time_days = number(
            row,
            if expanded { 7 } else { 2 },
            SHEET_SUPPLIERS,
            row_no,
            Some(0.0),
        )? as i64;
        let on_time_rate = number(
            row,
            if expanded { 8 } else { 3 },
            SHEET_SUPPLIERS,
            row_no,
            Some(100.0),
        )? as i64;
        if lead_time_days < 0 || !(0..=100).contains(&on_time_rate) {
            return Err(format!(
                "{SHEET_SUPPLIERS} 第 {row_no} 行：交期或准时率超出范围"
            ));
        }
        data.suppliers.push(SupplierInput {
            id: None,
            code: required(text(row, 0), SHEET_SUPPLIERS, row_no, "供应商编号")?,
            legal_name: required(text(row, 1), SHEET_SUPPLIERS, row_no, "公司全称")?,
            address: if expanded {
                text(row, 2)
            } else {
                String::new()
            },
            contacts: if expanded {
                text(row, 3)
            } else {
                String::new()
            },
            bank_details: if expanded {
                text(row, 4)
            } else {
                String::new()
            },
            currency: if expanded {
                required(text(row, 5), SHEET_SUPPLIERS, row_no, "默认币种")?.to_uppercase()
            } else {
                "CNY".to_owned()
            },
            payment_terms: if expanded {
                text(row, 6)
            } else {
                String::new()
            },
            lead_time_days,
            on_time_rate,
            qualification_notes: if expanded {
                text(row, 9)
            } else {
                String::new()
            },
            product_terms: Vec::new(),
        });
    }

    if let Ok(range) = workbook.worksheet_range(SHEET_SUPPLIER_PRODUCTS) {
        for (index, row) in range.rows().skip(1).enumerate() {
            if row.iter().all(|cell| cell.to_string().trim().is_empty()) {
                continue;
            }
            let row_no = index + 2;
            let supplier_code =
                required(text(row, 0), SHEET_SUPPLIER_PRODUCTS, row_no, "供应商编号")?;
            let product_sku = required(text(row, 1), SHEET_SUPPLIER_PRODUCTS, row_no, "产品 SKU")?;
            let price = number(row, 3, SHEET_SUPPLIER_PRODUCTS, row_no, None)?;
            let moq = number(row, 4, SHEET_SUPPLIER_PRODUCTS, row_no, Some(1.0))?;
            let lead_time_days = number(row, 5, SHEET_SUPPLIER_PRODUCTS, row_no, Some(0.0))? as i64;
            if price <= 0.0 || moq <= 0.0 || lead_time_days < 0 {
                return Err(format!(
                    "{SHEET_SUPPLIER_PRODUCTS} 第 {row_no} 行：采购价和 MOQ 必须大于零，交期不能为负数"
                ));
            }
            let supplier = data.suppliers.iter_mut().find(|item| item.code.eq_ignore_ascii_case(&supplier_code))
            .ok_or_else(|| format!("{SHEET_SUPPLIER_PRODUCTS} 第 {row_no} 行：供应商编号“{supplier_code}”未在供应商工作表中定义"))?;
            supplier.product_terms.push(SupplierProductTermInput {
                id: None,
                product_id: String::new(),
                product_sku,
                currency: required(text(row, 2), SHEET_SUPPLIER_PRODUCTS, row_no, "币种")?
                    .to_uppercase(),
                unit_price_minor: (price * 100.0).round() as i64,
                moq,
                lead_time_days,
            });
        }
    }

    let range = workbook
        .worksheet_range(SHEET_COMPONENTS)
        .map_err(|_| format!("缺少工作表：{SHEET_COMPONENTS}"))?;
    for (index, row) in range.rows().skip(1).enumerate() {
        if row.iter().all(|cell| cell.to_string().trim().is_empty()) {
            continue;
        }
        let row_no = index + 2;
        let quantity = number(row, 4, SHEET_COMPONENTS, row_no, Some(1.0))?;
        let price = number(row, 6, SHEET_COMPONENTS, row_no, Some(0.0))?;
        if quantity <= 0.0 || price < 0.0 {
            return Err(format!(
                "{SHEET_COMPONENTS} 第 {row_no} 行：数量必须大于 0 且单价不能为负数"
            ));
        }
        data.components.push(ConfigComponentInput {
            id: None,
            code: required(text(row, 0), SHEET_COMPONENTS, row_no, "组件编号")?,
            category: required(text(row, 1), SHEET_COMPONENTS, row_no, "组件类别")?,
            name: required(text(row, 2), SHEET_COMPONENTS, row_no, "品名")?,
            specification: text(row, 3),
            default_quantity: quantity,
            unit: required(text(row, 5), SHEET_COMPONENTS, row_no, "单位")?,
            unit_price_minor: (price * 100.0).round() as i64,
            currency: required(text(row, 7), SHEET_COMPONENTS, row_no, "币种")?.to_uppercase(),
            brand: text(row, 8),
            notes: text(row, 9),
        });
    }

    let range = workbook
        .worksheet_range(SHEET_CONFIGURATIONS)
        .map_err(|_| format!("缺少工作表：{SHEET_CONFIGURATIONS}"))?;
    for (index, row) in range.rows().skip(1).enumerate() {
        if row.iter().all(|cell| cell.to_string().trim().is_empty()) {
            continue;
        }
        let row_no = index + 2;
        let currency =
            required(text(row, 3), SHEET_CONFIGURATIONS, row_no, "报价币种")?.to_uppercase();
        let rate = number(row, 4, SHEET_CONFIGURATIONS, row_no, Some(1.0))?;
        if rate <= 0.0 {
            return Err(format!(
                "{SHEET_CONFIGURATIONS} 第 {row_no} 行：汇率必须大于 0"
            ));
        }
        data.configurations.push(ConfigurationImport {
            code: required(text(row, 0), SHEET_CONFIGURATIONS, row_no, "配置编号")?,
            name: required(text(row, 1), SHEET_CONFIGURATIONS, row_no, "产品名称")?,
            model: text(row, 2),
            currency,
            exchange_rate: rate,
            exchange_rate_date: text(row, 5),
            notes: text(row, 6),
            lines: Vec::new(),
        });
    }

    unique(
        data.products.iter().map(|item| item.sku.clone()),
        SHEET_PRODUCTS,
    )?;
    unique(
        data.customers.iter().map(|item| item.code.clone()),
        SHEET_CUSTOMERS,
    )?;
    unique(
        data.suppliers.iter().map(|item| item.code.clone()),
        SHEET_SUPPLIERS,
    )?;
    unique(
        data.components.iter().map(|item| item.code.clone()),
        SHEET_COMPONENTS,
    )?;
    unique(
        data.configurations.iter().map(|item| item.code.clone()),
        SHEET_CONFIGURATIONS,
    )?;

    let range = workbook
        .worksheet_range(SHEET_CONFIGURATION_LINES)
        .map_err(|_| format!("缺少工作表：{SHEET_CONFIGURATION_LINES}"))?;
    for (index, row) in range.rows().skip(1).enumerate() {
        if row.iter().all(|cell| cell.to_string().trim().is_empty()) {
            continue;
        }
        let row_no = index + 2;
        let configuration_code =
            required(text(row, 0), SHEET_CONFIGURATION_LINES, row_no, "配置编号")?;
        let component_code = required(text(row, 2), SHEET_CONFIGURATION_LINES, row_no, "组件编号")?;
        let quantity = number(row, 3, SHEET_CONFIGURATION_LINES, row_no, None)?;
        let price = number(row, 4, SHEET_CONFIGURATION_LINES, row_no, None)?;
        if quantity <= 0.0 || price < 0.0 {
            return Err(format!(
                "{SHEET_CONFIGURATION_LINES} 第 {row_no} 行：数量必须大于 0 且单价不能为负数"
            ));
        }
        let configuration = data.configurations.iter_mut().find(|item| item.code.eq_ignore_ascii_case(&configuration_code)).ok_or_else(|| format!("{SHEET_CONFIGURATION_LINES} 第 {row_no} 行：配置编号“{configuration_code}”未在自选配置工作表中定义"))?;
        configuration.lines.push(ConfigurationLineImport {
            component_code,
            quantity,
            unit_price_minor: (price * 100.0).round() as i64,
        });
    }
    for item in &data.configurations {
        if item.lines.is_empty() {
            return Err(format!("自选配置“{}”没有配置明细", item.code));
        }
        if item.currency != "CNY" && item.exchange_rate_date.is_empty() {
            return Err(format!("自选配置“{}”使用外币时必须填写汇率日期", item.code));
        }
    }
    Ok(data)
}

pub fn build_configuration_input(
    configuration: ConfigurationImport,
    id: Option<String>,
    components: &[ConfigComponent],
) -> Result<ConfigurableProductInput, String> {
    let lines = configuration
        .lines
        .into_iter()
        .map(|line| {
            let component = components
                .iter()
                .find(|item| item.code.eq_ignore_ascii_case(&line.component_code))
                .ok_or_else(|| {
                    format!(
                        "配置“{}”引用了不存在的组件编号“{}”",
                        configuration.code, line.component_code
                    )
                })?;
            Ok(ConfigurableProductLineInput {
                component_id: component.id.clone(),
                quantity: line.quantity,
                unit_price_minor: line.unit_price_minor,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(ConfigurableProductInput {
        id,
        code: configuration.code,
        name: configuration.name,
        model: configuration.model,
        currency: configuration.currency,
        exchange_rate: configuration.exchange_rate,
        exchange_rate_date: configuration.exchange_rate_date,
        notes: configuration.notes,
        lines,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ConfigurableProductLine;

    #[test]
    fn exports_requested_template_artifact() {
        let Ok(output) = std::env::var("TRADEDESK_TEMPLATE_OUTPUT") else {
            return;
        };
        export_master_workbook(Path::new(&output), true, &[], &[], &[], &[], &[]).unwrap();
    }

    #[test]
    fn exports_and_parses_master_workbook() {
        let path =
            std::env::temp_dir().join(format!("tradedesk-master-{}.xlsx", uuid::Uuid::new_v4()));
        let products = vec![Product {
            id: "p1".into(),
            sku: "SKU-1".into(),
            name_zh: "测试产品".into(),
            name_en: "Test product".into(),
            model: "M1".into(),
            hs_code: "8502".into(),
            unit: "set".into(),
            gross_weight_kg: 12.5,
            active: true,
        }];
        let customers = vec![Customer {
            id: "c1".into(),
            code: "C-1".into(),
            legal_name: "Customer LLC".into(),
            market: "EU".into(),
            currency: "EUR".into(),
            payment_terms: "T/T".into(),
            address: "Address".into(),
            shipping_address: "Shipping".into(),
            billing_address: "Billing".into(),
            purchase_intent: "Generator".into(),
            customer_analysis: "Key account".into(),
            strengths: "Stable".into(),
            weaknesses: "Long cycle".into(),
            contacts: "Buyer".into(),
            active: true,
        }];
        let suppliers = vec![Supplier {
            id: "s1".into(),
            code: "S-1".into(),
            legal_name: "Supplier Ltd.".into(),
            address: "Factory address".into(),
            contacts: "Buyer".into(),
            bank_details: String::new(),
            currency: "CNY".into(),
            payment_terms: "30% deposit".into(),
            lead_time_days: 30,
            on_time_rate: 95,
            qualification_notes: "ISO 9001".into(),
            product_terms: vec![crate::domain::SupplierProductTerm {
                id: "spt1".into(),
                product_id: "p1".into(),
                product_sku: "SKU-1".into(),
                product_name: "Product".into(),
                currency: "CNY".into(),
                unit_price_minor: 88_800,
                moq: 2.0,
                lead_time_days: 25,
            }],
            active: true,
        }];
        let components = vec![ConfigComponent {
            id: "cc1".into(),
            code: "CC-1".into(),
            category: "Engine".into(),
            name: "Engine".into(),
            specification: "K19".into(),
            default_quantity: 1.0,
            unit: "pc".into(),
            unit_price_minor: 123_456,
            currency: "CNY".into(),
            brand: "Brand".into(),
            notes: String::new(),
            active: true,
        }];
        let configurations = vec![ConfigurableProduct {
            id: "cfg1".into(),
            code: "CFG-1".into(),
            name: "Generator set".into(),
            model: "G1".into(),
            currency: "USD".into(),
            exchange_rate: 0.14,
            exchange_rate_date: "2026-08-10".into(),
            notes: String::new(),
            total_amount_minor: 17_284,
            active: true,
            lines: vec![ConfigurableProductLine {
                id: "line1".into(),
                component_id: "cc1".into(),
                category: "Engine".into(),
                name: "Engine".into(),
                specification: "K19".into(),
                quantity: 1.0,
                unit: "pc".into(),
                unit_price_minor: 17_284,
                brand: "Brand".into(),
                notes: String::new(),
                amount_minor: 17_284,
            }],
        }];
        export_master_workbook(
            &path,
            false,
            &products,
            &customers,
            &suppliers,
            &components,
            &configurations,
        )
        .unwrap();
        let bytes = std::fs::read(&path).unwrap();
        let parsed = parse_master_workbook(&bytes).unwrap();
        assert_eq!(parsed.products[0].sku, "SKU-1");
        assert_eq!(parsed.customers[0].shipping_address, "Shipping");
        assert_eq!(parsed.suppliers[0].product_terms.len(), 1);
        assert_eq!(parsed.suppliers[0].product_terms[0].product_sku, "SKU-1");
        assert_eq!(parsed.components[0].unit_price_minor, 123_456);
        assert_eq!(parsed.configurations[0].lines[0].component_code, "CC-1");
        let _ = std::fs::remove_file(path);
    }
}
