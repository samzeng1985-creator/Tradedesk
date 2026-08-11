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

#set document(title: "Trade Contract " + data.number, author: payload.seller)
#set page(
  paper: "a4",
  flipped: true,
  margin: (x: 10mm, y: 10mm),
  footer: context align(center, text(size: 7pt, fill: luma(90))[
    #branding.companyName - #data.number - V#data.version - Page #counter(page).display("1 / 1")
  ]),
)
#set text(font: ("Arial", "Microsoft YaHei"), size: 8.5pt)
#set par(justify: false, leading: 0.65em)

#grid(
  columns: (38mm, 1fr, 38mm),
  align: (left, center, right),
  [#if branding.logoPath != "" [#image(branding.logoPath, width: 34mm, height: 13mm, fit: "contain")]],
  [#text(size: 18pt, weight: "bold")[SALES CONTRACT] #h(6pt) #text(size: 8pt, fill: luma(90))[外贸销售合同]],
  [],
)
#if data.status == "draft" [
  #align(right, text(size: 9pt, weight: "bold", fill: rgb("b42318"))[DRAFT / 草稿])
]

#v(5pt)
#table(
  columns: (27mm, 1fr, 27mm, 1fr),
  inset: 4pt,
  stroke: .45pt + luma(155),
  fill: (column, _) => if calc.even(column) { luma(245) },
  [#nowrap[*Contract No.*]], [#nowrap[#data.number]], [#nowrap[*Date*]], [#nowrap[#data.issueDate]],
  [#nowrap[*Seller*]], [#nowrap[#payload.seller]], [#nowrap[*Buyer*]], [#nowrap[#payload.buyer]],
  [#nowrap[*Seller Address*]], [#nowrap[#payload.sellerAddress]], [#nowrap[*Buyer Address*]], [#nowrap[#payload.buyerAddress]],
  [#nowrap[*Currency*]], [#nowrap[#data.currency]], [#nowrap[*Incoterm*]], [#nowrap[#payload.incoterm]],
  [#nowrap[*Payment Terms*]], [#nowrap[#payload.paymentTerms]], [#nowrap[*Shipment Date*]], [#nowrap[#payload.shipmentDate]],
)

#v(7pt)
*1. GOODS AND VALUE*
#v(3pt)
#table(
  columns: (8mm, 29mm, 1fr, 20mm, 16mm, 29mm, 32mm),
  inset: (x: 3pt, y: 4pt),
  align: (center, left, left, right, center, right, right),
  stroke: .45pt + luma(155),
  fill: (_, row) => if row == 0 { luma(235) },
  table.header([#nowrap[*NO.*]], [#nowrap[*SKU*]], [#nowrap[*DESCRIPTION / MODEL*]], [#nowrap[*QTY*]], [#nowrap[*UNIT*]], [#nowrap[*UNIT PRICE*]], [#nowrap[*AMOUNT*]]),
  ..payload.lines.enumerate().map(((index, line)) => (
    [#nowrap[#(index + 1)]], [#nowrap[#line.sku]], [#nowrap[#line.description #if line.model != "" [ #h(3pt) #line.model]]],
    [#nowrap[#line.quantity]], [#nowrap[#line.unit]], [#nowrap[#money(line.unitPriceMinor)]], [#nowrap[#money(line.amountMinor)]],
  )).flatten(),
  table.cell(colspan: 6, align: right, fill: luma(245))[*SUBTOTAL #data.currency*],
  table.cell(align: right, fill: luma(245))[*#money(subtotal)*],
  table.cell(colspan: 6, align: right)[DISCOUNT],
  table.cell(align: right)[-#money(payload.discountMinor)],
  table.cell(colspan: 6, align: right, fill: luma(235))[*TOTAL #data.currency*],
  table.cell(align: right, fill: luma(235))[*#money(total)*],
)

#set par(justify: true, leading: 0.65em)

#v(8pt)
*2. DELIVERY AND PAYMENT*\
Delivery term: #payload.incoterm. Port of loading: #payload.portOfLoading. Port of discharge: #payload.portOfDischarge. Payment: #payload.paymentTerms.

#v(6pt)
*3. GENERAL TERMS*\
#if payload.contractTerms != "" [#payload.contractTerms] else [
  The parties shall perform this contract in good faith. Any amendment must be confirmed in writing by both parties. Force majeure, inspection, claims and dispute resolution shall follow the applicable written agreement between the parties.
]

#if payload.notes != "" [
  #v(6pt)
  *4. SPECIAL NOTES*\
  #payload.notes
]

#v(16pt)
#grid(
  columns: (1fr, 1fr),
  gutter: 22mm,
  [*FOR THE SELLER*\ #payload.seller\ #if branding.signaturePath != "" [#v(4mm) #align(center)[#image(branding.signaturePath, width: if branding.signingAssetKind == "stamp" { 24mm } else { 40mm }, height: if branding.signingAssetKind == "stamp" { 24mm } else { 14mm }, fit: "contain")]] else [#v(18mm)] #line(length: 100%)\ Authorized Signature / Stamp],
  [*FOR THE BUYER*\ #payload.buyer\ #v(18mm) #line(length: 100%)\ Authorized Signature],
)
