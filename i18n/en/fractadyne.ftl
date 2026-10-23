### Fractadyne user interface: English, the source language.
###
### Every message here is a constant in the code (build.rs reads this file), and every other
### language's file translates these IDs. Rules (design/multilingual.md §4):
###   - Fixing the English wording keeps the ID.
###   - Changing what a message says changes its ID, so old translations fall back to English
###     rather than saying something the app no longer says.
### The comment above a message is what a translator sees: where it appears, what its arguments are.

## Language data: read by Fractadyne, not shown as messages.

# This language's own name, shown in the language menu.
language-name = English
# The same name in English, shown beside it in the menu.
language-english-name = English
# The writing system the app must be able to draw (an ISO 15924 code: Latn, Cyrl, Jpan, Hans, Khmr).
language-script = Latn
# Punctuation for counts, durations and sizes. Never used for coordinates or magnifications.
number-decimal = .
number-group = ,

## The language menu (top bar) and the Languages window

# The language menu's last item: opens the Languages window.
language-more = More languages…
# The Languages window's title.
language-window-title = Languages
# The Languages window's introduction.
language-window-intro = Fractadyne's interface in another language. A language is one file: it comes with a release, or install one here. Coordinates, files and the command line stay in English in every language.
# Column headers of the installed-language list.
language-col-language = Language
language-col-translated = Translated
language-col-from = From
# Where a language comes from.
language-source-builtin = built in
language-source-package = this release
language-source-user = installed
language-source-dev = working folder
# $percent: how much of the interface the file translates, 0–100.
language-coverage = { $percent }%
# $count: messages in the file that failed the checks and show in English instead.
language-dropped =
    { $count ->
        [one] one message needs checking
       *[other] { $count } messages need checking
    }
# Buttons of the Languages window.
language-install = Install language file…
language-install-hover = Pick a Fractadyne language file (.ftl). It is checked, then copied into the languages folder of your settings, where it stays across updates.
language-open-folder = Open languages folder
language-reload = Reload
language-reload-hover = Read the language files again (after you edit or add one).
language-use = Use
# Notices after installing a file. $name: the language's own name; $file: the file's name.
language-installed = { $name } installed from { $file }.
# $file: the file's name; $why: what was wrong with it.
language-install-failed = { $file } is not a usable language file: { $why }
# Shown when a language file cannot be loaded and English is used instead. $name: the language.
language-load-failed = { $name } could not be loaded, so Fractadyne is in English: { $why }

## Asked once when the system's language changes

# The window's title.
language-changed-title = Language
# $new: the system's new language, in its own name.
language-changed-question = Your system language is now { $new }. Use it in Fractadyne?
# Buttons. $name: a language's own name.
language-changed-use = Use { $name }
language-changed-keep = Keep { $name }

## The welcome window (first run)

welcome-title = Welcome to Fractadyne
welcome-intro = A deep-zoom explorer for the Mandelbrot set and its relatives. A few controls to get you moving:
# The language row, at the top of the window.
welcome-language = Language
# Controls: the key or gesture on the left, what it does on the right.
welcome-key-drag = drag
welcome-key-drag-does = pan the view
welcome-key-scroll = scroll
welcome-key-scroll-does = zoom in / out toward the cursor
welcome-key-space = hold Space
welcome-key-space-does = smooth continuous zoom (Shift+Space out)
# Shown after the click-zoom toolbar icon.
welcome-key-click = then click
welcome-key-click-does = dive into the clicked point (toolbar)
welcome-key-m-does = jump to the nearest minibrot at its own scale
welcome-landmarks = Jump to a landmark to start:
welcome-theme = Theme
welcome-watermark = Show "Fd" mark
welcome-watermark-hover = The small brand mark in the lower-right of the view. Off removes it from the live view and from exported images.
welcome-updates = Updates
welcome-check-updates = Check for updates
welcome-check-updates-hover = Asks GitHub for the latest release when the app starts. Nothing is uploaded.
welcome-files = Files
welcome-no-config = (no config directory)
welcome-open = Open
welcome-rendering-on = Rendering on
welcome-open-help = Open full help  (F1)
# $icon: the Settings icon.
welcome-more-in-settings = More in File → { $icon } Settings
welcome-get-started = Get started
