#let data = json("document.json")
#let payload = data.payload
#let branding = data.branding
#let total-packages = payload.lines.fold(0, (sum, line) => sum + line.packages)
#let nowrap(body) = box[#text(size: 7pt)[#body]]

#set document(title: "Shipping Marks " + data.number, author: payload.seller)
#set page(
  paper: "a4", flipped: true, margin: (x: 12mm, y: 10mm),
  footer: context align(center, text(size: 7pt, fill: luma(90))[
    #branding.companyName - #data.number - V#data.version - Page #counter(page).display("1 / 1")
  ]),
)
#set text(font: ("Arial", "Microsoft YaHei"), size: 8pt)

#grid(
  columns: (42mm, 1fr, 42mm), align: (left, center, right),
  [#if branding.logoPath != "" [#image(branding.logoPath, width: 36mm, height: 12mm, fit: "contain")]],
  [#text(size: 17pt, weight: "bold")[SHIPPING MARKS] #h(6pt) #text(size: 8pt, fill: luma(90))[运输唛头]], [],
)
#if data.status == "draft" [#align(right, text(size: 9pt, weight: "bold", fill: rgb("b42318"))[DRAFT / 草稿])]
#v(5pt)
#table(
  columns: (25mm, 1fr, 25mm, 1fr), inset: 4pt, stroke: .45pt + luma(155),
  fill: (column, _) => if calc.even(column) { luma(245) },
  [#nowrap[*Marks No.*]], [#nowrap[#data.number]], [#nowrap[*Issue Date*]], [#nowrap[#data.issueDate]],
  [#nowrap[*Order No.*]], [#nowrap[#data.businessCaseNumber]], [#nowrap[*Destination*]], [#nowrap[#payload.destinationCountry]],
  [#nowrap[*Consignee*]], [#nowrap[#payload.buyer]], [#nowrap[*Total Packages*]], [#nowrap[#total-packages]],
)
#v(8pt)
#align(center)[
  #block(width: 150mm, inset: 12mm, stroke: 1pt + black)[
    #align(center)[#text(size: 18pt, weight: "bold")[#payload.shippingMarks]]
  ]
]
#v(8pt)
#table(
  columns: (8mm, 28mm, 1fr, 22mm, 24mm, 28mm), inset: 4pt,
  stroke: .45pt + luma(155), fill: (_, row) => if row == 0 { luma(235) },
  table.header([#nowrap[*NO.*]], [#nowrap[*SKU*]], [#nowrap[*DESCRIPTION / MODEL*]], [#nowrap[*QTY*]], [#nowrap[*PACKAGES*]], [#nowrap[*PACKAGE TYPE*]]),
  ..payload.lines.enumerate().map(((index, line)) => (
    [#nowrap[#(index + 1)]], [#nowrap[#line.sku]],
    [#nowrap[#line.description #if line.model != "" [ #h(3pt) #line.model]]],
    [#nowrap[#line.quantity #line.unit]], [#nowrap[#line.packages]], [#nowrap[#line.packageType]],
  )).flatten(),
)
#if payload.notes != "" [#v(6pt) *NOTES* #h(4pt) #payload.notes]
