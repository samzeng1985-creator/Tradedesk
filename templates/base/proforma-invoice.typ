#let data = json("document.json")
#let payload = data.payload
#let money(value) = {
  let whole = calc.floor(value / 100)
  let cents = calc.abs(value - whole * 100)
  str(whole) + "." + (if cents < 10 { "0" } else { "" }) + str(cents)
}
#let subtotal = payload.lines.fold(0, (sum, line) => sum + line.amountMinor)
#let total = subtotal - payload.discountMinor

#set document(title: "Proforma Invoice " + data.number, author: payload.seller)
#set page(
  paper: "a4",
  margin: (x: 14mm, y: 13mm),
  footer: context align(center, text(size: 7pt, fill: luma(90))[
    TradeDesk - #data.number - V#data.version - Page #counter(page).display("1 / 1")
  ]),
)
#set text(font: ("Arial", "Microsoft YaHei"), size: 8.5pt)
#set par(leading: 0.55em)

#align(center)[
  #text(size: 18pt, weight: "bold")[PROFORMA INVOICE]
  #linebreak()
  #text(size: 8pt, fill: luma(90))[形式发票 / Proforma Invoice]
]
#if data.status == "draft" [
  #align(right, text(size: 9pt, weight: "bold", fill: rgb("b42318"))[DRAFT / 草稿])
]

#v(5pt)
#table(
  columns: (1fr, 1fr),
  inset: 5pt,
  stroke: .55pt + luma(145),
  [*SELLER / EXPORTER*\ #payload.seller\ #text(fill: luma(70))[#payload.sellerAddress]],
  [*BUYER / IMPORTER*\ #payload.buyer\ #text(fill: luma(70))[#payload.buyerAddress]],
)
#v(5pt)
#table(
  columns: (23mm, 1fr, 23mm, 1fr),
  inset: 4pt,
  stroke: .45pt + luma(160),
  fill: (column, _) => if calc.even(column) { luma(245) },
  [*PI No.*], [#data.number], [*Issue Date*], [#data.issueDate],
  [*Reference*], [#data.businessCaseNumber], [*Customer PO*], [#payload.poReference],
  [*Currency*], [#data.currency], [*Incoterm*], [#payload.incoterm],
  [*Payment*], [#payload.paymentTerms], [*Shipment Date*], [#payload.shipmentDate],
  [*Origin*], [#payload.originCountry], [*Destination*], [#payload.destinationCountry],
  [*Port of Loading*], [#payload.portOfLoading], [*Port of Discharge*], [#payload.portOfDischarge],
)
#v(6pt)
#table(
  columns: (8mm, 25mm, 1fr, 20mm, 14mm, 25mm, 27mm),
  inset: (x: 3pt, y: 4pt),
  align: (center, left, left, right, center, right, right),
  stroke: .45pt + luma(155),
  fill: (_, row) => if row == 0 { luma(235) },
  table.header(
    [*NO.*], [*SKU*], [*DESCRIPTION / HS CODE*], [*QTY*], [*UNIT*], [*UNIT PRICE*], [*AMOUNT*],
  ),
  ..payload.lines.enumerate().map(((index, line)) => (
    [#(index + 1)],
    [#line.sku],
    [#line.description #if line.model != "" [\ Model: #line.model] #if line.hsCode != "" [\ HS: #line.hsCode]],
    [#line.quantity],
    [#line.unit],
    [#money(line.unitPriceMinor)],
    [#money(line.amountMinor)],
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

#v(12pt)
#grid(
  columns: (1fr, 45mm),
  gutter: 15mm,
  [#text(fill: luma(80))[Proforma invoice - not a tax or customs invoice.]],
  [#line(length: 100%)\ Authorized Signature],
)
