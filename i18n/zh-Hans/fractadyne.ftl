### Fractadyne user interface: Chinese, Simplified (简体中文).
### A draft: machine translation (Claude, by Anthropic) of the English file, 2026-10-23, not yet
### reviewed by a native speaker. Corrections are welcome as pull requests (see i18n/README.md).

## Language data: read by Fractadyne, not shown as messages.

# This language's own name, shown in the language menu.
language-name = 简体中文
# The same name in English, shown beside it in the menu.
language-english-name = Chinese (Simplified)
# The writing system the app must be able to draw (an ISO 15924 code: Latn, Cyrl, Jpan, Hans, Khmr).
language-script = Hans
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
language-more = 更多语言…
# The Languages window's title.
language-window-title = 语言
# The Languages window's introduction.
language-window-intro = 以其他语言显示 Fractadyne 的界面。每种语言就是一个文件：随版本一同提供，或在此处安装。在任何语言下，坐标、文件和命令行都保持英文。
# Column headers of the installed-language list.
language-col-language = 语言
language-col-translated = 已翻译
language-col-from = 来源
# Where a language comes from.
language-source-builtin = 内置
language-source-package = 本版本
language-source-user = 已安装
language-source-dev = 工作文件夹
# $percent: how much of the interface the file translates, 0–100.
language-coverage = { $percent }%
# $count: messages in the file that failed the checks and show in English instead.
language-dropped = { $count } 条消息需要检查
# Buttons of the Languages window.
language-install = 安装语言文件…
language-install-hover = 选择一个 Fractadyne 语言文件（.ftl）。它会先经过检查，再复制到你的设置中的语言文件夹，并在更新后保留。
language-open-folder = 打开语言文件夹
language-reload = 重新加载
language-reload-hover = 重新读取语言文件（在你编辑或添加文件之后）。
language-use = 使用
# Notices after installing a file. $name: the language's own name; $file: the file's name.
language-installed = 已从 { $file } 安装 { $name }。
# $file: the file's name; $why: what was wrong with it.
language-install-failed = { $file } 不是可用的语言文件：{ $why }
# Shown when a language file cannot be loaded and English is used instead. $name: the language.
language-load-failed = 无法加载 { $name }，因此 Fractadyne 以英文显示：{ $why }
# A language whose writing system needs a font this computer does not have (Japanese or Chinese without a system font). $name: the language, in its own name.
language-font-missing = 这台计算机上没有找到能够显示 { $name } 的字体，因此无法显示该语言。请安装 { $name } 字体（例如系统自带的语言包），然后重新选择它。
# Shown after a draft translation's name in the language menu and the Languages window. Short.
language-draft = 草稿
language-draft-hover = 草稿：此翻译尚未经过母语者审校。欢迎通过对该语言文件提交拉取请求来加以修正。
# The heading over the decimal-mark choice, in the language menu and the Languages window.
language-decimal-mark = 小数点符号
language-decimal-mark-hover = 数字的书写方式：用于时间、大小和数量。坐标和放大倍数始终使用小数点，因为文件和其他程序按这种方式读取。
# $sample: a number written with this language's own punctuation (1,234.5).
language-decimal-language = 按该语言的写法（{ $sample }）
# $sample: a number written with a decimal point (1,234.5).
language-decimal-point = 点号（{ $sample }）
# $sample: a number written with a decimal comma (1.234,5).
language-decimal-comma = 逗号（{ $sample }）

## Asked once when the system's language changes

# The window's title.
language-changed-title = 语言
# $new: the system's new language, in its own name.
language-changed-question = 你的系统语言现在是 { $new }。要在 Fractadyne 中使用该语言吗？
# Buttons. $name: a language's own name.
language-changed-use = 使用 { $name }
language-changed-keep = 保留 { $name }

## The welcome window (first run)

welcome-title = 欢迎使用 Fractadyne
welcome-intro = 一款用于探索 Mandelbrot 集及其同类分形的深度缩放工具。以下几个操作可以帮你快速上手：
# The language row, at the top of the window.
welcome-language = 语言
# Controls: the key or gesture on the left, what it does on the right.
welcome-key-drag = 拖动
welcome-key-drag-does = 平移视图
welcome-key-scroll = 滚动
welcome-key-scroll-does = 以光标为中心放大 / 缩小
welcome-key-space = 按住 Space
welcome-key-space-does = 平滑连续放大（Shift+Space 缩小）
# Shown after the click-zoom toolbar icon.
welcome-key-click = 然后单击
welcome-key-click-does = 深入所单击的点（工具栏）
welcome-key-m-does = 跳到最近的迷你 Mandelbrot，并以其自身的尺度显示
welcome-landmarks = 跳到一个地标开始探索：
welcome-theme = 主题
welcome-watermark = 显示“Fd”标记
welcome-watermark-hover = 视图右下角的小品牌标记。关闭后，实时视图和导出的图像中都不再显示它。
welcome-updates = 更新
welcome-check-updates = 检查更新
welcome-check-updates-hover = 程序启动时向 GitHub 查询最新版本。不会上传任何内容。
welcome-files = 文件
welcome-no-config = （无配置目录）
welcome-open = 打开
welcome-rendering-on = 渲染设备
welcome-open-help = 打开完整帮助（F1）
# $icon: the Settings icon.
welcome-more-in-settings = 更多选项见“文件 → { $icon } 设置”
welcome-get-started = 开始使用

## Menus: shared

# A percentage in a menu (UI scale). $value: the number.
common-percent = { $value }%
# A state shown after a menu item's name, in brackets.
menu-state-connected = 已连接
menu-state-rendering = 渲染中
menu-state-running = 运行中

## Navigate ▸ Bookmarks (also the toolbar's star)

menu-bookmarks = 书签
menu-bookmarks-add = 添加当前视图
menu-bookmarks-manage = 管理…

## The fractal picker (Fractal menu and the toolbar's dropdown)

menu-family-powers = 高次幂
menu-family-automata = 元胞自动机
menu-family-lsystems = L 系统
menu-fractal-custom = 自定义公式…
menu-fractal-library = 公式库…
menu-fractal-library-hover = 已保存的自定义公式：应用、编辑、导入和导出
# The "Show" group: which picture of the formula is drawn. $name: the formula's name (Mandelbrot, Burning Ship…).
menu-show = 显示
menu-show-set = { $name } 集
menu-show-set-hover = 参数平面——即 { $name } 集本身。

    其中每一个点都对应一个不同的 Julia 集；这是所有 Julia 集的地图。
menu-show-julia = Julia 集
menu-show-julia-hover = { $name } 集中的一个点，单独绘制成一幅图像——即固定该点时得到的形状。

    在“两者联动”中将鼠标悬停在地图上来选取该点；在那里单击即可固定。
menu-show-julia-none = { $name } 没有自由参数，因此没有 Julia 集。
menu-show-both = 两者联动
menu-show-both-hover = 左侧为 { $name } 集，右侧为光标所在点的 Julia 集。

    在地图上移动光标，观察 Julia 集的变化——这是了解两者关系最快的方法。
menu-show-both-none = { $name } 没有可与之配对的 Julia 集。

## File menu

menu-file = 文件
menu-file-open-view = 打开视图或位置…
menu-file-gallery = 图库…
menu-file-export = 导出图像…
menu-file-snapshot = 快照
menu-file-open-life = 打开生命游戏图样…
menu-file-open-life-hover = RLE、.cells 或 Life 1.05/1.06 文件，显示为一个生命游戏宇宙。
menu-file-life-text = 生命游戏图样文本…
menu-file-life-text-hover = 以文本形式粘贴图样（LifeWiki 所提供的 RLE 格式）。
menu-file-save-life = 保存生命游戏图样…
menu-file-save-life-hover = 宇宙的当前状态，保存为 RLE。
menu-file-open-lsystem = 打开 L 系统…
menu-file-open-lsystem-hover = Fractint 的 .l 文件（可从其中的条目中选取）或 .lsys 系统。
menu-file-save-lsystem = 保存 L 系统…
menu-file-save-lsystem-hover = 系统的文本，保存为 .lsys 文件。
menu-file-export-svg = 导出 SVG…
menu-file-export-svg-hover = 将视图导出为 SVG 图形：其线条和形状为矢量，并保留原有颜色。
menu-file-render-client = 渲染客户端…
menu-file-render-client-hover = 将本机的 GPU 借给渲染农场：连接到一个控制端（另一台计算机上的“工具 ▸ 在渲染农场上渲染…”），为它渲染帧。
menu-file-settings = 设置
menu-file-reset = 重置应用程序状态…
menu-file-reset-hover = 删除所有已保存的状态（会话、书签、缩略图）并重新开始。会先请求确认。
menu-file-quit = 退出

## File ▸ Settings

menu-settings-fps = 帧率上限
menu-settings-fps-uncapped = 不限制
# $fps: frames per second.
menu-settings-fps-n = { $fps } FPS
menu-settings-ui-scale = 界面缩放（字号）
menu-settings-theme = 主题
menu-settings-updates = 更新
menu-settings-updates-stable-hover = 最新的稳定版
menu-settings-updates-beta-hover = 最新版本，包括预发布版
menu-settings-updates-on-launch = 启动时检查更新
menu-settings-updates-on-launch-hover = 否则，请通过“帮助 → 检查更新”手动检查。
# Followed by its shortcut, (Ctrl+S).
menu-settings-snapshot = 快照
menu-settings-snapshot-ask-hover = 每次按下时都询问
menu-settings-snapshot-screen-hover = 完全按屏幕上的样子保存视图（即时，PNG）
menu-settings-snapshot-render-hover = 按“导出”设置在后台完整渲染（深度视图可能需要很长时间）
menu-settings-refcache = 参考轨道缓存…
menu-settings-refcache-hover = 保存在磁盘上的深度缩放参考轨道，使你再次访问的位置在几秒内即可渲染完成。可查看其位置、大小和上限，并可将其清除。

## Fractal menu

menu-fractal = 分形

## View menu

menu-view = 视图
menu-view-zoom-home = 缩小到完整视图
menu-view-zoom-home-hover = 缩小到完整的初始视图（带动画）
menu-view-reset = 重置为默认视图
menu-view-reset-hover = 重置为该分形的默认视图（立即）
# Followed by its shortcuts.
menu-view-undo = 撤销视图
menu-view-redo = 重做视图
menu-view-control-panel = 控制面板
menu-view-control-panel-hover = 显示/隐藏右侧的控制面板。
menu-view-minimap = 小地图概览
menu-view-minimap-hover = 显示初始视图的小型概览，带有“你在这里”标记和缩放深度。单击它可跳转到某个区域。
menu-view-perf-panel = 性能面板
menu-view-timestamp = 显示时间戳
menu-view-timestamp-hover = 在视图上绘制一个大号的已用时间时钟。它显示的“+12.345s”与日志每一行开头的时间戳相同，因此对于只在运动中才出现的问题，可以把屏幕录像与日志逐帧对照。默认关闭。
menu-view-zoom-target = 显示自动缩放目标
menu-view-zoom-target-hover = 自动缩放运行时，框出它正在放大的视图区域——即再放大几次后将填满屏幕的部分——并从该区域的四角向屏幕四角连线，在它正驶向的点上画一个圆环。默认关闭。
menu-view-live-panel = 实时渲染面板
menu-view-live-panel-hover = 视图上方的一块前面板，展示实时渲染器的工作情况：每帧派发了什么、屏幕上的画面有多旧、在各迭代区段间的步进、参考轨道构建器，以及底层的机器。按 B 循环切换，单击其角上的 × 可关闭；可将其拖到任意角落；将鼠标悬停在某个仪表上，可了解它的含义，以及正常和异常时分别是什么样子。
# The live render panel's sizes.
menu-view-live-off = 关闭
menu-view-live-off-hover = 不显示面板。
menu-view-live-strip = 条带
menu-view-live-strip-hover = 辉光管读数和指示灯组。
menu-view-live-panel-size = 面板
menu-view-live-panel-size-hover = 各个仪表：步进表盘、画面滞后、图块、GPU 渲染遍、深度表盘、指示灯。
menu-view-live-full = 完整
menu-view-live-full-hover = 面板加上机房：步数、GPU、内存、核心、像素。
menu-view-live-opacity = 不透明度
menu-view-watermark = 显示“Fd”水印
menu-view-watermark-hover = 在实时视图和导出图像的右下角绘制一个不显眼的“Fd”品牌标记。默认开启。

## Color menu

menu-color = 颜色
menu-color-method = 方法
menu-color-palette = 调色板
menu-color-custom = 自定义
menu-color-duotone = 双色调
menu-color-binary = 二值（集合）
menu-color-binary-hover = 纯双色：集合内与集合外，无渐变。
menu-color-edit-gradient = 编辑渐变…
menu-color-normalize = 归一化深层颜色
menu-color-normalize-hover = 当密集的区域原本会显得像噪点时，将调色板重新映射到视图实测的逃逸范围。不产生混叠的视图保持不变——勾选“始终适配范围”可无条件重新映射。仅适用于“平滑迭代”方法。
menu-color-more = 循环、偏移、动画和效果：控制面板 ▸ 着色

## Tools menu

menu-tools = 工具
# Followed by its shortcut, (M).
menu-tools-find-minibrot = 查找并缩放到迷你 Mandelbrot
menu-tools-find-minibrot-hover = 用牛顿法将视图中心吸附到附近迷你 Mandelbrot 的精确中心，报告其周期，并缩放到该迷你 Mandelbrot 自身的尺度——往往一步就跨越许多个数量级。如果已经比它更深？那就只移动中心。
menu-tools-solver = 牛顿 / Misiurewicz 求解器…
menu-tools-solver-hover = 求解精确的迷你 Mandelbrot 中心（周期 + 大小），或离视图最近的 Misiurewicz 点——以全精度运行牛顿法，并给出排斥乘子 λ。会打开包含这些求解器的位置对话框。
# Followed by its shortcut, (A).
menu-tools-autopilot = 自动缩放（自动巡航）
menu-tools-autopilot-stop = 停止自动巡航
menu-tools-autopilot-hover = 无需动手的连续深度缩放：朝细节最丰富的区域深入，并在途中不断调整方向。任何导航输入都会使其停止。
menu-tools-priority = 优先级：
menu-tools-priority-speed = 速度
menu-tools-priority-speed-hover = 按设定的速度缩放；一出现带有细节的帧就刷新。
menu-tools-priority-quality = 质量
menu-tools-priority-quality-hover = 只用完全解析的帧；缩放会等待这些帧。
menu-tools-target = 目标：
menu-tools-target-detail = 细节
menu-tools-target-detail-hover = 视图中最丰富的边缘，在深入过程中不断重新选择。
menu-tools-target-misiurewicz = Misiurewicz 点
menu-tools-target-misiurewicz-hover = 最近的 Misiurewicz 点，求解一次后精确地深入其中（仅限 Mandelbrot）。
menu-tools-orbits = 轨道叠加
menu-tools-orbits-hover = 绘制光标下方那个点的迭代路径。动画选项：控制面板 ▸ 叠加层。
menu-tools-orbits-normalize = 归一化（适配视图）
menu-tools-orbits-normalize-hover = 将轨道适配到整个视图，使其在任何缩放级别下都保持良好的取景（而不是通过视口映射——那样在深处它会飞出屏幕）。
menu-tools-play-tour = 播放漫游…
menu-tools-play-tour-hover = 选择要播放的漫游脚本。单击工具栏上的 ▶ 即可重放上一个漫游。
menu-tools-tour-from-view = 从当前视图创建漫游…
menu-tools-tour-from-view-hover = 创建一个从完整视图缩放到当前视图的漫游——可添加字幕并设置深入时长。保存为漫游脚本（.toml），你可以编辑、播放或渲染它。
menu-tools-render-tour = 渲染漫游…
menu-tools-render-tour-hover = 在单独的进程中，将已加载的漫游渲染为 PNG 帧序列（并可选择生成 mp4）。
menu-tools-close-player = 关闭漫游播放器
menu-tools-close-player-hover = 停止漫游，关闭其播放器，并恢复你自己的迭代和着色设置
menu-tools-render-farm = 在渲染农场上渲染…
menu-tools-render-farm-hover = 在多台计算机上渲染已加载的漫游：本机负责控制任务，其他计算机通过“文件 ▸ 渲染客户端…”加入。
menu-tools-benchmark = 基准测试…
menu-tools-benchmark-hover = 测量渲染速度——使用当前设置，或运行可在不同计算机之间比较的标准化测试（固定分辨率 + 设置）。“烤机”会反复运行，以检查稳定性 / 过热降频。
menu-tools-last-benchmark = 显示上次基准测试结果

## Navigate menu

menu-navigate = 导航
menu-navigate-goto = 转到位置…
menu-navigate-goto-hover = 输入中心/缩放，跳到一个著名的点，或用牛顿法求解视图附近的 Misiurewicz / 迷你 Mandelbrot 特征。
menu-navigate-misiurewicz = Misiurewicz 浏览器…
menu-navigate-misiurewicz-hover = 浏览小型 Misiurewicz 类型的样子——缩略图中标有每个点的缩放周期和扭转——并可用牛顿法将其中任意一个求解到任意深度。（适用于 Mandelbrot。）
menu-navigate-exact = 精确点（表达式）
menu-navigate-exact-note = 主心形线上各芽苞的根点，以精确表达式给出（2π 的某个分数的 cos/sin）。坐标以表达式的形式保存，因此放大时始终精确地停留在该点上。
menu-navigate-random = 随机位置
menu-navigate-random-hover = 跳到一个随机的、细节丰富的边界点
menu-navigate-import-kfr = 导入 .kfr…
menu-navigate-import-kfr-hover = 加载 Kalles Fraktaler 位置文件
menu-navigate-share = 分享位置…
menu-navigate-share-hover = 复制 / 粘贴 / 保存 / 加载一个独立完整的位置（.fdn）：分形、全精度中心、缩放、着色。

## Help menu

menu-help = 帮助
# Followed by its shortcut, (F1).
menu-help-reference = 帮助与参考…
menu-help-welcome = 欢迎 / 快速入门…
menu-help-welcome-hover = 首次运行时的概览：基本操作和几个地标。
menu-help-diagnostics = 诊断…
menu-help-diagnostics-hover = 检查 Fractadyne 能否在你的硬件上正常工作，并将结果附加到问题报告中
menu-help-report = 报告问题…
menu-help-report-hover = 提交 GitHub issue（或发送电子邮件），并附上崩溃报告、日志和位置
menu-help-accelerated = 更快的深度缩放…
menu-help-accelerated-hover = 一个可选的版本，计算深度缩放参考轨道的速度快 2.5-6.4x。图像完全相同；你的设置会沿用。
menu-help-check-updates = 检查更新
# $track: the update track (stable or beta).
# $track: "stable" or "beta", the update track in use (the variants hold the wording).
menu-help-check-updates-hover =
    { $track ->
        [beta] 在 GitHub 上检查是否有更新的 Beta（预发布）版本
       *[other] 在 GitHub 上检查是否有更新的稳定版
    }
# $version: the new version; $channel: "beta" for a pre-release, "stable" otherwise (the variants hold the wording).
menu-help-update-available =
    { $channel ->
        [beta] { $version }（beta）已发布——下载
       *[other] { $version }（稳定版）已发布——下载
    }
menu-help-up-to-date = 已是最新版本
menu-help-tagline = 原生分形探索器
menu-help-license = 许可证
# $year and $holder: the copyright year and holder, as the code gives them.
menu-help-copyright = © { $year } { $holder }。采用双重许可；你可以任选其中一种许可证使用。
menu-help-source = 源代码

## The toolbar (its buttons are icons; these are their tooltips)

menu-toolbar-julia = Julia
menu-toolbar-julia-hover = 显示此公式的 Julia 集，填满窗口
menu-toolbar-dual-hover = 两者联动：左侧为集合，右侧为光标所在点的 Julia 集
menu-toolbar-open-hover = 打开视图或位置（PNG / EXR、.fdn、.kfr）
menu-toolbar-gallery-hover = 图库
menu-toolbar-export-hover = 导出图像…
menu-toolbar-share-hover = 分享位置——复制、粘贴、保存或加载独立完整的 .fdn（分形、全精度中心、缩放、着色）
menu-toolbar-zoom-in-hover = 放大
menu-toolbar-zoom-out-hover = 缩小
# $factor: the click-zoom factor (4, 10…).
menu-toolbar-click-zoom-hover = 单击缩放（{ $factor }×）：左键单击深入该点，右键单击后退（拖动仍为平移）。倍数在“控制面板 ▸ 导航”中设置。
# Added to the click-to-zoom tooltip in the dual view.
menu-toolbar-click-zoom-dual = 此功能开启时，用 Ctrl+单击来钉住 Julia c。
menu-toolbar-autopilot-running-hover = 自动缩放正在运行——单击停止
menu-toolbar-autopilot-hover = 自动缩放：朝细节处深入（A）
menu-toolbar-reset-hover = 重置视图（立即）
menu-toolbar-home-hover = 缩小到初始视图（带动画）
menu-toolbar-bookmarks-hover = 书签：保存此视图，或跳到已保存的视图
# $name: the palette's name.
menu-toolbar-next-palette-hover = 下一个调色板（{ $name }）
# $file: the last tour's file name.
menu-toolbar-play-last-hover = 播放漫游——{ $file }（单击）· 重放上一个漫游；通过“工具 ▸ 播放漫游…”选择其他漫游
menu-toolbar-play-hover = 播放漫游…（工具 ▸ 播放漫游）
menu-toolbar-speed-hover = 播放速度
menu-toolbar-fullscreen-hover = 切换全屏

## Dialogs: words shared by many windows

# A button that closes a window without doing what it offers.
dlg-cancel = 取消
# A button that dismisses a window (nothing in it is pending).
dlg-close = 关闭
# A button that copies the window's text to the clipboard (after a copy icon).
dlg-copy = 复制
# A button that jumps to a location.
dlg-go = 转到
# A button that fills the fields from the view on screen.
dlg-use-current = 使用当前值
# A notice after a file could not be written. $error: the system's reason, as it gives it.
dlg-save-failed = 保存失败：{ $error }
# A notice when no graphics device is available to render with.
dlg-gpu-not-available = GPU 不可用
# The name of the plain-text file type in a save dialog (the extension .txt is added by the code).
dlg-filter-text = 文本

## Asked after a session that did not shut down cleanly

dlg-crash-title = Fractadyne 未正常关闭
dlg-crash-intro = 上一个会话意外结束。本机上已保存了一份报告——要发送它吗？
dlg-crash-review = 在你审阅之前不会发送任何内容：下一个界面会准确显示将包含的内容，系统信息只需取消一个复选框即可排除。
dlg-crash-report = 报告…
dlg-crash-report-hover = 打开“报告问题”，并附上崩溃报告和发生崩溃的位置。
dlg-crash-not-now = 以后再说
dlg-crash-dont-ask = 崩溃后不再询问
dlg-crash-dont-ask-hover = 报告仍会保存在本地，而且“帮助 ▸ 报告问题”始终可用。

## Asked after a session that ended with a second graphics device in use

dlg-second-device-title = 关闭第二图形设备？
# $device: the graphics device's name and details, as the system gives them.
dlg-second-device-intro = 上一个会话在使用第二图形设备（{ $device }）时意外结束。它可能就是原因。
dlg-second-device-note = 关闭后，由绘制窗口的显卡渲染所有内容，和以前一样。你可以在“高级 ▸ 第二显卡”中重新开启它。
dlg-second-device-off = 关闭它
dlg-second-device-keep = 保持开启

## Shown when the previous session's iteration setting was set aside

dlg-quarantine-title = 迭代设置已被搁置
dlg-quarantine-change = 你可以随时在“控制面板 ▸ 质量”中更改它。
dlg-quarantine-keep = 保留默认值
# A button. $setting: the iteration setting the previous session used (such as "20,000 iterations").
dlg-quarantine-restore = 恢复“{ $setting }”

## Navigate ▸ Go to location

dlg-goto-title = 转到位置
# Before two choices: "x, y" (cartesian) and the polar form below.
dlg-goto-enter-as = 输入方式：
# The polar way of entering a point: an offset plus a radius r at an angle θ. Keep the symbols.
dlg-goto-polar = 偏移 + r ∠ θ
# The tooltip of the "?" button beside the entry choice.
dlg-goto-help-hover = 各字段可接受的内容：分数、幂、sqrt/sin/cos…、pi 及其他常数——打开“帮助 → 坐标表达式”
# Labels over the entry fields. x0, y0, r, θ, Re and Im are the fields' symbols: keep them.
dlg-goto-polar-offset = 偏移中心（x0, y0——纯极坐标点请保留 0）
dlg-goto-polar-radius = 半径 r，角度 θ
dlg-goto-re = 中心——实部（Re）
dlg-goto-im = 中心——虚部（Im）
dlg-goto-zoom = 缩放（放大倍数）
# A warning under the zoom field, after a ⚠ sign. $have: the iteration count set; $typical: the count this depth typically needs.
dlg-goto-iter-warning = 迭代次数固定为 { $have }；这个深度通常需要约 { $typical }。在提高该值之前，视图会渲染成一片平坦——或者启用自动迭代（控制面板 ▸ 质量），它会自行调整。
dlg-goto-copy-hover = 将此位置复制到剪贴板
dlg-goto-cancel-hover = 关闭，不跳到任何地方
# The expressions in brackets are examples of what can be typed: keep them as they are.
dlg-goto-paste-note = 粘贴别人提供的中心/缩放，或单击“复制”来分享。坐标可接受表达式：精确分数（-3/4）、函数和常数（-0.5 + 0.25*cos(pi/4)、sqrt、root(x,n)、^；角度以弧度为单位），实部字段还可接受一个完整的复数值（(37+16i)/100），它会同时填入两个字段。
# A section that opens and closes.
dlg-goto-feature = 转到特征（Mandelbrot）
dlg-goto-feature-poi = 著名的点：
# The dropdown's text before a point is picked.
dlg-goto-feature-poi-pick = 跳到一个兴趣点…
dlg-goto-feature-solve = 或求解当前视图附近的特征：
# Two choices: what to solve for. Misiurewicz is a mathematician's name (a Misiurewicz point).
dlg-goto-feature-misiurewicz = Misiurewicz
dlg-goto-feature-minibrot = 最近的迷你 Mandelbrot
# Labels of two small number fields; k and p are their symbols.
dlg-goto-feature-preperiod = 预周期 k
dlg-goto-feature-period = 周期 p
# The faint text in an empty field: left empty, the value is found automatically.
dlg-goto-feature-auto-hint = 自动
# A small button that empties both fields, so they are found automatically again.
dlg-goto-feature-auto = 自动
dlg-goto-feature-auto-hover = 清空两个输入框，使下一次搜索根据视图中心处的轨道检测预周期和周期。
dlg-goto-feature-kp-hover = 两者都留空，则根据视图中心处的轨道检测它们；找到的值会填在这里。看着一个螺旋，没有人知道它的预周期，而必须提供预周期，正是这个求解器以前在实践中无法使用的原因。
# While a solve runs, followed by its elapsed time and the note below.
dlg-goto-feature-solving = 正在求解…
dlg-goto-feature-solving-note = （任意精度；深度视图耗时更长）
dlg-goto-feature-cancel-hover = 不再等待。求解本身会在后台运行完毕，其结果将被丢弃——不会留下任何半完成的状态。
dlg-goto-feature-find = 在视图附近查找
dlg-goto-feature-find-hover =
    从当前中心出发，用牛顿法求解出精确的特征点（任意精度）。请先导航到该特征附近。

    求解之前，先把上方的“缩放”字段设为你“想要”的深度：该点会按该深度所需的精度计算，视图会直接跳到那里。这样只需一次求解，而不必手动一步步缩放许多次。
dlg-goto-solve-cancelled = 已取消求解。

## Snapshot (the camera button): its tooltip, and the choice on the first press

# The camera button's tooltip, one per Snapshot setting. Followed by its shortcut, (Ctrl+S).
dlg-snapshot-hover-ask = 快照——保存屏幕上显示的视图，或完整渲染；第一次按下时会询问
dlg-snapshot-hover-screen = 快照——完全按显示的样子保存视图，存到上次的导出文件夹
dlg-snapshot-hover-render = 快照——按“导出”设置完整渲染，存到上次的导出文件夹
# The window's title.
dlg-snapshot-title = 快照
dlg-snapshot-question = “快照”（相机按钮，Ctrl+S）应该保存什么？
# The first choice, and a short tag beside it.
dlg-snapshot-screen = 屏幕截图
dlg-snapshot-screen-tag = 即时
dlg-snapshot-screen-note = 完全按屏幕上的样子、以屏幕分辨率保存视图，以 PNG 格式存到上次的导出文件夹。所见即所得。
# The second choice, and a short tag beside it.
dlg-snapshot-render = 完整渲染
dlg-snapshot-render-tag = 可能需要很长时间
# $size: the image size, as 1920×1080; $ss: the supersampling factor (1–4); $format: PNG or EXR.
dlg-snapshot-render-note = 按“导出”设置渲染——{ $size }、{ $ss }× 超采样、{ $format }。质量最佳，但深度视图可能需要几分钟到几小时。它在后台运行；“文件 → 导出图像…”会显示进度，并可将其取消。
dlg-snapshot-remember = 记住我的选择，不再询问
dlg-snapshot-change = 你可以随时在“文件 → 设置 → 快照”中更改。

## Share location (.fdn)

dlg-share-title = 分享位置
dlg-share-intro = 一个独立完整的位置（分形、全精度中心、缩放、着色）。复制它即可分享，或粘贴/加载一个位置后单击“应用”。
dlg-share-thumb = 在保存的文件中包含缩略图
dlg-share-thumb-hover = “保存 .fdn”会嵌入此视图的一张小图片，以便在图库中识别该文件。大约增加 55 KB，且不属于上方的文本。
dlg-share-copy-hover = 将文本复制到剪贴板
# Buttons (the first after a save icon).
dlg-share-save = 保存 .fdn…
dlg-share-load = 加载 .fdn…
dlg-share-apply = 应用
dlg-share-apply-hover = 跳到框中的位置
dlg-share-copied = 已复制到剪贴板。

## Help ▸ Report an issue (the report itself stays in English: it is read by the maintainer)

dlg-report-title = 报告问题
dlg-report-intro = 报告问题的最佳途径是 GitHub issue——它公开且可搜索，修复后你也能看到。也可以通过电子邮件报告。请先检查下方的全部内容：在你采取操作之前，任何内容都不会离开你的计算机。
# Labels before three dropdowns.
dlg-report-type = 类型：
dlg-report-severity = 严重程度：
dlg-report-repro = 可重现：
dlg-report-describe = 描述发生了什么：
# The faint text in the empty description box.
dlg-report-describe-hint = 发生了什么、重现的步骤，以及你原本期望的结果。
# Before the checkboxes of what the report carries.
dlg-report-include = 包含：
dlg-report-sysinfo = 系统信息（版本、操作系统、CPU、GPU、显存）
dlg-report-location = 当前位置（.fdn）
dlg-report-log = 最近的日志
dlg-report-frames = 帧记录（最后一分钟，以及最后 40 帧）
dlg-report-crash = 最新的崩溃报告
dlg-report-crash-none = 最新的崩溃报告（未找到）
dlg-report-crash-view = 发生崩溃的位置（.fdn）
dlg-report-crash-view-none = 发生崩溃的位置（.fdn）（未找到）
dlg-report-crash-view-hover = 发生崩溃或图形设备丢失时的确切视图——这是唯一能让维护者重现问题的东西。自动记录。
dlg-report-test = 诊断测试结果
dlg-report-test-none = 诊断测试结果（可从“帮助 → 诊断…”运行一次）
dlg-report-test-hover = 在你自己的硬件上得出、经机器验证的通过/失败结果——对维护者来说，比单纯的描述有用得多
# A section that opens and closes. $size: the report's size, as "12 KB".
dlg-report-preview = 预览将要发送的内容（{ $size }）
dlg-report-github = 提交 GitHub issue
dlg-report-github-hover = 打开已填好标题的新 issue 页面；报告会复制到你的剪贴板——请将其粘贴到 issue 正文中（Ctrl+V）。
# Buttons (after a copy and a save icon).
dlg-report-copy = 复制报告
dlg-report-copy-hover = 将全部内容复制到剪贴板
dlg-report-save = 保存报告…
dlg-report-save-hover = 保存为 .txt 以便作为附件
dlg-report-by-email = 或通过电子邮件：
dlg-report-gmail = 在 Gmail 中撰写
# $address: the email address reports go to.
dlg-report-gmail-hover = 打开 Gmail 撰写窗口，收件人为 { $address }，主题已填好；请粘贴已复制的报告
dlg-report-email = 邮件应用…
dlg-report-email-hover = 打开你的默认邮件应用（mailto）——需要已配置电子邮件处理程序
dlg-report-tip = 提示：无论点击其中哪一个，报告都会复制到你的剪贴板——请将其粘贴到 issue 或邮件正文中（Ctrl+V）。如果可以，也请附上截图或示例图像（拖进去即可）；这对渲染和界面问题非常有帮助。
# Notices in the window after a button is pressed.
dlg-report-opened-github = 已打开 GitHub——报告在剪贴板上；请将其粘贴到 issue 中。
dlg-report-copied = 报告已复制到剪贴板。
# $path: where the report was saved.
dlg-report-saved = 已保存到 { $path }
dlg-report-opened-email = 已打开你的邮件应用——报告在剪贴板上；请将其粘贴进去。
dlg-report-opened-gmail = 已打开 Gmail 撰写窗口——请将已复制的报告粘贴到正文中（Ctrl+V）。

## A location whose checksum does not match

dlg-checksum-title = 此位置可能已损坏
# After a ⚠ sign.
dlg-checksum-mismatch = 它的校验和与旁边的数据不匹配。
dlg-checksum-why = 这通常意味着文本在传递途中被截断或改动——常见的原因是从聊天窗口或论坛帖子中只复制了一部分。
# A label over where the location came from (a file path, or the clipboard).
dlg-checksum-source = 来源
# The two checksums, side by side: the one written in the file, and the one computed from its data.
dlg-checksum-in-file = 文件中的
dlg-checksum-from-data = 由数据算出的
dlg-checksum-load-note = 你仍然可以加载它——坐标可能仍然正确，而且不会覆盖任何内容。加载后会报告所有无法读取的内容。
dlg-checksum-load = 仍然加载
dlg-checksum-load-hover = 尽管不匹配，仍然跳到该位置
# Notices after loading it. $zoom: the magnification, as 1.2e148.
dlg-checksum-loaded = 已加载 @ { $zoom }×，尽管校验和不匹配
# $zoom: the magnification, as 1.2e148; $note: what the loader found wrong with the location.
dlg-checksum-loaded-note = 已加载 @ { $zoom }×——{ $note }
dlg-checksum-not-loaded = 未加载。

## File ▸ Reset application state

dlg-reset-title = 重置应用程序状态
# After a ⚠ sign, followed by a list.
dlg-reset-warning = 这将永久删除所有已保存的 Fractadyne 数据：
# The list's items, each after a bullet.
dlg-reset-session = 已保存的会话（当前视图、着色、偏好设置）
dlg-reset-bookmarks = 所有书签及其缩略图
dlg-reset-refcache = 参考轨道缓存（再次访问各个位置时会重建）
# A label over the folder that is deleted.
dlg-reset-location = 位置
dlg-reset-note = 此操作无法撤销。退出时不会再保存当前会话，因此下次启动时将加载默认设置。
dlg-reset-confirm = 全部重置
dlg-reset-done = 应用程序状态已重置——下次启动时将加载默认设置。
# $error: the system's reason, as it gives it.
dlg-reset-failed = 重置失败：{ $error }

## Tools ▸ Tour from current view

dlg-tour-title = 从当前视图创建漫游
dlg-tour-intro = 创建一个从完整视图一直缩放到你当前所在位置的漫游。
# A label before the fractal and magnification the tour ends at.
dlg-tour-destination = 目的地
dlg-tour-notation = 注释（深入过程中显示的可选字幕）
# The faint example text in the empty caption box.
dlg-tour-notation-hint = 例如：深入 Misiurewicz 三臂点…
dlg-tour-duration = 缩放时长
# $rate: how fast the dive zooms, in powers of ten a second (as 2.5).
dlg-tour-rate = ≈ { $rate } 个数量级/秒
# A small button that puts the duration back to its default.
dlg-tour-reset = 重置
dlg-tour-reset-hover = 基于深度的默认值
dlg-tour-duration-note = 默认时长随深度而变，使深入过程保持可观看。在极深的缩放下，更快的深入可能会闪现单色帧。
# A button, after a save icon.
dlg-tour-save = 保存漫游…
# The file type's name in the save dialog.
dlg-tour-filter = Fractadyne 脚本（TOML）
# A notice. $file: the saved file's name. --render-tour is a command-line option: keep it.
dlg-tour-saved = 已保存漫游 → { $file }。可通过“工具 → 播放漫游”播放，或用 --render-tour 渲染。

## Navigate ▸ Bookmarks ▸ Manage…

dlg-bookmarks-title = 书签
# The faint text in the empty name box.
dlg-bookmarks-name-hint = 名称（可选）
# A button, after a bookmark icon.
dlg-bookmarks-add = 添加当前视图
dlg-bookmarks-empty = 还没有书签。可从当前视图添加一个。
# The tooltip of a bookmark's delete (bin) button.
dlg-bookmarks-delete-hover = 删除

## Tools ▸ Benchmark…

dlg-bench-title = 基准测试
dlg-bench-mode = 模式
dlg-bench-current = 当前设置
dlg-bench-current-hover = 使用你当前的分辨率和设置，在实时视图中播放固定的深度缩放漫游。
dlg-bench-standard = 标准化
dlg-bench-standard-hover = 固定分辨率 + 所有渲染设置，使得分可以在不同计算机之间比较。
dlg-bench-resolution = 分辨率
dlg-bench-depth = 深度
# Standard and Ultra are the two depths in the dropdown this describes.
dlg-bench-depth-hover = 两种深入都会进入深层的 floatexp/BLA 路径；“超深”（1e48×）在那里停留的时间远长于“标准”（1e32×），运行时间也相应更长。
dlg-bench-burnin = 烤机（重复）
dlg-bench-burnin-hover = 反复运行基准测试，以暴露稳定性问题和过热降频。
# The label of a slider: how many times the benchmark runs.
dlg-bench-passes = 轮数
# $size: the image size, as 1920×1080; $ss: the supersampling factor; $depth: the dive's final magnification, as 1e32.
dlg-bench-std-summary = 在屏幕外渲染：{ $size }、{ $ss }× SS、Mandelbrot/平滑迭代、60 帧深入至 { $depth }×。
dlg-bench-run = 运行

## The standardized benchmark's progress window

dlg-bench-progress-title = 正在运行基准测试…
# $resolution: the benchmark's resolution, as the dropdown names it.
dlg-bench-progress-mode = 标准化 · { $resolution }
# $pass: the pass running now; $total: how many passes there are.
dlg-bench-progress-pass = 烤机第 { $pass }/{ $total } 轮
dlg-bench-progress-dive = 正在渲染固定的深入过程…
# On the progress bar. $done: frames rendered; $total: frames in the dive.
dlg-bench-progress-frame = 帧 { $done }/{ $total }
# $fps: frames per second, as 41.3.
dlg-bench-progress-last = 上一轮：{ $fps } fps

## The benchmark's results

dlg-bench-results-title = 基准测试结果
# Buttons, each after an icon.
dlg-bench-results-run-again = 再次运行…
dlg-bench-results-save = 保存…
dlg-bench-results-none = 尚未运行过基准测试。
dlg-bench-results-saved = 基准测试结果已保存。

## The accelerated build's notice when its arithmetic libraries are missing

dlg-backend-title = 加速版本——正在使用内置算术
dlg-backend-dont-show = 不再显示

## File ▸ Gallery…

dlg-gallery-title = 图库
# A button that picks the folder to show.
dlg-gallery-folder = 文件夹…
# A button, after a refresh icon.
dlg-gallery-refresh = 刷新
dlg-gallery-empty = 此文件夹中没有 Fractadyne 图像。
# An image's title when it does not name its fractal.
dlg-gallery-untitled = Fractadyne 图像
# $zoom: the image's magnification, as 1.2e148; $saved: when it was saved, as the file gives it.
dlg-gallery-zoom = 缩放 { $zoom }   { $saved }
# The notes saved with the image, in quotation marks. Use your language's quotation marks.
dlg-gallery-notes = “{ $notes }”
dlg-gallery-open = 打开此视图
dlg-gallery-loaded = 已从图库加载视图。

## File ▸ Export image…

dlg-export-title = 导出图像
# A dropdown of image sizes.
dlg-export-size = 尺寸
# The size dropdown's text for a size set by hand. $size: the size, as 1920×1080.
dlg-export-size-custom-current = 自定义——{ $size }
# The size dropdown's first choice.
dlg-export-size-custom = 自定义…
dlg-export-size-custom-hover = 自行设置宽度；高度由下方的“宽高比”决定。
dlg-export-width = 宽度（px）
# After the width field. $height: the height in pixels, which follows from the aspect.
dlg-export-height-from-aspect = × { $height }（按宽高比）
dlg-export-ss = 超采样
dlg-export-aspect = 宽高比
# The aspect choice that keeps the window's own shape.
dlg-export-aspect-window = 与窗口一致
# Before the choices PNG and OpenEXR.
dlg-export-format = 格式：
dlg-export-hud = 位置 HUD
dlg-export-hud-hover = 在导出图像的左上角烙印一个缩放级别 + 坐标面板（随输出尺寸缩放；显示地图/Mandelbrot 视图的缩放 + 中心）。
dlg-export-glitch = 失真校正
dlg-export-glitch-hover = 用多个参考点校正微扰失真。对于极深（floatexp）的单视图导出会自动跳过，使参考轨道构建 + 渲染在后台线程中运行，程序保持响应。从不用于 Mandelbrot 或 Multibrot 视图（Julia 模式之外）：与精确计算对照后发现，它在这些视图上没有修正任何东西，却让导出时间最多变为原来的 5x。
dlg-export-deep-note = 深度导出：参考轨道在后台线程中构建（界面保持响应）；在此深度下跳过失真校正。
# A dropdown, shown in the dual (map + Julia) view, and its three choices.
dlg-export-dual = 双视图布局
dlg-export-dual-side = 并排
dlg-export-dual-separate = 分开的文件
dlg-export-dual-map = 仅地图
dlg-export-notes = 备注：
# The faint text in the empty notes box.
dlg-export-notes-hint = 与图像一起保存（≤120 个字符）
# $width and $height: the image size in pixels; $left: how many characters the notes can still take.
dlg-export-output = 输出：{ $width } × { $height } px（还可输入 { $left } 个字符）
# When the export's shape differs from the window's. $percent: how much more, as 25.
dlg-export-extra-width = 比窗口多显示 { $percent }% 的宽度（不会裁剪任何内容）
# The same, when the export is taller than the window. $percent: how much more, as 25.
dlg-export-extra-height = 比窗口多显示 { $percent }% 的高度（不会裁剪任何内容）
dlg-export-tiles-note = 在后台线程中分块渲染（无尺寸上限）。文件中嵌入了视图 + 备注，因此可以通过“文件 ▸ 打开视图”重新打开。
# A label before the button that picks the folder exports go to.
dlg-export-folder = 文件夹：
dlg-export-choose = 选择…
dlg-export-choose-hover = 选择导出的目标目录
dlg-export-open-after = 完成后打开
dlg-export-open-after-hover = 图像写入后，立即交给系统查看器打开。
# $count: the graphics cards in this machine (2 or more).
dlg-export-all-gpus = 使用所有显卡（{ $count }）
dlg-export-all-gpus-hover = 按图块将图像分配给本机的每一块显卡，每块显卡各有一份深层参考轨道的副本。相同型号的显卡得到的图像与单卡渲染完全相同；不同型号的显卡会在各个图块中把少数像素画得略有不同（实测：无可见接缝）。如果某块显卡出错，它的图块会转给其他显卡。无法拆分的图像（某些着色方法和公式）使用本窗口所用的显卡。
# The export's progress.
dlg-export-preparing = 正在准备（构建参考轨道）…
dlg-export-saving = 正在保存…
dlg-export-rendering = 正在渲染…
# $time: the time since the export started, as 8.3s or 1m 04.0s.
dlg-export-elapsed = 已用时间：{ $time }
dlg-export-cancel-hover = 关闭，不导出
# A button, after a save icon.
dlg-export-save-as = 另存为…
dlg-export-save-as-hover = 选择文件名和位置
dlg-export-export = 导出
dlg-export-export-hover = 渲染并保存到上方的文件夹（自动命名）

## Notices worded at startup (main.rs)

# A notice shown once after starting, when the saved session comes from a newer version of the app. $saved and $handled: version numbers of the session file's format (1, 2…).
app-state-newer = 此会话由更新版本的 Fractadyne 保存（状态 v{ $saved }；本版本支持 v{ $handled }）。部分设置可能不适用，保存时会以本版本的格式重写。
# The title of the notice that says the accelerated build's arithmetic libraries are found again (its text comes from elsewhere).
app-notice-mpfr-restored-title = 加速算术已恢复
# After Fractadyne restarted itself because the graphics driver reset the GPU.
app-toast-gpu-recovered = 已从图形设备重置中恢复——由于上一个位置的渲染负担过重，已在初始视图重新打开。

## Short notices (toasts)

# When the autopilot reached a view that could not be rendered and put the last good view back.
app-toast-autopilot-view-restored = 自动巡航已停止：视图变得不可用，已恢复原样（已记入日志）
# $error: the system's reason, as it gives it (often in English).
app-toast-gradients-save-failed = 无法保存渐变：{ $error }
# $error: the reason, as the program gives it (often in English).
app-toast-gradients-serialize-failed = 无法序列化渐变：{ $error }
# $name: the name the user gave the gradient.
app-toast-gradient-saved = 已保存渐变“{ $name }”。
# $error: the reason, as the program gives it (often in English).
app-toast-bookmarks-serialize-failed = 无法序列化书签：{ $error }
# $error: the system's reason, as it gives it (often in English).
app-toast-bookmarks-save-failed = 无法保存书签：{ $error }
# Jumping to a named point given by exact expressions. $name: the point's name (in English).
app-toast-coordinates-failed = { $name }：无法计算坐标
# $zoom: the magnification, already formatted (1,234.5 or 1.2e148).
app-toast-random-location = 随机位置 · { $zoom }×
# Importing a Kalles Fraktaler .kfr location. $error: the reason (in English).
app-toast-import-failed = 导入失败：{ $error }
# $zoom: the magnification, already formatted.
app-toast-location-loaded = 已加载位置 @ { $zoom }×

## Go to location: why a jump could not be made (one line per field that failed)

# "Re" is the dialog's real-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-re = Re：{ $error }
# "Im" is the dialog's imaginary-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-im = Im：{ $error }
# The imaginary field was given a complex value. $error: the evaluator's reason (in English).
app-goto-error-im-real = Im：{ $error }——请将虚部输入为实数，不要带 i
# "Zoom" is the dialog's magnification field.
app-goto-error-zoom-empty = 缩放：请输入放大倍数，例如 1e50 或 250000
# $zoom: what was typed, in quotes.
app-goto-error-zoom = 缩放：无法读取 { $zoom }——请输入正的放大倍数，例如 1e50 或 250000
app-goto-error-invalid = 输入无效——请检查坐标和缩放。

## Finding a minibrot or Misiurewicz point (the M key and the Go to dialog)

app-feature-needs-holomorphic = 迷你 Mandelbrot 查找器需要全纯分形族（Mandelbrot / Multibrot）。
app-feature-not-in-julia = 迷你 Mandelbrot 查找器搜索的是 Mandelbrot 集——请关闭 Julia 模式后再使用。
app-feature-busy = 已在查找一个特征——一次只能查找一个。
app-feature-searching = 正在查找最近的迷你 Mandelbrot…
app-feature-mandelbrot-only = 特征查找仅适用于 Mandelbrot。
app-feature-no-pair = 在搜索预算内未找到 Misiurewicz 点——该点可能比查找器在此视图下所能到达的更深，或者中心可能不在螺旋或分支点附近。
# The name of what was found, used as $feature in the messages below. $period: the minibrot's period (a whole number).
app-feature-minibrot = 周期 { $period } 的迷你 Mandelbrot
# $preperiod and $period: the point's two whole numbers (k, p). $octaves: how often the view repeats when zooming in (2 decimals); $twist: the turn per repeat in degrees.
app-feature-misiurewicz-lambda = Misiurewicz ({ $preperiod },{ $period })——每 { $octaves } 个倍频程重复一次，扭转 { $twist }°
# $preperiod and $period: the point's two whole numbers (k, p).
app-feature-misiurewicz = Misiurewicz ({ $preperiod },{ $period })
# Added after the messages below when the fixed iteration count is too low for the new depth. $have: the iterations set now; $typical: what the depth typically needs.
app-feature-iter-note = ⚠ 迭代次数固定为 { $have }；这个深度通常需要约 { $typical }——如果视图渲染成一片平坦，请提高该值（或启用自动迭代）。
# $feature: what was found (see above). $zoom and $asked: magnifications, already formatted; $octaves: the solver's depth limit.
app-feature-zoomed-capped = 已缩放到：{ $feature }——{ $zoom }×。求解的上限为 { $octaves } 个倍频程，因此停在了这里，而不是 { $asked }×。
# $feature: what was found. $zoom: the magnification, already formatted. $seconds: how long the search took (1 decimal).
app-feature-zoomed = 已缩放到：{ $feature }——{ $zoom }×（{ $seconds }s）
# $feature: what was found. $seconds: how long the search took (1 decimal).
app-feature-snapped = 已吸附到中心：{ $feature }（{ $seconds }s）
app-feature-no-minibrot = 在视图附近未找到迷你 Mandelbrot 的中心——请再靠近一些。
# $feature: the point asked for, e.g. Misiurewicz (3,1). $widths: how many view-widths away, already formatted. $octaves: how far to zoom out.
app-feature-just-outside = 在此视图之外不远处找到一个 { $feature } 点——相距 { $widths } 个视图宽度。缩小约 { $octaves } 个倍频程即可让它出现在屏幕上，或者设置更大的预周期，以查找此尺度上的特征。
# $feature: the point asked for. $widths: how many view-widths away, already formatted (may be 2^187). $octaves: how far to zoom out.
app-feature-far = 找到一个 { $feature } 点，但它相距 { $widths } 个视图宽度——该特征的尺度比此视图“大得多”。请“缩小”约 { $octaves } 个倍频程才能到达它，或尝试更大的预周期，以查找此尺度上的特征。
# $feature: the point asked for. $residual: a small number in scientific notation.
app-feature-not-preperiodic = { $feature } 与此处的轨道不符（残差 { $residual }）——清空两个输入框，即可根据视图检测这一对数值。
# $feature: the point asked for. "k/p" are the dialog's two fields.
app-feature-not-converged = 视图附近没有收敛到任何 { $feature } 点——请导航得更近一些，或尝试其他 k/p。
app-feature-bad-request = 预周期和周期都必须为正数。

## Share location dialog and location files

app-location-empty = 没有可加载的内容（或文本过大）。
app-location-not-fractadyne = 不是 Fractadyne 位置。
# $zoom: the magnification, already formatted. $note: what in the text was ignored or repaired (in English).
app-share-loaded-note = 已加载 @ { $zoom }×——{ $note }。
# $error: the system's reason (often in English).
app-share-save-failed = 保存失败：{ $error }
app-share-saved = 已保存。
app-share-saved-rendering-thumb = 已保存。正在渲染缩略图…
app-share-saved-no-thumb = 已保存——没有缩略图（无法为此视图渲染缩略图）。
app-share-saved-with-thumb = 已保存，含缩略图。
# $error: the system's reason (often in English).
app-share-saved-thumb-failed = 已保存，但无法添加缩略图：{ $error }
app-share-loaded-into-box = 已载入框中——请检查后单击“应用”。
# Reading a file failed (a location or a palette). $error: the system's reason (often in English).
app-read-failed = 读取失败：{ $error }
app-share-too-large = 文件过大（不是 .fdn 位置？）。

## File-dialog filter names (the extensions are added by the code)

app-filter-kfr = Kalles Fraktaler 位置
app-filter-fdn = Fractadyne 位置
app-filter-map = Fractint / KF 调色板
app-filter-ugr = Ultra Fractal 渐变
app-filter-ase = Adobe 色板交换文件
app-filter-ggr = GIMP 渐变
app-filter-all = 所有文件

## Importing and saving palette files (shown under the gradient editor)

# The reason a file was refused before reading it.
app-gradient-map-too-large = 文件太大，不可能是 .map 调色板
app-gradient-ugr-too-large = 文件太大，不可能是 .ugr 调色板文件
app-gradient-ase-too-large = 文件太大，不可能是 .ase 色板列表
app-gradient-ggr-too-large = 文件太大，不可能是 .ggr 渐变
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported-vga = 已从 { $file } 导入 { $count } 个条目——这是一个 6 位 VGA 表（白色为 252 而非 255），按原样保留，以便与 Fractint 自身的输出一致。
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported = 已从 { $file } 导入 { $count } 个条目，作为硬色带。
# $error: the reader's reason (in English).
app-gradient-map-invalid = 不是 .map 调色板：{ $error }
# $name: the gradient's name in the file; $file: the file's name.
app-gradient-ugr-imported = 已从 { $file } 导入“{ $name }”。
# $file: the file's name; $count: how many gradients it holds.
app-gradient-ugr-pick = { $file } 包含 { $count } 个渐变——请在下方选择一个。
# $error: the reader's reason (in English).
app-gradient-ugr-invalid = 不是 .ugr 调色板文件：{ $error }
# $used: the swatches kept; $total: the swatches in the file.
app-gradient-ase-imported-some = 已导入 { $total } 个色板中的 { $used } 个，均匀分布。
# $count: the swatches in the file.
app-gradient-ase-imported = 已导入 { $count } 个色板，均匀分布并混合。
# $error: the reader's reason (in English).
app-gradient-ase-invalid = 不是 .ase 色板列表：{ $error }
app-gradient-nothing-to-save = 还没有可保存的自定义渐变。
# $file: the saved file's name.
app-gradient-ggr-saved = 已保存“{ $file }”——中点、混合曲线和色彩空间均完整保留。
# $file: the saved file's name; $count: the segments whose curve could not be kept.
app-gradient-ggr-saved-lossy = 已保存“{ $file }”，但有 { $count } 个分段使用了 .ggr 格式无法表示的贝塞尔曲线。相关分段已按线性混合方式写入；库中的副本仍保留原曲线。
# $path: the file; $error: the system's reason (often in English).
app-gradient-write-failed = 无法写入 { $path }：{ $error }
# $name: the gradient's name; $count: its segments.
app-gradient-ggr-imported = 已导入“{ $name }”——{ $count } 个分段，包括其中点、混合曲线和色彩空间。
# $error: the reader's reason (in English).
app-gradient-ggr-invalid = 不是 .ggr 渐变：{ $error }

## The eyedropper's banner across the top of the screen while picking a colour

# $hex: the colour under the pointer, as #rrggbb.
app-eyedropper-banner-color = 正在取色  ·  { $hex }  ·  单击选取，按 Esc 取消
app-eyedropper-banner = 正在取色  ·  单击选取，按 Esc 取消

## The gradient editor window

app-gradient-title = 渐变编辑器
# Before the two view buttons, Bar and Ring.
app-gradient-view = 视图
app-gradient-view-bar = 条形
app-gradient-view-bar-hover = 一条直的色带——最适合精确放置色标
app-gradient-view-ring = 环形
app-gradient-view-ring-hover = 之所以是环形，是因为调色板是“循环”使用的：顶部的接合处就是渲染器每次循环都会跨过的接缝，在条形视图中看起来没问题的渐变，在那里可能会出现生硬的边缘。
app-gradient-add-lane-hover = 单击这条线上的任意位置，即可在该处添加色标。+ 会在最宽的间隙中添加一个。
app-gradient-strip-hover = 拖动标记可移动其色标 · 拖动色带本身可“旋转”整个渐变，在接缝处循环 · 双击可添加色标，右键单击标记可将其删除 · 方向键可微调所选项（按住 Shift 可更细微地调整）。
app-gradient-remove-selected-hover = 删除所选色标
app-gradient-ring-hover = 拖动圆环本身可“旋转”整个渐变——越过接缝的部分会从另一侧绕回来 · 拖动标记可移动其色标 · 单击外圈轨道可在该处添加色标，或单击中间的 ⊕ 在最宽的间隙中添加一个 · ⊖ 删除所选色标。顶部的空心标记就是接缝，即循环调色板回绕的地方；它固定不动，因为它“就是”接合处，颜色在它下方转动。
app-gradient-empty = 还没有自定义渐变——“复制预设…”、导入，或下方已保存的渐变都可以作为起点，其色标和各分段的曲线会显示在这里。
# $count: the entries of an imported palette, more than the editor shows stop by stop.
app-gradient-too-many = 已导入 { $count } 个条目——太多，无法逐个色标编辑。请使用上方的预览，或用“复制预设”重新开始。
# The selected segment's heading. $index: its number; $total: how many there are; $left and $right: where it starts and ends, 0.000–1.000.
app-gradient-segment-heading = 第 { $index } 段，共 { $total } 段  ·  { $left } → { $right }
# A label before the list of blend curves.
app-gradient-curve = 曲线
app-gradient-curve-hover = 颜色在此分段中变化的快慢。“弯曲”的形状由中点决定——中点为 0.50 时恰好是线性的。
# The blend curves (GIMP's kinds), in a narrow list: keep them short.
app-gradient-blend-linear = 线性
app-gradient-blend-curved = 弯曲
app-gradient-blend-sine = 正弦
app-gradient-blend-sphere-increasing = 球面 ↑
app-gradient-blend-sphere-decreasing = 球面 ↓
app-gradient-blend-bezier = 贝塞尔
# Hover on Bézier in the list of blend curves.
app-gradient-bezier-exact-hover = 两个可以在两个方向上拖动的控制点。此曲线本身已是三次曲线，因此转换不会改变任何东西。
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-visible-hover = 两个可以在两个方向上拖动的控制点。转换会“近似”当前曲线——这一条的误差在 { $error } 以内，肉眼明显可见。
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-barely-hover = 两个可以在两个方向上拖动的控制点。转换会“近似”当前曲线——这一条的误差在 { $error } 以内，几乎看不出来。
# A label before the list of colour spaces (RGB, HSV).
app-gradient-space = 色彩空间
# A label before the Bézier presets beside it.
app-gradient-ease = 缓动
app-gradient-ease-hover = 预设会写入两个控制点；如需其他效果，请在曲线上拖动它们。贝塞尔曲线没有中点——由它的控制柄决定在何处到达一半。
# Bézier presets, as in CSS easing: short buttons.
app-gradient-ease-linear = 线性
app-gradient-ease-in = 缓入
app-gradient-ease-out = 缓出
app-gradient-ease-in-out = 缓入缓出
app-gradient-midpoint = 中点
app-gradient-midpoint-hover = 混合到达一半的位置，以占此分段的比例表示。0.50 为居中。
# A button that puts the midpoint back in the middle of the segment.
app-gradient-centre = 居中
app-gradient-centre-hover = 恢复为 0.50
app-gradient-no-hue = ⚠ 此分段有一端没有色相
app-gradient-no-hue-hover = 有一端是黑色、白色或灰色，因此色相扫掠从红色开始，走遍整个色轮。这是合法的，有时也正是想要的效果——但这正是中间可能出现两端都没有的颜色的原因。曲线下方的色带显示了它的效果。
app-gradient-segment-start-hover = 此分段“起点”处的颜色
app-gradient-segment-end-hover = 此分段“终点”处的颜色
app-gradient-bytes-hover = 红、绿、蓝，取值 0–255
app-gradient-floats-hover = 同一颜色，取值 0–1，这是渲染器存储的形式
app-gradient-canvas-bezier-hover = 所选分段的混合曲线。拖动任一控制点——同时在 x 和 y 方向上——即可改变其形状。浅色线条表示 0 和 1；曲线可以越出它们，此时被截断的是颜色而不是曲线。
app-gradient-canvas-hover = 所选分段的混合曲线：颜色从左侧色标过渡到右侧色标的快慢。拖动圆环可移动中点，或将曲线切换为贝塞尔，以获得可在两个方向上移动的控制点。
# The selected stop's heading. $index: its number; $total: how many there are.
app-gradient-stop-heading = 第 { $index } 个色标，共 { $total } 个
app-gradient-stop-color-hover = 此色标处的颜色——单击色块可选取颜色，或在旁边输入十六进制值
app-gradient-stop-bytes-hover = 红、绿、蓝，取值 0–255——即此颜色写入时的字节
# The eyedropper button.
app-gradient-pick = 取色
app-gradient-pick-hover = 吸管——从屏幕上任意位置取色。将指针移到想要的颜色上并单击；按 Esc 取消。
# Written before the stop's position (0.000–1.000) in its number field: a short abbreviation of "position".
app-gradient-stop-pos = 位置
app-gradient-nudge-hover = 微调此色标（按住 Shift 可更细微地调整）
app-gradient-remove = 删除
app-gradient-remove-hover = 删除此色标（或单击它下方的 ⊖）
app-gradient-add-stop = 添加色标
app-gradient-add-stop-hover = 拆分最宽的分段（或单击渐变上方的线）
# A menu of the built-in palettes, to start from a copy of one.
app-gradient-copy-preset = 复制预设…
app-gradient-paste = 粘贴…
app-gradient-paste-hover = 从十六进制颜色或 0–255 的 RGB 三元组导入调色板
app-gradient-save-ggr = 保存 .ggr…
app-gradient-save-ggr-hover = 将此渐变写入 GIMP .ggr 文件——GIMP、Krita、Inkscape 以及本编辑器都能读取
# A menu of the palette file formats (.map, .ugr, .ggr, .ase).
app-gradient-import = 导入
app-gradient-import-map-hover = 加载 Fractint / Kalles Fraktaler 的 .map 调色板文件（R G B 行，每个条目一行）
app-gradient-import-ugr-hover = 加载 Ultra Fractal 的 .ugr 渐变文件（一个 .ugr 文件通常包含许多渐变——由你选择其中一个）
app-gradient-import-ggr-hover = 加载 GIMP 的 .ggr 渐变，完整保留其中点、混合曲线和色彩空间
app-gradient-import-ase-hover = 加载 Adobe 色板列表（.ase）。色板没有位置信息，因此导入后会均匀分布并混合。
# The gradient's name, for the library of saved gradients.
app-gradient-name = 名称
# Shown greyed in the empty name field.
app-gradient-name-hint = 我的渐变
# The save button, when a gradient of that name is already saved.
app-gradient-update = 更新
app-gradient-save = 保存
app-gradient-save-hover = 将此渐变保存到你的库中，完整保留其曲线和色彩空间
app-gradient-save-needs-name-hover = 请先为渐变命名
# A menu of the saved gradients. $count: how many there are.
app-gradient-saved-menu = 已保存（{ $count }）
app-gradient-nothing-saved = 还没有保存任何内容
app-gradient-delete-saved-hover = 删除此已保存的渐变
# $count: the gradient's segments.
app-gradient-rich = 丰富渐变：{ $count } 个分段带有中点、混合曲线或色彩空间——超出了普通色标列表所能容纳的内容。
app-gradient-convert = 转换为可编辑的色标
app-gradient-convert-hover = 用普通色标列表替换它。这会“丢弃”色标列表无法容纳的中点、混合曲线和色相扫掠。
# Above the list of gradients in an Ultra Fractal file. $count: how many; $file: the file's name.
app-gradient-ugr-list = { $file } 中的 { $count } 个渐变
app-gradient-close-list = 关闭列表
app-gradient-seamless = 无缝（终点与起点一致）
app-gradient-seamless-hover = 调色板是循环使用的，因此位置 1 和位置 0 在屏幕上是相邻的。开启后，最后一个色标的颜色会保持与第一个相同——环形视图在顶部显示接合处。
app-gradient-ends-differ = ⚠ 两端不同
app-gradient-ends-differ-hover = 渐变终点的颜色与起点不同，因此循环调色板在回绕处会出现生硬的边缘。
app-gradient-hard-bands = 硬色带（条目之间不混合）
app-gradient-hard-bands-hover = Fractint / .map 的语义：每个条目都是一种纯色。关闭后会将它们混合成平滑的渐变。
app-gradient-paste-intro = 粘贴十六进制颜色（#ff8800）或 0–255 三元组（255 136 0），以逗号、空格或换行分隔：
# Applies the pasted colours.
app-gradient-apply = 应用
# Empties the paste box.
app-gradient-clear = 清空
# $used: the colours kept; $total: the colours pasted; $limit: the most that are kept.
app-gradient-paste-imported-some = 已导入 { $total } 种颜色中的 { $used } 种，在你的列表中均匀采样（上限为 { $limit }）。
# $count: the colours pasted.
app-gradient-paste-imported = 已导入 { $count } 种颜色。
# $error: why the pasted text could not be read (in English).
app-gradient-paste-unreadable = 无法读取：{ $error }
# A hint at the bottom of the editor. $count: the palette's entries.
app-gradient-bands-hint = { $count } 个色带 · 按文件顺序使用；开启色带时忽略位置。
# A hint at the bottom of the editor, in the ring view. $count: the gradient's stops.
app-gradient-ring-hint = { $count } 个色标 · 单击外圈轨道可在该处添加 · ⊕ 在最宽的间隙中添加，⊖ 删除所选色标 · 拖动标记可移动它 · 顶部的标记是接缝
# A hint at the bottom of the editor, in the bar view. $count: the gradient's stops.
app-gradient-bar-hint = { $count } 个色标 · 单击渐变上方的线可在该处添加 · 拖动标记可移动它 · ⊖ 或右键单击可删除
app-gradient-cancel = 取消
app-gradient-cancel-hover = 放弃自此窗口打开以来所做的全部更改，并关闭窗口
app-gradient-ok = 确定
app-gradient-ok-hover = 保留这些更改并关闭编辑器

## Shared

# The button that closes the Help window and the "Faster deep zoom" window.
app-close = 关闭

## The "Faster deep zoom" window (Help menu): the optional accelerated build

app-accel-title = 更快的深度缩放
app-accel-active-heading = 你正在运行加速版本
app-accel-active-body = 深度缩放参考轨道正在用 MPFR/GMP 计算，在这一步上比标准版本快 2.5-6.4x——也就是深度视图开始显现之前的那段停顿。
# $status: which arithmetic library is in use and its version (in English).
app-accel-in-use = 正在使用的算术库：{ $status }
app-accel-fallback-heading = 加速版本——正在使用内置算术
app-accel-fallback-body = 这是加速（MPFR）版本，但找不到它的数学库，因此正在使用内置算术（astro-float）。图像完全相同；只是深度视图显现之前的停顿更长。
# Followed by the folder the program runs from, on its own line.
app-accel-fallback-dlls = 要使用更快的路径，请将 libmpfr-6.dll 和 libgmp-10.dll（以及 libgcc_s_seh-1.dll、libwinpthread-1.dll）放在 fractadyne.exe 旁边，然后重新启动：
app-accel-warning-hidden = 有关此情况的启动警告已隐藏。
app-accel-show-again = 再次显示
app-accel-offer-heading = 一个可选版本在深处快 2.5-6.4x
app-accel-offer-body = 深度缩放的大部分时间花在用 CPU 计算参考轨道上——也就是深度视图开始显现之前的那段停顿。一个可选版本改用 MPFR/GMP 库来完成这项工作，速度快 2.5 到 6.4 倍，而且越深越快。其他一切都相同。
app-accel-offer-identical = 图像“逐字节完全相同”——这是由更快的库计算的同样的数学，已在每一个分形公式以及整个深度缩放对比语料库上验证过。
app-accel-offer-shared = 你的设置、已保存的会话和位置在两个版本之间共享，因此可以随意切换，无需导入任何东西。
# Shown on Windows.
app-accel-separate-windows = 它需要单独下载，因为它所用的库无法用标准 Windows 版本所用的编译器构建，而且这些库采用与 Fractadyne 本身不同的许可证（GNU LGPL v3）。
# Shown on other systems.
app-accel-separate = 它需要单独下载，因为它所用的库采用与 Fractadyne 本身不同的许可证（GNU LGPL v3）。
app-accel-download = 下载适用于此版本的加速版
app-accel-all-releases = 所有发布版本
app-accel-all-releases-hover = 如果直接链接返回 404，说明此版本还没有加速版——请在这里选择最接近的一个。
# Shown on Windows.
app-accel-install-windows = 解压后从该文件夹运行 fractadyne.exe，并让 .dll 文件保留在它旁边。
# Shown on Linux.
app-accel-install-linux = 解压后从该文件夹运行 ./fractadyne。它使用系统中的 GMP 和 MPFR（libgmp10 / libmpfr6），几乎所有桌面系统都已安装——请参阅软件包中的 README-ACCELERATED.txt。
# Shown on other systems.
app-accel-install-other = 此平台尚未构建加速软件包；可以使用 --features fractadyne-core/rug 从源代码构建。

## The Help window's frame (its sections are under help-)

app-help-title = Fractadyne 帮助

## Checking for updates

app-toast-up-to-date = 你使用的已是最新版本。
# $error: why the check failed (in English).
app-toast-update-failed = 检查更新失败：{ $error }
# The window that offers a newer version.
app-update-title = 有可用更新
# $version: the new version (0.3.0-beta.20). $channel: the token "stable" or "beta", in English; a translation can choose its words by it ({ $channel -> [beta] … *[other] … }).
app-update-version =
    Fractadyne { $version }（{ $channel ->
        [beta] Beta 版
       *[other] 稳定版
    }）现已推出
# $current: the version running now. $track: the update track the user follows (its name in the language in use).
app-update-running = 你正在运行 { $current }  ·  { $track }
app-update-download-intro = 从 GitHub 下载最新版本：
app-update-download = 从 GitHub 下载
app-update-later = 稍后提醒我

## The second graphics card (shown in its window, after "In use:" or "Could not start:")

# The second device is the same card as the window's. $name: the card's name.
app-worker-same-card = 同一块显卡，{ $name }
# Why the second device could not start.
app-worker-thread-stopped = 设备线程已停止

## Export status (shown in the Export image window)

app-export-gpu-unavailable = GPU 不可用
app-export-reference-aborted = 导出失败：参考轨道构建已中止。

## The performance panel (View ▸ Performance panel)
## Its row labels sit in a monospace column about 14 characters wide: keep them short (a longer label pushes its value to the right).

app-perf-fps = FPS
app-perf-fps-hover = 每秒呈现的帧数，根据平滑后的帧间隔计算。视图正在稳定时，此处改为报告细化进度——正在稳定的视图并不是以这里的数字所暗示的速率运行的。
# What the FPS row shows while a view is settling. $done and $total: tiles; $eta: the time left, as m:ss.
app-perf-refining-tiles = 细化 { $done }/{ $total }（约 { $eta }）
app-perf-refining-arming = 细化（准备中）
# $percent: how much is done, with one decimal.
app-perf-refining-percent = 细化 { $percent }%
app-perf-building-reference = 正在构建参考轨道
app-perf-idle = 空闲
app-perf-frame = 帧
app-perf-frame-hover = 平滑后的帧间墙钟时间——并不是下方各行之和。它包含等待呈现的时间，因此在已稳定的空闲视图上，它反映的是重绘节奏，而不是任何工作量。
app-perf-cpu = CPU
app-perf-cpu-hover = 一次更新中主线程所用的时间：参数设置和参考轨道缓存管理。参考轨道重建时，出现尖峰的正是这一行，而这里的尖峰就是你感觉到的卡顿。
app-perf-gpu-idle = GPU/空闲
app-perf-gpu-idle-hover = 帧时间减去主线程 CPU 时间。视图渲染时，这是 GPU 的工作时间；但在已稳定的视图上，它大部分是空闲等待——名称中同时写了两者，是因为仅凭这个减法，面板无法区分它们。
app-perf-timing = 计时
# How frame cost is measured, the timing row's value. TIMESTAMP_QUERY is a name.
app-perf-timing-starved = 墙钟（时间戳断供）
app-perf-timing-gpu = GPU 时间戳
app-perf-timing-no-query = 墙钟（无 TIMESTAMP_QUERY）
app-perf-timing-hover = 帧开销的测量方式。GPU 时间戳是精确的来源；当计时数据不再到达时，会改用墙钟时间作为后备，而按墙钟时间定价的预算曾多次把深度帧的大小估错——这里显示红色时，请理解为“安全余量目前只是估计值”。
app-perf-mode = 模式
app-perf-mode-hover = 上一帧渲染所用的算术路径：direct（无参考轨道）、df32 微扰，或 floatexp——大约 1e28x 之后使用的扩展范围路径，也是唯一运行 BLA 的路径。
app-perf-eff-iter = 有效迭代
app-perf-eff-iter-hover = 上一帧实际使用的迭代次数，已计入深度缩放以及移动时施加的上限——而不是“质量”中的基础设置，那只是起点。
app-perf-precision = 精度
app-perf-precision-hover = 计算参考轨道所用的尾数位数。它随深度增长；64 位是浅层的下限，并不是你碰到的上限。
app-perf-orbit-len = 轨道长度
# $cap: the most orbit samples this GPU holds, already formatted, or the text of app-perf-uncapped.
app-perf-orbit-len-hover = 参考轨道的样本数。此 GPU 最多容纳 { $cap }——接近这个值就是实际的深度墙：超过它，实时视图就无法解析需要更长轨道的更深位置。
app-perf-uncapped = （无上限）
app-perf-sa-skip = SA 跳过
app-perf-sa-skip-hover = 在逐像素计算开始之前，级数近似跳过的迭代次数。越高越快，而且不会改变图像的任何内容；如果没有跳过任何迭代，则不显示此行。
app-perf-aa = 抗锯齿
app-perf-aa-hover = 超采样：每个像素在每个轴上用这么多个样本渲染，因此 2x 需要四倍的工作量。
app-perf-dual = 双视图
app-perf-dual-hover = 拆分的 Mandelbrot/Julia 视图是否正在运行。为 true 时，此面板上的每一项开销都同时涵盖“两个”窗格。
app-perf-zoom = 缩放
app-perf-zoom-hover = 相对于完整视图的放大倍数。上面的算术模式在大约 1e4x 处切换一次，在大约 1e28x 处再切换一次。
app-perf-iterate = 迭代
# $target and $lethal: times in milliseconds, already formatted.
app-perf-iterate-hover = 上一次迭代派发的实测 GPU 时间。控制器的目标是 { $target } ms；持续接近 { $lethal } ms 的读数，正是这类硬件曾发生设备丢失的区间。
app-perf-steps = 步/秒
app-perf-steps-hover = GPU 上每秒的迭代步数，以十亿计。以内部区域为主的视图，每步的速度比单看计数所暗示的更慢，因此这是一个吞吐量读数，而不是评分。
app-perf-budget = 预算
# After the budget's value while it is still converging; the hover below quotes it.
app-perf-budget-settling = （稳定中）
app-perf-budget-hover = 自适应的每次派发步数预算，是控制器防止单帧过长、以致驱动程序重置 GPU 的保护措施。“稳定中”表示它仍在根据稀疏的读数收敛，尚不能作为可信的上限。
app-perf-rss = 常驻内存
# $mb and $peak: memory in megabytes, already formatted.
app-perf-rss-value = { $mb } MB（峰值 { $peak }）
app-perf-rss-hover = 进程的常驻内存，以及本次会话中达到的最高值。在实际使用中，深层参考轨道构建的峰值曾超过 2 GB，而屏幕上没有任何其他地方提示这一点；超过系统内存的四分之三时显示为琥珀色。
app-perf-gpu-est = GPU 估计
app-perf-gpu-est-hover = 估计的 GPU 驻留内存：参考轨道、BLA 树、迭代纹理、细化运行期间的分块状态乒乓缓冲，以及呈现门控快照。根据已知的分配大小汇总得出；驱动程序的真实数字无法以可移植的方式读取。
app-perf-julia-re = julia c.re
app-perf-julia-re-hover = Julia 窗格正在渲染的参数 c 的实部——即光标所在的点，除非已钉住。
app-perf-julia-im = julia c.im
app-perf-julia-im-hover = Julia 窗格正在渲染的参数 c 的虚部。
app-perf-julia-c = julia c
# The julia c row's value when the parameter is held fixed.
app-perf-pinned = 已钉住
app-perf-pinned-hover = 参数固定在一个点上，因此移动光标不再改变 Julia 窗格。
app-perf-c-panel = c/面板
app-perf-c-panel-hover = 整个 Mandelbrot 面板所跨越的 c 空间宽度。当它远小于一个 Julia 像素时，悬停会改变 c，但 Julia 看起来没有变化——这在深度缩放时是正常的，并不是卡死。
app-perf-ref-recompute = 参考重算
app-perf-ref-recompute-hover = 最近一次参考轨道重新计算的耗时。这就是深度视图开始显现之前的停顿，也是可选的加速版本所加速的那一步。
app-perf-recompute-rate = 重算/秒
app-perf-recompute-rate-hover = 每秒“安装”的参考轨道——即结果实际被使用的那些。请与下一行比较：两者之差就是被丢弃的工作。
app-perf-builds-rate = 参考构建/秒
app-perf-builds-rate-hover = 每秒“启动”的参考轨道构建，包括结果被丢弃的那些——深入过程永远到达不了的预取位置，或被拒绝的扩展。这些构建会占用全部 CPU，而且不会在其他任何地方显示：曾有一次预取空转，在六分钟内每秒运行约 400 次构建，而“重算/秒”显示的是 2。个位数是正常的；这两行之间差距很大，意味着有工作被丢弃。
app-perf-recompute-total = 重算总数
app-perf-recompute-total-hover = 启动以来参考轨道重算的次数。可用于计算已知时间间隔内的速率；在没有任何移动时这个数字仍在攀升，才是值得追查的信号。

## Help: the contents list (Help & reference…, F1)

# Each section's name in the contents list (about 165 px wide: keep them short), and its heading.
help-section-overview = 概览
help-section-navigation = 导航
help-section-expressions = 坐标表达式
help-section-options = 着色与选项
help-section-fractals = 分形
help-section-how-it-works = 工作原理
help-section-command-line = 命令行
help-section-shortcuts = 快捷键
help-section-hardware = 推荐硬件
# Credits to the prior work Fractadyne builds on.
help-section-acknowledgments = 致谢
# The third-party license notices.
help-section-licenses = 许可证
help-section-about = 关于

## Help ▸ Overview

help-overview-intro = Fractadyne 是一款原生分形探索器，专为超深缩放和速度而打造。它绘制“逃逸时间”分形——通过在每个像素上反复执行一个简单公式而生成的图像。
help-overview-what-is = 什么是逃逸时间分形？
help-overview-what-is-text = 对于每个像素，程序从零开始，一遍又一遍地执行一个公式，例如 z → z² + c。如果这个不断变化的值永远保持很小，该像素就属于集合，绘制为深色。如果它最终无限增长（“逃逸”），该像素就在集合之外，其颜色记录了逃逸所用的步数。“保持”与“逃逸”之间那条无限精细的边界，就是分形。
help-overview-can-do = 你可以做什么
help-overview-can-pan = 几乎无限制地平移和缩放（在任何深度下位置都是精确的）。
help-overview-can-families = 在 25 个内置分形族（十个，其中五个还有更高次幂）、你自己的公式、生命游戏和 L 系统之间切换，并可将 Newton 以外的任意逃逸时间分形族显示为 Julia 集。
help-overview-can-recolor = 使用预设或自定义渐变以及多种着色方法重新着色。
help-overview-can-lighting = 添加 3D 浮雕光照和发光的边界轮廓。
help-overview-can-minibrots = 吸附到迷你 Mandelbrot、为地点添加书签，以及导出高分辨率图像。
help-overview-can-autopilot = 让自动缩放的自动巡航自行朝细节深入。
help-overview-can-share = 将任意地点以小巧的“.fdn”文本片段或文件的形式分享。
help-overview-can-tours = 运行脚本化的漫游和硬件基准测试。
help-overview-first-steps = 入门
# "Seahorse Valley" is a place in the Navigate menu: use its name as that menu shows it. A and F1 are keys.
help-overview-first-steps-text = 打开“导航”菜单并选择“Seahorse Valley”，然后滚动鼠标滚轮放大。拖动即可平移。或者直接按 A，让自动巡航替你深入。随时按 F1 即可回到此帮助。

## Help ▸ Coordinate expressions
## The expressions, numbers and function names here are typed as shown in every language:
## keep them exactly as written. Menu items in quotation marks are the menus' own labels.

help-expr-intro = 凡是 Fractadyne 接受坐标的地方，你都可以输入表达式来代替小数：“导航 → 转到位置…”的 Re / Im 字段（以及其极坐标模式下的每个字段）、.fdn 位置文件中的 center_re / center_im 行、漫游脚本中的 re / im，以及命令行上的 --center。表达式会以缩放所需的位数求值，并且“转到位置”对话框会将它与视图一起保存——因此，以 1/3 或 -0.5 + 0.25*cos(pi/4) 的形式输入的点，无论放大多深，都会精确地停留在该点上，而小数一旦位数用尽就会发生漂移。
help-expr-examples = 示例
help-expr-ex-third = 精确的三分之一。放大时会显示更多位数。
help-expr-ex-neck = Seahorse Valley 的颈部——周期 2 芽苞的根点。
help-expr-ex-circle = 以 -0.5 为圆心、半径为 1/4 的圆上的一个点（极坐标模式会替你组合出这个表达式）。
help-expr-ex-bulb = 周期 3 芽苞根点的 Re；其 Im 为 sin(2*pi/3)/2 - sin(4*pi/3)/4。“导航 → 精确点（表达式）”中内置了这些点。
help-expr-ex-complex = 在 Re 字段中输入一个完整的复数值，会同时填入 Re 和 Im。
help-expr-ex-combine = 函数和常数可以自由组合。
help-expr-ex-powers = 幂：2^-20 等于 1/1048576。-2^2 等于 -4——负号在求幂之后才应用。
help-expr-ex-root = -2，即实数立方根。负数的偶次方根会被拒绝；sqrt(-1) 得到 i。
help-expr-ex-scientific = 表达式中也可以使用科学计数法。
help-expr-numbers = 数字和运算符
# The decimal point in an expression is '.' in every language: keep that rule as it is.
help-expr-numbers-decimals = 小数，可带指数：0.25、-1.5e-30、.5。只能包含数字——数字内部不能有空格或千位分隔逗号，并且小数点始终是“.”。
help-expr-numbers-imaginary = 在数字后加上 i 后缀表示虚数（0.25i），或单独使用 i：3 + 4*i。
# Starts with the operators themselves, two spaces apart.
help-expr-numbers-operators = +  -  *  /  以及 ^（幂；从右向左求值，因此 2^3^2 = 2^9）。括号用于分组。没有隐式乘法：请写 2*pi，而不是 2pi。
help-expr-numbers-case = 名称不区分大小写：PI、Pi 和 pi 是同一个常数。
help-expr-constants = 常数
help-expr-functions = 函数
help-expr-functions-angles = 角度以弧度为单位。如需使用角度制，请乘以 pi/180——cos(45*pi/180)——或者将“转到位置”对话框切换到极坐标模式，并选择“度”作为单位。
help-expr-refused = 哪些会被拒绝，以及原因
help-expr-refused-fractional = 负数的分数次幂，例如 (-2)^(1/2)：结果是复数，而且有多个根可供选择。请用 root(x, n) 或 cbrt(x) 求实根，或用 sqrt(x) 求主复数根。
help-expr-refused-complex = 向实值函数传入复数参数（sin(i)、ln(1+i)）。只有 sqrt 和 abs 接受复数参数；算术运算符和整数次幂可用于复数值。
help-expr-refused-domain = 定义域错误：对非正数取 ln 或 log，asin 或 acos 的参数超出 -1…1，0 的负数次幂，除以零。
help-expr-refused-bounds = sin、cos、tan、exp 或 ^ 的参数超出 ±2^32，以及嵌套超过 32 层：粘贴的坐标是不可信的输入，而这些情况会耗费无限长的时间。
help-expr-refused-grammar = 不支持变量、用户自定义函数、比较或条件语句。语法刻意保持精简，以确保保存的位置在每个版本中都表示同样的含义。
help-expr-no-parse = 如果无法解析
help-expr-no-parse-text = 消息会指出是哪个字段、在第几个字符处发现了什么，并且——对于常见的笔误——告诉你应该输入什么：从文档中粘贴来的印刷体减号（−）、× 或 ÷、小数逗号、数字与名称之间缺少 *、未闭合的括号，或者函数没有写括号。任何内容都不会只读一半：只要文本中有任何部分无法理解，整个坐标就会被拒绝，而不是被悄悄截短。

## Help ▸ Navigation
## Menu paths (Navigate → "Go to location…") and quoted items name the interface's own
## menus and controls: use their translations. Key names (Ctrl, Shift, Space, Esc) stay.

# A heading: the mouse controls (also in Shortcuts).
help-mouse = 鼠标
# The left column of a controls table: a gesture or key. Keep key names (Shift, Space, Ctrl) as they are.
help-key-left-drag = 左键拖动
help-nav-left-drag = 平移视图。
help-key-mouse-wheel = 鼠标滚轮
help-nav-wheel = 以光标为中心放大/缩小；滚轮转得越快，每一格缩放得越多。开启“缩放吸附到特征”时，在迷你 Mandelbrot 附近放大，缩放会被拉向它。
help-key-shift-drag = Shift+拖动
help-nav-shift-drag = 框选缩放——拖出一个矩形，它就会缩放到填满视图。
# $icon: the click-to-zoom toolbar button's symbol.
help-key-click-zoom = 单击缩放（{ $icon }）
help-nav-click-zoom = 在工具栏中切换目标按钮（单视图）：之后左键单击会按设定的倍数深入该点，右键单击则后退。拖动仍为平移；在右侧面板的“导航”下选择倍数（2–100×）。Backspace 可撤销一次单击。
help-nav-continuous = 连续缩放与回到初始视图
help-key-hold-space = 按住 Space
help-nav-hold-space = 以光标为锚点平滑放大。
help-key-hold-shift-space = 按住 Shift+Space
help-nav-hold-shift-space = 平滑缩小。
help-nav-continuous-text = “初始视图”按钮会以动画方式平缓地飞回完整视图。右侧面板中的“缩放速度”设置连续缩放的速率。
help-nav-autopilot = 自动巡航
help-nav-autopilot-text = 按 A（或“工具 → 自动缩放（自动巡航）”）即可开始无需动手的深入：它会在分形边缘上选取细节最密集的一个点（优先选择视图中部），然后滑向该点，同时将其缓缓移到屏幕中心。只要目标仍然有趣，它就会保持该目标；当出现更丰富的结构时，它会平滑地转向那里。任何导航输入——或 Esc——都会使其停止。右侧面板（或“工具”菜单下）的“自动缩放优先级”决定它优先考虑什么：“速度”按设定的速率缩放，一旦有带细节的帧准备好就刷新屏幕；“质量”只显示完全解析的全分辨率帧，并在需要时放慢缩放，使画面在下一帧到来之前永远不会被放大太多。无论哪种方式，屏幕都不会显示空白或单色的帧：尚未解析出任何细节的刷新，会一直隐藏在上一帧后面。“自动缩放目标”决定它瞄准什么：“细节”（视图中最丰富的边缘，随深入不断重新选择）或“Misiurewicz 点”——它会找到最近的 Misiurewicz 点，在保持静止的同时将其求解到深入上限的深度，然后精确地深入其中；这种点周围的结构每隔几个倍频程就会重复，永无止境，因此这样的深入永远不会走进死胡同（仅限 Mandelbrot 集）。
help-nav-history = 历史记录与精确移动
help-nav-undo = 撤销到上一个视图。
help-nav-redo = 重做。
help-nav-goto = “导航 → 转到位置…”可让你读取、输入、粘贴或复制精确的中心和缩放（全精度，任意深度）。中心字段接受的不只是普通数字：还有“1/3”这样的分数，以及由 pi、tau、e、sqrt 和 sin/cos/tan 构成的表达式——因此“-0.5 + 0.25*cos(pi/4)”是有效的坐标。表达式会与视图一起保存，并在每个深度下以所需的精度重新求值，因此以这种方式输入的点，无论放大多深都保持精确（四舍五入的小数最终做不到这一点）。极坐标模式（偏移 + r ∠ θ，单位为度 / 弧度 / 圈）会替你组合出这样的表达式。函数和常数的完整列表及示例，见本帮助的“坐标表达式”一节（对话框中的 ? 按钮可打开它）。“导航 → 书签”用于保存和调出位置。
help-nav-exact-points = “导航 → 精确点（表达式）”会跳到以精确方式定义、而非以四舍五入的小数定义的点——主心形线上各芽苞的根点，即每个周期 q 芽苞的附着处，以 cos/sin 表达式写出。由于表达式随视图一起保存，你可以放大其中一个点，而永远不会偏离它。
help-nav-share = “导航 → 分享位置…”会把整个视图——分形、全精度中心、缩放和着色——捕获为一段紧凑的“.fdn”片段，你可以复制它、再粘贴回来并单击“应用”，或将其作为文件保存/加载，以精确重现某个地点（或发送给别人）。
help-nav-open = “文件 → 打开视图或位置…”可直接打开以下任何一种：导出的 PNG/EXR（恢复其中嵌入的视图）、“.fdn”位置文件，或 Kalles Fraktaler 的“.kfr”。
help-nav-finding = 寻找细节
help-nav-minibrots = 迷你 Mandelbrot 是隐藏在边界上的整个集合的微小副本。大致将一个迷你 Mandelbrot 置于中心，然后按 M（或“工具 → 查找并缩放到迷你 Mandelbrot”），即可用牛顿法精确吸附到其中心、读出其周期，并直接缩放到它自身的尺度——往往一步就跨越许多个数量级。如果你已经比该迷你 Mandelbrot 更深，则只移动中心。
help-nav-minimap = “视图 → 小地图概览”会显示整个集合的小地图，带有“你在这里”标记和当前缩放深度；单击地图即可跳到某个区域。

## Help ▸ Coloring & options
## Names in quotation marks are the controls' own labels in the right-hand panel.

help-options-intro = 以下所有选项都在右侧面板中（并在会话之间保留）。
help-options-palette = 调色板
help-options-palette-text = “调色板”用于选择预设渐变、你的自定义渐变或双色模式。“编辑渐变…”会打开一个编辑器，其中每个色标都有颜色和位置（0–1）；可以添加色标、复制一个预设作为起点，或导入调色板文件。
help-options-paste = 渐变编辑器中的“粘贴…”可导入你在别处找到的调色板。它接受带或不带前导 # 的十六进制颜色（#ff8800、ff8800 或简写的 #f80），以及“255 136 0”这样的普通 0–255 RGB 三元组——即 Fractint 和 Kalles Fraktaler 的 .map 文件所用的格式——以逗号、空格或换行分隔。最多保留 256 种颜色；更长的列表会均匀采样到 256 种，并保留第一种和最后一种，使渐变的形状得以保留。
help-options-import-map = 旁边的“导入 ▸ .map…”用于加载 Fractint 或 Kalles Fraktaler 的 .map 调色板文件。这类文件是按迭代次数索引的查找表，条目之间没有混合，由此产生的生硬色阶正是经典 Fractint 风格的由来——因此它们就按这种方式导入，并提供一个“硬色带”复选框，可将同样的颜色混合成平滑的渐变。Fractint 自己的文件存储的是乘以四的 6 位颜色值，因此其白色是 252 而不是 255；Fractadyne 在导入这类文件时会提示这一点，并完全按原样保留这些值，因为 Fractint 自己保存的图像也带有同样的 252。
help-options-duotone = “双色调”将数值映射到平滑的双色渐变（阴影 → 高光）。“二值（集合）”是平涂的双色视图——集合内的点用一种纯色，集合外用另一种，没有渐变——这是观察集合形状最清楚的方式。
help-options-cycle = “循环”设置渐变在迭代范围内重复的次数（色带更密或更疏）。“偏移”旋转整个渐变。
help-options-normalize = “归一化深层颜色”会使调色板适配视图中实际存在的逃逸值范围。深度视图跨越数十万次迭代，如果不这样做，固定的循环会在相邻像素之间让渐变回绕数千次，把正确的细节变成噪点。
help-options-log-scale = “对数颜色刻度”按逃逸值的对数而非线性方式分布调色板。越深入，逃逸次数越向高端聚集，因此线性适配会把大部分渐变花在紧贴边界的一层薄壳上，其余部分几乎是平的；对数刻度会把它展开，并在缩放视频中随着范围增大，让颜色保持更稳定。
help-options-animate = “动画”让颜色随时间循环——“正向”、“反向”或“往返”会移动偏移；“随机”会持续合成平滑变形、和谐的渐变。“速度”控制每秒的循环（或变形）次数；在“随机”模式下，“随机更换渐变”会生成一个新的渐变。
help-options-method = 着色方法——数据如何变成颜色
# The coloring methods, by the names the Coloring panel gives them, each with what it does.
help-method-smooth = 平滑迭代
help-method-smooth-text = 按逃逸时间生成经典的连续色带。
help-method-stripe = 条纹平均
help-method-stripe-text = 由轨道角度生成流动的色带（“条纹密度”滑块）。
help-method-triangle = 三角不等式
help-method-triangle-text = 根据每一步落在上下界之间的位置，生成细腻的纹理。
help-method-trap = 轨道陷阱
help-method-trap-text = 轨道到某个形状（点/十字/圆）的距离；也会为内部着色。
help-method-distance = 距离估计
help-method-distance-text = 按到边界的远近着色。
help-method-decomposition = 分解
help-method-decomposition-text = 由最终逃逸角度生成的格子。
help-options-relief = 3D 浮雕光照
help-options-relief-text = 根据边界的斜率（导数）为表面着色，呈现浮雕、受光照的效果。“光照角度”设置方向；“浮雕”设置强度（越低越锐利）；“旋转光源”使其动起来。仅限全纯分形族（Mandelbrot / Multibrot）。
help-options-glow = 距离辉光
help-options-glow-text = 明亮的轮廓色带，在边界附近变密，形成发光的细丝。“辉光”是混合量，“条带宽度”是间距，“辉光动画”让它们流动起来。
help-options-quality = 质量与迭代
help-options-quality-text = “迭代次数”是将像素视为在集合内部之前的最大步数；“自动调整”会在你缩放时自动提高它（更深的细节需要更多迭代）。视图稳定后，“抗锯齿”会对静态图像进行超采样（2×–8×），消除外部细碎的“尘点”。
help-options-other = 其他
help-options-other-text = “缩放速度”设置连续缩放的速率；帧率上限限制帧率；“两者联动”并排显示 Mandelbrot 集及其 Julia 集（光标实时设置 Julia 参数）。

## Help ▸ Fractals
## Fractal family names (Mandelbrot, Multibrot, Tricorn, Burning Ship, Celtic, Buffalo,
## Phoenix, Newton) are names: keep them as written, as the formulas. Formula syntax
## (z = z^3 - p1*z + c, sqr, |z|, if / elseif / endif, p1-p5, maxit) is typed as shown.

help-fractals-intro = 除非另有说明，每个分形族都按如下方式迭代一个公式：z 从 0 开始，c 由像素决定（逃逸时间）。z = x + iy 是一个复数。
help-fractals-mandelbrot = z → z² + c。最初的那一个。该集合由所有轨道保持有界的 c 组成；它是连通的，其边界皱褶得如此厉害，以至于豪斯多夫维数为 2。
help-fractals-multibrot = z → z^d + c，幂次 d = 3、4、5。更高的幂次会增加瓣：该集合具有 (d−1) 重旋转对称性（Multibrot 3 为 2 重，4 为 3 重，5 为 4 重）。
help-fractals-tricorn = z → conj(z)² + c，其中 conj(x + iy) = x − iy。共轭使它成为反全纯的，从而具有 3 重对称性和标志性的弯曲“爪子”。
help-fractals-burning-ship = z → (|x| + i|y|)² + c——先取绝对值再平方（实部 = x²−y²+cx，虚部 = 2|xy|+cy）。非解析；深度缩放会显现出类似船只的结构（传统上上下颠倒观看）。
help-fractals-celtic = 类似 Mandelbrot，但对实部取绝对值：实部 = |x²−y²| + cx，虚部 = 2xy + cy。会产生心形和盾形的图案。
help-fractals-buffalo = 对 z² 的两部分都取绝对值：实部 = |x²−y²| + cx，虚部 = |2xy| + cy——介于 Celtic 和 Burning Ship 之间。
# A heading: the families at powers above 2 (the Fractal menu's "Higher powers").
help-fractals-powers = 高次幂
# ("Burning Ship  3 4 5") is how the Fractal menu shows such a row: keep it as it is.
help-fractals-powers-text = Multibrot 6、7 和 8（z^d + c），以及幂次为 3、4 和 5 的 Burning Ship、Tricorn、Celtic 和 Buffalo：同样的折叠，只是幂次更高。Burning Ship d 先对 z 的各部分取绝对值再求幂；Tricorn d 先取共轭；Celtic d 和 Buffalo d 折叠 z^d 的实部（对于 Buffalo，还有虚部）。“分形”菜单和下拉列表中每个分形族占一行（“Burning Ship  3 4 5”）。每一个都像其幂次为 2 的同类那样进行深度缩放，而 Multibrot 6、7 和 8 也和 Multibrot 3–5 一样支持级数近似和迷你 Mandelbrot 查找器（M）。Multibrot 8 的逃逸半径为 128 而不是 256，因为在 256 时它的下一步会超出单精度范围。
help-fractals-phoenix = z → z² + c + p·z(n-1)，其中 z(n-1) 是上一次的迭代值，p 是常数（p = -0.5）。这个记忆项会产生火焰般的细丝。
help-fractals-newton = 用牛顿法求 z³ − 1 = 0 的根：z → z − (z³−1)/(3z²)。像素不按逃逸时间着色，而是按迭代收敛到三个单位立方根中的哪一个（吸引域）以及收敛的快慢着色；纠缠在一起的吸引域边界就是分形。
help-fractals-julia = Julia 模式
help-fractals-julia-text = 除 Newton 之外的每个分形族都可以切换到 Julia 集：不再让 z 从 0 开始、c = 像素，而是固定 c（一个参数），让 z 从像素开始。对于这个固定的 c，Julia 集就是保持有界的起点与逃逸的起点之间的边界。在“两者联动”中，将光标移过 Mandelbrot 面板可实时设置 c。
help-fractals-deep = 深度缩放支持
help-fractals-deep-text = Mandelbrot、Multibrot 3/4/5 和 Tricorn 支持极深（floatexp）微扰深度缩放。Burning Ship、Celtic 和 Buffalo 是非解析的（它们要取绝对值），因此使用考虑符号的微扰；它现在也能在 floatexp 范围内运行，深度缩放远远超出旧的约 1e28× df32 上限（绝对值折叠附近偶尔出现的噪点，有待多参考点失真校正来解决）。Phoenix 使用双项微扰，也能在 floatexp 范围内深度缩放（不带 SA/BLA，因此比 Mandelbrot 更吃力）。高次幂以同样的方式深度缩放，每个幂次的微扰遵循它自己的平方链；Multibrot 3–8 也和 Mandelbrot 一样支持级数近似和 BLA，而各幂次的 Tricorn、Burning Ship、Celtic 和 Buffalo 支持 BLA（无级数近似），由 BLA 跨过深度视图的内部，而不是一步一步地迭代。只有 Newton 使用直接路径，清晰度可达约 1e6×；自定义公式可以深度缩放，除非下文“自定义公式”一节另有说明。
help-fractals-custom = 自定义公式
help-fractals-custom-text = “分形 ▸ 自定义公式…”接受以 Fractint 风格表达式写成的公式迭代步：z = z^3 - p1*z + c，或多条语句（t = sqr(z)，然后 z = t*t + c）。z 是迭代值，c 是像素（Julia 模式下为 Julia 常数），p1-p5 是参数；|z| 是模的平方，与 Fractint 中一样，并提供常用函数（sqr、abs、conj、real、imag、cabs、flip、recip、ident、exp、log、sqrt、sin、cos、tan、cotan、sinh、cosh、tanh、cotanh、asin、acos、atan、asinh、acosh、atanh、cosxx，以及对每一部分分别取整的 floor、ceil、trunc 和 round）。log 是自然对数，与 Fractint 和复分析中一样：即许多教科书写作 ln 的函数（没有以 10 为底的对数）。用 ln、cot、coth、Re 或 Im 写成的公式无法读取；语法检查会指出应使用的函数，一个按钮可以改写它，输入 ln 时会建议 log。每一步都通过微扰进行深度缩放，在 1e28x 之后使用与内置分形相同的扩展范围算术，但含有指数随 z 或 c 变化的幂（例如 z^c）的公式除外：它在直接路径上渲染，直到 c 的一个单精度步长跨越一个像素之前都是清晰的，大约为 1e4x 到 1e5x，在远离原点处则更浅。公式对话框会说明适用哪种情况。log、sqrt 和非整数次幂取其主值，主值在负实轴两侧会发生跳变。微扰会精确地跟随这种跳变，但如果某个像素的轨道在参考轨道没有跨越的地方跨越了割线，那么从此以后它只能以大约单精度继续计算，因此轨道跨越割线的深度视图会在那里出现噪点（实测：sqrt(z^4 + c) 在实轴上 1e5x 时是干净的，log 公式即使在 1e4x 时也有噪点）。每一步都完整迭代（不使用级数近似），因此迭代次数很高的深度视图，渲染速度比内置分形慢。在边界附近，含有函数的公式的轨道是混沌的，而那里的长轨道在单精度下只能被粗略地跟随：在 2,000 次迭代时，sin(z) + c 有三分之一的像素与精确轨道不同，在 60 次时几乎没有。视图、会话和导出的图像都会保存公式的文本。
help-fractals-custom-sections = Fractint 的分段也同样可用。“:”之前的语句是初始化段，每个像素运行一次；作为最后一条语句的比较是公式自己的逃逸测试，条件成立时继续迭代（|z| <= 100 && |z - 1| > 0.000001 会让逃逸或收敛到 1 的轨道停止）；if (...)、elseif (...)、else 和 endif 用于选择语句；在一步中先于赋值被读取的变量，其值来自上一步（来自初始化段，或为 0）。比较运算比较的是实部，结果为 1 或 0，&& 和 || 用于组合它们，赋值本身也是一个值（a = b = pixel）。pixel、p1-p5、pi 和 e 可以被赋值，在被赋值之前保持各自的值；maxit 是迭代次数，ismand 为 1（公式的 Mandelbrot 形式）。使用了以上任何一项或取整函数的公式，在直接路径上渲染，不支持深度缩放。“公式库 > 导入”也能读取 Fractint 的 .frm 文件。每个能读取的条目都会成为一个公式，其所在行会说明读取时做了哪些改动：fn1-fn4 取 Fractint 的默认值（sin、sqr、sinh、cosh）；Fractint 的变量 c 被重命名为 c_，因为这里 c 表示像素；没有被任何语句设置的名称为 0，与 Fractint 中一样；名称内部的空白会被去掉（end if 即 endif），与 Fractint 的做法一样。导入消息会统计无法读取的条目数，并给出最常见的原因：不支持屏幕和视图变量（whitesq、scrnpix、scrnmax、center、magxmag、rotskew）、lastsqr 和随机数。“合集”中的 Fractint 经典公式（Magnet、Nova、Spider、Manowar 等）就是用这些分段写成的。
help-fractals-custom-editor = 在公式字段中，括号按嵌套层级显示不同的颜色；光标旁的那一对括号会高亮显示，没有配对的括号显示为红色（注释中的括号不计在内，因为解析器会忽略它们）。名称会在输入时自动补全：输入两个字母后，会出现一个列表，列出以这两个字母开头的函数、名称和你自己的变量。Up 和 Down 用于选择，Tab 或 Enter 选取其中一个（函数会连同它的左括号一起插入），Esc 关闭列表；在完整输入一个名称之后按 Enter，仍然会另起一行。
help-fractals-custom-textbook = “教科书”（公式上方的开关）以教科书排版的方式显示公式，并可就地编辑。按书面写法输入即可：/ 把它前面的项变成分数，^ 变成指数（在指数末尾输入 +、- 或 = 会退出指数），( 输入括号，或者在函数名后输入则成为函数调用，| 输入竖线，括号内的逗号表示复数常数。方向键在公式中移动，Up 和 Down 在分子与分母之间移动，Tab 移到下一个空框；在分数或一对括号的开头按 Backspace 会选中它，再按一次则删除它并保留其中的内容。复制、粘贴和撤销的用法与文本字段中相同，名称补全的方式也与那里相同，键盘面板会输入到排版后的公式中（□² 对光标前的内容求平方，函数键会把选中的内容包起来）。每次编辑都会写回文本，因此两种视图始终保存同一个公式；你没有编辑过的行会保留其间距和注释。记法从不改变计算的内容：|z| 是模的平方，显示为 |z|²，cabs(z) 显示为 |z|，exp(z) 仍为 exp(z)。无法读取的行显示为红色；单击它即可作为文本编辑。“LaTeX”会按排版的样子把公式复制为 LaTeX 源码；开启“教科书”时，公式库也会以排版形式显示其中的公式。
help-fractals-custom-numbers = 公式中的数字（0.1、1/3、pi、参数）按双精度值读取，大约 16 位有效数字，深层参考轨道正是用这些值进行迭代的。因此，两个在纸面上相等的公式，在深处可能会不同：0.1*z 乘以的是最接近 0.1 的双精度数，而 z/10 除以的是精确的 10，两幅图像在约 1e15x 之后就会分道扬镳（实测：1e15x 时 5% 的像素不同，1e17x 时三分之一，1e20x 时全部）。两幅图像对各自的公式而言都是正确的；请写出你想要分享的那一个。
help-fractals-custom-library = 对话框中的“保存到库”会以一个名称保存公式及其参数（以库中已有的名称保存会更新该条目）。“分形 ▸ 公式库…”列出这些公式，可以应用、编辑、导出或删除；“导入”会添加公式文件中的公式而不替换任何已有公式：库中已有的公式会被跳过，名称已被占用的新公式会变成“名称 (2)”。库就是“文件与数据”（关于）下所列文件夹中的 formulas.toml，它本身就是一个公式文件。漫游关键帧以 formula = "..." 和 formula_params = [[re, im], ...] 的形式携带公式，“工具 > 从当前视图创建漫游”会为自定义视图写出它们。
help-fractals-custom-collection = Fractadyne 附带一个包含 45 个公式的合集，专为它编写：Burning Ship 和 Celtic 族的高次幂和折叠变体、混合公式与两步混杂公式、透过另一种映射看到的 Mandelbrot 集（反演、对数极坐标、经由正弦）、超越公式和有理公式，以及 Julia 集。它们就是对话框中的“示例”和公式库中的“合集”选项卡，在那里，“复制到我的公式”会把其中一个添加到你的公式中。每个公式都有一个起始视图，“应用”会跳到那里：在初始视图上打开的公式，往往并不在它的图像所在之处。在显示公式时保存它，同样会保留你当前所在的视图，“应用”会回到那里。

## Help ▸ How it works

help-how-escape = 逃逸时间与平滑着色
help-how-escape-text = 每个像素反复迭代公式，直到其模超过逃逸半径或达到迭代上限。仅凭原始的步数会产生生硬的色带；加上由最终模推导出的小数部分，就能得到连续、无色带的颜色。
help-how-precision = 任意精度的位置
help-how-precision-text = 普通的 64 位数字在约 1e15× 缩放时就会用尽位数。Fractadyne 以任意精度保存视图中心，位数随缩放而增加，因此位置永远不会退化。像素尺度同样以扩展指数保存，因此也不会停在 64 位约 1e308× 的极限——深度只受耐心限制，而不受任何固定数值范围的限制。
help-how-perturbation = 微扰
help-how-perturbation-text = 以高精度迭代每一个像素会慢得无法接受。取而代之的做法是：在 CPU 上以高精度迭代一个参考像素（“参考轨道”），其他每个像素则在 GPU 上以快速的低精度、作为与它的微小差值 δ 来计算：δz → 2·Z·δz + δz² + δc。
help-how-floatexp = 极深（floatexp）
help-how-floatexp-text = 大约超过 1e28× 之后，即使是那个微小的差值也会在 32 位范围内下溢，因此它被存储为尾数加上一个单独的整数指数（“floatexp”），从而突破 32 位的深度墙。引擎会自动切换：浅层时直接计算，深层时用微扰，最深时用 floatexp。于是深度受坐标精度和迭代预算的限制，而不是一个固定的极限——已在一个包含 39 个位置的参考语料库上与 Fraktaler-3 交叉核对，最深为 5.63e18003×（38 个非极端行是常规关卡，最深到 6.13e1105×），另有一套内部精度自洽测试，深达 1e1000000×；附带的漫游可达约 1.3e420×。
help-how-reference = 参考点选择与重定基
help-how-reference-text = 选择参考点（以高精度评分）时，会让它的轨道尽可能长时间地留在视图之内。当差值变得过大时，它会被“重定基”回参考轨道上，以保持准确。
help-how-series = 级数近似
help-how-series-text = 在深度缩放时，最初的迭代几乎不会让 δ 偏离参考轨道，因此可以一次性算出：δz 用一个关于 δc 的短多项式（3 阶）来近似，其系数沿参考轨道迭代得出。渲染器会直接跳到该多项式对整个视图仍然准确的最后一次迭代，然后再正常迭代——省下跳过的步骤，图像毫无变化（在“控制面板 ▸ 高级”中切换）。适用于 Mandelbrot 和 Multibrot 3–8，最深的范围；不适用于条纹 / 三角不等式 / 轨道陷阱 / 分解着色（它们需要每一次迭代）。在 BLA 生效的地方，它已经包含了这种早期跳过。
help-how-distance = 距离估计与光照
help-how-distance-text = 跟踪导数 dz/dc 可以得到每个像素到边界的距离。3D 浮雕光照、距离辉光和“距离”着色方法都由它驱动——在任何缩放深度下都有效。
help-how-safety = 抗锯齿与安全
help-how-safety-text = 视图稳定后，静态图像会进行超采样（每个轴 2–8×）。在极端设置下，工作预算会通过降低分辨率（而绝不降低迭代次数）把单次 GPU 绘制控制在驱动程序的看门狗时限之内，因此深度视图会保持细节，而不会变成空白。

## Help ▸ Command line (the flags themselves stay English)

# The section's introduction. The flags listed below it are the command line itself, English in every language.
help-cli-intro = Fractadyne 可以无界面运行，用于渲染、影片/漫游导出、自动化、黄金图像检查和基准测试。不带任何参数时，它会启动交互式图形界面。

## Help ▸ Shortcuts

# What each mouse gesture and key does (the gestures are the Navigation section's).
help-shortcuts-pan = 平移
# A gesture: turning the mouse wheel.
help-key-wheel = 滚轮
help-shortcuts-wheel = 以光标为中心缩放
help-shortcuts-box-zoom = 框选缩放（拖出一个矩形即可缩放到该矩形）
# $icon: the click-to-zoom toolbar button's symbol. "tool on": while that button is on.
help-key-click-tool = 单击（{ $icon } 工具开启时）
help-shortcuts-click = 深入该点；右键单击后退
# A heading: the keyboard shortcuts.
help-shortcuts-keyboard = 键盘
help-shortcuts-space = 连续放大 / 缩小
help-shortcuts-undo = 撤销视图
help-shortcuts-redo = 重做视图
help-shortcuts-minibrot = 查找并缩放到迷你 Mandelbrot（牛顿-拉弗森法）
help-shortcuts-autopilot = 自动缩放的自动巡航（朝细节深入；任何输入都会使其停止）
# off / strip / panel / full: the panel's sizes, as View ▸ Live render panel names them.
help-shortcuts-live-panel = 实时渲染面板：关闭 / 条带 / 面板 / 完整（将鼠标悬停在仪表上可了解其含义；单击其 × 可关闭面板）
help-shortcuts-esc = 停止自动巡航 / 正在播放的漫游，或退出全屏
help-shortcuts-snapshot = 快照到上次的导出文件夹——按屏幕显示的样子截取视图，或按导出设置完整渲染；第一次按下时会询问选哪一种，可在“文件 → 设置 → 快照”中更改。
help-shortcuts-help = 打开此帮助
# A heading: the shortcuts of the Game of Life, with the menu path to it.
help-shortcuts-life = 生命游戏（分形 → 元胞自动机 → 生命游戏）
help-shortcuts-life-run = 运行 / 暂停宇宙
help-shortcuts-life-step = 前进一代 / 按步长前进（在生命游戏面板中设置）
help-shortcuts-life-draw = 开启 / 关闭绘制工具（工具栏上的铅笔）
# A gesture. "Draw on": while the Life draw tool is on.
help-key-click-drag-draw = 单击 / 拖动（绘制开启时）
help-shortcuts-life-paint = 翻转一个细胞 / 涂画：从死细胞开始的笔画会绘制，从活细胞开始的笔画会擦除。正在运行的宇宙在你绘制时会暂停，松开后继续运行。
# $key: a keyboard shortcut (Ctrl+Z). "Draw on": while the Life draw tool is on.
help-key-when-drawing = { $key }（绘制开启时）
help-shortcuts-life-undo = 撤销上一次编辑、填充、清空或规则更改

## Help ▸ Recommended hardware

help-hardware-intro = Fractadyne 把工作分给 GPU（逐像素的迭代 + 着色）和 CPU（深度缩放所依赖的任意精度参考轨道）。两者都很重要，但作用不同。
help-hardware-gpu = GPU（最重要）
help-hardware-gpu-text = 每个像素都在 GPU 上迭代和着色，因此 GPU 的速度决定了帧率和导出时间。具有强大 32 位（单精度）吞吐量的独立显卡能带来最流畅的深度缩放。集成显卡也能正常工作——只是更慢，而且会更早达到每帧的工作预算（此时渲染器会降低分辨率，而不是细节）。需要现代图形 API：Vulkan、Direct3D 12、Metal，或通过 wgpu 使用 OpenGL。
# The left column of a requirements table.
help-hardware-minimum = 最低配置
help-hardware-gpu-minimum = 任何支持 Vulkan / DX12 / Metal / GL 3.3 的 GPU。
help-hardware-recommended = 推荐配置
help-hardware-gpu-recommended = 独立显卡（近 5 年左右）；>= 2 GB 显存。
help-hardware-big-exports = 用于大尺寸导出
help-hardware-gpu-big-exports = >= 4 GB 显存（分块渲染，因此性能下降平缓）。
# A heading: the processor.
help-hardware-cpu = CPU
help-hardware-cpu-text = 高精度参考轨道（以及 BLA 表）在 CPU 上计算，并且大部分是单线程的，因此单核性能最为重要——它决定了你移动之后深度视图显现的速度。额外的核心主要有助于后台导出编码。需要 64 位 CPU。
help-hardware-cpu-recommended = 单核速度良好的现代 64 位 CPU。
help-hardware-memory = 内存
help-hardware-memory-text = 深层参考轨道随深度增长（高精度下的大量迭代）；超大导出会将图块保存在内存中。8 GB 足以满足日常探索；16 GB 以上适合极深的深度和非常大的导出。
help-hardware-checking = 检查你的硬件
help-hardware-diagnostics = “帮助 → 诊断…”会在你的计算机上运行开发者所用的同样测试。“运行自检”会在大约十五秒内将数学计算和渲染结果与已知正确的结果对照；“运行界面测试”会在几个缩放深度下遍历界面和实时视图，并截取屏幕截图；“运行 GPU 算术检查”会在你的计算机提供的每一个图形后端上验证着色器的双浮点算术，只需几秒钟，而且不打开窗口。每项测试都作为单独的进程运行，因此扰乱显卡驱动程序的测试只会让测试本身崩溃，而不会影响你的会话。
help-hardware-gpu-check = GPU 算术检查没有通过或失败之分——它记录的是你的特定 GPU 和驱动程序如何计算，在某些硬件上（迄今试过的每一块 NVIDIA 显卡），它会报告着色器编译器丢弃了部分扩展精度算术。这是我们正在尝试跨厂商摸清的一项真实发现，而不是你计算机的故障，因此无论它说什么，把它附加到报告中都确实有帮助。
help-hardware-report = 如果看起来有什么不对，请运行自检并使用“附加到问题报告…”：这样报告就会带有一个来自你自己硬件、经机器验证的结果，这比单纯的描述有用得多。有一点值得了解——参考图像是在一块特定的显卡上生成的，而不同的 GPU 计算出的像素本来就会略有不同，因此在其他硬件上，图像比较会以更宽的容差进行检查，那里出现的细小差异是预期之中的，而不是故障。
help-hardware-notes = 说明
help-hardware-note-watchdog = 性能足够的 GPU 可以避免在极端设置下触发看门狗工作预算。
help-hardware-note-binary = 本程序是单一的独立可执行文件——无需安装任何运行时。
help-hardware-note-benchmark = 运行内置的基准测试（“工具 ▸ 基准测试”）来评估你的计算机。

## Help ▸ Acknowledgments & citations
## Names of people, programs, papers, books and websites stay as written, and so do the
## titles in quotation marks (the works' own titles); the prose around them translates.

help-ack-title = 致谢与引用
# The dedication, in italics at the top of the section.
help-ack-dedication = 献给 Stone Soup Group——Fractint 背后的志愿者们（始于 1988 年）。
help-ack-dedication-text = 他们无私分享的代码，以及开放、协作探索分形的精神，正是我们许多人爱上这门艺术的原因。Fractadyne 正是秉承这一传统而构建的。
help-ack-intro = Fractadyne 建立在分形社区数十年工作的基础之上。它所借鉴的技术和先前成果——已对照以下来源核实：
help-ack-deep-zoom = 深度缩放算法
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-martin-title = 微扰与级数近似——{ $who }
help-ack-martin = 参考轨道 + 低精度差值方法（δz → 2Z·δz + δz² + δc），以及跳过早期迭代的 n 阶级数，由 SuperFractalThing 及其说明文档“SuperFractalThing Maths”（sft_maths.pdf，2013）提出。
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bla-title = 双变量线性近似（BLA）与重定基——{ $who }
help-ack-bla = 通过合并的线性映射跳过成串的迭代，并重定基到单个参考轨道以避免失真；在 fractalforums.org 上开发（2021）。
# A link's label. $who: the linked page's author.
help-ack-bla-link = BLA 详解——{ $who }
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-glitch-title = 失真检测判据——{ $who }
help-ack-glitch = 用 |Z + z| << |Z| 判断像素何时需要另一个参考点——这是无失真微扰的基础（fractalforums）。
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-diffabs-title = 非解析（Burning Ship）微扰——{ $who }
help-ack-diffabs = 针对绝对值的“diffabs”分情况分析，使微扰适用于基于绝对值的分形族（Burning Ship / Celtic / Buffalo），发表于 fractalforums。
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mathr-title = 参考实现与交叉核对——{ $who }
help-ack-mathr = Fraktaler-3 和 Kalles Fraktaler 2+（最初由 Karl Runmo 编写）——它们的文档指导了本引擎的设计，Fractadyne 也用它们的 EXR 输出来交叉核对自己的渲染器。
# A heading: credits for the coloring methods.
help-ack-coloring = 着色
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-harkonen-title = 平滑迭代计数与条纹平均——{ $who }
help-ack-harkonen = 连续（归一化）逃逸时间着色和条纹平均方法，出自“On Smooth Fractal Coloring Techniques”（硕士论文，2007），该论文还分析了三角不等式 / 分支平均着色。
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mitchell-title = 三角不等式平均——{ $who }
help-ack-mitchell = 对每个 z² + c 落在三角不等式界限内的位置取平均，生成平滑的、火焰般的外部纹理。
# A link's label: the person's tutorial pages. $who: the person's name.
help-ack-mitchell-link = { $who }——教程
help-ack-distance = 距离估计着色/光照使用经典的 dz/dc 边界距离方法（见 Peitgen 与 Saupe，“The Science of Fractal Images”，1988）。
help-ack-lsystems-life = L 系统与生命游戏
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-lindenmayer-title = L 系统——{ $who }
help-ack-lindenmayer = 并行字符串重写，于 1968 年提出，用于模拟植物和藻类的生长（“Mathematical models for cellular interactions in development”，Journal of Theoretical Biology）。
help-ack-abop = L 系统读取器所遵循的，正是这本书（1990）中的海龟解释，以及随机、上下文相关和参数化系统的记法；标有 ABOP 的内置系统都是根据书中的插图录入的。
# A link's label: the book can be read free on that website. $site: the website's name.
help-ack-abop-link = 这本书，可从 { $site } 免费获取
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bourke-title = L 系统合集——{ $who }
help-ack-bourke = 许多内置系统来自他的 L 系统页面，其中包括 Chris Wallace 的 Sierpinski 曲线规则和 Dietmar Saupe 的灌木。可直接打开的 Fractint .l 文件采用的是 Stone Soup Group 的格式。
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-conway-title = 生命游戏——{ $who }
help-ack-conway = 元胞自动机 B3/S23（1970），通过 Martin Gardner 在 Scientific American 上的“Mathematical Games”专栏（1970 年 10 月）走向世界。
# A credit's heading: the website, then who runs it. $site: the website's name.
help-ack-lifewiki-title = { $site }——ConwayLife.com 社区
help-ack-lifewiki = 以下内容的参考资料：本程序所读取的规则记法（B/S、von Neumann、Generations、Alan Hensel 用于非总和型规则的字母，以及 MAP）、它与 Golly 共用的 RLE 图样格式，以及内置的规则和图样——它记录了这些规则和图样的发现者。
help-ack-foundations = 基础与工具
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mandelbrot-title = Mandelbrot 集——{ $who }
help-ack-mandelbrot = 这一切的核心对象。
help-ack-built-with = 使用 Rust、egui/eframe（emilk）和 wgpu（gfx-rs）构建；任意精度由 astro-float 提供，双字浮点数由 twofloat 提供；PNG/OpenEXR、rfd、serde。感谢它们的作者和维护者。
help-ack-errors = 署名中的任何错误均由作者负责；欢迎在项目的问题跟踪器上提出更正。

## Help ▸ Third-party licenses

help-licenses-title = 第三方许可证
# $licenses: Fractadyne's license expression (MIT OR Apache-2.0), as written. $url: the web address of a crate's source. THIRD-PARTY-NOTICES.md is a file name, `option-ext` a crate's name, MPL-2.0 a license's.
help-licenses-intro = Fractadyne 以静态链接的可执行文件形式分发，其中包含以下开源 crate 和随附的字体。完整的许可证文本（也包含在随下载提供的 THIRD-PARTY-NOTICES.md 中）如下。Fractadyne 自身的代码采用双重许可：{ $licenses }。`option-ext` 采用 MPL-2.0——其未经修改的源代码位于 { $url }。
# A button, after a copy icon: copies every notice to the clipboard.
help-licenses-copy-all = 复制全部声明

## Help ▸ About

help-about-tagline = 一款使用 Rust 和 wgpu 构建的原生分形探索器。
# The arithmetic deep zoom has used. $ran: the libraries that have run so far, as names (or a note that none has yet); $built: those this build contains, with versions.
help-about-arithmetic = 深度缩放算术：{ $ran }（本版本包含：{ $built }）。
help-about-files = 文件与数据
# Followed by the folder's path.
help-about-files-text = 你的会话（当前视图、着色和偏好设置）、书签及其缩略图、已保存的渐变和已保存的公式都存储在这个按用户划分的文件夹中，并在下次启动时重新加载：
# Tooltip of the copy button beside the folder's path.
help-about-copy-path = 复制路径
# $command: a command typed in a terminal (it stays English).
help-about-session = 会话文件带有版本号；如果它是由更新版本的 Fractadyne 写入、当前版本无法完全读取的，程序会尽力加载并向你发出警告。要完全重新开始，请使用“文件 ▸ 重置应用程序状态…”（会先询问），或在终端中运行 `{ $command }`。
help-about-license = 许可证
# $licenses: the license expression (MIT OR Apache-2.0), which stays as written.
help-about-license-text = { $licenses }——你可以任选其中一种许可证使用。
# A link to the source code, followed by ↗.
help-about-source = GitHub 上的源代码

## The live render panel ("blinkenlights": View ▸ Live render panel, key B)
## A front panel drawn over the view like a piece of 1970s test equipment. Its legends are engraved
## in capitals: write them in capitals here (the panel shows them exactly as written, it does not
## uppercase them). Numbers, units and readouts on its tubes and dials are not translated.

# Tooltip of the × button in the panel's top-right corner. The code adds the key after it: (B).
live-close-hover = 关闭实时渲染面板
# The legend engraved along the top plate, left of the maker's badge.
live-plate = 实时渲染 · 前面板

## Live render panel: instrument titles
## Engraved above each instrument, and repeated as the title of its help card.

# Above the depth readout, whose tubes show the magnification as a power of two ("2^" stays).
live-title-depth = 深度 2^
live-title-iterations = 迭代
# Above the frame-time readout (milliseconds).
live-title-frame-ms = 帧耗时 ms
# Above the date-and-time readout.
live-title-clock = 日期 · 时间
# The walk dial: a radar screen showing a slow refresh rendered pass after pass, band after band.
live-title-walk = 步进 · 区段
# A meter of how far the picture on screen lags the view.
live-title-age = 画面滞后
# What part of the picture this frame's GPU pass covered.
live-title-tiles = 图块
# A scope of the GPU's work per frame, in milliseconds.
live-title-scope = GPU 渲染遍 · ms
# The depth dial: depth, the reference orbits being built, and the arithmetic in use.
live-title-dial = 深度 · 参考 · 算术
# Iteration steps per second.
live-title-steps = 步 / 秒
# The two GPUs (the window's and a second, worker GPU).
live-title-engines = 双引擎
live-title-memory = 内存
live-title-cores = 核心 · 线程
live-title-pixels = 像素流
# Help card titles only: the three GPU timer lamps (T0, T1, T2 stay as they are) and the row of lamps.
live-title-timers = GPU 计时器 T0–T2
live-title-bank = 指示灯组

## Live render panel: the light bank's lamps
## Engraved under each lamp, and the title of its help card. A cell is narrow (about eight capital
## letters): a longer label breaks at a space onto two lines, and is drawn smaller if it still does
## not fit.

live-lamp-plan = 规划
live-lamp-dispatch = 派发
live-lamp-live = 实时
live-lamp-held = 保持
# The last picture reprojected (stretched) to fit the moving view.
live-lamp-reproj = 重投影
live-lamp-reading = 读数
# A pass of a pinned walk.
live-lamp-pin-pass = 钉住遍
live-lamp-split = 拆分
live-lamp-adopt = 采用
live-lamp-abandon = 放弃
# "REF": a reference orbit. Built for the view as it is (LIVE), ahead of it (AHEAD), or for a tour's next hold (HOLD).
live-lamp-ref-live = 实时参考
live-lamp-ref-ahead = 超前参考
live-lamp-ref-hold = 停留参考
# The series approximation.
live-lamp-series = 级数
# The second (worker) GPU.
live-lamp-worker = 辅助
live-lamp-tile = 图块
# Supersampling (anti-aliasing); usually kept as SSAA.
live-lamp-ssaa = SSAA
live-lamp-palette = 调色板
# The arithmetic in use (direct, df32, floatexp).
live-lamp-mode = 模式

## Live render panel: the kinds of GPU pass
## In the GPU scope's legend (MOVING, PIN, WORKER, SETTLE, SSAA). SETTLE and SSAA also name a still
## view's walk on the walk dial.

# A moving frame's pass.
live-kind-moving = 移动
# A pass of a pinned walk.
live-kind-pin = 钉住
# The worker GPU's passes.
live-kind-worker = 辅助
# A still view's settle tile or walk pass.
live-kind-settle = 稳定
# A supersampling sample.
live-kind-ssaa = SSAA

## Live render panel: legends and readouts on the instruments

# On the walk dial during a split refresh. $done: passes done; $passes: passes in all.
live-walk-split = 拆分 { $done }/{ $passes }
# On the picture-age meter's face, under its scale (0 to 2 octaves).
live-age-scale = 落后倍频程
# The depth dial's band for direct (plain floating-point) arithmetic. The other bands, DF32 and FLOATEXP, are names and stay.
live-dial-direct = 直接
# Under the two GPU gauges: GPU 0 draws the window; GPU 1 is the worker, or there is none.
live-engines-gpu-window = GPU 0 · 窗口
live-engines-gpu-worker = GPU 1 · 辅助
live-engines-no-worker = 无辅助
# The share of displayed frames each GPU made. $window and $worker: percentages, 0–100.
live-engines-on-screen = 显示占比 { $window } : { $worker }
# The same readout before any frame was shown.
live-engines-on-screen-none = 显示占比 —
# Under the CPU core bars. $count: the machine's cores.
live-cores-count = { $count } 核
# Under the threads scope. $count: the app's busy threads.
live-cores-threads = 线程 { $count }

## Live render panel: the memory tanks' legend
## What the GPU memory holds, by kind. Very narrow: about ten letters.

# G-buffers (the picture's working textures).
live-memory-gbuf = G 缓冲
# The held picture and the supersampling accumulator.
live-memory-hold = 保持+SSAA
# The reference orbit and its BLA (bilinear approximation) table.
live-memory-orbit = 轨道+BLA
# The worker GPU's copy of the reference.
live-memory-worker = 辅助参考

## Live render panel: the roles on the threads scope
## Tiny labels on a small scope: at most four letters each.

# The reference builders.
live-cores-role-ref = 参考
# The scoring pool.
live-cores-role-pool = 评分
# The thread driving the worker GPU.
live-cores-role-worker = 辅助
# Probes: short side renders.
live-cores-role-probe = 探测
# Image and tour exports.
live-cores-role-export = 导出

## Live render panel: the tiles instrument's captions
## The line under the picture's plane, then the tile size (left) and the tiles a frame (right).

# A walk's pass. $lo and $hi: the iteration band it rendered (numbers).
live-tiles-pin-band = 钉住遍 · 区段 { $lo }–{ $hi }
live-tiles-settle-band = 稳定遍 · 区段 { $lo }–{ $hi }
live-tiles-moving-whole = 移动帧 · 整幅画面
live-tiles-settled-whole = 已稳定帧 · 整幅画面
# $pass: this pass; $passes: the passes the refresh is split into.
live-tiles-split-pass = 拆分刷新 · 第 { $pass }/{ $passes } 遍
# $grid: the grid's columns × rows, e.g. 12×8.
live-tiles-grid-done = 稳定网格 { $grid } · 完成
# $grid: the grid's columns × rows; $done and $total: tiles done, and in all.
live-tiles-grid-progress = 稳定网格 { $grid } · 已完成 { $done }/{ $total }
# The same, while a tile too costly for one frame goes out in strips.
live-tiles-grid-strips = 稳定网格 { $grid } · 已完成 { $done }/{ $total } · 分条
# $samples: supersampling samples folded so far; $lo and $hi: the iteration band the next one is walking.
live-tiles-ssaa-band = SSAA · 已完成 { $samples } · 区段 { $lo }–{ $hi }
# $sample: the supersampling sample.
live-tiles-ssaa-whole = SSAA 样本 { $sample } · 整帧
# No GPU pass this frame, and the panel has not seen a frame yet.
live-tiles-idle-starting = 启动中
# No GPU pass this frame: the last complete picture is held on screen.
live-tiles-idle-held = 保持
# No GPU pass this frame: the last picture is stretched to fit the view.
live-tiles-idle-reprojected = 已重投影
# No GPU pass this frame: the picture on screen is current.
live-tiles-idle-live = 实时
# No GPU pass this frame, and nothing to show.
live-tiles-idle-none = 空闲
# The tile size: $res, a whole picture's resolution (2400×1300).
live-tiles-tile-res = 图块 { $res }
# The tile size: $px, a tile's side in pixels.
live-tiles-tile-px = 图块 { $px } px
live-tiles-no-pass = 无渲染遍
# Tiles a frame. $count: how many (0 when none).
live-tiles-rate-count = { $count }/帧
live-tiles-rate-one = 1 图块/帧
# $share: the part of one tile sent each frame, e.g. 0.33.
live-tiles-rate-part = { $share } 图块/帧
live-tiles-rate-sample = 1 图块/样本
# In the plane's corner during a walk: the share of pixels still iterating. $percent: a percentage, e.g. 54 or 1.5.
live-tiles-running = 运行中 { $percent }%

## Live render panel: help cards
## Shown when the pointer rests on an instrument or a lamp. Words in capitals (SETTLE, SSAA, ABANDON,
## HELD, REPROJ, ADOPT, REF AHEAD, ON SCREEN) are the panel's own legends: use the same words as
## their messages above.

# The heading of a card's list of what healthy looks like.
live-card-healthy = 正常
# The heading of a card's list of what signals trouble.
live-card-trouble = 异常

live-card-walk-what = 对于单帧开销过大的深度刷新，会将其钉住并逐遍步进：一遍接一遍地渲染，每一遍让像素再多迭代一个倍频程（256、512、1,024 … 直到所要求的次数）。外圈是这个次数的对数刻度，从顶部开始；扫描线是步进的进度；每段弧是一遍，按其 GPU 时间相对于 20 ms 目标的高低着色。白色光点在等待最后一遍的读数；闪光表示新画面上屏。
live-card-walk-note = 单遍开销过大的静止视图也以同样的方式步进，标记为“稳定”，而每个超采样样本会再次步进，标记为“SSAA”：它们的每一遍最多可用 400 ms，弧线也按这个标准着色。拆分刷新则改为每一遍画一个圆环：每一遍都包含全部区段，但只覆盖一部分图块。
live-card-walk-good-sweep = 扫描在几帧之内转完一圈，并以一次闪光结束。
live-card-walk-good-arcs = 弧线大多为绿色和琥珀色。
live-card-walk-good-still = 在深度静止视图上：每个样本的扫描较慢，弧线从绿色到琥珀色。
live-card-walk-bad-red = 红色弧线：超出预算的渲染遍，感觉为卡顿。
live-card-walk-bad-restart = 扫描中途停止并重新开始：步进被放弃了，屏幕上的画面仍然是旧的。
live-card-walk-bad-hollow = 空心弧线：计时结果始终没有返回的渲染遍，因此下一遍只能盲目确定大小。

live-card-age-what = 屏幕上的画面落后于视图多少，以缩放的倍频程计（一个倍频程为 2×）。刷新渲染期间，旧画面会被拉伸显示并逐渐变老；刷新完成上屏后，滞后会下降。下方的条带是最近几秒的记录。
live-card-age-good-sawtooth = 在约 0.3 到 0.8 之间呈锯齿状，指针围绕 0.5 刻度摆动。
live-card-age-good-still = 停下后接近 0。
live-card-age-bad-red = 指针一直停留在 1.0 以上的红区：画面已过时 2× 以上，缩放时看起来模糊或呈块状。
live-card-age-bad-ramp = 只升不降的斜坡：刷新没有完成（留意“放弃”）。

live-card-tiles-what = 本帧的 GPU 渲染遍如何覆盖画面。移动帧和步进遍覆盖整幅画面，说明文字会标出步进遍的迭代区段。步进的一遍只迭代尚未逃逸的像素：平面按这个比例填充（对数刻度，下限为 0.01%）并在角落显示读数，因此随着步进越来越深，它会逐渐排空。单帧开销稍大的刷新会被拆分为最多 8 遍，每一遍是一组交错的 16 px 图块。单遍开销过大的已稳定视图会变成一个图块网格，每帧一个图块，从中心向外，而开销仍然过大的图块会分条发送。下方读数：图块大小和每帧图块数。
live-card-tiles-good-whole = 缩放时为整遍；拆分刷新只有几遍。
live-card-tiles-good-grid = 停下后：几十个图块组成的网格在一两秒内填满，然后是 SSAA。
live-card-tiles-bad-small = 由数百个小图块组成的网格：每一遍的预算都很小，画面变清晰需要很长时间。
live-card-tiles-bad-strips = 许多图块以分条方式发送：图块的运行速度比网格规划时预计的要慢。
live-card-tiles-bad-splits = 拆分停留在 8 遍：刷新勉强能放下，再深一点就会退回到更慢的步进。
live-card-tiles-bad-dark = 移动时长时间暗着：根本没有渲染遍。

live-card-scope-what = 每帧一列，最新的在右侧：该帧运行的 GPU 工作，按类型堆叠（移动帧为绿色，钉住遍按开销从绿到红，稳定图块或静止视图的步进遍为米色，超采样样本为淡紫色，辅助 GPU 的帧和样本为蓝色），单位为毫秒。虚线位于 16.7 ms（60 fps）、33 和 50 处。空心标记表示计时结果始终没有返回的渲染遍。
live-card-scope-good-under = 移动时各列都低于 16.7 线。
live-card-scope-good-pin = 钉住遍为绿色到琥珀色。
live-card-scope-good-samples = 静止视图的超采样样本可能运行得更久：此时没有任何东西在移动。
live-card-scope-bad-over = 列超过 33 或 50 ms：可见的卡顿。
live-card-scope-bad-seconds = 耗时数秒的渲染遍：GPU 的看门狗可能会重置设备。
live-card-scope-bad-hollow = 许多空心标记：工作量是在没有测量数据的情况下确定的。

live-card-timers-what = GPU 的三个时间戳槽位。一个渲染遍被计时时，槽位亮起；两三帧之后读数返回时，槽位变暗。T0 为每帧的渲染遍计时；T1 和 T2 为额外的钉住遍计时。
live-card-timers-good = 有工作时 T0 就会闪烁；步进期间 T1 和 T2 闪烁。
live-card-timers-bad-stuck = 某个灯一直亮着：它的读数没有返回，开销估价会变得过时。
live-card-timers-bad-dark = GPU 忙碌时全部暗着：什么都没有被测量。

live-card-dial-what = 红色指针是视图的深度：以 2 的幂表示的放大倍数。各区段是 GPU 在该深度使用的算术：直接、df32（每个数用两个浮点数），以及 2^93 之后的 floatexp（带宽指数的浮点数）。圆点是正在构建的参考轨道：绿色表示当前视图，琥珀色表示在视图前方、缩放正前往的位置，蓝色表示漫游的下一个停留点。圆环表示一次安装；红叉表示因视图越过它而被丢弃的构建。
live-card-dial-good-ahead = 琥珀色圆点落在指针前方，并在指针到达之前变成圆环。
live-card-dial-good-crosses = 红叉很少。
live-card-dial-bad-passing = 指针越过了最后一个已安装的参考轨道：帧要等待参考轨道，感觉像卡住。
live-card-dial-bad-crosses = 反复出现红叉：缩放跑得比构建器还快，CPU 被浪费了。
live-card-dial-bad-crossing = 跨越 2^93 是已知的慢点：最初的几帧 floatexp 开销更大。

live-card-bank-what = 每个灯在其事件发生时亮起，并在约 150 ms 内渐暗，因此忙碌的渲染器会闪烁，空闲的则会变暗。从左到右依次是：帧、刷新、参考轨道，然后是辅助 GPU、稳定、调色板和算术。
live-card-bank-note = 将鼠标悬停在灯上，可了解其含义。

live-card-depth-what = 以 2 的幂表示的放大倍数：2^100 约为 1.3 × 10^30。
live-card-depth-note = 超过 2^93 后，每一步的开销都更大（floatexp），参考轨道的构建时间也更长。

live-card-iter-what = 迭代要求：每个像素在被视为位于集合内部之前最多可以迭代多少次。它随深度攀升。
live-card-iter-good = 随着深入稳步攀升。
live-card-iter-bad-low = 在深处仍停留在低值：螺旋和迷你 Mandelbrot 会渲染成平坦的内部。
live-card-iter-bad-high = 非常高：每次刷新的开销都更大，步进也更久。

live-card-ms-what = 显示的帧之间的时间。
live-card-ms-good = 等于你的显示器的刷新间隔：60 Hz 显示器上为 16.7。
live-card-ms-bad-stutter = 高于 33：可见的卡顿。
live-card-ms-bad-spikes = 高于 50 的尖峰：顿挫，通常是某一遍开销过大（见“GPU 渲染遍”）。

live-card-clock-what = 本地时间的墙钟，精确到十分之一秒。
live-card-clock-note = 它为面板的任何屏幕截图标上拍摄时间，这样一张出问题的截图就能与程序的日志对应起来。

live-card-tach-what = 每秒的迭代步数。白色：要求的步数（像素 × 迭代次数）。琥珀色：GPU 实际执行的步数。两者之差是 BLA（双线性近似）一次跳过许多迭代的结果；×N 就是这个倍数。里程表累计整个会话的总数。
live-card-tach-good = 在深处差距很大：BLA 正在发挥作用，×10 到 ×1,000。
live-card-tach-bad-together = 在深处两根指针重合：BLA 在这里无法跳过（常见于迷你 Mandelbrot 内部附近），因此刷新很慢。
live-card-tach-bad-red = 白色指针进入红区：要求的超过了 GPU 在一帧内能完成的量。

live-card-engines-what = 每个 GPU 的繁忙程度，圆点将其帧送到屏幕上：它的渲染遍在过去一秒内的 GPU 时间，包括辅助 GPU 的运动帧和超采样样本。“显示占比”是每个 GPU 所生成的帧在显示的帧中所占的比例。只有一个 GPU 时，辅助一侧保持空闲。
live-card-engines-good = 深入期间两者都很忙，辅助 GPU 提供了一部分帧。
live-card-engines-bad-pinned = GPU 0（窗口所用的 GPU）一直接近 100%：它就是瓶颈，帧会卡顿。
live-card-engines-bad-idle = 辅助 GPU 已启用却空闲：它没有交付帧。

live-card-memory-what = RAM：程序的工作集，红线为其峰值。VRAM：程序在 GPU 上占用的内存，根据它分配的大小按类型计算（G 缓冲、保持和 SSAA 纹理、轨道和 BLA 缓冲区、辅助 GPU 的参考轨道副本），并与显卡容量对比。
live-card-memory-good = 缩放时保持平稳；随着参考轨道的创建和释放而上下变动。
live-card-memory-bad-leak = RAM 不断攀升、从不回落：内存泄漏。
live-card-memory-bad-vram = VRAM 接近显卡容量：分配可能失败，或者设备可能丢失。

live-card-cores-what = 每个核心的 CPU 使用率，从绿到琥珀再到红，以及一个示波器，每个忙碌的线程在其中围绕自己的角色运转：参考轨道构建器、评分池、辅助 GPU、旁路渲染。
live-card-cores-good = 缩放时一两个核心忙碌；选取参考点时所有核心短暂地一起忙碌。
live-card-cores-bad-every = 所有核心持续忙碌数秒：在极深处构建参考轨道。
live-card-cores-bad-own = 程序自己的核心一直处于红区：窗口本身会卡顿。

live-card-pixels-what = 每秒计算的像素数，单位为 Mpx/s。暗：仍在步进中迭代。亮：已完成。
live-card-pixels-good = 移动时为几十 Mpx/s。
live-card-pixels-bad = 画面过时却接近零：GPU 在等待某样东西——参考轨道或读数。

## Live render panel: the lamps' help cards

live-card-lamp-plan-what = 规划了一帧：选定了分辨率、迭代要求和预算，以及是否刷新。
live-card-lamp-plan-good = 视图或设置发生变化时就会闪烁。
live-card-lamp-plan-bad = 缩放时暗着：界面线程卡住了。

live-card-lamp-dispatch-what = 本帧有工作发往 GPU。
live-card-lamp-dispatch-good = 有任何变化时亮起；空闲时暗着。
live-card-lamp-dispatch-bad = 没有任何变化却很忙：程序没有稳定下来。

live-card-lamp-live-what = 屏幕上的画面是本帧自己的渲染结果。
live-card-lamp-live-note = 深入期间很少亮起，这是正常的：见“保持”和“重投影”。
live-card-lamp-live-good = 在浅层亮起，那里每帧的开销很小。

# "Prefer detail" is a setting's name.
live-card-lamp-held-what = 上一幅完整的画面显示在屏幕上，下一幅在它下面合成：深度缩放的刷新，或者在开启“缩放时优先细节”时，静止视图的下一个图块或超采样样本。
live-card-lamp-held-good-dive = 深度缩放期间一阵一阵地亮起，被“采用”打断。
live-card-lamp-held-good-still = 静止视图逐渐清晰时持续亮着，同时“SSAA”闪烁。
live-card-lamp-held-bad = 缩放时亮了好几秒却没有“采用”：刷新没有完成。

live-card-lamp-reproj-what = 上一幅画面，被拉伸以适应移动中的视图，直到新画面准备好。
live-card-lamp-reproj-good = 移动时短暂地一阵阵亮起。
live-card-lamp-reproj-bad = 长时间亮着：画面拉伸得越远就越模糊。

live-card-lamp-reading-what = 从 GPU 返回了一个读数：某一遍的计时、逃逸范围或内容。
live-card-lamp-reading-good = 渲染时稳定地闪烁。
live-card-lamp-reading-bad = 渲染时长时间暗着：计时没有返回，预算只能靠猜。

live-card-lamp-pin-pass-what = 钉住的步进中的一遍：画面又多渲染了一个迭代区段。
live-card-lamp-pin-pass-good = 一串渲染遍以“采用”结束。
live-card-lamp-pin-pass-bad = 一串渲染遍以“放弃”结束。

live-card-lamp-split-what = 拆分刷新中的一遍：以完整迭代次数渲染的一部分图块。
live-card-lamp-split-good = 以“采用”结束的短串。
live-card-lamp-split-bad = 一串 8 遍，即允许的最大值：刷新勉强能放下。

live-card-lamp-adopt-what = 一次完成的刷新替换了屏幕上的画面。
live-card-lamp-adopt-good = 缩放时每秒数次。
# "Picture age" names the instrument (its title, PICTURE AGE).
live-card-lamp-adopt-bad = 缩放时很少出现：画面在变老（见“画面滞后”）。

live-card-lamp-abandon-what = 一次正在进行的刷新被丢弃：视图移动得太远、它变得太旧，或者被更新的刷新取代。被视图越过的参考轨道构建会让它微微亮起。
live-card-lamp-abandon-good = 很少出现，或者在一次跳转之后立即出现一次。
live-card-lamp-abandon-bad = 频繁出现：工作被浪费，画面一直是旧的。

live-card-lamp-ref-live-what = 正在为当前视图构建参考轨道：它还没有参考轨道、已离开原有的参考轨道，或者需要更长的参考轨道。
live-card-lamp-ref-live-good = 短暂闪烁。
live-card-lamp-ref-live-bad-long = 长时间亮着：深度视图在等待它。
live-card-lamp-ref-live-bad-lookahead = 深入期间经常亮起，而“超前参考”却暗着：预取跟不上。

live-card-lamp-ref-ahead-what = 正在比视图更深处、缩放正前往的位置构建一个预取参考轨道。
live-card-lamp-ref-ahead-good = 深入期间有规律地闪烁。
live-card-lamp-ref-ahead-bad = 深度深入期间暗着：视图会跑到参考轨道前面去。

live-card-lamp-ref-hold-what = 漫游的停留预取：在滑向下一个停留关键帧的途中为它构建参考轨道，使播放永远不必等待。
live-card-lamp-ref-hold-note = 漫游播放之外保持暗着。
live-card-lamp-ref-hold-good = 漫游中每次停留之前闪烁一次。
live-card-lamp-ref-hold-bad = 在整个停留期间都亮着：预取晚了，漫游在等它。

live-card-lamp-series-what = 为新参考轨道计算的级数近似，让每个像素可以跳过最初的迭代。
live-card-lamp-series-good = 短暂。
live-card-lamp-series-bad = 在深处持续很久：它会推迟参考轨道的安装。

live-card-lamp-worker-what = 第二个 GPU 交付了一帧。
live-card-lamp-worker-good = 启用辅助 GPU 时稳定闪烁。
live-card-lamp-worker-bad = 已启用辅助 GPU 却暗着：它没有交付帧。

live-card-lamp-tile-what = 一个稳定图块（或其中的一条）发往了 GPU。
live-card-lamp-tile-good = 停下后一阵闪烁，然后安静下来。
live-card-lamp-tile-bad = 一直不停：网格不断重新开始，因为有东西在不断改变视图或设置。

live-card-lamp-ssaa-what = 一个超采样样本被加到已稳定的画面上。
live-card-lamp-ssaa-good = 稳定后一串闪烁，然后暗下来。
live-card-lamp-ssaa-bad = 一次又一次重新开始：稳定过程不断被打扰。

live-card-lamp-palette-what = 颜色归一化采用了新的逃逸范围，因此颜色会跟随缩放变化。
live-card-lamp-palette-good = 缩放时闪烁。
live-card-lamp-palette-bad = 缩放时暗着：调色板范围被冻结，颜色变得平淡、对比度低。

live-card-lamp-mode-what = 算术发生了变化：直接、df32 或 floatexp。
live-card-lamp-mode-good = 在 2^93 这样的边界处闪烁一次。
live-card-lamp-mode-bad = 来回闪烁：视图在边界两侧来回摇摆。

## The Render client window (File ▸ Render client…): lending this machine's GPU to a render farm

farm-client-title = 渲染客户端
farm-client-intro = 为渲染农场渲染帧。控制端——另一台计算机上的“工具 ▸ 在渲染农场上渲染…”——会显示它的地址和农场密钥。本机不打开任何端口：由本机连接到控制端。
# A field label, and the label of the status row that names the controller.
farm-client-controller = 控制端
farm-client-find = 查找
farm-client-find-hover = 在此网络上查找控制端（控制端在其农场处于监听状态时会应答）
farm-client-key = 农场密钥
# A checkbox beside the masked farm key: show it in clear.
farm-client-show-key = 显示
# A field label (the name this machine goes by), and the label of the status row about this machine.
farm-client-machine = 本机
farm-client-machine-hover = 控制端为本机显示的名称。
farm-client-share = 共享驱动器
# Greyed hint inside the empty field; the path is an example.
farm-client-share-hint = 可选——例如 \\fileserver\share
farm-client-share-hover = 本机访问控制端所用共享驱动器的路径（共享模式）：帧会写到那里，而不是通过连接发送。留空则发送每一帧。
farm-client-gpu = 显卡
# The label of the row with the idle option.
farm-client-when = 何时
# A checkbox followed by a number of minutes ("Only when idle for [5] min").
farm-client-when-idle = 仅在空闲达到
farm-client-when-idle-hover = 只有在无人触碰本机的键盘或鼠标达到这么长时间后，或在屏幕锁定期间，才接受工作。有人开始使用本机时，正在进行的帧会立即停止并交还给农场。
# A collapsible section: the largest job this machine accepts.
farm-client-limits = 限制
farm-client-limits-note = 要求超出这些限制的任务会被拒绝，而绝不会被削减。
farm-client-limit-size = 最大帧尺寸
farm-client-limit-ss = 最大超采样
farm-client-limit-iter = 最大迭代次数

## Render client: the graphics-card choice

farm-client-gpu-system = 由系统选择
farm-client-gpu-every = 所有显卡
# $count: this machine's graphics cards.
farm-client-gpu-every-n = 所有显卡（{ $count }）
# $count: this machine's graphics cards.
farm-client-gpu-every-each = 所有显卡（{ $count }）——每块一个会话
# A card the window cannot name. $number: its number (or numbers) as --list-adapters gives it.
farm-client-gpu-adapter = 适配器 { $number }
farm-client-gpu-finding = 正在查找本机的显卡…
# "NAME · GPU 1" is how such a machine is named in the controller's list: keep it as it is.
farm-client-gpu-hover = 由哪块显卡渲染农场的帧。选择所有显卡时，每块显卡都会作为一台独立的机器加入农场——“NAME · GPU 1”、“NAME · GPU 2”——而“暂停”和“断开连接”会作用于所有这些显卡。

## Render client: looking for controllers on the network (the Find button)

farm-client-finding = 正在此网络上查找控制端…
# $port: the UDP port number.
farm-client-found-none = 没有控制端应答。控制端在其农场处于监听状态时才会应答；它的防火墙必须允许 UDP { $port } 入站。
farm-client-found-pick = 在此网络上——单击其中一个即可使用：
# $identity: the controller's identity fingerprint.
farm-client-found-hover = 身份 { $identity }——使用此控制端（仍需要它的农场密钥）
# After a warning sign. $version and $git: this app's version and build.
farm-client-found-other-build = 与本程序不同的构建（{ $version } { $git }）：它会拒绝本机。

## Render client: what is wrong with what was typed, or with starting the client

farm-client-bad-address = 请按控制端窗口中显示的格式，以 HOST:PORT 输入控制端的地址（例如 192.168.1.20:46733）。
# $why: what is wrong with the key (in English).
farm-client-bad-key = 农场密钥：{ $why }
# $why: what is wrong with the name (in English).
farm-client-bad-name = 名称：{ $why }
farm-client-no-config = 没有可用于保存农场密钥的配置文件夹。
# $why: the system's reason.
farm-client-start-failed = 无法启动渲染客户端：{ $why }

## Render client: the state, in the status heading

farm-client-phase-connecting = 正在连接…
farm-client-phase-checking = 正在运行自检…
farm-client-phase-idle = 已连接——等待工作
farm-client-phase-rendering = 正在渲染
farm-client-phase-in-use = 等待中——有人正在使用本机
farm-client-phase-paused = 已暂停
farm-client-phase-retrying = 无法访问控制端——正在重试
# $seconds: the wait before the next try.
farm-client-phase-retrying-in = 无法访问控制端——{ $seconds } s 后重试
farm-client-phase-disconnected = 已断开连接
farm-client-phase-ended = 已结束

## Render client: the line under the status heading

# $address: the controller's address (HOST:PORT).
farm-client-detail-connecting = 正在连接到 { $address }…
# $controller: the controller's name.
farm-client-detail-checking = 已连接到“{ $controller }”——正在运行自检
# $controller: the controller's name.
farm-client-detail-approval = 已连接到“{ $controller }”——等待其用户批准本机
# $controller: the controller's name.
farm-client-detail-idle = 已连接到“{ $controller }”——空闲
# $why: the check that failed and why (in English).
farm-client-detail-check-failed = 自检失败——{ $why }
# $why: the controller's reason (in English).
farm-client-detail-refused = 被控制端拒绝：{ $why }
# $why: the controller's reason (in English).
farm-client-detail-closed = 控制端关闭了连接：{ $why }
# $why: the controller's reason (in English).
farm-client-detail-ended = 控制端结束了连接：{ $why }
# $job: the job's name.
farm-client-detail-bundle = 任务“{ $job }”——正在接收其数据包
# $job: the job's name.
farm-client-detail-job-waiting = 任务“{ $job }”——等待要渲染的帧
# $why: the limit the job is over (in English).
farm-client-detail-job-refused = 已拒绝此任务：{ $why }
farm-client-detail-job-over = 任务已结束
# $first and $last: frame numbers.
farm-client-detail-rendering = 正在渲染第 { $first }–{ $last } 帧
# $time: how long nobody must use the machine ("90 s", "5 min").
farm-client-detail-in-use = 等待中——有人正在使用本机；无人使用达到 { $time } 后恢复工作
farm-client-detail-paused = 已暂停——已通知控制端
farm-client-detail-pausing = 正在暂停——正在完成当前帧
farm-client-detail-cancelled = 已取消当前帧；已暂停
# A machine with several graphics cards. $rendering: the cards rendering now; $total: all its cards (two or more).
farm-client-rendering-on-cards = 正在 { $total } 块显卡中的 { $rendering } 块上渲染

## Render client: the status rows

# The value of the Controller row. $name: the controller's name; $address: HOST:PORT; $identity: its identity fingerprint.
farm-client-row-controller-value = “{ $name }” · { $address } · 身份 { $identity }
# The value of the This machine row. $name: this machine's name; $identity: its identity fingerprint.
farm-client-row-machine-value = “{ $name }” · 身份 { $identity }
farm-client-row-gpu = GPU
# One graphics card's row. $slot: the card's number.
farm-client-row-gpu-n = GPU { $slot }
farm-client-row-job = 任务
# The frames handed to this machine.
farm-client-row-run = 批次
farm-client-row-sent = 已发送
# The Run row, in parts joined with " · ": the run, the frame in progress, its time. $first and $last: frame numbers.
farm-client-run-frames = 第 { $first }–{ $last } 帧
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-client-run-frame = 第 { $frame } 帧（{ $index }/{ $count }）
# $time: the time spent on it so far ("2.1 s").
farm-client-run-frame-time = 此帧已用 { $time }
# One graphics card's run. $frame: the frame in progress; $first and $last: the run's frames.
farm-client-session-frame = 第 { $frame } 帧，范围 { $first }–{ $last }
farm-client-session-no-card = 尚不知道是哪块显卡
# The Sent row. $count: frames sent; $time: the mean time per frame ("2.1 s").
farm-client-sent = { $count } 帧 · 每帧 { $time }
# The Sent row of a machine with several cards. $count: frames sent; $split: each card's share ("812 + 422"); $time: the mean time per frame.
farm-client-sent-split = { $count } 帧（{ $split }）· 每帧 { $time }
# After a warning sign. $note: how the GPUs differ, and both GPUs (in English).
farm-client-gpu-note = 本机的显卡情况：{ $note }。它渲染的帧会与控制端的略有不同，在停留镜头中可能看得出来。
# A collapsible section: the self-check's results.
farm-client-self-check = 自检
farm-client-thumb-hover = 本机发送的最后一帧
farm-client-resume = 继续
farm-client-resume-hover = 重新接受工作
farm-client-pause = 暂停
farm-client-pause-hover = 完成正在进行的帧，然后不再接受工作。控制端会把本机的帧交给其他机器。
farm-client-cancel-frame = 取消此帧
farm-client-cancel-frame-hover = 立即停止正在进行的帧并暂停；控制端会在其他机器上渲染它。
farm-client-live-paused = 本机渲染农场帧期间，实时视图已暂停。
farm-client-close = 关闭
farm-client-close-active-hover = 关闭窗口；本机仍留在农场中（可从“文件”菜单重新打开）
farm-client-close-hover = 关闭窗口
farm-client-disconnect = 断开连接
farm-client-disconnect-hover = 离开农场：正在进行的帧会交还
farm-client-connect = 连接
farm-client-connect-hover = 连接到控制端并为其渲染帧

## The Render on farm window (Tools ▸ Render on farm…): the controller of a render farm

farm-ctl-title = 在渲染农场上渲染
farm-ctl-out = 输出文件夹
farm-ctl-browse = 浏览…
farm-ctl-out-note = 帧，以及 farm/ 中的任务状态：再次运行同一任务即可继续。
farm-ctl-size = 尺寸
farm-ctl-size-custom = 自定义…
# The size list's choice when the size is typed in. $width and $height: pixels.
farm-ctl-size-custom-value = 自定义——{ $width }×{ $height }
farm-ctl-fps = 帧/秒
farm-ctl-ss = 超采样
farm-ctl-port = 监听端口
farm-ctl-port-hover = 客户端连接到此端口。请在防火墙中为本地网络放行该端口。
farm-ctl-share = 共享驱动器
# Greyed hint inside the empty field.
farm-ctl-share-hint = 可选——本机访问它的路径
farm-ctl-share-hover = 共享模式：拥有同一驱动器的客户端会把帧写到其中，而不是通过连接发送，本机会在那里逐一检查。没有该驱动器的客户端则发送它们的帧。
# A label followed by a number of clients and two checkboxes.
farm-ctl-start-with = 启动所需
# The unit after the number in the box ("Start with [3] clients"). $count: that number.
farm-ctl-start-clients = 台客户端
farm-ctl-start-clients-hover = 当这么多台机器已连接并通过自检后，任务开始；其他机器可以稍后加入。
farm-ctl-local = 也在本机上渲染
farm-ctl-local-hover = 本机算作一台客户端。
farm-ctl-sharing = 共享参考轨道
farm-ctl-sharing-hover = 一个深度帧的参考轨道可能需要几分钟到几小时才能构建完成。共享后，它只构建一次，然后传给其他机器。共享的参考轨道是另一个同样有效的参考轨道：在极深处，与自行构建参考轨道的帧相比，可能有少数像素不同。
farm-ctl-key = 农场密钥
# A checkbox beside the shortened farm key: show all of it.
farm-ctl-show-key = 显示
farm-ctl-copy-key = 复制密钥
farm-ctl-copy-key-hover = 将它粘贴到每台客户端的“渲染客户端”窗口中
# In place of the farm key, when there is none.
farm-ctl-no-key = 尚无
farm-ctl-make-key = 生成密钥
farm-ctl-make-key-hover = 客户端需要它才能加入；按“渲染”时也会生成
# What to do on the other machines. $address: this controller's HOST:PORT.
farm-ctl-steps = 在每台客户端上：“文件 ▸ 渲染客户端…”，控制端 { $address }，以及农场密钥。
# The same when this machine's address is not known. $port: the port number.
farm-ctl-steps-port = 在每台客户端上：“文件 ▸ 渲染客户端…”，本机的地址和端口 { $port }，以及农场密钥。
# The size of the job. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
farm-ctl-summary = { $count } 帧 · 漫游时长 { $seconds }s · { $width }×{ $height } ss{ $ss }
farm-ctl-strip-hover = 任务的每一帧，从左到右：已完成、正在渲染、尚待渲染、已放弃。
farm-ctl-no-tour = 未加载漫游。
# $folder: the folder; $why: the system's reason.
farm-ctl-out-error = 输出文件夹 { $folder }：{ $why }
# $why: the system's reason.
farm-ctl-start-failed = 无法启动农场：{ $why }
# $code: the controller process's exit code, as the code gives it.
farm-ctl-ended-exit = 控制端已结束（退出代码 { $code }）。

## Render on farm: the state, in the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-phase-waiting = 正在等待渲染客户端（已连接 { $connected }/{ $needed }）
farm-ctl-phase-preparing = 正在准备任务
farm-ctl-phase-rendering = 正在渲染
farm-ctl-phase-paused = 已暂停
farm-ctl-phase-complete = 已完成
farm-ctl-phase-stopped = 已停止
farm-ctl-phase-ended = 已结束

## Render on farm: the words after the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-detail-waiting = 正在等待渲染客户端：已连接 { $connected }/{ $needed }
farm-ctl-detail-preparing = 正在本机上测量归一化锚点，为所有客户端只测一次
farm-ctl-detail-no-space = 正在等待输出文件夹腾出空间
farm-ctl-detail-paused = 已暂停：不再分发新的工作
farm-ctl-detail-not-rendered = 已结束，但有未渲染的帧——在同一文件夹上再次运行即可继续
farm-ctl-detail-stopped = 已停止——在同一文件夹上再次运行即可继续
# $code: the controller process's exit code.
farm-ctl-detail-ended = 已结束（退出代码 { $code }）
farm-ctl-detail-finishing = 正在收尾

## Render on farm: progress

# On the progress bar. $done: frames done; $total: frames in the job.
farm-ctl-progress = { $done } / { $total } 帧
# Added to the progress bar. $rate: frames per second, with decimals.
farm-ctl-progress-rate = { $rate } 帧/秒
# Added to the progress bar. $time: the time still to go ("38m 10s").
farm-ctl-progress-eta = 剩余约 { $time }
# Facts under the progress bar, joined with " · ". $received and $sent: kilobytes per second.
farm-ctl-io = 接收 { $received } KB/s · 发送 { $sent } KB/s
# $space: free space on the output drive ("412 GB").
farm-ctl-free = 可用 { $space }
# $time: the time since the job started ("38m 10s").
farm-ctl-elapsed = 已用 { $time }
# $count: frames given up on.
farm-ctl-failed = { $count } 帧未渲染
# Reference orbits. $built: built for this job; $cached: found in the cache; $reused: reused between frames; $shared: passed between machines.
farm-ctl-refs = 参考轨道：构建 { $built }，来自缓存 { $cached }，复用 { $reused } · 共享 { $shared }
# After a warning sign.
farm-ctl-storage-low = 输出文件夹快满了：在腾出空间之前不会分发新的工作。
# After a warning sign.
farm-ctl-gpus-differ = 此农场中的 GPU 各不相同。它们渲染的帧会略有不同，在停留镜头中可能看得出来。
# After a warning sign. $count: the GPU classes in the farm.
farm-ctl-gpus-differ-classes = 此农场中的 GPU 各不相同（{ $count } 类 GPU：不同的 GPU、驱动程序或探测渲染结果）。它们渲染的帧会略有不同，在停留镜头中可能看得出来。
# $gpu: this machine's GPU, API and driver.
farm-ctl-this-gpu = 本机：{ $gpu }
# A machine whose GPU differs. $name: the machine; $note: how its GPU differs (in English); $adapter: its GPU.
farm-ctl-gpu-row = { $name }：{ $note }——{ $adapter }
# The same with its driver. $driver: the driver's name and version.
farm-ctl-gpu-row-driver = { $name }：{ $note }——{ $adapter }，驱动程序 { $driver }

## Render on farm: the table of machines

# Column headers (short: the table is narrow).
farm-ctl-col-machine = 机器
farm-ctl-col-state = 状态
farm-ctl-col-gpu = GPU
farm-ctl-col-frames = 帧
farm-ctl-col-per-frame = 每帧
# How the machine delivers its frames (a speed, or "shared drive").
farm-ctl-col-link = 链路
# When the controller last heard from the machine.
farm-ctl-col-heard = 上次响应
# In the machine's tooltip. $count: its frames that failed verification.
farm-ctl-strikes = { $count } 帧未通过验证
# In the Link column: it delivers its frames through the shared drive.
farm-ctl-link-share = 共享驱动器
# In the Heard column. $seconds: how long ago.
farm-ctl-heard-ago = { $seconds } s 前
farm-ctl-readmit = 重新接纳
farm-ctl-readmit-hover = 允许这台机器在重新连接时再次加入
farm-ctl-remove = 移除
farm-ctl-remove-hover = 将这台机器移出任务；它的帧会交给其他机器
# The GPU column's tooltip.
farm-ctl-probe-same = 它的探测渲染结果与本机完全相同。
# $count: the pixels that differ.
farm-ctl-probe-differs = 它的探测渲染结果与本机有 { $count } 个像素不同：它渲染的帧会与本机的略有不同。
farm-ctl-probe-unknown = 尚未比较它的探测渲染结果。
# $driver: the driver's name and version.
farm-ctl-gpu-driver = 驱动程序 { $driver }
# After a warning sign. $note: how its GPU differs (in English: "a different GPU").
farm-ctl-gpu-note = 与本机相比：{ $note }

## Render on farm: a machine's state, in the State column

# $count: its frames that failed verification.
farm-ctl-state-removed-bad = 已移除：{ $count } 个坏帧
farm-ctl-state-removed = 已被你移除
farm-ctl-state-removing = 正在移除——正在收集其诊断信息
farm-ctl-state-gone = 已离开
farm-ctl-state-leaving = 正在离开
farm-ctl-state-checking = 检查中（自检）
farm-ctl-state-paused = 已被其用户暂停
farm-ctl-state-parked = 已搁置：崩溃了两次——重新连接时会重新加入
farm-ctl-state-refused = 拒绝了此任务（其自身的限制）
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-ctl-state-rendering = 正在渲染 { $frame }（{ $index }/{ $count }）
farm-ctl-state-starting = 正在开始一个批次
farm-ctl-state-idle = 空闲
farm-ctl-state-admitted = 已接纳——等待任务

## Render on farm: the buttons

# A collapsible section: the controller's messages.
farm-ctl-events = 事件
farm-ctl-resume = 继续
farm-ctl-pause = 暂停
farm-ctl-pause-hover = 不再分发新的工作；正在进行的帧会完成
farm-ctl-live-paused = 本机为农场渲染期间，实时视图已暂停。
farm-ctl-close = 关闭
farm-ctl-close-active-hover = 关闭窗口；任务会继续运行（可从“工具”菜单重新打开）
farm-ctl-close-hover = 关闭窗口
farm-ctl-copy-command = 复制命令
farm-ctl-copy-command-hover = 复制等效的命令行
farm-ctl-stop = 停止
# The Stop button once a stop has been asked for and the job has not ended after 10 seconds.
farm-ctl-stop-now = 立即停止
farm-ctl-stop-hover = 以已有的帧结束任务；再次运行同一任务即可继续
farm-ctl-render = 渲染
farm-ctl-render-hover = 开始监听客户端，并在它们上面渲染漫游

## The Render tour window (Tools ▸ Render tour…)

tour-render-title = 渲染漫游
tour-render-out = 输出文件夹
tour-render-browse = 浏览…
tour-render-out-hover = 帧写入的完整路径。
tour-render-prefix = 帧前缀
# <prefix> stands for the prefix typed in the field.
tour-render-prefix-hover = 帧会写为 <前缀>_00000.png
tour-render-size = 尺寸
tour-render-size-custom = 自定义…
# The size list's choice when the size is typed in. $width and $height: pixels.
tour-render-size-custom-value = 自定义——{ $width }×{ $height }
tour-render-fps = 帧/秒
tour-render-fps-hover = 低于 1 fps 也是合理的：以 0.25 fps 对一段很长的深入进行采样，就是在不渲染数千帧的情况下检查深层回归停留点的方法。
tour-render-ss = 超采样
tour-render-chapter = 章节
tour-render-whole = 整个漫游
tour-render-chapter-hover = 只渲染一个章节时会保持“全局”帧编号，因此它的帧可以直接放回完整的序列中。
tour-render-mp4 = 合成 mp4
tour-render-mp4-hover = 之后运行 ffmpeg（必须在 PATH 中）。PNG 帧会保留。
tour-render-overwrite = 覆盖现有的帧
tour-render-overwrite-hover = 关闭时：渲染会停下来询问，而在这里它无法询问。
tour-render-resume = 继续中断的渲染
tour-render-resume-hover = 保留输出文件夹中已有的帧，只渲染缺少的帧。会先检查最新的一帧，如果它不完整就丢弃（渲染中断时所在的帧就是它正在写入的帧），然后逐帧回退，直到找到一个完好的帧；如果文件夹中的帧尺寸不同，会拒绝渲染，而不是把它们混入此序列。
# $count: this machine's graphics cards.
tour-render-all-gpus = 使用所有显卡（{ $count }）
tour-render-all-gpus-hover = 通过在这台计算机上运行的渲染农场（--gpus all），同时在本机的每一块显卡上渲染。帧、帧的文件名、“继续中断的渲染”和 mp4 都与单卡时相同；会渲染整个漫游，按农场自己的顺序。不同型号的显卡会把少数像素画得略有不同：停留镜头和溶解过渡会保持在同一块显卡上。
tour-render-order = 渲染顺序
tour-render-sequential = 顺序
tour-render-sequential-hover = 按第 0、1、2 … 帧的顺序（默认）。
tour-render-progressive = 渐进
tour-render-progressive-hover = 先预览整个漫游，再细化到完整细节：先渲染关键帧，然后不断二分最大的时间间隔，直到每一帧都存在。无论哪种方式，帧都会落在正确的序号上，因此“继续中断的渲染”和 mp4 都照常工作——你可以提前停止，粗略看看这本粗糙的翻页书，然后用“继续中断的渲染”以任意一种顺序重新开始。
# The cost of the render. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
tour-render-summary = { $count } 帧 · 漫游时长 { $seconds }s · { $width }×{ $height } ss{ $ss }
# The disk space the frames will take (an estimate). $size: a size ("5.3 GB").
tour-render-disk = 帧文件 ≈{ $size }
# $size: the frames' estimated size; $free: the free space on that drive ("6.0 GB").
tour-render-disk-free = 帧文件 ≈{ $size } · 可用 { $free }
# After a warning sign. $disk: the line above ("≈51.0 GB of frames · 6.0 GB free").
tour-render-disk-short = { $disk }——空间不足以容纳整个序列
tour-render-separate = 在单独的进程中渲染“磁盘上”的脚本，因此不包括未保存的编辑——而且那里丢失 GPU 设备也不会连累本会话。
# On the progress bar. $done: frames done; $total: frames to render.
tour-render-progress = { $done } / { $total } 帧
# Before the render's first progress line.
tour-render-starting = 正在启动…
tour-render-live-paused = 实时视图已暂停——渲染独占 GPU，因此能更快完成，并能准确测量自己的帧开销。
tour-render-cancel = 取消
tour-render-cancel-hover = 关闭，不渲染
tour-render-copy-command = 复制命令
tour-render-copy-command-hover = 将等效的命令行复制到剪贴板
tour-render-stop = 停止渲染
tour-render-render = 渲染
tour-render-render-hover = 开始渲染帧序列
# $folder: the output folder.
tour-render-finished = 渲染完成 → { $folder }
# $why: the render's own reason (in English).
tour-render-failed = 渲染失败：{ $why }
# $last: the render's last progress line (in English), or nothing.
tour-render-failed-or-stopped = 渲染失败或已被停止。{ $last }
# $why: the system's reason.
tour-render-no-exe = 找不到可执行文件：{ $why }
# $why: the system's reason.
tour-render-start-failed = 无法启动渲染：{ $why }

## The Custom formula dialog (Fractal ▸ Custom formula…)
## The formula language itself stays as it is in every language: function names (sin, cosh, conj…),
## operators, variables (z, c, p1), and the formulas. Keep them as written inside these messages.

# The window's title.
formula-dialog-title = 自定义公式
# A small line at the top of the window, above the formula.
formula-dialog-intro = 迭代步，以 Fractint 风格的表达式书写
# The two ways to show and edit the formula (top right of the window).
formula-dialog-textbook = 教科书
formula-dialog-textbook-hover = 以教科书排版的方式显示公式，并以这种形式编辑它
formula-dialog-text = 文本
formula-dialog-text-hover = 以文本形式编辑公式
# The tooltip of the "LaTeX" button (the button's own text stays LaTeX).
formula-dialog-latex-hover = 按此处的排版样子，将公式复制为 LaTeX
# The drop-down list of example formulas (the collection).
formula-dialog-examples = 示例
# Help under the formula in Textbook mode. The characters / ^ ( are what the user types: keep them.
formula-dialog-help-textbook = 按书面写法输入：/ 把它前面的项变成分数，^ 变成指数，( 输入括号。方向键在公式中移动；Tab 移到下一个空框。红色的行无法读取：单击它即可作为文本编辑。
# Help under the formula in Text mode. "; starts a comment": the ; is what the user types.
formula-dialog-help-text = 直接输入，或使用键盘面板——它包含公式语言认识的每一个名称。名称会在输入时自动补全（Tab）。语句之间以换行或逗号分隔；; 开始一条注释。
# The syntax check, when the formula reads.
formula-dialog-reads = 读取正确。
# The syntax check in Textbook mode, when the formula has an empty box to fill in.
formula-dialog-empty-box = 请填写空框。
# The syntax check in Textbook mode. $line: the line number; $message: what is wrong (from the formula parser, in English).
formula-dialog-error-line = 第 { $line } 行：{ $message }
# One function written with another notation's name, and the language's name for it: "ln to log".
# $from: the name as written (ln); $to: the formula language's name (log).
formula-respell-pair = { $from } 改为 { $to }
# A button that rewrites them. $changes: one or more formula-respell-pair, joined with ", ".
formula-respell = 将 { $changes }
formula-respell-hover = 按公式语言的命名方式书写这些函数
# Shown when Apply finds a parameter field that is not a number. $name: the parameter (p1 … p5).
formula-param-not-numbers = { $name } 不是一对数字
# The Apply button (also in the Formula library window).
formula-apply = 应用
formula-dialog-apply-hover = 编译公式并显示
# Shown beside Apply while the formula is being compiled.
formula-dialog-compiling = 正在为 GPU 编译公式…
# The greyed hint in an empty name box (also in the Formula library window).
formula-name-hint = 名称（可选）
# The save button: "Update" when the library already has a formula of that name.
formula-dialog-update-in-library = 在库中更新
formula-dialog-save-to-library = 保存到库
formula-dialog-save-hover = 将此公式及其参数保存在公式库中
formula-dialog-save-disabled-hover = 只有读取正确的公式才能保存
# Opens the Formula library window.
formula-dialog-library = 公式库…
formula-dialog-library-hover = 已保存的公式：应用、编辑、导入和导出

## The Custom formula dialog's keypad
## The keys' captions are formula tokens and stay as they are; these are their tooltips.

# The keypad's third tab (the other two are "123" and "f(z)"). A short word.
formula-keypad-tab-names = 名称
# The tabs' tooltips.
formula-keypad-tab-basic-hint = 数字、运算符和最常用的名称
formula-keypad-tab-functions-hint = 函数
formula-keypad-tab-names-hint = 所有变量、参数、常数以及语句语法
# The key z.
formula-key-z = 迭代值
# The key c.
formula-key-c = 像素（Julia 模式下为 Julia 常数）
# The key p1 on the first tab.
formula-key-p1-below = 参数 1——在下方的字段中设置
# The keys p1 … p5. $n: the parameter's number.
formula-key-param = 参数 { $n }
formula-key-multiply = 乘
formula-key-divide = 除
# The key that types ^2.
formula-key-square-after = 平方（放在值之后：z^2）
# The key that types ^.
formula-key-power = 幂：整数、实数或复数指数
formula-key-square-root = 平方根
# The key that types |…|: in this language it is the squared modulus, as in Fractint.
formula-key-squared-modulus = 模的“平方” |z|²——与 Fractint 中相同
formula-key-add = 加
formula-key-subtract = 减（或取负）
# The key that types =.
formula-key-assign = 赋值：z = …，或临时变量 t = …
# The key that starts a new line (first tab).
formula-key-next-statement-comma = 下一条语句（也可以用逗号）
# The key "del".
formula-key-backspace = 删除光标前的字符（或所选内容）
# The key e.
formula-key-euler = 欧拉数
# The key that types (0.5, 0.5).
formula-key-complex-constant = 复数常数：(实部, 虚部)
# The key that types a comma.
formula-key-comma = 分隔语句，或分隔复数常数的两部分
formula-key-left = 光标左移
formula-key-right = 光标右移
# The function keys (the f(z) tab), one per function.
formula-key-sin = 正弦
formula-key-cos = 余弦
formula-key-tan = 正切
formula-key-cotan = 余切
formula-key-exp = 指数函数 e^z
formula-key-sinh = 双曲正弦
formula-key-cosh = 双曲余弦
formula-key-tanh = 双曲正切
formula-key-cotanh = 双曲余切
formula-key-log = 自然对数 ln（主分支）
formula-key-sqr = 平方：sqr(z) = z²
formula-key-sqrt = 平方根（主值）
formula-key-recip = 倒数 1/z
formula-key-conj = 复共轭
formula-key-flip = 交换实部和虚部
formula-key-abs = 对“两部分”都取绝对值：|x| + i|y|
formula-key-cabs = 模 |z|（一个实数）
formula-key-real = 实部
formula-key-imag = 虚部，作为实数
# "fn" is Fractint's name for a function slot: keep it.
formula-key-ident = 恒等（Fractint 的 fn 占位符）
# The key "pixel".
formula-key-pixel = 与 c 相同（Fractint 中的名称）
# The key "z =".
formula-key-set-z = 开始一条设置 z 的语句
# The key "t =".
formula-key-temporary = 临时变量：任何新名称都可以（t、w、a2、…）
# The key "; …".
formula-key-comment = 注释，直到行尾
# The key that starts a new line (names tab).
formula-key-next-statement = 下一条语句

## The formula's completion list (names offered as you type)

# Beside a name the formula itself assigns.
formula-completion-variable = 此公式设置的变量
# Beside a function typed with another notation's name.
# $what: what the function is, in English (from the formula parser: "The natural logarithm");
# $other: the other notation's name (ln); $ours: the formula language's name (log).
formula-completion-other-spelling = { $what }——{ $other } 在这里写作 { $ours }
# Under a list cut short. $count: the names not shown.
formula-completion-more = … 还有 { $count } 个
# The last line of the list. Tab, Enter and Esc are key names.
formula-completion-keys = Tab 或 Enter 补全，Esc 关闭

## The Custom formula dialog's Textbook mode

# The tooltip of a line that does not read. $why: what is wrong (from the formula parser, in English, or formula-textbook-fractint-sections).
formula-textbook-unread-hover = 无法读取：{ $why }。单击即可作为文本编辑。
# Why a line with Fractint's sections is shown as text in Textbook mode.
formula-textbook-fractint-sections = Fractint 的分段（初始化段的“:”、比较、a = b = …）以文本形式编辑

## The Formula library window (Fractal ▸ Formula library…)

# The window's title.
formula-library-title = 公式库
# The window's two tabs. $count: the formulas in each.
formula-library-mine = 我的公式（{ $count }）
formula-library-mine-hover = 你保存或导入的公式
formula-library-collection = 合集（{ $count }）
formula-library-collection-hover = Fractadyne 附带的公式，每个都有一个起始视图
# A small marker before the name of the formula the view shows now. A short word.
formula-library-showing = 显示中
# The tooltip of Apply on a formula of the collection.
formula-library-apply-shipped-hover = 在其起始视图中显示它
# A row's Edit button (after a pencil icon).
formula-library-edit = 编辑
formula-library-edit-hover = 在“自定义公式”对话框中打开它，但不应用
# A collection row's button that copies the formula into "My formulas".
formula-library-copy-to-mine = 复制到我的公式
formula-library-copy-to-mine-hover = 将它添加到你的公式中，以便保留或修改
formula-library-close = 关闭
# Saves the formula the view shows, under the name typed beside it.
formula-library-add-current = 添加当前公式
formula-library-add-current-hover = 保存视图所显示的公式
# "Fractal > Custom formula…" is the menu path: write it as the menus are named.
formula-library-add-current-disabled-hover = 请先应用一个自定义公式（分形 > 自定义公式…）
formula-library-import = 导入…
formula-library-import-hover = 添加公式文件中的公式；这里的任何内容都不会被替换
formula-library-export-all = 全部导出…
formula-library-export-all-hover = 将这里的所有公式写入一个文件，以便分享或保存
# The filter box: its label, and the greyed hint inside it.
formula-library-filter = 筛选
formula-library-filter-hint = 按名称或文本筛选
# Shown while the user has no saved formulas.
formula-library-empty = 还没有已保存的公式。可从“自定义公式”对话框保存一个，添加视图所显示的公式，或导入一个文件（我们的格式，或 Fractint 的 .frm）。
# The tooltip of Apply on one of the user's formulas: with a view saved, and without one.
formula-library-apply-view-hover = 显示此公式，并转到与它一起保存的视图
formula-library-apply-hover = 显示此公式
# The tooltip of a row's save icon.
formula-library-export-one-hover = 将此公式导出到文件
# A row asks before deleting: the question, then its two buttons. "Delete" is also the delete icon's tooltip.
formula-library-delete-ask = 删除它？
formula-library-delete = 删除
formula-library-keep = 保留
# Under a list cut short. $count: the formulas not listed.
formula-library-more = …还有 { $count } 个：在筛选框中输入即可查找。
formula-library-no-match = 没有与筛选条件匹配的公式。
# Under a saved formula this version cannot read. $why: what is wrong (from the formula parser, in English).
formula-library-unread = 在此版本中无法读取：{ $why }

## Formula library notices

# $why: the system's reason, in English.
formula-library-save-failed = 无法保存公式库：{ $why }
formula-library-nothing-to-save = 没有可保存的内容：公式为空。
# $name: the formula's name in the library.
formula-library-updated = 已在公式库中更新“{ $name }”。
formula-library-saved = 已将“{ $name }”保存到公式库。
# $name: the formula's name; $why: what is wrong.
formula-library-apply-failed = 无法应用“{ $name }”：{ $why }
# Copying a formula of the collection into "My formulas". $name: its name; $as_named: the name it was given (the same, or "Name (2)" when the name was taken).
formula-library-copy-exists = “{ $name }”已在你的公式中。
formula-library-copied = 已将“{ $name }”复制到你的公式中，名称为“{ $as_named }”。
# $name: the formula's name.
formula-library-deleted = 已从公式库中删除“{ $name }”。
# Why the library could not be saved (follows formula-library-save-failed).
formula-library-no-config-dir = 没有配置目录
# At startup, when the library file cannot be read. $why: the reason, in English; $path: a file's full path.
formula-library-unreadable-kept = 无法读取公式库（{ $why }）。它已被保留为 { $path }，并已新建一个空的库。
formula-library-unreadable-stuck = 无法读取公式库（{ $why }），也无法将其移到一旁：请在保存公式之前将 { $path } 复制到安全的地方，因为保存会替换它。

## Formula library: Import and Export
## A formula file's contents stay in English; this is what the windows and notices around them say.

# File-type names in the open and save dialogs (the extensions are added by the system).
formula-filter-formula-files = 公式文件
formula-filter-fractadyne = Fractadyne 公式
formula-filter-fractint = Fractint 公式
formula-filter-all-files = 所有文件
# Why a file cannot be imported (they follow formula-import-failed).
formula-import-too-large = 文件太大，不可能是公式文件
formula-import-not-text = 不是文本文件
formula-import-no-formulas = 其中没有公式
# $why: the TOML reader's message, in English.
formula-file-not-formulas = 不是公式文件（{ $why }）
# $format: what the file says it is ("fractadyne-gradients").
formula-file-other-format = 一个“{ $format }”文件，不是公式文件
# "[[formula]]" is the file's own syntax: keep it.
formula-file-no-formulas = 其中没有公式（公式文件以 [[formula]] 表列出公式）
# $file: the file's name; $why: one of the reasons above, or the system's (in English).
formula-import-failed = 无法导入“{ $file }”：{ $why }。
# After an import that added nothing. $file: the file's name.
formula-import-nothing-new = “{ $file }”中没有新内容。
# After an import. $count: the formulas added; $file: the file's name.
formula-import-added = 已从“{ $file }”导入 { $count } 个公式。
# The same two, followed by what else happened. $notes: one or more of the formula-import-note-… messages below, joined with "; ".
formula-import-nothing-new-notes = “{ $file }”中没有新内容：{ $notes }。
formula-import-added-notes = 已从“{ $file }”导入 { $count } 个公式：{ $notes }。
# Notes (clauses, no final full stop). $count: the formulas already in the library, skipped.
formula-import-note-duplicates = { $count } 个公式已在库中
# One formula was renamed because its name was taken. $from: its name in the file; $to: the name it was given ("Name (2)").
formula-import-note-renamed-one = “{ $from }”已被占用，因此改名为“{ $to }”
# $count: the formulas renamed because their names were taken (2 or more).
formula-import-note-renamed = { $count } 个因名称已被占用而改名
# $count: the entries skipped as empty or too long.
formula-import-note-skipped = { $count } 个条目为空或过长，已跳过
# A second sentence after importing a Fractint .frm file. $count: the entries that do not read; $why: the commonest reason (from the formula parser, in English).
formula-import-frm-unread = { $count } 个条目在此版本中无法读取（最常见的原因：{ $why }）。
# After exporting one formula. $name: the formula's name; $file: the file's name.
formula-export-one = 已将“{ $name }”导出到“{ $file }”。
# After exporting several. $count: how many; $file: the file's name.
formula-export-many = 已将 { $count } 个公式导出到“{ $file }”。
# $file: the file's name; $why: the system's reason, in English.
formula-export-failed = 无法写入“{ $file }”：{ $why }

## The control panel (right-hand side): header and section headings

# The panel's title, beside the button that hides it.
panel-title = 控制面板
panel-hide-hover = 隐藏控制面板
# The button that brings the hidden panel back (top right of the view).
panel-show-hover = 显示控制面板
# Section headings.
panel-section-navigate = 导航
panel-section-coloring = 着色
panel-section-quality = 质量
panel-section-effects = 效果
panel-section-overlays = 叠加层
panel-section-advanced = 高级
# The last section: reference notes about the current fractal. $name: the fractal's name (Mandelbrot, Burning Ship…).
panel-section-about = 关于 { $name }
panel-section-performance = 性能

## The control panel: Navigate
## Labels before a control sit in a column about 104 px wide (roughly 15 characters): a longer label pushes its control to the right.

panel-zoom-speed = 缩放速度
panel-zoom-speed-hover = 按住 Space 连续缩放的速度（1× ≈ 每 1.5 秒 2×）。
panel-snap-zoom = 缩放吸附到特征
panel-snap-zoom-hover = 用鼠标滚轮或 Space 放大时，将缩放拉向光标附近的迷你 Mandelbrot（或实轴），使其滑到光标下方，缩放就会落在它上面。特征越近，拉力越强。仅限 Mandelbrot、单视图。
panel-click-zoom = 单击缩放
panel-click-zoom-hover = 开启后，在视图中左键单击会按下方的倍数深入（右键单击后退），并以单击的点为新的中心。拖动仍为平移；Shift+拖动 / 右键拖动仍为框选缩放。Backspace 可撤销一次单击。在双视图中，它在两个面板上都可用，而此功能开启时，钉住 Julia c 改为 Ctrl+单击。按住 Shift 可在光标下方显示一个放大镜，展示单击后将到达的视图，按住时单击即可选取该点；Shift+拖动仍为框选缩放。
# The click-to-zoom factor, followed by buttons 2×, 4×, 10×…
panel-click-zoom-factor = 倍数
panel-click-zoom-snap = 吸附到最近的中心
panel-click-zoom-snap-hover = 跳转后，求解最近的迷你 Mandelbrot 中心，并精确地落在它上面，使你落在特征上而不是它附近。缩放不会改变——那是单击决定的。仅限 Mandelbrot。求解在后台运行：视图会立即跳转，并在答案到达时自行修正，在极深处这可能需要几秒钟。

## The control panel: Coloring
## Labels before a control sit in a column about 104 px wide (roughly 15 characters).

panel-method = 方法
panel-method-hover = 逃逸数据如何映射为颜色。条纹 / 三角不等式 / 轨道陷阱 / 分解会显现轨道结构；距离按与边界的远近着色。
panel-stripe-density = 条纹密度
panel-stripe-tail = 仅尾部（深度缩放）
panel-stripe-tail-hover = 对每条轨道最后 N 次迭代（一个指数窗口）的条纹项取平均，而不是对整条轨道取平均。在极深处，每条轨道除最后几百次迭代外都紧跟着参考轨道，因此整条轨道的平均值会让整个视图只有一种颜色；尾部平均则能保留对比度。短于窗口的轨道不受影响。对于 --render，也可以使用 --stripe-tail N。
panel-tail-length = 尾部长度
# The unit after the tail length's number: short for "iterations" (the slider reads "64 it").
panel-tail-length-unit = 次
panel-tail-length-hover = 以迭代次数计的窗口长度。短窗口（8–32）在任何深度下都能产生经典的粗犷色带；更长的窗口会再次趋向于平均成一种纯色，因为长混沌轨道的条纹项会被平均掉。
panel-trap-shape = 陷阱形状
panel-palette = 调色板
# The two colours of the Binary palette, each beside its colour swatch.
panel-in-set = 集合内
panel-out-of-set = 集合外
# The two colours of the Duotone palette, each beside its colour swatch.
panel-shadow = 阴影
panel-highlight = 高光
panel-cycle = 循环
panel-offset = 偏移
panel-normalize-hover = 在极深处，逃逸次数跨越数十万，固定的“循环”会在相邻像素之间让调色板回绕数千次——正确的密集区域看起来就像噪点。这会把调色板重新映射到视图实测的逃逸范围（之后由“循环”设置跨越该范围的调色板循环次数），但“仅当”视图确实出现混叠时才这样做——平滑的视图保持不变，因为重新映射它只会白白改变画面。勾选下方的“始终适配范围”可无条件重新映射。仅适用于“平滑迭代”方法；与 --normalize 一致。
# A checkbox indented under "Normalize deep colors" (also in the Color menu).
panel-fit-range = 始终适配范围
panel-fit-range-hover =
    即使视图没有混叠，也将调色板重新映射到视图实测的逃逸范围——上面的复选框只在密集区域原本会显得像噪点时才重新映射。

    当你希望调色板覆盖屏幕上的内容、而不是为了消除噪点时，使用此选项。它仍然需要一次测量，因此在视图稳定之前的一两个瞬间不起作用，而在没有逃逸像素的视图上完全不起作用。
# A checkbox indented under "Always fit range" (also in the Color menu).
panel-log-scale = 对数颜色刻度
panel-log-scale-hover =
    按逃逸值的“对数”而不是线性方式分布调色板。在深处，逃逸次数向高端聚集，因此线性映射会把大部分调色板花在边界附近的一层薄壳上，而把其他一切压平。

    适用于归一化映射——也就是“归一化深层颜色”（或导出时的 --normalize）所开启的映射。范围严重偏斜时，无论是否勾选此项，都会采用对数映射。
# The same checkbox's tooltip while it is greyed out (normalization is off).
panel-log-scale-off-hover =
    需要先归一化：此选项决定调色板“如何”映射到实测的逃逸范围，因此在关闭“归一化深层颜色”（且没有 --normalize 导出）时，它没有可以改变的映射，不会改变任何一个像素。

    开启“归一化深层颜色”即可使用它。
# The palette animation: its kind, then its speed.
panel-animate = 动画
panel-anim-speed = 速度
panel-anim-speed-hover = 循环速度：每秒颜色偏移循环次数，或（随机模式下）每秒渐变变化次数。
panel-shuffle-gradient = 随机更换渐变

## The control panel: Quality

panel-auto-iter = 迭代次数随缩放自动调整
# The iteration-count label while auto-scaling is on (the count is then a base the zoom raises).
panel-iterations-base = 迭代次数（基础）
panel-iterations = 迭代次数
panel-iterations-hover = 基础迭代次数。开启“自动调整”后，有效次数会随缩放深度攀升。在你移动时，预览会把迭代次数限制在较低的值（50,000）以保持响应，然后在视图稳定时锐化到完整次数——因此深处的边缘在运动中看起来是平滑的，停下后才会解析出来。
# A warning under the iteration count. $count: the iterations the settled preview renders.
panel-res-limited = 在此深度下，已稳定的预览以降低的分辨率渲染 { $count } 次迭代，以保持响应。导出即可获得全分辨率的细节。
panel-anti-alias = 抗锯齿
# Anti-aliasing off (the other choices are 2×, 3×, 4×, 8×).
panel-anti-alias-off = 关闭
panel-anti-alias-hover = 静态图像的超采样（在视图稳定时应用）。越高越能抑制外部细碎的“尘点”，代价是渲染时间。

## The control panel: Effects

panel-relief-lighting = 3D 浮雕光照
panel-relief-lighting-hover = 利用距离估计的斜率为表面着色——呈现浮雕、受光照的效果。（全纯分形族：Mandelbrot / Multibrot。）
panel-light-angle = 光照角度
panel-relief = 浮雕
panel-relief-hover = 越低浮雕越锐利；越高越柔和/平坦。
panel-rotate-light = 旋转光源
panel-rotate-light-hover = 让光照方向随时间旋转（使用“速度”滑块）。
panel-distance-glow = 距离辉光
panel-distance-glow-hover = 明亮的距离估计轮廓色带，在边界附近变密，形成发光的细丝。（全纯分形族。）
panel-glow = 辉光
panel-band-width = 条带宽度
panel-band-width-hover = 距离轮廓的间距（每条带的倍频程数）。
panel-animate-glow = 辉光动画
panel-animate-glow-hover = 让辉光色带随时间流动（使用“速度”滑块）。

## The control panel: Overlays

panel-orbits-hover = 绘制光标下方那个点的迭代路径。
panel-orbit-anim = 动画（飞驰的点）
panel-orbit-anim-hover = 让一个颜色循环变化的点沿轨道飞驰而出。
panel-orbit-speed = 轨道速度
panel-minimap-hover = 初始视图的小型概览，带有“你在这里”标记和缩放深度。单击它可跳转到某个区域。

## The control panel: Advanced
## Labels before a control sit in a column about 104 px wide; most of these are already wider in English.

# Small headings of the section's two groups.
panel-accelerators = 加速
panel-performance-tuning = 性能调整
panel-bla = BLA 加速（深度缩放）
panel-bla-hover = 双线性近似：在极深处跳过整条轨道上的迭代（floatexp Mandelbrot，≥1e28×）——GPU 渲染快约 5×，输出完全相同（已由自检验证）。默认开启；如需对比或遇到瑕疵，可将其关闭。
panel-series-approx = 级数近似
panel-series-approx-hover = 用多项式为微扰提供初值，以跳过早期的迭代，适用于 BLA 未生效的情况（df32 深度、Multibrot、BLA 关闭）。输出完全相同；如需对比可将其关闭。
panel-glitch-correct = 失真校正（导出）
panel-glitch-correct-hover = 针对导出图像的多参考点失真校正：检测微扰失真，并针对额外的参考轨道重新渲染这些像素，直到干净为止。默认开启。适用于约 32 MP 以内的导出（非 aux 着色）；更大的图像和实时视图使用普通路径。从不用于 Mandelbrot 或 Multibrot 视图（Julia 模式之外）：与精确计算对照后发现，它在这些视图上没有修正任何东西，却让导出时间最多变为原来的 5x。
panel-dive-limit = 自动缩放深入上限
panel-dive-limit-hover = 自动缩放（A 键）停止的深度。在约 1e271× 以内，它会平滑地滑行；更深时，它会切换为不连贯的分步深入，以便快速到达极深处。
# Followed by two choices, Speed and Quality.
panel-autopilot-priority = 自动缩放优先级
panel-priority-speed-hover = 按设定的速度缩放。一旦有带细节的帧准备好，屏幕就会刷新，分辨率取能让刷新持续不断的值（如果缩放跑得比渲染快，会低于“运动时最低分辨率”的下限）；两次刷新之间，上一帧会跟随缩放并变得模糊。
panel-priority-quality-hover = 每次刷新都是完全解析的全分辨率帧，缩放会在需要时放慢，使屏幕上的帧在下一帧到来之前永远不会被放大太多。一种较慢、但每时每刻都清晰的深入。
# Followed by two choices, Detail and Misiurewicz point.
panel-autopilot-target = 自动缩放目标
panel-target-detail-hover = 朝视图中分形边缘上最丰富的结构深入，随深入不断重新选择目标。
panel-target-misiurewicz-hover = 找到最近的 Misiurewicz 点（结构永远重复的螺旋中心），并精确地深入其中。深入开始前会先保持静止，同时将该点求解到深入上限的深度——在普通深度下只需片刻，超过 1e1000× 后会更久。仅限 Mandelbrot 集。
panel-work-budget = 实时渲染预算
panel-work-budget-hover = 深度缩放时实时视图在细节与速度之间的取舍。越高，渲染的分辨率越完整（更清晰——较少出现“软”的放大观感），但帧率越低，而且非常高的值可能会在最重的帧上造成短暂的 GPU 停顿。无论此项如何设置，导出始终为全分辨率。
panel-min-motion-res = 运动时最低分辨率
panel-min-motion-res-hover = 在深度缩放时连续缩放或平移期间，实时视图允许降到的最低分辨率。越高，越能防止快速深入时画面变得块状/像素化（更清晰），代价是帧率——设为 100% 时，深入期间的细节刷新必须以完整开销渲染，这可能使深度缩放出现明显的跳步/卡顿。如果深入感觉顿挫，请调低此值。默认为 30%；不影响已稳定的图像或导出。
panel-finish-sound = 渲染完成时播放声音
panel-finish-sound-hover = 导出或漫游渲染完成时，播放 FRACTINT 的完成提示音：三个上升的音符（1047、1109、1175 Hz，各 100 ms），与 DOS 原版的编码完全一致——合成为带限方波并经过高通滤波，听起来就像当年播放它的那个小小的 PC 喇叭。
panel-prefer-detail = 缩放时优先细节
panel-prefer-detail-hover = 在缩放或平移时，持续显示上一幅完整细节的帧——经缩放并跟随运动——而不是每帧都以降低的质量重新渲染。运动暂停时，视图会完整渲染。关闭时：运动中以降低的分辨率实时渲染（更流畅、更粗糙）。浅层视图（直接模式）始终实时渲染——无论哪种方式，它们每帧都开销低且清晰。

## The control panel: About (the current fractal)

# A link to the fractal's reference page, followed by an arrow.
panel-about-reference = 参考资料

## The control panel: Advanced ▸ Second graphics card

# A label before a drop-down (wider than the panel's label column already).
gpu2-label = 第二显卡
gpu2-off = 关闭
# Another device on the same graphics card as the window (the default).
gpu2-same-card = 同一块显卡
# A card the list does not name. $number: the card's number.
gpu2-card-n = 显卡 { $number }
gpu2-finding = 正在查找本机的显卡…
gpu2-none = 本机没有其他显卡。
gpu2-hover =
    另一个图形设备会在深度视图移动时渲染其中的帧，并在停止后渲染部分用于锐化的额外样本，使画面跟得上缩放，并更快达到最终质量。绘制窗口的显卡仍然绘制所有内容。

    另一块显卡会贡献它自己的速度。两种不同型号的显卡会把少数像素画得略有不同；如果第二显卡出错，视图会只用一块显卡继续。

    “同一块显卡”（默认）会在这块显卡上打开第二个设备。它不会增加速度，但渲染时无需等待屏幕的帧。它与窗口共享显卡：导致显卡重置的故障会结束程序，就像窗口自身的渲染出错时一样，下次启动时会提议将其关闭。

## The view (canvas)

# A notice after Ctrl+click pins the Julia c at a deep zoom.
canvas-julia-pin-quantized = 已钉住 Julia c——但它以双精度保存，因此在此深度下整个面板会舍入为同一个值，钉住的点会落在视图中心。请缩小后再瞄准。
# A small caption in the top-left corner of the view while the orbit overlay is fitted to the view (its "Normalize (fit to view)" option).
canvas-orbit-normalized = 轨道（已归一化）
# The minimap's tooltip.
canvas-minimap-hover = 概览 / 你在这里。拖动可在当前缩放下平移，滚动可放大和缩小，单击可跳到该区域（初始缩放）。

## The timestamp overlay (View ▸ Show timestamp)

# Under the large elapsed-time clock. $frame: the frame number (as the log writes it, without grouping).
overlay-timestamp-frame = 帧 { $frame }

## File ▸ Settings ▸ Reference cache…

cache-title = 参考轨道缓存
cache-intro = 在极深处，视图中最慢的部分是它的参考轨道——最多可能需要一个小时的任意精度运算。Fractadyne 把已完成的轨道保存在这里，因此回到你访问过的位置，或在它附近缩放时，只需几秒钟。一条轨道约 4 MB。
cache-off = 本次运行已关闭（任务调用，或 --no-orbit-cache）。在下一次正常启动之前，不会读取或写入任何内容。
# Small headings of the window's parts.
cache-location = 位置
cache-usage = 用量
cache-limit = 上限
cache-no-config = （没有可用的配置目录）
cache-open-folder = 打开文件夹
# The text on the usage bar. $used and $limit: sizes such as "1.2 GB"; $count: the orbits kept.
cache-usage-bar = { $used } / { $limit }——{ $count } 条轨道
cache-limit-hover = 当缓存超过此大小时，会首先移除构建开销“最低”的轨道——一条花了一个小时构建的深层轨道，会比任意数量的快速轨道保留得更久，无论那些快速轨道最近是否用过。
# Beside the limit. $count: about how many orbits of the live view's length fit in it.
cache-limit-estimate = ≈ { $count } 条实时视图上限长度的轨道
cache-kept = 已保存（最有价值的在前）
# Column headers of the list of kept orbits.
cache-col-iterations = 迭代次数
cache-col-precision = 精度
cache-col-size = 大小
cache-col-last-used = 上次使用
# The precision column. $bits: the precision in bits.
cache-row-bits = { $bits } 位
# Under the list. $count: the orbits not listed.
cache-rows-more = … 还有 { $count } 条
# When an orbit was last used (the "last used" column).
cache-ago-now = 刚刚
# $count: minutes.
cache-ago-minutes = { $count } 分钟前
# $count: hours.
cache-ago-hours = { $count } 小时前
# $count: days.
cache-ago-days = { $count } 天前
# Asked before clearing. $count: the orbits; $size: their total size, such as "1.2 GB".
cache-confirm = 删除 { $count } 条轨道（{ $size }）？重建它们花费的是时间，不会丢失数据。
cache-delete = 删除
cache-cancel = 取消
cache-clear = 清空缓存…
cache-clear-hover = 删除所有已存储的轨道。会先询问。
cache-close = 关闭
# $count: the orbits removed.
cache-cleared = 参考轨道缓存已清空——已移除 { $count } 条轨道。
# $why: the system's error message.
cache-clear-failed = 无法清空参考轨道缓存：{ $why }

## Help ▸ Diagnostics…

diag-title = 诊断
diag-intro = 运行测试，检查 Fractadyne 能否在你的硬件上正常工作。结果保存在你的计算机上，并可附加到问题报告中——不会自行发送到任何地方。
# $status: which arbitrary-precision library is in use, as the program reports it (in English).
diag-arithmetic = 深度缩放算术：{ $status }
# The three tests' buttons, each followed by a sentence saying what it does.
diag-run-selftest = 运行自检
diag-run-uitest = 运行界面测试
diag-run-gputest = 运行 GPU 算术检查
diag-blurb-selftest = 将 GPU 上的数学计算和渲染结果与已知正确的结果对照。大约需要 15 秒。
diag-blurb-uitest = 遍历界面和实时视图，并在几个缩放深度下截取屏幕截图。需要一两分钟，运行期间会打开窗口。
diag-blurb-gputest = 在你的计算机提供的每一个图形后端上，将着色器的双浮点算术与已知正确的值对照。需要几秒钟，不打开窗口。如果它报告失败，那是我们很想看到的一项真实发现——请将它附加到报告中。
# While a test runs. $count: the checks finished so far.
diag-checks-done = 已完成 { $count } 项检查
# A finished test's result. $test: the test's name (Self-test, UI test, GPU arithmetic check).
diag-verdict-captured = { $test }——已记录结果
diag-verdict-passed = { $test }——通过
diag-verdict-problems = { $test }——报告了问题
# Under the GPU arithmetic check's result.
diag-informational-note = 这项检查没有通过或失败之分：它记录的是你的 GPU 如何计算，报告的失败是一项真实发现，而不是你计算机的故障。无论哪种情况，把它附加到报告中都是对它最有用的处理方式。
# Under a test's result when it reported problems.
diag-problems-note = 在参考 GPU 以外的硬件上，出现一些差异是预料之中的——尤其是图像比较。把它附加到问题报告中，是对它最有用的处理方式。
diag-open-results = 打开结果
diag-attach = 附加到问题报告…
diag-attach-hover = 打开报告对话框并附上此结果，使 issue 带有经机器验证的测试，而不只是一段描述
diag-console = 将诊断信息输出到控制台
diag-console-hover = 从现在起在 stderr 上显示 [fd-…] 行。不带参数启动 Fractadyne 时默认关闭。这不会改变“记录”的任何内容——日志文件始终会获得每一行——因此除非你在看终端，否则请保持关闭。仅在本次会话中有效；使用 --console 参数或 FRACTADYNE_CONSOLE=1 也可以捕获启动过程。
diag-close = 关闭
# Errors starting a test. $why: the system's error message; $dir: a folder.
diag-error-no-exe = 找不到可执行文件：{ $why }
diag-error-create-dir = 无法创建 { $dir }：{ $why }
diag-error-start = 无法启动测试：{ $why }

## Navigate ▸ Misiurewicz explorer…

misi-title = Misiurewicz 浏览器
misi-mandelbrot-only = Misiurewicz 浏览仅适用于 Mandelbrot——请切换分形后再使用。
# While the gallery is being built. $seconds: the time so far, such as "3".
misi-solving = 正在求解小类型族（{ $seconds }s）…
# Above the gallery. $total: all points; $curated: the hand-picked ones; $found: the ones the sweep found.
misi-summary = { $total } 个点——{ $curated } 个精选，{ $found } 个由对小类型进行的带种子牛顿扫描找到（上半平面；粗网格能找到大多数吸引域，但无法证明能找到全部）。
# The name of a point the sweep found. $kind: its type, such as "(3,1)"; $point: its coordinate, such as "-0.1011+0.9563i".
misi-entry-name = { $point } 处的 { $kind }
# Under each thumbnail. $kind: the point's type, such as "(3,1)"; $octaves: the zoom period in octaves, such as "3.17"; $twist: the twist per repeat in degrees, such as "+12.5". "oct" is short for octaves.
misi-entry-readout = { $kind } · { $octaves } 倍频程/重复 · { $twist }°
# Before the depth field.
misi-zoom-to = 缩放到
misi-cancel = 取消
misi-solve-jump = 求解并跳转
misi-solve-jump-hover = 以该点存储的坐标为种子，用牛顿法将这个精确点求解到所要求深度需要的精度，然后直接跳到那里。要求越深耗时越长——求解采用任意精度。
misi-select-hint = 选择一个点、指定深度，然后跳转——每个缩略图下方的乘子行，就是在那里深入时的样子：视图重复之前要经过多少个倍频程，以及每次重复扭转多少。
misi-cancelled = 已取消求解。
# $text: what was typed in the depth field.
misi-bad-zoom = 无法将“{ $text }”读取为放大倍数（请写 1e50、3.2e120、…）。
misi-no-kp = 此条目没有可用于求解的已检测 (k,p)。
misi-parse-failed = 无法解析存储的坐标。
# A notice after the jump. $name: the point's name; $zoom: the magnification, such as "1e50"; $seconds: how long the solve took, such as "1.2".
misi-zoomed = 已缩放到 { $name }——{ $zoom }×（{ $seconds }s）
# Added after the notice above when the iteration count is too low. $have: the fixed iteration count; $typical: what the depth typically needs.
misi-iter-note = 迭代次数固定为 { $have }；这个深度通常需要约 { $typical }——如果视图渲染成一片平坦，请提高该值（或启用自动迭代）。
# $name: the point's name; $why: why the solve failed (a technical word).
misi-solve-missed = 深层求解没有落在 { $name } 上：{ $why }。存储的点在其自身的深度上仍然是精确的——请尝试一个较浅的目标。

## Life (Game of Life): the side panel

# Shown when the graphics card cannot run the Life view.
life-no-compute = 此图形适配器无法运行生命游戏（不支持计算着色器）。
# Row label: the pattern list (pattern names stay in English).
life-pattern = 图样
life-open = 打开…
life-open-hover = 打开 RLE、.cells 或 Life 1.05/1.06 图样文件。
life-pattern-text = 图样文本…
life-pattern-text-hover = 粘贴图样（LifeWiki 所提供的 RLE，或纯文本）。
life-save = 保存…
life-save-hover = 将宇宙的当前状态保存为 RLE。
# Row label: the rule list and the typed rule (B3/S23 notation).
life-rule = 规则
# Shown in the rule list when the rule in use is none of the library's.
life-rule-custom = 自定义
life-apply = 应用
# Playback buttons (a pause or play symbol is shown before them).
life-pause = 暂停
life-play = 播放
life-play-hover = 运行宇宙（Enter）。
life-step = 单步
life-step-hover = 前进一代（N）。
# Tooltip of the button beside Step, which reads "+16", "+1,024"…: that many generations.
life-stride-hover = 前进这么多代（Shift+N）。
life-reset = 重置
life-reset-hover = 回到刚打开时的图样（第 0 代）。
life-speed = 速度
# After the speed slider's number: generations a second. Short.
life-speed-unit = 代/秒
# Row label: how many generations the "+n" button advances.
life-stride = 步长
# The panel's numbers.
life-generation = 代数
life-population = 活细胞数
life-tiles = 图块
# $count: the tiles stored; $capacity: how many the GPU pool holds.
life-tiles-of = { $count } / { $capacity }
life-tiles-hover = 已存储的 64×64 细胞图块（每个 4 KiB）以及 GPU 池的容量。
# Row label: what a left-drag in the view does (Pan or Draw).
life-mouse = 鼠标
life-pan = 平移
life-pan-hover = 拖动即可移动视图。
life-draw = 绘制
# The Draw tool's tooltip; the toolbar's shows it after "Draw (D):", so it starts lowercase.
life-draw-hint = 单击细胞可将其翻转；拖动可涂画——从死细胞开始的笔画会绘制，从活细胞开始的笔画会擦除。正在运行的宇宙在你绘制时会暂停，松开后继续运行。
life-undo = 撤销
life-undo-hover = 回到上一次编辑、填充、清空或规则更改之前的宇宙（绘制时按 Ctrl+Z）。
life-grid = 网格线
life-grid-hover = 放大到每个细胞 8 像素或以上时，显示细胞边框。
# A random fill: "Random" labels the density field, "seed" the seed field after it.
life-random = 随机
life-density-hover = 填充时设为活细胞的比例。
life-seed = 种子
life-fill-view = 填充视图
# $cells: the largest side of the filled area, in cells (2,048).
life-fill-view-hover = 随机填充可见区域（以其中心为准，最大 { $cells } × { $cells } 个细胞）。
life-clear = 清空
life-clear-hover = 杀死所有细胞（“撤销”可将它们恢复）。

## Life: notices in the panel, and file dialogs

# $path: the file written.
life-saved = 已保存 { $path }
# $error: why (from the system, in English).
life-save-failed = 保存失败：{ $error }
# The universe stopped running. $error: why (in English).
life-stopped = 已停止：{ $error }
# A pattern file named a rule that does not read. $error: why (in English); $rule: the rule kept, in B3/S23 notation.
life-rule-not-understood = 无法理解图样的规则（{ $error }）；保留 { $rule }
# $path: the file; $error: why (in English).
life-open-failed = 无法打开 { $path }：{ $error }
# File-dialog filter names.
life-filter-patterns = 生命游戏图样
life-filter-rle = 游程编码图样

## Life: the toolbar (its buttons are icons; these are their tooltips)

life-toolbar-pause-hover = 暂停宇宙（Enter）
life-toolbar-play-hover = 运行宇宙（Enter）
# $stride: the generations a Shift+click steps.
life-toolbar-step-hover = 前进一代（N）；Shift+单击或 Shift+N 前进 { $stride } 代
life-toolbar-reset-hover = 回到刚打开时的图样（第 0 代）
# $hint: the Draw tool's tooltip (life-draw-hint).
life-toolbar-draw-hover = 绘制（D）：{ $hint }

## Life: the Pattern text window

life-text-title = 图样文本
life-text-intro = 粘贴图样：RLE（LifeWiki 和 Golly 所提供的格式）、纯文本 .cells，或 Life 1.05 / 1.06。
life-text-load = 加载
life-text-current = 当前图样的 RLE
life-text-current-hover = 用最近一次加载的宇宙填充此框。

## L-systems: the side panel

# Row label: the system list (system names stay in English).
lsys-system = 系统
lsys-open = 打开…
lsys-open-hover = 打开 Fractint 的 .l 文件或 .lsys 系统。
lsys-edit = 编辑…
lsys-edit-hover = 系统的文本：公理、产生式、角度——编辑并应用。
lsys-save = 保存…
lsys-save-hover = 将系统保存为 .lsys 文本文件。
# Tooltip of the "SVG…" button.
lsys-svg-hover = 将视图导出为 SVG 图形：其线条和形状为矢量，并保留原有颜色。
lsys-follow = 阶数跟随缩放
lsys-follow-hover = 阶数随着放大而提高，因此细节永远不会耗尽。
lsys-follow-fixed-hover = 此系统并非每一阶都按固定倍数增长，因此以固定的阶数绘制。
lsys-order = 阶数
lsys-angle = 角度
lsys-sweep-hover = 来回扫动角度：曲线随着转动而变形
# A small button that puts back the system's own angle after the slider moved it.
lsys-system-angle = 系统角度
lsys-seed = 种子
lsys-seed-hover = 随机选择所依据的值：相同的种子，相同的植物。
lsys-seed-new = 新种子
lsys-seed-new-hover = 另一株植物：一个新的种子。
lsys-line-width = 线宽
# Row label of a percentage slider: how much of the curve is drawn.
lsys-draw-on = 绘制进度
lsys-draw-on-hover = 曲线已绘制的比例，按海龟绘制的顺序
lsys-draw-on-play-hover = 曲线逐步绘制出来
lsys-colour-by = 着色依据
# The colourings, in the "Colour by" list.
lsys-colour-position = 沿曲线的位置
lsys-colour-depth = 分支深度
lsys-colour-heading = 朝向
# C, < and > are the system's commands that set the colour index: keep them.
lsys-colour-index = 颜色索引（C、<、>）
lsys-colour-plain = 单一颜色
# The panel's numbers.
lsys-growth = 增长
lsys-growth-built = 按其阶数构建为字符串
lsys-growth-built-hover = 参数化或上下文相关：一个模块的重写取决于它的数值或相邻模块，因此字符串一次构建一代（最多两百万个模块），并以双精度绘制——阶数不跟随缩放。
# How much the picture grows from one order to the next. $factor: the factor (1.4142).
lsys-growth-factor = 每阶 ×{ $factor }
lsys-growth-factor-period-2 = 每阶 ×{ $factor }（阶数以 2 为步长）
lsys-growth-none = 无（固定阶数）
lsys-drawn = 已绘制
# $drawn: what was drawn (lsys-drawn-segments, lsys-drawn-shapes or lsys-drawn-segments-shapes); $order: the order.
lsys-drawn-at-order = { $drawn }，阶数 { $order }
# The same, when the walk stopped at its segment budget.
lsys-drawn-at-order-budget = { $drawn }，阶数 { $order }（预算）
# $segments: the line segments drawn.
lsys-drawn-segments = { $segments } 条线段
# $shapes: the filled shapes drawn.
lsys-drawn-shapes = { $shapes } 个填充形状
# Both counts. $segments: line segments; $shapes: filled shapes.
lsys-drawn-segments-shapes = { $segments } 条线段，{ $shapes } 个填充形状
# $ms: the walk's time in milliseconds (12.3).
lsys-drawn-hover = 遍历耗时 { $ms } ms；视图外的子树会被跳过，小于一个像素的子树绘制为一条线段
# A word too long to build at the order asked for. $wanted: the order asked for; $built: the order drawn.
lsys-drawn-short = 阶数 { $wanted } 的字符串超过两百万个模块：以阶数 { $built } 绘制。
# $order: the deepest order the app draws.
lsys-deepest-order = 已处于绘制的最深阶数（{ $order }）：继续放大不会增加细节。

## L-systems: notices in the panel and the editor, and file dialogs

# $path: the file; $error: why (in English).
lsys-open-failed = 无法打开 { $path }：{ $error }
# Why a file was not opened (shown after "Could not open …:").
lsys-file-too-large = 大于 4 MiB
# Why a Fractint .l file was not opened: it holds no entries. `Name { … }` is the file's syntax: keep it.
lsys-no-entries = 没有条目（.l 文件包含 `Name {"{"} … {"}"}` 形式的条目）
# $path: the file written.
lsys-saved = 已保存 { $path }
# $error: why (from the system, in English).
lsys-save-failed = 保存失败：{ $error }
# After File ▸ Export SVG. $path: the file written.
lsys-svg-exported = 已导出 { $path }
# $error: why (from the system, in English).
lsys-svg-failed = 导出失败：{ $error }
# File-dialog filter names: opening (any system file) and saving (one system).
lsys-filter-systems = L 系统
lsys-filter-system = L 系统

## L-systems: the toolbar

lsys-toolbar-lower-hover = 降低阶数（并将其固定）
lsys-toolbar-raise-hover = 提高阶数（并将其固定）
# Between the − and + buttons, in a fixed-width font. $order: the order, padded to 3 characters so the label keeps its width.
lsys-toolbar-order = 阶数 { $order }
# A toggle: the order follows the zoom. Short.
lsys-toolbar-auto = 自动
lsys-toolbar-auto-hover = 阶数跟随缩放
lsys-toolbar-edit-hover = 编辑系统的文本

## L-systems: the editor window, and a .l file's entries

lsys-editor-title = L 系统
# The editor's help. The quoted examples ('X = word', 'seed n', 'define R 1.456'…), the commands and "The Algorithmic Beauty of Plants" (a book's title) are the system's syntax and names: keep them as they are.
lsys-editor-help = 每行一个键：angle（角度，或用 /n 表示圆周的 n 等分）、axiom、产生式 'X = word'，以及可选的 heading、draw / move / variables、colour、order。一个带有随机选择备选项的符号，每个备选项写一行，并带上其权重：'X (0.3) = word'；'seed n' 用于选择植物。在一个字符串中，{"{"} {"}"} 会填充海龟的路径，. 会添加一个顶点。参数化和上下文相关的产生式（The Algorithmic Beauty of Plants）：'A(s) : s > 1 = F(s)[+A(s/2)]'、'b < a > c = b'，以及 'define R 1.456' 和 'ignore +-F'；F(l) 前进 l，+(a) 转动 a 度。Fractint 的 .l 条目可以原样粘贴。
lsys-editor-apply = 应用
lsys-editor-revert = 还原
lsys-editor-revert-hover = 所显示系统的文本。
# The window listing a .l file's entries to pick from. $file: the file's name.
lsys-entries-title = { $file }——条目
# Tooltip of an entry that does not parse (its error is shown beside it, in English).
lsys-entry-unreadable-hover = 此条目无法读取。

## Export: opening a view or location (File ▸ Open view or location…)

# File-dialog filter names.
export-filter-view = Fractadyne 视图或位置
export-filter-image = 图像（PNG / EXR）
export-filter-location = 位置（.fdn / .kfr）
# $zoom: the magnification (1.2e148), followed by the magnification sign ×.
export-loaded-location = 已加载位置 @ { $zoom }×
# $zoom: the magnification; $note: what was noteworthy in the file (in English).
export-loaded-location-note = 已加载 @ { $zoom }×——{ $note }
# $path: the file.
export-not-location = { $path } 不是 Fractadyne 位置。
export-file-too-large = 文件过大（不是 .fdn 位置？）。
# $path: the file; $error: why (from the system, in English).
export-read-failed = 无法读取 { $path }：{ $error }
# A Kalles Fraktaler .kfr file did not import. $error: why (in English).
export-import-failed = 导入失败：{ $error }
# $path: an image whose embedded text is not a view.
export-not-view = { $path } 带有 Fractadyne 元数据，但不是视图。
# $path: the image the view was read from.
export-loaded-view = 已从 { $path } 加载视图
# $path: the image; $note: what was noteworthy in it (in English).
export-loaded-view-note = 已从 { $path } 加载视图——{ $note }
export-no-metadata = 该文件中没有嵌入的 Fractadyne 视图元数据。

## Export: notices after an export or a snapshot
## "File → Export image…" in them names the menu items (menu-file, menu-file-export).

# The export window's status while the image renders.
export-rendering = 正在渲染…
export-canceled = 导出已取消。
# $error: why (in English).
export-failed = 导出失败：{ $error }
# $size: the image's size in pixels (1920×1080); $path: the file written.
export-saved = 已保存 { $size } → { $path }
# An L-system export whose walk stopped at its budget. $size: pixels; $segments: the budget (4,000,000); $path: the file.
export-saved-incomplete = 已保存 { $size }（不完整：超出了 { $segments } 条线段的预算——请降低阶数）→ { $path }
# A two-panel export written as two files. $path: the first of them.
export-saved-two = 已保存 2 个文件 → { $path }
# $size: pixels; $path: the file written.
export-saved-corrected = 已保存 { $size }（已校正失真）→ { $path }
# $path: the first of the two files.
export-saved-two-corrected = 已保存 2 个文件（已校正失真）→ { $path }
# Added after a finished export's notice. $duration: how long it took (8.3s, 1m 04.0s).
export-elapsed = （用时 { $duration }）
export-preparing-deep = 正在准备深度导出——正在构建参考轨道（这可能需要一段时间）…
# $size: pixels (5120×4035); $ss: the supersampling factor; $iterations: the iteration count.
export-large = 大尺寸导出（{ $size }、{ $ss }× 超采样、{ $iterations } 次迭代）——正在后台渲染；这种尺寸的导出会跳过失真校正…
export-life-unavailable = 暂不支持导出生命游戏视图的图像——请使用屏幕截图，或“文件 ▸ 保存生命游戏图样”。
export-already-running = 已有一个导出正在运行——“文件 → 导出图像…”会显示其进度。
# $size: the image's size in pixels.
export-snapshot-background = 快照：正在后台渲染 { $size }——“文件 → 导出图像…”会显示进度，并可将其取消。
export-no-gpu = GPU 不可用
# A screen-capture snapshot. $size: pixels; $path: the file written.
export-snapshot-saved = 已保存屏幕快照 { $size } → { $path }
# $error: why (in English).
export-snapshot-failed = 快照失败：{ $error }

## The colour picker (the popup of a colour swatch)

# The third row's label: the colour as hexadecimal bytes. Short (about 6 characters).
picker-row-hex = Hex
picker-hex-hover = 单击以输入十六进制值
picker-cancel = 取消
picker-cancel-hover = 恢复打开此取色器时的颜色
picker-ok = 确定
picker-ok-hover = 保留此颜色
# Starts the screen eyedropper (an icon is shown before it).
picker-pick = 取色
picker-pick-hover = 从屏幕上任意位置取色
# Tooltip of the disabled Pick button (in the picker and the gradient editor) on Linux and macOS.
picker-eyedropper-unsupported = 从屏幕上的其他位置取色需要平台的屏幕捕获 API。只有 Windows 版本具备；在 Linux 上这需要依赖 X11，而 Wayland 需要一个带有自己权限提示的门户（portal）。

## The fractal families: what each one is (the info panel's "About" section and the pickers' hover text).
## The formulas themselves (z -> z^2 + c) are written in code and stay as they are.

fractal-mandelbrot-about = 典型的逃逸时间分形：由使 0 的轨道保持有界的所有 c 组成的集合。它的边界无限复杂。
fractal-multibrot3-about = 一种 Multibrot 集——更高幂次下的 Mandelbrot 构造。幂次 d 带来 (d-1) 重旋转对称性。
fractal-multibrot4-about = 幂次为 4 的 Multibrot：三重对称，宽大的芽苞。
fractal-multibrot5-about = 幂次为 5 的 Multibrot：四重对称。
fractal-tricorn-about = Tricorn（Mandelbar）：每一步都对 z 取共轭。这种反全纯映射产生一个有三个角的形状。
fractal-burning-ship-about = 在平方之前先对 z 的各部分取绝对值；结果像一艘燃烧的船。
fractal-celtic-about = Burning Ship 的近亲，只对 z^2 的实部取绝对值，产生凯尔特结 / 心形图案。
fractal-buffalo-about = 一种绝对值变体，对 z^2 的两个分量都取绝对值。
fractal-phoenix-about = Phoenix 还会用到上一次的迭代值，产生火焰般的细丝。可通过 Julia 模式尝试它的 Julia 形式。
fractal-newton-about = 求解 z^3 = 1 的牛顿求根迭代，按每个点收敛的快慢着色。一种收敛（而非逃逸）分形。
fractal-multibrot6-about = 幂次为 6 的 Multibrot：五重对称。
fractal-multibrot7-about = 幂次为 7 的 Multibrot：六重对称，芽苞变细并趋近于一个圆。
fractal-multibrot8-about = 幂次为 8 的 Multibrot：七重对称。
fractal-burning-ship3-about = 幂次为 3 的 Burning Ship：先取绝对值，再求立方。
fractal-burning-ship4-about = 幂次为 4 的 Burning Ship。
fractal-burning-ship5-about = 幂次为 5 的 Burning Ship。
fractal-tricorn3-about = 幂次为 3 的 Tricorn（Mandelbar）：四重对称。
fractal-tricorn4-about = 幂次为 4 的 Tricorn：五重对称。
fractal-tricorn5-about = 幂次为 5 的 Tricorn：六重对称。
fractal-celtic3-about = 幂次为 3 的 Celtic 折叠：对 z^3 的实部取绝对值。
fractal-celtic4-about = 幂次为 4 的 Celtic 折叠。
fractal-celtic5-about = 幂次为 5 的 Celtic 折叠。
fractal-buffalo3-about = 幂次为 3 的 Buffalo：对 z^3 的两部分都取绝对值。
fractal-buffalo4-about = 幂次为 4 的 Buffalo。
fractal-buffalo5-about = 幂次为 5 的 Buffalo。
fractal-custom-about = 以 Fractint 风格表达式写成的公式（分形 ▸ 自定义公式…）。大多数公式与内置分形族一样，可通过微扰进行深度缩放；公式对话框会说明你的公式是否支持。
fractal-life-about = 无界平面上的 Conway 生命游戏及其同类：Generations 规则、非总和型（Hensel）规则、图样文件。播放、单步、绘制。
fractal-lsystem-about = Lindenmayer 系统：由海龟根据重写文法绘制的曲线、空间填充曲线、岛屿和植物。阶数跟随缩放，因此细节永远不会耗尽。可直接打开 Fractint 的 .l 文件。

# Shown where the other families show their formula (z -> z^2 + c).
fractal-custom-formula = z -> 你编写的公式
# Shown where the other families show their formula. "B3/S23" is the rule's notation: keep it.
fractal-life-formula = B3/S23：有 3 个活邻居时诞生，有 2 或 3 个时存活
# Shown where the other families show their formula. "F = F+F--F+F" is the grammar's notation: keep it.
fractal-lsystem-formula = F = F+F--F+F：每个符号被重写，然后由海龟绘制

# The last paragraph of the picker's hover text for a family that renders only on the direct path.
fractal-no-deep-zoom = 仅直接渲染——此分形族不支持深度缩放。

## Names of settings, shown in the Settings menu and the welcome window.

theme-dark = 深色
theme-light = 浅色
# The update track that follows only stable releases.
update-track-stable = 稳定版
# The update track that also offers pre-releases (betas).
update-track-beta = Beta（预发布版）

## The status bar (its hover texts; the readouts follow in their own section)

status-period-hover = 你跳转到的迷你 Mandelbrot 的周期（按 M，或使用“转到位置 ▸ 转到特征”）。只要视图停留在该特征上，它就会保留，离开后清除。

## The playback transport: the player over the view while a tour plays.

# $close: the close button's icon, as drawn on that button.
play-finished-hover = 已结束——拖回去重新查看，或单击 { $close } 关闭
# $kind: "benchmark" or "script" (the variants hold the wording); $name: the tour's name.
play-playing-hover =
    { $kind ->
        [benchmark] 正在播放基准测试“{ $name }”
       *[other] 正在播放脚本“{ $name }”
    }
play-restart = 从头重新开始
play-back = 后退 10 秒
play-again = 从头再次播放
play-resume = 继续
play-pause = 暂停
play-stop = 停止并回到开头
play-forward = 前进 10 秒
play-speed-hover = 播放速度（单击可在 0.5× / 1× / 2× / 4× 之间循环）
play-render = 将此脚本渲染为帧序列…
play-render-none = 内置基准测试没有可渲染的脚本文件
play-open = 播放其他脚本…
play-loop = 漫游播放到结尾时重复
play-close = 关闭播放器并恢复你自己的视图设置
# A notice beside the scrub bar while the renderer holds the tour's clock. Keep it short.
play-waiting = 等待细节
# "[playback] pace" is a setting in the script file: keep it as written.
play-waiting-hover = 渲染器解析此视图时（参考轨道构建 / 迭代预算攀升），漫游时钟会暂停。播放会自动恢复；参见脚本中的 [playback] pace。
play-scrub = 拖动可在漫游中快速定位
# $behind: how far the script clock is behind real time, as minutes:seconds (1:05).
play-scrub-behind =
    拖动可在漫游中快速定位
    脚本时钟比实际时间落后 { $behind }（节奏控制器放慢了漫游，让渲染器跟得上）。上方的刻度标出按墙钟时间播放时应在的位置。

## The autopilot's notices (toasts). Where a notice has two lines, the first is the verdict and the second what to do about it.

autopilot-on-detail = 自动巡航已开启——正朝细节深入（任何输入都会使其停止）
autopilot-off = 自动巡航已关闭
autopilot-misi-needs-mandelbrot =
    自动巡航：Misiurewicz 目标需要 Mandelbrot 集
    改为朝细节深入
autopilot-on-misi =
    自动巡航已开启——正在查找最近的 Misiurewicz 点…
    任何输入都会使其停止
# $seconds: how long the search has run, in whole seconds.
autopilot-misi-finding =
    自动巡航：正在查找最近的 Misiurewicz 点…
    { $seconds } s · 任何输入都会使其停止
autopilot-misi-not-found =
    自动巡航：在此视图附近未找到 Misiurewicz 点
    查找器可能无法到达这个深度——请尝试“细节”目标
# ($preperiod,$period) names the point, as mathematicians write it: keep the notation. $widths: a power of two (2^$widths screen widths).
autopilot-misi-off-screen =
    自动巡航：最近的 Misiurewicz 点 ({ $preperiod },{ $period }) 在屏幕之外
    相距约 2^{ $widths } 个屏幕宽度——请靠近一个螺旋中心
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-not-solved =
    自动巡航：此视图附近的 Misiurewicz 点 ({ $preperiod },{ $period }) 未能求解
    请尝试“细节”目标
autopilot-misi-refused = 自动巡航：Misiurewicz 求解被拒绝（数值对无效）
autopilot-misi-stopped = 自动巡航：Misiurewicz 求解器意外停止
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-diving =
    自动巡航：正在深入 Misiurewicz ({ $preperiod },{ $period })
    任何输入都会使其停止
# ($preperiod,$period) names the point: keep the notation. $octaves: how often the structure repeats (2.37); $twist: its turn in degrees, signed (+12.5).
autopilot-misi-diving-repeat =
    自动巡航：正在深入 Misiurewicz ({ $preperiod },{ $period })
    结构每 { $octaves } 个倍频程重复一次，扭转 { $twist }° · 任何输入都会使其停止
# $limit: the magnification, as written (1e300).
autopilot-dive-limit = 自动巡航：已到达深入上限（约 { $limit }×）
# $have: the fixed iteration count; $typical: what this depth typically needs.
autopilot-dead-end-short =
    自动巡航已停止：没有可瞄准的目标了——迭代次数固定为 { $have }，而这个深度通常需要约 { $typical }
    请启用自动迭代或提高迭代次数
# $limit: the fixed iteration count.
autopilot-dead-end-cap =
    自动巡航已停止：没有可瞄准的目标了——视图中的一切都达到了固定的迭代上限 { $limit }
    请启用自动迭代或提高迭代次数
autopilot-dead-end = 自动巡航：前方没有细节（已停止）

## Tours (camera scripts)

# The name of the file type in the file dialog's filter list.
script-file-filter = Fractadyne 脚本（TOML）
# $name: the script's name.
script-finished = 脚本已结束——“{ $name }”
# The title of the notice that follows (the file's path and the error, in English).
script-load-failed = 无法加载脚本

## Installing a language file

# The name of the file type in the file dialog's filter list.
language-file-filter = Fractadyne 语言文件

## The notice when the graphics driver's shader cache stops keeping shaders. The text is shown in a fixed-width font with its line breaks as written: break the lines where they read well.

driver-cache-title = 显卡驱动程序没有保留着色器
# $count: how many shaders compiled again; $pipeline: the slowest one's name (a technical id: keep it); $seconds: its time in seconds (7.5).
driver-cache-body =
    本次会话中，你的显卡驱动程序再次编译了 Fractadyne 的 { $count } 个
    着色器，尽管它之前已经编译过其中每一个（最慢的一个：
    { $pipeline }，{ $seconds } s）。它的着色器缓存没有保留已编译的
    着色器，最常见的原因是缓存已满。在此情况改变之前，导出、深度
    视图以及启动 Fractadyne 都可能各自花费数秒进行编译。
# $folder: the cache folder's path; $size: its size in MB (1024). "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia-folder =
    修复方法：关闭 Fractadyne，然后删除以下文件夹中的内容
      { $folder }   （{ $size } MB）
    驱动程序会重建缓存。为防止缓存再次填满，请调高
    NVIDIA 控制面板 > 管理 3D 设置 > 着色器缓存大小。
# "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia =
    修复方法：调高 NVIDIA 控制面板 > 管理 3D 设置 > 着色器缓存大小，
    或清除驱动程序的着色器缓存。
# "AMD Software > Graphics > Advanced > Reset Shader Cache" names AMD's own settings: use the names AMD's software shows in this language.
driver-cache-fix-amd = 修复方法：AMD Software > 显卡 > 高级 > 重置着色器缓存。
driver-cache-fix-other = 修复方法：清除显卡驱动程序的着色器缓存。

## Why a language file could not be installed: each ends the sentence "{ $file } is not a usable language file: …".

language-install-not-utf8 = 不是 UTF-8 文本
language-install-too-large = 文件过大
# "fractadyne-<language>.ftl" is the file name to use: keep it as written.
language-install-no-tag = 文件没有说明它是哪种语言（请将其命名为 fractadyne-<language>.ftl）
language-install-english = 英语是内置的
language-install-empty = 它没有翻译此版本的任何消息
language-install-no-folder = 没有设置文件夹

## The notice after a session that did not end cleanly, when its iteration setting was set aside.

# $base: the iteration count automatic iterations start from.
quarantine-setting-auto = 自动迭代（基础 { $base }）
# $count: the fixed iteration count.
quarantine-setting-fixed = { $count } 次迭代
# $end: "device" when the session lost its graphics device, "unclean" otherwise (the variants hold the wording). $depth: the magnification, as written (1.2e148). $setting and $restored: an iteration setting (quarantine-setting-auto or -fixed).
quarantine-notice =
    { $end ->
        [device] 上一个会话在 { $depth }× 处、设置为“{ $setting }”时丢失了图形设备。本次会话改为以“{ $restored }”启动，因此不会径直回到同样的负载中。
       *[other] 上一个会话在 { $depth }× 处、设置为“{ $setting }”时没有正常结束。本次会话改为以“{ $restored }”启动，因此不会径直回到同样的负载中。
    }

## The second graphics card's status line, under its setting in the control panel.

gpu2-state-off = 关闭：由此窗口的显卡渲染所有内容。
gpu2-state-starting = 正在启动…
# $name: the card's name (and its graphics API).
gpu2-state-running = 正在使用：{ $name }
# $why: the error, as the graphics driver reports it.
gpu2-state-failed = 无法启动：{ $why }
# $name: the card's name.
gpu2-state-lost = 已停止：{ $name } 已丢失。从现在起只用一块显卡；再次选择它即可重试。
# $state: one of the lines above. "--worker-gpu" is a command-line option: keep it.
gpu2-state-pinned = { $state }（由 --worker-gpu 设置）

## The custom-formula dialog: how deep the formula renders (one line under the formula).

formula-depth-deep = 通过微扰进行深度缩放，在 1e28x 之后与内置分形一样使用扩展范围（每一步都完整迭代：自定义公式暂不支持级数近似或 BLA）。
# $why: what the formula has that keeps it off the deep-zoom path (one of the formula-depth-why messages below).
formula-depth-direct = 直接渲染（{ $why }：尚无深度缩放形式）：直到 c 的一个单精度步长跨越一个像素之前都是清晰的——大约为 1e4x 到 1e5x，在远离原点处则更浅。
formula-depth-why-init = 初始化段
formula-depth-why-bailout = 自己的逃逸条件
formula-depth-why-variable = 在步与步之间保留的变量
formula-depth-why-previous = 上一次的迭代值
formula-depth-why-comparison = 比较或 if 块
# floor, ceil, trunc and round are functions of the formula language: keep them.
formula-depth-why-rounding = 取整（floor、ceil、trunc、round）
# "maxit" is a word of the formula language: keep it.
formula-depth-why-maxit = maxit
formula-depth-why-power = 指数可变的幂
formula-depth-why-perturbed = 已经微扰过的程序

## Image and frame sizes: the presets described in words (the others are standard names: HD 720p, WUXGA, DCI 4K).

# $size: the pixel size (2048×2048).
size-preset-square = 正方形——{ $size }
# $ratio: the aspect ratio (9:16); $size: the pixel size.
size-preset-portrait = 竖向 { $ratio }——{ $size }
# $ratio: the aspect ratio (32:9); $size: the pixel size.
size-preset-super-ultrawide = 超级超宽屏 { $ratio }——{ $size }

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
fractal-family-custom = 自定义
# Conway's Game of Life.
fractal-family-life = 生命游戏
fractal-family-lsystem = L 系统
# A family at a power: "Multibrot 3", "Burning Ship 4". $family: a name above; $power: the power.
fractal-with-power = { $family } { $power }

## Names of choices: colouring methods, orbit traps, palette animation, snapshots, angle units.

color-method-smooth = 平滑迭代
color-method-stripe = 条纹平均
color-method-triangle = 三角不等式
color-method-trap = 轨道陷阱
color-method-distance = 距离估计
color-method-decomposition = 分解
# The shape an orbit trap catches the orbit with.
trap-point = 点
trap-cross = 十字
trap-circle = 圆
# Palette animation: none.
palette-anim-off = 关闭
palette-anim-forward = 正向
palette-anim-reverse = 反向
palette-anim-pingpong = 往返
# Palette animation: a new random gradient every so often.
palette-anim-random = 随机渐变
# What the Snapshot button does.
snapshot-mode-ask = 每次询问
snapshot-mode-screen = 屏幕截图
snapshot-mode-render = 完整渲染
# The unit of an angle typed into Go to location's polar fields.
angle-degrees = 度
angle-radians = 弧度
angle-turns = 圈

## Report an issue: the choices. (The report itself is sent in English.)

issue-kind-crash = 程序崩溃 / 卡死
issue-kind-rendering = 渲染错误
issue-kind-performance = 性能问题
issue-kind-ui = 界面 / 易用性问题
issue-kind-feature = 功能请求
issue-kind-other = 其他
severity-low = 低
severity-medium = 中
severity-high = 高
severity-blocking = 阻断
repro-always = 总是
repro-sometimes = 有时
repro-once = 只发生过一次
repro-cannot = 无法重现

## The benchmark's depths and the diagnostics' tests.

# $zoom: the dive's final magnification (1e32).
bench-depth-standard = 标准（{ $zoom }×）
# $zoom: the dive's final magnification (1e48).
bench-depth-ultra = 超深（{ $zoom }×）
diag-test-selftest = 自检
diag-test-uitest = 界面测试
diag-test-gputest = GPU 算术检查

## The status bar's readouts: each word is followed by a value in a slot of fixed width. Keep the words short (one word each).

status-center = 中心
status-cursor = 光标
status-zoom = 缩放
# Short for iterations.
status-iter = 迭代
# The period of the minibrot jumped to.
status-period = 周期
# A Game of Life view's scale (followed by "1 px = 4 cells" or "8 px / cell").
status-scale = 比例
# A Game of Life view's generation. Short.
status-generation = 代
# A Game of Life view's population. Short.
status-population = 细胞
# An L-system's order (how many times its rules were applied).
status-order = 阶数
# How many segments and shapes an L-system drew.
status-drawn = 绘制
# $value: how many cells one pixel covers (4, 9.99e300). "px" is the unit symbol: keep it.
life-scale-cells = 1 px = { $value } 个细胞
# $value: how many pixels one cell covers (8). "px" is the unit symbol: keep it.
life-scale-pixels = { $value } px / 细胞

## The status bar's limit diagnostic: a short label (after a warning sign), and what it means when hovered.

# The view is as deep as this location can be rendered. Short.
status-limit-depth = 深度极限
# Every pixel ran out of iterations. Short.
status-limit-exhausted = 迭代耗尽
# Pixels are capped at the iteration budget. Short.
status-limit-capped = 迭代封顶
# $pixels: how many pixels wide one step of the coordinates is; $sharp_to: the magnification the view is sharp to (written out, 1,234,567).
status-limit-direct-detail = 此视图在 GPU 上以单精度直接渲染，而在这里，坐标的一个步长有 { $pixels } 个像素宽：相邻像素计算的是同一个点，因此图像会碎成方块。在此位置，它的清晰度可达约 { $sharp_to }×。此公式还没有深度缩放（微扰）路径，因此继续放大不会显示更多细节。
# $samples: the GPU buffer's capacity in orbit samples; $iterations: the iterations pixels are limited to.
status-limit-depth-detail = 参考轨道达到了 GPU 缓冲区上限（{ $samples } 个样本）仍未逃逸，因此这里的像素被限制在约 { $iterations } 次迭代，无法在此视图中计算更精细的细节——无论是实时视图还是导出。这是此位置族目前的深度极限。
# $budget: the largest iteration budget this view can be given.
status-limit-exhausted-detail = 在此视图所能获得的最大预算（{ $budget }）下，每个像素仍然用尽了迭代次数。要么这里的视图位于集合内部（在深处很正常），要么它需要的迭代次数超出了程序目前所能提供的——两种情况下图像都一样，因此预算已恢复为低开销的值。
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations" is the setting's name in the Rendering panel.
status-limit-capped-max = { $percent }% 的像素在 { $budget } 的预算下用尽了迭代次数，而逃逸的像素恰好在这个上限处才完成——视图的解析不足。“迭代次数”已是最大值，因此此视图需要的超出了程序目前所能提供的。
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations" and "Rendering" are the names of a setting and a panel section.
status-limit-capped-raise = { $percent }% 的像素在 { $budget } 的预算下用尽了迭代次数，而逃逸的像素恰好在这个上限处才完成——视图的解析不足。请提高“迭代次数”（控制面板 ▸ 质量）来解决。
