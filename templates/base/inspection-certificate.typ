#let data = json("document.json")
#let payload = data.payload
#let branding = data.branding
#let nowrap(body) = box[#text(size: 6.2pt)[#body]]
#let packages = payload.lines.fold(0, (sum, line) => sum + line.packages)
#let net = payload.lines.fold(0, (sum, line) => sum + line.netWeightKg)
#let gross = payload.lines.fold(0, (sum, line) => sum + line.grossWeightKg)

#set document(title: "Inspection Certificate Application " + data.number, author: payload.seller)
#set page(
  paper: "a4", flipped: true, margin: (x: 9mm, y: 9mm),
  footer: context align(center, text(size: 7pt, fill: luma(90))[
    APPLICATION / REFERENCE ONLY - NOT AN INSPECTION-BODY REPORT - #branding.companyName - #data.number - Page #counter(page).display("1 / 1")
  ]),
)
#set text(font: ("Arial", "Microsoft YaHei"), size: 7.5pt)

#grid(
  columns: (42mm, 1fr, 42mm), align: (left, center, right),
  [#if branding.logoPath != "" [#image(branding.logoPath, width: 36mm, height: 12mm, fit: "contain")]],
  [#text(size: 16pt, weight: "bold")[INSPECTION CERTIFICATE APPLICATION] #h(5pt) #text(size: 8pt, fill: luma(90))[检验证书申请/参考稿]], [],
)
#align(right, text(size: 8pt, weight: "bold", fill: rgb("b42318"))[APPLICATION / REFERENCE ONLY / 正式报告由检验机构签发])
#v(4pt)
#table(
  columns: (30mm, 1fr, 30mm, 1fr), inset: 3.5pt, stroke: .45pt + luma(155),
  fill: (column, _) => if calc.even(column) { luma(245) },
  [#nowrap[*Application No.*]], [#nowrap[#data.number]], [#nowrap[*Certificate No.*]], [#nowrap[#payload.certificateNumber]],
  [#nowrap[*Applicant*]], [#nowrap[#payload.seller]], [#nowrap[*Consignee*]], [#nowrap[#payload.buyer]],
  [#nowrap[*Manufacturer*]], [#nowrap[#payload.manufacturer]], [#nowrap[*Inspection Body*]], [#nowrap[#payload.certificationAuthority]],
  [#nowrap[*Batch / Lot No.*]], [#nowrap[#payload.batchNumber]], [#nowrap[*Inspection Date*]], [#nowrap[#payload.inspectionDate]],
  [#nowrap[*Inspection Place*]], [#nowrap[#payload.inspectionPlace]], [#nowrap[*Inspection Standard*]], [#nowrap[#payload.inspectionStandard]],
  [#nowrap[*Transport / Vessel*]], [#nowrap[#payload.transportMode / #payload.vesselVoyage]], [#nowrap[*Route*]], [#nowrap[#payload.portOfLoading to #payload.portOfDischarge]],
)
#v(5pt)
#table(
  columns: (8mm, 25mm, 1fr, 24mm, 20mm, 21mm, 22mm, 23mm), inset: 2.8pt,
  stroke: .4pt + luma(155), fill: (_, row) => if row == 0 { luma(232) },
  table.header([#nowrap[*NO.*]], [#nowrap[*SKU / MODEL*]], [#nowrap[*DESCRIPTION OF GOODS*]], [#nowrap[*HS CODE*]], [#nowrap[*QUANTITY*]], [#nowrap[*PACKAGES*]], [#nowrap[*NET KG*]], [#nowrap[*GROSS KG*]]),
  ..payload.lines.enumerate().map(((index, line)) => (
    [#nowrap[#(index + 1)]], [#nowrap[#line.sku #if line.model != "" [ #h(3pt) #line.model]]], [#nowrap[#line.description]], [#nowrap[#line.hsCode]],
    [#nowrap[#line.quantity #line.unit]], [#nowrap[#line.packages #line.packageType]], [#nowrap[#line.netWeightKg]], [#nowrap[#line.grossWeightKg]],
  )).flatten(),
  table.cell(colspan: 5, align: right, fill: luma(238))[*TOTAL*], [#nowrap[#packages]], [#nowrap[#net]], [#nowrap[#gross]],
)
#v(5pt)
#block(width: 100%, inset: 5pt, stroke: .55pt + luma(140))[*INSPECTION RESULT* #h(6pt) #payload.inspectionResult]
#if payload.notes != "" [#v(4pt) *NOTES* #h(4pt) #payload.notes]
#v(8pt)
#align(right)[#box(width: 48mm)[#if branding.signaturePath != "" [#align(center)[#image(branding.signaturePath, width: if branding.signingAssetKind == "stamp" { 22mm } else { 34mm }, height: if branding.signingAssetKind == "stamp" { 22mm } else { 12mm }, fit: "contain")]] #line(length: 100%)\ Applicant's Authorized Signature / Stamp]]
