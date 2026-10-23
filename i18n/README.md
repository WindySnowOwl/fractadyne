# Translating Fractadyne

Fractadyne's interface and Help are translated through one file per language:
`i18n/<tag>/fractadyne.ftl`, in [Fluent](https://projectfluent.org/fluent/guide/) syntax. The
English file, `i18n/en/fractadyne.ftl`, is the source: every message the program shows is there,
with a comment saying where it appears and what its arguments are. A translation is that file with
the values in another language. Contributions arrive as pull requests.

The design behind this is `design/multilingual.md`.

## Starting a language

1. Copy `i18n/en/fractadyne.ftl` to `i18n/<tag>/fractadyne.ftl`. The tag is a BCP 47 language tag:
   `de`, `fr`, `es`, `uk`, `ja`, `zh-Hans`.
2. Replace the header's `###` lines with the language's own, and set the language data at the top:

   ```ftl
   language-name = Deutsch
   language-english-name = German
   language-script = Latn
   number-decimal = ,
   number-group = .
   language-status = draft
   ```

   `language-script` is the ISO 15924 code of the writing system (`Latn`, `Cyrl`, `Jpan`, `Hans`).
   A space as the digit-group separator is written as a quoted literal so it can be seen:
   `number-group = { " " }` (narrow no-break space) or `{ " " }` (no-break space).
   `language-status` stays `draft` until a native speaker has reviewed the whole file; the
   language menu shows "(draft)" beside the name until then.
3. Translate the values. Keep everything else as it is.

## Rules

- **Keep the IDs, the comments and the layout.** Only the text after `=` (and on the indented lines
  below it) changes. Keeping each comment and blank line where the English file has them lets a
  reviewer read the two files side by side, line for line.
- **Arguments.** `{ $name }` is a value the program fills in. Keep every one the English message
  has, spelled exactly; move it wherever your language needs it. Never add one the English message
  does not have: the program sets such a message aside and shows the English one.
- **Plurals.** A selector on a count chooses a variant by your language's plural category:

  ```ftl
  cache-cleared =
      { $count ->
          [one] Ein Orbit entfernt.
         *[other] { $count } Orbits entfernt.
      }
  ```

  Use your language's categories (CLDR): German, Spanish, Italian, Turkish: `one`, `other`; French,
  Portuguese: `one` (0 and 1), `other`; Ukrainian, Russian, Polish: `one`, `few`, `many`, `other`;
  Japanese, Chinese, Korean, Indonesian: `other` only (write the sentence once, without a selector,
  or keep a selector with `*[other]` alone). Exactly one variant is the default, marked `*`.
- **Selectors on a word.** Some selectors choose by a token, not a number (`$kind`, `$channel`,
  `$track`, `$end`): keep their variant keys exactly as in English (`[beta]`, `[device]`) and
  translate the text of each variant.
- **Leave as written:** the program's name (Fractadyne), other programs and services (Kalles
  Fraktaler, Fractint, GitHub, MPFR…; but a path through another program's settings, such as the
  NVIDIA Control Panel's, follows that program's own translation, as its comment says), unit symbols (px, ms, MB, ×, %), file
  formats and extensions (PNG, EXR, .fdn, .kfr), key names (Ctrl, Shift, Space, F1, Esc), formula
  notation and function names (`z^2 + c`, `sin`, `conj`, `p1`), command-line options
  (`--worker-gpu`), and anything a comment says to keep.
- **Names of fractals and places.** Mandelbrot, Julia, Burning Ship, Misiurewicz: use the name
  mathematicians in your language use (often the original, or its transliteration).
- **Short where the comment says so.** The status bar's words sit in fixed slots; the live-render
  panel's legends are engraved in capitals and must stay short (write them in capitals, as the
  English file does: the program never uppercases a translation, so Turkish İ and ı and German ß
  come out as you write them); menu items and buttons should stay close to the English length.
- **Special characters.** A literal `{` or `}` is written `{"{"}` / `{"}"}`. An indented line that
  begins with `[`, `*` or `.` must begin with a literal instead: `{"["}Ctrl]`, `{"."}fdn`. A blank
  line inside a value is a paragraph break.
- **No links.** A message may not contain an address (`https://…`, `www.`): the program sets it
  aside. Links are in the code.
- **Typography of your language.** Use its quotation marks, apostrophes and spacing: „…“ (German),
  « … » with no-break spaces (French, Ukrainian), «…» (Spanish), 「…」 (Japanese), “…” (Chinese);
  French puts a narrow no-break space (U+202F) before `;` `!` `?` and a no-break space (U+00A0)
  before `:`. Japanese and Chinese use full-width punctuation (、。，：) and no spaces between
  words.

## Checking

```sh
python scripts/check_i18n.py i18n/de/fractadyne.ftl
```

It checks the file as the program does when it loads one (arguments, plural categories, length,
characters, links, the language data) and prints the coverage. Fix every error; a warning about a
missing plural category means some counts will read the default variant.

To see the translation in the program while you work, point it at the folder:

```sh
FRACTADYNE_LANG_DIR=i18n fractadyne --language de
```

`fractadyne --check-language i18n/de/fractadyne.ftl` loads the file exactly as the program will and
lists any message it would set aside, and why.

## Reviewing a draft

A file marked `language-status = draft` was not written by a native speaker, or not checked by one.
Reading it next to the English file, correcting what reads wrong, and changing the status to
`reviewed` when the whole file has been read is the most useful contribution a speaker of the
language can make.
