#let data = json("document.json")
#let payload = data.payload
#let branding = data.branding
#import "helpers.typ": fit-line
#let money(value) = {
  let whole = calc.floor(value / 100)
  let cents = calc.abs(value - whole * 100)
  str(whole) + "." + (if cents < 10 { "0" } else { "" }) + str(cents)
}
#let subtotal = payload.lines.fold(0, (sum, line) => sum + line.amountMinor)
#let total = subtotal - payload.discountMinor
#let nowrap(body) = fit-line(body, text-size: 6.5pt)

#set document(title: "Commercial Invoice " + data.number, author: payload.seller)
#set page(
  paper: "a4",
  flipped: true,
  margin: (x: 10mm, y: 10mm),
  footer: context align(center, text(size: 7pt, fill: luma(90))[
    #branding.companyName - #data.number - V#data.version - Page #counter(page).display("1 / 1")
  ]),
)
#set text(font: ("Arial", "Microsoft YaHei"), size: 8.5pt)
#set par(leading: 0.55em)

#grid(
  columns: (38mm, 1fr, 38mm),
  align: (left, center, right),
  [#if branding.logoPath != "" [#image(branding.logoPath, width: 34mm, height: 13mm, fit: "contain")]],
  [#text(size: 18pt, weight: "bold")[COMMERCIAL INVOICE] #h(6pt) #text(size: 8pt, fill: luma(90))[商业发票]],
  [],
)
#if data.status == "draft" [
  #align(right, text(size: 9pt, weight: "bold", fill: rgb("b42318"))[DRAFT / 草稿])
]

#v(5pt)
#table(
  columns: (1fr, 1fr),
  inset: 5pt,
  stroke: .55pt + luma(145),
  [#nowrap[*SELLER / EXPORTER* #h(4pt) #payload.seller #h(4pt) #text(fill: luma(70))[#payload.sellerAddress]]],
  [#nowrap[*BUYER / CONSIGNEE* #h(4pt) #payload.buyer #h(4pt) #text(fill: luma(70))[#payload.buyerAddress]]],
)
#v(5pt)
#table(
  columns: (25mm, 1fr, 25mm, 1fr),
  inset: 4pt,
  stroke: .45pt + luma(160),
  fill: (column, _) => if calc.even(column) { luma(245) },
  [#nowrap[*Invoice No.*]], [#nowrap[#data.number]], [#nowrap[*Issue Date*]], [#nowrap[#data.issueDate]],
  [#nowrap[*Order No.*]], [#nowrap[#data.businessCaseNumber]], [#nowrap[*PO Reference*]], [#nowrap[#payload.poReference]],
  [#nowrap[*Currency*]], [#nowrap[#data.currency]], [#nowrap[*Incoterm*]], [#nowrap[#payload.incoterm]],
  [#nowrap[*Payment*]], [#nowrap[#payload.paymentTerms]], [#nowrap[*Shipment Date*]], [#nowrap[#payload.shipmentDate]],
  [#nowrap[*Origin*]], [#nowrap[#payload.originCountry]], [#nowrap[*Destination*]], [#nowrap[#payload.destinationCountry]],
  [#nowrap[*Port of Loading*]], [#nowrap[#payload.portOfLoading]], [#nowrap[*Port of Discharge*]], [#nowrap[#payload.portOfDischarge]],
)
#v(6pt)
#table(
  columns: (8mm, 29mm, 1fr, 20mm, 16mm, 29mm, 32mm),
  inset: (x: 3pt, y: 4pt),
  align: (center, left, left, right, center, right, right),
  stroke: .45pt + luma(155),
  fill: (_, row) => if row == 0 { luma(235) },
  table.header(
    [#nowrap[*NO.*]], [#nowrap[*SKU*]], [#nowrap[*DESCRIPTION / MODEL / HS CODE*]], [#nowrap[*QTY*]], [#nowrap[*UNIT*]], [#nowrap[*UNIT PRICE*]], [#nowrap[*AMOUNT*]],
  ),
  ..payload.lines.enumerate().map(((index, line)) => (
    [#nowrap[#(index + 1)]],
    [#nowrap[#line.sku]],
    [#nowrap[#line.description #if line.model != "" [ #h(3pt) Model: #line.model] #if line.hsCode != "" [ #h(3pt) HS: #line.hsCode]]],
    [#nowrap[#line.quantity]],
    [#nowrap[#line.unit]],
    [#nowrap[#money(line.unitPriceMinor)]],
    [#nowrap[#money(line.amountMinor)]],
  )).flatten(),
  table.cell(colspan: 6, align: right, fill: luma(245))[*SUBTOTAL #data.currency*],
  table.cell(align: right, fill: luma(245))[*#money(subtotal)*],
  table.cell(colspan: 6, align: right)[DISCOUNT],
  table.cell(align: right)[-#money(payload.discountMinor)],
  table.cell(colspan: 6, align: right, fill: luma(235))[*TOTAL #data.currency*],
  table.cell(align: right, fill: luma(235))[*#money(total)*],
)

#v(7pt)
#if payload.bankDetails != "" [
  #block(width: 100%, inset: 5pt, stroke: .45pt + luma(170))[
    *BANK DETAILS*\ #payload.bankDetails
  ]
]
#if payload.notes != "" [
  #v(4pt)
  *NOTES*\ #payload.notes
]
#if payload.declaration != "" [
  #v(4pt)
  *DECLARATION*\ #payload.declaration
]

#v(12pt)
#grid(
  columns: (1fr, 45mm),
  gutter: 15mm,
  [#text(fill: luma(80))[Generated from an encrypted local snapshot.\ #branding.companyName]],
  [#if branding.signaturePath != "" [#align(center)[#image(branding.signaturePath, width: if branding.signingAssetKind == "stamp" { 22mm } else { 34mm }, height: if branding.signingAssetKind == "stamp" { 22mm } else { 13mm }, fit: "contain")]] #line(length: 100%)\ Authorized Signature / Stamp],
)
