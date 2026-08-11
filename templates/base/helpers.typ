#let fit-line(body, text-size: 6.2pt) = box(width: 100%)[
  #layout(size => {
    let content = text(size: text-size, body)
    let natural = measure(content)
    if natural.width > size.width {
      scale(x: size.width / natural.width * 100%, origin: left + horizon, reflow: true, content)
    } else {
      content
    }
  })
]
