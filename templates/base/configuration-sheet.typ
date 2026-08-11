#let data = json("configuration.json")
#let configuration = data.configuration
#let labels = data.labels
#let branding = data.branding
#import "helpers.typ": fit-line
#let money(value) = {
  let whole = calc.floor(value / 100)
  let cents = calc.abs(value - whole * 100)
  str(whole) + "." + (if cents < 10 { "0" } else { "" }) + str(cents)
}
#let nowrap(body) = fit-line(body, text-size: 6pt)

#set document(title: labels.title + " " + configuration.code, author: branding.companyName)
#set page(
  paper: "a4",
  flipped: true,
  margin: (x: 10mm, y: 10mm),
  footer: context align(center, text(size: 7pt, fill: luma(90))[
    #branding.companyName - #configuration.code - #counter(page).display("1 / 1")
  ]),
)
#set text(font: ("Arial", "Microsoft YaHei"), size: 8pt, dir: if data.rtl { rtl } else { ltr })
#set par(leading: 0.55em)

#grid(
  columns: (42mm, 1fr, 42mm),
  align: (left, center, right),
  [#if branding.logoPath != "" [#image(branding.logoPath, width: 36mm, height: 12mm, fit: "contain")]],
  [#text(size: 17pt, weight: "bold")[#labels.title]],
  [],
)

#v(5pt)
#table(
  columns: (34mm, 1fr, 37mm, 22mm, 41mm, 39mm),
  inset: 4pt,
  stroke: .45pt + luma(155),
  fill: (column, _) => if calc.even(column) { luma(242) },
  [#nowrap[*#labels.code*]], [#nowrap[#configuration.code]],
  [#nowrap[*#labels.componentCount*]], [#nowrap[#configuration.lines.len()]],
  [#nowrap[*#labels.configurationTotal*]], [#nowrap[*#configuration.currency: #money(configuration.totalAmountMinor)*]],
  [#nowrap[*#labels.productName*]], [#nowrap[#configuration.name]],
  [#nowrap[*#labels.model*]], [#nowrap[#if configuration.model == "" { "-" } else { configuration.model }]],
  [#nowrap[*#labels.currency*]], [#nowrap[#configuration.currency]],
)

#if configuration.currency != "CNY" [
  #v(4pt)
  #align(right)[
    #text(size: 7.5pt)[*#labels.exchangeRate:* 1 CNY = #configuration.exchangeRate #configuration.currency  ·  *#labels.rateDate:* #configuration.exchangeRateDate]
  ]
]

#v(6pt)
#table(
  columns: (8mm, 32mm, 1.45fr, 18mm, 16mm, 28mm, 31mm, 27mm, 1fr),
  inset: (x: 2.5pt, y: 3.5pt),
  align: (center, left, left, right, center, right, right, left, left),
  stroke: .4pt + luma(155),
  fill: (_, row) => if row == 0 { luma(232) },
  table.header(
    [#nowrap[*#labels.number*]], [#nowrap[*#labels.itemName*]], [#nowrap[*#labels.specification*]], [#nowrap[*#labels.quantity*]], [#nowrap[*#labels.unit*]],
    [#nowrap[*#labels.unitPrice*]], [#nowrap[*#labels.amount*]], [#nowrap[*#labels.brand*]], [#nowrap[*#labels.notes*]],
  ),
  ..configuration.lines.enumerate().map(((index, line)) => (
    [#nowrap[#(index + 1)]],
    [#nowrap[#line.name #if line.category != "" [ #h(3pt) #text(fill: luma(95))[#line.category]]]],
    [#nowrap[#if line.specification == "" { "-" } else { line.specification }]],
    [#nowrap[#line.quantity]],
    [#nowrap[#line.unit]],
    [#nowrap[#money(line.unitPriceMinor)]],
    [#nowrap[#money(line.amountMinor)]],
    [#nowrap[#if line.brand == "" { "-" } else { line.brand }]],
    [#nowrap[#if line.notes == "" { "-" } else { line.notes }]],
  )).flatten(),
  table.cell(colspan: 6, align: right, fill: luma(238))[*#labels.configurationTotal*],
  table.cell(align: right, fill: luma(238))[*#configuration.currency: #money(configuration.totalAmountMinor)*],
  table.cell(colspan: 2, fill: luma(238))[],
)

#if configuration.notes != "" [
  #v(6pt)
  #block(width: 100%, inset: 5pt, stroke: .45pt + luma(170))[
    *#labels.notes*\
    #configuration.notes
  ]
]

#v(10pt)
#grid(
  columns: (1fr, 48mm),
  gutter: 15mm,
  [#text(fill: luma(85))[#labels.snapshotNotice\ #branding.companyName]],
  [#if branding.signaturePath != "" [#align(center)[#image(branding.signaturePath, width: if branding.signingAssetKind == "stamp" { 22mm } else { 34mm }, height: if branding.signingAssetKind == "stamp" { 22mm } else { 12mm }, fit: "contain")]] #line(length: 100%)\ #labels.preparedBy],
)
