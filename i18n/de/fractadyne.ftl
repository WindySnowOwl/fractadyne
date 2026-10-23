### Fractadyne user interface: German (Deutsch).
### A draft: machine translation (Claude, by Anthropic) of the English file, 2026-10-23, not yet
### reviewed by a native speaker. Corrections are welcome as pull requests (see i18n/README.md).

## Language data: read by Fractadyne, not shown as messages.

# This language's own name, shown in the language menu.
language-name = Deutsch
# The same name in English, shown beside it in the menu.
language-english-name = German
# The writing system the app must be able to draw (an ISO 15924 code: Latn, Cyrl, Jpan, Hans, Khmr).
language-script = Latn
# Punctuation for counts, durations and sizes. Never used for coordinates or magnifications.
# One character each. A space as the group separator is written as a quoted literal, so it shows:
# { "\u00A0" } (no-break space) or { "\u202F" } (narrow no-break space).
number-decimal = ,
number-group = .
# Whether a native speaker has reviewed the translation: draft (the language menu says so) or
# reviewed. The English file is the source.
language-status = draft

## The language menu (top bar) and the Languages window

# The language menu's last item: opens the Languages window.
language-more = Weitere Sprachen…
# The Languages window's title.
language-window-title = Sprachen
# The Languages window's introduction.
language-window-intro = Die Oberfläche von Fractadyne in einer anderen Sprache. Eine Sprache ist eine Datei: Sie wird mit einer Version mitgeliefert oder lässt sich hier installieren. Koordinaten, Dateien und die Befehlszeile bleiben in jeder Sprache englisch.
# Column headers of the installed-language list.
language-col-language = Sprache
language-col-translated = Übersetzt
language-col-from = Herkunft
# Where a language comes from.
language-source-builtin = integriert
language-source-package = diese Version
language-source-user = installiert
language-source-dev = Arbeitsordner
# $percent: how much of the interface the file translates, 0–100.
language-coverage = { $percent } %
# $count: messages in the file that failed the checks and show in English instead.
language-dropped =
    { $count ->
        [one] ein Eintrag muss geprüft werden
       *[other] { $count } Einträge müssen geprüft werden
    }
# Buttons of the Languages window.
language-install = Sprachdatei installieren…
language-install-hover = Eine Fractadyne-Sprachdatei (.ftl) auswählen. Sie wird geprüft und dann in den Sprachordner der Einstellungen kopiert, wo sie über Updates hinweg erhalten bleibt.
language-open-folder = Sprachordner öffnen
language-reload = Neu laden
language-reload-hover = Die Sprachdateien erneut einlesen (nach dem Bearbeiten oder Hinzufügen einer Datei).
language-use = Verwenden
# Notices after installing a file. $name: the language's own name; $file: the file's name.
language-installed = { $name } aus { $file } installiert.
# $file: the file's name; $why: what was wrong with it.
language-install-failed = { $file } ist keine verwendbare Sprachdatei: { $why }
# Shown when a language file cannot be loaded and English is used instead. $name: the language.
language-load-failed = { $name } konnte nicht geladen werden, daher ist Fractadyne auf Englisch: { $why }
# A language whose writing system needs a font this computer does not have (Japanese or Chinese without a system font). $name: the language, in its own name.
language-font-missing = Auf diesem Computer wurde keine Schrift gefunden, die { $name } darstellen kann, daher lässt sich die Sprache nicht anzeigen. Eine Schrift für { $name } installieren (zum Beispiel das Sprachpaket des Systems) und die Sprache erneut auswählen.
# Shown after a draft translation's name in the language menu and the Languages window. Short.
language-draft = Entwurf
language-draft-hover = Ein Entwurf: Diese Übersetzung wurde noch nicht von einem Muttersprachler geprüft. Korrekturen sind willkommen, als Pull-Request für die Datei der Sprache.
# The heading over the decimal-mark choice, in the language menu and the Languages window.
language-decimal-mark = Dezimaltrennzeichen
language-decimal-mark-hover = Wie Zahlen geschrieben werden: Zeiten, Größen und Anzahlen. Koordinaten und Vergrößerungen verwenden immer einen Punkt, so wie Dateien und andere Programme sie lesen.
# $sample: a number written with this language's own punctuation (1,234.5).
language-decimal-language = Wie in der Sprache üblich ({ $sample })
# $sample: a number written with a decimal point (1,234.5).
language-decimal-point = Punkt ({ $sample })
# $sample: a number written with a decimal comma (1.234,5).
language-decimal-comma = Komma ({ $sample })

## Asked once when the system's language changes

# The window's title.
language-changed-title = Sprache
# $new: the system's new language, in its own name.
language-changed-question = Die Systemsprache ist jetzt { $new }. Auch in Fractadyne verwenden?
# Buttons. $name: a language's own name.
language-changed-use = { $name } verwenden
language-changed-keep = { $name } beibehalten

## The welcome window (first run)

welcome-title = Willkommen bei Fractadyne
welcome-intro = Ein Tiefenzoom-Explorer für die Mandelbrot-Menge und ihre Verwandten. Ein paar Bedienelemente für den Einstieg:
# The language row, at the top of the window.
welcome-language = Sprache
# Controls: the key or gesture on the left, what it does on the right.
welcome-key-drag = ziehen
welcome-key-drag-does = Ansicht verschieben
welcome-key-scroll = scrollen
welcome-key-scroll-does = zum Mauszeiger hin hinein-/herauszoomen
welcome-key-space = Space halten
welcome-key-space-does = stufenloser Dauerzoom (Shift+Space heraus)
# Shown after the click-zoom toolbar icon.
welcome-key-click = dann klicken
welcome-key-click-does = in den angeklickten Punkt eintauchen (Symbolleiste)
welcome-key-m-does = zum nächsten Minibrot springen, in dessen eigenem Maßstab
welcome-landmarks = Zum Einstieg zu einer bekannten Stelle springen:
welcome-theme = Design
welcome-watermark = „Fd“-Zeichen anzeigen
welcome-watermark-hover = Das kleine Markenzeichen unten rechts in der Ansicht. „Aus“ entfernt es aus der Live-Ansicht und aus exportierten Bildern.
welcome-updates = Updates
welcome-check-updates = Nach Updates suchen
welcome-check-updates-hover = Fragt beim Programmstart bei GitHub nach der neuesten Version. Es wird nichts hochgeladen.
welcome-files = Dateien
welcome-no-config = (kein Konfigurationsordner)
welcome-open = Öffnen
welcome-rendering-on = Rendern auf
welcome-open-help = Vollständige Hilfe öffnen  (F1)
# $icon: the Settings icon.
welcome-more-in-settings = Mehr unter Datei → { $icon } Einstellungen
welcome-get-started = Los geht’s

## Menus: shared

# A percentage in a menu (UI scale). $value: the number.
common-percent = { $value } %
# A state shown after a menu item's name, in brackets.
menu-state-connected = verbunden
menu-state-rendering = rendert
menu-state-running = läuft

## Navigate ▸ Bookmarks (also the toolbar's star)

menu-bookmarks = Lesezeichen
menu-bookmarks-add = Aktuelle Ansicht hinzufügen
menu-bookmarks-manage = Verwalten…

## The fractal picker (Fractal menu and the toolbar's dropdown)

menu-family-powers = Höhere Potenzen
menu-family-automata = Automaten
menu-family-lsystems = L-Systeme
menu-fractal-custom = Eigene Formel…
menu-fractal-library = Formelbibliothek…
menu-fractal-library-hover = Gespeicherte eigene Formeln: anwenden, bearbeiten, importieren und exportieren
# The "Show" group: which picture of the formula is drawn. $name: the formula's name (Mandelbrot, Burning Ship…).
menu-show = Anzeigen
menu-show-set = { $name }-Menge
menu-show-set-hover = Die Parameterebene — die { $name }-Menge selbst.

    Jeder ihrer Punkte ist eine andere Julia-Menge; dies ist die Karte all dieser Mengen.
menu-show-julia = Julia-Menge
menu-show-julia-hover = Ein Punkt der { $name }-Menge, als eigenes Bild gezeichnet — die Form, die entsteht, wenn dieser Punkt festgehalten wird.

    Den Punkt wählen: mit dem Mauszeiger über der Karte in „Beide, verknüpft“; ein Klick dort hält ihn fest.
menu-show-julia-none = { $name } hat keinen freien Parameter und daher keine Julia-Menge.
menu-show-both = Beide, verknüpft
menu-show-both-hover = Links die { $name }-Menge, rechts die Julia-Menge des Punktes, über dem gerade der Mauszeiger steht.

    Den Mauszeiger über die Karte bewegen und zusehen, wie sich die Julia-Menge verändert — so lässt sich am schnellsten sehen, wie beide zusammenhängen.
menu-show-both-none = { $name } hat keine Julia-Menge, mit der es sich paaren ließe.

## File menu

menu-file = Datei
menu-file-open-view = Ansicht oder Ort öffnen…
menu-file-gallery = Galerie…
menu-file-export = Bild exportieren…
menu-file-snapshot = Schnappschuss
menu-file-open-life = Life-Muster öffnen…
menu-file-open-life-hover = Eine RLE-, .cells- oder Life-1.05/1.06-Datei, als Life-Universum angezeigt.
menu-file-life-text = Life-Muster als Text…
menu-file-life-text-hover = Ein Muster als Text einfügen (RLE, wie LifeWiki es liefert).
menu-file-save-life = Life-Muster speichern…
menu-file-save-life-hover = Das Universum im aktuellen Zustand, als RLE.
menu-file-open-lsystem = L-System öffnen…
menu-file-open-lsystem-hover = Eine Fractint-.l-Datei (mit ihren Einträgen zur Auswahl) oder ein .lsys-System.
menu-file-save-lsystem = L-System speichern…
menu-file-save-lsystem-hover = Der Text des Systems, als .lsys-Datei.
menu-file-export-svg = SVG exportieren…
menu-file-export-svg-hover = Die Ansicht als SVG-Zeichnung: ihre Linien und Formen als Vektoren, in ihren Farben.
menu-file-render-client = Render-Client…
menu-file-render-client-hover = Die GPU dieses Rechners einer Renderfarm leihen: mit einem Steuerrechner verbinden (Extras ▸ Auf Farm rendern… auf einem anderen Rechner) und Bilder für ihn rendern.
menu-file-settings = Einstellungen
menu-file-reset = Programmzustand zurücksetzen…
menu-file-reset-hover = Den gesamten gespeicherten Zustand (Sitzung, Lesezeichen, Vorschaubilder) löschen und neu beginnen. Fragt vorher nach.
menu-file-quit = Beenden

## File ▸ Settings

menu-settings-fps = Bildraten-Obergrenze
menu-settings-fps-uncapped = Unbegrenzt
# $fps: frames per second.
menu-settings-fps-n = { $fps } FPS
menu-settings-ui-scale = UI-Skalierung (Schriftgröße)
menu-settings-theme = Design
menu-settings-updates = Updates
menu-settings-updates-stable-hover = Neueste stabile Version
menu-settings-updates-beta-hover = Neuester Build einschließlich Vorabversionen
menu-settings-updates-on-launch = Beim Start nach Updates suchen
menu-settings-updates-on-launch-hover = Andernfalls manuell über Hilfe → Nach Updates suchen.
# Followed by its shortcut, (Ctrl+S).
menu-settings-snapshot = Schnappschuss
menu-settings-snapshot-ask-hover = Bei jedem Drücken die Wahl anbieten
menu-settings-snapshot-screen-hover = Die Ansicht genau so speichern, wie sie auf dem Bildschirm erscheint (sofort, PNG)
menu-settings-snapshot-render-hover = Ein vollständiges Rendering mit den Export-Einstellungen, im Hintergrund (kann bei tiefen Ansichten lange dauern)
menu-settings-refcache = Referenz-Cache…
menu-settings-refcache-hover = Referenzorbits für den Tiefenzoom, auf der Festplatte gespeichert, damit ein wieder besuchter Ort in Sekunden gerendert ist. Wo er liegt, wie groß er ist, die Obergrenze und eine Möglichkeit, ihn zu leeren.

## Fractal menu

menu-fractal = Fraktal

## View menu

menu-view = Ansicht
menu-view-zoom-home = Ganz herauszoomen
menu-view-zoom-home-hover = Bis zur vollständigen Startansicht herauszoomen (animiert)
menu-view-reset = Standardansicht wiederherstellen
menu-view-reset-hover = Auf die Standardansicht des Fraktals zurücksetzen (sofort)
# Followed by its shortcuts.
menu-view-undo = Ansicht rückgängig
menu-view-redo = Ansicht wiederholen
menu-view-control-panel = Bedienfeld
menu-view-control-panel-hover = Das rechte Bedienfeld ein- oder ausblenden.
menu-view-minimap = Übersichtskarte
menu-view-minimap-hover = Eine kleine Übersicht der Startansicht mit einer „Sie sind hier“-Markierung und der Zoomtiefe anzeigen. Ein Klick darauf springt zu einem Bereich.
menu-view-perf-panel = Leistungsanzeige
menu-view-timestamp = Zeitstempel anzeigen
menu-view-timestamp-hover = Eine große Uhr der verstrichenen Zeit über die Ansicht zeichnen. Sie zeigt dasselbe „+12.345s“, mit dem das Protokoll jede Zeile versieht, sodass sich die Bildschirmaufnahme eines Problems, das sich nur in Bewegung zeigt, Bild für Bild mit dem Protokoll abgleichen lässt. Standardmäßig aus.
menu-view-zoom-target = Auto-Zoom-Ziel anzeigen
menu-view-zoom-target-hover = Während der Auto-Zoom läuft, den Bereich der Ansicht umranden, in den er hineinzoomt — den Teil, der nach einigen weiteren Vergrößerungen den Bildschirm füllt —, mit Linien von seinen Ecken zu den Ecken des Bildschirms und einem Ring um den Punkt, auf den er zusteuert. Standardmäßig aus.
menu-view-live-panel = Live-Render-Panel
menu-view-live-panel-hover = Eine Frontplatte über der Ansicht, die den Live-Renderer bei der Arbeit zeigt: was jedes Bild an die GPU geschickt hat, wie alt das Bild auf dem Bildschirm ist, den Gang durch die Iterationsbänder, die Referenzberechnungen und die Maschine darunter. B schaltet es weiter, das × in seiner Ecke schließt es; es lässt sich in jede Ecke ziehen; beim Überfahren eines Instruments erscheint, was es bedeutet und wie „gesund“ und „gestört“ aussehen.
# The live render panel's sizes.
menu-view-live-off = Aus
menu-view-live-off-hover = Kein Panel.
menu-view-live-strip = Leiste
menu-view-live-strip-hover = Die Nixie-Anzeigen und das Lampenfeld.
menu-view-live-panel-size = Panel
menu-view-live-panel-size-hover = Die Instrumente: Gang-Skala, Bildalter, Kacheln, GPU-Durchläufe, Tiefenskala, Lampen.
menu-view-live-full = Voll
menu-view-live-full-hover = Das Panel und der Maschinenraum: Schritte, GPUs, Speicher, Kerne, Pixel.
menu-view-live-opacity = Deckkraft
menu-view-watermark = „Fd“-Wasserzeichen anzeigen
menu-view-watermark-hover = Ein dezentes „Fd“-Markenzeichen in die untere rechte Ecke der Live-Ansicht und exportierter Bilder zeichnen. Standardmäßig an.

## Color menu

menu-color = Farbe
menu-color-method = Methode
menu-color-palette = Palette
menu-color-custom = Eigene
menu-color-duotone = Duoton
menu-color-binary = Binär (Menge)
menu-color-binary-hover = Flach in zwei Farben: in der Menge oder außerhalb, ohne Farbverlauf.
menu-color-edit-gradient = Farbverlauf bearbeiten…
menu-color-normalize = Tiefe Farben normalisieren
menu-color-normalize-hover = Die Palette auf den gemessenen Fluchtbereich der Ansicht umrechnen, wenn ein dichtes Feld sonst als Gesprenkel erscheinen würde. Eine Ansicht ohne Aliasing bleibt unverändert — „Bereich immer anpassen“ ankreuzen, um trotzdem umzurechnen. Nur bei der Methode „Glatte Iteration“.
menu-color-more = Zyklus, Versatz, Animation und Effekte: Bedienfeld ▸ Färbung

## Tools menu

menu-tools = Extras
# Followed by its shortcut, (M).
menu-tools-find-minibrot = Minibrot finden + hinzoomen
menu-tools-find-minibrot-hover = Die Ansichtsmitte per Newton-Verfahren auf das exakte Zentrum des nahen Minibrots setzen, seine Periode melden und auf den eigenen Maßstab des Minibrots zoomen — oft viele Größenordnungen in einem Schritt. Schon tiefer als das Minibrot? Dann bewegt sich nur die Mitte.
menu-tools-solver = Newton-/Misiurewicz-Löser…
menu-tools-solver-hover = Das exakte Minibrot-Zentrum (Periode + Größe) oder den der Ansicht nächstgelegenen Misiurewicz-Punkt berechnen — Newton-Verfahren mit voller Genauigkeit, mit dem abstoßenden Multiplikator λ. Öffnet den Orts-Dialog, in dem die Löser liegen.
# Followed by its shortcut, (A).
menu-tools-autopilot = Auto-Zoom (Autopilot)
menu-tools-autopilot-stop = Autopilot anhalten
menu-tools-autopilot-hover = Fortlaufender Tiefenzoom ohne Zutun: taucht auf den detailreichsten Bereich zu und korrigiert dabei laufend den Kurs. Jede Navigationseingabe hält ihn an.
menu-tools-priority = Priorität:
menu-tools-priority-speed = Tempo
menu-tools-priority-speed-hover = Mit der eingestellten Geschwindigkeit zoomen; mit dem ersten Bild aktualisieren, das Details zeigt.
menu-tools-priority-quality = Qualität
menu-tools-priority-quality-hover = Nur vollständig aufgelöste Bilder; der Zoom wartet auf sie.
menu-tools-target = Ziel:
menu-tools-target-detail = Detail
menu-tools-target-detail-hover = Der reichste Rand in der Ansicht, während des Tauchgangs immer wieder neu gewählt.
menu-tools-target-misiurewicz = Misiurewicz-Punkt
menu-tools-target-misiurewicz-hover = Der nächste Misiurewicz-Punkt, einmal berechnet und dann exakt angesteuert (nur Mandelbrot).
menu-tools-orbits = Orbit-Einblendung
menu-tools-orbits-hover = Den Iterationspfad des Punktes unter dem Mauszeiger zeichnen. Animationsoptionen: Bedienfeld ▸ Einblendungen.
menu-tools-orbits-normalize = Normalisieren (an Ansicht anpassen)
menu-tools-orbits-normalize-hover = Den Orbit an die ganze Ansicht anpassen, damit er bei jedem Zoom gut im Bild bleibt (statt ihn über den Ansichtsbereich abzubilden, wo er in der Tiefe aus dem Bild fliegt).
menu-tools-play-tour = Tour abspielen…
menu-tools-play-tour-hover = Ein Tour-Skript zum Abspielen auswählen. Das ▶ der Symbolleiste spielt das zuletzt gewählte mit einem einzigen Klick erneut ab.
menu-tools-tour-from-view = Tour aus aktueller Ansicht…
menu-tools-tour-from-view-hover = Eine Tour erstellen, die von der Gesamtansicht bis zur aktuellen Ansicht zoomt — eine Beschriftung hinzufügen und die Dauer des Tauchgangs festlegen. Gespeichert als Tour-Skript (.toml), das sich bearbeiten, abspielen oder rendern lässt.
menu-tools-render-tour = Tour rendern…
menu-tools-render-tour-hover = Die geladene Tour als PNG-Bildfolge (und wahlweise als mp4) in einem eigenen Prozess rendern.
menu-tools-close-player = Tour-Player schließen
menu-tools-close-player-hover = Die Tour anhalten, ihren Player schließen und die eigenen Iterations- und Färbungseinstellungen wiederherstellen
menu-tools-render-farm = Auf Farm rendern…
menu-tools-render-farm-hover = Die geladene Tour auf mehreren Rechnern rendern: Dieser steuert den Auftrag, andere schließen sich mit Datei ▸ Render-Client… an.
menu-tools-benchmark = Benchmark…
menu-tools-benchmark-hover = Die Rendergeschwindigkeit messen — mit den aktuellen Einstellungen oder als standardisierter Lauf (feste Auflösung + Einstellungen), vergleichbar zwischen Rechnern. Der Dauertest wiederholt ihn, um Stabilität / thermische Drosselung zu prüfen.
menu-tools-last-benchmark = Letzten Benchmark anzeigen

## Navigate menu

menu-navigate = Navigation
menu-navigate-goto = Zu Ort springen…
menu-navigate-goto-hover = Mitte/Zoom eingeben, zu einem bekannten Punkt springen oder ein Misiurewicz-/Minibrot-Merkmal nahe der Ansicht per Newton-Verfahren berechnen.
menu-navigate-misiurewicz = Misiurewicz-Explorer…
menu-navigate-misiurewicz-hover = Ansehen, wie die kleinen Misiurewicz-Typen aussehen — Vorschaubilder mit Zoom-Periode und Drehung jedes Punktes — und jeden davon per Newton-Verfahren bis in beliebige Tiefe berechnen. (Mandelbrot.)
menu-navigate-exact = Exakte Punkte (Ausdrücke)
menu-navigate-exact-note = Ansatzpunkte der Knospen der Hauptkardioide, als exakte Ausdrücke angegeben (cos/sin eines Bruchteils von 2π). Die Koordinate wird als Ausdruck gespeichert, sodass das Hineinzoomen exakt auf dem Punkt bleibt.
menu-navigate-random = Zufälliger Ort
menu-navigate-random-hover = Zu einem zufälligen, detailreichen Randpunkt springen
menu-navigate-import-kfr = .kfr importieren…
menu-navigate-import-kfr-hover = Eine Ortsdatei von Kalles Fraktaler laden
menu-navigate-share = Ort teilen…
menu-navigate-share-hover = Einen eigenständigen Ort (.fdn) kopieren / einfügen / speichern / laden: Fraktal, Mitte in voller Genauigkeit, Zoom, Färbung.

## Help menu

menu-help = Hilfe
# Followed by its shortcut, (F1).
menu-help-reference = Hilfe & Referenz…
menu-help-welcome = Willkommen / Schnellstart…
menu-help-welcome-hover = Die Übersicht beim ersten Start: die wichtigsten Bedienelemente und einige bekannte Stellen.
menu-help-diagnostics = Diagnose…
menu-help-diagnostics-hover = Prüfen, ob Fractadyne auf dieser Hardware korrekt arbeitet, und das Ergebnis einem Fehlerbericht beifügen
menu-help-report = Problem melden…
menu-help-report-hover = Ein GitHub-Issue (oder eine E-Mail) mit Absturzbericht, Protokoll und Ort im Anhang erstellen
menu-help-accelerated = Schnellerer Tiefenzoom…
menu-help-accelerated-hover = Ein optionaler Build, der Referenzorbits für den Tiefenzoom 2,5- bis 6,4-mal schneller berechnet. Identische Bilder; die Einstellungen werden übernommen.
menu-help-check-updates = Nach Updates suchen
# $track: the update track (stable or beta).
# $track: "stable" or "beta", the update track in use (the variants hold the wording).
menu-help-check-updates-hover =
    { $track ->
        [beta] Bei GitHub nach einem neueren Beta-Build (Vorabversionen) suchen
       *[other] Bei GitHub nach einem neueren stabilen Build suchen
    }
# $version: the new version; $channel: "beta" for a pre-release, "stable" otherwise (the variants hold the wording).
menu-help-update-available =
    { $channel ->
        [beta] { $version } (Beta) verfügbar — herunterladen
       *[other] { $version } (stabil) verfügbar — herunterladen
    }
menu-help-up-to-date = Aktuell
menu-help-tagline = Nativer Fraktal-Explorer
menu-help-license = Lizenz
# $year and $holder: the copyright year and holder, as the code gives them.
menu-help-copyright = © { $year } { $holder }. Doppelt lizenziert; Nutzung nach Wahl unter einer der beiden Lizenzen.
menu-help-source = Quellcode

## The toolbar (its buttons are icons; these are their tooltips)

menu-toolbar-julia = Julia
menu-toolbar-julia-hover = Die Julia-Menge dieser Formel fensterfüllend anzeigen
menu-toolbar-dual-hover = Beide, verknüpft: links die Menge, rechts die Julia-Menge des Punktes unter dem Mauszeiger
menu-toolbar-open-hover = Eine Ansicht oder einen Ort öffnen (PNG / EXR, .fdn, .kfr)
menu-toolbar-gallery-hover = Galerie
menu-toolbar-export-hover = Bild exportieren…
menu-toolbar-share-hover = Ort teilen — eine eigenständige .fdn kopieren, einfügen, speichern oder laden (Fraktal, Mitte in voller Genauigkeit, Zoom, Färbung)
menu-toolbar-zoom-in-hover = Hineinzoomen
menu-toolbar-zoom-out-hover = Herauszoomen
# $factor: the click-zoom factor (4, 10…).
menu-toolbar-click-zoom-hover = Klick-Zoom ({ $factor }×): Linksklick taucht in den Punkt ein, Rechtsklick zoomt zurück (Ziehen verschiebt weiterhin). Faktor unter Bedienfeld ▸ Navigation.
# Added to the click-to-zoom tooltip in the dual view.
menu-toolbar-click-zoom-dual = Solange dies aktiv ist, das Julia-c mit Ctrl+Klick festsetzen.
menu-toolbar-autopilot-running-hover = Auto-Zoom läuft — zum Anhalten klicken
menu-toolbar-autopilot-hover = Auto-Zoom: in Richtung Details tauchen (A)
menu-toolbar-reset-hover = Ansicht zurücksetzen (sofort)
menu-toolbar-home-hover = Zur Startansicht herauszoomen (animiert)
menu-toolbar-bookmarks-hover = Lesezeichen: diese Ansicht speichern oder zu einer gespeicherten springen
# $name: the palette's name.
menu-toolbar-next-palette-hover = Nächste Palette ({ $name })
# $file: the last tour's file name.
menu-toolbar-play-last-hover = Tour abspielen — { $file } (Klick) · spielt die letzte Tour erneut ab; eine andere über Extras ▸ Tour abspielen… wählen
menu-toolbar-play-hover = Eine Tour abspielen… (Extras ▸ Tour abspielen)
menu-toolbar-speed-hover = Wiedergabegeschwindigkeit
menu-toolbar-fullscreen-hover = Vollbild ein/aus

## Dialogs: words shared by many windows

# A button that closes a window without doing what it offers.
dlg-cancel = Abbrechen
# A button that dismisses a window (nothing in it is pending).
dlg-close = Schließen
# A button that copies the window's text to the clipboard (after a copy icon).
dlg-copy = Kopieren
# A button that jumps to a location.
dlg-go = Los
# A button that fills the fields from the view on screen.
dlg-use-current = Aktuelle übernehmen
# A notice after a file could not be written. $error: the system's reason, as it gives it.
dlg-save-failed = Speichern fehlgeschlagen: { $error }
# A notice when no graphics device is available to render with.
dlg-gpu-not-available = GPU nicht verfügbar
# The name of the plain-text file type in a save dialog (the extension .txt is added by the code).
dlg-filter-text = Text

## Asked after a session that did not shut down cleanly

dlg-crash-title = Fractadyne wurde nicht ordnungsgemäß beendet
dlg-crash-intro = Die vorige Sitzung endete unerwartet. Auf diesem Rechner wurde ein Bericht gespeichert — möchten Sie ihn senden?
dlg-crash-review = Nichts wird gesendet, bevor Sie es geprüft haben: Der nächste Bildschirm zeigt genau, was enthalten wäre, und die Systeminformationen lassen sich mit einem Häkchen ausschließen.
dlg-crash-report = Melden…
dlg-crash-report-hover = Öffnet „Problem melden“, mit dem Absturzbericht und dem Ort des Absturzes im Anhang.
dlg-crash-not-now = Nicht jetzt
dlg-crash-dont-ask = Nach einem Absturz nicht mehr fragen
dlg-crash-dont-ask-hover = Berichte werden weiterhin lokal gespeichert, und Hilfe ▸ Problem melden funktioniert immer.

## Asked after a session that ended with a second graphics device in use

dlg-second-device-title = Zweites Grafikgerät ausschalten?
# $device: the graphics device's name and details, as the system gives them.
dlg-second-device-intro = Die vorige Sitzung endete unerwartet, während ein zweites Grafikgerät in Gebrauch war ({ $device }). Es könnte die Ursache sein.
dlg-second-device-note = Ist es aus, rendert die Karte, die das Fenster zeichnet, alles, wie zuvor. Es lässt sich unter Erweitert ▸ Zweite Grafikkarte wieder einschalten.
dlg-second-device-off = Ausschalten
dlg-second-device-keep = Eingeschaltet lassen

## Shown when the previous session's iteration setting was set aside

dlg-quarantine-title = Iterationseinstellung zurückgestellt
dlg-quarantine-change = Sie lässt sich jederzeit unter Bedienfeld ▸ Qualität ändern.
dlg-quarantine-keep = Standard beibehalten
# A button. $setting: the iteration setting the previous session used (such as "20,000 iterations").
dlg-quarantine-restore = { $setting } wiederherstellen

## Navigate ▸ Go to location

dlg-goto-title = Zu Ort springen
# Before two choices: "x, y" (cartesian) and the polar form below.
dlg-goto-enter-as = Eingabe als:
# The polar way of entering a point: an offset plus a radius r at an angle θ. Keep the symbols.
dlg-goto-polar = Versatz + r ∠ θ
# The tooltip of the "?" button beside the entry choice.
dlg-goto-help-hover = Was die Felder annehmen: Brüche, Potenzen, sqrt/sin/cos…, pi und andere Konstanten — öffnet Hilfe → Koordinatenausdrücke
# Labels over the entry fields. x0, y0, r, θ, Re and Im are the fields' symbols: keep them.
dlg-goto-polar-offset = Versatz der Mitte  (x0, y0 — für einen reinen Polarpunkt 0 lassen)
dlg-goto-polar-radius = Radius r, Winkel θ
dlg-goto-re = Mitte — Realteil (Re)
dlg-goto-im = Mitte — Imaginärteil (Im)
dlg-goto-zoom = Zoom (Vergrößerung)
# A warning under the zoom field, after a ⚠ sign. $have: the iteration count set; $typical: the count this depth typically needs.
dlg-goto-iter-warning = Die Iterationen sind auf { $have } festgelegt; diese Tiefe braucht typischerweise ~{ $typical }. Die Ansicht wird flach gerendert, bis der Wert erhöht wird — oder Auto-Iterationen aktivieren (Bedienfeld ▸ Qualität), die sich von selbst anpassen.
dlg-goto-copy-hover = Diesen Ort in die Zwischenablage kopieren
dlg-goto-cancel-hover = Schließen, ohne irgendwohin zu springen
# The expressions in brackets are examples of what can be typed: keep them as they are.
dlg-goto-paste-note = Mitte/Zoom von jemand anderem einfügen oder zum Teilen kopieren. Koordinaten nehmen Ausdrücke an: exakte Brüche (-3/4), Funktionen und Konstanten (-0.5 + 0.25*cos(pi/4), sqrt, root(x,n), ^; Winkel im Bogenmaß), und das Realteil-Feld nimmt einen ganzen komplexen Wert an ((37+16i)/100), der beide Felder füllt.
# A section that opens and closes.
dlg-goto-feature = Zu Merkmal springen (Mandelbrot)
dlg-goto-feature-poi = Bekannte Punkte:
# The dropdown's text before a point is picked.
dlg-goto-feature-poi-pick = Zu einem interessanten Punkt springen…
dlg-goto-feature-solve = Oder ein Merkmal nahe der aktuellen Ansicht berechnen:
# Two choices: what to solve for. Misiurewicz is a mathematician's name (a Misiurewicz point).
dlg-goto-feature-misiurewicz = Misiurewicz
dlg-goto-feature-minibrot = Nächstes Minibrot
# Labels of two small number fields; k and p are their symbols.
dlg-goto-feature-preperiod = Vorperiode k
dlg-goto-feature-period = Periode p
# The faint text in an empty field: left empty, the value is found automatically.
dlg-goto-feature-auto-hint = auto
# A small button that empties both fields, so they are found automatically again.
dlg-goto-feature-auto = Auto
dlg-goto-feature-auto-hover = Beide Felder leeren, damit die nächste Suche Vorperiode und Periode aus dem Orbit in der Ansichtsmitte ermittelt.
dlg-goto-feature-kp-hover = Beide leer lassen, um sie aus dem Orbit in der Ansichtsmitte zu ermitteln; die gefundenen Werte werden hier eingetragen. Niemand, der eine Spirale betrachtet, kennt ihre Vorperiode, und dass man eine angeben musste, machte diesen Löser in der Praxis unbrauchbar.
# While a solve runs, followed by its elapsed time and the note below.
dlg-goto-feature-solving = Wird berechnet…
dlg-goto-feature-solving-note = (beliebige Genauigkeit; tiefe Ansichten dauern länger)
dlg-goto-feature-cancel-hover = Nicht länger warten. Die Berechnung selbst läuft im Hintergrund zu Ende, und ihr Ergebnis wird verworfen — nichts bleibt halbfertig zurück.
dlg-goto-feature-find = Nahe der Ansicht suchen
dlg-goto-feature-find-hover =
    Von der aktuellen Mitte aus per Newton-Verfahren auf das exakte Merkmal rechnen (beliebige Genauigkeit). Vorher in die Nähe des Merkmals navigieren.

    Vor dem Berechnen das Zoom-Feld oben auf die GEWÜNSCHTE Tiefe setzen: Der Punkt wird mit der Genauigkeit berechnet, die diese Tiefe braucht, und die Ansicht springt direkt dorthin. Das ist eine einzige Berechnung statt einer langen Folge manueller Zoomschritte.
dlg-goto-solve-cancelled = Berechnung abgebrochen.

## Snapshot (the camera button): its tooltip, and the choice on the first press

# The camera button's tooltip, one per Snapshot setting. Followed by its shortcut, (Ctrl+S).
dlg-snapshot-hover-ask = Schnappschuss — die Ansicht wie angezeigt oder ein vollständiges Rendering speichern; beim ersten Drücken wird gefragt
dlg-snapshot-hover-screen = Schnappschuss — die Ansicht genau wie angezeigt im letzten Exportordner speichern
dlg-snapshot-hover-render = Schnappschuss — vollständiges Rendering mit den Export-Einstellungen, in den letzten Exportordner
# The window's title.
dlg-snapshot-title = Schnappschuss
dlg-snapshot-question = Was soll der Schnappschuss (die Kamera-Schaltfläche, Ctrl+S) speichern?
# The first choice, and a short tag beside it.
dlg-snapshot-screen = Bildschirmaufnahme
dlg-snapshot-screen-tag = sofort
dlg-snapshot-screen-note = Speichert die Ansicht genau so, wie sie auf dem Bildschirm erscheint, in Bildschirmauflösung, als PNG im letzten Exportordner. Was man sieht, das bekommt man.
# The second choice, and a short tag beside it.
dlg-snapshot-render = Vollständiges Rendering
dlg-snapshot-render-tag = kann lange dauern
# $size: the image size, as 1920×1080; $ss: the supersampling factor (1–4); $format: PNG or EXR.
dlg-snapshot-render-note = Rendert mit den Export-Einstellungen — { $size }, { $ss }× Supersampling, { $format }. Beste Qualität, doch eine tiefe Ansicht kann Minuten bis Stunden dauern. Es läuft im Hintergrund; Datei → Bild exportieren… zeigt den Fortschritt und kann es abbrechen.
dlg-snapshot-remember = Auswahl merken und nicht mehr fragen
dlg-snapshot-change = Das lässt sich jederzeit unter Datei → Einstellungen → Schnappschuss ändern.

## Share location (.fdn)

dlg-share-title = Ort teilen
dlg-share-intro = Ein eigenständiger Ort (Fraktal, Mitte in voller Genauigkeit, Zoom, Färbung). Zum Teilen kopieren, oder einen Ort einfügen/laden und „Anwenden“ wählen.
dlg-share-thumb = Vorschaubild in die gespeicherte Datei aufnehmen
dlg-share-thumb-hover = „.fdn speichern…“ bettet ein kleines Bild dieser Ansicht ein, damit die Datei in der Galerie erkennbar ist. Fügt etwa 55 KB hinzu und ist nicht Teil des Texts oben.
dlg-share-copy-hover = Den Text in die Zwischenablage kopieren
# Buttons (the first after a save icon).
dlg-share-save = .fdn speichern…
dlg-share-load = .fdn laden…
dlg-share-apply = Anwenden
dlg-share-apply-hover = Zum Ort im Textfeld springen
dlg-share-copied = In die Zwischenablage kopiert.

## Help ▸ Report an issue (the report itself stays in English: it is read by the maintainer)

dlg-report-title = Problem melden
dlg-report-intro = Am besten wird ein Problem als GitHub-Issue gemeldet — es ist öffentlich und durchsuchbar, und Sie sehen, wann es behoben ist. E-Mail geht auch. Bitte zuerst alles unten prüfen: Nichts verlässt diesen Rechner, bevor Sie es veranlassen.
# Labels before three dropdowns.
dlg-report-type = Art:
dlg-report-severity = Schweregrad:
dlg-report-repro = Reproduzierbar:
dlg-report-describe = Beschreiben, was passiert ist:
# The faint text in the empty description box.
dlg-report-describe-hint = Was passiert ist, die Schritte, um es zu reproduzieren, und was stattdessen erwartet wurde.
# Before the checkboxes of what the report carries.
dlg-report-include = Beifügen:
dlg-report-sysinfo = Systeminfo (Version, Betriebssystem, CPU, GPU, VRAM)
dlg-report-location = Aktueller Ort (.fdn)
dlg-report-log = Aktuelles Protokoll
dlg-report-frames = Bildaufzeichnung (die letzte Minute und die letzten 40 Bilder)
dlg-report-crash = Letzter Absturzbericht
dlg-report-crash-none = Letzter Absturzbericht (keiner gefunden)
dlg-report-crash-view = Ort des Absturzes (.fdn)
dlg-report-crash-view-none = Ort des Absturzes (.fdn) (keiner gefunden)
dlg-report-crash-view-hover = Die genaue Ansicht, bei der ein Absturz oder der Verlust des Grafikgeräts geschah — das Einzige, womit ein Maintainer ihn reproduzieren kann. Wird automatisch aufgezeichnet.
dlg-report-test = Ergebnis des Diagnosetests
dlg-report-test-none = Ergebnis des Diagnosetests (einen unter Hilfe → Diagnose… ausführen)
dlg-report-test-hover = Ein maschinell geprüftes Bestanden/Nicht bestanden von der eigenen Hardware — für einen Maintainer weit nützlicher als eine Beschreibung allein
# A section that opens and closes. $size: the report's size, as "12 KB".
dlg-report-preview = Vorschau des zu Sendenden  ({ $size })
dlg-report-github = GitHub-Issue öffnen
dlg-report-github-hover = Eine Seite für ein neues Issue mit ausgefülltem Titel öffnen; der Bericht wird in die Zwischenablage kopiert — in den Text des Issues einfügen (Ctrl+V).
# Buttons (after a copy and a save icon).
dlg-report-copy = Bericht kopieren
dlg-report-copy-hover = Alles in die Zwischenablage kopieren
dlg-report-save = Bericht speichern…
dlg-report-save-hover = Als .txt zum Anhängen speichern
dlg-report-by-email = Oder per E-Mail:
dlg-report-gmail = In Gmail verfassen
# $address: the email address reports go to.
dlg-report-gmail-hover = Ein Gmail-Fenster zum Verfassen an { $address } mit ausgefülltem Betreff öffnen; den kopierten Bericht einfügen
dlg-report-email = E-Mail-Programm…
dlg-report-email-hover = Das Standard-E-Mail-Programm öffnen (mailto) — erfordert ein eingerichtetes E-Mail-Programm
dlg-report-tip = Tipp: Bei jeder dieser Möglichkeiten wird der Bericht in die Zwischenablage kopiert — in das Issue oder den Nachrichtentext einfügen (Ctrl+V). Bitte nach Möglichkeit auch einen Screenshot oder ein Beispielbild anhängen (hineinziehen); das hilft bei Render- und Oberflächenproblemen sehr.
# Notices in the window after a button is pressed.
dlg-report-opened-github = GitHub geöffnet — der Bericht liegt in der Zwischenablage; in das Issue einfügen.
dlg-report-copied = Bericht in die Zwischenablage kopiert.
# $path: where the report was saved.
dlg-report-saved = Gespeichert unter { $path }
dlg-report-opened-email = E-Mail-Programm geöffnet — der Bericht liegt in der Zwischenablage; dort einfügen.
dlg-report-opened-gmail = Gmail-Fenster geöffnet — den kopierten Bericht in den Text einfügen (Ctrl+V).

## A location whose checksum does not match

dlg-checksum-title = Dieser Ort ist möglicherweise beschädigt
# After a ⚠ sign.
dlg-checksum-mismatch = Seine Prüfsumme passt nicht zu den Daten daneben.
dlg-checksum-why = Das bedeutet meist, dass der Text unterwegs gekürzt oder verändert wurde — eine unvollständige Kopie aus einem Chatfenster oder Forenbeitrag ist die übliche Ursache.
# A label over where the location came from (a file path, or the clipboard).
dlg-checksum-source = Quelle
# The two checksums, side by side: the one written in the file, and the one computed from its data.
dlg-checksum-in-file = in der Datei
dlg-checksum-from-data = aus den Daten
dlg-checksum-load-note = Er lässt sich trotzdem laden — die Koordinaten können dennoch stimmen, und nichts wird überschrieben. Alles Unlesbare wird nach dem Laden gemeldet.
dlg-checksum-load = Trotzdem laden
dlg-checksum-load-hover = Trotz der Abweichung dorthin springen
# Notices after loading it. $zoom: the magnification, as 1.2e148.
dlg-checksum-loaded = Geladen @ { $zoom }× trotz abweichender Prüfsumme
# $zoom: the magnification, as 1.2e148; $note: what the loader found wrong with the location.
dlg-checksum-loaded-note = Geladen @ { $zoom }× — { $note }
dlg-checksum-not-loaded = Nicht geladen.

## File ▸ Reset application state

dlg-reset-title = Programmzustand zurücksetzen
# After a ⚠ sign, followed by a list.
dlg-reset-warning = Dies löscht dauerhaft alle gespeicherten Fractadyne-Daten:
# The list's items, each after a bullet.
dlg-reset-session = die gespeicherte Sitzung (aktuelle Ansicht, Färbung, Einstellungen)
dlg-reset-bookmarks = alle Lesezeichen und ihre Vorschaubilder
dlg-reset-refcache = den Referenzorbit-Cache (wird beim erneuten Besuch von Orten neu aufgebaut)
# A label over the folder that is deleted.
dlg-reset-location = Speicherort
dlg-reset-note = Das lässt sich nicht rückgängig machen. Die aktuelle Sitzung wird beim Beenden nicht erneut gespeichert, sodass beim nächsten Start die Standardwerte geladen werden.
dlg-reset-confirm = Alles zurücksetzen
dlg-reset-done = Programmzustand zurückgesetzt — beim nächsten Start werden die Standardwerte geladen.
# $error: the system's reason, as it gives it.
dlg-reset-failed = Zurücksetzen fehlgeschlagen: { $error }

## Tools ▸ Tour from current view

dlg-tour-title = Tour aus aktueller Ansicht
dlg-tour-intro = Erstellt eine Tour, die von der Gesamtansicht bis zur aktuellen Position zoomt.
# A label before the fractal and magnification the tour ends at.
dlg-tour-destination = Ziel
dlg-tour-notation = Beschriftung (optional, während des Tauchgangs angezeigt)
# The faint example text in the empty caption box.
dlg-tour-notation-hint = z. B. Tauchgang zum Misiurewicz-Dreiarm…
dlg-tour-duration = Zoomdauer
# $rate: how fast the dive zooms, in powers of ten a second (as 2.5).
dlg-tour-rate = ≈ { $rate } Dekaden/s
# A small button that puts the duration back to its default.
dlg-tour-reset = Zurücksetzen
dlg-tour-reset-hover = Standardwert nach Tiefe
dlg-tour-duration-note = Der Standardwert wächst mit der Tiefe, damit der Tauchgang gut anzusehen bleibt. Schnellere Tauchgänge können bei extremem Zoom einfarbige Bilder aufblitzen lassen.
# A button, after a save icon.
dlg-tour-save = Tour speichern…
# The file type's name in the save dialog.
dlg-tour-filter = Fractadyne-Skript (TOML)
# A notice. $file: the saved file's name. --render-tour is a command-line option: keep it.
dlg-tour-saved = Tour gespeichert → { $file }. Abspielen über Extras → Tour abspielen oder mit --render-tour rendern.

## Navigate ▸ Bookmarks ▸ Manage…

dlg-bookmarks-title = Lesezeichen
# The faint text in the empty name box.
dlg-bookmarks-name-hint = Name (optional)
# A button, after a bookmark icon.
dlg-bookmarks-add = Aktuelle Ansicht hinzufügen
dlg-bookmarks-empty = Noch keine Lesezeichen. Eines aus der aktuellen Ansicht hinzufügen.
# The tooltip of a bookmark's delete (bin) button.
dlg-bookmarks-delete-hover = Löschen

## Tools ▸ Benchmark…

dlg-bench-title = Benchmark
dlg-bench-mode = Modus
dlg-bench-current = Aktuelle Einstellungen
dlg-bench-current-hover = Die feste Tiefenzoom-Tour in der Live-Ansicht mit der aktuellen Auflösung und den aktuellen Einstellungen abspielen.
dlg-bench-standard = Standardisiert
dlg-bench-standard-hover = Auflösung + alle Render-Einstellungen festlegen, damit das Ergebnis zwischen Rechnern vergleichbar ist.
dlg-bench-resolution = Auflösung
dlg-bench-depth = Tiefe
# Standard and Ultra are the two depths in the dropdown this describes.
dlg-bench-depth-hover = Beide Tauchgänge gelangen in den tiefen floatexp/BLA-Pfad; Ultra (1e48×) verbringt dort weit mehr Zeit als Standard (1e32×) und dauert entsprechend länger.
dlg-bench-burnin = Dauertest (Wiederholung)
dlg-bench-burnin-hover = Den Benchmark wiederholt ausführen, um Stabilität und thermische Drosselung sichtbar zu machen.
# The label of a slider: how many times the benchmark runs.
dlg-bench-passes = Durchläufe
# $size: the image size, as 1920×1080; $ss: the supersampling factor; $depth: the dive's final magnification, as 1e32.
dlg-bench-std-summary = Rendert offscreen mit { $size }, { $ss }× SS, Mandelbrot/glatt, Tauchgang über 60 Bilder bis { $depth }×.
dlg-bench-run = Starten

## The standardized benchmark's progress window

dlg-bench-progress-title = Benchmark läuft…
# $resolution: the benchmark's resolution, as the dropdown names it.
dlg-bench-progress-mode = Standardisiert · { $resolution }
# $pass: the pass running now; $total: how many passes there are.
dlg-bench-progress-pass = Dauertest-Durchlauf { $pass }/{ $total }
dlg-bench-progress-dive = Der feste Tauchgang wird gerendert…
# On the progress bar. $done: frames rendered; $total: frames in the dive.
dlg-bench-progress-frame = Bild { $done }/{ $total }
# $fps: frames per second, as 41.3.
dlg-bench-progress-last = letzter Durchlauf: { $fps } fps

## The benchmark's results

dlg-bench-results-title = Benchmark-Ergebnisse
# Buttons, each after an icon.
dlg-bench-results-run-again = Erneut ausführen…
dlg-bench-results-save = Speichern…
dlg-bench-results-none = Bisher wurde kein Benchmark ausgeführt.
dlg-bench-results-saved = Benchmark gespeichert.

## The accelerated build's notice when its arithmetic libraries are missing

dlg-backend-title = Beschleunigter Build — verwendet die integrierte Arithmetik
dlg-backend-dont-show = Nicht mehr anzeigen

## File ▸ Gallery…

dlg-gallery-title = Galerie
# A button that picks the folder to show.
dlg-gallery-folder = Ordner…
# A button, after a refresh icon.
dlg-gallery-refresh = Aktualisieren
dlg-gallery-empty = Keine Fractadyne-Bilder in diesem Ordner.
# An image's title when it does not name its fractal.
dlg-gallery-untitled = Fractadyne-Bild
# $zoom: the image's magnification, as 1.2e148; $saved: when it was saved, as the file gives it.
dlg-gallery-zoom = Zoom { $zoom }   { $saved }
# The notes saved with the image, in quotation marks. Use your language's quotation marks.
dlg-gallery-notes = „{ $notes }“
dlg-gallery-open = Diese Ansicht öffnen
dlg-gallery-loaded = Ansicht aus der Galerie geladen.

## File ▸ Export image…

dlg-export-title = Bild exportieren
# A dropdown of image sizes.
dlg-export-size = Größe
# The size dropdown's text for a size set by hand. $size: the size, as 1920×1080.
dlg-export-size-custom-current = Eigene — { $size }
# The size dropdown's first choice.
dlg-export-size-custom = Eigene…
dlg-export-size-custom-hover = Die Breite selbst festlegen; die Höhe folgt dem Seitenverhältnis unten.
dlg-export-width = Breite (px)
# After the width field. $height: the height in pixels, which follows from the aspect.
dlg-export-height-from-aspect = × { $height } (aus Seitenverhältnis)
dlg-export-ss = Supersampling
dlg-export-aspect = Seitenverhältnis
# The aspect choice that keeps the window's own shape.
dlg-export-aspect-window = Wie Fenster
# Before the choices PNG and OpenEXR.
dlg-export-format = Format:
dlg-export-hud = Orts-HUD
dlg-export-hud-hover = Ein Feld mit Zoomstufe und Koordinaten oben links in das exportierte Bild einbrennen (skaliert mit der Ausgabe; zeigt Zoom + Mitte der Karten-/Mandelbrot-Ansicht).
dlg-export-glitch = Glitch-Korrektur
dlg-export-glitch-hover = Korrektur von Glitches der Störungsrechnung mit mehreren Referenzen. Wird bei sehr tiefen (floatexp-)Exporten einer Einzelansicht automatisch übersprungen, damit Referenzberechnung und Rendering außerhalb des Haupt-Threads laufen und das Programm bedienbar bleibt. Nie verwendet für Mandelbrot- oder Multibrot-Ansichten (außerhalb des Julia-Modus): Gegen eine exakte Berechnung geprüft, behob sie dort nichts und kostete bis zum 5-Fachen der Exportzeit.
dlg-export-deep-note = Tiefer Export: Die Referenz wird außerhalb des Haupt-Threads berechnet (die Oberfläche bleibt bedienbar); die Glitch-Korrektur wird in dieser Tiefe übersprungen.
# A dropdown, shown in the dual (map + Julia) view, and its three choices.
dlg-export-dual = Doppel-Layout
dlg-export-dual-side = Nebeneinander
dlg-export-dual-separate = Getrennte Dateien
dlg-export-dual-map = Nur Karte
dlg-export-notes = Notizen:
# The faint text in the empty notes box.
dlg-export-notes-hint = wird mit dem Bild gespeichert (≤120 Zeichen)
# $width and $height: the image size in pixels; $left: how many characters the notes can still take.
dlg-export-output = Ausgabe: { $width } × { $height } px   (noch { $left } Zeichen)
# When the export's shape differs from the window's. $percent: how much more, as 25.
dlg-export-extra-width = Zeigt { $percent } % mehr Breite als das Fenster (nichts wird beschnitten)
# The same, when the export is taller than the window. $percent: how much more, as 25.
dlg-export-extra-height = Zeigt { $percent } % mehr Höhe als das Fenster (nichts wird beschnitten)
dlg-export-tiles-note = In Kacheln gerendert (ohne Größengrenze), in einem Hintergrund-Thread. Die Datei enthält Ansicht + Notizen, sodass sie sich über Datei ▸ Ansicht oder Ort öffnen wieder öffnen lässt.
# A label before the button that picks the folder exports go to.
dlg-export-folder = Ordner:
dlg-export-choose = Wählen…
dlg-export-choose-hover = Den Zielordner für Exporte auswählen
dlg-export-open-after = Nach Abschluss öffnen
dlg-export-open-after-hover = Das fertige Bild an den Bildbetrachter des Systems übergeben, sobald es geschrieben ist.
# $count: the graphics cards in this machine (2 or more).
dlg-export-all-gpus = Alle Grafikkarten verwenden ({ $count })
dlg-export-all-gpus-hover = Das Bild kachelweise auf alle Grafikkarten dieses Rechners verteilen, jede mit eigener Kopie der tiefen Referenz. Identische Karten liefern genau das Bild, das eine allein rendert; verschiedene Modelle zeichnen einige Pixel geringfügig anders, Kachel für Kachel (gemessen: keine sichtbare Naht). Fällt eine Karte aus, gehen ihre Kacheln an die anderen. Bilder, die sich nicht aufteilen lassen (einige Färbemethoden und Formeln), verwenden die Karte dieses Fensters.
# The export's progress.
dlg-export-preparing = Vorbereitung (Referenz wird berechnet)…
dlg-export-saving = Wird gespeichert…
dlg-export-rendering = Wird gerendert…
# $time: the time since the export started, as 8.3s or 1m 04.0s.
dlg-export-elapsed = Verstrichen: { $time }
dlg-export-cancel-hover = Schließen, ohne zu exportieren
# A button, after a save icon.
dlg-export-save-as = Speichern unter…
dlg-export-save-as-hover = Dateinamen und Speicherort wählen
dlg-export-export = Exportieren
dlg-export-export-hover = Rendern und im obigen Ordner speichern (automatisch benannt)

## Notices worded at startup (main.rs)

# A notice shown once after starting, when the saved session comes from a newer version of the app. $saved and $handled: version numbers of the session file's format (1, 2…).
app-state-newer = Diese Sitzung wurde von einer neueren Fractadyne-Version gespeichert (Zustand v{ $saved }; dieser Build verarbeitet v{ $handled }). Manche Einstellungen greifen möglicherweise nicht, und beim Speichern wird sie im Format dieses Builds neu geschrieben.
# The title of the notice that says the accelerated build's arithmetic libraries are found again (its text comes from elsewhere).
app-notice-mpfr-restored-title = Beschleunigte Arithmetik wiederhergestellt
# After Fractadyne restarted itself because the graphics driver reset the GPU.
app-toast-gpu-recovered = Nach dem Zurücksetzen des Grafikgeräts wiederhergestellt — in der Startansicht neu geöffnet, da der vorige Ort zu anspruchsvoll zum Rendern war.

## Short notices (toasts)

# When the autopilot reached a view that could not be rendered and put the last good view back.
app-toast-autopilot-view-restored = Autopilot angehalten: Die Ansicht wurde unbrauchbar und wurde zurückgesetzt (protokolliert)
# $error: the system's reason, as it gives it (often in English).
app-toast-gradients-save-failed = Farbverläufe konnten nicht gespeichert werden: { $error }
# $error: the reason, as the program gives it (often in English).
app-toast-gradients-serialize-failed = Farbverläufe konnten nicht serialisiert werden: { $error }
# $name: the name the user gave the gradient.
app-toast-gradient-saved = Farbverlauf „{ $name }“ gespeichert.
# $error: the reason, as the program gives it (often in English).
app-toast-bookmarks-serialize-failed = Lesezeichen konnten nicht serialisiert werden: { $error }
# $error: the system's reason, as it gives it (often in English).
app-toast-bookmarks-save-failed = Lesezeichen konnten nicht gespeichert werden: { $error }
# Jumping to a named point given by exact expressions. $name: the point's name (in English).
app-toast-coordinates-failed = { $name }: Die Koordinaten ließen sich nicht auswerten
# $zoom: the magnification, already formatted (1,234.5 or 1.2e148).
app-toast-random-location = Zufälliger Ort · { $zoom }×
# Importing a Kalles Fraktaler .kfr location. $error: the reason (in English).
app-toast-import-failed = Import fehlgeschlagen: { $error }
# $zoom: the magnification, already formatted.
app-toast-location-loaded = Ort geladen @ { $zoom }×

## Go to location: why a jump could not be made (one line per field that failed)

# "Re" is the dialog's real-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-re = Re: { $error }
# "Im" is the dialog's imaginary-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-im = Im: { $error }
# The imaginary field was given a complex value. $error: the evaluator's reason (in English).
app-goto-error-im-real = Im: { $error } — den Imaginärteil als reelle Zahl eingeben, ohne das i
# "Zoom" is the dialog's magnification field.
app-goto-error-zoom-empty = Zoom: eine Vergrößerung eingeben, z. B. 1e50 oder 250000
# $zoom: what was typed, in quotes.
app-goto-error-zoom = Zoom: { $zoom } ist nicht lesbar — eine positive Vergrößerung wie 1e50 oder 250000 eingeben
app-goto-error-invalid = Ungültige Eingabe — Koordinaten und Zoom prüfen.

## Finding a minibrot or Misiurewicz point (the M key and the Go to dialog)

app-feature-needs-holomorphic = Die Minibrot-Suche braucht eine holomorphe Familie (Mandelbrot / Multibrot).
app-feature-not-in-julia = Die Minibrot-Suche durchsucht die Mandelbrot-Menge — zum Verwenden den Julia-Modus ausschalten.
app-feature-busy = Es wird bereits nach einem Merkmal gesucht — eines nach dem anderen.
app-feature-searching = Suche nach dem nächsten Minibrot…
app-feature-mandelbrot-only = Die Merkmalsuche gibt es nur für Mandelbrot.
app-feature-no-pair = Innerhalb des Suchbudgets wurde kein Misiurewicz-Punkt gefunden — der Punkt liegt womöglich tiefer, als die Suche von dieser Ansicht aus reicht, oder die Mitte liegt nicht nahe einer Spirale oder eines Verzweigungspunkts.
# The name of what was found, used as $feature in the messages below. $period: the minibrot's period (a whole number).
app-feature-minibrot = Minibrot der Periode { $period }
# $preperiod and $period: the point's two whole numbers (k, p). $octaves: how often the view repeats when zooming in (2 decimals); $twist: the turn per repeat in degrees.
app-feature-misiurewicz-lambda = Misiurewicz ({ $preperiod },{ $period }) — wiederholt sich alle { $octaves } Oktaven, Drehung { $twist }°
# $preperiod and $period: the point's two whole numbers (k, p).
app-feature-misiurewicz = Misiurewicz ({ $preperiod },{ $period })
# Added after the messages below when the fixed iteration count is too low for the new depth. $have: the iterations set now; $typical: what the depth typically needs.
app-feature-iter-note = ⚠ Die Iterationen sind auf { $have } festgelegt; diese Tiefe braucht typischerweise ~{ $typical } — den Wert erhöhen (oder Auto-Iterationen aktivieren), falls die Ansicht flach gerendert wird.
# $feature: what was found (see above). $zoom and $asked: magnifications, already formatted; $octaves: the solver's depth limit.
app-feature-zoomed-capped = Gezoomt auf: { $feature } — { $zoom }×. Die Berechnung ist auf { $octaves } Oktaven begrenzt, daher endete sie dort statt bei { $asked }×.
# $feature: what was found. $zoom: the magnification, already formatted. $seconds: how long the search took (1 decimal).
app-feature-zoomed = Gezoomt auf: { $feature } — { $zoom }× ({ $seconds } s)
# $feature: what was found. $seconds: how long the search took (1 decimal).
app-feature-snapped = Auf das Zentrum gesetzt: { $feature } ({ $seconds } s)
app-feature-no-minibrot = Kein Minibrot-Zentrum nahe der Ansicht gefunden — näher an eines heranzoomen.
# $feature: the point asked for, e.g. Misiurewicz (3,1). $widths: how many view-widths away, already formatted. $octaves: how far to zoom out.
app-feature-just-outside = Ein Punkt vom Typ { $feature } wurde knapp außerhalb dieser Ansicht gefunden — { $widths } Ansichtsbreiten entfernt. Um etwa { $octaves } Oktaven herauszoomen, um ihn ins Bild zu holen, oder eine größere Vorperiode für ein Merkmal in diesem Maßstab setzen.
# $feature: the point asked for. $widths: how many view-widths away, already formatted (may be 2^187). $octaves: how far to zoom out.
app-feature-far = Ein Punkt vom Typ { $feature } wurde gefunden, aber er ist { $widths } Ansichtsbreiten entfernt — dieses Merkmal ist weit GRÖBER als diese Ansicht. Um etwa { $octaves } Oktaven HERAUSzoomen, um es zu erreichen, oder eine größere Vorperiode für ein Merkmal in diesem Maßstab versuchen.
# $feature: the point asked for. $residual: a small number in scientific notation.
app-feature-not-preperiodic = { $feature } passt hier nicht zum Orbit (Residuum { $residual }) — beide Felder leeren, um das Paar aus der Ansicht zu ermitteln.
# $feature: the point asked for. "k/p" are the dialog's two fields.
app-feature-not-converged = Kein Punkt vom Typ { $feature } ist nahe der Ansicht konvergiert — näher heran navigieren oder andere Werte für k/p versuchen.
app-feature-bad-request = Vorperiode und Periode müssen beide positiv sein.

## Share location dialog and location files

app-location-empty = Nichts zu laden (oder Text zu groß).
app-location-not-fractadyne = Kein Fractadyne-Ort.
# $zoom: the magnification, already formatted. $note: what in the text was ignored or repaired (in English).
app-share-loaded-note = Geladen @ { $zoom }× — { $note }.
# $error: the system's reason (often in English).
app-share-save-failed = Speichern fehlgeschlagen: { $error }
app-share-saved = Gespeichert.
app-share-saved-rendering-thumb = Gespeichert. Vorschaubild wird gerendert…
app-share-saved-no-thumb = Gespeichert — ohne Vorschaubild (für diese Ansicht ließ sich keines rendern).
app-share-saved-with-thumb = Gespeichert, mit Vorschaubild.
# $error: the system's reason (often in English).
app-share-saved-thumb-failed = Gespeichert, aber das Vorschaubild ließ sich nicht hinzufügen: { $error }
app-share-loaded-into-box = In das Textfeld geladen — prüfen, dann „Anwenden“.
# Reading a file failed (a location or a palette). $error: the system's reason (often in English).
app-read-failed = Lesen fehlgeschlagen: { $error }
app-share-too-large = Datei zu groß (kein .fdn-Ort?).

## File-dialog filter names (the extensions are added by the code)

app-filter-kfr = Kalles-Fraktaler-Ort
app-filter-fdn = Fractadyne-Ort
app-filter-map = Fractint-/KF-Palette
app-filter-ugr = Ultra-Fractal-Farbverlauf
app-filter-ase = Adobe Swatch Exchange
app-filter-ggr = GIMP-Farbverlauf
app-filter-all = Alle Dateien

## Importing and saving palette files (shown under the gradient editor)

# The reason a file was refused before reading it.
app-gradient-map-too-large = viel zu groß für eine .map-Palette
app-gradient-ugr-too-large = viel zu groß für eine .ugr-Palettendatei
app-gradient-ase-too-large = viel zu groß für eine .ase-Farbfeldliste
app-gradient-ggr-too-large = viel zu groß für einen .ggr-Farbverlauf
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported-vga = { $count } Einträge aus { $file } importiert – eine 6-Bit-VGA-Tabelle (Weiß ist 252, nicht 255), so belassen, wie sie geschrieben ist, damit sie der Ausgabe von Fractint selbst entspricht.
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported = { $count } Einträge aus { $file } importiert, als harte Bänder.
# $error: the reader's reason (in English).
app-gradient-map-invalid = Keine .map-Palette: { $error }
# $name: the gradient's name in the file; $file: the file's name.
app-gradient-ugr-imported = „{ $name }“ aus { $file } importiert.
# $file: the file's name; $count: how many gradients it holds.
app-gradient-ugr-pick = { $file } enthält { $count } Farbverläufe — unten einen auswählen.
# $error: the reader's reason (in English).
app-gradient-ugr-invalid = Keine .ugr-Palettendatei: { $error }
# $used: the swatches kept; $total: the swatches in the file.
app-gradient-ase-imported-some = { $used } von { $total } Farbfeldern importiert, gleichmäßig verteilt.
# $count: the swatches in the file.
app-gradient-ase-imported = { $count } Farbfelder importiert, gleichmäßig verteilt und überblendet.
# $error: the reader's reason (in English).
app-gradient-ase-invalid = Keine .ase-Farbfeldliste: { $error }
app-gradient-nothing-to-save = Noch kein eigener Farbverlauf zum Speichern.
# $file: the saved file's name.
app-gradient-ggr-saved = „{ $file }“ gespeichert — Mittelpunkte, Überblendkurven und Farbräume vollständig erhalten.
# $file: the saved file's name; $count: the segments whose curve could not be kept.
app-gradient-ggr-saved-lossy = „{ $file }“ gespeichert — aber { $count } Segment(e) verwenden eine Bézierkurve, die das .ggr-Format nicht ausdrücken kann. Diese wurden als gerade Überblendungen geschrieben; die Kopie in der Bibliothek behält die Kurven.
# $path: the file; $error: the system's reason (often in English).
app-gradient-write-failed = { $path } konnte nicht geschrieben werden: { $error }
# $name: the gradient's name; $count: its segments.
app-gradient-ggr-imported = „{ $name }“ importiert – { $count } Segmente, mit ihren Mittelpunkten, Überblendkurven und Farbräumen.
# $error: the reader's reason (in English).
app-gradient-ggr-invalid = Kein .ggr-Farbverlauf: { $error }

## The eyedropper's banner across the top of the screen while picking a colour

# $hex: the colour under the pointer, as #rrggbb.
app-eyedropper-banner-color = Farbe aufnehmen  ·  { $hex }  ·  Klick übernimmt sie, Esc bricht ab
app-eyedropper-banner = Farbe aufnehmen  ·  Klick übernimmt sie, Esc bricht ab

## The gradient editor window

app-gradient-title = Farbverlaufseditor
# Before the two view buttons, Bar and Ring.
app-gradient-view = Ansicht
app-gradient-view-bar = Balken
app-gradient-view-bar-hover = Ein gerader Streifen — am besten, um Farbstopps genau zu setzen
app-gradient-view-ring = Ring
app-gradient-view-ring-hover = Ein Ring, denn die Palette wird ZYKLISCH durchlaufen: Die Verbindungsstelle oben ist die Naht, die der Renderer bei jedem Umlauf überquert, und ein Farbverlauf, der auf einem Balken gut aussieht, kann dort eine harte Kante haben.
app-gradient-add-lane-hover = Irgendwo auf diese Linie klicken, um dort einen Farbstopp hinzuzufügen. Das + fügt einen in der breitesten Lücke hinzu.
app-gradient-strip-hover = Eine Markierung ziehen, um ihren Farbstopp zu verschieben · den Streifen selbst ziehen, um den ganzen Farbverlauf zu DREHEN, mit Umbruch an der Naht · Doppelklick fügt einen Farbstopp hinzu, Rechtsklick auf eine Markierung entfernt ihn · Pfeiltasten verschieben die Auswahl (Shift für feine Schritte).
app-gradient-remove-selected-hover = Den ausgewählten Farbstopp entfernen
app-gradient-ring-hover = Den Ring selbst ziehen, um den ganzen Farbverlauf zu DREHEN — was die Naht passiert, kommt auf der anderen Seite wieder herum · eine Markierung ziehen, um ihren Farbstopp zu verschieben · auf die äußere Spur klicken, um dort einen Farbstopp hinzuzufügen, oder auf ⊕ in der Mitte, um einen in der breitesten Lücke hinzuzufügen · ⊖ entfernt den ausgewählten Farbstopp. Die hohle Markierung oben ist die Naht, an der eine zyklische Palette umbricht; sie bleibt an ihrem Platz, weil sie die Verbindungsstelle IST, und die Farben drehen sich unter ihr.
app-gradient-empty = Noch kein eigener Farbverlauf — „Vorlage kopieren…“, ein Import oder ein gespeicherter Farbverlauf unten beginnt einen, und seine Farbstopps und Kurven pro Segment erscheinen hier.
# $count: the entries of an imported palette, more than the editor shows stop by stop.
app-gradient-too-many = { $count } importierte Einträge – zu viele, um sie Stopp für Stopp zu bearbeiten. Die Vorschau oben verwenden oder mit „Vorlage kopieren“ neu beginnen.
# The selected segment's heading. $index: its number; $total: how many there are; $left and $right: where it starts and ends, 0.000–1.000.
app-gradient-segment-heading = Segment { $index } von { $total }  ·  { $left } → { $right }
# A label before the list of blend curves.
app-gradient-curve = Kurve
app-gradient-curve-hover = Wie schnell sich die Farbe über dieses Segment bewegt. „Gekrümmt“ wird durch den Mittelpunkt geformt — bei Mittelpunkt 0.50 ist sie genau linear.
# The blend curves (GIMP's kinds), in a narrow list: keep them short.
app-gradient-blend-linear = Linear
app-gradient-blend-curved = Gekrümmt
app-gradient-blend-sine = Sinus
app-gradient-blend-sphere-increasing = Kugel ↑
app-gradient-blend-sphere-decreasing = Kugel ↓
app-gradient-blend-bezier = Bézier
# Hover on Bézier in the list of blend curves.
app-gradient-bezier-exact-hover = Zwei Kontrollpunkte, die sich in beide Richtungen ziehen lassen. Diese Kurve ist bereits kubisch, daher ändert die Umwandlung nichts.
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-visible-hover = Zwei Kontrollpunkte, die sich in beide Richtungen ziehen lassen. Die Umwandlung NÄHERT die aktuelle Kurve nur AN — diese bis auf { $error }, was deutlich sichtbar ist.
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-barely-hover = Zwei Kontrollpunkte, die sich in beide Richtungen ziehen lassen. Die Umwandlung NÄHERT die aktuelle Kurve nur AN — diese bis auf { $error }, was kaum sichtbar ist.
# A label before the list of colour spaces (RGB, HSV).
app-gradient-space = Farbraum
# A label before the Bézier presets beside it.
app-gradient-ease = Easing
app-gradient-ease-hover = Vorlagen setzen die beiden Kontrollpunkte; für alles andere die Punkte auf der Kurve ziehen. Eine Bézierkurve hat keinen Mittelpunkt — ihre Griffe bestimmen, wo sie die Hälfte erreicht.
# Bézier presets, as in CSS easing: short buttons.
app-gradient-ease-linear = Linear
app-gradient-ease-in = In
app-gradient-ease-out = Out
app-gradient-ease-in-out = In-out
app-gradient-midpoint = Mittelpunkt
app-gradient-midpoint-hover = Wo die Überblendung die Hälfte erreicht, als Bruchteil dieses Segments. 0.50 ist mittig.
# A button that puts the midpoint back in the middle of the segment.
app-gradient-centre = Mitte
app-gradient-centre-hover = Zurück auf 0.50
app-gradient-no-hue = ⚠ ein Ende dieses Segments hat keinen Farbton
app-gradient-no-hue-hover = Ein Ende ist Schwarz, Weiß oder Grau, daher beginnt der Übergang bei Rot und durchläuft das ganze Farbrad. Das ist zulässig und manchmal gewollt — aber deshalb kann die Mitte eine Farbe haben, die keines der Enden enthält. Der Streifen unter der Kurve zeigt, was geschieht.
app-gradient-segment-start-hover = Farbe am ANFANG dieses Segments
app-gradient-segment-end-hover = Farbe am ENDE dieses Segments
app-gradient-bytes-hover = Rot, Grün und Blau als 0–255
app-gradient-floats-hover = Dieselbe Farbe als 0–1, so wie der Renderer sie speichert
app-gradient-canvas-bezier-hover = Die Überblendkurve des ausgewählten Segments. Einen der Kontrollpunkte ziehen — in x UND y —, um sie umzuformen. Die blassen Linien sind 0 und 1; eine Kurve darf über sie hinausgehen, und begrenzt wird dann die Farbe, nicht die Kurve.
app-gradient-canvas-hover = Die Überblendkurve des ausgewählten Segments: wie schnell die Farbe vom linken zum rechten Farbstopp wandert. Den Ring ziehen, um den Mittelpunkt zu verschieben, oder die Kurve auf Bézier umstellen, um einen Kontrollpunkt zu erhalten, der sich in beide Richtungen bewegt.
# The selected stop's heading. $index: its number; $total: how many there are.
app-gradient-stop-heading = Farbstopp { $index } von { $total }
app-gradient-stop-color-hover = Die Farbe an diesem Farbstopp — auf das Farbfeld klicken, um sie zu wählen, oder daneben einen Hex-Wert eingeben
app-gradient-stop-bytes-hover = Rot, Grün und Blau als 0–255 — die Bytes, in denen diese Farbe geschrieben wird
# The eyedropper button.
app-gradient-pick = Aufnehmen
app-gradient-pick-hover = Pipette — eine Farbe von einer beliebigen Stelle des Bildschirms aufnehmen. Den Zeiger auf die gewünschte Farbe bewegen und klicken; Esc bricht ab.
# Written before the stop's position (0.000–1.000) in its number field: a short abbreviation of "position".
app-gradient-stop-pos = Pos.
app-gradient-nudge-hover = Diesen Farbstopp schrittweise verschieben (Shift gedrückt halten für feinere Schritte)
app-gradient-remove = Entfernen
app-gradient-remove-hover = Diesen Farbstopp entfernen (oder auf das ⊖ darunter klicken)
app-gradient-add-stop = Farbstopp hinzufügen
app-gradient-add-stop-hover = Das breiteste Segment teilen (oder auf die Linie über dem Farbverlauf klicken)
# A menu of the built-in palettes, to start from a copy of one.
app-gradient-copy-preset = Vorlage kopieren…
app-gradient-paste = Einfügen…
app-gradient-paste-hover = Eine Palette aus Hex-Farben oder RGB-Tripeln (0–255) importieren
app-gradient-save-ggr = .ggr speichern…
app-gradient-save-ggr-hover = Diesen Farbverlauf in eine GIMP-.ggr-Datei schreiben — lesbar von GIMP, Krita und Inkscape sowie von diesem Editor
# A menu of the palette file formats (.map, .ugr, .ggr, .ase).
app-gradient-import = Importieren
app-gradient-import-map-hover = Eine .map-Palettendatei von Fractint / Kalles Fraktaler laden (R-G-B-Zeilen, eine pro Eintrag)
app-gradient-import-ugr-hover = Eine .ugr-Farbverlaufsdatei von Ultra Fractal laden (eine .ugr enthält meist viele Farbverläufe — einer wird ausgewählt)
app-gradient-import-ggr-hover = Einen GIMP-.ggr-Farbverlauf laden, mit Mittelpunkten, Überblendkurven und Farbräumen vollständig
app-gradient-import-ase-hover = Eine Adobe-Farbfeldliste (.ase) laden. Farbfelder haben keine Positionen, daher werden sie gleichmäßig verteilt und überblendet.
# The gradient's name, for the library of saved gradients.
app-gradient-name = Name
# Shown greyed in the empty name field.
app-gradient-name-hint = mein Farbverlauf
# The save button, when a gradient of that name is already saved.
app-gradient-update = Aktualisieren
app-gradient-save = Speichern
app-gradient-save-hover = Diesen Farbverlauf in der Bibliothek speichern, mit Kurven und Farbräumen vollständig
app-gradient-save-needs-name-hover = Dem Farbverlauf zuerst einen Namen geben
# A menu of the saved gradients. $count: how many there are.
app-gradient-saved-menu = Gespeichert ({ $count })
app-gradient-nothing-saved = Noch nichts gespeichert
app-gradient-delete-saved-hover = Diesen gespeicherten Farbverlauf löschen
# $count: the gradient's segments.
app-gradient-rich = Reichhaltiger Farbverlauf: { $count } Segmente tragen Mittelpunkte, Überblendkurven oder Farbräume — mehr, als eine einfache Liste von Farbstopps fassen kann.
app-gradient-convert = In bearbeitbare Farbstopps umwandeln
app-gradient-convert-hover = Durch eine einfache Liste von Farbstopps ersetzen. Dies VERWIRFT die Mittelpunkte, Überblendkurven und Farbtonläufe, die eine solche Liste nicht fassen kann.
# Above the list of gradients in an Ultra Fractal file. $count: how many; $file: the file's name.
app-gradient-ugr-list = { $count } Farbverläufe in { $file }
app-gradient-close-list = Liste schließen
app-gradient-seamless = Nahtlos (Ende gleich Anfang)
app-gradient-seamless-hover = Eine Palette wird zyklisch durchlaufen, daher liegen Position 1 und Position 0 auf dem Bildschirm nebeneinander. Ist dies aktiviert, wird die Farbe des letzten Farbstopps gleich der des ersten gehalten — die Ringansicht zeigt die Verbindungsstelle oben.
app-gradient-ends-differ = ⚠ Enden verschieden
app-gradient-ends-differ-hover = Die Farbe am Ende des Farbverlaufs ist nicht die am Anfang, daher zeigt eine zyklische Palette an der Umbruchstelle eine harte Kante.
app-gradient-hard-bands = Harte Bänder (keine Überblendung zwischen Einträgen)
app-gradient-hard-bands-hover = Fractint-/.map-Semantik: Jeder Eintrag ist eine flache Farbe. Ausgeschaltet werden sie zu einem glatten Farbverlauf überblendet.
app-gradient-paste-intro = Hex-Farben (#ff8800) oder Tripel 0–255 (255 136 0) einfügen, getrennt durch Kommas, Leerzeichen oder Zeilenumbrüche:
# Applies the pasted colours.
app-gradient-apply = Anwenden
# Empties the paste box.
app-gradient-clear = Leeren
# $used: the colours kept; $total: the colours pasted; $limit: the most that are kept.
app-gradient-paste-imported-some = { $used } von { $total } Farben importiert, gleichmäßig aus der Liste ausgewählt ({ $limit } ist die Obergrenze).
# $count: the colours pasted.
app-gradient-paste-imported = { $count } Farben importiert.
# $error: why the pasted text could not be read (in English).
app-gradient-paste-unreadable = Das ließ sich nicht lesen: { $error }
# A hint at the bottom of the editor. $count: the palette's entries.
app-gradient-bands-hint = { $count } Bänder · in Dateireihenfolge übernommen; Positionen werden ignoriert, solange Bänder aktiv sind.
# A hint at the bottom of the editor, in the ring view. $count: the gradient's stops.
app-gradient-ring-hint = { $count } Farbstopps · auf die äußere Spur klicken, um dort einen hinzuzufügen · ⊕ fügt in der breitesten Lücke hinzu, ⊖ entfernt den ausgewählten · eine Markierung ziehen, um sie zu verschieben · die Markierung oben ist die Naht
# A hint at the bottom of the editor, in the bar view. $count: the gradient's stops.
app-gradient-bar-hint = { $count } Farbstopps · auf die Linie über dem Farbverlauf klicken, um dort einen hinzuzufügen · eine Markierung ziehen, um sie zu verschieben · ⊖ oder Rechtsklick entfernt
app-gradient-cancel = Abbrechen
app-gradient-cancel-hover = Alle Änderungen seit dem Öffnen dieses Fensters verwerfen und es schließen
app-gradient-ok = OK
app-gradient-ok-hover = Diese Änderungen behalten und den Editor schließen

## Shared

# The button that closes the Help window and the "Faster deep zoom" window.
app-close = Schließen

## The "Faster deep zoom" window (Help menu): the optional accelerated build

app-accel-title = Schnellerer Tiefenzoom
app-accel-active-heading = Der beschleunigte Build läuft
app-accel-active-body = Referenzorbits für den Tiefenzoom werden mit MPFR/GMP berechnet, was bei diesem Schritt 2,5- bis 6,4-mal schneller ist als der Standard-Build – die Pause, bevor sich eine tiefe Ansicht aufzubauen beginnt.
# $status: which arithmetic library is in use and its version (in English).
app-accel-in-use = Verwendete Arithmetik: { $status }
app-accel-fallback-heading = Beschleunigter Build — verwendet die integrierte Arithmetik
app-accel-fallback-body = Dies ist der beschleunigte (MPFR-)Build, aber seine Mathematikbibliotheken wurden nicht gefunden, daher verwendet er die integrierte Arithmetik (astro-float). Die Bilder sind identisch; nur die Pause, bevor sich eine tiefe Ansicht aufbaut, ist länger.
# Followed by the folder the program runs from, on its own line.
app-accel-fallback-dlls = Für den schnelleren Weg libmpfr-6.dll und libgmp-10.dll (sowie libgcc_s_seh-1.dll, libwinpthread-1.dll) neben fractadyne.exe legen und dann neu starten:
app-accel-warning-hidden = Die Startwarnung dazu ist ausgeblendet.
app-accel-show-again = Wieder anzeigen
app-accel-offer-heading = Ein optionaler Build ist in der Tiefe 2,5- bis 6,4-mal schneller
app-accel-offer-body = Der Tiefenzoom verbringt einen Großteil seiner Zeit damit, Referenzorbits auf der CPU zu berechnen – die Pause, bevor sich eine tiefe Ansicht aufzubauen beginnt. Ein optionaler Build erledigt das stattdessen mit den Bibliotheken MPFR/GMP, was 2,5- bis 6,4-mal schneller ist, und umso mehr, je tiefer es geht. Alles andere ist gleich.
app-accel-offer-identical = Die Bilder sind BYTEGLEICH – es ist dieselbe Mathematik, berechnet von einer schnelleren Bibliothek, geprüft über jede Fraktalformel und den gesamten Tiefenzoom-Vergleichskorpus.
app-accel-offer-shared = Einstellungen, gespeicherte Sitzung und Orte werden von beiden Builds gemeinsam genutzt, sodass sich frei wechseln lässt und nichts importiert werden muss.
# Shown on Windows.
app-accel-separate-windows = Er ist ein separater Download, weil sich die verwendeten Bibliotheken nicht mit dem Compiler des Standard-Windows-Builds bauen lassen und sie einer anderen Lizenz (GNU LGPL v3) unterliegen als Fractadyne selbst.
# Shown on other systems.
app-accel-separate = Er ist ein separater Download, weil die verwendeten Bibliotheken einer anderen Lizenz (GNU LGPL v3) unterliegen als Fractadyne selbst.
app-accel-download = Für diese Version herunterladen
app-accel-all-releases = Alle Versionen
app-accel-all-releases-hover = Liefert der direkte Link einen 404-Fehler, gibt es für diese Version noch keinen beschleunigten Build – hier den nächstgelegenen wählen.
# Shown on Windows.
app-accel-install-windows = Entpacken und fractadyne.exe aus diesem Ordner starten; die .dll-Dateien daneben lassen.
# Shown on Linux.
app-accel-install-linux = Entpacken und ./fractadyne aus diesem Ordner starten. Er verwendet GMP und MPFR des Systems (libgmp10 / libmpfr6), die fast jeder Desktop bereits hat – siehe README-ACCELERATED.txt im Paket.
# Shown on other systems.
app-accel-install-other = Für diese Plattform wird noch kein beschleunigtes Paket gebaut; es lässt sich mit --features fractadyne-core/rug aus dem Quellcode bauen.

## The Help window's frame (its sections are under help-)

app-help-title = Fractadyne-Hilfe

## Checking for updates

app-toast-up-to-date = Dies ist die neueste Version.
# $error: why the check failed (in English).
app-toast-update-failed = Update-Prüfung fehlgeschlagen: { $error }
# The window that offers a newer version.
app-update-title = Update verfügbar
# $version: the new version (0.3.0-beta.20). $channel: the token "stable" or "beta", in English; a translation can choose its words by it ({ $channel -> [beta] … *[other] … }).
app-update-version =
    { $channel ->
        [beta] Fractadyne { $version } (Beta) ist verfügbar
       *[other] Fractadyne { $version } (stabil) ist verfügbar
    }
# $current: the version running now. $track: the update track the user follows (its name in the language in use).
app-update-running = Aktuell läuft { $current }  ·  { $track }
app-update-download-intro = Die neueste Version von GitHub herunterladen:
app-update-download = Von GitHub herunterladen
app-update-later = Später erinnern

## The second graphics card (shown in its window, after "In use:" or "Could not start:")

# The second device is the same card as the window's. $name: the card's name.
app-worker-same-card = dieselbe Karte, { $name }
# Why the second device could not start.
app-worker-thread-stopped = der Geräte-Thread wurde beendet

## Export status (shown in the Export image window)

app-export-gpu-unavailable = GPU nicht verfügbar
app-export-reference-aborted = Export fehlgeschlagen: Referenzberechnung abgebrochen.

## The performance panel (View ▸ Performance panel)
## Its row labels sit in a monospace column about 14 characters wide: keep them short (a longer label pushes its value to the right).

app-perf-fps = FPS
app-perf-fps-hover = Pro Sekunde angezeigte Bilder, aus dem geglätteten Abstand zwischen den Bildern. Solange eine Ansicht zur Ruhe kommt, zeigt dies stattdessen den Verfeinerungsfortschritt – eine Ansicht, die zur Ruhe kommt, läuft nicht mit der Rate, die eine Zahl hier nahelegen würde.
# What the FPS row shows while a view is settling. $done and $total: tiles; $eta: the time left, as m:ss.
app-perf-refining-tiles = verfeinert { $done }/{ $total } (~{ $eta })
app-perf-refining-arming = verfeinert (Vorbereitung)
# $percent: how much is done, with one decimal.
app-perf-refining-percent = verfeinert { $percent } %
app-perf-building-reference = Referenz wird berechnet
app-perf-idle = Leerlauf
app-perf-frame = Bild
app-perf-frame-hover = Geglättete Echtzeit zwischen den Bildern – nicht die Summe der Zeilen darunter. Sie enthält das Warten auf die Anzeige, daher spiegelt sie bei einer ruhenden, untätigen Ansicht den Takt des Neuzeichnens wider statt irgendeiner Arbeit.
app-perf-cpu = cpu
app-perf-cpu-hover = Zeit des Haupt-Threads innerhalb eines Updates: Parametereinrichtung und Verwaltung des Referenz-Cache. Diese Zeile schlägt aus, wenn eine Referenz neu berechnet wird, und ein Ausschlag hier ist das Ruckeln, das man spürt.
app-perf-gpu-idle = gpu/Leerlauf
app-perf-gpu-idle-hover = Bildzeit minus CPU-Zeit des Haupt-Threads. Das ist GPU-Arbeit, solange die Ansicht rendert, aber bei einer ruhenden Ansicht meist untätiges Warten – der Name nennt beides, weil die Anzeige die beiden allein aus dieser Subtraktion nicht unterscheiden kann.
app-perf-timing = Zeitmessung
# How frame cost is measured, the timing row's value. TIMESTAMP_QUERY is a name.
app-perf-timing-starved = Echtzeit (Zeitstempel bleiben aus)
app-perf-timing-gpu = GPU-Zeitstempel
app-perf-timing-no-query = Echtzeit (kein TIMESTAMP_QUERY)
app-perf-timing-hover = Wie die Bildkosten gemessen werden. GPU-Zeitstempel sind die genaue Quelle; der Rückgriff auf die Echtzeit greift, wenn keine Messwerte mehr ankommen, und ein nach Echtzeit bemessenes Budget hat tiefe Bilder wiederholt falsch dimensioniert – Rot bedeutet hier: „Die Sicherheitsmargen sind gerade nur Schätzungen.“
app-perf-mode = Modus
app-perf-mode-hover = Der Rechenweg, mit dem das letzte Bild gerendert wurde: direkt (kein Referenzorbit), df32-Störungsrechnung oder floatexp – der Pfad mit erweitertem Wertebereich, der ab etwa 1e28x verwendet wird und als einziger BLA ausführt.
app-perf-eff-iter = eff. Iter.
app-perf-eff-iter-hover = Iterationen, die das letzte Bild tatsächlich verwendet hat, nach Tiefenskalierung und einer etwaigen Begrenzung während der Bewegung – nicht die Grundeinstellung unter Qualität, die nur der Ausgangspunkt ist.
app-perf-precision = Genauigkeit
app-perf-precision-hover = Mantissenbits, mit denen der Referenzorbit berechnet wird. Sie wachsen mit der Tiefe; 64 Bit ist die Untergrenze bei geringer Tiefe, keine Grenze, an die man stößt.
app-perf-orbit-len = Orbitlänge
# $cap: the most orbit samples this GPU holds, already formatted, or the text of app-perf-uncapped.
app-perf-orbit-len-hover = Punkte des Referenzorbits. Diese GPU fasst höchstens { $cap } – sich dem zu nähern ist die praktische Tiefengrenze: Darüber hinaus kann die Live-Ansicht tiefere Orte, die einen längeren Orbit brauchen, nicht auflösen.
app-perf-uncapped = (unbegrenzt)
app-perf-sa-skip = SA-Sprung
app-perf-sa-skip-hover = Iterationen, die die Reihennäherung übersprungen hat, bevor die Arbeit pro Pixel begann. Höher ist schneller und ändert nichts am Bild; die Zeile fehlt, wenn nichts übersprungen wurde.
app-perf-aa = aa
app-perf-aa-hover = Supersampling: Jedes Pixel wird aus so vielen Abtastwerten pro Achse gerendert, also kostet 2x die vierfache Arbeit.
app-perf-dual = Doppel
app-perf-dual-hover = Ob die geteilte Mandelbrot-/Julia-Ansicht läuft. Ist sie aktiv, decken alle Kosten in dieser Anzeige BEIDE Hälften ab.
app-perf-zoom = Zoom
app-perf-zoom-hover = Vergrößerung relativ zur Gesamtansicht. Der Rechenmodus oben wechselt bei etwa 1e4x und erneut bei etwa 1e28x.
app-perf-iterate = iterieren
# $target and $lethal: times in milliseconds, already formatted.
app-perf-iterate-hover = Gemessene GPU-Zeit des letzten Iterations-Dispatch. Die Steuerung zielt auf { $target } ms; anhaltende Werte nahe { $lethal } ms sind der Bereich, in dem diese Hardwareklasse das Gerät verloren hat.
app-perf-steps = Schritte/s
app-perf-steps-hover = Iterationsschritte pro Sekunde auf der GPU, in Milliarden. Ansichten mit viel Innerem laufen pro Schritt langsamer, als die Zahl allein vermuten lässt, daher ist dies ein Durchsatzwert und keine Punktzahl.
app-perf-budget = Budget
# After the budget's value while it is still converging; the hover below quotes it.
app-perf-budget-settling = (pendelt ein)
app-perf-budget-hover = Adaptives Schrittbudget pro Dispatch, der Schutz der Steuerung gegen ein Bild, das so lange dauert, dass der Treiber die GPU zurücksetzt. „pendelt ein“ bedeutet, dass es sich aus spärlichen Messwerten noch annähert und noch keine verlässliche Obergrenze ist.
app-perf-rss = rss
# $mb and $peak: memory in megabytes, already formatted.
app-perf-rss-value = { $mb } MB (Spitze { $peak })
app-perf-rss-hover = Residenter Speicher des Prozesses und der Höchstwert in dieser Sitzung. Tiefe Referenzberechnungen haben in der Praxis Spitzen über 2 GB erreicht, ohne dass irgendetwas anderes auf dem Bildschirm darauf hinwies; gelb ab drei Vierteln des System-RAM.
app-perf-gpu-est = gpu gesch.
app-perf-gpu-est-hover = Geschätzter Speicher auf der GPU: Referenzorbits, BLA-Bäume, Iterationstexturen, das Chunk-Status-Ping-Pong während einer Verfeinerung und der Schnappschuss des Present-Gates. Zusammengesetzt aus bekannten Allokationsgrößen; der tatsächliche Wert des Treibers ist nicht portabel auslesbar.
app-perf-julia-re = julia c.re
app-perf-julia-re-hover = Realteil des Parameters c, den die Julia-Hälfte rendert – der Punkt unter dem Mauszeiger, sofern nicht festgesetzt.
app-perf-julia-im = julia c.im
app-perf-julia-im-hover = Imaginärteil des Parameters c, den die Julia-Hälfte rendert.
app-perf-julia-c = julia c
# The julia c row's value when the parameter is held fixed.
app-perf-pinned = festgesetzt
app-perf-pinned-hover = Der Parameter wird an einem festen Punkt gehalten, daher ändert eine Bewegung des Mauszeigers die Julia-Hälfte nicht mehr.
app-perf-c-panel = c/Hälfte
app-perf-c-panel-hover = Breite des c-Raums, den die ganze Mandelbrot-Hälfte überspannt. Liegt sie weit unter einem Julia-Pixel, ändert das Überfahren zwar c, aber die Julia-Menge sieht unverändert aus – bei tiefem Zoom zu erwarten, kein Einfrieren.
app-perf-ref-recompute = Ref.-Neuber.
app-perf-ref-recompute-hover = Dauer der letzten Neuberechnung des Referenzorbits. Das ist die Pause, bevor sich eine tiefe Ansicht aufzubauen beginnt, und der Schritt, den der optionale beschleunigte Build beschleunigt.
app-perf-recompute-rate = Neuber./s
app-perf-recompute-rate-hover = Pro Sekunde EINGESETZTE Referenzorbits – solche, deren Ergebnis tatsächlich verwendet wurde. Mit der Zeile darunter vergleichen: Der Unterschied ist die verworfene Arbeit.
app-perf-builds-rate = Ref.-Berech./s
app-perf-builds-rate-hover = Pro Sekunde GESTARTETE Referenzberechnungen, einschließlich solcher, deren Ergebnis verworfen wird – ein Vorausplatz, den der Tauchgang nie erreicht, oder eine abgelehnte Erweiterung. Sie kosten volle CPU-Leistung und erscheinen nirgends sonst: Eine Prefetch-Schleife lief einmal sechs Minuten lang mit etwa 400 Berechnungen pro Sekunde, während Neuber./s 2 anzeigte. Einstellige Werte sind normal; eine große Lücke zwischen diesen beiden Zeilen bedeutet, dass Arbeit weggeworfen wird.
app-perf-recompute-total = Neuber. ges.
app-perf-recompute-total-hover = Neuberechnungen der Referenz seit dem Start. Nützlich als Rate über ein bekanntes Intervall; eine Zahl, die steigt, während sich nichts bewegt, ist das Signal, dem nachzugehen sich lohnt.

## Help: the contents list (Help & reference…, F1)

# Each section's name in the contents list (about 165 px wide: keep them short), and its heading.
help-section-overview = Überblick
help-section-navigation = Navigation
help-section-expressions = Koordinatenausdrücke
help-section-options = Färbung & Optionen
help-section-fractals = Fraktale
help-section-how-it-works = Funktionsweise
help-section-command-line = Befehlszeile
help-section-shortcuts = Tastenkürzel
help-section-hardware = Empfohlene Hardware
# Credits to the prior work Fractadyne builds on.
help-section-acknowledgments = Danksagungen
# The third-party license notices.
help-section-licenses = Lizenzen
help-section-about = Über

## Help ▸ Overview

help-overview-intro = Fractadyne ist ein nativer Fraktal-Explorer, gebaut für extrem tiefe Zooms und Geschwindigkeit. Er zeichnet „Fluchtzeit-Fraktale“ — Bilder, die entstehen, indem an jedem Pixel eine einfache Formel immer wieder angewandt wird.
help-overview-what-is = Was ist ein Fluchtzeit-Fraktal?
help-overview-what-is-text = Für jedes Pixel wendet das Programm eine Formel wie z → z² + c immer wieder an, beginnend bei null. Bleibt der laufende Wert für immer klein, gehört das Pixel zur Menge und wird dunkel gezeichnet. Wächst er irgendwann über alle Grenzen („entkommt“), liegt das Pixel außerhalb der Menge, und seine Farbe hält fest, wie viele Schritte das dauerte. Die unendlich detailreiche Grenze zwischen „bleibt“ und „entkommt“ ist das Fraktal.
help-overview-can-do = Was möglich ist
help-overview-can-pan = Praktisch ohne Grenze verschieben und zoomen (die Position ist in jeder Tiefe exakt).
help-overview-can-families = Zwischen 25 eingebauten Fraktalfamilien (zehn, fünf davon auch in höheren Potenzen), eigenen Formeln, Life und L-Systemen wechseln und jede Fluchtzeit-Familie außer Newton als Julia-Menge betrachten.
help-overview-can-recolor = Mit vorgefertigten oder eigenen Farbverläufen und mehreren Färbemethoden umfärben.
help-overview-can-lighting = 3D-Reliefbeleuchtung und leuchtende Randkonturen hinzufügen.
help-overview-can-minibrots = Auf Minibrots einrasten, Stellen als Lesezeichen speichern und hochauflösende Bilder exportieren.
help-overview-can-autopilot = Den Auto-Zoom-Autopiloten selbstständig in Richtung Details tauchen lassen.
help-overview-can-share = Jede Stelle als kleines „.fdn“-Textstück oder als Datei teilen.
help-overview-can-tours = Skriptgesteuerte Touren und einen Hardware-Benchmark ausführen.
help-overview-first-steps = Erste Schritte
# "Seahorse Valley" is a place in the Navigate menu: use its name as that menu shows it. A and F1 are keys.
help-overview-first-steps-text = Das Menü „Navigation“ öffnen und „Seahorse Valley“ wählen, dann am Mausrad drehen, um hineinzuzoomen. Ziehen verschiebt die Ansicht. Oder einfach A drücken und den Autopiloten tauchen lassen. F1 führt jederzeit zu dieser Hilfe zurück.

## Help ▸ Coordinate expressions
## The expressions, numbers and function names here are typed as shown in every language:
## keep them exactly as written. Menu items in quotation marks are the menus' own labels.

help-expr-intro = Überall, wo Fractadyne eine Koordinate annimmt, lässt sich statt einer Dezimalzahl ein Ausdruck eingeben: in den Feldern Re / Im von Navigation → „Zu Ort springen…“ (und in jedem Feld seines Polarmodus), in den Zeilen center_re / center_im eines .fdn-Orts, bei re / im eines Tour-Skripts und mit --center in der Befehlszeile. Der Ausdruck wird mit so vielen Stellen ausgewertet, wie der Zoom braucht, und der Dialog „Zu Ort springen“ bewahrt ihn mit der Ansicht auf — so bleibt ein als 1/3 oder als -0.5 + 0.25*cos(pi/4) eingegebener Punkt exakt auf diesem Punkt, wie tief auch immer gezoomt wird, während eine Dezimalzahl abdriften würde, sobald ihre Stellen erschöpft sind.
help-expr-examples = Beispiele
help-expr-ex-third = Ein exaktes Drittel. Beim Hineinzoomen erscheinen weitere Stellen.
help-expr-ex-neck = Der Hals des Seahorse Valley — der Ansatzpunkt der Knospe der Periode 2.
help-expr-ex-circle = Ein Punkt auf dem Kreis mit Radius 1/4 um -0.5 (der Polarmodus setzt dies selbst zusammen).
help-expr-ex-bulb = Re des Ansatzpunkts der Knospe der Periode 3; sein Im ist sin(2*pi/3)/2 - sin(4*pi/3)/4. Navigation → „Exakte Punkte (Ausdrücke)“ enthält diese bereits.
help-expr-ex-complex = Ein ganzer komplexer Wert im Re-Feld füllt Re und Im.
help-expr-ex-combine = Funktionen und Konstanten lassen sich frei kombinieren.
help-expr-ex-powers = Potenzen: 2^-20 ist 1/1048576. -2^2 ist -4 — das Minus wird nach der Potenz angewandt.
help-expr-ex-root = -2, die reelle Kubikwurzel. Gerade Wurzeln einer negativen Zahl werden abgelehnt; sqrt(-1) ergibt i.
help-expr-ex-scientific = Wissenschaftliche Schreibweise funktioniert auch innerhalb von Ausdrücken.
help-expr-numbers = Zahlen und Operatoren
# The decimal point in an expression is '.' in every language: keep that rule as it is.
help-expr-numbers-decimals = Dezimalzahlen mit optionalem Exponenten: 0.25, -1.5e-30, .5. Nur Ziffern — keine Leerzeichen oder Tausendertrennzeichen innerhalb einer Zahl, und das Dezimalzeichen ist immer der Punkt '.'.
help-expr-numbers-imaginary = Ein angehängtes i ergibt eine imaginäre Zahl (0.25i), oder i allein verwenden: 3 + 4*i.
# Starts with the operators themselves, two spaces apart.
help-expr-numbers-operators = +  -  *  /  und ^ (Potenz; von rechts nach links ausgewertet, also 2^3^2 = 2^9). Klammern gruppieren. Es gibt keine implizite Multiplikation: 2*pi schreiben, nicht 2pi.
help-expr-numbers-case = Groß- und Kleinschreibung spielt bei Namen keine Rolle: PI, Pi und pi sind dieselbe Konstante.
help-expr-constants = Konstanten
help-expr-functions = Funktionen
help-expr-functions-angles = Winkel sind im Bogenmaß. Für Grad mit pi/180 multiplizieren — cos(45*pi/180) — oder den Dialog „Zu Ort springen“ in den Polarmodus schalten und die Einheit Grad wählen.
help-expr-refused = Was abgelehnt wird, und warum
help-expr-refused-fractional = Eine negative Zahl hoch einem gebrochenen Exponenten, etwa (-2)^(1/2): Das Ergebnis ist komplex, und es gibt mehrere Wurzeln zur Auswahl. Für eine reelle Wurzel root(x, n) oder cbrt(x) verwenden, für die komplexe Hauptwurzel sqrt(x).
help-expr-refused-complex = Ein komplexes Argument für eine reellwertige Funktion (sin(i), ln(1+i)). Nur sqrt und abs nehmen ein komplexes Argument an; die arithmetischen Operatoren und ganzzahlige Potenzen arbeiten mit komplexen Werten.
help-expr-refused-domain = Definitionsbereichsfehler: ln oder log einer nicht positiven Zahl, asin oder acos außerhalb von -1…1, 0 hoch einer negativen Zahl, Division durch null.
help-expr-refused-bounds = Argumente von sin, cos, tan, exp oder ^ jenseits von ±2^32 und Verschachtelungen tiefer als 32 Ebenen: Eine eingefügte Koordinate ist eine nicht vertrauenswürdige Eingabe, und diese würden unbegrenzt Zeit kosten.
help-expr-refused-grammar = Keine Variablen, benutzerdefinierten Funktionen, Vergleiche oder Bedingungen. Die Grammatik ist bewusst klein gehalten, damit ein gespeicherter Ort in jeder Version dasselbe bedeutet.
help-expr-no-parse = Wenn sich der Ausdruck nicht lesen lässt
help-expr-no-parse-text = Die Meldung nennt das Feld, sagt, was an welchem Zeichen gefunden wurde, und — bei den üblichen Versehen — was stattdessen einzugeben ist: ein aus einem Dokument eingefügtes typografisches Minuszeichen (−), × oder ÷, ein Dezimalkomma, ein fehlendes * zwischen einer Zahl und einem Namen, eine nicht geschlossene Klammer oder eine Funktion ohne ihre Klammern. Nichts wird je halb gelesen: Ist irgendein Teil des Texts unverständlich, wird die ganze Koordinate abgelehnt, statt stillschweigend gekürzt zu werden.

## Help ▸ Navigation
## Menu paths (Navigate → "Go to location…") and quoted items name the interface's own
## menus and controls: use their translations. Key names (Ctrl, Shift, Space, Esc) stay.

# A heading: the mouse controls (also in Shortcuts).
help-mouse = Maus
# The left column of a controls table: a gesture or key. Keep key names (Shift, Space, Ctrl) as they are.
help-key-left-drag = Links ziehen
help-nav-left-drag = Die Ansicht verschieben.
help-key-mouse-wheel = Mausrad
help-nav-wheel = Zum Mauszeiger hin hinein-/herauszoomen; schneller drehen zoomt pro Raste weiter. Ist „Zoom an Merkmalen einrasten“ an, zieht das Hineinzoomen nahe einem Minibrot den Zoom auf dieses.
help-key-shift-drag = Shift+Ziehen
help-nav-shift-drag = Rahmenzoom — ein Rechteck aufziehen, und es wird auf Ansichtsgröße gezoomt.
# $icon: the click-to-zoom toolbar button's symbol.
help-key-click-zoom = Klick-Zoom ({ $icon })
help-nav-click-zoom = Die Zielscheiben-Schaltfläche in der Symbolleiste umschalten (Einzelansicht): Ein Linksklick taucht dann um einen eingestellten Faktor in den Punkt ein, ein Rechtsklick zoomt zurück. Ziehen verschiebt weiterhin; der Faktor (2–100×) wird unter „Navigation“ im rechten Bedienfeld gewählt. Backspace macht einen Klick rückgängig.
help-nav-continuous = Dauerzoom & Startansicht
help-key-hold-space = Space halten
help-nav-hold-space = Stufenlos hineinzoomen, verankert am Mauszeiger.
help-key-hold-shift-space = Shift+Space halten
help-nav-hold-shift-space = Stufenlos herauszoomen.
help-nav-continuous-text = Die Schaltfläche „Zur Startansicht“ animiert einen sanften Rückflug zur Gesamtansicht. „Zoomtempo“ im rechten Bedienfeld legt die Geschwindigkeit des Dauerzooms fest.
help-nav-autopilot = Autopilot
help-nav-autopilot-text = A drücken (oder Extras → „Auto-Zoom (Autopilot)“) für einen Tauchgang ohne Zutun: Er wählt einen Punkt am Rand des Fraktals, wo die Details am dichtesten sind, bevorzugt in der Mitte der Ansicht, und gleitet hinein, während er ihn sanft zur Bildschirmmitte führt. Er behält dieses Ziel, solange es interessant bleibt, und schwenkt sanft zu reicherer Struktur, wenn welche auftaucht. Jede Navigationseingabe — oder Esc — hält ihn an. „Auto-Zoom-Priorität“ im rechten Bedienfeld (oder unter Extras) bestimmt, was Vorrang hat: Tempo zoomt mit der eingestellten Geschwindigkeit und aktualisiert den Bildschirm, sobald ein Bild mit Details bereitsteht; Qualität zeigt nur vollständig aufgelöste Bilder in voller Auflösung und verlangsamt den Zoom nach Bedarf, damit das Bild nie stark vergrößert wird, bevor das nächste eintrifft. So oder so zeigt der Bildschirm nie ein leeres oder einfarbiges Bild: Eine Aktualisierung, die noch keine Details aufgelöst hat, bleibt hinter der letzten verborgen. „Auto-Zoom-Ziel“ bestimmt, worauf er zielt: Detail (der reichste Rand in der Ansicht, laufend neu gewählt) oder Misiurewicz-Punkt — er findet den nächsten Misiurewicz-Punkt, berechnet ihn bis zur Tiefe der Tauchgrenze, während er stillhält, und taucht dann exakt hinein; die Struktur um einen solchen Punkt wiederholt sich alle paar Oktaven endlos, daher endet dieser Tauchgang nie in einer Sackgasse (nur Mandelbrot-Menge).
help-nav-history = Verlauf & präzise Bewegungen
help-nav-undo = Die letzte Ansichtsänderung rückgängig machen.
help-nav-redo = Wiederholen.
help-nav-goto = Navigation → „Zu Ort springen…“ ermöglicht es, die exakte Mitte und den Zoom abzulesen, einzugeben, einzufügen oder zu kopieren (volle Genauigkeit, jede Tiefe). Die Felder der Mitte nehmen mehr als einfache Zahlen an: Brüche wie „1/3“ und Ausdrücke aus pi, tau, e, sqrt und sin/cos/tan — also ist „-0.5 + 0.25*cos(pi/4)“ eine gültige Koordinate. Ein Ausdruck wird mit der Ansicht aufbewahrt und mit der Genauigkeit neu ausgewertet, die jede Tiefe braucht, sodass ein so eingegebener Punkt exakt bleibt, wie weit auch hineingezoomt wird (eine gerundete Dezimalzahl täte das irgendwann nicht mehr). Ein Polarmodus (Versatz + r ∠ θ, in Grad / Bogenmaß / Umdrehungen) setzt einen solchen Ausdruck selbst zusammen. Die vollständige Liste der Funktionen und Konstanten mit Beispielen steht im Abschnitt „Koordinatenausdrücke“ dieser Hilfe (die Schaltfläche ? im Dialog öffnet ihn). Navigation → Lesezeichen speichert Orte und ruft sie wieder auf.
help-nav-exact-points = Navigation → „Exakte Punkte (Ausdrücke)“ springt zu Punkten, die exakt statt durch eine gerundete Dezimalzahl definiert sind — die Ansatzpunkte der Knospen der Hauptkardioide, an denen jede Knospe der Periode q ansetzt, als cos/sin-Ausdrücke geschrieben. Da der Ausdruck mit der Ansicht mitwandert, lässt sich in einen solchen Punkt hineinzoomen, ohne je von ihm abzudriften.
help-nav-share = Navigation → „Ort teilen…“ erfasst die ganze Ansicht — Fraktal, Mitte in voller Genauigkeit, Zoom und Färbung — als kompaktes „.fdn“-Textstück, das sich kopieren, wieder einfügen und anwenden oder als Datei speichern/laden lässt, um eine Stelle exakt wiederherzustellen (oder sie jemandem zu schicken).
help-nav-open = Datei → „Ansicht oder Ort öffnen…“ öffnet jedes davon direkt: ein exportiertes PNG/EXR (dessen eingebettete Ansicht wiederhergestellt wird), eine „.fdn“-Ortsdatei oder eine „.kfr“ von Kalles Fraktaler.
help-nav-finding = Details finden
help-nav-minibrots = Minibrots sind winzige Kopien der ganzen Menge, die entlang des Randes verborgen sind. Eines grob zentrieren und M drücken (oder Extras → „Minibrot finden + hinzoomen“), um per Newton-Verfahren exakt auf sein Zentrum einzurasten, seine Periode abzulesen und direkt auf seinen eigenen Maßstab zu zoomen — oft viele Größenordnungen in einem Schritt. Ist man schon tiefer als das Minibrot, bewegt sich nur die Mitte.
help-nav-minimap = Ansicht → „Übersichtskarte“ zeigt eine kleine Karte der ganzen Menge mit einer „Sie sind hier“-Markierung und der aktuellen Zoomtiefe; ein Klick auf die Karte springt zu einem Bereich.

## Help ▸ Coloring & options
## Names in quotation marks are the controls' own labels in the right-hand panel.

help-options-intro = All dies befindet sich im rechten Bedienfeld (und bleibt zwischen Sitzungen erhalten).
help-options-palette = Palette
help-options-palette-text = Palette wählt einen vorgefertigten Farbverlauf, den eigenen (Eigene) oder einen Zweifarbmodus. „Farbverlauf bearbeiten…“ öffnet einen Editor, in dem jeder Farbstopp eine Farbe und eine Position (0–1) hat; Farbstopps hinzufügen, eine Vorlage als Ausgangspunkt kopieren oder eine Palettendatei importieren.
help-options-paste = „Einfügen…“ im Farbverlaufseditor importiert eine anderswo gefundene Palette. Es nimmt Hex-Farben mit oder ohne vorangestelltes # an (#ff8800, ff8800 oder das kurze #f80) sowie einfache RGB-Tripel 0–255 wie „255 136 0“ — das Format der .map-Dateien von Fractint und Kalles Fraktaler —, getrennt durch Kommas, Leerzeichen oder Zeilenumbrüche. Bis zu 256 Farben werden übernommen; eine längere Liste wird gleichmäßig auf 256 ausgedünnt, wobei die erste und die letzte erhalten bleiben, damit die Form des Farbverlaufs bestehen bleibt.
help-options-import-map = „Importieren ▸ .map…“ daneben lädt eine .map-Palettendatei von Fractint oder Kalles Fraktaler. Das sind nach Iterationszahl indizierte Nachschlagetabellen ohne Überblendung zwischen den Einträgen, und die harten Farbstufen, die dabei entstehen, machen den klassischen Fractint-Look aus — so werden sie also importiert, mit einem Kästchen „Harte Bänder“, um dieselben Farben stattdessen zu einem glatten Farbverlauf zu überblenden. Fractints eigene Dateien speichern 6-Bit-Farbwerte mal vier, daher ist ihr Weiß 252 statt 255; Fractadyne weist beim Import darauf hin und belässt die Werte genau so, wie sie geschrieben sind, weil Fractints eigene gespeicherte Bilder dasselbe 252 tragen.
help-options-duotone = Duoton bildet den Wert auf eine glatte Zweifarbrampe ab (Schatten → Lichter). Binär (Menge) ist eine flache Zweifarbansicht — eine Vollfarbe für Punkte in der Menge und eine andere für außerhalb, ohne Farbverlauf — die klarste Art, die Form der Menge zu sehen.
help-options-cycle = Zyklus legt fest, wie oft sich der Farbverlauf über den Iterationsbereich wiederholt (engere oder weitere Bänder). Versatz dreht den ganzen Farbverlauf.
help-options-normalize = „Tiefe Farben normalisieren“ passt die Palette an den Bereich der Fluchtwerte an, die in der Ansicht tatsächlich vorkommen. Tiefe Ansichten umfassen Hunderttausende von Iterationen, und ohne dies bricht ein fester Zyklus den Farbverlauf tausendfach zwischen benachbarten Pixeln um und macht korrekte Details zu Gesprenkel.
help-options-log-scale = „Log. Farbskala“ verteilt die Palette nach dem Logarithmus des Fluchtwerts statt linear. Mit zunehmender Tiefe drängen sich die Fluchtzahlen am oberen Ende, sodass eine lineare Anpassung den Großteil des Farbverlaufs auf eine dünne Schale dicht am Rand verwendet und den Rest fast flach lässt; die logarithmische Skala öffnet das und hält die Farben in einem Zoomvideo stabiler, während der Bereich wächst.
help-options-animate = Animieren lässt die Farben mit der Zeit wandern — Vorwärts, Rückwärts oder Ping-Pong verschieben den Versatz; Zufällige Farbverläufe erzeugt fortlaufend weich ineinander übergehende, harmonische Farbverläufe. Tempo steuert die Zyklen (oder Übergänge) pro Sekunde; „Farbverlauf mischen“ würfelt im Zufallsmodus einen neuen aus.
help-options-method = Färbemethode — wie aus den Daten Farbe wird
# The coloring methods, by the names the Coloring panel gives them, each with what it does.
help-method-smooth = Glatte Iteration
help-method-smooth-text = Klassische kontinuierliche Bänder nach Fluchtzeit.
help-method-stripe = Streifenmittel
help-method-stripe-text = Fließende Bänder aus dem Winkel des Orbits (Regler „Streifendichte“).
help-method-triangle = Dreiecksungleichung
help-method-triangle-text = Feine Textur daraus, wo jeder Schritt zwischen den Schranken landet.
help-method-trap = Orbitfalle
help-method-trap-text = Abstand des Orbits zu einer Form (Punkt/Kreuz/Kreis); färbt auch das Innere.
help-method-distance = Distanzschätzung
help-method-distance-text = Schattiert nach Nähe zum Rand.
help-method-decomposition = Zerlegung
help-method-decomposition-text = Zellen aus dem letzten Fluchtwinkel.
help-options-relief = 3D-Reliefbeleuchtung
help-options-relief-text = Schattiert die Oberfläche anhand der Steigung des Randes (der Ableitung) für einen geprägten, beleuchteten Look. Lichtwinkel legt die Richtung fest, Relief die Stärke (niedriger = schärfer); „Licht drehen“ animiert es. Nur holomorphe Familien (Mandelbrot / Multibrot).
help-options-glow = Distanzleuchten
help-options-glow-text = Helle Konturbänder, die sich nahe dem Rand zu leuchtenden Filamenten verdichten. Leuchten ist der Mischanteil, Bandbreite der Abstand, und „Leuchten animieren“ lässt sie fließen.
help-options-quality = Qualität & Iterationen
help-options-quality-text = Iterationen ist die Höchstzahl an Schritten, bevor ein Pixel als in der Menge liegend gilt; die „Auto-Skalierung“ erhöht sie beim Zoomen automatisch (tiefere Details brauchen mehr). Kantenglättung tastet Standbilder mehrfach ab (2×–8×), sobald die Ansicht zur Ruhe kommt, und bändigt den feinen „Staub“ im Äußeren.
help-options-other = Sonstiges
help-options-other-text = Zoomtempo legt die Geschwindigkeit des Dauerzooms fest; die FPS-Obergrenze begrenzt die Bildrate; „Beide, verknüpft“ zeigt eine Mandelbrot-Menge und ihre Julia-Menge nebeneinander (der Mauszeiger setzt den Julia-Parameter live).

## Help ▸ Fractals
## Fractal family names (Mandelbrot, Multibrot, Tricorn, Burning Ship, Celtic, Buffalo,
## Phoenix, Newton) are names: keep them as written, as the formulas. Formula syntax
## (z = z^3 - p1*z + c, sqr, |z|, if / elseif / endif, p1-p5, maxit) is typed as shown.

help-fractals-intro = Jede Familie iteriert eine Formel, bei der z bei 0 beginnt und c durch das Pixel gegeben ist (Fluchtzeit), sofern nicht anders angegeben. z = x + iy ist eine komplexe Zahl.
help-fractals-mandelbrot = z → z² + c. Das Original. Die Menge besteht aus allen c, deren Orbit beschränkt bleibt; sie ist zusammenhängend, und ihr Rand ist so zerknittert, dass er die Hausdorff-Dimension 2 hat.
help-fractals-multibrot = z → z^d + c für die Potenz d = 3, 4, 5. Höhere Potenzen fügen Lappen hinzu: Die Menge hat eine (d−1)-zählige Rotationssymmetrie (Multibrot 3 ist 2-zählig, 4 ist 3-zählig, 5 ist 4-zählig).
help-fractals-tricorn = z → conj(z)² + c, wobei conj(x + iy) = x − iy. Die Konjugation macht es antiholomorph, was eine 3-zählige Symmetrie und charakteristische gekrümmte „Klauen“ ergibt.
help-fractals-burning-ship = z → (|x| + i|y|)² + c — vor dem Quadrieren werden die Beträge genommen (real = x²−y²+cx, imag = 2|xy|+cy). Nicht analytisch; tiefe Zooms zeigen schiffsartige Strukturen (traditionell kopfüber betrachtet).
help-fractals-celtic = Wie Mandelbrot, aber mit dem Betrag des Realteils: real = |x²−y²| + cx, imag = 2xy + cy. Erzeugt herz- und schildförmige Motive.
help-fractals-buffalo = Betrag beider Teile von z²: real = |x²−y²| + cx, imag = |2xy| + cy — eine Kreuzung aus Celtic und Burning Ship.
# A heading: the families at powers above 2 (the Fractal menu's "Higher powers").
help-fractals-powers = Höhere Potenzen
# ("Burning Ship  3 4 5") is how the Fractal menu shows such a row: keep it as it is.
help-fractals-powers-text = Multibrot 6, 7 und 8 (z^d + c) sowie Burning Ship, Tricorn, Celtic und Buffalo mit den Potenzen 3, 4 und 5: dieselben Faltungen bei höherer Potenz. Burning Ship d nimmt die Beträge der Teile von z und dann die Potenz; Tricorn d konjugiert zuerst; Celtic d und Buffalo d falten den Realteil (und bei Buffalo auch den Imaginärteil) von z^d. Das Menü „Fraktal“ und die Auswahlliste führen sie mit einer Familie pro Zeile auf („Burning Ship  3 4 5“). Jede zoomt so tief wie ihr Geschwister mit Potenz 2, und Multibrot 6, 7 und 8 nutzen ebenfalls die Reihennäherung und die Minibrot-Suche (M), wie Multibrot 3–5. Multibrot 8 entkommt bei Radius 128 statt 256, wo sein nächster Schritt den Bereich einfacher Genauigkeit verlassen würde.
help-fractals-phoenix = z → z² + c + p·z(n-1), wobei z(n-1) der vorige Iterationswert ist und p eine Konstante (p = -0.5). Der Gedächtnisterm erzeugt flammenartige Filamente.
help-fractals-newton = Das Newton-Verfahren für die Nullstellen von z³ − 1 = 0: z → z − (z³−1)/(3z²). Statt nach Fluchtzeit werden Pixel danach gefärbt, gegen welche der drei dritten Einheitswurzeln die Iteration konvergiert (die Einzugsgebiete) und wie schnell; die verschlungenen Grenzen der Einzugsgebiete sind das Fraktal.
help-fractals-julia = Julia-Modus
help-fractals-julia-text = Für jede Familie außer Newton lässt sich zu einer Julia-Menge wechseln: Statt z bei 0 mit c = Pixel zu beginnen, wird c (ein Parameter) festgehalten, und z beginnt beim Pixel. Die Julia-Menge ist für dieses feste c die Grenze zwischen Startpunkten, die beschränkt bleiben, und solchen, die entkommen. In „Beide, verknüpft“ setzt die Bewegung des Mauszeigers über der Mandelbrot-Hälfte c live.
help-fractals-deep = Tiefenzoom-Unterstützung
help-fractals-deep-text = Mandelbrot, Multibrot 3/4/5 und Tricorn unterstützen Tiefenzoom per Störungsrechnung in extremer Tiefe (floatexp). Burning Ship, Celtic und Buffalo sind nicht analytisch (sie nehmen Beträge), daher verwenden sie eine vorzeichenbewusste Störungsrechnung; diese läuft jetzt ebenfalls im floatexp-Bereich und zoomt weit über die alte df32-Grenze von ~1e28× hinaus (seltenes Gesprenkel nahe den Betragsfaltungen wartet noch auf die Glitch-Korrektur mit mehreren Referenzen). Phoenix verwendet eine zweigliedrige Störungsrechnung und zoomt ebenfalls im floatexp-Bereich tief (ohne SA/BLA, daher aufwendiger als Mandelbrot). Die höheren Potenzen zoomen auf dieselbe Weise tief, wobei die Störungsrechnung jeder Potenz ihrer eigenen Kette von Quadrierungen folgt; Multibrot 3–8 nutzen außerdem Reihennäherung und BLA wie Mandelbrot, und Tricorn, Burning Ship, Celtic und Buffalo nutzen bei jeder Potenz BLA (ohne Reihe), das das Innere einer tiefen Ansicht trägt, statt es Schritt für Schritt zu iterieren. Nur Newton verwendet den direkten Pfad, scharf bis ~1e6×; eigene Formeln zoomen tief, sofern der Abschnitt „Eigene Formeln“ unten nichts anderes sagt.
help-fractals-custom = Eigene Formeln
help-fractals-custom-text = Fraktal ▸ Eigene Formel… nimmt den Schritt einer Formel in Ausdrücken im Fractint-Stil an: z = z^3 - p1*z + c oder mehrere Anweisungen (t = sqr(z), dann z = t*t + c). z ist der Iterationswert, c das Pixel (die Julia-Konstante im Julia-Modus), p1-p5 sind Parameter; |z| ist der quadrierte Betrag, wie in Fractint, und die üblichen Funktionen stehen bereit (sqr, abs, conj, real, imag, cabs, flip, recip, ident, exp, log, sqrt, sin, cos, tan, cotan, sinh, cosh, tanh, cotanh, asin, acos, atan, asinh, acosh, atanh, cosxx sowie floor, ceil, trunc und round, die jeden Teil runden). log ist der natürliche Logarithmus, wie in Fractint und in der Funktionentheorie: was viele Lehrbücher ln schreiben (einen Zehnerlogarithmus gibt es nicht). Eine mit ln, cot, coth, Re oder Im geschriebene Formel lässt sich nicht lesen; die Syntaxprüfung nennt die zu verwendende Funktion, eine Schaltfläche schreibt sie um, und die Eingabe von ln bietet log an. Jeder Schritt zoomt per Störungsrechnung tief, mit derselben Arithmetik mit erweitertem Wertebereich ab 1e28x wie die eingebauten Fraktale, außer bei einer Potenz, deren Exponent von z oder c abhängt (etwa z^c): Eine solche Formel wird auf dem direkten Pfad gerendert, scharf, bis ein einzelner Schritt von c in einfacher Genauigkeit ein Pixel überspannt, etwa 1e4x bis 1e5x, weit vom Ursprung entfernt weniger tief. Der Formeldialog sagt, was zutrifft. log, sqrt und nicht ganzzahlige Potenzen nehmen ihre Hauptwerte an, die über die negative reelle Achse springen. Die Störungsrechnung folgt dem Sprung exakt, aber ein Pixel, dessen Orbit den Schnitt dort kreuzt, wo der der Referenz es nicht tut, wird von da an nur noch mit etwa einfacher Genauigkeit weitergeführt, sodass tiefe Ansichten, deren Orbits einen Schnitt überspannen, dort Rauschen zeigen (gemessen: sauber bei 1e5x auf der reellen Achse für sqrt(z^4 + c), verrauscht für log-Formeln schon bei 1e4x). Jeder Schritt wird vollständig iteriert (keine Reihennäherung), daher rendert eine tiefe Ansicht mit hoher Iterationszahl langsamer als die eines eingebauten Fraktals. Nahe ihrem Rand verlaufen die Orbits einer Formel mit Funktionen chaotisch, und ein langer Orbit wird dort in einfacher Genauigkeit nur lose verfolgt: Bei 2.000 Iterationen weicht ein Drittel der Pixel von sin(z) + c vom exakten Orbit ab, bei 60 fast keines. Ansichten, Sitzungen und exportierte Bilder bewahren den Text der Formel.
help-fractals-custom-sections = Die Abschnitte von Fractint funktionieren ebenfalls. Anweisungen vor einem ':' bilden den Init-Abschnitt, der einmal pro Pixel läuft; ein Vergleich als letzte Anweisung ist der eigene Abbruchtest der Formel, die iteriert, solange er gilt (|z| <= 100 && |z - 1| > 0.000001 stoppt einen Orbit, der entkommt oder sich auf 1 einpendelt); if (...), elseif (...), else und endif wählen Anweisungen aus; und eine Variable, die in einem Schritt gelesen wird, bevor sie gesetzt ist, hat ihren Wert aus dem vorigen Schritt (aus dem Init-Abschnitt oder 0). Vergleiche vergleichen Realteile und ergeben 1 oder 0, && und || verknüpfen sie, und eine Zuweisung ist ebenfalls ein Wert (a = b = pixel). pixel, p1-p5, pi und e dürfen zugewiesen werden und behalten ihren eigenen Wert, bis das geschieht; maxit ist die Iterationszahl und ismand ist 1 (die Mandelbrot-Form der Formel). Eine Formel mit einem dieser Elemente oder mit Rundung wird auf dem direkten Pfad gerendert, ohne Tiefenzoom. Formelbibliothek > Importieren liest auch .frm-Dateien von Fractint. Jeder lesbare Eintrag wird zu einer Formel, und seine Zeile sagt, was das Einlesen verändert hat: fn1-fn4 übernehmen Fractints Standardwerte (sin, sqr, sinh, cosh); Fractints Variable c wird in c_ umbenannt, da c hier das Pixel ist; ein Name, den nichts setzt, ist 0, wie in Fractint; Leerzeichen innerhalb eines Namens entfallen (end if ist endif), so wie Fractint sie entfernt. Die Meldung des Imports zählt die nicht lesbaren Einträge und nennt den häufigsten Grund: Die Bildschirm- und Ansichtsvariablen (whitesq, scrnpix, scrnmax, center, magxmag, rotskew), lastsqr und Zufallszahlen werden nicht unterstützt. Die Fractint-Klassiker der Sammlung (Magnet, Nova, Spider, Manowar und andere) sind mit diesen Abschnitten geschrieben.
help-fractals-custom-editor = Im Formelfeld erhalten Klammern eine Farbe pro Verschachtelungsebene; das Paar neben dem Cursor wird hervorgehoben, und eine Klammer ohne Partner erscheint rot (die eines Kommentars zählen nicht, da der Parser sie ignoriert). Namen werden beim Tippen ergänzt: Nach zwei Buchstaben bietet eine Liste die Funktionen, Namen und eigenen Variablen an, die so beginnen. Up und Down wählen, Tab oder Enter übernimmt (eine Funktion kommt mit ihrer öffnenden Klammer), Esc schließt die Liste, und Enter nach einem vollständig eingegebenen Namen beginnt weiterhin eine neue Zeile.
help-fractals-custom-textbook = Lehrbuch (der Schalter über der Formel) zeigt sie so gesetzt, wie ein Lehrbuch sie setzt, und bearbeitet sie direkt dort. Eingabe wie im Text: / macht aus dem Term davor einen Bruch, ^ einen Exponenten (+, - oder = am Ende eines Exponenten getippt, verlässt ihn), ( Klammern oder, nach einem Funktionsnamen, einen Aufruf, | senkrechte Striche, und ein Komma innerhalb von Klammern eine komplexe Konstante. Die Pfeiltasten bewegen sich durch die Formel, Up und Down zwischen Zähler und Nenner, Tab zum nächsten leeren Feld; Backspace am Anfang eines Bruchs oder eines Klammerpaars wählt es aus, und ein zweiter Druck entfernt es und behält, was es enthielt. Kopieren, Einfügen und Rückgängig funktionieren wie in einem Textfeld, Namen werden wie dort ergänzt, und das Tastenfeld schreibt in die gesetzte Formel (□² quadriert, was vor der Einfügemarke steht, eine Funktionstaste umschließt die Auswahl). Jede Bearbeitung wird in den Text zurückgeschrieben, sodass beide Ansichten stets dieselbe Formel enthalten; eine nicht bearbeitete Zeile behält ihre Abstände und ihren Kommentar. Die Notation ändert nie, was berechnet wird: |z| ist der quadrierte Betrag und erscheint als |z|², cabs(z) erscheint als |z|, und exp(z) bleibt exp(z). Eine nicht lesbare Zeile erscheint rot; ein Klick darauf bearbeitet sie als Text. LaTeX kopiert die Formel als LaTeX-Quelltext, so wie sie gesetzt ist, und solange Lehrbuch an ist, zeigt auch die Formelbibliothek ihre Formeln gesetzt.
help-fractals-custom-numbers = Zahlen in einer Formel (0.1, 1/3, pi, die Parameter) werden als Werte doppelter Genauigkeit gelesen, etwa 16 Stellen, und der tiefe Referenzorbit iteriert mit genau diesen Werten. Zwei auf dem Papier gleiche Formeln können sich daher in der Tiefe unterscheiden: 0.1*z multipliziert mit dem Double, das 0.1 am nächsten liegt, während z/10 durch eine exakte 10 teilt, und die beiden Bilder trennen sich ab etwa 1e15x (gemessen: 5 % der Pixel bei 1e15x, ein Drittel bei 1e17x, alle bei 1e20x). Jedes Bild ist für seine eigene Formel korrekt; die Formel schreiben, die man teilen möchte.
help-fractals-custom-library = „In Bibliothek speichern“ im Dialog bewahrt eine Formel und ihre Parameter unter einem Namen auf (das Speichern unter einem Namen, den die Bibliothek schon hat, aktualisiert diesen Eintrag). Fraktal ▸ Formelbibliothek… listet sie zum Anwenden, Bearbeiten, Exportieren oder Löschen auf; Importieren fügt die Formeln einer Formeldatei hinzu, ohne eine zu ersetzen: Eine, die die Bibliothek schon hat, wird übersprungen, und eine neue, deren Name vergeben ist, wird zu „Name (2)“. Die Bibliothek ist die Datei formulas.toml in dem Ordner, der unter „Dateien & Daten“ (Über) genannt ist, und selbst eine Formeldatei. Ein Tour-Keyframe trägt eine Formel als formula = "..." mit formula_params = [[re, im], ...], und Extras > Tour aus aktueller Ansicht schreibt sie für eine Ansicht mit eigener Formel.
help-fractals-custom-collection = Mit Fractadyne kommt eine Sammlung von 45 eigens dafür geschriebenen Formeln: höhere Potenzen und die gefalteten Varianten der Familien Burning Ship und Celtic, Mischungen und zweistufige Hybride, die Mandelbrot-Menge durch eine andere Abbildung gesehen (invertiert, log-polar, durch den Sinus), transzendente und rationale Formeln sowie Julia-Mengen. Sie sind die Beispiele des Dialogs und der Reiter „Sammlung“ der Formelbibliothek, wo „Zu meinen kopieren“ eine davon zu den eigenen Formeln hinzufügt. Jede hat eine Ausgangsansicht, und Anwenden springt dorthin: Eine in der Startansicht geöffnete Formel ist oft nicht dort, wo ihr Bild ist. Eine Formel, die gespeichert wird, während sie angezeigt wird, behält auf dieselbe Weise die aktuelle Ansicht, und Anwenden kehrt dorthin zurück.

## Help ▸ How it works

help-how-escape = Fluchtzeit & glatte Farbe
help-how-escape-text = Jedes Pixel iteriert die Formel, bis sein Betrag einen Fluchtradius übersteigt oder es die Iterationsgrenze erreicht. Die reine Schrittzahl allein erzeugt harte Bänder; ein aus dem letzten Betrag abgeleiteter Nachkommaanteil ergibt kontinuierliche Farbe ohne Bänder.
help-how-precision = Position mit beliebiger Genauigkeit
help-how-precision-text = Gewöhnlichen 64-Bit-Zahlen gehen bei etwa 1e15× Zoom die Stellen aus. Fractadyne hält die Ansichtsmitte in beliebiger Genauigkeit, wobei die Zahl der Stellen beim Zoomen wächst, sodass der Ort nie an Genauigkeit verliert. Der Pixelmaßstab wird ebenso mit erweitertem Exponenten geführt, sodass er auch nicht an der 64-Bit-Grenze von ~1e308× hängen bleibt — die Tiefe ist nur durch Geduld begrenzt, nicht durch einen festen Zahlenbereich.
help-how-perturbation = Störungsrechnung
help-how-perturbation-text = Jedes Pixel in hoher Genauigkeit zu iterieren, wäre viel zu langsam. Stattdessen wird ein Referenzpixel in hoher Genauigkeit auf der CPU iteriert (der „Referenzorbit“), und jedes andere Pixel wird auf der GPU als winzige Differenz δ dazu in schneller, niedriger Genauigkeit berechnet: δz → 2·Z·δz + δz² + δc.
help-how-floatexp = Extreme Tiefe (floatexp)
help-how-floatexp-text = Ab etwa 1e28× unterschreitet selbst diese winzige Differenz den 32-Bit-Bereich, daher wird sie als Mantisse plus separatem ganzzahligem Exponenten gespeichert („floatexp“), was die 32-Bit-Tiefengrenze aufhebt. Die Engine wechselt automatisch: direkte Rechnung bei geringer Tiefe, Störungsrechnung in der Tiefe und floatexp in größter Tiefe. Die Tiefe ist dann durch die Koordinatengenauigkeit und das Iterationsbudget begrenzt statt durch eine feste Grenze — gegengeprüft mit Fraktaler-3 über einen Referenzkorpus von 39 Orten, der tiefste bei 5.63e18003× (die 38 nicht extremen Zeilen sind die Routineprüfung, bis 6.13e1105×), mit einer internen Batterie von Selbstkonsistenztests der Genauigkeit bis 1e1000000×; die mitgelieferte Tour erreicht ~1.3e420×.
help-how-reference = Referenzwahl & Rebasing
help-how-reference-text = Die Referenz wird so gewählt (mit hoher Genauigkeit bewertet), dass ihr Orbit möglichst lange innerhalb der Ansicht bleibt. Wird die Differenz zu groß, wird sie auf die Referenz „zurückgesetzt“ (Rebasing), um genau zu bleiben.
help-how-series = Reihennäherung
help-how-series-text = Bei tiefem Zoom bewegen die frühen Iterationen δ kaum von der Referenz weg, daher lassen sie sich auf einmal berechnen: δz wird durch ein kurzes Polynom in δc (Ordnung 3) angenähert, dessen Koeffizienten entlang der Referenz iteriert werden. Der Renderer springt bis zur letzten Iteration vor, bei der dieses Polynom für die ganze Ansicht noch genau ist, und iteriert dann normal weiter — die übersprungenen Schritte werden gespart, ohne das Bild zu verändern (umschaltbar unter Bedienfeld ▸ Erweitert). Mandelbrot und Multibrot 3–8, tiefster Bereich; nicht mit der Färbung Streifenmittel / Dreiecksungleichung / Orbitfalle / Zerlegung (diese brauchen jede Iteration). Wo BLA aktiv ist, schließt es dieses frühe Überspringen bereits ein.
help-how-distance = Distanzschätzung & Beleuchtung
help-how-distance-text = Die Verfolgung der Ableitung dz/dc liefert für jedes Pixel den Abstand zum Rand. Darauf beruhen die 3D-Reliefbeleuchtung, das Distanzleuchten und die Färbemethode „Distanzschätzung“ — alle in jeder Zoomtiefe gültig.
help-how-safety = Kantenglättung & Sicherheit
help-how-safety-text = Standbilder werden mit Supersampling berechnet (2–8× pro Achse), sobald die Ansicht zur Ruhe kommt. Ein Arbeitsbudget hält einen einzelnen GPU-Zeichenaufruf innerhalb der Watchdog-Grenze des Treibers, indem es bei extremen Einstellungen die Auflösung verringert (nie die Iterationszahl), sodass tiefe Ansichten detailreich bleiben, statt leer zu werden.

## Help ▸ Command line (the flags themselves stay English)

# The section's introduction. The flags listed below it are the command line itself, English in every language.
help-cli-intro = Fractadyne läuft ohne Oberfläche zum Rendern, für Film-/Tour-Export, Automatisierung, Referenzbildprüfungen und Benchmarks. Ohne Optionen startet es die interaktive Oberfläche.

## Help ▸ Shortcuts

# What each mouse gesture and key does (the gestures are the Navigation section's).
help-shortcuts-pan = Verschieben
# A gesture: turning the mouse wheel.
help-key-wheel = Mausrad
help-shortcuts-wheel = Zoom am Mauszeiger
help-shortcuts-box-zoom = Rahmenzoom (ein Rechteck aufziehen, um darauf zu zoomen)
# $icon: the click-to-zoom toolbar button's symbol. "tool on": while that button is on.
help-key-click-tool = Klick ({ $icon } aktiv)
help-shortcuts-click = In den Punkt eintauchen; Rechtsklick zoomt zurück
# A heading: the keyboard shortcuts.
help-shortcuts-keyboard = Tastatur
help-shortcuts-space = Dauerzoom hinein / heraus
help-shortcuts-undo = Ansicht rückgängig
help-shortcuts-redo = Ansicht wiederholen
help-shortcuts-minibrot = Minibrot finden + hinzoomen (Newton-Raphson)
help-shortcuts-autopilot = Auto-Zoom-Autopilot (taucht auf Details zu; jede Eingabe hält ihn an)
# off / strip / panel / full: the panel's sizes, as View ▸ Live render panel names them.
help-shortcuts-live-panel = Live-Render-Panel: Aus / Leiste / Panel / Voll (beim Überfahren eines Instruments erscheint seine Bedeutung; sein × schließt es)
help-shortcuts-esc = Autopilot / eine laufende Tour anhalten oder Vollbild verlassen
help-shortcuts-snapshot = Schnappschuss in den letzten Exportordner — eine Bildschirmaufnahme der Ansicht wie angezeigt oder ein vollständiges Rendering mit den Export-Einstellungen; beim ersten Drücken wird gefragt, und Datei → Einstellungen → Schnappschuss ändert es.
help-shortcuts-help = Diese Hilfe öffnen
# A heading: the shortcuts of the Game of Life, with the menu path to it.
help-shortcuts-life = Life (Fraktal → Automaten → Life)
help-shortcuts-life-run = Universum starten / anhalten
help-shortcuts-life-step = Eine Generation weiter / um die Schrittweite weiter (im Life-Bedienfeld eingestellt)
help-shortcuts-life-draw = Zeichenwerkzeug ein / aus (der Stift in der Symbolleiste)
# A gesture. "Draw on": while the Life draw tool is on.
help-key-click-drag-draw = Klicken / Ziehen (Zeichnen an)
help-shortcuts-life-paint = Eine Zelle umschalten / malen: Ein Strich, der auf einer toten Zelle beginnt, zeichnet; einer, der auf einer lebenden Zelle beginnt, radiert. Ein laufendes Universum pausiert während des Zeichnens und läuft weiter, sobald losgelassen wird.
# $key: a keyboard shortcut (Ctrl+Z). "Draw on": while the Life draw tool is on.
help-key-when-drawing = { $key } (Zeichnen an)
help-shortcuts-life-undo = Die letzte Bearbeitung, Füllung, Leerung oder Regeländerung rückgängig machen

## Help ▸ Recommended hardware

help-hardware-intro = Fractadyne teilt seine Arbeit zwischen der GPU (Iteration + Färbung pro Pixel) und der CPU (dem Referenzorbit mit beliebiger Genauigkeit, auf dem der Tiefenzoom aufbaut). Beide zählen, aber für verschiedene Dinge.
help-hardware-gpu = GPU (am wichtigsten)
help-hardware-gpu-text = Jedes Pixel wird auf der GPU iteriert und gefärbt, daher bestimmt die GPU-Geschwindigkeit die Bildrate und die Exportzeit. Eine dedizierte GPU mit hohem 32-Bit-Durchsatz (einfache Genauigkeit) ergibt den flüssigsten Tiefenzoom. Integrierte GPUs funktionieren gut — nur langsamer, und sie erreichen das Arbeitsbudget pro Bild früher (dann reduziert der Renderer die Auflösung, nicht die Details). Erfordert eine moderne Grafik-API: Vulkan, Direct3D 12, Metal oder OpenGL über wgpu.
# The left column of a requirements table.
help-hardware-minimum = Minimum
help-hardware-gpu-minimum = Jede GPU mit Vulkan / DX12 / Metal / GL 3.3.
help-hardware-recommended = Empfohlen
help-hardware-gpu-recommended = Eine dedizierte GPU (etwa der letzten 5 Jahre); >= 2 GB VRAM.
help-hardware-big-exports = Für große Exporte
help-hardware-gpu-big-exports = >= 4 GB VRAM (gekachelt, daher sanfter Leistungsabfall).
# A heading: the processor.
help-hardware-cpu = CPU
help-hardware-cpu-text = Der hochgenaue Referenzorbit (und die BLA-Tabellen) werden auf der CPU berechnet und laufen größtenteils in einem einzigen Thread, daher zählt vor allem die Leistung pro Kern — sie bestimmt, wie schnell sich eine tiefe Ansicht nach einer Bewegung aufbaut. Zusätzliche Kerne helfen hauptsächlich bei der Export-Kodierung im Hintergrund. Eine 64-Bit-CPU ist erforderlich.
help-hardware-cpu-recommended = Eine moderne 64-Bit-CPU mit guter Einzelkernleistung.
help-hardware-memory = Arbeitsspeicher
help-hardware-memory-text = Tiefe Referenzen wachsen mit der Tiefe (viele Iterationen in hoher Genauigkeit); riesige Exporte halten Kacheln im RAM. 8 GB reichen bequem für die tägliche Erkundung; 16 GB+ eignen sich für extreme Tiefe und sehr große Exporte.
help-hardware-checking = Die eigene Hardware prüfen
help-hardware-diagnostics = Hilfe → „Diagnose…“ führt dieselben Tests aus wie die Entwickler, auf dem eigenen Rechner. „Selbsttest ausführen“ prüft Mathematik und Rendering in etwa fünfzehn Sekunden gegen bekannt korrekte Ergebnisse; „UI-Test ausführen“ geht die Oberfläche und die Live-Ansicht in mehreren Zoomtiefen durch und nimmt dabei Screenshots auf; „GPU-Arithmetik prüfen“ prüft die Double-Float-Arithmetik des Shaders auf jedem Grafik-Backend, das der Rechner bietet, in wenigen Sekunden und ohne Fenster. Jeder Test läuft als eigener Prozess, sodass ein Test, der den Grafiktreiber aus dem Tritt bringt, nur den Test beendet und nicht die eigene Sitzung.
help-hardware-gpu-check = Die GPU-Arithmetikprüfung kennt kein Bestanden oder Nicht bestanden — sie hält fest, wie die jeweilige GPU mit ihrem Treiber rechnet, und auf mancher Hardware (bei jeder bisher getesteten NVIDIA-Karte) meldet sie, dass der Shader-Compiler einen Teil der Arithmetik mit erweiterter Genauigkeit verwirft. Das ist ein echter Befund, den wir über die Hersteller hinweg zu erfassen versuchen, kein Fehler des Rechners — was immer sie meldet, das Anhängen an einen Bericht hilft wirklich.
help-hardware-report = Sieht etwas falsch aus, den Selbsttest ausführen und „An Fehlerbericht anhängen…“ verwenden: Der Bericht enthält dann ein maschinell geprüftes Ergebnis von der eigenen Hardware, was weit nützlicher ist als eine Beschreibung allein. Ein Vorbehalt, den man kennen sollte — die Referenzbilder wurden auf einer bestimmten Grafikkarte erzeugt, und verschiedene GPUs berechnen zu Recht leicht unterschiedliche Pixel, daher werden Bildvergleiche auf anderer Hardware mit größerer Toleranz geprüft, und kleine Abweichungen dort sind zu erwarten und keine Fehler.
help-hardware-notes = Hinweise
help-hardware-note-watchdog = Eine leistungsfähige GPU vermeidet, dass bei extremen Einstellungen das Watchdog-Arbeitsbudget eingreift.
help-hardware-note-binary = Das Programm ist eine einzige, eigenständige Binärdatei — keine Laufzeitinstallation nötig.
help-hardware-note-benchmark = Den eingebauten Benchmark ausführen (Extras ▸ Benchmark), um den eigenen Rechner einzuschätzen.

## Help ▸ Acknowledgments & citations
## Names of people, programs, papers, books and websites stay as written, and so do the
## titles in quotation marks (the works' own titles); the prose around them translates.

help-ack-title = Danksagungen & Quellen
# The dedication, in italics at the top of the section.
help-ack-dedication = Gewidmet der Stone Soup Group — den Freiwilligen hinter Fractint (seit 1988).
help-ack-dedication-text = Ihr frei geteilter Code und ihr Geist offener, gemeinschaftlicher Fraktalerkundung sind der Grund, warum sich so viele von uns in diese Kunst verliebt haben. Fractadyne steht in dieser Tradition.
help-ack-intro = Fractadyne baut auf jahrzehntelanger Arbeit der Fraktal-Community auf. Techniken und Vorarbeiten, auf denen es aufbaut — geprüft anhand der Quellen unten:
help-ack-deep-zoom = Tiefenzoom-Algorithmen
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-martin-title = Störungsrechnung & Reihennäherung — { $who }
help-ack-martin = Die Methode aus Referenzorbit + Delta niedriger Genauigkeit (δz → 2Z·δz + δz² + δc) und die Reihe n-ter Ordnung, die frühe Iterationen überspringt, eingeführt in SuperFractalThing und der zugehörigen Notiz „SuperFractalThing Maths“ (sft_maths.pdf, 2013).
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bla-title = Bivariate lineare Approximation (BLA) & Rebasing — { $who }
help-ack-bla = Überspringen von Iterationsfolgen durch zusammengeführte lineare Abbildungen und Rebasing auf eine einzige Referenz zur Vermeidung von Glitches; entwickelt auf fractalforums.org (2021).
# A link's label. $who: the linked page's author.
help-ack-bla-link = BLA erklärt — { $who }
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-glitch-title = Kriterium zur Glitch-Erkennung — { $who }
help-ack-glitch = Der Test |Z + z| << |Z| dafür, wann ein Pixel eine andere Referenz braucht — die Grundlage glitchfreier Störungsrechnung (fractalforums).
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-diffabs-title = Nicht analytische Störungsrechnung (Burning Ship) — { $who }
help-ack-diffabs = Die Fallunterscheidung „diffabs“ für den Betrag, mit der die Störungsrechnung für die betragsbasierten Familien (Burning Ship / Celtic / Buffalo) funktioniert, auf fractalforums.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mathr-title = Referenzimplementierungen & Gegenprüfungen — { $who }
help-ack-mathr = Fraktaler-3 und Kalles Fraktaler 2+ (ursprünglich von Karl Runmo) — deren Beschreibungen diese Engine geleitet haben und gegen deren EXR-Ausgabe Fractadyne seinen Renderer gegenprüft.
# A heading: credits for the coloring methods.
help-ack-coloring = Färbung
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-harkonen-title = Glatte Iterationszahl & Streifenmittel (Stripe Average) — { $who }
help-ack-harkonen = Die kontinuierliche (normalisierte) Fluchtzeit-Färbung und die Methode Streifenmittel (Stripe Average), aus „On Smooth Fractal Coloring Techniques“ (Masterarbeit, 2007), die auch die Färbungen Dreiecksungleichung / Verzweigungsmittel analysiert.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mitchell-title = Dreiecksungleichungsmittel (Triangle Inequality Average) — { $who }
help-ack-mitchell = Mittelung darüber, wo jedes z² + c innerhalb der Schranken der Dreiecksungleichung landet, für eine glatte, flammenartige Textur im Äußeren.
# A link's label: the person's tutorial pages. $who: the person's name.
help-ack-mitchell-link = { $who } — Tutorials
help-ack-distance = Die Färbung/Beleuchtung per Distanzschätzung verwendet die klassische dz/dc-Methode für den Randabstand (siehe Peitgen & Saupe, „The Science of Fractal Images“, 1988).
help-ack-lsystems-life = L-Systeme & das Game of Life
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-lindenmayer-title = L-Systeme — { $who }
help-ack-lindenmayer = Parallele Zeichenkettenersetzung, 1968 eingeführt, um das Wachstum von Pflanzen und Algen zu modellieren („Mathematical models for cellular interactions in development“, Journal of Theoretical Biology).
help-ack-abop = Das Buch (1990), dessen Turtle-Interpretation und Notation für stochastische, kontextsensitive und parametrische Systeme der L-System-Leser übernimmt; die mit ABOP markierten eingebauten Systeme sind nach seinen Abbildungen eingegeben.
# A link's label: the book can be read free on that website. $site: the website's name.
help-ack-abop-link = Das Buch, kostenlos bei { $site }
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bourke-title = L-System-Sammlung — { $who }
help-ack-bourke = Viele der eingebauten Systeme stammen von seinen L-System-Seiten, darunter Chris Wallaces Regeln für die Sierpinski-Kurve und Dietmar Saupes Busch. Die .l-Dateien von Fractint, die sich direkt öffnen lassen, sind im Format der Stone Soup Group.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-conway-title = Das Game of Life — { $who }
help-ack-conway = Der zelluläre Automat B3/S23 (1970), der durch Martin Gardners Kolumne „Mathematical Games“ im Scientific American (Oktober 1970) in die Welt gelangte.
# A credit's heading: the website, then who runs it. $site: the website's name.
help-ack-lifewiki-title = { $site } — die ConwayLife.com-Community
help-ack-lifewiki = Die Referenz für die Regelnotationen, die das Programm liest (B/S, von Neumann, Generations, Alan Hensels Buchstaben für nicht totalistische Regeln und MAP), für das RLE-Musterformat, das es mit Golly teilt, und für die eingebauten Regeln und Muster, deren Entdecker sie verzeichnet.
help-ack-foundations = Grundlagen & Werkzeuge
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mandelbrot-title = Die Mandelbrot-Menge — { $who }
help-ack-mandelbrot = Das Objekt, um das sich alles dreht.
help-ack-built-with = Gebaut mit Rust, egui/eframe (emilk) und wgpu (gfx-rs); beliebige Genauigkeit über astro-float, Double-Word-Gleitkommazahlen über twofloat; PNG/OpenEXR, rfd, serde. Dank an ihre Autoren und Maintainer.
help-ack-errors = Etwaige Fehler in der Zuschreibung liegen beim Autor; Korrekturen sind im Issue-Tracker des Projekts willkommen.

## Help ▸ Third-party licenses

help-licenses-title = Lizenzen von Drittanbietern
# $licenses: Fractadyne's license expression (MIT OR Apache-2.0), as written. $url: the web address of a crate's source. THIRD-PARTY-NOTICES.md is a file name, `option-ext` a crate's name, MPL-2.0 a license's.
help-licenses-intro = Fractadyne wird als statisch gelinkte Binärdatei verteilt, die die unten aufgeführten Open-Source-Crates und mitgelieferten Schriften enthält. Die vollständigen Lizenztexte (auch in THIRD-PARTY-NOTICES.md, im Download enthalten) folgen. Fractadynes eigener Code ist doppelt lizenziert: { $licenses }. `option-ext` steht unter MPL-2.0 — sein unveränderter Quellcode liegt unter { $url }.
# A button, after a copy icon: copies every notice to the clipboard.
help-licenses-copy-all = Alle Hinweise kopieren

## Help ▸ About

help-about-tagline = Ein nativer Fraktal-Explorer, gebaut in Rust mit wgpu.
# The arithmetic deep zoom has used. $ran: the libraries that have run so far, as names (or a note that none has yet); $built: those this build contains, with versions.
help-about-arithmetic = Tiefenzoom-Arithmetik: { $ran } (dieser Build enthält: { $built }).
help-about-files = Dateien & Daten
# Followed by the folder's path.
help-about-files-text = Die Sitzung (aktuelle Ansicht, Färbung und Einstellungen), Lesezeichen und ihre Vorschaubilder, gespeicherte Farbverläufe und gespeicherte Formeln liegen in diesem benutzereigenen Ordner und werden beim nächsten Start neu geladen:
# Tooltip of the copy button beside the folder's path.
help-about-copy-path = Pfad kopieren
# $command: a command typed in a terminal (it stays English).
help-about-session = Die Sitzungsdatei ist versioniert; wurde sie von einer neueren Fractadyne-Version geschrieben, als vollständig gelesen werden kann, lädt das Programm sie so gut wie möglich und warnt. Für einen völlig neuen Anfang Datei ▸ Programmzustand zurücksetzen… verwenden (fragt vorher nach) oder `{ $command }` in einem Terminal ausführen.
help-about-license = Lizenz
# $licenses: the license expression (MIT OR Apache-2.0), which stays as written.
help-about-license-text = { $licenses } — Nutzung nach Wahl unter einer der beiden Lizenzen.
# A link to the source code, followed by ↗.
help-about-source = Quellcode auf GitHub

## The live render panel ("blinkenlights": View ▸ Live render panel, key B)
## A front panel drawn over the view like a piece of 1970s test equipment. Its legends are engraved
## in capitals: write them in capitals here (the panel shows them exactly as written, it does not
## uppercase them). Numbers, units and readouts on its tubes and dials are not translated.

# Tooltip of the × button in the panel's top-right corner. The code adds the key after it: (B).
live-close-hover = Live-Render-Panel schließen
# The legend engraved along the top plate, left of the maker's badge.
live-plate = LIVE-RENDER · FRONTPLATTE

## Live render panel: instrument titles
## Engraved above each instrument, and repeated as the title of its help card.

# Above the depth readout, whose tubes show the magnification as a power of two ("2^" stays).
live-title-depth = TIEFE 2^
live-title-iterations = ITERATIONEN
# Above the frame-time readout (milliseconds).
live-title-frame-ms = BILD MS
# Above the date-and-time readout.
live-title-clock = DATUM · ZEIT
# The walk dial: a radar screen showing a slow refresh rendered pass after pass, band after band.
live-title-walk = GANG · BÄNDER
# A meter of how far the picture on screen lags the view.
live-title-age = BILDALTER
# What part of the picture this frame's GPU pass covered.
live-title-tiles = KACHELN
# A scope of the GPU's work per frame, in milliseconds.
live-title-scope = GPU-DURCHLÄUFE · MS
# The depth dial: depth, the reference orbits being built, and the arithmetic in use.
live-title-dial = TIEFE · REFERENZEN · ARITHMETIK
# Iteration steps per second.
live-title-steps = SCHRITTE / S
# The two GPUs (the window's and a second, worker GPU).
live-title-engines = ZWEI MASCHINEN
live-title-memory = SPEICHER
live-title-cores = KERNE · THREADS
live-title-pixels = PIXELSTROM
# Help card titles only: the three GPU timer lamps (T0, T1, T2 stay as they are) and the row of lamps.
live-title-timers = GPU-TIMER T0–T2
live-title-bank = LAMPENFELD

## Live render panel: the light bank's lamps
## Engraved under each lamp, and the title of its help card. A cell is narrow (about eight capital
## letters): a longer label breaks at a space onto two lines, and is drawn smaller if it still does
## not fit.

live-lamp-plan = PLAN
live-lamp-dispatch = AUFTRAG
live-lamp-live = LIVE
live-lamp-held = GEHALTEN
# The last picture reprojected (stretched) to fit the moving view.
live-lamp-reproj = REPROJ
live-lamp-reading = MESSUNG
# A pass of a pinned walk.
live-lamp-pin-pass = PIN DURCHLAUF
live-lamp-split = GETEILT
live-lamp-adopt = ÜBERNAHME
live-lamp-abandon = ABBRUCH
# "REF": a reference orbit. Built for the view as it is (LIVE), ahead of it (AHEAD), or for a tour's next hold (HOLD).
live-lamp-ref-live = REF LIVE
live-lamp-ref-ahead = REF VORAUS
live-lamp-ref-hold = REF HALT
# The series approximation.
live-lamp-series = REIHE
# The second (worker) GPU.
live-lamp-worker = HELFER
live-lamp-tile = KACHEL
# Supersampling (anti-aliasing); usually kept as SSAA.
live-lamp-ssaa = SSAA
live-lamp-palette = PALETTE
# The arithmetic in use (direct, df32, floatexp).
live-lamp-mode = MODUS

## Live render panel: the kinds of GPU pass
## In the GPU scope's legend (MOVING, PIN, WORKER, SETTLE, SSAA). SETTLE and SSAA also name a still
## view's walk on the walk dial.

# A moving frame's pass.
live-kind-moving = BEWEGT
# A pass of a pinned walk.
live-kind-pin = PIN
# The worker GPU's passes.
live-kind-worker = HELFER
# A still view's settle tile or walk pass.
live-kind-settle = RUHE
# A supersampling sample.
live-kind-ssaa = SSAA

## Live render panel: legends and readouts on the instruments

# On the walk dial during a split refresh. $done: passes done; $passes: passes in all.
live-walk-split = GETEILT { $done }/{ $passes }
# On the picture-age meter's face, under its scale (0 to 2 octaves).
live-age-scale = OKTAVEN ZURÜCK
# The depth dial's band for direct (plain floating-point) arithmetic. The other bands, DF32 and FLOATEXP, are names and stay.
live-dial-direct = DIREKT
# Under the two GPU gauges: GPU 0 draws the window; GPU 1 is the worker, or there is none.
live-engines-gpu-window = GPU 0 · FENSTER
live-engines-gpu-worker = GPU 1 · HELFER
live-engines-no-worker = KEIN HELFER
# The share of displayed frames each GPU made. $window and $worker: percentages, 0–100.
live-engines-on-screen = AUF SCHIRM { $window } : { $worker }
# The same readout before any frame was shown.
live-engines-on-screen-none = AUF SCHIRM —
# Under the CPU core bars. $count: the machine's cores.
live-cores-count = { $count } KERNE
# Under the threads scope. $count: the app's busy threads.
live-cores-threads = THREADS { $count }

## Live render panel: the memory tanks' legend
## What the GPU memory holds, by kind. Very narrow: about ten letters.

# G-buffers (the picture's working textures).
live-memory-gbuf = G-BUF
# The held picture and the supersampling accumulator.
live-memory-hold = BILD+SSAA
# The reference orbit and its BLA (bilinear approximation) table.
live-memory-orbit = ORBIT+BLA
# The worker GPU's copy of the reference.
live-memory-worker = HELFER-REF

## Live render panel: the roles on the threads scope
## Tiny labels on a small scope: at most four letters each.

# The reference builders.
live-cores-role-ref = REF
# The scoring pool.
live-cores-role-pool = POOL
# The thread driving the worker GPU.
live-cores-role-worker = HLF
# Probes: short side renders.
live-cores-role-probe = PRB
# Image and tour exports.
live-cores-role-export = EXP

## Live render panel: the tiles instrument's captions
## The line under the picture's plane, then the tile size (left) and the tiles a frame (right).

# A walk's pass. $lo and $hi: the iteration band it rendered (numbers).
live-tiles-pin-band = PIN-DURCHLAUF · BAND { $lo }–{ $hi }
live-tiles-settle-band = RUHE-DURCHLAUF · BAND { $lo }–{ $hi }
live-tiles-moving-whole = BEWEGTES BILD · VOLLSTÄNDIG
live-tiles-settled-whole = RUHENDES BILD · VOLLSTÄNDIG
# $pass: this pass; $passes: the passes the refresh is split into.
live-tiles-split-pass = GETEILT · DURCHLAUF { $pass } VON { $passes }
# $grid: the grid's columns × rows, e.g. 12×8.
live-tiles-grid-done = RUHE-RASTER { $grid } · FERTIG
# $grid: the grid's columns × rows; $done and $total: tiles done, and in all.
live-tiles-grid-progress = RUHE-RASTER { $grid } · { $done } VON { $total } FERTIG
# The same, while a tile too costly for one frame goes out in strips.
live-tiles-grid-strips = RUHE-RASTER { $grid } · { $done } VON { $total } FERTIG · IN STREIFEN
# $samples: supersampling samples folded so far; $lo and $hi: the iteration band the next one is walking.
live-tiles-ssaa-band = SSAA · { $samples } FERTIG · BAND { $lo }–{ $hi }
# $sample: the supersampling sample.
live-tiles-ssaa-whole = SSAA-SAMPLE { $sample } · VOLLSTÄNDIG
# No GPU pass this frame, and the panel has not seen a frame yet.
live-tiles-idle-starting = START
# No GPU pass this frame: the last complete picture is held on screen.
live-tiles-idle-held = GEHALTEN
# No GPU pass this frame: the last picture is stretched to fit the view.
live-tiles-idle-reprojected = REPROJIZIERT
# No GPU pass this frame: the picture on screen is current.
live-tiles-idle-live = LIVE
# No GPU pass this frame, and nothing to show.
live-tiles-idle-none = LEERLAUF
# The tile size: $res, a whole picture's resolution (2400×1300).
live-tiles-tile-res = KACHEL { $res }
# The tile size: $px, a tile's side in pixels.
live-tiles-tile-px = KACHEL { $px } PX
live-tiles-no-pass = KEIN DURCHLAUF
# Tiles a frame. $count: how many (0 when none).
live-tiles-rate-count = { $count }/BILD
live-tiles-rate-one = 1 KACHEL/BILD
# $share: the part of one tile sent each frame, e.g. 0.33.
live-tiles-rate-part = { $share } KACHEL/BILD
live-tiles-rate-sample = 1 KACHEL/SAMPLE
# In the plane's corner during a walk: the share of pixels still iterating. $percent: a percentage, e.g. 54 or 1.5.
live-tiles-running = LÄUFT { $percent } %

## Live render panel: help cards
## Shown when the pointer rests on an instrument or a lamp. Words in capitals (SETTLE, SSAA, ABANDON,
## HELD, REPROJ, ADOPT, REF AHEAD, ON SCREEN) are the panel's own legends: use the same words as
## their messages above.

# The heading of a card's list of what healthy looks like.
live-card-healthy = GESUND
# The heading of a card's list of what signals trouble.
live-card-trouble = STÖRUNG

live-card-walk-what = Eine tiefe Aktualisierung, die für ein Bild zu aufwendig ist, wird angeheftet und abgeschritten: Durchlauf für Durchlauf gerendert, wobei jeder die Pixel eine Oktave weiter iterieren lässt (256, 512, 1.024 … bis zur Vorgabe). Der Rand zeigt diese Zahl auf einer logarithmischen Skala, oben beginnend; der Strahl ist der Fortschritt des Gangs; jeder Bogen ist ein Durchlauf, gefärbt nach seiner GPU-Zeit im Verhältnis zum Ziel von 20 ms. Der weiße Punkt wartet auf den Messwert des letzten Durchlaufs; das Aufblitzen ist das neue Bild, das auf den Schirm kommt.
live-card-walk-note = Eine ruhende Ansicht, die für einen Durchlauf zu aufwendig ist, wird auf dieselbe Weise abgeschritten, markiert mit RUHE, und jedes Supersampling-Sample wird erneut abgeschritten, markiert mit SSAA: Ihre Durchläufe dürfen jeweils bis zu 400 ms dauern, und die Bögen werden daran gemessen gefärbt. Eine geteilte Aktualisierung zeichnet stattdessen einen Ring pro Durchlauf: jedes Band, ein Teil der Kacheln.
live-card-walk-good-sweep = Ein Umlauf, der in einer Handvoll Bilder herumgeht und mit einem Aufblitzen endet.
live-card-walk-good-arcs = Bögen meist grün und gelb.
live-card-walk-good-still = Bei einer tiefen ruhenden Ansicht: ein langsamerer Umlauf pro Sample, seine Bögen grün bis gelb.
live-card-walk-bad-red = Rote Bögen: Durchläufe über dem Budget, spürbar als Ruckeln.
live-card-walk-bad-restart = Ein Umlauf, der auf halbem Weg stoppt und neu beginnt: Der Gang wurde abgebrochen, und das Bild auf dem Schirm bleibt alt.
live-card-walk-bad-hollow = Hohle Bögen: Durchläufe, deren Zeitmessung nie zurückkam, sodass der nächste Durchlauf blind bemessen wird.

live-card-age-what = Wie weit das Bild auf dem Schirm der Ansicht hinterherhinkt, in Oktaven des Zooms (eine Oktave ist 2×). Während eine Aktualisierung rendert, wird das alte Bild gestreckt gezeigt und altert; trifft die Aktualisierung ein, sinkt das Alter. Der Streifen darunter zeigt die letzten Sekunden.
live-card-age-good-sawtooth = Ein Sägezahn zwischen etwa 0,3 und 0,8, die Nadel pendelt um die Marke 0,5.
live-card-age-good-still = Nahe 0, sobald die Bewegung aufhört.
live-card-age-bad-red = Die Nadel dauerhaft über 1,0, im roten Bereich: Das Bild ist mehr als 2× veraltet und wirkt beim Zoomen weich oder blockig.
live-card-age-bad-ramp = Eine Rampe, die nie fällt: Aktualisierungen werden nicht fertig (auf ABBRUCH achten).

live-card-tiles-what = Wie der aktuelle GPU-Durchlauf das Bild abdeckt. Bewegte Bilder und Gang-Durchläufe decken es ganz ab, und die Beschriftung nennt das Iterationsband eines Gang-Durchlaufs. Ein Durchlauf eines Gangs iteriert nur die Pixel, die noch nicht entkommen sind: Die Fläche füllt sich bis zu diesem Anteil (logarithmische Skala, 0,01 % am Boden) und zeigt ihn in der Ecke an, sodass sie sich leert, während der Gang tiefer geht. Eine Aktualisierung, die für ein Bild etwas zu aufwendig ist, wird in bis zu 8 Durchläufe geteilt, jeder ein verschränkter Satz von 16-px-Kacheln. Eine ruhende Ansicht, die für einen Durchlauf zu aufwendig ist, wird zu einem Raster von Kacheln, eine pro Bild von der Mitte nach außen, und eine Kachel, die immer noch zu aufwendig ist, geht in Streifen hinaus. Unten angezeigt: Kachelgröße und Kacheln pro Bild.
live-card-tiles-good-whole = Ganze Durchläufe beim Zoomen; geteilte Aktualisierungen aus wenigen Durchläufen.
live-card-tiles-good-grid = Nach dem Anhalten: ein Raster aus einigen Dutzend Kacheln, in ein bis zwei Sekunden gefüllt, dann SSAA.
live-card-tiles-bad-small = Ein Raster aus Hunderten kleiner Kacheln: Das Budget jedes Durchlaufs ist klein, und das Schärfen wird lange dauern.
live-card-tiles-bad-strips = Viele Kacheln gehen als Streifen hinaus: Die Kacheln liefen langsamer, als das Raster bemessen war.
live-card-tiles-bad-splits = Teilungen, die bei 8 Durchläufen hängen: Die Aktualisierung passt gerade noch, und etwas tiefer fällt sie auf den langsameren Gang zurück.
live-card-tiles-bad-dark = Lange dunkel während der Bewegung: gar keine Durchläufe.

live-card-scope-what = Eine Spalte pro Bild, das neueste rechts: die GPU-Arbeit, die es ausgeführt hat, gestapelt nach Art (bewegtes Bild grün, PIN-Durchlauf grün bis rot je nach Kosten, Ruhe-Kachel oder Gang-Durchlauf einer ruhenden Ansicht cremefarben, Supersampling-Sample lavendelfarben, die Bilder und Samples der Helfer-GPU blau), in Millisekunden. Gestrichelte Linien bei 16,7 ms (60 fps), 33 und 50. Eine hohle Markierung ist ein Durchlauf, dessen Zeitmessung nie zurückkam.
live-card-scope-good-under = Spalten unter der Linie bei 16,7 während der Bewegung.
live-card-scope-good-pin = PIN-Durchläufe grün bis gelb.
live-card-scope-good-samples = Die Supersampling-Samples einer ruhenden Ansicht dürfen länger laufen: Es bewegt sich nichts.
live-card-scope-bad-over = Spalten über 33 oder 50 ms: sichtbares Ruckeln.
live-card-scope-bad-seconds = Durchläufe von Sekunden: Der Watchdog der GPU kann das Gerät zurücksetzen.
live-card-scope-bad-hollow = Viele hohle Markierungen: Arbeit wird ohne Messwerte bemessen.

live-card-timers-what = Die drei GPU-Zeitstempel-Plätze. Ein Platz leuchtet auf, wenn ein Durchlauf gemessen wird, und erlischt, wenn sein Messwert zurückkommt, zwei oder drei Bilder später. T0 misst den Durchlauf jedes Bildes; T1 und T2 messen zusätzliche PIN-Durchläufe.
live-card-timers-good = T0 blinkt, wann immer es Arbeit gibt; T1 und T2 blinken während Gängen.
live-card-timers-bad-stuck = Eine Lampe leuchtet dauerhaft: Ihr Messwert kommt nicht zurück, und die Kostenschätzungen veralten.
live-card-timers-bad-dark = Alle dunkel, während die GPU beschäftigt ist: Es wird nichts gemessen.

live-card-dial-what = Der rote Zeiger ist die Tiefe der Ansicht: ihre Vergrößerung als Zweierpotenz. Die Bänder sind die Arithmetik, die die GPU dort verwendet: direkt, df32 (zwei Floats pro Zahl) und floatexp (ein Float mit weitem Exponenten) ab 2^93. Die Punkte sind Referenzorbits im Aufbau: grün für die Ansicht, gelb voraus, wohin der Zoom steuert, blau für den nächsten Halt einer Tour. Ein Ring ist ein Einsetzen; ein rotes Kreuz ist eine Berechnung, die verworfen wurde, weil die Ansicht sie überholt hat.
live-card-dial-good-ahead = Gelbe Punkte, die vor dem Zeiger landen und einen Ring bekommen, bevor er sie erreicht.
live-card-dial-good-crosses = Wenige rote Kreuze.
live-card-dial-bad-passing = Der Zeiger passiert seine zuletzt eingesetzte Referenz: Bilder warten auf eine, spürbar als Einfrieren.
live-card-dial-bad-crosses = Wiederholte rote Kreuze: Der Zoom läuft den Referenzberechnungen davon, und CPU-Zeit wird verschwendet.
live-card-dial-bad-crossing = Der Übergang bei 2^93 ist eine bekannte langsame Stelle: Die ersten floatexp-Bilder kosten mehr.

live-card-bank-what = Jede Lampe leuchtet auf, wenn ihr Ereignis eintritt, und verblasst über etwa 150 ms, sodass ein beschäftigter Renderer flackert und ein untätiger dunkel wird. Von links nach rechts: das Bild, die Aktualisierung, die Referenzen, dann der Helfer, das Zur-Ruhe-Kommen, die Palette und die Arithmetik.
live-card-bank-note = Beim Überfahren einer Lampe erscheint ihre Bedeutung.

live-card-depth-what = Die Vergrößerung als Zweierpotenz: 2^100 ist etwa 1,3 × 10^30.
live-card-depth-note = Ab 2^93 kostet jeder Schritt mehr (floatexp), und Referenzen brauchen länger zum Aufbauen.

live-card-iter-what = Die Iterationsvorgabe: wie weit jedes Pixel iterieren darf, bevor es als in der Menge liegend gilt. Sie steigt mit der Tiefe.
live-card-iter-good = Steigt beim Tauchen stetig.
live-card-iter-bad-low = Bleibt in der Tiefe niedrig: Spiralen und Minibrots werden als flaches Inneres gerendert.
live-card-iter-bad-high = Sehr hoch: Jede Aktualisierung kostet mehr, und Gänge dauern länger.

live-card-ms-what = Die Zeit zwischen angezeigten Bildern.
live-card-ms-good = Das Intervall des Bildschirms: 16,7 bei einem 60-Hz-Bildschirm.
live-card-ms-bad-stutter = Über 33: sichtbares Ruckeln.
live-card-ms-bad-spikes = Spitzen über 50: Hänger, meist ein einzelner aufwendiger Durchlauf (siehe GPU-Durchläufe).

live-card-clock-what = Die Uhrzeit in Ortszeit, auf eine Zehntelsekunde genau.
live-card-clock-note = Sie versieht jeden Screenshot des Panels mit seinem Aufnahmezeitpunkt, sodass sich das Bild eines Problems dem Protokoll des Programms zuordnen lässt.

live-card-tach-what = Iterationsschritte pro Sekunde. Weiß: die angeforderten Schritte (Pixel × Iterationen). Gelb: die Schritte, die die GPU tatsächlich ausgeführt hat. Die Lücke ist BLA (bilineare Approximation), das viele Iterationen auf einmal überspringt; ×N ist dieser Faktor. Das Zählwerk summiert die Sitzung.
live-card-tach-good = Eine weite Lücke in der Tiefe: BLA tut seine Arbeit, ×10 bis ×1.000.
live-card-tach-bad-together = Nadeln beieinander in der Tiefe: BLA kann hier nicht überspringen (oft nahe dem Inneren eines Minibrots), daher sind Aktualisierungen langsam.
live-card-tach-bad-red = Die weiße Nadel im roten Bereich: Es wird mehr verlangt, als die GPU in einem Bild leisten kann.

live-card-engines-what = Wie ausgelastet jede GPU ist, mit Punkten, die ihre Bilder zum Schirm tragen: die GPU-Zeit ihrer Durchläufe in der letzten Sekunde, einschließlich der Bewegungsbilder und Supersampling-Samples des Helfers. AUF SCHIRM ist der Anteil der angezeigten Bilder, den jede geliefert hat. Mit einer GPU bleibt die Helferseite untätig.
live-card-engines-good = Beide beschäftigt während eines Tauchgangs, der Helfer liefert einen Teil der Bilder.
live-card-engines-bad-pinned = GPU 0, die des Fensters, nahe 100 % festgenagelt: Sie ist der Engpass, und die Bilder ruckeln.
live-card-engines-bad-idle = Ein Helfer ist aktiviert, aber untätig: Er liefert keine Bilder.

live-card-memory-what = RAM: der Arbeitsspeicher des Programms, die rote Linie seine Spitze. VRAM: was das Programm auf der GPU hält, berechnet aus den Größen, die es reserviert, nach Art (G-Buffer, Halte- und SSAA-Texturen, Orbit- und BLA-Puffer, die Kopie der Referenz beim Helfer), im Verhältnis zur Karte.
live-card-memory-good = Flach beim Zoomen; steigt und fällt stufenweise, während Referenzen kommen und gehen.
live-card-memory-bad-leak = RAM steigt und sinkt nie: ein Speicherleck.
live-card-memory-bad-vram = VRAM nahe der Größe der Karte: Reservierungen können fehlschlagen, oder das Gerät kann verloren gehen.

live-card-cores-what = CPU-Auslastung pro Kern, grün über gelb bis rot, und ein Schirm, auf dem jeder beschäftigte Thread um seine Rolle kreist: die Referenzberechnungen, der Bewertungs-Pool, der Helfer, Nebenrenderings.
live-card-cores-good = Ein oder zwei Kerne beschäftigt beim Zoomen; ein kurzer Ausschlag über alle Kerne, wenn eine Referenz gewählt wird.
live-card-cores-bad-every = Alle Kerne sekundenlang beschäftigt: Referenzberechnung in extremer Tiefe.
live-card-cores-bad-own = Der eigene Kern des Programms im roten Bereich festgenagelt: Das Fenster selbst ruckelt.

live-card-pixels-what = Pro Sekunde berechnete Pixel, in Mpx/s. Gedämpft: iteriert noch in einem Gang. Hell: fertig.
live-card-pixels-good = Einige Dutzend Mpx/s während der Bewegung.
live-card-pixels-bad = Nahe null, während das Bild veraltet ist: Die GPU wartet auf etwas, eine Referenz oder einen Messwert.

## Live render panel: the lamps' help cards

live-card-lamp-plan-what = Ein Bild wurde geplant: Auflösung, Iterationsvorgabe und Budget gewählt, und ob aktualisiert wird.
live-card-lamp-plan-good = Flackert, wann immer sich Ansicht oder Einstellungen ändern.
live-card-lamp-plan-bad = Dunkel beim Zoomen: Der UI-Thread hängt.

live-card-lamp-dispatch-what = In diesem Bild ging Arbeit an die GPU.
live-card-lamp-dispatch-good = Leuchtet, solange sich etwas ändert; dunkel im Leerlauf.
live-card-lamp-dispatch-bad = Beschäftigt, obwohl sich nichts ändert: Das Programm kommt nicht zur Ruhe.

live-card-lamp-live-what = Das Bild auf dem Schirm ist das eigene Rendering des aktuellen Bildes.
live-card-lamp-live-note = Während eines tiefen Tauchgangs selten an, was normal ist: siehe GEHALTEN und REPROJ.
live-card-lamp-live-good = Leuchtet bei geringen Tiefen, wo ein Bild billig ist.

# "Prefer detail" is a setting's name.
live-card-lamp-held-what = Das letzte vollständige Bild steht auf dem Schirm, während das nächste darunter entsteht: die Aktualisierung eines tiefen Zooms oder, mit „Details beim Zoomen bevorzugen“, die nächste Kachel oder das nächste Supersampling-Sample einer ruhenden Ansicht.
live-card-lamp-held-good-dive = Leuchtet abschnittsweise während tiefer Zooms, unterbrochen von ÜBERNAHME.
live-card-lamp-held-good-still = Leuchtet stetig, während eine ruhende Ansicht schärfer wird, und SSAA blinkt.
live-card-lamp-held-bad = Leuchtet beim Zoomen sekundenlang ohne ÜBERNAHME: Aktualisierungen werden nicht fertig.

live-card-lamp-reproj-what = Das letzte Bild, gestreckt, um der bewegten Ansicht zu folgen, bis ein neues bereit ist.
live-card-lamp-reproj-good = Kurze Schübe während der Bewegung.
live-card-lamp-reproj-bad = Leuchtet lange: Das Bild wird umso weicher, je weiter es gestreckt wird.

live-card-lamp-reading-what = Ein Messwert kam von der GPU zurück: die Zeit eines Durchlaufs, Fluchtbereich oder Inhalt.
live-card-lamp-reading-good = Ein stetiges Flackern während des Renderns.
live-card-lamp-reading-bad = Lange dunkel während des Renderns: Zeitmessungen kommen nicht zurück, und die Budgets raten.

live-card-lamp-pin-pass-what = Ein Durchlauf eines angehefteten Gangs: das Bild ein Iterationsband weiter gerendert.
live-card-lamp-pin-pass-good = Folgen von Durchläufen, die mit ÜBERNAHME enden.
live-card-lamp-pin-pass-bad = Folgen, die mit ABBRUCH enden.

live-card-lamp-split-what = Ein Durchlauf einer geteilten Aktualisierung: ein Teil der Kacheln, mit voller Iterationszahl gerendert.
live-card-lamp-split-good = Kurze Folgen, die mit ÜBERNAHME enden.
live-card-lamp-split-bad = Folgen von 8, dem erlaubten Maximum: Die Aktualisierung passt gerade noch.

live-card-lamp-adopt-what = Eine fertige Aktualisierung hat das Bild auf dem Schirm ersetzt.
live-card-lamp-adopt-good = Mehrmals pro Sekunde beim Zoomen.
# "Picture age" names the instrument (its title, PICTURE AGE).
live-card-lamp-adopt-bad = Selten beim Zoomen: Das Bild altert (siehe Bildalter).

live-card-lamp-abandon-what = Eine laufende Aktualisierung wurde verworfen: Die Ansicht hat sich zu weit bewegt, sie wurde zu alt, oder eine neuere hat sie ersetzt. Eine von der Ansicht überholte Referenzberechnung lässt die Lampe schwach leuchten.
live-card-lamp-abandon-good = Selten, oder einmal direkt nach einem Sprung.
live-card-lamp-abandon-bad = Häufig: Arbeit verschwendet, und das Bild bleibt alt.

live-card-lamp-ref-live-what = Für die Ansicht, wie sie ist, wird ein Referenzorbit berechnet: Sie hat keinen, hat ihren bisherigen verlassen oder braucht einen längeren.
live-card-lamp-ref-live-good = Kurzes Blinken.
live-card-lamp-ref-live-bad-long = Lange Abschnitte: Tiefe Ansichten warten darauf.
live-card-lamp-ref-live-bad-lookahead = Während eines Tauchgangs oft an, während REF VORAUS dunkel ist: Die Vorausberechnung hält nicht Schritt.

live-card-lamp-ref-ahead-what = Eine Vorausreferenz wird tiefer als die Ansicht berechnet, dort, wohin der Zoom steuert.
live-card-lamp-ref-ahead-good = Regelmäßiges Blinken während eines Tauchgangs.
live-card-lamp-ref-ahead-bad = Dunkel während eines tiefen Tauchgangs: Die Ansicht wird ihren Referenzen davonlaufen.

live-card-lamp-ref-hold-what = Das Vorausladen für den Halt einer Tour: die Referenz für das nächste gehaltene Keyframe, berechnet während des Gleitens dorthin, damit die Wiedergabe nie wartet.
live-card-lamp-ref-hold-note = Außerhalb der Tour-Wiedergabe dunkel.
live-card-lamp-ref-hold-good = Ein Blinken vor jedem Halt einer Tour.
live-card-lamp-ref-hold-bad = Leuchtet während eines ganzen Halts: Das Vorausladen ist spät, und die Tour wartet darauf.

live-card-lamp-series-what = Die Reihennäherung für eine neue Referenz, mit der jedes Pixel seine ersten Iterationen überspringen kann.
live-card-lamp-series-good = Kurz.
live-card-lamp-series-bad = Lang in der Tiefe: Sie verzögert das Einsetzen der Referenz.

live-card-lamp-worker-what = Die zweite GPU hat ein Bild geliefert.
live-card-lamp-worker-good = Stetiges Blinken, wenn eine Helfer-GPU aktiviert ist.
live-card-lamp-worker-bad = Dunkel bei aktiviertem Helfer: Er liefert nicht.

live-card-lamp-tile-what = Eine Ruhe-Kachel oder ein Streifen davon ging an die GPU.
live-card-lamp-tile-good = Ein Schub nach dem Anhalten, dann Ruhe.
live-card-lamp-tile-bad = Hört nie auf: Das Raster startet immer wieder neu, weil etwas ständig die Ansicht oder die Einstellungen ändert.

live-card-lamp-ssaa-what = Ein Supersampling-Sample wurde dem ruhenden Bild hinzugefügt.
live-card-lamp-ssaa-good = Eine Folge von Blinkern nach dem Zur-Ruhe-Kommen, dann dunkel.
live-card-lamp-ssaa-bad = Startet immer wieder neu: Das Zur-Ruhe-Kommen wird ständig gestört.

live-card-lamp-palette-what = Die Farbnormalisierung hat einen neuen Fluchtbereich übernommen, sodass die Farben dem Zoom folgen.
live-card-lamp-palette-good = Flackert beim Zoomen.
live-card-lamp-palette-bad = Dunkel beim Zoomen: Der Palettenbereich ist eingefroren, und die Farben werden flach und kontrastarm.

live-card-lamp-mode-what = Die Arithmetik hat gewechselt: direkt, df32 oder floatexp.
live-card-lamp-mode-good = Ein Blinken an einer Grenze wie 2^93.
live-card-lamp-mode-bad = Blinkt hin und her: Die Ansicht pendelt über eine Grenze.

## The Render client window (File ▸ Render client…): lending this machine's GPU to a render farm

farm-client-title = Render-Client
farm-client-intro = Bilder für eine Renderfarm rendern. Der Steuerrechner — Extras ▸ Auf Farm rendern… auf einem anderen Rechner — zeigt seine Adresse und seinen Farmschlüssel an. Dieser Rechner öffnet keinen Port: Er verbindet sich mit dem Steuerrechner.
# A field label, and the label of the status row that names the controller.
farm-client-controller = Steuerrechner
farm-client-find = Suchen
farm-client-find-hover = In diesem Netzwerk nach Steuerrechnern suchen (ein Steuerrechner antwortet, solange seine Farm lauscht)
farm-client-key = Farmschlüssel
# A checkbox beside the masked farm key: show it in clear.
farm-client-show-key = Anzeigen
# A field label (the name this machine goes by), and the label of the status row about this machine.
farm-client-machine = Dieser Rechner
farm-client-machine-hover = Der Name, den der Steuerrechner für diesen Rechner anzeigt.
farm-client-share = Freigegebenes Laufwerk
# Greyed hint inside the empty field; the path is an example.
farm-client-share-hint = optional — z. B. \\fileserver\share
farm-client-share-hover = Der Pfad dieses Rechners zum freigegebenen Laufwerk, das der Steuerrechner verwendet (Freigabemodus): Bilder werden dort geschrieben, statt über die Verbindung gesendet zu werden. Leer lassen, um jedes Bild zu senden.
farm-client-gpu = Grafikkarte
# The label of the row with the idle option.
farm-client-when = Wann
# A checkbox followed by a number of minutes ("Only when idle for [5] min").
farm-client-when-idle = Nur wenn untätig seit
farm-client-when-idle-hover = Arbeit erst annehmen, wenn so lange niemand Tastatur oder Maus dieses Rechners berührt hat, oder solange sein Bildschirm gesperrt ist. Sobald jemand ihn benutzt, stoppt das laufende Bild sofort und geht an die Farm zurück.
# A collapsible section: the largest job this machine accepts.
farm-client-limits = Grenzen
farm-client-limits-note = Ein Auftrag, der mehr verlangt, wird abgelehnt, nie gekürzt.
farm-client-limit-size = Max. Bildgröße
farm-client-limit-ss = Max. Supersampling
farm-client-limit-iter = Max. Iterationen

## Render client: the graphics-card choice

farm-client-gpu-system = Wahl des Systems
farm-client-gpu-every = Alle Grafikkarten
# $count: this machine's graphics cards.
farm-client-gpu-every-n = Alle Grafikkarten ({ $count })
# $count: this machine's graphics cards.
farm-client-gpu-every-each = Alle Grafikkarten ({ $count }) — je eine Sitzung
# A card the window cannot name. $number: its number (or numbers) as --list-adapters gives it.
farm-client-gpu-adapter = Adapter { $number }
farm-client-gpu-finding = Grafikkarten dieses Rechners werden gesucht…
# "NAME · GPU 1" is how such a machine is named in the controller's list: keep it as it is.
farm-client-gpu-hover = Welche Grafikkarte die Bilder der Farm rendert. Mit allen Karten tritt jede als eigener Rechner der Farm bei — „NAME · GPU 1“, „NAME · GPU 2“ —, und „Pausieren“ und „Trennen“ wirken auf alle.

## Render client: looking for controllers on the network (the Find button)

farm-client-finding = Suche nach Steuerrechnern in diesem Netzwerk…
# $port: the UDP port number.
farm-client-found-none = Kein Steuerrechner hat geantwortet. Einer antwortet, solange seine Farm lauscht; seine Firewall muss UDP { $port } hereinlassen.
farm-client-found-pick = In diesem Netzwerk — zum Verwenden einen anklicken:
# $identity: the controller's identity fingerprint.
farm-client-found-hover = Identität { $identity } — diesen Steuerrechner verwenden (sein Farmschlüssel wird weiterhin benötigt)
# After a warning sign. $version and $git: this app's version and build.
farm-client-found-other-build = Ein anderer Build als dieser ({ $version } { $git }): Er würde diesen Rechner ablehnen.

## Render client: what is wrong with what was typed, or with starting the client

farm-client-bad-address = Die Adresse des Steuerrechners als HOST:PORT eingeben, wie sein Fenster sie zeigt (zum Beispiel 192.168.1.20:46733).
# $why: what is wrong with the key (in English).
farm-client-bad-key = Der Farmschlüssel: { $why }
# $why: what is wrong with the name (in English).
farm-client-bad-name = Der Name: { $why }
farm-client-no-config = Es gibt keinen Konfigurationsordner, in dem der Farmschlüssel aufbewahrt werden könnte.
# $why: the system's reason.
farm-client-start-failed = Der Render-Client konnte nicht gestartet werden: { $why }

## Render client: the state, in the status heading

farm-client-phase-connecting = Verbindung wird hergestellt…
farm-client-phase-checking = Selbstprüfung läuft…
farm-client-phase-idle = Verbunden — wartet auf Arbeit
farm-client-phase-rendering = Rendert
farm-client-phase-in-use = Wartet — jemand benutzt diesen Rechner
farm-client-phase-paused = Pausiert
farm-client-phase-retrying = Steuerrechner nicht erreichbar — neuer Versuch
# $seconds: the wait before the next try.
farm-client-phase-retrying-in = Steuerrechner nicht erreichbar — neuer Versuch in { $seconds } s
farm-client-phase-disconnected = Getrennt
farm-client-phase-ended = Beendet

## Render client: the line under the status heading

# $address: the controller's address (HOST:PORT).
farm-client-detail-connecting = Verbindung mit { $address } wird hergestellt…
# $controller: the controller's name.
farm-client-detail-checking = Verbunden mit „{ $controller }“ — Selbstprüfung läuft
# $controller: the controller's name.
farm-client-detail-approval = Verbunden mit „{ $controller }“ — wartet, bis dessen Benutzer diesen Rechner freigibt
# $controller: the controller's name.
farm-client-detail-idle = Verbunden mit „{ $controller }“ — untätig
# $why: the check that failed and why (in English).
farm-client-detail-check-failed = Die Selbstprüfung ist fehlgeschlagen — { $why }
# $why: the controller's reason (in English).
farm-client-detail-refused = Vom Steuerrechner abgelehnt: { $why }
# $why: the controller's reason (in English).
farm-client-detail-closed = Der Steuerrechner hat die Verbindung geschlossen: { $why }
# $why: the controller's reason (in English).
farm-client-detail-ended = Der Steuerrechner hat die Verbindung beendet: { $why }
# $job: the job's name.
farm-client-detail-bundle = Auftrag „{ $job }“ — Paket wird empfangen
# $job: the job's name.
farm-client-detail-job-waiting = Auftrag „{ $job }“ — wartet auf Bilder zum Rendern
# $why: the limit the job is over (in English).
farm-client-detail-job-refused = Diesen Auftrag abgelehnt: { $why }
farm-client-detail-job-over = Der Auftrag ist beendet
# $first and $last: frame numbers.
farm-client-detail-rendering = Rendert Bilder { $first }–{ $last }
# $time: how long nobody must use the machine ("90 s", "5 min").
farm-client-detail-in-use = Wartet — jemand benutzt diesen Rechner; die Arbeit geht weiter, sobald ihn { $time } lang niemand benutzt hat
farm-client-detail-paused = Pausiert — der Steuerrechner wurde benachrichtigt
farm-client-detail-pausing = Wird pausiert — das laufende Bild wird fertiggestellt
farm-client-detail-cancelled = Laufendes Bild abgebrochen; pausiert
# A machine with several graphics cards. $rendering: the cards rendering now; $total: all its cards (two or more).
farm-client-rendering-on-cards = Rendert auf { $rendering } von { $total } Grafikkarten

## Render client: the status rows

# The value of the Controller row. $name: the controller's name; $address: HOST:PORT; $identity: its identity fingerprint.
farm-client-row-controller-value = „{ $name }“ · { $address } · Identität { $identity }
# The value of the This machine row. $name: this machine's name; $identity: its identity fingerprint.
farm-client-row-machine-value = „{ $name }“ · Identität { $identity }
farm-client-row-gpu = GPU
# One graphics card's row. $slot: the card's number.
farm-client-row-gpu-n = GPU { $slot }
farm-client-row-job = Auftrag
# The frames handed to this machine.
farm-client-row-run = Zuteilung
farm-client-row-sent = Gesendet
# The Run row, in parts joined with " · ": the run, the frame in progress, its time. $first and $last: frame numbers.
farm-client-run-frames = Bilder { $first }–{ $last }
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-client-run-frame = Bild { $frame } ({ $index } von { $count })
# $time: the time spent on it so far ("2.1 s").
farm-client-run-frame-time = { $time } für dieses Bild
# One graphics card's run. $frame: the frame in progress; $first and $last: the run's frames.
farm-client-session-frame = Bild { $frame } von { $first }–{ $last }
farm-client-session-no-card = Grafikkarte noch nicht bekannt
# The Sent row. $count: frames sent; $time: the mean time per frame ("2.1 s").
farm-client-sent =
    { $count ->
        [one] { $count } Bild · { $time } pro Bild
       *[other] { $count } Bilder · { $time } pro Bild
    }
# The Sent row of a machine with several cards. $count: frames sent; $split: each card's share ("812 + 422"); $time: the mean time per frame.
farm-client-sent-split =
    { $count ->
        [one] { $count } Bild ({ $split }) · { $time } pro Bild
       *[other] { $count } Bilder ({ $split }) · { $time } pro Bild
    }
# After a warning sign. $note: how the GPUs differ, and both GPUs (in English).
farm-client-gpu-note = Dieser Rechner hat { $note }. Seine Bilder werden leicht von denen des Steuerrechners abweichen, was in einer stehenden Einstellung sichtbar werden kann.
# A collapsible section: the self-check's results.
farm-client-self-check = Selbstprüfung
farm-client-thumb-hover = Das letzte Bild, das dieser Rechner gesendet hat
farm-client-resume = Fortsetzen
farm-client-resume-hover = Wieder Arbeit annehmen
farm-client-pause = Pausieren
farm-client-pause-hover = Das laufende Bild fertigstellen, dann keine Arbeit mehr annehmen. Der Steuerrechner gibt die Bilder dieses Rechners an die anderen.
farm-client-cancel-frame = Bild abbrechen
farm-client-cancel-frame-hover = Das laufende Bild sofort stoppen und pausieren; der Steuerrechner rendert es anderswo.
farm-client-live-paused = Live-Ansicht pausiert, während dieser Rechner Bilder für die Farm rendert.
farm-client-close = Schließen
farm-client-close-active-hover = Das Fenster schließen; dieser Rechner bleibt in der Farm (über das Menü „Datei“ wieder öffnen)
farm-client-close-hover = Das Fenster schließen
farm-client-disconnect = Trennen
farm-client-disconnect-hover = Die Farm verlassen: Das laufende Bild wird zurückgegeben
farm-client-connect = Verbinden
farm-client-connect-hover = Mit dem Steuerrechner verbinden und Bilder für ihn rendern

## The Render on farm window (Tools ▸ Render on farm…): the controller of a render farm

farm-ctl-title = Auf Farm rendern
farm-ctl-out = Ausgabeordner
farm-ctl-browse = Durchsuchen…
farm-ctl-out-note = Bilder sowie der Zustand des Auftrags in farm/: Denselben Auftrag erneut starten, um ihn fortzusetzen.
farm-ctl-size = Größe
farm-ctl-size-custom = Eigene…
# The size list's choice when the size is typed in. $width and $height: pixels.
farm-ctl-size-custom-value = Eigene — { $width }×{ $height }
farm-ctl-fps = Bilder / Sekunde
farm-ctl-ss = Supersampling
farm-ctl-port = Lauschen auf Port
farm-ctl-port-hover = Clients verbinden sich mit diesem Port. Ihn in der Firewall für das lokale Netzwerk freigeben.
farm-ctl-share = Freigegebenes Laufwerk
# Greyed hint inside the empty field.
farm-ctl-share-hint = optional — der Pfad dieses Rechners dorthin
farm-ctl-share-hover = Freigabemodus: Clients, die dasselbe Laufwerk haben, schreiben ihre Bilder dorthin, statt sie über die Verbindung zu senden, und dieser Rechner prüft dort jedes einzelne. Clients ohne das Laufwerk senden ihre.
# A label followed by a number of clients and two checkboxes.
farm-ctl-start-with = Starten mit
# The unit after the number in the box ("Start with [3] clients"). $count: that number.
farm-ctl-start-clients =
    { $count ->
        [one] Client
       *[other] Clients
    }
farm-ctl-start-clients-hover = Der Auftrag beginnt, sobald sich so viele Rechner verbunden und ihre Selbstprüfung bestanden haben; weitere können später beitreten.
farm-ctl-local = Auch auf diesem Rechner rendern
farm-ctl-local-hover = Dieser Rechner zählt als ein Client.
farm-ctl-sharing = Referenzen teilen
farm-ctl-sharing-hover = Der Referenzorbit eines tiefen Bildes kann Minuten bis Stunden zum Aufbau brauchen. Geteilt wird er einmal berechnet und an die anderen Rechner weitergegeben. Eine geteilte Referenz ist eine andere, ebenso gültige: In extremer Tiefe kann sich eine Handvoll Pixel von einem Bild unterscheiden, das seine eigene berechnet hat.
farm-ctl-key = Farmschlüssel
# A checkbox beside the shortened farm key: show all of it.
farm-ctl-show-key = Anzeigen
farm-ctl-copy-key = Schlüssel kopieren
farm-ctl-copy-key-hover = In das Render-Client-Fenster jedes Clients einfügen
# In place of the farm key, when there is none.
farm-ctl-no-key = noch keiner
farm-ctl-make-key = Schlüssel erzeugen
farm-ctl-make-key-hover = Clients brauchen ihn zum Beitreten; er wird auch beim Drücken von „Rendern“ erzeugt
# What to do on the other machines. $address: this controller's HOST:PORT.
farm-ctl-steps = Auf jedem Client: Datei ▸ Render-Client…, Steuerrechner { $address } und der Farmschlüssel.
# The same when this machine's address is not known. $port: the port number.
farm-ctl-steps-port = Auf jedem Client: Datei ▸ Render-Client…, die Adresse dieses Rechners und Port { $port } sowie der Farmschlüssel.
# The size of the job. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
farm-ctl-summary =
    { $count ->
        [one] { $count } Bild · { $seconds } s Tour · { $width }×{ $height } ss{ $ss }
       *[other] { $count } Bilder · { $seconds } s Tour · { $width }×{ $height } ss{ $ss }
    }
farm-ctl-strip-hover = Jedes Bild des Auftrags, von links nach rechts: fertig, in Arbeit, noch offen, aufgegeben.
farm-ctl-no-tour = Keine Tour geladen.
# $folder: the folder; $why: the system's reason.
farm-ctl-out-error = Der Ausgabeordner { $folder }: { $why }
# $why: the system's reason.
farm-ctl-start-failed = Die Farm konnte nicht gestartet werden: { $why }
# $code: the controller process's exit code, as the code gives it.
farm-ctl-ended-exit = Der Steuerrechner wurde beendet (Exit-Code { $code }).

## Render on farm: the state, in the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-phase-waiting = Wartet auf Render-Clients ({ $connected } von { $needed } verbunden)
farm-ctl-phase-preparing = Auftrag wird vorbereitet
farm-ctl-phase-rendering = Rendert
farm-ctl-phase-paused = Pausiert
farm-ctl-phase-complete = Fertig
farm-ctl-phase-stopped = Gestoppt
farm-ctl-phase-ended = Beendet

## Render on farm: the words after the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-detail-waiting = Wartet auf Render-Clients: { $connected } von { $needed } verbunden
farm-ctl-detail-preparing = Die Normalisierungsanker werden auf diesem Rechner gemessen, einmal für alle Clients
farm-ctl-detail-no-space = Wartet auf Platz im Ausgabeordner
farm-ctl-detail-paused = Pausiert: Es wird keine neue Arbeit verteilt
farm-ctl-detail-not-rendered = Beendet, mit nicht gerenderten Bildern — erneut mit demselben Ordner ausführen, um fortzusetzen
farm-ctl-detail-stopped = Gestoppt — erneut mit demselben Ordner ausführen, um fortzusetzen
# $code: the controller process's exit code.
farm-ctl-detail-ended = Beendet (Exit-Code { $code })
farm-ctl-detail-finishing = Wird abgeschlossen

## Render on farm: progress

# On the progress bar. $done: frames done; $total: frames in the job.
farm-ctl-progress =
    { $total ->
        [one] { $done } / { $total } Bild
       *[other] { $done } / { $total } Bilder
    }
# Added to the progress bar. $rate: frames per second, with decimals.
farm-ctl-progress-rate = { $rate } Bilder/s
# Added to the progress bar. $time: the time still to go ("38m 10s").
farm-ctl-progress-eta = noch etwa { $time }
# Facts under the progress bar, joined with " · ". $received and $sent: kilobytes per second.
farm-ctl-io = ein { $received } KB/s · aus { $sent } KB/s
# $space: free space on the output drive ("412 GB").
farm-ctl-free = { $space } frei
# $time: the time since the job started ("38m 10s").
farm-ctl-elapsed = { $time } vergangen
# $count: frames given up on.
farm-ctl-failed = { $count } nicht gerendert
# Reference orbits. $built: built for this job; $cached: found in the cache; $reused: reused between frames; $shared: passed between machines.
farm-ctl-refs = Referenzen: { $built } berechnet, { $cached } aus dem Cache, { $reused } wiederverwendet · { $shared } geteilt
# After a warning sign.
farm-ctl-storage-low = Der Ausgabeordner ist fast voll: Bis wieder Platz ist, wird keine neue Arbeit verteilt.
# After a warning sign.
farm-ctl-gpus-differ = Die GPUs in dieser Farm unterscheiden sich. Ihre Bilder weichen leicht voneinander ab, was in einer stehenden Einstellung sichtbar werden kann.
# After a warning sign. $count: the GPU classes in the farm.
farm-ctl-gpus-differ-classes = Die GPUs in dieser Farm unterscheiden sich ({ $count } GPU-Klassen: verschiedene GPUs, Treiber oder Proberenderings). Ihre Bilder weichen leicht voneinander ab, was in einer stehenden Einstellung sichtbar werden kann.
# $gpu: this machine's GPU, API and driver.
farm-ctl-this-gpu = Dieser Rechner: { $gpu }
# A machine whose GPU differs. $name: the machine; $note: how its GPU differs (in English); $adapter: its GPU.
farm-ctl-gpu-row = { $name }: { $note } — { $adapter }
# The same with its driver. $driver: the driver's name and version.
farm-ctl-gpu-row-driver = { $name }: { $note } — { $adapter }, Treiber { $driver }

## Render on farm: the table of machines

# Column headers (short: the table is narrow).
farm-ctl-col-machine = Rechner
farm-ctl-col-state = Status
farm-ctl-col-gpu = GPU
farm-ctl-col-frames = Bilder
farm-ctl-col-per-frame = Pro Bild
# How the machine delivers its frames (a speed, or "shared drive").
farm-ctl-col-link = Verbindung
# When the controller last heard from the machine.
farm-ctl-col-heard = Zuletzt
# In the machine's tooltip. $count: its frames that failed verification.
farm-ctl-strikes =
    { $count ->
        [one] { $count } Bild hat die Prüfung nicht bestanden
       *[other] { $count } Bilder haben die Prüfung nicht bestanden
    }
# In the Link column: it delivers its frames through the shared drive.
farm-ctl-link-share = Freigabe
# In the Heard column. $seconds: how long ago.
farm-ctl-heard-ago = vor { $seconds } s
farm-ctl-readmit = Wieder zulassen
farm-ctl-readmit-hover = Diesen Rechner wieder beitreten lassen, wenn er sich erneut verbindet
farm-ctl-remove = Entfernen
farm-ctl-remove-hover = Diesen Rechner aus dem Auftrag nehmen; seine Bilder gehen an die anderen
# The GPU column's tooltip.
farm-ctl-probe-same = Sein Proberendering ist mit dem dieses Rechners identisch.
# $count: the pixels that differ.
farm-ctl-probe-differs =
    { $count ->
        [one] Sein Proberendering weicht in { $count } Pixel von dem dieses Rechners ab: Seine Bilder werden leicht von denen dieses Rechners abweichen.
       *[other] Sein Proberendering weicht in { $count } Pixeln von dem dieses Rechners ab: Seine Bilder werden leicht von denen dieses Rechners abweichen.
    }
farm-ctl-probe-unknown = Sein Proberendering wurde noch nicht verglichen.
# $driver: the driver's name and version.
farm-ctl-gpu-driver = Treiber { $driver }
# After a warning sign. $note: how its GPU differs (in English: "a different GPU").
farm-ctl-gpu-note = { $note } (gegenüber diesem Rechner)

## Render on farm: a machine's state, in the State column

# $count: its frames that failed verification.
farm-ctl-state-removed-bad =
    { $count ->
        [one] entfernt: { $count } fehlerhaftes Bild
       *[other] entfernt: { $count } fehlerhafte Bilder
    }
farm-ctl-state-removed = von Ihnen entfernt
farm-ctl-state-removing = wird entfernt — Diagnosedaten werden gesammelt
farm-ctl-state-gone = weg
farm-ctl-state-leaving = verlässt die Farm
farm-ctl-state-checking = prüft (Selbstprüfung)
farm-ctl-state-paused = von seinem Benutzer pausiert
farm-ctl-state-parked = geparkt: zweimal abgestürzt — tritt bei erneuter Verbindung wieder bei
farm-ctl-state-refused = hat diesen Auftrag abgelehnt (eigene Grenzen)
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-ctl-state-rendering = rendert { $frame } ({ $index }/{ $count })
farm-ctl-state-starting = beginnt eine Zuteilung
farm-ctl-state-idle = untätig
farm-ctl-state-admitted = zugelassen — wartet auf den Auftrag

## Render on farm: the buttons

# A collapsible section: the controller's messages.
farm-ctl-events = Ereignisse
farm-ctl-resume = Fortsetzen
farm-ctl-pause = Pausieren
farm-ctl-pause-hover = Keine neue Arbeit verteilen; laufende Bilder werden fertiggestellt
farm-ctl-live-paused = Live-Ansicht pausiert, während dieser Rechner für die Farm rendert.
farm-ctl-close = Schließen
farm-ctl-close-active-hover = Das Fenster schließen; der Auftrag läuft weiter (über das Menü „Extras“ wieder öffnen)
farm-ctl-close-hover = Das Fenster schließen
farm-ctl-copy-command = Befehl kopieren
farm-ctl-copy-command-hover = Die entsprechende Befehlszeile kopieren
farm-ctl-stop = Stoppen
# The Stop button once a stop has been asked for and the job has not ended after 10 seconds.
farm-ctl-stop-now = Sofort stoppen
farm-ctl-stop-hover = Den Auftrag mit den vorhandenen Bildern beenden; ein erneuter Start desselben Auftrags setzt ihn fort
farm-ctl-render = Rendern
farm-ctl-render-hover = Auf Clients lauschen und die Tour auf ihnen rendern

## The Render tour window (Tools ▸ Render tour…)

tour-render-title = Tour rendern
tour-render-out = Ausgabeordner
tour-render-browse = Durchsuchen…
tour-render-out-hover = Vollständiger Pfad, in den die Bilder geschrieben werden.
tour-render-prefix = Bildpräfix
# <prefix> stands for the prefix typed in the field.
tour-render-prefix-hover = Bilder werden als <Präfix>_00000.png geschrieben
tour-render-size = Größe
tour-render-size-custom = Eigene…
# The size list's choice when the size is typed in. $width and $height: pixels.
tour-render-size-custom-value = Eigene — { $width }×{ $height }
tour-render-fps = Bilder / Sekunde
tour-render-fps-hover = Weniger als 1 fps ist zulässig: Einen langen Tauchgang mit 0,25 fps abzutasten ist der Weg, die tiefen Regressionshalte zu prüfen, ohne Tausende von Bildern zu rendern.
tour-render-ss = Supersampling
tour-render-chapter = Kapitel
tour-render-whole = Ganze Tour
tour-render-chapter-hover = Beim Rendern eines einzelnen Kapitels bleibt die GLOBALE Bildnummerierung erhalten, sodass seine Bilder direkt in eine vollständige Folge zurückpassen.
tour-render-mp4 = mp4 zusammenfügen
tour-render-mp4-hover = Führt anschließend ffmpeg aus (muss im PATH liegen). Die PNG-Bilder bleiben erhalten.
tour-render-overwrite = Vorhandene Bilder überschreiben
tour-render-overwrite-hover = Aus: Das Rendern hält an, um nachzufragen, was es von hier aus nicht kann.
tour-render-resume = Unterbrochenes Rendern fortsetzen
tour-render-resume-hover = Die bereits im Ausgabeordner vorhandenen Bilder behalten und nur die fehlenden rendern. Das neueste Bild wird zuerst geprüft und verworfen, wenn es unvollständig ist (das Bild, bei dem ein Rendern abbricht, ist das, das es gerade schrieb), wobei schrittweise zurückgegangen wird, bis ein gutes gefunden ist; ein Ordner mit Bildern in einer anderen Größe wird abgelehnt, statt in diese Folge gemischt zu werden.
# $count: this machine's graphics cards.
tour-render-all-gpus = Alle Grafikkarten verwenden ({ $count })
tour-render-all-gpus-hover = Auf allen Grafikkarten dieses Rechners zugleich rendern, über eine Renderfarm auf diesem Computer (--gpus all). Die Bilder, ihre Namen, „Fortsetzen“ und das mp4 sind wie bei einer Karte; die ganze Tour wird gerendert, in der eigenen Reihenfolge der Farm. Verschiedene Kartenmodelle zeichnen einige Pixel geringfügig anders: Eine stehende Einstellung und eine Überblendung bleiben auf einer Karte.
tour-render-order = Render-Reihenfolge
tour-render-sequential = Sequenziell
tour-render-sequential-hover = Bild 0, 1, 2, … der Reihe nach (Standard).
tour-render-progressive = Progressiv
tour-render-progressive-hover = Zuerst die ganze Tour als Vorschau, dann bis zum vollen Detail verfeinern: Die Keyframes werden zuerst gerendert, dann werden die größten zeitlichen Lücken halbiert, bis jedes Bild existiert. Die Bilder landen so oder so an ihren richtigen Indizes, sodass „Fortsetzen“ und das mp4 unverändert funktionieren — man kann früh anhalten, das grobe Daumenkino begutachten und mit „Fortsetzen“ in beliebiger Reihenfolge neu starten.
# The cost of the render. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
tour-render-summary =
    { $count ->
        [one] { $count } Bild · { $seconds } s Tour · { $width }×{ $height } ss{ $ss }
       *[other] { $count } Bilder · { $seconds } s Tour · { $width }×{ $height } ss{ $ss }
    }
# The disk space the frames will take (an estimate). $size: a size ("5.3 GB").
tour-render-disk = ≈{ $size } an Bildern
# $size: the frames' estimated size; $free: the free space on that drive ("6.0 GB").
tour-render-disk-free = ≈{ $size } an Bildern · { $free } frei
# After a warning sign. $disk: the line above ("≈51.0 GB of frames · 6.0 GB free").
tour-render-disk-short = { $disk } — nicht genug Platz für die ganze Folge
tour-render-separate = Rendert das Skript VON DER FESTPLATTE in einem eigenen Prozess, daher sind ungespeicherte Änderungen nicht enthalten — und ein dort verlorenes GPU-Gerät kann diese Sitzung nicht mitreißen.
# On the progress bar. $done: frames done; $total: frames to render.
tour-render-progress =
    { $total ->
        [one] { $done } / { $total } Bild
       *[other] { $done } / { $total } Bilder
    }
# Before the render's first progress line.
tour-render-starting = wird gestartet…
tour-render-live-paused = Live-Ansicht pausiert — das Rendern hat die GPU für sich allein, wird daher schneller fertig und misst die eigenen Bildkosten genau.
tour-render-cancel = Abbrechen
tour-render-cancel-hover = Schließen, ohne zu rendern
tour-render-copy-command = Befehl kopieren
tour-render-copy-command-hover = Die entsprechende Befehlszeile in die Zwischenablage kopieren
tour-render-stop = Rendern stoppen
tour-render-render = Rendern
tour-render-render-hover = Das Rendern der Bildfolge starten
# $folder: the output folder.
tour-render-finished = Rendern abgeschlossen → { $folder }
# $why: the render's own reason (in English).
tour-render-failed = Rendern fehlgeschlagen: { $why }
# $last: the render's last progress line (in English), or nothing.
tour-render-failed-or-stopped = Rendern fehlgeschlagen oder gestoppt. { $last }
# $why: the system's reason.
tour-render-no-exe = Programmdatei nicht gefunden: { $why }
# $why: the system's reason.
tour-render-start-failed = Das Rendern konnte nicht gestartet werden: { $why }

## The Custom formula dialog (Fractal ▸ Custom formula…)
## The formula language itself stays as it is in every language: function names (sin, cosh, conj…),
## operators, variables (z, c, p1), and the formulas. Keep them as written inside these messages.

# The window's title.
formula-dialog-title = Eigene Formel
# A small line at the top of the window, above the formula.
formula-dialog-intro = Der Schritt, in Ausdrücken im Fractint-Stil
# The two ways to show and edit the formula (top right of the window).
formula-dialog-textbook = Lehrbuch
formula-dialog-textbook-hover = Die Formel gesetzt anzeigen, wie ein Lehrbuch sie setzen würde, und sie so bearbeiten
formula-dialog-text = Text
formula-dialog-text-hover = Die Formel als Text bearbeiten
# The tooltip of the "LaTeX" button (the button's own text stays LaTeX).
formula-dialog-latex-hover = Die Formel als LaTeX kopieren, so wie sie hier gesetzt ist
# The drop-down list of example formulas (the collection).
formula-dialog-examples = Beispiele
# Help under the formula in Textbook mode. The characters / ^ ( are what the user types: keep them.
formula-dialog-help-textbook = Eingabe wie im Text: / macht aus dem Term davor einen Bruch, ^ einen Exponenten, ( Klammern. Die Pfeiltasten bewegen sich durch die Formel; Tab springt zum nächsten leeren Feld. Eine rote Zeile lässt sich nicht lesen: Ein Klick darauf bearbeitet sie als Text.
# Help under the formula in Text mode. "; starts a comment": the ; is what the user types.
formula-dialog-help-text = Tippen oder das Tastenfeld verwenden — es enthält jeden Namen, den die Formelsprache kennt. Namen werden beim Tippen ergänzt (Tab). Anweisungen werden durch einen Zeilenumbruch oder ein Komma getrennt; ; beginnt einen Kommentar.
# The syntax check, when the formula reads.
formula-dialog-reads = Korrekt lesbar.
# The syntax check in Textbook mode, when the formula has an empty box to fill in.
formula-dialog-empty-box = Das leere Feld ausfüllen.
# The syntax check in Textbook mode. $line: the line number; $message: what is wrong (from the formula parser, in English).
formula-dialog-error-line = Zeile { $line }: { $message }
# One function written with another notation's name, and the language's name for it: "ln to log".
# $from: the name as written (ln); $to: the formula language's name (log).
formula-respell-pair = { $from } zu { $to }
# A button that rewrites them. $changes: one or more formula-respell-pair, joined with ", ".
formula-respell = { $changes } ändern
formula-respell-hover = Diese Funktionen so schreiben, wie die Formelsprache sie nennt
# Shown when Apply finds a parameter field that is not a number. $name: the parameter (p1 … p5).
formula-param-not-numbers = { $name } ist kein Zahlenpaar
# The Apply button (also in the Formula library window).
formula-apply = Anwenden
formula-dialog-apply-hover = Die Formel kompilieren und anzeigen
# Shown beside Apply while the formula is being compiled.
formula-dialog-compiling = Die Formel wird für die GPU kompiliert…
# The greyed hint in an empty name box (also in the Formula library window).
formula-name-hint = Name (optional)
# The save button: "Update" when the library already has a formula of that name.
formula-dialog-update-in-library = In Bibliothek aktualisieren
formula-dialog-save-to-library = In Bibliothek speichern
formula-dialog-save-hover = Diese Formel und ihre Parameter in der Formelbibliothek aufbewahren
formula-dialog-save-disabled-hover = Nur eine korrekt lesbare Formel kann gespeichert werden
# Opens the Formula library window.
formula-dialog-library = Bibliothek…
formula-dialog-library-hover = Die gespeicherten Formeln: anwenden, bearbeiten, importieren und exportieren

## The Custom formula dialog's keypad
## The keys' captions are formula tokens and stay as they are; these are their tooltips.

# The keypad's third tab (the other two are "123" and "f(z)"). A short word.
formula-keypad-tab-names = Namen
# The tabs' tooltips.
formula-keypad-tab-basic-hint = Zahlen, Operatoren und die meistverwendeten Namen
formula-keypad-tab-functions-hint = Funktionen
formula-keypad-tab-names-hint = Alle Variablen, Parameter und Konstanten sowie die Syntax der Anweisungen
# The key z.
formula-key-z = Der Iterationswert
# The key c.
formula-key-c = Das Pixel (die Julia-Konstante im Julia-Modus)
# The key p1 on the first tab.
formula-key-p1-below = Parameter 1 — in den Feldern unten gesetzt
# The keys p1 … p5. $n: the parameter's number.
formula-key-param = Parameter { $n }
formula-key-multiply = Multiplizieren
formula-key-divide = Dividieren
# The key that types ^2.
formula-key-square-after = Quadrat (nach einem Wert: z^2)
# The key that types ^.
formula-key-power = Potenz: ganzzahliger, reeller oder komplexer Exponent
formula-key-square-root = Quadratwurzel
# The key that types |…|: in this language it is the squared modulus, as in Fractint.
formula-key-squared-modulus = QUADRIERTER Betrag |z|² — wie in Fractint
formula-key-add = Addieren
formula-key-subtract = Subtrahieren (oder negieren)
# The key that types =.
formula-key-assign = Zuweisen: z = … oder eine Hilfsvariable t = …
# The key that starts a new line (first tab).
formula-key-next-statement-comma = Nächste Anweisung (ein Komma geht auch)
# The key "del".
formula-key-backspace = Das Zeichen vor dem Cursor löschen (oder die Auswahl)
# The key e.
formula-key-euler = Eulersche Zahl
# The key that types (0.5, 0.5).
formula-key-complex-constant = Eine komplexe Konstante: (Realteil, Imaginärteil)
# The key that types a comma.
formula-key-comma = Trennt Anweisungen oder die Teile einer komplexen Konstante
formula-key-left = Cursor nach links
formula-key-right = Cursor nach rechts
# The function keys (the f(z) tab), one per function.
formula-key-sin = Sinus
formula-key-cos = Kosinus
formula-key-tan = Tangens
formula-key-cotan = Kotangens
formula-key-exp = Exponentialfunktion e^z
formula-key-sinh = Sinus hyperbolicus
formula-key-cosh = Kosinus hyperbolicus
formula-key-tanh = Tangens hyperbolicus
formula-key-cotanh = Kotangens hyperbolicus
formula-key-log = Natürlicher Logarithmus, ln (Hauptzweig)
formula-key-sqr = Quadrat: sqr(z) = z²
formula-key-sqrt = Quadratwurzel (Hauptwert)
formula-key-recip = Kehrwert 1/z
formula-key-conj = Komplex konjugiert
formula-key-flip = Real- und Imaginärteil vertauschen
formula-key-abs = Betrag BEIDER Teile: |x| + i|y|
formula-key-cabs = Betrag |z| (eine reelle Zahl)
formula-key-real = Realteil
formula-key-imag = Imaginärteil, als reelle Zahl
# "fn" is Fractint's name for a function slot: keep it.
formula-key-ident = Identität (Fractints fn-Platzhalter)
# The key "pixel".
formula-key-pixel = Dasselbe wie c (Fractints Name)
# The key "z =".
formula-key-set-z = Eine Anweisung beginnen, die z setzt
# The key "t =".
formula-key-temporary = Eine Hilfsvariable: Jeder neue Name geht (t, w, a2, …)
# The key "; …".
formula-key-comment = Ein Kommentar, bis zum Zeilenende
# The key that starts a new line (names tab).
formula-key-next-statement = Nächste Anweisung

## The formula's completion list (names offered as you type)

# Beside a name the formula itself assigns.
formula-completion-variable = Eine Variable, die diese Formel setzt
# Beside a function typed with another notation's name.
# $what: what the function is, in English (from the formula parser: "The natural logarithm");
# $other: the other notation's name (ln); $ours: the formula language's name (log).
formula-completion-other-spelling = { $what } — { $other } heißt hier { $ours }
# Under a list cut short. $count: the names not shown.
formula-completion-more = … { $count } weitere
# The last line of the list. Tab, Enter and Esc are key names.
formula-completion-keys = Tab oder Enter ergänzt, Esc schließt

## The Custom formula dialog's Textbook mode

# The tooltip of a line that does not read. $why: what is wrong (from the formula parser, in English, or formula-textbook-fractint-sections).
formula-textbook-unread-hover = Nicht lesbar: { $why }. Zum Bearbeiten als Text klicken.
# Why a line with Fractint's sections is shown as text in Textbook mode.
formula-textbook-fractint-sections = Fractint-Abschnitte (das ':' eines Init-Abschnitts, Vergleiche, a = b = …) werden als Text bearbeitet

## The Formula library window (Fractal ▸ Formula library…)

# The window's title.
formula-library-title = Formelbibliothek
# The window's two tabs. $count: the formulas in each.
formula-library-mine = Meine Formeln ({ $count })
formula-library-mine-hover = Die selbst gespeicherten oder importierten Formeln
formula-library-collection = Sammlung ({ $count })
formula-library-collection-hover = Formeln, die mit Fractadyne kommen, jede mit einer Ausgangsansicht
# A small marker before the name of the formula the view shows now. A short word.
formula-library-showing = angezeigt
# The tooltip of Apply on a formula of the collection.
formula-library-apply-shipped-hover = Anzeigen, in ihrer Ausgangsansicht
# A row's Edit button (after a pencil icon).
formula-library-edit = Bearbeiten
formula-library-edit-hover = Im Dialog „Eigene Formel“ öffnen, ohne sie anzuwenden
# A collection row's button that copies the formula into "My formulas".
formula-library-copy-to-mine = Zu meinen kopieren
formula-library-copy-to-mine-hover = Zu den eigenen Formeln hinzufügen, um sie zu behalten oder zu ändern
formula-library-close = Schließen
# Saves the formula the view shows, under the name typed beside it.
formula-library-add-current = Aktuelle Formel hinzufügen
formula-library-add-current-hover = Die Formel speichern, die die Ansicht zeigt
# "Fractal > Custom formula…" is the menu path: write it as the menus are named.
formula-library-add-current-disabled-hover = Zuerst eine eigene Formel anwenden (Fraktal > Eigene Formel…)
formula-library-import = Importieren…
formula-library-import-hover = Die Formeln einer Formeldatei hinzufügen; hier wird nichts ersetzt
formula-library-export-all = Alle exportieren…
formula-library-export-all-hover = Alle Formeln hier in eine Datei schreiben, zum Teilen oder Aufbewahren
# The filter box: its label, and the greyed hint inside it.
formula-library-filter = Filter
formula-library-filter-hint = nach Name oder Text filtern
# Shown while the user has no saved formulas.
formula-library-empty = Noch keine gespeicherten Formeln. Eine aus dem Dialog „Eigene Formel“ speichern, die gerade angezeigte hinzufügen oder eine Datei importieren (unsere oder Fractints .frm).
# The tooltip of Apply on one of the user's formulas: with a view saved, and without one.
formula-library-apply-view-hover = Diese Formel anzeigen, in der mit ihr gespeicherten Ansicht
formula-library-apply-hover = Diese Formel anzeigen
# The tooltip of a row's save icon.
formula-library-export-one-hover = Diese Formel in eine Datei exportieren
# A row asks before deleting: the question, then its two buttons. "Delete" is also the delete icon's tooltip.
formula-library-delete-ask = Löschen?
formula-library-delete = Löschen
formula-library-keep = Behalten
# Under a list cut short. $count: the formulas not listed.
formula-library-more = …und { $count } weitere: Im Filter tippen, um eine zu finden.
formula-library-no-match = Keine passt zum Filter.
# Under a saved formula this version cannot read. $why: what is wrong (from the formula parser, in English).
formula-library-unread = In dieser Version nicht lesbar: { $why }

## Formula library notices

# $why: the system's reason, in English.
formula-library-save-failed = Die Formelbibliothek konnte nicht gespeichert werden: { $why }
formula-library-nothing-to-save = Nichts zu speichern: Die Formel ist leer.
# $name: the formula's name in the library.
formula-library-updated = „{ $name }“ in der Formelbibliothek aktualisiert.
formula-library-saved = „{ $name }“ in der Formelbibliothek gespeichert.
# $name: the formula's name; $why: what is wrong.
formula-library-apply-failed = „{ $name }“ lässt sich nicht anwenden: { $why }
# Copying a formula of the collection into "My formulas". $name: its name; $as_named: the name it was given (the same, or "Name (2)" when the name was taken).
formula-library-copy-exists = „{ $name }“ ist bereits unter den eigenen Formeln.
formula-library-copied = „{ $name }“ als „{ $as_named }“ zu den eigenen Formeln kopiert.
# $name: the formula's name.
formula-library-deleted = „{ $name }“ aus der Formelbibliothek gelöscht.
# Why the library could not be saved (follows formula-library-save-failed).
formula-library-no-config-dir = kein Konfigurationsordner
# At startup, when the library file cannot be read. $why: the reason, in English; $path: a file's full path.
formula-library-unreadable-kept = Die Formelbibliothek konnte nicht gelesen werden ({ $why }). Sie wurde als { $path } aufbewahrt, und eine neue, leere Bibliothek wurde begonnen.
formula-library-unreadable-stuck = Die Formelbibliothek konnte weder gelesen ({ $why }) noch beiseitegelegt werden: { $path } an einen sicheren Ort kopieren, bevor eine Formel gespeichert wird, denn das ersetzt die Datei.

## Formula library: Import and Export
## A formula file's contents stay in English; this is what the windows and notices around them say.

# File-type names in the open and save dialogs (the extensions are added by the system).
formula-filter-formula-files = Formeldateien
formula-filter-fractadyne = Fractadyne-Formeln
formula-filter-fractint = Fractint-Formeln
formula-filter-all-files = Alle Dateien
# Why a file cannot be imported (they follow formula-import-failed).
formula-import-too-large = viel zu groß für eine Formeldatei
formula-import-not-text = keine Textdatei
formula-import-no-formulas = keine Formeln darin
# $why: the TOML reader's message, in English.
formula-file-not-formulas = keine Formeldatei ({ $why })
# $format: what the file says it is ("fractadyne-gradients").
formula-file-other-format = eine „{ $format }“-Datei, keine Formeldatei
# "[[formula]]" is the file's own syntax: keep it.
formula-file-no-formulas = keine Formeln darin (eine Formeldatei führt sie als [[formula]]-Tabellen auf)
# $file: the file's name; $why: one of the reasons above, or the system's (in English).
formula-import-failed = „{ $file }“ konnte nicht importiert werden: { $why }.
# After an import that added nothing. $file: the file's name.
formula-import-nothing-new = Nichts Neues in „{ $file }“.
# After an import. $count: the formulas added; $file: the file's name.
formula-import-added =
    { $count ->
        [one] { $count } Formel aus „{ $file }“ importiert.
       *[other] { $count } Formeln aus „{ $file }“ importiert.
    }
# The same two, followed by what else happened. $notes: one or more of the formula-import-note-… messages below, joined with "; ".
formula-import-nothing-new-notes = Nichts Neues in „{ $file }“: { $notes }.
formula-import-added-notes =
    { $count ->
        [one] { $count } Formel aus „{ $file }“ importiert: { $notes }.
       *[other] { $count } Formeln aus „{ $file }“ importiert: { $notes }.
    }
# Notes (clauses, no final full stop). $count: the formulas already in the library, skipped.
formula-import-note-duplicates =
    { $count ->
        [one] { $count } Formel war bereits in der Bibliothek
       *[other] { $count } Formeln waren bereits in der Bibliothek
    }
# One formula was renamed because its name was taken. $from: its name in the file; $to: the name it was given ("Name (2)").
formula-import-note-renamed-one = „{ $from }“ war vergeben, daher heißt sie „{ $to }“
# $count: the formulas renamed because their names were taken (2 or more).
formula-import-note-renamed = { $count } umbenannt, weil der Name vergeben war
# $count: the entries skipped as empty or too long.
formula-import-note-skipped =
    { $count ->
        [one] { $count } Eintrag war leer oder zu lang, übersprungen
       *[other] { $count } Einträge waren leer oder zu lang, übersprungen
    }
# A second sentence after importing a Fractint .frm file. $count: the entries that do not read; $why: the commonest reason (from the formula parser, in English).
formula-import-frm-unread =
    { $count ->
        [one] { $count } Eintrag ist in dieser Version nicht lesbar (meist: { $why }).
       *[other] { $count } Einträge sind in dieser Version nicht lesbar (meist: { $why }).
    }
# After exporting one formula. $name: the formula's name; $file: the file's name.
formula-export-one = „{ $name }“ nach „{ $file }“ exportiert.
# After exporting several. $count: how many; $file: the file's name.
formula-export-many =
    { $count ->
        [one] { $count } Formel nach „{ $file }“ exportiert.
       *[other] { $count } Formeln nach „{ $file }“ exportiert.
    }
# $file: the file's name; $why: the system's reason, in English.
formula-export-failed = „{ $file }“ konnte nicht geschrieben werden: { $why }

## The control panel (right-hand side): header and section headings

# The panel's title, beside the button that hides it.
panel-title = Bedienfeld
panel-hide-hover = Bedienfeld ausblenden
# The button that brings the hidden panel back (top right of the view).
panel-show-hover = Bedienfeld einblenden
# Section headings.
panel-section-navigate = Navigation
panel-section-coloring = Färbung
panel-section-quality = Qualität
panel-section-effects = Effekte
panel-section-overlays = Einblendungen
panel-section-advanced = Erweitert
# The last section: reference notes about the current fractal. $name: the fractal's name (Mandelbrot, Burning Ship…).
panel-section-about = Über { $name }
panel-section-performance = Leistung

## The control panel: Navigate
## Labels before a control sit in a column about 104 px wide (roughly 15 characters): a longer label pushes its control to the right.

panel-zoom-speed = Zoomtempo
panel-zoom-speed-hover = Tempo des Dauerzooms bei gehaltener Space-Taste (1× ≈ 2× pro 1,5 s).
panel-snap-zoom = Zoom an Merkmalen einrasten
panel-snap-zoom-hover = Beim Hineinzoomen mit dem Mausrad oder Space den Zoom zu einem Minibrot (oder zur reellen Achse) nahe dem Mauszeiger ziehen, sodass es unter den Mauszeiger gleitet und der Zoom darauf landet. Je näher das Merkmal, desto stärker der Zug. Nur Mandelbrot, Einzelansicht.
panel-click-zoom = Klick-Zoom
panel-click-zoom-hover = Wenn aktiv, taucht ein Linksklick in die Ansicht um den Faktor unten ein (Rechtsklick zoomt zurück), zentriert auf den angeklickten Punkt. Ziehen verschiebt weiterhin; Shift+Ziehen / Ziehen mit der rechten Taste zoomt weiterhin per Rahmen. Backspace macht einen Klick rückgängig. In der Doppelansicht funktioniert es in beiden Hälften, und das Festsetzen des Julia-c wechselt zu Ctrl+Klick, solange dies aktiv ist. Shift gedrückt halten, um unter dem Mauszeiger eine Lupe einzublenden, die die Ansicht zeigt, in der der Klick landen würde, und bei gehaltener Taste klicken, um diesen Punkt zu übernehmen; Shift+ZIEHEN zoomt weiterhin per Rahmen.
# The click-to-zoom factor, followed by buttons 2×, 4×, 10×…
panel-click-zoom-factor = Faktor
panel-click-zoom-snap = Auf nächstes Zentrum einrasten
panel-click-zoom-snap-hover = Nach dem Sprung den nächsten Minibrot-Kern berechnen und exakt darauf einrasten, sodass man auf dem Merkmal landet statt daneben. Der Zoom wird nicht verändert — den hat der Klick gewählt. Nur Mandelbrot. Die Berechnung läuft im Hintergrund: Die Ansicht springt sofort und korrigiert sich, wenn die Antwort eintrifft, was in extremer Tiefe einige Sekunden dauern kann.

## The control panel: Coloring
## Labels before a control sit in a column about 104 px wide (roughly 15 characters).

panel-method = Methode
panel-method-hover = Wie die Fluchtdaten zu Farbe werden. Streifenmittel / Dreiecksungleichung / Orbitfalle / Zerlegung zeigen die Struktur des Orbits; Distanzschätzung schattiert nach Nähe zum Rand.
panel-stripe-density = Streifendichte
panel-stripe-tail = Nur Ende (Tiefenzoom)
panel-stripe-tail-hover = Den Streifenterm über die letzten N Iterationen jedes Orbits mitteln (ein exponentielles Fenster) statt über den ganzen Orbit. In extremer Tiefe folgt jeder Orbit der Referenz bis auf seine letzten paar hundert Iterationen, sodass der Mittelwert über den ganzen Orbit eine einzige Farbe für die ganze Ansicht ist; das Ende bewahrt den Kontrast. Orbits, die kürzer als das Fenster sind, bleiben unberührt. Auch --stripe-tail N für --render.
panel-tail-length = Endlänge
# The unit after the tail length's number: short for "iterations" (the slider reads "64 it").
panel-tail-length-unit = it
panel-tail-length-hover = Fensterlänge in Iterationen. Kurze Fenster (8–32) ergeben in jeder Tiefe die klassischen kräftigen Bänder; längere mitteln wieder zu einer flachen Farbe hin, da sich der Streifenterm eines langen chaotischen Orbits herausmittelt.
panel-trap-shape = Fallenform
panel-palette = Palette
# The two colours of the Binary palette, each beside its colour swatch.
panel-in-set = In Menge
panel-out-of-set = Außerhalb
# The two colours of the Duotone palette, each beside its colour swatch.
panel-shadow = Schatten
panel-highlight = Lichter
panel-cycle = Zyklus
panel-offset = Versatz
panel-normalize-hover = In extremer Tiefe umfassen die Fluchtzahlen Hunderttausende, und ein fester Zyklus bricht die Palette tausendfach zwischen benachbarten Pixeln um — ein korrektes dichtes Feld erscheint als Gesprenkel. Dies bildet die Palette auf den gemessenen Fluchtbereich der Ansicht ab (Zyklus legt dann fest, wie viele Palettendurchläufe ihn überspannen), WENN die Ansicht tatsächlich Aliasing zeigt — eine glatte Ansicht bleibt unverändert, denn eine Umrechnung würde das Bild umsonst verändern. „Bereich immer anpassen“ unten ankreuzen, um trotzdem umzurechnen. Nur bei der Methode „Glatte Iteration“; entspricht --normalize.
# A checkbox indented under "Normalize deep colors" (also in the Color menu).
panel-fit-range = Bereich immer anpassen
panel-fit-range-hover =
    Die Palette auf den gemessenen Fluchtbereich der Ansicht umrechnen, auch wenn die Ansicht kein Aliasing zeigt — das Kästchen darüber rechnet nur um, wenn ein dichtes Feld sonst als Gesprenkel erscheinen würde.

    Dies verwenden, wenn die Palette das auf dem Bildschirm Sichtbare überspannen soll, statt Rauschen zu beheben. Es braucht trotzdem eine Messung, wirkt also in den ein, zwei Momenten, bevor die Ansicht zur Ruhe kommt, noch nicht, und bei einer Ansicht ohne entkommene Pixel gar nicht.
# A checkbox indented under "Always fit range" (also in the Color menu).
panel-log-scale = Log. Farbskala
panel-log-scale-hover =
    Die Palette nach dem LOGARITHMUS des Fluchtwerts statt linear verteilen. Fluchtzahlen drängen sich in der Tiefe am oberen Ende, sodass eine lineare Abbildung den Großteil der Palette auf eine dünne Schale nahe dem Rand verwendet und alles andere flach macht.

    Gilt für die normalisierte Abbildung — die „Tiefe Farben normalisieren“ (oder --normalize bei einem Export) einschaltet. Ein stark schiefer Bereich verwendet die logarithmische Abbildung ohnehin, ob dies angekreuzt ist oder nicht.
# The same checkbox's tooltip while it is greyed out (normalization is off).
panel-log-scale-off-hover =
    Erfordert Normalisierung: Dies wählt, WIE die Palette auf den gemessenen Fluchtbereich abgebildet wird; bei ausgeschaltetem „Tiefe Farben normalisieren“ (und ohne Export mit --normalize) gibt es also keine Abbildung, die es ändern könnte, und es würde kein einziges Pixel verändern.

    „Tiefe Farben normalisieren“ einschalten, um es zu verwenden.
# The palette animation: its kind, then its speed.
panel-animate = Animieren
panel-anim-speed = Tempo
panel-anim-speed-hover = Zyklustempo: Farbversatz-Zyklen/s oder (Zufällige Farbverläufe) Farbverlaufswechsel/s.
panel-shuffle-gradient = Farbverlauf mischen

## The control panel: Quality

panel-auto-iter = Iterationen automatisch mit Zoom skalieren
# The iteration-count label while auto-scaling is on (the count is then a base the zoom raises).
panel-iterations-base = Iterationen (Basis)
panel-iterations = Iterationen
panel-iterations-hover = Grundlegende Iterationszahl. Bei aktiver Auto-Skalierung steigt die effektive Zahl mit der Zoomtiefe. Während der Bewegung begrenzt die Vorschau die Iterationen niedrig (50.000), damit sie reaktionsschnell bleibt, und schärft dann auf die volle Zahl nach, sobald die Ansicht zur Ruhe kommt — so wirken tiefe Kanten während der Bewegung glatt und lösen sich beim Anhalten auf.
# A warning under the iteration count. $count: the iterations the settled preview renders.
panel-res-limited = In dieser Tiefe rendert die ruhende Vorschau { $count } Iterationen in verringerter Auflösung, um reaktionsschnell zu bleiben. Für volle Auflösung exportieren.
panel-anti-alias = Kantenglättung
# Anti-aliasing off (the other choices are 2×, 3×, 4×, 8×).
panel-anti-alias-off = Aus
panel-anti-alias-hover = Supersampling für Standbilder (angewandt, sobald die Ansicht zur Ruhe kommt). Höher bändigt den feinen „Staub“ im Äußeren, kostet aber Renderzeit.

## The control panel: Effects

panel-relief-lighting = 3D-Reliefbeleuchtung
panel-relief-lighting-hover = Die Oberfläche anhand der Steigung der Distanzschätzung schattieren — ein geprägter, beleuchteter Look. (Holomorphe Familien: Mandelbrot / Multibrot.)
panel-light-angle = Lichtwinkel
panel-relief = Relief
panel-relief-hover = Niedriger = schärferes Relief; höher = weicher/flacher.
panel-rotate-light = Licht drehen
panel-rotate-light-hover = Die Lichtrichtung mit der Zeit drehen (verwendet den Tempo-Regler).
panel-distance-glow = Distanzleuchten
panel-distance-glow-hover = Helle Konturbänder der Distanzschätzung, die sich nahe dem Rand zu leuchtenden Filamenten verdichten. (Holomorphe Familien.)
panel-glow = Leuchten
panel-band-width = Bandbreite
panel-band-width-hover = Abstand der Distanzkonturen (Oktaven pro Band).
panel-animate-glow = Leuchten animieren
panel-animate-glow-hover = Die Leuchtbänder mit der Zeit fließen lassen (verwendet den Tempo-Regler).

## The control panel: Overlays

panel-orbits-hover = Den Iterationspfad des Punktes unter dem Mauszeiger zeichnen.
panel-orbit-anim = Animieren (rasender Punkt)
panel-orbit-anim-hover = Einen farbwechselnden Punkt den Orbit entlang rasen lassen.
panel-orbit-speed = Orbittempo
panel-minimap-hover = Eine kleine Übersicht der Startansicht mit einer „Sie sind hier“-Markierung und der Zoomtiefe. Ein Klick darauf springt zu einem Bereich.

## The control panel: Advanced
## Labels before a control sit in a column about 104 px wide; most of these are already wider in English.

# Small headings of the section's two groups.
panel-accelerators = Beschleuniger
panel-performance-tuning = Leistungsoptimierung
panel-bla = BLA-Beschleunigung (Tiefenzoom)
panel-bla-hover = Bilineare Approximation: in extremer Tiefe Iterationen im gesamten Orbit überspringen (floatexp-Mandelbrot, ≥1e28×) — ~5× schnelleres GPU-Rendering, identische Ausgabe (vom Selbsttest geprüft). Standardmäßig an; zum Vergleichen ausschalten oder wenn ein Artefakt auftritt.
panel-series-approx = Reihennäherung
panel-series-approx-hover = Die Störungsrechnung mit einem Polynom vorbelegen, um frühe Iterationen zu überspringen, wo BLA nicht aktiv ist (df32-Tiefen, Multibrot, BLA aus). Identische Ausgabe; zum Vergleichen ausschalten.
panel-glitch-correct = Glitch-Korrektur (Export)
panel-glitch-correct-hover = Glitch-Korrektur mit mehreren Referenzen für exportierte Bilder: erkennt Glitches der Störungsrechnung und rendert diese Pixel gegen zusätzliche Referenzen neu, bis sie sauber sind. Standardmäßig an. Exporte bis ~32 MP (ohne Hilfsfärbung); größere Bilder und die Live-Ansicht verwenden den einfachen Pfad. Nie verwendet für Mandelbrot- oder Multibrot-Ansichten (außerhalb des Julia-Modus): Gegen eine exakte Berechnung geprüft, behob sie dort nichts und kostete bis zum 5-Fachen der Exportzeit.
panel-dive-limit = Auto-Zoom-Tauchgrenze
panel-dive-limit-hover = Tiefe, bei der der Auto-Zoom (Taste A) stoppt. Bis ~1e271× gleitet er sanft; tiefer wechselt er zu einem ruckartigen, gestuften Tauchgang, um extreme Tiefe schnell zu erreichen.
# Followed by two choices, Speed and Quality.
panel-autopilot-priority = Auto-Zoom-Priorität
panel-priority-speed-hover = Mit der eingestellten Geschwindigkeit zoomen. Der Bildschirm wird aktualisiert, sobald ein Bild mit Details bereit ist, in der Auflösung, die die Aktualisierungen am Laufen hält (unter der Untergrenze „Min. Bewegungsauflösung“, wenn der Zoom dem Rendern davonläuft); zwischen den Aktualisierungen folgt das letzte Bild dem Zoom und wird weicher.
panel-priority-quality-hover = Jede Aktualisierung ist ein vollständig aufgelöstes Bild in voller Auflösung, und der Zoom verlangsamt sich nach Bedarf, damit das Bild auf dem Schirm nie stark vergrößert wird, bevor das nächste eintrifft. Ein langsamerer Tauchgang, der in jedem Moment scharf ist.
# Followed by two choices, Detail and Misiurewicz point.
panel-autopilot-target = Auto-Zoom-Ziel
panel-target-detail-hover = In Richtung der reichsten Struktur am Rand des Fraktals in der Ansicht tauchen und das Ziel während des Tauchgangs laufend neu wählen.
panel-target-misiurewicz-hover = Den nächsten Misiurewicz-Punkt finden (ein Spiralzentrum, dessen Struktur sich endlos wiederholt) und exakt hineintauchen. Der Tauchgang hält zuerst still, während der Punkt bis zur Tiefe der Tauchgrenze berechnet wird — einen Moment bei üblichen Tiefen, länger jenseits von 1e1000×. Nur Mandelbrot-Menge.
panel-work-budget = Live-Render-Budget
panel-work-budget-hover = Detail oder Tempo für die Live-Ansicht bei tiefem Zoom. Höher rendert in vollerer Auflösung (schärfer — weniger vom „weichen“ hochskalierten Look), senkt aber die Bildrate, und sehr hohe Werte riskieren bei den schwersten Bildern einen kurzen GPU-Hänger. Exporte sind unabhängig davon immer in voller Auflösung.
panel-min-motion-res = Min. Bewegungsauflösung
panel-min-motion-res-hover = Die niedrigste Auflösung, auf die die Live-Ansicht beim fortlaufenden Zoomen oder Verschieben in tiefem Zoom fallen darf. Höher verhindert, dass ein schneller Tauchgang blockig/pixelig wird (schärfer), kostet aber Bildrate — bei 100 % müssen die Detailaktualisierungen eines tiefen Tauchgangs zu vollen Kosten rendern, was tiefes Zoomen sichtbar stufig/ruckelig machen kann. Wirkt ein tiefer Tauchgang ruckelig, diesen Wert senken. Standard 30 %; beeinflusst weder das ruhende Bild noch Exporte.
panel-finish-sound = Ton nach fertigem Rendern
panel-finish-sound-hover = FRACTINTs Abschlussmelodie spielen, wenn ein Export oder das Rendern einer Tour fertig ist: drei steigende Töne (1047, 1109, 1175 Hz für je 100 ms), genau so, wie das DOS-Original sie kodierte — synthetisiert als bandbegrenztes Rechtecksignal und hochpassgefiltert, damit es wie der kleine PC-Lautsprecher klingt, der sie spielte.
panel-prefer-detail = Details beim Zoomen bevorzugen
panel-prefer-detail-hover = Beim Zoomen oder Verschieben weiterhin das letzte vollständig detaillierte Bild zeigen — skaliert und der Bewegung nachgeführt —, statt in jedem Bild mit verringerter Qualität neu zu rendern. Die Ansicht rendert vollständig, sobald die Bewegung pausiert. Aus: Bewegung wird live in verringerter Auflösung gerendert (flüssiger, gröber). Flache Ansichten (direkter Modus) rendern immer live — sie sind in jedem Bild ohnehin billig und scharf.

## The control panel: About (the current fractal)

# A link to the fractal's reference page, followed by an arrow.
panel-about-reference = Referenz

## The control panel: Advanced ▸ Second graphics card

# A label before a drop-down (wider than the panel's label column already).
gpu2-label = Zweite Grafikkarte
gpu2-off = Aus
# Another device on the same graphics card as the window (the default).
gpu2-same-card = Dieselbe Karte
# A card the list does not name. $number: the card's number.
gpu2-card-n = Karte { $number }
gpu2-finding = Grafikkarten dieses Rechners werden gesucht…
gpu2-none = Dieser Rechner hat keine weitere Grafikkarte.
gpu2-hover =
    Ein weiteres Grafikgerät rendert Bilder einer tiefen Ansicht, während sie sich bewegt, und einige der zusätzlichen Samples, die sie nach dem Anhalten schärfen, sodass das Bild mit einem Zoom Schritt hält und seine endgültige Qualität früher erreicht. Die Karte, die das Fenster zeichnet, zeichnet weiterhin alles.

    Eine weitere Karte bringt ihr eigenes Tempo ein. Zwei verschiedene Kartenmodelle zeichnen einige Pixel geringfügig anders; fällt die zweite Karte aus, läuft die Ansicht mit einer weiter.

    „Dieselbe Karte“ (Standard) öffnet ein zweites Gerät auf dieser Karte. Es bringt kein Tempo, rendert aber, ohne auf die Bilder des Bildschirms zu warten. Es teilt sich die Karte: Ein Fehler, der die Karte zurücksetzt, beendet das Programm, wie es ein Fehler beim eigenen Rendern des Fensters tut, und beim nächsten Start wird angeboten, es auszuschalten.

## The view (canvas)

# A notice after Ctrl+click pins the Julia c at a deep zoom.
canvas-julia-pin-quantized = Julia-c festgesetzt — aber es wird in doppelter Genauigkeit gehalten, sodass in dieser Tiefe die ganze Hälfte auf einen einzigen Wert rundet und die Festsetzung auf der Ansichtsmitte landet. Zum Zielen herauszoomen.
# A small caption in the top-left corner of the view while the orbit overlay is fitted to the view (its "Normalize (fit to view)" option).
canvas-orbit-normalized = Orbit (normalisiert)
# The minimap's tooltip.
canvas-minimap-hover = Übersicht / Sie sind hier. Ziehen verschiebt beim aktuellen Zoom, Scrollen zoomt hinein und heraus, ein Klick springt zu diesem Bereich (Startzoom).

## The timestamp overlay (View ▸ Show timestamp)

# Under the large elapsed-time clock. $frame: the frame number (as the log writes it, without grouping).
overlay-timestamp-frame = Bild { $frame }

## File ▸ Settings ▸ Reference cache…

cache-title = Referenz-Cache
cache-intro = In extremer Tiefe ist der langsame Teil einer Ansicht ihr Referenzorbit — bis zu einer Stunde Arithmetik mit beliebiger Genauigkeit. Fractadyne bewahrt fertige Orbits hier auf, sodass die Rückkehr zu einem besuchten Ort oder das Zoomen in seine Nähe stattdessen nur Sekunden dauert. Ein Orbit belegt etwa 4 MB.
cache-off = Für diesen Lauf aus (ein Aufgabenaufruf oder --no-orbit-cache). Bis zum nächsten gewöhnlichen Start wird nichts gelesen oder geschrieben.
# Small headings of the window's parts.
cache-location = Speicherort
cache-usage = Belegung
cache-limit = Obergrenze
cache-no-config = (kein Konfigurationsordner verfügbar)
cache-open-folder = Ordner öffnen
# The text on the usage bar. $used and $limit: sizes such as "1.2 GB"; $count: the orbits kept.
cache-usage-bar =
    { $count ->
        [one] { $used } von { $limit } — ein Orbit
       *[other] { $used } von { $limit } — { $count } Orbits
    }
cache-limit-hover = Wächst der Cache darüber hinaus, werden zuerst die Orbits entfernt, deren Berechnung am BILLIGSTEN war — ein tiefer Orbit, der eine Stunde gedauert hat, überdauert beliebig viele schnelle, wie kürzlich diese auch verwendet wurden.
# Beside the limit. $count: about how many orbits of the live view's length fit in it.
cache-limit-estimate =
    { $count ->
        [one] ≈ ein Orbit bei der Live-Obergrenze
       *[other] ≈ { $count } Orbits bei der Live-Obergrenze
    }
cache-kept = Aufbewahrt (wertvollste zuerst)
# Column headers of the list of kept orbits.
cache-col-iterations = Iterationen
cache-col-precision = Genauigkeit
cache-col-size = Größe
cache-col-last-used = zuletzt verwendet
# The precision column. $bits: the precision in bits.
cache-row-bits = { $bits } Bit
# Under the list. $count: the orbits not listed.
cache-rows-more = … und { $count } weitere
# When an orbit was last used (the "last used" column).
cache-ago-now = gerade eben
# $count: minutes.
cache-ago-minutes = vor { $count } min
# $count: hours.
cache-ago-hours = vor { $count } h
# $count: days.
cache-ago-days = vor { $count } d
# Asked before clearing. $count: the orbits; $size: their total size, such as "1.2 GB".
cache-confirm =
    { $count ->
        [one] Einen Orbit löschen ({ $size })? Ihn neu zu berechnen kostet Zeit, keine Daten.
       *[other] { $count } Orbits löschen ({ $size })? Sie neu zu berechnen kostet Zeit, keine Daten.
    }
cache-delete = Löschen
cache-cancel = Abbrechen
cache-clear = Cache leeren…
cache-clear-hover = Löscht alle gespeicherten Orbits. Fragt vorher nach.
cache-close = Schließen
# $count: the orbits removed.
cache-cleared =
    { $count ->
        [one] Referenz-Cache geleert — ein Orbit entfernt.
       *[other] Referenz-Cache geleert — { $count } Orbits entfernt.
    }
# $why: the system's error message.
cache-clear-failed = Der Referenz-Cache konnte nicht geleert werden: { $why }

## Help ▸ Diagnostics…

diag-title = Diagnose
diag-intro = Einen Test ausführen, um zu prüfen, ob Fractadyne auf dieser Hardware korrekt arbeitet. Die Ergebnisse werden auf diesem Rechner gespeichert und können einem Fehlerbericht beigefügt werden — nichts wird von selbst irgendwohin gesendet.
# $status: which arbitrary-precision library is in use, as the program reports it (in English).
diag-arithmetic = Tiefenzoom-Arithmetik: { $status }
# The three tests' buttons, each followed by a sentence saying what it does.
diag-run-selftest = Selbsttest ausführen
diag-run-uitest = UI-Test ausführen
diag-run-gputest = GPU-Arithmetik prüfen
diag-blurb-selftest = Prüft Mathematik und Rendering auf dieser GPU gegen bekannt korrekte Ergebnisse. Dauert etwa 15 Sekunden.
diag-blurb-uitest = Geht die Oberfläche und die Live-Ansicht durch und nimmt in mehreren Zoomtiefen Screenshots auf. Dauert ein bis zwei Minuten und öffnet während des Laufs Fenster.
diag-blurb-gputest = Prüft die Double-Float-Arithmetik des Shaders gegen bekannt korrekte Werte auf jedem Grafik-Backend, das der Rechner bietet. Dauert wenige Sekunden, öffnet kein Fenster. Meldet die Prüfung einen Fehler, ist das ein echter Befund, den wir gern sehen würden — bitte einem Bericht beifügen.
# While a test runs. $count: the checks finished so far.
diag-checks-done =
    { $count ->
        [one] eine Prüfung erledigt
       *[other] { $count } Prüfungen erledigt
    }
# A finished test's result. $test: the test's name (Self-test, UI test, GPU arithmetic check).
diag-verdict-captured = { $test } — Ergebnis erfasst
diag-verdict-passed = { $test } — bestanden
diag-verdict-problems = { $test } — Probleme gemeldet
# Under the GPU arithmetic check's result.
diag-informational-note = Diese Prüfung kennt kein Bestanden oder Nicht bestanden: Sie hält fest, wie die GPU rechnet, und ein gemeldeter Fehler ist ein echter Befund und kein Fehler des Rechners. So oder so ist das Anhängen an einen Bericht das Nützlichste, was sich damit tun lässt.
# Under a test's result when it reported problems.
diag-problems-note = Auf anderer Hardware als der Referenz-GPU sind einige Abweichungen zu erwarten — insbesondere bei Bildvergleichen. Das Anhängen an einen Fehlerbericht ist das Nützlichste, was sich damit tun lässt.
diag-open-results = Ergebnisse öffnen
diag-attach = An Fehlerbericht anhängen…
diag-attach-hover = Öffnet den Berichtsdialog mit diesem Ergebnis, sodass das Issue einen maschinell geprüften Test enthält statt nur einer Beschreibung
diag-console = Diagnose in der Konsole ausgeben
diag-console-hover = Ab jetzt die [fd-…]-Zeilen auf stderr anzeigen. Standardmäßig aus, wenn Fractadyne ohne Argumente gestartet wird. Das ändert nichts daran, was AUFGEZEICHNET wird — die Protokolldatei erhält immer jede Zeile —, also ausgeschaltet lassen, außer man beobachtet ein Terminal. Gilt nur für diese Sitzung; mit der Option --console oder FRACTADYNE_CONSOLE=1 lässt sich auch der Start erfassen.
diag-close = Schließen
# Errors starting a test. $why: the system's error message; $dir: a folder.
diag-error-no-exe = Programmdatei nicht gefunden: { $why }
diag-error-create-dir = { $dir } kann nicht erstellt werden: { $why }
diag-error-start = Der Test konnte nicht gestartet werden: { $why }

## Navigate ▸ Misiurewicz explorer…

misi-title = Misiurewicz-Explorer
misi-mandelbrot-only = Das Durchsuchen von Misiurewicz-Punkten gibt es nur für Mandelbrot — zum Verwenden das Fraktal wechseln.
# While the gallery is being built. $seconds: the time so far, such as "3".
misi-solving = Die Familien der kleinen Typen werden berechnet ({ $seconds } s)…
# Above the gallery. $total: all points; $curated: the hand-picked ones; $found: the ones the sweep found.
misi-summary = { $total } Punkte — { $curated } handverlesen, { $found } gefunden durch einen Newton-Suchlauf mit Startwerten über die kleinen Typen (obere Halbebene; ein grobes Raster findet die meisten Einzugsgebiete, nachweislich aber nicht alle).
# The name of a point the sweep found. $kind: its type, such as "(3,1)"; $point: its coordinate, such as "-0.1011+0.9563i".
misi-entry-name = { $kind } bei { $point }
# Under each thumbnail. $kind: the point's type, such as "(3,1)"; $octaves: the zoom period in octaves, such as "3.17"; $twist: the twist per repeat in degrees, such as "+12.5". "oct" is short for octaves.
misi-entry-readout = { $kind } · { $octaves } Okt./Wdh. · { $twist }°
# Before the depth field.
misi-zoom-to = Zoomen auf
misi-cancel = Abbrechen
misi-solve-jump = Berechnen & springen
misi-solve-jump-hover = Diesen exakten Punkt per Newton-Verfahren mit der Genauigkeit berechnen, die die gewünschte Tiefe braucht (mit seiner gespeicherten Koordinate als Startwert), dann direkt dorthin springen. Tiefe Ziele dauern länger — die Berechnung erfolgt mit beliebiger Genauigkeit.
misi-select-hint = Einen Punkt auswählen, eine Tiefe angeben und springen — die Multiplikatorzeile unter jedem Vorschaubild zeigt, wie ein Tauchgang dort aussehen wird: wie viele Oktaven, bevor sich die Ansicht wiederholt, und wie weit sie sich pro Wiederholung dreht.
misi-cancelled = Berechnung abgebrochen.
# $text: what was typed in the depth field.
misi-bad-zoom = „{ $text }“ lässt sich nicht als Vergrößerung lesen (1e50, 3.2e120, … schreiben).
misi-no-kp = Für diesen Eintrag wurde kein (k,p) erkannt, mit dem sich rechnen ließe.
misi-parse-failed = Die gespeicherte Koordinate ließ sich nicht lesen.
# A notice after the jump. $name: the point's name; $zoom: the magnification, such as "1e50"; $seconds: how long the solve took, such as "1.2".
misi-zoomed = Gezoomt auf { $name } — { $zoom }× ({ $seconds } s)
# Added after the notice above when the iteration count is too low. $have: the fixed iteration count; $typical: what the depth typically needs.
misi-iter-note = Die Iterationen sind auf { $have } festgelegt; diese Tiefe braucht typischerweise ~{ $typical } — den Wert erhöhen (oder Auto-Iterationen aktivieren), falls die Ansicht flach gerendert wird.
# $name: the point's name; $why: why the solve failed (a technical word).
misi-solve-missed = Die tiefe Berechnung ist nicht auf { $name } gelandet: { $why }. Der gespeicherte Punkt ist in seiner eigenen Tiefe weiterhin exakt — ein flacheres Ziel versuchen.

## Life (Game of Life): the side panel

# Shown when the graphics card cannot run the Life view.
life-no-compute = Dieser Grafikadapter kann Life nicht ausführen (keine Compute-Shader).
# Row label: the pattern list (pattern names stay in English).
life-pattern = Muster
life-open = Öffnen…
life-open-hover = Eine RLE-, .cells- oder Life-1.05/1.06-Musterdatei öffnen.
life-pattern-text = Mustertext…
life-pattern-text-hover = Ein Muster einfügen (RLE, wie LifeWiki es liefert, oder Klartext).
life-save = Speichern…
life-save-hover = Das Universum im aktuellen Zustand als RLE speichern.
# Row label: the rule list and the typed rule (B3/S23 notation).
life-rule = Regel
# Shown in the rule list when the rule in use is none of the library's.
life-rule-custom = eigene
life-apply = Anwenden
# Playback buttons (a pause or play symbol is shown before them).
life-pause = Pause
life-play = Start
life-play-hover = Das Universum laufen lassen (Enter).
life-step = Schritt
life-step-hover = Eine Generation weiter (N).
# Tooltip of the button beside Step, which reads "+16", "+1,024"…: that many generations.
life-stride-hover = So viele Generationen weiter (Shift+N).
life-reset = Zurücksetzen
life-reset-hover = Zurück zum Muster wie geöffnet (Generation 0).
life-speed = Tempo
# After the speed slider's number: generations a second. Short.
life-speed-unit = Gen./s
# Row label: how many generations the "+n" button advances.
life-stride = Schrittweite
# The panel's numbers.
life-generation = Generation
life-population = Population
life-tiles = Kacheln
# $count: the tiles stored; $capacity: how many the GPU pool holds.
life-tiles-of = { $count } von { $capacity }
life-tiles-hover = Gespeicherte Kacheln mit 64×64 Zellen (je 4 KiB) und die Kapazität des GPU-Pools.
# Row label: what a left-drag in the view does (Pan or Draw).
life-mouse = Maus
life-pan = Verschieben
life-pan-hover = Ziehen, um die Ansicht zu bewegen.
life-draw = Zeichnen
# The Draw tool's tooltip; the toolbar's shows it after "Draw (D):", so it starts lowercase.
life-draw-hint = eine Zelle anklicken, um sie umzuschalten; ziehen, um zu malen — ein Strich, der auf einer toten Zelle beginnt, zeichnet; einer, der auf einer lebenden Zelle beginnt, radiert. Ein laufendes Universum pausiert während des Zeichnens und läuft weiter, sobald losgelassen wird.
life-undo = Rückgängig
life-undo-hover = Zurück zum Universum vor der letzten Bearbeitung, Füllung, Leerung oder Regeländerung (Ctrl+Z beim Zeichnen).
life-grid = Gitterlinien
life-grid-hover = Zellränder ab einer Vergrößerung von 8 Pixeln pro Zelle.
# A random fill: "Random" labels the density field, "seed" the seed field after it.
life-random = Zufall
life-density-hover = Der Anteil der Zellen, die eine Füllung lebendig setzt.
life-seed = Startwert
life-fill-view = Ansicht füllen
# $cells: the largest side of the filled area, in cells (2,048).
life-fill-view-hover = Den sichtbaren Bereich (bis zu { $cells } × { $cells } Zellen um seine Mitte) zufällig füllen.
life-clear = Leeren
life-clear-hover = Alle Zellen töten (Rückgängig holt sie zurück).

## Life: notices in the panel, and file dialogs

# $path: the file written.
life-saved = { $path } gespeichert
# $error: why (from the system, in English).
life-save-failed = Speichern fehlgeschlagen: { $error }
# The universe stopped running. $error: why (in English).
life-stopped = Angehalten: { $error }
# A pattern file named a rule that does not read. $error: why (in English); $rule: the rule kept, in B3/S23 notation.
life-rule-not-understood = Die Regel des Musters wurde nicht verstanden ({ $error }); { $rule } beibehalten
# $path: the file; $error: why (in English).
life-open-failed = { $path } konnte nicht geöffnet werden: { $error }
# File-dialog filter names.
life-filter-patterns = Life-Muster
life-filter-rle = Lauflängenkodiertes Muster

## Life: the toolbar (its buttons are icons; these are their tooltips)

life-toolbar-pause-hover = Das Universum anhalten (Enter)
life-toolbar-play-hover = Das Universum laufen lassen (Enter)
# $stride: the generations a Shift+click steps.
life-toolbar-step-hover = Eine Generation weiter (N); Shift+Klick oder Shift+N geht { $stride } weiter
life-toolbar-reset-hover = Zurück zum Muster wie geöffnet (Generation 0)
# $hint: the Draw tool's tooltip (life-draw-hint).
life-toolbar-draw-hover = Zeichnen (D): { $hint }

## Life: the Pattern text window

life-text-title = Mustertext
life-text-intro = Ein Muster einfügen: RLE (wie LifeWiki und Golly es liefern), Klartext-.cells oder Life 1.05 / 1.06.
life-text-load = Laden
life-text-current = Aktuelles als RLE
life-text-current-hover = Das Feld mit dem zuletzt geladenen Universum füllen.

## L-systems: the side panel

# Row label: the system list (system names stay in English).
lsys-system = System
lsys-open = Öffnen…
lsys-open-hover = Eine Fractint-.l-Datei oder ein .lsys-System öffnen.
lsys-edit = Bearbeiten…
lsys-edit-hover = Der Text des Systems: Axiom, Produktionen, Winkel — bearbeiten und anwenden.
lsys-save = Speichern…
lsys-save-hover = Das System als .lsys-Textdatei speichern.
# Tooltip of the "SVG…" button.
lsys-svg-hover = Die Ansicht als SVG-Zeichnung exportieren: ihre Linien und Formen als Vektoren, in ihren Farben.
lsys-follow = Ordnung folgt dem Zoom
lsys-follow-hover = Die Ordnung steigt beim Hineinzoomen, sodass die Details nie ausgehen.
lsys-follow-fixed-hover = Dieses System wächst nicht mit jeder Ordnung um einen Faktor, daher wird es in einer festen Ordnung gezeichnet.
lsys-order = Ordnung
lsys-angle = Winkel
lsys-sweep-hover = Den Winkel hin und zurück durchfahren: Die Kurve verwandelt sich, während sie sich dreht
# A small button that puts back the system's own angle after the slider moved it.
lsys-system-angle = Winkel des Systems
lsys-seed = Startwert
lsys-seed-hover = Was die Zufallsentscheidungen bestimmt: derselbe Startwert, dieselbe Pflanze.
lsys-seed-new = Neu
lsys-seed-new-hover = Eine andere Pflanze: ein neuer Startwert.
lsys-line-width = Linienbreite
# Row label of a percentage slider: how much of the curve is drawn.
lsys-draw-on = Zeichenanteil
lsys-draw-on-hover = Wie viel der Kurve gezeichnet wird, in der Reihenfolge, in der die Turtle sie zeichnet
lsys-draw-on-play-hover = Die Kurve zeichnet sich selbst
lsys-colour-by = Färben nach
# The colourings, in the "Colour by" list.
lsys-colour-position = Position entlang der Kurve
lsys-colour-depth = Verzweigungstiefe
lsys-colour-heading = Richtung
# C, < and > are the system's commands that set the colour index: keep them.
lsys-colour-index = Farbindex (C, <, >)
lsys-colour-plain = eine Farbe
# The panel's numbers.
lsys-growth = Wachstum
lsys-growth-built = als Wort aufgebaut, in seiner Ordnung
lsys-growth-built-hover = Parametrisch oder kontextsensitiv: Die Ersetzung eines Moduls hängt von seinen Zahlen oder seinen Nachbarn ab, daher wird das Wort Generation für Generation aufgebaut (bis zu zwei Millionen Module) und in doppelter Genauigkeit gezeichnet — die Ordnung folgt nicht dem Zoom.
# How much the picture grows from one order to the next. $factor: the factor (1.4142).
lsys-growth-factor = ×{ $factor } pro Ordnung
lsys-growth-factor-period-2 = ×{ $factor } pro Ordnung (Ordnungen in 2er-Schritten)
lsys-growth-none = keines (feste Ordnung)
lsys-drawn = Gezeichnet
# $drawn: what was drawn (lsys-drawn-segments, lsys-drawn-shapes or lsys-drawn-segments-shapes); $order: the order.
lsys-drawn-at-order = { $drawn } in Ordnung { $order }
# The same, when the walk stopped at its segment budget.
lsys-drawn-at-order-budget = { $drawn } in Ordnung { $order } (Budget)
# $segments: the line segments drawn.
lsys-drawn-segments =
    { $segments ->
        [one] { $segments } Segment
       *[other] { $segments } Segmente
    }
# $shapes: the filled shapes drawn.
lsys-drawn-shapes =
    { $shapes ->
        [one] { $shapes } gefüllte Form
       *[other] { $shapes } gefüllte Formen
    }
# Both counts. $segments: line segments; $shapes: filled shapes.
lsys-drawn-segments-shapes =
    { $segments ->
        [one] { $segments } Segment
       *[other] { $segments } Segmente
    }, { $shapes ->
        [one] { $shapes } gefüllte Form
       *[other] { $shapes } gefüllte Formen
    }
# $ms: the walk's time in milliseconds (12.3).
lsys-drawn-hover = durchlaufen in { $ms } ms; Teilbäume außerhalb der Ansicht werden übersprungen, solche unter einem Pixel als ein Segment gezeichnet
# A word too long to build at the order asked for. $wanted: the order asked for; $built: the order drawn.
lsys-drawn-short = Das Wort der Ordnung { $wanted } hat über zwei Millionen Module: in Ordnung { $built } gezeichnet.
# $order: the deepest order the app draws.
lsys-deepest-order = In der tiefsten gezeichneten Ordnung ({ $order }): Weiteres Zoomen fügt keine Details hinzu.

## L-systems: notices in the panel and the editor, and file dialogs

# $path: the file; $error: why (in English).
lsys-open-failed = { $path } konnte nicht geöffnet werden: { $error }
# Why a file was not opened (shown after "Could not open …:").
lsys-file-too-large = größer als 4 MiB
# Why a Fractint .l file was not opened: it holds no entries. `Name { … }` is the file's syntax: keep it.
lsys-no-entries = keine Einträge (eine .l-Datei enthält Einträge der Form `Name {"{"} … {"}"}`)
# $path: the file written.
lsys-saved = { $path } gespeichert
# $error: why (from the system, in English).
lsys-save-failed = Speichern fehlgeschlagen: { $error }
# After File ▸ Export SVG. $path: the file written.
lsys-svg-exported = { $path } exportiert
# $error: why (from the system, in English).
lsys-svg-failed = Export fehlgeschlagen: { $error }
# File-dialog filter names: opening (any system file) and saving (one system).
lsys-filter-systems = L-Systeme
lsys-filter-system = L-System

## L-systems: the toolbar

lsys-toolbar-lower-hover = Die Ordnung senken (legt sie fest)
lsys-toolbar-raise-hover = Die Ordnung erhöhen (legt sie fest)
# Between the − and + buttons, in a fixed-width font. $order: the order, padded to 3 characters so the label keeps its width.
lsys-toolbar-order = Ordnung { $order }
# A toggle: the order follows the zoom. Short.
lsys-toolbar-auto = auto
lsys-toolbar-auto-hover = Die Ordnung folgt dem Zoom
lsys-toolbar-edit-hover = Den Text des Systems bearbeiten

## L-systems: the editor window, and a .l file's entries

lsys-editor-title = L-System
# The editor's help. The quoted examples ('X = word', 'seed n', 'define R 1.456'…), the commands and "The Algorithmic Beauty of Plants" (a book's title) are the system's syntax and names: keep them as they are.
lsys-editor-help = Ein Schlüssel pro Zeile: angle (Grad, oder /n für einen Bruchteil des Kreises), axiom, Produktionen 'X = word' und optional heading, draw / move / variables, colour, order. Ein Symbol mit zufällig gewählten Alternativen hat für jede eine Zeile mit ihrem Gewicht: 'X (0.3) = word'; 'seed n' wählt die Pflanze. In einem Wort füllt {"{"} {"}"} den Pfad der Turtle, und . fügt einen Eckpunkt hinzu. Parametrische und kontextsensitive Produktionen (The Algorithmic Beauty of Plants): 'A(s) : s > 1 = F(s)[+A(s/2)]', 'b < a > c = b', mit 'define R 1.456' und 'ignore +-F'; F(l) geht l weit, +(a) dreht um a Grad. Ein Fractint-.l-Eintrag kann unverändert eingefügt werden.
lsys-editor-apply = Anwenden
lsys-editor-revert = Zurücksetzen
lsys-editor-revert-hover = Der Text des angezeigten Systems.
# The window listing a .l file's entries to pick from. $file: the file's name.
lsys-entries-title = { $file } — Einträge
# Tooltip of an entry that does not parse (its error is shown beside it, in English).
lsys-entry-unreadable-hover = Dieser Eintrag ist nicht lesbar.

## Export: opening a view or location (File ▸ Open view or location…)

# File-dialog filter names.
export-filter-view = Fractadyne-Ansicht oder -Ort
export-filter-image = Bild (PNG / EXR)
export-filter-location = Ort (.fdn / .kfr)
# $zoom: the magnification (1.2e148), followed by the magnification sign ×.
export-loaded-location = Ort geladen @ { $zoom }×
# $zoom: the magnification; $note: what was noteworthy in the file (in English).
export-loaded-location-note = Geladen @ { $zoom }× — { $note }
# $path: the file.
export-not-location = { $path } ist kein Fractadyne-Ort.
export-file-too-large = Datei zu groß (kein .fdn-Ort?).
# $path: the file; $error: why (from the system, in English).
export-read-failed = { $path } konnte nicht gelesen werden: { $error }
# A Kalles Fraktaler .kfr file did not import. $error: why (in English).
export-import-failed = Import fehlgeschlagen: { $error }
# $path: an image whose embedded text is not a view.
export-not-view = { $path } enthält Fractadyne-Metadaten, aber keine Ansicht.
# $path: the image the view was read from.
export-loaded-view = Ansicht aus { $path } geladen
# $path: the image; $note: what was noteworthy in it (in English).
export-loaded-view-note = Ansicht aus { $path } geladen — { $note }
export-no-metadata = Diese Datei enthält keine eingebetteten Fractadyne-Ansichtsmetadaten.

## Export: notices after an export or a snapshot
## "File → Export image…" in them names the menu items (menu-file, menu-file-export).

# The export window's status while the image renders.
export-rendering = Wird gerendert…
export-canceled = Export abgebrochen.
# $error: why (in English).
export-failed = Export fehlgeschlagen: { $error }
# $size: the image's size in pixels (1920×1080); $path: the file written.
export-saved = { $size } gespeichert → { $path }
# An L-system export whose walk stopped at its budget. $size: pixels; $segments: the budget (4,000,000); $path: the file.
export-saved-incomplete = { $size } gespeichert (unvollständig: über dem Budget von { $segments } Segmenten — die Ordnung senken) → { $path }
# A two-panel export written as two files. $path: the first of them.
export-saved-two = 2 Dateien gespeichert → { $path }
# $size: pixels; $path: the file written.
export-saved-corrected = { $size } gespeichert (Glitch-korrigiert) → { $path }
# $path: the first of the two files.
export-saved-two-corrected = 2 Dateien gespeichert (Glitch-korrigiert) → { $path }
# Added after a finished export's notice. $duration: how long it took (8.3s, 1m 04.0s).
export-elapsed = (in { $duration })
export-preparing-deep = Tiefer Export wird vorbereitet — Referenz wird berechnet (das kann eine Weile dauern)…
# $size: pixels (5120×4035); $ss: the supersampling factor; $iterations: the iteration count.
export-large = Großer Export ({ $size }, { $ss }× Supersampling, { $iterations } Iterationen) — wird im Hintergrund gerendert; die Glitch-Korrektur wird bei Exporten dieser Größe übersprungen…
export-life-unavailable = Der Bildexport einer Life-Ansicht ist noch nicht verfügbar — einen Screenshot verwenden oder Datei ▸ Life-Muster speichern.
export-already-running = Es läuft bereits ein Export — Datei → Bild exportieren… zeigt seinen Fortschritt.
# $size: the image's size in pixels.
export-snapshot-background = Schnappschuss: { $size } wird im Hintergrund gerendert — Datei → Bild exportieren… zeigt den Fortschritt und kann es abbrechen.
export-no-gpu = GPU nicht verfügbar
# A screen-capture snapshot. $size: pixels; $path: the file written.
export-snapshot-saved = Bildschirm-Schnappschuss { $size } gespeichert → { $path }
# $error: why (in English).
export-snapshot-failed = Schnappschuss fehlgeschlagen: { $error }

## The colour picker (the popup of a colour swatch)

# The third row's label: the colour as hexadecimal bytes. Short (about 6 characters).
picker-row-hex = Hex
picker-hex-hover = Klicken, um einen Hex-Wert einzugeben
picker-cancel = Abbrechen
picker-cancel-hover = Die Farbe wiederherstellen, mit der diese Farbauswahl geöffnet wurde
picker-ok = OK
picker-ok-hover = Diese Farbe behalten
# Starts the screen eyedropper (an icon is shown before it).
picker-pick = Aufnehmen
picker-pick-hover = Eine Farbe von einer beliebigen Stelle des Bildschirms aufnehmen
# Tooltip of the disabled Pick button (in the picker and the gradient editor) on Linux and macOS.
picker-eyedropper-unsupported = Das Aufnehmen einer Farbe von anderswo auf dem Bildschirm braucht eine Bildschirmaufnahme-API der Plattform. Nur der Windows-Build hat eine; unter Linux bräuchte das eine X11-Abhängigkeit, und Wayland verlangt ein Portal mit eigener Berechtigungsabfrage.

## The fractal families: what each one is (the info panel's "About" section and the pickers' hover text).
## The formulas themselves (z -> z^2 + c) are written in code and stay as they are.

fractal-mandelbrot-about = Das kanonische Fluchtzeit-Fraktal: die Menge aller c, für die der Orbit von 0 beschränkt bleibt. Ihr Rand ist unendlich verwickelt.
fractal-multibrot3-about = Eine Multibrot-Menge – die Mandelbrot-Konstruktion mit höherer Potenz. Die Potenz d ergibt eine (d-1)-zählige Rotationssymmetrie.
fractal-multibrot4-about = Multibrot mit Potenz 4: dreizählige Symmetrie, breite Knospen.
fractal-multibrot5-about = Multibrot mit Potenz 5: vierzählige Symmetrie.
fractal-tricorn-about = Der Tricorn (Mandelbar): konjugiert z in jedem Schritt. Diese antiholomorphe Abbildung ergibt eine dreispitzige Form.
fractal-burning-ship-about = Die Beträge der Teile von z werden vor dem Quadrieren genommen; das Ergebnis ähnelt einem brennenden Schiff.
fractal-celtic-about = Ein Verwandter des Burning Ship, der nur vom Realteil von z^2 den Betrag nimmt und Motive aus keltischen Knoten / Herzen erzeugt.
fractal-buffalo-about = Eine Betragsvariante, die von beiden Komponenten von z^2 den Betrag nimmt.
fractal-phoenix-about = Der Phoenix verwendet auch den vorigen Iterationswert, was flammenartige Filamente ergibt. Seine Julia-Form lässt sich über den Julia-Modus ausprobieren.
fractal-newton-about = Newtons Iteration zur Nullstellensuche für z^3 = 1, gefärbt danach, wie schnell jeder Punkt konvergiert. Ein Konvergenz- (kein Flucht-)Fraktal.
fractal-multibrot6-about = Multibrot mit Potenz 6: fünfzählige Symmetrie.
fractal-multibrot7-about = Multibrot mit Potenz 7: sechszählige Symmetrie, die Knospen werden zu einem Kreis hin dünner.
fractal-multibrot8-about = Multibrot mit Potenz 8: siebenzählige Symmetrie.
fractal-burning-ship3-about = Das Burning Ship mit Potenz 3: Die Beträge werden genommen, dann kubiert.
fractal-burning-ship4-about = Das Burning Ship mit Potenz 4.
fractal-burning-ship5-about = Das Burning Ship mit Potenz 5.
fractal-tricorn3-about = Der Tricorn (Mandelbar) mit Potenz 3: vierzählige Symmetrie.
fractal-tricorn4-about = Der Tricorn mit Potenz 4: fünfzählige Symmetrie.
fractal-tricorn5-about = Der Tricorn mit Potenz 5: sechszählige Symmetrie.
fractal-celtic3-about = Die Celtic-Faltung mit Potenz 3: der Betrag des Realteils von z^3.
fractal-celtic4-about = Die Celtic-Faltung mit Potenz 4.
fractal-celtic5-about = Die Celtic-Faltung mit Potenz 5.
fractal-buffalo3-about = Der Buffalo mit Potenz 3: die Beträge beider Teile von z^3.
fractal-buffalo4-about = Der Buffalo mit Potenz 4.
fractal-buffalo5-about = Der Buffalo mit Potenz 5.
fractal-custom-about = Eine Formel in Ausdrücken im Fractint-Stil (Fraktal ▸ Eigene Formel…). Die meisten erlauben Tiefenzoom per Störungsrechnung wie die eingebauten Familien; der Formeldialog zeigt, ob das für Ihre gilt.
fractal-life-about = Conways Game of Life und seine Verwandten auf einer unbegrenzten Ebene: Generations-Regeln, nicht totalistische (Hensel-)Regeln, Musterdateien. Abspielen, schrittweise weiterschalten, zeichnen.
fractal-lsystem-about = Lindenmayer-Systeme: Kurven, raumfüllende Kurven, Inseln und Pflanzen, von einer Turtle nach einer Ersetzungsgrammatik gezeichnet. Die Ordnung folgt dem Zoom, sodass die Details nie ausgehen. Fractint-.l-Dateien lassen sich direkt öffnen.

# Shown where the other families show their formula (z -> z^2 + c).
fractal-custom-formula = z -> die selbst geschriebene Formel
# Shown where the other families show their formula. "B3/S23" is the rule's notation: keep it.
fractal-life-formula = B3/S23: geboren mit 3 lebenden Nachbarn, überlebt mit 2 oder 3
# Shown where the other families show their formula. "F = F+F--F+F" is the grammar's notation: keep it.
fractal-lsystem-formula = F = F+F--F+F: Jedes Symbol wird ersetzt und dann von einer Turtle gezeichnet

# The last paragraph of the picker's hover text for a family that renders only on the direct path.
fractal-no-deep-zoom = Nur direktes Rendern – diese Familie hat keinen Tiefenzoom.

## Names of settings, shown in the Settings menu and the welcome window.

theme-dark = Dunkel
theme-light = Hell
# The update track that follows only stable releases.
update-track-stable = Stabil
# The update track that also offers pre-releases (betas).
update-track-beta = Beta (Vorabversionen)

## The status bar (its hover texts; the readouts follow in their own section)

status-period-hover = Periode des Minibrots, zu dem gesprungen wurde (M drücken oder Zu Ort springen ▸ Zu Merkmal springen). Sie bleibt, solange die Ansicht auf dem Merkmal steht, und verschwindet, sobald man sich entfernt.

## The playback transport: the player over the view while a tour plays.

# $close: the close button's icon, as drawn on that button.
play-finished-hover = Fertig — zurückspulen oder mit { $close } schließen
# $kind: "benchmark" or "script" (the variants hold the wording); $name: the tour's name.
play-playing-hover =
    { $kind ->
        [benchmark] Benchmark „{ $name }“ wird abgespielt
       *[other] Skript „{ $name }“ wird abgespielt
    }
play-restart = Von vorn beginnen
play-back = 10 Sekunden zurück
play-again = Noch einmal von vorn abspielen
play-resume = Fortsetzen
play-pause = Pause
play-stop = Stoppen und an den Anfang zurückspulen
play-forward = 10 Sekunden vor
play-speed-hover = Wiedergabegeschwindigkeit (Klick wechselt 0,5× / 1× / 2× / 4×)
play-render = Dieses Skript als Bildfolge rendern…
play-render-none = Der eingebaute Benchmark hat keine Skriptdatei zum Rendern
play-open = Ein anderes Skript abspielen…
play-loop = Die Tour am Ende wiederholen
play-close = Den Player schließen und die eigenen Ansichtseinstellungen wiederherstellen
# A notice beside the scrub bar while the renderer holds the tour's clock. Keep it short.
play-waiting = wartet auf Details
# "[playback] pace" is a setting in the script file: keep it as written.
play-waiting-hover = Die Tour-Uhr ist pausiert, während der Renderer diese Ansicht auflöst (Referenzberechnung / steigendes Iterationsbudget). Die Wiedergabe läuft von selbst weiter; siehe [playback] pace im Skript.
play-scrub = Ziehen, um durch die Tour zu spulen
# $behind: how far the script clock is behind real time, as minutes:seconds (1:05).
play-scrub-behind =
    Ziehen, um durch die Tour zu spulen
    Die Skript-Uhr liegt { $behind } hinter der Echtzeit (der Taktgeber hat die Tour verlangsamt, damit der Renderer mithalten kann). Die obere Markierung zeigt, wo die Wiedergabe nach Echtzeit stünde.

## The autopilot's notices (toasts). Where a notice has two lines, the first is the verdict and the second what to do about it.

autopilot-on-detail = Autopilot an — taucht auf Details zu (jede Eingabe hält ihn an)
autopilot-off = Autopilot aus
autopilot-misi-needs-mandelbrot =
    Autopilot: Ein Misiurewicz-Ziel braucht die Mandelbrot-Menge
    Taucht stattdessen auf Details zu
autopilot-on-misi =
    Autopilot an — sucht den nächsten Misiurewicz-Punkt…
    Jede Eingabe hält ihn an
# $seconds: how long the search has run, in whole seconds.
autopilot-misi-finding =
    Autopilot: sucht den nächsten Misiurewicz-Punkt…
    { $seconds } s · jede Eingabe hält ihn an
autopilot-misi-not-found =
    Autopilot: kein Misiurewicz-Punkt nahe dieser Ansicht gefunden
    Die Suche reicht womöglich nicht bis in diese Tiefe — das Ziel „Detail“ versuchen
# ($preperiod,$period) names the point, as mathematicians write it: keep the notation. $widths: a power of two (2^$widths screen widths).
autopilot-misi-off-screen =
    Autopilot: Der nächste Misiurewicz-Punkt ({ $preperiod },{ $period }) liegt außerhalb des Bildschirms
    ~2^{ $widths } Bildschirmbreiten entfernt — näher an ein Spiralzentrum heranfahren
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-not-solved =
    Autopilot: Der Misiurewicz-Punkt ({ $preperiod },{ $period }) nahe dieser Ansicht ließ sich nicht berechnen
    Das Ziel „Detail“ versuchen
autopilot-misi-refused = Autopilot: Misiurewicz-Berechnung abgelehnt (ungültiges Paar)
autopilot-misi-stopped = Autopilot: Der Misiurewicz-Löser wurde unerwartet beendet
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-diving =
    Autopilot: taucht in Misiurewicz ({ $preperiod },{ $period }) ein
    Jede Eingabe hält ihn an
# ($preperiod,$period) names the point: keep the notation. $octaves: how often the structure repeats (2.37); $twist: its turn in degrees, signed (+12.5).
autopilot-misi-diving-repeat =
    Autopilot: taucht in Misiurewicz ({ $preperiod },{ $period }) ein
    Struktur wiederholt sich alle { $octaves } Oktaven, Drehung { $twist }° · jede Eingabe hält ihn an
# $limit: the magnification, as written (1e300).
autopilot-dive-limit = Autopilot: Tauchgrenze erreicht (~{ $limit }×)
# $have: the fixed iteration count; $typical: what this depth typically needs.
autopilot-dead-end-short =
    Autopilot angehalten: nichts mehr anzusteuern — die Iterationen sind auf { $have } festgelegt, und diese Tiefe braucht typischerweise ~{ $typical }
    Auto-Iterationen aktivieren oder die Zahl erhöhen
# $limit: the fixed iteration count.
autopilot-dead-end-cap =
    Autopilot angehalten: nichts mehr anzusteuern — alles in der Ansicht läuft bis zur festen Iterationsgrenze von { $limit }
    Auto-Iterationen aktivieren oder die Zahl erhöhen
autopilot-dead-end = Autopilot: keine Details voraus (angehalten)

## Tours (camera scripts)

# The name of the file type in the file dialog's filter list.
script-file-filter = Fractadyne-Skript (TOML)
# $name: the script's name.
script-finished = Skript beendet — „{ $name }“
# The title of the notice that follows (the file's path and the error, in English).
script-load-failed = Skript konnte nicht geladen werden

## Installing a language file

# The name of the file type in the file dialog's filter list.
language-file-filter = Fractadyne-Sprachdatei

## The notice when the graphics driver's shader cache stops keeping shaders. The text is shown in a fixed-width font with its line breaks as written: break the lines where they read well.

driver-cache-title = Grafiktreiber behält keine Shader
# $count: how many shaders compiled again; $pipeline: the slowest one's name (a technical id: keep it); $seconds: its time in seconds (7.5).
driver-cache-body =
    Der Grafiktreiber hat in dieser Sitzung { $count } der Shader von Fractadyne erneut
    kompiliert, obwohl er jeden davon schon zuvor kompiliert hatte (der langsamste:
    { $pipeline }, { $seconds } s). Sein Shader-Cache behält kompilierte Shader nicht, meist
    weil der Cache voll ist. Bis sich das ändert, können Exporte, tiefe Ansichten und der
    Start von Fractadyne jeweils Sekunden mit Kompilieren verbringen.
# $folder: the cache folder's path; $size: its size in MB (1024). "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia-folder =
    Zur Behebung: Fractadyne schließen, dann den Inhalt dieses Ordners löschen:
      { $folder }   ({ $size } MB)
    Der Treiber baut den Cache neu auf. Damit er nicht wieder vollläuft, unter NVIDIA
    Systemsteuerung > 3D-Einstellungen verwalten > Shader-Cache-Größe den Wert erhöhen.
# "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia =
    Zur Behebung: NVIDIA Systemsteuerung > 3D-Einstellungen verwalten > Shader-Cache-Größe
    erhöhen oder den Shader-Cache des Treibers leeren.
# "AMD Software > Graphics > Advanced > Reset Shader Cache" names AMD's own settings: use the names AMD's software shows in this language.
driver-cache-fix-amd = Zur Behebung: AMD Software > Grafik > Erweitert > Shader-Cache zurücksetzen.
driver-cache-fix-other = Zur Behebung: den Shader-Cache des Grafiktreibers leeren.

## Why a language file could not be installed: each ends the sentence "{ $file } is not a usable language file: …".

language-install-not-utf8 = kein UTF-8-Text
language-install-too-large = zu groß
# "fractadyne-<language>.ftl" is the file name to use: keep it as written.
language-install-no-tag = sie gibt nicht an, welche Sprache sie enthält (als fractadyne-<language>.ftl benennen)
language-install-english = Englisch ist integriert
language-install-empty = sie übersetzt keinen Eintrag dieser Version
language-install-no-folder = es gibt keinen Einstellungsordner

## The notice after a session that did not end cleanly, when its iteration setting was set aside.

# $base: the iteration count automatic iterations start from.
quarantine-setting-auto = automatische Iterationen ab einer Basis von { $base }
# $count: the fixed iteration count.
quarantine-setting-fixed =
    { $count ->
        [one] { $count } Iteration
       *[other] { $count } Iterationen
    }
# $end: "device" when the session lost its graphics device, "unclean" otherwise (the variants hold the wording). $depth: the magnification, as written (1.2e148). $setting and $restored: an iteration setting (quarantine-setting-auto or -fixed).
quarantine-notice =
    { $end ->
        [device] Die vorige Sitzung hat bei { $depth }× ihr Grafikgerät verloren, eingestellt auf { $setting }. Diese Sitzung begann stattdessen mit { $restored }, sodass sie nicht direkt wieder in dieselbe Last hineinlaufen kann.
       *[other] Die vorige Sitzung endete bei { $depth }× nicht ordnungsgemäß, eingestellt auf { $setting }. Diese Sitzung begann stattdessen mit { $restored }, sodass sie nicht direkt wieder in dieselbe Last hineinlaufen kann.
    }

## The second graphics card's status line, under its setting in the control panel.

gpu2-state-off = Aus: Die Grafikkarte dieses Fensters rendert alles.
gpu2-state-starting = Wird gestartet…
# $name: the card's name (and its graphics API).
gpu2-state-running = In Gebrauch: { $name }
# $why: the error, as the graphics driver reports it.
gpu2-state-failed = Konnte nicht starten: { $why }
# $name: the card's name.
gpu2-state-lost = Gestoppt: { $name } ging verloren. Ab jetzt eine Karte; zum erneuten Versuch die Karte erneut auswählen.
# $state: one of the lines above. "--worker-gpu" is a command-line option: keep it.
gpu2-state-pinned = { $state } (gesetzt durch --worker-gpu)

## The custom-formula dialog: how deep the formula renders (one line under the formula).

formula-depth-deep = Tiefenzoom per Störungsrechnung, mit erweitertem Wertebereich ab 1e28x wie bei den eingebauten Fraktalen (jeder Schritt wird iteriert: für eigene Formeln noch keine Reihennäherung oder BLA).
# $why: what the formula has that keeps it off the deep-zoom path (one of the formula-depth-why messages below).
formula-depth-direct = Direktes Rendern ({ $why } hat noch keine Tiefenzoom-Form): scharf, bis ein einzelner Schritt von c in einfacher Genauigkeit ein Pixel überspannt — etwa 1e4x bis 1e5x, weit vom Ursprung entfernt weniger tief.
formula-depth-why-init = ein Init-Abschnitt
formula-depth-why-bailout = ein eigener Abbruchtest
formula-depth-why-variable = eine von Schritt zu Schritt behaltene Variable
formula-depth-why-previous = der vorige Iterationswert
formula-depth-why-comparison = ein Vergleich oder ein if-Block
# floor, ceil, trunc and round are functions of the formula language: keep them.
formula-depth-why-rounding = Rundung (floor, ceil, trunc, round)
# "maxit" is a word of the formula language: keep it.
formula-depth-why-maxit = maxit
formula-depth-why-power = eine Potenz mit variablem Exponenten
formula-depth-why-perturbed = ein bereits gestörtes Programm

## Image and frame sizes: the presets described in words (the others are standard names: HD 720p, WUXGA, DCI 4K).

# $size: the pixel size (2048×2048).
size-preset-square = Quadrat — { $size }
# $ratio: the aspect ratio (9:16); $size: the pixel size.
size-preset-portrait = Hochformat { $ratio } — { $size }
# $ratio: the aspect ratio (32:9); $size: the pixel size.
size-preset-super-ultrawide = Super-Ultrawide { $ratio } — { $size }

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
fractal-family-custom = Eigene
# Conway's Game of Life.
fractal-family-life = Life
fractal-family-lsystem = L-System
# A family at a power: "Multibrot 3", "Burning Ship 4". $family: a name above; $power: the power.
fractal-with-power = { $family } { $power }

## Names of choices: colouring methods, orbit traps, palette animation, snapshots, angle units.

color-method-smooth = Glatte Iteration
color-method-stripe = Streifenmittel
color-method-triangle = Dreiecksungleichung
color-method-trap = Orbitfalle
color-method-distance = Distanzschätzung
color-method-decomposition = Zerlegung
# The shape an orbit trap catches the orbit with.
trap-point = Punkt
trap-cross = Kreuz
trap-circle = Kreis
# Palette animation: none.
palette-anim-off = Aus
palette-anim-forward = Vorwärts
palette-anim-reverse = Rückwärts
palette-anim-pingpong = Ping-Pong
# Palette animation: a new random gradient every so often.
palette-anim-random = Zufällige Farbverläufe
# What the Snapshot button does.
snapshot-mode-ask = Jedes Mal fragen
snapshot-mode-screen = Bildschirmaufnahme
snapshot-mode-render = Vollständiges Rendering
# The unit of an angle typed into Go to location's polar fields.
angle-degrees = Grad
angle-radians = Bogenmaß
angle-turns = Umdrehungen

## Report an issue: the choices. (The report itself is sent in English.)

issue-kind-crash = Programmabsturz / Einfrieren
issue-kind-rendering = Fehlerhafte Darstellung
issue-kind-performance = Leistungsproblem
issue-kind-ui = Oberflächen-/Bedienungsproblem
issue-kind-feature = Funktionswunsch
issue-kind-other = Sonstiges
severity-low = Niedrig
severity-medium = Mittel
severity-high = Hoch
severity-blocking = Blockierend
repro-always = Immer
repro-sometimes = Manchmal
repro-once = Einmal aufgetreten
repro-cannot = Nicht reproduzierbar

## The benchmark's depths and the diagnostics' tests.

# $zoom: the dive's final magnification (1e32).
bench-depth-standard = Standard ({ $zoom }×)
# $zoom: the dive's final magnification (1e48).
bench-depth-ultra = Ultratief ({ $zoom }×)
diag-test-selftest = Selbsttest
diag-test-uitest = UI-Test
diag-test-gputest = GPU-Arithmetikprüfung

## The status bar's readouts: each word is followed by a value in a slot of fixed width. Keep the words short (one word each).

status-center = Mitte
status-cursor = Zeiger
status-zoom = Zoom
# Short for iterations.
status-iter = Iter.
# The period of the minibrot jumped to.
status-period = Periode
# A Game of Life view's scale (followed by "1 px = 4 cells" or "8 px / cell").
status-scale = Maßstab
# A Game of Life view's generation. Short.
status-generation = Gen.
# A Game of Life view's population. Short.
status-population = Pop.
# An L-system's order (how many times its rules were applied).
status-order = Ordnung
# How many segments and shapes an L-system drew.
status-drawn = Gezeichnet
# $value: how many cells one pixel covers (4, 9.99e300). "px" is the unit symbol: keep it.
life-scale-cells = 1 px = { $value } Zellen
# $value: how many pixels one cell covers (8). "px" is the unit symbol: keep it.
life-scale-pixels = { $value } px / Zelle

## The status bar's limit diagnostic: a short label (after a warning sign), and what it means when hovered.

# The view is as deep as this location can be rendered. Short.
status-limit-depth = Tiefengrenze
# Every pixel ran out of iterations. Short.
status-limit-exhausted = Iter. erschöpft
# Pixels are capped at the iteration budget. Short.
status-limit-capped = Iter. begrenzt
# $pixels: how many pixels wide one step of the coordinates is; $sharp_to: the magnification the view is sharp to (written out, 1,234,567).
status-limit-direct-detail = Diese Ansicht wird direkt auf der GPU in einfacher Genauigkeit gerendert, wo ein Schritt der Koordinaten hier { $pixels } Pixel breit ist: Benachbarte Pixel berechnen denselben Punkt, sodass das Bild in Blöcke zerfällt. An diesem Ort ist es bis etwa { $sharp_to }× scharf. Diese Formel hat noch keinen Tiefenzoom-Pfad (Störungsrechnung), daher zeigt weiteres Zoomen keine weiteren Details.
# $samples: the GPU buffer's capacity in orbit samples; $iterations: the iterations pixels are limited to.
status-limit-depth-detail = Der Referenzorbit hat die Obergrenze des GPU-Puffers ({ $samples } Punkte) erreicht, ohne zu entkommen, daher sind die Pixel hier auf ~{ $iterations } Iterationen begrenzt, und feinere Details lassen sich in dieser Ansicht nicht berechnen — weder live noch beim Export. Dies ist die aktuelle Tiefengrenze für diese Ortsfamilie.
# $budget: the largest iteration budget this view can be given.
status-limit-exhausted-detail = Jedes Pixel ist selbst mit dem größten Budget, das diese Ansicht bekommen kann ({ $budget }), an die Iterationsgrenze gestoßen. Entweder liegt die Ansicht hier in der Menge (in der Tiefe normal), oder sie braucht mehr Iterationen, als das Programm derzeit liefern kann — das Bild ist so oder so dasselbe, daher wurde das Budget auf ein günstiges zurückgesetzt.
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations" is the setting's name in the Rendering panel.
status-limit-capped-max = Bei { $percent } % der Pixel waren die Iterationen beim Budget von { $budget } erschöpft, und die entkommenden enden direkt an dieser Obergrenze — die Ansicht ist unteraufgelöst. Iterationen steht bereits auf dem Maximum, daher braucht diese Ansicht mehr, als das Programm derzeit liefern kann.
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations" and "Rendering" are the names of a setting and a panel section.
status-limit-capped-raise = Bei { $percent } % der Pixel waren die Iterationen beim Budget von { $budget } erschöpft, und die entkommenden enden direkt an dieser Obergrenze — die Ansicht ist unteraufgelöst. Iterationen (Bedienfeld ▸ Qualität) erhöhen, um sie aufzulösen.
