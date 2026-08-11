#let data = json("document.json")
#let payload = data.payload
#let branding = data.branding
#import "helpers.typ": fit-line
#let money(value) = {
  let whole = calc.floor(value / 100)
  let cents = calc.abs(value - whole * 100)
  str(whole) + "." + (if cents < 10 { "0" } else { "" }) + str(cents)
}
#let nowrap(body) = fit-line(body, text-size: 6.2pt)
#let cargo = payload.lines.fold(0, (sum, line) => sum + line.amountMinor) - payload.discountMinor

#set document(title: "Cargo Insurance Application " + data.number, author: payload.seller)
#set page(
  paper: "a4", flipped: true, margin: (x: 9mm, y: 9mm),
  footer: context align(center, text(size: 7pt, fill: luma(90))[
    APPLICATION / REFERENCE ONLY - NOT AN INSURER-ISSUED POLICY - #branding.companyName - #data.number - Page #counter(page).display("1 / 1")
  ]),
)
#set text(font: ("Arial", "Microsoft YaHei"), size: 7.5pt)

#grid(
  columns: (42mm, 1fr, 42mm), align: (left, center, right),
  [#if branding.logoPath != "" [#image(branding.logoPath, width: 36mm, height: 12mm, fit: "contain")]],
  [#text(size: 16pt, weight: "bold")[CARGO INSURANCE APPLICATION] #h(5pt) #text(size: 8pt, fill: luma(90))[货物运输保险申请稿]], [],
)
#align(right, text(size: 8pt, weight: "bold", fill: rgb("b42318"))[APPLICATION / REFERENCE ONLY / 正式保单由保险公司签发])
#v(4pt)
#table(
  columns: (30mm, 1fr, 30mm, 1fr), inset: 3.5pt, stroke: .45pt + luma(155),
  fill: (column, _) => if calc.even(column) { luma(245) },
  [#nowrap[*Application No.*]], [#nowrap[#data.number]], [#nowrap[*Policy No.*]], [#nowrap[#payload.policyNumber]],
  [#nowrap[*Applicant / Assured*]], [#nowrap[#payload.seller]], [#nowrap[*Beneficiary*]], [#nowrap[#payload.buyer]],
  [#nowrap[*Insurance Company*]], [#nowrap[#payload.insuranceCompany]], [#nowrap[*Claims Payable At*]], [#nowrap[#payload.claimsPayableAt]],
  [#nowrap[*Transport Mode*]], [#nowrap[#payload.transportMode]], [#nowrap[*Vessel / Voyage*]], [#nowrap[#payload.vesselVoyage]],
  [#nowrap[*From*]], [#nowrap[#payload.portOfLoading]], [#nowrap[*To*]], [#nowrap[#payload.portOfDischarge]],
  [#nowrap[*Shipment Date*]], [#nowrap[#payload.shipmentDate]], [#nowrap[*B/L No.*]], [#nowrap[#payload.billOfLadingNumber]],
  [#nowrap[*Coverage*]], [#nowrap[#payload.insuranceCoverage]], [#nowrap[*Incoterm*]], [#nowrap[#payload.incoterm]],
  [#nowrap[*Cargo Value*]], [#nowrap[#data.currency: #money(cargo)]], [#nowrap[*Insurance Mark-up*]], [#nowrap[#payload.insuranceMarkupPercent%]],
  [#nowrap[*Insured Value*]], [#nowrap[#data.currency: #money(payload.insuredValueMinor)]], [#nowrap[*Premium Rate / Premium*]], [#nowrap[#payload.premiumRatePercent% / #data.currency #money(payload.premiumMinor)]],
)
#v(5pt)
#table(
  columns: (8mm, 25mm, 25mm, 1fr, 24mm, 18mm, 24mm, 26mm), inset: 2.8pt,
  stroke: .4pt + luma(155), fill: (_, row) => if row == 0 { luma(232) },
  table.header([#nowrap[*NO.*]], [#nowrap[*HS CODE*]], [#nowrap[*SKU / MODEL*]], [#nowrap[*DESCRIPTION OF GOODS*]], [#nowrap[*QTY*]], [#nowrap[*UNIT*]], [#nowrap[*UNIT PRICE*]], [#nowrap[*AMOUNT*]]),
  ..payload.lines.enumerate().map(((index, line)) => (
    [#nowrap[#(index + 1)]], [#nowrap[#line.hsCode]], [#nowrap[#line.sku #if line.model != "" [ #h(3pt) #line.model]]], [#nowrap[#line.description]],
    [#nowrap[#line.quantity]], [#nowrap[#line.unit]], [#nowrap[#money(line.unitPriceMinor)]], [#nowrap[#money(line.amountMinor)]],
  )).flatten(),
  table.cell(colspan: 7, align: right, fill: luma(238))[*TOTAL #data.currency*], table.cell(align: right, fill: luma(238))[*#money(cargo)*],
)
#v(5pt)
#block(width: 100%, inset: 5pt, stroke: .45pt + luma(170))[*SPECIAL CONDITIONS / NOTES* #h(5pt) #payload.notes]
#v(8pt)
#align(right)[#box(width: 48mm)[#if branding.signaturePath != "" [#align(center)[#image(branding.signaturePath, width: if branding.signingAssetKind == "stamp" { 22mm } else { 34mm }, height: if branding.signingAssetKind == "stamp" { 22mm } else { 12mm }, fit: "contain")]] #line(length: 100%)\ Authorized Signature / Stamp]]
