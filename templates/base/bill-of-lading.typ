#let data = json("document.json")
#let payload = data.payload
#let branding = data.branding
#let nowrap(body) = box[#text(size: 6.2pt)[#body]]
#let packages = payload.lines.fold(0, (sum, line) => sum + line.packages)
#let net = payload.lines.fold(0, (sum, line) => sum + line.netWeightKg)
#let gross = payload.lines.fold(0, (sum, line) => sum + line.grossWeightKg)
#let cbm = payload.lines.fold(0, (sum, line) => sum + line.cbm)

#set document(title: "Bill of Lading Instructions " + data.number, author: payload.seller)
#set page(
  paper: "a4", flipped: true, margin: (x: 9mm, y: 9mm),
  footer: context align(center, text(size: 7pt, fill: luma(90))[
    SHIPPING INSTRUCTIONS - NOT A CARRIER-ISSUED B/L - #branding.companyName - #data.number - V#data.version - Page #counter(page).display("1 / 1")
  ]),
)
#set text(font: ("Arial", "Microsoft YaHei"), size: 7.5pt)

#grid(
  columns: (42mm, 1fr, 42mm), align: (left, center, right),
  [#if branding.logoPath != "" [#image(branding.logoPath, width: 36mm, height: 12mm, fit: "contain")]],
  [#text(size: 16pt, weight: "bold")[BILL OF LADING INSTRUCTIONS] #h(5pt) #text(size: 8pt, fill: luma(90))[提单补料]], [],
)
#align(right, text(size: 8pt, weight: "bold", fill: rgb("b42318"))[REFERENCE ONLY / 以承运人正式提单为准])
#v(4pt)
#table(
  columns: (28mm, 1fr, 28mm, 1fr), inset: 3.5pt, stroke: .45pt + luma(155),
  fill: (column, _) => if calc.even(column) { luma(245) },
  [#nowrap[*Instruction No.*]], [#nowrap[#data.number]], [#nowrap[*B/L No.*]], [#nowrap[#payload.billOfLadingNumber]],
  [#nowrap[*Shipper*]], [#nowrap[#payload.seller]], [#nowrap[*Consignee*]], [#nowrap[#payload.buyer]],
  [#nowrap[*Notify Party*]], [#nowrap[#payload.notifyParty]], [#nowrap[*Carrier*]], [#nowrap[#payload.carrier]],
  [#nowrap[*Transport Mode*]], [#nowrap[#payload.transportMode]], [#nowrap[*B/L Type*]], [#nowrap[#payload.billOfLadingType]],
  [#nowrap[*Vessel / Voyage*]], [#nowrap[#payload.vesselVoyage]], [#nowrap[*Booking Reference*]], [#nowrap[#payload.bookingReference]],
  [#nowrap[*Place of Receipt*]], [#nowrap[#payload.placeOfReceipt]], [#nowrap[*Port of Loading*]], [#nowrap[#payload.portOfLoading]],
  [#nowrap[*Port of Discharge*]], [#nowrap[#payload.portOfDischarge]], [#nowrap[*Place of Delivery*]], [#nowrap[#payload.placeOfDelivery]],
  [#nowrap[*Container No.*]], [#nowrap[#payload.containerNumbers]], [#nowrap[*Seal No.*]], [#nowrap[#payload.sealNumbers]],
  [#nowrap[*Freight Terms*]], [#nowrap[#payload.freightTerms]], [#nowrap[*Shipment Date*]], [#nowrap[#payload.shipmentDate]],
)
#v(5pt)
#table(
  columns: (8mm, 25mm, 1fr, 18mm, 21mm, 21mm, 23mm, 19mm), inset: 2.8pt,
  stroke: .4pt + luma(155), fill: (_, row) => if row == 0 { luma(232) },
  table.header([#nowrap[*NO.*]], [#nowrap[*MARKS / SKU*]], [#nowrap[*DESCRIPTION OF GOODS*]], [#nowrap[*QTY*]], [#nowrap[*PACKAGES*]], [#nowrap[*NET KG*]], [#nowrap[*GROSS KG*]], [#nowrap[*CBM*]]),
  ..payload.lines.enumerate().map(((index, line)) => (
    [#nowrap[#(index + 1)]], [#nowrap[#line.sku]], [#nowrap[#line.description #if line.model != "" [ #h(3pt) #line.model]]],
    [#nowrap[#line.quantity #line.unit]], [#nowrap[#line.packages #line.packageType]], [#nowrap[#line.netWeightKg]], [#nowrap[#line.grossWeightKg]], [#nowrap[#line.cbm]],
  )).flatten(),
  table.cell(colspan: 4, align: right, fill: luma(238))[*TOTAL*], [#nowrap[#packages]], [#nowrap[#net]], [#nowrap[#gross]], [#nowrap[#cbm]],
)
#v(5pt)
#grid(columns: (1fr, 1fr), gutter: 5pt,
  block(width: 100%, inset: 4pt, stroke: .45pt + luma(170))[*SHIPPING MARKS* #h(5pt) #payload.shippingMarks],
  block(width: 100%, inset: 4pt, stroke: .45pt + luma(170))[*SPECIAL INSTRUCTIONS* #h(5pt) #payload.notes],
)
#v(8pt)
#align(right)[#box(width: 48mm)[#if branding.signaturePath != "" [#align(center)[#image(branding.signaturePath, width: if branding.signingAssetKind == "stamp" { 22mm } else { 34mm }, height: if branding.signingAssetKind == "stamp" { 22mm } else { 12mm }, fit: "contain")]] #line(length: 100%)\ Authorized Signature / Stamp]]
