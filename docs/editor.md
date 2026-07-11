# Editor

The hardest and most performance-critical part of the app.
"Performance above everything" lives or dies here, not in the sync
layer.

## Model

- The markdown source text **is** the document; the editor styles it
  in place (Obsidian-style live preview) -- no separate edit/preview
  modes
- The Rust core supplies markdown syntax spans for highlighting
  (pulldown-cmark offset iterator)
- Incremental restyling on edit: re-parse from the nearest structural
  boundary (paragraph, heading, code fence) rather than the whole
  document. Requires building an offset-index layer on top of
  pulldown-cmark's event stream -- significant engineering, not just
  an optimization.

## Requirements (all platforms, v1)

- Instant typing latency on 10k+ line documents
- Tables render properly (non-negotiable)
- Checklists toggle by tap

## Android (Phase 1)

- Jetpack Compose `BasicTextField` with visual transformation for
  markdown styling
- Benchmark with multi-thousand-line notes on mid-range hardware from
  day one -- this is where v1 schedule risk lives; Compose text is
  less proven for this use case than TextKit

## macOS / iOS (Phase 3 / later)

- `NSTextView` / `UITextView` on **TextKit 2**, wrapped for SwiftUI
- SwiftUI `TextEditor` is not capable enough for large documents --
  do not start there

## Markdown support (v1)

Native note format; markdown source is the stored truth. Parsed with
pulldown-cmark.

- Headings
- Bold / Italic
- Lists
- Checklists
- Blockquotes
- Tables
- Code blocks
- Links
- Horizontal rules
