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
# One character each. A space as the group separator is written as a quoted literal, so it shows:
# { "\u00A0" } (no-break space) or { "\u202F" } (narrow no-break space).
number-decimal = .
number-group = ,
# Whether a native speaker has reviewed the translation: draft (the language menu says so) or
# reviewed. The English file is the source.
language-status = source

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
# A language whose writing system needs a font this computer does not have (Japanese or Chinese without a system font). $name: the language, in its own name.
language-font-missing = No font that can draw { $name } was found on this computer, so it cannot be shown. Install a { $name } font (for example the system's own language pack) and choose it again.
# Shown after a draft translation's name in the language menu and the Languages window. Short.
language-draft = draft
language-draft-hover = A draft: this translation has not yet been reviewed by a native speaker. Corrections are welcome, as a pull request to the language's file.
# The heading over the decimal-mark choice, in the language menu and the Languages window.
language-decimal-mark = Decimal mark
language-decimal-mark-hover = How numbers are written: times, sizes and counts. Coordinates and magnifications always use a point, as files and other programs read them.
# $sample: a number written with this language's own punctuation (1,234.5).
language-decimal-language = As the language writes it ({ $sample })
# $sample: a number written with a decimal point (1,234.5).
language-decimal-point = Point ({ $sample })
# $sample: a number written with a decimal comma (1.234,5).
language-decimal-comma = Comma ({ $sample })

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

## Menus: shared

# A percentage in a menu (UI scale). $value: the number.
common-percent = { $value }%
# A state shown after a menu item's name, in brackets.
menu-state-connected = connected
menu-state-rendering = rendering
menu-state-running = running

## Navigate ▸ Bookmarks (also the toolbar's star)

menu-bookmarks = Bookmarks
menu-bookmarks-add = Add current view
menu-bookmarks-manage = Manage…

## The fractal picker (Fractal menu and the toolbar's dropdown)

menu-family-powers = Higher powers
menu-family-automata = Automata
menu-family-lsystems = L-systems
menu-fractal-custom = Custom formula…
menu-fractal-library = Formula library…
menu-fractal-library-hover = Saved custom formulas: apply, edit, import and export them
# The "Show" group: which picture of the formula is drawn. $name: the formula's name (Mandelbrot, Burning Ship…).
menu-show = Show
menu-show-set = { $name } set
menu-show-set-hover = The parameter plane — the { $name } set itself.

    Every point of it is a different Julia set; this is the map of all of them.
menu-show-julia = Julia set
menu-show-julia-hover = One point of the { $name } set, drawn as its own picture — the shape you get by holding that point fixed.

    Pick the point by hovering the map in "Both, linked"; a click there pins it.
menu-show-julia-none = { $name } has no free parameter, so it has no Julia set.
menu-show-both = Both, linked
menu-show-both-hover = The { $name } set on the left, and on the right the Julia set of whatever point your cursor is over.

    Move the cursor across the map and watch the Julia change — this is the quickest way to see how the two relate.
menu-show-both-none = { $name } has no Julia set to pair with.

## File menu

menu-file = File
menu-file-open-view = Open view or location…
menu-file-gallery = Gallery…
menu-file-export = Export image…
menu-file-snapshot = Snapshot
menu-file-open-life = Open Life pattern…
menu-file-open-life-hover = An RLE, .cells or Life 1.05/1.06 file, shown as a Life universe.
menu-file-life-text = Life pattern text…
menu-file-life-text-hover = Paste a pattern as text (RLE as LifeWiki gives it).
menu-file-save-life = Save Life pattern…
menu-file-save-life-hover = The universe as it is now, as RLE.
menu-file-open-lsystem = Open L-system…
menu-file-open-lsystem-hover = A Fractint .l file (its entries to pick from) or a .lsys system.
menu-file-save-lsystem = Save L-system…
menu-file-save-lsystem-hover = The system's text, as a .lsys file.
menu-file-export-svg = Export SVG…
menu-file-export-svg-hover = The view as an SVG drawing: its lines and shapes as vectors, in its colours.
menu-file-render-client = Render client…
menu-file-render-client-hover = Lend this machine's GPU to a render farm: connect to a controller (Tools ▸ Render on farm… on another machine) and render frames for it.
menu-file-settings = Settings
menu-file-reset = Reset application state…
menu-file-reset-hover = Delete all saved state (session, bookmarks, thumbnails) and start fresh. Asks for confirmation first.
menu-file-quit = Quit

## File ▸ Settings

menu-settings-fps = Frame-rate cap
menu-settings-fps-uncapped = Uncapped
# $fps: frames per second.
menu-settings-fps-n = { $fps } FPS
menu-settings-ui-scale = UI scale (font size)
menu-settings-theme = Theme
menu-settings-updates = Updates
menu-settings-updates-stable-hover = Latest stable release
menu-settings-updates-beta-hover = Latest build including pre-releases
menu-settings-updates-on-launch = Check for updates on launch
menu-settings-updates-on-launch-hover = Otherwise, check manually via Help → Check for updates.
# Followed by its shortcut, (Ctrl+S).
menu-settings-snapshot = Snapshot
menu-settings-snapshot-ask-hover = Offer the choice on every press
menu-settings-snapshot-screen-hover = Save the view exactly as it appears on screen (instant, PNG)
menu-settings-snapshot-render-hover = A full render at the Export settings, in the background (can take a long time on deep views)
menu-settings-refcache = Reference cache…
menu-settings-refcache-hover = Deep-zoom reference orbits kept on disk, so a location you return to renders in seconds. Where it is, how big, the limit, and a way to clear it.

## Fractal menu

menu-fractal = Fractal

## View menu

menu-view = View
menu-view-zoom-home = Zoom out to full view
menu-view-zoom-home-hover = Zoom out to the full home view (animated)
menu-view-reset = Reset to default view
menu-view-reset-hover = Reset to the fractal's default view (instant)
# Followed by its shortcuts.
menu-view-undo = Undo view
menu-view-redo = Redo view
menu-view-control-panel = Control panel
menu-view-control-panel-hover = Show/hide the right-hand control panel.
menu-view-minimap = Minimap overview
menu-view-minimap-hover = Show a small home-view overview with a "you are here" marker and the zoom depth. Click it to jump to a region.
menu-view-perf-panel = Performance panel
menu-view-timestamp = Show timestamp
menu-view-timestamp-hover = Draw a large elapsed-time clock over the view. It reads the same "+12.345s" the log stamps every line with, so a screen recording of a problem that only shows itself in motion can be lined up against the log frame by frame. Off by default.
menu-view-zoom-target = Show auto-zoom target
menu-view-zoom-target-hover = While the auto-zoom runs, outline the region of the view it is zooming into — the part that will fill the screen after a few more magnifications — with lines from its corners to the corners of the screen, and a ring on the point it is steering toward. Off by default.
menu-view-live-panel = Live render panel
menu-view-live-panel-hover = A front panel over the view that shows the live renderer at work: what each frame dispatched, how old the picture on screen is, the walk through the iteration bands, the reference builders, and the machine underneath. B cycles it, and the × in its corner closes it; drag it to any corner; hover an instrument for what it means and what healthy and troubled look like.
# The live render panel's sizes.
menu-view-live-off = Off
menu-view-live-off-hover = No panel.
menu-view-live-strip = Strip
menu-view-live-strip-hover = The nixie readouts and the light bank.
menu-view-live-panel-size = Panel
menu-view-live-panel-size-hover = The instruments: walk dial, picture age, tiles, GPU passes, depth dial, lamps.
menu-view-live-full = Full
menu-view-live-full-hover = The panel and the engine room: steps, GPUs, memory, cores, pixels.
menu-view-live-opacity = Opacity
menu-view-watermark = Show "Fd" watermark
menu-view-watermark-hover = Draw a discreet "Fd" brand mark in the lower-right corner of the live view and exported images. On by default.

## Color menu

menu-color = Color
menu-color-method = Method
menu-color-palette = Palette
menu-color-custom = Custom
menu-color-duotone = Duotone
menu-color-binary = Binary (set)
menu-color-binary-hover = Flat two-color: in-set vs out-of-set, no gradient.
menu-color-edit-gradient = Edit gradient…
menu-color-normalize = Normalize deep colors
menu-color-normalize-hover = Remap the palette to the view's measured escape range when a dense field would otherwise read as speckle. A view that does not alias is left alone — tick 'Always fit range' to remap regardless. Smooth method only.
menu-color-more = Cycle, offset, animation and effects: Controls panel ▸ Coloring

## Tools menu

menu-tools = Tools
# Followed by its shortcut, (M).
menu-tools-find-minibrot = Find minibrot + zoom to it
menu-tools-find-minibrot-hover = Newton-snap the view center to the nearby minibrot's exact center, report its period, and zoom to the minibrot's own scale — often many orders of magnitude in one step. Already deeper than it? Only the center moves.
menu-tools-solver = Newton / Misiurewicz solver…
menu-tools-solver-hover = Solve for the exact minibrot center (period + size) or the Misiurewicz point nearest the view — Newton's method at full precision, with the repelling multiplier λ. Opens the location dialog, which hosts the solvers.
# Followed by its shortcut, (A).
menu-tools-autopilot = Auto-zoom (autopilot)
menu-tools-autopilot-stop = Stop autopilot
menu-tools-autopilot-hover = Hands-free continuous deep zoom: dives toward the detail-richest region, re-steering as it goes. Any navigation input stops it.
menu-tools-priority = Priority:
menu-tools-priority-speed = Speed
menu-tools-priority-speed-hover = Zoom at the set speed; refresh with the first frame that has detail.
menu-tools-priority-quality = Quality
menu-tools-priority-quality-hover = Fully resolved frames only; the zoom waits for them.
menu-tools-target = Target:
menu-tools-target-detail = Detail
menu-tools-target-detail-hover = The richest edge in view, re-chosen as the dive goes.
menu-tools-target-misiurewicz = Misiurewicz point
menu-tools-target-misiurewicz-hover = The nearest Misiurewicz point, solved once and dived into exactly (Mandelbrot only).
menu-tools-orbits = Orbit overlay
menu-tools-orbits-hover = Draw the iteration path of the point under the cursor. Animation options: Controls panel ▸ Overlays.
menu-tools-orbits-normalize = Normalize (fit to view)
menu-tools-orbits-normalize-hover = Fit the orbit to the whole view so it stays well-framed at any zoom (instead of mapped through the viewport, where it flies off-screen when deep).
menu-tools-play-tour = Play tour…
menu-tools-play-tour-hover = Pick a tour script to play. The toolbar ▶ replays the last one with a single click.
menu-tools-tour-from-view = Tour from current view…
menu-tools-tour-from-view-hover = Create a tour that zooms from the full view down to the current view — add a caption and set the dive duration. Saved as a tour script (.toml) you can edit, play, or render.
menu-tools-render-tour = Render tour…
menu-tools-render-tour-hover = Render the loaded tour to a PNG frame sequence (and optionally an mp4) in a separate process.
menu-tools-render-zoom-video = Render zoom video…
menu-tools-render-zoom-video-hover = Render a dive from the whole set into the current view as a video, from exponential-map strips: much faster than rendering every frame.
menu-tools-close-player = Close tour player
menu-tools-close-player-hover = Stop the tour, close its player, and restore your own iteration and coloring settings
menu-tools-render-farm = Render on farm…
menu-tools-render-farm-hover = Render the loaded tour on several machines: this one controls the job, others join with File ▸ Render client….
menu-tools-benchmark = Benchmark…
menu-tools-benchmark-hover = Measure rendering speed — current settings, or a standardized run (fixed resolution + settings) comparable across machines. Burn-in repeats it to check stability / thermal throttling.
menu-tools-last-benchmark = Show last benchmark

## Navigate menu

menu-navigate = Navigate
menu-navigate-goto = Go to location…
menu-navigate-goto-hover = Enter a center/zoom, jump to a well-known point, or Newton-solve a Misiurewicz / minibrot feature near the view.
menu-navigate-misiurewicz = Misiurewicz explorer…
menu-navigate-misiurewicz-hover = Browse what the small Misiurewicz types look like — thumbnails with each point's zoom period and twist — and Newton-solve any of them to an arbitrary depth. (Mandelbrot.)
menu-navigate-exact = Exact points (expressions)
menu-navigate-exact-note = Roots of the main-cardioid bulbs, given as exact expressions (cos/sin of a fraction of 2π). The coordinate is kept as its expression, so zooming in stays exactly on the point.
menu-navigate-random = Random location
menu-navigate-random-hover = Jump to a random detail-rich boundary point
menu-navigate-import-kfr = Import .kfr…
menu-navigate-import-kfr-hover = Load a Kalles Fraktaler location file
menu-navigate-share = Share location…
menu-navigate-share-hover = Copy / paste / save / load a self-contained location (.fdn): fractal, full-precision center, zoom, coloring.

## Help menu

menu-help = Help
# Followed by its shortcut, (F1).
menu-help-reference = Help & reference…
menu-help-welcome = Welcome / quick start…
menu-help-welcome-hover = The first-run overview: the essential controls and a few landmarks.
menu-help-diagnostics = Diagnostics…
menu-help-diagnostics-hover = Check that Fractadyne works correctly on your hardware, and attach the result to an issue report
menu-help-report = Report an issue…
menu-help-report-hover = File a GitHub issue (or email) with a crash report, log, and location attached
menu-help-accelerated = Faster deep zoom…
menu-help-accelerated-hover = An optional build that computes deep-zoom reference orbits 2.5–6.4× faster. Identical images; your settings carry over.
menu-help-check-updates = Check for updates
# $track: "stable" or "beta", the update track in use (the variants hold the wording).
menu-help-check-updates-hover =
    { $track ->
        [beta] Check GitHub for a newer Beta (pre-releases) build
       *[other] Check GitHub for a newer Stable build
    }
# $version: the new version; $channel: "beta" for a pre-release, "stable" otherwise (the variants hold the wording).
menu-help-update-available =
    { $channel ->
        [beta] { $version } (beta) available — download
       *[other] { $version } (stable) available — download
    }
menu-help-up-to-date = Up to date
menu-help-tagline = Native fractal explorer
menu-help-license = License
# $year and $holder: the copyright year and holder, as the code gives them.
menu-help-copyright = © { $year } { $holder }. Dual-licensed; use under either license at your option.
menu-help-source = Source

## The toolbar (its buttons are icons; these are their tooltips)

menu-toolbar-julia = Julia
menu-toolbar-julia-hover = Show the Julia set of this formula, filling the window
menu-toolbar-dual-hover = Both, linked: the set on the left, the Julia of the cursor's point on the right
menu-toolbar-open-hover = Open a view or location (PNG / EXR, .fdn, .kfr)
menu-toolbar-gallery-hover = Gallery
menu-toolbar-export-hover = Export image…
menu-toolbar-share-hover = Share location — copy, paste, save or load a self-contained .fdn (fractal, full-precision center, zoom, coloring)
menu-toolbar-zoom-in-hover = Zoom in
menu-toolbar-zoom-out-hover = Zoom out
# $factor: the click-zoom factor (4, 10…).
menu-toolbar-click-zoom-hover = Click-to-zoom ({ $factor }×): left-click dives into the point, right-click backs out (drag still pans). Factor in Controls ▸ Navigate.
# Added to the click-to-zoom tooltip in the dual view.
menu-toolbar-click-zoom-dual = Pin the Julia c with Ctrl+click while this is on.
menu-toolbar-autopilot-running-hover = Auto-zoom is running — click to stop
menu-toolbar-autopilot-hover = Auto-zoom: dive toward detail (A)
menu-toolbar-reset-hover = Reset view (instant)
menu-toolbar-home-hover = Zoom out to the home view (animated)
menu-toolbar-bookmarks-hover = Bookmarks: save this view, or jump to a saved one
# $name: the palette's name.
menu-toolbar-next-palette-hover = Next palette ({ $name })
# $file: the last tour's file name.
menu-toolbar-play-last-hover = Play tour — { $file } (click) · replays the last tour; pick another via Tools ▸ Play tour…
menu-toolbar-play-hover = Play a tour… (Tools ▸ Play tour)
menu-toolbar-speed-hover = Playback speed
menu-toolbar-fullscreen-hover = Toggle fullscreen

## Dialogs: words shared by many windows

# A button that closes a window without doing what it offers.
dlg-cancel = Cancel
# A button that dismisses a window (nothing in it is pending).
dlg-close = Close
# A button that copies the window's text to the clipboard (after a copy icon).
dlg-copy = Copy
# A button that jumps to a location.
dlg-go = Go
# A button that fills the fields from the view on screen.
dlg-use-current = Use current
# A notice after a file could not be written. $error: the system's reason, as it gives it.
dlg-save-failed = Save failed: { $error }
# A notice when no graphics device is available to render with.
dlg-gpu-not-available = GPU not available
# The name of the plain-text file type in a save dialog (the extension .txt is added by the code).
dlg-filter-text = Text

## Asked after a session that did not shut down cleanly

dlg-crash-title = Fractadyne didn't shut down cleanly
dlg-crash-intro = The previous session ended unexpectedly. A report has been saved on this machine — would you like to send it?
dlg-crash-review = Nothing is sent until you review it: the next screen shows exactly what would be included, and system information is one checkbox to exclude.
dlg-crash-report = Report it…
dlg-crash-report-hover = Opens Report an issue, with the crash report and the location that crashed attached.
dlg-crash-not-now = Not now
dlg-crash-dont-ask = Don't ask again after a crash
dlg-crash-dont-ask-hover = Reports are still saved locally, and Help ▸ Report an issue always works.

## Asked after a session that ended with a second graphics device in use

dlg-second-device-title = Turn off the second graphics device?
# $device: the graphics device's name and details, as the system gives them.
dlg-second-device-intro = The previous session ended unexpectedly while a second graphics device was in use ({ $device }). It may be the cause.
dlg-second-device-note = With it off, the card drawing the window renders everything, as before. You can turn it back on in Advanced ▸ Second graphics card.
dlg-second-device-off = Turn it off
dlg-second-device-keep = Keep it on

## Shown when the previous session's iteration setting was set aside

dlg-quarantine-title = Iteration setting set aside
dlg-quarantine-change = You can change it any time in Controls ▸ Quality.
dlg-quarantine-keep = Keep the default
# A button. $setting: the iteration setting the previous session used (such as "20,000 iterations").
dlg-quarantine-restore = Restore { $setting }

## Navigate ▸ Go to location

dlg-goto-title = Go to location
# Before two choices: "x, y" (cartesian) and the polar form below.
dlg-goto-enter-as = Enter as:
# The polar way of entering a point: an offset plus a radius r at an angle θ. Keep the symbols.
dlg-goto-polar = offset + r ∠ θ
# The tooltip of the "?" button beside the entry choice.
dlg-goto-help-hover = What the fields accept: fractions, powers, sqrt/sin/cos…, pi and other constants — opens Help → Coordinate expressions
# Labels over the entry fields. x0, y0, r, θ, Re and Im are the fields' symbols: keep them.
dlg-goto-polar-offset = Offset centre  (x0, y0 — leave 0 for a pure polar point)
dlg-goto-polar-radius = Radius r, angle θ
dlg-goto-re = Center — real (Re)
dlg-goto-im = Center — imaginary (Im)
dlg-goto-zoom = Zoom (magnification)
# A warning under the zoom field, after a ⚠ sign. $have: the iteration count set; $typical: the count this depth typically needs.
dlg-goto-iter-warning = Iterations is fixed at { $have }; this depth typically needs ~{ $typical }. The view will render flat until it is raised — or enable auto-iterations (Controls ▸ Quality), which adapts by itself.
dlg-goto-copy-hover = Copy this location to the clipboard
dlg-goto-cancel-hover = Close without going anywhere
# The expressions in brackets are examples of what can be typed: keep them as they are.
dlg-goto-paste-note = Paste a center/zoom from someone else, or Copy to share. Coordinates accept expressions: exact fractions (-3/4), functions and constants (-0.5 + 0.25*cos(pi/4), sqrt, root(x,n), ^; angles in radians), and the real field accepts a whole complex value ((37+16i)/100), which fills both.
# A section that opens and closes.
dlg-goto-feature = Go to feature (Mandelbrot)
dlg-goto-feature-poi = Well-known points:
# The dropdown's text before a point is picked.
dlg-goto-feature-poi-pick = Jump to a point of interest…
dlg-goto-feature-solve = Or solve for a feature near the current view:
# Two choices: what to solve for. Misiurewicz is a mathematician's name (a Misiurewicz point).
dlg-goto-feature-misiurewicz = Misiurewicz
dlg-goto-feature-minibrot = Nearest minibrot
# Labels of two small number fields; k and p are their symbols.
dlg-goto-feature-preperiod = Preperiod k
dlg-goto-feature-period = Period p
# The faint text in an empty field: left empty, the value is found automatically.
dlg-goto-feature-auto-hint = auto
# A small button that empties both fields, so they are found automatically again.
dlg-goto-feature-auto = Auto
dlg-goto-feature-auto-hover = Clear both boxes so the next search detects the preperiod and period from the orbit at the view centre.
dlg-goto-feature-kp-hover = Leave both blank to detect them from the orbit at the view centre; the values found are filled in here. Nobody looking at a spiral knows its preperiod, and having to supply one is what made this solver unusable in practice.
# While a solve runs, followed by its elapsed time and the note below.
dlg-goto-feature-solving = Solving…
dlg-goto-feature-solving-note = (arbitrary precision; deep views take longer)
dlg-goto-feature-cancel-hover = Stop waiting. The solve itself runs to completion in the background and its answer is discarded — nothing is left in a half-finished state.
dlg-goto-feature-find = Find near view
dlg-goto-feature-find-hover =
    Newton-solve from the current center onto the exact feature (arbitrary precision). Navigate near the feature first.

    Set the Zoom field above to the depth you WANT before solving: the point is computed to the precision that depth needs, and the view jumps straight there. That is one solve instead of a long sequence of manual zoom steps.
dlg-goto-solve-cancelled = Solve cancelled.

## Snapshot (the camera button): its tooltip, and the choice on the first press

# The camera button's tooltip, one per Snapshot setting. Followed by its shortcut, (Ctrl+S).
dlg-snapshot-hover-ask = Snapshot — save the view as shown, or a full render; the first press asks
dlg-snapshot-hover-screen = Snapshot — save the view exactly as shown, to the last export folder
dlg-snapshot-hover-render = Snapshot — full render at the Export settings, to the last export folder
# The window's title.
dlg-snapshot-title = Snapshot
dlg-snapshot-question = What should Snapshot (the camera button, Ctrl+S) save?
# The first choice, and a short tag beside it.
dlg-snapshot-screen = Screen capture
dlg-snapshot-screen-tag = instant
dlg-snapshot-screen-note = Saves the view exactly as it appears on screen, at screen resolution, as a PNG in the last export folder. What you see is what you get.
# The second choice, and a short tag beside it.
dlg-snapshot-render = Full render
dlg-snapshot-render-tag = may take a long time
# $size: the image size, as 1920×1080; $ss: the supersampling factor (1–4); $format: PNG or EXR.
dlg-snapshot-render-note = Renders at the Export settings — { $size }, { $ss }× supersampling, { $format }. Best quality, but a deep view can take minutes to hours. It runs in the background; File → Export image… shows progress and can cancel it.
dlg-snapshot-remember = Remember my choice and don't ask again
dlg-snapshot-change = You can change it any time in File → Settings → Snapshot.

## Share location (.fdn)

dlg-share-title = Share location
dlg-share-intro = A self-contained location (fractal, full-precision center, zoom, coloring). Copy it to share, or paste/load one and Apply.
dlg-share-thumb = Include a thumbnail in the saved file
dlg-share-thumb-hover = Save .fdn embeds a small picture of this view so the file can be recognized in the gallery. Adds roughly 55 KB, and is not part of the text above.
dlg-share-copy-hover = Copy the text to the clipboard
# Buttons (the first after a save icon).
dlg-share-save = Save .fdn…
dlg-share-load = Load .fdn…
dlg-share-apply = Apply
dlg-share-apply-hover = Jump to the location in the box
dlg-share-copied = Copied to clipboard.

## Help ▸ Report an issue (the report itself stays in English: it is read by the maintainer)

dlg-report-title = Report an issue
dlg-report-intro = The best place to report is a GitHub issue — it's public and searchable, and you'll see when it's fixed. Email works too. Review everything below first: nothing leaves your machine until you act.
# Labels before three dropdowns.
dlg-report-type = Type:
dlg-report-severity = Severity:
dlg-report-repro = Reproducible:
dlg-report-describe = Describe what happened:
# The faint text in the empty description box.
dlg-report-describe-hint = What happened, the steps to reproduce it, and what you expected instead.
# Before the checkboxes of what the report carries.
dlg-report-include = Include:
dlg-report-sysinfo = System info (version, OS, CPU, GPU, VRAM)
dlg-report-location = Current location (.fdn)
dlg-report-log = Recent log
dlg-report-frames = Frame record (the last minute, and the last 40 frames)
dlg-report-crash = Latest crash report
dlg-report-crash-none = Latest crash report (none found)
dlg-report-crash-view = Location that crashed (.fdn)
dlg-report-crash-view-none = Location that crashed (.fdn) (none found)
dlg-report-crash-view-hover = The exact view a crash or graphics-device loss happened at — the one thing that lets a maintainer reproduce it. Recorded automatically.
dlg-report-test = Diagnostics test result
dlg-report-test-none = Diagnostics test result (run one from Help → Diagnostics…)
dlg-report-test-hover = A machine-validated pass/fail from your own hardware — far more useful to a maintainer than a description alone
# A section that opens and closes. $size: the report's size, as "12 KB".
dlg-report-preview = Preview what will be sent  ({ $size })
dlg-report-github = Open a GitHub issue
dlg-report-github-hover = Open a new-issue page with the title filled in; the report is copied to your clipboard — paste it into the issue body (Ctrl+V).
# Buttons (after a copy and a save icon).
dlg-report-copy = Copy report
dlg-report-copy-hover = Copy everything to the clipboard
dlg-report-save = Save report…
dlg-report-save-hover = Save as a .txt to attach
dlg-report-by-email = Or by email:
dlg-report-gmail = Compose in Gmail
# $address: the email address reports go to.
dlg-report-gmail-hover = Open a Gmail compose window to { $address } with the subject filled; paste the copied report
dlg-report-email = Email app…
dlg-report-email-hover = Open your default mail app (mailto) — needs an email handler configured
dlg-report-tip = Tip: the report is copied to your clipboard on any of these — paste it into the issue or message body (Ctrl+V). Please also attach a screenshot or sample image if you can (drag it in); it helps a lot for rendering and UI problems.
# Notices in the window after a button is pressed.
dlg-report-opened-github = Opened GitHub — the report is on the clipboard; paste it into the issue.
dlg-report-copied = Report copied to the clipboard.
# $path: where the report was saved.
dlg-report-saved = Saved to { $path }
dlg-report-opened-email = Opened your email app — the report is on the clipboard; paste it in.
dlg-report-opened-gmail = Opened Gmail compose — paste the copied report into the body (Ctrl+V).

## A location whose checksum does not match

dlg-checksum-title = This location may be damaged
# After a ⚠ sign.
dlg-checksum-mismatch = Its checksum does not match the data beside it.
dlg-checksum-why = That usually means the text was cut short or altered on the way here — a partial copy out of a chat window or a forum post is the common cause.
# A label over where the location came from (a file path, or the clipboard).
dlg-checksum-source = Source
# The two checksums, side by side: the one written in the file, and the one computed from its data.
dlg-checksum-in-file = in the file
dlg-checksum-from-data = from the data
dlg-checksum-load-note = You can load it anyway — the coordinates may still be fine, and nothing is overwritten. Anything unreadable is reported after loading.
dlg-checksum-load = Load anyway
dlg-checksum-load-hover = Jump to it despite the mismatch
# Notices after loading it. $zoom: the magnification, as 1.2e148.
dlg-checksum-loaded = Loaded @ { $zoom }× despite the checksum mismatch
# $zoom: the magnification, as 1.2e148; $note: what the loader found wrong with the location.
dlg-checksum-loaded-note = Loaded @ { $zoom }× — { $note }
dlg-checksum-not-loaded = Not loaded.

## File ▸ Reset application state

dlg-reset-title = Reset application state
# After a ⚠ sign, followed by a list.
dlg-reset-warning = This permanently deletes all saved Fractadyne data:
# The list's items, each after a bullet.
dlg-reset-session = the saved session (current view, coloring, preferences)
dlg-reset-bookmarks = all bookmarks and their thumbnails
dlg-reset-refcache = the reference-orbit cache (rebuilt as you revisit locations)
# A label over the folder that is deleted.
dlg-reset-location = Location
dlg-reset-note = This can't be undone. The current session won't be re-saved on exit, so defaults load on the next launch.
dlg-reset-confirm = Reset everything
dlg-reset-done = Application state reset — defaults will load on the next launch.
# $error: the system's reason, as it gives it.
dlg-reset-failed = Reset failed: { $error }

## Tools ▸ Tour from current view

dlg-tour-title = Tour from current view
dlg-tour-intro = Creates a tour that zooms from the full view down to where you are now.
# A label before the fractal and magnification the tour ends at.
dlg-tour-destination = Destination
dlg-tour-notation = Notation (optional caption shown during the dive)
# The faint example text in the empty caption box.
dlg-tour-notation-hint = e.g. Diving to the Misiurewicz three-spar…
dlg-tour-duration = Zoom duration
# $rate: how fast the dive zooms, in powers of ten a second (as 2.5).
dlg-tour-rate = ≈ { $rate } decades/s
# A small button that puts the duration back to its default.
dlg-tour-reset = Reset
dlg-tour-reset-hover = Depth-based default
dlg-tour-duration-note = Default scales with depth so the dive stays watchable. Faster dives can flash single-color frames at extreme zoom.
# A button, after a save icon.
dlg-tour-save = Save tour…
# The file type's name in the save dialog.
dlg-tour-filter = Fractadyne script (TOML)
# A notice. $file: the saved file's name. --render-tour is a command-line option: keep it.
dlg-tour-saved = Saved tour → { $file }. Play it via Tools → Play tour, or render with --render-tour.

## Navigate ▸ Bookmarks ▸ Manage…

dlg-bookmarks-title = Bookmarks
# The faint text in the empty name box.
dlg-bookmarks-name-hint = name (optional)
# A button, after a bookmark icon.
dlg-bookmarks-add = Add current view
dlg-bookmarks-empty = No bookmarks yet. Add one from the current view.
# The tooltip of a bookmark's delete (bin) button.
dlg-bookmarks-delete-hover = Delete

## Tools ▸ Benchmark…

dlg-bench-title = Benchmark
dlg-bench-mode = Mode
dlg-bench-current = Current settings
dlg-bench-current-hover = Play the fixed deep-zoom tour into the live view using your current resolution and settings.
dlg-bench-standard = Standardized
dlg-bench-standard-hover = Pin resolution + all render settings so the score is comparable across machines.
dlg-bench-resolution = Resolution
dlg-bench-depth = Depth
# Standard and Ultra are the two depths in the dropdown this describes.
dlg-bench-depth-hover = Both dives cross into the deep floatexp/BLA path; Ultra (1e48×) spends far longer there than Standard (1e32×), and takes correspondingly longer to run.
dlg-bench-burnin = Burn-in (repeat)
dlg-bench-burnin-hover = Run the benchmark repeatedly to reveal stability and thermal throttling.
# The label of a slider: how many times the benchmark runs.
dlg-bench-passes = passes
# $size: the image size, as 1920×1080; $ss: the supersampling factor; $depth: the dive's final magnification, as 1e32.
dlg-bench-std-summary = Renders offscreen at { $size }, { $ss }× SS, Mandelbrot/smooth, 60-frame dive to { $depth }×.
dlg-bench-run = Run

## The standardized benchmark's progress window

dlg-bench-progress-title = Running benchmark…
# $resolution: the benchmark's resolution, as the dropdown names it.
dlg-bench-progress-mode = Standardized · { $resolution }
# $pass: the pass running now; $total: how many passes there are.
dlg-bench-progress-pass = Burn-in pass { $pass }/{ $total }
dlg-bench-progress-dive = Rendering the fixed dive…
# On the progress bar. $done: frames rendered; $total: frames in the dive.
dlg-bench-progress-frame = frame { $done }/{ $total }
# $fps: frames per second, as 41.3.
dlg-bench-progress-last = last pass: { $fps } fps

## The benchmark's results

dlg-bench-results-title = Benchmark results
# Buttons, each after an icon.
dlg-bench-results-run-again = Run again…
dlg-bench-results-save = Save…
dlg-bench-results-none = No benchmark has been run yet.
dlg-bench-results-saved = Benchmark saved.

## The accelerated build's notice when its arithmetic libraries are missing

dlg-backend-title = Accelerated build — using built-in arithmetic
dlg-backend-dont-show = Don't show this again

## File ▸ Gallery…

dlg-gallery-title = Gallery
# A button that picks the folder to show.
dlg-gallery-folder = Folder…
# A button, after a refresh icon.
dlg-gallery-refresh = Refresh
dlg-gallery-empty = No Fractadyne images in this folder.
# An image's title when it does not name its fractal.
dlg-gallery-untitled = Fractadyne image
# $zoom: the image's magnification, as 1.2e148; $saved: when it was saved, as the file gives it.
dlg-gallery-zoom = zoom { $zoom }   { $saved }
# The notes saved with the image, in quotation marks. Use your language's quotation marks.
dlg-gallery-notes = “{ $notes }”
dlg-gallery-open = Open this view
dlg-gallery-loaded = Loaded view from gallery.

## File ▸ Export image…

dlg-export-title = Export image
# A dropdown of image sizes.
dlg-export-size = Size
# The size dropdown's text for a size set by hand. $size: the size, as 1920×1080.
dlg-export-size-custom-current = Custom — { $size }
# The size dropdown's first choice.
dlg-export-size-custom = Custom…
dlg-export-size-custom-hover = Set the width yourself; height follows the Aspect below.
dlg-export-width = Width (px)
# After the width field. $height: the height in pixels, which follows from the aspect.
dlg-export-height-from-aspect = × { $height } (from Aspect)
dlg-export-ss = Supersampling
dlg-export-aspect = Aspect
# The aspect choice that keeps the window's own shape.
dlg-export-aspect-window = Match window
# Before the choices PNG and OpenEXR.
dlg-export-format = Format:
dlg-export-hud = Location HUD
dlg-export-hud-hover = Burn a zoom-level + coordinate panel into the top-left of the exported image (scales with the output; shows the map/Mandelbrot view's zoom + center).
dlg-export-glitch = Glitch correction
dlg-export-glitch-hover = Multi-reference correction of perturbation glitches. Automatically skipped for very deep (floatexp) single-view exports so the reference build + render run off-thread and the app stays responsive. Never used for Mandelbrot or Multibrot views (outside Julia mode): checked against an exact calculation, it fixed nothing there and cost up to 5x the export time.
dlg-export-deep-note = Deep export: the reference builds off-thread (UI stays live); glitch correction is skipped at this depth.
# A dropdown, shown in the dual (map + Julia) view, and its three choices.
dlg-export-dual = Dual layout
dlg-export-dual-side = Side by side
dlg-export-dual-separate = Separate files
dlg-export-dual-map = Map only
dlg-export-notes = Notes:
# The faint text in the empty notes box.
dlg-export-notes-hint = saved with the image (≤120 chars)
# $width and $height: the image size in pixels; $left: how many characters the notes can still take.
dlg-export-output = Output: { $width } × { $height } px   ({ $left } chars left)
# When the export's shape differs from the window's. $percent: how much more, as 25.
dlg-export-extra-width = Shows { $percent }% more width than the window (nothing is cropped)
# The same, when the export is taller than the window. $percent: how much more, as 25.
dlg-export-extra-height = Shows { $percent }% more height than the window (nothing is cropped)
dlg-export-tiles-note = Rendered in tiles (no size cap) on a background thread. The file embeds the view + notes so it can be reopened with File ▸ Open view or location…
# A label before the button that picks the folder exports go to.
dlg-export-folder = Folder:
dlg-export-choose = Choose…
dlg-export-choose-hover = Pick the target directory for exports
dlg-export-open-after = Open when done
dlg-export-open-after-hover = Hand the finished image to the system viewer as soon as it is written.
# $count: the graphics cards in this machine (2 or more).
dlg-export-all-gpus = Use all graphics cards ({ $count })
dlg-export-all-gpus-hover = Split the image by tile across every graphics card of this machine, each with its own copy of the deep reference. Identical cards give exactly the image one renders alone; different models draw a few pixels slightly differently, tile by tile (measured: no visible seam). If a card fails, its tiles go to the others. Images the split cannot take (some colouring methods and formulas) use this window's card.
# The export's progress.
dlg-export-preparing = Preparing (building reference)…
dlg-export-saving = Saving…
dlg-export-rendering = Rendering…
# $time: the time since the export started, as 8.3s or 1m 04.0s.
dlg-export-elapsed = Elapsed: { $time }
dlg-export-cancel-hover = Close without exporting
# A button, after a save icon.
dlg-export-save-as = Save as…
dlg-export-save-as-hover = Choose the file name and location
dlg-export-export = Export
dlg-export-export-hover = Render and save into the folder above (auto-named)

## Notices worded at startup (main.rs)

# A notice shown once after starting, when the saved session comes from a newer version of the app. $saved and $handled: version numbers of the session file's format (1, 2…).
app-state-newer = This session was saved by a newer Fractadyne (state v{ $saved }; this build handles v{ $handled }). Some settings may not apply, and saving will rewrite it in this build's format.
# The title of the notice that says the accelerated build's arithmetic libraries are found again (its text comes from elsewhere).
app-notice-mpfr-restored-title = Accelerated arithmetic restored
# After Fractadyne restarted itself because the graphics driver reset the GPU.
app-toast-gpu-recovered = Recovered from a graphics device reset — reopened at the home view, since the previous location was too demanding to render.

## Short notices (toasts)

# When the autopilot reached a view that could not be rendered and put the last good view back.
app-toast-autopilot-view-restored = Autopilot stopped: the view became unusable and was put back (logged)
# $error: the system's reason, as it gives it (often in English).
app-toast-gradients-save-failed = Couldn't save gradients: { $error }
# $error: the reason, as the program gives it (often in English).
app-toast-gradients-serialize-failed = Couldn't serialize gradients: { $error }
# $name: the name the user gave the gradient.
app-toast-gradient-saved = Saved gradient "{ $name }".
# $error: the reason, as the program gives it (often in English).
app-toast-bookmarks-serialize-failed = Couldn't serialize bookmarks: { $error }
# $error: the system's reason, as it gives it (often in English).
app-toast-bookmarks-save-failed = Couldn't save bookmarks: { $error }
# Jumping to a named point given by exact expressions. $name: the point's name (in English).
app-toast-coordinates-failed = { $name }: could not evaluate the coordinates
# $zoom: the magnification, already formatted (1,234.5 or 1.2e148).
app-toast-random-location = Random location · { $zoom }×
# Importing a Kalles Fraktaler .kfr location. $error: the reason (in English).
app-toast-import-failed = Import failed: { $error }
# $zoom: the magnification, already formatted.
app-toast-location-loaded = Loaded location @ { $zoom }×

## Go to location: why a jump could not be made (one line per field that failed)

# "Re" is the dialog's real-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-re = Re: { $error }
# "Im" is the dialog's imaginary-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-im = Im: { $error }
# The imaginary field was given a complex value. $error: the evaluator's reason (in English).
app-goto-error-im-real = Im: { $error } — type the imaginary part as a real number, without the i
# "Zoom" is the dialog's magnification field.
app-goto-error-zoom-empty = Zoom: enter a magnification, e.g. 1e50 or 250000
# $zoom: what was typed, in quotes.
app-goto-error-zoom = Zoom: cannot read { $zoom } — enter a positive magnification such as 1e50 or 250000
app-goto-error-invalid = Invalid input — check the coordinates and zoom.

## Finding a minibrot or Misiurewicz point (the M key and the Go to dialog)

app-feature-needs-holomorphic = Minibrot finder needs a holomorphic family (Mandelbrot / Multibrot).
app-feature-not-in-julia = The minibrot finder searches the Mandelbrot set — switch off Julia mode to use it.
app-feature-busy = Already looking for a feature — one at a time.
app-feature-searching = Looking for the nearest minibrot…
app-feature-mandelbrot-only = Feature finding is Mandelbrot-only.
app-feature-no-pair = No Misiurewicz point found within the search budget — the point may be deeper than the finder reaches at this view, or the centre may not be near a spiral or branch point.
# The name of what was found, used as $feature in the messages below. $period: the minibrot's period (a whole number).
app-feature-minibrot = period-{ $period } minibrot
# $preperiod and $period: the point's two whole numbers (k, p). $octaves: how often the view repeats when zooming in (2 decimals); $twist: the turn per repeat in degrees.
app-feature-misiurewicz-lambda = Misiurewicz ({ $preperiod },{ $period }) — repeats every { $octaves } octaves, twist { $twist }°
# $preperiod and $period: the point's two whole numbers (k, p).
app-feature-misiurewicz = Misiurewicz ({ $preperiod },{ $period })
# Added after the messages below when the fixed iteration count is too low for the new depth. $have: the iterations set now; $typical: what the depth typically needs.
app-feature-iter-note = ⚠ Iterations is fixed at { $have }; this depth typically needs ~{ $typical } — raise it (or enable auto-iterations) if the view renders flat.
# $feature: what was found (see above). $zoom and $asked: magnifications, already formatted; $octaves: the solver's depth limit.
app-feature-zoomed-capped = Zoomed to the { $feature } — { $zoom }×. Solving is capped at { $octaves } octaves, so it stopped there rather than { $asked }×.
# $feature: what was found. $zoom: the magnification, already formatted. $seconds: how long the search took (1 decimal).
app-feature-zoomed = Zoomed to the { $feature } — { $zoom }× ({ $seconds }s)
# $feature: what was found. $seconds: how long the search took (1 decimal).
app-feature-snapped = Snapped to { $feature } center ({ $seconds }s)
app-feature-no-minibrot = No minibrot center found near the view — zoom closer to one.
# $feature: the point asked for, e.g. Misiurewicz (3,1). $widths: how many view-widths away, already formatted. $octaves: how far to zoom out.
app-feature-just-outside = Found a { $feature } point just outside this view — { $widths } view-widths away. Zoom out about { $octaves } octaves to bring it on screen, or set a larger preperiod for a feature at this scale.
# $feature: the point asked for. $widths: how many view-widths away, already formatted (may be 2^187). $octaves: how far to zoom out.
app-feature-far = Found a { $feature } point, but it is { $widths } view-widths away — that feature is far COARSER than this view. Zoom OUT by about { $octaves } octaves to reach it, or try a larger preperiod for a feature at this scale.
# $feature: the point asked for. $residual: a small number in scientific notation.
app-feature-not-preperiodic = { $feature } does not fit the orbit here (residual { $residual }) — clear both boxes to detect the pair from the view.
# $feature: the point asked for. "k/p" are the dialog's two fields.
app-feature-not-converged = No { $feature } point converged near the view — navigate closer, or try different k/p.
app-feature-bad-request = Preperiod and period must both be positive.

## Share location dialog and location files

app-location-empty = Nothing to load (or text too large).
app-location-not-fractadyne = Not a Fractadyne location.
# $zoom: the magnification, already formatted. $note: what in the text was ignored or repaired (in English).
app-share-loaded-note = Loaded @ { $zoom }× — { $note }.
# $error: the system's reason (often in English).
app-share-save-failed = Save failed: { $error }
app-share-saved = Saved.
app-share-saved-rendering-thumb = Saved. Rendering the thumbnail…
app-share-saved-no-thumb = Saved — without a thumbnail (none could be rendered for this view).
app-share-saved-with-thumb = Saved, with a thumbnail.
# $error: the system's reason (often in English).
app-share-saved-thumb-failed = Saved, but the thumbnail could not be added: { $error }
app-share-loaded-into-box = Loaded into the box — review, then Apply.
# Reading a file failed (a location or a palette). $error: the system's reason (often in English).
app-read-failed = Read failed: { $error }
app-share-too-large = File too large (not a .fdn location?).

## File-dialog filter names (the extensions are added by the code)

app-filter-kfr = Kalles Fraktaler location
app-filter-fdn = Fractadyne location
app-filter-map = Fractint / KF palette
app-filter-ugr = Ultra Fractal gradient
app-filter-ase = Adobe swatch exchange
app-filter-ggr = GIMP gradient
app-filter-all = All files

## Importing and saving palette files (shown under the gradient editor)

# The reason a file was refused before reading it.
app-gradient-map-too-large = far too large to be a .map palette
app-gradient-ugr-too-large = far too large to be a .ugr palette file
app-gradient-ase-too-large = far too large to be a .ase swatch list
app-gradient-ggr-too-large = far too large to be a .ggr gradient
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported-vga = Imported { $count } entries from { $file } - a 6-bit VGA table (white is 252, not 255), kept as written so it matches Fractint's own output.
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported = Imported { $count } entries from { $file }, as hard bands.
# $error: the reader's reason (in English).
app-gradient-map-invalid = Not a .map palette: { $error }
# $name: the gradient's name in the file; $file: the file's name.
app-gradient-ugr-imported = Imported "{ $name }" from { $file }.
# $file: the file's name; $count: how many gradients it holds.
app-gradient-ugr-pick = { $file } contains { $count } gradients — pick one below.
# $error: the reader's reason (in English).
app-gradient-ugr-invalid = Not a .ugr palette file: { $error }
# $used: the swatches kept; $total: the swatches in the file.
app-gradient-ase-imported-some = Imported { $used } of { $total } swatches, evenly spaced.
# $count: the swatches in the file.
app-gradient-ase-imported = Imported { $count } swatches, evenly spaced and blended.
# $error: the reader's reason (in English).
app-gradient-ase-invalid = Not a .ase swatch list: { $error }
app-gradient-nothing-to-save = No custom gradient to save yet.
# $file: the saved file's name.
app-gradient-ggr-saved = Saved "{ $file }" — midpoints, blend curves and colour spaces all intact.
# $file: the saved file's name; $count: the segments whose curve could not be kept.
app-gradient-ggr-saved-lossy =
    { $count ->
        [one] Saved "{ $file }" — but { $count } segment uses a Bézier curve, which the .ggr format cannot express. It was written as a straight blend; the copy in your library keeps the curve.
       *[other] Saved "{ $file }" — but { $count } segments use a Bézier curve, which the .ggr format cannot express. Those were written as straight blends; the copy in your library keeps the curves.
    }
# $path: the file; $error: the system's reason (often in English).
app-gradient-write-failed = Couldn't write { $path }: { $error }
# $name: the gradient's name; $count: its segments.
app-gradient-ggr-imported = Imported "{ $name }" - { $count } segments, with their midpoints, blend curves and colour spaces.
# $error: the reader's reason (in English).
app-gradient-ggr-invalid = Not a .ggr gradient: { $error }

## The eyedropper's banner across the top of the screen while picking a colour

# $hex: the colour under the pointer, as #rrggbb.
app-eyedropper-banner-color = Picking a colour  ·  { $hex }  ·  click to take it, Esc to cancel
app-eyedropper-banner = Picking a colour  ·  click to take it, Esc to cancel

## The gradient editor window

app-gradient-title = Gradient editor
# Before the two view buttons, Bar and Ring.
app-gradient-view = View
app-gradient-view-bar = Bar
app-gradient-view-bar-hover = A straight strip — best for placing stops precisely
app-gradient-view-ring = Ring
app-gradient-view-ring-hover = A ring, because the palette is CYCLED: the join at the top is the seam the renderer crosses on every sweep, and a gradient that looks fine on a bar can have a hard edge there.
app-gradient-add-lane-hover = Click anywhere on this line to add a stop there. The + adds one in the widest gap.
app-gradient-strip-hover = Drag a marker to move its stop · drag the strip itself to ROTATE the whole gradient, wrapping at the seam · double-click to add a stop, right-click a marker to remove it · arrow keys nudge the selection (Shift for fine).
app-gradient-remove-selected-hover = Remove the selected stop
app-gradient-ring-hover = Drag the ring itself to ROTATE the whole gradient — what passes the seam comes back round the other side · drag a marker to move its stop · click the outer track to add a stop there, or ⊕ in the middle to add one in the widest gap · ⊖ removes the selected stop. The hollow mark at the top is the seam, where a cycled palette wraps; it stays put because it IS the join, and the colours turn under it.
app-gradient-empty = No custom gradient yet — "Copy preset…", an import or a saved gradient below starts one, and its stops and per-segment curves appear here.
# $count: the entries of an imported palette, more than the editor shows stop by stop.
app-gradient-too-many = { $count } imported entries - too many to edit stop by stop. Use the preview above, or Copy preset to start over.
# The selected segment's heading. $index: its number; $total: how many there are; $left and $right: where it starts and ends, 0.000–1.000.
app-gradient-segment-heading = Segment { $index } of { $total }  ·  { $left } → { $right }
# A label before the list of blend curves.
app-gradient-curve = Curve
app-gradient-curve-hover = How fast the colour travels across this segment. "Curved" is shaped by the midpoint — at midpoint 0.50 it is exactly linear.
# The blend curves (GIMP's kinds), in a narrow list: keep them short.
app-gradient-blend-linear = Linear
app-gradient-blend-curved = Curved
app-gradient-blend-sine = Sine
app-gradient-blend-sphere-increasing = Sphere ↑
app-gradient-blend-sphere-decreasing = Sphere ↓
app-gradient-blend-bezier = Bézier
# Hover on Bézier in the list of blend curves.
app-gradient-bezier-exact-hover = Two control points you can drag in both directions. This curve is already a cubic, so converting it changes nothing.
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-visible-hover = Two control points you can drag in both directions. Converting APPROXIMATES the current curve — this one to within { $error }, which is clearly visible.
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-barely-hover = Two control points you can drag in both directions. Converting APPROXIMATES the current curve — this one to within { $error }, which is barely visible.
# A label before the list of colour spaces (RGB, HSV).
app-gradient-space = Space
# A label before the Bézier presets beside it.
app-gradient-ease = Ease
app-gradient-ease-hover = Presets write the two control points; drag them on the curve for anything else. A Bézier has no midpoint — its handles say where it reaches halfway.
# Bézier presets, as in CSS easing: short buttons.
app-gradient-ease-linear = Linear
app-gradient-ease-in = In
app-gradient-ease-out = Out
app-gradient-ease-in-out = In-out
app-gradient-midpoint = Midpoint
app-gradient-midpoint-hover = Where the blend reaches halfway, as a fraction of this segment. 0.50 is centred.
# A button that puts the midpoint back in the middle of the segment.
app-gradient-centre = Centre
app-gradient-centre-hover = Back to 0.50
app-gradient-no-hue = ⚠ one end of this segment has no hue
app-gradient-no-hue-hover = One end is black, white or grey, so the sweep starts from red and travels the whole colour wheel. That is legal and sometimes wanted — but it is why the middle can be a colour neither end contains. The strip under the curve shows what it does.
app-gradient-segment-start-hover = Colour at the START of this segment
app-gradient-segment-end-hover = Colour at the END of this segment
app-gradient-bytes-hover = Red, green and blue as 0–255
app-gradient-floats-hover = The same colour as 0–1, which is what the renderer stores
app-gradient-canvas-bezier-hover = The blend curve for the selected segment. Drag either control point — in x AND y — to reshape it. The faint lines are 0 and 1; a curve may pass outside them, and the colour is clamped rather than the curve.
app-gradient-canvas-hover = The blend curve for the selected segment: how fast the colour travels from the left stop to the right one. Drag the ring to move the midpoint, or switch the curve to Bézier for a control point that moves in both directions.
# The selected stop's heading. $index: its number; $total: how many there are.
app-gradient-stop-heading = Stop { $index } of { $total }
app-gradient-stop-color-hover = The colour at this stop — click the swatch to pick, or type a hex value beside it
app-gradient-stop-bytes-hover = Red, green and blue as 0–255 — the bytes this colour is written as
# The eyedropper button.
app-gradient-pick = Pick
app-gradient-pick-hover = Eyedropper — take a colour from anywhere on screen. Move the pointer to the colour you want and click; Esc cancels.
# Written before the stop's position (0.000–1.000) in its number field: a short abbreviation of "position".
app-gradient-stop-pos = pos
app-gradient-nudge-hover = Nudge this stop (hold Shift for a finer step)
app-gradient-remove = Remove
app-gradient-remove-hover = Remove this stop (or click the ⊖ under it)
app-gradient-add-stop = Add stop
app-gradient-add-stop-hover = Split the widest segment (or click the line above the gradient)
# A menu of the built-in palettes, to start from a copy of one.
app-gradient-copy-preset = Copy preset…
app-gradient-paste = Paste…
app-gradient-paste-hover = Import a palette from hex colours or 0–255 RGB triples
app-gradient-save-ggr = Save .ggr…
app-gradient-save-ggr-hover = Write this gradient to a GIMP .ggr file — readable by GIMP, Krita and Inkscape, and by this editor
# A menu of the palette file formats (.map, .ugr, .ggr, .ase).
app-gradient-import = Import
app-gradient-import-map-hover = Load a Fractint / Kalles Fraktaler .map palette file (R G B lines, one per entry)
app-gradient-import-ugr-hover = Load an Ultra Fractal .ugr gradient file (a .ugr usually holds many gradients — you pick one)
app-gradient-import-ggr-hover = Load a GIMP .ggr gradient, with its midpoints, blend curves and colour spaces intact
app-gradient-import-ase-hover = Load an Adobe swatch list (.ase). Swatches have no positions, so they arrive evenly spaced and blended.
# The gradient's name, for the library of saved gradients.
app-gradient-name = Name
# Shown greyed in the empty name field.
app-gradient-name-hint = my gradient
# The save button, when a gradient of that name is already saved.
app-gradient-update = Update
app-gradient-save = Save
app-gradient-save-hover = Save this gradient to your library, with its curves and colour spaces intact
app-gradient-save-needs-name-hover = Give the gradient a name first
# A menu of the saved gradients. $count: how many there are.
app-gradient-saved-menu = Saved ({ $count })
app-gradient-nothing-saved = Nothing saved yet
app-gradient-delete-saved-hover = Delete this saved gradient
# $count: the gradient's segments.
app-gradient-rich = Rich gradient: { $count } segments carry midpoints, blend curves or colour spaces — more than a plain stop list can hold.
app-gradient-convert = Convert to editable stops
app-gradient-convert-hover = Replace it with a plain stop list. This DISCARDS the midpoints, blend curves and hue sweeps a stop list cannot hold.
# Above the list of gradients in an Ultra Fractal file. $count: how many; $file: the file's name.
app-gradient-ugr-list = { $count } gradients in { $file }
app-gradient-close-list = Close list
app-gradient-seamless = Seamless (end matches start)
app-gradient-seamless-hover = A palette is cycled, so position 1 and position 0 are next to each other on screen. With this on, the last stop's colour is held equal to the first — the ring view shows the join at the top.
app-gradient-ends-differ = ⚠ ends differ
app-gradient-ends-differ-hover = The colour at the end of the gradient is not the colour at the start, so a cycled palette shows a hard edge where it wraps.
app-gradient-hard-bands = Hard bands (no blending between entries)
app-gradient-hard-bands-hover = Fractint / .map semantics: every entry is a flat colour. Off blends them into a smooth gradient.
app-gradient-paste-intro = Paste hex colours (#ff8800) or 0–255 triples (255 136 0), separated by commas, spaces or new lines:
# Applies the pasted colours.
app-gradient-apply = Apply
# Empties the paste box.
app-gradient-clear = Clear
# $used: the colours kept; $total: the colours pasted; $limit: the most that are kept.
app-gradient-paste-imported-some = Imported { $used } of { $total } colours, sampled evenly across your list ({ $limit } is the limit).
# $count: the colours pasted.
app-gradient-paste-imported = Imported { $count } colours.
# $error: why the pasted text could not be read (in English).
app-gradient-paste-unreadable = Couldn't read that: { $error }
# A hint at the bottom of the editor. $count: the palette's entries.
app-gradient-bands-hint = { $count } bands · taken in file order; positions are ignored while bands are on.
# A hint at the bottom of the editor, in the ring view. $count: the gradient's stops.
app-gradient-ring-hint = { $count } stops · click the outer track to add one there · ⊕ adds in the widest gap, ⊖ removes the selected stop · drag a marker to move it · the mark at the top is the seam
# A hint at the bottom of the editor, in the bar view. $count: the gradient's stops.
app-gradient-bar-hint = { $count } stops · click the line above the gradient to add one there · drag a marker to move it · ⊖ or right-click removes
app-gradient-cancel = Cancel
app-gradient-cancel-hover = Discard every change made since this window was opened, and close it
app-gradient-ok = OK
app-gradient-ok-hover = Keep these changes and close the editor

## Shared

# The button that closes the Help window and the "Faster deep zoom" window.
app-close = Close

## The "Faster deep zoom" window (Help menu): the optional accelerated build

app-accel-title = Faster deep zoom
app-accel-active-heading = You are running the accelerated build
app-accel-active-body = Deep-zoom reference orbits are being computed with MPFR/GMP, which is 2.5–6.4× faster than the standard build at that step — the pause before a deep view starts resolving.
# $status: which arithmetic library is in use and its version (in English).
app-accel-in-use = Arithmetic in use: { $status }
app-accel-fallback-heading = Accelerated build — using built-in arithmetic
app-accel-fallback-body = This is the accelerated (MPFR) build, but its math libraries could not be found, so it is using the built-in arithmetic (astro-float). The images are identical; only the pause before a deep view resolves is slower.
# Followed by the folder the program runs from, on its own line.
app-accel-fallback-dlls = To use the faster path, put libmpfr-6.dll and libgmp-10.dll (and libgcc_s_seh-1.dll, libwinpthread-1.dll) beside fractadyne.exe, then restart:
app-accel-warning-hidden = The startup warning about this is hidden.
app-accel-show-again = Show it again
app-accel-offer-heading = An optional build is 2.5–6.4× faster at depth
app-accel-offer-body = Deep zoom spends much of its time computing reference orbits on the CPU - the pause before a deep view starts resolving. An optional build does that with the MPFR/GMP libraries instead, which is 2.5 to 6.4 times faster, and more so the deeper you go. Everything else is the same.
app-accel-offer-identical = The images are BYTE-IDENTICAL - it is the same mathematics computed by a faster library, checked across every fractal formula and the whole deep-zoom comparison corpus.
app-accel-offer-shared = Your settings, saved session and locations are shared between the two, so you can switch freely and nothing needs importing.
# Shown on Windows.
app-accel-separate-windows = It is a separate download because the libraries it uses cannot be built with the compiler the standard Windows build uses, and they carry a different licence (GNU LGPL v3) from Fractadyne's own.
# Shown on other systems.
app-accel-separate = It is a separate download because the libraries it uses carry a different licence (GNU LGPL v3) from Fractadyne's own.
app-accel-download = Download for this version
app-accel-all-releases = All releases
app-accel-all-releases-hover = If the direct link 404s, this version has no accelerated build yet - pick the nearest one here.
# Shown on Windows.
app-accel-install-windows = Extract it and run fractadyne.exe from that folder, keeping the .dll files beside it.
# Shown on Linux.
app-accel-install-linux = Extract it and run ./fractadyne from that folder. It uses your system's GMP and MPFR (libgmp10 / libmpfr6), which nearly every desktop already has - see README-ACCELERATED.txt in the package.
# Shown on other systems.
app-accel-install-other = No accelerated package is built for this platform yet; it can be built from source with --features fractadyne-core/rug.

## The Help window's frame (its sections are under help-)

app-help-title = Fractadyne Help

## Checking for updates

app-toast-up-to-date = You're on the latest version.
# $error: why the check failed (in English).
app-toast-update-failed = Update check failed: { $error }
# The window that offers a newer version.
app-update-title = Update available
# $version: the new version (0.3.0-beta.20). $channel: the token "stable" or "beta", in English; a translation can choose its words by it ({ $channel -> [beta] … *[other] … }).
app-update-version = Fractadyne { $version } ({ $channel }) is available
# $current: the version running now. $track: the update track the user follows (its name in the language in use).
app-update-running = You're running { $current }  ·  { $track }
app-update-download-intro = Download the latest release from GitHub:
app-update-download = Download from GitHub
app-update-later = Remind me later

## The second graphics card (shown in its window, after "In use:" or "Could not start:")

# The second device is the same card as the window's. $name: the card's name.
app-worker-same-card = same card, { $name }
# Why the second device could not start.
app-worker-thread-stopped = the device thread stopped

## Export status (shown in the Export image window)

app-export-gpu-unavailable = GPU not available
app-export-reference-aborted = Export failed: reference build aborted.

## The performance panel (View ▸ Performance panel)
## Its row labels sit in a monospace column about 14 characters wide: keep them short (a longer label pushes its value to the right).

app-perf-fps = FPS
app-perf-fps-hover = Frames presented per second, from the smoothed interval between frames. While a view is settling this reports refinement progress instead - a settling view is not running at the rate a number here would imply.
# What the FPS row shows while a view is settling. $done and $total: tiles; $eta: the time left, as m:ss.
app-perf-refining-tiles = refining { $done }/{ $total } (~{ $eta })
app-perf-refining-arming = refining (arming)
# $percent: how much is done, with one decimal.
app-perf-refining-percent = refining { $percent }%
app-perf-building-reference = building reference
app-perf-idle = idle
app-perf-frame = frame
app-perf-frame-hover = Smoothed wall-clock time between frames - not a sum of the rows below it. It includes waiting to present, so on a settled idle view it reflects the repaint cadence rather than any work.
app-perf-cpu = cpu
app-perf-cpu-hover = Main-thread time inside one update: parameter setup and reference-cache management. This is the row that spikes when a reference rebuilds, and a spike here is the stutter you feel.
app-perf-gpu-idle = gpu/idle
app-perf-gpu-idle-hover = Frame time minus main-thread CPU time. That is GPU work while the view is rendering, but on a settled view it is mostly idle waiting - the name says both because the panel cannot tell them apart from this subtraction alone.
app-perf-timing = timing
# How frame cost is measured, the timing row's value. TIMESTAMP_QUERY is a name.
app-perf-timing-starved = wall clock (timestamps starved)
app-perf-timing-gpu = GPU timestamps
app-perf-timing-no-query = wall clock (no TIMESTAMP_QUERY)
app-perf-timing-hover = How frame cost is measured. GPU timestamps are the precise source; the wall-clock fallback engages when timings stop arriving, and a budget priced by wall clock has repeatedly mis-sized deep frames — treat red here as 'the safety margins are estimates right now'.
app-perf-mode = mode
app-perf-mode-hover = Arithmetic path the last frame rendered with: direct (no reference orbit), df32 perturbation, or floatexp - the extended-range path used past about 1e28x, and the only one on which BLA runs.
app-perf-eff-iter = eff iter
app-perf-eff-iter-hover = Iterations the last frame actually used, after depth scaling and any cap applied while moving - not the base setting in Quality, which is only the starting point.
app-perf-precision = precision
app-perf-precision-hover = Mantissa bits the reference orbit is computed at. It grows with depth; 64 bits is the shallow floor, not a limit you are hitting.
app-perf-orbit-len = orbit len
# $cap: the most orbit samples this GPU holds, already formatted, or the text of app-perf-uncapped.
app-perf-orbit-len-hover = Reference orbit samples. This GPU holds at most { $cap } — approaching it is the practical depth wall: past it the live view cannot resolve deeper locations that need a longer orbit.
app-perf-uncapped = (uncapped)
app-perf-sa-skip = SA skip
app-perf-sa-skip-hover = Iterations the series approximation skipped before per-pixel work began. Higher is faster and changes nothing about the image; the row is absent when it skipped none.
app-perf-aa = aa
app-perf-aa-hover = Supersampling: each pixel is rendered from this many samples per axis, so 2x costs four times the work.
app-perf-dual = dual
app-perf-dual-hover = Whether the split Mandelbrot/Julia view is running. When true, every cost on this panel covers BOTH panes.
app-perf-zoom = zoom
app-perf-zoom-hover = Magnification relative to the full view. The arithmetic mode above changes at about 1e4x and again at about 1e28x.
app-perf-iterate = iterate
# $target and $lethal: times in milliseconds, already formatted.
app-perf-iterate-hover = Measured GPU time of the last iterate dispatch. The controller aims at { $target } ms; sustained readings near { $lethal } ms are the band where this hardware class has lost the device.
app-perf-steps = steps/s
app-perf-steps-hover = Iteration steps per second on the GPU, in billions. Interior-heavy views run slower per step than the count alone suggests, so this is a throughput reading rather than a score.
app-perf-budget = budget
# After the budget's value while it is still converging; the hover below quotes it.
app-perf-budget-settling = (settling)
app-perf-budget-hover = Adaptive per-dispatch step budget, the controller's guard against a frame long enough for the driver to reset the GPU. "settling" means it is still converging from sparse readings and is not yet a trustworthy ceiling.
app-perf-rss = rss
# $mb and $peak: memory in megabytes, already formatted.
app-perf-rss-value = { $mb } MB (peak { $peak })
app-perf-rss-hover = Resident memory of the process, and the highest it has reached this session. Deep reference builds have peaked past 2 GB in the field with nothing else on screen saying so; amber past three quarters of system RAM.
app-perf-gpu-est = gpu est.
app-perf-gpu-est-hover = Estimated GPU-resident memory: reference orbits, BLA trees, iteration textures, the chunk-state ping-pong while a refinement runs, and the present-gate snapshot. Assembled from known allocation sizes; the driver's true figure is not portably readable.
app-perf-julia-re = julia c.re
app-perf-julia-re-hover = Real part of the parameter c the Julia pane is rendering - the point the cursor is over, unless pinned.
app-perf-julia-im = julia c.im
app-perf-julia-im-hover = Imaginary part of the parameter c the Julia pane is rendering.
app-perf-julia-c = julia c
# The julia c row's value when the parameter is held fixed.
app-perf-pinned = pinned
app-perf-pinned-hover = The parameter is held at a fixed point, so moving the cursor no longer changes the Julia pane.
app-perf-c-panel = c/panel
app-perf-c-panel-hover = Width of c-space spanned by the whole Mandelbrot panel. When this is far below one Julia pixel, hovering changes c but the Julia looks unchanged — expected at deep zoom, not a freeze.
app-perf-ref-recompute = ref recompute
app-perf-ref-recompute-hover = Duration of the most recent reference-orbit recompute. This is the pause before a deep view starts resolving, and the step the optional accelerated build speeds up.
app-perf-recompute-rate = recompute/s
app-perf-recompute-rate-hover = Reference orbits INSTALLED per second - ones whose result was actually used. Compare it with the row below: they differ by the work that was thrown away.
app-perf-builds-rate = ref builds/s
app-perf-builds-rate-hover = Reference builds SPAWNED per second, including ones whose result is discarded - a lookahead slot the dive never reaches, or a refused extension. Those cost full CPU and appear nowhere else: a prefetch spin once ran about 400 builds a second for six minutes while recompute/s read 2. Single digits is normal; a large gap between these two rows means work is being thrown away.
app-perf-recompute-total = recompute tot
app-perf-recompute-total-hover = Reference recomputes since launch. Useful as a rate over a known interval; a number climbing while nothing moves is the signal worth chasing.

## Help: the contents list (Help & reference…, F1)

# Each section's name in the contents list (about 165 px wide: keep them short), and its heading.
help-section-overview = Overview
help-section-navigation = Navigation
help-section-expressions = Coordinate expressions
help-section-options = Coloring & options
help-section-fractals = Fractals
help-section-how-it-works = How it works
help-section-command-line = Command line
help-section-shortcuts = Shortcuts
help-section-hardware = Recommended hardware
# Credits to the prior work Fractadyne builds on.
help-section-acknowledgments = Acknowledgments
# The third-party license notices.
help-section-licenses = Licenses
help-section-about = About

## Help ▸ Overview

help-overview-intro = Fractadyne is a native fractal explorer built for ultra-deep zooming and speed. It draws "escape-time" fractals — images created by repeating one simple formula at every pixel.
help-overview-what-is = What is an escape-time fractal?
help-overview-what-is-text = For each pixel the program runs a formula such as z → z² + c over and over, starting from zero. If the running value stays small forever, the pixel belongs to the set and is drawn dark. If it eventually grows without bound ("escapes"), the pixel is outside the set, and its color records how many steps that took. The infinitely detailed border between "stays" and "escapes" is the fractal.
help-overview-can-do = What you can do
help-overview-can-pan = Pan and zoom essentially without limit (position is exact at any depth).
help-overview-can-families = Switch between 25 built-in fractal families (ten, and five of them at higher powers), your own formulas, Life and L-systems, and view any escape-time family except Newton as a Julia set.
help-overview-can-recolor = Recolor with preset or custom gradients and several coloring methods.
help-overview-can-lighting = Add 3D relief lighting and glowing boundary contours.
help-overview-can-minibrots = Snap to minibrots, bookmark spots, and export high-resolution images.
help-overview-can-autopilot = Let the auto-zoom autopilot dive toward detail on its own.
help-overview-can-share = Share any spot as a small ".fdn" text snippet or file.
help-overview-can-tours = Run scripted tours and a hardware benchmark.
help-overview-first-steps = First steps
# "Seahorse Valley" is a place in the Navigate menu: use its name as that menu shows it. A and F1 are keys.
help-overview-first-steps-text = Open the Navigate menu and pick "Seahorse Valley", then roll the mouse wheel to zoom in. Drag to pan. Or just press A and let the autopilot dive for you. Press F1 at any time to return to this help.

## Help ▸ Coordinate expressions
## The expressions, numbers and function names here are typed as shown in every language:
## keep them exactly as written. Menu items in quotation marks are the menus' own labels.

help-expr-intro = Wherever Fractadyne takes a coordinate you can type an expression instead of a decimal: the Re / Im fields of Navigate → "Go to location…" (and each field of its polar mode), the center_re / center_im lines of a .fdn location, a tour script's re / im, and --center on the command line. The expression is evaluated with as many digits as the zoom needs, and the Go-to dialog keeps it with the view — so a point you enter as 1/3, or as -0.5 + 0.25*cos(pi/4), stays exactly on that point however deep you zoom, where a decimal would drift once its digits ran out.
help-expr-examples = Examples
help-expr-ex-third = An exact third. More digits appear as you zoom in.
help-expr-ex-neck = The neck of Seahorse Valley — the root of the period-2 bulb.
help-expr-ex-circle = A point on the circle of radius 1/4 around -0.5 (the polar mode composes this for you).
help-expr-ex-bulb = Re of the period-3 bulb root; its Im is sin(2*pi/3)/2 - sin(4*pi/3)/4. Navigate → "Exact points (expressions)" has these built in.
help-expr-ex-complex = A whole complex value typed in the Re field fills both Re and Im.
help-expr-ex-combine = Functions and constants combine freely.
help-expr-ex-powers = Powers: 2^-20 is 1/1048576. -2^2 is -4 — the minus is applied after the power.
help-expr-ex-root = -2, the real cube root. Even roots of a negative are refused; sqrt(-1) gives i.
help-expr-ex-scientific = Scientific notation works inside expressions too.
help-expr-numbers = Numbers and operators
# The decimal point in an expression is '.' in every language: keep that rule as it is.
help-expr-numbers-decimals = Decimals with an optional exponent: 0.25, -1.5e-30, .5. Digits only — no spaces or grouping commas inside a number, and the decimal point is always '.'.
help-expr-numbers-imaginary = Add an i suffix for an imaginary number (0.25i), or use i by itself: 3 + 4*i.
# Starts with the operators themselves, two spaces apart.
help-expr-numbers-operators = +  -  *  /  and ^ (power; evaluated right to left, so 2^3^2 = 2^9). Parentheses group. There is no implicit multiplication: write 2*pi, not 2pi.
help-expr-numbers-case = Names are case-insensitive: PI, Pi and pi are the same constant.
help-expr-constants = Constants
help-expr-functions = Functions
help-expr-functions-angles = Angles are in radians. For degrees, multiply by pi/180 — cos(45*pi/180) — or switch the Go-to dialog to its polar mode and pick the degrees unit.
help-expr-refused = What is refused, and why
help-expr-refused-fractional = A negative number to a fractional power, such as (-2)^(1/2): the result is complex and there are several roots to choose from. Use root(x, n) or cbrt(x) for a real root, or sqrt(x) for the principal complex one.
help-expr-refused-complex = A complex argument to a real-valued function (sin(i), ln(1+i)). Only sqrt and abs accept a complex argument; the arithmetic operators and integer powers work on complex values.
help-expr-refused-domain = Domain errors: ln or log of a number that is not positive, asin or acos outside -1…1, 0 to a negative power, division by zero.
help-expr-refused-bounds = Arguments to sin, cos, tan, exp or ^ beyond ±2^32, and nesting deeper than 32 levels: a pasted coordinate is untrusted input, and those would take unbounded time.
help-expr-refused-grammar = No variables, user-defined functions, comparisons or conditionals. The grammar is kept deliberately small so a saved location means the same thing in every version.
help-expr-no-parse = If it does not parse
help-expr-no-parse-text = The message names the field, says what was found and at which character, and — for the usual slips — what to type instead: a typographic minus sign (−) pasted from a document, × or ÷, a decimal comma, a missing * between a number and a name, an unclosed parenthesis, or a function written without its parentheses. Nothing is ever half-read: if any part of the text cannot be understood, the whole coordinate is refused rather than silently shortened.

## Help ▸ Navigation
## Menu paths (Navigate → "Go to location…") and quoted items name the interface's own
## menus and controls: use their translations. Key names (Ctrl, Shift, Space, Esc) stay.

# A heading: the mouse controls (also in Shortcuts).
help-mouse = Mouse
# The left column of a controls table: a gesture or key. Keep key names (Shift, Space, Ctrl) as they are.
help-key-left-drag = Left-drag
help-nav-left-drag = Pan the view.
help-key-mouse-wheel = Mouse wheel
help-nav-wheel = Zoom in/out toward the cursor; turn it faster to zoom further per notch. With Snap zoom to features on, zooming in near a minibrot pulls the zoom onto it.
help-key-shift-drag = Shift+drag
help-nav-shift-drag = Box zoom — drag a rectangle and it zooms to fill the view.
# $icon: the click-to-zoom toolbar button's symbol.
help-key-click-zoom = Click-to-zoom ({ $icon })
help-nav-click-zoom = Toggle the target button in the toolbar (single view): a left-click then dives into the point by a set factor, right-click backs out. Drag still pans; pick the factor (2–100×) under Controls ▸ Navigate. Backspace undoes a click.
help-nav-continuous = Continuous zoom & home
help-key-hold-space = Hold Space
help-nav-hold-space = Smoothly zoom in, anchored at the cursor.
help-key-hold-shift-space = Hold Shift+Space
help-nav-hold-shift-space = Smoothly zoom out.
help-nav-continuous-text = The Zoom-home button animates a gentle fly-back to the full view. "Zoom speed" in the right panel sets the continuous-zoom rate.
help-nav-autopilot = Autopilot
help-nav-autopilot-text = Press A (or Tools → "Auto-zoom (autopilot)") for a hands-free dive: it picks a point on the fractal's edge where the detail is densest, preferring the middle of the view, and glides into it while easing it to the centre of the screen. It keeps that target while it stays interesting and turns smoothly toward richer structure when some appears. Any navigation input — or Esc — stops it. "Auto-zoom priority" in the right panel (or under Tools) chooses what it puts first: Speed zooms at the set rate and refreshes the screen as soon as a frame with detail is ready; Quality shows only fully resolved full-resolution frames and slows the zoom as needed so the picture never magnifies far before the next one lands. Either way the screen never shows a blank or single-colour frame: a refresh that has not yet resolved any detail stays hidden behind the last one. "Auto-zoom target" chooses what it aims at: Detail (the richest edge in view, re-chosen as it goes) or Misiurewicz point — it finds the nearest Misiurewicz point, solves it to the dive limit's depth while holding still, then dives exactly into it; the structure around such a point repeats every few octaves forever, so that dive never dead-ends (Mandelbrot set only).
help-nav-history = History & precise moves
help-nav-undo = Undo the previous view.
help-nav-redo = Redo.
help-nav-goto = Navigate → "Go to location…" lets you read, type, paste, or copy the exact center and zoom (full precision, any depth). The center fields accept more than plain numbers: fractions like "1/3" and expressions built from pi, tau, e, sqrt and sin/cos/tan — so "-0.5 + 0.25*cos(pi/4)" is a valid coordinate. An expression is kept with the view and re-evaluated at the precision each depth needs, so a point entered that way stays exact no matter how far you zoom in (a rounded decimal eventually would not). A polar mode (offset + r ∠ θ, in degrees / radians / turns) composes one of these expressions for you. The full list of functions and constants, with examples, is the "Coordinate expressions" section of this help (the ? button in the dialog opens it). Navigate → Bookmarks saves and recalls locations.
help-nav-exact-points = Navigate → "Exact points (expressions)" jumps to points defined exactly rather than by a rounded decimal — the roots of the main-cardioid bulbs, where each period-q bud attaches, written as cos/sin expressions. Because the expression travels with the view, you can zoom into one without it ever drifting off the point.
help-nav-share = Navigate → "Share location…" captures the whole view — fractal, full-precision center, zoom, and coloring — as a compact ".fdn" snippet you can Copy, paste back and Apply, or Save/Load as a file, to reproduce a spot exactly (or send it to someone).
help-nav-open = File → "Open view or location…" opens any of these directly: an exported PNG/EXR (its embedded view is restored), a ".fdn" location file, or a Kalles Fraktaler ".kfr".
help-nav-finding = Finding detail
help-nav-minibrots = Minibrots are tiny copies of the whole set hidden along the boundary. Center one roughly and press M (or Tools → "Find minibrot + zoom to it") to Newton-snap exactly onto its center, read its period, and zoom straight to its own scale — often many orders of magnitude in one step. If you are already deeper than the minibrot, only the center moves.
help-nav-minimap = View → "Minimap overview" shows a small map of the whole set with a "you are here" marker and the current zoom depth; click the map to jump to a region.

## Help ▸ Coloring & options
## Names in quotation marks are the controls' own labels in the right-hand panel.

help-options-intro = All of these live in the right-hand panel (and persist between sessions).
help-options-palette = Palette
help-options-palette-text = Palette chooses a preset gradient, your Custom one, or a two-color mode. "Edit gradient…" opens an editor where each color stop has a color and a position (0–1); add stops, copy a preset to start from, or import a palette file.
help-options-paste = "Paste…" in the gradient editor imports a palette you found elsewhere. It accepts hex colors with or without a leading # (#ff8800, ff8800, or the short #f80) and plain 0–255 RGB triples such as "255 136 0" — the format Fractint and Kalles Fraktaler .map files use — separated by commas, spaces or new lines. Up to 256 colors are kept; a longer list is sampled evenly down to 256, keeping the first and last so the gradient's shape survives.
help-options-import-map = "Import ▸ .map…" beside it loads a Fractint or Kalles Fraktaler .map palette file. Those are lookup tables indexed by iteration count with no blending between entries, and the hard color steps that produces are what the classic Fractint look is made of — so that is how they import, with a "Hard bands" checkbox to blend the same colors into a smooth gradient instead. Fractint's own files store 6-bit color values multiplied by four, so their white is 252 rather than 255; Fractadyne says so when it imports one and leaves the values exactly as written, because Fractint's own saved images carry the same 252.
help-options-duotone = Duotone maps the value to a smooth two-color ramp (Shadow → Highlight). Binary (set) is a flat two-color view — one solid color for points inside the set and another for outside, with no gradient — the clearest way to see the set's shape.
help-options-cycle = Cycle sets how many times the gradient repeats across the iteration range (tighter or looser bands). Offset rotates the whole gradient.
help-options-normalize = "Normalize deep colors" fits the palette to the range of escape values actually present in the view. Deep views span hundreds of thousands of iterations, and without this a fixed cycle wraps the gradient thousands of times between neighboring pixels, turning correct detail into speckle.
help-options-log-scale = "Log color scale" spreads the palette by the logarithm of the escape value instead of linearly. Escape counts crowd toward the high end as you go deeper, so a linear fit spends most of the gradient on a thin shell hugging the boundary and leaves the rest nearly flat; the log scale opens that out, and keeps colors steadier through a zoom video as the range grows.
help-options-animate = Animate cycles the colors over time — Forward, Reverse, or Ping-pong shift the offset; Random continuously synthesizes smoothly morphing, harmonious gradients. Speed controls cycles (or morphs) per second; "Shuffle gradient" rolls a new one in Random mode.
help-options-method = Coloring method — how the data becomes color
# The coloring methods, by the names the Coloring panel gives them, each with what it does.
help-method-smooth = Smooth iteration
help-method-smooth-text = Classic continuous bands by escape time.
help-method-stripe = Stripe average
help-method-stripe-text = Flowing bands from the orbit's angle (Stripe density slider).
help-method-triangle = Triangle inequality
help-method-triangle-text = Fine texture from where each step lands between bounds.
help-method-trap = Orbit trap
help-method-trap-text = Distance of the orbit to a shape (point/cross/circle); colors interior too.
help-method-distance = Distance estimate
help-method-distance-text = Shades by nearness to the boundary.
help-method-decomposition = Decomposition
help-method-decomposition-text = Cells from the final escape angle.
help-options-relief = 3D relief lighting
help-options-relief-text = Shades the surface from the boundary's slope (the derivative) for an embossed, lit look. Light angle sets the direction; Relief sets strength (lower = sharper); "Rotate light" animates it. Holomorphic families only (Mandelbrot / Multibrot).
help-options-glow = Distance glow
help-options-glow-text = Bright contour bands that densify into glowing filaments near the boundary. Glow is the blend amount, Band width the spacing, and "Animate glow" flows them.
help-options-quality = Quality & iterations
help-options-quality-text = Iterations is the maximum number of steps before a pixel is treated as inside the set; "Auto-scale iterations with zoom" raises it automatically as you zoom (deeper detail needs more). Anti-alias supersamples still images (2×–8×) once the view settles, taming the fine exterior "dust".
help-options-other = Other
help-options-other-text = Zoom speed sets the continuous-zoom rate; the FPS cap limits frame rate; "Both, linked" shows a Mandelbrot set and its Julia set side by side (the cursor sets the Julia parameter live).

## Help ▸ Fractals
## Fractal family names (Mandelbrot, Multibrot, Tricorn, Burning Ship, Celtic, Buffalo,
## Phoenix, Newton) are names: keep them as written, as the formulas. Formula syntax
## (z = z^3 - p1*z + c, sqr, |z|, if / elseif / endif, p1-p5, maxit) is typed as shown.

help-fractals-intro = Every family iterates a formula with z starting at 0 and c set by the pixel (escape-time), unless noted. z = x + iy is a complex number.
help-fractals-mandelbrot = z → z² + c. The original. The set is every c whose orbit stays bounded; it is connected, and its boundary is so crinkled it has Hausdorff dimension 2.
help-fractals-multibrot = z → z^d + c for power d = 3, 4, 5. Higher powers add lobes: the set has (d−1)-fold rotational symmetry (Multibrot 3 is 2-fold, 4 is 3-fold, 5 is 4-fold).
help-fractals-tricorn = z → conj(z)² + c, where conj(x + iy) = x − iy. The conjugation makes it anti-holomorphic, giving 3-fold symmetry and characteristic curved "claws".
help-fractals-burning-ship = z → (|x| + i|y|)² + c — take absolute values before squaring (real = x²−y²+cx, imag = 2|xy|+cy). Non-analytic; deep zooms reveal ship-like structures (traditionally viewed upside-down).
help-fractals-celtic = Like Mandelbrot but with the absolute value of the real part: real = |x²−y²| + cx, imag = 2xy + cy. Produces heart- and shield-shaped motifs.
help-fractals-buffalo = Absolute value of both parts of z²: real = |x²−y²| + cx, imag = |2xy| + cy — a cross between Celtic and Burning Ship.
# A heading: the families at powers above 2 (the Fractal menu's "Higher powers").
help-fractals-powers = Higher powers
# ("Burning Ship  3 4 5") is how the Fractal menu shows such a row: keep it as it is.
help-fractals-powers-text = Multibrot 6, 7 and 8 (z^d + c), and Burning Ship, Tricorn, Celtic and Buffalo at powers 3, 4 and 5: the same folds at a higher power. Burning Ship d takes the absolute values of z's parts and then the power; Tricorn d conjugates first; Celtic d and Buffalo d fold the real part (and, for Buffalo, the imaginary part) of z^d. The Fractal menu and the dropdown list them one family to a row ("Burning Ship  3 4 5"). Each deep-zooms as its power-2 sibling does, and Multibrot 6, 7 and 8 also take series approximation and the minibrot finder (M), as Multibrot 3–5 do. Multibrot 8 escapes at radius 128 instead of 256, where its next step would leave single-precision range.
help-fractals-phoenix = z → z² + c + p·z(n-1), where z(n-1) is the previous iterate and p is a constant (p = -0.5). The memory term produces flame-like filaments.
help-fractals-newton = Newton's method for the roots of z³ − 1 = 0: z → z − (z³−1)/(3z²). Rather than escape time, pixels are colored by which of the three cube roots of unity the iteration converges to (the basins of attraction) and how quickly; the tangled basin boundaries are the fractal.
help-fractals-julia = Julia mode
help-fractals-julia-text = For every family except Newton you can switch to a Julia set: instead of starting z at 0 with c = pixel, you fix c (a parameter) and let z start at the pixel. The Julia set is the boundary between starting points that stay bounded and those that escape, for that fixed c. In "Both, linked", moving the cursor over the Mandelbrot panel sets c live.
help-fractals-deep = Deep-zoom support
help-fractals-deep-text = Mandelbrot, Multibrot 3/4/5 and Tricorn support extreme-depth (floatexp) perturbation deep zoom. Burning Ship, Celtic and Buffalo are non-analytic (they take absolute values), so they use a sign-aware perturbation; this now runs at floatexp range too, deep-zooming far past the old ~1e28× df32 limit (rare speckle near the abs folds awaits multi-reference glitch correction). Phoenix uses a two-term perturbation and also deep-zooms at floatexp range (without SA/BLA, so heavier than Mandelbrot). The higher powers deep-zoom the same way, each power's perturbation following its own chain of squarings; Multibrot 3–8 also take series approximation and BLA, as Mandelbrot does, and Tricorn, Burning Ship, Celtic and Buffalo at every power take BLA (no series), which carries a deep view's interior instead of iterating it step by step. Only Newton uses the direct path, sharp to ~1e6×; custom formulas deep-zoom unless the Custom formulas section below says otherwise.
help-fractals-custom = Custom formulas
help-fractals-custom-text = Fractal ▸ Custom formula… takes a formula's step in Fractint-style expressions: z = z^3 - p1*z + c, or several statements (t = sqr(z), then z = t*t + c). z is the iterate, c the pixel (the Julia constant in Julia mode), p1-p5 parameters; |z| is the squared modulus, as in Fractint, and the usual functions are available (sqr, abs, conj, real, imag, cabs, flip, recip, ident, exp, log, sqrt, sin, cos, tan, cotan, sinh, cosh, tanh, cotanh, asin, acos, atan, asinh, acosh, atanh, cosxx, and floor, ceil, trunc and round, which round each part). log is the natural logarithm, as in Fractint and in complex analysis: what many textbooks write ln (there is no base-10 logarithm). A formula written with ln, cot, coth, Re or Im does not read; the syntax check names the function to use, a button rewrites it, and typing ln offers log. Every step deep-zooms by perturbation, with the same extended-range arithmetic past 1e28x as the built-in fractals, except one with a power whose exponent varies with z or c (such as z^c): that renders on the direct path, sharp until one single-precision step of c spans a pixel, about 1e4x to 1e5x, and less deep away from the origin. The formula dialog says which applies. log, sqrt and non-integer powers take their principal values, which jump across the negative real axis. Perturbation follows the jump exactly, but a pixel whose orbit crosses where the reference's does not is carried on at about single precision from then on, so deep views whose orbits straddle a cut show noise there (measured: clean at 1e5x on the real axis for sqrt(z^4 + c), noisy for log formulas even at 1e4x). Every step is iterated in full (no series approximation), so a deep view with a high iteration count renders more slowly than a built-in fractal's. Near its boundary a formula with functions orbits chaotically, and a long orbit there is followed only loosely in single precision: at 2,000 iterations a third of sin(z) + c's pixels differ from the exact orbit's, at 60 almost none. Views, sessions and exported images keep the formula's text.
help-fractals-custom-sections = Fractint's sections work too. Statements before a ':' are the init section, run once per pixel; a comparison as the last statement is the formula's own bailout test, iterating while it holds (|z| <= 100 && |z - 1| > 0.000001 stops an orbit that escapes or settles on 1); if (...), elseif (...), else and endif choose statements; and a variable read before it is set in a step has its value from the step before (from the init section, or 0). Comparisons compare real parts and give 1 or 0, && and || combine them, and an assignment is a value too (a = b = pixel). pixel, p1-p5, pi and e may be assigned, keeping their own value until they are; maxit is the iteration count and ismand is 1 (the formula's Mandelbrot form). A formula with any of these, or with rounding, renders on the direct path, without deep zoom. Formula library ▸ Import reads Fractint .frm files as well. Each entry that reads becomes a formula, and its row says what the reading changed: fn1-fn4 take Fractint's defaults (sin, sqr, sinh, cosh); Fractint's variable c is renamed c_, as c is the pixel here; a name nothing sets is 0, as in Fractint; blanks inside a name are dropped (end if is endif), as Fractint drops them. The import's message counts the entries that do not read and gives the commonest reason: the screen and view variables (whitesq, scrnpix, scrnmax, center, magxmag, rotskew), lastsqr and random numbers are not supported. The Collection's Fractint classics (Magnet, Nova, Spider, Manowar and others) are written with these sections.
help-fractals-custom-editor = In the formula field, parentheses take a colour per nesting level; the pair next to the cursor is highlighted, and one without a partner is shown in red (a comment's do not count, as the parser ignores them). Names complete as you type: after two letters a list offers the functions, names and your own variables that begin so. Up and Down choose, Tab or Enter takes one (a function comes with its opening parenthesis), Esc closes the list, and Enter after a name typed in full still starts a new line.
help-fractals-custom-textbook = Textbook (the switch above the formula) shows it typeset as a textbook sets it, and edits it in place. Type as in the text: / makes a fraction of the term before it, ^ an exponent (+, - or = typed at an exponent's end steps out of it), ( parentheses, or a call after a function's name, | bars, and a comma inside parentheses a complex constant. The arrows move through the formula, Up and Down between a numerator and its denominator, Tab to the next empty box; Backspace at the start of a fraction or a pair of parentheses selects it, and a second press removes it and keeps what it held. Copy, paste and undo work as in a text field, names complete as they do there, and the keypad types into the typeset formula (□² squares what is before the caret, a function key wraps the selection). Every edit is written back to the text, so the two views always hold the same formula; a line you have not edited keeps its spacing and comment. The notation never changes what is computed: |z| is the squared modulus and shows as |z|², cabs(z) shows as |z|, and exp(z) stays exp(z). A line that does not read shows in red; click it to edit it as text. LaTeX copies the formula as LaTeX source, as it is typeset, and while Textbook is on the formula library shows its formulas typeset too.
help-fractals-custom-numbers = Numbers in a formula (0.1, 1/3, pi, the parameters) are read as double-precision values, about 16 digits, and the deep reference orbit iterates with exactly those values. Two formulas that are equal on paper can therefore differ deep down: 0.1*z multiplies by the double nearest 0.1, while z/10 divides by an exact 10, and the two pictures part past about 1e15x (measured: 5% of pixels at 1e15x, a third at 1e17x, all of them at 1e20x). Either picture is correct for its own formula; write the one you mean to share.
help-fractals-custom-library = The dialog's Save to library keeps a formula and its parameters under a name (saving under a name the library has updates that entry). Fractal ▸ Formula library… lists them to apply, edit, export or delete; Import adds the formulas in a formula file without replacing any: one the library already has is skipped, and a new one whose name is taken becomes "name (2)". The library is formulas.toml in the folder named under Files & data (About), itself a formula file. A tour keyframe carries a formula as formula = "..." with formula_params = [[re, im], ...], and Tools ▸ Tour from current view… writes them for a custom view.
help-fractals-custom-collection = A collection of 45 formulas comes with Fractadyne, written for it: higher powers and the folded variants of the Burning Ship and Celtic families, blends and two-step hybrids, the Mandelbrot set seen through another map (inverted, log-polar, through sine), transcendental and rational formulas, and Julia sets. They are the dialog's Examples and the Collection tab of the formula library, where Copy to mine adds one to your formulas. Each has a starting view, and Apply goes there: a formula opened on the home view is often not where its picture is. A formula you save while it is showing keeps the view you are at in the same way, and Apply returns to it.

## Help ▸ How it works

help-how-escape = Escape-time & smooth color
help-how-escape-text = Each pixel iterates the formula until its magnitude exceeds a bailout radius or it hits the iteration cap. The raw step count alone makes hard bands; adding a fractional part derived from the final magnitude gives continuous, bandless color.
help-how-precision = Arbitrary-precision position
help-how-precision-text = Ordinary 64-bit numbers run out of digits near 1e15× zoom. Fractadyne keeps the view center in arbitrary precision, with the number of digits growing as you zoom, so the location never degrades. The pixel scale is likewise held with an extended exponent, so it doesn't stall at 64-bit's ~1e308× limit either — depth is bounded only by patience, not by any fixed number range.
help-how-perturbation = Perturbation
help-how-perturbation-text = Iterating every pixel in high precision would be far too slow. Instead one reference pixel is iterated in high precision on the CPU (the "reference orbit"), and every other pixel is computed on the GPU as a tiny difference δ from it in fast low precision: δz → 2·Z·δz + δz² + δc.
help-how-floatexp = Extreme depth (floatexp)
help-how-floatexp-text = Past about 1e28× even that tiny difference underflows 32-bit range, so it is stored as a mantissa plus a separate integer exponent ("floatexp"), lifting the 32-bit depth wall. The engine switches automatically: direct math when shallow, perturbation when deep, and floatexp when deepest. Depth is then bounded by coordinate precision and the iteration budget rather than a fixed limit — cross-checked against Fraktaler-3 across a 39-location reference corpus, the deepest at 5.63e18003× (the 38 non-extreme rows are the routine gate, to 6.13e1105×), with an internal precision self-consistency battery to 1e1000000×; the bundled tour reaches ~1.3e420×.
help-how-reference = Reference choice & rebasing
help-how-reference-text = The reference is chosen (scored in high precision) so its orbit stays within the view as long as possible. When the difference grows too large it is "rebased" back onto the reference to stay accurate.
help-how-series = Series approximation
help-how-series-text = At deep zoom the early iterations barely move δ from the reference, so they can be computed all at once: δz is approximated by a short polynomial in δc (order 3), whose coefficients are iterated along the reference. The renderer skips ahead to the last iteration where that polynomial is still accurate for the whole view, then iterates normally — saving the skipped steps with no change to the image (toggle in Controls ▸ Advanced). Mandelbrot and Multibrot 3–8, deepest range; not with stripe / triangle-inequality / orbit-trap / decomposition coloring (those need every iteration). Where the BLA is active it already subsumes this early skip.
help-how-distance = Distance estimation & lighting
help-how-distance-text = Tracking the derivative dz/dc yields each pixel's distance to the boundary. That powers the 3D relief lighting, the distance glow, and the "distance" coloring method — all valid at any zoom depth.
help-how-safety = Anti-aliasing & safety
help-how-safety-text = Still images are supersampled (2–8× per axis) once the view settles. A work budget keeps a single GPU draw within the driver's watchdog limit by reducing resolution (never the iteration count) at extreme settings, so deep views stay detailed instead of going blank.

## Help ▸ Command line (the flags themselves stay English)

# The section's introduction. The flags listed below it are the command line itself, English in every language.
help-cli-intro = Fractadyne runs headless for rendering, movie/tour export, automation, golden-image checks, and benchmarking. With no flags it launches the interactive GUI.

## Help ▸ Shortcuts

# What each mouse gesture and key does (the gestures are the Navigation section's).
help-shortcuts-pan = Pan
# A gesture: turning the mouse wheel.
help-key-wheel = Wheel
help-shortcuts-wheel = Zoom at cursor
help-shortcuts-box-zoom = Box zoom (drag a rectangle to zoom to it)
# $icon: the click-to-zoom toolbar button's symbol. "tool on": while that button is on.
help-key-click-tool = Click ({ $icon } tool on)
help-shortcuts-click = Dive into the point; right-click backs out
# A heading: the keyboard shortcuts.
help-shortcuts-keyboard = Keyboard
help-shortcuts-space = Continuous zoom in / out
help-shortcuts-undo = Undo view
help-shortcuts-redo = Redo view
help-shortcuts-minibrot = Find minibrot + zoom to it (Newton-Raphson)
help-shortcuts-autopilot = Auto-zoom autopilot (dive toward detail; any input stops)
# off / strip / panel / full: the panel's sizes, as View ▸ Live render panel names them.
help-shortcuts-live-panel = Live-render panel: off / strip / panel / full (hover an instrument for what it means; its × closes it)
help-shortcuts-esc = Stop autopilot / a playing tour, or exit fullscreen
help-shortcuts-snapshot = Snapshot to the last export folder — a screen capture of the view as shown, or a full render at the export settings; the first press asks which, and File → Settings → Snapshot changes it.
help-shortcuts-help = Open this help
# A heading: the shortcuts of the Game of Life, with the menu path to it.
help-shortcuts-life = Life (Fractal → Automata → Life)
help-shortcuts-life-run = Run / pause the universe
help-shortcuts-life-step = Step one generation / step the stride (set in the Life panel)
help-shortcuts-life-draw = Draw tool on / off (the pencil on the toolbar)
# A gesture. "Draw on": while the Life draw tool is on.
help-key-click-drag-draw = Click / drag (Draw on)
help-shortcuts-life-paint = Flip a cell / paint: a stroke that starts on a dead cell draws, one that starts on a live cell erases. A running universe pauses while you draw and runs on when you let go.
# $key: a keyboard shortcut (Ctrl+Z). "Draw on": while the Life draw tool is on.
help-key-when-drawing = { $key } (Draw on)
help-shortcuts-life-undo = Undo the last edit, fill, clear or rule change

## Help ▸ Recommended hardware

help-hardware-intro = Fractadyne splits its work between the GPU (per-pixel iteration + coloring) and the CPU (the arbitrary-precision reference orbit that deep zoom is built on). Both matter, but for different things.
help-hardware-gpu = GPU (most important)
help-hardware-gpu-text = Every pixel is iterated and colored on the GPU, so GPU speed sets the frame rate and export time. A discrete GPU with strong 32-bit (single-precision) throughput gives the smoothest deep zooming. Integrated GPUs work fine — just slower, and they reach the per-frame work budget sooner (at which point the renderer trims resolution, not detail). Requires a modern graphics API: Vulkan, Direct3D 12, Metal, or OpenGL via wgpu.
# The left column of a requirements table.
help-hardware-minimum = Minimum
help-hardware-gpu-minimum = Any GPU with Vulkan / DX12 / Metal / GL 3.3.
help-hardware-recommended = Recommended
help-hardware-gpu-recommended = A discrete GPU (last ~5 years); >= 2 GB VRAM.
help-hardware-big-exports = For big exports
help-hardware-gpu-big-exports = >= 4 GB VRAM (tiled, so it degrades gracefully).
# A heading: the processor.
help-hardware-cpu = CPU
help-hardware-cpu-text = The high-precision reference orbit (and BLA tables) are computed on the CPU and are largely single-threaded, so fast per-core performance matters most — it drives how quickly a deep view resolves after you move. Extra cores mainly help background export encoding. A 64-bit CPU is required.
help-hardware-cpu-recommended = A modern 64-bit CPU with good single-core speed.
help-hardware-memory = Memory
help-hardware-memory-text = Deep references grow with depth (many iterations at high precision); huge exports hold tiles in RAM. 8 GB is comfortable for everyday exploring; 16 GB+ suits extreme depth and very large exports.
help-hardware-checking = Checking your hardware
help-hardware-diagnostics = Help → "Diagnostics…" runs the same tests the developers do, on your machine. "Run self-test" checks the mathematics and rendering against known-correct results in about fifteen seconds; "Run UI test" walks the interface and the live view at several zoom depths, capturing screenshots; "Run GPU arithmetic check" verifies the shader's double-float arithmetic on every graphics backend your machine offers, in a few seconds and with no window. Each runs as a separate process, so a test that upsets the graphics driver takes the test down rather than your session.
help-hardware-gpu-check = The GPU arithmetic check has no pass or fail — it records how your particular GPU and driver compute, and on some hardware (every NVIDIA card tried so far) it reports that the shader compiler discards part of the extended-precision arithmetic. That is a genuine finding we are trying to map across vendors, not a fault in your machine, so whatever it says, attaching it to a report genuinely helps.
help-hardware-report = If something looks wrong, run the self-test and use "Attach to an issue report…": the report then carries a machine-validated result from your own hardware, which is far more useful than a description alone. One caveat worth knowing — the reference images were produced on one particular graphics card, and different GPUs legitimately compute slightly different pixels, so image comparisons are checked against a wider tolerance on other hardware and small differences there are expected rather than faults.
help-hardware-notes = Notes
help-hardware-note-watchdog = A capable GPU avoids the watchdog work-budget kicking in at extreme settings.
help-hardware-note-binary = The build is single-binary and self-contained — no runtime install needed.
help-hardware-note-benchmark = Run the built-in benchmark (Tools ▸ Benchmark…) to gauge your machine.

## Help ▸ Acknowledgments & citations
## Names of people, programs, papers, books and websites stay as written, and so do the
## titles in quotation marks (the works' own titles); the prose around them translates.

help-ack-title = Acknowledgments & citations
# The dedication, in italics at the top of the section.
help-ack-dedication = Dedicated to the Stone Soup Group — the volunteers behind Fractint (from 1988).
help-ack-dedication-text = Their freely shared code and spirit of open, collaborative fractal exploration are the reason so many of us fell in love with this art. Fractadyne is built in that tradition.
help-ack-intro = Fractadyne stands on decades of work by the fractal community. Techniques and prior art it builds on — verified against the sources below:
help-ack-deep-zoom = Deep-zoom algorithms
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-martin-title = Perturbation & series approximation — { $who }
help-ack-martin = The reference-orbit + low-precision-delta method (δz → 2Z·δz + δz² + δc) and the order-n series that skips early iterations, introduced in SuperFractalThing and its note “SuperFractalThing Maths” (sft_maths.pdf, 2013).
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bla-title = Bivariate Linear Approximation (BLA) & rebasing — { $who }
help-ack-bla = Skipping runs of iterations via merged linear maps, and rebasing to a single reference to avoid glitches; developed on fractalforums.org (2021).
# A link's label. $who: the linked page's author.
help-ack-bla-link = BLA explained — { $who }
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-glitch-title = Glitch-detection criterion — { $who }
help-ack-glitch = The |Z + z| << |Z| test for when a pixel needs a different reference — the basis of glitch-free perturbation (fractalforums).
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-diffabs-title = Non-analytic (Burning Ship) perturbation — { $who }
help-ack-diffabs = The absolute-value “diffabs” case analysis that makes perturbation work for the abs-based families (Burning Ship / Celtic / Buffalo), on fractalforums.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mathr-title = Reference implementations & cross-checks — { $who }
help-ack-mathr = Fraktaler-3 and Kalles Fraktaler 2+ (originally by Karl Runmo) — whose write-ups guided this engine, and whose EXR output Fractadyne cross-checks its renderer against.
# A heading: credits for the coloring methods.
help-ack-coloring = Coloring
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-harkonen-title = Smooth iteration count & Stripe Average — { $who }
help-ack-harkonen = The continuous (normalized) escape-time coloring and the Stripe Average method, from “On Smooth Fractal Coloring Techniques” (M.Sc. thesis, 2007), which also analyzes the Triangle Inequality / branching-average colorings.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mitchell-title = Triangle Inequality Average — { $who }
help-ack-mitchell = Averaging where each z² + c lands within the triangle-inequality bounds, for smooth flame-like exterior texture.
# A link's label: the person's tutorial pages. $who: the person's name.
help-ack-mitchell-link = { $who } — tutorials
help-ack-distance = The distance-estimate coloring/lighting uses the classic dz/dc boundary-distance method (see Peitgen & Saupe, “The Science of Fractal Images”, 1988).
help-ack-lsystems-life = L-systems & the Game of Life
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-lindenmayer-title = L-systems — { $who }
help-ack-lindenmayer = Parallel string rewriting, introduced in 1968 to model how plants and algae grow (“Mathematical models for cellular interactions in development”, Journal of Theoretical Biology).
help-ack-abop = The book (1990) whose turtle interpretation and notation for stochastic, context-sensitive and parametric systems the L-system reader follows; the built-in systems marked ABOP are entered from its figures.
# A link's label: the book can be read free on that website. $site: the website's name.
help-ack-abop-link = The book, free from { $site }
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bourke-title = L-system collection — { $who }
help-ack-bourke = Many of the built-in systems come from his L-system pages, among them Chris Wallace's rules for Sierpinski's curve and Dietmar Saupe's bush. Fractint's .l files, which open directly, are in the Stone Soup Group's format.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-conway-title = The Game of Life — { $who }
help-ack-conway = The cellular automaton B3/S23 (1970), which reached the world through Martin Gardner's “Mathematical Games” column in Scientific American (October 1970).
# A credit's heading: the website, then who runs it. $site: the website's name.
help-ack-lifewiki-title = { $site } — the ConwayLife.com community
help-ack-lifewiki = The reference for the rule notations the app reads (B/S, von Neumann, Generations, Alan Hensel's letters for non-totalistic rules, and MAP), for the RLE pattern format it shares with Golly, and for the built-in rules and patterns, whose discoverers it records.
help-ack-foundations = Foundations & tools
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mandelbrot-title = The Mandelbrot set — { $who }
help-ack-mandelbrot = The object at the heart of it all.
help-ack-built-with = Built with Rust, egui/eframe (emilk) and wgpu (gfx-rs); arbitrary precision via astro-float, double-word floats via twofloat; PNG/OpenEXR, rfd, serde. Thanks to their authors and maintainers.
help-ack-errors = Any errors in attribution are the author's; corrections are welcome on the project's issue tracker.

## Help ▸ Third-party licenses

help-licenses-title = Third-party licenses
# $licenses: Fractadyne's license expression (MIT OR Apache-2.0), as written. $url: the web address of a crate's source. THIRD-PARTY-NOTICES.md is a file name, `option-ext` a crate's name, MPL-2.0 a license's.
help-licenses-intro = Fractadyne is distributed as a statically-linked binary that includes the open-source crates and bundled fonts below. The full license texts (also in THIRD-PARTY-NOTICES.md, included with the download) follow. Fractadyne's own code is dual-licensed { $licenses }. `option-ext` is MPL-2.0 — its unmodified source is at { $url }.
# A button, after a copy icon: copies every notice to the clipboard.
help-licenses-copy-all = Copy all notices

## Help ▸ About

help-about-tagline = A native fractal explorer built in Rust with wgpu.
# The arithmetic deep zoom has used. $ran: the libraries that have run so far, as names (or a note that none has yet); $built: those this build contains, with versions.
help-about-arithmetic = Deep-zoom arithmetic: { $ran } (this build contains: { $built }).
help-about-files = Files & data
# Followed by the folder's path.
help-about-files-text = Your session (current view, coloring, and preferences), bookmarks and their thumbnails, saved gradients and saved formulas are stored in this per-user folder, and reloaded on the next launch:
# Tooltip of the copy button beside the folder's path.
help-about-copy-path = Copy path
# $command: a command typed in a terminal (it stays English).
help-about-session = The session file is versioned; if it was written by a newer Fractadyne than can fully read it, the app loads it best-effort and warns you. To start completely fresh, use File ▸ Reset application state… (asks first), or run `{ $command }` from a terminal.
help-about-license = License
# $licenses: the license expression (MIT OR Apache-2.0), which stays as written.
help-about-license-text = { $licenses } — use under either license, at your option.
# A link to the source code, followed by ↗.
help-about-source = Source on GitHub

## The live render panel ("blinkenlights": View ▸ Live render panel, key B)
## A front panel drawn over the view like a piece of 1970s test equipment. Its legends are engraved
## in capitals: write them in capitals here (the panel shows them exactly as written, it does not
## uppercase them). Numbers, units and readouts on its tubes and dials are not translated.

# Tooltip of the × button in the panel's top-right corner. The code adds the key after it: (B).
live-close-hover = Close the live-render panel
# The legend engraved along the top plate, left of the maker's badge.
live-plate = LIVE RENDER · FRONT PANEL

## Live render panel: instrument titles
## Engraved above each instrument, and repeated as the title of its help card.

# Above the depth readout, whose tubes show the magnification as a power of two ("2^" stays).
live-title-depth = DEPTH 2^
live-title-iterations = ITERATIONS
# Above the frame-time readout (milliseconds).
live-title-frame-ms = FRAME MS
# Above the date-and-time readout.
live-title-clock = DATE · TIME
# The walk dial: a radar screen showing a slow refresh rendered pass after pass, band after band.
live-title-walk = WALK · BANDS
# A meter of how far the picture on screen lags the view.
live-title-age = PICTURE AGE
# What part of the picture this frame's GPU pass covered.
live-title-tiles = TILES
# A scope of the GPU's work per frame, in milliseconds.
live-title-scope = GPU PASSES · MS
# The depth dial: depth, the reference orbits being built, and the arithmetic in use.
live-title-dial = DEPTH · REFERENCES · ARITHMETIC
# Iteration steps per second.
live-title-steps = STEPS / S
# The two GPUs (the window's and a second, worker GPU).
live-title-engines = TWO ENGINES
live-title-memory = MEMORY
live-title-cores = CORES · THREADS
live-title-pixels = PIXEL STREAM
# Help card titles only: the three GPU timer lamps (T0, T1, T2 stay as they are) and the row of lamps.
live-title-timers = GPU TIMERS T0–T2
live-title-bank = LIGHT BANK

## Live render panel: the light bank's lamps
## Engraved under each lamp, and the title of its help card. A cell is narrow (about eight capital
## letters): a longer label breaks at a space onto two lines, and is drawn smaller if it still does
## not fit.

live-lamp-plan = PLAN
live-lamp-dispatch = DISPATCH
live-lamp-live = LIVE
live-lamp-held = HELD
# The last picture reprojected (stretched) to fit the moving view.
live-lamp-reproj = REPROJ
live-lamp-reading = READING
# A pass of a pinned walk.
live-lamp-pin-pass = PIN PASS
live-lamp-split = SPLIT
live-lamp-adopt = ADOPT
live-lamp-abandon = ABANDON
# "REF": a reference orbit. Built for the view as it is (LIVE), ahead of it (AHEAD), or for a tour's next hold (HOLD).
live-lamp-ref-live = REF LIVE
live-lamp-ref-ahead = REF AHEAD
live-lamp-ref-hold = REF HOLD
# The series approximation.
live-lamp-series = SERIES
# The second (worker) GPU.
live-lamp-worker = WORKER
live-lamp-tile = TILE
# Supersampling (anti-aliasing); usually kept as SSAA.
live-lamp-ssaa = SSAA
live-lamp-palette = PALETTE
# The arithmetic in use (direct, df32, floatexp).
live-lamp-mode = MODE

## Live render panel: the kinds of GPU pass
## In the GPU scope's legend (MOVING, PIN, WORKER, SETTLE, SSAA). SETTLE and SSAA also name a still
## view's walk on the walk dial.

# A moving frame's pass.
live-kind-moving = MOVING
# A pass of a pinned walk.
live-kind-pin = PIN
# The worker GPU's passes.
live-kind-worker = WORKER
# A still view's settle tile or walk pass.
live-kind-settle = SETTLE
# A supersampling sample.
live-kind-ssaa = SSAA

## Live render panel: legends and readouts on the instruments

# On the walk dial during a split refresh. $done: passes done; $passes: passes in all.
live-walk-split = SPLIT { $done }/{ $passes }
# On the picture-age meter's face, under its scale (0 to 2 octaves).
live-age-scale = OCTAVES BEHIND
# The depth dial's band for direct (plain floating-point) arithmetic. The other bands, DF32 and FLOATEXP, are names and stay.
live-dial-direct = DIRECT
# Under the two GPU gauges: GPU 0 draws the window; GPU 1 is the worker, or there is none.
live-engines-gpu-window = GPU 0 · WINDOW
live-engines-gpu-worker = GPU 1 · WORKER
live-engines-no-worker = NO WORKER
# The share of displayed frames each GPU made. $window and $worker: percentages, 0–100.
live-engines-on-screen = ON SCREEN { $window } : { $worker }
# The same readout before any frame was shown.
live-engines-on-screen-none = ON SCREEN —
# Under the CPU core bars. $count: the machine's cores.
live-cores-count = { $count } CORES
# Under the threads scope. $count: the app's busy threads.
live-cores-threads = THREADS { $count }

## Live render panel: the memory tanks' legend
## What the GPU memory holds, by kind. Very narrow: about ten letters.

# G-buffers (the picture's working textures).
live-memory-gbuf = G-BUF
# The held picture and the supersampling accumulator.
live-memory-hold = HOLD+SSAA
# The reference orbit and its BLA (bilinear approximation) table.
live-memory-orbit = ORBIT+BLA
# The worker GPU's copy of the reference.
live-memory-worker = WORKER REF

## Live render panel: the roles on the threads scope
## Tiny labels on a small scope: at most four letters each.

# The reference builders.
live-cores-role-ref = REF
# The scoring pool.
live-cores-role-pool = POOL
# The thread driving the worker GPU.
live-cores-role-worker = WRK
# Probes: short side renders.
live-cores-role-probe = PRB
# Image and tour exports.
live-cores-role-export = EXP

## Live render panel: the tiles instrument's captions
## The line under the picture's plane, then the tile size (left) and the tiles a frame (right).

# A walk's pass. $lo and $hi: the iteration band it rendered (numbers).
live-tiles-pin-band = PIN PASS · BAND { $lo }–{ $hi }
live-tiles-settle-band = SETTLE PASS · BAND { $lo }–{ $hi }
live-tiles-moving-whole = MOVING FRAME · WHOLE PICTURE
live-tiles-settled-whole = SETTLED FRAME · WHOLE PICTURE
# $pass: this pass; $passes: the passes the refresh is split into.
live-tiles-split-pass = SPLIT REFRESH · PASS { $pass } OF { $passes }
# $grid: the grid's columns × rows, e.g. 12×8.
live-tiles-grid-done = SETTLE GRID { $grid } · DONE
# $grid: the grid's columns × rows; $done and $total: tiles done, and in all.
live-tiles-grid-progress = SETTLE GRID { $grid } · { $done } OF { $total } DONE
# The same, while a tile too costly for one frame goes out in strips.
live-tiles-grid-strips = SETTLE GRID { $grid } · { $done } OF { $total } DONE · IN STRIPS
# $samples: supersampling samples folded so far; $lo and $hi: the iteration band the next one is walking.
live-tiles-ssaa-band = SSAA · { $samples } DONE · BAND { $lo }–{ $hi }
# $sample: the supersampling sample.
live-tiles-ssaa-whole = SSAA SAMPLE { $sample } · WHOLE FRAME
# No GPU pass this frame, and the panel has not seen a frame yet.
live-tiles-idle-starting = STARTING
# No GPU pass this frame: the last complete picture is held on screen.
live-tiles-idle-held = HELD
# No GPU pass this frame: the last picture is stretched to fit the view.
live-tiles-idle-reprojected = REPROJECTED
# No GPU pass this frame: the picture on screen is current.
live-tiles-idle-live = LIVE
# No GPU pass this frame, and nothing to show.
live-tiles-idle-none = IDLE
# The tile size: $res, a whole picture's resolution (2400×1300).
live-tiles-tile-res = TILE { $res }
# The tile size: $px, a tile's side in pixels.
live-tiles-tile-px = TILE { $px } PX
live-tiles-no-pass = NO PASS
# Tiles a frame. $count: how many (0 when none).
live-tiles-rate-count = { $count }/FRAME
live-tiles-rate-one = 1 TILE/FRAME
# $share: the part of one tile sent each frame, e.g. 0.33.
live-tiles-rate-part = { $share } TILE/FRAME
live-tiles-rate-sample = 1 TILE/SAMPLE
# In the plane's corner during a walk: the share of pixels still iterating. $percent: a percentage, e.g. 54 or 1.5.
live-tiles-running = RUNNING { $percent }%

## Live render panel: help cards
## Shown when the pointer rests on an instrument or a lamp. Words in capitals (SETTLE, SSAA, ABANDON,
## HELD, REPROJ, ADOPT, REF AHEAD, ON SCREEN) are the panel's own legends: use the same words as
## their messages above.

# The heading of a card's list of what healthy looks like.
live-card-healthy = HEALTHY
# The heading of a card's list of what signals trouble.
live-card-trouble = TROUBLE

live-card-walk-what = A deep refresh too costly for one frame is pinned and walked: rendered pass after pass, each letting pixels iterate an octave further (256, 512, 1,024 … up to the ask). The rim is that count on a log scale, starting at the top; the beam is the walk’s progress; each arc is one pass, coloured by its GPU time against the 20 ms target. The white blip waits for the last pass’s reading; the flash is the new picture going on screen.
live-card-walk-note = A still view too costly for one pass is walked the same way, marked SETTLE, and each supersampling sample walks again, marked SSAA: their passes may each take up to 400 ms, and the arcs are coloured against that. A split refresh draws one ring per pass instead: every band, a share of the tiles.
live-card-walk-good-sweep = A sweep that goes round in a handful of frames and ends in a flash.
live-card-walk-good-arcs = Arcs mostly green and amber.
live-card-walk-good-still = On a deep still view: a slower sweep per sample, its arcs green to amber.
live-card-walk-bad-red = Red arcs: passes over budget, felt as stutter.
live-card-walk-bad-restart = A sweep that stops part-way and starts again: the walk was abandoned, and the picture on screen stays old.
live-card-walk-bad-hollow = Hollow arcs: passes whose timing never came back, so the next pass is sized blind.

live-card-age-what = How far the picture on screen lags the view, in octaves of zoom (one octave is 2×). While a refresh renders, the old picture is shown stretched and ages; when the refresh lands, the age drops. The strip below is the last few seconds.
live-card-age-good-sawtooth = A sawtooth between about 0.3 and 0.8, the needle swinging round the 0.5 mark.
live-card-age-good-still = Close to 0 once you stop.
live-card-age-bad-red = The needle living past 1.0, in the red: the picture is more than 2× out of date and looks soft or blocky while zooming.
live-card-age-bad-ramp = A ramp that never drops: refreshes are not finishing (watch ABANDON).

live-card-tiles-what = How this frame’s GPU pass covers the picture. Moving frames and walk passes cover it whole, and the caption names a walk pass’s iteration band. A walk’s pass iterates only the pixels that have not escaped yet: the plane fills to that share (log scale, 0.01% at the floor) and reads it in the corner, so it drains as the walk goes deeper. A refresh a little too costly for one frame is split into up to 8 passes, each an interleaved set of 16-px tiles. A settled view too costly for one pass becomes a grid of tiles, one a frame from the centre out, and a tile still too costly goes out in strips. Read out below: tile size and tiles per frame. The bar under the plane is the range of iterations the pass covers, on a log scale: a walk lights its band, with what it has already finished below it. A whole frame is drawn as a checker whose squares grow as it renders below the screen’s resolution (the fraction reads in the corner), and while supersampling a magnified pixel shows where each sample landed.
live-card-tiles-good-whole = Whole passes while you zoom; split refreshes of a few passes.
live-card-tiles-good-grid = When you stop: a grid of tens of tiles filled in a second or two, then SSAA.
live-card-tiles-bad-small = A grid of hundreds of small tiles: each pass’s budget is small, and sharpening will take a long time.
live-card-tiles-bad-strips = Many tiles going out as strips: tiles ran slower than the grid was sized for.
live-card-tiles-bad-splits = Splits stuck at 8 passes: the refresh barely fits, and a little deeper it falls back to the slower walk.
live-card-tiles-bad-dark = Dark for long stretches while moving: no passes at all.

live-card-scope-what = One column per frame, newest on the right: the GPU work it ran, stacked by kind (moving frame green, pin pass green to red by cost, settle tile or still view’s walk pass cream, supersampling sample lavender, the worker GPU’s frames and samples blue), in milliseconds. Dashed lines at 16.7 ms (60 fps), 33 and 50. A hollow mark is a pass whose timing never came back.
live-card-scope-good-under = Columns under the 16.7 line while you move.
live-card-scope-good-pin = Pin passes green to amber.
live-card-scope-good-samples = A still view's supersampling samples may run longer: nothing is moving.
live-card-scope-bad-over = Columns past 33 or 50 ms: visible stutter.
live-card-scope-bad-seconds = Passes of seconds: the GPU’s watchdog may reset the device.
live-card-scope-bad-hollow = Many hollow marks: work is being sized without measurements.

live-card-timers-what = The three GPU timestamp slots. A slot lights when a pass is timed and dims when its reading comes back, two or three frames later. T0 times every frame’s pass; T1 and T2 time extra pin passes.
live-card-timers-good = T0 blinking whenever there is work; T1 and T2 blinking during walks.
live-card-timers-bad-stuck = A lamp stuck lit: its reading is not coming back, and prices go stale.
live-card-timers-bad-dark = All dark while the GPU is busy: nothing is being measured.

live-card-dial-what = The red pointer is the view’s depth: its magnification as a power of two. The bands are the arithmetic the GPU uses there: direct, df32 (two floats per number), and floatexp (a float with a wide exponent) past 2^93. The dots are reference orbits being built: green for the view, amber ahead of it where the zoom is heading, blue for a tour’s next hold. A ring is an install; a red cross is a build dropped because the view overtook it.
live-card-dial-good-ahead = Amber dots landing ahead of the pointer and ringing before it reaches them.
live-card-dial-good-crosses = Few red crosses.
live-card-dial-bad-passing = The pointer passing its last installed reference: frames wait for one, felt as a freeze.
live-card-dial-bad-crosses = Repeated red crosses: the zoom is outrunning the builders, and CPU is wasted.
live-card-dial-bad-crossing = The 2^93 crossing is a known slow spot: the first floatexp frames cost more.

live-card-bank-what = Each lamp lights when its event happens and fades over about 150 ms, so a busy renderer flickers and an idle one goes dark. Left to right: the frame, the refresh, the references, then the worker, settling, palette and arithmetic.
live-card-bank-note = Hover a lamp for what it means.

live-card-depth-what = The magnification as a power of two: 2^100 is about 1.3 × 10^30.
live-card-depth-note = Past 2^93 every step costs more (floatexp), and references take longer to build.

live-card-iter-what = The iteration ask: how far each pixel may iterate before it counts as inside the set. It climbs with depth.
live-card-iter-good = Climbing steadily as you dive.
live-card-iter-bad-low = Stuck low while deep: spirals and minibrots render as flat interior.
live-card-iter-bad-high = Very high: every refresh costs more, and walks take longer.

live-card-ms-what = The time between displayed frames.
live-card-ms-good = Your display’s interval: 16.7 on a 60 Hz display.
live-card-ms-bad-stutter = Above 33: visible stutter.
live-card-ms-bad-spikes = Spikes above 50: hitches, usually one costly pass (see GPU passes).

live-card-clock-what = The wall clock in local time, to a tenth of a second.
live-card-clock-note = It stamps any screenshot of the panel with when it was taken, so a picture of trouble can be matched to the app’s log.

live-card-tach-what = Iteration steps per second. White: the steps asked for (pixels × iterations). Amber: the steps the GPU actually executed. The gap is BLA (bilinear approximation) jumping many iterations at once; ×N is that multiplier. The odometer totals the session.
live-card-tach-good = A wide gap at depth: BLA doing its job, ×10 to ×1,000.
live-card-tach-bad-together = Needles together at depth: BLA cannot skip here (often near a minibrot’s interior), so refreshes are slow.
live-card-tach-bad-red = The white needle in the red: asking more than the GPU can do in a frame.

live-card-engines-what = How busy each GPU is, with dots carrying its frames to the screen: its passes’ GPU time over the last second, the worker’s motion frames and supersampling samples included. ON SCREEN is the share of displayed frames each one made. With one GPU, the worker side stays idle.
live-card-engines-good = Both busy during a dive, the worker supplying a share of the frames.
live-card-engines-bad-pinned = GPU 0, the window’s, pinned near 100%: it is the bottleneck, and frames stutter.
live-card-engines-bad-idle = A worker enabled but idle: it is not delivering frames.

live-card-memory-what = RAM: the app’s working set, the red line its peak. VRAM: what the app holds on the GPU, reckoned from the sizes it allocates, by kind (G-buffers, hold and SSAA textures, orbit and BLA buffers, the worker’s copy of the reference), against the card.
live-card-memory-good = Flat while zooming; steps up and back down as references come and go.
live-card-memory-bad-leak = RAM climbing and never coming down: a leak.
live-card-memory-bad-vram = VRAM near the card’s size: allocations can fail, or the device can be lost.

live-card-cores-what = CPU use per core, green to amber to red, and a scope where each busy thread orbits its role: the reference builders, the scoring pool, the worker, side renders.
live-card-cores-good = One or two cores busy while zooming; a short burst across every core when a reference is picked.
live-card-cores-bad-every = Every core busy for seconds: reference building at extreme depth.
live-card-cores-bad-own = The app’s own core pinned in the red: the window itself stutters.

live-card-pixels-what = Pixels computed per second, in Mpx/s. Dim: still iterating in a walk. Bright: finished.
live-card-pixels-good = Tens of Mpx/s while moving.
live-card-pixels-bad = Near zero while the picture is stale: the GPU is waiting on something, a reference or a reading.

## Live render panel: the lamps' help cards

live-card-lamp-plan-what = A frame was planned: resolution, iteration ask and budget chosen, and whether to refresh.
live-card-lamp-plan-good = Flickers whenever the view or settings change.
live-card-lamp-plan-bad = Dark while you zoom: the UI thread is stuck.

live-card-lamp-dispatch-what = Work went to the GPU this frame.
live-card-lamp-dispatch-good = Lit while anything changes; dark when idle.
live-card-lamp-dispatch-bad = Busy with nothing changing: the app is not settling.

live-card-lamp-live-what = The picture on screen is this frame’s own render.
live-card-lamp-live-note = Seldom lit during a deep dive, which is normal: see HELD and REPROJ.
live-card-lamp-live-good = Lit at shallow depths, where a frame is cheap.

# "Prefer detail" is a setting's name.
live-card-lamp-held-what = The last complete picture is on screen while the next one composes underneath it: a deep zoom's refresh, or, with Prefer detail, a still view's next tile or supersampling sample.
live-card-lamp-held-good-dive = Lit in stretches during deep zooms, broken by ADOPT.
live-card-lamp-held-good-still = Lit steadily while a still view sharpens, with SSAA blinking.
live-card-lamp-held-bad = Lit for seconds while zooming with no ADOPT: refreshes are not finishing.

live-card-lamp-reproj-what = The last picture, stretched to fit the moving view until a new one is ready.
live-card-lamp-reproj-good = Short bursts while you move.
live-card-lamp-reproj-bad = Lit for long: the picture gets softer the further it is stretched.

live-card-lamp-reading-what = A reading came back from the GPU: a pass’s timing, escape range or content.
live-card-lamp-reading-good = A steady flicker while rendering.
live-card-lamp-reading-bad = Dark for long while rendering: timings are not returning, and budgets are guessing.

live-card-lamp-pin-pass-what = A pass of a pinned walk: the picture rendered one iteration band further.
live-card-lamp-pin-pass-good = Runs of passes that end in ADOPT.
live-card-lamp-pin-pass-bad = Runs that end in ABANDON.

live-card-lamp-split-what = A pass of a split refresh: a share of the tiles, rendered at the full iteration count.
live-card-lamp-split-good = Short runs that end in ADOPT.
live-card-lamp-split-bad = Runs of 8, the most allowed: the refresh barely fits.

live-card-lamp-adopt-what = A finished refresh replaced the picture on screen.
live-card-lamp-adopt-good = Several times a second while zooming.
# "Picture age" names the instrument (its title, PICTURE AGE).
live-card-lamp-adopt-bad = Rare while zooming: the picture ages (see Picture age).

live-card-lamp-abandon-what = A refresh in progress was thrown away: the view moved too far, it grew too old, or a newer one replaced it. A reference build overtaken by the view lights it dimly.
live-card-lamp-abandon-good = Rare, or once right after a jump.
live-card-lamp-abandon-bad = Frequent: work wasted, and the picture stays old.

live-card-lamp-ref-live-what = A reference orbit is being built for the view as it is: it has none, has left the one it had, or needs a longer one.
live-card-lamp-ref-live-good = Brief blinks.
live-card-lamp-ref-live-bad-long = Long stretches: deep views wait for it.
live-card-lamp-ref-live-bad-lookahead = Lit often during a dive while REF AHEAD is dark: the lookahead is not keeping up.

live-card-lamp-ref-ahead-what = A lookahead reference is being built deeper than the view, where the zoom is heading.
live-card-lamp-ref-ahead-good = Regular blinks during a dive.
live-card-lamp-ref-ahead-bad = Dark during a deep dive: the view will outrun its references.

live-card-lamp-ref-hold-what = A tour’s hold prefetch: the reference for the next held keyframe, built during the glide toward it so playback never waits.
live-card-lamp-ref-hold-note = Dark outside tour playback.
live-card-lamp-ref-hold-good = A blink ahead of each hold in a tour.
live-card-lamp-ref-hold-bad = Lit through a whole hold: the prefetch is late, and the tour waits for it.

live-card-lamp-series-what = The series approximation for a new reference, which lets every pixel skip its first iterations.
live-card-lamp-series-good = Brief.
live-card-lamp-series-bad = Long at depth: it delays the reference’s install.

live-card-lamp-worker-what = The second GPU delivered a frame.
live-card-lamp-worker-good = Steady blinks when a worker GPU is enabled.
live-card-lamp-worker-bad = Dark with a worker enabled: it is not delivering.

live-card-lamp-tile-what = A settle tile, or a strip of one, went to the GPU.
live-card-lamp-tile-good = A burst after you stop, then quiet.
live-card-lamp-tile-bad = Never stopping: the grid keeps restarting, because something keeps changing the view or settings.

live-card-lamp-ssaa-what = A supersampling sample was added to the settled picture.
live-card-lamp-ssaa-good = A run of blinks after a settle, then dark.
live-card-lamp-ssaa-bad = Restarting over and over: the settle keeps being disturbed.

live-card-lamp-palette-what = The colour normalization took in a new escape range, so the colours follow the zoom.
live-card-lamp-palette-good = Flickers while zooming.
live-card-lamp-palette-bad = Dark while zooming: the palette range is frozen, and colours go flat and low in contrast.

live-card-lamp-mode-what = The arithmetic changed: direct, df32 or floatexp.
live-card-lamp-mode-good = One blink at a boundary such as 2^93.
live-card-lamp-mode-bad = Blinking back and forth: the view is hunting across a boundary.

## The Render client window (File ▸ Render client…): lending this machine's GPU to a render farm

farm-client-title = Render client
farm-client-intro = Render frames for a render farm. The controller — Tools ▸ Render on farm… on another machine — shows its address and farm key. This machine opens no port: it connects to the controller.
# A field label, and the label of the status row that names the controller.
farm-client-controller = Controller
farm-client-find = Find
farm-client-find-hover = Look for controllers on this network (a controller answers while its farm is listening)
farm-client-key = Farm key
# A checkbox beside the masked farm key: show it in clear.
farm-client-show-key = Show
# A field label (the name this machine goes by), and the label of the status row about this machine.
farm-client-machine = This machine
farm-client-machine-hover = The name the controller shows for this machine.
farm-client-share = Shared drive
# Greyed hint inside the empty field; the path is an example.
farm-client-share-hint = optional — e.g. \\fileserver\share
farm-client-share-hover = This machine's path to the shared drive the controller uses (share mode): frames are written there instead of sent over the connection. Leave empty to send every frame.
farm-client-gpu = Graphics card
# The label of the row with the idle option.
farm-client-when = When
# A checkbox followed by a number of minutes ("Only when idle for [5] min").
farm-client-when-idle = Only when idle for
farm-client-when-idle-hover = Take work only once nobody has touched this machine's keyboard or mouse for this long, or while its screen is locked. When someone starts using it, the frame in progress stops at once and goes back to the farm.
# A collapsible section: the largest job this machine accepts.
farm-client-limits = Limits
farm-client-limits-note = A job asking for more than these is refused, never cut down.
farm-client-limit-size = Largest frame
farm-client-limit-ss = Most supersampling
farm-client-limit-iter = Most iterations

## Render client: the graphics-card choice

farm-client-gpu-system = The system's choice
farm-client-gpu-every = Every graphics card
# $count: this machine's graphics cards.
farm-client-gpu-every-n = Every graphics card ({ $count })
# $count: this machine's graphics cards.
farm-client-gpu-every-each = Every graphics card ({ $count }) — one session each
# A card the window cannot name. $number: its number (or numbers) as --list-adapters gives it.
farm-client-gpu-adapter = Adapter { $number }
farm-client-gpu-finding = Finding this machine's graphics cards…
# "NAME · GPU 1" is how such a machine is named in the controller's list: keep it as it is.
farm-client-gpu-hover = Which graphics card renders the farm's frames. With every card, each joins the farm as its own machine — "NAME · GPU 1", "NAME · GPU 2" — and Pause and Disconnect act on all of them.

## Render client: looking for controllers on the network (the Find button)

farm-client-finding = Looking for controllers on this network…
# $port: the UDP port number.
farm-client-found-none = No controller answered. One answers while its farm is listening; its firewall must let UDP { $port } in.
farm-client-found-pick = On this network — click one to use it:
# $identity: the controller's identity fingerprint.
farm-client-found-hover = Identity { $identity } — use this controller (its farm key is still needed)
# After a warning sign. $version and $git: this app's version and build.
farm-client-found-other-build = A different build from this one ({ $version } { $git }): it would refuse this machine.

## Render client: what is wrong with what was typed, or with starting the client

farm-client-bad-address = Enter the controller's address as HOST:PORT, as its window shows it (for example 192.168.1.20:46733).
# $why: what is wrong with the key (in English).
farm-client-bad-key = The farm key: { $why }
# $why: what is wrong with the name (in English).
farm-client-bad-name = The name: { $why }
farm-client-no-config = There is no configuration folder to keep the farm key in.
# $why: the system's reason.
farm-client-start-failed = Could not start the render client: { $why }

## Render client: the state, in the status heading

farm-client-phase-connecting = Connecting…
farm-client-phase-checking = Running the self-check…
farm-client-phase-idle = Connected — waiting for work
farm-client-phase-rendering = Rendering
farm-client-phase-in-use = Waiting — someone is using this machine
farm-client-phase-paused = Paused
farm-client-phase-retrying = Controller unreachable — retrying
# $seconds: the wait before the next try.
farm-client-phase-retrying-in = Controller unreachable — retrying in { $seconds } s
farm-client-phase-disconnected = Disconnected
farm-client-phase-ended = Ended

## Render client: the line under the status heading

# $address: the controller's address (HOST:PORT).
farm-client-detail-connecting = Connecting to { $address }…
# $controller: the controller's name.
farm-client-detail-checking = Connected to "{ $controller }" — running the self-check
# $controller: the controller's name.
farm-client-detail-approval = Connected to "{ $controller }" — waiting for its user to approve this machine
# $controller: the controller's name.
farm-client-detail-idle = Connected to "{ $controller }" — idle
# $why: the check that failed and why (in English).
farm-client-detail-check-failed = The self-check failed — { $why }
# $why: the controller's reason (in English).
farm-client-detail-refused = Refused by the controller: { $why }
# $why: the controller's reason (in English).
farm-client-detail-closed = The controller closed the connection: { $why }
# $why: the controller's reason (in English).
farm-client-detail-ended = The controller ended the connection: { $why }
# $job: the job's name.
farm-client-detail-bundle = Job "{ $job }" — receiving its bundle
# $job: the job's name.
farm-client-detail-job-waiting = Job "{ $job }" — waiting for frames to render
# $why: the limit the job is over (in English).
farm-client-detail-job-refused = Refused this job: { $why }
farm-client-detail-job-over = The job is over
# $first and $last: frame numbers.
farm-client-detail-rendering = Rendering frames { $first }–{ $last }
# $time: how long nobody must use the machine ("90 s", "5 min").
farm-client-detail-in-use = Waiting — someone is using this machine; work resumes once nobody has for { $time }
farm-client-detail-paused = Paused — the controller has been told
farm-client-detail-pausing = Pausing — finishing the frame in progress
farm-client-detail-cancelled = Cancelled the frame in progress; paused
# A machine with several graphics cards. $rendering: the cards rendering now; $total: all its cards (two or more).
farm-client-rendering-on-cards = Rendering on { $rendering } of { $total } graphics cards

## Render client: the status rows

# The value of the Controller row. $name: the controller's name; $address: HOST:PORT; $identity: its identity fingerprint.
farm-client-row-controller-value = "{ $name }" · { $address } · identity { $identity }
# The value of the This machine row. $name: this machine's name; $identity: its identity fingerprint.
farm-client-row-machine-value = "{ $name }" · identity { $identity }
farm-client-row-gpu = GPU
# One graphics card's row. $slot: the card's number.
farm-client-row-gpu-n = GPU { $slot }
farm-client-row-job = Job
# The frames handed to this machine.
farm-client-row-run = Run
farm-client-row-sent = Sent
# The Run row, in parts joined with " · ": the run, the frame in progress, its time. $first and $last: frame numbers.
farm-client-run-frames = frames { $first }–{ $last }
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-client-run-frame = frame { $frame } ({ $index } of { $count })
# $time: the time spent on it so far ("2.1 s").
farm-client-run-frame-time = { $time } on this frame
# One graphics card's run. $frame: the frame in progress; $first and $last: the run's frames.
farm-client-session-frame = frame { $frame } of { $first }–{ $last }
farm-client-session-no-card = graphics card not known yet
# The Sent row. $count: frames sent; $time: the mean time per frame ("2.1 s").
farm-client-sent =
    { $count ->
        [one] { $count } frame · { $time } a frame
       *[other] { $count } frames · { $time } a frame
    }
# The Sent row of a machine with several cards. $count: frames sent; $split: each card's share ("812 + 422"); $time: the mean time per frame.
farm-client-sent-split =
    { $count ->
        [one] { $count } frame ({ $split }) · { $time } a frame
       *[other] { $count } frames ({ $split }) · { $time } a frame
    }
# After a warning sign. $note: how the GPUs differ, and both GPUs (in English).
farm-client-gpu-note = This machine has { $note }. Its frames will differ slightly from the controller's, which can show in a held shot.
# A collapsible section: the self-check's results.
farm-client-self-check = Self-check
farm-client-thumb-hover = The last frame this machine sent
farm-client-resume = Resume
farm-client-resume-hover = Take work again
farm-client-pause = Pause
farm-client-pause-hover = Finish the frame in progress, then take no more work. The controller hands this machine's frames to the others.
farm-client-cancel-frame = Cancel frame
farm-client-cancel-frame-hover = Stop the frame in progress now and pause; the controller renders it elsewhere.
farm-client-live-paused = Live view paused while this machine renders farm frames.
farm-client-close = Close
farm-client-close-active-hover = Close the window; this machine stays in the farm (reopen it from the File menu)
farm-client-close-hover = Close the window
farm-client-disconnect = Disconnect
farm-client-disconnect-hover = Leave the farm: the frame in progress is handed back
farm-client-connect = Connect
farm-client-connect-hover = Connect to the controller and render frames for it

## The Render on farm window (Tools ▸ Render on farm…): the controller of a render farm

farm-ctl-title = Render on farm
farm-ctl-out = Output folder
farm-ctl-browse = Browse…
farm-ctl-out-note = Frames, and the job's state in farm/: run the same job again to resume it.
farm-ctl-size = Size
farm-ctl-size-custom = Custom…
# The size list's choice when the size is typed in. $width and $height: pixels.
farm-ctl-size-custom-value = Custom — { $width }×{ $height }
farm-ctl-fps = Frames / second
farm-ctl-ss = Supersampling
farm-ctl-port = Listen on port
farm-ctl-port-hover = Clients connect to this port. Allow it through the firewall for your local network.
farm-ctl-share = Shared drive
# Greyed hint inside the empty field.
farm-ctl-share-hint = optional — this machine's path to it
farm-ctl-share-hover = Share mode: clients that have the same drive write their frames to it instead of sending them over the connection, and this machine checks every one there. Clients without it send theirs.
# A label followed by a number of clients and two checkboxes.
farm-ctl-start-with = Start with
# The unit after the number in the box ("Start with [3] clients"). $count: that number.
farm-ctl-start-clients =
    { $count ->
        [one] client
       *[other] clients
    }
farm-ctl-start-clients-hover = The job starts once this many machines have connected and passed their self-check; others can join later.
farm-ctl-local = Also render on this machine
farm-ctl-local-hover = This machine counts as one client.
farm-ctl-sharing = Share references
farm-ctl-sharing-hover = A deep frame's reference orbit can take minutes to hours to build. Shared, it is built once and passed to the other machines. A shared reference is a different, equally valid one: at extreme depth a handful of pixels can differ from a frame that built its own.
farm-ctl-key = Farm key
# A checkbox beside the shortened farm key: show all of it.
farm-ctl-show-key = Show
farm-ctl-copy-key = Copy key
farm-ctl-copy-key-hover = Paste it into each client's Render client window
# In place of the farm key, when there is none.
farm-ctl-no-key = none yet
farm-ctl-make-key = Make a key
farm-ctl-make-key-hover = Clients need it to join; it is also made when you press Render
# What to do on the other machines. $address: this controller's HOST:PORT.
farm-ctl-steps = On each client: File ▸ Render client…, controller { $address }, and the farm key.
# The same when this machine's address is not known. $port: the port number.
farm-ctl-steps-port = On each client: File ▸ Render client…, this machine's address and port { $port }, and the farm key.
# The size of the job. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
farm-ctl-summary =
    { $count ->
        [one] { $count } frame · { $seconds }s of tour · { $width }×{ $height } ss{ $ss }
       *[other] { $count } frames · { $seconds }s of tour · { $width }×{ $height } ss{ $ss }
    }
farm-ctl-strip-hover = Every frame of the job, left to right: done, being rendered, still to do, given up.
farm-ctl-no-tour = No tour is loaded.
# $folder: the folder; $why: the system's reason.
farm-ctl-out-error = The output folder { $folder }: { $why }
# $why: the system's reason.
farm-ctl-start-failed = Could not start the farm: { $why }
# $code: the controller process's exit code, as the code gives it.
farm-ctl-ended-exit = The controller ended (exit { $code }).

## Render on farm: the state, in the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-phase-waiting = Waiting for render clients ({ $connected } of { $needed } connected)
farm-ctl-phase-preparing = Preparing the job
farm-ctl-phase-rendering = Rendering
farm-ctl-phase-paused = Paused
farm-ctl-phase-complete = Complete
farm-ctl-phase-stopped = Stopped
farm-ctl-phase-ended = Ended

## Render on farm: the words after the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-detail-waiting = Waiting for render clients: { $connected } of { $needed } connected
farm-ctl-detail-preparing = Measuring the normalize anchors on this machine, once for every client
farm-ctl-detail-no-space = Waiting for space in the output folder
farm-ctl-detail-paused = Paused: no new work is handed out
farm-ctl-detail-not-rendered = Finished with frames not rendered — run again on the same folder to resume
farm-ctl-detail-stopped = Stopped — run again on the same folder to resume
# $code: the controller process's exit code.
farm-ctl-detail-ended = Ended (exit { $code })
farm-ctl-detail-finishing = Finishing

## Render on farm: progress

# On the progress bar. $done: frames done; $total: frames in the job.
farm-ctl-progress =
    { $total ->
        [one] { $done } / { $total } frame
       *[other] { $done } / { $total } frames
    }
# Added to the progress bar. $rate: frames per second, with decimals.
farm-ctl-progress-rate = { $rate } frames/s
# Added to the progress bar. $time: the time still to go ("38m 10s").
farm-ctl-progress-eta = about { $time } left
# Facts under the progress bar, joined with " · ". $received and $sent: kilobytes per second.
farm-ctl-io = in { $received } KB/s · out { $sent } KB/s
# $space: free space on the output drive ("412 GB").
farm-ctl-free = { $space } free
# $time: the time since the job started ("38m 10s").
farm-ctl-elapsed = { $time } elapsed
# $count: frames given up on.
farm-ctl-failed = { $count } not rendered
# Reference orbits. $built: built for this job; $cached: found in the cache; $reused: reused between frames; $shared: passed between machines.
farm-ctl-refs = references: { $built } built, { $cached } from the cache, { $reused } reused · { $shared } shared
# After a warning sign.
farm-ctl-storage-low = The output folder is nearly full: no new work is handed out until there is room.
# After a warning sign.
farm-ctl-gpus-differ = GPUs differ in this farm. Their frames differ slightly, which can show in a held shot.
# After a warning sign. $count: the GPU classes in the farm.
farm-ctl-gpus-differ-classes = GPUs differ in this farm ({ $count } GPU classes: different GPUs, drivers or probe renders). Their frames differ slightly, which can show in a held shot.
# $gpu: this machine's GPU, API and driver.
farm-ctl-this-gpu = This machine: { $gpu }
# A machine whose GPU differs. $name: the machine; $note: how its GPU differs (in English); $adapter: its GPU.
farm-ctl-gpu-row = { $name }: { $note } — { $adapter }
# The same with its driver. $driver: the driver's name and version.
farm-ctl-gpu-row-driver = { $name }: { $note } — { $adapter }, driver { $driver }

## Render on farm: the table of machines

# Column headers (short: the table is narrow).
farm-ctl-col-machine = Machine
farm-ctl-col-state = State
farm-ctl-col-gpu = GPU
farm-ctl-col-frames = Frames
farm-ctl-col-per-frame = Per frame
# How the machine delivers its frames (a speed, or "shared drive").
farm-ctl-col-link = Link
# When the controller last heard from the machine.
farm-ctl-col-heard = Heard
# In the machine's tooltip. $count: its frames that failed verification.
farm-ctl-strikes =
    { $count ->
        [one] { $count } frame failed verification
       *[other] { $count } frames failed verification
    }
# In the Link column: it delivers its frames through the shared drive.
farm-ctl-link-share = shared drive
# In the Heard column. $seconds: how long ago.
farm-ctl-heard-ago = { $seconds } s ago
farm-ctl-readmit = Re-admit
farm-ctl-readmit-hover = Let this machine join again when it reconnects
farm-ctl-remove = Remove
farm-ctl-remove-hover = Take this machine out of the job; its frames go to the others
# The GPU column's tooltip.
farm-ctl-probe-same = Its probe render is identical to this machine's.
# $count: the pixels that differ.
farm-ctl-probe-differs =
    { $count ->
        [one] Its probe render differs from this machine's in { $count } pixel: its frames will differ slightly from this machine's.
       *[other] Its probe render differs from this machine's in { $count } pixels: its frames will differ slightly from this machine's.
    }
farm-ctl-probe-unknown = Its probe render has not been compared.
# $driver: the driver's name and version.
farm-ctl-gpu-driver = driver { $driver }
# After a warning sign. $note: how its GPU differs (in English: "a different GPU").
farm-ctl-gpu-note = { $note } from this machine's

## Render on farm: a machine's state, in the State column

# $count: its frames that failed verification.
farm-ctl-state-removed-bad =
    { $count ->
        [one] removed: { $count } bad frame
       *[other] removed: { $count } bad frames
    }
farm-ctl-state-removed = removed by you
farm-ctl-state-removing = removing — collecting its diagnostics
farm-ctl-state-gone = gone
farm-ctl-state-leaving = leaving
farm-ctl-state-checking = checking (self-check)
farm-ctl-state-paused = paused by its user
farm-ctl-state-parked = parked: crashed twice — rejoins when it reconnects
farm-ctl-state-refused = refused this job (its own limits)
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-ctl-state-rendering = rendering { $frame } ({ $index }/{ $count })
farm-ctl-state-starting = starting a run
farm-ctl-state-idle = idle
farm-ctl-state-admitted = admitted — waiting for the job

## Render on farm: the buttons

# A collapsible section: the controller's messages.
farm-ctl-events = Events
farm-ctl-resume = Resume
farm-ctl-pause = Pause
farm-ctl-pause-hover = Hand out no new work; frames in progress finish
farm-ctl-live-paused = Live view paused while this machine renders for the farm.
farm-ctl-close = Close
farm-ctl-close-active-hover = Close the window; the job keeps running (reopen it from the Tools menu)
farm-ctl-close-hover = Close the window
farm-ctl-copy-command = Copy command
farm-ctl-copy-command-hover = Copy the equivalent command line
farm-ctl-stop = Stop
# The Stop button once a stop has been asked for and the job has not ended after 10 seconds.
farm-ctl-stop-now = Stop now
farm-ctl-stop-hover = End the job with the frames it has; running the same job again resumes it
farm-ctl-render = Render
farm-ctl-render-hover = Start listening for clients and render the tour on them

## The Render tour window (Tools ▸ Render tour…)

tour-render-title = Render tour
tour-render-out = Output folder
tour-render-browse = Browse…
tour-render-out-hover = Full path the frames are written to.
tour-render-prefix = Frame prefix
# <prefix> stands for the prefix typed in the field.
tour-render-prefix-hover = Frames are written <prefix>_00000.png
tour-render-size = Size
tour-render-size-custom = Custom…
# The size list's choice when the size is typed in. $width and $height: pixels.
tour-render-size-custom-value = Custom — { $width }×{ $height }
tour-render-fps = Frames / second
tour-render-fps-hover = Sub-1 fps is legitimate: sampling a long dive at 0.25 fps is how the deep regression holds get inspected without rendering thousands of frames.
tour-render-ss = Supersampling
tour-render-chapter = Chapter
tour-render-whole = Whole tour
tour-render-chapter-hover = Rendering one chapter keeps the GLOBAL frame numbering, so its frames drop straight back into a full sequence.
tour-render-mp4 = Assemble mp4
tour-render-mp4-hover = Runs ffmpeg afterwards (must be on PATH). The PNG frames are kept.
tour-render-overwrite = Overwrite existing frames
tour-render-overwrite-hover = Off: the render stops to ask, which it cannot do from here.
tour-render-resume = Resume an interrupted render
tour-render-resume-hover = Keep the frames already in the output folder and render only the missing ones. The newest frame is checked first and discarded if it is incomplete (the frame a render dies on is the one it was writing), stepping back until a good one is found; a folder holding frames at a different size is refused rather than mixed into this sequence.
# $count: this machine's graphics cards.
tour-render-all-gpus = Use all graphics cards ({ $count })
tour-render-all-gpus-hover = Render on every graphics card of this machine at once, through a render farm on this computer (--gpus all). The frames, their names, Resume and the mp4 are as for one card; the whole tour renders, in the farm's own order. Different card models draw a few pixels slightly differently: a held shot and a dissolve stay on one card.
tour-render-order = Render order
tour-render-sequential = Sequential
tour-render-sequential-hover = Frame 0, 1, 2, … in order (the default).
tour-render-progressive = Progressive
tour-render-progressive-hover = Preview the whole tour first, refine to full detail: the keyframes render first, then the largest temporal gaps are bisected until every frame exists. Frames land at their correct indices either way, so Resume and the mp4 work unchanged — you can stop early, eyeball the coarse flip-book, and restart with Resume in either order.
# The cost of the render. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
tour-render-summary =
    { $count ->
        [one] { $count } frame · { $seconds }s of tour · { $width }×{ $height } ss{ $ss }
       *[other] { $count } frames · { $seconds }s of tour · { $width }×{ $height } ss{ $ss }
    }
# The disk space the frames will take (an estimate). $size: a size ("5.3 GB").
tour-render-disk = ≈{ $size } of frames
# $size: the frames' estimated size; $free: the free space on that drive ("6.0 GB").
tour-render-disk-free = ≈{ $size } of frames · { $free } free
# After a warning sign. $disk: the line above ("≈51.0 GB of frames · 6.0 GB free").
tour-render-disk-short = { $disk } — not enough room for the whole sequence
tour-render-separate = Renders the script FROM DISK in a separate process, so unsaved edits aren't included — and a lost GPU device there can't take this session with it.
# On the progress bar. $done: frames done; $total: frames to render.
tour-render-progress =
    { $total ->
        [one] { $done } / { $total } frame
       *[other] { $done } / { $total } frames
    }
# Before the render's first progress line.
tour-render-starting = starting…
tour-render-live-paused = Live view paused — the render has the GPU to itself, so it finishes sooner and measures its own frame costs accurately.
tour-render-cancel = Cancel
tour-render-cancel-hover = Close without rendering
tour-render-copy-command = Copy command
tour-render-copy-command-hover = Copy the equivalent command line to the clipboard
tour-render-stop = Stop render
tour-render-render = Render
tour-render-render-hover = Start rendering the frame sequence
# $folder: the output folder.
tour-render-finished = Render finished → { $folder }
# $why: the render's own reason (in English).
tour-render-failed = Render failed: { $why }
# $last: the render's last progress line (in English), or nothing.
tour-render-failed-or-stopped = Render failed or was stopped. { $last }
# $why: the system's reason.
tour-render-no-exe = Cannot find the executable: { $why }
# $why: the system's reason.
tour-render-start-failed = Could not start the render: { $why }

## The Render zoom video dialog (Tools ▸ Render zoom video…): a dive into the current view as a
## video, rendered from exponential-map strips in a separate process.

# The window's title.
zoom-video-title = Render zoom video
# What the video dives into. $fractal: the fractal's name; $magnification: "2.2e99×"; $octaves: a number.
zoom-video-view = { $fractal } · into the current view at { $magnification } ({ $octaves } octaves)
zoom-video-no-julia = Zoom videos dive into the Mandelbrot set and Multibrot 3–8, not Julia sets yet.
# $fractal: the fractal's name.
zoom-video-no-fractal = Zoom videos dive into the Mandelbrot set and Multibrot 3–8 for now, not { $fractal }.
zoom-video-no-method = This colouring reads statistics a zoom video's strips do not keep (stripe, triangle inequality, orbit trap, decomposition): choose smooth, distance estimate, relief or glow.
zoom-video-too-shallow = Zoom in first: the video dives from its start into the current view.
zoom-video-out = Video file
zoom-video-browse = Browse…
# The file dialog's filter: video files.
zoom-video-filter = Video
zoom-video-size = Size
zoom-video-size-custom = Custom…
zoom-video-size-custom-value = Custom — { $width }×{ $height }
zoom-video-fps = Frames / second
zoom-video-speed = Octaves / second
zoom-video-speed-hover = How fast the video zooms: one octave doubles the magnification. One a second is a steady dive; the video lasts the dive's octaves divided by this.
zoom-video-start = Starts at
zoom-video-start-hover = The magnification the video starts from (2^0 is the whole set). It ends at the current view.
zoom-video-rotate = Turns / octave
zoom-video-rotate-hover = Turn the picture as it dives: turns for every doubling of the magnification (0 for none). Relief lighting stays fixed to the screen.
zoom-video-ss = Anti-aliasing
zoom-video-ss-hover = Samples a pixel along each side. The strips are rendered as much finer, so 2× costs about four times as long.
zoom-video-bla = BLA acceleration
zoom-video-bla-hover = Skip iterations with bilinear approximation (BLA) in strips deeper than 1e4×. Deep strips usually render faster with it, but the picture changes slightly: near a minibrot a few points can come out several iterations off. Off computes every iteration.
zoom-video-normalize = Palette follows the depth (normalize)
zoom-video-normalize-hover = Spread the palette over each moment's escape range, as Normalize deep colors does on screen, changing smoothly through the dive. Deep dives need it: past ~1e50× a fixed palette cycle turns to confetti.
zoom-video-keep = Keep the strips
zoom-video-keep-hover = Also write the strips to this folder, so the video can be coloured again later without rendering (fractadyne --assemble FOLDER). They take a lot of space.
# $frames: the number of frames; $seconds: the video's length; $strips: strips to render, each $width × $rows texels.
zoom-video-summary =
    { $frames ->
        [one] { $frames } frame · { $seconds }s of video · { $strips } strips of { $width }×{ $rows }
       *[other] { $frames } frames · { $seconds }s of video · { $strips } strips of { $width }×{ $rows }
    }
# The graphics memory the strips the frames read take. $size: a number of gigabytes ("1.0").
zoom-video-memory = ≈{ $size } GB of graphics memory for the strips in view
# The disk space the kept strips take (an estimate). $size: a number of gigabytes.
zoom-video-disk = ≈{ $size } GB of strips on disk
# $width: texels around a strip; $limit: the graphics card's largest texture side.
zoom-video-too-wide = A strip would be { $width } texels around, more than this graphics card's { $limit }: lower the size or the anti-aliasing.
# $width × $height: the frame's samples; $limit: the graphics card's largest texture side.
zoom-video-grid-too-big = The frames' samples ({ $width }×{ $height }) are more than this graphics card's { $limit }-texel textures hold: lower the size or the anti-aliasing.
# $need, $have: gigabytes ("17.6", "10.0").
zoom-video-too-much-memory = The strips in view need ≈{ $need } GB of graphics memory, more than this card's { $have } GB can spare: lower the size or the anti-aliasing.
# $have: the card's gigabytes ("10.0").
zoom-video-memory-tight = That is most of this card's { $have } GB, and the picture behind this dialog holds some too: if the render runs out, it stops and says so.
zoom-video-separate = Renders in a separate process, coloured as the view is now (the session is saved first) — a lost GPU device there cannot take this session with it.
# $done, $total: frames.
zoom-video-progress = { $done } / { $total } frames
zoom-video-starting = starting…
zoom-video-live-paused = Live view paused — the video has the GPU to itself.
zoom-video-cancel = Cancel
zoom-video-copy-command = Copy command
zoom-video-copy-command-hover = Copy the equivalent command line to the clipboard
zoom-video-stop = Stop render
zoom-video-render = Render
zoom-video-render-hover = Render the strips and assemble the video
# $file: the video file.
zoom-video-finished = Video finished → { $file }
# $why: the reason, as the render gave it.
zoom-video-failed = Render failed: { $why }
# $last: the render's last progress line.
zoom-video-failed-or-stopped = Render failed or was stopped. { $last }
# $why: the operating system's reason.
zoom-video-no-exe = Cannot find the executable: { $why }
# $why: the operating system's reason.
zoom-video-start-failed = Could not start the render: { $why }

## The Custom formula dialog (Fractal ▸ Custom formula…)
## The formula language itself stays as it is in every language: function names (sin, cosh, conj…),
## operators, variables (z, c, p1), and the formulas. Keep them as written inside these messages.

# The window's title.
formula-dialog-title = Custom formula
# A small line at the top of the window, above the formula.
formula-dialog-intro = The step, in Fractint-style expressions
# The two ways to show and edit the formula (top right of the window).
formula-dialog-textbook = Textbook
formula-dialog-textbook-hover = Show the formula typeset, as a textbook would set it, and edit it so
formula-dialog-text = Text
formula-dialog-text-hover = Edit the formula as text
# The tooltip of the "LaTeX" button (the button's own text stays LaTeX).
formula-dialog-latex-hover = Copy the formula as LaTeX, as it is typeset here
# The drop-down list of example formulas (the collection).
formula-dialog-examples = Examples
# Help under the formula in Textbook mode. The characters / ^ ( are what the user types: keep them.
formula-dialog-help-textbook = Type as in the text: / makes a fraction of the term before it, ^ an exponent, ( parentheses. The arrows move through the formula; Tab goes to the next empty box. A line in red does not read: click it to edit it as text.
# Help under the formula in Text mode. "; starts a comment": the ; is what the user types.
formula-dialog-help-text = Type, or use the keypad — it holds every name the formula language knows. Names complete as you type (Tab). Statements are separated by a new line or a comma; ; starts a comment.
# The syntax check, when the formula reads.
formula-dialog-reads = Reads correctly.
# The syntax check in Textbook mode, when the formula has an empty box to fill in.
formula-dialog-empty-box = Fill the empty box.
# The syntax check in Textbook mode. $line: the line number; $message: what is wrong (from the formula parser, in English).
formula-dialog-error-line = Line { $line }: { $message }
# One function written with another notation's name, and the language's name for it: "ln to log".
# $from: the name as written (ln); $to: the formula language's name (log).
formula-respell-pair = { $from } to { $to }
# A button that rewrites them. $changes: one or more formula-respell-pair, joined with ", ".
formula-respell = Change { $changes }
formula-respell-hover = Write these functions as the formula language names them
# Shown when Apply finds a parameter field that is not a number. $name: the parameter (p1 … p5).
formula-param-not-numbers = { $name } is not a pair of numbers
# The Apply button (also in the Formula library window).
formula-apply = Apply
formula-dialog-apply-hover = Compile the formula and show it
# Shown beside Apply while the formula is being compiled.
formula-dialog-compiling = Compiling the formula for the GPU…
# The greyed hint in an empty name box (also in the Formula library window).
formula-name-hint = name (optional)
# The save button: "Update" when the library already has a formula of that name.
formula-dialog-update-in-library = Update in library
formula-dialog-save-to-library = Save to library
formula-dialog-save-hover = Keep this formula and its parameters in the formula library
formula-dialog-save-disabled-hover = Only a formula that reads correctly can be saved
# Opens the Formula library window.
formula-dialog-library = Library…
formula-dialog-library-hover = The saved formulas: apply, edit, import and export them

## The Custom formula dialog's keypad
## The keys' captions are formula tokens and stay as they are; these are their tooltips.

# The keypad's third tab (the other two are "123" and "f(z)"). A short word.
formula-keypad-tab-names = names
# The tabs' tooltips.
formula-keypad-tab-basic-hint = Numbers, operators and the most used names
formula-keypad-tab-functions-hint = Functions
formula-keypad-tab-names-hint = Every variable, parameter and constant, and statement syntax
# The key z.
formula-key-z = The iterate
# The key c.
formula-key-c = The pixel (the Julia constant in Julia mode)
# The key p1 on the first tab.
formula-key-p1-below = Parameter 1 — set in the fields below
# The keys p1 … p5. $n: the parameter's number.
formula-key-param = Parameter { $n }
formula-key-multiply = Multiply
formula-key-divide = Divide
# The key that types ^2.
formula-key-square-after = Square (after a value: z^2)
# The key that types ^.
formula-key-power = Power: an integer, real or complex exponent
formula-key-square-root = Square root
# The key that types |…|: in this language it is the squared modulus, as in Fractint.
formula-key-squared-modulus = SQUARED modulus |z|² — as in Fractint
formula-key-add = Add
formula-key-subtract = Subtract (or negate)
# The key that types =.
formula-key-assign = Assign: z = …, or a temporary t = …
# The key that starts a new line (first tab).
formula-key-next-statement-comma = Next statement (a comma works too)
# The key "del".
formula-key-backspace = Delete the character before the cursor (or the selection)
# The key e.
formula-key-euler = Euler's number
# The key that types (0.5, 0.5).
formula-key-complex-constant = A complex constant: (real, imaginary)
# The key that types a comma.
formula-key-comma = Separates statements, or a complex constant's parts
formula-key-left = Cursor left
formula-key-right = Cursor right
# The function keys (the f(z) tab), one per function.
formula-key-sin = Sine
formula-key-cos = Cosine
formula-key-tan = Tangent
formula-key-cotan = Cotangent
formula-key-exp = Exponential e^z
formula-key-sinh = Hyperbolic sine
formula-key-cosh = Hyperbolic cosine
formula-key-tanh = Hyperbolic tangent
formula-key-cotanh = Hyperbolic cotangent
formula-key-log = Natural logarithm, ln (principal branch)
formula-key-sqr = Square: sqr(z) = z²
formula-key-sqrt = Square root (principal)
formula-key-recip = Reciprocal 1/z
formula-key-conj = Complex conjugate
formula-key-flip = Swap real and imaginary parts
formula-key-abs = Absolute value of BOTH parts: |x| + i|y|
formula-key-cabs = Modulus |z| (a real number)
formula-key-real = Real part
formula-key-imag = Imaginary part, as a real number
# "fn" is Fractint's name for a function slot: keep it.
formula-key-ident = Identity (Fractint's fn placeholder)
# The key "pixel".
formula-key-pixel = The same as c (Fractint's name)
# The key "z =".
formula-key-set-z = Start a statement setting z
# The key "t =".
formula-key-temporary = A temporary: any new name works (t, w, a2, …)
# The key "; …".
formula-key-comment = A comment, to the end of the line
# The key that starts a new line (names tab).
formula-key-next-statement = Next statement

## The formula's completion list (names offered as you type)

# Beside a name the formula itself assigns.
formula-completion-variable = A variable this formula sets
# Beside a function typed with another notation's name.
# $what: what the function is, in English (from the formula parser: "The natural logarithm");
# $other: the other notation's name (ln); $ours: the formula language's name (log).
formula-completion-other-spelling = { $what } — { $other } is written { $ours } here
# Under a list cut short. $count: the names not shown.
formula-completion-more = … { $count } more
# The last line of the list. Tab, Enter and Esc are key names.
formula-completion-keys = Tab or Enter to complete, Esc to close

## The Custom formula dialog's Textbook mode

# The tooltip of a line that does not read. $why: what is wrong (from the formula parser, in English, or formula-textbook-fractint-sections).
formula-textbook-unread-hover = Does not read: { $why }. Click to edit it as text.
# Why a line with Fractint's sections is shown as text in Textbook mode.
formula-textbook-fractint-sections = Fractint's sections (an init's ':', comparisons, a = b = …) are edited as text

## The Formula library window (Fractal ▸ Formula library…)

# The window's title.
formula-library-title = Formula library
# The window's two tabs. $count: the formulas in each.
formula-library-mine = My formulas ({ $count })
formula-library-mine-hover = The formulas you saved or imported
formula-library-collection = Collection ({ $count })
formula-library-collection-hover = Formulas that come with Fractadyne, each with a view to start from
# A small marker before the name of the formula the view shows now. A short word.
formula-library-showing = showing
# The tooltip of Apply on a formula of the collection.
formula-library-apply-shipped-hover = Show it, at its starting view
# A row's Edit button (after a pencil icon).
formula-library-edit = Edit
formula-library-edit-hover = Open it in the Custom formula dialog, without applying it
# A collection row's button that copies the formula into "My formulas".
formula-library-copy-to-mine = Copy to mine
formula-library-copy-to-mine-hover = Add it to your formulas, to keep or change
formula-library-close = Close
# Saves the formula the view shows, under the name typed beside it.
formula-library-add-current = Add current formula
formula-library-add-current-hover = Save the formula the view shows
# "Fractal ▸ Custom formula…" is the menu path: write it as the menus are named.
formula-library-add-current-disabled-hover = Apply a custom formula first (Fractal ▸ Custom formula…)
formula-library-import = Import…
formula-library-import-hover = Add the formulas in a formula file; nothing here is replaced
formula-library-export-all = Export all…
formula-library-export-all-hover = Write every formula here to one file, to share or keep
# The filter box: its label, and the greyed hint inside it.
formula-library-filter = Filter
formula-library-filter-hint = filter by name or text
# Shown while the user has no saved formulas.
formula-library-empty = No saved formulas yet. Save one from the Custom formula dialog, add the one the view shows, or import a file (ours, or Fractint's .frm).
# The tooltip of Apply on one of the user's formulas: with a view saved, and without one.
formula-library-apply-view-hover = Show this formula, at the view saved with it
formula-library-apply-hover = Show this formula
# The tooltip of a row's save icon.
formula-library-export-one-hover = Export this formula to a file
# A row asks before deleting: the question, then its two buttons. "Delete" is also the delete icon's tooltip.
formula-library-delete-ask = Delete it?
formula-library-delete = Delete
formula-library-keep = Keep
# Under a list cut short. $count: the formulas not listed.
formula-library-more = …and { $count } more: type in the filter to find one.
formula-library-no-match = None match the filter.
# Under a saved formula this version cannot read. $why: what is wrong (from the formula parser, in English).
formula-library-unread = Doesn't read in this version: { $why }

## Formula library notices

# $why: the system's reason, in English.
formula-library-save-failed = Couldn't save the formula library: { $why }
formula-library-nothing-to-save = Nothing to save: the formula is empty.
# $name: the formula's name in the library.
formula-library-updated = Updated "{ $name }" in the formula library.
formula-library-saved = Saved "{ $name }" to the formula library.
# $name: the formula's name; $why: what is wrong.
formula-library-apply-failed = "{ $name }" can't be applied: { $why }
# Copying a formula of the collection into "My formulas". $name: its name; $as_named: the name it was given (the same, or "Name (2)" when the name was taken).
formula-library-copy-exists = "{ $name }" is already in your formulas.
formula-library-copied = Copied "{ $name }" to your formulas as "{ $as_named }".
# $name: the formula's name.
formula-library-deleted = Deleted "{ $name }" from the formula library.
# Why the library could not be saved (follows formula-library-save-failed).
formula-library-no-config-dir = no config directory
# At startup, when the library file cannot be read. $why: the reason, in English; $path: a file's full path.
formula-library-unreadable-kept = The formula library could not be read ({ $why }). It was kept as { $path } and a new, empty library started.
formula-library-unreadable-stuck = The formula library could not be read ({ $why }), nor moved aside: copy { $path } somewhere safe before saving a formula, which replaces it.

## Formula library: Import and Export
## A formula file's contents stay in English; this is what the windows and notices around them say.

# File-type names in the open and save dialogs (the extensions are added by the system).
formula-filter-formula-files = Formula files
formula-filter-fractadyne = Fractadyne formulas
formula-filter-fractint = Fractint formulas
formula-filter-all-files = All files
# Why a file cannot be imported (they follow formula-import-failed).
formula-import-too-large = far too large to be a formula file
formula-import-not-text = not a text file
formula-import-no-formulas = no formulas in it
# $why: the TOML reader's message, in English.
formula-file-not-formulas = not a formula file ({ $why })
# $format: what the file says it is ("fractadyne-gradients").
formula-file-other-format = a "{ $format }" file, not a formula file
# "[[formula]]" is the file's own syntax: keep it.
formula-file-no-formulas = no formulas in it (a formula file lists them as [[formula]] tables)
# $file: the file's name; $why: one of the reasons above, or the system's (in English).
formula-import-failed = Couldn't import "{ $file }": { $why }.
# After an import that added nothing. $file: the file's name.
formula-import-nothing-new = Nothing new in "{ $file }".
# After an import. $count: the formulas added; $file: the file's name.
formula-import-added =
    { $count ->
        [one] Imported { $count } formula from "{ $file }".
       *[other] Imported { $count } formulas from "{ $file }".
    }
# The same two, followed by what else happened. $notes: one or more of the formula-import-note-… messages below, joined with "; ".
formula-import-nothing-new-notes = Nothing new in "{ $file }": { $notes }.
formula-import-added-notes =
    { $count ->
        [one] Imported { $count } formula from "{ $file }": { $notes }.
       *[other] Imported { $count } formulas from "{ $file }": { $notes }.
    }
# Notes (clauses, no final full stop). $count: the formulas already in the library, skipped.
formula-import-note-duplicates =
    { $count ->
        [one] { $count } formula was already in the library
       *[other] { $count } formulas were already in the library
    }
# One formula was renamed because its name was taken. $from: its name in the file; $to: the name it was given ("Name (2)").
formula-import-note-renamed-one = "{ $from }" was taken, so it is "{ $to }"
# $count: the formulas renamed because their names were taken (2 or more).
formula-import-note-renamed = { $count } renamed where the name was taken
# $count: the entries skipped as empty or too long.
formula-import-note-skipped =
    { $count ->
        [one] { $count } entry was empty or too long, skipped
       *[other] { $count } entries were empty or too long, skipped
    }
# A second sentence after importing a Fractint .frm file. $count: the entries that do not read; $why: the commonest reason (from the formula parser, in English).
formula-import-frm-unread =
    { $count ->
        [one] { $count } entry doesn't read in this version (most often: { $why }).
       *[other] { $count } entries don't read in this version (most often: { $why }).
    }
# After exporting one formula. $name: the formula's name; $file: the file's name.
formula-export-one = Exported "{ $name }" to "{ $file }".
# After exporting several. $count: how many; $file: the file's name.
formula-export-many =
    { $count ->
        [one] Exported { $count } formula to "{ $file }".
       *[other] Exported { $count } formulas to "{ $file }".
    }
# $file: the file's name; $why: the system's reason, in English.
formula-export-failed = Couldn't write "{ $file }": { $why }

## The control panel (right-hand side): header and section headings

# The panel's title, beside the button that hides it.
panel-title = Controls
panel-hide-hover = Hide control panel
# The button that brings the hidden panel back (top right of the view).
panel-show-hover = Show control panel
# Section headings.
panel-section-navigate = Navigate
panel-section-coloring = Coloring
panel-section-quality = Quality
panel-section-effects = Effects
panel-section-overlays = Overlays
panel-section-advanced = Advanced
# The last section: reference notes about the current fractal. $name: the fractal's name (Mandelbrot, Burning Ship…).
panel-section-about = About { $name }
panel-section-performance = Performance

## The control panel: Navigate
## Labels before a control sit in a column about 104 px wide (roughly 15 characters): a longer label pushes its control to the right.

panel-zoom-speed = Zoom speed
panel-zoom-speed-hover = Speed of hold-Space continuous zoom (1× ≈ 2× per 1.5 s).
panel-snap-zoom = Snap zoom to features
panel-snap-zoom-hover = While zooming in with the mouse wheel or Space, pull the zoom toward a minibrot (or the real axis) close to the cursor, so it slides under the cursor and the zoom lands on it. The nearer the feature, the stronger the pull. Mandelbrot only, single view.
panel-click-zoom = Click to zoom
panel-click-zoom-hover = When on, a left-click in the view dives in by the factor below (right-click backs out), recentered on the clicked point. Drag still pans; Shift+drag / right-drag still box-zoom. Backspace undoes a click. In the dual view it works on either panel, and pinning the Julia c moves to Ctrl+click while this is on. Hold Shift to raise a magnifier under the cursor showing the view the click would land you in, and click while holding it to take that point; Shift+DRAG still box-zooms.
# The click-to-zoom factor, followed by buttons 2×, 4×, 10×…
panel-click-zoom-factor = Factor
panel-click-zoom-snap = Snap to nearest center
panel-click-zoom-snap-hover = After the jump, solve for the nearest minibrot nucleus and settle exactly onto it, so you land on the feature rather than near it. The zoom is not changed — the click chose that. Mandelbrot only. The solve runs in the background: the view jumps at once and corrects itself when the answer arrives, which at extreme depth can take a few seconds.

## The control panel: Coloring
## Labels before a control sit in a column about 104 px wide (roughly 15 characters).

panel-method = Method
panel-method-hover = How escape data maps to color. Stripe / triangle-inequality / orbit-trap / decomposition reveal orbit structure; distance shades by proximity to the boundary.
panel-stripe-density = Stripe density
panel-stripe-tail = Tail only (deep zoom)
panel-stripe-tail-hover = Average the stripe term over the last N iterations of each orbit (an exponential window) instead of the whole orbit. At extreme depth every orbit shadows the reference for all but its last few hundred iterations, so the full-orbit average is one colour for the whole view; the tail keeps the contrast. Orbits shorter than the window are unaffected. Also --stripe-tail N for --render.
panel-tail-length = Tail length
# The unit after the tail length's number: short for "iterations" (the slider reads "64 it").
panel-tail-length-unit = it
panel-tail-length-hover = Window length in iterations. Short windows (8–32) give the classic bold banding at any depth; longer ones average toward a flat colour again, since the stripe term of a long chaotic orbit averages out.
panel-trap-shape = Trap shape
panel-palette = Palette
# The two colours of the Binary palette, each beside its colour swatch.
panel-in-set = In-set
panel-out-of-set = Out-of-set
# The two colours of the Duotone palette, each beside its colour swatch.
panel-shadow = Shadow
panel-highlight = Highlight
panel-cycle = Cycle
panel-offset = Offset
panel-normalize-hover = At extreme depth, escape counts span hundreds of thousands and a fixed Cycle wraps the palette thousands of times between neighboring pixels — a correct dense field reads as speckle noise. This remaps the palette to the view's measured escape range (Cycle then sets how many palette sweeps span it) WHEN the view actually aliases — a smooth view is left alone, because remapping it would change the picture for nothing. Tick 'Always fit range' below to remap regardless. Smooth method only; matches --normalize.
# A checkbox indented under "Normalize deep colors" (also in the Color menu).
panel-fit-range = Always fit range
panel-fit-range-hover =
    Remap the palette to the view's measured escape range even when the view does not alias — the checkbox above only remaps when a dense field would otherwise read as speckle.

    Use this when you want the palette to span what is on screen rather than to fix noise. It still needs a measurement, so it does nothing for the moment or two before the view settles, and nothing at all on a view with no escaped pixels.
# A checkbox indented under "Always fit range" (also in the Color menu).
panel-log-scale = Log color scale
panel-log-scale-hover =
    Spread the palette by the LOGARITHM of the escape value rather than linearly. Escape counts crowd towards the high end at depth, so a linear map spends most of the palette on a thin shell near the boundary and flattens everything else.

    Applies to the normalized mapping — which is what 'Normalize deep colors' (or --normalize on an export) turns on. A heavily skewed range takes the log mapping anyway, whether or not this is ticked.
# The same checkbox's tooltip while it is greyed out (normalization is off).
panel-log-scale-off-hover =
    Requires normalization: this chooses HOW the palette is mapped to the measured escape range, so with 'Normalize deep colors' off (and no --normalize export) there is no mapping for it to change and it would not alter a single pixel.

    Turn on 'Normalize deep colors' to use it.
# The palette animation: its kind, then its speed.
panel-animate = Animate
panel-anim-speed = Speed
panel-anim-speed-hover = Cycle speed: color-offset cycles/sec, or (Random) gradient changes/sec.
panel-shuffle-gradient = Shuffle gradient

## The control panel: Quality

panel-auto-iter = Auto-scale iterations with zoom
# The iteration-count label while auto-scaling is on (the count is then a base the zoom raises).
panel-iterations-base = Iterations (base)
panel-iterations = Iterations
panel-iterations-hover = Base iteration count. With Auto-scale on, the effective count climbs with zoom depth. While you're moving, the preview caps iterations low (50,000) for responsiveness, then sharpens to the full count when the view settles — so deep edges look smooth during motion and resolve when you stop.
# A warning under the iteration count. $count: the iterations the settled preview renders.
panel-res-limited = At this depth the settled preview renders { $count } iterations at reduced resolution to stay responsive. Export for full-resolution detail.
panel-anti-alias = Anti-alias
# Anti-aliasing off (the other choices are 2×, 3×, 4×, 8×).
panel-anti-alias-off = Off
panel-anti-alias-hover = Supersampling for still images (applied when the view settles). Higher tames the fine exterior 'dust' at the cost of render time.

## The control panel: Effects

panel-relief-lighting = 3D relief lighting
panel-relief-lighting-hover = Shade the surface using the distance-estimate slope — an embossed, lit look. (Holomorphic families: Mandelbrot / Multibrot.)
panel-light-angle = Light angle
panel-relief = Relief
panel-relief-hover = Lower = sharper relief; higher = softer/flatter.
panel-rotate-light = Rotate light
panel-rotate-light-hover = Spin the light direction over time (uses the Speed slider).
panel-distance-glow = Distance glow
panel-distance-glow-hover = Bright distance-estimate contour bands that densify into glowing filaments near the boundary. (Holomorphic families.)
panel-glow = Glow
panel-band-width = Band width
panel-band-width-hover = Spacing of the distance contours (octaves per band).
panel-animate-glow = Animate glow
panel-animate-glow-hover = Flow the glow bands over time (uses the Speed slider).

## The control panel: Overlays

panel-orbits-hover = Draw the iteration path of the point under the cursor.
panel-orbit-anim = Animate (racing dot)
panel-orbit-anim-hover = Send a color-cycling dot racing out along the orbit.
panel-orbit-speed = Orbit speed
panel-minimap-hover = A small home-view overview with a "you are here" marker and the zoom depth. Click it to jump to a region.

## The control panel: Advanced
## Labels before a control sit in a column about 104 px wide; most of these are already wider in English.

# Small headings of the section's two groups.
panel-accelerators = Accelerators
panel-performance-tuning = Performance tuning
panel-bla = BLA acceleration (deep zoom)
panel-bla-hover = Bilinear approximation: skip iterations throughout the orbit at extreme depth (floatexp Mandelbrot, ≥1e28×) — ~5× faster GPU render, identical output (verified by the self-test). On by default; turn off to compare or if you hit an artifact.
panel-series-approx = Series approximation
panel-series-approx-hover = Seed the perturbation from a polynomial to skip early iterations, where BLA isn't active (df32 depths, Multibrot, BLA off). Identical output; turn off to compare.
panel-glitch-correct = Glitch correction (export)
panel-glitch-correct-hover = Multi-reference glitch correction for exported images: detects perturbation glitches and re-renders those pixels against extra references until clean. On by default. Exports up to ~32 MP (non-aux coloring); larger images and the live view use the plain path. Never used for Mandelbrot or Multibrot views (outside Julia mode): checked against an exact calculation, it fixed nothing there and cost up to 5x the export time.
panel-dive-limit = Auto-zoom dive limit
panel-dive-limit-hover = Depth where auto-zoom (A key) stops. Up to ~1e271× it glides smoothly; deeper, it switches to a choppy stepped dive to reach extreme depth quickly.
# Followed by two choices, Speed and Quality.
panel-autopilot-priority = Auto-zoom priority
panel-priority-speed-hover = Zoom at the set speed. The screen refreshes as soon as a frame with detail is ready, at whatever resolution keeps refreshes coming (below the Min motion resolution floor if the zoom outruns the render); between refreshes the last frame follows the zoom and softens.
panel-priority-quality-hover = Every refresh is a fully resolved full-resolution frame, and the zoom slows down as needed so the frame on screen never magnifies far before the next one lands. A slower dive that is crisp at every moment.
# Followed by two choices, Detail and Misiurewicz point.
panel-autopilot-target = Auto-zoom target
panel-target-detail-hover = Dive toward the richest structure on the fractal's edge in view, re-choosing the target as the dive goes.
panel-target-misiurewicz-hover = Find the nearest Misiurewicz point (a spiral centre whose structure repeats forever) and dive exactly into it. The dive first holds still while the point is solved to the dive limit's depth — a moment at ordinary depths, longer past 1e1000×. Mandelbrot set only.
panel-work-budget = Live render budget
panel-work-budget-hover = Detail vs. speed for the live view at deep zoom. Higher renders at fuller resolution (crisper — less of the "soft" upscaled look) but lowers frame-rate, and very high values risk a brief GPU stall on the heaviest frames. Exports are always full resolution regardless of this.
panel-min-motion-res = Min motion resolution
panel-min-motion-res-hover = The lowest resolution the live view may drop to while continuously zooming or panning at deep zoom. Higher stops a fast dive from getting blocky/pixelated (sharper) at the cost of frame-rate — at 100% deep-dive detail refreshes must render at full cost, which can make deep zooming visibly step/stutter. If a deep dive feels jerky, lower this. Default 30%; doesn't affect the settled image or exports.
panel-finish-sound = Sound when a render finishes
panel-finish-sound-hover = Play FRACTINT's completion tune when an export or tour render finishes: three rising notes (1047, 1109, 1175 Hz for 100 ms each), exactly as the DOS original encoded them — synthesized as a band-limited square and high-passed to sound like the little PC-speaker cone that played it.
panel-prefer-detail = Prefer detail while zooming
panel-prefer-detail-hover = While zooming or panning, keep showing the last fully detailed frame — scaled and tracked to follow the motion — instead of re-rendering at reduced quality every frame. The view renders in full when the motion pauses. Off: motion renders live at reduced resolution (smoother, coarser). Shallow views (direct mode) always render live — they are cheap and sharp every frame either way.

## The control panel: About (the current fractal)

# A link to the fractal's reference page, followed by an arrow.
panel-about-reference = Reference

## The control panel: Advanced ▸ Second graphics card

# A label before a drop-down (wider than the panel's label column already).
gpu2-label = Second graphics card
gpu2-off = Off
# Another device on the same graphics card as the window (the default).
gpu2-same-card = Same card
# A card the list does not name. $number: the card's number.
gpu2-card-n = Card { $number }
gpu2-finding = Finding this machine's graphics cards…
gpu2-none = This machine has no other graphics card.
gpu2-hover =
    Another graphics device renders frames of a deep view while it moves, and some of the extra samples that sharpen it after it stops, so the picture keeps up with a zoom and reaches its final quality sooner. The card drawing the window still draws everything.

    Another card adds its own speed. Two different card models draw a few pixels slightly differently; if the second card fails, the view carries on with one.

    "Same card" (the default) opens a second device on this card. It adds no speed but renders without waiting for the screen's frames. It shares the card: a failure that resets the card ends the app, as a failure of the window's own rendering does, and the next start offers to turn it off.

## The view (canvas)

# A notice after Ctrl+click pins the Julia c at a deep zoom.
canvas-julia-pin-quantized = Pinned Julia c — but it is held in double precision, so at this depth the whole panel rounds to a single value and the pin lands on the view centre. Zoom out to aim it.
# A small caption in the top-left corner of the view while the orbit overlay is fitted to the view (its "Normalize (fit to view)" option).
canvas-orbit-normalized = orbit (normalized)
# The minimap's tooltip.
canvas-minimap-hover = Overview / you-are-here. Drag to pan at the current zoom, scroll to zoom in and out, click to jump to that region (home zoom).

## The timestamp overlay (View ▸ Show timestamp)

# Under the large elapsed-time clock. $frame: the frame number (as the log writes it, without grouping).
overlay-timestamp-frame = frame { $frame }

## File ▸ Settings ▸ Reference cache…

cache-title = Reference cache
cache-intro = At extreme depth the slow part of a view is its reference orbit — up to an hour of arbitrary-precision arithmetic. Fractadyne keeps finished orbits here, so returning to a location you have visited, or zooming somewhere near it, takes seconds instead. One orbit is about 4 MB.
cache-off = Off for this run (a task invocation, or --no-orbit-cache). Nothing is read or written until the next ordinary launch.
# Small headings of the window's parts.
cache-location = Location
cache-usage = Usage
cache-limit = Limit
cache-no-config = (no config directory available)
cache-open-folder = Open folder
# The text on the usage bar. $used and $limit: sizes such as "1.2 GB"; $count: the orbits kept.
cache-usage-bar =
    { $count ->
        [one] { $used } of { $limit } — one orbit
       *[other] { $used } of { $limit } — { $count } orbits
    }
cache-limit-hover = When the cache grows past this, the orbits that were CHEAPEST to build are removed first — a deep orbit that took an hour outlives any number of quick ones, however recently those were used.
# Beside the limit. $count: about how many orbits of the live view's length fit in it.
cache-limit-estimate =
    { $count ->
        [one] ≈ one orbit at the live cap
       *[other] ≈ { $count } orbits at the live cap
    }
cache-kept = Kept (most valuable first)
# Column headers of the list of kept orbits.
cache-col-iterations = iterations
cache-col-precision = precision
cache-col-size = size
cache-col-last-used = last used
# The precision column. $bits: the precision in bits.
cache-row-bits = { $bits } bits
# Under the list. $count: the orbits not listed.
cache-rows-more = … and { $count } more
# When an orbit was last used (the "last used" column).
cache-ago-now = just now
# $count: minutes.
cache-ago-minutes = { $count } min ago
# $count: hours.
cache-ago-hours = { $count } h ago
# $count: days.
cache-ago-days = { $count } d ago
# Asked before clearing. $count: the orbits; $size: their total size, such as "1.2 GB".
cache-confirm =
    { $count ->
        [one] Delete one orbit ({ $size })? Rebuilding it costs time, not data.
       *[other] Delete { $count } orbits ({ $size })? Rebuilding them costs time, not data.
    }
cache-delete = Delete
cache-cancel = Cancel
cache-clear = Clear cache…
cache-clear-hover = Deletes every stored orbit. Asks first.
cache-close = Close
# $count: the orbits removed.
cache-cleared =
    { $count ->
        [one] Reference cache cleared — one orbit removed.
       *[other] Reference cache cleared — { $count } orbits removed.
    }
# $why: the system's error message.
cache-clear-failed = Could not clear the reference cache: { $why }

## Help ▸ Diagnostics…

diag-title = Diagnostics
diag-intro = Run a test to check that Fractadyne works correctly on your hardware. Results are saved on your machine and can be attached to an issue report — nothing is sent anywhere on its own.
# $status: which arbitrary-precision library is in use, as the program reports it (in English).
diag-arithmetic = Deep-zoom arithmetic: { $status }
# The three tests' buttons, each followed by a sentence saying what it does.
diag-run-selftest = Run self-test
diag-run-uitest = Run UI test
diag-run-gputest = Run GPU arithmetic check
diag-blurb-selftest = Checks the maths and rendering on your GPU against known-correct results. Takes about 15 seconds.
diag-blurb-uitest = Walks the interface and the live view, capturing screenshots at several zoom depths. Takes a minute or two and opens windows while it runs.
diag-blurb-gputest = Checks the shader's double-float arithmetic against known-correct values on every graphics backend your machine offers. Takes a few seconds, opens no window. If it reports a failure, that is a genuine finding we would like to see — please attach it to a report.
# While a test runs. $count: the checks finished so far.
diag-checks-done =
    { $count ->
        [one] one check done
       *[other] { $count } checks done
    }
# A finished test's result. $test: the test's name (Self-test, UI test, GPU arithmetic check).
diag-verdict-captured = { $test } — result captured
diag-verdict-passed = { $test } — passed
diag-verdict-problems = { $test } — reported problems
# Under the GPU arithmetic check's result.
diag-informational-note = This one has no pass or fail: it records how your GPU computes, and a reported failure is a real finding rather than a fault in your machine. Either way, attaching it to a report is the most useful thing you can do with it.
# Under a test's result when it reported problems.
diag-problems-note = Some differences are expected on hardware other than the reference GPU — image comparisons in particular. Attaching this to an issue report is the most useful thing you can do with it.
diag-open-results = Open results
diag-attach = Attach to an issue report…
diag-attach-hover = Opens the report dialog with this result included, so the issue carries a machine-validated test rather than just a description
diag-console = Print diagnostics to the console
diag-console-hover = Show the [fd-…] lines on stderr from now on. Off by default when Fractadyne is launched without arguments. This changes nothing about what is RECORDED — the log file always gets every line — so leave it off unless you are watching a terminal. Persists for this session only; use the --console flag or FRACTADYNE_CONSOLE=1 to catch startup too.
diag-close = Close
# Errors starting a test. $why: the system's error message; $dir: a folder.
diag-error-no-exe = Cannot find the executable: { $why }
diag-error-create-dir = Cannot create { $dir }: { $why }
diag-error-start = Could not start the test: { $why }

## Navigate ▸ Misiurewicz explorer…

misi-title = Misiurewicz explorer
misi-mandelbrot-only = Misiurewicz browsing is Mandelbrot-only — switch the fractal to use it.
# While the gallery is being built. $seconds: the time so far, such as "3".
misi-solving = Solving the small-type families ({ $seconds }s)…
# Above the gallery. $total: all points; $curated: the hand-picked ones; $found: the ones the sweep found.
misi-summary = { $total } points — { $curated } curated, { $found } found by a seeded Newton sweep of the small types (upper half-plane; a coarse grid finds most basins, not provably all).
# The name of a point the sweep found. $kind: its type, such as "(3,1)"; $point: its coordinate, such as "-0.1011+0.9563i".
misi-entry-name = { $kind } at { $point }
# Under each thumbnail. $kind: the point's type, such as "(3,1)"; $octaves: the zoom period in octaves, such as "3.17"; $twist: the twist per repeat in degrees, such as "+12.5". "oct" is short for octaves.
misi-entry-readout = { $kind } · { $octaves } oct/repeat · { $twist }°
# Before the depth field.
misi-zoom-to = Zoom to
misi-cancel = Cancel
misi-solve-jump = Solve & jump
misi-solve-jump-hover = Newton-solve this exact point to the precision the asked depth needs (from its stored coordinate as the seed), then jump straight there. Deep asks take longer — the solve is arbitrary-precision.
misi-select-hint = Select a point, name a depth, and jump — the multiplier line under each thumbnail is what a dive there will look like: how many octaves before the view repeats, and how far it twists per repeat.
misi-cancelled = Solve cancelled.
# $text: what was typed in the depth field.
misi-bad-zoom = Cannot read "{ $text }" as a magnification (write 1e50, 3.2e120, …).
misi-no-kp = This entry has no detected (k,p) to solve with.
misi-parse-failed = Stored coordinate failed to parse.
# A notice after the jump. $name: the point's name; $zoom: the magnification, such as "1e50"; $seconds: how long the solve took, such as "1.2".
misi-zoomed = Zoomed to { $name } — { $zoom }× ({ $seconds }s)
# Added after the notice above when the iteration count is too low. $have: the fixed iteration count; $typical: what the depth typically needs.
misi-iter-note = Iterations is fixed at { $have }; this depth typically needs ~{ $typical } — raise it (or enable auto-iterations) if the view renders flat.
# $name: the point's name; $why: why the solve failed (a technical word).
misi-solve-missed = The deep solve did not land on { $name }: { $why }. The stored point is still exact at its own depth — try a shallower target.

## Life (Game of Life): the side panel

# Shown when the graphics card cannot run the Life view.
life-no-compute = This graphics adapter cannot run Life (no compute shaders).
# Row label: the pattern list (pattern names stay in English).
life-pattern = Pattern
life-open = Open…
life-open-hover = Open an RLE, .cells or Life 1.05/1.06 pattern file.
life-pattern-text = Pattern text…
life-pattern-text-hover = Paste a pattern (RLE as LifeWiki gives it, or plaintext).
life-save = Save…
life-save-hover = Save the universe as it is now, as RLE.
# Row label: the rule list and the typed rule (B3/S23 notation).
life-rule = Rule
# Shown in the rule list when the rule in use is none of the library's.
life-rule-custom = custom
life-apply = Apply
# Playback buttons (a pause or play symbol is shown before them).
life-pause = Pause
life-play = Play
life-play-hover = Run the universe (Enter).
life-step = Step
life-step-hover = Advance one generation (N).
# Tooltip of the button beside Step, which reads "+16", "+1,024"…: that many generations.
life-stride-hover = Advance this many generations (Shift+N).
life-reset = Reset
life-reset-hover = Back to the pattern as opened (generation 0).
life-speed = Speed
# After the speed slider's number: generations a second. Short.
life-speed-unit = gen/s
# Row label: how many generations the "+n" button advances.
life-stride = Stride
# The panel's numbers.
life-generation = Generation
life-population = Population
life-tiles = Tiles
# $count: the tiles stored; $capacity: how many the GPU pool holds.
life-tiles-of = { $count } of { $capacity }
life-tiles-hover = Stored 64×64-cell tiles (4 KiB each) and the GPU pool's capacity.
# Row label: what a left-drag in the view does (Pan or Draw).
life-mouse = Mouse
life-pan = Pan
life-pan-hover = Drag to move the view.
life-draw = Draw
# The Draw tool's tooltip; the toolbar's shows it after "Draw (D):", so it starts lowercase.
life-draw-hint = click a cell to flip it; drag to paint — a stroke that starts on a dead cell draws, one that starts on a live cell erases. A running universe pauses while you draw and runs on when you let go.
life-undo = Undo
life-undo-hover = Back to the universe before the last edit, fill, clear or rule change (Ctrl+Z while drawing).
life-grid = Grid lines
life-grid-hover = Cell borders when zoomed in to 8 pixels a cell or more.
# A random fill: "Random" labels the density field, "seed" the seed field after it.
life-random = Random
life-density-hover = The fraction of cells a fill sets alive.
life-seed = seed
life-fill-view = Fill view
# $cells: the largest side of the filled area, in cells (2,048).
life-fill-view-hover = Fill the visible area (up to { $cells } × { $cells } cells about its centre) at random.
life-clear = Clear
life-clear-hover = Kill every cell (Undo brings them back).

## Life: notices in the panel, and file dialogs

# $path: the file written.
life-saved = Saved { $path }
# $error: why (from the system, in English).
life-save-failed = Save failed: { $error }
# The universe stopped running. $error: why (in English).
life-stopped = Stopped: { $error }
# A pattern file named a rule that does not read. $error: why (in English); $rule: the rule kept, in B3/S23 notation.
life-rule-not-understood = The pattern's rule was not understood ({ $error }); kept { $rule }
# $path: the file; $error: why (in English).
life-open-failed = Could not open { $path }: { $error }
# File-dialog filter names.
life-filter-patterns = Life patterns
life-filter-rle = Run-length encoded pattern

## Life: the toolbar (its buttons are icons; these are their tooltips)

life-toolbar-pause-hover = Pause the universe (Enter)
life-toolbar-play-hover = Run the universe (Enter)
# $stride: the generations a Shift+click steps.
life-toolbar-step-hover = Step one generation (N); Shift+click or Shift+N steps { $stride }
life-toolbar-reset-hover = Back to the pattern as opened (generation 0)
# $hint: the Draw tool's tooltip (life-draw-hint).
life-toolbar-draw-hover = Draw (D): { $hint }

## Life: the Pattern text window

life-text-title = Pattern text
life-text-intro = Paste a pattern: RLE (as LifeWiki and Golly give it), plaintext .cells, or Life 1.05 / 1.06.
life-text-load = Load
life-text-current = Current as RLE
life-text-current-hover = Fill the box with the universe as last loaded.

## L-systems: the side panel

# Row label: the system list (system names stay in English).
lsys-system = System
lsys-open = Open…
lsys-open-hover = Open a Fractint .l file or a .lsys system.
lsys-edit = Edit…
lsys-edit-hover = The system's text: axiom, productions, angle — edit and apply.
lsys-save = Save…
lsys-save-hover = Save the system as a .lsys text file.
# Tooltip of the "SVG…" button.
lsys-svg-hover = Export the view as an SVG drawing: its lines and shapes as vectors, in its colours.
lsys-follow = Order follows zoom
lsys-follow-hover = The order rises as you zoom in, so the detail never runs out.
lsys-follow-fixed-hover = This system does not grow by a factor each order, so it is drawn at a fixed order.
lsys-order = Order
lsys-angle = Angle
lsys-sweep-hover = Sweep the angle, there and back: the curve morphing as it turns
# A small button that puts back the system's own angle after the slider moved it.
lsys-system-angle = System's angle
lsys-seed = Seed
lsys-seed-hover = What the random choices follow: the same seed, the same plant.
lsys-seed-new = New
lsys-seed-new-hover = Another plant: a new seed.
lsys-line-width = Line width
# Row label of a percentage slider: how much of the curve is drawn.
lsys-draw-on = Draw on
lsys-draw-on-hover = How much of the curve is drawn, in the order the turtle draws it
lsys-draw-on-play-hover = The curve drawing itself on
lsys-colour-by = Colour by
# The colourings, in the "Colour by" list.
lsys-colour-position = position along the curve
lsys-colour-depth = branch depth
lsys-colour-heading = heading
# C, < and > are the system's commands that set the colour index: keep them.
lsys-colour-index = colour index (C, <, >)
lsys-colour-plain = one colour
# The panel's numbers.
lsys-growth = Growth
lsys-growth-built = built as a word, at its order
lsys-growth-built-hover = Parametric or context-sensitive: a module's rewrite depends on its numbers or its neighbours, so the word is built a generation at a time (up to two million modules) and drawn in double precision — the order does not follow the zoom.
# How much the picture grows from one order to the next. $factor: the factor (1.4142).
lsys-growth-factor = ×{ $factor } an order
lsys-growth-factor-period-2 = ×{ $factor } an order (orders step by 2)
lsys-growth-none = none (fixed order)
lsys-drawn = Drawn
# $drawn: what was drawn (lsys-drawn-segments, lsys-drawn-shapes or lsys-drawn-segments-shapes); $order: the order.
lsys-drawn-at-order = { $drawn } at order { $order }
# The same, when the walk stopped at its segment budget.
lsys-drawn-at-order-budget = { $drawn } at order { $order } (budget)
# $segments: the line segments drawn.
lsys-drawn-segments =
    { $segments ->
        [one] { $segments } segment
       *[other] { $segments } segments
    }
# $shapes: the filled shapes drawn.
lsys-drawn-shapes =
    { $shapes ->
        [one] { $shapes } filled shape
       *[other] { $shapes } filled shapes
    }
# Both counts. $segments: line segments; $shapes: filled shapes.
lsys-drawn-segments-shapes =
    { $segments ->
        [one] { $segments } segment
       *[other] { $segments } segments
    }, { $shapes ->
        [one] { $shapes } filled shape
       *[other] { $shapes } filled shapes
    }
# $ms: the walk's time in milliseconds (12.3).
lsys-drawn-hover = walked in { $ms } ms; subtrees off the view are skipped, those under a pixel drawn as one segment
# A word too long to build at the order asked for. $wanted: the order asked for; $built: the order drawn.
lsys-drawn-short = Order { $wanted }'s word is over two million modules: drawn at { $built }.
# $order: the deepest order the app draws.
lsys-deepest-order = At the deepest order drawn ({ $order }): zooming further adds no detail.

## L-systems: notices in the panel and the editor, and file dialogs

# $path: the file; $error: why (in English).
lsys-open-failed = Could not open { $path }: { $error }
# Why a file was not opened (shown after "Could not open …:").
lsys-file-too-large = larger than 4 MiB
# Why a Fractint .l file was not opened: it holds no entries. `Name { … }` is the file's syntax: keep it.
lsys-no-entries = no entries (a .l file holds `Name {"{"} … {"}"}` entries)
# $path: the file written.
lsys-saved = Saved { $path }
# $error: why (from the system, in English).
lsys-save-failed = Save failed: { $error }
# After File ▸ Export SVG. $path: the file written.
lsys-svg-exported = Exported { $path }
# $error: why (from the system, in English).
lsys-svg-failed = Export failed: { $error }
# File-dialog filter names: opening (any system file) and saving (one system).
lsys-filter-systems = L-systems
lsys-filter-system = L-system

## L-systems: the toolbar

lsys-toolbar-lower-hover = Lower the order (fixes it)
lsys-toolbar-raise-hover = Raise the order (fixes it)
# Between the − and + buttons, in a fixed-width font. $order: the order, padded to 3 characters so the label keeps its width.
lsys-toolbar-order = order { $order }
# A toggle: the order follows the zoom. Short.
lsys-toolbar-auto = auto
lsys-toolbar-auto-hover = The order follows the zoom
lsys-toolbar-edit-hover = Edit the system's text

## L-systems: the editor window, and a .l file's entries

lsys-editor-title = L-system
# The editor's help. The line keys (angle, axiom, heading, draw / move / variables, colour, order), the quoted examples ('X = word', 'seed n', 'define R 1.456'…), the commands and "The Algorithmic Beauty of Plants" (a book's title) are the system's syntax and names: keep them as they are.
lsys-editor-help = One key a line: angle (degrees, or /n for a division of the circle), axiom, productions 'X = word', and optionally heading, draw / move / variables, colour, order. A symbol with alternatives chosen at random has one line for each, with its weight: 'X (0.3) = word'; 'seed n' picks the plant. In a word, {"{"} {"}"} fills the turtle's path and . adds a vertex. Parametric and context-sensitive productions (The Algorithmic Beauty of Plants): 'A(s) : s > 1 = F(s)[+A(s/2)]', 'b < a > c = b', with 'define R 1.456' and 'ignore +-F'; F(l) steps l, +(a) turns a degrees. A Fractint .l entry can be pasted as it is.
lsys-editor-apply = Apply
lsys-editor-revert = Revert
lsys-editor-revert-hover = The text of the system shown.
# The window listing a .l file's entries to pick from. $file: the file's name.
lsys-entries-title = { $file } — entries
# Tooltip of an entry that does not parse (its error is shown beside it, in English).
lsys-entry-unreadable-hover = This entry does not read.

## Export: opening a view or location (File ▸ Open view or location…)

# File-dialog filter names.
export-filter-view = Fractadyne view or location
export-filter-image = Image (PNG / EXR)
export-filter-location = Location (.fdn / .kfr)
# $zoom: the magnification (1.2e148), followed by the magnification sign ×.
export-loaded-location = Loaded location @ { $zoom }×
# $zoom: the magnification; $note: what was noteworthy in the file (in English).
export-loaded-location-note = Loaded @ { $zoom }× — { $note }
# $path: the file.
export-not-location = { $path } isn't a Fractadyne location.
export-file-too-large = File too large (not a .fdn location?).
# $path: the file; $error: why (from the system, in English).
export-read-failed = Couldn't read { $path }: { $error }
# A Kalles Fraktaler .kfr file did not import. $error: why (in English).
export-import-failed = Import failed: { $error }
# $path: an image whose embedded text is not a view.
export-not-view = { $path } has Fractadyne metadata, but not a view.
# $path: the image the view was read from.
export-loaded-view = Loaded view from { $path }
# $path: the image; $note: what was noteworthy in it (in English).
export-loaded-view-note = Loaded view from { $path } — { $note }
export-no-metadata = That file has no embedded Fractadyne view metadata.

## Export: notices after an export or a snapshot
## "File → Export image…" in them names the menu items (menu-file, menu-file-export).

# The export window's status while the image renders.
export-rendering = Rendering…
export-canceled = Export canceled.
# $error: why (in English).
export-failed = Export failed: { $error }
# $size: the image's size in pixels (1920×1080); $path: the file written.
export-saved = Saved { $size } → { $path }
# An L-system export whose walk stopped at its budget. $size: pixels; $segments: the budget (4,000,000); $path: the file.
export-saved-incomplete = Saved { $size } (incomplete: past the { $segments }-segment budget — lower the order) → { $path }
# A two-panel export written as two files. $path: the first of them.
export-saved-two = Saved 2 files → { $path }
# $size: pixels; $path: the file written.
export-saved-corrected = Saved { $size } (glitch-corrected) → { $path }
# $path: the first of the two files.
export-saved-two-corrected = Saved 2 files (glitch-corrected) → { $path }
# Added after a finished export's notice. $duration: how long it took (8.3s, 1m 04.0s).
export-elapsed = (in { $duration })
export-preparing-deep = Preparing deep export — building reference (this can take a while)…
# $size: pixels (5120×4035); $ss: the supersampling factor; $iterations: the iteration count.
export-large = Large export ({ $size }, { $ss }× supersampling, { $iterations } iterations) — rendering in the background; glitch correction is skipped for exports this size…
export-life-unavailable = Image export of a Life view is not available yet — use a screenshot, or File ▸ Save Life pattern…
export-already-running = An export is already running — File → Export image… shows its progress.
# $size: the image's size in pixels.
export-snapshot-background = Snapshot: rendering { $size } in the background — File → Export image… shows progress and can cancel it.
export-no-gpu = GPU not available
# A screen-capture snapshot. $size: pixels; $path: the file written.
export-snapshot-saved = Saved screen snapshot { $size } → { $path }
# $error: why (in English).
export-snapshot-failed = Snapshot failed: { $error }

## The colour picker (the popup of a colour swatch)

# The third row's label: the colour as hexadecimal bytes. Short (about 6 characters).
picker-row-hex = Hex
picker-hex-hover = Click to type a hex value
picker-cancel = Cancel
picker-cancel-hover = Put back the colour this picker opened with
picker-ok = OK
picker-ok-hover = Keep this colour
# Starts the screen eyedropper (an icon is shown before it).
picker-pick = Pick
picker-pick-hover = Take a colour from anywhere on screen
# Tooltip of the disabled Pick button (in the picker and the gradient editor) on Linux and macOS.
picker-eyedropper-unsupported = Picking a colour from elsewhere on screen needs a platform screen-capture API. Only the Windows build has one; on Linux this would need an X11 dependency, and Wayland requires a portal with its own permission prompt.

## The fractal families: what each one is (the info panel's "About" section and the pickers' hover text).
## The formulas themselves (z -> z^2 + c) are written in code and stay as they are.

fractal-mandelbrot-about = The canonical escape-time fractal: the set of c for which the orbit of 0 stays bounded. Its boundary is infinitely intricate.
fractal-multibrot3-about = A Multibrot set - the Mandelbrot construction at a higher power. Power d gives (d-1)-fold rotational symmetry.
fractal-multibrot4-about = Multibrot at power 4: threefold symmetry, broad bulbs.
fractal-multibrot5-about = Multibrot at power 5: fourfold symmetry.
fractal-tricorn-about = The Tricorn (Mandelbar): conjugates z each step. This anti-holomorphic map yields a three-cornered shape.
fractal-burning-ship-about = Absolute values of z's parts are taken before squaring; the result resembles a ship in flames.
fractal-celtic-about = A Burning-Ship relative that takes the absolute value of only the real part of z^2, producing celtic-knot / heart motifs.
fractal-buffalo-about = An abs-variant taking absolute values of both components of z^2.
fractal-phoenix-about = The Phoenix uses the previous iterate too, giving flame-like filaments. Try its Julia form via Julia mode.
fractal-newton-about = Newton's root-finding iteration for z^3 = 1, colored by how fast each point converges. A convergence (not escape) fractal.
fractal-multibrot6-about = Multibrot at power 6: fivefold symmetry.
fractal-multibrot7-about = Multibrot at power 7: sixfold symmetry, the bulbs thinning towards a circle.
fractal-multibrot8-about = Multibrot at power 8: sevenfold symmetry.
fractal-burning-ship3-about = The Burning Ship at power 3: the absolute values are taken, then cubed.
fractal-burning-ship4-about = The Burning Ship at power 4.
fractal-burning-ship5-about = The Burning Ship at power 5.
fractal-tricorn3-about = The Tricorn (Mandelbar) at power 3: fourfold symmetry.
fractal-tricorn4-about = The Tricorn at power 4: fivefold symmetry.
fractal-tricorn5-about = The Tricorn at power 5: sixfold symmetry.
fractal-celtic3-about = The Celtic fold at power 3: the absolute value of the real part of z^3.
fractal-celtic4-about = The Celtic fold at power 4.
fractal-celtic5-about = The Celtic fold at power 5.
fractal-buffalo3-about = The Buffalo at power 3: the absolute values of both parts of z^3.
fractal-buffalo4-about = The Buffalo at power 4.
fractal-buffalo5-about = The Buffalo at power 5.
fractal-custom-about = A formula written in Fractint-style expressions (Fractal ▸ Custom formula…). Most deep-zoom by perturbation like the built-in families; the formula dialog says whether yours does.
fractal-life-about = Conway's Game of Life and its relatives on an unbounded plane: Generations rules, non-totalistic (Hensel) rules, pattern files. Play, step, draw.
fractal-lsystem-about = Lindenmayer systems: curves, space-filling curves, islands and plants drawn by a turtle from a rewriting grammar. The order follows the zoom, so the detail never runs out. Fractint .l files open directly.

# Shown where the other families show their formula (z -> z^2 + c).
fractal-custom-formula = z -> the formula you write
# Shown where the other families show their formula. "B3/S23" is the rule's notation: keep it.
fractal-life-formula = B3/S23: born with 3 live neighbours, survives with 2 or 3
# Shown where the other families show their formula. "F = F+F--F+F" is the grammar's notation: keep it.
fractal-lsystem-formula = F = F+F--F+F: each symbol rewritten, then drawn by a turtle

# The last paragraph of the picker's hover text for a family that renders only on the direct path.
fractal-no-deep-zoom = Direct rendering only - this family has no deep zoom.

## Names of settings, shown in the Settings menu and the welcome window.

theme-dark = Dark
theme-light = Light
# The update track that follows only stable releases.
update-track-stable = Stable
# The update track that also offers pre-releases (betas).
update-track-beta = Beta (pre-releases)

## The status bar (its hover texts; the readouts follow in their own section)

status-period-hover = Period of the minibrot you jumped to (press M, or Go to location ▸ Go to feature). It stays while the view sits on the feature and clears when you move away.

## The playback transport: the player over the view while a tour plays.

# $close: the close button's icon, as drawn on that button.
play-finished-hover = Finished — scrub back in, or { $close } to close
# $kind: "benchmark" or "script" (the variants hold the wording); $name: the tour's name.
play-playing-hover =
    { $kind ->
        [benchmark] Playing benchmark "{ $name }"
       *[other] Playing script "{ $name }"
    }
play-restart = Restart from the beginning
play-back = Back 10 seconds
play-again = Play again from the beginning
play-resume = Resume
play-pause = Pause
play-stop = Stop and rewind to the start
play-forward = Forward 10 seconds
play-speed-hover = Playback speed (click to cycle 0.5× / 1× / 2× / 4×)
play-render = Render this script to a frame sequence…
play-render-none = The built-in benchmark has no script file to render
play-open = Play a different script…
play-loop = Repeat the tour when it reaches the end
play-close = Close the player and restore your own view settings
# A notice beside the scrub bar while the renderer holds the tour's clock. Keep it short.
play-waiting = waiting for detail
# "[playback] pace" is a setting in the script file: keep it as written.
play-waiting-hover = The tour clock is paused while the renderer resolves this view (reference build / iteration budget climbing). Playback resumes by itself; see [playback] pace in the script.
play-scrub = Drag to scrub through the tour
# $behind: how far the script clock is behind real time, as minutes:seconds (1:05).
play-scrub-behind =
    Drag to scrub through the tour
    Script clock is { $behind } behind real time (the pacer slowed the tour to let the renderer keep up). The upper tick marks where wall-clock playback would be.

## The autopilot's notices (toasts). Where a notice has two lines, the first is the verdict and the second what to do about it.

autopilot-on-detail = Autopilot on — diving toward detail (any input stops)
autopilot-off = Autopilot off
autopilot-misi-needs-mandelbrot =
    Autopilot: a Misiurewicz target needs the Mandelbrot set
    Diving toward detail instead
autopilot-on-misi =
    Autopilot on — finding the nearest Misiurewicz point…
    Any input stops
# $seconds: how long the search has run, in whole seconds.
autopilot-misi-finding =
    Autopilot: finding the nearest Misiurewicz point…
    { $seconds } s · any input stops
autopilot-misi-not-found =
    Autopilot: no Misiurewicz point found near this view
    The finder may not reach this depth — try the Detail target
# ($preperiod,$period) names the point, as mathematicians write it: keep the notation. $widths: a power of two (2^$widths screen widths).
autopilot-misi-off-screen =
    Autopilot: the nearest Misiurewicz point ({ $preperiod },{ $period }) is off screen
    ~2^{ $widths } screen widths away — move closer to a spiral centre
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-not-solved =
    Autopilot: the Misiurewicz point ({ $preperiod },{ $period }) near this view did not solve
    Try the Detail target
autopilot-misi-refused = Autopilot: Misiurewicz solve refused (bad pair)
autopilot-misi-stopped = Autopilot: the Misiurewicz solver stopped unexpectedly
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-diving =
    Autopilot: diving into Misiurewicz ({ $preperiod },{ $period })
    Any input stops
# ($preperiod,$period) names the point: keep the notation. $octaves: how often the structure repeats (2.37); $twist: its turn in degrees, signed (+12.5).
autopilot-misi-diving-repeat =
    Autopilot: diving into Misiurewicz ({ $preperiod },{ $period })
    Structure repeats every { $octaves } octaves, twist { $twist }° · any input stops
# $limit: the magnification, as written (1e300).
autopilot-dive-limit = Autopilot: dive limit reached (~{ $limit }×)
# $have: the fixed iteration count; $typical: what this depth typically needs.
autopilot-dead-end-short =
    Autopilot stopped: nothing left to aim at — iterations are fixed at { $have } and this depth typically needs ~{ $typical }
    Enable auto-iterations or raise the count
# $limit: the fixed iteration count.
autopilot-dead-end-cap =
    Autopilot stopped: nothing left to aim at — everything in view runs to the fixed iteration limit of { $limit }
    Enable auto-iterations or raise the count
autopilot-dead-end = Autopilot: no detail ahead (stopped)

## Tours (camera scripts)

# The name of the file type in the file dialog's filter list.
script-file-filter = Fractadyne script (TOML)
# $name: the script's name.
script-finished = Script finished — "{ $name }"
# The title of the notice that follows (the file's path and the error, in English).
script-load-failed = Could not load script

## Installing a language file

# The name of the file type in the file dialog's filter list.
language-file-filter = Fractadyne language

## The notice when the graphics driver's shader cache stops keeping shaders. The text is shown in a fixed-width font with its line breaks as written: break the lines where they read well.

driver-cache-title = Graphics driver not keeping shaders
# $count: how many shaders compiled again; $pipeline: the slowest one's name (a technical id: keep it); $seconds: its time in seconds (7.5).
driver-cache-body =
    Your graphics driver compiled { $count } of Fractadyne's shaders again this
    session, though it had compiled each of them earlier (the slowest:
    { $pipeline }, { $seconds } s). Its shader cache is not keeping compiled shaders, most
    often because the cache is full. Until that changes, exports, deep
    views and starting Fractadyne can each spend seconds compiling.
# $folder: the cache folder's path; $size: its size in MB (1024). "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia-folder =
    To fix it: close Fractadyne, then delete the contents of
      { $folder }   ({ $size } MB)
    The driver rebuilds the cache. To keep it from filling again, raise
    NVIDIA Control Panel > Manage 3D settings > Shader Cache Size.
# "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia =
    To fix it: raise NVIDIA Control Panel > Manage 3D settings > Shader Cache
    Size, or clear the driver's shader cache.
# "AMD Software > Graphics > Advanced > Reset Shader Cache" names AMD's own settings: use the names AMD's software shows in this language.
driver-cache-fix-amd = To fix it: AMD Software > Graphics > Advanced > Reset Shader Cache.
driver-cache-fix-other = To fix it: clear the graphics driver's shader cache.

## Why a language file could not be installed: each ends the sentence "{ $file } is not a usable language file: …".

language-install-not-utf8 = not UTF-8 text
language-install-too-large = too large
# "fractadyne-<language>.ftl" is the file name to use: keep it as written.
language-install-no-tag = it does not say which language it is (name it fractadyne-<language>.ftl)
language-install-english = English is built in
language-install-empty = it translates no message of this version
language-install-no-folder = there is no settings folder

## The notice after a session that did not end cleanly, when its iteration setting was set aside.

# $base: the iteration count automatic iterations start from.
quarantine-setting-auto = automatic iterations from a base of { $base }
# $count: the fixed iteration count.
quarantine-setting-fixed =
    { $count ->
        [one] { $count } iteration
       *[other] { $count } iterations
    }
# $end: "device" when the session lost its graphics device, "unclean" otherwise (the variants hold the wording). $depth: the magnification, as written (1.2e148). $setting and $restored: an iteration setting (quarantine-setting-auto or -fixed).
quarantine-notice =
    { $end ->
        [device] The previous session lost its graphics device at { $depth }× while set to { $setting }. This session started with { $restored } instead, so it cannot walk straight back into the same load.
       *[other] The previous session did not end cleanly at { $depth }× while set to { $setting }. This session started with { $restored } instead, so it cannot walk straight back into the same load.
    }

## The second graphics card's status line, under its setting in the control panel.

gpu2-state-off = Off: this window's graphics card renders everything.
gpu2-state-starting = Starting…
# $name: the card's name (and its graphics API).
gpu2-state-running = In use: { $name }
# $why: the error, as the graphics driver reports it.
gpu2-state-failed = Could not start: { $why }
# $name: the card's name.
gpu2-state-lost = Stopped: { $name } was lost. One card from here; choose it again to retry.
# $state: one of the lines above. "--worker-gpu" is a command-line option: keep it.
gpu2-state-pinned = { $state } (set by --worker-gpu)

## The custom-formula dialog: how deep the formula renders (one line under the formula).

formula-depth-deep = Deep zoom by perturbation, extended range past 1e28x as for the built-in fractals (every step iterated: no series approximation or BLA for custom formulas yet).
# $why: what the formula has that keeps it off the deep-zoom path (one of the formula-depth-why messages below).
formula-depth-direct = Direct rendering ({ $why } has no deep-zoom form yet): sharp until one single-precision step of c spans a pixel — about 1e4x to 1e5x, and less deep away from the origin.
formula-depth-why-init = an init section
formula-depth-why-bailout = a bailout of its own
formula-depth-why-variable = a variable kept from step to step
formula-depth-why-previous = the previous iterate
formula-depth-why-comparison = a comparison or an if block
# floor, ceil, trunc and round are functions of the formula language: keep them.
formula-depth-why-rounding = rounding (floor, ceil, trunc, round)
# "maxit" is a word of the formula language: keep it.
formula-depth-why-maxit = maxit
formula-depth-why-power = a power whose exponent varies
formula-depth-why-perturbed = an already perturbed program

## Image and frame sizes: the presets described in words (the others are standard names: HD 720p, WUXGA, DCI 4K).

# $size: the pixel size (2048×2048).
size-preset-square = Square — { $size }
# $ratio: the aspect ratio (9:16); $size: the pixel size.
size-preset-portrait = Portrait { $ratio } — { $size }
# $ratio: the aspect ratio (32:9); $size: the pixel size.
size-preset-super-ultrawide = Super ultrawide { $ratio } — { $size }

## The fractal families' names, in the pickers, the panel and the dialogs. Most are mathematicians' names for the sets: keep a name your language's mathematicians use (often the English one, or its transliteration).

fractal-family-mandelbrot = Mandelbrot
fractal-family-multibrot = Multibrot
fractal-family-tricorn = Tricorn
fractal-family-burning-ship = Burning Ship
fractal-family-celtic = Celtic
fractal-family-buffalo = Buffalo
fractal-family-phoenix = Phoenix
fractal-family-newton = Newton
# The user's own formula.
fractal-family-custom = Custom
# Conway's Game of Life.
fractal-family-life = Life
fractal-family-lsystem = L-system
# A family at a power: "Multibrot 3", "Burning Ship 4". $family: a name above; $power: the power.
fractal-with-power = { $family } { $power }

## Names of choices: colouring methods, orbit traps, palette animation, snapshots, angle units.

color-method-smooth = Smooth iteration
color-method-stripe = Stripe average
color-method-triangle = Triangle inequality
color-method-trap = Orbit trap
color-method-distance = Distance estimate
color-method-decomposition = Decomposition
# The shape an orbit trap catches the orbit with.
trap-point = Point
trap-cross = Cross
trap-circle = Circle
# Palette animation: none.
palette-anim-off = Off
palette-anim-forward = Forward
palette-anim-reverse = Reverse
palette-anim-pingpong = Ping-pong
# Palette animation: a new random gradient every so often.
palette-anim-random = Random gradients
# What the Snapshot button does.
snapshot-mode-ask = Ask each time
snapshot-mode-screen = Screen capture
snapshot-mode-render = Full render
# The unit of an angle typed into Go to location's polar fields.
angle-degrees = degrees
angle-radians = radians
angle-turns = turns

## Report an issue: the choices. (The report itself is sent in English.)

issue-kind-crash = Application crash / freeze
issue-kind-rendering = Incorrect rendering
issue-kind-performance = Performance issue
issue-kind-ui = UI / usability issue
issue-kind-feature = Feature request
issue-kind-other = Other
severity-low = Low
severity-medium = Medium
severity-high = High
severity-blocking = Blocking
repro-always = Always
repro-sometimes = Sometimes
repro-once = Happened once
repro-cannot = Can't reproduce

## The benchmark's depths and the diagnostics' tests.

# $zoom: the dive's final magnification (1e32).
bench-depth-standard = Standard ({ $zoom }×)
# $zoom: the dive's final magnification (1e48).
bench-depth-ultra = Ultra deep ({ $zoom }×)
diag-test-selftest = Self-test
diag-test-uitest = UI test
diag-test-gputest = GPU arithmetic check

## The status bar's readouts: each word is followed by a value in a slot of fixed width. Keep the words short (one word each).

status-center = center
status-cursor = cursor
status-zoom = zoom
# Short for iterations.
status-iter = iter
# The period of the minibrot jumped to.
status-period = period
# A Game of Life view's scale (followed by "1 px = 4 cells" or "8 px / cell").
status-scale = scale
# A Game of Life view's generation. Short.
status-generation = gen
# A Game of Life view's population. Short.
status-population = pop
# An L-system's order (how many times its rules were applied).
status-order = order
# How many segments and shapes an L-system drew.
status-drawn = drawn
# $value: how many cells one pixel covers (4, 9.99e300). "px" is the unit symbol: keep it.
life-scale-cells = 1 px = { $value } cells
# $value: how many pixels one cell covers (8). "px" is the unit symbol: keep it.
life-scale-pixels = { $value } px / cell

## The status bar's limit diagnostic: a short label (after a warning sign), and what it means when hovered.

# The view is as deep as this location can be rendered. Short.
status-limit-depth = depth limit
# Every pixel ran out of iterations. Short.
status-limit-exhausted = iter exhausted
# Pixels are capped at the iteration budget. Short.
status-limit-capped = iter capped
# $pixels: how many pixels wide one step of the coordinates is; $sharp_to: the magnification the view is sharp to (written out, 1,234,567).
status-limit-direct-detail = This view renders directly on the GPU in single precision, where one step of the coordinates is { $pixels } pixels wide here: neighbouring pixels compute the same point, so the image breaks into blocks. At this location it is sharp to about { $sharp_to }×. This formula has no deep-zoom (perturbation) path yet, so zooming further shows no more detail.
# $samples: the GPU buffer's capacity in orbit samples; $iterations: the iterations pixels are limited to.
status-limit-depth-detail = The reference orbit reached the GPU buffer cap ({ $samples } samples) without escaping, so pixels here are limited to ~{ $iterations } iterations and finer detail cannot be computed at this view — live or exported. This is the current depth limit for this location family.
# $budget: the largest iteration budget this view can be given.
status-limit-exhausted-detail = Every pixel still ran out of iterations at the largest budget this view can be given ({ $budget }). Either the view is inside the set here (normal at depth), or it needs more iterations than the app can currently provide — the image is the same either way, so the budget was returned to a cheap one.
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations" is the setting's name in the control panel's Quality section.
status-limit-capped-max = { $percent }% of pixels ran out of iterations at the { $budget } budget, and the escaping ones are finishing right at that ceiling — the view is under-resolved. Iterations is already at the maximum, so this view needs more than the app can currently give.
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations", "Controls" and "Quality" are the names of a setting, the control panel and its section.
status-limit-capped-raise = { $percent }% of pixels ran out of iterations at the { $budget } budget, and the escaping ones are finishing right at that ceiling — the view is under-resolved. Raise Iterations (Controls ▸ Quality) to resolve it.
