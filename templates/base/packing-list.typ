#let data = json("document.json")
#let payload = data.payload
#let branding = data.branding
#import "helpers.typ": fit-line
#let total-quantity = payload.lines.fold(0, (sum, line) => sum + line.quantity)
#let total-packages = payload.lines.fold(0, (sum, line) => sum + line.packages)
#let total-net = payload.lines.fold(0, (sum, line) => sum + line.netWeightKg)
#let total-gross = payload.lines.fold(0, (sum, line) => sum + line.grossWeightKg)
#let total-cbm = payload.lines.fold(0, (sum, line) => sum + line.cbm)
#let nowrap(body) = fit-line(body, text-size: 6.3pt)

#set document(title: "Detailed Packing List " + data.number, author: payload.seller)
#set page(
  paper: "a4",
  flipped: true,
  margin: (x: 12mm, y: 11mm),
  footer: context align(center, text(size: 7pt, fill: luma(90))[
    #branding.companyName - #data.number - V#data.version - Page #counter(page).display("1 / 1")
  ]),
)
#set text(font: ("Arial", "Microsoft YaHei"), size: 8pt)
#set par(leading: 0.5em)

#grid(
  columns: (42mm, 1fr, 42mm),
  align: (left, center, right),
  [#if branding.logoPath != "" [#image(branding.logoPath, width: 36mm, height: 12mm, fit: "contain")]],
  [#text(size: 17pt, weight: "bold")[DETAILED PACKING LIST] #h(6pt) #text(size: 8pt, fill: luma(90))[详细装箱单]],
  [],
)
#if data.status == "draft" [
  #align(right, text(size: 9pt, weight: "bold", fill: rgb("b42318"))[DRAFT / 草稿])
]

#v(4pt)
#table(
  columns: (1fr, 1fr, 29mm, 1fr, 25mm, 1fr),
  inset: 4pt,
  stroke: .45pt + luma(155),
  [#nowrap[*SHIPPER* #h(3pt) #payload.seller #h(3pt) #text(fill: luma(70))[#payload.sellerAddress]]],
  [#nowrap[*CONSIGNEE* #h(3pt) #payload.buyer #h(3pt) #text(fill: luma(70))[#payload.buyerAddress]]],
  [#nowrap[*Packing List No.*]], [#nowrap[#data.number]], [#nowrap[*Issue Date*]], [#nowrap[#data.issueDate]],
  [#nowrap[*Order No.*]], [#nowrap[#data.businessCaseNumber]], [#nowrap[*Shipment Date*]], [#nowrap[#payload.shipmentDate]],
  [#nowrap[*Port of Loading*]], [#nowrap[#payload.portOfLoading]], [#nowrap[*Port of Discharge*]], [#nowrap[#payload.portOfDischarge]],
)
#v(5pt)
#table(
  columns: (7mm, 26mm, 1fr, 18mm, 15mm, 22mm, 22mm, 24mm, 19mm),
  inset: (x: 3pt, y: 4pt),
  align: (center, left, left, right, center, right, right, right, right),
  stroke: .45pt + luma(155),
  fill: (_, row) => if row == 0 { luma(235) },
  table.header(
    [#nowrap[*NO.*]], [#nowrap[*SKU*]], [#nowrap[*DESCRIPTION / MODEL / PACKAGE*]], [#nowrap[*QTY*]], [#nowrap[*UNIT*]], [#nowrap[*PACKAGES*]], [#nowrap[*NET KG*]], [#nowrap[*GROSS KG*]], [#nowrap[*CBM*]],
  ),
  ..payload.lines.enumerate().map(((index, line)) => (
    [#nowrap[#(index + 1)]],
    [#nowrap[#line.sku]],
    [#nowrap[#line.description #if line.model != "" [ #h(3pt) #line.model] #if line.packageType != "" [ #h(3pt) #line.packageType]]],
    [#nowrap[#line.quantity]],
    [#nowrap[#line.unit]],
    [#nowrap[#line.packages]],
    [#nowrap[#line.netWeightKg]],
    [#nowrap[#line.grossWeightKg]],
    [#nowrap[#line.cbm]],
  )).flatten(),
  table.cell(colspan: 3, align: right, fill: luma(245))[*TOTAL*],
  table.cell(align: right, fill: luma(245))[*#total-quantity*],
  table.cell(fill: luma(245))[],
  table.cell(align: right, fill: luma(245))[*#total-packages*],
  table.cell(align: right, fill: luma(245))[*#total-net*],
  table.cell(align: right, fill: luma(245))[*#total-gross*],
  table.cell(align: right, fill: luma(245))[*#total-cbm*],
)

#v(6pt)
#if payload.notes != "" [*PACKING NOTES*\ #payload.notes]
#v(12pt)
#align(right)[#box(width: 48mm)[#if branding.signaturePath != "" [#align(center)[#image(branding.signaturePath, width: if branding.signingAssetKind == "stamp" { 22mm } else { 34mm }, height: if branding.signingAssetKind == "stamp" { 22mm } else { 12mm }, fit: "contain")]] #line(length: 100%)\ Authorized Signature / Stamp]]
