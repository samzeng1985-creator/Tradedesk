#let data = json("document.json")
#let payload = data.payload
#let branding = data.branding
#let nowrap(body) = box[#text(size: 6.2pt)[#body]]
#let total = payload.lines.fold(0, (sum, line) => sum + line.amountMinor) - payload.discountMinor
#let money(value) = data.currency + " " + str(calc.round(value / 100, digits: 2))

#set document(title: "Beneficiary Certificate " + data.number, author: payload.seller)
#set page(
  paper: "a4", flipped: true, margin: (x: 10mm, y: 10mm),
  footer: context align(center, text(size: 7pt, fill: luma(90))[
    VERIFY AGAINST THE ORIGINAL LETTER OF CREDIT - #branding.companyName - #data.number - Page #counter(page).display("1 / 1")
  ]),
)
#set text(font: ("Arial", "Microsoft YaHei"), size: 7.5pt)

#grid(
  columns: (42mm, 1fr, 42mm), align: (left, center, right),
  [#if branding.logoPath != "" [#image(branding.logoPath, width: 36mm, height: 12mm, fit: "contain")]],
  [#text(size: 17pt, weight: "bold")[BENEFICIARY'S CERTIFICATE] #h(5pt) #text(size: 8pt, fill: luma(90))[受益人证明]], [],
)
#align(right, text(size: 7.5pt, weight: "bold", fill: rgb("b45309"))[L/C TERMS MUST BE VERIFIED AGAINST THE ORIGINAL BANK DOCUMENT])
#v(4pt)
#table(
  columns: (31mm, 1fr, 31mm, 1fr), inset: 3.4pt, stroke: .45pt + luma(155),
  fill: (column, _) => if calc.even(column) { luma(245) },
  [#nowrap[*Certificate No.*]], [#nowrap[#data.number]], [#nowrap[*Certificate Type*]], [#nowrap[#payload.beneficiaryCertificateType]],
  [#nowrap[*Beneficiary*]], [#nowrap[#payload.seller]], [#nowrap[*Applicant / Consignee*]], [#nowrap[#payload.buyer]],
  [#nowrap[*L/C No.*]], [#nowrap[#payload.letterOfCreditNumber]], [#nowrap[*Issuing Bank*]], [#nowrap[#payload.issuingBank]],
  [#nowrap[*L/C Issue Date*]], [#nowrap[#payload.letterOfCreditIssueDate]], [#nowrap[*L/C Expiry Date*]], [#nowrap[#payload.letterOfCreditExpiryDate]],
  [#nowrap[*Presentation Deadline*]], [#nowrap[#payload.presentationDeadline]], [#nowrap[*Trade Reference*]], [#nowrap[#data.businessCaseNumber]],
  [#nowrap[*Transport / Vessel*]], [#nowrap[#payload.transportMode / #payload.vesselVoyage]], [#nowrap[*Route*]], [#nowrap[#payload.portOfLoading to #payload.portOfDischarge]],
  [#nowrap[*Shipment Date*]], [#nowrap[#payload.shipmentDate]], [#nowrap[*Invoice Currency / Value*]], [#nowrap[#money(total)]],
)
#v(5pt)
#table(
  columns: (8mm, 25mm, 1fr, 24mm, 23mm, 25mm, 28mm), inset: 2.8pt,
  stroke: .4pt + luma(155), fill: (_, row) => if row == 0 { luma(232) },
  table.header([#nowrap[*NO.*]], [#nowrap[*SKU / MODEL*]], [#nowrap[*DESCRIPTION OF GOODS*]], [#nowrap[*HS CODE*]], [#nowrap[*QUANTITY*]], [#nowrap[*UNIT PRICE*]], [#nowrap[*AMOUNT*]]),
  ..payload.lines.enumerate().map(((index, line)) => (
    [#nowrap[#(index + 1)]], [#nowrap[#line.sku #if line.model != "" [ #h(3pt) #line.model]]], [#nowrap[#line.description]], [#nowrap[#line.hsCode]],
    [#nowrap[#line.quantity #line.unit]], [#nowrap[#money(line.unitPriceMinor)]], [#nowrap[#money(line.amountMinor)]],
  )).flatten(),
  table.cell(colspan: 6, align: right, fill: luma(238))[*TOTAL*], [#nowrap[#money(total)]],
)
#v(5pt)
#block(width: 100%, inset: 5pt, stroke: .55pt + luma(140))[*BENEFICIARY'S STATEMENT* #h(6pt) #payload.beneficiaryStatement]
#v(4pt)
#grid(columns: (1fr, 1fr), gutter: 5pt,
  block(width: 100%, inset: 4pt, stroke: .45pt + luma(170))[*L/C TERMS CHECKLIST* #h(5pt) #payload.letterOfCreditTerms],
  block(width: 100%, inset: 4pt, stroke: .45pt + luma(170))[*REQUIRED PRESENTATION DOCUMENTS* #h(5pt) #payload.requiredDocuments],
)
#v(8pt)
#align(right)[#box(width: 48mm)[#if branding.signaturePath != "" [#align(center)[#image(branding.signaturePath, width: if branding.signingAssetKind == "stamp" { 22mm } else { 34mm }, height: if branding.signingAssetKind == "stamp" { 22mm } else { 12mm }, fit: "contain")]] #line(length: 100%)\ Beneficiary's Authorized Signature / Stamp]]
