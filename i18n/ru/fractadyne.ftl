### Fractadyne user interface: Russian (Русский).
### A draft: machine translation (Claude, by Anthropic) of the English file, 2026-10-23, not yet
### reviewed by a native speaker. Corrections are welcome as pull requests (see i18n/README.md).

## Language data: read by Fractadyne, not shown as messages.

# This language's own name, shown in the language menu.
language-name = Русский
# The same name in English, shown beside it in the menu.
language-english-name = Russian
# The writing system the app must be able to draw (an ISO 15924 code: Latn, Cyrl, Jpan, Hans, Khmr).
language-script = Cyrl
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
language-more = Другие языки…
# The Languages window's title.
language-window-title = Языки
# The Languages window's introduction.
language-window-intro = Интерфейс Fractadyne на другом языке. Язык — это один файл: он поставляется с выпуском, или же установите его здесь. Координаты, файлы и командная строка на любом языке остаются на английском.
# Column headers of the installed-language list.
language-col-language = Язык
language-col-translated = Переведено
language-col-from = Источник
# Where a language comes from.
language-source-builtin = встроенный
language-source-package = этот выпуск
language-source-user = установленный
language-source-dev = рабочая папка
# $percent: how much of the interface the file translates, 0–100.
language-coverage = { $percent }%
# $count: messages in the file that failed the checks and show in English instead.
language-dropped =
    { $count ->
        [one] { $count } сообщение требует проверки
        [few] { $count } сообщения требуют проверки
        [many] { $count } сообщений требуют проверки
       *[other] { $count } сообщения требуют проверки
    }
# Buttons of the Languages window.
language-install = Установить файл языка…
language-install-hover = Выберите файл языка Fractadyne (.ftl). Он будет проверен, а затем скопирован в папку языков в ваших настройках, где сохранится при обновлениях.
language-open-folder = Открыть папку языков
language-reload = Перезагрузить
language-reload-hover = Прочитать файлы языков заново (после того как вы изменили или добавили файл).
language-use = Использовать
# Notices after installing a file. $name: the language's own name; $file: the file's name.
language-installed = Язык { $name } установлен из { $file }.
# $file: the file's name; $why: what was wrong with it.
language-install-failed = { $file } не является пригодным файлом языка: { $why }
# Shown when a language file cannot be loaded and English is used instead. $name: the language.
language-load-failed = Не удалось загрузить язык { $name }, поэтому Fractadyne работает на английском: { $why }
# A language whose writing system needs a font this computer does not have (Japanese or Chinese without a system font). $name: the language, in its own name.
language-font-missing = На этом компьютере не найден шрифт, способный отобразить язык { $name }, поэтому его нельзя показать. Установите шрифт для языка { $name } (например, языковой пакет самой системы) и выберите его снова.
# Shown after a draft translation's name in the language menu and the Languages window. Short.
language-draft = черновик
language-draft-hover = Черновик: этот перевод ещё не проверен носителем языка. Исправления приветствуются — в виде запроса на слияние (pull request) к файлу языка.
# The heading over the decimal-mark choice, in the language menu and the Languages window.
language-decimal-mark = Десятичный разделитель
language-decimal-mark-hover = Как записываются числа: время, размеры и количества. В координатах и увеличениях всегда используется точка, так как их так читают файлы и другие программы.
# $sample: a number written with this language's own punctuation (1,234.5).
language-decimal-language = Как принято в языке ({ $sample })
# $sample: a number written with a decimal point (1,234.5).
language-decimal-point = Точка ({ $sample })
# $sample: a number written with a decimal comma (1.234,5).
language-decimal-comma = Запятая ({ $sample })

## Asked once when the system's language changes

# The window's title.
language-changed-title = Язык
# $new: the system's new language, in its own name.
language-changed-question = Язык вашей системы теперь — { $new }. Использовать его в Fractadyne?
# Buttons. $name: a language's own name.
language-changed-use = Использовать: { $name }
language-changed-keep = Оставить: { $name }

## The welcome window (first run)

welcome-title = Добро пожаловать в Fractadyne
welcome-intro = Программа для глубокого приближения к множеству Мандельброта и его родственникам. Несколько приёмов управления для начала:
# The language row, at the top of the window.
welcome-language = Язык
# Controls: the key or gesture on the left, what it does on the right.
welcome-key-drag = перетаскивание
welcome-key-drag-does = сдвинуть вид
welcome-key-scroll = прокрутка
welcome-key-scroll-does = приблизить / отдалить к курсору
welcome-key-space = удерживать Space
welcome-key-space-does = плавное непрерывное приближение (Shift+Space — отдаление)
# Shown after the click-zoom toolbar icon.
welcome-key-click = затем щелчок
welcome-key-click-does = нырнуть в точку щелчка (панель инструментов)
welcome-key-m-does = перейти к ближайшему миниброту в его собственном масштабе
welcome-landmarks = Для начала перейдите к одному из ориентиров:
welcome-theme = Тема
welcome-watermark = Показывать знак «Fd»
welcome-watermark-hover = Небольшой фирменный знак в правом нижнем углу вида. Если выключить, он исчезнет из живого вида и из экспортированных изображений.
welcome-updates = Обновления
welcome-check-updates = Проверять обновления
welcome-check-updates-hover = При запуске программа запрашивает у GitHub последний выпуск. Ничего не отправляется.
welcome-files = Файлы
welcome-no-config = (нет папки конфигурации)
welcome-open = Открыть
welcome-rendering-on = Рендеринг на
welcome-open-help = Открыть полную справку  (F1)
# $icon: the Settings icon.
welcome-more-in-settings = Больше — в меню Файл → { $icon } Настройки
welcome-get-started = Начать

## Menus: shared

# A percentage in a menu (UI scale). $value: the number.
common-percent = { $value }%
# A state shown after a menu item's name, in brackets.
menu-state-connected = подключено
menu-state-rendering = рендеринг
menu-state-running = работает

## Navigate ▸ Bookmarks (also the toolbar's star)

menu-bookmarks = Закладки
menu-bookmarks-add = Добавить текущий вид
menu-bookmarks-manage = Управлять…

## The fractal picker (Fractal menu and the toolbar's dropdown)

menu-family-powers = Высшие степени
menu-family-automata = Автоматы
menu-family-lsystems = L-системы
menu-fractal-custom = Своя формула…
menu-fractal-library = Библиотека формул…
menu-fractal-library-hover = Сохранённые свои формулы: применяйте, редактируйте, импортируйте и экспортируйте их
# The "Show" group: which picture of the formula is drawn. $name: the formula's name (Mandelbrot, Burning Ship…).
menu-show = Показать
menu-show-set = Множество «{ $name }»
menu-show-set-hover = Плоскость параметров — само множество «{ $name }».

    Каждая его точка — отдельное множество Жюлиа; это карта их всех.
menu-show-julia = Множество Жюлиа
menu-show-julia-hover = Одна точка множества «{ $name }», нарисованная как отдельная картина, — форма, которая получается, если закрепить эту точку.

    Выберите точку, наводя курсор на карту в режиме «Оба, связанные»; щелчок там её закрепляет.
menu-show-julia-none = У формулы «{ $name }» нет свободного параметра, поэтому нет и множества Жюлиа.
menu-show-both = Оба, связанные
menu-show-both-hover = Слева — множество «{ $name }», а справа — множество Жюлиа той точки, над которой находится курсор.

    Ведите курсор по карте и смотрите, как меняется множество Жюлиа, — так быстрее всего понять, как они связаны.
menu-show-both-none = У формулы «{ $name }» нет множества Жюлиа, с которым её можно связать.

## File menu

menu-file = Файл
menu-file-open-view = Открыть вид или местоположение…
menu-file-gallery = Галерея…
menu-file-export = Экспортировать изображение…
menu-file-snapshot = Снимок
menu-file-open-life = Открыть узор Life…
menu-file-open-life-hover = Файл RLE, .cells или Life 1.05/1.06, показанный как вселенная Life.
menu-file-life-text = Текст узора Life…
menu-file-life-text-hover = Вставить узор как текст (RLE в том виде, как его даёт LifeWiki).
menu-file-save-life = Сохранить узор Life…
menu-file-save-life-hover = Вселенная в её нынешнем состоянии, в формате RLE.
menu-file-open-lsystem = Открыть L-систему…
menu-file-open-lsystem-hover = Файл Fractint .l (с записями на выбор) или система .lsys.
menu-file-save-lsystem = Сохранить L-систему…
menu-file-save-lsystem-hover = Текст системы в виде файла .lsys.
menu-file-export-svg = Экспортировать SVG…
menu-file-export-svg-hover = Вид в виде рисунка SVG: его линии и фигуры — векторами, в его цветах.
menu-file-render-client = Клиент рендеринга…
menu-file-render-client-hover = Предоставить GPU этого компьютера рендер-ферме: подключиться к контроллеру (Инструменты ▸ Рендеринг на ферме… на другом компьютере) и рендерить для него кадры.
menu-file-settings = Настройки
menu-file-reset = Сбросить состояние программы…
menu-file-reset-hover = Удалить всё сохранённое состояние (сеанс, закладки, миниатюры) и начать заново. Сначала запрашивает подтверждение.
menu-file-quit = Выйти

## File ▸ Settings

menu-settings-fps = Ограничение частоты кадров
menu-settings-fps-uncapped = Без ограничения
# $fps: frames per second.
menu-settings-fps-n = { $fps } FPS
menu-settings-ui-scale = Масштаб интерфейса (размер шрифта)
menu-settings-theme = Тема
menu-settings-updates = Обновления
menu-settings-updates-stable-hover = Последний стабильный выпуск
menu-settings-updates-beta-hover = Последняя сборка, включая предварительные выпуски
menu-settings-updates-on-launch = Проверять обновления при запуске
menu-settings-updates-on-launch-hover = Иначе проверяйте вручную: Справка → Проверить обновления.
# Followed by its shortcut, (Ctrl+S).
menu-settings-snapshot = Снимок
menu-settings-snapshot-ask-hover = Предлагать выбор при каждом нажатии
menu-settings-snapshot-screen-hover = Сохранять вид точно таким, каким он выглядит на экране (мгновенно, PNG)
menu-settings-snapshot-render-hover = Полный рендеринг с настройками экспорта, в фоне (на глубоких видах может занять много времени)
menu-settings-refcache = Кэш опорных орбит…
menu-settings-refcache-hover = Опорные орбиты глубокого приближения, хранящиеся на диске, чтобы место, к которому вы возвращаетесь, отрисовывалось за секунды. Где он находится, его размер, лимит и способ его очистить.

## Fractal menu

menu-fractal = Фрактал

## View menu

menu-view = Вид
menu-view-zoom-home = Отдалить до полного вида
menu-view-zoom-home-hover = Отдалить до полного исходного вида (с анимацией)
menu-view-reset = Сбросить к виду по умолчанию
menu-view-reset-hover = Сбросить к виду фрактала по умолчанию (мгновенно)
# Followed by its shortcuts.
menu-view-undo = Отменить изменение вида
menu-view-redo = Повторить изменение вида
menu-view-control-panel = Панель управления
menu-view-control-panel-hover = Показать/скрыть панель управления справа.
menu-view-minimap = Мини-карта
menu-view-minimap-hover = Показывать небольшой обзор исходного вида с меткой «вы здесь» и глубиной приближения. Щёлкните по нему, чтобы перейти к области.
menu-view-perf-panel = Панель производительности
menu-view-timestamp = Показывать отметку времени
menu-view-timestamp-hover = Рисовать поверх вида крупные часы прошедшего времени. Они показывают ту же отметку «+12.345s», которой журнал помечает каждую строку, поэтому запись экрана с проблемой, проявляющейся только в движении, можно покадрово сопоставить с журналом. По умолчанию выключено.
menu-view-zoom-target = Показывать цель автоприближения
menu-view-zoom-target-hover = Пока работает автоприближение, обводить область вида, к которой оно приближается, — часть, которая заполнит экран ещё через несколько увеличений, — линиями от её углов к углам экрана, а также кольцом на точке, к которой оно направляется. По умолчанию выключено.
menu-view-live-panel = Панель живого рендеринга
menu-view-live-panel-hover = Лицевая панель поверх вида, показывающая живой рендерер в работе: что отправил на выполнение каждый кадр, насколько старо изображение на экране, обход полос итераций, построители опорных орбит и машину под всем этим. Клавиша B переключает её размеры, а × в её углу закрывает её; перетащите её в любой угол; наведите курсор на прибор, чтобы узнать, что он означает и как выглядят норма и неполадка.
# The live render panel's sizes.
menu-view-live-off = Выкл.
menu-view-live-off-hover = Без панели.
menu-view-live-strip = Полоса
menu-view-live-strip-hover = Индикаторы-никси и блок ламп.
menu-view-live-panel-size = Панель
menu-view-live-panel-size-hover = Приборы: шкала обхода, возраст изображения, тайлы, проходы GPU, шкала глубины, лампы.
menu-view-live-full = Полная
menu-view-live-full-hover = Панель и машинное отделение: шаги, GPU, память, ядра, пиксели.
menu-view-live-opacity = Непрозрачность
menu-view-watermark = Показывать водяной знак «Fd»
menu-view-watermark-hover = Рисовать неброский фирменный знак «Fd» в правом нижнем углу живого вида и экспортированных изображений. По умолчанию включено.

## Color menu

menu-color = Цвет
menu-color-method = Метод
menu-color-palette = Палитра
menu-color-custom = Своя
menu-color-duotone = Двухтоновая
menu-color-binary = Двоичная (множество)
menu-color-binary-hover = Плоская двухцветная раскраска: внутри множества или вне его, без градиента.
menu-color-edit-gradient = Редактировать градиент…
menu-color-normalize = Нормализовать глубокие цвета
menu-color-normalize-hover = Перераспределять палитру по измеренному диапазону убегания вида, когда плотное поле иначе выглядело бы как рябь. Вид без алиасинга остаётся как есть — отметьте «Всегда подгонять диапазон», чтобы перераспределять в любом случае. Только для плавного метода.
menu-color-more = Цикл, сдвиг, анимация и эффекты: панель «Управление» ▸ Раскраска

## Tools menu

menu-tools = Инструменты
# Followed by its shortcut, (M).
menu-tools-find-minibrot = Найти миниброт и приблизиться к нему
menu-tools-find-minibrot-hover = Притянуть центр вида методом Ньютона к точному центру ближайшего миниброта, сообщить его период и приблизиться до собственного масштаба миниброта — часто на много порядков за один шаг. Вы уже глубже него? Тогда сдвинется только центр.
menu-tools-solver = Решатель Ньютона / Мисюревича…
menu-tools-solver-hover = Найти точный центр миниброта (период + размер) или ближайшую к виду точку Мисюревича — методом Ньютона с полной точностью, с отталкивающим мультипликатором λ. Открывает диалог местоположения, в котором находятся решатели.
# Followed by its shortcut, (A).
menu-tools-autopilot = Автоприближение (автопилот)
menu-tools-autopilot-stop = Остановить автопилот
menu-tools-autopilot-hover = Непрерывное глубокое приближение без участия рук: погружается к самой богатой деталями области, корректируя курс по ходу. Любое действие навигации останавливает его.
menu-tools-priority = Приоритет:
menu-tools-priority-speed = Скорость
menu-tools-priority-speed-hover = Приближаться с заданной скоростью; обновлять экран первым же кадром, в котором есть детали.
menu-tools-priority-quality = Качество
menu-tools-priority-quality-hover = Только полностью проработанные кадры; приближение их дожидается.
menu-tools-target = Цель:
menu-tools-target-detail = Детали
menu-tools-target-detail-hover = Самый богатый край в поле зрения, выбираемый заново по ходу погружения.
menu-tools-target-misiurewicz = Точка Мисюревича
menu-tools-target-misiurewicz-hover = Ближайшая точка Мисюревича, вычисленная один раз, с точным погружением в неё (только Mandelbrot).
menu-tools-orbits = Наложение орбиты
menu-tools-orbits-hover = Рисовать путь итераций точки под курсором. Параметры анимации: панель «Управление» ▸ Наложения.
menu-tools-orbits-normalize = Нормализовать (вписать в вид)
menu-tools-orbits-normalize-hover = Вписывать орбиту во весь вид, чтобы она оставалась в кадре при любом приближении (вместо отображения через область просмотра, где на глубине она улетает за пределы экрана).
menu-tools-play-tour = Воспроизвести тур…
menu-tools-play-tour-hover = Выбрать сценарий тура для воспроизведения. Кнопка ▶ на панели инструментов повторяет последний одним щелчком.
menu-tools-tour-from-view = Тур от текущего вида…
menu-tools-tour-from-view-hover = Создать тур, который приближает от полного вида до текущего, — добавьте подпись и задайте длительность погружения. Сохраняется как сценарий тура (.toml), который можно редактировать, воспроизводить или рендерить.
menu-tools-render-tour = Рендеринг тура…
menu-tools-render-tour-hover = Отрендерить загруженный тур в последовательность кадров PNG (и при желании в mp4) в отдельном процессе.
menu-tools-close-player = Закрыть проигрыватель тура
menu-tools-close-player-hover = Остановить тур, закрыть его проигрыватель и восстановить ваши собственные настройки итераций и раскраски
menu-tools-render-farm = Рендеринг на ферме…
menu-tools-render-farm-hover = Отрендерить загруженный тур на нескольких компьютерах: этот управляет заданием, а другие присоединяются через Файл ▸ Клиент рендеринга….
menu-tools-benchmark = Тест производительности…
menu-tools-benchmark-hover = Измерить скорость рендеринга — с текущими настройками или стандартизированным прогоном (фиксированные разрешение и настройки), сравнимым между компьютерами. «Длительный прогон» повторяет его, чтобы проверить стабильность и тепловой троттлинг.
menu-tools-last-benchmark = Показать последний тест

## Navigate menu

menu-navigate = Навигация
menu-navigate-goto = Перейти к местоположению…
menu-navigate-goto-hover = Ввести центр/масштаб, перейти к известной точке или найти методом Ньютона точку Мисюревича / миниброт рядом с видом.
menu-navigate-misiurewicz = Обозреватель точек Мисюревича…
menu-navigate-misiurewicz-hover = Посмотреть, как выглядят малые типы точек Мисюревича, — миниатюры с периодом приближения и закруткой каждой точки — и найти любую из них методом Ньютона на произвольной глубине. (Mandelbrot.)
menu-navigate-exact = Точные точки (выражения)
menu-navigate-exact-note = Корни почек главной кардиоиды, заданные точными выражениями (cos/sin от доли 2π). Координата хранится как выражение, поэтому при приближении вид остаётся точно на точке.
menu-navigate-random = Случайное местоположение
menu-navigate-random-hover = Перейти к случайной богатой деталями точке границы
menu-navigate-import-kfr = Импортировать .kfr…
menu-navigate-import-kfr-hover = Загрузить файл местоположения Kalles Fraktaler
menu-navigate-share = Поделиться местоположением…
menu-navigate-share-hover = Копировать / вставить / сохранить / загрузить самодостаточное местоположение (.fdn): фрактал, центр с полной точностью, масштаб, раскраска.

## Help menu

menu-help = Справка
# Followed by its shortcut, (F1).
menu-help-reference = Справка и справочник…
menu-help-welcome = Приветствие / быстрый старт…
menu-help-welcome-hover = Обзор при первом запуске: основные приёмы управления и несколько ориентиров.
menu-help-diagnostics = Диагностика…
menu-help-diagnostics-hover = Проверить, что Fractadyne правильно работает на вашем оборудовании, и приложить результат к сообщению о проблеме
menu-help-report = Сообщить о проблеме…
menu-help-report-hover = Создать issue на GitHub (или письмо) с приложенными отчётом о сбое, журналом и местоположением
menu-help-accelerated = Ускоренное глубокое приближение…
menu-help-accelerated-hover = Необязательная сборка, которая вычисляет опорные орбиты глубокого приближения в 2,5–6,4 раза быстрее. Изображения идентичны; ваши настройки переносятся.
menu-help-check-updates = Проверить обновления
# $track: "stable" or "beta", the update track in use (the variants hold the wording).
menu-help-check-updates-hover =
    { $track ->
        [beta] Проверить на GitHub, нет ли более новой бета-сборки (предварительные выпуски)
       *[other] Проверить на GitHub, нет ли более новой стабильной сборки
    }
# $version: the new version; $channel: "beta" for a pre-release, "stable" otherwise (the variants hold the wording).
menu-help-update-available =
    { $channel ->
        [beta] Доступна версия { $version } (бета) — скачать
       *[other] Доступна версия { $version } (стабильная) — скачать
    }
menu-help-up-to-date = Установлена последняя версия
menu-help-tagline = Нативный обозреватель фракталов
menu-help-license = Лицензия
# $year and $holder: the copyright year and holder, as the code gives them.
menu-help-copyright = © { $year } { $holder }. Двойное лицензирование: используйте на условиях любой из двух лицензий по своему выбору.
menu-help-source = Исходный код

## The toolbar (its buttons are icons; these are their tooltips)

menu-toolbar-julia = Жюлиа
menu-toolbar-julia-hover = Показать множество Жюлиа этой формулы на всё окно
menu-toolbar-dual-hover = Оба, связанные: множество слева, Жюлиа точки под курсором справа
menu-toolbar-open-hover = Открыть вид или местоположение (PNG / EXR, .fdn, .kfr)
menu-toolbar-gallery-hover = Галерея
menu-toolbar-export-hover = Экспортировать изображение…
menu-toolbar-share-hover = Поделиться местоположением — копировать, вставить, сохранить или загрузить самодостаточный .fdn (фрактал, центр с полной точностью, масштаб, раскраска)
menu-toolbar-zoom-in-hover = Приблизить
menu-toolbar-zoom-out-hover = Отдалить
# $factor: the click-zoom factor (4, 10…).
menu-toolbar-click-zoom-hover = Приближение щелчком ({ $factor }×): левый щелчок ныряет в точку, правый — возвращает назад (перетаскивание по-прежнему сдвигает вид). Коэффициент — в разделе Управление ▸ Навигация.
# Added to the click-to-zoom tooltip in the dual view.
menu-toolbar-click-zoom-dual = Пока это включено, c Жюлиа закрепляется щелчком с Ctrl.
menu-toolbar-autopilot-running-hover = Автоприближение работает — щёлкните, чтобы остановить
menu-toolbar-autopilot-hover = Автоприближение: погружение к деталям (A)
menu-toolbar-reset-hover = Сбросить вид (мгновенно)
menu-toolbar-home-hover = Отдалить до исходного вида (с анимацией)
menu-toolbar-bookmarks-hover = Закладки: сохранить этот вид или перейти к сохранённому
# $name: the palette's name.
menu-toolbar-next-palette-hover = Следующая палитра ({ $name })
# $file: the last tour's file name.
menu-toolbar-play-last-hover = Воспроизвести тур — { $file } (щелчок) · повторяет последний тур; другой можно выбрать через Инструменты ▸ Воспроизвести тур…
menu-toolbar-play-hover = Воспроизвести тур… (Инструменты ▸ Воспроизвести тур)
menu-toolbar-speed-hover = Скорость воспроизведения
menu-toolbar-fullscreen-hover = Полноэкранный режим

## Dialogs: words shared by many windows

# A button that closes a window without doing what it offers.
dlg-cancel = Отмена
# A button that dismisses a window (nothing in it is pending).
dlg-close = Закрыть
# A button that copies the window's text to the clipboard (after a copy icon).
dlg-copy = Копировать
# A button that jumps to a location.
dlg-go = Перейти
# A button that fills the fields from the view on screen.
dlg-use-current = Взять текущий
# A notice after a file could not be written. $error: the system's reason, as it gives it.
dlg-save-failed = Не удалось сохранить: { $error }
# A notice when no graphics device is available to render with.
dlg-gpu-not-available = GPU недоступен
# The name of the plain-text file type in a save dialog (the extension .txt is added by the code).
dlg-filter-text = Текст

## Asked after a session that did not shut down cleanly

dlg-crash-title = Fractadyne завершил работу некорректно
dlg-crash-intro = Предыдущий сеанс неожиданно завершился. Отчёт сохранён на этом компьютере — хотите отправить его?
dlg-crash-review = Ничего не отправляется, пока вы это не просмотрите: на следующем экране показано, что именно будет включено, а сведения о системе можно исключить одним флажком.
dlg-crash-report = Сообщить…
dlg-crash-report-hover = Открывает окно «Сообщить о проблеме» с приложенными отчётом о сбое и местоположением, на котором произошёл сбой.
dlg-crash-not-now = Не сейчас
dlg-crash-dont-ask = Больше не спрашивать после сбоя
dlg-crash-dont-ask-hover = Отчёты по-прежнему сохраняются локально, а Справка ▸ Сообщить о проблеме работает всегда.

## Asked after a session that ended with a second graphics device in use

dlg-second-device-title = Выключить второе графическое устройство?
# $device: the graphics device's name and details, as the system gives them.
dlg-second-device-intro = Предыдущий сеанс неожиданно завершился, когда использовалось второе графическое устройство ({ $device }). Возможно, причина в нём.
dlg-second-device-note = Когда оно выключено, всё рендерит видеокарта, которая рисует окно, как и раньше. Снова включить его можно в разделе Дополнительно ▸ Вторая видеокарта.
dlg-second-device-off = Выключить
dlg-second-device-keep = Оставить включённым

## Shown when the previous session's iteration setting was set aside

dlg-quarantine-title = Настройка итераций отложена
dlg-quarantine-change = Изменить её можно в любой момент в разделе Управление ▸ Качество.
dlg-quarantine-keep = Оставить значение по умолчанию
# A button. $setting: the iteration setting the previous session used (such as "20,000 iterations").
dlg-quarantine-restore = Восстановить: { $setting }

## Navigate ▸ Go to location

dlg-goto-title = Перейти к местоположению
# Before two choices: "x, y" (cartesian) and the polar form below.
dlg-goto-enter-as = Ввод в виде:
# The polar way of entering a point: an offset plus a radius r at an angle θ. Keep the symbols.
dlg-goto-polar = смещение + r ∠ θ
# The tooltip of the "?" button beside the entry choice.
dlg-goto-help-hover = Что принимают поля: дроби, степени, sqrt/sin/cos…, pi и другие константы — открывает Справка → Выражения координат
# Labels over the entry fields. x0, y0, r, θ, Re and Im are the fields' symbols: keep them.
dlg-goto-polar-offset = Центр смещения  (x0, y0 — оставьте 0 для чисто полярной точки)
dlg-goto-polar-radius = Радиус r, угол θ
dlg-goto-re = Центр — действительная часть (Re)
dlg-goto-im = Центр — мнимая часть (Im)
dlg-goto-zoom = Масштаб (увеличение)
# A warning under the zoom field, after a ⚠ sign. $have: the iteration count set; $typical: the count this depth typically needs.
dlg-goto-iter-warning = «Итерации» зафиксированы на { $have }; этой глубине обычно нужно ~{ $typical }. Вид будет отрисовываться плоским, пока их не увеличат, — или включите автоитерации (Управление ▸ Качество), которые подстраиваются сами.
dlg-goto-copy-hover = Скопировать это местоположение в буфер обмена
dlg-goto-cancel-hover = Закрыть, никуда не переходя
# The expressions in brackets are examples of what can be typed: keep them as they are.
dlg-goto-paste-note = Вставьте центр/масштаб, полученные от кого-то, или нажмите «Копировать», чтобы поделиться. Координаты принимают выражения: точные дроби (-3/4), функции и константы (-0.5 + 0.25*cos(pi/4), sqrt, root(x,n), ^; углы в радианах), а поле действительной части принимает и целое комплексное значение ((37+16i)/100), которое заполняет оба поля.
# A section that opens and closes.
dlg-goto-feature = Перейти к объекту (Mandelbrot)
dlg-goto-feature-poi = Известные точки:
# The dropdown's text before a point is picked.
dlg-goto-feature-poi-pick = Перейти к интересной точке…
dlg-goto-feature-solve = Или найти объект рядом с текущим видом:
# Two choices: what to solve for. Misiurewicz is a mathematician's name (a Misiurewicz point).
dlg-goto-feature-misiurewicz = Точка Мисюревича
dlg-goto-feature-minibrot = Ближайший миниброт
# Labels of two small number fields; k and p are their symbols.
dlg-goto-feature-preperiod = Предпериод k
dlg-goto-feature-period = Период p
# The faint text in an empty field: left empty, the value is found automatically.
dlg-goto-feature-auto-hint = авто
# A small button that empties both fields, so they are found automatically again.
dlg-goto-feature-auto = Авто
dlg-goto-feature-auto-hover = Очистить оба поля, чтобы следующий поиск определил предпериод и период по орбите в центре вида.
dlg-goto-feature-kp-hover = Оставьте оба поля пустыми, чтобы определить их по орбите в центре вида; найденные значения появятся здесь. Никто, глядя на спираль, не знает её предпериода, и именно необходимость его указывать делала этот решатель непригодным на практике.
# While a solve runs, followed by its elapsed time and the note below.
dlg-goto-feature-solving = Вычисление…
dlg-goto-feature-solving-note = (произвольная точность; глубокие виды занимают больше времени)
dlg-goto-feature-cancel-hover = Перестать ждать. Само вычисление доводится до конца в фоне, а его ответ отбрасывается — ничего не остаётся в незавершённом состоянии.
dlg-goto-feature-find = Найти рядом с видом
dlg-goto-feature-find-hover =
    Найти методом Ньютона, начиная с текущего центра, точный объект (произвольная точность). Сначала переместитесь поближе к объекту.

    Перед поиском задайте в поле «Масштаб» выше глубину, которая вам НУЖНА: точка вычисляется с точностью, которой требует эта глубина, и вид сразу переходит туда. Это одно вычисление вместо долгой череды ручных шагов приближения.
dlg-goto-solve-cancelled = Вычисление отменено.

## Snapshot (the camera button): its tooltip, and the choice on the first press

# The camera button's tooltip, one per Snapshot setting. Followed by its shortcut, (Ctrl+S).
dlg-snapshot-hover-ask = Снимок — сохранить вид как на экране или полный рендеринг; при первом нажатии программа спросит
dlg-snapshot-hover-screen = Снимок — сохранить вид точно как на экране в последнюю папку экспорта
dlg-snapshot-hover-render = Снимок — полный рендеринг с настройками экспорта в последнюю папку экспорта
# The window's title.
dlg-snapshot-title = Снимок
dlg-snapshot-question = Что должен сохранять «Снимок» (кнопка с камерой, Ctrl+S)?
# The first choice, and a short tag beside it.
dlg-snapshot-screen = Захват экрана
dlg-snapshot-screen-tag = мгновенно
dlg-snapshot-screen-note = Сохраняет вид точно таким, каким он выглядит на экране, в разрешении экрана, в формате PNG в последнюю папку экспорта. Что видите, то и получаете.
# The second choice, and a short tag beside it.
dlg-snapshot-render = Полный рендеринг
dlg-snapshot-render-tag = может занять много времени
# $size: the image size, as 1920×1080; $ss: the supersampling factor (1–4); $format: PNG or EXR.
dlg-snapshot-render-note = Рендеринг с настройками экспорта — { $size }, суперсэмплинг { $ss }×, { $format }. Наилучшее качество, но глубокий вид может рендериться от нескольких минут до нескольких часов. Выполняется в фоне; Файл → Экспортировать изображение… показывает ход работы и позволяет её отменить.
dlg-snapshot-remember = Запомнить мой выбор и больше не спрашивать
dlg-snapshot-change = Изменить это можно в любой момент: Файл → Настройки → Снимок.

## Share location (.fdn)

dlg-share-title = Поделиться местоположением
dlg-share-intro = Самодостаточное местоположение (фрактал, центр с полной точностью, масштаб, раскраска). Скопируйте его, чтобы поделиться, или вставьте/загрузите и нажмите «Применить».
dlg-share-thumb = Включить миниатюру в сохраняемый файл
dlg-share-thumb-hover = «Сохранить .fdn» встраивает небольшое изображение этого вида, чтобы файл можно было узнать в галерее. Добавляет примерно 55 KB и не входит в текст выше.
dlg-share-copy-hover = Скопировать текст в буфер обмена
# Buttons (the first after a save icon).
dlg-share-save = Сохранить .fdn…
dlg-share-load = Загрузить .fdn…
dlg-share-apply = Применить
dlg-share-apply-hover = Перейти к местоположению из поля
dlg-share-copied = Скопировано в буфер обмена.

## Help ▸ Report an issue (the report itself stays in English: it is read by the maintainer)

dlg-report-title = Сообщить о проблеме
dlg-report-intro = Лучше всего сообщать через issue на GitHub — это публично, там работает поиск, и вы увидите, когда проблему исправят. Электронная почта тоже подойдёт. Сначала просмотрите всё, что ниже: ничего не покидает ваш компьютер, пока вы сами этого не сделаете.
# Labels before three dropdowns.
dlg-report-type = Тип:
dlg-report-severity = Серьёзность:
dlg-report-repro = Воспроизводимость:
dlg-report-describe = Опишите, что произошло:
# The faint text in the empty description box.
dlg-report-describe-hint = Что произошло, шаги для воспроизведения и что вы ожидали вместо этого.
# Before the checkboxes of what the report carries.
dlg-report-include = Включить:
dlg-report-sysinfo = Сведения о системе (версия, ОС, CPU, GPU, VRAM)
dlg-report-location = Текущее местоположение (.fdn)
dlg-report-log = Недавний журнал
dlg-report-frames = Запись кадров (последняя минута и последние 40 кадров)
dlg-report-crash = Последний отчёт о сбое
dlg-report-crash-none = Последний отчёт о сбое (не найден)
dlg-report-crash-view = Местоположение сбоя (.fdn)
dlg-report-crash-view-none = Местоположение сбоя (.fdn) (не найдено)
dlg-report-crash-view-hover = Точный вид, на котором произошёл сбой или потеря графического устройства, — единственное, что позволяет разработчику воспроизвести проблему. Записывается автоматически.
dlg-report-test = Результат диагностического теста
dlg-report-test-none = Результат диагностического теста (запустите его через Справка → Диагностика…)
dlg-report-test-hover = Проверенный машиной результат «пройдено/не пройдено» на вашем собственном оборудовании — для разработчика он гораздо полезнее, чем одно лишь описание
# A section that opens and closes. $size: the report's size, as "12 KB".
dlg-report-preview = Предпросмотр того, что будет отправлено  ({ $size })
dlg-report-github = Открыть issue на GitHub
dlg-report-github-hover = Открыть страницу новой issue с заполненным заголовком; отчёт копируется в буфер обмена — вставьте его в текст issue (Ctrl+V).
# Buttons (after a copy and a save icon).
dlg-report-copy = Копировать отчёт
dlg-report-copy-hover = Скопировать всё в буфер обмена
dlg-report-save = Сохранить отчёт…
dlg-report-save-hover = Сохранить как .txt, чтобы приложить
dlg-report-by-email = Или по электронной почте:
dlg-report-gmail = Написать в Gmail
# $address: the email address reports go to.
dlg-report-gmail-hover = Открыть окно нового письма Gmail на адрес { $address } с заполненной темой; вставьте скопированный отчёт
dlg-report-email = Почтовая программа…
dlg-report-email-hover = Открыть почтовую программу по умолчанию (mailto) — нужен настроенный обработчик почты
dlg-report-tip = Совет: при нажатии любой из этих кнопок отчёт копируется в буфер обмена — вставьте его в текст issue или письма (Ctrl+V). Если можете, приложите также снимок экрана или пример изображения (перетащите его сюда); это очень помогает с проблемами рендеринга и интерфейса.
# Notices in the window after a button is pressed.
dlg-report-opened-github = GitHub открыт — отчёт в буфере обмена; вставьте его в issue.
dlg-report-copied = Отчёт скопирован в буфер обмена.
# $path: where the report was saved.
dlg-report-saved = Сохранено в { $path }
dlg-report-opened-email = Почтовая программа открыта — отчёт в буфере обмена; вставьте его.
dlg-report-opened-gmail = Окно нового письма Gmail открыто — вставьте скопированный отчёт в текст письма (Ctrl+V).

## A location whose checksum does not match

dlg-checksum-title = Это местоположение может быть повреждено
# After a ⚠ sign.
dlg-checksum-mismatch = Его контрольная сумма не совпадает с данными рядом с ней.
dlg-checksum-why = Обычно это значит, что текст по дороге сюда обрезали или изменили, — частая причина тому неполная копия из окна чата или сообщения на форуме.
# A label over where the location came from (a file path, or the clipboard).
dlg-checksum-source = Источник
# The two checksums, side by side: the one written in the file, and the one computed from its data.
dlg-checksum-in-file = в файле
dlg-checksum-from-data = по данным
dlg-checksum-load-note = Его всё равно можно загрузить — координаты могут быть в порядке, и ничего не перезаписывается. Обо всём, что не удастся прочитать, будет сообщено после загрузки.
dlg-checksum-load = Всё равно загрузить
dlg-checksum-load-hover = Перейти к нему, несмотря на несовпадение
# Notices after loading it. $zoom: the magnification, as 1.2e148.
dlg-checksum-loaded = Загружено @ { $zoom }×, несмотря на несовпадение контрольной суммы
# $zoom: the magnification, as 1.2e148; $note: what the loader found wrong with the location.
dlg-checksum-loaded-note = Загружено @ { $zoom }× — { $note }
dlg-checksum-not-loaded = Не загружено.

## File ▸ Reset application state

dlg-reset-title = Сброс состояния программы
# After a ⚠ sign, followed by a list.
dlg-reset-warning = Это безвозвратно удалит все сохранённые данные Fractadyne:
# The list's items, each after a bullet.
dlg-reset-session = сохранённый сеанс (текущий вид, раскраска, предпочтения)
dlg-reset-bookmarks = все закладки и их миниатюры
dlg-reset-refcache = кэш опорных орбит (восстанавливается, когда вы снова посещаете места)
# A label over the folder that is deleted.
dlg-reset-location = Расположение
dlg-reset-note = Это нельзя отменить. Текущий сеанс не будет сохранён при выходе, поэтому при следующем запуске загрузятся значения по умолчанию.
dlg-reset-confirm = Сбросить всё
dlg-reset-done = Состояние программы сброшено — при следующем запуске загрузятся значения по умолчанию.
# $error: the system's reason, as it gives it.
dlg-reset-failed = Не удалось выполнить сброс: { $error }

## Tools ▸ Tour from current view

dlg-tour-title = Тур от текущего вида
dlg-tour-intro = Создаёт тур, который приближает от полного вида до того места, где вы сейчас.
# A label before the fractal and magnification the tour ends at.
dlg-tour-destination = Цель
dlg-tour-notation = Надпись (необязательная подпись, показываемая во время погружения)
# The faint example text in the empty caption box.
dlg-tour-notation-hint = напр. Погружение к трёхлучевой точке Мисюревича…
dlg-tour-duration = Длительность приближения
# $rate: how fast the dive zooms, in powers of ten a second (as 2.5).
dlg-tour-rate = ≈ { $rate } декад/с
# A small button that puts the duration back to its default.
dlg-tour-reset = Сбросить
dlg-tour-reset-hover = Значение по умолчанию в зависимости от глубины
dlg-tour-duration-note = Значение по умолчанию растёт с глубиной, чтобы за погружением можно было уследить. При более быстрых погружениях на экстремальной глубине могут мелькать одноцветные кадры.
# A button, after a save icon.
dlg-tour-save = Сохранить тур…
# The file type's name in the save dialog.
dlg-tour-filter = Сценарий Fractadyne (TOML)
# A notice. $file: the saved file's name. --render-tour is a command-line option: keep it.
dlg-tour-saved = Тур сохранён → { $file }. Воспроизведите его через Инструменты → Воспроизвести тур или отрендерите с --render-tour.

## Navigate ▸ Bookmarks ▸ Manage…

dlg-bookmarks-title = Закладки
# The faint text in the empty name box.
dlg-bookmarks-name-hint = имя (необязательно)
# A button, after a bookmark icon.
dlg-bookmarks-add = Добавить текущий вид
dlg-bookmarks-empty = Закладок пока нет. Добавьте закладку на текущий вид.
# The tooltip of a bookmark's delete (bin) button.
dlg-bookmarks-delete-hover = Удалить

## Tools ▸ Benchmark…

dlg-bench-title = Тест производительности
dlg-bench-mode = Режим
dlg-bench-current = Текущие настройки
dlg-bench-current-hover = Воспроизвести фиксированный тур глубокого приближения в живом виде с вашими текущими разрешением и настройками.
dlg-bench-standard = Стандартизированный
dlg-bench-standard-hover = Зафиксировать разрешение и все настройки рендеринга, чтобы результат можно было сравнивать между компьютерами.
dlg-bench-resolution = Разрешение
dlg-bench-depth = Глубина
# Standard and Ultra are the two depths in the dropdown this describes.
dlg-bench-depth-hover = Оба погружения заходят на глубокий путь floatexp/BLA; «Сверхглубокое» (1e48×) проводит там гораздо больше времени, чем «Стандартное» (1e32×), и соответственно дольше выполняется.
dlg-bench-burnin = Длительный прогон (повтор)
dlg-bench-burnin-hover = Запускать тест многократно, чтобы выявить проблемы со стабильностью и тепловой троттлинг.
# The label of a slider: how many times the benchmark runs.
dlg-bench-passes = прогоны
# $size: the image size, as 1920×1080; $ss: the supersampling factor; $depth: the dive's final magnification, as 1e32.
dlg-bench-std-summary = Рендеринг вне экрана в { $size }, { $ss }× SS, Mandelbrot/плавная раскраска, погружение из 60 кадров до { $depth }×.
dlg-bench-run = Запустить

## The standardized benchmark's progress window

dlg-bench-progress-title = Выполняется тест…
# $resolution: the benchmark's resolution, as the dropdown names it.
dlg-bench-progress-mode = Стандартизированный · { $resolution }
# $pass: the pass running now; $total: how many passes there are.
dlg-bench-progress-pass = Длительный прогон: проход { $pass }/{ $total }
dlg-bench-progress-dive = Рендеринг фиксированного погружения…
# On the progress bar. $done: frames rendered; $total: frames in the dive.
dlg-bench-progress-frame = кадр { $done }/{ $total }
# $fps: frames per second, as 41.3.
dlg-bench-progress-last = последний проход: { $fps } fps

## The benchmark's results

dlg-bench-results-title = Результаты теста
# Buttons, each after an icon.
dlg-bench-results-run-again = Запустить снова…
dlg-bench-results-save = Сохранить…
dlg-bench-results-none = Тест ещё не запускался.
dlg-bench-results-saved = Результаты теста сохранены.

## The accelerated build's notice when its arithmetic libraries are missing

dlg-backend-title = Ускоренная сборка — используется встроенная арифметика
dlg-backend-dont-show = Больше не показывать

## File ▸ Gallery…

dlg-gallery-title = Галерея
# A button that picks the folder to show.
dlg-gallery-folder = Папка…
# A button, after a refresh icon.
dlg-gallery-refresh = Обновить
dlg-gallery-empty = В этой папке нет изображений Fractadyne.
# An image's title when it does not name its fractal.
dlg-gallery-untitled = Изображение Fractadyne
# $zoom: the image's magnification, as 1.2e148; $saved: when it was saved, as the file gives it.
dlg-gallery-zoom = масштаб { $zoom }   { $saved }
# The notes saved with the image, in quotation marks. Use your language's quotation marks.
dlg-gallery-notes = «{ $notes }»
dlg-gallery-open = Открыть этот вид
dlg-gallery-loaded = Вид загружен из галереи.

## File ▸ Export image…

dlg-export-title = Экспорт изображения
# A dropdown of image sizes.
dlg-export-size = Размер
# The size dropdown's text for a size set by hand. $size: the size, as 1920×1080.
dlg-export-size-custom-current = Свой — { $size }
# The size dropdown's first choice.
dlg-export-size-custom = Свой…
dlg-export-size-custom-hover = Задать ширину самостоятельно; высота следует из пропорций ниже.
dlg-export-width = Ширина (px)
# After the width field. $height: the height in pixels, which follows from the aspect.
dlg-export-height-from-aspect = × { $height } (по пропорциям)
dlg-export-ss = Суперсэмплинг
dlg-export-aspect = Пропорции
# The aspect choice that keeps the window's own shape.
dlg-export-aspect-window = Как у окна
# Before the choices PNG and OpenEXR.
dlg-export-format = Формат:
dlg-export-hud = HUD местоположения
dlg-export-hud-hover = Впечатать панель с уровнем приближения и координатами в левый верхний угол экспортируемого изображения (масштабируется вместе с результатом; показывает масштаб и центр вида карты/Mandelbrot).
dlg-export-glitch = Исправление глитчей
dlg-export-glitch-hover = Исправление глитчей метода возмущений с помощью нескольких опорных точек. Автоматически пропускается при очень глубоком (floatexp) экспорте одиночного вида, чтобы построение опорной орбиты и рендеринг шли в отдельном потоке, а программа оставалась отзывчивой. Никогда не используется для видов Mandelbrot и Multibrot (вне режима Жюлиа): при проверке по точному вычислению оно там ничего не исправило, а время экспорта увеличивало до 5 раз.
dlg-export-deep-note = Глубокий экспорт: опорная орбита строится в отдельном потоке (интерфейс остаётся отзывчивым); на этой глубине исправление глитчей пропускается.
# A dropdown, shown in the dual (map + Julia) view, and its three choices.
dlg-export-dual = Двойная компоновка
dlg-export-dual-side = Рядом
dlg-export-dual-separate = Отдельные файлы
dlg-export-dual-map = Только карта
dlg-export-notes = Заметки:
# The faint text in the empty notes box.
dlg-export-notes-hint = сохраняются с изображением (≤120 символов)
# $width and $height: the image size in pixels; $left: how many characters the notes can still take.
dlg-export-output = Результат: { $width } × { $height } px   (осталось символов: { $left })
# When the export's shape differs from the window's. $percent: how much more, as 25.
dlg-export-extra-width = Показывает на { $percent }% больше по ширине, чем окно (ничего не обрезается)
# The same, when the export is taller than the window. $percent: how much more, as 25.
dlg-export-extra-height = Показывает на { $percent }% больше по высоте, чем окно (ничего не обрезается)
dlg-export-tiles-note = Рендерится тайлами (без ограничения размера) в фоновом потоке. Файл содержит вид и заметки, поэтому его можно снова открыть через меню Файл ▸ Открыть вид или местоположение…
# A label before the button that picks the folder exports go to.
dlg-export-folder = Папка:
dlg-export-choose = Выбрать…
dlg-export-choose-hover = Выбрать папку назначения для экспорта
dlg-export-open-after = Открыть по завершении
dlg-export-open-after-hover = Передать готовое изображение системному просмотрщику сразу после записи.
# $count: the graphics cards in this machine (2 or more).
dlg-export-all-gpus = Использовать все видеокарты ({ $count })
dlg-export-all-gpus-hover = Разделить изображение по тайлам между всеми видеокартами этого компьютера, у каждой — своя копия глубокой опорной орбиты. Одинаковые карты дают в точности то же изображение, что и одна карта; разные модели рисуют несколько пикселей чуть по-разному, тайл за тайлом (измерено: шов не виден). Если карта откажет, её тайлы перейдут к остальным. Изображения, которые нельзя так разделить (некоторые методы раскраски и формулы), рендерит видеокарта этого окна.
# The export's progress.
dlg-export-preparing = Подготовка (построение опорной орбиты)…
dlg-export-saving = Сохранение…
dlg-export-rendering = Рендеринг…
# $time: the time since the export started, as 8.3s or 1m 04.0s.
dlg-export-elapsed = Прошло: { $time }
dlg-export-cancel-hover = Закрыть без экспорта
# A button, after a save icon.
dlg-export-save-as = Сохранить как…
dlg-export-save-as-hover = Выбрать имя и расположение файла
dlg-export-export = Экспортировать
dlg-export-export-hover = Отрендерить и сохранить в папку выше (имя задаётся автоматически)

## Notices worded at startup (main.rs)

# A notice shown once after starting, when the saved session comes from a newer version of the app. $saved and $handled: version numbers of the session file's format (1, 2…).
app-state-newer = Этот сеанс сохранён более новой версией Fractadyne (состояние v{ $saved }; эта сборка поддерживает v{ $handled }). Некоторые настройки могут не примениться, а при сохранении сеанс будет перезаписан в формате этой сборки.
# The title of the notice that says the accelerated build's arithmetic libraries are found again (its text comes from elsewhere).
app-notice-mpfr-restored-title = Ускоренная арифметика восстановлена
# After Fractadyne restarted itself because the graphics driver reset the GPU.
app-toast-gpu-recovered = Работа восстановлена после сброса графического устройства — открыт исходный вид, так как предыдущее местоположение было слишком тяжёлым для рендеринга.

## Short notices (toasts)

# When the autopilot reached a view that could not be rendered and put the last good view back.
app-toast-autopilot-view-restored = Автопилот остановлен: вид стал непригодным и был возвращён (записано в журнал)
# $error: the system's reason, as it gives it (often in English).
app-toast-gradients-save-failed = Не удалось сохранить градиенты: { $error }
# $error: the reason, as the program gives it (often in English).
app-toast-gradients-serialize-failed = Не удалось сериализовать градиенты: { $error }
# $name: the name the user gave the gradient.
app-toast-gradient-saved = Градиент «{ $name }» сохранён.
# $error: the reason, as the program gives it (often in English).
app-toast-bookmarks-serialize-failed = Не удалось сериализовать закладки: { $error }
# $error: the system's reason, as it gives it (often in English).
app-toast-bookmarks-save-failed = Не удалось сохранить закладки: { $error }
# Jumping to a named point given by exact expressions. $name: the point's name (in English).
app-toast-coordinates-failed = { $name }: не удалось вычислить координаты
# $zoom: the magnification, already formatted (1,234.5 or 1.2e148).
app-toast-random-location = Случайное местоположение · { $zoom }×
# Importing a Kalles Fraktaler .kfr location. $error: the reason (in English).
app-toast-import-failed = Ошибка импорта: { $error }
# $zoom: the magnification, already formatted.
app-toast-location-loaded = Местоположение загружено @ { $zoom }×

## Go to location: why a jump could not be made (one line per field that failed)

# "Re" is the dialog's real-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-re = Re: { $error }
# "Im" is the dialog's imaginary-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-im = Im: { $error }
# The imaginary field was given a complex value. $error: the evaluator's reason (in English).
app-goto-error-im-real = Im: { $error } — введите мнимую часть как действительное число, без i
# "Zoom" is the dialog's magnification field.
app-goto-error-zoom-empty = Масштаб: введите увеличение, например 1e50 или 250000
# $zoom: what was typed, in quotes.
app-goto-error-zoom = Масштаб: не удаётся прочитать { $zoom } — введите положительное увеличение, например 1e50 или 250000
app-goto-error-invalid = Неверный ввод — проверьте координаты и масштаб.

## Finding a minibrot or Misiurewicz point (the M key and the Go to dialog)

app-feature-needs-holomorphic = Поиску минибротов нужно голоморфное семейство (Mandelbrot / Multibrot).
app-feature-not-in-julia = Поиск минибротов ведётся в множестве Мандельброта — выключите режим Жюлиа, чтобы им воспользоваться.
app-feature-busy = Объект уже ищется — только по одному за раз.
app-feature-searching = Поиск ближайшего миниброта…
app-feature-mandelbrot-only = Поиск объектов работает только для Mandelbrot.
app-feature-no-pair = В пределах бюджета поиска точка Мисюревича не найдена — возможно, она глубже, чем поиск достаёт с этого вида, или центр находится не рядом со спиралью или точкой ветвления.
# The name of what was found, used as $feature in the messages below. $period: the minibrot's period (a whole number).
app-feature-minibrot = миниброта периода { $period }
# $preperiod and $period: the point's two whole numbers (k, p). $octaves: how often the view repeats when zooming in (2 decimals); $twist: the turn per repeat in degrees.
app-feature-misiurewicz-lambda = точки Мисюревича ({ $preperiod },{ $period }) — повторяется каждые { $octaves } октавы, закрутка { $twist }°
# $preperiod and $period: the point's two whole numbers (k, p).
app-feature-misiurewicz = точки Мисюревича ({ $preperiod },{ $period })
# Added after the messages below when the fixed iteration count is too low for the new depth. $have: the iterations set now; $typical: what the depth typically needs.
app-feature-iter-note = ⚠ «Итерации» зафиксированы на { $have }; этой глубине обычно нужно ~{ $typical } — увеличьте их (или включите автоитерации), если вид отрисовывается плоским.
# $feature: what was found (see above). $zoom and $asked: magnifications, already formatted; $octaves: the solver's depth limit.
app-feature-zoomed-capped =
    { $octaves ->
        [one] Выполнено приближение до { $feature } — { $zoom }×. Вычисление ограничено { $octaves } октавой, поэтому оно остановилось здесь, а не на { $asked }×.
        [few] Выполнено приближение до { $feature } — { $zoom }×. Вычисление ограничено { $octaves } октавами, поэтому оно остановилось здесь, а не на { $asked }×.
        [many] Выполнено приближение до { $feature } — { $zoom }×. Вычисление ограничено { $octaves } октавами, поэтому оно остановилось здесь, а не на { $asked }×.
       *[other] Выполнено приближение до { $feature } — { $zoom }×. Вычисление ограничено { $octaves } октавы, поэтому оно остановилось здесь, а не на { $asked }×.
    }
# $feature: what was found. $zoom: the magnification, already formatted. $seconds: how long the search took (1 decimal).
app-feature-zoomed = Выполнено приближение до { $feature } — { $zoom }× ({ $seconds }s)
# $feature: what was found. $seconds: how long the search took (1 decimal).
app-feature-snapped = Притянуто к центру { $feature } ({ $seconds }s)
app-feature-no-minibrot = Рядом с видом не найден центр миниброта — приблизьтесь к нему.
# $feature: the point asked for, e.g. Misiurewicz (3,1). $widths: how many view-widths away, already formatted. $octaves: how far to zoom out.
app-feature-just-outside =
    { $octaves ->
        [one] Положение { $feature } найдено сразу за пределами этого вида (расстояние в ширинах вида: { $widths }). Отдалитесь примерно на { $octaves } октаву, чтобы увидеть точку на экране, или задайте больший предпериод, чтобы найти объект этого масштаба.
        [few] Положение { $feature } найдено сразу за пределами этого вида (расстояние в ширинах вида: { $widths }). Отдалитесь примерно на { $octaves } октавы, чтобы увидеть точку на экране, или задайте больший предпериод, чтобы найти объект этого масштаба.
        [many] Положение { $feature } найдено сразу за пределами этого вида (расстояние в ширинах вида: { $widths }). Отдалитесь примерно на { $octaves } октав, чтобы увидеть точку на экране, или задайте больший предпериод, чтобы найти объект этого масштаба.
       *[other] Положение { $feature } найдено сразу за пределами этого вида (расстояние в ширинах вида: { $widths }). Отдалитесь примерно на { $octaves } октавы, чтобы увидеть точку на экране, или задайте больший предпериод, чтобы найти объект этого масштаба.
    }
# $feature: the point asked for. $widths: how many view-widths away, already formatted (may be 2^187). $octaves: how far to zoom out.
app-feature-far =
    { $octaves ->
        [one] Положение { $feature } найдено, но расстояние до неё в ширинах вида — { $widths }: этот объект гораздо КРУПНЕЕ этого вида. ОТДАЛИТЕСЬ примерно на { $octaves } октаву, чтобы до него добраться, или попробуйте больший предпериод, чтобы найти объект этого масштаба.
        [few] Положение { $feature } найдено, но расстояние до неё в ширинах вида — { $widths }: этот объект гораздо КРУПНЕЕ этого вида. ОТДАЛИТЕСЬ примерно на { $octaves } октавы, чтобы до него добраться, или попробуйте больший предпериод, чтобы найти объект этого масштаба.
        [many] Положение { $feature } найдено, но расстояние до неё в ширинах вида — { $widths }: этот объект гораздо КРУПНЕЕ этого вида. ОТДАЛИТЕСЬ примерно на { $octaves } октав, чтобы до него добраться, или попробуйте больший предпериод, чтобы найти объект этого масштаба.
       *[other] Положение { $feature } найдено, но расстояние до неё в ширинах вида — { $widths }: этот объект гораздо КРУПНЕЕ этого вида. ОТДАЛИТЕСЬ примерно на { $octaves } октавы, чтобы до него добраться, или попробуйте больший предпериод, чтобы найти объект этого масштаба.
    }
# $feature: the point asked for. $residual: a small number in scientific notation.
app-feature-not-preperiodic = Орбита здесь не подходит для { $feature } (невязка { $residual }) — очистите оба поля, чтобы определить пару по виду.
# $feature: the point asked for. "k/p" are the dialog's two fields.
app-feature-not-converged = Рядом с видом вычисление { $feature } не сошлось — подойдите ближе или попробуйте другие k/p.
app-feature-bad-request = Предпериод и период должны быть положительными.

## Share location dialog and location files

app-location-empty = Нечего загружать (или текст слишком большой).
app-location-not-fractadyne = Это не местоположение Fractadyne.
# $zoom: the magnification, already formatted. $note: what in the text was ignored or repaired (in English).
app-share-loaded-note = Загружено @ { $zoom }× — { $note }.
# $error: the system's reason (often in English).
app-share-save-failed = Не удалось сохранить: { $error }
app-share-saved = Сохранено.
app-share-saved-rendering-thumb = Сохранено. Рендеринг миниатюры…
app-share-saved-no-thumb = Сохранено — без миниатюры (для этого вида её не удалось отрендерить).
app-share-saved-with-thumb = Сохранено, с миниатюрой.
# $error: the system's reason (often in English).
app-share-saved-thumb-failed = Сохранено, но миниатюру добавить не удалось: { $error }
app-share-loaded-into-box = Загружено в поле — проверьте и нажмите «Применить».
# Reading a file failed (a location or a palette). $error: the system's reason (often in English).
app-read-failed = Ошибка чтения: { $error }
app-share-too-large = Файл слишком большой (это не местоположение .fdn?).

## File-dialog filter names (the extensions are added by the code)

app-filter-kfr = Местоположение Kalles Fraktaler
app-filter-fdn = Местоположение Fractadyne
app-filter-map = Палитра Fractint / KF
app-filter-ugr = Градиент Ultra Fractal
app-filter-ase = Adobe Swatch Exchange
app-filter-ggr = Градиент GIMP
app-filter-all = Все файлы

## Importing and saving palette files (shown under the gradient editor)

# The reason a file was refused before reading it.
app-gradient-map-too-large = слишком велик для палитры .map
app-gradient-ugr-too-large = слишком велик для файла палитр .ugr
app-gradient-ase-too-large = слишком велик для списка образцов .ase
app-gradient-ggr-too-large = слишком велик для градиента .ggr
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported-vga = Импортировано записей из { $file }: { $count } — это 6-битная таблица VGA (белый — 252, а не 255); она сохранена как есть, чтобы совпадать с выводом самого Fractint.
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported = Импортировано записей из { $file }: { $count }, в виде резких полос.
# $error: the reader's reason (in English).
app-gradient-map-invalid = Это не палитра .map: { $error }
# $name: the gradient's name in the file; $file: the file's name.
app-gradient-ugr-imported = Импортирован «{ $name }» из { $file }.
# $file: the file's name; $count: how many gradients it holds.
app-gradient-ugr-pick = Градиентов в { $file }: { $count } — выберите один ниже.
# $error: the reader's reason (in English).
app-gradient-ugr-invalid = Это не файл палитр .ugr: { $error }
# $used: the swatches kept; $total: the swatches in the file.
app-gradient-ase-imported-some = Импортировано образцов: { $used } из { $total }, с равными промежутками.
# $count: the swatches in the file.
app-gradient-ase-imported = Импортировано образцов: { $count }, с равными промежутками и плавными переходами.
# $error: the reader's reason (in English).
app-gradient-ase-invalid = Это не список образцов .ase: { $error }
app-gradient-nothing-to-save = Пока нет своего градиента для сохранения.
# $file: the saved file's name.
app-gradient-ggr-saved = Сохранено «{ $file }» — средние точки, кривые перехода и цветовые пространства сохранены полностью.
# $file: the saved file's name; $count: the segments whose curve could not be kept.
app-gradient-ggr-saved-lossy = Сохранено «{ $file }», но сегменты с кривой Безье (их { $count }) формат .ggr выразить не может. Они записаны как прямые переходы; копия в вашей библиотеке сохраняет кривые.
# $path: the file; $error: the system's reason (often in English).
app-gradient-write-failed = Не удалось записать { $path }: { $error }
# $name: the gradient's name; $count: its segments.
app-gradient-ggr-imported = Импортирован «{ $name }» — сегментов: { $count }, с их средними точками, кривыми перехода и цветовыми пространствами.
# $error: the reader's reason (in English).
app-gradient-ggr-invalid = Это не градиент .ggr: { $error }

## The eyedropper's banner across the top of the screen while picking a colour

# $hex: the colour under the pointer, as #rrggbb.
app-eyedropper-banner-color = Выбор цвета  ·  { $hex }  ·  щёлкните, чтобы взять его, Esc — отмена
app-eyedropper-banner = Выбор цвета  ·  щёлкните, чтобы взять его, Esc — отмена

## The gradient editor window

app-gradient-title = Редактор градиента
# Before the two view buttons, Bar and Ring.
app-gradient-view = Вид
app-gradient-view-bar = Полоса
app-gradient-view-bar-hover = Прямая полоса — лучше всего подходит для точной расстановки точек
app-gradient-view-ring = Кольцо
app-gradient-view-ring-hover = Кольцо, потому что палитра ЦИКЛИЧЕСКАЯ: стык наверху — это шов, который рендерер пересекает при каждом проходе палитры, и градиент, который хорошо выглядит на полосе, может иметь там резкую границу.
app-gradient-add-lane-hover = Щёлкните в любом месте этой линии, чтобы добавить туда точку. + добавляет точку в самый широкий промежуток.
app-gradient-strip-hover = Перетащите маркер, чтобы сдвинуть его точку · перетащите саму полосу, чтобы ПОВЕРНУТЬ весь градиент с переходом через шов · двойной щелчок добавляет точку, правый щелчок по маркеру удаляет её · клавиши со стрелками сдвигают выделенное (с Shift — мельче).
app-gradient-remove-selected-hover = Удалить выбранную точку
app-gradient-ring-hover = Перетащите само кольцо, чтобы ПОВЕРНУТЬ весь градиент — то, что уходит за шов, возвращается с другой стороны · перетащите маркер, чтобы сдвинуть его точку · щёлкните по внешней дорожке, чтобы добавить туда точку, или по ⊕ в середине, чтобы добавить точку в самый широкий промежуток · ⊖ удаляет выбранную точку. Полая метка наверху — это шов, где циклическая палитра замыкается; она остаётся на месте, потому что она И ЕСТЬ стык, а цвета поворачиваются под ней.
app-gradient-empty = Своего градиента пока нет — его создаёт «Копировать пресет…», импорт или сохранённый градиент ниже, и здесь появятся его точки и кривые сегментов.
# $count: the entries of an imported palette, more than the editor shows stop by stop.
app-gradient-too-many = Импортировано записей: { $count } — слишком много, чтобы редактировать их по одной. Используйте предпросмотр выше или «Копировать пресет», чтобы начать заново.
# The selected segment's heading. $index: its number; $total: how many there are; $left and $right: where it starts and ends, 0.000–1.000.
app-gradient-segment-heading = Сегмент { $index } из { $total }  ·  { $left } → { $right }
# A label before the list of blend curves.
app-gradient-curve = Кривая
app-gradient-curve-hover = Как быстро меняется цвет вдоль этого сегмента. Форму «Изогнутой» задаёт средняя точка — при средней точке 0.50 она в точности линейна.
# The blend curves (GIMP's kinds), in a narrow list: keep them short.
app-gradient-blend-linear = Линейная
app-gradient-blend-curved = Изогнутая
app-gradient-blend-sine = Синус
app-gradient-blend-sphere-increasing = Сфера ↑
app-gradient-blend-sphere-decreasing = Сфера ↓
app-gradient-blend-bezier = Безье
# Hover on Bézier in the list of blend curves.
app-gradient-bezier-exact-hover = Две управляющие точки, которые можно перетаскивать в обоих направлениях. Эта кривая уже кубическая, поэтому преобразование ничего не меняет.
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-visible-hover = Две управляющие точки, которые можно перетаскивать в обоих направлениях. Преобразование ПРИБЛИЖАЕТ текущую кривую — эту с точностью до { $error }, что хорошо заметно.
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-barely-hover = Две управляющие точки, которые можно перетаскивать в обоих направлениях. Преобразование ПРИБЛИЖАЕТ текущую кривую — эту с точностью до { $error }, что едва заметно.
# A label before the list of colour spaces (RGB, HSV).
app-gradient-space = Пространство
# A label before the Bézier presets beside it.
app-gradient-ease = Плавность
app-gradient-ease-hover = Пресеты задают две управляющие точки; для всего остального перетаскивайте их на кривой. У кривой Безье нет средней точки — где она достигает половины, определяют её рукоятки.
# Bézier presets, as in CSS easing: short buttons.
app-gradient-ease-linear = Линейно
app-gradient-ease-in = Вход
app-gradient-ease-out = Выход
app-gradient-ease-in-out = Вход-выход
app-gradient-midpoint = Средняя точка
app-gradient-midpoint-hover = Где переход достигает половины, в долях этого сегмента. 0.50 — по центру.
# A button that puts the midpoint back in the middle of the segment.
app-gradient-centre = По центру
app-gradient-centre-hover = Вернуть 0.50
app-gradient-no-hue = ⚠ у одного конца этого сегмента нет оттенка
app-gradient-no-hue-hover = Один конец чёрный, белый или серый, поэтому переход начинается с красного и проходит весь цветовой круг. Это допустимо и иногда даже желательно — но именно поэтому в середине может оказаться цвет, которого нет ни на одном из концов. Полоса под кривой показывает, что происходит.
app-gradient-segment-start-hover = Цвет в НАЧАЛЕ этого сегмента
app-gradient-segment-end-hover = Цвет в КОНЦЕ этого сегмента
app-gradient-bytes-hover = Красный, зелёный и синий в диапазоне 0–255
app-gradient-floats-hover = Тот же цвет в диапазоне 0–1 — так его хранит рендерер
app-gradient-canvas-bezier-hover = Кривая перехода выбранного сегмента. Перетащите любую из управляющих точек — по x И по y, — чтобы изменить её форму. Бледные линии — это 0 и 1; кривая может выходить за них, и тогда ограничивается цвет, а не кривая.
app-gradient-canvas-hover = Кривая перехода выбранного сегмента: как быстро цвет меняется от левой точки к правой. Перетащите кольцо, чтобы сдвинуть среднюю точку, или переключите кривую на Безье, чтобы получить управляющую точку, которая движется в обоих направлениях.
# The selected stop's heading. $index: its number; $total: how many there are.
app-gradient-stop-heading = Точка { $index } из { $total }
app-gradient-stop-color-hover = Цвет в этой точке — щёлкните по образцу, чтобы выбрать, или введите рядом шестнадцатеричное значение
app-gradient-stop-bytes-hover = Красный, зелёный и синий в диапазоне 0–255 — байты, которыми записан этот цвет
# The eyedropper button.
app-gradient-pick = Пипетка
app-gradient-pick-hover = Пипетка — взять цвет из любого места экрана. Наведите указатель на нужный цвет и щёлкните; Esc — отмена.
# Written before the stop's position (0.000–1.000) in its number field: a short abbreviation of "position".
app-gradient-stop-pos = поз.
app-gradient-nudge-hover = Сдвинуть эту точку (удерживайте Shift для более мелкого шага)
app-gradient-remove = Удалить
app-gradient-remove-hover = Удалить эту точку (или щёлкните ⊖ под ней)
app-gradient-add-stop = Добавить точку
app-gradient-add-stop-hover = Разделить самый широкий сегмент (или щёлкните по линии над градиентом)
# A menu of the built-in palettes, to start from a copy of one.
app-gradient-copy-preset = Копировать пресет…
app-gradient-paste = Вставить…
app-gradient-paste-hover = Импортировать палитру из шестнадцатеричных цветов или троек RGB 0–255
app-gradient-save-ggr = Сохранить .ggr…
app-gradient-save-ggr-hover = Записать этот градиент в файл GIMP .ggr — его читают GIMP, Krita и Inkscape, а также этот редактор
# A menu of the palette file formats (.map, .ugr, .ggr, .ase).
app-gradient-import = Импорт
app-gradient-import-map-hover = Загрузить файл палитры Fractint / Kalles Fraktaler .map (строки R G B, по одной на запись)
app-gradient-import-ugr-hover = Загрузить файл градиентов Ultra Fractal .ugr (обычно .ugr содержит много градиентов — вы выбираете один)
app-gradient-import-ggr-hover = Загрузить градиент GIMP .ggr с сохранением его средних точек, кривых перехода и цветовых пространств
app-gradient-import-ase-hover = Загрузить список образцов Adobe (.ase). У образцов нет позиций, поэтому они располагаются с равными промежутками и плавными переходами.
# The gradient's name, for the library of saved gradients.
app-gradient-name = Имя
# Shown greyed in the empty name field.
app-gradient-name-hint = мой градиент
# The save button, when a gradient of that name is already saved.
app-gradient-update = Обновить
app-gradient-save = Сохранить
app-gradient-save-hover = Сохранить этот градиент в свою библиотеку вместе с кривыми и цветовыми пространствами
app-gradient-save-needs-name-hover = Сначала дайте градиенту имя
# A menu of the saved gradients. $count: how many there are.
app-gradient-saved-menu = Сохранённые ({ $count })
app-gradient-nothing-saved = Пока ничего не сохранено
app-gradient-delete-saved-hover = Удалить этот сохранённый градиент
# $count: the gradient's segments.
app-gradient-rich = Сложный градиент: в его сегментах (их { $count }) есть средние точки, кривые перехода или цветовые пространства — больше, чем может вместить простой список точек.
app-gradient-convert = Преобразовать в редактируемые точки
app-gradient-convert-hover = Заменить его простым списком точек. Это ОТБРАСЫВАЕТ средние точки, кривые перехода и переходы по кругу оттенков, которые список точек вместить не может.
# Above the list of gradients in an Ultra Fractal file. $count: how many; $file: the file's name.
app-gradient-ugr-list = Градиентов в { $file }: { $count }
app-gradient-close-list = Закрыть список
app-gradient-seamless = Бесшовный (конец совпадает с началом)
app-gradient-seamless-hover = Палитра циклическая, поэтому позиции 1 и 0 на экране соседствуют. Когда это включено, цвет последней точки поддерживается равным цвету первой — вид «Кольцо» показывает стык наверху.
app-gradient-ends-differ = ⚠ концы различаются
app-gradient-ends-differ-hover = Цвет в конце градиента не совпадает с цветом в начале, поэтому циклическая палитра показывает резкую границу там, где она замыкается.
app-gradient-hard-bands = Резкие полосы (без смешивания записей)
app-gradient-hard-bands-hover = Семантика Fractint / .map: каждая запись — сплошной цвет. Если выключить, записи смешиваются в плавный градиент.
app-gradient-paste-intro = Вставьте шестнадцатеричные цвета (#ff8800) или тройки 0–255 (255 136 0), разделённые запятыми, пробелами или переводами строк:
# Applies the pasted colours.
app-gradient-apply = Применить
# Empties the paste box.
app-gradient-clear = Очистить
# $used: the colours kept; $total: the colours pasted; $limit: the most that are kept.
app-gradient-paste-imported-some = Импортировано цветов: { $used } из { $total }, равномерно выбранных по вашему списку (предел — { $limit }).
# $count: the colours pasted.
app-gradient-paste-imported = Импортировано цветов: { $count }.
# $error: why the pasted text could not be read (in English).
app-gradient-paste-unreadable = Не удалось это прочитать: { $error }
# A hint at the bottom of the editor. $count: the palette's entries.
app-gradient-bands-hint = Полос: { $count } · берутся в порядке файла; пока полосы включены, позиции игнорируются.
# A hint at the bottom of the editor, in the ring view. $count: the gradient's stops.
app-gradient-ring-hint = Точек: { $count } · щёлкните по внешней дорожке, чтобы добавить туда точку · ⊕ добавляет в самый широкий промежуток, ⊖ удаляет выбранную точку · перетащите маркер, чтобы сдвинуть его · метка наверху — это шов
# A hint at the bottom of the editor, in the bar view. $count: the gradient's stops.
app-gradient-bar-hint = Точек: { $count } · щёлкните по линии над градиентом, чтобы добавить туда точку · перетащите маркер, чтобы сдвинуть его · ⊖ или правый щелчок удаляет
app-gradient-cancel = Отмена
app-gradient-cancel-hover = Отменить все изменения, сделанные с момента открытия этого окна, и закрыть его
app-gradient-ok = ОК
app-gradient-ok-hover = Сохранить эти изменения и закрыть редактор

## Shared

# The button that closes the Help window and the "Faster deep zoom" window.
app-close = Закрыть

## The "Faster deep zoom" window (Help menu): the optional accelerated build

app-accel-title = Ускоренное глубокое приближение
app-accel-active-heading = У вас запущена ускоренная сборка
app-accel-active-body = Опорные орбиты глубокого приближения вычисляются с помощью MPFR/GMP, что на этом шаге в 2,5–6,4 раза быстрее стандартной сборки, — это пауза перед тем, как глубокий вид начинает прорисовываться.
# $status: which arithmetic library is in use and its version (in English).
app-accel-in-use = Используемая арифметика: { $status }
app-accel-fallback-heading = Ускоренная сборка — используется встроенная арифметика
app-accel-fallback-body = Это ускоренная сборка (MPFR), но её математические библиотеки не найдены, поэтому используется встроенная арифметика (astro-float). Изображения идентичны; медленнее только пауза перед прорисовкой глубокого вида.
# Followed by the folder the program runs from, on its own line.
app-accel-fallback-dlls = Чтобы использовать быстрый путь, положите libmpfr-6.dll и libgmp-10.dll (а также libgcc_s_seh-1.dll, libwinpthread-1.dll) рядом с fractadyne.exe и перезапустите программу:
app-accel-warning-hidden = Предупреждение об этом при запуске скрыто.
app-accel-show-again = Показывать снова
app-accel-offer-heading = Необязательная сборка в 2,5–6,4 раза быстрее на глубине
app-accel-offer-body = Глубокое приближение тратит много времени на вычисление опорных орбит на CPU — это пауза перед тем, как глубокий вид начинает прорисовываться. Необязательная сборка делает это с помощью библиотек MPFR/GMP, что быстрее в 2,5–6,4 раза, и тем больше, чем глубже вы погружаетесь. Всё остальное — то же самое.
app-accel-offer-identical = Изображения ПОБАЙТОВО ИДЕНТИЧНЫ — это та же математика, вычисленная более быстрой библиотекой; проверено на всех формулах фракталов и на всём сравнительном корпусе глубокого приближения.
app-accel-offer-shared = Ваши настройки, сохранённый сеанс и местоположения общие для обеих сборок, поэтому можно свободно переключаться, и ничего не нужно импортировать.
# Shown on Windows.
app-accel-separate-windows = Это отдельная загрузка, потому что используемые ею библиотеки нельзя собрать компилятором, которым собирается стандартная сборка для Windows, и у них другая лицензия (GNU LGPL v3), чем у самого Fractadyne.
# Shown on other systems.
app-accel-separate = Это отдельная загрузка, потому что у используемых ею библиотек другая лицензия (GNU LGPL v3), чем у самого Fractadyne.
app-accel-download = Скачать для этой версии
app-accel-all-releases = Все выпуски
app-accel-all-releases-hover = Если прямая ссылка выдаёт 404, для этой версии ещё нет ускоренной сборки — выберите здесь ближайшую.
# Shown on Windows.
app-accel-install-windows = Распакуйте архив и запускайте fractadyne.exe из этой папки, оставив файлы .dll рядом с ним.
# Shown on Linux.
app-accel-install-linux = Распакуйте архив и запускайте ./fractadyne из этой папки. Сборка использует системные GMP и MPFR (libgmp10 / libmpfr6), которые есть почти на любом настольном компьютере, — см. README-ACCELERATED.txt в пакете.
# Shown on other systems.
app-accel-install-other = Для этой платформы ускоренный пакет пока не собирается; его можно собрать из исходного кода с --features fractadyne-core/rug.

## The Help window's frame (its sections are under help-)

app-help-title = Справка Fractadyne

## Checking for updates

app-toast-up-to-date = У вас последняя версия.
# $error: why the check failed (in English).
app-toast-update-failed = Не удалось проверить обновления: { $error }
# The window that offers a newer version.
app-update-title = Доступно обновление
# $version: the new version (0.3.0-beta.20). $channel: the token "stable" or "beta", in English; a translation can choose its words by it ({ $channel -> [beta] … *[other] … }).
app-update-version =
    { $channel ->
        [beta] Доступна версия Fractadyne { $version } (бета)
       *[other] Доступна версия Fractadyne { $version } (стабильная)
    }
# $current: the version running now. $track: the update track the user follows (its name in the language in use).
app-update-running = Установлена версия { $current }  ·  { $track }
app-update-download-intro = Скачайте последний выпуск с GitHub:
app-update-download = Скачать с GitHub
app-update-later = Напомнить позже

## The second graphics card (shown in its window, after "In use:" or "Could not start:")

# The second device is the same card as the window's. $name: the card's name.
app-worker-same-card = та же видеокарта, { $name }
# Why the second device could not start.
app-worker-thread-stopped = поток устройства остановился

## Export status (shown in the Export image window)

app-export-gpu-unavailable = GPU недоступен
app-export-reference-aborted = Экспорт не удался: построение опорной орбиты прервано.

## The performance panel (View ▸ Performance panel)
## Its row labels sit in a monospace column about 14 characters wide: keep them short (a longer label pushes its value to the right).

app-perf-fps = FPS
app-perf-fps-hover = Кадров, выведенных на экран в секунду, по сглаженному интервалу между кадрами. Пока идёт доводка вида, здесь показывается её ход — во время доводки вид не работает с той частотой, которую подразумевало бы число здесь.
# What the FPS row shows while a view is settling. $done and $total: tiles; $eta: the time left, as m:ss.
app-perf-refining-tiles = доработка { $done }/{ $total } (~{ $eta })
app-perf-refining-arming = доработка (подготовка)
# $percent: how much is done, with one decimal.
app-perf-refining-percent = доработка { $percent }%
app-perf-building-reference = построение опорной орбиты
app-perf-idle = простой
app-perf-frame = кадр
app-perf-frame-hover = Сглаженное реальное время между кадрами — не сумма строк ниже. Оно включает ожидание вывода на экран, поэтому на готовом неподвижном виде отражает частоту перерисовки, а не какую-либо работу.
app-perf-cpu = cpu
app-perf-cpu-hover = Время главного потока внутри одного обновления: подготовка параметров и управление кэшем опорных орбит. Именно эта строка подскакивает при перестройке опорной орбиты, и такой скачок — это и есть ощущаемое вами подтормаживание.
app-perf-gpu-idle = gpu/простой
app-perf-gpu-idle-hover = Время кадра минус время CPU главного потока. Пока вид рендерится, это работа GPU, а на готовом виде — в основном ожидание в простое; в названии есть и то и другое, потому что одним этим вычитанием панель их не различает.
app-perf-timing = замер
# How frame cost is measured, the timing row's value. TIMESTAMP_QUERY is a name.
app-perf-timing-starved = реальное время (нет меток времени)
app-perf-timing-gpu = метки времени GPU
app-perf-timing-no-query = реальное время (нет TIMESTAMP_QUERY)
app-perf-timing-hover = Как измеряется стоимость кадра. Метки времени GPU — точный источник; запасной вариант по реальному времени включается, когда замеры перестают поступать, а бюджет, рассчитанный по реальному времени, не раз неверно оценивал глубокие кадры, — понимайте красный цвет здесь как «запасы прочности сейчас лишь оценочные».
app-perf-mode = режим
app-perf-mode-hover = Арифметический путь, которым был отрендерен последний кадр: прямой (без опорной орбиты), возмущения df32 или floatexp — путь с расширенным диапазоном, используемый примерно после 1e28x, и единственный, на котором работает BLA.
app-perf-eff-iter = эфф. итер.
app-perf-eff-iter-hover = Итерации, фактически использованные последним кадром, после масштабирования по глубине и любого ограничения во время движения, — а не базовая настройка в разделе «Качество», которая лишь отправная точка.
app-perf-precision = точность
app-perf-precision-hover = Число бит мантиссы, с которым вычисляется опорная орбита. Оно растёт с глубиной; 64 бита — нижний предел для малых глубин, а не ограничение, в которое вы упёрлись.
app-perf-orbit-len = длина орбиты
# $cap: the most orbit samples this GPU holds, already formatted, or the text of app-perf-uncapped.
app-perf-orbit-len-hover = Отсчёты опорной орбиты. Этот GPU вмещает не более { $cap } — приближение к этому пределу и есть практическая стена глубины: за ней живой вид не может прорисовать более глубокие места, которым нужна более длинная орбита.
app-perf-uncapped = (без ограничения)
app-perf-sa-skip = пропуск SA
app-perf-sa-skip-hover = Итерации, пропущенные аппроксимацией рядами до начала попиксельной работы. Больше — быстрее, и на изображение это никак не влияет; если ничего не пропущено, строки нет.
app-perf-aa = aa
app-perf-aa-hover = Суперсэмплинг: каждый пиксель рендерится из такого числа отсчётов по каждой оси, поэтому 2x стоит вчетверо больше работы.
app-perf-dual = двойной
app-perf-dual-hover = Работает ли разделённый вид Mandelbrot/Жюлиа. Если да, каждая стоимость на этой панели охватывает ОБЕ половины.
app-perf-zoom = масштаб
app-perf-zoom-hover = Увеличение относительно полного вида. Арифметический режим выше меняется примерно на 1e4x и ещё раз примерно на 1e28x.
app-perf-iterate = итерирование
# $target and $lethal: times in milliseconds, already formatted.
app-perf-iterate-hover = Измеренное время GPU последнего запуска итерирования. Контроллер нацелен на { $target } ms; устойчивые показания около { $lethal } ms — это диапазон, в котором оборудование этого класса теряло устройство.
app-perf-steps = шагов/с
app-perf-steps-hover = Шагов итерации в секунду на GPU, в миллиардах. Виды с преобладанием внутренней области работают в пересчёте на шаг медленнее, чем подсказывает само число, так что это показатель пропускной способности, а не оценка.
app-perf-budget = бюджет
# After the budget's value while it is still converging; the hover below quotes it.
app-perf-budget-settling = (сходится)
app-perf-budget-hover = Адаптивный бюджет шагов на запуск — защита контроллера от кадра, достаточно длинного, чтобы драйвер сбросил GPU. «Сходится» означает, что бюджет ещё сходится по редким замерам и пока не является надёжным потолком.
app-perf-rss = rss
# $mb and $peak: memory in megabytes, already formatted.
app-perf-rss-value = { $mb } MB (пик { $peak })
app-perf-rss-hover = Резидентная память процесса и наибольшее значение, которого она достигала за этот сеанс. Построение глубоких опорных орбит на практике превышало 2 GB, причём больше ничего на экране об этом не говорило; янтарный цвет — свыше трёх четвертей оперативной памяти системы.
app-perf-gpu-est = gpu, оценка
app-perf-gpu-est-hover = Оценка памяти, занятой на GPU: опорные орбиты, деревья BLA, текстуры итераций, попеременные буферы состояния фрагментов во время доработки и снимок для шлюза вывода. Собрана по известным размерам выделений; истинную цифру драйвера переносимым способом прочитать нельзя.
app-perf-julia-re = Жюлиа c.re
app-perf-julia-re-hover = Действительная часть параметра c, который рендерит половина Жюлиа, — точка под курсором, если она не закреплена.
app-perf-julia-im = Жюлиа c.im
app-perf-julia-im-hover = Мнимая часть параметра c, который рендерит половина Жюлиа.
app-perf-julia-c = Жюлиа c
# The julia c row's value when the parameter is held fixed.
app-perf-pinned = закреплён
app-perf-pinned-hover = Параметр закреплён в фиксированной точке, поэтому движение курсора больше не меняет половину Жюлиа.
app-perf-c-panel = c/панель
app-perf-c-panel-hover = Ширина области c, охватываемой всей панелью Mandelbrot. Когда она гораздо меньше одного пикселя Жюлиа, наведение меняет c, но Жюлиа выглядит неизменной, — на глубоком приближении это ожидаемо и не является зависанием.
app-perf-ref-recompute = пересчёт орб.
app-perf-ref-recompute-hover = Длительность последнего пересчёта опорной орбиты. Это пауза перед тем, как глубокий вид начинает прорисовываться, и тот самый шаг, который ускоряет необязательная ускоренная сборка.
app-perf-recompute-rate = пересчётов/с
app-perf-recompute-rate-hover = Опорные орбиты, УСТАНОВЛЕННЫЕ в секунду, — те, чей результат действительно использован. Сравните со строкой ниже: разница между ними — выброшенная работа.
app-perf-builds-rate = построений/с
app-perf-builds-rate-hover = Построения опорных орбит, ЗАПУЩЕННЫЕ в секунду, включая те, чей результат отброшен, — слот упреждения, до которого погружение так и не дошло, или отклонённое расширение. Они стоят полного CPU и больше нигде не видны: однажды холостая предвыборка шесть минут выполняла около 400 построений в секунду, пока «пересчётов/с» показывало 2. Однозначные числа — норма; большой разрыв между этими двумя строками значит, что работа выбрасывается.
app-perf-recompute-total = пересчётов всего
app-perf-recompute-total-hover = Пересчёты опорных орбит с момента запуска. Полезно как скорость за известный интервал; число, растущее, пока ничего не движется, — сигнал, который стоит расследовать.

## Help: the contents list (Help & reference…, F1)

# Each section's name in the contents list (about 165 px wide: keep them short), and its heading.
help-section-overview = Обзор
help-section-navigation = Навигация
help-section-expressions = Выражения координат
help-section-options = Раскраска и параметры
help-section-fractals = Фракталы
help-section-how-it-works = Как это работает
help-section-command-line = Командная строка
help-section-shortcuts = Сочетания клавиш
help-section-hardware = Рекомендуемое оборудование
# Credits to the prior work Fractadyne builds on.
help-section-acknowledgments = Благодарности
# The third-party license notices.
help-section-licenses = Лицензии
help-section-about = О программе

## Help ▸ Overview

help-overview-intro = Fractadyne — нативная программа для исследования фракталов, созданная для сверхглубокого приближения и скорости. Она рисует фракталы «времени убегания» — изображения, которые получаются повторением одной простой формулы в каждом пикселе.
help-overview-what-is = Что такое фрактал времени убегания?
help-overview-what-is-text = Для каждого пикселя программа снова и снова применяет формулу вроде z → z² + c, начиная с нуля. Если текущее значение навсегда остаётся малым, пиксель принадлежит множеству и рисуется тёмным. Если оно в конце концов неограниченно растёт («убегает»), пиксель лежит вне множества, а его цвет показывает, сколько шагов на это ушло. Бесконечно детализированная граница между «остаётся» и «убегает» и есть фрактал.
help-overview-can-do = Что можно делать
help-overview-can-pan = Сдвигать и приближать вид практически без ограничений (положение точно на любой глубине).
help-overview-can-families = Переключаться между 25 встроенными семействами фракталов (десять, а пять из них — ещё и в высших степенях), своими формулами, Life и L-системами, а любое семейство времени убегания, кроме Newton, просматривать как множество Жюлиа.
help-overview-can-recolor = Перекрашивать изображение готовыми или своими градиентами и несколькими методами раскраски.
help-overview-can-lighting = Добавлять объёмное рельефное освещение и светящиеся контуры границы.
help-overview-can-minibrots = Притягиваться к минибротам, сохранять места в закладки и экспортировать изображения высокого разрешения.
help-overview-can-autopilot = Позволить автопилоту автоприближения самому погружаться к деталям.
help-overview-can-share = Делиться любым местом в виде небольшого текстового фрагмента или файла «.fdn».
help-overview-can-tours = Запускать туры по сценариям и тест производительности оборудования.
help-overview-first-steps = Первые шаги
# "Seahorse Valley" is a place in the Navigate menu: use its name as that menu shows it. A and F1 are keys.
help-overview-first-steps-text = Откройте меню «Навигация» и выберите «Seahorse Valley», затем покрутите колесо мыши, чтобы приблизиться. Перетаскивайте, чтобы сдвигать вид. Или просто нажмите A, и автопилот погрузится за вас. Нажмите F1 в любой момент, чтобы вернуться к этой справке.

## Help ▸ Coordinate expressions
## The expressions, numbers and function names here are typed as shown in every language:
## keep them exactly as written. Menu items in quotation marks are the menus' own labels.

help-expr-intro = Везде, где Fractadyne принимает координату, вместо десятичного числа можно ввести выражение: в полях Re / Im окна Навигация → «Перейти к местоположению…» (и в каждом поле его полярного режима), в строках center_re / center_im местоположения .fdn, в re / im сценария тура и в --center в командной строке. Выражение вычисляется с таким числом цифр, какое нужно для текущего масштаба, а диалог перехода хранит его вместе с видом — поэтому точка, введённая как 1/3 или как -0.5 + 0.25*cos(pi/4), остаётся точно на месте, как бы глубоко вы ни погрузились, тогда как десятичное число начало бы смещаться, когда закончатся его цифры.
help-expr-examples = Примеры
help-expr-ex-third = Точная треть. По мере приближения появляются новые цифры.
help-expr-ex-neck = Перешеек Seahorse Valley — корень почки периода 2.
help-expr-ex-circle = Точка на окружности радиуса 1/4 вокруг -0.5 (полярный режим составит это выражение за вас).
help-expr-ex-bulb = Re корня почки периода 3; его Im равно sin(2*pi/3)/2 - sin(4*pi/3)/4. В Навигация → «Точные точки (выражения)» такие точки уже встроены.
help-expr-ex-complex = Целое комплексное значение, введённое в поле Re, заполняет и Re, и Im.
help-expr-ex-combine = Функции и константы свободно сочетаются.
help-expr-ex-powers = Степени: 2^-20 — это 1/1048576. -2^2 равно -4 — минус применяется после возведения в степень.
help-expr-ex-root = -2, действительный кубический корень. Корни чётной степени из отрицательного числа отклоняются; sqrt(-1) даёт i.
help-expr-ex-scientific = Экспоненциальная запись тоже работает внутри выражений.
help-expr-numbers = Числа и операторы
# The decimal point in an expression is '.' in every language: keep that rule as it is.
help-expr-numbers-decimals = Десятичные числа с необязательной экспонентой: 0.25, -1.5e-30, .5. Только цифры — без пробелов и запятых-разделителей разрядов внутри числа, а десятичный разделитель — всегда «.».
help-expr-numbers-imaginary = Добавьте суффикс i для мнимого числа (0.25i) или используйте i отдельно: 3 + 4*i.
# Starts with the operators themselves, two spaces apart.
help-expr-numbers-operators = +  -  *  /  и ^ (степень; вычисляется справа налево, поэтому 2^3^2 = 2^9). Скобки группируют. Неявного умножения нет: пишите 2*pi, а не 2pi.
help-expr-numbers-case = Регистр в именах не важен: PI, Pi и pi — одна и та же константа.
help-expr-constants = Константы
help-expr-functions = Функции
help-expr-functions-angles = Углы задаются в радианах. Для градусов умножайте на pi/180 — cos(45*pi/180) — или переключите диалог перехода в полярный режим и выберите единицу «градусы».
help-expr-refused = Что отклоняется и почему
help-expr-refused-fractional = Отрицательное число в дробной степени, например (-2)^(1/2): результат комплексный, и корней, из которых надо выбирать, несколько. Используйте root(x, n) или cbrt(x) для действительного корня либо sqrt(x) для главного комплексного.
help-expr-refused-complex = Комплексный аргумент у функции с действительными значениями (sin(i), ln(1+i)). Комплексный аргумент принимают только sqrt и abs; арифметические операторы и целые степени работают с комплексными значениями.
help-expr-refused-domain = Ошибки области определения: ln или log от неположительного числа, asin или acos вне -1…1, 0 в отрицательной степени, деление на ноль.
help-expr-refused-bounds = Аргументы sin, cos, tan, exp или ^ за пределами ±2^32 и вложенность глубже 32 уровней: вставленная координата — недоверенный ввод, а такие выражения вычислялись бы неограниченно долго.
help-expr-refused-grammar = Никаких переменных, пользовательских функций, сравнений или условий. Грамматика намеренно сделана маленькой, чтобы сохранённое местоположение означало одно и то же в любой версии.
help-expr-no-parse = Если выражение не разбирается
help-expr-no-parse-text = Сообщение называет поле, говорит, что было найдено и на каком символе, а для обычных опечаток — что ввести вместо этого: типографский знак минуса (−), вставленный из документа, × или ÷, десятичную запятую, пропущенный * между числом и именем, незакрытую скобку или функцию, записанную без скобок. Ничего не читается наполовину: если какую-то часть текста понять нельзя, отклоняется вся координата, а не молча укорачивается.

## Help ▸ Navigation
## Menu paths (Navigate → "Go to location…") and quoted items name the interface's own
## menus and controls: use their translations. Key names (Ctrl, Shift, Space, Esc) stay.

# A heading: the mouse controls (also in Shortcuts).
help-mouse = Мышь
# The left column of a controls table: a gesture or key. Keep key names (Shift, Space, Ctrl) as they are.
help-key-left-drag = Перетаскивание левой кнопкой
help-nav-left-drag = Сдвинуть вид.
help-key-mouse-wheel = Колесо мыши
help-nav-wheel = Приблизить/отдалить к курсору; чем быстрее крутите колесо, тем дальше приближение за один щелчок. Когда включено «Притяжение к объектам», приближение рядом с минибротом притягивается к нему.
help-key-shift-drag = Shift+перетаскивание
help-nav-shift-drag = Приближение рамкой — растяните прямоугольник, и он будет приближен на весь вид.
# $icon: the click-to-zoom toolbar button's symbol.
help-key-click-zoom = Приближение щелчком ({ $icon })
help-nav-click-zoom = Включите кнопку-мишень на панели инструментов (одиночный вид): тогда левый щелчок ныряет в точку с заданным коэффициентом, а правый возвращает назад. Перетаскивание по-прежнему сдвигает вид; коэффициент (2–100×) выбирается в разделе «Навигация» правой панели. Backspace отменяет щелчок.
help-nav-continuous = Непрерывное приближение и возврат домой
help-key-hold-space = Удерживать Space
help-nav-hold-space = Плавно приближать с привязкой к курсору.
help-key-hold-shift-space = Удерживать Shift+Space
help-nav-hold-shift-space = Плавно отдалять.
help-nav-continuous-text = Кнопка возврата домой анимирует плавный обратный полёт к полному виду. «Скорость приближения» в правой панели задаёт темп непрерывного приближения.
help-nav-autopilot = Автопилот
help-nav-autopilot-text = Нажмите A (или Инструменты → «Автоприближение (автопилот)»), чтобы погрузиться без участия рук: автопилот выбирает на краю фрактала точку, где детали плотнее всего, предпочитая середину вида, и плавно погружается в неё, постепенно сводя её к центру экрана. Он держит эту цель, пока она остаётся интересной, и плавно поворачивает к более богатой структуре, когда та появляется. Любое действие навигации — или Esc — останавливает его. «Приоритет автоприближения» в правой панели (или в меню «Инструменты») выбирает, что для него главное: «Скорость» приближает с заданным темпом и обновляет экран, как только готов кадр с деталями; «Качество» показывает только полностью проработанные кадры в полном разрешении и при необходимости замедляет приближение, чтобы изображение никогда не увеличивалось сильно до прихода следующего кадра. В любом случае на экране никогда не бывает пустого или одноцветного кадра: обновление, в котором ещё не проработаны детали, остаётся скрытым за предыдущим. «Цель автоприближения» выбирает, куда он нацелен: «Детали» (самый богатый край в поле зрения, выбираемый заново по ходу) или «Точка Мисюревича» — он находит ближайшую точку Мисюревича, вычисляет её до глубины предела погружения, стоя на месте, а затем погружается точно в неё; структура вокруг такой точки бесконечно повторяется через каждые несколько октав, поэтому такое погружение никогда не заходит в тупик (только множество Мандельброта).
help-nav-history = История и точные перемещения
help-nav-undo = Отменить предыдущий вид.
help-nav-redo = Повторить.
help-nav-goto = Навигация → «Перейти к местоположению…» позволяет прочитать, ввести, вставить или скопировать точный центр и масштаб (полная точность, любая глубина). Поля центра принимают не только простые числа: дроби вроде «1/3» и выражения из pi, tau, e, sqrt и sin/cos/tan — так что «-0.5 + 0.25*cos(pi/4)» — допустимая координата. Выражение хранится вместе с видом и заново вычисляется с той точностью, которая нужна каждой глубине, поэтому введённая так точка остаётся точной, как бы далеко вы ни приближались (округлённое десятичное число со временем перестало бы быть точным). Полярный режим (смещение + r ∠ θ, в градусах / радианах / оборотах) составит такое выражение за вас. Полный список функций и констант с примерами — в разделе «Выражения координат» этой справки (его открывает кнопка ? в диалоге). Навигация → «Закладки» сохраняет и вызывает местоположения.
help-nav-exact-points = Навигация → «Точные точки (выражения)» переходит к точкам, заданным точно, а не округлённым десятичным числом, — к корням почек главной кардиоиды, где прикрепляется каждая почка периода q, записанным как выражения cos/sin. Поскольку выражение путешествует вместе с видом, в такую точку можно приближаться, и вид никогда с неё не сместится.
help-nav-share = Навигация → «Поделиться местоположением…» сохраняет весь вид — фрактал, центр с полной точностью, масштаб и раскраску — в виде компактного фрагмента «.fdn», который можно скопировать, вставить обратно и применить или сохранить/загрузить как файл, чтобы точно воспроизвести место (или отправить его кому-нибудь).
help-nav-open = Файл → «Открыть вид или местоположение…» напрямую открывает любой из них: экспортированный PNG/EXR (встроенный в него вид восстанавливается), файл местоположения «.fdn» или «.kfr» Kalles Fraktaler.
help-nav-finding = Поиск деталей
help-nav-minibrots = Миниброты — крошечные копии всего множества, спрятанные вдоль границы. Примерно поместите один из них в центр и нажмите M (или Инструменты → «Найти миниброт и приблизиться к нему»), чтобы методом Ньютона точно притянуться к его центру, узнать его период и сразу приблизиться до его собственного масштаба — часто на много порядков за один шаг. Если вы уже глубже миниброта, сдвинется только центр.
help-nav-minimap = Вид → «Мини-карта» показывает небольшую карту всего множества с меткой «вы здесь» и текущей глубиной приближения; щёлкните по карте, чтобы перейти к области.

## Help ▸ Coloring & options
## Names in quotation marks are the controls' own labels in the right-hand panel.

help-options-intro = Всё это находится в правой панели (и сохраняется между сеансами).
help-options-palette = Палитра
help-options-palette-text = «Палитра» выбирает готовый градиент, ваш собственный («Своя») или двухцветный режим. «Редактировать градиент…» открывает редактор, в котором у каждой точки градиента есть цвет и позиция (0–1); добавляйте точки, скопируйте пресет в качестве основы или импортируйте файл палитры.
help-options-paste = «Вставить…» в редакторе градиента импортирует палитру, найденную где-то ещё. Принимаются шестнадцатеричные цвета с ведущим # или без него (#ff8800, ff8800 или короткое #f80) и простые тройки RGB 0–255, например «255 136 0», — формат, который используют файлы .map Fractint и Kalles Fraktaler, — разделённые запятыми, пробелами или переводами строк. Сохраняется до 256 цветов; более длинный список равномерно прореживается до 256 с сохранением первого и последнего цвета, чтобы форма градиента не пострадала.
help-options-import-map = «Импорт ▸ .map…» рядом с ним загружает файл палитры .map Fractint или Kalles Fraktaler. Это таблицы соответствия, индексированные числом итераций, без смешивания записей, и получающиеся резкие ступени цвета — это и есть классический облик Fractint, поэтому так они и импортируются; снятый флажок «Резкие полосы» вместо этого смешивает те же цвета в плавный градиент. Собственные файлы Fractint хранят 6-битные значения цвета, умноженные на четыре, поэтому белый у них — 252, а не 255; Fractadyne сообщает об этом при импорте и оставляет значения ровно такими, как они записаны, потому что в изображениях, сохранённых самим Fractint, стоит то же 252.
help-options-duotone = «Двухтоновая» отображает значение на плавный двухцветный переход («Тень» → «Свет»). «Двоичная (множество)» — плоский двухцветный вид: один сплошной цвет для точек внутри множества и другой — для точек снаружи, без градиента; это самый наглядный способ увидеть форму множества.
help-options-cycle = «Цикл» задаёт, сколько раз градиент повторяется на диапазоне итераций (более частые или более редкие полосы). «Сдвиг» поворачивает весь градиент.
help-options-normalize = «Нормализовать глубокие цвета» подгоняет палитру под диапазон значений убегания, реально присутствующих в виде. Глубокие виды охватывают сотни тысяч итераций, и без этого фиксированный цикл оборачивает градиент тысячи раз между соседними пикселями, превращая правильные детали в рябь.
help-options-log-scale = «Логарифмическая шкала цвета» распределяет палитру по логарифму значения убегания, а не линейно. По мере погружения числа итераций до убегания скапливаются у верхнего края, поэтому линейная подгонка тратит большую часть градиента на тонкую оболочку у самой границы, а остальное оставляет почти плоским; логарифмическая шкала раскрывает это и делает цвета стабильнее на протяжении видео с приближением, когда диапазон растёт.
help-options-animate = «Анимация» циклически меняет цвета со временем: «Вперёд», «Назад» или «Туда-обратно» сдвигают смещение; «Случайные градиенты» непрерывно синтезируют плавно перетекающие гармоничные градиенты. «Скорость» задаёт число циклов (или перетеканий) в секунду; «Перемешать градиент» создаёт новый градиент в случайном режиме.
help-options-method = Метод раскраски — как данные становятся цветом
# The coloring methods, by the names the Coloring panel gives them, each with what it does.
help-method-smooth = Плавные итерации
help-method-smooth-text = Классические непрерывные полосы по времени убегания.
help-method-stripe = Среднее полос
help-method-stripe-text = Плавные полосы по углу орбиты (ползунок «Плотность полос»).
help-method-triangle = Неравенство треугольника
help-method-triangle-text = Тонкая текстура по тому, куда каждый шаг попадает между границами.
help-method-trap = Ловушка орбиты
help-method-trap-text = Расстояние орбиты до фигуры (точка/крест/окружность); раскрашивает и внутреннюю область.
help-method-distance = Оценка расстояния
help-method-distance-text = Оттенок по близости к границе.
help-method-decomposition = Декомпозиция
help-method-decomposition-text = Ячейки по конечному углу убегания.
help-options-relief = Рельефное 3D-освещение
help-options-relief-text = Затеняет поверхность по наклону границы (производной), создавая выпуклый, освещённый вид. «Угол света» задаёт направление, «Рельеф» — силу (меньше = резче), «Вращать свет» анимирует его. Только голоморфные семейства (Mandelbrot / Multibrot).
help-options-glow = Свечение по расстоянию
help-options-glow-text = Яркие контурные полосы, которые сгущаются в светящиеся нити у границы. «Свечение» — степень смешивания, «Ширина полос» — интервал между ними, а «Анимировать свечение» заставляет их течь.
help-options-quality = Качество и итерации
help-options-quality-text = «Итерации» — максимальное число шагов, после которого пиксель считается лежащим внутри множества; «Автомасштаб» автоматически увеличивает его при приближении (более глубоким деталям нужно больше). «Сглаживание» выполняет суперсэмплинг неподвижных изображений (2×–8×), когда вид устоится, укрощая мелкую внешнюю «пыль».
help-options-other = Прочее
help-options-other-text = «Скорость приближения» задаёт темп непрерывного приближения; ограничение частоты кадров ограничивает FPS; «Оба, связанные» показывает множество Мандельброта и его множество Жюлиа рядом (курсор задаёт параметр Жюлиа на лету).

## Help ▸ Fractals
## Fractal family names (Mandelbrot, Multibrot, Tricorn, Burning Ship, Celtic, Buffalo,
## Phoenix, Newton) are names: keep them as written, as the formulas. Formula syntax
## (z = z^3 - p1*z + c, sqr, |z|, if / elseif / endif, p1-p5, maxit) is typed as shown.

help-fractals-intro = Каждое семейство итерирует формулу, в которой z начинается с 0, а c задаётся пикселем (время убегания), если не сказано иное. z = x + iy — комплексное число.
help-fractals-mandelbrot = z → z² + c. Оригинал. Множество — это все c, орбита которых остаётся ограниченной; оно связно, а его граница настолько изрезана, что имеет хаусдорфову размерность 2.
help-fractals-multibrot = z → z^d + c для степени d = 3, 4, 5. Более высокие степени добавляют лепестки: множество обладает вращательной симметрией порядка d−1 (Multibrot 3 — порядка 2, 4 — порядка 3, 5 — порядка 4).
help-fractals-tricorn = z → conj(z)² + c, где conj(x + iy) = x − iy. Сопряжение делает отображение антиголоморфным, что даёт симметрию порядка 3 и характерные изогнутые «когти».
help-fractals-burning-ship = z → (|x| + i|y|)² + c — перед возведением в квадрат берутся абсолютные значения (действительная часть = x²−y²+cx, мнимая = 2|xy|+cy). Неаналитично; глубокие приближения открывают структуры, похожие на корабли (традиционно их смотрят перевёрнутыми).
help-fractals-celtic = Как Mandelbrot, но с модулем действительной части: действительная часть = |x²−y²| + cx, мнимая = 2xy + cy. Даёт мотивы в форме сердец и щитов.
help-fractals-buffalo = Модуль обеих частей z²: действительная часть = |x²−y²| + cx, мнимая = |2xy| + cy — нечто среднее между Celtic и Burning Ship.
# A heading: the families at powers above 2 (the Fractal menu's "Higher powers").
help-fractals-powers = Высшие степени
# ("Burning Ship  3 4 5") is how the Fractal menu shows such a row: keep it as it is.
help-fractals-powers-text = Multibrot 6, 7 и 8 (z^d + c), а также Burning Ship, Tricorn, Celtic и Buffalo в степенях 3, 4 и 5: те же складки в более высокой степени. Burning Ship d берёт модули частей z, а затем степень; Tricorn d сначала выполняет сопряжение; Celtic d и Buffalo d складывают действительную часть (а Buffalo — и мнимую) z^d. Меню «Фрактал» и выпадающий список показывают их по одному семейству в строке («Burning Ship  3 4 5»). Каждое из них поддерживает глубокое приближение так же, как его собрат степени 2, а Multibrot 6, 7 и 8, как и Multibrot 3–5, поддерживают также аппроксимацию рядами и поиск минибротов (M). Multibrot 8 убегает на радиусе 128 вместо 256, где его следующий шаг вышел бы за пределы диапазона одинарной точности.
help-fractals-phoenix = z → z² + c + p·z(n-1), где z(n-1) — предыдущая итерация, а p — константа (p = -0.5). Член памяти создаёт пламеподобные нити.
help-fractals-newton = Метод Ньютона для корней z³ − 1 = 0: z → z − (z³−1)/(3z²). Вместо времени убегания пиксели окрашиваются по тому, к какому из трёх кубических корней из единицы сходится итерация (бассейны притяжения), и по тому, как быстро; запутанные границы бассейнов и есть фрактал.
help-fractals-julia = Режим Жюлиа
help-fractals-julia-text = Для любого семейства, кроме Newton, можно переключиться на множество Жюлиа: вместо того чтобы начинать z с 0 при c = пиксель, вы фиксируете c (параметр), а z начинается с пикселя. Множество Жюлиа — это граница между начальными точками, которые остаются ограниченными, и теми, что убегают, при этом фиксированном c. В режиме «Оба, связанные» движение курсора над панелью Mandelbrot задаёт c на лету.
help-fractals-deep = Поддержка глубокого приближения
help-fractals-deep-text = Mandelbrot, Multibrot 3/4/5 и Tricorn поддерживают глубокое приближение методом возмущений на экстремальной глубине (floatexp). Burning Ship, Celtic и Buffalo неаналитичны (в них берутся модули), поэтому используют метод возмущений с учётом знака; теперь он тоже работает в диапазоне floatexp и позволяет приближаться далеко за прежний предел df32 ~1e28× (редкая рябь у складок модуля ждёт исправления глитчей с несколькими опорными точками). Phoenix использует двучленный метод возмущений и тоже поддерживает глубокое приближение в диапазоне floatexp (без SA/BLA, поэтому он тяжелее, чем Mandelbrot). Высшие степени приближаются так же: метод возмущений каждой степени следует собственной цепочке возведений в квадрат; Multibrot 3–8, как и Mandelbrot, поддерживают также аппроксимацию рядами и BLA, а Tricorn, Burning Ship, Celtic и Buffalo во всех степенях поддерживают BLA (без рядов), которая проносит внутреннюю область глубокого вида целиком, вместо того чтобы итерировать её шаг за шагом. Только Newton использует прямой путь, резкий до ~1e6×; свои формулы поддерживают глубокое приближение, если в разделе «Свои формулы» ниже не сказано иное.
help-fractals-custom = Свои формулы
help-fractals-custom-text = Фрактал ▸ Своя формула… принимает шаг формулы в виде выражений в стиле Fractint: z = z^3 - p1*z + c или несколько операторов (t = sqr(z), затем z = t*t + c). z — итерируемое значение, c — пиксель (константа Жюлиа в режиме Жюлиа), p1-p5 — параметры; |z| — квадрат модуля, как во Fractint, и доступны обычные функции (sqr, abs, conj, real, imag, cabs, flip, recip, ident, exp, log, sqrt, sin, cos, tan, cotan, sinh, cosh, tanh, cotanh, asin, acos, atan, asinh, acosh, atanh, cosxx, а также floor, ceil, trunc и round, которые округляют каждую часть). log — натуральный логарифм, как во Fractint и в комплексном анализе: то, что во многих учебниках пишут как ln (десятичного логарифма нет). Формула, записанная с ln, cot, coth, Re или Im, не читается; проверка синтаксиса называет функцию, которую нужно использовать, кнопка переписывает формулу, а при вводе ln предлагается log. Каждый шаг поддерживает глубокое приближение методом возмущений с той же арифметикой расширенного диапазона после 1e28x, что и у встроенных фракталов, кроме шага со степенью, показатель которой зависит от z или c (например, z^c): такой шаг рендерится прямым путём и остаётся резким, пока один шаг одинарной точности по c не станет шире пикселя, — примерно до 1e4x–1e5x, а вдали от начала координат и того меньше. Диалог формулы сообщает, какой случай применяется. log, sqrt и нецелые степени берут главные значения, которые скачут при переходе через отрицательную действительную ось. Метод возмущений точно следует этому скачку, но пиксель, чья орбита пересекает разрез там, где орбита опорной точки его не пересекает, с этого момента считается примерно с одинарной точностью, поэтому на глубоких видах, орбиты которых проходят через разрез, там виден шум (измерено: чисто при 1e5x на действительной оси для sqrt(z^4 + c), шумно для формул с log уже при 1e4x). Каждый шаг итерируется полностью (без аппроксимации рядами), поэтому глубокий вид с большим числом итераций рендерится медленнее, чем у встроенного фрактала. У своей границы формула с функциями имеет хаотические орбиты, и длинная орбита там отслеживается с одинарной точностью лишь приблизительно: при 2000 итераций треть пикселей sin(z) + c отличается от точной орбиты, при 60 — почти ни один. Виды, сеансы и экспортированные изображения хранят текст формулы.
help-fractals-custom-sections = Разделы Fractint тоже работают. Операторы перед «:» составляют раздел инициализации, выполняемый один раз на пиксель; сравнение в роли последнего оператора — собственная проверка выхода формулы, и итерации идут, пока оно выполняется (|z| <= 100 && |z - 1| > 0.000001 останавливает орбиту, которая убегает или оседает на 1); if (...), elseif (...), else и endif выбирают операторы; а переменная, прочитанная в шаге до того, как ей присвоено значение, берёт значение из предыдущего шага (из раздела инициализации или 0). Сравнения сравнивают действительные части и дают 1 или 0, && и || объединяют их, а присваивание — тоже значение (a = b = pixel). pixel, p1-p5, pi и e можно присваивать, и они сохраняют собственное значение, пока этого не произошло; maxit — число итераций, а ismand равно 1 (форма Мандельброта у формулы). Формула с любым из этих элементов или с округлением рендерится прямым путём, без глубокого приближения. Библиотека формул > Импорт читает также файлы Fractint .frm. Каждая прочитанная запись становится формулой, и её строка сообщает, что изменило чтение: fn1-fn4 принимают значения Fractint по умолчанию (sin, sqr, sinh, cosh); переменная c из Fractint переименовывается в c_, так как c здесь — пиксель; имя, которому ничего не присвоено, равно 0, как во Fractint; пробелы внутри имени отбрасываются (end if — это endif), как их отбрасывает Fractint. Сообщение импорта подсчитывает записи, которые не читаются, и называет самую частую причину: переменные экрана и вида (whitesq, scrnpix, scrnmax, center, magxmag, rotskew), lastsqr и случайные числа не поддерживаются. Классика Fractint из «Коллекции» (Magnet, Nova, Spider, Manowar и другие) написана с этими разделами.
help-fractals-custom-editor = В поле формулы скобки окрашиваются своим цветом для каждого уровня вложенности; пара рядом с курсором подсвечивается, а скобка без пары показывается красным (скобки в комментариях не учитываются, так как анализатор их игнорирует). Имена дополняются по мере ввода: после двух букв список предлагает функции, имена и ваши собственные переменные, начинающиеся так же. Up и Down выбирают вариант, Tab или Enter принимает его (функция вставляется с открывающей скобкой), Esc закрывает список, а Enter после полностью введённого имени по-прежнему начинает новую строку.
help-fractals-custom-textbook = «Учебник» (переключатель над формулой) показывает формулу набранной так, как её набирают в учебнике, и позволяет править её прямо там. Вводите как в тексте: / делает дробь из предшествующего члена, ^ — показатель степени (+, - или =, введённые в конце показателя, выходят из него), ( — скобки или вызов после имени функции, | — вертикальные черты, а запятая внутри скобок — комплексную константу. Стрелки перемещают по формуле, Up и Down — между числителем и знаменателем, Tab — к следующему пустому полю; Backspace в начале дроби или пары скобок выделяет её, а повторное нажатие удаляет её, сохраняя содержимое. Копирование, вставка и отмена работают как в текстовом поле, имена дополняются так же, как там, а панель клавиш вводит в набранную формулу (□² возводит в квадрат то, что стоит перед кареткой, клавиша функции оборачивает выделение). Каждое изменение записывается обратно в текст, поэтому оба представления всегда содержат одну и ту же формулу; строка, которую вы не правили, сохраняет свои пробелы и комментарий. Запись никогда не меняет того, что вычисляется: |z| — квадрат модуля и показывается как |z|², cabs(z) показывается как |z|, а exp(z) остаётся exp(z). Строка, которая не читается, показывается красным; щёлкните по ней, чтобы править её как текст. LaTeX копирует формулу как исходный код LaTeX в том виде, как она набрана, а пока включён «Учебник», библиотека формул тоже показывает свои формулы набранными.
help-fractals-custom-numbers = Числа в формуле (0.1, 1/3, pi, параметры) читаются как значения двойной точности, примерно 16 цифр, и глубокая опорная орбита итерируется именно с этими значениями. Поэтому две формулы, равные на бумаге, в глубине могут различаться: 0.1*z умножает на ближайшее к 0.1 число двойной точности, а z/10 делит на точное 10, и две картинки расходятся примерно после 1e15x (измерено: 5% пикселей при 1e15x, треть при 1e17x, все при 1e20x). Каждая картинка верна для своей формулы; записывайте ту, которой собираетесь делиться.
help-fractals-custom-library = Кнопка диалога «Сохранить в библиотеку» хранит формулу и её параметры под именем (сохранение под именем, которое уже есть в библиотеке, обновляет эту запись). Фрактал ▸ Библиотека формул… перечисляет их, чтобы применить, изменить, экспортировать или удалить; «Импорт» добавляет формулы из файла формул, ничего не заменяя: формула, которая в библиотеке уже есть, пропускается, а новая, чьё имя занято, становится «имя (2)». Библиотека — это formulas.toml в папке, указанной в разделе «Файлы и данные» (О программе), и сама является файлом формул. Ключевой кадр тура хранит формулу как formula = "..." с formula_params = [[re, im], ...], и Инструменты > Тур от текущего вида записывает их для вида со своей формулой.
help-fractals-custom-collection = Вместе с Fractadyne поставляется коллекция из 45 формул, написанных специально для него: высшие степени и сложенные варианты семейств Burning Ship и Celtic, смеси и двухшаговые гибриды, множество Мандельброта, увиденное через другое отображение (обращённое, лог-полярное, через синус), трансцендентные и рациональные формулы, а также множества Жюлиа. Это «Примеры» диалога и вкладка «Коллекция» библиотеки формул, где «Копировать к себе» добавляет формулу к вашим. У каждой формулы есть начальный вид, и «Применить» переходит туда: формула, открытая на исходном виде, часто находится не там, где её картинка. Формула, которую вы сохраняете, пока она показывается, так же запоминает вид, на котором вы находитесь, и «Применить» возвращает к нему.

## Help ▸ How it works

help-how-escape = Время убегания и плавный цвет
help-how-escape-text = Каждый пиксель итерирует формулу, пока её модуль не превысит радиус выхода или не будет достигнут предел итераций. Одно лишь сырое число шагов даёт резкие полосы; добавление дробной части, полученной из конечного модуля, даёт непрерывный цвет без полос.
help-how-precision = Положение с произвольной точностью
help-how-precision-text = Обычным 64-битным числам не хватает цифр уже около увеличения 1e15×. Fractadyne хранит центр вида с произвольной точностью, и число цифр растёт по мере приближения, поэтому местоположение никогда не теряет точность. Масштаб пикселя тоже хранится с расширенной экспонентой, поэтому и он не застревает на пределе 64-битных чисел ~1e308× — глубина ограничена только терпением, а не каким-либо фиксированным диапазоном чисел.
help-how-perturbation = Метод возмущений
help-how-perturbation-text = Итерировать каждый пиксель с высокой точностью было бы слишком медленно. Вместо этого одна опорная точка итерируется с высокой точностью на CPU («опорная орбита»), а каждый другой пиксель вычисляется на GPU как крошечная разность δ от неё в быстрой низкой точности: δz → 2·Z·δz + δz² + δc.
help-how-floatexp = Экстремальная глубина (floatexp)
help-how-floatexp-text = Примерно после 1e28× даже эта крошечная разность уходит ниже 32-битного диапазона, поэтому она хранится как мантисса плюс отдельная целая экспонента («floatexp»), что снимает 32-битную стену глубины. Движок переключается автоматически: прямые вычисления на малой глубине, метод возмущений на большой и floatexp на самой большой. Тогда глубина ограничена точностью координат и бюджетом итераций, а не фиксированным пределом — это перепроверено по Fraktaler-3 на эталонном корпусе из 39 местоположений, самое глубокое из которых — 5.63e18003× (38 неэкстремальных строк служат обычной проверкой, до 6.13e1105×), а также внутренним набором тестов на согласованность точности до 1e1000000×; встроенный тур доходит до ~1.3e420×.
help-how-reference = Выбор опорной точки и перебазирование
help-how-reference-text = Опорная точка выбирается (с оценкой в высокой точности) так, чтобы её орбита как можно дольше оставалась в пределах вида. Когда разность становится слишком большой, она «перебазируется» обратно на опорную орбиту, чтобы сохранить точность.
help-how-series = Аппроксимация рядами
help-how-series-text = На глубоком приближении ранние итерации почти не сдвигают δ от опорной орбиты, поэтому их можно вычислить разом: δz аппроксимируется коротким многочленом от δc (порядка 3), коэффициенты которого итерируются вдоль опорной орбиты. Рендерер перескакивает к последней итерации, на которой этот многочлен ещё точен для всего вида, а затем итерирует обычным образом — экономя пропущенные шаги без какого-либо изменения изображения (переключается в Управление ▸ Дополнительно). Mandelbrot и Multibrot 3–8, самый глубокий диапазон; не работает с раскраской «Среднее полос» / «Неравенство треугольника» / «Ловушка орбиты» / «Декомпозиция» (им нужна каждая итерация). Там, где активна BLA, она уже включает в себя этот ранний пропуск.
help-how-distance = Оценка расстояния и освещение
help-how-distance-text = Отслеживание производной dz/dc даёт расстояние от каждого пикселя до границы. На этом основаны рельефное 3D-освещение, свечение по расстоянию и метод раскраски «Оценка расстояния» — всё это работает на любой глубине приближения.
help-how-safety = Сглаживание и безопасность
help-how-safety-text = Неподвижные изображения подвергаются суперсэмплингу (2–8× по каждой оси), когда вид устоится. Бюджет работы удерживает одну отрисовку GPU в пределах сторожевого лимита драйвера, снижая при экстремальных настройках разрешение (но никогда не число итераций), поэтому глубокие виды остаются детальными, а не становятся пустыми.

## Help ▸ Command line (the flags themselves stay English)

# The section's introduction. The flags listed below it are the command line itself, English in every language.
help-cli-intro = Fractadyne может работать без интерфейса — для рендеринга, экспорта видео и туров, автоматизации, проверок по эталонным изображениям и тестов производительности. Без флагов запускается интерактивный графический интерфейс.

## Help ▸ Shortcuts

# What each mouse gesture and key does (the gestures are the Navigation section's).
help-shortcuts-pan = Сдвиг вида
# A gesture: turning the mouse wheel.
help-key-wheel = Колесо
help-shortcuts-wheel = Приближение к курсору
help-shortcuts-box-zoom = Приближение рамкой (растяните прямоугольник, чтобы приблизиться к нему)
# $icon: the click-to-zoom toolbar button's symbol. "tool on": while that button is on.
help-key-click-tool = Щелчок (инструмент { $icon } вкл.)
help-shortcuts-click = Нырнуть в точку; правый щелчок — назад
# A heading: the keyboard shortcuts.
help-shortcuts-keyboard = Клавиатура
help-shortcuts-space = Непрерывное приближение / отдаление
help-shortcuts-undo = Отменить изменение вида
help-shortcuts-redo = Повторить изменение вида
help-shortcuts-minibrot = Найти миниброт и приблизиться к нему (метод Ньютона — Рафсона)
help-shortcuts-autopilot = Автопилот автоприближения (погружение к деталям; любое действие останавливает)
# off / strip / panel / full: the panel's sizes, as View ▸ Live render panel names them.
help-shortcuts-live-panel = Панель живого рендеринга: выкл. / полоса / панель / полная (наведите курсор на прибор, чтобы узнать, что он означает; × на панели закрывает её)
help-shortcuts-esc = Остановить автопилот / воспроизводимый тур или выйти из полноэкранного режима
help-shortcuts-snapshot = Снимок в последнюю папку экспорта — захват экрана с видом как на экране или полный рендеринг с настройками экспорта; при первом нажатии программа спросит, что именно, а изменить это можно в Файл → Настройки → Снимок.
help-shortcuts-help = Открыть эту справку
# A heading: the shortcuts of the Game of Life, with the menu path to it.
help-shortcuts-life = Life (Фрактал → Автоматы → Life)
help-shortcuts-life-run = Запустить / приостановить вселенную
help-shortcuts-life-step = Шаг на одно поколение / прыжок на несколько поколений (задаётся в панели Life)
help-shortcuts-life-draw = Включить / выключить рисование (карандаш на панели инструментов)
# A gesture. "Draw on": while the Life draw tool is on.
help-key-click-drag-draw = Щелчок / перетаскивание (рисование вкл.)
help-shortcuts-life-paint = Переключить клетку / рисовать: штрих, начатый на мёртвой клетке, рисует, а начатый на живой — стирает. Работающая вселенная приостанавливается, пока вы рисуете, и продолжает работу, когда вы отпускаете кнопку.
# $key: a keyboard shortcut (Ctrl+Z). "Draw on": while the Life draw tool is on.
help-key-when-drawing = { $key } (рисование вкл.)
help-shortcuts-life-undo = Отменить последнюю правку, заливку, очистку или смену правила

## Help ▸ Recommended hardware

help-hardware-intro = Fractadyne распределяет работу между GPU (попиксельные итерации + раскраска) и CPU (опорная орбита произвольной точности, на которой строится глубокое приближение). Важны оба, но для разного.
help-hardware-gpu = GPU (самое важное)
help-hardware-gpu-text = Каждый пиксель итерируется и раскрашивается на GPU, поэтому скорость GPU определяет частоту кадров и время экспорта. Дискретный GPU с высокой 32-битной производительностью (одинарной точности) даёт самое плавное глубокое приближение. Встроенные GPU тоже работают — просто медленнее, и они раньше упираются в бюджет работы на кадр (после чего рендерер урезает разрешение, а не детали). Требуется современный графический API: Vulkan, Direct3D 12, Metal или OpenGL через wgpu.
# The left column of a requirements table.
help-hardware-minimum = Минимум
help-hardware-gpu-minimum = Любой GPU с Vulkan / DX12 / Metal / GL 3.3.
help-hardware-recommended = Рекомендуется
help-hardware-gpu-recommended = Дискретный GPU (примерно последних 5 лет); >= 2 GB VRAM.
help-hardware-big-exports = Для больших экспортов
help-hardware-gpu-big-exports = >= 4 GB VRAM (рендеринг тайлами, поэтому при нехватке памяти работа замедляется плавно).
# A heading: the processor.
help-hardware-cpu = CPU
help-hardware-cpu-text = Опорная орбита высокой точности (и таблицы BLA) вычисляются на CPU и в основном в одном потоке, поэтому важнее всего высокая производительность на ядро — от неё зависит, как быстро прорисовывается глубокий вид после перемещения. Дополнительные ядра в основном помогают с фоновым кодированием при экспорте. Требуется 64-битный CPU.
help-hardware-cpu-recommended = Современный 64-битный CPU с хорошей одноядерной производительностью.
help-hardware-memory = Память
help-hardware-memory-text = Глубокие опорные орбиты растут с глубиной (много итераций с высокой точностью); огромные экспорты держат тайлы в оперативной памяти. 8 GB вполне достаточно для повседневного исследования; 16 GB и больше подходят для экстремальной глубины и очень больших экспортов.
help-hardware-checking = Проверка вашего оборудования
help-hardware-diagnostics = Справка → «Диагностика…» запускает на вашем компьютере те же тесты, что и у разработчиков. «Запустить самотест» сверяет математику и рендеринг с заведомо правильными результатами примерно за пятнадцать секунд; «Запустить тест интерфейса» проходит по интерфейсу и живому виду на нескольких глубинах приближения, делая снимки экрана; «Запустить проверку арифметики GPU» проверяет арифметику double-float в шейдере на каждом графическом бэкенде, который предлагает ваш компьютер, — за несколько секунд и без окна. Каждый тест запускается отдельным процессом, поэтому тест, нарушивший работу графического драйвера, падает сам, а ваш сеанс остаётся цел.
help-hardware-gpu-check = У проверки арифметики GPU нет результата «пройдено/не пройдено» — она записывает, как считают именно ваш GPU и драйвер, и на некотором оборудовании (на всех опробованных пока картах NVIDIA) сообщает, что компилятор шейдеров отбрасывает часть арифметики повышенной точности. Это настоящая находка, которую мы пытаемся проследить у разных производителей, а не неисправность вашего компьютера, поэтому, что бы она ни сообщила, приложить её к отчёту действительно полезно.
help-hardware-report = Если что-то выглядит неправильно, запустите самотест и воспользуйтесь кнопкой «Приложить к сообщению о проблеме…»: тогда отчёт будет содержать проверенный машиной результат с вашего собственного оборудования, что гораздо полезнее одного лишь описания. Одна оговорка, которую стоит знать: эталонные изображения получены на одной конкретной видеокарте, а разные GPU вполне законно вычисляют немного разные пиксели, поэтому на другом оборудовании изображения сравниваются с более широким допуском, и небольшие различия там ожидаемы, а не являются неисправностью.
help-hardware-notes = Примечания
help-hardware-note-watchdog = Мощный GPU не даёт сторожевому бюджету работы включиться при экстремальных настройках.
help-hardware-note-binary = Сборка — один самодостаточный исполняемый файл, установка среды выполнения не нужна.
help-hardware-note-benchmark = Запустите встроенный тест производительности (Инструменты ▸ Тест производительности), чтобы оценить свой компьютер.

## Help ▸ Acknowledgments & citations
## Names of people, programs, papers, books and websites stay as written, and so do the
## titles in quotation marks (the works' own titles); the prose around them translates.

help-ack-title = Благодарности и ссылки
# The dedication, in italics at the top of the section.
help-ack-dedication = Посвящается Stone Soup Group — добровольцам, создавшим Fractint (с 1988 года).
help-ack-dedication-text = Их свободно распространяемый код и дух открытого совместного исследования фракталов — причина, по которой столь многие из нас полюбили это искусство. Fractadyne создан в этой традиции.
help-ack-intro = Fractadyne опирается на десятилетия работы фрактального сообщества. Методы и предшествующие работы, на которых он основан, сверены с источниками ниже:
help-ack-deep-zoom = Алгоритмы глубокого приближения
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-martin-title = Метод возмущений и аппроксимация рядами — { $who }
help-ack-martin = Метод опорной орбиты и разности низкой точности (δz → 2Z·δz + δz² + δc) и ряд порядка n, пропускающий ранние итерации, впервые представленные в SuperFractalThing и его заметке «SuperFractalThing Maths» (sft_maths.pdf, 2013).
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bla-title = Двумерная линейная аппроксимация (BLA) и перебазирование — { $who }
help-ack-bla = Пропуск серий итераций с помощью объединённых линейных отображений и перебазирование на единственную опорную орбиту во избежание глитчей; разработано на fractalforums.org (2021).
# A link's label. $who: the linked page's author.
help-ack-bla-link = Объяснение BLA — { $who }
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-glitch-title = Критерий обнаружения глитчей — { $who }
help-ack-glitch = Проверка |Z + z| << |Z|, определяющая, когда пикселю нужна другая опорная точка, — основа метода возмущений без глитчей (fractalforums).
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-diffabs-title = Метод возмущений для неаналитических семейств (Burning Ship) — { $who }
help-ack-diffabs = Разбор случаев для модуля («diffabs»), благодаря которому метод возмущений работает для семейств с модулем (Burning Ship / Celtic / Buffalo), на fractalforums.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mathr-title = Эталонные реализации и перекрёстные проверки — { $who }
help-ack-mathr = Fraktaler-3 и Kalles Fraktaler 2+ (изначально созданный Karl Runmo) — их описания направляли разработку этого движка, а с их выводом EXR Fractadyne сверяет свой рендерер.
# A heading: credits for the coloring methods.
help-ack-coloring = Раскраска
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-harkonen-title = Плавное число итераций и среднее полос (Stripe Average) — { $who }
help-ack-harkonen = Непрерывная (нормализованная) раскраска по времени убегания и метод среднего полос из работы «On Smooth Fractal Coloring Techniques» (магистерская диссертация, 2007), где также анализируются раскраски по неравенству треугольника и по среднему ветвления.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mitchell-title = Среднее по неравенству треугольника — { $who }
help-ack-mitchell = Усреднение того, куда попадает каждое z² + c в пределах границ неравенства треугольника, для плавной пламеподобной внешней текстуры.
# A link's label: the person's tutorial pages. $who: the person's name.
help-ack-mitchell-link = { $who } — учебные материалы
help-ack-distance = Раскраска и освещение по оценке расстояния используют классический метод расстояния до границы через dz/dc (см. Peitgen & Saupe, «The Science of Fractal Images», 1988).
help-ack-lsystems-life = L-системы и игра «Жизнь»
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-lindenmayer-title = L-системы — { $who }
help-ack-lindenmayer = Параллельное переписывание строк, предложенное в 1968 году для моделирования роста растений и водорослей («Mathematical models for cellular interactions in development», Journal of Theoretical Biology).
help-ack-abop = Книга (1990), черепашьей интерпретации и нотации которой для стохастических, контекстно-зависимых и параметрических систем следует программа при чтении L-систем; встроенные системы с пометкой ABOP взяты из её иллюстраций.
# A link's label: the book can be read free on that website. $site: the website's name.
help-ack-abop-link = Книга, бесплатно на { $site }
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bourke-title = Коллекция L-систем — { $who }
help-ack-bourke = Многие встроенные системы взяты с его страниц об L-системах, среди них правила Chris Wallace для кривой Серпинского и куст Dietmar Saupe. Файлы .l Fractint, которые открываются напрямую, записаны в формате Stone Soup Group.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-conway-title = Игра «Жизнь» — { $who }
help-ack-conway = Клеточный автомат B3/S23 (1970), о котором мир узнал из колонки Martin Gardner «Mathematical Games» в Scientific American (октябрь 1970 года).
# A credit's heading: the website, then who runs it. $site: the website's name.
help-ack-lifewiki-title = { $site } — сообщество ConwayLife.com
help-ack-lifewiki = Справочник по нотациям правил, которые читает программа (B/S, von Neumann, Generations, буквы Alan Hensel для нетоталистических правил и MAP), по формату узоров RLE, общему с Golly, и по встроенным правилам и узорам, первооткрывателей которых он указывает.
help-ack-foundations = Основы и инструменты
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mandelbrot-title = Множество Мандельброта — { $who }
help-ack-mandelbrot = Объект, лежащий в основе всего.
help-ack-built-with = Создано на Rust, egui/eframe (emilk) и wgpu (gfx-rs); произвольная точность — astro-float, числа двойного слова — twofloat; PNG/OpenEXR, rfd, serde. Спасибо их авторам и сопровождающим.
help-ack-errors = Все ошибки в указании авторства — на совести автора; исправления приветствуются в трекере задач проекта.

## Help ▸ Third-party licenses

help-licenses-title = Лицензии сторонних компонентов
# $licenses: Fractadyne's license expression (MIT OR Apache-2.0), as written. $url: the web address of a crate's source. THIRD-PARTY-NOTICES.md is a file name, `option-ext` a crate's name, MPL-2.0 a license's.
help-licenses-intro = Fractadyne распространяется как статически скомпонованный исполняемый файл, включающий перечисленные ниже крейты с открытым исходным кодом и встроенные шрифты. Далее приведены полные тексты лицензий (они также есть в файле THIRD-PARTY-NOTICES.md, входящем в загрузку). Собственный код Fractadyne распространяется по двойной лицензии { $licenses }. `option-ext` распространяется по MPL-2.0 — его неизменённый исходный код находится по адресу { $url }.
# A button, after a copy icon: copies every notice to the clipboard.
help-licenses-copy-all = Копировать все уведомления

## Help ▸ About

help-about-tagline = Нативная программа для исследования фракталов, написанная на Rust с wgpu.
# The arithmetic deep zoom has used. $ran: the libraries that have run so far, as names (or a note that none has yet); $built: those this build contains, with versions.
help-about-arithmetic = Арифметика глубокого приближения: { $ran } (в этой сборке есть: { $built }).
help-about-files = Файлы и данные
# Followed by the folder's path.
help-about-files-text = Ваш сеанс (текущий вид, раскраска и предпочтения), закладки и их миниатюры, сохранённые градиенты и сохранённые формулы хранятся в этой папке пользователя и загружаются при следующем запуске:
# Tooltip of the copy button beside the folder's path.
help-about-copy-path = Копировать путь
# $command: a command typed in a terminal (it stays English).
help-about-session = Файл сеанса имеет версию; если он записан более новой версией Fractadyne, чем та, что может полностью его прочитать, программа загружает его, насколько это возможно, и предупреждает вас. Чтобы начать полностью с чистого листа, используйте Файл ▸ Сбросить состояние программы… (сначала спросит) или выполните `{ $command }` в терминале.
help-about-license = Лицензия
# $licenses: the license expression (MIT OR Apache-2.0), which stays as written.
help-about-license-text = { $licenses } — используйте на условиях любой из лицензий по своему выбору.
# A link to the source code, followed by ↗.
help-about-source = Исходный код на GitHub

## The live render panel ("blinkenlights": View ▸ Live render panel, key B)
## A front panel drawn over the view like a piece of 1970s test equipment. Its legends are engraved
## in capitals: write them in capitals here (the panel shows them exactly as written, it does not
## uppercase them). Numbers, units and readouts on its tubes and dials are not translated.

# Tooltip of the × button in the panel's top-right corner. The code adds the key after it: (B).
live-close-hover = Закрыть панель живого рендеринга
# The legend engraved along the top plate, left of the maker's badge.
live-plate = ЖИВОЙ РЕНДЕРИНГ · ЛИЦЕВАЯ ПАНЕЛЬ

## Live render panel: instrument titles
## Engraved above each instrument, and repeated as the title of its help card.

# Above the depth readout, whose tubes show the magnification as a power of two ("2^" stays).
live-title-depth = ГЛУБИНА 2^
live-title-iterations = ИТЕРАЦИИ
# Above the frame-time readout (milliseconds).
live-title-frame-ms = КАДР MS
# Above the date-and-time readout.
live-title-clock = ДАТА · ВРЕМЯ
# The walk dial: a radar screen showing a slow refresh rendered pass after pass, band after band.
live-title-walk = ОБХОД · ПОЛОСЫ
# A meter of how far the picture on screen lags the view.
live-title-age = ВОЗРАСТ КАРТИНКИ
# What part of the picture this frame's GPU pass covered.
live-title-tiles = ТАЙЛЫ
# A scope of the GPU's work per frame, in milliseconds.
live-title-scope = ПРОХОДЫ GPU · MS
# The depth dial: depth, the reference orbits being built, and the arithmetic in use.
live-title-dial = ГЛУБИНА · ОРБИТЫ · АРИФМЕТИКА
# Iteration steps per second.
live-title-steps = ШАГОВ / С
# The two GPUs (the window's and a second, worker GPU).
live-title-engines = ДВА ДВИГАТЕЛЯ
live-title-memory = ПАМЯТЬ
live-title-cores = ЯДРА · ПОТОКИ
live-title-pixels = ПОТОК ПИКСЕЛЕЙ
# Help card titles only: the three GPU timer lamps (T0, T1, T2 stay as they are) and the row of lamps.
live-title-timers = ТАЙМЕРЫ GPU T0–T2
live-title-bank = БЛОК ЛАМП

## Live render panel: the light bank's lamps
## Engraved under each lamp, and the title of its help card. A cell is narrow (about eight capital
## letters): a longer label breaks at a space onto two lines, and is drawn smaller if it still does
## not fit.

live-lamp-plan = ПЛАН
live-lamp-dispatch = ЗАПУСК
live-lamp-live = ЖИВОЙ
live-lamp-held = УДЕРЖАН
# The last picture reprojected (stretched) to fit the moving view.
live-lamp-reproj = РЕПРОЕКЦ
live-lamp-reading = ЗАМЕР
# A pass of a pinned walk.
live-lamp-pin-pass = ПРОХОД ЗАКРЕП
live-lamp-split = ДЕЛЕНИЕ
live-lamp-adopt = ПРИНЯТ
live-lamp-abandon = ОТКАЗ
# "REF": a reference orbit. Built for the view as it is (LIVE), ahead of it (AHEAD), or for a tour's next hold (HOLD).
live-lamp-ref-live = ОПОРА СЕЙЧАС
live-lamp-ref-ahead = ОПОРА ВПЕРЕДИ
live-lamp-ref-hold = ОПОРА ПАУЗА
# The series approximation.
live-lamp-series = РЯДЫ
# The second (worker) GPU.
live-lamp-worker = ПОМОЩНИК
live-lamp-tile = ТАЙЛ
# Supersampling (anti-aliasing); usually kept as SSAA.
live-lamp-ssaa = SSAA
live-lamp-palette = ПАЛИТРА
# The arithmetic in use (direct, df32, floatexp).
live-lamp-mode = РЕЖИМ

## Live render panel: the kinds of GPU pass
## In the GPU scope's legend (MOVING, PIN, WORKER, SETTLE, SSAA). SETTLE and SSAA also name a still
## view's walk on the walk dial.

# A moving frame's pass.
live-kind-moving = ДВИЖЕНИЕ
# A pass of a pinned walk.
live-kind-pin = ЗАКРЕП
# The worker GPU's passes.
live-kind-worker = ПОМОЩНИК
# A still view's settle tile or walk pass.
live-kind-settle = ДОВОДКА
# A supersampling sample.
live-kind-ssaa = SSAA

## Live render panel: legends and readouts on the instruments

# On the walk dial during a split refresh. $done: passes done; $passes: passes in all.
live-walk-split = ДЕЛЕНИЕ { $done }/{ $passes }
# On the picture-age meter's face, under its scale (0 to 2 octaves).
live-age-scale = ОКТАВ ОТСТАВАНИЯ
# The depth dial's band for direct (plain floating-point) arithmetic. The other bands, DF32 and FLOATEXP, are names and stay.
live-dial-direct = ПРЯМАЯ
# Under the two GPU gauges: GPU 0 draws the window; GPU 1 is the worker, or there is none.
live-engines-gpu-window = GPU 0 · ОКНО
live-engines-gpu-worker = GPU 1 · ПОМОЩНИК
live-engines-no-worker = НЕТ ПОМОЩНИКА
# The share of displayed frames each GPU made. $window and $worker: percentages, 0–100.
live-engines-on-screen = НА ЭКРАНЕ { $window } : { $worker }
# The same readout before any frame was shown.
live-engines-on-screen-none = НА ЭКРАНЕ —
# Under the CPU core bars. $count: the machine's cores.
live-cores-count = ЯДЕР: { $count }
# Under the threads scope. $count: the app's busy threads.
live-cores-threads = ПОТОКИ { $count }

## Live render panel: the memory tanks' legend
## What the GPU memory holds, by kind. Very narrow: about ten letters.

# G-buffers (the picture's working textures).
live-memory-gbuf = G-БУФЕР
# The held picture and the supersampling accumulator.
live-memory-hold = УДЕРЖ+SSAA
# The reference orbit and its BLA (bilinear approximation) table.
live-memory-orbit = ОРБИТА+BLA
# The worker GPU's copy of the reference.
live-memory-worker = КОПИЯ ОРБ.

## Live render panel: the roles on the threads scope
## Tiny labels on a small scope: at most four letters each.

# The reference builders.
live-cores-role-ref = ОРБ
# The scoring pool.
live-cores-role-pool = ПУЛ
# The thread driving the worker GPU.
live-cores-role-worker = ПОМ
# Probes: short side renders.
live-cores-role-probe = ЗОНД
# Image and tour exports.
live-cores-role-export = ЭКСП

## Live render panel: the tiles instrument's captions
## The line under the picture's plane, then the tile size (left) and the tiles a frame (right).

# A walk's pass. $lo and $hi: the iteration band it rendered (numbers).
live-tiles-pin-band = ПРОХОД ЗАКРЕП · ПОЛОСА { $lo }–{ $hi }
live-tiles-settle-band = ПРОХОД ДОВОДКИ · ПОЛОСА { $lo }–{ $hi }
live-tiles-moving-whole = КАДР В ДВИЖЕНИИ · ВСЯ КАРТИНКА
live-tiles-settled-whole = ГОТОВЫЙ КАДР · ВСЯ КАРТИНКА
# $pass: this pass; $passes: the passes the refresh is split into.
live-tiles-split-pass = ДЕЛЁННОЕ ОБНОВЛЕНИЕ · ПРОХОД { $pass } ИЗ { $passes }
# $grid: the grid's columns × rows, e.g. 12×8.
live-tiles-grid-done = СЕТКА ДОВОДКИ { $grid } · ГОТОВО
# $grid: the grid's columns × rows; $done and $total: tiles done, and in all.
live-tiles-grid-progress = СЕТКА ДОВОДКИ { $grid } · ГОТОВО { $done } ИЗ { $total }
# The same, while a tile too costly for one frame goes out in strips.
live-tiles-grid-strips = СЕТКА ДОВОДКИ { $grid } · ГОТОВО { $done } ИЗ { $total } · ПОЛОСАМИ
# $samples: supersampling samples folded so far; $lo and $hi: the iteration band the next one is walking.
live-tiles-ssaa-band = SSAA · ГОТОВО { $samples } · ПОЛОСА { $lo }–{ $hi }
# $sample: the supersampling sample.
live-tiles-ssaa-whole = ОТСЧЁТ SSAA { $sample } · ВЕСЬ КАДР
# No GPU pass this frame, and the panel has not seen a frame yet.
live-tiles-idle-starting = СТАРТ
# No GPU pass this frame: the last complete picture is held on screen.
live-tiles-idle-held = УДЕРЖАН
# No GPU pass this frame: the last picture is stretched to fit the view.
live-tiles-idle-reprojected = РЕПРОЕКЦИЯ
# No GPU pass this frame: the picture on screen is current.
live-tiles-idle-live = ЖИВОЙ
# No GPU pass this frame, and nothing to show.
live-tiles-idle-none = ПРОСТОЙ
# The tile size: $res, a whole picture's resolution (2400×1300).
live-tiles-tile-res = ТАЙЛ { $res }
# The tile size: $px, a tile's side in pixels.
live-tiles-tile-px = ТАЙЛ { $px } PX
live-tiles-no-pass = НЕТ ПРОХОДА
# Tiles a frame. $count: how many (0 when none).
live-tiles-rate-count = { $count }/КАДР
live-tiles-rate-one = 1 ТАЙЛ/КАДР
# $share: the part of one tile sent each frame, e.g. 0.33.
live-tiles-rate-part = { $share } ТАЙЛА/КАДР
live-tiles-rate-sample = 1 ТАЙЛ/ОТСЧЁТ
# In the plane's corner during a walk: the share of pixels still iterating. $percent: a percentage, e.g. 54 or 1.5.
live-tiles-running = В РАБОТЕ { $percent }%

## Live render panel: help cards
## Shown when the pointer rests on an instrument or a lamp. Words in capitals (SETTLE, SSAA, ABANDON,
## HELD, REPROJ, ADOPT, REF AHEAD, ON SCREEN) are the panel's own legends: use the same words as
## their messages above.

# The heading of a card's list of what healthy looks like.
live-card-healthy = НОРМА
# The heading of a card's list of what signals trouble.
live-card-trouble = ПРОБЛЕМА

live-card-walk-what = Глубокое обновление, слишком дорогое для одного кадра, закрепляется и проходится обходом: рендерится проход за проходом, и каждый позволяет пикселям итерировать на октаву дальше (256, 512, 1024 … вплоть до запрошенного). Обод — это число итераций в логарифмической шкале, начиная сверху; луч — ход обхода; каждая дуга — один проход, окрашенный по своему времени GPU относительно цели 20 ms. Белая отметка ждёт замера последнего прохода; вспышка — новая картинка, выходящая на экран.
live-card-walk-note = Неподвижный вид, слишком дорогой для одного прохода, обходится так же и помечается ДОВОДКА, а каждый отсчёт суперсэмплинга снова проходит обход и помечается SSAA: их проходы могут занимать до 400 ms каждый, и дуги окрашиваются относительно этого. Делённое обновление вместо этого рисует по одному кольцу на проход: каждый проход — все полосы, но лишь доля тайлов.
live-card-walk-good-sweep = Развёртка, которая проходит круг за несколько кадров и заканчивается вспышкой.
live-card-walk-good-arcs = Дуги в основном зелёные и янтарные.
live-card-walk-good-still = На глубоком неподвижном виде: более медленная развёртка на каждый отсчёт, дуги от зелёных до янтарных.
live-card-walk-bad-red = Красные дуги: проходы сверх бюджета, ощущаются как подтормаживание.
live-card-walk-bad-restart = Развёртка останавливается на полпути и начинается снова: обход был брошен, и картинка на экране остаётся старой.
live-card-walk-bad-hollow = Полые дуги: проходы, замеры которых так и не вернулись, поэтому следующий проход подбирается вслепую.

live-card-age-what = Насколько картинка на экране отстаёт от вида, в октавах приближения (одна октава — это 2×). Пока рендерится обновление, старая картинка показывается растянутой и стареет; когда обновление приходит, возраст падает. Полоса внизу — последние несколько секунд.
live-card-age-good-sawtooth = Пила примерно между 0,3 и 0,8, стрелка качается около отметки 0,5.
live-card-age-good-still = Близко к 0, когда вы останавливаетесь.
live-card-age-bad-red = Стрелка всё время за 1,0, в красной зоне: картинка устарела более чем в 2 раза и при приближении выглядит размытой или блочной.
live-card-age-bad-ramp = Подъём, который никогда не спадает: обновления не завершаются (следите за ОТКАЗ).

live-card-tiles-what = Как проход GPU этого кадра покрывает картинку. Кадры в движении и проходы обхода покрывают её целиком, а подпись называет полосу итераций прохода обхода. Проход обхода итерирует только ещё не убежавшие пиксели: плоскость заполняется до их доли (логарифмическая шкала, 0,01% внизу) и показывает её в углу, так что по мере углубления обхода она пустеет. Обновление, чуть слишком дорогое для одного кадра, делится максимум на 8 проходов, каждый из которых — чередующийся набор тайлов по 16 px. Готовый вид, слишком дорогой для одного прохода, становится сеткой тайлов — по одному за кадр, от центра наружу, — а тайл, который всё ещё слишком дорог, отправляется полосами. Внизу: размер тайла и число тайлов за кадр.
live-card-tiles-good-whole = Целые проходы, пока вы приближаетесь; делённые обновления из нескольких проходов.
live-card-tiles-good-grid = Когда вы останавливаетесь: сетка из десятков тайлов, заполненная за секунду-две, затем SSAA.
live-card-tiles-bad-small = Сетка из сотен маленьких тайлов: бюджет каждого прохода мал, и повышение резкости займёт много времени.
live-card-tiles-bad-strips = Много тайлов уходит полосами: тайлы работали медленнее, чем было рассчитано при построении сетки.
live-card-tiles-bad-splits = Деление застряло на 8 проходах: обновление едва помещается, а чуть глубже оно перейдёт на более медленный обход.
live-card-tiles-bad-dark = Долгие тёмные промежутки во время движения: проходов нет вообще.

live-card-scope-what = Один столбец на кадр, самый новый справа: выполненная им работа GPU, сложенная по видам (кадр в движении — зелёный, проход закреплённого обхода — от зелёного до красного по стоимости, тайл доводки или проход обхода неподвижного вида — кремовый, отсчёт суперсэмплинга — лавандовый, кадры и отсчёты вспомогательного GPU — синие), в миллисекундах. Пунктирные линии на 16,7 ms (60 fps), 33 и 50. Полая метка — проход, замер которого так и не вернулся.
live-card-scope-good-under = Столбцы ниже линии 16,7, пока вы двигаетесь.
live-card-scope-good-pin = Проходы закреплённого обхода от зелёного до янтарного.
live-card-scope-good-samples = Отсчёты суперсэмплинга неподвижного вида могут идти дольше: ничего не движется.
live-card-scope-bad-over = Столбцы выше 33 или 50 ms: заметное подтормаживание.
live-card-scope-bad-seconds = Проходы по нескольку секунд: сторожевой таймер GPU может сбросить устройство.
live-card-scope-bad-hollow = Много полых меток: работа подбирается без замеров.

live-card-timers-what = Три слота меток времени GPU. Слот загорается, когда проход замеряется, и гаснет, когда его замер возвращается, через два-три кадра. T0 замеряет проход каждого кадра; T1 и T2 — дополнительные проходы закреплённого обхода.
live-card-timers-good = T0 мигает, когда есть работа; T1 и T2 мигают во время обходов.
live-card-timers-bad-stuck = Лампа горит не переставая: её замер не возвращается, и оценки стоимости устаревают.
live-card-timers-bad-dark = Все тёмные, когда GPU занят: ничего не измеряется.

live-card-dial-what = Красная стрелка — глубина вида: увеличение в виде степени двойки. Полосы — арифметика, которую там использует GPU: прямая, df32 (два float на число) и floatexp (float с широкой экспонентой) после 2^93. Точки — строящиеся опорные орбиты: зелёные — для вида, янтарные — впереди него, куда направляется приближение, синие — для следующей паузы тура. Кольцо — установка; красный крестик — построение, брошенное, потому что вид его обогнал.
live-card-dial-good-ahead = Янтарные точки появляются впереди стрелки и становятся кольцами до того, как она до них дойдёт.
live-card-dial-good-crosses = Мало красных крестиков.
live-card-dial-bad-passing = Стрелка проходит свою последнюю установленную опорную орбиту: кадры ждут новую, и это ощущается как зависание.
live-card-dial-bad-crosses = Повторяющиеся красные крестики: приближение обгоняет построители, и CPU тратится впустую.
live-card-dial-bad-crossing = Переход через 2^93 — известное медленное место: первые кадры floatexp стоят дороже.

live-card-bank-what = Каждая лампа загорается, когда происходит её событие, и гаснет примерно за 150 ms, поэтому занятый рендерер мерцает, а простаивающий темнеет. Слева направо: кадр, обновление, опорные орбиты, затем вспомогательный GPU, доводка, палитра и арифметика.
live-card-bank-note = Наведите курсор на лампу, чтобы узнать, что она означает.

live-card-depth-what = Увеличение в виде степени двойки: 2^100 — это около 1,3 × 10^30.
live-card-depth-note = После 2^93 каждый шаг стоит дороже (floatexp), а опорные орбиты строятся дольше.

live-card-iter-what = Запрос итераций: сколько может итерировать каждый пиксель, прежде чем он будет считаться лежащим внутри множества. Он растёт с глубиной.
live-card-iter-good = Равномерно растёт, пока вы погружаетесь.
live-card-iter-bad-low = Застрял на низком значении на глубине: спирали и миниброты отрисовываются как плоская внутренняя область.
live-card-iter-bad-high = Очень высокий: каждое обновление стоит дороже, а обходы занимают больше времени.

live-card-ms-what = Время между выведенными на экран кадрами.
live-card-ms-good = Интервал вашего дисплея: 16,7 на дисплее 60 Hz.
live-card-ms-bad-stutter = Выше 33: заметное подтормаживание.
live-card-ms-bad-spikes = Пики выше 50: рывки, обычно из-за одного дорогого прохода (см. «Проходы GPU»).

live-card-clock-what = Часы реального времени по местному времени с точностью до десятой доли секунды.
live-card-clock-note = Они отмечают на любом снимке экрана с панелью, когда он был сделан, так что картинку с неполадкой можно сопоставить с журналом программы.

live-card-tach-what = Шаги итерации в секунду. Белая стрелка: запрошенные шаги (пиксели × итерации). Янтарная: шаги, фактически выполненные GPU. Разрыв между ними — это BLA (билинейная аппроксимация), перепрыгивающая сразу много итераций; ×N — этот множитель. Счётчик суммирует шаги за сеанс.
live-card-tach-good = Большой разрыв на глубине: BLA делает своё дело, от ×10 до ×1000.
live-card-tach-bad-together = Стрелки вместе на глубине: BLA здесь не может пропускать (часто рядом с внутренностью миниброта), поэтому обновления медленные.
live-card-tach-bad-red = Белая стрелка в красной зоне: запрашивается больше, чем GPU может сделать за кадр.

live-card-engines-what = Насколько занят каждый GPU; точки несут его кадры на экран: время GPU его проходов за последнюю секунду, включая кадры движения и отсчёты суперсэмплинга вспомогательного GPU. НА ЭКРАНЕ — доля выведенных кадров, сделанных каждым из них. При одном GPU сторона вспомогательного остаётся без дела.
live-card-engines-good = Оба заняты во время погружения, вспомогательный поставляет часть кадров.
live-card-engines-bad-pinned = GPU 0, рисующий окно, загружен почти на 100%: он — узкое место, и кадры подтормаживают.
live-card-engines-bad-idle = Вспомогательный GPU включён, но простаивает: он не поставляет кадры.

live-card-memory-what = RAM: рабочий набор программы, красная линия — его пик. VRAM: то, что программа держит на GPU, подсчитанное по размерам выделений, по видам (G-буферы, текстуры удержания и SSAA, буферы орбиты и BLA, копия опорной орбиты у вспомогательного GPU), относительно объёма карты.
live-card-memory-good = Ровно при приближении; ступенчато растёт и снижается, когда опорные орбиты появляются и уходят.
live-card-memory-bad-leak = RAM растёт и никогда не снижается: утечка.
live-card-memory-bad-vram = VRAM близко к объёму карты: выделения могут не удаваться, или устройство может быть потеряно.

live-card-cores-what = Загрузка CPU по ядрам, от зелёного к янтарному и красному, и осциллограф, где каждый занятый поток вращается по орбите своей роли: построители опорных орбит, пул оценки, вспомогательный GPU, побочные рендеры.
live-card-cores-good = Одно-два ядра заняты во время приближения; короткий всплеск на всех ядрах, когда выбирается опорная точка.
live-card-cores-bad-every = Все ядра заняты по нескольку секунд: построение опорной орбиты на экстремальной глубине.
live-card-cores-bad-own = Собственное ядро программы в красной зоне: подтормаживает само окно.

live-card-pixels-what = Пикселей, вычисленных в секунду, в Mpx/s. Тусклые: ещё итерируются в обходе. Яркие: готовы.
live-card-pixels-good = Десятки Mpx/s во время движения.
live-card-pixels-bad = Около нуля, пока картинка устарела: GPU чего-то ждёт — опорную орбиту или замер.

## Live render panel: the lamps' help cards

live-card-lamp-plan-what = Кадр спланирован: выбраны разрешение, запрос итераций и бюджет, а также решено, обновлять ли картинку.
live-card-lamp-plan-good = Мерцает при любом изменении вида или настроек.
live-card-lamp-plan-bad = Темно, пока вы приближаетесь: поток интерфейса завис.

live-card-lamp-dispatch-what = В этом кадре работа ушла на GPU.
live-card-lamp-dispatch-good = Горит, пока что-то меняется; темно в простое.
live-card-lamp-dispatch-bad = Занят, хотя ничего не меняется: программа не успокаивается.

live-card-lamp-live-what = Картинка на экране — собственный рендер этого кадра.
live-card-lamp-live-note = Во время глубокого погружения горит редко, и это нормально: см. УДЕРЖАН и РЕПРОЕКЦ.
live-card-lamp-live-good = Горит на малых глубинах, где кадр дёшев.

# "Prefer detail" is a setting's name.
live-card-lamp-held-what = Последняя полная картинка остаётся на экране, пока под ней собирается следующая: обновление глубокого приближения или, при включённом «Предпочитать детали при приближении», следующий тайл или отсчёт суперсэмплинга неподвижного вида.
live-card-lamp-held-good-dive = Горит отрезками во время глубокого приближения, прерываясь на ПРИНЯТ.
live-card-lamp-held-good-still = Горит ровно, пока неподвижный вид становится резче, а SSAA мигает.
live-card-lamp-held-bad = Горит по нескольку секунд при приближении без ПРИНЯТ: обновления не завершаются.

live-card-lamp-reproj-what = Последняя картинка, растянутая по движущемуся виду, пока не будет готова новая.
live-card-lamp-reproj-good = Короткие вспышки, пока вы двигаетесь.
live-card-lamp-reproj-bad = Горит долго: чем сильнее растянута картинка, тем она мягче.

live-card-lamp-reading-what = С GPU вернулся замер: время прохода, диапазон убегания или содержимое.
live-card-lamp-reading-good = Ровное мерцание во время рендеринга.
live-card-lamp-reading-bad = Долго темно во время рендеринга: замеры не возвращаются, и бюджеты рассчитываются наугад.

live-card-lamp-pin-pass-what = Проход закреплённого обхода: картинка отрендерена на одну полосу итераций дальше.
live-card-lamp-pin-pass-good = Серии проходов, заканчивающиеся на ПРИНЯТ.
live-card-lamp-pin-pass-bad = Серии, заканчивающиеся на ОТКАЗ.

live-card-lamp-split-what = Проход делённого обновления: доля тайлов, отрендеренная с полным числом итераций.
live-card-lamp-split-good = Короткие серии, заканчивающиеся на ПРИНЯТ.
live-card-lamp-split-bad = Серии по 8 — максимум допустимого: обновление едва помещается.

live-card-lamp-adopt-what = Завершённое обновление заменило картинку на экране.
live-card-lamp-adopt-good = Несколько раз в секунду во время приближения.
# "Picture age" names the instrument (its title, PICTURE AGE).
live-card-lamp-adopt-bad = Редко во время приближения: картинка стареет (см. «Возраст картинки»).

live-card-lamp-abandon-what = Выполняемое обновление было отброшено: вид ушёл слишком далеко, обновление слишком устарело или его заменило более новое. Построение опорной орбиты, которое обогнал вид, зажигает лампу тускло.
live-card-lamp-abandon-good = Редко или один раз сразу после перехода.
live-card-lamp-abandon-bad = Часто: работа тратится впустую, и картинка остаётся старой.

live-card-lamp-ref-live-what = Строится опорная орбита для вида в его нынешнем состоянии: у него её нет, он покинул прежнюю или ему нужна более длинная.
live-card-lamp-ref-live-good = Короткие вспышки.
live-card-lamp-ref-live-bad-long = Долгие отрезки: глубокие виды её ждут.
live-card-lamp-ref-live-bad-lookahead = Часто горит во время погружения, пока ОПОРА ВПЕРЕДИ темна: упреждение не успевает.

live-card-lamp-ref-ahead-what = Строится упреждающая опорная орбита глубже вида, там, куда направляется приближение.
live-card-lamp-ref-ahead-good = Регулярные вспышки во время погружения.
live-card-lamp-ref-ahead-bad = Темно во время глубокого погружения: вид обгонит свои опорные орбиты.

live-card-lamp-ref-hold-what = Предвыборка для паузы тура: опорная орбита для следующего ключевого кадра с паузой, построенная во время плавного движения к нему, чтобы воспроизведение никогда не ждало.
live-card-lamp-ref-hold-note = Вне воспроизведения тура темно.
live-card-lamp-ref-hold-good = Вспышка перед каждой паузой в туре.
live-card-lamp-ref-hold-bad = Горит всю паузу: предвыборка запаздывает, и тур её ждёт.

live-card-lamp-series-what = Аппроксимация рядами для новой опорной орбиты, позволяющая каждому пикселю пропустить первые итерации.
live-card-lamp-series-good = Кратко.
live-card-lamp-series-bad = Долго на глубине: задерживает установку опорной орбиты.

live-card-lamp-worker-what = Второй GPU поставил кадр.
live-card-lamp-worker-good = Ровные вспышки, когда вспомогательный GPU включён.
live-card-lamp-worker-bad = Темно при включённом вспомогательном GPU: он не поставляет кадры.

live-card-lamp-tile-what = Тайл доводки или его полоса ушли на GPU.
live-card-lamp-tile-good = Всплеск после того, как вы остановились, затем тишина.
live-card-lamp-tile-bad = Никогда не прекращается: сетка всё время перезапускается, потому что что-то постоянно меняет вид или настройки.

live-card-lamp-ssaa-what = К готовой картинке добавлен отсчёт суперсэмплинга.
live-card-lamp-ssaa-good = Серия вспышек после доводки, затем темно.
live-card-lamp-ssaa-bad = Перезапускается снова и снова: доводке постоянно что-то мешает.

live-card-lamp-palette-what = Нормализация цвета приняла новый диапазон убегания, так что цвета следуют за приближением.
live-card-lamp-palette-good = Мерцает во время приближения.
live-card-lamp-palette-bad = Темно во время приближения: диапазон палитры заморожен, и цвета становятся плоскими и малоконтрастными.

live-card-lamp-mode-what = Сменилась арифметика: прямая, df32 или floatexp.
live-card-lamp-mode-good = Одна вспышка на границе, например 2^93.
live-card-lamp-mode-bad = Мигает туда-обратно: вид мечется через границу.

## The Render client window (File ▸ Render client…): lending this machine's GPU to a render farm

farm-client-title = Клиент рендеринга
farm-client-intro = Рендерить кадры для рендер-фермы. Контроллер — Инструменты ▸ Рендеринг на ферме… на другом компьютере — показывает свой адрес и ключ фермы. Этот компьютер не открывает портов: он сам подключается к контроллеру.
# A field label, and the label of the status row that names the controller.
farm-client-controller = Контроллер
farm-client-find = Найти
farm-client-find-hover = Искать контроллеры в этой сети (контроллер отвечает, пока его ферма ждёт подключений)
farm-client-key = Ключ фермы
# A checkbox beside the masked farm key: show it in clear.
farm-client-show-key = Показать
# A field label (the name this machine goes by), and the label of the status row about this machine.
farm-client-machine = Этот компьютер
farm-client-machine-hover = Имя, под которым контроллер показывает этот компьютер.
farm-client-share = Общий диск
# Greyed hint inside the empty field; the path is an example.
farm-client-share-hint = необязательно — напр. \\fileserver\share
farm-client-share-hover = Путь этого компьютера к общему диску, который использует контроллер (режим общего диска): кадры записываются туда, а не отправляются по соединению. Оставьте пустым, чтобы отправлять каждый кадр.
farm-client-gpu = Видеокарта
# The label of the row with the idle option.
farm-client-when = Когда
# A checkbox followed by a number of minutes ("Only when idle for [5] min").
farm-client-when-idle = Только при простое дольше
farm-client-when-idle-hover = Брать работу, только когда никто не трогал клавиатуру или мышь этого компьютера столько времени или пока его экран заблокирован. Когда кто-то начинает им пользоваться, текущий кадр сразу останавливается и возвращается на ферму.
# A collapsible section: the largest job this machine accepts.
farm-client-limits = Ограничения
farm-client-limits-note = Задание, требующее больше этого, отклоняется, а не урезается.
farm-client-limit-size = Наибольший кадр
farm-client-limit-ss = Наибольший суперсэмплинг
farm-client-limit-iter = Наибольшее число итераций

## Render client: the graphics-card choice

farm-client-gpu-system = Выбор системы
farm-client-gpu-every = Все видеокарты
# $count: this machine's graphics cards.
farm-client-gpu-every-n = Все видеокарты ({ $count })
# $count: this machine's graphics cards.
farm-client-gpu-every-each = Все видеокарты ({ $count }) — по сеансу на каждую
# A card the window cannot name. $number: its number (or numbers) as --list-adapters gives it.
farm-client-gpu-adapter = Адаптер { $number }
farm-client-gpu-finding = Поиск видеокарт этого компьютера…
# "NAME · GPU 1" is how such a machine is named in the controller's list: keep it as it is.
farm-client-gpu-hover = Какая видеокарта рендерит кадры фермы. При выборе всех карт каждая подключается к ферме как отдельный компьютер — «NAME · GPU 1», «NAME · GPU 2», — а «Пауза» и «Отключиться» действуют сразу на все.

## Render client: looking for controllers on the network (the Find button)

farm-client-finding = Поиск контроллеров в этой сети…
# $port: the UDP port number.
farm-client-found-none = Ни один контроллер не ответил. Контроллер отвечает, пока его ферма ждёт подключений; его брандмауэр должен пропускать входящий UDP { $port }.
farm-client-found-pick = В этой сети — щёлкните по контроллеру, чтобы использовать его:
# $identity: the controller's identity fingerprint.
farm-client-found-hover = Идентификатор { $identity } — использовать этот контроллер (ключ его фермы всё равно нужен)
# After a warning sign. $version and $git: this app's version and build.
farm-client-found-other-build = Сборка, отличная от этой ({ $version } { $git }): она отклонит этот компьютер.

## Render client: what is wrong with what was typed, or with starting the client

farm-client-bad-address = Введите адрес контроллера в виде HOST:PORT, как его показывает окно контроллера (например, 192.168.1.20:46733).
# $why: what is wrong with the key (in English).
farm-client-bad-key = Ключ фермы: { $why }
# $why: what is wrong with the name (in English).
farm-client-bad-name = Имя: { $why }
farm-client-no-config = Нет папки конфигурации, в которой можно хранить ключ фермы.
# $why: the system's reason.
farm-client-start-failed = Не удалось запустить клиент рендеринга: { $why }

## Render client: the state, in the status heading

farm-client-phase-connecting = Подключение…
farm-client-phase-checking = Выполняется самопроверка…
farm-client-phase-idle = Подключено — ожидание работы
farm-client-phase-rendering = Рендеринг
farm-client-phase-in-use = Ожидание — кто-то пользуется этим компьютером
farm-client-phase-paused = Приостановлено
farm-client-phase-retrying = Контроллер недоступен — повторная попытка
# $seconds: the wait before the next try.
farm-client-phase-retrying-in = Контроллер недоступен — повтор через { $seconds } s
farm-client-phase-disconnected = Отключено
farm-client-phase-ended = Завершено

## Render client: the line under the status heading

# $address: the controller's address (HOST:PORT).
farm-client-detail-connecting = Подключение к { $address }…
# $controller: the controller's name.
farm-client-detail-checking = Подключено к «{ $controller }» — выполняется самопроверка
# $controller: the controller's name.
farm-client-detail-approval = Подключено к «{ $controller }» — ожидание, пока его пользователь одобрит этот компьютер
# $controller: the controller's name.
farm-client-detail-idle = Подключено к «{ $controller }» — простой
# $why: the check that failed and why (in English).
farm-client-detail-check-failed = Самопроверка не пройдена — { $why }
# $why: the controller's reason (in English).
farm-client-detail-refused = Отклонено контроллером: { $why }
# $why: the controller's reason (in English).
farm-client-detail-closed = Контроллер закрыл соединение: { $why }
# $why: the controller's reason (in English).
farm-client-detail-ended = Контроллер завершил соединение: { $why }
# $job: the job's name.
farm-client-detail-bundle = Задание «{ $job }» — получение пакета
# $job: the job's name.
farm-client-detail-job-waiting = Задание «{ $job }» — ожидание кадров для рендеринга
# $why: the limit the job is over (in English).
farm-client-detail-job-refused = Это задание отклонено: { $why }
farm-client-detail-job-over = Задание завершено
# $first and $last: frame numbers.
farm-client-detail-rendering = Рендеринг кадров { $first }–{ $last }
# $time: how long nobody must use the machine ("90 s", "5 min").
farm-client-detail-in-use = Ожидание — кто-то пользуется этим компьютером; работа возобновится, когда им никто не будет пользоваться в течение { $time }
farm-client-detail-paused = Приостановлено — контроллер уведомлён
farm-client-detail-pausing = Приостановка — завершается текущий кадр
farm-client-detail-cancelled = Текущий кадр отменён; приостановлено
# A machine with several graphics cards. $rendering: the cards rendering now; $total: all its cards (two or more).
farm-client-rendering-on-cards = Рендеринг на { $rendering } из { $total } видеокарт

## Render client: the status rows

# The value of the Controller row. $name: the controller's name; $address: HOST:PORT; $identity: its identity fingerprint.
farm-client-row-controller-value = «{ $name }» · { $address } · идентификатор { $identity }
# The value of the This machine row. $name: this machine's name; $identity: its identity fingerprint.
farm-client-row-machine-value = «{ $name }» · идентификатор { $identity }
farm-client-row-gpu = GPU
# One graphics card's row. $slot: the card's number.
farm-client-row-gpu-n = GPU { $slot }
farm-client-row-job = Задание
# The frames handed to this machine.
farm-client-row-run = Серия
farm-client-row-sent = Отправлено
# The Run row, in parts joined with " · ": the run, the frame in progress, its time. $first and $last: frame numbers.
farm-client-run-frames = кадры { $first }–{ $last }
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-client-run-frame = кадр { $frame } ({ $index } из { $count })
# $time: the time spent on it so far ("2.1 s").
farm-client-run-frame-time = { $time } на этот кадр
# One graphics card's run. $frame: the frame in progress; $first and $last: the run's frames.
farm-client-session-frame = кадр { $frame } из { $first }–{ $last }
farm-client-session-no-card = видеокарта пока неизвестна
# The Sent row. $count: frames sent; $time: the mean time per frame ("2.1 s").
farm-client-sent =
    { $count ->
        [one] { $count } кадр · { $time } на кадр
        [few] { $count } кадра · { $time } на кадр
        [many] { $count } кадров · { $time } на кадр
       *[other] { $count } кадра · { $time } на кадр
    }
# The Sent row of a machine with several cards. $count: frames sent; $split: each card's share ("812 + 422"); $time: the mean time per frame.
farm-client-sent-split =
    { $count ->
        [one] { $count } кадр ({ $split }) · { $time } на кадр
        [few] { $count } кадра ({ $split }) · { $time } на кадр
        [many] { $count } кадров ({ $split }) · { $time } на кадр
       *[other] { $count } кадра ({ $split }) · { $time } на кадр
    }
# After a warning sign. $note: how the GPUs differ, and both GPUs (in English).
farm-client-gpu-note = На этом компьютере — { $note }. Его кадры будут немного отличаться от кадров контроллера, что может проявиться в кадрах паузы.
# A collapsible section: the self-check's results.
farm-client-self-check = Самопроверка
farm-client-thumb-hover = Последний кадр, отправленный этим компьютером
farm-client-resume = Продолжить
farm-client-resume-hover = Снова брать работу
farm-client-pause = Пауза
farm-client-pause-hover = Завершить текущий кадр, а затем больше не брать работу. Контроллер передаст кадры этого компьютера остальным.
farm-client-cancel-frame = Отменить кадр
farm-client-cancel-frame-hover = Остановить текущий кадр сейчас и приостановиться; контроллер отрендерит его в другом месте.
farm-client-live-paused = Живой вид приостановлен, пока этот компьютер рендерит кадры фермы.
farm-client-close = Закрыть
farm-client-close-active-hover = Закрыть окно; этот компьютер остаётся в ферме (снова открыть окно можно из меню «Файл»)
farm-client-close-hover = Закрыть окно
farm-client-disconnect = Отключиться
farm-client-disconnect-hover = Покинуть ферму: текущий кадр возвращается
farm-client-connect = Подключиться
farm-client-connect-hover = Подключиться к контроллеру и рендерить для него кадры

## The Render on farm window (Tools ▸ Render on farm…): the controller of a render farm

farm-ctl-title = Рендеринг на ферме
farm-ctl-out = Папка вывода
farm-ctl-browse = Обзор…
farm-ctl-out-note = Кадры и состояние задания в farm/: запустите то же задание снова, чтобы продолжить его.
farm-ctl-size = Размер
farm-ctl-size-custom = Свой…
# The size list's choice when the size is typed in. $width and $height: pixels.
farm-ctl-size-custom-value = Свой — { $width }×{ $height }
farm-ctl-fps = Кадров в секунду
farm-ctl-ss = Суперсэмплинг
farm-ctl-port = Слушать порт
farm-ctl-port-hover = Клиенты подключаются к этому порту. Разрешите его в брандмауэре для вашей локальной сети.
farm-ctl-share = Общий диск
# Greyed hint inside the empty field.
farm-ctl-share-hint = необязательно — путь к нему с этого компьютера
farm-ctl-share-hover = Режим общего диска: клиенты, у которых есть тот же диск, записывают кадры на него, а не отправляют их по соединению, и этот компьютер проверяет там каждый кадр. Клиенты без него отправляют свои кадры.
# A label followed by a number of clients and two checkboxes.
farm-ctl-start-with = Начать при
# The unit after the number in the box ("Start with [3] clients"). $count: that number.
farm-ctl-start-clients =
    { $count ->
        [one] клиенте
        [few] клиентах
        [many] клиентах
       *[other] клиентах
    }
farm-ctl-start-clients-hover = Задание начинается, когда столько компьютеров подключилось и прошло самопроверку; другие могут присоединиться позже.
farm-ctl-local = Рендерить также на этом компьютере
farm-ctl-local-hover = Этот компьютер считается одним клиентом.
farm-ctl-sharing = Делиться опорными орбитами
farm-ctl-sharing-hover = Построение опорной орбиты глубокого кадра может занять от нескольких минут до нескольких часов. При совместном использовании она строится один раз и передаётся другим компьютерам. Общая опорная орбита — другая, но столь же корректная: на экстремальной глубине несколько пикселей могут отличаться от кадра, который построил собственную.
farm-ctl-key = Ключ фермы
# A checkbox beside the shortened farm key: show all of it.
farm-ctl-show-key = Показать
farm-ctl-copy-key = Копировать ключ
farm-ctl-copy-key-hover = Вставьте его в окно «Клиент рендеринга» каждого клиента
# In place of the farm key, when there is none.
farm-ctl-no-key = пока нет
farm-ctl-make-key = Создать ключ
farm-ctl-make-key-hover = Он нужен клиентам для подключения; он также создаётся при нажатии «Рендерить»
# What to do on the other machines. $address: this controller's HOST:PORT.
farm-ctl-steps = На каждом клиенте: Файл ▸ Клиент рендеринга…, контроллер { $address } и ключ фермы.
# The same when this machine's address is not known. $port: the port number.
farm-ctl-steps-port = На каждом клиенте: Файл ▸ Клиент рендеринга…, адрес этого компьютера и порт { $port }, а также ключ фермы.
# The size of the job. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
farm-ctl-summary =
    { $count ->
        [one] { $count } кадр · { $seconds }s тура · { $width }×{ $height } ss{ $ss }
        [few] { $count } кадра · { $seconds }s тура · { $width }×{ $height } ss{ $ss }
        [many] { $count } кадров · { $seconds }s тура · { $width }×{ $height } ss{ $ss }
       *[other] { $count } кадра · { $seconds }s тура · { $width }×{ $height } ss{ $ss }
    }
farm-ctl-strip-hover = Все кадры задания слева направо: готовые, рендерящиеся, ещё не сделанные, брошенные.
farm-ctl-no-tour = Тур не загружен.
# $folder: the folder; $why: the system's reason.
farm-ctl-out-error = Папка вывода { $folder }: { $why }
# $why: the system's reason.
farm-ctl-start-failed = Не удалось запустить ферму: { $why }
# $code: the controller process's exit code, as the code gives it.
farm-ctl-ended-exit = Контроллер завершил работу (код выхода { $code }).

## Render on farm: the state, in the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-phase-waiting = Ожидание клиентов рендеринга (подключено { $connected } из { $needed })
farm-ctl-phase-preparing = Подготовка задания
farm-ctl-phase-rendering = Рендеринг
farm-ctl-phase-paused = Приостановлено
farm-ctl-phase-complete = Готово
farm-ctl-phase-stopped = Остановлено
farm-ctl-phase-ended = Завершено

## Render on farm: the words after the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-detail-waiting = Ожидание клиентов рендеринга: подключено { $connected } из { $needed }
farm-ctl-detail-preparing = Измерение якорей нормализации на этом компьютере, один раз для всех клиентов
farm-ctl-detail-no-space = Ожидание свободного места в папке вывода
farm-ctl-detail-paused = Приостановлено: новая работа не раздаётся
farm-ctl-detail-not-rendered = Завершено, но часть кадров не отрендерена — запустите снова на той же папке, чтобы продолжить
farm-ctl-detail-stopped = Остановлено — запустите снова на той же папке, чтобы продолжить
# $code: the controller process's exit code.
farm-ctl-detail-ended = Завершено (код выхода { $code })
farm-ctl-detail-finishing = Завершение

## Render on farm: progress

# On the progress bar. $done: frames done; $total: frames in the job.
farm-ctl-progress =
    { $total ->
        [one] { $done } / { $total } кадр
        [few] { $done } / { $total } кадра
        [many] { $done } / { $total } кадров
       *[other] { $done } / { $total } кадра
    }
# Added to the progress bar. $rate: frames per second, with decimals.
farm-ctl-progress-rate = { $rate } кадр/с
# Added to the progress bar. $time: the time still to go ("38m 10s").
farm-ctl-progress-eta = осталось около { $time }
# Facts under the progress bar, joined with " · ". $received and $sent: kilobytes per second.
farm-ctl-io = вход { $received } KB/s · выход { $sent } KB/s
# $space: free space on the output drive ("412 GB").
farm-ctl-free = свободно { $space }
# $time: the time since the job started ("38m 10s").
farm-ctl-elapsed = прошло { $time }
# $count: frames given up on.
farm-ctl-failed = не отрендерено: { $count }
# Reference orbits. $built: built for this job; $cached: found in the cache; $reused: reused between frames; $shared: passed between machines.
farm-ctl-refs = опорные орбиты: построено { $built }, из кэша { $cached }, повторно использовано { $reused } · передано { $shared }
# After a warning sign.
farm-ctl-storage-low = Папка вывода почти заполнена: новая работа не раздаётся, пока не появится место.
# After a warning sign.
farm-ctl-gpus-differ = В этой ферме разные GPU. Их кадры немного различаются, что может проявиться в кадрах паузы.
# After a warning sign. $count: the GPU classes in the farm.
farm-ctl-gpus-differ-classes = В этой ферме разные GPU (классов GPU: { $count } — разные GPU, драйверы или пробные рендеры). Их кадры немного различаются, что может проявиться в кадрах паузы.
# $gpu: this machine's GPU, API and driver.
farm-ctl-this-gpu = Этот компьютер: { $gpu }
# A machine whose GPU differs. $name: the machine; $note: how its GPU differs (in English); $adapter: its GPU.
farm-ctl-gpu-row = { $name }: { $note } — { $adapter }
# The same with its driver. $driver: the driver's name and version.
farm-ctl-gpu-row-driver = { $name }: { $note } — { $adapter }, драйвер { $driver }

## Render on farm: the table of machines

# Column headers (short: the table is narrow).
farm-ctl-col-machine = Компьютер
farm-ctl-col-state = Состояние
farm-ctl-col-gpu = GPU
farm-ctl-col-frames = Кадры
farm-ctl-col-per-frame = На кадр
# How the machine delivers its frames (a speed, or "shared drive").
farm-ctl-col-link = Связь
# When the controller last heard from the machine.
farm-ctl-col-heard = Отклик
# In the machine's tooltip. $count: its frames that failed verification.
farm-ctl-strikes =
    { $count ->
        [one] { $count } кадр не прошёл проверку
        [few] { $count } кадра не прошли проверку
        [many] { $count } кадров не прошли проверку
       *[other] { $count } кадра не прошли проверку
    }
# In the Link column: it delivers its frames through the shared drive.
farm-ctl-link-share = общий диск
# In the Heard column. $seconds: how long ago.
farm-ctl-heard-ago = { $seconds } s назад
farm-ctl-readmit = Допустить снова
farm-ctl-readmit-hover = Позволить этому компьютеру снова присоединиться, когда он переподключится
farm-ctl-remove = Убрать
farm-ctl-remove-hover = Вывести этот компьютер из задания; его кадры переходят к остальным
# The GPU column's tooltip.
farm-ctl-probe-same = Его пробный рендер идентичен рендеру этого компьютера.
# $count: the pixels that differ.
farm-ctl-probe-differs =
    { $count ->
        [one] Его пробный рендер отличается от рендера этого компьютера в { $count } пикселе: его кадры будут немного отличаться от кадров этого компьютера.
        [few] Его пробный рендер отличается от рендера этого компьютера в { $count } пикселях: его кадры будут немного отличаться от кадров этого компьютера.
        [many] Его пробный рендер отличается от рендера этого компьютера в { $count } пикселях: его кадры будут немного отличаться от кадров этого компьютера.
       *[other] Его пробный рендер отличается от рендера этого компьютера в { $count } пикселях: его кадры будут немного отличаться от кадров этого компьютера.
    }
farm-ctl-probe-unknown = Его пробный рендер ещё не сравнивался.
# $driver: the driver's name and version.
farm-ctl-gpu-driver = драйвер { $driver }
# After a warning sign. $note: how its GPU differs (in English: "a different GPU").
farm-ctl-gpu-note = { $note } (в отличие от этого компьютера)

## Render on farm: a machine's state, in the State column

# $count: its frames that failed verification.
farm-ctl-state-removed-bad =
    { $count ->
        [one] убран: { $count } плохой кадр
        [few] убран: { $count } плохих кадра
        [many] убран: { $count } плохих кадров
       *[other] убран: { $count } плохого кадра
    }
farm-ctl-state-removed = убран вами
farm-ctl-state-removing = удаление — сбор его диагностики
farm-ctl-state-gone = ушёл
farm-ctl-state-leaving = уходит
farm-ctl-state-checking = проверка (самопроверка)
farm-ctl-state-paused = приостановлен своим пользователем
farm-ctl-state-parked = отложен: дважды упал — вернётся при переподключении
farm-ctl-state-refused = отклонил это задание (собственные ограничения)
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-ctl-state-rendering = рендеринг { $frame } ({ $index }/{ $count })
farm-ctl-state-starting = начало серии
farm-ctl-state-idle = простой
farm-ctl-state-admitted = допущен — ожидает задания

## Render on farm: the buttons

# A collapsible section: the controller's messages.
farm-ctl-events = События
farm-ctl-resume = Продолжить
farm-ctl-pause = Пауза
farm-ctl-pause-hover = Не раздавать новую работу; текущие кадры будут завершены
farm-ctl-live-paused = Живой вид приостановлен, пока этот компьютер рендерит для фермы.
farm-ctl-close = Закрыть
farm-ctl-close-active-hover = Закрыть окно; задание продолжает выполняться (снова открыть окно можно из меню «Инструменты»)
farm-ctl-close-hover = Закрыть окно
farm-ctl-copy-command = Копировать команду
farm-ctl-copy-command-hover = Скопировать эквивалентную командную строку
farm-ctl-stop = Остановить
# The Stop button once a stop has been asked for and the job has not ended after 10 seconds.
farm-ctl-stop-now = Остановить сейчас
farm-ctl-stop-hover = Завершить задание с уже готовыми кадрами; повторный запуск того же задания продолжит его
farm-ctl-render = Рендерить
farm-ctl-render-hover = Начать ожидать клиентов и отрендерить на них тур

## The Render tour window (Tools ▸ Render tour…)

tour-render-title = Рендеринг тура
tour-render-out = Папка вывода
tour-render-browse = Обзор…
tour-render-out-hover = Полный путь, по которому записываются кадры.
tour-render-prefix = Префикс кадров
# <prefix> stands for the prefix typed in the field.
tour-render-prefix-hover = Кадры записываются как <префикс>_00000.png
tour-render-size = Размер
tour-render-size-custom = Свой…
# The size list's choice when the size is typed in. $width and $height: pixels.
tour-render-size-custom-value = Свой — { $width }×{ $height }
tour-render-fps = Кадров в секунду
tour-render-fps-hover = Меньше 1 fps — это нормально: выборка длинного погружения с частотой 0,25 fps — способ проверять паузы глубоких регрессионных тестов, не рендеря тысячи кадров.
tour-render-ss = Суперсэмплинг
tour-render-chapter = Глава
tour-render-whole = Весь тур
tour-render-chapter-hover = Рендеринг одной главы сохраняет ГЛОБАЛЬНУЮ нумерацию кадров, поэтому её кадры сразу встают на своё место в полной последовательности.
tour-render-mp4 = Собрать mp4
tour-render-mp4-hover = Затем запускает ffmpeg (он должен быть в PATH). Кадры PNG сохраняются.
tour-render-overwrite = Перезаписывать существующие кадры
tour-render-overwrite-hover = Если выключено, рендеринг остановится, чтобы спросить, а отсюда он этого сделать не может.
tour-render-resume = Продолжить прерванный рендеринг
tour-render-resume-hover = Сохранить кадры, уже имеющиеся в папке вывода, и отрендерить только недостающие. Сначала проверяется самый новый кадр и отбрасывается, если он неполон (кадр, на котором рендеринг прервался, — это тот, который он записывал), с отступлением назад, пока не найдётся хороший; папка с кадрами другого размера отклоняется, а не смешивается с этой последовательностью.
# $count: this machine's graphics cards.
tour-render-all-gpus = Использовать все видеокарты ({ $count })
tour-render-all-gpus-hover = Рендерить сразу на всех видеокартах этого компьютера через рендер-ферму на нём же (--gpus all). Кадры, их имена, «Продолжить прерванный рендеринг» и mp4 — как для одной карты; рендерится весь тур, в собственном порядке фермы. Разные модели карт рисуют несколько пикселей чуть по-разному: кадр паузы и наплыв остаются на одной карте.
tour-render-order = Порядок рендеринга
tour-render-sequential = Последовательный
tour-render-sequential-hover = Кадры 0, 1, 2, … по порядку (по умолчанию).
tour-render-progressive = Прогрессивный
tour-render-progressive-hover = Сначала предпросмотр всего тура, затем уточнение до полной детализации: сначала рендерятся ключевые кадры, затем самые большие промежутки во времени делятся пополам, пока не появится каждый кадр. Кадры в любом случае попадают на свои правильные номера, поэтому «Продолжить прерванный рендеринг» и mp4 работают без изменений — можно остановиться рано, просмотреть грубую раскадровку и перезапуститься с продолжением в любом из двух порядков.
# The cost of the render. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
tour-render-summary =
    { $count ->
        [one] { $count } кадр · { $seconds }s тура · { $width }×{ $height } ss{ $ss }
        [few] { $count } кадра · { $seconds }s тура · { $width }×{ $height } ss{ $ss }
        [many] { $count } кадров · { $seconds }s тура · { $width }×{ $height } ss{ $ss }
       *[other] { $count } кадра · { $seconds }s тура · { $width }×{ $height } ss{ $ss }
    }
# The disk space the frames will take (an estimate). $size: a size ("5.3 GB").
tour-render-disk = ≈{ $size } на кадры
# $size: the frames' estimated size; $free: the free space on that drive ("6.0 GB").
tour-render-disk-free = ≈{ $size } на кадры · свободно { $free }
# After a warning sign. $disk: the line above ("≈51.0 GB of frames · 6.0 GB free").
tour-render-disk-short = { $disk } — недостаточно места для всей последовательности
tour-render-separate = Рендерит сценарий С ДИСКА в отдельном процессе, поэтому несохранённые правки не учитываются, — зато потеря GPU-устройства там не утянет за собой этот сеанс.
# On the progress bar. $done: frames done; $total: frames to render.
tour-render-progress =
    { $total ->
        [one] { $done } / { $total } кадр
        [few] { $done } / { $total } кадра
        [many] { $done } / { $total } кадров
       *[other] { $done } / { $total } кадра
    }
# Before the render's first progress line.
tour-render-starting = запуск…
tour-render-live-paused = Живой вид приостановлен — рендеринг получает GPU в своё распоряжение, поэтому завершается быстрее и точно измеряет стоимость своих кадров.
tour-render-cancel = Отмена
tour-render-cancel-hover = Закрыть без рендеринга
tour-render-copy-command = Копировать команду
tour-render-copy-command-hover = Скопировать эквивалентную командную строку в буфер обмена
tour-render-stop = Остановить рендеринг
tour-render-render = Рендерить
tour-render-render-hover = Начать рендеринг последовательности кадров
# $folder: the output folder.
tour-render-finished = Рендеринг завершён → { $folder }
# $why: the render's own reason (in English).
tour-render-failed = Рендеринг не удался: { $why }
# $last: the render's last progress line (in English), or nothing.
tour-render-failed-or-stopped = Рендеринг не удался или был остановлен. { $last }
# $why: the system's reason.
tour-render-no-exe = Не удаётся найти исполняемый файл: { $why }
# $why: the system's reason.
tour-render-start-failed = Не удалось запустить рендеринг: { $why }

## The Custom formula dialog (Fractal ▸ Custom formula…)
## The formula language itself stays as it is in every language: function names (sin, cosh, conj…),
## operators, variables (z, c, p1), and the formulas. Keep them as written inside these messages.

# The window's title.
formula-dialog-title = Своя формула
# A small line at the top of the window, above the formula.
formula-dialog-intro = Шаг в виде выражений в стиле Fractint
# The two ways to show and edit the formula (top right of the window).
formula-dialog-textbook = Учебник
formula-dialog-textbook-hover = Показать формулу набранной, как в учебнике, и править её в таком виде
formula-dialog-text = Текст
formula-dialog-text-hover = Править формулу как текст
# The tooltip of the "LaTeX" button (the button's own text stays LaTeX).
formula-dialog-latex-hover = Скопировать формулу как LaTeX — в том виде, как она набрана здесь
# The drop-down list of example formulas (the collection).
formula-dialog-examples = Примеры
# Help under the formula in Textbook mode. The characters / ^ ( are what the user types: keep them.
formula-dialog-help-textbook = Вводите как в тексте: / делает дробь из предшествующего члена, ^ — показатель степени, ( — скобки. Стрелки перемещают по формуле; Tab переходит к следующему пустому полю. Строка, выделенная красным, не читается: щёлкните по ней, чтобы править её как текст.
# Help under the formula in Text mode. "; starts a comment": the ; is what the user types.
formula-dialog-help-text = Вводите с клавиатуры или используйте панель клавиш — в ней есть все имена, которые знает язык формул. Имена дополняются по мере ввода (Tab). Операторы разделяются новой строкой или запятой; ; начинает комментарий.
# The syntax check, when the formula reads.
formula-dialog-reads = Читается правильно.
# The syntax check in Textbook mode, when the formula has an empty box to fill in.
formula-dialog-empty-box = Заполните пустое поле.
# The syntax check in Textbook mode. $line: the line number; $message: what is wrong (from the formula parser, in English).
formula-dialog-error-line = Строка { $line }: { $message }
# One function written with another notation's name, and the language's name for it: "ln to log".
# $from: the name as written (ln); $to: the formula language's name (log).
formula-respell-pair = { $from } на { $to }
# A button that rewrites them. $changes: one or more formula-respell-pair, joined with ", ".
formula-respell = Заменить { $changes }
formula-respell-hover = Записать эти функции так, как их называет язык формул
# Shown when Apply finds a parameter field that is not a number. $name: the parameter (p1 … p5).
formula-param-not-numbers = { $name } — не пара чисел
# The Apply button (also in the Formula library window).
formula-apply = Применить
formula-dialog-apply-hover = Скомпилировать формулу и показать её
# Shown beside Apply while the formula is being compiled.
formula-dialog-compiling = Компиляция формулы для GPU…
# The greyed hint in an empty name box (also in the Formula library window).
formula-name-hint = имя (необязательно)
# The save button: "Update" when the library already has a formula of that name.
formula-dialog-update-in-library = Обновить в библиотеке
formula-dialog-save-to-library = Сохранить в библиотеку
formula-dialog-save-hover = Сохранить эту формулу и её параметры в библиотеке формул
formula-dialog-save-disabled-hover = Сохранить можно только формулу, которая читается правильно
# Opens the Formula library window.
formula-dialog-library = Библиотека…
formula-dialog-library-hover = Сохранённые формулы: применяйте, редактируйте, импортируйте и экспортируйте их

## The Custom formula dialog's keypad
## The keys' captions are formula tokens and stay as they are; these are their tooltips.

# The keypad's third tab (the other two are "123" and "f(z)"). A short word.
formula-keypad-tab-names = имена
# The tabs' tooltips.
formula-keypad-tab-basic-hint = Числа, операторы и самые используемые имена
formula-keypad-tab-functions-hint = Функции
formula-keypad-tab-names-hint = Все переменные, параметры и константы, а также синтаксис операторов
# The key z.
formula-key-z = Итерируемое значение
# The key c.
formula-key-c = Пиксель (константа Жюлиа в режиме Жюлиа)
# The key p1 on the first tab.
formula-key-p1-below = Параметр 1 — задаётся в полях ниже
# The keys p1 … p5. $n: the parameter's number.
formula-key-param = Параметр { $n }
formula-key-multiply = Умножить
formula-key-divide = Разделить
# The key that types ^2.
formula-key-square-after = Квадрат (после значения: z^2)
# The key that types ^.
formula-key-power = Степень: целый, действительный или комплексный показатель
formula-key-square-root = Квадратный корень
# The key that types |…|: in this language it is the squared modulus, as in Fractint.
formula-key-squared-modulus = КВАДРАТ модуля |z|² — как во Fractint
formula-key-add = Сложить
formula-key-subtract = Вычесть (или сменить знак)
# The key that types =.
formula-key-assign = Присвоить: z = … или временная t = …
# The key that starts a new line (first tab).
formula-key-next-statement-comma = Следующий оператор (подойдёт и запятая)
# The key "del".
formula-key-backspace = Удалить символ перед курсором (или выделение)
# The key e.
formula-key-euler = Число Эйлера
# The key that types (0.5, 0.5).
formula-key-complex-constant = Комплексная константа: (действительная, мнимая)
# The key that types a comma.
formula-key-comma = Разделяет операторы или части комплексной константы
formula-key-left = Курсор влево
formula-key-right = Курсор вправо
# The function keys (the f(z) tab), one per function.
formula-key-sin = Синус
formula-key-cos = Косинус
formula-key-tan = Тангенс
formula-key-cotan = Котангенс
formula-key-exp = Экспонента e^z
formula-key-sinh = Гиперболический синус
formula-key-cosh = Гиперболический косинус
formula-key-tanh = Гиперболический тангенс
formula-key-cotanh = Гиперболический котангенс
formula-key-log = Натуральный логарифм, ln (главная ветвь)
formula-key-sqr = Квадрат: sqr(z) = z²
formula-key-sqrt = Квадратный корень (главное значение)
formula-key-recip = Обратная величина 1/z
formula-key-conj = Комплексное сопряжение
formula-key-flip = Поменять местами действительную и мнимую части
formula-key-abs = Модуль ОБЕИХ частей: |x| + i|y|
formula-key-cabs = Модуль |z| (действительное число)
formula-key-real = Действительная часть
formula-key-imag = Мнимая часть как действительное число
# "fn" is Fractint's name for a function slot: keep it.
formula-key-ident = Тождественная функция (заполнитель fn из Fractint)
# The key "pixel".
formula-key-pixel = То же, что c (имя из Fractint)
# The key "z =".
formula-key-set-z = Начать оператор, задающий z
# The key "t =".
formula-key-temporary = Временная переменная: подойдёт любое новое имя (t, w, a2, …)
# The key "; …".
formula-key-comment = Комментарий до конца строки
# The key that starts a new line (names tab).
formula-key-next-statement = Следующий оператор

## The formula's completion list (names offered as you type)

# Beside a name the formula itself assigns.
formula-completion-variable = Переменная, которую задаёт эта формула
# Beside a function typed with another notation's name.
# $what: what the function is, in English (from the formula parser: "The natural logarithm");
# $other: the other notation's name (ln); $ours: the formula language's name (log).
formula-completion-other-spelling = { $what } — { $other } здесь пишется как { $ours }
# Under a list cut short. $count: the names not shown.
formula-completion-more = … ещё { $count }
# The last line of the list. Tab, Enter and Esc are key names.
formula-completion-keys = Tab или Enter — дополнить, Esc — закрыть

## The Custom formula dialog's Textbook mode

# The tooltip of a line that does not read. $why: what is wrong (from the formula parser, in English, or formula-textbook-fractint-sections).
formula-textbook-unread-hover = Не читается: { $why }. Щёлкните, чтобы править её как текст.
# Why a line with Fractint's sections is shown as text in Textbook mode.
formula-textbook-fractint-sections = Разделы Fractint (':' раздела инициализации, сравнения, a = b = …) правятся как текст

## The Formula library window (Fractal ▸ Formula library…)

# The window's title.
formula-library-title = Библиотека формул
# The window's two tabs. $count: the formulas in each.
formula-library-mine = Мои формулы ({ $count })
formula-library-mine-hover = Формулы, которые вы сохранили или импортировали
formula-library-collection = Коллекция ({ $count })
formula-library-collection-hover = Формулы, поставляемые с Fractadyne, у каждой — свой начальный вид
# A small marker before the name of the formula the view shows now. A short word.
formula-library-showing = показана
# The tooltip of Apply on a formula of the collection.
formula-library-apply-shipped-hover = Показать её на её начальном виде
# A row's Edit button (after a pencil icon).
formula-library-edit = Изменить
formula-library-edit-hover = Открыть её в диалоге «Своя формула», не применяя
# A collection row's button that copies the formula into "My formulas".
formula-library-copy-to-mine = Копировать к себе
formula-library-copy-to-mine-hover = Добавить её к вашим формулам, чтобы сохранить или изменить
formula-library-close = Закрыть
# Saves the formula the view shows, under the name typed beside it.
formula-library-add-current = Добавить текущую формулу
formula-library-add-current-hover = Сохранить формулу, которую показывает вид
# "Fractal > Custom formula…" is the menu path: write it as the menus are named.
formula-library-add-current-disabled-hover = Сначала примените свою формулу (Фрактал > Своя формула…)
formula-library-import = Импорт…
formula-library-import-hover = Добавить формулы из файла формул; ничего здесь не заменяется
formula-library-export-all = Экспортировать все…
formula-library-export-all-hover = Записать все формулы отсюда в один файл, чтобы поделиться ими или сохранить
# The filter box: its label, and the greyed hint inside it.
formula-library-filter = Фильтр
formula-library-filter-hint = фильтр по имени или тексту
# Shown while the user has no saved formulas.
formula-library-empty = Сохранённых формул пока нет. Сохраните формулу из диалога «Своя формула», добавьте ту, что показывает вид, или импортируйте файл (наш или .frm из Fractint).
# The tooltip of Apply on one of the user's formulas: with a view saved, and without one.
formula-library-apply-view-hover = Показать эту формулу на сохранённом с ней виде
formula-library-apply-hover = Показать эту формулу
# The tooltip of a row's save icon.
formula-library-export-one-hover = Экспортировать эту формулу в файл
# A row asks before deleting: the question, then its two buttons. "Delete" is also the delete icon's tooltip.
formula-library-delete-ask = Удалить её?
formula-library-delete = Удалить
formula-library-keep = Оставить
# Under a list cut short. $count: the formulas not listed.
formula-library-more = …и ещё { $count }: введите текст в фильтр, чтобы найти нужную.
formula-library-no-match = Под фильтр ничего не подходит.
# Under a saved formula this version cannot read. $why: what is wrong (from the formula parser, in English).
formula-library-unread = Не читается в этой версии: { $why }

## Formula library notices

# $why: the system's reason, in English.
formula-library-save-failed = Не удалось сохранить библиотеку формул: { $why }
formula-library-nothing-to-save = Нечего сохранять: формула пуста.
# $name: the formula's name in the library.
formula-library-updated = Формула «{ $name }» обновлена в библиотеке формул.
formula-library-saved = Формула «{ $name }» сохранена в библиотеку формул.
# $name: the formula's name; $why: what is wrong.
formula-library-apply-failed = Формулу «{ $name }» нельзя применить: { $why }
# Copying a formula of the collection into "My formulas". $name: its name; $as_named: the name it was given (the same, or "Name (2)" when the name was taken).
formula-library-copy-exists = Формула «{ $name }» уже есть в ваших формулах.
formula-library-copied = Формула «{ $name }» скопирована в ваши формулы как «{ $as_named }».
# $name: the formula's name.
formula-library-deleted = Формула «{ $name }» удалена из библиотеки формул.
# Why the library could not be saved (follows formula-library-save-failed).
formula-library-no-config-dir = нет папки конфигурации
# At startup, when the library file cannot be read. $why: the reason, in English; $path: a file's full path.
formula-library-unreadable-kept = Не удалось прочитать библиотеку формул ({ $why }). Она сохранена как { $path }, и начата новая, пустая библиотека.
formula-library-unreadable-stuck = Библиотеку формул не удалось ни прочитать ({ $why }), ни переместить: скопируйте { $path } в надёжное место, прежде чем сохранять формулу, — сохранение её заменит.

## Formula library: Import and Export
## A formula file's contents stay in English; this is what the windows and notices around them say.

# File-type names in the open and save dialogs (the extensions are added by the system).
formula-filter-formula-files = Файлы формул
formula-filter-fractadyne = Формулы Fractadyne
formula-filter-fractint = Формулы Fractint
formula-filter-all-files = Все файлы
# Why a file cannot be imported (they follow formula-import-failed).
formula-import-too-large = слишком велик для файла формул
formula-import-not-text = не текстовый файл
formula-import-no-formulas = в нём нет формул
# $why: the TOML reader's message, in English.
formula-file-not-formulas = не файл формул ({ $why })
# $format: what the file says it is ("fractadyne-gradients").
formula-file-other-format = файл «{ $format }», а не файл формул
# "[[formula]]" is the file's own syntax: keep it.
formula-file-no-formulas = в нём нет формул (файл формул перечисляет их как таблицы [[formula]])
# $file: the file's name; $why: one of the reasons above, or the system's (in English).
formula-import-failed = Не удалось импортировать «{ $file }»: { $why }.
# After an import that added nothing. $file: the file's name.
formula-import-nothing-new = В «{ $file }» нет ничего нового.
# After an import. $count: the formulas added; $file: the file's name.
formula-import-added =
    { $count ->
        [one] Импортирована { $count } формула из «{ $file }».
        [few] Импортированы { $count } формулы из «{ $file }».
        [many] Импортировано { $count } формул из «{ $file }».
       *[other] Импортировано { $count } формулы из «{ $file }».
    }
# The same two, followed by what else happened. $notes: one or more of the formula-import-note-… messages below, joined with "; ".
formula-import-nothing-new-notes = В «{ $file }» нет ничего нового: { $notes }.
formula-import-added-notes =
    { $count ->
        [one] Импортирована { $count } формула из «{ $file }»: { $notes }.
        [few] Импортированы { $count } формулы из «{ $file }»: { $notes }.
        [many] Импортировано { $count } формул из «{ $file }»: { $notes }.
       *[other] Импортировано { $count } формулы из «{ $file }»: { $notes }.
    }
# Notes (clauses, no final full stop). $count: the formulas already in the library, skipped.
formula-import-note-duplicates =
    { $count ->
        [one] { $count } формула уже была в библиотеке
        [few] { $count } формулы уже были в библиотеке
        [many] { $count } формул уже были в библиотеке
       *[other] { $count } формулы уже были в библиотеке
    }
# One formula was renamed because its name was taken. $from: its name in the file; $to: the name it was given ("Name (2)").
formula-import-note-renamed-one = имя «{ $from }» было занято, поэтому формула называется «{ $to }»
# $count: the formulas renamed because their names were taken (2 or more).
formula-import-note-renamed = переименовано из-за занятых имён: { $count }
# $count: the entries skipped as empty or too long.
formula-import-note-skipped =
    { $count ->
        [one] { $count } запись была пустой или слишком длинной и пропущена
        [few] { $count } записи были пустыми или слишком длинными и пропущены
        [many] { $count } записей были пустыми или слишком длинными и пропущены
       *[other] { $count } записи были пустыми или слишком длинными и пропущены
    }
# A second sentence after importing a Fractint .frm file. $count: the entries that do not read; $why: the commonest reason (from the formula parser, in English).
formula-import-frm-unread =
    { $count ->
        [one] { $count } запись не читается в этой версии (чаще всего: { $why }).
        [few] { $count } записи не читаются в этой версии (чаще всего: { $why }).
        [many] { $count } записей не читаются в этой версии (чаще всего: { $why }).
       *[other] { $count } записи не читаются в этой версии (чаще всего: { $why }).
    }
# After exporting one formula. $name: the formula's name; $file: the file's name.
formula-export-one = Формула «{ $name }» экспортирована в «{ $file }».
# After exporting several. $count: how many; $file: the file's name.
formula-export-many =
    { $count ->
        [one] Экспортирована { $count } формула в «{ $file }».
        [few] Экспортированы { $count } формулы в «{ $file }».
        [many] Экспортировано { $count } формул в «{ $file }».
       *[other] Экспортировано { $count } формулы в «{ $file }».
    }
# $file: the file's name; $why: the system's reason, in English.
formula-export-failed = Не удалось записать «{ $file }»: { $why }

## The control panel (right-hand side): header and section headings

# The panel's title, beside the button that hides it.
panel-title = Управление
panel-hide-hover = Скрыть панель управления
# The button that brings the hidden panel back (top right of the view).
panel-show-hover = Показать панель управления
# Section headings.
panel-section-navigate = Навигация
panel-section-coloring = Раскраска
panel-section-quality = Качество
panel-section-effects = Эффекты
panel-section-overlays = Наложения
panel-section-advanced = Дополнительно
# The last section: reference notes about the current fractal. $name: the fractal's name (Mandelbrot, Burning Ship…).
panel-section-about = О фрактале «{ $name }»
panel-section-performance = Производительность

## The control panel: Navigate
## Labels before a control sit in a column about 104 px wide (roughly 15 characters): a longer label pushes its control to the right.

panel-zoom-speed = Скорость приближения
panel-zoom-speed-hover = Скорость непрерывного приближения при удержании Space (1× ≈ 2× за 1,5 s).
panel-snap-zoom = Притяжение к объектам
panel-snap-zoom-hover = При приближении колесом мыши или клавишей Space притягивать приближение к миниброту (или к действительной оси) рядом с курсором, чтобы он скользнул под курсор и приближение пришлось точно на него. Чем ближе объект, тем сильнее притяжение. Только Mandelbrot, одиночный вид.
panel-click-zoom = Приближение щелчком
panel-click-zoom-hover = Когда включено, левый щелчок в виде ныряет с коэффициентом, заданным ниже (правый щелчок возвращает назад), с центрированием на точке щелчка. Перетаскивание по-прежнему сдвигает вид; Shift+перетаскивание / перетаскивание правой кнопкой по-прежнему приближает рамкой. Backspace отменяет щелчок. В двойном виде работает на любой из панелей, а закрепление c Жюлиа, пока это включено, переходит на Ctrl+щелчок. Удерживайте Shift, чтобы вызвать под курсором лупу, показывающую вид, в который приведёт щелчок, и щёлкните, удерживая её, чтобы перейти в эту точку; Shift+ПЕРЕТАСКИВАНИЕ по-прежнему приближает рамкой.
# The click-to-zoom factor, followed by buttons 2×, 4×, 10×…
panel-click-zoom-factor = Коэффициент
panel-click-zoom-snap = Притягивать к ближайшему центру
panel-click-zoom-snap-hover = После перехода найти ближайшее ядро миниброта и точно встать на него, чтобы оказаться на объекте, а не рядом с ним. Масштаб не меняется — его выбрал щелчок. Только Mandelbrot. Вычисление идёт в фоне: вид переходит сразу и поправляет себя, когда приходит ответ, что на экстремальной глубине может занять несколько секунд.

## The control panel: Coloring
## Labels before a control sit in a column about 104 px wide (roughly 15 characters).

panel-method = Метод
panel-method-hover = Как данные убегания превращаются в цвет. «Среднее полос» / «Неравенство треугольника» / «Ловушка орбиты» / «Декомпозиция» раскрывают структуру орбит; «Оценка расстояния» затеняет по близости к границе.
panel-stripe-density = Плотность полос
panel-stripe-tail = Только хвост (глуб. приближение)
panel-stripe-tail-hover = Усреднять член полос по последним N итерациям каждой орбиты (экспоненциальное окно), а не по всей орбите. На экстремальной глубине каждая орбита повторяет опорную на протяжении всех итераций, кроме последних нескольких сотен, поэтому среднее по всей орбите даёт один цвет на весь вид; хвост сохраняет контраст. Орбиты короче окна не затрагиваются. Также --stripe-tail N для --render.
panel-tail-length = Длина хвоста
# The unit after the tail length's number: short for "iterations" (the slider reads "64 it").
panel-tail-length-unit = ит
panel-tail-length-hover = Длина окна в итерациях. Короткие окна (8–32) дают классические чёткие полосы на любой глубине; более длинные снова усредняются к ровному цвету, поскольку член полос длинной хаотической орбиты усредняется.
panel-trap-shape = Форма ловушки
panel-palette = Палитра
# The two colours of the Binary palette, each beside its colour swatch.
panel-in-set = Внутри
panel-out-of-set = Снаружи
# The two colours of the Duotone palette, each beside its colour swatch.
panel-shadow = Тень
panel-highlight = Свет
panel-cycle = Цикл
panel-offset = Сдвиг
panel-normalize-hover = На экстремальной глубине числа итераций до убегания охватывают сотни тысяч, и фиксированный «Цикл» оборачивает палитру тысячи раз между соседними пикселями — правильное плотное поле выглядит как шумная рябь. Эта функция перераспределяет палитру по измеренному диапазону убегания вида («Цикл» тогда задаёт, сколько проходов палитры его охватывают), КОГДА у вида действительно есть алиасинг, — плавный вид остаётся как есть, потому что перераспределение изменило бы картинку впустую. Отметьте ниже «Всегда подгонять диапазон», чтобы перераспределять в любом случае. Только для плавного метода; соответствует --normalize.
# A checkbox indented under "Normalize deep colors" (also in the Color menu).
panel-fit-range = Всегда подгонять диапазон
panel-fit-range-hover =
    Перераспределять палитру по измеренному диапазону убегания вида, даже когда у вида нет алиасинга, — флажок выше перераспределяет её, только когда плотное поле иначе выглядело бы как рябь.

    Используйте это, когда хотите, чтобы палитра охватывала то, что на экране, а не чтобы устранить шум. Для этого всё равно нужно измерение, поэтому в первые мгновения, пока вид не устоится, ничего не происходит, а на виде без убежавших пикселей — не происходит вообще.
# A checkbox indented under "Always fit range" (also in the Color menu).
panel-log-scale = Логарифмическая шкала цвета
panel-log-scale-hover =
    Распределять палитру по ЛОГАРИФМУ значения убегания, а не линейно. На глубине числа итераций до убегания скапливаются у верхнего края, поэтому линейное отображение тратит большую часть палитры на тонкую оболочку у границы и сплющивает всё остальное.

    Применяется к нормализованному отображению — его включает «Нормализовать глубокие цвета» (или --normalize при экспорте). Сильно перекошенный диапазон получает логарифмическое отображение в любом случае, отмечен этот флажок или нет.
# The same checkbox's tooltip while it is greyed out (normalization is off).
panel-log-scale-off-hover =
    Требуется нормализация: этот флажок выбирает, КАК палитра отображается на измеренный диапазон убегания, поэтому при выключенном «Нормализовать глубокие цвета» (и без экспорта с --normalize) ему нечего менять, и он не изменил бы ни одного пикселя.

    Включите «Нормализовать глубокие цвета», чтобы им воспользоваться.
# The palette animation: its kind, then its speed.
panel-animate = Анимация
panel-anim-speed = Скорость
panel-anim-speed-hover = Скорость цикла: циклов сдвига цвета в секунду или (для «Случайные градиенты») смен градиента в секунду.
panel-shuffle-gradient = Перемешать градиент

## The control panel: Quality

panel-auto-iter = Автомасштаб итераций по глубине
# The iteration-count label while auto-scaling is on (the count is then a base the zoom raises).
panel-iterations-base = Итерации (база)
panel-iterations = Итерации
panel-iterations-hover = Базовое число итераций. Когда включён автомасштаб, фактическое число растёт с глубиной приближения. Пока вы двигаетесь, предпросмотр ради отзывчивости ограничивает итерации низким значением (50 000), а когда вид устоится, повышает резкость до полного числа, — поэтому глубокие края во время движения выглядят гладкими и прорисовываются, когда вы останавливаетесь.
# A warning under the iteration count. $count: the iterations the settled preview renders.
panel-res-limited = На этой глубине готовый предпросмотр ради отзывчивости рендерится в пониженном разрешении (итераций: { $count }). Для полной детализации экспортируйте изображение.
panel-anti-alias = Сглаживание
# Anti-aliasing off (the other choices are 2×, 3×, 4×, 8×).
panel-anti-alias-off = Выкл.
panel-anti-alias-hover = Суперсэмплинг для неподвижных изображений (применяется, когда вид устоится). Чем выше значение, тем лучше укрощается мелкая внешняя «пыль» — ценой времени рендеринга.

## The control panel: Effects

panel-relief-lighting = Рельефное 3D-освещение
panel-relief-lighting-hover = Затенять поверхность по наклону оценки расстояния — выпуклый, освещённый вид. (Голоморфные семейства: Mandelbrot / Multibrot.)
panel-light-angle = Угол света
panel-relief = Рельеф
panel-relief-hover = Меньше — резче рельеф; больше — мягче/площе.
panel-rotate-light = Вращать свет
panel-rotate-light-hover = Вращать направление света со временем (использует ползунок «Скорость»).
panel-distance-glow = Свечение по расстоянию
panel-distance-glow-hover = Яркие контурные полосы по оценке расстояния, сгущающиеся в светящиеся нити у границы. (Голоморфные семейства.)
panel-glow = Свечение
panel-band-width = Ширина полос
panel-band-width-hover = Интервал между контурами расстояния (октав на полосу).
panel-animate-glow = Анимировать свечение
panel-animate-glow-hover = Заставить полосы свечения течь со временем (использует ползунок «Скорость»).

## The control panel: Overlays

panel-orbits-hover = Рисовать путь итераций точки под курсором.
panel-orbit-anim = Анимация (бегущая точка)
panel-orbit-anim-hover = Пустить по орбите бегущую точку с переливающимся цветом.
panel-orbit-speed = Скорость орбиты
panel-minimap-hover = Небольшой обзор исходного вида с меткой «вы здесь» и глубиной приближения. Щёлкните по нему, чтобы перейти к области.

## The control panel: Advanced
## Labels before a control sit in a column about 104 px wide; most of these are already wider in English.

# Small headings of the section's two groups.
panel-accelerators = Ускорители
panel-performance-tuning = Настройка производительности
panel-bla = Ускорение BLA (глуб. приближение)
panel-bla-hover = Билинейная аппроксимация: пропуск итераций по всей орбите на экстремальной глубине (floatexp Mandelbrot, ≥1e28×) — рендеринг на GPU примерно в 5 раз быстрее при идентичном результате (проверено самотестом). По умолчанию включено; выключите для сравнения или если встретите артефакт.
panel-series-approx = Аппроксимация рядами
panel-series-approx-hover = Начинать метод возмущений с многочлена, чтобы пропустить ранние итерации там, где BLA не активна (глубины df32, Multibrot, BLA выключена). Идентичный результат; выключите для сравнения.
panel-glitch-correct = Исправление глитчей (экспорт)
panel-glitch-correct-hover = Исправление глитчей с несколькими опорными точками для экспортируемых изображений: обнаруживает глитчи метода возмущений и перерисовывает эти пиксели относительно дополнительных опорных точек, пока они не станут чистыми. По умолчанию включено. Работает для экспортов до ~32 MP (без вспомогательных методов раскраски); более крупные изображения и живой вид используют простой путь. Никогда не используется для видов Mandelbrot и Multibrot (вне режима Жюлиа): при проверке по точному вычислению оно там ничего не исправило, а время экспорта увеличивало до 5 раз.
panel-dive-limit = Предел погружения автоприближения
panel-dive-limit-hover = Глубина, на которой останавливается автоприближение (клавиша A). До ~1e271× оно скользит плавно; глубже переключается на рывковое ступенчатое погружение, чтобы быстро достичь экстремальной глубины.
# Followed by two choices, Speed and Quality.
panel-autopilot-priority = Приоритет автоприближения
panel-priority-speed-hover = Приближаться с заданной скоростью. Экран обновляется, как только готов кадр с деталями, в таком разрешении, которое не прерывает поток обновлений (ниже порога «Мин. разрешение в движении», если приближение обгоняет рендеринг); между обновлениями последний кадр следует за приближением и размывается.
panel-priority-quality-hover = Каждое обновление — полностью проработанный кадр в полном разрешении, а приближение при необходимости замедляется, чтобы кадр на экране никогда сильно не увеличивался до прихода следующего. Более медленное погружение, чёткое в каждый момент.
# Followed by two choices, Detail and Misiurewicz point.
panel-autopilot-target = Цель автоприближения
panel-target-detail-hover = Погружаться к самой богатой структуре на краю фрактала в поле зрения, выбирая цель заново по ходу погружения.
panel-target-misiurewicz-hover = Найти ближайшую точку Мисюревича (центр спирали, структура которой повторяется бесконечно) и погрузиться точно в неё. Сначала погружение стоит на месте, пока точка вычисляется до глубины предела погружения, — мгновение на обычных глубинах и дольше после 1e1000×. Только множество Мандельброта.
panel-work-budget = Бюджет живого рендеринга
panel-work-budget-hover = Детали против скорости для живого вида на глубоком приближении. Чем выше, тем полнее разрешение рендеринга (чётче — меньше «мягкого» вида после масштабирования), но тем ниже частота кадров, а очень высокие значения рискуют вызвать кратковременную остановку GPU на самых тяжёлых кадрах. Экспорт всегда выполняется в полном разрешении, независимо от этого.
panel-min-motion-res = Мин. разрешение в движении
panel-min-motion-res-hover = Наименьшее разрешение, до которого живой вид может опускаться при непрерывном приближении или сдвиге на глубоком приближении. Чем выше, тем меньше быстрое погружение становится блочным/пиксельным (резче), ценой частоты кадров — при 100% обновления деталей глубокого погружения должны рендериться по полной стоимости, из-за чего глубокое приближение может заметно дёргаться. Если глубокое погружение кажется рывковым, уменьшите это значение. По умолчанию 30%; не влияет на готовое изображение и экспорт.
panel-finish-sound = Звук по окончании рендеринга
panel-finish-sound-hover = Проигрывать мелодию завершения FRACTINT, когда заканчивается экспорт или рендеринг тура: три восходящие ноты (1047, 1109, 1175 Hz по 100 ms каждая), в точности как их закодировал оригинал для DOS, — синтезированные как прямоугольный сигнал с ограниченной полосой и пропущенные через фильтр верхних частот, чтобы звучать как маленький динамик PC, который их играл.
panel-prefer-detail = Предпочитать детали при приближении
panel-prefer-detail-hover = Во время приближения или сдвига продолжать показывать последний полностью детализированный кадр — масштабированный и следующий за движением, — вместо того чтобы каждый кадр заново рендерить в пониженном качестве. Вид рендерится полностью, когда движение приостанавливается. Выключено: движение рендерится вживую в пониженном разрешении (плавнее, грубее). Неглубокие виды (прямой режим) всегда рендерятся вживую — они в любом случае дёшевы и резки в каждом кадре.

## The control panel: About (the current fractal)

# A link to the fractal's reference page, followed by an arrow.
panel-about-reference = Справочная страница

## The control panel: Advanced ▸ Second graphics card

# A label before a drop-down (wider than the panel's label column already).
gpu2-label = Вторая видеокарта
gpu2-off = Выкл.
# Another device on the same graphics card as the window (the default).
gpu2-same-card = Та же карта
# A card the list does not name. $number: the card's number.
gpu2-card-n = Карта { $number }
gpu2-finding = Поиск видеокарт этого компьютера…
gpu2-none = На этом компьютере нет другой видеокарты.
gpu2-hover =
    Другое графическое устройство рендерит кадры глубокого вида во время движения и часть дополнительных отсчётов, повышающих резкость после остановки, поэтому картинка поспевает за приближением и быстрее достигает итогового качества. Видеокарта, рисующая окно, по-прежнему рисует всё.

    Другая карта добавляет собственную скорость. Две разные модели карт рисуют несколько пикселей чуть по-разному; если вторая карта откажет, вид продолжит работу с одной.

    «Та же карта» (по умолчанию) открывает второе устройство на этой карте. Скорости это не добавляет, но рендеринг идёт без ожидания кадров экрана. Карта при этом общая: сбой, сбрасывающий карту, завершает программу так же, как сбой рендеринга самого окна, и при следующем запуске будет предложено выключить эту функцию.

## The view (canvas)

# A notice after Ctrl+click pins the Julia c at a deep zoom.
canvas-julia-pin-quantized = c Жюлиа закреплено — но оно хранится с двойной точностью, поэтому на этой глубине вся панель округляется до одного значения, и закрепление попадает в центр вида. Отдалитесь, чтобы прицелиться.
# A small caption in the top-left corner of the view while the orbit overlay is fitted to the view (its "Normalize (fit to view)" option).
canvas-orbit-normalized = орбита (нормализована)
# The minimap's tooltip.
canvas-minimap-hover = Обзор / вы здесь. Перетаскивайте, чтобы сдвигать при текущем масштабе, прокручивайте, чтобы приближать и отдалять, щёлкните, чтобы перейти к этой области (исходный масштаб).

## The timestamp overlay (View ▸ Show timestamp)

# Under the large elapsed-time clock. $frame: the frame number (as the log writes it, without grouping).
overlay-timestamp-frame = кадр { $frame }

## File ▸ Settings ▸ Reference cache…

cache-title = Кэш опорных орбит
cache-intro = На экстремальной глубине медленная часть вида — его опорная орбита: до часа арифметики произвольной точности. Fractadyne хранит здесь готовые орбиты, поэтому возвращение к месту, где вы уже были, или приближение где-то рядом с ним занимает вместо этого секунды. Одна орбита — около 4 MB.
cache-off = Выключен для этого запуска (вызов задачи или --no-orbit-cache). Ничего не читается и не записывается до следующего обычного запуска.
# Small headings of the window's parts.
cache-location = Расположение
cache-usage = Использование
cache-limit = Лимит
cache-no-config = (папка конфигурации недоступна)
cache-open-folder = Открыть папку
# The text on the usage bar. $used and $limit: sizes such as "1.2 GB"; $count: the orbits kept.
cache-usage-bar =
    { $count ->
        [one] { $used } из { $limit } — { $count } орбита
        [few] { $used } из { $limit } — { $count } орбиты
        [many] { $used } из { $limit } — { $count } орбит
       *[other] { $used } из { $limit } — { $count } орбиты
    }
cache-limit-hover = Когда кэш превышает этот размер, первыми удаляются орбиты, которые было ДЕШЕВЛЕ всего построить, — глубокая орбита, на которую ушёл час, переживёт сколько угодно быстрых, как бы недавно те ни использовались.
# Beside the limit. $count: about how many orbits of the live view's length fit in it.
cache-limit-estimate =
    { $count ->
        [one] ≈ { $count } орбита предельной длины живого вида
        [few] ≈ { $count } орбиты предельной длины живого вида
        [many] ≈ { $count } орбит предельной длины живого вида
       *[other] ≈ { $count } орбиты предельной длины живого вида
    }
cache-kept = Хранятся (сначала самые ценные)
# Column headers of the list of kept orbits.
cache-col-iterations = итерации
cache-col-precision = точность
cache-col-size = размер
cache-col-last-used = использована
# The precision column. $bits: the precision in bits.
cache-row-bits = { $bits } бит
# Under the list. $count: the orbits not listed.
cache-rows-more = … и ещё { $count }
# When an orbit was last used (the "last used" column).
cache-ago-now = только что
# $count: minutes.
cache-ago-minutes = { $count } мин назад
# $count: hours.
cache-ago-hours = { $count } ч назад
# $count: days.
cache-ago-days = { $count } дн. назад
# Asked before clearing. $count: the orbits; $size: their total size, such as "1.2 GB".
cache-confirm =
    { $count ->
        [one] Удалить { $count } орбиту ({ $size })? Её восстановление стоит времени, а не данных.
        [few] Удалить { $count } орбиты ({ $size })? Их восстановление стоит времени, а не данных.
        [many] Удалить { $count } орбит ({ $size })? Их восстановление стоит времени, а не данных.
       *[other] Удалить { $count } орбиты ({ $size })? Их восстановление стоит времени, а не данных.
    }
cache-delete = Удалить
cache-cancel = Отмена
cache-clear = Очистить кэш…
cache-clear-hover = Удаляет все сохранённые орбиты. Сначала спрашивает.
cache-close = Закрыть
# $count: the orbits removed.
cache-cleared =
    { $count ->
        [one] Кэш опорных орбит очищен — удалена { $count } орбита.
        [few] Кэш опорных орбит очищен — удалены { $count } орбиты.
        [many] Кэш опорных орбит очищен — удалено { $count } орбит.
       *[other] Кэш опорных орбит очищен — удалено { $count } орбиты.
    }
# $why: the system's error message.
cache-clear-failed = Не удалось очистить кэш опорных орбит: { $why }

## Help ▸ Diagnostics…

diag-title = Диагностика
diag-intro = Запустите тест, чтобы проверить, что Fractadyne правильно работает на вашем оборудовании. Результаты сохраняются на вашем компьютере и могут быть приложены к сообщению о проблеме — сами по себе они никуда не отправляются.
# $status: which arbitrary-precision library is in use, as the program reports it (in English).
diag-arithmetic = Арифметика глубокого приближения: { $status }
# The three tests' buttons, each followed by a sentence saying what it does.
diag-run-selftest = Запустить самотест
diag-run-uitest = Запустить тест интерфейса
diag-run-gputest = Запустить проверку арифметики GPU
diag-blurb-selftest = Сверяет математику и рендеринг на вашем GPU с заведомо правильными результатами. Занимает около 15 секунд.
diag-blurb-uitest = Проходит по интерфейсу и живому виду, делая снимки экрана на нескольких глубинах приближения. Занимает минуту-две и во время работы открывает окна.
diag-blurb-gputest = Сверяет арифметику double-float в шейдере с заведомо правильными значениями на каждом графическом бэкенде, который предлагает ваш компьютер. Занимает несколько секунд, окон не открывает. Если она сообщит о сбое, это настоящая находка, которую мы хотели бы увидеть, — пожалуйста, приложите её к отчёту.
# While a test runs. $count: the checks finished so far.
diag-checks-done =
    { $count ->
        [one] выполнена { $count } проверка
        [few] выполнены { $count } проверки
        [many] выполнено { $count } проверок
       *[other] выполнено { $count } проверки
    }
# A finished test's result. $test: the test's name (Self-test, UI test, GPU arithmetic check).
diag-verdict-captured = { $test } — результат получен
diag-verdict-passed = { $test } — пройдено
diag-verdict-problems = { $test } — обнаружены проблемы
# Under the GPU arithmetic check's result.
diag-informational-note = У этой проверки нет результата «пройдено/не пройдено»: она записывает, как считает ваш GPU, и сообщение о сбое — это настоящая находка, а не неисправность вашего компьютера. В любом случае самое полезное, что можно с ней сделать, — приложить её к отчёту.
# Under a test's result when it reported problems.
diag-problems-note = На оборудовании, отличном от эталонного GPU, некоторые различия ожидаемы — особенно при сравнении изображений. Самое полезное, что можно с этим сделать, — приложить результат к сообщению о проблеме.
diag-open-results = Открыть результаты
diag-attach = Приложить к сообщению о проблеме…
diag-attach-hover = Открывает диалог отчёта с этим результатом, чтобы сообщение о проблеме содержало проверенный машиной тест, а не только описание
diag-console = Выводить диагностику в консоль
diag-console-hover = Показывать строки [fd-…] в stderr с этого момента. По умолчанию выключено, когда Fractadyne запускается без аргументов. Это ничего не меняет в том, что ЗАПИСЫВАЕТСЯ, — файл журнала всегда получает каждую строку, — поэтому оставьте выключенным, если не следите за терминалом. Действует только в этом сеансе; чтобы захватить и запуск, используйте флаг --console или FRACTADYNE_CONSOLE=1.
diag-close = Закрыть
# Errors starting a test. $why: the system's error message; $dir: a folder.
diag-error-no-exe = Не удаётся найти исполняемый файл: { $why }
diag-error-create-dir = Не удаётся создать { $dir }: { $why }
diag-error-start = Не удалось запустить тест: { $why }

## Navigate ▸ Misiurewicz explorer…

misi-title = Обозреватель точек Мисюревича
misi-mandelbrot-only = Просмотр точек Мисюревича доступен только для Mandelbrot — переключите фрактал, чтобы им воспользоваться.
# While the gallery is being built. $seconds: the time so far, such as "3".
misi-solving = Вычисление семейств малых типов ({ $seconds }s)…
# Above the gallery. $total: all points; $curated: the hand-picked ones; $found: the ones the sweep found.
misi-summary = Точек: { $total } — отобранных вручную: { $curated }, найденных затравочным перебором методом Ньютона по малым типам: { $found } (верхняя полуплоскость; грубая сетка находит большинство бассейнов, но не доказуемо все).
# The name of a point the sweep found. $kind: its type, such as "(3,1)"; $point: its coordinate, such as "-0.1011+0.9563i".
misi-entry-name = { $kind } в { $point }
# Under each thumbnail. $kind: the point's type, such as "(3,1)"; $octaves: the zoom period in octaves, such as "3.17"; $twist: the twist per repeat in degrees, such as "+12.5". "oct" is short for octaves.
misi-entry-readout = { $kind } · { $octaves } окт./повтор · { $twist }°
# Before the depth field.
misi-zoom-to = Приблизить до
misi-cancel = Отмена
misi-solve-jump = Вычислить и перейти
misi-solve-jump-hover = Вычислить эту точку методом Ньютона с точностью, которая нужна для запрошенной глубины (взяв в качестве затравки её сохранённую координату), а затем сразу перейти туда. Глубокие запросы занимают больше времени — вычисление ведётся с произвольной точностью.
misi-select-hint = Выберите точку, укажите глубину и перейдите — строка мультипликатора под каждой миниатюрой показывает, как будет выглядеть погружение туда: через сколько октав вид повторяется и насколько он закручивается за каждый повтор.
misi-cancelled = Вычисление отменено.
# $text: what was typed in the depth field.
misi-bad-zoom = Не удаётся прочитать «{ $text }» как увеличение (пишите 1e50, 3.2e120, …).
misi-no-kp = У этой записи нет найденных (k,p), по которым можно вычислять.
misi-parse-failed = Не удалось разобрать сохранённую координату.
# A notice after the jump. $name: the point's name; $zoom: the magnification, such as "1e50"; $seconds: how long the solve took, such as "1.2".
misi-zoomed = Переход выполнен: { $name } — { $zoom }× ({ $seconds }s)
# Added after the notice above when the iteration count is too low. $have: the fixed iteration count; $typical: what the depth typically needs.
misi-iter-note = «Итерации» зафиксированы на { $have }; этой глубине обычно нужно ~{ $typical } — увеличьте их (или включите автоитерации), если вид отрисовывается плоским.
# $name: the point's name; $why: why the solve failed (a technical word).
misi-solve-missed = Глубокое вычисление не попало в точку { $name }: { $why }. Сохранённая точка по-прежнему точна на своей собственной глубине — попробуйте цель помельче.

## Life (Game of Life): the side panel

# Shown when the graphics card cannot run the Life view.
life-no-compute = Этот графический адаптер не может запустить Life (нет вычислительных шейдеров).
# Row label: the pattern list (pattern names stay in English).
life-pattern = Узор
life-open = Открыть…
life-open-hover = Открыть файл узора RLE, .cells или Life 1.05/1.06.
life-pattern-text = Текст узора…
life-pattern-text-hover = Вставить узор (RLE в том виде, как его даёт LifeWiki, или простой текст).
life-save = Сохранить…
life-save-hover = Сохранить вселенную в её нынешнем состоянии в формате RLE.
# Row label: the rule list and the typed rule (B3/S23 notation).
life-rule = Правило
# Shown in the rule list when the rule in use is none of the library's.
life-rule-custom = своё
life-apply = Применить
# Playback buttons (a pause or play symbol is shown before them).
life-pause = Пауза
life-play = Пуск
life-play-hover = Запустить вселенную (Enter).
life-step = Шаг
life-step-hover = Продвинуться на одно поколение (N).
# Tooltip of the button beside Step, which reads "+16", "+1,024"…: that many generations.
life-stride-hover = Продвинуться на столько поколений (Shift+N).
life-reset = Сбросить
life-reset-hover = Вернуться к узору в том виде, в каком он был открыт (поколение 0).
life-speed = Скорость
# After the speed slider's number: generations a second. Short.
life-speed-unit = пок./с
# Row label: how many generations the "+n" button advances.
life-stride = Прыжок
# The panel's numbers.
life-generation = Поколение
life-population = Население
life-tiles = Тайлы
# $count: the tiles stored; $capacity: how many the GPU pool holds.
life-tiles-of = { $count } из { $capacity }
life-tiles-hover = Хранимые тайлы по 64×64 клетки (по 4 KiB) и ёмкость пула GPU.
# Row label: what a left-drag in the view does (Pan or Draw).
life-mouse = Мышь
life-pan = Сдвиг
life-pan-hover = Перетаскивайте, чтобы сдвигать вид.
life-draw = Рисование
# The Draw tool's tooltip; the toolbar's shows it after "Draw (D):", so it starts lowercase.
life-draw-hint = щёлкните по клетке, чтобы переключить её; перетаскивайте, чтобы рисовать, — штрих, начатый на мёртвой клетке, рисует, а начатый на живой — стирает. Работающая вселенная приостанавливается, пока вы рисуете, и продолжает работу, когда вы отпускаете кнопку.
life-undo = Отменить
life-undo-hover = Вернуться к вселенной до последней правки, заливки, очистки или смены правила (Ctrl+Z во время рисования).
life-grid = Линии сетки
life-grid-hover = Границы клеток при приближении до 8 пикселей на клетку и больше.
# A random fill: "Random" labels the density field, "seed" the seed field after it.
life-random = Случайно
life-density-hover = Доля клеток, которые заливка делает живыми.
life-seed = зерно
life-fill-view = Заполнить вид
# $cells: the largest side of the filled area, in cells (2,048).
life-fill-view-hover = Заполнить видимую область (до { $cells } × { $cells } клеток вокруг её центра) случайным образом.
life-clear = Очистить
life-clear-hover = Убить все клетки («Отменить» вернёт их).

## Life: notices in the panel, and file dialogs

# $path: the file written.
life-saved = Сохранено: { $path }
# $error: why (from the system, in English).
life-save-failed = Не удалось сохранить: { $error }
# The universe stopped running. $error: why (in English).
life-stopped = Остановлено: { $error }
# A pattern file named a rule that does not read. $error: why (in English); $rule: the rule kept, in B3/S23 notation.
life-rule-not-understood = Правило узора не распознано ({ $error }); оставлено { $rule }
# $path: the file; $error: why (in English).
life-open-failed = Не удалось открыть { $path }: { $error }
# File-dialog filter names.
life-filter-patterns = Узоры Life
life-filter-rle = Узор с кодированием длин серий (RLE)

## Life: the toolbar (its buttons are icons; these are their tooltips)

life-toolbar-pause-hover = Приостановить вселенную (Enter)
life-toolbar-play-hover = Запустить вселенную (Enter)
# $stride: the generations a Shift+click steps.
life-toolbar-step-hover = Шаг на одно поколение (N); Shift+щелчок или Shift+N — шаг на { $stride }
life-toolbar-reset-hover = Вернуться к узору в том виде, в каком он был открыт (поколение 0)
# $hint: the Draw tool's tooltip (life-draw-hint).
life-toolbar-draw-hover = Рисование (D): { $hint }

## Life: the Pattern text window

life-text-title = Текст узора
life-text-intro = Вставьте узор: RLE (как его дают LifeWiki и Golly), простой текст .cells или Life 1.05 / 1.06.
life-text-load = Загрузить
life-text-current = Текущий в RLE
life-text-current-hover = Заполнить поле вселенной в том виде, в каком она была загружена последней.

## L-systems: the side panel

# Row label: the system list (system names stay in English).
lsys-system = Система
lsys-open = Открыть…
lsys-open-hover = Открыть файл Fractint .l или систему .lsys.
lsys-edit = Изменить…
lsys-edit-hover = Текст системы: аксиома, правила, угол — изменить и применить.
lsys-save = Сохранить…
lsys-save-hover = Сохранить систему как текстовый файл .lsys.
# Tooltip of the "SVG…" button.
lsys-svg-hover = Экспортировать вид как рисунок SVG: его линии и фигуры — векторами, в его цветах.
lsys-follow = Порядок следует за приближением
lsys-follow-hover = Порядок растёт по мере приближения, поэтому детали никогда не заканчиваются.
lsys-follow-fixed-hover = Эта система не растёт в одно и то же число раз с каждым порядком, поэтому рисуется с фиксированным порядком.
lsys-order = Порядок
lsys-angle = Угол
lsys-sweep-hover = Качать угол туда и обратно: кривая преображается, поворачиваясь
# A small button that puts back the system's own angle after the slider moved it.
lsys-system-angle = Угол системы
lsys-seed = Зерно
lsys-seed-hover = От чего зависят случайные выборы: то же зерно — то же растение.
lsys-seed-new = Новое
lsys-seed-new-hover = Другое растение: новое зерно.
lsys-line-width = Толщина линий
# Row label of a percentage slider: how much of the curve is drawn.
lsys-draw-on = Прорисовка
lsys-draw-on-hover = Какая часть кривой нарисована, в том порядке, в каком её рисует черепаха
lsys-draw-on-play-hover = Кривая рисует сама себя
lsys-colour-by = Цвет по
# The colourings, in the "Colour by" list.
lsys-colour-position = положению вдоль кривой
lsys-colour-depth = глубине ветвления
lsys-colour-heading = направлению
# C, < and > are the system's commands that set the colour index: keep them.
lsys-colour-index = индексу цвета (C, <, >)
lsys-colour-plain = один цвет
# The panel's numbers.
lsys-growth = Рост
lsys-growth-built = построено как слово на своём порядке
lsys-growth-built-hover = Параметрическая или контекстно-зависимая система: перезапись модуля зависит от его чисел или соседей, поэтому слово строится по одному поколению за раз (до двух миллионов модулей) и рисуется с двойной точностью — порядок не следует за приближением.
# How much the picture grows from one order to the next. $factor: the factor (1.4142).
lsys-growth-factor = ×{ $factor } за порядок
lsys-growth-factor-period-2 = ×{ $factor } за порядок (порядки идут через 2)
lsys-growth-none = нет (фиксированный порядок)
lsys-drawn = Нарисовано
# $drawn: what was drawn (lsys-drawn-segments, lsys-drawn-shapes or lsys-drawn-segments-shapes); $order: the order.
lsys-drawn-at-order = { $drawn } на порядке { $order }
# The same, when the walk stopped at its segment budget.
lsys-drawn-at-order-budget = { $drawn } на порядке { $order } (бюджет)
# $segments: the line segments drawn.
lsys-drawn-segments =
    { $segments ->
        [one] { $segments } отрезок
        [few] { $segments } отрезка
        [many] { $segments } отрезков
       *[other] { $segments } отрезка
    }
# $shapes: the filled shapes drawn.
lsys-drawn-shapes =
    { $shapes ->
        [one] { $shapes } закрашенная фигура
        [few] { $shapes } закрашенные фигуры
        [many] { $shapes } закрашенных фигур
       *[other] { $shapes } закрашенной фигуры
    }
# Both counts. $segments: line segments; $shapes: filled shapes.
lsys-drawn-segments-shapes =
    { $segments ->
        [one] { $segments } отрезок
        [few] { $segments } отрезка
        [many] { $segments } отрезков
       *[other] { $segments } отрезка
    }, { $shapes ->
        [one] { $shapes } закрашенная фигура
        [few] { $shapes } закрашенные фигуры
        [many] { $shapes } закрашенных фигур
       *[other] { $shapes } закрашенной фигуры
    }
# $ms: the walk's time in milliseconds (12.3).
lsys-drawn-hover = обход занял { $ms } ms; поддеревья за пределами вида пропускаются, а меньшие пикселя рисуются одним отрезком
# A word too long to build at the order asked for. $wanted: the order asked for; $built: the order drawn.
lsys-drawn-short = Слово порядка { $wanted } длиннее двух миллионов модулей: нарисовано на порядке { $built }.
# $order: the deepest order the app draws.
lsys-deepest-order = На самом глубоком рисуемом порядке ({ $order }) дальнейшее приближение не добавляет деталей.

## L-systems: notices in the panel and the editor, and file dialogs

# $path: the file; $error: why (in English).
lsys-open-failed = Не удалось открыть { $path }: { $error }
# Why a file was not opened (shown after "Could not open …:").
lsys-file-too-large = больше 4 MiB
# Why a Fractint .l file was not opened: it holds no entries. `Name { … }` is the file's syntax: keep it.
lsys-no-entries = нет записей (файл .l содержит записи вида `Name {"{"} … {"}"}`)
# $path: the file written.
lsys-saved = Сохранено: { $path }
# $error: why (from the system, in English).
lsys-save-failed = Не удалось сохранить: { $error }
# After File ▸ Export SVG. $path: the file written.
lsys-svg-exported = Экспортировано: { $path }
# $error: why (from the system, in English).
lsys-svg-failed = Ошибка экспорта: { $error }
# File-dialog filter names: opening (any system file) and saving (one system).
lsys-filter-systems = L-системы
lsys-filter-system = L-система

## L-systems: the toolbar

lsys-toolbar-lower-hover = Понизить порядок (фиксирует его)
lsys-toolbar-raise-hover = Повысить порядок (фиксирует его)
# Between the − and + buttons, in a fixed-width font. $order: the order, padded to 3 characters so the label keeps its width.
lsys-toolbar-order = порядок { $order }
# A toggle: the order follows the zoom. Short.
lsys-toolbar-auto = авто
lsys-toolbar-auto-hover = Порядок следует за приближением
lsys-toolbar-edit-hover = Изменить текст системы

## L-systems: the editor window, and a .l file's entries

lsys-editor-title = L-система
# The editor's help. The quoted examples ('X = word', 'seed n', 'define R 1.456'…), the commands and "The Algorithmic Beauty of Plants" (a book's title) are the system's syntax and names: keep them as they are.
lsys-editor-help = Один ключ на строку: angle (градусы или /n — доля окружности), axiom, правила 'X = word' и при желании heading, draw / move / variables, colour, order. Символ с альтернативами, выбираемыми случайно, занимает по строке на каждую из них, с её весом: 'X (0.3) = word'; 'seed n' выбирает растение. В слове {"{"} {"}"} заливает путь черепахи, а . добавляет вершину. Параметрические и контекстно-зависимые правила (The Algorithmic Beauty of Plants): 'A(s) : s > 1 = F(s)[+A(s/2)]', 'b < a > c = b', с 'define R 1.456' и 'ignore +-F'; F(l) делает шаг l, +(a) поворачивает на a градусов. Запись Fractint .l можно вставить как есть.
lsys-editor-apply = Применить
lsys-editor-revert = Вернуть
lsys-editor-revert-hover = Текст показанной системы.
# The window listing a .l file's entries to pick from. $file: the file's name.
lsys-entries-title = { $file } — записи
# Tooltip of an entry that does not parse (its error is shown beside it, in English).
lsys-entry-unreadable-hover = Эта запись не читается.

## Export: opening a view or location (File ▸ Open view or location…)

# File-dialog filter names.
export-filter-view = Вид или местоположение Fractadyne
export-filter-image = Изображение (PNG / EXR)
export-filter-location = Местоположение (.fdn / .kfr)
# $zoom: the magnification (1.2e148), followed by the magnification sign ×.
export-loaded-location = Местоположение загружено @ { $zoom }×
# $zoom: the magnification; $note: what was noteworthy in the file (in English).
export-loaded-location-note = Загружено @ { $zoom }× — { $note }
# $path: the file.
export-not-location = { $path } — не местоположение Fractadyne.
export-file-too-large = Файл слишком большой (это не местоположение .fdn?).
# $path: the file; $error: why (from the system, in English).
export-read-failed = Не удалось прочитать { $path }: { $error }
# A Kalles Fraktaler .kfr file did not import. $error: why (in English).
export-import-failed = Ошибка импорта: { $error }
# $path: an image whose embedded text is not a view.
export-not-view = В { $path } есть метаданные Fractadyne, но не вид.
# $path: the image the view was read from.
export-loaded-view = Вид загружен из { $path }
# $path: the image; $note: what was noteworthy in it (in English).
export-loaded-view-note = Вид загружен из { $path } — { $note }
export-no-metadata = В этом файле нет встроенных метаданных вида Fractadyne.

## Export: notices after an export or a snapshot
## "File → Export image…" in them names the menu items (menu-file, menu-file-export).

# The export window's status while the image renders.
export-rendering = Рендеринг…
export-canceled = Экспорт отменён.
# $error: why (in English).
export-failed = Экспорт не удался: { $error }
# $size: the image's size in pixels (1920×1080); $path: the file written.
export-saved = Сохранено { $size } → { $path }
# An L-system export whose walk stopped at its budget. $size: pixels; $segments: the budget (4,000,000); $path: the file.
export-saved-incomplete = Сохранено { $size } (неполное: превышен бюджет в { $segments } отрезков — понизьте порядок) → { $path }
# A two-panel export written as two files. $path: the first of them.
export-saved-two = Сохранено 2 файла → { $path }
# $size: pixels; $path: the file written.
export-saved-corrected = Сохранено { $size } (с исправлением глитчей) → { $path }
# $path: the first of the two files.
export-saved-two-corrected = Сохранено 2 файла (с исправлением глитчей) → { $path }
# Added after a finished export's notice. $duration: how long it took (8.3s, 1m 04.0s).
export-elapsed = (за { $duration })
export-preparing-deep = Подготовка глубокого экспорта — построение опорной орбиты (это может занять некоторое время)…
# $size: pixels (5120×4035); $ss: the supersampling factor; $iterations: the iteration count.
export-large = Большой экспорт ({ $size }, суперсэмплинг { $ss }×, итераций: { $iterations }) — рендеринг в фоне; при экспорте такого размера исправление глитчей пропускается…
export-life-unavailable = Экспорт изображения для вида Life пока недоступен — сделайте снимок экрана или используйте Файл ▸ Сохранить узор Life…
export-already-running = Экспорт уже выполняется — Файл → Экспортировать изображение… показывает его ход.
# $size: the image's size in pixels.
export-snapshot-background = Снимок: рендеринг { $size } в фоне — Файл → Экспортировать изображение… показывает ход работы и позволяет её отменить.
export-no-gpu = GPU недоступен
# A screen-capture snapshot. $size: pixels; $path: the file written.
export-snapshot-saved = Снимок экрана { $size } сохранён → { $path }
# $error: why (in English).
export-snapshot-failed = Снимок не удался: { $error }

## The colour picker (the popup of a colour swatch)

# The third row's label: the colour as hexadecimal bytes. Short (about 6 characters).
picker-row-hex = Hex
picker-hex-hover = Щёлкните, чтобы ввести шестнадцатеричное значение
picker-cancel = Отмена
picker-cancel-hover = Вернуть цвет, с которым открылась эта палитра выбора
picker-ok = ОК
picker-ok-hover = Оставить этот цвет
# Starts the screen eyedropper (an icon is shown before it).
picker-pick = Пипетка
picker-pick-hover = Взять цвет из любого места экрана
# Tooltip of the disabled Pick button (in the picker and the gradient editor) on Linux and macOS.
picker-eyedropper-unsupported = Чтобы взять цвет из другого места экрана, нужен API захвата экрана платформы. Он есть только в сборке для Windows; в Linux для этого потребовалась бы зависимость от X11, а Wayland требует портал с собственным запросом разрешения.

## The fractal families: what each one is (the info panel's "About" section and the pickers' hover text).
## The formulas themselves (z -> z^2 + c) are written in code and stay as they are.

fractal-mandelbrot-about = Каноничный фрактал времени убегания: множество тех c, для которых орбита 0 остаётся ограниченной. Его граница бесконечно сложна.
fractal-multibrot3-about = Множество Multibrot — построение Мандельброта в более высокой степени. Степень d даёт вращательную симметрию порядка d−1.
fractal-multibrot4-about = Multibrot в степени 4: симметрия третьего порядка, широкие почки.
fractal-multibrot5-about = Multibrot в степени 5: симметрия четвёртого порядка.
fractal-tricorn-about = Tricorn (Mandelbar): на каждом шаге z сопрягается. Это антиголоморфное отображение даёт фигуру с тремя углами.
fractal-burning-ship-about = Перед возведением в квадрат берутся модули частей z; результат напоминает горящий корабль.
fractal-celtic-about = Родственник Burning Ship, берущий модуль только действительной части z^2, что даёт мотивы кельтских узлов / сердец.
fractal-buffalo-about = Вариант с модулем, берущий модули обеих компонент z^2.
fractal-phoenix-about = Phoenix использует и предыдущую итерацию, что даёт пламеподобные нити. Попробуйте его форму Жюлиа в режиме Жюлиа.
fractal-newton-about = Итерация Ньютона для поиска корней z^3 = 1, раскрашенная по скорости сходимости каждой точки. Фрактал сходимости (а не убегания).
fractal-multibrot6-about = Multibrot в степени 6: симметрия пятого порядка.
fractal-multibrot7-about = Multibrot в степени 7: симметрия шестого порядка, почки истончаются к окружности.
fractal-multibrot8-about = Multibrot в степени 8: симметрия седьмого порядка.
fractal-burning-ship3-about = Burning Ship в степени 3: берутся модули, затем возводятся в куб.
fractal-burning-ship4-about = Burning Ship в степени 4.
fractal-burning-ship5-about = Burning Ship в степени 5.
fractal-tricorn3-about = Tricorn (Mandelbar) в степени 3: симметрия четвёртого порядка.
fractal-tricorn4-about = Tricorn в степени 4: симметрия пятого порядка.
fractal-tricorn5-about = Tricorn в степени 5: симметрия шестого порядка.
fractal-celtic3-about = Складка Celtic в степени 3: модуль действительной части z^3.
fractal-celtic4-about = Складка Celtic в степени 4.
fractal-celtic5-about = Складка Celtic в степени 5.
fractal-buffalo3-about = Buffalo в степени 3: модули обеих частей z^3.
fractal-buffalo4-about = Buffalo в степени 4.
fractal-buffalo5-about = Buffalo в степени 5.
fractal-custom-about = Формула, записанная выражениями в стиле Fractint (Фрактал ▸ Своя формула…). Большинство таких формул поддерживает глубокое приближение методом возмущений, как встроенные семейства; поддерживает ли его ваша, сообщает диалог формулы.
fractal-life-about = Игра «Жизнь» Конвея и её родственники на неограниченной плоскости: правила Generations, нетоталистические правила (Hensel), файлы узоров. Запускайте, шагайте, рисуйте.
fractal-lsystem-about = Системы Линденмайера: кривые, заполняющие пространство кривые, острова и растения, нарисованные черепахой по переписывающей грамматике. Порядок следует за приближением, поэтому детали никогда не заканчиваются. Файлы Fractint .l открываются напрямую.

# Shown where the other families show their formula (z -> z^2 + c).
fractal-custom-formula = z -> формула, которую вы пишете
# Shown where the other families show their formula. "B3/S23" is the rule's notation: keep it.
fractal-life-formula = B3/S23: рождается при 3 живых соседях, выживает при 2 или 3
# Shown where the other families show their formula. "F = F+F--F+F" is the grammar's notation: keep it.
fractal-lsystem-formula = F = F+F--F+F: каждый символ переписывается, затем всё рисует черепаха

# The last paragraph of the picker's hover text for a family that renders only on the direct path.
fractal-no-deep-zoom = Только прямой рендеринг — у этого семейства нет глубокого приближения.

## Names of settings, shown in the Settings menu and the welcome window.

theme-dark = Тёмная
theme-light = Светлая
# The update track that follows only stable releases.
update-track-stable = Стабильные
# The update track that also offers pre-releases (betas).
update-track-beta = Бета (предварительные выпуски)

## The status bar (its hover texts; the readouts follow in their own section)

status-period-hover = Период миниброта, к которому вы перешли (нажмите M или Перейти к местоположению ▸ Перейти к объекту). Он остаётся, пока вид находится на объекте, и исчезает, когда вы уходите.

## The playback transport: the player over the view while a tour plays.

# $close: the close button's icon, as drawn on that button.
play-finished-hover = Завершено — перемотайте назад или нажмите { $close }, чтобы закрыть
# $kind: "benchmark" or "script" (the variants hold the wording); $name: the tour's name.
play-playing-hover =
    { $kind ->
        [benchmark] Воспроизводится тест «{ $name }»
       *[other] Воспроизводится сценарий «{ $name }»
    }
play-restart = Начать сначала
play-back = Назад на 10 секунд
play-again = Воспроизвести снова с начала
play-resume = Продолжить
play-pause = Пауза
play-stop = Остановить и перемотать в начало
play-forward = Вперёд на 10 секунд
play-speed-hover = Скорость воспроизведения (щёлкайте, чтобы переключать 0.5× / 1× / 2× / 4×)
play-render = Отрендерить этот сценарий в последовательность кадров…
play-render-none = У встроенного теста производительности нет файла сценария для рендеринга
play-open = Воспроизвести другой сценарий…
play-loop = Повторять тур, когда он доходит до конца
play-close = Закрыть проигрыватель и восстановить ваши собственные настройки вида
# A notice beside the scrub bar while the renderer holds the tour's clock. Keep it short.
play-waiting = ожидание деталей
# "[playback] pace" is a setting in the script file: keep it as written.
play-waiting-hover = Часы тура приостановлены, пока рендерер прорисовывает этот вид (построение опорной орбиты / рост бюджета итераций). Воспроизведение возобновится само; см. [playback] pace в сценарии.
play-scrub = Перетаскивайте, чтобы перематывать тур
# $behind: how far the script clock is behind real time, as minutes:seconds (1:05).
play-scrub-behind =
    Перетаскивайте, чтобы перематывать тур
    Часы сценария отстают от реального времени на { $behind } (регулятор темпа замедлил тур, чтобы рендерер успевал). Верхняя отметка показывает, где было бы воспроизведение по реальному времени.

## The autopilot's notices (toasts). Where a notice has two lines, the first is the verdict and the second what to do about it.

autopilot-on-detail = Автопилот включён — погружение к деталям (любое действие останавливает)
autopilot-off = Автопилот выключен
autopilot-misi-needs-mandelbrot =
    Автопилот: для цели «Точка Мисюревича» нужно множество Мандельброта
    Вместо этого — погружение к деталям
autopilot-on-misi =
    Автопилот включён — поиск ближайшей точки Мисюревича…
    Любое действие останавливает
# $seconds: how long the search has run, in whole seconds.
autopilot-misi-finding =
    Автопилот: поиск ближайшей точки Мисюревича…
    { $seconds } s · любое действие останавливает
autopilot-misi-not-found =
    Автопилот: рядом с этим видом не найдено точки Мисюревича
    Поиск может не доставать до этой глубины — попробуйте цель «Детали»
# ($preperiod,$period) names the point, as mathematicians write it: keep the notation. $widths: a power of two (2^$widths screen widths).
autopilot-misi-off-screen =
    Автопилот: ближайшая точка Мисюревича ({ $preperiod },{ $period }) за пределами экрана
    На расстоянии ~2^{ $widths } ширин экрана — подойдите ближе к центру спирали
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-not-solved =
    Автопилот: точку Мисюревича ({ $preperiod },{ $period }) рядом с этим видом вычислить не удалось
    Попробуйте цель «Детали»
autopilot-misi-refused = Автопилот: вычисление точки Мисюревича отклонено (неверная пара)
autopilot-misi-stopped = Автопилот: решатель точек Мисюревича неожиданно остановился
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-diving =
    Автопилот: погружение в точку Мисюревича ({ $preperiod },{ $period })
    Любое действие останавливает
# ($preperiod,$period) names the point: keep the notation. $octaves: how often the structure repeats (2.37); $twist: its turn in degrees, signed (+12.5).
autopilot-misi-diving-repeat =
    Автопилот: погружение в точку Мисюревича ({ $preperiod },{ $period })
    Структура повторяется каждые { $octaves } октавы, закрутка { $twist }° · любое действие останавливает
# $limit: the magnification, as written (1e300).
autopilot-dive-limit = Автопилот: достигнут предел погружения (~{ $limit }×)
# $have: the fixed iteration count; $typical: what this depth typically needs.
autopilot-dead-end-short =
    Автопилот остановлен: больше не на что нацелиться — итерации зафиксированы на { $have }, а этой глубине обычно нужно ~{ $typical }
    Включите автоитерации или увеличьте число итераций
# $limit: the fixed iteration count.
autopilot-dead-end-cap =
    Автопилот остановлен: больше не на что нацелиться — всё в поле зрения доходит до фиксированного предела итераций { $limit }
    Включите автоитерации или увеличьте число итераций
autopilot-dead-end = Автопилот: впереди нет деталей (остановлен)

## Tours (camera scripts)

# The name of the file type in the file dialog's filter list.
script-file-filter = Сценарий Fractadyne (TOML)
# $name: the script's name.
script-finished = Сценарий завершён — «{ $name }»
# The title of the notice that follows (the file's path and the error, in English).
script-load-failed = Не удалось загрузить сценарий

## Installing a language file

# The name of the file type in the file dialog's filter list.
language-file-filter = Файл языка Fractadyne

## The notice when the graphics driver's shader cache stops keeping shaders. The text is shown in a fixed-width font with its line breaks as written: break the lines where they read well.

driver-cache-title = Графический драйвер не сохраняет шейдеры
# $count: how many shaders compiled again; $pipeline: the slowest one's name (a technical id: keep it); $seconds: its time in seconds (7.5).
driver-cache-body =
    Ваш графический драйвер в этом сеансе снова скомпилировал шейдеры
    Fractadyne (их { $count }), хотя уже компилировал каждый из них раньше
    (самый медленный: { $pipeline }, { $seconds } s). Его кэш шейдеров
    не сохраняет скомпилированные шейдеры — чаще всего потому, что кэш
    заполнен. Пока это не изменится, экспорт, глубокие виды и запуск
    Fractadyne могут каждый раз тратить секунды на компиляцию.
# $folder: the cache folder's path; $size: its size in MB (1024). "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia-folder =
    Как исправить: закройте Fractadyne, затем удалите содержимое папки
      { $folder }   ({ $size } MB)
    Драйвер заново создаст кэш. Чтобы он снова не переполнился, увеличьте
    Панель управления NVIDIA > Управление параметрами 3D > Размер кэша шейдеров.
# "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia =
    Как исправить: увеличьте Панель управления NVIDIA > Управление параметрами 3D >
    Размер кэша шейдеров или очистите кэш шейдеров драйвера.
# "AMD Software > Graphics > Advanced > Reset Shader Cache" names AMD's own settings: use the names AMD's software shows in this language.
driver-cache-fix-amd = Как исправить: AMD Software > Графика > Расширенные > Сбросить кэш шейдеров.
driver-cache-fix-other = Как исправить: очистите кэш шейдеров графического драйвера.

## Why a language file could not be installed: each ends the sentence "{ $file } is not a usable language file: …".

language-install-not-utf8 = это не текст в UTF-8
language-install-too-large = слишком большой
# "fractadyne-<language>.ftl" is the file name to use: keep it as written.
language-install-no-tag = в нём не указано, какой это язык (назовите его fractadyne-<language>.ftl)
language-install-english = английский встроен в программу
language-install-empty = он не переводит ни одного сообщения этой версии
language-install-no-folder = нет папки настроек

## The notice after a session that did not end cleanly, when its iteration setting was set aside.

# $base: the iteration count automatic iterations start from.
quarantine-setting-auto = автоматические итерации от базы { $base }
# $count: the fixed iteration count.
quarantine-setting-fixed =
    { $count ->
        [one] { $count } итерация
        [few] { $count } итерации
        [many] { $count } итераций
       *[other] { $count } итерации
    }
# $end: "device" when the session lost its graphics device, "unclean" otherwise (the variants hold the wording). $depth: the magnification, as written (1.2e148). $setting and $restored: an iteration setting (quarantine-setting-auto or -fixed).
quarantine-notice =
    { $end ->
        [device] Предыдущий сеанс потерял графическое устройство на { $depth }× при настройке «{ $setting }». Этот сеанс вместо неё запущен с настройкой «{ $restored }», чтобы не попасть сразу же обратно под ту же нагрузку.
       *[other] Предыдущий сеанс завершился некорректно на { $depth }× при настройке «{ $setting }». Этот сеанс вместо неё запущен с настройкой «{ $restored }», чтобы не попасть сразу же обратно под ту же нагрузку.
    }

## The second graphics card's status line, under its setting in the control panel.

gpu2-state-off = Выкл.: всё рендерит видеокарта этого окна.
gpu2-state-starting = Запуск…
# $name: the card's name (and its graphics API).
gpu2-state-running = Используется: { $name }
# $why: the error, as the graphics driver reports it.
gpu2-state-failed = Не удалось запустить: { $why }
# $name: the card's name.
gpu2-state-lost = Остановлено: устройство { $name } потеряно. Дальше работает одна карта; выберите вторую снова, чтобы повторить попытку.
# $state: one of the lines above. "--worker-gpu" is a command-line option: keep it.
gpu2-state-pinned = { $state } (задано через --worker-gpu)

## The custom-formula dialog: how deep the formula renders (one line under the formula).

formula-depth-deep = Глубокое приближение методом возмущений, с расширенным диапазоном после 1e28x, как у встроенных фракталов (каждый шаг итерируется: для своих формул пока нет ни аппроксимации рядами, ни BLA).
# $why: what the formula has that keeps it off the deep-zoom path (one of the formula-depth-why messages below).
formula-depth-direct = Прямой рендеринг ({ $why } — для этого пока нет формы глубокого приближения): резко до тех пор, пока один шаг одинарной точности по c не станет шире пикселя, — примерно до 1e4x–1e5x, а вдали от начала координат и того меньше.
formula-depth-why-init = раздел инициализации
formula-depth-why-bailout = собственная проверка выхода
formula-depth-why-variable = переменная, сохраняемая от шага к шагу
formula-depth-why-previous = предыдущая итерация
formula-depth-why-comparison = сравнение или блок if
# floor, ceil, trunc and round are functions of the formula language: keep them.
formula-depth-why-rounding = округление (floor, ceil, trunc, round)
# "maxit" is a word of the formula language: keep it.
formula-depth-why-maxit = maxit
formula-depth-why-power = степень с переменным показателем
formula-depth-why-perturbed = уже возмущённая программа

## Image and frame sizes: the presets described in words (the others are standard names: HD 720p, WUXGA, DCI 4K).

# $size: the pixel size (2048×2048).
size-preset-square = Квадрат — { $size }
# $ratio: the aspect ratio (9:16); $size: the pixel size.
size-preset-portrait = Портрет { $ratio } — { $size }
# $ratio: the aspect ratio (32:9); $size: the pixel size.
size-preset-super-ultrawide = Суперширокий { $ratio } — { $size }

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
fractal-family-custom = Своя
# Conway's Game of Life.
fractal-family-life = Life
fractal-family-lsystem = L-система
# A family at a power: "Multibrot 3", "Burning Ship 4". $family: a name above; $power: the power.
fractal-with-power = { $family } { $power }

## Names of choices: colouring methods, orbit traps, palette animation, snapshots, angle units.

color-method-smooth = Плавные итерации
color-method-stripe = Среднее полос
color-method-triangle = Неравенство треугольника
color-method-trap = Ловушка орбиты
color-method-distance = Оценка расстояния
color-method-decomposition = Декомпозиция
# The shape an orbit trap catches the orbit with.
trap-point = Точка
trap-cross = Крест
trap-circle = Окружность
# Palette animation: none.
palette-anim-off = Выкл.
palette-anim-forward = Вперёд
palette-anim-reverse = Назад
palette-anim-pingpong = Туда-обратно
# Palette animation: a new random gradient every so often.
palette-anim-random = Случайные градиенты
# What the Snapshot button does.
snapshot-mode-ask = Спрашивать каждый раз
snapshot-mode-screen = Захват экрана
snapshot-mode-render = Полный рендеринг
# The unit of an angle typed into Go to location's polar fields.
angle-degrees = градусы
angle-radians = радианы
angle-turns = обороты

## Report an issue: the choices. (The report itself is sent in English.)

issue-kind-crash = Сбой / зависание программы
issue-kind-rendering = Неправильный рендеринг
issue-kind-performance = Проблема с производительностью
issue-kind-ui = Проблема интерфейса / удобства
issue-kind-feature = Запрос функции
issue-kind-other = Другое
severity-low = Низкая
severity-medium = Средняя
severity-high = Высокая
severity-blocking = Блокирующая
repro-always = Всегда
repro-sometimes = Иногда
repro-once = Случилось один раз
repro-cannot = Не воспроизводится

## The benchmark's depths and the diagnostics' tests.

# $zoom: the dive's final magnification (1e32).
bench-depth-standard = Стандартное ({ $zoom }×)
# $zoom: the dive's final magnification (1e48).
bench-depth-ultra = Сверхглубокое ({ $zoom }×)
diag-test-selftest = Самотест
diag-test-uitest = Тест интерфейса
diag-test-gputest = Проверка арифметики GPU

## The status bar's readouts: each word is followed by a value in a slot of fixed width. Keep the words short (one word each).

status-center = центр
status-cursor = курсор
status-zoom = масштаб
# Short for iterations.
status-iter = итер
# The period of the minibrot jumped to.
status-period = период
# A Game of Life view's scale (followed by "1 px = 4 cells" or "8 px / cell").
status-scale = масштаб
# A Game of Life view's generation. Short.
status-generation = пок.
# A Game of Life view's population. Short.
status-population = насел.
# An L-system's order (how many times its rules were applied).
status-order = порядок
# How many segments and shapes an L-system drew.
status-drawn = нарис.
# $value: how many cells one pixel covers (4, 9.99e300). "px" is the unit symbol: keep it.
life-scale-cells = 1 px = { $value } кл.
# $value: how many pixels one cell covers (8). "px" is the unit symbol: keep it.
life-scale-pixels = { $value } px / клетка

## The status bar's limit diagnostic: a short label (after a warning sign), and what it means when hovered.

# The view is as deep as this location can be rendered. Short.
status-limit-depth = предел глубины
# Every pixel ran out of iterations. Short.
status-limit-exhausted = итер. исчерпаны
# Pixels are capped at the iteration budget. Short.
status-limit-capped = итер. на пределе
# $pixels: how many pixels wide one step of the coordinates is; $sharp_to: the magnification the view is sharp to (written out, 1,234,567).
status-limit-direct-detail = Этот вид рендерится напрямую на GPU с одинарной точностью, а здесь один шаг координат имеет ширину { $pixels } px: соседние пиксели вычисляют одну и ту же точку, поэтому изображение распадается на блоки. В этом месте оно резкое примерно до { $sharp_to }×. У этой формулы пока нет пути глубокого приближения (метода возмущений), поэтому дальнейшее приближение не показывает новых деталей.
# $samples: the GPU buffer's capacity in orbit samples; $iterations: the iterations pixels are limited to.
status-limit-depth-detail = Опорная орбита достигла предела буфера GPU (отсчётов: { $samples }) без убегания, поэтому число итераций пикселей здесь ограничено ~{ $iterations }, и более мелкие детали на этом виде вычислить нельзя — ни вживую, ни при экспорте. Это текущий предел глубины для этого семейства местоположений.
# $budget: the largest iteration budget this view can be given.
status-limit-exhausted-detail = Каждый пиксель всё равно исчерпал итерации при наибольшем бюджете, который можно дать этому виду ({ $budget }). Либо вид здесь находится внутри множества (на глубине это нормально), либо ему нужно больше итераций, чем программа сейчас может дать, — изображение в обоих случаях одинаковое, поэтому бюджет вернули к дешёвому.
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations" is the setting's name in the control panel's Quality section.
status-limit-capped-max = { $percent }% пикселей исчерпали итерации при бюджете { $budget }, а убегающие завершаются прямо у этого потолка — вид недостаточно проработан. «Итерации» уже на максимуме, так что этому виду нужно больше, чем программа сейчас может дать.
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations", "Controls" and "Quality" are the names of a setting, the control panel and its section.
status-limit-capped-raise = { $percent }% пикселей исчерпали итерации при бюджете { $budget }, а убегающие завершаются прямо у этого потолка — вид недостаточно проработан. Увеличьте «Итерации» (Управление ▸ Качество), чтобы проработать его.
