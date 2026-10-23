### Fractadyne user interface: Polish (Polski).
### A draft: machine translation (Claude, by Anthropic) of the English file, 2026-10-23, not yet
### reviewed by a native speaker. Corrections are welcome as pull requests (see i18n/README.md).

## Language data: read by Fractadyne, not shown as messages.

# This language's own name, shown in the language menu.
language-name = Polski
# The same name in English, shown beside it in the menu.
language-english-name = Polish
# The writing system the app must be able to draw (an ISO 15924 code: Latn, Cyrl, Jpan, Hans, Khmr).
language-script = Latn
# Punctuation for counts, durations and sizes. Never used for coordinates or magnifications.
# One character each. A space as the group separator is written as a quoted literal, so it shows:
# { "\u00A0" } (no-break space) or { "\u202F" } (narrow no-break space).
number-decimal = ,
number-group = { "\u00A0" }
# Whether a native speaker has reviewed the translation: draft (the language menu says so) or
# reviewed. The English file is the source.
language-status = draft

## The language menu (top bar) and the Languages window

# The language menu's last item: opens the Languages window.
language-more = Więcej języków…
# The Languages window's title.
language-window-title = Języki
# The Languages window's introduction.
language-window-intro = Interfejs Fractadyne w innym języku. Język to jeden plik: jest dołączany do wydania albo można go zainstalować tutaj. Współrzędne, pliki i wiersz poleceń pozostają po angielsku w każdym języku.
# Column headers of the installed-language list.
language-col-language = Język
language-col-translated = Przetłumaczono
language-col-from = Źródło
# Where a language comes from.
language-source-builtin = wbudowany
language-source-package = to wydanie
language-source-user = zainstalowany
language-source-dev = folder roboczy
# $percent: how much of the interface the file translates, 0–100.
language-coverage = { $percent }%
# $count: messages in the file that failed the checks and show in English instead.
language-dropped =
    { $count ->
        [one] jeden komunikat wymaga sprawdzenia
        [few] { $count } komunikaty wymagają sprawdzenia
        [many] { $count } komunikatów wymaga sprawdzenia
       *[other] { $count } komunikatu wymaga sprawdzenia
    }
# Buttons of the Languages window.
language-install = Zainstaluj plik języka…
language-install-hover = Wybierz plik języka Fractadyne (.ftl). Zostanie sprawdzony, a potem skopiowany do folderu języków w ustawieniach, gdzie pozostanie po aktualizacjach.
language-open-folder = Otwórz folder języków
language-reload = Wczytaj ponownie
language-reload-hover = Wczytaj pliki języków jeszcze raz (po edycji lub dodaniu pliku).
language-use = Użyj
# Notices after installing a file. $name: the language's own name; $file: the file's name.
language-installed = Zainstalowano język { $name } z pliku { $file }.
# $file: the file's name; $why: what was wrong with it.
language-install-failed = { $file } nie jest prawidłowym plikiem języka: { $why }
# Shown when a language file cannot be loaded and English is used instead. $name: the language.
language-load-failed = Nie udało się wczytać języka { $name }, więc Fractadyne działa po angielsku: { $why }
# A language whose writing system needs a font this computer does not have (Japanese or Chinese without a system font). $name: the language, in its own name.
language-font-missing = Na tym komputerze nie znaleziono czcionki, która potrafi wyświetlić język { $name }, więc nie można go pokazać. Zainstaluj czcionkę dla języka { $name } (na przykład systemowy pakiet językowy) i wybierz go ponownie.
# Shown after a draft translation's name in the language menu and the Languages window. Short.
language-draft = szkic
language-draft-hover = Szkic: to tłumaczenie nie zostało jeszcze sprawdzone przez rodzimego użytkownika języka. Poprawki są mile widziane jako pull request do pliku języka.
# The heading over the decimal-mark choice, in the language menu and the Languages window.
language-decimal-mark = Separator dziesiętny
language-decimal-mark-hover = Sposób zapisu liczb: czasów, rozmiarów i liczebności. Współrzędne i powiększenia zawsze mają kropkę, bo tak odczytują je pliki i inne programy.
# $sample: a number written with this language's own punctuation (1,234.5).
language-decimal-language = Tak jak w języku ({ $sample })
# $sample: a number written with a decimal point (1,234.5).
language-decimal-point = Kropka ({ $sample })
# $sample: a number written with a decimal comma (1.234,5).
language-decimal-comma = Przecinek ({ $sample })

## Asked once when the system's language changes

# The window's title.
language-changed-title = Język
# $new: the system's new language, in its own name.
language-changed-question = Język systemu to teraz { $new }. Używać go w Fractadyne?
# Buttons. $name: a language's own name.
language-changed-use = Użyj: { $name }
language-changed-keep = Zachowaj: { $name }

## The welcome window (first run)

welcome-title = Witamy w Fractadyne
welcome-intro = Eksplorator głębokiego zoomu dla zbioru Mandelbrota i jego krewnych. Kilka elementów sterowania na początek:
# The language row, at the top of the window.
welcome-language = Język
# Controls: the key or gesture on the left, what it does on the right.
welcome-key-drag = przeciągnij
welcome-key-drag-does = przesuwa widok
welcome-key-scroll = przewiń
welcome-key-scroll-does = przybliża / oddala w stronę kursora
welcome-key-space = przytrzymaj Space
welcome-key-space-does = płynny, ciągły zoom (Shift+Space oddala)
# Shown after the click-zoom toolbar icon.
welcome-key-click = potem kliknij
welcome-key-click-does = zanurza w kliknięty punkt (pasek narzędzi)
welcome-key-m-does = skacze do najbliższego minibrota w jego własnej skali
welcome-landmarks = Na początek skocz do punktu orientacyjnego:
welcome-theme = Motyw
welcome-watermark = Pokaż znak „Fd”
welcome-watermark-hover = Mały znak marki w prawym dolnym rogu widoku. Wyłączenie usuwa go z widoku na żywo i z eksportowanych obrazów.
welcome-updates = Aktualizacje
welcome-check-updates = Sprawdzaj aktualizacje
welcome-check-updates-hover = Przy uruchomieniu programu pyta GitHub o najnowsze wydanie. Nic nie jest wysyłane.
welcome-files = Pliki
welcome-no-config = (brak katalogu konfiguracji)
welcome-open = Otwórz
welcome-rendering-on = Renderowanie na
welcome-open-help = Otwórz pełną pomoc  (F1)
# $icon: the Settings icon.
welcome-more-in-settings = Więcej w menu Plik → { $icon } Ustawienia
welcome-get-started = Zaczynamy

## Menus: shared

# A percentage in a menu (UI scale). $value: the number.
common-percent = { $value }%
# A state shown after a menu item's name, in brackets.
menu-state-connected = połączono
menu-state-rendering = renderowanie
menu-state-running = działa

## Navigate ▸ Bookmarks (also the toolbar's star)

menu-bookmarks = Zakładki
menu-bookmarks-add = Dodaj bieżący widok
menu-bookmarks-manage = Zarządzaj…

## The fractal picker (Fractal menu and the toolbar's dropdown)

menu-family-powers = Wyższe potęgi
menu-family-automata = Automaty komórkowe
menu-family-lsystems = L-systemy
menu-fractal-custom = Własny wzór…
menu-fractal-library = Biblioteka wzorów…
menu-fractal-library-hover = Zapisane własne wzory: stosowanie, edycja, import i eksport
# The "Show" group: which picture of the formula is drawn. $name: the formula's name (Mandelbrot, Burning Ship…).
menu-show = Pokaż
menu-show-set = Zbiór { $name }
menu-show-set-hover = Płaszczyzna parametrów — sam zbiór { $name }.

    Każdy jego punkt to inny zbiór Julii; to mapa ich wszystkich.
menu-show-julia = Zbiór Julii
menu-show-julia-hover = Jeden punkt zbioru { $name } narysowany jako osobny obraz — kształt, który powstaje, gdy ten punkt pozostaje stały.

    Punkt wybiera się, najeżdżając kursorem na mapę w trybie „Oba, połączone”; kliknięcie tam go przypina.
menu-show-julia-none = { $name } nie ma wolnego parametru, więc nie ma zbioru Julii.
menu-show-both = Oba, połączone
menu-show-both-hover = Zbiór { $name } po lewej, a po prawej zbiór Julii punktu, nad którym jest kursor.

    Przesuwaj kursor po mapie i obserwuj, jak zmienia się zbiór Julii — to najszybszy sposób, by zobaczyć związek między nimi.
menu-show-both-none = { $name } nie ma zbioru Julii, z którym można go połączyć.

## File menu

menu-file = Plik
menu-file-open-view = Otwórz widok lub lokalizację…
menu-file-gallery = Galeria…
menu-file-export = Eksportuj obraz…
menu-file-snapshot = Migawka
menu-file-open-life = Otwórz wzorzec Life…
menu-file-open-life-hover = Plik RLE, .cells lub Life 1.05/1.06, pokazany jako wszechświat Life.
menu-file-life-text = Tekst wzorca Life…
menu-file-life-text-hover = Wklej wzorzec jako tekst (RLE w takiej postaci, w jakiej podaje go LifeWiki).
menu-file-save-life = Zapisz wzorzec Life…
menu-file-save-life-hover = Wszechświat w obecnym stanie, jako RLE.
menu-file-open-lsystem = Otwórz L-system…
menu-file-open-lsystem-hover = Plik .l programu Fractint (z wpisami do wyboru) lub system .lsys.
menu-file-save-lsystem = Zapisz L-system…
menu-file-save-lsystem-hover = Tekst systemu jako plik .lsys.
menu-file-export-svg = Eksportuj SVG…
menu-file-export-svg-hover = Widok jako rysunek SVG: jego linie i kształty jako wektory, w jego kolorach.
menu-file-render-client = Klient renderowania…
menu-file-render-client-hover = Użycz GPU tego komputera farmie renderującej: połącz się z kontrolerem (Narzędzia ▸ Renderuj na farmie… na innym komputerze) i renderuj dla niego klatki.
menu-file-settings = Ustawienia
menu-file-reset = Resetuj stan aplikacji…
menu-file-reset-hover = Usuwa cały zapisany stan (sesję, zakładki, miniatury) i zaczyna od nowa. Najpierw prosi o potwierdzenie.
menu-file-quit = Zakończ

## File ▸ Settings

menu-settings-fps = Limit klatek na sekundę
menu-settings-fps-uncapped = Bez limitu
# $fps: frames per second.
menu-settings-fps-n = { $fps } FPS
menu-settings-ui-scale = Skala interfejsu (rozmiar czcionki)
menu-settings-theme = Motyw
menu-settings-updates = Aktualizacje
menu-settings-updates-stable-hover = Najnowsze stabilne wydanie
menu-settings-updates-beta-hover = Najnowsza kompilacja, łącznie z wydaniami wstępnymi
menu-settings-updates-on-launch = Sprawdzaj aktualizacje przy uruchomieniu
menu-settings-updates-on-launch-hover = W przeciwnym razie sprawdzaj ręcznie: Pomoc → Sprawdź aktualizacje.
# Followed by its shortcut, (Ctrl+S).
menu-settings-snapshot = Migawka
menu-settings-snapshot-ask-hover = Pytaj o wybór przy każdym naciśnięciu
menu-settings-snapshot-screen-hover = Zapisz widok dokładnie tak, jak wygląda na ekranie (natychmiast, PNG)
menu-settings-snapshot-render-hover = Pełny render z ustawieniami eksportu, w tle (przy głębokich widokach może potrwać długo)
menu-settings-refcache = Pamięć podręczna referencji…
menu-settings-refcache-hover = Orbity referencyjne głębokiego zoomu przechowywane na dysku, aby lokalizacja, do której się wraca, renderowała się w kilka sekund. Gdzie jest, ile zajmuje, jej limit i sposób jej wyczyszczenia.

## Fractal menu

menu-fractal = Fraktal

## View menu

menu-view = Widok
menu-view-zoom-home = Oddal do pełnego widoku
menu-view-zoom-home-hover = Oddal do pełnego widoku początkowego (z animacją)
menu-view-reset = Przywróć widok domyślny
menu-view-reset-hover = Przywróć domyślny widok fraktala (natychmiast)
# Followed by its shortcuts.
menu-view-undo = Cofnij widok
menu-view-redo = Ponów widok
menu-view-control-panel = Panel sterowania
menu-view-control-panel-hover = Pokaż/ukryj panel sterowania po prawej stronie.
menu-view-minimap = Minimapa
menu-view-minimap-hover = Pokazuje mały podgląd widoku początkowego ze znacznikiem „jesteś tutaj” i głębokością zoomu. Kliknięcie przenosi do wskazanego obszaru.
menu-view-perf-panel = Panel wydajności
menu-view-timestamp = Pokaż znacznik czasu
menu-view-timestamp-hover = Rysuje nad widokiem duży zegar czasu, który upłynął. Pokazuje to samo „+12.345s”, którym dziennik oznacza każdy wiersz, więc nagranie ekranu z problemem widocznym tylko w ruchu można zestawić z dziennikiem klatka po klatce. Domyślnie wyłączone.
menu-view-zoom-target = Pokaż cel autozoomu
menu-view-zoom-target-hover = Podczas autozoomu obrysowuje obszar widoku, w który zmierza zoom — część, która po kilku kolejnych powiększeniach wypełni ekran — liniami od jego narożników do narożników ekranu, a punkt, ku któremu się kieruje, zaznacza pierścieniem. Domyślnie wyłączone.
menu-view-live-panel = Panel renderowania na żywo
menu-view-live-panel-hover = Panel czołowy nad widokiem, który pokazuje renderer na żywo przy pracy: co zleciła każda klatka, jak stary jest obraz na ekranie, przejście przez pasma iteracji, budowanie referencji i maszynę pod spodem. Klawisz B przełącza jego rozmiary, a × w rogu go zamyka; można go przeciągnąć do dowolnego rogu; po najechaniu na przyrząd widać, co oznacza oraz jak wygląda stan zdrowy, a jak kłopotliwy.
# The live render panel's sizes.
menu-view-live-off = Wyłączony
menu-view-live-off-hover = Bez panelu.
menu-view-live-strip = Pasek
menu-view-live-strip-hover = Wyświetlacze nixie i rząd lampek.
menu-view-live-panel-size = Panel
menu-view-live-panel-size-hover = Przyrządy: tarcza przejścia, wiek obrazu, kafelki, przebiegi GPU, tarcza głębokości, lampki.
menu-view-live-full = Pełny
menu-view-live-full-hover = Panel i maszynownia: kroki, GPU, pamięć, rdzenie, piksele.
menu-view-live-opacity = Krycie
menu-view-watermark = Pokaż znak wodny „Fd”
menu-view-watermark-hover = Rysuje dyskretny znak marki „Fd” w prawym dolnym rogu widoku na żywo i eksportowanych obrazów. Domyślnie włączone.

## Color menu

menu-color = Kolor
menu-color-method = Metoda
menu-color-palette = Paleta
menu-color-custom = Własna
menu-color-duotone = Dwutonowa
menu-color-binary = Binarna (zbiór)
menu-color-binary-hover = Płaskie dwa kolory: w zbiorze lub poza zbiorem, bez gradientu.
menu-color-edit-gradient = Edytuj gradient…
menu-color-normalize = Normalizuj głębokie kolory
menu-color-normalize-hover = Przemapowuje paletę na zmierzony zakres ucieczki widoku, gdy gęste pole wyglądałoby inaczej jak drobne plamki. Widok bez aliasingu pozostaje bez zmian — zaznacz „Zawsze dopasuj zakres”, aby przemapować mimo to. Tylko metoda gładka.
menu-color-more = Cykl, przesunięcie, animacja i efekty: panel Sterowanie ▸ Kolorowanie

## Tools menu

menu-tools = Narzędzia
# Followed by its shortcut, (M).
menu-tools-find-minibrot = Znajdź minibrot i przybliż do niego
menu-tools-find-minibrot-hover = Dociąga środek widoku metodą Newtona do dokładnego środka pobliskiego minibrota, podaje jego okres i przybliża do własnej skali minibrota — często o wiele rzędów wielkości w jednym kroku. Widok jest już głębiej? Przesuwa się tylko środek.
menu-tools-solver = Solver Newtona / Misiurewicza…
menu-tools-solver-hover = Wyznacza dokładny środek minibrota (okres + rozmiar) lub punkt Misiurewicza najbliższy widokowi — metodą Newtona w pełnej precyzji, z mnożnikiem odpychającym λ. Otwiera okno lokalizacji, w którym znajdują się solvery.
# Followed by its shortcut, (A).
menu-tools-autopilot = Autozoom (autopilot)
menu-tools-autopilot-stop = Zatrzymaj autopilota
menu-tools-autopilot-hover = Ciągły głęboki zoom bez użycia rąk: zanurza się w stronę obszaru najbogatszego w szczegóły i koryguje kurs po drodze. Każda czynność nawigacyjna go zatrzymuje.
menu-tools-priority = Priorytet:
menu-tools-priority-speed = Szybkość
menu-tools-priority-speed-hover = Zoom z ustawioną szybkością; odświeżenie pierwszą klatką, która ma szczegóły.
menu-tools-priority-quality = Jakość
menu-tools-priority-quality-hover = Tylko w pełni dopracowane klatki; zoom na nie czeka.
menu-tools-target = Cel:
menu-tools-target-detail = Szczegóły
menu-tools-target-detail-hover = Najbogatsza krawędź w widoku, wybierana na nowo w trakcie zanurzania.
menu-tools-target-misiurewicz = Punkt Misiurewicza
menu-tools-target-misiurewicz-hover = Najbliższy punkt Misiurewicza, wyznaczony raz, w który zoom zanurza się dokładnie (tylko Mandelbrot).
menu-tools-orbits = Nakładka orbity
menu-tools-orbits-hover = Rysuje drogę iteracji punktu pod kursorem. Opcje animacji: panel Sterowanie ▸ Nakładki.
menu-tools-orbits-normalize = Normalizuj (dopasuj do widoku)
menu-tools-orbits-normalize-hover = Dopasowuje orbitę do całego widoku, aby przy każdym zoomie była dobrze skadrowana (zamiast odwzorowania przez obszar widoku, przy którym w głębi ucieka poza ekran).
menu-tools-play-tour = Odtwórz przelot…
menu-tools-play-tour-hover = Wybierz skrypt przelotu do odtworzenia. Przycisk ▶ na pasku narzędzi odtwarza ostatni jednym kliknięciem.
menu-tools-tour-from-view = Przelot do bieżącego widoku…
menu-tools-tour-from-view-hover = Tworzy przelot, który przybliża od pełnego widoku do bieżącego — z podpisem i ustawionym czasem zanurzania. Zapisywany jako skrypt przelotu (.toml), który można edytować, odtwarzać lub renderować.
menu-tools-render-tour = Renderuj przelot…
menu-tools-render-tour-hover = Renderuje wczytany przelot do sekwencji klatek PNG (i opcjonalnie do pliku mp4) w osobnym procesie.
menu-tools-close-player = Zamknij odtwarzacz przelotu
menu-tools-close-player-hover = Zatrzymuje przelot, zamyka jego odtwarzacz i przywraca własne ustawienia iteracji i kolorowania
menu-tools-render-farm = Renderuj na farmie…
menu-tools-render-farm-hover = Renderuje wczytany przelot na kilku komputerach: ten steruje zadaniem, inne dołączają przez Plik ▸ Klient renderowania….
menu-tools-benchmark = Test wydajności…
menu-tools-benchmark-hover = Mierzy szybkość renderowania — przy bieżących ustawieniach lub w przebiegu standardowym (stała rozdzielczość i ustawienia), porównywalnym między komputerami. Wygrzewanie powtarza go, aby sprawdzić stabilność i dławienie termiczne.
menu-tools-last-benchmark = Pokaż ostatni test wydajności

## Navigate menu

menu-navigate = Nawigacja
menu-navigate-goto = Przejdź do lokalizacji…
menu-navigate-goto-hover = Wpisz środek/zoom, skocz do znanego punktu albo wyznacz metodą Newtona strukturę Misiurewicza / minibrota w pobliżu widoku.
menu-navigate-misiurewicz = Eksplorator Misiurewicza…
menu-navigate-misiurewicz-hover = Przegląd tego, jak wyglądają małe typy punktów Misiurewicza — miniatury z okresem zoomu i skrętem każdego punktu — i wyznaczanie dowolnego z nich metodą Newtona na dowolną głębokość. (Mandelbrot.)
menu-navigate-exact = Punkty dokładne (wyrażenia)
menu-navigate-exact-note = Korzenie bąbli głównej kardioidy, podane jako dokładne wyrażenia (cos/sin ułamka 2π). Współrzędna jest przechowywana jako wyrażenie, więc przybliżanie pozostaje dokładnie na punkcie.
menu-navigate-random = Losowa lokalizacja
menu-navigate-random-hover = Skocz do losowego punktu brzegowego bogatego w szczegóły
menu-navigate-import-kfr = Importuj .kfr…
menu-navigate-import-kfr-hover = Wczytaj plik lokalizacji Kalles Fraktaler
menu-navigate-share = Udostępnij lokalizację…
menu-navigate-share-hover = Kopiuj / wklej / zapisz / wczytaj samodzielną lokalizację (.fdn): fraktal, środek w pełnej precyzji, zoom, kolorowanie.

## Help menu

menu-help = Pomoc
# Followed by its shortcut, (F1).
menu-help-reference = Pomoc i dokumentacja…
menu-help-welcome = Powitanie / szybki start…
menu-help-welcome-hover = Przegląd z pierwszego uruchomienia: podstawowe sterowanie i kilka punktów orientacyjnych.
menu-help-diagnostics = Diagnostyka…
menu-help-diagnostics-hover = Sprawdź, czy Fractadyne działa poprawnie na tym sprzęcie, i dołącz wynik do zgłoszenia problemu
menu-help-report = Zgłoś problem…
menu-help-report-hover = Zgłoś problem w GitHub (lub e-mailem) z dołączonym raportem awarii, dziennikiem i lokalizacją
menu-help-accelerated = Szybszy głęboki zoom…
menu-help-accelerated-hover = Opcjonalna kompilacja, która oblicza orbity referencyjne głębokiego zoomu od 2,5 do 6,4 raza szybciej. Identyczne obrazy; ustawienia zostają przeniesione.
menu-help-check-updates = Sprawdź aktualizacje
# $track: "stable" or "beta", the update track in use (the variants hold the wording).
menu-help-check-updates-hover =
    { $track ->
        [beta] Sprawdź w GitHub, czy jest nowsza kompilacja beta (wydania wstępne)
       *[other] Sprawdź w GitHub, czy jest nowsza kompilacja stabilna
    }
# $version: the new version; $channel: "beta" for a pre-release, "stable" otherwise (the variants hold the wording).
menu-help-update-available =
    { $channel ->
        [beta] Dostępna wersja { $version } (beta) — pobierz
       *[other] Dostępna wersja { $version } (stabilna) — pobierz
    }
menu-help-up-to-date = Wersja aktualna
menu-help-tagline = Natywny eksplorator fraktali
menu-help-license = Licencja
# $year and $holder: the copyright year and holder, as the code gives them.
menu-help-copyright = © { $year } { $holder }. Podwójna licencja; do użytku na warunkach dowolnej z nich, według wyboru.
menu-help-source = Kod źródłowy

## The toolbar (its buttons are icons; these are their tooltips)

menu-toolbar-julia = Julia
menu-toolbar-julia-hover = Pokaż zbiór Julii tego wzoru na całe okno
menu-toolbar-dual-hover = Oba, połączone: zbiór po lewej, zbiór Julii punktu pod kursorem po prawej
menu-toolbar-open-hover = Otwórz widok lub lokalizację (PNG / EXR, .fdn, .kfr)
menu-toolbar-gallery-hover = Galeria
menu-toolbar-export-hover = Eksportuj obraz…
menu-toolbar-share-hover = Udostępnij lokalizację — kopiuj, wklej, zapisz lub wczytaj samodzielny plik .fdn (fraktal, środek w pełnej precyzji, zoom, kolorowanie)
menu-toolbar-zoom-in-hover = Przybliż
menu-toolbar-zoom-out-hover = Oddal
# $factor: the click-zoom factor (4, 10…).
menu-toolbar-click-zoom-hover = Zoom kliknięciem ({ $factor }×): lewy przycisk zanurza w punkt, prawy cofa (przeciąganie nadal przesuwa). Współczynnik w Sterowanie ▸ Nawigacja.
# Added to the click-to-zoom tooltip in the dual view.
menu-toolbar-click-zoom-dual = Gdy ta opcja jest włączona, c zbioru Julii przypina się Ctrl+kliknięciem.
menu-toolbar-autopilot-running-hover = Autozoom działa — kliknij, aby zatrzymać
menu-toolbar-autopilot-hover = Autozoom: zanurzanie w stronę szczegółów (A)
menu-toolbar-reset-hover = Resetuj widok (natychmiast)
menu-toolbar-home-hover = Oddal do widoku początkowego (z animacją)
menu-toolbar-bookmarks-hover = Zakładki: zapisz ten widok lub skocz do zapisanego
# $name: the palette's name.
menu-toolbar-next-palette-hover = Następna paleta ({ $name })
# $file: the last tour's file name.
menu-toolbar-play-last-hover = Odtwórz przelot — { $file } (kliknij) · odtwarza ostatni przelot; inny wybierz w Narzędzia ▸ Odtwórz przelot…
menu-toolbar-play-hover = Odtwórz przelot… (Narzędzia ▸ Odtwórz przelot)
menu-toolbar-speed-hover = Szybkość odtwarzania
menu-toolbar-fullscreen-hover = Przełącz pełny ekran

## Dialogs: words shared by many windows

# A button that closes a window without doing what it offers.
dlg-cancel = Anuluj
# A button that dismisses a window (nothing in it is pending).
dlg-close = Zamknij
# A button that copies the window's text to the clipboard (after a copy icon).
dlg-copy = Kopiuj
# A button that jumps to a location.
dlg-go = Przejdź
# A button that fills the fields from the view on screen.
dlg-use-current = Użyj bieżącego
# A notice after a file could not be written. $error: the system's reason, as it gives it.
dlg-save-failed = Zapis nie powiódł się: { $error }
# A notice when no graphics device is available to render with.
dlg-gpu-not-available = GPU niedostępny
# The name of the plain-text file type in a save dialog (the extension .txt is added by the code).
dlg-filter-text = Tekst

## Asked after a session that did not shut down cleanly

dlg-crash-title = Fractadyne nie został poprawnie zamknięty
dlg-crash-intro = Poprzednia sesja zakończyła się nieoczekiwanie. Na tym komputerze zapisano raport — wysłać go?
dlg-crash-review = Nic nie zostanie wysłane przed przejrzeniem: następny ekran pokazuje dokładnie, co zostałoby dołączone, a informacje o systemie można wykluczyć jednym polem wyboru.
dlg-crash-report = Zgłoś…
dlg-crash-report-hover = Otwiera okno Zgłoś problem z dołączonym raportem awarii i lokalizacją, w której wystąpiła awaria.
dlg-crash-not-now = Nie teraz
dlg-crash-dont-ask = Nie pytaj ponownie po awarii
dlg-crash-dont-ask-hover = Raporty nadal są zapisywane lokalnie, a Pomoc ▸ Zgłoś problem działa zawsze.

## Asked after a session that ended with a second graphics device in use

dlg-second-device-title = Wyłączyć drugie urządzenie graficzne?
# $device: the graphics device's name and details, as the system gives them.
dlg-second-device-intro = Poprzednia sesja zakończyła się nieoczekiwanie, gdy używane było drugie urządzenie graficzne ({ $device }). Może ono być przyczyną.
dlg-second-device-note = Po jego wyłączeniu karta, która rysuje okno, renderuje wszystko, tak jak wcześniej. Można je ponownie włączyć w Zaawansowane ▸ Druga karta graficzna.
dlg-second-device-off = Wyłącz je
dlg-second-device-keep = Pozostaw włączone

## Shown when the previous session's iteration setting was set aside

dlg-quarantine-title = Pominięto ustawienie iteracji
dlg-quarantine-change = Można je zmienić w dowolnej chwili w Sterowanie ▸ Jakość.
dlg-quarantine-keep = Zachowaj domyślne
# A button. $setting: the iteration setting the previous session used (such as "20,000 iterations").
dlg-quarantine-restore = Przywróć: { $setting }

## Navigate ▸ Go to location

dlg-goto-title = Przejdź do lokalizacji
# Before two choices: "x, y" (cartesian) and the polar form below.
dlg-goto-enter-as = Wpisz jako:
# The polar way of entering a point: an offset plus a radius r at an angle θ. Keep the symbols.
dlg-goto-polar = przesunięcie + r ∠ θ
# The tooltip of the "?" button beside the entry choice.
dlg-goto-help-hover = Co przyjmują pola: ułamki, potęgi, sqrt/sin/cos…, pi i inne stałe — otwiera Pomoc → Wyrażenia współrzędnych
# Labels over the entry fields. x0, y0, r, θ, Re and Im are the fields' symbols: keep them.
dlg-goto-polar-offset = Środek przesunięcia  (x0, y0 — zostaw 0 dla czysto biegunowego punktu)
dlg-goto-polar-radius = Promień r, kąt θ
dlg-goto-re = Środek — część rzeczywista (Re)
dlg-goto-im = Środek — część urojona (Im)
dlg-goto-zoom = Zoom (powiększenie)
# A warning under the zoom field, after a ⚠ sign. $have: the iteration count set; $typical: the count this depth typically needs.
dlg-goto-iter-warning = Iteracje są ustalone na { $have }; ta głębokość zwykle wymaga ~{ $typical }. Widok będzie renderowany płasko, dopóki ich liczba nie wzrośnie — albo włącz automatyczne iteracje (Sterowanie ▸ Jakość), które dostosowują się same.
dlg-goto-copy-hover = Skopiuj tę lokalizację do schowka
dlg-goto-cancel-hover = Zamknij bez przechodzenia
# The expressions in brackets are examples of what can be typed: keep them as they are.
dlg-goto-paste-note = Wklej środek/zoom od kogoś innego albo użyj Kopiuj, aby udostępnić. Współrzędne przyjmują wyrażenia: dokładne ułamki (-3/4), funkcje i stałe (-0.5 + 0.25*cos(pi/4), sqrt, root(x,n), ^; kąty w radianach), a pole części rzeczywistej przyjmuje całą wartość zespoloną ((37+16i)/100), która wypełnia oba pola.
# A section that opens and closes.
dlg-goto-feature = Przejdź do struktury (Mandelbrot)
dlg-goto-feature-poi = Znane punkty:
# The dropdown's text before a point is picked.
dlg-goto-feature-poi-pick = Skocz do interesującego punktu…
dlg-goto-feature-solve = Lub wyznacz strukturę w pobliżu bieżącego widoku:
# Two choices: what to solve for. Misiurewicz is a mathematician's name (a Misiurewicz point).
dlg-goto-feature-misiurewicz = Misiurewicz
dlg-goto-feature-minibrot = Najbliższy minibrot
# Labels of two small number fields; k and p are their symbols.
dlg-goto-feature-preperiod = Przedokres k
dlg-goto-feature-period = Okres p
# The faint text in an empty field: left empty, the value is found automatically.
dlg-goto-feature-auto-hint = auto
# A small button that empties both fields, so they are found automatically again.
dlg-goto-feature-auto = Auto
dlg-goto-feature-auto-hover = Wyczyść oba pola, aby następne wyszukiwanie wykryło przedokres i okres z orbity w środku widoku.
dlg-goto-feature-kp-hover = Pozostaw oba puste, aby wykryć je z orbity w środku widoku; znalezione wartości zostaną tu wpisane. Nikt, kto patrzy na spiralę, nie zna jej przedokresu, a konieczność jego podania sprawiała, że ten solver był w praktyce bezużyteczny.
# While a solve runs, followed by its elapsed time and the note below.
dlg-goto-feature-solving = Wyznaczanie…
dlg-goto-feature-solving-note = (dowolna precyzja; głębokie widoki trwają dłużej)
dlg-goto-feature-cancel-hover = Przestań czekać. Samo obliczenie dobiega końca w tle, a jego wynik jest odrzucany — nic nie zostaje w stanie połowicznym.
dlg-goto-feature-find = Znajdź w pobliżu widoku
dlg-goto-feature-find-hover =
    Wyznacza metodą Newtona dokładną strukturę, startując od bieżącego środka (dowolna precyzja). Najpierw przejdź w pobliże struktury.

    Przed wyznaczaniem ustaw powyżej pole Zoom na głębokość, której CHCESZ: punkt jest obliczany z precyzją, jakiej ta głębokość wymaga, a widok skacze prosto tam. To jedno obliczenie zamiast długiej serii ręcznych kroków zoomu.
dlg-goto-solve-cancelled = Wyznaczanie anulowane.

## Snapshot (the camera button): its tooltip, and the choice on the first press

# The camera button's tooltip, one per Snapshot setting. Followed by its shortcut, (Ctrl+S).
dlg-snapshot-hover-ask = Migawka — zapisz widok tak, jak jest wyświetlany, lub pełny render; pierwsze naciśnięcie pyta
dlg-snapshot-hover-screen = Migawka — zapisz widok dokładnie tak, jak jest wyświetlany, do ostatniego folderu eksportu
dlg-snapshot-hover-render = Migawka — pełny render z ustawieniami eksportu, do ostatniego folderu eksportu
# The window's title.
dlg-snapshot-title = Migawka
dlg-snapshot-question = Co ma zapisywać Migawka (przycisk aparatu, Ctrl+S)?
# The first choice, and a short tag beside it.
dlg-snapshot-screen = Zrzut ekranu
dlg-snapshot-screen-tag = natychmiast
dlg-snapshot-screen-note = Zapisuje widok dokładnie tak, jak wygląda na ekranie, w rozdzielczości ekranu, jako PNG w ostatnim folderze eksportu. Dokładnie to, co widać.
# The second choice, and a short tag beside it.
dlg-snapshot-render = Pełny render
dlg-snapshot-render-tag = może potrwać długo
# $size: the image size, as 1920×1080; $ss: the supersampling factor (1–4); $format: PNG or EXR.
dlg-snapshot-render-note = Renderuje z ustawieniami eksportu — { $size }, supersampling { $ss }×, { $format }. Najlepsza jakość, ale głęboki widok może zająć od minut do godzin. Działa w tle; Plik → Eksportuj obraz… pokazuje postęp i pozwala go anulować.
dlg-snapshot-remember = Zapamiętaj wybór i nie pytaj ponownie
dlg-snapshot-change = Można to zmienić w dowolnej chwili w Plik → Ustawienia → Migawka.

## Share location (.fdn)

dlg-share-title = Udostępnij lokalizację
dlg-share-intro = Samodzielna lokalizacja (fraktal, środek w pełnej precyzji, zoom, kolorowanie). Skopiuj ją, aby udostępnić, albo wklej/wczytaj inną i kliknij Zastosuj.
dlg-share-thumb = Dołącz miniaturę do zapisanego pliku
dlg-share-thumb-hover = Zapisz .fdn osadza mały obraz tego widoku, aby plik dało się rozpoznać w galerii. Dodaje około 55 KB i nie jest częścią powyższego tekstu.
dlg-share-copy-hover = Skopiuj tekst do schowka
# Buttons (the first after a save icon).
dlg-share-save = Zapisz .fdn…
dlg-share-load = Wczytaj .fdn…
dlg-share-apply = Zastosuj
dlg-share-apply-hover = Skocz do lokalizacji z pola
dlg-share-copied = Skopiowano do schowka.

## Help ▸ Report an issue (the report itself stays in English: it is read by the maintainer)

dlg-report-title = Zgłoś problem
dlg-report-intro = Najlepiej zgłosić problem w GitHub (issue) — zgłoszenie jest publiczne i można je wyszukać, a widać w nim, kiedy problem zostanie naprawiony. E-mail też się sprawdzi. Najpierw przejrzyj wszystko poniżej: nic nie opuści tego komputera bez wyraźnego polecenia.
# Labels before three dropdowns.
dlg-report-type = Typ:
dlg-report-severity = Waga:
dlg-report-repro = Powtarzalność:
dlg-report-describe = Opisz, co się stało:
# The faint text in the empty description box.
dlg-report-describe-hint = Co się stało, kroki prowadzące do problemu i czego oczekiwano zamiast tego.
# Before the checkboxes of what the report carries.
dlg-report-include = Dołącz:
dlg-report-sysinfo = Informacje o systemie (wersja, system operacyjny, CPU, GPU, VRAM)
dlg-report-location = Bieżąca lokalizacja (.fdn)
dlg-report-log = Najnowszy dziennik
dlg-report-frames = Zapis klatek (ostatnia minuta i ostatnie 40 klatek)
dlg-report-crash = Najnowszy raport awarii
dlg-report-crash-none = Najnowszy raport awarii (nie znaleziono)
dlg-report-crash-view = Lokalizacja awarii (.fdn)
dlg-report-crash-view-none = Lokalizacja awarii (.fdn) (nie znaleziono)
dlg-report-crash-view-hover = Dokładny widok, przy którym wystąpiła awaria lub utrata urządzenia graficznego — jedyna rzecz, która pozwala opiekunowi projektu ją odtworzyć. Zapisywany automatycznie.
dlg-report-test = Wynik testu diagnostycznego
dlg-report-test-none = Wynik testu diagnostycznego (uruchom go z Pomoc → Diagnostyka…)
dlg-report-test-hover = Zweryfikowany maszynowo wynik (zaliczony/niezaliczony) z własnego sprzętu — dla opiekuna projektu znacznie bardziej przydatny niż sam opis
# A section that opens and closes. $size: the report's size, as "12 KB".
dlg-report-preview = Podgląd tego, co zostanie wysłane  ({ $size })
dlg-report-github = Otwórz zgłoszenie w GitHub
dlg-report-github-hover = Otwiera stronę nowego zgłoszenia z wypełnionym tytułem; raport jest kopiowany do schowka — wklej go w treść zgłoszenia (Ctrl+V).
# Buttons (after a copy and a save icon).
dlg-report-copy = Kopiuj raport
dlg-report-copy-hover = Skopiuj wszystko do schowka
dlg-report-save = Zapisz raport…
dlg-report-save-hover = Zapisz jako .txt do załączenia
dlg-report-by-email = Lub e-mailem:
dlg-report-gmail = Napisz w Gmailu
# $address: the email address reports go to.
dlg-report-gmail-hover = Otwiera okno nowej wiadomości Gmail do { $address } z wypełnionym tematem; wklej skopiowany raport
dlg-report-email = Program pocztowy…
dlg-report-email-hover = Otwiera domyślny program pocztowy (mailto) — wymaga skonfigurowanej obsługi poczty
dlg-report-tip = Wskazówka: każdy z tych przycisków kopiuje raport do schowka — wklej go w treść zgłoszenia lub wiadomości (Ctrl+V). Jeśli to możliwe, dołącz też zrzut ekranu lub przykładowy obraz (przeciągnij go); to bardzo pomaga przy problemach z renderowaniem i interfejsem.
# Notices in the window after a button is pressed.
dlg-report-opened-github = Otwarto GitHub — raport jest w schowku; wklej go w zgłoszenie.
dlg-report-copied = Raport skopiowany do schowka.
# $path: where the report was saved.
dlg-report-saved = Zapisano w { $path }
dlg-report-opened-email = Otwarto program pocztowy — raport jest w schowku; wklej go.
dlg-report-opened-gmail = Otwarto okno nowej wiadomości Gmail — wklej skopiowany raport w treść (Ctrl+V).

## A location whose checksum does not match

dlg-checksum-title = Ta lokalizacja może być uszkodzona
# After a ⚠ sign.
dlg-checksum-mismatch = Jej suma kontrolna nie zgadza się z danymi obok.
dlg-checksum-why = Zwykle oznacza to, że tekst został po drodze obcięty lub zmieniony — częstą przyczyną jest niepełna kopia z okna czatu lub z posta na forum.
# A label over where the location came from (a file path, or the clipboard).
dlg-checksum-source = Źródło
# The two checksums, side by side: the one written in the file, and the one computed from its data.
dlg-checksum-in-file = w pliku
dlg-checksum-from-data = z danych
dlg-checksum-load-note = Można ją mimo to wczytać — współrzędne mogą być nadal poprawne, a nic nie zostanie nadpisane. Wszystko, czego nie da się odczytać, zostanie zgłoszone po wczytaniu.
dlg-checksum-load = Wczytaj mimo to
dlg-checksum-load-hover = Skocz do niej mimo niezgodności
# Notices after loading it. $zoom: the magnification, as 1.2e148.
dlg-checksum-loaded = Wczytano @ { $zoom }× mimo niezgodności sumy kontrolnej
# $zoom: the magnification, as 1.2e148; $note: what the loader found wrong with the location.
dlg-checksum-loaded-note = Wczytano @ { $zoom }× — { $note }
dlg-checksum-not-loaded = Nie wczytano.

## File ▸ Reset application state

dlg-reset-title = Resetuj stan aplikacji
# After a ⚠ sign, followed by a list.
dlg-reset-warning = To trwale usuwa wszystkie zapisane dane Fractadyne:
# The list's items, each after a bullet.
dlg-reset-session = zapisaną sesję (bieżący widok, kolorowanie, preferencje)
dlg-reset-bookmarks = wszystkie zakładki i ich miniatury
dlg-reset-refcache = pamięć podręczną orbit referencyjnych (odbudowywaną przy ponownych odwiedzinach lokalizacji)
# A label over the folder that is deleted.
dlg-reset-location = Położenie
dlg-reset-note = Tego nie można cofnąć. Bieżąca sesja nie zostanie ponownie zapisana przy wyjściu, więc przy następnym uruchomieniu wczytają się ustawienia domyślne.
dlg-reset-confirm = Resetuj wszystko
dlg-reset-done = Zresetowano stan aplikacji — przy następnym uruchomieniu wczytają się ustawienia domyślne.
# $error: the system's reason, as it gives it.
dlg-reset-failed = Reset nie powiódł się: { $error }

## Tools ▸ Tour from current view

dlg-tour-title = Przelot do bieżącego widoku
dlg-tour-intro = Tworzy przelot, który przybliża od pełnego widoku do bieżącego miejsca.
# A label before the fractal and magnification the tour ends at.
dlg-tour-destination = Cel
dlg-tour-notation = Opis (opcjonalny podpis wyświetlany podczas zanurzania)
# The faint example text in the empty caption box.
dlg-tour-notation-hint = np. Zanurzanie w trójramienną strukturę Misiurewicza…
dlg-tour-duration = Czas zoomu
# $rate: how fast the dive zooms, in powers of ten a second (as 2.5).
dlg-tour-rate = ≈ { $rate } dek./s
# A small button that puts the duration back to its default.
dlg-tour-reset = Resetuj
dlg-tour-reset-hover = Wartość domyślna zależna od głębokości
dlg-tour-duration-note = Wartość domyślna rośnie z głębokością, aby zanurzanie dało się oglądać. Szybsze zanurzania mogą przy ekstremalnym zoomie migać jednokolorowymi klatkami.
# A button, after a save icon.
dlg-tour-save = Zapisz przelot…
# The file type's name in the save dialog.
dlg-tour-filter = Skrypt Fractadyne (TOML)
# A notice. $file: the saved file's name. --render-tour is a command-line option: keep it.
dlg-tour-saved = Zapisano przelot → { $file }. Odtwórz go przez Narzędzia → Odtwórz przelot albo wyrenderuj z --render-tour.

## Navigate ▸ Bookmarks ▸ Manage…

dlg-bookmarks-title = Zakładki
# The faint text in the empty name box.
dlg-bookmarks-name-hint = nazwa (opcjonalnie)
# A button, after a bookmark icon.
dlg-bookmarks-add = Dodaj bieżący widok
dlg-bookmarks-empty = Nie ma jeszcze zakładek. Dodaj pierwszą z bieżącego widoku.
# The tooltip of a bookmark's delete (bin) button.
dlg-bookmarks-delete-hover = Usuń

## Tools ▸ Benchmark…

dlg-bench-title = Test wydajności
dlg-bench-mode = Tryb
dlg-bench-current = Bieżące ustawienia
dlg-bench-current-hover = Odtwarza stały przelot głębokiego zoomu w widoku na żywo przy bieżącej rozdzielczości i ustawieniach.
dlg-bench-standard = Standardowy
dlg-bench-standard-hover = Ustala rozdzielczość i wszystkie ustawienia renderowania, aby wynik był porównywalny między komputerami.
dlg-bench-resolution = Rozdzielczość
dlg-bench-depth = Głębokość
# Standard and Ultra are the two depths in the dropdown this describes.
dlg-bench-depth-hover = Oba zanurzenia wchodzą w głęboką ścieżkę floatexp/BLA; Ultragłęboka (1e48×) spędza tam znacznie więcej czasu niż Standardowa (1e32×) i odpowiednio dłużej trwa.
dlg-bench-burnin = Wygrzewanie (powtarzanie)
dlg-bench-burnin-hover = Uruchamia test wielokrotnie, aby ujawnić problemy ze stabilnością i dławienie termiczne.
# The label of a slider: how many times the benchmark runs.
dlg-bench-passes = przebiegi
# $size: the image size, as 1920×1080; $ss: the supersampling factor; $depth: the dive's final magnification, as 1e32.
dlg-bench-std-summary = Renderuje poza ekranem w { $size }, { $ss }× SS, Mandelbrot/gładka, zanurzanie 60 klatek do { $depth }×.
dlg-bench-run = Uruchom

## The standardized benchmark's progress window

dlg-bench-progress-title = Trwa test wydajności…
# $resolution: the benchmark's resolution, as the dropdown names it.
dlg-bench-progress-mode = Standardowy · { $resolution }
# $pass: the pass running now; $total: how many passes there are.
dlg-bench-progress-pass = Przebieg wygrzewania { $pass }/{ $total }
dlg-bench-progress-dive = Renderowanie stałego zanurzania…
# On the progress bar. $done: frames rendered; $total: frames in the dive.
dlg-bench-progress-frame = klatka { $done }/{ $total }
# $fps: frames per second, as 41.3.
dlg-bench-progress-last = ostatni przebieg: { $fps } fps

## The benchmark's results

dlg-bench-results-title = Wyniki testu wydajności
# Buttons, each after an icon.
dlg-bench-results-run-again = Uruchom ponownie…
dlg-bench-results-save = Zapisz…
dlg-bench-results-none = Nie uruchomiono jeszcze żadnego testu wydajności.
dlg-bench-results-saved = Zapisano test wydajności.

## The accelerated build's notice when its arithmetic libraries are missing

dlg-backend-title = Kompilacja przyspieszona — używa wbudowanej arytmetyki
dlg-backend-dont-show = Nie pokazuj ponownie

## File ▸ Gallery…

dlg-gallery-title = Galeria
# A button that picks the folder to show.
dlg-gallery-folder = Folder…
# A button, after a refresh icon.
dlg-gallery-refresh = Odśwież
dlg-gallery-empty = W tym folderze nie ma obrazów Fractadyne.
# An image's title when it does not name its fractal.
dlg-gallery-untitled = Obraz Fractadyne
# $zoom: the image's magnification, as 1.2e148; $saved: when it was saved, as the file gives it.
dlg-gallery-zoom = zoom { $zoom }   { $saved }
# The notes saved with the image, in quotation marks. Use your language's quotation marks.
dlg-gallery-notes = „{ $notes }”
dlg-gallery-open = Otwórz ten widok
dlg-gallery-loaded = Wczytano widok z galerii.

## File ▸ Export image…

dlg-export-title = Eksportuj obraz
# A dropdown of image sizes.
dlg-export-size = Rozmiar
# The size dropdown's text for a size set by hand. $size: the size, as 1920×1080.
dlg-export-size-custom-current = Własny — { $size }
# The size dropdown's first choice.
dlg-export-size-custom = Własny…
dlg-export-size-custom-hover = Ustaw szerokość ręcznie; wysokość wynika z proporcji poniżej.
dlg-export-width = Szerokość (px)
# After the width field. $height: the height in pixels, which follows from the aspect.
dlg-export-height-from-aspect = × { $height } (z proporcji)
dlg-export-ss = Supersampling
dlg-export-aspect = Proporcje
# The aspect choice that keeps the window's own shape.
dlg-export-aspect-window = Jak okno
# Before the choices PNG and OpenEXR.
dlg-export-format = Format:
dlg-export-hud = HUD lokalizacji
dlg-export-hud-hover = Wtapia w lewy górny róg eksportowanego obrazu panel z poziomem zoomu i współrzędnymi (skaluje się z obrazem; pokazuje zoom i środek widoku mapy/Mandelbrota).
dlg-export-glitch = Korekcja glitchy
dlg-export-glitch-hover = Korekcja glitchy perturbacji z wieloma referencjami. Automatycznie pomijana przy bardzo głębokich (floatexp) eksportach pojedynczego widoku, aby budowa referencji i renderowanie działały poza wątkiem interfejsu, a aplikacja pozostała responsywna. Nigdy nie jest używana dla widoków Mandelbrot ani Multibrot (poza trybem Julii): sprawdzona względem dokładnych obliczeń, niczego tam nie poprawiała, a wydłużała eksport nawet 5-krotnie.
dlg-export-deep-note = Głęboki eksport: referencja buduje się poza wątkiem interfejsu (interfejs pozostaje aktywny); na tej głębokości korekcja glitchy jest pomijana.
# A dropdown, shown in the dual (map + Julia) view, and its three choices.
dlg-export-dual = Układ podwójny
dlg-export-dual-side = Obok siebie
dlg-export-dual-separate = Osobne pliki
dlg-export-dual-map = Tylko mapa
dlg-export-notes = Notatki:
# The faint text in the empty notes box.
dlg-export-notes-hint = zapisywane z obrazem (≤120 znaków)
# $width and $height: the image size in pixels; $left: how many characters the notes can still take.
dlg-export-output = Wynik: { $width } × { $height } px   (pozostało znaków: { $left })
# When the export's shape differs from the window's. $percent: how much more, as 25.
dlg-export-extra-width = Pokazuje o { $percent }% więcej szerokości niż okno (nic nie jest przycinane)
# The same, when the export is taller than the window. $percent: how much more, as 25.
dlg-export-extra-height = Pokazuje o { $percent }% więcej wysokości niż okno (nic nie jest przycinane)
dlg-export-tiles-note = Renderowany w kafelkach (bez limitu rozmiaru) w wątku w tle. Plik zawiera widok i notatki, więc można go ponownie otworzyć przez Plik ▸ Otwórz widok lub lokalizację….
# A label before the button that picks the folder exports go to.
dlg-export-folder = Folder:
dlg-export-choose = Wybierz…
dlg-export-choose-hover = Wybierz katalog docelowy eksportów
dlg-export-open-after = Otwórz po zakończeniu
dlg-export-open-after-hover = Przekazuje gotowy obraz do systemowej przeglądarki, gdy tylko zostanie zapisany.
# $count: the graphics cards in this machine (2 or more).
dlg-export-all-gpus = Użyj wszystkich kart graficznych ({ $count })
dlg-export-all-gpus-hover = Dzieli obraz według kafelków między wszystkie karty graficzne tego komputera, każdą z własną kopią głębokiej referencji. Identyczne karty dają dokładnie taki obraz, jaki renderuje jedna; różne modele rysują kilka pikseli nieco inaczej, kafelek po kafelku (zmierzono: brak widocznego szwu). Jeśli karta zawiedzie, jej kafelki trafiają do pozostałych. Obrazy, których nie da się tak podzielić (niektóre metody kolorowania i wzory), używają karty tego okna.
# The export's progress.
dlg-export-preparing = Przygotowywanie (budowa referencji)…
dlg-export-saving = Zapisywanie…
dlg-export-rendering = Renderowanie…
# $time: the time since the export started, as 8.3s or 1m 04.0s.
dlg-export-elapsed = Upłynęło: { $time }
dlg-export-cancel-hover = Zamknij bez eksportowania
# A button, after a save icon.
dlg-export-save-as = Zapisz jako…
dlg-export-save-as-hover = Wybierz nazwę i położenie pliku
dlg-export-export = Eksportuj
dlg-export-export-hover = Renderuj i zapisz do folderu powyżej (nazwa automatyczna)

## Notices worded at startup (main.rs)

# A notice shown once after starting, when the saved session comes from a newer version of the app. $saved and $handled: version numbers of the session file's format (1, 2…).
app-state-newer = Tę sesję zapisała nowsza wersja Fractadyne (stan v{ $saved }; ta kompilacja obsługuje v{ $handled }). Niektóre ustawienia mogą nie zostać zastosowane, a zapis przepisze sesję w formacie tej kompilacji.
# The title of the notice that says the accelerated build's arithmetic libraries are found again (its text comes from elsewhere).
app-notice-mpfr-restored-title = Przywrócono przyspieszoną arytmetykę
# After Fractadyne restarted itself because the graphics driver reset the GPU.
app-toast-gpu-recovered = Przywrócono działanie po resecie urządzenia graficznego — otwarto widok początkowy, bo poprzednia lokalizacja była zbyt wymagająca do renderowania.

## Short notices (toasts)

# When the autopilot reached a view that could not be rendered and put the last good view back.
app-toast-autopilot-view-restored = Autopilot zatrzymany: widok stał się nieużyteczny i został przywrócony (zapisano w dzienniku)
# $error: the system's reason, as it gives it (often in English).
app-toast-gradients-save-failed = Nie udało się zapisać gradientów: { $error }
# $error: the reason, as the program gives it (often in English).
app-toast-gradients-serialize-failed = Nie udało się zserializować gradientów: { $error }
# $name: the name the user gave the gradient.
app-toast-gradient-saved = Zapisano gradient „{ $name }”.
# $error: the reason, as the program gives it (often in English).
app-toast-bookmarks-serialize-failed = Nie udało się zserializować zakładek: { $error }
# $error: the system's reason, as it gives it (often in English).
app-toast-bookmarks-save-failed = Nie udało się zapisać zakładek: { $error }
# Jumping to a named point given by exact expressions. $name: the point's name (in English).
app-toast-coordinates-failed = { $name }: nie udało się obliczyć współrzędnych
# $zoom: the magnification, already formatted (1,234.5 or 1.2e148).
app-toast-random-location = Losowa lokalizacja · { $zoom }×
# Importing a Kalles Fraktaler .kfr location. $error: the reason (in English).
app-toast-import-failed = Import nie powiódł się: { $error }
# $zoom: the magnification, already formatted.
app-toast-location-loaded = Wczytano lokalizację @ { $zoom }×

## Go to location: why a jump could not be made (one line per field that failed)

# "Re" is the dialog's real-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-re = Re: { $error }
# "Im" is the dialog's imaginary-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-im = Im: { $error }
# The imaginary field was given a complex value. $error: the evaluator's reason (in English).
app-goto-error-im-real = Im: { $error } — wpisz część urojoną jako liczbę rzeczywistą, bez i
# "Zoom" is the dialog's magnification field.
app-goto-error-zoom-empty = Zoom: wpisz powiększenie, np. 1e50 lub 250000
# $zoom: what was typed, in quotes.
app-goto-error-zoom = Zoom: nie można odczytać { $zoom } — wpisz dodatnie powiększenie, np. 1e50 lub 250000
app-goto-error-invalid = Nieprawidłowe dane — sprawdź współrzędne i zoom.

## Finding a minibrot or Misiurewicz point (the M key and the Go to dialog)

app-feature-needs-holomorphic = Wyszukiwanie minibrotów wymaga rodziny holomorficznej (Mandelbrot / Multibrot).
app-feature-not-in-julia = Wyszukiwanie minibrotów przeszukuje zbiór Mandelbrota — wyłącz tryb Julii, aby z niego skorzystać.
app-feature-busy = Wyszukiwanie struktury już trwa — tylko jedno naraz.
app-feature-searching = Szukanie najbliższego minibrota…
app-feature-mandelbrot-only = Wyszukiwanie struktur działa tylko dla zbioru Mandelbrota.
app-feature-no-pair = W budżecie wyszukiwania nie znaleziono punktu Misiurewicza — punkt może leżeć głębiej, niż sięga wyszukiwanie przy tym widoku, albo środek może nie leżeć w pobliżu spirali lub punktu rozgałęzienia.
# The name of what was found, used as $feature in the messages below. $period: the minibrot's period (a whole number).
app-feature-minibrot = minibrot o okresie { $period }
# $preperiod and $period: the point's two whole numbers (k, p). $octaves: how often the view repeats when zooming in (2 decimals); $twist: the turn per repeat in degrees.
app-feature-misiurewicz-lambda = Misiurewicz ({ $preperiod },{ $period }) — powtarza się co { $octaves } oktawy, skręt { $twist }°
# $preperiod and $period: the point's two whole numbers (k, p).
app-feature-misiurewicz = Misiurewicz ({ $preperiod },{ $period })
# Added after the messages below when the fixed iteration count is too low for the new depth. $have: the iterations set now; $typical: what the depth typically needs.
app-feature-iter-note = ⚠ Iteracje są ustalone na { $have }; ta głębokość zwykle wymaga ~{ $typical } — zwiększ je (lub włącz automatyczne iteracje), jeśli widok renderuje się płasko.
# $feature: what was found (see above). $zoom and $asked: magnifications, already formatted; $octaves: the solver's depth limit.
app-feature-zoomed-capped = Przybliżono do: { $feature } — { $zoom }×. Wyznaczanie jest ograniczone do { $octaves } oktaw, więc zatrzymało się tutaj zamiast przy { $asked }×.
# $feature: what was found. $zoom: the magnification, already formatted. $seconds: how long the search took (1 decimal).
app-feature-zoomed = Przybliżono do: { $feature } — { $zoom }× ({ $seconds }s)
# $feature: what was found. $seconds: how long the search took (1 decimal).
app-feature-snapped = Dociągnięto do środka: { $feature } ({ $seconds }s)
app-feature-no-minibrot = Nie znaleziono środka minibrota w pobliżu widoku — przybliż się do któregoś.
# $feature: the point asked for, e.g. Misiurewicz (3,1). $widths: how many view-widths away, already formatted. $octaves: how far to zoom out.
app-feature-just-outside = Znaleziono punkt { $feature } tuż poza tym widokiem — w odległości { $widths } szerokości widoku. Oddal o około { $octaves } { $octaves ->
        [one] oktawę
        [few] oktawy
        [many] oktaw
       *[other] oktawy
    }, aby był na ekranie, albo ustaw większy przedokres, aby znaleźć strukturę w tej skali.
# $feature: the point asked for. $widths: how many view-widths away, already formatted (may be 2^187). $octaves: how far to zoom out.
app-feature-far = Znaleziono punkt { $feature }, ale w odległości { $widths } szerokości widoku — ta struktura ma skalę o wiele GRUBSZĄ niż ten widok. Aby do niej dotrzeć, ODDAL o około { $octaves } { $octaves ->
        [one] oktawę
        [few] oktawy
        [many] oktaw
       *[other] oktawy
    } albo spróbuj większego przedokresu, aby znaleźć strukturę w tej skali.
# $feature: the point asked for. $residual: a small number in scientific notation.
app-feature-not-preperiodic = { $feature } nie pasuje tutaj do orbity (residuum { $residual }) — wyczyść oba pola, aby wykryć parę z widoku.
# $feature: the point asked for. "k/p" are the dialog's two fields.
app-feature-not-converged = W pobliżu widoku nie uzyskano zbieżności do punktu { $feature } — podejdź bliżej albo spróbuj innych k/p.
app-feature-bad-request = Przedokres i okres muszą być dodatnie.

## Share location dialog and location files

app-location-empty = Nie ma czego wczytać (lub tekst jest za duży).
app-location-not-fractadyne = To nie jest lokalizacja Fractadyne.
# $zoom: the magnification, already formatted. $note: what in the text was ignored or repaired (in English).
app-share-loaded-note = Wczytano @ { $zoom }× — { $note }.
# $error: the system's reason (often in English).
app-share-save-failed = Zapis nie powiódł się: { $error }
app-share-saved = Zapisano.
app-share-saved-rendering-thumb = Zapisano. Renderowanie miniatury…
app-share-saved-no-thumb = Zapisano — bez miniatury (dla tego widoku nie udało się jej wyrenderować).
app-share-saved-with-thumb = Zapisano z miniaturą.
# $error: the system's reason (often in English).
app-share-saved-thumb-failed = Zapisano, ale nie udało się dodać miniatury: { $error }
app-share-loaded-into-box = Wczytano do pola — przejrzyj, a potem kliknij Zastosuj.
# Reading a file failed (a location or a palette). $error: the system's reason (often in English).
app-read-failed = Odczyt nie powiódł się: { $error }
app-share-too-large = Plik jest za duży (to nie lokalizacja .fdn?).

## File-dialog filter names (the extensions are added by the code)

app-filter-kfr = Lokalizacja Kalles Fraktaler
app-filter-fdn = Lokalizacja Fractadyne
app-filter-map = Paleta Fractint / KF
app-filter-ugr = Gradient Ultra Fractal
app-filter-ase = Adobe Swatch Exchange
app-filter-ggr = Gradient GIMP
app-filter-all = Wszystkie pliki

## Importing and saving palette files (shown under the gradient editor)

# The reason a file was refused before reading it.
app-gradient-map-too-large = o wiele za duży jak na paletę .map
app-gradient-ugr-too-large = o wiele za duży jak na plik palety .ugr
app-gradient-ase-too-large = o wiele za duży jak na listę próbek .ase
app-gradient-ggr-too-large = o wiele za duży jak na gradient .ggr
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported-vga =
    Zaimportowano { $count } { $count ->
        [one] wpis
        [few] wpisy
        [many] wpisów
       *[other] wpisu
    } z pliku { $file } — tabela VGA 6-bitowa (biel to 252, nie 255), zachowana bez zmian, aby zgadzała się z wynikami samego programu Fractint.
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported =
    Zaimportowano { $count } { $count ->
        [one] wpis
        [few] wpisy
        [many] wpisów
       *[other] wpisu
    } z pliku { $file } jako twarde pasma.
# $error: the reader's reason (in English).
app-gradient-map-invalid = To nie jest paleta .map: { $error }
# $name: the gradient's name in the file; $file: the file's name.
app-gradient-ugr-imported = Zaimportowano „{ $name }” z pliku { $file }.
# $file: the file's name; $count: how many gradients it holds.
app-gradient-ugr-pick =
    { $file } zawiera { $count } { $count ->
        [one] gradient
        [few] gradienty
        [many] gradientów
       *[other] gradientu
    } — wybierz jeden poniżej.
# $error: the reader's reason (in English).
app-gradient-ugr-invalid = To nie jest plik palety .ugr: { $error }
# $used: the swatches kept; $total: the swatches in the file.
app-gradient-ase-imported-some = Zaimportowano { $used } z { $total } próbek, rozmieszczonych równomiernie.
# $count: the swatches in the file.
app-gradient-ase-imported = Zaimportowano próbki: { $count }; rozmieszczono je równomiernie i połączono płynnymi przejściami.
# $error: the reader's reason (in English).
app-gradient-ase-invalid = To nie jest lista próbek .ase: { $error }
app-gradient-nothing-to-save = Nie ma jeszcze własnego gradientu do zapisania.
# $file: the saved file's name.
app-gradient-ggr-saved = Zapisano „{ $file }” — punkty środkowe, krzywe przejścia i przestrzenie barw pozostały nienaruszone.
# $file: the saved file's name; $count: the segments whose curve could not be kept.
app-gradient-ggr-saved-lossy =
    { $count ->
        [one] Zapisano „{ $file }”, ale { $count } segment wykorzystuje krzywą Béziera, której format .ggr nie obsługuje. Segment zapisano jako przejście liniowe; kopia w bibliotece zachowuje tę krzywą.
        [few] Zapisano „{ $file }”, ale { $count } segmenty wykorzystują krzywe Béziera, których format .ggr nie obsługuje. Zapisano je jako przejścia liniowe; kopia w bibliotece zachowuje te krzywe.
        [many] Zapisano „{ $file }”, ale { $count } segmentów wykorzystuje krzywe Béziera, których format .ggr nie obsługuje. Zapisano je jako przejścia liniowe; kopia w bibliotece zachowuje te krzywe.
       *[other] Zapisano „{ $file }”, ale liczba segmentów używających krzywych Béziera, których format .ggr nie obsługuje, wynosi { $count }. Zapisano je jako przejścia liniowe; kopia w bibliotece zachowuje oryginalne krzywe.
    }
# $path: the file; $error: the system's reason (often in English).
app-gradient-write-failed = Nie udało się zapisać { $path }: { $error }
# $name: the gradient's name; $count: its segments.
app-gradient-ggr-imported =
    Zaimportowano „{ $name }” — { $count } { $count ->
        [one] segment
        [few] segmenty
        [many] segmentów
       *[other] segmentu
    } z punktami środkowymi, krzywymi przejścia i przestrzeniami barw.
# $error: the reader's reason (in English).
app-gradient-ggr-invalid = To nie jest gradient .ggr: { $error }

## The eyedropper's banner across the top of the screen while picking a colour

# $hex: the colour under the pointer, as #rrggbb.
app-eyedropper-banner-color = Pobieranie koloru  ·  { $hex }  ·  kliknij, aby go pobrać, Esc anuluje
app-eyedropper-banner = Pobieranie koloru  ·  kliknij, aby go pobrać, Esc anuluje

## The gradient editor window

app-gradient-title = Edytor gradientu
# Before the two view buttons, Bar and Ring.
app-gradient-view = Widok
app-gradient-view-bar = Pasek
app-gradient-view-bar-hover = Prosty pasek — najlepszy do precyzyjnego rozmieszczania węzłów
app-gradient-view-ring = Pierścień
app-gradient-view-ring-hover = Pierścień, ponieważ paleta jest CYKLICZNA: połączenie na górze to szew, który renderer przekracza przy każdym obiegu, a gradient, który na pasku wygląda dobrze, może mieć tam ostrą krawędź.
app-gradient-add-lane-hover = Kliknij w dowolnym miejscu tej linii, aby dodać tam węzeł. + dodaje węzeł w największej przerwie.
app-gradient-strip-hover = Przeciągnij znacznik, aby przesunąć jego węzeł · przeciągnij sam pasek, aby OBRÓCIĆ cały gradient z zawinięciem na szwie · kliknij dwukrotnie, aby dodać węzeł, kliknij znacznik prawym przyciskiem, aby go usunąć · klawisze strzałek przesuwają zaznaczenie (z Shift — precyzyjnie).
app-gradient-remove-selected-hover = Usuń zaznaczony węzeł
app-gradient-ring-hover = Przeciągnij sam pierścień, aby OBRÓCIĆ cały gradient — to, co minie szew, wraca z drugiej strony · przeciągnij znacznik, aby przesunąć jego węzeł · kliknij zewnętrzny tor, aby dodać tam węzeł, lub ⊕ pośrodku, aby dodać węzeł w największej przerwie · ⊖ usuwa zaznaczony węzeł. Pusty znak na górze to szew, w którym paleta cykliczna się zawija; pozostaje na miejscu, bo JEST połączeniem, a kolory obracają się pod nim.
app-gradient-empty = Nie ma jeszcze własnego gradientu — „Kopiuj szablon…”, import lub zapisany gradient poniżej rozpoczyna nowy, a jego węzły i krzywe segmentów pojawią się tutaj.
# $count: the entries of an imported palette, more than the editor shows stop by stop.
app-gradient-too-many = Zaimportowane wpisy: { $count } — za dużo, by edytować je węzeł po węźle. Użyj podglądu powyżej albo Kopiuj szablon, aby zacząć od nowa.
# The selected segment's heading. $index: its number; $total: how many there are; $left and $right: where it starts and ends, 0.000–1.000.
app-gradient-segment-heading = Segment { $index } z { $total }  ·  { $left } → { $right }
# A label before the list of blend curves.
app-gradient-curve = Krzywa
app-gradient-curve-hover = Jak szybko kolor przechodzi przez ten segment. „Zakrzywiona” jest kształtowana przez punkt środkowy — przy punkcie środkowym 0.50 jest dokładnie liniowa.
# The blend curves (GIMP's kinds), in a narrow list: keep them short.
app-gradient-blend-linear = Liniowa
app-gradient-blend-curved = Zakrzywiona
app-gradient-blend-sine = Sinusoidalna
app-gradient-blend-sphere-increasing = Sferyczna ↑
app-gradient-blend-sphere-decreasing = Sferyczna ↓
app-gradient-blend-bezier = Bézier
# Hover on Bézier in the list of blend curves.
app-gradient-bezier-exact-hover = Dwa punkty kontrolne, które można przeciągać w obu kierunkach. Ta krzywa jest już sześcienna, więc konwersja niczego nie zmienia.
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-visible-hover = Dwa punkty kontrolne, które można przeciągać w obu kierunkach. Konwersja APROKSYMUJE bieżącą krzywą — tę z dokładnością do { $error }, co jest wyraźnie widoczne.
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-barely-hover = Dwa punkty kontrolne, które można przeciągać w obu kierunkach. Konwersja APROKSYMUJE bieżącą krzywą — tę z dokładnością do { $error }, co jest ledwo widoczne.
# A label before the list of colour spaces (RGB, HSV).
app-gradient-space = Przestrzeń
# A label before the Bézier presets beside it.
app-gradient-ease = Łagodzenie
app-gradient-ease-hover = Szablony ustawiają oba punkty kontrolne; dla innych kształtów przeciągnij je na krzywej. Krzywa Béziera nie ma punktu środkowego — jej uchwyty określają, gdzie osiąga połowę.
# Bézier presets, as in CSS easing: short buttons.
app-gradient-ease-linear = Liniowe
app-gradient-ease-in = Początek
app-gradient-ease-out = Koniec
app-gradient-ease-in-out = Oba końce
app-gradient-midpoint = Punkt środkowy
app-gradient-midpoint-hover = Gdzie przejście osiąga połowę, jako ułamek tego segmentu. 0.50 to środek.
# A button that puts the midpoint back in the middle of the segment.
app-gradient-centre = Wyśrodkuj
app-gradient-centre-hover = Z powrotem do 0.50
app-gradient-no-hue = ⚠ jeden koniec tego segmentu nie ma odcienia
app-gradient-no-hue-hover = Jeden koniec jest czarny, biały lub szary, więc przejście zaczyna się od czerwieni i obiega całe koło barw. To dozwolone i czasem pożądane — ale właśnie dlatego środek może mieć kolor, którego nie ma żaden z końców. Pasek pod krzywą pokazuje, co się dzieje.
app-gradient-segment-start-hover = Kolor na POCZĄTKU tego segmentu
app-gradient-segment-end-hover = Kolor na KOŃCU tego segmentu
app-gradient-bytes-hover = Czerwony, zielony i niebieski w zakresie 0–255
app-gradient-floats-hover = Ten sam kolor w zakresie 0–1, czyli tak, jak przechowuje go renderer
app-gradient-canvas-bezier-hover = Krzywa przejścia zaznaczonego segmentu. Przeciągnij dowolny punkt kontrolny — po x I po y — aby zmienić jej kształt. Blade linie to 0 i 1; krzywa może wyjść poza nie, a obcinany jest wtedy kolor, nie krzywa.
app-gradient-canvas-hover = Krzywa przejścia zaznaczonego segmentu: jak szybko kolor przechodzi od lewego węzła do prawego. Przeciągnij pierścień, aby przesunąć punkt środkowy, albo przełącz krzywą na Bézier, aby uzyskać punkt kontrolny poruszający się w obu kierunkach.
# The selected stop's heading. $index: its number; $total: how many there are.
app-gradient-stop-heading = Węzeł { $index } z { $total }
app-gradient-stop-color-hover = Kolor w tym węźle — kliknij próbkę, aby go wybrać, lub wpisz obok wartość szesnastkową
app-gradient-stop-bytes-hover = Czerwony, zielony i niebieski w zakresie 0–255 — bajty, jakimi zapisywany jest ten kolor
# The eyedropper button.
app-gradient-pick = Pobierz
app-gradient-pick-hover = Pipeta — pobierz kolor z dowolnego miejsca na ekranie. Przesuń wskaźnik na wybrany kolor i kliknij; Esc anuluje.
# Written before the stop's position (0.000–1.000) in its number field: a short abbreviation of "position".
app-gradient-stop-pos = poz.
app-gradient-nudge-hover = Przesuń lekko ten węzeł (przytrzymaj Shift, aby krok był drobniejszy)
app-gradient-remove = Usuń
app-gradient-remove-hover = Usuń ten węzeł (lub kliknij ⊖ pod nim)
app-gradient-add-stop = Dodaj węzeł
app-gradient-add-stop-hover = Podziel najszerszy segment (lub kliknij linię nad gradientem)
# A menu of the built-in palettes, to start from a copy of one.
app-gradient-copy-preset = Kopiuj szablon…
app-gradient-paste = Wklej…
app-gradient-paste-hover = Zaimportuj paletę z kolorów szesnastkowych lub trójek RGB 0–255
app-gradient-save-ggr = Zapisz .ggr…
app-gradient-save-ggr-hover = Zapisz ten gradient do pliku GIMP .ggr — czytelnego dla programów GIMP, Krita i Inkscape oraz dla tego edytora
# A menu of the palette file formats (.map, .ugr, .ggr, .ase).
app-gradient-import = Importuj
app-gradient-import-map-hover = Wczytaj plik palety .map programu Fractint / Kalles Fraktaler (wiersze R G B, po jednym na wpis)
app-gradient-import-ugr-hover = Wczytaj plik gradientów Ultra Fractal .ugr (plik .ugr zwykle zawiera wiele gradientów — wybiera się jeden)
app-gradient-import-ggr-hover = Wczytaj gradient GIMP .ggr z nienaruszonymi punktami środkowymi, krzywymi przejścia i przestrzeniami barw
app-gradient-import-ase-hover = Wczytaj listę próbek Adobe (.ase). Próbki nie mają pozycji, więc są rozmieszczane równomiernie i łączone płynnymi przejściami.
# The gradient's name, for the library of saved gradients.
app-gradient-name = Nazwa
# Shown greyed in the empty name field.
app-gradient-name-hint = mój gradient
# The save button, when a gradient of that name is already saved.
app-gradient-update = Aktualizuj
app-gradient-save = Zapisz
app-gradient-save-hover = Zapisz ten gradient w bibliotece z nienaruszonymi krzywymi i przestrzeniami barw
app-gradient-save-needs-name-hover = Najpierw nadaj gradientowi nazwę
# A menu of the saved gradients. $count: how many there are.
app-gradient-saved-menu = Zapisane ({ $count })
app-gradient-nothing-saved = Nic jeszcze nie zapisano
app-gradient-delete-saved-hover = Usuń ten zapisany gradient
# $count: the gradient's segments.
app-gradient-rich =
    { $count ->
        [one] Bogaty gradient: { $count } segment ma punkty środkowe, krzywe przejścia lub przestrzenie barw — więcej, niż pomieści zwykła lista węzłów.
        [few] Bogaty gradient: { $count } segmenty mają punkty środkowe, krzywe przejścia lub przestrzenie barw — więcej, niż pomieści zwykła lista węzłów.
        [many] Bogaty gradient: { $count } segmentów ma punkty środkowe, krzywe przejścia lub przestrzenie barw — więcej, niż pomieści zwykła lista węzłów.
       *[other] Bogaty gradient: { $count } segmentu ma punkty środkowe, krzywe przejścia lub przestrzenie barw — więcej, niż pomieści zwykła lista węzłów.
    }
app-gradient-convert = Konwertuj na edytowalne węzły
app-gradient-convert-hover = Zastępuje go zwykłą listą węzłów. To ODRZUCA punkty środkowe, krzywe przejścia i przejścia przez odcienie, których lista węzłów nie pomieści.
# Above the list of gradients in an Ultra Fractal file. $count: how many; $file: the file's name.
app-gradient-ugr-list = Gradienty w pliku { $file }: { $count }
app-gradient-close-list = Zamknij listę
app-gradient-seamless = Bez szwu (koniec pasuje do początku)
app-gradient-seamless-hover = Paleta jest cykliczna, więc pozycje 1 i 0 sąsiadują na ekranie. Gdy ta opcja jest włączona, kolor ostatniego węzła jest utrzymywany równy pierwszemu — widok pierścienia pokazuje połączenie na górze.
app-gradient-ends-differ = ⚠ końce się różnią
app-gradient-ends-differ-hover = Kolor na końcu gradientu nie jest kolorem na początku, więc paleta cykliczna pokazuje ostrą krawędź w miejscu zawinięcia.
app-gradient-hard-bands = Twarde pasma (bez przejść między wpisami)
app-gradient-hard-bands-hover = Semantyka Fractint / .map: każdy wpis to jednolity kolor. Wyłączenie łączy je w płynny gradient.
app-gradient-paste-intro = Wklej kolory szesnastkowe (#ff8800) lub trójki 0–255 (255 136 0), oddzielone przecinkami, spacjami lub nowymi wierszami:
# Applies the pasted colours.
app-gradient-apply = Zastosuj
# Empties the paste box.
app-gradient-clear = Wyczyść
# $used: the colours kept; $total: the colours pasted; $limit: the most that are kept.
app-gradient-paste-imported-some = Zaimportowano { $used } z { $total } kolorów, próbkowanych równomiernie z całej listy (limit to { $limit }).
# $count: the colours pasted.
app-gradient-paste-imported = Zaimportowano kolory: { $count }.
# $error: why the pasted text could not be read (in English).
app-gradient-paste-unreadable = Nie udało się tego odczytać: { $error }
# A hint at the bottom of the editor. $count: the palette's entries.
app-gradient-bands-hint = Pasma: { $count } · w kolejności z pliku; gdy pasma są włączone, pozycje są ignorowane.
# A hint at the bottom of the editor, in the ring view. $count: the gradient's stops.
app-gradient-ring-hint = Węzły: { $count } · kliknij zewnętrzny tor, aby dodać tam węzeł · ⊕ dodaje w największej przerwie, ⊖ usuwa zaznaczony węzeł · przeciągnij znacznik, aby go przesunąć · znak na górze to szew
# A hint at the bottom of the editor, in the bar view. $count: the gradient's stops.
app-gradient-bar-hint = Węzły: { $count } · kliknij linię nad gradientem, aby dodać tam węzeł · przeciągnij znacznik, aby go przesunąć · ⊖ lub prawy przycisk usuwa
app-gradient-cancel = Anuluj
app-gradient-cancel-hover = Odrzuć wszystkie zmiany wprowadzone od otwarcia tego okna i zamknij je
app-gradient-ok = OK
app-gradient-ok-hover = Zachowaj te zmiany i zamknij edytor

## Shared

# The button that closes the Help window and the "Faster deep zoom" window.
app-close = Zamknij

## The "Faster deep zoom" window (Help menu): the optional accelerated build

app-accel-title = Szybszy głęboki zoom
app-accel-active-heading = Działa kompilacja przyspieszona
app-accel-active-body = Orbity referencyjne głębokiego zoomu są obliczane za pomocą MPFR/GMP, co na tym etapie jest od 2,5 do 6,4 raza szybsze niż w kompilacji standardowej — to pauza, zanim głęboki widok zacznie nabierać szczegółów.
# $status: which arithmetic library is in use and its version (in English).
app-accel-in-use = Używana arytmetyka: { $status }
app-accel-fallback-heading = Kompilacja przyspieszona — używa wbudowanej arytmetyki
app-accel-fallback-body = To jest kompilacja przyspieszona (MPFR), ale nie znaleziono jej bibliotek matematycznych, więc używa wbudowanej arytmetyki (astro-float). Obrazy są identyczne; wolniejsza jest tylko pauza, zanim głęboki widok nabierze szczegółów.
# Followed by the folder the program runs from, on its own line.
app-accel-fallback-dlls = Aby korzystać z szybszej ścieżki, umieść libmpfr-6.dll i libgmp-10.dll (oraz libgcc_s_seh-1.dll, libwinpthread-1.dll) obok fractadyne.exe, a następnie uruchom program ponownie:
app-accel-warning-hidden = Ostrzeżenie o tym przy uruchomieniu jest ukryte.
app-accel-show-again = Pokaż je ponownie
app-accel-offer-heading = Opcjonalna kompilacja jest w głębi od 2,5 do 6,4 raza szybsza
app-accel-offer-body = Głęboki zoom spędza dużo czasu na obliczaniu orbit referencyjnych na CPU — to pauza, zanim głęboki widok zacznie nabierać szczegółów. Opcjonalna kompilacja robi to za pomocą bibliotek MPFR/GMP, co jest od 2,5 do 6,4 raza szybsze, a im głębiej, tym większa różnica. Wszystko inne jest takie samo.
app-accel-offer-identical = Obrazy są IDENTYCZNE CO DO BAJTU — to ta sama matematyka obliczana szybszą biblioteką, sprawdzona na każdym wzorze fraktala i na całym korpusie porównawczym głębokiego zoomu.
app-accel-offer-shared = Ustawienia, zapisana sesja i lokalizacje są wspólne dla obu kompilacji, więc można swobodnie się między nimi przełączać i niczego nie trzeba importować.
# Shown on Windows.
app-accel-separate-windows = To osobny plik do pobrania, ponieważ używanych przez nią bibliotek nie da się zbudować kompilatorem, którego używa standardowa kompilacja dla Windows, a do tego mają one inną licencję (GNU LGPL v3) niż sam Fractadyne.
# Shown on other systems.
app-accel-separate = To osobny plik do pobrania, ponieważ używane przez nią biblioteki mają inną licencję (GNU LGPL v3) niż sam Fractadyne.
app-accel-download = Pobierz dla tej wersji
app-accel-all-releases = Wszystkie wydania
app-accel-all-releases-hover = Jeśli bezpośredni link zwraca 404, ta wersja nie ma jeszcze kompilacji przyspieszonej — wybierz tu najbliższą.
# Shown on Windows.
app-accel-install-windows = Rozpakuj archiwum i uruchom fractadyne.exe z tego folderu, zostawiając obok niego pliki .dll.
# Shown on Linux.
app-accel-install-linux = Rozpakuj archiwum i uruchom ./fractadyne z tego folderu. Program używa systemowych bibliotek GMP i MPFR (libgmp10 / libmpfr6), które ma niemal każdy system desktopowy — zobacz README-ACCELERATED.txt w pakiecie.
# Shown on other systems.
app-accel-install-other = Dla tej platformy nie zbudowano jeszcze pakietu przyspieszonego; można go zbudować ze źródeł z --features fractadyne-core/rug.

## The Help window's frame (its sections are under help-)

app-help-title = Pomoc Fractadyne

## Checking for updates

app-toast-up-to-date = To jest najnowsza wersja.
# $error: why the check failed (in English).
app-toast-update-failed = Sprawdzanie aktualizacji nie powiodło się: { $error }
# The window that offers a newer version.
app-update-title = Dostępna aktualizacja
# $version: the new version (0.3.0-beta.20). $channel: the token "stable" or "beta", in English; a translation can choose its words by it ({ $channel -> [beta] … *[other] … }).
app-update-version =
    { $channel ->
        [beta] Dostępna jest wersja Fractadyne { $version } (beta)
       *[other] Dostępna jest wersja Fractadyne { $version } (stabilna)
    }
# $current: the version running now. $track: the update track the user follows (its name in the language in use).
app-update-running = Zainstalowana wersja: { $current }  ·  { $track }
app-update-download-intro = Pobierz najnowsze wydanie z GitHub:
app-update-download = Pobierz z GitHub
app-update-later = Przypomnij później

## The second graphics card (shown in its window, after "In use:" or "Could not start:")

# The second device is the same card as the window's. $name: the card's name.
app-worker-same-card = ta sama karta, { $name }
# Why the second device could not start.
app-worker-thread-stopped = wątek urządzenia się zatrzymał

## Export status (shown in the Export image window)

app-export-gpu-unavailable = GPU niedostępny
app-export-reference-aborted = Eksport nie powiódł się: przerwano budowę referencji.

## The performance panel (View ▸ Performance panel)
## Its row labels sit in a monospace column about 14 characters wide: keep them short (a longer label pushes its value to the right).

app-perf-fps = FPS
app-perf-fps-hover = Klatki wyświetlane na sekundę, z wygładzonego odstępu między klatkami. Gdy widok się ustala, wiersz pokazuje zamiast tego postęp dopracowania — ustalający się widok nie działa w tempie, jakie sugerowałaby tu liczba.
# What the FPS row shows while a view is settling. $done and $total: tiles; $eta: the time left, as m:ss.
app-perf-refining-tiles = dopracowanie { $done }/{ $total } (~{ $eta })
app-perf-refining-arming = dopracowanie (przygotowanie)
# $percent: how much is done, with one decimal.
app-perf-refining-percent = dopracowanie { $percent }%
app-perf-building-reference = budowa referencji
app-perf-idle = bezczynny
app-perf-frame = klatka
app-perf-frame-hover = Wygładzony czas rzeczywisty między klatkami — nie suma wierszy poniżej. Obejmuje czekanie na wyświetlenie, więc przy ustalonym, bezczynnym widoku odzwierciedla rytm odświeżania, a nie jakąkolwiek pracę.
app-perf-cpu = cpu
app-perf-cpu-hover = Czas wątku głównego w jednej aktualizacji: przygotowanie parametrów i zarządzanie pamięcią podręczną referencji. Ten wiersz skacze, gdy referencja się przebudowuje, a skok w tym miejscu to odczuwalne przycięcie.
app-perf-gpu-idle = gpu/bezczyn.
app-perf-gpu-idle-hover = Czas klatki minus czas CPU wątku głównego. To praca GPU, gdy widok się renderuje, ale przy ustalonym widoku głównie bezczynne oczekiwanie — nazwa mówi o obu, bo samo to odejmowanie nie pozwala panelowi ich rozróżnić.
app-perf-timing = pomiar
# How frame cost is measured, the timing row's value. TIMESTAMP_QUERY is a name.
app-perf-timing-starved = czas rzeczywisty (brak znaczników)
app-perf-timing-gpu = znaczniki czasu GPU
app-perf-timing-no-query = czas rzeczywisty (brak TIMESTAMP_QUERY)
app-perf-timing-hover = Sposób pomiaru kosztu klatki. Znaczniki czasu GPU są dokładnym źródłem; zapasowy pomiar czasu rzeczywistego włącza się, gdy pomiary przestają napływać, a budżet wyceniany czasem rzeczywistym wielokrotnie źle dobierał rozmiar głębokich klatek — kolor czerwony oznacza tu „marginesy bezpieczeństwa są teraz szacunkowe”.
app-perf-mode = tryb
app-perf-mode-hover = Ścieżka arytmetyczna, którą wyrenderowano ostatnią klatkę: bezpośrednia (bez orbity referencyjnej), perturbacja df32 lub floatexp — ścieżka o rozszerzonym zakresie używana powyżej około 1e28x i jedyna, na której działa BLA.
app-perf-eff-iter = efekt. iter.
app-perf-eff-iter-hover = Iteracje faktycznie użyte w ostatniej klatce, po skalowaniu z głębokością i ewentualnym ograniczeniu w ruchu — a nie ustawienie bazowe w sekcji Jakość, które jest tylko punktem wyjścia.
app-perf-precision = precyzja
app-perf-precision-hover = Liczba bitów mantysy, z jaką obliczana jest orbita referencyjna. Rośnie z głębokością; 64 bity to minimum dla płytkich widoków, a nie osiągnięty limit.
app-perf-orbit-len = dł. orbity
# $cap: the most orbit samples this GPU holds, already formatted, or the text of app-perf-uncapped.
app-perf-orbit-len-hover = Próbki orbity referencyjnej. Ten GPU mieści najwyżej { $cap } — zbliżanie się do tej wartości to praktyczna ściana głębokości: poza nią widok na żywo nie rozdzieli głębszych lokalizacji, które wymagają dłuższej orbity.
app-perf-uncapped = (bez limitu)
app-perf-sa-skip = pominięte SA
app-perf-sa-skip-hover = Iteracje pominięte przez aproksymację szeregiem, zanim zaczęła się praca na pikselach. Więcej znaczy szybciej i niczego nie zmienia w obrazie; wiersza nie ma, gdy nic nie pominięto.
app-perf-aa = aa
app-perf-aa-hover = Supersampling: każdy piksel jest renderowany z tylu próbek na oś, więc 2x kosztuje cztery razy więcej pracy.
app-perf-dual = podwójny
app-perf-dual-hover = Czy działa dzielony widok Mandelbrot/Julia. Gdy tak, każdy koszt w tym panelu obejmuje OBA panele.
app-perf-zoom = zoom
app-perf-zoom-hover = Powiększenie względem pełnego widoku. Tryb arytmetyki powyżej zmienia się przy około 1e4x i ponownie przy około 1e28x.
app-perf-iterate = iteracja
# $target and $lethal: times in milliseconds, already formatted.
app-perf-iterate-hover = Zmierzony czas GPU ostatniego zlecenia iteracji. Kontroler celuje w { $target } ms; utrzymujące się odczyty w pobliżu { $lethal } ms to pasmo, w którym ta klasa sprzętu traciła urządzenie.
app-perf-steps = kroki/s
app-perf-steps-hover = Kroki iteracji na sekundę na GPU, w miliardach. Widoki z dużą ilością wnętrza działają wolniej na krok, niż sugeruje sama liczba, więc to odczyt przepustowości, a nie wynik punktowy.
app-perf-budget = budżet
# After the budget's value while it is still converging; the hover below quotes it.
app-perf-budget-settling = (ustalanie)
app-perf-budget-hover = Adaptacyjny budżet kroków na zlecenie — zabezpieczenie kontrolera przed klatką tak długą, że sterownik zresetowałby GPU. „ustalanie” oznacza, że wciąż zbiega na podstawie nielicznych odczytów i nie jest jeszcze wiarygodnym pułapem.
app-perf-rss = rss
# $mb and $peak: memory in megabytes, already formatted.
app-perf-rss-value = { $mb } MB (szczyt { $peak })
app-perf-rss-hover = Pamięć rezydentna procesu i najwyższa wartość osiągnięta w tej sesji. Głębokie budowy referencji przekraczały w praktyce 2 GB, choć nic innego na ekranie o tym nie mówiło; kolor bursztynowy powyżej trzech czwartych pamięci RAM systemu.
app-perf-gpu-est = gpu szac.
app-perf-gpu-est-hover = Szacowana pamięć zajęta na GPU: orbity referencyjne, drzewa BLA, tekstury iteracji, naprzemienne bufory stanu fragmentów podczas dopracowania i migawka bramki wyświetlania. Złożona ze znanych rozmiarów alokacji; rzeczywistej wartości ze sterownika nie da się odczytać w sposób przenośny.
app-perf-julia-re = julia c.re
app-perf-julia-re-hover = Część rzeczywista parametru c, który renderuje panel Julii — punkt pod kursorem, chyba że jest przypięty.
app-perf-julia-im = julia c.im
app-perf-julia-im-hover = Część urojona parametru c, który renderuje panel Julii.
app-perf-julia-c = julia c
# The julia c row's value when the parameter is held fixed.
app-perf-pinned = przypięty
app-perf-pinned-hover = Parametr jest ustalony w stałym punkcie, więc ruch kursora nie zmienia już panelu Julii.
app-perf-c-panel = c/panel
app-perf-c-panel-hover = Szerokość przestrzeni c obejmowanej przez cały panel Mandelbrota. Gdy jest dużo mniejsza niż jeden piksel Julii, najechanie zmienia c, ale zbiór Julii wygląda bez zmian — to normalne przy głębokim zoomie, a nie zawieszenie.
app-perf-ref-recompute = przelicz. ref.
app-perf-ref-recompute-hover = Czas trwania ostatniego przeliczenia orbity referencyjnej. To pauza, zanim głęboki widok zacznie nabierać szczegółów, i etap, który przyspiesza opcjonalna kompilacja przyspieszona.
app-perf-recompute-rate = przelicz./s
app-perf-recompute-rate-hover = Orbity referencyjne ZAINSTALOWANE na sekundę — te, których wynik został faktycznie użyty. Porównaj z wierszem poniżej: różnica między nimi to praca, która została wyrzucona.
app-perf-builds-rate = bud. ref./s
app-perf-builds-rate-hover = Budowy referencji URUCHOMIONE na sekundę, łącznie z tymi, których wynik jest odrzucany — slot wyprzedzający, do którego zanurzanie nigdy nie dociera, albo odrzucone rozszerzenie. Kosztują pełny czas CPU i nie pojawiają się nigdzie indziej: pętla wstępnego pobierania wykonywała kiedyś około 400 budów na sekundę przez sześć minut, podczas gdy przelicz./s pokazywało 2. Wartości jednocyfrowe są normalne; duża różnica między tymi dwoma wierszami oznacza, że praca jest wyrzucana.
app-perf-recompute-total = przelicz. razem
app-perf-recompute-total-hover = Przeliczenia referencji od uruchomienia. Przydatne jako tempo w znanym przedziale czasu; liczba rosnąca, gdy nic się nie porusza, to sygnał, który warto zbadać.

## Help: the contents list (Help & reference…, F1)

# Each section's name in the contents list (about 165 px wide: keep them short), and its heading.
help-section-overview = Przegląd
help-section-navigation = Nawigacja
help-section-expressions = Wyrażenia współrzędnych
help-section-options = Kolorowanie i opcje
help-section-fractals = Fraktale
help-section-how-it-works = Jak to działa
help-section-command-line = Wiersz poleceń
help-section-shortcuts = Skróty
help-section-hardware = Zalecany sprzęt
# Credits to the prior work Fractadyne builds on.
help-section-acknowledgments = Podziękowania
# The third-party license notices.
help-section-licenses = Licencje
help-section-about = O programie

## Help ▸ Overview

help-overview-intro = Fractadyne to natywny eksplorator fraktali stworzony z myślą o ultragłębokim zoomie i szybkości. Rysuje fraktale typu „escape-time” (czasu ucieczki) — obrazy powstające przez powtarzanie jednego prostego wzoru w każdym pikselu.
help-overview-what-is = Czym jest fraktal czasu ucieczki?
help-overview-what-is-text = Dla każdego piksela program wielokrotnie wykonuje wzór, taki jak z → z² + c, zaczynając od zera. Jeśli bieżąca wartość zawsze pozostaje mała, piksel należy do zbioru i jest rysowany ciemno. Jeśli w końcu rośnie bez ograniczeń („ucieka”), piksel leży poza zbiorem, a jego kolor zapisuje, ile kroków to zajęło. Nieskończenie szczegółowa granica między „pozostaje” a „ucieka” to właśnie fraktal.
help-overview-can-do = Co można robić
help-overview-can-pan = Przesuwać i przybliżać praktycznie bez ograniczeń (położenie jest dokładne na każdej głębokości).
help-overview-can-families = Przełączać się między 25 wbudowanymi rodzinami fraktali (dziesięć, a pięć z nich także w wyższych potęgach), własnymi wzorami, Life i L-systemami, a każdą rodzinę czasu ucieczki oprócz Newtona oglądać jako zbiór Julii.
help-overview-can-recolor = Zmieniać kolory za pomocą gotowych lub własnych gradientów i kilku metod kolorowania.
help-overview-can-lighting = Dodawać oświetlenie reliefu 3D i świecące kontury brzegu.
help-overview-can-minibrots = Dociągać do minibrotów, zapisywać miejsca w zakładkach i eksportować obrazy w wysokiej rozdzielczości.
help-overview-can-autopilot = Pozwolić autopilotowi autozoomu samodzielnie zanurzać się w szczegóły.
help-overview-can-share = Udostępniać dowolne miejsce jako mały fragment tekstu „.fdn” lub plik.
help-overview-can-tours = Uruchamiać przeloty ze skryptów i test wydajności sprzętu.
help-overview-first-steps = Pierwsze kroki
# "Seahorse Valley" is a place in the Navigate menu: use its name as that menu shows it. A and F1 are keys.
help-overview-first-steps-text = Otwórz menu Nawigacja i wybierz „Seahorse Valley”, a potem obróć kółko myszy, aby przybliżyć. Przeciągnij, aby przesunąć widok. Albo po prostu naciśnij A i pozwól, by autopilot sam się zanurzał. Naciśnij F1 w dowolnej chwili, aby wrócić do tej pomocy.

## Help ▸ Coordinate expressions
## The expressions, numbers and function names here are typed as shown in every language:
## keep them exactly as written. Menu items in quotation marks are the menus' own labels.

help-expr-intro = Wszędzie tam, gdzie Fractadyne przyjmuje współrzędną, zamiast liczby dziesiętnej można wpisać wyrażenie: pola Re / Im w Nawigacja → „Przejdź do lokalizacji…” (i każde pole jego trybu biegunowego), wiersze center_re / center_im lokalizacji .fdn, re / im skryptu przelotu oraz --center w wierszu poleceń. Wyrażenie jest obliczane z taką liczbą cyfr, jakiej wymaga zoom, a okno Przejdź do lokalizacji przechowuje je razem z widokiem — dzięki temu punkt wpisany jako 1/3 albo -0.5 + 0.25*cos(pi/4) pozostaje dokładnie w tym punkcie bez względu na głębokość zoomu, podczas gdy liczba dziesiętna zaczęłaby dryfować, gdy skończyłyby się jej cyfry.
help-expr-examples = Przykłady
help-expr-ex-third = Dokładna jedna trzecia. Przy przybliżaniu pojawia się więcej cyfr.
help-expr-ex-neck = Szyjka Seahorse Valley — korzeń bąbla okresu 2.
help-expr-ex-circle = Punkt na okręgu o promieniu 1/4 wokół -0.5 (tryb biegunowy składa to wyrażenie automatycznie).
help-expr-ex-bulb = Re korzenia bąbla okresu 3; jego Im to sin(2*pi/3)/2 - sin(4*pi/3)/4. Nawigacja → „Punkty dokładne (wyrażenia)” ma je wbudowane.
help-expr-ex-complex = Cała wartość zespolona wpisana w pole Re wypełnia zarówno Re, jak i Im.
help-expr-ex-combine = Funkcje i stałe można dowolnie łączyć.
help-expr-ex-powers = Potęgi: 2^-20 to 1/1048576. -2^2 to -4 — minus jest stosowany po potędze.
help-expr-ex-root = -2, rzeczywisty pierwiastek sześcienny. Pierwiastki parzystego stopnia z liczby ujemnej są odrzucane; sqrt(-1) daje i.
help-expr-ex-scientific = Notacja naukowa działa także wewnątrz wyrażeń.
help-expr-numbers = Liczby i operatory
# The decimal point in an expression is '.' in every language: keep that rule as it is.
help-expr-numbers-decimals = Liczby dziesiętne z opcjonalnym wykładnikiem: 0.25, -1.5e-30, .5. Tylko cyfry — bez spacji i przecinków grupujących wewnątrz liczby, a separatorem dziesiętnym jest zawsze '.'.
help-expr-numbers-imaginary = Dodaj przyrostek i, aby zapisać liczbę urojoną (0.25i), albo użyj samego i: 3 + 4*i.
# Starts with the operators themselves, two spaces apart.
help-expr-numbers-operators = +  -  *  /  oraz ^ (potęga; obliczana od prawej do lewej, więc 2^3^2 = 2^9). Nawiasy grupują. Nie ma niejawnego mnożenia: pisz 2*pi, a nie 2pi.
help-expr-numbers-case = Wielkość liter w nazwach nie ma znaczenia: PI, Pi i pi to ta sama stała.
help-expr-constants = Stałe
help-expr-functions = Funkcje
help-expr-functions-angles = Kąty są w radianach. Dla stopni pomnóż przez pi/180 — cos(45*pi/180) — albo przełącz okno Przejdź do lokalizacji w tryb biegunowy i wybierz jednostkę stopnie.
help-expr-refused = Co jest odrzucane i dlaczego
help-expr-refused-fractional = Liczba ujemna podniesiona do potęgi ułamkowej, np. (-2)^(1/2): wynik jest zespolony i jest kilka pierwiastków do wyboru. Użyj root(x, n) lub cbrt(x) dla pierwiastka rzeczywistego albo sqrt(x) dla głównego pierwiastka zespolonego.
help-expr-refused-complex = Argument zespolony funkcji o wartościach rzeczywistych (sin(i), ln(1+i)). Argument zespolony przyjmują tylko sqrt i abs; operatory arytmetyczne i potęgi całkowite działają na wartościach zespolonych.
help-expr-refused-domain = Błędy dziedziny: ln lub log liczby niedodatniej, asin lub acos poza -1…1, 0 do potęgi ujemnej, dzielenie przez zero.
help-expr-refused-bounds = Argumenty sin, cos, tan, exp lub ^ poza ±2^32 oraz zagnieżdżenie głębsze niż 32 poziomy: wklejona współrzędna to niezaufane dane, a takie wyrażenia zajęłyby nieograniczony czas.
help-expr-refused-grammar = Żadnych zmiennych, funkcji definiowanych przez użytkownika, porównań ani warunków. Gramatyka jest celowo mała, aby zapisana lokalizacja znaczyła to samo w każdej wersji.
help-expr-no-parse = Jeśli wyrażenie się nie parsuje
help-expr-no-parse-text = Komunikat wskazuje pole, mówi, co znaleziono i na którym znaku, a — przy typowych pomyłkach — co wpisać zamiast tego: typograficzny znak minus (−) wklejony z dokumentu, × lub ÷, przecinek dziesiętny, brakujący * między liczbą a nazwą, niezamknięty nawias albo funkcję zapisaną bez nawiasów. Nic nigdy nie jest odczytywane połowicznie: jeśli jakiejkolwiek części tekstu nie da się zrozumieć, cała współrzędna jest odrzucana, a nie po cichu skracana.

## Help ▸ Navigation
## Menu paths (Navigate → "Go to location…") and quoted items name the interface's own
## menus and controls: use their translations. Key names (Ctrl, Shift, Space, Esc) stay.

# A heading: the mouse controls (also in Shortcuts).
help-mouse = Mysz
# The left column of a controls table: a gesture or key. Keep key names (Shift, Space, Ctrl) as they are.
help-key-left-drag = Przeciąganie lewym przyciskiem
help-nav-left-drag = Przesuwanie widoku.
help-key-mouse-wheel = Kółko myszy
help-nav-wheel = Przybliżanie/oddalanie w stronę kursora; szybsze obracanie przybliża dalej na jeden ząbek. Gdy włączone jest „Przyciągaj zoom do struktur”, przybliżanie w pobliżu minibrota ściąga zoom na niego.
help-key-shift-drag = Shift+przeciąganie
help-nav-shift-drag = Zoom prostokątem — narysuj prostokąt, a widok przybliży się tak, by go wypełnić.
# $icon: the click-to-zoom toolbar button's symbol.
help-key-click-zoom = Zoom kliknięciem ({ $icon })
help-nav-click-zoom = Włącz przycisk celownika na pasku narzędzi (widok pojedynczy): kliknięcie lewym przyciskiem zanurza w punkt o ustawiony współczynnik, prawym cofa. Przeciąganie nadal przesuwa; współczynnik (2–100×) wybiera się w sekcji Nawigacja w prawym panelu. Backspace cofa kliknięcie.
help-nav-continuous = Ciągły zoom i powrót
help-key-hold-space = Przytrzymaj Space
help-nav-hold-space = Płynne przybliżanie zakotwiczone w kursorze.
help-key-hold-shift-space = Przytrzymaj Shift+Space
help-nav-hold-shift-space = Płynne oddalanie.
help-nav-continuous-text = Przycisk powrotu do widoku początkowego animuje łagodny lot z powrotem do pełnego widoku. „Szybkość zoomu” w prawym panelu ustawia tempo ciągłego zoomu.
help-nav-autopilot = Autopilot
help-nav-autopilot-text = Naciśnij A (lub Narzędzia → „Autozoom (autopilot)”), aby zanurzać się bez użycia rąk: autopilot wybiera punkt na krawędzi fraktala, gdzie szczegóły są najgęstsze, preferując środek widoku, i płynnie w niego wlatuje, stopniowo przesuwając go na środek ekranu. Trzyma się tego celu, dopóki pozostaje on interesujący, i płynnie skręca ku bogatszej strukturze, gdy taka się pojawi. Każda czynność nawigacyjna — lub Esc — go zatrzymuje. „Priorytet autozoomu” w prawym panelu (lub w menu Narzędzia) wybiera, co jest ważniejsze: Szybkość przybliża w ustawionym tempie i odświeża ekran, gdy tylko gotowa jest klatka ze szczegółami; Jakość pokazuje tylko w pełni dopracowane klatki w pełnej rozdzielczości i w razie potrzeby spowalnia zoom, aby obraz nigdy nie powiększył się zbytnio przed nadejściem następnego. Tak czy inaczej ekran nigdy nie pokazuje pustej ani jednokolorowej klatki: odświeżenie, które nie ma jeszcze żadnych szczegółów, pozostaje ukryte za poprzednim. „Cel autozoomu” wybiera, w co celuje autopilot: Szczegóły (najbogatsza krawędź w widoku, wybierana na nowo po drodze) albo Punkt Misiurewicza — znajduje najbliższy punkt Misiurewicza, wyznacza go na głębokość limitu zanurzania, stojąc w miejscu, a potem zanurza się dokładnie w niego; struktura wokół takiego punktu powtarza się co kilka oktaw w nieskończoność, więc to zanurzanie nigdy nie trafia w ślepy zaułek (tylko zbiór Mandelbrota).
help-nav-history = Historia i precyzyjne ruchy
help-nav-undo = Cofnij poprzedni widok.
help-nav-redo = Ponów.
help-nav-goto = Nawigacja → „Przejdź do lokalizacji…” pozwala odczytać, wpisać, wkleić lub skopiować dokładny środek i zoom (pełna precyzja, dowolna głębokość). Pola środka przyjmują więcej niż zwykłe liczby: ułamki takie jak „1/3” i wyrażenia zbudowane z pi, tau, e, sqrt i sin/cos/tan — więc „-0.5 + 0.25*cos(pi/4)” to poprawna współrzędna. Wyrażenie jest przechowywane razem z widokiem i obliczane ponownie z precyzją, jakiej wymaga każda głębokość, więc tak wpisany punkt pozostaje dokładny bez względu na to, jak głęboko się przybliża (zaokrąglona liczba dziesiętna w końcu by nie pozostała). Tryb biegunowy (przesunięcie + r ∠ θ, w stopniach / radianach / obrotach) składa takie wyrażenie automatycznie. Pełna lista funkcji i stałych z przykładami to sekcja „Wyrażenia współrzędnych” tej pomocy (otwiera ją przycisk ? w oknie). Nawigacja → Zakładki zapisuje i przywołuje lokalizacje.
help-nav-exact-points = Nawigacja → „Punkty dokładne (wyrażenia)” przenosi do punktów zdefiniowanych dokładnie, a nie zaokrągloną liczbą dziesiętną — korzeni bąbli głównej kardioidy, w których przyczepia się każdy pąk okresu q, zapisanych jako wyrażenia cos/sin. Ponieważ wyrażenie podróżuje razem z widokiem, w taki punkt można przybliżać bez dryfowania od niego.
help-nav-share = Nawigacja → „Udostępnij lokalizację…” zapisuje cały widok — fraktal, środek w pełnej precyzji, zoom i kolorowanie — jako zwarty fragment „.fdn”, który można skopiować (Kopiuj), wkleić z powrotem i zastosować (Zastosuj) albo zapisać/wczytać jako plik, aby dokładnie odtworzyć miejsce (lub wysłać je komuś).
help-nav-open = Plik → „Otwórz widok lub lokalizację…” otwiera bezpośrednio każdy z nich: wyeksportowany PNG/EXR (przywracany jest osadzony w nim widok), plik lokalizacji „.fdn” lub „.kfr” programu Kalles Fraktaler.
help-nav-finding = Znajdowanie szczegółów
help-nav-minibrots = Minibroty to maleńkie kopie całego zbioru ukryte wzdłuż brzegu. Ustaw jeden mniej więcej na środku i naciśnij M (lub Narzędzia → „Znajdź minibrot i przybliż do niego”), aby metodą Newtona dociągnąć dokładnie do jego środka, odczytać jego okres i przybliżyć prosto do jego własnej skali — często o wiele rzędów wielkości w jednym kroku. Jeśli widok jest już głębiej niż minibrot, przesuwa się tylko środek.
help-nav-minimap = Widok → „Minimapa” pokazuje małą mapę całego zbioru ze znacznikiem „jesteś tutaj” i bieżącą głębokością zoomu; kliknięcie mapy przenosi do wskazanego obszaru.

## Help ▸ Coloring & options
## Names in quotation marks are the controls' own labels in the right-hand panel.

help-options-intro = Wszystkie te ustawienia znajdują się w prawym panelu (i są zachowywane między sesjami).
help-options-palette = Paleta
help-options-palette-text = Paleta wybiera gotowy gradient, własny (Własna) lub tryb dwukolorowy. „Edytuj gradient…” otwiera edytor, w którym każdy węzeł ma kolor i pozycję (0–1); można dodawać węzły, skopiować szablon jako punkt wyjścia lub zaimportować plik palety.
help-options-paste = „Wklej…” w edytorze gradientu importuje paletę znalezioną gdzie indziej. Przyjmuje kolory szesnastkowe z początkowym # lub bez niego (#ff8800, ff8800 lub krótkie #f80) oraz zwykłe trójki RGB 0–255, takie jak „255 136 0” — format plików .map programów Fractint i Kalles Fraktaler — oddzielone przecinkami, spacjami lub nowymi wierszami. Zachowywanych jest do 256 kolorów; dłuższa lista jest równomiernie próbkowana do 256, z zachowaniem pierwszego i ostatniego, aby kształt gradientu przetrwał.
help-options-import-map = „Importuj ▸ .map…” obok wczytuje plik palety .map programu Fractint lub Kalles Fraktaler. To tablice indeksowane liczbą iteracji, bez przejść między wpisami, a wynikające z tego ostre skoki kolorów tworzą klasyczny wygląd Fractinta — dlatego tak są importowane, z polem wyboru „Twarde pasma”, które zamiast tego łączy te same kolory w płynny gradient. Własne pliki Fractinta przechowują 6-bitowe wartości kolorów pomnożone przez cztery, więc ich biel to 252, a nie 255; Fractadyne informuje o tym przy imporcie i pozostawia wartości dokładnie tak, jak zapisano, bo zapisane obrazy samego Fractinta mają to samo 252.
help-options-duotone = Dwutonowa odwzorowuje wartość na płynną rampę dwóch kolorów (Cień → Światło). Binarna (zbiór) to płaski widok dwukolorowy — jeden jednolity kolor dla punktów wewnątrz zbioru, drugi dla zewnętrznych, bez gradientu — najczytelniejszy sposób, by zobaczyć kształt zbioru.
help-options-cycle = Cykl ustawia, ile razy gradient powtarza się w zakresie iteracji (gęstsze lub rzadsze pasma). Przesunięcie obraca cały gradient.
help-options-normalize = „Normalizuj głębokie kolory” dopasowuje paletę do zakresu wartości ucieczki rzeczywiście obecnych w widoku. Głębokie widoki obejmują setki tysięcy iteracji, a bez tego stały cykl zawija gradient tysiące razy między sąsiednimi pikselami, zamieniając poprawne szczegóły w drobne plamki.
help-options-log-scale = „Logarytmiczna skala kolorów” rozkłada paletę według logarytmu wartości ucieczki zamiast liniowo. Liczby iteracji do ucieczki z głębokością skupiają się przy górnym końcu, więc dopasowanie liniowe zużywa większość gradientu na cienką warstwę przylegającą do brzegu, a resztę zostawia niemal płaską; skala logarytmiczna to rozciąga i utrzymuje stabilniejsze kolory w filmie z zoomu, gdy zakres rośnie.
help-options-animate = Animacja zmienia kolory w czasie — Do przodu, Wstecz lub Ping-pong zmieniają przesunięcie; Losowe gradienty nieustannie tworzą płynnie przechodzące w siebie, harmonijne gradienty. Szybkość ustala liczbę cykli (lub przemian) na sekundę; „Losuj gradient” losuje nowy w trybie losowym.
help-options-method = Metoda kolorowania — jak dane stają się kolorem
# The coloring methods, by the names the Coloring panel gives them, each with what it does.
help-method-smooth = Gładka iteracja
help-method-smooth-text = Klasyczne płynne pasma według czasu ucieczki.
help-method-stripe = Średnia pasków
help-method-stripe-text = Płynące pasma z kąta orbity (suwak Gęstość pasków).
help-method-triangle = Nierówność trójkąta
help-method-triangle-text = Drobna tekstura z tego, gdzie każdy krok wypada między ograniczeniami.
help-method-trap = Pułapka orbity
help-method-trap-text = Odległość orbity od kształtu (punkt/krzyż/okrąg); koloruje też wnętrze.
help-method-distance = Estymacja odległości
help-method-distance-text = Cieniuje według bliskości brzegu.
help-method-decomposition = Dekompozycja
help-method-decomposition-text = Komórki z końcowego kąta ucieczki.
help-options-relief = Oświetlenie reliefu 3D
help-options-relief-text = Cieniuje powierzchnię według nachylenia brzegu (pochodnej), dając wytłoczony, oświetlony wygląd. Kąt światła ustala kierunek; Relief — siłę (mniej = ostrzej); „Obracaj światło” je animuje. Tylko rodziny holomorficzne (Mandelbrot / Multibrot).
help-options-glow = Poświata odległości
help-options-glow-text = Jasne pasma konturowe, które w pobliżu brzegu zagęszczają się w świecące włókna. Poświata to stopień zmieszania, Szerokość pasm — odstęp, a „Animuj poświatę” wprawia je w ruch.
help-options-quality = Jakość i iteracje
help-options-quality-text = Iteracje to maksymalna liczba kroków, po której piksel jest traktowany jako należący do zbioru; „Autoskalowanie” zwiększa ją automatycznie podczas przybliżania (głębsze szczegóły wymagają więcej). Antyaliasing nadpróbkowuje nieruchome obrazy (2×–8×), gdy widok się ustali, poskramiając drobny zewnętrzny „pył”.
help-options-other = Inne
help-options-other-text = Szybkość zoomu ustawia tempo ciągłego zoomu; limit klatek na sekundę ogranicza liczbę klatek; „Oba, połączone” pokazuje obok siebie zbiór Mandelbrota i jego zbiór Julii (kursor na bieżąco ustawia parametr Julii).

## Help ▸ Fractals
## Fractal family names (Mandelbrot, Multibrot, Tricorn, Burning Ship, Celtic, Buffalo,
## Phoenix, Newton) are names: keep them as written, as the formulas. Formula syntax
## (z = z^3 - p1*z + c, sqr, |z|, if / elseif / endif, p1-p5, maxit) is typed as shown.

help-fractals-intro = Każda rodzina iteruje wzór, w którym z zaczyna od 0, a c wyznacza piksel (czas ucieczki), chyba że zaznaczono inaczej. z = x + iy to liczba zespolona.
help-fractals-mandelbrot = z → z² + c. Oryginał. Zbiór to wszystkie c, których orbita pozostaje ograniczona; jest spójny, a jego brzeg jest tak pomarszczony, że ma wymiar Hausdorffa 2.
help-fractals-multibrot = z → z^d + c dla potęgi d = 3, 4, 5. Wyższe potęgi dodają płaty: zbiór ma symetrię obrotową rzędu (d−1) (Multibrot 3 — rzędu 2, 4 — rzędu 3, 5 — rzędu 4).
help-fractals-tricorn = z → conj(z)² + c, gdzie conj(x + iy) = x − iy. Sprzężenie czyni go antyholomorficznym, co daje symetrię rzędu 3 i charakterystyczne zakrzywione „pazury”.
help-fractals-burning-ship = z → (|x| + i|y|)² + c — przed podniesieniem do kwadratu bierze się wartości bezwzględne (część rzeczywista = x²−y²+cx, urojona = 2|xy|+cy). Nieanalityczny; głębokie zoomy odsłaniają struktury przypominające statki (tradycyjnie oglądane do góry nogami).
help-fractals-celtic = Jak Mandelbrot, ale z wartością bezwzględną części rzeczywistej: część rzeczywista = |x²−y²| + cx, urojona = 2xy + cy. Tworzy motywy w kształcie serc i tarcz.
help-fractals-buffalo = Wartość bezwzględna obu części z²: część rzeczywista = |x²−y²| + cx, urojona = |2xy| + cy — skrzyżowanie Celtic i Burning Ship.
# A heading: the families at powers above 2 (the Fractal menu's "Higher powers").
help-fractals-powers = Wyższe potęgi
# ("Burning Ship  3 4 5") is how the Fractal menu shows such a row: keep it as it is.
help-fractals-powers-text = Multibrot 6, 7 i 8 (z^d + c) oraz Burning Ship, Tricorn, Celtic i Buffalo w potęgach 3, 4 i 5: te same zagięcia w wyższej potędze. Burning Ship d bierze wartości bezwzględne części z, a potem potęgę; Tricorn d najpierw sprzęga; Celtic d i Buffalo d zaginają część rzeczywistą (a w Buffalo także urojoną) z^d. Menu Fraktal i lista rozwijana pokazują je po jednej rodzinie w wierszu („Burning Ship  3 4 5”). Każda z nich obsługuje głęboki zoom tak jak jej odpowiednik w potędze 2, a Multibrot 6, 7 i 8 obsługują też aproksymację szeregiem i wyszukiwanie minibrotów (M), tak jak Multibrot 3–5. Multibrot 8 ucieka przy promieniu 128 zamiast 256, przy którym jego następny krok wyszedłby poza zakres pojedynczej precyzji.
help-fractals-phoenix = z → z² + c + p·z(n-1), gdzie z(n-1) to poprzednia iteracja, a p jest stałą (p = -0.5). Człon pamięci tworzy włókna przypominające płomienie.
help-fractals-newton = Metoda Newtona dla pierwiastków z³ − 1 = 0: z → z − (z³−1)/(3z²). Zamiast według czasu ucieczki piksele są kolorowane według tego, do którego z trzech pierwiastków trzeciego stopnia z jedynki zbiega iteracja (baseny przyciągania), i jak szybko; splątane granice basenów to fraktal.
help-fractals-julia = Tryb Julii
help-fractals-julia-text = Dla każdej rodziny oprócz Newtona można przełączyć się na zbiór Julii: zamiast zaczynać z od 0 przy c = piksel, ustala się c (parametr), a z zaczyna od piksela. Zbiór Julii to granica między punktami startowymi, które pozostają ograniczone, a tymi, które uciekają, dla tego ustalonego c. W trybie „Oba, połączone” ruch kursora nad panelem Mandelbrota ustawia c na bieżąco.
help-fractals-deep = Obsługa głębokiego zoomu
help-fractals-deep-text = Mandelbrot, Multibrot 3/4/5 i Tricorn obsługują perturbacyjny głęboki zoom na ekstremalnych głębokościach (floatexp). Burning Ship, Celtic i Buffalo są nieanalityczne (biorą wartości bezwzględne), więc używają perturbacji uwzględniającej znaki; działa ona teraz także w zakresie floatexp, sięgając daleko poza dawny limit df32 ~1e28× (rzadkie plamki w pobliżu zagięć abs czekają na korekcję glitchy z wieloma referencjami). Phoenix używa perturbacji dwuczłonowej i także obsługuje głęboki zoom w zakresie floatexp (bez SA/BLA, więc jest cięższy niż Mandelbrot). Wyższe potęgi obsługują głęboki zoom w ten sam sposób — perturbacja każdej potęgi podąża za własnym łańcuchem podnoszeń do kwadratu; Multibrot 3–8 obsługują też aproksymację szeregiem i BLA, tak jak Mandelbrot, a Tricorn, Burning Ship, Celtic i Buffalo w każdej potędze obsługują BLA (bez szeregu), które przenosi wnętrze głębokiego widoku zamiast iterować je krok po kroku. Tylko Newton używa ścieżki bezpośredniej, ostrej do ~1e6×; własne wzory obsługują głęboki zoom, chyba że poniższa sekcja Własne wzory mówi inaczej.
help-fractals-custom = Własne wzory
help-fractals-custom-text = Fraktal ▸ Własny wzór… przyjmuje krok wzoru w wyrażeniach w stylu Fractinta: z = z^3 - p1*z + c albo kilka instrukcji (t = sqr(z), potem z = t*t + c). z to iteracja, c piksel (stała Julii w trybie Julii), p1-p5 parametry; |z| to kwadrat modułu, jak we Fractincie, i dostępne są zwykłe funkcje (sqr, abs, conj, real, imag, cabs, flip, recip, ident, exp, log, sqrt, sin, cos, tan, cotan, sinh, cosh, tanh, cotanh, asin, acos, atan, asinh, acosh, atanh, cosxx oraz floor, ceil, trunc i round, które zaokrąglają każdą część). log to logarytm naturalny, jak we Fractincie i w analizie zespolonej: to, co wiele podręczników zapisuje jako ln (nie ma logarytmu dziesiętnego). Wzór zapisany z ln, cot, coth, Re lub Im nie zostanie odczytany; sprawdzanie składni wskazuje funkcję, której należy użyć, przycisk przepisuje wzór, a wpisanie ln podpowiada log. Każdy krok wzoru obsługuje głęboki zoom przez perturbację, z tą samą arytmetyką o rozszerzonym zakresie powyżej 1e28x co wbudowane fraktale, z wyjątkiem kroku z potęgą, której wykładnik zależy od z lub c (np. z^c): taki renderuje się ścieżką bezpośrednią, ostrą, dopóki jeden krok c w pojedynczej precyzji nie obejmie piksela, czyli do około 1e4x–1e5x, a z dala od początku układu płycej. Okno wzoru informuje, który przypadek zachodzi. log, sqrt i potęgi niecałkowite przyjmują wartości główne, które skaczą na ujemnej półosi rzeczywistej. Perturbacja dokładnie odtwarza ten skok, ale piksel, którego orbita przecina linię cięcia tam, gdzie orbita referencji jej nie przecina, jest od tego momentu liczony dalej z mniej więcej pojedynczą precyzją, więc głębokie widoki, których orbity leżą po obu stronach cięcia, pokazują tam szum (zmierzono: czysto przy 1e5x na osi rzeczywistej dla sqrt(z^4 + c), szum dla wzorów z log już przy 1e4x). Każdy krok jest iterowany w całości (bez aproksymacji szeregiem), więc głęboki widok z dużą liczbą iteracji renderuje się wolniej niż wbudowany fraktal. W pobliżu brzegu wzór z funkcjami ma chaotyczne orbity, a długą orbitę śledzi się tam w pojedynczej precyzji tylko z grubsza: przy 2000 iteracji jedna trzecia pikseli sin(z) + c różni się od dokładnej orbity, przy 60 prawie żaden. Widoki, sesje i eksportowane obrazy zachowują tekst wzoru.
help-fractals-custom-sections = Sekcje Fractinta też działają. Instrukcje przed „:” to sekcja inicjalizacji, wykonywana raz na piksel; porównanie jako ostatnia instrukcja to własny test ucieczki wzoru — iteracja trwa, dopóki jest spełnione (|z| <= 100 && |z - 1| > 0.000001 zatrzymuje orbitę, która ucieka lub zbiega do 1); if (...), elseif (...), else i endif wybierają instrukcje; a zmienna odczytana w kroku przed jej ustawieniem ma wartość z poprzedniego kroku (z sekcji inicjalizacji lub 0). Porównania porównują części rzeczywiste i dają 1 lub 0, && i || je łączą, a przypisanie też jest wartością (a = b = pixel). pixel, p1-p5, pi i e można przypisywać — zachowują własną wartość, dopóki nie zostaną przypisane; maxit to liczba iteracji, a ismand ma wartość 1 (postać Mandelbrota wzoru). Wzór z którymkolwiek z tych elementów lub z zaokrąglaniem renderuje się ścieżką bezpośrednią, bez głębokiego zoomu. Biblioteka wzorów > Importuj odczytuje też pliki .frm Fractinta. Każdy wpis, który da się odczytać, staje się wzorem, a jego wiersz mówi, co odczyt zmienił: fn1-fn4 przyjmują domyślne funkcje Fractinta (sin, sqr, sinh, cosh); zmienna c Fractinta zostaje przemianowana na c_, bo tutaj c to piksel; nazwa, której nic nie ustawia, ma wartość 0, jak we Fractincie; spacje wewnątrz nazwy są pomijane (end if to endif), tak jak pomija je Fractint. Komunikat importu podaje liczbę wpisów, których nie da się odczytać, i najczęstszą przyczynę: zmienne ekranu i widoku (whitesq, scrnpix, scrnmax, center, magxmag, rotskew), lastsqr i liczby losowe nie są obsługiwane. Klasyki Fractinta z Kolekcji (Magnet, Nova, Spider, Manowar i inne) są zapisane z użyciem tych sekcji.
help-fractals-custom-editor = W polu wzoru nawiasy mają kolor zależny od poziomu zagnieżdżenia; para obok kursora jest wyróżniona, a nawias bez pary jest pokazany na czerwono (nawiasy w komentarzach się nie liczą, bo parser je ignoruje). Nazwy uzupełniają się podczas pisania: po dwóch literach lista proponuje funkcje, nazwy i własne zmienne zaczynające się tak samo. Up i Down wybierają, Tab lub Enter zatwierdza (funkcja pojawia się z nawiasem otwierającym), Esc zamyka listę, a Enter po nazwie wpisanej w całości nadal zaczyna nowy wiersz.
help-fractals-custom-textbook = Podręcznik (przełącznik nad wzorem) pokazuje wzór złożony tak, jak składa go podręcznik, i pozwala go edytować w miejscu. Pisz jak w tekście: / tworzy ułamek z wyrazu przed nim, ^ wykładnik (+, - lub = wpisane na końcu wykładnika wychodzą z niego), ( nawiasy lub wywołanie po nazwie funkcji, | kreski, a przecinek w nawiasach stałą zespoloną. Strzałki przesuwają po wzorze, Up i Down między licznikiem a jego mianownikiem, Tab do następnego pustego pola; Backspace na początku ułamka lub pary nawiasów zaznacza go, a drugie naciśnięcie usuwa go, zachowując zawartość. Kopiowanie, wklejanie i cofanie działają jak w polu tekstowym, nazwy uzupełniają się tak jak tam, a klawiatura ekranowa wpisuje do złożonego wzoru (□² podnosi do kwadratu to, co jest przed kursorem, klawisz funkcji obejmuje zaznaczenie). Każda zmiana jest zapisywana z powrotem do tekstu, więc oba widoki zawsze zawierają ten sam wzór; nieedytowany wiersz zachowuje swoje odstępy i komentarz. Notacja nigdy nie zmienia tego, co jest obliczane: |z| to kwadrat modułu i jest pokazywane jako |z|², cabs(z) jako |z|, a exp(z) pozostaje exp(z). Wiersz, którego nie da się odczytać, jest pokazany na czerwono; kliknij go, aby edytować go jako tekst. LaTeX kopiuje wzór jako źródło LaTeX, tak jak jest złożony, a gdy Podręcznik jest włączony, biblioteka wzorów także pokazuje swoje wzory w postaci złożonej.
help-fractals-custom-numbers = Liczby we wzorze (0.1, 1/3, pi, parametry) są odczytywane jako wartości podwójnej precyzji, około 16 cyfr, a głęboka orbita referencyjna iteruje dokładnie z tymi wartościami. Dwa wzory równe na papierze mogą więc różnić się w głębi: 0.1*z mnoży przez liczbę double najbliższą 0.1, a z/10 dzieli przez dokładne 10, i oba obrazy rozchodzą się powyżej około 1e15x (zmierzono: 5% pikseli przy 1e15x, jedna trzecia przy 1e17x, wszystkie przy 1e20x). Każdy z obrazów jest poprawny dla własnego wzoru; zapisz ten, który chcesz udostępnić.
help-fractals-custom-library = Przycisk Zapisz w bibliotece w oknie zachowuje wzór i jego parametry pod nazwą (zapis pod nazwą, która już jest w bibliotece, aktualizuje ten wpis). Fraktal ▸ Biblioteka wzorów… wyświetla je, aby można je było zastosować, edytować, wyeksportować lub usunąć; Importuj dodaje wzory z pliku wzorów, nie zastępując żadnego: wzór, który już jest w bibliotece, jest pomijany, a nowy, którego nazwa jest zajęta, staje się „nazwa (2)”. Biblioteka to plik formulas.toml w folderze podanym w sekcji Pliki i dane (O programie), sam będący plikiem wzorów. Klatka kluczowa przelotu przenosi wzór jako formula = "..." z formula_params = [[re, im], ...], a Narzędzia > Przelot do bieżącego widoku zapisuje je dla własnego widoku.
help-fractals-custom-collection = Z Fractadyne dostarczana jest kolekcja 45 wzorów napisanych specjalnie dla niego: wyższe potęgi i zagięte warianty rodzin Burning Ship i Celtic, mieszanki i hybrydy dwukrokowe, zbiór Mandelbrota widziany przez inne odwzorowanie (odwrócony, logarytmiczno-biegunowy, przez sinus), wzory przestępne i wymierne oraz zbiory Julii. Są to Przykłady w oknie wzoru i karta Kolekcja w bibliotece wzorów, gdzie Kopiuj do moich dodaje wzór do własnych. Każdy ma widok początkowy, a Zastosuj tam przenosi: wzór otwarty w widoku początkowym często nie jest tam, gdzie jest jego obraz. Wzór zapisany podczas wyświetlania w ten sam sposób zachowuje bieżący widok, a Zastosuj do niego wraca.

## Help ▸ How it works

help-how-escape = Czas ucieczki i płynny kolor
help-how-escape-text = Każdy piksel iteruje wzór, dopóki jego moduł nie przekroczy promienia ucieczki albo nie zostanie osiągnięty limit iteracji. Sama surowa liczba kroków daje ostre pasma; dodanie części ułamkowej wyprowadzonej z końcowego modułu daje ciągły kolor bez pasm.
help-how-precision = Położenie w dowolnej precyzji
help-how-precision-text = Zwykłym liczbom 64-bitowym kończą się cyfry przy zoomie około 1e15×. Fractadyne przechowuje środek widoku w dowolnej precyzji, z liczbą cyfr rosnącą wraz z zoomem, więc lokalizacja nigdy się nie degraduje. Skala piksela jest również przechowywana z rozszerzonym wykładnikiem, więc nie zatrzymuje się też na limicie ~1e308× liczb 64-bitowych — głębokość ogranicza tylko cierpliwość, a nie jakikolwiek stały zakres liczb.
help-how-perturbation = Perturbacja
help-how-perturbation-text = Iterowanie każdego piksela w wysokiej precyzji byłoby zdecydowanie zbyt wolne. Zamiast tego jeden piksel referencyjny jest iterowany w wysokiej precyzji na CPU („orbita referencyjna”), a każdy inny piksel jest obliczany na GPU jako maleńka różnica δ od niego, w szybkiej niskiej precyzji: δz → 2·Z·δz + δz² + δc.
help-how-floatexp = Ekstremalna głębokość (floatexp)
help-how-floatexp-text = Powyżej około 1e28× nawet ta maleńka różnica wypada poza zakres 32 bitów (niedomiar), więc jest przechowywana jako mantysa plus osobny wykładnik całkowity („floatexp”), co znosi 32-bitową ścianę głębokości. Silnik przełącza się automatycznie: bezpośrednie obliczenia przy płytkich widokach, perturbacja przy głębokich i floatexp przy najgłębszych. Głębokość ogranicza wtedy precyzja współrzędnych i budżet iteracji, a nie stały limit — sprawdzone krzyżowo z Fraktaler-3 na korpusie referencyjnym 39 lokalizacji, z których najgłębsza ma 5.63e18003× (38 nieekstremalnych wierszy to rutynowa bramka, do 6.13e1105×), oraz wewnętrzną baterią testów spójności precyzji do 1e1000000×; dołączony przelot sięga ~1.3e420×.
help-how-reference = Wybór referencji i przebazowanie
help-how-reference-text = Referencja jest wybierana (z oceną w wysokiej precyzji) tak, aby jej orbita jak najdłużej pozostawała w obrębie widoku. Gdy różnica staje się zbyt duża, jest „przebazowywana” z powrotem na referencję, aby zachować dokładność.
help-how-series = Aproksymacja szeregiem
help-how-series-text = Przy głębokim zoomie wczesne iteracje ledwo odsuwają δ od referencji, więc można je obliczyć za jednym razem: δz jest aproksymowane krótkim wielomianem w δc (rzędu 3), którego współczynniki są iterowane wzdłuż referencji. Renderer przeskakuje do ostatniej iteracji, w której ten wielomian jest jeszcze dokładny dla całego widoku, a potem iteruje normalnie — oszczędzając pominięte kroki bez zmiany obrazu (przełącznik w Sterowanie ▸ Zaawansowane). Mandelbrot i Multibrot 3–8, najgłębszy zakres; nie z kolorowaniem średnią pasków / nierównością trójkąta / pułapką orbity / dekompozycją (te wymagają każdej iteracji). Tam, gdzie działa BLA, obejmuje już ono to wczesne pominięcie.
help-how-distance = Estymacja odległości i oświetlenie
help-how-distance-text = Śledzenie pochodnej dz/dc daje odległość każdego piksela od brzegu. Na tym opiera się oświetlenie reliefu 3D, poświata odległości i metoda kolorowania „Estymacja odległości” — wszystkie poprawne na każdej głębokości zoomu.
help-how-safety = Antyaliasing i bezpieczeństwo
help-how-safety-text = Nieruchome obrazy są nadpróbkowywane (2–8× na oś), gdy widok się ustali. Budżet pracy utrzymuje pojedyncze rysowanie GPU w granicach limitu watchdoga sterownika, zmniejszając przy ekstremalnych ustawieniach rozdzielczość (nigdy liczbę iteracji), dzięki czemu głębokie widoki zachowują szczegóły zamiast stawać się puste.

## Help ▸ Command line (the flags themselves stay English)

# The section's introduction. The flags listed below it are the command line itself, English in every language.
help-cli-intro = Fractadyne działa bez interfejsu przy renderowaniu, eksporcie filmów/przelotów, automatyzacji, sprawdzaniu obrazów wzorcowych i testach wydajności. Uruchomiony bez opcji otwiera interaktywny interfejs graficzny.

## Help ▸ Shortcuts

# What each mouse gesture and key does (the gestures are the Navigation section's).
help-shortcuts-pan = Przesuwanie
# A gesture: turning the mouse wheel.
help-key-wheel = Kółko
help-shortcuts-wheel = Zoom w miejscu kursora
help-shortcuts-box-zoom = Zoom prostokątem (narysuj prostokąt, aby go przybliżyć)
# $icon: the click-to-zoom toolbar button's symbol. "tool on": while that button is on.
help-key-click-tool = Kliknięcie (narzędzie { $icon } wł.)
help-shortcuts-click = Zanurzenie w punkt; prawy przycisk cofa
# A heading: the keyboard shortcuts.
help-shortcuts-keyboard = Klawiatura
help-shortcuts-space = Ciągłe przybliżanie / oddalanie
help-shortcuts-undo = Cofnij widok
help-shortcuts-redo = Ponów widok
help-shortcuts-minibrot = Znajdź minibrot i przybliż do niego (Newton-Raphson)
help-shortcuts-autopilot = Autopilot autozoomu (zanurzanie w stronę szczegółów; każda czynność go zatrzymuje)
# off / strip / panel / full: the panel's sizes, as View ▸ Live render panel names them.
help-shortcuts-live-panel = Panel renderowania na żywo: wyłączony / pasek / panel / pełny (najechanie na przyrząd wyjaśnia, co oznacza; × panelu go zamyka)
help-shortcuts-esc = Zatrzymaj autopilota / odtwarzany przelot lub wyjdź z pełnego ekranu
help-shortcuts-snapshot = Migawka do ostatniego folderu eksportu — zrzut ekranu widoku w postaci wyświetlanej lub pełny render z ustawieniami eksportu; pierwsze naciśnięcie pyta, które z nich, a Plik → Ustawienia → Migawka to zmienia.
help-shortcuts-help = Otwórz tę pomoc
# A heading: the shortcuts of the Game of Life, with the menu path to it.
help-shortcuts-life = Life (Fraktal → Automaty komórkowe → Life)
help-shortcuts-life-run = Uruchom / wstrzymaj wszechświat
help-shortcuts-life-step = Krok o jedno pokolenie / krok o skok (ustawiany w panelu Life)
help-shortcuts-life-draw = Narzędzie rysowania wł. / wył. (ołówek na pasku narzędzi)
# A gesture. "Draw on": while the Life draw tool is on.
help-key-click-drag-draw = Kliknięcie / przeciąganie (rysowanie wł.)
help-shortcuts-life-paint = Odwróć komórkę / maluj: pociągnięcie, które zaczyna się na martwej komórce, rysuje, a takie, które zaczyna się na żywej, wymazuje. Działający wszechświat wstrzymuje się podczas rysowania i rusza dalej po puszczeniu przycisku.
# $key: a keyboard shortcut (Ctrl+Z). "Draw on": while the Life draw tool is on.
help-key-when-drawing = { $key } (rysowanie wł.)
help-shortcuts-life-undo = Cofnij ostatnią edycję, wypełnienie, wyczyszczenie lub zmianę reguły

## Help ▸ Recommended hardware

help-hardware-intro = Fractadyne dzieli pracę między GPU (iteracja i kolorowanie każdego piksela) a CPU (orbita referencyjna w dowolnej precyzji, na której opiera się głęboki zoom). Oba się liczą, ale do różnych rzeczy.
help-hardware-gpu = GPU (najważniejszy)
help-hardware-gpu-text = Każdy piksel jest iterowany i kolorowany na GPU, więc szybkość GPU wyznacza liczbę klatek na sekundę i czas eksportu. Dedykowany GPU z wysoką przepustowością 32-bitową (pojedynczej precyzji) daje najpłynniejszy głęboki zoom. Zintegrowane GPU działają dobrze — tylko wolniej i szybciej osiągają budżet pracy na klatkę (wtedy renderer obcina rozdzielczość, a nie szczegóły). Wymaga nowoczesnego API graficznego: Vulkan, Direct3D 12, Metal lub OpenGL przez wgpu.
# The left column of a requirements table.
help-hardware-minimum = Minimum
help-hardware-gpu-minimum = Dowolny GPU z Vulkan / DX12 / Metal / GL 3.3.
help-hardware-recommended = Zalecane
help-hardware-gpu-recommended = Dedykowany GPU (z ostatnich ~5 lat); >= 2 GB VRAM.
help-hardware-big-exports = Do dużych eksportów
help-hardware-gpu-big-exports = >= 4 GB VRAM (renderowanie kafelkowe, więc przy mniejszej pamięci działa łagodnie wolniej).
# A heading: the processor.
help-hardware-cpu = CPU
help-hardware-cpu-text = Orbita referencyjna wysokiej precyzji (i tablice BLA) są obliczane na CPU i w dużej mierze jednowątkowo, więc najważniejsza jest szybkość pojedynczego rdzenia — od niej zależy, jak szybko głęboki widok nabiera szczegółów po ruchu. Dodatkowe rdzenie pomagają głównie przy kodowaniu eksportu w tle. Wymagany jest 64-bitowy CPU.
help-hardware-cpu-recommended = Nowoczesny 64-bitowy CPU z dobrą wydajnością pojedynczego rdzenia.
help-hardware-memory = Pamięć
help-hardware-memory-text = Głębokie referencje rosną z głębokością (wiele iteracji w wysokiej precyzji); ogromne eksporty trzymają kafelki w pamięci RAM. 8 GB wystarcza do codziennej eksploracji; 16 GB i więcej to dobry wybór do ekstremalnych głębokości i bardzo dużych eksportów.
help-hardware-checking = Sprawdzanie sprzętu
help-hardware-diagnostics = Pomoc → „Diagnostyka…” uruchamia na tym komputerze te same testy, których używają twórcy programu. „Uruchom autotest” sprawdza matematykę i renderowanie względem znanych poprawnych wyników w około piętnaście sekund; „Uruchom test interfejsu” przechodzi przez interfejs i widok na żywo na kilku głębokościach zoomu, robiąc zrzuty ekranu; „Uruchom test arytmetyki GPU” sprawdza arytmetykę double-float shadera na każdym backendzie graficznym dostępnym na tym komputerze, w kilka sekund i bez okna. Każdy test działa jako osobny proces, więc test, który zaburzy działanie sterownika graficznego, kończy tylko siebie, a nie bieżącą sesję.
help-hardware-gpu-check = Test arytmetyki GPU nie ma wyniku pozytywnego ani negatywnego — zapisuje, jak liczy dany GPU i sterownik, a na części sprzętu (każda dotąd sprawdzona karta NVIDIA) zgłasza, że kompilator shaderów odrzuca część arytmetyki o rozszerzonej precyzji. To rzeczywiste odkrycie, które staramy się zmapować u różnych producentów, a nie usterka komputera, więc bez względu na wynik dołączenie go do zgłoszenia naprawdę pomaga.
help-hardware-report = Jeśli coś wygląda źle, uruchom autotest i użyj „Dołącz do zgłoszenia problemu…”: zgłoszenie zawiera wtedy maszynowo zweryfikowany wynik z własnego sprzętu, co jest znacznie bardziej przydatne niż sam opis. Jedno zastrzeżenie warte poznania — obrazy referencyjne powstały na jednej konkretnej karcie graficznej, a różne GPU całkiem prawidłowo obliczają nieco inne piksele, więc na innym sprzęcie porównania obrazów są sprawdzane z szerszą tolerancją, a niewielkie różnice są tam oczekiwane i nie są usterkami.
help-hardware-notes = Uwagi
help-hardware-note-watchdog = Wydajny GPU nie dopuszcza do uruchomienia budżetu pracy watchdoga przy ekstremalnych ustawieniach.
help-hardware-note-binary = Program to jeden samodzielny plik wykonywalny — nie wymaga instalowania środowiska uruchomieniowego.
help-hardware-note-benchmark = Uruchom wbudowany test wydajności (Narzędzia ▸ Test wydajności), aby ocenić swój komputer.

## Help ▸ Acknowledgments & citations
## Names of people, programs, papers, books and websites stay as written, and so do the
## titles in quotation marks (the works' own titles); the prose around them translates.

help-ack-title = Podziękowania i cytowania
# The dedication, in italics at the top of the section.
help-ack-dedication = Dedykowane Stone Soup Group — wolontariuszom, którzy stworzyli Fractint (od 1988 r.).
help-ack-dedication-text = Ich swobodnie udostępniany kod i duch otwartej, wspólnej eksploracji fraktali to powód, dla którego tak wielu z nas pokochało tę sztukę. Fractadyne powstał w tej tradycji.
help-ack-intro = Fractadyne opiera się na dziesięcioleciach pracy społeczności fraktalowej. Techniki i wcześniejsze prace, na których bazuje — sprawdzone w poniższych źródłach:
help-ack-deep-zoom = Algorytmy głębokiego zoomu
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-martin-title = Perturbacja i aproksymacja szeregiem — { $who }
help-ack-martin = Metoda orbity referencyjnej + różnicy w niskiej precyzji (δz → 2Z·δz + δz² + δc) oraz szereg rzędu n pomijający wczesne iteracje, wprowadzone w SuperFractalThing i jego notatce „SuperFractalThing Maths” (sft_maths.pdf, 2013).
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bla-title = Dwuwymiarowa aproksymacja liniowa (BLA) i przebazowanie — { $who }
help-ack-bla = Pomijanie serii iteracji za pomocą scalonych odwzorowań liniowych oraz przebazowanie na pojedynczą referencję w celu uniknięcia glitchy; opracowane na fractalforums.org (2021).
# A link's label. $who: the linked page's author.
help-ack-bla-link = BLA wyjaśnione — { $who }
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-glitch-title = Kryterium wykrywania glitchy — { $who }
help-ack-glitch = Test |Z + z| << |Z| określający, kiedy piksel potrzebuje innej referencji — podstawa perturbacji bez glitchy (fractalforums).
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-diffabs-title = Perturbacja nieanalityczna (Burning Ship) — { $who }
help-ack-diffabs = Analiza przypadków wartości bezwzględnej „diffabs”, dzięki której perturbacja działa dla rodzin opartych na abs (Burning Ship / Celtic / Buffalo), na fractalforums.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mathr-title = Implementacje referencyjne i sprawdzanie krzyżowe — { $who }
help-ack-mathr = Fraktaler-3 i Kalles Fraktaler 2+ (pierwotnie: Karl Runmo) — ich opisy były drogowskazem dla tego silnika, a z ich wynikami EXR Fractadyne porównuje swój renderer.
# A heading: credits for the coloring methods.
help-ack-coloring = Kolorowanie
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-harkonen-title = Płynna liczba iteracji i średnia pasków (Stripe Average) — { $who }
help-ack-harkonen = Ciągłe (znormalizowane) kolorowanie według czasu ucieczki i metoda średniej pasków, z pracy „On Smooth Fractal Coloring Techniques” (praca magisterska, 2007), która analizuje też kolorowania nierównością trójkąta / średnią rozgałęzień.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mitchell-title = Średnia nierówności trójkąta — { $who }
help-ack-mitchell = Uśrednianie tego, gdzie każde z² + c wypada w granicach nierówności trójkąta, dla gładkiej zewnętrznej tekstury przypominającej płomienie.
# A link's label: the person's tutorial pages. $who: the person's name.
help-ack-mitchell-link = { $who } — samouczki
help-ack-distance = Kolorowanie/oświetlenie z estymacji odległości używa klasycznej metody odległości od brzegu dz/dc (zob. Peitgen i Saupe, „The Science of Fractal Images”, 1988).
help-ack-lsystems-life = L-systemy i Gra w życie
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-lindenmayer-title = L-systemy — { $who }
help-ack-lindenmayer = Równoległe przepisywanie ciągów znaków, wprowadzone w 1968 r. do modelowania wzrostu roślin i glonów („Mathematical models for cellular interactions in development”, Journal of Theoretical Biology).
help-ack-abop = Książka (1990), której interpretację żółwia i notację systemów stochastycznych, kontekstowych i parametrycznych stosuje czytnik L-systemów; wbudowane systemy oznaczone ABOP zostały wprowadzone na podstawie jej rysunków.
# A link's label: the book can be read free on that website. $site: the website's name.
help-ack-abop-link = Książka, bezpłatnie na { $site }
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bourke-title = Kolekcja L-systemów — { $who }
help-ack-bourke = Wiele wbudowanych systemów pochodzi z jego stron o L-systemach, w tym reguły Chrisa Wallace’a dla krzywej Sierpińskiego i krzak Dietmara Saupego. Pliki .l programu Fractint, które otwierają się bezpośrednio, są w formacie Stone Soup Group.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-conway-title = Gra w życie — { $who }
help-ack-conway = Automat komórkowy B3/S23 (1970), który trafił do szerokiego grona odbiorców dzięki rubryce Martina Gardnera „Mathematical Games” w Scientific American (październik 1970).
# A credit's heading: the website, then who runs it. $site: the website's name.
help-ack-lifewiki-title = { $site } — społeczność ConwayLife.com
help-ack-lifewiki = Źródło informacji o notacjach reguł odczytywanych przez aplikację (B/S, von Neumann, Generations, litery Alana Hensela dla reguł nietotalistycznych oraz MAP), o formacie wzorców RLE, który aplikacja dzieli z Golly, oraz o wbudowanych regułach i wzorcach, których odkrywców odnotowuje.
help-ack-foundations = Podstawy i narzędzia
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mandelbrot-title = Zbiór Mandelbrota — { $who }
help-ack-mandelbrot = Obiekt, wokół którego wszystko się kręci.
help-ack-built-with = Zbudowano przy użyciu Rust, egui/eframe (emilk) i wgpu (gfx-rs); dowolna precyzja dzięki astro-float, liczby double-word dzięki twofloat; PNG/OpenEXR, rfd, serde. Podziękowania dla ich autorów i opiekunów.
help-ack-errors = Wszelkie błędy w przypisaniu autorstwa obciążają autora; poprawki są mile widziane w systemie zgłoszeń projektu.

## Help ▸ Third-party licenses

help-licenses-title = Licencje zewnętrzne
# $licenses: Fractadyne's license expression (MIT OR Apache-2.0), as written. $url: the web address of a crate's source. THIRD-PARTY-NOTICES.md is a file name, `option-ext` a crate's name, MPL-2.0 a license's.
help-licenses-intro = Fractadyne jest rozpowszechniany jako statycznie konsolidowany plik binarny, który zawiera poniższe otwartoźródłowe biblioteki (crate) i dołączone czcionki. Poniżej znajdują się pełne teksty licencji (także w THIRD-PARTY-NOTICES.md, dołączonym do pobieranego pakietu). Własny kod Fractadyne jest objęty podwójną licencją { $licenses }. `option-ext` jest na licencji MPL-2.0 — jego niezmodyfikowane źródło znajduje się pod adresem { $url }.
# A button, after a copy icon: copies every notice to the clipboard.
help-licenses-copy-all = Kopiuj wszystkie informacje

## Help ▸ About

help-about-tagline = Natywny eksplorator fraktali napisany w języku Rust z użyciem wgpu.
# The arithmetic deep zoom has used. $ran: the libraries that have run so far, as names (or a note that none has yet); $built: those this build contains, with versions.
help-about-arithmetic = Arytmetyka głębokiego zoomu: { $ran } (ta kompilacja zawiera: { $built }).
help-about-files = Pliki i dane
# Followed by the folder's path.
help-about-files-text = Sesja (bieżący widok, kolorowanie i preferencje), zakładki i ich miniatury, zapisane gradienty i zapisane wzory są przechowywane w tym folderze użytkownika i wczytywane przy następnym uruchomieniu:
# Tooltip of the copy button beside the folder's path.
help-about-copy-path = Kopiuj ścieżkę
# $command: a command typed in a terminal (it stays English).
help-about-session = Plik sesji ma numer wersji; jeśli zapisała go wersja Fractadyne nowsza niż ta, która potrafi go w pełni odczytać, aplikacja wczytuje go w miarę możliwości i ostrzega. Aby zacząć zupełnie od nowa, użyj Plik ▸ Resetuj stan aplikacji… (najpierw pyta) lub uruchom `{ $command }` w terminalu.
help-about-license = Licencja
# $licenses: the license expression (MIT OR Apache-2.0), which stays as written.
help-about-license-text = { $licenses } — do użytku na warunkach dowolnej z tych licencji, według wyboru.
# A link to the source code, followed by ↗.
help-about-source = Kod źródłowy w GitHub

## The live render panel ("blinkenlights": View ▸ Live render panel, key B)
## A front panel drawn over the view like a piece of 1970s test equipment. Its legends are engraved
## in capitals: write them in capitals here (the panel shows them exactly as written, it does not
## uppercase them). Numbers, units and readouts on its tubes and dials are not translated.

# Tooltip of the × button in the panel's top-right corner. The code adds the key after it: (B).
live-close-hover = Zamknij panel renderowania na żywo
# The legend engraved along the top plate, left of the maker's badge.
live-plate = RENDER NA ŻYWO · PANEL CZOŁOWY

## Live render panel: instrument titles
## Engraved above each instrument, and repeated as the title of its help card.

# Above the depth readout, whose tubes show the magnification as a power of two ("2^" stays).
live-title-depth = GŁĘBOKOŚĆ 2^
live-title-iterations = ITERACJE
# Above the frame-time readout (milliseconds).
live-title-frame-ms = KLATKA MS
# Above the date-and-time readout.
live-title-clock = DATA · CZAS
# The walk dial: a radar screen showing a slow refresh rendered pass after pass, band after band.
live-title-walk = PRZEJŚCIE · PASMA
# A meter of how far the picture on screen lags the view.
live-title-age = WIEK OBRAZU
# What part of the picture this frame's GPU pass covered.
live-title-tiles = KAFELKI
# A scope of the GPU's work per frame, in milliseconds.
live-title-scope = PRZEBIEGI GPU · MS
# The depth dial: depth, the reference orbits being built, and the arithmetic in use.
live-title-dial = GŁĘBOKOŚĆ · REFERENCJE · ARYTMETYKA
# Iteration steps per second.
live-title-steps = KROKI / S
# The two GPUs (the window's and a second, worker GPU).
live-title-engines = DWA SILNIKI
live-title-memory = PAMIĘĆ
live-title-cores = RDZENIE · WĄTKI
live-title-pixels = STRUMIEŃ PIKSELI
# Help card titles only: the three GPU timer lamps (T0, T1, T2 stay as they are) and the row of lamps.
live-title-timers = ZEGARY GPU T0–T2
live-title-bank = RZĄD LAMPEK

## Live render panel: the light bank's lamps
## Engraved under each lamp, and the title of its help card. A cell is narrow (about eight capital
## letters): a longer label breaks at a space onto two lines, and is drawn smaller if it still does
## not fit.

live-lamp-plan = PLAN
live-lamp-dispatch = ZLECENIE
live-lamp-live = NA ŻYWO
live-lamp-held = TRZYMANY
# The last picture reprojected (stretched) to fit the moving view.
live-lamp-reproj = REPROJ
live-lamp-reading = ODCZYT
# A pass of a pinned walk.
live-lamp-pin-pass = PRZEBIEG PRZYP.
live-lamp-split = PODZIAŁ
live-lamp-adopt = PRZYJĘCIE
live-lamp-abandon = ODRZUCENIE
# "REF": a reference orbit. Built for the view as it is (LIVE), ahead of it (AHEAD), or for a tour's next hold (HOLD).
live-lamp-ref-live = REF NA ŻYWO
live-lamp-ref-ahead = REF WPRZÓD
live-lamp-ref-hold = REF POSTÓJ
# The series approximation.
live-lamp-series = SZEREG
# The second (worker) GPU.
live-lamp-worker = POMOCNIK
live-lamp-tile = KAFELEK
# Supersampling (anti-aliasing); usually kept as SSAA.
live-lamp-ssaa = SSAA
live-lamp-palette = PALETA
# The arithmetic in use (direct, df32, floatexp).
live-lamp-mode = TRYB

## Live render panel: the kinds of GPU pass
## In the GPU scope's legend (MOVING, PIN, WORKER, SETTLE, SSAA). SETTLE and SSAA also name a still
## view's walk on the walk dial.

# A moving frame's pass.
live-kind-moving = RUCH
# A pass of a pinned walk.
live-kind-pin = PRZYP.
# The worker GPU's passes.
live-kind-worker = POMOCNIK
# A still view's settle tile or walk pass.
live-kind-settle = USTALANIE
# A supersampling sample.
live-kind-ssaa = SSAA

## Live render panel: legends and readouts on the instruments

# On the walk dial during a split refresh. $done: passes done; $passes: passes in all.
live-walk-split = PODZIAŁ { $done }/{ $passes }
# On the picture-age meter's face, under its scale (0 to 2 octaves).
live-age-scale = OKTAWY OPÓŹNIENIA
# The depth dial's band for direct (plain floating-point) arithmetic. The other bands, DF32 and FLOATEXP, are names and stay.
live-dial-direct = BEZPOŚR.
# Under the two GPU gauges: GPU 0 draws the window; GPU 1 is the worker, or there is none.
live-engines-gpu-window = GPU 0 · OKNO
live-engines-gpu-worker = GPU 1 · POMOCNIK
live-engines-no-worker = BRAK POMOCNIKA
# The share of displayed frames each GPU made. $window and $worker: percentages, 0–100.
live-engines-on-screen = NA EKRANIE { $window } : { $worker }
# The same readout before any frame was shown.
live-engines-on-screen-none = NA EKRANIE —
# Under the CPU core bars. $count: the machine's cores.
live-cores-count =
    { $count ->
        [one] { $count } RDZEŃ
        [few] { $count } RDZENIE
        [many] { $count } RDZENI
       *[other] { $count } RDZENIA
    }
# Under the threads scope. $count: the app's busy threads.
live-cores-threads = WĄTKI { $count }

## Live render panel: the memory tanks' legend
## What the GPU memory holds, by kind. Very narrow: about ten letters.

# G-buffers (the picture's working textures).
live-memory-gbuf = G-BUF
# The held picture and the supersampling accumulator.
live-memory-hold = TRZYM+SSAA
# The reference orbit and its BLA (bilinear approximation) table.
live-memory-orbit = ORBITA+BLA
# The worker GPU's copy of the reference.
live-memory-worker = REF POMOC.

## Live render panel: the roles on the threads scope
## Tiny labels on a small scope: at most four letters each.

# The reference builders.
live-cores-role-ref = REF
# The scoring pool.
live-cores-role-pool = PULA
# The thread driving the worker GPU.
live-cores-role-worker = POM
# Probes: short side renders.
live-cores-role-probe = SOND
# Image and tour exports.
live-cores-role-export = EKS

## Live render panel: the tiles instrument's captions
## The line under the picture's plane, then the tile size (left) and the tiles a frame (right).

# A walk's pass. $lo and $hi: the iteration band it rendered (numbers).
live-tiles-pin-band = PRZEBIEG PRZYP. · PASMO { $lo }–{ $hi }
live-tiles-settle-band = PRZEBIEG USTALANIA · PASMO { $lo }–{ $hi }
live-tiles-moving-whole = KLATKA W RUCHU · CAŁY OBRAZ
live-tiles-settled-whole = KLATKA USTALONA · CAŁY OBRAZ
# $pass: this pass; $passes: the passes the refresh is split into.
live-tiles-split-pass = PODZIELONE ODŚWIEŻENIE · PRZEBIEG { $pass } Z { $passes }
# $grid: the grid's columns × rows, e.g. 12×8.
live-tiles-grid-done = SIATKA USTALANIA { $grid } · GOTOWE
# $grid: the grid's columns × rows; $done and $total: tiles done, and in all.
live-tiles-grid-progress = SIATKA USTALANIA { $grid } · GOTOWE { $done } Z { $total }
# The same, while a tile too costly for one frame goes out in strips.
live-tiles-grid-strips = SIATKA USTALANIA { $grid } · GOTOWE { $done } Z { $total } · PASKAMI
# $samples: supersampling samples folded so far; $lo and $hi: the iteration band the next one is walking.
live-tiles-ssaa-band = SSAA · GOTOWE { $samples } · PASMO { $lo }–{ $hi }
# $sample: the supersampling sample.
live-tiles-ssaa-whole = PRÓBKA SSAA { $sample } · CAŁA KLATKA
# No GPU pass this frame, and the panel has not seen a frame yet.
live-tiles-idle-starting = START
# No GPU pass this frame: the last complete picture is held on screen.
live-tiles-idle-held = TRZYMANY
# No GPU pass this frame: the last picture is stretched to fit the view.
live-tiles-idle-reprojected = REPROJEKCJA
# No GPU pass this frame: the picture on screen is current.
live-tiles-idle-live = NA ŻYWO
# No GPU pass this frame, and nothing to show.
live-tiles-idle-none = BEZCZYNNY
# The tile size: $res, a whole picture's resolution (2400×1300).
live-tiles-tile-res = KAFELEK { $res }
# The tile size: $px, a tile's side in pixels.
live-tiles-tile-px = KAFELEK { $px } PX
live-tiles-no-pass = BRAK PRZEBIEGU
# Tiles a frame. $count: how many (0 when none).
live-tiles-rate-count = { $count }/KLATKĘ
live-tiles-rate-one = 1 KAFELEK/KLATKĘ
# $share: the part of one tile sent each frame, e.g. 0.33.
live-tiles-rate-part = { $share } KAFELKA/KLATKĘ
live-tiles-rate-sample = 1 KAFELEK/PRÓBKĘ
# In the plane's corner during a walk: the share of pixels still iterating. $percent: a percentage, e.g. 54 or 1.5.
live-tiles-running = W TOKU { $percent }%

## Live render panel: help cards
## Shown when the pointer rests on an instrument or a lamp. Words in capitals (SETTLE, SSAA, ABANDON,
## HELD, REPROJ, ADOPT, REF AHEAD, ON SCREEN) are the panel's own legends: use the same words as
## their messages above.

# The heading of a card's list of what healthy looks like.
live-card-healthy = W NORMIE
# The heading of a card's list of what signals trouble.
live-card-trouble = PROBLEMY

live-card-walk-what = Głębokie odświeżenie zbyt kosztowne na jedną klatkę jest przypinane i przechodzone: renderowane przebieg po przebiegu, a każdy przebieg pozwala pikselom iterować o oktawę dalej (256, 512, 1024 … aż do żądanej liczby). Obwód to ta liczba w skali logarytmicznej, licząc od góry; wiązka to postęp przejścia; każdy łuk to jeden przebieg, pokolorowany według jego czasu GPU względem celu 20 ms. Biały punkt czeka na odczyt ostatniego przebiegu; błysk to nowy obraz trafiający na ekran.
live-card-walk-note = Nieruchomy widok zbyt kosztowny na jeden przebieg jest przechodzony tak samo, z oznaczeniem USTALANIE, a każda próbka supersamplingu przechodzi ponownie, z oznaczeniem SSAA: każdy z ich przebiegów może trwać do 400 ms i względem tego kolorowane są łuki. Podzielone odświeżenie rysuje zamiast tego jeden pierścień na przebieg: każde pasmo, część kafelków.
live-card-walk-good-sweep = Obieg zataczający koło w kilku klatkach i kończący się błyskiem.
live-card-walk-good-arcs = Łuki głównie zielone i bursztynowe.
live-card-walk-good-still = Przy głębokim nieruchomym widoku: wolniejszy obieg na próbkę, z łukami od zielonych do bursztynowych.
live-card-walk-bad-red = Czerwone łuki: przebiegi ponad budżet, odczuwalne jako przycięcia.
live-card-walk-bad-restart = Obieg, który zatrzymuje się w połowie i zaczyna od nowa: przejście zostało porzucone, a obraz na ekranie pozostaje stary.
live-card-walk-bad-hollow = Puste łuki: przebiegi, których pomiar czasu nigdy nie wrócił, więc następny przebieg jest dobierany na ślepo.

live-card-age-what = Jak bardzo obraz na ekranie opóźnia się względem widoku, w oktawach zoomu (jedna oktawa to 2×). Gdy renderuje się odświeżenie, stary obraz jest pokazywany rozciągnięty i się starzeje; gdy odświeżenie trafi na ekran, wiek spada. Pasek poniżej to ostatnie kilka sekund.
live-card-age-good-sawtooth = Piła między około 0,3 a 0,8, ze wskazówką wahającą się wokół znaku 0,5.
live-card-age-good-still = Blisko 0 po zatrzymaniu.
live-card-age-bad-red = Wskazówka stale powyżej 1,0, w czerwonym polu: obraz jest nieaktualny o więcej niż 2× i podczas zoomu wygląda na miękki lub blokowy.
live-card-age-bad-ramp = Rampa, która nigdy nie spada: odświeżenia się nie kończą (obserwuj ODRZUCENIE).

live-card-tiles-what = Jak przebieg GPU tej klatki pokrywa obraz. Klatki w ruchu i przebiegi przejścia pokrywają go w całości, a podpis podaje pasmo iteracji przebiegu przejścia. Przebieg przejścia iteruje tylko piksele, które jeszcze nie uciekły: płaszczyzna wypełnia się do tego udziału (skala logarytmiczna, 0,01% na dole) i pokazuje go w rogu, więc opróżnia się, w miarę jak przejście sięga głębiej. Odświeżenie nieco zbyt kosztowne na jedną klatkę jest dzielone na maksymalnie 8 przebiegów, z których każdy to przeplatany zestaw kafelków 16 px. Ustalony widok zbyt kosztowny na jeden przebieg staje się siatką kafelków, po jednym na klatkę, od środka na zewnątrz, a kafelek wciąż zbyt kosztowny jest wysyłany paskami. Poniżej: rozmiar kafelka i kafelki na klatkę.
live-card-tiles-good-whole = Całe przebiegi podczas zoomu; podzielone odświeżenia z kilku przebiegów.
live-card-tiles-good-grid = Po zatrzymaniu: siatka kilkudziesięciu kafelków wypełniona w sekundę lub dwie, potem SSAA.
live-card-tiles-bad-small = Siatka setek małych kafelków: budżet każdego przebiegu jest mały, a wyostrzanie potrwa długo.
live-card-tiles-bad-strips = Wiele kafelków wysyłanych paskami: kafelki działały wolniej, niż zakładał rozmiar siatki.
live-card-tiles-bad-splits = Podziały utknięte na 8 przebiegach: odświeżenie ledwo się mieści, a trochę głębiej wraca do wolniejszego przejścia.
live-card-tiles-bad-dark = Długo ciemno podczas ruchu: brak jakichkolwiek przebiegów.

live-card-scope-what = Jedna kolumna na klatkę, najnowsza po prawej: wykonana praca GPU, ułożona według rodzaju (klatka w ruchu — zielona, przebieg przypięty — od zielonego do czerwonego według kosztu, kafelek ustalania lub przebieg przejścia nieruchomego widoku — kremowy, próbka supersamplingu — lawendowa, klatki i próbki GPU pomocniczego — niebieskie), w milisekundach. Linie przerywane przy 16,7 ms (60 fps), 33 i 50. Pusty znacznik to przebieg, którego pomiar czasu nigdy nie wrócił.
live-card-scope-good-under = Kolumny poniżej linii 16,7 podczas ruchu.
live-card-scope-good-pin = Przebiegi przypięte od zielonych do bursztynowych.
live-card-scope-good-samples = Próbki supersamplingu nieruchomego widoku mogą trwać dłużej: nic się nie porusza.
live-card-scope-bad-over = Kolumny powyżej 33 lub 50 ms: widoczne przycięcia.
live-card-scope-bad-seconds = Przebiegi trwające sekundy: watchdog GPU może zresetować urządzenie.
live-card-scope-bad-hollow = Wiele pustych znaczników: praca jest dobierana bez pomiarów.

live-card-timers-what = Trzy sloty znaczników czasu GPU. Slot zapala się, gdy przebieg jest mierzony, i gaśnie, gdy wraca jego odczyt, dwie lub trzy klatki później. T0 mierzy przebieg każdej klatki; T1 i T2 mierzą dodatkowe przebiegi przypięte.
live-card-timers-good = T0 miga zawsze, gdy jest praca; T1 i T2 migają podczas przejść.
live-card-timers-bad-stuck = Lampka stale zapalona: jej odczyt nie wraca, a wyceny się dezaktualizują.
live-card-timers-bad-dark = Wszystkie ciemne, gdy GPU jest zajęty: nic nie jest mierzone.

live-card-dial-what = Czerwona wskazówka to głębokość widoku: jego powiększenie jako potęga dwójki. Pasma to arytmetyka, której GPU tam używa: bezpośrednia, df32 (dwie liczby float na liczbę) i floatexp (float z szerokim wykładnikiem) powyżej 2^93. Kropki to budowane orbity referencyjne: zielone dla widoku, bursztynowe przed nim, tam, dokąd zmierza zoom, niebieskie dla następnego postoju przelotu. Pierścień to instalacja; czerwony krzyżyk to budowa porzucona, bo widok ją wyprzedził.
live-card-dial-good-ahead = Bursztynowe kropki pojawiające się przed wskazówką i otoczone pierścieniem, zanim do nich dotrze.
live-card-dial-good-crosses = Mało czerwonych krzyżyków.
live-card-dial-bad-passing = Wskazówka mija ostatnią zainstalowaną referencję: klatki na nią czekają, co odczuwa się jako zamrożenie.
live-card-dial-bad-crosses = Powtarzające się czerwone krzyżyki: zoom wyprzedza budowanie referencji, a czas CPU jest marnowany.
live-card-dial-bad-crossing = Przejście przez 2^93 to znane wolne miejsce: pierwsze klatki floatexp kosztują więcej.

live-card-bank-what = Każda lampka zapala się, gdy zachodzi jej zdarzenie, i gaśnie w ciągu około 150 ms, więc zajęty renderer migocze, a bezczynny ciemnieje. Od lewej do prawej: klatka, odświeżenie, referencje, potem pomocnik, ustalanie, paleta i arytmetyka.
live-card-bank-note = Najedź na lampkę, aby zobaczyć, co oznacza.

live-card-depth-what = Powiększenie jako potęga dwójki: 2^100 to około 1,3 × 10^30.
live-card-depth-note = Powyżej 2^93 każdy krok kosztuje więcej (floatexp), a budowa referencji trwa dłużej.

live-card-iter-what = Żądana liczba iteracji: jak daleko każdy piksel może iterować, zanim zostanie uznany za należący do zbioru. Rośnie z głębokością.
live-card-iter-good = Stały wzrost podczas zanurzania.
live-card-iter-bad-low = Utknęła nisko w głębi: spirale i minibroty renderują się jako płaskie wnętrze.
live-card-iter-bad-high = Bardzo wysoka: każde odświeżenie kosztuje więcej, a przejścia trwają dłużej.

live-card-ms-what = Czas między wyświetlanymi klatkami.
live-card-ms-good = Odstęp odświeżania wyświetlacza: 16,7 na wyświetlaczu 60 Hz.
live-card-ms-bad-stutter = Powyżej 33: widoczne przycięcia.
live-card-ms-bad-spikes = Skoki powyżej 50: zacięcia, zwykle jeden kosztowny przebieg (zob. PRZEBIEGI GPU).

live-card-clock-what = Zegar w czasie lokalnym, z dokładnością do dziesiątej części sekundy.
live-card-clock-note = Oznacza każdy zrzut ekranu panelu czasem jego wykonania, więc obraz problemu można dopasować do dziennika aplikacji.

live-card-tach-what = Kroki iteracji na sekundę. Biała: kroki żądane (piksele × iteracje). Bursztynowa: kroki faktycznie wykonane przez GPU. Różnica to BLA (aproksymacja dwuliniowa) przeskakujące wiele iteracji naraz; ×N to ten mnożnik. Licznik sumuje całą sesję.
live-card-tach-good = Duża różnica w głębi: BLA wykonuje swoją pracę, od ×10 do ×1000.
live-card-tach-bad-together = Wskazówki razem w głębi: BLA nie może tu nic pominąć (często w pobliżu wnętrza minibrota), więc odświeżenia są wolne.
live-card-tach-bad-red = Biała wskazówka w czerwonym polu: żądanie większe, niż GPU może wykonać w jednej klatce.

live-card-engines-what = Jak bardzo zajęty jest każdy GPU, z kropkami niosącymi jego klatki na ekran: czas GPU jego przebiegów w ostatniej sekundzie, łącznie z klatkami ruchu i próbkami supersamplingu pomocnika. NA EKRANIE to udział wyświetlonych klatek, które wykonał każdy z nich. Przy jednym GPU strona pomocnika pozostaje bezczynna.
live-card-engines-good = Oba zajęte podczas zanurzania, a pomocnik dostarcza część klatek.
live-card-engines-bad-pinned = GPU 0, ten od okna, przyklejony blisko 100%: jest wąskim gardłem, a klatki się przycinają.
live-card-engines-bad-idle = Pomocnik włączony, ale bezczynny: nie dostarcza klatek.

live-card-memory-what = RAM: zestaw roboczy aplikacji, czerwona linia to jego szczyt. VRAM: to, co aplikacja trzyma na GPU, liczone z rozmiarów jej alokacji, według rodzaju (G-bufory, tekstury trzymanego obrazu i SSAA, bufory orbity i BLA, kopia referencji pomocnika), na tle pojemności karty.
live-card-memory-good = Płasko podczas zoomu; skoki w górę i z powrotem w dół, gdy referencje pojawiają się i znikają.
live-card-memory-bad-leak = RAM rośnie i nigdy nie spada: wyciek.
live-card-memory-bad-vram = VRAM blisko pojemności karty: alokacje mogą się nie udać albo urządzenie może zostać utracone.

live-card-cores-what = Użycie CPU na rdzeń, od zielonego przez bursztynowy do czerwonego, i oscyloskop, na którym każdy zajęty wątek krąży wokół swojej roli: budowanie referencji, pula oceniania, pomocnik, renderowanie poboczne.
live-card-cores-good = Jeden lub dwa rdzenie zajęte podczas zoomu; krótki zryw na wszystkich rdzeniach, gdy wybierana jest referencja.
live-card-cores-bad-every = Wszystkie rdzenie zajęte przez sekundy: budowa referencji na ekstremalnej głębokości.
live-card-cores-bad-own = Własny rdzeń aplikacji przyklejony w czerwonym polu: samo okno się przycina.

live-card-pixels-what = Piksele obliczane na sekundę, w Mpx/s. Przygaszone: wciąż iterowane w przejściu. Jasne: gotowe.
live-card-pixels-good = Dziesiątki Mpx/s podczas ruchu.
live-card-pixels-bad = Blisko zera, gdy obraz jest nieaktualny: GPU na coś czeka — na referencję lub odczyt.

## Live render panel: the lamps' help cards

live-card-lamp-plan-what = Zaplanowano klatkę: wybrano rozdzielczość, żądaną liczbę iteracji i budżet oraz to, czy odświeżać.
live-card-lamp-plan-good = Miga, gdy tylko zmienia się widok lub ustawienia.
live-card-lamp-plan-bad = Ciemna podczas zoomu: wątek interfejsu utknął.

live-card-lamp-dispatch-what = W tej klatce wysłano pracę do GPU.
live-card-lamp-dispatch-good = Zapalona, gdy cokolwiek się zmienia; ciemna w bezczynności.
live-card-lamp-dispatch-bad = Zajęta, choć nic się nie zmienia: aplikacja się nie ustala.

live-card-lamp-live-what = Obraz na ekranie to własny render tej klatki.
live-card-lamp-live-note = Rzadko zapalona podczas głębokiego zanurzania, co jest normalne: zob. TRZYMANY i REPROJ.
live-card-lamp-live-good = Zapalona na płytkich głębokościach, gdzie klatka jest tania.

# "Prefer detail" is a setting's name.
live-card-lamp-held-what = Na ekranie jest ostatni pełny obraz, a pod nim składa się następny: odświeżenie głębokiego zoomu lub, przy włączonym „Preferuj szczegóły podczas zoomu”, następny kafelek albo próbka supersamplingu nieruchomego widoku.
live-card-lamp-held-good-dive = Zapalona odcinkami podczas głębokiego zoomu, przerywana przez PRZYJĘCIE.
live-card-lamp-held-good-still = Zapalona stale, gdy nieruchomy widok się wyostrza, a SSAA miga.
live-card-lamp-held-bad = Zapalona przez sekundy podczas zoomu bez lampki PRZYJĘCIE: odświeżenia się nie kończą.

live-card-lamp-reproj-what = Ostatni obraz rozciągnięty, aby pasował do poruszającego się widoku, dopóki nowy nie będzie gotowy.
live-card-lamp-reproj-good = Krótkie serie podczas ruchu.
live-card-lamp-reproj-bad = Zapalona długo: obraz robi się tym bardziej miękki, im mocniej jest rozciągnięty.

live-card-lamp-reading-what = Z GPU wrócił odczyt: czas przebiegu, zakres ucieczki lub zawartość.
live-card-lamp-reading-good = Równomierne miganie podczas renderowania.
live-card-lamp-reading-bad = Długo ciemna podczas renderowania: pomiary czasu nie wracają, a budżety są zgadywane.

live-card-lamp-pin-pass-what = Przebieg przypiętego przejścia: obraz wyrenderowany o jedno pasmo iteracji dalej.
live-card-lamp-pin-pass-good = Serie przebiegów kończące się lampką PRZYJĘCIE.
live-card-lamp-pin-pass-bad = Serie kończące się lampką ODRZUCENIE.

live-card-lamp-split-what = Przebieg podzielonego odświeżenia: część kafelków wyrenderowana z pełną liczbą iteracji.
live-card-lamp-split-good = Krótkie serie kończące się lampką PRZYJĘCIE.
live-card-lamp-split-bad = Serie po 8, maksymalnie dozwolone: odświeżenie ledwo się mieści.

live-card-lamp-adopt-what = Gotowe odświeżenie zastąpiło obraz na ekranie.
live-card-lamp-adopt-good = Kilka razy na sekundę podczas zoomu.
# "Picture age" names the instrument (its title, PICTURE AGE).
live-card-lamp-adopt-bad = Rzadko podczas zoomu: obraz się starzeje (zob. WIEK OBRAZU).

live-card-lamp-abandon-what = Trwające odświeżenie zostało wyrzucone: widok przesunął się za daleko, odświeżenie zbytnio się zestarzało albo zastąpiło je nowsze. Budowa referencji wyprzedzona przez widok zapala tę lampkę słabo.
live-card-lamp-abandon-good = Rzadko albo raz zaraz po skoku.
live-card-lamp-abandon-bad = Często: praca jest marnowana, a obraz pozostaje stary.

live-card-lamp-ref-live-what = Budowana jest orbita referencyjna dla widoku w obecnym stanie: widok nie ma żadnej, opuścił tę, którą miał, albo potrzebuje dłuższej.
live-card-lamp-ref-live-good = Krótkie mignięcia.
live-card-lamp-ref-live-bad-long = Długie odcinki: głębokie widoki na nią czekają.
live-card-lamp-ref-live-bad-lookahead = Często zapalona podczas zanurzania, gdy REF WPRZÓD jest ciemna: wyprzedzanie nie nadąża.

live-card-lamp-ref-ahead-what = Budowana jest referencja wyprzedzająca, głębiej niż widok, tam, dokąd zmierza zoom.
live-card-lamp-ref-ahead-good = Regularne mignięcia podczas zanurzania.
live-card-lamp-ref-ahead-bad = Ciemna podczas głębokiego zanurzania: widok wyprzedzi swoje referencje.

live-card-lamp-ref-hold-what = Wstępne pobieranie dla postoju przelotu: referencja dla następnej zatrzymanej klatki kluczowej, budowana podczas lotu ku niej, aby odtwarzanie nigdy nie czekało.
live-card-lamp-ref-hold-note = Ciemna poza odtwarzaniem przelotu.
live-card-lamp-ref-hold-good = Mignięcie przed każdym postojem w przelocie.
live-card-lamp-ref-hold-bad = Zapalona przez cały postój: wstępne pobieranie się spóźnia, a przelot na nie czeka.

live-card-lamp-series-what = Aproksymacja szeregiem dla nowej referencji, która pozwala każdemu pikselowi pominąć pierwsze iteracje.
live-card-lamp-series-good = Krótko.
live-card-lamp-series-bad = Długo w głębi: opóźnia instalację referencji.

live-card-lamp-worker-what = Drugi GPU dostarczył klatkę.
live-card-lamp-worker-good = Równomierne mignięcia, gdy GPU pomocniczy jest włączony.
live-card-lamp-worker-bad = Ciemna przy włączonym pomocniku: nie dostarcza klatek.

live-card-lamp-tile-what = Kafelek ustalania lub jego pasek trafił do GPU.
live-card-lamp-tile-good = Seria po zatrzymaniu, potem cisza.
live-card-lamp-tile-bad = Nigdy nie ustaje: siatka wciąż startuje od nowa, bo coś ciągle zmienia widok lub ustawienia.

live-card-lamp-ssaa-what = Do ustalonego obrazu dodano próbkę supersamplingu.
live-card-lamp-ssaa-good = Seria mignięć po ustaleniu, potem ciemność.
live-card-lamp-ssaa-bad = Ciągłe ponowne starty: ustalanie jest wciąż zakłócane.

live-card-lamp-palette-what = Normalizacja kolorów przyjęła nowy zakres ucieczki, więc kolory podążają za zoomem.
live-card-lamp-palette-good = Miga podczas zoomu.
live-card-lamp-palette-bad = Ciemna podczas zoomu: zakres palety jest zamrożony, a kolory stają się płaskie i mało kontrastowe.

live-card-lamp-mode-what = Zmieniła się arytmetyka: bezpośrednia, df32 lub floatexp.
live-card-lamp-mode-good = Jedno mignięcie na granicy, np. 2^93.
live-card-lamp-mode-bad = Miganie tam i z powrotem: widok oscyluje wokół granicy.

## The Render client window (File ▸ Render client…): lending this machine's GPU to a render farm

farm-client-title = Klient renderowania
farm-client-intro = Renderuj klatki dla farmy renderującej. Kontroler — Narzędzia ▸ Renderuj na farmie… na innym komputerze — pokazuje swój adres i klucz farmy. Ten komputer nie otwiera żadnego portu: to on łączy się z kontrolerem.
# A field label, and the label of the status row that names the controller.
farm-client-controller = Kontroler
farm-client-find = Znajdź
farm-client-find-hover = Szukaj kontrolerów w tej sieci (kontroler odpowiada, gdy jego farma nasłuchuje)
farm-client-key = Klucz farmy
# A checkbox beside the masked farm key: show it in clear.
farm-client-show-key = Pokaż
# A field label (the name this machine goes by), and the label of the status row about this machine.
farm-client-machine = Ten komputer
farm-client-machine-hover = Nazwa, pod którą kontroler pokazuje ten komputer.
farm-client-share = Dysk współdzielony
# Greyed hint inside the empty field; the path is an example.
farm-client-share-hint = opcjonalnie — np. \\fileserver\share
farm-client-share-hover = Ścieżka tego komputera do dysku współdzielonego, którego używa kontroler (tryb współdzielenia): klatki są tam zapisywane, zamiast być wysyłane przez połączenie. Pozostaw puste, aby wysyłać każdą klatkę.
farm-client-gpu = Karta graficzna
# The label of the row with the idle option.
farm-client-when = Kiedy
# A checkbox followed by a number of minutes ("Only when idle for [5] min").
farm-client-when-idle = Tylko gdy bezczynny przez
farm-client-when-idle-hover = Przyjmuj pracę dopiero wtedy, gdy nikt nie dotykał klawiatury ani myszy tego komputera przez ten czas, albo gdy jego ekran jest zablokowany. Gdy ktoś zacznie z niego korzystać, bieżąca klatka natychmiast się zatrzymuje i wraca do farmy.
# A collapsible section: the largest job this machine accepts.
farm-client-limits = Limity
farm-client-limits-note = Zadanie wymagające więcej niż te wartości jest odrzucane, nigdy przycinane.
farm-client-limit-size = Największa klatka
farm-client-limit-ss = Maks. supersampling
farm-client-limit-iter = Maks. iteracji

## Render client: the graphics-card choice

farm-client-gpu-system = Wybór systemu
farm-client-gpu-every = Wszystkie karty graficzne
# $count: this machine's graphics cards.
farm-client-gpu-every-n = Wszystkie karty graficzne ({ $count })
# $count: this machine's graphics cards.
farm-client-gpu-every-each = Wszystkie karty graficzne ({ $count }) — każda z osobną sesją
# A card the window cannot name. $number: its number (or numbers) as --list-adapters gives it.
farm-client-gpu-adapter = Adapter { $number }
farm-client-gpu-finding = Wyszukiwanie kart graficznych tego komputera…
# "NAME · GPU 1" is how such a machine is named in the controller's list: keep it as it is.
farm-client-gpu-hover = Która karta graficzna renderuje klatki farmy. Przy wszystkich kartach każda dołącza do farmy jako osobny komputer — „NAME · GPU 1”, „NAME · GPU 2” — a Wstrzymaj i Rozłącz działają na wszystkie.

## Render client: looking for controllers on the network (the Find button)

farm-client-finding = Szukanie kontrolerów w tej sieci…
# $port: the UDP port number.
farm-client-found-none = Żaden kontroler nie odpowiedział. Kontroler odpowiada, gdy jego farma nasłuchuje; jego zapora musi przepuszczać UDP { $port }.
farm-client-found-pick = W tej sieci — kliknij jeden, aby go użyć:
# $identity: the controller's identity fingerprint.
farm-client-found-hover = Tożsamość { $identity } — użyj tego kontrolera (jego klucz farmy jest nadal potrzebny)
# After a warning sign. $version and $git: this app's version and build.
farm-client-found-other-build = Inna kompilacja niż ta ({ $version } { $git }): odrzuciłaby ten komputer.

## Render client: what is wrong with what was typed, or with starting the client

farm-client-bad-address = Wpisz adres kontrolera jako HOST:PORT, tak jak pokazuje go jego okno (na przykład 192.168.1.20:46733).
# $why: what is wrong with the key (in English).
farm-client-bad-key = Klucz farmy: { $why }
# $why: what is wrong with the name (in English).
farm-client-bad-name = Nazwa: { $why }
farm-client-no-config = Nie ma folderu konfiguracji, w którym można przechować klucz farmy.
# $why: the system's reason.
farm-client-start-failed = Nie udało się uruchomić klienta renderowania: { $why }

## Render client: the state, in the status heading

farm-client-phase-connecting = Łączenie…
farm-client-phase-checking = Autotest w toku…
farm-client-phase-idle = Połączono — oczekiwanie na pracę
farm-client-phase-rendering = Renderowanie
farm-client-phase-in-use = Oczekiwanie — ktoś używa tego komputera
farm-client-phase-paused = Wstrzymano
farm-client-phase-retrying = Kontroler nieosiągalny — ponawianie
# $seconds: the wait before the next try.
farm-client-phase-retrying-in = Kontroler nieosiągalny — ponowna próba za { $seconds } s
farm-client-phase-disconnected = Rozłączono
farm-client-phase-ended = Zakończono

## Render client: the line under the status heading

# $address: the controller's address (HOST:PORT).
farm-client-detail-connecting = Łączenie z { $address }…
# $controller: the controller's name.
farm-client-detail-checking = Połączono z „{ $controller }” — autotest w toku
# $controller: the controller's name.
farm-client-detail-approval = Połączono z „{ $controller }” — oczekiwanie, aż jego użytkownik zatwierdzi ten komputer
# $controller: the controller's name.
farm-client-detail-idle = Połączono z „{ $controller }” — bezczynny
# $why: the check that failed and why (in English).
farm-client-detail-check-failed = Autotest nie powiódł się — { $why }
# $why: the controller's reason (in English).
farm-client-detail-refused = Odrzucony przez kontroler: { $why }
# $why: the controller's reason (in English).
farm-client-detail-closed = Kontroler zamknął połączenie: { $why }
# $why: the controller's reason (in English).
farm-client-detail-ended = Kontroler zakończył połączenie: { $why }
# $job: the job's name.
farm-client-detail-bundle = Zadanie „{ $job }” — odbieranie pakietu
# $job: the job's name.
farm-client-detail-job-waiting = Zadanie „{ $job }” — oczekiwanie na klatki do renderowania
# $why: the limit the job is over (in English).
farm-client-detail-job-refused = Odrzucono to zadanie: { $why }
farm-client-detail-job-over = Zadanie się zakończyło
# $first and $last: frame numbers.
farm-client-detail-rendering = Renderowanie klatek { $first }–{ $last }
# $time: how long nobody must use the machine ("90 s", "5 min").
farm-client-detail-in-use = Oczekiwanie — ktoś używa tego komputera; praca zostanie wznowiona, gdy nikt nie będzie go używał przez { $time }
farm-client-detail-paused = Wstrzymano — kontroler został powiadomiony
farm-client-detail-pausing = Wstrzymywanie — kończenie bieżącej klatki
farm-client-detail-cancelled = Anulowano bieżącą klatkę; wstrzymano
# A machine with several graphics cards. $rendering: the cards rendering now; $total: all its cards (two or more).
farm-client-rendering-on-cards = Renderowanie na { $rendering } z { $total } kart graficznych

## Render client: the status rows

# The value of the Controller row. $name: the controller's name; $address: HOST:PORT; $identity: its identity fingerprint.
farm-client-row-controller-value = „{ $name }” · { $address } · tożsamość { $identity }
# The value of the This machine row. $name: this machine's name; $identity: its identity fingerprint.
farm-client-row-machine-value = „{ $name }” · tożsamość { $identity }
farm-client-row-gpu = GPU
# One graphics card's row. $slot: the card's number.
farm-client-row-gpu-n = GPU { $slot }
farm-client-row-job = Zadanie
# The frames handed to this machine.
farm-client-row-run = Seria
farm-client-row-sent = Wysłane
# The Run row, in parts joined with " · ": the run, the frame in progress, its time. $first and $last: frame numbers.
farm-client-run-frames = klatki { $first }–{ $last }
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-client-run-frame = klatka { $frame } ({ $index } z { $count })
# $time: the time spent on it so far ("2.1 s").
farm-client-run-frame-time = { $time } na tej klatce
# One graphics card's run. $frame: the frame in progress; $first and $last: the run's frames.
farm-client-session-frame = klatka { $frame } z { $first }–{ $last }
farm-client-session-no-card = karta graficzna jeszcze nieznana
# The Sent row. $count: frames sent; $time: the mean time per frame ("2.1 s").
farm-client-sent =
    { $count ->
        [one] { $count } klatka · { $time } na klatkę
        [few] { $count } klatki · { $time } na klatkę
        [many] { $count } klatek · { $time } na klatkę
       *[other] { $count } klatki · { $time } na klatkę
    }
# The Sent row of a machine with several cards. $count: frames sent; $split: each card's share ("812 + 422"); $time: the mean time per frame.
farm-client-sent-split =
    { $count ->
        [one] { $count } klatka ({ $split }) · { $time } na klatkę
        [few] { $count } klatki ({ $split }) · { $time } na klatkę
        [many] { $count } klatek ({ $split }) · { $time } na klatkę
       *[other] { $count } klatki ({ $split }) · { $time } na klatkę
    }
# After a warning sign. $note: how the GPUs differ, and both GPUs (in English).
farm-client-gpu-note = Ten komputer ma { $note }. Jego klatki będą się nieco różnić od klatek kontrolera, co może być widoczne w statycznym ujęciu.
# A collapsible section: the self-check's results.
farm-client-self-check = Autotest
farm-client-thumb-hover = Ostatnia klatka wysłana przez ten komputer
farm-client-resume = Wznów
farm-client-resume-hover = Ponownie przyjmuj pracę
farm-client-pause = Wstrzymaj
farm-client-pause-hover = Dokończ bieżącą klatkę, a potem nie przyjmuj więcej pracy. Kontroler przekazuje klatki tego komputera pozostałym.
farm-client-cancel-frame = Anuluj klatkę
farm-client-cancel-frame-hover = Zatrzymaj bieżącą klatkę od razu i wstrzymaj; kontroler wyrenderuje ją gdzie indziej.
farm-client-live-paused = Widok na żywo jest wstrzymany, gdy ten komputer renderuje klatki farmy.
farm-client-close = Zamknij
farm-client-close-active-hover = Zamknij okno; ten komputer pozostaje w farmie (otwórz je ponownie z menu Plik)
farm-client-close-hover = Zamknij okno
farm-client-disconnect = Rozłącz
farm-client-disconnect-hover = Opuść farmę: bieżąca klatka zostaje zwrócona
farm-client-connect = Połącz
farm-client-connect-hover = Połącz się z kontrolerem i renderuj dla niego klatki

## The Render on farm window (Tools ▸ Render on farm…): the controller of a render farm

farm-ctl-title = Renderuj na farmie
farm-ctl-out = Folder wyjściowy
farm-ctl-browse = Przeglądaj…
farm-ctl-out-note = Klatki i stan zadania w farm/: uruchom to samo zadanie ponownie, aby je wznowić.
farm-ctl-size = Rozmiar
farm-ctl-size-custom = Własny…
# The size list's choice when the size is typed in. $width and $height: pixels.
farm-ctl-size-custom-value = Własny — { $width }×{ $height }
farm-ctl-fps = Klatki / sekundę
farm-ctl-ss = Supersampling
farm-ctl-port = Nasłuchuj na porcie
farm-ctl-port-hover = Klienci łączą się z tym portem. Zezwól na niego w zaporze dla sieci lokalnej.
farm-ctl-share = Dysk współdzielony
# Greyed hint inside the empty field.
farm-ctl-share-hint = opcjonalnie — ścieżka tego komputera do niego
farm-ctl-share-hover = Tryb współdzielenia: klienci, którzy mają ten sam dysk, zapisują na nim swoje klatki zamiast wysyłać je przez połączenie, a ten komputer sprawdza tam każdą z nich. Klienci bez niego wysyłają swoje.
# A label followed by a number of clients and two checkboxes.
farm-ctl-start-with = Rozpocznij z
# The unit after the number in the box ("Start with [3] clients"). $count: that number.
farm-ctl-start-clients =
    { $count ->
        [one] klientem
        [few] klientami
        [many] klientami
       *[other] klienta
    }
farm-ctl-start-clients-hover = Zadanie rusza, gdy połączy się tyle komputerów i przejdą one autotest; inne mogą dołączyć później.
farm-ctl-local = Renderuj także na tym komputerze
farm-ctl-local-hover = Ten komputer liczy się jako jeden klient.
farm-ctl-sharing = Udostępniaj referencje
farm-ctl-sharing-hover = Budowa orbity referencyjnej głębokiej klatki może trwać od minut do godzin. Udostępniana jest budowana raz i przekazywana pozostałym komputerom. Udostępniona referencja jest inna, ale równie poprawna: na ekstremalnej głębokości garść pikseli może różnić się od klatki, która zbudowała własną.
farm-ctl-key = Klucz farmy
# A checkbox beside the shortened farm key: show all of it.
farm-ctl-show-key = Pokaż
farm-ctl-copy-key = Kopiuj klucz
farm-ctl-copy-key-hover = Wklej go w oknie Klient renderowania każdego klienta
# In place of the farm key, when there is none.
farm-ctl-no-key = jeszcze brak
farm-ctl-make-key = Utwórz klucz
farm-ctl-make-key-hover = Klienci potrzebują go, aby dołączyć; jest też tworzony po naciśnięciu Renderuj
# What to do on the other machines. $address: this controller's HOST:PORT.
farm-ctl-steps = Na każdym kliencie: Plik ▸ Klient renderowania…, kontroler { $address } i klucz farmy.
# The same when this machine's address is not known. $port: the port number.
farm-ctl-steps-port = Na każdym kliencie: Plik ▸ Klient renderowania…, adres tego komputera i port { $port } oraz klucz farmy.
# The size of the job. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
farm-ctl-summary =
    { $count ->
        [one] { $count } klatka · { $seconds }s przelotu · { $width }×{ $height } ss{ $ss }
        [few] { $count } klatki · { $seconds }s przelotu · { $width }×{ $height } ss{ $ss }
        [many] { $count } klatek · { $seconds }s przelotu · { $width }×{ $height } ss{ $ss }
       *[other] { $count } klatki · { $seconds }s przelotu · { $width }×{ $height } ss{ $ss }
    }
farm-ctl-strip-hover = Każda klatka zadania, od lewej do prawej: gotowe, renderowane, do zrobienia, porzucone.
farm-ctl-no-tour = Nie wczytano przelotu.
# $folder: the folder; $why: the system's reason.
farm-ctl-out-error = Folder wyjściowy { $folder }: { $why }
# $why: the system's reason.
farm-ctl-start-failed = Nie udało się uruchomić farmy: { $why }
# $code: the controller process's exit code, as the code gives it.
farm-ctl-ended-exit = Kontroler zakończył działanie (kod wyjścia { $code }).

## Render on farm: the state, in the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-phase-waiting = Oczekiwanie na klientów renderowania (połączono { $connected } z { $needed })
farm-ctl-phase-preparing = Przygotowywanie zadania
farm-ctl-phase-rendering = Renderowanie
farm-ctl-phase-paused = Wstrzymano
farm-ctl-phase-complete = Ukończono
farm-ctl-phase-stopped = Zatrzymano
farm-ctl-phase-ended = Zakończono

## Render on farm: the words after the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-detail-waiting = Oczekiwanie na klientów renderowania: połączono { $connected } z { $needed }
farm-ctl-detail-preparing = Mierzenie kotwic normalizacji na tym komputerze, raz dla wszystkich klientów
farm-ctl-detail-no-space = Oczekiwanie na miejsce w folderze wyjściowym
farm-ctl-detail-paused = Wstrzymano: nowa praca nie jest przydzielana
farm-ctl-detail-not-rendered = Zakończono z niewyrenderowanymi klatkami — uruchom ponownie na tym samym folderze, aby wznowić
farm-ctl-detail-stopped = Zatrzymano — uruchom ponownie na tym samym folderze, aby wznowić
# $code: the controller process's exit code.
farm-ctl-detail-ended = Zakończono (kod wyjścia { $code })
farm-ctl-detail-finishing = Kończenie

## Render on farm: progress

# On the progress bar. $done: frames done; $total: frames in the job.
farm-ctl-progress =
    { $total ->
        [one] { $done } / { $total } klatka
        [few] { $done } / { $total } klatki
        [many] { $done } / { $total } klatek
       *[other] { $done } / { $total } klatki
    }
# Added to the progress bar. $rate: frames per second, with decimals.
farm-ctl-progress-rate = { $rate } kl./s
# Added to the progress bar. $time: the time still to go ("38m 10s").
farm-ctl-progress-eta = pozostało około { $time }
# Facts under the progress bar, joined with " · ". $received and $sent: kilobytes per second.
farm-ctl-io = odbiór { $received } KB/s · wysyłka { $sent } KB/s
# $space: free space on the output drive ("412 GB").
farm-ctl-free = wolne: { $space }
# $time: the time since the job started ("38m 10s").
farm-ctl-elapsed = upłynęło: { $time }
# $count: frames given up on.
farm-ctl-failed = niewyrenderowane: { $count }
# Reference orbits. $built: built for this job; $cached: found in the cache; $reused: reused between frames; $shared: passed between machines.
farm-ctl-refs = referencje: zbudowane { $built }, z pamięci podręcznej { $cached }, użyte ponownie { $reused } · udostępnione { $shared }
# After a warning sign.
farm-ctl-storage-low = Folder wyjściowy jest prawie pełny: nowa praca nie będzie przydzielana, dopóki nie zwolni się miejsce.
# After a warning sign.
farm-ctl-gpus-differ = GPU w tej farmie się różnią. Ich klatki nieco się różnią, co może być widoczne w statycznym ujęciu.
# After a warning sign. $count: the GPU classes in the farm.
farm-ctl-gpus-differ-classes = GPU w tej farmie się różnią (klasy GPU: { $count } — różne GPU, sterowniki lub rendery próbne). Ich klatki nieco się różnią, co może być widoczne w statycznym ujęciu.
# $gpu: this machine's GPU, API and driver.
farm-ctl-this-gpu = Ten komputer: { $gpu }
# A machine whose GPU differs. $name: the machine; $note: how its GPU differs (in English); $adapter: its GPU.
farm-ctl-gpu-row = { $name }: { $note } — { $adapter }
# The same with its driver. $driver: the driver's name and version.
farm-ctl-gpu-row-driver = { $name }: { $note } — { $adapter }, sterownik { $driver }

## Render on farm: the table of machines

# Column headers (short: the table is narrow).
farm-ctl-col-machine = Komputer
farm-ctl-col-state = Stan
farm-ctl-col-gpu = GPU
farm-ctl-col-frames = Klatki
farm-ctl-col-per-frame = Na klatkę
# How the machine delivers its frames (a speed, or "shared drive").
farm-ctl-col-link = Łącze
# When the controller last heard from the machine.
farm-ctl-col-heard = Kontakt
# In the machine's tooltip. $count: its frames that failed verification.
farm-ctl-strikes =
    { $count ->
        [one] { $count } klatka nie przeszła weryfikacji
        [few] { $count } klatki nie przeszły weryfikacji
        [many] { $count } klatek nie przeszło weryfikacji
       *[other] { $count } klatki nie przeszło weryfikacji
    }
# In the Link column: it delivers its frames through the shared drive.
farm-ctl-link-share = dysk współdzielony
# In the Heard column. $seconds: how long ago.
farm-ctl-heard-ago = { $seconds } s temu
farm-ctl-readmit = Przyjmij ponownie
farm-ctl-readmit-hover = Pozwól temu komputerowi dołączyć ponownie, gdy znów się połączy
farm-ctl-remove = Usuń
farm-ctl-remove-hover = Wyłącz ten komputer z zadania; jego klatki trafią do pozostałych
# The GPU column's tooltip.
farm-ctl-probe-same = Jego render próbny jest identyczny z renderem tego komputera.
# $count: the pixels that differ.
farm-ctl-probe-differs =
    { $count ->
        [one] Jego render próbny różni się od renderu tego komputera w { $count } pikselu: jego klatki będą się nieco różnić od klatek tego komputera.
        [few] Jego render próbny różni się od renderu tego komputera w { $count } pikselach: jego klatki będą się nieco różnić od klatek tego komputera.
        [many] Jego render próbny różni się od renderu tego komputera w { $count } pikselach: jego klatki będą się nieco różnić od klatek tego komputera.
       *[other] Jego render próbny różni się od renderu tego komputera w { $count } piksela: jego klatki będą się nieco różnić od klatek tego komputera.
    }
farm-ctl-probe-unknown = Jego render próbny nie został porównany.
# $driver: the driver's name and version.
farm-ctl-gpu-driver = sterownik { $driver }
# After a warning sign. $note: how its GPU differs (in English: "a different GPU").
farm-ctl-gpu-note = { $note } — w porównaniu z tym komputerem

## Render on farm: a machine's state, in the State column

# $count: its frames that failed verification.
farm-ctl-state-removed-bad =
    { $count ->
        [one] usunięty: { $count } zła klatka
        [few] usunięty: { $count } złe klatki
        [many] usunięty: { $count } złych klatek
       *[other] usunięty: { $count } złej klatki
    }
farm-ctl-state-removed = usunięty ręcznie
farm-ctl-state-removing = usuwanie — zbieranie diagnostyki
farm-ctl-state-gone = odłączony
farm-ctl-state-leaving = opuszcza farmę
farm-ctl-state-checking = sprawdzanie (autotest)
farm-ctl-state-paused = wstrzymany przez użytkownika
farm-ctl-state-parked = odstawiony: dwie awarie — dołączy ponownie po ponownym połączeniu
farm-ctl-state-refused = odrzucił to zadanie (własne limity)
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-ctl-state-rendering = renderuje { $frame } ({ $index }/{ $count })
farm-ctl-state-starting = rozpoczyna serię
farm-ctl-state-idle = bezczynny
farm-ctl-state-admitted = przyjęty — czeka na zadanie

## Render on farm: the buttons

# A collapsible section: the controller's messages.
farm-ctl-events = Zdarzenia
farm-ctl-resume = Wznów
farm-ctl-pause = Wstrzymaj
farm-ctl-pause-hover = Nie przydzielaj nowej pracy; bieżące klatki zostaną dokończone
farm-ctl-live-paused = Widok na żywo jest wstrzymany, gdy ten komputer renderuje dla farmy.
farm-ctl-close = Zamknij
farm-ctl-close-active-hover = Zamknij okno; zadanie nadal działa (otwórz je ponownie z menu Narzędzia)
farm-ctl-close-hover = Zamknij okno
farm-ctl-copy-command = Kopiuj polecenie
farm-ctl-copy-command-hover = Skopiuj równoważne polecenie wiersza poleceń
farm-ctl-stop = Zatrzymaj
# The Stop button once a stop has been asked for and the job has not ended after 10 seconds.
farm-ctl-stop-now = Zatrzymaj teraz
farm-ctl-stop-hover = Zakończ zadanie z dotychczasowymi klatkami; ponowne uruchomienie tego samego zadania je wznowi
farm-ctl-render = Renderuj
farm-ctl-render-hover = Zacznij nasłuchiwać klientów i renderuj na nich przelot

## The Render tour window (Tools ▸ Render tour…)

tour-render-title = Renderuj przelot
tour-render-out = Folder wyjściowy
tour-render-browse = Przeglądaj…
tour-render-out-hover = Pełna ścieżka, do której zapisywane są klatki.
tour-render-prefix = Prefiks klatek
# <prefix> stands for the prefix typed in the field.
tour-render-prefix-hover = Klatki są zapisywane jako <prefiks>_00000.png
tour-render-size = Rozmiar
tour-render-size-custom = Własny…
# The size list's choice when the size is typed in. $width and $height: pixels.
tour-render-size-custom-value = Własny — { $width }×{ $height }
tour-render-fps = Klatki / sekundę
tour-render-fps-hover = Mniej niż 1 fps jest w pełni uzasadnione: tak, próbkując długie zanurzanie z częstotliwością 0,25 fps, sprawdza się głębokie postoje testów regresji bez renderowania tysięcy klatek.
tour-render-ss = Supersampling
tour-render-chapter = Rozdział
tour-render-whole = Cały przelot
tour-render-chapter-hover = Renderowanie jednego rozdziału zachowuje GLOBALNĄ numerację klatek, więc jego klatki od razu pasują do pełnej sekwencji.
tour-render-mp4 = Złóż mp4
tour-render-mp4-hover = Uruchamia potem ffmpeg (musi być w PATH). Klatki PNG są zachowywane.
tour-render-overwrite = Nadpisuj istniejące klatki
tour-render-overwrite-hover = Wyłączone: renderowanie zatrzyma się, by zapytać, czego nie może zrobić z tego miejsca.
tour-render-resume = Wznów przerwane renderowanie
tour-render-resume-hover = Zachowuje klatki już obecne w folderze wyjściowym i renderuje tylko brakujące. Najnowsza klatka jest sprawdzana najpierw i odrzucana, jeśli jest niekompletna (klatka, na której renderowanie padło, to ta, którą właśnie zapisywało), z cofaniem się aż do znalezienia dobrej; folder z klatkami w innym rozmiarze jest odrzucany, a nie mieszany z tą sekwencją.
# $count: this machine's graphics cards.
tour-render-all-gpus = Użyj wszystkich kart graficznych ({ $count })
tour-render-all-gpus-hover = Renderuje na wszystkich kartach graficznych tego komputera jednocześnie, przez farmę renderującą na tym komputerze (--gpus all). Klatki, ich nazwy, wznawianie i mp4 są takie same jak dla jednej karty; renderowany jest cały przelot, w kolejności ustalonej przez farmę. Różne modele kart rysują kilka pikseli nieco inaczej: statyczne ujęcie i przenikanie pozostają na jednej karcie.
tour-render-order = Kolejność renderowania
tour-render-sequential = Sekwencyjna
tour-render-sequential-hover = Klatka 0, 1, 2, … po kolei (domyślnie).
tour-render-progressive = Progresywna
tour-render-progressive-hover = Najpierw podgląd całego przelotu, potem dopracowanie do pełnych szczegółów: najpierw renderują się klatki kluczowe, potem największe luki czasowe są dzielone na pół, aż powstanie każda klatka. W obu trybach klatki trafiają pod właściwe indeksy, więc wznawianie i mp4 działają bez zmian — można przerwać wcześniej, obejrzeć zgrubną animację poklatkową i wznowić (Wznów) w dowolnej z tych kolejności.
# The cost of the render. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
tour-render-summary =
    { $count ->
        [one] { $count } klatka · { $seconds }s przelotu · { $width }×{ $height } ss{ $ss }
        [few] { $count } klatki · { $seconds }s przelotu · { $width }×{ $height } ss{ $ss }
        [many] { $count } klatek · { $seconds }s przelotu · { $width }×{ $height } ss{ $ss }
       *[other] { $count } klatki · { $seconds }s przelotu · { $width }×{ $height } ss{ $ss }
    }
# The disk space the frames will take (an estimate). $size: a size ("5.3 GB").
tour-render-disk = ≈{ $size } klatek
# $size: the frames' estimated size; $free: the free space on that drive ("6.0 GB").
tour-render-disk-free = ≈{ $size } klatek · wolne: { $free }
# After a warning sign. $disk: the line above ("≈51.0 GB of frames · 6.0 GB free").
tour-render-disk-short = { $disk } — za mało miejsca na całą sekwencję
tour-render-separate = Renderuje skrypt Z DYSKU w osobnym procesie, więc niezapisane zmiany nie są uwzględniane — a utrata urządzenia GPU w tamtym procesie nie pociągnie za sobą tej sesji.
# On the progress bar. $done: frames done; $total: frames to render.
tour-render-progress =
    { $total ->
        [one] { $done } / { $total } klatka
        [few] { $done } / { $total } klatki
        [many] { $done } / { $total } klatek
       *[other] { $done } / { $total } klatki
    }
# Before the render's first progress line.
tour-render-starting = uruchamianie…
tour-render-live-paused = Widok na żywo wstrzymany — renderowanie ma GPU tylko dla siebie, więc kończy szybciej i dokładnie mierzy koszty własnych klatek.
tour-render-cancel = Anuluj
tour-render-cancel-hover = Zamknij bez renderowania
tour-render-copy-command = Kopiuj polecenie
tour-render-copy-command-hover = Skopiuj równoważne polecenie wiersza poleceń do schowka
tour-render-stop = Zatrzymaj renderowanie
tour-render-render = Renderuj
tour-render-render-hover = Zacznij renderować sekwencję klatek
# $folder: the output folder.
tour-render-finished = Renderowanie zakończone → { $folder }
# $why: the render's own reason (in English).
tour-render-failed = Renderowanie nie powiodło się: { $why }
# $last: the render's last progress line (in English), or nothing.
tour-render-failed-or-stopped = Renderowanie nie powiodło się lub zostało zatrzymane. { $last }
# $why: the system's reason.
tour-render-no-exe = Nie można znaleźć pliku wykonywalnego: { $why }
# $why: the system's reason.
tour-render-start-failed = Nie udało się uruchomić renderowania: { $why }

## The Custom formula dialog (Fractal ▸ Custom formula…)
## The formula language itself stays as it is in every language: function names (sin, cosh, conj…),
## operators, variables (z, c, p1), and the formulas. Keep them as written inside these messages.

# The window's title.
formula-dialog-title = Własny wzór
# A small line at the top of the window, above the formula.
formula-dialog-intro = Krok wzoru w wyrażeniach w stylu Fractinta
# The two ways to show and edit the formula (top right of the window).
formula-dialog-textbook = Podręcznik
formula-dialog-textbook-hover = Pokaż wzór złożony tak, jak składa go podręcznik, i edytuj go w tej postaci
formula-dialog-text = Tekst
formula-dialog-text-hover = Edytuj wzór jako tekst
# The tooltip of the "LaTeX" button (the button's own text stays LaTeX).
formula-dialog-latex-hover = Skopiuj wzór jako LaTeX, tak jak jest tu złożony
# The drop-down list of example formulas (the collection).
formula-dialog-examples = Przykłady
# Help under the formula in Textbook mode. The characters / ^ ( are what the user types: keep them.
formula-dialog-help-textbook = Pisz jak w tekście: / tworzy ułamek z wyrazu przed nim, ^ wykładnik, ( nawiasy. Strzałki przesuwają po wzorze; Tab przechodzi do następnego pustego pola. Wiersz na czerwono nie daje się odczytać: kliknij go, aby edytować go jako tekst.
# Help under the formula in Text mode. "; starts a comment": the ; is what the user types.
formula-dialog-help-text = Pisz lub użyj klawiatury ekranowej — zawiera każdą nazwę, którą zna język wzorów. Nazwy uzupełniają się podczas pisania (Tab). Instrukcje oddziela się nowym wierszem lub przecinkiem; ; rozpoczyna komentarz.
# The syntax check, when the formula reads.
formula-dialog-reads = Odczytano poprawnie.
# The syntax check in Textbook mode, when the formula has an empty box to fill in.
formula-dialog-empty-box = Wypełnij puste pole.
# The syntax check in Textbook mode. $line: the line number; $message: what is wrong (from the formula parser, in English).
formula-dialog-error-line = Wiersz { $line }: { $message }
# One function written with another notation's name, and the language's name for it: "ln to log".
# $from: the name as written (ln); $to: the formula language's name (log).
formula-respell-pair = { $from } na { $to }
# A button that rewrites them. $changes: one or more formula-respell-pair, joined with ", ".
formula-respell = Zmień { $changes }
formula-respell-hover = Zapisz te funkcje tak, jak nazywa je język wzorów
# Shown when Apply finds a parameter field that is not a number. $name: the parameter (p1 … p5).
formula-param-not-numbers = { $name } nie jest parą liczb
# The Apply button (also in the Formula library window).
formula-apply = Zastosuj
formula-dialog-apply-hover = Skompiluj wzór i pokaż go
# Shown beside Apply while the formula is being compiled.
formula-dialog-compiling = Kompilowanie wzoru dla GPU…
# The greyed hint in an empty name box (also in the Formula library window).
formula-name-hint = nazwa (opcjonalnie)
# The save button: "Update" when the library already has a formula of that name.
formula-dialog-update-in-library = Aktualizuj w bibliotece
formula-dialog-save-to-library = Zapisz w bibliotece
formula-dialog-save-hover = Zachowaj ten wzór i jego parametry w bibliotece wzorów
formula-dialog-save-disabled-hover = Zapisać można tylko wzór, który odczytuje się poprawnie
# Opens the Formula library window.
formula-dialog-library = Biblioteka…
formula-dialog-library-hover = Zapisane wzory: stosowanie, edycja, import i eksport

## The Custom formula dialog's keypad
## The keys' captions are formula tokens and stay as they are; these are their tooltips.

# The keypad's third tab (the other two are "123" and "f(z)"). A short word.
formula-keypad-tab-names = nazwy
# The tabs' tooltips.
formula-keypad-tab-basic-hint = Liczby, operatory i najczęściej używane nazwy
formula-keypad-tab-functions-hint = Funkcje
formula-keypad-tab-names-hint = Wszystkie zmienne, parametry i stałe oraz składnia instrukcji
# The key z.
formula-key-z = Iteracja
# The key c.
formula-key-c = Piksel (stała Julii w trybie Julii)
# The key p1 on the first tab.
formula-key-p1-below = Parametr 1 — ustawiany w polach poniżej
# The keys p1 … p5. $n: the parameter's number.
formula-key-param = Parametr { $n }
formula-key-multiply = Mnożenie
formula-key-divide = Dzielenie
# The key that types ^2.
formula-key-square-after = Kwadrat (po wartości: z^2)
# The key that types ^.
formula-key-power = Potęga: wykładnik całkowity, rzeczywisty lub zespolony
formula-key-square-root = Pierwiastek kwadratowy
# The key that types |…|: in this language it is the squared modulus, as in Fractint.
formula-key-squared-modulus = KWADRAT modułu |z|² — jak we Fractincie
formula-key-add = Dodawanie
formula-key-subtract = Odejmowanie (lub zmiana znaku)
# The key that types =.
formula-key-assign = Przypisanie: z = … lub zmienna tymczasowa t = …
# The key that starts a new line (first tab).
formula-key-next-statement-comma = Następna instrukcja (przecinek też działa)
# The key "del".
formula-key-backspace = Usuń znak przed kursorem (lub zaznaczenie)
# The key e.
formula-key-euler = Liczba Eulera
# The key that types (0.5, 0.5).
formula-key-complex-constant = Stała zespolona: (rzeczywista, urojona)
# The key that types a comma.
formula-key-comma = Oddziela instrukcje lub części stałej zespolonej
formula-key-left = Kursor w lewo
formula-key-right = Kursor w prawo
# The function keys (the f(z) tab), one per function.
formula-key-sin = Sinus
formula-key-cos = Cosinus
formula-key-tan = Tangens
formula-key-cotan = Cotangens
formula-key-exp = Funkcja wykładnicza e^z
formula-key-sinh = Sinus hiperboliczny
formula-key-cosh = Cosinus hiperboliczny
formula-key-tanh = Tangens hiperboliczny
formula-key-cotanh = Cotangens hiperboliczny
formula-key-log = Logarytm naturalny, ln (gałąź główna)
formula-key-sqr = Kwadrat: sqr(z) = z²
formula-key-sqrt = Pierwiastek kwadratowy (główny)
formula-key-recip = Odwrotność 1/z
formula-key-conj = Sprzężenie zespolone
formula-key-flip = Zamiana części rzeczywistej i urojonej
formula-key-abs = Wartość bezwzględna OBU części: |x| + i|y|
formula-key-cabs = Moduł |z| (liczba rzeczywista)
formula-key-real = Część rzeczywista
formula-key-imag = Część urojona, jako liczba rzeczywista
# "fn" is Fractint's name for a function slot: keep it.
formula-key-ident = Identyczność (symbol zastępczy fn z Fractinta)
# The key "pixel".
formula-key-pixel = To samo co c (nazwa z Fractinta)
# The key "z =".
formula-key-set-z = Rozpocznij instrukcję ustawiającą z
# The key "t =".
formula-key-temporary = Zmienna tymczasowa: działa każda nowa nazwa (t, w, a2, …)
# The key "; …".
formula-key-comment = Komentarz do końca wiersza
# The key that starts a new line (names tab).
formula-key-next-statement = Następna instrukcja

## The formula's completion list (names offered as you type)

# Beside a name the formula itself assigns.
formula-completion-variable = Zmienna ustawiana przez ten wzór
# Beside a function typed with another notation's name.
# $what: what the function is, in English (from the formula parser: "The natural logarithm");
# $other: the other notation's name (ln); $ours: the formula language's name (log).
formula-completion-other-spelling = { $what } — { $other } zapisuje się tutaj jako { $ours }
# Under a list cut short. $count: the names not shown.
formula-completion-more = … i jeszcze { $count }
# The last line of the list. Tab, Enter and Esc are key names.
formula-completion-keys = Tab lub Enter — uzupełnij, Esc — zamknij

## The Custom formula dialog's Textbook mode

# The tooltip of a line that does not read. $why: what is wrong (from the formula parser, in English, or formula-textbook-fractint-sections).
formula-textbook-unread-hover = Nie daje się odczytać: { $why }. Kliknij, aby edytować jako tekst.
# Why a line with Fractint's sections is shown as text in Textbook mode.
formula-textbook-fractint-sections = Sekcje Fractinta (':' sekcji inicjalizacji, porównania, a = b = …) edytuje się jako tekst

## The Formula library window (Fractal ▸ Formula library…)

# The window's title.
formula-library-title = Biblioteka wzorów
# The window's two tabs. $count: the formulas in each.
formula-library-mine = Moje wzory ({ $count })
formula-library-mine-hover = Wzory, które zostały zapisane lub zaimportowane
formula-library-collection = Kolekcja ({ $count })
formula-library-collection-hover = Wzory dostarczane z Fractadyne, każdy z widokiem początkowym
# A small marker before the name of the formula the view shows now. A short word.
formula-library-showing = wyświetlany
# The tooltip of Apply on a formula of the collection.
formula-library-apply-shipped-hover = Pokaż go w jego widoku początkowym
# A row's Edit button (after a pencil icon).
formula-library-edit = Edytuj
formula-library-edit-hover = Otwórz go w oknie Własny wzór, bez stosowania
# A collection row's button that copies the formula into "My formulas".
formula-library-copy-to-mine = Kopiuj do moich
formula-library-copy-to-mine-hover = Dodaj go do Moich wzorów, aby go zachować lub zmienić
formula-library-close = Zamknij
# Saves the formula the view shows, under the name typed beside it.
formula-library-add-current = Dodaj bieżący wzór
formula-library-add-current-hover = Zapisz wzór pokazywany w widoku
# "Fractal > Custom formula…" is the menu path: write it as the menus are named.
formula-library-add-current-disabled-hover = Najpierw zastosuj własny wzór (Fraktal > Własny wzór…)
formula-library-import = Importuj…
formula-library-import-hover = Dodaj wzory z pliku wzorów; nic tutaj nie zostanie zastąpione
formula-library-export-all = Eksportuj wszystkie…
formula-library-export-all-hover = Zapisz wszystkie tutejsze wzory do jednego pliku, aby je udostępnić lub zachować
# The filter box: its label, and the greyed hint inside it.
formula-library-filter = Filtr
formula-library-filter-hint = filtruj według nazwy lub tekstu
# Shown while the user has no saved formulas.
formula-library-empty = Nie ma jeszcze zapisanych wzorów. Zapisz wzór z okna Własny wzór, dodaj ten, który pokazuje widok, lub zaimportuj plik (w formacie Fractadyne albo .frm Fractinta).
# The tooltip of Apply on one of the user's formulas: with a view saved, and without one.
formula-library-apply-view-hover = Pokaż ten wzór w zapisanym z nim widoku
formula-library-apply-hover = Pokaż ten wzór
# The tooltip of a row's save icon.
formula-library-export-one-hover = Eksportuj ten wzór do pliku
# A row asks before deleting: the question, then its two buttons. "Delete" is also the delete icon's tooltip.
formula-library-delete-ask = Usunąć?
formula-library-delete = Usuń
formula-library-keep = Zachowaj
# Under a list cut short. $count: the formulas not listed.
formula-library-more = …i jeszcze { $count }: wpisz tekst w filtrze, aby znaleźć wzór.
formula-library-no-match = Żaden wzór nie pasuje do filtra.
# Under a saved formula this version cannot read. $why: what is wrong (from the formula parser, in English).
formula-library-unread = Nie daje się odczytać w tej wersji: { $why }

## Formula library notices

# $why: the system's reason, in English.
formula-library-save-failed = Nie udało się zapisać biblioteki wzorów: { $why }
formula-library-nothing-to-save = Nie ma czego zapisać: wzór jest pusty.
# $name: the formula's name in the library.
formula-library-updated = Zaktualizowano „{ $name }” w bibliotece wzorów.
formula-library-saved = Zapisano „{ $name }” w bibliotece wzorów.
# $name: the formula's name; $why: what is wrong.
formula-library-apply-failed = Nie można zastosować „{ $name }”: { $why }
# Copying a formula of the collection into "My formulas". $name: its name; $as_named: the name it was given (the same, or "Name (2)" when the name was taken).
formula-library-copy-exists = „{ $name }” jest już w Moich wzorach.
formula-library-copied = Skopiowano „{ $name }” do Moich wzorów jako „{ $as_named }”.
# $name: the formula's name.
formula-library-deleted = Usunięto „{ $name }” z biblioteki wzorów.
# Why the library could not be saved (follows formula-library-save-failed).
formula-library-no-config-dir = brak katalogu konfiguracji
# At startup, when the library file cannot be read. $why: the reason, in English; $path: a file's full path.
formula-library-unreadable-kept = Nie udało się odczytać biblioteki wzorów ({ $why }). Zachowano ją jako { $path } i rozpoczęto nową, pustą bibliotekę.
formula-library-unreadable-stuck = Nie udało się odczytać biblioteki wzorów ({ $why }) ani przenieść jej na bok: skopiuj { $path } w bezpieczne miejsce przed zapisaniem wzoru, który ją zastąpi.

## Formula library: Import and Export
## A formula file's contents stay in English; this is what the windows and notices around them say.

# File-type names in the open and save dialogs (the extensions are added by the system).
formula-filter-formula-files = Pliki wzorów
formula-filter-fractadyne = Wzory Fractadyne
formula-filter-fractint = Wzory Fractinta
formula-filter-all-files = Wszystkie pliki
# Why a file cannot be imported (they follow formula-import-failed).
formula-import-too-large = o wiele za duży jak na plik wzorów
formula-import-not-text = to nie jest plik tekstowy
formula-import-no-formulas = nie zawiera wzorów
# $why: the TOML reader's message, in English.
formula-file-not-formulas = to nie jest plik wzorów ({ $why })
# $format: what the file says it is ("fractadyne-gradients").
formula-file-other-format = plik „{ $format }”, a nie plik wzorów
# "[[formula]]" is the file's own syntax: keep it.
formula-file-no-formulas = nie zawiera wzorów (plik wzorów wymienia je jako tabele [[formula]])
# $file: the file's name; $why: one of the reasons above, or the system's (in English).
formula-import-failed = Nie udało się zaimportować „{ $file }”: { $why }.
# After an import that added nothing. $file: the file's name.
formula-import-nothing-new = Nic nowego w „{ $file }”.
# After an import. $count: the formulas added; $file: the file's name.
formula-import-added =
    { $count ->
        [one] Zaimportowano { $count } wzór z „{ $file }”.
        [few] Zaimportowano { $count } wzory z „{ $file }”.
        [many] Zaimportowano { $count } wzorów z „{ $file }”.
       *[other] Zaimportowano { $count } wzoru z „{ $file }”.
    }
# The same two, followed by what else happened. $notes: one or more of the formula-import-note-… messages below, joined with "; ".
formula-import-nothing-new-notes = Nic nowego w „{ $file }”: { $notes }.
formula-import-added-notes =
    { $count ->
        [one] Zaimportowano { $count } wzór z „{ $file }”: { $notes }.
        [few] Zaimportowano { $count } wzory z „{ $file }”: { $notes }.
        [many] Zaimportowano { $count } wzorów z „{ $file }”: { $notes }.
       *[other] Zaimportowano { $count } wzoru z „{ $file }”: { $notes }.
    }
# Notes (clauses, no final full stop). $count: the formulas already in the library, skipped.
formula-import-note-duplicates =
    { $count ->
        [one] { $count } wzór był już w bibliotece
        [few] { $count } wzory były już w bibliotece
        [many] { $count } wzorów było już w bibliotece
       *[other] { $count } wzoru było już w bibliotece
    }
# One formula was renamed because its name was taken. $from: its name in the file; $to: the name it was given ("Name (2)").
formula-import-note-renamed-one = nazwa „{ $from }” była zajęta, więc wzór nazywa się „{ $to }”
# $count: the formulas renamed because their names were taken (2 or more).
formula-import-note-renamed =
    { $count ->
        [one] { $count } wzór przemianowano, bo jego nazwa była zajęta
        [few] { $count } wzory przemianowano, bo ich nazwy były zajęte
        [many] { $count } wzorów przemianowano, bo ich nazwy były zajęte
       *[other] { $count } wzoru przemianowano, bo ich nazwy były zajęte
    }
# $count: the entries skipped as empty or too long.
formula-import-note-skipped =
    { $count ->
        [one] { $count } wpis był pusty lub za długi, pominięto go
        [few] { $count } wpisy były puste lub za długie, pominięto je
        [many] { $count } wpisów było pustych lub za długich, pominięto je
       *[other] { $count } wpisu było pustych lub za długich, pominięto je
    }
# A second sentence after importing a Fractint .frm file. $count: the entries that do not read; $why: the commonest reason (from the formula parser, in English).
formula-import-frm-unread =
    { $count ->
        [one] { $count } wpis nie daje się odczytać w tej wersji (najczęściej: { $why }).
        [few] { $count } wpisy nie dają się odczytać w tej wersji (najczęściej: { $why }).
        [many] { $count } wpisów nie daje się odczytać w tej wersji (najczęściej: { $why }).
       *[other] { $count } wpisu nie daje się odczytać w tej wersji (najczęściej: { $why }).
    }
# After exporting one formula. $name: the formula's name; $file: the file's name.
formula-export-one = Wyeksportowano „{ $name }” do „{ $file }”.
# After exporting several. $count: how many; $file: the file's name.
formula-export-many =
    { $count ->
        [one] Wyeksportowano { $count } wzór do „{ $file }”.
        [few] Wyeksportowano { $count } wzory do „{ $file }”.
        [many] Wyeksportowano { $count } wzorów do „{ $file }”.
       *[other] Wyeksportowano { $count } wzoru do „{ $file }”.
    }
# $file: the file's name; $why: the system's reason, in English.
formula-export-failed = Nie udało się zapisać „{ $file }”: { $why }

## The control panel (right-hand side): header and section headings

# The panel's title, beside the button that hides it.
panel-title = Sterowanie
panel-hide-hover = Ukryj panel sterowania
# The button that brings the hidden panel back (top right of the view).
panel-show-hover = Pokaż panel sterowania
# Section headings.
panel-section-navigate = Nawigacja
panel-section-coloring = Kolorowanie
panel-section-quality = Jakość
panel-section-effects = Efekty
panel-section-overlays = Nakładki
panel-section-advanced = Zaawansowane
# The last section: reference notes about the current fractal. $name: the fractal's name (Mandelbrot, Burning Ship…).
panel-section-about = Informacje: { $name }
panel-section-performance = Wydajność

## The control panel: Navigate
## Labels before a control sit in a column about 104 px wide (roughly 15 characters): a longer label pushes its control to the right.

panel-zoom-speed = Szybkość zoomu
panel-zoom-speed-hover = Szybkość ciągłego zoomu przy przytrzymanym klawiszu Space (1× ≈ 2× na 1,5 s).
panel-snap-zoom = Przyciągaj zoom do struktur
panel-snap-zoom-hover = Podczas przybliżania kółkiem myszy lub klawiszem Space przyciąga zoom do minibrota (lub osi rzeczywistej) w pobliżu kursora, tak że struktura wsuwa się pod kursor, a zoom na niej ląduje. Im bliżej struktura, tym silniejsze przyciąganie. Tylko Mandelbrot, widok pojedynczy.
panel-click-zoom = Zoom kliknięciem
panel-click-zoom-hover = Gdy jest włączone, kliknięcie lewym przyciskiem w widoku zanurza o poniższy współczynnik (prawy przycisk cofa), z wyśrodkowaniem na klikniętym punkcie. Przeciąganie nadal przesuwa; Shift+przeciąganie / przeciąganie prawym przyciskiem nadal wykonuje zoom prostokątem. Backspace cofa kliknięcie. W widoku podwójnym działa na obu panelach, a przypinanie c Julii przenosi się wtedy na Ctrl+kliknięcie. Przytrzymaj Shift, aby pod kursorem pojawiła się lupa pokazująca widok, do którego przeniesie kliknięcie, i kliknij, trzymając Shift, aby wybrać ten punkt; Shift+PRZECIĄGANIE nadal wykonuje zoom prostokątem.
# The click-to-zoom factor, followed by buttons 2×, 4×, 10×…
panel-click-zoom-factor = Współczynnik
panel-click-zoom-snap = Przyciągaj do najbliższego środka
panel-click-zoom-snap-hover = Po skoku wyznacza najbliższe jądro minibrota i ustawia widok dokładnie na nim, więc trafia się na strukturę, a nie obok niej. Zoom się nie zmienia — wybrało go kliknięcie. Tylko Mandelbrot. Obliczenie działa w tle: widok skacze od razu i koryguje się, gdy nadejdzie wynik, co na ekstremalnej głębokości może potrwać kilka sekund.

## The control panel: Coloring
## Labels before a control sit in a column about 104 px wide (roughly 15 characters).

panel-method = Metoda
panel-method-hover = Sposób odwzorowania danych ucieczki na kolor. Średnia pasków / nierówność trójkąta / pułapka orbity / dekompozycja ujawniają strukturę orbity; estymacja odległości cieniuje według bliskości brzegu.
panel-stripe-density = Gęstość pasków
panel-stripe-tail = Tylko ogon (głęboki zoom)
panel-stripe-tail-hover = Uśrednia człon pasków z ostatnich N iteracji każdej orbity (okno wykładnicze) zamiast z całej orbity. Na ekstremalnej głębokości każda orbita podąża za referencją przez wszystkie iteracje z wyjątkiem ostatnich kilkuset, więc średnia z całej orbity daje jeden kolor dla całego widoku; ogon zachowuje kontrast. Orbity krótsze niż okno pozostają bez zmian. Także --stripe-tail N dla --render.
panel-tail-length = Długość ogona
# The unit after the tail length's number: short for "iterations" (the slider reads "64 it").
panel-tail-length-unit = it.
panel-tail-length-hover = Długość okna w iteracjach. Krótkie okna (8–32) dają klasyczne wyraźne pasma na każdej głębokości; dłuższe znów uśredniają się do płaskiego koloru, bo człon pasków długiej chaotycznej orbity się uśrednia.
panel-trap-shape = Kształt pułapki
panel-palette = Paleta
# The two colours of the Binary palette, each beside its colour swatch.
panel-in-set = W zbiorze
panel-out-of-set = Poza zbiorem
# The two colours of the Duotone palette, each beside its colour swatch.
panel-shadow = Cień
panel-highlight = Światło
panel-cycle = Cykl
panel-offset = Przesunięcie
panel-normalize-hover = Na ekstremalnej głębokości liczby iteracji do ucieczki sięgają setek tysięcy, a stały Cykl zawija paletę tysiące razy między sąsiednimi pikselami — poprawne gęste pole wygląda jak szum plamek. To ustawienie przemapowuje paletę na zmierzony zakres ucieczki widoku (Cykl ustala wtedy, ile obiegów palety go obejmuje) TYLKO WTEDY, gdy widok rzeczywiście ma aliasing — gładki widok pozostaje bez zmian, bo przemapowanie zmieniłoby obraz bez powodu. Zaznacz poniżej „Zawsze dopasuj zakres”, aby przemapować mimo to. Tylko metoda gładka; odpowiada --normalize.
# A checkbox indented under "Normalize deep colors" (also in the Color menu).
panel-fit-range = Zawsze dopasuj zakres
panel-fit-range-hover =
    Przemapowuje paletę na zmierzony zakres ucieczki widoku, nawet gdy widok nie ma aliasingu — pole wyboru powyżej przemapowuje tylko wtedy, gdy gęste pole wyglądałoby inaczej jak drobne plamki.

    Użyj tego, gdy paleta ma obejmować to, co jest na ekranie, a nie usuwać szum. Nadal potrzebny jest pomiar, więc przez chwilę lub dwie przed ustaleniem się widoku nic nie robi, a w widoku bez pikseli, które uciekły, nie robi nic.
# A checkbox indented under "Always fit range" (also in the Color menu).
panel-log-scale = Logarytmiczna skala kolorów
panel-log-scale-hover =
    Rozkłada paletę według LOGARYTMU wartości ucieczki zamiast liniowo. Liczby iteracji do ucieczki skupiają się w głębi przy górnym końcu, więc odwzorowanie liniowe zużywa większość palety na cienką warstwę przy brzegu i spłaszcza całą resztę.

    Dotyczy odwzorowania znormalizowanego — tego, które włącza „Normalizuj głębokie kolory” (lub --normalize przy eksporcie). Mocno skośny zakres i tak dostaje odwzorowanie logarytmiczne, niezależnie od tego, czy to pole jest zaznaczone.
# The same checkbox's tooltip while it is greyed out (normalization is off).
panel-log-scale-off-hover =
    Wymaga normalizacji: to ustawienie wybiera, JAK paleta jest odwzorowywana na zmierzony zakres ucieczki, więc gdy „Normalizuj głębokie kolory” jest wyłączone (i nie ma eksportu z --normalize), nie ma odwzorowania, które mogłoby zmienić, i nie zmieniłoby ani jednego piksela.

    Włącz „Normalizuj głębokie kolory”, aby z niego korzystać.
# The palette animation: its kind, then its speed.
panel-animate = Animacja
panel-anim-speed = Szybkość
panel-anim-speed-hover = Szybkość cyklu: cykle przesunięcia koloru na sekundę lub (Losowe gradienty) zmiany gradientu na sekundę.
panel-shuffle-gradient = Losuj gradient

## The control panel: Quality

panel-auto-iter = Autoskalowanie iteracji z zoomem
# The iteration-count label while auto-scaling is on (the count is then a base the zoom raises).
panel-iterations-base = Iteracje (baza)
panel-iterations = Iteracje
panel-iterations-hover = Bazowa liczba iteracji. Przy włączonym autoskalowaniu efektywna liczba rośnie z głębokością zoomu. Podczas ruchu podgląd dla płynności ogranicza iteracje do niskiej wartości (50 000), a gdy widok się ustali, wyostrza się do pełnej liczby — dzięki temu głębokie krawędzie wyglądają gładko w ruchu i nabierają szczegółów po zatrzymaniu.
# A warning under the iteration count. $count: the iterations the settled preview renders.
panel-res-limited = Na tej głębokości ustalony podgląd renderuje { $count } iteracji w zmniejszonej rozdzielczości, aby zachować płynność. Eksportuj, aby uzyskać szczegóły w pełnej rozdzielczości.
panel-anti-alias = Antyaliasing
# Anti-aliasing off (the other choices are 2×, 3×, 4×, 8×).
panel-anti-alias-off = Wył.
panel-anti-alias-hover = Supersampling dla nieruchomych obrazów (stosowany, gdy widok się ustali). Wyższe wartości poskramiają drobny zewnętrzny „pył” kosztem czasu renderowania.

## The control panel: Effects

panel-relief-lighting = Oświetlenie reliefu 3D
panel-relief-lighting-hover = Cieniuje powierzchnię na podstawie nachylenia estymacji odległości — wytłoczony, oświetlony wygląd. (Rodziny holomorficzne: Mandelbrot / Multibrot.)
panel-light-angle = Kąt światła
panel-relief = Relief
panel-relief-hover = Mniej = ostrzejszy relief; więcej = łagodniejszy/bardziej płaski.
panel-rotate-light = Obracaj światło
panel-rotate-light-hover = Obraca kierunek światła w czasie (używa suwaka Szybkość).
panel-distance-glow = Poświata odległości
panel-distance-glow-hover = Jasne pasma konturowe estymacji odległości, które w pobliżu brzegu zagęszczają się w świecące włókna. (Rodziny holomorficzne.)
panel-glow = Poświata
panel-band-width = Szerokość pasm
panel-band-width-hover = Odstęp konturów odległości (oktawy na pasmo).
panel-animate-glow = Animuj poświatę
panel-animate-glow-hover = Przesuwa pasma poświaty w czasie (używa suwaka Szybkość).

## The control panel: Overlays

panel-orbits-hover = Rysuje drogę iteracji punktu pod kursorem.
panel-orbit-anim = Animacja (pędząca kropka)
panel-orbit-anim-hover = Wysyła wzdłuż orbity pędzącą kropkę zmieniającą kolory.
panel-orbit-speed = Szybkość orbity
panel-minimap-hover = Mały podgląd widoku początkowego ze znacznikiem „jesteś tutaj” i głębokością zoomu. Kliknięcie przenosi do wskazanego obszaru.

## The control panel: Advanced
## Labels before a control sit in a column about 104 px wide; most of these are already wider in English.

# Small headings of the section's two groups.
panel-accelerators = Przyspieszenia
panel-performance-tuning = Dostrajanie wydajności
panel-bla = Przyspieszenie BLA (głęboki zoom)
panel-bla-hover = Aproksymacja dwuliniowa: pomija iteracje na całej długości orbity na ekstremalnej głębokości (Mandelbrot floatexp, ≥1e28×) — ok. 5× szybsze renderowanie na GPU, identyczny wynik (sprawdzony przez autotest). Domyślnie włączone; wyłącz, aby porównać lub jeśli pojawi się artefakt.
panel-series-approx = Aproksymacja szeregiem
panel-series-approx-hover = Inicjuje perturbację wielomianem, aby pominąć wczesne iteracje tam, gdzie BLA nie działa (głębokości df32, Multibrot, wyłączone BLA). Identyczny wynik; wyłącz, aby porównać.
panel-glitch-correct = Korekcja glitchy (eksport)
panel-glitch-correct-hover = Korekcja glitchy z wieloma referencjami dla eksportowanych obrazów: wykrywa glitche perturbacji i renderuje te piksele ponownie względem dodatkowych referencji, aż będą czyste. Domyślnie włączona. Eksporty do ~32 MP (kolorowanie bez danych pomocniczych); większe obrazy i widok na żywo używają zwykłej ścieżki. Nigdy nie jest używana dla widoków Mandelbrot ani Multibrot (poza trybem Julii): sprawdzona względem dokładnych obliczeń, niczego tam nie poprawiała, a wydłużała eksport nawet 5-krotnie.
panel-dive-limit = Limit głębokości autozoomu
panel-dive-limit-hover = Głębokość, na której zatrzymuje się autozoom (klawisz A). Do ~1e271× zanurza się płynnie; głębiej przełącza się na nierówne, skokowe zanurzanie, aby szybko dotrzeć na ekstremalną głębokość.
# Followed by two choices, Speed and Quality.
panel-autopilot-priority = Priorytet autozoomu
panel-priority-speed-hover = Zoom z ustawioną szybkością. Ekran odświeża się, gdy tylko gotowa jest klatka ze szczegółami, w takiej rozdzielczości, która zapewnia ciągłe odświeżanie (poniżej progu Min. rozdzielczość ruchu, jeśli zoom wyprzedza renderowanie); między odświeżeniami ostatnia klatka podąża za zoomem i się rozmywa.
panel-priority-quality-hover = Każde odświeżenie to w pełni dopracowana klatka w pełnej rozdzielczości, a zoom w razie potrzeby zwalnia, aby klatka na ekranie nigdy nie powiększyła się zbytnio przed nadejściem następnej. Wolniejsze zanurzanie, ostre w każdej chwili.
# Followed by two choices, Detail and Misiurewicz point.
panel-autopilot-target = Cel autozoomu
panel-target-detail-hover = Zanurza się w stronę najbogatszej struktury na krawędzi fraktala w widoku, wybierając cel na nowo w trakcie zanurzania.
panel-target-misiurewicz-hover = Znajduje najbliższy punkt Misiurewicza (środek spirali, której struktura powtarza się w nieskończoność) i zanurza się dokładnie w niego. Zanurzanie najpierw stoi w miejscu, dopóki punkt nie zostanie wyznaczony na głębokość limitu zanurzania — przy zwykłych głębokościach to chwila, powyżej 1e1000× dłużej. Tylko zbiór Mandelbrota.
panel-work-budget = Budżet na żywo
panel-work-budget-hover = Szczegóły a szybkość widoku na żywo przy głębokim zoomie. Wyższe wartości renderują w pełniejszej rozdzielczości (ostrzej — mniej „miękkiego”, przeskalowanego wyglądu), ale obniżają liczbę klatek na sekundę, a bardzo wysokie grożą krótkim zawieszeniem GPU przy najcięższych klatkach. Eksporty niezależnie od tego zawsze mają pełną rozdzielczość.
panel-min-motion-res = Min. rozdzielczość ruchu
panel-min-motion-res-hover = Najniższa rozdzielczość, do jakiej może spaść widok na żywo podczas ciągłego zoomu lub przesuwania przy głębokim zoomie. Wyższa wartość chroni szybkie zanurzanie przed blokowością/pikselizacją (ostrzej) kosztem liczby klatek — przy 100% odświeżenia szczegółów głębokiego zanurzania muszą być renderowane w pełnym koszcie, co może sprawić, że głęboki zoom będzie widocznie skokowy/przycinający się. Jeśli głębokie zanurzanie wydaje się szarpane, obniż tę wartość. Domyślnie 30%; nie wpływa na ustalony obraz ani eksporty.
panel-finish-sound = Dźwięk po zakończeniu renderowania
panel-finish-sound-hover = Odtwarza melodię zakończenia z FRACTINT, gdy kończy się eksport lub renderowanie przelotu: trzy rosnące nuty (1047, 1109, 1175 Hz po 100 ms każda), dokładnie tak, jak zakodował je oryginał dla DOS — syntetyzowane jako fala prostokątna o ograniczonym paśmie i przepuszczone przez filtr górnoprzepustowy, by brzmiały jak mały głośniczek PC, który je odtwarzał.
panel-prefer-detail = Preferuj szczegóły podczas zoomu
panel-prefer-detail-hover = Podczas zoomu lub przesuwania pokazuje ostatnią w pełni szczegółową klatkę — przeskalowaną i śledzącą ruch — zamiast co klatkę renderować w obniżonej jakości. Widok renderuje się w pełni, gdy ruch się zatrzyma. Wyłączone: ruch renderuje się na żywo w obniżonej rozdzielczości (płynniej, ale zgrubniej). Płytkie widoki (tryb bezpośredni) zawsze renderują się na żywo — i tak są tanie i ostre w każdej klatce.

## The control panel: About (the current fractal)

# A link to the fractal's reference page, followed by an arrow.
panel-about-reference = Źródło

## The control panel: Advanced ▸ Second graphics card

# A label before a drop-down (wider than the panel's label column already).
gpu2-label = Druga karta graficzna
gpu2-off = Wył.
# Another device on the same graphics card as the window (the default).
gpu2-same-card = Ta sama karta
# A card the list does not name. $number: the card's number.
gpu2-card-n = Karta { $number }
gpu2-finding = Wyszukiwanie kart graficznych tego komputera…
gpu2-none = Ten komputer nie ma innej karty graficznej.
gpu2-hover =
    Inne urządzenie graficzne renderuje klatki głębokiego widoku, gdy ten się porusza, oraz część dodatkowych próbek, które wyostrzają go po zatrzymaniu, dzięki czemu obraz nadąża za zoomem i szybciej osiąga ostateczną jakość. Karta rysująca okno nadal rysuje wszystko.

    Inna karta dodaje własną szybkość. Dwa różne modele kart rysują kilka pikseli nieco inaczej; jeśli druga karta zawiedzie, widok działa dalej z jedną.

    „Ta sama karta” (domyślnie) otwiera drugie urządzenie na tej karcie. Nie dodaje szybkości, ale renderuje bez czekania na klatki ekranu. Dzieli kartę: awaria, która resetuje kartę, kończy działanie aplikacji, tak jak awaria renderowania samego okna, a przy następnym uruchomieniu pojawi się propozycja jego wyłączenia.

## The view (canvas)

# A notice after Ctrl+click pins the Julia c at a deep zoom.
canvas-julia-pin-quantized = Przypięto c Julii — ale jest ono przechowywane w podwójnej precyzji, więc na tej głębokości cały panel zaokrągla się do jednej wartości, a przypięcie trafia w środek widoku. Oddal, aby wycelować.
# A small caption in the top-left corner of the view while the orbit overlay is fitted to the view (its "Normalize (fit to view)" option).
canvas-orbit-normalized = orbita (znormalizowana)
# The minimap's tooltip.
canvas-minimap-hover = Podgląd / jesteś tutaj. Przeciągnij, aby przesuwać przy bieżącym zoomie, przewiń, aby przybliżać i oddalać, kliknij, aby skoczyć do tego obszaru (zoom początkowy).

## The timestamp overlay (View ▸ Show timestamp)

# Under the large elapsed-time clock. $frame: the frame number (as the log writes it, without grouping).
overlay-timestamp-frame = klatka { $frame }

## File ▸ Settings ▸ Reference cache…

cache-title = Pamięć podręczna referencji
cache-intro = Na ekstremalnej głębokości wolną częścią widoku jest jego orbita referencyjna — nawet godzina arytmetyki dowolnej precyzji. Fractadyne przechowuje tutaj gotowe orbity, więc powrót do odwiedzonej lokalizacji lub zoom gdzieś w jej pobliżu trwa zamiast tego kilka sekund. Jedna orbita to około 4 MB.
cache-off = Wyłączona dla tego uruchomienia (wywołanie zadania lub --no-orbit-cache). Nic nie jest odczytywane ani zapisywane aż do następnego zwykłego uruchomienia.
# Small headings of the window's parts.
cache-location = Położenie
cache-usage = Użycie
cache-limit = Limit
cache-no-config = (brak dostępnego katalogu konfiguracji)
cache-open-folder = Otwórz folder
# The text on the usage bar. $used and $limit: sizes such as "1.2 GB"; $count: the orbits kept.
cache-usage-bar =
    { $count ->
        [one] { $used } z { $limit } — jedna orbita
        [few] { $used } z { $limit } — { $count } orbity
        [many] { $used } z { $limit } — { $count } orbit
       *[other] { $used } z { $limit } — { $count } orbity
    }
cache-limit-hover = Gdy pamięć podręczna przekroczy ten rozmiar, najpierw usuwane są orbity, których budowa była NAJTAŃSZA — głęboka orbita budowana przez godzinę przetrwa dowolną liczbę szybkich, niezależnie od tego, jak niedawno ich używano.
# Beside the limit. $count: about how many orbits of the live view's length fit in it.
cache-limit-estimate =
    { $count ->
        [one] ≈ jedna orbita przy limicie widoku na żywo
        [few] ≈ { $count } orbity przy limicie widoku na żywo
        [many] ≈ { $count } orbit przy limicie widoku na żywo
       *[other] ≈ { $count } orbity przy limicie widoku na żywo
    }
cache-kept = Przechowywane (najcenniejsze najpierw)
# Column headers of the list of kept orbits.
cache-col-iterations = iteracje
cache-col-precision = precyzja
cache-col-size = rozmiar
cache-col-last-used = ostatnie użycie
# The precision column. $bits: the precision in bits.
cache-row-bits =
    { $bits ->
        [one] { $bits } bit
        [few] { $bits } bity
        [many] { $bits } bitów
       *[other] { $bits } bitu
    }
# Under the list. $count: the orbits not listed.
cache-rows-more = … i jeszcze { $count }
# When an orbit was last used (the "last used" column).
cache-ago-now = przed chwilą
# $count: minutes.
cache-ago-minutes = { $count } min temu
# $count: hours.
cache-ago-hours = { $count } godz. temu
# $count: days.
cache-ago-days =
    { $count ->
        [one] { $count } dzień temu
        [few] { $count } dni temu
        [many] { $count } dni temu
       *[other] { $count } dnia temu
    }
# Asked before clearing. $count: the orbits; $size: their total size, such as "1.2 GB".
cache-confirm =
    { $count ->
        [one] Usunąć jedną orbitę ({ $size })? Jej odbudowa kosztuje czas, nie dane.
        [few] Usunąć { $count } orbity ({ $size })? Ich odbudowa kosztuje czas, nie dane.
        [many] Usunąć { $count } orbit ({ $size })? Ich odbudowa kosztuje czas, nie dane.
       *[other] Usunąć { $count } orbity ({ $size })? Ich odbudowa kosztuje czas, nie dane.
    }
cache-delete = Usuń
cache-cancel = Anuluj
cache-clear = Wyczyść pamięć podręczną…
cache-clear-hover = Usuwa wszystkie przechowywane orbity. Najpierw pyta.
cache-close = Zamknij
# $count: the orbits removed.
cache-cleared =
    { $count ->
        [one] Wyczyszczono pamięć podręczną referencji — usunięto jedną orbitę.
        [few] Wyczyszczono pamięć podręczną referencji — usunięto { $count } orbity.
        [many] Wyczyszczono pamięć podręczną referencji — usunięto { $count } orbit.
       *[other] Wyczyszczono pamięć podręczną referencji — usunięto { $count } orbity.
    }
# $why: the system's error message.
cache-clear-failed = Nie udało się wyczyścić pamięci podręcznej referencji: { $why }

## Help ▸ Diagnostics…

diag-title = Diagnostyka
diag-intro = Uruchom test, aby sprawdzić, czy Fractadyne działa poprawnie na tym sprzęcie. Wyniki są zapisywane na tym komputerze i można je dołączyć do zgłoszenia problemu — nic nie jest nigdzie wysyłane samoczynnie.
# $status: which arbitrary-precision library is in use, as the program reports it (in English).
diag-arithmetic = Arytmetyka głębokiego zoomu: { $status }
# The three tests' buttons, each followed by a sentence saying what it does.
diag-run-selftest = Uruchom autotest
diag-run-uitest = Uruchom test interfejsu
diag-run-gputest = Uruchom test arytmetyki GPU
diag-blurb-selftest = Sprawdza matematykę i renderowanie na tym GPU względem znanych poprawnych wyników. Trwa około 15 sekund.
diag-blurb-uitest = Przechodzi przez interfejs i widok na żywo, robiąc zrzuty ekranu na kilku głębokościach zoomu. Trwa minutę lub dwie i w trakcie otwiera okna.
diag-blurb-gputest = Sprawdza arytmetykę double-float shadera względem znanych poprawnych wartości na każdym backendzie graficznym dostępnym na tym komputerze. Trwa kilka sekund i nie otwiera okna. Jeśli zgłosi błąd, to rzeczywiste odkrycie, które chcielibyśmy zobaczyć — prosimy dołączyć je do zgłoszenia.
# While a test runs. $count: the checks finished so far.
diag-checks-done =
    { $count ->
        [one] ukończono jedno sprawdzenie
        [few] ukończono { $count } sprawdzenia
        [many] ukończono { $count } sprawdzeń
       *[other] ukończono { $count } sprawdzenia
    }
# A finished test's result. $test: the test's name (Self-test, UI test, GPU arithmetic check).
diag-verdict-captured = { $test } — wynik zapisany
diag-verdict-passed = { $test } — zaliczony
diag-verdict-problems = { $test } — zgłoszono problemy
# Under the GPU arithmetic check's result.
diag-informational-note = Ten test nie ma wyniku pozytywnego ani negatywnego: zapisuje, jak liczy dany GPU, a zgłoszony błąd to rzeczywiste odkrycie, a nie usterka komputera. Tak czy inaczej, dołączenie go do zgłoszenia to najbardziej przydatne, co można z nim zrobić.
# Under a test's result when it reported problems.
diag-problems-note = Na sprzęcie innym niż referencyjny GPU niektóre różnice są oczekiwane — zwłaszcza w porównaniach obrazów. Dołączenie tego wyniku do zgłoszenia problemu to najbardziej przydatne, co można z nim zrobić.
diag-open-results = Otwórz wyniki
diag-attach = Dołącz do zgłoszenia problemu…
diag-attach-hover = Otwiera okno zgłoszenia z dołączonym tym wynikiem, więc zgłoszenie zawiera test zweryfikowany maszynowo, a nie tylko opis
diag-console = Wypisuj diagnostykę w konsoli
diag-console-hover = Od teraz pokazuje wiersze [fd-…] na stderr. Domyślnie wyłączone, gdy Fractadyne jest uruchamiany bez argumentów. Nie zmienia to niczego w tym, co jest ZAPISYWANE — plik dziennika zawsze dostaje każdy wiersz — więc zostaw to wyłączone, chyba że obserwujesz terminal. Obowiązuje tylko w tej sesji; użyj opcji --console lub FRACTADYNE_CONSOLE=1, aby objąć też uruchamianie.
diag-close = Zamknij
# Errors starting a test. $why: the system's error message; $dir: a folder.
diag-error-no-exe = Nie można znaleźć pliku wykonywalnego: { $why }
diag-error-create-dir = Nie można utworzyć { $dir }: { $why }
diag-error-start = Nie udało się uruchomić testu: { $why }

## Navigate ▸ Misiurewicz explorer…

misi-title = Eksplorator Misiurewicza
misi-mandelbrot-only = Przeglądanie punktów Misiurewicza działa tylko dla zbioru Mandelbrota — zmień fraktal, aby z niego skorzystać.
# While the gallery is being built. $seconds: the time so far, such as "3".
misi-solving = Wyznaczanie rodzin małych typów ({ $seconds }s)…
# Above the gallery. $total: all points; $curated: the hand-picked ones; $found: the ones the sweep found.
misi-summary = Punkty: { $total } — wybrane ręcznie: { $curated }, znalezione przez przeszukiwanie małych typów metodą Newtona z punktami startowymi: { $found } (górna półpłaszczyzna; zgrubna siatka znajduje większość basenów, ale nie ma dowodu, że wszystkie).
# The name of a point the sweep found. $kind: its type, such as "(3,1)"; $point: its coordinate, such as "-0.1011+0.9563i".
misi-entry-name = { $kind } w { $point }
# Under each thumbnail. $kind: the point's type, such as "(3,1)"; $octaves: the zoom period in octaves, such as "3.17"; $twist: the twist per repeat in degrees, such as "+12.5". "oct" is short for octaves.
misi-entry-readout = { $kind } · { $octaves } okt./powt. · { $twist }°
# Before the depth field.
misi-zoom-to = Zoom do
misi-cancel = Anuluj
misi-solve-jump = Wyznacz i skocz
misi-solve-jump-hover = Wyznacza ten dokładny punkt metodą Newtona z precyzją, jakiej wymaga żądana głębokość (startując od jego zapisanej współrzędnej), a potem skacze prosto tam. Głębokie żądania trwają dłużej — obliczenie jest w dowolnej precyzji.
misi-select-hint = Wybierz punkt, podaj głębokość i skocz — wiersz mnożnika pod każdą miniaturą pokazuje, jak będzie wyglądać zanurzanie w tym miejscu: po ilu oktawach widok się powtarza i jak bardzo skręca przy każdym powtórzeniu.
misi-cancelled = Wyznaczanie anulowane.
# $text: what was typed in the depth field.
misi-bad-zoom = Nie można odczytać „{ $text }” jako powiększenia (wpisz 1e50, 3.2e120, …).
misi-no-kp = Ten wpis nie ma wykrytej pary (k,p), z którą można by go wyznaczyć.
misi-parse-failed = Nie udało się przetworzyć zapisanej współrzędnej.
# A notice after the jump. $name: the point's name; $zoom: the magnification, such as "1e50"; $seconds: how long the solve took, such as "1.2".
misi-zoomed = Przybliżono do: { $name } — { $zoom }× ({ $seconds }s)
# Added after the notice above when the iteration count is too low. $have: the fixed iteration count; $typical: what the depth typically needs.
misi-iter-note = Iteracje są ustalone na { $have }; ta głębokość zwykle wymaga ~{ $typical } — zwiększ je (lub włącz automatyczne iteracje), jeśli widok renderuje się płasko.
# $name: the point's name; $why: why the solve failed (a technical word).
misi-solve-missed = Głębokie wyznaczanie nie trafiło w { $name }: { $why }. Zapisany punkt jest nadal dokładny na własnej głębokości — spróbuj płytszego celu.

## Life (Game of Life): the side panel

# Shown when the graphics card cannot run the Life view.
life-no-compute = Ta karta graficzna nie może uruchomić Life (brak shaderów obliczeniowych).
# Row label: the pattern list (pattern names stay in English).
life-pattern = Wzorzec
life-open = Otwórz…
life-open-hover = Otwórz plik wzorca RLE, .cells lub Life 1.05/1.06.
life-pattern-text = Tekst wzorca…
life-pattern-text-hover = Wklej wzorzec (RLE w postaci z LifeWiki lub zwykły tekst).
life-save = Zapisz…
life-save-hover = Zapisz wszechświat w obecnym stanie jako RLE.
# Row label: the rule list and the typed rule (B3/S23 notation).
life-rule = Reguła
# Shown in the rule list when the rule in use is none of the library's.
life-rule-custom = własna
life-apply = Zastosuj
# Playback buttons (a pause or play symbol is shown before them).
life-pause = Wstrzymaj
life-play = Uruchom
life-play-hover = Uruchom wszechświat (Enter).
life-step = Krok
life-step-hover = Przejdź o jedno pokolenie (N).
# Tooltip of the button beside Step, which reads "+16", "+1,024"…: that many generations.
life-stride-hover = Przejdź o tyle pokoleń (Shift+N).
life-reset = Resetuj
life-reset-hover = Powrót do wzorca w postaci otwartej (pokolenie 0).
life-speed = Szybkość
# After the speed slider's number: generations a second. Short.
life-speed-unit = pok./s
# Row label: how many generations the "+n" button advances.
life-stride = Skok
# The panel's numbers.
life-generation = Pokolenie
life-population = Populacja
life-tiles = Kafelki
# $count: the tiles stored; $capacity: how many the GPU pool holds.
life-tiles-of = { $count } z { $capacity }
life-tiles-hover = Przechowywane kafelki 64×64 komórek (po 4 KiB) i pojemność puli GPU.
# Row label: what a left-drag in the view does (Pan or Draw).
life-mouse = Mysz
life-pan = Przesuwanie
life-pan-hover = Przeciągnij, aby przesunąć widok.
life-draw = Rysowanie
# The Draw tool's tooltip; the toolbar's shows it after "Draw (D):", so it starts lowercase.
life-draw-hint = kliknij komórkę, aby ją odwrócić; przeciągnij, aby malować — pociągnięcie, które zaczyna się na martwej komórce, rysuje, a takie, które zaczyna się na żywej, wymazuje. Działający wszechświat wstrzymuje się podczas rysowania i rusza dalej po puszczeniu przycisku.
life-undo = Cofnij
life-undo-hover = Powrót do wszechświata sprzed ostatniej edycji, wypełnienia, wyczyszczenia lub zmiany reguły (Ctrl+Z podczas rysowania).
life-grid = Linie siatki
life-grid-hover = Granice komórek przy powiększeniu do 8 pikseli na komórkę lub więcej.
# A random fill: "Random" labels the density field, "seed" the seed field after it.
life-random = Losowo
life-density-hover = Odsetek komórek, które wypełnienie ożywia.
life-seed = ziarno
life-fill-view = Wypełnij widok
# $cells: the largest side of the filled area, in cells (2,048).
life-fill-view-hover = Wypełnia losowo widoczny obszar (do { $cells } × { $cells } komórek wokół jego środka).
life-clear = Wyczyść
life-clear-hover = Zabija wszystkie komórki (Cofnij je przywraca).

## Life: notices in the panel, and file dialogs

# $path: the file written.
life-saved = Zapisano { $path }
# $error: why (from the system, in English).
life-save-failed = Zapis nie powiódł się: { $error }
# The universe stopped running. $error: why (in English).
life-stopped = Zatrzymano: { $error }
# A pattern file named a rule that does not read. $error: why (in English); $rule: the rule kept, in B3/S23 notation.
life-rule-not-understood = Nie zrozumiano reguły wzorca ({ $error }); zachowano { $rule }
# $path: the file; $error: why (in English).
life-open-failed = Nie udało się otworzyć { $path }: { $error }
# File-dialog filter names.
life-filter-patterns = Wzorce Life
life-filter-rle = Wzorzec zakodowany RLE

## Life: the toolbar (its buttons are icons; these are their tooltips)

life-toolbar-pause-hover = Wstrzymaj wszechświat (Enter)
life-toolbar-play-hover = Uruchom wszechświat (Enter)
# $stride: the generations a Shift+click steps.
life-toolbar-step-hover = Krok o jedno pokolenie (N); Shift+kliknięcie lub Shift+N przechodzi o { $stride }
life-toolbar-reset-hover = Powrót do wzorca w postaci otwartej (pokolenie 0)
# $hint: the Draw tool's tooltip (life-draw-hint).
life-toolbar-draw-hover = Rysowanie (D): { $hint }

## Life: the Pattern text window

life-text-title = Tekst wzorca
life-text-intro = Wklej wzorzec: RLE (w postaci z LifeWiki i Golly), zwykły tekst .cells lub Life 1.05 / 1.06.
life-text-load = Wczytaj
life-text-current = Bieżący jako RLE
life-text-current-hover = Wypełnij pole wszechświatem w ostatnio wczytanej postaci.

## L-systems: the side panel

# Row label: the system list (system names stay in English).
lsys-system = System
lsys-open = Otwórz…
lsys-open-hover = Otwórz plik .l programu Fractint lub system .lsys.
lsys-edit = Edytuj…
lsys-edit-hover = Tekst systemu: aksjomat, produkcje, kąt — edytuj i zastosuj.
lsys-save = Zapisz…
lsys-save-hover = Zapisz system jako plik tekstowy .lsys.
# Tooltip of the "SVG…" button.
lsys-svg-hover = Eksportuj widok jako rysunek SVG: jego linie i kształty jako wektory, w jego kolorach.
lsys-follow = Rząd podąża za zoomem
lsys-follow-hover = Rząd rośnie podczas przybliżania, więc szczegółów nigdy nie brakuje.
lsys-follow-fixed-hover = Ten system nie rośnie o stały współczynnik z każdym rzędem, więc jest rysowany w stałym rzędzie.
lsys-order = Rząd
lsys-angle = Kąt
lsys-sweep-hover = Przemiataj kąt tam i z powrotem: krzywa zmienia kształt podczas obrotu
# A small button that puts back the system's own angle after the slider moved it.
lsys-system-angle = Kąt systemu
lsys-seed = Ziarno
lsys-seed-hover = Od czego zależą losowe wybory: to samo ziarno, ta sama roślina.
lsys-seed-new = Nowe
lsys-seed-new-hover = Inna roślina: nowe ziarno.
lsys-line-width = Grubość linii
# Row label of a percentage slider: how much of the curve is drawn.
lsys-draw-on = Postęp rysowania
lsys-draw-on-hover = Jaka część krzywej jest narysowana, w kolejności, w jakiej rysuje ją żółw
lsys-draw-on-play-hover = Krzywa rysująca się sama
lsys-colour-by = Koloruj według
# The colourings, in the "Colour by" list.
lsys-colour-position = pozycji na krzywej
lsys-colour-depth = głębokości gałęzi
lsys-colour-heading = kierunku
# C, < and > are the system's commands that set the colour index: keep them.
lsys-colour-index = indeksu koloru (C, <, >)
lsys-colour-plain = jeden kolor
# The panel's numbers.
lsys-growth = Wzrost
lsys-growth-built = budowany jako słowo, w swoim rzędzie
lsys-growth-built-hover = Parametryczny lub kontekstowy: przepisanie modułu zależy od jego liczb lub sąsiadów, więc słowo jest budowane po jednej generacji naraz (do dwóch milionów modułów) i rysowane w podwójnej precyzji — rząd nie podąża za zoomem.
# How much the picture grows from one order to the next. $factor: the factor (1.4142).
lsys-growth-factor = ×{ $factor } na rząd
lsys-growth-factor-period-2 = ×{ $factor } na rząd (rzędy co 2)
lsys-growth-none = brak (stały rząd)
lsys-drawn = Narysowano
# $drawn: what was drawn (lsys-drawn-segments, lsys-drawn-shapes or lsys-drawn-segments-shapes); $order: the order.
lsys-drawn-at-order = { $drawn } w rzędzie { $order }
# The same, when the walk stopped at its segment budget.
lsys-drawn-at-order-budget = { $drawn } w rzędzie { $order } (budżet)
# $segments: the line segments drawn.
lsys-drawn-segments =
    { $segments ->
        [one] { $segments } odcinek
        [few] { $segments } odcinki
        [many] { $segments } odcinków
       *[other] { $segments } odcinka
    }
# $shapes: the filled shapes drawn.
lsys-drawn-shapes =
    { $shapes ->
        [one] { $shapes } wypełniony kształt
        [few] { $shapes } wypełnione kształty
        [many] { $shapes } wypełnionych kształtów
       *[other] { $shapes } wypełnionego kształtu
    }
# Both counts. $segments: line segments; $shapes: filled shapes.
lsys-drawn-segments-shapes =
    { $segments ->
        [one] { $segments } odcinek
        [few] { $segments } odcinki
        [many] { $segments } odcinków
       *[other] { $segments } odcinka
    }, { $shapes ->
        [one] { $shapes } wypełniony kształt
        [few] { $shapes } wypełnione kształty
        [many] { $shapes } wypełnionych kształtów
       *[other] { $shapes } wypełnionego kształtu
    }
# $ms: the walk's time in milliseconds (12.3).
lsys-drawn-hover = przejście trwało { $ms } ms; poddrzewa poza widokiem są pomijane, a te mniejsze niż piksel rysowane jako jeden odcinek
# A word too long to build at the order asked for. $wanted: the order asked for; $built: the order drawn.
lsys-drawn-short = Słowo rzędu { $wanted } ma ponad dwa miliony modułów: narysowano w rzędzie { $built }.
# $order: the deepest order the app draws.
lsys-deepest-order = W najgłębszym rysowanym rzędzie ({ $order }): dalszy zoom nie dodaje szczegółów.

## L-systems: notices in the panel and the editor, and file dialogs

# $path: the file; $error: why (in English).
lsys-open-failed = Nie udało się otworzyć { $path }: { $error }
# Why a file was not opened (shown after "Could not open …:").
lsys-file-too-large = większy niż 4 MiB
# Why a Fractint .l file was not opened: it holds no entries. `Name { … }` is the file's syntax: keep it.
lsys-no-entries = brak wpisów (plik .l zawiera wpisy `Name {"{"} … {"}"}`)
# $path: the file written.
lsys-saved = Zapisano { $path }
# $error: why (from the system, in English).
lsys-save-failed = Zapis nie powiódł się: { $error }
# After File ▸ Export SVG. $path: the file written.
lsys-svg-exported = Wyeksportowano { $path }
# $error: why (from the system, in English).
lsys-svg-failed = Eksport nie powiódł się: { $error }
# File-dialog filter names: opening (any system file) and saving (one system).
lsys-filter-systems = L-systemy
lsys-filter-system = L-system

## L-systems: the toolbar

lsys-toolbar-lower-hover = Obniż rząd (ustala go)
lsys-toolbar-raise-hover = Podnieś rząd (ustala go)
# Between the − and + buttons, in a fixed-width font. $order: the order, padded to 3 characters so the label keeps its width.
lsys-toolbar-order = rząd { $order }
# A toggle: the order follows the zoom. Short.
lsys-toolbar-auto = auto
lsys-toolbar-auto-hover = Rząd podąża za zoomem
lsys-toolbar-edit-hover = Edytuj tekst systemu

## L-systems: the editor window, and a .l file's entries

lsys-editor-title = L-system
# The editor's help. The quoted examples ('X = word', 'seed n', 'define R 1.456'…), the commands and "The Algorithmic Beauty of Plants" (a book's title) are the system's syntax and names: keep them as they are.
lsys-editor-help = Jeden klucz w wierszu: angle (stopnie lub /n dla podziału okręgu), axiom, produkcje 'X = word' oraz opcjonalnie heading, draw / move / variables, colour, order. Symbol z alternatywami wybieranymi losowo ma po jednym wierszu na każdą z nich, z jej wagą: 'X (0.3) = word'; 'seed n' wybiera roślinę. W słowie {"{"} {"}"} wypełnia ścieżkę żółwia, a . dodaje wierzchołek. Produkcje parametryczne i kontekstowe (The Algorithmic Beauty of Plants): 'A(s) : s > 1 = F(s)[+A(s/2)]', 'b < a > c = b', z 'define R 1.456' i 'ignore +-F'; F(l) robi krok o długości l, +(a) obraca o a stopni. Wpis .l programu Fractint można wkleić bez zmian.
lsys-editor-apply = Zastosuj
lsys-editor-revert = Przywróć
lsys-editor-revert-hover = Tekst wyświetlanego systemu.
# The window listing a .l file's entries to pick from. $file: the file's name.
lsys-entries-title = { $file } — wpisy
# Tooltip of an entry that does not parse (its error is shown beside it, in English).
lsys-entry-unreadable-hover = Tego wpisu nie da się odczytać.

## Export: opening a view or location (File ▸ Open view or location…)

# File-dialog filter names.
export-filter-view = Widok lub lokalizacja Fractadyne
export-filter-image = Obraz (PNG / EXR)
export-filter-location = Lokalizacja (.fdn / .kfr)
# $zoom: the magnification (1.2e148), followed by the magnification sign ×.
export-loaded-location = Wczytano lokalizację @ { $zoom }×
# $zoom: the magnification; $note: what was noteworthy in the file (in English).
export-loaded-location-note = Wczytano @ { $zoom }× — { $note }
# $path: the file.
export-not-location = { $path } nie jest lokalizacją Fractadyne.
export-file-too-large = Plik jest za duży (to nie lokalizacja .fdn?).
# $path: the file; $error: why (from the system, in English).
export-read-failed = Nie udało się odczytać { $path }: { $error }
# A Kalles Fraktaler .kfr file did not import. $error: why (in English).
export-import-failed = Import nie powiódł się: { $error }
# $path: an image whose embedded text is not a view.
export-not-view = { $path } ma metadane Fractadyne, ale nie zawiera widoku.
# $path: the image the view was read from.
export-loaded-view = Wczytano widok z { $path }
# $path: the image; $note: what was noteworthy in it (in English).
export-loaded-view-note = Wczytano widok z { $path } — { $note }
export-no-metadata = Ten plik nie ma osadzonych metadanych widoku Fractadyne.

## Export: notices after an export or a snapshot
## "File → Export image…" in them names the menu items (menu-file, menu-file-export).

# The export window's status while the image renders.
export-rendering = Renderowanie…
export-canceled = Eksport anulowany.
# $error: why (in English).
export-failed = Eksport nie powiódł się: { $error }
# $size: the image's size in pixels (1920×1080); $path: the file written.
export-saved = Zapisano { $size } → { $path }
# An L-system export whose walk stopped at its budget. $size: pixels; $segments: the budget (4,000,000); $path: the file.
export-saved-incomplete = Zapisano { $size } (niekompletny: przekroczono budżet { $segments } odcinków — obniż rząd) → { $path }
# A two-panel export written as two files. $path: the first of them.
export-saved-two = Zapisano 2 pliki → { $path }
# $size: pixels; $path: the file written.
export-saved-corrected = Zapisano { $size } (z korekcją glitchy) → { $path }
# $path: the first of the two files.
export-saved-two-corrected = Zapisano 2 pliki (z korekcją glitchy) → { $path }
# Added after a finished export's notice. $duration: how long it took (8.3s, 1m 04.0s).
export-elapsed = (czas: { $duration })
export-preparing-deep = Przygotowywanie głębokiego eksportu — budowa referencji (to może chwilę potrwać)…
# $size: pixels (5120×4035); $ss: the supersampling factor; $iterations: the iteration count.
export-large = Duży eksport ({ $size }, supersampling { $ss }×, { $iterations } iteracji) — renderowanie w tle; przy eksportach tej wielkości korekcja glitchy jest pomijana…
export-life-unavailable = Eksport obrazu widoku Life nie jest jeszcze dostępny — użyj zrzutu ekranu lub Plik ▸ Zapisz wzorzec Life.
export-already-running = Eksport już trwa — Plik → Eksportuj obraz… pokazuje jego postęp.
# $size: the image's size in pixels.
export-snapshot-background = Migawka: renderowanie { $size } w tle — Plik → Eksportuj obraz… pokazuje postęp i pozwala anulować.
export-no-gpu = GPU niedostępny
# A screen-capture snapshot. $size: pixels; $path: the file written.
export-snapshot-saved = Zapisano migawkę ekranu { $size } → { $path }
# $error: why (in English).
export-snapshot-failed = Migawka nie powiodła się: { $error }

## The colour picker (the popup of a colour swatch)

# The third row's label: the colour as hexadecimal bytes. Short (about 6 characters).
picker-row-hex = Hex
picker-hex-hover = Kliknij, aby wpisać wartość szesnastkową
picker-cancel = Anuluj
picker-cancel-hover = Przywróć kolor, z którym otwarto ten selektor
picker-ok = OK
picker-ok-hover = Zachowaj ten kolor
# Starts the screen eyedropper (an icon is shown before it).
picker-pick = Pobierz
picker-pick-hover = Pobierz kolor z dowolnego miejsca na ekranie
# Tooltip of the disabled Pick button (in the picker and the gradient editor) on Linux and macOS.
picker-eyedropper-unsupported = Pobieranie koloru z innego miejsca na ekranie wymaga systemowego API przechwytywania ekranu. Ma je tylko kompilacja dla Windows; w systemie Linux wymagałoby to zależności od X11, a Wayland wymaga portalu z własnym monitem o zgodę.

## The fractal families: what each one is (the info panel's "About" section and the pickers' hover text).
## The formulas themselves (z -> z^2 + c) are written in code and stay as they are.

fractal-mandelbrot-about = Kanoniczny fraktal czasu ucieczki: zbiór tych c, dla których orbita 0 pozostaje ograniczona. Jego brzeg jest nieskończenie złożony.
fractal-multibrot3-about = Zbiór typu Multibrot — konstrukcja Mandelbrota w wyższej potędze. Potęga d daje symetrię obrotową rzędu (d-1).
fractal-multibrot4-about = Multibrot w potędze 4: symetria rzędu 3, szerokie bąble.
fractal-multibrot5-about = Multibrot w potędze 5: symetria rzędu 4.
fractal-tricorn-about = Tricorn (Mandelbar): w każdym kroku sprzęga z. To antyholomorficzne odwzorowanie daje kształt o trzech narożnikach.
fractal-burning-ship-about = Przed podniesieniem do kwadratu bierze się wartości bezwzględne części z; wynik przypomina płonący statek.
fractal-celtic-about = Krewny Burning Ship, który bierze wartość bezwzględną tylko części rzeczywistej z^2, tworząc motywy celtyckich węzłów / serc.
fractal-buffalo-about = Wariant z abs, biorący wartości bezwzględne obu składowych z^2.
fractal-phoenix-about = Phoenix używa także poprzedniej iteracji, co daje włókna przypominające płomienie. Wypróbuj jego postać Julii w trybie Julii.
fractal-newton-about = Iteracja Newtona szukająca pierwiastków z^3 = 1, kolorowana według tego, jak szybko każdy punkt zbiega. Fraktal zbieżności (a nie ucieczki).
fractal-multibrot6-about = Multibrot w potędze 6: symetria rzędu 5.
fractal-multibrot7-about = Multibrot w potędze 7: symetria rzędu 6, bąble cienieją ku okręgowi.
fractal-multibrot8-about = Multibrot w potędze 8: symetria rzędu 7.
fractal-burning-ship3-about = Burning Ship w potędze 3: bierze się wartości bezwzględne, a potem podnosi do sześcianu.
fractal-burning-ship4-about = Burning Ship w potędze 4.
fractal-burning-ship5-about = Burning Ship w potędze 5.
fractal-tricorn3-about = Tricorn (Mandelbar) w potędze 3: symetria rzędu 4.
fractal-tricorn4-about = Tricorn w potędze 4: symetria rzędu 5.
fractal-tricorn5-about = Tricorn w potędze 5: symetria rzędu 6.
fractal-celtic3-about = Zagięcie Celtic w potędze 3: wartość bezwzględna części rzeczywistej z^3.
fractal-celtic4-about = Zagięcie Celtic w potędze 4.
fractal-celtic5-about = Zagięcie Celtic w potędze 5.
fractal-buffalo3-about = Buffalo w potędze 3: wartości bezwzględne obu części z^3.
fractal-buffalo4-about = Buffalo w potędze 4.
fractal-buffalo5-about = Buffalo w potędze 5.
fractal-custom-about = Wzór zapisany w wyrażeniach w stylu Fractinta (Fraktal ▸ Własny wzór…). Większość obsługuje głęboki zoom przez perturbację, tak jak wbudowane rodziny; okno wzoru informuje, czy dotyczy to danego wzoru.
fractal-life-about = Gra w życie Conwaya i jej krewni na nieograniczonej płaszczyźnie: reguły Generations, reguły nietotalistyczne (Hensel), pliki wzorców. Uruchamiaj, krokuj, rysuj.
fractal-lsystem-about = Systemy Lindenmayera: krzywe, krzywe wypełniające przestrzeń, wyspy i rośliny rysowane przez żółwia na podstawie gramatyki przepisywania. Rząd podąża za zoomem, więc szczegółów nigdy nie brakuje. Pliki .l programu Fractint otwierają się bezpośrednio.

# Shown where the other families show their formula (z -> z^2 + c).
fractal-custom-formula = z -> wpisany wzór
# Shown where the other families show their formula. "B3/S23" is the rule's notation: keep it.
fractal-life-formula = B3/S23: rodzi się przy 3 żywych sąsiadach, przeżywa przy 2 lub 3
# Shown where the other families show their formula. "F = F+F--F+F" is the grammar's notation: keep it.
fractal-lsystem-formula = F = F+F--F+F: każdy symbol jest przepisywany, a potem rysowany przez żółwia

# The last paragraph of the picker's hover text for a family that renders only on the direct path.
fractal-no-deep-zoom = Tylko renderowanie bezpośrednie — ta rodzina nie ma głębokiego zoomu.

## Names of settings, shown in the Settings menu and the welcome window.

theme-dark = Ciemny
theme-light = Jasny
# The update track that follows only stable releases.
update-track-stable = Stabilne
# The update track that also offers pre-releases (betas).
update-track-beta = Beta (wydania wstępne)

## The status bar (its hover texts; the readouts follow in their own section)

status-period-hover = Okres minibrota, do którego nastąpił skok (klawisz M lub Przejdź do lokalizacji ▸ Przejdź do struktury). Pozostaje, dopóki widok jest na strukturze, i znika po odejściu od niej.

## The playback transport: the player over the view while a tour plays.

# $close: the close button's icon, as drawn on that button.
play-finished-hover = Zakończono — przewiń wstecz lub { $close }, aby zamknąć
# $kind: "benchmark" or "script" (the variants hold the wording); $name: the tour's name.
play-playing-hover =
    { $kind ->
        [benchmark] Odtwarzanie testu wydajności „{ $name }”
       *[other] Odtwarzanie skryptu „{ $name }”
    }
play-restart = Uruchom od początku
play-back = 10 sekund wstecz
play-again = Odtwórz ponownie od początku
play-resume = Wznów
play-pause = Wstrzymaj
play-stop = Zatrzymaj i przewiń na początek
play-forward = 10 sekund do przodu
play-speed-hover = Szybkość odtwarzania (kliknij, aby przełączać 0,5× / 1× / 2× / 4×)
play-render = Renderuj ten skrypt do sekwencji klatek…
play-render-none = Wbudowany test wydajności nie ma pliku skryptu do renderowania
play-open = Odtwórz inny skrypt…
play-loop = Powtarzaj przelot po dojściu do końca
play-close = Zamknij odtwarzacz i przywróć własne ustawienia widoku
# A notice beside the scrub bar while the renderer holds the tour's clock. Keep it short.
play-waiting = czekanie na szczegóły
# "[playback] pace" is a setting in the script file: keep it as written.
play-waiting-hover = Zegar przelotu jest wstrzymany, gdy renderer dopracowuje ten widok (budowa referencji / rosnący budżet iteracji). Odtwarzanie wznowi się samo; zob. [playback] pace w skrypcie.
play-scrub = Przeciągnij, aby przewijać przelot
# $behind: how far the script clock is behind real time, as minutes:seconds (1:05).
play-scrub-behind =
    Przeciągnij, aby przewijać przelot
    Zegar skryptu opóźnia się o { $behind } względem czasu rzeczywistego (regulator tempa spowolnił przelot, aby renderer nadążał). Górny znacznik pokazuje, gdzie byłoby odtwarzanie według czasu rzeczywistego.

## The autopilot's notices (toasts). Where a notice has two lines, the first is the verdict and the second what to do about it.

autopilot-on-detail = Autopilot włączony — zanurzanie w stronę szczegółów (każda czynność go zatrzymuje)
autopilot-off = Autopilot wyłączony
autopilot-misi-needs-mandelbrot =
    Autopilot: cel Misiurewicza wymaga zbioru Mandelbrota
    Zamiast tego zanurzanie w stronę szczegółów
autopilot-on-misi =
    Autopilot włączony — szukanie najbliższego punktu Misiurewicza…
    Każda czynność go zatrzymuje
# $seconds: how long the search has run, in whole seconds.
autopilot-misi-finding =
    Autopilot: szukanie najbliższego punktu Misiurewicza…
    { $seconds } s · każda czynność go zatrzymuje
autopilot-misi-not-found =
    Autopilot: nie znaleziono punktu Misiurewicza w pobliżu tego widoku
    Wyszukiwanie może nie sięgać tej głębokości — spróbuj celu Szczegóły
# ($preperiod,$period) names the point, as mathematicians write it: keep the notation. $widths: a power of two (2^$widths screen widths).
autopilot-misi-off-screen =
    Autopilot: najbliższy punkt Misiurewicza ({ $preperiod },{ $period }) jest poza ekranem
    ~2^{ $widths } szerokości ekranu dalej — podejdź bliżej środka spirali
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-not-solved =
    Autopilot: nie udało się wyznaczyć punktu Misiurewicza ({ $preperiod },{ $period }) w pobliżu tego widoku
    Spróbuj celu Szczegóły
autopilot-misi-refused = Autopilot: odrzucono wyznaczanie punktu Misiurewicza (zła para)
autopilot-misi-stopped = Autopilot: solver Misiurewicza nieoczekiwanie się zatrzymał
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-diving =
    Autopilot: zanurzanie w punkt Misiurewicza ({ $preperiod },{ $period })
    Każda czynność go zatrzymuje
# ($preperiod,$period) names the point: keep the notation. $octaves: how often the structure repeats (2.37); $twist: its turn in degrees, signed (+12.5).
autopilot-misi-diving-repeat =
    Autopilot: zanurzanie w punkt Misiurewicza ({ $preperiod },{ $period })
    Struktura powtarza się co { $octaves } oktawy, skręt { $twist }° · każda czynność go zatrzymuje
# $limit: the magnification, as written (1e300).
autopilot-dive-limit = Autopilot: osiągnięto limit zanurzania (~{ $limit }×)
# $have: the fixed iteration count; $typical: what this depth typically needs.
autopilot-dead-end-short =
    Autopilot zatrzymany: nie ma już w co celować — iteracje są ustalone na { $have }, a ta głębokość zwykle wymaga ~{ $typical }
    Włącz automatyczne iteracje lub zwiększ ich liczbę
# $limit: the fixed iteration count.
autopilot-dead-end-cap =
    Autopilot zatrzymany: nie ma już w co celować — wszystko w widoku dochodzi do stałego limitu iteracji { $limit }
    Włącz automatyczne iteracje lub zwiększ ich liczbę
autopilot-dead-end = Autopilot: brak szczegółów na drodze (zatrzymany)

## Tours (camera scripts)

# The name of the file type in the file dialog's filter list.
script-file-filter = Skrypt Fractadyne (TOML)
# $name: the script's name.
script-finished = Skrypt zakończony — „{ $name }”
# The title of the notice that follows (the file's path and the error, in English).
script-load-failed = Nie udało się wczytać skryptu

## Installing a language file

# The name of the file type in the file dialog's filter list.
language-file-filter = Plik języka Fractadyne

## The notice when the graphics driver's shader cache stops keeping shaders. The text is shown in a fixed-width font with its line breaks as written: break the lines where they read well.

driver-cache-title = Sterownik graficzny nie przechowuje shaderów
# $count: how many shaders compiled again; $pipeline: the slowest one's name (a technical id: keep it); $seconds: its time in seconds (7.5).
driver-cache-body =
    Sterownik graficzny ponownie skompilował w tej sesji shadery Fractadyne
    ({ $count }), choć każdy z nich skompilował już wcześniej (najwolniejszy:
    { $pipeline }, { $seconds } s). Jego pamięć podręczna shaderów nie
    zachowuje skompilowanych shaderów, najczęściej dlatego, że jest pełna.
    Dopóki to się nie zmieni, eksporty, głębokie widoki i uruchamianie
    Fractadyne mogą za każdym razem tracić sekundy na kompilację.
# $folder: the cache folder's path; $size: its size in MB (1024). "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia-folder =
    Rozwiązanie: zamknij Fractadyne, a następnie usuń zawartość
      { $folder }   ({ $size } MB)
    Sterownik odbuduje pamięć podręczną. Aby znów się nie zapełniła, zwiększ
    Panel sterowania NVIDIA > Zarządzaj ustawieniami 3D > Rozmiar pamięci
    podręcznej modułu cieniującego.
# "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia =
    Rozwiązanie: zwiększ Panel sterowania NVIDIA > Zarządzaj ustawieniami 3D >
    Rozmiar pamięci podręcznej modułu cieniującego albo wyczyść pamięć
    podręczną shaderów sterownika.
# "AMD Software > Graphics > Advanced > Reset Shader Cache" names AMD's own settings: use the names AMD's software shows in this language.
driver-cache-fix-amd = Rozwiązanie: AMD Software > Grafika > Zaawansowane > Resetuj pamięć podręczną shaderów.
driver-cache-fix-other = Rozwiązanie: wyczyść pamięć podręczną shaderów sterownika graficznego.

## Why a language file could not be installed: each ends the sentence "{ $file } is not a usable language file: …".

language-install-not-utf8 = to nie jest tekst UTF-8
language-install-too-large = jest za duży
# "fractadyne-<language>.ftl" is the file name to use: keep it as written.
language-install-no-tag = nie wskazuje, jakiego języka dotyczy (nazwij go fractadyne-<language>.ftl)
language-install-english = angielski jest wbudowany
language-install-empty = nie tłumaczy żadnego komunikatu tej wersji
language-install-no-folder = nie ma folderu ustawień

## The notice after a session that did not end cleanly, when its iteration setting was set aside.

# $base: the iteration count automatic iterations start from.
quarantine-setting-auto = automatyczne iteracje od bazy { $base }
# $count: the fixed iteration count.
quarantine-setting-fixed =
    { $count ->
        [one] { $count } iteracja
        [few] { $count } iteracje
        [many] { $count } iteracji
       *[other] { $count } iteracji
    }
# $end: "device" when the session lost its graphics device, "unclean" otherwise (the variants hold the wording). $depth: the magnification, as written (1.2e148). $setting and $restored: an iteration setting (quarantine-setting-auto or -fixed).
quarantine-notice =
    { $end ->
        [device] Poprzednia sesja utraciła urządzenie graficzne przy { $depth }×, gdy ustawienie wynosiło: { $setting }. Ta sesja zaczęła się zamiast tego od ustawienia: { $restored }, więc nie może od razu wrócić do tego samego obciążenia.
       *[other] Poprzednia sesja nie zakończyła się poprawnie przy { $depth }×, gdy ustawienie wynosiło: { $setting }. Ta sesja zaczęła się zamiast tego od ustawienia: { $restored }, więc nie może od razu wrócić do tego samego obciążenia.
    }

## The second graphics card's status line, under its setting in the control panel.

gpu2-state-off = Wyłączona: karta graficzna tego okna renderuje wszystko.
gpu2-state-starting = Uruchamianie…
# $name: the card's name (and its graphics API).
gpu2-state-running = W użyciu: { $name }
# $why: the error, as the graphics driver reports it.
gpu2-state-failed = Nie udało się uruchomić: { $why }
# $name: the card's name.
gpu2-state-lost = Zatrzymano: utracono { $name }. Od teraz jedna karta; wybierz ją ponownie, aby spróbować jeszcze raz.
# $state: one of the lines above. "--worker-gpu" is a command-line option: keep it.
gpu2-state-pinned = { $state } (ustawione przez --worker-gpu)

## The custom-formula dialog: how deep the formula renders (one line under the formula).

formula-depth-deep = Głęboki zoom przez perturbację, z rozszerzonym zakresem powyżej 1e28x jak we wbudowanych fraktalach (każdy krok iterowany: własne wzory nie mają jeszcze aproksymacji szeregiem ani BLA).
# $why: what the formula has that keeps it off the deep-zoom path (one of the formula-depth-why messages below).
formula-depth-direct = Renderowanie bezpośrednie ({ $why } nie ma jeszcze postaci dla głębokiego zoomu): ostre, dopóki jeden krok c w pojedynczej precyzji nie obejmie piksela — około 1e4x–1e5x, a z dala od początku układu płycej.
formula-depth-why-init = sekcja inicjalizacji
formula-depth-why-bailout = własny warunek ucieczki
formula-depth-why-variable = zmienna przenoszona z kroku na krok
formula-depth-why-previous = poprzednia iteracja
formula-depth-why-comparison = porównanie lub blok if
# floor, ceil, trunc and round are functions of the formula language: keep them.
formula-depth-why-rounding = zaokrąglanie (floor, ceil, trunc, round)
# "maxit" is a word of the formula language: keep it.
formula-depth-why-maxit = maxit
formula-depth-why-power = potęga o zmiennym wykładniku
formula-depth-why-perturbed = program już poddany perturbacji

## Image and frame sizes: the presets described in words (the others are standard names: HD 720p, WUXGA, DCI 4K).

# $size: the pixel size (2048×2048).
size-preset-square = Kwadrat — { $size }
# $ratio: the aspect ratio (9:16); $size: the pixel size.
size-preset-portrait = Pionowy { $ratio } — { $size }
# $ratio: the aspect ratio (32:9); $size: the pixel size.
size-preset-super-ultrawide = Super ultrapanoramiczny { $ratio } — { $size }

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
fractal-family-custom = Własny
# Conway's Game of Life.
fractal-family-life = Life
fractal-family-lsystem = L-system
# A family at a power: "Multibrot 3", "Burning Ship 4". $family: a name above; $power: the power.
fractal-with-power = { $family } { $power }

## Names of choices: colouring methods, orbit traps, palette animation, snapshots, angle units.

color-method-smooth = Gładka iteracja
color-method-stripe = Średnia pasków
color-method-triangle = Nierówność trójkąta
color-method-trap = Pułapka orbity
color-method-distance = Estymacja odległości
color-method-decomposition = Dekompozycja
# The shape an orbit trap catches the orbit with.
trap-point = Punkt
trap-cross = Krzyż
trap-circle = Okrąg
# Palette animation: none.
palette-anim-off = Wył.
palette-anim-forward = Do przodu
palette-anim-reverse = Wstecz
palette-anim-pingpong = Ping-pong
# Palette animation: a new random gradient every so often.
palette-anim-random = Losowe gradienty
# What the Snapshot button does.
snapshot-mode-ask = Pytaj za każdym razem
snapshot-mode-screen = Zrzut ekranu
snapshot-mode-render = Pełny render
# The unit of an angle typed into Go to location's polar fields.
angle-degrees = stopnie
angle-radians = radiany
angle-turns = obroty

## Report an issue: the choices. (The report itself is sent in English.)

issue-kind-crash = Awaria / zawieszenie aplikacji
issue-kind-rendering = Nieprawidłowe renderowanie
issue-kind-performance = Problem z wydajnością
issue-kind-ui = Problem z interfejsem / użytecznością
issue-kind-feature = Prośba o funkcję
issue-kind-other = Inne
severity-low = Niska
severity-medium = Średnia
severity-high = Wysoka
severity-blocking = Blokująca
repro-always = Zawsze
repro-sometimes = Czasami
repro-once = Wystąpił raz
repro-cannot = Nie da się odtworzyć

## The benchmark's depths and the diagnostics' tests.

# $zoom: the dive's final magnification (1e32).
bench-depth-standard = Standardowa ({ $zoom }×)
# $zoom: the dive's final magnification (1e48).
bench-depth-ultra = Ultragłęboka ({ $zoom }×)
diag-test-selftest = Autotest
diag-test-uitest = Test interfejsu
diag-test-gputest = Test arytmetyki GPU

## The status bar's readouts: each word is followed by a value in a slot of fixed width. Keep the words short (one word each).

status-center = środek
status-cursor = kursor
status-zoom = zoom
# Short for iterations.
status-iter = iter
# The period of the minibrot jumped to.
status-period = okres
# A Game of Life view's scale (followed by "1 px = 4 cells" or "8 px / cell").
status-scale = skala
# A Game of Life view's generation. Short.
status-generation = pok.
# A Game of Life view's population. Short.
status-population = pop.
# An L-system's order (how many times its rules were applied).
status-order = rząd
# How many segments and shapes an L-system drew.
status-drawn = narys.
# $value: how many cells one pixel covers (4, 9.99e300). "px" is the unit symbol: keep it.
life-scale-cells = 1 px = { $value } kom.
# $value: how many pixels one cell covers (8). "px" is the unit symbol: keep it.
life-scale-pixels = { $value } px / kom.

## The status bar's limit diagnostic: a short label (after a warning sign), and what it means when hovered.

# The view is as deep as this location can be rendered. Short.
status-limit-depth = limit głębokości
# Every pixel ran out of iterations. Short.
status-limit-exhausted = iteracje wyczerpane
# Pixels are capped at the iteration budget. Short.
status-limit-capped = iteracje ograniczone
# $pixels: how many pixels wide one step of the coordinates is; $sharp_to: the magnification the view is sharp to (written out, 1,234,567).
status-limit-direct-detail = Ten widok renderuje się bezpośrednio na GPU w pojedynczej precyzji, a jeden krok współrzędnych ma tu szerokość { $pixels } px: sąsiednie piksele obliczają ten sam punkt, więc obraz rozpada się na bloki. W tej lokalizacji jest ostry do około { $sharp_to }×. Ten wzór nie ma jeszcze ścieżki głębokiego zoomu (perturbacji), więc dalszy zoom nie pokazuje więcej szczegółów.
# $samples: the GPU buffer's capacity in orbit samples; $iterations: the iterations pixels are limited to.
status-limit-depth-detail = Orbita referencyjna osiągnęła limit bufora GPU (próbki: { $samples }), nie uciekając, więc piksele są tu ograniczone do ~{ $iterations } iteracji, a drobniejszych szczegółów nie da się obliczyć w tym widoku — ani na żywo, ani w eksporcie. To obecny limit głębokości dla tej rodziny lokalizacji.
# $budget: the largest iteration budget this view can be given.
status-limit-exhausted-detail = Każdy piksel nadal wyczerpał iteracje przy największym budżecie, jaki można dać temu widokowi ({ $budget }). Albo widok jest tu wewnątrz zbioru (normalne w głębi), albo wymaga więcej iteracji, niż aplikacja może obecnie zapewnić — obraz jest taki sam w obu przypadkach, więc budżet przywrócono do taniego.
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations" is the setting's name in the control panel's Quality section.
status-limit-capped-max = { $percent }% pikseli wyczerpało iteracje przy budżecie { $budget }, a te, które uciekają, kończą tuż przy tym pułapie — widok jest niedostatecznie rozdzielony. Iteracje są już na maksimum, więc ten widok potrzebuje więcej, niż aplikacja może obecnie dać.
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations", "Controls" and "Quality" are the names of a setting, the control panel and its section.
status-limit-capped-raise = { $percent }% pikseli wyczerpało iteracje przy budżecie { $budget }, a te, które uciekają, kończą tuż przy tym pułapie — widok jest niedostatecznie rozdzielony. Zwiększ Iteracje (Sterowanie ▸ Jakość), aby to rozwiązać.
