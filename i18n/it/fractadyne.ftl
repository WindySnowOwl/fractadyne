### Fractadyne user interface: Italian (Italiano).
### A draft: machine translation (Claude, by Anthropic) of the English file, 2026-10-23, not yet
### reviewed by a native speaker. Corrections are welcome as pull requests (see i18n/README.md).

## Language data: read by Fractadyne, not shown as messages.

# This language's own name, shown in the language menu.
language-name = Italiano
# The same name in English, shown beside it in the menu.
language-english-name = Italian
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
language-more = Altre lingue…
# The Languages window's title.
language-window-title = Lingue
# The Languages window's introduction.
language-window-intro = L’interfaccia di Fractadyne in un’altra lingua. Una lingua è un unico file: è inclusa in una versione, oppure puoi installarla qui. Coordinate, file e riga di comando restano in inglese in tutte le lingue.
# Column headers of the installed-language list.
language-col-language = Lingua
language-col-translated = Tradotto
language-col-from = Origine
# Where a language comes from.
language-source-builtin = integrata
language-source-package = questa versione
language-source-user = installata
language-source-dev = cartella di lavoro
# $percent: how much of the interface the file translates, 0–100.
language-coverage = { $percent }%
# $count: messages in the file that failed the checks and show in English instead.
language-dropped =
    { $count ->
        [one] un messaggio da verificare
       *[other] { $count } messaggi da verificare
    }
# Buttons of the Languages window.
language-install = Installa file di lingua…
language-install-hover = Scegliere un file di lingua di Fractadyne (.ftl). Viene verificato, poi copiato nella cartella delle lingue delle tue impostazioni, dove resta da un aggiornamento all’altro.
language-open-folder = Apri cartella delle lingue
language-reload = Ricarica
language-reload-hover = Rileggere i file di lingua (dopo averne modificato o aggiunto uno).
language-use = Usa
# Notices after installing a file. $name: the language's own name; $file: the file's name.
language-installed = Lingua { $name } installata da { $file }.
# $file: the file's name; $why: what was wrong with it.
language-install-failed = { $file } non è un file di lingua utilizzabile: { $why }
# Shown when a language file cannot be loaded and English is used instead. $name: the language.
language-load-failed = Impossibile caricare la lingua { $name }, quindi Fractadyne è in inglese: { $why }
# A language whose writing system needs a font this computer does not have (Japanese or Chinese without a system font). $name: the language, in its own name.
language-font-missing = Su questo computer non è stato trovato alcun font in grado di disegnare { $name }, quindi la lingua non può essere mostrata. Installa un font per { $name } (per esempio il pacchetto di lingua del sistema) e scegli di nuovo la lingua.
# Shown after a draft translation's name in the language menu and the Languages window. Short.
language-draft = bozza
language-draft-hover = Una bozza: questa traduzione non è ancora stata revisionata da un madrelingua. Le correzioni sono benvenute, come pull request al file della lingua.
# The heading over the decimal-mark choice, in the language menu and the Languages window.
language-decimal-mark = Separatore decimale
language-decimal-mark-hover = Come si scrivono i numeri: tempi, dimensioni e conteggi. Coordinate e ingrandimenti usano sempre il punto, come li leggono i file e gli altri programmi.
# $sample: a number written with this language's own punctuation (1,234.5).
language-decimal-language = Come nella lingua ({ $sample })
# $sample: a number written with a decimal point (1,234.5).
language-decimal-point = Punto ({ $sample })
# $sample: a number written with a decimal comma (1.234,5).
language-decimal-comma = Virgola ({ $sample })

## Asked once when the system's language changes

# The window's title.
language-changed-title = Lingua
# $new: the system's new language, in its own name.
language-changed-question = La lingua del sistema ora è { $new }. Usarla in Fractadyne?
# Buttons. $name: a language's own name.
language-changed-use = Usa { $name }
language-changed-keep = Mantieni { $name }

## The welcome window (first run)

welcome-title = Benvenuto in Fractadyne
welcome-intro = Un esploratore a zoom profondo per l’insieme di Mandelbrot e i suoi parenti. Alcuni comandi per iniziare:
# The language row, at the top of the window.
welcome-language = Lingua
# Controls: the key or gesture on the left, what it does on the right.
welcome-key-drag = trascina
welcome-key-drag-does = sposta la vista
welcome-key-scroll = rotellina
welcome-key-scroll-does = zoom avanti / indietro verso il cursore
welcome-key-space = tieni premuto Space
welcome-key-space-does = zoom continuo e fluido (Shift+Space all’indietro)
# Shown after the click-zoom toolbar icon.
welcome-key-click = poi clic
welcome-key-click-does = zoom sul punto cliccato (barra degli strumenti)
welcome-key-m-does = salta al minibrot più vicino, alla sua scala
welcome-landmarks = Per iniziare, salta a un luogo celebre:
welcome-theme = Tema
welcome-watermark = Mostra il marchio «Fd»
welcome-watermark-hover = Il piccolo marchio in basso a destra della vista. Se disattivato, scompare dalla vista in tempo reale e dalle immagini esportate.
welcome-updates = Aggiornamenti
welcome-check-updates = Controlla aggiornamenti
welcome-check-updates-hover = All’avvio dell’app chiede a GitHub l’ultima versione. Non viene inviato nulla.
welcome-files = File
welcome-no-config = (nessuna cartella di configurazione)
welcome-open = Apri
welcome-rendering-on = Rendering su
welcome-open-help = Apri la guida completa  (F1)
# $icon: the Settings icon.
welcome-more-in-settings = Altro in File → { $icon } Impostazioni
welcome-get-started = Inizia

## Menus: shared

# A percentage in a menu (UI scale). $value: the number.
common-percent = { $value }%
# A state shown after a menu item's name, in brackets.
menu-state-connected = connesso
menu-state-rendering = rendering in corso
menu-state-running = in esecuzione

## Navigate ▸ Bookmarks (also the toolbar's star)

menu-bookmarks = Segnalibri
menu-bookmarks-add = Aggiungi vista corrente
menu-bookmarks-manage = Gestisci…

## The fractal picker (Fractal menu and the toolbar's dropdown)

menu-family-powers = Potenze superiori
menu-family-automata = Automi
menu-family-lsystems = L-system
menu-fractal-custom = Formula personalizzata…
menu-fractal-library = Libreria di formule…
menu-fractal-library-hover = Formule personalizzate salvate: applicarle, modificarle, importarle ed esportarle
# The "Show" group: which picture of the formula is drawn. $name: the formula's name (Mandelbrot, Burning Ship…).
menu-show = Mostra
menu-show-set = Insieme di { $name }
menu-show-set-hover = Il piano dei parametri — l’insieme di { $name } stesso.

    Ogni suo punto è un insieme di Julia diverso; questa è la mappa di tutti.
menu-show-julia = Insieme di Julia
menu-show-julia-hover = Un punto dell’insieme di { $name }, disegnato come immagine a sé — la forma che si ottiene tenendo fisso quel punto.

    Scegli il punto passando con il cursore sulla mappa in «Entrambi, collegati»; un clic lì lo fissa.
menu-show-julia-none = { $name } non ha parametri liberi, quindi non ha un insieme di Julia.
menu-show-both = Entrambi, collegati
menu-show-both-hover = L’insieme di { $name } a sinistra e, a destra, l’insieme di Julia del punto su cui si trova il cursore.

    Muovi il cursore sulla mappa e guarda il Julia cambiare — è il modo più rapido per vedere come i due sono legati.
menu-show-both-none = { $name } non ha un insieme di Julia da affiancare.

## File menu

menu-file = File
menu-file-open-view = Apri vista o posizione…
menu-file-gallery = Galleria…
menu-file-export = Esporta immagine…
menu-file-snapshot = Istantanea
menu-file-open-life = Apri pattern di Life…
menu-file-open-life-hover = Un file RLE, .cells o Life 1.05/1.06, mostrato come universo di Life.
menu-file-life-text = Testo del pattern di Life…
menu-file-life-text-hover = Incollare un pattern come testo (RLE come lo fornisce LifeWiki).
menu-file-save-life = Salva pattern di Life…
menu-file-save-life-hover = L’universo così com’è ora, in formato RLE.
menu-file-open-lsystem = Apri L-system…
menu-file-open-lsystem-hover = Un file .l di Fractint (con le sue voci tra cui scegliere) o un sistema .lsys.
menu-file-save-lsystem = Salva L-system…
menu-file-save-lsystem-hover = Il testo del sistema, come file .lsys.
menu-file-export-svg = Esporta SVG…
menu-file-export-svg-hover = La vista come disegno SVG: le sue linee e forme come vettori, nei suoi colori.
menu-file-render-client = Client di rendering…
menu-file-render-client-hover = Prestare la GPU di questa macchina a una render farm: connettersi a un controller (Strumenti ▸ Renderizza sulla farm… su un’altra macchina) e renderizzare fotogrammi per esso.
menu-file-settings = Impostazioni
menu-file-reset = Reimposta stato dell’applicazione…
menu-file-reset-hover = Eliminare tutto lo stato salvato (sessione, segnalibri, miniature) e ripartire da zero. Chiede prima una conferma.
menu-file-quit = Esci

## File ▸ Settings

menu-settings-fps = Limite di FPS
menu-settings-fps-uncapped = Illimitato
# $fps: frames per second.
menu-settings-fps-n = { $fps } FPS
menu-settings-ui-scale = Scala interfaccia (dimensione testo)
menu-settings-theme = Tema
menu-settings-updates = Aggiornamenti
menu-settings-updates-stable-hover = Ultima versione stabile
menu-settings-updates-beta-hover = Ultima build, comprese le pre-release
menu-settings-updates-on-launch = Controlla aggiornamenti all’avvio
menu-settings-updates-on-launch-hover = Altrimenti, controllare manualmente da Aiuto → Controlla aggiornamenti.
# Followed by its shortcut, (Ctrl+S).
menu-settings-snapshot = Istantanea
menu-settings-snapshot-ask-hover = Proporre la scelta a ogni pressione
menu-settings-snapshot-screen-hover = Salvare la vista esattamente come appare sullo schermo (istantaneo, PNG)
menu-settings-snapshot-render-hover = Un rendering completo con le impostazioni di esportazione, in background (può richiedere molto tempo sulle viste profonde)
menu-settings-refcache = Cache dei riferimenti…
menu-settings-refcache-hover = Orbite di riferimento dello zoom profondo conservate su disco, così una posizione a cui si torna si renderizza in pochi secondi. Dove si trova, quanto è grande, il limite e un modo per svuotarla.

## Fractal menu

menu-fractal = Frattale

## View menu

menu-view = Visualizza
menu-view-zoom-home = Rimpicciolisci alla vista completa
menu-view-zoom-home-hover = Rimpicciolire fino alla vista iniziale completa (animato)
menu-view-reset = Ripristina vista predefinita
menu-view-reset-hover = Ripristinare la vista predefinita del frattale (istantaneo)
# Followed by its shortcuts.
menu-view-undo = Annulla vista
menu-view-redo = Ripeti vista
menu-view-control-panel = Pannello dei controlli
menu-view-control-panel-hover = Mostrare/nascondere il pannello dei controlli a destra.
menu-view-minimap = Minimappa
menu-view-minimap-hover = Mostrare una piccola panoramica della vista iniziale con un indicatore «sei qui» e la profondità di zoom. Cliccarla per saltare a una regione.
menu-view-perf-panel = Pannello prestazioni
menu-view-timestamp = Mostra marca temporale
menu-view-timestamp-hover = Disegnare sopra la vista un grande orologio del tempo trascorso. Indica lo stesso «+12.345s» con cui il log marca ogni riga, così la registrazione dello schermo di un problema che si manifesta solo in movimento si può allineare al log fotogramma per fotogramma. Disattivato per impostazione predefinita.
menu-view-zoom-target = Mostra bersaglio dell’auto-zoom
menu-view-zoom-target-hover = Mentre l’auto-zoom è in corso, contornare la regione della vista verso cui sta zoomando — la parte che riempirà lo schermo dopo qualche altro ingrandimento — con linee dai suoi angoli agli angoli dello schermo, e un anello sul punto verso cui si sta dirigendo. Disattivato per impostazione predefinita.
menu-view-live-panel = Pannello rendering in tempo reale
menu-view-live-panel-hover = Un pannello frontale sopra la vista che mostra il renderer in tempo reale al lavoro: che cosa ha inviato ogni fotogramma, quanto è vecchia l’immagine sullo schermo, il percorso attraverso le fasce di iterazioni, i costruttori dei riferimenti e la macchina sottostante. B ne cambia la dimensione a rotazione e la × nel suo angolo lo chiude; si può trascinare in qualsiasi angolo; passando sopra uno strumento si legge che cosa significa e come appare quando tutto va bene e quando qualcosa non va.
# The live render panel's sizes.
menu-view-live-off = Spento
menu-view-live-off-hover = Nessun pannello.
menu-view-live-strip = Striscia
menu-view-live-strip-hover = I display nixie e il banco di spie.
menu-view-live-panel-size = Pannello
menu-view-live-panel-size-hover = Gli strumenti: quadrante del percorso, età dell’immagine, riquadri, passate GPU, quadrante della profondità, spie.
menu-view-live-full = Completo
menu-view-live-full-hover = Il pannello e la sala macchine: passi, GPU, memoria, core, pixel.
menu-view-live-opacity = Opacità
menu-view-watermark = Mostra filigrana «Fd»
menu-view-watermark-hover = Disegnare un discreto marchio «Fd» nell’angolo in basso a destra della vista in tempo reale e delle immagini esportate. Attivo per impostazione predefinita.

## Color menu

menu-color = Colore
menu-color-method = Metodo
menu-color-palette = Tavolozza
menu-color-custom = Personalizzata
menu-color-duotone = Duotono
menu-color-binary = Binario (insieme)
menu-color-binary-hover = Due colori piatti: dentro o fuori dall’insieme, senza gradiente.
menu-color-edit-gradient = Modifica gradiente…
menu-color-normalize = Normalizza colori profondi
menu-color-normalize-hover = Rimappare la tavolozza sull’intervallo di fuga misurato della vista quando un campo denso apparirebbe altrimenti come puntinatura. Una vista senza aliasing resta invariata — spuntare «Adatta sempre l’intervallo» per rimappare comunque. Solo con il metodo Iterazione continua.
menu-color-more = Ciclo, offset, animazione ed effetti: pannello Controlli ▸ Colorazione

## Tools menu

menu-tools = Strumenti
# Followed by its shortcut, (M).
menu-tools-find-minibrot = Trova minibrot + zoom
menu-tools-find-minibrot-hover = Agganciare con Newton il centro della vista al centro esatto del minibrot vicino, indicarne il periodo e zoomare alla scala del minibrot stesso — spesso molti ordini di grandezza in un solo passo. Già più in profondità? Si sposta solo il centro.
menu-tools-solver = Risolutore Newton / Misiurewicz…
menu-tools-solver-hover = Calcolare il centro esatto del minibrot (periodo + dimensione) o il punto di Misiurewicz più vicino alla vista — metodo di Newton a precisione piena, con il moltiplicatore repulsivo λ. Apre la finestra della posizione, che ospita i risolutori.
# Followed by its shortcut, (A).
menu-tools-autopilot = Auto-zoom (pilota automatico)
menu-tools-autopilot-stop = Ferma pilota automatico
menu-tools-autopilot-hover = Zoom profondo continuo a mani libere: si tuffa verso la regione più ricca di dettagli, correggendo la rotta strada facendo. Qualsiasi comando di navigazione lo ferma.
menu-tools-priority = Priorità:
menu-tools-priority-speed = Velocità
menu-tools-priority-speed-hover = Zoomare alla velocità impostata; aggiornare con il primo fotogramma che ha dettagli.
menu-tools-priority-quality = Qualità
menu-tools-priority-quality-hover = Solo fotogrammi completamente risolti; lo zoom li aspetta.
menu-tools-target = Bersaglio:
menu-tools-target-detail = Dettaglio
menu-tools-target-detail-hover = Il bordo più ricco nella vista, scelto di nuovo man mano che il tuffo procede.
menu-tools-target-misiurewicz = Punto di Misiurewicz
menu-tools-target-misiurewicz-hover = Il punto di Misiurewicz più vicino, calcolato una volta e raggiunto esattamente (solo Mandelbrot).
menu-tools-orbits = Sovrapposizione orbita
menu-tools-orbits-hover = Disegnare il percorso delle iterazioni del punto sotto il cursore. Opzioni di animazione: pannello Controlli ▸ Sovrapposizioni.
menu-tools-orbits-normalize = Normalizza (adatta alla vista)
menu-tools-orbits-normalize-hover = Adattare l’orbita all’intera vista, così resta ben inquadrata a qualsiasi zoom (invece di essere mappata attraverso la finestra di visualizzazione, dove in profondità esce dallo schermo).
menu-tools-play-tour = Riproduci tour…
menu-tools-play-tour-hover = Scegliere uno script di tour da riprodurre. Il ▶ della barra degli strumenti riproduce l’ultimo con un solo clic.
menu-tools-tour-from-view = Tour dalla vista corrente…
menu-tools-tour-from-view-hover = Creare un tour che zooma dalla vista completa fino alla vista corrente — aggiungere una didascalia e impostare la durata del tuffo. Viene salvato come script di tour (.toml) da modificare, riprodurre o renderizzare.
menu-tools-render-tour = Renderizza tour…
menu-tools-render-tour-hover = Renderizzare il tour caricato in una sequenza di fotogrammi PNG (e facoltativamente un mp4) in un processo separato.
menu-tools-close-player = Chiudi lettore tour
menu-tools-close-player-hover = Fermare il tour, chiuderne il lettore e ripristinare le proprie impostazioni di iterazione e colorazione
menu-tools-render-farm = Renderizza sulla farm…
menu-tools-render-farm-hover = Renderizzare il tour caricato su più macchine: questa controlla il lavoro, le altre si uniscono con File ▸ Client di rendering….
menu-tools-benchmark = Benchmark…
menu-tools-benchmark-hover = Misurare la velocità di rendering — con le impostazioni correnti, o con un’esecuzione standardizzata (risoluzione e impostazioni fisse) confrontabile tra macchine. Il burn-in la ripete per verificare la stabilità / il thermal throttling.
menu-tools-last-benchmark = Mostra ultimo benchmark

## Navigate menu

menu-navigate = Naviga
menu-navigate-goto = Vai alla posizione…
menu-navigate-goto-hover = Inserire centro/zoom, saltare a un punto noto, o risolvere con Newton un punto di Misiurewicz / minibrot vicino alla vista.
menu-navigate-misiurewicz = Esploratore di Misiurewicz…
menu-navigate-misiurewicz-hover = Sfogliare l’aspetto dei piccoli tipi di Misiurewicz — miniature con il periodo di zoom e la torsione di ogni punto — e risolvere con Newton uno qualsiasi di essi a una profondità arbitraria. (Mandelbrot.)
menu-navigate-exact = Punti esatti (espressioni)
menu-navigate-exact-note = Radici dei bulbi della cardioide principale, date come espressioni esatte (cos/sin di una frazione di 2π). La coordinata resta un’espressione, così lo zoom rimane esattamente sul punto.
menu-navigate-random = Posizione casuale
menu-navigate-random-hover = Saltare a un punto casuale del bordo ricco di dettagli
menu-navigate-import-kfr = Importa .kfr…
menu-navigate-import-kfr-hover = Caricare un file di posizione di Kalles Fraktaler
menu-navigate-share = Condividi posizione…
menu-navigate-share-hover = Copiare / incollare / salvare / caricare una posizione autonoma (.fdn): frattale, centro a precisione piena, zoom, colorazione.

## Help menu

menu-help = Aiuto
# Followed by its shortcut, (F1).
menu-help-reference = Guida e riferimento…
menu-help-welcome = Benvenuto / guida rapida…
menu-help-welcome-hover = La panoramica del primo avvio: i comandi essenziali e alcuni luoghi celebri.
menu-help-diagnostics = Diagnostica…
menu-help-diagnostics-hover = Verificare che Fractadyne funzioni correttamente sul proprio hardware e allegare il risultato a una segnalazione
menu-help-report = Segnala un problema…
menu-help-report-hover = Aprire una segnalazione su GitHub (o via email) con rapporto dell’arresto anomalo, log e posizione allegati
menu-help-accelerated = Zoom profondo più veloce…
menu-help-accelerated-hover = Una build facoltativa che calcola le orbite di riferimento dello zoom profondo da 2,5 a 6,4 volte più velocemente. Immagini identiche; le impostazioni vengono mantenute.
menu-help-check-updates = Controlla aggiornamenti
# $track: "stable" or "beta", the update track in use (the variants hold the wording).
menu-help-check-updates-hover =
    { $track ->
        [beta] Cercare su GitHub una build Beta (pre-release) più recente
       *[other] Cercare su GitHub una build Stabile più recente
    }
# $version: the new version; $channel: "beta" for a pre-release, "stable" otherwise (the variants hold the wording).
menu-help-update-available =
    { $channel ->
        [beta] { $version } (beta) disponibile — scarica
       *[other] { $version } (stabile) disponibile — scarica
    }
menu-help-up-to-date = Aggiornato
menu-help-tagline = Esploratore di frattali nativo
menu-help-license = Licenza
# $year and $holder: the copyright year and holder, as the code gives them.
menu-help-copyright = © { $year } { $holder }. Doppia licenza; si può usare con l’una o l’altra licenza, a scelta.
menu-help-source = Sorgente

## The toolbar (its buttons are icons; these are their tooltips)

menu-toolbar-julia = Julia
menu-toolbar-julia-hover = Mostrare l’insieme di Julia di questa formula, a tutta finestra
menu-toolbar-dual-hover = Entrambi, collegati: l’insieme a sinistra, il Julia del punto sotto il cursore a destra
menu-toolbar-open-hover = Aprire una vista o una posizione (PNG / EXR, .fdn, .kfr)
menu-toolbar-gallery-hover = Galleria
menu-toolbar-export-hover = Esporta immagine…
menu-toolbar-share-hover = Condividi posizione — copiare, incollare, salvare o caricare un .fdn autonomo (frattale, centro a precisione piena, zoom, colorazione)
menu-toolbar-zoom-in-hover = Zoom avanti
menu-toolbar-zoom-out-hover = Zoom indietro
# $factor: the click-zoom factor (4, 10…).
menu-toolbar-click-zoom-hover = Clic per zoom ({ $factor }×): il clic sinistro si tuffa nel punto, il clic destro torna indietro (il trascinamento sposta ancora la vista). Fattore in Controlli ▸ Navigazione.
# Added to the click-to-zoom tooltip in the dual view.
menu-toolbar-click-zoom-dual = Con questa opzione attiva, il c di Julia si fissa con Ctrl+clic.
menu-toolbar-autopilot-running-hover = Auto-zoom in corso — clic per fermarlo
menu-toolbar-autopilot-hover = Auto-zoom: tuffo verso i dettagli (A)
menu-toolbar-reset-hover = Ripristina vista (istantaneo)
menu-toolbar-home-hover = Rimpicciolisci fino alla vista iniziale (animato)
menu-toolbar-bookmarks-hover = Segnalibri: salvare questa vista o saltare a una salvata
# $name: the palette's name.
menu-toolbar-next-palette-hover = Tavolozza successiva ({ $name })
# $file: the last tour's file name.
menu-toolbar-play-last-hover = Riproduci tour — { $file } (clic) · riproduce l’ultimo tour; scegline un altro da Strumenti ▸ Riproduci tour…
menu-toolbar-play-hover = Riproduci un tour… (Strumenti ▸ Riproduci tour)
menu-toolbar-speed-hover = Velocità di riproduzione
menu-toolbar-fullscreen-hover = Attiva/disattiva schermo intero

## Dialogs: words shared by many windows

# A button that closes a window without doing what it offers.
dlg-cancel = Annulla
# A button that dismisses a window (nothing in it is pending).
dlg-close = Chiudi
# A button that copies the window's text to the clipboard (after a copy icon).
dlg-copy = Copia
# A button that jumps to a location.
dlg-go = Vai
# A button that fills the fields from the view on screen.
dlg-use-current = Usa corrente
# A notice after a file could not be written. $error: the system's reason, as it gives it.
dlg-save-failed = Salvataggio non riuscito: { $error }
# A notice when no graphics device is available to render with.
dlg-gpu-not-available = GPU non disponibile
# The name of the plain-text file type in a save dialog (the extension .txt is added by the code).
dlg-filter-text = Testo

## Asked after a session that did not shut down cleanly

dlg-crash-title = Fractadyne non si è chiuso correttamente
dlg-crash-intro = La sessione precedente è terminata in modo imprevisto. Su questo computer è stato salvato un rapporto: vuoi inviarlo?
dlg-crash-review = Non viene inviato nulla prima che tu lo abbia controllato: la schermata successiva mostra esattamente che cosa verrebbe incluso, e le informazioni di sistema si escludono con una casella di controllo.
dlg-crash-report = Segnalalo…
dlg-crash-report-hover = Apre Segnala un problema, con il rapporto dell’arresto anomalo e la posizione in cui si è verificato già allegati.
dlg-crash-not-now = Non ora
dlg-crash-dont-ask = Non chiedere più dopo un arresto anomalo
dlg-crash-dont-ask-hover = I rapporti vengono comunque salvati in locale, e Aiuto ▸ Segnala un problema funziona sempre.

## Asked after a session that ended with a second graphics device in use

dlg-second-device-title = Disattivare il secondo dispositivo grafico?
# $device: the graphics device's name and details, as the system gives them.
dlg-second-device-intro = La sessione precedente è terminata in modo imprevisto mentre era in uso un secondo dispositivo grafico ({ $device }). Potrebbe esserne la causa.
dlg-second-device-note = Se è disattivato, la scheda che disegna la finestra renderizza tutto, come prima. Puoi riattivarlo in Avanzate ▸ Seconda scheda grafica.
dlg-second-device-off = Disattivalo
dlg-second-device-keep = Lascialo attivo

## Shown when the previous session's iteration setting was set aside

dlg-quarantine-title = Impostazione delle iterazioni accantonata
dlg-quarantine-change = Puoi cambiarla in qualsiasi momento in Controlli ▸ Qualità.
dlg-quarantine-keep = Mantieni il valore predefinito
# A button. $setting: the iteration setting the previous session used (such as "20,000 iterations").
dlg-quarantine-restore = Ripristina { $setting }

## Navigate ▸ Go to location

dlg-goto-title = Vai alla posizione
# Before two choices: "x, y" (cartesian) and the polar form below.
dlg-goto-enter-as = Inserisci come:
# The polar way of entering a point: an offset plus a radius r at an angle θ. Keep the symbols.
dlg-goto-polar = offset + r ∠ θ
# The tooltip of the "?" button beside the entry choice.
dlg-goto-help-hover = Che cosa accettano i campi: frazioni, potenze, sqrt/sin/cos…, pi e altre costanti — apre Aiuto → Espressioni delle coordinate
# Labels over the entry fields. x0, y0, r, θ, Re and Im are the fields' symbols: keep them.
dlg-goto-polar-offset = Centro dell’offset  (x0, y0 — lasciare 0 per un punto puramente polare)
dlg-goto-polar-radius = Raggio r, angolo θ
dlg-goto-re = Centro — reale (Re)
dlg-goto-im = Centro — immaginario (Im)
dlg-goto-zoom = Zoom (ingrandimento)
# A warning under the zoom field, after a ⚠ sign. $have: the iteration count set; $typical: the count this depth typically needs.
dlg-goto-iter-warning = Le iterazioni sono fissate a { $have }; questa profondità di solito ne richiede ~{ $typical }. La vista resterà piatta finché non verranno aumentate — oppure attivare le iterazioni automatiche (Controlli ▸ Qualità), che si adattano da sole.
dlg-goto-copy-hover = Copiare questa posizione negli appunti
dlg-goto-cancel-hover = Chiudere senza andare da nessuna parte
# The expressions in brackets are examples of what can be typed: keep them as they are.
dlg-goto-paste-note = Incollare un centro/zoom ricevuto da altri, oppure Copia per condividere. Le coordinate accettano espressioni: frazioni esatte (-3/4), funzioni e costanti (-0.5 + 0.25*cos(pi/4), sqrt, root(x,n), ^; angoli in radianti), e il campo reale accetta un intero valore complesso ((37+16i)/100), che riempie entrambi i campi.
# A section that opens and closes.
dlg-goto-feature = Vai a una struttura (Mandelbrot)
dlg-goto-feature-poi = Punti noti:
# The dropdown's text before a point is picked.
dlg-goto-feature-poi-pick = Salta a un punto d’interesse…
dlg-goto-feature-solve = Oppure calcola una struttura vicina alla vista corrente:
# Two choices: what to solve for. Misiurewicz is a mathematician's name (a Misiurewicz point).
dlg-goto-feature-misiurewicz = Misiurewicz
dlg-goto-feature-minibrot = Minibrot più vicino
# Labels of two small number fields; k and p are their symbols.
dlg-goto-feature-preperiod = Preperiodo k
dlg-goto-feature-period = Periodo p
# The faint text in an empty field: left empty, the value is found automatically.
dlg-goto-feature-auto-hint = auto
# A small button that empties both fields, so they are found automatically again.
dlg-goto-feature-auto = Auto
dlg-goto-feature-auto-hover = Svuotare entrambe le caselle, così la ricerca successiva rileva preperiodo e periodo dall’orbita al centro della vista.
dlg-goto-feature-kp-hover = Lasciarle entrambe vuote per rilevarli dall’orbita al centro della vista; i valori trovati vengono inseriti qui. Nessuno, guardando una spirale, ne conosce il preperiodo, e doverlo fornire è ciò che rendeva questo risolutore inutilizzabile in pratica.
# While a solve runs, followed by its elapsed time and the note below.
dlg-goto-feature-solving = Risoluzione…
dlg-goto-feature-solving-note = (precisione arbitraria; le viste profonde richiedono più tempo)
dlg-goto-feature-cancel-hover = Smettere di aspettare. Il calcolo prosegue fino alla fine in background e la sua risposta viene scartata — nulla resta a metà.
dlg-goto-feature-find = Trova vicino alla vista
dlg-goto-feature-find-hover =
    Risolvere con Newton dal centro corrente fino alla struttura esatta (precisione arbitraria). Prima, spostarsi vicino alla struttura.

    Impostare il campo Zoom qui sopra alla profondità VOLUTA prima di risolvere: il punto viene calcolato con la precisione richiesta da quella profondità, e la vista salta direttamente lì. È un solo calcolo invece di una lunga serie di passi di zoom manuali.
dlg-goto-solve-cancelled = Calcolo annullato.

## Snapshot (the camera button): its tooltip, and the choice on the first press

# The camera button's tooltip, one per Snapshot setting. Followed by its shortcut, (Ctrl+S).
dlg-snapshot-hover-ask = Istantanea — salva la vista come appare, o un rendering completo; alla prima pressione chiede
dlg-snapshot-hover-screen = Istantanea — salva la vista esattamente come appare, nell’ultima cartella di esportazione
dlg-snapshot-hover-render = Istantanea — rendering completo con le impostazioni di esportazione, nell’ultima cartella di esportazione
# The window's title.
dlg-snapshot-title = Istantanea
dlg-snapshot-question = Che cosa deve salvare Istantanea (il pulsante della fotocamera, Ctrl+S)?
# The first choice, and a short tag beside it.
dlg-snapshot-screen = Cattura dello schermo
dlg-snapshot-screen-tag = immediato
dlg-snapshot-screen-note = Salva la vista esattamente come appare sullo schermo, alla risoluzione dello schermo, come PNG nell’ultima cartella di esportazione. Ciò che vedi è ciò che ottieni.
# The second choice, and a short tag beside it.
dlg-snapshot-render = Rendering completo
dlg-snapshot-render-tag = può richiedere molto tempo
# $size: the image size, as 1920×1080; $ss: the supersampling factor (1–4); $format: PNG or EXR.
dlg-snapshot-render-note = Renderizza con le impostazioni di esportazione — { $size }, supersampling { $ss }×, { $format }. La qualità migliore, ma una vista profonda può richiedere da minuti a ore. Viene eseguito in background; File → Esporta immagine… ne mostra l’avanzamento e può annullarlo.
dlg-snapshot-remember = Ricorda la mia scelta e non chiedere più
dlg-snapshot-change = Puoi cambiarla in qualsiasi momento in File → Impostazioni → Istantanea.

## Share location (.fdn)

dlg-share-title = Condividi posizione
dlg-share-intro = Una posizione autonoma (frattale, centro a precisione piena, zoom, colorazione). Copiala per condividerla, oppure incollane o caricane una e premi Applica.
dlg-share-thumb = Includi una miniatura nel file salvato
dlg-share-thumb-hover = «Salva .fdn…» incorpora una piccola immagine di questa vista, così il file si riconosce nella galleria. Aggiunge circa 55 KB e non fa parte del testo qui sopra.
dlg-share-copy-hover = Copiare il testo negli appunti
# Buttons (the first after a save icon).
dlg-share-save = Salva .fdn…
dlg-share-load = Carica .fdn…
dlg-share-apply = Applica
dlg-share-apply-hover = Saltare alla posizione nella casella
dlg-share-copied = Copiato negli appunti.

## Help ▸ Report an issue (the report itself stays in English: it is read by the maintainer)

dlg-report-title = Segnala un problema
dlg-report-intro = Il posto migliore per una segnalazione è una issue su GitHub — è pubblica e ricercabile, e vedrai quando verrà risolta. Va bene anche l’email. Controlla prima tutto ciò che segue: nulla lascia il tuo computer finché non agisci.
# Labels before three dropdowns.
dlg-report-type = Tipo:
dlg-report-severity = Gravità:
dlg-report-repro = Riproducibile:
dlg-report-describe = Descrivi che cosa è successo:
# The faint text in the empty description box.
dlg-report-describe-hint = Che cosa è successo, i passi per riprodurlo e che cosa ti aspettavi invece.
# Before the checkboxes of what the report carries.
dlg-report-include = Includi:
dlg-report-sysinfo = Informazioni di sistema (versione, SO, CPU, GPU, VRAM)
dlg-report-location = Posizione corrente (.fdn)
dlg-report-log = Log recente
dlg-report-frames = Registro dei fotogrammi (l’ultimo minuto e gli ultimi 40 fotogrammi)
dlg-report-crash = Ultimo rapporto di arresto anomalo
dlg-report-crash-none = Ultimo rapporto di arresto anomalo (nessuno trovato)
dlg-report-crash-view = Posizione dell’arresto anomalo (.fdn)
dlg-report-crash-view-none = Posizione dell’arresto anomalo (.fdn) (nessuna trovata)
dlg-report-crash-view-hover = La vista esatta in cui si è verificato un arresto anomalo o una perdita del dispositivo grafico — l’unica cosa che permette a un manutentore di riprodurlo. Registrata automaticamente.
dlg-report-test = Risultato del test di diagnostica
dlg-report-test-none = Risultato del test di diagnostica (eseguirne uno da Aiuto → Diagnostica…)
dlg-report-test-hover = Un esito superato/non superato convalidato dalla macchina sul tuo hardware — molto più utile a un manutentore di una semplice descrizione
# A section that opens and closes. $size: the report's size, as "12 KB".
dlg-report-preview = Anteprima di ciò che verrà inviato  ({ $size })
dlg-report-github = Apri una issue su GitHub
dlg-report-github-hover = Aprire la pagina di una nuova issue con il titolo già compilato; il rapporto viene copiato negli appunti — incollarlo nel corpo della issue (Ctrl+V).
# Buttons (after a copy and a save icon).
dlg-report-copy = Copia rapporto
dlg-report-copy-hover = Copiare tutto negli appunti
dlg-report-save = Salva rapporto…
dlg-report-save-hover = Salvare come .txt da allegare
dlg-report-by-email = Oppure via email:
dlg-report-gmail = Scrivi in Gmail
# $address: the email address reports go to.
dlg-report-gmail-hover = Aprire una finestra di composizione di Gmail verso { $address } con l’oggetto compilato; incollare il rapporto copiato
dlg-report-email = App di posta…
dlg-report-email-hover = Aprire l’app di posta predefinita (mailto) — richiede un gestore email configurato
dlg-report-tip = Suggerimento: con ognuno di questi pulsanti il rapporto viene copiato negli appunti — incollalo nel corpo della issue o del messaggio (Ctrl+V). Se puoi, allega anche uno screenshot o un’immagine di esempio (trascinala dentro); aiuta molto con i problemi di rendering e di interfaccia.
# Notices in the window after a button is pressed.
dlg-report-opened-github = GitHub aperto — il rapporto è negli appunti; incollalo nella issue.
dlg-report-copied = Rapporto copiato negli appunti.
# $path: where the report was saved.
dlg-report-saved = Salvato in { $path }
dlg-report-opened-email = App di posta aperta — il rapporto è negli appunti; incollalo.
dlg-report-opened-gmail = Composizione di Gmail aperta — incolla il rapporto copiato nel corpo (Ctrl+V).

## A location whose checksum does not match

dlg-checksum-title = Questa posizione potrebbe essere danneggiata
# After a ⚠ sign.
dlg-checksum-mismatch = Il suo checksum non corrisponde ai dati che lo accompagnano.
dlg-checksum-why = Di solito significa che il testo è stato troncato o alterato lungo la strada — una copia parziale da una finestra di chat o da un post di un forum è la causa più comune.
# A label over where the location came from (a file path, or the clipboard).
dlg-checksum-source = Origine
# The two checksums, side by side: the one written in the file, and the one computed from its data.
dlg-checksum-in-file = nel file
dlg-checksum-from-data = dai dati
dlg-checksum-load-note = Puoi caricarla comunque — le coordinate potrebbero essere ancora corrette, e nulla viene sovrascritto. Ciò che risulta illeggibile viene segnalato dopo il caricamento.
dlg-checksum-load = Carica comunque
dlg-checksum-load-hover = Saltarci nonostante la discrepanza
# Notices after loading it. $zoom: the magnification, as 1.2e148.
dlg-checksum-loaded = Caricata @ { $zoom }× nonostante la discrepanza del checksum
# $zoom: the magnification, as 1.2e148; $note: what the loader found wrong with the location.
dlg-checksum-loaded-note = Caricata @ { $zoom }× — { $note }
dlg-checksum-not-loaded = Non caricata.

## File ▸ Reset application state

dlg-reset-title = Reimposta stato dell’applicazione
# After a ⚠ sign, followed by a list.
dlg-reset-warning = Questa operazione elimina definitivamente tutti i dati salvati di Fractadyne:
# The list's items, each after a bullet.
dlg-reset-session = la sessione salvata (vista corrente, colorazione, preferenze)
dlg-reset-bookmarks = tutti i segnalibri e le loro miniature
dlg-reset-refcache = la cache delle orbite di riferimento (ricostruita quando si torna nelle posizioni)
# A label over the folder that is deleted.
dlg-reset-location = Percorso
dlg-reset-note = Non si può annullare. La sessione corrente non verrà salvata di nuovo all’uscita, quindi al prossimo avvio verranno caricati i valori predefiniti.
dlg-reset-confirm = Reimposta tutto
dlg-reset-done = Stato dell’applicazione reimpostato — al prossimo avvio verranno caricati i valori predefiniti.
# $error: the system's reason, as it gives it.
dlg-reset-failed = Reimpostazione non riuscita: { $error }

## Tools ▸ Tour from current view

dlg-tour-title = Tour dalla vista corrente
dlg-tour-intro = Crea un tour che zooma dalla vista completa fino al punto in cui ti trovi ora.
# A label before the fractal and magnification the tour ends at.
dlg-tour-destination = Destinazione
dlg-tour-notation = Annotazione (didascalia facoltativa mostrata durante il tuffo)
# The faint example text in the empty caption box.
dlg-tour-notation-hint = es. Tuffo verso il Misiurewicz a tre bracci…
dlg-tour-duration = Durata dello zoom
# $rate: how fast the dive zooms, in powers of ten a second (as 2.5).
dlg-tour-rate = ≈ { $rate } decadi/s
# A small button that puts the duration back to its default.
dlg-tour-reset = Ripristina
dlg-tour-reset-hover = Valore predefinito in base alla profondità
dlg-tour-duration-note = Il valore predefinito cresce con la profondità, così il tuffo resta guardabile. Tuffi più rapidi possono far lampeggiare fotogrammi di un solo colore a zoom estremi.
# A button, after a save icon.
dlg-tour-save = Salva tour…
# The file type's name in the save dialog.
dlg-tour-filter = Script Fractadyne (TOML)
# A notice. $file: the saved file's name. --render-tour is a command-line option: keep it.
dlg-tour-saved = Tour salvato → { $file }. Riproducilo da Strumenti → Riproduci tour, oppure renderizzalo con --render-tour.

## Navigate ▸ Bookmarks ▸ Manage…

dlg-bookmarks-title = Segnalibri
# The faint text in the empty name box.
dlg-bookmarks-name-hint = nome (facoltativo)
# A button, after a bookmark icon.
dlg-bookmarks-add = Aggiungi vista corrente
dlg-bookmarks-empty = Ancora nessun segnalibro. Aggiungine uno dalla vista corrente.
# The tooltip of a bookmark's delete (bin) button.
dlg-bookmarks-delete-hover = Elimina

## Tools ▸ Benchmark…

dlg-bench-title = Benchmark
dlg-bench-mode = Modalità
dlg-bench-current = Impostazioni correnti
dlg-bench-current-hover = Riprodurre il tour fisso di zoom profondo nella vista in tempo reale con la risoluzione e le impostazioni correnti.
dlg-bench-standard = Standardizzato
dlg-bench-standard-hover = Fissare la risoluzione e tutte le impostazioni di rendering, così il punteggio è confrontabile tra macchine.
dlg-bench-resolution = Risoluzione
dlg-bench-depth = Profondità
# Standard and Ultra are the two depths in the dropdown this describes.
dlg-bench-depth-hover = Entrambi i tuffi entrano nel percorso profondo floatexp/BLA; Ultra profondo (1e48×) vi trascorre molto più tempo di Standard (1e32×), e richiede di conseguenza più tempo.
dlg-bench-burnin = Burn-in (ripetizione)
dlg-bench-burnin-hover = Eseguire il benchmark ripetutamente per rivelare stabilità e thermal throttling.
# The label of a slider: how many times the benchmark runs.
dlg-bench-passes = passaggi
# $size: the image size, as 1920×1080; $ss: the supersampling factor; $depth: the dive's final magnification, as 1e32.
dlg-bench-std-summary = Renderizza fuori schermo a { $size }, SS { $ss }×, Mandelbrot/iterazione continua, tuffo di 60 fotogrammi fino a { $depth }×.
dlg-bench-run = Esegui

## The standardized benchmark's progress window

dlg-bench-progress-title = Benchmark in corso…
# $resolution: the benchmark's resolution, as the dropdown names it.
dlg-bench-progress-mode = Standardizzato · { $resolution }
# $pass: the pass running now; $total: how many passes there are.
dlg-bench-progress-pass = Passaggio di burn-in { $pass }/{ $total }
dlg-bench-progress-dive = Rendering del tuffo fisso…
# On the progress bar. $done: frames rendered; $total: frames in the dive.
dlg-bench-progress-frame = fotogramma { $done }/{ $total }
# $fps: frames per second, as 41.3.
dlg-bench-progress-last = ultimo passaggio: { $fps } fps

## The benchmark's results

dlg-bench-results-title = Risultati del benchmark
# Buttons, each after an icon.
dlg-bench-results-run-again = Esegui di nuovo…
dlg-bench-results-save = Salva…
dlg-bench-results-none = Non è ancora stato eseguito alcun benchmark.
dlg-bench-results-saved = Benchmark salvato.

## The accelerated build's notice when its arithmetic libraries are missing

dlg-backend-title = Build accelerata — in uso l’aritmetica integrata
dlg-backend-dont-show = Non mostrare più

## File ▸ Gallery…

dlg-gallery-title = Galleria
# A button that picks the folder to show.
dlg-gallery-folder = Cartella…
# A button, after a refresh icon.
dlg-gallery-refresh = Aggiorna
dlg-gallery-empty = Nessuna immagine di Fractadyne in questa cartella.
# An image's title when it does not name its fractal.
dlg-gallery-untitled = Immagine di Fractadyne
# $zoom: the image's magnification, as 1.2e148; $saved: when it was saved, as the file gives it.
dlg-gallery-zoom = zoom { $zoom }   { $saved }
# The notes saved with the image, in quotation marks. Use your language's quotation marks.
dlg-gallery-notes = «{ $notes }»
dlg-gallery-open = Apri questa vista
dlg-gallery-loaded = Vista caricata dalla galleria.

## File ▸ Export image…

dlg-export-title = Esporta immagine
# A dropdown of image sizes.
dlg-export-size = Dimensione
# The size dropdown's text for a size set by hand. $size: the size, as 1920×1080.
dlg-export-size-custom-current = Personalizzata — { $size }
# The size dropdown's first choice.
dlg-export-size-custom = Personalizzata…
dlg-export-size-custom-hover = Impostare la larghezza a mano; l’altezza segue le Proporzioni qui sotto.
dlg-export-width = Larghezza (px)
# After the width field. $height: the height in pixels, which follows from the aspect.
dlg-export-height-from-aspect = × { $height } (dalle Proporzioni)
dlg-export-ss = Supersampling
dlg-export-aspect = Proporzioni
# The aspect choice that keeps the window's own shape.
dlg-export-aspect-window = Come la finestra
# Before the choices PNG and OpenEXR.
dlg-export-format = Formato:
dlg-export-hud = HUD della posizione
dlg-export-hud-hover = Imprimere un pannello con livello di zoom + coordinate nell’angolo in alto a sinistra dell’immagine esportata (si adatta all’output; mostra zoom + centro della mappa/vista Mandelbrot).
dlg-export-glitch = Correzione dei glitch
dlg-export-glitch-hover = Correzione multi-riferimento dei glitch di perturbazione. Saltata automaticamente per le esportazioni a vista singola molto profonde (floatexp), così la costruzione del riferimento + il rendering girano fuori dal thread principale e l’app resta reattiva. Mai usata per le viste Mandelbrot o Multibrot (fuori dalla modalità Julia): verificata con un calcolo esatto, lì non correggeva nulla e costava fino a 5 volte il tempo di esportazione.
dlg-export-deep-note = Esportazione profonda: il riferimento si costruisce fuori dal thread principale (l’interfaccia resta attiva); a questa profondità la correzione dei glitch viene saltata.
# A dropdown, shown in the dual (map + Julia) view, and its three choices.
dlg-export-dual = Disposizione doppia
dlg-export-dual-side = Affiancate
dlg-export-dual-separate = File separati
dlg-export-dual-map = Solo mappa
dlg-export-notes = Note:
# The faint text in the empty notes box.
dlg-export-notes-hint = salvate con l’immagine (≤120 caratteri)
# $width and $height: the image size in pixels; $left: how many characters the notes can still take.
dlg-export-output = Output: { $width } × { $height } px   (ancora { $left } caratteri)
# When the export's shape differs from the window's. $percent: how much more, as 25.
dlg-export-extra-width = Mostra il { $percent }% di larghezza in più rispetto alla finestra (nulla viene ritagliato)
# The same, when the export is taller than the window. $percent: how much more, as 25.
dlg-export-extra-height = Mostra il { $percent }% di altezza in più rispetto alla finestra (nulla viene ritagliato)
dlg-export-tiles-note = Renderizzata a riquadri (nessun limite di dimensione) su un thread in background. Il file incorpora la vista + le note, così si può riaprire con File ▸ Apri vista o posizione….
# A label before the button that picks the folder exports go to.
dlg-export-folder = Cartella:
dlg-export-choose = Scegli…
dlg-export-choose-hover = Scegliere la cartella di destinazione delle esportazioni
dlg-export-open-after = Apri al termine
dlg-export-open-after-hover = Passare l’immagine finita al visualizzatore di sistema appena viene scritta.
# $count: the graphics cards in this machine (2 or more).
dlg-export-all-gpus = Usa tutte le schede grafiche ({ $count })
dlg-export-all-gpus-hover = Dividere l’immagine per riquadri tra tutte le schede grafiche di questa macchina, ciascuna con la propria copia del riferimento profondo. Schede identiche danno esattamente l’immagine che una sola renderizza da sola; modelli diversi disegnano alcuni pixel in modo leggermente diverso, riquadro per riquadro (misurato: nessuna giunzione visibile). Se una scheda si guasta, i suoi riquadri passano alle altre. Le immagini che la suddivisione non può gestire (alcuni metodi di colorazione e formule) usano la scheda di questa finestra.
# The export's progress.
dlg-export-preparing = Preparazione (costruzione del riferimento)…
dlg-export-saving = Salvataggio…
dlg-export-rendering = Rendering…
# $time: the time since the export started, as 8.3s or 1m 04.0s.
dlg-export-elapsed = Trascorso: { $time }
dlg-export-cancel-hover = Chiudere senza esportare
# A button, after a save icon.
dlg-export-save-as = Salva con nome…
dlg-export-save-as-hover = Scegliere nome e posizione del file
dlg-export-export = Esporta
dlg-export-export-hover = Renderizzare e salvare nella cartella qui sopra (nome automatico)

## Notices worded at startup (main.rs)

# A notice shown once after starting, when the saved session comes from a newer version of the app. $saved and $handled: version numbers of the session file's format (1, 2…).
app-state-newer = Questa sessione è stata salvata da una versione più recente di Fractadyne (stato v{ $saved }; questa build gestisce v{ $handled }). Alcune impostazioni potrebbero non applicarsi, e il salvataggio la riscriverà nel formato di questa build.
# The title of the notice that says the accelerated build's arithmetic libraries are found again (its text comes from elsewhere).
app-notice-mpfr-restored-title = Aritmetica accelerata ripristinata
# After Fractadyne restarted itself because the graphics driver reset the GPU.
app-toast-gpu-recovered = Ripristino dopo un reset del dispositivo grafico — riaperto alla vista iniziale, perché la posizione precedente era troppo impegnativa da renderizzare.

## Short notices (toasts)

# When the autopilot reached a view that could not be rendered and put the last good view back.
app-toast-autopilot-view-restored = Pilota automatico fermato: la vista era diventata inutilizzabile ed è stata ripristinata (annotato nel log)
# $error: the system's reason, as it gives it (often in English).
app-toast-gradients-save-failed = Impossibile salvare i gradienti: { $error }
# $error: the reason, as the program gives it (often in English).
app-toast-gradients-serialize-failed = Impossibile serializzare i gradienti: { $error }
# $name: the name the user gave the gradient.
app-toast-gradient-saved = Gradiente «{ $name }» salvato.
# $error: the reason, as the program gives it (often in English).
app-toast-bookmarks-serialize-failed = Impossibile serializzare i segnalibri: { $error }
# $error: the system's reason, as it gives it (often in English).
app-toast-bookmarks-save-failed = Impossibile salvare i segnalibri: { $error }
# Jumping to a named point given by exact expressions. $name: the point's name (in English).
app-toast-coordinates-failed = { $name }: impossibile valutare le coordinate
# $zoom: the magnification, already formatted (1,234.5 or 1.2e148).
app-toast-random-location = Posizione casuale · { $zoom }×
# Importing a Kalles Fraktaler .kfr location. $error: the reason (in English).
app-toast-import-failed = Importazione non riuscita: { $error }
# $zoom: the magnification, already formatted.
app-toast-location-loaded = Posizione caricata @ { $zoom }×

## Go to location: why a jump could not be made (one line per field that failed)

# "Re" is the dialog's real-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-re = Re: { $error }
# "Im" is the dialog's imaginary-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-im = Im: { $error }
# The imaginary field was given a complex value. $error: the evaluator's reason (in English).
app-goto-error-im-real = Im: { $error } — scrivere la parte immaginaria come numero reale, senza la i
# "Zoom" is the dialog's magnification field.
app-goto-error-zoom-empty = Zoom: inserire un ingrandimento, ad es. 1e50 o 250000
# $zoom: what was typed, in quotes.
app-goto-error-zoom = Zoom: impossibile leggere { $zoom } — inserire un ingrandimento positivo come 1e50 o 250000
app-goto-error-invalid = Input non valido — controllare le coordinate e lo zoom.

## Finding a minibrot or Misiurewicz point (the M key and the Go to dialog)

app-feature-needs-holomorphic = La ricerca dei minibrot richiede una famiglia olomorfa (Mandelbrot / Multibrot).
app-feature-not-in-julia = La ricerca dei minibrot esplora l’insieme di Mandelbrot — disattiva la modalità Julia per usarla.
app-feature-busy = Ricerca di una struttura già in corso — una alla volta.
app-feature-searching = Ricerca del minibrot più vicino…
app-feature-mandelbrot-only = La ricerca delle strutture funziona solo con Mandelbrot.
app-feature-no-pair = Nessun punto di Misiurewicz trovato entro il budget di ricerca — il punto potrebbe essere più profondo di quanto la ricerca raggiunga da questa vista, oppure il centro potrebbe non essere vicino a una spirale o a un punto di diramazione.
# The name of what was found, used as $feature in the messages below. $period: the minibrot's period (a whole number).
app-feature-minibrot = minibrot di periodo { $period }
# $preperiod and $period: the point's two whole numbers (k, p). $octaves: how often the view repeats when zooming in (2 decimals); $twist: the turn per repeat in degrees.
app-feature-misiurewicz-lambda = Misiurewicz ({ $preperiod },{ $period }) — si ripete ogni { $octaves } ottave, torsione { $twist }°
# $preperiod and $period: the point's two whole numbers (k, p).
app-feature-misiurewicz = Misiurewicz ({ $preperiod },{ $period })
# Added after the messages below when the fixed iteration count is too low for the new depth. $have: the iterations set now; $typical: what the depth typically needs.
app-feature-iter-note = ⚠ Le iterazioni sono fissate a { $have }; questa profondità di solito ne richiede ~{ $typical } — aumentale (o attiva le iterazioni automatiche) se la vista risulta piatta.
# $feature: what was found (see above). $zoom and $asked: magnifications, already formatted; $octaves: the solver's depth limit.
app-feature-zoomed-capped = Zoom sul { $feature } — { $zoom }×. La risoluzione è limitata a { $octaves } ottave, quindi si è fermata lì invece che a { $asked }×.
# $feature: what was found. $zoom: the magnification, already formatted. $seconds: how long the search took (1 decimal).
app-feature-zoomed = Zoom sul { $feature } — { $zoom }× ({ $seconds } s)
# $feature: what was found. $seconds: how long the search took (1 decimal).
app-feature-snapped = Agganciato al centro del { $feature } ({ $seconds } s)
app-feature-no-minibrot = Nessun centro di minibrot trovato vicino alla vista — avvicinati a uno con lo zoom.
# $feature: the point asked for, e.g. Misiurewicz (3,1). $widths: how many view-widths away, already formatted. $octaves: how far to zoom out.
app-feature-just-outside = Trovato un punto { $feature } appena fuori da questa vista — a { $widths } larghezze di vista. Rimpicciolisci di circa { $octaves } ottave per portarlo sullo schermo, oppure imposta un preperiodo più grande per una struttura a questa scala.
# $feature: the point asked for. $widths: how many view-widths away, already formatted (may be 2^187). $octaves: how far to zoom out.
app-feature-far = Trovato un punto { $feature }, ma si trova a { $widths } larghezze di vista — quella struttura è di scala molto più GRANDE di questa vista. Fai zoom INDIETRO di circa { $octaves } ottave per raggiungerla, oppure prova un preperiodo più grande per una struttura a questa scala.
# $feature: the point asked for. $residual: a small number in scientific notation.
app-feature-not-preperiodic = { $feature } non corrisponde all’orbita qui (residuo { $residual }) — svuota entrambe le caselle per rilevare la coppia dalla vista.
# $feature: the point asked for. "k/p" are the dialog's two fields.
app-feature-not-converged = Nessun punto { $feature } è arrivato a convergenza vicino alla vista — avvicinati, oppure prova altri k/p.
app-feature-bad-request = Preperiodo e periodo devono essere entrambi positivi.

## Share location dialog and location files

app-location-empty = Niente da caricare (o testo troppo grande).
app-location-not-fractadyne = Non è una posizione di Fractadyne.
# $zoom: the magnification, already formatted. $note: what in the text was ignored or repaired (in English).
app-share-loaded-note = Caricata @ { $zoom }× — { $note }.
# $error: the system's reason (often in English).
app-share-save-failed = Salvataggio non riuscito: { $error }
app-share-saved = Salvato.
app-share-saved-rendering-thumb = Salvato. Rendering della miniatura…
app-share-saved-no-thumb = Salvato — senza miniatura (per questa vista non è stato possibile renderizzarne una).
app-share-saved-with-thumb = Salvato, con una miniatura.
# $error: the system's reason (often in English).
app-share-saved-thumb-failed = Salvato, ma non è stato possibile aggiungere la miniatura: { $error }
app-share-loaded-into-box = Caricata nella casella — controllala, poi premi Applica.
# Reading a file failed (a location or a palette). $error: the system's reason (often in English).
app-read-failed = Lettura non riuscita: { $error }
app-share-too-large = File troppo grande (non è una posizione .fdn?).

## File-dialog filter names (the extensions are added by the code)

app-filter-kfr = Posizione di Kalles Fraktaler
app-filter-fdn = Posizione di Fractadyne
app-filter-map = Tavolozza Fractint / KF
app-filter-ugr = Gradiente di Ultra Fractal
app-filter-ase = Adobe Swatch Exchange
app-filter-ggr = Gradiente di GIMP
app-filter-all = Tutti i file

## Importing and saving palette files (shown under the gradient editor)

# The reason a file was refused before reading it.
app-gradient-map-too-large = decisamente troppo grande per essere una tavolozza .map
app-gradient-ugr-too-large = decisamente troppo grande per essere un file di tavolozza .ugr
app-gradient-ase-too-large = decisamente troppo grande per essere un elenco di campioni .ase
app-gradient-ggr-too-large = decisamente troppo grande per essere un gradiente .ggr
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported-vga = Importate { $count } voci da { $file } - una tabella VGA a 6 bit (il bianco è 252, non 255), mantenuta così com’è scritta, in modo che corrisponda all’output di Fractint stesso.
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported = Importate { $count } voci da { $file }, come fasce nette.
# $error: the reader's reason (in English).
app-gradient-map-invalid = Non è una tavolozza .map: { $error }
# $name: the gradient's name in the file; $file: the file's name.
app-gradient-ugr-imported = Importato «{ $name }» da { $file }.
# $file: the file's name; $count: how many gradients it holds.
app-gradient-ugr-pick = { $file } contiene { $count } gradienti — sceglierne uno qui sotto.
# $error: the reader's reason (in English).
app-gradient-ugr-invalid = Non è un file di tavolozza .ugr: { $error }
# $used: the swatches kept; $total: the swatches in the file.
app-gradient-ase-imported-some = Importati { $used } campioni su { $total }, distribuiti uniformemente.
# $count: the swatches in the file.
app-gradient-ase-imported = Importati { $count } campioni, distribuiti uniformemente e sfumati.
# $error: the reader's reason (in English).
app-gradient-ase-invalid = Non è un elenco di campioni .ase: { $error }
app-gradient-nothing-to-save = Ancora nessun gradiente personalizzato da salvare.
# $file: the saved file's name.
app-gradient-ggr-saved = Salvato «{ $file }» — punti medi, curve di fusione e spazi colore tutti intatti.
# $file: the saved file's name; $count: the segments whose curve could not be kept.
app-gradient-ggr-saved-lossy =
    { $count ->
        [one] Salvato «{ $file }» — ma { $count } segmento usa una curva di Bézier che il formato .ggr non può rappresentare. Il segmento è stato salvato come sfumatura lineare; la copia nella tua libreria conserva la curva.
       *[other] Salvato «{ $file }» — ma { $count } segmenti usano curve di Bézier che il formato .ggr non può rappresentare. I segmenti sono stati salvati come sfumature lineari; la copia nella tua libreria conserva le curve.
    }
# $path: the file; $error: the system's reason (often in English).
app-gradient-write-failed = Impossibile scrivere { $path }: { $error }
# $name: the gradient's name; $count: its segments.
app-gradient-ggr-imported = Importato «{ $name }» - { $count } segmenti, con punti medi, curve di fusione e spazi colore.
# $error: the reader's reason (in English).
app-gradient-ggr-invalid = Non è un gradiente .ggr: { $error }

## The eyedropper's banner across the top of the screen while picking a colour

# $hex: the colour under the pointer, as #rrggbb.
app-eyedropper-banner-color = Selezione di un colore  ·  { $hex }  ·  clic per prenderlo, Esc per annullare
app-eyedropper-banner = Selezione di un colore  ·  clic per prenderlo, Esc per annullare

## The gradient editor window

app-gradient-title = Editor dei gradienti
# Before the two view buttons, Bar and Ring.
app-gradient-view = Vista
app-gradient-view-bar = Barra
app-gradient-view-bar-hover = Una striscia dritta — ideale per posizionare i punti di colore con precisione
app-gradient-view-ring = Anello
app-gradient-view-ring-hover = Un anello, perché la tavolozza è CICLICA: la giunzione in alto è la cucitura che il renderer attraversa a ogni giro, e un gradiente che sembra a posto su una barra lì può avere un bordo netto.
app-gradient-add-lane-hover = Clic in un punto qualsiasi di questa linea per aggiungervi un punto di colore. Il + ne aggiunge uno nello spazio più ampio.
app-gradient-strip-hover = Trascinare un indicatore per spostarne il punto · trascinare la striscia stessa per RUOTARE l’intero gradiente, che si riavvolge alla cucitura · doppio clic per aggiungere un punto, clic destro su un indicatore per rimuoverlo · i tasti freccia spostano di poco la selezione (Shift per un passo fine).
app-gradient-remove-selected-hover = Rimuovere il punto selezionato
app-gradient-ring-hover = Trascinare l’anello stesso per RUOTARE l’intero gradiente — ciò che supera la cucitura ritorna dall’altro lato · trascinare un indicatore per spostarne il punto · clic sulla traccia esterna per aggiungervi un punto, o su ⊕ al centro per aggiungerne uno nello spazio più ampio · ⊖ rimuove il punto selezionato. Il segno vuoto in alto è la cucitura, dove una tavolozza ciclica si riavvolge; resta fermo perché È la giunzione, e i colori ruotano sotto di esso.
app-gradient-empty = Ancora nessun gradiente personalizzato — «Copia preset…», un’importazione o un gradiente salvato qui sotto ne avvia uno, e qui compaiono i suoi punti e le curve per segmento.
# $count: the entries of an imported palette, more than the editor shows stop by stop.
app-gradient-too-many = { $count } voci importate - troppe da modificare punto per punto. Usa l’anteprima qui sopra, oppure Copia preset per ricominciare.
# The selected segment's heading. $index: its number; $total: how many there are; $left and $right: where it starts and ends, 0.000–1.000.
app-gradient-segment-heading = Segmento { $index } di { $total }  ·  { $left } → { $right }
# A label before the list of blend curves.
app-gradient-curve = Curva
app-gradient-curve-hover = Quanto velocemente il colore attraversa questo segmento. «Curvo» è modellata dal punto medio — con punto medio 0.50 è esattamente lineare.
# The blend curves (GIMP's kinds), in a narrow list: keep them short.
app-gradient-blend-linear = Lineare
app-gradient-blend-curved = Curvo
app-gradient-blend-sine = Sinusoidale
app-gradient-blend-sphere-increasing = Sferico ↑
app-gradient-blend-sphere-decreasing = Sferico ↓
app-gradient-blend-bezier = Bézier
# Hover on Bézier in the list of blend curves.
app-gradient-bezier-exact-hover = Due punti di controllo trascinabili in entrambe le direzioni. Questa curva è già una cubica, quindi convertirla non cambia nulla.
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-visible-hover = Due punti di controllo trascinabili in entrambe le direzioni. La conversione APPROSSIMA la curva attuale — questa entro { $error }, uno scarto ben visibile.
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-barely-hover = Due punti di controllo trascinabili in entrambe le direzioni. La conversione APPROSSIMA la curva attuale — questa entro { $error }, uno scarto appena visibile.
# A label before the list of colour spaces (RGB, HSV).
app-gradient-space = Spazio
# A label before the Bézier presets beside it.
app-gradient-ease = Easing
app-gradient-ease-hover = I preset scrivono i due punti di controllo; per qualsiasi altra forma, trascinarli sulla curva. Una Bézier non ha punto medio — sono le sue maniglie a dire dove arriva a metà.
# Bézier presets, as in CSS easing: short buttons.
app-gradient-ease-linear = Lineare
app-gradient-ease-in = In
app-gradient-ease-out = Out
app-gradient-ease-in-out = In-out
app-gradient-midpoint = Punto medio
app-gradient-midpoint-hover = Dove la fusione arriva a metà, come frazione di questo segmento. 0.50 è al centro.
# A button that puts the midpoint back in the middle of the segment.
app-gradient-centre = Centra
app-gradient-centre-hover = Torna a 0.50
app-gradient-no-hue = ⚠ un estremo di questo segmento non ha tonalità
app-gradient-no-hue-hover = Un estremo è nero, bianco o grigio, quindi la sfumatura parte dal rosso e percorre l’intera ruota dei colori. È lecito e a volte voluto — ma è il motivo per cui il centro può avere un colore che nessuno dei due estremi contiene. La striscia sotto la curva mostra che cosa fa.
app-gradient-segment-start-hover = Colore all’INIZIO di questo segmento
app-gradient-segment-end-hover = Colore alla FINE di questo segmento
app-gradient-bytes-hover = Rosso, verde e blu come 0–255
app-gradient-floats-hover = Lo stesso colore come 0–1, cioè come lo memorizza il renderer
app-gradient-canvas-bezier-hover = La curva di fusione del segmento selezionato. Trascinare uno dei due punti di controllo — in x E in y — per rimodellarla. Le linee tenui sono 0 e 1; una curva può uscirne, e viene limitato il colore anziché la curva.
app-gradient-canvas-hover = La curva di fusione del segmento selezionato: quanto velocemente il colore passa dal punto di sinistra a quello di destra. Trascinare l’anello per spostare il punto medio, oppure passare la curva a Bézier per avere un punto di controllo che si muove in entrambe le direzioni.
# The selected stop's heading. $index: its number; $total: how many there are.
app-gradient-stop-heading = Punto { $index } di { $total }
app-gradient-stop-color-hover = Il colore in questo punto — clic sul campione per sceglierlo, oppure scrivere un valore esadecimale accanto
app-gradient-stop-bytes-hover = Rosso, verde e blu come 0–255 — i byte con cui viene scritto questo colore
# The eyedropper button.
app-gradient-pick = Preleva
app-gradient-pick-hover = Contagocce — prendere un colore da qualsiasi punto dello schermo. Spostare il puntatore sul colore voluto e fare clic; Esc annulla.
# Written before the stop's position (0.000–1.000) in its number field: a short abbreviation of "position".
app-gradient-stop-pos = pos
app-gradient-nudge-hover = Spostare di poco questo punto (tenere premuto Shift per un passo più fine)
app-gradient-remove = Rimuovi
app-gradient-remove-hover = Rimuovere questo punto (o fare clic sul ⊖ sotto di esso)
app-gradient-add-stop = Aggiungi punto
app-gradient-add-stop-hover = Dividere il segmento più ampio (o fare clic sulla linea sopra il gradiente)
# A menu of the built-in palettes, to start from a copy of one.
app-gradient-copy-preset = Copia preset…
app-gradient-paste = Incolla…
app-gradient-paste-hover = Importare una tavolozza da colori esadecimali o terne RGB 0–255
app-gradient-save-ggr = Salva .ggr…
app-gradient-save-ggr-hover = Scrivere questo gradiente in un file .ggr di GIMP — leggibile da GIMP, Krita e Inkscape, e da questo editor
# A menu of the palette file formats (.map, .ugr, .ggr, .ase).
app-gradient-import = Importa
app-gradient-import-map-hover = Caricare un file di tavolozza .map di Fractint / Kalles Fraktaler (righe R G B, una per voce)
app-gradient-import-ugr-hover = Caricare un file di gradienti .ugr di Ultra Fractal (un .ugr di solito contiene molti gradienti — se ne sceglie uno)
app-gradient-import-ggr-hover = Caricare un gradiente .ggr di GIMP, con punti medi, curve di fusione e spazi colore intatti
app-gradient-import-ase-hover = Caricare un elenco di campioni Adobe (.ase). I campioni non hanno posizioni, quindi arrivano distribuiti uniformemente e sfumati.
# The gradient's name, for the library of saved gradients.
app-gradient-name = Nome
# Shown greyed in the empty name field.
app-gradient-name-hint = il mio gradiente
# The save button, when a gradient of that name is already saved.
app-gradient-update = Aggiorna
app-gradient-save = Salva
app-gradient-save-hover = Salvare questo gradiente nella propria libreria, con curve e spazi colore intatti
app-gradient-save-needs-name-hover = Prima dare un nome al gradiente
# A menu of the saved gradients. $count: how many there are.
app-gradient-saved-menu = Salvati ({ $count })
app-gradient-nothing-saved = Ancora niente di salvato
app-gradient-delete-saved-hover = Eliminare questo gradiente salvato
# $count: the gradient's segments.
app-gradient-rich = Gradiente ricco: { $count } segmenti hanno punti medi, curve di fusione o spazi colore — più di quanto possa contenere un semplice elenco di punti.
app-gradient-convert = Converti in punti modificabili
app-gradient-convert-hover = Sostituirlo con un semplice elenco di punti. Questo SCARTA i punti medi, le curve di fusione e i giri di tonalità che un elenco di punti non può contenere.
# Above the list of gradients in an Ultra Fractal file. $count: how many; $file: the file's name.
app-gradient-ugr-list = { $count } gradienti in { $file }
app-gradient-close-list = Chiudi elenco
app-gradient-seamless = Senza giunzione (la fine coincide con l’inizio)
app-gradient-seamless-hover = Una tavolozza è ciclica, quindi la posizione 1 e la posizione 0 sono adiacenti sullo schermo. Con questa opzione attiva, il colore dell’ultimo punto resta uguale a quello del primo — la vista ad anello mostra la giunzione in alto.
app-gradient-ends-differ = ⚠ estremi diversi
app-gradient-ends-differ-hover = Il colore alla fine del gradiente non è il colore all’inizio, quindi una tavolozza ciclica mostra un bordo netto dove si riavvolge.
app-gradient-hard-bands = Fasce nette (nessuna fusione tra le voci)
app-gradient-hard-bands-hover = Semantica Fractint / .map: ogni voce è un colore piatto. Se disattivato, le voci si fondono in un gradiente continuo.
app-gradient-paste-intro = Incolla colori esadecimali (#ff8800) o terne 0–255 (255 136 0), separati da virgole, spazi o a capo:
# Applies the pasted colours.
app-gradient-apply = Applica
# Empties the paste box.
app-gradient-clear = Svuota
# $used: the colours kept; $total: the colours pasted; $limit: the most that are kept.
app-gradient-paste-imported-some = Importati { $used } colori su { $total }, campionati uniformemente lungo l’elenco ({ $limit } è il limite).
# $count: the colours pasted.
app-gradient-paste-imported = Colori importati: { $count }.
# $error: why the pasted text could not be read (in English).
app-gradient-paste-unreadable = Impossibile leggerlo: { $error }
# A hint at the bottom of the editor. $count: the palette's entries.
app-gradient-bands-hint = { $count } fasce · prese nell’ordine del file; le posizioni vengono ignorate mentre le fasce sono attive.
# A hint at the bottom of the editor, in the ring view. $count: the gradient's stops.
app-gradient-ring-hint = { $count } punti · clic sulla traccia esterna per aggiungerne uno lì · ⊕ aggiunge nello spazio più ampio, ⊖ rimuove il punto selezionato · trascinare un indicatore per spostarlo · il segno in alto è la cucitura
# A hint at the bottom of the editor, in the bar view. $count: the gradient's stops.
app-gradient-bar-hint = { $count } punti · clic sulla linea sopra il gradiente per aggiungerne uno lì · trascinare un indicatore per spostarlo · ⊖ o clic destro per rimuovere
app-gradient-cancel = Annulla
app-gradient-cancel-hover = Scartare tutte le modifiche fatte da quando questa finestra è stata aperta, e chiuderla
app-gradient-ok = OK
app-gradient-ok-hover = Mantenere queste modifiche e chiudere l’editor

## Shared

# The button that closes the Help window and the "Faster deep zoom" window.
app-close = Chiudi

## The "Faster deep zoom" window (Help menu): the optional accelerated build

app-accel-title = Zoom profondo più veloce
app-accel-active-heading = Stai usando la build accelerata
app-accel-active-body = Le orbite di riferimento dello zoom profondo vengono calcolate con MPFR/GMP, da 2,5 a 6,4 volte più velocemente che nella build standard in questa fase - la pausa prima che una vista profonda inizi a risolversi.
# $status: which arithmetic library is in use and its version (in English).
app-accel-in-use = Aritmetica in uso: { $status }
app-accel-fallback-heading = Build accelerata — in uso l’aritmetica integrata
app-accel-fallback-body = Questa è la build accelerata (MPFR), ma le sue librerie matematiche non sono state trovate, quindi usa l’aritmetica integrata (astro-float). Le immagini sono identiche; è solo più lenta la pausa prima che una vista profonda si risolva.
# Followed by the folder the program runs from, on its own line.
app-accel-fallback-dlls = Per usare il percorso più veloce, metti libmpfr-6.dll e libgmp-10.dll (e libgcc_s_seh-1.dll, libwinpthread-1.dll) accanto a fractadyne.exe, poi riavvia:
app-accel-warning-hidden = L’avviso all’avvio su questo argomento è nascosto.
app-accel-show-again = Mostralo di nuovo
app-accel-offer-heading = Una build facoltativa è da 2,5 a 6,4 volte più veloce in profondità
app-accel-offer-body = Lo zoom profondo passa gran parte del tempo a calcolare le orbite di riferimento sulla CPU - la pausa prima che una vista profonda inizi a risolversi. Una build facoltativa lo fa invece con le librerie MPFR/GMP, da 2,5 a 6,4 volte più velocemente, e tanto più quanto più si va in profondità. Tutto il resto è uguale.
app-accel-offer-identical = Le immagini sono IDENTICHE BYTE PER BYTE - è la stessa matematica calcolata da una libreria più veloce, verificata su ogni formula frattale e sull’intero corpus di confronto dello zoom profondo.
app-accel-offer-shared = Impostazioni, sessione salvata e posizioni sono condivise tra le due, quindi puoi passare liberamente dall’una all’altra senza importare nulla.
# Shown on Windows.
app-accel-separate-windows = È un download separato perché le librerie che usa non si possono compilare con il compilatore usato dalla build standard per Windows, e hanno una licenza diversa (GNU LGPL v3) da quella di Fractadyne.
# Shown on other systems.
app-accel-separate = È un download separato perché le librerie che usa hanno una licenza diversa (GNU LGPL v3) da quella di Fractadyne.
app-accel-download = Scarica per questa versione
app-accel-all-releases = Tutte le versioni
app-accel-all-releases-hover = Se il link diretto dà 404, questa versione non ha ancora una build accelerata - scegli qui la più vicina.
# Shown on Windows.
app-accel-install-windows = Estrai l’archivio ed esegui fractadyne.exe da quella cartella, lasciando i file .dll accanto.
# Shown on Linux.
app-accel-install-linux = Estrai l’archivio ed esegui ./fractadyne da quella cartella. Usa GMP e MPFR del sistema (libgmp10 / libmpfr6), che quasi ogni desktop ha già - vedi README-ACCELERATED.txt nel pacchetto.
# Shown on other systems.
app-accel-install-other = Per questa piattaforma non viene ancora compilato alcun pacchetto accelerato; si può compilare dai sorgenti con --features fractadyne-core/rug.

## The Help window's frame (its sections are under help-)

app-help-title = Guida di Fractadyne

## Checking for updates

app-toast-up-to-date = Hai già l’ultima versione.
# $error: why the check failed (in English).
app-toast-update-failed = Controllo degli aggiornamenti non riuscito: { $error }
# The window that offers a newer version.
app-update-title = Aggiornamento disponibile
# $version: the new version (0.3.0-beta.20). $channel: the token "stable" or "beta", in English; a translation can choose its words by it ({ $channel -> [beta] … *[other] … }).
app-update-version =
    { $channel ->
        [beta] È disponibile Fractadyne { $version } (beta)
       *[other] È disponibile Fractadyne { $version } (stabile)
    }
# $current: the version running now. $track: the update track the user follows (its name in the language in use).
app-update-running = Stai usando { $current }  ·  { $track }
app-update-download-intro = Scarica l’ultima versione da GitHub:
app-update-download = Scarica da GitHub
app-update-later = Ricordamelo più tardi

## The second graphics card (shown in its window, after "In use:" or "Could not start:")

# The second device is the same card as the window's. $name: the card's name.
app-worker-same-card = stessa scheda, { $name }
# Why the second device could not start.
app-worker-thread-stopped = il thread del dispositivo si è fermato

## Export status (shown in the Export image window)

app-export-gpu-unavailable = GPU non disponibile
app-export-reference-aborted = Esportazione non riuscita: costruzione del riferimento interrotta.

## The performance panel (View ▸ Performance panel)
## Its row labels sit in a monospace column about 14 characters wide: keep them short (a longer label pushes its value to the right).

app-perf-fps = FPS
app-perf-fps-hover = Fotogrammi presentati al secondo, dall’intervallo livellato tra i fotogrammi. Mentre una vista si assesta, mostra invece l’avanzamento della rifinitura - una vista in assestamento non gira alla frequenza che un numero qui farebbe pensare.
# What the FPS row shows while a view is settling. $done and $total: tiles; $eta: the time left, as m:ss.
app-perf-refining-tiles = rifinitura { $done }/{ $total } (~{ $eta })
app-perf-refining-arming = rifinitura (preparazione)
# $percent: how much is done, with one decimal.
app-perf-refining-percent = rifinitura { $percent }%
app-perf-building-reference = costruzione riferimento
app-perf-idle = inattivo
app-perf-frame = fotogramma
app-perf-frame-hover = Tempo reale livellato tra i fotogrammi - non la somma delle righe sottostanti. Include l’attesa della presentazione, quindi su una vista assestata e inattiva riflette la cadenza di ridisegno piuttosto che un lavoro.
app-perf-cpu = cpu
app-perf-cpu-hover = Tempo del thread principale in un aggiornamento: preparazione dei parametri e gestione della cache dei riferimenti. È la riga che ha dei picchi quando un riferimento viene ricostruito, e un picco qui è lo scatto che si percepisce.
app-perf-gpu-idle = gpu/attesa
app-perf-gpu-idle-hover = Tempo del fotogramma meno il tempo CPU del thread principale. È lavoro della GPU mentre la vista si renderizza, ma su una vista assestata è soprattutto attesa inattiva - il nome dice entrambe le cose perché il pannello non può distinguerle con questa sola sottrazione.
app-perf-timing = tempi
# How frame cost is measured, the timing row's value. TIMESTAMP_QUERY is a name.
app-perf-timing-starved = orologio di sistema (timestamp assenti)
app-perf-timing-gpu = timestamp GPU
app-perf-timing-no-query = orologio di sistema (nessun TIMESTAMP_QUERY)
app-perf-timing-hover = Come si misura il costo di un fotogramma. I timestamp GPU sono la fonte precisa; il ripiego sull’orologio di sistema entra in funzione quando le misure smettono di arrivare, e un budget calcolato con l’orologio di sistema ha ripetutamente dimensionato male i fotogrammi profondi — il rosso qui va letto come «i margini di sicurezza in questo momento sono stime».
app-perf-mode = modalità
app-perf-mode-hover = Percorso aritmetico con cui è stato renderizzato l’ultimo fotogramma: diretto (nessuna orbita di riferimento), perturbazione df32, o floatexp - il percorso a intervallo esteso usato oltre circa 1e28x, e l’unico su cui gira la BLA.
app-perf-eff-iter = iter eff.
app-perf-eff-iter-hover = Iterazioni effettivamente usate dall’ultimo fotogramma, dopo l’adattamento alla profondità e qualsiasi limite applicato durante il movimento - non l’impostazione di base in Qualità, che è solo il punto di partenza.
app-perf-precision = precisione
app-perf-precision-hover = Bit di mantissa con cui viene calcolata l’orbita di riferimento. Cresce con la profondità; 64 bit è il minimo per le viste poco profonde, non un limite che stai raggiungendo.
app-perf-orbit-len = lungh. orbita
# $cap: the most orbit samples this GPU holds, already formatted, or the text of app-perf-uncapped.
app-perf-orbit-len-hover = Campioni dell’orbita di riferimento. Questa GPU ne contiene al massimo { $cap } — avvicinarsi a questo valore è il muro pratico della profondità: oltre, la vista in tempo reale non può risolvere posizioni più profonde che richiedono un’orbita più lunga.
app-perf-uncapped = (senza limite)
app-perf-sa-skip = salto SA
app-perf-sa-skip-hover = Iterazioni saltate dall’approssimazione in serie prima che iniziasse il lavoro per pixel. Più alto è più veloce e non cambia nulla nell’immagine; la riga manca quando non ne ha saltata nessuna.
app-perf-aa = aa
app-perf-aa-hover = Supersampling: ogni pixel viene renderizzato da questo numero di campioni per asse, quindi 2x costa quattro volte il lavoro.
app-perf-dual = doppia
app-perf-dual-hover = Indica se è attiva la vista divisa Mandelbrot/Julia. Se è vero, ogni costo in questo pannello copre ENTRAMBI i pannelli della vista.
app-perf-zoom = zoom
app-perf-zoom-hover = Ingrandimento rispetto alla vista completa. La modalità aritmetica qui sopra cambia a circa 1e4x e di nuovo a circa 1e28x.
app-perf-iterate = iterazione
# $target and $lethal: times in milliseconds, already formatted.
app-perf-iterate-hover = Tempo GPU misurato dell’ultimo dispatch di iterazione. Il regolatore punta a { $target } ms; letture prolungate vicine a { $lethal } ms sono la fascia in cui questa classe di hardware ha perso il dispositivo.
app-perf-steps = passi/s
app-perf-steps-hover = Passi di iterazione al secondo sulla GPU, in miliardi. Le viste con molto interno girano più lente per passo di quanto suggerisca il solo conteggio, quindi è una misura di throughput più che un punteggio.
app-perf-budget = budget
# After the budget's value while it is still converging; the hover below quotes it.
app-perf-budget-settling = (in assestamento)
app-perf-budget-hover = Budget di passi adattivo per dispatch, la protezione del regolatore contro un fotogramma abbastanza lungo da indurre il driver a resettare la GPU. «in assestamento» significa che sta ancora convergendo da letture sparse e non è ancora un tetto affidabile.
app-perf-rss = rss
# $mb and $peak: memory in megabytes, already formatted.
app-perf-rss-value = { $mb } MB (picco { $peak })
app-perf-rss-hover = Memoria residente del processo, e il massimo raggiunto in questa sessione. Costruzioni di riferimenti profondi hanno superato i 2 GB sul campo senza che nient’altro sullo schermo lo segnalasse; ambra oltre tre quarti della RAM di sistema.
app-perf-gpu-est = gpu stim.
app-perf-gpu-est-hover = Memoria stimata residente sulla GPU: orbite di riferimento, alberi BLA, texture delle iterazioni, il ping-pong dello stato dei blocchi durante una rifinitura, e l’istantanea del present-gate. Ricavata dalle dimensioni note delle allocazioni; il valore reale del driver non è leggibile in modo portabile.
app-perf-julia-re = julia c.re
app-perf-julia-re-hover = Parte reale del parametro c che il pannello Julia sta renderizzando - il punto sotto il cursore, salvo se è fissato.
app-perf-julia-im = julia c.im
app-perf-julia-im-hover = Parte immaginaria del parametro c che il pannello Julia sta renderizzando.
app-perf-julia-c = julia c
# The julia c row's value when the parameter is held fixed.
app-perf-pinned = fissato
app-perf-pinned-hover = Il parametro è tenuto in un punto fisso, quindi muovere il cursore non cambia più il pannello Julia.
app-perf-c-panel = c/pannello
app-perf-c-panel-hover = Larghezza dello spazio di c coperta dall’intero pannello Mandelbrot. Quando è molto inferiore a un pixel di Julia, passando con il cursore c cambia ma il Julia sembra invariato — normale nello zoom profondo, non un blocco.
app-perf-ref-recompute = ricalcolo rif.
app-perf-ref-recompute-hover = Durata dell’ultimo ricalcolo dell’orbita di riferimento. È la pausa prima che una vista profonda inizi a risolversi, ed è la fase che la build accelerata facoltativa velocizza.
app-perf-recompute-rate = ricalcoli/s
app-perf-recompute-rate-hover = Orbite di riferimento INSTALLATE al secondo - quelle il cui risultato è stato effettivamente usato. Confrontala con la riga sotto: differiscono per il lavoro buttato via.
app-perf-builds-rate = costr. rif./s
app-perf-builds-rate-hover = Costruzioni di riferimenti AVVIATE al secondo, comprese quelle il cui risultato viene scartato - uno slot di anticipo che il tuffo non raggiunge mai, o un’estensione rifiutata. Costano CPU piena e non compaiono da nessun’altra parte: una volta un ciclo di prefetch ha eseguito circa 400 costruzioni al secondo per sei minuti mentre «ricalcoli/s» indicava 2. Valori a una cifra sono normali; un grande divario tra queste due righe significa che si sta buttando via lavoro.
app-perf-recompute-total = ricalcoli tot
app-perf-recompute-total-hover = Ricalcoli dei riferimenti dall’avvio. Utile come frequenza su un intervallo noto; un numero che sale mentre nulla si muove è il segnale da indagare.

## Help: the contents list (Help & reference…, F1)

# Each section's name in the contents list (about 165 px wide: keep them short), and its heading.
help-section-overview = Panoramica
help-section-navigation = Navigazione
help-section-expressions = Espressioni delle coordinate
help-section-options = Colorazione e opzioni
help-section-fractals = Frattali
help-section-how-it-works = Come funziona
help-section-command-line = Riga di comando
help-section-shortcuts = Scorciatoie
help-section-hardware = Hardware consigliato
# Credits to the prior work Fractadyne builds on.
help-section-acknowledgments = Riconoscimenti
# The third-party license notices.
help-section-licenses = Licenze
help-section-about = Informazioni

## Help ▸ Overview

help-overview-intro = Fractadyne è un esploratore di frattali nativo pensato per lo zoom ultra-profondo e per la velocità. Disegna frattali «a tempo di fuga» — immagini create ripetendo una semplice formula in ogni pixel.
help-overview-what-is = Che cos’è un frattale a tempo di fuga?
help-overview-what-is-text = Per ogni pixel il programma esegue più e più volte una formula come z → z² + c, partendo da zero. Se il valore resta piccolo per sempre, il pixel appartiene all’insieme ed è disegnato scuro. Se prima o poi cresce senza limiti («fugge»), il pixel è fuori dall’insieme, e il suo colore registra quanti passi ci sono voluti. Il confine infinitamente dettagliato tra «resta» e «fugge» è il frattale.
help-overview-can-do = Che cosa puoi fare
help-overview-can-pan = Spostare la vista e zoomare praticamente senza limiti (la posizione è esatta a qualsiasi profondità).
help-overview-can-families = Passare tra 25 famiglie di frattali integrate (dieci, e cinque di esse a potenze superiori), le tue formule, Life e gli L-system, e vedere qualsiasi famiglia a tempo di fuga tranne Newton come insieme di Julia.
help-overview-can-recolor = Ricolorare con gradienti predefiniti o personalizzati e diversi metodi di colorazione.
help-overview-can-lighting = Aggiungere un’illuminazione a rilievo 3D e contorni luminosi sul bordo.
help-overview-can-minibrots = Agganciarsi ai minibrot, salvare punti nei segnalibri ed esportare immagini ad alta risoluzione.
help-overview-can-autopilot = Lasciare che il pilota automatico dell’auto-zoom si tuffi da solo verso i dettagli.
help-overview-can-share = Condividere qualsiasi punto come un piccolo frammento di testo o un file «.fdn».
help-overview-can-tours = Eseguire tour da script e un benchmark dell’hardware.
help-overview-first-steps = Primi passi
# "Seahorse Valley" is a place in the Navigate menu: use its name as that menu shows it. A and F1 are keys.
help-overview-first-steps-text = Apri il menu Naviga e scegli «Seahorse Valley», poi gira la rotellina del mouse per ingrandire. Trascina per spostare la vista. Oppure premi semplicemente A e lascia che il pilota automatico si tuffi per te. Premi F1 in qualsiasi momento per tornare a questa guida.

## Help ▸ Coordinate expressions
## The expressions, numbers and function names here are typed as shown in every language:
## keep them exactly as written. Menu items in quotation marks are the menus' own labels.

help-expr-intro = Ovunque Fractadyne accetti una coordinata, si può scrivere un’espressione invece di un decimale: i campi Re / Im di Naviga → «Vai alla posizione…» (e ogni campo della sua modalità polare), le righe center_re / center_im di una posizione .fdn, i re / im di uno script di tour, e --center sulla riga di comando. L’espressione viene valutata con tutte le cifre richieste dallo zoom, e la finestra Vai alla posizione la conserva con la vista — così un punto inserito come 1/3, o come -0.5 + 0.25*cos(pi/4), resta esattamente su quel punto per quanto in profondità si zoomi, mentre un decimale andrebbe alla deriva una volta esaurite le sue cifre.
help-expr-examples = Esempi
help-expr-ex-third = Un terzo esatto. Zoomando compaiono altre cifre.
help-expr-ex-neck = Il collo di Seahorse Valley — la radice del bulbo di periodo 2.
help-expr-ex-circle = Un punto sul cerchio di raggio 1/4 attorno a -0.5 (la modalità polare lo compone per te).
help-expr-ex-bulb = Re della radice del bulbo di periodo 3; la sua Im è sin(2*pi/3)/2 - sin(4*pi/3)/4. Naviga → «Punti esatti (espressioni)» li ha già integrati.
help-expr-ex-complex = Un intero valore complesso scritto nel campo Re riempie sia Re sia Im.
help-expr-ex-combine = Funzioni e costanti si combinano liberamente.
help-expr-ex-powers = Potenze: 2^-20 è 1/1048576. -2^2 è -4 — il meno si applica dopo la potenza.
help-expr-ex-root = -2, la radice cubica reale. Le radici pari di un numero negativo vengono rifiutate; sqrt(-1) dà i.
help-expr-ex-scientific = Anche la notazione scientifica funziona dentro le espressioni.
help-expr-numbers = Numeri e operatori
# The decimal point in an expression is '.' in every language: keep that rule as it is.
help-expr-numbers-decimals = Decimali con esponente facoltativo: 0.25, -1.5e-30, .5. Solo cifre — niente spazi o virgole di raggruppamento dentro un numero, e il separatore decimale è sempre il punto «.».
help-expr-numbers-imaginary = Aggiungere il suffisso i per un numero immaginario (0.25i), oppure usare i da sola: 3 + 4*i.
# Starts with the operators themselves, two spaces apart.
help-expr-numbers-operators = +  -  *  /  e ^ (potenza; valutata da destra a sinistra, quindi 2^3^2 = 2^9). Le parentesi raggruppano. Non c’è moltiplicazione implicita: scrivere 2*pi, non 2pi.
help-expr-numbers-case = I nomi non distinguono tra maiuscole e minuscole: PI, Pi e pi sono la stessa costante.
help-expr-constants = Costanti
help-expr-functions = Funzioni
help-expr-functions-angles = Gli angoli sono in radianti. Per i gradi, moltiplicare per pi/180 — cos(45*pi/180) — oppure passare la finestra Vai alla posizione alla modalità polare e scegliere l’unità gradi.
help-expr-refused = Che cosa viene rifiutato, e perché
help-expr-refused-fractional = Un numero negativo elevato a una potenza frazionaria, come (-2)^(1/2): il risultato è complesso e ci sono diverse radici tra cui scegliere. Usare root(x, n) o cbrt(x) per una radice reale, o sqrt(x) per quella complessa principale.
help-expr-refused-complex = Un argomento complesso per una funzione a valori reali (sin(i), ln(1+i)). Solo sqrt e abs accettano un argomento complesso; gli operatori aritmetici e le potenze intere funzionano sui valori complessi.
help-expr-refused-domain = Errori di dominio: ln o log di un numero non positivo, asin o acos fuori da -1…1, 0 elevato a una potenza negativa, divisione per zero.
help-expr-refused-bounds = Argomenti di sin, cos, tan, exp o ^ oltre ±2^32, e annidamenti oltre i 32 livelli: una coordinata incollata è un input non fidato, e questi richiederebbero un tempo illimitato.
help-expr-refused-grammar = Niente variabili, funzioni definite dall’utente, confronti o condizioni. La grammatica è volutamente ridotta, così una posizione salvata significa la stessa cosa in ogni versione.
help-expr-no-parse = Se non viene interpretata
help-expr-no-parse-text = Il messaggio indica il campo, dice che cosa è stato trovato e a quale carattere, e — per gli errori più comuni — che cosa scrivere invece: un segno meno tipografico (−) incollato da un documento, × o ÷, una virgola decimale, un * mancante tra un numero e un nome, una parentesi non chiusa, o una funzione scritta senza parentesi. Nulla viene mai letto a metà: se una parte qualsiasi del testo non si capisce, l’intera coordinata viene rifiutata invece di essere accorciata in silenzio.

## Help ▸ Navigation
## Menu paths (Navigate → "Go to location…") and quoted items name the interface's own
## menus and controls: use their translations. Key names (Ctrl, Shift, Space, Esc) stay.

# A heading: the mouse controls (also in Shortcuts).
help-mouse = Mouse
# The left column of a controls table: a gesture or key. Keep key names (Shift, Space, Ctrl) as they are.
help-key-left-drag = Trascinamento sinistro
help-nav-left-drag = Spostare la vista.
help-key-mouse-wheel = Rotellina del mouse
help-nav-wheel = Zoom avanti/indietro verso il cursore; girandola più velocemente si zooma di più per scatto. Con «Aggancia zoom alle strutture» attivo, zoomando vicino a un minibrot lo zoom viene attirato su di esso.
help-key-shift-drag = Shift+trascina
help-nav-shift-drag = Zoom a rettangolo — si trascina un rettangolo e lo zoom lo porta a riempire la vista.
# $icon: the click-to-zoom toolbar button's symbol.
help-key-click-zoom = Clic per zoom ({ $icon })
help-nav-click-zoom = Attivare il pulsante a mirino nella barra degli strumenti (vista singola): un clic sinistro si tuffa allora nel punto di un fattore stabilito, il clic destro torna indietro. Il trascinamento sposta ancora la vista; il fattore (2–100×) si sceglie sotto Navigazione nel pannello di destra. Backspace annulla un clic.
help-nav-continuous = Zoom continuo e vista iniziale
help-key-hold-space = Tieni premuto Space
help-nav-hold-space = Zoom avanti fluido, ancorato al cursore.
help-key-hold-shift-space = Tieni premuto Shift+Space
help-nav-hold-shift-space = Zoom indietro fluido.
help-nav-continuous-text = Il pulsante della vista iniziale anima un dolce volo all’indietro fino alla vista completa. «Velocità zoom» nel pannello di destra imposta la velocità dello zoom continuo.
help-nav-autopilot = Pilota automatico
help-nav-autopilot-text = Premi A (o Strumenti → «Auto-zoom (pilota automatico)») per un tuffo a mani libere: sceglie un punto sul bordo del frattale dove i dettagli sono più densi, preferendo il centro della vista, e vi scivola dentro portandolo gradualmente al centro dello schermo. Mantiene quel bersaglio finché resta interessante e vira dolcemente verso strutture più ricche quando ne compaiono. Qualsiasi comando di navigazione — o Esc — lo ferma. «Priorità auto-zoom» nel pannello di destra (o in Strumenti) sceglie che cosa mettere al primo posto: Velocità zooma alla velocità impostata e aggiorna lo schermo appena è pronto un fotogramma con dettagli; Qualità mostra solo fotogrammi a piena risoluzione completamente risolti e rallenta lo zoom quanto serve perché l’immagine non venga mai ingrandita troppo prima che arrivi la successiva. In entrambi i casi lo schermo non mostra mai un fotogramma vuoto o di un solo colore: un aggiornamento che non ha ancora risolto alcun dettaglio resta nascosto dietro l’ultimo. «Bersaglio auto-zoom» sceglie a che cosa mira: Dettaglio (il bordo più ricco nella vista, scelto di nuovo strada facendo) o Punto di Misiurewicz — trova il punto di Misiurewicz più vicino, lo risolve fino alla profondità del limite di tuffo restando fermo, poi vi si tuffa esattamente; la struttura attorno a un tale punto si ripete ogni poche ottave all’infinito, quindi quel tuffo non finisce mai in un vicolo cieco (solo insieme di Mandelbrot).
help-nav-history = Cronologia e spostamenti precisi
help-nav-undo = Annulla la vista precedente.
help-nav-redo = Ripeti.
help-nav-goto = Naviga → «Vai alla posizione…» permette di leggere, scrivere, incollare o copiare il centro e lo zoom esatti (precisione piena, qualsiasi profondità). I campi del centro accettano più dei semplici numeri: frazioni come «1/3» ed espressioni costruite con pi, tau, e, sqrt e sin/cos/tan — così «-0.5 + 0.25*cos(pi/4)» è una coordinata valida. Un’espressione viene conservata con la vista e rivalutata con la precisione richiesta da ogni profondità, quindi un punto inserito così resta esatto per quanto si zoomi (un decimale arrotondato prima o poi non lo sarebbe). Una modalità polare (offset + r ∠ θ, in gradi / radianti / giri) compone per te una di queste espressioni. L’elenco completo di funzioni e costanti, con esempi, è la sezione «Espressioni delle coordinate» di questa guida (il pulsante ? nella finestra la apre). Naviga → Segnalibri salva e richiama le posizioni.
help-nav-exact-points = Naviga → «Punti esatti (espressioni)» salta a punti definiti esattamente invece che da un decimale arrotondato — le radici dei bulbi della cardioide principale, dove si attacca ogni gemma di periodo q, scritte come espressioni cos/sin. Poiché l’espressione viaggia con la vista, puoi zoomare in uno di essi senza che si allontani mai dal punto.
help-nav-share = Naviga → «Condividi posizione…» cattura l’intera vista — frattale, centro a precisione piena, zoom e colorazione — come un compatto frammento «.fdn» che puoi copiare (Copia), reincollare e applicare (Applica), o salvare/caricare come file, per riprodurre un punto esattamente (o inviarlo a qualcuno).
help-nav-open = File → «Apri vista o posizione…» apre direttamente uno qualsiasi di questi: un PNG/EXR esportato (la vista incorporata viene ripristinata), un file di posizione «.fdn», o un «.kfr» di Kalles Fraktaler.
help-nav-finding = Trovare i dettagli
help-nav-minibrots = I minibrot sono minuscole copie dell’intero insieme nascoste lungo il bordo. Centrane uno approssimativamente e premi M (o Strumenti → «Trova minibrot + zoom») per agganciarti con Newton esattamente al suo centro, leggerne il periodo e zoomare direttamente alla sua scala — spesso molti ordini di grandezza in un solo passo. Se sei già più in profondità del minibrot, si sposta solo il centro.
help-nav-minimap = Visualizza → «Minimappa» mostra una piccola mappa dell’intero insieme con un indicatore «sei qui» e la profondità di zoom corrente; clicca sulla mappa per saltare a una regione.

## Help ▸ Coloring & options
## Names in quotation marks are the controls' own labels in the right-hand panel.

help-options-intro = Tutte queste opzioni si trovano nel pannello di destra (e restano da una sessione all’altra).
help-options-palette = Tavolozza
help-options-palette-text = Tavolozza sceglie un gradiente predefinito, quello Personalizzato o una modalità a due colori. «Modifica gradiente…» apre un editor in cui ogni punto di colore ha un colore e una posizione (0–1); si possono aggiungere punti, copiare un preset da cui partire, o importare un file di tavolozza.
help-options-paste = «Incolla…» nell’editor dei gradienti importa una tavolozza trovata altrove. Accetta colori esadecimali con o senza # iniziale (#ff8800, ff8800, o la forma breve #f80) e semplici terne RGB 0–255 come «255 136 0» — il formato dei file .map di Fractint e di Kalles Fraktaler — separati da virgole, spazi o a capo. Si conservano fino a 256 colori; un elenco più lungo viene campionato uniformemente fino a 256, mantenendo il primo e l’ultimo così la forma del gradiente si conserva.
help-options-import-map = «Importa ▸ .map…» lì accanto carica un file di tavolozza .map di Fractint o di Kalles Fraktaler. Sono tabelle di consultazione indicizzate dal numero di iterazioni, senza fusione tra le voci, e gli scalini netti di colore che ne risultano sono ciò di cui è fatto il classico aspetto di Fractint — per questo si importano così, con una casella «Fasce nette» per fondere invece gli stessi colori in un gradiente continuo. I file di Fractint memorizzano valori di colore a 6 bit moltiplicati per quattro, quindi il loro bianco è 252 invece di 255; Fractadyne lo segnala quando ne importa uno e lascia i valori esattamente come sono scritti, perché anche le immagini salvate da Fractint riportano lo stesso 252.
help-options-duotone = Duotono mappa il valore su una rampa continua a due colori (Ombra → Luce). Binario (insieme) è una vista piatta a due colori — un colore pieno per i punti dentro l’insieme e un altro per quelli fuori, senza gradiente — il modo più chiaro per vedere la forma dell’insieme.
help-options-cycle = Ciclo imposta quante volte il gradiente si ripete lungo l’intervallo delle iterazioni (fasce più strette o più larghe). Offset ruota l’intero gradiente.
help-options-normalize = «Normalizza colori profondi» adatta la tavolozza all’intervallo dei valori di fuga effettivamente presenti nella vista. Le viste profonde coprono centinaia di migliaia di iterazioni, e senza questa opzione un ciclo fisso fa girare il gradiente migliaia di volte tra pixel vicini, trasformando dettagli corretti in puntinatura.
help-options-log-scale = «Scala colori logaritmica» distribuisce la tavolozza secondo il logaritmo del valore di fuga anziché linearmente. Andando più in profondità i conteggi di fuga si affollano verso l’estremo alto, quindi un adattamento lineare spende la maggior parte del gradiente su un sottile guscio aderente al bordo e lascia il resto quasi piatto; la scala logaritmica lo apre, e mantiene i colori più stabili in un video di zoom man mano che l’intervallo cresce.
help-options-animate = Anima fa ciclare i colori nel tempo — Avanti, Indietro o Ping-pong spostano l’offset; Gradienti casuali sintetizza di continuo gradienti armoniosi che si trasformano dolcemente. Velocità regola i cicli (o le trasformazioni) al secondo; «Mescola gradiente» ne estrae uno nuovo in modalità casuale.
help-options-method = Metodo di colorazione — come i dati diventano colore
# The coloring methods, by the names the Coloring panel gives them, each with what it does.
help-method-smooth = Iterazione continua
help-method-smooth-text = Classiche fasce continue in base al tempo di fuga.
help-method-stripe = Media delle strisce
help-method-stripe-text = Fasce fluide dall’angolo dell’orbita (cursore Densità strisce).
help-method-triangle = Disuguaglianza triangolare
help-method-triangle-text = Trama fine data dal punto in cui ogni passo cade tra i limiti.
help-method-trap = Trappola orbitale
help-method-trap-text = Distanza dell’orbita da una forma (punto/croce/cerchio); colora anche l’interno.
help-method-distance = Stima della distanza
help-method-distance-text = Ombreggia in base alla vicinanza al bordo.
help-method-decomposition = Decomposizione
help-method-decomposition-text = Celle dall’angolo di fuga finale.
help-options-relief = Illuminazione a rilievo 3D
help-options-relief-text = Ombreggia la superficie in base alla pendenza del bordo (la derivata) per un aspetto in rilievo e illuminato. Angolo luce imposta la direzione; Rilievo imposta l’intensità (più basso = più netto); «Ruota luce» la anima. Solo famiglie olomorfe (Mandelbrot / Multibrot).
help-options-glow = Bagliore distanza
help-options-glow-text = Fasce di contorno luminose che si infittiscono in filamenti incandescenti vicino al bordo. Bagliore è la quantità di fusione, Larghezza banda la spaziatura, e «Anima bagliore» le fa scorrere.
help-options-quality = Qualità e iterazioni
help-options-quality-text = Iterazioni è il numero massimo di passi prima che un pixel venga considerato dentro l’insieme; «Iterazioni automatiche con lo zoom» lo aumenta automaticamente mentre si zooma (i dettagli più profondi ne richiedono di più). Anti-aliasing applica il supersampling alle immagini ferme (2×–8×) una volta che la vista si è assestata, domando la fine «polvere» esterna.
help-options-other = Altro
help-options-other-text = Velocità zoom imposta la velocità dello zoom continuo; il limite di FPS limita la frequenza dei fotogrammi; «Entrambi, collegati» mostra un insieme di Mandelbrot e il suo insieme di Julia affiancati (il cursore imposta il parametro di Julia in tempo reale).

## Help ▸ Fractals
## Fractal family names (Mandelbrot, Multibrot, Tricorn, Burning Ship, Celtic, Buffalo,
## Phoenix, Newton) are names: keep them as written, as the formulas. Formula syntax
## (z = z^3 - p1*z + c, sqr, |z|, if / elseif / endif, p1-p5, maxit) is typed as shown.

help-fractals-intro = Ogni famiglia itera una formula con z che parte da 0 e c dato dal pixel (tempo di fuga), salvo indicazione contraria. z = x + iy è un numero complesso.
help-fractals-mandelbrot = z → z² + c. L’originale. L’insieme è formato da ogni c la cui orbita resta limitata; è connesso, e il suo bordo è così increspato da avere dimensione di Hausdorff 2.
help-fractals-multibrot = z → z^d + c per potenze d = 3, 4, 5. Le potenze superiori aggiungono lobi: l’insieme ha simmetria rotazionale di ordine (d−1) (Multibrot 3 di ordine 2, 4 di ordine 3, 5 di ordine 4).
help-fractals-tricorn = z → conj(z)² + c, dove conj(x + iy) = x − iy. La coniugazione lo rende anti-olomorfo, con simmetria di ordine 3 e caratteristici «artigli» curvi.
help-fractals-burning-ship = z → (|x| + i|y|)² + c — si prendono i valori assoluti prima di elevare al quadrato (reale = x²−y²+cx, immaginaria = 2|xy|+cy). Non analitico; gli zoom profondi rivelano strutture a forma di nave (tradizionalmente viste capovolte).
help-fractals-celtic = Come Mandelbrot ma con il valore assoluto della parte reale: reale = |x²−y²| + cx, immaginaria = 2xy + cy. Produce motivi a forma di cuore e di scudo.
help-fractals-buffalo = Valore assoluto di entrambe le parti di z²: reale = |x²−y²| + cx, immaginaria = |2xy| + cy — un incrocio tra Celtic e Burning Ship.
# A heading: the families at powers above 2 (the Fractal menu's "Higher powers").
help-fractals-powers = Potenze superiori
# ("Burning Ship  3 4 5") is how the Fractal menu shows such a row: keep it as it is.
help-fractals-powers-text = Multibrot 6, 7 e 8 (z^d + c), e Burning Ship, Tricorn, Celtic e Buffalo alle potenze 3, 4 e 5: le stesse pieghe a una potenza superiore. Burning Ship d prende i valori assoluti delle parti di z e poi la potenza; Tricorn d coniuga prima; Celtic d e Buffalo d ripiegano la parte reale (e, per Buffalo, la parte immaginaria) di z^d. Il menu Frattale e l’elenco a discesa le mostrano una famiglia per riga («Burning Ship  3 4 5»). Ognuna fa lo zoom profondo come la sorella di potenza 2, e Multibrot 6, 7 e 8 supportano anche l’approssimazione in serie e la ricerca dei minibrot (M), come Multibrot 3–5. Multibrot 8 fugge con raggio 128 invece di 256, dove il passo successivo uscirebbe dall’intervallo della precisione singola.
help-fractals-phoenix = z → z² + c + p·z(n-1), dove z(n-1) è l’iterata precedente e p è una costante (p = -0.5). Il termine di memoria produce filamenti simili a fiamme.
help-fractals-newton = Metodo di Newton per le radici di z³ − 1 = 0: z → z − (z³−1)/(3z²). Invece che in base al tempo di fuga, i pixel sono colorati secondo la radice cubica dell’unità, tra le tre, a cui converge l’iterazione (i bacini di attrazione) e secondo la rapidità; i confini intricati dei bacini sono il frattale.
help-fractals-julia = Modalità Julia
help-fractals-julia-text = Per ogni famiglia tranne Newton si può passare a un insieme di Julia: invece di far partire z da 0 con c = pixel, si fissa c (un parametro) e si fa partire z dal pixel. L’insieme di Julia è il confine tra i punti di partenza che restano limitati e quelli che fuggono, per quel c fisso. In «Entrambi, collegati», muovendo il cursore sul pannello Mandelbrot si imposta c in tempo reale.
help-fractals-deep = Supporto dello zoom profondo
help-fractals-deep-text = Mandelbrot, Multibrot 3/4/5 e Tricorn supportano lo zoom profondo per perturbazione a profondità estrema (floatexp). Burning Ship, Celtic e Buffalo non sono analitici (prendono valori assoluti), quindi usano una perturbazione attenta ai segni; ora anche questa gira nell’intervallo floatexp, con zoom profondi ben oltre il vecchio limite df32 di ~1e28× (rari puntini vicino alle pieghe dei valori assoluti attendono la correzione multi-riferimento dei glitch). Phoenix usa una perturbazione a due termini e fa anch’esso lo zoom profondo nell’intervallo floatexp (senza SA/BLA, quindi è più pesante di Mandelbrot). Le potenze superiori fanno lo zoom profondo allo stesso modo, con la perturbazione di ogni potenza che segue la propria catena di elevamenti al quadrato; anche Multibrot 3–8 supportano l’approssimazione in serie e la BLA, come Mandelbrot, e Tricorn, Burning Ship, Celtic e Buffalo a ogni potenza supportano la BLA (senza serie), che porta avanti l’interno di una vista profonda invece di iterarlo passo per passo. Solo Newton usa il percorso diretto, nitido fino a ~1e6×; le formule personalizzate fanno lo zoom profondo salvo quanto indicato nella sezione Formule personalizzate qui sotto.
help-fractals-custom = Formule personalizzate
help-fractals-custom-text = Frattale ▸ Formula personalizzata… accetta il passo di una formula in espressioni in stile Fractint: z = z^3 - p1*z + c, oppure più istruzioni (t = sqr(z), poi z = t*t + c). z è l’iterata, c il pixel (la costante di Julia in modalità Julia), p1-p5 i parametri; |z| è il modulo al quadrato, come in Fractint, e sono disponibili le funzioni consuete (sqr, abs, conj, real, imag, cabs, flip, recip, ident, exp, log, sqrt, sin, cos, tan, cotan, sinh, cosh, tanh, cotanh, asin, acos, atan, asinh, acosh, atanh, cosxx, e floor, ceil, trunc e round, che arrotondano ciascuna parte). log è il logaritmo naturale, come in Fractint e nell’analisi complessa: ciò che molti libri di testo scrivono ln (non esiste un logaritmo in base 10). Una formula scritta con ln, cot, coth, Re o Im non viene letta; il controllo della sintassi indica la funzione da usare, un pulsante la riscrive, e digitando ln viene proposto log. Ogni passo fa lo zoom profondo per perturbazione, con la stessa aritmetica a intervallo esteso oltre 1e28x dei frattali integrati, tranne uno con una potenza il cui esponente varia con z o c (come z^c): questo si renderizza sul percorso diretto, nitido finché un passo di c in precisione singola non copre un pixel, circa da 1e4x a 1e5x, e meno in profondità lontano dall’origine. La finestra della formula dice quale caso si applica. log, sqrt e le potenze non intere prendono i valori principali, che saltano attraverso il semiasse reale negativo. La perturbazione segue il salto esattamente, ma un pixel la cui orbita attraversa il taglio dove quella del riferimento non lo fa viene portato avanti da quel momento a circa la precisione singola, quindi le viste profonde le cui orbite stanno a cavallo di un taglio lì mostrano rumore (misurato: pulito a 1e5x sull’asse reale per sqrt(z^4 + c), rumoroso per le formule con log già a 1e4x). Ogni passo viene iterato per intero (senza approssimazione in serie), quindi una vista profonda con un alto numero di iterazioni si renderizza più lentamente di quella di un frattale integrato. Vicino al bordo una formula con funzioni ha orbite caotiche, e un’orbita lunga lì viene seguita solo approssimativamente in precisione singola: a 2.000 iterazioni un terzo dei pixel di sin(z) + c differisce da quelli dell’orbita esatta, a 60 quasi nessuno. Viste, sessioni e immagini esportate conservano il testo della formula.
help-fractals-custom-sections = Funzionano anche le sezioni di Fractint. Le istruzioni prima di un ':' sono la sezione di inizializzazione, eseguita una volta per pixel; un confronto come ultima istruzione è il test di bailout della formula stessa, che itera finché è vero (|z| <= 100 && |z - 1| > 0.000001 ferma un’orbita che fugge o si stabilizza su 1); if (...), elseif (...), else ed endif scelgono le istruzioni; e una variabile letta prima di essere impostata in un passo ha il valore del passo precedente (dalla sezione di inizializzazione, o 0). I confronti confrontano le parti reali e danno 1 o 0, && e || li combinano, e anche un’assegnazione è un valore (a = b = pixel). pixel, p1-p5, pi ed e si possono assegnare, e mantengono il proprio valore finché non vengono assegnati; maxit è il numero di iterazioni e ismand vale 1 (la forma Mandelbrot della formula). Una formula con uno qualsiasi di questi elementi, o con arrotondamenti, si renderizza sul percorso diretto, senza zoom profondo. Libreria di formule > Importa legge anche i file .frm di Fractint. Ogni voce leggibile diventa una formula, e la sua riga dice che cosa ha cambiato la lettura: fn1-fn4 prendono i valori predefiniti di Fractint (sin, sqr, sinh, cosh); la variabile c di Fractint viene rinominata c_, perché qui c è il pixel; un nome che nulla imposta vale 0, come in Fractint; gli spazi dentro un nome vengono eliminati (end if equivale a endif), come fa Fractint. Il messaggio dell’importazione conta le voci non leggibili e indica il motivo più comune: le variabili dello schermo e della vista (whitesq, scrnpix, scrnmax, center, magxmag, rotskew), lastsqr e i numeri casuali non sono supportati. I classici di Fractint della Raccolta (Magnet, Nova, Spider, Manowar e altri) sono scritti con queste sezioni.
help-fractals-custom-editor = Nel campo della formula, le parentesi prendono un colore per livello di annidamento; la coppia vicina al cursore viene evidenziata, e una senza corrispondenza appare in rosso (quelle dei commenti non contano, perché il parser li ignora). I nomi si completano mentre scrivi: dopo due lettere un elenco propone le funzioni, i nomi e le tue variabili che iniziano così. Up e Down scelgono, Tab o Enter ne prende uno (una funzione arriva con la sua parentesi aperta), Esc chiude l’elenco, e Enter dopo un nome scritto per intero inizia comunque una nuova riga.
help-fractals-custom-textbook = Libro di testo (l’interruttore sopra la formula) la mostra composta come in un libro di testo, e la modifica sul posto. Si scrive come nel testo: / fa una frazione del termine che lo precede, ^ un esponente (+, - o = digitati alla fine di un esponente ne escono), ( le parentesi, o una chiamata dopo il nome di una funzione, | le barre, e una virgola dentro le parentesi una costante complessa. Le frecce si muovono nella formula, Up e Down tra un numeratore e il suo denominatore, Tab alla casella vuota successiva; Backspace all’inizio di una frazione o di una coppia di parentesi la seleziona, e una seconda pressione la rimuove conservandone il contenuto. Copia, incolla e annulla funzionano come in un campo di testo, i nomi si completano come lì, e il tastierino scrive nella formula composta (□² eleva al quadrato ciò che precede il cursore, un tasto funzione racchiude la selezione). Ogni modifica viene riscritta nel testo, quindi le due viste contengono sempre la stessa formula; una riga che non hai modificato conserva spaziatura e commento. La notazione non cambia mai ciò che viene calcolato: |z| è il modulo al quadrato e appare come |z|², cabs(z) appare come |z|, ed exp(z) resta exp(z). Una riga che non viene letta appare in rosso; cliccala per modificarla come testo. LaTeX copia la formula come sorgente LaTeX, così come è composta, e mentre Libro di testo è attivo anche la libreria di formule mostra le sue formule composte.
help-fractals-custom-numbers = I numeri in una formula (0.1, 1/3, pi, i parametri) vengono letti come valori a doppia precisione, circa 16 cifre, e l’orbita di riferimento profonda itera esattamente con quei valori. Due formule uguali sulla carta possono quindi differire in profondità: 0.1*z moltiplica per il double più vicino a 0.1, mentre z/10 divide per un 10 esatto, e le due immagini si separano oltre circa 1e15x (misurato: il 5% dei pixel a 1e15x, un terzo a 1e17x, tutti a 1e20x). Ciascuna immagine è corretta per la propria formula; scrivi quella che intendi condividere.
help-fractals-custom-library = «Salva nella libreria» della finestra conserva una formula e i suoi parametri sotto un nome (salvare con un nome già presente nella libreria aggiorna quella voce). Frattale ▸ Libreria di formule… le elenca per applicarle, modificarle, esportarle o eliminarle; Importa aggiunge le formule di un file di formule senza sostituirne nessuna: quella che la libreria ha già viene saltata, e una nuova il cui nome è già preso diventa «nome (2)». La libreria è formulas.toml nella cartella indicata sotto File e dati (Informazioni), ed è essa stessa un file di formule. Un fotogramma chiave di un tour porta una formula come formula = "..." con formula_params = [[re, im], ...], e Strumenti > Tour dalla vista corrente li scrive per una vista personalizzata.
help-fractals-custom-collection = Con Fractadyne viene fornita una raccolta di 45 formule, scritte apposta: potenze superiori e le varianti ripiegate delle famiglie Burning Ship e Celtic, miscele e ibridi a due passi, l’insieme di Mandelbrot visto attraverso un’altra mappa (invertito, log-polare, attraverso il seno), formule trascendenti e razionali, e insiemi di Julia. Sono gli Esempi della finestra e la scheda Raccolta della libreria di formule, dove Copia nelle mie ne aggiunge una alle tue formule. Ognuna ha una vista di partenza, e Applica la raggiunge: una formula aperta sulla vista iniziale spesso non è dove si trova la sua immagine. Allo stesso modo, una formula che salvi mentre è visualizzata conserva la vista in cui ti trovi, e Applica vi ritorna.

## Help ▸ How it works

help-how-escape = Tempo di fuga e colore continuo
help-how-escape-text = Ogni pixel itera la formula finché il suo modulo supera un raggio di bailout o finché raggiunge il limite di iterazioni. Il solo conteggio grezzo dei passi produce fasce nette; aggiungendo una parte frazionaria ricavata dal modulo finale si ottiene un colore continuo, senza fasce.
help-how-precision = Posizione a precisione arbitraria
help-how-precision-text = I normali numeri a 64 bit esauriscono le cifre vicino a 1e15× di zoom. Fractadyne conserva il centro della vista a precisione arbitraria, con un numero di cifre che cresce zoomando, così la posizione non si degrada mai. Anche la scala dei pixel è tenuta con un esponente esteso, quindi non si blocca nemmeno al limite di ~1e308× dei 64 bit — la profondità è limitata solo dalla pazienza, non da un intervallo numerico fisso.
help-how-perturbation = Perturbazione
help-how-perturbation-text = Iterare ogni pixel in alta precisione sarebbe troppo lento. Invece un pixel di riferimento viene iterato in alta precisione sulla CPU (l’«orbita di riferimento»), e ogni altro pixel viene calcolato sulla GPU come una minuscola differenza δ da esso, in bassa precisione veloce: δz → 2·Z·δz + δz² + δc.
help-how-floatexp = Profondità estrema (floatexp)
help-how-floatexp-text = Oltre circa 1e28× anche quella minuscola differenza va in underflow nell’intervallo a 32 bit, quindi viene memorizzata come mantissa più un esponente intero separato («floatexp»), superando il muro di profondità dei 32 bit. Il motore cambia automaticamente: calcolo diretto a basse profondità, perturbazione in profondità, e floatexp alle profondità massime. La profondità è allora limitata dalla precisione delle coordinate e dal budget di iterazioni anziché da un limite fisso — verificata con Fraktaler-3 su un corpus di riferimento di 39 posizioni, la più profonda a 5.63e18003× (le 38 righe non estreme sono il controllo di routine, fino a 6.13e1105×), con una batteria interna di test di autoconsistenza della precisione fino a 1e1000000×; il tour incluso raggiunge ~1.3e420×.
help-how-reference = Scelta del riferimento e ribasamento
help-how-reference-text = Il riferimento viene scelto (valutato in alta precisione) in modo che la sua orbita resti nella vista il più a lungo possibile. Quando la differenza cresce troppo, viene «ribasata» sul riferimento per restare accurata.
help-how-series = Approssimazione in serie
help-how-series-text = Nello zoom profondo le prime iterazioni spostano appena δ dal riferimento, quindi si possono calcolare tutte insieme: δz viene approssimato da un breve polinomio in δc (di ordine 3), i cui coefficienti vengono iterati lungo il riferimento. Il renderer salta in avanti fino all’ultima iterazione in cui quel polinomio è ancora accurato per l’intera vista, poi itera normalmente — risparmiando i passi saltati senza alcun cambiamento nell’immagine (attivabile in Controlli ▸ Avanzate). Mandelbrot e Multibrot 3–8, intervallo più profondo; non con le colorazioni media delle strisce / disuguaglianza triangolare / trappola orbitale / decomposizione (queste richiedono ogni iterazione). Dove la BLA è attiva, comprende già questo salto iniziale.
help-how-distance = Stima della distanza e illuminazione
help-how-distance-text = Seguendo la derivata dz/dc si ottiene la distanza di ogni pixel dal bordo. Questo alimenta l’illuminazione a rilievo 3D, il bagliore distanza e il metodo di colorazione «stima della distanza» — tutti validi a qualsiasi profondità di zoom.
help-how-safety = Anti-aliasing e sicurezza
help-how-safety-text = Le immagini ferme vengono supercampionate (2–8× per asse) una volta che la vista si è assestata. Un budget di lavoro mantiene ogni singolo disegno della GPU entro il limite del watchdog del driver riducendo la risoluzione (mai il numero di iterazioni) con impostazioni estreme, così le viste profonde restano dettagliate invece di diventare vuote.

## Help ▸ Command line (the flags themselves stay English)

# The section's introduction. The flags listed below it are the command line itself, English in every language.
help-cli-intro = Fractadyne funziona anche senza interfaccia (headless) per rendering, esportazione di filmati/tour, automazione, controlli con immagini di riferimento (golden image) e benchmark. Senza opzioni avvia l’interfaccia grafica interattiva.

## Help ▸ Shortcuts

# What each mouse gesture and key does (the gestures are the Navigation section's).
help-shortcuts-pan = Spostare la vista
# A gesture: turning the mouse wheel.
help-key-wheel = Rotellina
help-shortcuts-wheel = Zoom sul cursore
help-shortcuts-box-zoom = Zoom a rettangolo (trascinare un rettangolo per zoomare su di esso)
# $icon: the click-to-zoom toolbar button's symbol. "tool on": while that button is on.
help-key-click-tool = Clic (strumento { $icon } attivo)
help-shortcuts-click = Tuffo nel punto; il clic destro torna indietro
# A heading: the keyboard shortcuts.
help-shortcuts-keyboard = Tastiera
help-shortcuts-space = Zoom continuo avanti / indietro
help-shortcuts-undo = Annulla vista
help-shortcuts-redo = Ripeti vista
help-shortcuts-minibrot = Trova minibrot + zoom (Newton-Raphson)
help-shortcuts-autopilot = Pilota automatico dell’auto-zoom (tuffo verso i dettagli; qualsiasi comando lo ferma)
# off / strip / panel / full: the panel's sizes, as View ▸ Live render panel names them.
help-shortcuts-live-panel = Pannello rendering in tempo reale: spento / striscia / pannello / completo (passando sopra uno strumento si legge che cosa significa; la sua × lo chiude)
help-shortcuts-esc = Ferma il pilota automatico / un tour in riproduzione, o esce dallo schermo intero
help-shortcuts-snapshot = Istantanea nell’ultima cartella di esportazione — una cattura dello schermo della vista così com’è, o un rendering completo con le impostazioni di esportazione; la prima pressione chiede quale, e File → Impostazioni → Istantanea cambia la scelta.
help-shortcuts-help = Apre questa guida
# A heading: the shortcuts of the Game of Life, with the menu path to it.
help-shortcuts-life = Life (Frattale → Automi → Life)
help-shortcuts-life-run = Avvia / mette in pausa l’universo
help-shortcuts-life-step = Avanza di una generazione / del salto (impostato nel pannello Life)
help-shortcuts-life-draw = Attiva / disattiva lo strumento di disegno (la matita nella barra degli strumenti)
# A gesture. "Draw on": while the Life draw tool is on.
help-key-click-drag-draw = Clic / trascina (Disegno attivo)
help-shortcuts-life-paint = Inverte una cella / dipinge: un tratto che inizia su una cella morta disegna, uno che inizia su una cella viva cancella. Un universo in esecuzione si mette in pausa mentre disegni e riprende quando rilasci.
# $key: a keyboard shortcut (Ctrl+Z). "Draw on": while the Life draw tool is on.
help-key-when-drawing = { $key } (Disegno attivo)
help-shortcuts-life-undo = Annulla l’ultima modifica, riempimento, svuotamento o cambio di regola

## Help ▸ Recommended hardware

help-hardware-intro = Fractadyne divide il lavoro tra la GPU (iterazione per pixel + colorazione) e la CPU (l’orbita di riferimento a precisione arbitraria su cui si basa lo zoom profondo). Contano entrambe, ma per cose diverse.
help-hardware-gpu = GPU (la più importante)
help-hardware-gpu-text = Ogni pixel viene iterato e colorato sulla GPU, quindi la velocità della GPU determina la frequenza dei fotogrammi e il tempo di esportazione. Una GPU dedicata con un buon throughput a 32 bit (precisione singola) dà lo zoom profondo più fluido. Le GPU integrate funzionano bene — solo più lentamente, e raggiungono prima il budget di lavoro per fotogramma (a quel punto il renderer riduce la risoluzione, non i dettagli). Richiede un’API grafica moderna: Vulkan, Direct3D 12, Metal, o OpenGL tramite wgpu.
# The left column of a requirements table.
help-hardware-minimum = Minimo
help-hardware-gpu-minimum = Qualsiasi GPU con Vulkan / DX12 / Metal / GL 3.3.
help-hardware-recommended = Consigliato
help-hardware-gpu-recommended = Una GPU dedicata (degli ultimi ~5 anni); >= 2 GB di VRAM.
help-hardware-big-exports = Per grandi esportazioni
help-hardware-gpu-big-exports = >= 4 GB di VRAM (a riquadri, quindi degrada con gradualità).
# A heading: the processor.
help-hardware-cpu = CPU
help-hardware-cpu-text = L’orbita di riferimento ad alta precisione (e le tabelle BLA) vengono calcolate sulla CPU e sono in gran parte a thread singolo, quindi conta soprattutto la velocità per core — determina quanto rapidamente una vista profonda si risolve dopo uno spostamento. I core aggiuntivi aiutano soprattutto la codifica delle esportazioni in background. È richiesta una CPU a 64 bit.
help-hardware-cpu-recommended = Una CPU moderna a 64 bit con una buona velocità su singolo core.
help-hardware-memory = Memoria
help-hardware-memory-text = I riferimenti profondi crescono con la profondità (molte iterazioni ad alta precisione); le esportazioni enormi tengono i riquadri in RAM. 8 GB sono comodi per l’esplorazione quotidiana; 16 GB o più sono adatti alle profondità estreme e alle esportazioni molto grandi.
help-hardware-checking = Verificare il proprio hardware
help-hardware-diagnostics = Aiuto → «Diagnostica…» esegue sulla tua macchina gli stessi test che usano gli sviluppatori. «Esegui autotest» verifica la matematica e il rendering rispetto a risultati notoriamente corretti in circa quindici secondi; «Esegui test dell’interfaccia» percorre l’interfaccia e la vista in tempo reale a diverse profondità di zoom, catturando screenshot; «Esegui verifica aritmetica GPU» verifica l’aritmetica double-float dello shader su ogni backend grafico offerto dalla tua macchina, in pochi secondi e senza finestre. Ognuno gira come processo separato, quindi un test che manda in crisi il driver grafico fa cadere il test anziché la tua sessione.
help-hardware-gpu-check = La verifica aritmetica GPU non ha esito superato o non superato — registra come calcolano la tua GPU e il tuo driver, e su alcuni hardware (tutte le schede NVIDIA provate finora) segnala che il compilatore degli shader scarta parte dell’aritmetica a precisione estesa. È una scoperta reale che stiamo cercando di mappare tra i produttori, non un guasto della tua macchina, quindi qualunque cosa dica, allegarla a una segnalazione aiuta davvero.
help-hardware-report = Se qualcosa sembra sbagliato, esegui l’autotest e usa «Allega a una segnalazione…»: la segnalazione porterà allora un risultato convalidato dalla macchina sul tuo hardware, molto più utile di una semplice descrizione. Un’avvertenza da conoscere — le immagini di riferimento sono state prodotte su una specifica scheda grafica, e GPU diverse calcolano legittimamente pixel leggermente diversi, quindi su altro hardware i confronti tra immagini vengono verificati con una tolleranza più ampia e lì piccole differenze sono attese, non guasti.
help-hardware-notes = Note
help-hardware-note-watchdog = Una GPU capace evita che il budget di lavoro del watchdog intervenga con impostazioni estreme.
help-hardware-note-binary = La build è un unico eseguibile autonomo — non serve installare alcun runtime.
help-hardware-note-benchmark = Esegui il benchmark integrato (Strumenti ▸ Benchmark) per valutare la tua macchina.

## Help ▸ Acknowledgments & citations
## Names of people, programs, papers, books and websites stay as written, and so do the
## titles in quotation marks (the works' own titles); the prose around them translates.

help-ack-title = Riconoscimenti e citazioni
# The dedication, in italics at the top of the section.
help-ack-dedication = Dedicato allo Stone Soup Group — i volontari dietro Fractint (dal 1988).
help-ack-dedication-text = Il loro codice condiviso liberamente e il loro spirito di esplorazione dei frattali aperta e collaborativa sono la ragione per cui tanti di noi si sono innamorati di quest’arte. Fractadyne è costruito in quella tradizione.
help-ack-intro = Fractadyne poggia su decenni di lavoro della comunità dei frattali. Tecniche e lavori precedenti su cui si basa — verificati con le fonti qui sotto:
help-ack-deep-zoom = Algoritmi di zoom profondo
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-martin-title = Perturbazione e approssimazione in serie — { $who }
help-ack-martin = Il metodo orbita di riferimento + delta a bassa precisione (δz → 2Z·δz + δz² + δc) e la serie di ordine n che salta le prime iterazioni, introdotti in SuperFractalThing e nella sua nota «SuperFractalThing Maths» (sft_maths.pdf, 2013).
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bla-title = Approssimazione lineare bivariata (BLA) e ribasamento — { $who }
help-ack-bla = Saltare sequenze di iterazioni tramite mappe lineari unite, e ribasare su un unico riferimento per evitare i glitch; sviluppato su fractalforums.org (2021).
# A link's label. $who: the linked page's author.
help-ack-bla-link = La BLA spiegata — { $who }
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-glitch-title = Criterio di rilevamento dei glitch — { $who }
help-ack-glitch = Il test |Z + z| << |Z| per stabilire quando un pixel ha bisogno di un altro riferimento — la base della perturbazione senza glitch (fractalforums).
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-diffabs-title = Perturbazione non analitica (Burning Ship) — { $who }
help-ack-diffabs = L’analisi per casi del valore assoluto «diffabs» che fa funzionare la perturbazione per le famiglie basate su abs (Burning Ship / Celtic / Buffalo), su fractalforums.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mathr-title = Implementazioni di riferimento e verifiche incrociate — { $who }
help-ack-mathr = Fraktaler-3 e Kalles Fraktaler 2+ (in origine di Karl Runmo) — i cui scritti hanno guidato questo motore, e con il cui output EXR Fractadyne verifica il proprio renderer.
# A heading: credits for the coloring methods.
help-ack-coloring = Colorazione
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-harkonen-title = Conteggio continuo delle iterazioni e media delle strisce (Stripe Average) — { $who }
help-ack-harkonen = La colorazione continua (normalizzata) a tempo di fuga e il metodo della media delle strisce (Stripe Average), da «On Smooth Fractal Coloring Techniques» (tesi di laurea magistrale, 2007), che analizza anche le colorazioni a disuguaglianza triangolare / branching average.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mitchell-title = Media della disuguaglianza triangolare (Triangle Inequality Average) — { $who }
help-ack-mitchell = Media dei punti in cui ogni z² + c cade entro i limiti della disuguaglianza triangolare, per una trama esterna morbida simile a fiamme.
# A link's label: the person's tutorial pages. $who: the person's name.
help-ack-mitchell-link = { $who } — tutorial
help-ack-distance = La colorazione/illuminazione a stima della distanza usa il classico metodo dz/dc della distanza dal bordo (vedi Peitgen & Saupe, «The Science of Fractal Images», 1988).
help-ack-lsystems-life = L-system e il Gioco della vita
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-lindenmayer-title = L-system — { $who }
help-ack-lindenmayer = Riscrittura parallela di stringhe, introdotta nel 1968 per modellare la crescita di piante e alghe («Mathematical models for cellular interactions in development», Journal of Theoretical Biology).
help-ack-abop = Il libro (1990) di cui il lettore di L-system segue l’interpretazione a tartaruga e la notazione per i sistemi stocastici, sensibili al contesto e parametrici; i sistemi integrati contrassegnati ABOP sono ricavati dalle sue figure.
# A link's label: the book can be read free on that website. $site: the website's name.
help-ack-abop-link = Il libro, gratuito su { $site }
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bourke-title = Raccolta di L-system — { $who }
help-ack-bourke = Molti dei sistemi integrati provengono dalle sue pagine sugli L-system, tra cui le regole di Chris Wallace per la curva di Sierpinski e il cespuglio di Dietmar Saupe. I file .l di Fractint, che si aprono direttamente, sono nel formato dello Stone Soup Group.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-conway-title = Il Gioco della vita — { $who }
help-ack-conway = L’automa cellulare B3/S23 (1970), che arrivò al mondo attraverso la rubrica «Mathematical Games» di Martin Gardner su Scientific American (ottobre 1970).
# A credit's heading: the website, then who runs it. $site: the website's name.
help-ack-lifewiki-title = { $site } — la comunità di ConwayLife.com
help-ack-lifewiki = Il riferimento per le notazioni delle regole lette dall’app (B/S, von Neumann, Generations, le lettere di Alan Hensel per le regole non totalistiche, e MAP), per il formato di pattern RLE che condivide con Golly, e per le regole e i pattern integrati, di cui registra gli scopritori.
help-ack-foundations = Fondamenti e strumenti
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mandelbrot-title = L’insieme di Mandelbrot — { $who }
help-ack-mandelbrot = L’oggetto al centro di tutto.
help-ack-built-with = Realizzato con Rust, egui/eframe (emilk) e wgpu (gfx-rs); precisione arbitraria tramite astro-float, float a doppia parola tramite twofloat; PNG/OpenEXR, rfd, serde. Grazie ai loro autori e manutentori.
help-ack-errors = Eventuali errori di attribuzione sono dell’autore; le correzioni sono benvenute nel tracker delle issue del progetto.

## Help ▸ Third-party licenses

help-licenses-title = Licenze di terze parti
# $licenses: Fractadyne's license expression (MIT OR Apache-2.0), as written. $url: the web address of a crate's source. THIRD-PARTY-NOTICES.md is a file name, `option-ext` a crate's name, MPL-2.0 a license's.
help-licenses-intro = Fractadyne è distribuito come eseguibile collegato staticamente che include i crate open source e i font elencati qui sotto. Seguono i testi completi delle licenze (anche in THIRD-PARTY-NOTICES.md, incluso nel download). Il codice di Fractadyne è a doppia licenza { $licenses }. `option-ext` è MPL-2.0 — il suo sorgente non modificato è su { $url }.
# A button, after a copy icon: copies every notice to the clipboard.
help-licenses-copy-all = Copia tutti gli avvisi

## Help ▸ About

help-about-tagline = Un esploratore di frattali nativo scritto in Rust con wgpu.
# The arithmetic deep zoom has used. $ran: the libraries that have run so far, as names (or a note that none has yet); $built: those this build contains, with versions.
help-about-arithmetic = Aritmetica dello zoom profondo: { $ran } (questa build contiene: { $built }).
help-about-files = File e dati
# Followed by the folder's path.
help-about-files-text = La tua sessione (vista corrente, colorazione e preferenze), i segnalibri e le loro miniature, i gradienti salvati e le formule salvate sono memorizzati in questa cartella per utente, e ricaricati al prossimo avvio:
# Tooltip of the copy button beside the folder's path.
help-about-copy-path = Copia percorso
# $command: a command typed in a terminal (it stays English).
help-about-session = Il file di sessione ha una versione; se è stato scritto da un Fractadyne più recente di quanto questo possa leggere completamente, l’app lo carica al meglio e ti avvisa. Per ripartire completamente da zero, usa File ▸ Reimposta stato dell’applicazione… (chiede prima conferma), oppure esegui `{ $command }` da un terminale.
help-about-license = Licenza
# $licenses: the license expression (MIT OR Apache-2.0), which stays as written.
help-about-license-text = { $licenses } — da usare con l’una o l’altra licenza, a scelta.
# A link to the source code, followed by ↗.
help-about-source = Sorgente su GitHub

## The live render panel ("blinkenlights": View ▸ Live render panel, key B)
## A front panel drawn over the view like a piece of 1970s test equipment. Its legends are engraved
## in capitals: write them in capitals here (the panel shows them exactly as written, it does not
## uppercase them). Numbers, units and readouts on its tubes and dials are not translated.

# Tooltip of the × button in the panel's top-right corner. The code adds the key after it: (B).
live-close-hover = Chiudere il pannello rendering in tempo reale
# The legend engraved along the top plate, left of the maker's badge.
live-plate = RENDERING LIVE · PANNELLO FRONTALE

## Live render panel: instrument titles
## Engraved above each instrument, and repeated as the title of its help card.

# Above the depth readout, whose tubes show the magnification as a power of two ("2^" stays).
live-title-depth = PROFONDITÀ 2^
live-title-iterations = ITERAZIONI
# Above the frame-time readout (milliseconds).
live-title-frame-ms = MS FOTOGRAMMA
# Above the date-and-time readout.
live-title-clock = DATA · ORA
# The walk dial: a radar screen showing a slow refresh rendered pass after pass, band after band.
live-title-walk = PERCORSO · FASCE
# A meter of how far the picture on screen lags the view.
live-title-age = ETÀ IMMAGINE
# What part of the picture this frame's GPU pass covered.
live-title-tiles = RIQUADRI
# A scope of the GPU's work per frame, in milliseconds.
live-title-scope = PASSATE GPU · MS
# The depth dial: depth, the reference orbits being built, and the arithmetic in use.
live-title-dial = PROFONDITÀ · RIFERIMENTI · ARITMETICA
# Iteration steps per second.
live-title-steps = PASSI / S
# The two GPUs (the window's and a second, worker GPU).
live-title-engines = DUE MOTORI
live-title-memory = MEMORIA
live-title-cores = CORE · THREAD
live-title-pixels = FLUSSO DI PIXEL
# Help card titles only: the three GPU timer lamps (T0, T1, T2 stay as they are) and the row of lamps.
live-title-timers = TIMER GPU T0–T2
live-title-bank = BANCO DI SPIE

## Live render panel: the light bank's lamps
## Engraved under each lamp, and the title of its help card. A cell is narrow (about eight capital
## letters): a longer label breaks at a space onto two lines, and is drawn smaller if it still does
## not fit.

live-lamp-plan = PIANO
live-lamp-dispatch = INVIO
live-lamp-live = ATTUALE
live-lamp-held = TENUTA
# The last picture reprojected (stretched) to fit the moving view.
live-lamp-reproj = RIPROIEZ.
live-lamp-reading = LETTURA
# A pass of a pinned walk.
live-lamp-pin-pass = PASSATA FISSA
live-lamp-split = DIVISA
live-lamp-adopt = ADOZIONE
live-lamp-abandon = ABBANDONO
# "REF": a reference orbit. Built for the view as it is (LIVE), ahead of it (AHEAD), or for a tour's next hold (HOLD).
live-lamp-ref-live = RIF ATTUALE
live-lamp-ref-ahead = RIF AVANTI
live-lamp-ref-hold = RIF SOSTA
# The series approximation.
live-lamp-series = SERIE
# The second (worker) GPU.
live-lamp-worker = AUSILIARIA
live-lamp-tile = RIQUADRO
# Supersampling (anti-aliasing); usually kept as SSAA.
live-lamp-ssaa = SSAA
live-lamp-palette = TAVOLOZZA
# The arithmetic in use (direct, df32, floatexp).
live-lamp-mode = MODALITÀ

## Live render panel: the kinds of GPU pass
## In the GPU scope's legend (MOVING, PIN, WORKER, SETTLE, SSAA). SETTLE and SSAA also name a still
## view's walk on the walk dial.

# A moving frame's pass.
live-kind-moving = IN MOTO
# A pass of a pinned walk.
live-kind-pin = FISSA
# The worker GPU's passes.
live-kind-worker = AUSILIARIA
# A still view's settle tile or walk pass.
live-kind-settle = RIFINITURA
# A supersampling sample.
live-kind-ssaa = SSAA

## Live render panel: legends and readouts on the instruments

# On the walk dial during a split refresh. $done: passes done; $passes: passes in all.
live-walk-split = DIVISA { $done }/{ $passes }
# On the picture-age meter's face, under its scale (0 to 2 octaves).
live-age-scale = OTTAVE DI RITARDO
# The depth dial's band for direct (plain floating-point) arithmetic. The other bands, DF32 and FLOATEXP, are names and stay.
live-dial-direct = DIRETTA
# Under the two GPU gauges: GPU 0 draws the window; GPU 1 is the worker, or there is none.
live-engines-gpu-window = GPU 0 · FINESTRA
live-engines-gpu-worker = GPU 1 · AUSILIARIA
live-engines-no-worker = NESSUNA AUSILIARIA
# The share of displayed frames each GPU made. $window and $worker: percentages, 0–100.
live-engines-on-screen = SULLO SCHERMO { $window } : { $worker }
# The same readout before any frame was shown.
live-engines-on-screen-none = SULLO SCHERMO —
# Under the CPU core bars. $count: the machine's cores.
live-cores-count = { $count } CORE
# Under the threads scope. $count: the app's busy threads.
live-cores-threads = THREAD { $count }

## Live render panel: the memory tanks' legend
## What the GPU memory holds, by kind. Very narrow: about ten letters.

# G-buffers (the picture's working textures).
live-memory-gbuf = G-BUF
# The held picture and the supersampling accumulator.
live-memory-hold = TENUTA+SSAA
# The reference orbit and its BLA (bilinear approximation) table.
live-memory-orbit = ORBITA+BLA
# The worker GPU's copy of the reference.
live-memory-worker = RIF AUSIL.

## Live render panel: the roles on the threads scope
## Tiny labels on a small scope: at most four letters each.

# The reference builders.
live-cores-role-ref = RIF
# The scoring pool.
live-cores-role-pool = POOL
# The thread driving the worker GPU.
live-cores-role-worker = AUS
# Probes: short side renders.
live-cores-role-probe = SND
# Image and tour exports.
live-cores-role-export = ESP

## Live render panel: the tiles instrument's captions
## The line under the picture's plane, then the tile size (left) and the tiles a frame (right).

# A walk's pass. $lo and $hi: the iteration band it rendered (numbers).
live-tiles-pin-band = PASSATA FISSA · FASCIA { $lo }–{ $hi }
live-tiles-settle-band = PASSATA DI RIFINITURA · FASCIA { $lo }–{ $hi }
live-tiles-moving-whole = FOTOGRAMMA IN MOTO · IMMAGINE INTERA
live-tiles-settled-whole = FOTOGRAMMA ASSESTATO · IMMAGINE INTERA
# $pass: this pass; $passes: the passes the refresh is split into.
live-tiles-split-pass = AGGIORNAMENTO DIVISO · PASSATA { $pass } DI { $passes }
# $grid: the grid's columns × rows, e.g. 12×8.
live-tiles-grid-done = GRIGLIA DI RIFINITURA { $grid } · COMPLETATA
# $grid: the grid's columns × rows; $done and $total: tiles done, and in all.
live-tiles-grid-progress = GRIGLIA DI RIFINITURA { $grid } · { $done } DI { $total } FATTI
# The same, while a tile too costly for one frame goes out in strips.
live-tiles-grid-strips = GRIGLIA DI RIFINITURA { $grid } · { $done } DI { $total } FATTI · A STRISCE
# $samples: supersampling samples folded so far; $lo and $hi: the iteration band the next one is walking.
live-tiles-ssaa-band = SSAA · { $samples } FATTI · FASCIA { $lo }–{ $hi }
# $sample: the supersampling sample.
live-tiles-ssaa-whole = CAMPIONE SSAA { $sample } · FOTOGRAMMA INTERO
# No GPU pass this frame, and the panel has not seen a frame yet.
live-tiles-idle-starting = AVVIO
# No GPU pass this frame: the last complete picture is held on screen.
live-tiles-idle-held = TENUTA
# No GPU pass this frame: the last picture is stretched to fit the view.
live-tiles-idle-reprojected = RIPROIETTATA
# No GPU pass this frame: the picture on screen is current.
live-tiles-idle-live = ATTUALE
# No GPU pass this frame, and nothing to show.
live-tiles-idle-none = INATTIVO
# The tile size: $res, a whole picture's resolution (2400×1300).
live-tiles-tile-res = RIQUADRO { $res }
# The tile size: $px, a tile's side in pixels.
live-tiles-tile-px = RIQUADRO { $px } PX
live-tiles-no-pass = NESSUNA PASSATA
# Tiles a frame. $count: how many (0 when none).
live-tiles-rate-count = { $count }/FOTOGRAMMA
live-tiles-rate-one = 1 RIQUADRO/FOTOGRAMMA
# $share: the part of one tile sent each frame, e.g. 0.33.
live-tiles-rate-part = { $share } RIQUADRO/FOTOGRAMMA
live-tiles-rate-sample = 1 RIQUADRO/CAMPIONE
# In the plane's corner during a walk: the share of pixels still iterating. $percent: a percentage, e.g. 54 or 1.5.
live-tiles-running = IN CORSO { $percent }%

## Live render panel: help cards
## Shown when the pointer rests on an instrument or a lamp. Words in capitals (SETTLE, SSAA, ABANDON,
## HELD, REPROJ, ADOPT, REF AHEAD, ON SCREEN) are the panel's own legends: use the same words as
## their messages above.

# The heading of a card's list of what healthy looks like.
live-card-healthy = NORMALE
# The heading of a card's list of what signals trouble.
live-card-trouble = PROBLEMI

live-card-walk-what = Un aggiornamento profondo troppo costoso per un solo fotogramma viene fissato e percorso: renderizzato passata dopo passata, ognuna delle quali lascia iterare i pixel un’ottava più in là (256, 512, 1.024 … fino al valore richiesto). Il bordo è quel conteggio in scala logaritmica, a partire dall’alto; il fascio è l’avanzamento del percorso; ogni arco è una passata, colorata in base al suo tempo GPU rispetto all’obiettivo di 20 ms. Il punto bianco attende la lettura dell’ultima passata; il lampo è la nuova immagine che va sullo schermo.
live-card-walk-note = Una vista ferma troppo costosa per una sola passata viene percorsa allo stesso modo, contrassegnata RIFINITURA, e ogni campione di supersampling viene percorso di nuovo, contrassegnato SSAA: le loro passate possono durare fino a 400 ms ciascuna, e gli archi sono colorati rispetto a quel valore. Un aggiornamento diviso disegna invece un anello per passata: tutte le fasce, su una parte dei riquadri.
live-card-walk-good-sweep = Una scansione che fa il giro in pochi fotogrammi e finisce con un lampo.
live-card-walk-good-arcs = Archi per lo più verdi e ambra.
live-card-walk-good-still = Su una vista ferma profonda: una scansione più lenta per campione, con archi dal verde all’ambra.
live-card-walk-bad-red = Archi rossi: passate oltre il budget, percepite come scatti.
live-card-walk-bad-restart = Una scansione che si ferma a metà e ricomincia: il percorso è stato abbandonato, e l’immagine sullo schermo resta vecchia.
live-card-walk-bad-hollow = Archi vuoti: passate il cui tempo non è mai tornato, quindi la passata successiva viene dimensionata alla cieca.

live-card-age-what = Quanto l’immagine sullo schermo è in ritardo rispetto alla vista, in ottave di zoom (un’ottava è 2×). Mentre un aggiornamento si renderizza, viene mostrata la vecchia immagine stirata, che invecchia; quando l’aggiornamento arriva, l’età scende. La striscia sotto mostra gli ultimi secondi.
live-card-age-good-sawtooth = Un dente di sega tra circa 0,3 e 0,8, con l’ago che oscilla attorno al segno 0,5.
live-card-age-good-still = Vicino a 0 quando ti fermi.
live-card-age-bad-red = L’ago che resta oltre 1,0, nel rosso: l’immagine è indietro di più di 2× e appare sfocata o a blocchi durante lo zoom.
live-card-age-bad-ramp = Una rampa che non scende mai: gli aggiornamenti non finiscono (osserva ABBANDONO).

live-card-tiles-what = Come la passata GPU di questo fotogramma copre l’immagine. I fotogrammi in movimento e le passate di percorso la coprono per intero, e la didascalia indica la fascia di iterazioni di una passata di percorso. Una passata di percorso itera solo i pixel che non sono ancora fuggiti: il piano si riempie fino a quella frazione (scala logaritmica, 0,01% al minimo) e la indica nell’angolo, quindi si svuota man mano che il percorso va più in profondità. Un aggiornamento appena troppo costoso per un fotogramma viene diviso in un massimo di 8 passate, ciascuna un insieme intercalato di riquadri da 16 px. Una vista assestata troppo costosa per una sola passata diventa una griglia di riquadri, uno per fotogramma dal centro verso l’esterno, e un riquadro ancora troppo costoso viene inviato a strisce. Indicati sotto: dimensione dei riquadri e riquadri per fotogramma.
live-card-tiles-good-whole = Passate intere durante lo zoom; aggiornamenti divisi in poche passate.
live-card-tiles-good-grid = Quando ti fermi: una griglia di decine di riquadri riempita in un secondo o due, poi SSAA.
live-card-tiles-bad-small = Una griglia di centinaia di piccoli riquadri: il budget di ogni passata è piccolo, e la definizione richiederà molto tempo.
live-card-tiles-bad-strips = Molti riquadri inviati a strisce: i riquadri sono stati più lenti di quanto previsto nel dimensionare la griglia.
live-card-tiles-bad-splits = Divisioni ferme a 8 passate: l’aggiornamento ci sta a malapena, e un po’ più in profondità ripiega sul percorso, più lento.
live-card-tiles-bad-dark = Buio per lunghi tratti durante il movimento: nessuna passata.

live-card-scope-what = Una colonna per fotogramma, la più recente a destra: il lavoro GPU eseguito, impilato per tipo (fotogramma in moto verde, passata fissa dal verde al rosso secondo il costo, riquadro di rifinitura o passata di percorso di una vista ferma crema, campione di supersampling lavanda, fotogrammi e campioni della GPU ausiliaria blu), in millisecondi. Linee tratteggiate a 16,7 ms (60 fps), 33 e 50. Un segno vuoto è una passata il cui tempo non è mai tornato.
live-card-scope-good-under = Colonne sotto la linea dei 16,7 durante il movimento.
live-card-scope-good-pin = Passate fisse dal verde all’ambra.
live-card-scope-good-samples = I campioni di supersampling di una vista ferma possono durare di più: non si muove nulla.
live-card-scope-bad-over = Colonne oltre 33 o 50 ms: scatti visibili.
live-card-scope-bad-seconds = Passate di secondi: il watchdog della GPU può resettare il dispositivo.
live-card-scope-bad-hollow = Molti segni vuoti: il lavoro viene dimensionato senza misure.

live-card-timers-what = I tre slot di timestamp della GPU. Uno slot si accende quando una passata viene cronometrata e si spegne quando torna la sua lettura, due o tre fotogrammi dopo. T0 cronometra la passata di ogni fotogramma; T1 e T2 cronometrano passate fisse aggiuntive.
live-card-timers-good = T0 lampeggia ogni volta che c’è lavoro; T1 e T2 lampeggiano durante i percorsi.
live-card-timers-bad-stuck = Una spia rimasta accesa: la sua lettura non torna, e le stime dei costi diventano obsolete.
live-card-timers-bad-dark = Tutto spento mentre la GPU è occupata: non si misura nulla.

live-card-dial-what = La lancetta rossa è la profondità della vista: il suo ingrandimento come potenza di due. Le fasce sono l’aritmetica che la GPU usa lì: diretta, df32 (due float per numero) e floatexp (un float con un esponente ampio) oltre 2^93. I punti sono orbite di riferimento in costruzione: verdi per la vista, ambra più avanti, dove lo zoom si sta dirigendo, blu per la prossima sosta di un tour. Un anello è un’installazione; una croce rossa è una costruzione abbandonata perché la vista l’ha superata.
live-card-dial-good-ahead = Punti ambra che compaiono davanti alla lancetta e diventano anelli prima che li raggiunga.
live-card-dial-good-crosses = Poche croci rosse.
live-card-dial-bad-passing = La lancetta che supera l’ultimo riferimento installato: i fotogrammi ne aspettano uno, e lo si percepisce come un blocco.
live-card-dial-bad-crosses = Croci rosse ripetute: lo zoom corre più veloce dei costruttori, e si spreca CPU.
live-card-dial-bad-crossing = Il passaggio di 2^93 è un punto lento noto: i primi fotogrammi floatexp costano di più.

live-card-bank-what = Ogni spia si accende quando avviene il suo evento e si affievolisce in circa 150 ms, così un renderer occupato sfarfalla e uno inattivo resta spento. Da sinistra a destra: il fotogramma, l’aggiornamento, i riferimenti, poi la GPU ausiliaria, la rifinitura, la tavolozza e l’aritmetica.
live-card-bank-note = Passa sopra una spia per sapere che cosa significa.

live-card-depth-what = L’ingrandimento come potenza di due: 2^100 è circa 1,3 × 10^30.
live-card-depth-note = Oltre 2^93 ogni passo costa di più (floatexp), e la costruzione dei riferimenti richiede più tempo.

live-card-iter-what = Le iterazioni richieste: fin dove ogni pixel può iterare prima di essere considerato dentro l’insieme. Crescono con la profondità.
live-card-iter-good = In crescita costante durante il tuffo.
live-card-iter-bad-low = Bloccate in basso in profondità: spirali e minibrot appaiono come interno piatto.
live-card-iter-bad-high = Molto alte: ogni aggiornamento costa di più, e i percorsi durano di più.

live-card-ms-what = Il tempo tra i fotogrammi visualizzati.
live-card-ms-good = L’intervallo del tuo schermo: 16,7 su uno schermo a 60 Hz.
live-card-ms-bad-stutter = Sopra 33: scatti visibili.
live-card-ms-bad-spikes = Picchi sopra 50: intoppi, di solito una passata costosa (vedi Passate GPU).

live-card-clock-what = L’orologio di sistema in ora locale, al decimo di secondo.
live-card-clock-note = Marca ogni screenshot del pannello con il momento in cui è stato fatto, così l’immagine di un problema può essere abbinata al log dell’app.

live-card-tach-what = Passi di iterazione al secondo. Bianco: i passi richiesti (pixel × iterazioni). Ambra: i passi effettivamente eseguiti dalla GPU. Il divario è la BLA (approssimazione bilineare) che salta molte iterazioni in una volta; ×N è quel moltiplicatore. Il contachilometri totalizza la sessione.
live-card-tach-good = Un ampio divario in profondità: la BLA fa il suo lavoro, da ×10 a ×1.000.
live-card-tach-bad-together = Aghi affiancati in profondità: qui la BLA non può saltare (spesso vicino all’interno di un minibrot), quindi gli aggiornamenti sono lenti.
live-card-tach-bad-red = L’ago bianco nel rosso: si chiede più di quanto la GPU possa fare in un fotogramma.

live-card-engines-what = Quanto è occupata ogni GPU, con punti che portano i suoi fotogrammi sullo schermo: il tempo GPU delle sue passate nell’ultimo secondo, inclusi i fotogrammi in movimento e i campioni di supersampling della GPU ausiliaria. SULLO SCHERMO è la quota di fotogrammi visualizzati prodotta da ciascuna. Con una sola GPU, il lato ausiliario resta inattivo.
live-card-engines-good = Entrambe occupate durante un tuffo, con la GPU ausiliaria che fornisce una parte dei fotogrammi.
live-card-engines-bad-pinned = La GPU 0, quella della finestra, ferma vicino al 100%: è il collo di bottiglia, e i fotogrammi scattano.
live-card-engines-bad-idle = Una GPU ausiliaria attivata ma inattiva: non fornisce fotogrammi.

live-card-memory-what = RAM: il working set dell’app, la linea rossa il suo picco. VRAM: ciò che l’app tiene sulla GPU, calcolato dalle dimensioni che alloca, per tipo (G-buffer, texture di tenuta e SSAA, buffer di orbita e BLA, la copia del riferimento della GPU ausiliaria), rispetto alla scheda.
live-card-memory-good = Piatta durante lo zoom; sale a gradini e riscende man mano che i riferimenti vanno e vengono.
live-card-memory-bad-leak = RAM che sale e non scende mai: una perdita di memoria.
live-card-memory-bad-vram = VRAM vicina alla capacità della scheda: le allocazioni possono fallire, o il dispositivo può andare perso.

live-card-cores-what = Uso della CPU per core, dal verde all’ambra al rosso, e un oscilloscopio in cui ogni thread occupato orbita attorno al suo ruolo: i costruttori dei riferimenti, il pool di valutazione, la GPU ausiliaria, i rendering secondari.
live-card-cores-good = Uno o due core occupati durante lo zoom; una breve raffica su tutti i core quando viene scelto un riferimento.
live-card-cores-bad-every = Tutti i core occupati per secondi: costruzione dei riferimenti a profondità estrema.
live-card-cores-bad-own = Il core dell’app stessa fermo nel rosso: è la finestra stessa a scattare.

live-card-pixels-what = Pixel calcolati al secondo, in Mpx/s. Tenue: ancora in iterazione in un percorso. Brillante: completati.
live-card-pixels-good = Decine di Mpx/s durante il movimento.
live-card-pixels-bad = Vicino a zero mentre l’immagine è vecchia: la GPU sta aspettando qualcosa, un riferimento o una lettura.

## Live render panel: the lamps' help cards

live-card-lamp-plan-what = È stato pianificato un fotogramma: scelti risoluzione, iterazioni richieste e budget, e se aggiornare.
live-card-lamp-plan-good = Sfarfalla ogni volta che cambiano la vista o le impostazioni.
live-card-lamp-plan-bad = Spenta durante lo zoom: il thread dell’interfaccia è bloccato.

live-card-lamp-dispatch-what = In questo fotogramma è stato inviato lavoro alla GPU.
live-card-lamp-dispatch-good = Accesa mentre qualcosa cambia; spenta quando tutto è inattivo.
live-card-lamp-dispatch-bad = Occupata senza che nulla cambi: l’app non si sta assestando.

live-card-lamp-live-what = L’immagine sullo schermo è il rendering di questo stesso fotogramma.
live-card-lamp-live-note = Raramente accesa durante un tuffo profondo, ed è normale: vedi TENUTA e RIPROIEZ.
live-card-lamp-live-good = Accesa a basse profondità, dove un fotogramma costa poco.

# "Prefer detail" is a setting's name.
live-card-lamp-held-what = L’ultima immagine completa è sullo schermo mentre la successiva si compone sotto di essa: l’aggiornamento di uno zoom profondo o, con «Preferisci il dettaglio», il riquadro o il campione di supersampling successivo di una vista ferma.
live-card-lamp-held-good-dive = Accesa a tratti durante gli zoom profondi, interrotta da ADOZIONE.
live-card-lamp-held-good-still = Accesa fissa mentre una vista ferma si definisce, con SSAA che lampeggia.
live-card-lamp-held-bad = Accesa per secondi durante lo zoom senza ADOZIONE: gli aggiornamenti non finiscono.

live-card-lamp-reproj-what = L’ultima immagine, stirata per adattarsi alla vista in movimento finché non ne è pronta una nuova.
live-card-lamp-reproj-good = Brevi raffiche durante il movimento.
live-card-lamp-reproj-bad = Accesa a lungo: l’immagine diventa più sfocata quanto più viene stirata.

live-card-lamp-reading-what = Una lettura è tornata dalla GPU: il tempo, l’intervallo di fuga o il contenuto di una passata.
live-card-lamp-reading-good = Uno sfarfallio costante durante il rendering.
live-card-lamp-reading-bad = Spenta a lungo durante il rendering: i tempi non tornano, e i budget vanno a tentativi.

live-card-lamp-pin-pass-what = Una passata di un percorso fissato: l’immagine renderizzata una fascia di iterazioni più in là.
live-card-lamp-pin-pass-good = Serie di passate che terminano con ADOZIONE.
live-card-lamp-pin-pass-bad = Serie che terminano con ABBANDONO.

live-card-lamp-split-what = Una passata di un aggiornamento diviso: una parte dei riquadri, renderizzata con il numero completo di iterazioni.
live-card-lamp-split-good = Brevi serie che terminano con ADOZIONE.
live-card-lamp-split-bad = Serie di 8, il massimo consentito: l’aggiornamento ci sta a malapena.

live-card-lamp-adopt-what = Un aggiornamento completato ha sostituito l’immagine sullo schermo.
live-card-lamp-adopt-good = Diverse volte al secondo durante lo zoom.
# "Picture age" names the instrument (its title, PICTURE AGE).
live-card-lamp-adopt-bad = Rara durante lo zoom: l’immagine invecchia (vedi Età immagine).

live-card-lamp-abandon-what = Un aggiornamento in corso è stato buttato via: la vista si è spostata troppo, è diventato troppo vecchio, o uno più recente l’ha sostituito. Una costruzione di riferimento superata dalla vista la accende debolmente.
live-card-lamp-abandon-good = Rara, o una volta subito dopo un salto.
live-card-lamp-abandon-bad = Frequente: lavoro sprecato, e l’immagine resta vecchia.

live-card-lamp-ref-live-what = Si sta costruendo un’orbita di riferimento per la vista così com’è: non ne ha una, ha lasciato quella che aveva, o ne serve una più lunga.
live-card-lamp-ref-live-good = Brevi lampeggi.
live-card-lamp-ref-live-bad-long = Lunghi tratti: le viste profonde la aspettano.
live-card-lamp-ref-live-bad-lookahead = Accesa spesso durante un tuffo mentre RIF AVANTI è spenta: l’anticipo non tiene il passo.

live-card-lamp-ref-ahead-what = Si sta costruendo un riferimento di anticipo più in profondità della vista, dove lo zoom si sta dirigendo.
live-card-lamp-ref-ahead-good = Lampeggi regolari durante un tuffo.
live-card-lamp-ref-ahead-bad = Spenta durante un tuffo profondo: la vista supererà i suoi riferimenti.

live-card-lamp-ref-hold-what = Il prefetch della sosta di un tour: il riferimento per il prossimo fotogramma chiave in sosta, costruito durante la planata verso di esso, così la riproduzione non attende mai.
live-card-lamp-ref-hold-note = Spenta al di fuori della riproduzione di un tour.
live-card-lamp-ref-hold-good = Un lampeggio prima di ogni sosta in un tour.
live-card-lamp-ref-hold-bad = Accesa per un’intera sosta: il prefetch è in ritardo, e il tour lo aspetta.

live-card-lamp-series-what = L’approssimazione in serie per un nuovo riferimento, che permette a ogni pixel di saltare le prime iterazioni.
live-card-lamp-series-good = Breve.
live-card-lamp-series-bad = Lunga in profondità: ritarda l’installazione del riferimento.

live-card-lamp-worker-what = La seconda GPU ha consegnato un fotogramma.
live-card-lamp-worker-good = Lampeggi regolari quando una GPU ausiliaria è attivata.
live-card-lamp-worker-bad = Spenta con una GPU ausiliaria attivata: non sta consegnando.

live-card-lamp-tile-what = Un riquadro di rifinitura, o una sua striscia, è stato inviato alla GPU.
live-card-lamp-tile-good = Una raffica dopo che ti fermi, poi silenzio.
live-card-lamp-tile-bad = Non si ferma mai: la griglia continua a ripartire, perché qualcosa continua a cambiare la vista o le impostazioni.

live-card-lamp-ssaa-what = Un campione di supersampling è stato aggiunto all’immagine assestata.
live-card-lamp-ssaa-good = Una serie di lampeggi dopo un assestamento, poi spenta.
live-card-lamp-ssaa-bad = Ricomincia di continuo: l’assestamento continua a essere disturbato.

live-card-lamp-palette-what = La normalizzazione dei colori ha recepito un nuovo intervallo di fuga, così i colori seguono lo zoom.
live-card-lamp-palette-good = Sfarfalla durante lo zoom.
live-card-lamp-palette-bad = Spenta durante lo zoom: l’intervallo della tavolozza è congelato, e i colori diventano piatti e poco contrastati.

live-card-lamp-mode-what = L’aritmetica è cambiata: diretta, df32 o floatexp.
live-card-lamp-mode-good = Un lampeggio a un confine come 2^93.
live-card-lamp-mode-bad = Lampeggia avanti e indietro: la vista oscilla a cavallo di un confine.

## The Render client window (File ▸ Render client…): lending this machine's GPU to a render farm

farm-client-title = Client di rendering
farm-client-intro = Renderizzare fotogrammi per una render farm. Il controller — Strumenti ▸ Renderizza sulla farm… su un’altra macchina — mostra il suo indirizzo e la chiave della farm. Questa macchina non apre alcuna porta: è lei a connettersi al controller.
# A field label, and the label of the status row that names the controller.
farm-client-controller = Controller
farm-client-find = Trova
farm-client-find-hover = Cercare i controller su questa rete (un controller risponde mentre la sua farm è in ascolto)
farm-client-key = Chiave della farm
# A checkbox beside the masked farm key: show it in clear.
farm-client-show-key = Mostra
# A field label (the name this machine goes by), and the label of the status row about this machine.
farm-client-machine = Questa macchina
farm-client-machine-hover = Il nome con cui il controller mostra questa macchina.
farm-client-share = Unità condivisa
# Greyed hint inside the empty field; the path is an example.
farm-client-share-hint = facoltativo — es. \\fileserver\share
farm-client-share-hover = Il percorso di questa macchina verso l’unità condivisa usata dal controller (modalità condivisione): i fotogrammi vengono scritti lì invece di essere inviati tramite la connessione. Lasciare vuoto per inviare ogni fotogramma.
farm-client-gpu = Scheda grafica
# The label of the row with the idle option.
farm-client-when = Quando
# A checkbox followed by a number of minutes ("Only when idle for [5] min").
farm-client-when-idle = Solo se inattiva da
farm-client-when-idle-hover = Accettare lavoro solo quando nessuno ha toccato la tastiera o il mouse di questa macchina per questo tempo, oppure mentre il suo schermo è bloccato. Quando qualcuno inizia a usarla, il fotogramma in corso si ferma subito e torna alla farm.
# A collapsible section: the largest job this machine accepts.
farm-client-limits = Limiti
farm-client-limits-note = Un lavoro che chiede più di questi valori viene rifiutato, mai ridotto.
farm-client-limit-size = Fotogramma più grande
farm-client-limit-ss = Supersampling massimo
farm-client-limit-iter = Iterazioni massime

## Render client: the graphics-card choice

farm-client-gpu-system = Scelta del sistema
farm-client-gpu-every = Tutte le schede grafiche
# $count: this machine's graphics cards.
farm-client-gpu-every-n = Tutte le schede grafiche ({ $count })
# $count: this machine's graphics cards.
farm-client-gpu-every-each = Tutte le schede grafiche ({ $count }) — una sessione ciascuna
# A card the window cannot name. $number: its number (or numbers) as --list-adapters gives it.
farm-client-gpu-adapter = Adattatore { $number }
farm-client-gpu-finding = Ricerca delle schede grafiche di questa macchina…
# "NAME · GPU 1" is how such a machine is named in the controller's list: keep it as it is.
farm-client-gpu-hover = Quale scheda grafica renderizza i fotogrammi della farm. Con tutte le schede, ciascuna si unisce alla farm come macchina a sé — «NAME · GPU 1», «NAME · GPU 2» — e Pausa e Disconnetti agiscono su tutte.

## Render client: looking for controllers on the network (the Find button)

farm-client-finding = Ricerca dei controller su questa rete…
# $port: the UDP port number.
farm-client-found-none = Nessun controller ha risposto. Un controller risponde mentre la sua farm è in ascolto; il suo firewall deve lasciar entrare UDP { $port }.
farm-client-found-pick = Su questa rete — fai clic su uno per usarlo:
# $identity: the controller's identity fingerprint.
farm-client-found-hover = Identità { $identity } — usare questo controller (serve comunque la sua chiave della farm)
# After a warning sign. $version and $git: this app's version and build.
farm-client-found-other-build = Una build diversa da questa ({ $version } { $git }): rifiuterebbe questa macchina.

## Render client: what is wrong with what was typed, or with starting the client

farm-client-bad-address = Inserisci l’indirizzo del controller come HOST:PORT, come lo mostra la sua finestra (per esempio 192.168.1.20:46733).
# $why: what is wrong with the key (in English).
farm-client-bad-key = La chiave della farm: { $why }
# $why: what is wrong with the name (in English).
farm-client-bad-name = Il nome: { $why }
farm-client-no-config = Non c’è una cartella di configurazione in cui conservare la chiave della farm.
# $why: the system's reason.
farm-client-start-failed = Impossibile avviare il client di rendering: { $why }

## Render client: the state, in the status heading

farm-client-phase-connecting = Connessione…
farm-client-phase-checking = Autoverifica in corso…
farm-client-phase-idle = Connesso — in attesa di lavoro
farm-client-phase-rendering = Rendering in corso
farm-client-phase-in-use = In attesa — qualcuno sta usando questa macchina
farm-client-phase-paused = In pausa
farm-client-phase-retrying = Controller non raggiungibile — nuovo tentativo
# $seconds: the wait before the next try.
farm-client-phase-retrying-in = Controller non raggiungibile — nuovo tentativo tra { $seconds } s
farm-client-phase-disconnected = Disconnesso
farm-client-phase-ended = Terminato

## Render client: the line under the status heading

# $address: the controller's address (HOST:PORT).
farm-client-detail-connecting = Connessione a { $address }…
# $controller: the controller's name.
farm-client-detail-checking = Connesso a «{ $controller }» — autoverifica in corso
# $controller: the controller's name.
farm-client-detail-approval = Connesso a «{ $controller }» — in attesa che il suo utente approvi questa macchina
# $controller: the controller's name.
farm-client-detail-idle = Connesso a «{ $controller }» — inattivo
# $why: the check that failed and why (in English).
farm-client-detail-check-failed = L’autoverifica non è riuscita — { $why }
# $why: the controller's reason (in English).
farm-client-detail-refused = Rifiutato dal controller: { $why }
# $why: the controller's reason (in English).
farm-client-detail-closed = Il controller ha chiuso la connessione: { $why }
# $why: the controller's reason (in English).
farm-client-detail-ended = Il controller ha terminato la connessione: { $why }
# $job: the job's name.
farm-client-detail-bundle = Lavoro «{ $job }» — ricezione del pacchetto
# $job: the job's name.
farm-client-detail-job-waiting = Lavoro «{ $job }» — in attesa di fotogrammi da renderizzare
# $why: the limit the job is over (in English).
farm-client-detail-job-refused = Lavoro rifiutato: { $why }
farm-client-detail-job-over = Il lavoro è terminato
# $first and $last: frame numbers.
farm-client-detail-rendering = Rendering dei fotogrammi { $first }–{ $last }
# $time: how long nobody must use the machine ("90 s", "5 min").
farm-client-detail-in-use = In attesa — qualcuno sta usando questa macchina; il lavoro riprende quando nessuno l’avrà usata per { $time }
farm-client-detail-paused = In pausa — il controller è stato avvisato
farm-client-detail-pausing = Messa in pausa — completamento del fotogramma in corso
farm-client-detail-cancelled = Fotogramma in corso annullato; in pausa
# A machine with several graphics cards. $rendering: the cards rendering now; $total: all its cards (two or more).
farm-client-rendering-on-cards = Rendering su { $rendering } di { $total } schede grafiche

## Render client: the status rows

# The value of the Controller row. $name: the controller's name; $address: HOST:PORT; $identity: its identity fingerprint.
farm-client-row-controller-value = «{ $name }» · { $address } · identità { $identity }
# The value of the This machine row. $name: this machine's name; $identity: its identity fingerprint.
farm-client-row-machine-value = «{ $name }» · identità { $identity }
farm-client-row-gpu = GPU
# One graphics card's row. $slot: the card's number.
farm-client-row-gpu-n = GPU { $slot }
farm-client-row-job = Lavoro
# The frames handed to this machine.
farm-client-row-run = Serie
farm-client-row-sent = Inviati
# The Run row, in parts joined with " · ": the run, the frame in progress, its time. $first and $last: frame numbers.
farm-client-run-frames = fotogrammi { $first }–{ $last }
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-client-run-frame = fotogramma { $frame } ({ $index } di { $count })
# $time: the time spent on it so far ("2.1 s").
farm-client-run-frame-time = { $time } su questo fotogramma
# One graphics card's run. $frame: the frame in progress; $first and $last: the run's frames.
farm-client-session-frame = fotogramma { $frame } di { $first }–{ $last }
farm-client-session-no-card = scheda grafica non ancora nota
# The Sent row. $count: frames sent; $time: the mean time per frame ("2.1 s").
farm-client-sent =
    { $count ->
        [one] { $count } fotogramma · { $time } a fotogramma
       *[other] { $count } fotogrammi · { $time } a fotogramma
    }
# The Sent row of a machine with several cards. $count: frames sent; $split: each card's share ("812 + 422"); $time: the mean time per frame.
farm-client-sent-split =
    { $count ->
        [one] { $count } fotogramma ({ $split }) · { $time } a fotogramma
       *[other] { $count } fotogrammi ({ $split }) · { $time } a fotogramma
    }
# After a warning sign. $note: how the GPUs differ, and both GPUs (in English).
farm-client-gpu-note = Questa macchina ha { $note }. I suoi fotogrammi differiranno leggermente da quelli del controller, cosa che può notarsi in una sosta.
# A collapsible section: the self-check's results.
farm-client-self-check = Autoverifica
farm-client-thumb-hover = L’ultimo fotogramma inviato da questa macchina
farm-client-resume = Riprendi
farm-client-resume-hover = Accettare di nuovo lavoro
farm-client-pause = Pausa
farm-client-pause-hover = Completare il fotogramma in corso, poi non accettare altro lavoro. Il controller passa i fotogrammi di questa macchina alle altre.
farm-client-cancel-frame = Annulla fotogramma
farm-client-cancel-frame-hover = Fermare subito il fotogramma in corso e mettere in pausa; il controller lo renderizza altrove.
farm-client-live-paused = Vista in tempo reale in pausa mentre questa macchina renderizza fotogrammi della farm.
farm-client-close = Chiudi
farm-client-close-active-hover = Chiudere la finestra; questa macchina resta nella farm (si riapre dal menu File)
farm-client-close-hover = Chiudere la finestra
farm-client-disconnect = Disconnetti
farm-client-disconnect-hover = Lasciare la farm: il fotogramma in corso viene restituito
farm-client-connect = Connetti
farm-client-connect-hover = Connettersi al controller e renderizzare fotogrammi per esso

## The Render on farm window (Tools ▸ Render on farm…): the controller of a render farm

farm-ctl-title = Rendering sulla farm
farm-ctl-out = Cartella di output
farm-ctl-browse = Sfoglia…
farm-ctl-out-note = I fotogrammi, e lo stato del lavoro in farm/: eseguendo di nuovo lo stesso lavoro lo si riprende.
farm-ctl-size = Dimensione
farm-ctl-size-custom = Personalizzata…
# The size list's choice when the size is typed in. $width and $height: pixels.
farm-ctl-size-custom-value = Personalizzata — { $width }×{ $height }
farm-ctl-fps = Fotogrammi / secondo
farm-ctl-ss = Supersampling
farm-ctl-port = Porta di ascolto
farm-ctl-port-hover = I client si connettono a questa porta. Consentila nel firewall per la tua rete locale.
farm-ctl-share = Unità condivisa
# Greyed hint inside the empty field.
farm-ctl-share-hint = facoltativo — il percorso di questa macchina verso di essa
farm-ctl-share-hover = Modalità condivisione: i client che hanno la stessa unità vi scrivono i loro fotogrammi invece di inviarli tramite la connessione, e questa macchina li verifica tutti lì. I client che non ce l’hanno inviano i propri.
# A label followed by a number of clients and two checkboxes.
farm-ctl-start-with = Inizia con
# The unit after the number in the box ("Start with [3] clients"). $count: that number.
farm-ctl-start-clients =
    { $count ->
        [one] client
       *[other] client
    }
farm-ctl-start-clients-hover = Il lavoro inizia quando questo numero di macchine si è connesso e ha superato l’autoverifica; altre possono unirsi in seguito.
farm-ctl-local = Renderizza anche su questa macchina
farm-ctl-local-hover = Questa macchina conta come un client.
farm-ctl-sharing = Condividi i riferimenti
farm-ctl-sharing-hover = L’orbita di riferimento di un fotogramma profondo può richiedere da minuti a ore per essere costruita. Se condivisa, viene costruita una volta e passata alle altre macchine. Un riferimento condiviso è un riferimento diverso, ugualmente valido: a profondità estreme una manciata di pixel può differire da un fotogramma che ha costruito il proprio.
farm-ctl-key = Chiave della farm
# A checkbox beside the shortened farm key: show all of it.
farm-ctl-show-key = Mostra
farm-ctl-copy-key = Copia chiave
farm-ctl-copy-key-hover = Incollarla nella finestra Client di rendering di ogni client
# In place of the farm key, when there is none.
farm-ctl-no-key = ancora nessuna
farm-ctl-make-key = Crea una chiave
farm-ctl-make-key-hover = I client ne hanno bisogno per unirsi; viene creata anche quando premi Renderizza
# What to do on the other machines. $address: this controller's HOST:PORT.
farm-ctl-steps = Su ogni client: File ▸ Client di rendering…, controller { $address }, e la chiave della farm.
# The same when this machine's address is not known. $port: the port number.
farm-ctl-steps-port = Su ogni client: File ▸ Client di rendering…, l’indirizzo di questa macchina e la porta { $port }, e la chiave della farm.
# The size of the job. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
farm-ctl-summary =
    { $count ->
        [one] { $count } fotogramma · { $seconds } s di tour · { $width }×{ $height } ss{ $ss }
       *[other] { $count } fotogrammi · { $seconds } s di tour · { $width }×{ $height } ss{ $ss }
    }
farm-ctl-strip-hover = Ogni fotogramma del lavoro, da sinistra a destra: completato, in rendering, ancora da fare, abbandonato.
farm-ctl-no-tour = Nessun tour caricato.
# $folder: the folder; $why: the system's reason.
farm-ctl-out-error = La cartella di output { $folder }: { $why }
# $why: the system's reason.
farm-ctl-start-failed = Impossibile avviare la farm: { $why }
# $code: the controller process's exit code, as the code gives it.
farm-ctl-ended-exit = Il controller è terminato (uscita { $code }).

## Render on farm: the state, in the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-phase-waiting = In attesa dei client di rendering ({ $connected } di { $needed } connessi)
farm-ctl-phase-preparing = Preparazione del lavoro
farm-ctl-phase-rendering = Rendering in corso
farm-ctl-phase-paused = In pausa
farm-ctl-phase-complete = Completato
farm-ctl-phase-stopped = Fermato
farm-ctl-phase-ended = Terminato

## Render on farm: the words after the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-detail-waiting = In attesa dei client di rendering: { $connected } di { $needed } connessi
farm-ctl-detail-preparing = Misura degli ancoraggi di normalizzazione su questa macchina, una volta per tutti i client
farm-ctl-detail-no-space = In attesa di spazio nella cartella di output
farm-ctl-detail-paused = In pausa: non viene distribuito nuovo lavoro
farm-ctl-detail-not-rendered = Terminato con fotogrammi non renderizzati — eseguilo di nuovo sulla stessa cartella per riprendere
farm-ctl-detail-stopped = Fermato — eseguilo di nuovo sulla stessa cartella per riprendere
# $code: the controller process's exit code.
farm-ctl-detail-ended = Terminato (uscita { $code })
farm-ctl-detail-finishing = Completamento

## Render on farm: progress

# On the progress bar. $done: frames done; $total: frames in the job.
farm-ctl-progress =
    { $total ->
        [one] { $done } / { $total } fotogramma
       *[other] { $done } / { $total } fotogrammi
    }
# Added to the progress bar. $rate: frames per second, with decimals.
farm-ctl-progress-rate = { $rate } fotogrammi/s
# Added to the progress bar. $time: the time still to go ("38m 10s").
farm-ctl-progress-eta = circa { $time } rimanenti
# Facts under the progress bar, joined with " · ". $received and $sent: kilobytes per second.
farm-ctl-io = in entrata { $received } KB/s · in uscita { $sent } KB/s
# $space: free space on the output drive ("412 GB").
farm-ctl-free = { $space } liberi
# $time: the time since the job started ("38m 10s").
farm-ctl-elapsed = { $time } trascorsi
# $count: frames given up on.
farm-ctl-failed = { $count } non renderizzati
# Reference orbits. $built: built for this job; $cached: found in the cache; $reused: reused between frames; $shared: passed between machines.
farm-ctl-refs = riferimenti: { $built } costruiti, { $cached } dalla cache, { $reused } riusati · { $shared } condivisi
# After a warning sign.
farm-ctl-storage-low = La cartella di output è quasi piena: non viene distribuito nuovo lavoro finché non c’è spazio.
# After a warning sign.
farm-ctl-gpus-differ = In questa farm le GPU sono diverse. I loro fotogrammi differiscono leggermente, cosa che può notarsi in una sosta.
# After a warning sign. $count: the GPU classes in the farm.
farm-ctl-gpus-differ-classes = In questa farm le GPU sono diverse ({ $count } classi di GPU: GPU, driver o rendering di prova diversi). I loro fotogrammi differiscono leggermente, cosa che può notarsi in una sosta.
# $gpu: this machine's GPU, API and driver.
farm-ctl-this-gpu = Questa macchina: { $gpu }
# A machine whose GPU differs. $name: the machine; $note: how its GPU differs (in English); $adapter: its GPU.
farm-ctl-gpu-row = { $name }: { $note } — { $adapter }
# The same with its driver. $driver: the driver's name and version.
farm-ctl-gpu-row-driver = { $name }: { $note } — { $adapter }, driver { $driver }

## Render on farm: the table of machines

# Column headers (short: the table is narrow).
farm-ctl-col-machine = Macchina
farm-ctl-col-state = Stato
farm-ctl-col-gpu = GPU
farm-ctl-col-frames = Fotogrammi
farm-ctl-col-per-frame = A fotogramma
# How the machine delivers its frames (a speed, or "shared drive").
farm-ctl-col-link = Collegamento
# When the controller last heard from the machine.
farm-ctl-col-heard = Contatto
# In the machine's tooltip. $count: its frames that failed verification.
farm-ctl-strikes =
    { $count ->
        [one] { $count } fotogramma non ha superato la verifica
       *[other] { $count } fotogrammi non hanno superato la verifica
    }
# In the Link column: it delivers its frames through the shared drive.
farm-ctl-link-share = unità condivisa
# In the Heard column. $seconds: how long ago.
farm-ctl-heard-ago = { $seconds } s fa
farm-ctl-readmit = Riammetti
farm-ctl-readmit-hover = Lasciare che questa macchina si unisca di nuovo quando si riconnette
farm-ctl-remove = Rimuovi
farm-ctl-remove-hover = Togliere questa macchina dal lavoro; i suoi fotogrammi passano alle altre
# The GPU column's tooltip.
farm-ctl-probe-same = Il suo rendering di prova è identico a quello di questa macchina.
# $count: the pixels that differ.
farm-ctl-probe-differs =
    { $count ->
        [one] Il suo rendering di prova differisce da quello di questa macchina in { $count } pixel: i suoi fotogrammi differiranno leggermente da quelli di questa macchina.
       *[other] Il suo rendering di prova differisce da quello di questa macchina in { $count } pixel: i suoi fotogrammi differiranno leggermente da quelli di questa macchina.
    }
farm-ctl-probe-unknown = Il suo rendering di prova non è stato confrontato.
# $driver: the driver's name and version.
farm-ctl-gpu-driver = driver { $driver }
# After a warning sign. $note: how its GPU differs (in English: "a different GPU").
farm-ctl-gpu-note = { $note } rispetto a questa macchina

## Render on farm: a machine's state, in the State column

# $count: its frames that failed verification.
farm-ctl-state-removed-bad =
    { $count ->
        [one] rimossa: { $count } fotogramma errato
       *[other] rimossa: { $count } fotogrammi errati
    }
farm-ctl-state-removed = rimossa da te
farm-ctl-state-removing = rimozione — raccolta della sua diagnostica
farm-ctl-state-gone = scomparsa
farm-ctl-state-leaving = in uscita
farm-ctl-state-checking = verifica (autoverifica)
farm-ctl-state-paused = messa in pausa dal suo utente
farm-ctl-state-parked = parcheggiata: si è bloccata due volte — rientra quando si riconnette
farm-ctl-state-refused = ha rifiutato questo lavoro (i suoi limiti)
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-ctl-state-rendering = rendering { $frame } ({ $index }/{ $count })
farm-ctl-state-starting = avvio di una serie
farm-ctl-state-idle = inattiva
farm-ctl-state-admitted = ammessa — in attesa del lavoro

## Render on farm: the buttons

# A collapsible section: the controller's messages.
farm-ctl-events = Eventi
farm-ctl-resume = Riprendi
farm-ctl-pause = Pausa
farm-ctl-pause-hover = Non distribuire nuovo lavoro; i fotogrammi in corso vengono completati
farm-ctl-live-paused = Vista in tempo reale in pausa mentre questa macchina renderizza per la farm.
farm-ctl-close = Chiudi
farm-ctl-close-active-hover = Chiudere la finestra; il lavoro continua (si riapre dal menu Strumenti)
farm-ctl-close-hover = Chiudere la finestra
farm-ctl-copy-command = Copia comando
farm-ctl-copy-command-hover = Copiare la riga di comando equivalente
farm-ctl-stop = Ferma
# The Stop button once a stop has been asked for and the job has not ended after 10 seconds.
farm-ctl-stop-now = Ferma subito
farm-ctl-stop-hover = Terminare il lavoro con i fotogrammi che ha; eseguendo di nuovo lo stesso lavoro lo si riprende
farm-ctl-render = Renderizza
farm-ctl-render-hover = Iniziare ad ascoltare i client e renderizzare il tour su di essi

## The Render tour window (Tools ▸ Render tour…)

tour-render-title = Rendering del tour
tour-render-out = Cartella di output
tour-render-browse = Sfoglia…
tour-render-out-hover = Percorso completo in cui vengono scritti i fotogrammi.
tour-render-prefix = Prefisso dei fotogrammi
# <prefix> stands for the prefix typed in the field.
tour-render-prefix-hover = I fotogrammi vengono scritti come <prefisso>_00000.png
tour-render-size = Dimensione
tour-render-size-custom = Personalizzata…
# The size list's choice when the size is typed in. $width and $height: pixels.
tour-render-size-custom-value = Personalizzata — { $width }×{ $height }
tour-render-fps = Fotogrammi / secondo
tour-render-fps-hover = Meno di 1 fps è legittimo: campionare un lungo tuffo a 0,25 fps è il modo in cui si ispezionano le soste profonde di regressione senza renderizzare migliaia di fotogrammi.
tour-render-ss = Supersampling
tour-render-chapter = Capitolo
tour-render-whole = Tour intero
tour-render-chapter-hover = Il rendering di un solo capitolo mantiene la numerazione GLOBALE dei fotogrammi, così i suoi fotogrammi si reinseriscono direttamente in una sequenza completa.
tour-render-mp4 = Assembla mp4
tour-render-mp4-hover = Esegue ffmpeg alla fine (deve essere nel PATH). I fotogrammi PNG vengono conservati.
tour-render-overwrite = Sovrascrivi i fotogrammi esistenti
tour-render-overwrite-hover = Se disattivato, il rendering si ferma per chiedere, cosa che da qui non può fare.
tour-render-resume = Riprendi un rendering interrotto
tour-render-resume-hover = Conservare i fotogrammi già presenti nella cartella di output e renderizzare solo quelli mancanti. Il fotogramma più recente viene controllato per primo e scartato se è incompleto (il fotogramma su cui un rendering si interrompe è quello che stava scrivendo), tornando indietro finché non se ne trova uno valido; una cartella che contiene fotogrammi di dimensione diversa viene rifiutata anziché mescolata a questa sequenza.
# $count: this machine's graphics cards.
tour-render-all-gpus = Usa tutte le schede grafiche ({ $count })
tour-render-all-gpus-hover = Renderizzare su tutte le schede grafiche di questa macchina contemporaneamente, tramite una render farm su questo computer (--gpus all). I fotogrammi, i loro nomi, «Riprendi» e l’mp4 funzionano come con una sola scheda; viene renderizzato l’intero tour, nell’ordine della farm. Modelli di scheda diversi disegnano alcuni pixel in modo leggermente diverso: una sosta e una dissolvenza restano su una sola scheda.
tour-render-order = Ordine di rendering
tour-render-sequential = Sequenziale
tour-render-sequential-hover = Fotogramma 0, 1, 2, … in ordine (predefinito).
tour-render-progressive = Progressivo
tour-render-progressive-hover = Prima un’anteprima dell’intero tour, poi la rifinitura fino al pieno dettaglio: i fotogrammi chiave vengono renderizzati per primi, poi i maggiori intervalli temporali vengono bisecati finché non esiste ogni fotogramma. In entrambi i casi i fotogrammi finiscono ai loro indici corretti, quindi «Riprendi» e l’mp4 funzionano senza cambiamenti — puoi fermarti presto, dare un’occhiata al flip-book approssimativo, e ripartire con «Riprendi» in uno dei due ordini.
# The cost of the render. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
tour-render-summary =
    { $count ->
        [one] { $count } fotogramma · { $seconds } s di tour · { $width }×{ $height } ss{ $ss }
       *[other] { $count } fotogrammi · { $seconds } s di tour · { $width }×{ $height } ss{ $ss }
    }
# The disk space the frames will take (an estimate). $size: a size ("5.3 GB").
tour-render-disk = ≈{ $size } di fotogrammi
# $size: the frames' estimated size; $free: the free space on that drive ("6.0 GB").
tour-render-disk-free = ≈{ $size } di fotogrammi · { $free } liberi
# After a warning sign. $disk: the line above ("≈51.0 GB of frames · 6.0 GB free").
tour-render-disk-short = { $disk } — spazio insufficiente per l’intera sequenza
tour-render-separate = Renderizza lo script DAL DISCO in un processo separato, quindi le modifiche non salvate non sono incluse — e una perdita del dispositivo GPU lì non può trascinare con sé questa sessione.
# On the progress bar. $done: frames done; $total: frames to render.
tour-render-progress =
    { $total ->
        [one] { $done } / { $total } fotogramma
       *[other] { $done } / { $total } fotogrammi
    }
# Before the render's first progress line.
tour-render-starting = avvio…
tour-render-live-paused = Vista in tempo reale in pausa — il rendering ha la GPU tutta per sé, quindi finisce prima e misura con precisione i costi dei propri fotogrammi.
tour-render-cancel = Annulla
tour-render-cancel-hover = Chiudere senza renderizzare
tour-render-copy-command = Copia comando
tour-render-copy-command-hover = Copiare negli appunti la riga di comando equivalente
tour-render-stop = Ferma rendering
tour-render-render = Renderizza
tour-render-render-hover = Avviare il rendering della sequenza di fotogrammi
# $folder: the output folder.
tour-render-finished = Rendering completato → { $folder }
# $why: the render's own reason (in English).
tour-render-failed = Rendering non riuscito: { $why }
# $last: the render's last progress line (in English), or nothing.
tour-render-failed-or-stopped = Il rendering non è riuscito o è stato fermato. { $last }
# $why: the system's reason.
tour-render-no-exe = Impossibile trovare l’eseguibile: { $why }
# $why: the system's reason.
tour-render-start-failed = Impossibile avviare il rendering: { $why }

## The Custom formula dialog (Fractal ▸ Custom formula…)
## The formula language itself stays as it is in every language: function names (sin, cosh, conj…),
## operators, variables (z, c, p1), and the formulas. Keep them as written inside these messages.

# The window's title.
formula-dialog-title = Formula personalizzata
# A small line at the top of the window, above the formula.
formula-dialog-intro = Il passo, in espressioni in stile Fractint
# The two ways to show and edit the formula (top right of the window).
formula-dialog-textbook = Libro di testo
formula-dialog-textbook-hover = Mostrare la formula composta come in un libro di testo, e modificarla così
formula-dialog-text = Testo
formula-dialog-text-hover = Modificare la formula come testo
# The tooltip of the "LaTeX" button (the button's own text stays LaTeX).
formula-dialog-latex-hover = Copiare la formula come LaTeX, così come è composta qui
# The drop-down list of example formulas (the collection).
formula-dialog-examples = Esempi
# Help under the formula in Textbook mode. The characters / ^ ( are what the user types: keep them.
formula-dialog-help-textbook = Scrivi come nel testo: / fa una frazione del termine che lo precede, ^ un esponente, ( le parentesi. Le frecce si muovono nella formula; Tab va alla casella vuota successiva. Una riga in rosso non viene letta: cliccala per modificarla come testo.
# Help under the formula in Text mode. "; starts a comment": the ; is what the user types.
formula-dialog-help-text = Scrivi, oppure usa il tastierino — contiene ogni nome che il linguaggio delle formule conosce. I nomi si completano mentre scrivi (Tab). Le istruzioni sono separate da un a capo o da una virgola; ; inizia un commento.
# The syntax check, when the formula reads.
formula-dialog-reads = Letta correttamente.
# The syntax check in Textbook mode, when the formula has an empty box to fill in.
formula-dialog-empty-box = Riempi la casella vuota.
# The syntax check in Textbook mode. $line: the line number; $message: what is wrong (from the formula parser, in English).
formula-dialog-error-line = Riga { $line }: { $message }
# One function written with another notation's name, and the language's name for it: "ln to log".
# $from: the name as written (ln); $to: the formula language's name (log).
formula-respell-pair = { $from } in { $to }
# A button that rewrites them. $changes: one or more formula-respell-pair, joined with ", ".
formula-respell = Cambia { $changes }
formula-respell-hover = Scrivere queste funzioni come le chiama il linguaggio delle formule
# Shown when Apply finds a parameter field that is not a number. $name: the parameter (p1 … p5).
formula-param-not-numbers = { $name } non è una coppia di numeri
# The Apply button (also in the Formula library window).
formula-apply = Applica
formula-dialog-apply-hover = Compilare la formula e mostrarla
# Shown beside Apply while the formula is being compiled.
formula-dialog-compiling = Compilazione della formula per la GPU…
# The greyed hint in an empty name box (also in the Formula library window).
formula-name-hint = nome (facoltativo)
# The save button: "Update" when the library already has a formula of that name.
formula-dialog-update-in-library = Aggiorna nella libreria
formula-dialog-save-to-library = Salva nella libreria
formula-dialog-save-hover = Conservare questa formula e i suoi parametri nella libreria di formule
formula-dialog-save-disabled-hover = Si può salvare solo una formula letta correttamente
# Opens the Formula library window.
formula-dialog-library = Libreria…
formula-dialog-library-hover = Le formule salvate: applicarle, modificarle, importarle ed esportarle

## The Custom formula dialog's keypad
## The keys' captions are formula tokens and stay as they are; these are their tooltips.

# The keypad's third tab (the other two are "123" and "f(z)"). A short word.
formula-keypad-tab-names = nomi
# The tabs' tooltips.
formula-keypad-tab-basic-hint = Numeri, operatori e i nomi più usati
formula-keypad-tab-functions-hint = Funzioni
formula-keypad-tab-names-hint = Ogni variabile, parametro e costante, e la sintassi delle istruzioni
# The key z.
formula-key-z = L’iterata
# The key c.
formula-key-c = Il pixel (la costante di Julia in modalità Julia)
# The key p1 on the first tab.
formula-key-p1-below = Parametro 1 — impostato nei campi qui sotto
# The keys p1 … p5. $n: the parameter's number.
formula-key-param = Parametro { $n }
formula-key-multiply = Moltiplica
formula-key-divide = Dividi
# The key that types ^2.
formula-key-square-after = Quadrato (dopo un valore: z^2)
# The key that types ^.
formula-key-power = Potenza: esponente intero, reale o complesso
formula-key-square-root = Radice quadrata
# The key that types |…|: in this language it is the squared modulus, as in Fractint.
formula-key-squared-modulus = Modulo AL QUADRATO |z|² — come in Fractint
formula-key-add = Somma
formula-key-subtract = Sottrai (o cambia segno)
# The key that types =.
formula-key-assign = Assegna: z = …, o una variabile temporanea t = …
# The key that starts a new line (first tab).
formula-key-next-statement-comma = Istruzione successiva (funziona anche una virgola)
# The key "del".
formula-key-backspace = Elimina il carattere prima del cursore (o la selezione)
# The key e.
formula-key-euler = Numero di Eulero
# The key that types (0.5, 0.5).
formula-key-complex-constant = Una costante complessa: (reale, immaginaria)
# The key that types a comma.
formula-key-comma = Separa le istruzioni, o le parti di una costante complessa
formula-key-left = Cursore a sinistra
formula-key-right = Cursore a destra
# The function keys (the f(z) tab), one per function.
formula-key-sin = Seno
formula-key-cos = Coseno
formula-key-tan = Tangente
formula-key-cotan = Cotangente
formula-key-exp = Esponenziale e^z
formula-key-sinh = Seno iperbolico
formula-key-cosh = Coseno iperbolico
formula-key-tanh = Tangente iperbolica
formula-key-cotanh = Cotangente iperbolica
formula-key-log = Logaritmo naturale, ln (ramo principale)
formula-key-sqr = Quadrato: sqr(z) = z²
formula-key-sqrt = Radice quadrata (principale)
formula-key-recip = Reciproco 1/z
formula-key-conj = Complesso coniugato
formula-key-flip = Scambia parte reale e parte immaginaria
formula-key-abs = Valore assoluto di ENTRAMBE le parti: |x| + i|y|
formula-key-cabs = Modulo |z| (un numero reale)
formula-key-real = Parte reale
formula-key-imag = Parte immaginaria, come numero reale
# "fn" is Fractint's name for a function slot: keep it.
formula-key-ident = Identità (il segnaposto fn di Fractint)
# The key "pixel".
formula-key-pixel = Uguale a c (il nome di Fractint)
# The key "z =".
formula-key-set-z = Inizia un’istruzione che imposta z
# The key "t =".
formula-key-temporary = Una variabile temporanea: va bene qualsiasi nome nuovo (t, w, a2, …)
# The key "; …".
formula-key-comment = Un commento, fino alla fine della riga
# The key that starts a new line (names tab).
formula-key-next-statement = Istruzione successiva

## The formula's completion list (names offered as you type)

# Beside a name the formula itself assigns.
formula-completion-variable = Una variabile impostata da questa formula
# Beside a function typed with another notation's name.
# $what: what the function is, in English (from the formula parser: "The natural logarithm");
# $other: the other notation's name (ln); $ours: the formula language's name (log).
formula-completion-other-spelling = { $what } — qui { $other } si scrive { $ours }
# Under a list cut short. $count: the names not shown.
formula-completion-more = … altri { $count }
# The last line of the list. Tab, Enter and Esc are key names.
formula-completion-keys = Tab o Enter per completare, Esc per chiudere

## The Custom formula dialog's Textbook mode

# The tooltip of a line that does not read. $why: what is wrong (from the formula parser, in English, or formula-textbook-fractint-sections).
formula-textbook-unread-hover = Non viene letta: { $why }. Clic per modificarla come testo.
# Why a line with Fractint's sections is shown as text in Textbook mode.
formula-textbook-fractint-sections = Le sezioni di Fractint (il ':' di un’inizializzazione, i confronti, a = b = …) si modificano come testo

## The Formula library window (Fractal ▸ Formula library…)

# The window's title.
formula-library-title = Libreria di formule
# The window's two tabs. $count: the formulas in each.
formula-library-mine = Le mie formule ({ $count })
formula-library-mine-hover = Le formule che hai salvato o importato
formula-library-collection = Raccolta ({ $count })
formula-library-collection-hover = Formule fornite con Fractadyne, ciascuna con una vista da cui partire
# A small marker before the name of the formula the view shows now. A short word.
formula-library-showing = in vista
# The tooltip of Apply on a formula of the collection.
formula-library-apply-shipped-hover = Mostrarla, nella sua vista di partenza
# A row's Edit button (after a pencil icon).
formula-library-edit = Modifica
formula-library-edit-hover = Aprirla nella finestra Formula personalizzata, senza applicarla
# A collection row's button that copies the formula into "My formulas".
formula-library-copy-to-mine = Copia nelle mie
formula-library-copy-to-mine-hover = Aggiungerla alle tue formule, per conservarla o modificarla
formula-library-close = Chiudi
# Saves the formula the view shows, under the name typed beside it.
formula-library-add-current = Aggiungi formula corrente
formula-library-add-current-hover = Salvare la formula mostrata nella vista
# "Fractal > Custom formula…" is the menu path: write it as the menus are named.
formula-library-add-current-disabled-hover = Applica prima una formula personalizzata (Frattale > Formula personalizzata…)
formula-library-import = Importa…
formula-library-import-hover = Aggiungere le formule di un file di formule; qui non viene sostituito nulla
formula-library-export-all = Esporta tutte…
formula-library-export-all-hover = Scrivere tutte le formule qui presenti in un unico file, da condividere o conservare
# The filter box: its label, and the greyed hint inside it.
formula-library-filter = Filtro
formula-library-filter-hint = filtra per nome o testo
# Shown while the user has no saved formulas.
formula-library-empty = Ancora nessuna formula salvata. Salvane una dalla finestra Formula personalizzata, aggiungi quella mostrata nella vista, o importa un file (nostro, o un .frm di Fractint).
# The tooltip of Apply on one of the user's formulas: with a view saved, and without one.
formula-library-apply-view-hover = Mostrare questa formula, nella vista salvata con essa
formula-library-apply-hover = Mostrare questa formula
# The tooltip of a row's save icon.
formula-library-export-one-hover = Esportare questa formula in un file
# A row asks before deleting: the question, then its two buttons. "Delete" is also the delete icon's tooltip.
formula-library-delete-ask = Eliminarla?
formula-library-delete = Elimina
formula-library-keep = Mantieni
# Under a list cut short. $count: the formulas not listed.
formula-library-more = …e altre { $count }: scrivi nel filtro per trovarne una.
formula-library-no-match = Nessuna corrisponde al filtro.
# Under a saved formula this version cannot read. $why: what is wrong (from the formula parser, in English).
formula-library-unread = Non viene letta in questa versione: { $why }

## Formula library notices

# $why: the system's reason, in English.
formula-library-save-failed = Impossibile salvare la libreria di formule: { $why }
formula-library-nothing-to-save = Niente da salvare: la formula è vuota.
# $name: the formula's name in the library.
formula-library-updated = «{ $name }» aggiornata nella libreria di formule.
formula-library-saved = «{ $name }» salvata nella libreria di formule.
# $name: the formula's name; $why: what is wrong.
formula-library-apply-failed = Impossibile applicare «{ $name }»: { $why }
# Copying a formula of the collection into "My formulas". $name: its name; $as_named: the name it was given (the same, or "Name (2)" when the name was taken).
formula-library-copy-exists = «{ $name }» è già tra le tue formule.
formula-library-copied = «{ $name }» copiata nelle tue formule come «{ $as_named }».
# $name: the formula's name.
formula-library-deleted = «{ $name }» eliminata dalla libreria di formule.
# Why the library could not be saved (follows formula-library-save-failed).
formula-library-no-config-dir = nessuna cartella di configurazione
# At startup, when the library file cannot be read. $why: the reason, in English; $path: a file's full path.
formula-library-unreadable-kept = Impossibile leggere la libreria di formule ({ $why }). È stata conservata come { $path } ed è stata avviata una nuova libreria vuota.
formula-library-unreadable-stuck = Impossibile leggere la libreria di formule ({ $why }), né spostarla da parte: copia { $path } in un posto sicuro prima di salvare una formula, perché il salvataggio la sostituisce.

## Formula library: Import and Export
## A formula file's contents stay in English; this is what the windows and notices around them say.

# File-type names in the open and save dialogs (the extensions are added by the system).
formula-filter-formula-files = File di formule
formula-filter-fractadyne = Formule di Fractadyne
formula-filter-fractint = Formule di Fractint
formula-filter-all-files = Tutti i file
# Why a file cannot be imported (they follow formula-import-failed).
formula-import-too-large = decisamente troppo grande per essere un file di formule
formula-import-not-text = non è un file di testo
formula-import-no-formulas = non contiene formule
# $why: the TOML reader's message, in English.
formula-file-not-formulas = non è un file di formule ({ $why })
# $format: what the file says it is ("fractadyne-gradients").
formula-file-other-format = un file «{ $format }», non un file di formule
# "[[formula]]" is the file's own syntax: keep it.
formula-file-no-formulas = non contiene formule (un file di formule le elenca come tabelle [[formula]])
# $file: the file's name; $why: one of the reasons above, or the system's (in English).
formula-import-failed = Impossibile importare «{ $file }»: { $why }.
# After an import that added nothing. $file: the file's name.
formula-import-nothing-new = Niente di nuovo in «{ $file }».
# After an import. $count: the formulas added; $file: the file's name.
formula-import-added =
    { $count ->
        [one] Importata { $count } formula da «{ $file }».
       *[other] Importate { $count } formule da «{ $file }».
    }
# The same two, followed by what else happened. $notes: one or more of the formula-import-note-… messages below, joined with "; ".
formula-import-nothing-new-notes = Niente di nuovo in «{ $file }»: { $notes }.
formula-import-added-notes =
    { $count ->
        [one] Importata { $count } formula da «{ $file }»: { $notes }.
       *[other] Importate { $count } formule da «{ $file }»: { $notes }.
    }
# Notes (clauses, no final full stop). $count: the formulas already in the library, skipped.
formula-import-note-duplicates =
    { $count ->
        [one] { $count } formula era già nella libreria
       *[other] { $count } formule erano già nella libreria
    }
# One formula was renamed because its name was taken. $from: its name in the file; $to: the name it was given ("Name (2)").
formula-import-note-renamed-one = il nome «{ $from }» era già preso, quindi ora è «{ $to }»
# $count: the formulas renamed because their names were taken (2 or more).
formula-import-note-renamed = { $count } rinominate perché il nome era già preso
# $count: the entries skipped as empty or too long.
formula-import-note-skipped =
    { $count ->
        [one] { $count } voce era vuota o troppo lunga ed è stata saltata
       *[other] { $count } voci erano vuote o troppo lunghe e sono state saltate
    }
# A second sentence after importing a Fractint .frm file. $count: the entries that do not read; $why: the commonest reason (from the formula parser, in English).
formula-import-frm-unread =
    { $count ->
        [one] { $count } voce non viene letta in questa versione (il motivo più frequente: { $why }).
       *[other] { $count } voci non vengono lette in questa versione (il motivo più frequente: { $why }).
    }
# After exporting one formula. $name: the formula's name; $file: the file's name.
formula-export-one = «{ $name }» esportata in «{ $file }».
# After exporting several. $count: how many; $file: the file's name.
formula-export-many =
    { $count ->
        [one] Esportata { $count } formula in «{ $file }».
       *[other] Esportate { $count } formule in «{ $file }».
    }
# $file: the file's name; $why: the system's reason, in English.
formula-export-failed = Impossibile scrivere «{ $file }»: { $why }

## The control panel (right-hand side): header and section headings

# The panel's title, beside the button that hides it.
panel-title = Controlli
panel-hide-hover = Nascondere il pannello dei controlli
# The button that brings the hidden panel back (top right of the view).
panel-show-hover = Mostrare il pannello dei controlli
# Section headings.
panel-section-navigate = Navigazione
panel-section-coloring = Colorazione
panel-section-quality = Qualità
panel-section-effects = Effetti
panel-section-overlays = Sovrapposizioni
panel-section-advanced = Avanzate
# The last section: reference notes about the current fractal. $name: the fractal's name (Mandelbrot, Burning Ship…).
panel-section-about = Informazioni su { $name }
panel-section-performance = Prestazioni

## The control panel: Navigate
## Labels before a control sit in a column about 104 px wide (roughly 15 characters): a longer label pushes its control to the right.

panel-zoom-speed = Velocità zoom
panel-zoom-speed-hover = Velocità dello zoom continuo tenendo premuto Space (1× ≈ 2× ogni 1,5 s).
panel-snap-zoom = Aggancia zoom alle strutture
panel-snap-zoom-hover = Durante lo zoom avanti con la rotellina del mouse o con Space, attirare lo zoom verso un minibrot (o l’asse reale) vicino al cursore, così che scivoli sotto il cursore e lo zoom vi atterri sopra. Più la struttura è vicina, più forte è l’attrazione. Solo Mandelbrot, vista singola.
panel-click-zoom = Clic per zoom
panel-click-zoom-hover = Quando è attivo, un clic sinistro nella vista si tuffa del fattore indicato sotto (il clic destro torna indietro), ricentrando sul punto cliccato. Il trascinamento sposta ancora la vista; Shift+trascinamento / trascinamento destro fanno ancora lo zoom a rettangolo. Backspace annulla un clic. Nella vista doppia funziona su entrambi i pannelli, e mentre è attivo il c di Julia si fissa con Ctrl+clic. Tieni premuto Shift per far comparire una lente sotto il cursore che mostra la vista in cui ti porterebbe il clic, e fai clic tenendolo premuto per prendere quel punto; Shift+TRASCINAMENTO fa ancora lo zoom a rettangolo.
# The click-to-zoom factor, followed by buttons 2×, 4×, 10×…
panel-click-zoom-factor = Fattore
panel-click-zoom-snap = Aggancia al centro più vicino
panel-click-zoom-snap-hover = Dopo il salto, calcolare il nucleo del minibrot più vicino e posizionarsi esattamente su di esso, così da atterrare sulla struttura e non vicino. Lo zoom non cambia — l’ha scelto il clic. Solo Mandelbrot. Il calcolo gira in background: la vista salta subito e si corregge quando arriva la risposta, cosa che a profondità estreme può richiedere qualche secondo.

## The control panel: Coloring
## Labels before a control sit in a column about 104 px wide (roughly 15 characters).

panel-method = Metodo
panel-method-hover = Come i dati di fuga diventano colore. Media delle strisce / disuguaglianza triangolare / trappola orbitale / decomposizione rivelano la struttura dell’orbita; stima della distanza ombreggia in base alla vicinanza al bordo.
panel-stripe-density = Densità strisce
panel-stripe-tail = Solo coda (zoom profondo)
panel-stripe-tail-hover = Mediare il termine delle strisce sulle ultime N iterazioni di ogni orbita (una finestra esponenziale) invece che sull’intera orbita. A profondità estreme ogni orbita segue il riferimento per tutte le iterazioni tranne le ultime poche centinaia, quindi la media sull’orbita intera dà un solo colore per tutta la vista; la coda mantiene il contrasto. Le orbite più corte della finestra non cambiano. Anche --stripe-tail N per --render.
panel-tail-length = Lunghezza coda
# The unit after the tail length's number: short for "iterations" (the slider reads "64 it").
panel-tail-length-unit = it
panel-tail-length-hover = Lunghezza della finestra in iterazioni. Finestre corte (8–32) danno le classiche bande marcate a qualsiasi profondità; quelle più lunghe tornano a mediare verso un colore piatto, perché nella media il termine delle strisce di un’orbita caotica lunga si annulla.
panel-trap-shape = Forma trappola
panel-palette = Tavolozza
# The two colours of the Binary palette, each beside its colour swatch.
panel-in-set = Nell’insieme
panel-out-of-set = Fuori insieme
# The two colours of the Duotone palette, each beside its colour swatch.
panel-shadow = Ombra
panel-highlight = Luce
panel-cycle = Ciclo
panel-offset = Offset
panel-normalize-hover = A profondità estreme i conteggi di fuga coprono centinaia di migliaia di valori e un Ciclo fisso fa girare la tavolozza migliaia di volte tra pixel vicini — un campo denso corretto appare come rumore puntiforme. Questa opzione rimappa la tavolozza sull’intervallo di fuga misurato della vista (Ciclo imposta allora quante ripetizioni della tavolozza lo coprono) QUANDO la vista ha effettivamente aliasing — una vista uniforme resta invariata, perché rimapparla cambierebbe l’immagine per niente. Spuntare «Adatta sempre l’intervallo» qui sotto per rimappare comunque. Solo con il metodo Iterazione continua; corrisponde a --normalize.
# A checkbox indented under "Normalize deep colors" (also in the Color menu).
panel-fit-range = Adatta sempre l’intervallo
panel-fit-range-hover =
    Rimappare la tavolozza sull’intervallo di fuga misurato della vista anche quando la vista non ha aliasing — la casella qui sopra rimappa solo quando un campo denso apparirebbe altrimenti come puntinatura.

    Da usare quando si vuole che la tavolozza copra ciò che è sullo schermo piuttosto che correggere il rumore. Richiede comunque una misura, quindi non fa nulla per un attimo o due prima che la vista si assesti, e niente del tutto su una vista senza pixel fuggiti.
# A checkbox indented under "Always fit range" (also in the Color menu).
panel-log-scale = Scala colori logaritmica
panel-log-scale-hover =
    Distribuire la tavolozza secondo il LOGARITMO del valore di fuga anziché linearmente. In profondità i conteggi di fuga si affollano verso l’estremo alto, quindi una mappatura lineare spende la maggior parte della tavolozza su un sottile guscio vicino al bordo e appiattisce tutto il resto.

    Si applica alla mappatura normalizzata — quella che «Normalizza colori profondi» (o --normalize in un’esportazione) attiva. Un intervallo molto asimmetrico usa comunque la mappatura logaritmica, che questa casella sia spuntata o no.
# The same checkbox's tooltip while it is greyed out (normalization is off).
panel-log-scale-off-hover =
    Richiede la normalizzazione: questa opzione sceglie COME la tavolozza viene mappata sull’intervallo di fuga misurato, quindi con «Normalizza colori profondi» disattivato (e nessuna esportazione --normalize) non c’è alcuna mappatura da cambiare e non modificherebbe un solo pixel.

    Attiva «Normalizza colori profondi» per usarla.
# The palette animation: its kind, then its speed.
panel-animate = Anima
panel-anim-speed = Velocità
panel-anim-speed-hover = Velocità del ciclo: cicli di offset del colore al secondo, oppure (Gradienti casuali) cambi di gradiente al secondo.
panel-shuffle-gradient = Mescola gradiente

## The control panel: Quality

panel-auto-iter = Iterazioni automatiche con lo zoom
# The iteration-count label while auto-scaling is on (the count is then a base the zoom raises).
panel-iterations-base = Iterazioni (base)
panel-iterations = Iterazioni
panel-iterations-hover = Numero di iterazioni di base. Con le iterazioni automatiche attive, il numero effettivo cresce con la profondità di zoom. Durante il movimento, l’anteprima limita le iterazioni a un valore basso (50.000) per restare reattiva, poi si affina al numero completo quando la vista si assesta — così i bordi profondi appaiono lisci durante il movimento e si risolvono quando ti fermi.
# A warning under the iteration count. $count: the iterations the settled preview renders.
panel-res-limited = A questa profondità l’anteprima assestata renderizza { $count } iterazioni a risoluzione ridotta per restare reattiva. Esporta per avere i dettagli a piena risoluzione.
panel-anti-alias = Anti-aliasing
# Anti-aliasing off (the other choices are 2×, 3×, 4×, 8×).
panel-anti-alias-off = No
panel-anti-alias-hover = Supersampling per le immagini ferme (applicato quando la vista si assesta). Valori più alti domano la fine «polvere» esterna a costo del tempo di rendering.

## The control panel: Effects

panel-relief-lighting = Illuminazione a rilievo 3D
panel-relief-lighting-hover = Ombreggiare la superficie usando la pendenza della stima della distanza — un aspetto in rilievo e illuminato. (Famiglie olomorfe: Mandelbrot / Multibrot.)
panel-light-angle = Angolo luce
panel-relief = Rilievo
panel-relief-hover = Più basso = rilievo più netto; più alto = più morbido/piatto.
panel-rotate-light = Ruota luce
panel-rotate-light-hover = Far ruotare la direzione della luce nel tempo (usa il cursore Velocità).
panel-distance-glow = Bagliore distanza
panel-distance-glow-hover = Fasce di contorno luminose della stima della distanza che si infittiscono in filamenti incandescenti vicino al bordo. (Famiglie olomorfe.)
panel-glow = Bagliore
panel-band-width = Larghezza banda
panel-band-width-hover = Spaziatura dei contorni di distanza (ottave per banda).
panel-animate-glow = Anima bagliore
panel-animate-glow-hover = Far scorrere le fasce del bagliore nel tempo (usa il cursore Velocità).

## The control panel: Overlays

panel-orbits-hover = Disegnare il percorso delle iterazioni del punto sotto il cursore.
panel-orbit-anim = Anima (punto in corsa)
panel-orbit-anim-hover = Lanciare un punto dai colori ciclici in corsa lungo l’orbita.
panel-orbit-speed = Velocità orbita
panel-minimap-hover = Una piccola panoramica della vista iniziale con un indicatore «sei qui» e la profondità di zoom. Cliccarla per saltare a una regione.

## The control panel: Advanced
## Labels before a control sit in a column about 104 px wide; most of these are already wider in English.

# Small headings of the section's two groups.
panel-accelerators = Acceleratori
panel-performance-tuning = Ottimizzazione prestazioni
panel-bla = Accelerazione BLA (zoom profondo)
panel-bla-hover = Approssimazione bilineare: saltare iterazioni lungo tutta l’orbita a profondità estreme (Mandelbrot floatexp, ≥1e28×) — rendering GPU ~5× più veloce, output identico (verificato dall’autotest). Attiva per impostazione predefinita; disattivarla per confrontare o in caso di artefatti.
panel-series-approx = Approssimazione in serie
panel-series-approx-hover = Inizializzare la perturbazione da un polinomio per saltare le prime iterazioni, dove la BLA non è attiva (profondità df32, Multibrot, BLA disattivata). Output identico; disattivarla per confrontare.
panel-glitch-correct = Correzione glitch (esportazione)
panel-glitch-correct-hover = Correzione multi-riferimento dei glitch per le immagini esportate: rileva i glitch di perturbazione e renderizza di nuovo quei pixel rispetto a riferimenti aggiuntivi finché non sono puliti. Attiva per impostazione predefinita. Esportazioni fino a ~32 MP (colorazione non ausiliaria); le immagini più grandi e la vista in tempo reale usano il percorso semplice. Mai usata per le viste Mandelbrot o Multibrot (fuori dalla modalità Julia): verificata con un calcolo esatto, lì non correggeva nulla e costava fino a 5 volte il tempo di esportazione.
panel-dive-limit = Limite tuffo auto-zoom
panel-dive-limit-hover = Profondità a cui l’auto-zoom (tasto A) si ferma. Fino a ~1e271× scivola in modo fluido; più in profondità passa a un tuffo a scatti, per gradini, per raggiungere rapidamente profondità estreme.
# Followed by two choices, Speed and Quality.
panel-autopilot-priority = Priorità auto-zoom
panel-priority-speed-hover = Zoomare alla velocità impostata. Lo schermo si aggiorna appena è pronto un fotogramma con dettagli, alla risoluzione che permette di continuare ad aggiornare (sotto la soglia di «Risoluzione min. in movimento» se lo zoom supera il rendering); tra un aggiornamento e l’altro l’ultimo fotogramma segue lo zoom e si ammorbidisce.
panel-priority-quality-hover = Ogni aggiornamento è un fotogramma completamente risolto a piena risoluzione, e lo zoom rallenta quanto serve perché il fotogramma sullo schermo non venga mai ingrandito troppo prima che arrivi il successivo. Un tuffo più lento, nitido in ogni momento.
# Followed by two choices, Detail and Misiurewicz point.
panel-autopilot-target = Bersaglio auto-zoom
panel-target-detail-hover = Tuffarsi verso la struttura più ricca sul bordo del frattale nella vista, scegliendo di nuovo il bersaglio man mano che il tuffo procede.
panel-target-misiurewicz-hover = Trovare il punto di Misiurewicz più vicino (il centro di una spirale la cui struttura si ripete all’infinito) e tuffarsi esattamente al suo interno. Il tuffo prima resta fermo mentre il punto viene calcolato fino alla profondità del limite di tuffo — un attimo alle profondità ordinarie, più a lungo oltre 1e1000×. Solo insieme di Mandelbrot.
panel-work-budget = Budget rendering in tempo reale
panel-work-budget-hover = Dettaglio contro velocità per la vista in tempo reale nello zoom profondo. Valori più alti renderizzano a una risoluzione più piena (più nitida — meno dell’aspetto «morbido» ingrandito) ma abbassano la frequenza dei fotogrammi, e valori molto alti rischiano un breve blocco della GPU sui fotogrammi più pesanti. Le esportazioni sono sempre a piena risoluzione, indipendentemente da questo.
panel-min-motion-res = Risoluzione min. in movimento
panel-min-motion-res-hover = La risoluzione più bassa a cui la vista in tempo reale può scendere durante lo zoom o lo spostamento continui nello zoom profondo. Valori più alti impediscono che un tuffo veloce diventi a blocchi/pixelato (più nitido) a costo della frequenza dei fotogrammi — al 100% gli aggiornamenti dei dettagli nei tuffi profondi devono essere renderizzati a costo pieno, il che può rendere lo zoom profondo visibilmente a scatti. Se un tuffo profondo sembra scattoso, abbassare questo valore. Predefinito 30%; non influisce sull’immagine assestata né sulle esportazioni.
panel-finish-sound = Suono a fine rendering
panel-finish-sound-hover = Suonare la melodia di completamento di FRACTINT quando un’esportazione o il rendering di un tour finisce: tre note ascendenti (1047, 1109, 1175 Hz per 100 ms ciascuna), esattamente come le codificava l’originale DOS — sintetizzate come un’onda quadra a banda limitata e filtrate passa-alto per suonare come il piccolo cono dello speaker del PC che le suonava.
panel-prefer-detail = Preferisci il dettaglio durante lo zoom
panel-prefer-detail-hover = Durante lo zoom o lo spostamento, continuare a mostrare l’ultimo fotogramma completamente dettagliato — scalato e spostato per seguire il movimento — invece di renderizzare di nuovo a qualità ridotta a ogni fotogramma. La vista viene renderizzata per intero quando il movimento si ferma. Se disattivato, il movimento viene renderizzato in tempo reale a risoluzione ridotta (più fluido, più grossolano). Le viste poco profonde (modalità diretta) vengono sempre renderizzate in tempo reale — in ogni caso costano poco e sono nitide a ogni fotogramma.

## The control panel: About (the current fractal)

# A link to the fractal's reference page, followed by an arrow.
panel-about-reference = Pagina di riferimento

## The control panel: Advanced ▸ Second graphics card

# A label before a drop-down (wider than the panel's label column already).
gpu2-label = Seconda scheda grafica
gpu2-off = Disattivata
# Another device on the same graphics card as the window (the default).
gpu2-same-card = Stessa scheda
# A card the list does not name. $number: the card's number.
gpu2-card-n = Scheda { $number }
gpu2-finding = Ricerca delle schede grafiche di questa macchina…
gpu2-none = Questa macchina non ha altre schede grafiche.
gpu2-hover =
    Un altro dispositivo grafico renderizza fotogrammi di una vista profonda mentre si muove, e parte dei campioni aggiuntivi che la rendono più nitida dopo che si ferma, così l’immagine tiene il passo con uno zoom e raggiunge prima la qualità finale. La scheda che disegna la finestra continua a disegnare tutto.

    Un’altra scheda aggiunge la propria velocità. Due modelli di scheda diversi disegnano alcuni pixel in modo leggermente diverso; se la seconda scheda si guasta, la vista continua con una sola.

    «Stessa scheda» (il valore predefinito) apre un secondo dispositivo su questa scheda. Non aggiunge velocità ma renderizza senza aspettare i fotogrammi dello schermo. Condivide la scheda: un guasto che resetta la scheda chiude l’app, come fa un guasto del rendering della finestra stessa, e all’avvio successivo viene proposto di disattivarlo.

## The view (canvas)

# A notice after Ctrl+click pins the Julia c at a deep zoom.
canvas-julia-pin-quantized = c di Julia fissato — ma è tenuto in doppia precisione, quindi a questa profondità l’intero pannello si arrotonda a un unico valore e il punto fissato cade al centro della vista. Rimpicciolisci per mirarlo.
# A small caption in the top-left corner of the view while the orbit overlay is fitted to the view (its "Normalize (fit to view)" option).
canvas-orbit-normalized = orbita (normalizzata)
# The minimap's tooltip.
canvas-minimap-hover = Panoramica / sei qui. Trascina per spostarti allo zoom corrente, scorri per zoomare avanti e indietro, fai clic per saltare a quella regione (zoom iniziale).

## The timestamp overlay (View ▸ Show timestamp)

# Under the large elapsed-time clock. $frame: the frame number (as the log writes it, without grouping).
overlay-timestamp-frame = fotogramma { $frame }

## File ▸ Settings ▸ Reference cache…

cache-title = Cache dei riferimenti
cache-intro = A profondità estreme la parte lenta di una vista è la sua orbita di riferimento — fino a un’ora di aritmetica a precisione arbitraria. Fractadyne conserva qui le orbite completate, così tornare in una posizione già visitata, o zoomare da qualche parte lì vicino, richiede invece pochi secondi. Un’orbita occupa circa 4 MB.
cache-off = Disattivata per questa esecuzione (un’invocazione di attività, o --no-orbit-cache). Non viene letto né scritto nulla fino al prossimo avvio normale.
# Small headings of the window's parts.
cache-location = Percorso
cache-usage = Utilizzo
cache-limit = Limite
cache-no-config = (nessuna cartella di configurazione disponibile)
cache-open-folder = Apri cartella
# The text on the usage bar. $used and $limit: sizes such as "1.2 GB"; $count: the orbits kept.
cache-usage-bar =
    { $count ->
        [one] { $used } di { $limit } — un’orbita
       *[other] { $used } di { $limit } — { $count } orbite
    }
cache-limit-hover = Quando la cache supera questo limite, vengono rimosse per prime le orbite che sono state MENO COSTOSE da costruire — un’orbita profonda che ha richiesto un’ora sopravvive a qualsiasi numero di orbite rapide, per quanto recentemente siano state usate.
# Beside the limit. $count: about how many orbits of the live view's length fit in it.
cache-limit-estimate =
    { $count ->
        [one] ≈ un’orbita al limite della vista in tempo reale
       *[other] ≈ { $count } orbite al limite della vista in tempo reale
    }
cache-kept = Conservate (prima le più preziose)
# Column headers of the list of kept orbits.
cache-col-iterations = iterazioni
cache-col-precision = precisione
cache-col-size = dimensione
cache-col-last-used = ultimo uso
# The precision column. $bits: the precision in bits.
cache-row-bits = { $bits } bit
# Under the list. $count: the orbits not listed.
cache-rows-more = … e altre { $count }
# When an orbit was last used (the "last used" column).
cache-ago-now = proprio ora
# $count: minutes.
cache-ago-minutes = { $count } min fa
# $count: hours.
cache-ago-hours = { $count } h fa
# $count: days.
cache-ago-days = { $count } g fa
# Asked before clearing. $count: the orbits; $size: their total size, such as "1.2 GB".
cache-confirm =
    { $count ->
        [one] Eliminare un’orbita ({ $size })? Ricostruirla costa tempo, non dati.
       *[other] Eliminare { $count } orbite ({ $size })? Ricostruirle costa tempo, non dati.
    }
cache-delete = Elimina
cache-cancel = Annulla
cache-clear = Svuota cache…
cache-clear-hover = Elimina tutte le orbite memorizzate. Chiede prima conferma.
cache-close = Chiudi
# $count: the orbits removed.
cache-cleared =
    { $count ->
        [one] Cache dei riferimenti svuotata — un’orbita rimossa.
       *[other] Cache dei riferimenti svuotata — { $count } orbite rimosse.
    }
# $why: the system's error message.
cache-clear-failed = Impossibile svuotare la cache dei riferimenti: { $why }

## Help ▸ Diagnostics…

diag-title = Diagnostica
diag-intro = Esegui un test per verificare che Fractadyne funzioni correttamente sul tuo hardware. I risultati vengono salvati sulla tua macchina e possono essere allegati a una segnalazione — nulla viene inviato da nessuna parte di propria iniziativa.
# $status: which arbitrary-precision library is in use, as the program reports it (in English).
diag-arithmetic = Aritmetica dello zoom profondo: { $status }
# The three tests' buttons, each followed by a sentence saying what it does.
diag-run-selftest = Esegui autotest
diag-run-uitest = Esegui test dell’interfaccia
diag-run-gputest = Esegui verifica aritmetica GPU
diag-blurb-selftest = Verifica la matematica e il rendering sulla tua GPU rispetto a risultati notoriamente corretti. Richiede circa 15 secondi.
diag-blurb-uitest = Percorre l’interfaccia e la vista in tempo reale, catturando screenshot a diverse profondità di zoom. Richiede un minuto o due e apre finestre durante l’esecuzione.
diag-blurb-gputest = Verifica l’aritmetica double-float dello shader rispetto a valori notoriamente corretti su ogni backend grafico offerto dalla tua macchina. Richiede pochi secondi, non apre finestre. Se segnala un errore, è una scoperta reale che vorremmo vedere — allegalo a una segnalazione.
# While a test runs. $count: the checks finished so far.
diag-checks-done =
    { $count ->
        [one] una verifica completata
       *[other] { $count } verifiche completate
    }
# A finished test's result. $test: the test's name (Self-test, UI test, GPU arithmetic check).
diag-verdict-captured = { $test } — risultato acquisito
diag-verdict-passed = { $test } — superato
diag-verdict-problems = { $test } — problemi segnalati
# Under the GPU arithmetic check's result.
diag-informational-note = Questo test non ha esito superato o non superato: registra come calcola la tua GPU, e un errore segnalato è una scoperta reale, non un guasto della tua macchina. In ogni caso, allegarlo a una segnalazione è la cosa più utile che puoi farne.
# Under a test's result when it reported problems.
diag-problems-note = Su hardware diverso dalla GPU di riferimento sono previste alcune differenze — in particolare nei confronti tra immagini. Allegare questo risultato a una segnalazione è la cosa più utile che puoi farne.
diag-open-results = Apri risultati
diag-attach = Allega a una segnalazione…
diag-attach-hover = Apre la finestra di segnalazione con questo risultato incluso, così la issue porta un test convalidato dalla macchina e non solo una descrizione
diag-console = Stampa la diagnostica sulla console
diag-console-hover = Mostrare da ora in poi le righe [fd-…] su stderr. Disattivato per impostazione predefinita quando Fractadyne viene avviato senza argomenti. Non cambia nulla di ciò che viene REGISTRATO — il file di log riceve sempre ogni riga — quindi conviene lasciarlo disattivato a meno di non stare guardando un terminale. Vale solo per questa sessione; usare l’opzione --console o FRACTADYNE_CONSOLE=1 per catturare anche l’avvio.
diag-close = Chiudi
# Errors starting a test. $why: the system's error message; $dir: a folder.
diag-error-no-exe = Impossibile trovare l’eseguibile: { $why }
diag-error-create-dir = Impossibile creare { $dir }: { $why }
diag-error-start = Impossibile avviare il test: { $why }

## Navigate ▸ Misiurewicz explorer…

misi-title = Esploratore di Misiurewicz
misi-mandelbrot-only = L’esplorazione dei punti di Misiurewicz funziona solo con Mandelbrot — cambia frattale per usarla.
# While the gallery is being built. $seconds: the time so far, such as "3".
misi-solving = Risoluzione delle famiglie di tipo piccolo ({ $seconds } s)…
# Above the gallery. $total: all points; $curated: the hand-picked ones; $found: the ones the sweep found.
misi-summary = { $total } punti — { $curated } selezionati a mano, { $found } trovati da una scansione di Newton con semi dei tipi piccoli (semipiano superiore; una griglia grossolana trova la maggior parte dei bacini, non dimostrabilmente tutti).
# The name of a point the sweep found. $kind: its type, such as "(3,1)"; $point: its coordinate, such as "-0.1011+0.9563i".
misi-entry-name = { $kind } in { $point }
# Under each thumbnail. $kind: the point's type, such as "(3,1)"; $octaves: the zoom period in octaves, such as "3.17"; $twist: the twist per repeat in degrees, such as "+12.5". "oct" is short for octaves.
misi-entry-readout = { $kind } · { $octaves } ott/ripetizione · { $twist }°
# Before the depth field.
misi-zoom-to = Zoom a
misi-cancel = Annulla
misi-solve-jump = Risolvi e salta
misi-solve-jump-hover = Risolvere con Newton questo punto esatto con la precisione richiesta dalla profondità chiesta (usando come seme la sua coordinata memorizzata), poi saltarci direttamente. Le richieste profonde richiedono più tempo — il calcolo è a precisione arbitraria.
misi-select-hint = Seleziona un punto, indica una profondità e salta — la riga del moltiplicatore sotto ogni miniatura mostra come apparirà un tuffo lì: quante ottave prima che la vista si ripeta, e quanto ruota a ogni ripetizione.
misi-cancelled = Calcolo annullato.
# $text: what was typed in the depth field.
misi-bad-zoom = Impossibile leggere «{ $text }» come ingrandimento (scrivi 1e50, 3.2e120, …).
misi-no-kp = Questa voce non ha una coppia (k,p) rilevata con cui risolvere.
misi-parse-failed = Impossibile interpretare la coordinata memorizzata.
# A notice after the jump. $name: the point's name; $zoom: the magnification, such as "1e50"; $seconds: how long the solve took, such as "1.2".
misi-zoomed = Zoom su { $name } — { $zoom }× ({ $seconds } s)
# Added after the notice above when the iteration count is too low. $have: the fixed iteration count; $typical: what the depth typically needs.
misi-iter-note = Le iterazioni sono fissate a { $have }; questa profondità di solito ne richiede ~{ $typical } — aumentale (o attiva le iterazioni automatiche) se la vista risulta piatta.
# $name: the point's name; $why: why the solve failed (a technical word).
misi-solve-missed = Il calcolo profondo non è arrivato su { $name }: { $why }. Il punto memorizzato è comunque esatto alla sua profondità — prova un obiettivo meno profondo.

## Life (Game of Life): the side panel

# Shown when the graphics card cannot run the Life view.
life-no-compute = Questa scheda grafica non può eseguire Life (nessun compute shader).
# Row label: the pattern list (pattern names stay in English).
life-pattern = Pattern
life-open = Apri…
life-open-hover = Aprire un file di pattern RLE, .cells o Life 1.05/1.06.
life-pattern-text = Testo del pattern…
life-pattern-text-hover = Incollare un pattern (RLE come lo fornisce LifeWiki, o testo semplice).
life-save = Salva…
life-save-hover = Salvare l’universo così com’è ora, in formato RLE.
# Row label: the rule list and the typed rule (B3/S23 notation).
life-rule = Regola
# Shown in the rule list when the rule in use is none of the library's.
life-rule-custom = personalizzata
life-apply = Applica
# Playback buttons (a pause or play symbol is shown before them).
life-pause = Pausa
life-play = Avvia
life-play-hover = Far evolvere l’universo (Enter).
life-step = Passo
life-step-hover = Avanzare di una generazione (N).
# Tooltip of the button beside Step, which reads "+16", "+1,024"…: that many generations.
life-stride-hover = Avanzare di questo numero di generazioni (Shift+N).
life-reset = Ripristina
life-reset-hover = Tornare al pattern com’era all’apertura (generazione 0).
life-speed = Velocità
# After the speed slider's number: generations a second. Short.
life-speed-unit = gen/s
# Row label: how many generations the "+n" button advances.
life-stride = Salto
# The panel's numbers.
life-generation = Generazione
life-population = Popolazione
life-tiles = Riquadri
# $count: the tiles stored; $capacity: how many the GPU pool holds.
life-tiles-of = { $count } di { $capacity }
life-tiles-hover = Riquadri memorizzati da 64×64 celle (4 KiB ciascuno) e capacità del pool della GPU.
# Row label: what a left-drag in the view does (Pan or Draw).
life-mouse = Mouse
life-pan = Sposta
life-pan-hover = Trascinare per spostare la vista.
life-draw = Disegna
# The Draw tool's tooltip; the toolbar's shows it after "Draw (D):", so it starts lowercase.
life-draw-hint = clic su una cella per invertirla; trascina per dipingere — un tratto che inizia su una cella morta disegna, uno che inizia su una cella viva cancella. Un universo in esecuzione si mette in pausa mentre disegni e riprende quando rilasci.
life-undo = Annulla
life-undo-hover = Tornare all’universo com’era prima dell’ultima modifica, riempimento, svuotamento o cambio di regola (Ctrl+Z durante il disegno).
life-grid = Linee della griglia
life-grid-hover = Bordi delle celle quando lo zoom è di 8 pixel per cella o più.
# A random fill: "Random" labels the density field, "seed" the seed field after it.
life-random = Casuale
life-density-hover = La frazione di celle che un riempimento rende vive.
life-seed = seme
life-fill-view = Riempi vista
# $cells: the largest side of the filled area, in cells (2,048).
life-fill-view-hover = Riempire a caso l’area visibile (fino a { $cells } × { $cells } celle attorno al suo centro).
life-clear = Svuota
life-clear-hover = Uccidere tutte le celle (Annulla le riporta in vita).

## Life: notices in the panel, and file dialogs

# $path: the file written.
life-saved = Salvato { $path }
# $error: why (from the system, in English).
life-save-failed = Salvataggio non riuscito: { $error }
# The universe stopped running. $error: why (in English).
life-stopped = Fermato: { $error }
# A pattern file named a rule that does not read. $error: why (in English); $rule: the rule kept, in B3/S23 notation.
life-rule-not-understood = La regola del pattern non è stata capita ({ $error }); mantenuta { $rule }
# $path: the file; $error: why (in English).
life-open-failed = Impossibile aprire { $path }: { $error }
# File-dialog filter names.
life-filter-patterns = Pattern di Life
life-filter-rle = Pattern codificato run-length

## Life: the toolbar (its buttons are icons; these are their tooltips)

life-toolbar-pause-hover = Mettere in pausa l’universo (Enter)
life-toolbar-play-hover = Far evolvere l’universo (Enter)
# $stride: the generations a Shift+click steps.
life-toolbar-step-hover = Avanzare di una generazione (N); Shift+clic o Shift+N avanza di { $stride }
life-toolbar-reset-hover = Tornare al pattern com’era all’apertura (generazione 0)
# $hint: the Draw tool's tooltip (life-draw-hint).
life-toolbar-draw-hover = Disegna (D): { $hint }

## Life: the Pattern text window

life-text-title = Testo del pattern
life-text-intro = Incolla un pattern: RLE (come lo forniscono LifeWiki e Golly), testo semplice .cells, o Life 1.05 / 1.06.
life-text-load = Carica
life-text-current = Corrente come RLE
life-text-current-hover = Riempire la casella con l’universo così come è stato caricato l’ultima volta.

## L-systems: the side panel

# Row label: the system list (system names stay in English).
lsys-system = Sistema
lsys-open = Apri…
lsys-open-hover = Aprire un file .l di Fractint o un sistema .lsys.
lsys-edit = Modifica…
lsys-edit-hover = Il testo del sistema: assioma, produzioni, angolo — modificarlo e applicarlo.
lsys-save = Salva…
lsys-save-hover = Salvare il sistema come file di testo .lsys.
# Tooltip of the "SVG…" button.
lsys-svg-hover = Esportare la vista come disegno SVG: le sue linee e forme come vettori, nei suoi colori.
lsys-follow = L’ordine segue lo zoom
lsys-follow-hover = L’ordine aumenta mentre ingrandisci, così i dettagli non finiscono mai.
lsys-follow-fixed-hover = Questo sistema non cresce di un fattore a ogni ordine, quindi viene disegnato a un ordine fisso.
lsys-order = Ordine
lsys-angle = Angolo
lsys-sweep-hover = Far variare l’angolo, avanti e indietro: la curva si trasforma mentre ruota
# A small button that puts back the system's own angle after the slider moved it.
lsys-system-angle = Angolo del sistema
lsys-seed = Seme
lsys-seed-hover = Ciò che seguono le scelte casuali: stesso seme, stessa pianta.
lsys-seed-new = Nuovo
lsys-seed-new-hover = Un’altra pianta: un nuovo seme.
lsys-line-width = Spessore linea
# Row label of a percentage slider: how much of the curve is drawn.
lsys-draw-on = Tracciato
lsys-draw-on-hover = Quanta parte della curva viene disegnata, nell’ordine in cui la disegna la tartaruga
lsys-draw-on-play-hover = La curva che si disegna da sola
lsys-colour-by = Colora per
# The colourings, in the "Colour by" list.
lsys-colour-position = posizione lungo la curva
lsys-colour-depth = profondità del ramo
lsys-colour-heading = direzione
# C, < and > are the system's commands that set the colour index: keep them.
lsys-colour-index = indice di colore (C, <, >)
lsys-colour-plain = un solo colore
# The panel's numbers.
lsys-growth = Crescita
lsys-growth-built = costruito come parola, al suo ordine
lsys-growth-built-hover = Parametrico o sensibile al contesto: la riscrittura di un modulo dipende dai suoi numeri o dai suoi vicini, quindi la parola viene costruita una generazione alla volta (fino a due milioni di moduli) e disegnata in doppia precisione — l’ordine non segue lo zoom.
# How much the picture grows from one order to the next. $factor: the factor (1.4142).
lsys-growth-factor = ×{ $factor } per ordine
lsys-growth-factor-period-2 = ×{ $factor } per ordine (gli ordini avanzano di 2)
lsys-growth-none = nessuna (ordine fisso)
lsys-drawn = Disegnato
# $drawn: what was drawn (lsys-drawn-segments, lsys-drawn-shapes or lsys-drawn-segments-shapes); $order: the order.
lsys-drawn-at-order = { $drawn } all’ordine { $order }
# The same, when the walk stopped at its segment budget.
lsys-drawn-at-order-budget = { $drawn } all’ordine { $order } (budget)
# $segments: the line segments drawn.
lsys-drawn-segments =
    { $segments ->
        [one] { $segments } segmento
       *[other] { $segments } segmenti
    }
# $shapes: the filled shapes drawn.
lsys-drawn-shapes =
    { $shapes ->
        [one] { $shapes } forma piena
       *[other] { $shapes } forme piene
    }
# Both counts. $segments: line segments; $shapes: filled shapes.
lsys-drawn-segments-shapes =
    { $segments ->
        [one] { $segments } segmento
       *[other] { $segments } segmenti
    }, { $shapes ->
        [one] { $shapes } forma piena
       *[other] { $shapes } forme piene
    }
# $ms: the walk's time in milliseconds (12.3).
lsys-drawn-hover = percorso in { $ms } ms; i sottoalberi fuori dalla vista vengono saltati, quelli più piccoli di un pixel disegnati come un unico segmento
# A word too long to build at the order asked for. $wanted: the order asked for; $built: the order drawn.
lsys-drawn-short = La parola dell’ordine { $wanted } supera i due milioni di moduli: disegnato all’ordine { $built }.
# $order: the deepest order the app draws.
lsys-deepest-order = All’ordine più profondo disegnato ({ $order }): zoomare oltre non aggiunge dettagli.

## L-systems: notices in the panel and the editor, and file dialogs

# $path: the file; $error: why (in English).
lsys-open-failed = Impossibile aprire { $path }: { $error }
# Why a file was not opened (shown after "Could not open …:").
lsys-file-too-large = più grande di 4 MiB
# Why a Fractint .l file was not opened: it holds no entries. `Name { … }` is the file's syntax: keep it.
lsys-no-entries = nessuna voce (un file .l contiene voci `Name {"{"} … {"}"}`)
# $path: the file written.
lsys-saved = Salvato { $path }
# $error: why (from the system, in English).
lsys-save-failed = Salvataggio non riuscito: { $error }
# After File ▸ Export SVG. $path: the file written.
lsys-svg-exported = Esportato { $path }
# $error: why (from the system, in English).
lsys-svg-failed = Esportazione non riuscita: { $error }
# File-dialog filter names: opening (any system file) and saving (one system).
lsys-filter-systems = L-system
lsys-filter-system = L-system

## L-systems: the toolbar

lsys-toolbar-lower-hover = Abbassare l’ordine (lo fissa)
lsys-toolbar-raise-hover = Alzare l’ordine (lo fissa)
# Between the − and + buttons, in a fixed-width font. $order: the order, padded to 3 characters so the label keeps its width.
lsys-toolbar-order = ordine { $order }
# A toggle: the order follows the zoom. Short.
lsys-toolbar-auto = auto
lsys-toolbar-auto-hover = L’ordine segue lo zoom
lsys-toolbar-edit-hover = Modificare il testo del sistema

## L-systems: the editor window, and a .l file's entries

lsys-editor-title = L-system
# The editor's help. The quoted examples ('X = word', 'seed n', 'define R 1.456'…), the commands and "The Algorithmic Beauty of Plants" (a book's title) are the system's syntax and names: keep them as they are.
lsys-editor-help = Una chiave per riga: angle (in gradi, o /n per una divisione del cerchio), axiom, le produzioni 'X = word', e facoltativamente heading, draw / move / variables, colour, order. Un simbolo con alternative scelte a caso ha una riga per ciascuna, con il suo peso: 'X (0.3) = word'; 'seed n' sceglie la pianta. In una parola, {"{"} {"}"} riempie il percorso della tartaruga e . aggiunge un vertice. Produzioni parametriche e sensibili al contesto (The Algorithmic Beauty of Plants): 'A(s) : s > 1 = F(s)[+A(s/2)]', 'b < a > c = b', con 'define R 1.456' e 'ignore +-F'; F(l) avanza di l, +(a) ruota di a gradi. Una voce .l di Fractint si può incollare così com’è.
lsys-editor-apply = Applica
lsys-editor-revert = Ripristina
lsys-editor-revert-hover = Il testo del sistema mostrato.
# The window listing a .l file's entries to pick from. $file: the file's name.
lsys-entries-title = { $file } — voci
# Tooltip of an entry that does not parse (its error is shown beside it, in English).
lsys-entry-unreadable-hover = Questa voce non viene letta.

## Export: opening a view or location (File ▸ Open view or location…)

# File-dialog filter names.
export-filter-view = Vista o posizione di Fractadyne
export-filter-image = Immagine (PNG / EXR)
export-filter-location = Posizione (.fdn / .kfr)
# $zoom: the magnification (1.2e148), followed by the magnification sign ×.
export-loaded-location = Posizione caricata @ { $zoom }×
# $zoom: the magnification; $note: what was noteworthy in the file (in English).
export-loaded-location-note = Caricata @ { $zoom }× — { $note }
# $path: the file.
export-not-location = { $path } non è una posizione di Fractadyne.
export-file-too-large = File troppo grande (non è una posizione .fdn?).
# $path: the file; $error: why (from the system, in English).
export-read-failed = Impossibile leggere { $path }: { $error }
# A Kalles Fraktaler .kfr file did not import. $error: why (in English).
export-import-failed = Importazione non riuscita: { $error }
# $path: an image whose embedded text is not a view.
export-not-view = { $path } ha metadati di Fractadyne, ma non una vista.
# $path: the image the view was read from.
export-loaded-view = Vista caricata da { $path }
# $path: the image; $note: what was noteworthy in it (in English).
export-loaded-view-note = Vista caricata da { $path } — { $note }
export-no-metadata = Quel file non contiene metadati di vista di Fractadyne incorporati.

## Export: notices after an export or a snapshot
## "File → Export image…" in them names the menu items (menu-file, menu-file-export).

# The export window's status while the image renders.
export-rendering = Rendering…
export-canceled = Esportazione annullata.
# $error: why (in English).
export-failed = Esportazione non riuscita: { $error }
# $size: the image's size in pixels (1920×1080); $path: the file written.
export-saved = Salvato { $size } → { $path }
# An L-system export whose walk stopped at its budget. $size: pixels; $segments: the budget (4,000,000); $path: the file.
export-saved-incomplete = Salvato { $size } (incompleto: oltre il budget di { $segments } segmenti — abbassa l’ordine) → { $path }
# A two-panel export written as two files. $path: the first of them.
export-saved-two = Salvati 2 file → { $path }
# $size: pixels; $path: the file written.
export-saved-corrected = Salvato { $size } (glitch corretti) → { $path }
# $path: the first of the two files.
export-saved-two-corrected = Salvati 2 file (glitch corretti) → { $path }
# Added after a finished export's notice. $duration: how long it took (8.3s, 1m 04.0s).
export-elapsed = (in { $duration })
export-preparing-deep = Preparazione dell’esportazione profonda — costruzione del riferimento (può richiedere un po’ di tempo)…
# $size: pixels (5120×4035); $ss: the supersampling factor; $iterations: the iteration count.
export-large = Esportazione grande ({ $size }, supersampling { $ss }×, { $iterations } iterazioni) — rendering in background; per esportazioni di queste dimensioni la correzione dei glitch viene saltata…
export-life-unavailable = L’esportazione come immagine di una vista Life non è ancora disponibile — usa uno screenshot, oppure File ▸ Salva pattern di Life.
export-already-running = È già in corso un’esportazione — File → Esporta immagine… ne mostra l’avanzamento.
# $size: the image's size in pixels.
export-snapshot-background = Istantanea: rendering di { $size } in background — File → Esporta immagine… ne mostra l’avanzamento e può annullarla.
export-no-gpu = GPU non disponibile
# A screen-capture snapshot. $size: pixels; $path: the file written.
export-snapshot-saved = Istantanea dello schermo salvata { $size } → { $path }
# $error: why (in English).
export-snapshot-failed = Istantanea non riuscita: { $error }

## The colour picker (the popup of a colour swatch)

# The third row's label: the colour as hexadecimal bytes. Short (about 6 characters).
picker-row-hex = Hex
picker-hex-hover = Clic per scrivere un valore esadecimale
picker-cancel = Annulla
picker-cancel-hover = Ripristinare il colore con cui si è aperto questo selettore
picker-ok = OK
picker-ok-hover = Mantenere questo colore
# Starts the screen eyedropper (an icon is shown before it).
picker-pick = Preleva
picker-pick-hover = Prendere un colore da qualsiasi punto dello schermo
# Tooltip of the disabled Pick button (in the picker and the gradient editor) on Linux and macOS.
picker-eyedropper-unsupported = Prelevare un colore da altri punti dello schermo richiede un’API di cattura dello schermo della piattaforma. Solo la build per Windows ne ha una; su Linux servirebbe una dipendenza X11, e Wayland richiede un portale con una propria richiesta di autorizzazione.

## The fractal families: what each one is (the info panel's "About" section and the pickers' hover text).
## The formulas themselves (z -> z^2 + c) are written in code and stay as they are.

fractal-mandelbrot-about = Il frattale a tempo di fuga per eccellenza: l’insieme dei c per cui l’orbita di 0 resta limitata. Il suo bordo è infinitamente intricato.
fractal-multibrot3-about = Un insieme Multibrot - la costruzione di Mandelbrot a una potenza superiore. La potenza d dà una simmetria rotazionale di ordine (d-1).
fractal-multibrot4-about = Multibrot alla potenza 4: simmetria di ordine tre, bulbi ampi.
fractal-multibrot5-about = Multibrot alla potenza 5: simmetria di ordine quattro.
fractal-tricorn-about = Il Tricorn (Mandelbar): coniuga z a ogni passo. Questa mappa anti-olomorfa produce una forma a tre punte.
fractal-burning-ship-about = I valori assoluti delle parti di z vengono presi prima di elevare al quadrato; il risultato ricorda una nave in fiamme.
fractal-celtic-about = Un parente del Burning Ship che prende il valore assoluto solo della parte reale di z^2, producendo motivi a nodo celtico / a cuore.
fractal-buffalo-about = Una variante con valori assoluti che li prende su entrambe le componenti di z^2.
fractal-phoenix-about = Il Phoenix usa anche l’iterata precedente, producendo filamenti simili a fiamme. Prova la sua forma Julia con la modalità Julia.
fractal-newton-about = L’iterazione di Newton per trovare le radici di z^3 = 1, colorata in base alla velocità con cui ogni punto converge. Un frattale di convergenza (non di fuga).
fractal-multibrot6-about = Multibrot alla potenza 6: simmetria di ordine cinque.
fractal-multibrot7-about = Multibrot alla potenza 7: simmetria di ordine sei, con i bulbi che si assottigliano verso un cerchio.
fractal-multibrot8-about = Multibrot alla potenza 8: simmetria di ordine sette.
fractal-burning-ship3-about = Il Burning Ship alla potenza 3: si prendono i valori assoluti, poi si eleva al cubo.
fractal-burning-ship4-about = Il Burning Ship alla potenza 4.
fractal-burning-ship5-about = Il Burning Ship alla potenza 5.
fractal-tricorn3-about = Il Tricorn (Mandelbar) alla potenza 3: simmetria di ordine quattro.
fractal-tricorn4-about = Il Tricorn alla potenza 4: simmetria di ordine cinque.
fractal-tricorn5-about = Il Tricorn alla potenza 5: simmetria di ordine sei.
fractal-celtic3-about = La piega Celtic alla potenza 3: il valore assoluto della parte reale di z^3.
fractal-celtic4-about = La piega Celtic alla potenza 4.
fractal-celtic5-about = La piega Celtic alla potenza 5.
fractal-buffalo3-about = Il Buffalo alla potenza 3: i valori assoluti di entrambe le parti di z^3.
fractal-buffalo4-about = Il Buffalo alla potenza 4.
fractal-buffalo5-about = Il Buffalo alla potenza 5.
fractal-custom-about = Una formula scritta in espressioni in stile Fractint (Frattale ▸ Formula personalizzata…). La maggior parte fa lo zoom profondo per perturbazione come le famiglie integrate; la finestra della formula dice se la tua lo fa.
fractal-life-about = Il Gioco della vita di Conway e i suoi parenti su un piano illimitato: regole Generations, regole non totalistiche (Hensel), file di pattern. Avvia, avanza, disegna.
fractal-lsystem-about = Sistemi di Lindenmayer: curve, curve che riempiono lo spazio, isole e piante disegnate da una tartaruga a partire da una grammatica di riscrittura. L’ordine segue lo zoom, così i dettagli non finiscono mai. I file .l di Fractint si aprono direttamente.

# Shown where the other families show their formula (z -> z^2 + c).
fractal-custom-formula = z -> la formula che scrivi
# Shown where the other families show their formula. "B3/S23" is the rule's notation: keep it.
fractal-life-formula = B3/S23: nasce con 3 vicini vivi, sopravvive con 2 o 3
# Shown where the other families show their formula. "F = F+F--F+F" is the grammar's notation: keep it.
fractal-lsystem-formula = F = F+F--F+F: ogni simbolo viene riscritto, poi disegnato da una tartaruga

# The last paragraph of the picker's hover text for a family that renders only on the direct path.
fractal-no-deep-zoom = Solo rendering diretto - questa famiglia non ha zoom profondo.

## Names of settings, shown in the Settings menu and the welcome window.

theme-dark = Scuro
theme-light = Chiaro
# The update track that follows only stable releases.
update-track-stable = Stabile
# The update track that also offers pre-releases (betas).
update-track-beta = Beta (pre-release)

## The status bar (its hover texts; the readouts follow in their own section)

status-period-hover = Periodo del minibrot su cui sei saltato (premi M, o Vai alla posizione ▸ Vai a una struttura). Resta finché la vista rimane sulla struttura e scompare quando ti sposti.

## The playback transport: the player over the view while a tour plays.

# $close: the close button's icon, as drawn on that button.
play-finished-hover = Finito — torna indietro con la barra, o { $close } per chiudere
# $kind: "benchmark" or "script" (the variants hold the wording); $name: the tour's name.
play-playing-hover =
    { $kind ->
        [benchmark] Riproduzione del benchmark «{ $name }»
       *[other] Riproduzione dello script «{ $name }»
    }
play-restart = Ricomincia dall’inizio
play-back = Indietro di 10 secondi
play-again = Riproduci di nuovo dall’inizio
play-resume = Riprendi
play-pause = Pausa
play-stop = Ferma e riavvolgi all’inizio
play-forward = Avanti di 10 secondi
play-speed-hover = Velocità di riproduzione (clic per passare tra 0,5× / 1× / 2× / 4×)
play-render = Renderizza questo script in una sequenza di fotogrammi…
play-render-none = Il benchmark integrato non ha un file di script da renderizzare
play-open = Riproduci un altro script…
play-loop = Ripeti il tour quando arriva alla fine
play-close = Chiudi il lettore e ripristina le tue impostazioni di vista
# A notice beside the scrub bar while the renderer holds the tour's clock. Keep it short.
play-waiting = in attesa dei dettagli
# "[playback] pace" is a setting in the script file: keep it as written.
play-waiting-hover = L’orologio del tour è in pausa mentre il renderer risolve questa vista (costruzione del riferimento / budget di iterazioni in crescita). La riproduzione riprende da sola; vedi [playback] pace nello script.
play-scrub = Trascina per scorrere il tour
# $behind: how far the script clock is behind real time, as minutes:seconds (1:05).
play-scrub-behind =
    Trascina per scorrere il tour
    L’orologio dello script è indietro di { $behind } rispetto al tempo reale (il regolatore di ritmo ha rallentato il tour per permettere al renderer di stare al passo). Il segno superiore indica dove sarebbe la riproduzione in tempo reale.

## The autopilot's notices (toasts). Where a notice has two lines, the first is the verdict and the second what to do about it.

autopilot-on-detail = Pilota automatico attivo — tuffo verso i dettagli (qualsiasi comando lo ferma)
autopilot-off = Pilota automatico disattivato
autopilot-misi-needs-mandelbrot =
    Pilota automatico: un bersaglio Misiurewicz richiede l’insieme di Mandelbrot
    Tuffo verso i dettagli, invece
autopilot-on-misi =
    Pilota automatico attivo — ricerca del punto di Misiurewicz più vicino…
    Qualsiasi comando lo ferma
# $seconds: how long the search has run, in whole seconds.
autopilot-misi-finding =
    Pilota automatico: ricerca del punto di Misiurewicz più vicino…
    { $seconds } s · qualsiasi comando lo ferma
autopilot-misi-not-found =
    Pilota automatico: nessun punto di Misiurewicz trovato vicino a questa vista
    La ricerca potrebbe non arrivare a questa profondità — prova il bersaglio Dettaglio
# ($preperiod,$period) names the point, as mathematicians write it: keep the notation. $widths: a power of two (2^$widths screen widths).
autopilot-misi-off-screen =
    Pilota automatico: il punto di Misiurewicz più vicino ({ $preperiod },{ $period }) è fuori dallo schermo
    A ~2^{ $widths } larghezze di schermo — avvicinati al centro di una spirale
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-not-solved =
    Pilota automatico: il punto di Misiurewicz ({ $preperiod },{ $period }) vicino a questa vista non è stato risolto
    Prova il bersaglio Dettaglio
autopilot-misi-refused = Pilota automatico: calcolo di Misiurewicz rifiutato (coppia non valida)
autopilot-misi-stopped = Pilota automatico: il risolutore di Misiurewicz si è fermato in modo imprevisto
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-diving =
    Pilota automatico: tuffo nel Misiurewicz ({ $preperiod },{ $period })
    Qualsiasi comando lo ferma
# ($preperiod,$period) names the point: keep the notation. $octaves: how often the structure repeats (2.37); $twist: its turn in degrees, signed (+12.5).
autopilot-misi-diving-repeat =
    Pilota automatico: tuffo nel Misiurewicz ({ $preperiod },{ $period })
    La struttura si ripete ogni { $octaves } ottave, torsione { $twist }° · qualsiasi comando lo ferma
# $limit: the magnification, as written (1e300).
autopilot-dive-limit = Pilota automatico: limite di tuffo raggiunto (~{ $limit }×)
# $have: the fixed iteration count; $typical: what this depth typically needs.
autopilot-dead-end-short =
    Pilota automatico fermato: non resta nulla a cui mirare — le iterazioni sono fissate a { $have } e questa profondità di solito ne richiede ~{ $typical }
    Attiva le iterazioni automatiche o aumenta il numero
# $limit: the fixed iteration count.
autopilot-dead-end-cap =
    Pilota automatico fermato: non resta nulla a cui mirare — tutto ciò che è in vista arriva al limite fisso di { $limit } iterazioni
    Attiva le iterazioni automatiche o aumenta il numero
autopilot-dead-end = Pilota automatico: nessun dettaglio davanti (fermato)

## Tours (camera scripts)

# The name of the file type in the file dialog's filter list.
script-file-filter = Script Fractadyne (TOML)
# $name: the script's name.
script-finished = Script terminato — «{ $name }»
# The title of the notice that follows (the file's path and the error, in English).
script-load-failed = Impossibile caricare lo script

## Installing a language file

# The name of the file type in the file dialog's filter list.
language-file-filter = Lingua di Fractadyne

## The notice when the graphics driver's shader cache stops keeping shaders. The text is shown in a fixed-width font with its line breaks as written: break the lines where they read well.

driver-cache-title = Il driver grafico non conserva gli shader
# $count: how many shaders compiled again; $pipeline: the slowest one's name (a technical id: keep it); $seconds: its time in seconds (7.5).
driver-cache-body =
    Il driver grafico ha ricompilato { $count } shader di Fractadyne in questa
    sessione, benché li avesse già compilati tutti in precedenza (il più lento:
    { $pipeline }, { $seconds } s). La sua cache degli shader non conserva gli
    shader compilati, molto spesso perché è piena. Finché non cambia, esportazioni,
    viste profonde e avvio di Fractadyne possono richiedere secondi di compilazione.
# $folder: the cache folder's path; $size: its size in MB (1024). "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia-folder =
    Per risolvere: chiudi Fractadyne, poi elimina il contenuto di
      { $folder }   ({ $size } MB)
    Il driver ricostruisce la cache. Per evitare che si riempia di nuovo, aumenta
    Pannello di controllo NVIDIA > Gestisci le impostazioni 3D > Dimensione cache shader.
# "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia =
    Per risolvere: aumenta Pannello di controllo NVIDIA > Gestisci le impostazioni 3D >
    Dimensione cache shader, oppure svuota la cache degli shader del driver.
# "AMD Software > Graphics > Advanced > Reset Shader Cache" names AMD's own settings: use the names AMD's software shows in this language.
driver-cache-fix-amd = Per risolvere: AMD Software > Grafica > Avanzate > Reimposta cache shader.
driver-cache-fix-other = Per risolvere: svuota la cache degli shader del driver grafico.

## Why a language file could not be installed: each ends the sentence "{ $file } is not a usable language file: …".

language-install-not-utf8 = non è testo UTF-8
language-install-too-large = troppo grande
# "fractadyne-<language>.ftl" is the file name to use: keep it as written.
language-install-no-tag = non indica di quale lingua si tratti (chiamalo fractadyne-<language>.ftl)
language-install-english = l’inglese è integrato
language-install-empty = non traduce alcun messaggio di questa versione
language-install-no-folder = non esiste una cartella delle impostazioni

## The notice after a session that did not end cleanly, when its iteration setting was set aside.

# $base: the iteration count automatic iterations start from.
quarantine-setting-auto = iterazioni automatiche a partire da una base di { $base }
# $count: the fixed iteration count.
quarantine-setting-fixed =
    { $count ->
        [one] { $count } iterazione
       *[other] { $count } iterazioni
    }
# $end: "device" when the session lost its graphics device, "unclean" otherwise (the variants hold the wording). $depth: the magnification, as written (1.2e148). $setting and $restored: an iteration setting (quarantine-setting-auto or -fixed).
quarantine-notice =
    { $end ->
        [device] La sessione precedente ha perso il dispositivo grafico a { $depth }× mentre era impostata su { $setting }. Questa sessione è partita invece con { $restored }, così non può ricadere subito nello stesso carico.
       *[other] La sessione precedente non si è chiusa correttamente a { $depth }× mentre era impostata su { $setting }. Questa sessione è partita invece con { $restored }, così non può ricadere subito nello stesso carico.
    }

## The second graphics card's status line, under its setting in the control panel.

gpu2-state-off = Disattivata: la scheda grafica di questa finestra renderizza tutto.
gpu2-state-starting = Avvio…
# $name: the card's name (and its graphics API).
gpu2-state-running = In uso: { $name }
# $why: the error, as the graphics driver reports it.
gpu2-state-failed = Impossibile avviare: { $why }
# $name: the card's name.
gpu2-state-lost = Fermata: { $name } è andata persa. D’ora in poi una sola scheda; sceglila di nuovo per riprovare.
# $state: one of the lines above. "--worker-gpu" is a command-line option: keep it.
gpu2-state-pinned = { $state } (impostato da --worker-gpu)

## The custom-formula dialog: how deep the formula renders (one line under the formula).

formula-depth-deep = Zoom profondo per perturbazione, con intervallo esteso oltre 1e28x come per i frattali integrati (ogni passo viene iterato: per ora niente approssimazione in serie né BLA per le formule personalizzate).
# $why: what the formula has that keeps it off the deep-zoom path (one of the formula-depth-why messages below).
formula-depth-direct = Rendering diretto ({ $why } non ha ancora una forma per lo zoom profondo): nitido finché un passo di c in precisione singola non copre un pixel — circa da 1e4x a 1e5x, e meno in profondità lontano dall’origine.
formula-depth-why-init = una sezione di inizializzazione
formula-depth-why-bailout = un bailout proprio
formula-depth-why-variable = una variabile conservata da un passo all’altro
formula-depth-why-previous = l’iterata precedente
formula-depth-why-comparison = un confronto o un blocco if
# floor, ceil, trunc and round are functions of the formula language: keep them.
formula-depth-why-rounding = l’arrotondamento (floor, ceil, trunc, round)
# "maxit" is a word of the formula language: keep it.
formula-depth-why-maxit = maxit
formula-depth-why-power = una potenza con esponente variabile
formula-depth-why-perturbed = un programma già perturbato

## Image and frame sizes: the presets described in words (the others are standard names: HD 720p, WUXGA, DCI 4K).

# $size: the pixel size (2048×2048).
size-preset-square = Quadrato — { $size }
# $ratio: the aspect ratio (9:16); $size: the pixel size.
size-preset-portrait = Verticale { $ratio } — { $size }
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
fractal-family-custom = Personalizzata
# Conway's Game of Life.
fractal-family-life = Life
fractal-family-lsystem = L-system
# A family at a power: "Multibrot 3", "Burning Ship 4". $family: a name above; $power: the power.
fractal-with-power = { $family } { $power }

## Names of choices: colouring methods, orbit traps, palette animation, snapshots, angle units.

color-method-smooth = Iterazione continua
color-method-stripe = Media delle strisce
color-method-triangle = Disuguaglianza triangolare
color-method-trap = Trappola orbitale
color-method-distance = Stima della distanza
color-method-decomposition = Decomposizione
# The shape an orbit trap catches the orbit with.
trap-point = Punto
trap-cross = Croce
trap-circle = Cerchio
# Palette animation: none.
palette-anim-off = No
palette-anim-forward = Avanti
palette-anim-reverse = Indietro
palette-anim-pingpong = Ping-pong
# Palette animation: a new random gradient every so often.
palette-anim-random = Gradienti casuali
# What the Snapshot button does.
snapshot-mode-ask = Chiedi ogni volta
snapshot-mode-screen = Cattura dello schermo
snapshot-mode-render = Rendering completo
# The unit of an angle typed into Go to location's polar fields.
angle-degrees = gradi
angle-radians = radianti
angle-turns = giri

## Report an issue: the choices. (The report itself is sent in English.)

issue-kind-crash = Arresto anomalo / blocco dell’applicazione
issue-kind-rendering = Rendering errato
issue-kind-performance = Problema di prestazioni
issue-kind-ui = Problema di interfaccia / usabilità
issue-kind-feature = Richiesta di funzionalità
issue-kind-other = Altro
severity-low = Bassa
severity-medium = Media
severity-high = Alta
severity-blocking = Bloccante
repro-always = Sempre
repro-sometimes = A volte
repro-once = È successo una volta
repro-cannot = Non riproducibile

## The benchmark's depths and the diagnostics' tests.

# $zoom: the dive's final magnification (1e32).
bench-depth-standard = Standard ({ $zoom }×)
# $zoom: the dive's final magnification (1e48).
bench-depth-ultra = Ultra profondo ({ $zoom }×)
diag-test-selftest = Autotest
diag-test-uitest = Test dell’interfaccia
diag-test-gputest = Verifica aritmetica GPU

## The status bar's readouts: each word is followed by a value in a slot of fixed width. Keep the words short (one word each).

status-center = centro
status-cursor = cursore
status-zoom = zoom
# Short for iterations.
status-iter = iter
# The period of the minibrot jumped to.
status-period = periodo
# A Game of Life view's scale (followed by "1 px = 4 cells" or "8 px / cell").
status-scale = scala
# A Game of Life view's generation. Short.
status-generation = gen
# A Game of Life view's population. Short.
status-population = pop
# An L-system's order (how many times its rules were applied).
status-order = ordine
# How many segments and shapes an L-system drew.
status-drawn = disegnati
# $value: how many cells one pixel covers (4, 9.99e300). "px" is the unit symbol: keep it.
life-scale-cells = 1 px = { $value } celle
# $value: how many pixels one cell covers (8). "px" is the unit symbol: keep it.
life-scale-pixels = { $value } px / cella

## The status bar's limit diagnostic: a short label (after a warning sign), and what it means when hovered.

# The view is as deep as this location can be rendered. Short.
status-limit-depth = limite profondità
# Every pixel ran out of iterations. Short.
status-limit-exhausted = iter esaurite
# Pixels are capped at the iteration budget. Short.
status-limit-capped = iter limitate
# $pixels: how many pixels wide one step of the coordinates is; $sharp_to: the magnification the view is sharp to (written out, 1,234,567).
status-limit-direct-detail = Questa vista viene renderizzata direttamente sulla GPU in precisione singola, dove qui un passo delle coordinate è largo { $pixels } pixel: pixel vicini calcolano lo stesso punto, quindi l’immagine si spezza in blocchi. In questa posizione è nitida fino a circa { $sharp_to }×. Questa formula non ha ancora un percorso di zoom profondo (perturbazione), quindi zoomando oltre non si vedono altri dettagli.
# $samples: the GPU buffer's capacity in orbit samples; $iterations: the iterations pixels are limited to.
status-limit-depth-detail = L’orbita di riferimento ha raggiunto il limite del buffer della GPU ({ $samples } campioni) senza fuggire, quindi qui i pixel sono limitati a ~{ $iterations } iterazioni e dettagli più fini non si possono calcolare in questa vista — né in tempo reale né in esportazione. È l’attuale limite di profondità per questa famiglia di posizioni.
# $budget: the largest iteration budget this view can be given.
status-limit-exhausted-detail = Ogni pixel ha comunque esaurito le iterazioni con il budget più grande che si può dare a questa vista ({ $budget }). O qui la vista è dentro l’insieme (normale in profondità), o richiede più iterazioni di quante l’app possa fornire al momento — l’immagine è la stessa in entrambi i casi, quindi il budget è stato riportato a un valore economico.
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations" is the setting's name in the control panel's Quality section.
status-limit-capped-max = Il { $percent }% dei pixel ha esaurito le iterazioni con il budget di { $budget }, e quelli che fuggono terminano proprio a quel tetto — la vista è sottorisolta. Iterazioni è già al massimo, quindi questa vista ha bisogno di più di quanto l’app possa dare al momento.
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations", "Controls" and "Quality" are the names of a setting, the control panel and its section.
status-limit-capped-raise = Il { $percent }% dei pixel ha esaurito le iterazioni con il budget di { $budget }, e quelli che fuggono terminano proprio a quel tetto — la vista è sottorisolta. Aumenta Iterazioni (Controlli ▸ Qualità) per risolverla.
