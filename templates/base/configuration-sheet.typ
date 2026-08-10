#let data = json("configuration.json")
#let configuration = data.configuration
#let labels = data.labels
#let branding = data.branding
#let money(value) = {
  let whole = calc.floor(value / 100)
  let cents = calc.abs(value - whole * 100)
  str(whole) + "." + (if cents < 10 { "0" } else { "" }) + str(cents)
}

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
  [#box[#text(size: 7pt, weight: "bold")[#labels.code]]], [#configuration.code],
  [#box[#text(size: 7pt, weight: "bold")[#labels.componentCount]]], [#configuration.lines.len()],
  [#box[#text(size: 7pt, weight: "bold")[#labels.configurationTotal]]], [*#configuration.currency: #money(configuration.totalAmountMinor)*],
  [#box[#text(size: 7pt, weight: "bold")[#labels.productName]]], [#configuration.name],
  [#box[#text(size: 7pt, weight: "bold")[#labels.model]]], [#if configuration.model == "" { "-" } else { configuration.model }],
  [#box[#text(size: 7pt, weight: "bold")[#labels.currency]]], [#configuration.currency],
)

#if configuration.currency != "CNY" [
  #v(4pt)
  #align(right)[
    #text(size: 7.5pt)[*#labels.exchangeRate:* 1 CNY = #configuration.exchangeRate #configuration.currency  ·  *#labels.rateDate:* #configuration.exchangeRateDate]
  ]
]

#v(6pt)
#table(
  columns: (8mm, 28mm, 1.35fr, 17mm, 14mm, 25mm, 27mm, 24mm, .85fr),
  inset: (x: 2.5pt, y: 3.5pt),
  align: (center, left, left, right, center, right, right, left, left),
  stroke: .4pt + luma(155),
  fill: (_, row) => if row == 0 { luma(232) },
  table.header(
    [*#labels.number*], [*#labels.itemName*], [*#labels.specification*], [*#labels.quantity*], [*#labels.unit*],
    [*#labels.unitPrice*], [*#labels.amount*], [*#labels.brand*], [*#labels.notes*],
  ),
  ..configuration.lines.enumerate().map(((index, line)) => (
    [#(index + 1)],
    [#line.name #if line.category != "" [\ #text(size: 6.5pt, fill: luma(95))[#line.category]]],
    [#if line.specification == "" { "-" } else { line.specification }],
    [#line.quantity],
    [#line.unit],
    [#money(line.unitPriceMinor)],
    [#money(line.amountMinor)],
    [#if line.brand == "" { "-" } else { line.brand }],
    [#if line.notes == "" { "-" } else { line.notes }],
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
  [#if branding.signaturePath != "" [#align(center)[#image(branding.signaturePath, width: 34mm, height: 12mm, fit: "contain")]] #line(length: 100%)\ #labels.preparedBy],
)
