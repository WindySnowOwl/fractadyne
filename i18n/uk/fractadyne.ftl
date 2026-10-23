### Fractadyne user interface: Ukrainian (Українська).
### A draft: machine translation (Claude, by Anthropic) of the English file, 2026-10-23, not yet
### reviewed by a native speaker. Corrections are welcome as pull requests (see i18n/README.md).

## Language data: read by Fractadyne, not shown as messages.

# This language's own name, shown in the language menu.
language-name = Українська
# The same name in English, shown beside it in the menu.
language-english-name = Ukrainian
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
language-more = Інші мови…
# The Languages window's title.
language-window-title = Мови
# The Languages window's introduction.
language-window-intro = Інтерфейс Fractadyne іншою мовою. Мова — це один файл: він постачається з випуском, або ж установіть його тут. Координати, файли й командний рядок у кожній мові лишаються англійською.
# Column headers of the installed-language list.
language-col-language = Мова
language-col-translated = Перекладено
language-col-from = Джерело
# Where a language comes from.
language-source-builtin = вбудована
language-source-package = цей випуск
language-source-user = установлена
language-source-dev = робоча тека
# $percent: how much of the interface the file translates, 0–100.
language-coverage = { $percent }%
# $count: messages in the file that failed the checks and show in English instead.
language-dropped =
    { $count ->
        [one] { $count } повідомлення потребує перевірки
        [few] { $count } повідомлення потребують перевірки
        [many] { $count } повідомлень потребують перевірки
       *[other] { $count } повідомлення потребують перевірки
    }
# Buttons of the Languages window.
language-install = Установити файл мови…
language-install-hover = Виберіть файл мови Fractadyne (.ftl). Його буде перевірено, а потім скопійовано до теки мов у ваших налаштуваннях, де він лишатиметься після оновлень.
language-open-folder = Відкрити теку мов
language-reload = Перезавантажити
language-reload-hover = Прочитати файли мов заново (після того як ви змінили або додали файл).
language-use = Використати
# Notices after installing a file. $name: the language's own name; $file: the file's name.
language-installed = Мову { $name } установлено з { $file }.
# $file: the file's name; $why: what was wrong with it.
language-install-failed = { $file } не є придатним файлом мови: { $why }
# Shown when a language file cannot be loaded and English is used instead. $name: the language.
language-load-failed = Не вдалося завантажити мову { $name }, тому Fractadyne показано англійською: { $why }
# A language whose writing system needs a font this computer does not have (Japanese or Chinese without a system font). $name: the language, in its own name.
language-font-missing = На цьому компʼютері не знайдено шрифту, який може відобразити мову { $name }, тому цю мову неможливо показати. Установіть шрифт для мови { $name } (наприклад, мовний пакет самої системи) і виберіть її знову.
# Shown after a draft translation's name in the language menu and the Languages window. Short.
language-draft = чернетка
language-draft-hover = Чернетка: цей переклад ще не перевірив носій мови. Виправлення вітаються — у вигляді запиту на злиття (pull request) до файлу мови.
# The heading over the decimal-mark choice, in the language menu and the Languages window.
language-decimal-mark = Десятковий роздільник
language-decimal-mark-hover = Як записуються числа: час, розміри й кількості. Координати та збільшення завжди записуються з крапкою, бо саме так їх читають файли й інші програми.
# $sample: a number written with this language's own punctuation (1,234.5).
language-decimal-language = Як прийнято в мові ({ $sample })
# $sample: a number written with a decimal point (1,234.5).
language-decimal-point = Крапка ({ $sample })
# $sample: a number written with a decimal comma (1.234,5).
language-decimal-comma = Кома ({ $sample })

## Asked once when the system's language changes

# The window's title.
language-changed-title = Мова
# $new: the system's new language, in its own name.
language-changed-question = Мову вашої системи змінено на { $new }. Використовувати її у Fractadyne?
# Buttons. $name: a language's own name.
language-changed-use = Використовувати { $name }
language-changed-keep = Залишити { $name }

## The welcome window (first run)

welcome-title = Ласкаво просимо до Fractadyne
welcome-intro = Програма для глибокого наближення до множини Мандельброта та її родичів. Кілька елементів керування, щоб почати:
# The language row, at the top of the window.
welcome-language = Мова
# Controls: the key or gesture on the left, what it does on the right.
welcome-key-drag = перетягування
welcome-key-drag-does = пересунути вигляд
welcome-key-scroll = прокручування
welcome-key-scroll-does = наблизити / віддалити в бік курсора
welcome-key-space = утримання Space
welcome-key-space-does = плавне безперервне наближення (Shift+Space — віддалення)
# Shown after the click-zoom toolbar icon.
welcome-key-click = потім клацання
welcome-key-click-does = пірнути в точку, де клацнули (панель інструментів)
welcome-key-m-does = перейти до найближчого мініброта в його власному масштабі
welcome-landmarks = Для початку перейдіть до орієнтира:
welcome-theme = Тема
welcome-watermark = Показувати знак «Fd»
welcome-watermark-hover = Невеликий фірмовий знак у правому нижньому куті вигляду. Якщо вимкнути, його не буде ні в живому вигляді, ні в експортованих зображеннях.
welcome-updates = Оновлення
welcome-check-updates = Перевіряти оновлення
welcome-check-updates-hover = Під час запуску програма запитує в GitHub останній випуск. Нічого не надсилається.
welcome-files = Файли
welcome-no-config = (немає теки конфігурації)
welcome-open = Відкрити
welcome-rendering-on = Рендеринг на
welcome-open-help = Відкрити повну довідку  (F1)
# $icon: the Settings icon.
welcome-more-in-settings = Більше — у Файл → { $icon } Налаштування
welcome-get-started = Почати

## Menus: shared

# A percentage in a menu (UI scale). $value: the number.
common-percent = { $value }%
# A state shown after a menu item's name, in brackets.
menu-state-connected = підключено
menu-state-rendering = рендеринг
menu-state-running = працює

## Navigate ▸ Bookmarks (also the toolbar's star)

menu-bookmarks = Закладки
menu-bookmarks-add = Додати поточний вигляд
menu-bookmarks-manage = Керувати…

## The fractal picker (Fractal menu and the toolbar's dropdown)

menu-family-powers = Вищі степені
menu-family-automata = Автомати
menu-family-lsystems = L-системи
menu-fractal-custom = Власна формула…
menu-fractal-library = Бібліотека формул…
menu-fractal-library-hover = Збережені власні формули: застосовуйте, редагуйте, імпортуйте та експортуйте їх
# The "Show" group: which picture of the formula is drawn. $name: the formula's name (Mandelbrot, Burning Ship…).
menu-show = Показати
menu-show-set = Множина «{ $name }»
menu-show-set-hover = Площина параметрів — сама множина «{ $name }».

    Кожна її точка — окрема множина Жюліа; це карта їх усіх.
menu-show-julia = Множина Жюліа
menu-show-julia-hover = Одна точка множини «{ $name }», намальована як окреме зображення, — форма, яку ви отримуєте, зафіксувавши цю точку.

    Виберіть точку, навівши курсор на карту в режимі «Обидві, повʼязані»; клацання там її закріплює.
menu-show-julia-none = Формула «{ $name }» не має вільного параметра, тому не має й множини Жюліа.
menu-show-both = Обидві, повʼязані
menu-show-both-hover = Ліворуч — множина «{ $name }», а праворуч — множина Жюліа тієї точки, над якою перебуває ваш курсор.

    Ведіть курсор по карті й дивіться, як змінюється Жюліа, — це найшвидший спосіб побачити, як вони повʼязані.
menu-show-both-none = Формула «{ $name }» не має множини Жюліа, з якою її можна поєднати в пару.

## File menu

menu-file = Файл
menu-file-open-view = Відкрити вигляд або розташування…
menu-file-gallery = Галерея…
menu-file-export = Експортувати зображення…
menu-file-snapshot = Знімок
menu-file-open-life = Відкрити шаблон Life…
menu-file-open-life-hover = Файл RLE, .cells або Life 1.05/1.06, показаний як всесвіт Life.
menu-file-life-text = Текст шаблону Life…
menu-file-life-text-hover = Вставте шаблон як текст (RLE у тому вигляді, як його подає LifeWiki).
menu-file-save-life = Зберегти шаблон Life…
menu-file-save-life-hover = Всесвіт у його теперішньому стані, у форматі RLE.
menu-file-open-lsystem = Відкрити L-систему…
menu-file-open-lsystem-hover = Файл Fractint .l (з нього можна вибрати записи) або система .lsys.
menu-file-save-lsystem = Зберегти L-систему…
menu-file-save-lsystem-hover = Текст системи як файл .lsys.
menu-file-export-svg = Експортувати SVG…
menu-file-export-svg-hover = Вигляд як SVG-малюнок: його лінії та фігури — векторами, у його кольорах.
menu-file-render-client = Клієнт рендерингу…
menu-file-render-client-hover = Позичити GPU цього компʼютера рендер-фермі: підключитися до контролера (Інструменти ▸ Рендеринг на фермі… на іншому компʼютері) і рендерити для нього кадри.
menu-file-settings = Налаштування
menu-file-reset = Скинути стан програми…
menu-file-reset-hover = Видалити весь збережений стан (сеанс, закладки, мініатюри) і почати з чистого аркуша. Спершу просить підтвердження.
menu-file-quit = Вийти

## File ▸ Settings

menu-settings-fps = Обмеження частоти кадрів
menu-settings-fps-uncapped = Без обмеження
# $fps: frames per second.
menu-settings-fps-n = { $fps } FPS
menu-settings-ui-scale = Масштаб інтерфейсу (розмір шрифту)
menu-settings-theme = Тема
menu-settings-updates = Оновлення
menu-settings-updates-stable-hover = Останній стабільний випуск
menu-settings-updates-beta-hover = Остання збірка, включно з попередніми випусками
menu-settings-updates-on-launch = Перевіряти оновлення під час запуску
menu-settings-updates-on-launch-hover = Інакше перевіряйте вручну через Довідка → Перевірити оновлення.
# Followed by its shortcut, (Ctrl+S).
menu-settings-snapshot = Знімок
menu-settings-snapshot-ask-hover = Пропонувати вибір при кожному натисканні
menu-settings-snapshot-screen-hover = Зберегти вигляд точно таким, яким він є на екрані (миттєво, PNG)
menu-settings-snapshot-render-hover = Повний рендер із параметрами експорту, у фоновому режимі (на глибоких виглядах може тривати довго)
menu-settings-refcache = Кеш опорних орбіт…
menu-settings-refcache-hover = Опорні орбіти глибокого наближення, що зберігаються на диску, щоб місце, до якого ви повертаєтеся, рендерилося за секунди. Де він розташований, який має розмір, ліміт і спосіб його очистити.

## Fractal menu

menu-fractal = Фрактал

## View menu

menu-view = Вигляд
menu-view-zoom-home = Віддалити до повного вигляду
menu-view-zoom-home-hover = Віддалити до повного початкового вигляду (з анімацією)
menu-view-reset = Скинути до типового вигляду
menu-view-reset-hover = Скинути до типового вигляду фрактала (миттєво)
# Followed by its shortcuts.
menu-view-undo = Скасувати зміну вигляду
menu-view-redo = Повторити зміну вигляду
menu-view-control-panel = Панель керування
menu-view-control-panel-hover = Показати/сховати панель керування праворуч.
menu-view-minimap = Мінікарта
menu-view-minimap-hover = Показати невеликий огляд початкового вигляду з позначкою «ви тут» і глибиною наближення. Клацніть на ньому, щоб перейти до області.
menu-view-perf-panel = Панель продуктивності
menu-view-timestamp = Показувати позначку часу
menu-view-timestamp-hover = Показувати поверх вигляду великий годинник часу, що минув. Він показує ту саму позначку «+12.345s», якою журнал позначає кожен рядок, тож запис екрана з проблемою, що проявляється лише в русі, можна покадрово зіставити з журналом. Типово вимкнено.
menu-view-zoom-target = Показувати ціль автонаближення
menu-view-zoom-target-hover = Поки працює автонаближення, обводити область вигляду, до якої воно наближається, — частину, що заповнить екран ще через кілька збільшень, — лініями від її кутів до кутів екрана, а також кільцем на точці, до якої воно прямує. Типово вимкнено.
menu-view-live-panel = Панель живого рендеру
menu-view-live-panel-hover = Передня панель поверх вигляду, що показує живий рендерер за роботою: що запустив кожен кадр, який вік зображення на екрані, обхід смуг ітерацій, побудову опорних орбіт і машину під усім цим. B перемикає її розміри, а × у її куті закриває її; перетягніть її в будь-який кут; наведіть курсор на прилад, щоб дізнатися, що він означає і який вигляд мають норма та проблема.
# The live render panel's sizes.
menu-view-live-off = Вимкнено
menu-view-live-off-hover = Без панелі.
menu-view-live-strip = Смуга
menu-view-live-strip-hover = Ніксі-індикатори та блок ламп.
menu-view-live-panel-size = Панель
menu-view-live-panel-size-hover = Прилади: шкала обходу, вік зображення, плитки, проходи GPU, шкала глибини, лампи.
menu-view-live-full = Повна
menu-view-live-full-hover = Панель і машинне відділення: кроки, GPU, памʼять, ядра, пікселі.
menu-view-live-opacity = Непрозорість
menu-view-watermark = Показувати водяний знак «Fd»
menu-view-watermark-hover = Малювати непомітний фірмовий знак «Fd» у правому нижньому куті живого вигляду та експортованих зображень. Типово ввімкнено.

## Color menu

menu-color = Колір
menu-color-method = Метод
menu-color-palette = Палітра
menu-color-custom = Власна
menu-color-duotone = Двотонна
menu-color-binary = Двійкова (множина)
menu-color-binary-hover = Пласке двоколірне розфарбування: у множині чи поза нею, без градієнта.
menu-color-edit-gradient = Редагувати градієнт…
menu-color-normalize = Нормалізувати глибокі кольори
menu-color-normalize-hover = Переналаштовувати палітру на виміряний діапазон утечі вигляду, коли щільне поле інакше виглядало б як рябизна. Вигляд без аліасингу лишається без змін — позначте «Завжди підганяти діапазон», щоб переналаштовувати попри все. Лише для плавного методу.
menu-color-more = Цикл, зсув, анімація й ефекти: Керування ▸ Розфарбування

## Tools menu

menu-tools = Інструменти
# Followed by its shortcut, (M).
menu-tools-find-minibrot = Знайти мініброт і наблизитися до нього
menu-tools-find-minibrot-hover = Притягнути центр вигляду методом Ньютона до точного центру найближчого мініброта, повідомити його період і наблизитися до власного масштабу мініброта — часто на багато порядків за один крок. Ви вже глибше за нього? Зміститься лише центр.
menu-tools-solver = Розвʼязувач Ньютона / Мішуревича…
menu-tools-solver-hover = Знайти точний центр мініброта (період + розмір) або точку Мішуревича, найближчу до вигляду, — методом Ньютона з повною точністю, з відштовхувальним мультиплікатором λ. Відкриває діалог розташування, у якому містяться розвʼязувачі.
# Followed by its shortcut, (A).
menu-tools-autopilot = Автонаближення (автопілот)
menu-tools-autopilot-stop = Зупинити автопілот
menu-tools-autopilot-hover = Безперервне глибоке наближення без рук: пірнає до найбагатшої на деталі області, коригуючи курс дорогою. Будь-яка дія навігації зупиняє його.
menu-tools-priority = Пріоритет:
menu-tools-priority-speed = Швидкість
menu-tools-priority-speed-hover = Наближатися із заданою швидкістю; оновлювати з першим кадром, що має деталі.
menu-tools-priority-quality = Якість
menu-tools-priority-quality-hover = Лише повністю опрацьовані кадри; наближення чекає на них.
menu-tools-target = Ціль:
menu-tools-target-detail = Деталі
menu-tools-target-detail-hover = Найбагатший край у вигляді, що вибирається заново під час занурення.
menu-tools-target-misiurewicz = Точка Мішуревича
menu-tools-target-misiurewicz-hover = Найближча точка Мішуревича, знайдена один раз; занурення йде точно в неї (лише Mandelbrot).
menu-tools-orbits = Накладання орбіти
menu-tools-orbits-hover = Малювати шлях ітерацій точки під курсором. Параметри анімації: Керування ▸ Накладання.
menu-tools-orbits-normalize = Нормалізувати (вписати у вигляд)
menu-tools-orbits-normalize-hover = Вписувати орбіту в увесь вигляд, щоб вона лишалася добре скомпонованою за будь-якого масштабу (замість відображення через область перегляду, де на глибині вона вилітає за межі екрана).
menu-tools-play-tour = Відтворити тур…
menu-tools-play-tour-hover = Виберіть сценарій туру для відтворення. Кнопка ▶ на панелі інструментів повторює останній одним клацанням.
menu-tools-tour-from-view = Тур від поточного вигляду…
menu-tools-tour-from-view-hover = Створити тур, що наближається від повного вигляду до поточного, — додайте підпис і задайте тривалість занурення. Зберігається як сценарій туру (.toml), який можна редагувати, відтворювати або рендерити.
menu-tools-render-tour = Рендерити тур…
menu-tools-render-tour-hover = Рендерити завантажений тур у послідовність кадрів PNG (і за бажанням у mp4) в окремому процесі.
menu-tools-close-player = Закрити програвач туру
menu-tools-close-player-hover = Зупинити тур, закрити його програвач і відновити ваші власні налаштування ітерацій і розфарбування
menu-tools-render-farm = Рендеринг на фермі…
menu-tools-render-farm-hover = Рендерити завантажений тур на кількох компʼютерах: цей керує завданням, інші приєднуються через Файл ▸ Клієнт рендерингу….
menu-tools-benchmark = Тест продуктивності…
menu-tools-benchmark-hover = Виміряти швидкість рендерингу — з поточними налаштуваннями або стандартизованим запуском (фіксована роздільність + налаштування), порівнюваним між компʼютерами. Тривалий прогін (burn-in) повторює його, щоб перевірити стабільність / тепловий тротлінг.
menu-tools-last-benchmark = Показати останній тест продуктивності

## Navigate menu

menu-navigate = Навігація
menu-navigate-goto = Перейти до розташування…
menu-navigate-goto-hover = Введіть центр/масштаб, перейдіть до відомої точки або знайдіть методом Ньютона точку Мішуревича / мініброт поблизу вигляду.
menu-navigate-misiurewicz = Оглядач точок Мішуревича…
menu-navigate-misiurewicz-hover = Перегляньте, який вигляд мають малі типи точок Мішуревича, — мініатюри з періодом наближення та закрученням кожної точки — і знайдіть будь-яку з них методом Ньютона до довільної глибини. (Mandelbrot.)
menu-navigate-exact = Точні точки (вирази)
menu-navigate-exact-note = Корені бруньок головної кардіоїди, задані точними виразами (cos/sin частки 2π). Координата зберігається як вираз, тож під час наближення ви лишаєтеся точно на точці.
menu-navigate-random = Випадкове розташування
menu-navigate-random-hover = Перейти до випадкової багатої на деталі точки межі
menu-navigate-import-kfr = Імпортувати .kfr…
menu-navigate-import-kfr-hover = Завантажити файл розташування Kalles Fraktaler
menu-navigate-share = Поділитися розташуванням…
menu-navigate-share-hover = Копіювати / вставити / зберегти / завантажити самодостатнє розташування (.fdn): фрактал, центр із повною точністю, масштаб, розфарбування.

## Help menu

menu-help = Довідка
# Followed by its shortcut, (F1).
menu-help-reference = Довідка та посібник…
menu-help-welcome = Вітання / швидкий старт…
menu-help-welcome-hover = Огляд для першого запуску: основні елементи керування та кілька орієнтирів.
menu-help-diagnostics = Діагностика…
menu-help-diagnostics-hover = Перевірити, чи правильно Fractadyne працює на вашому обладнанні, і додати результат до звіту про проблему
menu-help-report = Повідомити про проблему…
menu-help-report-hover = Створити issue на GitHub (або надіслати лист) із доданими звітом про збій, журналом і розташуванням
menu-help-accelerated = Швидше глибоке наближення…
menu-help-accelerated-hover = Додаткова збірка, що обчислює опорні орбіти глибокого наближення у 2,5–6,4 раза швидше. Ті самі зображення; ваші налаштування переносяться.
menu-help-check-updates = Перевірити оновлення
# $track: the update track (stable or beta).
# $track: "stable" or "beta", the update track in use (the variants hold the wording).
menu-help-check-updates-hover =
    { $track ->
        [beta] Перевірити на GitHub, чи є новіша бета-збірка (попередні випуски)
       *[other] Перевірити на GitHub, чи є новіша стабільна збірка
    }
# $version: the new version; $channel: "beta" for a pre-release, "stable" otherwise (the variants hold the wording).
menu-help-update-available =
    { $channel ->
        [beta] Доступна версія { $version } (бета) — завантажити
       *[other] Доступна версія { $version } (стабільна) — завантажити
    }
menu-help-up-to-date = Версія актуальна
menu-help-tagline = Нативний оглядач фракталів
menu-help-license = Ліцензія
# $year and $holder: the copyright year and holder, as the code gives them.
menu-help-copyright = © { $year } { $holder }. Подвійне ліцензування; використовуйте за будь-якою з двох ліцензій на ваш вибір.
menu-help-source = Вихідний код

## The toolbar (its buttons are icons; these are their tooltips)

menu-toolbar-julia = Жюліа
menu-toolbar-julia-hover = Показати множину Жюліа цієї формули на все вікно
menu-toolbar-dual-hover = Обидві, повʼязані: множина ліворуч, Жюліа точки під курсором праворуч
menu-toolbar-open-hover = Відкрити вигляд або розташування (PNG / EXR, .fdn, .kfr)
menu-toolbar-gallery-hover = Галерея
menu-toolbar-export-hover = Експортувати зображення…
menu-toolbar-share-hover = Поділитися розташуванням — копіювати, вставити, зберегти або завантажити самодостатній .fdn (фрактал, центр із повною точністю, масштаб, розфарбування)
menu-toolbar-zoom-in-hover = Наблизити
menu-toolbar-zoom-out-hover = Віддалити
# $factor: the click-zoom factor (4, 10…).
menu-toolbar-click-zoom-hover = Наближення клацанням ({ $factor }×): клацання лівою кнопкою пірнає в точку, правою — відступає назад (перетягування й далі пересуває вигляд). Коефіцієнт — у Керування ▸ Навігація.
# Added to the click-to-zoom tooltip in the dual view.
menu-toolbar-click-zoom-dual = Поки це ввімкнено, закріплюйте параметр c Жюліа клацанням із Ctrl.
menu-toolbar-autopilot-running-hover = Автонаближення працює — клацніть, щоб зупинити
menu-toolbar-autopilot-hover = Автонаближення: занурення до деталей (A)
menu-toolbar-reset-hover = Скинути вигляд (миттєво)
menu-toolbar-home-hover = Віддалити до початкового вигляду (з анімацією)
menu-toolbar-bookmarks-hover = Закладки: зберегти цей вигляд або перейти до збереженого
# $name: the palette's name.
menu-toolbar-next-palette-hover = Наступна палітра ({ $name })
# $file: the last tour's file name.
menu-toolbar-play-last-hover = Відтворити тур — { $file } (клацання) · повторює останній тур; інший можна вибрати через Інструменти ▸ Відтворити тур…
menu-toolbar-play-hover = Відтворити тур… (Інструменти ▸ Відтворити тур)
menu-toolbar-speed-hover = Швидкість відтворення
menu-toolbar-fullscreen-hover = Перемкнути повноекранний режим

## Dialogs: words shared by many windows

# A button that closes a window without doing what it offers.
dlg-cancel = Скасувати
# A button that dismisses a window (nothing in it is pending).
dlg-close = Закрити
# A button that copies the window's text to the clipboard (after a copy icon).
dlg-copy = Копіювати
# A button that jumps to a location.
dlg-go = Перейти
# A button that fills the fields from the view on screen.
dlg-use-current = Взяти поточне
# A notice after a file could not be written. $error: the system's reason, as it gives it.
dlg-save-failed = Не вдалося зберегти: { $error }
# A notice when no graphics device is available to render with.
dlg-gpu-not-available = GPU недоступний
# The name of the plain-text file type in a save dialog (the extension .txt is added by the code).
dlg-filter-text = Текст

## Asked after a session that did not shut down cleanly

dlg-crash-title = Fractadyne завершив роботу некоректно
dlg-crash-intro = Попередній сеанс несподівано завершився. Звіт збережено на цьому компʼютері — бажаєте його надіслати?
dlg-crash-review = Нічого не буде надіслано, доки ви не переглянете звіт: наступний екран показує, що саме буде включено, а системну інформацію можна виключити однією позначкою.
dlg-crash-report = Повідомити…
dlg-crash-report-hover = Відкриває «Повідомити про проблему» з доданими звітом про збій і розташуванням, на якому стався збій.
dlg-crash-not-now = Не зараз
dlg-crash-dont-ask = Більше не питати після збою
dlg-crash-dont-ask-hover = Звіти й надалі зберігаються локально, а Довідка ▸ Повідомити про проблему працює завжди.

## Asked after a session that ended with a second graphics device in use

dlg-second-device-title = Вимкнути другий графічний пристрій?
# $device: the graphics device's name and details, as the system gives them.
dlg-second-device-intro = Попередній сеанс несподівано завершився, коли використовувався другий графічний пристрій ({ $device }). Можливо, причина в ньому.
dlg-second-device-note = Коли його вимкнено, усе рендерить карта, що малює вікно, як і раніше. Знову ввімкнути його можна в Додатково ▸ Друга відеокарта.
dlg-second-device-off = Вимкнути
dlg-second-device-keep = Залишити ввімкненим

## Shown when the previous session's iteration setting was set aside

dlg-quarantine-title = Налаштування ітерацій відкладено
dlg-quarantine-change = Змінити його можна будь-коли в Керування ▸ Якість.
dlg-quarantine-keep = Залишити типове
# A button. $setting: the iteration setting the previous session used (such as "20,000 iterations").
dlg-quarantine-restore = Відновити: { $setting }

## Navigate ▸ Go to location

dlg-goto-title = Перейти до розташування
# Before two choices: "x, y" (cartesian) and the polar form below.
dlg-goto-enter-as = Вводити як:
# The polar way of entering a point: an offset plus a radius r at an angle θ. Keep the symbols.
dlg-goto-polar = зсув + r ∠ θ
# The tooltip of the "?" button beside the entry choice.
dlg-goto-help-hover = Що приймають поля: дроби, степені, sqrt/sin/cos…, pi та інші константи — відкриває Довідка → Вирази координат
# Labels over the entry fields. x0, y0, r, θ, Re and Im are the fields' symbols: keep them.
dlg-goto-polar-offset = Центр зсуву  (x0, y0 — залиште 0 для чисто полярної точки)
dlg-goto-polar-radius = Радіус r, кут θ
dlg-goto-re = Центр — дійсна частина (Re)
dlg-goto-im = Центр — уявна частина (Im)
dlg-goto-zoom = Масштаб (збільшення)
# A warning under the zoom field, after a ⚠ sign. $have: the iteration count set; $typical: the count this depth typically needs.
dlg-goto-iter-warning = Кількість ітерацій зафіксовано на { $have }; ця глибина зазвичай потребує ~{ $typical }. Вигляд рендеритиметься пласким, доки її не підвищити, — або ввімкніть автоітерації (Керування ▸ Якість), які підлаштовуються самі.
dlg-goto-copy-hover = Скопіювати це розташування в буфер обміну
dlg-goto-cancel-hover = Закрити, нікуди не переходячи
# The expressions in brackets are examples of what can be typed: keep them as they are.
dlg-goto-paste-note = Вставте центр/масштаб, отримані від когось, або скопіюйте, щоб поділитися. Координати приймають вирази: точні дроби (-3/4), функції та константи (-0.5 + 0.25*cos(pi/4), sqrt, root(x,n), ^; кути в радіанах), а поле дійсної частини приймає ціле комплексне значення ((37+16i)/100), яке заповнює обидва поля.
# A section that opens and closes.
dlg-goto-feature = Перейти до особливої точки (Mandelbrot)
dlg-goto-feature-poi = Відомі точки:
# The dropdown's text before a point is picked.
dlg-goto-feature-poi-pick = Перейти до цікавої точки…
dlg-goto-feature-solve = Або знайдіть особливу точку поблизу поточного вигляду:
# Two choices: what to solve for. Misiurewicz is a mathematician's name (a Misiurewicz point).
dlg-goto-feature-misiurewicz = Мішуревич
dlg-goto-feature-minibrot = Найближчий мініброт
# Labels of two small number fields; k and p are their symbols.
dlg-goto-feature-preperiod = Передперіод k
dlg-goto-feature-period = Період p
# The faint text in an empty field: left empty, the value is found automatically.
dlg-goto-feature-auto-hint = авто
# A small button that empties both fields, so they are found automatically again.
dlg-goto-feature-auto = Авто
dlg-goto-feature-auto-hover = Очистити обидва поля, щоб наступний пошук визначив передперіод і період з орбіти в центрі вигляду.
dlg-goto-feature-kp-hover = Залиште обидва поля порожніми, щоб визначити їх з орбіти в центрі вигляду; знайдені значення буде вписано сюди. Ніхто, дивлячись на спіраль, не знає її передперіоду, і саме потреба вказувати його робила цей розвʼязувач непридатним на практиці.
# While a solve runs, followed by its elapsed time and the note below.
dlg-goto-feature-solving = Обчислення…
dlg-goto-feature-solving-note = (довільна точність; глибокі вигляди потребують більше часу)
dlg-goto-feature-cancel-hover = Припинити очікування. Саме обчислення йде до кінця у фоновому режимі, а його відповідь відкидається — нічого не лишається в напівзавершеному стані.
dlg-goto-feature-find = Знайти біля вигляду
dlg-goto-feature-find-hover =
    Знайти методом Ньютона точну особливу точку, починаючи з поточного центру (довільна точність). Спершу перейдіть ближче до неї.

    Перед пошуком задайте в полі «Масштаб» вище глибину, яку ви ХОЧЕТЕ: точку буде обчислено з точністю, потрібною для цієї глибини, і вигляд перейде просто туди. Це один пошук замість довгої послідовності ручних кроків наближення.
dlg-goto-solve-cancelled = Пошук скасовано.

## Snapshot (the camera button): its tooltip, and the choice on the first press

# The camera button's tooltip, one per Snapshot setting. Followed by its shortcut, (Ctrl+S).
dlg-snapshot-hover-ask = Знімок — зберегти вигляд як на екрані або повний рендер; перше натискання запитує
dlg-snapshot-hover-screen = Знімок — зберегти вигляд точно як на екрані, до останньої теки експорту
dlg-snapshot-hover-render = Знімок — повний рендер із параметрами експорту, до останньої теки експорту
# The window's title.
dlg-snapshot-title = Знімок
dlg-snapshot-question = Що має зберігати «Знімок» (кнопка з камерою, Ctrl+S)?
# The first choice, and a short tag beside it.
dlg-snapshot-screen = Знімок екрана
dlg-snapshot-screen-tag = миттєво
dlg-snapshot-screen-note = Зберігає вигляд точно таким, яким він є на екрані, з роздільністю екрана, як PNG в останній теці експорту. Що бачите, те й отримуєте.
# The second choice, and a short tag beside it.
dlg-snapshot-render = Повний рендер
dlg-snapshot-render-tag = може тривати довго
# $size: the image size, as 1920×1080; $ss: the supersampling factor (1–4); $format: PNG or EXR.
dlg-snapshot-render-note = Рендерить із параметрами експорту — { $size }, суперсемплінг { $ss }×, { $format }. Найкраща якість, але глибокий вигляд може рендеритися від кількох хвилин до кількох годин. Працює у фоновому режимі; Файл → Експортувати зображення… показує поступ і дає змогу скасувати рендер.
dlg-snapshot-remember = Запамʼятати мій вибір і більше не питати
dlg-snapshot-change = Змінити це можна будь-коли в меню Файл → Налаштування → Знімок.

## Share location (.fdn)

dlg-share-title = Поділитися розташуванням
dlg-share-intro = Самодостатнє розташування (фрактал, центр із повною точністю, масштаб, розфарбування). Скопіюйте його, щоб поділитися, або вставте/завантажте інше й натисніть «Застосувати».
dlg-share-thumb = Додати мініатюру до збереженого файлу
dlg-share-thumb-hover = «Зберегти .fdn» вбудовує невелике зображення цього вигляду, щоб файл можна було впізнати в галереї. Додає приблизно 55 KB і не входить до тексту вище.
dlg-share-copy-hover = Скопіювати текст у буфер обміну
# Buttons (the first after a save icon).
dlg-share-save = Зберегти .fdn…
dlg-share-load = Завантажити .fdn…
dlg-share-apply = Застосувати
dlg-share-apply-hover = Перейти до розташування з поля
dlg-share-copied = Скопійовано в буфер обміну.

## Help ▸ Report an issue (the report itself stays in English: it is read by the maintainer)

dlg-report-title = Повідомити про проблему
dlg-report-intro = Найкраще місце для повідомлення — issue на GitHub: він публічний, його можна знайти пошуком, і ви побачите, коли проблему виправлять. Електронна пошта теж підходить. Спершу перегляньте все нижче: ніщо не покине ваш компʼютер, доки ви самі цього не зробите.
# Labels before three dropdowns.
dlg-report-type = Тип:
dlg-report-severity = Серйозність:
dlg-report-repro = Відтворюваність:
dlg-report-describe = Опишіть, що сталося:
# The faint text in the empty description box.
dlg-report-describe-hint = Що сталося, кроки для відтворення і чого ви очікували натомість.
# Before the checkboxes of what the report carries.
dlg-report-include = Додати:
dlg-report-sysinfo = Відомості про систему (версія, ОС, CPU, GPU, VRAM)
dlg-report-location = Поточне розташування (.fdn)
dlg-report-log = Недавній журнал
dlg-report-frames = Запис кадрів (остання хвилина й останні 40 кадрів)
dlg-report-crash = Останній звіт про збій
dlg-report-crash-none = Останній звіт про збій (не знайдено)
dlg-report-crash-view = Розташування, на якому стався збій (.fdn)
dlg-report-crash-view-none = Розташування, на якому стався збій (.fdn) (не знайдено)
dlg-report-crash-view-hover = Точний вигляд, на якому стався збій або втрата графічного пристрою, — єдине, що дає змогу розробникові його відтворити. Записується автоматично.
dlg-report-test = Результат діагностичного тесту
dlg-report-test-none = Результат діагностичного тесту (запустіть його з Довідка → Діагностика…)
dlg-report-test-hover = Машинно перевірений результат «пройдено / не пройдено» на вашому власному обладнанні — для розробника він набагато корисніший, ніж сам лише опис
# A section that opens and closes. $size: the report's size, as "12 KB".
dlg-report-preview = Переглянути, що буде надіслано  ({ $size })
dlg-report-github = Створити issue на GitHub
dlg-report-github-hover = Відкрити сторінку нового issue із заповненим заголовком; звіт буде скопійовано в буфер обміну — вставте його в текст issue (Ctrl+V).
# Buttons (after a copy and a save icon).
dlg-report-copy = Копіювати звіт
dlg-report-copy-hover = Скопіювати все в буфер обміну
dlg-report-save = Зберегти звіт…
dlg-report-save-hover = Зберегти як .txt, щоб додати вкладенням
dlg-report-by-email = Або електронною поштою:
dlg-report-gmail = Написати в Gmail
# $address: the email address reports go to.
dlg-report-gmail-hover = Відкрити вікно створення листа Gmail на адресу { $address } із заповненою темою; вставте скопійований звіт
dlg-report-email = Поштова програма…
dlg-report-email-hover = Відкрити вашу типову поштову програму (mailto) — потрібен налаштований обробник пошти
dlg-report-tip = Порада: будь-яка з цих кнопок копіює звіт у буфер обміну — вставте його в текст issue або листа (Ctrl+V). Якщо можете, додайте також знімок екрана чи зразок зображення (перетягніть його); це дуже допомагає з проблемами рендерингу та інтерфейсу.
# Notices in the window after a button is pressed.
dlg-report-opened-github = GitHub відкрито — звіт у буфері обміну; вставте його в issue.
dlg-report-copied = Звіт скопійовано в буфер обміну.
# $path: where the report was saved.
dlg-report-saved = Збережено в { $path }
dlg-report-opened-email = Поштову програму відкрито — звіт у буфері обміну; вставте його.
dlg-report-opened-gmail = Вікно нового листа Gmail відкрито — вставте скопійований звіт у текст листа (Ctrl+V).

## A location whose checksum does not match

dlg-checksum-title = Це розташування може бути пошкоджене
# After a ⚠ sign.
dlg-checksum-mismatch = Його контрольна сума не збігається з даними поруч.
dlg-checksum-why = Зазвичай це означає, що текст обрізали або змінили дорогою сюди, — часта причина: неповна копія з вікна чату чи допису на форумі.
# A label over where the location came from (a file path, or the clipboard).
dlg-checksum-source = Джерело
# The two checksums, side by side: the one written in the file, and the one computed from its data.
dlg-checksum-in-file = у файлі
dlg-checksum-from-data = з даних
dlg-checksum-load-note = Ви все одно можете його завантажити — координати цілком можуть бути правильними, і нічого не буде перезаписано. Про все, що не вдасться прочитати, буде повідомлено після завантаження.
dlg-checksum-load = Усе одно завантажити
dlg-checksum-load-hover = Перейти туди попри розбіжність
# Notices after loading it. $zoom: the magnification, as 1.2e148.
dlg-checksum-loaded = Завантажено @ { $zoom }× попри розбіжність контрольної суми
# $zoom: the magnification, as 1.2e148; $note: what the loader found wrong with the location.
dlg-checksum-loaded-note = Завантажено @ { $zoom }× — { $note }
dlg-checksum-not-loaded = Не завантажено.

## File ▸ Reset application state

dlg-reset-title = Скинути стан програми
# After a ⚠ sign, followed by a list.
dlg-reset-warning = Це остаточно видалить усі збережені дані Fractadyne:
# The list's items, each after a bullet.
dlg-reset-session = збережений сеанс (поточний вигляд, розфарбування, параметри)
dlg-reset-bookmarks = усі закладки та їхні мініатюри
dlg-reset-refcache = кеш опорних орбіт (відбудовується, коли ви знову відвідуєте розташування)
# A label over the folder that is deleted.
dlg-reset-location = Розташування
dlg-reset-note = Цю дію неможливо скасувати. Поточний сеанс не буде збережено під час виходу, тож під час наступного запуску завантажаться типові налаштування.
dlg-reset-confirm = Скинути все
dlg-reset-done = Стан програми скинуто — під час наступного запуску завантажаться типові налаштування.
# $error: the system's reason, as it gives it.
dlg-reset-failed = Не вдалося скинути: { $error }

## Tools ▸ Tour from current view

dlg-tour-title = Тур від поточного вигляду
dlg-tour-intro = Створює тур, що наближається від повного вигляду до місця, де ви зараз.
# A label before the fractal and magnification the tour ends at.
dlg-tour-destination = Призначення
dlg-tour-notation = Примітка (необовʼязковий підпис, що показується під час занурення)
# The faint example text in the empty caption box.
dlg-tour-notation-hint = напр. Занурення до трипроменевої точки Мішуревича…
dlg-tour-duration = Тривалість наближення
# $rate: how fast the dive zooms, in powers of ten a second (as 2.5).
dlg-tour-rate = ≈ { $rate } декад/с
# A small button that puts the duration back to its default.
dlg-tour-reset = Скинути
dlg-tour-reset-hover = Типове значення за глибиною
dlg-tour-duration-note = Типове значення зростає з глибиною, щоб за зануренням було зручно стежити. Швидші занурення на екстремальному наближенні можуть блимати одноколірними кадрами.
# A button, after a save icon.
dlg-tour-save = Зберегти тур…
# The file type's name in the save dialog.
dlg-tour-filter = Сценарій Fractadyne (TOML)
# A notice. $file: the saved file's name. --render-tour is a command-line option: keep it.
dlg-tour-saved = Тур збережено → { $file }. Відтворіть його через Інструменти → Відтворити тур або рендерте з --render-tour.

## Navigate ▸ Bookmarks ▸ Manage…

dlg-bookmarks-title = Закладки
# The faint text in the empty name box.
dlg-bookmarks-name-hint = назва (необовʼязково)
# A button, after a bookmark icon.
dlg-bookmarks-add = Додати поточний вигляд
dlg-bookmarks-empty = Закладок поки немає. Додайте першу з поточного вигляду.
# The tooltip of a bookmark's delete (bin) button.
dlg-bookmarks-delete-hover = Видалити

## Tools ▸ Benchmark…

dlg-bench-title = Тест продуктивності
dlg-bench-mode = Режим
dlg-bench-current = Поточні налаштування
dlg-bench-current-hover = Відтворити фіксований тур глибокого наближення в живому вигляді з вашою поточною роздільністю та налаштуваннями.
dlg-bench-standard = Стандартизований
dlg-bench-standard-hover = Зафіксувати роздільність + усі налаштування рендерингу, щоб результат можна було порівнювати між компʼютерами.
dlg-bench-resolution = Роздільність
dlg-bench-depth = Глибина
# Standard and Ultra are the two depths in the dropdown this describes.
dlg-bench-depth-hover = Обидва занурення заходять на глибокий шлях floatexp/BLA; «Ультраглибока» (1e48×) проводить там набагато більше часу, ніж «Стандартна» (1e32×), і відповідно довше триває.
dlg-bench-burnin = Тривалий прогін (повтор)
dlg-bench-burnin-hover = Запускати тест повторно, щоб виявити проблеми стабільності й тепловий тротлінг.
# The label of a slider: how many times the benchmark runs.
dlg-bench-passes = проходи
# $size: the image size, as 1920×1080; $ss: the supersampling factor; $depth: the dive's final magnification, as 1e32.
dlg-bench-std-summary = Рендерить поза екраном у { $size }, SS { $ss }×, Mandelbrot/плавне, занурення з 60 кадрів до { $depth }×.
dlg-bench-run = Запустити

## The standardized benchmark's progress window

dlg-bench-progress-title = Виконується тест продуктивності…
# $resolution: the benchmark's resolution, as the dropdown names it.
dlg-bench-progress-mode = Стандартизований · { $resolution }
# $pass: the pass running now; $total: how many passes there are.
dlg-bench-progress-pass = Прохід тривалого прогону { $pass }/{ $total }
dlg-bench-progress-dive = Рендеринг фіксованого занурення…
# On the progress bar. $done: frames rendered; $total: frames in the dive.
dlg-bench-progress-frame = кадр { $done }/{ $total }
# $fps: frames per second, as 41.3.
dlg-bench-progress-last = останній прохід: { $fps } fps

## The benchmark's results

dlg-bench-results-title = Результати тесту продуктивності
# Buttons, each after an icon.
dlg-bench-results-run-again = Запустити знову…
dlg-bench-results-save = Зберегти…
dlg-bench-results-none = Тест продуктивності ще не запускали.
dlg-bench-results-saved = Результати тесту збережено.

## The accelerated build's notice when its arithmetic libraries are missing

dlg-backend-title = Прискорена збірка — використовується вбудована арифметика
dlg-backend-dont-show = Більше не показувати

## File ▸ Gallery…

dlg-gallery-title = Галерея
# A button that picks the folder to show.
dlg-gallery-folder = Тека…
# A button, after a refresh icon.
dlg-gallery-refresh = Оновити
dlg-gallery-empty = У цій теці немає зображень Fractadyne.
# An image's title when it does not name its fractal.
dlg-gallery-untitled = Зображення Fractadyne
# $zoom: the image's magnification, as 1.2e148; $saved: when it was saved, as the file gives it.
dlg-gallery-zoom = масштаб { $zoom }   { $saved }
# The notes saved with the image, in quotation marks. Use your language's quotation marks.
dlg-gallery-notes = «{ $notes }»
dlg-gallery-open = Відкрити цей вигляд
dlg-gallery-loaded = Вигляд завантажено з галереї.

## File ▸ Export image…

dlg-export-title = Експорт зображення
# A dropdown of image sizes.
dlg-export-size = Розмір
# The size dropdown's text for a size set by hand. $size: the size, as 1920×1080.
dlg-export-size-custom-current = Власний — { $size }
# The size dropdown's first choice.
dlg-export-size-custom = Власний…
dlg-export-size-custom-hover = Задайте ширину самі; висота визначається пропорціями нижче.
dlg-export-width = Ширина (px)
# After the width field. $height: the height in pixels, which follows from the aspect.
dlg-export-height-from-aspect = × { $height } (з пропорцій)
dlg-export-ss = Суперсемплінг
dlg-export-aspect = Пропорції
# The aspect choice that keeps the window's own shape.
dlg-export-aspect-window = Як у вікна
# Before the choices PNG and OpenEXR.
dlg-export-format = Формат:
dlg-export-hud = Панель розташування (HUD)
dlg-export-hud-hover = Впекти панель із рівнем масштабу та координатами у верхній лівий кут експортованого зображення (масштабується разом із результатом; показує масштаб і центр вигляду карти/Mandelbrot).
dlg-export-glitch = Виправлення збоїв
dlg-export-glitch-hover = Виправлення збоїв збурення за допомогою кількох опорних орбіт. Автоматично пропускається для дуже глибоких (floatexp) експортів одного вигляду, щоб побудова опорної орбіти й рендеринг ішли поза основним потоком і програма лишалася чуйною. Ніколи не використовується для виглядів Mandelbrot чи Multibrot (поза режимом Жюліа): перевірка за точним обчисленням показала, що там воно нічого не виправляло, а забирало до 5× часу експорту.
dlg-export-deep-note = Глибокий експорт: опорна орбіта будується поза основним потоком (інтерфейс лишається активним); на цій глибині виправлення збоїв пропускається.
# A dropdown, shown in the dual (map + Julia) view, and its three choices.
dlg-export-dual = Подвійне компонування
dlg-export-dual-side = Поруч
dlg-export-dual-separate = Окремі файли
dlg-export-dual-map = Лише карта
dlg-export-notes = Нотатки:
# The faint text in the empty notes box.
dlg-export-notes-hint = зберігаються із зображенням (≤120 символів)
# $width and $height: the image size in pixels; $left: how many characters the notes can still take.
dlg-export-output = Результат: { $width } × { $height } px   (залишилося символів: { $left })
# When the export's shape differs from the window's. $percent: how much more, as 25.
dlg-export-extra-width = Показує на { $percent }% більше по ширині, ніж вікно (нічого не обрізається)
# The same, when the export is taller than the window. $percent: how much more, as 25.
dlg-export-extra-height = Показує на { $percent }% більше по висоті, ніж вікно (нічого не обрізається)
dlg-export-tiles-note = Рендериться плитками (без обмеження розміру) у фоновому потоці. Файл містить вигляд і нотатки, тож його можна знову відкрити через Файл ▸ Відкрити вигляд.
# A label before the button that picks the folder exports go to.
dlg-export-folder = Тека:
dlg-export-choose = Вибрати…
dlg-export-choose-hover = Вибрати цільову теку для експорту
dlg-export-open-after = Відкрити після завершення
dlg-export-open-after-hover = Передати готове зображення системному переглядачеві, щойно його буде записано.
# $count: the graphics cards in this machine (2 or more).
dlg-export-all-gpus = Використовувати всі відеокарти ({ $count })
dlg-export-all-gpus-hover = Розділити зображення поплитково між усіма відеокартами цього компʼютера, кожна з власною копією глибокої опорної орбіти. Однакові карти дають точно таке саме зображення, як одна карта сама; різні моделі малюють кілька пікселів трохи по-різному, плитка за плиткою (виміряно: видимого шва немає). Якщо карта дає збій, її плитки переходять до інших. Зображення, яких поділ не підтримує (деякі методи розфарбування та формули), рендерить карта цього вікна.
# The export's progress.
dlg-export-preparing = Підготовка (побудова опорної орбіти)…
dlg-export-saving = Збереження…
dlg-export-rendering = Рендеринг…
# $time: the time since the export started, as 8.3s or 1m 04.0s.
dlg-export-elapsed = Минуло: { $time }
dlg-export-cancel-hover = Закрити без експорту
# A button, after a save icon.
dlg-export-save-as = Зберегти як…
dlg-export-save-as-hover = Вибрати назву та розташування файлу
dlg-export-export = Експортувати
dlg-export-export-hover = Рендерити та зберегти в теку вище (з автоматичною назвою)

## Notices worded at startup (main.rs)

# A notice shown once after starting, when the saved session comes from a newer version of the app. $saved and $handled: version numbers of the session file's format (1, 2…).
app-state-newer = Цей сеанс збережено новішою версією Fractadyne (стан v{ $saved }; ця збірка підтримує v{ $handled }). Деякі налаштування можуть не застосуватися, а під час збереження сеанс буде переписано у форматі цієї збірки.
# The title of the notice that says the accelerated build's arithmetic libraries are found again (its text comes from elsewhere).
app-notice-mpfr-restored-title = Прискорену арифметику відновлено
# After Fractadyne restarted itself because the graphics driver reset the GPU.
app-toast-gpu-recovered = Відновлено після скидання графічного пристрою — програму знову відкрито на початковому вигляді, бо попереднє розташування було надто вимогливим для рендерингу.

## Short notices (toasts)

# When the autopilot reached a view that could not be rendered and put the last good view back.
app-toast-autopilot-view-restored = Автопілот зупинено: вигляд став непридатним, тож його повернуто (записано в журнал)
# $error: the system's reason, as it gives it (often in English).
app-toast-gradients-save-failed = Не вдалося зберегти градієнти: { $error }
# $error: the reason, as the program gives it (often in English).
app-toast-gradients-serialize-failed = Не вдалося серіалізувати градієнти: { $error }
# $name: the name the user gave the gradient.
app-toast-gradient-saved = Градієнт «{ $name }» збережено.
# $error: the reason, as the program gives it (often in English).
app-toast-bookmarks-serialize-failed = Не вдалося серіалізувати закладки: { $error }
# $error: the system's reason, as it gives it (often in English).
app-toast-bookmarks-save-failed = Не вдалося зберегти закладки: { $error }
# Jumping to a named point given by exact expressions. $name: the point's name (in English).
app-toast-coordinates-failed = { $name }: не вдалося обчислити координати
# $zoom: the magnification, already formatted (1,234.5 or 1.2e148).
app-toast-random-location = Випадкове розташування · { $zoom }×
# Importing a Kalles Fraktaler .kfr location. $error: the reason (in English).
app-toast-import-failed = Не вдалося імпортувати: { $error }
# $zoom: the magnification, already formatted.
app-toast-location-loaded = Розташування завантажено @ { $zoom }×

## Go to location: why a jump could not be made (one line per field that failed)

# "Re" is the dialog's real-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-re = Re: { $error }
# "Im" is the dialog's imaginary-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-im = Im: { $error }
# The imaginary field was given a complex value. $error: the evaluator's reason (in English).
app-goto-error-im-real = Im: { $error } — введіть уявну частину як дійсне число, без i
# "Zoom" is the dialog's magnification field.
app-goto-error-zoom-empty = Масштаб: введіть збільшення, напр. 1e50 або 250000
# $zoom: what was typed, in quotes.
app-goto-error-zoom = Масштаб: не вдається прочитати { $zoom } — введіть додатне збільшення, наприклад 1e50 або 250000
app-goto-error-invalid = Некоректне введення — перевірте координати й масштаб.

## Finding a minibrot or Misiurewicz point (the M key and the Go to dialog)

app-feature-needs-holomorphic = Пошук мінібротів потребує голоморфної родини (Mandelbrot / Multibrot).
app-feature-not-in-julia = Пошук мінібротів шукає в множині Мандельброта — вимкніть режим Жюліа, щоб ним скористатися.
app-feature-busy = Пошук особливої точки вже триває — лише один за раз.
app-feature-searching = Пошук найближчого мініброта…
app-feature-mandelbrot-only = Пошук особливих точок працює лише для Mandelbrot.
app-feature-no-pair = У межах бюджету пошуку точки Мішуревича не знайдено — можливо, точка лежить глибше, ніж пошук сягає на цьому вигляді, або центр не поблизу спіралі чи точки розгалуження.
# The name of what was found, used as $feature in the messages below. $period: the minibrot's period (a whole number).
app-feature-minibrot = мініброта періоду { $period }
# $preperiod and $period: the point's two whole numbers (k, p). $octaves: how often the view repeats when zooming in (2 decimals); $twist: the turn per repeat in degrees.
app-feature-misiurewicz-lambda = точки Мішуревича ({ $preperiod },{ $period }) — повторюється кожні { $octaves } октави, закручення { $twist }°
# $preperiod and $period: the point's two whole numbers (k, p).
app-feature-misiurewicz = точки Мішуревича ({ $preperiod },{ $period })
# Added after the messages below when the fixed iteration count is too low for the new depth. $have: the iterations set now; $typical: what the depth typically needs.
app-feature-iter-note = ⚠ Кількість ітерацій зафіксовано на { $have }; ця глибина зазвичай потребує ~{ $typical } — збільште її (або ввімкніть автоітерації), якщо вигляд рендериться пласким.
# $feature: what was found (see above). $zoom and $asked: magnifications, already formatted; $octaves: the solver's depth limit.
app-feature-zoomed-capped =
    { $octaves ->
        [one] Наближено до { $feature } — { $zoom }×. Пошук обмежено { $octaves } октавою, тому він зупинився тут, а не на { $asked }×.
        [few] Наближено до { $feature } — { $zoom }×. Пошук обмежено { $octaves } октавами, тому він зупинився тут, а не на { $asked }×.
        [many] Наближено до { $feature } — { $zoom }×. Пошук обмежено { $octaves } октавами, тому він зупинився тут, а не на { $asked }×.
       *[other] Наближено до { $feature } — { $zoom }×. Пошук обмежено { $octaves } октави, тому він зупинився тут, а не на { $asked }×.
    }
# $feature: what was found. $zoom: the magnification, already formatted. $seconds: how long the search took (1 decimal).
app-feature-zoomed = Наближено до { $feature } — { $zoom }× ({ $seconds } s)
# $feature: what was found. $seconds: how long the search took (1 decimal).
app-feature-snapped = Притягнуто до центру { $feature } ({ $seconds } s)
app-feature-no-minibrot = Поблизу вигляду не знайдено центру мініброта — наблизьтеся до нього.
# $feature: the point asked for, e.g. Misiurewicz (3,1). $widths: how many view-widths away, already formatted. $octaves: how far to zoom out.
app-feature-just-outside =
    { $octaves ->
        [one] Розташування { $feature } знайдено трохи за межами цього вигляду — на відстані, що дорівнює { $widths } ширинам вигляду. Віддаліться приблизно на { $octaves } октаву, щоб побачити точку на екрані, або задайте більший передперіод для особливої точки цього масштабу.
        [few] Розташування { $feature } знайдено трохи за межами цього вигляду — на відстані, що дорівнює { $widths } ширинам вигляду. Віддаліться приблизно на { $octaves } октави, щоб побачити точку на екрані, або задайте більший передперіод для особливої точки цього масштабу.
        [many] Розташування { $feature } знайдено трохи за межами цього вигляду — на відстані, що дорівнює { $widths } ширинам вигляду. Віддаліться приблизно на { $octaves } октав, щоб побачити точку на екрані, або задайте більший передперіод для особливої точки цього масштабу.
       *[other] Розташування { $feature } знайдено трохи за межами цього вигляду — на відстані, що дорівнює { $widths } ширинам вигляду. Віддаліться приблизно на { $octaves } октави, щоб побачити точку на екрані, або задайте більший передперіод для особливої точки цього масштабу.
    }
# $feature: the point asked for. $widths: how many view-widths away, already formatted (may be 2^187). $octaves: how far to zoom out.
app-feature-far =
    { $octaves ->
        [one] Розташування { $feature } знайдено, але воно на відстані, що дорівнює { $widths } ширинам вигляду, — ця особлива точка значно ГРУБША за цей вигляд. ВІДДАЛІТЬСЯ приблизно на { $octaves } октаву, щоб дістатися до неї, або спробуйте більший передперіод для особливої точки цього масштабу.
        [few] Розташування { $feature } знайдено, але воно на відстані, що дорівнює { $widths } ширинам вигляду, — ця особлива точка значно ГРУБША за цей вигляд. ВІДДАЛІТЬСЯ приблизно на { $octaves } октави, щоб дістатися до неї, або спробуйте більший передперіод для особливої точки цього масштабу.
        [many] Розташування { $feature } знайдено, але воно на відстані, що дорівнює { $widths } ширинам вигляду, — ця особлива точка значно ГРУБША за цей вигляд. ВІДДАЛІТЬСЯ приблизно на { $octaves } октав, щоб дістатися до неї, або спробуйте більший передперіод для особливої точки цього масштабу.
       *[other] Розташування { $feature } знайдено, але воно на відстані, що дорівнює { $widths } ширинам вигляду, — ця особлива точка значно ГРУБША за цей вигляд. ВІДДАЛІТЬСЯ приблизно на { $octaves } октави, щоб дістатися до неї, або спробуйте більший передперіод для особливої точки цього масштабу.
    }
# $feature: the point asked for. $residual: a small number in scientific notation.
app-feature-not-preperiodic = Орбіта тут не відповідає умовам { $feature } (залишок { $residual }) — очистьте обидва поля, щоб визначити пару з вигляду.
# $feature: the point asked for. "k/p" are the dialog's two fields.
app-feature-not-converged = Пошук { $feature } поблизу вигляду не зійшовся — перейдіть ближче або спробуйте інші k/p.
app-feature-bad-request = Передперіод і період мають бути додатними.

## Share location dialog and location files

app-location-empty = Нічого завантажувати (або текст завеликий).
app-location-not-fractadyne = Це не розташування Fractadyne.
# $zoom: the magnification, already formatted. $note: what in the text was ignored or repaired (in English).
app-share-loaded-note = Завантажено @ { $zoom }× — { $note }.
# $error: the system's reason (often in English).
app-share-save-failed = Не вдалося зберегти: { $error }
app-share-saved = Збережено.
app-share-saved-rendering-thumb = Збережено. Рендеринг мініатюри…
app-share-saved-no-thumb = Збережено — без мініатюри (для цього вигляду її не вдалося відрендерити).
app-share-saved-with-thumb = Збережено з мініатюрою.
# $error: the system's reason (often in English).
app-share-saved-thumb-failed = Збережено, але мініатюру не вдалося додати: { $error }
app-share-loaded-into-box = Завантажено в поле — перегляньте, потім натисніть «Застосувати».
# Reading a file failed (a location or a palette). $error: the system's reason (often in English).
app-read-failed = Не вдалося прочитати: { $error }
app-share-too-large = Файл завеликий (можливо, це не розташування .fdn?).

## File-dialog filter names (the extensions are added by the code)

app-filter-kfr = Розташування Kalles Fraktaler
app-filter-fdn = Розташування Fractadyne
app-filter-map = Палітра Fractint / KF
app-filter-ugr = Градієнт Ultra Fractal
app-filter-ase = Adobe swatch exchange
app-filter-ggr = Градієнт GIMP
app-filter-all = Усі файли

## Importing and saving palette files (shown under the gradient editor)

# The reason a file was refused before reading it.
app-gradient-map-too-large = занадто великий, щоб бути палітрою .map
app-gradient-ugr-too-large = занадто великий, щоб бути файлом палітри .ugr
app-gradient-ase-too-large = занадто великий, щоб бути списком зразків .ase
app-gradient-ggr-too-large = занадто великий, щоб бути градієнтом .ggr
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported-vga =
    { $count ->
        [one] Імпортовано { $count } запис із { $file } — це 6-бітна таблиця VGA (білий — 252, а не 255), збережена як є, щоб вона збігалася з власним виводом Fractint.
        [few] Імпортовано { $count } записи з { $file } — це 6-бітна таблиця VGA (білий — 252, а не 255), збережена як є, щоб вона збігалася з власним виводом Fractint.
        [many] Імпортовано { $count } записів із { $file } — це 6-бітна таблиця VGA (білий — 252, а не 255), збережена як є, щоб вона збігалася з власним виводом Fractint.
       *[other] Імпортовано { $count } запису з { $file } — це 6-бітна таблиця VGA (білий — 252, а не 255), збережена як є, щоб вона збігалася з власним виводом Fractint.
    }
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported =
    { $count ->
        [one] Імпортовано { $count } запис із { $file } як чіткі смуги.
        [few] Імпортовано { $count } записи з { $file } як чіткі смуги.
        [many] Імпортовано { $count } записів із { $file } як чіткі смуги.
       *[other] Імпортовано { $count } запису з { $file } як чіткі смуги.
    }
# $error: the reader's reason (in English).
app-gradient-map-invalid = Це не палітра .map: { $error }
# $name: the gradient's name in the file; $file: the file's name.
app-gradient-ugr-imported = Імпортовано «{ $name }» з { $file }.
# $file: the file's name; $count: how many gradients it holds.
app-gradient-ugr-pick =
    { $count ->
        [one] { $file } містить { $count } градієнт — виберіть його нижче.
        [few] { $file } містить { $count } градієнти — виберіть один нижче.
        [many] { $file } містить { $count } градієнтів — виберіть один нижче.
       *[other] { $file } містить { $count } градієнта — виберіть один нижче.
    }
# $error: the reader's reason (in English).
app-gradient-ugr-invalid = Це не файл палітри .ugr: { $error }
# $used: the swatches kept; $total: the swatches in the file.
app-gradient-ase-imported-some = Імпортовано зразків: { $used } з { $total }, рівномірно розташованих.
# $count: the swatches in the file.
app-gradient-ase-imported = Імпортовано зразків: { $count }; їх рівномірно розташовано й змішано.
# $error: the reader's reason (in English).
app-gradient-ase-invalid = Це не список зразків .ase: { $error }
app-gradient-nothing-to-save = Ще немає власного градієнта, який можна зберегти.
# $file: the saved file's name.
app-gradient-ggr-saved = Збережено «{ $file }» — середні точки, криві переходу й колірні простори повністю збережено.
# $file: the saved file's name; $count: the segments whose curve could not be kept.
app-gradient-ggr-saved-lossy =
    { $count ->
        [one] Збережено «{ $file }» — але { $count } сегмент використовує криву Безьє, яку формат .ggr не підтримує. Сегмент записано як лінійний перехід; копія у вашій бібліотеці зберігає криву.
        [few] Збережено «{ $file }» — але { $count } сегменти використовують криві Безьє, які формат .ggr не підтримує. Їх записано як лінійні переходи; копія у вашій бібліотеці зберігає криві.
        [many] Збережено «{ $file }» — але { $count } сегментів використовують криві Безьє, які формат .ggr не підтримує. Їх записано як лінійні переходи; копія у вашій бібліотеці зберігає криві.
       *[other] Збережено «{ $file }» — але { $count } сегмента використовують криві Безьє, які формат .ggr не підтримує. Їх записано як лінійні переходи; копія у вашій бібліотеці зберігає криві.
    }
# $path: the file; $error: the system's reason (often in English).
app-gradient-write-failed = Не вдалося записати { $path }: { $error }
# $name: the gradient's name; $count: its segments.
app-gradient-ggr-imported =
    { $count ->
        [one] Імпортовано «{ $name }» — { $count } сегмент із середніми точками, кривими переходу та колірними просторами.
        [few] Імпортовано «{ $name }» — { $count } сегменти з їхніми середніми точками, кривими переходу та колірними просторами.
        [many] Імпортовано «{ $name }» — { $count } сегментів з їхніми середніми точками, кривими переходу та колірними просторами.
       *[other] Імпортовано «{ $name }» — { $count } сегмента з їхніми середніми точками, кривими переходу та колірними просторами.
    }
# $error: the reader's reason (in English).
app-gradient-ggr-invalid = Це не градієнт .ggr: { $error }

## The eyedropper's banner across the top of the screen while picking a colour

# $hex: the colour under the pointer, as #rrggbb.
app-eyedropper-banner-color = Вибір кольору  ·  { $hex }  ·  клацніть, щоб узяти його, Esc — скасувати
app-eyedropper-banner = Вибір кольору  ·  клацніть, щоб узяти його, Esc — скасувати

## The gradient editor window

app-gradient-title = Редактор градієнта
# Before the two view buttons, Bar and Ring.
app-gradient-view = Вигляд
app-gradient-view-bar = Смуга
app-gradient-view-bar-hover = Пряма смуга — найкраще для точного розміщення вузлів
app-gradient-view-ring = Кільце
app-gradient-view-ring-hover = Кільце, бо палітра ЦИКЛІЧНА: стик угорі — це шов, який рендерер перетинає на кожному оберті, і градієнт, що добре виглядає на смузі, може мати там різкий край.
app-gradient-add-lane-hover = Клацніть будь-де на цій лінії, щоб додати там вузол. «+» додає вузол у найширший проміжок.
app-gradient-strip-hover = Перетягніть маркер, щоб пересунути його вузол · перетягніть саму смугу, щоб ОБЕРНУТИ весь градієнт із переходом через шов · подвійне клацання додає вузол, клацання правою кнопкою на маркері видаляє його · клавіші зі стрілками зсувають вибране (із Shift — точніше).
app-gradient-remove-selected-hover = Видалити вибраний вузол
app-gradient-ring-hover = Перетягніть саме кільце, щоб ОБЕРНУТИ весь градієнт — те, що проходить шов, повертається з іншого боку · перетягніть маркер, щоб пересунути його вузол · клацніть зовнішню доріжку, щоб додати там вузол, або ⊕ посередині, щоб додати вузол у найширший проміжок · ⊖ видаляє вибраний вузол. Порожниста позначка вгорі — це шов, де циклічна палітра замикається; вона лишається на місці, бо вона І Є стиком, а кольори обертаються під нею.
app-gradient-empty = Власного градієнта ще немає — його створить «Копіювати шаблон…», імпорт або збережений градієнт нижче, і тут зʼявляться його вузли та криві кожного сегмента.
# $count: the entries of an imported palette, more than the editor shows stop by stop.
app-gradient-too-many =
    { $count ->
        [one] { $count } імпортований запис — забагато, щоб редагувати вузол за вузлом. Скористайтеся попереднім переглядом вище або «Копіювати шаблон», щоб почати заново.
        [few] { $count } імпортовані записи — забагато, щоб редагувати вузол за вузлом. Скористайтеся попереднім переглядом вище або «Копіювати шаблон», щоб почати заново.
        [many] { $count } імпортованих записів — забагато, щоб редагувати вузол за вузлом. Скористайтеся попереднім переглядом вище або «Копіювати шаблон», щоб почати заново.
       *[other] { $count } імпортованого запису — забагато, щоб редагувати вузол за вузлом. Скористайтеся попереднім переглядом вище або «Копіювати шаблон», щоб почати заново.
    }
# The selected segment's heading. $index: its number; $total: how many there are; $left and $right: where it starts and ends, 0.000–1.000.
app-gradient-segment-heading = Сегмент { $index } з { $total }  ·  { $left } → { $right }
# A label before the list of blend curves.
app-gradient-curve = Крива
app-gradient-curve-hover = Як швидко змінюється колір уздовж цього сегмента. Форму кривої «Вигнута» задає середня точка — за середньої точки 0.50 крива точно лінійна.
# The blend curves (GIMP's kinds), in a narrow list: keep them short.
app-gradient-blend-linear = Лінійна
app-gradient-blend-curved = Вигнута
app-gradient-blend-sine = Синус
app-gradient-blend-sphere-increasing = Сфера ↑
app-gradient-blend-sphere-decreasing = Сфера ↓
app-gradient-blend-bezier = Безьє
# Hover on Bézier in the list of blend curves.
app-gradient-bezier-exact-hover = Дві контрольні точки, які можна тягнути в обох напрямках. Ця крива вже кубічна, тож перетворення нічого не змінює.
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-visible-hover = Дві контрольні точки, які можна тягнути в обох напрямках. Перетворення АПРОКСИМУЄ поточну криву — цю з точністю до { $error }, що добре помітно.
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-barely-hover = Дві контрольні точки, які можна тягнути в обох напрямках. Перетворення АПРОКСИМУЄ поточну криву — цю з точністю до { $error }, що ледь помітно.
# A label before the list of colour spaces (RGB, HSV).
app-gradient-space = Простір
# A label before the Bézier presets beside it.
app-gradient-ease = Плавність
app-gradient-ease-hover = Шаблони задають дві контрольні точки; для будь-чого іншого перетягуйте їх на кривій. Крива Безьє не має середньої точки — її маркери показують, де вона досягає половини.
# Bézier presets, as in CSS easing: short buttons.
app-gradient-ease-linear = Лінійна
app-gradient-ease-in = Вхід
app-gradient-ease-out = Вихід
app-gradient-ease-in-out = Вхід-вихід
app-gradient-midpoint = Середня точка
app-gradient-midpoint-hover = Де перехід досягає половини, як частка цього сегмента. 0.50 — посередині.
# A button that puts the midpoint back in the middle of the segment.
app-gradient-centre = Центр
app-gradient-centre-hover = Повернути до 0.50
app-gradient-no-hue = ⚠ один кінець цього сегмента не має відтінку
app-gradient-no-hue-hover = Один кінець чорний, білий або сірий, тож обхід починається з червоного й проходить усе колірне коло. Це допустимо й іноді бажано — але саме тому посередині може бути колір, якого немає на жодному кінці. Смуга під кривою показує, що відбувається.
app-gradient-segment-start-hover = Колір на ПОЧАТКУ цього сегмента
app-gradient-segment-end-hover = Колір у КІНЦІ цього сегмента
app-gradient-bytes-hover = Червоний, зелений і синій у межах 0–255
app-gradient-floats-hover = Той самий колір у межах 0–1 — саме так його зберігає рендерер
app-gradient-canvas-bezier-hover = Крива переходу для вибраного сегмента. Перетягніть будь-яку контрольну точку — по x І по y, — щоб змінити її форму. Бліді лінії — це 0 і 1; крива може виходити за них, і тоді обмежується колір, а не крива.
app-gradient-canvas-hover = Крива переходу для вибраного сегмента: як швидко змінюється колір від лівого вузла до правого. Перетягніть кільце, щоб пересунути середню точку, або перемкніть криву на Безьє, щоб мати контрольну точку, яка рухається в обох напрямках.
# The selected stop's heading. $index: its number; $total: how many there are.
app-gradient-stop-heading = Вузол { $index } з { $total }
app-gradient-stop-color-hover = Колір у цьому вузлі — клацніть зразок, щоб вибрати, або введіть шістнадцяткове значення поруч
app-gradient-stop-bytes-hover = Червоний, зелений і синій у межах 0–255 — байти, якими записано цей колір
# The eyedropper button.
app-gradient-pick = Піпетка
app-gradient-pick-hover = Піпетка — візьміть колір будь-де на екрані. Наведіть вказівник на потрібний колір і клацніть; Esc скасовує.
# Written before the stop's position (0.000–1.000) in its number field: a short abbreviation of "position".
app-gradient-stop-pos = поз.
app-gradient-nudge-hover = Зсунути цей вузол (утримуйте Shift для меншого кроку)
app-gradient-remove = Видалити
app-gradient-remove-hover = Видалити цей вузол (або клацніть ⊖ під ним)
app-gradient-add-stop = Додати вузол
app-gradient-add-stop-hover = Розділити найширший сегмент (або клацніть лінію над градієнтом)
# A menu of the built-in palettes, to start from a copy of one.
app-gradient-copy-preset = Копіювати шаблон…
app-gradient-paste = Вставити…
app-gradient-paste-hover = Імпортувати палітру з шістнадцяткових кольорів або трійок RGB 0–255
app-gradient-save-ggr = Зберегти .ggr…
app-gradient-save-ggr-hover = Записати цей градієнт у файл GIMP .ggr — його читають GIMP, Krita та Inkscape, а також цей редактор
# A menu of the palette file formats (.map, .ugr, .ggr, .ase).
app-gradient-import = Імпорт
app-gradient-import-map-hover = Завантажити файл палітри Fractint / Kalles Fraktaler .map (рядки R G B, по одному на запис)
app-gradient-import-ugr-hover = Завантажити файл градієнтів Ultra Fractal .ugr (у .ugr зазвичай багато градієнтів — ви вибираєте один)
app-gradient-import-ggr-hover = Завантажити градієнт GIMP .ggr зі збереженими середніми точками, кривими переходу та колірними просторами
app-gradient-import-ase-hover = Завантажити список зразків Adobe (.ase). Зразки не мають позицій, тому їх буде розташовано рівномірно й змішано.
# The gradient's name, for the library of saved gradients.
app-gradient-name = Назва
# Shown greyed in the empty name field.
app-gradient-name-hint = мій градієнт
# The save button, when a gradient of that name is already saved.
app-gradient-update = Оновити
app-gradient-save = Зберегти
app-gradient-save-hover = Зберегти цей градієнт у вашій бібліотеці разом із кривими та колірними просторами
app-gradient-save-needs-name-hover = Спершу дайте градієнту назву
# A menu of the saved gradients. $count: how many there are.
app-gradient-saved-menu = Збережені ({ $count })
app-gradient-nothing-saved = Ще нічого не збережено
app-gradient-delete-saved-hover = Видалити цей збережений градієнт
# $count: the gradient's segments.
app-gradient-rich =
    { $count ->
        [one] Складний градієнт: { $count } сегмент має середні точки, криві переходу або колірні простори — більше, ніж може вмістити простий список вузлів.
        [few] Складний градієнт: { $count } сегменти мають середні точки, криві переходу або колірні простори — більше, ніж може вмістити простий список вузлів.
        [many] Складний градієнт: { $count } сегментів мають середні точки, криві переходу або колірні простори — більше, ніж може вмістити простий список вузлів.
       *[other] Складний градієнт: { $count } сегмента мають середні точки, криві переходу або колірні простори — більше, ніж може вмістити простий список вузлів.
    }
app-gradient-convert = Перетворити на редаговані вузли
app-gradient-convert-hover = Замінити його простим списком вузлів. Це ВІДКИДАЄ середні точки, криві переходу та обходи відтінку, яких список вузлів не може зберегти.
# Above the list of gradients in an Ultra Fractal file. $count: how many; $file: the file's name.
app-gradient-ugr-list =
    { $count ->
        [one] { $count } градієнт у { $file }
        [few] { $count } градієнти в { $file }
        [many] { $count } градієнтів у { $file }
       *[other] { $count } градієнта в { $file }
    }
app-gradient-close-list = Закрити список
app-gradient-seamless = Безшовний (кінець збігається з початком)
app-gradient-seamless-hover = Палітра циклічна, тож позиції 1 і 0 на екрані стоять поруч. Коли це ввімкнено, колір останнього вузла тримається рівним кольору першого — вигляд «Кільце» показує стик угорі.
app-gradient-ends-differ = ⚠ кінці різні
app-gradient-ends-differ-hover = Колір у кінці градієнта не такий, як на початку, тож циклічна палітра показує різкий край там, де вона замикається.
app-gradient-hard-bands = Чіткі смуги (без змішування між записами)
app-gradient-hard-bands-hover = Семантика Fractint / .map: кожен запис — суцільний колір. Якщо вимкнути, записи змішуються в плавний градієнт.
app-gradient-paste-intro = Вставте шістнадцяткові кольори (#ff8800) або трійки 0–255 (255 136 0), розділені комами, пробілами чи новими рядками:
# Applies the pasted colours.
app-gradient-apply = Застосувати
# Empties the paste box.
app-gradient-clear = Очистити
# $used: the colours kept; $total: the colours pasted; $limit: the most that are kept.
app-gradient-paste-imported-some = Імпортовано кольорів: { $used } з { $total }, рівномірно вибраних із вашого списку (межа — { $limit }).
# $count: the colours pasted.
app-gradient-paste-imported =
    { $count ->
        [one] Імпортовано { $count } колір.
        [few] Імпортовано { $count } кольори.
        [many] Імпортовано { $count } кольорів.
       *[other] Імпортовано { $count } кольору.
    }
# $error: why the pasted text could not be read (in English).
app-gradient-paste-unreadable = Не вдалося це прочитати: { $error }
# A hint at the bottom of the editor. $count: the palette's entries.
app-gradient-bands-hint =
    { $count ->
        [one] { $count } смуга · у порядку файлу; поки смуги ввімкнено, позиції ігноруються.
        [few] { $count } смуги · у порядку файлу; поки смуги ввімкнено, позиції ігноруються.
        [many] { $count } смуг · у порядку файлу; поки смуги ввімкнено, позиції ігноруються.
       *[other] { $count } смуги · у порядку файлу; поки смуги ввімкнено, позиції ігноруються.
    }
# A hint at the bottom of the editor, in the ring view. $count: the gradient's stops.
app-gradient-ring-hint =
    { $count ->
        [one] { $count } вузол
        [few] { $count } вузли
        [many] { $count } вузлів
       *[other] { $count } вузла
    } · клацніть зовнішню доріжку, щоб додати там вузол · ⊕ додає в найширший проміжок, ⊖ видаляє вибраний вузол · перетягніть маркер, щоб пересунути його · позначка вгорі — шов
# A hint at the bottom of the editor, in the bar view. $count: the gradient's stops.
app-gradient-bar-hint =
    { $count ->
        [one] { $count } вузол
        [few] { $count } вузли
        [many] { $count } вузлів
       *[other] { $count } вузла
    } · клацніть лінію над градієнтом, щоб додати там вузол · перетягніть маркер, щоб пересунути його · ⊖ або клацання правою кнопкою видаляє
app-gradient-cancel = Скасувати
app-gradient-cancel-hover = Відкинути всі зміни, зроблені відтоді, як відкрито це вікно, і закрити його
app-gradient-ok = Гаразд
app-gradient-ok-hover = Зберегти ці зміни й закрити редактор

## Shared

# The button that closes the Help window and the "Faster deep zoom" window.
app-close = Закрити

## The "Faster deep zoom" window (Help menu): the optional accelerated build

app-accel-title = Швидше глибоке наближення
app-accel-active-heading = Ви використовуєте прискорену збірку
app-accel-active-body = Опорні орбіти глибокого наближення обчислюються за допомогою MPFR/GMP, що у 2,5–6,4 раза швидше, ніж стандартна збірка на цьому кроці — паузі перед тим, як глибокий вигляд почне проявлятися.
# $status: which arithmetic library is in use and its version (in English).
app-accel-in-use = Використовувана арифметика: { $status }
app-accel-fallback-heading = Прискорена збірка — використовується вбудована арифметика
app-accel-fallback-body = Це прискорена (MPFR) збірка, але її математичні бібліотеки не знайдено, тому вона використовує вбудовану арифметику (astro-float). Зображення ідентичні; повільніша лише пауза, перш ніж глибокий вигляд проявиться.
# Followed by the folder the program runs from, on its own line.
app-accel-fallback-dlls = Щоб використовувати швидший шлях, покладіть libmpfr-6.dll і libgmp-10.dll (а також libgcc_s_seh-1.dll, libwinpthread-1.dll) поруч із fractadyne.exe, а потім перезапустіть програму:
app-accel-warning-hidden = Попередження про це під час запуску приховано.
app-accel-show-again = Показувати знову
app-accel-offer-heading = Додаткова збірка у 2,5–6,4 раза швидша на глибині
app-accel-offer-body = Глибоке наближення витрачає багато часу на обчислення опорних орбіт на CPU — це пауза перед тим, як глибокий вигляд почне проявлятися. Додаткова збірка робить це натомість за допомогою бібліотек MPFR/GMP, що в 2,5–6,4 раза швидше, і тим більше, чим глибше ви занурюєтеся. Усе інше те саме.
app-accel-offer-identical = Зображення ІДЕНТИЧНІ ДО БАЙТА — це та сама математика, обчислена швидшою бібліотекою, перевірена на кожній формулі фракталів і на всьому корпусі порівнянь глибокого наближення.
app-accel-offer-shared = Ваші налаштування, збережений сеанс і розташування спільні для обох збірок, тож ви можете вільно перемикатися, і нічого не потрібно імпортувати.
# Shown on Windows.
app-accel-separate-windows = Це окреме завантаження, бо бібліотеки, які вона використовує, неможливо зібрати компілятором, яким збирається стандартна збірка для Windows, і вони мають іншу ліцензію (GNU LGPL v3), ніж сам Fractadyne.
# Shown on other systems.
app-accel-separate = Це окреме завантаження, бо бібліотеки, які вона використовує, мають іншу ліцензію (GNU LGPL v3), ніж сам Fractadyne.
app-accel-download = Завантажити для цієї версії
app-accel-all-releases = Усі випуски
app-accel-all-releases-hover = Якщо пряме посилання видає 404, для цієї версії ще немає прискореної збірки — виберіть тут найближчу.
# Shown on Windows.
app-accel-install-windows = Розпакуйте її та запустіть fractadyne.exe з тієї теки, залишивши файли .dll поруч.
# Shown on Linux.
app-accel-install-linux = Розпакуйте її та запустіть ./fractadyne з тієї теки. Вона використовує системні GMP і MPFR (libgmp10 / libmpfr6), які є майже на кожному настільному компʼютері, — див. README-ACCELERATED.txt у пакеті.
# Shown on other systems.
app-accel-install-other = Для цієї платформи прискорений пакет ще не збирається; його можна зібрати з вихідного коду з --features fractadyne-core/rug.

## The Help window's frame (its sections are under help-)

app-help-title = Довідка Fractadyne

## Checking for updates

app-toast-up-to-date = У вас найновіша версія.
# $error: why the check failed (in English).
app-toast-update-failed = Не вдалося перевірити оновлення: { $error }
# The window that offers a newer version.
app-update-title = Доступне оновлення
# $version: the new version (0.3.0-beta.20). $channel: the token "stable" or "beta", in English; a translation can choose its words by it ({ $channel -> [beta] … *[other] … }).
app-update-version =
    { $channel ->
        [beta] Доступна версія Fractadyne { $version } (бета)
       *[other] Доступна версія Fractadyne { $version } (стабільна)
    }
# $current: the version running now. $track: the update track the user follows (its name in the language in use).
app-update-running = Встановлено: { $current }  ·  { $track }
app-update-download-intro = Завантажте останній випуск із GitHub:
app-update-download = Завантажити з GitHub
app-update-later = Нагадати пізніше

## The second graphics card (shown in its window, after "In use:" or "Could not start:")

# The second device is the same card as the window's. $name: the card's name.
app-worker-same-card = та сама карта, { $name }
# Why the second device could not start.
app-worker-thread-stopped = потік пристрою зупинився

## Export status (shown in the Export image window)

app-export-gpu-unavailable = GPU недоступний
app-export-reference-aborted = Не вдалося експортувати: побудову опорної орбіти перервано.

## The performance panel (View ▸ Performance panel)
## Its row labels sit in a monospace column about 14 characters wide: keep them short (a longer label pushes its value to the right).

app-perf-fps = FPS
app-perf-fps-hover = Кадри, показані за секунду, за згладженим інтервалом між кадрами. Поки вигляд усталюється, тут натомість показано поступ уточнення — вигляд, що усталюється, не працює з тією частотою, яку означало б число тут.
# What the FPS row shows while a view is settling. $done and $total: tiles; $eta: the time left, as m:ss.
app-perf-refining-tiles = уточнення { $done }/{ $total } (~{ $eta })
app-perf-refining-arming = уточнення (підготовка)
# $percent: how much is done, with one decimal.
app-perf-refining-percent = уточнення { $percent }%
app-perf-building-reference = побудова опорної орбіти
app-perf-idle = простій
app-perf-frame = кадр
app-perf-frame-hover = Згладжений реальний час між кадрами — не сума рядків під ним. Він включає очікування показу, тож на усталеному вигляді без дій він відображає частоту перемальовування, а не якусь роботу.
app-perf-cpu = cpu
app-perf-cpu-hover = Час основного потоку за одне оновлення: підготовка параметрів і керування кешем опорних орбіт. Саме цей рядок стрибає, коли перебудовується опорна орбіта, і стрибок тут — це ривок, який ви відчуваєте.
app-perf-gpu-idle = gpu/простій
app-perf-gpu-idle-hover = Час кадру мінус час CPU основного потоку. Поки вигляд рендериться, це робота GPU, але на усталеному вигляді це переважно бездіяльне очікування — назва вказує на обидва, бо лише з цього віднімання панель не може їх розрізнити.
app-perf-timing = хронометраж
# How frame cost is measured, the timing row's value. TIMESTAMP_QUERY is a name.
app-perf-timing-starved = реальний час (мітки не надходять)
app-perf-timing-gpu = мітки часу GPU
app-perf-timing-no-query = реальний час (немає TIMESTAMP_QUERY)
app-perf-timing-hover = Як вимірюється вартість кадру. Мітки часу GPU — точне джерело; запасний варіант із реальним часом вмикається, коли виміри перестають надходити, а бюджет, оцінений за реальним часом, неодноразово неправильно розраховував глибокі кадри, — сприймайте червоний колір тут як «запаси безпеки зараз лише оцінені».
app-perf-mode = режим
app-perf-mode-hover = Арифметичний шлях, яким рендерився останній кадр: прямий (без опорної орбіти), збурення df32 або floatexp — шлях із розширеним діапазоном, що використовується приблизно після 1e28x, і єдиний, на якому працює BLA.
app-perf-eff-iter = еф. ітер.
app-perf-eff-iter-hover = Ітерації, які останній кадр справді використав, після масштабування за глибиною та будь-якого обмеження під час руху, — а не базове налаштування в «Якість», яке є лише відправною точкою.
app-perf-precision = точність
app-perf-precision-hover = Біти мантиси, з якими обчислюється опорна орбіта. Їхня кількість зростає з глибиною; 64 біти — це нижня межа для мілких виглядів, а не обмеження, у яке ви впираєтеся.
app-perf-orbit-len = довж. орбіти
# $cap: the most orbit samples this GPU holds, already formatted, or the text of app-perf-uncapped.
app-perf-orbit-len-hover = Відліки опорної орбіти. Цей GPU вміщує щонайбільше { $cap } — наближення до цього числа і є практичною стіною глибини: за нею живий вигляд не може опрацювати глибші розташування, яким потрібна довша орбіта.
app-perf-uncapped = (без обмеження)
app-perf-sa-skip = пропуск SA
app-perf-sa-skip-hover = Ітерації, які апроксимація рядом пропустила, перш ніж почалася попіксельна робота. Більше — швидше, і це нічого не змінює в зображенні; рядка немає, коли нічого не пропущено.
app-perf-aa = aa
app-perf-aa-hover = Суперсемплінг: кожен піксель рендериться з такої кількості відліків на вісь, тож 2x коштує вчетверо більше роботи.
app-perf-dual = подвійний
app-perf-dual-hover = Чи працює розділений вигляд Mandelbrot/Жюліа. Коли так, кожна вартість на цій панелі охоплює ОБИДВІ половини.
app-perf-zoom = масштаб
app-perf-zoom-hover = Збільшення відносно повного вигляду. Арифметичний режим вище змінюється приблизно на 1e4x і знову приблизно на 1e28x.
app-perf-iterate = ітерування
# $target and $lethal: times in milliseconds, already formatted.
app-perf-iterate-hover = Виміряний час GPU останнього запуску ітерування. Регулятор цілиться в { $target } ms; тривалі показники поблизу { $lethal } ms — це смуга, у якій цей клас обладнання втрачав пристрій.
app-perf-steps = кроків/с
app-perf-steps-hover = Кроки ітерацій за секунду на GPU, у мільярдах. Вигляди з великою часткою внутрішніх пікселів працюють повільніше на крок, ніж підказує саме число, тож це показник пропускної здатності, а не оцінка.
app-perf-budget = бюджет
# After the budget's value while it is still converging; the hover below quotes it.
app-perf-budget-settling = (усталюється)
app-perf-budget-hover = Адаптивний бюджет кроків на один запуск — захист регулятора від кадру, достатньо довгого, щоб драйвер скинув GPU. «усталюється» означає, що бюджет ще сходиться за нечастими вимірами й поки не є надійною стелею.
app-perf-rss = rss
# $mb and $peak: memory in megabytes, already formatted.
app-perf-rss-value = { $mb } MB (пік { $peak })
app-perf-rss-hover = Резидентна памʼять процесу та найбільше значення, якого вона досягла за цей сеанс. Глибокі побудови опорних орбіт у реальних умовах сягали піка понад 2 GB, і ніщо інше на екрані про це не повідомляло; бурштиновий колір — понад три чверті системної RAM.
app-perf-gpu-est = gpu оцін.
app-perf-gpu-est-hover = Оцінка памʼяті, зайнятої на GPU: опорні орбіти, дерева BLA, текстури ітерацій, пінг-понг стану фрагментів під час уточнення та знімок для шлюзу показу. Зібрано з відомих розмірів виділень; справжнє значення драйвера неможливо прочитати переносимим способом.
app-perf-julia-re = жюліа c.re
app-perf-julia-re-hover = Дійсна частина параметра c, який рендерить панель Жюліа, — точка під курсором, якщо її не закріплено.
app-perf-julia-im = жюліа c.im
app-perf-julia-im-hover = Уявна частина параметра c, який рендерить панель Жюліа.
app-perf-julia-c = жюліа c
# The julia c row's value when the parameter is held fixed.
app-perf-pinned = закріплено
app-perf-pinned-hover = Параметр утримується в фіксованій точці, тож рух курсора більше не змінює панель Жюліа.
app-perf-c-panel = c/панель
app-perf-c-panel-hover = Ширина простору c, яку охоплює вся панель Mandelbrot. Коли вона набагато менша за один піксель Жюліа, наведення змінює c, але Жюліа виглядає незмінною, — це очікувано на глибокому наближенні, а не зависання.
app-perf-ref-recompute = перерах. опор.
app-perf-ref-recompute-hover = Тривалість останнього перерахунку опорної орбіти. Це пауза перед тим, як глибокий вигляд почне проявлятися, і саме той крок, який пришвидшує додаткова прискорена збірка.
app-perf-recompute-rate = перерах./с
app-perf-recompute-rate-hover = Опорні орбіти, ВСТАНОВЛЕНІ за секунду, — ті, чий результат справді використано. Порівняйте з рядком нижче: вони відрізняються на обсяг викинутої роботи.
app-perf-builds-rate = побуд. опор./с
app-perf-builds-rate-hover = Побудови опорних орбіт, ЗАПУЩЕНІ за секунду, включно з тими, чий результат відкинуто, — слот наперед, до якого занурення так і не доходить, або відхилене розширення. Вони коштують повного навантаження CPU і ніде більше не видні: колись цикл попереднього завантаження виконував близько 400 побудов за секунду протягом шести хвилин, тоді як «перерах./с» показував 2. Однозначні числа — норма; великий розрив між цими двома рядками означає, що робота йде на смітник.
app-perf-recompute-total = перерах. всього
app-perf-recompute-total-hover = Перерахунки опорних орбіт від запуску. Корисно як частота за відомий проміжок; число, що зростає, коли нічого не рухається, — сигнал, який варто розслідувати.

## Help: the contents list (Help & reference…, F1)

# Each section's name in the contents list (about 165 px wide: keep them short), and its heading.
help-section-overview = Огляд
help-section-navigation = Навігація
help-section-expressions = Вирази координат
help-section-options = Розфарбування й параметри
help-section-fractals = Фрактали
help-section-how-it-works = Як це працює
help-section-command-line = Командний рядок
help-section-shortcuts = Гарячі клавіші
help-section-hardware = Рекомендоване обладнання
# Credits to the prior work Fractadyne builds on.
help-section-acknowledgments = Подяки
# The third-party license notices.
help-section-licenses = Ліцензії
help-section-about = Про програму

## Help ▸ Overview

help-overview-intro = Fractadyne — нативна програма для дослідження фракталів, створена для надглибокого наближення та швидкості. Вона малює фрактали «часу втечі» — зображення, що виникають, коли в кожному пікселі знову й знову повторюється одна проста формула.
help-overview-what-is = Що таке фрактал часу втечі?
help-overview-what-is-text = Для кожного пікселя програма знову й знову виконує формулу на зразок z → z² + c, починаючи з нуля. Якщо поточне значення назавжди лишається малим, піксель належить множині й малюється темним. Якщо воно зрештою необмежено зростає («втікає»), піксель лежить поза множиною, а його колір фіксує, скільки кроків на це знадобилося. Нескінченно деталізована межа між «лишається» і «втікає» — це і є фрактал.
help-overview-can-do = Що ви можете робити
help-overview-can-pan = Пересувати вигляд і наближатися практично без обмежень (положення точне на будь-якій глибині).
help-overview-can-families = Перемикатися між 25 вбудованими родинами фракталів (десять, і пʼять із них — також у вищих степенях), власними формулами, Life та L-системами і переглядати будь-яку родину часу втечі, крім Newton, як множину Жюліа.
help-overview-can-recolor = Перефарбовувати зображення готовими або власними градієнтами та кількома методами розфарбування.
help-overview-can-lighting = Додавати рельєфне 3D-освітлення та світні контури меж.
help-overview-can-minibrots = Притягуватися до мінібротів, додавати місця в закладки й експортувати зображення високої роздільності.
help-overview-can-autopilot = Дозволити автопілоту автонаближення самостійно пірнати до деталей.
help-overview-can-share = Ділитися будь-яким місцем як невеликим текстовим фрагментом або файлом «.fdn».
help-overview-can-tours = Запускати тури за сценаріями й тест продуктивності обладнання.
help-overview-first-steps = Перші кроки
# "Seahorse Valley" is a place in the Navigate menu: use its name as that menu shows it. A and F1 are keys.
help-overview-first-steps-text = Відкрийте меню «Навігація» й виберіть «Seahorse Valley», а потім покрутіть коліщатко миші, щоб наблизитися. Перетягуйте, щоб пересувати вигляд. Або просто натисніть A, і автопілот пірне за вас. Натисніть F1 будь-коли, щоб повернутися до цієї довідки.

## Help ▸ Coordinate expressions
## The expressions, numbers and function names here are typed as shown in every language:
## keep them exactly as written. Menu items in quotation marks are the menus' own labels.

help-expr-intro = Скрізь, де Fractadyne приймає координату, замість десяткового числа можна ввести вираз: поля Re / Im у Навігація → «Перейти до розташування…» (і кожне поле його полярного режиму), рядки center_re / center_im розташування .fdn, re / im сценарію туру та --center у командному рядку. Вираз обчислюється з такою кількістю цифр, якої потребує масштаб, а діалог «Перейти до розташування» зберігає його разом із виглядом — тож точка, введена як 1/3 або як -0.5 + 0.25*cos(pi/4), лишається точно на місці, хоч як глибоко ви наближаєтеся, тоді як десяткове число зміститься, щойно його цифри закінчаться.
help-expr-examples = Приклади
help-expr-ex-third = Точна третина. Під час наближення зʼявляються нові цифри.
help-expr-ex-neck = Перешийок долини морських коників (Seahorse Valley) — корінь бруньки періоду 2.
help-expr-ex-circle = Точка на колі радіуса 1/4 навколо -0.5 (полярний режим складає цей вираз за вас).
help-expr-ex-bulb = Re кореня бруньки періоду 3; її Im дорівнює sin(2*pi/3)/2 - sin(4*pi/3)/4. У Навігація → «Точні точки (вирази)» такі точки вже вбудовано.
help-expr-ex-complex = Ціле комплексне значення, введене в поле Re, заповнює і Re, і Im.
help-expr-ex-combine = Функції та константи вільно поєднуються.
help-expr-ex-powers = Степені: 2^-20 — це 1/1048576. -2^2 — це -4: мінус застосовується після піднесення до степеня.
help-expr-ex-root = -2, дійсний кубічний корінь. Корені парного степеня з відʼємного числа відхиляються; sqrt(-1) дає i.
help-expr-ex-scientific = Експоненційний запис теж працює всередині виразів.
help-expr-numbers = Числа й оператори
# The decimal point in an expression is '.' in every language: keep that rule as it is.
help-expr-numbers-decimals = Десяткові числа з необовʼязковим порядком: 0.25, -1.5e-30, .5. Лише цифри — жодних пробілів чи ком для групування всередині числа, а десятковий роздільник — завжди «.».
help-expr-numbers-imaginary = Додайте суфікс i для уявного числа (0.25i) або використовуйте i окремо: 3 + 4*i.
# Starts with the operators themselves, two spaces apart.
help-expr-numbers-operators = +  -  *  /  і ^ (степінь; обчислюється справа наліво, тож 2^3^2 = 2^9). Дужки групують. Неявного множення немає: пишіть 2*pi, а не 2pi.
help-expr-numbers-case = Регістр у назвах не важливий: PI, Pi і pi — та сама константа.
help-expr-constants = Константи
help-expr-functions = Функції
help-expr-functions-angles = Кути задаються в радіанах. Для градусів помножте на pi/180 — cos(45*pi/180) — або перемкніть діалог «Перейти до розташування» в полярний режим і виберіть одиницю «градуси».
help-expr-refused = Що відхиляється і чому
help-expr-refused-fractional = Відʼємне число в дробовому степені, наприклад (-2)^(1/2): результат комплексний, і є кілька коренів на вибір. Використовуйте root(x, n) або cbrt(x) для дійсного кореня чи sqrt(x) для головного комплексного.
help-expr-refused-complex = Комплексний аргумент дійснозначної функції (sin(i), ln(1+i)). Комплексний аргумент приймають лише sqrt і abs; арифметичні оператори та цілі степені працюють із комплексними значеннями.
help-expr-refused-domain = Помилки області визначення: ln або log від недодатного числа, asin або acos поза -1…1, 0 у відʼємному степені, ділення на нуль.
help-expr-refused-bounds = Аргументи sin, cos, tan, exp або ^ за межами ±2^32 і вкладеність глибша за 32 рівні: вставлена координата — це недовірене введення, а такі вирази обчислювалися б необмежено довго.
help-expr-refused-grammar = Жодних змінних, функцій користувача, порівнянь чи умов. Граматику навмисно зроблено малою, щоб збережене розташування означало те саме в кожній версії.
help-expr-no-parse = Якщо вираз не розбирається
help-expr-no-parse-text = Повідомлення називає поле, каже, що знайдено і на якому символі, а для типових помилок — що ввести натомість: типографський знак мінус (−), вставлений із документа, × або ÷, десяткову кому, пропущену * між числом і назвою, незакриту дужку або функцію, записану без дужок. Ніщо не читається наполовину: якщо будь-яку частину тексту неможливо зрозуміти, всю координату відхиляють, а не мовчки скорочують.

## Help ▸ Navigation
## Menu paths (Navigate → "Go to location…") and quoted items name the interface's own
## menus and controls: use their translations. Key names (Ctrl, Shift, Space, Esc) stay.

# A heading: the mouse controls (also in Shortcuts).
help-mouse = Миша
# The left column of a controls table: a gesture or key. Keep key names (Shift, Space, Ctrl) as they are.
help-key-left-drag = Перетягування лівою кнопкою
help-nav-left-drag = Пересунути вигляд.
help-key-mouse-wheel = Коліщатко миші
help-nav-wheel = Наближення/віддалення в бік курсора; крутіть швидше, щоб за одну поділку наблизитися сильніше. Коли ввімкнено «Притягувати до особливих точок», наближення поблизу мініброта притягується до нього.
help-key-shift-drag = Shift+перетягування
help-nav-shift-drag = Наближення рамкою — намалюйте прямокутник, і його буде наближено на весь вигляд.
# $icon: the click-to-zoom toolbar button's symbol.
help-key-click-zoom = Наближення клацанням ({ $icon })
help-nav-click-zoom = Увімкніть кнопку-ціль на панелі інструментів (один вигляд): тоді клацання лівою кнопкою пірнає в точку із заданим коефіцієнтом, а правою — відступає назад. Перетягування й далі пересуває вигляд; коефіцієнт (2–100×) вибирається в розділі «Навігація» правої панелі. Backspace скасовує клацання.
help-nav-continuous = Безперервне наближення й повернення на початок
help-key-hold-space = Утримання Space
help-nav-hold-space = Плавне наближення з привʼязкою до курсора.
help-key-hold-shift-space = Утримання Shift+Space
help-nav-hold-shift-space = Плавне віддалення.
help-nav-continuous-text = Кнопка повернення до початкового вигляду анімує плавний відліт назад до повного вигляду. «Швидкість наближення» на правій панелі задає темп безперервного наближення.
help-nav-autopilot = Автопілот
help-nav-autopilot-text = Натисніть A (або Інструменти → «Автонаближення (автопілот)») для занурення без рук: автопілот вибирає точку на краю фрактала, де деталі найщільніші, віддаючи перевагу середині вигляду, і плавно пірнає в неї, поступово зводячи її до центру екрана. Він тримається цієї цілі, доки вона лишається цікавою, і плавно повертає до багатшої структури, коли така зʼявляється. Будь-яка дія навігації — або Esc — зупиняє його. «Пріоритет автонаближення» на правій панелі (або в меню «Інструменти») визначає, що для нього головне: «Швидкість» наближається із заданим темпом і оновлює екран, щойно готовий кадр із деталями; «Якість» показує лише повністю опрацьовані кадри повної роздільності й за потреби сповільнює наближення, щоб зображення ніколи не збільшувалося надто сильно, перш ніж зʼявиться наступне. У будь-якому разі екран ніколи не показує порожнього чи одноколірного кадру: оновлення, яке ще не опрацювало жодних деталей, лишається прихованим за попереднім. «Ціль автонаближення» визначає, куди він цілиться: «Деталі» (найбагатший край у вигляді, що вибирається заново дорогою) або «Точка Мішуревича» — автопілот знаходить найближчу точку Мішуревича, обчислює її до глибини межі занурення, стоячи на місці, а потім пірнає точно в неї; структура навколо такої точки повторюється кожні кілька октав без кінця, тож таке занурення ніколи не заходить у глухий кут (лише множина Мандельброта).
help-nav-history = Історія й точні переміщення
help-nav-undo = Повернутися до попереднього вигляду.
help-nav-redo = Повторити.
help-nav-goto = Навігація → «Перейти до розташування…» дає змогу прочитати, ввести, вставити або скопіювати точний центр і масштаб (повна точність, будь-яка глибина). Поля центру приймають не лише прості числа: дроби на кшталт «1/3» і вирази з pi, tau, e, sqrt і sin/cos/tan — тож «-0.5 + 0.25*cos(pi/4)» є допустимою координатою. Вираз зберігається разом із виглядом і переобчислюється з точністю, потрібною для кожної глибини, тож точка, введена так, лишається точною, хоч як далеко ви наближаєтеся (округлене десяткове число рано чи пізно змістилося б). Полярний режим (зсув + r ∠ θ, у градусах / радіанах / обертах) складає такий вираз за вас. Повний перелік функцій і констант із прикладами — у розділі «Вирази координат» цієї довідки (його відкриває кнопка ? у діалозі). Навігація → «Закладки» зберігає й повертає розташування.
help-nav-exact-points = Навігація → «Точні точки (вирази)» переходить до точок, заданих точно, а не округленим десятковим числом, — коренів бруньок головної кардіоїди, де приєднується кожна брунька періоду q, записаних як вирази з cos/sin. Оскільки вираз подорожує разом із виглядом, до такої точки можна наближатися, і вона ніколи не зміститься.
help-nav-share = Навігація → «Поділитися розташуванням…» фіксує весь вигляд — фрактал, центр із повною точністю, масштаб і розфарбування — як компактний фрагмент «.fdn», який можна скопіювати, вставити назад і застосувати або зберегти/завантажити як файл, щоб точно відтворити місце (або надіслати комусь).
help-nav-open = Файл → «Відкрити вигляд або розташування…» безпосередньо відкриває будь-що з цього: експортований PNG/EXR (його вбудований вигляд відновлюється), файл розташування «.fdn» або «.kfr» Kalles Fraktaler.
help-nav-finding = Пошук деталей
help-nav-minibrots = Мініброти — крихітні копії всієї множини, сховані вздовж межі. Приблизно відцентруйте один і натисніть M (або Інструменти → «Знайти мініброт і наблизитися до нього»), щоб методом Ньютона точно притягнутися до його центру, дізнатися його період і наблизитися просто до його власного масштабу — часто на багато порядків за один крок. Якщо ви вже глибше за мініброт, зміститься лише центр.
help-nav-minimap = Вигляд → «Мінікарта» показує невелику карту всієї множини з позначкою «ви тут» і поточною глибиною наближення; клацніть карту, щоб перейти до області.

## Help ▸ Coloring & options
## Names in quotation marks are the controls' own labels in the right-hand panel.

help-options-intro = Усе це розташовано на правій панелі (і зберігається між сеансами).
help-options-palette = Палітра
help-options-palette-text = «Палітра» вибирає готовий градієнт, ваш власний («Власна») або двоколірний режим. «Редагувати градієнт…» відкриває редактор, у якому кожен вузол має колір і позицію (0–1); додавайте вузли, скопіюйте шаблон як відправну точку або імпортуйте файл палітри.
help-options-paste = «Вставити…» в редакторі градієнта імпортує палітру, знайдену деінде. Приймаються шістнадцяткові кольори з початковим # або без нього (#ff8800, ff8800 або коротке #f80) і прості трійки RGB 0–255, як-от «255 136 0», — формат файлів .map Fractint і Kalles Fraktaler, — розділені комами, пробілами чи новими рядками. Зберігається до 256 кольорів; довший список рівномірно проріджується до 256 зі збереженням першого й останнього кольору, щоб форма градієнта вціліла.
help-options-import-map = «Імпорт ▸ .map…» поруч завантажує файл палітри .map Fractint або Kalles Fraktaler. Це таблиці підстановки, індексовані кількістю ітерацій, без змішування між записами, і різкі колірні сходинки, які вони дають, — саме з них складається класичний вигляд Fractint, — тож так вони й імпортуються, а позначка «Чіткі смуги» дає змогу натомість змішати ті самі кольори в плавний градієнт. Власні файли Fractint зберігають 6-бітні значення кольорів, помножені на чотири, тож їхній білий — 252, а не 255; Fractadyne повідомляє про це під час імпорту й лишає значення точно такими, як записано, бо власні збережені зображення Fractint мають ті самі 252.
help-options-duotone = «Двотонна» відображає значення на плавну двоколірну шкалу («Тінь» → «Світло»). «Двійкова (множина)» — плаский двоколірний вигляд: один суцільний колір для точок усередині множини й інший для точок поза нею, без градієнта, — найнаочніший спосіб побачити форму множини.
help-options-cycle = «Цикл» задає, скільки разів градієнт повторюється в діапазоні ітерацій (щільніші чи ширші смуги). «Зсув» обертає весь градієнт.
help-options-normalize = «Нормалізувати глибокі кольори» підганяє палітру під діапазон значень утечі, справді присутніх у вигляді. Глибокі вигляди охоплюють сотні тисяч ітерацій, і без цього фіксований цикл обертає градієнт тисячі разів між сусідніми пікселями, перетворюючи правильні деталі на рябизну.
help-options-log-scale = «Логарифмічна шкала кольорів» розподіляє палітру за логарифмом значення утечі, а не лінійно. Що глибше, то більше значення утечі скупчуються біля верхньої межі, тож лінійне підганяння витрачає більшу частину градієнта на тонку оболонку, що прилягає до межі, а решту лишає майже пласкою; логарифмічна шкала розгортає цю область і робить кольори стабільнішими у відео з наближенням, коли діапазон зростає.
help-options-animate = «Анімація» змінює кольори з часом: «Уперед», «Назад» або «Пінг-понг» пересувають зсув; «Випадкові градієнти» безперервно синтезують гармонійні градієнти, що плавно перетікають один в один. «Швидкість» задає кількість циклів (або перетворень) за секунду; «Перемішати градієнт» генерує новий у режимі «Випадкові градієнти».
help-options-method = Метод розфарбування — як дані стають кольором
# The coloring methods, by the names the Coloring panel gives them, each with what it does.
help-method-smooth = Плавна ітерація
help-method-smooth-text = Класичні безперервні смуги за часом утечі.
help-method-stripe = Середнє смуг
help-method-stripe-text = Плавні смуги за кутом орбіти (повзунок «Щільність смуг»).
help-method-triangle = Нерівність трикутника
help-method-triangle-text = Дрібна текстура за тим, куди кожен крок потрапляє між межами.
help-method-trap = Пастка орбіти
help-method-trap-text = Відстань орбіти до фігури (точка/хрест/коло); розфарбовує й внутрішність.
help-method-distance = Оцінка відстані
help-method-distance-text = Затінення за близькістю до межі.
help-method-decomposition = Декомпозиція
help-method-decomposition-text = Комірки за кінцевим кутом утечі.
help-options-relief = Рельєфне 3D-освітлення
help-options-relief-text = Затінює поверхню за нахилом межі (похідною), створюючи рельєфний, освітлений вигляд. «Кут світла» задає напрямок; «Рельєф» — силу (менше = різкіше); «Обертати світло» анімує його. Лише голоморфні родини (Mandelbrot / Multibrot).
help-options-glow = Світіння за відстанню
help-options-glow-text = Яскраві контурні смуги, що згущуються у світні нитки біля межі. «Світіння» — міра змішування, «Ширина смуг» — проміжок між ними, а «Анімувати світіння» змушує їх текти.
help-options-quality = Якість та ітерації
help-options-quality-text = «Ітерації» — максимальна кількість кроків, після якої піксель вважається належним множині; «Автомасштабування» автоматично збільшує її під час наближення (глибші деталі потребують більше). «Згладжування» застосовує суперсемплінг до нерухомих зображень (2×–8×), щойно вигляд усталиться, приборкуючи дрібний зовнішній «пил».
help-options-other = Інше
help-options-other-text = «Швидкість наближення» задає темп безперервного наближення; обмеження FPS обмежує частоту кадрів; «Обидві, повʼязані» показує множину Мандельброта та її множину Жюліа поруч (курсор задає параметр Жюліа наживо).

## Help ▸ Fractals
## Fractal family names (Mandelbrot, Multibrot, Tricorn, Burning Ship, Celtic, Buffalo,
## Phoenix, Newton) are names: keep them as written, as the formulas. Formula syntax
## (z = z^3 - p1*z + c, sqr, |z|, if / elseif / endif, p1-p5, maxit) is typed as shown.

help-fractals-intro = Кожна родина ітерує формулу, у якій z починається з 0, а c задає піксель (час утечі), якщо не зазначено інше. z = x + iy — комплексне число.
help-fractals-mandelbrot = z → z² + c. Оригінал. Множина — це всі c, чия орбіта лишається обмеженою; вона звʼязна, а її межа настільки зморшкувата, що має розмірність Гаусдорфа 2.
help-fractals-multibrot = z → z^d + c для степеня d = 3, 4, 5. Вищі степені додають пелюстки: множина має обертову симетрію порядку (d−1) (Multibrot 3 — порядку 2, 4 — порядку 3, 5 — порядку 4).
help-fractals-tricorn = z → conj(z)² + c, де conj(x + iy) = x − iy. Спряження робить відображення антиголоморфним, що дає симетрію 3-го порядку й характерні вигнуті «кігті».
help-fractals-burning-ship = z → (|x| + i|y|)² + c — абсолютні значення беруться перед піднесенням до квадрата (дійсна = x²−y²+cx, уявна = 2|xy|+cy). Неаналітичний; глибокі наближення відкривають структури, схожі на кораблі (традиційно їх розглядають догори дриґом).
help-fractals-celtic = Як Mandelbrot, але з абсолютним значенням дійсної частини: дійсна = |x²−y²| + cx, уявна = 2xy + cy. Дає мотиви у формі сердець і щитів.
help-fractals-buffalo = Абсолютне значення обох частин z²: дійсна = |x²−y²| + cx, уявна = |2xy| + cy — щось середнє між Celtic і Burning Ship.
# A heading: the families at powers above 2 (the Fractal menu's "Higher powers").
help-fractals-powers = Вищі степені
# ("Burning Ship  3 4 5") is how the Fractal menu shows such a row: keep it as it is.
help-fractals-powers-text = Multibrot 6, 7 і 8 (z^d + c), а також Burning Ship, Tricorn, Celtic і Buffalo зі степенями 3, 4 і 5: ті самі згини, але з вищим степенем. Burning Ship d бере абсолютні значення частин z, а потім степінь; Tricorn d спершу спряжує; Celtic d і Buffalo d згинають дійсну частину (а Buffalo — і уявну) z^d. Меню «Фрактал» і випадний список показують їх по одній родині в рядку («Burning Ship  3 4 5»). Кожна з них підтримує глибоке наближення так само, як її родич зі степенем 2, а Multibrot 6, 7 і 8 також підтримують апроксимацію рядом і пошук мінібротів (M), як і Multibrot 3–5. Для Multibrot 8 радіус утечі — 128 замість 256, бо там його наступний крок вийшов би за діапазон одинарної точності.
help-fractals-phoenix = z → z² + c + p·z(n-1), де z(n-1) — попередня ітерація, а p — стала (p = -0.5). Член памʼяті створює полумʼяноподібні нитки.
help-fractals-newton = Метод Ньютона для коренів z³ − 1 = 0: z → z − (z³−1)/(3z²). Замість часу втечі пікселі розфарбовуються за тим, до якого з трьох кубічних коренів з одиниці збігається ітерація (басейни притягання), і за тим, як швидко; заплутані межі басейнів і є фракталом.
help-fractals-julia = Режим Жюліа
help-fractals-julia-text = Для кожної родини, крім Newton, можна перемкнутися на множину Жюліа: замість того щоб починати z з 0 при c = піксель, ви фіксуєте c (параметр), а z починається з пікселя. Множина Жюліа — це межа між початковими точками, що лишаються обмеженими, і тими, що втікають, для цього фіксованого c. У режимі «Обидві, повʼязані» рух курсора над панеллю Mandelbrot задає c наживо.
help-fractals-deep = Підтримка глибокого наближення
help-fractals-deep-text = Mandelbrot, Multibrot 3/4/5 і Tricorn підтримують глибоке наближення збуренням на екстремальній глибині (floatexp). Burning Ship, Celtic і Buffalo неаналітичні (вони беруть абсолютні значення), тож використовують збурення з урахуванням знака; тепер воно теж працює в діапазоні floatexp і сягає набагато глибше за стару межу df32 ~1e28× (рідкісна рябизна біля згинів abs чекає на виправлення збоїв за допомогою кількох опорних орбіт). Phoenix використовує двочленне збурення й також підтримує глибоке наближення в діапазоні floatexp (без SA/BLA, тож важче, ніж Mandelbrot). Вищі степені наближаються так само, і збурення кожного степеня йде власним ланцюжком піднесень до квадрата; Multibrot 3–8 також підтримують апроксимацію рядом і BLA, як Mandelbrot, а Tricorn, Burning Ship, Celtic і Buffalo на кожному степені підтримують BLA (без рядів), яка бере на себе внутрішність глибокого вигляду замість ітерувати її крок за кроком. Лише Newton використовує прямий шлях, різкий до ~1e6×; власні формули підтримують глибоке наближення, якщо розділ «Власні формули» нижче не каже іншого.
help-fractals-custom = Власні формули
help-fractals-custom-text = Фрактал ▸ Власна формула… приймає крок формули у виразах у стилі Fractint: z = z^3 - p1*z + c або кілька інструкцій (t = sqr(z), потім z = t*t + c). z — ітерація, c — піксель (у режимі Жюліа — стала Жюліа), p1-p5 — параметри; |z| — квадрат модуля, як у Fractint, і доступні звичні функції (sqr, abs, conj, real, imag, cabs, flip, recip, ident, exp, log, sqrt, sin, cos, tan, cotan, sinh, cosh, tanh, cotanh, asin, acos, atan, asinh, acosh, atanh, cosxx, а також floor, ceil, trunc і round, які округлюють кожну частину). log — натуральний логарифм, як у Fractint і в комплексному аналізі: те, що багато підручників пишуть як ln (десяткового логарифма немає). Формула, записана з ln, cot, coth, Re чи Im, не читається; перевірка синтаксису називає функцію, яку слід використати, кнопка переписує формулу, а коли ви вводите ln, пропонується log. Кожен крок підтримує глибоке наближення збуренням, з тією самою арифметикою розширеного діапазону після 1e28x, що й у вбудованих фракталах, — крім кроку зі степенем, показник якого змінюється разом із z або c (як-от z^c): такий рендериться прямим шляхом і різкий доти, доки один крок c в одинарній точності не охопить піксель, — приблизно від 1e4x до 1e5x, а далі від початку координат — на меншій глибині. Діалог формули повідомляє, що саме застосовується. log, sqrt і нецілі степені беруть головні значення, які стрибають на відʼємній дійсній осі. Збурення точно відстежує цей стрибок, але піксель, чия орбіта перетинає розріз там, де орбіта опорної точки його не перетинає, далі обчислюється приблизно з одинарною точністю, тож глибокі вигляди, чиї орбіти перетинають розріз, показують там шум (виміряно: чисто на 1e5x на дійсній осі для sqrt(z^4 + c), шумно для формул із log навіть на 1e4x). Кожен крок ітерується повністю (без апроксимації рядом), тож глибокий вигляд із великою кількістю ітерацій рендериться повільніше, ніж у вбудованого фрактала. Біля межі формула з функціями має хаотичні орбіти, і довга орбіта там відстежується в одинарній точності лише приблизно: при 2000 ітерацій третина пікселів sin(z) + c відрізняється від точної орбіти, при 60 — майже жоден. Вигляди, сеанси й експортовані зображення зберігають текст формули.
help-fractals-custom-sections = Розділи Fractint теж працюють. Інструкції перед ':' — це розділ ініціалізації, що виконується один раз на піксель; порівняння як остання інструкція — це власна умова втечі формули, і ітерація триває, поки вона виконується (|z| <= 100 && |z - 1| > 0.000001 зупиняє орбіту, яка втікає або збігається до 1); if (...), elseif (...), else і endif вибирають інструкції; а змінна, прочитана в кроці до того, як її задано, має значення з попереднього кроку (з розділу ініціалізації або 0). Порівняння порівнюють дійсні частини й дають 1 або 0, && і || поєднують їх, а присвоєння теж є значенням (a = b = pixel). pixel, p1-p5, pi і e можна присвоювати, і вони зберігають власне значення, доки цього не зроблено; maxit — кількість ітерацій, а ismand дорівнює 1 (Mandelbrot-форма формули). Формула з будь-чим із цього або з округленням рендериться прямим шляхом, без глибокого наближення. Бібліотека формул > Імпорт так само читає файли .frm Fractint. Кожен запис, що читається, стає формулою, а його рядок повідомляє, що змінило читання: fn1-fn4 беруть типові значення Fractint (sin, sqr, sinh, cosh); змінну Fractint c перейменовано на c_, бо c тут — піксель; імʼя, якого ніщо не задає, дорівнює 0, як у Fractint; пробіли всередині імені відкидаються (end if — це endif), як їх відкидає Fractint. Повідомлення про імпорт рахує записи, які не читаються, і наводить найчастішу причину: змінні екрана й вигляду (whitesq, scrnpix, scrnmax, center, magxmag, rotskew), lastsqr і випадкові числа не підтримуються. Класика Fractint у «Колекції» (Magnet, Nova, Spider, Manowar та інші) написана з цими розділами.
help-fractals-custom-editor = У полі формули дужки кожного рівня вкладеності мають свій колір; пара біля курсора підсвічується, а дужка без пари показується червоним (дужки в коментарях не враховуються, бо синтаксичний аналізатор їх ігнорує). Назви доповнюються під час введення: після двох літер список пропонує функції, назви та ваші власні змінні, що так починаються. Стрілки ↑ і ↓ вибирають, Tab або Enter підставляє вибране (функція — разом із відкривною дужкою), Esc закриває список, а Enter після повністю введеної назви, як і раніше, починає новий рядок.
help-fractals-custom-textbook = «Підручник» (перемикач над формулою) показує формулу такою, якою її верстають у підручнику, і дає змогу редагувати її на місці. Вводьте як у тексті: / робить дріб із попереднього члена, ^ — показник степеня (+, - або =, введені в кінці показника, виводять із нього), ( — дужки або виклик після назви функції, | — модульні риски, а кома в дужках — комплексну сталу. Стрілки переміщують по формулі, ↑ і ↓ — між чисельником і знаменником, Tab — до наступного порожнього поля; Backspace на початку дробу чи пари дужок виділяє їх, а друге натискання вилучає їх, зберігаючи вміст. Копіювання, вставлення й скасування працюють як у текстовому полі, назви доповнюються так само, як там, а екранна клавіатура вводить у зверстану формулу (□² підносить до квадрата те, що перед курсором, клавіша функції обгортає виділене). Кожна зміна записується назад у текст, тож обидва подання завжди містять ту саму формулу; рядок, який ви не редагували, зберігає свої пробіли й коментар. Запис ніколи не змінює того, що обчислюється: |z| — квадрат модуля, і показується як |z|², cabs(z) показується як |z|, а exp(z) лишається exp(z). Рядок, що не читається, показано червоним; клацніть його, щоб редагувати як текст. LaTeX копіює формулу як код LaTeX у тому вигляді, як її зверстано, а поки «Підручник» увімкнено, бібліотека формул теж показує свої формули зверстаними.
help-fractals-custom-numbers = Числа у формулі (0.1, 1/3, pi, параметри) читаються як значення подвійної точності, приблизно 16 цифр, і глибока опорна орбіта ітерується саме з цими значеннями. Тому дві формули, рівні на папері, у глибині можуть різнитися: 0.1*z множить на найближче до 0.1 число подвійної точності, тоді як z/10 ділить на точне 10, і дві картини розходяться приблизно після 1e15x (виміряно: 5% пікселів на 1e15x, третина на 1e17x, усі на 1e20x). Кожна картина правильна для своєї формули; записуйте ту, якою хочете поділитися.
help-fractals-custom-library = «Зберегти в бібліотеку» в діалозі зберігає формулу та її параметри під назвою (збереження під назвою, яка вже є в бібліотеці, оновлює цей запис). Фрактал ▸ Бібліотека формул… показує їх, щоб застосувати, редагувати, експортувати чи видалити; «Імпорт» додає формули з файлу формул, нічого не замінюючи: ту, що вже є в бібліотеці, буде пропущено, а нова, чию назву вже зайнято, стає «назва (2)». Бібліотека — це formulas.toml у теці, зазначеній у розділі «Файли й дані» (Про програму), і сама є файлом формул. Ключовий кадр туру містить формулу як formula = "..." із formula_params = [[re, im], ...], а Інструменти > Тур від поточного вигляду записує їх для вигляду з власною формулою.
help-fractals-custom-collection = Разом із Fractadyne постачається колекція з 45 формул, написаних для неї: вищі степені та варіанти родин Burning Ship і Celtic зі згинами, суміші й двокрокові гібриди, множина Мандельброта крізь інше відображення (інвертована, лог-полярна, крізь синус), трансцендентні й раціональні формули та множини Жюліа. Це «Приклади» діалогу та вкладка «Колекція» бібліотеки формул, де «Копіювати до моїх» додає формулу до ваших. Кожна має початковий вигляд, і «Застосувати» переходить туди: формула, відкрита на початковому вигляді, часто не там, де її картина. Формула, яку ви зберігаєте, поки вона показується, так само запамʼятовує вигляд, на якому ви перебуваєте, і «Застосувати» повертає до нього.

## Help ▸ How it works

help-how-escape = Час утечі й плавний колір
help-how-escape-text = Кожен піксель ітерує формулу, доки модуль значення не перевищить радіус утечі або доки не буде досягнуто межі ітерацій. Сама лише кількість кроків дає різкі смуги; додавання дробової частини, отриманої з кінцевого модуля, дає безперервний колір без смуг.
help-how-precision = Положення з довільною точністю
help-how-precision-text = Звичайним 64-бітним числам бракує цифр приблизно на наближенні 1e15×. Fractadyne зберігає центр вигляду з довільною точністю, і кількість цифр зростає під час наближення, тож розташування ніколи не погіршується. Масштаб пікселя так само зберігається з розширеним порядком, тож він не застрягає й на межі 64-бітних чисел ~1e308× — глибину обмежує лише терпіння, а не якийсь фіксований діапазон чисел.
help-how-perturbation = Збурення
help-how-perturbation-text = Ітерувати кожен піксель із високою точністю було б надто повільно. Натомість один опорний піксель ітерується з високою точністю на CPU («опорна орбіта»), а кожен інший піксель обчислюється на GPU як крихітна різниця δ від нього у швидкій низькій точності: δz → 2·Z·δz + δz² + δc.
help-how-floatexp = Екстремальна глибина (floatexp)
help-how-floatexp-text = Приблизно після 1e28× навіть ця крихітна різниця виходить за нижню межу 32-бітного діапазону, тож її зберігають як мантису плюс окремий цілий порядок («floatexp»), що знімає 32-бітну стіну глибини. Рушій перемикається автоматично: пряма математика на мілкому, збурення на глибокому і floatexp на найглибшому. Тоді глибину обмежують точність координат і бюджет ітерацій, а не фіксована межа, — перевірено порівнянням із Fraktaler-3 на еталонному корпусі з 39 розташувань, найглибше з яких — 5.63e18003× (38 неекстремальних рядків — це звичайна перевірка, до 6.13e1105×), а також внутрішнім набором тестів самоузгодженості точності до 1e1000000×; вбудований тур сягає ~1.3e420×.
help-how-reference = Вибір опорної точки й перебазування
help-how-reference-text = Опорну точку вибирають (з оцінкою у високій точності) так, щоб її орбіта якомога довше лишалася в межах вигляду. Коли різниця стає надто великою, її «перебазовують» назад на опорну орбіту, щоб зберегти точність.
help-how-series = Апроксимація рядом
help-how-series-text = На глибокому наближенні ранні ітерації ледь зсувають δ від опорної орбіти, тож їх можна обчислити всі разом: δz апроксимується коротким многочленом від δc (3-го порядку), коефіцієнти якого ітеруються вздовж опорної орбіти. Рендерер перескакує вперед до останньої ітерації, на якій цей многочлен ще точний для всього вигляду, а потім ітерує як звичайно — заощаджуючи пропущені кроки без жодної зміни зображення (перемикач у Керування ▸ Додатково). Mandelbrot і Multibrot 3–8, найглибший діапазон; не з розфарбуванням «смуги» / «нерівність трикутника» / «пастка орбіти» / «декомпозиція» (їм потрібна кожна ітерація). Там, де активна BLA, вона вже охоплює цей ранній пропуск.
help-how-distance = Оцінка відстані й освітлення
help-how-distance-text = Відстеження похідної dz/dc дає відстань кожного пікселя до межі. На ній працюють рельєфне 3D-освітлення, світіння за відстанню і метод розфарбування «відстань» — усе це коректно на будь-якій глибині наближення.
help-how-safety = Згладжування й безпека
help-how-safety-text = Нерухомі зображення проходять суперсемплінг (2–8× на вісь), щойно вигляд усталиться. Бюджет роботи тримає кожне окреме малювання GPU в межах ліміту сторожового таймера драйвера, знижуючи роздільність (ніколи не кількість ітерацій) за екстремальних налаштувань, тож глибокі вигляди лишаються деталізованими, а не стають порожніми.

## Help ▸ Command line (the flags themselves stay English)

# The section's introduction. The flags listed below it are the command line itself, English in every language.
help-cli-intro = Fractadyne працює без графічного інтерфейсу для рендерингу, експорту фільмів і турів, автоматизації, перевірок за еталонними зображеннями й тестів продуктивності. Без прапорців запускається інтерактивний графічний інтерфейс.

## Help ▸ Shortcuts

# What each mouse gesture and key does (the gestures are the Navigation section's).
help-shortcuts-pan = Пересування вигляду
# A gesture: turning the mouse wheel.
help-key-wheel = Коліщатко
help-shortcuts-wheel = Наближення до курсора
help-shortcuts-box-zoom = Наближення рамкою (намалюйте прямокутник, щоб наблизитися до нього)
# $icon: the click-to-zoom toolbar button's symbol. "tool on": while that button is on.
help-key-click-tool = Клацання (інструмент { $icon } увімкнено)
help-shortcuts-click = Пірнути в точку; клацання правою кнопкою відступає назад
# A heading: the keyboard shortcuts.
help-shortcuts-keyboard = Клавіатура
help-shortcuts-space = Безперервне наближення / віддалення
help-shortcuts-undo = Скасувати зміну вигляду
help-shortcuts-redo = Повторити зміну вигляду
help-shortcuts-minibrot = Знайти мініброт і наблизитися до нього (метод Ньютона — Рафсона)
help-shortcuts-autopilot = Автопілот автонаближення (занурення до деталей; будь-яка дія зупиняє)
# off / strip / panel / full: the panel's sizes, as View ▸ Live render panel names them.
help-shortcuts-live-panel = Панель живого рендеру: вимкнено / смуга / панель / повна (наведіть курсор на прилад, щоб дізнатися, що він означає; × на ній закриває її)
help-shortcuts-esc = Зупинити автопілот / тур, що відтворюється, або вийти з повноекранного режиму
help-shortcuts-snapshot = Знімок до останньої теки експорту — знімок екрана з виглядом таким, яким він є, або повний рендер із параметрами експорту; перше натискання запитує, що саме, а Файл → Налаштування → Знімок змінює вибір.
help-shortcuts-help = Відкрити цю довідку
# A heading: the shortcuts of the Game of Life, with the menu path to it.
help-shortcuts-life = Life (Фрактал → Автомати → Life)
help-shortcuts-life-run = Запустити / призупинити всесвіт
help-shortcuts-life-step = Крок на одне покоління / стрибок (його величина задається на панелі Life)
help-shortcuts-life-draw = Інструмент малювання ввімк. / вимк. (олівець на панелі інструментів)
# A gesture. "Draw on": while the Life draw tool is on.
help-key-click-drag-draw = Клацання / перетягування (малювання ввімк.)
help-shortcuts-life-paint = Перемкнути клітинку / малювати: штрих, що починається на мертвій клітинці, малює, а той, що починається на живій, стирає. Запущений всесвіт призупиняється, поки ви малюєте, і продовжує роботу, щойно ви відпускаєте кнопку.
# $key: a keyboard shortcut (Ctrl+Z). "Draw on": while the Life draw tool is on.
help-key-when-drawing = { $key } (малювання ввімк.)
help-shortcuts-life-undo = Скасувати останнє редагування, заливку, очищення чи зміну правила

## Help ▸ Recommended hardware

help-hardware-intro = Fractadyne розподіляє роботу між GPU (попіксельна ітерація + розфарбування) і CPU (опорна орбіта з довільною точністю, на якій побудовано глибоке наближення). Важливі обидва, але для різного.
help-hardware-gpu = GPU (найважливіше)
help-hardware-gpu-text = Кожен піксель ітерується й розфарбовується на GPU, тож швидкість GPU визначає частоту кадрів і час експорту. Дискретний GPU з високою пропускною здатністю у 32-бітних обчисленнях (одинарної точності) дає найплавніше глибоке наближення. Вбудовані GPU цілком придатні — лише повільніші, і вони раніше впираються в бюджет роботи на кадр (тоді рендерер зменшує роздільність, а не деталізацію). Потрібен сучасний графічний API: Vulkan, Direct3D 12, Metal або OpenGL через wgpu.
# The left column of a requirements table.
help-hardware-minimum = Мінімум
help-hardware-gpu-minimum = Будь-який GPU з Vulkan / DX12 / Metal / GL 3.3.
help-hardware-recommended = Рекомендовано
help-hardware-gpu-recommended = Дискретний GPU (останніх ~5 років); >= 2 GB VRAM.
help-hardware-big-exports = Для великих експортів
help-hardware-gpu-big-exports = >= 4 GB VRAM (рендеринг плитками, тож за нестачі памʼяті робота сповільнюється поступово).
# A heading: the processor.
help-hardware-cpu = CPU
help-hardware-cpu-text = Опорна орбіта високої точності (і таблиці BLA) обчислюються на CPU і здебільшого в одному потоці, тож найважливіша швидкість одного ядра — від неї залежить, як швидко проявляється глибокий вигляд після руху. Додаткові ядра здебільшого допомагають фоновому кодуванню експорту. Потрібен 64-бітний CPU.
help-hardware-cpu-recommended = Сучасний 64-бітний CPU з високою швидкістю одного ядра.
help-hardware-memory = Памʼять
help-hardware-memory-text = Глибокі опорні орбіти зростають із глибиною (багато ітерацій із високою точністю); величезні експорти тримають плитки в RAM. 8 GB цілком достатньо для щоденного дослідження; 16 GB+ — для екстремальної глибини й дуже великих експортів.
help-hardware-checking = Перевірка вашого обладнання
help-hardware-diagnostics = Довідка → «Діагностика…» запускає на вашому компʼютері ті самі тести, що й розробники. «Запустити самотест» перевіряє математику й рендеринг за відомими правильними результатами приблизно за пʼятнадцять секунд; «Запустити тест інтерфейсу» проходить інтерфейс і живий вигляд на кількох глибинах наближення, роблячи знімки екрана; «Запустити перевірку арифметики GPU» за кілька секунд і без жодного вікна перевіряє арифметику double-float шейдера на кожному графічному бекенді, який пропонує ваш компʼютер. Кожен тест запускається окремим процесом, тож тест, що порушить роботу графічного драйвера, завершить лише себе, а не ваш сеанс.
help-hardware-gpu-check = Перевірка арифметики GPU не має результату «пройдено / не пройдено» — вона фіксує, як обчислюють саме ваш GPU і драйвер, і на деякому обладнанні (на кожній карті NVIDIA, перевіреній досі) повідомляє, що компілятор шейдерів відкидає частину арифметики розширеної точності. Це справжня знахідка, яку ми намагаємося дослідити для різних виробників, а не несправність вашого компʼютера, тож хоч би що вона показала, додавання її до звіту справді допомагає.
help-hardware-report = Якщо щось виглядає неправильно, запустіть самотест і скористайтеся «Додати до звіту про проблему…»: тоді звіт міститиме машинно перевірений результат з вашого власного обладнання, а це набагато корисніше, ніж сам лише опис. Одне застереження, яке варто знати: еталонні зображення створено на одній конкретній відеокарті, а різні GPU цілком законно обчислюють трохи різні пікселі, тож на іншому обладнанні порівняння зображень перевіряються з ширшим допуском, і невеликі відмінності там очікувані, а не є несправностями.
help-hardware-notes = Примітки
help-hardware-note-watchdog = Потужний GPU не дає бюджету роботи сторожового таймера спрацьовувати за екстремальних налаштувань.
help-hardware-note-binary = Збірка — це один самодостатній виконуваний файл, нічого додатково встановлювати не потрібно.
help-hardware-note-benchmark = Запустіть вбудований тест продуктивності (Інструменти ▸ Тест продуктивності), щоб оцінити свій компʼютер.

## Help ▸ Acknowledgments & citations
## Names of people, programs, papers, books and websites stay as written, and so do the
## titles in quotation marks (the works' own titles); the prose around them translates.

help-ack-title = Подяки й цитування
# The dedication, in italics at the top of the section.
help-ack-dedication = Присвячується Stone Soup Group — волонтерам, які створили Fractint (з 1988 року).
help-ack-dedication-text = Їхній вільно поширюваний код і дух відкритого, спільного дослідження фракталів — причина, чому так багато з нас закохалися в це мистецтво. Fractadyne створено в цій традиції.
help-ack-intro = Fractadyne спирається на десятиліття праці фрактальної спільноти. Методи й попередні напрацювання, на яких він побудований, — звірені з джерелами нижче:
help-ack-deep-zoom = Алгоритми глибокого наближення
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-martin-title = Збурення й апроксимація рядом — { $who }
help-ack-martin = Метод опорної орбіти й різниці низької точності (δz → 2Z·δz + δz² + δc) і ряд порядку n, що пропускає ранні ітерації, запроваджені в SuperFractalThing та його нотатці «SuperFractalThing Maths» (sft_maths.pdf, 2013).
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bla-title = Двовимірна лінійна апроксимація (BLA) й перебазування — { $who }
help-ack-bla = Пропуск серій ітерацій за допомогою обʼєднаних лінійних відображень і перебазування на єдину опорну орбіту, щоб уникнути збоїв; розроблено на fractalforums.org (2021).
# A link's label. $who: the linked page's author.
help-ack-bla-link = Пояснення BLA — { $who }
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-glitch-title = Критерій виявлення збоїв — { $who }
help-ack-glitch = Перевірка |Z + z| << |Z|, яка визначає, коли пікселю потрібна інша опорна орбіта, — основа збурення без збоїв (fractalforums).
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-diffabs-title = Неаналітичне збурення (Burning Ship) — { $who }
help-ack-diffabs = Розбір випадків абсолютного значення «diffabs», завдяки якому збурення працює для родин на основі abs (Burning Ship / Celtic / Buffalo), на fractalforums.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mathr-title = Еталонні реалізації й перехресні перевірки — { $who }
help-ack-mathr = Fraktaler-3 і Kalles Fraktaler 2+ (спершу створений Karl Runmo) — чиї описи спрямовували розробку цього рушія і з чиїм виводом EXR Fractadyne звіряє свій рендерер.
# A heading: credits for the coloring methods.
help-ack-coloring = Розфарбування
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-harkonen-title = Плавна кількість ітерацій і «Середнє смуг» (Stripe Average) — { $who }
help-ack-harkonen = Безперервне (нормалізоване) розфарбування за часом утечі та метод «Середнє смуг» (Stripe Average) з роботи «On Smooth Fractal Coloring Techniques» (магістерська дисертація, 2007), яка також аналізує розфарбування «Нерівність трикутника» / середнє за розгалуженнями.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mitchell-title = Середнє нерівності трикутника — { $who }
help-ack-mitchell = Усереднення того, куди кожне z² + c потрапляє в межах нерівності трикутника, для плавної полумʼяноподібної зовнішньої текстури.
# A link's label: the person's tutorial pages. $who: the person's name.
help-ack-mitchell-link = { $who } — посібники
help-ack-distance = Розфарбування/освітлення за оцінкою відстані використовує класичний метод відстані до межі через dz/dc (див. Peitgen & Saupe, «The Science of Fractal Images», 1988).
help-ack-lsystems-life = L-системи та гра «Життя»
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-lindenmayer-title = L-системи — { $who }
help-ack-lindenmayer = Паралельне переписування рядків, запроваджене 1968 року для моделювання росту рослин і водоростей («Mathematical models for cellular interactions in development», Journal of Theoretical Biology).
help-ack-abop = Книга (1990), чию черепашачу інтерпретацію та нотацію для стохастичних, контекстно-залежних і параметричних систем наслідує читач L-систем; вбудовані системи з позначкою ABOP введено з її ілюстрацій.
# A link's label: the book can be read free on that website. $site: the website's name.
help-ack-abop-link = Книга, безкоштовно на { $site }
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bourke-title = Колекція L-систем — { $who }
help-ack-bourke = Багато вбудованих систем узято з його сторінок про L-системи, серед них правила Chris Wallace для кривої Серпінського й кущ Dietmar Saupe. Файли .l Fractint, які відкриваються безпосередньо, мають формат Stone Soup Group.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-conway-title = Гра «Життя» — { $who }
help-ack-conway = Клітинний автомат B3/S23 (1970), який став відомий світові завдяки рубриці Martin Gardner «Mathematical Games» у Scientific American (жовтень 1970 року).
# A credit's heading: the website, then who runs it. $site: the website's name.
help-ack-lifewiki-title = { $site } — спільнота ConwayLife.com
help-ack-lifewiki = Довідник щодо нотацій правил, які читає програма (B/S, von Neumann, Generations, літери Alan Hensel для нетоталістичних правил і MAP), щодо формату шаблонів RLE, спільного з Golly, а також щодо вбудованих правил і шаблонів, чиїх першовідкривачів він фіксує.
help-ack-foundations = Основи й інструменти
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mandelbrot-title = Множина Мандельброта — { $who }
help-ack-mandelbrot = Обʼєкт, що лежить в основі всього.
help-ack-built-with = Створено за допомогою Rust, egui/eframe (emilk) і wgpu (gfx-rs); довільна точність — через astro-float, числа подвійного слова — через twofloat; PNG/OpenEXR, rfd, serde. Дякуємо їхнім авторам і супровідникам.
help-ack-errors = Будь-які помилки в атрибуції — на совісті автора; виправлення вітаються в трекері проблем проєкту.

## Help ▸ Third-party licenses

help-licenses-title = Ліцензії сторонніх компонентів
# $licenses: Fractadyne's license expression (MIT OR Apache-2.0), as written. $url: the web address of a crate's source. THIRD-PARTY-NOTICES.md is a file name, `option-ext` a crate's name, MPL-2.0 a license's.
help-licenses-intro = Fractadyne поширюється як статично скомпонований виконуваний файл, що містить наведені нижче крейти з відкритим кодом і вбудовані шрифти. Далі наведено повні тексти ліцензій (вони є також у THIRD-PARTY-NOTICES.md, що постачається із завантаженням). Власний код Fractadyne має подвійну ліцензію { $licenses }. `option-ext` має ліцензію MPL-2.0 — його незмінений вихідний код розміщено за адресою { $url }.
# A button, after a copy icon: copies every notice to the clipboard.
help-licenses-copy-all = Копіювати всі повідомлення

## Help ▸ About

help-about-tagline = Нативна програма для дослідження фракталів, створена на Rust із wgpu.
# The arithmetic deep zoom has used. $ran: the libraries that have run so far, as names (or a note that none has yet); $built: those this build contains, with versions.
help-about-arithmetic = Арифметика глибокого наближення: { $ran } (ця збірка містить: { $built }).
help-about-files = Файли й дані
# Followed by the folder's path.
help-about-files-text = Ваш сеанс (поточний вигляд, розфарбування та параметри), закладки та їхні мініатюри, збережені градієнти й збережені формули зберігаються в цій теці користувача і завантажуються під час наступного запуску:
# Tooltip of the copy button beside the folder's path.
help-about-copy-path = Копіювати шлях
# $command: a command typed in a terminal (it stays English).
help-about-session = Файл сеансу має версію; якщо його записала новіша версія Fractadyne, ніж та, що може повністю його прочитати, програма завантажує його, наскільки може, і попереджає вас. Щоб почати з повного нуля, скористайтеся Файл ▸ Скинути стан програми… (спершу запитує) або виконайте `{ $command }` у терміналі.
help-about-license = Ліцензія
# $licenses: the license expression (MIT OR Apache-2.0), which stays as written.
help-about-license-text = { $licenses } — використовуйте за будь-якою з ліцензій на ваш вибір.
# A link to the source code, followed by ↗.
help-about-source = Вихідний код на GitHub

## The live render panel ("blinkenlights": View ▸ Live render panel, key B)
## A front panel drawn over the view like a piece of 1970s test equipment. Its legends are engraved
## in capitals: write them in capitals here (the panel shows them exactly as written, it does not
## uppercase them). Numbers, units and readouts on its tubes and dials are not translated.

# Tooltip of the × button in the panel's top-right corner. The code adds the key after it: (B).
live-close-hover = Закрити панель живого рендеру
# The legend engraved along the top plate, left of the maker's badge.
live-plate = ЖИВИЙ РЕНДЕР · ПЕРЕДНЯ ПАНЕЛЬ

## Live render panel: instrument titles
## Engraved above each instrument, and repeated as the title of its help card.

# Above the depth readout, whose tubes show the magnification as a power of two ("2^" stays).
live-title-depth = ГЛИБИНА 2^
live-title-iterations = ІТЕРАЦІЇ
# Above the frame-time readout (milliseconds).
live-title-frame-ms = КАДР MS
# Above the date-and-time readout.
live-title-clock = ДАТА · ЧАС
# The walk dial: a radar screen showing a slow refresh rendered pass after pass, band after band.
live-title-walk = ОБХІД · СМУГИ
# A meter of how far the picture on screen lags the view.
live-title-age = ВІК ЗОБРАЖЕННЯ
# What part of the picture this frame's GPU pass covered.
live-title-tiles = ПЛИТКИ
# A scope of the GPU's work per frame, in milliseconds.
live-title-scope = ПРОХОДИ GPU · MS
# The depth dial: depth, the reference orbits being built, and the arithmetic in use.
live-title-dial = ГЛИБИНА · ОПОРНІ ОРБІТИ · АРИФМЕТИКА
# Iteration steps per second.
live-title-steps = КРОКИ / С
# The two GPUs (the window's and a second, worker GPU).
live-title-engines = ДВА РУШІЇ
live-title-memory = ПАМʼЯТЬ
live-title-cores = ЯДРА · ПОТОКИ
live-title-pixels = ПОТІК ПІКСЕЛІВ
# Help card titles only: the three GPU timer lamps (T0, T1, T2 stay as they are) and the row of lamps.
live-title-timers = ТАЙМЕРИ GPU T0–T2
live-title-bank = БЛОК ЛАМП

## Live render panel: the light bank's lamps
## Engraved under each lamp, and the title of its help card. A cell is narrow (about eight capital
## letters): a longer label breaks at a space onto two lines, and is drawn smaller if it still does
## not fit.

live-lamp-plan = ПЛАН
live-lamp-dispatch = ЗАПУСК
live-lamp-live = НАЖИВО
live-lamp-held = УТРИМАНО
# The last picture reprojected (stretched) to fit the moving view.
live-lamp-reproj = РЕПРОЄКЦІЯ
live-lamp-reading = ЧИТАННЯ
# A pass of a pinned walk.
live-lamp-pin-pass = ПРОХІД ФІКС
live-lamp-split = ПОДІЛ
live-lamp-adopt = ПРИЙНЯТТЯ
live-lamp-abandon = ВІДМОВА
# "REF": a reference orbit. Built for the view as it is (LIVE), ahead of it (AHEAD), or for a tour's next hold (HOLD).
live-lamp-ref-live = ОПОРА НАЖИВО
live-lamp-ref-ahead = ОПОРА НАПЕРЕД
live-lamp-ref-hold = ОПОРА ЗУПИНКА
# The series approximation.
live-lamp-series = РЯД
# The second (worker) GPU.
live-lamp-worker = ПОМІЧНИК
live-lamp-tile = ПЛИТКА
# Supersampling (anti-aliasing); usually kept as SSAA.
live-lamp-ssaa = SSAA
live-lamp-palette = ПАЛІТРА
# The arithmetic in use (direct, df32, floatexp).
live-lamp-mode = РЕЖИМ

## Live render panel: the kinds of GPU pass
## In the GPU scope's legend (MOVING, PIN, WORKER, SETTLE, SSAA). SETTLE and SSAA also name a still
## view's walk on the walk dial.

# A moving frame's pass.
live-kind-moving = РУХ
# A pass of a pinned walk.
live-kind-pin = ФІКС
# The worker GPU's passes.
live-kind-worker = ПОМІЧНИК
# A still view's settle tile or walk pass.
live-kind-settle = УСТАЛЕННЯ
# A supersampling sample.
live-kind-ssaa = SSAA

## Live render panel: legends and readouts on the instruments

# On the walk dial during a split refresh. $done: passes done; $passes: passes in all.
live-walk-split = ПОДІЛ { $done }/{ $passes }
# On the picture-age meter's face, under its scale (0 to 2 octaves).
live-age-scale = ОКТАВ ПОЗАДУ
# The depth dial's band for direct (plain floating-point) arithmetic. The other bands, DF32 and FLOATEXP, are names and stay.
live-dial-direct = ПРЯМА
# Under the two GPU gauges: GPU 0 draws the window; GPU 1 is the worker, or there is none.
live-engines-gpu-window = GPU 0 · ВІКНО
live-engines-gpu-worker = GPU 1 · ПОМІЧНИК
live-engines-no-worker = БЕЗ ПОМІЧНИКА
# The share of displayed frames each GPU made. $window and $worker: percentages, 0–100.
live-engines-on-screen = НА ЕКРАНІ { $window } : { $worker }
# The same readout before any frame was shown.
live-engines-on-screen-none = НА ЕКРАНІ —
# Under the CPU core bars. $count: the machine's cores.
live-cores-count =
    { $count ->
        [one] { $count } ЯДРО
        [few] { $count } ЯДРА
        [many] { $count } ЯДЕР
       *[other] { $count } ЯДРА
    }
# Under the threads scope. $count: the app's busy threads.
live-cores-threads = ПОТОКИ { $count }

## Live render panel: the memory tanks' legend
## What the GPU memory holds, by kind. Very narrow: about ten letters.

# G-buffers (the picture's working textures).
live-memory-gbuf = G-БУФЕР
# The held picture and the supersampling accumulator.
live-memory-hold = УТРИМ+SSAA
# The reference orbit and its BLA (bilinear approximation) table.
live-memory-orbit = ОРБІТА+BLA
# The worker GPU's copy of the reference.
live-memory-worker = ОПОРА ПОМ.

## Live render panel: the roles on the threads scope
## Tiny labels on a small scope: at most four letters each.

# The reference builders.
live-cores-role-ref = ОПОР
# The scoring pool.
live-cores-role-pool = ПУЛ
# The thread driving the worker GPU.
live-cores-role-worker = ПОМ
# Probes: short side renders.
live-cores-role-probe = ПРОБ
# Image and tour exports.
live-cores-role-export = ЕКСП

## Live render panel: the tiles instrument's captions
## The line under the picture's plane, then the tile size (left) and the tiles a frame (right).

# A walk's pass. $lo and $hi: the iteration band it rendered (numbers).
live-tiles-pin-band = ПРОХІД ФІКС · СМУГА { $lo }–{ $hi }
live-tiles-settle-band = ПРОХІД УСТАЛЕННЯ · СМУГА { $lo }–{ $hi }
live-tiles-moving-whole = КАДР У РУСІ · УСЕ ЗОБРАЖЕННЯ
live-tiles-settled-whole = УСТАЛЕНИЙ КАДР · УСЕ ЗОБРАЖЕННЯ
# $pass: this pass; $passes: the passes the refresh is split into.
live-tiles-split-pass = ПОДІЛЕНЕ ОНОВЛЕННЯ · ПРОХІД { $pass } З { $passes }
# $grid: the grid's columns × rows, e.g. 12×8.
live-tiles-grid-done = СІТКА УСТАЛЕННЯ { $grid } · ГОТОВО
# $grid: the grid's columns × rows; $done and $total: tiles done, and in all.
live-tiles-grid-progress = СІТКА УСТАЛЕННЯ { $grid } · ГОТОВО { $done } З { $total }
# The same, while a tile too costly for one frame goes out in strips.
live-tiles-grid-strips = СІТКА УСТАЛЕННЯ { $grid } · ГОТОВО { $done } З { $total } · СТРІЧКАМИ
# $samples: supersampling samples folded so far; $lo and $hi: the iteration band the next one is walking.
live-tiles-ssaa-band = SSAA · ГОТОВО { $samples } · СМУГА { $lo }–{ $hi }
# $sample: the supersampling sample.
live-tiles-ssaa-whole = ВИБІРКА SSAA { $sample } · УВЕСЬ КАДР
# No GPU pass this frame, and the panel has not seen a frame yet.
live-tiles-idle-starting = СТАРТ
# No GPU pass this frame: the last complete picture is held on screen.
live-tiles-idle-held = УТРИМАНО
# No GPU pass this frame: the last picture is stretched to fit the view.
live-tiles-idle-reprojected = РЕПРОЄКЦІЯ
# No GPU pass this frame: the picture on screen is current.
live-tiles-idle-live = НАЖИВО
# No GPU pass this frame, and nothing to show.
live-tiles-idle-none = ПРОСТІЙ
# The tile size: $res, a whole picture's resolution (2400×1300).
live-tiles-tile-res = ПЛИТКА { $res }
# The tile size: $px, a tile's side in pixels.
live-tiles-tile-px = ПЛИТКА { $px } PX
live-tiles-no-pass = БЕЗ ПРОХОДУ
# Tiles a frame. $count: how many (0 when none).
live-tiles-rate-count = { $count }/КАДР
live-tiles-rate-one = 1 ПЛИТКА/КАДР
# $share: the part of one tile sent each frame, e.g. 0.33.
live-tiles-rate-part = { $share } ПЛИТКИ/КАДР
live-tiles-rate-sample = 1 ПЛИТКА/ВИБІРКУ
# In the plane's corner during a walk: the share of pixels still iterating. $percent: a percentage, e.g. 54 or 1.5.
live-tiles-running = У РОБОТІ { $percent }%

## Live render panel: help cards
## Shown when the pointer rests on an instrument or a lamp. Words in capitals (SETTLE, SSAA, ABANDON,
## HELD, REPROJ, ADOPT, REF AHEAD, ON SCREEN) are the panel's own legends: use the same words as
## their messages above.

# The heading of a card's list of what healthy looks like.
live-card-healthy = НОРМА
# The heading of a card's list of what signals trouble.
live-card-trouble = ПРОБЛЕМА

live-card-walk-what = Глибоке оновлення, надто дороге для одного кадру, фіксується й обходиться: рендериться прохід за проходом, і кожен дає пікселям ітерувати на октаву далі (256, 512, 1024 … аж до запитаного). Обідок — це ця кількість у логарифмічній шкалі, починаючи згори; промінь — поступ обходу; кожна дуга — один прохід, розфарбований за його часом на GPU відносно цілі 20 ms. Біла позначка чекає на показник останнього проходу; спалах — це нове зображення, що виходить на екран.
live-card-walk-note = Нерухомий вигляд, надто дорогий для одного проходу, обходиться так само, з позначкою УСТАЛЕННЯ, а кожна вибірка суперсемплінгу обходиться знову, з позначкою SSAA: кожен їхній прохід може тривати до 400 ms, і дуги розфарбовано відносно цього. Поділене оновлення натомість малює одне кільце на прохід: кожне кільце — усі смуги, але лише частка плиток.
live-card-walk-good-sweep = Розгортка, що проходить коло за кілька кадрів і закінчується спалахом.
live-card-walk-good-arcs = Дуги переважно зелені й бурштинові.
live-card-walk-good-still = На глибокому нерухомому вигляді: повільніша розгортка на кожну вибірку, дуги від зелених до бурштинових.
live-card-walk-bad-red = Червоні дуги: проходи понад бюджет, що відчуваються як ривки.
live-card-walk-bad-restart = Розгортка, що зупиняється на півдорозі й починається знову: обхід відкинуто, і зображення на екрані лишається старим.
live-card-walk-bad-hollow = Порожнисті дуги: проходи, чий час так і не повернувся, тож наступний прохід розраховується наосліп.

live-card-age-what = Наскільки зображення на екрані відстає від вигляду, в октавах наближення (одна октава — це 2×). Поки рендериться оновлення, показується розтягнуте старе зображення, і воно старіє; коли оновлення надходить, вік падає. Смуга внизу — останні кілька секунд.
live-card-age-good-sawtooth = Пилка приблизно між 0,3 і 0,8, стрілка гойдається навколо позначки 0,5.
live-card-age-good-still = Близько 0, щойно ви зупиняєтеся.
live-card-age-bad-red = Стрілка постійно за 1,0, у червоній зоні: зображення застаріло більш ніж у 2× і під час наближення виглядає розмитим або блоковим.
live-card-age-bad-ramp = Підйом, що ніколи не падає: оновлення не завершуються (стежте за лампою ВІДМОВА).

live-card-tiles-what = Як GPU-прохід цього кадру покриває зображення. Кадри в русі й проходи обходу покривають його повністю, а підпис називає смугу ітерацій проходу обходу. Прохід обходу ітерує лише ті пікселі, що ще не втекли: площина заповнюється до цієї частки (логарифмічна шкала, 0,01% унизу) і показує її в кутку, тож вона спорожнюється, коли обхід заглиблюється. Оновлення, трохи надто дороге для одного кадру, ділиться на до 8 проходів, кожен — переплетений набір плиток по 16 px. Усталений вигляд, надто дорогий для одного проходу, стає сіткою плиток, по одній за кадр від центру назовні, а плитка, яка все ще надто дорога, виходить стрічками. Унизу показано: розмір плитки й кількість плиток за кадр.
live-card-tiles-good-whole = Цілі проходи, поки ви наближаєтеся; поділені оновлення з кількох проходів.
live-card-tiles-good-grid = Коли ви зупиняєтеся: сітка з десятків плиток, заповнена за секунду-дві, потім SSAA.
live-card-tiles-bad-small = Сітка із сотень дрібних плиток: бюджет кожного проходу малий, і уточнення триватиме довго.
live-card-tiles-bad-strips = Багато плиток виходять стрічками: плитки працювали повільніше, ніж розраховано для сітки.
live-card-tiles-bad-splits = Поділи застрягли на 8 проходах: оновлення ледь уміщається, і трохи глибше воно повертається до повільнішого обходу.
live-card-tiles-bad-dark = Довго темно під час руху: жодних проходів.

live-card-scope-what = Один стовпчик на кадр, найновіший праворуч: робота GPU, яку він виконав, складена за типами (кадр у русі — зелений, прохід ФІКС — від зеленого до червоного залежно від вартості, плитка усталення чи прохід обходу нерухомого вигляду — кремовий, вибірка суперсемплінгу — лавандовий, кадри й вибірки допоміжного GPU — сині), у мілісекундах. Пунктирні лінії на 16,7 ms (60 fps), 33 і 50. Порожниста позначка — прохід, чий час так і не повернувся.
live-card-scope-good-under = Стовпчики нижче лінії 16,7, поки ви рухаєтеся.
live-card-scope-good-pin = Проходи ФІКС від зелених до бурштинових.
live-card-scope-good-samples = Вибірки суперсемплінгу нерухомого вигляду можуть тривати довше: нічого не рухається.
live-card-scope-bad-over = Стовпчики понад 33 чи 50 ms: помітні ривки.
live-card-scope-bad-seconds = Проходи тривалістю в секунди: сторожовий таймер GPU може скинути пристрій.
live-card-scope-bad-hollow = Багато порожнистих позначок: роботу розраховують без вимірів.

live-card-timers-what = Три слоти міток часу GPU. Слот загоряється, коли прохід вимірюється, і гасне, коли його показник повертається, через два-три кадри. T0 вимірює прохід кожного кадру; T1 і T2 вимірюють додаткові проходи ФІКС.
live-card-timers-good = T0 блимає щоразу, коли є робота; T1 і T2 блимають під час обходів.
live-card-timers-bad-stuck = Лампа застрягла ввімкненою: її показник не повертається, і ціни застарівають.
live-card-timers-bad-dark = Усі темні, поки GPU зайнятий: нічого не вимірюється.

live-card-dial-what = Червона стрілка — глибина вигляду: його збільшення як степінь двійки. Смуги — арифметика, яку там використовує GPU: пряма, df32 (два float на число) і floatexp (float із широким порядком) після 2^93. Точки — опорні орбіти, що будуються: зелені — для вигляду, бурштинові — попереду, куди прямує наближення, сині — для наступної зупинки туру. Кільце — встановлення; червоний хрестик — побудова, відкинута, бо вигляд її обігнав.
live-card-dial-good-ahead = Бурштинові точки зʼявляються попереду стрілки й отримують кільце, перш ніж вона до них дійде.
live-card-dial-good-crosses = Мало червоних хрестиків.
live-card-dial-bad-passing = Стрілка проходить свою останню встановлену опорну орбіту: кадри чекають на нову, що відчувається як завмирання.
live-card-dial-bad-crosses = Повторювані червоні хрестики: наближення випереджає побудову, і CPU працює марно.
live-card-dial-bad-crossing = Перехід через 2^93 — відоме повільне місце: перші кадри floatexp коштують більше.

live-card-bank-what = Кожна лампа загоряється, коли стається її подія, і згасає приблизно за 150 ms, тож зайнятий рендерер мерехтить, а бездіяльний темніє. Зліва направо: кадр, оновлення, опорні орбіти, потім помічник, усталення, палітра й арифметика.
live-card-bank-note = Наведіть курсор на лампу, щоб дізнатися, що вона означає.

live-card-depth-what = Збільшення як степінь двійки: 2^100 — це приблизно 1.3 × 10^30.
live-card-depth-note = Після 2^93 кожен крок коштує більше (floatexp), і опорні орбіти будуються довше.

live-card-iter-what = Запитана кількість ітерацій: як далеко кожен піксель може ітерувати, перш ніж вважатиметься належним множині. Вона зростає з глибиною.
live-card-iter-good = Рівномірно зростає під час занурення.
live-card-iter-bad-low = Застрягла низько на глибині: спіралі й мініброти рендеряться як пласка внутрішність.
live-card-iter-bad-high = Дуже висока: кожне оновлення коштує більше, а обходи тривають довше.

live-card-ms-what = Час між показаними кадрами.
live-card-ms-good = Інтервал вашого дисплея: 16,7 на дисплеї 60 Hz.
live-card-ms-bad-stutter = Понад 33: помітні ривки.
live-card-ms-bad-spikes = Сплески понад 50: затримки, зазвичай через один дорогий прохід (див. ПРОХОДИ GPU).

live-card-clock-what = Настінний годинник за місцевим часом, із точністю до десятої частки секунди.
live-card-clock-note = Він позначає будь-який знімок екрана з панеллю часом його створення, тож зображення проблеми можна зіставити з журналом програми.

live-card-tach-what = Кроки ітерацій за секунду. Біла стрілка: запитані кроки (пікселі × ітерації). Бурштинова: кроки, які GPU справді виконав. Розрив — це BLA (білінійна апроксимація), що перестрибує багато ітерацій за раз; ×N — цей множник. Одометр підсумовує сеанс.
live-card-tach-good = Широкий розрив на глибині: BLA робить свою справу, від ×10 до ×1000.
live-card-tach-bad-together = Стрілки разом на глибині: BLA тут не може перестрибувати (часто біля внутрішності мініброта), тож оновлення повільні.
live-card-tach-bad-red = Біла стрілка в червоній зоні: запит більший, ніж GPU може виконати за кадр.

live-card-engines-what = Наскільки зайнятий кожен GPU, з точками, що несуть його кадри на екран: час GPU його проходів за останню секунду, включно з кадрами руху й вибірками суперсемплінгу помічника. НА ЕКРАНІ — частка показаних кадрів, яку зробив кожен. З одним GPU бік помічника лишається бездіяльним.
live-card-engines-good = Обидва зайняті під час занурення, помічник постачає частку кадрів.
live-card-engines-bad-pinned = GPU 0, що малює вікно, завантажений майже на 100%: він — вузьке місце, і кадри смикаються.
live-card-engines-bad-idle = Помічник увімкнений, але бездіяльний: він не постачає кадрів.

live-card-memory-what = RAM: робочий набір програми, червона лінія — його пік. VRAM: те, що програма тримає на GPU, обчислене з розмірів, які вона виділяє, за типами (G-буфери, текстури утримання й SSAA, буфери орбіти й BLA, копія опорної орбіти помічника), відносно обсягу карти.
live-card-memory-good = Рівно під час наближення; сходинками вгору й назад униз, коли опорні орбіти зʼявляються й зникають.
live-card-memory-bad-leak = RAM зростає й ніколи не зменшується: витік.
live-card-memory-bad-vram = VRAM майже на розмір карти: виділення можуть не вдатися, або пристрій може бути втрачено.

live-card-cores-what = Використання CPU на ядро, від зеленого через бурштиновий до червоного, і осцилограф, де кожен зайнятий потік обертається навколо своєї ролі: побудова опорних орбіт, пул оцінювання, помічник, побічні рендери.
live-card-cores-good = Одне-два ядра зайняті під час наближення; короткий сплеск на всіх ядрах, коли вибирається опорна точка.
live-card-cores-bad-every = Усі ядра зайняті секундами: побудова опорної орбіти на екстремальній глибині.
live-card-cores-bad-own = Власне ядро програми в червоній зоні: смикається саме вікно.

live-card-pixels-what = Пікселі, обчислені за секунду, у Mpx/s. Тьмяні: ще ітерують в обході. Яскраві: завершені.
live-card-pixels-good = Десятки Mpx/s під час руху.
live-card-pixels-bad = Майже нуль, поки зображення застаріле: GPU на щось чекає — на опорну орбіту чи на показник.

## Live render panel: the lamps' help cards

live-card-lamp-plan-what = Кадр сплановано: вибрано роздільність, запитану кількість ітерацій і бюджет, а також чи оновлювати.
live-card-lamp-plan-good = Мерехтить щоразу, коли змінюються вигляд чи налаштування.
live-card-lamp-plan-bad = Темна, поки ви наближаєтеся: потік інтерфейсу застряг.

live-card-lamp-dispatch-what = Цього кадру робота пішла на GPU.
live-card-lamp-dispatch-good = Світиться, поки щось змінюється; темна в бездіяльності.
live-card-lamp-dispatch-bad = Зайнята, хоча нічого не змінюється: програма не усталюється.

live-card-lamp-live-what = Зображення на екрані — власний рендер цього кадру.
live-card-lamp-live-note = Рідко світиться під час глибокого занурення, і це нормально: див. УТРИМАНО і РЕПРОЄКЦІЯ.
live-card-lamp-live-good = Світиться на малих глибинах, де кадр дешевий.

# "Prefer detail" is a setting's name.
live-card-lamp-held-what = Останнє завершене зображення на екрані, поки під ним складається наступне: оновлення глибокого наближення або, з «Віддавати перевагу деталям», наступна плитка чи вибірка суперсемплінгу нерухомого вигляду.
live-card-lamp-held-good-dive = Світиться відрізками під час глибоких наближень, що перериваються лампою ПРИЙНЯТТЯ.
live-card-lamp-held-good-still = Рівно світиться, поки нерухомий вигляд уточнюється, а SSAA блимає.
live-card-lamp-held-bad = Світиться секундами під час наближення без ПРИЙНЯТТЯ: оновлення не завершуються.

live-card-lamp-reproj-what = Останнє зображення, розтягнуте під вигляд у русі, доки не буде готове нове.
live-card-lamp-reproj-good = Короткі спалахи, поки ви рухаєтеся.
live-card-lamp-reproj-bad = Довго світиться: що сильніше розтягнуто зображення, то воно розмитіше.

live-card-lamp-reading-what = З GPU повернувся показник: час проходу, діапазон утечі або вміст.
live-card-lamp-reading-good = Рівне мерехтіння під час рендерингу.
live-card-lamp-reading-bad = Довго темна під час рендерингу: виміри часу не повертаються, і бюджети вгадують.

live-card-lamp-pin-pass-what = Прохід зафіксованого обходу: зображення відрендерено на одну смугу ітерацій далі.
live-card-lamp-pin-pass-good = Серії проходів, що закінчуються лампою ПРИЙНЯТТЯ.
live-card-lamp-pin-pass-bad = Серії, що закінчуються лампою ВІДМОВА.

live-card-lamp-split-what = Прохід поділеного оновлення: частка плиток, відрендерена з повною кількістю ітерацій.
live-card-lamp-split-good = Короткі серії, що закінчуються лампою ПРИЙНЯТТЯ.
live-card-lamp-split-bad = Серії по 8, найбільше дозволене: оновлення ледь уміщається.

live-card-lamp-adopt-what = Завершене оновлення замінило зображення на екрані.
live-card-lamp-adopt-good = Кілька разів на секунду під час наближення.
# "Picture age" names the instrument (its title, PICTURE AGE).
live-card-lamp-adopt-bad = Рідко під час наближення: зображення старіє (див. ВІК ЗОБРАЖЕННЯ).

live-card-lamp-abandon-what = Оновлення, що виконувалося, відкинуто: вигляд зсунувся надто далеко, воно надто застаріло або його замінило новіше. Побудова опорної орбіти, яку обігнав вигляд, засвічує лампу тьмяно.
live-card-lamp-abandon-good = Рідко або один раз одразу після стрибка.
live-card-lamp-abandon-bad = Часто: робота марна, і зображення лишається старим.

live-card-lamp-ref-live-what = Опорна орбіта будується для вигляду як він є: у нього немає орбіти, він покинув ту, що мав, або йому потрібна довша.
live-card-lamp-ref-live-good = Короткі спалахи.
live-card-lamp-ref-live-bad-long = Довгі відрізки: глибокі вигляди чекають на неї.
live-card-lamp-ref-live-bad-lookahead = Часто світиться під час занурення, поки ОПОРА НАПЕРЕД темна: побудова наперед не встигає.

live-card-lamp-ref-ahead-what = Опорна орбіта наперед будується глибше за вигляд, туди, куди прямує наближення.
live-card-lamp-ref-ahead-good = Регулярні спалахи під час занурення.
live-card-lamp-ref-ahead-bad = Темна під час глибокого занурення: вигляд обжене свої опорні орбіти.

live-card-lamp-ref-hold-what = Попереднє завантаження для зупинки туру: опорна орбіта для наступного ключового кадру із зупинкою, побудована під час плавного руху до нього, щоб відтворення ніколи не чекало.
live-card-lamp-ref-hold-note = Темна поза відтворенням туру.
live-card-lamp-ref-hold-good = Спалах перед кожною зупинкою в турі.
live-card-lamp-ref-hold-bad = Світиться протягом усієї зупинки: попереднє завантаження запізнюється, і тур на нього чекає.

live-card-lamp-series-what = Апроксимація рядом для нової опорної орбіти, що дає змогу кожному пікселю пропустити перші ітерації.
live-card-lamp-series-good = Коротко.
live-card-lamp-series-bad = Довго на глибині: затримує встановлення опорної орбіти.

live-card-lamp-worker-what = Другий GPU доставив кадр.
live-card-lamp-worker-good = Рівні спалахи, коли допоміжний GPU ввімкнено.
live-card-lamp-worker-bad = Темна, хоча помічник увімкнений: він не постачає кадрів.

live-card-lamp-tile-what = Плитку усталення або її стрічку надіслано на GPU.
live-card-lamp-tile-good = Сплеск після зупинки, потім тиша.
live-card-lamp-tile-bad = Ніколи не згасає: сітка весь час перезапускається, бо щось постійно змінює вигляд чи налаштування.

live-card-lamp-ssaa-what = До усталеного зображення додано вибірку суперсемплінгу.
live-card-lamp-ssaa-good = Серія спалахів після усталення, потім темно.
live-card-lamp-ssaa-bad = Перезапускається знову й знову: усталенню постійно заважають.

live-card-lamp-palette-what = Нормалізація кольорів прийняла новий діапазон утечі, тож кольори стежать за наближенням.
live-card-lamp-palette-good = Мерехтить під час наближення.
live-card-lamp-palette-bad = Темна під час наближення: діапазон палітри заморожено, і кольори стають пласкими й низькоконтрастними.

live-card-lamp-mode-what = Арифметика змінилася: пряма, df32 або floatexp.
live-card-lamp-mode-good = Один спалах на межі, як-от 2^93.
live-card-lamp-mode-bad = Блимає туди й назад: вигляд «полює» вздовж межі.

## The Render client window (File ▸ Render client…): lending this machine's GPU to a render farm

farm-client-title = Клієнт рендерингу
farm-client-intro = Рендерте кадри для рендер-ферми. Контролер — Інструменти ▸ Рендеринг на фермі… на іншому компʼютері — показує свою адресу й ключ ферми. Цей компʼютер не відкриває жодного порту: він сам підключається до контролера.
# A field label, and the label of the status row that names the controller.
farm-client-controller = Контролер
farm-client-find = Знайти
farm-client-find-hover = Шукати контролери в цій мережі (контролер відповідає, поки його ферма очікує підключень)
farm-client-key = Ключ ферми
# A checkbox beside the masked farm key: show it in clear.
farm-client-show-key = Показати
# A field label (the name this machine goes by), and the label of the status row about this machine.
farm-client-machine = Цей компʼютер
farm-client-machine-hover = Назва, під якою контролер показує цей компʼютер.
farm-client-share = Спільний диск
# Greyed hint inside the empty field; the path is an example.
farm-client-share-hint = необовʼязково — напр. \\fileserver\share
farm-client-share-hover = Шлях цього компʼютера до спільного диска, який використовує контролер (режим спільного диска): кадри записуються туди, а не надсилаються через зʼєднання. Залиште порожнім, щоб надсилати кожен кадр.
farm-client-gpu = Відеокарта
# The label of the row with the idle option.
farm-client-when = Коли
# A checkbox followed by a number of minutes ("Only when idle for [5] min").
farm-client-when-idle = Лише після бездіяльності протягом
farm-client-when-idle-hover = Брати роботу лише тоді, коли ніхто не торкався клавіатури чи миші цього компʼютера стільки часу, або поки його екран заблоковано. Щойно хтось починає ним користуватися, поточний кадр одразу зупиняється й повертається фермі.
# A collapsible section: the largest job this machine accepts.
farm-client-limits = Обмеження
farm-client-limits-note = Завдання, що вимагає більшого, відхиляється, а не урізається.
farm-client-limit-size = Найбільший кадр
farm-client-limit-ss = Найбільший суперсемплінг
farm-client-limit-iter = Найбільше ітерацій

## Render client: the graphics-card choice

farm-client-gpu-system = Вибір системи
farm-client-gpu-every = Усі відеокарти
# $count: this machine's graphics cards.
farm-client-gpu-every-n = Усі відеокарти ({ $count })
# $count: this machine's graphics cards.
farm-client-gpu-every-each = Усі відеокарти ({ $count }) — по сеансу на кожну
# A card the window cannot name. $number: its number (or numbers) as --list-adapters gives it.
farm-client-gpu-adapter = Адаптер { $number }
farm-client-gpu-finding = Пошук відеокарт цього компʼютера…
# "NAME · GPU 1" is how such a machine is named in the controller's list: keep it as it is.
farm-client-gpu-hover = Яка відеокарта рендерить кадри ферми. Якщо вибрано всі карти, кожна приєднується до ферми як окремий компʼютер — «NAME · GPU 1», «NAME · GPU 2», — а «Призупинити» й «Відʼєднатися» діють на всі.

## Render client: looking for controllers on the network (the Find button)

farm-client-finding = Пошук контролерів у цій мережі…
# $port: the UDP port number.
farm-client-found-none = Жоден контролер не відповів. Контролер відповідає, поки його ферма очікує підключень; його брандмауер має пропускати вхідний UDP { $port }.
farm-client-found-pick = У цій мережі — клацніть контролер, щоб використати його:
# $identity: the controller's identity fingerprint.
farm-client-found-hover = Відбиток { $identity } — використати цей контролер (ключ ферми все одно потрібен)
# After a warning sign. $version and $git: this app's version and build.
farm-client-found-other-build = Інша збірка, ніж ця ({ $version } { $git }): вона відхилить цей компʼютер.

## Render client: what is wrong with what was typed, or with starting the client

farm-client-bad-address = Введіть адресу контролера як HOST:PORT, так, як її показує його вікно (наприклад, 192.168.1.20:46733).
# $why: what is wrong with the key (in English).
farm-client-bad-key = Ключ ферми: { $why }
# $why: what is wrong with the name (in English).
farm-client-bad-name = Назва: { $why }
farm-client-no-config = Немає теки конфігурації, де можна зберігати ключ ферми.
# $why: the system's reason.
farm-client-start-failed = Не вдалося запустити клієнт рендерингу: { $why }

## Render client: the state, in the status heading

farm-client-phase-connecting = Підключення…
farm-client-phase-checking = Виконується самоперевірка…
farm-client-phase-idle = Підключено — очікування роботи
farm-client-phase-rendering = Рендеринг
farm-client-phase-in-use = Очікування — цим компʼютером хтось користується
farm-client-phase-paused = Призупинено
farm-client-phase-retrying = Контролер недоступний — повторна спроба
# $seconds: the wait before the next try.
farm-client-phase-retrying-in = Контролер недоступний — повторна спроба через { $seconds } s
farm-client-phase-disconnected = Відʼєднано
farm-client-phase-ended = Завершено

## Render client: the line under the status heading

# $address: the controller's address (HOST:PORT).
farm-client-detail-connecting = Підключення до { $address }…
# $controller: the controller's name.
farm-client-detail-checking = Підключено до «{ $controller }» — виконується самоперевірка
# $controller: the controller's name.
farm-client-detail-approval = Підключено до «{ $controller }» — очікування, поки його користувач схвалить цей компʼютер
# $controller: the controller's name.
farm-client-detail-idle = Підключено до «{ $controller }» — простій
# $why: the check that failed and why (in English).
farm-client-detail-check-failed = Самоперевірка не пройшла — { $why }
# $why: the controller's reason (in English).
farm-client-detail-refused = Відхилено контролером: { $why }
# $why: the controller's reason (in English).
farm-client-detail-closed = Контролер закрив зʼєднання: { $why }
# $why: the controller's reason (in English).
farm-client-detail-ended = Контролер завершив зʼєднання: { $why }
# $job: the job's name.
farm-client-detail-bundle = Завдання «{ $job }» — отримання його пакета
# $job: the job's name.
farm-client-detail-job-waiting = Завдання «{ $job }» — очікування кадрів для рендерингу
# $why: the limit the job is over (in English).
farm-client-detail-job-refused = Це завдання відхилено: { $why }
farm-client-detail-job-over = Завдання завершено
# $first and $last: frame numbers.
farm-client-detail-rendering = Рендеринг кадрів { $first }–{ $last }
# $time: how long nobody must use the machine ("90 s", "5 min").
farm-client-detail-in-use = Очікування — цим компʼютером хтось користується; робота відновиться, коли ним ніхто не користуватиметься протягом { $time }
farm-client-detail-paused = Призупинено — контролер повідомлено
farm-client-detail-pausing = Призупинення — завершується поточний кадр
farm-client-detail-cancelled = Поточний кадр скасовано; призупинено
# A machine with several graphics cards. $rendering: the cards rendering now; $total: all its cards (two or more).
farm-client-rendering-on-cards =
    { $total ->
        [one] Рендеринг на { $rendering } з { $total } відеокарти
        [few] Рендеринг на { $rendering } з { $total } відеокарт
        [many] Рендеринг на { $rendering } з { $total } відеокарт
       *[other] Рендеринг на { $rendering } з { $total } відеокарт
    }

## Render client: the status rows

# The value of the Controller row. $name: the controller's name; $address: HOST:PORT; $identity: its identity fingerprint.
farm-client-row-controller-value = «{ $name }» · { $address } · відбиток { $identity }
# The value of the This machine row. $name: this machine's name; $identity: its identity fingerprint.
farm-client-row-machine-value = «{ $name }» · відбиток { $identity }
farm-client-row-gpu = GPU
# One graphics card's row. $slot: the card's number.
farm-client-row-gpu-n = GPU { $slot }
farm-client-row-job = Завдання
# The frames handed to this machine.
farm-client-row-run = Серія
farm-client-row-sent = Надіслано
# The Run row, in parts joined with " · ": the run, the frame in progress, its time. $first and $last: frame numbers.
farm-client-run-frames = кадри { $first }–{ $last }
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-client-run-frame = кадр { $frame } ({ $index } з { $count })
# $time: the time spent on it so far ("2.1 s").
farm-client-run-frame-time = { $time } на цей кадр
# One graphics card's run. $frame: the frame in progress; $first and $last: the run's frames.
farm-client-session-frame = кадр { $frame } з { $first }–{ $last }
farm-client-session-no-card = відеокарта ще невідома
# The Sent row. $count: frames sent; $time: the mean time per frame ("2.1 s").
farm-client-sent =
    { $count ->
        [one] { $count } кадр · { $time } на кадр
        [few] { $count } кадри · { $time } на кадр
        [many] { $count } кадрів · { $time } на кадр
       *[other] { $count } кадру · { $time } на кадр
    }
# The Sent row of a machine with several cards. $count: frames sent; $split: each card's share ("812 + 422"); $time: the mean time per frame.
farm-client-sent-split =
    { $count ->
        [one] { $count } кадр ({ $split }) · { $time } на кадр
        [few] { $count } кадри ({ $split }) · { $time } на кадр
        [many] { $count } кадрів ({ $split }) · { $time } на кадр
       *[other] { $count } кадру ({ $split }) · { $time } на кадр
    }
# After a warning sign. $note: how the GPUs differ, and both GPUs (in English).
farm-client-gpu-note = На цьому компʼютері: { $note }. Його кадри трохи відрізнятимуться від кадрів контролера, що може бути помітно в кадрі із зупинкою.
# A collapsible section: the self-check's results.
farm-client-self-check = Самоперевірка
farm-client-thumb-hover = Останній кадр, надісланий цим компʼютером
farm-client-resume = Продовжити
farm-client-resume-hover = Знову брати роботу
farm-client-pause = Призупинити
farm-client-pause-hover = Завершити поточний кадр і більше не брати роботи. Контролер передасть кадри цього компʼютера іншим.
farm-client-cancel-frame = Скасувати кадр
farm-client-cancel-frame-hover = Зупинити поточний кадр зараз і призупинитися; контролер відрендерить його деінде.
farm-client-live-paused = Живий вигляд призупинено, поки цей компʼютер рендерить кадри ферми.
farm-client-close = Закрити
farm-client-close-active-hover = Закрити вікно; цей компʼютер лишається у фермі (знову відкрийте вікно з меню «Файл»)
farm-client-close-hover = Закрити вікно
farm-client-disconnect = Відʼєднатися
farm-client-disconnect-hover = Вийти з ферми: поточний кадр буде повернуто
farm-client-connect = Підключитися
farm-client-connect-hover = Підключитися до контролера й рендерити для нього кадри

## The Render on farm window (Tools ▸ Render on farm…): the controller of a render farm

farm-ctl-title = Рендеринг на фермі
farm-ctl-out = Тека результатів
farm-ctl-browse = Огляд…
farm-ctl-out-note = Кадри, а також стан завдання в farm/: запустіть те саме завдання ще раз, щоб продовжити його.
farm-ctl-size = Розмір
farm-ctl-size-custom = Власний…
# The size list's choice when the size is typed in. $width and $height: pixels.
farm-ctl-size-custom-value = Власний — { $width }×{ $height }
farm-ctl-fps = Кадрів / секунду
farm-ctl-ss = Суперсемплінг
farm-ctl-port = Слухати порт
farm-ctl-port-hover = Клієнти підключаються до цього порту. Дозвольте його в брандмауері для вашої локальної мережі.
farm-ctl-share = Спільний диск
# Greyed hint inside the empty field.
farm-ctl-share-hint = необовʼязково — шлях до нього на цьому компʼютері
farm-ctl-share-hover = Режим спільного диска: клієнти, що мають той самий диск, записують на нього свої кадри замість надсилати їх через зʼєднання, а цей компʼютер перевіряє там кожен кадр. Клієнти без цього диска надсилають свої.
# A label followed by a number of clients and two checkboxes.
farm-ctl-start-with = Почати з
# The unit after the number in the box ("Start with [3] clients"). $count: that number.
farm-ctl-start-clients =
    { $count ->
        [one] клієнтом
        [few] клієнтами
        [many] клієнтами
       *[other] клієнта
    }
farm-ctl-start-clients-hover = Завдання починається, щойно стільки компʼютерів підключилися й пройшли самоперевірку; інші можуть приєднатися пізніше.
farm-ctl-local = Також рендерити на цьому компʼютері
farm-ctl-local-hover = Цей компʼютер рахується як один клієнт.
farm-ctl-sharing = Спільні опорні орбіти
farm-ctl-sharing-hover = Побудова опорної орбіти глибокого кадру може тривати від кількох хвилин до кількох годин. Якщо вона спільна, її будують один раз і передають іншим компʼютерам. Спільна опорна орбіта — інша, але так само правильна: на екстремальній глибині кілька пікселів можуть відрізнятися від кадру, що побудував власну.
farm-ctl-key = Ключ ферми
# A checkbox beside the shortened farm key: show all of it.
farm-ctl-show-key = Показати
farm-ctl-copy-key = Копіювати ключ
farm-ctl-copy-key-hover = Вставте його у вікно «Клієнт рендерингу» кожного клієнта
# In place of the farm key, when there is none.
farm-ctl-no-key = ще немає
farm-ctl-make-key = Створити ключ
farm-ctl-make-key-hover = Клієнтам він потрібен, щоб приєднатися; його також буде створено, коли ви натиснете «Рендерити»
# What to do on the other machines. $address: this controller's HOST:PORT.
farm-ctl-steps = На кожному клієнті: Файл ▸ Клієнт рендерингу…, контролер { $address } і ключ ферми.
# The same when this machine's address is not known. $port: the port number.
farm-ctl-steps-port = На кожному клієнті: Файл ▸ Клієнт рендерингу…, адреса цього компʼютера й порт { $port }, а також ключ ферми.
# The size of the job. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
farm-ctl-summary =
    { $count ->
        [one] { $count } кадр · { $seconds } s туру · { $width }×{ $height } ss{ $ss }
        [few] { $count } кадри · { $seconds } s туру · { $width }×{ $height } ss{ $ss }
        [many] { $count } кадрів · { $seconds } s туру · { $width }×{ $height } ss{ $ss }
       *[other] { $count } кадру · { $seconds } s туру · { $width }×{ $height } ss{ $ss }
    }
farm-ctl-strip-hover = Кожен кадр завдання, зліва направо: готовий, рендериться, ще в черзі, покинутий.
farm-ctl-no-tour = Тур не завантажено.
# $folder: the folder; $why: the system's reason.
farm-ctl-out-error = Тека результатів { $folder }: { $why }
# $why: the system's reason.
farm-ctl-start-failed = Не вдалося запустити ферму: { $why }
# $code: the controller process's exit code, as the code gives it.
farm-ctl-ended-exit = Контролер завершив роботу (код виходу { $code }).

## Render on farm: the state, in the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-phase-waiting = Очікування клієнтів рендерингу (підключено { $connected } з { $needed })
farm-ctl-phase-preparing = Підготовка завдання
farm-ctl-phase-rendering = Рендеринг
farm-ctl-phase-paused = Призупинено
farm-ctl-phase-complete = Завершено
farm-ctl-phase-stopped = Зупинено
farm-ctl-phase-ended = Закінчено

## Render on farm: the words after the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-detail-waiting = Очікування клієнтів рендерингу: підключено { $connected } з { $needed }
farm-ctl-detail-preparing = Вимірювання опорних значень нормалізації на цьому компʼютері, один раз для всіх клієнтів
farm-ctl-detail-no-space = Очікування вільного місця в теці результатів
farm-ctl-detail-paused = Призупинено: нова робота не роздається
farm-ctl-detail-not-rendered = Завершено, але деякі кадри не відрендерено — запустіть знову з тією самою текою, щоб продовжити
farm-ctl-detail-stopped = Зупинено — запустіть знову з тією самою текою, щоб продовжити
# $code: the controller process's exit code.
farm-ctl-detail-ended = Закінчено (код виходу { $code })
farm-ctl-detail-finishing = Завершення

## Render on farm: progress

# On the progress bar. $done: frames done; $total: frames in the job.
farm-ctl-progress =
    { $total ->
        [one] { $done } / { $total } кадр
        [few] { $done } / { $total } кадри
        [many] { $done } / { $total } кадрів
       *[other] { $done } / { $total } кадру
    }
# Added to the progress bar. $rate: frames per second, with decimals.
farm-ctl-progress-rate = { $rate } кадр/с
# Added to the progress bar. $time: the time still to go ("38m 10s").
farm-ctl-progress-eta = залишилося приблизно { $time }
# Facts under the progress bar, joined with " · ". $received and $sent: kilobytes per second.
farm-ctl-io = вхід { $received } KB/s · вихід { $sent } KB/s
# $space: free space on the output drive ("412 GB").
farm-ctl-free = вільно { $space }
# $time: the time since the job started ("38m 10s").
farm-ctl-elapsed = минуло { $time }
# $count: frames given up on.
farm-ctl-failed = не відрендерено: { $count }
# Reference orbits. $built: built for this job; $cached: found in the cache; $reused: reused between frames; $shared: passed between machines.
farm-ctl-refs = опорні орбіти: побудовано { $built }, з кешу { $cached }, повторно використано { $reused } · передано між компʼютерами { $shared }
# After a warning sign.
farm-ctl-storage-low = Тека результатів майже заповнена: нова робота не роздаватиметься, доки не зʼявиться місце.
# After a warning sign.
farm-ctl-gpus-differ = GPU у цій фермі різні. Їхні кадри трохи відрізняються, що може бути помітно в кадрі із зупинкою.
# After a warning sign. $count: the GPU classes in the farm.
farm-ctl-gpus-differ-classes =
    { $count ->
        [one] GPU у цій фермі різні ({ $count } клас GPU: різні GPU, драйвери чи пробні рендери). Їхні кадри трохи відрізняються, що може бути помітно в кадрі із зупинкою.
        [few] GPU у цій фермі різні ({ $count } класи GPU: різні GPU, драйвери чи пробні рендери). Їхні кадри трохи відрізняються, що може бути помітно в кадрі із зупинкою.
        [many] GPU у цій фермі різні ({ $count } класів GPU: різні GPU, драйвери чи пробні рендери). Їхні кадри трохи відрізняються, що може бути помітно в кадрі із зупинкою.
       *[other] GPU у цій фермі різні ({ $count } класу GPU: різні GPU, драйвери чи пробні рендери). Їхні кадри трохи відрізняються, що може бути помітно в кадрі із зупинкою.
    }
# $gpu: this machine's GPU, API and driver.
farm-ctl-this-gpu = Цей компʼютер: { $gpu }
# A machine whose GPU differs. $name: the machine; $note: how its GPU differs (in English); $adapter: its GPU.
farm-ctl-gpu-row = { $name }: { $note } — { $adapter }
# The same with its driver. $driver: the driver's name and version.
farm-ctl-gpu-row-driver = { $name }: { $note } — { $adapter }, драйвер { $driver }

## Render on farm: the table of machines

# Column headers (short: the table is narrow).
farm-ctl-col-machine = Компʼютер
farm-ctl-col-state = Стан
farm-ctl-col-gpu = GPU
farm-ctl-col-frames = Кадри
farm-ctl-col-per-frame = На кадр
# How the machine delivers its frames (a speed, or "shared drive").
farm-ctl-col-link = Звʼязок
# When the controller last heard from the machine.
farm-ctl-col-heard = Відгук
# In the machine's tooltip. $count: its frames that failed verification.
farm-ctl-strikes =
    { $count ->
        [one] { $count } кадр не пройшов перевірку
        [few] { $count } кадри не пройшли перевірку
        [many] { $count } кадрів не пройшли перевірку
       *[other] { $count } кадру не пройшли перевірку
    }
# In the Link column: it delivers its frames through the shared drive.
farm-ctl-link-share = спільний диск
# In the Heard column. $seconds: how long ago.
farm-ctl-heard-ago = { $seconds } s тому
farm-ctl-readmit = Допустити знову
farm-ctl-readmit-hover = Дозволити цьому компʼютеру знову приєднатися, коли він перепідключиться
farm-ctl-remove = Вилучити
farm-ctl-remove-hover = Вилучити цей компʼютер із завдання; його кадри перейдуть до інших
# The GPU column's tooltip.
farm-ctl-probe-same = Його пробний рендер ідентичний рендеру цього компʼютера.
# $count: the pixels that differ.
farm-ctl-probe-differs =
    { $count ->
        [one] Його пробний рендер відрізняється від рендеру цього компʼютера в { $count } пікселі: його кадри трохи відрізнятимуться від кадрів цього компʼютера.
        [few] Його пробний рендер відрізняється від рендеру цього компʼютера в { $count } пікселях: його кадри трохи відрізнятимуться від кадрів цього компʼютера.
        [many] Його пробний рендер відрізняється від рендеру цього компʼютера в { $count } пікселях: його кадри трохи відрізнятимуться від кадрів цього компʼютера.
       *[other] Його пробний рендер відрізняється від рендеру цього компʼютера в { $count } пікселя: його кадри трохи відрізнятимуться від кадрів цього компʼютера.
    }
farm-ctl-probe-unknown = Його пробний рендер ще не порівняно.
# $driver: the driver's name and version.
farm-ctl-gpu-driver = драйвер { $driver }
# After a warning sign. $note: how its GPU differs (in English: "a different GPU").
farm-ctl-gpu-note = { $note } порівняно з GPU цього компʼютера

## Render on farm: a machine's state, in the State column

# $count: its frames that failed verification.
farm-ctl-state-removed-bad =
    { $count ->
        [one] вилучено: { $count } поганий кадр
        [few] вилучено: { $count } погані кадри
        [many] вилучено: { $count } поганих кадрів
       *[other] вилучено: { $count } поганого кадру
    }
farm-ctl-state-removed = вилучено вами
farm-ctl-state-removing = вилучення — збирання його діагностики
farm-ctl-state-gone = зник
farm-ctl-state-leaving = виходить
farm-ctl-state-checking = перевірка (самоперевірка)
farm-ctl-state-paused = призупинено його користувачем
farm-ctl-state-parked = відкладено: двічі збій — повернеться, коли перепідключиться
farm-ctl-state-refused = відхилив це завдання (власні обмеження)
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-ctl-state-rendering = рендеринг { $frame } ({ $index }/{ $count })
farm-ctl-state-starting = початок серії
farm-ctl-state-idle = простій
farm-ctl-state-admitted = допущено — очікування завдання

## Render on farm: the buttons

# A collapsible section: the controller's messages.
farm-ctl-events = Події
farm-ctl-resume = Продовжити
farm-ctl-pause = Призупинити
farm-ctl-pause-hover = Не роздавати нової роботи; поточні кадри завершуються
farm-ctl-live-paused = Живий вигляд призупинено, поки цей компʼютер рендерить для ферми.
farm-ctl-close = Закрити
farm-ctl-close-active-hover = Закрити вікно; завдання продовжує виконуватися (знову відкрийте вікно з меню «Інструменти»)
farm-ctl-close-hover = Закрити вікно
farm-ctl-copy-command = Копіювати команду
farm-ctl-copy-command-hover = Скопіювати еквівалентний командний рядок
farm-ctl-stop = Зупинити
# The Stop button once a stop has been asked for and the job has not ended after 10 seconds.
farm-ctl-stop-now = Зупинити негайно
farm-ctl-stop-hover = Завершити завдання з наявними кадрами; повторний запуск того самого завдання продовжить його
farm-ctl-render = Рендерити
farm-ctl-render-hover = Почати очікувати клієнтів і рендерити на них тур

## The Render tour window (Tools ▸ Render tour…)

tour-render-title = Рендеринг туру
tour-render-out = Тека результатів
tour-render-browse = Огляд…
tour-render-out-hover = Повний шлях, куди записуються кадри.
tour-render-prefix = Префікс кадрів
# <prefix> stands for the prefix typed in the field.
tour-render-prefix-hover = Кадри записуються як <prefix>_00000.png
tour-render-size = Розмір
tour-render-size-custom = Власний…
# The size list's choice when the size is typed in. $width and $height: pixels.
tour-render-size-custom-value = Власний — { $width }×{ $height }
tour-render-fps = Кадрів / секунду
tour-render-fps-hover = Менше 1 fps — це нормально: вибірка довгого занурення з частотою 0,25 fps дає змогу переглянути глибокі контрольні зупинки, не рендерячи тисячі кадрів.
tour-render-ss = Суперсемплінг
tour-render-chapter = Розділ
tour-render-whole = Увесь тур
tour-render-chapter-hover = Рендеринг одного розділу зберігає ГЛОБАЛЬНУ нумерацію кадрів, тож його кадри просто стають на свої місця в повній послідовності.
tour-render-mp4 = Зібрати mp4
tour-render-mp4-hover = Після цього запускає ffmpeg (має бути в PATH). Кадри PNG зберігаються.
tour-render-overwrite = Перезаписувати наявні кадри
tour-render-overwrite-hover = Вимкнено: рендер зупиняється, щоб запитати, а звідси він цього зробити не може.
tour-render-resume = Продовжити перерваний рендер
tour-render-resume-hover = Залишити кадри, які вже є в теці результатів, і рендерити лише відсутні. Спершу перевіряється найновіший кадр і відкидається, якщо він неповний (кадр, на якому рендер обірвався, — той, який він записував), і так крок за кроком назад, доки не знайдеться добрий; теку з кадрами іншого розміру відхиляють, а не змішують із цією послідовністю.
# $count: this machine's graphics cards.
tour-render-all-gpus = Використовувати всі відеокарти ({ $count })
tour-render-all-gpus-hover = Рендерити одночасно на всіх відеокартах цього компʼютера через рендер-ферму на ньому самому (--gpus all). Кадри, їхні назви, продовження й mp4 — такі самі, як для однієї карти; рендериться весь тур, у власному порядку ферми. Різні моделі карт малюють кілька пікселів трохи по-різному: кадр із зупинкою та перехід-розчинення лишаються на одній карті.
tour-render-order = Порядок рендерингу
tour-render-sequential = Послідовний
tour-render-sequential-hover = Кадри 0, 1, 2, … по порядку (типово).
tour-render-progressive = Поступовий
tour-render-progressive-hover = Спершу попередній перегляд усього туру, потім уточнення до повної деталізації: спочатку рендеряться ключові кадри, а потім найбільші часові проміжки діляться навпіл, доки не буде готовий кожен кадр. Кадри в будь-якому разі потрапляють на свої правильні номери, тож продовження й mp4 працюють без змін — можна зупинитися раніше, переглянути грубий фліпбук і перезапустити з продовженням у будь-якому з порядків.
# The cost of the render. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
tour-render-summary =
    { $count ->
        [one] { $count } кадр · { $seconds } s туру · { $width }×{ $height } ss{ $ss }
        [few] { $count } кадри · { $seconds } s туру · { $width }×{ $height } ss{ $ss }
        [many] { $count } кадрів · { $seconds } s туру · { $width }×{ $height } ss{ $ss }
       *[other] { $count } кадру · { $seconds } s туру · { $width }×{ $height } ss{ $ss }
    }
# The disk space the frames will take (an estimate). $size: a size ("5.3 GB").
tour-render-disk = ≈{ $size } на кадри
# $size: the frames' estimated size; $free: the free space on that drive ("6.0 GB").
tour-render-disk-free = ≈{ $size } на кадри · вільно { $free }
# After a warning sign. $disk: the line above ("≈51.0 GB of frames · 6.0 GB free").
tour-render-disk-short = { $disk } — недостатньо місця для всієї послідовності
tour-render-separate = Рендерить сценарій З ДИСКА в окремому процесі, тож незбережені зміни не враховуються — і втрата GPU-пристрою там не потягне за собою цей сеанс.
# On the progress bar. $done: frames done; $total: frames to render.
tour-render-progress =
    { $total ->
        [one] { $done } / { $total } кадр
        [few] { $done } / { $total } кадри
        [many] { $done } / { $total } кадрів
       *[other] { $done } / { $total } кадру
    }
# Before the render's first progress line.
tour-render-starting = запуск…
tour-render-live-paused = Живий вигляд призупинено — рендер має GPU у своєму розпорядженні, тож завершується швидше й точно вимірює вартість власних кадрів.
tour-render-cancel = Скасувати
tour-render-cancel-hover = Закрити без рендерингу
tour-render-copy-command = Копіювати команду
tour-render-copy-command-hover = Скопіювати еквівалентний командний рядок у буфер обміну
tour-render-stop = Зупинити рендер
tour-render-render = Рендерити
tour-render-render-hover = Почати рендеринг послідовності кадрів
# $folder: the output folder.
tour-render-finished = Рендер завершено → { $folder }
# $why: the render's own reason (in English).
tour-render-failed = Помилка рендеру: { $why }
# $last: the render's last progress line (in English), or nothing.
tour-render-failed-or-stopped = Рендер завершився помилкою або його зупинено. { $last }
# $why: the system's reason.
tour-render-no-exe = Не вдається знайти виконуваний файл: { $why }
# $why: the system's reason.
tour-render-start-failed = Не вдалося запустити рендер: { $why }

## The Custom formula dialog (Fractal ▸ Custom formula…)
## The formula language itself stays as it is in every language: function names (sin, cosh, conj…),
## operators, variables (z, c, p1), and the formulas. Keep them as written inside these messages.

# The window's title.
formula-dialog-title = Власна формула
# A small line at the top of the window, above the formula.
formula-dialog-intro = Крок у виразах у стилі Fractint
# The two ways to show and edit the formula (top right of the window).
formula-dialog-textbook = Підручник
formula-dialog-textbook-hover = Показати формулу зверстаною, як у підручнику, і редагувати її в такому вигляді
formula-dialog-text = Текст
formula-dialog-text-hover = Редагувати формулу як текст
# The tooltip of the "LaTeX" button (the button's own text stays LaTeX).
formula-dialog-latex-hover = Скопіювати формулу як LaTeX — так, як її зверстано тут
# The drop-down list of example formulas (the collection).
formula-dialog-examples = Приклади
# Help under the formula in Textbook mode. The characters / ^ ( are what the user types: keep them.
formula-dialog-help-textbook = Вводьте як у тексті: / робить дріб із попереднього члена, ^ — показник степеня, ( — дужки. Стрілки переміщують по формулі; Tab переходить до наступного порожнього поля. Червоний рядок не читається: клацніть його, щоб редагувати як текст.
# Help under the formula in Text mode. "; starts a comment": the ; is what the user types.
formula-dialog-help-text = Вводьте текст або користуйтеся екранною клавіатурою — на ній є кожна назва, яку знає мова формул. Назви доповнюються під час введення (Tab). Інструкції розділяються новим рядком або комою; ; починає коментар.
# The syntax check, when the formula reads.
formula-dialog-reads = Читається правильно.
# The syntax check in Textbook mode, when the formula has an empty box to fill in.
formula-dialog-empty-box = Заповніть порожнє поле.
# The syntax check in Textbook mode. $line: the line number; $message: what is wrong (from the formula parser, in English).
formula-dialog-error-line = Рядок { $line }: { $message }
# One function written with another notation's name, and the language's name for it: "ln to log".
# $from: the name as written (ln); $to: the formula language's name (log).
formula-respell-pair = { $from } на { $to }
# A button that rewrites them. $changes: one or more formula-respell-pair, joined with ", ".
formula-respell = Замінити { $changes }
formula-respell-hover = Записати ці функції так, як їх називає мова формул
# Shown when Apply finds a parameter field that is not a number. $name: the parameter (p1 … p5).
formula-param-not-numbers = { $name } не є парою чисел
# The Apply button (also in the Formula library window).
formula-apply = Застосувати
formula-dialog-apply-hover = Скомпілювати формулу й показати її
# Shown beside Apply while the formula is being compiled.
formula-dialog-compiling = Компіляція формули для GPU…
# The greyed hint in an empty name box (also in the Formula library window).
formula-name-hint = назва (необовʼязково)
# The save button: "Update" when the library already has a formula of that name.
formula-dialog-update-in-library = Оновити в бібліотеці
formula-dialog-save-to-library = Зберегти в бібліотеку
formula-dialog-save-hover = Зберегти цю формулу та її параметри в бібліотеці формул
formula-dialog-save-disabled-hover = Зберегти можна лише формулу, що читається правильно
# Opens the Formula library window.
formula-dialog-library = Бібліотека…
formula-dialog-library-hover = Збережені формули: застосовуйте, редагуйте, імпортуйте та експортуйте їх

## The Custom formula dialog's keypad
## The keys' captions are formula tokens and stay as they are; these are their tooltips.

# The keypad's third tab (the other two are "123" and "f(z)"). A short word.
formula-keypad-tab-names = назви
# The tabs' tooltips.
formula-keypad-tab-basic-hint = Числа, оператори й найуживаніші назви
formula-keypad-tab-functions-hint = Функції
formula-keypad-tab-names-hint = Усі змінні, параметри й константи, а також синтаксис інструкцій
# The key z.
formula-key-z = Ітероване значення
# The key c.
formula-key-c = Піксель (у режимі Жюліа — стала Жюліа)
# The key p1 on the first tab.
formula-key-p1-below = Параметр 1 — задається в полях нижче
# The keys p1 … p5. $n: the parameter's number.
formula-key-param = Параметр { $n }
formula-key-multiply = Множення
formula-key-divide = Ділення
# The key that types ^2.
formula-key-square-after = Квадрат (після значення: z^2)
# The key that types ^.
formula-key-power = Степінь: цілий, дійсний або комплексний показник
formula-key-square-root = Квадратний корінь
# The key that types |…|: in this language it is the squared modulus, as in Fractint.
formula-key-squared-modulus = КВАДРАТ модуля |z|² — як у Fractint
formula-key-add = Додавання
formula-key-subtract = Віднімання (або зміна знака)
# The key that types =.
formula-key-assign = Присвоєння: z = … або тимчасова змінна t = …
# The key that starts a new line (first tab).
formula-key-next-statement-comma = Наступна інструкція (кома теж підходить)
# The key "del".
formula-key-backspace = Видалити символ перед курсором (або виділене)
# The key e.
formula-key-euler = Число Ейлера
# The key that types (0.5, 0.5).
formula-key-complex-constant = Комплексна стала: (дійсна, уявна)
# The key that types a comma.
formula-key-comma = Розділяє інструкції або частини комплексної сталої
formula-key-left = Курсор ліворуч
formula-key-right = Курсор праворуч
# The function keys (the f(z) tab), one per function.
formula-key-sin = Синус
formula-key-cos = Косинус
formula-key-tan = Тангенс
formula-key-cotan = Котангенс
formula-key-exp = Експонента e^z
formula-key-sinh = Гіперболічний синус
formula-key-cosh = Гіперболічний косинус
formula-key-tanh = Гіперболічний тангенс
formula-key-cotanh = Гіперболічний котангенс
formula-key-log = Натуральний логарифм, ln (головна гілка)
formula-key-sqr = Квадрат: sqr(z) = z²
formula-key-sqrt = Квадратний корінь (головне значення)
formula-key-recip = Обернене значення 1/z
formula-key-conj = Комплексне спряження
formula-key-flip = Поміняти місцями дійсну й уявну частини
formula-key-abs = Абсолютне значення ОБОХ частин: |x| + i|y|
formula-key-cabs = Модуль |z| (дійсне число)
formula-key-real = Дійсна частина
formula-key-imag = Уявна частина як дійсне число
# "fn" is Fractint's name for a function slot: keep it.
formula-key-ident = Тотожність (заповнювач fn у Fractint)
# The key "pixel".
formula-key-pixel = Те саме, що c (назва у Fractint)
# The key "z =".
formula-key-set-z = Почати інструкцію, що задає z
# The key "t =".
formula-key-temporary = Тимчасова змінна: підходить будь-яка нова назва (t, w, a2, …)
# The key "; …".
formula-key-comment = Коментар до кінця рядка
# The key that starts a new line (names tab).
formula-key-next-statement = Наступна інструкція

## The formula's completion list (names offered as you type)

# Beside a name the formula itself assigns.
formula-completion-variable = Змінна, яку задає ця формула
# Beside a function typed with another notation's name.
# $what: what the function is, in English (from the formula parser: "The natural logarithm");
# $other: the other notation's name (ln); $ours: the formula language's name (log).
formula-completion-other-spelling = { $what } — { $other } тут записується як { $ours }
# Under a list cut short. $count: the names not shown.
formula-completion-more = … ще { $count }
# The last line of the list. Tab, Enter and Esc are key names.
formula-completion-keys = Tab або Enter — доповнити, Esc — закрити

## The Custom formula dialog's Textbook mode

# The tooltip of a line that does not read. $why: what is wrong (from the formula parser, in English, or formula-textbook-fractint-sections).
formula-textbook-unread-hover = Не читається: { $why }. Клацніть, щоб редагувати як текст.
# Why a line with Fractint's sections is shown as text in Textbook mode.
formula-textbook-fractint-sections = розділи Fractint (':' ініціалізації, порівняння, a = b = …) редагуються як текст

## The Formula library window (Fractal ▸ Formula library…)

# The window's title.
formula-library-title = Бібліотека формул
# The window's two tabs. $count: the formulas in each.
formula-library-mine = Мої формули ({ $count })
formula-library-mine-hover = Формули, які ви зберегли або імпортували
formula-library-collection = Колекція ({ $count })
formula-library-collection-hover = Формули, що постачаються з Fractadyne, кожна з виглядом, з якого можна почати
# A small marker before the name of the formula the view shows now. A short word.
formula-library-showing = показано
# The tooltip of Apply on a formula of the collection.
formula-library-apply-shipped-hover = Показати її на початковому вигляді
# A row's Edit button (after a pencil icon).
formula-library-edit = Редагувати
formula-library-edit-hover = Відкрити її в діалозі «Власна формула», не застосовуючи
# A collection row's button that copies the formula into "My formulas".
formula-library-copy-to-mine = Копіювати до моїх
formula-library-copy-to-mine-hover = Додати її до ваших формул, щоб зберегти чи змінити
formula-library-close = Закрити
# Saves the formula the view shows, under the name typed beside it.
formula-library-add-current = Додати поточну формулу
formula-library-add-current-hover = Зберегти формулу, яку показує вигляд
# "Fractal > Custom formula…" is the menu path: write it as the menus are named.
formula-library-add-current-disabled-hover = Спершу застосуйте власну формулу (Фрактал > Власна формула…)
formula-library-import = Імпорт…
formula-library-import-hover = Додати формули з файлу формул; тут нічого не замінюється
formula-library-export-all = Експортувати все…
formula-library-export-all-hover = Записати всі формули звідси в один файл, щоб поділитися чи зберегти
# The filter box: its label, and the greyed hint inside it.
formula-library-filter = Фільтр
formula-library-filter-hint = фільтр за назвою або текстом
# Shown while the user has no saved formulas.
formula-library-empty = Збережених формул ще немає. Збережіть формулу з діалогу «Власна формула», додайте ту, яку показує вигляд, або імпортуйте файл (наш або .frm Fractint).
# The tooltip of Apply on one of the user's formulas: with a view saved, and without one.
formula-library-apply-view-hover = Показати цю формулу на збереженому з нею вигляді
formula-library-apply-hover = Показати цю формулу
# The tooltip of a row's save icon.
formula-library-export-one-hover = Експортувати цю формулу у файл
# A row asks before deleting: the question, then its two buttons. "Delete" is also the delete icon's tooltip.
formula-library-delete-ask = Видалити її?
formula-library-delete = Видалити
formula-library-keep = Залишити
# Under a list cut short. $count: the formulas not listed.
formula-library-more = …і ще { $count }: введіть текст у фільтр, щоб знайти потрібну.
formula-library-no-match = Нічого не відповідає фільтру.
# Under a saved formula this version cannot read. $why: what is wrong (from the formula parser, in English).
formula-library-unread = Не читається в цій версії: { $why }

## Formula library notices

# $why: the system's reason, in English.
formula-library-save-failed = Не вдалося зберегти бібліотеку формул: { $why }
formula-library-nothing-to-save = Нічого зберігати: формула порожня.
# $name: the formula's name in the library.
formula-library-updated = «{ $name }» оновлено в бібліотеці формул.
formula-library-saved = «{ $name }» збережено в бібліотеці формул.
# $name: the formula's name; $why: what is wrong.
formula-library-apply-failed = «{ $name }» не вдається застосувати: { $why }
# Copying a formula of the collection into "My formulas". $name: its name; $as_named: the name it was given (the same, or "Name (2)" when the name was taken).
formula-library-copy-exists = «{ $name }» уже є серед ваших формул.
formula-library-copied = «{ $name }» скопійовано до ваших формул як «{ $as_named }».
# $name: the formula's name.
formula-library-deleted = «{ $name }» видалено з бібліотеки формул.
# Why the library could not be saved (follows formula-library-save-failed).
formula-library-no-config-dir = немає теки конфігурації
# At startup, when the library file cannot be read. $why: the reason, in English; $path: a file's full path.
formula-library-unreadable-kept = Не вдалося прочитати бібліотеку формул ({ $why }). Її збережено як { $path }, а нову порожню бібліотеку створено.
formula-library-unreadable-stuck = Не вдалося ні прочитати бібліотеку формул ({ $why }), ні перемістити її вбік: скопіюйте { $path } у безпечне місце, перш ніж зберігати формулу, бо збереження замінить цей файл.

## Formula library: Import and Export
## A formula file's contents stay in English; this is what the windows and notices around them say.

# File-type names in the open and save dialogs (the extensions are added by the system).
formula-filter-formula-files = Файли формул
formula-filter-fractadyne = Формули Fractadyne
formula-filter-fractint = Формули Fractint
formula-filter-all-files = Усі файли
# Why a file cannot be imported (they follow formula-import-failed).
formula-import-too-large = занадто великий, щоб бути файлом формул
formula-import-not-text = не текстовий файл
formula-import-no-formulas = у ньому немає формул
# $why: the TOML reader's message, in English.
formula-file-not-formulas = не файл формул ({ $why })
# $format: what the file says it is ("fractadyne-gradients").
formula-file-other-format = файл «{ $format }», а не файл формул
# "[[formula]]" is the file's own syntax: keep it.
formula-file-no-formulas = у ньому немає формул (файл формул перелічує їх як таблиці [[formula]])
# $file: the file's name; $why: one of the reasons above, or the system's (in English).
formula-import-failed = Не вдалося імпортувати «{ $file }»: { $why }.
# After an import that added nothing. $file: the file's name.
formula-import-nothing-new = У «{ $file }» нічого нового.
# After an import. $count: the formulas added; $file: the file's name.
formula-import-added =
    { $count ->
        [one] Імпортовано { $count } формулу з «{ $file }».
        [few] Імпортовано { $count } формули з «{ $file }».
        [many] Імпортовано { $count } формул з «{ $file }».
       *[other] Імпортовано { $count } формули з «{ $file }».
    }
# The same two, followed by what else happened. $notes: one or more of the formula-import-note-… messages below, joined with "; ".
formula-import-nothing-new-notes = У «{ $file }» нічого нового: { $notes }.
formula-import-added-notes =
    { $count ->
        [one] Імпортовано { $count } формулу з «{ $file }»: { $notes }.
        [few] Імпортовано { $count } формули з «{ $file }»: { $notes }.
        [many] Імпортовано { $count } формул з «{ $file }»: { $notes }.
       *[other] Імпортовано { $count } формули з «{ $file }»: { $notes }.
    }
# Notes (clauses, no final full stop). $count: the formulas already in the library, skipped.
formula-import-note-duplicates =
    { $count ->
        [one] { $count } формула вже була в бібліотеці
        [few] { $count } формули вже були в бібліотеці
        [many] { $count } формул уже були в бібліотеці
       *[other] { $count } формули вже були в бібліотеці
    }
# One formula was renamed because its name was taken. $from: its name in the file; $to: the name it was given ("Name (2)").
formula-import-note-renamed-one = назву «{ $from }» було зайнято, тож тепер це «{ $to }»
# $count: the formulas renamed because their names were taken (2 or more).
formula-import-note-renamed =
    { $count ->
        [one] { $count } формулу перейменовано, бо назву було зайнято
        [few] { $count } формули перейменовано, бо назви було зайнято
        [many] { $count } формул перейменовано, бо назви було зайнято
       *[other] { $count } формули перейменовано, бо назви було зайнято
    }
# $count: the entries skipped as empty or too long.
formula-import-note-skipped =
    { $count ->
        [one] { $count } запис був порожній або задовгий, його пропущено
        [few] { $count } записи були порожні або задовгі, їх пропущено
        [many] { $count } записів були порожні або задовгі, їх пропущено
       *[other] { $count } запису були порожні або задовгі, їх пропущено
    }
# A second sentence after importing a Fractint .frm file. $count: the entries that do not read; $why: the commonest reason (from the formula parser, in English).
formula-import-frm-unread =
    { $count ->
        [one] { $count } запис не читається в цій версії (найчастіше: { $why }).
        [few] { $count } записи не читаються в цій версії (найчастіше: { $why }).
        [many] { $count } записів не читаються в цій версії (найчастіше: { $why }).
       *[other] { $count } запису не читаються в цій версії (найчастіше: { $why }).
    }
# After exporting one formula. $name: the formula's name; $file: the file's name.
formula-export-one = «{ $name }» експортовано в «{ $file }».
# After exporting several. $count: how many; $file: the file's name.
formula-export-many =
    { $count ->
        [one] Експортовано { $count } формулу в «{ $file }».
        [few] Експортовано { $count } формули в «{ $file }».
        [many] Експортовано { $count } формул у «{ $file }».
       *[other] Експортовано { $count } формули в «{ $file }».
    }
# $file: the file's name; $why: the system's reason, in English.
formula-export-failed = Не вдалося записати «{ $file }»: { $why }

## The control panel (right-hand side): header and section headings

# The panel's title, beside the button that hides it.
panel-title = Керування
panel-hide-hover = Сховати панель керування
# The button that brings the hidden panel back (top right of the view).
panel-show-hover = Показати панель керування
# Section headings.
panel-section-navigate = Навігація
panel-section-coloring = Розфарбування
panel-section-quality = Якість
panel-section-effects = Ефекти
panel-section-overlays = Накладання
panel-section-advanced = Додатково
# The last section: reference notes about the current fractal. $name: the fractal's name (Mandelbrot, Burning Ship…).
panel-section-about = Про { $name }
panel-section-performance = Продуктивність

## The control panel: Navigate
## Labels before a control sit in a column about 104 px wide (roughly 15 characters): a longer label pushes its control to the right.

panel-zoom-speed = Швидкість наближення
panel-zoom-speed-hover = Швидкість безперервного наближення з утриманням Space (1× ≈ 2× за 1,5 s).
panel-snap-zoom = Притягувати до особливих точок
panel-snap-zoom-hover = Під час наближення коліщатком миші чи Space притягувати наближення до мініброта (або дійсної осі) поблизу курсора, щоб він ковзав під курсор і наближення приходило точно на нього. Що ближча особлива точка, то сильніше притягування. Лише Mandelbrot, один вигляд.
panel-click-zoom = Наближення клацанням
panel-click-zoom-hover = Коли ввімкнено, клацання лівою кнопкою у вигляді пірнає на коефіцієнт нижче (правою — відступає назад) із центруванням на точці клацання. Перетягування й далі пересуває вигляд; Shift+перетягування / перетягування правою кнопкою й далі наближають рамкою. Backspace скасовує клацання. У подвійному вигляді це працює на будь-якій панелі, а закріплення параметра c Жюліа, поки це ввімкнено, переходить на Ctrl+клацання. Утримуйте Shift, щоб показати під курсором лупу з виглядом, до якого приведе клацання, і клацніть, утримуючи його, щоб узяти цю точку; Shift+ПЕРЕТЯГУВАННЯ й далі наближає рамкою.
# The click-to-zoom factor, followed by buttons 2×, 4×, 10×…
panel-click-zoom-factor = Коефіцієнт
panel-click-zoom-snap = Притягувати до найближчого центру
panel-click-zoom-snap-hover = Після стрибка знайти найближче ядро мініброта й точно усталитися на ньому, щоб потрапити на особливу точку, а не поруч із нею. Масштаб не змінюється — його вибрало клацання. Лише Mandelbrot. Пошук іде у фоновому режимі: вигляд стрибає одразу й виправляється, коли надходить відповідь, а на екстремальній глибині це може тривати кілька секунд.

## The control panel: Coloring
## Labels before a control sit in a column about 104 px wide (roughly 15 characters).

panel-method = Метод
panel-method-hover = Як дані про втечу перетворюються на колір. «Смуги» / «нерівність трикутника» / «пастка орбіти» / «декомпозиція» розкривають структуру орбіти; «відстань» затінює за близькістю до межі.
panel-stripe-density = Щільність смуг
panel-stripe-tail = Лише хвіст (глибоке наближення)
panel-stripe-tail-hover = Усереднювати член смуг за останні N ітерацій кожної орбіти (експоненційне вікно), а не за всю орбіту. На екстремальній глибині кожна орбіта повторює опорну всі ітерації, крім останніх кількох сотень, тож середнє за всю орбіту дає один колір на весь вигляд; хвіст зберігає контраст. Орбіти, коротші за вікно, не зачіпаються. Також --stripe-tail N для --render.
panel-tail-length = Довжина хвоста
# The unit after the tail length's number: short for "iterations" (the slider reads "64 it").
panel-tail-length-unit = іт
panel-tail-length-hover = Довжина вікна в ітераціях. Короткі вікна (8–32) дають класичні виразні смуги на будь-якій глибині; довші знову усереднюються до плаского кольору, бо член смуг довгої хаотичної орбіти усереднюється.
panel-trap-shape = Форма пастки
panel-palette = Палітра
# The two colours of the Binary palette, each beside its colour swatch.
panel-in-set = У множині
panel-out-of-set = Поза множиною
# The two colours of the Duotone palette, each beside its colour swatch.
panel-shadow = Тінь
panel-highlight = Світло
panel-cycle = Цикл
panel-offset = Зсув
panel-normalize-hover = На екстремальній глибині кількості ітерацій до втечі охоплюють сотні тисяч, і фіксований «Цикл» обертає палітру тисячі разів між сусідніми пікселями — правильне щільне поле виглядає як шумова рябизна. Це переналаштовує палітру на виміряний діапазон утечі вигляду («Цикл» тоді задає, скільки обертів палітри його охоплює) ТОДІ, КОЛИ вигляд справді має аліасинг, — плавний вигляд лишається без змін, бо переналаштування змінило б картину марно. Позначте «Завжди підганяти діапазон» нижче, щоб переналаштовувати попри все. Лише плавний метод; відповідає --normalize.
# A checkbox indented under "Normalize deep colors" (also in the Color menu).
panel-fit-range = Завжди підганяти діапазон
panel-fit-range-hover =
    Переналаштовувати палітру на виміряний діапазон утечі вигляду, навіть коли вигляд не має аліасингу, — позначка вище переналаштовує лише тоді, коли щільне поле інакше виглядало б як рябизна.

    Використовуйте це, коли хочете, щоб палітра охоплювала те, що на екрані, а не щоб прибрати шум. Це все одно потребує вимірювання, тож нічого не робить мить-другу, поки вигляд не усталиться, і зовсім нічого на вигляді без пікселів, що втекли.
# A checkbox indented under "Always fit range" (also in the Color menu).
panel-log-scale = Логарифмічна шкала кольорів
panel-log-scale-hover =
    Розподіляти палітру за ЛОГАРИФМОМ значення утечі, а не лінійно. На глибині значення утечі скупчуються біля верхньої межі, тож лінійне відображення витрачає більшу частину палітри на тонку оболонку біля межі й робить пласким усе інше.

    Застосовується до нормалізованого відображення — саме його вмикає «Нормалізувати глибокі кольори» (або --normalize під час експорту). Сильно перекошений діапазон усе одно отримує логарифмічне відображення, позначено це чи ні.
# The same checkbox's tooltip while it is greyed out (normalization is off).
panel-log-scale-off-hover =
    Потребує нормалізації: це визначає, ЯК палітра відображається на виміряний діапазон утечі, тож коли «Нормалізувати глибокі кольори» вимкнено (і немає експорту з --normalize), немає відображення, яке це могло б змінити, і воно не змінило б жодного пікселя.

    Увімкніть «Нормалізувати глибокі кольори», щоб цим скористатися.
# The palette animation: its kind, then its speed.
panel-animate = Анімація
panel-anim-speed = Швидкість
panel-anim-speed-hover = Швидкість циклу: цикли зсуву кольору за секунду, або (Випадкові градієнти) зміни градієнта за секунду.
panel-shuffle-gradient = Перемішати градієнт

## The control panel: Quality

panel-auto-iter = Автомасштабувати ітерації з наближенням
# The iteration-count label while auto-scaling is on (the count is then a base the zoom raises).
panel-iterations-base = Ітерації (база)
panel-iterations = Ітерації
panel-iterations-hover = Базова кількість ітерацій. Коли ввімкнено автомасштабування, ефективна кількість зростає з глибиною наближення. Поки ви рухаєтеся, попередній перегляд обмежує ітерації низьким значенням (50 000) заради чуйності, а потім уточнюється до повної кількості, коли вигляд усталиться, — тож глибокі краї виглядають плавно під час руху й проявляються, коли ви зупиняєтеся.
# A warning under the iteration count. $count: the iterations the settled preview renders.
panel-res-limited =
    { $count ->
        [one] На цій глибині усталений попередній перегляд рендерить { $count } ітерацію зі зниженою роздільністю, щоб лишатися чуйним. Експортуйте, щоб отримати деталі в повній роздільності.
        [few] На цій глибині усталений попередній перегляд рендерить { $count } ітерації зі зниженою роздільністю, щоб лишатися чуйним. Експортуйте, щоб отримати деталі в повній роздільності.
        [many] На цій глибині усталений попередній перегляд рендерить { $count } ітерацій зі зниженою роздільністю, щоб лишатися чуйним. Експортуйте, щоб отримати деталі в повній роздільності.
       *[other] На цій глибині усталений попередній перегляд рендерить { $count } ітерації зі зниженою роздільністю, щоб лишатися чуйним. Експортуйте, щоб отримати деталі в повній роздільності.
    }
panel-anti-alias = Згладжування
# Anti-aliasing off (the other choices are 2×, 3×, 4×, 8×).
panel-anti-alias-off = Вимкнено
panel-anti-alias-hover = Суперсемплінг для нерухомих зображень (застосовується, коли вигляд усталиться). Більше значення приборкує дрібний зовнішній «пил» ціною часу рендерингу.

## The control panel: Effects

panel-relief-lighting = Рельєфне 3D-освітлення
panel-relief-lighting-hover = Затінювати поверхню за нахилом оцінки відстані — тиснений, освітлений вигляд. (Голоморфні родини: Mandelbrot / Multibrot.)
panel-light-angle = Кут світла
panel-relief = Рельєф
panel-relief-hover = Менше = різкіший рельєф; більше = мʼякший/пласкіший.
panel-rotate-light = Обертати світло
panel-rotate-light-hover = Обертати напрямок світла з часом (використовує повзунок «Швидкість»).
panel-distance-glow = Світіння за відстанню
panel-distance-glow-hover = Яскраві контурні смуги оцінки відстані, що згущуються у світні нитки біля межі. (Голоморфні родини.)
panel-glow = Світіння
panel-band-width = Ширина смуг
panel-band-width-hover = Проміжок між контурами відстані (октав на смугу).
panel-animate-glow = Анімувати світіння
panel-animate-glow-hover = Змушувати смуги світіння текти з часом (використовує повзунок «Швидкість»).

## The control panel: Overlays

panel-orbits-hover = Малювати шлях ітерацій точки під курсором.
panel-orbit-anim = Анімувати (бігова точка)
panel-orbit-anim-hover = Запускати вздовж орбіти точку, що біжить і змінює колір.
panel-orbit-speed = Швидкість орбіти
panel-minimap-hover = Невеликий огляд початкового вигляду з позначкою «ви тут» і глибиною наближення. Клацніть на ньому, щоб перейти до області.

## The control panel: Advanced
## Labels before a control sit in a column about 104 px wide; most of these are already wider in English.

# Small headings of the section's two groups.
panel-accelerators = Прискорювачі
panel-performance-tuning = Налаштування продуктивності
panel-bla = Прискорення BLA (глибоке наближення)
panel-bla-hover = Білінійна апроксимація: пропуск ітерацій уздовж усієї орбіти на екстремальній глибині (floatexp Mandelbrot, ≥1e28×) — рендер на GPU ~5× швидший, результат ідентичний (перевірено самотестом). Типово ввімкнено; вимкніть, щоб порівняти, або якщо натрапили на артефакт.
panel-series-approx = Апроксимація рядом
panel-series-approx-hover = Ініціалізувати збурення многочленом, щоб пропустити ранні ітерації там, де BLA не активна (глибини df32, Multibrot, BLA вимкнено). Результат ідентичний; вимкніть, щоб порівняти.
panel-glitch-correct = Виправлення збоїв (експорт)
panel-glitch-correct-hover = Виправлення збоїв за допомогою кількох опорних орбіт для експортованих зображень: виявляє збої збурення й перерендерює ці пікселі відносно додаткових опорних орбіт, доки не стане чисто. Типово ввімкнено. Експорти до ~32 MP (без допоміжних методів розфарбування); більші зображення й живий вигляд використовують простий шлях. Ніколи не використовується для виглядів Mandelbrot чи Multibrot (поза режимом Жюліа): перевірка за точним обчисленням показала, що там воно нічого не виправляло, а забирало до 5× часу експорту.
panel-dive-limit = Межа занурення автонаближення
panel-dive-limit-hover = Глибина, на якій автонаближення (клавіша A) зупиняється. До ~1e271× воно ковзає плавно; глибше перемикається на уривчасте покрокове занурення, щоб швидко дістатися екстремальної глибини.
# Followed by two choices, Speed and Quality.
panel-autopilot-priority = Пріоритет автонаближення
panel-priority-speed-hover = Наближатися із заданою швидкістю. Екран оновлюється, щойно готовий кадр із деталями, з такою роздільністю, яка забезпечує постійні оновлення (нижче межі «Мін. роздільність у русі», якщо наближення випереджає рендер); між оновленнями останній кадр стежить за наближенням і розмивається.
panel-priority-quality-hover = Кожне оновлення — повністю опрацьований кадр повної роздільності, а наближення за потреби сповільнюється, щоб кадр на екрані ніколи не збільшувався надто сильно, перш ніж зʼявиться наступний. Повільніше занурення, чітке в кожну мить.
# Followed by two choices, Detail and Misiurewicz point.
panel-autopilot-target = Ціль автонаближення
panel-target-detail-hover = Пірнати до найбагатшої структури на краю фрактала у вигляді, вибираючи ціль заново під час занурення.
panel-target-misiurewicz-hover = Знайти найближчу точку Мішуревича (центр спіралі, структура якої повторюється без кінця) і пірнути точно в неї. Спершу занурення стоїть на місці, поки точку обчислюють до глибини межі занурення, — мить на звичайних глибинах, довше після 1e1000×. Лише множина Мандельброта.
panel-work-budget = Бюджет живого рендеру
panel-work-budget-hover = Деталізація проти швидкості для живого вигляду на глибокому наближенні. Більше значення рендерить із повнішою роздільністю (чіткіше — менше «мʼякого» вигляду після масштабування), але знижує частоту кадрів, а дуже високі значення ризикують короткочасно загальмувати GPU на найважчих кадрах. Експорти завжди мають повну роздільність незалежно від цього.
panel-min-motion-res = Мін. роздільність у русі
panel-min-motion-res-hover = Найнижча роздільність, до якої може опуститися живий вигляд під час безперервного наближення чи пересування на глибокому наближенні. Більше значення не дає швидкому зануренню ставати блоковим/піксельованим (чіткіше) ціною частоти кадрів — на 100% оновлення деталей глибокого занурення мусять рендеритися з повною вартістю, через що глибоке наближення може помітно смикатися. Якщо глибоке занурення здається уривчастим, зменште це значення. Типово 30%; не впливає на усталене зображення чи експорти.
panel-finish-sound = Звук після завершення рендеру
panel-finish-sound-hover = Програвати мелодію завершення FRACTINT, коли закінчується експорт чи рендер туру: три висхідні ноти (1047, 1109, 1175 Hz по 100 ms кожна), точно так, як їх закодовано в оригіналі для DOS, — синтезовані як смугообмежений меандр і пропущені через фільтр високих частот, щоб звучати як маленький динамік PC-speaker, що їх грав.
panel-prefer-detail = Віддавати перевагу деталям під час наближення
panel-prefer-detail-hover = Під час наближення чи пересування продовжувати показувати останній повністю деталізований кадр — масштабований і зсунутий услід за рухом — замість перерендерювати кожен кадр зі зниженою якістю. Вигляд рендериться повністю, коли рух призупиняється. Вимкнено: рух рендериться наживо зі зниженою роздільністю (плавніше, грубіше). Мілкі вигляди (прямий режим) завжди рендеряться наживо — вони в будь-якому разі дешеві й чіткі в кожному кадрі.

## The control panel: About (the current fractal)

# A link to the fractal's reference page, followed by an arrow.
panel-about-reference = Джерело

## The control panel: Advanced ▸ Second graphics card

# A label before a drop-down (wider than the panel's label column already).
gpu2-label = Друга відеокарта
gpu2-off = Вимкнено
# Another device on the same graphics card as the window (the default).
gpu2-same-card = Та сама карта
# A card the list does not name. $number: the card's number.
gpu2-card-n = Карта { $number }
gpu2-finding = Пошук відеокарт цього компʼютера…
gpu2-none = Цей компʼютер не має іншої відеокарти.
gpu2-hover =
    Інший графічний пристрій рендерить кадри глибокого вигляду, поки той рухається, і частину додаткових вибірок, що уточнюють його після зупинки, тож зображення встигає за наближенням і раніше досягає кінцевої якості. Карта, що малює вікно, як і раніше малює все.

    Інша карта додає власну швидкість. Дві різні моделі карт малюють кілька пікселів трохи по-різному; якщо друга карта дає збій, вигляд працює далі з однією.

    «Та сама карта» (типово) відкриває другий пристрій на цій карті. Він не додає швидкості, але рендерить, не чекаючи кадрів екрана. Він ділить карту: збій, що скидає карту, завершує програму, як і збій власного рендерингу вікна, а наступний запуск запропонує вимкнути цей пристрій.

## The view (canvas)

# A notice after Ctrl+click pins the Julia c at a deep zoom.
canvas-julia-pin-quantized = Параметр c Жюліа закріплено — але він зберігається з подвійною точністю, тож на цій глибині вся панель округлюється до одного значення, і закріплення потрапляє на центр вигляду. Віддаліться, щоб націлити його.
# A small caption in the top-left corner of the view while the orbit overlay is fitted to the view (its "Normalize (fit to view)" option).
canvas-orbit-normalized = орбіта (нормалізована)
# The minimap's tooltip.
canvas-minimap-hover = Огляд / ви тут. Перетягуйте, щоб пересувати вигляд у поточному масштабі, прокручуйте, щоб наближати й віддаляти, клацніть, щоб перейти до цієї області (початковий масштаб).

## The timestamp overlay (View ▸ Show timestamp)

# Under the large elapsed-time clock. $frame: the frame number (as the log writes it, without grouping).
overlay-timestamp-frame = кадр { $frame }

## File ▸ Settings ▸ Reference cache…

cache-title = Кеш опорних орбіт
cache-intro = На екстремальній глибині повільна частина вигляду — його опорна орбіта: до години арифметики з довільною точністю. Fractadyne зберігає тут готові орбіти, тож повернення до вже відвіданого розташування чи наближення десь поруч займає лише секунди. Одна орбіта — приблизно 4 MB.
cache-off = Вимкнено для цього запуску (запуск завдання або --no-orbit-cache). Нічого не читатиметься й не записуватиметься до наступного звичайного запуску.
# Small headings of the window's parts.
cache-location = Розташування
cache-usage = Використання
cache-limit = Ліміт
cache-no-config = (немає доступної теки конфігурації)
cache-open-folder = Відкрити теку
# The text on the usage bar. $used and $limit: sizes such as "1.2 GB"; $count: the orbits kept.
cache-usage-bar =
    { $count ->
        [one] { $used } з { $limit } — { $count } орбіта
        [few] { $used } з { $limit } — { $count } орбіти
        [many] { $used } з { $limit } — { $count } орбіт
       *[other] { $used } з { $limit } — { $count } орбіти
    }
cache-limit-hover = Коли кеш перевищує це значення, першими видаляються орбіти, які було НАЙДЕШЕВШЕ побудувати, — глибока орбіта, що будувалася годину, переживе будь-яку кількість швидких, хоч як недавно їх використовували.
# Beside the limit. $count: about how many orbits of the live view's length fit in it.
cache-limit-estimate =
    { $count ->
        [one] ≈ { $count } орбіта максимальної довжини живого вигляду
        [few] ≈ { $count } орбіти максимальної довжини живого вигляду
        [many] ≈ { $count } орбіт максимальної довжини живого вигляду
       *[other] ≈ { $count } орбіти максимальної довжини живого вигляду
    }
cache-kept = Збережено (найцінніші першими)
# Column headers of the list of kept orbits.
cache-col-iterations = ітерації
cache-col-precision = точність
cache-col-size = розмір
cache-col-last-used = востаннє
# The precision column. $bits: the precision in bits.
cache-row-bits =
    { $bits ->
        [one] { $bits } біт
        [few] { $bits } біти
        [many] { $bits } бітів
       *[other] { $bits } біта
    }
# Under the list. $count: the orbits not listed.
cache-rows-more = … і ще { $count }
# When an orbit was last used (the "last used" column).
cache-ago-now = щойно
# $count: minutes.
cache-ago-minutes = { $count } хв тому
# $count: hours.
cache-ago-hours = { $count } год тому
# $count: days.
cache-ago-days = { $count } дн. тому
# Asked before clearing. $count: the orbits; $size: their total size, such as "1.2 GB".
cache-confirm =
    { $count ->
        [one] Видалити { $count } орбіту ({ $size })? Її відбудова коштує часу, а не даних.
        [few] Видалити { $count } орбіти ({ $size })? Їх відбудова коштує часу, а не даних.
        [many] Видалити { $count } орбіт ({ $size })? Їх відбудова коштує часу, а не даних.
       *[other] Видалити { $count } орбіти ({ $size })? Їх відбудова коштує часу, а не даних.
    }
cache-delete = Видалити
cache-cancel = Скасувати
cache-clear = Очистити кеш…
cache-clear-hover = Видаляє всі збережені орбіти. Спершу запитує.
cache-close = Закрити
# $count: the orbits removed.
cache-cleared =
    { $count ->
        [one] Кеш опорних орбіт очищено — видалено { $count } орбіту.
        [few] Кеш опорних орбіт очищено — видалено { $count } орбіти.
        [many] Кеш опорних орбіт очищено — видалено { $count } орбіт.
       *[other] Кеш опорних орбіт очищено — видалено { $count } орбіти.
    }
# $why: the system's error message.
cache-clear-failed = Не вдалося очистити кеш опорних орбіт: { $why }

## Help ▸ Diagnostics…

diag-title = Діагностика
diag-intro = Запустіть тест, щоб перевірити, чи правильно Fractadyne працює на вашому обладнанні. Результати зберігаються на вашому компʼютері, і їх можна додати до звіту про проблему — сам по собі нікуди нічого не надсилається.
# $status: which arbitrary-precision library is in use, as the program reports it (in English).
diag-arithmetic = Арифметика глибокого наближення: { $status }
# The three tests' buttons, each followed by a sentence saying what it does.
diag-run-selftest = Запустити самотест
diag-run-uitest = Запустити тест інтерфейсу
diag-run-gputest = Запустити перевірку арифметики GPU
diag-blurb-selftest = Перевіряє математику й рендеринг на вашому GPU за відомими правильними результатами. Триває близько 15 секунд.
diag-blurb-uitest = Проходить інтерфейс і живий вигляд, роблячи знімки екрана на кількох глибинах наближення. Триває хвилину-дві й під час роботи відкриває вікна.
diag-blurb-gputest = Перевіряє арифметику double-float шейдера за відомими правильними значеннями на кожному графічному бекенді, який пропонує ваш компʼютер. Триває кілька секунд, вікон не відкриває. Якщо вона повідомить про збій, це справжня знахідка, яку ми хотіли б побачити, — будь ласка, додайте її до звіту.
# While a test runs. $count: the checks finished so far.
diag-checks-done =
    { $count ->
        [one] виконано { $count } перевірку
        [few] виконано { $count } перевірки
        [many] виконано { $count } перевірок
       *[other] виконано { $count } перевірки
    }
# A finished test's result. $test: the test's name (Self-test, UI test, GPU arithmetic check).
diag-verdict-captured = { $test } — результат зафіксовано
diag-verdict-passed = { $test } — пройдено
diag-verdict-problems = { $test } — виявлено проблеми
# Under the GPU arithmetic check's result.
diag-informational-note = Ця перевірка не має результату «пройдено / не пройдено»: вона фіксує, як обчислює ваш GPU, і повідомлений збій — це справжня знахідка, а не несправність вашого компʼютера. У будь-якому разі найкорисніше, що ви можете з нею зробити, — додати її до звіту.
# Under a test's result when it reported problems.
diag-problems-note = Деякі відмінності очікувані на обладнанні, відмінному від еталонного GPU, — особливо в порівняннях зображень. Найкорисніше, що ви можете з цим зробити, — додати це до звіту про проблему.
diag-open-results = Відкрити результати
diag-attach = Додати до звіту про проблему…
diag-attach-hover = Відкриває діалог звіту з цим результатом, щоб звіт про проблему містив машинно перевірений тест, а не лише опис
diag-console = Виводити діагностику в консоль
diag-console-hover = Відтепер показувати рядки [fd-…] у stderr. Типово вимкнено, коли Fractadyne запущено без аргументів. Це нічого не змінює в тому, що ЗАПИСУЄТЬСЯ, — файл журналу завжди отримує кожен рядок, — тож залиште вимкненим, якщо ви не стежите за терміналом. Діє лише для цього сеансу; щоб охопити й запуск, використовуйте прапорець --console або FRACTADYNE_CONSOLE=1.
diag-close = Закрити
# Errors starting a test. $why: the system's error message; $dir: a folder.
diag-error-no-exe = Не вдається знайти виконуваний файл: { $why }
diag-error-create-dir = Не вдається створити { $dir }: { $why }
diag-error-start = Не вдалося запустити тест: { $why }

## Navigate ▸ Misiurewicz explorer…

misi-title = Оглядач точок Мішуревича
misi-mandelbrot-only = Перегляд точок Мішуревича працює лише для Mandelbrot — перемкніть фрактал, щоб ним скористатися.
# While the gallery is being built. $seconds: the time so far, such as "3".
misi-solving = Обчислення родин малих типів ({ $seconds } s)…
# Above the gallery. $total: all points; $curated: the hand-picked ones; $found: the ones the sweep found.
misi-summary = Точок: { $total } — відібраних вручну: { $curated }, знайдених проходом методу Ньютона з початковими точками по малих типах: { $found } (верхня півплощина; груба сітка знаходить більшість басейнів, але не доведено, що всі).
# The name of a point the sweep found. $kind: its type, such as "(3,1)"; $point: its coordinate, such as "-0.1011+0.9563i".
misi-entry-name = { $kind } у { $point }
# Under each thumbnail. $kind: the point's type, such as "(3,1)"; $octaves: the zoom period in octaves, such as "3.17"; $twist: the twist per repeat in degrees, such as "+12.5". "oct" is short for octaves.
misi-entry-readout = { $kind } · { $octaves } окт/повтор · { $twist }°
# Before the depth field.
misi-zoom-to = Наблизити до
misi-cancel = Скасувати
misi-solve-jump = Обчислити й перейти
misi-solve-jump-hover = Обчислити цю точну точку методом Ньютона з точністю, якої потребує задана глибина (з її збереженою координатою як початковим наближенням), а потім перейти просто туди. Глибокі запити тривають довше — обчислення виконується з довільною точністю.
misi-select-hint = Виберіть точку, задайте глибину й переходьте — рядок мультиплікатора під кожною мініатюрою показує, який вигляд матиме занурення туди: скільки октав до повтору вигляду і наскільки він закручується за кожен повтор.
misi-cancelled = Обчислення скасовано.
# $text: what was typed in the depth field.
misi-bad-zoom = Не вдається прочитати «{ $text }» як збільшення (пишіть 1e50, 3.2e120, …).
misi-no-kp = Цей запис не має виявленої пари (k,p) для обчислення.
misi-parse-failed = Не вдалося розібрати збережену координату.
# A notice after the jump. $name: the point's name; $zoom: the magnification, such as "1e50"; $seconds: how long the solve took, such as "1.2".
misi-zoomed = Наближено до точки { $name } — { $zoom }× ({ $seconds } s)
# Added after the notice above when the iteration count is too low. $have: the fixed iteration count; $typical: what the depth typically needs.
misi-iter-note = Кількість ітерацій зафіксовано на { $have }; ця глибина зазвичай потребує ~{ $typical } — збільште її (або ввімкніть автоітерації), якщо вигляд рендериться пласким.
# $name: the point's name; $why: why the solve failed (a technical word).
misi-solve-missed = Глибоке обчислення не потрапило на точку { $name }: { $why }. Збережена точка все одно точна на власній глибині — спробуйте менш глибоку ціль.

## Life (Game of Life): the side panel

# Shown when the graphics card cannot run the Life view.
life-no-compute = Цей графічний адаптер не може запустити Life (немає обчислювальних шейдерів).
# Row label: the pattern list (pattern names stay in English).
life-pattern = Шаблон
life-open = Відкрити…
life-open-hover = Відкрити файл шаблону RLE, .cells або Life 1.05/1.06.
life-pattern-text = Текст шаблону…
life-pattern-text-hover = Вставити шаблон (RLE у тому вигляді, як його подає LifeWiki, або простий текст).
life-save = Зберегти…
life-save-hover = Зберегти всесвіт у його теперішньому стані у форматі RLE.
# Row label: the rule list and the typed rule (B3/S23 notation).
life-rule = Правило
# Shown in the rule list when the rule in use is none of the library's.
life-rule-custom = власне
life-apply = Застосувати
# Playback buttons (a pause or play symbol is shown before them).
life-pause = Пауза
life-play = Пуск
life-play-hover = Запустити всесвіт (Enter).
life-step = Крок
life-step-hover = Просунутися на одне покоління (N).
# Tooltip of the button beside Step, which reads "+16", "+1,024"…: that many generations.
life-stride-hover = Просунутися на стільки поколінь (Shift+N).
life-reset = Скинути
life-reset-hover = Повернутися до шаблону в тому вигляді, у якому його відкрито (покоління 0).
life-speed = Швидкість
# After the speed slider's number: generations a second. Short.
life-speed-unit = пок/с
# Row label: how many generations the "+n" button advances.
life-stride = Стрибок
# The panel's numbers.
life-generation = Покоління
life-population = Популяція
life-tiles = Плитки
# $count: the tiles stored; $capacity: how many the GPU pool holds.
life-tiles-of = { $count } з { $capacity }
life-tiles-hover = Збережені плитки по 64×64 клітинки (по 4 KiB кожна) і місткість пулу GPU.
# Row label: what a left-drag in the view does (Pan or Draw).
life-mouse = Миша
life-pan = Пересування
life-pan-hover = Перетягуйте, щоб пересувати вигляд.
life-draw = Малювання
# The Draw tool's tooltip; the toolbar's shows it after "Draw (D):", so it starts lowercase.
life-draw-hint = клацніть клітинку, щоб перемкнути її; перетягуйте, щоб малювати — штрих, що починається на мертвій клітинці, малює, а той, що починається на живій, стирає. Запущений всесвіт призупиняється, поки ви малюєте, і продовжує роботу, щойно ви відпускаєте кнопку.
life-undo = Скасувати
life-undo-hover = Повернутися до всесвіту перед останнім редагуванням, заливкою, очищенням чи зміною правила (Ctrl+Z під час малювання).
life-grid = Лінії сітки
life-grid-hover = Межі клітинок, коли наближено до 8 пікселів на клітинку або більше.
# A random fill: "Random" labels the density field, "seed" the seed field after it.
life-random = Випадково
life-density-hover = Частка клітинок, які заливка робить живими.
life-seed = зерно
life-fill-view = Заповнити вигляд
# $cells: the largest side of the filled area, in cells (2,048).
life-fill-view-hover = Випадково заповнити видиму область (до { $cells } × { $cells } клітинок навколо її центру).
life-clear = Очистити
life-clear-hover = Убити всі клітинки («Скасувати» повертає їх).

## Life: notices in the panel, and file dialogs

# $path: the file written.
life-saved = Збережено { $path }
# $error: why (from the system, in English).
life-save-failed = Не вдалося зберегти: { $error }
# The universe stopped running. $error: why (in English).
life-stopped = Зупинено: { $error }
# A pattern file named a rule that does not read. $error: why (in English); $rule: the rule kept, in B3/S23 notation.
life-rule-not-understood = Правило шаблону не вдалося зрозуміти ({ $error }); залишено { $rule }
# $path: the file; $error: why (in English).
life-open-failed = Не вдалося відкрити { $path }: { $error }
# File-dialog filter names.
life-filter-patterns = Шаблони Life
life-filter-rle = Шаблон із кодуванням довжин серій (RLE)

## Life: the toolbar (its buttons are icons; these are their tooltips)

life-toolbar-pause-hover = Призупинити всесвіт (Enter)
life-toolbar-play-hover = Запустити всесвіт (Enter)
# $stride: the generations a Shift+click steps.
life-toolbar-step-hover = Крок на одне покоління (N); Shift+клацання або Shift+N — крок на { $stride }
life-toolbar-reset-hover = Повернутися до шаблону в тому вигляді, у якому його відкрито (покоління 0)
# $hint: the Draw tool's tooltip (life-draw-hint).
life-toolbar-draw-hover = Малювання (D): { $hint }

## Life: the Pattern text window

life-text-title = Текст шаблону
life-text-intro = Вставте шаблон: RLE (у тому вигляді, як його подають LifeWiki і Golly), простий текст .cells або Life 1.05 / 1.06.
life-text-load = Завантажити
life-text-current = Поточний як RLE
life-text-current-hover = Заповнити поле всесвітом у тому вигляді, у якому його востаннє завантажено.

## L-systems: the side panel

# Row label: the system list (system names stay in English).
lsys-system = Система
lsys-open = Відкрити…
lsys-open-hover = Відкрити файл Fractint .l або систему .lsys.
lsys-edit = Редагувати…
lsys-edit-hover = Текст системи: аксіома, продукції, кут — редагуйте й застосовуйте.
lsys-save = Зберегти…
lsys-save-hover = Зберегти систему як текстовий файл .lsys.
# Tooltip of the "SVG…" button.
lsys-svg-hover = Експортувати вигляд як SVG-малюнок: його лінії та фігури — векторами, у його кольорах.
lsys-follow = Порядок слідує за наближенням
lsys-follow-hover = Порядок зростає під час наближення, тож деталі ніколи не закінчуються.
lsys-follow-fixed-hover = Ця система не збільшується в однакову кількість разів із кожним порядком, тож її малюють із фіксованим порядком.
lsys-order = Порядок
lsys-angle = Кут
lsys-sweep-hover = Проводити кут туди й назад: крива змінюється, обертаючись
# A small button that puts back the system's own angle after the slider moved it.
lsys-system-angle = Кут системи
lsys-seed = Зерно
lsys-seed-hover = Від чого залежать випадкові вибори: те саме зерно — та сама рослина.
lsys-seed-new = Нове
lsys-seed-new-hover = Інша рослина: нове зерно.
lsys-line-width = Товщина лінії
# Row label of a percentage slider: how much of the curve is drawn.
lsys-draw-on = Промальовування
lsys-draw-on-hover = Яку частину кривої намальовано, у тому порядку, в якому її малює черепаха
lsys-draw-on-play-hover = Крива, що сама себе малює
lsys-colour-by = Колір за
# The colourings, in the "Colour by" list.
lsys-colour-position = позицією вздовж кривої
lsys-colour-depth = глибиною гілки
lsys-colour-heading = напрямком
# C, < and > are the system's commands that set the colour index: keep them.
lsys-colour-index = індексом кольору (C, <, >)
lsys-colour-plain = одним кольором
# The panel's numbers.
lsys-growth = Зростання
lsys-growth-built = побудовано як слово, з його порядком
lsys-growth-built-hover = Параметрична або контекстно-залежна: переписування модуля залежить від його чисел чи сусідів, тож слово будується по одному поколінню (до двох мільйонів модулів) і малюється з подвійною точністю — порядок не слідує за наближенням.
# How much the picture grows from one order to the next. $factor: the factor (1.4142).
lsys-growth-factor = ×{ $factor } на порядок
lsys-growth-factor-period-2 = ×{ $factor } на порядок (порядки йдуть із кроком 2)
lsys-growth-none = немає (фіксований порядок)
lsys-drawn = Намальовано
# $drawn: what was drawn (lsys-drawn-segments, lsys-drawn-shapes or lsys-drawn-segments-shapes); $order: the order.
lsys-drawn-at-order = { $drawn } на порядку { $order }
# The same, when the walk stopped at its segment budget.
lsys-drawn-at-order-budget = { $drawn } на порядку { $order } (бюджет)
# $segments: the line segments drawn.
lsys-drawn-segments =
    { $segments ->
        [one] { $segments } відрізок
        [few] { $segments } відрізки
        [many] { $segments } відрізків
       *[other] { $segments } відрізка
    }
# $shapes: the filled shapes drawn.
lsys-drawn-shapes =
    { $shapes ->
        [one] { $shapes } заповнена фігура
        [few] { $shapes } заповнені фігури
        [many] { $shapes } заповнених фігур
       *[other] { $shapes } заповненої фігури
    }
# Both counts. $segments: line segments; $shapes: filled shapes.
lsys-drawn-segments-shapes =
    { $segments ->
        [one] { $segments } відрізок
        [few] { $segments } відрізки
        [many] { $segments } відрізків
       *[other] { $segments } відрізка
    }, { $shapes ->
        [one] { $shapes } заповнена фігура
        [few] { $shapes } заповнені фігури
        [many] { $shapes } заповнених фігур
       *[other] { $shapes } заповненої фігури
    }
# $ms: the walk's time in milliseconds (12.3).
lsys-drawn-hover = обійдено за { $ms } ms; піддерева поза виглядом пропускаються, а менші за піксель малюються одним відрізком
# A word too long to build at the order asked for. $wanted: the order asked for; $built: the order drawn.
lsys-drawn-short = Слово порядку { $wanted } має понад два мільйони модулів: намальовано з порядком { $built }.
# $order: the deepest order the app draws.
lsys-deepest-order = На найглибшому порядку, що малюється ({ $order }): подальше наближення не додає деталей.

## L-systems: notices in the panel and the editor, and file dialogs

# $path: the file; $error: why (in English).
lsys-open-failed = Не вдалося відкрити { $path }: { $error }
# Why a file was not opened (shown after "Could not open …:").
lsys-file-too-large = більший за 4 MiB
# Why a Fractint .l file was not opened: it holds no entries. `Name { … }` is the file's syntax: keep it.
lsys-no-entries = немає записів (файл .l містить записи `Name {"{"} … {"}"}`)
# $path: the file written.
lsys-saved = Збережено { $path }
# $error: why (from the system, in English).
lsys-save-failed = Не вдалося зберегти: { $error }
# After File ▸ Export SVG. $path: the file written.
lsys-svg-exported = Експортовано { $path }
# $error: why (from the system, in English).
lsys-svg-failed = Не вдалося експортувати: { $error }
# File-dialog filter names: opening (any system file) and saving (one system).
lsys-filter-systems = L-системи
lsys-filter-system = L-система

## L-systems: the toolbar

lsys-toolbar-lower-hover = Знизити порядок (фіксує його)
lsys-toolbar-raise-hover = Підвищити порядок (фіксує його)
# Between the − and + buttons, in a fixed-width font. $order: the order, padded to 3 characters so the label keeps its width.
lsys-toolbar-order = порядок { $order }
# A toggle: the order follows the zoom. Short.
lsys-toolbar-auto = авто
lsys-toolbar-auto-hover = Порядок слідує за наближенням
lsys-toolbar-edit-hover = Редагувати текст системи

## L-systems: the editor window, and a .l file's entries

lsys-editor-title = L-система
# The editor's help. The quoted examples ('X = word', 'seed n', 'define R 1.456'…), the commands and "The Algorithmic Beauty of Plants" (a book's title) are the system's syntax and names: keep them as they are.
lsys-editor-help = Один ключ на рядок: angle (градуси або /n для поділу кола), axiom, продукції 'X = word', а також за бажанням heading, draw / move / variables, colour, order. Символ з альтернативами, що вибираються випадково, має по рядку для кожної, з її вагою: 'X (0.3) = word'; 'seed n' вибирає рослину. У слові {"{"} {"}"} заповнює шлях черепахи, а . додає вершину. Параметричні й контекстно-залежні продукції (The Algorithmic Beauty of Plants): 'A(s) : s > 1 = F(s)[+A(s/2)]', 'b < a > c = b', з 'define R 1.456' та 'ignore +-F'; F(l) робить крок l, +(a) повертає на a градусів. Запис Fractint .l можна вставити як є.
lsys-editor-apply = Застосувати
lsys-editor-revert = Повернути
lsys-editor-revert-hover = Текст показаної системи.
# The window listing a .l file's entries to pick from. $file: the file's name.
lsys-entries-title = { $file } — записи
# Tooltip of an entry that does not parse (its error is shown beside it, in English).
lsys-entry-unreadable-hover = Цей запис не читається.

## Export: opening a view or location (File ▸ Open view or location…)

# File-dialog filter names.
export-filter-view = Вигляд або розташування Fractadyne
export-filter-image = Зображення (PNG / EXR)
export-filter-location = Розташування (.fdn / .kfr)
# $zoom: the magnification (1.2e148), followed by the magnification sign ×.
export-loaded-location = Розташування завантажено @ { $zoom }×
# $zoom: the magnification; $note: what was noteworthy in the file (in English).
export-loaded-location-note = Завантажено @ { $zoom }× — { $note }
# $path: the file.
export-not-location = { $path } не є розташуванням Fractadyne.
export-file-too-large = Файл завеликий (можливо, це не розташування .fdn?).
# $path: the file; $error: why (from the system, in English).
export-read-failed = Не вдалося прочитати { $path }: { $error }
# A Kalles Fraktaler .kfr file did not import. $error: why (in English).
export-import-failed = Не вдалося імпортувати: { $error }
# $path: an image whose embedded text is not a view.
export-not-view = { $path } має метадані Fractadyne, але не вигляд.
# $path: the image the view was read from.
export-loaded-view = Вигляд завантажено з { $path }
# $path: the image; $note: what was noteworthy in it (in English).
export-loaded-view-note = Вигляд завантажено з { $path } — { $note }
export-no-metadata = Цей файл не має вбудованих метаданих вигляду Fractadyne.

## Export: notices after an export or a snapshot
## "File → Export image…" in them names the menu items (menu-file, menu-file-export).

# The export window's status while the image renders.
export-rendering = Рендеринг…
export-canceled = Експорт скасовано.
# $error: why (in English).
export-failed = Не вдалося експортувати: { $error }
# $size: the image's size in pixels (1920×1080); $path: the file written.
export-saved = Збережено { $size } → { $path }
# An L-system export whose walk stopped at its budget. $size: pixels; $segments: the budget (4,000,000); $path: the file.
export-saved-incomplete = Збережено { $size } (неповне: перевищено бюджет відрізків ({ $segments }) — знизьте порядок) → { $path }
# A two-panel export written as two files. $path: the first of them.
export-saved-two = Збережено 2 файли → { $path }
# $size: pixels; $path: the file written.
export-saved-corrected = Збережено { $size } (з виправленням збоїв) → { $path }
# $path: the first of the two files.
export-saved-two-corrected = Збережено 2 файли (з виправленням збоїв) → { $path }
# Added after a finished export's notice. $duration: how long it took (8.3s, 1m 04.0s).
export-elapsed = (за { $duration })
export-preparing-deep = Підготовка глибокого експорту — побудова опорної орбіти (це може зайняти певний час)…
# $size: pixels (5120×4035); $ss: the supersampling factor; $iterations: the iteration count.
export-large = Великий експорт ({ $size }, суперсемплінг { $ss }×, ітерацій: { $iterations }) — рендеринг у фоновому режимі; для експортів такого розміру виправлення збоїв пропускається…
export-life-unavailable = Експорт зображення вигляду Life поки недоступний — скористайтеся знімком екрана або Файл ▸ Зберегти шаблон Life.
export-already-running = Експорт уже виконується — Файл → Експортувати зображення… показує його поступ.
# $size: the image's size in pixels.
export-snapshot-background = Знімок: рендеринг { $size } у фоновому режимі — Файл → Експортувати зображення… показує поступ і дає змогу скасувати рендер.
export-no-gpu = GPU недоступний
# A screen-capture snapshot. $size: pixels; $path: the file written.
export-snapshot-saved = Знімок екрана { $size } збережено → { $path }
# $error: why (in English).
export-snapshot-failed = Не вдалося зробити знімок: { $error }

## The colour picker (the popup of a colour swatch)

# The third row's label: the colour as hexadecimal bytes. Short (about 6 characters).
picker-row-hex = Hex
picker-hex-hover = Клацніть, щоб ввести шістнадцяткове значення
picker-cancel = Скасувати
picker-cancel-hover = Повернути колір, з яким відкрито цей вибір кольору
picker-ok = Гаразд
picker-ok-hover = Залишити цей колір
# Starts the screen eyedropper (an icon is shown before it).
picker-pick = Піпетка
picker-pick-hover = Взяти колір будь-де на екрані
# Tooltip of the disabled Pick button (in the picker and the gradient editor) on Linux and macOS.
picker-eyedropper-unsupported = Щоб узяти колір деінде на екрані, потрібен API захоплення екрана платформи. Він є лише у збірці для Windows; на Linux для цього знадобилася б залежність від X11, а Wayland вимагає порталу з власним запитом дозволу.

## The fractal families: what each one is (the info panel's "About" section and the pickers' hover text).
## The formulas themselves (z -> z^2 + c) are written in code and stay as they are.

fractal-mandelbrot-about = Канонічний фрактал часу втечі: множина тих c, для яких орбіта 0 лишається обмеженою. Її межа нескінченно складна.
fractal-multibrot3-about = Множина Multibrot — побудова Мандельброта з вищим степенем. Степінь d дає обертову симетрію порядку (d-1).
fractal-multibrot4-about = Multibrot зі степенем 4: симетрія 3-го порядку, широкі бруньки.
fractal-multibrot5-about = Multibrot зі степенем 5: симетрія 4-го порядку.
fractal-tricorn-about = Tricorn (Mandelbar): на кожному кроці спряжує z. Це антиголоморфне відображення дає фігуру з трьома кутами.
fractal-burning-ship-about = Абсолютні значення частин z беруться перед піднесенням до квадрата; результат нагадує корабель у полумʼї.
fractal-celtic-about = Родич Burning Ship, що бере абсолютне значення лише дійсної частини z^2, створюючи мотиви кельтських вузлів / сердець.
fractal-buffalo-about = Варіант з abs, що бере абсолютні значення обох компонент z^2.
fractal-phoenix-about = Phoenix використовує й попередню ітерацію, що дає полумʼяноподібні нитки. Спробуйте його форму Жюліа через режим Жюліа.
fractal-newton-about = Ітерація Ньютона для пошуку коренів z^3 = 1, розфарбована за тим, як швидко збігається кожна точка. Фрактал збіжності (а не втечі).
fractal-multibrot6-about = Multibrot зі степенем 6: симетрія 5-го порядку.
fractal-multibrot7-about = Multibrot зі степенем 7: симетрія 6-го порядку, бруньки витончуються, наближаючись до кола.
fractal-multibrot8-about = Multibrot зі степенем 8: симетрія 7-го порядку.
fractal-burning-ship3-about = Burning Ship зі степенем 3: беруться абсолютні значення, потім підносяться до куба.
fractal-burning-ship4-about = Burning Ship зі степенем 4.
fractal-burning-ship5-about = Burning Ship зі степенем 5.
fractal-tricorn3-about = Tricorn (Mandelbar) зі степенем 3: симетрія 4-го порядку.
fractal-tricorn4-about = Tricorn зі степенем 4: симетрія 5-го порядку.
fractal-tricorn5-about = Tricorn зі степенем 5: симетрія 6-го порядку.
fractal-celtic3-about = Згин Celtic зі степенем 3: абсолютне значення дійсної частини z^3.
fractal-celtic4-about = Згин Celtic зі степенем 4.
fractal-celtic5-about = Згин Celtic зі степенем 5.
fractal-buffalo3-about = Buffalo зі степенем 3: абсолютні значення обох частин z^3.
fractal-buffalo4-about = Buffalo зі степенем 4.
fractal-buffalo5-about = Buffalo зі степенем 5.
fractal-custom-about = Формула, записана у виразах у стилі Fractint (Фрактал ▸ Власна формула…). Більшість формул підтримують глибоке наближення через збурення, як і вбудовані родини; діалог формули показує, чи підтримує його ваша.
fractal-life-about = Гра «Життя» Конвея та її родичі на необмеженій площині: правила Generations, нетоталістичні правила (Hensel), файли шаблонів. Запускайте, крокуйте, малюйте.
fractal-lsystem-about = Системи Лінденмаєра: криві, зокрема ті, що заповнюють простір, острови й рослини, які малює черепаха за граматикою переписування. Порядок слідує за наближенням, тож деталі ніколи не закінчуються. Файли Fractint .l відкриваються безпосередньо.

# Shown where the other families show their formula (z -> z^2 + c).
fractal-custom-formula = z -> формула, яку ви пишете
# Shown where the other families show their formula. "B3/S23" is the rule's notation: keep it.
fractal-life-formula = B3/S23: народжується з 3 живими сусідами, виживає з 2 або 3
# Shown where the other families show their formula. "F = F+F--F+F" is the grammar's notation: keep it.
fractal-lsystem-formula = F = F+F--F+F: кожен символ переписується, а потім черепаха малює результат

# The last paragraph of the picker's hover text for a family that renders only on the direct path.
fractal-no-deep-zoom = Лише прямий рендеринг — ця родина не має глибокого наближення.

## Names of settings, shown in the Settings menu and the welcome window.

theme-dark = Темна
theme-light = Світла
# The update track that follows only stable releases.
update-track-stable = Стабільні випуски
# The update track that also offers pre-releases (betas).
update-track-beta = Бета (попередні випуски)

## The status bar (its hover texts; the readouts follow in their own section)

status-period-hover = Період мініброта, до якого ви перейшли (натисніть M або Перейти до розташування ▸ Перейти до особливої точки). Він лишається, поки вигляд стоїть на цій особливій точці, і зникає, коли ви відходите.

## The playback transport: the player over the view while a tour plays.

# $close: the close button's icon, as drawn on that button.
play-finished-hover = Завершено — прокрутіть назад або натисніть { $close }, щоб закрити
# $kind: "benchmark" or "script" (the variants hold the wording); $name: the tour's name.
play-playing-hover =
    { $kind ->
        [benchmark] Відтворюється тест продуктивності «{ $name }»
       *[other] Відтворюється сценарій «{ $name }»
    }
play-restart = Почати спочатку
play-back = Назад на 10 секунд
play-again = Відтворити знову з початку
play-resume = Продовжити
play-pause = Пауза
play-stop = Зупинити й перемотати на початок
play-forward = Уперед на 10 секунд
play-speed-hover = Швидкість відтворення (клацайте, щоб перемикати 0.5× / 1× / 2× / 4×)
play-render = Рендерити цей сценарій у послідовність кадрів…
play-render-none = Вбудований тест продуктивності не має файлу сценарію для рендерингу
play-open = Відтворити інший сценарій…
play-loop = Повторювати тур, коли він доходить до кінця
play-close = Закрити програвач і відновити ваші власні налаштування вигляду
# A notice beside the scrub bar while the renderer holds the tour's clock. Keep it short.
play-waiting = очікування деталей
# "[playback] pace" is a setting in the script file: keep it as written.
play-waiting-hover = Годинник туру призупинено, поки рендерер опрацьовує цей вигляд (побудова опорної орбіти / зростання бюджету ітерацій). Відтворення продовжиться саме; див. [playback] pace у сценарії.
play-scrub = Перетягніть, щоб прокрутити тур
# $behind: how far the script clock is behind real time, as minutes:seconds (1:05).
play-scrub-behind =
    Перетягніть, щоб прокрутити тур
    Годинник сценарію відстає від реального часу на { $behind } (регулятор темпу сповільнив тур, щоб рендерер устигав). Верхня позначка показує, де було б відтворення за реальним часом.

## The autopilot's notices (toasts). Where a notice has two lines, the first is the verdict and the second what to do about it.

autopilot-on-detail = Автопілот увімкнено — занурення до деталей (будь-яка дія зупиняє)
autopilot-off = Автопілот вимкнено
autopilot-misi-needs-mandelbrot =
    Автопілот: ціль «Точка Мішуревича» потребує множини Мандельброта
    Натомість занурення до деталей
autopilot-on-misi =
    Автопілот увімкнено — пошук найближчої точки Мішуревича…
    Будь-яка дія зупиняє
# $seconds: how long the search has run, in whole seconds.
autopilot-misi-finding =
    Автопілот: пошук найближчої точки Мішуревича…
    { $seconds } s · будь-яка дія зупиняє
autopilot-misi-not-found =
    Автопілот: поблизу цього вигляду не знайдено точки Мішуревича
    Пошук може не сягати цієї глибини — спробуйте ціль «Деталі»
# ($preperiod,$period) names the point, as mathematicians write it: keep the notation. $widths: a power of two (2^$widths screen widths).
autopilot-misi-off-screen =
    Автопілот: найближча точка Мішуревича ({ $preperiod },{ $period }) поза екраном
    ~2^{ $widths } ширин екрана звідси — перейдіть ближче до центру спіралі
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-not-solved =
    Автопілот: не вдалося обчислити точку Мішуревича ({ $preperiod },{ $period }) поблизу цього вигляду
    Спробуйте ціль «Деталі»
autopilot-misi-refused = Автопілот: обчислення точки Мішуревича відхилено (погана пара)
autopilot-misi-stopped = Автопілот: обчислювач точки Мішуревича несподівано зупинився
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-diving =
    Автопілот: занурення в точку Мішуревича ({ $preperiod },{ $period })
    Будь-яка дія зупиняє
# ($preperiod,$period) names the point: keep the notation. $octaves: how often the structure repeats (2.37); $twist: its turn in degrees, signed (+12.5).
autopilot-misi-diving-repeat =
    Автопілот: занурення в точку Мішуревича ({ $preperiod },{ $period })
    Структура повторюється кожні { $octaves } октави, закручення { $twist }° · будь-яка дія зупиняє
# $limit: the magnification, as written (1e300).
autopilot-dive-limit = Автопілот: досягнуто межі занурення (~{ $limit }×)
# $have: the fixed iteration count; $typical: what this depth typically needs.
autopilot-dead-end-short =
    Автопілот зупинено: більше немає куди цілитися — кількість ітерацій зафіксовано на { $have }, а ця глибина зазвичай потребує ~{ $typical }
    Увімкніть автоітерації або збільште кількість
# $limit: the fixed iteration count.
autopilot-dead-end-cap =
    Автопілот зупинено: більше немає куди цілитися — усе у вигляді доходить до фіксованої межі ітерацій { $limit }
    Увімкніть автоітерації або збільште кількість
autopilot-dead-end = Автопілот: попереду немає деталей (зупинено)

## Tours (camera scripts)

# The name of the file type in the file dialog's filter list.
script-file-filter = Сценарій Fractadyne (TOML)
# $name: the script's name.
script-finished = Сценарій завершено — «{ $name }»
# The title of the notice that follows (the file's path and the error, in English).
script-load-failed = Не вдалося завантажити сценарій

## Installing a language file

# The name of the file type in the file dialog's filter list.
language-file-filter = Файл мови Fractadyne

## The notice when the graphics driver's shader cache stops keeping shaders. The text is shown in a fixed-width font with its line breaks as written: break the lines where they read well.

driver-cache-title = Графічний драйвер не зберігає шейдери
# $count: how many shaders compiled again; $pipeline: the slowest one's name (a technical id: keep it); $seconds: its time in seconds (7.5).
driver-cache-body =
    Ваш графічний драйвер у цьому сеансі знову скомпілював { $count ->
        [one] { $count } шейдер
        [few] { $count } шейдери
        [many] { $count } шейдерів
       *[other] { $count } шейдера
    } Fractadyne,
    хоча раніше вже компілював кожен із них (найповільніший:
    { $pipeline }, { $seconds } s). Його кеш шейдерів не зберігає
    скомпільовані шейдери — найчастіше тому, що кеш заповнений. Доки це
    не зміниться, експорт, глибокі вигляди й запуск Fractadyne можуть
    щоразу витрачати секунди на компіляцію.
# $folder: the cache folder's path; $size: its size in MB (1024). "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia-folder =
    Як виправити: закрийте Fractadyne, потім видаліть уміст теки
      { $folder }   ({ $size } MB)
    Драйвер відбудує кеш. Щоб він знову не заповнився, збільште
    NVIDIA Control Panel > Manage 3D settings > Shader Cache Size.
# "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia =
    Як виправити: збільште NVIDIA Control Panel > Manage 3D settings > Shader
    Cache Size або очистьте кеш шейдерів драйвера.
# "AMD Software > Graphics > Advanced > Reset Shader Cache" names AMD's own settings: use the names AMD's software shows in this language.
driver-cache-fix-amd = Як виправити: AMD Software > Graphics > Advanced > Reset Shader Cache.
driver-cache-fix-other = Як виправити: очистьте кеш шейдерів графічного драйвера.

## Why a language file could not be installed: each ends the sentence "{ $file } is not a usable language file: …".

language-install-not-utf8 = це не текст UTF-8
language-install-too-large = завеликий
# "fractadyne-<language>.ftl" is the file name to use: keep it as written.
language-install-no-tag = у ньому не вказано, яка це мова (назвіть його fractadyne-<language>.ftl)
language-install-english = англійська вбудована в програму
language-install-empty = він не перекладає жодного повідомлення цієї версії
language-install-no-folder = немає теки налаштувань

## The notice after a session that did not end cleanly, when its iteration setting was set aside.

# $base: the iteration count automatic iterations start from.
quarantine-setting-auto = автоматичні ітерації від бази { $base }
# $count: the fixed iteration count.
quarantine-setting-fixed =
    { $count ->
        [one] { $count } ітерація
        [few] { $count } ітерації
        [many] { $count } ітерацій
       *[other] { $count } ітерації
    }
# $end: "device" when the session lost its graphics device, "unclean" otherwise (the variants hold the wording). $depth: the magnification, as written (1.2e148). $setting and $restored: an iteration setting (quarantine-setting-auto or -fixed).
quarantine-notice =
    { $end ->
        [device] Попередній сеанс утратив графічний пристрій на { $depth }×, коли було встановлено { $setting }. Цей сеанс натомість почався з налаштуванням «{ $restored }», тож він не може одразу повернутися до того самого навантаження.
       *[other] Попередній сеанс завершився некоректно на { $depth }×, коли було встановлено { $setting }. Цей сеанс натомість почався з налаштуванням «{ $restored }», тож він не може одразу повернутися до того самого навантаження.
    }

## The second graphics card's status line, under its setting in the control panel.

gpu2-state-off = Вимкнено: усе рендерить відеокарта цього вікна.
gpu2-state-starting = Запуск…
# $name: the card's name (and its graphics API).
gpu2-state-running = Використовується: { $name }
# $why: the error, as the graphics driver reports it.
gpu2-state-failed = Не вдалося запустити: { $why }
# $name: the card's name.
gpu2-state-lost = Зупинено: { $name } втрачено. Відтепер — одна карта; виберіть її знову, щоб повторити спробу.
# $state: one of the lines above. "--worker-gpu" is a command-line option: keep it.
gpu2-state-pinned = { $state } (задано через --worker-gpu)

## The custom-formula dialog: how deep the formula renders (one line under the formula).

formula-depth-deep = Глибоке наближення збуренням, розширений діапазон після 1e28x, як у вбудованих фракталах (кожен крок ітерується: для власних формул поки немає апроксимації рядом чи BLA).
# $why: what the formula has that keeps it off the deep-zoom path (one of the formula-depth-why messages below).
formula-depth-direct = Прямий рендеринг ({ $why } — для цього поки немає форми глибокого наближення): різкий, доки один крок c в одинарній точності не охопить піксель, — приблизно від 1e4x до 1e5x, а далі від початку координат — на меншій глибині.
formula-depth-why-init = розділ ініціалізації
formula-depth-why-bailout = власна умова втечі
formula-depth-why-variable = змінна, що зберігається від кроку до кроку
formula-depth-why-previous = попередня ітерація
formula-depth-why-comparison = порівняння або блок if
# floor, ceil, trunc and round are functions of the formula language: keep them.
formula-depth-why-rounding = округлення (floor, ceil, trunc, round)
# "maxit" is a word of the formula language: keep it.
formula-depth-why-maxit = maxit
formula-depth-why-power = степінь зі змінним показником
formula-depth-why-perturbed = уже збурена програма

## Image and frame sizes: the presets described in words (the others are standard names: HD 720p, WUXGA, DCI 4K).

# $size: the pixel size (2048×2048).
size-preset-square = Квадрат — { $size }
# $ratio: the aspect ratio (9:16); $size: the pixel size.
size-preset-portrait = Портрет { $ratio } — { $size }
# $ratio: the aspect ratio (32:9); $size: the pixel size.
size-preset-super-ultrawide = Суперультраширокий { $ratio } — { $size }

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
fractal-family-custom = Власна
# Conway's Game of Life.
fractal-family-life = Life
fractal-family-lsystem = L-система
# A family at a power: "Multibrot 3", "Burning Ship 4". $family: a name above; $power: the power.
fractal-with-power = { $family } { $power }

## Names of choices: colouring methods, orbit traps, palette animation, snapshots, angle units.

color-method-smooth = Плавна ітерація
color-method-stripe = Середнє смуг
color-method-triangle = Нерівність трикутника
color-method-trap = Пастка орбіти
color-method-distance = Оцінка відстані
color-method-decomposition = Декомпозиція
# The shape an orbit trap catches the orbit with.
trap-point = Точка
trap-cross = Хрест
trap-circle = Коло
# Palette animation: none.
palette-anim-off = Вимкнено
palette-anim-forward = Уперед
palette-anim-reverse = Назад
palette-anim-pingpong = Пінг-понг
# Palette animation: a new random gradient every so often.
palette-anim-random = Випадкові градієнти
# What the Snapshot button does.
snapshot-mode-ask = Питати щоразу
snapshot-mode-screen = Знімок екрана
snapshot-mode-render = Повний рендер
# The unit of an angle typed into Go to location's polar fields.
angle-degrees = градуси
angle-radians = радіани
angle-turns = оберти

## Report an issue: the choices. (The report itself is sent in English.)

issue-kind-crash = Збій / зависання програми
issue-kind-rendering = Неправильний рендеринг
issue-kind-performance = Проблема продуктивності
issue-kind-ui = Проблема інтерфейсу / зручності
issue-kind-feature = Запит функції
issue-kind-other = Інше
severity-low = Низька
severity-medium = Середня
severity-high = Висока
severity-blocking = Блокувальна
repro-always = Завжди
repro-sometimes = Іноді
repro-once = Сталося один раз
repro-cannot = Не вдається відтворити

## The benchmark's depths and the diagnostics' tests.

# $zoom: the dive's final magnification (1e32).
bench-depth-standard = Стандартна ({ $zoom }×)
# $zoom: the dive's final magnification (1e48).
bench-depth-ultra = Ультраглибока ({ $zoom }×)
diag-test-selftest = Самотест
diag-test-uitest = Тест інтерфейсу
diag-test-gputest = Перевірка арифметики GPU

## The status bar's readouts: each word is followed by a value in a slot of fixed width. Keep the words short (one word each).

status-center = центр
status-cursor = курсор
status-zoom = масштаб
# Short for iterations.
status-iter = ітер
# The period of the minibrot jumped to.
status-period = період
# A Game of Life view's scale (followed by "1 px = 4 cells" or "8 px / cell").
status-scale = масштаб
# A Game of Life view's generation. Short.
status-generation = пок.
# A Game of Life view's population. Short.
status-population = поп.
# An L-system's order (how many times its rules were applied).
status-order = порядок
# How many segments and shapes an L-system drew.
status-drawn = намальовано
# $value: how many cells one pixel covers (4, 9.99e300). "px" is the unit symbol: keep it.
life-scale-cells = 1 px = { $value } кл.
# $value: how many pixels one cell covers (8). "px" is the unit symbol: keep it.
life-scale-pixels = { $value } px / кл.

## The status bar's limit diagnostic: a short label (after a warning sign), and what it means when hovered.

# The view is as deep as this location can be rendered. Short.
status-limit-depth = межа глибини
# Every pixel ran out of iterations. Short.
status-limit-exhausted = ітерації вичерпано
# Pixels are capped at the iteration budget. Short.
status-limit-capped = ітерації обмежено
# $pixels: how many pixels wide one step of the coordinates is; $sharp_to: the magnification the view is sharp to (written out, 1,234,567).
status-limit-direct-detail = Цей вигляд рендериться безпосередньо на GPU з одинарною точністю, де один крок координат тут завширшки { $pixels } px: сусідні пікселі обчислюють ту саму точку, тож зображення розпадається на блоки. У цьому розташуванні воно різке приблизно до { $sharp_to }×. Ця формула поки не має шляху глибокого наближення (збурення), тож подальше наближення не показує нових деталей.
# $samples: the GPU buffer's capacity in orbit samples; $iterations: the iterations pixels are limited to.
status-limit-depth-detail = Опорна орбіта досягла межі буфера GPU (відліків: { $samples }), так і не втікши, тож пікселі тут обмежено ~{ $iterations } ітераціями, і дрібніші деталі на цьому вигляді неможливо обчислити — ні наживо, ні під час експорту. Це поточна межа глибини для цієї родини розташувань.
# $budget: the largest iteration budget this view can be given.
status-limit-exhausted-detail = Кожному пікселю все одно забракло ітерацій на найбільшому бюджеті, який можна дати цьому вигляду ({ $budget }). Або вигляд тут усередині множини (на глибині це нормально), або йому потрібно більше ітерацій, ніж програма зараз може надати, — зображення в будь-якому разі те саме, тож бюджет повернуто до дешевого.
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations" is the setting's name in the Rendering panel.
status-limit-capped-max = { $percent }% пікселів забракло ітерацій на бюджеті { $budget }, а ті, що втікають, завершуються якраз біля цієї стелі — вигляд недоопрацьований. «Ітерації» вже на максимумі, тож цьому вигляду потрібно більше, ніж програма зараз може дати.
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations" and "Rendering" are the names of a setting and a panel section.
status-limit-capped-raise = { $percent }% пікселів забракло ітерацій на бюджеті { $budget }, а ті, що втікають, завершуються якраз біля цієї стелі — вигляд недоопрацьований. Збільште «Ітерації» (Керування ▸ Якість), щоб це виправити.
