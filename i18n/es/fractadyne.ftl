### Fractadyne user interface: Spanish (Español).
### A draft: machine translation (Claude, by Anthropic) of the English file, 2026-10-23, not yet
### reviewed by a native speaker. Corrections are welcome as pull requests (see i18n/README.md).

## Language data: read by Fractadyne, not shown as messages.

# This language's own name, shown in the language menu.
language-name = Español
# The same name in English, shown beside it in the menu.
language-english-name = Spanish
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
language-more = Más idiomas…
# The Languages window's title.
language-window-title = Idiomas
# The Languages window's introduction.
language-window-intro = La interfaz de Fractadyne en otro idioma. Cada idioma es un archivo: viene con una versión, o puedes instalarlo aquí. Las coordenadas, los archivos y la línea de comandos siguen en inglés en todos los idiomas.
# Column headers of the installed-language list.
language-col-language = Idioma
language-col-translated = Traducido
language-col-from = Origen
# Where a language comes from.
language-source-builtin = integrado
language-source-package = esta versión
language-source-user = instalado
language-source-dev = carpeta de trabajo
# $percent: how much of the interface the file translates, 0–100.
language-coverage = { $percent }%
# $count: messages in the file that failed the checks and show in English instead.
language-dropped =
    { $count ->
        [one] un mensaje necesita revisión
       *[other] { $count } mensajes necesitan revisión
    }
# Buttons of the Languages window.
language-install = Instalar archivo de idioma…
language-install-hover = Elige un archivo de idioma de Fractadyne (.ftl). Se comprueba y después se copia en la carpeta de idiomas de tu configuración, donde se conserva entre actualizaciones.
language-open-folder = Abrir carpeta de idiomas
language-reload = Recargar
language-reload-hover = Volver a leer los archivos de idioma (después de editar o añadir uno).
language-use = Usar
# Notices after installing a file. $name: the language's own name; $file: the file's name.
language-installed = { $name } instalado desde { $file }.
# $file: the file's name; $why: what was wrong with it.
language-install-failed = { $file } no es un archivo de idioma utilizable: { $why }
# Shown when a language file cannot be loaded and English is used instead. $name: the language.
language-load-failed = No se pudo cargar { $name }, así que Fractadyne está en inglés: { $why }
# A language whose writing system needs a font this computer does not have (Japanese or Chinese without a system font). $name: the language, in its own name.
language-font-missing = No se encontró en este equipo ninguna fuente que pueda dibujar { $name }, así que no se puede mostrar. Instala una fuente de { $name } (por ejemplo, el paquete de idioma del propio sistema) y vuelve a elegirlo.
# Shown after a draft translation's name in the language menu and the Languages window. Short.
language-draft = borrador
language-draft-hover = Un borrador: un hablante nativo aún no ha revisado esta traducción. Las correcciones son bienvenidas, como pull request al archivo del idioma.
# The heading over the decimal-mark choice, in the language menu and the Languages window.
language-decimal-mark = Separador decimal
language-decimal-mark-hover = Cómo se escriben los números: tiempos, tamaños y recuentos. Las coordenadas y los aumentos siempre usan punto, tal como los leen los archivos y otros programas.
# $sample: a number written with this language's own punctuation (1,234.5).
language-decimal-language = Como lo escribe el idioma ({ $sample })
# $sample: a number written with a decimal point (1,234.5).
language-decimal-point = Punto ({ $sample })
# $sample: a number written with a decimal comma (1.234,5).
language-decimal-comma = Coma ({ $sample })

## Asked once when the system's language changes

# The window's title.
language-changed-title = Idioma
# $new: the system's new language, in its own name.
language-changed-question = El idioma de tu sistema ahora es { $new }. ¿Usarlo en Fractadyne?
# Buttons. $name: a language's own name.
language-changed-use = Usar { $name }
language-changed-keep = Mantener { $name }

## The welcome window (first run)

welcome-title = Te damos la bienvenida a Fractadyne
welcome-intro = Un explorador de zoom profundo para el conjunto de Mandelbrot y sus parientes. Unos cuantos controles para empezar a moverte:
# The language row, at the top of the window.
welcome-language = Idioma
# Controls: the key or gesture on the left, what it does on the right.
welcome-key-drag = arrastrar
welcome-key-drag-does = desplazar la vista
welcome-key-scroll = rueda
welcome-key-scroll-does = acercar / alejar hacia el cursor
welcome-key-space = mantener Space
welcome-key-space-does = zoom continuo y suave (Shift+Space para alejar)
# Shown after the click-zoom toolbar icon.
welcome-key-click = y luego clic
welcome-key-click-does = sumergirse en el punto pulsado (barra de herramientas)
welcome-key-m-does = saltar al minibrot más cercano, a su propia escala
welcome-landmarks = Salta a un lugar destacado para empezar:
welcome-theme = Tema
welcome-watermark = Mostrar la marca «Fd»
welcome-watermark-hover = La pequeña marca en la esquina inferior derecha de la vista. Si se desactiva, desaparece de la vista en vivo y de las imágenes exportadas.
welcome-updates = Actualizaciones
welcome-check-updates = Buscar actualizaciones
welcome-check-updates-hover = Pregunta a GitHub por la última versión al iniciar la aplicación. No se envía nada.
welcome-files = Archivos
welcome-no-config = (sin carpeta de configuración)
welcome-open = Abrir
welcome-rendering-on = Renderizando en
welcome-open-help = Abrir la ayuda completa  (F1)
# $icon: the Settings icon.
welcome-more-in-settings = Más en Archivo → { $icon } Configuración
welcome-get-started = Empezar

## Menus: shared

# A percentage in a menu (UI scale). $value: the number.
common-percent = { $value }%
# A state shown after a menu item's name, in brackets.
menu-state-connected = conectado
menu-state-rendering = renderizando
menu-state-running = en marcha

## Navigate ▸ Bookmarks (also the toolbar's star)

menu-bookmarks = Marcadores
menu-bookmarks-add = Añadir la vista actual
menu-bookmarks-manage = Administrar…

## The fractal picker (Fractal menu and the toolbar's dropdown)

menu-family-powers = Potencias superiores
menu-family-automata = Autómatas
menu-family-lsystems = Sistemas L
menu-fractal-custom = Fórmula personalizada…
menu-fractal-library = Biblioteca de fórmulas…
menu-fractal-library-hover = Fórmulas personalizadas guardadas: aplicarlas, editarlas, importarlas y exportarlas
# The "Show" group: which picture of the formula is drawn. $name: the formula's name (Mandelbrot, Burning Ship…).
menu-show = Mostrar
menu-show-set = Conjunto de { $name }
menu-show-set-hover = El plano de parámetros: el propio conjunto de { $name }.

    Cada uno de sus puntos es un conjunto de Julia distinto; esto es el mapa de todos ellos.
menu-show-julia = Conjunto de Julia
menu-show-julia-hover = Un punto del conjunto de { $name }, dibujado como una imagen propia: la forma que se obtiene al mantener fijo ese punto.

    Elige el punto pasando el cursor por el mapa en «Ambos, enlazados»; un clic allí lo fija.
menu-show-julia-none = { $name } no tiene ningún parámetro libre, así que no tiene conjunto de Julia.
menu-show-both = Ambos, enlazados
menu-show-both-hover = El conjunto de { $name } a la izquierda y, a la derecha, el conjunto de Julia del punto que tenga debajo el cursor.

    Mueve el cursor por el mapa y observa cómo cambia el Julia: es la forma más rápida de ver cómo se relacionan los dos.
menu-show-both-none = { $name } no tiene ningún conjunto de Julia con el que emparejarse.

## File menu

menu-file = Archivo
menu-file-open-view = Abrir vista o ubicación…
menu-file-gallery = Galería…
menu-file-export = Exportar imagen…
menu-file-snapshot = Instantánea
menu-file-open-life = Abrir patrón de Life…
menu-file-open-life-hover = Un archivo RLE, .cells o Life 1.05/1.06, mostrado como un universo de Life.
menu-file-life-text = Texto de patrón de Life…
menu-file-life-text-hover = Pegar un patrón como texto (RLE, tal como lo da LifeWiki).
menu-file-save-life = Guardar patrón de Life…
menu-file-save-life-hover = El universo tal como está ahora, en RLE.
menu-file-open-lsystem = Abrir sistema L…
menu-file-open-lsystem-hover = Un archivo .l de Fractint (con sus entradas para elegir) o un sistema .lsys.
menu-file-save-lsystem = Guardar sistema L…
menu-file-save-lsystem-hover = El texto del sistema, como archivo .lsys.
menu-file-export-svg = Exportar SVG…
menu-file-export-svg-hover = La vista como dibujo SVG: sus líneas y formas como vectores, en sus colores.
menu-file-render-client = Cliente de renderizado…
menu-file-render-client-hover = Prestar la GPU de este equipo a una granja de renderizado: conectarse a un coordinador (Herramientas ▸ Renderizar en granja… en otro equipo) y renderizar fotogramas para él.
menu-file-settings = Configuración
menu-file-reset = Restablecer el estado de la aplicación…
menu-file-reset-hover = Eliminar todo el estado guardado (sesión, marcadores, miniaturas) y empezar de cero. Antes pide confirmación.
menu-file-quit = Salir

## File ▸ Settings

menu-settings-fps = Límite de fotogramas
menu-settings-fps-uncapped = Sin límite
# $fps: frames per second.
menu-settings-fps-n = { $fps } FPS
menu-settings-ui-scale = Escala de la interfaz (tamaño de letra)
menu-settings-theme = Tema
menu-settings-updates = Actualizaciones
menu-settings-updates-stable-hover = Última versión estable
menu-settings-updates-beta-hover = Última compilación, incluidas las versiones preliminares
menu-settings-updates-on-launch = Buscar actualizaciones al iniciar
menu-settings-updates-on-launch-hover = Si no, búscalas a mano con Ayuda → Buscar actualizaciones.
# Followed by its shortcut, (Ctrl+S).
menu-settings-snapshot = Instantánea
menu-settings-snapshot-ask-hover = Ofrecer la elección en cada pulsación
menu-settings-snapshot-screen-hover = Guardar la vista exactamente como aparece en pantalla (inmediato, PNG)
menu-settings-snapshot-render-hover = Un renderizado completo con los ajustes de exportación, en segundo plano (puede tardar mucho en vistas profundas)
menu-settings-refcache = Caché de referencias…
menu-settings-refcache-hover = Órbitas de referencia de zoom profundo guardadas en disco, para que una ubicación a la que vuelvas se renderice en segundos. Dónde está, cuánto ocupa, el límite y una forma de vaciarla.

## Fractal menu

menu-fractal = Fractal

## View menu

menu-view = Ver
menu-view-zoom-home = Alejar hasta la vista completa
menu-view-zoom-home-hover = Alejar hasta la vista inicial completa (animado)
menu-view-reset = Restablecer la vista predeterminada
menu-view-reset-hover = Volver a la vista predeterminada del fractal (inmediato)
# Followed by its shortcuts.
menu-view-undo = Deshacer vista
menu-view-redo = Rehacer vista
menu-view-control-panel = Panel de control
menu-view-control-panel-hover = Mostrar u ocultar el panel de control de la derecha.
menu-view-minimap = Minimapa
menu-view-minimap-hover = Mostrar una pequeña vista general de la vista inicial, con un marcador «estás aquí» y la profundidad del zoom. Haz clic en ella para saltar a una región.
menu-view-perf-panel = Panel de rendimiento
menu-view-timestamp = Mostrar marca de tiempo
menu-view-timestamp-hover = Dibujar sobre la vista un gran reloj de tiempo transcurrido. Muestra el mismo «+12.345s» con el que el registro marca cada línea, de modo que la grabación de pantalla de un problema que solo aparece en movimiento puede alinearse con el registro fotograma a fotograma. Desactivado de forma predeterminada.
menu-view-zoom-target = Mostrar el objetivo del zoom automático
menu-view-zoom-target-hover = Mientras se ejecuta el zoom automático, recuadrar la región de la vista hacia la que se acerca (la parte que llenará la pantalla tras unos cuantos aumentos más) con líneas desde sus esquinas hasta las esquinas de la pantalla, y un anillo en el punto hacia el que se dirige. Desactivado de forma predeterminada.
menu-view-live-panel = Panel de renderizado en vivo
menu-view-live-panel-hover = Un panel frontal sobre la vista que muestra el renderizador en vivo trabajando: qué despachó cada fotograma, qué antigüedad tiene la imagen en pantalla, el avance por las bandas de iteración, los constructores de referencias y la máquina que hay debajo. La tecla B recorre sus tamaños y la × de su esquina lo cierra; arrástralo a cualquier esquina; pasa el cursor sobre un instrumento para ver qué significa y qué aspecto tiene cuando todo va bien y cuando hay problemas.
# The live render panel's sizes.
menu-view-live-off = Desactivado
menu-view-live-off-hover = Sin panel.
menu-view-live-strip = Tira
menu-view-live-strip-hover = Los indicadores nixie y el banco de luces.
menu-view-live-panel-size = Panel
menu-view-live-panel-size-hover = Los instrumentos: dial de avance, edad de la imagen, teselas, pasadas de GPU, dial de profundidad, lámparas.
menu-view-live-full = Completo
menu-view-live-full-hover = El panel y la sala de máquinas: pasos, GPU, memoria, núcleos, píxeles.
menu-view-live-opacity = Opacidad
menu-view-watermark = Mostrar la marca de agua «Fd»
menu-view-watermark-hover = Dibujar una discreta marca «Fd» en la esquina inferior derecha de la vista en vivo y de las imágenes exportadas. Activada de forma predeterminada.

## Color menu

menu-color = Color
menu-color-method = Método
menu-color-palette = Paleta
menu-color-custom = Personalizada
menu-color-duotone = Bitono
menu-color-binary = Binario (conjunto)
menu-color-binary-hover = Dos colores planos: dentro del conjunto frente a fuera de él, sin degradado.
menu-color-edit-gradient = Editar degradado…
menu-color-normalize = Normalizar colores profundos
menu-color-normalize-hover = Reasignar la paleta al rango de escape medido en la vista cuando un campo denso se vería, si no, como un moteado. Una vista sin aliasing se deja tal cual; marca «Ajustar siempre el rango» para reasignarla de todos modos. Solo con el método suave.
menu-color-more = Ciclo, desplazamiento, animación y efectos: panel Controles ▸ Coloreado

## Tools menu

menu-tools = Herramientas
# Followed by its shortcut, (M).
menu-tools-find-minibrot = Buscar minibrot + acercarse a él
menu-tools-find-minibrot-hover = Ajustar por Newton el centro de la vista al centro exacto del minibrot cercano, indicar su período y acercarse hasta la propia escala del minibrot: a menudo, muchos órdenes de magnitud en un solo paso. ¿Ya estás a más profundidad que él? Solo se mueve el centro.
menu-tools-solver = Solucionador de Newton / Misiurewicz…
menu-tools-solver-hover = Calcular el centro exacto del minibrot (período + tamaño) o el punto de Misiurewicz más cercano a la vista: método de Newton con precisión completa, con el multiplicador repulsor λ. Abre el cuadro de diálogo de ubicación, que aloja los solucionadores.
# Followed by its shortcut, (A).
menu-tools-autopilot = Zoom automático (piloto automático)
menu-tools-autopilot-stop = Detener el piloto automático
menu-tools-autopilot-hover = Zoom profundo continuo sin manos: se sumerge hacia la región con más detalle y va corrigiendo el rumbo sobre la marcha. Cualquier acción de navegación lo detiene.
menu-tools-priority = Prioridad:
menu-tools-priority-speed = Velocidad
menu-tools-priority-speed-hover = Hacer zoom a la velocidad fijada; refrescar con el primer fotograma que tenga detalle.
menu-tools-priority-quality = Calidad
menu-tools-priority-quality-hover = Solo fotogramas totalmente resueltos; el zoom los espera.
menu-tools-target = Objetivo:
menu-tools-target-detail = Detalle
menu-tools-target-detail-hover = El borde más rico de la vista, que se vuelve a elegir a medida que avanza la inmersión.
menu-tools-target-misiurewicz = Punto de Misiurewicz
menu-tools-target-misiurewicz-hover = El punto de Misiurewicz más cercano, calculado una sola vez y en el que se sumerge con exactitud (solo Mandelbrot).
menu-tools-orbits = Superposición de la órbita
menu-tools-orbits-hover = Dibujar la trayectoria de iteración del punto bajo el cursor. Opciones de animación: panel Controles ▸ Superposiciones.
menu-tools-orbits-normalize = Normalizar (ajustar a la vista)
menu-tools-orbits-normalize-hover = Ajustar la órbita a toda la vista para que quede bien encuadrada con cualquier zoom (en lugar de proyectarla a través de la ventana de visualización, donde se sale de la pantalla en zonas profundas).
menu-tools-play-tour = Reproducir recorrido…
menu-tools-play-tour-hover = Elegir un guion de recorrido para reproducirlo. El ▶ de la barra de herramientas vuelve a reproducir el último con un solo clic.
menu-tools-tour-from-view = Recorrido desde la vista actual…
menu-tools-tour-from-view-hover = Crear un recorrido que hace zoom desde la vista completa hasta la vista actual: añade un rótulo y fija la duración de la inmersión. Se guarda como un guion de recorrido (.toml) que puedes editar, reproducir o renderizar.
menu-tools-render-tour = Renderizar recorrido…
menu-tools-render-tour-hover = Renderizar el recorrido cargado como una secuencia de fotogramas PNG (y, opcionalmente, un mp4) en un proceso aparte.
menu-tools-close-player = Cerrar el reproductor de recorridos
menu-tools-close-player-hover = Detener el recorrido, cerrar su reproductor y restaurar tus propios ajustes de iteración y coloreado
menu-tools-render-farm = Renderizar en granja…
menu-tools-render-farm-hover = Renderizar el recorrido cargado en varios equipos: este coordina el trabajo y los demás se unen con Archivo ▸ Cliente de renderizado….
menu-tools-benchmark = Prueba de rendimiento…
menu-tools-benchmark-hover = Medir la velocidad de renderizado: con los ajustes actuales o en una ejecución estandarizada (resolución + ajustes fijos) comparable entre equipos. La prueba de resistencia la repite para comprobar la estabilidad / la limitación térmica.
menu-tools-last-benchmark = Mostrar la última prueba de rendimiento

## Navigate menu

menu-navigate = Navegar
menu-navigate-goto = Ir a la ubicación…
menu-navigate-goto-hover = Introducir un centro/zoom, saltar a un punto conocido o resolver por Newton un elemento de Misiurewicz / minibrot cerca de la vista.
menu-navigate-misiurewicz = Explorador de Misiurewicz…
menu-navigate-misiurewicz-hover = Ver qué aspecto tienen los tipos pequeños de Misiurewicz (miniaturas con el período de zoom y el giro de cada punto) y resolver por Newton cualquiera de ellos hasta una profundidad arbitraria. (Mandelbrot.)
menu-navigate-exact = Puntos exactos (expresiones)
menu-navigate-exact-note = Raíces de los bulbos de la cardioide principal, dadas como expresiones exactas (cos/sin de una fracción de 2π). La coordenada se conserva como su expresión, así que al acercarte te mantienes exactamente sobre el punto.
menu-navigate-random = Ubicación aleatoria
menu-navigate-random-hover = Saltar a un punto aleatorio de la frontera, rico en detalle
menu-navigate-import-kfr = Importar .kfr…
menu-navigate-import-kfr-hover = Cargar un archivo de ubicación de Kalles Fraktaler
menu-navigate-share = Compartir ubicación…
menu-navigate-share-hover = Copiar / pegar / guardar / cargar una ubicación autónoma (.fdn): fractal, centro con precisión completa, zoom, coloreado.

## Help menu

menu-help = Ayuda
# Followed by its shortcut, (F1).
menu-help-reference = Ayuda y referencia…
menu-help-welcome = Bienvenida / inicio rápido…
menu-help-welcome-hover = La presentación del primer inicio: los controles esenciales y unos cuantos lugares destacados.
menu-help-diagnostics = Diagnóstico…
menu-help-diagnostics-hover = Comprobar que Fractadyne funciona correctamente en tu hardware y adjuntar el resultado a un informe de problema
menu-help-report = Informar de un problema…
menu-help-report-hover = Abrir una incidencia en GitHub (o enviar un correo) con un informe de fallo, el registro y la ubicación adjuntos
menu-help-accelerated = Zoom profundo más rápido…
menu-help-accelerated-hover = Una compilación opcional que calcula las órbitas de referencia de zoom profundo entre 2,5 y 6,4 veces más rápido. Imágenes idénticas; tus ajustes se conservan.
menu-help-check-updates = Buscar actualizaciones
# $track: the update track (stable or beta).
# $track: "stable" or "beta", the update track in use (the variants hold the wording).
menu-help-check-updates-hover =
    { $track ->
        [beta] Buscar en GitHub una compilación Beta (versiones preliminares) más reciente
       *[other] Buscar en GitHub una compilación Estable más reciente
    }
# $version: the new version; $channel: "beta" for a pre-release, "stable" otherwise (the variants hold the wording).
menu-help-update-available =
    { $channel ->
        [beta] { $version } (beta) disponible — descargar
       *[other] { $version } (estable) disponible — descargar
    }
menu-help-up-to-date = Actualizado
menu-help-tagline = Explorador de fractales nativo
menu-help-license = Licencia
# $year and $holder: the copyright year and holder, as the code gives them.
menu-help-copyright = © { $year } { $holder }. Con doble licencia; úsalo bajo cualquiera de las dos licencias, a tu elección.
menu-help-source = Código fuente

## The toolbar (its buttons are icons; these are their tooltips)

menu-toolbar-julia = Julia
menu-toolbar-julia-hover = Mostrar el conjunto de Julia de esta fórmula, ocupando toda la ventana
menu-toolbar-dual-hover = Ambos, enlazados: el conjunto a la izquierda y, a la derecha, el Julia del punto del cursor
menu-toolbar-open-hover = Abrir una vista o ubicación (PNG / EXR, .fdn, .kfr)
menu-toolbar-gallery-hover = Galería
menu-toolbar-export-hover = Exportar imagen…
menu-toolbar-share-hover = Compartir ubicación: copiar, pegar, guardar o cargar un .fdn autónomo (fractal, centro con precisión completa, zoom, coloreado)
menu-toolbar-zoom-in-hover = Acercar
menu-toolbar-zoom-out-hover = Alejar
# $factor: the click-zoom factor (4, 10…).
menu-toolbar-click-zoom-hover = Clic para zoom ({ $factor }×): el clic izquierdo se sumerge en el punto y el clic derecho retrocede (arrastrar sigue desplazando). El factor está en Controles ▸ Navegación.
# Added to the click-to-zoom tooltip in the dual view.
menu-toolbar-click-zoom-dual = Mientras esto esté activado, fija la c de Julia con Ctrl+clic.
menu-toolbar-autopilot-running-hover = El zoom automático está en marcha: haz clic para detenerlo
menu-toolbar-autopilot-hover = Zoom automático: sumergirse hacia el detalle (A)
menu-toolbar-reset-hover = Restablecer la vista (inmediato)
menu-toolbar-home-hover = Alejar hasta la vista inicial (animado)
menu-toolbar-bookmarks-hover = Marcadores: guardar esta vista o saltar a una guardada
# $name: the palette's name.
menu-toolbar-next-palette-hover = Paleta siguiente ({ $name })
# $file: the last tour's file name.
menu-toolbar-play-last-hover = Reproducir recorrido: { $file } (clic) · vuelve a reproducir el último recorrido; elige otro con Herramientas ▸ Reproducir recorrido…
menu-toolbar-play-hover = Reproducir un recorrido… (Herramientas ▸ Reproducir recorrido)
menu-toolbar-speed-hover = Velocidad de reproducción
menu-toolbar-fullscreen-hover = Alternar pantalla completa

## Dialogs: words shared by many windows

# A button that closes a window without doing what it offers.
dlg-cancel = Cancelar
# A button that dismisses a window (nothing in it is pending).
dlg-close = Cerrar
# A button that copies the window's text to the clipboard (after a copy icon).
dlg-copy = Copiar
# A button that jumps to a location.
dlg-go = Ir
# A button that fills the fields from the view on screen.
dlg-use-current = Usar la actual
# A notice after a file could not be written. $error: the system's reason, as it gives it.
dlg-save-failed = Error al guardar: { $error }
# A notice when no graphics device is available to render with.
dlg-gpu-not-available = GPU no disponible
# The name of the plain-text file type in a save dialog (the extension .txt is added by the code).
dlg-filter-text = Texto

## Asked after a session that did not shut down cleanly

dlg-crash-title = Fractadyne no se cerró correctamente
dlg-crash-intro = La sesión anterior terminó de forma inesperada. Se ha guardado un informe en este equipo: ¿quieres enviarlo?
dlg-crash-review = No se envía nada hasta que lo revises: la siguiente pantalla muestra exactamente lo que se incluiría, y la información del sistema se puede excluir con una sola casilla.
dlg-crash-report = Informar…
dlg-crash-report-hover = Abre Informar de un problema, con el informe de fallo y la ubicación que falló adjuntos.
dlg-crash-not-now = Ahora no
dlg-crash-dont-ask = No volver a preguntar tras un fallo
dlg-crash-dont-ask-hover = Los informes se siguen guardando localmente, y Ayuda ▸ Informar de un problema siempre funciona.

## Asked after a session that ended with a second graphics device in use

dlg-second-device-title = ¿Desactivar el segundo dispositivo gráfico?
# $device: the graphics device's name and details, as the system gives them.
dlg-second-device-intro = La sesión anterior terminó de forma inesperada mientras se usaba un segundo dispositivo gráfico ({ $device }). Puede ser la causa.
dlg-second-device-note = Con él desactivado, la tarjeta que dibuja la ventana lo renderiza todo, como antes. Puedes volver a activarlo en Avanzado ▸ Segunda tarjeta gráfica.
dlg-second-device-off = Desactivarlo
dlg-second-device-keep = Mantenerlo activado

## Shown when the previous session's iteration setting was set aside

dlg-quarantine-title = Ajuste de iteraciones apartado
dlg-quarantine-change = Puedes cambiarlo en cualquier momento en Controles ▸ Calidad.
dlg-quarantine-keep = Mantener el predeterminado
# A button. $setting: the iteration setting the previous session used (such as "20,000 iterations").
dlg-quarantine-restore = Restaurar { $setting }

## Navigate ▸ Go to location

dlg-goto-title = Ir a la ubicación
# Before two choices: "x, y" (cartesian) and the polar form below.
dlg-goto-enter-as = Introducir como:
# The polar way of entering a point: an offset plus a radius r at an angle θ. Keep the symbols.
dlg-goto-polar = desplazamiento + r ∠ θ
# The tooltip of the "?" button beside the entry choice.
dlg-goto-help-hover = Qué aceptan los campos: fracciones, potencias, sqrt/sin/cos…, pi y otras constantes. Abre Ayuda → Expresiones de coordenadas
# Labels over the entry fields. x0, y0, r, θ, Re and Im are the fields' symbols: keep them.
dlg-goto-polar-offset = Centro desplazado  (x0, y0 — deja 0 para un punto polar puro)
dlg-goto-polar-radius = Radio r, ángulo θ
dlg-goto-re = Centro — parte real (Re)
dlg-goto-im = Centro — parte imaginaria (Im)
dlg-goto-zoom = Zoom (aumento)
# A warning under the zoom field, after a ⚠ sign. $have: the iteration count set; $typical: the count this depth typically needs.
dlg-goto-iter-warning = Las iteraciones están fijadas en { $have }; esta profundidad suele necesitar ~{ $typical }. La vista se renderizará plana hasta que se aumenten, o activa las iteraciones automáticas (Controles ▸ Calidad), que se adaptan solas.
dlg-goto-copy-hover = Copiar esta ubicación al portapapeles
dlg-goto-cancel-hover = Cerrar sin ir a ninguna parte
# The expressions in brackets are examples of what can be typed: keep them as they are.
dlg-goto-paste-note = Pega un centro/zoom de otra persona, o pulsa Copiar para compartir. Las coordenadas aceptan expresiones: fracciones exactas (-3/4), funciones y constantes (-0.5 + 0.25*cos(pi/4), sqrt, root(x,n), ^; ángulos en radianes), y el campo real acepta un valor complejo completo ((37+16i)/100), que rellena ambos.
# A section that opens and closes.
dlg-goto-feature = Ir a un elemento (Mandelbrot)
dlg-goto-feature-poi = Puntos conocidos:
# The dropdown's text before a point is picked.
dlg-goto-feature-poi-pick = Saltar a un punto de interés…
dlg-goto-feature-solve = O calcular un elemento cerca de la vista actual:
# Two choices: what to solve for. Misiurewicz is a mathematician's name (a Misiurewicz point).
dlg-goto-feature-misiurewicz = Misiurewicz
dlg-goto-feature-minibrot = Minibrot más cercano
# Labels of two small number fields; k and p are their symbols.
dlg-goto-feature-preperiod = Preperíodo k
dlg-goto-feature-period = Período p
# The faint text in an empty field: left empty, the value is found automatically.
dlg-goto-feature-auto-hint = auto
# A small button that empties both fields, so they are found automatically again.
dlg-goto-feature-auto = Auto
dlg-goto-feature-auto-hover = Vaciar ambos campos para que la próxima búsqueda detecte el preperíodo y el período a partir de la órbita en el centro de la vista.
dlg-goto-feature-kp-hover = Déjalos ambos en blanco para detectarlos a partir de la órbita en el centro de la vista; los valores encontrados se rellenan aquí. Nadie que mire una espiral conoce su preperíodo, y tener que indicarlo es lo que hacía inutilizable este solucionador en la práctica.
# While a solve runs, followed by its elapsed time and the note below.
dlg-goto-feature-solving = Resolviendo…
dlg-goto-feature-solving-note = (precisión arbitraria; las vistas profundas tardan más)
dlg-goto-feature-cancel-hover = Dejar de esperar. El cálculo en sí sigue hasta terminar en segundo plano y su respuesta se descarta: nada queda a medio hacer.
dlg-goto-feature-find = Buscar cerca de la vista
dlg-goto-feature-find-hover =
    Resolver por Newton desde el centro actual hasta el elemento exacto (precisión arbitraria). Primero navega cerca del elemento.

    Antes de resolver, pon en el campo Zoom de arriba la profundidad que QUIERES: el punto se calcula con la precisión que necesita esa profundidad y la vista salta directamente allí. Es un solo cálculo en lugar de una larga serie de pasos de zoom manuales.
dlg-goto-solve-cancelled = Cálculo cancelado.

## Snapshot (the camera button): its tooltip, and the choice on the first press

# The camera button's tooltip, one per Snapshot setting. Followed by its shortcut, (Ctrl+S).
dlg-snapshot-hover-ask = Instantánea: guardar la vista tal como se ve, o un renderizado completo; la primera pulsación pregunta
dlg-snapshot-hover-screen = Instantánea: guardar la vista exactamente como se ve, en la última carpeta de exportación
dlg-snapshot-hover-render = Instantánea: renderizado completo con los ajustes de exportación, en la última carpeta de exportación
# The window's title.
dlg-snapshot-title = Instantánea
dlg-snapshot-question = ¿Qué debe guardar Instantánea (el botón de la cámara, Ctrl+S)?
# The first choice, and a short tag beside it.
dlg-snapshot-screen = Captura de pantalla
dlg-snapshot-screen-tag = inmediata
dlg-snapshot-screen-note = Guarda la vista exactamente como aparece en pantalla, a la resolución de la pantalla, como PNG en la última carpeta de exportación. Lo que ves es lo que obtienes.
# The second choice, and a short tag beside it.
dlg-snapshot-render = Renderizado completo
dlg-snapshot-render-tag = puede tardar mucho
# $size: the image size, as 1920×1080; $ss: the supersampling factor (1–4); $format: PNG or EXR.
dlg-snapshot-render-note = Renderiza con los ajustes de exportación: { $size }, supermuestreo { $ss }×, { $format }. La mejor calidad, pero una vista profunda puede tardar de minutos a horas. Se ejecuta en segundo plano; Archivo → Exportar imagen… muestra el progreso y permite cancelarlo.
dlg-snapshot-remember = Recordar mi elección y no volver a preguntar
dlg-snapshot-change = Puedes cambiarlo en cualquier momento en Archivo → Configuración → Instantánea.

## Share location (.fdn)

dlg-share-title = Compartir ubicación
dlg-share-intro = Una ubicación autónoma (fractal, centro con precisión completa, zoom, coloreado). Cópiala para compartirla, o pega/carga una y pulsa Aplicar.
dlg-share-thumb = Incluir una miniatura en el archivo guardado
dlg-share-thumb-hover = Guardar .fdn incrusta una pequeña imagen de esta vista para que el archivo se pueda reconocer en la galería. Añade unos 55 KB y no forma parte del texto de arriba.
dlg-share-copy-hover = Copiar el texto al portapapeles
# Buttons (the first after a save icon).
dlg-share-save = Guardar .fdn…
dlg-share-load = Cargar .fdn…
dlg-share-apply = Aplicar
dlg-share-apply-hover = Saltar a la ubicación del cuadro
dlg-share-copied = Copiado al portapapeles.

## Help ▸ Report an issue (the report itself stays in English: it is read by the maintainer)

dlg-report-title = Informar de un problema
dlg-report-intro = El mejor lugar para informar es una incidencia en GitHub: es pública y se puede buscar, y verás cuándo se corrige. El correo electrónico también sirve. Revisa primero todo lo de abajo: nada sale de tu equipo hasta que tú actúes.
# Labels before three dropdowns.
dlg-report-type = Tipo:
dlg-report-severity = Gravedad:
dlg-report-repro = Reproducible:
dlg-report-describe = Describe lo que pasó:
# The faint text in the empty description box.
dlg-report-describe-hint = Qué pasó, los pasos para reproducirlo y qué esperabas en su lugar.
# Before the checkboxes of what the report carries.
dlg-report-include = Incluir:
dlg-report-sysinfo = Información del sistema (versión, SO, CPU, GPU, VRAM)
dlg-report-location = Ubicación actual (.fdn)
dlg-report-log = Registro reciente
dlg-report-frames = Registro de fotogramas (el último minuto y los últimos 40 fotogramas)
dlg-report-crash = Último informe de fallo
dlg-report-crash-none = Último informe de fallo (no se encontró ninguno)
dlg-report-crash-view = Ubicación que falló (.fdn)
dlg-report-crash-view-none = Ubicación que falló (.fdn) (no se encontró ninguna)
dlg-report-crash-view-hover = La vista exacta en la que se produjo un fallo o una pérdida del dispositivo gráfico: lo único que permite a un mantenedor reproducirlo. Se registra automáticamente.
dlg-report-test = Resultado de la prueba de diagnóstico
dlg-report-test-none = Resultado de la prueba de diagnóstico (ejecuta una desde Ayuda → Diagnóstico…)
dlg-report-test-hover = Un aprobado/suspenso validado por la máquina en tu propio hardware: mucho más útil para un mantenedor que una simple descripción
# A section that opens and closes. $size: the report's size, as "12 KB".
dlg-report-preview = Vista previa de lo que se enviará  ({ $size })
dlg-report-github = Abrir una incidencia en GitHub
dlg-report-github-hover = Abrir una página de nueva incidencia con el título rellenado; el informe se copia al portapapeles: pégalo en el cuerpo de la incidencia (Ctrl+V).
# Buttons (after a copy and a save icon).
dlg-report-copy = Copiar informe
dlg-report-copy-hover = Copiar todo al portapapeles
dlg-report-save = Guardar informe…
dlg-report-save-hover = Guardar como .txt para adjuntarlo
dlg-report-by-email = O por correo electrónico:
dlg-report-gmail = Redactar en Gmail
# $address: the email address reports go to.
dlg-report-gmail-hover = Abrir una ventana de redacción de Gmail para { $address } con el asunto rellenado; pega el informe copiado
dlg-report-email = Aplicación de correo…
dlg-report-email-hover = Abrir tu aplicación de correo predeterminada (mailto): requiere tener configurado un gestor de correo
dlg-report-tip = Consejo: con cualquiera de estos botones, el informe se copia al portapapeles; pégalo en la incidencia o en el cuerpo del mensaje (Ctrl+V). Si puedes, adjunta también una captura de pantalla o una imagen de ejemplo (arrástrala); ayuda mucho con los problemas de renderizado y de interfaz.
# Notices in the window after a button is pressed.
dlg-report-opened-github = GitHub abierto: el informe está en el portapapeles; pégalo en la incidencia.
dlg-report-copied = Informe copiado al portapapeles.
# $path: where the report was saved.
dlg-report-saved = Guardado en { $path }
dlg-report-opened-email = Aplicación de correo abierta: el informe está en el portapapeles; pégalo.
dlg-report-opened-gmail = Redacción de Gmail abierta: pega el informe copiado en el cuerpo (Ctrl+V).

## A location whose checksum does not match

dlg-checksum-title = Puede que esta ubicación esté dañada
# After a ⚠ sign.
dlg-checksum-mismatch = Su suma de comprobación no coincide con los datos que la acompañan.
dlg-checksum-why = Eso suele significar que el texto se cortó o se alteró por el camino: la causa habitual es una copia parcial desde una ventana de chat o una publicación de un foro.
# A label over where the location came from (a file path, or the clipboard).
dlg-checksum-source = Origen
# The two checksums, side by side: the one written in the file, and the one computed from its data.
dlg-checksum-in-file = en el archivo
dlg-checksum-from-data = según los datos
dlg-checksum-load-note = Puedes cargarla de todos modos: puede que las coordenadas estén bien, y no se sobrescribe nada. Lo que no se pueda leer se indica después de cargar.
dlg-checksum-load = Cargar de todos modos
dlg-checksum-load-hover = Saltar a ella a pesar de la discrepancia
# Notices after loading it. $zoom: the magnification, as 1.2e148.
dlg-checksum-loaded = Cargada @ { $zoom }× a pesar de la discrepancia en la suma de comprobación
# $zoom: the magnification, as 1.2e148; $note: what the loader found wrong with the location.
dlg-checksum-loaded-note = Cargada @ { $zoom }× — { $note }
dlg-checksum-not-loaded = No se cargó.

## File ▸ Reset application state

dlg-reset-title = Restablecer el estado de la aplicación
# After a ⚠ sign, followed by a list.
dlg-reset-warning = Esto elimina de forma permanente todos los datos guardados de Fractadyne:
# The list's items, each after a bullet.
dlg-reset-session = la sesión guardada (vista actual, coloreado, preferencias)
dlg-reset-bookmarks = todos los marcadores y sus miniaturas
dlg-reset-refcache = la caché de órbitas de referencia (se reconstruye a medida que vuelves a visitar ubicaciones)
# A label over the folder that is deleted.
dlg-reset-location = Ubicación
dlg-reset-note = Esto no se puede deshacer. La sesión actual no se volverá a guardar al salir, así que en el próximo inicio se cargarán los valores predeterminados.
dlg-reset-confirm = Restablecer todo
dlg-reset-done = Estado de la aplicación restablecido: en el próximo inicio se cargarán los valores predeterminados.
# $error: the system's reason, as it gives it.
dlg-reset-failed = Error al restablecer: { $error }

## Tools ▸ Tour from current view

dlg-tour-title = Recorrido desde la vista actual
dlg-tour-intro = Crea un recorrido que hace zoom desde la vista completa hasta donde estás ahora.
# A label before the fractal and magnification the tour ends at.
dlg-tour-destination = Destino
dlg-tour-notation = Anotación (rótulo opcional que se muestra durante la inmersión)
# The faint example text in the empty caption box.
dlg-tour-notation-hint = p. ej., Inmersión en el Misiurewicz de tres brazos…
dlg-tour-duration = Duración del zoom
# $rate: how fast the dive zooms, in powers of ten a second (as 2.5).
dlg-tour-rate = ≈ { $rate } décadas/s
# A small button that puts the duration back to its default.
dlg-tour-reset = Restablecer
dlg-tour-reset-hover = Valor predeterminado según la profundidad
dlg-tour-duration-note = El valor predeterminado se ajusta a la profundidad para que la inmersión se pueda seguir con la vista. Las inmersiones más rápidas pueden mostrar destellos de fotogramas de un solo color con zoom extremo.
# A button, after a save icon.
dlg-tour-save = Guardar recorrido…
# The file type's name in the save dialog.
dlg-tour-filter = Guion de Fractadyne (TOML)
# A notice. $file: the saved file's name. --render-tour is a command-line option: keep it.
dlg-tour-saved = Recorrido guardado → { $file }. Reprodúcelo con Herramientas → Reproducir recorrido, o renderízalo con --render-tour.

## Navigate ▸ Bookmarks ▸ Manage…

dlg-bookmarks-title = Marcadores
# The faint text in the empty name box.
dlg-bookmarks-name-hint = nombre (opcional)
# A button, after a bookmark icon.
dlg-bookmarks-add = Añadir la vista actual
dlg-bookmarks-empty = Aún no hay marcadores. Añade uno desde la vista actual.
# The tooltip of a bookmark's delete (bin) button.
dlg-bookmarks-delete-hover = Eliminar

## Tools ▸ Benchmark…

dlg-bench-title = Prueba de rendimiento
dlg-bench-mode = Modo
dlg-bench-current = Ajustes actuales
dlg-bench-current-hover = Reproducir el recorrido fijo de zoom profundo en la vista en vivo, con tu resolución y tus ajustes actuales.
dlg-bench-standard = Estandarizada
dlg-bench-standard-hover = Fijar la resolución + todos los ajustes de renderizado para que la puntuación sea comparable entre equipos.
dlg-bench-resolution = Resolución
dlg-bench-depth = Profundidad
# Standard and Ultra are the two depths in the dropdown this describes.
dlg-bench-depth-hover = Ambas inmersiones entran en la ruta profunda floatexp/BLA; Ultra (1e48×) pasa allí mucho más tiempo que Estándar (1e32×) y, en consecuencia, tarda más en ejecutarse.
dlg-bench-burnin = Prueba de resistencia (repetir)
dlg-bench-burnin-hover = Ejecutar la prueba de rendimiento repetidamente para poner de manifiesto la estabilidad y la limitación térmica.
# The label of a slider: how many times the benchmark runs.
dlg-bench-passes = pasadas
# $size: the image size, as 1920×1080; $ss: the supersampling factor; $depth: the dive's final magnification, as 1e32.
dlg-bench-std-summary = Renderiza fuera de pantalla a { $size }, SS { $ss }×, Mandelbrot/suave, inmersión de 60 fotogramas hasta { $depth }×.
dlg-bench-run = Ejecutar

## The standardized benchmark's progress window

dlg-bench-progress-title = Ejecutando la prueba de rendimiento…
# $resolution: the benchmark's resolution, as the dropdown names it.
dlg-bench-progress-mode = Estandarizada · { $resolution }
# $pass: the pass running now; $total: how many passes there are.
dlg-bench-progress-pass = Pasada de resistencia { $pass }/{ $total }
dlg-bench-progress-dive = Renderizando la inmersión fija…
# On the progress bar. $done: frames rendered; $total: frames in the dive.
dlg-bench-progress-frame = fotograma { $done }/{ $total }
# $fps: frames per second, as 41.3.
dlg-bench-progress-last = última pasada: { $fps } fps

## The benchmark's results

dlg-bench-results-title = Resultados de la prueba de rendimiento
# Buttons, each after an icon.
dlg-bench-results-run-again = Volver a ejecutar…
dlg-bench-results-save = Guardar…
dlg-bench-results-none = Aún no se ha ejecutado ninguna prueba de rendimiento.
dlg-bench-results-saved = Prueba de rendimiento guardada.

## The accelerated build's notice when its arithmetic libraries are missing

dlg-backend-title = Compilación acelerada: se usa la aritmética integrada
dlg-backend-dont-show = No volver a mostrar

## File ▸ Gallery…

dlg-gallery-title = Galería
# A button that picks the folder to show.
dlg-gallery-folder = Carpeta…
# A button, after a refresh icon.
dlg-gallery-refresh = Actualizar
dlg-gallery-empty = No hay imágenes de Fractadyne en esta carpeta.
# An image's title when it does not name its fractal.
dlg-gallery-untitled = Imagen de Fractadyne
# $zoom: the image's magnification, as 1.2e148; $saved: when it was saved, as the file gives it.
dlg-gallery-zoom = zoom { $zoom }   { $saved }
# The notes saved with the image, in quotation marks. Use your language's quotation marks.
dlg-gallery-notes = «{ $notes }»
dlg-gallery-open = Abrir esta vista
dlg-gallery-loaded = Vista cargada desde la galería.

## File ▸ Export image…

dlg-export-title = Exportar imagen
# A dropdown of image sizes.
dlg-export-size = Tamaño
# The size dropdown's text for a size set by hand. $size: the size, as 1920×1080.
dlg-export-size-custom-current = Personalizado — { $size }
# The size dropdown's first choice.
dlg-export-size-custom = Personalizado…
dlg-export-size-custom-hover = Fija tú el ancho; el alto sigue la Proporción de abajo.
dlg-export-width = Ancho (px)
# After the width field. $height: the height in pixels, which follows from the aspect.
dlg-export-height-from-aspect = × { $height } (según la Proporción)
dlg-export-ss = Supermuestreo
dlg-export-aspect = Proporción
# The aspect choice that keeps the window's own shape.
dlg-export-aspect-window = Igual que la ventana
# Before the choices PNG and OpenEXR.
dlg-export-format = Formato:
dlg-export-hud = HUD de ubicación
dlg-export-hud-hover = Grabar un panel con el nivel de zoom + las coordenadas en la esquina superior izquierda de la imagen exportada (se escala con la salida; muestra el zoom + el centro de la vista del mapa/Mandelbrot).
dlg-export-glitch = Corrección de glitches
dlg-export-glitch-hover = Corrección de los glitches de perturbación con varias referencias. Se omite automáticamente en las exportaciones de vista única muy profundas (floatexp), para que la construcción de la referencia + el renderizado se ejecuten fuera del hilo principal y la aplicación siga respondiendo. Nunca se usa en vistas de Mandelbrot o Multibrot (fuera del modo Julia): comparada con un cálculo exacto, ahí no corregía nada y costaba hasta 5 veces el tiempo de exportación.
dlg-export-deep-note = Exportación profunda: la referencia se construye fuera del hilo principal (la interfaz sigue activa); a esta profundidad se omite la corrección de glitches.
# A dropdown, shown in the dual (map + Julia) view, and its three choices.
dlg-export-dual = Disposición doble
dlg-export-dual-side = Lado a lado
dlg-export-dual-separate = Archivos separados
dlg-export-dual-map = Solo el mapa
dlg-export-notes = Notas:
# The faint text in the empty notes box.
dlg-export-notes-hint = se guardan con la imagen (≤120 caracteres)
# $width and $height: the image size in pixels; $left: how many characters the notes can still take.
dlg-export-output = Salida: { $width } × { $height } px   (quedan { $left } caracteres)
# When the export's shape differs from the window's. $percent: how much more, as 25.
dlg-export-extra-width = Muestra un { $percent }% más de ancho que la ventana (no se recorta nada)
# The same, when the export is taller than the window. $percent: how much more, as 25.
dlg-export-extra-height = Muestra un { $percent }% más de alto que la ventana (no se recorta nada)
dlg-export-tiles-note = Se renderiza en teselas (sin límite de tamaño) en un hilo en segundo plano. El archivo incluye la vista + las notas, para poder volver a abrirlo con Archivo ▸ Abrir vista.
# A label before the button that picks the folder exports go to.
dlg-export-folder = Carpeta:
dlg-export-choose = Elegir…
dlg-export-choose-hover = Elegir la carpeta de destino de las exportaciones
dlg-export-open-after = Abrir al terminar
dlg-export-open-after-hover = Pasar la imagen terminada al visor del sistema en cuanto se haya escrito.
# $count: the graphics cards in this machine (2 or more).
dlg-export-all-gpus = Usar todas las tarjetas gráficas ({ $count })
dlg-export-all-gpus-hover = Repartir la imagen por teselas entre todas las tarjetas gráficas de este equipo, cada una con su propia copia de la referencia profunda. Tarjetas idénticas dan exactamente la imagen que renderiza una sola; modelos distintos dibujan unos pocos píxeles de forma ligeramente distinta, tesela a tesela (medido: ninguna costura visible). Si una tarjeta falla, sus teselas pasan a las demás. Las imágenes que el reparto no admite (algunos métodos de coloreado y fórmulas) usan la tarjeta de esta ventana.
# The export's progress.
dlg-export-preparing = Preparando (construyendo la referencia)…
dlg-export-saving = Guardando…
dlg-export-rendering = Renderizando…
# $time: the time since the export started, as 8.3s or 1m 04.0s.
dlg-export-elapsed = Transcurrido: { $time }
dlg-export-cancel-hover = Cerrar sin exportar
# A button, after a save icon.
dlg-export-save-as = Guardar como…
dlg-export-save-as-hover = Elegir el nombre y la ubicación del archivo
dlg-export-export = Exportar
dlg-export-export-hover = Renderizar y guardar en la carpeta de arriba (con nombre automático)

## Notices worded at startup (main.rs)

# A notice shown once after starting, when the saved session comes from a newer version of the app. $saved and $handled: version numbers of the session file's format (1, 2…).
app-state-newer = Esta sesión la guardó una versión más reciente de Fractadyne (estado v{ $saved }; esta compilación admite v{ $handled }). Puede que algunos ajustes no se apliquen, y al guardar se reescribirá en el formato de esta compilación.
# The title of the notice that says the accelerated build's arithmetic libraries are found again (its text comes from elsewhere).
app-notice-mpfr-restored-title = Aritmética acelerada restablecida
# After Fractadyne restarted itself because the graphics driver reset the GPU.
app-toast-gpu-recovered = Recuperado tras un reinicio del dispositivo gráfico: se ha vuelto a abrir en la vista inicial, porque la ubicación anterior era demasiado exigente para renderizarla.

## Short notices (toasts)

# When the autopilot reached a view that could not be rendered and put the last good view back.
app-toast-autopilot-view-restored = Piloto automático detenido: la vista quedó inutilizable y se restauró la anterior (registrado)
# $error: the system's reason, as it gives it (often in English).
app-toast-gradients-save-failed = No se pudieron guardar los degradados: { $error }
# $error: the reason, as the program gives it (often in English).
app-toast-gradients-serialize-failed = No se pudieron serializar los degradados: { $error }
# $name: the name the user gave the gradient.
app-toast-gradient-saved = Degradado «{ $name }» guardado.
# $error: the reason, as the program gives it (often in English).
app-toast-bookmarks-serialize-failed = No se pudieron serializar los marcadores: { $error }
# $error: the system's reason, as it gives it (often in English).
app-toast-bookmarks-save-failed = No se pudieron guardar los marcadores: { $error }
# Jumping to a named point given by exact expressions. $name: the point's name (in English).
app-toast-coordinates-failed = { $name }: no se pudieron evaluar las coordenadas
# $zoom: the magnification, already formatted (1,234.5 or 1.2e148).
app-toast-random-location = Ubicación aleatoria · { $zoom }×
# Importing a Kalles Fraktaler .kfr location. $error: the reason (in English).
app-toast-import-failed = Error al importar: { $error }
# $zoom: the magnification, already formatted.
app-toast-location-loaded = Ubicación cargada @ { $zoom }×

## Go to location: why a jump could not be made (one line per field that failed)

# "Re" is the dialog's real-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-re = Re: { $error }
# "Im" is the dialog's imaginary-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-im = Im: { $error }
# The imaginary field was given a complex value. $error: the evaluator's reason (in English).
app-goto-error-im-real = Im: { $error } — escribe la parte imaginaria como un número real, sin la i
# "Zoom" is the dialog's magnification field.
app-goto-error-zoom-empty = Zoom: introduce un aumento, p. ej. 1e50 o 250000
# $zoom: what was typed, in quotes.
app-goto-error-zoom = Zoom: no se puede leer { $zoom } — introduce un aumento positivo, como 1e50 o 250000
app-goto-error-invalid = Entrada no válida: revisa las coordenadas y el zoom.

## Finding a minibrot or Misiurewicz point (the M key and the Go to dialog)

app-feature-needs-holomorphic = El buscador de minibrots necesita una familia holomorfa (Mandelbrot / Multibrot).
app-feature-not-in-julia = El buscador de minibrots busca en el conjunto de Mandelbrot: desactiva el modo Julia para usarlo.
app-feature-busy = Ya se está buscando un elemento: de uno en uno.
app-feature-searching = Buscando el minibrot más cercano…
app-feature-mandelbrot-only = La búsqueda de elementos es solo para Mandelbrot.
app-feature-no-pair = No se encontró ningún punto de Misiurewicz dentro del presupuesto de búsqueda: puede que el punto esté a más profundidad de la que alcanza el buscador en esta vista, o que el centro no esté cerca de una espiral o de un punto de ramificación.
# The name of what was found, used as $feature in the messages below. $period: the minibrot's period (a whole number).
app-feature-minibrot = minibrot de período { $period }
# $preperiod and $period: the point's two whole numbers (k, p). $octaves: how often the view repeats when zooming in (2 decimals); $twist: the turn per repeat in degrees.
app-feature-misiurewicz-lambda = Misiurewicz ({ $preperiod },{ $period }) — se repite cada { $octaves } octavas, giro de { $twist }°
# $preperiod and $period: the point's two whole numbers (k, p).
app-feature-misiurewicz = Misiurewicz ({ $preperiod },{ $period })
# Added after the messages below when the fixed iteration count is too low for the new depth. $have: the iterations set now; $typical: what the depth typically needs.
app-feature-iter-note = ⚠ Las iteraciones están fijadas en { $have }; esta profundidad suele necesitar ~{ $typical }: auméntalas (o activa las iteraciones automáticas) si la vista se renderiza plana.
# $feature: what was found (see above). $zoom and $asked: magnifications, already formatted; $octaves: the solver's depth limit.
app-feature-zoomed-capped = Acercado al { $feature } — { $zoom }×. El cálculo está limitado a { $octaves } octavas, así que se detuvo ahí en lugar de en { $asked }×.
# $feature: what was found. $zoom: the magnification, already formatted. $seconds: how long the search took (1 decimal).
app-feature-zoomed = Acercado al { $feature } — { $zoom }× ({ $seconds }s)
# $feature: what was found. $seconds: how long the search took (1 decimal).
app-feature-snapped = Ajustado al centro del { $feature } ({ $seconds }s)
app-feature-no-minibrot = No se encontró ningún centro de minibrot cerca de la vista: acércate más a uno.
# $feature: the point asked for, e.g. Misiurewicz (3,1). $widths: how many view-widths away, already formatted. $octaves: how far to zoom out.
app-feature-just-outside = Se encontró un punto { $feature } justo fuera de esta vista, a { $widths } anchos de vista. Aléjate unas { $octaves } octavas para traerlo a la pantalla, o fija un preperíodo mayor para buscar un elemento a esta escala.
# $feature: the point asked for. $widths: how many view-widths away, already formatted (may be 2^187). $octaves: how far to zoom out.
app-feature-far = Se encontró un punto { $feature }, pero está a { $widths } anchos de vista: ese elemento es de una escala mucho MÁS GRUESA que esta vista. ALÉJATE unas { $octaves } octavas para alcanzarlo, o prueba un preperíodo mayor para buscar un elemento a esta escala.
# $feature: the point asked for. $residual: a small number in scientific notation.
app-feature-not-preperiodic = { $feature } no encaja con la órbita de aquí (residuo { $residual }): vacía ambos campos para detectar el par a partir de la vista.
# $feature: the point asked for. "k/p" are the dialog's two fields.
app-feature-not-converged = Ningún punto { $feature } convergió cerca de la vista: acércate más o prueba otros k/p.
app-feature-bad-request = El preperíodo y el período deben ser ambos positivos.

## Share location dialog and location files

app-location-empty = Nada que cargar (o el texto es demasiado grande).
app-location-not-fractadyne = No es una ubicación de Fractadyne.
# $zoom: the magnification, already formatted. $note: what in the text was ignored or repaired (in English).
app-share-loaded-note = Cargada @ { $zoom }× — { $note }.
# $error: the system's reason (often in English).
app-share-save-failed = Error al guardar: { $error }
app-share-saved = Guardado.
app-share-saved-rendering-thumb = Guardado. Renderizando la miniatura…
app-share-saved-no-thumb = Guardado, sin miniatura (no se pudo renderizar ninguna para esta vista).
app-share-saved-with-thumb = Guardado, con miniatura.
# $error: the system's reason (often in English).
app-share-saved-thumb-failed = Guardado, pero no se pudo añadir la miniatura: { $error }
app-share-loaded-into-box = Cargada en el cuadro: revísala y pulsa Aplicar.
# Reading a file failed (a location or a palette). $error: the system's reason (often in English).
app-read-failed = Error de lectura: { $error }
app-share-too-large = Archivo demasiado grande (¿no es una ubicación .fdn?).

## File-dialog filter names (the extensions are added by the code)

app-filter-kfr = Ubicación de Kalles Fraktaler
app-filter-fdn = Ubicación de Fractadyne
app-filter-map = Paleta de Fractint / KF
app-filter-ugr = Degradado de Ultra Fractal
app-filter-ase = Adobe swatch exchange
app-filter-ggr = Degradado de GIMP
app-filter-all = Todos los archivos

## Importing and saving palette files (shown under the gradient editor)

# The reason a file was refused before reading it.
app-gradient-map-too-large = mucho más grande de lo que puede ser una paleta .map
app-gradient-ugr-too-large = mucho más grande de lo que puede ser un archivo de paleta .ugr
app-gradient-ase-too-large = mucho más grande de lo que puede ser una lista de muestras .ase
app-gradient-ggr-too-large = mucho más grande de lo que puede ser un degradado .ggr
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported-vga = Se importaron { $count } entradas de { $file }: una tabla VGA de 6 bits (el blanco es 252, no 255), conservada tal cual para que coincida con la salida del propio Fractint.
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported = Se importaron { $count } entradas de { $file }, como bandas duras.
# $error: the reader's reason (in English).
app-gradient-map-invalid = No es una paleta .map: { $error }
# $name: the gradient's name in the file; $file: the file's name.
app-gradient-ugr-imported = Se importó «{ $name }» de { $file }.
# $file: the file's name; $count: how many gradients it holds.
app-gradient-ugr-pick = { $file } contiene { $count } degradados: elige uno abajo.
# $error: the reader's reason (in English).
app-gradient-ugr-invalid = No es un archivo de paleta .ugr: { $error }
# $used: the swatches kept; $total: the swatches in the file.
app-gradient-ase-imported-some = Se importaron { $used } de { $total } muestras, espaciadas uniformemente.
# $count: the swatches in the file.
app-gradient-ase-imported = Se importaron { $count } muestras, espaciadas uniformemente y mezcladas.
# $error: the reader's reason (in English).
app-gradient-ase-invalid = No es una lista de muestras .ase: { $error }
app-gradient-nothing-to-save = Aún no hay ningún degradado personalizado que guardar.
# $file: the saved file's name.
app-gradient-ggr-saved = «{ $file }» guardado: puntos medios, curvas de mezcla y espacios de color, todo intacto.
# $file: the saved file's name; $count: the segments whose curve could not be kept.
app-gradient-ggr-saved-lossy = «{ $file }» guardado, pero hay { $count } segmento(s) con una curva Bézier, que el formato .ggr no puede expresar. Se escribieron como mezclas rectas; la copia de tu biblioteca conserva las curvas.
# $path: the file; $error: the system's reason (often in English).
app-gradient-write-failed = No se pudo escribir { $path }: { $error }
# $name: the gradient's name; $count: its segments.
app-gradient-ggr-imported = Se importó «{ $name }»: { $count } segmentos, con sus puntos medios, curvas de mezcla y espacios de color.
# $error: the reader's reason (in English).
app-gradient-ggr-invalid = No es un degradado .ggr: { $error }

## The eyedropper's banner across the top of the screen while picking a colour

# $hex: the colour under the pointer, as #rrggbb.
app-eyedropper-banner-color = Eligiendo un color  ·  { $hex }  ·  haz clic para tomarlo, Esc para cancelar
app-eyedropper-banner = Eligiendo un color  ·  haz clic para tomarlo, Esc para cancelar

## The gradient editor window

app-gradient-title = Editor de degradados
# Before the two view buttons, Bar and Ring.
app-gradient-view = Vista
app-gradient-view-bar = Barra
app-gradient-view-bar-hover = Una tira recta: lo mejor para colocar paradas con precisión
app-gradient-view-ring = Anillo
app-gradient-view-ring-hover = Un anillo, porque la paleta es CÍCLICA: la unión de arriba es la costura que el renderizador cruza en cada vuelta, y un degradado que se ve bien en una barra puede tener ahí un borde brusco.
app-gradient-add-lane-hover = Haz clic en cualquier punto de esta línea para añadir ahí una parada. El + añade una en el hueco más ancho.
app-gradient-strip-hover = Arrastra un tirador para mover su parada · arrastra la propia tira para GIRAR todo el degradado, que da la vuelta en la costura · doble clic para añadir una parada, clic derecho en un tirador para quitarla · las flechas mueven ligeramente la selección (Shift para un ajuste fino).
app-gradient-remove-selected-hover = Quitar la parada seleccionada
app-gradient-ring-hover = Arrastra el propio anillo para GIRAR todo el degradado: lo que pasa la costura reaparece por el otro lado · arrastra un tirador para mover su parada · haz clic en la pista exterior para añadir ahí una parada, o en ⊕ en el centro para añadir una en el hueco más ancho · ⊖ quita la parada seleccionada. La marca hueca de arriba es la costura, donde una paleta cíclica da la vuelta; se queda fija porque ES la unión, y los colores giran bajo ella.
app-gradient-empty = Aún no hay degradado personalizado: «Copiar preajuste…», una importación o un degradado guardado de abajo empiezan uno, y sus paradas y curvas por segmento aparecen aquí.
# $count: the entries of an imported palette, more than the editor shows stop by stop.
app-gradient-too-many = { $count } entradas importadas: demasiadas para editarlas parada a parada. Usa la vista previa de arriba, o Copiar preajuste para empezar de nuevo.
# The selected segment's heading. $index: its number; $total: how many there are; $left and $right: where it starts and ends, 0.000–1.000.
app-gradient-segment-heading = Segmento { $index } de { $total }  ·  { $left } → { $right }
# A label before the list of blend curves.
app-gradient-curve = Curva
app-gradient-curve-hover = Con qué rapidez avanza el color por este segmento. «Curvada» se moldea con el punto medio: con el punto medio en 0,50 es exactamente lineal.
# The blend curves (GIMP's kinds), in a narrow list: keep them short.
app-gradient-blend-linear = Lineal
app-gradient-blend-curved = Curvada
app-gradient-blend-sine = Seno
app-gradient-blend-sphere-increasing = Esfera ↑
app-gradient-blend-sphere-decreasing = Esfera ↓
app-gradient-blend-bezier = Bézier
# Hover on Bézier in the list of blend curves.
app-gradient-bezier-exact-hover = Dos puntos de control que puedes arrastrar en ambas direcciones. Esta curva ya es cúbica, así que convertirla no cambia nada.
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-visible-hover = Dos puntos de control que puedes arrastrar en ambas direcciones. Convertirla APROXIMA la curva actual: esta, con un margen de { $error }, que se nota claramente.
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-barely-hover = Dos puntos de control que puedes arrastrar en ambas direcciones. Convertirla APROXIMA la curva actual: esta, con un margen de { $error }, que apenas se nota.
# A label before the list of colour spaces (RGB, HSV).
app-gradient-space = Espacio
# A label before the Bézier presets beside it.
app-gradient-ease = Suavizado
app-gradient-ease-hover = Los preajustes escriben los dos puntos de control; para cualquier otra cosa, arrástralos sobre la curva. Una Bézier no tiene punto medio: sus manejadores indican dónde llega a la mitad.
# Bézier presets, as in CSS easing: short buttons.
app-gradient-ease-linear = Lineal
app-gradient-ease-in = Entrada
app-gradient-ease-out = Salida
app-gradient-ease-in-out = Entrada-salida
app-gradient-midpoint = Punto medio
app-gradient-midpoint-hover = Dónde llega la mezcla a la mitad, como fracción de este segmento. 0,50 es el centro.
# A button that puts the midpoint back in the middle of the segment.
app-gradient-centre = Centrar
app-gradient-centre-hover = Volver a 0,50
app-gradient-no-hue = ⚠ un extremo de este segmento no tiene tono
app-gradient-no-hue-hover = Un extremo es negro, blanco o gris, así que el barrido empieza en el rojo y recorre toda la rueda de color. Es válido y a veces deseado, pero es la razón de que el centro pueda tener un color que no contiene ninguno de los extremos. La tira bajo la curva muestra lo que hace.
app-gradient-segment-start-hover = Color al PRINCIPIO de este segmento
app-gradient-segment-end-hover = Color al FINAL de este segmento
app-gradient-bytes-hover = Rojo, verde y azul de 0 a 255
app-gradient-floats-hover = El mismo color de 0 a 1, que es lo que guarda el renderizador
app-gradient-canvas-bezier-hover = La curva de mezcla del segmento seleccionado. Arrastra cualquiera de los dos puntos de control (en x Y en y) para cambiar su forma. Las líneas tenues son 0 y 1; una curva puede salirse de ellas, y lo que se recorta es el color, no la curva.
app-gradient-canvas-hover = La curva de mezcla del segmento seleccionado: con qué rapidez avanza el color de la parada izquierda a la derecha. Arrastra el anillo para mover el punto medio, o cambia la curva a Bézier para tener un punto de control que se mueve en ambas direcciones.
# The selected stop's heading. $index: its number; $total: how many there are.
app-gradient-stop-heading = Parada { $index } de { $total }
app-gradient-stop-color-hover = El color en esta parada: haz clic en la muestra para elegirlo, o escribe al lado un valor hexadecimal
app-gradient-stop-bytes-hover = Rojo, verde y azul de 0 a 255: los bytes con los que se escribe este color
# The eyedropper button.
app-gradient-pick = Elegir
app-gradient-pick-hover = Cuentagotas: toma un color de cualquier parte de la pantalla. Mueve el puntero hasta el color que quieras y haz clic; Esc cancela.
# Written before the stop's position (0.000–1.000) in its number field: a short abbreviation of "position".
app-gradient-stop-pos = pos
app-gradient-nudge-hover = Mover ligeramente esta parada (mantén Shift para un paso más fino)
app-gradient-remove = Quitar
app-gradient-remove-hover = Quitar esta parada (o haz clic en el ⊖ que tiene debajo)
app-gradient-add-stop = Añadir parada
app-gradient-add-stop-hover = Dividir el segmento más ancho (o haz clic en la línea que hay sobre el degradado)
# A menu of the built-in palettes, to start from a copy of one.
app-gradient-copy-preset = Copiar preajuste…
app-gradient-paste = Pegar…
app-gradient-paste-hover = Importar una paleta a partir de colores hexadecimales o de tríos RGB de 0 a 255
app-gradient-save-ggr = Guardar .ggr…
app-gradient-save-ggr-hover = Escribir este degradado en un archivo .ggr de GIMP, legible por GIMP, Krita e Inkscape, y por este editor
# A menu of the palette file formats (.map, .ugr, .ggr, .ase).
app-gradient-import = Importar
app-gradient-import-map-hover = Cargar un archivo de paleta .map de Fractint / Kalles Fraktaler (líneas R G B, una por entrada)
app-gradient-import-ugr-hover = Cargar un archivo de degradados .ugr de Ultra Fractal (un .ugr suele contener muchos degradados: eliges uno)
app-gradient-import-ggr-hover = Cargar un degradado .ggr de GIMP, con sus puntos medios, curvas de mezcla y espacios de color intactos
app-gradient-import-ase-hover = Cargar una lista de muestras de Adobe (.ase). Las muestras no tienen posición, así que llegan espaciadas uniformemente y mezcladas.
# The gradient's name, for the library of saved gradients.
app-gradient-name = Nombre
# Shown greyed in the empty name field.
app-gradient-name-hint = mi degradado
# The save button, when a gradient of that name is already saved.
app-gradient-update = Actualizar
app-gradient-save = Guardar
app-gradient-save-hover = Guardar este degradado en tu biblioteca, con sus curvas y espacios de color intactos
app-gradient-save-needs-name-hover = Primero ponle un nombre al degradado
# A menu of the saved gradients. $count: how many there are.
app-gradient-saved-menu = Guardados ({ $count })
app-gradient-nothing-saved = Aún no hay nada guardado
app-gradient-delete-saved-hover = Eliminar este degradado guardado
# $count: the gradient's segments.
app-gradient-rich = Degradado enriquecido: { $count } segmentos llevan puntos medios, curvas de mezcla o espacios de color, más de lo que admite una simple lista de paradas.
app-gradient-convert = Convertir en paradas editables
app-gradient-convert-hover = Sustituirlo por una simple lista de paradas. Esto DESCARTA los puntos medios, las curvas de mezcla y los barridos de tono que una lista de paradas no puede contener.
# Above the list of gradients in an Ultra Fractal file. $count: how many; $file: the file's name.
app-gradient-ugr-list = { $count } degradados en { $file }
app-gradient-close-list = Cerrar lista
app-gradient-seamless = Sin costura (el final coincide con el principio)
app-gradient-seamless-hover = Una paleta es cíclica, así que la posición 1 y la posición 0 quedan juntas en pantalla. Con esto activado, el color de la última parada se mantiene igual al de la primera; la vista de anillo muestra la unión arriba.
app-gradient-ends-differ = ⚠ los extremos difieren
app-gradient-ends-differ-hover = El color al final del degradado no es el del principio, así que una paleta cíclica muestra un borde brusco donde da la vuelta.
app-gradient-hard-bands = Bandas duras (sin mezcla entre entradas)
app-gradient-hard-bands-hover = Semántica de Fractint / .map: cada entrada es un color plano. Desactivado, se mezclan en un degradado suave.
app-gradient-paste-intro = Pega colores hexadecimales (#ff8800) o tríos de 0 a 255 (255 136 0), separados por comas, espacios o saltos de línea:
# Applies the pasted colours.
app-gradient-apply = Aplicar
# Empties the paste box.
app-gradient-clear = Vaciar
# $used: the colours kept; $total: the colours pasted; $limit: the most that are kept.
app-gradient-paste-imported-some = Se importaron { $used } de { $total } colores, muestreados uniformemente a lo largo de tu lista (el límite es { $limit }).
# $count: the colours pasted.
app-gradient-paste-imported = Se importaron { $count } colores.
# $error: why the pasted text could not be read (in English).
app-gradient-paste-unreadable = No se pudo leer: { $error }
# A hint at the bottom of the editor. $count: the palette's entries.
app-gradient-bands-hint = { $count } bandas · tomadas en el orden del archivo; las posiciones se ignoran mientras las bandas estén activadas.
# A hint at the bottom of the editor, in the ring view. $count: the gradient's stops.
app-gradient-ring-hint = { $count } paradas · haz clic en la pista exterior para añadir una ahí · ⊕ añade en el hueco más ancho, ⊖ quita la parada seleccionada · arrastra un tirador para moverlo · la marca de arriba es la costura
# A hint at the bottom of the editor, in the bar view. $count: the gradient's stops.
app-gradient-bar-hint = { $count } paradas · haz clic en la línea sobre el degradado para añadir una ahí · arrastra un tirador para moverlo · ⊖ o clic derecho para quitar
app-gradient-cancel = Cancelar
app-gradient-cancel-hover = Descartar todos los cambios hechos desde que se abrió esta ventana, y cerrarla
app-gradient-ok = Aceptar
app-gradient-ok-hover = Conservar estos cambios y cerrar el editor

## Shared

# The button that closes the Help window and the "Faster deep zoom" window.
app-close = Cerrar

## The "Faster deep zoom" window (Help menu): the optional accelerated build

app-accel-title = Zoom profundo más rápido
app-accel-active-heading = Estás usando la compilación acelerada
app-accel-active-body = Las órbitas de referencia del zoom profundo se calculan con MPFR/GMP, que en ese paso es entre 2,5 y 6,4 veces más rápido que la compilación estándar: la pausa antes de que una vista profunda empiece a resolverse.
# $status: which arithmetic library is in use and its version (in English).
app-accel-in-use = Aritmética en uso: { $status }
app-accel-fallback-heading = Compilación acelerada: se usa la aritmética integrada
app-accel-fallback-body = Esta es la compilación acelerada (MPFR), pero no se encontraron sus bibliotecas matemáticas, así que usa la aritmética integrada (astro-float). Las imágenes son idénticas; solo es más lenta la pausa antes de que se resuelva una vista profunda.
# Followed by the folder the program runs from, on its own line.
app-accel-fallback-dlls = Para usar la vía más rápida, coloca libmpfr-6.dll y libgmp-10.dll (y libgcc_s_seh-1.dll, libwinpthread-1.dll) junto a fractadyne.exe y reinicia:
app-accel-warning-hidden = El aviso de inicio sobre esto está oculto.
app-accel-show-again = Volver a mostrarlo
app-accel-offer-heading = Una compilación opcional es entre 2,5 y 6,4 veces más rápida en profundidad
app-accel-offer-body = El zoom profundo dedica gran parte de su tiempo a calcular órbitas de referencia en la CPU: la pausa antes de que una vista profunda empiece a resolverse. Una compilación opcional lo hace con las bibliotecas MPFR/GMP, lo que es entre 2,5 y 6,4 veces más rápido, y más cuanto más profundo vas. Todo lo demás es igual.
app-accel-offer-identical = Las imágenes son IDÉNTICAS BYTE A BYTE: son las mismas matemáticas calculadas con una biblioteca más rápida, comprobadas con todas las fórmulas de fractales y con todo el corpus de comparación de zoom profundo.
app-accel-offer-shared = Tus ajustes, tu sesión guardada y tus ubicaciones se comparten entre las dos, así que puedes cambiar libremente y no hace falta importar nada.
# Shown on Windows.
app-accel-separate-windows = Es una descarga aparte porque las bibliotecas que usa no se pueden compilar con el compilador que usa la compilación estándar para Windows, y tienen una licencia distinta (GNU LGPL v3) de la de Fractadyne.
# Shown on other systems.
app-accel-separate = Es una descarga aparte porque las bibliotecas que usa tienen una licencia distinta (GNU LGPL v3) de la de Fractadyne.
app-accel-download = Descargar para esta versión
app-accel-all-releases = Todas las versiones
app-accel-all-releases-hover = Si el enlace directo da un error 404, esta versión aún no tiene compilación acelerada: elige aquí la más cercana.
# Shown on Windows.
app-accel-install-windows = Descomprímela y ejecuta fractadyne.exe desde esa carpeta, manteniendo los archivos .dll a su lado.
# Shown on Linux.
app-accel-install-linux = Descomprímela y ejecuta ./fractadyne desde esa carpeta. Usa GMP y MPFR del sistema (libgmp10 / libmpfr6), que casi todos los escritorios ya tienen; consulta README-ACCELERATED.txt en el paquete.
# Shown on other systems.
app-accel-install-other = Aún no se compila ningún paquete acelerado para esta plataforma; se puede compilar desde el código fuente con --features fractadyne-core/rug.

## The Help window's frame (its sections are under help-)

app-help-title = Ayuda de Fractadyne

## Checking for updates

app-toast-up-to-date = Tienes la última versión.
# $error: why the check failed (in English).
app-toast-update-failed = Error al buscar actualizaciones: { $error }
# The window that offers a newer version.
app-update-title = Actualización disponible
# $version: the new version (0.3.0-beta.20). $channel: the token "stable" or "beta", in English; a translation can choose its words by it ({ $channel -> [beta] … *[other] … }).
app-update-version =
    { $channel ->
        [beta] Fractadyne { $version } (beta) está disponible
       *[other] Fractadyne { $version } (estable) está disponible
    }
# $current: the version running now. $track: the update track the user follows (its name in the language in use).
app-update-running = Estás usando { $current }  ·  { $track }
app-update-download-intro = Descarga la última versión desde GitHub:
app-update-download = Descargar desde GitHub
app-update-later = Recordármelo más tarde

## The second graphics card (shown in its window, after "In use:" or "Could not start:")

# The second device is the same card as the window's. $name: the card's name.
app-worker-same-card = la misma tarjeta, { $name }
# Why the second device could not start.
app-worker-thread-stopped = el hilo del dispositivo se detuvo

## Export status (shown in the Export image window)

app-export-gpu-unavailable = GPU no disponible
app-export-reference-aborted = Error al exportar: se interrumpió la construcción de la referencia.

## The performance panel (View ▸ Performance panel)
## Its row labels sit in a monospace column about 14 characters wide: keep them short (a longer label pushes its value to the right).

app-perf-fps = FPS
app-perf-fps-hover = Fotogramas presentados por segundo, a partir del intervalo suavizado entre fotogramas. Mientras una vista se asienta, muestra en su lugar el progreso del refinado: una vista que se está asentando no funciona al ritmo que daría a entender un número aquí.
# What the FPS row shows while a view is settling. $done and $total: tiles; $eta: the time left, as m:ss.
app-perf-refining-tiles = refinando { $done }/{ $total } (~{ $eta })
app-perf-refining-arming = refinando (preparando)
# $percent: how much is done, with one decimal.
app-perf-refining-percent = refinando { $percent }%
app-perf-building-reference = construyendo referencia
app-perf-idle = inactivo
app-perf-frame = fotograma
app-perf-frame-hover = Tiempo real suavizado entre fotogramas, no la suma de las filas de abajo. Incluye la espera para presentar, así que en una vista asentada e inactiva refleja la cadencia de repintado y no ningún trabajo.
app-perf-cpu = cpu
app-perf-cpu-hover = Tiempo del hilo principal dentro de una actualización: preparación de parámetros y gestión de la caché de referencias. Esta es la fila que se dispara cuando se reconstruye una referencia, y un pico aquí es el tirón que notas.
app-perf-gpu-idle = gpu/inactiva
app-perf-gpu-idle-hover = Tiempo de fotograma menos el tiempo de CPU del hilo principal. Es trabajo de la GPU mientras la vista se renderiza, pero en una vista asentada es casi todo espera inactiva: el nombre dice ambas cosas porque el panel no puede distinguirlas solo con esta resta.
app-perf-timing = medición
# How frame cost is measured, the timing row's value. TIMESTAMP_QUERY is a name.
app-perf-timing-starved = reloj real (faltan marcas de tiempo)
app-perf-timing-gpu = marcas de tiempo de GPU
app-perf-timing-no-query = reloj real (sin TIMESTAMP_QUERY)
app-perf-timing-hover = Cómo se mide el coste de un fotograma. Las marcas de tiempo de la GPU son la fuente precisa; el recurso al reloj real entra en juego cuando dejan de llegar mediciones, y un presupuesto calculado con el reloj real ha dimensionado mal repetidamente los fotogramas profundos: interpreta el rojo aquí como «los márgenes de seguridad son estimaciones en este momento».
app-perf-mode = modo
app-perf-mode-hover = Vía aritmética con la que se renderizó el último fotograma: directa (sin órbita de referencia), perturbación df32 o floatexp, la vía de rango extendido que se usa a partir de unos 1e28x, y la única en la que funciona BLA.
app-perf-eff-iter = iter efect.
app-perf-eff-iter-hover = Iteraciones que usó realmente el último fotograma, tras el escalado por profundidad y cualquier límite aplicado durante el movimiento; no el ajuste base de Calidad, que es solo el punto de partida.
app-perf-precision = precisión
app-perf-precision-hover = Bits de mantisa con los que se calcula la órbita de referencia. Crece con la profundidad; 64 bits es el mínimo para zonas poco profundas, no un límite que estés alcanzando.
app-perf-orbit-len = long. órbita
# $cap: the most orbit samples this GPU holds, already formatted, or the text of app-perf-uncapped.
app-perf-orbit-len-hover = Muestras de la órbita de referencia. Esta GPU admite como máximo { $cap }; acercarse a esa cifra es el muro de profundidad práctico: más allá, la vista en vivo no puede resolver ubicaciones más profundas que necesitan una órbita más larga.
app-perf-uncapped = (sin límite)
app-perf-sa-skip = salto SA
app-perf-sa-skip-hover = Iteraciones que se saltó la aproximación por series antes de empezar el trabajo por píxel. Más es más rápido y no cambia nada de la imagen; la fila no aparece cuando no se saltó ninguna.
app-perf-aa = aa
app-perf-aa-hover = Supermuestreo: cada píxel se renderiza a partir de este número de muestras por eje, así que 2x cuesta cuatro veces el trabajo.
app-perf-dual = doble
app-perf-dual-hover = Si la vista dividida Mandelbrot/Julia está activa. Cuando lo está, cada coste de este panel abarca AMBOS paneles.
app-perf-zoom = zoom
app-perf-zoom-hover = Aumento respecto a la vista completa. El modo aritmético de arriba cambia hacia 1e4x y de nuevo hacia 1e28x.
app-perf-iterate = iterar
# $target and $lethal: times in milliseconds, already formatted.
app-perf-iterate-hover = Tiempo de GPU medido del último despacho de iteración. El regulador apunta a { $target } ms; las lecturas sostenidas cerca de { $lethal } ms son la banda en la que esta clase de hardware ha perdido el dispositivo.
app-perf-steps = pasos/s
app-perf-steps-hover = Pasos de iteración por segundo en la GPU, en miles de millones. Las vistas con mucho interior van más lentas por paso de lo que sugiere el recuento por sí solo, así que esto es una lectura de rendimiento, no una puntuación.
app-perf-budget = presupuesto
# After the budget's value while it is still converging; the hover below quotes it.
app-perf-budget-settling = (asentándose)
app-perf-budget-hover = Presupuesto adaptativo de pasos por despacho, la protección del regulador contra un fotograma tan largo que el controlador gráfico reinicie la GPU. «asentándose» significa que aún está convergiendo a partir de lecturas escasas y todavía no es un techo fiable.
app-perf-rss = rss
# $mb and $peak: memory in megabytes, already formatted.
app-perf-rss-value = { $mb } MB (máx. { $peak })
app-perf-rss-hover = Memoria residente del proceso, y el máximo que ha alcanzado en esta sesión. Las construcciones de referencias profundas han superado los 2 GB en uso real sin que nada más en pantalla lo indicara; ámbar por encima de tres cuartos de la RAM del sistema.
app-perf-gpu-est = gpu estim.
app-perf-gpu-est-hover = Memoria residente en la GPU estimada: órbitas de referencia, árboles BLA, texturas de iteración, el ping-pong del estado de bloques mientras se ejecuta un refinado, y la instantánea de la puerta de presentación. Se calcula a partir de los tamaños de asignación conocidos; la cifra real del controlador gráfico no se puede leer de forma portable.
app-perf-julia-re = julia c.re
app-perf-julia-re-hover = Parte real del parámetro c que renderiza el panel de Julia: el punto sobre el que está el cursor, salvo que esté fijado.
app-perf-julia-im = julia c.im
app-perf-julia-im-hover = Parte imaginaria del parámetro c que renderiza el panel de Julia.
app-perf-julia-c = julia c
# The julia c row's value when the parameter is held fixed.
app-perf-pinned = fijado
app-perf-pinned-hover = El parámetro se mantiene en un punto fijo, así que mover el cursor ya no cambia el panel de Julia.
app-perf-c-panel = c/panel
app-perf-c-panel-hover = Anchura del espacio de c que abarca todo el panel de Mandelbrot. Cuando es muy inferior a un píxel de Julia, pasar el cursor cambia c pero el Julia parece igual: es lo esperado con zoom profundo, no un bloqueo.
app-perf-ref-recompute = recálculo ref
app-perf-ref-recompute-hover = Duración del recálculo de órbita de referencia más reciente. Es la pausa antes de que una vista profunda empiece a resolverse, y el paso que acelera la compilación acelerada opcional.
app-perf-recompute-rate = recálculos/s
app-perf-recompute-rate-hover = Órbitas de referencia INSTALADAS por segundo: aquellas cuyo resultado se usó de verdad. Compárala con la fila de abajo: se diferencian en el trabajo que se tiró.
app-perf-builds-rate = constr. ref/s
app-perf-builds-rate-hover = Construcciones de referencia LANZADAS por segundo, incluidas aquellas cuyo resultado se descarta: una ranura de anticipación a la que la inmersión nunca llega, o una extensión rechazada. Cuestan CPU completa y no aparecen en ningún otro sitio: una vez, un bucle de precarga llegó a lanzar unas 400 construcciones por segundo durante seis minutos mientras recálculos/s marcaba 2. Lo normal es una sola cifra; una gran diferencia entre estas dos filas significa que se está tirando trabajo.
app-perf-recompute-total = recálculos tot
app-perf-recompute-total-hover = Recálculos de referencia desde el inicio. Útil como tasa en un intervalo conocido; un número que sube mientras nada se mueve es la señal que vale la pena investigar.

## Help: the contents list (Help & reference…, F1)

# Each section's name in the contents list (about 165 px wide: keep them short), and its heading.
help-section-overview = Introducción
help-section-navigation = Navegación
help-section-expressions = Expresiones de coordenadas
help-section-options = Coloreado y opciones
help-section-fractals = Fractales
help-section-how-it-works = Cómo funciona
help-section-command-line = Línea de comandos
help-section-shortcuts = Atajos de teclado
help-section-hardware = Hardware recomendado
# Credits to the prior work Fractadyne builds on.
help-section-acknowledgments = Agradecimientos
# The third-party license notices.
help-section-licenses = Licencias
help-section-about = Acerca de

## Help ▸ Overview

help-overview-intro = Fractadyne es un explorador de fractales nativo pensado para el zoom ultraprofundo y la velocidad. Dibuja fractales de «tiempo de escape»: imágenes creadas repitiendo una sola fórmula sencilla en cada píxel.
help-overview-what-is = ¿Qué es un fractal de tiempo de escape?
help-overview-what-is-text = Para cada píxel, el programa aplica una fórmula como z → z² + c una y otra vez, empezando desde cero. Si el valor se mantiene pequeño para siempre, el píxel pertenece al conjunto y se dibuja oscuro. Si acaba creciendo sin límite («escapa»), el píxel está fuera del conjunto, y su color indica cuántos pasos hicieron falta. La frontera infinitamente detallada entre «se mantiene» y «escapa» es el fractal.
help-overview-can-do = Qué puedes hacer
help-overview-can-pan = Desplazarte y hacer zoom prácticamente sin límite (la posición es exacta a cualquier profundidad).
help-overview-can-families = Cambiar entre 25 familias de fractales integradas (diez, y cinco de ellas también con potencias mayores), tus propias fórmulas, Life y los sistemas L, y ver cualquier familia de tiempo de escape, salvo Newton, como conjunto de Julia.
help-overview-can-recolor = Cambiar los colores con degradados predefinidos o personalizados y varios métodos de coloreado.
help-overview-can-lighting = Añadir iluminación de relieve 3D y contornos luminosos en la frontera.
help-overview-can-minibrots = Ajustarte a minibrots, guardar lugares en marcadores y exportar imágenes de alta resolución.
help-overview-can-autopilot = Dejar que el piloto automático de zoom se sumerja solo hacia el detalle.
help-overview-can-share = Compartir cualquier lugar como un pequeño fragmento de texto «.fdn» o como archivo.
help-overview-can-tours = Ejecutar recorridos con guion y una prueba de rendimiento del hardware.
help-overview-first-steps = Primeros pasos
# "Seahorse Valley" is a place in the Navigate menu: use its name as that menu shows it. A and F1 are keys.
help-overview-first-steps-text = Abre el menú Navegar y elige «Seahorse Valley»; luego gira la rueda del ratón para acercarte. Arrastra para desplazarte. O simplemente pulsa A y deja que el piloto automático se sumerja por ti. Pulsa F1 en cualquier momento para volver a esta ayuda.

## Help ▸ Coordinate expressions
## The expressions, numbers and function names here are typed as shown in every language:
## keep them exactly as written. Menu items in quotation marks are the menus' own labels.

help-expr-intro = Allí donde Fractadyne acepta una coordenada, puedes escribir una expresión en lugar de un decimal: los campos Re / Im de Navegar → «Ir a la ubicación…» (y cada campo de su modo polar), las líneas center_re / center_im de una ubicación .fdn, los re / im de un guion de recorrido y --center en la línea de comandos. La expresión se evalúa con tantos dígitos como necesite el zoom, y el cuadro Ir a la ubicación la conserva con la vista; así, un punto que introduzcas como 1/3, o como -0.5 + 0.25*cos(pi/4), se mantiene exactamente sobre ese punto por mucho que te acerques, mientras que un decimal se desviaría en cuanto se le acabaran los dígitos.
help-expr-examples = Ejemplos
help-expr-ex-third = Un tercio exacto. Aparecen más dígitos a medida que te acercas.
help-expr-ex-neck = El cuello de Seahorse Valley: la raíz del bulbo de período 2.
help-expr-ex-circle = Un punto de la circunferencia de radio 1/4 alrededor de -0.5 (el modo polar compone esto por ti).
help-expr-ex-bulb = Re de la raíz del bulbo de período 3; su Im es sin(2*pi/3)/2 - sin(4*pi/3)/4. Navegar → «Puntos exactos (expresiones)» los trae incorporados.
help-expr-ex-complex = Un valor complejo completo escrito en el campo Re rellena tanto Re como Im.
help-expr-ex-combine = Las funciones y las constantes se combinan libremente.
help-expr-ex-powers = Potencias: 2^-20 es 1/1048576. -2^2 es -4: el signo menos se aplica después de la potencia.
help-expr-ex-root = -2, la raíz cúbica real. Las raíces pares de un número negativo se rechazan; sqrt(-1) da i.
help-expr-ex-scientific = La notación científica también funciona dentro de las expresiones.
help-expr-numbers = Números y operadores
# The decimal point in an expression is '.' in every language: keep that rule as it is.
help-expr-numbers-decimals = Decimales con un exponente opcional: 0.25, -1.5e-30, .5. Solo dígitos: sin espacios ni comas de agrupación dentro de un número, y el separador decimal es siempre «.».
help-expr-numbers-imaginary = Añade el sufijo i para un número imaginario (0.25i), o usa i sola: 3 + 4*i.
# Starts with the operators themselves, two spaces apart.
help-expr-numbers-operators = +  -  *  /  y ^ (potencia; se evalúa de derecha a izquierda, así que 2^3^2 = 2^9). Los paréntesis agrupan. No hay multiplicación implícita: escribe 2*pi, no 2pi.
help-expr-numbers-case = Los nombres no distinguen mayúsculas de minúsculas: PI, Pi y pi son la misma constante.
help-expr-constants = Constantes
help-expr-functions = Funciones
help-expr-functions-angles = Los ángulos están en radianes. Para grados, multiplica por pi/180 (cos(45*pi/180)), o cambia el cuadro Ir a la ubicación a su modo polar y elige la unidad grados.
help-expr-refused = Qué se rechaza, y por qué
help-expr-refused-fractional = Un número negativo elevado a una potencia fraccionaria, como (-2)^(1/2): el resultado es complejo y hay varias raíces entre las que elegir. Usa root(x, n) o cbrt(x) para una raíz real, o sqrt(x) para la raíz compleja principal.
help-expr-refused-complex = Un argumento complejo para una función de valor real (sin(i), ln(1+i)). Solo sqrt y abs aceptan un argumento complejo; los operadores aritméticos y las potencias enteras funcionan con valores complejos.
help-expr-refused-domain = Errores de dominio: ln o log de un número que no es positivo, asin o acos fuera de -1…1, 0 elevado a una potencia negativa, división por cero.
help-expr-refused-bounds = Argumentos de sin, cos, tan, exp o ^ más allá de ±2^32, y anidamientos de más de 32 niveles: una coordenada pegada es una entrada no fiable, y esos casos tardarían un tiempo sin límite.
help-expr-refused-grammar = Ni variables, ni funciones definidas por el usuario, ni comparaciones, ni condicionales. La gramática se mantiene deliberadamente pequeña para que una ubicación guardada signifique lo mismo en todas las versiones.
help-expr-no-parse = Si no se puede interpretar
help-expr-no-parse-text = El mensaje indica el campo, dice qué se encontró y en qué carácter y, para los despistes habituales, qué escribir en su lugar: un signo menos tipográfico (−) pegado desde un documento, × o ÷, una coma decimal, un * que falta entre un número y un nombre, un paréntesis sin cerrar o una función escrita sin sus paréntesis. Nunca se lee nada a medias: si alguna parte del texto no se entiende, se rechaza la coordenada entera en lugar de acortarla en silencio.

## Help ▸ Navigation
## Menu paths (Navigate → "Go to location…") and quoted items name the interface's own
## menus and controls: use their translations. Key names (Ctrl, Shift, Space, Esc) stay.

# A heading: the mouse controls (also in Shortcuts).
help-mouse = Ratón
# The left column of a controls table: a gesture or key. Keep key names (Shift, Space, Ctrl) as they are.
help-key-left-drag = Arrastrar con el botón izquierdo
help-nav-left-drag = Desplazar la vista.
help-key-mouse-wheel = Rueda del ratón
help-nav-wheel = Acercar/alejar hacia el cursor; gírala más rápido para hacer más zoom por cada muesca. Con Ajustar el zoom a los elementos activado, al acercarte cerca de un minibrot el zoom se ve atraído hacia él.
help-key-shift-drag = Shift+arrastrar
help-nav-shift-drag = Zoom de recuadro: arrastra un rectángulo y el zoom lo amplía hasta llenar la vista.
# $icon: the click-to-zoom toolbar button's symbol.
help-key-click-zoom = Clic para zoom ({ $icon })
help-nav-click-zoom = Activa el botón de diana de la barra de herramientas (vista única): entonces un clic izquierdo se sumerge en el punto según un factor fijado, y el clic derecho retrocede. Arrastrar sigue desplazando; elige el factor (2–100×) en Navegación, en el panel derecho. Backspace deshace un clic.
help-nav-continuous = Zoom continuo y vista inicial
help-key-hold-space = Mantener Space
help-nav-hold-space = Acercar suavemente, anclado en el cursor.
help-key-hold-shift-space = Mantener Shift+Space
help-nav-hold-shift-space = Alejar suavemente.
help-nav-continuous-text = El botón de alejar a la vista inicial anima un suave vuelo de regreso a la vista completa. «Velocidad de zoom», en el panel derecho, fija el ritmo del zoom continuo.
help-nav-autopilot = Piloto automático
help-nav-autopilot-text = Pulsa A (o Herramientas → «Zoom automático (piloto automático)») para una inmersión sin manos: elige un punto del borde del fractal donde el detalle es más denso, prefiriendo el centro de la vista, y se desliza hacia él mientras lo lleva suavemente al centro de la pantalla. Mantiene ese objetivo mientras siga siendo interesante y gira con suavidad hacia una estructura más rica cuando aparece alguna. Cualquier acción de navegación, o Esc, lo detiene. «Prioridad del zoom automático», en el panel derecho (o en Herramientas), elige qué pone por delante: Velocidad hace zoom al ritmo fijado y refresca la pantalla en cuanto hay listo un fotograma con detalle; Calidad muestra solo fotogramas a resolución completa totalmente resueltos y frena el zoom lo necesario para que la imagen nunca se amplíe mucho antes de que llegue la siguiente. En ambos casos, la pantalla nunca muestra un fotograma en blanco o de un solo color: un refresco que aún no ha resuelto ningún detalle queda oculto tras el anterior. «Objetivo del zoom automático» elige hacia dónde apunta: Detalle (el borde más rico de la vista, que se vuelve a elegir sobre la marcha) o Punto de Misiurewicz: busca el punto de Misiurewicz más cercano, lo resuelve hasta la profundidad del límite de inmersión mientras se mantiene quieto y luego se sumerge exactamente en él; la estructura alrededor de un punto así se repite cada pocas octavas para siempre, así que esa inmersión nunca llega a un callejón sin salida (solo en el conjunto de Mandelbrot).
help-nav-history = Historial y movimientos precisos
help-nav-undo = Deshacer la vista anterior.
help-nav-redo = Rehacer.
help-nav-goto = Navegar → «Ir a la ubicación…» te permite leer, escribir, pegar o copiar el centro y el zoom exactos (precisión completa, cualquier profundidad). Los campos del centro aceptan más que simples números: fracciones como «1/3» y expresiones construidas con pi, tau, e, sqrt y sin/cos/tan, de modo que «-0.5 + 0.25*cos(pi/4)» es una coordenada válida. Una expresión se conserva con la vista y se vuelve a evaluar con la precisión que necesita cada profundidad, así que un punto introducido de esa manera se mantiene exacto por mucho que te acerques (un decimal redondeado acabaría por no hacerlo). Un modo polar (desplazamiento + r ∠ θ, en grados / radianes / vueltas) compone una de estas expresiones por ti. La lista completa de funciones y constantes, con ejemplos, está en la sección «Expresiones de coordenadas» de esta ayuda (el botón ? del cuadro la abre). Navegar → Marcadores guarda y recupera ubicaciones.
help-nav-exact-points = Navegar → «Puntos exactos (expresiones)» salta a puntos definidos de forma exacta y no mediante un decimal redondeado: las raíces de los bulbos de la cardioide principal, donde se une cada brote de período q, escritas como expresiones cos/sin. Como la expresión viaja con la vista, puedes acercarte a uno sin que llegue a desviarse del punto.
help-nav-share = Navegar → «Compartir ubicación…» captura la vista completa (fractal, centro con precisión completa, zoom y coloreado) como un fragmento «.fdn» compacto que puedes Copiar, volver a pegar y Aplicar, o Guardar/Cargar como archivo, para reproducir un lugar con exactitud (o enviárselo a alguien).
help-nav-open = Archivo → «Abrir vista o ubicación…» abre directamente cualquiera de estos: un PNG/EXR exportado (se restaura la vista que lleva incrustada), un archivo de ubicación «.fdn» o un «.kfr» de Kalles Fraktaler.
help-nav-finding = Encontrar detalle
help-nav-minibrots = Los minibrots son copias diminutas del conjunto entero escondidas a lo largo de la frontera. Centra uno aproximadamente y pulsa M (o Herramientas → «Buscar minibrot + acercarse a él») para ajustarte por Newton exactamente a su centro, leer su período y hacer zoom directamente hasta su propia escala: a menudo, muchos órdenes de magnitud en un solo paso. Si ya estás a más profundidad que el minibrot, solo se mueve el centro.
help-nav-minimap = Ver → «Minimapa» muestra un pequeño mapa de todo el conjunto con un marcador «estás aquí» y la profundidad de zoom actual; haz clic en el mapa para saltar a una región.

## Help ▸ Coloring & options
## Names in quotation marks are the controls' own labels in the right-hand panel.

help-options-intro = Todo esto está en el panel derecho (y se conserva entre sesiones).
help-options-palette = Paleta
help-options-palette-text = Paleta elige un degradado predefinido, tu paleta Personalizada o un modo de dos colores. «Editar degradado…» abre un editor en el que cada parada de color tiene un color y una posición (0–1); añade paradas, copia un preajuste como punto de partida o importa un archivo de paleta.
help-options-paste = «Pegar…», en el editor de degradados, importa una paleta que hayas encontrado en otra parte. Acepta colores hexadecimales con o sin # inicial (#ff8800, ff8800, o el corto #f80) y tríos RGB simples de 0 a 255 como «255 136 0» (el formato que usan los archivos .map de Fractint y Kalles Fraktaler), separados por comas, espacios o saltos de línea. Se conservan hasta 256 colores; una lista más larga se muestrea uniformemente hasta dejarla en 256, conservando el primero y el último para que se mantenga la forma del degradado.
help-options-import-map = «Importar ▸ .map…», a su lado, carga un archivo de paleta .map de Fractint o Kalles Fraktaler. Son tablas de consulta indexadas por número de iteraciones, sin mezcla entre entradas, y los saltos de color bruscos que eso produce son la esencia del aspecto clásico de Fractint; por eso se importan así, con una casilla «Bandas duras» para mezclar esos mismos colores en un degradado suave. Los propios archivos de Fractint guardan valores de color de 6 bits multiplicados por cuatro, así que su blanco es 252 en lugar de 255; Fractadyne lo indica al importar uno y deja los valores exactamente como están escritos, porque las imágenes guardadas por el propio Fractint llevan ese mismo 252.
help-options-duotone = Bitono asigna el valor a una rampa suave de dos colores (Sombra → Luz). Binario (conjunto) es una vista plana de dos colores (un color sólido para los puntos dentro del conjunto y otro para los de fuera, sin degradado): la forma más clara de ver la silueta del conjunto.
help-options-cycle = Ciclo fija cuántas veces se repite el degradado a lo largo del rango de iteraciones (bandas más estrechas o más anchas). Desplazamiento gira todo el degradado.
help-options-normalize = «Normalizar colores profundos» ajusta la paleta al rango de valores de escape realmente presentes en la vista. Las vistas profundas abarcan cientos de miles de iteraciones y, sin esto, un ciclo fijo da miles de vueltas al degradado entre píxeles vecinos, lo que convierte el detalle correcto en moteado.
help-options-log-scale = «Escala de color logarítmica» reparte la paleta según el logaritmo del valor de escape en lugar de linealmente. Los recuentos de escape se amontonan hacia el extremo alto a medida que profundizas, así que un ajuste lineal gasta la mayor parte del degradado en una fina capa pegada a la frontera y deja el resto casi plano; la escala logarítmica lo despliega y mantiene los colores más estables a lo largo de un vídeo de zoom a medida que crece el rango.
help-options-animate = Animar hace ciclar los colores con el tiempo: Adelante, Atrás o Ping-pong mueven el desplazamiento; Degradados aleatorios sintetiza continuamente degradados armoniosos que se transforman con suavidad. Velocidad controla los ciclos (o las transformaciones) por segundo; «Barajar degradado» genera uno nuevo en el modo aleatorio.
help-options-method = Método de coloreado: cómo se convierten los datos en color
# The coloring methods, by the names the Coloring panel gives them, each with what it does.
help-method-smooth = Iteración suave
help-method-smooth-text = Bandas continuas clásicas según el tiempo de escape.
help-method-stripe = Promedio de franjas
help-method-stripe-text = Bandas fluidas a partir del ángulo de la órbita (control Densidad de franjas).
help-method-triangle = Desigualdad triangular
help-method-triangle-text = Textura fina según dónde cae cada paso entre unas cotas.
help-method-trap = Trampa de órbita
help-method-trap-text = Distancia de la órbita a una forma (punto/cruz/círculo); también colorea el interior.
help-method-distance = Estimación de distancia
help-method-distance-text = Sombrea según la cercanía a la frontera.
help-method-decomposition = Descomposición
help-method-decomposition-text = Celdas a partir del ángulo de escape final.
help-options-relief = Iluminación de relieve 3D
help-options-relief-text = Sombrea la superficie a partir de la pendiente de la frontera (la derivada) para lograr un aspecto en relieve e iluminado. Ángulo de la luz fija la dirección; Relieve, la intensidad (menor = más nítido); «Girar la luz» la anima. Solo familias holomorfas (Mandelbrot / Multibrot).
help-options-glow = Brillo de distancia
help-options-glow-text = Bandas de contorno brillantes que se densifican en filamentos luminosos cerca de la frontera. Brillo es la cantidad de mezcla, Ancho de bandas el espaciado, y «Animar el brillo» las hace fluir.
help-options-quality = Calidad e iteraciones
help-options-quality-text = Iteraciones es el número máximo de pasos antes de que un píxel se considere dentro del conjunto; «Autoescalar» lo aumenta automáticamente al hacer zoom (el detalle más profundo necesita más). Antialiasing supermuestrea las imágenes fijas (2×–8×) una vez que la vista se asienta, domando el fino «polvo» exterior.
help-options-other = Otros
help-options-other-text = Velocidad de zoom fija el ritmo del zoom continuo; el límite de FPS limita la frecuencia de fotogramas; «Ambos, enlazados» muestra un conjunto de Mandelbrot y su conjunto de Julia lado a lado (el cursor fija el parámetro de Julia en vivo).

## Help ▸ Fractals
## Fractal family names (Mandelbrot, Multibrot, Tricorn, Burning Ship, Celtic, Buffalo,
## Phoenix, Newton) are names: keep them as written, as the formulas. Formula syntax
## (z = z^3 - p1*z + c, sqr, |z|, if / elseif / endif, p1-p5, maxit) is typed as shown.

help-fractals-intro = Cada familia itera una fórmula con z empezando en 0 y c fijado por el píxel (tiempo de escape), salvo que se indique otra cosa. z = x + iy es un número complejo.
help-fractals-mandelbrot = z → z² + c. El original. El conjunto está formado por todos los c cuya órbita se mantiene acotada; es conexo, y su frontera es tan rugosa que tiene dimensión de Hausdorff 2.
help-fractals-multibrot = z → z^d + c para la potencia d = 3, 4, 5. Las potencias más altas añaden lóbulos: el conjunto tiene simetría rotacional de orden (d−1) (Multibrot 3 es de orden 2, 4 de orden 3, 5 de orden 4).
help-fractals-tricorn = z → conj(z)² + c, donde conj(x + iy) = x − iy. La conjugación lo hace antiholomorfo, lo que le da simetría de orden 3 y unas características «garras» curvas.
help-fractals-burning-ship = z → (|x| + i|y|)² + c: se toman valores absolutos antes de elevar al cuadrado (real = x²−y²+cx, imag = 2|xy|+cy). No analítico; los zooms profundos revelan estructuras con forma de barco (tradicionalmente se ve boca abajo).
help-fractals-celtic = Como Mandelbrot, pero con el valor absoluto de la parte real: real = |x²−y²| + cx, imag = 2xy + cy. Produce motivos con forma de corazón y de escudo.
help-fractals-buffalo = Valor absoluto de ambas partes de z²: real = |x²−y²| + cx, imag = |2xy| + cy; un cruce entre Celtic y Burning Ship.
# A heading: the families at powers above 2 (the Fractal menu's "Higher powers").
help-fractals-powers = Potencias superiores
# ("Burning Ship  3 4 5") is how the Fractal menu shows such a row: keep it as it is.
help-fractals-powers-text = Multibrot 6, 7 y 8 (z^d + c), y Burning Ship, Tricorn, Celtic y Buffalo con potencias 3, 4 y 5: los mismos pliegues con una potencia más alta. Burning Ship d toma los valores absolutos de las partes de z y luego la potencia; Tricorn d conjuga primero; Celtic d y Buffalo d pliegan la parte real (y, en Buffalo, la parte imaginaria) de z^d. El menú Fractal y la lista desplegable los muestran con una familia por fila («Burning Ship  3 4 5»). Cada uno admite zoom profundo igual que su hermano de potencia 2, y Multibrot 6, 7 y 8 también admiten la aproximación por series y el buscador de minibrots (M), como Multibrot 3–5. Multibrot 8 escapa con radio 128 en lugar de 256, porque con su siguiente paso saldría del rango de precisión simple.
help-fractals-phoenix = z → z² + c + p·z(n-1), donde z(n-1) es el iterado anterior y p es una constante (p = -0.5). El término de memoria produce filamentos como llamas.
help-fractals-newton = Método de Newton para las raíces de z³ − 1 = 0: z → z − (z³−1)/(3z²). En lugar de por el tiempo de escape, los píxeles se colorean según a cuál de las tres raíces cúbicas de la unidad converge la iteración (las cuencas de atracción) y con qué rapidez; las enmarañadas fronteras entre cuencas son el fractal.
help-fractals-julia = Modo Julia
help-fractals-julia-text = Para todas las familias salvo Newton puedes cambiar a un conjunto de Julia: en lugar de empezar z en 0 con c = píxel, fijas c (un parámetro) y dejas que z empiece en el píxel. El conjunto de Julia es la frontera entre los puntos de partida que se mantienen acotados y los que escapan, para ese c fijo. En «Ambos, enlazados», mover el cursor sobre el panel de Mandelbrot fija c en vivo.
help-fractals-deep = Compatibilidad con zoom profundo
help-fractals-deep-text = Mandelbrot, Multibrot 3/4/5 y Tricorn admiten zoom profundo por perturbación a profundidad extrema (floatexp). Burning Ship, Celtic y Buffalo no son analíticos (toman valores absolutos), así que usan una perturbación que tiene en cuenta el signo; ahora también funciona en el rango floatexp, con zoom profundo mucho más allá del antiguo límite de ~1e28× de df32 (el moteado ocasional cerca de los pliegues abs espera a la corrección de glitches con varias referencias). Phoenix usa una perturbación de dos términos y también admite zoom profundo en el rango floatexp (sin SA/BLA, así que es más pesado que Mandelbrot). Las potencias superiores tienen zoom profundo de la misma manera, con la perturbación de cada potencia siguiendo su propia cadena de cuadrados; Multibrot 3–8 también admiten aproximación por series y BLA, como Mandelbrot, y Tricorn, Burning Ship, Celtic y Buffalo admiten BLA en todas las potencias (sin series), que se encarga del interior de una vista profunda en lugar de iterarlo paso a paso. Solo Newton usa la vía directa, nítida hasta ~1e6×; las fórmulas personalizadas tienen zoom profundo salvo que la sección Fórmulas personalizadas de abajo diga lo contrario.
help-fractals-custom = Fórmulas personalizadas
help-fractals-custom-text = Fractal ▸ Fórmula personalizada… acepta el paso de una fórmula en expresiones al estilo de Fractint: z = z^3 - p1*z + c, o varias sentencias (t = sqr(z), luego z = t*t + c). z es el iterado, c el píxel (la constante de Julia en el modo Julia), p1-p5 los parámetros; |z| es el módulo al cuadrado, como en Fractint, y están disponibles las funciones habituales (sqr, abs, conj, real, imag, cabs, flip, recip, ident, exp, log, sqrt, sin, cos, tan, cotan, sinh, cosh, tanh, cotanh, asin, acos, atan, asinh, acosh, atanh, cosxx, y floor, ceil, trunc y round, que redondean cada parte). log es el logaritmo natural, como en Fractint y en análisis complejo: lo que muchos libros de texto escriben ln (no hay logaritmo en base 10). Una fórmula escrita con ln, cot, coth, Re o Im no se puede leer; la comprobación de sintaxis indica la función que hay que usar, un botón la reescribe, y al escribir ln se ofrece log. Cada paso admite zoom profundo por perturbación, con la misma aritmética de rango extendido a partir de 1e28x que los fractales integrados, salvo uno con una potencia cuyo exponente varía con z o c (como z^c): ese se renderiza por la vía directa, nítido hasta que un paso de c en precisión simple abarca un píxel, hacia 1e4x a 1e5x, y a menos profundidad lejos del origen. El cuadro de la fórmula indica cuál se aplica. log, sqrt y las potencias no enteras toman sus valores principales, que saltan al cruzar el semieje real negativo. La perturbación sigue el salto con exactitud, pero un píxel cuya órbita cruza donde la de la referencia no lo hace continúa a partir de entonces con una precisión aproximadamente simple, así que las vistas profundas cuyas órbitas atraviesan un corte muestran ruido ahí (medido: limpio a 1e5x en el eje real para sqrt(z^4 + c), con ruido para las fórmulas con log incluso a 1e4x). Cada paso se itera por completo (sin aproximación por series), así que una vista profunda con muchas iteraciones se renderiza más despacio que la de un fractal integrado. Cerca de su frontera, una fórmula con funciones tiene órbitas caóticas, y una órbita larga allí solo se sigue de forma aproximada en precisión simple: con 2.000 iteraciones, un tercio de los píxeles de sin(z) + c difieren de los de la órbita exacta; con 60, casi ninguno. Las vistas, las sesiones y las imágenes exportadas conservan el texto de la fórmula.
help-fractals-custom-sections = Las secciones de Fractint también funcionan. Las sentencias anteriores a un ':' son la sección de inicialización, que se ejecuta una vez por píxel; una comparación como última sentencia es la propia prueba de escape de la fórmula, que itera mientras se cumpla (|z| <= 100 && |z - 1| > 0.000001 detiene una órbita que escapa o que se estabiliza en 1); if (...), elseif (...), else y endif eligen sentencias; y una variable que se lee antes de asignarse en un paso tiene el valor del paso anterior (de la sección de inicialización, o 0). Las comparaciones comparan partes reales y dan 1 o 0, && y || las combinan, y una asignación también es un valor (a = b = pixel). pixel, p1-p5, pi y e se pueden asignar, y conservan su propio valor mientras no se les asigne otro; maxit es el número de iteraciones e ismand vale 1 (la forma Mandelbrot de la fórmula). Una fórmula con cualquiera de estos, o con redondeo, se renderiza por la vía directa, sin zoom profundo. Biblioteca de fórmulas > Importar también lee archivos .frm de Fractint. Cada entrada que se puede leer se convierte en una fórmula, y su fila indica qué cambió la lectura: fn1-fn4 toman los valores predeterminados de Fractint (sin, sqr, sinh, cosh); la variable c de Fractint se renombra como c_, ya que aquí c es el píxel; un nombre al que nada asigna valor vale 0, como en Fractint; los espacios dentro de un nombre se eliminan (end if es endif), igual que hace Fractint. El mensaje de la importación cuenta las entradas que no se pueden leer e indica el motivo más común: no se admiten las variables de pantalla y de vista (whitesq, scrnpix, scrnmax, center, magxmag, rotskew), lastsqr ni los números aleatorios. Los clásicos de Fractint de la Colección (Magnet, Nova, Spider, Manowar y otros) están escritos con estas secciones.
help-fractals-custom-editor = En el campo de la fórmula, los paréntesis llevan un color por nivel de anidamiento; el par junto al cursor se resalta, y uno sin pareja se muestra en rojo (los de un comentario no cuentan, ya que el analizador los ignora). Los nombres se completan mientras escribes: tras dos letras, una lista ofrece las funciones, los nombres y tus propias variables que empiezan así. Up y Down eligen, Tab o Enter toma uno (una función viene con su paréntesis de apertura), Esc cierra la lista, y Enter después de un nombre escrito completo sigue empezando una línea nueva.
help-fractals-custom-textbook = Libro de texto (el interruptor sobre la fórmula) la muestra compuesta tal como la compondría un libro de texto, y permite editarla ahí mismo. Escribe como en el texto: / convierte en fracción el término anterior, ^ crea un exponente (+, - o = escritos al final de un exponente salen de él), ( abre un paréntesis, o una llamada tras el nombre de una función, | crea barras, y una coma dentro de paréntesis, una constante compleja. Las flechas recorren la fórmula, Up y Down pasan entre un numerador y su denominador, Tab va a la siguiente casilla vacía; Backspace al principio de una fracción o de un par de paréntesis la selecciona, y una segunda pulsación la quita y conserva lo que contenía. Copiar, pegar y deshacer funcionan como en un campo de texto, los nombres se completan igual que allí, y el teclado escribe en la fórmula compuesta (□² eleva al cuadrado lo que hay antes del cursor, la tecla de una función envuelve la selección). Cada edición se escribe de vuelta en el texto, así que las dos vistas contienen siempre la misma fórmula; una línea que no hayas editado conserva su espaciado y su comentario. La notación nunca cambia lo que se calcula: |z| es el módulo al cuadrado y se muestra como |z|², cabs(z) se muestra como |z|, y exp(z) sigue siendo exp(z). Una línea que no se puede leer se muestra en rojo; haz clic en ella para editarla como texto. LaTeX copia la fórmula como código fuente LaTeX, tal como está compuesta, y mientras Libro de texto está activado la biblioteca de fórmulas también muestra sus fórmulas compuestas.
help-fractals-custom-numbers = Los números de una fórmula (0.1, 1/3, pi, los parámetros) se leen como valores de doble precisión, de unos 16 dígitos, y la órbita de referencia profunda itera exactamente con esos valores. Por eso, dos fórmulas iguales sobre el papel pueden diferir en lo profundo: 0.1*z multiplica por el double más cercano a 0.1, mientras que z/10 divide por un 10 exacto, y las dos imágenes se separan a partir de unos 1e15x (medido: un 5% de los píxeles a 1e15x, un tercio a 1e17x, todos a 1e20x). Ambas imágenes son correctas para su propia fórmula; escribe la que quieras compartir.
help-fractals-custom-library = Guardar en la biblioteca, en el cuadro de la fórmula, guarda una fórmula y sus parámetros con un nombre (guardar con un nombre que ya está en la biblioteca actualiza esa entrada). Fractal ▸ Biblioteca de fórmulas… las muestra para aplicarlas, editarlas, exportarlas o eliminarlas; Importar añade las fórmulas de un archivo de fórmulas sin sustituir ninguna: se omite una que la biblioteca ya tiene, y una nueva cuyo nombre ya está ocupado pasa a llamarse «nombre (2)». La biblioteca es formulas.toml, en la carpeta indicada en Archivos y datos (Acerca de), y es a su vez un archivo de fórmulas. Un fotograma clave de un recorrido lleva una fórmula como formula = "..." con formula_params = [[re, im], ...], y Herramientas > Recorrido desde la vista actual los escribe para una vista personalizada.
help-fractals-custom-collection = Con Fractadyne viene una colección de 45 fórmulas, escritas para él: potencias superiores y las variantes plegadas de las familias Burning Ship y Celtic, mezclas e híbridos de dos pasos, el conjunto de Mandelbrot visto a través de otra transformación (invertido, log-polar, a través del seno), fórmulas trascendentes y racionales, y conjuntos de Julia. Son los Ejemplos del cuadro y la pestaña Colección de la biblioteca de fórmulas, donde Copiar a las mías añade una a tus fórmulas. Cada una tiene una vista de partida, y Aplicar va allí: una fórmula abierta en la vista inicial a menudo no está donde está su imagen. Una fórmula que guardas mientras se muestra conserva de la misma manera la vista en la que estás, y Aplicar vuelve a ella.

## Help ▸ How it works

help-how-escape = Tiempo de escape y color suave
help-how-escape-text = Cada píxel itera la fórmula hasta que su magnitud supera un radio de escape o llega al límite de iteraciones. El recuento de pasos en bruto produce por sí solo bandas duras; añadir una parte fraccionaria derivada de la magnitud final da un color continuo, sin bandas.
help-how-precision = Posición con precisión arbitraria
help-how-precision-text = Los números ordinarios de 64 bits se quedan sin dígitos cerca de un zoom de 1e15×. Fractadyne guarda el centro de la vista con precisión arbitraria, con un número de dígitos que crece a medida que haces zoom, así que la ubicación nunca se degrada. La escala del píxel también se guarda con un exponente extendido, así que tampoco se atasca en el límite de ~1e308× de los 64 bits: la profundidad solo está limitada por la paciencia, no por ningún rango numérico fijo.
help-how-perturbation = Perturbación
help-how-perturbation-text = Iterar cada píxel con alta precisión sería demasiado lento. En su lugar, se itera un píxel de referencia con alta precisión en la CPU (la «órbita de referencia»), y todos los demás píxeles se calculan en la GPU como una diminuta diferencia δ respecto a él, en baja precisión rápida: δz → 2·Z·δz + δz² + δc.
help-how-floatexp = Profundidad extrema (floatexp)
help-how-floatexp-text = A partir de unos 1e28×, incluso esa diminuta diferencia queda por debajo del rango de 32 bits (subdesbordamiento), así que se guarda como una mantisa más un exponente entero aparte («floatexp»), lo que levanta el muro de profundidad de los 32 bits. El motor cambia automáticamente: matemáticas directas en zonas poco profundas, perturbación en zonas profundas y floatexp en las más profundas. La profundidad queda entonces limitada por la precisión de las coordenadas y el presupuesto de iteraciones, y no por un límite fijo: contrastado con Fraktaler-3 en un corpus de referencia de 39 ubicaciones, la más profunda a 5.63e18003× (las 38 filas no extremas son la comprobación rutinaria, hasta 6.13e1105×), con una batería interna de autoconsistencia de la precisión hasta 1e1000000×; el recorrido incluido llega a ~1.3e420×.
help-how-reference = Elección de la referencia y reanclaje
help-how-reference-text = La referencia se elige (puntuándola con alta precisión) de modo que su órbita permanezca dentro de la vista el mayor tiempo posible. Cuando la diferencia crece demasiado, se «reancla» de nuevo a la referencia para mantener la precisión.
help-how-series = Aproximación por series
help-how-series-text = Con zoom profundo, las primeras iteraciones apenas alejan δ de la referencia, así que se pueden calcular todas de una vez: δz se aproxima mediante un polinomio corto en δc (de orden 3), cuyos coeficientes se iteran a lo largo de la referencia. El renderizador salta hasta la última iteración en la que ese polinomio sigue siendo preciso para toda la vista y luego itera normalmente, ahorrándose los pasos saltados sin cambiar la imagen (se activa y desactiva en Controles ▸ Avanzado). Mandelbrot y Multibrot 3–8, rango más profundo; no con el coloreado de franjas / desigualdad triangular / trampa de órbita / descomposición (esos necesitan cada iteración). Donde el BLA está activo, ya incluye este salto inicial.
help-how-distance = Estimación de distancia e iluminación
help-how-distance-text = Seguir la derivada dz/dc da la distancia de cada píxel a la frontera. Eso alimenta la iluminación de relieve 3D, el brillo de distancia y el método de coloreado «distancia», todos válidos a cualquier profundidad de zoom.
help-how-safety = Antialiasing y seguridad
help-how-safety-text = Las imágenes fijas se supermuestrean (2–8× por eje) una vez que la vista se asienta. Un presupuesto de trabajo mantiene cada dibujado de la GPU dentro del límite del temporizador de vigilancia (watchdog) del controlador gráfico reduciendo la resolución (nunca el número de iteraciones) en ajustes extremos, para que las vistas profundas sigan detalladas en lugar de quedarse en blanco.

## Help ▸ Command line (the flags themselves stay English)

# The section's introduction. The flags listed below it are the command line itself, English in every language.
help-cli-intro = Fractadyne funciona sin interfaz gráfica para renderizar, exportar películas/recorridos, automatizar, hacer comprobaciones con imágenes patrón y medir el rendimiento. Sin opciones, abre la interfaz gráfica interactiva.

## Help ▸ Shortcuts

# What each mouse gesture and key does (the gestures are the Navigation section's).
help-shortcuts-pan = Desplazar
# A gesture: turning the mouse wheel.
help-key-wheel = Rueda
help-shortcuts-wheel = Zoom en el cursor
help-shortcuts-box-zoom = Zoom de recuadro (arrastra un rectángulo para hacer zoom hasta él)
# $icon: the click-to-zoom toolbar button's symbol. "tool on": while that button is on.
help-key-click-tool = Clic (herramienta { $icon } activada)
help-shortcuts-click = Sumergirse en el punto; el clic derecho retrocede
# A heading: the keyboard shortcuts.
help-shortcuts-keyboard = Teclado
help-shortcuts-space = Zoom continuo para acercar / alejar
help-shortcuts-undo = Deshacer vista
help-shortcuts-redo = Rehacer vista
help-shortcuts-minibrot = Buscar minibrot + acercarse a él (Newton-Raphson)
help-shortcuts-autopilot = Piloto automático de zoom (se sumerge hacia el detalle; cualquier acción lo detiene)
# off / strip / panel / full: the panel's sizes, as View ▸ Live render panel names them.
help-shortcuts-live-panel = Panel de renderizado en vivo: desactivado / tira / panel / completo (pasa el cursor sobre un instrumento para ver qué significa; su × lo cierra)
help-shortcuts-esc = Detener el piloto automático / un recorrido en reproducción, o salir de la pantalla completa
help-shortcuts-snapshot = Instantánea en la última carpeta de exportación: una captura de pantalla de la vista tal como se ve, o un renderizado completo con los ajustes de exportación; la primera pulsación pregunta cuál, y Archivo → Configuración → Instantánea lo cambia.
help-shortcuts-help = Abrir esta ayuda
# A heading: the shortcuts of the Game of Life, with the menu path to it.
help-shortcuts-life = Life (Fractal → Autómatas → Life)
help-shortcuts-life-run = Ejecutar / pausar el universo
help-shortcuts-life-step = Avanzar una generación / avanzar el salto (se fija en el panel de Life)
help-shortcuts-life-draw = Herramienta de dibujo activada / desactivada (el lápiz de la barra de herramientas)
# A gesture. "Draw on": while the Life draw tool is on.
help-key-click-drag-draw = Clic / arrastrar (dibujo activado)
help-shortcuts-life-paint = Invertir una celda / pintar: un trazo que empieza en una celda muerta dibuja, y uno que empieza en una celda viva borra. Un universo en marcha se pausa mientras dibujas y sigue cuando sueltas.
# $key: a keyboard shortcut (Ctrl+Z). "Draw on": while the Life draw tool is on.
help-key-when-drawing = { $key } (dibujo activado)
help-shortcuts-life-undo = Deshacer la última edición, relleno, borrado o cambio de regla

## Help ▸ Recommended hardware

help-hardware-intro = Fractadyne reparte su trabajo entre la GPU (iteración por píxel + coloreado) y la CPU (la órbita de referencia de precisión arbitraria en la que se basa el zoom profundo). Las dos importan, pero para cosas distintas.
help-hardware-gpu = GPU (lo más importante)
help-hardware-gpu-text = Cada píxel se itera y se colorea en la GPU, así que la velocidad de la GPU determina la frecuencia de fotogramas y el tiempo de exportación. Una GPU dedicada con un buen rendimiento en 32 bits (precisión simple) ofrece el zoom profundo más fluido. Las GPU integradas funcionan bien, solo que son más lentas y alcanzan antes el presupuesto de trabajo por fotograma (momento en el que el renderizador recorta la resolución, no el detalle). Requiere una API gráfica moderna: Vulkan, Direct3D 12, Metal u OpenGL mediante wgpu.
# The left column of a requirements table.
help-hardware-minimum = Mínimo
help-hardware-gpu-minimum = Cualquier GPU con Vulkan / DX12 / Metal / GL 3.3.
help-hardware-recommended = Recomendado
help-hardware-gpu-recommended = Una GPU dedicada (de los últimos ~5 años); >= 2 GB de VRAM.
help-hardware-big-exports = Para exportaciones grandes
help-hardware-gpu-big-exports = >= 4 GB de VRAM (por teselas, así que se degrada con suavidad).
# A heading: the processor.
help-hardware-cpu = CPU
help-hardware-cpu-text = La órbita de referencia de alta precisión (y las tablas BLA) se calculan en la CPU y en gran parte en un solo hilo, así que lo que más importa es el rendimiento por núcleo: determina lo rápido que se resuelve una vista profunda después de moverte. Los núcleos adicionales ayudan sobre todo a la codificación de exportaciones en segundo plano. Se necesita una CPU de 64 bits.
help-hardware-cpu-recommended = Una CPU moderna de 64 bits con buena velocidad por núcleo.
help-hardware-memory = Memoria
help-hardware-memory-text = Las referencias profundas crecen con la profundidad (muchas iteraciones con alta precisión); las exportaciones enormes guardan teselas en la RAM. Con 8 GB se explora a diario con holgura; 16 GB o más convienen para profundidades extremas y exportaciones muy grandes.
help-hardware-checking = Comprobar tu hardware
help-hardware-diagnostics = Ayuda → «Diagnóstico…» ejecuta en tu equipo las mismas pruebas que hacen los desarrolladores. «Ejecutar autoprueba» comprueba las matemáticas y el renderizado frente a resultados que se saben correctos en unos quince segundos; «Ejecutar prueba de interfaz» recorre la interfaz y la vista en vivo a varias profundidades de zoom, haciendo capturas de pantalla; «Ejecutar comprobación aritmética de la GPU» verifica la aritmética double-float del shader en cada backend gráfico que ofrece tu equipo, en unos segundos y sin ventana. Cada una se ejecuta como un proceso aparte, así que una prueba que trastorna el controlador gráfico hace caer a la prueba y no a tu sesión.
help-hardware-gpu-check = La comprobación aritmética de la GPU no tiene aprobado ni suspenso: registra cómo calculan tu GPU y tu controlador concretos, y en algunos equipos (todas las tarjetas NVIDIA probadas hasta ahora) informa de que el compilador de shaders descarta parte de la aritmética de precisión extendida. Es un hallazgo real que intentamos cartografiar entre fabricantes, no un fallo de tu equipo, así que, diga lo que diga, adjuntarlo a un informe ayuda de verdad.
help-hardware-report = Si algo se ve mal, ejecuta la autoprueba y usa «Adjuntar a un informe de problema…»: el informe llevará entonces un resultado validado por la máquina en tu propio hardware, que es mucho más útil que una simple descripción. Una advertencia que conviene conocer: las imágenes de referencia se generaron con una tarjeta gráfica concreta, y GPU distintas calculan legítimamente píxeles ligeramente distintos, así que en otro hardware las comparaciones de imágenes se comprueban con una tolerancia mayor y allí las pequeñas diferencias son de esperar, no fallos.
help-hardware-notes = Notas
help-hardware-note-watchdog = Una GPU capaz evita que el presupuesto de trabajo del watchdog entre en acción con ajustes extremos.
help-hardware-note-binary = La compilación es un único binario autónomo: no hace falta instalar ningún entorno de ejecución.
help-hardware-note-benchmark = Ejecuta la prueba de rendimiento integrada (Herramientas ▸ Prueba de rendimiento) para evaluar tu equipo.

## Help ▸ Acknowledgments & citations
## Names of people, programs, papers, books and websites stay as written, and so do the
## titles in quotation marks (the works' own titles); the prose around them translates.

help-ack-title = Agradecimientos y citas
# The dedication, in italics at the top of the section.
help-ack-dedication = Dedicado al Stone Soup Group, los voluntarios detrás de Fractint (desde 1988).
help-ack-dedication-text = Su código compartido libremente y su espíritu de exploración fractal abierta y colaborativa son la razón por la que tantos de nosotros nos enamoramos de este arte. Fractadyne está hecho en esa tradición.
help-ack-intro = Fractadyne se apoya en décadas de trabajo de la comunidad fractal. Técnicas y trabajos previos en los que se basa, verificados con las fuentes de abajo:
help-ack-deep-zoom = Algoritmos de zoom profundo
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-martin-title = Perturbación y aproximación por series — { $who }
help-ack-martin = El método de órbita de referencia + delta de baja precisión (δz → 2Z·δz + δz² + δc) y la serie de orden n que se salta las primeras iteraciones, presentados en SuperFractalThing y en su nota «SuperFractalThing Maths» (sft_maths.pdf, 2013).
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bla-title = Aproximación lineal bivariante (BLA) y reanclaje — { $who }
help-ack-bla = Saltarse tandas de iteraciones mediante aplicaciones lineales fusionadas, y reanclar a una sola referencia para evitar glitches; desarrollado en fractalforums.org (2021).
# A link's label. $who: the linked page's author.
help-ack-bla-link = BLA explicado — { $who }
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-glitch-title = Criterio de detección de glitches — { $who }
help-ack-glitch = La prueba |Z + z| << |Z| para saber cuándo un píxel necesita otra referencia: la base de la perturbación sin glitches (fractalforums).
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-diffabs-title = Perturbación no analítica (Burning Ship) — { $who }
help-ack-diffabs = El análisis por casos del valor absoluto, «diffabs», que hace funcionar la perturbación en las familias basadas en abs (Burning Ship / Celtic / Buffalo), en fractalforums.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mathr-title = Implementaciones de referencia y comprobaciones cruzadas — { $who }
help-ack-mathr = Fraktaler-3 y Kalles Fraktaler 2+ (creado originalmente por Karl Runmo), cuyas explicaciones guiaron este motor y con cuya salida EXR contrasta Fractadyne su renderizador.
# A heading: credits for the coloring methods.
help-ack-coloring = Coloreado
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-harkonen-title = Recuento de iteraciones suave y Stripe Average — { $who }
help-ack-harkonen = El coloreado continuo (normalizado) por tiempo de escape y el método Stripe Average (promedio de franjas), de «On Smooth Fractal Coloring Techniques» (tesis de máster, 2007), que también analiza los coloreados Triangle Inequality / branching-average.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mitchell-title = Triangle Inequality Average — { $who }
help-ack-mitchell = Promediar dónde cae cada z² + c dentro de las cotas de la desigualdad triangular, para lograr una textura exterior suave, como de llamas.
# A link's label: the person's tutorial pages. $who: the person's name.
help-ack-mitchell-link = { $who } — tutoriales
help-ack-distance = El coloreado y la iluminación por estimación de distancia usan el método clásico de distancia a la frontera dz/dc (véase Peitgen y Saupe, «The Science of Fractal Images», 1988).
help-ack-lsystems-life = Sistemas L y el Juego de la vida
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-lindenmayer-title = Sistemas L — { $who }
help-ack-lindenmayer = Reescritura paralela de cadenas, presentada en 1968 para modelar cómo crecen las plantas y las algas («Mathematical models for cellular interactions in development», Journal of Theoretical Biology).
help-ack-abop = El libro (1990) cuya interpretación con tortuga y cuya notación para sistemas estocásticos, sensibles al contexto y paramétricos sigue el lector de sistemas L; los sistemas integrados marcados como ABOP se han introducido a partir de sus figuras.
# A link's label: the book can be read free on that website. $site: the website's name.
help-ack-abop-link = El libro, gratis en { $site }
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bourke-title = Colección de sistemas L — { $who }
help-ack-bourke = Muchos de los sistemas integrados proceden de sus páginas sobre sistemas L, entre ellos las reglas de Chris Wallace para la curva de Sierpinski y el arbusto de Dietmar Saupe. Los archivos .l de Fractint, que se abren directamente, están en el formato del Stone Soup Group.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-conway-title = El Juego de la vida — { $who }
help-ack-conway = El autómata celular B3/S23 (1970), que llegó al mundo a través de la columna «Mathematical Games» de Martin Gardner en Scientific American (octubre de 1970).
# A credit's heading: the website, then who runs it. $site: the website's name.
help-ack-lifewiki-title = { $site } — la comunidad de ConwayLife.com
help-ack-lifewiki = La referencia para las notaciones de reglas que lee la aplicación (B/S, von Neumann, Generations, las letras de Alan Hensel para reglas no totalísticas, y MAP), para el formato de patrones RLE que comparte con Golly, y para las reglas y patrones integrados, cuyos descubridores registra.
help-ack-foundations = Fundamentos y herramientas
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mandelbrot-title = El conjunto de Mandelbrot — { $who }
help-ack-mandelbrot = El objeto en el corazón de todo.
help-ack-built-with = Hecho con Rust, egui/eframe (emilk) y wgpu (gfx-rs); precisión arbitraria mediante astro-float, flotantes de doble palabra mediante twofloat; PNG/OpenEXR, rfd, serde. Gracias a sus autores y mantenedores.
help-ack-errors = Cualquier error de atribución es responsabilidad del autor; las correcciones son bienvenidas en el gestor de incidencias del proyecto.

## Help ▸ Third-party licenses

help-licenses-title = Licencias de terceros
# $licenses: Fractadyne's license expression (MIT OR Apache-2.0), as written. $url: the web address of a crate's source. THIRD-PARTY-NOTICES.md is a file name, `option-ext` a crate's name, MPL-2.0 a license's.
help-licenses-intro = Fractadyne se distribuye como un binario enlazado estáticamente que incluye los crates de código abierto y las fuentes tipográficas incorporadas que se indican abajo. A continuación figuran los textos completos de las licencias (también en THIRD-PARTY-NOTICES.md, incluido con la descarga). El código propio de Fractadyne tiene doble licencia { $licenses }. `option-ext` es MPL-2.0: su código fuente sin modificar está en { $url }.
# A button, after a copy icon: copies every notice to the clipboard.
help-licenses-copy-all = Copiar todos los avisos

## Help ▸ About

help-about-tagline = Un explorador de fractales nativo hecho en Rust con wgpu.
# The arithmetic deep zoom has used. $ran: the libraries that have run so far, as names (or a note that none has yet); $built: those this build contains, with versions.
help-about-arithmetic = Aritmética de zoom profundo: { $ran } (esta compilación contiene: { $built }).
help-about-files = Archivos y datos
# Followed by the folder's path.
help-about-files-text = Tu sesión (vista actual, coloreado y preferencias), los marcadores y sus miniaturas, los degradados guardados y las fórmulas guardadas se almacenan en esta carpeta por usuario, y se vuelven a cargar en el próximo inicio:
# Tooltip of the copy button beside the folder's path.
help-about-copy-path = Copiar ruta
# $command: a command typed in a terminal (it stays English).
help-about-session = El archivo de sesión tiene versión; si lo escribió una versión de Fractadyne más reciente que la que puede leerlo por completo, la aplicación lo carga lo mejor que puede y te avisa. Para empezar completamente de cero, usa Archivo ▸ Restablecer el estado de la aplicación… (pregunta antes), o ejecuta `{ $command }` desde una terminal.
help-about-license = Licencia
# $licenses: the license expression (MIT OR Apache-2.0), which stays as written.
help-about-license-text = { $licenses }: úsalo bajo cualquiera de las dos licencias, a tu elección.
# A link to the source code, followed by ↗.
help-about-source = Código fuente en GitHub

## The live render panel ("blinkenlights": View ▸ Live render panel, key B)
## A front panel drawn over the view like a piece of 1970s test equipment. Its legends are engraved
## in capitals: write them in capitals here (the panel shows them exactly as written, it does not
## uppercase them). Numbers, units and readouts on its tubes and dials are not translated.

# Tooltip of the × button in the panel's top-right corner. The code adds the key after it: (B).
live-close-hover = Cerrar el panel de renderizado en vivo
# The legend engraved along the top plate, left of the maker's badge.
live-plate = RENDERIZADO EN VIVO · PANEL FRONTAL

## Live render panel: instrument titles
## Engraved above each instrument, and repeated as the title of its help card.

# Above the depth readout, whose tubes show the magnification as a power of two ("2^" stays).
live-title-depth = PROFUNDIDAD 2^
live-title-iterations = ITERACIONES
# Above the frame-time readout (milliseconds).
live-title-frame-ms = FOTOGRAMA MS
# Above the date-and-time readout.
live-title-clock = FECHA · HORA
# The walk dial: a radar screen showing a slow refresh rendered pass after pass, band after band.
live-title-walk = AVANCE · BANDAS
# A meter of how far the picture on screen lags the view.
live-title-age = EDAD DE LA IMAGEN
# What part of the picture this frame's GPU pass covered.
live-title-tiles = TESELAS
# A scope of the GPU's work per frame, in milliseconds.
live-title-scope = PASADAS GPU · MS
# The depth dial: depth, the reference orbits being built, and the arithmetic in use.
live-title-dial = PROFUNDIDAD · REFERENCIAS · ARITMÉTICA
# Iteration steps per second.
live-title-steps = PASOS / S
# The two GPUs (the window's and a second, worker GPU).
live-title-engines = DOS MOTORES
live-title-memory = MEMORIA
live-title-cores = NÚCLEOS · HILOS
live-title-pixels = FLUJO DE PÍXELES
# Help card titles only: the three GPU timer lamps (T0, T1, T2 stay as they are) and the row of lamps.
live-title-timers = TEMPORIZADORES GPU T0–T2
live-title-bank = BANCO DE LUCES

## Live render panel: the light bank's lamps
## Engraved under each lamp, and the title of its help card. A cell is narrow (about eight capital
## letters): a longer label breaks at a space onto two lines, and is drawn smaller if it still does
## not fit.

live-lamp-plan = PLAN
live-lamp-dispatch = DESPACHO
live-lamp-live = EN VIVO
live-lamp-held = RETENIDA
# The last picture reprojected (stretched) to fit the moving view.
live-lamp-reproj = REPROY
live-lamp-reading = LECTURA
# A pass of a pinned walk.
live-lamp-pin-pass = PASADA FIJA
live-lamp-split = DIVISIÓN
live-lamp-adopt = ADOPCIÓN
live-lamp-abandon = ABANDONO
# "REF": a reference orbit. Built for the view as it is (LIVE), ahead of it (AHEAD), or for a tour's next hold (HOLD).
live-lamp-ref-live = REF EN VIVO
live-lamp-ref-ahead = REF ADELANTE
live-lamp-ref-hold = REF PAUSA
# The series approximation.
live-lamp-series = SERIES
# The second (worker) GPU.
live-lamp-worker = AUXILIAR
live-lamp-tile = TESELA
# Supersampling (anti-aliasing); usually kept as SSAA.
live-lamp-ssaa = SSAA
live-lamp-palette = PALETA
# The arithmetic in use (direct, df32, floatexp).
live-lamp-mode = MODO

## Live render panel: the kinds of GPU pass
## In the GPU scope's legend (MOVING, PIN, WORKER, SETTLE, SSAA). SETTLE and SSAA also name a still
## view's walk on the walk dial.

# A moving frame's pass.
live-kind-moving = MOVIMIENTO
# A pass of a pinned walk.
live-kind-pin = FIJA
# The worker GPU's passes.
live-kind-worker = AUXILIAR
# A still view's settle tile or walk pass.
live-kind-settle = REPOSO
# A supersampling sample.
live-kind-ssaa = SSAA

## Live render panel: legends and readouts on the instruments

# On the walk dial during a split refresh. $done: passes done; $passes: passes in all.
live-walk-split = DIVISIÓN { $done }/{ $passes }
# On the picture-age meter's face, under its scale (0 to 2 octaves).
live-age-scale = OCTAVAS DE RETRASO
# The depth dial's band for direct (plain floating-point) arithmetic. The other bands, DF32 and FLOATEXP, are names and stay.
live-dial-direct = DIRECTA
# Under the two GPU gauges: GPU 0 draws the window; GPU 1 is the worker, or there is none.
live-engines-gpu-window = GPU 0 · VENTANA
live-engines-gpu-worker = GPU 1 · AUXILIAR
live-engines-no-worker = SIN AUXILIAR
# The share of displayed frames each GPU made. $window and $worker: percentages, 0–100.
live-engines-on-screen = EN PANTALLA { $window } : { $worker }
# The same readout before any frame was shown.
live-engines-on-screen-none = EN PANTALLA —
# Under the CPU core bars. $count: the machine's cores.
live-cores-count = { $count } NÚCLEOS
# Under the threads scope. $count: the app's busy threads.
live-cores-threads = HILOS { $count }

## Live render panel: the memory tanks' legend
## What the GPU memory holds, by kind. Very narrow: about ten letters.

# G-buffers (the picture's working textures).
live-memory-gbuf = G-BUF
# The held picture and the supersampling accumulator.
live-memory-hold = RET+SSAA
# The reference orbit and its BLA (bilinear approximation) table.
live-memory-orbit = ÓRBITA+BLA
# The worker GPU's copy of the reference.
live-memory-worker = REF AUXIL.

## Live render panel: the roles on the threads scope
## Tiny labels on a small scope: at most four letters each.

# The reference builders.
live-cores-role-ref = REF
# The scoring pool.
live-cores-role-pool = POOL
# The thread driving the worker GPU.
live-cores-role-worker = AUX
# Probes: short side renders.
live-cores-role-probe = SOND
# Image and tour exports.
live-cores-role-export = EXP

## Live render panel: the tiles instrument's captions
## The line under the picture's plane, then the tile size (left) and the tiles a frame (right).

# A walk's pass. $lo and $hi: the iteration band it rendered (numbers).
live-tiles-pin-band = PASADA FIJA · BANDA { $lo }–{ $hi }
live-tiles-settle-band = PASADA EN REPOSO · BANDA { $lo }–{ $hi }
live-tiles-moving-whole = FOTOGRAMA EN MOVIMIENTO · IMAGEN ENTERA
live-tiles-settled-whole = FOTOGRAMA EN REPOSO · IMAGEN ENTERA
# $pass: this pass; $passes: the passes the refresh is split into.
live-tiles-split-pass = REFRESCO DIVIDIDO · PASADA { $pass } DE { $passes }
# $grid: the grid's columns × rows, e.g. 12×8.
live-tiles-grid-done = REJILLA DE REPOSO { $grid } · HECHA
# $grid: the grid's columns × rows; $done and $total: tiles done, and in all.
live-tiles-grid-progress = REJILLA DE REPOSO { $grid } · { $done } DE { $total } HECHAS
# The same, while a tile too costly for one frame goes out in strips.
live-tiles-grid-strips = REJILLA DE REPOSO { $grid } · { $done } DE { $total } HECHAS · EN TIRAS
# $samples: supersampling samples folded so far; $lo and $hi: the iteration band the next one is walking.
live-tiles-ssaa-band = SSAA · { $samples } HECHAS · BANDA { $lo }–{ $hi }
# $sample: the supersampling sample.
live-tiles-ssaa-whole = MUESTRA SSAA { $sample } · FOTOGRAMA ENTERO
# No GPU pass this frame, and the panel has not seen a frame yet.
live-tiles-idle-starting = INICIANDO
# No GPU pass this frame: the last complete picture is held on screen.
live-tiles-idle-held = RETENIDA
# No GPU pass this frame: the last picture is stretched to fit the view.
live-tiles-idle-reprojected = REPROYECTADA
# No GPU pass this frame: the picture on screen is current.
live-tiles-idle-live = EN VIVO
# No GPU pass this frame, and nothing to show.
live-tiles-idle-none = INACTIVO
# The tile size: $res, a whole picture's resolution (2400×1300).
live-tiles-tile-res = TESELA { $res }
# The tile size: $px, a tile's side in pixels.
live-tiles-tile-px = TESELA { $px } PX
live-tiles-no-pass = SIN PASADA
# Tiles a frame. $count: how many (0 when none).
live-tiles-rate-count = { $count }/FOTOGRAMA
live-tiles-rate-one = 1 TESELA/FOTOGRAMA
# $share: the part of one tile sent each frame, e.g. 0.33.
live-tiles-rate-part = { $share } TESELA/FOTOGRAMA
live-tiles-rate-sample = 1 TESELA/MUESTRA
# In the plane's corner during a walk: the share of pixels still iterating. $percent: a percentage, e.g. 54 or 1.5.
live-tiles-running = EN CURSO { $percent }%

## Live render panel: help cards
## Shown when the pointer rests on an instrument or a lamp. Words in capitals (SETTLE, SSAA, ABANDON,
## HELD, REPROJ, ADOPT, REF AHEAD, ON SCREEN) are the panel's own legends: use the same words as
## their messages above.

# The heading of a card's list of what healthy looks like.
live-card-healthy = CORRECTO
# The heading of a card's list of what signals trouble.
live-card-trouble = PROBLEMAS

live-card-walk-what = Un refresco profundo demasiado costoso para un solo fotograma se fija y se recorre: se renderiza pasada tras pasada, y cada una deja que los píxeles iteren una octava más (256, 512, 1.024 … hasta lo pedido). El borde es ese recuento en escala logarítmica, empezando arriba; el haz es el progreso del avance; cada arco es una pasada, coloreada según su tiempo de GPU frente al objetivo de 20 ms. El punto blanco espera la lectura de la última pasada; el destello es la nueva imagen que aparece en pantalla.
live-card-walk-note = Una vista quieta demasiado costosa para una sola pasada se recorre del mismo modo, marcada REPOSO, y cada muestra de supermuestreo se vuelve a recorrer, marcada SSAA: cada una de sus pasadas puede tardar hasta 400 ms, y los arcos se colorean en función de eso. Un refresco dividido dibuja en cambio un anillo por pasada: todas las bandas, una parte de las teselas.
live-card-walk-good-sweep = Un barrido que da la vuelta en unos pocos fotogramas y termina en un destello.
live-card-walk-good-arcs = Arcos mayormente verdes y ámbar.
live-card-walk-good-still = En una vista profunda quieta: un barrido más lento por muestra, con arcos de verde a ámbar.
live-card-walk-bad-red = Arcos rojos: pasadas por encima del presupuesto, que se notan como tirones.
live-card-walk-bad-restart = Un barrido que se detiene a medias y vuelve a empezar: se abandonó el avance, y la imagen en pantalla sigue siendo antigua.
live-card-walk-bad-hollow = Arcos huecos: pasadas cuyo tiempo nunca volvió, así que la siguiente pasada se dimensiona a ciegas.

live-card-age-what = Cuánto se retrasa la imagen en pantalla respecto a la vista, en octavas de zoom (una octava es 2×). Mientras se renderiza un refresco, la imagen antigua se muestra estirada y envejece; cuando llega el refresco, la edad baja. La tira de abajo muestra los últimos segundos.
live-card-age-good-sawtooth = Un diente de sierra entre aproximadamente 0,3 y 0,8, con la aguja oscilando en torno a la marca de 0,5.
live-card-age-good-still = Cerca de 0 en cuanto te detienes.
live-card-age-bad-red = La aguja instalada por encima de 1,0, en la zona roja: la imagen tiene más de 2× de retraso y se ve borrosa o pixelada al hacer zoom.
live-card-age-bad-ramp = Una rampa que nunca baja: los refrescos no terminan (vigila ABANDONO).

live-card-tiles-what = Cómo cubre la imagen la pasada de GPU de este fotograma. Los fotogramas en movimiento y las pasadas de avance la cubren entera, y el rótulo indica la banda de iteración de una pasada de avance. Una pasada de avance itera solo los píxeles que aún no han escapado: el plano se llena hasta esa proporción (escala logarítmica, 0,01% en el mínimo) y la muestra en la esquina, así que se vacía a medida que el avance profundiza. Un refresco algo demasiado costoso para un solo fotograma se divide en hasta 8 pasadas, cada una un conjunto intercalado de teselas de 16 px. Una vista en reposo demasiado costosa para una sola pasada se convierte en una rejilla de teselas, una por fotograma desde el centro hacia fuera, y una tesela que sigue siendo demasiado costosa sale en tiras. Abajo se indican el tamaño de tesela y las teselas por fotograma.
live-card-tiles-good-whole = Pasadas enteras mientras haces zoom; refrescos divididos en unas pocas pasadas.
live-card-tiles-good-grid = Cuando te detienes: una rejilla de decenas de teselas que se llena en uno o dos segundos, y luego SSAA.
live-card-tiles-bad-small = Una rejilla de cientos de teselas pequeñas: el presupuesto de cada pasada es pequeño, y la imagen tardará mucho en definirse.
live-card-tiles-bad-strips = Muchas teselas saliendo en tiras: las teselas fueron más lentas de lo previsto al dimensionar la rejilla.
live-card-tiles-bad-splits = Divisiones atascadas en 8 pasadas: el refresco apenas cabe, y un poco más hondo recurre al avance, más lento.
live-card-tiles-bad-dark = A oscuras durante largos ratos mientras te mueves: ninguna pasada en absoluto.

live-card-scope-what = Una columna por fotograma, la más reciente a la derecha: el trabajo de GPU que ejecutó, apilado por tipo (fotograma en movimiento en verde, pasada fija de verde a rojo según el coste, tesela de reposo o pasada de avance de una vista quieta en crema, muestra de supermuestreo en lavanda, los fotogramas y muestras de la GPU auxiliar en azul), en milisegundos. Líneas discontinuas en 16,7 ms (60 fps), 33 y 50. Una marca hueca es una pasada cuyo tiempo nunca volvió.
live-card-scope-good-under = Columnas por debajo de la línea de 16,7 mientras te mueves.
live-card-scope-good-pin = Pasadas fijas de verde a ámbar.
live-card-scope-good-samples = Las muestras de supermuestreo de una vista quieta pueden tardar más: no se mueve nada.
live-card-scope-bad-over = Columnas por encima de 33 o 50 ms: tirones visibles.
live-card-scope-bad-seconds = Pasadas de segundos: el watchdog de la GPU puede reiniciar el dispositivo.
live-card-scope-bad-hollow = Muchas marcas huecas: el trabajo se dimensiona sin mediciones.

live-card-timers-what = Las tres ranuras de marcas de tiempo de la GPU. Una ranura se enciende cuando se cronometra una pasada y se atenúa cuando vuelve su lectura, dos o tres fotogramas después. T0 cronometra la pasada de cada fotograma; T1 y T2 cronometran pasadas fijas adicionales.
live-card-timers-good = T0 parpadeando siempre que hay trabajo; T1 y T2 parpadeando durante los avances.
live-card-timers-bad-stuck = Una lámpara encendida sin parar: su lectura no vuelve, y los costes quedan desfasados.
live-card-timers-bad-dark = Todas apagadas mientras la GPU está ocupada: no se mide nada.

live-card-dial-what = La aguja roja es la profundidad de la vista: su aumento como potencia de dos. Las bandas son la aritmética que usa la GPU allí: directa, df32 (dos floats por número) y floatexp (un float con un exponente amplio) a partir de 2^93. Los puntos son órbitas de referencia en construcción: verdes para la vista, ámbar por delante de ella, hacia donde se dirige el zoom, azules para la siguiente pausa de un recorrido. Un anillo es una instalación; una cruz roja es una construcción descartada porque la vista la adelantó.
live-card-dial-good-ahead = Puntos ámbar que aparecen por delante de la aguja y se convierten en anillo antes de que ella llegue.
live-card-dial-good-crosses = Pocas cruces rojas.
live-card-dial-bad-passing = La aguja pasando su última referencia instalada: los fotogramas esperan una, y se nota como una congelación.
live-card-dial-bad-crosses = Cruces rojas repetidas: el zoom va más rápido que los constructores, y se desperdicia CPU.
live-card-dial-bad-crossing = El cruce de 2^93 es un punto lento conocido: los primeros fotogramas floatexp cuestan más.

live-card-bank-what = Cada lámpara se enciende cuando ocurre su evento y se apaga en unos 150 ms, así que un renderizador ocupado parpadea y uno inactivo se queda a oscuras. De izquierda a derecha: el fotograma, el refresco, las referencias, y luego el auxiliar, el asentamiento, la paleta y la aritmética.
live-card-bank-note = Pasa el cursor sobre una lámpara para ver qué significa.

live-card-depth-what = El aumento como potencia de dos: 2^100 son unos 1,3 × 10^30.
live-card-depth-note = Más allá de 2^93 cada paso cuesta más (floatexp), y las referencias tardan más en construirse.

live-card-iter-what = Las iteraciones pedidas: hasta dónde puede iterar cada píxel antes de contar como dentro del conjunto. Sube con la profundidad.
live-card-iter-good = Subiendo de forma constante a medida que te sumerges.
live-card-iter-bad-low = Atascado en un valor bajo a mucha profundidad: las espirales y los minibrots se renderizan como interior plano.
live-card-iter-bad-high = Muy alto: cada refresco cuesta más, y los avances tardan más.

live-card-ms-what = El tiempo entre fotogramas mostrados.
live-card-ms-good = El intervalo de tu pantalla: 16,7 en una pantalla de 60 Hz.
live-card-ms-bad-stutter = Por encima de 33: tirones visibles.
live-card-ms-bad-spikes = Picos por encima de 50: enganches, normalmente una pasada costosa (consulta PASADAS GPU).

live-card-clock-what = La hora real en hora local, con una precisión de una décima de segundo.
live-card-clock-note = Marca cualquier captura de pantalla del panel con el momento en que se hizo, de modo que la imagen de un problema se pueda emparejar con el registro de la aplicación.

live-card-tach-what = Pasos de iteración por segundo. En blanco: los pasos pedidos (píxeles × iteraciones). En ámbar: los pasos que la GPU ejecutó realmente. La diferencia es BLA (aproximación bilineal) saltando muchas iteraciones de una vez; ×N es ese multiplicador. El cuentakilómetros acumula el total de la sesión.
live-card-tach-good = Una gran diferencia en profundidad: BLA haciendo su trabajo, de ×10 a ×1.000.
live-card-tach-bad-together = Agujas juntas en profundidad: BLA no puede saltar aquí (a menudo cerca del interior de un minibrot), así que los refrescos son lentos.
live-card-tach-bad-red = La aguja blanca en la zona roja: se pide más de lo que la GPU puede hacer en un fotograma.

live-card-engines-what = Lo ocupada que está cada GPU, con puntos que llevan sus fotogramas a la pantalla: el tiempo de GPU de sus pasadas durante el último segundo, incluidos los fotogramas en movimiento y las muestras de supermuestreo del auxiliar. EN PANTALLA es la proporción de los fotogramas mostrados que hizo cada una. Con una sola GPU, el lado del auxiliar sigue inactivo.
live-card-engines-good = Ambas ocupadas durante una inmersión, con el auxiliar aportando una parte de los fotogramas.
live-card-engines-bad-pinned = La GPU 0, la de la ventana, clavada cerca del 100%: es el cuello de botella, y los fotogramas dan tirones.
live-card-engines-bad-idle = Un auxiliar activado pero inactivo: no está entregando fotogramas.

live-card-memory-what = RAM: el conjunto de trabajo de la aplicación, con la línea roja como su pico. VRAM: lo que la aplicación ocupa en la GPU, calculado a partir de los tamaños que reserva, por tipo (G-buffers, texturas de retención y SSAA, búferes de órbita y BLA, la copia de la referencia del auxiliar), frente al total de la tarjeta.
live-card-memory-good = Plana mientras haces zoom; sube y vuelve a bajar a medida que van y vienen las referencias.
live-card-memory-bad-leak = RAM que sube y nunca baja: una fuga.
live-card-memory-bad-vram = VRAM cerca del tamaño de la tarjeta: las reservas pueden fallar, o el dispositivo puede perderse.

live-card-cores-what = Uso de CPU por núcleo, de verde a ámbar y a rojo, y un osciloscopio en el que cada hilo ocupado orbita en torno a su función: los constructores de referencias, el pool de puntuación, el auxiliar, los renderizados secundarios.
live-card-cores-good = Uno o dos núcleos ocupados mientras haces zoom; una breve ráfaga en todos los núcleos cuando se elige una referencia.
live-card-cores-bad-every = Todos los núcleos ocupados durante segundos: construcción de referencias a profundidad extrema.
live-card-cores-bad-own = El núcleo propio de la aplicación clavado en la zona roja: la propia ventana da tirones.

live-card-pixels-what = Píxeles calculados por segundo, en Mpx/s. Tenue: aún iterando en un avance. Brillante: terminados.
live-card-pixels-good = Decenas de Mpx/s mientras te mueves.
live-card-pixels-bad = Cerca de cero mientras la imagen está desfasada: la GPU está esperando algo, una referencia o una lectura.

## Live render panel: the lamps' help cards

live-card-lamp-plan-what = Se planificó un fotograma: se eligieron la resolución, las iteraciones pedidas y el presupuesto, y si hay que refrescar.
live-card-lamp-plan-good = Parpadea siempre que cambian la vista o los ajustes.
live-card-lamp-plan-bad = Apagada mientras haces zoom: el hilo de la interfaz está atascado.

live-card-lamp-dispatch-what = Se envió trabajo a la GPU en este fotograma.
live-card-lamp-dispatch-good = Encendida mientras algo cambia; apagada en reposo.
live-card-lamp-dispatch-bad = Ocupada sin que nada cambie: la aplicación no se asienta.

live-card-lamp-live-what = La imagen en pantalla es el propio renderizado de este fotograma.
live-card-lamp-live-note = Rara vez encendida durante una inmersión profunda, lo cual es normal: consulta RETENIDA y REPROY.
live-card-lamp-live-good = Encendida a poca profundidad, donde un fotograma es barato.

# "Prefer detail" is a setting's name.
live-card-lamp-held-what = La última imagen completa está en pantalla mientras la siguiente se compone debajo: el refresco de un zoom profundo o, con Preferir detalle, la siguiente tesela o muestra de supermuestreo de una vista quieta.
live-card-lamp-held-good-dive = Encendida a ratos durante los zooms profundos, interrumpida por ADOPCIÓN.
live-card-lamp-held-good-still = Encendida de forma continua mientras una vista quieta se define, con SSAA parpadeando.
live-card-lamp-held-bad = Encendida durante segundos mientras haces zoom, sin ADOPCIÓN: los refrescos no terminan.

live-card-lamp-reproj-what = La última imagen, estirada para ajustarse a la vista en movimiento hasta que haya una nueva lista.
live-card-lamp-reproj-good = Ráfagas cortas mientras te mueves.
live-card-lamp-reproj-bad = Encendida mucho tiempo: la imagen se ve más borrosa cuanto más se estira.

live-card-lamp-reading-what = Volvió una lectura de la GPU: el tiempo, el rango de escape o el contenido de una pasada.
live-card-lamp-reading-good = Un parpadeo constante mientras se renderiza.
live-card-lamp-reading-bad = Apagada mucho tiempo mientras se renderiza: los tiempos no vuelven, y los presupuestos van a ciegas.

live-card-lamp-pin-pass-what = Una pasada de un avance fijado: la imagen renderizada una banda de iteración más allá.
live-card-lamp-pin-pass-good = Series de pasadas que terminan en ADOPCIÓN.
live-card-lamp-pin-pass-bad = Series que terminan en ABANDONO.

live-card-lamp-split-what = Una pasada de un refresco dividido: una parte de las teselas, renderizada con el número completo de iteraciones.
live-card-lamp-split-good = Series cortas que terminan en ADOPCIÓN.
live-card-lamp-split-bad = Series de 8, el máximo permitido: el refresco apenas cabe.

live-card-lamp-adopt-what = Un refresco terminado sustituyó la imagen en pantalla.
live-card-lamp-adopt-good = Varias veces por segundo mientras haces zoom.
# "Picture age" names the instrument (its title, PICTURE AGE).
live-card-lamp-adopt-bad = Rara mientras haces zoom: la imagen envejece (consulta Edad de la imagen).

live-card-lamp-abandon-what = Se descartó un refresco en curso: la vista se movió demasiado, se quedó demasiado antiguo o lo sustituyó otro más reciente. Una construcción de referencia adelantada por la vista la enciende débilmente.
live-card-lamp-abandon-good = Rara, o una vez justo después de un salto.
live-card-lamp-abandon-bad = Frecuente: trabajo desperdiciado, y la imagen sigue siendo antigua.

live-card-lamp-ref-live-what = Se está construyendo una órbita de referencia para la vista tal como está: no tiene ninguna, se ha salido de la que tenía o necesita una más larga.
live-card-lamp-ref-live-good = Parpadeos breves.
live-card-lamp-ref-live-bad-long = Largos ratos: las vistas profundas la esperan.
live-card-lamp-ref-live-bad-lookahead = Encendida a menudo durante una inmersión mientras REF ADELANTE está apagada: la anticipación no da abasto.

live-card-lamp-ref-ahead-what = Se está construyendo una referencia anticipada a más profundidad que la vista, hacia donde se dirige el zoom.
live-card-lamp-ref-ahead-good = Parpadeos regulares durante una inmersión.
live-card-lamp-ref-ahead-bad = Apagada durante una inmersión profunda: la vista dejará atrás sus referencias.

live-card-lamp-ref-hold-what = La precarga de la pausa de un recorrido: la referencia del siguiente fotograma clave con pausa, construida durante el deslizamiento hacia él para que la reproducción nunca espere.
live-card-lamp-ref-hold-note = Apagada fuera de la reproducción de recorridos.
live-card-lamp-ref-hold-good = Un parpadeo antes de cada pausa de un recorrido.
live-card-lamp-ref-hold-bad = Encendida durante toda una pausa: la precarga llega tarde, y el recorrido la espera.

live-card-lamp-series-what = La aproximación por series para una nueva referencia, que permite a cada píxel saltarse sus primeras iteraciones.
live-card-lamp-series-good = Breve.
live-card-lamp-series-bad = Larga a mucha profundidad: retrasa la instalación de la referencia.

live-card-lamp-worker-what = La segunda GPU entregó un fotograma.
live-card-lamp-worker-good = Parpadeos regulares cuando hay una GPU auxiliar activada.
live-card-lamp-worker-bad = Apagada con un auxiliar activado: no está entregando.

live-card-lamp-tile-what = Una tesela de reposo, o una tira de una, se envió a la GPU.
live-card-lamp-tile-good = Una ráfaga después de que te detienes, y luego calma.
live-card-lamp-tile-bad = Nunca se detiene: la rejilla vuelve a empezar una y otra vez, porque algo sigue cambiando la vista o los ajustes.

live-card-lamp-ssaa-what = Se añadió una muestra de supermuestreo a la imagen asentada.
live-card-lamp-ssaa-good = Una serie de parpadeos tras el asentamiento, y luego apagada.
live-card-lamp-ssaa-bad = Vuelve a empezar una y otra vez: algo sigue perturbando el asentamiento.

live-card-lamp-palette-what = La normalización del color tomó un nuevo rango de escape, así que los colores siguen al zoom.
live-card-lamp-palette-good = Parpadea mientras haces zoom.
live-card-lamp-palette-bad = Apagada mientras haces zoom: el rango de la paleta está congelado, y los colores se vuelven planos y con poco contraste.

live-card-lamp-mode-what = Cambió la aritmética: directa, df32 o floatexp.
live-card-lamp-mode-good = Un parpadeo en una frontera como 2^93.
live-card-lamp-mode-bad = Parpadeando de un lado a otro: la vista oscila en torno a una frontera.

## The Render client window (File ▸ Render client…): lending this machine's GPU to a render farm

farm-client-title = Cliente de renderizado
farm-client-intro = Renderiza fotogramas para una granja de renderizado. El coordinador (Herramientas ▸ Renderizar en granja… en otro equipo) muestra su dirección y la clave de la granja. Este equipo no abre ningún puerto: se conecta al coordinador.
# A field label, and the label of the status row that names the controller.
farm-client-controller = Coordinador
farm-client-find = Buscar
farm-client-find-hover = Buscar coordinadores en esta red (un coordinador responde mientras su granja está a la escucha)
farm-client-key = Clave de la granja
# A checkbox beside the masked farm key: show it in clear.
farm-client-show-key = Mostrar
# A field label (the name this machine goes by), and the label of the status row about this machine.
farm-client-machine = Este equipo
farm-client-machine-hover = El nombre con el que el coordinador muestra este equipo.
farm-client-share = Unidad compartida
# Greyed hint inside the empty field; the path is an example.
farm-client-share-hint = opcional — p. ej., \\fileserver\share
farm-client-share-hover = La ruta de este equipo a la unidad compartida que usa el coordinador (modo compartido): los fotogramas se escriben allí en lugar de enviarse por la conexión. Déjalo vacío para enviar todos los fotogramas.
farm-client-gpu = Tarjeta gráfica
# The label of the row with the idle option.
farm-client-when = Cuándo
# A checkbox followed by a number of minutes ("Only when idle for [5] min").
farm-client-when-idle = Solo tras estar inactivo durante
farm-client-when-idle-hover = Aceptar trabajo solo cuando nadie haya tocado el teclado ni el ratón de este equipo durante este tiempo, o mientras su pantalla esté bloqueada. Cuando alguien empieza a usarlo, el fotograma en curso se detiene al instante y vuelve a la granja.
# A collapsible section: the largest job this machine accepts.
farm-client-limits = Límites
farm-client-limits-note = Un trabajo que pida más que esto se rechaza, nunca se recorta.
farm-client-limit-size = Fotograma más grande
farm-client-limit-ss = Supermuestreo máximo
farm-client-limit-iter = Iteraciones máximas

## Render client: the graphics-card choice

farm-client-gpu-system = La elección del sistema
farm-client-gpu-every = Todas las tarjetas gráficas
# $count: this machine's graphics cards.
farm-client-gpu-every-n = Todas las tarjetas gráficas ({ $count })
# $count: this machine's graphics cards.
farm-client-gpu-every-each = Todas las tarjetas gráficas ({ $count }), una sesión cada una
# A card the window cannot name. $number: its number (or numbers) as --list-adapters gives it.
farm-client-gpu-adapter = Adaptador { $number }
farm-client-gpu-finding = Buscando las tarjetas gráficas de este equipo…
# "NAME · GPU 1" is how such a machine is named in the controller's list: keep it as it is.
farm-client-gpu-hover = Qué tarjeta gráfica renderiza los fotogramas de la granja. Con todas las tarjetas, cada una se une a la granja como un equipo propio («NAME · GPU 1», «NAME · GPU 2»), y Pausar y Desconectar actúan sobre todas ellas.

## Render client: looking for controllers on the network (the Find button)

farm-client-finding = Buscando coordinadores en esta red…
# $port: the UDP port number.
farm-client-found-none = No respondió ningún coordinador. Un coordinador responde mientras su granja está a la escucha; su cortafuegos debe dejar entrar UDP { $port }.
farm-client-found-pick = En esta red (haz clic en uno para usarlo):
# $identity: the controller's identity fingerprint.
farm-client-found-hover = Identidad { $identity }: usar este coordinador (sigue haciendo falta la clave de su granja)
# After a warning sign. $version and $git: this app's version and build.
farm-client-found-other-build = Una compilación distinta de esta ({ $version } { $git }): rechazaría este equipo.

## Render client: what is wrong with what was typed, or with starting the client

farm-client-bad-address = Introduce la dirección del coordinador como HOST:PORT, tal como la muestra su ventana (por ejemplo, 192.168.1.20:46733).
# $why: what is wrong with the key (in English).
farm-client-bad-key = La clave de la granja: { $why }
# $why: what is wrong with the name (in English).
farm-client-bad-name = El nombre: { $why }
farm-client-no-config = No hay ninguna carpeta de configuración en la que guardar la clave de la granja.
# $why: the system's reason.
farm-client-start-failed = No se pudo iniciar el cliente de renderizado: { $why }

## Render client: the state, in the status heading

farm-client-phase-connecting = Conectando…
farm-client-phase-checking = Ejecutando la autocomprobación…
farm-client-phase-idle = Conectado: esperando trabajo
farm-client-phase-rendering = Renderizando
farm-client-phase-in-use = En espera: alguien está usando este equipo
farm-client-phase-paused = En pausa
farm-client-phase-retrying = Coordinador inaccesible: reintentando
# $seconds: the wait before the next try.
farm-client-phase-retrying-in = Coordinador inaccesible: reintentando en { $seconds } s
farm-client-phase-disconnected = Desconectado
farm-client-phase-ended = Terminado

## Render client: the line under the status heading

# $address: the controller's address (HOST:PORT).
farm-client-detail-connecting = Conectando con { $address }…
# $controller: the controller's name.
farm-client-detail-checking = Conectado a «{ $controller }»: ejecutando la autocomprobación
# $controller: the controller's name.
farm-client-detail-approval = Conectado a «{ $controller }»: esperando a que su usuario apruebe este equipo
# $controller: the controller's name.
farm-client-detail-idle = Conectado a «{ $controller }»: inactivo
# $why: the check that failed and why (in English).
farm-client-detail-check-failed = La autocomprobación falló: { $why }
# $why: the controller's reason (in English).
farm-client-detail-refused = Rechazado por el coordinador: { $why }
# $why: the controller's reason (in English).
farm-client-detail-closed = El coordinador cerró la conexión: { $why }
# $why: the controller's reason (in English).
farm-client-detail-ended = El coordinador terminó la conexión: { $why }
# $job: the job's name.
farm-client-detail-bundle = Trabajo «{ $job }»: recibiendo su paquete
# $job: the job's name.
farm-client-detail-job-waiting = Trabajo «{ $job }»: esperando fotogramas que renderizar
# $why: the limit the job is over (in English).
farm-client-detail-job-refused = Se rechazó este trabajo: { $why }
farm-client-detail-job-over = El trabajo ha terminado
# $first and $last: frame numbers.
farm-client-detail-rendering = Renderizando los fotogramas { $first }–{ $last }
# $time: how long nobody must use the machine ("90 s", "5 min").
farm-client-detail-in-use = En espera: alguien está usando este equipo; el trabajo se reanuda cuando nadie lo haya usado durante { $time }
farm-client-detail-paused = En pausa: se ha avisado al coordinador
farm-client-detail-pausing = Pausando: terminando el fotograma en curso
farm-client-detail-cancelled = Se canceló el fotograma en curso; en pausa
# A machine with several graphics cards. $rendering: the cards rendering now; $total: all its cards (two or more).
farm-client-rendering-on-cards = Renderizando en { $rendering } de { $total } tarjetas gráficas

## Render client: the status rows

# The value of the Controller row. $name: the controller's name; $address: HOST:PORT; $identity: its identity fingerprint.
farm-client-row-controller-value = «{ $name }» · { $address } · identidad { $identity }
# The value of the This machine row. $name: this machine's name; $identity: its identity fingerprint.
farm-client-row-machine-value = «{ $name }» · identidad { $identity }
farm-client-row-gpu = GPU
# One graphics card's row. $slot: the card's number.
farm-client-row-gpu-n = GPU { $slot }
farm-client-row-job = Trabajo
# The frames handed to this machine.
farm-client-row-run = Tanda
farm-client-row-sent = Enviados
# The Run row, in parts joined with " · ": the run, the frame in progress, its time. $first and $last: frame numbers.
farm-client-run-frames = fotogramas { $first }–{ $last }
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-client-run-frame = fotograma { $frame } ({ $index } de { $count })
# $time: the time spent on it so far ("2.1 s").
farm-client-run-frame-time = { $time } en este fotograma
# One graphics card's run. $frame: the frame in progress; $first and $last: the run's frames.
farm-client-session-frame = fotograma { $frame } de { $first }–{ $last }
farm-client-session-no-card = tarjeta gráfica aún desconocida
# The Sent row. $count: frames sent; $time: the mean time per frame ("2.1 s").
farm-client-sent =
    { $count ->
        [one] { $count } fotograma · { $time } por fotograma
       *[other] { $count } fotogramas · { $time } por fotograma
    }
# The Sent row of a machine with several cards. $count: frames sent; $split: each card's share ("812 + 422"); $time: the mean time per frame.
farm-client-sent-split =
    { $count ->
        [one] { $count } fotograma ({ $split }) · { $time } por fotograma
       *[other] { $count } fotogramas ({ $split }) · { $time } por fotograma
    }
# After a warning sign. $note: how the GPUs differ, and both GPUs (in English).
farm-client-gpu-note = Este equipo tiene { $note }. Sus fotogramas diferirán ligeramente de los del coordinador, lo que puede notarse en un plano fijo.
# A collapsible section: the self-check's results.
farm-client-self-check = Autocomprobación
farm-client-thumb-hover = El último fotograma que envió este equipo
farm-client-resume = Reanudar
farm-client-resume-hover = Volver a aceptar trabajo
farm-client-pause = Pausar
farm-client-pause-hover = Terminar el fotograma en curso y luego no aceptar más trabajo. El coordinador reparte los fotogramas de este equipo entre los demás.
farm-client-cancel-frame = Cancelar fotograma
farm-client-cancel-frame-hover = Detener ahora el fotograma en curso y pausar; el coordinador lo renderiza en otro equipo.
farm-client-live-paused = Vista en vivo en pausa mientras este equipo renderiza fotogramas para la granja.
farm-client-close = Cerrar
farm-client-close-active-hover = Cerrar la ventana; este equipo sigue en la granja (vuelve a abrirla desde el menú Archivo)
farm-client-close-hover = Cerrar la ventana
farm-client-disconnect = Desconectar
farm-client-disconnect-hover = Salir de la granja: el fotograma en curso se devuelve
farm-client-connect = Conectar
farm-client-connect-hover = Conectarse al coordinador y renderizar fotogramas para él

## The Render on farm window (Tools ▸ Render on farm…): the controller of a render farm

farm-ctl-title = Renderizar en granja
farm-ctl-out = Carpeta de salida
farm-ctl-browse = Examinar…
farm-ctl-out-note = Los fotogramas, y el estado del trabajo en farm/: vuelve a ejecutar el mismo trabajo para reanudarlo.
farm-ctl-size = Tamaño
farm-ctl-size-custom = Personalizado…
# The size list's choice when the size is typed in. $width and $height: pixels.
farm-ctl-size-custom-value = Personalizado — { $width }×{ $height }
farm-ctl-fps = Fotogramas / segundo
farm-ctl-ss = Supermuestreo
farm-ctl-port = Escuchar en el puerto
farm-ctl-port-hover = Los clientes se conectan a este puerto. Permítelo en el cortafuegos para tu red local.
farm-ctl-share = Unidad compartida
# Greyed hint inside the empty field.
farm-ctl-share-hint = opcional — la ruta de este equipo a ella
farm-ctl-share-hover = Modo compartido: los clientes que tienen la misma unidad escriben en ella sus fotogramas en lugar de enviarlos por la conexión, y este equipo comprueba allí cada uno. Los clientes que no la tienen envían los suyos.
# A label followed by a number of clients and two checkboxes.
farm-ctl-start-with = Empezar con
# The unit after the number in the box ("Start with [3] clients"). $count: that number.
farm-ctl-start-clients =
    { $count ->
        [one] cliente
       *[other] clientes
    }
farm-ctl-start-clients-hover = El trabajo empieza en cuanto se han conectado este número de equipos y han superado su autocomprobación; otros pueden unirse más tarde.
farm-ctl-local = Renderizar también en este equipo
farm-ctl-local-hover = Este equipo cuenta como un cliente.
farm-ctl-sharing = Compartir referencias
farm-ctl-sharing-hover = La órbita de referencia de un fotograma profundo puede tardar de minutos a horas en construirse. Compartida, se construye una sola vez y se pasa a los demás equipos. Una referencia compartida es otra referencia, igual de válida: a profundidad extrema, un puñado de píxeles puede diferir de un fotograma que construyó la suya propia.
farm-ctl-key = Clave de la granja
# A checkbox beside the shortened farm key: show all of it.
farm-ctl-show-key = Mostrar
farm-ctl-copy-key = Copiar clave
farm-ctl-copy-key-hover = Pégala en la ventana Cliente de renderizado de cada cliente
# In place of the farm key, when there is none.
farm-ctl-no-key = aún no hay ninguna
farm-ctl-make-key = Crear una clave
farm-ctl-make-key-hover = Los clientes la necesitan para unirse; también se crea al pulsar Renderizar
# What to do on the other machines. $address: this controller's HOST:PORT.
farm-ctl-steps = En cada cliente: Archivo ▸ Cliente de renderizado…, coordinador { $address }, y la clave de la granja.
# The same when this machine's address is not known. $port: the port number.
farm-ctl-steps-port = En cada cliente: Archivo ▸ Cliente de renderizado…, la dirección de este equipo y el puerto { $port }, y la clave de la granja.
# The size of the job. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
farm-ctl-summary =
    { $count ->
        [one] { $count } fotograma · { $seconds }s de recorrido · { $width }×{ $height } ss{ $ss }
       *[other] { $count } fotogramas · { $seconds }s de recorrido · { $width }×{ $height } ss{ $ss }
    }
farm-ctl-strip-hover = Todos los fotogramas del trabajo, de izquierda a derecha: hechos, en renderizado, pendientes, abandonados.
farm-ctl-no-tour = No hay ningún recorrido cargado.
# $folder: the folder; $why: the system's reason.
farm-ctl-out-error = La carpeta de salida { $folder }: { $why }
# $why: the system's reason.
farm-ctl-start-failed = No se pudo iniciar la granja: { $why }
# $code: the controller process's exit code, as the code gives it.
farm-ctl-ended-exit = El coordinador terminó (salida { $code }).

## Render on farm: the state, in the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-phase-waiting = Esperando clientes de renderizado ({ $connected } de { $needed } conectados)
farm-ctl-phase-preparing = Preparando el trabajo
farm-ctl-phase-rendering = Renderizando
farm-ctl-phase-paused = En pausa
farm-ctl-phase-complete = Completado
farm-ctl-phase-stopped = Detenido
farm-ctl-phase-ended = Terminado

## Render on farm: the words after the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-detail-waiting = Esperando clientes de renderizado: { $connected } de { $needed } conectados
farm-ctl-detail-preparing = Midiendo las anclas de normalización en este equipo, una vez para todos los clientes
farm-ctl-detail-no-space = Esperando a que haya espacio en la carpeta de salida
farm-ctl-detail-paused = En pausa: no se reparte trabajo nuevo
farm-ctl-detail-not-rendered = Terminado con fotogramas sin renderizar: vuelve a ejecutarlo sobre la misma carpeta para reanudarlo
farm-ctl-detail-stopped = Detenido: vuelve a ejecutarlo sobre la misma carpeta para reanudarlo
# $code: the controller process's exit code.
farm-ctl-detail-ended = Terminado (salida { $code })
farm-ctl-detail-finishing = Terminando

## Render on farm: progress

# On the progress bar. $done: frames done; $total: frames in the job.
farm-ctl-progress =
    { $total ->
        [one] { $done } / { $total } fotograma
       *[other] { $done } / { $total } fotogramas
    }
# Added to the progress bar. $rate: frames per second, with decimals.
farm-ctl-progress-rate = { $rate } fotogramas/s
# Added to the progress bar. $time: the time still to go ("38m 10s").
farm-ctl-progress-eta = quedan unos { $time }
# Facts under the progress bar, joined with " · ". $received and $sent: kilobytes per second.
farm-ctl-io = entrada { $received } KB/s · salida { $sent } KB/s
# $space: free space on the output drive ("412 GB").
farm-ctl-free = { $space } libres
# $time: the time since the job started ("38m 10s").
farm-ctl-elapsed = { $time } transcurridos
# $count: frames given up on.
farm-ctl-failed = { $count } sin renderizar
# Reference orbits. $built: built for this job; $cached: found in the cache; $reused: reused between frames; $shared: passed between machines.
farm-ctl-refs = referencias: { $built } construidas, { $cached } de la caché, { $reused } reutilizadas · { $shared } compartidas
# After a warning sign.
farm-ctl-storage-low = La carpeta de salida está casi llena: no se reparte trabajo nuevo hasta que haya espacio.
# After a warning sign.
farm-ctl-gpus-differ = Las GPU de esta granja difieren. Sus fotogramas difieren ligeramente, lo que puede notarse en un plano fijo.
# After a warning sign. $count: the GPU classes in the farm.
farm-ctl-gpus-differ-classes = Las GPU de esta granja difieren ({ $count } clases de GPU: GPU, controladores o renderizados de sondeo distintos). Sus fotogramas difieren ligeramente, lo que puede notarse en un plano fijo.
# $gpu: this machine's GPU, API and driver.
farm-ctl-this-gpu = Este equipo: { $gpu }
# A machine whose GPU differs. $name: the machine; $note: how its GPU differs (in English); $adapter: its GPU.
farm-ctl-gpu-row = { $name }: { $note } — { $adapter }
# The same with its driver. $driver: the driver's name and version.
farm-ctl-gpu-row-driver = { $name }: { $note } — { $adapter }, controlador { $driver }

## Render on farm: the table of machines

# Column headers (short: the table is narrow).
farm-ctl-col-machine = Equipo
farm-ctl-col-state = Estado
farm-ctl-col-gpu = GPU
farm-ctl-col-frames = Fotogramas
farm-ctl-col-per-frame = Por fotograma
# How the machine delivers its frames (a speed, or "shared drive").
farm-ctl-col-link = Enlace
# When the controller last heard from the machine.
farm-ctl-col-heard = Contacto
# In the machine's tooltip. $count: its frames that failed verification.
farm-ctl-strikes =
    { $count ->
        [one] { $count } fotograma no superó la verificación
       *[other] { $count } fotogramas no superaron la verificación
    }
# In the Link column: it delivers its frames through the shared drive.
farm-ctl-link-share = unidad compartida
# In the Heard column. $seconds: how long ago.
farm-ctl-heard-ago = hace { $seconds } s
farm-ctl-readmit = Readmitir
farm-ctl-readmit-hover = Permitir que este equipo vuelva a unirse cuando se reconecte
farm-ctl-remove = Quitar
farm-ctl-remove-hover = Sacar este equipo del trabajo; sus fotogramas pasan a los demás
# The GPU column's tooltip.
farm-ctl-probe-same = Su renderizado de sondeo es idéntico al de este equipo.
# $count: the pixels that differ.
farm-ctl-probe-differs =
    { $count ->
        [one] Su renderizado de sondeo difiere del de este equipo en { $count } píxel: sus fotogramas diferirán ligeramente de los de este equipo.
       *[other] Su renderizado de sondeo difiere del de este equipo en { $count } píxeles: sus fotogramas diferirán ligeramente de los de este equipo.
    }
farm-ctl-probe-unknown = Su renderizado de sondeo aún no se ha comparado.
# $driver: the driver's name and version.
farm-ctl-gpu-driver = controlador { $driver }
# After a warning sign. $note: how its GPU differs (in English: "a different GPU").
farm-ctl-gpu-note = { $note } respecto a la de este equipo

## Render on farm: a machine's state, in the State column

# $count: its frames that failed verification.
farm-ctl-state-removed-bad =
    { $count ->
        [one] quitado: { $count } fotograma erróneo
       *[other] quitado: { $count } fotogramas erróneos
    }
farm-ctl-state-removed = quitado por ti
farm-ctl-state-removing = quitando: recopilando su diagnóstico
farm-ctl-state-gone = se fue
farm-ctl-state-leaving = saliendo
farm-ctl-state-checking = comprobando (autocomprobación)
farm-ctl-state-paused = en pausa por su usuario
farm-ctl-state-parked = aparcado: falló dos veces; vuelve a unirse cuando se reconecte
farm-ctl-state-refused = rechazó este trabajo (sus propios límites)
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-ctl-state-rendering = renderizando { $frame } ({ $index }/{ $count })
farm-ctl-state-starting = empezando una tanda
farm-ctl-state-idle = inactivo
farm-ctl-state-admitted = admitido: esperando el trabajo

## Render on farm: the buttons

# A collapsible section: the controller's messages.
farm-ctl-events = Eventos
farm-ctl-resume = Reanudar
farm-ctl-pause = Pausar
farm-ctl-pause-hover = No repartir trabajo nuevo; los fotogramas en curso terminan
farm-ctl-live-paused = Vista en vivo en pausa mientras este equipo renderiza para la granja.
farm-ctl-close = Cerrar
farm-ctl-close-active-hover = Cerrar la ventana; el trabajo sigue en marcha (vuelve a abrirla desde el menú Herramientas)
farm-ctl-close-hover = Cerrar la ventana
farm-ctl-copy-command = Copiar comando
farm-ctl-copy-command-hover = Copiar la línea de comandos equivalente
farm-ctl-stop = Detener
# The Stop button once a stop has been asked for and the job has not ended after 10 seconds.
farm-ctl-stop-now = Detener ya
farm-ctl-stop-hover = Terminar el trabajo con los fotogramas que tiene; volver a ejecutar el mismo trabajo lo reanuda
farm-ctl-render = Renderizar
farm-ctl-render-hover = Empezar a escuchar a los clientes y renderizar el recorrido en ellos

## The Render tour window (Tools ▸ Render tour…)

tour-render-title = Renderizar recorrido
tour-render-out = Carpeta de salida
tour-render-browse = Examinar…
tour-render-out-hover = Ruta completa en la que se escriben los fotogramas.
tour-render-prefix = Prefijo de fotograma
# <prefix> stands for the prefix typed in the field.
tour-render-prefix-hover = Los fotogramas se escriben como <prefijo>_00000.png
tour-render-size = Tamaño
tour-render-size-custom = Personalizado…
# The size list's choice when the size is typed in. $width and $height: pixels.
tour-render-size-custom-value = Personalizado — { $width }×{ $height }
tour-render-fps = Fotogramas / segundo
tour-render-fps-hover = Menos de 1 fps es válido: muestrear una inmersión larga a 0,25 fps es la forma de inspeccionar las pausas de las pruebas de regresión profundas sin renderizar miles de fotogramas.
tour-render-ss = Supermuestreo
tour-render-chapter = Capítulo
tour-render-whole = Recorrido completo
tour-render-chapter-hover = Renderizar un solo capítulo conserva la numeración GLOBAL de fotogramas, así que sus fotogramas encajan directamente en una secuencia completa.
tour-render-mp4 = Montar mp4
tour-render-mp4-hover = Ejecuta ffmpeg al final (debe estar en el PATH). Los fotogramas PNG se conservan.
tour-render-overwrite = Sobrescribir los fotogramas existentes
tour-render-overwrite-hover = Desactivado: el renderizado se detiene para preguntar, cosa que no puede hacer desde aquí.
tour-render-resume = Reanudar un renderizado interrumpido
tour-render-resume-hover = Conservar los fotogramas que ya hay en la carpeta de salida y renderizar solo los que faltan. Primero se comprueba el fotograma más reciente y se descarta si está incompleto (el fotograma en el que muere un renderizado es el que estaba escribiendo), retrocediendo hasta encontrar uno bueno; una carpeta con fotogramas de otro tamaño se rechaza en lugar de mezclarse con esta secuencia.
# $count: this machine's graphics cards.
tour-render-all-gpus = Usar todas las tarjetas gráficas ({ $count })
tour-render-all-gpus-hover = Renderizar a la vez en todas las tarjetas gráficas de este equipo, mediante una granja de renderizado en este ordenador (--gpus all). Los fotogramas, sus nombres, Reanudar y el mp4 quedan igual que con una sola tarjeta; se renderiza el recorrido completo, en el orden propio de la granja. Los distintos modelos de tarjeta dibujan unos pocos píxeles de forma ligeramente distinta: un plano fijo y un fundido se quedan en una misma tarjeta.
tour-render-order = Orden de renderizado
tour-render-sequential = Secuencial
tour-render-sequential-hover = Fotograma 0, 1, 2, … en orden (el predeterminado).
tour-render-progressive = Progresivo
tour-render-progressive-hover = Previsualizar primero todo el recorrido y refinarlo hasta el detalle completo: primero se renderizan los fotogramas clave y luego se dividen en dos los mayores huecos temporales hasta que existen todos los fotogramas. En ambos casos, los fotogramas quedan en sus índices correctos, así que Reanudar y el mp4 funcionan igual: puedes detenerlo pronto, echar un vistazo al folioscopio aproximado y reiniciar con Reanudar en cualquiera de los dos órdenes.
# The cost of the render. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
tour-render-summary =
    { $count ->
        [one] { $count } fotograma · { $seconds }s de recorrido · { $width }×{ $height } ss{ $ss }
       *[other] { $count } fotogramas · { $seconds }s de recorrido · { $width }×{ $height } ss{ $ss }
    }
# The disk space the frames will take (an estimate). $size: a size ("5.3 GB").
tour-render-disk = ≈{ $size } de fotogramas
# $size: the frames' estimated size; $free: the free space on that drive ("6.0 GB").
tour-render-disk-free = ≈{ $size } de fotogramas · { $free } libres
# After a warning sign. $disk: the line above ("≈51.0 GB of frames · 6.0 GB free").
tour-render-disk-short = { $disk } — no hay espacio para toda la secuencia
tour-render-separate = Renderiza el guion DESDE EL DISCO en un proceso aparte, así que no se incluyen los cambios sin guardar, y una pérdida del dispositivo de GPU allí no puede arrastrar consigo esta sesión.
# On the progress bar. $done: frames done; $total: frames to render.
tour-render-progress =
    { $total ->
        [one] { $done } / { $total } fotograma
       *[other] { $done } / { $total } fotogramas
    }
# Before the render's first progress line.
tour-render-starting = iniciando…
tour-render-live-paused = Vista en vivo en pausa: el renderizado tiene la GPU para él solo, así que termina antes y mide con precisión el coste de sus propios fotogramas.
tour-render-cancel = Cancelar
tour-render-cancel-hover = Cerrar sin renderizar
tour-render-copy-command = Copiar comando
tour-render-copy-command-hover = Copiar al portapapeles la línea de comandos equivalente
tour-render-stop = Detener el renderizado
tour-render-render = Renderizar
tour-render-render-hover = Empezar a renderizar la secuencia de fotogramas
# $folder: the output folder.
tour-render-finished = Renderizado terminado → { $folder }
# $why: the render's own reason (in English).
tour-render-failed = Error en el renderizado: { $why }
# $last: the render's last progress line (in English), or nothing.
tour-render-failed-or-stopped = El renderizado falló o se detuvo. { $last }
# $why: the system's reason.
tour-render-no-exe = No se encuentra el ejecutable: { $why }
# $why: the system's reason.
tour-render-start-failed = No se pudo iniciar el renderizado: { $why }

## The Custom formula dialog (Fractal ▸ Custom formula…)
## The formula language itself stays as it is in every language: function names (sin, cosh, conj…),
## operators, variables (z, c, p1), and the formulas. Keep them as written inside these messages.

# The window's title.
formula-dialog-title = Fórmula personalizada
# A small line at the top of the window, above the formula.
formula-dialog-intro = El paso, en expresiones al estilo de Fractint
# The two ways to show and edit the formula (top right of the window).
formula-dialog-textbook = Libro de texto
formula-dialog-textbook-hover = Mostrar la fórmula compuesta, como la compondría un libro de texto, y editarla así
formula-dialog-text = Texto
formula-dialog-text-hover = Editar la fórmula como texto
# The tooltip of the "LaTeX" button (the button's own text stays LaTeX).
formula-dialog-latex-hover = Copiar la fórmula como LaTeX, tal como está compuesta aquí
# The drop-down list of example formulas (the collection).
formula-dialog-examples = Ejemplos
# Help under the formula in Textbook mode. The characters / ^ ( are what the user types: keep them.
formula-dialog-help-textbook = Escribe como en el texto: / convierte en fracción el término anterior, ^ crea un exponente, ( abre paréntesis. Las flechas recorren la fórmula; Tab va a la siguiente casilla vacía. Una línea en rojo no se puede leer: haz clic en ella para editarla como texto.
# Help under the formula in Text mode. "; starts a comment": the ; is what the user types.
formula-dialog-help-text = Escribe, o usa el teclado: contiene todos los nombres que conoce el lenguaje de fórmulas. Los nombres se completan mientras escribes (Tab). Las sentencias se separan con un salto de línea o una coma; ; empieza un comentario.
# The syntax check, when the formula reads.
formula-dialog-reads = Se lee correctamente.
# The syntax check in Textbook mode, when the formula has an empty box to fill in.
formula-dialog-empty-box = Rellena la casilla vacía.
# The syntax check in Textbook mode. $line: the line number; $message: what is wrong (from the formula parser, in English).
formula-dialog-error-line = Línea { $line }: { $message }
# One function written with another notation's name, and the language's name for it: "ln to log".
# $from: the name as written (ln); $to: the formula language's name (log).
formula-respell-pair = { $from } por { $to }
# A button that rewrites them. $changes: one or more formula-respell-pair, joined with ", ".
formula-respell = Cambiar { $changes }
formula-respell-hover = Escribir estas funciones como las nombra el lenguaje de fórmulas
# Shown when Apply finds a parameter field that is not a number. $name: the parameter (p1 … p5).
formula-param-not-numbers = { $name } no es un par de números
# The Apply button (also in the Formula library window).
formula-apply = Aplicar
formula-dialog-apply-hover = Compilar la fórmula y mostrarla
# Shown beside Apply while the formula is being compiled.
formula-dialog-compiling = Compilando la fórmula para la GPU…
# The greyed hint in an empty name box (also in the Formula library window).
formula-name-hint = nombre (opcional)
# The save button: "Update" when the library already has a formula of that name.
formula-dialog-update-in-library = Actualizar en la biblioteca
formula-dialog-save-to-library = Guardar en la biblioteca
formula-dialog-save-hover = Guardar esta fórmula y sus parámetros en la biblioteca de fórmulas
formula-dialog-save-disabled-hover = Solo se puede guardar una fórmula que se lea correctamente
# Opens the Formula library window.
formula-dialog-library = Biblioteca…
formula-dialog-library-hover = Las fórmulas guardadas: aplicarlas, editarlas, importarlas y exportarlas

## The Custom formula dialog's keypad
## The keys' captions are formula tokens and stay as they are; these are their tooltips.

# The keypad's third tab (the other two are "123" and "f(z)"). A short word.
formula-keypad-tab-names = nombres
# The tabs' tooltips.
formula-keypad-tab-basic-hint = Números, operadores y los nombres más usados
formula-keypad-tab-functions-hint = Funciones
formula-keypad-tab-names-hint = Todas las variables, parámetros y constantes, y la sintaxis de las sentencias
# The key z.
formula-key-z = El iterado
# The key c.
formula-key-c = El píxel (la constante de Julia en el modo Julia)
# The key p1 on the first tab.
formula-key-p1-below = Parámetro 1: se fija en los campos de abajo
# The keys p1 … p5. $n: the parameter's number.
formula-key-param = Parámetro { $n }
formula-key-multiply = Multiplicar
formula-key-divide = Dividir
# The key that types ^2.
formula-key-square-after = Cuadrado (tras un valor: z^2)
# The key that types ^.
formula-key-power = Potencia: un exponente entero, real o complejo
formula-key-square-root = Raíz cuadrada
# The key that types |…|: in this language it is the squared modulus, as in Fractint.
formula-key-squared-modulus = Módulo AL CUADRADO |z|², como en Fractint
formula-key-add = Sumar
formula-key-subtract = Restar (o cambiar de signo)
# The key that types =.
formula-key-assign = Asignar: z = …, o una temporal t = …
# The key that starts a new line (first tab).
formula-key-next-statement-comma = Siguiente sentencia (también sirve una coma)
# The key "del".
formula-key-backspace = Borrar el carácter anterior al cursor (o la selección)
# The key e.
formula-key-euler = El número de Euler
# The key that types (0.5, 0.5).
formula-key-complex-constant = Una constante compleja: (real, imaginaria)
# The key that types a comma.
formula-key-comma = Separa sentencias, o las partes de una constante compleja
formula-key-left = Cursor a la izquierda
formula-key-right = Cursor a la derecha
# The function keys (the f(z) tab), one per function.
formula-key-sin = Seno
formula-key-cos = Coseno
formula-key-tan = Tangente
formula-key-cotan = Cotangente
formula-key-exp = Exponencial e^z
formula-key-sinh = Seno hiperbólico
formula-key-cosh = Coseno hiperbólico
formula-key-tanh = Tangente hiperbólica
formula-key-cotanh = Cotangente hiperbólica
formula-key-log = Logaritmo natural, ln (rama principal)
formula-key-sqr = Cuadrado: sqr(z) = z²
formula-key-sqrt = Raíz cuadrada (principal)
formula-key-recip = Recíproco 1/z
formula-key-conj = Conjugado complejo
formula-key-flip = Intercambiar las partes real e imaginaria
formula-key-abs = Valor absoluto de AMBAS partes: |x| + i|y|
formula-key-cabs = Módulo |z| (un número real)
formula-key-real = Parte real
formula-key-imag = Parte imaginaria, como número real
# "fn" is Fractint's name for a function slot: keep it.
formula-key-ident = Identidad (el comodín fn de Fractint)
# The key "pixel".
formula-key-pixel = Lo mismo que c (el nombre de Fractint)
# The key "z =".
formula-key-set-z = Empezar una sentencia que asigna z
# The key "t =".
formula-key-temporary = Una temporal: sirve cualquier nombre nuevo (t, w, a2, …)
# The key "; …".
formula-key-comment = Un comentario, hasta el final de la línea
# The key that starts a new line (names tab).
formula-key-next-statement = Siguiente sentencia

## The formula's completion list (names offered as you type)

# Beside a name the formula itself assigns.
formula-completion-variable = Una variable que asigna esta fórmula
# Beside a function typed with another notation's name.
# $what: what the function is, in English (from the formula parser: "The natural logarithm");
# $other: the other notation's name (ln); $ours: the formula language's name (log).
formula-completion-other-spelling = { $what } — { $other } se escribe { $ours } aquí
# Under a list cut short. $count: the names not shown.
formula-completion-more = … { $count } más
# The last line of the list. Tab, Enter and Esc are key names.
formula-completion-keys = Tab o Enter para completar, Esc para cerrar

## The Custom formula dialog's Textbook mode

# The tooltip of a line that does not read. $why: what is wrong (from the formula parser, in English, or formula-textbook-fractint-sections).
formula-textbook-unread-hover = No se puede leer: { $why }. Haz clic para editarla como texto.
# Why a line with Fractint's sections is shown as text in Textbook mode.
formula-textbook-fractint-sections = Las secciones de Fractint (el ':' de una inicialización, las comparaciones, a = b = …) se editan como texto

## The Formula library window (Fractal ▸ Formula library…)

# The window's title.
formula-library-title = Biblioteca de fórmulas
# The window's two tabs. $count: the formulas in each.
formula-library-mine = Mis fórmulas ({ $count })
formula-library-mine-hover = Las fórmulas que guardaste o importaste
formula-library-collection = Colección ({ $count })
formula-library-collection-hover = Fórmulas que vienen con Fractadyne, cada una con una vista de partida
# A small marker before the name of the formula the view shows now. A short word.
formula-library-showing = actual
# The tooltip of Apply on a formula of the collection.
formula-library-apply-shipped-hover = Mostrarla, en su vista de partida
# A row's Edit button (after a pencil icon).
formula-library-edit = Editar
formula-library-edit-hover = Abrirla en el cuadro Fórmula personalizada, sin aplicarla
# A collection row's button that copies the formula into "My formulas".
formula-library-copy-to-mine = Copiar a las mías
formula-library-copy-to-mine-hover = Añadirla a tus fórmulas, para conservarla o modificarla
formula-library-close = Cerrar
# Saves the formula the view shows, under the name typed beside it.
formula-library-add-current = Añadir la fórmula actual
formula-library-add-current-hover = Guardar la fórmula que muestra la vista
# "Fractal > Custom formula…" is the menu path: write it as the menus are named.
formula-library-add-current-disabled-hover = Primero aplica una fórmula personalizada (Fractal > Fórmula personalizada…)
formula-library-import = Importar…
formula-library-import-hover = Añadir las fórmulas de un archivo de fórmulas; no se sustituye nada de lo que hay aquí
formula-library-export-all = Exportar todas…
formula-library-export-all-hover = Escribir todas las fórmulas de aquí en un solo archivo, para compartirlas o guardarlas
# The filter box: its label, and the greyed hint inside it.
formula-library-filter = Filtrar
formula-library-filter-hint = filtrar por nombre o texto
# Shown while the user has no saved formulas.
formula-library-empty = Aún no hay fórmulas guardadas. Guarda una desde el cuadro Fórmula personalizada, añade la que muestra la vista o importa un archivo (nuestro, o un .frm de Fractint).
# The tooltip of Apply on one of the user's formulas: with a view saved, and without one.
formula-library-apply-view-hover = Mostrar esta fórmula, en la vista guardada con ella
formula-library-apply-hover = Mostrar esta fórmula
# The tooltip of a row's save icon.
formula-library-export-one-hover = Exportar esta fórmula a un archivo
# A row asks before deleting: the question, then its two buttons. "Delete" is also the delete icon's tooltip.
formula-library-delete-ask = ¿Eliminarla?
formula-library-delete = Eliminar
formula-library-keep = Conservar
# Under a list cut short. $count: the formulas not listed.
formula-library-more = …y { $count } más: escribe en el filtro para encontrar una.
formula-library-no-match = Ninguna coincide con el filtro.
# Under a saved formula this version cannot read. $why: what is wrong (from the formula parser, in English).
formula-library-unread = No se puede leer en esta versión: { $why }

## Formula library notices

# $why: the system's reason, in English.
formula-library-save-failed = No se pudo guardar la biblioteca de fórmulas: { $why }
formula-library-nothing-to-save = Nada que guardar: la fórmula está vacía.
# $name: the formula's name in the library.
formula-library-updated = «{ $name }» actualizada en la biblioteca de fórmulas.
formula-library-saved = «{ $name }» guardada en la biblioteca de fórmulas.
# $name: the formula's name; $why: what is wrong.
formula-library-apply-failed = «{ $name }» no se puede aplicar: { $why }
# Copying a formula of the collection into "My formulas". $name: its name; $as_named: the name it was given (the same, or "Name (2)" when the name was taken).
formula-library-copy-exists = «{ $name }» ya está en tus fórmulas.
formula-library-copied = «{ $name }» copiada a tus fórmulas como «{ $as_named }».
# $name: the formula's name.
formula-library-deleted = «{ $name }» eliminada de la biblioteca de fórmulas.
# Why the library could not be saved (follows formula-library-save-failed).
formula-library-no-config-dir = no hay carpeta de configuración
# At startup, when the library file cannot be read. $why: the reason, in English; $path: a file's full path.
formula-library-unreadable-kept = No se pudo leer la biblioteca de fórmulas ({ $why }). Se conservó como { $path } y se empezó una biblioteca nueva y vacía.
formula-library-unreadable-stuck = No se pudo leer la biblioteca de fórmulas ({ $why }) ni apartarla: copia { $path } a un lugar seguro antes de guardar una fórmula, lo que la sustituirá.

## Formula library: Import and Export
## A formula file's contents stay in English; this is what the windows and notices around them say.

# File-type names in the open and save dialogs (the extensions are added by the system).
formula-filter-formula-files = Archivos de fórmulas
formula-filter-fractadyne = Fórmulas de Fractadyne
formula-filter-fractint = Fórmulas de Fractint
formula-filter-all-files = Todos los archivos
# Why a file cannot be imported (they follow formula-import-failed).
formula-import-too-large = mucho más grande de lo que puede ser un archivo de fórmulas
formula-import-not-text = no es un archivo de texto
formula-import-no-formulas = no contiene fórmulas
# $why: the TOML reader's message, in English.
formula-file-not-formulas = no es un archivo de fórmulas ({ $why })
# $format: what the file says it is ("fractadyne-gradients").
formula-file-other-format = un archivo «{ $format }», no un archivo de fórmulas
# "[[formula]]" is the file's own syntax: keep it.
formula-file-no-formulas = no contiene fórmulas (un archivo de fórmulas las enumera como tablas [[formula]])
# $file: the file's name; $why: one of the reasons above, or the system's (in English).
formula-import-failed = No se pudo importar «{ $file }»: { $why }.
# After an import that added nothing. $file: the file's name.
formula-import-nothing-new = Nada nuevo en «{ $file }».
# After an import. $count: the formulas added; $file: the file's name.
formula-import-added =
    { $count ->
        [one] Se importó { $count } fórmula de «{ $file }».
       *[other] Se importaron { $count } fórmulas de «{ $file }».
    }
# The same two, followed by what else happened. $notes: one or more of the formula-import-note-… messages below, joined with "; ".
formula-import-nothing-new-notes = Nada nuevo en «{ $file }»: { $notes }.
formula-import-added-notes =
    { $count ->
        [one] Se importó { $count } fórmula de «{ $file }»: { $notes }.
       *[other] Se importaron { $count } fórmulas de «{ $file }»: { $notes }.
    }
# Notes (clauses, no final full stop). $count: the formulas already in the library, skipped.
formula-import-note-duplicates =
    { $count ->
        [one] { $count } fórmula ya estaba en la biblioteca
       *[other] { $count } fórmulas ya estaban en la biblioteca
    }
# One formula was renamed because its name was taken. $from: its name in the file; $to: the name it was given ("Name (2)").
formula-import-note-renamed-one = «{ $from }» ya existía, así que ahora es «{ $to }»
# $count: the formulas renamed because their names were taken (2 or more).
formula-import-note-renamed = { $count } renombradas porque el nombre ya existía
# $count: the entries skipped as empty or too long.
formula-import-note-skipped =
    { $count ->
        [one] { $count } entrada estaba vacía o era demasiado larga, omitida
       *[other] { $count } entradas estaban vacías o eran demasiado largas, omitidas
    }
# A second sentence after importing a Fractint .frm file. $count: the entries that do not read; $why: the commonest reason (from the formula parser, in English).
formula-import-frm-unread =
    { $count ->
        [one] { $count } entrada no se puede leer en esta versión (lo más habitual: { $why }).
       *[other] { $count } entradas no se pueden leer en esta versión (lo más habitual: { $why }).
    }
# After exporting one formula. $name: the formula's name; $file: the file's name.
formula-export-one = «{ $name }» exportada a «{ $file }».
# After exporting several. $count: how many; $file: the file's name.
formula-export-many =
    { $count ->
        [one] Se exportó { $count } fórmula a «{ $file }».
       *[other] Se exportaron { $count } fórmulas a «{ $file }».
    }
# $file: the file's name; $why: the system's reason, in English.
formula-export-failed = No se pudo escribir «{ $file }»: { $why }

## The control panel (right-hand side): header and section headings

# The panel's title, beside the button that hides it.
panel-title = Controles
panel-hide-hover = Ocultar el panel de control
# The button that brings the hidden panel back (top right of the view).
panel-show-hover = Mostrar el panel de control
# Section headings.
panel-section-navigate = Navegación
panel-section-coloring = Coloreado
panel-section-quality = Calidad
panel-section-effects = Efectos
panel-section-overlays = Superposiciones
panel-section-advanced = Avanzado
# The last section: reference notes about the current fractal. $name: the fractal's name (Mandelbrot, Burning Ship…).
panel-section-about = Acerca de { $name }
panel-section-performance = Rendimiento

## The control panel: Navigate
## Labels before a control sit in a column about 104 px wide (roughly 15 characters): a longer label pushes its control to the right.

panel-zoom-speed = Velocidad de zoom
panel-zoom-speed-hover = Velocidad del zoom continuo al mantener Space (1× ≈ 2× cada 1,5 s).
panel-snap-zoom = Ajustar el zoom a los elementos
panel-snap-zoom-hover = Al acercarte con la rueda del ratón o con Space, atraer el zoom hacia un minibrot (o el eje real) próximo al cursor, de modo que se deslice bajo el cursor y el zoom aterrice en él. Cuanto más cerca está el elemento, más fuerte es la atracción. Solo Mandelbrot, vista única.
panel-click-zoom = Clic para zoom
panel-click-zoom-hover = Si está activado, un clic izquierdo en la vista se sumerge según el factor de abajo (el clic derecho retrocede), recentrando la vista en el punto pulsado. Arrastrar sigue desplazando; Shift+arrastrar / arrastrar con el botón derecho siguen haciendo zoom de recuadro. Backspace deshace un clic. En la vista doble funciona en cualquiera de los dos paneles, y fijar la c de Julia pasa a Ctrl+clic mientras esto está activado. Mantén Shift para mostrar una lupa bajo el cursor con la vista a la que te llevaría el clic, y haz clic mientras la mantienes para tomar ese punto; Shift+ARRASTRAR sigue haciendo zoom de recuadro.
# The click-to-zoom factor, followed by buttons 2×, 4×, 10×…
panel-click-zoom-factor = Factor
panel-click-zoom-snap = Ajustar al centro más cercano
panel-click-zoom-snap-hover = Tras el salto, calcular el núcleo del minibrot más cercano y colocarse exactamente sobre él, para que aterrices en el elemento y no cerca de él. El zoom no cambia: eso lo eligió el clic. Solo Mandelbrot. El cálculo se ejecuta en segundo plano: la vista salta de inmediato y se corrige cuando llega la respuesta, lo que a profundidad extrema puede tardar unos segundos.

## The control panel: Coloring
## Labels before a control sit in a column about 104 px wide (roughly 15 characters).

panel-method = Método
panel-method-hover = Cómo se convierten en color los datos de escape. Franjas / desigualdad triangular / trampa de órbita / descomposición revelan la estructura de la órbita; distancia sombrea según la proximidad a la frontera.
panel-stripe-density = Densidad de franjas
panel-stripe-tail = Solo la cola (zoom profundo)
panel-stripe-tail-hover = Promediar el término de franjas sobre las últimas N iteraciones de cada órbita (una ventana exponencial) en lugar de sobre toda la órbita. A profundidad extrema, cada órbita sigue de cerca a la referencia salvo en sus últimos pocos cientos de iteraciones, así que el promedio de la órbita completa da un solo color para toda la vista; la cola conserva el contraste. Las órbitas más cortas que la ventana no se ven afectadas. También --stripe-tail N para --render.
panel-tail-length = Longitud de cola
# The unit after the tail length's number: short for "iterations" (the slider reads "64 it").
panel-tail-length-unit = it
panel-tail-length-hover = Longitud de la ventana en iteraciones. Las ventanas cortas (8–32) dan las bandas marcadas clásicas a cualquier profundidad; las más largas vuelven a promediar hacia un color plano, ya que el término de franjas de una órbita caótica larga se compensa en promedio.
panel-trap-shape = Forma de trampa
panel-palette = Paleta
# The two colours of the Binary palette, each beside its colour swatch.
panel-in-set = En el conjunto
panel-out-of-set = Fuera del conjunto
# The two colours of the Duotone palette, each beside its colour swatch.
panel-shadow = Sombra
panel-highlight = Luz
panel-cycle = Ciclo
panel-offset = Desplazamiento
panel-normalize-hover = A profundidad extrema, los recuentos de escape abarcan cientos de miles y un Ciclo fijo da miles de vueltas a la paleta entre píxeles vecinos: un campo denso correcto se ve como ruido moteado. Esto reasigna la paleta al rango de escape medido en la vista (Ciclo fija entonces cuántos barridos de la paleta lo abarcan) CUANDO la vista realmente sufre aliasing; una vista suave se deja tal cual, porque reasignarla cambiaría la imagen para nada. Marca «Ajustar siempre el rango», abajo, para reasignarla de todos modos. Solo con el método suave; equivale a --normalize.
# A checkbox indented under "Normalize deep colors" (also in the Color menu).
panel-fit-range = Ajustar siempre el rango
panel-fit-range-hover =
    Reasignar la paleta al rango de escape medido en la vista incluso cuando la vista no sufre aliasing; la casilla de arriba solo reasigna cuando un campo denso se vería, si no, como un moteado.

    Usa esto cuando quieras que la paleta abarque lo que hay en pantalla, más que corregir ruido. Sigue necesitando una medición, así que no hace nada durante el momento o dos antes de que la vista se asiente, y nada en absoluto en una vista sin píxeles que escapen.
# A checkbox indented under "Always fit range" (also in the Color menu).
panel-log-scale = Escala de color logarítmica
panel-log-scale-hover =
    Repartir la paleta según el LOGARITMO del valor de escape en lugar de linealmente. Los recuentos de escape se amontonan hacia el extremo alto en profundidad, así que una asignación lineal gasta la mayor parte de la paleta en una fina capa cerca de la frontera y aplana todo lo demás.

    Se aplica a la asignación normalizada, que es lo que activa «Normalizar colores profundos» (o --normalize en una exportación). Un rango muy sesgado usa la asignación logarítmica de todos modos, esté marcada o no esta casilla.
# The same checkbox's tooltip while it is greyed out (normalization is off).
panel-log-scale-off-hover =
    Requiere normalización: esto elige CÓMO se asigna la paleta al rango de escape medido, así que con «Normalizar colores profundos» desactivado (y sin exportación con --normalize) no hay ninguna asignación que cambiar y no alteraría ni un solo píxel.

    Activa «Normalizar colores profundos» para usarla.
# The palette animation: its kind, then its speed.
panel-animate = Animar
panel-anim-speed = Velocidad
panel-anim-speed-hover = Velocidad del ciclo: ciclos de desplazamiento de color por segundo o, con Degradados aleatorios, cambios de degradado por segundo.
panel-shuffle-gradient = Barajar degradado

## The control panel: Quality

panel-auto-iter = Autoescalar iteraciones con el zoom
# The iteration-count label while auto-scaling is on (the count is then a base the zoom raises).
panel-iterations-base = Iteraciones (base)
panel-iterations = Iteraciones
panel-iterations-hover = Número base de iteraciones. Con Autoescalar activado, el número efectivo sube con la profundidad del zoom. Mientras te mueves, la vista previa limita las iteraciones a un valor bajo (50.000) para responder con agilidad, y luego se afina hasta el número completo cuando la vista se asienta; así, los bordes profundos se ven suaves durante el movimiento y se resuelven cuando te detienes.
# A warning under the iteration count. $count: the iterations the settled preview renders.
panel-res-limited = A esta profundidad, la vista previa asentada renderiza { $count } iteraciones a resolución reducida para seguir respondiendo. Exporta para obtener el detalle a resolución completa.
panel-anti-alias = Antialiasing
# Anti-aliasing off (the other choices are 2×, 3×, 4×, 8×).
panel-anti-alias-off = Desactivado
panel-anti-alias-hover = Supermuestreo para imágenes fijas (se aplica cuando la vista se asienta). Un valor más alto doma el fino «polvo» exterior a costa de tiempo de renderizado.

## The control panel: Effects

panel-relief-lighting = Iluminación de relieve 3D
panel-relief-lighting-hover = Sombrear la superficie con la pendiente de la estimación de distancia: un aspecto en relieve e iluminado. (Familias holomorfas: Mandelbrot / Multibrot.)
panel-light-angle = Ángulo de la luz
panel-relief = Relieve
panel-relief-hover = Menor = relieve más nítido; mayor = más suave/plano.
panel-rotate-light = Girar la luz
panel-rotate-light-hover = Hacer girar la dirección de la luz con el tiempo (usa el control Velocidad).
panel-distance-glow = Brillo de distancia
panel-distance-glow-hover = Bandas de contorno brillantes de la estimación de distancia que se densifican en filamentos luminosos cerca de la frontera. (Familias holomorfas.)
panel-glow = Brillo
panel-band-width = Ancho de bandas
panel-band-width-hover = Espaciado de los contornos de distancia (octavas por banda).
panel-animate-glow = Animar el brillo
panel-animate-glow-hover = Hacer fluir las bandas de brillo con el tiempo (usa el control Velocidad).

## The control panel: Overlays

panel-orbits-hover = Dibujar la trayectoria de iteración del punto bajo el cursor.
panel-orbit-anim = Animar (punto viajero)
panel-orbit-anim-hover = Lanzar un punto que cambia de color y recorre la órbita a toda velocidad.
panel-orbit-speed = Velocidad de órbita
panel-minimap-hover = Una pequeña vista general de la vista inicial con un marcador «estás aquí» y la profundidad del zoom. Haz clic en ella para saltar a una región.

## The control panel: Advanced
## Labels before a control sit in a column about 104 px wide; most of these are already wider in English.

# Small headings of the section's two groups.
panel-accelerators = Aceleradores
panel-performance-tuning = Ajuste de rendimiento
panel-bla = Aceleración BLA (zoom profundo)
panel-bla-hover = Aproximación bilineal: saltar iteraciones a lo largo de toda la órbita a profundidad extrema (Mandelbrot floatexp, ≥1e28×): renderizado en GPU ~5× más rápido, resultado idéntico (verificado por la autoprueba). Activada de forma predeterminada; desactívala para comparar o si encuentras un artefacto.
panel-series-approx = Aproximación por series
panel-series-approx-hover = Inicializar la perturbación a partir de un polinomio para saltarse las primeras iteraciones donde BLA no está activo (profundidades df32, Multibrot, BLA desactivado). Resultado idéntico; desactívala para comparar.
panel-glitch-correct = Corrección de glitches (exportación)
panel-glitch-correct-hover = Corrección de glitches con varias referencias para las imágenes exportadas: detecta los glitches de perturbación y vuelve a renderizar esos píxeles con referencias adicionales hasta que quedan limpios. Activada de forma predeterminada. Exportaciones de hasta ~32 MP (coloreado sin datos auxiliares); las imágenes más grandes y la vista en vivo usan la vía simple. Nunca se usa en vistas de Mandelbrot o Multibrot (fuera del modo Julia): comparada con un cálculo exacto, ahí no corregía nada y costaba hasta 5 veces el tiempo de exportación.
panel-dive-limit = Límite de inmersión del zoom automático
panel-dive-limit-hover = Profundidad a la que se detiene el zoom automático (tecla A). Hasta ~1e271× se desliza con suavidad; más allá, pasa a una inmersión escalonada y entrecortada para alcanzar rápidamente la profundidad extrema.
# Followed by two choices, Speed and Quality.
panel-autopilot-priority = Prioridad del zoom automático
panel-priority-speed-hover = Hacer zoom a la velocidad fijada. La pantalla se refresca en cuanto hay listo un fotograma con detalle, a la resolución que permita que sigan llegando refrescos (por debajo del mínimo de Resolución mínima en movimiento si el zoom va más rápido que el renderizado); entre refrescos, el último fotograma sigue al zoom y se suaviza.
panel-priority-quality-hover = Cada refresco es un fotograma a resolución completa totalmente resuelto, y el zoom se frena lo necesario para que el fotograma en pantalla nunca se amplíe mucho antes de que llegue el siguiente. Una inmersión más lenta y nítida en todo momento.
# Followed by two choices, Detail and Misiurewicz point.
panel-autopilot-target = Objetivo del zoom automático
panel-target-detail-hover = Sumergirse hacia la estructura más rica del borde del fractal en la vista, volviendo a elegir el objetivo a medida que avanza la inmersión.
panel-target-misiurewicz-hover = Buscar el punto de Misiurewicz más cercano (un centro de espiral cuya estructura se repite para siempre) y sumergirse exactamente en él. La inmersión primero se mantiene quieta mientras el punto se resuelve hasta la profundidad del límite de inmersión: un momento a profundidades normales, más tiempo más allá de 1e1000×. Solo en el conjunto de Mandelbrot.
panel-work-budget = Presupuesto de renderizado en vivo
panel-work-budget-hover = Detalle frente a velocidad en la vista en vivo con zoom profundo. Un valor más alto renderiza a una resolución más completa (más nítido, con menos aspecto «blando» de imagen ampliada) pero reduce la frecuencia de fotogramas, y los valores muy altos se arriesgan a un breve bloqueo de la GPU en los fotogramas más pesados. Las exportaciones siempre son a resolución completa, independientemente de esto.
panel-min-motion-res = Resolución mínima en movimiento
panel-min-motion-res-hover = La resolución más baja a la que puede bajar la vista en vivo mientras haces zoom o te desplazas de forma continua con zoom profundo. Un valor más alto evita que una inmersión rápida se vea pixelada (más nítida) a costa de frecuencia de fotogramas: al 100%, los refrescos de detalle en inmersiones profundas deben renderizarse a coste completo, lo que puede hacer que el zoom profundo avance a saltos o a tirones visibles. Si una inmersión profunda se nota entrecortada, bájalo. Predeterminado: 30%; no afecta a la imagen asentada ni a las exportaciones.
panel-finish-sound = Sonido al terminar un renderizado
panel-finish-sound-hover = Reproducir la melodía de finalización de FRACTINT cuando termina una exportación o el renderizado de un recorrido: tres notas ascendentes (1047, 1109, 1175 Hz durante 100 ms cada una), exactamente como las codificaba el original de DOS, sintetizadas como una onda cuadrada de banda limitada y con un filtro de paso alto para que suenen como el pequeño cono del altavoz del PC que las tocaba.
panel-prefer-detail = Preferir detalle al hacer zoom
panel-prefer-detail-hover = Mientras haces zoom o te desplazas, seguir mostrando el último fotograma con todo el detalle (escalado y desplazado para seguir el movimiento) en lugar de volver a renderizar con calidad reducida en cada fotograma. La vista se renderiza por completo cuando el movimiento se detiene. Desactivado: el movimiento se renderiza en vivo a resolución reducida (más fluido, más tosco). Las vistas poco profundas (modo directo) siempre se renderizan en vivo: son baratas y nítidas en cada fotograma de todos modos.

## The control panel: About (the current fractal)

# A link to the fractal's reference page, followed by an arrow.
panel-about-reference = Referencia

## The control panel: Advanced ▸ Second graphics card

# A label before a drop-down (wider than the panel's label column already).
gpu2-label = Segunda tarjeta gráfica
gpu2-off = Desactivada
# Another device on the same graphics card as the window (the default).
gpu2-same-card = Misma tarjeta
# A card the list does not name. $number: the card's number.
gpu2-card-n = Tarjeta { $number }
gpu2-finding = Buscando las tarjetas gráficas de este equipo…
gpu2-none = Este equipo no tiene ninguna otra tarjeta gráfica.
gpu2-hover =
    Otro dispositivo gráfico renderiza fotogramas de una vista profunda mientras se mueve, y algunas de las muestras adicionales que la definen cuando se detiene, para que la imagen siga el ritmo de un zoom y alcance antes su calidad final. La tarjeta que dibuja la ventana sigue dibujándolo todo.

    Otra tarjeta añade su propia velocidad. Dos modelos de tarjeta distintos dibujan unos pocos píxeles de forma ligeramente distinta; si la segunda tarjeta falla, la vista sigue con una.

    «Misma tarjeta» (el valor predeterminado) abre un segundo dispositivo en esta tarjeta. No añade velocidad, pero renderiza sin esperar a los fotogramas de la pantalla. Comparte la tarjeta: un fallo que reinicie la tarjeta cierra la aplicación, igual que un fallo del propio renderizado de la ventana, y en el siguiente inicio se ofrece desactivarla.

## The view (canvas)

# A notice after Ctrl+click pins the Julia c at a deep zoom.
canvas-julia-pin-quantized = c de Julia fijada, pero se guarda en doble precisión, así que a esta profundidad todo el panel se redondea a un único valor y la fijación cae en el centro de la vista. Aléjate para apuntarla.
# A small caption in the top-left corner of the view while the orbit overlay is fitted to the view (its "Normalize (fit to view)" option).
canvas-orbit-normalized = órbita (normalizada)
# The minimap's tooltip.
canvas-minimap-hover = Vista general / estás aquí. Arrastra para desplazarte con el zoom actual, usa la rueda para acercar y alejar, haz clic para saltar a esa región (zoom inicial).

## The timestamp overlay (View ▸ Show timestamp)

# Under the large elapsed-time clock. $frame: the frame number (as the log writes it, without grouping).
overlay-timestamp-frame = fotograma { $frame }

## File ▸ Settings ▸ Reference cache…

cache-title = Caché de referencias
cache-intro = A profundidad extrema, la parte lenta de una vista es su órbita de referencia: hasta una hora de aritmética de precisión arbitraria. Fractadyne guarda aquí las órbitas terminadas, así que volver a una ubicación que ya visitaste, o hacer zoom en algún lugar cercano, tarda en cambio segundos. Una órbita ocupa unos 4 MB.
cache-off = Desactivada en esta ejecución (una invocación de tarea, o --no-orbit-cache). No se lee ni se escribe nada hasta el siguiente inicio normal.
# Small headings of the window's parts.
cache-location = Ubicación
cache-usage = Uso
cache-limit = Límite
cache-no-config = (no hay carpeta de configuración disponible)
cache-open-folder = Abrir carpeta
# The text on the usage bar. $used and $limit: sizes such as "1.2 GB"; $count: the orbits kept.
cache-usage-bar =
    { $count ->
        [one] { $used } de { $limit } — una órbita
       *[other] { $used } de { $limit } — { $count } órbitas
    }
cache-limit-hover = Cuando la caché supera este tamaño, se eliminan primero las órbitas que resultaron MÁS BARATAS de construir: una órbita profunda que tardó una hora sobrevive a cualquier número de órbitas rápidas, por recientemente que se hayan usado.
# Beside the limit. $count: about how many orbits of the live view's length fit in it.
cache-limit-estimate =
    { $count ->
        [one] ≈ una órbita de la longitud máxima en vivo
       *[other] ≈ { $count } órbitas de la longitud máxima en vivo
    }
cache-kept = Guardadas (primero las más valiosas)
# Column headers of the list of kept orbits.
cache-col-iterations = iteraciones
cache-col-precision = precisión
cache-col-size = tamaño
cache-col-last-used = último uso
# The precision column. $bits: the precision in bits.
cache-row-bits = { $bits } bits
# Under the list. $count: the orbits not listed.
cache-rows-more = … y { $count } más
# When an orbit was last used (the "last used" column).
cache-ago-now = ahora mismo
# $count: minutes.
cache-ago-minutes = hace { $count } min
# $count: hours.
cache-ago-hours = hace { $count } h
# $count: days.
cache-ago-days = hace { $count } d
# Asked before clearing. $count: the orbits; $size: their total size, such as "1.2 GB".
cache-confirm =
    { $count ->
        [one] ¿Eliminar una órbita ({ $size })? Reconstruirla cuesta tiempo, no datos.
       *[other] ¿Eliminar { $count } órbitas ({ $size })? Reconstruirlas cuesta tiempo, no datos.
    }
cache-delete = Eliminar
cache-cancel = Cancelar
cache-clear = Vaciar la caché…
cache-clear-hover = Elimina todas las órbitas guardadas. Pregunta antes.
cache-close = Cerrar
# $count: the orbits removed.
cache-cleared =
    { $count ->
        [one] Caché de referencias vaciada: se eliminó una órbita.
       *[other] Caché de referencias vaciada: se eliminaron { $count } órbitas.
    }
# $why: the system's error message.
cache-clear-failed = No se pudo vaciar la caché de referencias: { $why }

## Help ▸ Diagnostics…

diag-title = Diagnóstico
diag-intro = Ejecuta una prueba para comprobar que Fractadyne funciona correctamente en tu hardware. Los resultados se guardan en tu equipo y se pueden adjuntar a un informe de problema: nada se envía a ninguna parte por sí solo.
# $status: which arbitrary-precision library is in use, as the program reports it (in English).
diag-arithmetic = Aritmética de zoom profundo: { $status }
# The three tests' buttons, each followed by a sentence saying what it does.
diag-run-selftest = Ejecutar autoprueba
diag-run-uitest = Ejecutar prueba de interfaz
diag-run-gputest = Ejecutar comprobación aritmética de la GPU
diag-blurb-selftest = Comprueba las matemáticas y el renderizado en tu GPU frente a resultados que se saben correctos. Tarda unos 15 segundos.
diag-blurb-uitest = Recorre la interfaz y la vista en vivo, haciendo capturas de pantalla a varias profundidades de zoom. Tarda uno o dos minutos y abre ventanas mientras se ejecuta.
diag-blurb-gputest = Comprueba la aritmética double-float del shader frente a valores que se saben correctos en cada backend gráfico que ofrece tu equipo. Tarda unos segundos y no abre ninguna ventana. Si informa de un fallo, es un hallazgo real que nos gustaría ver: adjúntalo a un informe, por favor.
# While a test runs. $count: the checks finished so far.
diag-checks-done =
    { $count ->
        [one] una comprobación hecha
       *[other] { $count } comprobaciones hechas
    }
# A finished test's result. $test: the test's name (Self-test, UI test, GPU arithmetic check).
diag-verdict-captured = { $test } — resultado registrado
diag-verdict-passed = { $test } — superada
diag-verdict-problems = { $test } — informó de problemas
# Under the GPU arithmetic check's result.
diag-informational-note = Esta no tiene aprobado ni suspenso: registra cómo calcula tu GPU, y un fallo notificado es un hallazgo real, no un defecto de tu equipo. En cualquier caso, adjuntarla a un informe es lo más útil que puedes hacer con ella.
# Under a test's result when it reported problems.
diag-problems-note = Es de esperar que haya algunas diferencias en hardware distinto de la GPU de referencia, sobre todo en las comparaciones de imágenes. Adjuntar esto a un informe de problema es lo más útil que puedes hacer con ello.
diag-open-results = Abrir resultados
diag-attach = Adjuntar a un informe de problema…
diag-attach-hover = Abre el cuadro de informe con este resultado incluido, para que la incidencia lleve una prueba validada por la máquina y no solo una descripción
diag-console = Mostrar el diagnóstico en la consola
diag-console-hover = Mostrar las líneas [fd-…] en stderr a partir de ahora. Desactivado de forma predeterminada cuando Fractadyne se inicia sin argumentos. Esto no cambia nada de lo que se REGISTRA (el archivo de registro siempre recibe todas las líneas), así que déjalo desactivado salvo que estés mirando una terminal. Solo dura esta sesión; usa la opción --console o FRACTADYNE_CONSOLE=1 para capturar también el inicio.
diag-close = Cerrar
# Errors starting a test. $why: the system's error message; $dir: a folder.
diag-error-no-exe = No se encuentra el ejecutable: { $why }
diag-error-create-dir = No se puede crear { $dir }: { $why }
diag-error-start = No se pudo iniciar la prueba: { $why }

## Navigate ▸ Misiurewicz explorer…

misi-title = Explorador de Misiurewicz
misi-mandelbrot-only = La exploración de Misiurewicz es solo para Mandelbrot: cambia de fractal para usarla.
# While the gallery is being built. $seconds: the time so far, such as "3".
misi-solving = Resolviendo las familias de tipos pequeños ({ $seconds }s)…
# Above the gallery. $total: all points; $curated: the hand-picked ones; $found: the ones the sweep found.
misi-summary = { $total } puntos: { $curated } seleccionados a mano y { $found } encontrados por un barrido de Newton con semillas de los tipos pequeños (semiplano superior; una rejilla gruesa encuentra la mayoría de las cuencas, no está demostrado que todas).
# The name of a point the sweep found. $kind: its type, such as "(3,1)"; $point: its coordinate, such as "-0.1011+0.9563i".
misi-entry-name = { $kind } en { $point }
# Under each thumbnail. $kind: the point's type, such as "(3,1)"; $octaves: the zoom period in octaves, such as "3.17"; $twist: the twist per repeat in degrees, such as "+12.5". "oct" is short for octaves.
misi-entry-readout = { $kind } · { $octaves } oct/repetición · { $twist }°
# Before the depth field.
misi-zoom-to = Zoom hasta
misi-cancel = Cancelar
misi-solve-jump = Resolver y saltar
misi-solve-jump-hover = Resolver por Newton este punto exacto con la precisión que necesita la profundidad pedida (a partir de su coordenada guardada como semilla) y luego saltar directamente allí. Las profundidades grandes tardan más: el cálculo es de precisión arbitraria.
misi-select-hint = Selecciona un punto, indica una profundidad y salta. La línea del multiplicador bajo cada miniatura es el aspecto que tendrá una inmersión allí: cuántas octavas hasta que la vista se repite, y cuánto gira en cada repetición.
misi-cancelled = Cálculo cancelado.
# $text: what was typed in the depth field.
misi-bad-zoom = No se puede leer «{ $text }» como aumento (escribe 1e50, 3.2e120, …).
misi-no-kp = Esta entrada no tiene ningún (k,p) detectado con el que resolver.
misi-parse-failed = No se pudo interpretar la coordenada guardada.
# A notice after the jump. $name: the point's name; $zoom: the magnification, such as "1e50"; $seconds: how long the solve took, such as "1.2".
misi-zoomed = Acercado a { $name } — { $zoom }× ({ $seconds }s)
# Added after the notice above when the iteration count is too low. $have: the fixed iteration count; $typical: what the depth typically needs.
misi-iter-note = Las iteraciones están fijadas en { $have }; esta profundidad suele necesitar ~{ $typical }: auméntalas (o activa las iteraciones automáticas) si la vista se renderiza plana.
# $name: the point's name; $why: why the solve failed (a technical word).
misi-solve-missed = El cálculo profundo no llegó a { $name }: { $why }. El punto guardado sigue siendo exacto a su propia profundidad: prueba un objetivo menos profundo.

## Life (Game of Life): the side panel

# Shown when the graphics card cannot run the Life view.
life-no-compute = Este adaptador gráfico no puede ejecutar Life (no tiene compute shaders).
# Row label: the pattern list (pattern names stay in English).
life-pattern = Patrón
life-open = Abrir…
life-open-hover = Abrir un archivo de patrón RLE, .cells o Life 1.05/1.06.
life-pattern-text = Texto del patrón…
life-pattern-text-hover = Pegar un patrón (RLE tal como lo da LifeWiki, o texto plano).
life-save = Guardar…
life-save-hover = Guardar el universo tal como está ahora, en RLE.
# Row label: the rule list and the typed rule (B3/S23 notation).
life-rule = Regla
# Shown in the rule list when the rule in use is none of the library's.
life-rule-custom = personalizada
life-apply = Aplicar
# Playback buttons (a pause or play symbol is shown before them).
life-pause = Pausar
life-play = Reproducir
life-play-hover = Ejecutar el universo (Enter).
life-step = Paso
life-step-hover = Avanzar una generación (N).
# Tooltip of the button beside Step, which reads "+16", "+1,024"…: that many generations.
life-stride-hover = Avanzar este número de generaciones (Shift+N).
life-reset = Restablecer
life-reset-hover = Volver al patrón tal como se abrió (generación 0).
life-speed = Velocidad
# After the speed slider's number: generations a second. Short.
life-speed-unit = gen/s
# Row label: how many generations the "+n" button advances.
life-stride = Salto
# The panel's numbers.
life-generation = Generación
life-population = Población
life-tiles = Teselas
# $count: the tiles stored; $capacity: how many the GPU pool holds.
life-tiles-of = { $count } de { $capacity }
life-tiles-hover = Teselas guardadas de 64×64 celdas (4 KiB cada una) y la capacidad del pool de la GPU.
# Row label: what a left-drag in the view does (Pan or Draw).
life-mouse = Ratón
life-pan = Desplazar
life-pan-hover = Arrastra para mover la vista.
life-draw = Dibujar
# The Draw tool's tooltip; the toolbar's shows it after "Draw (D):", so it starts lowercase.
life-draw-hint = haz clic en una celda para invertirla; arrastra para pintar: un trazo que empieza en una celda muerta dibuja, y uno que empieza en una celda viva borra. Un universo en marcha se pausa mientras dibujas y sigue cuando sueltas.
life-undo = Deshacer
life-undo-hover = Volver al universo anterior a la última edición, relleno, borrado o cambio de regla (Ctrl+Z mientras dibujas).
life-grid = Líneas de rejilla
life-grid-hover = Bordes de las celdas con un zoom de 8 píxeles por celda o más.
# A random fill: "Random" labels the density field, "seed" the seed field after it.
life-random = Aleatorio
life-density-hover = La fracción de celdas que un relleno deja vivas.
life-seed = semilla
life-fill-view = Rellenar la vista
# $cells: the largest side of the filled area, in cells (2,048).
life-fill-view-hover = Rellenar al azar la zona visible (hasta { $cells } × { $cells } celdas alrededor de su centro).
life-clear = Vaciar
life-clear-hover = Matar todas las celdas (Deshacer las recupera).

## Life: notices in the panel, and file dialogs

# $path: the file written.
life-saved = Guardado { $path }
# $error: why (from the system, in English).
life-save-failed = Error al guardar: { $error }
# The universe stopped running. $error: why (in English).
life-stopped = Detenido: { $error }
# A pattern file named a rule that does not read. $error: why (in English); $rule: the rule kept, in B3/S23 notation.
life-rule-not-understood = No se entendió la regla del patrón ({ $error }); se mantuvo { $rule }
# $path: the file; $error: why (in English).
life-open-failed = No se pudo abrir { $path }: { $error }
# File-dialog filter names.
life-filter-patterns = Patrones de Life
life-filter-rle = Patrón con codificación por longitud de serie

## Life: the toolbar (its buttons are icons; these are their tooltips)

life-toolbar-pause-hover = Pausar el universo (Enter)
life-toolbar-play-hover = Ejecutar el universo (Enter)
# $stride: the generations a Shift+click steps.
life-toolbar-step-hover = Avanzar una generación (N); Shift+clic o Shift+N avanza { $stride }
life-toolbar-reset-hover = Volver al patrón tal como se abrió (generación 0)
# $hint: the Draw tool's tooltip (life-draw-hint).
life-toolbar-draw-hover = Dibujar (D): { $hint }

## Life: the Pattern text window

life-text-title = Texto del patrón
life-text-intro = Pega un patrón: RLE (tal como lo dan LifeWiki y Golly), .cells en texto plano, o Life 1.05 / 1.06.
life-text-load = Cargar
life-text-current = Actual como RLE
life-text-current-hover = Rellenar el cuadro con el universo tal como se cargó por última vez.

## L-systems: the side panel

# Row label: the system list (system names stay in English).
lsys-system = Sistema
lsys-open = Abrir…
lsys-open-hover = Abrir un archivo .l de Fractint o un sistema .lsys.
lsys-edit = Editar…
lsys-edit-hover = El texto del sistema: axioma, producciones, ángulo; edítalo y aplícalo.
lsys-save = Guardar…
lsys-save-hover = Guardar el sistema como archivo de texto .lsys.
# Tooltip of the "SVG…" button.
lsys-svg-hover = Exportar la vista como dibujo SVG: sus líneas y formas como vectores, en sus colores.
lsys-follow = El orden sigue al zoom
lsys-follow-hover = El orden sube a medida que te acercas, así que el detalle nunca se acaba.
lsys-follow-fixed-hover = Este sistema no crece según un factor en cada orden, así que se dibuja con un orden fijo.
lsys-order = Orden
lsys-angle = Ángulo
lsys-sweep-hover = Barrer el ángulo, de ida y vuelta: la curva transformándose a medida que gira
# A small button that puts back the system's own angle after the slider moved it.
lsys-system-angle = Ángulo del sistema
lsys-seed = Semilla
lsys-seed-hover = Lo que siguen las elecciones aleatorias: la misma semilla, la misma planta.
lsys-seed-new = Nueva
lsys-seed-new-hover = Otra planta: una semilla nueva.
lsys-line-width = Grosor de línea
# Row label of a percentage slider: how much of the curve is drawn.
lsys-draw-on = Trazado
lsys-draw-on-hover = Qué parte de la curva se dibuja, en el orden en que la dibuja la tortuga
lsys-draw-on-play-hover = La curva dibujándose a sí misma
lsys-colour-by = Colorear por
# The colourings, in the "Colour by" list.
lsys-colour-position = posición a lo largo de la curva
lsys-colour-depth = profundidad de rama
lsys-colour-heading = rumbo
# C, < and > are the system's commands that set the colour index: keep them.
lsys-colour-index = índice de color (C, <, >)
lsys-colour-plain = un solo color
# The panel's numbers.
lsys-growth = Crecimiento
lsys-growth-built = construido como palabra, en su orden
lsys-growth-built-hover = Paramétrico o sensible al contexto: la reescritura de un módulo depende de sus números o de sus vecinos, así que la palabra se construye generación a generación (hasta dos millones de módulos) y se dibuja en doble precisión; el orden no sigue al zoom.
# How much the picture grows from one order to the next. $factor: the factor (1.4142).
lsys-growth-factor = ×{ $factor } por orden
lsys-growth-factor-period-2 = ×{ $factor } por orden (los órdenes avanzan de 2 en 2)
lsys-growth-none = ninguno (orden fijo)
lsys-drawn = Dibujado
# $drawn: what was drawn (lsys-drawn-segments, lsys-drawn-shapes or lsys-drawn-segments-shapes); $order: the order.
lsys-drawn-at-order = { $drawn } en el orden { $order }
# The same, when the walk stopped at its segment budget.
lsys-drawn-at-order-budget = { $drawn } en el orden { $order } (presupuesto)
# $segments: the line segments drawn.
lsys-drawn-segments =
    { $segments ->
        [one] { $segments } segmento
       *[other] { $segments } segmentos
    }
# $shapes: the filled shapes drawn.
lsys-drawn-shapes =
    { $shapes ->
        [one] { $shapes } forma rellena
       *[other] { $shapes } formas rellenas
    }
# Both counts. $segments: line segments; $shapes: filled shapes.
lsys-drawn-segments-shapes =
    { $segments ->
        [one] { $segments } segmento
       *[other] { $segments } segmentos
    }, { $shapes ->
        [one] { $shapes } forma rellena
       *[other] { $shapes } formas rellenas
    }
# $ms: the walk's time in milliseconds (12.3).
lsys-drawn-hover = recorrido en { $ms } ms; los subárboles fuera de la vista se omiten, y los que caben en un píxel se dibujan como un solo segmento
# A word too long to build at the order asked for. $wanted: the order asked for; $built: the order drawn.
lsys-drawn-short = La palabra del orden { $wanted } supera los dos millones de módulos: se dibuja en el orden { $built }.
# $order: the deepest order the app draws.
lsys-deepest-order = En el orden más profundo que se dibuja ({ $order }): acercarse más no añade detalle.

## L-systems: notices in the panel and the editor, and file dialogs

# $path: the file; $error: why (in English).
lsys-open-failed = No se pudo abrir { $path }: { $error }
# Why a file was not opened (shown after "Could not open …:").
lsys-file-too-large = mayor de 4 MiB
# Why a Fractint .l file was not opened: it holds no entries. `Name { … }` is the file's syntax: keep it.
lsys-no-entries = no tiene entradas (un archivo .l contiene entradas `Name {"{"} … {"}"}`)
# $path: the file written.
lsys-saved = Guardado { $path }
# $error: why (from the system, in English).
lsys-save-failed = Error al guardar: { $error }
# After File ▸ Export SVG. $path: the file written.
lsys-svg-exported = Exportado { $path }
# $error: why (from the system, in English).
lsys-svg-failed = Error al exportar: { $error }
# File-dialog filter names: opening (any system file) and saving (one system).
lsys-filter-systems = Sistemas L
lsys-filter-system = Sistema L

## L-systems: the toolbar

lsys-toolbar-lower-hover = Bajar el orden (lo fija)
lsys-toolbar-raise-hover = Subir el orden (lo fija)
# Between the − and + buttons, in a fixed-width font. $order: the order, padded to 3 characters so the label keeps its width.
lsys-toolbar-order = orden { $order }
# A toggle: the order follows the zoom. Short.
lsys-toolbar-auto = auto
lsys-toolbar-auto-hover = El orden sigue al zoom
lsys-toolbar-edit-hover = Editar el texto del sistema

## L-systems: the editor window, and a .l file's entries

lsys-editor-title = Sistema L
# The editor's help. The quoted examples ('X = word', 'seed n', 'define R 1.456'…), the commands and "The Algorithmic Beauty of Plants" (a book's title) are the system's syntax and names: keep them as they are.
lsys-editor-help = Una clave por línea: angle (grados, o /n para una división del círculo), axiom, producciones 'X = word' y, opcionalmente, heading, draw / move / variables, colour, order. Un símbolo con alternativas elegidas al azar tiene una línea para cada una, con su peso: 'X (0.3) = word'; 'seed n' elige la planta. En una palabra, {"{"} {"}"} rellena el camino de la tortuga y . añade un vértice. Producciones paramétricas y sensibles al contexto (The Algorithmic Beauty of Plants): 'A(s) : s > 1 = F(s)[+A(s/2)]', 'b < a > c = b', con 'define R 1.456' e 'ignore +-F'; F(l) avanza l, +(a) gira a grados. Una entrada .l de Fractint se puede pegar tal cual.
lsys-editor-apply = Aplicar
lsys-editor-revert = Revertir
lsys-editor-revert-hover = El texto del sistema mostrado.
# The window listing a .l file's entries to pick from. $file: the file's name.
lsys-entries-title = { $file } — entradas
# Tooltip of an entry that does not parse (its error is shown beside it, in English).
lsys-entry-unreadable-hover = Esta entrada no se puede leer.

## Export: opening a view or location (File ▸ Open view or location…)

# File-dialog filter names.
export-filter-view = Vista o ubicación de Fractadyne
export-filter-image = Imagen (PNG / EXR)
export-filter-location = Ubicación (.fdn / .kfr)
# $zoom: the magnification (1.2e148), followed by the magnification sign ×.
export-loaded-location = Ubicación cargada @ { $zoom }×
# $zoom: the magnification; $note: what was noteworthy in the file (in English).
export-loaded-location-note = Cargada @ { $zoom }× — { $note }
# $path: the file.
export-not-location = { $path } no es una ubicación de Fractadyne.
export-file-too-large = Archivo demasiado grande (¿no es una ubicación .fdn?).
# $path: the file; $error: why (from the system, in English).
export-read-failed = No se pudo leer { $path }: { $error }
# A Kalles Fraktaler .kfr file did not import. $error: why (in English).
export-import-failed = Error al importar: { $error }
# $path: an image whose embedded text is not a view.
export-not-view = { $path } tiene metadatos de Fractadyne, pero no una vista.
# $path: the image the view was read from.
export-loaded-view = Vista cargada desde { $path }
# $path: the image; $note: what was noteworthy in it (in English).
export-loaded-view-note = Vista cargada desde { $path } — { $note }
export-no-metadata = Ese archivo no tiene metadatos de vista de Fractadyne incrustados.

## Export: notices after an export or a snapshot
## "File → Export image…" in them names the menu items (menu-file, menu-file-export).

# The export window's status while the image renders.
export-rendering = Renderizando…
export-canceled = Exportación cancelada.
# $error: why (in English).
export-failed = Error al exportar: { $error }
# $size: the image's size in pixels (1920×1080); $path: the file written.
export-saved = Guardado { $size } → { $path }
# An L-system export whose walk stopped at its budget. $size: pixels; $segments: the budget (4,000,000); $path: the file.
export-saved-incomplete = Guardado { $size } (incompleto: se superó el presupuesto de { $segments } segmentos; baja el orden) → { $path }
# A two-panel export written as two files. $path: the first of them.
export-saved-two = Guardados 2 archivos → { $path }
# $size: pixels; $path: the file written.
export-saved-corrected = Guardado { $size } (con glitches corregidos) → { $path }
# $path: the first of the two files.
export-saved-two-corrected = Guardados 2 archivos (con glitches corregidos) → { $path }
# Added after a finished export's notice. $duration: how long it took (8.3s, 1m 04.0s).
export-elapsed = (en { $duration })
export-preparing-deep = Preparando la exportación profunda: construyendo la referencia (puede tardar un rato)…
# $size: pixels (5120×4035); $ss: the supersampling factor; $iterations: the iteration count.
export-large = Exportación grande ({ $size }, supermuestreo { $ss }×, { $iterations } iteraciones): renderizando en segundo plano; en exportaciones de este tamaño se omite la corrección de glitches…
export-life-unavailable = La exportación de imágenes de una vista de Life aún no está disponible: usa una captura de pantalla, o Archivo ▸ Guardar patrón de Life.
export-already-running = Ya hay una exportación en marcha: Archivo → Exportar imagen… muestra su progreso.
# $size: the image's size in pixels.
export-snapshot-background = Instantánea: renderizando { $size } en segundo plano; Archivo → Exportar imagen… muestra el progreso y permite cancelarla.
export-no-gpu = GPU no disponible
# A screen-capture snapshot. $size: pixels; $path: the file written.
export-snapshot-saved = Captura de pantalla de { $size } guardada → { $path }
# $error: why (in English).
export-snapshot-failed = Error en la instantánea: { $error }

## The colour picker (the popup of a colour swatch)

# The third row's label: the colour as hexadecimal bytes. Short (about 6 characters).
picker-row-hex = Hex
picker-hex-hover = Haz clic para escribir un valor hexadecimal
picker-cancel = Cancelar
picker-cancel-hover = Restaurar el color con el que se abrió este selector
picker-ok = Aceptar
picker-ok-hover = Conservar este color
# Starts the screen eyedropper (an icon is shown before it).
picker-pick = Elegir
picker-pick-hover = Tomar un color de cualquier parte de la pantalla
# Tooltip of the disabled Pick button (in the picker and the gradient editor) on Linux and macOS.
picker-eyedropper-unsupported = Tomar un color de otra parte de la pantalla requiere una API de captura de pantalla de la plataforma. Solo la compilación para Windows tiene una; en Linux haría falta una dependencia de X11, y Wayland exige un portal con su propio aviso de permiso.

## The fractal families: what each one is (the info panel's "About" section and the pickers' hover text).
## The formulas themselves (z -> z^2 + c) are written in code and stay as they are.

fractal-mandelbrot-about = El fractal de tiempo de escape canónico: el conjunto de los c para los que la órbita de 0 se mantiene acotada. Su frontera es infinitamente intrincada.
fractal-multibrot3-about = Un conjunto Multibrot: la construcción de Mandelbrot con una potencia más alta. La potencia d da simetría rotacional de orden (d-1).
fractal-multibrot4-about = Multibrot con potencia 4: simetría de orden 3, bulbos anchos.
fractal-multibrot5-about = Multibrot con potencia 5: simetría de orden 4.
fractal-tricorn-about = El Tricorn (Mandelbar): conjuga z en cada paso. Esta aplicación antiholomorfa produce una forma de tres picos.
fractal-burning-ship-about = Se toman los valores absolutos de las partes de z antes de elevar al cuadrado; el resultado se parece a un barco en llamas.
fractal-celtic-about = Un pariente del Burning Ship que toma el valor absoluto solo de la parte real de z^2, lo que produce motivos de nudo celta / corazón.
fractal-buffalo-about = Una variante abs que toma los valores absolutos de ambas componentes de z^2.
fractal-phoenix-about = El Phoenix también usa el iterado anterior, lo que da filamentos como llamas. Prueba su forma Julia con el modo Julia.
fractal-newton-about = La iteración de Newton para buscar las raíces de z^3 = 1, coloreada según la rapidez con que converge cada punto. Un fractal de convergencia (no de escape).
fractal-multibrot6-about = Multibrot con potencia 6: simetría de orden 5.
fractal-multibrot7-about = Multibrot con potencia 7: simetría de orden 6, con los bulbos adelgazando hacia un círculo.
fractal-multibrot8-about = Multibrot con potencia 8: simetría de orden 7.
fractal-burning-ship3-about = El Burning Ship con potencia 3: se toman los valores absolutos y luego se elevan al cubo.
fractal-burning-ship4-about = El Burning Ship con potencia 4.
fractal-burning-ship5-about = El Burning Ship con potencia 5.
fractal-tricorn3-about = El Tricorn (Mandelbar) con potencia 3: simetría de orden 4.
fractal-tricorn4-about = El Tricorn con potencia 4: simetría de orden 5.
fractal-tricorn5-about = El Tricorn con potencia 5: simetría de orden 6.
fractal-celtic3-about = El pliegue Celtic con potencia 3: el valor absoluto de la parte real de z^3.
fractal-celtic4-about = El pliegue Celtic con potencia 4.
fractal-celtic5-about = El pliegue Celtic con potencia 5.
fractal-buffalo3-about = El Buffalo con potencia 3: los valores absolutos de ambas partes de z^3.
fractal-buffalo4-about = El Buffalo con potencia 4.
fractal-buffalo5-about = El Buffalo con potencia 5.
fractal-custom-about = Una fórmula escrita en expresiones al estilo de Fractint (Fractal ▸ Fórmula personalizada…). La mayoría admiten zoom profundo por perturbación, como las familias integradas; el cuadro de la fórmula indica si la tuya lo admite.
fractal-life-about = El Juego de la vida de Conway y sus parientes en un plano ilimitado: reglas Generations, reglas no totalísticas (Hensel), archivos de patrones. Reproducir, avanzar, dibujar.
fractal-lsystem-about = Sistemas de Lindenmayer: curvas, curvas que llenan el espacio, islas y plantas dibujadas por una tortuga a partir de una gramática de reescritura. El orden sigue al zoom, así que el detalle nunca se acaba. Los archivos .l de Fractint se abren directamente.

# Shown where the other families show their formula (z -> z^2 + c).
fractal-custom-formula = z -> la fórmula que escribas
# Shown where the other families show their formula. "B3/S23" is the rule's notation: keep it.
fractal-life-formula = B3/S23: nace con 3 vecinas vivas, sobrevive con 2 o 3
# Shown where the other families show their formula. "F = F+F--F+F" is the grammar's notation: keep it.
fractal-lsystem-formula = F = F+F--F+F: cada símbolo se reescribe y luego lo dibuja una tortuga

# The last paragraph of the picker's hover text for a family that renders only on the direct path.
fractal-no-deep-zoom = Solo renderizado directo: esta familia no tiene zoom profundo.

## Names of settings, shown in the Settings menu and the welcome window.

theme-dark = Oscuro
theme-light = Claro
# The update track that follows only stable releases.
update-track-stable = Estable
# The update track that also offers pre-releases (betas).
update-track-beta = Beta (versiones preliminares)

## The status bar (its hover texts; the readouts follow in their own section)

status-period-hover = Período del minibrot al que saltaste (pulsa M, o Ir a la ubicación ▸ Ir a un elemento). Se mantiene mientras la vista está sobre el elemento y desaparece cuando te alejas.

## The playback transport: the player over the view while a tour plays.

# $close: the close button's icon, as drawn on that button.
play-finished-hover = Terminado: retrocede con la barra, o { $close } para cerrar
# $kind: "benchmark" or "script" (the variants hold the wording); $name: the tour's name.
play-playing-hover =
    { $kind ->
        [benchmark] Reproduciendo la prueba de rendimiento «{ $name }»
       *[other] Reproduciendo el guion «{ $name }»
    }
play-restart = Reiniciar desde el principio
play-back = Retroceder 10 segundos
play-again = Volver a reproducir desde el principio
play-resume = Reanudar
play-pause = Pausar
play-stop = Detener y rebobinar al principio
play-forward = Avanzar 10 segundos
play-speed-hover = Velocidad de reproducción (haz clic para alternar 0,5× / 1× / 2× / 4×)
play-render = Renderizar este guion como secuencia de fotogramas…
play-render-none = La prueba de rendimiento integrada no tiene ningún archivo de guion que renderizar
play-open = Reproducir otro guion…
play-loop = Repetir el recorrido cuando llegue al final
play-close = Cerrar el reproductor y restaurar tus propios ajustes de vista
# A notice beside the scrub bar while the renderer holds the tour's clock. Keep it short.
play-waiting = esperando detalle
# "[playback] pace" is a setting in the script file: keep it as written.
play-waiting-hover = El reloj del recorrido está en pausa mientras el renderizador resuelve esta vista (construcción de la referencia / presupuesto de iteraciones en aumento). La reproducción se reanuda sola; consulta [playback] pace en el guion.
play-scrub = Arrastra para desplazarte por el recorrido
# $behind: how far the script clock is behind real time, as minutes:seconds (1:05).
play-scrub-behind =
    Arrastra para desplazarte por el recorrido
    El reloj del guion va { $behind } por detrás del tiempo real (el regulador de ritmo ralentizó el recorrido para que el renderizador pudiera seguirlo). La marca superior indica dónde estaría la reproducción en tiempo real.

## The autopilot's notices (toasts). Where a notice has two lines, the first is the verdict and the second what to do about it.

autopilot-on-detail = Piloto automático activado: sumergiéndose hacia el detalle (cualquier acción lo detiene)
autopilot-off = Piloto automático desactivado
autopilot-misi-needs-mandelbrot =
    Piloto automático: un objetivo de Misiurewicz necesita el conjunto de Mandelbrot
    Se sumerge hacia el detalle en su lugar
autopilot-on-misi =
    Piloto automático activado: buscando el punto de Misiurewicz más cercano…
    Cualquier acción lo detiene
# $seconds: how long the search has run, in whole seconds.
autopilot-misi-finding =
    Piloto automático: buscando el punto de Misiurewicz más cercano…
    { $seconds } s · cualquier acción lo detiene
autopilot-misi-not-found =
    Piloto automático: no se encontró ningún punto de Misiurewicz cerca de esta vista
    Puede que el buscador no llegue a esta profundidad: prueba el objetivo Detalle
# ($preperiod,$period) names the point, as mathematicians write it: keep the notation. $widths: a power of two (2^$widths screen widths).
autopilot-misi-off-screen =
    Piloto automático: el punto de Misiurewicz más cercano ({ $preperiod },{ $period }) está fuera de la pantalla
    A ~2^{ $widths } anchos de pantalla: acércate a un centro de espiral
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-not-solved =
    Piloto automático: el punto de Misiurewicz ({ $preperiod },{ $period }) cercano a esta vista no se pudo resolver
    Prueba el objetivo Detalle
autopilot-misi-refused = Piloto automático: cálculo de Misiurewicz rechazado (par no válido)
autopilot-misi-stopped = Piloto automático: el solucionador de Misiurewicz se detuvo de forma inesperada
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-diving =
    Piloto automático: sumergiéndose en Misiurewicz ({ $preperiod },{ $period })
    Cualquier acción lo detiene
# ($preperiod,$period) names the point: keep the notation. $octaves: how often the structure repeats (2.37); $twist: its turn in degrees, signed (+12.5).
autopilot-misi-diving-repeat =
    Piloto automático: sumergiéndose en Misiurewicz ({ $preperiod },{ $period })
    La estructura se repite cada { $octaves } octavas, giro de { $twist }° · cualquier acción lo detiene
# $limit: the magnification, as written (1e300).
autopilot-dive-limit = Piloto automático: se alcanzó el límite de inmersión (~{ $limit }×)
# $have: the fixed iteration count; $typical: what this depth typically needs.
autopilot-dead-end-short =
    Piloto automático detenido: no queda nada a lo que apuntar; las iteraciones están fijadas en { $have } y esta profundidad suele necesitar ~{ $typical }
    Activa las iteraciones automáticas o aumenta el número
# $limit: the fixed iteration count.
autopilot-dead-end-cap =
    Piloto automático detenido: no queda nada a lo que apuntar; todo lo que hay en la vista llega al límite fijo de { $limit } iteraciones
    Activa las iteraciones automáticas o aumenta el número
autopilot-dead-end = Piloto automático: no hay detalle por delante (detenido)

## Tours (camera scripts)

# The name of the file type in the file dialog's filter list.
script-file-filter = Guion de Fractadyne (TOML)
# $name: the script's name.
script-finished = Guion terminado: «{ $name }»
# The title of the notice that follows (the file's path and the error, in English).
script-load-failed = No se pudo cargar el guion

## Installing a language file

# The name of the file type in the file dialog's filter list.
language-file-filter = Idioma de Fractadyne

## The notice when the graphics driver's shader cache stops keeping shaders. The text is shown in a fixed-width font with its line breaks as written: break the lines where they read well.

driver-cache-title = El controlador gráfico no conserva los shaders
# $count: how many shaders compiled again; $pipeline: the slowest one's name (a technical id: keep it); $seconds: its time in seconds (7.5).
driver-cache-body =
    Tu controlador gráfico volvió a compilar { $count } de los shaders de
    Fractadyne en esta sesión, aunque ya había compilado cada uno antes (el más
    lento: { $pipeline }, { $seconds } s). Su caché de shaders no conserva los
    shaders compilados, casi siempre porque está llena. Hasta que eso cambie, cada
    exportación, vista profunda o inicio de Fractadyne puede pasar segundos compilando.
# $folder: the cache folder's path; $size: its size in MB (1024). "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia-folder =
    Para solucionarlo: cierra Fractadyne y luego borra el contenido de
      { $folder }   ({ $size } MB)
    El controlador reconstruye la caché. Para que no vuelva a llenarse, aumenta
    Panel de control de NVIDIA > Administrar la configuración 3D > Tamaño de caché del sombreador.
# "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia =
    Para solucionarlo: aumenta Panel de control de NVIDIA > Administrar la configuración 3D >
    Tamaño de caché del sombreador, o vacía la caché de shaders del controlador.
# "AMD Software > Graphics > Advanced > Reset Shader Cache" names AMD's own settings: use the names AMD's software shows in this language.
driver-cache-fix-amd = Para solucionarlo: AMD Software > Gráficos > Avanzado > Restablecer caché de sombreadores.
driver-cache-fix-other = Para solucionarlo: vacía la caché de shaders del controlador gráfico.

## Why a language file could not be installed: each ends the sentence "{ $file } is not a usable language file: …".

language-install-not-utf8 = no es texto UTF-8
language-install-too-large = es demasiado grande
# "fractadyne-<language>.ftl" is the file name to use: keep it as written.
language-install-no-tag = no indica qué idioma es (llámalo fractadyne-<language>.ftl)
language-install-english = el inglés ya está integrado
language-install-empty = no traduce ningún mensaje de esta versión
language-install-no-folder = no hay carpeta de configuración

## The notice after a session that did not end cleanly, when its iteration setting was set aside.

# $base: the iteration count automatic iterations start from.
quarantine-setting-auto = iteraciones automáticas a partir de una base de { $base }
# $count: the fixed iteration count.
quarantine-setting-fixed =
    { $count ->
        [one] { $count } iteración
       *[other] { $count } iteraciones
    }
# $end: "device" when the session lost its graphics device, "unclean" otherwise (the variants hold the wording). $depth: the magnification, as written (1.2e148). $setting and $restored: an iteration setting (quarantine-setting-auto or -fixed).
quarantine-notice =
    { $end ->
        [device] La sesión anterior perdió su dispositivo gráfico a { $depth }× mientras estaba configurada con { $setting }. Esta sesión empezó con { $restored } en su lugar, para que no pueda volver directamente a la misma carga.
       *[other] La sesión anterior no terminó correctamente a { $depth }× mientras estaba configurada con { $setting }. Esta sesión empezó con { $restored } en su lugar, para que no pueda volver directamente a la misma carga.
    }

## The second graphics card's status line, under its setting in the control panel.

gpu2-state-off = Desactivada: la tarjeta gráfica de esta ventana lo renderiza todo.
gpu2-state-starting = Iniciando…
# $name: the card's name (and its graphics API).
gpu2-state-running = En uso: { $name }
# $why: the error, as the graphics driver reports it.
gpu2-state-failed = No se pudo iniciar: { $why }
# $name: the card's name.
gpu2-state-lost = Detenida: se perdió { $name }. Una sola tarjeta a partir de ahora; vuelve a elegirla para reintentarlo.
# $state: one of the lines above. "--worker-gpu" is a command-line option: keep it.
gpu2-state-pinned = { $state } (fijado por --worker-gpu)

## The custom-formula dialog: how deep the formula renders (one line under the formula).

formula-depth-deep = Zoom profundo por perturbación, con rango extendido a partir de 1e28x como en los fractales integrados (cada paso se itera: aún no hay aproximación por series ni BLA para las fórmulas personalizadas).
# $why: what the formula has that keeps it off the deep-zoom path (one of the formula-depth-why messages below).
formula-depth-direct = Renderizado directo ({ $why } aún no tiene forma de zoom profundo): nítido hasta que un paso de c en precisión simple abarca un píxel, hacia 1e4x a 1e5x, y a menos profundidad lejos del origen.
formula-depth-why-init = una sección de inicialización
formula-depth-why-bailout = una prueba de escape propia
formula-depth-why-variable = una variable que se conserva de un paso a otro
formula-depth-why-previous = el iterado anterior
formula-depth-why-comparison = una comparación o un bloque if
# floor, ceil, trunc and round are functions of the formula language: keep them.
formula-depth-why-rounding = el redondeo (floor, ceil, trunc, round)
# "maxit" is a word of the formula language: keep it.
formula-depth-why-maxit = maxit
formula-depth-why-power = una potencia cuyo exponente varía
formula-depth-why-perturbed = un programa ya perturbado

## Image and frame sizes: the presets described in words (the others are standard names: HD 720p, WUXGA, DCI 4K).

# $size: the pixel size (2048×2048).
size-preset-square = Cuadrado — { $size }
# $ratio: the aspect ratio (9:16); $size: the pixel size.
size-preset-portrait = Vertical { $ratio } — { $size }
# $ratio: the aspect ratio (32:9); $size: the pixel size.
size-preset-super-ultrawide = Superultrapanorámico { $ratio } — { $size }

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
fractal-family-custom = Personalizada
# Conway's Game of Life.
fractal-family-life = Life
fractal-family-lsystem = Sistema L
# A family at a power: "Multibrot 3", "Burning Ship 4". $family: a name above; $power: the power.
fractal-with-power = { $family } { $power }

## Names of choices: colouring methods, orbit traps, palette animation, snapshots, angle units.

color-method-smooth = Iteración suave
color-method-stripe = Promedio de franjas
color-method-triangle = Desigualdad triangular
color-method-trap = Trampa de órbita
color-method-distance = Estimación de distancia
color-method-decomposition = Descomposición
# The shape an orbit trap catches the orbit with.
trap-point = Punto
trap-cross = Cruz
trap-circle = Círculo
# Palette animation: none.
palette-anim-off = Desactivada
palette-anim-forward = Adelante
palette-anim-reverse = Atrás
palette-anim-pingpong = Ping-pong
# Palette animation: a new random gradient every so often.
palette-anim-random = Degradados aleatorios
# What the Snapshot button does.
snapshot-mode-ask = Preguntar cada vez
snapshot-mode-screen = Captura de pantalla
snapshot-mode-render = Renderizado completo
# The unit of an angle typed into Go to location's polar fields.
angle-degrees = grados
angle-radians = radianes
angle-turns = vueltas

## Report an issue: the choices. (The report itself is sent in English.)

issue-kind-crash = Cierre inesperado / bloqueo de la aplicación
issue-kind-rendering = Renderizado incorrecto
issue-kind-performance = Problema de rendimiento
issue-kind-ui = Problema de interfaz / usabilidad
issue-kind-feature = Solicitud de función
issue-kind-other = Otro
severity-low = Baja
severity-medium = Media
severity-high = Alta
severity-blocking = Bloqueante
repro-always = Siempre
repro-sometimes = A veces
repro-once = Ocurrió una vez
repro-cannot = No se puede reproducir

## The benchmark's depths and the diagnostics' tests.

# $zoom: the dive's final magnification (1e32).
bench-depth-standard = Estándar ({ $zoom }×)
# $zoom: the dive's final magnification (1e48).
bench-depth-ultra = Ultraprofunda ({ $zoom }×)
diag-test-selftest = Autoprueba
diag-test-uitest = Prueba de interfaz
diag-test-gputest = Comprobación aritmética de la GPU

## The status bar's readouts: each word is followed by a value in a slot of fixed width. Keep the words short (one word each).

status-center = centro
status-cursor = cursor
status-zoom = zoom
# Short for iterations.
status-iter = iter
# The period of the minibrot jumped to.
status-period = período
# A Game of Life view's scale (followed by "1 px = 4 cells" or "8 px / cell").
status-scale = escala
# A Game of Life view's generation. Short.
status-generation = gen
# A Game of Life view's population. Short.
status-population = pobl
# An L-system's order (how many times its rules were applied).
status-order = orden
# How many segments and shapes an L-system drew.
status-drawn = dibujado
# $value: how many cells one pixel covers (4, 9.99e300). "px" is the unit symbol: keep it.
life-scale-cells = 1 px = { $value } celdas
# $value: how many pixels one cell covers (8). "px" is the unit symbol: keep it.
life-scale-pixels = { $value } px / celda

## The status bar's limit diagnostic: a short label (after a warning sign), and what it means when hovered.

# The view is as deep as this location can be rendered. Short.
status-limit-depth = límite prof.
# Every pixel ran out of iterations. Short.
status-limit-exhausted = iter agotadas
# Pixels are capped at the iteration budget. Short.
status-limit-capped = iter limitadas
# $pixels: how many pixels wide one step of the coordinates is; $sharp_to: the magnification the view is sharp to (written out, 1,234,567).
status-limit-direct-detail = Esta vista se renderiza directamente en la GPU en precisión simple, donde aquí un paso de las coordenadas mide { $pixels } píxeles de ancho: los píxeles vecinos calculan el mismo punto, así que la imagen se rompe en bloques. En esta ubicación es nítida hasta unos { $sharp_to }×. Esta fórmula aún no tiene vía de zoom profundo (perturbación), así que acercarse más no muestra más detalle.
# $samples: the GPU buffer's capacity in orbit samples; $iterations: the iterations pixels are limited to.
status-limit-depth-detail = La órbita de referencia alcanzó el límite del búfer de la GPU ({ $samples } muestras) sin escapar, así que aquí los píxeles están limitados a ~{ $iterations } iteraciones y no se puede calcular un detalle más fino en esta vista, ni en vivo ni al exportar. Este es el límite de profundidad actual para esta familia de ubicaciones.
# $budget: the largest iteration budget this view can be given.
status-limit-exhausted-detail = Todos los píxeles se quedaron sin iteraciones incluso con el presupuesto más grande que se le puede dar a esta vista ({ $budget }). O bien la vista está aquí dentro del conjunto (algo normal en profundidad), o bien necesita más iteraciones de las que la aplicación puede ofrecer actualmente; la imagen es la misma en ambos casos, así que el presupuesto se devolvió a uno barato.
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations" is the setting's name in the Rendering panel.
status-limit-capped-max = El { $percent }% de los píxeles se quedó sin iteraciones con el presupuesto de { $budget }, y los que escapan terminan justo en ese techo: la vista no está suficientemente resuelta. Iteraciones ya está al máximo, así que esta vista necesita más de lo que la aplicación puede dar actualmente.
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations" and "Rendering" are the names of a setting and a panel section.
status-limit-capped-raise = El { $percent }% de los píxeles se quedó sin iteraciones con el presupuesto de { $budget }, y los que escapan terminan justo en ese techo: la vista no está suficientemente resuelta. Aumenta Iteraciones (Controles ▸ Calidad) para resolverla.
