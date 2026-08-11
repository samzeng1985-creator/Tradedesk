#let data = json("purchase-order.json")
#let order = data.purchaseOrder
#let branding = data.branding
#import "helpers.typ": fit-line

#let money(value) = {
  let whole = calc.floor(value / 100)
  let cents = calc.abs(value - whole * 100)
  str(whole) + "." + (if cents < 10 { "0" } else { "" }) + str(cents)
}
#let nowrap(body, size: 6.4pt) = fit-line(body, text-size: size)

#set document(title: "Purchase Order " + order.number, author: branding.companyName)
#set page(
  paper: "a4",
  margin: (x: 13mm, y: 12mm),
  footer: context align(center, text(size: 7pt, fill: luma(95))[
    #branding.companyName - #order.number - #counter(page).display("1 / 1")
  ]),
)
#set text(font: ("Arial", "Microsoft YaHei"), size: 8pt)
#set par(leading: 0.6em)

#grid(
  columns: (42mm, 1fr, 42mm),
  align: (left, center, right),
  [#if branding.logoPath != "" [#image(branding.logoPath, width: 36mm, height: 13mm, fit: "contain")]],
  [#text(size: 18pt, weight: "bold")[采购订单] #linebreak() #text(size: 11pt, weight: "bold")[PURCHASE ORDER]],
  [#align(right)[*#branding.companyName*]],
)

#v(8pt)
#table(
  columns: (28mm, 1fr, 28mm, 1fr),
  inset: (x: 4pt, y: 4pt),
  stroke: .45pt + luma(150),
  fill: (column, _) => if calc.even(column) { luma(240) },
  [#nowrap[*采购单号 / PO No.*]], [#nowrap[#order.number]],
  [#nowrap[*状态 / Status*]], [#nowrap[#data.statusLabel]],
  [#nowrap[*供应商 / Supplier*]], [#nowrap[#order.supplierName]],
  [#nowrap[*来源业务单 / Sales Ref.*]], [#nowrap[#order.businessCaseNumber]],
  [#nowrap[*预计交货 / Delivery*]], [#nowrap[#order.expectedDate]],
  [#nowrap[*币种 / Currency*]], [#nowrap[#order.currency]],
)

#if order.exchangeRateDate != "" [
  #v(4pt)
  #align(right)[#text(size: 7.5pt)[*汇率快照 / FX Snapshot:* 1 #data.salesCurrency = #order.exchangeRate #order.currency · #order.exchangeRateDate]]
]

#v(7pt)
#table(
  columns: (8mm, 26mm, 1fr, 18mm, 17mm, 28mm, 30mm),
  inset: (x: 3pt, y: 4pt),
  align: (center, left, left, right, center, right, right),
  stroke: .45pt + luma(150),
  fill: (_, row) => if row == 0 { luma(230) },
  table.header(
    [#nowrap[*序号 / No.*]],
    [#nowrap[*产品编码 / SKU*]],
    [#nowrap[*品名 / Description*]],
    [#nowrap[*数量 / Qty*]],
    [#nowrap[*单位 / Unit*]],
    [#nowrap[*单价 / Unit Price*]],
    [#nowrap[*金额 / Amount*]],
  ),
  ..order.lines.enumerate().map(((index, line)) => (
    [#nowrap[#(index + 1)]],
    [#nowrap[#line.sku]],
    [#nowrap[#if line.nameZh != "" { line.nameZh } else { line.nameEn }]],
    [#nowrap[#line.quantity]],
    [#nowrap[#line.unit]],
    [#nowrap[#order.currency #money(line.unitCostMinor)]],
    [#nowrap[#order.currency #money(line.amountMinor)]],
  )).flatten(),
  table.cell(colspan: 6, align: right, fill: luma(238))[*采购总额 / TOTAL*],
  table.cell(align: right, fill: luma(238))[*#order.currency #money(order.totalAmountMinor)*],
)

#if order.notes != "" [
  #v(7pt)
  #block(width: 100%, inset: 5pt, stroke: .45pt + luma(170))[
    *备注 / Notes*\
    #order.notes
  ]
]

#v(14pt)
#grid(
  columns: (1fr, 50mm),
  gutter: 18mm,
  [#text(fill: luma(90))[本采购单以保存的采购数据及价格快照为准。]],
  [#if branding.signaturePath != "" [#align(center)[#image(branding.signaturePath, width: if branding.signingAssetKind == "stamp" { 24mm } else { 38mm }, height: if branding.signingAssetKind == "stamp" { 24mm } else { 14mm }, fit: "contain")]] #line(length: 100%)\ #align(center)[采购方签章 / Authorized by]],
)
