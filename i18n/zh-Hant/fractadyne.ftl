### Fractadyne user interface: Chinese, Traditional (繁體中文).
### A draft: machine translation (Claude, by Anthropic) of the English file, 2026-10-23, not yet
### reviewed by a native speaker. Corrections are welcome as pull requests (see i18n/README.md).

## Language data: read by Fractadyne, not shown as messages.

# This language's own name, shown in the language menu.
language-name = 繁體中文
# The same name in English, shown beside it in the menu.
language-english-name = Chinese (Traditional)
# The writing system the app must be able to draw (an ISO 15924 code: Latn, Cyrl, Jpan, Hans, Khmr).
language-script = Hant
# Punctuation for counts, durations and sizes. Never used for coordinates or magnifications.
# One character each. A space as the group separator is written as a quoted literal, so it shows:
# { "\u00A0" } (no-break space) or { "\u202F" } (narrow no-break space).
number-decimal = .
number-group = ,
# Whether a native speaker has reviewed the translation: draft (the language menu says so) or
# reviewed. The English file is the source.
language-status = draft

## The language menu (top bar) and the Languages window

# The language menu's last item: opens the Languages window.
language-more = 更多語言…
# The Languages window's title.
language-window-title = 語言
# The Languages window's introduction.
language-window-intro = 以其他語言顯示 Fractadyne 的介面。每種語言就是一個檔案：隨版本一起提供，或在這裡安裝。無論使用哪種語言，座標、檔案與命令列一律維持英文。
# Column headers of the installed-language list.
language-col-language = 語言
language-col-translated = 已翻譯
language-col-from = 來源
# Where a language comes from.
language-source-builtin = 內建
language-source-package = 此版本
language-source-user = 已安裝
language-source-dev = 工作資料夾
# $percent: how much of the interface the file translates, 0–100.
language-coverage = { $percent }%
# $count: messages in the file that failed the checks and show in English instead.
language-dropped = { $count } 則訊息需要檢查
# Buttons of the Languages window.
language-install = 安裝語言檔…
language-install-hover = 選擇一個 Fractadyne 語言檔（.ftl）。它會先經過檢查，再複製到你設定中的語言資料夾，更新後仍會保留。
language-open-folder = 開啟語言資料夾
language-reload = 重新載入
language-reload-hover = 重新讀取語言檔（在你編輯或新增語言檔之後）。
language-use = 使用
# Notices after installing a file. $name: the language's own name; $file: the file's name.
language-installed = 已從 { $file } 安裝 { $name }。
# $file: the file's name; $why: what was wrong with it.
language-install-failed = { $file } 不是可用的語言檔：{ $why }
# Shown when a language file cannot be loaded and English is used instead. $name: the language.
language-load-failed = 無法載入 { $name }，因此 Fractadyne 以英文顯示：{ $why }
# A language whose writing system needs a font this computer does not have (Japanese or Chinese without a system font). $name: the language, in its own name.
language-font-missing = 這台電腦上找不到能顯示 { $name } 的字型，因此無法顯示這種語言。請安裝 { $name } 字型（例如系統本身的語言套件），然後再選擇一次。
# Shown after a draft translation's name in the language menu and the Languages window. Short.
language-draft = 草稿
language-draft-hover = 草稿：這份翻譯尚未經過母語人士審閱。歡迎以 pull request 的形式修正該語言的檔案。
# The heading over the decimal-mark choice, in the language menu and the Languages window.
language-decimal-mark = 小數點符號
language-decimal-mark-hover = 數字的寫法：用於時間、大小與數量。座標與放大倍率一律使用小數點，因為檔案與其他程式都以這種方式讀取。
# $sample: a number written with this language's own punctuation (1,234.5).
language-decimal-language = 依此語言的寫法（{ $sample }）
# $sample: a number written with a decimal point (1,234.5).
language-decimal-point = 句點（{ $sample }）
# $sample: a number written with a decimal comma (1.234,5).
language-decimal-comma = 逗號（{ $sample }）

## Asked once when the system's language changes

# The window's title.
language-changed-title = 語言
# $new: the system's new language, in its own name.
language-changed-question = 你的系統語言現在是 { $new }。要在 Fractadyne 中使用這種語言嗎？
# Buttons. $name: a language's own name.
language-changed-use = 使用 { $name }
language-changed-keep = 保留 { $name }

## The welcome window (first run)

welcome-title = 歡迎使用 Fractadyne
welcome-intro = 一套探索 Mandelbrot 集合及其相關碎形的深度縮放工具。以下幾個操作能讓你快速上手：
# The language row, at the top of the window.
welcome-language = 語言
# Controls: the key or gesture on the left, what it does on the right.
welcome-key-drag = 拖曳
welcome-key-drag-does = 平移畫面
welcome-key-scroll = 捲動
welcome-key-scroll-does = 朝游標位置放大／縮小
welcome-key-space = 按住 Space
welcome-key-space-does = 平滑連續放大（Shift+Space 縮小）
# Shown after the click-zoom toolbar icon.
welcome-key-click = 然後按一下
welcome-key-click-does = 深入按下的那一點（工具列）
welcome-key-m-does = 跳到最近的迷你 Mandelbrot，並以它本身的尺度顯示
welcome-landmarks = 跳到一個地標開始探索：
welcome-theme = 主題
welcome-watermark = 顯示「Fd」標記
welcome-watermark-hover = 畫面右下角的小型品牌標記。關閉後，即時畫面與匯出的影像都不再顯示它。
welcome-updates = 更新
welcome-check-updates = 檢查更新
welcome-check-updates-hover = 程式啟動時向 GitHub 查詢最新版本。不會上傳任何內容。
welcome-files = 檔案
welcome-no-config = （沒有設定目錄）
welcome-open = 開啟
welcome-rendering-on = 算繪裝置
welcome-open-help = 開啟完整說明（F1）
# $icon: the Settings icon.
welcome-more-in-settings = 更多選項請見「檔案 → { $icon } 設定」
welcome-get-started = 開始使用

## Menus: shared

# A percentage in a menu (UI scale). $value: the number.
common-percent = { $value }%
# A state shown after a menu item's name, in brackets.
menu-state-connected = 已連線
menu-state-rendering = 算繪中
menu-state-running = 執行中

## Navigate ▸ Bookmarks (also the toolbar's star)

menu-bookmarks = 書籤
menu-bookmarks-add = 加入目前畫面
menu-bookmarks-manage = 管理…

## The fractal picker (Fractal menu and the toolbar's dropdown)

menu-family-powers = 高次方
menu-family-automata = 自動機
menu-family-lsystems = L 系統
menu-fractal-custom = 自訂公式…
menu-fractal-library = 公式庫…
menu-fractal-library-hover = 已儲存的自訂公式：套用、編輯、匯入及匯出
# The "Show" group: which picture of the formula is drawn. $name: the formula's name (Mandelbrot, Burning Ship…).
menu-show = 顯示
menu-show-set = { $name } 集合
menu-show-set-hover = 參數平面，也就是 { $name } 集合本身。

    它的每一點都是一個不同的 Julia 集合；這是所有 Julia 集合的地圖。
menu-show-julia = Julia 集合
menu-show-julia-hover = { $name } 集合中的一點，單獨繪成一張圖：固定這一點時得到的形狀。

    在「並列（連動）」中將游標停在地圖上來選取這一點；在那裡按一下即可固定。
menu-show-julia-none = { $name } 沒有自由參數，因此沒有 Julia 集合。
menu-show-both = 並列（連動）
menu-show-both-hover = 左邊是 { $name } 集合，右邊是游標所在那一點的 Julia 集合。

    在地圖上移動游標，看著 Julia 集合跟著變化；這是了解兩者關係最快的方法。
menu-show-both-none = { $name } 沒有可以搭配的 Julia 集合。

## File menu

menu-file = 檔案
menu-file-open-view = 開啟畫面或位置…
menu-file-gallery = 圖庫…
menu-file-export = 匯出影像…
menu-file-snapshot = 快照
menu-file-open-life = 開啟生命遊戲圖樣…
menu-file-open-life-hover = RLE、.cells 或 Life 1.05/1.06 檔案，以生命遊戲的宇宙顯示。
menu-file-life-text = 生命遊戲圖樣文字…
menu-file-life-text-hover = 以文字貼上圖樣（LifeWiki 提供的 RLE 格式）。
menu-file-save-life = 儲存生命遊戲圖樣…
menu-file-save-life-hover = 將宇宙目前的狀態存成 RLE。
menu-file-open-lsystem = 開啟 L 系統…
menu-file-open-lsystem-hover = Fractint 的 .l 檔案（從其中的條目挑選）或 .lsys 系統。
menu-file-save-lsystem = 儲存 L 系統…
menu-file-save-lsystem-hover = 將系統的文字存成 .lsys 檔案。
menu-file-export-svg = 匯出 SVG…
menu-file-export-svg-hover = 將畫面存成 SVG 繪圖：線條與形狀皆為向量，並保留畫面的色彩。
menu-file-render-client = 算繪用戶端…
menu-file-render-client-hover = 將這台電腦的 GPU 借給算繪農場：連線到主控端（在另一台電腦上執行「工具 ▸ 在算繪農場上算繪…」），替它算繪影格。
menu-file-settings = 設定
menu-file-reset = 重設應用程式狀態…
menu-file-reset-hover = 刪除所有已儲存的狀態（工作階段、書籤、縮圖）並重新開始。會先要求確認。
menu-file-quit = 結束

## File ▸ Settings

menu-settings-fps = 影格速率上限
menu-settings-fps-uncapped = 不限制
# $fps: frames per second.
menu-settings-fps-n = { $fps } FPS
menu-settings-ui-scale = 介面縮放（字型大小）
menu-settings-theme = 主題
menu-settings-updates = 更新
menu-settings-updates-stable-hover = 最新的穩定版
menu-settings-updates-beta-hover = 包含預先發行版在內的最新版本
menu-settings-updates-on-launch = 啟動時檢查更新
menu-settings-updates-on-launch-hover = 否則請透過「說明 → 檢查更新」手動檢查。
# Followed by its shortcut, (Ctrl+S).
menu-settings-snapshot = 快照
menu-settings-snapshot-ask-hover = 每次按下時都讓你選擇
menu-settings-snapshot-screen-hover = 完全依照螢幕上的樣子儲存畫面（立即完成，PNG）
menu-settings-snapshot-render-hover = 依匯出設定在背景完整算繪（深層畫面可能需要很長時間）
menu-settings-refcache = 參考軌道快取…
menu-settings-refcache-hover = 儲存在磁碟上的深度縮放參考軌道，讓你回到某個位置時幾秒內就能算繪完成。這裡顯示它的位置、大小、上限，以及清除它的方法。

## Fractal menu

menu-fractal = 碎形

## View menu

menu-view = 檢視
menu-view-zoom-home = 縮小至完整畫面
menu-view-zoom-home-hover = 縮小至完整的初始畫面（有動畫）
menu-view-reset = 重設為預設畫面
menu-view-reset-hover = 重設為此碎形的預設畫面（立即）
# Followed by its shortcuts.
menu-view-undo = 復原畫面
menu-view-redo = 重做畫面
menu-view-control-panel = 控制面板
menu-view-control-panel-hover = 顯示／隱藏右側的控制面板。
menu-view-minimap = 小地圖總覽
menu-view-minimap-hover = 顯示一個小型的初始畫面總覽，附有「你在這裡」標記與縮放深度。按一下即可跳到該區域。
menu-view-perf-panel = 效能面板
menu-view-timestamp = 顯示時間戳記
menu-view-timestamp-hover = 在畫面上繪製一個大型的經過時間時鐘。它顯示的「+12.345s」與記錄檔替每一行加上的時間戳記相同，因此對於只在移動時才會出現的問題，可以將螢幕錄影與記錄檔逐格對照。預設為關閉。
menu-view-zoom-target = 顯示自動縮放目標
menu-view-zoom-target-hover = 自動縮放執行時，框出它正在放大的畫面區域（再放大幾次後將填滿螢幕的部分），從該區域的四角連線到螢幕的四角，並在它瞄準的點上畫一個圓環。預設為關閉。
menu-view-live-panel = 即時算繪面板
menu-view-live-panel-hover = 覆蓋在畫面上的一塊前面板，呈現即時算繪器的運作情形：每個影格派送了什麼、螢幕上的圖像有多舊、在各疊代區段間的逐段推進、參考軌道建構器，以及底層的電腦。按 B 可循環切換，按面板角落的 × 可關閉；可拖曳到任一角落；將游標停在儀表上，可看到它的意義，以及正常與出問題時的樣子。
# The live render panel's sizes.
menu-view-live-off = 關閉
menu-view-live-off-hover = 不顯示面板。
menu-view-live-strip = 條狀
menu-view-live-strip-hover = 輝光管讀數與指示燈排。
menu-view-live-panel-size = 面板
menu-view-live-panel-size-hover = 各個儀表：推進錶盤、圖像年齡、圖塊、GPU 回合、深度錶盤、指示燈。
menu-view-live-full = 完整
menu-view-live-full-hover = 面板加上機房：步數、GPU、記憶體、核心、像素。
menu-view-live-opacity = 不透明度
menu-view-watermark = 顯示「Fd」浮水印
menu-view-watermark-hover = 在即時畫面與匯出影像的右下角繪製一個不顯眼的「Fd」品牌標記。預設為開啟。

## Color menu

menu-color = 色彩
menu-color-method = 方法
menu-color-palette = 調色盤
menu-color-custom = 自訂
menu-color-duotone = 雙色調
menu-color-binary = 二元（集合）
menu-color-binary-hover = 平面雙色：集合內與集合外，沒有漸層。
menu-color-edit-gradient = 編輯漸層…
menu-color-normalize = 正規化深層色彩
menu-color-normalize-hover = 當密集的區域原本會看起來像斑點雜訊時，將調色盤重新對應到畫面量測到的逃逸範圍。不會混疊的畫面則維持原樣；勾選「一律貼合範圍」即可無條件重新對應。僅適用於平滑方法。
menu-color-more = 循環、位移、動畫與效果：控制面板 ▸ 著色

## Tools menu

menu-tools = 工具
# Followed by its shortcut, (M).
menu-tools-find-minibrot = 尋找迷你 Mandelbrot 並縮放至該處
menu-tools-find-minibrot-hover = 以牛頓法將畫面中心吸附到附近迷你 Mandelbrot 的精確中心，回報它的週期，並縮放到該迷你 Mandelbrot 本身的尺度，往往一步就跨越許多個數量級。已經比它更深了？那就只移動中心。
menu-tools-solver = 牛頓／Misiurewicz 求解器…
menu-tools-solver-hover = 求出精確的迷你 Mandelbrot 中心（週期與大小），或最接近畫面的 Misiurewicz 點：以全精度執行牛頓法，並附上排斥乘數 λ。會開啟位置對話方塊，求解器就在其中。
# Followed by its shortcut, (A).
menu-tools-autopilot = 自動縮放（自動駕駛）
menu-tools-autopilot-stop = 停止自動駕駛
menu-tools-autopilot-hover = 免動手的連續深度縮放：朝細節最豐富的區域下潛，並一路重新調整方向。任何瀏覽操作都會讓它停止。
menu-tools-priority = 優先：
menu-tools-priority-speed = 速度
menu-tools-priority-speed-hover = 以設定的速度縮放；一有含細節的影格就更新畫面。
menu-tools-priority-quality = 品質
menu-tools-priority-quality-hover = 只顯示完全解析的影格；縮放會等待這些影格。
menu-tools-target = 目標：
menu-tools-target-detail = 細節
menu-tools-target-detail-hover = 畫面中最豐富的邊緣，隨著下潛重新選擇。
menu-tools-target-misiurewicz = Misiurewicz 點
menu-tools-target-misiurewicz-hover = 最近的 Misiurewicz 點，求解一次後精確地下潛進去（僅限 Mandelbrot）。
menu-tools-orbits = 軌道疊加
menu-tools-orbits-hover = 繪製游標下那一點的疊代路徑。動畫選項：控制面板 ▸ 疊加顯示。
menu-tools-orbits-normalize = 正規化（貼合畫面）
menu-tools-orbits-normalize-hover = 讓軌道貼合整個畫面，在任何縮放下都保持良好的取景（而不是透過檢視區對應，那樣在深處時軌道會飛出螢幕）。
menu-tools-play-tour = 播放導覽…
menu-tools-play-tour-hover = 選擇要播放的導覽腳本。工具列的 ▶ 只要按一下就能重播上一個導覽。
menu-tools-tour-from-view = 從目前畫面建立導覽…
menu-tools-tour-from-view-hover = 建立一段從完整畫面一路放大到目前畫面的導覽：可加上字幕並設定下潛的時間長度。存成導覽腳本（.toml），可以編輯、播放或算繪。
menu-tools-render-tour = 算繪導覽…
menu-tools-render-tour-hover = 在獨立的處理程序中，將載入的導覽算繪成 PNG 影格序列（也可選擇另外輸出 mp4）。
menu-tools-close-player = 關閉導覽播放器
menu-tools-close-player-hover = 停止導覽、關閉播放器，並還原你自己的疊代與著色設定
menu-tools-render-farm = 在算繪農場上算繪…
menu-tools-render-farm-hover = 在多台電腦上算繪載入的導覽：由這台電腦控制工作，其他電腦透過「檔案 ▸ 算繪用戶端…」加入。
menu-tools-benchmark = 基準測試…
menu-tools-benchmark-hover = 測量算繪速度：使用目前的設定，或執行可在不同電腦間比較的標準化測試（固定的解析度與設定）。燒機模式會反覆執行，以檢查穩定性／過熱降頻。
menu-tools-last-benchmark = 顯示上次的基準測試

## Navigate menu

menu-navigate = 瀏覽
menu-navigate-goto = 前往位置…
menu-navigate-goto-hover = 輸入中心／縮放、跳到知名的點，或以牛頓法求解畫面附近的 Misiurewicz／迷你 Mandelbrot 特徵。
menu-navigate-misiurewicz = Misiurewicz 探索器…
menu-navigate-misiurewicz-hover = 瀏覽小型 Misiurewicz 類型的樣貌（縮圖上附有每個點的縮放週期與扭轉），並以牛頓法將其中任何一個求解到任意深度。（Mandelbrot。）
menu-navigate-exact = 精確點（運算式）
menu-navigate-exact-note = 主心形區上各個圓盤的根點，以精確的運算式表示（2π 某個分數的 cos/sin）。座標會保留為運算式，因此放大時會精確地停在該點上。
menu-navigate-random = 隨機位置
menu-navigate-random-hover = 跳到一個隨機的、細節豐富的邊界點
menu-navigate-import-kfr = 匯入 .kfr…
menu-navigate-import-kfr-hover = 載入 Kalles Fraktaler 位置檔
menu-navigate-share = 分享位置…
menu-navigate-share-hover = 複製／貼上／儲存／載入一個自成一體的位置（.fdn）：碎形、全精度中心、縮放、著色。

## Help menu

menu-help = 說明
# Followed by its shortcut, (F1).
menu-help-reference = 說明與參考資料…
menu-help-welcome = 歡迎／快速入門…
menu-help-welcome-hover = 首次執行時的概覽：基本操作與幾個地標。
menu-help-diagnostics = 診斷…
menu-help-diagnostics-hover = 檢查 Fractadyne 在你的硬體上是否正常運作，並將結果附加到問題回報中
menu-help-report = 回報問題…
menu-help-report-hover = 提交 GitHub issue（或寄送電子郵件），並附上當機報告、記錄檔與位置
menu-help-accelerated = 更快的深度縮放…
menu-help-accelerated-hover = 一個選用的版本，計算深度縮放參考軌道的速度快 2.5-6.4 倍。影像完全相同；你的設定會沿用。
menu-help-check-updates = 檢查更新
# $track: "stable" or "beta", the update track in use (the variants hold the wording).
menu-help-check-updates-hover =
    { $track ->
        [beta] 向 GitHub 檢查是否有更新的 Beta（預先發行版）版本
       *[other] 向 GitHub 檢查是否有更新的穩定版
    }
# $version: the new version; $channel: "beta" for a pre-release, "stable" otherwise (the variants hold the wording).
menu-help-update-available =
    { $channel ->
        [beta] { $version }（Beta 版）已推出：下載
       *[other] { $version }（穩定版）已推出：下載
    }
menu-help-up-to-date = 已是最新版本
menu-help-tagline = 原生碎形探索程式
menu-help-license = 授權
# $year and $holder: the copyright year and holder, as the code gives them.
menu-help-copyright = © { $year } { $holder }。雙重授權；你可以自行選擇依其中任一授權使用。
menu-help-source = 原始碼

## The toolbar (its buttons are icons; these are their tooltips)

menu-toolbar-julia = Julia
menu-toolbar-julia-hover = 顯示此公式的 Julia 集合，填滿整個視窗
menu-toolbar-dual-hover = 並列（連動）：左邊是集合，右邊是游標所在那一點的 Julia 集合
menu-toolbar-open-hover = 開啟畫面或位置（PNG / EXR、.fdn、.kfr）
menu-toolbar-gallery-hover = 圖庫
menu-toolbar-export-hover = 匯出影像…
menu-toolbar-share-hover = 分享位置：複製、貼上、儲存或載入一個自成一體的 .fdn（碎形、全精度中心、縮放、著色）
menu-toolbar-zoom-in-hover = 放大
menu-toolbar-zoom-out-hover = 縮小
# $factor: the click-zoom factor (4, 10…).
menu-toolbar-click-zoom-hover = 點按縮放（{ $factor }×）：按左鍵深入該點，按右鍵退回（拖曳仍可平移）。倍數在「控制項 ▸ 瀏覽」中設定。
# Added to the click-to-zoom tooltip in the dual view.
menu-toolbar-click-zoom-dual = 開啟此功能時，以 Ctrl+按一下來釘選 Julia c。
menu-toolbar-autopilot-running-hover = 自動縮放執行中，按一下即可停止
menu-toolbar-autopilot-hover = 自動縮放：朝細節下潛（A）
menu-toolbar-reset-hover = 重設畫面（立即）
menu-toolbar-home-hover = 縮小至初始畫面（有動畫）
menu-toolbar-bookmarks-hover = 書籤：儲存此畫面，或跳到已儲存的畫面
# $name: the palette's name.
menu-toolbar-next-palette-hover = 下一個調色盤（{ $name }）
# $file: the last tour's file name.
menu-toolbar-play-last-hover = 播放導覽：{ $file }（按一下）· 重播上一個導覽；可透過「工具 ▸ 播放導覽…」選擇其他導覽
menu-toolbar-play-hover = 播放導覽…（工具 ▸ 播放導覽）
menu-toolbar-speed-hover = 播放速度
menu-toolbar-fullscreen-hover = 切換全螢幕

## Dialogs: words shared by many windows

# A button that closes a window without doing what it offers.
dlg-cancel = 取消
# A button that dismisses a window (nothing in it is pending).
dlg-close = 關閉
# A button that copies the window's text to the clipboard (after a copy icon).
dlg-copy = 複製
# A button that jumps to a location.
dlg-go = 前往
# A button that fills the fields from the view on screen.
dlg-use-current = 使用目前畫面
# A notice after a file could not be written. $error: the system's reason, as it gives it.
dlg-save-failed = 儲存失敗：{ $error }
# A notice when no graphics device is available to render with.
dlg-gpu-not-available = 無法使用 GPU
# The name of the plain-text file type in a save dialog (the extension .txt is added by the code).
dlg-filter-text = 文字

## Asked after a session that did not shut down cleanly

dlg-crash-title = Fractadyne 未正常關閉
dlg-crash-intro = 上一個工作階段意外結束。這台電腦上已儲存一份報告，要把它傳送出去嗎？
dlg-crash-review = 在你檢閱之前，不會傳送任何內容：下一個畫面會確切顯示將包含哪些內容，而系統資訊只要取消勾選一個核取方塊即可排除。
dlg-crash-report = 回報…
dlg-crash-report-hover = 開啟「回報問題」，並附上當機報告與當機時的位置。
dlg-crash-not-now = 暫時不要
dlg-crash-dont-ask = 當機後不再詢問
dlg-crash-dont-ask-hover = 報告仍會儲存在本機，而且「說明 ▸ 回報問題」隨時都能使用。

## Asked after a session that ended with a second graphics device in use

dlg-second-device-title = 要關閉第二個圖形裝置嗎？
# $device: the graphics device's name and details, as the system gives them.
dlg-second-device-intro = 上一個工作階段在使用第二個圖形裝置（{ $device }）時意外結束。它可能就是原因。
dlg-second-device-note = 關閉後，就由繪製視窗的顯示卡算繪所有內容，和以前一樣。你可以在「進階 ▸ 第二張顯示卡」中重新開啟它。
dlg-second-device-off = 關閉它
dlg-second-device-keep = 保持開啟

## Shown when the previous session's iteration setting was set aside

dlg-quarantine-title = 疊代設定已擱置
dlg-quarantine-change = 你隨時可以在「控制項 ▸ 品質」中變更。
dlg-quarantine-keep = 保留預設值
# A button. $setting: the iteration setting the previous session used (such as "20,000 iterations").
dlg-quarantine-restore = 還原為 { $setting }

## Navigate ▸ Go to location

dlg-goto-title = 前往位置
# Before two choices: "x, y" (cartesian) and the polar form below.
dlg-goto-enter-as = 輸入方式：
# The polar way of entering a point: an offset plus a radius r at an angle θ. Keep the symbols.
dlg-goto-polar = 位移 + r ∠ θ
# The tooltip of the "?" button beside the entry choice.
dlg-goto-help-hover = 欄位可接受的內容：分數、乘冪、sqrt/sin/cos…、pi 及其他常數。會開啟「說明 → 座標運算式」
# Labels over the entry fields. x0, y0, r, θ, Re and Im are the fields' symbols: keep them.
dlg-goto-polar-offset = 位移中心（x0, y0；純極座標點請保留 0）
dlg-goto-polar-radius = 半徑 r，角度 θ
dlg-goto-re = 中心：實部（Re）
dlg-goto-im = 中心：虛部（Im）
dlg-goto-zoom = 縮放（放大倍率）
# A warning under the zoom field, after a ⚠ sign. $have: the iteration count set; $typical: the count this depth typically needs.
dlg-goto-iter-warning = 「疊代次數」固定為 { $have }；這個深度通常需要約 { $typical }。調高之前，畫面會算繪成一片平坦；或者開啟自動疊代（控制項 ▸ 品質），它會自行調整。
dlg-goto-copy-hover = 將此位置複製到剪貼簿
dlg-goto-cancel-hover = 關閉，不前往任何地方
# The expressions in brackets are examples of what can be typed: keep them as they are.
dlg-goto-paste-note = 貼上別人提供的中心／縮放，或按「複製」來分享。座標可接受運算式：精確分數（-3/4）、函數與常數（-0.5 + 0.25*cos(pi/4)、sqrt、root(x,n)、^；角度以弧度為單位），而實部欄位可接受完整的複數值（(37+16i)/100），會同時填入兩個欄位。
# A section that opens and closes.
dlg-goto-feature = 前往特徵（Mandelbrot）
dlg-goto-feature-poi = 知名的點：
# The dropdown's text before a point is picked.
dlg-goto-feature-poi-pick = 跳到一個值得一看的點…
dlg-goto-feature-solve = 或求解目前畫面附近的特徵：
# Two choices: what to solve for. Misiurewicz is a mathematician's name (a Misiurewicz point).
dlg-goto-feature-misiurewicz = Misiurewicz
dlg-goto-feature-minibrot = 最近的迷你 Mandelbrot
# Labels of two small number fields; k and p are their symbols.
dlg-goto-feature-preperiod = 前週期 k
dlg-goto-feature-period = 週期 p
# The faint text in an empty field: left empty, the value is found automatically.
dlg-goto-feature-auto-hint = 自動
# A small button that empties both fields, so they are found automatically again.
dlg-goto-feature-auto = 自動
dlg-goto-feature-auto-hover = 清空兩個欄位，讓下一次搜尋從畫面中心的軌道偵測前週期與週期。
dlg-goto-feature-kp-hover = 兩者都留空，就會從畫面中心的軌道偵測；找到的值會填在這裡。沒有人看著一個螺旋就能知道它的前週期，而過去必須自行提供前週期，正是這個求解器在實務上無法使用的原因。
# While a solve runs, followed by its elapsed time and the note below.
dlg-goto-feature-solving = 求解中…
dlg-goto-feature-solving-note = （任意精度；深層畫面需要較久）
dlg-goto-feature-cancel-hover = 停止等待。求解本身會在背景執行到完成，其結果會被捨棄，不會留下任何做到一半的狀態。
dlg-goto-feature-find = 在畫面附近尋找
dlg-goto-feature-find-hover =
    從目前的中心以牛頓法求解出精確的特徵（任意精度）。請先瀏覽到特徵附近。

    求解之前，先將上方的「縮放」欄位設為你「想要」的深度：該點會以那個深度所需的精度計算，畫面會直接跳到那裡。這樣只需要一次求解，而不是一長串手動的縮放步驟。
dlg-goto-solve-cancelled = 已取消求解。

## Snapshot (the camera button): its tooltip, and the choice on the first press

# The camera button's tooltip, one per Snapshot setting. Followed by its shortcut, (Ctrl+S).
dlg-snapshot-hover-ask = 快照：儲存所見的畫面，或完整算繪；第一次按下時會詢問
dlg-snapshot-hover-screen = 快照：完全依照所見儲存畫面，存到上次的匯出資料夾
dlg-snapshot-hover-render = 快照：依匯出設定完整算繪，存到上次的匯出資料夾
# The window's title.
dlg-snapshot-title = 快照
dlg-snapshot-question = 「快照」（相機按鈕，Ctrl+S）應該儲存什麼？
# The first choice, and a short tag beside it.
dlg-snapshot-screen = 螢幕擷取
dlg-snapshot-screen-tag = 立即完成
dlg-snapshot-screen-note = 完全依照螢幕上的樣子儲存畫面，使用螢幕解析度，以 PNG 存到上次的匯出資料夾。所見即所得。
# The second choice, and a short tag beside it.
dlg-snapshot-render = 完整算繪
dlg-snapshot-render-tag = 可能需要很久
# $size: the image size, as 1920×1080; $ss: the supersampling factor (1–4); $format: PNG or EXR.
dlg-snapshot-render-note = 依匯出設定算繪：{ $size }、{ $ss }× 超取樣、{ $format }。品質最佳，但深層畫面可能需要數分鐘到數小時。它在背景執行；「檔案 → 匯出影像…」會顯示進度，也可以取消。
dlg-snapshot-remember = 記住我的選擇，不再詢問
dlg-snapshot-change = 你隨時可以在「檔案 → 設定 → 快照」中變更。

## Share location (.fdn)

dlg-share-title = 分享位置
dlg-share-intro = 一個自成一體的位置（碎形、全精度中心、縮放、著色）。複製它來分享，或貼上／載入一個位置後按「套用」。
dlg-share-thumb = 在儲存的檔案中加入縮圖
dlg-share-thumb-hover = 「儲存 .fdn」會嵌入這個畫面的小圖，讓檔案在圖庫中能被認出來。大約增加 55 KB，而且不屬於上方文字的一部分。
dlg-share-copy-hover = 將文字複製到剪貼簿
# Buttons (the first after a save icon).
dlg-share-save = 儲存 .fdn…
dlg-share-load = 載入 .fdn…
dlg-share-apply = 套用
dlg-share-apply-hover = 跳到方塊中的位置
dlg-share-copied = 已複製到剪貼簿。

## Help ▸ Report an issue (the report itself stays in English: it is read by the maintainer)

dlg-report-title = 回報問題
dlg-report-intro = 回報問題最好的地方是 GitHub issue：它公開且可供搜尋，問題修好時你也會知道。用電子郵件也可以。請先檢閱下方的所有內容：在你採取行動之前，不會有任何資料離開你的電腦。
# Labels before three dropdowns.
dlg-report-type = 類型：
dlg-report-severity = 嚴重程度：
dlg-report-repro = 可重現：
dlg-report-describe = 描述發生了什麼事：
# The faint text in the empty description box.
dlg-report-describe-hint = 發生了什麼事、重現的步驟，以及你原本預期的結果。
# Before the checkboxes of what the report carries.
dlg-report-include = 包含：
dlg-report-sysinfo = 系統資訊（版本、作業系統、CPU、GPU、VRAM）
dlg-report-location = 目前位置（.fdn）
dlg-report-log = 最近的記錄
dlg-report-frames = 影格記錄（最後一分鐘，以及最後 40 個影格）
dlg-report-crash = 最新的當機報告
dlg-report-crash-none = 最新的當機報告（找不到）
dlg-report-crash-view = 當機時的位置（.fdn）
dlg-report-crash-view-none = 當機時的位置（.fdn）（找不到）
dlg-report-crash-view-hover = 發生當機或圖形裝置遺失時的確切畫面：這是唯一能讓維護者重現問題的東西。會自動記錄。
dlg-report-test = 診斷測試結果
dlg-report-test-none = 診斷測試結果（請從「說明 → 診斷…」執行一次）
dlg-report-test-hover = 在你自己的硬體上經機器驗證的通過／失敗結果，對維護者來說遠比單純的描述有用
# A section that opens and closes. $size: the report's size, as "12 KB".
dlg-report-preview = 預覽將傳送的內容（{ $size }）
dlg-report-github = 開啟 GitHub issue
dlg-report-github-hover = 開啟已填好標題的新 issue 頁面；報告會複製到你的剪貼簿，請貼到 issue 的內文中（Ctrl+V）。
# Buttons (after a copy and a save icon).
dlg-report-copy = 複製報告
dlg-report-copy-hover = 將所有內容複製到剪貼簿
dlg-report-save = 儲存報告…
dlg-report-save-hover = 存成 .txt 以便附加
dlg-report-by-email = 或透過電子郵件：
dlg-report-gmail = 在 Gmail 中撰寫
# $address: the email address reports go to.
dlg-report-gmail-hover = 開啟 Gmail 撰寫視窗，收件人為 { $address }，主旨已填好；請貼上複製的報告
dlg-report-email = 電子郵件程式…
dlg-report-email-hover = 開啟你的預設郵件程式（mailto）；需要已設定好電子郵件處理程式
dlg-report-tip = 提示：按下以上任一按鈕時，報告都會複製到你的剪貼簿，請貼到 issue 或郵件的內文中（Ctrl+V）。如果可以，也請附上螢幕截圖或範例影像（直接拖曳進去）；這對算繪與介面問題很有幫助。
# Notices in the window after a button is pressed.
dlg-report-opened-github = 已開啟 GitHub，報告在剪貼簿中；請貼到 issue 裡。
dlg-report-copied = 報告已複製到剪貼簿。
# $path: where the report was saved.
dlg-report-saved = 已儲存至 { $path }
dlg-report-opened-email = 已開啟你的電子郵件程式，報告在剪貼簿中；請貼上。
dlg-report-opened-gmail = 已開啟 Gmail 撰寫視窗；請將複製的報告貼到內文中（Ctrl+V）。

## A location whose checksum does not match

dlg-checksum-title = 這個位置可能已損壞
# After a ⚠ sign.
dlg-checksum-mismatch = 它的檢查碼與旁邊的資料不符。
dlg-checksum-why = 這通常表示文字在傳過來的途中被截斷或修改了：從聊天視窗或論壇貼文中只複製到一部分，是常見的原因。
# A label over where the location came from (a file path, or the clipboard).
dlg-checksum-source = 來源
# The two checksums, side by side: the one written in the file, and the one computed from its data.
dlg-checksum-in-file = 檔案中記載
dlg-checksum-from-data = 由資料算出
dlg-checksum-load-note = 你仍然可以載入它：座標可能還是正確的，而且不會覆寫任何東西。任何無法讀取的部分都會在載入後回報。
dlg-checksum-load = 仍要載入
dlg-checksum-load-hover = 不理會不符，直接跳過去
# Notices after loading it. $zoom: the magnification, as 1.2e148.
dlg-checksum-loaded = 已載入 @ { $zoom }×，儘管檢查碼不符
# $zoom: the magnification, as 1.2e148; $note: what the loader found wrong with the location.
dlg-checksum-loaded-note = 已載入 @ { $zoom }×：{ $note }
dlg-checksum-not-loaded = 未載入。

## File ▸ Reset application state

dlg-reset-title = 重設應用程式狀態
# After a ⚠ sign, followed by a list.
dlg-reset-warning = 這會永久刪除所有已儲存的 Fractadyne 資料：
# The list's items, each after a bullet.
dlg-reset-session = 已儲存的工作階段（目前畫面、著色、偏好設定）
dlg-reset-bookmarks = 所有書籤及其縮圖
dlg-reset-refcache = 參考軌道快取（重新造訪位置時會重建）
# A label over the folder that is deleted.
dlg-reset-location = 位置
dlg-reset-note = 此動作無法復原。目前的工作階段在結束時不會再儲存，因此下次啟動時會載入預設值。
dlg-reset-confirm = 全部重設
dlg-reset-done = 應用程式狀態已重設，下次啟動時將載入預設值。
# $error: the system's reason, as it gives it.
dlg-reset-failed = 重設失敗：{ $error }

## Tools ▸ Tour from current view

dlg-tour-title = 從目前畫面建立導覽
dlg-tour-intro = 建立一段從完整畫面一路放大到你目前所在位置的導覽。
# A label before the fractal and magnification the tour ends at.
dlg-tour-destination = 目的地
dlg-tour-notation = 註記（下潛時顯示的選用字幕）
# The faint example text in the empty caption box.
dlg-tour-notation-hint = 例如：深入 Misiurewicz 三臂點…
dlg-tour-duration = 縮放時間長度
# $rate: how fast the dive zooms, in powers of ten a second (as 2.5).
dlg-tour-rate = ≈ 每秒 { $rate } 個數量級
# A small button that puts the duration back to its default.
dlg-tour-reset = 重設
dlg-tour-reset-hover = 依深度而定的預設值
dlg-tour-duration-note = 預設值會隨深度調整，讓下潛過程保持可觀賞的速度。在極深的縮放下，較快的下潛可能會閃現單色影格。
# A button, after a save icon.
dlg-tour-save = 儲存導覽…
# The file type's name in the save dialog.
dlg-tour-filter = Fractadyne 腳本（TOML）
# A notice. $file: the saved file's name. --render-tour is a command-line option: keep it.
dlg-tour-saved = 已儲存導覽 → { $file }。可透過「工具 → 播放導覽」播放，或使用 --render-tour 算繪。

## Navigate ▸ Bookmarks ▸ Manage…

dlg-bookmarks-title = 書籤
# The faint text in the empty name box.
dlg-bookmarks-name-hint = 名稱（選填）
# A button, after a bookmark icon.
dlg-bookmarks-add = 加入目前畫面
dlg-bookmarks-empty = 還沒有書籤。從目前畫面加入一個吧。
# The tooltip of a bookmark's delete (bin) button.
dlg-bookmarks-delete-hover = 刪除

## Tools ▸ Benchmark…

dlg-bench-title = 基準測試
dlg-bench-mode = 模式
dlg-bench-current = 目前設定
dlg-bench-current-hover = 使用你目前的解析度與設定，在即時畫面中播放固定的深度縮放導覽。
dlg-bench-standard = 標準化
dlg-bench-standard-hover = 固定解析度與所有算繪設定，讓分數可以在不同電腦之間比較。
dlg-bench-resolution = 解析度
dlg-bench-depth = 深度
# Standard and Ultra are the two depths in the dropdown this describes.
dlg-bench-depth-hover = 兩種下潛都會進入深層的 floatexp/BLA 路徑；「超深」（1e48×）停留在那裡的時間遠比「標準」（1e32×）長，執行時間也相應較長。
dlg-bench-burnin = 燒機（重複）
dlg-bench-burnin-hover = 反覆執行基準測試，以揭露穩定性問題與過熱降頻。
# The label of a slider: how many times the benchmark runs.
dlg-bench-passes = 次數
# $size: the image size, as 1920×1080; $ss: the supersampling factor; $depth: the dive's final magnification, as 1e32.
dlg-bench-std-summary = 在螢幕外以 { $size }、{ $ss }× SS、Mandelbrot／平滑方式算繪，以 60 個影格下潛至 { $depth }×。
dlg-bench-run = 執行

## The standardized benchmark's progress window

dlg-bench-progress-title = 正在執行基準測試…
# $resolution: the benchmark's resolution, as the dropdown names it.
dlg-bench-progress-mode = 標準化 · { $resolution }
# $pass: the pass running now; $total: how many passes there are.
dlg-bench-progress-pass = 燒機第 { $pass }/{ $total } 輪
dlg-bench-progress-dive = 正在算繪固定的下潛…
# On the progress bar. $done: frames rendered; $total: frames in the dive.
dlg-bench-progress-frame = 影格 { $done }/{ $total }
# $fps: frames per second, as 41.3.
dlg-bench-progress-last = 上一輪：{ $fps } fps

## The benchmark's results

dlg-bench-results-title = 基準測試結果
# Buttons, each after an icon.
dlg-bench-results-run-again = 再執行一次…
dlg-bench-results-save = 儲存…
dlg-bench-results-none = 尚未執行過基準測試。
dlg-bench-results-saved = 基準測試已儲存。

## The accelerated build's notice when its arithmetic libraries are missing

dlg-backend-title = 加速版：改用內建的算術運算
dlg-backend-dont-show = 不再顯示

## File ▸ Gallery…

dlg-gallery-title = 圖庫
# A button that picks the folder to show.
dlg-gallery-folder = 資料夾…
# A button, after a refresh icon.
dlg-gallery-refresh = 重新整理
dlg-gallery-empty = 這個資料夾中沒有 Fractadyne 影像。
# An image's title when it does not name its fractal.
dlg-gallery-untitled = Fractadyne 影像
# $zoom: the image's magnification, as 1.2e148; $saved: when it was saved, as the file gives it.
dlg-gallery-zoom = 縮放 { $zoom }   { $saved }
# The notes saved with the image, in quotation marks. Use your language's quotation marks.
dlg-gallery-notes = 「{ $notes }」
dlg-gallery-open = 開啟此畫面
dlg-gallery-loaded = 已從圖庫載入畫面。

## File ▸ Export image…

dlg-export-title = 匯出影像
# A dropdown of image sizes.
dlg-export-size = 尺寸
# The size dropdown's text for a size set by hand. $size: the size, as 1920×1080.
dlg-export-size-custom-current = 自訂：{ $size }
# The size dropdown's first choice.
dlg-export-size-custom = 自訂…
dlg-export-size-custom-hover = 自行設定寬度；高度依下方的「長寬比」而定。
dlg-export-width = 寬度（px）
# After the width field. $height: the height in pixels, which follows from the aspect.
dlg-export-height-from-aspect = × { $height }（依長寬比）
dlg-export-ss = 超取樣
dlg-export-aspect = 長寬比
# The aspect choice that keeps the window's own shape.
dlg-export-aspect-window = 符合視窗
# Before the choices PNG and OpenEXR.
dlg-export-format = 格式：
dlg-export-hud = 位置 HUD
dlg-export-hud-hover = 在匯出影像的左上角燒入一個縮放等級與座標面板（隨輸出大小縮放；顯示地圖／Mandelbrot 畫面的縮放與中心）。
dlg-export-glitch = 瑕疵修正
dlg-export-glitch-hover = 以多個參考點修正微擾瑕疵。對於非常深（floatexp）的單一畫面匯出會自動略過，讓參考軌道建構與算繪在其他執行緒進行，程式也能保持回應。絕不用於 Mandelbrot 或 Multibrot 畫面（Julia 模式以外）：與精確計算比對後，它在那裡什麼也沒有修正，卻讓匯出時間最多增加到 5 倍。
dlg-export-deep-note = 深層匯出：參考軌道在其他執行緒建構（介面保持可操作）；在此深度會略過瑕疵修正。
# A dropdown, shown in the dual (map + Julia) view, and its three choices.
dlg-export-dual = 雙畫面版面
dlg-export-dual-side = 並排
dlg-export-dual-separate = 分開的檔案
dlg-export-dual-map = 僅地圖
dlg-export-notes = 註記：
# The faint text in the empty notes box.
dlg-export-notes-hint = 與影像一起儲存（≤120 個字元）
# $width and $height: the image size in pixels; $left: how many characters the notes can still take.
dlg-export-output = 輸出：{ $width } × { $height } px（還可輸入 { $left } 個字元）
# When the export's shape differs from the window's. $percent: how much more, as 25.
dlg-export-extra-width = 比視窗多顯示 { $percent }% 的寬度（不會裁切任何內容）
# The same, when the export is taller than the window. $percent: how much more, as 25.
dlg-export-extra-height = 比視窗多顯示 { $percent }% 的高度（不會裁切任何內容）
dlg-export-tiles-note = 在背景執行緒中分圖塊算繪（沒有尺寸上限）。檔案會嵌入畫面與註記，因此可以透過「檔案 ▸ 開啟畫面或位置」重新開啟。
# A label before the button that picks the folder exports go to.
dlg-export-folder = 資料夾：
dlg-export-choose = 選擇…
dlg-export-choose-hover = 選擇匯出的目標目錄
dlg-export-open-after = 完成後開啟
dlg-export-open-after-hover = 影像一寫入完成，就交給系統的檢視程式開啟。
# $count: the graphics cards in this machine (2 or more).
dlg-export-all-gpus = 使用所有顯示卡（{ $count }）
dlg-export-all-gpus-hover = 依圖塊將影像分給這台電腦的每一張顯示卡，每張卡各有一份深層參考軌道。相同的顯示卡得到的影像，與單張卡獨自算繪的完全相同；不同型號的顯示卡在少數像素上會略有差異，以圖塊為單位（實測：看不到接縫）。如果某張卡失敗，它的圖塊會交給其他卡。無法分割的影像（某些著色方法與公式）則使用這個視窗的顯示卡。
# The export's progress.
dlg-export-preparing = 準備中（建構參考軌道）…
dlg-export-saving = 儲存中…
dlg-export-rendering = 算繪中…
# $time: the time since the export started, as 8.3s or 1m 04.0s.
dlg-export-elapsed = 經過時間：{ $time }
dlg-export-cancel-hover = 關閉，不匯出
# A button, after a save icon.
dlg-export-save-as = 另存新檔…
dlg-export-save-as-hover = 選擇檔案名稱與位置
dlg-export-export = 匯出
dlg-export-export-hover = 算繪並儲存到上方的資料夾（自動命名）

## Notices worded at startup (main.rs)

# A notice shown once after starting, when the saved session comes from a newer version of the app. $saved and $handled: version numbers of the session file's format (1, 2…).
app-state-newer = 這個工作階段是由較新版的 Fractadyne 儲存的（狀態 v{ $saved }；此版本可處理 v{ $handled }）。有些設定可能不會套用，而儲存時會以此版本的格式改寫。
# The title of the notice that says the accelerated build's arithmetic libraries are found again (its text comes from elsewhere).
app-notice-mpfr-restored-title = 已恢復加速算術運算
# After Fractadyne restarted itself because the graphics driver reset the GPU.
app-toast-gpu-recovered = 已從圖形裝置重設中復原：由於先前的位置算繪負擔過重，已在初始畫面重新開啟。

## Short notices (toasts)

# When the autopilot reached a view that could not be rendered and put the last good view back.
app-toast-autopilot-view-restored = 自動駕駛已停止：畫面變得無法使用，已還原（已記錄）
# $error: the system's reason, as it gives it (often in English).
app-toast-gradients-save-failed = 無法儲存漸層：{ $error }
# $error: the reason, as the program gives it (often in English).
app-toast-gradients-serialize-failed = 無法序列化漸層：{ $error }
# $name: the name the user gave the gradient.
app-toast-gradient-saved = 已儲存漸層「{ $name }」。
# $error: the reason, as the program gives it (often in English).
app-toast-bookmarks-serialize-failed = 無法序列化書籤：{ $error }
# $error: the system's reason, as it gives it (often in English).
app-toast-bookmarks-save-failed = 無法儲存書籤：{ $error }
# Jumping to a named point given by exact expressions. $name: the point's name (in English).
app-toast-coordinates-failed = { $name }：無法計算座標
# $zoom: the magnification, already formatted (1,234.5 or 1.2e148).
app-toast-random-location = 隨機位置 · { $zoom }×
# Importing a Kalles Fraktaler .kfr location. $error: the reason (in English).
app-toast-import-failed = 匯入失敗：{ $error }
# $zoom: the magnification, already formatted.
app-toast-location-loaded = 已載入位置 @ { $zoom }×

## Go to location: why a jump could not be made (one line per field that failed)

# "Re" is the dialog's real-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-re = Re：{ $error }
# "Im" is the dialog's imaginary-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-im = Im：{ $error }
# The imaginary field was given a complex value. $error: the evaluator's reason (in English).
app-goto-error-im-real = Im：{ $error }。請將虛部輸入為實數，不要加 i
# "Zoom" is the dialog's magnification field.
app-goto-error-zoom-empty = 縮放：請輸入放大倍率，例如 1e50 或 250000
# $zoom: what was typed, in quotes.
app-goto-error-zoom = 縮放：無法讀取 { $zoom }；請輸入正的放大倍率，例如 1e50 或 250000
app-goto-error-invalid = 輸入無效：請檢查座標與縮放。

## Finding a minibrot or Misiurewicz point (the M key and the Go to dialog)

app-feature-needs-holomorphic = 迷你 Mandelbrot 尋找器需要全純族（Mandelbrot／Multibrot）。
app-feature-not-in-julia = 迷你 Mandelbrot 尋找器搜尋的是 Mandelbrot 集合；請關閉 Julia 模式後再使用。
app-feature-busy = 已經在尋找特徵了，一次只能找一個。
app-feature-searching = 正在尋找最近的迷你 Mandelbrot…
app-feature-mandelbrot-only = 特徵尋找僅適用於 Mandelbrot。
app-feature-no-pair = 在搜尋預算內找不到 Misiurewicz 點：該點可能比尋找器在此畫面所能到達的更深，或者中心不在螺旋或分支點附近。
# The name of what was found, used as $feature in the messages below. $period: the minibrot's period (a whole number).
app-feature-minibrot = 週期 { $period } 的迷你 Mandelbrot
# $preperiod and $period: the point's two whole numbers (k, p). $octaves: how often the view repeats when zooming in (2 decimals); $twist: the turn per repeat in degrees.
app-feature-misiurewicz-lambda = Misiurewicz ({ $preperiod },{ $period })：每 { $octaves } 個八度重複一次，扭轉 { $twist }°
# $preperiod and $period: the point's two whole numbers (k, p).
app-feature-misiurewicz = Misiurewicz ({ $preperiod },{ $period })
# Added after the messages below when the fixed iteration count is too low for the new depth. $have: the iterations set now; $typical: what the depth typically needs.
app-feature-iter-note = ⚠「疊代次數」固定為 { $have }；這個深度通常需要約 { $typical }。如果畫面算繪成一片平坦，請調高它（或開啟自動疊代）。
# $feature: what was found (see above). $zoom and $asked: magnifications, already formatted; $octaves: the solver's depth limit.
app-feature-zoomed-capped = 已縮放至 { $feature }：{ $zoom }×。求解的上限為 { $octaves } 個八度，因此停在這裡，而不是 { $asked }×。
# $feature: what was found. $zoom: the magnification, already formatted. $seconds: how long the search took (1 decimal).
app-feature-zoomed = 已縮放至 { $feature }：{ $zoom }×（{ $seconds } 秒）
# $feature: what was found. $seconds: how long the search took (1 decimal).
app-feature-snapped = 已吸附至 { $feature } 的中心（{ $seconds } 秒）
app-feature-no-minibrot = 在畫面附近找不到迷你 Mandelbrot 的中心；請再靠近一點。
# $feature: the point asked for, e.g. Misiurewicz (3,1). $widths: how many view-widths away, already formatted. $octaves: how far to zoom out.
app-feature-just-outside = 在此畫面外不遠處找到一個 { $feature } 點，距離 { $widths } 個畫面寬度。縮小約 { $octaves } 個八度即可讓它出現在螢幕上，或設定較大的前週期，以尋找這個尺度的特徵。
# $feature: the point asked for. $widths: how many view-widths away, already formatted (may be 2^187). $octaves: how far to zoom out.
app-feature-far = 找到一個 { $feature } 點，但它距離 { $widths } 個畫面寬度：該特徵的尺度比此畫面「大」得多。請「縮小」約 { $octaves } 個八度才能到達，或嘗試較大的前週期，以尋找這個尺度的特徵。
# $feature: the point asked for. $residual: a small number in scientific notation.
app-feature-not-preperiodic = { $feature } 與此處的軌道不符（殘差 { $residual }）；請清空兩個欄位，從畫面偵測這一對數值。
# $feature: the point asked for. "k/p" are the dialog's two fields.
app-feature-not-converged = 畫面附近沒有收斂的 { $feature } 點；請瀏覽到更近的地方，或嘗試不同的 k/p。
app-feature-bad-request = 前週期與週期都必須是正數。

## Share location dialog and location files

app-location-empty = 沒有可載入的內容（或文字太大）。
app-location-not-fractadyne = 不是 Fractadyne 位置。
# $zoom: the magnification, already formatted. $note: what in the text was ignored or repaired (in English).
app-share-loaded-note = 已載入 @ { $zoom }×：{ $note }。
# $error: the system's reason (often in English).
app-share-save-failed = 儲存失敗：{ $error }
app-share-saved = 已儲存。
app-share-saved-rendering-thumb = 已儲存。正在算繪縮圖…
app-share-saved-no-thumb = 已儲存，但沒有縮圖（此畫面無法算繪縮圖）。
app-share-saved-with-thumb = 已儲存，含縮圖。
# $error: the system's reason (often in English).
app-share-saved-thumb-failed = 已儲存，但無法加入縮圖：{ $error }
app-share-loaded-into-box = 已載入方塊中：請檢閱後按「套用」。
# Reading a file failed (a location or a palette). $error: the system's reason (often in English).
app-read-failed = 讀取失敗：{ $error }
app-share-too-large = 檔案太大（不是 .fdn 位置？）。

## File-dialog filter names (the extensions are added by the code)

app-filter-kfr = Kalles Fraktaler 位置
app-filter-fdn = Fractadyne 位置
app-filter-map = Fractint / KF 調色盤
app-filter-ugr = Ultra Fractal 漸層
app-filter-ase = Adobe 色票交換檔
app-filter-ggr = GIMP 漸層
app-filter-all = 所有檔案

## Importing and saving palette files (shown under the gradient editor)

# The reason a file was refused before reading it.
app-gradient-map-too-large = 太大了，不可能是 .map 調色盤
app-gradient-ugr-too-large = 太大了，不可能是 .ugr 調色盤檔
app-gradient-ase-too-large = 太大了，不可能是 .ase 色票清單
app-gradient-ggr-too-large = 太大了，不可能是 .ggr 漸層
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported-vga = 已從 { $file } 匯入 { $count } 個項目：這是 6 位元的 VGA 色表（白色為 252，而非 255），已依原樣保留，以便與 Fractint 本身的輸出一致。
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported = 已從 { $file } 匯入 { $count } 個項目，作為硬邊色帶。
# $error: the reader's reason (in English).
app-gradient-map-invalid = 不是 .map 調色盤：{ $error }
# $name: the gradient's name in the file; $file: the file's name.
app-gradient-ugr-imported = 已從 { $file } 匯入「{ $name }」。
# $file: the file's name; $count: how many gradients it holds.
app-gradient-ugr-pick = { $file } 包含 { $count } 個漸層，請在下方選擇一個。
# $error: the reader's reason (in English).
app-gradient-ugr-invalid = 不是 .ugr 調色盤檔：{ $error }
# $used: the swatches kept; $total: the swatches in the file.
app-gradient-ase-imported-some = 已匯入 { $total } 個色票中的 { $used } 個，平均分布。
# $count: the swatches in the file.
app-gradient-ase-imported = 已匯入 { $count } 個色票，平均分布並混合。
# $error: the reader's reason (in English).
app-gradient-ase-invalid = 不是 .ase 色票清單：{ $error }
app-gradient-nothing-to-save = 還沒有可儲存的自訂漸層。
# $file: the saved file's name.
app-gradient-ggr-saved = 已儲存「{ $file }」：中點、混合曲線與色彩空間全部完整保留。
# $file: the saved file's name; $count: the segments whose curve could not be kept.
app-gradient-ggr-saved-lossy = 已儲存「{ $file }」，但有 { $count } 個區段使用 .ggr 格式無法表達的 Bézier 曲線。相關區段已改以線性混合方式寫入；漸層庫中的副本仍保留原曲線。
# $path: the file; $error: the system's reason (often in English).
app-gradient-write-failed = 無法寫入 { $path }：{ $error }
# $name: the gradient's name; $count: its segments.
app-gradient-ggr-imported = 已匯入「{ $name }」：{ $count } 個區段，包含其中點、混合曲線與色彩空間。
# $error: the reader's reason (in English).
app-gradient-ggr-invalid = 不是 .ggr 漸層：{ $error }

## The eyedropper's banner across the top of the screen while picking a colour

# $hex: the colour under the pointer, as #rrggbb.
app-eyedropper-banner-color = 正在選取色彩  ·  { $hex }  ·  按一下以取用，按 Esc 取消
app-eyedropper-banner = 正在選取色彩  ·  按一下以取用，按 Esc 取消

## The gradient editor window

app-gradient-title = 漸層編輯器
# Before the two view buttons, Bar and Ring.
app-gradient-view = 檢視
app-gradient-view-bar = 長條
app-gradient-view-bar-hover = 一條直的色帶，最適合精確放置色標
app-gradient-view-ring = 圓環
app-gradient-view-ring-hover = 用圓環呈現，因為調色盤是「循環」使用的：頂端的接合處，就是算繪器每次掃過時都會跨越的接縫；在長條上看起來沒問題的漸層，在那裡可能會出現明顯的邊緣。
app-gradient-add-lane-hover = 在這條線上的任何位置按一下，即可在該處新增色標。+ 會在最寬的間隙中新增一個。
app-gradient-strip-hover = 拖曳標記可移動其色標 · 拖曳色帶本身可「旋轉」整個漸層，並在接縫處繞回 · 按兩下可新增色標，在標記上按右鍵可移除 · 方向鍵可微調所選項目（按住 Shift 可更精細）。
app-gradient-remove-selected-hover = 移除所選的色標
app-gradient-ring-hover = 拖曳圓環本身可「旋轉」整個漸層：越過接縫的部分會從另一側繞回來 · 拖曳標記可移動其色標 · 按一下外圈軌道可在該處新增色標，或按中間的 ⊕ 在最寬的間隙中新增一個 · ⊖ 會移除所選的色標。頂端的空心標記就是接縫，也就是循環調色盤繞回的地方；它固定不動，因為它「就是」接合處，色彩在它下方轉動。
app-gradient-empty = 還沒有自訂漸層：從「複製預設集…」、匯入檔案或下方已儲存的漸層開始，它的色標與各區段的曲線就會出現在這裡。
# $count: the entries of an imported palette, more than the editor shows stop by stop.
app-gradient-too-many = { $count } 個匯入的項目：太多了，無法逐一編輯色標。請使用上方的預覽，或用「複製預設集」重新開始。
# The selected segment's heading. $index: its number; $total: how many there are; $left and $right: where it starts and ends, 0.000–1.000.
app-gradient-segment-heading = 區段 { $index }/{ $total }  ·  { $left } → { $right }
# A label before the list of blend curves.
app-gradient-curve = 曲線
app-gradient-curve-hover = 色彩在此區段中變化的速度。「彎曲」的形狀由中點決定：中點為 0.50 時恰好是線性。
# The blend curves (GIMP's kinds), in a narrow list: keep them short.
app-gradient-blend-linear = 線性
app-gradient-blend-curved = 彎曲
app-gradient-blend-sine = 正弦
app-gradient-blend-sphere-increasing = 球面 ↑
app-gradient-blend-sphere-decreasing = 球面 ↓
app-gradient-blend-bezier = Bézier
# Hover on Bézier in the list of blend curves.
app-gradient-bezier-exact-hover = 兩個可以朝兩個方向拖曳的控制點。這條曲線本來就是三次曲線，因此轉換不會改變任何東西。
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-visible-hover = 兩個可以朝兩個方向拖曳的控制點。轉換會「近似」目前的曲線：這一條的誤差在 { $error } 以內，明顯看得出來。
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-barely-hover = 兩個可以朝兩個方向拖曳的控制點。轉換會「近似」目前的曲線：這一條的誤差在 { $error } 以內，幾乎看不出來。
# A label before the list of colour spaces (RGB, HSV).
app-gradient-space = 色彩空間
# A label before the Bézier presets beside it.
app-gradient-ease = 緩動
app-gradient-ease-hover = 預設集會寫入兩個控制點；若要其他效果，請在曲線上拖曳它們。Bézier 沒有中點：由它的控制柄決定在何處到達一半。
# Bézier presets, as in CSS easing: short buttons.
app-gradient-ease-linear = 線性
app-gradient-ease-in = 漸入
app-gradient-ease-out = 漸出
app-gradient-ease-in-out = 漸入漸出
app-gradient-midpoint = 中點
app-gradient-midpoint-hover = 混合到達一半的位置，以此區段的比例表示。0.50 為置中。
# A button that puts the midpoint back in the middle of the segment.
app-gradient-centre = 置中
app-gradient-centre-hover = 回到 0.50
app-gradient-no-hue = ⚠ 此區段的一端沒有色相
app-gradient-no-hue-hover = 有一端是黑色、白色或灰色，因此掃描會從紅色開始，繞完整個色環。這是合法的，有時也正是想要的效果；但這就是中間可能出現兩端都沒有的色彩的原因。曲線下方的色帶會顯示它的效果。
app-gradient-segment-start-hover = 此區段「起點」的色彩
app-gradient-segment-end-hover = 此區段「終點」的色彩
app-gradient-bytes-hover = 紅、綠、藍，以 0–255 表示
app-gradient-floats-hover = 同一色彩以 0–1 表示，這是算繪器儲存的形式
app-gradient-canvas-bezier-hover = 所選區段的混合曲線。拖曳任一控制點（x「和」y 方向皆可）即可改變形狀。淡色的線代表 0 與 1；曲線可以超出它們，被限制的是色彩而不是曲線。
app-gradient-canvas-hover = 所選區段的混合曲線：色彩從左邊色標變化到右邊色標的速度。拖曳圓環可移動中點，或將曲線切換為 Bézier，以取得可朝兩個方向移動的控制點。
# The selected stop's heading. $index: its number; $total: how many there are.
app-gradient-stop-heading = 色標 { $index }/{ $total }
app-gradient-stop-color-hover = 此色標的色彩：按一下色票即可選取，或在旁邊輸入十六進位值
app-gradient-stop-bytes-hover = 紅、綠、藍，以 0–255 表示：這個色彩寫入時的位元組
# The eyedropper button.
app-gradient-pick = 取色
app-gradient-pick-hover = 滴管：從螢幕上任何地方取用色彩。將指標移到你要的色彩上並按一下；按 Esc 取消。
# Written before the stop's position (0.000–1.000) in its number field: a short abbreviation of "position".
app-gradient-stop-pos = 位置
app-gradient-nudge-hover = 微調此色標（按住 Shift 可更精細）
app-gradient-remove = 移除
app-gradient-remove-hover = 移除此色標（或按它下方的 ⊖）
app-gradient-add-stop = 新增色標
app-gradient-add-stop-hover = 分割最寬的區段（或按一下漸層上方的線）
# A menu of the built-in palettes, to start from a copy of one.
app-gradient-copy-preset = 複製預設集…
app-gradient-paste = 貼上…
app-gradient-paste-hover = 從十六進位色彩或 0–255 的 RGB 三元組匯入調色盤
app-gradient-save-ggr = 儲存 .ggr…
app-gradient-save-ggr-hover = 將此漸層寫入 GIMP .ggr 檔案：GIMP、Krita 與 Inkscape 都能讀取，這個編輯器也可以
# A menu of the palette file formats (.map, .ugr, .ggr, .ase).
app-gradient-import = 匯入
app-gradient-import-map-hover = 載入 Fractint / Kalles Fraktaler 的 .map 調色盤檔（R G B 行，每個項目一行）
app-gradient-import-ugr-hover = 載入 Ultra Fractal 的 .ugr 漸層檔（一個 .ugr 通常包含許多漸層，由你挑選一個）
app-gradient-import-ggr-hover = 載入 GIMP 的 .ggr 漸層，完整保留其中點、混合曲線與色彩空間
app-gradient-import-ase-hover = 載入 Adobe 色票清單（.ase）。色票沒有位置資訊，因此會平均分布並混合。
# The gradient's name, for the library of saved gradients.
app-gradient-name = 名稱
# Shown greyed in the empty name field.
app-gradient-name-hint = 我的漸層
# The save button, when a gradient of that name is already saved.
app-gradient-update = 更新
app-gradient-save = 儲存
app-gradient-save-hover = 將此漸層儲存到你的漸層庫，完整保留其曲線與色彩空間
app-gradient-save-needs-name-hover = 請先替漸層命名
# A menu of the saved gradients. $count: how many there are.
app-gradient-saved-menu = 已儲存（{ $count }）
app-gradient-nothing-saved = 尚未儲存任何漸層
app-gradient-delete-saved-hover = 刪除這個已儲存的漸層
# $count: the gradient's segments.
app-gradient-rich = 豐富漸層：{ $count } 個區段帶有中點、混合曲線或色彩空間，超出單純色標清單所能容納的範圍。
app-gradient-convert = 轉換為可編輯的色標
app-gradient-convert-hover = 以單純的色標清單取代它。這會「捨棄」色標清單無法容納的中點、混合曲線與色相掃描。
# Above the list of gradients in an Ultra Fractal file. $count: how many; $file: the file's name.
app-gradient-ugr-list = { $file } 中的 { $count } 個漸層
app-gradient-close-list = 關閉清單
app-gradient-seamless = 無縫（終點與起點一致）
app-gradient-seamless-hover = 調色盤是循環使用的，因此位置 1 與位置 0 在螢幕上是相鄰的。開啟此選項後，最後一個色標的色彩會保持與第一個相同；圓環檢視會在頂端顯示接合處。
app-gradient-ends-differ = ⚠ 兩端不同
app-gradient-ends-differ-hover = 漸層終點的色彩與起點不同，因此循環的調色盤在繞回處會出現明顯的邊緣。
app-gradient-hard-bands = 硬邊色帶（項目之間不混合）
app-gradient-hard-bands-hover = Fractint / .map 的語意：每個項目都是單一的平塗色彩。關閉後會將它們混合成平滑的漸層。
app-gradient-paste-intro = 貼上十六進位色彩（#ff8800）或 0–255 三元組（255 136 0），以逗號、空格或換行分隔：
# Applies the pasted colours.
app-gradient-apply = 套用
# Empties the paste box.
app-gradient-clear = 清除
# $used: the colours kept; $total: the colours pasted; $limit: the most that are kept.
app-gradient-paste-imported-some = 已匯入 { $total } 個色彩中的 { $used } 個，從你的清單中平均取樣（上限為 { $limit }）。
# $count: the colours pasted.
app-gradient-paste-imported = 已匯入 { $count } 個色彩。
# $error: why the pasted text could not be read (in English).
app-gradient-paste-unreadable = 無法讀取：{ $error }
# A hint at the bottom of the editor. $count: the palette's entries.
app-gradient-bands-hint = { $count } 個色帶 · 依檔案順序取用；開啟色帶時會忽略位置。
# A hint at the bottom of the editor, in the ring view. $count: the gradient's stops.
app-gradient-ring-hint = { $count } 個色標 · 按一下外圈軌道可在該處新增 · ⊕ 在最寬的間隙中新增，⊖ 移除所選的色標 · 拖曳標記可移動它 · 頂端的標記是接縫
# A hint at the bottom of the editor, in the bar view. $count: the gradient's stops.
app-gradient-bar-hint = { $count } 個色標 · 按一下漸層上方的線可在該處新增 · 拖曳標記可移動它 · ⊖ 或按右鍵可移除
app-gradient-cancel = 取消
app-gradient-cancel-hover = 捨棄開啟此視窗以來所做的所有變更，並關閉視窗
app-gradient-ok = 確定
app-gradient-ok-hover = 保留這些變更並關閉編輯器

## Shared

# The button that closes the Help window and the "Faster deep zoom" window.
app-close = 關閉

## The "Faster deep zoom" window (Help menu): the optional accelerated build

app-accel-title = 更快的深度縮放
app-accel-active-heading = 你正在執行加速版
app-accel-active-body = 深度縮放的參考軌道正以 MPFR/GMP 計算，在這個步驟上比標準版快 2.5-6.4 倍；這個步驟就是深層畫面開始解析前的那段停頓。
# $status: which arithmetic library is in use and its version (in English).
app-accel-in-use = 使用中的算術運算：{ $status }
app-accel-fallback-heading = 加速版：改用內建的算術運算
app-accel-fallback-body = 這是加速版（MPFR），但找不到它的數學函式庫，因此改用內建的算術運算（astro-float）。影像完全相同；只是深層畫面解析前的停頓會比較久。
# Followed by the folder the program runs from, on its own line.
app-accel-fallback-dlls = 若要使用較快的路徑，請將 libmpfr-6.dll 與 libgmp-10.dll（以及 libgcc_s_seh-1.dll、libwinpthread-1.dll）放在 fractadyne.exe 旁邊，然後重新啟動：
app-accel-warning-hidden = 關於這件事的啟動警告已隱藏。
app-accel-show-again = 再次顯示
app-accel-offer-heading = 有一個選用版本在深處快 2.5-6.4 倍
app-accel-offer-body = 深度縮放的大部分時間都花在以 CPU 計算參考軌道上，也就是深層畫面開始解析前的那段停頓。有一個選用版本改用 MPFR/GMP 函式庫來做這件事，速度快 2.5 到 6.4 倍，而且越深越明顯。其他一切都相同。
app-accel-offer-identical = 影像「逐位元組完全相同」：這是由較快的函式庫計算的同一套數學，已在每一種碎形公式及整套深度縮放比對測試集上驗證過。
app-accel-offer-shared = 你的設定、已儲存的工作階段與位置在兩個版本之間共用，因此可以自由切換，不需要匯入任何東西。
# Shown on Windows.
app-accel-separate-windows = 它需要另外下載，因為它使用的函式庫無法以標準 Windows 版所用的編譯器建置，而且這些函式庫採用與 Fractadyne 本身不同的授權（GNU LGPL v3）。
# Shown on other systems.
app-accel-separate = 它需要另外下載，因為它使用的函式庫採用與 Fractadyne 本身不同的授權（GNU LGPL v3）。
app-accel-download = 下載對應此版本的加速版
app-accel-all-releases = 所有發行版本
app-accel-all-releases-hover = 如果直接連結出現 404，表示此版本還沒有加速版；請在這裡挑選最接近的版本。
# Shown on Windows.
app-accel-install-windows = 解壓縮後，從該資料夾執行 fractadyne.exe，並讓 .dll 檔案留在它旁邊。
# Shown on Linux.
app-accel-install-linux = 解壓縮後，從該資料夾執行 ./fractadyne。它使用系統的 GMP 與 MPFR（libgmp10 / libmpfr6），幾乎每個桌面系統都已安裝；請參閱套件中的 README-ACCELERATED.txt。
# Shown on other systems.
app-accel-install-other = 目前還沒有為這個平台建置加速套件；可以使用 --features fractadyne-core/rug 從原始碼建置。

## The Help window's frame (its sections are under help-)

app-help-title = Fractadyne 說明

## Checking for updates

app-toast-up-to-date = 你使用的是最新版本。
# $error: why the check failed (in English).
app-toast-update-failed = 檢查更新失敗：{ $error }
# The window that offers a newer version.
app-update-title = 有可用的更新
# $version: the new version (0.3.0-beta.20). $channel: the token "stable" or "beta", in English; a translation can choose its words by it ({ $channel -> [beta] … *[other] … }).
app-update-version =
    { $channel ->
        [beta] Fractadyne { $version }（Beta 版）已推出
       *[other] Fractadyne { $version }（穩定版）已推出
    }
# $current: the version running now. $track: the update track the user follows (its name in the language in use).
app-update-running = 你正在執行 { $current }  ·  { $track }
app-update-download-intro = 從 GitHub 下載最新版本：
app-update-download = 從 GitHub 下載
app-update-later = 稍後提醒我

## The second graphics card (shown in its window, after "In use:" or "Could not start:")

# The second device is the same card as the window's. $name: the card's name.
app-worker-same-card = 同一張顯示卡，{ $name }
# Why the second device could not start.
app-worker-thread-stopped = 裝置執行緒已停止

## Export status (shown in the Export image window)

app-export-gpu-unavailable = 無法使用 GPU
app-export-reference-aborted = 匯出失敗：參考軌道建構已中止。

## The performance panel (View ▸ Performance panel)
## Its row labels sit in a monospace column about 14 characters wide: keep them short (a longer label pushes its value to the right).

app-perf-fps = FPS
app-perf-fps-hover = 每秒呈現的影格數，依影格之間經平滑處理的間隔計算。畫面正在穩定下來時，這裡改為顯示細化進度：正在穩定的畫面，並不是以這裡的數字所暗示的速率在執行。
# What the FPS row shows while a view is settling. $done and $total: tiles; $eta: the time left, as m:ss.
app-perf-refining-tiles = 細化中 { $done }/{ $total }（約 { $eta }）
app-perf-refining-arming = 細化中（準備中）
# $percent: how much is done, with one decimal.
app-perf-refining-percent = 細化中 { $percent }%
app-perf-building-reference = 建構參考軌道
app-perf-idle = 閒置
app-perf-frame = 影格
app-perf-frame-hover = 影格之間經平滑處理的實際經過時間，並不是下方各列的總和。它包含等待呈現的時間，因此在已穩定的閒置畫面上，它反映的是重繪節奏，而不是任何工作量。
app-perf-cpu = cpu
app-perf-cpu-hover = 一次更新中主執行緒所花的時間：參數設定與參考軌道快取管理。參考軌道重建時，這一列會出現尖峰，而這裡的尖峰就是你感受到的卡頓。
app-perf-gpu-idle = gpu/閒置
app-perf-gpu-idle-hover = 影格時間減去主執行緒的 CPU 時間。畫面算繪時，這是 GPU 的工作；但在已穩定的畫面上，它大多是閒置等待。名稱同時寫出兩者，因為面板僅靠這個減法無法區分它們。
app-perf-timing = 計時
# How frame cost is measured, the timing row's value. TIMESTAMP_QUERY is a name.
app-perf-timing-starved = 實際時間（時間戳記不足）
app-perf-timing-gpu = GPU 時間戳記
app-perf-timing-no-query = 實際時間（無 TIMESTAMP_QUERY）
app-perf-timing-hover = 影格成本的測量方式。GPU 時間戳記是精確的來源；計時資料停止傳回時，會改以實際時間作為備援，而以實際時間定價的預算曾多次估錯深層影格的大小：此處顯示紅色時，請理解為「目前的安全邊際只是估計值」。
app-perf-mode = 模式
app-perf-mode-hover = 上一個影格算繪時使用的算術路徑：direct（直接計算，無參考軌道）、df32 微擾，或 floatexp，也就是大約超過 1e28x 時使用的擴充範圍路徑，也是唯一會執行 BLA 的路徑。
app-perf-eff-iter = 有效疊代
app-perf-eff-iter-hover = 上一個影格實際使用的疊代次數，已計入深度調整與移動時套用的上限；而不是「品質」中的基準設定，那只是起點。
app-perf-precision = 精度
app-perf-precision-hover = 計算參考軌道所用的尾數位元數。它會隨深度增加；64 位元是淺層的下限，不是你碰到的上限。
app-perf-orbit-len = 軌道長度
# $cap: the most orbit samples this GPU holds, already formatted, or the text of app-perf-uncapped.
app-perf-orbit-len-hover = 參考軌道的樣本數。這張 GPU 最多可容納 { $cap } 個；接近這個數字就是實際的深度極限：超過之後，即時畫面就無法解析需要更長軌道的更深位置。
app-perf-uncapped = （無上限）
app-perf-sa-skip = SA 略過
app-perf-sa-skip-hover = 逐像素計算開始之前，級數近似所略過的疊代次數。數值越高越快，而且完全不會改變影像；沒有略過任何疊代時，這一列不會出現。
app-perf-aa = aa
app-perf-aa-hover = 超取樣：每個像素在每個軸向上以這麼多個樣本算繪，因此 2x 需要四倍的工作量。
app-perf-dual = 雙畫面
app-perf-dual-hover = 分割的 Mandelbrot/Julia 畫面是否正在執行。為 true 時，此面板上的每一項成本都涵蓋「兩個」窗格。
app-perf-zoom = 縮放
app-perf-zoom-hover = 相對於完整畫面的放大倍率。上方的算術模式大約在 1e4x 時改變，大約在 1e28x 時再改變一次。
app-perf-iterate = 疊代
# $target and $lethal: times in milliseconds, already formatted.
app-perf-iterate-hover = 上一次疊代派送實測的 GPU 時間。控制器的目標是 { $target } ms；持續接近 { $lethal } ms 的讀數，就是這類硬體曾經遺失裝置的區間。
app-perf-steps = 步/秒
app-perf-steps-hover = GPU 每秒執行的疊代步數，以十億為單位。以內部為主的畫面，每一步的速度比單看數字所暗示的還慢，因此這是吞吐量讀數，而不是分數。
app-perf-budget = 預算
# After the budget's value while it is still converging; the hover below quotes it.
app-perf-budget-settling = （收斂中）
app-perf-budget-hover = 每次派送的自適應步數預算，是控制器用來防止影格長到讓驅動程式重設 GPU 的保護措施。「收斂中」表示它仍在從稀疏的讀數收斂，還不是可信賴的上限。
app-perf-rss = rss
# $mb and $peak: memory in megabytes, already formatted.
app-perf-rss-value = { $mb } MB（峰值 { $peak }）
app-perf-rss-hover = 處理程序的常駐記憶體，以及本工作階段達到的最高值。在實際使用中，深層參考軌道的建構曾經超過 2 GB 的峰值，而螢幕上沒有任何其他地方顯示這一點；超過系統 RAM 的四分之三時會顯示為琥珀色。
app-perf-gpu-est = gpu 估計
app-perf-gpu-est-hover = 估計的 GPU 常駐記憶體：參考軌道、BLA 樹、疊代紋理、細化進行時的區塊狀態乒乓緩衝，以及呈現閘門的快照。由已知的配置大小加總而得；驅動程式的實際數字無法以可攜的方式讀取。
app-perf-julia-re = julia c.re
app-perf-julia-re-hover = Julia 窗格正在算繪的參數 c 的實部：除非已釘選，否則就是游標所在的點。
app-perf-julia-im = julia c.im
app-perf-julia-im-hover = Julia 窗格正在算繪的參數 c 的虛部。
app-perf-julia-c = julia c
# The julia c row's value when the parameter is held fixed.
app-perf-pinned = 已釘選
app-perf-pinned-hover = 參數固定在某一點上，因此移動游標不再改變 Julia 窗格。
app-perf-c-panel = c/面板
app-perf-c-panel-hover = 整個 Mandelbrot 面板所涵蓋的 c 空間寬度。當它遠小於一個 Julia 像素時，停留游標會改變 c，但 Julia 看起來沒有變化：這在深度縮放時是預期的情況，並不是凍結。
app-perf-ref-recompute = 參考重算
app-perf-ref-recompute-hover = 最近一次參考軌道重新計算的時間長度。這就是深層畫面開始解析前的停頓，也是選用加速版所加快的步驟。
app-perf-recompute-rate = 重算/秒
app-perf-recompute-rate-hover = 每秒「安裝」的參考軌道數：結果確實被使用的那些。請與下一列比較：兩者的差距就是被丟棄的工作。
app-perf-builds-rate = 參考建構/秒
app-perf-builds-rate-hover = 每秒「產生」的參考軌道建構數，包括結果被捨棄的那些：下潛永遠到不了的預先計算位置，或被拒絕的延伸。這些都會耗用完整的 CPU，而且不會出現在其他任何地方：曾經有一次預先擷取空轉，在六分鐘內每秒執行約 400 次建構，而「重算/秒」的讀數為 2。個位數是正常的；這兩列之間差距很大，表示有工作被丟棄。
app-perf-recompute-total = 重算總計
app-perf-recompute-total-hover = 啟動以來的參考軌道重新計算次數。適合用來計算已知時間區間內的速率；畫面沒有移動時這個數字卻持續攀升，就是值得追查的訊號。

## Help: the contents list (Help & reference…, F1)

# Each section's name in the contents list (about 165 px wide: keep them short), and its heading.
help-section-overview = 概覽
help-section-navigation = 瀏覽操作
help-section-expressions = 座標運算式
help-section-options = 著色與選項
help-section-fractals = 碎形
help-section-how-it-works = 運作原理
help-section-command-line = 命令列
help-section-shortcuts = 快速鍵
help-section-hardware = 建議硬體
# Credits to the prior work Fractadyne builds on.
help-section-acknowledgments = 致謝
# The third-party license notices.
help-section-licenses = 授權
help-section-about = 關於

## Help ▸ Overview

help-overview-intro = Fractadyne 是一套原生的碎形探索程式，專為超深度縮放與速度而打造。它繪製的是「逃逸時間」碎形：在每個像素上反覆套用同一個簡單公式所產生的影像。
help-overview-what-is = 什麼是逃逸時間碎形？
help-overview-what-is-text = 對每個像素，程式會從零開始，一再執行像 z → z² + c 這樣的公式。如果運算中的值永遠保持很小，該像素就屬於集合，會畫成暗色。如果它最終無限制地增長（「逃逸」），該像素就在集合之外，而它的色彩記錄了這花了多少步。介於「留下」與「逃逸」之間、無限精細的邊界，就是碎形。
help-overview-can-do = 你可以做什麼
help-overview-can-pan = 幾乎無限制地平移與縮放（在任何深度下位置都是精確的）。
help-overview-can-families = 在 25 種內建碎形族（十種，其中五種另有更高次方的版本）、你自己的公式、生命遊戲與 L 系統之間切換，並可將 Newton 以外的任何逃逸時間族以 Julia 集合檢視。
help-overview-can-recolor = 使用預設或自訂的漸層，以及多種著色方法重新上色。
help-overview-can-lighting = 加上 3D 浮雕光影與發光的邊界等高線。
help-overview-can-minibrots = 吸附到迷你 Mandelbrot、將地點加入書籤，並匯出高解析度影像。
help-overview-can-autopilot = 讓自動縮放的自動駕駛自行朝細節下潛。
help-overview-can-share = 將任何地點分享為小小的「.fdn」文字片段或檔案。
help-overview-can-tours = 執行腳本導覽與硬體基準測試。
help-overview-first-steps = 第一步
# "Seahorse Valley" is a place in the Navigate menu: use its name as that menu shows it. A and F1 are keys.
help-overview-first-steps-text = 開啟「瀏覽」選單並選擇「Seahorse Valley」，然後轉動滑鼠滾輪放大。拖曳可平移。或者直接按 A，讓自動駕駛替你下潛。隨時按 F1 都能回到這份說明。

## Help ▸ Coordinate expressions
## The expressions, numbers and function names here are typed as shown in every language:
## keep them exactly as written. Menu items in quotation marks are the menus' own labels.

help-expr-intro = 只要 Fractadyne 需要座標，你都可以輸入運算式來代替小數：「瀏覽 → 前往位置…」的 Re / Im 欄位（以及其極座標模式的每個欄位）、.fdn 位置中的 center_re / center_im 行、導覽腳本的 re / im，以及命令列上的 --center。運算式會以縮放所需的位數來計算，而「前往位置」對話方塊會將它與畫面一起保存；因此，以 1/3 或 -0.5 + 0.25*cos(pi/4) 輸入的點，不論放大多深都會精確地停在該點上，而小數一旦位數用盡就會漂移。
help-expr-examples = 範例
help-expr-ex-third = 精確的三分之一。放大時會出現更多位數。
help-expr-ex-neck = Seahorse Valley 的頸部：週期 2 圓盤的根點。
help-expr-ex-circle = 以 -0.5 為中心、半徑 1/4 的圓上的一點（極座標模式會替你組出這個運算式）。
help-expr-ex-bulb = 週期 3 圓盤根點的 Re；它的 Im 是 sin(2*pi/3)/2 - sin(4*pi/3)/4。「瀏覽 → 精確點（運算式）」已內建這些點。
help-expr-ex-complex = 在 Re 欄位中輸入完整的複數值，會同時填入 Re 與 Im。
help-expr-ex-combine = 函數與常數可以自由組合。
help-expr-ex-powers = 乘冪：2^-20 等於 1/1048576。-2^2 等於 -4：負號是在乘冪之後才套用。
help-expr-ex-root = -2，即實數立方根。負數的偶次方根會被拒絕；sqrt(-1) 得到 i。
help-expr-ex-scientific = 運算式中也可以使用科學記號。
help-expr-numbers = 數字與運算子
# The decimal point in an expression is '.' in every language: keep that rule as it is.
help-expr-numbers-decimals = 小數可以帶有選用的指數：0.25、-1.5e-30、.5。只能使用數字：數字中不能有空格或千分位逗號，而且小數點一律是「.」。
help-expr-numbers-imaginary = 加上 i 字尾表示虛數（0.25i），或單獨使用 i：3 + 4*i。
# Starts with the operators themselves, two spaces apart.
help-expr-numbers-operators = +  -  *  /  與 ^（乘冪；由右至左計算，因此 2^3^2 = 2^9）。括號用於分組。沒有隱含的乘法：請寫 2*pi，而不是 2pi。
help-expr-numbers-case = 名稱不區分大小寫：PI、Pi 與 pi 是同一個常數。
help-expr-constants = 常數
help-expr-functions = 函數
help-expr-functions-angles = 角度以弧度為單位。若要使用度數，請乘以 pi/180，例如 cos(45*pi/180)；或將「前往位置」對話方塊切換到極座標模式，並選擇度數單位。
help-expr-refused = 哪些會被拒絕，以及原因
help-expr-refused-fractional = 負數的分數次方，例如 (-2)^(1/2)：結果是複數，而且有多個根可以選。請使用 root(x, n) 或 cbrt(x) 取得實數根，或使用 sqrt(x) 取得主值複數根。
help-expr-refused-complex = 將複數引數傳給實值函數（sin(i)、ln(1+i)）。只有 sqrt 與 abs 接受複數引數；算術運算子與整數次方可用於複數值。
help-expr-refused-domain = 定義域錯誤：對非正數取 ln 或 log、asin 或 acos 的引數超出 -1…1、0 的負數次方、除以零。
help-expr-refused-bounds = sin、cos、tan、exp 或 ^ 的引數超過 ±2^32，以及巢狀深度超過 32 層：貼上的座標是不可信任的輸入，而這些情況會耗費無上限的時間。
help-expr-refused-grammar = 不支援變數、使用者自訂函數、比較或條件式。語法刻意保持精簡，讓儲存的位置在每個版本中都代表同樣的意思。
help-expr-no-parse = 無法剖析時
help-expr-no-parse-text = 訊息會指出是哪個欄位、找到了什麼、在第幾個字元，並針對常見的失誤說明應該改輸入什麼：從文件貼上的排版用減號（−）、× 或 ÷、小數逗號、數字與名稱之間少了 *、未閉合的括號，或函數沒有寫括號。絕不會只讀取一半：只要文字中有任何部分無法理解，整個座標就會被拒絕，而不會被悄悄截短。

## Help ▸ Navigation
## Menu paths (Navigate → "Go to location…") and quoted items name the interface's own
## menus and controls: use their translations. Key names (Ctrl, Shift, Space, Esc) stay.

# A heading: the mouse controls (also in Shortcuts).
help-mouse = 滑鼠
# The left column of a controls table: a gesture or key. Keep key names (Shift, Space, Ctrl) as they are.
help-key-left-drag = 左鍵拖曳
help-nav-left-drag = 平移畫面。
help-key-mouse-wheel = 滑鼠滾輪
help-nav-wheel = 朝游標位置放大／縮小；滾得越快，每一格縮放得越多。開啟「縮放吸附至特徵」時，在迷你 Mandelbrot 附近放大，會把縮放拉向它。
help-key-shift-drag = Shift+拖曳
help-nav-shift-drag = 框選縮放：拖曳出一個矩形，它就會縮放到填滿畫面。
# $icon: the click-to-zoom toolbar button's symbol.
help-key-click-zoom = 點按縮放（{ $icon }）
help-nav-click-zoom = 切換工具列中的目標按鈕（單一畫面）：之後按左鍵會以設定的倍數深入該點，按右鍵則退回。拖曳仍可平移；倍數（2–100×）可在右側面板的「瀏覽」中選擇。Backspace 可復原一次點按。
help-nav-continuous = 連續縮放與回到初始畫面
help-key-hold-space = 按住 Space
help-nav-hold-space = 以游標為錨點平滑放大。
help-key-hold-shift-space = 按住 Shift+Space
help-nav-hold-shift-space = 平滑縮小。
help-nav-continuous-text = 「縮小至初始畫面」按鈕會以動畫平緩地飛回完整畫面。右側面板中的「縮放速度」可設定連續縮放的速率。
help-nav-autopilot = 自動駕駛
help-nav-autopilot-text = 按 A（或「工具 → 自動縮放（自動駕駛）」）即可免動手下潛：它會在碎形邊緣上細節最密集的地方挑選一點，並偏好畫面中央，然後滑行進去，同時將該點緩緩移到螢幕中央。只要該目標仍然有趣，它就會持續瞄準；出現更豐富的結構時，它會平順地轉向。任何瀏覽操作（或 Esc）都會讓它停止。右側面板（或「工具」選單）中的「自動縮放優先」決定它以什麼為優先：「速度」以設定的速率縮放，含有細節的影格一準備好就更新螢幕；「品質」只顯示完全解析的全解析度影格，並視需要放慢縮放，讓畫面在下一個影格到來之前不會放大太多。無論哪一種，螢幕都不會顯示空白或單色的影格：尚未解析出任何細節的更新會隱藏在上一個影格後面。「自動縮放目標」決定它瞄準什麼：「細節」（畫面中最豐富的邊緣，隨下潛重新選擇）或「Misiurewicz 點」：它會找出最近的 Misiurewicz 點，在保持靜止的同時將它求解到下潛上限的深度，然後精確地下潛進去；這種點周圍的結構每隔幾個八度就會重複，永無止境，因此這種下潛永遠不會走進死路（僅限 Mandelbrot 集合）。
help-nav-history = 歷程與精確移動
help-nav-undo = 復原上一個畫面。
help-nav-redo = 重做。
help-nav-goto = 「瀏覽 → 前往位置…」可讓你讀取、輸入、貼上或複製精確的中心與縮放（全精度、任何深度）。中心欄位接受的不只是單純的數字：像「1/3」這樣的分數，以及由 pi、tau、e、sqrt 與 sin/cos/tan 組成的運算式，因此「-0.5 + 0.25*cos(pi/4)」是有效的座標。運算式會與畫面一起保存，並以每個深度所需的精度重新計算，因此以這種方式輸入的點，不論放大多深都會保持精確（四捨五入的小數最終做不到）。極座標模式（位移 + r ∠ θ，單位為度／弧度／圈）會替你組出這樣的運算式。完整的函數與常數清單及範例，請見本說明的「座標運算式」一節（對話方塊中的 ? 按鈕會開啟它）。「瀏覽 → 書籤」可以儲存及叫出位置。
help-nav-exact-points = 「瀏覽 → 精確點（運算式）」會跳到以精確方式定義、而非以四捨五入的小數定義的點：主心形區各個圓盤的根點，也就是每個週期 q 芽苞附著的地方，以 cos/sin 運算式表示。由於運算式會隨畫面一起保存，你可以放大其中任何一點，而永遠不會偏離該點。
help-nav-share = 「瀏覽 → 分享位置…」會擷取整個畫面（碎形、全精度中心、縮放與著色），存成精簡的「.fdn」片段，你可以「複製」、貼回後「套用」，或以檔案「儲存」／「載入」，以精確重現某個地點（或傳送給別人）。
help-nav-open = 「檔案 → 開啟畫面或位置…」可直接開啟以下任何一種：匯出的 PNG/EXR（會還原其中嵌入的畫面）、「.fdn」位置檔，或 Kalles Fraktaler 的「.kfr」。
help-nav-finding = 尋找細節
help-nav-minibrots = 迷你 Mandelbrot 是隱藏在邊界上、整個集合的微小複製品。將其中一個大致置中，然後按 M（或「工具 → 尋找迷你 Mandelbrot 並縮放至該處」），就能以牛頓法精確吸附到它的中心、讀出它的週期，並直接縮放到它本身的尺度，往往一步就跨越許多個數量級。如果你已經比該迷你 Mandelbrot 更深，就只會移動中心。
help-nav-minimap = 「檢視 → 小地圖總覽」會顯示整個集合的小地圖，附有「你在這裡」標記與目前的縮放深度；按一下地圖即可跳到該區域。

## Help ▸ Coloring & options
## Names in quotation marks are the controls' own labels in the right-hand panel.

help-options-intro = 這些設定全都在右側面板中（而且會在工作階段之間保留）。
help-options-palette = 調色盤
help-options-palette-text = 「調色盤」可選擇預設漸層、你的自訂漸層或雙色模式。「編輯漸層…」會開啟編輯器，其中每個色標都有色彩與位置（0–1）；你可以新增色標、複製一個預設集作為起點，或匯入調色盤檔。
help-options-paste = 漸層編輯器中的「貼上…」可匯入你在別處找到的調色盤。它接受開頭有或沒有 # 的十六進位色彩（#ff8800、ff8800，或簡寫的 #f80），以及單純的 0–255 RGB 三元組，例如「255 136 0」，也就是 Fractint 與 Kalles Fraktaler .map 檔案所用的格式，以逗號、空格或換行分隔。最多保留 256 個色彩；更長的清單會平均取樣成 256 個，並保留第一個與最後一個，讓漸層的形狀得以保留。
help-options-import-map = 旁邊的「匯入 ▸ .map…」可載入 Fractint 或 Kalles Fraktaler 的 .map 調色盤檔。這類檔案是以疊代次數為索引的查找表，項目之間不混合，而由此產生的生硬色階，正是經典 Fractint 風格的組成；因此它們就是以這種方式匯入，並附有「硬邊色帶」核取方塊，可改將同樣的色彩混合成平滑的漸層。Fractint 本身的檔案儲存的是乘以四的 6 位元色彩值，因此它們的白色是 252 而不是 255；Fractadyne 在匯入時會說明這一點，並完全依原樣保留數值，因為 Fractint 本身儲存的影像也帶有同樣的 252。
help-options-duotone = 「雙色調」將數值對應到平滑的雙色漸層（陰影 → 亮部）。「二元（集合）」是平面的雙色畫面：集合內的點用一種純色，集合外用另一種，沒有漸層，這是看清集合形狀最清楚的方式。
help-options-cycle = 「循環」設定漸層在疊代範圍內重複的次數（色帶更密或更疏）。「位移」會旋轉整個漸層。
help-options-normalize = 「正規化深層色彩」會讓調色盤貼合畫面中實際存在的逃逸值範圍。深層畫面橫跨數十萬次疊代，如果沒有這個選項，固定的循環會在相鄰像素之間讓漸層繞上數千圈，把正確的細節變成斑點雜訊。
help-options-log-scale = 「對數色階」依逃逸值的對數而非線性地分配調色盤。越深入，逃逸次數就越集中在高端，因此線性對應會把大部分漸層花在緊貼邊界的一層薄殼上，其餘部分則幾乎一片平坦；對數色階會把它展開，並讓縮放影片在範圍擴大時，色彩保持得更穩定。
help-options-animate = 「動畫」讓色彩隨時間循環：「正向」、「反向」或「來回」會移動位移量；「隨機漸層」則持續合成平滑變形、色彩和諧的漸層。「速度」控制每秒的循環（或變形）次數；「重新產生漸層」在隨機模式下會換上一個新的漸層。
help-options-method = 著色方法：資料如何變成色彩
# The coloring methods, by the names the Coloring panel gives them, each with what it does.
help-method-smooth = 平滑疊代
help-method-smooth-text = 依逃逸時間形成的經典連續色帶。
help-method-stripe = 條紋平均
help-method-stripe-text = 由軌道角度形成的流動色帶（「條紋密度」滑桿）。
help-method-triangle = 三角不等式
help-method-triangle-text = 由每一步落在上下界之間的位置所形成的細緻紋理。
help-method-trap = 軌道陷阱
help-method-trap-text = 軌道到某個形狀（點／十字／圓）的距離；也會為內部著色。
help-method-distance = 距離估計
help-method-distance-text = 依與邊界的接近程度著色。
help-method-decomposition = 分解
help-method-decomposition-text = 由最終逃逸角度形成的格子。
help-options-relief = 3D 浮雕光影
help-options-relief-text = 依邊界的斜率（導數）為表面著色，呈現浮雕、受光的效果。「光線角度」設定方向；「浮雕」設定強度（越低越銳利）；「旋轉光源」會讓它動起來。僅限全純族（Mandelbrot／Multibrot）。
help-options-glow = 距離光暈
help-options-glow-text = 明亮的等高色帶，在邊界附近變密，成為發光的細絲。「光暈」是混合量，「條帶寬度」是間距，「光暈動畫」會讓它們流動。
help-options-quality = 品質與疊代
help-options-quality-text = 「疊代次數」是像素被視為在集合內之前的最大步數；「疊代次數隨縮放自動調整」會在你縮放時自動提高它（更深的細節需要更多疊代）。「反鋸齒」會在畫面穩定下來後，對靜態影像進行超取樣（2×–8×），抑制外部細微的「塵點」。
help-options-other = 其他
help-options-other-text = 「縮放速度」設定連續縮放的速率；「影格速率上限」限制影格速率；「並列（連動）」會並排顯示 Mandelbrot 集合及其 Julia 集合（游標會即時設定 Julia 參數）。

## Help ▸ Fractals
## Fractal family names (Mandelbrot, Multibrot, Tricorn, Burning Ship, Celtic, Buffalo,
## Phoenix, Newton) are names: keep them as written, as the formulas. Formula syntax
## (z = z^3 - p1*z + c, sqr, |z|, if / elseif / endif, p1-p5, maxit) is typed as shown.

help-fractals-intro = 除非另有說明，每個族都從 z = 0 開始疊代公式，c 由像素決定（逃逸時間）。z = x + iy 是複數。
help-fractals-mandelbrot = z → z² + c。最原始的版本。集合是所有軌道保持有界的 c；它是連通的，而它的邊界皺褶到 Hausdorff 維度為 2。
help-fractals-multibrot = z → z^d + c，次方 d = 3、4、5。更高的次方會增加葉瓣：集合具有 (d−1) 重旋轉對稱（Multibrot 3 為 2 重，4 為 3 重，5 為 4 重）。
help-fractals-tricorn = z → conj(z)² + c，其中 conj(x + iy) = x − iy。共軛使它成為反全純的，帶來 3 重對稱與特有的彎曲「爪子」。
help-fractals-burning-ship = z → (|x| + i|y|)² + c：平方之前先取絕對值（實部 = x²−y²+cx，虛部 = 2|xy|+cy）。非解析；深度縮放會顯露出像船一樣的結構（傳統上是上下顛倒來看）。
help-fractals-celtic = 類似 Mandelbrot，但對實部取絕對值：實部 = |x²−y²| + cx，虛部 = 2xy + cy。會產生心形與盾形的圖案。
help-fractals-buffalo = 對 z² 的兩個部分都取絕對值：實部 = |x²−y²| + cx，虛部 = |2xy| + cy，介於 Celtic 與 Burning Ship 之間。
# A heading: the families at powers above 2 (the Fractal menu's "Higher powers").
help-fractals-powers = 高次方
# ("Burning Ship  3 4 5") is how the Fractal menu shows such a row: keep it as it is.
help-fractals-powers-text = Multibrot 6、7 與 8（z^d + c），以及次方為 3、4 與 5 的 Burning Ship、Tricorn、Celtic 與 Buffalo：在更高次方下的相同折疊。Burning Ship d 先對 z 的各部分取絕對值再取次方；Tricorn d 先取共軛；Celtic d 與 Buffalo d 則折疊 z^d 的實部（Buffalo 還包括虛部）。「碎形」選單與下拉式選單中，每個族列在一列（「Burning Ship  3 4 5」）。每一種都像其次方為 2 的同類一樣進行深度縮放，而 Multibrot 6、7 與 8 也和 Multibrot 3–5 一樣支援級數近似與迷你 Mandelbrot 尋找器（M）。Multibrot 8 的逃逸半徑是 128 而不是 256，因為在 256 時它的下一步會超出單精度範圍。
help-fractals-phoenix = z → z² + c + p·z(n-1)，其中 z(n-1) 是前一次疊代的值，p 是常數（p = -0.5）。這個記憶項會產生火焰般的細絲。
help-fractals-newton = 求 z³ − 1 = 0 之根的牛頓法：z → z − (z³−1)/(3z²)。像素不是依逃逸時間著色，而是依疊代收斂到三個單位立方根中的哪一個（吸引盆）以及收斂的速度著色；糾結的吸引盆邊界就是碎形。
help-fractals-julia = Julia 模式
help-fractals-julia-text = 除了 Newton 以外的每個族，都可以切換為 Julia 集合：不再是 z 從 0 開始、c = 像素，而是固定 c（一個參數），讓 z 從像素開始。Julia 集合就是在這個固定的 c 之下，保持有界與會逃逸的起點之間的邊界。在「並列（連動）」中，將游標移過 Mandelbrot 面板會即時設定 c。
help-fractals-deep = 深度縮放支援
help-fractals-deep-text = Mandelbrot、Multibrot 3/4/5 與 Tricorn 支援極深（floatexp）的微擾深度縮放。Burning Ship、Celtic 與 Buffalo 是非解析的（它們取絕對值），因此使用考慮正負號的微擾；這現在也能在 floatexp 範圍內執行，深度縮放遠遠超過舊有的約 1e28× df32 上限（絕對值折疊附近偶爾出現的斑點雜訊，有待多參考點瑕疵修正來處理）。Phoenix 使用雙項微擾，也能在 floatexp 範圍內深度縮放（沒有 SA/BLA，因此比 Mandelbrot 吃重）。高次方以相同方式深度縮放，每個次方的微擾都遵循自己的一連串平方運算；Multibrot 3–8 也和 Mandelbrot 一樣支援級數近似與 BLA，而各個次方的 Tricorn、Burning Ship、Celtic 與 Buffalo 支援 BLA（沒有級數近似），由 BLA 帶過深層畫面的內部，而不是逐步疊代。只有 Newton 使用直接路徑，清晰度可達約 1e6×；自訂公式會進行深度縮放，除非下方「自訂公式」一節另有說明。
help-fractals-custom = 自訂公式
help-fractals-custom-text = 「碎形 ▸ 自訂公式…」接受以 Fractint 風格運算式寫成的公式步驟：z = z^3 - p1*z + c，或多個陳述式（t = sqr(z)，然後 z = t*t + c）。z 是疊代值，c 是像素（Julia 模式下是 Julia 常數），p1-p5 是參數；|z| 是模的平方，與 Fractint 相同，並提供常用的函數（sqr、abs、conj、real、imag、cabs、flip、recip、ident、exp、log、sqrt、sin、cos、tan、cotan、sinh、cosh、tanh、cotanh、asin、acos、atan、asinh、acosh、atanh、cosxx，以及對各部分分別取整的 floor、ceil、trunc 與 round）。log 是自然對數，與 Fractint 及複分析中相同：也就是許多教科書寫成 ln 的函數（沒有以 10 為底的對數）。以 ln、cot、coth、Re 或 Im 寫成的公式無法讀取；語法檢查會指出應使用的函數，有一個按鈕可以改寫它，而輸入 ln 時會建議 log。每一步都以微擾進行深度縮放，超過 1e28x 時使用與內建碎形相同的擴充範圍算術；唯一的例外是含有指數隨 z 或 c 變化之乘冪的公式（例如 z^c）：這種公式以直接路徑算繪，清晰度維持到 c 的一個單精度步長跨越一個像素為止，大約 1e4x 到 1e5x，離原點越遠就越淺。公式對話方塊會說明適用哪一種。log、sqrt 與非整數次方取其主值，主值會在負實軸兩側跳躍。微擾會精確地跟隨這個跳躍，但軌道跨越之處與參考軌道不同的像素，從那之後只會以大約單精度繼續計算，因此軌道橫跨分支切割的深層畫面，會在那裡出現雜訊（實測：sqrt(z^4 + c) 在實軸上 1e5x 時仍然乾淨，而 log 公式即使在 1e4x 也有雜訊）。每一步都完整疊代（沒有級數近似），因此疊代次數高的深層畫面，算繪速度會比內建碎形慢。在邊界附近，含有函數的公式其軌道是混沌的，而在那裡，長軌道在單精度下只能被大致跟隨：在 2,000 次疊代時，sin(z) + c 有三分之一的像素與精確軌道的結果不同，在 60 次時則幾乎沒有。畫面、工作階段與匯出的影像都會保存公式的文字。
help-fractals-custom-sections = Fractint 的各個區段也能使用。「:」之前的陳述式是初始化區段，每個像素執行一次；最後一個陳述式若是比較，就是公式自己的逃逸測試，條件成立時持續疊代（|z| <= 100 && |z - 1| > 0.000001 會讓逃逸或停在 1 上的軌道停止）；if (...)、elseif (...)、else 與 endif 用來選擇陳述式；而在一步中先讀取後設定的變數，其值來自上一步（來自初始化區段，或為 0）。比較運算比較的是實部，結果為 1 或 0，&& 與 || 用來組合它們，而指派本身也是一個值（a = b = pixel）。pixel、p1-p5、pi 與 e 可以被指派，在被指派之前保有各自的值；maxit 是疊代次數，ismand 為 1（公式的 Mandelbrot 形式）。含有以上任何一項或含有取整的公式，會以直接路徑算繪，不支援深度縮放。「公式庫 > 匯入」也能讀取 Fractint 的 .frm 檔。每個可讀取的條目都會成為一個公式，其所在列會說明讀取時做了哪些變更：fn1-fn4 採用 Fractint 的預設值（sin、sqr、sinh、cosh）；Fractint 的變數 c 會更名為 c_，因為在這裡 c 是像素；沒有任何東西設定的名稱為 0，與 Fractint 相同；名稱中的空白會被捨棄（end if 即 endif），與 Fractint 的做法相同。匯入訊息會計算無法讀取的條目數，並列出最常見的原因：不支援螢幕與畫面變數（whitesq、scrnpix、scrnmax、center、magxmag、rotskew）、lastsqr 與亂數。「精選集」中的 Fractint 經典公式（Magnet、Nova、Spider、Manowar 等）都是以這些區段寫成的。
help-fractals-custom-editor = 在公式欄位中，括號會依巢狀層級顯示不同色彩；游標旁的那一對會被醒目提示，沒有配對的括號會以紅色顯示（註解中的括號不算，因為剖析器會忽略它們）。名稱會在你輸入時自動完成：輸入兩個字母後，會出現一份清單，列出以此開頭的函數、名稱與你自己的變數。上、下方向鍵用來選擇，Tab 或 Enter 選取一項（函數會連同左括號一起帶入），Esc 關閉清單，而完整輸入名稱後按 Enter 仍會開始新的一行。
help-fractals-custom-textbook = 「教科書」（公式上方的切換開關）會以教科書的排版方式顯示公式，並可直接在上面編輯。輸入方式與文字相同：/ 會將它前面的項變成分數，^ 產生指數（在指數末端輸入 +、- 或 = 會離開指數），( 產生括號，在函數名稱後則產生函數呼叫，| 產生直線符號，括號內的逗號產生複數常數。方向鍵可在公式中移動，上、下方向鍵在分子與分母之間移動，Tab 移到下一個空格；在分數或一對括號的開頭按 Backspace 會選取它，再按一次則移除它並保留其內容。複製、貼上與復原的運作方式與文字欄位相同，名稱的自動完成也和那裡一樣，而按鍵面板會輸入到排版後的公式中（□² 會將插入點前面的部分平方，函數鍵會包住所選內容）。每一次編輯都會寫回文字，因此兩種檢視永遠是同一個公式；你沒有編輯過的行會保留其間距與註解。記法絕不會改變計算的內容：|z| 是模的平方，顯示為 |z|²；cabs(z) 顯示為 |z|；exp(z) 仍是 exp(z)。無法讀取的行會以紅色顯示；按一下即可以文字編輯。「LaTeX」會將公式依排版結果複製為 LaTeX 原始碼，而開啟「教科書」時，公式庫也會以排版方式顯示其中的公式。
help-fractals-custom-numbers = 公式中的數字（0.1、1/3、pi、參數）會以雙精度值讀取，大約 16 位數，而深層參考軌道就是精確地以這些值疊代。因此，兩個在紙上相等的公式，在深處可能會有差異：0.1*z 乘以最接近 0.1 的雙精度數，而 z/10 除以精確的 10，兩張圖在大約 1e15x 之後就會分道揚鑣（實測：1e15x 時有 5% 的像素不同，1e17x 時三分之一，1e20x 時全部）。每張圖對它自己的公式來說都是正確的；請寫下你想分享的那一個。
help-fractals-custom-library = 對話方塊中的「儲存到公式庫」會以一個名稱保存公式及其參數（以公式庫中已有的名稱儲存，會更新該條目）。「碎形 ▸ 公式庫…」會列出它們，供你套用、編輯、匯出或刪除；「匯入」會加入公式檔中的公式，但不會取代任何現有公式：公式庫中已有的會略過，而名稱已被使用的新公式會變成「名稱 (2)」。公式庫就是「檔案與資料」（關於）中所列資料夾裡的 formulas.toml，它本身也是一個公式檔。導覽的關鍵影格以 formula = "..." 與 formula_params = [[re, im], ...] 攜帶公式，而「工具 > 從目前畫面建立導覽」會為自訂畫面寫入這些內容。
help-fractals-custom-collection = Fractadyne 隨附一套專為它撰寫的 45 個公式精選集：Burning Ship 與 Celtic 族的高次方與折疊變體、混合與兩步混成公式、透過其他映射看到的 Mandelbrot 集合（反演、對數極座標、透過正弦）、超越公式與有理公式，以及 Julia 集合。它們就是對話方塊中的「範例」，以及公式庫的「精選集」分頁，在那裡按「複製到我的公式」即可將其中一個加入你的公式。每個公式都有起始畫面，「套用」會前往該處：在初始畫面上開啟的公式，往往不在它的圖案所在之處。你在公式顯示時儲存它，也會以同樣方式保留你所在的畫面，「套用」會回到那裡。

## Help ▸ How it works

help-how-escape = 逃逸時間與平滑色彩
help-how-escape-text = 每個像素都會疊代公式，直到其大小超過逃逸半徑，或達到疊代上限為止。單用原始步數會產生生硬的色帶；加上一個由最終大小推導出的小數部分，就能得到連續、沒有色帶的色彩。
help-how-precision = 任意精度的位置
help-how-precision-text = 一般的 64 位元數字在大約 1e15× 的縮放附近就會用盡位數。Fractadyne 以任意精度保存畫面中心，位數隨著縮放增加，因此位置永遠不會劣化。像素尺度同樣以擴充指數保存，因此也不會停在 64 位元約 1e308× 的極限：深度只受耐心限制，而不受任何固定數值範圍限制。
help-how-perturbation = 微擾
help-how-perturbation-text = 以高精度疊代每個像素會慢得無法接受。因此改為在 CPU 上以高精度疊代一個參考像素（「參考軌道」），其他每個像素則在 GPU 上以快速的低精度，計算為與它之間的微小差值 δ：δz → 2·Z·δz + δz² + δc。
help-how-floatexp = 極深（floatexp）
help-how-floatexp-text = 大約超過 1e28× 後，就連那個微小差值也會在 32 位元範圍內下溢，因此改以尾數加上一個獨立的整數指數來儲存（「floatexp」），突破 32 位元的深度極限。引擎會自動切換：淺層時直接計算，深層時用微擾，最深時用 floatexp。如此一來，深度受限於座標精度與疊代預算，而不是固定的上限：已在一套 39 個位置的參考測試集上與 Fraktaler-3 交叉比對，最深達 5.63e18003×（38 個非極端的項目是例行的檢查關卡，深達 6.13e1105×），另有一組內部精度自我一致性測試，深達 1e1000000×；隨附的導覽可達約 1.3e420×。
help-how-reference = 參考點的選擇與重定基準
help-how-reference-text = 參考點經過挑選（以高精度評分），讓它的軌道盡可能長時間留在畫面內。當差值變得太大時，就會「重定基準」回到參考軌道上，以保持準確。
help-how-series = 級數近似
help-how-series-text = 在深度縮放時，早期的疊代幾乎不會讓 δ 偏離參考軌道，因此可以一次全部算出：δz 以 δc 的一個短多項式（3 階）來近似，其係數沿著參考軌道疊代。算繪器會直接跳到該多項式對整個畫面仍然準確的最後一次疊代，然後正常疊代：省下被跳過的步驟，而影像完全不變（可在「控制項 ▸ 進階」中切換）。適用於 Mandelbrot 與 Multibrot 3–8 的最深範圍；不適用於條紋／三角不等式／軌道陷阱／分解著色（這些需要每一次疊代）。在 BLA 啟用的地方，它已經涵蓋了這種早期跳躍。
help-how-distance = 距離估計與光影
help-how-distance-text = 追蹤導數 dz/dc，就能得出每個像素與邊界的距離。3D 浮雕光影、距離光暈與「距離估計」著色方法都以此為基礎，而且在任何縮放深度下都有效。
help-how-safety = 反鋸齒與安全機制
help-how-safety-text = 畫面穩定下來後，靜態影像會進行超取樣（每個軸向 2–8×）。在極端設定下，工作量預算會藉由降低解析度（絕不降低疊代次數），讓單一次 GPU 繪製保持在驅動程式看門狗的時限內，因此深層畫面能保持細節，而不會變成空白。

## Help ▸ Command line (the flags themselves stay English)

# The section's introduction. The flags listed below it are the command line itself, English in every language.
help-cli-intro = Fractadyne 可以在無介面模式下執行，用於算繪、影片／導覽匯出、自動化、黃金影像比對與基準測試。不加任何旗標時，它會啟動互動式圖形介面。

## Help ▸ Shortcuts

# What each mouse gesture and key does (the gestures are the Navigation section's).
help-shortcuts-pan = 平移
# A gesture: turning the mouse wheel.
help-key-wheel = 滾輪
help-shortcuts-wheel = 朝游標處縮放
help-shortcuts-box-zoom = 框選縮放（拖曳出矩形即縮放至該範圍）
# $icon: the click-to-zoom toolbar button's symbol. "tool on": while that button is on.
help-key-click-tool = 按一下（{ $icon } 工具開啟時）
help-shortcuts-click = 深入該點；按右鍵退回
# A heading: the keyboard shortcuts.
help-shortcuts-keyboard = 鍵盤
help-shortcuts-space = 連續放大／縮小
help-shortcuts-undo = 復原畫面
help-shortcuts-redo = 重做畫面
help-shortcuts-minibrot = 尋找迷你 Mandelbrot 並縮放至該處（Newton-Raphson）
help-shortcuts-autopilot = 自動縮放的自動駕駛（朝細節下潛；任何輸入都會讓它停止）
# off / strip / panel / full: the panel's sizes, as View ▸ Live render panel names them.
help-shortcuts-live-panel = 即時算繪面板：關閉／條狀／面板／完整（將游標停在儀表上可看它的意義；按它的 × 可關閉）
help-shortcuts-esc = 停止自動駕駛／正在播放的導覽，或離開全螢幕
help-shortcuts-snapshot = 將快照存到上次的匯出資料夾：可以是所見畫面的螢幕擷取，或依匯出設定的完整算繪；第一次按下時會詢問要哪一種，之後可在「檔案 → 設定 → 快照」中變更。
help-shortcuts-help = 開啟這份說明
# A heading: the shortcuts of the Game of Life, with the menu path to it.
help-shortcuts-life = 生命遊戲（碎形 → 自動機 → 生命遊戲）
help-shortcuts-life-run = 執行／暫停宇宙
help-shortcuts-life-step = 前進一個世代／前進一個步幅（在生命遊戲面板中設定）
help-shortcuts-life-draw = 開啟／關閉繪製工具（工具列上的鉛筆）
# A gesture. "Draw on": while the Life draw tool is on.
help-key-click-drag-draw = 按一下／拖曳（繪製開啟時）
help-shortcuts-life-paint = 翻轉一個細胞／塗繪：從死細胞開始的筆畫會繪製，從活細胞開始的筆畫會擦除。執行中的宇宙會在你繪製時暫停，放開後繼續執行。
# $key: a keyboard shortcut (Ctrl+Z). "Draw on": while the Life draw tool is on.
help-key-when-drawing = { $key }（繪製開啟時）
help-shortcuts-life-undo = 復原上一次的編輯、填滿、清除或規則變更

## Help ▸ Recommended hardware

help-hardware-intro = Fractadyne 將工作分配給 GPU（逐像素疊代與著色）和 CPU（深度縮放賴以建立的任意精度參考軌道）。兩者都很重要，但重要的地方不同。
help-hardware-gpu = GPU（最重要）
help-hardware-gpu-text = 每個像素都在 GPU 上疊代與著色，因此 GPU 的速度決定影格速率與匯出時間。具備強大 32 位元（單精度）運算吞吐量的獨立 GPU，能提供最流暢的深度縮放。內建 GPU 也能正常運作，只是比較慢，而且會較早達到每個影格的工作量預算（到那時算繪器會降低解析度，而不是細節）。需要現代的圖形 API：Vulkan、Direct3D 12、Metal，或透過 wgpu 使用 OpenGL。
# The left column of a requirements table.
help-hardware-minimum = 最低需求
help-hardware-gpu-minimum = 任何支援 Vulkan / DX12 / Metal / GL 3.3 的 GPU。
help-hardware-recommended = 建議配備
help-hardware-gpu-recommended = 獨立 GPU（約近 5 年內的產品）；>= 2 GB VRAM。
help-hardware-big-exports = 大型匯出
help-hardware-gpu-big-exports = >= 4 GB VRAM（分圖塊處理，因此不足時也能從容降級）。
# A heading: the processor.
help-hardware-cpu = CPU
help-hardware-cpu-text = 高精度參考軌道（以及 BLA 表）在 CPU 上計算，而且大多是單執行緒，因此單核心效能最為重要：它決定你移動後深層畫面解析的速度。額外的核心主要有助於背景匯出的編碼。需要 64 位元 CPU。
help-hardware-cpu-recommended = 單核心速度良好的現代 64 位元 CPU。
help-hardware-memory = 記憶體
help-hardware-memory-text = 深層參考軌道會隨深度增長（高精度下的大量疊代）；超大型匯出會將圖塊保存在 RAM 中。8 GB 足以應付日常探索；16 GB 以上適合極深的深度與非常大的匯出。
help-hardware-checking = 檢查你的硬體
help-hardware-diagnostics = 「說明 → 診斷…」會在你的電腦上執行與開發者相同的測試。「執行自我測試」會在大約十五秒內，將數學運算與算繪結果與已知正確的結果比對；「執行介面測試」會在數個縮放深度下走過介面與即時畫面，並擷取螢幕截圖；「執行 GPU 算術檢查」會在你的電腦提供的每一種圖形後端上，驗證著色器的雙浮點數算術，只需幾秒鐘，也不會開啟視窗。每項測試都在獨立的處理程序中執行，因此讓圖形驅動程式出問題的測試只會讓測試本身結束，而不會影響你的工作階段。
help-hardware-gpu-check = GPU 算術檢查沒有通過或失敗之分：它記錄的是你這張 GPU 與驅動程式的計算方式，而在某些硬體上（目前試過的每一張 NVIDIA 顯示卡），它會回報著色器編譯器捨棄了部分擴充精度的算術。這是我們正試著在各家廠商之間釐清的真實發現，而不是你電腦的故障，因此不論結果如何，將它附加到回報中都真的很有幫助。
help-hardware-report = 如果有東西看起來不對，請執行自我測試並使用「附加到問題回報…」：這樣回報就會帶有在你自己的硬體上經機器驗證的結果，遠比單純的描述有用。有一點值得注意：參考影像是在某一張特定的顯示卡上產生的，而不同的 GPU 本來就會算出略有不同的像素，因此在其他硬體上，影像比對會以較寬的容許誤差檢查，那裡出現的小差異是預期中的，並不是故障。
help-hardware-notes = 備註
help-hardware-note-watchdog = 效能足夠的 GPU 可避免在極端設定下觸發看門狗工作量預算。
help-hardware-note-binary = 程式是單一執行檔且自成一體，不需要安裝任何執行階段。
help-hardware-note-benchmark = 執行內建的基準測試（工具 ▸ 基準測試）來評估你的電腦。

## Help ▸ Acknowledgments & citations
## Names of people, programs, papers, books and websites stay as written, and so do the
## titles in quotation marks (the works' own titles); the prose around them translates.

help-ack-title = 致謝與引用
# The dedication, in italics at the top of the section.
help-ack-dedication = 謹獻給 Stone Soup Group：Fractint 背後的志工群（自 1988 年起）。
help-ack-dedication-text = 他們無償分享的程式碼，以及開放、協作探索碎形的精神，正是我們許多人愛上這門藝術的原因。Fractadyne 正是承襲這個傳統而打造。
help-ack-intro = Fractadyne 建立在碎形社群數十年的成果之上。它所採用的技術與先前的研究成果，已對照以下來源驗證：
help-ack-deep-zoom = 深度縮放演算法
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-martin-title = 微擾與級數近似：{ $who }
help-ack-martin = 參考軌道加低精度差值的方法（δz → 2Z·δz + δz² + δc），以及略過早期疊代的 n 階級數，首見於 SuperFractalThing 及其說明文件「SuperFractalThing Maths」（sft_maths.pdf，2013）。
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bla-title = 雙變數線性近似（BLA）與重定基準：{ $who }
help-ack-bla = 透過合併的線性映射略過一連串疊代，並重定基準到單一參考點以避免瑕疵；於 fractalforums.org 上發展（2021）。
# A link's label. $who: the linked page's author.
help-ack-bla-link = BLA 解說：{ $who }
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-glitch-title = 瑕疵偵測準則：{ $who }
help-ack-glitch = 判斷像素何時需要另一個參考點的 |Z + z| << |Z| 測試，是無瑕疵微擾的基礎（fractalforums）。
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-diffabs-title = 非解析（Burning Ship）微擾：{ $who }
help-ack-diffabs = 絕對值的「diffabs」分情況分析，讓微擾能用於以絕對值為基礎的族（Burning Ship / Celtic / Buffalo），發表於 fractalforums。
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mathr-title = 參考實作與交叉比對：{ $who }
help-ack-mathr = Fraktaler-3 與 Kalles Fraktaler 2+（最初由 Karl Runmo 開發）：它們的技術文章指引了這個引擎，而 Fractadyne 也以它們的 EXR 輸出交叉比對自己的算繪器。
# A heading: credits for the coloring methods.
help-ack-coloring = 著色
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-harkonen-title = 平滑疊代次數與條紋平均（Stripe Average）：{ $who }
help-ack-harkonen = 連續（正規化）的逃逸時間著色與條紋平均法，出自「On Smooth Fractal Coloring Techniques」（碩士論文，2007），該論文也分析了三角不等式／分支平均著色。
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mitchell-title = 三角不等式平均：{ $who }
help-ack-mitchell = 對每個 z² + c 落在三角不等式上下界之間的位置取平均，得到平滑、火焰般的外部紋理。
# A link's label: the person's tutorial pages. $who: the person's name.
help-ack-mitchell-link = { $who }：教學
help-ack-distance = 距離估計著色／光影使用經典的 dz/dc 邊界距離方法（參見 Peitgen & Saupe，「The Science of Fractal Images」，1988）。
help-ack-lsystems-life = L 系統與生命遊戲
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-lindenmayer-title = L 系統：{ $who }
help-ack-lindenmayer = 平行字串改寫，於 1968 年提出，用來模擬植物與藻類的生長方式（「Mathematical models for cellular interactions in development」，Journal of Theoretical Biology）。
help-ack-abop = L 系統讀取器遵循這本書（1990）的海龜詮釋，以及它對隨機、上下文相關與參數化系統的記法；標示為 ABOP 的內建系統是根據書中的圖輸入的。
# A link's label: the book can be read free on that website. $site: the website's name.
help-ack-abop-link = 這本書，可從 { $site } 免費取得
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bourke-title = L 系統集：{ $who }
help-ack-bourke = 許多內建系統來自他的 L 系統網頁，其中包括 Chris Wallace 的 Sierpinski 曲線規則與 Dietmar Saupe 的灌木。可直接開啟的 Fractint .l 檔案採用 Stone Soup Group 的格式。
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-conway-title = 生命遊戲：{ $who }
help-ack-conway = 細胞自動機 B3/S23（1970），透過 Martin Gardner 在 Scientific American 的「Mathematical Games」專欄（1970 年 10 月）而廣為世人所知。
# A credit's heading: the website, then who runs it. $site: the website's name.
help-ack-lifewiki-title = { $site }：ConwayLife.com 社群
help-ack-lifewiki = 程式所讀取的規則記法（B/S、von Neumann、Generations、Alan Hensel 用於非總和型規則的字母，以及 MAP）、它與 Golly 共用的 RLE 圖樣格式，以及內建規則與圖樣的參考資料，該網站也記錄了這些規則與圖樣的發現者。
help-ack-foundations = 基礎與工具
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mandelbrot-title = Mandelbrot 集合：{ $who }
help-ack-mandelbrot = 這一切的核心對象。
help-ack-built-with = 以 Rust、egui/eframe（emilk）與 wgpu（gfx-rs）打造；任意精度採用 astro-float，雙字浮點數採用 twofloat；PNG/OpenEXR、rfd、serde。感謝它們的作者與維護者。
help-ack-errors = 任何歸屬上的錯誤都由作者負責；歡迎在專案的 issue 追蹤系統上提出更正。

## Help ▸ Third-party licenses

help-licenses-title = 第三方授權
# $licenses: Fractadyne's license expression (MIT OR Apache-2.0), as written. $url: the web address of a crate's source. THIRD-PARTY-NOTICES.md is a file name, `option-ext` a crate's name, MPL-2.0 a license's.
help-licenses-intro = Fractadyne 以靜態連結的執行檔發行，其中包含下列開放原始碼 crate 與隨附的字型。完整的授權條款（也收錄在下載內容所附的 THIRD-PARTY-NOTICES.md 中）如下。Fractadyne 本身的程式碼採用 { $licenses } 雙重授權。`option-ext` 採用 MPL-2.0：其未經修改的原始碼位於 { $url }。
# A button, after a copy icon: copies every notice to the clipboard.
help-licenses-copy-all = 複製所有聲明

## Help ▸ About

help-about-tagline = 一套以 Rust 與 wgpu 打造的原生碎形探索程式。
# The arithmetic deep zoom has used. $ran: the libraries that have run so far, as names (or a note that none has yet); $built: those this build contains, with versions.
help-about-arithmetic = 深度縮放算術：{ $ran }（此版本包含：{ $built }）。
help-about-files = 檔案與資料
# Followed by the folder's path.
help-about-files-text = 你的工作階段（目前畫面、著色與偏好設定）、書籤及其縮圖、已儲存的漸層與已儲存的公式，都存放在這個使用者專屬的資料夾中，並在下次啟動時重新載入：
# Tooltip of the copy button beside the folder's path.
help-about-copy-path = 複製路徑
# $command: a command typed in a terminal (it stays English).
help-about-session = 工作階段檔案有版本編號；如果它是由較新版的 Fractadyne 寫入、此版本無法完整讀取，程式會盡力載入並警告你。若要完全重新開始，請使用「檔案 ▸ 重設應用程式狀態…」（會先詢問），或在終端機中執行 `{ $command }`。
help-about-license = 授權
# $licenses: the license expression (MIT OR Apache-2.0), which stays as written.
help-about-license-text = { $licenses }：你可以自行選擇依其中任一授權使用。
# A link to the source code, followed by ↗.
help-about-source = GitHub 上的原始碼

## The live render panel ("blinkenlights": View ▸ Live render panel, key B)
## A front panel drawn over the view like a piece of 1970s test equipment. Its legends are engraved
## in capitals: write them in capitals here (the panel shows them exactly as written, it does not
## uppercase them). Numbers, units and readouts on its tubes and dials are not translated.

# Tooltip of the × button in the panel's top-right corner. The code adds the key after it: (B).
live-close-hover = 關閉即時算繪面板
# The legend engraved along the top plate, left of the maker's badge.
live-plate = 即時算繪 · 前面板

## Live render panel: instrument titles
## Engraved above each instrument, and repeated as the title of its help card.

# Above the depth readout, whose tubes show the magnification as a power of two ("2^" stays).
live-title-depth = 深度 2^
live-title-iterations = 疊代次數
# Above the frame-time readout (milliseconds).
live-title-frame-ms = 影格 MS
# Above the date-and-time readout.
live-title-clock = 日期 · 時間
# The walk dial: a radar screen showing a slow refresh rendered pass after pass, band after band.
live-title-walk = 推進 · 區段
# A meter of how far the picture on screen lags the view.
live-title-age = 圖像年齡
# What part of the picture this frame's GPU pass covered.
live-title-tiles = 圖塊
# A scope of the GPU's work per frame, in milliseconds.
live-title-scope = GPU 回合 · MS
# The depth dial: depth, the reference orbits being built, and the arithmetic in use.
live-title-dial = 深度 · 參考軌道 · 算術
# Iteration steps per second.
live-title-steps = 步數 / 秒
# The two GPUs (the window's and a second, worker GPU).
live-title-engines = 雙引擎
live-title-memory = 記憶體
live-title-cores = 核心 · 執行緒
live-title-pixels = 像素流
# Help card titles only: the three GPU timer lamps (T0, T1, T2 stay as they are) and the row of lamps.
live-title-timers = GPU 計時器 T0–T2
live-title-bank = 指示燈排

## Live render panel: the light bank's lamps
## Engraved under each lamp, and the title of its help card. A cell is narrow (about eight capital
## letters): a longer label breaks at a space onto two lines, and is drawn smaller if it still does
## not fit.

live-lamp-plan = 規劃
live-lamp-dispatch = 派送
live-lamp-live = 即時
live-lamp-held = 保留
# The last picture reprojected (stretched) to fit the moving view.
live-lamp-reproj = 重投影
live-lamp-reading = 讀數
# A pass of a pinned walk.
live-lamp-pin-pass = 固定回合
live-lamp-split = 分割
live-lamp-adopt = 採用
live-lamp-abandon = 放棄
# "REF": a reference orbit. Built for the view as it is (LIVE), ahead of it (AHEAD), or for a tour's next hold (HOLD).
live-lamp-ref-live = 即時參考
live-lamp-ref-ahead = 預先參考
live-lamp-ref-hold = 停留參考
# The series approximation.
live-lamp-series = 級數
# The second (worker) GPU.
live-lamp-worker = 輔助
live-lamp-tile = 圖塊
# Supersampling (anti-aliasing); usually kept as SSAA.
live-lamp-ssaa = SSAA
live-lamp-palette = 調色盤
# The arithmetic in use (direct, df32, floatexp).
live-lamp-mode = 模式

## Live render panel: the kinds of GPU pass
## In the GPU scope's legend (MOVING, PIN, WORKER, SETTLE, SSAA). SETTLE and SSAA also name a still
## view's walk on the walk dial.

# A moving frame's pass.
live-kind-moving = 移動
# A pass of a pinned walk.
live-kind-pin = 固定
# The worker GPU's passes.
live-kind-worker = 輔助
# A still view's settle tile or walk pass.
live-kind-settle = 穩定
# A supersampling sample.
live-kind-ssaa = SSAA

## Live render panel: legends and readouts on the instruments

# On the walk dial during a split refresh. $done: passes done; $passes: passes in all.
live-walk-split = 分割 { $done }/{ $passes }
# On the picture-age meter's face, under its scale (0 to 2 octaves).
live-age-scale = 落後八度數
# The depth dial's band for direct (plain floating-point) arithmetic. The other bands, DF32 and FLOATEXP, are names and stay.
live-dial-direct = 直接
# Under the two GPU gauges: GPU 0 draws the window; GPU 1 is the worker, or there is none.
live-engines-gpu-window = GPU 0 · 視窗
live-engines-gpu-worker = GPU 1 · 輔助
live-engines-no-worker = 無輔助
# The share of displayed frames each GPU made. $window and $worker: percentages, 0–100.
live-engines-on-screen = 顯示 { $window } : { $worker }
# The same readout before any frame was shown.
live-engines-on-screen-none = 顯示 —
# Under the CPU core bars. $count: the machine's cores.
live-cores-count = { $count } 核心
# Under the threads scope. $count: the app's busy threads.
live-cores-threads = 執行緒 { $count }

## Live render panel: the memory tanks' legend
## What the GPU memory holds, by kind. Very narrow: about ten letters.

# G-buffers (the picture's working textures).
live-memory-gbuf = G-BUF
# The held picture and the supersampling accumulator.
live-memory-hold = 保留+SSAA
# The reference orbit and its BLA (bilinear approximation) table.
live-memory-orbit = 軌道+BLA
# The worker GPU's copy of the reference.
live-memory-worker = 輔助參考

## Live render panel: the roles on the threads scope
## Tiny labels on a small scope: at most four letters each.

# The reference builders.
live-cores-role-ref = 參考
# The scoring pool.
live-cores-role-pool = 評分
# The thread driving the worker GPU.
live-cores-role-worker = 輔助
# Probes: short side renders.
live-cores-role-probe = 探測
# Image and tour exports.
live-cores-role-export = 匯出

## Live render panel: the tiles instrument's captions
## The line under the picture's plane, then the tile size (left) and the tiles a frame (right).

# A walk's pass. $lo and $hi: the iteration band it rendered (numbers).
live-tiles-pin-band = 固定回合 · 區段 { $lo }–{ $hi }
live-tiles-settle-band = 穩定回合 · 區段 { $lo }–{ $hi }
live-tiles-moving-whole = 移動影格 · 整張圖像
live-tiles-settled-whole = 已穩定影格 · 整張圖像
# $pass: this pass; $passes: the passes the refresh is split into.
live-tiles-split-pass = 分割更新 · 第 { $pass }/{ $passes } 回合
# $grid: the grid's columns × rows, e.g. 12×8.
live-tiles-grid-done = 穩定網格 { $grid } · 完成
# $grid: the grid's columns × rows; $done and $total: tiles done, and in all.
live-tiles-grid-progress = 穩定網格 { $grid } · 已完成 { $done }/{ $total }
# The same, while a tile too costly for one frame goes out in strips.
live-tiles-grid-strips = 穩定網格 { $grid } · 已完成 { $done }/{ $total } · 分條傳送
# $samples: supersampling samples folded so far; $lo and $hi: the iteration band the next one is walking.
live-tiles-ssaa-band = SSAA · 已完成 { $samples } · 區段 { $lo }–{ $hi }
# $sample: the supersampling sample.
live-tiles-ssaa-whole = SSAA 樣本 { $sample } · 整個影格
# No GPU pass this frame, and the panel has not seen a frame yet.
live-tiles-idle-starting = 啟動中
# No GPU pass this frame: the last complete picture is held on screen.
live-tiles-idle-held = 保留
# No GPU pass this frame: the last picture is stretched to fit the view.
live-tiles-idle-reprojected = 重投影
# No GPU pass this frame: the picture on screen is current.
live-tiles-idle-live = 即時
# No GPU pass this frame, and nothing to show.
live-tiles-idle-none = 閒置
# The tile size: $res, a whole picture's resolution (2400×1300).
live-tiles-tile-res = 圖塊 { $res }
# The tile size: $px, a tile's side in pixels.
live-tiles-tile-px = 圖塊 { $px } PX
live-tiles-no-pass = 無回合
# Tiles a frame. $count: how many (0 when none).
live-tiles-rate-count = { $count }/影格
live-tiles-rate-one = 1 圖塊/影格
# $share: the part of one tile sent each frame, e.g. 0.33.
live-tiles-rate-part = { $share } 圖塊/影格
live-tiles-rate-sample = 1 圖塊/樣本
# In the plane's corner during a walk: the share of pixels still iterating. $percent: a percentage, e.g. 54 or 1.5.
live-tiles-running = 執行中 { $percent }%

## Live render panel: help cards
## Shown when the pointer rests on an instrument or a lamp. Words in capitals (SETTLE, SSAA, ABANDON,
## HELD, REPROJ, ADOPT, REF AHEAD, ON SCREEN) are the panel's own legends: use the same words as
## their messages above.

# The heading of a card's list of what healthy looks like.
live-card-healthy = 正常
# The heading of a card's list of what signals trouble.
live-card-trouble = 異常

live-card-walk-what = 對一個影格來說成本太高的深層更新，會被固定下來並逐段推進：一個回合接一個回合地算繪，每個回合讓像素多疊代一個八度（256、512、1,024 … 一直到要求的次數）。外框就是以對數刻度表示的這個次數，從頂端開始；掃描光束是推進的進度；每段弧是一個回合，依其 GPU 時間相對於 20 ms 目標著色。白色光點在等待上一個回合的讀數；閃光表示新的圖像上了螢幕。
live-card-walk-note = 對一個回合來說成本太高的靜止畫面，也會以同樣方式逐段推進，標示為「穩定」；每個超取樣樣本會再推進一次，標示為「SSAA」：它們的每個回合最多可花 400 ms，弧的顏色也是相對於這個時間。分割更新則改為每個回合畫一個圓環：每個回合包含全部區段，但只涵蓋一部分圖塊。
live-card-walk-good-sweep = 掃描在幾個影格內轉完一圈，並以閃光結束。
live-card-walk-good-arcs = 弧大多是綠色與琥珀色。
live-card-walk-good-still = 在深層的靜止畫面上：每個樣本的掃描較慢，弧從綠色到琥珀色。
live-card-walk-bad-red = 紅色的弧：回合超出預算，感覺起來就是卡頓。
live-card-walk-bad-restart = 掃描到一半停下又重新開始：推進被放棄了，螢幕上的圖像仍是舊的。
live-card-walk-bad-hollow = 空心的弧：計時始終沒有傳回的回合，因此下一個回合只能盲目決定大小。

live-card-age-what = 螢幕上的圖像落後畫面多少，以縮放的八度表示（一個八度是 2×）。更新算繪期間，舊圖像會被拉伸顯示並逐漸變舊；更新完成時，年齡就會下降。下方的長條是最近幾秒的記錄。
live-card-age-good-sawtooth = 在大約 0.3 到 0.8 之間的鋸齒波，指針在 0.5 刻度附近擺動。
live-card-age-good-still = 你停下後接近 0。
live-card-age-bad-red = 指針一直停在 1.0 以上的紅區：圖像落後超過 2×，縮放時看起來模糊或有方塊。
live-card-age-bad-ramp = 只升不降的斜坡：更新沒有完成（請注意「放棄」）。

live-card-tiles-what = 這個影格的 GPU 回合如何涵蓋圖像。移動影格與推進回合會涵蓋整張圖像，說明文字會標出推進回合的疊代區段。推進的回合只會疊代尚未逃逸的像素：平面會依此比例填滿（對數刻度，底部為 0.01%），並在角落顯示這個比例，因此推進越深，它就越少。對一個影格來說成本稍高的更新，會分割成最多 8 個回合，每個回合是一組交錯的 16 px 圖塊。對一個回合來說成本太高的已穩定畫面，會變成圖塊網格，從中心向外每個影格一個圖塊，而成本仍然太高的圖塊會分條傳送。下方讀數：圖塊大小與每個影格的圖塊數。
live-card-tiles-good-whole = 縮放時是完整的回合；分割更新只有幾個回合。
live-card-tiles-good-grid = 你停下時：數十個圖塊組成的網格在一兩秒內填滿，接著是「SSAA」。
live-card-tiles-bad-small = 由數百個小圖塊組成的網格：每個回合的預算很小，銳利化需要很長時間。
live-card-tiles-bad-strips = 許多圖塊以分條方式傳送：圖塊執行得比網格規劃時預期的慢。
live-card-tiles-bad-splits = 分割一直停在 8 個回合：更新勉強塞得下，再深一點就會退回較慢的逐段推進。
live-card-tiles-bad-dark = 移動時長時間沒有顯示：完全沒有回合。

live-card-scope-what = 每個影格一欄，最新的在右邊：它執行的 GPU 工作，依種類堆疊（移動影格為綠色，固定回合依成本由綠到紅，穩定圖塊或靜止畫面的推進回合為米色，超取樣樣本為淡紫色，輔助 GPU 的影格與樣本為藍色），單位為毫秒。虛線位於 16.7 ms（60 fps）、33 與 50。空心標記表示計時始終沒有傳回的回合。
live-card-scope-good-under = 移動時欄位低於 16.7 線。
live-card-scope-good-pin = 固定回合為綠色到琥珀色。
live-card-scope-good-samples = 靜止畫面的超取樣樣本可能執行得較久：沒有東西在移動。
live-card-scope-bad-over = 欄位超過 33 或 50 ms：可見的卡頓。
live-card-scope-bad-seconds = 長達數秒的回合：GPU 的看門狗可能會重設裝置。
live-card-scope-bad-hollow = 許多空心標記：工作量是在沒有量測的情況下決定的。

live-card-timers-what = 三個 GPU 時間戳記槽。回合被計時時，槽會亮起；兩三個影格後讀數傳回時，就會變暗。T0 為每個影格的回合計時；T1 與 T2 為額外的固定回合計時。
live-card-timers-good = 只要有工作，T0 就會閃爍；推進期間 T1 與 T2 會閃爍。
live-card-timers-bad-stuck = 某盞燈一直亮著：它的讀數沒有傳回，定價會變得過時。
live-card-timers-bad-dark = GPU 忙碌時全部熄滅：沒有任何東西在被量測。

live-card-dial-what = 紅色指針是畫面的深度：以 2 的次方表示的放大倍率。各個區段是 GPU 在該處使用的算術：直接、df32（每個數字兩個浮點數），以及超過 2^93 之後的 floatexp（指數範圍很寬的浮點數）。圓點是正在建構的參考軌道：綠色代表目前的畫面，琥珀色代表畫面前方、縮放前進的方向，藍色代表導覽的下一個停留點。圓環表示一次安裝；紅色叉號表示因畫面超越而被捨棄的建構。
live-card-dial-good-ahead = 琥珀色圓點落在指針前方，並在指針到達之前變成圓環。
live-card-dial-good-crosses = 紅色叉號很少。
live-card-dial-bad-passing = 指針越過最後一個已安裝的參考軌道：影格得等待參考軌道，感覺起來就像凍結。
live-card-dial-bad-crosses = 反覆出現紅色叉號：縮放跑得比建構器快，CPU 被浪費了。
live-card-dial-bad-crossing = 越過 2^93 是已知的慢點：最初的 floatexp 影格成本較高。

live-card-bank-what = 每盞燈在其事件發生時亮起，並在大約 150 ms 內淡出，因此忙碌的算繪器會閃爍，閒置時則會熄滅。由左至右：影格、更新、參考軌道，接著是輔助、穩定、調色盤與算術。
live-card-bank-note = 將游標停在指示燈上，即可查看它的意義。

live-card-depth-what = 以 2 的次方表示的放大倍率：2^100 約為 1.3 × 10^30。
live-card-depth-note = 超過 2^93 後，每一步的成本都更高（floatexp），參考軌道的建構也需要更久。

live-card-iter-what = 疊代要求：每個像素在被視為位於集合內之前可以疊代多少次。它隨深度攀升。
live-card-iter-good = 下潛時穩定攀升。
live-card-iter-bad-low = 在深處時一直停在低值：螺旋與迷你 Mandelbrot 會算繪成平坦的內部。
live-card-iter-bad-high = 非常高：每次更新的成本都更高，推進也需要更久。

live-card-ms-what = 顯示的影格之間的時間。
live-card-ms-good = 你顯示器的更新間隔：60 Hz 顯示器上為 16.7。
live-card-ms-bad-stutter = 高於 33：可見的卡頓。
live-card-ms-bad-spikes = 高於 50 的尖峰：頓挫，通常是某個成本很高的回合（參見「GPU 回合」）。

live-card-clock-what = 本地時間的實際時鐘，精確到十分之一秒。
live-card-clock-note = 它會在面板的任何螢幕截圖上標出拍攝時間，讓出問題的畫面可以與程式的記錄檔對照。

live-card-tach-what = 每秒疊代步數。白色：要求的步數（像素 × 疊代次數）。琥珀色：GPU 實際執行的步數。兩者的差距就是 BLA（雙線性近似）一次跳過許多疊代；×N 就是這個倍數。里程表累計整個工作階段。
live-card-tach-good = 在深處差距很大：BLA 正在發揮作用，×10 到 ×1,000。
live-card-tach-bad-together = 在深處兩根指針靠在一起：BLA 在這裡無法跳躍（常見於迷你 Mandelbrot 的內部附近），因此更新很慢。
live-card-tach-bad-red = 白色指針進入紅區：要求的超過 GPU 在一個影格內能做的。

live-card-engines-what = 每張 GPU 的忙碌程度，並以圓點將它的影格送到螢幕上：它的回合在最近一秒內的 GPU 時間，包括輔助 GPU 的移動影格與超取樣樣本。「顯示」是每張 GPU 產生的已顯示影格所占的比例。只有一張 GPU 時，輔助那一側會保持閒置。
live-card-engines-good = 下潛時兩者都很忙碌，輔助 GPU 提供一部分影格。
live-card-engines-bad-pinned = GPU 0（視窗的）一直接近 100%：它是瓶頸，影格會卡頓。
live-card-engines-bad-idle = 輔助 GPU 已啟用卻閒置：它沒有在提供影格。

live-card-memory-what = RAM：程式的工作集，紅線是它的峰值。VRAM：程式在 GPU 上占用的記憶體，依它配置的大小估算，按種類區分（G-buffer、保留與 SSAA 紋理、軌道與 BLA 緩衝區、輔助 GPU 的參考軌道副本），並與顯示卡的容量對照。
live-card-memory-good = 縮放時保持平穩；隨參考軌道的來去而上升、再下降。
live-card-memory-bad-leak = RAM 持續攀升且從不下降：記憶體洩漏。
live-card-memory-bad-vram = VRAM 接近顯示卡的容量：配置可能失敗，或裝置可能遺失。

live-card-cores-what = 每個核心的 CPU 使用率，由綠到琥珀到紅，以及一個示波器，其中每個忙碌的執行緒繞著它的角色運行：參考軌道建構器、評分池、輔助、側邊算繪。
live-card-cores-good = 縮放時有一兩個核心忙碌；挑選參考點時，所有核心會短暫地一起忙碌。
live-card-cores-bad-every = 所有核心忙碌長達數秒：在極深處建構參考軌道。
live-card-cores-bad-own = 程式自己的核心一直在紅區：視窗本身會卡頓。

live-card-pixels-what = 每秒計算的像素數，單位為 Mpx/s。暗：仍在推進中疊代。亮：已完成。
live-card-pixels-good = 移動時每秒數十 Mpx。
live-card-pixels-bad = 圖像過時時卻接近零：GPU 在等待某樣東西，例如參考軌道或讀數。

## Live render panel: the lamps' help cards

live-card-lamp-plan-what = 已規劃一個影格：選定解析度、疊代要求與預算，以及是否要更新。
live-card-lamp-plan-good = 每當畫面或設定改變時閃爍。
live-card-lamp-plan-bad = 縮放時熄滅：UI 執行緒卡住了。

live-card-lamp-dispatch-what = 這個影格有工作送到 GPU。
live-card-lamp-dispatch-good = 有任何變化時亮著；閒置時熄滅。
live-card-lamp-dispatch-bad = 沒有任何變化卻很忙碌：程式沒有穩定下來。

live-card-lamp-live-what = 螢幕上的圖像是這個影格自己算繪的。
live-card-lamp-live-note = 深層下潛時很少亮起，這是正常的：參見「保留」與「重投影」。
live-card-lamp-live-good = 在淺層時亮著，因為那裡一個影格的成本很低。

# "Prefer detail" is a setting's name.
live-card-lamp-held-what = 上一張完整的圖像留在螢幕上，下一張在它下方合成中：深度縮放的更新，或在開啟「縮放時優先顯示細節」時，靜止畫面的下一個圖塊或超取樣樣本。
live-card-lamp-held-good-dive = 深度縮放時斷斷續續地亮著，中間被「採用」打斷。
live-card-lamp-held-good-still = 靜止畫面銳利化時穩定亮著，同時「SSAA」在閃爍。
live-card-lamp-held-bad = 縮放時亮了好幾秒卻沒有「採用」：更新沒有完成。

live-card-lamp-reproj-what = 上一張圖像被拉伸以配合移動中的畫面，直到新的圖像準備好。
live-card-lamp-reproj-good = 移動時短暫亮起。
live-card-lamp-reproj-bad = 長時間亮著：圖像拉伸得越多就越模糊。

live-card-lamp-reading-what = 從 GPU 傳回了一個讀數：某個回合的計時、逃逸範圍或內容。
live-card-lamp-reading-good = 算繪時穩定地閃爍。
live-card-lamp-reading-bad = 算繪時長時間熄滅：計時沒有傳回，預算只能用猜的。

live-card-lamp-pin-pass-what = 固定推進中的一個回合：圖像多算繪了一個疊代區段。
live-card-lamp-pin-pass-good = 一連串以「採用」結束的回合。
live-card-lamp-pin-pass-bad = 一連串以「放棄」結束的回合。

live-card-lamp-split-what = 分割更新中的一個回合：一部分的圖塊，以完整的疊代次數算繪。
live-card-lamp-split-good = 以「採用」結束的短串回合。
live-card-lamp-split-bad = 連續 8 個，也就是允許的最大值：更新勉強塞得下。

live-card-lamp-adopt-what = 一次完成的更新取代了螢幕上的圖像。
live-card-lamp-adopt-good = 縮放時每秒好幾次。
# "Picture age" names the instrument (its title, PICTURE AGE).
live-card-lamp-adopt-bad = 縮放時很少出現：圖像會變舊（參見「圖像年齡」）。

live-card-lamp-abandon-what = 進行中的更新被丟棄了：畫面移動得太遠、它變得太舊，或被較新的更新取代。被畫面超越的參考軌道建構會讓它微微亮起。
live-card-lamp-abandon-good = 很少，或在跳躍後立刻出現一次。
live-card-lamp-abandon-bad = 頻繁出現：工作被浪費，圖像仍是舊的。

live-card-lamp-ref-live-what = 正在為目前的畫面建構參考軌道：畫面沒有參考軌道、已離開原有的參考軌道，或需要更長的參考軌道。
live-card-lamp-ref-live-good = 短暫閃爍。
live-card-lamp-ref-live-bad-long = 長時間亮著：深層畫面在等待它。
live-card-lamp-ref-live-bad-lookahead = 下潛時經常亮起，而「預先參考」卻是熄滅的：預先計算跟不上。

live-card-lamp-ref-ahead-what = 正在比畫面更深、縮放前進的方向上建構預先計算的參考軌道。
live-card-lamp-ref-ahead-good = 下潛時規律地閃爍。
live-card-lamp-ref-ahead-bad = 深層下潛時熄滅：畫面將會超越它的參考軌道。

live-card-lamp-ref-hold-what = 導覽的停留點預先擷取：下一個停留關鍵影格的參考軌道，在滑向它的途中建構，讓播放永遠不必等待。
live-card-lamp-ref-hold-note = 導覽播放以外的時間會熄滅。
live-card-lamp-ref-hold-good = 導覽中每個停留點之前閃一下。
live-card-lamp-ref-hold-bad = 整個停留期間都亮著：預先擷取太晚，導覽必須等待它。

live-card-lamp-series-what = 為新參考軌道計算的級數近似，讓每個像素都能略過它最初的疊代。
live-card-lamp-series-good = 很短暫。
live-card-lamp-series-bad = 在深處持續很久：它會延遲參考軌道的安裝。

live-card-lamp-worker-what = 第二張 GPU 送出了一個影格。
live-card-lamp-worker-good = 啟用輔助 GPU 時穩定閃爍。
live-card-lamp-worker-bad = 已啟用輔助 GPU 卻熄滅：它沒有在提供影格。

live-card-lamp-tile-what = 一個穩定圖塊（或其中一條）送到了 GPU。
live-card-lamp-tile-good = 你停下後一陣閃爍，然後安靜下來。
live-card-lamp-tile-bad = 永不停止：網格一直重新開始，因為有東西一直在改變畫面或設定。

live-card-lamp-ssaa-what = 一個超取樣樣本加入了已穩定的圖像。
live-card-lamp-ssaa-good = 穩定後一連串閃爍，然後熄滅。
live-card-lamp-ssaa-bad = 一再重新開始：穩定過程不斷被打擾。

live-card-lamp-palette-what = 色彩正規化採用了新的逃逸範圍，因此色彩會跟著縮放變化。
live-card-lamp-palette-good = 縮放時閃爍。
live-card-lamp-palette-bad = 縮放時熄滅：調色盤範圍被凍結，色彩變得平淡、對比度低。

live-card-lamp-mode-what = 算術改變了：直接、df32 或 floatexp。
live-card-lamp-mode-good = 在 2^93 這類邊界閃一下。
live-card-lamp-mode-bad = 來回閃爍：畫面在邊界兩側反覆擺盪。

## The Render client window (File ▸ Render client…): lending this machine's GPU to a render farm

farm-client-title = 算繪用戶端
farm-client-intro = 為算繪農場算繪影格。主控端（另一台電腦上的「工具 ▸ 在算繪農場上算繪…」）會顯示它的位址與農場金鑰。這台電腦不會開啟任何連接埠：它會主動連線到主控端。
# A field label, and the label of the status row that names the controller.
farm-client-controller = 主控端
farm-client-find = 尋找
farm-client-find-hover = 在這個網路上尋找主控端（主控端在其農場監聽期間會回應）
farm-client-key = 農場金鑰
# A checkbox beside the masked farm key: show it in clear.
farm-client-show-key = 顯示
# A field label (the name this machine goes by), and the label of the status row about this machine.
farm-client-machine = 這台電腦
farm-client-machine-hover = 主控端為這台電腦顯示的名稱。
farm-client-share = 共用磁碟
# Greyed hint inside the empty field; the path is an example.
farm-client-share-hint = 選填，例如 \\fileserver\share
farm-client-share-hover = 這台電腦通往主控端所用共用磁碟的路徑（共用模式）：影格會寫入該處，而不是透過連線傳送。留空則會傳送每個影格。
farm-client-gpu = 顯示卡
# The label of the row with the idle option.
farm-client-when = 時機
# A checkbox followed by a number of minutes ("Only when idle for [5] min").
farm-client-when-idle = 僅在閒置達
farm-client-when-idle-hover = 只有在沒有人碰這台電腦的鍵盤或滑鼠達到這段時間後，或螢幕鎖定時，才接受工作。有人開始使用時，進行中的影格會立刻停止並交回農場。
# A collapsible section: the largest job this machine accepts.
farm-client-limits = 限制
farm-client-limits-note = 要求超過這些限制的工作會被拒絕，絕不會被縮減。
farm-client-limit-size = 最大影格
farm-client-limit-ss = 最高超取樣
farm-client-limit-iter = 最多疊代次數

## Render client: the graphics-card choice

farm-client-gpu-system = 由系統選擇
farm-client-gpu-every = 每一張顯示卡
# $count: this machine's graphics cards.
farm-client-gpu-every-n = 每一張顯示卡（{ $count }）
# $count: this machine's graphics cards.
farm-client-gpu-every-each = 每一張顯示卡（{ $count }），各自一個工作階段
# A card the window cannot name. $number: its number (or numbers) as --list-adapters gives it.
farm-client-gpu-adapter = 介面卡 { $number }
farm-client-gpu-finding = 正在尋找這台電腦的顯示卡…
# "NAME · GPU 1" is how such a machine is named in the controller's list: keep it as it is.
farm-client-gpu-hover = 由哪一張顯示卡算繪農場的影格。選擇每一張顯示卡時，每張卡都會以獨立的電腦身分加入農場（「NAME · GPU 1」、「NAME · GPU 2」），而「暫停」與「中斷連線」會作用於所有顯示卡。

## Render client: looking for controllers on the network (the Find button)

farm-client-finding = 正在這個網路上尋找主控端…
# $port: the UDP port number.
farm-client-found-none = 沒有主控端回應。主控端在其農場監聽期間會回應；它的防火牆必須允許 UDP { $port } 傳入。
farm-client-found-pick = 在這個網路上，按一下其中一個即可使用：
# $identity: the controller's identity fingerprint.
farm-client-found-hover = 身分 { $identity }：使用這個主控端（仍需要它的農場金鑰）
# After a warning sign. $version and $git: this app's version and build.
farm-client-found-other-build = 與此版本不同的組建（{ $version } { $git }）：它會拒絕這台電腦。

## Render client: what is wrong with what was typed, or with starting the client

farm-client-bad-address = 請以 HOST:PORT 格式輸入主控端的位址，與其視窗顯示的相同（例如 192.168.1.20:46733）。
# $why: what is wrong with the key (in English).
farm-client-bad-key = 農場金鑰：{ $why }
# $why: what is wrong with the name (in English).
farm-client-bad-name = 名稱：{ $why }
farm-client-no-config = 沒有可以存放農場金鑰的設定資料夾。
# $why: the system's reason.
farm-client-start-failed = 無法啟動算繪用戶端：{ $why }

## Render client: the state, in the status heading

farm-client-phase-connecting = 連線中…
farm-client-phase-checking = 正在執行自我檢查…
farm-client-phase-idle = 已連線，等待工作中
farm-client-phase-rendering = 算繪中
farm-client-phase-in-use = 等待中：有人正在使用這台電腦
farm-client-phase-paused = 已暫停
farm-client-phase-retrying = 無法連上主控端，正在重試
# $seconds: the wait before the next try.
farm-client-phase-retrying-in = 無法連上主控端，{ $seconds } 秒後重試
farm-client-phase-disconnected = 已中斷連線
farm-client-phase-ended = 已結束

## Render client: the line under the status heading

# $address: the controller's address (HOST:PORT).
farm-client-detail-connecting = 正在連線到 { $address }…
# $controller: the controller's name.
farm-client-detail-checking = 已連線到「{ $controller }」，正在執行自我檢查
# $controller: the controller's name.
farm-client-detail-approval = 已連線到「{ $controller }」，等待其使用者核准這台電腦
# $controller: the controller's name.
farm-client-detail-idle = 已連線到「{ $controller }」，閒置中
# $why: the check that failed and why (in English).
farm-client-detail-check-failed = 自我檢查失敗：{ $why }
# $why: the controller's reason (in English).
farm-client-detail-refused = 被主控端拒絕：{ $why }
# $why: the controller's reason (in English).
farm-client-detail-closed = 主控端關閉了連線：{ $why }
# $why: the controller's reason (in English).
farm-client-detail-ended = 主控端結束了連線：{ $why }
# $job: the job's name.
farm-client-detail-bundle = 工作「{ $job }」：正在接收其資料包
# $job: the job's name.
farm-client-detail-job-waiting = 工作「{ $job }」：等待要算繪的影格
# $why: the limit the job is over (in English).
farm-client-detail-job-refused = 已拒絕此工作：{ $why }
farm-client-detail-job-over = 工作已結束
# $first and $last: frame numbers.
farm-client-detail-rendering = 正在算繪影格 { $first }–{ $last }
# $time: how long nobody must use the machine ("90 s", "5 min").
farm-client-detail-in-use = 等待中：有人正在使用這台電腦；無人使用達 { $time } 後恢復工作
farm-client-detail-paused = 已暫停，並已通知主控端
farm-client-detail-pausing = 正在暫停，先完成進行中的影格
farm-client-detail-cancelled = 已取消進行中的影格；已暫停
# A machine with several graphics cards. $rendering: the cards rendering now; $total: all its cards (two or more).
farm-client-rendering-on-cards = 正在 { $total } 張顯示卡中的 { $rendering } 張上算繪

## Render client: the status rows

# The value of the Controller row. $name: the controller's name; $address: HOST:PORT; $identity: its identity fingerprint.
farm-client-row-controller-value = 「{ $name }」 · { $address } · 身分 { $identity }
# The value of the This machine row. $name: this machine's name; $identity: its identity fingerprint.
farm-client-row-machine-value = 「{ $name }」 · 身分 { $identity }
farm-client-row-gpu = GPU
# One graphics card's row. $slot: the card's number.
farm-client-row-gpu-n = GPU { $slot }
farm-client-row-job = 工作
# The frames handed to this machine.
farm-client-row-run = 批次
farm-client-row-sent = 已傳送
# The Run row, in parts joined with " · ": the run, the frame in progress, its time. $first and $last: frame numbers.
farm-client-run-frames = 影格 { $first }–{ $last }
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-client-run-frame = 影格 { $frame }（第 { $index }/{ $count } 個）
# $time: the time spent on it so far ("2.1 s").
farm-client-run-frame-time = 此影格已花 { $time }
# One graphics card's run. $frame: the frame in progress; $first and $last: the run's frames.
farm-client-session-frame = 影格 { $frame }，範圍 { $first }–{ $last }
farm-client-session-no-card = 尚未得知顯示卡
# The Sent row. $count: frames sent; $time: the mean time per frame ("2.1 s").
farm-client-sent = { $count } 個影格 · 每個影格 { $time }
# The Sent row of a machine with several cards. $count: frames sent; $split: each card's share ("812 + 422"); $time: the mean time per frame.
farm-client-sent-split = { $count } 個影格（{ $split }） · 每個影格 { $time }
# After a warning sign. $note: how the GPUs differ, and both GPUs (in English).
farm-client-gpu-note = 這台電腦：{ $note }。它的影格會與主控端的略有不同，這在停留的鏡頭中可能看得出來。
# A collapsible section: the self-check's results.
farm-client-self-check = 自我檢查
farm-client-thumb-hover = 這台電腦傳送的最後一個影格
farm-client-resume = 繼續
farm-client-resume-hover = 再次接受工作
farm-client-pause = 暫停
farm-client-pause-hover = 完成進行中的影格，然後不再接受工作。主控端會將這台電腦的影格交給其他電腦。
farm-client-cancel-frame = 取消影格
farm-client-cancel-frame-hover = 立即停止進行中的影格並暫停；主控端會在其他電腦上算繪它。
farm-client-live-paused = 這台電腦算繪農場影格期間，即時畫面會暫停。
farm-client-close = 關閉
farm-client-close-active-hover = 關閉視窗；這台電腦仍會留在農場中（可從「檔案」選單重新開啟）
farm-client-close-hover = 關閉視窗
farm-client-disconnect = 中斷連線
farm-client-disconnect-hover = 離開農場：進行中的影格會交回
farm-client-connect = 連線
farm-client-connect-hover = 連線到主控端並替它算繪影格

## The Render on farm window (Tools ▸ Render on farm…): the controller of a render farm

farm-ctl-title = 在算繪農場上算繪
farm-ctl-out = 輸出資料夾
farm-ctl-browse = 瀏覽…
farm-ctl-out-note = 影格，以及 farm/ 中的工作狀態：再次執行同一個工作即可繼續。
farm-ctl-size = 尺寸
farm-ctl-size-custom = 自訂…
# The size list's choice when the size is typed in. $width and $height: pixels.
farm-ctl-size-custom-value = 自訂：{ $width }×{ $height }
farm-ctl-fps = 影格/秒
farm-ctl-ss = 超取樣
farm-ctl-port = 監聽連接埠
farm-ctl-port-hover = 用戶端會連線到這個連接埠。請在防火牆中允許你的區域網路使用它。
farm-ctl-share = 共用磁碟
# Greyed hint inside the empty field.
farm-ctl-share-hint = 選填，這台電腦通往它的路徑
farm-ctl-share-hover = 共用模式：擁有相同磁碟的用戶端會將影格寫入該處，而不是透過連線傳送，這台電腦會在那裡檢查每一個影格。沒有該磁碟的用戶端則會傳送它們的影格。
# A label followed by a number of clients and two checkboxes.
farm-ctl-start-with = 開始時需要
# The unit after the number in the box ("Start with [3] clients"). $count: that number.
farm-ctl-start-clients = 個用戶端
farm-ctl-start-clients-hover = 當這麼多台電腦連線並通過自我檢查後，工作就會開始；其他電腦可以稍後加入。
farm-ctl-local = 也在這台電腦上算繪
farm-ctl-local-hover = 這台電腦算作一個用戶端。
farm-ctl-sharing = 共用參考軌道
farm-ctl-sharing-hover = 深層影格的參考軌道可能要花數分鐘到數小時建構。共用時，它只建構一次，再傳給其他電腦。共用的參考軌道是另一個同樣有效的參考軌道：在極深處，會有少數像素與自行建構參考軌道的影格不同。
farm-ctl-key = 農場金鑰
# A checkbox beside the shortened farm key: show all of it.
farm-ctl-show-key = 顯示
farm-ctl-copy-key = 複製金鑰
farm-ctl-copy-key-hover = 將它貼到每個用戶端的「算繪用戶端」視窗中
# In place of the farm key, when there is none.
farm-ctl-no-key = 尚未產生
farm-ctl-make-key = 產生金鑰
farm-ctl-make-key-hover = 用戶端需要它才能加入；按下「算繪」時也會產生
# What to do on the other machines. $address: this controller's HOST:PORT.
farm-ctl-steps = 在每個用戶端上：「檔案 ▸ 算繪用戶端…」、主控端 { $address }，以及農場金鑰。
# The same when this machine's address is not known. $port: the port number.
farm-ctl-steps-port = 在每個用戶端上：「檔案 ▸ 算繪用戶端…」、這台電腦的位址與連接埠 { $port }，以及農場金鑰。
# The size of the job. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
farm-ctl-summary = { $count } 個影格 · 導覽 { $seconds } 秒 · { $width }×{ $height } ss{ $ss }
farm-ctl-strip-hover = 工作中的每個影格，由左至右：已完成、算繪中、尚待處理、已放棄。
farm-ctl-no-tour = 尚未載入導覽。
# $folder: the folder; $why: the system's reason.
farm-ctl-out-error = 輸出資料夾 { $folder }：{ $why }
# $why: the system's reason.
farm-ctl-start-failed = 無法啟動農場：{ $why }
# $code: the controller process's exit code, as the code gives it.
farm-ctl-ended-exit = 主控端已結束（結束代碼 { $code }）。

## Render on farm: the state, in the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-phase-waiting = 等待算繪用戶端（已連線 { $connected }/{ $needed }）
farm-ctl-phase-preparing = 準備工作中
farm-ctl-phase-rendering = 算繪中
farm-ctl-phase-paused = 已暫停
farm-ctl-phase-complete = 已完成
farm-ctl-phase-stopped = 已停止
farm-ctl-phase-ended = 已結束

## Render on farm: the words after the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-detail-waiting = 等待算繪用戶端：已連線 { $connected }/{ $needed }
farm-ctl-detail-preparing = 在這台電腦上量測一次正規化錨點，供所有用戶端使用
farm-ctl-detail-no-space = 等待輸出資料夾騰出空間
farm-ctl-detail-paused = 已暫停：不再分派新工作
farm-ctl-detail-not-rendered = 已結束，但有影格未算繪：在同一個資料夾上再次執行即可繼續
farm-ctl-detail-stopped = 已停止：在同一個資料夾上再次執行即可繼續
# $code: the controller process's exit code.
farm-ctl-detail-ended = 已結束（結束代碼 { $code }）
farm-ctl-detail-finishing = 收尾中

## Render on farm: progress

# On the progress bar. $done: frames done; $total: frames in the job.
farm-ctl-progress = { $done } / { $total } 個影格
# Added to the progress bar. $rate: frames per second, with decimals.
farm-ctl-progress-rate = { $rate } 影格/秒
# Added to the progress bar. $time: the time still to go ("38m 10s").
farm-ctl-progress-eta = 約剩 { $time }
# Facts under the progress bar, joined with " · ". $received and $sent: kilobytes per second.
farm-ctl-io = 接收 { $received } KB/s · 傳送 { $sent } KB/s
# $space: free space on the output drive ("412 GB").
farm-ctl-free = 可用 { $space }
# $time: the time since the job started ("38m 10s").
farm-ctl-elapsed = 已經過 { $time }
# $count: frames given up on.
farm-ctl-failed = { $count } 個未算繪
# Reference orbits. $built: built for this job; $cached: found in the cache; $reused: reused between frames; $shared: passed between machines.
farm-ctl-refs = 參考軌道：建構 { $built } 個、取自快取 { $cached } 個、重複使用 { $reused } 個 · 共用 { $shared } 個
# After a warning sign.
farm-ctl-storage-low = 輸出資料夾快滿了：在騰出空間之前，不會分派新工作。
# After a warning sign.
farm-ctl-gpus-differ = 這個農場中的 GPU 各不相同。它們的影格會略有差異，這在停留的鏡頭中可能看得出來。
# After a warning sign. $count: the GPU classes in the farm.
farm-ctl-gpus-differ-classes = 這個農場中的 GPU 各不相同（{ $count } 種 GPU 類別：不同的 GPU、驅動程式或探測算繪）。它們的影格會略有差異，這在停留的鏡頭中可能看得出來。
# $gpu: this machine's GPU, API and driver.
farm-ctl-this-gpu = 這台電腦：{ $gpu }
# A machine whose GPU differs. $name: the machine; $note: how its GPU differs (in English); $adapter: its GPU.
farm-ctl-gpu-row = { $name }：{ $note }，{ $adapter }
# The same with its driver. $driver: the driver's name and version.
farm-ctl-gpu-row-driver = { $name }：{ $note }，{ $adapter }，驅動程式 { $driver }

## Render on farm: the table of machines

# Column headers (short: the table is narrow).
farm-ctl-col-machine = 電腦
farm-ctl-col-state = 狀態
farm-ctl-col-gpu = GPU
farm-ctl-col-frames = 影格
farm-ctl-col-per-frame = 每影格
# How the machine delivers its frames (a speed, or "shared drive").
farm-ctl-col-link = 連線
# When the controller last heard from the machine.
farm-ctl-col-heard = 最後回應
# In the machine's tooltip. $count: its frames that failed verification.
farm-ctl-strikes = { $count } 個影格未通過驗證
# In the Link column: it delivers its frames through the shared drive.
farm-ctl-link-share = 共用磁碟
# In the Heard column. $seconds: how long ago.
farm-ctl-heard-ago = { $seconds } 秒前
farm-ctl-readmit = 重新接納
farm-ctl-readmit-hover = 讓這台電腦在重新連線時再次加入
farm-ctl-remove = 移除
farm-ctl-remove-hover = 將這台電腦移出工作；它的影格會交給其他電腦
# The GPU column's tooltip.
farm-ctl-probe-same = 它的探測算繪與這台電腦的完全相同。
# $count: the pixels that differ.
farm-ctl-probe-differs = 它的探測算繪與這台電腦的有 { $count } 個像素不同：它的影格會與這台電腦的略有差異。
farm-ctl-probe-unknown = 它的探測算繪尚未比對。
# $driver: the driver's name and version.
farm-ctl-gpu-driver = 驅動程式 { $driver }
# After a warning sign. $note: how its GPU differs (in English: "a different GPU").
farm-ctl-gpu-note = 與這台電腦相比：{ $note }

## Render on farm: a machine's state, in the State column

# $count: its frames that failed verification.
farm-ctl-state-removed-bad = 已移除：{ $count } 個不良影格
farm-ctl-state-removed = 已由你移除
farm-ctl-state-removing = 移除中：正在收集其診斷資料
farm-ctl-state-gone = 已離開
farm-ctl-state-leaving = 正在離開
farm-ctl-state-checking = 檢查中（自我檢查）
farm-ctl-state-paused = 已被其使用者暫停
farm-ctl-state-parked = 已擱置：當機兩次，重新連線時會再加入
farm-ctl-state-refused = 已拒絕此工作（其自身的限制）
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-ctl-state-rendering = 算繪中 { $frame }（{ $index }/{ $count }）
farm-ctl-state-starting = 正在開始一個批次
farm-ctl-state-idle = 閒置
farm-ctl-state-admitted = 已接納，等待工作

## Render on farm: the buttons

# A collapsible section: the controller's messages.
farm-ctl-events = 事件
farm-ctl-resume = 繼續
farm-ctl-pause = 暫停
farm-ctl-pause-hover = 不再分派新工作；進行中的影格會完成
farm-ctl-live-paused = 這台電腦為農場算繪期間，即時畫面會暫停。
farm-ctl-close = 關閉
farm-ctl-close-active-hover = 關閉視窗；工作會繼續執行（可從「工具」選單重新開啟）
farm-ctl-close-hover = 關閉視窗
farm-ctl-copy-command = 複製命令
farm-ctl-copy-command-hover = 複製對應的命令列
farm-ctl-stop = 停止
# The Stop button once a stop has been asked for and the job has not ended after 10 seconds.
farm-ctl-stop-now = 立即停止
farm-ctl-stop-hover = 以已有的影格結束工作；再次執行同一個工作即可繼續
farm-ctl-render = 算繪
farm-ctl-render-hover = 開始監聽用戶端，並在它們上面算繪導覽

## The Render tour window (Tools ▸ Render tour…)

tour-render-title = 算繪導覽
tour-render-out = 輸出資料夾
tour-render-browse = 瀏覽…
tour-render-out-hover = 影格寫入的完整路徑。
tour-render-prefix = 影格前置詞
# <prefix> stands for the prefix typed in the field.
tour-render-prefix-hover = 影格會寫成 <prefix>_00000.png
tour-render-size = 尺寸
tour-render-size-custom = 自訂…
# The size list's choice when the size is typed in. $width and $height: pixels.
tour-render-size-custom-value = 自訂：{ $width }×{ $height }
tour-render-fps = 影格/秒
tour-render-fps-hover = 低於 1 fps 是合理的：以 0.25 fps 取樣一段長下潛，就是在不算繪數千個影格的情況下，檢查深層迴歸停留點的方法。
tour-render-ss = 超取樣
tour-render-chapter = 章節
tour-render-whole = 整個導覽
tour-render-chapter-hover = 只算繪一個章節時，會保留「全域」影格編號，因此它的影格可以直接放回完整的序列中。
tour-render-mp4 = 組合 mp4
tour-render-mp4-hover = 之後執行 ffmpeg（必須在 PATH 中）。PNG 影格會保留。
tour-render-overwrite = 覆寫現有影格
tour-render-overwrite-hover = 關閉時：算繪會停下來詢問，但在這裡無法這麼做。
tour-render-resume = 繼續被中斷的算繪
tour-render-resume-hover = 保留輸出資料夾中已有的影格，只算繪缺少的部分。最新的影格會先被檢查，如果不完整就捨棄（算繪中斷時正在寫入的就是那個影格），並往回檢查，直到找到完好的影格為止；含有不同尺寸影格的資料夾會被拒絕，而不會混入這個序列。
# $count: this machine's graphics cards.
tour-render-all-gpus = 使用所有顯示卡（{ $count }）
tour-render-all-gpus-hover = 透過這台電腦上的算繪農場（--gpus all），同時在這台電腦的每一張顯示卡上算繪。影格、其名稱、「繼續被中斷的算繪」與 mp4 都與單張顯示卡時相同；整個導覽都會算繪，但依照農場自己的順序。不同型號的顯示卡在少數像素上會略有差異：停留的鏡頭與溶接會保持在同一張顯示卡上。
tour-render-order = 算繪順序
tour-render-sequential = 依序
tour-render-sequential-hover = 依順序算繪影格 0、1、2、…（預設）。
tour-render-progressive = 漸進式
tour-render-progressive-hover = 先預覽整個導覽，再細化到完整細節：先算繪關鍵影格，然後將最大的時間間隙一再對分，直到每個影格都存在為止。無論哪種方式，影格都會放在正確的索引位置，因此「繼續被中斷的算繪」與 mp4 都照常運作：你可以提早停止、目測粗略的翻頁動畫，再以任一順序用「繼續被中斷的算繪」重新開始。
# The cost of the render. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
tour-render-summary = { $count } 個影格 · 導覽 { $seconds } 秒 · { $width }×{ $height } ss{ $ss }
# The disk space the frames will take (an estimate). $size: a size ("5.3 GB").
tour-render-disk = 影格 ≈{ $size }
# $size: the frames' estimated size; $free: the free space on that drive ("6.0 GB").
tour-render-disk-free = 影格 ≈{ $size } · 可用 { $free }
# After a warning sign. $disk: the line above ("≈51.0 GB of frames · 6.0 GB free").
tour-render-disk-short = { $disk }：空間不足以容納整個序列
tour-render-separate = 在獨立的處理程序中「從磁碟」算繪腳本，因此不會包含未儲存的編輯；而且在那裡遺失 GPU 裝置，也不會連帶影響這個工作階段。
# On the progress bar. $done: frames done; $total: frames to render.
tour-render-progress = { $done } / { $total } 個影格
# Before the render's first progress line.
tour-render-starting = 啟動中…
tour-render-live-paused = 即時畫面已暫停：算繪獨占 GPU，因此能更快完成，並準確量測它自己的影格成本。
tour-render-cancel = 取消
tour-render-cancel-hover = 關閉，不算繪
tour-render-copy-command = 複製命令
tour-render-copy-command-hover = 將對應的命令列複製到剪貼簿
tour-render-stop = 停止算繪
tour-render-render = 算繪
tour-render-render-hover = 開始算繪影格序列
# $folder: the output folder.
tour-render-finished = 算繪完成 → { $folder }
# $why: the render's own reason (in English).
tour-render-failed = 算繪失敗：{ $why }
# $last: the render's last progress line (in English), or nothing.
tour-render-failed-or-stopped = 算繪失敗或已停止。{ $last }
# $why: the system's reason.
tour-render-no-exe = 找不到執行檔：{ $why }
# $why: the system's reason.
tour-render-start-failed = 無法啟動算繪：{ $why }

## The Custom formula dialog (Fractal ▸ Custom formula…)
## The formula language itself stays as it is in every language: function names (sin, cosh, conj…),
## operators, variables (z, c, p1), and the formulas. Keep them as written inside these messages.

# The window's title.
formula-dialog-title = 自訂公式
# A small line at the top of the window, above the formula.
formula-dialog-intro = 公式的步驟，以 Fractint 風格的運算式表示
# The two ways to show and edit the formula (top right of the window).
formula-dialog-textbook = 教科書
formula-dialog-textbook-hover = 以教科書的排版方式顯示公式，並以此方式編輯
formula-dialog-text = 文字
formula-dialog-text-hover = 以文字編輯公式
# The tooltip of the "LaTeX" button (the button's own text stays LaTeX).
formula-dialog-latex-hover = 將公式依此處的排版複製為 LaTeX
# The drop-down list of example formulas (the collection).
formula-dialog-examples = 範例
# Help under the formula in Textbook mode. The characters / ^ ( are what the user types: keep them.
formula-dialog-help-textbook = 輸入方式與文字相同：/ 會將它前面的項變成分數，^ 產生指數，( 產生括號。方向鍵可在公式中移動；Tab 移到下一個空格。紅色的行無法讀取：按一下即可以文字編輯。
# Help under the formula in Text mode. "; starts a comment": the ; is what the user types.
formula-dialog-help-text = 直接輸入，或使用按鍵面板：它包含公式語言認得的每一個名稱。名稱會在你輸入時自動完成（Tab）。陳述式以換行或逗號分隔；; 開始註解。
# The syntax check, when the formula reads.
formula-dialog-reads = 讀取正確。
# The syntax check in Textbook mode, when the formula has an empty box to fill in.
formula-dialog-empty-box = 請填入空格。
# The syntax check in Textbook mode. $line: the line number; $message: what is wrong (from the formula parser, in English).
formula-dialog-error-line = 第 { $line } 行：{ $message }
# One function written with another notation's name, and the language's name for it: "ln to log".
# $from: the name as written (ln); $to: the formula language's name (log).
formula-respell-pair = { $from } 改為 { $to }
# A button that rewrites them. $changes: one or more formula-respell-pair, joined with ", ".
formula-respell = 將 { $changes }
formula-respell-hover = 將這些函數改寫為公式語言使用的名稱
# Shown when Apply finds a parameter field that is not a number. $name: the parameter (p1 … p5).
formula-param-not-numbers = { $name } 不是一對數字
# The Apply button (also in the Formula library window).
formula-apply = 套用
formula-dialog-apply-hover = 編譯公式並顯示
# Shown beside Apply while the formula is being compiled.
formula-dialog-compiling = 正在為 GPU 編譯公式…
# The greyed hint in an empty name box (also in the Formula library window).
formula-name-hint = 名稱（選填）
# The save button: "Update" when the library already has a formula of that name.
formula-dialog-update-in-library = 在公式庫中更新
formula-dialog-save-to-library = 儲存到公式庫
formula-dialog-save-hover = 將此公式及其參數保存在公式庫中
formula-dialog-save-disabled-hover = 只有讀取正確的公式才能儲存
# Opens the Formula library window.
formula-dialog-library = 公式庫…
formula-dialog-library-hover = 已儲存的公式：套用、編輯、匯入及匯出

## The Custom formula dialog's keypad
## The keys' captions are formula tokens and stay as they are; these are their tooltips.

# The keypad's third tab (the other two are "123" and "f(z)"). A short word.
formula-keypad-tab-names = 名稱
# The tabs' tooltips.
formula-keypad-tab-basic-hint = 數字、運算子與最常用的名稱
formula-keypad-tab-functions-hint = 函數
formula-keypad-tab-names-hint = 所有變數、參數與常數，以及陳述式語法
# The key z.
formula-key-z = 疊代值
# The key c.
formula-key-c = 像素（Julia 模式下為 Julia 常數）
# The key p1 on the first tab.
formula-key-p1-below = 參數 1：在下方欄位中設定
# The keys p1 … p5. $n: the parameter's number.
formula-key-param = 參數 { $n }
formula-key-multiply = 乘
formula-key-divide = 除
# The key that types ^2.
formula-key-square-after = 平方（接在值之後：z^2）
# The key that types ^.
formula-key-power = 乘冪：整數、實數或複數指數
formula-key-square-root = 平方根
# The key that types |…|: in this language it is the squared modulus, as in Fractint.
formula-key-squared-modulus = 模的「平方」|z|²：與 Fractint 相同
formula-key-add = 加
formula-key-subtract = 減（或取負）
# The key that types =.
formula-key-assign = 指派：z = …，或暫存變數 t = …
# The key that starts a new line (first tab).
formula-key-next-statement-comma = 下一個陳述式（也可以用逗號）
# The key "del".
formula-key-backspace = 刪除游標前的字元（或所選內容）
# The key e.
formula-key-euler = 歐拉數
# The key that types (0.5, 0.5).
formula-key-complex-constant = 複數常數：(實部, 虛部)
# The key that types a comma.
formula-key-comma = 分隔陳述式，或複數常數的各部分
formula-key-left = 游標左移
formula-key-right = 游標右移
# The function keys (the f(z) tab), one per function.
formula-key-sin = 正弦
formula-key-cos = 餘弦
formula-key-tan = 正切
formula-key-cotan = 餘切
formula-key-exp = 指數函數 e^z
formula-key-sinh = 雙曲正弦
formula-key-cosh = 雙曲餘弦
formula-key-tanh = 雙曲正切
formula-key-cotanh = 雙曲餘切
formula-key-log = 自然對數，ln（主分支）
formula-key-sqr = 平方：sqr(z) = z²
formula-key-sqrt = 平方根（主值）
formula-key-recip = 倒數 1/z
formula-key-conj = 共軛複數
formula-key-flip = 交換實部與虛部
formula-key-abs = 「兩個」部分都取絕對值：|x| + i|y|
formula-key-cabs = 模 |z|（實數）
formula-key-real = 實部
formula-key-imag = 虛部，以實數表示
# "fn" is Fractint's name for a function slot: keep it.
formula-key-ident = 恆等函數（Fractint 的 fn 佔位符）
# The key "pixel".
formula-key-pixel = 與 c 相同（Fractint 的名稱）
# The key "z =".
formula-key-set-z = 開始一個設定 z 的陳述式
# The key "t =".
formula-key-temporary = 暫存變數：任何新名稱都可以（t、w、a2、…）
# The key "; …".
formula-key-comment = 註解，直到行尾
# The key that starts a new line (names tab).
formula-key-next-statement = 下一個陳述式

## The formula's completion list (names offered as you type)

# Beside a name the formula itself assigns.
formula-completion-variable = 此公式所設定的變數
# Beside a function typed with another notation's name.
# $what: what the function is, in English (from the formula parser: "The natural logarithm");
# $other: the other notation's name (ln); $ours: the formula language's name (log).
formula-completion-other-spelling = { $what }：{ $other } 在這裡寫作 { $ours }
# Under a list cut short. $count: the names not shown.
formula-completion-more = … 還有 { $count } 個
# The last line of the list. Tab, Enter and Esc are key names.
formula-completion-keys = Tab 或 Enter 完成，Esc 關閉

## The Custom formula dialog's Textbook mode

# The tooltip of a line that does not read. $why: what is wrong (from the formula parser, in English, or formula-textbook-fractint-sections).
formula-textbook-unread-hover = 無法讀取：{ $why }。按一下即可以文字編輯。
# Why a line with Fractint's sections is shown as text in Textbook mode.
formula-textbook-fractint-sections = Fractint 的區段（初始化的「:」、比較、a = b = …）以文字編輯

## The Formula library window (Fractal ▸ Formula library…)

# The window's title.
formula-library-title = 公式庫
# The window's two tabs. $count: the formulas in each.
formula-library-mine = 我的公式（{ $count }）
formula-library-mine-hover = 你儲存或匯入的公式
formula-library-collection = 精選集（{ $count }）
formula-library-collection-hover = Fractadyne 隨附的公式，每個都附有起始畫面
# A small marker before the name of the formula the view shows now. A short word.
formula-library-showing = 顯示中
# The tooltip of Apply on a formula of the collection.
formula-library-apply-shipped-hover = 在其起始畫面上顯示它
# A row's Edit button (after a pencil icon).
formula-library-edit = 編輯
formula-library-edit-hover = 在「自訂公式」對話方塊中開啟它，但不套用
# A collection row's button that copies the formula into "My formulas".
formula-library-copy-to-mine = 複製到我的公式
formula-library-copy-to-mine-hover = 將它加入你的公式，以便保留或修改
formula-library-close = 關閉
# Saves the formula the view shows, under the name typed beside it.
formula-library-add-current = 加入目前的公式
formula-library-add-current-hover = 儲存畫面所顯示的公式
# "Fractal > Custom formula…" is the menu path: write it as the menus are named.
formula-library-add-current-disabled-hover = 請先套用自訂公式（碎形 > 自訂公式…）
formula-library-import = 匯入…
formula-library-import-hover = 加入公式檔中的公式；這裡的公式都不會被取代
formula-library-export-all = 全部匯出…
formula-library-export-all-hover = 將這裡的每個公式寫入一個檔案，以便分享或保存
# The filter box: its label, and the greyed hint inside it.
formula-library-filter = 篩選
formula-library-filter-hint = 依名稱或內容篩選
# Shown while the user has no saved formulas.
formula-library-empty = 還沒有已儲存的公式。你可以從「自訂公式」對話方塊儲存一個、加入畫面所顯示的公式，或匯入檔案（本程式的格式，或 Fractint 的 .frm）。
# The tooltip of Apply on one of the user's formulas: with a view saved, and without one.
formula-library-apply-view-hover = 在與此公式一起儲存的畫面上顯示它
formula-library-apply-hover = 顯示此公式
# The tooltip of a row's save icon.
formula-library-export-one-hover = 將此公式匯出到檔案
# A row asks before deleting: the question, then its two buttons. "Delete" is also the delete icon's tooltip.
formula-library-delete-ask = 要刪除嗎？
formula-library-delete = 刪除
formula-library-keep = 保留
# Under a list cut short. $count: the formulas not listed.
formula-library-more = …還有 { $count } 個：在篩選欄中輸入即可找到。
formula-library-no-match = 沒有符合篩選條件的公式。
# Under a saved formula this version cannot read. $why: what is wrong (from the formula parser, in English).
formula-library-unread = 無法在此版本中讀取：{ $why }

## Formula library notices

# $why: the system's reason, in English.
formula-library-save-failed = 無法儲存公式庫：{ $why }
formula-library-nothing-to-save = 沒有可儲存的內容：公式是空的。
# $name: the formula's name in the library.
formula-library-updated = 已更新公式庫中的「{ $name }」。
formula-library-saved = 已將「{ $name }」儲存到公式庫。
# $name: the formula's name; $why: what is wrong.
formula-library-apply-failed = 「{ $name }」無法套用：{ $why }
# Copying a formula of the collection into "My formulas". $name: its name; $as_named: the name it was given (the same, or "Name (2)" when the name was taken).
formula-library-copy-exists = 「{ $name }」已經在你的公式中。
formula-library-copied = 已將「{ $name }」複製到你的公式，名稱為「{ $as_named }」。
# $name: the formula's name.
formula-library-deleted = 已從公式庫刪除「{ $name }」。
# Why the library could not be saved (follows formula-library-save-failed).
formula-library-no-config-dir = 沒有設定目錄
# At startup, when the library file cannot be read. $why: the reason, in English; $path: a file's full path.
formula-library-unreadable-kept = 無法讀取公式庫（{ $why }）。它已保留為 { $path }，並建立了一個新的空白公式庫。
formula-library-unreadable-stuck = 無法讀取公式庫（{ $why }），也無法將它移到一旁：在儲存公式之前，請先將 { $path } 複製到安全的地方，因為儲存會取代它。

## Formula library: Import and Export
## A formula file's contents stay in English; this is what the windows and notices around them say.

# File-type names in the open and save dialogs (the extensions are added by the system).
formula-filter-formula-files = 公式檔
formula-filter-fractadyne = Fractadyne 公式
formula-filter-fractint = Fractint 公式
formula-filter-all-files = 所有檔案
# Why a file cannot be imported (they follow formula-import-failed).
formula-import-too-large = 太大了，不可能是公式檔
formula-import-not-text = 不是文字檔
formula-import-no-formulas = 其中沒有公式
# $why: the TOML reader's message, in English.
formula-file-not-formulas = 不是公式檔（{ $why }）
# $format: what the file says it is ("fractadyne-gradients").
formula-file-other-format = 這是「{ $format }」檔案，不是公式檔
# "[[formula]]" is the file's own syntax: keep it.
formula-file-no-formulas = 其中沒有公式（公式檔會以 [[formula]] 表格列出公式）
# $file: the file's name; $why: one of the reasons above, or the system's (in English).
formula-import-failed = 無法匯入「{ $file }」：{ $why }。
# After an import that added nothing. $file: the file's name.
formula-import-nothing-new = 「{ $file }」中沒有新內容。
# After an import. $count: the formulas added; $file: the file's name.
formula-import-added = 已從「{ $file }」匯入 { $count } 個公式。
# The same two, followed by what else happened. $notes: one or more of the formula-import-note-… messages below, joined with "; ".
formula-import-nothing-new-notes = 「{ $file }」中沒有新內容：{ $notes }。
formula-import-added-notes = 已從「{ $file }」匯入 { $count } 個公式：{ $notes }。
# Notes (clauses, no final full stop). $count: the formulas already in the library, skipped.
formula-import-note-duplicates = { $count } 個公式已在公式庫中
# One formula was renamed because its name was taken. $from: its name in the file; $to: the name it was given ("Name (2)").
formula-import-note-renamed-one = 「{ $from }」已被使用，因此改名為「{ $to }」
# $count: the formulas renamed because their names were taken (2 or more).
formula-import-note-renamed = { $count } 個因名稱已被使用而重新命名
# $count: the entries skipped as empty or too long.
formula-import-note-skipped = { $count } 個條目是空的或太長，已略過
# A second sentence after importing a Fractint .frm file. $count: the entries that do not read; $why: the commonest reason (from the formula parser, in English).
formula-import-frm-unread = { $count } 個條目無法在此版本中讀取（最常見的原因：{ $why }）。
# After exporting one formula. $name: the formula's name; $file: the file's name.
formula-export-one = 已將「{ $name }」匯出到「{ $file }」。
# After exporting several. $count: how many; $file: the file's name.
formula-export-many = 已將 { $count } 個公式匯出到「{ $file }」。
# $file: the file's name; $why: the system's reason, in English.
formula-export-failed = 無法寫入「{ $file }」：{ $why }

## The control panel (right-hand side): header and section headings

# The panel's title, beside the button that hides it.
panel-title = 控制項
panel-hide-hover = 隱藏控制面板
# The button that brings the hidden panel back (top right of the view).
panel-show-hover = 顯示控制面板
# Section headings.
panel-section-navigate = 瀏覽
panel-section-coloring = 著色
panel-section-quality = 品質
panel-section-effects = 效果
panel-section-overlays = 疊加顯示
panel-section-advanced = 進階
# The last section: reference notes about the current fractal. $name: the fractal's name (Mandelbrot, Burning Ship…).
panel-section-about = 關於 { $name }
panel-section-performance = 效能

## The control panel: Navigate
## Labels before a control sit in a column about 104 px wide (roughly 15 characters): a longer label pushes its control to the right.

panel-zoom-speed = 縮放速度
panel-zoom-speed-hover = 按住 Space 連續縮放的速度（1× ≈ 每 1.5 秒 2×）。
panel-snap-zoom = 縮放吸附至特徵
panel-snap-zoom-hover = 以滑鼠滾輪或 Space 放大時，將縮放拉向游標附近的迷你 Mandelbrot（或實軸），讓它滑到游標下方，縮放就會落在它上面。特徵越近，拉力越強。僅限 Mandelbrot 的單一畫面。
panel-click-zoom = 點按縮放
panel-click-zoom-hover = 開啟時，在畫面中按左鍵會以下方的倍數深入（按右鍵退回），並以按下的點為新中心。拖曳仍可平移；Shift+拖曳／右鍵拖曳仍為框選縮放。Backspace 可復原一次點按。在雙畫面中，它在任一面板上都能運作，而開啟此功能時，釘選 Julia c 改為 Ctrl+按一下。按住 Shift 會在游標下方顯示放大鏡，呈現點按後會到達的畫面；按住 Shift 時按一下即可前往該點；Shift+「拖曳」仍為框選縮放。
# The click-to-zoom factor, followed by buttons 2×, 4×, 10×…
panel-click-zoom-factor = 倍數
panel-click-zoom-snap = 吸附至最近的中心
panel-click-zoom-snap-hover = 跳躍之後，求解最近的迷你 Mandelbrot 核心並精確地停在它上面，讓你落在特徵上，而不只是附近。縮放倍率不會改變：那是點按所決定的。僅限 Mandelbrot。求解在背景執行：畫面會立刻跳過去，並在答案送達時自行修正；在極深處，這可能需要幾秒鐘。

## The control panel: Coloring
## Labels before a control sit in a column about 104 px wide (roughly 15 characters).

panel-method = 方法
panel-method-hover = 逃逸資料如何對應到色彩。條紋／三角不等式／軌道陷阱／分解會顯露軌道結構；距離則依與邊界的接近程度著色。
panel-stripe-density = 條紋密度
panel-stripe-tail = 僅尾段（深度縮放）
panel-stripe-tail-hover = 只對每條軌道的最後 N 次疊代（指數加權區間）平均條紋項，而不是整條軌道。在極深處，每條軌道除了最後幾百次疊代之外都緊跟著參考軌道，因此整條軌道的平均值在整個畫面上會是同一個顏色；只取尾段可以保留對比。比區間短的軌道不受影響。搭配 --render 時也可以使用 --stripe-tail N。
panel-tail-length = 尾段長度
# The unit after the tail length's number: short for "iterations" (the slider reads "64 it").
panel-tail-length-unit = 次
panel-tail-length-hover = 以疊代次數表示的區間長度。短區間（8–32）在任何深度都能呈現經典的粗色帶；較長的區間又會平均成一片平坦的顏色，因為長混沌軌道的條紋項會被平均掉。
panel-trap-shape = 陷阱形狀
panel-palette = 調色盤
# The two colours of the Binary palette, each beside its colour swatch.
panel-in-set = 集合內
panel-out-of-set = 集合外
# The two colours of the Duotone palette, each beside its colour swatch.
panel-shadow = 陰影
panel-highlight = 亮部
panel-cycle = 循環
panel-offset = 位移
panel-normalize-hover = 在極深處，逃逸次數橫跨數十萬，而固定的「循環」會讓調色盤在相鄰像素之間繞上數千圈，使正確的密集區域看起來像斑點雜訊。這個選項只在畫面「確實」發生混疊時，將調色盤重新對應到畫面量測到的逃逸範圍（此時「循環」設定的是這個範圍內調色盤掃過的次數）；平滑的畫面則維持原樣，因為重新對應只會白白改變圖像。勾選下方的「一律貼合範圍」即可無條件重新對應。僅適用於平滑方法；與 --normalize 相同。
# A checkbox indented under "Normalize deep colors" (also in the Color menu).
panel-fit-range = 一律貼合範圍
panel-fit-range-hover =
    即使畫面沒有混疊，也將調色盤重新對應到畫面量測到的逃逸範圍；上方的核取方塊只有在密集區域原本會看起來像斑點雜訊時，才會重新對應。

    當你希望調色盤涵蓋螢幕上的內容，而不是為了修正雜訊時，請使用這個選項。它仍然需要量測，因此在畫面穩定下來之前的一兩刻不會有作用，在完全沒有逃逸像素的畫面上也完全沒有作用。
# A checkbox indented under "Always fit range" (also in the Color menu).
panel-log-scale = 對數色階
panel-log-scale-hover =
    依逃逸值的「對數」而非線性地分配調色盤。在深處，逃逸次數會集中在高端，因此線性對應會把大部分調色盤花在邊界附近的一層薄殼上，並讓其他部分變得平坦。

    這適用於正規化對應，也就是「正規化深層色彩」（或匯出時的 --normalize）所開啟的功能。範圍嚴重偏斜時，無論是否勾選此項，都會採用對數對應。
# The same checkbox's tooltip while it is greyed out (normalization is off).
panel-log-scale-off-hover =
    需要正規化：這個選項決定調色盤「如何」對應到量測到的逃逸範圍，因此在「正規化深層色彩」關閉時（且沒有使用 --normalize 匯出），就沒有可以讓它改變的對應，也不會改變任何一個像素。

    請開啟「正規化深層色彩」來使用它。
# The palette animation: its kind, then its speed.
panel-animate = 動畫
panel-anim-speed = 速度
panel-anim-speed-hover = 循環速度：每秒的色彩位移循環次數，或（隨機漸層）每秒的漸層變化次數。
panel-shuffle-gradient = 重新產生漸層

## The control panel: Quality

panel-auto-iter = 疊代次數隨縮放自動調整
# The iteration-count label while auto-scaling is on (the count is then a base the zoom raises).
panel-iterations-base = 疊代次數（基準）
panel-iterations = 疊代次數
panel-iterations-hover = 基準疊代次數。開啟自動調整時，實際的次數會隨縮放深度攀升。移動時，預覽會將疊代次數限制在較低的值（50,000）以保持流暢，畫面穩定下來後再銳利化到完整的次數；因此深層邊緣在移動時看起來平滑，停下時才會解析出來。
# A warning under the iteration count. $count: the iterations the settled preview renders.
panel-res-limited = 在這個深度，穩定後的預覽會以較低的解析度算繪 { $count } 次疊代，以保持流暢。若要完整解析度的細節，請匯出。
panel-anti-alias = 反鋸齒
# Anti-aliasing off (the other choices are 2×, 3×, 4×, 8×).
panel-anti-alias-off = 關閉
panel-anti-alias-hover = 靜態影像的超取樣（在畫面穩定下來後套用）。越高越能抑制外部細微的「塵點」，但會增加算繪時間。

## The control panel: Effects

panel-relief-lighting = 3D 浮雕光影
panel-relief-lighting-hover = 利用距離估計的斜率為表面著色，呈現浮雕、受光的效果。（全純族：Mandelbrot / Multibrot。）
panel-light-angle = 光線角度
panel-relief = 浮雕
panel-relief-hover = 越低浮雕越銳利；越高越柔和／平坦。
panel-rotate-light = 旋轉光源
panel-rotate-light-hover = 讓光線方向隨時間旋轉（使用「速度」滑桿）。
panel-distance-glow = 距離光暈
panel-distance-glow-hover = 明亮的距離估計等高色帶，在邊界附近變密，成為發光的細絲。（全純族。）
panel-glow = 光暈
panel-band-width = 條帶寬度
panel-band-width-hover = 距離等高線的間距（每條帶的八度數）。
panel-animate-glow = 光暈動畫
panel-animate-glow-hover = 讓光暈色帶隨時間流動（使用「速度」滑桿）。

## The control panel: Overlays

panel-orbits-hover = 繪製游標下那一點的疊代路徑。
panel-orbit-anim = 動畫（飛馳的點）
panel-orbit-anim-hover = 讓一個色彩循環的點沿著軌道飛馳而出。
panel-orbit-speed = 軌道速度
panel-minimap-hover = 小型的初始畫面總覽，附有「你在這裡」標記與縮放深度。按一下即可跳到該區域。

## The control panel: Advanced
## Labels before a control sit in a column about 104 px wide; most of these are already wider in English.

# Small headings of the section's two groups.
panel-accelerators = 加速功能
panel-performance-tuning = 效能調校
panel-bla = BLA 加速（深度縮放）
panel-bla-hover = 雙線性近似：在極深處（floatexp Mandelbrot，≥1e28×）跳過整條軌道中的疊代：GPU 算繪快約 5 倍，輸出完全相同（經自我測試驗證）。預設為開啟；若要比較或遇到瑕疵，可以關閉。
panel-series-approx = 級數近似
panel-series-approx-hover = 以多項式為微擾提供初始值，以略過早期疊代，用於 BLA 沒有作用的地方（df32 深度、Multibrot、BLA 關閉時）。輸出完全相同；若要比較可以關閉。
panel-glitch-correct = 瑕疵修正（匯出）
panel-glitch-correct-hover = 匯出影像時的多參考點瑕疵修正：偵測微擾瑕疵，並以額外的參考點重新算繪這些像素，直到乾淨為止。預設為開啟。適用於最多約 32 MP 的匯出（非輔助著色方法）；更大的影像與即時畫面使用一般路徑。絕不用於 Mandelbrot 或 Multibrot 畫面（Julia 模式以外）：與精確計算比對後，它在那裡什麼也沒有修正，卻讓匯出時間最多增加到 5 倍。
panel-dive-limit = 自動縮放深度上限
panel-dive-limit-hover = 自動縮放（A 鍵）停止的深度。大約 1e271× 以內會平滑滑行；更深時，它會改用斷續的分段下潛，以快速到達極深處。
# Followed by two choices, Speed and Quality.
panel-autopilot-priority = 自動縮放優先
panel-priority-speed-hover = 以設定的速度縮放。只要有含細節的影格準備好，螢幕就會更新，並採用能讓更新持續進行的任何解析度（如果縮放跑得比算繪快，會低於「移動時最低解析度」的下限）；在兩次更新之間，上一個影格會跟著縮放並變得模糊。
panel-priority-quality-hover = 每次更新都是完全解析的全解析度影格，而且縮放會視需要放慢，讓螢幕上的影格在下一個影格到來之前不會放大太多。下潛較慢，但每一刻都很清晰。
# Followed by two choices, Detail and Misiurewicz point.
panel-autopilot-target = 自動縮放目標
panel-target-detail-hover = 朝畫面中碎形邊緣上最豐富的結構下潛，並隨下潛重新選擇目標。
panel-target-misiurewicz-hover = 找出最近的 Misiurewicz 點（結構永遠重複的螺旋中心），並精確地下潛進去。下潛會先保持靜止，同時將該點求解到下潛上限的深度：在一般深度只需片刻，超過 1e1000× 則較久。僅限 Mandelbrot 集合。
panel-work-budget = 即時算繪預算
panel-work-budget-hover = 深度縮放時，即時畫面在細節與速度之間的取捨。越高，算繪的解析度越完整（越清晰，較少「柔化」的放大感），但影格速率會降低，而且非常高的值可能在最吃重的影格上造成短暫的 GPU 停頓。不論此設定為何，匯出一律為完整解析度。
panel-min-motion-res = 移動時最低解析度
panel-min-motion-res-hover = 深度縮放時，連續縮放或平移期間即時畫面可以降到的最低解析度。越高越能避免快速下潛時出現方塊／像素化（較清晰），但會降低影格速率：在 100% 時，深度下潛的細節更新必須以完整成本算繪，可能讓深度縮放明顯地頓挫／卡頓。如果深度下潛感覺不順，請調低此值。預設為 30%；不影響穩定後的影像或匯出。
panel-finish-sound = 算繪完成時播放音效
panel-finish-sound-hover = 匯出或導覽算繪完成時，播放 FRACTINT 的完成曲：三個上升的音（1047、1109、1175 Hz，各 100 ms），完全依照 DOS 原版的編碼，並以限頻方波合成，再經高通濾波，聽起來就像當年播放它的小小 PC 喇叭。
panel-prefer-detail = 縮放時優先顯示細節
panel-prefer-detail-hover = 縮放或平移時，持續顯示上一個完整細節的影格（經縮放並追蹤以跟隨移動），而不是每個影格都以較低品質重新算繪。移動暫停時，畫面會完整算繪。關閉時：移動中會以較低解析度即時算繪（較流暢、較粗糙）。淺層畫面（直接模式）一律即時算繪：無論如何，它們每個影格都便宜又清晰。

## The control panel: About (the current fractal)

# A link to the fractal's reference page, followed by an arrow.
panel-about-reference = 參考資料

## The control panel: Advanced ▸ Second graphics card

# A label before a drop-down (wider than the panel's label column already).
gpu2-label = 第二張顯示卡
gpu2-off = 關閉
# Another device on the same graphics card as the window (the default).
gpu2-same-card = 同一張顯示卡
# A card the list does not name. $number: the card's number.
gpu2-card-n = 顯示卡 { $number }
gpu2-finding = 正在尋找這台電腦的顯示卡…
gpu2-none = 這台電腦沒有其他顯示卡。
gpu2-hover =
    另一個圖形裝置會在深層畫面移動時算繪它的影格，並在停下後算繪部分讓畫面變清晰的額外樣本，因此圖像能跟上縮放，並更快達到最終品質。繪製視窗的顯示卡仍會繪製所有內容。

    另一張顯示卡會帶來它自己的速度。兩種不同型號的顯示卡在少數像素上的繪製會略有差異；如果第二張卡失敗，畫面會繼續以單張卡運作。

    「同一張顯示卡」（預設）會在這張顯示卡上開啟第二個裝置。它不會增加速度，但算繪時不必等待螢幕的影格。它與視窗共用這張顯示卡：導致顯示卡重設的失敗會結束程式，就像視窗本身的算繪失敗一樣，而下次啟動時會詢問是否要關閉它。

## The view (canvas)

# A notice after Ctrl+click pins the Julia c at a deep zoom.
canvas-julia-pin-quantized = 已釘選 Julia c，但它以雙精度保存，因此在這個深度，整個面板都會四捨五入成同一個值，釘選會落在畫面中心。請縮小後再瞄準。
# A small caption in the top-left corner of the view while the orbit overlay is fitted to the view (its "Normalize (fit to view)" option).
canvas-orbit-normalized = 軌道（正規化）
# The minimap's tooltip.
canvas-minimap-hover = 總覽／你在這裡。拖曳可在目前縮放下平移，捲動可放大與縮小，按一下即可跳到該區域（初始縮放）。

## The timestamp overlay (View ▸ Show timestamp)

# Under the large elapsed-time clock. $frame: the frame number (as the log writes it, without grouping).
overlay-timestamp-frame = 影格 { $frame }

## File ▸ Settings ▸ Reference cache…

cache-title = 參考軌道快取
cache-intro = 在極深處，畫面最慢的部分是它的參考軌道：最多長達一小時的任意精度算術。Fractadyne 會將完成的軌道保存在這裡，因此回到你造訪過的位置，或縮放到它附近的某處，只需幾秒鐘。一條軌道約 4 MB。
cache-off = 這次執行已關閉（以工作任務啟動，或使用 --no-orbit-cache）。在下次一般啟動之前，不會讀取或寫入任何內容。
# Small headings of the window's parts.
cache-location = 位置
cache-usage = 使用量
cache-limit = 上限
cache-no-config = （沒有可用的設定目錄）
cache-open-folder = 開啟資料夾
# The text on the usage bar. $used and $limit: sizes such as "1.2 GB"; $count: the orbits kept.
cache-usage-bar = { $used } / { $limit }：{ $count } 條軌道
cache-limit-hover = 快取超過這個大小時，會先移除建構成本「最低」的軌道：花了一小時建構的深層軌道，會比任何數量的快速軌道保留得更久，無論那些軌道最近是否用過。
# Beside the limit. $count: about how many orbits of the live view's length fit in it.
cache-limit-estimate = ≈ { $count } 條即時上限長度的軌道
cache-kept = 已保存（最有價值的在前）
# Column headers of the list of kept orbits.
cache-col-iterations = 疊代次數
cache-col-precision = 精度
cache-col-size = 大小
cache-col-last-used = 上次使用
# The precision column. $bits: the precision in bits.
cache-row-bits = { $bits } 位元
# Under the list. $count: the orbits not listed.
cache-rows-more = … 還有 { $count } 條
# When an orbit was last used (the "last used" column).
cache-ago-now = 剛剛
# $count: minutes.
cache-ago-minutes = { $count } 分鐘前
# $count: hours.
cache-ago-hours = { $count } 小時前
# $count: days.
cache-ago-days = { $count } 天前
# Asked before clearing. $count: the orbits; $size: their total size, such as "1.2 GB".
cache-confirm = 要刪除 { $count } 條軌道（{ $size }）嗎？重建它們只花時間，不會遺失資料。
cache-delete = 刪除
cache-cancel = 取消
cache-clear = 清除快取…
cache-clear-hover = 刪除所有已儲存的軌道。會先詢問。
cache-close = 關閉
# $count: the orbits removed.
cache-cleared = 已清除參考軌道快取：移除了 { $count } 條軌道。
# $why: the system's error message.
cache-clear-failed = 無法清除參考軌道快取：{ $why }

## Help ▸ Diagnostics…

diag-title = 診斷
diag-intro = 執行測試，檢查 Fractadyne 在你的硬體上是否正常運作。結果會儲存在你的電腦上，並可附加到問題回報中；不會自行傳送到任何地方。
# $status: which arbitrary-precision library is in use, as the program reports it (in English).
diag-arithmetic = 深度縮放算術：{ $status }
# The three tests' buttons, each followed by a sentence saying what it does.
diag-run-selftest = 執行自我測試
diag-run-uitest = 執行介面測試
diag-run-gputest = 執行 GPU 算術檢查
diag-blurb-selftest = 在你的 GPU 上，將數學運算與算繪結果與已知正確的結果比對。大約需要 15 秒。
diag-blurb-uitest = 走過介面與即時畫面，在數個縮放深度下擷取螢幕截圖。需要一兩分鐘，執行期間會開啟視窗。
diag-blurb-gputest = 在你的電腦提供的每一種圖形後端上，將著色器的雙浮點數算術與已知正確的值比對。只需幾秒鐘，不會開啟視窗。如果它回報失敗，那是我們想看到的真實發現：請將它附加到回報中。
# While a test runs. $count: the checks finished so far.
diag-checks-done = 已完成 { $count } 項檢查
# A finished test's result. $test: the test's name (Self-test, UI test, GPU arithmetic check).
diag-verdict-captured = { $test }：已擷取結果
diag-verdict-passed = { $test }：通過
diag-verdict-problems = { $test }：回報了問題
# Under the GPU arithmetic check's result.
diag-informational-note = 這項測試沒有通過或失敗之分：它記錄的是你的 GPU 的計算方式，回報的失敗是真實的發現，而不是你電腦的故障。不論如何，將它附加到回報中，是你能拿它做的最有用的事。
# Under a test's result when it reported problems.
diag-problems-note = 在參考 GPU 以外的硬體上，出現一些差異是預期中的，尤其是影像比對。將它附加到問題回報中，是你能拿它做的最有用的事。
diag-open-results = 開啟結果
diag-attach = 附加到問題回報…
diag-attach-hover = 開啟回報對話方塊並附上這個結果，讓 issue 帶有經機器驗證的測試，而不只是描述
diag-console = 將診斷資訊輸出到主控台
diag-console-hover = 從現在起在 stderr 上顯示 [fd-…] 行。不帶參數啟動 Fractadyne 時，預設為關閉。這完全不會改變「記錄」的內容（記錄檔一律會收到每一行），因此除非你正在看終端機，否則請保持關閉。僅在這個工作階段有效；若要連啟動過程也擷取，請使用 --console 旗標或 FRACTADYNE_CONSOLE=1。
diag-close = 關閉
# Errors starting a test. $why: the system's error message; $dir: a folder.
diag-error-no-exe = 找不到執行檔：{ $why }
diag-error-create-dir = 無法建立 { $dir }：{ $why }
diag-error-start = 無法啟動測試：{ $why }

## Navigate ▸ Misiurewicz explorer…

misi-title = Misiurewicz 探索器
misi-mandelbrot-only = Misiurewicz 瀏覽僅適用於 Mandelbrot；請切換碎形後再使用。
# While the gallery is being built. $seconds: the time so far, such as "3".
misi-solving = 正在求解小型類型的族（{ $seconds } 秒）…
# Above the gallery. $total: all points; $curated: the hand-picked ones; $found: the ones the sweep found.
misi-summary = { $total } 個點：{ $curated } 個精選，{ $found } 個由小型類型的帶種子牛頓掃描找到（上半平面；粗略的網格能找到大多數吸引盆，但無法證明找到全部）。
# The name of a point the sweep found. $kind: its type, such as "(3,1)"; $point: its coordinate, such as "-0.1011+0.9563i".
misi-entry-name = { $kind }，位於 { $point }
# Under each thumbnail. $kind: the point's type, such as "(3,1)"; $octaves: the zoom period in octaves, such as "3.17"; $twist: the twist per repeat in degrees, such as "+12.5". "oct" is short for octaves.
misi-entry-readout = { $kind } · { $octaves } 八度/次 · { $twist }°
# Before the depth field.
misi-zoom-to = 縮放至
misi-cancel = 取消
misi-solve-jump = 求解並前往
misi-solve-jump-hover = 以牛頓法將這個精確的點求解到所要求深度需要的精度（以它儲存的座標作為初始值），然後直接跳過去。要求越深，所需時間越長：求解採用任意精度。
misi-select-hint = 選擇一個點、指定深度，然後前往：每張縮圖下方的乘數行，就是在那裡下潛時的樣子：畫面重複之前要經過幾個八度，以及每次重複扭轉多少。
misi-cancelled = 已取消求解。
# $text: what was typed in the depth field.
misi-bad-zoom = 無法將「{ $text }」讀取為放大倍率（請寫成 1e50、3.2e120、…）。
misi-no-kp = 此條目沒有偵測到可供求解的 (k,p)。
misi-parse-failed = 無法剖析儲存的座標。
# A notice after the jump. $name: the point's name; $zoom: the magnification, such as "1e50"; $seconds: how long the solve took, such as "1.2".
misi-zoomed = 已縮放至 { $name }：{ $zoom }×（{ $seconds } 秒）
# Added after the notice above when the iteration count is too low. $have: the fixed iteration count; $typical: what the depth typically needs.
misi-iter-note = 「疊代次數」固定為 { $have }；這個深度通常需要約 { $typical }。如果畫面算繪成一片平坦，請調高它（或開啟自動疊代）。
# $name: the point's name; $why: why the solve failed (a technical word).
misi-solve-missed = 深層求解沒有落在 { $name } 上：{ $why }。儲存的點在它自己的深度仍然是精確的，請嘗試較淺的目標。

## Life (Game of Life): the side panel

# Shown when the graphics card cannot run the Life view.
life-no-compute = 這個顯示介面卡無法執行生命遊戲（沒有計算著色器）。
# Row label: the pattern list (pattern names stay in English).
life-pattern = 圖樣
life-open = 開啟…
life-open-hover = 開啟 RLE、.cells 或 Life 1.05/1.06 圖樣檔。
life-pattern-text = 圖樣文字…
life-pattern-text-hover = 貼上圖樣（LifeWiki 提供的 RLE 格式，或純文字）。
life-save = 儲存…
life-save-hover = 將宇宙目前的狀態存成 RLE。
# Row label: the rule list and the typed rule (B3/S23 notation).
life-rule = 規則
# Shown in the rule list when the rule in use is none of the library's.
life-rule-custom = 自訂
life-apply = 套用
# Playback buttons (a pause or play symbol is shown before them).
life-pause = 暫停
life-play = 播放
life-play-hover = 執行宇宙（Enter）。
life-step = 單步
life-step-hover = 前進一個世代（N）。
# Tooltip of the button beside Step, which reads "+16", "+1,024"…: that many generations.
life-stride-hover = 前進這麼多個世代（Shift+N）。
life-reset = 重設
life-reset-hover = 回到開啟時的圖樣（第 0 代）。
life-speed = 速度
# After the speed slider's number: generations a second. Short.
life-speed-unit = 代/秒
# Row label: how many generations the "+n" button advances.
life-stride = 步幅
# The panel's numbers.
life-generation = 世代
life-population = 細胞數
life-tiles = 圖塊
# $count: the tiles stored; $capacity: how many the GPU pool holds.
life-tiles-of = { $count } / { $capacity }
life-tiles-hover = 已儲存的 64×64 細胞圖塊（每個 4 KiB）與 GPU 池的容量。
# Row label: what a left-drag in the view does (Pan or Draw).
life-mouse = 滑鼠
life-pan = 平移
life-pan-hover = 拖曳可移動畫面。
life-draw = 繪製
# The Draw tool's tooltip; the toolbar's shows it after "Draw (D):", so it starts lowercase.
life-draw-hint = 按一下細胞可翻轉它；拖曳可塗繪：從死細胞開始的筆畫會繪製，從活細胞開始的筆畫會擦除。執行中的宇宙會在你繪製時暫停，放開後繼續執行。
life-undo = 復原
life-undo-hover = 回到上一次編輯、填滿、清除或規則變更之前的宇宙（繪製時按 Ctrl+Z）。
life-grid = 格線
life-grid-hover = 放大到每個細胞 8 像素以上時，顯示細胞邊框。
# A random fill: "Random" labels the density field, "seed" the seed field after it.
life-random = 隨機
life-density-hover = 填滿時設為存活的細胞比例。
life-seed = 種子
life-fill-view = 填滿畫面
# $cells: the largest side of the filled area, in cells (2,048).
life-fill-view-hover = 以隨機方式填滿可見區域（以其中心為準，最多 { $cells } × { $cells } 個細胞）。
life-clear = 清除
life-clear-hover = 殺死所有細胞（「復原」可以讓它們回來）。

## Life: notices in the panel, and file dialogs

# $path: the file written.
life-saved = 已儲存 { $path }
# $error: why (from the system, in English).
life-save-failed = 儲存失敗：{ $error }
# The universe stopped running. $error: why (in English).
life-stopped = 已停止：{ $error }
# A pattern file named a rule that does not read. $error: why (in English); $rule: the rule kept, in B3/S23 notation.
life-rule-not-understood = 無法理解圖樣的規則（{ $error }）；保留 { $rule }
# $path: the file; $error: why (in English).
life-open-failed = 無法開啟 { $path }：{ $error }
# File-dialog filter names.
life-filter-patterns = 生命遊戲圖樣
life-filter-rle = 行程長度編碼圖樣

## Life: the toolbar (its buttons are icons; these are their tooltips)

life-toolbar-pause-hover = 暫停宇宙（Enter）
life-toolbar-play-hover = 執行宇宙（Enter）
# $stride: the generations a Shift+click steps.
life-toolbar-step-hover = 前進一個世代（N）；Shift+按一下或 Shift+N 前進 { $stride } 個世代
life-toolbar-reset-hover = 回到開啟時的圖樣（第 0 代）
# $hint: the Draw tool's tooltip (life-draw-hint).
life-toolbar-draw-hover = 繪製（D）：{ $hint }

## Life: the Pattern text window

life-text-title = 圖樣文字
life-text-intro = 貼上圖樣：RLE（LifeWiki 與 Golly 提供的格式）、純文字 .cells，或 Life 1.05 / 1.06。
life-text-load = 載入
life-text-current = 目前內容（RLE）
life-text-current-hover = 以上次載入的宇宙填入方塊。

## L-systems: the side panel

# Row label: the system list (system names stay in English).
lsys-system = 系統
lsys-open = 開啟…
lsys-open-hover = 開啟 Fractint .l 檔案或 .lsys 系統。
lsys-edit = 編輯…
lsys-edit-hover = 系統的文字：公理、產生式、角度；編輯後套用。
lsys-save = 儲存…
lsys-save-hover = 將系統存成 .lsys 文字檔。
# Tooltip of the "SVG…" button.
lsys-svg-hover = 將畫面匯出為 SVG 繪圖：線條與形狀皆為向量，並保留畫面的色彩。
lsys-follow = 階數隨縮放變化
lsys-follow-hover = 放大時階數會提高，因此細節永遠不會用盡。
lsys-follow-fixed-hover = 這個系統不會每一階都以固定倍數成長，因此以固定階數繪製。
lsys-order = 階數
lsys-angle = 角度
lsys-sweep-hover = 來回掃動角度：曲線會隨著旋轉而變形
# A small button that puts back the system's own angle after the slider moved it.
lsys-system-angle = 系統的角度
lsys-seed = 種子
lsys-seed-hover = 隨機選擇所依據的值：相同的種子，就是相同的植物。
lsys-seed-new = 換新
lsys-seed-new-hover = 另一株植物：新的種子。
lsys-line-width = 線寬
# Row label of a percentage slider: how much of the curve is drawn.
lsys-draw-on = 繪製進度
lsys-draw-on-hover = 依海龜繪製的順序，曲線已繪製了多少
lsys-draw-on-play-hover = 曲線自行繪製出來
lsys-colour-by = 著色依據
# The colourings, in the "Colour by" list.
lsys-colour-position = 沿曲線的位置
lsys-colour-depth = 分支深度
lsys-colour-heading = 方向
# C, < and > are the system's commands that set the colour index: keep them.
lsys-colour-index = 色彩索引（C、<、>）
lsys-colour-plain = 單一色彩
# The panel's numbers.
lsys-growth = 成長
lsys-growth-built = 以字串逐代建構（依其階數）
lsys-growth-built-hover = 參數化或上下文相關：模組的改寫取決於它的數值或鄰居，因此字串是一代一代建構的（最多兩百萬個模組），並以雙精度繪製；階數不會隨縮放變化。
# How much the picture grows from one order to the next. $factor: the factor (1.4142).
lsys-growth-factor = 每階 ×{ $factor }
lsys-growth-factor-period-2 = 每階 ×{ $factor }（階數以 2 為步進）
lsys-growth-none = 無（固定階數）
lsys-drawn = 已繪製
# $drawn: what was drawn (lsys-drawn-segments, lsys-drawn-shapes or lsys-drawn-segments-shapes); $order: the order.
lsys-drawn-at-order = { $drawn }，階數 { $order }
# The same, when the walk stopped at its segment budget.
lsys-drawn-at-order-budget = { $drawn }，階數 { $order }（達到預算）
# $segments: the line segments drawn.
lsys-drawn-segments = { $segments } 條線段
# $shapes: the filled shapes drawn.
lsys-drawn-shapes = { $shapes } 個填色形狀
# Both counts. $segments: line segments; $shapes: filled shapes.
lsys-drawn-segments-shapes = { $segments } 條線段，{ $shapes } 個填色形狀
# $ms: the walk's time in milliseconds (12.3).
lsys-drawn-hover = 走訪耗時 { $ms } ms；畫面外的子樹會略過，小於一個像素的子樹畫成一條線段
# A word too long to build at the order asked for. $wanted: the order asked for; $built: the order drawn.
lsys-drawn-short = 階數 { $wanted } 的字串超過兩百萬個模組：改以階數 { $built } 繪製。
# $order: the deepest order the app draws.
lsys-deepest-order = 已達繪製的最深階數（{ $order }）：繼續放大不會增加細節。

## L-systems: notices in the panel and the editor, and file dialogs

# $path: the file; $error: why (in English).
lsys-open-failed = 無法開啟 { $path }：{ $error }
# Why a file was not opened (shown after "Could not open …:").
lsys-file-too-large = 大於 4 MiB
# Why a Fractint .l file was not opened: it holds no entries. `Name { … }` is the file's syntax: keep it.
lsys-no-entries = 沒有條目（.l 檔案包含 `Name {"{"} … {"}"}` 條目）
# $path: the file written.
lsys-saved = 已儲存 { $path }
# $error: why (from the system, in English).
lsys-save-failed = 儲存失敗：{ $error }
# After File ▸ Export SVG. $path: the file written.
lsys-svg-exported = 已匯出 { $path }
# $error: why (from the system, in English).
lsys-svg-failed = 匯出失敗：{ $error }
# File-dialog filter names: opening (any system file) and saving (one system).
lsys-filter-systems = L 系統
lsys-filter-system = L 系統

## L-systems: the toolbar

lsys-toolbar-lower-hover = 降低階數（並固定）
lsys-toolbar-raise-hover = 提高階數（並固定）
# Between the − and + buttons, in a fixed-width font. $order: the order, padded to 3 characters so the label keeps its width.
lsys-toolbar-order = 階數 { $order }
# A toggle: the order follows the zoom. Short.
lsys-toolbar-auto = 自動
lsys-toolbar-auto-hover = 階數隨縮放變化
lsys-toolbar-edit-hover = 編輯系統的文字

## L-systems: the editor window, and a .l file's entries

lsys-editor-title = L 系統
# The editor's help. The quoted examples ('X = word', 'seed n', 'define R 1.456'…), the commands and "The Algorithmic Beauty of Plants" (a book's title) are the system's syntax and names: keep them as they are.
lsys-editor-help = 每行一個鍵：angle（度數，或以 /n 表示圓的 n 等分）、axiom、產生式 'X = word'，以及選用的 heading、draw / move / variables、colour、order。有多個隨機替代項的符號，每個替代項各占一行，並附上權重：'X (0.3) = word'；'seed n' 選擇植物。在字串中，{"{"} {"}"} 會填滿海龜的路徑，. 會加入一個頂點。參數化與上下文相關的產生式（The Algorithmic Beauty of Plants）：'A(s) : s > 1 = F(s)[+A(s/2)]'、'b < a > c = b'，搭配 'define R 1.456' 與 'ignore +-F'；F(l) 前進 l，+(a) 轉 a 度。Fractint 的 .l 條目可以直接貼上。
lsys-editor-apply = 套用
lsys-editor-revert = 還原
lsys-editor-revert-hover = 顯示中系統的文字。
# The window listing a .l file's entries to pick from. $file: the file's name.
lsys-entries-title = { $file }：條目
# Tooltip of an entry that does not parse (its error is shown beside it, in English).
lsys-entry-unreadable-hover = 此條目無法讀取。

## Export: opening a view or location (File ▸ Open view or location…)

# File-dialog filter names.
export-filter-view = Fractadyne 畫面或位置
export-filter-image = 影像（PNG / EXR）
export-filter-location = 位置（.fdn / .kfr）
# $zoom: the magnification (1.2e148), followed by the magnification sign ×.
export-loaded-location = 已載入位置 @ { $zoom }×
# $zoom: the magnification; $note: what was noteworthy in the file (in English).
export-loaded-location-note = 已載入 @ { $zoom }×：{ $note }
# $path: the file.
export-not-location = { $path } 不是 Fractadyne 位置。
export-file-too-large = 檔案太大（不是 .fdn 位置？）。
# $path: the file; $error: why (from the system, in English).
export-read-failed = 無法讀取 { $path }：{ $error }
# A Kalles Fraktaler .kfr file did not import. $error: why (in English).
export-import-failed = 匯入失敗：{ $error }
# $path: an image whose embedded text is not a view.
export-not-view = { $path } 有 Fractadyne 中繼資料，但不是畫面。
# $path: the image the view was read from.
export-loaded-view = 已從 { $path } 載入畫面
# $path: the image; $note: what was noteworthy in it (in English).
export-loaded-view-note = 已從 { $path } 載入畫面：{ $note }
export-no-metadata = 該檔案沒有嵌入的 Fractadyne 畫面中繼資料。

## Export: notices after an export or a snapshot
## "File → Export image…" in them names the menu items (menu-file, menu-file-export).

# The export window's status while the image renders.
export-rendering = 算繪中…
export-canceled = 已取消匯出。
# $error: why (in English).
export-failed = 匯出失敗：{ $error }
# $size: the image's size in pixels (1920×1080); $path: the file written.
export-saved = 已儲存 { $size } → { $path }
# An L-system export whose walk stopped at its budget. $size: pixels; $segments: the budget (4,000,000); $path: the file.
export-saved-incomplete = 已儲存 { $size }（不完整：超過 { $segments } 條線段的預算，請降低階數）→ { $path }
# A two-panel export written as two files. $path: the first of them.
export-saved-two = 已儲存 2 個檔案 → { $path }
# $size: pixels; $path: the file written.
export-saved-corrected = 已儲存 { $size }（已修正瑕疵）→ { $path }
# $path: the first of the two files.
export-saved-two-corrected = 已儲存 2 個檔案（已修正瑕疵）→ { $path }
# Added after a finished export's notice. $duration: how long it took (8.3s, 1m 04.0s).
export-elapsed = （耗時 { $duration }）
export-preparing-deep = 正在準備深層匯出：建構參考軌道中（可能需要一段時間）…
# $size: pixels (5120×4035); $ss: the supersampling factor; $iterations: the iteration count.
export-large = 大型匯出（{ $size }、{ $ss }× 超取樣、{ $iterations } 次疊代）：在背景算繪中；這種大小的匯出會略過瑕疵修正…
export-life-unavailable = 目前還無法將生命遊戲畫面匯出為影像：請使用螢幕截圖，或「檔案 ▸ 儲存生命遊戲圖樣」。
export-already-running = 已有匯出正在執行：「檔案 → 匯出影像…」會顯示其進度。
# $size: the image's size in pixels.
export-snapshot-background = 快照：正在背景算繪 { $size }；「檔案 → 匯出影像…」會顯示進度，也可以取消。
export-no-gpu = 無法使用 GPU
# A screen-capture snapshot. $size: pixels; $path: the file written.
export-snapshot-saved = 已儲存螢幕快照 { $size } → { $path }
# $error: why (in English).
export-snapshot-failed = 快照失敗：{ $error }

## The colour picker (the popup of a colour swatch)

# The third row's label: the colour as hexadecimal bytes. Short (about 6 characters).
picker-row-hex = 色碼
picker-hex-hover = 按一下即可輸入十六進位值
picker-cancel = 取消
picker-cancel-hover = 恢復此選色器開啟時的色彩
picker-ok = 確定
picker-ok-hover = 保留此色彩
# Starts the screen eyedropper (an icon is shown before it).
picker-pick = 取色
picker-pick-hover = 從螢幕上任何地方取用色彩
# Tooltip of the disabled Pick button (in the picker and the gradient editor) on Linux and macOS.
picker-eyedropper-unsupported = 從螢幕上其他地方取色，需要平台的螢幕擷取 API。只有 Windows 版有這個 API；在 Linux 上這需要 X11 相依套件，而 Wayland 則需要具有自己權限提示的 portal。

## The fractal families: what each one is (the info panel's "About" section and the pickers' hover text).
## The formulas themselves (z -> z^2 + c) are written in code and stay as they are.

fractal-mandelbrot-about = 典型的逃逸時間碎形：使 0 的軌道保持有界的所有 c 所構成的集合。它的邊界無限複雜。
fractal-multibrot3-about = Multibrot 集合：以更高次方進行的 Mandelbrot 構造。次方 d 帶來 (d-1) 重旋轉對稱。
fractal-multibrot4-about = 4 次方的 Multibrot：三重對稱，寬大的圓盤。
fractal-multibrot5-about = 5 次方的 Multibrot：四重對稱。
fractal-tricorn-about = Tricorn（Mandelbar）：每一步都對 z 取共軛。這個反全純映射會產生三個角的形狀。
fractal-burning-ship-about = 平方之前先對 z 的各部分取絕對值；結果看起來像一艘著火的船。
fractal-celtic-about = Burning Ship 的近親，只對 z^2 的實部取絕對值，產生凱爾特結／心形的圖案。
fractal-buffalo-about = 一種絕對值變體，對 z^2 的兩個分量都取絕對值。
fractal-phoenix-about = Phoenix 也會用到前一次的疊代值，產生火焰般的細絲。可以透過 Julia 模式試試它的 Julia 形式。
fractal-newton-about = 求 z^3 = 1 之根的牛頓疊代，依每個點收斂的速度著色。一種收斂（而非逃逸）碎形。
fractal-multibrot6-about = 6 次方的 Multibrot：五重對稱。
fractal-multibrot7-about = 7 次方的 Multibrot：六重對稱，圓盤逐漸變細，趨近於圓形。
fractal-multibrot8-about = 8 次方的 Multibrot：七重對稱。
fractal-burning-ship3-about = 3 次方的 Burning Ship：先取絕對值，再取立方。
fractal-burning-ship4-about = 4 次方的 Burning Ship。
fractal-burning-ship5-about = 5 次方的 Burning Ship。
fractal-tricorn3-about = 3 次方的 Tricorn（Mandelbar）：四重對稱。
fractal-tricorn4-about = 4 次方的 Tricorn：五重對稱。
fractal-tricorn5-about = 5 次方的 Tricorn：六重對稱。
fractal-celtic3-about = 3 次方的 Celtic 折疊：z^3 實部的絕對值。
fractal-celtic4-about = 4 次方的 Celtic 折疊。
fractal-celtic5-about = 5 次方的 Celtic 折疊。
fractal-buffalo3-about = 3 次方的 Buffalo：z^3 兩個部分的絕對值。
fractal-buffalo4-about = 4 次方的 Buffalo。
fractal-buffalo5-about = 5 次方的 Buffalo。
fractal-custom-about = 以 Fractint 風格運算式寫成的公式（碎形 ▸ 自訂公式…）。大多數都像內建族一樣以微擾進行深度縮放；公式對話方塊會告訴你你的公式是否如此。
fractal-life-about = Conway 的生命遊戲及其同類，位於無界的平面上：Generations 規則、非總和型（Hensel）規則、圖樣檔。播放、單步、繪製。
fractal-lsystem-about = Lindenmayer 系統：由海龜依改寫文法繪出的曲線、空間填充曲線、島嶼與植物。階數隨縮放變化，因此細節永遠不會用盡。Fractint 的 .l 檔案可以直接開啟。

# Shown where the other families show their formula (z -> z^2 + c).
fractal-custom-formula = z -> 你寫的公式
# Shown where the other families show their formula. "B3/S23" is the rule's notation: keep it.
fractal-life-formula = B3/S23：有 3 個活鄰居時誕生，有 2 或 3 個時存活
# Shown where the other families show their formula. "F = F+F--F+F" is the grammar's notation: keep it.
fractal-lsystem-formula = F = F+F--F+F：每個符號經過改寫，再由海龜繪出

# The last paragraph of the picker's hover text for a family that renders only on the direct path.
fractal-no-deep-zoom = 僅限直接算繪：這個族沒有深度縮放。

## Names of settings, shown in the Settings menu and the welcome window.

theme-dark = 深色
theme-light = 淺色
# The update track that follows only stable releases.
update-track-stable = 穩定版
# The update track that also offers pre-releases (betas).
update-track-beta = Beta（預先發行版）

## The status bar (its hover texts; the readouts follow in their own section)

status-period-hover = 你跳到的迷你 Mandelbrot 的週期（按 M，或「前往位置 ▸ 前往特徵」）。畫面停留在該特徵上時會保留，移開後就會清除。

## The playback transport: the player over the view while a tour plays.

# $close: the close button's icon, as drawn on that button.
play-finished-hover = 已結束：拖曳回去重看，或按 { $close } 關閉
# $kind: "benchmark" or "script" (the variants hold the wording); $name: the tour's name.
play-playing-hover =
    { $kind ->
        [benchmark] 正在播放基準測試「{ $name }」
       *[other] 正在播放腳本「{ $name }」
    }
play-restart = 從頭重新開始
play-back = 倒退 10 秒
play-again = 從頭再播放一次
play-resume = 繼續
play-pause = 暫停
play-stop = 停止並倒回開頭
play-forward = 快轉 10 秒
play-speed-hover = 播放速度（按一下可在 0.5× / 1× / 2× / 4× 之間循環）
play-render = 將這個腳本算繪成影格序列…
play-render-none = 內建的基準測試沒有可算繪的腳本檔
play-open = 播放其他腳本…
play-loop = 導覽播放到結尾時重複播放
play-close = 關閉播放器並還原你自己的畫面設定
# A notice beside the scrub bar while the renderer holds the tour's clock. Keep it short.
play-waiting = 等待細節
# "[playback] pace" is a setting in the script file: keep it as written.
play-waiting-hover = 算繪器解析此畫面時（建構參考軌道／疊代預算攀升中），導覽時鐘會暫停。播放會自行恢復；請參閱腳本中的 [playback] pace。
play-scrub = 拖曳以在導覽中快速瀏覽
# $behind: how far the script clock is behind real time, as minutes:seconds (1:05).
play-scrub-behind =
    拖曳以在導覽中快速瀏覽
    腳本時鐘落後實際時間 { $behind }（節奏控制器放慢了導覽，好讓算繪器跟上）。上方的刻度標出依實際時間播放時應在的位置。

## The autopilot's notices (toasts). Where a notice has two lines, the first is the verdict and the second what to do about it.

autopilot-on-detail = 自動駕駛已開啟：朝細節下潛（任何輸入都會讓它停止）
autopilot-off = 自動駕駛已關閉
autopilot-misi-needs-mandelbrot =
    自動駕駛：Misiurewicz 目標需要 Mandelbrot 集合
    改為朝細節下潛
autopilot-on-misi =
    自動駕駛已開啟：正在尋找最近的 Misiurewicz 點…
    任何輸入都會讓它停止
# $seconds: how long the search has run, in whole seconds.
autopilot-misi-finding =
    自動駕駛：正在尋找最近的 Misiurewicz 點…
    { $seconds } 秒 · 任何輸入都會讓它停止
autopilot-misi-not-found =
    自動駕駛：在此畫面附近找不到 Misiurewicz 點
    尋找器可能無法到達這個深度；請試試「細節」目標
# ($preperiod,$period) names the point, as mathematicians write it: keep the notation. $widths: a power of two (2^$widths screen widths).
autopilot-misi-off-screen =
    自動駕駛：最近的 Misiurewicz 點 ({ $preperiod },{ $period }) 在螢幕外
    距離約 2^{ $widths } 個螢幕寬度；請靠近螺旋中心
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-not-solved =
    自動駕駛：此畫面附近的 Misiurewicz 點 ({ $preperiod },{ $period }) 求解失敗
    請試試「細節」目標
autopilot-misi-refused = 自動駕駛：Misiurewicz 求解被拒絕（無效的數對）
autopilot-misi-stopped = 自動駕駛：Misiurewicz 求解器意外停止
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-diving =
    自動駕駛：正在下潛至 Misiurewicz ({ $preperiod },{ $period })
    任何輸入都會讓它停止
# ($preperiod,$period) names the point: keep the notation. $octaves: how often the structure repeats (2.37); $twist: its turn in degrees, signed (+12.5).
autopilot-misi-diving-repeat =
    自動駕駛：正在下潛至 Misiurewicz ({ $preperiod },{ $period })
    結構每 { $octaves } 個八度重複一次，扭轉 { $twist }° · 任何輸入都會讓它停止
# $limit: the magnification, as written (1e300).
autopilot-dive-limit = 自動駕駛：已達下潛上限（~{ $limit }×）
# $have: the fixed iteration count; $typical: what this depth typically needs.
autopilot-dead-end-short =
    自動駕駛已停止：沒有可以瞄準的目標了：疊代次數固定為 { $have }，而這個深度通常需要約 { $typical }
    請開啟自動疊代或調高次數
# $limit: the fixed iteration count.
autopilot-dead-end-cap =
    自動駕駛已停止：沒有可以瞄準的目標了：畫面中的一切都跑到了固定的疊代上限 { $limit }
    請開啟自動疊代或調高次數
autopilot-dead-end = 自動駕駛：前方沒有細節（已停止）

## Tours (camera scripts)

# The name of the file type in the file dialog's filter list.
script-file-filter = Fractadyne 腳本（TOML）
# $name: the script's name.
script-finished = 腳本已結束：「{ $name }」
# The title of the notice that follows (the file's path and the error, in English).
script-load-failed = 無法載入腳本

## Installing a language file

# The name of the file type in the file dialog's filter list.
language-file-filter = Fractadyne 語言檔

## The notice when the graphics driver's shader cache stops keeping shaders. The text is shown in a fixed-width font with its line breaks as written: break the lines where they read well.

driver-cache-title = 圖形驅動程式未保留著色器
# $count: how many shaders compiled again; $pipeline: the slowest one's name (a technical id: keep it); $seconds: its time in seconds (7.5).
driver-cache-body =
    你的圖形驅動程式在這個工作階段中，再次編譯了 { $count } 個
    Fractadyne 著色器，儘管每一個先前都已編譯過（最慢的：
    { $pipeline }，{ $seconds } 秒）。它的著色器快取沒有保留
    已編譯的著色器，最常見的原因是快取已滿。在情況改善之前，
    匯出、深層畫面以及啟動 Fractadyne，都可能各花上數秒編譯。
# $folder: the cache folder's path; $size: its size in MB (1024). "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia-folder =
    修正方法：關閉 Fractadyne，然後刪除以下資料夾的內容
      { $folder }   ({ $size } MB)
    驅動程式會重建快取。為避免快取再次填滿，請調高
    NVIDIA 控制面板 > 管理 3D 設定 > 著色器快取大小。
# "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia =
    修正方法：調高 NVIDIA 控制面板 > 管理 3D 設定 > 著色器快取大小，
    或清除驅動程式的著色器快取。
# "AMD Software > Graphics > Advanced > Reset Shader Cache" names AMD's own settings: use the names AMD's software shows in this language.
driver-cache-fix-amd = 修正方法：AMD Software > 圖形 > 進階 > 重設著色器快取。
driver-cache-fix-other = 修正方法：清除圖形驅動程式的著色器快取。

## Why a language file could not be installed: each ends the sentence "{ $file } is not a usable language file: …".

language-install-not-utf8 = 不是 UTF-8 文字
language-install-too-large = 檔案太大
# "fractadyne-<language>.ftl" is the file name to use: keep it as written.
language-install-no-tag = 檔案沒有說明它是哪一種語言（請將它命名為 fractadyne-<language>.ftl）
language-install-english = 英文是內建的
language-install-empty = 它沒有翻譯此版本的任何訊息
language-install-no-folder = 沒有設定資料夾

## The notice after a session that did not end cleanly, when its iteration setting was set aside.

# $base: the iteration count automatic iterations start from.
quarantine-setting-auto = 從基準 { $base } 開始的自動疊代
# $count: the fixed iteration count.
quarantine-setting-fixed = { $count } 次疊代
# $end: "device" when the session lost its graphics device, "unclean" otherwise (the variants hold the wording). $depth: the magnification, as written (1.2e148). $setting and $restored: an iteration setting (quarantine-setting-auto or -fixed).
quarantine-notice =
    { $end ->
        [device] 上一個工作階段在 { $depth }× 處、設定為「{ $setting }」時遺失了圖形裝置。這個工作階段改以「{ $restored }」啟動，因此不會直接走回同樣的負載中。
       *[other] 上一個工作階段在 { $depth }× 處、設定為「{ $setting }」時未正常結束。這個工作階段改以「{ $restored }」啟動，因此不會直接走回同樣的負載中。
    }

## The second graphics card's status line, under its setting in the control panel.

gpu2-state-off = 關閉：由這個視窗的顯示卡算繪所有內容。
gpu2-state-starting = 啟動中…
# $name: the card's name (and its graphics API).
gpu2-state-running = 使用中：{ $name }
# $why: the error, as the graphics driver reports it.
gpu2-state-failed = 無法啟動：{ $why }
# $name: the card's name.
gpu2-state-lost = 已停止：{ $name } 已遺失。從現在起只用一張顯示卡；再次選擇它即可重試。
# $state: one of the lines above. "--worker-gpu" is a command-line option: keep it.
gpu2-state-pinned = { $state }（由 --worker-gpu 設定）

## The custom-formula dialog: how deep the formula renders (one line under the formula).

formula-depth-deep = 以微擾進行深度縮放，超過 1e28x 時與內建碎形一樣使用擴充範圍（每一步都完整疊代：自訂公式目前還沒有級數近似或 BLA）。
# $why: what the formula has that keeps it off the deep-zoom path (one of the formula-depth-why messages below).
formula-depth-direct = 直接算繪（{ $why } 目前還沒有深度縮放形式）：清晰度維持到 c 的一個單精度步長跨越一個像素為止，大約 1e4x 到 1e5x，離原點越遠就越淺。
formula-depth-why-init = 初始化區段
formula-depth-why-bailout = 自己的逃逸條件
formula-depth-why-variable = 在步與步之間保留的變數
formula-depth-why-previous = 前一次的疊代值
formula-depth-why-comparison = 比較或 if 區塊
# floor, ceil, trunc and round are functions of the formula language: keep them.
formula-depth-why-rounding = 取整（floor、ceil、trunc、round）
# "maxit" is a word of the formula language: keep it.
formula-depth-why-maxit = maxit
formula-depth-why-power = 指數會變化的乘冪
formula-depth-why-perturbed = 已經微擾過的程式

## Image and frame sizes: the presets described in words (the others are standard names: HD 720p, WUXGA, DCI 4K).

# $size: the pixel size (2048×2048).
size-preset-square = 正方形：{ $size }
# $ratio: the aspect ratio (9:16); $size: the pixel size.
size-preset-portrait = 直向 { $ratio }：{ $size }
# $ratio: the aspect ratio (32:9); $size: the pixel size.
size-preset-super-ultrawide = 超寬螢幕 { $ratio }：{ $size }

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
fractal-family-custom = 自訂
# Conway's Game of Life.
fractal-family-life = 生命遊戲
fractal-family-lsystem = L 系統
# A family at a power: "Multibrot 3", "Burning Ship 4". $family: a name above; $power: the power.
fractal-with-power = { $family } { $power }

## Names of choices: colouring methods, orbit traps, palette animation, snapshots, angle units.

color-method-smooth = 平滑疊代
color-method-stripe = 條紋平均
color-method-triangle = 三角不等式
color-method-trap = 軌道陷阱
color-method-distance = 距離估計
color-method-decomposition = 分解
# The shape an orbit trap catches the orbit with.
trap-point = 點
trap-cross = 十字
trap-circle = 圓
# Palette animation: none.
palette-anim-off = 關閉
palette-anim-forward = 正向
palette-anim-reverse = 反向
palette-anim-pingpong = 來回
# Palette animation: a new random gradient every so often.
palette-anim-random = 隨機漸層
# What the Snapshot button does.
snapshot-mode-ask = 每次詢問
snapshot-mode-screen = 螢幕擷取
snapshot-mode-render = 完整算繪
# The unit of an angle typed into Go to location's polar fields.
angle-degrees = 度
angle-radians = 弧度
angle-turns = 圈

## Report an issue: the choices. (The report itself is sent in English.)

issue-kind-crash = 程式當機／凍結
issue-kind-rendering = 算繪不正確
issue-kind-performance = 效能問題
issue-kind-ui = 介面／易用性問題
issue-kind-feature = 功能建議
issue-kind-other = 其他
severity-low = 低
severity-medium = 中
severity-high = 高
severity-blocking = 無法繼續使用
repro-always = 每次都會
repro-sometimes = 有時會
repro-once = 只發生過一次
repro-cannot = 無法重現

## The benchmark's depths and the diagnostics' tests.

# $zoom: the dive's final magnification (1e32).
bench-depth-standard = 標準（{ $zoom }×）
# $zoom: the dive's final magnification (1e48).
bench-depth-ultra = 超深（{ $zoom }×）
diag-test-selftest = 自我測試
diag-test-uitest = 介面測試
diag-test-gputest = GPU 算術檢查

## The status bar's readouts: each word is followed by a value in a slot of fixed width. Keep the words short (one word each).

status-center = 中心
status-cursor = 游標
status-zoom = 縮放
# Short for iterations.
status-iter = 疊代
# The period of the minibrot jumped to.
status-period = 週期
# A Game of Life view's scale (followed by "1 px = 4 cells" or "8 px / cell").
status-scale = 比例
# A Game of Life view's generation. Short.
status-generation = 世代
# A Game of Life view's population. Short.
status-population = 細胞數
# An L-system's order (how many times its rules were applied).
status-order = 階數
# How many segments and shapes an L-system drew.
status-drawn = 已繪
# $value: how many cells one pixel covers (4, 9.99e300). "px" is the unit symbol: keep it.
life-scale-cells = 1 px = { $value } 個細胞
# $value: how many pixels one cell covers (8). "px" is the unit symbol: keep it.
life-scale-pixels = { $value } px / 細胞

## The status bar's limit diagnostic: a short label (after a warning sign), and what it means when hovered.

# The view is as deep as this location can be rendered. Short.
status-limit-depth = 深度極限
# Every pixel ran out of iterations. Short.
status-limit-exhausted = 疊代耗盡
# Pixels are capped at the iteration budget. Short.
status-limit-capped = 疊代受限
# $pixels: how many pixels wide one step of the coordinates is; $sharp_to: the magnification the view is sharp to (written out, 1,234,567).
status-limit-direct-detail = 此畫面直接在 GPU 上以單精度算繪，在這裡座標的一個步長有 { $pixels } 個像素寬：相鄰的像素計算的是同一個點，因此影像會碎成方塊。在這個位置，它的清晰度約可達 { $sharp_to }×。這個公式目前還沒有深度縮放（微擾）路徑，因此繼續放大也不會顯示更多細節。
# $samples: the GPU buffer's capacity in orbit samples; $iterations: the iterations pixels are limited to.
status-limit-depth-detail = 參考軌道達到 GPU 緩衝區上限（{ $samples } 個樣本）時仍未逃逸，因此這裡的像素被限制在約 { $iterations } 次疊代，在此畫面無法計算更細的細節，無論是即時畫面或匯出皆然。這是此位置族目前的深度極限。
# $budget: the largest iteration budget this view can be given.
status-limit-exhausted-detail = 即使以此畫面所能給予的最大預算（{ $budget }），每個像素仍然用盡了疊代次數。可能是畫面在這裡位於集合內部（在深處很正常），也可能是它需要的疊代次數超過程式目前所能提供的；無論哪一種，影像都相同，因此預算已調回成本較低的值。
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations" is the setting's name in the control panel's Quality section.
status-limit-capped-max = 在 { $budget } 的預算下，有 { $percent }% 的像素用盡了疊代次數，而會逃逸的像素也剛好在這個上限處結束：畫面解析不足。「疊代次數」已經是最大值，因此這個畫面需要的超過程式目前所能提供的。
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations", "Controls" and "Quality" are the names of a setting, the control panel and its section.
status-limit-capped-raise = 在 { $budget } 的預算下，有 { $percent }% 的像素用盡了疊代次數，而會逃逸的像素也剛好在這個上限處結束：畫面解析不足。請調高「疊代次數」（控制項 ▸ 品質）來解析它。
