#let data = json("document.json")
#let payload = data.payload
#let branding = data.branding
#import "helpers.typ": fit-line
#let nowrap(body) = fit-line(body, text-size: 6.2pt)
#let packages = payload.lines.fold(0, (sum, line) => sum + line.packages)
#let gross = payload.lines.fold(0, (sum, line) => sum + line.grossWeightKg)

#set document(title: "Certificate of Origin Application " + data.number, author: payload.seller)
#set page(
  paper: "a4", flipped: true, margin: (x: 9mm, y: 9mm),
  footer: context align(center, text(size: 7pt, fill: luma(90))[
    APPLICATION / REFERENCE ONLY - NOT AN AUTHORITY-ISSUED CERTIFICATE - #branding.companyName - #data.number - Page #counter(page).display("1 / 1")
  ]),
)
#set text(font: ("Arial", "Microsoft YaHei"), size: 7.5pt)

#grid(
  columns: (42mm, 1fr, 42mm), align: (left, center, right),
  [#if branding.logoPath != "" [#image(branding.logoPath, width: 36mm, height: 12mm, fit: "contain")]],
  [#text(size: 16pt, weight: "bold")[CERTIFICATE OF ORIGIN APPLICATION] #h(5pt) #text(size: 8pt, fill: luma(90))[原产地证明申请稿]], [],
)
#align(right, text(size: 8pt, weight: "bold", fill: rgb("b42318"))[APPLICATION / REFERENCE ONLY / 正式证书由签证机构签发])
#v(4pt)
#table(
  columns: (30mm, 1fr, 30mm, 1fr), inset: 3.5pt, stroke: .45pt + luma(155),
  fill: (column, _) => if calc.even(column) { luma(245) },
  [#nowrap[*Application No.*]], [#nowrap[#data.number]], [#nowrap[*Certificate No.*]], [#nowrap[#payload.certificateNumber]],
  [#nowrap[*Exporter*]], [#nowrap[#payload.seller]], [#nowrap[*Consignee*]], [#nowrap[#payload.buyer]],
  [#nowrap[*Certificate Type*]], [#nowrap[#payload.certificateType]], [#nowrap[*Issuing Authority*]], [#nowrap[#payload.certificationAuthority]],
  [#nowrap[*Country of Origin*]], [#nowrap[#payload.originCountry]], [#nowrap[*Destination*]], [#nowrap[#payload.destinationCountry]],
  [#nowrap[*Transport Mode*]], [#nowrap[#payload.transportMode]], [#nowrap[*Vessel / Voyage*]], [#nowrap[#payload.vesselVoyage]],
  [#nowrap[*Port of Loading*]], [#nowrap[#payload.portOfLoading]], [#nowrap[*Port of Discharge*]], [#nowrap[#payload.portOfDischarge]],
  [#nowrap[*Shipment Date*]], [#nowrap[#payload.shipmentDate]], [#nowrap[*Trade Reference*]], [#nowrap[#data.businessCaseNumber]],
)
#v(5pt)
#table(
  columns: (8mm, 28mm, 1fr, 25mm, 21mm, 23mm, 20mm, 23mm), inset: 2.8pt,
  stroke: .4pt + luma(155), fill: (_, row) => if row == 0 { luma(232) },
  table.header([#nowrap[*NO.*]], [#nowrap[*MARKS / SKU*]], [#nowrap[*DESCRIPTION OF GOODS*]], [#nowrap[*HS CODE*]], [#nowrap[*PACKAGES*]], [#nowrap[*QUANTITY*]], [#nowrap[*UNIT*]], [#nowrap[*GROSS KG*]]),
  ..payload.lines.enumerate().map(((index, line)) => (
    [#nowrap[#(index + 1)]], [#nowrap[#line.sku]], [#nowrap[#line.description #if line.model != "" [ #h(3pt) #line.model]]], [#nowrap[#line.hsCode]],
    [#nowrap[#line.packages #line.packageType]], [#nowrap[#line.quantity]], [#nowrap[#line.unit]], [#nowrap[#line.grossWeightKg]],
  )).flatten(),
  table.cell(colspan: 4, align: right, fill: luma(238))[*TOTAL*], [#nowrap[#packages]], table.cell(colspan: 2, fill: luma(238))[], [#nowrap[#gross]],
)
#v(5pt)
#grid(columns: (1fr, 1fr), gutter: 5pt,
  block(width: 100%, inset: 4pt, stroke: .45pt + luma(170))[*SHIPPING MARKS* #h(5pt) #payload.shippingMarks],
  block(width: 100%, inset: 4pt, stroke: .45pt + luma(170))[*DECLARATION / NOTES* #h(5pt) #payload.declaration #h(5pt) #payload.notes],
)
#v(8pt)
#align(right)[#box(width: 48mm)[#if branding.signaturePath != "" [#align(center)[#image(branding.signaturePath, width: if branding.signingAssetKind == "stamp" { 22mm } else { 34mm }, height: if branding.signingAssetKind == "stamp" { 22mm } else { 12mm }, fit: "contain")]] #line(length: 100%)\ Applicant's Authorized Signature / Stamp]]
