#let data = json("document.json")
#let payload = data.payload
#let branding = data.branding
#import "helpers.typ": fit-line
#let nowrap(body) = fit-line(body, text-size: 6.5pt)

#set document(title: "Shipper Instruction " + data.number, author: payload.seller)
#set page(
  paper: "a4", flipped: true, margin: (x: 10mm, y: 10mm),
  footer: context align(center, text(size: 7pt, fill: luma(90))[
    #branding.companyName - #data.number - V#data.version - Page #counter(page).display("1 / 1")
  ]),
)
#set text(font: ("Arial", "Microsoft YaHei"), size: 8pt)

#grid(
  columns: (42mm, 1fr, 42mm), align: (left, center, right),
  [#if branding.logoPath != "" [#image(branding.logoPath, width: 36mm, height: 12mm, fit: "contain")]],
  [#text(size: 17pt, weight: "bold")[SHIPPER'S INSTRUCTION] #h(6pt) #text(size: 8pt, fill: luma(90))[发货人委托书]], [],
)
#if data.status == "draft" [#align(right, text(size: 9pt, weight: "bold", fill: rgb("b42318"))[DRAFT / REFERENCE ONLY])]
#v(5pt)
#table(
  columns: (28mm, 1fr, 28mm, 1fr), inset: 4pt, stroke: .45pt + luma(155),
  fill: (column, _) => if calc.even(column) { luma(245) },
  [#nowrap[*Instruction No.*]], [#nowrap[#data.number]], [#nowrap[*Issue Date*]], [#nowrap[#data.issueDate]],
  [#nowrap[*Shipper*]], [#nowrap[#payload.seller]], [#nowrap[*Consignee*]], [#nowrap[#payload.buyer]],
  [#nowrap[*Transport Mode*]], [#nowrap[#payload.transportMode]], [#nowrap[*Vessel / Voyage*]], [#nowrap[#payload.vesselVoyage]],
  [#nowrap[*Port of Loading*]], [#nowrap[#payload.portOfLoading]], [#nowrap[*Port of Discharge*]], [#nowrap[#payload.portOfDischarge]],
  [#nowrap[*Booking Reference*]], [#nowrap[#payload.bookingReference]], [#nowrap[*Freight Terms*]], [#nowrap[#payload.freightTerms]],
  [#nowrap[*B/L Type*]], [#nowrap[#payload.billOfLadingType]], [#nowrap[*Shipment Date*]], [#nowrap[#payload.shipmentDate]],
)
#v(6pt)
#table(
  columns: (8mm, 27mm, 1fr, 19mm, 21mm, 23mm, 23mm, 20mm), inset: 3pt,
  stroke: .45pt + luma(155), fill: (_, row) => if row == 0 { luma(235) },
  table.header([#nowrap[*NO.*]], [#nowrap[*SKU*]], [#nowrap[*CARGO DESCRIPTION*]], [#nowrap[*QTY*]], [#nowrap[*PACKAGES*]], [#nowrap[*NET KG*]], [#nowrap[*GROSS KG*]], [#nowrap[*CBM*]]),
  ..payload.lines.enumerate().map(((index, line)) => (
    [#nowrap[#(index + 1)]], [#nowrap[#line.sku]], [#nowrap[#line.description #if line.model != "" [ #h(3pt) #line.model]]],
    [#nowrap[#line.quantity #line.unit]], [#nowrap[#line.packages #line.packageType]], [#nowrap[#line.netWeightKg]], [#nowrap[#line.grossWeightKg]], [#nowrap[#line.cbm]],
  )).flatten(),
)
#v(6pt)
#block(width: 100%, inset: 5pt, stroke: .45pt + luma(170))[*SPECIAL INSTRUCTIONS* #h(5pt) #payload.notes]
#v(10pt)
#align(right)[#box(width: 48mm)[#if branding.signaturePath != "" [#align(center)[#image(branding.signaturePath, width: if branding.signingAssetKind == "stamp" { 22mm } else { 34mm }, height: if branding.signingAssetKind == "stamp" { 22mm } else { 12mm }, fit: "contain")]] #line(length: 100%)\ Authorized Signature / Stamp]]
