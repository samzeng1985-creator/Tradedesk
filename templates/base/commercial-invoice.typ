#let invoice(data) = {
  set page(paper: "a4", margin: 14mm)
  set text(font: ("Noto Sans", "Noto Sans CJK SC"), size: 8.5pt)

  align(center)[
    #text(size: 16pt, weight: "bold")[COMMERCIAL INVOICE]
  ]

  v(6pt)
  table(
    columns: (1fr, 1fr),
    inset: 5pt,
    stroke: .5pt,
    [*SHIPPER / EXPORTER*\ #data.shipper],
    [*CONSIGNEE*\ #data.consignee],
  )

  v(6pt)
  table(
    columns: (auto, 1fr, 1fr, 1fr, 1fr),
    inset: 4pt,
    stroke: .5pt,
    table.header([*#*], [*DESCRIPTION*], [*QTY*], [*UNIT PRICE*], [*TOTAL*]),
    ..data.items.enumerate().map(((index, item)) => (
      [#index + 1],
      [#item.description],
      [#item.quantity],
      [#item.unit_price],
      [#item.total],
    )).flatten(),
  )
}
