# Textbook mode for the formula editor — design

Status: **P0–P3 built** on `feat/formula-textbook` (2026-10-01); P4 (keypad, completion, Copy as
LaTeX, library rows) to do. The user chose the recommendations of §8: Latin Modern Math, `=` as
typed, juxtaposition, the notation of §4.5, full scope. Extends the Custom formula dialog
(`design/custom-formulas.md` §4.8; `ui/formula_dialog.rs`, `ui/formula_editor.rs`). Facts about the
current code are checked against it and cited; anything still to be proven is marked **spike**.

## As built (P3): where it departs from the text below

- **Atoms are characters.** Names and numbers are runs of `Char` atoms (MathQuill's model), not
  `Word`/`Number` strings: every caret place is a boundary between atoms, and a run reads as the
  lexer reads the same text (`model::run_tokens`: `p1` one name, `2z` a number then a name). There is
  no `Slot` atom: an empty row inside a structure is drawn as the dashed box.
- **Implied products are not atoms.** A `*` the printer would write anyway (`2*z`, `z*(z + 1)`) is
  left out of the row, on reading and after every edit, so no caret step is invisible; one it would
  not (`z*c`, which would run into the name `zc`) stays, and shows as a dot. The dot rule of §4.5 is
  extended on the same principle: between two names (𝑧·𝑐) and before a sign (𝑧·−𝑐).
- **`+`, `-`, `=` typed at the end of a non-empty exponent step out of it** (MathQuill's
  `charsThatBreakOutOfSupSub`, as Desmos sets it), so `z^2+c` types as 𝑧² + 𝑐, as the text reads.
  `z^-1` keeps its sign. A `/` in an exponent still makes a fraction there.
- **`)` with no `(`** groups what is before the caret (after the row's `=`), as MathQuill does.
- **Backspace taking a call apart keeps its argument's parentheses** (`sin(z)` → `(z)`); an empty
  structure goes in one press.
- **Paste** reads the text alone, except that a leading `+`/`-` after an operand is typed as the
  operator first (read alone, `- 1/z` is a negative fraction).
- **Errors:** the line the syntax check stops at is underlined; the message gives that line, or
  "Fill the empty box." when there is one. Atom-level underlining needs a printer source map: not
  done.
- **Comments** are shown, kept and moved with their line, but edited in Text mode.
- **Focus:** `Response::has_focus()` is false in an unfocused WINDOW (egui ≥ 0.29); the editor
  reads its focus from memory, and only the caret hides with the window's focus.

## 1. Goal

A toggle in the formula editor, **Text | Textbook**. In Textbook mode the formula is typeset as a
mathematics textbook (or LaTeX) would set it — stacked fractions, raised exponents, parentheses that
grow with what they enclose, radicals, italic variables — and it is **edited in that form**: the
caret moves through numerators, denominators and exponents, typing `/` makes a fraction, `^` an
exponent, `(` a pair of parentheses.

- **The text stays the formula's identity.** Sessions, `.fdn` files, the library, tours and the
  parser all keep reading the text (`custom_formula.rs`). Textbook mode is a second way of editing the
  same string, never a second format.
- **Toggling never changes the formula.** Text → Textbook → Text with no edit returns the text byte
  for byte; an edit rewrites only the statement it touched.
- **What is shown is what is computed.** The typeset form must mean exactly what the parser makes of
  the text (§4.5); where a familiar notation would mislead, the honest one is used.
- **It looks like LaTeX**: Computer Modern glyphs and TeX's own layout rules, driven by the
  OpenType MATH table of the font (§4.6).

### Non-goals

- A general mathematics editor: only the formula language's constructs (§4.3) are editable.
- Typing LaTeX commands (`\frac`). The keyboard, keypad and completion are the input methods; a
  "Copy as LaTeX" output is in scope (§4.4), LaTeX input is not.
- Line breaking of long formulas (they scroll horizontally); right-to-left scripts; IME composition
  beyond what egui delivers as text (identifiers are ASCII in this language).

## 2. What exists today

- **The text field** (`ui/formula_editor.rs`, `933412a`): a multi-line `TextEdit` with a custom
  layouter — parentheses coloured by depth, the pair at the caret highlighted, comments dimmed — and
  completion of the language's names. Its egui lessons apply here: Tab/Esc must be claimed with
  `set_focus_lock_filter`, and a popup entry must be taken on the press (`topic-ui-standards`).
- **The keypad** (`ui/formula_keypad.rs`): every name and operator, as `Action::Insert` / `Action::Wrap`;
  tests hold it to the parser's vocabulary in both directions.
- **The parser** (`core/src/ir/parse.rs`) goes straight from tokens to IR, folding constants and
  resolving variables as it goes. **There is no syntax tree**: precedence, parentheses and spellings
  are gone by the time anything else sees the formula. Grammar, as implemented:

  ```
  formula   := statement (sep statement)*            sep = newline | ',' outside parentheses
  statement := ident '=' expr | expr                  ';' comments run to the end of the line
  expr      := term (('+' | '-') term)*
  term      := unary (('*' | '/') unary)*
  unary     := ('-' | '+') unary | power              so -z^2 = -(z^2), -b*c = (-b)*c
  power     := atom ('^' unary)?                      right-associative: z^2^3 = z^(2^3); z^-1
  atom      := number | name | func '(' expr ')' | '(' expr ')' | '(' expr ',' expr ')' | '|' expr '|'
  ```

- **Spellings that compute differently** (verified in `parse.rs`): `z^2` lowers to `PowI(z, 2)` and
  `sqr(z)` to `Sqr(z)`; `|z|` is the **squared** modulus (`Norm`), `cabs(z)` the modulus,
  `abs(z)` the absolute value of each PART; `e^z` is a power with a varying exponent (which renders
  direct only), not `exp(z)`; `0.1*z` multiplies by the rounded double while `z/10` divides by an
  exact 10 — different fractals past ~1e15× (Help, `75eaff6`). The typeset form must keep all of
  these apart (§4.5).
- **Fonts**: Spline Sans / Spline Sans Mono (OFL, `assets/fonts/OFL.txt`) and a Lucide icon subset
  produced by `scripts/subset_lucide.py` (fonttools), which also generates `icons.rs`. egui renders
  text from a glyph atlas via ab_glyph; `ttf-parser` 0.25.1 is already in the build under it, with its
  OpenType MATH-table API (`tables::math`: `Constants`, `Variants`, `GlyphAssembly`) behind the
  default `opentype-layout` feature.

## 3. Assumptions to validate

| # | Assumption | How it will be checked |
|---|---|---|
| T1 | egui draws glyphs of an OTF/CFF math font at arbitrary sizes, crisp at 1.0–2.0 points-per-pixel | **Spike**: Latin Modern Math through `install_fonts`, screenshots at 1.0×, 1.5×, 2.0× |
| T2 | The baseline of an egui galley can be found exactly (for TeX's baseline-relative placement) | **Spike**: draw a glyph at a known baseline; compare its ink box with `ttf-parser`'s glyph bbox |
| T3 | A subset of the font with Private Use Area code points for the stretchy variants keeps its MATH table usable | **Spike**: `fonttools subset` + cmap edit; read the MATH table back with `ttf-parser`; draw `(` size variants by code point |
| T4 | The GUST Font License (Latin Modern) allows shipping a renamed subset in an MIT/Apache app | Read the licence; **the user decides** (§8) |
| T5 | A flat-row editor model (§4.3) can represent every parse of the grammar, and printing it back reproduces the parse | The identity tests of §6 over every formula in the tests and corpus |

## 4. Design

### 4.1 Overview

```
 text ──parse──▶ syntax tree (core, with spans) ──lower──▶ IR          (unchanged consumers)
  ▲                    │
  │                    ▼ flatten (§4.4)
  └──print (§4.4)── editor tree ──layout (§4.6)──▶ boxes ──paint──▶ egui painter
                       ▲  │                         │
            commands ──┘  └── caret, selection ◀────┘ hit test
```

The editor tree is the only thing Textbook mode edits. After every command the tree is printed to the
source string (`formula_dialog.source`), so the syntax check, Apply, Save to library and completion
keep working on text exactly as today.

### 4.2 A syntax tree in core (byte-neutral refactor)

`core::ir::syntax` gains a proper syntax tree, and the existing parser is split into
**parse → syntax tree** and **lower → IR**:

```rust
pub struct Syntax { pub statements: Vec<Statement> }
pub struct Statement {
    pub target: Option<(String, Span)>,   // `t = …`
    pub body: Expr,
    pub sep: Sep,                          // newline or comma, for verbatim printing
    pub comment: Option<(String, Span)>,   // `; …`, kept (the lexer drops it today)
    pub span: Span,
}
pub enum Expr {                            // every node carries its source Span
    Num { text: String, value: f64 }, Name(String), Call { func: String, arg: Box<Expr> },
    Group(Box<Expr>), Complex(Box<Expr>, Box<Expr>), Bars(Box<Expr>),
    Neg(Box<Expr>), Pos(Box<Expr>), Bin { op: BinOp, l: Box<Expr>, r: Box<Expr> },
    Pow { base: Box<Expr>, exp: Box<Expr> },
}
```

**As built (P1, `5de58e7`…):** not a parse-then-lower split but **one pass** that builds both.
A split would have changed which error a source reports: the parser stops at its first error of
either kind, so `a2 ^= z` is "`a2` is used before it is assigned" today, while a syntax pass first
would report the `=`. Instead every production returns its IR value AND its node; `parse(src)` runs
the pass building IR exactly as before (the same calls in the same order), and `syntax(src)` runs it
with the three evaluation-only checks — a name used before it is assigned, a complex constant of
non-constants, a formula with no step — left out, so a formula still being typed has a tree. **One
grammar** serves the renderer and the editor, so the two cannot drift.

**Gate (byte-neutral), met:** a snapshot of 2,100 inputs — the repository's formulas plus 1,500
generated ones and corrupted copies — records each one's IR (Debug form, every constant's bits) or
exact error; it was recorded BEFORE the change and is unchanged after it, and a planted one-character
change to the parser turns it red, naming the inputs. Over the same corpus: whatever `parse` accepts
`syntax` accepts; where `syntax` refuses, `parse` refuses with the same error or with an evaluation
error earlier in the text.

### 4.3 The editor tree

Modelled on MathQuill's: a formula is a list of **statements**, each a **row** — a flat sequence of
atoms in which standard precedence applies, exactly as in the text — and the structures that need
two dimensions hold rows of their own.

```rust
enum Atom {
    Word(String),                // identifier: z, c, t, p1, pixel, pi, e — edited letter by letter
    Number(String),              // as typed: "0.25", "1e-5"
    Op(OpKind),                  // + − (binary or unary), × (explicit or implied), =
    Frac { num: Row, den: Row }, // from '/'
    Sup(Row),                    // from '^'; attaches to the atom before it (the base)
    Group(Row),                  // parentheses the USER wrote: shown in both modes
    Func { name: Func, arg: Row },
    Bars(Row),                   // |…| = squared modulus
    Complex { re: Row, im: Row },// (re, im)
    Slot,                        // an empty place still to fill (drawn as a dashed box)
}
```

- **Caret** = (path to a row, index between atoms or between the letters of a `Word`/`Number`).
  **Selection** = a contiguous range of atoms in one row (as in MathQuill), so every selection
  prints to a valid fragment.
- **Invariant:** printing a tree without `Slot`s gives text that parses; printing any tree, with
  `Slot`s printed as nothing, gives text whose parse error points at the first `Slot` (mapped back for
  display, §4.8).
- Comments are text atoms at the end of their statement, edited as plain text.

### 4.4 Conversions

**Syntax tree → editor tree (flatten).** `Bin(+|−|×)` becomes the operands' rows joined by the
operator; `Bin(/)` becomes `Frac`; `Pow` becomes base atoms + `Sup`; a `Group` that is the whole
numerator, denominator or exponent is unwrapped (the structure already groups it: `c/(z + 1)` shows no
parentheses), and every other `Group` is kept (`(z + 1)²`, `a − (b + c)`). Each atom keeps its
statement's span, so the caret maps between modes in both directions.

**Editor tree → text (print).** Tokens in order, `×` printed as `*` (implied products included —
the language has no implicit multiplication), minus as `-`. A `Frac`, `Sup` base or exponent whose
row is not a single primary is wrapped in **synthetic** parentheses (`(a + b)/(c*d)`, `z^(p1 + 1)`);
these exist only in the text. Redundant parentheses do not change the IR (the parser returns the
inner value; constants fold through them), so wrapping is always safe. Spacing is canonical
(`a + b`, `a*b`, `a/b`, `x^2`), but **a statement the user did not edit is printed from its original
span, verbatim** — comments, spacing and redundant parentheses included.

**Editor tree → LaTeX** ("Copy as LaTeX", a menu item in Textbook mode): `\frac{}{}`, `^{}`,
`\left( \right)`, `\sqrt{}`, `\overline{}`, `\lvert\cdot\rvert^2`, `\operatorname{Re}` — for forum
posts and papers. Output only.

### 4.5 What each construct looks like (truthful notation)

| Text | Textbook | Why |
|---|---|---|
| `z`, `c`, `t`, `w2` | 𝑧, 𝑐, 𝑡, 𝑤₂ | math italic; trailing digits of a name as a subscript |
| `p1`…`p5` | 𝑝₁…𝑝₅ | |
| `pixel` | 𝑝𝑖𝑥𝑒𝑙 | a multi-letter name, italic as one word |
| `pi`, `e` | π, 𝑒 | |
| `0.25`, `1e-5` | 0.25, 1×10⁻⁵ | numbers upright; scientific form when the caret is outside the number |
| `a - b`, `-a` | 𝑎 − 𝑏, −𝑎 | true minus sign |
| `2*z`, `p1*conj(t)` | 2𝑧, 𝑝₁𝑡̄ | juxtaposition; `·` only between two numbers (2 · 3) |
| `a/b` | stacked fraction | smaller in exponents, as TeX's styles do |
| `z^p1`, `z^-1` | 𝑧^𝑝₁, 𝑧⁻¹ | |
| `sqr(z + c)` | (𝑧 + 𝑐)² | ²; prints back as `sqr(…)`, NOT `^2` (`Sqr` ≠ `PowI`) |
| `sqrt(x)` | √x with vinculum | |
| `\|z\|` | \|𝑧\|² | the language's bars are the SQUARED modulus |
| `cabs(z)` | \|𝑧\| | the modulus |
| `abs(z)` | abs(𝑧) | per-part absolute value: neither \|𝑧\| nor anything shorter is honest |
| `conj(z)` | 𝑧̄ (overline over the whole argument) | |
| `real(z)`, `imag(z)` | Re(𝑧), Im(𝑧) | |
| `recip(z)` | 1/𝑧 stacked | prints back as `recip(…)` |
| `exp(z)` | exp(𝑧) | NOT 𝑒^𝑧: `e^z` is a varying-exponent power, a different computation |
| `sin`…`tanh`, `log` | sin, cos, tan, sinh, cosh, tanh, log | upright operator names, thin space |
| `cotan`, `cotanh` | cot, coth | standard names; print back as written |
| `flip`, `ident` | flip(…), ident(…) | no textbook equivalent |
| `(0.25, -0.1)` | (0.25 − 0.1𝑖) | parentheses whenever it is not a whole slot |
| `t = sqr(z)` ⏎ `z = t + c` | two rows aligned at = | as `align*`; separators (`,` or newline) print back as written |
| `; comment` | dimmed UI-font text after the row | |

Each node that changes its look (²,  1/𝑧, cot) keeps its source spelling, so printing it back is
exact. Two decisions are the user's (§8): `=` versus `←` (or the recurrence 𝑧ₙ₊₁ = …), and
juxtaposition versus `·` throughout.

### 4.6 Layout: TeX's rules on the OpenType MATH table

A small TeX-style box layout (`ui/textbook/layout.rs`): every atom becomes a box with width, height
(above the baseline), depth (below) and italic correction, laid out in a **style** — display, text,
script, scriptscript, each normal or cramped — exactly as in TeX (*The TeXbook*, Appendix G). Every
dimension comes from the font's MATH table through `ttf-parser`, not from constants of our own:

- **Sizes:** script and scriptscript at `scriptPercentScaleDown` / `scriptScriptPercentScaleDown`
  (70% / 50% in Latin Modern).
- **Spacing between atoms:** TeX's table over Ord, Op, Bin, Rel, Open, Close, Punct, Inner (thin,
  medium, thick space = 3, 4, 5 mu), none in script styles except after operators; a binary minus
  with nothing to its left becomes Ord (unary).
- **Fractions** (rule 15): numerator in the next smaller style, denominator cramped, both centred;
  the bar `fractionRuleThickness` thick on the math axis (`axisHeight`); shifts and minimum gaps from
  `fractionNumerator(DisplayStyle)ShiftUp`, `…GapMin` and the denominator counterparts.
- **Exponents and subscripts** (rules 17–18): `superscriptShiftUp(Cramped)`, `superscriptBottomMin`,
  `superscriptBaselineDropMax`, `subscriptShiftDown`, `subscriptTopMax`, `spaceAfterScript`; the
  base's italic correction from `MathItalicsCorrectionInfo`.
- **Parentheses and bars** grow with their content (TeX's `\left…\right`): the size needed is set
  by `delimitedSubFormulaMinHeight` and the content's extent about the axis; the glyph is the first
  of the font's vertical **size variants** tall enough, or past the largest, an **assembly** of
  top/extender/bottom parts (`GlyphAssembly`, `minConnectorOverlap`).
- **Radicals:** `radicalVerticalGap`, `radicalRuleThickness`, `radicalExtraAscender`; the radical sign
  from the √ variants and assembly, as for delimiters.
- **Overline** (`conj`): `overbarVerticalGap`, `overbarRuleThickness`, `overbarExtraAscender`.
- **Rows:** statements stacked with the font's line gap, aligned at their first `=`.
- **Slots:** a dashed rectangle of the x-height's size; the caret's row is never empty-looking.

Glyph ink boxes (height/depth per glyph, which TeX uses and egui does not expose) come from
`ttf-parser`'s glyph bounding boxes. Rules are snapped to whole physical pixels. The whole layout of a
formula is a few hundred boxes; it is recomputed only when the tree, the size or the DPI changes.

### 4.7 The math font

- **Latin Modern Math** — the OpenType Computer Modern that LaTeX users know — is the recommended
  face; STIX Two Math and Libertinus Math (both OFL) are the alternatives if its licence is not
  acceptable (T4, §8).
- `scripts/subset_math_font.py` (fonttools, as `subset_lucide.py`) keeps only what the language can
  show: Latin letters and math italics, digits, π, the operators, the parenthesis / bar / radical
  **size variants and assembly parts**, and the MATH table. It **maps every variant and part to a
  Private Use Area code point**, because egui renders text by character and the variants have none;
  it renames the font (a modified font must not carry the original name) and writes the map into a
  generated `ui/textbook/math_glyphs.rs`, as `icons.rs` is generated. Estimated 50–80 KB, against
  ~700 KB for the whole face.
- `install_fonts` registers it as `FontFamily::Name("math")`; its licence ships beside `OFL.txt`.
- Why not draw glyphs ourselves: `ttf-parser` outlines would need tessellating into meshes (egui's
  `PathShape` fills convex shapes only), and egui draws a mesh as given, without the edge feathering
  that anti-aliases its own shapes; glyphs from the atlas are anti-aliased already.

### 4.8 The widget

`ui/textbook/widget.rs`, a custom egui widget in place of the `TextEdit` when the toggle says so:

- **Paint:** glyph runs as egui text in the `math` family at the box's size and baseline, fraction
  bars and vinculums as rectangles, slots as dashed rectangles, the selection as a fill behind its
  atoms, the caret as a line the height of its row's box (blinking as egui's text cursor does).
- **Hit testing:** each row records its caret x-positions and vertical extent; a click goes to the
  innermost row containing the point and the nearest caret position.
- **Keyboard:** while focused, the widget reads `Event::Text`, `Event::Key`, `Copy`, `Cut` and
  `Paste`. It claims Tab, Esc and all four arrows with `set_focus_lock_filter` (egui moves focus at
  the start of a frame otherwise).
- **Completion:** the same list as the text field (`formula_editor::candidates`, the keypad's
  vocabulary plus assigned variables), opened on the `Word` at the caret and placed under it. Taking a
  function makes a `Func` with the caret in its argument.
- **Keypad:** each `Action` maps to a command: `Insert("^2")` → `Sup(2)`, `Insert("^")` → empty `Sup`,
  `Wrap("sqrt(", ")")` → radical around the selection, `Wrap("|", "|")` → `Bars`, the function keys →
  `Func`, `÷` → `Frac`, `(a, b)` → `Complex` with two slots, `↵` → new statement, the arrows → caret
  moves. A test walks every key, as the keypad tests do.
- **Errors:** the syntax check still runs on the printed text; its line/column maps through the spans
  to the atom, which is underlined in the error colour. An empty slot reads "Fill the empty box".
- **Accessibility:** the widget's AccessKit value is the source text.

### 4.9 Editing commands

| Input | Effect (caret `│`) |
|---|---|
| letter, digit | extends the `Word`/`Number` at the caret, else starts one (a digit after a letter extends the name, as the lexer reads `p1`); a new factor right after a complete one inserts an implied `×` (`2𝑧`) |
| `+` `-` `*` `=` | an `Op`; `-` is unary at the start of a row or after an operator or `(` |
| `/` | `Frac` whose numerator is the term to the left — back to the previous binary `+`/`−`, `=`, `,` or the row start, unary minus included, exactly the parser's left operand of `/` — caret in the denominator; with a selection, the selection |
| `^` | `Sup` on the atom to the left (a `Slot` base if none), caret in the exponent |
| `(` | `Group` with the caret inside; after a function name, that name becomes a `Func` |
| `)` | at the end of a `Group`: step out of it; elsewhere ignored (a structure cannot be unbalanced) |
| `\|` | `Bars`, typed again at its end: step out |
| `,` | top level: new statement; inside a `Group`: it becomes a `Complex` |
| Enter | new statement below (Enter after a complete name, never swallowed by completion) |
| ← → | through atoms and letters, into and out of structures (MathQuill's order) |
| ↑ ↓ | numerator ↔ denominator, into/out of an exponent, else previous/next statement |
| Tab | next `Slot` (completion takes it first when its list is open) |
| Backspace | the atom to the left; at the start of a structure's slot, select the structure, then a second press removes it and keeps its contents inline (`a/b` → `a b`) |
| Shift+arrows, drag | select within one row |
| Ctrl+C / X / V | the selection's printed text; paste parses text as atoms or statements |
| Ctrl+Z / Y | snapshot undo of (text, caret) |

After each command a local **normalisation** keeps the tree in grammar form: adjacent letters merge
into one `Word`; a function name followed by a `Group` becomes a `Func`; a `Func` whose name was edited
into something else falls back to `Word` + `Group`; empty `Frac`/`Sup` slots stay `Slot`s. Re-parsing
the printed text would do this too, but most states while typing do not parse.

### 4.10 Toggle and integration

- A segmented **Text | Textbook** control in the dialog's header row, beside Examples. The choice
  persists (`SessionState::formula_view`, `serde(default)` = text, so older sessions load).
- Switching maps the caret through the spans. If the text does not parse, Textbook mode typesets the
  statements that do and shows the others as red text; clicking one returns to Text mode at that place.
- Apply, Save to library, the parameters grid, the depth note and the syntax line are unchanged: all
  read `formula_dialog.source`, which Textbook mode keeps printed.
- Later, the same renderer can typeset the formula library's rows and the side panel's "About Custom"
  (display only).

## 5. Phases and gates

| Phase | Work | Gate | Estimate |
|---|---|---|---|
| P0 spike | font subset with PUA variants; MATH table via `ttf-parser`; a fraction, an exponent and stretchy parentheses drawn in egui | T1–T3 hold; a screenshot beside KaTeX's rendering of the same formula looks the same to the user; T4 decided | 1–2 days |
| P1 syntax tree | §4.2 split, spans, comments kept | byte-neutral: identical IR, WGSL, errors over the corpus; all tests and the self-test unchanged | 2–3 days |
| P2 display | flatten, layout, paint, the toggle (Textbook read-only; clicking edits in Text mode) | every corpus formula typesets without panic; layout invariants (§6); uitest screens reviewed | 4–6 days |
| P3 editing | caret, the commands of §4.9, normalisation, selection, clipboard, undo, error mapping | command-sequence suite; round-trip identities; headless egui interaction tests | 6–10 days |
| P4 polish | keypad and completion in Textbook mode, Copy as LaTeX, library rows typeset | uitest; the user's review | 2–3 days |

P2 is useful on its own: it delivers the typeset display while editing stays textual.

## 6. Validation plan

- **Identity, the property everything rests on:** for every formula `f` in the corpus,
  `print(flatten(syntax(f)))` parses to the IR `parse(f)` produces, and with no edit, toggling returns
  `f` byte for byte. A grammar-driven random expression generator extends the corpus.
- **Commands:** sequences of keystrokes with the expected printed text — `z^2+c/z` → `z^2 + c/z`;
  `a+b/` takes `b`; `-b/` takes `-b`; `(z+1)^2`; Backspace unwrapping; every keypad key.
- **Layout invariants** per construct: the fraction bar on the axis, numerator and denominator clear
  of it by the MATH gaps, an exponent raised at least `superscriptBottomMin`, delimiters covering
  their content, boxes never overlapping in a row. Box metrics of a fixed formula set snapshotted to
  0.01 pt, so an unintended layout change shows.
- **Interaction in a headless egui context** (the method that found three egui traps in `933412a`):
  typing, arrows, Tab and Esc keeping focus, a click into a denominator, selection by drag.
- **Look:** uitest screens for each construct, the caret in a denominator, an empty slot, the
  completion list in Textbook mode, both themes; reviewed beside LaTeX renders of the same formulas
  at 1.0×, 1.5× and 2.0×.

## 7. Risks

| Risk | Mitigation |
|---|---|
| The font's licence (GUST) does not fit | OFL alternatives (STIX Two Math, Libertinus Math); decided at P0 |
| egui's text placement cannot be pinned to a baseline exactly | P0 spike; fall back to measuring each glyph run's ink box once per size |
| Structural editing feels wrong where it differs from typing text | Every command mirrors the parser (the `/` rule above all); the Text toggle is always one click away; P3 ends with the user trying it |
| The typeset form drifts from what the parser computes | One grammar (§4.2); the identity tests; spellings kept in nodes |
| Canonical spacing rewrites a statement the user formatted | Only edited statements are reprinted |
| Scope: a WYSIWYG maths editor is a large project in general | The language is small: ten constructs, ASCII names, no matrices or big operators |

## 8. Decisions for the user

1. **Font:** Latin Modern Math (the LaTeX look; GUST Font License, a renamed subset shipped with
   its licence) — recommended — or an OFL face (STIX Two Math, Libertinus Math).
2. **Assignment:** `=` as typed (recommended for v1), `←`, or the recurrence 𝑧ₙ₊₁ = 𝑓(𝑧ₙ) — the last
   needs care when `z` is assigned mid-formula and read afterwards.
3. **Products:** juxtaposition with `·` only between numbers (recommended), or `·` everywhere.
4. **Notation that changes the look but not the computation** (`sqr` as ², `recip` as a fraction,
   `cotan` as cot, `|z|` as |𝑧|²): recommended as in §4.5.
5. **Scope:** full Textbook editing (P0–P4), or stop after P2 (typeset display, editing in text).

## 9. Alternatives considered

- **KaTeX or MathJax** produce HTML/SVG for a browser; there is no web view in the app, and embedding a
  JavaScript engine for one dialog is out of proportion.
- **Pure-Rust renderers:** ReX renders LaTeX to SVG/Cairo and is unmaintained; Typst's math layout
  comes with the whole Typst stack. Neither edits, and editing is most of the work.
- **A live preview only** (typeset view above the text field, clicks moving the text caret) is P2 of
  this plan, kept as a milestone rather than the goal, because the request is to edit in textbook form.
- **Drawing glyph outlines as meshes** (`ttf-parser` + a tessellator such as lyon): exact shapes, but
  egui draws meshes without anti-aliased edges; the PUA subset keeps every glyph on egui's
  anti-aliased atlas.
- **Procedurally drawn parentheses** (crescents as triangle strips): no font work, but they would not
  match Computer Modern's shapes, which is the point of the request.
