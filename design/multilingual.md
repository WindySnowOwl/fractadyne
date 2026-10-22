# Multilingual support — design

Status: **design (2026-10-22). Nothing built.** Line references are to `main` at `5f9193d1`.

The goal: Fractadyne's interface in more languages than English, with each language held in **one
file that is easy to distribute and to update** — shipped with a release, installable by hand,
refreshable without a new app release, and editable by a translator with a text editor.

In short:

- **One plain-text file per language**, in the Fluent format (`.ftl`, §3): named messages, the
  translator's own plurals and word order, and the language's own number punctuation inside the file.
- **The file stands alone.** It works with any app version: messages the app does not know are
  ignored, and messages the file lacks fall back to English one by one. A half-finished translation
  is usable, and an old file keeps working after an app update (§4).
- **Three ways in** (§5): the release package, a file installed from Settings, and an opt-in
  online update that verifies a signature before installing anything.
- **English stays the language of everything that is data or diagnostics** (§2): coordinates,
  magnifications, `.fdn` files, the command line, logs and crash reports.

Most of the work is not the loader. It is moving about 2,700 inline English strings into the
English file, and first untangling the places where English text is also an identifier (§7).

## 1. What exists today

There is no localization of any kind: no string table, no language setting, no locale detection.

**Text in the interface.** About 2,500–2,800 user-facing strings, all inline literals in the
`fractadyne-app` crate. The largest holders:

| Where | Text |
|---|---|
| `ui/dialogs.rs` | ~310 strings: welcome, go-to, snapshot, share, report issue, crash prompt, export, notices |
| `ui/menus.rs` | ~180: the menu bar (:236), File ▸ Settings (:336), the status bar (:1477) |
| `main.rs` | ~470: gradient editor (:11510), Help window (:13770), update dialog (:13948), enum labels |
| `help.rs` | ~450: the in-app manual (12 sections) and the CLI reference (:680) |
| `blinken/panel.rs`, `blinken/help.rs` | ~260: the live-render panel's legends and help cards |
| `ui/panels.rs`, farm, tour, Life, L-system, formula dialogs | the rest |

The library crates add ~520 more as error messages that reach the screen: coordinate-expression
errors (`core/bignum.rs`), formula parse errors (`core/ir/parse.rs:46`), palette import errors
(`color/import.rs`), the text crate's repair names (`text/lib.rs:92–117`).

**Habits that do not survive translation.**

- ~224 `format!` calls splice a value into an English sentence ("Imported {m} of {n} swatches…",
  `main.rs:10854`); other languages need another word order.
- Plurals are English `if n == 1` logic (`ui/orbit_cache.rs:207`, `formula_library.rs:349`, ~10
  sites); Polish, Russian and Arabic have three to six plural forms.
- Icons and shortcuts are glued into labels (`format!("{}  Settings", icons::SETTINGS)`,
  `"Snapshot  (Ctrl+S)"`, `menus.rs:244–336`).
- Counts are grouped with a hard-coded comma (`commas`, `main.rs:3742`; four copies of the idea).

**English text that is also an identifier** (would break if translated):

- **Fractal family names** are the keys in `session.toml`, `.fdn` (`fractal=`), the command line,
  scripting and export file names (`FractalKind::from_name`, `fractal.rs:432`) — and the same
  strings are shown in the menus.
- **Help sections** are looked up by English name (`section_index`, `help.rs:1180`).
- **The command line's flag list** is scraped from the CLI reference's English descriptions
  (`cli.rs:19–35`).
- **egui widget identity** comes from visible titles: none of the 35 `Window::new` calls sets an
  `.id()`, and ten `CollapsingHeader`s in `panels.rs` have no `id_salt`. Switching language would
  reset their open/closed state and position, and two titles translated alike would collide.
- **Harnesses match English**: uitest finds status-bar readouts by prefix (`uitest.rs:2083`),
  `logcheck-rules.toml` matches log text, ~127 test assertions use `contains("…")`.

**Fonts.** The interface fonts (Spline Sans, Spline Sans Mono, the live panel's Routed Gothic,
Barlow Condensed and Share Tech Mono; `theme.rs:254`, `blinken/paint.rs:516`) cover Latin
scripts. Greek and Cyrillic draw through egui's Ubuntu/Hack fallback, in a different typeface.
Nothing covers Chinese, Japanese, Korean, Arabic, Hebrew, Devanagari or Thai, and egui 0.31 does
no complex shaping and no bidirectional layout.

**Network.** One opt-in path: the update check (`update.rs`, ureq + rustls, to
`api.github.com`), off by default (`update_check_on_launch`), with links vetted by
`safe_release_url` (`update.rs:19`).

**Text hygiene.** `fractadyne_text::clean` repairs pasted data: it turns typographic quotes and
dashes into ASCII and strips zero-width joiners. That is right for coordinates and wrong for
translations, which need „“ « » and – and (in some scripts) ZWJ. Translations get their own rules
(§6).

## 2. What is translated

**Translated:** everything in the window — menus, panels, dialogs, tooltips, notices, the status
bar's labels, the live-render panel, the in-app Help, file-dialog titles and filters, and error
messages shown to the user.

**Not translated, deliberately:**

- **Data.** Coordinates, magnifications (`1.2e148×`, `2^93`), iteration counts typed into fields,
  anything that can be copied out and pasted back in. These keep `.` as the decimal point in every
  language, so a view copied from a German interface pastes into a Japanese one.
- **Files.** `.fdn` keys and values, `session.toml`, palettes, formulas, tours. A file written in
  one language opens in every other.
- **The command line**, `--help`, scripting output, logs, `[fd-…]` lines and crash reports. They
  are parsed by scripts and by `--logcheck`, and read when helping someone; one language for them
  keeps both working.
  The command-line reference inside Help is the same table as `--help` (`CLI_REFERENCE`,
  `help.rs:680`) and stays English with it: it documents English flags.
- **The changelog and the documents in the package** (README, DIAGNOSTICS, TOURS). The website is
  the better home for translated documentation, later.

Formula keywords stay as they are (they are the language of the formula, not of the interface).

## 3. The file format: Fluent

[Fluent](https://projectfluent.org/) (Mozilla's localization system) fits the brief best. A
Fluent file is plain UTF-8 text, one per language:

```ftl
### Fractadyne user interface, English (the source language).

## Language data: read by Fractadyne, not shown as messages.
# This language's own name, shown in the language list.
language-name = English
# The writing system the app must be able to draw (ISO 15924).
language-script = Latn
# Punctuation for counts, durations and sizes. Never used for coordinates or magnifications.
number-decimal = .
number-group = ,

## Menus
menu-file = File
menu-file-snapshot = Snapshot

## Notices
# Help ▸ Reference cache ▸ Clear. $count: the orbits removed.
toast-orbit-cache-cleared =
    { $count ->
        [one] Reference cache cleared: one orbit removed.
       *[other] Reference cache cleared: { $count } orbits removed.
    }
# After a palette import. $done, $total: swatches; $file: the file's name.
toast-palette-imported = Imported { $done } of { $total } swatches from { $file }.
```

The same messages in German and Polish — each translator decides the plural forms and the word
order, with no change to the code:

```ftl
toast-orbit-cache-cleared =
    { $count ->
        [one] Referenz-Cache geleert: ein Orbit entfernt.
       *[other] Referenz-Cache geleert: { $count } Orbits entfernt.
    }
toast-palette-imported = { $done } von { $total } Farbfeldern aus { $file } importiert.
```

```ftl
toast-orbit-cache-cleared =
    { $count ->
        [one] Wyczyszczono pamięć orbit: usunięto jedną orbitę.
        [few] Wyczyszczono pamięć orbit: usunięto { $count } orbity.
       *[many] Wyczyszczono pamięć orbit: usunięto { $count } orbit.
    }
```

Why Fluent:

- **Named messages and named arguments.** The code asks for `toast-palette-imported` with `done`,
  `total` and `file`; the translator puts them where the language wants them. No positional
  `%1$s`, no sentence built from fragments.
- **Plurals belong to the translation.** Plural categories come from Unicode's CLDR rules for the
  file's language (`one`, `few`, `many`, `other`…), chosen by the translator per message.
- **A broken message breaks only itself.** The parser recovers per message; the app falls back to
  English for that one message and keeps the rest of the file. (The project's rule for user data
  — repair, don't reject — applied to translations.)
- **Comments travel with messages.** The `#` line above a message is shown to translators as its
  context (where it appears, what the arguments are, how short it must be).
- **Rust-native and permissively licensed.** `fluent-bundle` is Apache-2.0 or MIT
  ([licenses.dev](https://licenses.dev/cargo/fluent-bundle/0.15.3)); plural rules come from
  `intl_pluralrules`, language matching from `fluent-langneg`.

The alternative is **gettext PO**, the most widely supported format: every translation tool reads
it, and Weblate supports it best ([Weblate formats](https://docs.weblate.org/en/latest/formats.html)).
Against it here: messages are keyed by their English text, so fixing an English typo orphans every
translation of it; plurals are a C-like expression in the file header; arguments are positional;
and the runtime reads compiled `.mo` files, a second artefact to distribute. **Weblate's Fluent
support is monolingual and has no plural editor**
([Weblate: Fluent](https://docs.weblate.org/en/latest/formats/fluent.html)), so translators there
type the `{ $count -> … }` selector as text; Weblate does check its syntax. That cost is real but
small for an interface with a few dozen plural messages. **Recommendation: Fluent.** (Decision 1.)

**Numbers.** The app formats numbers itself, with the separators the file declares
(`number-decimal`, `number-group`), and hands the result to Fluent. Plural selection still sees the
number: Fluent gets the value, and a custom formatter (`FluentBundle::set_formatter`, to confirm in
the prototype) prints it with the file's punctuation. Fluent's own `NUMBER()` options are not relied
on. Grouping is in threes; languages that group otherwise (Hindi's 2-2-3) are a later extension.

**Bidirectional isolation off.** Fluent wraps arguments in Unicode isolation marks (FSI/PDI) by
default; egui would draw them as boxes. The loader turns that off (`set_use_isolating(false)`)
until right-to-left scripts are supported (§8).

## 4. The language file

The file is the unit everything else moves around: what a translator edits, what a release ships,
what a user installs and what an update replaces. Its properties:

- **One file per language**, `fractadyne-<tag>.ftl`, where the tag is a BCP 47 language tag:
  `de`, `fr`, `pt-BR`, `zh-Hans`. About 3,000 messages, ~200–300 KB of text. In the repository
  they live at `i18n/<tag>/fractadyne.ftl` (the layout Weblate expects), the English one at
  `i18n/en/fractadyne.ftl`.
- **Plain UTF-8 text**, LF line endings, no BOM. It diffs and reviews in git, opens in any editor,
  and needs no build step: the app reads the `.ftl` itself.
- **Self-describing.** The language-data messages at the top name the language (in itself), its
  script, and its number punctuation. Nothing about a language lives in the app's code, so adding
  a language is adding a file.
- **Independent of the app's version.** The app never refuses a file for being old or new. It
  formats each message from the file if the file has it and it passes the checks of §6, and from
  English otherwise. A file made for 0.3.0 works in 0.5.0 with English for whatever is new, and a
  file made for 0.5.0 works in 0.3.0 with its extra messages ignored.
- **Partial is fine.** Coverage is shown in the language list ("Deutsch — 94%"), and a language
  can ship at any coverage. Below a threshold (say 50%) it is listed as incomplete.
- **A publish stamp.** When a file is published (§5) a script adds a header of `###` comments,
  which Fluent ignores and the app reads:

```ftl
### fractadyne-language-file: 1
### tag: de
### revision: 42
### checked-against: 0.3.0-beta.24 (2,734 messages, 2,571 translated)
```

  `revision` is a counter per language that only grows; when two copies of a language are
  installed, the higher revision wins (§5). The copy in the repository carries no stamp, so
  Weblate's commits and hand edits do not fight over it.

**Message IDs are an API.** An ID names a meaning, not a wording, grouped by area: `menu-`,
`panel-`, `dlg-<dialog>-`, `toast-`, `status-`, `help-`, `live-` (the front panel), `err-`,
`file-dialog-`, `fractal-` (family display names). Two rules keep old translations honest:

1. **Fixing English wording keeps the ID.** A typo fix or a clearer phrasing leaves translations
   in place.
2. **Changing what a message says changes the ID** (`dlg-export-gpu-note` → `dlg-export-gpu-note-2`).
   Every translation of the old meaning stops matching and that message shows in English until it
   is translated again — English where a translation is missing, never a translation of something
   the app no longer says.

## 5. Distribution and updates

**Where the app looks for language files**, for each language taking the highest revision found:

1. `<config>/languages/` — files the user installed or the updater downloaded
   (`%APPDATA%\Fractadyne\Fractadyne\config\languages\` on Windows). They survive installing a new
   release over the old one.
2. `<folder of the executable>/languages/` — the files shipped in the release package.
3. English, compiled into the executable — always present, so the app never starts without text.

A translator can also point `FRACTADYNE_LANG_DIR` at a working folder; the app then watches that
file and reloads it when it is saved, so a translation can be checked on screen as it is written,
with nothing but the released app.

**Ways a file arrives:**

- **With a release.** `release.yml` copies every `i18n/<tag>/fractadyne.ftl` into the package's
  `languages/` folder, stamped (§4). Every release carries every language as it stands that day.
- **By hand.** Settings ▸ Language ▸ **Install language file…** picks a `.ftl`, checks it (§6), and
  copies it into `<config>/languages/`. Dropping the file into that folder works too. A hand-
  installed file is listed as "installed from a file" (not signed).
- **Online, opt-in.** With update checks on (the existing `update_check_on_launch`, or a **Check
  now** button in Settings ▸ Language), the app fetches a signed index of the published languages,
  downloads any file whose revision is newer than the one installed, verifies it, and installs it
  into `<config>/languages/` (written to a temporary file and renamed, as session saves are). A
  notice says what changed: "German updated (revision 42, 97%)". Nothing is fetched unless the user
  turned update checks on.

**The index** is a small JSON file published with the language files:

```json
{
  "format": 1,
  "generated": "2026-11-02T10:00:00Z",
  "languages": [
    { "tag": "de", "name": "Deutsch", "script": "Latn", "revision": 42,
      "translated": 2571, "of": 2734,
      "file": "fractadyne-de.ftl", "size": 214532, "sha256": "…" }
  ]
}
```

**Signing.** The index is signed with an Ed25519 key held by the maintainer (a CI secret); the
app carries the public key and verifies the index before reading it, then each file against the
index's `sha256`. A file that fails is not installed. The `minisign` signature format and its small
verify-only crate (`minisign-verify`) are enough. With signatures the host only has to be
available, not trusted.

**The host** (decision 3): either GitHub — the files as assets of a standing `languages` release,
on the same host the update check already talks to — or fractadyne.org, which deploys with one
`git push`. A publish script builds the stamped files and the signed index from `i18n/` and pushes
them; it runs when translations change, independent of app releases.

**Translations coming in** (decision 2): either Weblate (hosted, free for libre projects; component
file mask `i18n/*/fractadyne.ftl`, base file `i18n/en/fractadyne.ftl`, format "Fluent"), which
commits to the repository, or pull requests on GitHub. Both end as commits to `i18n/`, checked by
CI (§9).

## 6. Checks on every file the app reads

A language file is untrusted text: anyone can write one, and the updater downloads them. Before a
file's messages are used:

- **Size and encoding.** At most 2 MB; UTF-8 (a BOM is dropped); line endings normalized.
- **Characters.** C0/C1 control characters other than newline and tab are removed. Bidirectional
  embedding and override controls (U+202A–U+202E) and isolates (U+2066–U+2069) are removed: the
  interface draws left-to-right only, and these are the characters that make text display in
  another order than it reads (the "Trojan Source" trick). Typographic quotes, dashes, non-breaking
  and narrow no-break spaces, and ZWJ/ZWNJ are kept: they are language, not paste damage.
  `fractadyne_text::clean` is not used for translations.
- **Per message:**
  - it parses (Fluent recovers per message; a message that does not parse is dropped);
  - every argument it uses is one the English message declares — a translation that refers to
    `{ $path }` where English has none would show a raw placeholder, so the message falls back;
  - it is at most four times the English length plus 64 characters, so a runaway message cannot
    push a dialog off the screen;
  - it contains no URL. Links in the interface take their targets from the code (as
    `safe_release_url` does for the update dialog), so a language file can change a link's words
    but never where it goes.
- **The language-data messages** are checked for sense (a single separator character each; a
  known script code) and replaced by English's when they fail.

Dropped messages are counted, not hidden: the language list shows "94% (12 messages need
checking)", and `FRACTADYNE_TRACE=lang` lists them.

## 7. In the code

**Calling a message.** A macro returns the text in the current language:

```rust
ui.label(tr!(panel_iterations));
ui.button(tr!(menu_file_snapshot));
self.set_toast(tr!(toast_orbit_cache_cleared, count = removed));
self.set_toast(tr!(toast_palette_imported, done = m, total = n, file = name));
```

A build script reads `i18n/en/fractadyne.ftl` and generates one constant per message, so a
misspelt or deleted ID fails to compile. A unit test formats every English message with the
arguments it declares, and a lint (in the style of `check_test_layout.py`) flags string literals
passed to widgets in the GUI crate, so new English text cannot be added outside the file. (The
`i18n-embed-fl` crate's `fl!` macro checks arguments at compile time as well; its loader brings
asset-embedding assumptions this design does not need, so it is an option, not the plan.)

**Cost per frame.** egui rebuilds the interface every frame, so `tr!` runs thousands of times a
second. Messages without arguments — most of them — are formatted once when a language loads and
kept in a table indexed by the generated constant: a lookup, no formatting. Messages with
arguments are formatted at the call, as `format!` is today.

**Switching language** happens on the next frame, without a restart: the active catalog sits
behind one shared pointer, and a generation counter tells the caches that hold laid-out text (the
live panel's tessellated static layer, measured label widths) to redo it.

**Choosing a language.** A new setting, `language` in `session.toml`: `"auto"` (the default) or a
tag. `auto` follows the system's preferred languages (`GetUserPreferredUILanguages` on Windows,
`LANGUAGE`/`LANG` on Linux), matched against the installed files with `fluent-langneg`: `fr-CA`
takes `fr`, `zh-Hant-TW` takes `zh-Hant`, and anything without a match takes English. Settings ▸
Language lists every installed language in its own name with its coverage and source.

**Groundwork before any translation** (no visible change in English):

1. **Fractal family names**: the English names stay the persisted tokens forever (`session.toml`,
   `.fdn`, the command line and scripting keep reading and writing them); the menus show
   `fractal-<token>` messages instead.
2. **Help sections** are found by an enum, not by name.
3. **Every `Window`, `CollapsingHeader` and `ComboBox` gets an explicit ID**, so what a widget is
   does not depend on what it says.
4. **Icons and shortcuts** are composed by the code around the translated label, not inside it.
5. **Number formatting** goes through one module (counts, durations, sizes, percentages) that
   takes its punctuation from the active language. Coordinates and magnifications keep their
   formatters and their `.`. Integer fields accept the language's grouping character as well as
   `,`, `.` and spaces (integers have no decimals to confuse); decimal fields keep `.`.
6. **Library errors** (`core`, `color`, `gpu`, `text`) become error values with arguments, turned
   into words in the app; the command line formats them in English.
7. **The live panel's legends** are written uppercase in the file rather than uppercased by
   `to_uppercase` (which mishandles Turkish ı/İ and German ß); their width estimate
   (`chars().count() * 6.4`, `panel.rs:1161`) becomes a measured width.

## 8. Fonts and scripts

- **Latin-script languages** (German, French, Spanish, Italian, Portuguese, Dutch, Polish, Czech,
  Turkish, Vietnamese…) draw with today's fonts; Spline Sans lacks a few Latin Extended letters,
  which fall back to Ubuntu. A test (extending `icons_coverage.rs`) checks every character of every
  shipped file against the interface fonts.
- **Greek and Cyrillic** draw today through the Ubuntu fallback. Before a Greek or Cyrillic
  language ships, add a sans with those scripts that sits well beside Spline Sans (a Noto Sans
  subset, OFL).
- **Chinese, Japanese and Korean** need a font the package does not have, and a full CJK font is
  5–30 MB. The plan: load the system's font when the language needs it (`language-script` in the
  file says which), and only then. Windows 10 and 11 ship Microsoft YaHei, Microsoft JhengHei, Yu
  Gothic and Malgun Gothic (to confirm on a clean install of each); Linux finds Noto CJK through
  fontconfig. egui reads font collections (`.ttc`) by index. Where no font is found, the language
  is listed as unavailable with the reason, rather than shown as boxes. A subset font published
  with the language file (only the characters it uses, made with the same fontTools subsetter the
  icons use) is the fallback if system fonts prove unreliable.
- **Right-to-left and complex scripts** (Arabic, Hebrew, Persian, Hindi, Thai) are out of scope:
  egui does no shaping and no bidirectional layout. The file format does not shut them out; the
  app lists such a file as unavailable, with the reason.
- The math font (formula editor) and the live panel's display fonts are Latin-only; translated
  text in them falls back to the interface fonts, so a front panel in Japanese loses its period
  typeface but stays legible.

## 9. Testing

- **A pseudo-language.** `en-XA`, generated from English at build time: accented letters, text
  40% longer, and brackets around every message — `[Ŝñåþšĥöţ ~~~]`. Anything on screen without
  brackets is still hard-coded English; anything cut off will be cut off in German.
- **uitest in the pseudo-language.** The 69 screens of `--uitest`, photographed in `en-XA` as well
  as English, and LOOKED at; a new check for labels clipped inside dialogs, which uitest does not
  test today (it checks regions, not text). The status bar's fixed character widths
  (`zoom_slot_width`, `main.rs:3883`) become measured widths, or its labels become short symbols —
  it must still fit one line, or the canvas resizes and the renderer restarts.
- **CI** (`scripts/check_i18n.py`, run with the text-hygiene check): every file in `i18n/` parses;
  every argument a translation uses is declared in English; no forbidden characters (§6); every
  character is in the fonts; a coverage table per language; the English file has no message the
  code never asks for.
- **Harnesses stay in English.** uitest's readout prefixes, `logcheck` rules and test assertions
  keep matching English, because the harnesses run with `language = "en"`.

## 10. Plan

| Phase | What | Visible |
|---|---|---|
| P0 groundwork | §7's seven items; the number-formatting module | nothing (English as before) |
| P1 loader | `fluent-bundle`, the generated IDs and `tr!`, the English file with the menus moved into it, the `language` setting, `en-XA`, live switching, §6's checks | Settings ▸ Language with English and the pseudo-language |
| P2 extraction | the remaining ~2,700 strings, area by area (menus, panels, dialogs, notices, status bar, live panel, then Help); plurals and word order fixed as each moves; the lint switched on | nothing in English |
| P3 distribution | `languages/` in the packages; Install / Reload / Check now; the publish script, signed index and host; CI checks; Weblate if chosen | language files installable and updatable |
| P4 first languages | the first translations (decision 4), then system-font loading for CJK | the first languages |

P2 is most of the work, and it parallelizes by file. P0 and P1 are small and can land behind no
switch at all, since English output does not change.

## 11. Decisions needed

1. **Format:** Fluent (recommended, §3) or gettext PO.
2. **How translations come in:** Weblate (hosted, translators need no git) or pull requests only.
3. **Online language updates:** the opt-in, signed update of §5 (recommended), or files by hand
   only. And the host: GitHub (the update check's host) or fractadyne.org.
4. **First languages**, and whether Chinese, Japanese or Korean is among them (that brings §8's
   font work into the first round).
5. **Scope of the first round:** all of Help (~450 strings, the manual) and the live panel, or the
   interface first and Help after.
6. **First run:** follow the system language (recommended), or English until the user picks one.
