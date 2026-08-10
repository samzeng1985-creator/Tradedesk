#let data = json("configuration.json")
#let money(value) = {
  let whole = calc.floor(value / 100)
  let cents = calc.abs(value - whole * 100)
  str(whole) + "." + (if cents < 10 { "0" } else { "" }) + str(cents)
}

#set document(title: "Configuration Sheet " + data.code, author: "TradeDesk")
#set page(
  paper: "a4",
  flipped: true,
  margin: (x: 10mm, y: 10mm),
  footer: context align(center, text(size: 7pt, fill: luma(90))[
    TradeDesk - #data.code - Page #counter(page).display("1 / 1")
  ]),
)
#set text(font: ("Arial", "Microsoft YaHei"), size: 8pt)
#set par(leading: 0.55em)

#align(center)[
  #text(size: 17pt, weight: "bold")[PRODUCT CONFIGURATION SHEET]
  #linebreak()
  #text(size: 8pt, fill: luma(90))[产品配置报价清单 / Product Configuration]
]

#v(5pt)
#table(
  columns: (24mm, 1fr, 24mm, 1fr, 20mm, 28mm),
  inset: 4pt,
  stroke: .45pt + luma(155),
  fill: (column, _) => if calc.even(column) { luma(242) },
  [*配置编号*], [#data.code], [*产品名称*], [#data.name], [*币种*], [#data.currency],
  [*型号*], [#if data.model == "" { "-" } else { data.model }],
  [*组件数量*], [#data.lines.len()], [*配置总价*], [*#data.currency #money(data.totalAmountMinor)*],
)

#v(6pt)
#table(
  columns: (8mm, 28mm, 1.35fr, 17mm, 14mm, 25mm, 27mm, 24mm, .85fr),
  inset: (x: 2.5pt, y: 3.5pt),
  align: (center, left, left, right, center, right, right, left, left),
  stroke: .4pt + luma(155),
  fill: (_, row) => if row == 0 { luma(232) },
  table.header(
    [*序号*], [*品名*], [*型号 / 规格 / 材质*], [*数量*], [*单位*],
    [*单价*], [*总价*], [*品牌*], [*备注*],
  ),
  ..data.lines.enumerate().map(((index, line)) => (
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
  table.cell(colspan: 6, align: right, fill: luma(238))[*配置总价 / TOTAL #data.currency*],
  table.cell(align: right, fill: luma(238))[*#money(data.totalAmountMinor)*],
  table.cell(colspan: 2, fill: luma(238))[],
)

#if data.notes != "" [
  #v(6pt)
  #block(width: 100%, inset: 5pt, stroke: .45pt + luma(170))[
    *配置说明 / NOTES*\
    #data.notes
  ]
]

#v(10pt)
#grid(
  columns: (1fr, 48mm),
  gutter: 15mm,
  [#text(fill: luma(85))[本清单价格以保存时的配置快照为准。]],
  [#line(length: 100%)\ 制表 / Prepared by],
)
