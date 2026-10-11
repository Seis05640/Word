# Layout and pagination parity with Microsoft Word

> **Last reviewed:** 2026-10-11 · **Last updated:** 2026-10-11 · **Change:** trivial (legacy table contextual spacing measured and supported) · **Target:** Microsoft Word (Microsoft 365) for Mac 16.113.4; this spacing comparison used 16.113.3

A document only "opens correctly" if its pages break where Word breaks them: the same lines, the
same page count, footnotes and pictures on the same pages. This checklist covers line breaking,
pagination and object placement in `crates/layout`. Typing behaviour is in
[`typing-parity.md`](typing-parity.md); the reading and writing of the properties is in
[`file-format-parity.md`](file-format-parity.md).

**Dimension: ~55% (estimated), 90–145 h to full** (the sum of the rows below). The engine implements most of Word's layout
rules, but nothing has been compared page-for-page with Word on real documents, and the default
font can't match Word's metrics.

## Checklist

✅ matches Word as far as our tests show · 🟡 implemented, differs or untested against Word · ❌ missing

| Feature | State | Evidence / gap | Hours |
|---|---|---|---|
| Line breaking (UAX #14 with Word's exceptions: no break after `/`) | ✅ | `layout::tests::no_line_break_right_after_a_slash` | — |
| Justification, Word 2013+ space shrinking (compat mode 15) | ✅ | #109 | — |
| Kashida justification (Arabic) | ❌ | no kashida in `crates/layout` | 3–5 |
| Distributed / Thai distributed alignment | 🟡 | read; layout untested | 1–2 |
| Default font metrics (Aptos, Calibri, Cambria…) | 🟡 | Carlito/Caladea/Liberation are metric-compatible for Calibri/Cambria/Arial/Times/Courier; **Aptos (Word's default since 2024) has no metric-compatible open substitute**, so new Word documents wrap differently ([typing-parity.md](typing-parity.md) Known gaps). Needs an owner decision (commission or license a metric-matched font) | 6–10 + owner |
| Line spacing (single, 1.5, double, at least, exactly, multiple), spacing before/after, contextual spacing | 🟡 | Word puts multiple-spacing extra below the text and collapses spacing differently (#135, #108 open) | 3–5 |
| Widow/orphan control, keep with next, keep lines together, page break before | ✅ | `lib.rs` keep_next/keep_lines | — |
| Tabs (left, center, right, decimal, bar), leaders | 🟡 | Bar tabs untested | 1–2 |
| Hyphenation (automatic, 0.25" zone, soft hyphens, consecutive limit) | 🟡 | English patterns only; other languages hyphenate wrongly or not at all | in localization |
| Drop caps | ✅ | 2026-10-06 | — |
| Columns, column breaks, separators | 🟡 | **Columns don't balance** at a continuous section break or document end | 4–6 |
| Section breaks (next page, continuous, even, odd), different first page, odd/even headers | ✅ | Section breaks don't show in Draft view (#42) | 1–2 |
| Body top below a tall header, footer pushes body up | ✅ | #138 | — |
| Footnotes at the page bottom, endnotes at the end, note numbering | 🟡 | A long note continues onto the next page at a line boundary (its first line stays with the reference), first in that page's note area under a full-width continuation separator (#352); the document's own `w:continuationSeparator`/`w:continuationNotice` aren't read (default line, no notice); per-section restart partial | 2–4 |
| Tables: row heights (at least/exact), rows split across pages, header rows repeat, Can't Split | ✅ | 2026-10-06, #138 | — |
| Tables: autofit to contents/window, fixed widths, Word 2013 edge | 🟡 | #137; #44: AutoFit Contents measures each column's narrowest and widest text and shares the width like Word, once (not live as you type); cell preferred widths and spans still differ | 3–5 |
| Floating tables (`w:tblpPr`) | 🟡 | #137; overlap rules untested | 2–3 |
| Nested tables | 🟡 | render; polish missing | 2–4 |
| Floating pictures/shapes placement (relative to page/margin/column/paragraph/line/character) | ✅ | #136 | — |
| Text wrap: square, top-and-bottom, behind, in front | ✅ | 2026-10-06 | — |
| Text wrap: tight, through (contour, wrap polygon) | ❌ | laid out as square | 6–10 |
| Text boxes, overflow, linked text boxes | 🟡 | #46; linked text boxes missing | 3–5 |
| Rotated objects and text | 🟡 | #332: pictures, shapes, charts and groups drawn turned and flipped about their centre (screen and PDF); square wrap and inline lines keep clear of the rotated bounds; hit testing on the turned shape. Text in a rotated text box stays upright | 2–3 |
| Page borders (from page edge or text), page colour, watermark | ✅ | | — |
| Line numbers (restart per page/section, count by) | ✅ | | — |
| Vertical page alignment | ✅ | | — |
| Gutter, mirror margins, book fold, 2 pages per sheet | 🟡 | gutter and mirror margins; book fold missing | 2–3 |
| Document grid (`w:docGrid`, lines per page, characters per line) | ❌ | East Asian documents paginate differently without it | 4–6 |
| Vertical text (`tbRl`) in sections and text boxes; cell text direction | 🟡 | cell text direction landed (#245); page-level and text-box vertical text missing | 8–12 |
| Ruby / phonetic guide, enclose characters, combined characters | ❌ | | 4–6 |
| Bidirectional paragraphs, mixed-direction lines | 🟡 | #207; RTL sections and tables (`w:bidiVisual`) missing | 4–6 |
| Equations: display layout, numbering, wrapping long display equations | ✅ | #191; long display equations break at top-level operators (`m:brkBin`, `m:brkBinSub`), manual breaks and `m:alnAt`, continuation lines indented by `m:wrapIndent` or set right (`m:wrapRight`) (#326) | — |
| Hidden text excluded from breaking and hyphenation | ✅ | #173 | — |
| Fields: PAGE, NUMPAGES, SECTIONPAGES, TOC page numbers | ✅ | #52 | — |
| Compatibility modes (Word 2003/2007/2010 layout for old files: `w:compat` options) | 🟡 | `allowSpaceOfSameStyleInTable` supported in modes 11/12/14; mode 15 retains contextual suppression. Most other legacy compat options remain ignored | 10–20 |
| Page-by-page comparison harness against Word | ❌ | Run Word locally on our own synthetic documents, export PDF, compare line breaks; Word output stays under `plan/` (never committed) | 8–12 |

## Legacy table contextual spacing

Observed in Word for Mac 16.113.3 (build 26092714) on macOS 26.6.2, using
original synthetic DOCX files, Roboto Regular 12 pt, exactly 24 pt lines, equal paragraph
styles, both paragraphs contextual, first paragraph after-space 12 pt and before-space 0 pt.
The one-cell table had no borders or cell margins. Baseline separation was 36 pt with
`w:allowSpaceOfSameStyleInTable` enabled in compatibility modes 11, 12 and 14, versus
24 pt with it disabled. Mode 15 measured 24 pt in either state. Body and header controls
in modes 14 and 15 measured 24 pt with the flag enabled. Comparison tolerance: 0.25 pt.

The model and DOCX reader/writer preserve the flag. Cell layout bypasses contextual
suppression only for enabled modes 11/12/14, including cells without table-style formatting.
Tests: `legacy_table_contextual_spacing_flag_keeps_after_space` and
`legacy_table_contextual_spacing_flag_is_scoped_to_cells_and_legacy_mode`.
WordCraft original-open and save/reopen measurements match all 12 controls; reopening
its saved DOCX files in Word retains the same gaps. Word-derived exports stay local under
ignored `plan/word/`; no Word-generated fixture is committed.

The [published option](https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.wordprocessing.allowspaceofsamestyleintable?view=openxml-3.0.1)
specifies an override inside tables. Its mode 15 exception is not explained by that source;
the guard follows this observation. [MS-DOCX compatibilityMode](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-docx/90138c4d-eb18-4edc-aa6c-dfb799cb1d0d)
allows 11, 12, 14 and 15. Mode 13 is invalid and Word rejected its synthetic input.
Ordinary overlapping before/after space and one-sided contextual spacing remain separate
work in #108 and #568. Nested tables, rotated cells, other fonts and Word for Windows
were not reference-tested here. This closes one compatibility-option gap, not table parity.

## Performance

| Measure | WordCraft | Word | Kind |
|---|---|---|---|
| Cold layout, 188-page document | 61 ms | not measured | measured 2026-10-06 (not re-run this pass) |
| Relayout after an edit | 1.4 ms | not measured | measured 2026-10-06 |
| Font picker with many installed fonts | fixed freezes (#233) | instant | issue #31, #59 |
| Large real-world documents (500+ pages, many pictures) | not measured | | gap |

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-11 | trivial | Legacy table contextual spacing measured and supported; modes 11/12/14 versus 15 checked with original synthetic documents |
| 2026-10-11 | trivial | Group and Ungroup keep turned members' orientation and placement (#441, #443); Size resizes a group (#442); the eraser hits turned ink where it is drawn (#445) |
| 2026-10-10 | trivial | Long footnotes continue onto the next page with a continuation separator (#352) |
| 2026-10-10 | trivial | Rotated and flipped objects laid out and drawn (#332) |
| 2026-10-10 | trivial | Long display equations wrap across lines (#326) |
| 2026-10-10 | trivial | AutoFit Contents measures the text (#44) |
| 2026-10-10 | major | First version: layout and pagination checklist with evidence from `crates/layout`, PRs and issues |
