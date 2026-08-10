#let data = json("document.json")
#let payload = data.payload
#let branding = data.branding
#let money(value) = {
  let whole = calc.floor(value / 100)
  let cents = calc.abs(value - whole * 100)
  str(whole) + "." + (if cents < 10 { "0" } else { "" }) + str(cents)
}
#let total = payload.lines.fold(0, (sum, line) => sum + line.amountMinor) - payload.discountMinor
#let nowrap(body) = box[#text(size: 6pt)[#body]]

#set document(title: "Customs Declaration Data " + data.number, author: payload.seller)
#set page(
  paper: "a4", flipped: true, margin: (x: 9mm, y: 9mm),
  footer: context align(center, text(size: 7pt, fill: luma(90))[
    REFERENCE DATA - NOT AN OFFICIAL CUSTOMS DECLARATION - #branding.companyName - #data.number - Page #counter(page).display("1 / 1")
  ]),
)
#set text(font: ("Arial", "Microsoft YaHei"), size: 7.5pt)

#grid(
  columns: (42mm, 1fr, 42mm), align: (left, center, right),
  [#if branding.logoPath != "" [#image(branding.logoPath, width: 36mm, height: 12mm, fit: "contain")]],
  [#text(size: 16pt, weight: "bold")[CUSTOMS DECLARATION DATA] #h(5pt) #text(size: 8pt, fill: luma(90))[报关资料参考稿]], [],
)
#align(right, text(size: 8pt, weight: "bold", fill: rgb("b42318"))[REFERENCE ONLY / 仅供报关行制单参考])
#v(4pt)
#table(
  columns: (28mm, 1fr, 28mm, 1fr), inset: 3.5pt, stroke: .45pt + luma(155),
  fill: (column, _) => if calc.even(column) { luma(245) },
  [#nowrap[*Data No.*]], [#nowrap[#data.number]], [#nowrap[*Issue Date*]], [#nowrap[#data.issueDate]],
  [#nowrap[*Exporter*]], [#nowrap[#payload.seller]], [#nowrap[*Consignee*]], [#nowrap[#payload.buyer]],
  [#nowrap[*Trade Reference*]], [#nowrap[#data.businessCaseNumber]], [#nowrap[*Supervision Code*]], [#nowrap[#payload.customsSupervisionCode]],
  [#nowrap[*Transport Mode*]], [#nowrap[#payload.transportMode]], [#nowrap[*Vessel / Voyage*]], [#nowrap[#payload.vesselVoyage]],
  [#nowrap[*Origin*]], [#nowrap[#payload.originCountry]], [#nowrap[*Destination*]], [#nowrap[#payload.destinationCountry]],
  [#nowrap[*Loading Port*]], [#nowrap[#payload.portOfLoading]], [#nowrap[*Discharge Port*]], [#nowrap[#payload.portOfDischarge]],
  [#nowrap[*Incoterm*]], [#nowrap[#payload.incoterm]], [#nowrap[*Currency / Total*]], [#nowrap[#data.currency: #money(total)]],
)
#v(5pt)
#table(
  columns: (7mm, 24mm, 1fr, 25mm, 18mm, 16mm, 25mm, 28mm, 22mm, 22mm), inset: 2.5pt,
  stroke: .4pt + luma(155), fill: (_, row) => if row == 0 { luma(232) },
  table.header([#nowrap[*NO.*]], [#nowrap[*HS CODE*]], [#nowrap[*DESCRIPTION / MODEL*]], [#nowrap[*DECLARATION ELEMENTS*]], [#nowrap[*QTY*]], [#nowrap[*UNIT*]], [#nowrap[*UNIT PRICE*]], [#nowrap[*AMOUNT*]], [#nowrap[*NET KG*]], [#nowrap[*GROSS KG*]]),
  ..payload.lines.enumerate().map(((index, line)) => (
    [#nowrap[#(index + 1)]], [#nowrap[#line.hsCode]], [#nowrap[#line.description #if line.model != "" [ #h(3pt) #line.model]]],
    [#nowrap[#payload.customsDeclarationElements]], [#nowrap[#line.quantity]], [#nowrap[#line.unit]], [#nowrap[#money(line.unitPriceMinor)]], [#nowrap[#money(line.amountMinor)]], [#nowrap[#line.netWeightKg]], [#nowrap[#line.grossWeightKg]],
  )).flatten(),
  table.cell(colspan: 7, align: right, fill: luma(238))[*TOTAL #data.currency*],
  table.cell(align: right, fill: luma(238))[*#money(total)*], table.cell(colspan: 2, fill: luma(238))[],
)
#if payload.notes != "" [#v(5pt) *NOTES* #h(4pt) #payload.notes]
