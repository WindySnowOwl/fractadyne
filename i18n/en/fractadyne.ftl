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

    Pick the point by hovering the map in "Both, linked", or with Fractal ▸ Julia c.
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
menu-help-accelerated-hover = An optional build that computes deep-zoom reference orbits 2.5-6.4x faster. Identical images; your settings carry over.
menu-help-check-updates = Check for updates
# $track: the update track (stable or beta).
menu-help-check-updates-hover = Check GitHub for a newer { $track } build
# $version: the new version; $channel: stable or pre-release.
menu-help-update-available = { $version } ({ $channel }) available — download
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
menu-toolbar-click-zoom-hover = Click-to-zoom ({ $factor }×): left-click dives into the point, right-click backs out (drag still pans). Factor in Settings ▸ Navigation.
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
