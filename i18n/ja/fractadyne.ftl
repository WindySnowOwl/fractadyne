### Fractadyne user interface: Japanese (日本語).
### A draft: machine translation (Claude, by Anthropic) of the English file, 2026-10-23, not yet
### reviewed by a native speaker. Corrections are welcome as pull requests (see i18n/README.md).

## Language data: read by Fractadyne, not shown as messages.

# This language's own name, shown in the language menu.
language-name = 日本語
# The same name in English, shown beside it in the menu.
language-english-name = Japanese
# The writing system the app must be able to draw (an ISO 15924 code: Latn, Cyrl, Jpan, Hans, Khmr).
language-script = Jpan
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
language-more = その他の言語…
# The Languages window's title.
language-window-title = 言語
# The Languages window's introduction.
language-window-intro = Fractadyneのインターフェースを別の言語で表示します。1つの言語は1つのファイルで、リリースに同梱されているか、ここでインストールします。座標、ファイル、コマンドラインはどの言語でも英語のままです。
# Column headers of the installed-language list.
language-col-language = 言語
language-col-translated = 翻訳済み
language-col-from = 提供元
# Where a language comes from.
language-source-builtin = 組み込み
language-source-package = このリリース
language-source-user = インストール済み
language-source-dev = 作業フォルダー
# $percent: how much of the interface the file translates, 0–100.
language-coverage = { $percent }%
# $count: messages in the file that failed the checks and show in English instead.
language-dropped = 要確認のメッセージ{ $count }件
# Buttons of the Languages window.
language-install = 言語ファイルをインストール…
language-install-hover = Fractadyneの言語ファイル（.ftl）を選びます。ファイルはチェックされたあと、設定内の言語フォルダーにコピーされ、アップデート後もそこに残ります。
language-open-folder = 言語フォルダーを開く
language-reload = 再読み込み
language-reload-hover = 言語ファイルを読み込み直します（ファイルを編集または追加したあとに）。
language-use = 使用
# Notices after installing a file. $name: the language's own name; $file: the file's name.
language-installed = { $file }から{ $name }をインストールしました。
# $file: the file's name; $why: what was wrong with it.
language-install-failed = { $file }は使用できる言語ファイルではありません：{ $why }
# Shown when a language file cannot be loaded and English is used instead. $name: the language.
language-load-failed = { $name }を読み込めなかったため、Fractadyneは英語で表示されています：{ $why }
# A language whose writing system needs a font this computer does not have (Japanese or Chinese without a system font). $name: the language, in its own name.
language-font-missing = { $name }を表示できるフォントがこのコンピューターに見つからないため、表示できません。{ $name }のフォント（たとえばシステムの言語パック）をインストールしてから、もう一度選んでください。
# Shown after a draft translation's name in the language menu and the Languages window. Short.
language-draft = 下書き
language-draft-hover = 下書きです：この翻訳はまだネイティブスピーカーのレビューを受けていません。修正は、その言語のファイルへのプルリクエストとして歓迎します。
# The heading over the decimal-mark choice, in the language menu and the Languages window.
language-decimal-mark = 小数点記号
language-decimal-mark-hover = 数値の書き方です（時間、サイズ、個数）。座標と倍率は、ファイルやほかのプログラムが読み取れるよう、常にピリオドを使います。
# $sample: a number written with this language's own punctuation (1,234.5).
language-decimal-language = 言語の書き方に従う（{ $sample }）
# $sample: a number written with a decimal point (1,234.5).
language-decimal-point = ピリオド（{ $sample }）
# $sample: a number written with a decimal comma (1.234,5).
language-decimal-comma = カンマ（{ $sample }）

## Asked once when the system's language changes

# The window's title.
language-changed-title = 言語
# $new: the system's new language, in its own name.
language-changed-question = システムの言語が{ $new }になりました。Fractadyneでも使用しますか？
# Buttons. $name: a language's own name.
language-changed-use = { $name }を使用
language-changed-keep = { $name }のままにする

## The welcome window (first run)

welcome-title = Fractadyneへようこそ
welcome-intro = Mandelbrot集合とその仲間を深くズームして探索するソフトウェアです。まずは基本の操作をいくつか紹介します：
# The language row, at the top of the window.
welcome-language = 言語
# Controls: the key or gesture on the left, what it does on the right.
welcome-key-drag = ドラッグ
welcome-key-drag-does = ビューを移動
welcome-key-scroll = スクロール
welcome-key-scroll-does = カーソルに向かってズームイン/アウト
welcome-key-space = Spaceを長押し
welcome-key-space-does = なめらかな連続ズーム（Shift+Spaceでズームアウト）
# Shown after the click-zoom toolbar icon.
welcome-key-click = を選んでクリック
welcome-key-click-does = クリックした点へダイブ（ツールバー）
welcome-key-m-does = 最寄りのミニブロットへ、その大きさに合う倍率でジャンプ
welcome-landmarks = ランドマークにジャンプして始めましょう：
welcome-theme = テーマ
welcome-watermark = 「Fd」マークを表示
welcome-watermark-hover = ビューの右下にある小さなブランドマークです。オフにすると、ライブビューとエクスポートした画像から取り除かれます。
welcome-updates = アップデート
welcome-check-updates = アップデートを確認
welcome-check-updates-hover = アプリの起動時に、最新リリースをGitHubに問い合わせます。何もアップロードされません。
welcome-files = ファイル
welcome-no-config = （設定フォルダーなし）
welcome-open = 開く
welcome-rendering-on = レンダリングデバイス
welcome-open-help = ヘルプ全体を開く（F1）
# $icon: the Settings icon.
welcome-more-in-settings = その他の設定：ファイル → { $icon } 設定
welcome-get-started = はじめる

## Menus: shared

# A percentage in a menu (UI scale). $value: the number.
common-percent = { $value }%
# A state shown after a menu item's name, in brackets.
menu-state-connected = 接続中
menu-state-rendering = レンダリング中
menu-state-running = 実行中

## Navigate ▸ Bookmarks (also the toolbar's star)

menu-bookmarks = ブックマーク
menu-bookmarks-add = 現在のビューを追加
menu-bookmarks-manage = 管理…

## The fractal picker (Fractal menu and the toolbar's dropdown)

menu-family-powers = 高次のべき
menu-family-automata = オートマトン
menu-family-lsystems = Lシステム
menu-fractal-custom = カスタム数式…
menu-fractal-library = 数式ライブラリ…
menu-fractal-library-hover = 保存したカスタム数式：適用、編集、インポート、エクスポート
# The "Show" group: which picture of the formula is drawn. $name: the formula's name (Mandelbrot, Burning Ship…).
menu-show = 表示対象
menu-show-set = { $name }集合
menu-show-set-hover = パラメーター平面、つまり{ $name }集合そのものです。

    その各点がそれぞれ異なるJulia集合であり、これはそれらすべての地図です。
menu-show-julia = Julia集合
menu-show-julia-hover = { $name }集合の1点を、それ自体の絵として描いたものです。その点を固定したときに得られる形です。

    点は「両方（連動）」で地図にポインターを合わせるか、フラクタル ▸ Julia cで選びます。
menu-show-julia-none = { $name }には自由なパラメーターがないため、Julia集合はありません。
menu-show-both = 両方（連動）
menu-show-both-hover = 左に{ $name }集合を、右にカーソルが指している点のJulia集合を表示します。

    地図の上でカーソルを動かすと、Julia集合が変化します。2つの関係を知るいちばん手軽な方法です。
menu-show-both-none = { $name }には組み合わせるJulia集合がありません。

## File menu

menu-file = ファイル
menu-file-open-view = ビューまたは位置を開く…
menu-file-gallery = ギャラリー…
menu-file-export = 画像をエクスポート…
menu-file-snapshot = スナップショット
menu-file-open-life = ライフゲームのパターンを開く…
menu-file-open-life-hover = RLE、.cells、またはLife 1.05/1.06のファイルを、ライフゲームの盤面として表示します。
menu-file-life-text = ライフゲームのパターンテキスト…
menu-file-life-text-hover = パターンをテキストとして貼り付けます（LifeWikiが示すRLE形式）。
menu-file-save-life = ライフゲームのパターンを保存…
menu-file-save-life-hover = 現在の盤面を、RLEとして保存します。
menu-file-open-lsystem = Lシステムを開く…
menu-file-open-lsystem-hover = Fractintの.lファイル（含まれるエントリから選択）、または.lsysのシステム。
menu-file-save-lsystem = Lシステムを保存…
menu-file-save-lsystem-hover = システムのテキストを、.lsysファイルとして保存します。
menu-file-export-svg = SVGをエクスポート…
menu-file-export-svg-hover = ビューをSVG図面として保存します：線と図形をベクターとして、表示中の色で。
menu-file-render-client = レンダークライアント…
menu-file-render-client-hover = このマシンのGPUをレンダーファームに貸し出します：コントローラー（別のマシンのツール ▸ ファームでレンダリング…）に接続し、そのためにフレームをレンダリングします。
menu-file-settings = 設定
menu-file-reset = アプリケーションの状態をリセット…
menu-file-reset-hover = 保存されたすべての状態（セッション、ブックマーク、サムネイル）を削除して、最初からやり直します。実行前に確認を求めます。
menu-file-quit = 終了

## File ▸ Settings

menu-settings-fps = フレームレート上限
menu-settings-fps-uncapped = 上限なし
# $fps: frames per second.
menu-settings-fps-n = { $fps } FPS
menu-settings-ui-scale = UIの拡大率（文字サイズ）
menu-settings-theme = テーマ
menu-settings-updates = アップデート
menu-settings-updates-stable-hover = 最新の安定版リリース
menu-settings-updates-beta-hover = プレリリースを含む最新のビルド
menu-settings-updates-on-launch = 起動時にアップデートを確認
menu-settings-updates-on-launch-hover = オフの場合は、ヘルプ → 「アップデートを確認」から手動で確認します。
# Followed by its shortcut, (Ctrl+S).
menu-settings-snapshot = スナップショット
menu-settings-snapshot-ask-hover = 押すたびに選択肢を表示
menu-settings-snapshot-screen-hover = ビューを画面に表示されているとおりに保存（即時、PNG）
menu-settings-snapshot-render-hover = エクスポート設定でのフルレンダリングを、バックグラウンドで実行（深いビューでは長い時間がかかることがあります）
menu-settings-refcache = 参照軌道キャッシュ…
menu-settings-refcache-hover = ディープズームの参照軌道をディスクに保存しておき、再び訪れた位置を数秒でレンダリングできるようにします。保存場所、サイズ、上限、そして消去の方法。

## Fractal menu

menu-fractal = フラクタル

## View menu

menu-view = 表示
menu-view-zoom-home = 全体表示までズームアウト
menu-view-zoom-home-hover = ホームビュー全体までズームアウト（アニメーション）
menu-view-reset = 既定のビューにリセット
menu-view-reset-hover = フラクタルの既定のビューにリセット（即時）
# Followed by its shortcuts.
menu-view-undo = ビューを元に戻す
menu-view-redo = ビューをやり直す
menu-view-control-panel = コントロールパネル
menu-view-control-panel-hover = 右側のコントロールパネルの表示/非表示を切り替えます。
menu-view-minimap = ミニマップ
menu-view-minimap-hover = 「現在地」マーカーとズーム深度を添えた、ホームビューの小さな全体図を表示します。クリックすると、その領域へジャンプします。
menu-view-perf-panel = パフォーマンスパネル
menu-view-timestamp = タイムスタンプを表示
menu-view-timestamp-hover = ビューの上に、大きな経過時間の時計を表示します。ログが各行に付けるのと同じ「+12.345s」を表示するので、動いているときにしか現れない問題を画面録画し、ログとフレーム単位で照らし合わせられます。既定ではオフです。
menu-view-zoom-target = 自動ズームの目標を表示
menu-view-zoom-target-hover = 自動ズームの実行中、ズームしていく先の領域（あと数回拡大すると画面いっぱいになる部分）を枠で囲み、その四隅から画面の四隅へ線を引き、向かっている点に輪を表示します。既定ではオフです。
menu-view-live-panel = ライブレンダリングパネル
menu-view-live-panel-hover = ビューの上に重ねて、ライブレンダラーの動作を示すフロントパネルです：各フレームがディスパッチした内容、画面上の画像の古さ、反復帯を進むウォーク、参照軌道の構築、そしてその下で動くマシン。Bで切り替え、隅の×で閉じます。どの隅にもドラッグできます。計器にポインターを合わせると、その意味と、正常なときと問題があるときの見え方が表示されます。
# The live render panel's sizes.
menu-view-live-off = オフ
menu-view-live-off-hover = パネルなし。
menu-view-live-strip = ストリップ
menu-view-live-strip-hover = ニキシー管の表示とランプ列。
menu-view-live-panel-size = パネル
menu-view-live-panel-size-hover = 計器類：ウォークダイヤル、画像の古さ、タイル、GPUパス、深度ダイヤル、ランプ。
menu-view-live-full = フル
menu-view-live-full-hover = パネルと機関室：ステップ、GPU、メモリ、コア、ピクセル。
menu-view-live-opacity = 不透明度
menu-view-watermark = 「Fd」透かしを表示
menu-view-watermark-hover = ライブビューとエクスポートした画像の右下隅に、控えめな「Fd」ブランドマークを表示します。既定ではオンです。

## Color menu

menu-color = カラー
menu-color-method = 着色方法
menu-color-palette = パレット
menu-color-custom = カスタム
menu-color-duotone = デュオトーン
menu-color-binary = 二値（集合）
menu-color-binary-hover = グラデーションのない2色表示：集合の内側と外側。
menu-color-edit-gradient = グラデーションを編集…
menu-color-normalize = 深い色を正規化
menu-color-normalize-hover = 密集した模様が斑点のように見えてしまうとき、パレットをビューで測定した脱出値の範囲に割り当て直します。エイリアシングが起きていないビューはそのままです。それでも割り当て直すには「常に範囲に合わせる」をオンにします。着色方法が「スムーズ反復」のときのみ。
menu-color-more = サイクル、オフセット、アニメーション、エフェクト：コントロールパネル ▸ 着色

## Tools menu

menu-tools = ツール
# Followed by its shortcut, (M).
menu-tools-find-minibrot = ミニブロットを探してズーム
menu-tools-find-minibrot-hover = ニュートン法でビューの中心を近くのミニブロットの正確な中心に合わせ、その周期を表示し、ミニブロット自体の大きさまでズームします。1回で何桁も深くなることがよくあります。すでにそれより深い場合は、中心だけが移動します。
menu-tools-solver = Newton / Misiurewiczソルバー…
menu-tools-solver-hover = ビューに最も近い、ミニブロットの正確な中心（周期とサイズ）またはMisiurewicz点を求めます。完全な精度でのニュートン法と、反発的な乗数λを使います。ソルバーのある位置ダイアログが開きます。
# Followed by its shortcut, (A).
menu-tools-autopilot = 自動ズーム（オートパイロット）
menu-tools-autopilot-stop = オートパイロットを停止
menu-tools-autopilot-hover = 操作不要の連続ディープズーム：最も細部の豊かな領域に向かってダイブし、進みながら進路を修正します。ナビゲーション操作をすると停止します。
menu-tools-priority = 優先：
menu-tools-priority-speed = 速度
menu-tools-priority-speed-hover = 設定した速度でズームし、細部のある最初のフレームで画面を更新します。
menu-tools-priority-quality = 品質
menu-tools-priority-quality-hover = 完全に解像したフレームのみを表示します。ズームはそのフレームを待ちます。
menu-tools-target = 目標：
menu-tools-target-detail = 細部
menu-tools-target-detail-hover = ビュー内で最も豊かな境界。ダイブの進行に合わせて選び直します。
menu-tools-target-misiurewicz = Misiurewicz点
menu-tools-target-misiurewicz-hover = 最寄りのMisiurewicz点を1回だけ求め、そこへ正確にダイブします（Mandelbrotのみ）。
menu-tools-orbits = 軌道オーバーレイ
menu-tools-orbits-hover = カーソルの下の点の反復経路を描きます。アニメーションのオプション：コントロールパネル ▸ オーバーレイ。
menu-tools-orbits-normalize = 正規化（ビューに合わせる）
menu-tools-orbits-normalize-hover = 軌道をビュー全体に合わせ、どのズームでもきちんと収まるようにします（ビューポートを通して写すと、深いところでは画面外へ飛び出してしまいます）。
menu-tools-play-tour = ツアーを再生…
menu-tools-play-tour-hover = 再生するツアースクリプトを選びます。ツールバーの▶は、前回のツアーをワンクリックで再生します。
menu-tools-tour-from-view = 現在のビューからツアーを作成…
menu-tools-tour-from-view-hover = 全体表示から現在のビューまでズームするツアーを作成します。キャプションを付け、ダイブの長さを設定できます。編集、再生、レンダリングができるツアースクリプト（.toml）として保存されます。
menu-tools-render-tour = ツアーをレンダリング…
menu-tools-render-tour-hover = 読み込んだツアーを、別のプロセスでPNGの連番フレーム（必要ならmp4も）にレンダリングします。
menu-tools-close-player = ツアープレーヤーを閉じる
menu-tools-close-player-hover = ツアーを停止してプレーヤーを閉じ、自分の反復と着色の設定に戻します
menu-tools-render-farm = ファームでレンダリング…
menu-tools-render-farm-hover = 読み込んだツアーを複数のマシンでレンダリングします：このマシンがジョブを制御し、ほかのマシンはファイル ▸ レンダークライアント…で参加します。
menu-tools-benchmark = ベンチマーク…
menu-tools-benchmark-hover = レンダリング速度を測定します。現在の設定で測るか、マシン間で比較できる標準化された実行（固定の解像度と設定）で測ります。バーンインはこれを繰り返し、安定性やサーマルスロットリングを確認します。
menu-tools-last-benchmark = 前回のベンチマークを表示

## Navigate menu

menu-navigate = ナビゲート
menu-navigate-goto = 位置へ移動…
menu-navigate-goto-hover = 中心とズームを入力するか、よく知られた点へジャンプするか、ビューの近くのMisiurewicz点やミニブロットをニュートン法で求めます。
menu-navigate-misiurewicz = Misiurewiczエクスプローラー…
menu-navigate-misiurewicz-hover = 小さなMisiurewicz型がどのように見えるかを閲覧できます。各点のズーム周期とねじれを添えたサムネイルが表示され、どれでもニュートン法で任意の深さまで求められます。（Mandelbrot）
menu-navigate-exact = 厳密な点（式）
menu-navigate-exact-note = 主カージオイドに付いたバルブの根元を、厳密な式（2πの分数のcos/sin）で示します。座標は式のまま保持されるので、ズームインしても正確にその点の上にとどまります。
menu-navigate-random = ランダムな位置
menu-navigate-random-hover = 細部の豊かな境界上のランダムな点へジャンプ
menu-navigate-import-kfr = .kfrをインポート…
menu-navigate-import-kfr-hover = Kalles Fraktalerの位置ファイルを読み込みます
menu-navigate-share = 位置を共有…
menu-navigate-share-hover = 自己完結した位置（.fdn）をコピー、貼り付け、保存、読み込みします：フラクタル、完全精度の中心、ズーム、着色。

## Help menu

menu-help = ヘルプ
# Followed by its shortcut, (F1).
menu-help-reference = ヘルプとリファレンス…
menu-help-welcome = ようこそ / クイックスタート…
menu-help-welcome-hover = 初回起動時の概要：基本の操作といくつかのランドマーク。
menu-help-diagnostics = 診断…
menu-help-diagnostics-hover = Fractadyneがお使いのハードウェアで正しく動作するかを確認し、その結果を問題の報告に添付します
menu-help-report = 問題を報告…
menu-help-report-hover = クラッシュレポート、ログ、位置を添付して、GitHubのissue（またはメール）を送ります
menu-help-accelerated = ディープズームの高速化…
menu-help-accelerated-hover = ディープズームの参照軌道を2.5～6.4倍速く計算する、オプションのビルドです。画像は同一で、設定はそのまま引き継がれます。
menu-help-check-updates = アップデートを確認
# $track: the update track (stable or beta).
# $track: "stable" or "beta", the update track in use (the variants hold the wording).
menu-help-check-updates-hover =
    { $track ->
        [beta] 新しいベータ版（プレリリース）のビルドがないかGitHubで確認
       *[other] 新しい安定版のビルドがないかGitHubで確認
    }
# $version: the new version; $channel: "beta" for a pre-release, "stable" otherwise (the variants hold the wording).
menu-help-update-available =
    { $channel ->
        [beta] { $version }（ベータ版）が利用可能：ダウンロード
       *[other] { $version }（安定版）が利用可能：ダウンロード
    }
menu-help-up-to-date = 最新です
menu-help-tagline = ネイティブのフラクタル探索ソフトウェア
menu-help-license = ライセンス
# $year and $holder: the copyright year and holder, as the code gives them.
menu-help-copyright = © { $year } { $holder }。デュアルライセンスです。どちらかのライセンスを選んで使用できます。
menu-help-source = ソースコード

## The toolbar (its buttons are icons; these are their tooltips)

menu-toolbar-julia = Julia
menu-toolbar-julia-hover = この数式のJulia集合をウィンドウいっぱいに表示
menu-toolbar-dual-hover = 両方（連動）：左に集合、右にカーソルの点のJulia集合
menu-toolbar-open-hover = ビューまたは位置を開く（PNG / EXR、.fdn、.kfr）
menu-toolbar-gallery-hover = ギャラリー
menu-toolbar-export-hover = 画像をエクスポート…
menu-toolbar-share-hover = 位置を共有：自己完結した.fdn（フラクタル、完全精度の中心、ズーム、着色）をコピー、貼り付け、保存、読み込み
menu-toolbar-zoom-in-hover = ズームイン
menu-toolbar-zoom-out-hover = ズームアウト
# $factor: the click-zoom factor (4, 10…).
menu-toolbar-click-zoom-hover = クリックでズーム（{ $factor }×）：左クリックでその点へダイブ、右クリックで戻ります（ドラッグは引き続きパン）。倍率はコントロール ▸ ナビゲートで変更します。
# Added to the click-to-zoom tooltip in the dual view.
menu-toolbar-click-zoom-dual = これがオンの間は、Ctrl+クリックでJulia cをピン留めします。
menu-toolbar-autopilot-running-hover = 自動ズームの実行中：クリックで停止
menu-toolbar-autopilot-hover = 自動ズーム：細部に向かってダイブ（A）
menu-toolbar-reset-hover = ビューをリセット（即時）
menu-toolbar-home-hover = ホームビューまでズームアウト（アニメーション）
menu-toolbar-bookmarks-hover = ブックマーク：このビューを保存、または保存したビューへジャンプ
# $name: the palette's name.
menu-toolbar-next-palette-hover = 次のパレット（{ $name }）
# $file: the last tour's file name.
menu-toolbar-play-last-hover = ツアーを再生：{ $file }（クリック）· 前回のツアーを再生します。別のツアーはツール ▸ ツアーを再生…で選びます
menu-toolbar-play-hover = ツアーを再生…（ツール ▸ ツアーを再生）
menu-toolbar-speed-hover = 再生速度
menu-toolbar-fullscreen-hover = 全画面表示の切り替え

## Dialogs: words shared by many windows

# A button that closes a window without doing what it offers.
dlg-cancel = キャンセル
# A button that dismisses a window (nothing in it is pending).
dlg-close = 閉じる
# A button that copies the window's text to the clipboard (after a copy icon).
dlg-copy = コピー
# A button that jumps to a location.
dlg-go = 移動
# A button that fills the fields from the view on screen.
dlg-use-current = 現在の値を使用
# A notice after a file could not be written. $error: the system's reason, as it gives it.
dlg-save-failed = 保存できませんでした：{ $error }
# A notice when no graphics device is available to render with.
dlg-gpu-not-available = GPUを使用できません
# The name of the plain-text file type in a save dialog (the extension .txt is added by the code).
dlg-filter-text = テキスト

## Asked after a session that did not shut down cleanly

dlg-crash-title = Fractadyneが正常に終了しませんでした
dlg-crash-intro = 前回のセッションが予期せず終了しました。このマシンにレポートが保存されています。送信しますか？
dlg-crash-review = 内容を確認するまでは何も送信されません：次の画面に含まれる内容がそのまま表示され、システム情報はチェックボックス1つで除外できます。
dlg-crash-report = 報告する…
dlg-crash-report-hover = クラッシュレポートとクラッシュした位置を添付して、「問題を報告」を開きます。
dlg-crash-not-now = 今はしない
dlg-crash-dont-ask = 今後はクラッシュ後に確認しない
dlg-crash-dont-ask-hover = レポートは引き続きローカルに保存され、ヘルプ ▸ 問題を報告はいつでも使えます。

## Asked after a session that ended with a second graphics device in use

dlg-second-device-title = 2つ目のグラフィックスデバイスをオフにしますか？
# $device: the graphics device's name and details, as the system gives them.
dlg-second-device-intro = 2つ目のグラフィックスデバイス（{ $device }）の使用中に、前回のセッションが予期せず終了しました。これが原因の可能性があります。
dlg-second-device-note = オフにすると、以前と同じように、ウィンドウを描画しているカードがすべてをレンダリングします。詳細設定 ▸ 2枚目のグラフィックスカードで、再びオンにできます。
dlg-second-device-off = オフにする
dlg-second-device-keep = オンのままにする

## Shown when the previous session's iteration setting was set aside

dlg-quarantine-title = 反復回数の設定を保留しました
dlg-quarantine-change = コントロール ▸ 品質で、いつでも変更できます。
dlg-quarantine-keep = 既定値のままにする
# A button. $setting: the iteration setting the previous session used (such as "20,000 iterations").
dlg-quarantine-restore = { $setting }に戻す

## Navigate ▸ Go to location

dlg-goto-title = 位置へ移動
# Before two choices: "x, y" (cartesian) and the polar form below.
dlg-goto-enter-as = 入力形式：
# The polar way of entering a point: an offset plus a radius r at an angle θ. Keep the symbols.
dlg-goto-polar = オフセット + r ∠ θ
# The tooltip of the "?" button beside the entry choice.
dlg-goto-help-hover = 入力欄で使えるもの：分数、べき乗、sqrt/sin/cos…、piなどの定数。ヘルプの「座標の式」を開きます
# Labels over the entry fields. x0, y0, r, θ, Re and Im are the fields' symbols: keep them.
dlg-goto-polar-offset = オフセット中心（x0, y0：純粋な極座標の点なら0のまま）
dlg-goto-polar-radius = 半径 r、角度 θ
dlg-goto-re = 中心：実部（Re）
dlg-goto-im = 中心：虚部（Im）
dlg-goto-zoom = ズーム（倍率）
# A warning under the zoom field, after a ⚠ sign. $have: the iteration count set; $typical: the count this depth typically needs.
dlg-goto-iter-warning = 反復回数は{ $have }に固定されています。この深度では通常、約{ $typical }が必要です。反復回数を上げるまで、ビューは平坦にレンダリングされます。または、反復回数の自動調整（コントロール ▸ 品質）を有効にすれば、自動的に合わせてくれます。
dlg-goto-copy-hover = この位置をクリップボードにコピー
dlg-goto-cancel-hover = どこにも移動せずに閉じる
# The expressions in brackets are examples of what can be typed: keep them as they are.
dlg-goto-paste-note = ほかの人の中心とズームを貼り付けるか、コピーして共有します。座標には式を使えます：厳密な分数（-3/4）、関数と定数（-0.5 + 0.25*cos(pi/4)、sqrt、root(x,n)、^。角度はラジアン）。また実部の欄には複素数の値全体（(37+16i)/100）を入力でき、両方の欄が埋まります。
# A section that opens and closes.
dlg-goto-feature = 特徴点へ移動（Mandelbrot）
dlg-goto-feature-poi = よく知られた点：
# The dropdown's text before a point is picked.
dlg-goto-feature-poi-pick = 注目点へジャンプ…
dlg-goto-feature-solve = または、現在のビューの近くの特徴点を求める：
# Two choices: what to solve for. Misiurewicz is a mathematician's name (a Misiurewicz point).
dlg-goto-feature-misiurewicz = Misiurewicz
dlg-goto-feature-minibrot = 最寄りのミニブロット
# Labels of two small number fields; k and p are their symbols.
dlg-goto-feature-preperiod = 前周期 k
dlg-goto-feature-period = 周期 p
# The faint text in an empty field: left empty, the value is found automatically.
dlg-goto-feature-auto-hint = 自動
# A small button that empties both fields, so they are found automatically again.
dlg-goto-feature-auto = 自動
dlg-goto-feature-auto-hover = 両方の欄を空にして、次の探索でビュー中心の軌道から前周期と周期を検出するようにします。
dlg-goto-feature-kp-hover = 両方を空欄にすると、ビュー中心の軌道から検出され、見つかった値がここに入力されます。螺旋を見てその前周期がわかる人はいません。それを指定しなければならなかったことが、このソルバーを実際には使えないものにしていました。
# While a solve runs, followed by its elapsed time and the note below.
dlg-goto-feature-solving = 計算中…
dlg-goto-feature-solving-note = （任意精度。深いビューほど時間がかかります）
dlg-goto-feature-cancel-hover = 待つのをやめます。計算自体はバックグラウンドで最後まで実行され、その答えは破棄されます。中途半端な状態は何も残りません。
dlg-goto-feature-find = ビューの近くを探す
dlg-goto-feature-find-hover =
    現在の中心からニュートン法で正確な特徴点を求めます（任意精度）。先に特徴点の近くまで移動してください。

    計算する前に、上のズーム欄を実際に行きたい深度に設定してください：点はその深度に必要な精度で計算され、ビューはそこへ直接ジャンプします。手作業で何度もズームする代わりに、1回の計算で済みます。
dlg-goto-solve-cancelled = 計算をキャンセルしました。

## Snapshot (the camera button): its tooltip, and the choice on the first press

# The camera button's tooltip, one per Snapshot setting. Followed by its shortcut, (Ctrl+S).
dlg-snapshot-hover-ask = スナップショット：表示どおりのビュー、またはフルレンダリングを保存。最初に押したときに確認します
dlg-snapshot-hover-screen = スナップショット：ビューを表示どおりに、前回のエクスポート先フォルダーへ保存
dlg-snapshot-hover-render = スナップショット：エクスポート設定でフルレンダリングし、前回のエクスポート先フォルダーへ保存
# The window's title.
dlg-snapshot-title = スナップショット
dlg-snapshot-question = スナップショット（カメラボタン、Ctrl+S）で何を保存しますか？
# The first choice, and a short tag beside it.
dlg-snapshot-screen = 画面キャプチャ
dlg-snapshot-screen-tag = 即時
dlg-snapshot-screen-note = ビューを画面に表示されているとおりに、画面の解像度で、前回のエクスポート先フォルダーへPNGとして保存します。見たままが保存されます。
# The second choice, and a short tag beside it.
dlg-snapshot-render = フルレンダリング
dlg-snapshot-render-tag = 時間がかかることがあります
# $size: the image size, as 1920×1080; $ss: the supersampling factor (1–4); $format: PNG or EXR.
dlg-snapshot-render-note = エクスポート設定でレンダリングします：{ $size }、{ $ss }×スーパーサンプリング、{ $format }。最高品質ですが、深いビューでは数分から数時間かかることがあります。バックグラウンドで実行され、ファイル → 画像をエクスポート…で進行状況の確認やキャンセルができます。
dlg-snapshot-remember = 選択を記憶して、今後は確認しない
dlg-snapshot-change = ファイル → 設定 → スナップショットで、いつでも変更できます。

## Share location (.fdn)

dlg-share-title = 位置を共有
dlg-share-intro = 自己完結した位置（フラクタル、完全精度の中心、ズーム、着色）です。コピーして共有するか、貼り付けまたは読み込んで「適用」を押します。
dlg-share-thumb = 保存するファイルにサムネイルを含める
dlg-share-thumb-hover = .fdnの保存時にこのビューの小さな画像を埋め込み、ギャラリーでファイルを見分けられるようにします。約55 KB増えます。上のテキストには含まれません。
dlg-share-copy-hover = テキストをクリップボードにコピー
# Buttons (the first after a save icon).
dlg-share-save = .fdnを保存…
dlg-share-load = .fdnを読み込む…
dlg-share-apply = 適用
dlg-share-apply-hover = ボックス内の位置へジャンプ
dlg-share-copied = クリップボードにコピーしました。

## Help ▸ Report an issue (the report itself stays in English: it is read by the maintainer)

dlg-report-title = 問題を報告
dlg-report-intro = 報告にいちばん適しているのはGitHubのissueです。公開されていて検索でき、修正されたときにわかります。メールでもかまいません。まず下の内容をすべて確認してください：操作するまで、何もこのマシンから送信されません。
# Labels before three dropdowns.
dlg-report-type = 種類：
dlg-report-severity = 重大度：
dlg-report-repro = 再現性：
dlg-report-describe = 何が起きたかを説明してください：
# The faint text in the empty description box.
dlg-report-describe-hint = 何が起きたか、再現する手順、そして本来はどうなるはずだったか。
# Before the checkboxes of what the report carries.
dlg-report-include = 含めるもの：
dlg-report-sysinfo = システム情報（バージョン、OS、CPU、GPU、VRAM）
dlg-report-location = 現在の位置（.fdn）
dlg-report-log = 最近のログ
dlg-report-frames = フレームの記録（直近1分間と、直近40フレーム）
dlg-report-crash = 最新のクラッシュレポート
dlg-report-crash-none = 最新のクラッシュレポート（見つかりません）
dlg-report-crash-view = クラッシュした位置（.fdn）
dlg-report-crash-view-none = クラッシュした位置（.fdn）（見つかりません）
dlg-report-crash-view-hover = クラッシュやグラフィックスデバイスのロストが起きたときの正確なビューです。メンテナーが問題を再現するための唯一の手がかりになります。自動的に記録されます。
dlg-report-test = 診断テストの結果
dlg-report-test-none = 診断テストの結果（ヘルプ → 診断…で実行してください）
dlg-report-test-hover = お使いのハードウェアで機械的に検証された合否の結果です。メンテナーにとっては、説明だけよりもはるかに役立ちます
# A section that opens and closes. $size: the report's size, as "12 KB".
dlg-report-preview = 送信される内容をプレビュー（{ $size }）
dlg-report-github = GitHubのissueを開く
dlg-report-github-hover = タイトルを入力済みの新しいissueのページを開きます。レポートはクリップボードにコピーされるので、issueの本文に貼り付けてください（Ctrl+V）。
# Buttons (after a copy and a save icon).
dlg-report-copy = レポートをコピー
dlg-report-copy-hover = すべてをクリップボードにコピー
dlg-report-save = レポートを保存…
dlg-report-save-hover = 添付用に.txtとして保存
dlg-report-by-email = またはメールで：
dlg-report-gmail = Gmailで作成
# $address: the email address reports go to.
dlg-report-gmail-hover = 件名を入力済みのGmail作成ウィンドウを、{ $address }宛てに開きます。コピーしたレポートを貼り付けてください
dlg-report-email = メールアプリ…
dlg-report-email-hover = 既定のメールアプリを開きます（mailto）。メールのハンドラーが設定されている必要があります
dlg-report-tip = ヒント：どのボタンでもレポートはクリップボードにコピーされます。issueやメッセージの本文に貼り付けてください（Ctrl+V）。できれば、スクリーンショットやサンプル画像も添付してください（ドラッグして追加）。レンダリングやUIの問題では特に役立ちます。
# Notices in the window after a button is pressed.
dlg-report-opened-github = GitHubを開きました。レポートはクリップボードにあるので、issueに貼り付けてください。
dlg-report-copied = レポートをクリップボードにコピーしました。
# $path: where the report was saved.
dlg-report-saved = { $path }に保存しました
dlg-report-opened-email = メールアプリを開きました。レポートはクリップボードにあるので、貼り付けてください。
dlg-report-opened-gmail = Gmailの作成画面を開きました。コピーしたレポートを本文に貼り付けてください（Ctrl+V）。

## A location whose checksum does not match

dlg-checksum-title = この位置は破損している可能性があります
# After a ⚠ sign.
dlg-checksum-mismatch = チェックサムが、一緒にあるデータと一致しません。
dlg-checksum-why = 通常は、ここに届くまでにテキストが途中で切れたか、書き換えられたことを意味します。チャットウィンドウやフォーラムの投稿から一部だけをコピーしたことがよくある原因です。
# A label over where the location came from (a file path, or the clipboard).
dlg-checksum-source = 読み込み元
# The two checksums, side by side: the one written in the file, and the one computed from its data.
dlg-checksum-in-file = ファイル内の値
dlg-checksum-from-data = データからの計算値
dlg-checksum-load-note = それでも読み込むことはできます。座標は問題ない可能性があり、何も上書きされません。読み取れない部分は、読み込み後に報告されます。
dlg-checksum-load = それでも読み込む
dlg-checksum-load-hover = 不一致を無視してジャンプします
# Notices after loading it. $zoom: the magnification, as 1.2e148.
dlg-checksum-loaded = チェックサムは一致しませんが、{ $zoom }×の位置を読み込みました
# $zoom: the magnification, as 1.2e148; $note: what the loader found wrong with the location.
dlg-checksum-loaded-note = { $zoom }×の位置を読み込みました：{ $note }
dlg-checksum-not-loaded = 読み込みませんでした。

## File ▸ Reset application state

dlg-reset-title = アプリケーションの状態をリセット
# After a ⚠ sign, followed by a list.
dlg-reset-warning = 保存されているFractadyneのデータがすべて完全に削除されます：
# The list's items, each after a bullet.
dlg-reset-session = 保存されたセッション（現在のビュー、着色、環境設定）
dlg-reset-bookmarks = すべてのブックマークとそのサムネイル
dlg-reset-refcache = 参照軌道キャッシュ（位置を再び訪れると再構築されます）
# A label over the folder that is deleted.
dlg-reset-location = 場所
dlg-reset-note = この操作は元に戻せません。現在のセッションは終了時に保存されないため、次回の起動時には既定値が読み込まれます。
dlg-reset-confirm = すべてリセット
dlg-reset-done = アプリケーションの状態をリセットしました。次回の起動時に既定値が読み込まれます。
# $error: the system's reason, as it gives it.
dlg-reset-failed = リセットできませんでした：{ $error }

## Tools ▸ Tour from current view

dlg-tour-title = 現在のビューからツアーを作成
dlg-tour-intro = 全体表示から現在の位置までズームするツアーを作成します。
# A label before the fractal and magnification the tour ends at.
dlg-tour-destination = 目的地
dlg-tour-notation = 注記（ダイブ中に表示するキャプション、省略可）
# The faint example text in the empty caption box.
dlg-tour-notation-hint = 例：Misiurewiczの三本腕へダイブ…
dlg-tour-duration = ズーム時間
# $rate: how fast the dive zooms, in powers of ten a second (as 2.5).
dlg-tour-rate = ≈ { $rate } 桁/秒
# A small button that puts the duration back to its default.
dlg-tour-reset = リセット
dlg-tour-reset-hover = 深度に応じた既定値
dlg-tour-duration-note = 既定値は、ダイブが見やすい速さになるよう深度に応じて変わります。速いダイブでは、極端なズームで単色のフレームが一瞬表示されることがあります。
# A button, after a save icon.
dlg-tour-save = ツアーを保存…
# The file type's name in the save dialog.
dlg-tour-filter = Fractadyneスクリプト（TOML）
# A notice. $file: the saved file's name. --render-tour is a command-line option: keep it.
dlg-tour-saved = ツアーを保存しました → { $file }。ツール → ツアーを再生で再生するか、--render-tourでレンダリングします。

## Navigate ▸ Bookmarks ▸ Manage…

dlg-bookmarks-title = ブックマーク
# The faint text in the empty name box.
dlg-bookmarks-name-hint = 名前（省略可）
# A button, after a bookmark icon.
dlg-bookmarks-add = 現在のビューを追加
dlg-bookmarks-empty = ブックマークはまだありません。現在のビューから追加してください。
# The tooltip of a bookmark's delete (bin) button.
dlg-bookmarks-delete-hover = 削除

## Tools ▸ Benchmark…

dlg-bench-title = ベンチマーク
dlg-bench-mode = モード
dlg-bench-current = 現在の設定
dlg-bench-current-hover = 現在の解像度と設定で、固定のディープズームツアーをライブビューで再生します。
dlg-bench-standard = 標準化
dlg-bench-standard-hover = 解像度とすべてのレンダリング設定を固定し、スコアをマシン間で比較できるようにします。
dlg-bench-resolution = 解像度
dlg-bench-depth = 深度
# Standard and Ultra are the two depths in the dropdown this describes.
dlg-bench-depth-hover = どちらのダイブも、深いfloatexp/BLAの処理経路に入ります。超深度（1e48×）は標準（1e32×）よりそこではるかに長い時間を費やし、その分、実行にも長くかかります。
dlg-bench-burnin = バーンイン（繰り返し）
dlg-bench-burnin-hover = ベンチマークを繰り返し実行し、安定性とサーマルスロットリングを明らかにします。
# The label of a slider: how many times the benchmark runs.
dlg-bench-passes = 回数
# $size: the image size, as 1920×1080; $ss: the supersampling factor; $depth: the dive's final magnification, as 1e32.
dlg-bench-std-summary = オフスクリーンで{ $size }、{ $ss }× SS、Mandelbrot/スムーズでレンダリングし、{ $depth }×まで60フレームでダイブします。
dlg-bench-run = 実行

## The standardized benchmark's progress window

dlg-bench-progress-title = ベンチマークを実行中…
# $resolution: the benchmark's resolution, as the dropdown names it.
dlg-bench-progress-mode = 標準化 · { $resolution }
# $pass: the pass running now; $total: how many passes there are.
dlg-bench-progress-pass = バーンイン：{ $pass }/{ $total }回目
dlg-bench-progress-dive = 固定のダイブをレンダリング中…
# On the progress bar. $done: frames rendered; $total: frames in the dive.
dlg-bench-progress-frame = フレーム { $done }/{ $total }
# $fps: frames per second, as 41.3.
dlg-bench-progress-last = 前回：{ $fps } fps

## The benchmark's results

dlg-bench-results-title = ベンチマークの結果
# Buttons, each after an icon.
dlg-bench-results-run-again = もう一度実行…
dlg-bench-results-save = 保存…
dlg-bench-results-none = ベンチマークはまだ実行されていません。
dlg-bench-results-saved = ベンチマークを保存しました。

## The accelerated build's notice when its arithmetic libraries are missing

dlg-backend-title = 高速版ビルド：組み込みの演算を使用中
dlg-backend-dont-show = 今後表示しない

## File ▸ Gallery…

dlg-gallery-title = ギャラリー
# A button that picks the folder to show.
dlg-gallery-folder = フォルダー…
# A button, after a refresh icon.
dlg-gallery-refresh = 更新
dlg-gallery-empty = このフォルダーにはFractadyneの画像がありません。
# An image's title when it does not name its fractal.
dlg-gallery-untitled = Fractadyneの画像
# $zoom: the image's magnification, as 1.2e148; $saved: when it was saved, as the file gives it.
dlg-gallery-zoom = ズーム { $zoom }   { $saved }
# The notes saved with the image, in quotation marks. Use your language's quotation marks.
dlg-gallery-notes = 「{ $notes }」
dlg-gallery-open = このビューを開く
dlg-gallery-loaded = ギャラリーからビューを読み込みました。

## File ▸ Export image…

dlg-export-title = 画像をエクスポート
# A dropdown of image sizes.
dlg-export-size = サイズ
# The size dropdown's text for a size set by hand. $size: the size, as 1920×1080.
dlg-export-size-custom-current = カスタム：{ $size }
# The size dropdown's first choice.
dlg-export-size-custom = カスタム…
dlg-export-size-custom-hover = 幅を自分で設定します。高さは下の縦横比に従います。
dlg-export-width = 幅（px）
# After the width field. $height: the height in pixels, which follows from the aspect.
dlg-export-height-from-aspect = × { $height }（縦横比から）
dlg-export-ss = スーパーサンプリング
dlg-export-aspect = 縦横比
# The aspect choice that keeps the window's own shape.
dlg-export-aspect-window = ウィンドウに合わせる
# Before the choices PNG and OpenEXR.
dlg-export-format = 形式：
dlg-export-hud = 位置HUD
dlg-export-hud-hover = エクスポートする画像の左上に、ズームレベルと座標のパネルを焼き込みます（出力に合わせて拡大縮小され、地図/Mandelbrotビューのズームと中心を表示します）。
dlg-export-glitch = グリッチ補正
dlg-export-glitch-hover = 複数の参照軌道による摂動グリッチの補正です。非常に深い（floatexp）単一ビューのエクスポートでは自動的に省略され、参照軌道の構築とレンダリングが別スレッドで実行されて、アプリは応答し続けます。MandelbrotやMultibrotのビュー（Juliaモード以外）では使われません：厳密な計算と照合したところ、そこでは何も修正せず、エクスポート時間が最大5倍かかったためです。
dlg-export-deep-note = 深いエクスポート：参照軌道は別スレッドで構築されます（UIは操作可能なまま）。この深度ではグリッチ補正は省略されます。
# A dropdown, shown in the dual (map + Julia) view, and its three choices.
dlg-export-dual = デュアルのレイアウト
dlg-export-dual-side = 横に並べる
dlg-export-dual-separate = 別々のファイル
dlg-export-dual-map = 地図のみ
dlg-export-notes = メモ：
# The faint text in the empty notes box.
dlg-export-notes-hint = 画像と一緒に保存されます（120文字以内）
# $width and $height: the image size in pixels; $left: how many characters the notes can still take.
dlg-export-output = 出力：{ $width } × { $height } px（残り{ $left }文字）
# When the export's shape differs from the window's. $percent: how much more, as 25.
dlg-export-extra-width = ウィンドウより横に{ $percent }%広く表示されます（何も切り取られません）
# The same, when the export is taller than the window. $percent: how much more, as 25.
dlg-export-extra-height = ウィンドウより縦に{ $percent }%広く表示されます（何も切り取られません）
dlg-export-tiles-note = バックグラウンドのスレッドでタイルに分けてレンダリングします（サイズの上限なし）。ファイルにはビューとメモが埋め込まれるので、ファイル ▸ ビューまたは位置を開くで再び開けます。
# A label before the button that picks the folder exports go to.
dlg-export-folder = フォルダー：
dlg-export-choose = 選択…
dlg-export-choose-hover = エクスポート先のフォルダーを選びます
dlg-export-open-after = 完了したら開く
dlg-export-open-after-hover = 画像が書き込まれたらすぐに、システムのビューアーで開きます。
# $count: the graphics cards in this machine (2 or more).
dlg-export-all-gpus = すべてのグラフィックスカードを使用（{ $count }）
dlg-export-all-gpus-hover = 画像をタイル単位で、このマシンのすべてのグラフィックスカードに分割します。各カードは深い参照軌道のコピーをそれぞれ持ちます。同じカードどうしなら、1枚でレンダリングした場合とまったく同じ画像になります。異なるモデルでは、タイルごとにいくつかのピクセルがわずかに異なって描かれます（測定結果：目に見える継ぎ目はありません）。カードが失敗した場合、そのタイルはほかのカードに回されます。分割できない画像（一部の着色方法と数式）は、このウィンドウのカードを使います。
# The export's progress.
dlg-export-preparing = 準備中（参照軌道を構築中）…
dlg-export-saving = 保存中…
dlg-export-rendering = レンダリング中…
# $time: the time since the export started, as 8.3s or 1m 04.0s.
dlg-export-elapsed = 経過時間：{ $time }
dlg-export-cancel-hover = エクスポートせずに閉じる
# A button, after a save icon.
dlg-export-save-as = 名前を付けて保存…
dlg-export-save-as-hover = ファイル名と保存場所を選びます
dlg-export-export = エクスポート
dlg-export-export-hover = レンダリングして上のフォルダーに保存します（名前は自動）

## Notices worded at startup (main.rs)

# A notice shown once after starting, when the saved session comes from a newer version of the app. $saved and $handled: version numbers of the session file's format (1, 2…).
app-state-newer = このセッションは、より新しいFractadyneで保存されました（状態 v{ $saved }。このビルドが扱えるのは v{ $handled }）。一部の設定は適用されない可能性があり、保存するとこのビルドの形式で書き直されます。
# The title of the notice that says the accelerated build's arithmetic libraries are found again (its text comes from elsewhere).
app-notice-mpfr-restored-title = 高速演算が復帰しました
# After Fractadyne restarted itself because the graphics driver reset the GPU.
app-toast-gpu-recovered = グラフィックスデバイスのリセットから復旧しました。前の位置はレンダリングの負荷が高すぎたため、ホームビューで開き直しました。

## Short notices (toasts)

# When the autopilot reached a view that could not be rendered and put the last good view back.
app-toast-autopilot-view-restored = オートパイロットを停止しました：ビューが使用できない状態になったため、元に戻しました（ログに記録済み）
# $error: the system's reason, as it gives it (often in English).
app-toast-gradients-save-failed = グラデーションを保存できませんでした：{ $error }
# $error: the reason, as the program gives it (often in English).
app-toast-gradients-serialize-failed = グラデーションをシリアライズできませんでした：{ $error }
# $name: the name the user gave the gradient.
app-toast-gradient-saved = グラデーション「{ $name }」を保存しました。
# $error: the reason, as the program gives it (often in English).
app-toast-bookmarks-serialize-failed = ブックマークをシリアライズできませんでした：{ $error }
# $error: the system's reason, as it gives it (often in English).
app-toast-bookmarks-save-failed = ブックマークを保存できませんでした：{ $error }
# Jumping to a named point given by exact expressions. $name: the point's name (in English).
app-toast-coordinates-failed = { $name }：座標を評価できませんでした
# $zoom: the magnification, already formatted (1,234.5 or 1.2e148).
app-toast-random-location = ランダムな位置 · { $zoom }×
# Importing a Kalles Fraktaler .kfr location. $error: the reason (in English).
app-toast-import-failed = インポートできませんでした：{ $error }
# $zoom: the magnification, already formatted.
app-toast-location-loaded = 位置を読み込みました（{ $zoom }×）

## Go to location: why a jump could not be made (one line per field that failed)

# "Re" is the dialog's real-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-re = Re：{ $error }
# "Im" is the dialog's imaginary-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-im = Im：{ $error }
# The imaginary field was given a complex value. $error: the evaluator's reason (in English).
app-goto-error-im-real = Im：{ $error }。虚部は i を付けずに実数で入力してください
# "Zoom" is the dialog's magnification field.
app-goto-error-zoom-empty = ズーム：倍率を入力してください（例：1e50、250000）
# $zoom: what was typed, in quotes.
app-goto-error-zoom = ズーム：{ $zoom }を読み取れません。1e50や250000のような正の倍率を入力してください
app-goto-error-invalid = 入力が無効です。座標とズームを確認してください。

## Finding a minibrot or Misiurewicz point (the M key and the Go to dialog)

app-feature-needs-holomorphic = ミニブロットの検出には、正則な族（Mandelbrot / Multibrot）が必要です。
app-feature-not-in-julia = ミニブロットの検出はMandelbrot集合を探索します。使うにはJuliaモードをオフにしてください。
app-feature-busy = すでに特徴点を探しています。一度に1つずつです。
app-feature-searching = 最寄りのミニブロットを探しています…
app-feature-mandelbrot-only = 特徴点の検出はMandelbrot専用です。
app-feature-no-pair = 探索の予算内でMisiurewicz点が見つかりませんでした。点がこのビューで探索できる範囲より深くにあるか、中心が螺旋や分岐点の近くにない可能性があります。
# The name of what was found, used as $feature in the messages below. $period: the minibrot's period (a whole number).
app-feature-minibrot = 周期{ $period }のミニブロット
# $preperiod and $period: the point's two whole numbers (k, p). $octaves: how often the view repeats when zooming in (2 decimals); $twist: the turn per repeat in degrees.
app-feature-misiurewicz-lambda = Misiurewicz ({ $preperiod },{ $period })：{ $octaves }オクターブごとに繰り返し、ねじれ{ $twist }°
# $preperiod and $period: the point's two whole numbers (k, p).
app-feature-misiurewicz = Misiurewicz ({ $preperiod },{ $period })
# Added after the messages below when the fixed iteration count is too low for the new depth. $have: the iterations set now; $typical: what the depth typically needs.
app-feature-iter-note = ⚠ 反復回数は{ $have }に固定されています。この深度では通常、約{ $typical }が必要です。ビューが平坦にレンダリングされる場合は、反復回数を上げてください（または反復回数の自動調整を有効にしてください）。
# $feature: what was found (see above). $zoom and $asked: magnifications, already formatted; $octaves: the solver's depth limit.
app-feature-zoomed-capped = { $feature }にズームしました：{ $zoom }×。計算は{ $octaves }オクターブまでに制限されているため、{ $asked }×ではなくそこで止まりました。
# $feature: what was found. $zoom: the magnification, already formatted. $seconds: how long the search took (1 decimal).
app-feature-zoomed = { $feature }にズームしました：{ $zoom }×（{ $seconds }秒）
# $feature: what was found. $seconds: how long the search took (1 decimal).
app-feature-snapped = { $feature }の中心に合わせました（{ $seconds }秒）
app-feature-no-minibrot = ビューの近くにミニブロットの中心が見つかりませんでした。もっと近くまでズームしてください。
# $feature: the point asked for, e.g. Misiurewicz (3,1). $widths: how many view-widths away, already formatted. $octaves: how far to zoom out.
app-feature-just-outside = このビューのすぐ外に{ $feature }点が見つかりました（ビューの幅の{ $widths }倍離れています）。画面に入れるには約{ $octaves }オクターブズームアウトするか、このスケールの特徴点を探すなら前周期を大きくしてください。
# $feature: the point asked for. $widths: how many view-widths away, already formatted (may be 2^187). $octaves: how far to zoom out.
app-feature-far = { $feature }点が見つかりましたが、ビューの幅の{ $widths }倍離れています。その特徴点はこのビューよりはるかに粗いスケールのものです。そこへ行くには約{ $octaves }オクターブズームアウトするか、このスケールの特徴点を探すなら前周期を大きくしてみてください。
# $feature: the point asked for. $residual: a small number in scientific notation.
app-feature-not-preperiodic = { $feature }はここの軌道に合いません（残差 { $residual }）。両方の欄を空にして、ビューから組を検出してください。
# $feature: the point asked for. "k/p" are the dialog's two fields.
app-feature-not-converged = ビューの近くで収束する{ $feature }点がありませんでした。もっと近くへ移動するか、別のk/pを試してください。
app-feature-bad-request = 前周期と周期はどちらも正の値である必要があります。

## Share location dialog and location files

app-location-empty = 読み込むものがありません（またはテキストが大きすぎます）。
app-location-not-fractadyne = Fractadyneの位置ではありません。
# $zoom: the magnification, already formatted. $note: what in the text was ignored or repaired (in English).
app-share-loaded-note = { $zoom }×の位置を読み込みました：{ $note }。
# $error: the system's reason (often in English).
app-share-save-failed = 保存できませんでした：{ $error }
app-share-saved = 保存しました。
app-share-saved-rendering-thumb = 保存しました。サムネイルをレンダリングしています…
app-share-saved-no-thumb = 保存しました。サムネイルはありません（このビューではレンダリングできませんでした）。
app-share-saved-with-thumb = サムネイル付きで保存しました。
# $error: the system's reason (often in English).
app-share-saved-thumb-failed = 保存しましたが、サムネイルを追加できませんでした：{ $error }
app-share-loaded-into-box = ボックスに読み込みました。確認してから「適用」を押してください。
# Reading a file failed (a location or a palette). $error: the system's reason (often in English).
app-read-failed = 読み込めませんでした：{ $error }
app-share-too-large = ファイルが大きすぎます（.fdnの位置ではない可能性があります）。

## File-dialog filter names (the extensions are added by the code)

app-filter-kfr = Kalles Fraktalerの位置
app-filter-fdn = Fractadyneの位置
app-filter-map = Fractint / KFのパレット
app-filter-ugr = Ultra Fractalのグラデーション
app-filter-ase = Adobeスウォッチ交換
app-filter-ggr = GIMPのグラデーション
app-filter-all = すべてのファイル

## Importing and saving palette files (shown under the gradient editor)

# The reason a file was refused before reading it.
app-gradient-map-too-large = .mapパレットにしては大きすぎます
app-gradient-ugr-too-large = .ugrパレットファイルにしては大きすぎます
app-gradient-ase-too-large = .aseスウォッチリストにしては大きすぎます
app-gradient-ggr-too-large = .ggrグラデーションにしては大きすぎます
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported-vga = { $file }から{ $count }個のエントリをインポートしました。6ビットのVGAテーブル（白が255ではなく252）なので、Fractint自身の出力と一致するよう、書かれたとおりに保持しています。
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported = { $file }から{ $count }個のエントリを、ハードバンドとしてインポートしました。
# $error: the reader's reason (in English).
app-gradient-map-invalid = .mapパレットではありません：{ $error }
# $name: the gradient's name in the file; $file: the file's name.
app-gradient-ugr-imported = { $file }から「{ $name }」をインポートしました。
# $file: the file's name; $count: how many gradients it holds.
app-gradient-ugr-pick = { $file }には{ $count }個のグラデーションがあります。下から1つ選んでください。
# $error: the reader's reason (in English).
app-gradient-ugr-invalid = .ugrパレットファイルではありません：{ $error }
# $used: the swatches kept; $total: the swatches in the file.
app-gradient-ase-imported-some = { $total }個のスウォッチのうち{ $used }個を、等間隔に配置してインポートしました。
# $count: the swatches in the file.
app-gradient-ase-imported = { $count }個のスウォッチを、等間隔に配置してブレンドし、インポートしました。
# $error: the reader's reason (in English).
app-gradient-ase-invalid = .aseスウォッチリストではありません：{ $error }
app-gradient-nothing-to-save = 保存するカスタムグラデーションはまだありません。
# $file: the saved file's name.
app-gradient-ggr-saved = 「{ $file }」を保存しました。中間点、ブレンドカーブ、色空間はすべてそのままです。
# $file: the saved file's name; $count: the segments whose curve could not be kept.
app-gradient-ggr-saved-lossy = 「{ $file }」を保存しました。ただし、{ $count }個のセグメントが.ggr形式では表現できないベジェ曲線を使っています。それらは直線的なブレンドとして書き出しました。ライブラリ内のコピーには曲線が残っています。
# $path: the file; $error: the system's reason (often in English).
app-gradient-write-failed = { $path }に書き込めませんでした：{ $error }
# $name: the gradient's name; $count: its segments.
app-gradient-ggr-imported = 「{ $name }」をインポートしました：{ $count }個のセグメントと、その中間点、ブレンドカーブ、色空間。
# $error: the reader's reason (in English).
app-gradient-ggr-invalid = .ggrグラデーションではありません：{ $error }

## The eyedropper's banner across the top of the screen while picking a colour

# $hex: the colour under the pointer, as #rrggbb.
app-eyedropper-banner-color = 色を取得中  ·  { $hex }  ·  クリックで取得、Escでキャンセル
app-eyedropper-banner = 色を取得中  ·  クリックで取得、Escでキャンセル

## The gradient editor window

app-gradient-title = グラデーションエディター
# Before the two view buttons, Bar and Ring.
app-gradient-view = 表示
app-gradient-view-bar = バー
app-gradient-view-bar-hover = まっすぐな帯です。ストップを正確に配置するのに最適です
app-gradient-view-ring = リング
app-gradient-view-ring-hover = パレットは循環して使われるため、リングで表示します。上の接合部は、レンダラーが1周ごとに越える継ぎ目です。バーでは問題なく見えるグラデーションでも、そこにくっきりした境界ができることがあります。
app-gradient-add-lane-hover = この線上のどこかをクリックすると、そこにストップを追加します。+ は最も広い間隔に1つ追加します。
app-gradient-strip-hover = マーカーをドラッグしてストップを移動 · 帯自体をドラッグするとグラデーション全体を回転（継ぎ目で折り返し） · ダブルクリックでストップを追加、マーカーを右クリックで削除 · 矢印キーで選択を微調整（Shiftでさらに細かく）。
app-gradient-remove-selected-hover = 選択したストップを削除
app-gradient-ring-hover = リング自体をドラッグするとグラデーション全体を回転します。継ぎ目を越えた部分は反対側から戻ってきます · マーカーをドラッグしてストップを移動 · 外側のトラックをクリックするとそこにストップを追加、中央の ⊕ で最も広い間隔に追加 · ⊖ で選択したストップを削除。上の白抜きの印は継ぎ目で、循環するパレットが折り返す場所です。そこが接合部そのものなので印は動かず、色がその下を回ります。
app-gradient-empty = カスタムグラデーションはまだありません。「プリセットをコピー…」、インポート、または下の保存したグラデーションから始めると、そのストップとセグメントごとのカーブがここに表示されます。
# $count: the entries of an imported palette, more than the editor shows stop by stop.
app-gradient-too-many = インポートしたエントリが{ $count }個あり、ストップごとに編集するには多すぎます。上のプレビューを使うか、「プリセットをコピー」でやり直してください。
# The selected segment's heading. $index: its number; $total: how many there are; $left and $right: where it starts and ends, 0.000–1.000.
app-gradient-segment-heading = セグメント { $index }/{ $total }  ·  { $left } → { $right }
# A label before the list of blend curves.
app-gradient-curve = カーブ
app-gradient-curve-hover = このセグメントで色が移り変わる速さです。「曲線」は中間点で形が決まり、中間点が0.50のときは正確に直線になります。
# The blend curves (GIMP's kinds), in a narrow list: keep them short.
app-gradient-blend-linear = 直線
app-gradient-blend-curved = 曲線
app-gradient-blend-sine = 正弦
app-gradient-blend-sphere-increasing = 球面 ↑
app-gradient-blend-sphere-decreasing = 球面 ↓
app-gradient-blend-bezier = ベジェ
# Hover on Bézier in the list of blend curves.
app-gradient-bezier-exact-hover = 両方向にドラッグできる2つの制御点です。このカーブはすでに3次曲線なので、変換しても何も変わりません。
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-visible-hover = 両方向にドラッグできる2つの制御点です。変換すると現在のカーブを近似します。このカーブでは誤差が{ $error }以内で、はっきり目に見えます。
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-barely-hover = 両方向にドラッグできる2つの制御点です。変換すると現在のカーブを近似します。このカーブでは誤差が{ $error }以内で、ほとんど目に見えません。
# A label before the list of colour spaces (RGB, HSV).
app-gradient-space = 色空間
# A label before the Bézier presets beside it.
app-gradient-ease = イージング
app-gradient-ease-hover = プリセットは2つの制御点を書き込みます。それ以外の形にするには、カーブ上で制御点をドラッグします。ベジェには中間点がなく、ハンドルが半分に達する位置を決めます。
# Bézier presets, as in CSS easing: short buttons.
app-gradient-ease-linear = 直線
app-gradient-ease-in = イン
app-gradient-ease-out = アウト
app-gradient-ease-in-out = イン・アウト
app-gradient-midpoint = 中間点
app-gradient-midpoint-hover = ブレンドが半分に達する位置を、このセグメントに対する割合で示します。0.50で中央です。
# A button that puts the midpoint back in the middle of the segment.
app-gradient-centre = 中央
app-gradient-centre-hover = 0.50に戻す
app-gradient-no-hue = ⚠ このセグメントの一方の端に色相がありません
app-gradient-no-hue-hover = 一方の端が黒、白、または灰色なので、色の移り変わりは赤から始まって色相環を一周します。これは正しい動作で、望ましい場合もありますが、中間がどちらの端にも含まれない色になるのはこのためです。カーブの下の帯で、その様子がわかります。
app-gradient-segment-start-hover = このセグメントの始点の色
app-gradient-segment-end-hover = このセグメントの終点の色
app-gradient-bytes-hover = 赤、緑、青を0～255で
app-gradient-floats-hover = 同じ色を0～1で表したもの。レンダラーが保存する形式です
app-gradient-canvas-bezier-hover = 選択したセグメントのブレンドカーブです。どちらかの制御点をx方向とy方向の両方にドラッグして形を変えます。薄い線は0と1です。カーブはその外側にはみ出すことがあり、その場合はカーブではなく色のほうが範囲内に収められます。
app-gradient-canvas-hover = 選択したセグメントのブレンドカーブです：左のストップから右のストップへ色が移り変わる速さを示します。リングをドラッグして中間点を動かすか、カーブをベジェに切り替えると、両方向に動かせる制御点が使えます。
# The selected stop's heading. $index: its number; $total: how many there are.
app-gradient-stop-heading = ストップ { $index }/{ $total }
app-gradient-stop-color-hover = このストップの色です。色見本をクリックして選ぶか、横に16進値を入力します
app-gradient-stop-bytes-hover = 赤、緑、青を0～255で。この色が書き出されるときのバイト値です
# The eyedropper button.
app-gradient-pick = スポイト
app-gradient-pick-hover = スポイト：画面上のどこからでも色を取得します。ポインターを目的の色に移動してクリックします。Escでキャンセルします。
# Written before the stop's position (0.000–1.000) in its number field: a short abbreviation of "position".
app-gradient-stop-pos = 位置
app-gradient-nudge-hover = このストップを微調整（Shiftを押しながらでさらに細かく）
app-gradient-remove = 削除
app-gradient-remove-hover = このストップを削除（またはその下の ⊖ をクリック）
app-gradient-add-stop = ストップを追加
app-gradient-add-stop-hover = 最も広いセグメントを分割します（またはグラデーションの上の線をクリック）
# A menu of the built-in palettes, to start from a copy of one.
app-gradient-copy-preset = プリセットをコピー…
app-gradient-paste = 貼り付け…
app-gradient-paste-hover = 16進の色、または0～255のRGBの3つ組からパレットをインポートします
app-gradient-save-ggr = .ggrを保存…
app-gradient-save-ggr-hover = このグラデーションをGIMPの.ggrファイルに書き出します。GIMP、Krita、Inkscape、そしてこのエディターで読み込めます
# A menu of the palette file formats (.map, .ugr, .ggr, .ase).
app-gradient-import = インポート
app-gradient-import-map-hover = Fractint / Kalles Fraktalerの.mapパレットファイルを読み込みます（エントリごとに1行のR G B）
app-gradient-import-ugr-hover = Ultra Fractalの.ugrグラデーションファイルを読み込みます（.ugrには通常多くのグラデーションが入っているので、1つ選びます）
app-gradient-import-ggr-hover = GIMPの.ggrグラデーションを、中間点、ブレンドカーブ、色空間をそのままにして読み込みます
app-gradient-import-ase-hover = Adobeのスウォッチリスト（.ase）を読み込みます。スウォッチには位置がないため、等間隔に配置してブレンドします。
# The gradient's name, for the library of saved gradients.
app-gradient-name = 名前
# Shown greyed in the empty name field.
app-gradient-name-hint = マイグラデーション
# The save button, when a gradient of that name is already saved.
app-gradient-update = 更新
app-gradient-save = 保存
app-gradient-save-hover = このグラデーションを、カーブと色空間をそのままにしてライブラリに保存します
app-gradient-save-needs-name-hover = 先にグラデーションに名前を付けてください
# A menu of the saved gradients. $count: how many there are.
app-gradient-saved-menu = 保存済み（{ $count }）
app-gradient-nothing-saved = まだ何も保存されていません
app-gradient-delete-saved-hover = この保存済みグラデーションを削除
# $count: the gradient's segments.
app-gradient-rich = リッチグラデーション：{ $count }個のセグメントが中間点、ブレンドカーブ、色空間を持っており、単純なストップのリストには収まりません。
app-gradient-convert = 編集可能なストップに変換
app-gradient-convert-hover = 単純なストップのリストに置き換えます。ストップのリストでは表せない中間点、ブレンドカーブ、色相の回り方は破棄されます。
# Above the list of gradients in an Ultra Fractal file. $count: how many; $file: the file's name.
app-gradient-ugr-list = { $file }内のグラデーション：{ $count }個
app-gradient-close-list = リストを閉じる
app-gradient-seamless = 継ぎ目なし（終点を始点に合わせる）
app-gradient-seamless-hover = パレットは循環して使われるため、位置1と位置0は画面上で隣り合います。これをオンにすると、最後のストップの色が最初のストップと同じに保たれます。リング表示では上に接合部が表示されます。
app-gradient-ends-differ = ⚠ 両端の色が異なります
app-gradient-ends-differ-hover = グラデーションの終点の色が始点の色と異なるため、循環するパレットでは折り返す場所にくっきりした境界が現れます。
app-gradient-hard-bands = ハードバンド（エントリ間をブレンドしない）
app-gradient-hard-bands-hover = Fractint / .mapの考え方です：各エントリが単色になります。オフにすると、エントリをなめらかなグラデーションにブレンドします。
app-gradient-paste-intro = 16進の色（#ff8800）または0～255の3つ組（255 136 0）を、カンマ、スペース、改行で区切って貼り付けてください：
# Applies the pasted colours.
app-gradient-apply = 適用
# Empties the paste box.
app-gradient-clear = クリア
# $used: the colours kept; $total: the colours pasted; $limit: the most that are kept.
app-gradient-paste-imported-some = { $total }色のうち{ $used }色を、リスト全体から均等に抜き出してインポートしました（上限は{ $limit }色）。
# $count: the colours pasted.
app-gradient-paste-imported = { $count }色をインポートしました。
# $error: why the pasted text could not be read (in English).
app-gradient-paste-unreadable = 読み取れませんでした：{ $error }
# A hint at the bottom of the editor. $count: the palette's entries.
app-gradient-bands-hint = { $count }本の帯 · ファイルの順に使用。ハードバンドがオンの間は位置は無視されます。
# A hint at the bottom of the editor, in the ring view. $count: the gradient's stops.
app-gradient-ring-hint = ストップ{ $count }個 · 外側のトラックをクリックでそこに追加 · ⊕ で最も広い間隔に追加、⊖ で選択したストップを削除 · マーカーをドラッグで移動 · 上の印は継ぎ目
# A hint at the bottom of the editor, in the bar view. $count: the gradient's stops.
app-gradient-bar-hint = ストップ{ $count }個 · グラデーションの上の線をクリックでそこに追加 · マーカーをドラッグで移動 · ⊖ または右クリックで削除
app-gradient-cancel = キャンセル
app-gradient-cancel-hover = このウィンドウを開いてから行ったすべての変更を破棄して、閉じます
app-gradient-ok = OK
app-gradient-ok-hover = 変更を保持してエディターを閉じます

## Shared

# The button that closes the Help window and the "Faster deep zoom" window.
app-close = 閉じる

## The "Faster deep zoom" window (Help menu): the optional accelerated build

app-accel-title = ディープズームの高速化
app-accel-active-heading = 高速版ビルドを実行中です
app-accel-active-body = ディープズームの参照軌道はMPFR/GMPで計算されています。この処理（深いビューが解像し始めるまでの待ち時間）では、標準ビルドより2.5～6.4倍高速です。
# $status: which arithmetic library is in use and its version (in English).
app-accel-in-use = 使用中の演算：{ $status }
app-accel-fallback-heading = 高速版ビルド：組み込みの演算を使用中
app-accel-fallback-body = これは高速版（MPFR）ビルドですが、数学ライブラリが見つからないため、組み込みの演算（astro-float）を使っています。画像は同一で、深いビューが解像するまでの待ち時間が長くなるだけです。
# Followed by the folder the program runs from, on its own line.
app-accel-fallback-dlls = 高速な処理を使うには、libmpfr-6.dllとlibgmp-10.dll（およびlibgcc_s_seh-1.dll、libwinpthread-1.dll）をfractadyne.exeと同じフォルダーに置いてから、再起動してください：
app-accel-warning-hidden = これに関する起動時の警告は非表示になっています。
app-accel-show-again = 再び表示する
app-accel-offer-heading = 深い位置で2.5～6.4倍速い、オプションのビルドがあります
app-accel-offer-body = ディープズームでは、CPUで参照軌道を計算するのに多くの時間がかかります。深いビューが解像し始めるまでの待ち時間がそれです。オプションのビルドはこれをMPFR/GMPライブラリで行い、2.5～6.4倍速く、深くなるほど差が広がります。それ以外はすべて同じです。
app-accel-offer-identical = 画像はバイト単位で同一です。同じ数学をより高速なライブラリで計算しているだけで、すべてのフラクタルの数式と、ディープズーム比較用のコーパス全体で確認済みです。
app-accel-offer-shared = 設定、保存されたセッション、位置は2つのビルドで共有されるので、自由に切り替えられ、何もインポートする必要はありません。
# Shown on Windows.
app-accel-separate-windows = 別のダウンロードになっているのは、使用しているライブラリが標準のWindowsビルドのコンパイラーではビルドできず、Fractadyne自身とは異なるライセンス（GNU LGPL v3）だからです。
# Shown on other systems.
app-accel-separate = 別のダウンロードになっているのは、使用しているライブラリがFractadyne自身とは異なるライセンス（GNU LGPL v3）だからです。
app-accel-download = このバージョン用をダウンロード
app-accel-all-releases = すべてのリリース
app-accel-all-releases-hover = 直接のリンクが404になる場合、このバージョンにはまだ高速版ビルドがありません。ここで最も近いものを選んでください。
# Shown on Windows.
app-accel-install-windows = 展開して、そのフォルダーからfractadyne.exeを実行してください。.dllファイルは同じフォルダーに置いたままにします。
# Shown on Linux.
app-accel-install-linux = 展開して、そのフォルダーから./fractadyneを実行してください。システムのGMPとMPFR（libgmp10 / libmpfr6）を使います。これはほぼすべてのデスクトップ環境に入っています。詳しくはパッケージ内のREADME-ACCELERATED.txtを参照してください。
# Shown on other systems.
app-accel-install-other = このプラットフォーム用の高速版パッケージはまだビルドされていません。--features fractadyne-core/rugを指定してソースからビルドできます。

## The Help window's frame (its sections are under help-)

app-help-title = Fractadyneヘルプ

## Checking for updates

app-toast-up-to-date = 最新バージョンを使用しています。
# $error: why the check failed (in English).
app-toast-update-failed = アップデートを確認できませんでした：{ $error }
# The window that offers a newer version.
app-update-title = アップデートがあります
# $version: the new version (0.3.0-beta.20). $channel: the token "stable" or "beta", in English; a translation can choose its words by it ({ $channel -> [beta] … *[other] … }).
app-update-version =
    { $channel ->
        [beta] Fractadyne { $version }（ベータ版）が利用可能です
       *[other] Fractadyne { $version }（安定版）が利用可能です
    }
# $current: the version running now. $track: the update track the user follows (its name in the language in use).
app-update-running = 現在のバージョン：{ $current }  ·  { $track }
app-update-download-intro = GitHubから最新リリースをダウンロードしてください：
app-update-download = GitHubからダウンロード
app-update-later = 後で通知する

## The second graphics card (shown in its window, after "In use:" or "Could not start:")

# The second device is the same card as the window's. $name: the card's name.
app-worker-same-card = 同じカード、{ $name }
# Why the second device could not start.
app-worker-thread-stopped = デバイスのスレッドが停止しました

## Export status (shown in the Export image window)

app-export-gpu-unavailable = GPUを使用できません
app-export-reference-aborted = エクスポートに失敗しました：参照軌道の構築が中止されました。

## The performance panel (View ▸ Performance panel)
## Its row labels sit in a monospace column about 14 characters wide: keep them short (a longer label pushes its value to the right).

app-perf-fps = FPS
app-perf-fps-hover = 1秒あたりに表示されたフレーム数で、フレーム間隔を平滑化して求めます。ビューの仕上げ中は、代わりに精細化の進行状況を表示します。仕上げ中のビューは、ここに数値が示すようなレートで動いているわけではありません。
# What the FPS row shows while a view is settling. $done and $total: tiles; $eta: the time left, as m:ss.
app-perf-refining-tiles = 精細化 { $done }/{ $total }（残り約{ $eta }）
app-perf-refining-arming = 精細化（準備中）
# $percent: how much is done, with one decimal.
app-perf-refining-percent = 精細化 { $percent }%
app-perf-building-reference = 参照軌道を構築中
app-perf-idle = 待機中
app-perf-frame = フレーム
app-perf-frame-hover = フレーム間の実時間を平滑化したもので、下の行の合計ではありません。表示を待つ時間も含むため、仕上がって何もしていないビューでは、処理量ではなく再描画の間隔を反映します。
app-perf-cpu = CPU
app-perf-cpu-hover = 1回の更新でのメインスレッドの時間：パラメーターの準備と参照軌道キャッシュの管理。参照軌道の再構築で跳ね上がるのはこの行で、ここでのスパイクが体感するカクつきです。
app-perf-gpu-idle = GPU/待機
app-perf-gpu-idle-hover = フレーム時間からメインスレッドのCPU時間を引いたものです。ビューのレンダリング中はGPUの処理時間ですが、仕上がったビューではほとんどが待機時間です。この引き算だけでは両者を区別できないため、名前が両方を表しています。
app-perf-timing = 計時
# How frame cost is measured, the timing row's value. TIMESTAMP_QUERY is a name.
app-perf-timing-starved = 実時間（タイムスタンプ途絶）
app-perf-timing-gpu = GPUタイムスタンプ
app-perf-timing-no-query = 実時間（TIMESTAMP_QUERYなし）
app-perf-timing-hover = フレームのコストの測り方です。GPUタイムスタンプが正確な情報源で、計測値が届かなくなると実時間による代替に切り替わります。実時間で見積もった予算は、深いフレームの大きさを何度も見誤ってきました。ここが赤いときは「安全マージンは現在推定値である」という意味に受け取ってください。
app-perf-mode = モード
app-perf-mode-hover = 直前のフレームのレンダリングに使った演算方式：direct（参照軌道なし）、df32摂動、またはfloatexp（約1e28x以降で使う拡張範囲の方式で、BLAが動作する唯一の方式）。
app-perf-eff-iter = 実効反復
app-perf-eff-iter-hover = 直前のフレームで実際に使われた反復回数です。深度によるスケーリングと移動中の上限を適用したあとの値で、品質の基本設定（出発点にすぎません）ではありません。
app-perf-precision = 精度
app-perf-precision-hover = 参照軌道を計算する仮数部のビット数です。深度とともに増えます。64ビットは浅い位置での下限であり、突き当たっている限界ではありません。
app-perf-orbit-len = 軌道長
# $cap: the most orbit samples this GPU holds, already formatted, or the text of app-perf-uncapped.
app-perf-orbit-len-hover = 参照軌道のサンプル数です。このGPUが保持できるのは最大{ $cap }で、これに近づくことが実質的な深度の壁です：それを超えると、より長い軌道を必要とする深い位置をライブビューでは解像できません。
app-perf-uncapped = （上限なし）
app-perf-sa-skip = SAスキップ
app-perf-sa-skip-hover = ピクセルごとの処理が始まる前に、級数近似が飛ばした反復回数です。多いほど速く、画像は何も変わりません。何も飛ばさなかったときはこの行は表示されません。
app-perf-aa = AA
app-perf-aa-hover = スーパーサンプリング：各ピクセルを軸ごとにこの数のサンプルからレンダリングするので、2xは4倍の処理量になります。
app-perf-dual = デュアル
app-perf-dual-hover = Mandelbrot/Juliaの分割ビューが動作しているかどうかです。trueのとき、このパネルのすべてのコストは両方のペインを含みます。
app-perf-zoom = ズーム
app-perf-zoom-hover = 全体表示に対する倍率です。上の演算モードは約1e4xと、さらに約1e28xで切り替わります。
app-perf-iterate = 反復処理
# $target and $lethal: times in milliseconds, already formatted.
app-perf-iterate-hover = 直前の反復ディスパッチで測定されたGPU時間です。制御機構は{ $target } msを目標にしています。{ $lethal } ms付近の値が続くのは、このクラスのハードウェアでデバイスロストが起きてきた範囲です。
app-perf-steps = ステップ/秒
app-perf-steps-hover = GPUでの1秒あたりの反復ステップ数（10億単位）です。内部の多いビューは、数だけから想像されるよりもステップあたりの速度が遅くなるため、これはスコアではなくスループットの読み値です。
app-perf-budget = 予算
# After the budget's value while it is still converging; the hover below quotes it.
app-perf-budget-settling = （収束中）
app-perf-budget-hover = ディスパッチごとの適応的なステップ予算で、ドライバーがGPUをリセットするほど長いフレームを防ぐ制御機構の安全策です。「収束中」は、まだまばらな計測値から収束している途中で、信頼できる上限ではないことを意味します。
app-perf-rss = RSS
# $mb and $peak: memory in megabytes, already formatted.
app-perf-rss-value = { $mb } MB（ピーク { $peak }）
app-perf-rss-hover = プロセスの常駐メモリと、このセッションでの最大値です。深い参照軌道の構築では、画面上のほかの表示に何も現れないまま、実際の使用環境で2 GBを超えたことがあります。システムRAMの4分の3を超えると琥珀色になります。
app-perf-gpu-est = GPU推定
app-perf-gpu-est-hover = GPU上のメモリの推定値：参照軌道、BLAツリー、反復テクスチャ、精細化の実行中に交互に使うチャンク状態、表示ゲートのスナップショット。既知の割り当てサイズから算出しています。ドライバーの実際の値は、移植可能な方法では読み取れません。
app-perf-julia-re = julia c.re
app-perf-julia-re-hover = Juliaペインがレンダリングしているパラメーターcの実部です。ピン留めしていなければ、カーソルが指している点です。
app-perf-julia-im = julia c.im
app-perf-julia-im-hover = Juliaペインがレンダリングしているパラメーターcの虚部です。
app-perf-julia-c = julia c
# The julia c row's value when the parameter is held fixed.
app-perf-pinned = ピン留め
app-perf-pinned-hover = パラメーターは固定された点に保たれているため、カーソルを動かしてもJuliaペインは変わりません。
app-perf-c-panel = c/パネル
app-perf-c-panel-hover = Mandelbrotパネル全体が占めるc空間の幅です。これがJuliaの1ピクセルよりはるかに小さいと、ポインターを動かしてcが変わってもJuliaは変わらないように見えます。ディープズームでは想定どおりの動作で、フリーズではありません。
app-perf-ref-recompute = 参照再計算
app-perf-ref-recompute-hover = 最後に参照軌道を再計算したときの所要時間です。これは深いビューが解像し始めるまでの待ち時間で、オプションの高速版ビルドが速くする処理です。
app-perf-recompute-rate = 再計算/秒
app-perf-recompute-rate-hover = 1秒あたりに導入された参照軌道の数です（結果が実際に使われたもの）。下の行と比べてください：その差が捨てられた処理です。
app-perf-builds-rate = 参照構築/秒
app-perf-builds-rate-hover = 1秒あたりに開始された参照軌道の構築数で、結果が破棄されたもの（ダイブが到達しない先読みスロットや、拒否された延長）も含みます。これらはCPUを丸ごと消費し、ほかのどこにも表示されません：かつてプリフェッチの空回りが、再計算/秒が2を示している間に、毎秒約400回の構築を6分間続けたことがあります。1桁なら正常です。この2行の差が大きい場合は、処理が捨てられています。
app-perf-recompute-total = 再計算合計
app-perf-recompute-total-hover = 起動以降の参照軌道の再計算回数です。既知の時間間隔での割合として使えます。何も動いていないのに数が増え続けるなら、それこそ追うべき兆候です。

## Help: the contents list (Help & reference…, F1)

# Each section's name in the contents list (about 165 px wide: keep them short), and its heading.
help-section-overview = 概要
help-section-navigation = ナビゲーション
help-section-expressions = 座標の式
help-section-options = 着色とオプション
help-section-fractals = フラクタル
help-section-how-it-works = しくみ
help-section-command-line = コマンドライン
help-section-shortcuts = ショートカット
help-section-hardware = 推奨ハードウェア
# Credits to the prior work Fractadyne builds on.
help-section-acknowledgments = 謝辞
# The third-party license notices.
help-section-licenses = ライセンス
help-section-about = バージョン情報

## Help ▸ Overview

help-overview-intro = Fractadyneは、超深度のズームと速度のために作られたネイティブのフラクタル探索ソフトウェアです。「脱出時間」フラクタル、つまり1つの簡単な数式を各ピクセルで繰り返すことで生まれる画像を描きます。
help-overview-what-is = 脱出時間フラクタルとは？
help-overview-what-is-text = プログラムは各ピクセルについて、z → z² + cのような数式をゼロから始めて何度も繰り返し計算します。値がいつまでも小さいままなら、そのピクセルは集合に属し、暗く描かれます。値がやがて限りなく大きくなる（「脱出する」）なら、そのピクセルは集合の外にあり、その色は脱出までにかかったステップ数を表します。「とどまる」と「脱出する」の間にある無限に細かい境界が、フラクタルです。
help-overview-can-do = できること
help-overview-can-pan = 事実上無制限にパンとズームができます（どの深度でも位置は正確です）。
help-overview-can-families = 25種類の組み込みフラクタルの族（10種類と、そのうち5種類の高次版）、自作の数式、ライフゲーム、Lシステムを切り替え、Newtonを除く脱出時間型の族はどれでもJulia集合として表示できます。
help-overview-can-recolor = プリセットやカスタムのグラデーションと、いくつかの着色方法で色を付け直せます。
help-overview-can-lighting = 3Dのレリーフ照明や、光る境界の等高線を加えられます。
help-overview-can-minibrots = ミニブロットに合わせたり、場所をブックマークしたり、高解像度の画像をエクスポートしたりできます。
help-overview-can-autopilot = 自動ズームのオートパイロットに、細部へ向かって自動でダイブさせられます。
help-overview-can-share = どの場所でも、小さな「.fdn」テキストやファイルとして共有できます。
help-overview-can-tours = スクリプトによるツアーや、ハードウェアのベンチマークを実行できます。
help-overview-first-steps = はじめの一歩
# "Seahorse Valley" is a place in the Navigate menu: use its name as that menu shows it. A and F1 are keys.
help-overview-first-steps-text = ナビゲートメニューを開いて「Seahorse Valley」を選び、マウスホイールを回してズームインします。ドラッグでパンします。または、Aを押してオートパイロットにダイブを任せましょう。F1を押せば、いつでもこのヘルプに戻れます。

## Help ▸ Coordinate expressions
## The expressions, numbers and function names here are typed as shown in every language:
## keep them exactly as written. Menu items in quotation marks are the menus' own labels.

help-expr-intro = Fractadyneが座標を受け取る場所ではどこでも、小数の代わりに式を入力できます：ナビゲート → 「位置へ移動…」のRe / Im欄（とその極座標モードの各欄）、.fdnの位置のcenter_re / center_im行、ツアースクリプトのre / im、そしてコマンドラインの--center。式はズームに必要なだけの桁数で評価され、位置へ移動ダイアログは式をビューと一緒に保持します。そのため、1/3や-0.5 + 0.25*cos(pi/4)として入力した点は、どれだけ深くズームしても正確にその点の上にとどまります。小数では、桁が尽きた時点でずれていきます。
help-expr-examples = 例
help-expr-ex-third = 正確な3分の1です。ズームインするにつれて桁が増えます。
help-expr-ex-neck = Seahorse Valleyのくびれ、つまり周期2のバルブの根元です。
help-expr-ex-circle = -0.5を中心とする半径1/4の円上の点です（極座標モードならこれを組み立ててくれます）。
help-expr-ex-bulb = 周期3のバルブの根元のReです。そのImはsin(2*pi/3)/2 - sin(4*pi/3)/4です。ナビゲート → 「厳密な点（式）」にこれらが組み込まれています。
help-expr-ex-complex = Re欄に複素数の値全体を入力すると、ReとImの両方が埋まります。
help-expr-ex-combine = 関数と定数は自由に組み合わせられます。
help-expr-ex-powers = べき乗：2^-20は1/1048576です。-2^2は-4です（マイナスはべき乗のあとに適用されます）。
help-expr-ex-root = -2、つまり実数の立方根です。負の数の偶数乗根は拒否されます。sqrt(-1)はiになります。
help-expr-ex-scientific = 式の中でも指数表記が使えます。
help-expr-numbers = 数値と演算子
# The decimal point in an expression is '.' in every language: keep that rule as it is.
help-expr-numbers-decimals = 小数（指数は省略可）：0.25、-1.5e-30、.5。数字のみで、数値の中にスペースや桁区切りのカンマは入れられません。小数点は常に'.'です。
help-expr-numbers-imaginary = 虚数にはiの接尾辞を付けるか（0.25i）、iを単独で使います：3 + 4*i。
# Starts with the operators themselves, two spaces apart.
help-expr-numbers-operators = +  -  *  /  と ^（べき乗。右から左に評価されるので、2^3^2 = 2^9）。括弧でグループ化します。暗黙の乗算はありません：2piではなく2*piと書きます。
help-expr-numbers-case = 名前の大文字と小文字は区別されません：PI、Pi、piは同じ定数です。
help-expr-constants = 定数
help-expr-functions = 関数
help-expr-functions-angles = 角度はラジアンです。度を使うには、pi/180を掛けるか（cos(45*pi/180)）、位置へ移動ダイアログを極座標モードに切り替えて単位に度を選びます。
help-expr-refused = 拒否されるものとその理由
help-expr-refused-fractional = (-2)^(1/2)のような、負の数の分数乗：結果は複素数になり、選ぶべき根が複数あります。実数の根にはroot(x, n)やcbrt(x)を、主値の複素数の根にはsqrt(x)を使ってください。
help-expr-refused-complex = 実数値関数への複素数の引数（sin(i)、ln(1+i)）。複素数の引数を受け付けるのはsqrtとabsだけです。算術演算子と整数のべき乗は複素数の値にも使えます。
help-expr-refused-domain = 定義域のエラー：正でない数のlnやlog、-1…1の範囲外のasinやacos、0の負のべき乗、ゼロによる除算。
help-expr-refused-bounds = ±2^32を超えるsin、cos、tan、exp、^の引数と、32段階を超える入れ子：貼り付けられた座標は信頼できない入力であり、これらは際限なく時間がかかる可能性があるためです。
help-expr-refused-grammar = 変数、ユーザー定義関数、比較、条件分岐はありません。保存された位置がどのバージョンでも同じ意味になるよう、文法は意図的に小さく保たれています。
help-expr-no-parse = 解析できない場合
help-expr-no-parse-text = メッセージは欄の名前、見つかったものとその文字位置を示し、よくある間違いについては代わりに入力すべきものも示します：文書から貼り付けた活字のマイナス記号（−）、×や÷、小数点のカンマ、数と名前の間の*の欠落、閉じていない括弧、括弧なしで書かれた関数。中途半端に読み取られることはありません：テキストのどこかが理解できなければ、黙って短くするのではなく、座標全体が拒否されます。

## Help ▸ Navigation
## Menu paths (Navigate → "Go to location…") and quoted items name the interface's own
## menus and controls: use their translations. Key names (Ctrl, Shift, Space, Esc) stay.

# A heading: the mouse controls (also in Shortcuts).
help-mouse = マウス
# The left column of a controls table: a gesture or key. Keep key names (Shift, Space, Ctrl) as they are.
help-key-left-drag = 左ドラッグ
help-nav-left-drag = ビューをパンします。
help-key-mouse-wheel = マウスホイール
help-nav-wheel = カーソルに向かってズームイン/アウトします。速く回すほど、1ノッチあたりのズーム量が大きくなります。「ズームを特徴点に吸着」がオンのときは、ミニブロットの近くでズームインすると、ズームがそこへ引き寄せられます。
help-key-shift-drag = Shift+ドラッグ
help-nav-shift-drag = ボックスズーム：長方形をドラッグすると、それがビューいっぱいになるようにズームします。
# $icon: the click-to-zoom toolbar button's symbol.
help-key-click-zoom = クリックでズーム（{ $icon }）
help-nav-click-zoom = ツールバーの照準ボタンをオンにします（単一ビュー）：左クリックでその点へ設定した倍率でダイブし、右クリックで戻ります。ドラッグは引き続きパンです。倍率（2～100×）は右側のパネルの「ナビゲート」で選びます。Backspaceでクリックを元に戻せます。
help-nav-continuous = 連続ズームとホーム
help-key-hold-space = Spaceを長押し
help-nav-hold-space = カーソルを基準に、なめらかにズームインします。
help-key-hold-shift-space = Shift+Spaceを長押し
help-nav-hold-shift-space = なめらかにズームアウトします。
help-nav-continuous-text = ホームへのズームボタンは、全体表示へゆっくり戻るアニメーションを行います。右側のパネルの「ズーム速度」で、連続ズームの速さを設定します。
help-nav-autopilot = オートパイロット
help-nav-autopilot-text = A（またはツール → 「自動ズーム（オートパイロット）」）を押すと、操作不要のダイブが始まります：フラクタルの境界上で細部が最も密な点を、ビューの中央寄りを優先して選び、それを画面の中心へ寄せながら滑るようにズームしていきます。目標が面白いうちはそれを保ち、より豊かな構造が現れるとなめらかにそちらへ向きを変えます。ナビゲーション操作（またはEsc）で停止します。右側のパネル（またはツール）の「自動ズームの優先」で、何を優先するかを選びます：「速度」は設定した速さでズームし、細部のあるフレームができしだい画面を更新します。「品質」は完全に解像したフル解像度のフレームだけを表示し、次のフレームが届く前に画像が大きく拡大されすぎないよう、必要に応じてズームを遅くします。どちらの場合も、空白や単色のフレームが画面に表示されることはありません：まだ細部が解像していない更新は、前のフレームの陰に隠れたままです。「自動ズームの目標」で狙う対象を選びます：「細部」（ビュー内で最も豊かな境界。進みながら選び直します）または「Misiurewicz点」です。後者は最寄りのMisiurewicz点を見つけ、静止したままダイブ上限の深度まで計算してから、そこへ正確にダイブします。そのような点の周りの構造は数オクターブごとに永遠に繰り返すので、このダイブが行き止まりになることはありません（Mandelbrot集合のみ）。
help-nav-history = 履歴と正確な移動
help-nav-undo = 前のビューに戻します。
help-nav-redo = やり直します。
help-nav-goto = ナビゲート → 「位置へ移動…」では、正確な中心とズームを読み取り、入力し、貼り付け、コピーできます（完全な精度で、どの深度でも）。中心の欄には単なる数値以上のものを入力できます：「1/3」のような分数や、pi、tau、e、sqrt、sin/cos/tanを使った式です。たとえば「-0.5 + 0.25*cos(pi/4)」は有効な座標です。式はビューと一緒に保持され、深度ごとに必要な精度で再評価されるので、そのように入力した点はどれだけズームインしても正確なままです（丸めた小数ではいずれずれます）。極座標モード（オフセット + r ∠ θ、単位は度 / ラジアン / 回転）では、このような式を組み立ててくれます。関数と定数の一覧と例は、このヘルプの「座標の式」セクションにあります（ダイアログの ? ボタンで開きます）。ナビゲート → ブックマークで、位置を保存して呼び出せます。
help-nav-exact-points = ナビゲート → 「厳密な点（式）」は、丸めた小数ではなく厳密に定義された点へジャンプします：主カージオイドのバルブの根元、つまり周期qの芽が付いている場所を、cos/sinの式で表したものです。式がビューと一緒に移動するので、その点からずれることなくズームインできます。
help-nav-share = ナビゲート → 「位置を共有…」は、ビュー全体（フラクタル、完全精度の中心、ズーム、着色）をコンパクトな「.fdn」テキストとして取り込みます。コピーし、貼り戻して「適用」したり、ファイルとして保存/読み込みしたりして、場所を正確に再現できます（ほかの人に送ることもできます）。
help-nav-open = ファイル → 「ビューまたは位置を開く…」では、次のどれでも直接開けます：エクスポートしたPNG/EXR（埋め込まれたビューが復元されます）、「.fdn」位置ファイル、Kalles Fraktalerの「.kfr」。
help-nav-finding = 細部を探す
help-nav-minibrots = ミニブロットは、境界に沿って隠れている集合全体の小さなコピーです。おおよそ中央に置いてM（またはツール → 「ミニブロットを探してズーム」）を押すと、ニュートン法でその中心に正確に合わせ、周期を読み取り、ミニブロット自体の大きさまで一気にズームします。1回で何桁も深くなることがよくあります。すでにミニブロットより深くにいる場合は、中心だけが移動します。
help-nav-minimap = 表示 → 「ミニマップ」は、「現在地」マーカーと現在のズーム深度を添えた、集合全体の小さな地図を表示します。地図をクリックすると、その領域へジャンプします。

## Help ▸ Coloring & options
## Names in quotation marks are the controls' own labels in the right-hand panel.

help-options-intro = これらはすべて右側のパネルにあります（セッションをまたいで保持されます）。
help-options-palette = パレット
help-options-palette-text = 「パレット」では、プリセットのグラデーション、自分のカスタムグラデーション、または2色モードを選びます。「グラデーションを編集…」で開くエディターでは、各カラーストップが色と位置（0～1）を持ちます。ストップを追加したり、プリセットをコピーして出発点にしたり、パレットファイルをインポートしたりできます。
help-options-paste = グラデーションエディターの「貼り付け…」は、ほかで見つけたパレットをインポートします。先頭の#があってもなくてもよい16進の色（#ff8800、ff8800、または短縮形の#f80）と、「255 136 0」のような0～255のRGBの3つ組（FractintとKalles Fraktalerの.mapファイルが使う形式）を、カンマ、スペース、改行で区切って受け付けます。最大256色が保持されます。それより長いリストは、グラデーションの形が保たれるよう最初と最後の色を残して、256色まで均等に間引かれます。
help-options-import-map = その隣の「インポート ▸ .map…」は、FractintまたはKalles Fraktalerの.mapパレットファイルを読み込みます。これらは反復回数で引く参照テーブルでエントリ間のブレンドがなく、それが生むくっきりした色の段差こそが古典的なFractintの見た目の正体です。そのためそのままの形でインポートされ、「ハードバンド」チェックボックスで、代わりに同じ色をなめらかなグラデーションにブレンドすることもできます。Fractint自身のファイルは6ビットの色の値を4倍して保存しているため、白は255ではなく252です。Fractadyneはそのようなファイルをインポートするときにその旨を表示し、値を書かれたとおりに残します。Fractint自身が保存した画像にも同じ252が使われているからです。
help-options-duotone = 「デュオトーン」は値をなめらかな2色の階調（シャドウ → ハイライト）に割り当てます。「二値（集合）」は平坦な2色表示で、集合の内側の点に1色、外側にもう1色を使い、グラデーションはありません。集合の形を見るにはいちばんわかりやすい方法です。
help-options-cycle = 「サイクル」は、反復の範囲でグラデーションを何回繰り返すかを設定します（帯が細かくなるか粗くなるか）。「オフセット」はグラデーション全体を回転させます。
help-options-normalize = 「深い色を正規化」は、ビューに実際に存在する脱出値の範囲にパレットを合わせます。深いビューは数十万回の反復にわたるため、これがないと固定のサイクルが隣り合うピクセルの間でグラデーションを何千回も折り返し、正しい細部が斑点に変わってしまいます。
help-options-log-scale = 「対数カラースケール」は、パレットを脱出値に比例させず、その対数に従って広げます。深くなるほど脱出回数は大きい側に偏るため、線形に合わせると、グラデーションの大部分が境界に張り付いた薄い層に費やされ、残りはほぼ平坦になります。対数スケールはそれを広げ、範囲が広がっていくズーム動画でも色を安定させます。
help-options-animate = 「アニメーション」は、時間とともに色を循環させます。「順方向」「逆方向」「往復」はオフセットをずらし、「ランダム」はなめらかに変化する調和の取れたグラデーションを次々と合成します。「速度」は1秒あたりのサイクル数（または変化の回数）を制御し、「グラデーションをシャッフル」はランダムモードで新しいグラデーションを作ります。
help-options-method = 着色方法：データが色になるしくみ
# The coloring methods, by the names the Coloring panel gives them, each with what it does.
help-method-smooth = スムーズ反復
help-method-smooth-text = 脱出時間による、古典的な連続した帯。
help-method-stripe = ストライプ平均
help-method-stripe-text = 軌道の角度から生まれる、流れるような帯（「ストライプ密度」スライダー）。
help-method-triangle = 三角不等式
help-method-triangle-text = 各ステップが上下限の間のどこに来るかから生まれる、細かい質感。
help-method-trap = 軌道トラップ
help-method-trap-text = 軌道と図形（点/十字/円）との距離。内部にも色が付きます。
help-method-distance = 距離推定
help-method-distance-text = 境界への近さで陰影を付けます。
help-method-decomposition = 分解
help-method-decomposition-text = 最後の脱出角度から生まれるセル模様。
help-options-relief = 3Dレリーフ照明
help-options-relief-text = 境界の傾き（導関数）から表面に陰影を付け、浮き彫りに光を当てたような見た目にします。「光の角度」で方向を、「レリーフ」で強さを設定し（低いほどシャープ）、「光を回転」でアニメーションさせます。正則な族のみ（Mandelbrot / Multibrot）。
help-options-glow = 距離グロー
help-options-glow-text = 境界の近くで密になり、光る糸のようになる明るい等高線の帯です。「グロー」は混ぜる量、「帯の幅」は間隔で、「グローをアニメーション」で流れるように動かします。
help-options-quality = 品質と反復回数
help-options-quality-text = 「反復回数」は、ピクセルを集合の内側とみなすまでの最大ステップ数です。「自動調整」はズームに合わせて自動的に上げます（深い細部ほど多く必要です）。「アンチエイリアス」は、ビューが仕上がると静止画をスーパーサンプリングし（2×～8×）、外側の細かい「ちり」を抑えます。
help-options-other = その他
help-options-other-text = 「ズーム速度」は連続ズームの速さを、FPS上限はフレームレートの上限を設定します。デュアルビューは、Mandelbrot集合とそのJulia集合を並べて表示します（カーソルでJuliaのパラメーターがリアルタイムに決まります）。

## Help ▸ Fractals
## Fractal family names (Mandelbrot, Multibrot, Tricorn, Burning Ship, Celtic, Buffalo,
## Phoenix, Newton) are names: keep them as written, as the formulas. Formula syntax
## (z = z^3 - p1*z + c, sqr, |z|, if / elseif / endif, p1-p5, maxit) is typed as shown.

help-fractals-intro = 特に記載がない限り、どの族もzを0から始め、cをピクセルで決めて数式を反復します（脱出時間）。z = x + iyは複素数です。
help-fractals-mandelbrot = z → z² + c。元祖です。集合は、軌道が有界にとどまるすべてのcからなります。集合は連結で、その境界はハウスドルフ次元が2になるほど入り組んでいます。
help-fractals-multibrot = べき d = 3, 4, 5 の z → z^d + c。べきが高いほどローブが増えます：集合は(d−1)回の回転対称性を持ちます（Multibrot 3は2回、4は3回、5は4回）。
help-fractals-tricorn = z → conj(z)² + c。ここでconj(x + iy) = x − iyです。共役によって反正則になり、3回対称性と特徴的な曲がった「爪」が生まれます。
help-fractals-burning-ship = z → (|x| + i|y|)² + c。2乗する前に絶対値を取ります（実部 = x²−y²+cx、虚部 = 2|xy|+cy）。解析的ではなく、深くズームすると船のような構造が現れます（伝統的に上下逆さまに表示されます）。
help-fractals-celtic = Mandelbrotと同様ですが、実部の絶対値を取ります：実部 = |x²−y²| + cx、虚部 = 2xy + cy。ハートや盾の形をした模様が生まれます。
help-fractals-buffalo = z²の両方の部分の絶対値を取ります：実部 = |x²−y²| + cx、虚部 = |2xy| + cy。CelticとBurning Shipの中間です。
# A heading: the families at powers above 2 (the Fractal menu's "Higher powers").
help-fractals-powers = 高次のべき
# ("Burning Ship  3 4 5") is how the Fractal menu shows such a row: keep it as it is.
help-fractals-powers-text = Multibrot 6、7、8（z^d + c）と、べき3、4、5のBurning Ship、Tricorn、Celtic、Buffalo：同じ折り返しを高いべきで行うものです。Burning Ship dはzの各部分の絶対値を取ってからべき乗し、Tricorn dは先に共役を取ります。Celtic dとBuffalo dはz^dの実部（Buffaloでは虚部も）を折り返します。フラクタルメニューとドロップダウンでは、1つの族を1行に並べて表示します（「Burning Ship  3 4 5」）。どれもべき2の兄弟と同じようにディープズームでき、Multibrot 6、7、8はMultibrot 3～5と同様に、級数近似とミニブロットの検出（M）にも対応しています。Multibrot 8は脱出半径が256ではなく128です。256では次のステップが単精度の範囲を超えてしまうためです。
help-fractals-phoenix = z → z² + c + p·z(n-1)。ここでz(n-1)は1つ前の反復値、pは定数（p = -0.5）です。この記憶項が炎のような糸状の模様を生みます。
help-fractals-newton = z³ − 1 = 0 の根を求めるニュートン法：z → z − (z³−1)/(3z²)。脱出時間ではなく、反復が1の3つの立方根のどれに収束するか（引き込み領域）と、どれだけ速く収束するかでピクセルに色を付けます。入り組んだ引き込み領域の境界がフラクタルです。
help-fractals-julia = Juliaモード
help-fractals-julia-text = Newtonを除くすべての族で、Julia集合に切り替えられます：zを0から始めてc = ピクセルとする代わりに、c（パラメーター）を固定し、zをピクセルから始めます。Julia集合は、その固定したcについて、有界にとどまる出発点と脱出する出発点の境界です。デュアルビューでは、Mandelbrotパネル上でカーソルを動かすと、cがリアルタイムに決まります。
help-fractals-deep = ディープズームへの対応
help-fractals-deep-text = Mandelbrot、Multibrot 3/4/5、Tricornは、超深度（floatexp）の摂動によるディープズームに対応しています。Burning Ship、Celtic、Buffaloは解析的ではない（絶対値を取る）ため、符号を考慮した摂動を使います。これも現在はfloatexpの範囲で動作し、以前のdf32での約1e28×の限界をはるかに超えてディープズームできます（絶対値の折り目付近でまれに出る斑点は、複数参照によるグリッチ補正の実装待ちです）。Phoenixは2項の摂動を使い、これもfloatexpの範囲でディープズームします（SA/BLAなしなので、Mandelbrotより重くなります）。高次のべきも同じようにディープズームし、各べきの摂動はそれぞれの2乗の連鎖に従います。Multibrot 3～8はMandelbrotと同様に級数近似とBLAにも対応し、すべてのべきのTricorn、Burning Ship、Celtic、BuffaloはBLAに対応しています（級数近似なし）。BLAは深いビューの内部を1ステップずつ反復する代わりに、まとめて進めます。directの方式を使うのはNewtonだけで、約1e6×まで鮮明です。カスタム数式は、下のカスタム数式のセクションに別の記載がない限りディープズームできます。
help-fractals-custom = カスタム数式
help-fractals-custom-text = フラクタル ▸ カスタム数式…では、数式の1ステップをFractint形式の式で入力します：z = z^3 - p1*z + c、または複数の文（t = sqr(z)、続いてz = t*t + c）。zは反復値、cはピクセル（JuliaモードではJulia定数）、p1-p5はパラメーターです。|z|はFractintと同じく絶対値の2乗で、一般的な関数が使えます（sqr、abs、conj、real、imag、cabs、flip、recip、ident、exp、log、sqrt、sin、cos、tan、cotan、sinh、cosh、tanh、cotanh、asin、acos、atan、asinh、acosh、atanh、cosxx、そして各部分を丸めるfloor、ceil、trunc、round）。logはFractintや複素解析と同じく自然対数で、多くの教科書がlnと書くものです（常用対数はありません）。ln、cot、coth、Re、Imを使って書いた数式は読み込めません。構文チェックが使うべき関数を示し、ボタンで書き換えられます。lnと入力するとlogが候補に出ます。どのステップも、組み込みのフラクタルと同じく1e28x以降は拡張範囲の演算を使って、摂動でディープズームします。ただし、指数がzやcによって変わるべき（z^cなど）を含む数式は例外です：それはdirectの方式でレンダリングされ、cの単精度での1ステップが1ピクセルにまたがるまで、つまり約1e4xから1e5xまで鮮明です（原点から離れるほど浅くなります）。どちらが適用されるかは数式ダイアログに表示されます。log、sqrt、整数でないべきは主値を取り、負の実軸をまたぐと値が跳びます。摂動はこの跳びを正確に追いますが、参照軌道がまたがない場所で軌道がまたいだピクセルは、それ以降およそ単精度で計算が続けられます。そのため、軌道が分岐截線をまたぐ深いビューでは、そこにノイズが出ます（測定結果：sqrt(z^4 + c)では実軸上で1e5xでもきれい、logを含む数式では1e4xでもノイズあり）。どのステップも完全に反復されるため（級数近似なし）、反復回数の多い深いビューは、組み込みのフラクタルより遅くレンダリングされます。関数を含む数式は境界の近くでカオス的な軌道を描き、そこでの長い軌道は単精度では大まかにしか追えません：2,000回の反復では、sin(z) + cのピクセルの3分の1が正確な軌道と異なり、60回ではほぼ一致します。ビュー、セッション、エクスポートした画像には数式のテキストが保存されます。
help-fractals-custom-sections = Fractintのセクションも使えます。':'より前の文は初期化セクションで、ピクセルごとに1回実行されます。最後の文が比較であれば、それが数式独自の脱出判定になり、成り立つ間は反復を続けます（|z| <= 100 && |z - 1| > 0.000001は、脱出するか1に落ち着く軌道を止めます）。if (...)、elseif (...)、else、endifで文を選びます。また、あるステップで代入される前に読まれた変数は、前のステップの値（初期化セクションの値、または0）を持ちます。比較は実部どうしを比べて1か0を返し、&&と||でそれらを組み合わせます。代入も値になります（a = b = pixel）。pixel、p1-p5、pi、eには代入でき、代入されるまでは本来の値を保ちます。maxitは反復回数で、ismandは1です（数式のMandelbrot形式）。これらのいずれか、または丸めを含む数式は、ディープズームなしのdirectの方式でレンダリングされます。数式ライブラリ > インポートは、Fractintの.frmファイルも読み込みます。読み込めた各エントリは数式になり、その行には読み込みで変更された点が示されます：fn1-fn4はFractintの既定値（sin、sqr、sinh、cosh）になります。ここではcがピクセルなので、Fractintの変数cはc_に名前が変わります。Fractintと同じく、何も代入されない名前は0です。Fractintと同じく、名前の中の空白は取り除かれます（end ifはendifになります）。インポートのメッセージは読み込めなかったエントリの数と、最も多い理由を示します：画面とビューの変数（whitesq、scrnpix、scrnmax、center、magxmag、rotskew）、lastsqr、乱数には対応していません。コレクションにあるFractintの古典（Magnet、Nova、Spider、Manowarなど）は、これらのセクションを使って書かれています。
help-fractals-custom-editor = 数式の欄では、括弧は入れ子の深さごとに色分けされます。カーソルの隣の対は強調表示され、相手のない括弧は赤で表示されます（コメント内の括弧はパーサーが無視するので数えません）。名前は入力中に補完されます：2文字入力すると、その文字で始まる関数、名前、自分の変数がリストに表示されます。UpとDownで選び、TabまたはEnterで確定し（関数には開き括弧が付きます）、Escでリストを閉じます。名前を最後まで入力したあとのEnterは、これまでどおり改行になります。
help-fractals-custom-textbook = 「教科書」（数式の上の切り替え）は、数式を教科書のように組版して表示し、その場で編集できるようにします。テキストと同じように入力します：/ は直前の項を分数にし、^ は指数にします（指数の末尾で+、-、=を入力すると指数から抜けます）。( は括弧、または関数名のあとなら関数呼び出しになり、| は縦棒、括弧内のカンマは複素数の定数になります。矢印キーで数式内を移動し、UpとDownで分子と分母の間を移動し、Tabで次の空欄へ移動します。分数や括弧の先頭でBackspaceを押すとそれが選択され、もう一度押すと中身を残してそれを取り除きます。コピー、貼り付け、元に戻すはテキスト欄と同じように動作し、名前もテキスト欄と同じように補完されます。キーパッドは組版された数式に入力します（□² はキャレットの前にあるものを2乗し、関数キーは選択範囲を囲みます）。すべての編集はテキストに書き戻されるので、2つの表示は常に同じ数式を保持します。編集していない行は、空白とコメントがそのまま残ります。表記によって計算内容が変わることはありません：|z| は絶対値の2乗で |z|² と表示され、cabs(z) は |z| と表示され、exp(z) は exp(z) のままです。読み込めない行は赤で表示されます。クリックするとテキストとして編集できます。「LaTeX」は数式を組版されたとおりにLaTeXのソースとしてコピーします。また「教科書」がオンの間は、数式ライブラリも数式を組版して表示します。
help-fractals-custom-numbers = 数式中の数値（0.1、1/3、pi、パラメーター）は倍精度の値（約16桁）として読み取られ、深い参照軌道はまさにその値で反復されます。そのため、紙の上では等しい2つの数式が、深いところでは異なることがあります：0.1*zは0.1に最も近い倍精度の値を掛けますが、z/10は正確な10で割るため、2つの画像は約1e15xを超えると分かれていきます（測定結果：1e15xでピクセルの5%、1e17xで3分の1、1e20xではすべて）。どちらの画像も、それぞれの数式としては正しいものです。共有したいほうの数式を書いてください。
help-fractals-custom-library = ダイアログの「ライブラリに保存」は、数式とそのパラメーターを名前を付けて保存します（ライブラリにある名前で保存すると、そのエントリが更新されます）。フラクタル ▸ 数式ライブラリ…ではそれらが一覧表示され、適用、編集、エクスポート、削除ができます。「インポート」は数式ファイル内の数式を、何も置き換えずに追加します：ライブラリにすでにあるものは飛ばされ、名前が使われている新しい数式は「名前 (2)」になります。ライブラリは「ファイルとデータ」（バージョン情報）に示されるフォルダー内のformulas.tomlで、それ自体が数式ファイルです。ツアーのキーフレームは数式をformula = "..."とformula_params = [[re, im], ...]として持ち、ツール > 現在のビューからツアーを作成は、カスタムのビューに対してこれらを書き出します。
help-fractals-custom-collection = Fractadyneには、Fractadyneのために書かれた45個の数式のコレクションが付属しています：高次のべき、Burning ShipとCelticの族の折り返しの変種、混合と2段階のハイブリッド、別の写像を通して見たMandelbrot集合（反転、対数極座標、正弦を通したもの）、超越関数や有理関数の数式、そしてJulia集合です。これらはダイアログの「例」と、数式ライブラリの「コレクション」タブにあり、そこで「マイ数式にコピー」を押すと自分の数式に追加されます。それぞれに開始ビューがあり、「適用」でそこへ移動します：ホームビューで開いた数式は、絵のある場所から外れていることがよくあるためです。表示中に保存した数式も同じように現在のビューを保持し、「適用」でそこへ戻ります。

## Help ▸ How it works

help-how-escape = 脱出時間となめらかな色
help-how-escape-text = 各ピクセルは、大きさが脱出半径を超えるか反復の上限に達するまで、数式を反復します。生のステップ数だけでは、くっきりした帯ができます。最終的な大きさから求めた小数部を加えると、帯のない連続した色になります。
help-how-precision = 任意精度の位置
help-how-precision-text = 通常の64ビットの数値は、1e15×付近のズームで桁が足りなくなります。Fractadyneはビューの中心を任意精度で保持し、ズームに応じて桁数を増やすので、位置が劣化することはありません。ピクセルのスケールも同様に拡張された指数で保持するので、64ビットの約1e308×の限界で止まることもありません。深度を制限するのは忍耐力だけで、固定の数値範囲ではありません。
help-how-perturbation = 摂動
help-how-perturbation-text = すべてのピクセルを高精度で反復すると、遅すぎて使えません。代わりに、1つの参照ピクセルをCPU上で高精度で反復し（「参照軌道」）、ほかのすべてのピクセルはGPU上で、そこからのごく小さな差δとして高速な低精度で計算します：δz → 2·Z·δz + δz² + δc。
help-how-floatexp = 超深度（floatexp）
help-how-floatexp-text = 約1e28×を超えると、そのごく小さな差でさえ32ビットの範囲でアンダーフローするため、仮数部と別の整数の指数として保存し（「floatexp」）、32ビットの深度の壁を取り払います。エンジンは自動的に切り替わります：浅いところでは直接計算、深いところでは摂動、最も深いところではfloatexpです。こうして深度を制限するのは固定の上限ではなく、座標の精度と反復予算になります。39か所の参照コーパスでFraktaler-3と照合しており、最も深いものは5.63e18003×です（極端でない38行が通常のゲートで、6.13e1105×まで）。また、1e1000000×までの内部精度の自己整合性テスト群があります。同梱のツアーは約1.3e420×に達します。
help-how-reference = 参照点の選択とリベース
help-how-reference-text = 参照点は、その軌道ができるだけ長くビュー内にとどまるように選ばれます（高精度でスコアを付けます）。差が大きくなりすぎると、精度を保つために参照軌道へ「リベース」されます。
help-how-series = 級数近似
help-how-series-text = ディープズームでは、最初のほうの反復はδを参照軌道からほとんど動かさないため、まとめて計算できます：δzをδcの短い多項式（3次）で近似し、その係数を参照軌道に沿って反復します。レンダラーは、その多項式がビュー全体でまだ正確な最後の反復まで先に飛ばし、そこから通常どおり反復します。飛ばしたステップの分だけ時間が節約され、画像は何も変わりません（コントロール ▸ 詳細設定で切り替え）。対象はMandelbrotとMultibrot 3/4/5の最も深い範囲で、ストライプ/三角不等式/軌道トラップ/分解の着色では使われません（すべての反復が必要なため）。BLAが有効な場合は、BLAがこの初期のスキップをすでに含んでいます。
help-how-distance = 距離推定と照明
help-how-distance-text = 導関数dz/dcを追跡すると、各ピクセルの境界までの距離が得られます。これが3Dレリーフ照明、距離グロー、「距離推定」の着色方法を支えており、どれもどのズーム深度でも有効です。
help-how-safety = アンチエイリアスと安全性
help-how-safety-text = 静止画は、ビューが仕上がるとスーパーサンプリングされます（軸ごとに2～8×）。極端な設定では、作業予算が解像度を下げる（反復回数は決して下げません）ことで、1回のGPU描画をドライバーのウォッチドッグの制限内に収めます。そのため深いビューは、空白になることなく細部を保ちます。

## Help ▸ Command line (the flags themselves stay English)

# The section's introduction. The flags listed below it are the command line itself, English in every language.
help-cli-intro = Fractadyneは、レンダリング、動画/ツアーのエクスポート、自動化、基準画像との照合、ベンチマークのために、ヘッドレス（ウィンドウなし）で動作します。フラグを付けずに起動すると、対話的なGUIが起動します。

## Help ▸ Shortcuts

# What each mouse gesture and key does (the gestures are the Navigation section's).
help-shortcuts-pan = パン
# A gesture: turning the mouse wheel.
help-key-wheel = ホイール
help-shortcuts-wheel = カーソル位置でズーム
help-shortcuts-box-zoom = ボックスズーム（長方形をドラッグしてそこへズーム）
# $icon: the click-to-zoom toolbar button's symbol. "tool on": while that button is on.
help-key-click-tool = クリック（{ $icon }ツールがオン）
help-shortcuts-click = その点へダイブ。右クリックで戻る
# A heading: the keyboard shortcuts.
help-shortcuts-keyboard = キーボード
help-shortcuts-space = 連続ズームイン/アウト
help-shortcuts-undo = ビューを元に戻す
help-shortcuts-redo = ビューをやり直す
help-shortcuts-minibrot = ミニブロットを探してズーム（ニュートン・ラフソン法）
help-shortcuts-autopilot = 自動ズームのオートパイロット（細部へ向かってダイブ。何か入力すると停止）
# off / strip / panel / full: the panel's sizes, as View ▸ Live render panel names them.
help-shortcuts-live-panel = ライブレンダリングパネル：オフ / ストリップ / パネル / フル（計器にポインターを合わせると意味を表示。×で閉じる）
help-shortcuts-esc = オートパイロットや再生中のツアーを停止、または全画面表示を終了
help-shortcuts-snapshot = 前回のエクスポート先フォルダーへスナップショット：表示どおりのビューの画面キャプチャ、またはエクスポート設定でのフルレンダリング。最初に押したときにどちらにするかを確認し、ファイル → 設定 → スナップショットで変更できます。
help-shortcuts-help = このヘルプを開く
# A heading: the shortcuts of the Game of Life, with the menu path to it.
help-shortcuts-life = ライフゲーム（フラクタル → オートマトン → ライフゲーム）
help-shortcuts-life-run = 盤面を実行/一時停止
help-shortcuts-life-step = 1世代進める / ストライド分進める（ライフゲームのパネルで設定）
help-shortcuts-life-draw = 描画ツールのオン/オフ（ツールバーの鉛筆）
# A gesture. "Draw on": while the Life draw tool is on.
help-key-click-drag-draw = クリック / ドラッグ（描画オン）
help-shortcuts-life-paint = セルの反転 / ペイント：死んだセルから始めたストロークは描画し、生きたセルから始めたストロークは消去します。実行中の盤面は、描いている間は一時停止し、離すと再開します。
# $key: a keyboard shortcut (Ctrl+Z). "Draw on": while the Life draw tool is on.
help-key-when-drawing = { $key }（描画オン）
help-shortcuts-life-undo = 直前の編集、塗りつぶし、クリア、ルール変更を元に戻す

## Help ▸ Recommended hardware

help-hardware-intro = Fractadyneは、処理をGPU（ピクセルごとの反復と着色）とCPU（ディープズームの土台となる任意精度の参照軌道）に分担させます。どちらも重要ですが、役割が異なります。
help-hardware-gpu = GPU（最も重要）
help-hardware-gpu-text = すべてのピクセルはGPUで反復され着色されるので、GPUの速度がフレームレートとエクスポート時間を決めます。32ビット（単精度）の処理能力が高い単体GPUなら、最もなめらかにディープズームできます。内蔵GPUでも問題なく動作します。遅くなるだけで、フレームごとの作業予算に早く達します（その時点でレンダラーが削るのは細部ではなく解像度です）。最新のグラフィックスAPIが必要です：wgpuを通したVulkan、Direct3D 12、Metal、またはOpenGL。
# The left column of a requirements table.
help-hardware-minimum = 最小
help-hardware-gpu-minimum = Vulkan / DX12 / Metal / GL 3.3に対応した任意のGPU。
help-hardware-recommended = 推奨
help-hardware-gpu-recommended = 単体GPU（過去約5年以内のもの）。VRAM 2 GB以上。
help-hardware-big-exports = 大きなエクスポート向け
help-hardware-gpu-big-exports = VRAM 4 GB以上（タイル分割するので、不足しても緩やかに性能が落ちるだけです）。
# A heading: the processor.
help-hardware-cpu = CPU
help-hardware-cpu-text = 高精度の参照軌道（とBLAテーブル）はCPUで計算され、大部分がシングルスレッドなので、コアあたりの性能が最も重要です。移動したあと深いビューがどれだけ速く解像するかを左右します。追加のコアは主にバックグラウンドでのエクスポートのエンコードに役立ちます。64ビットのCPUが必要です。
help-hardware-cpu-recommended = シングルコア性能の高い、最新の64ビットCPU。
help-hardware-memory = メモリ
help-hardware-memory-text = 深い参照軌道は深度とともに大きくなります（高精度での多数の反復）。巨大なエクスポートではタイルをRAMに保持します。普段の探索には8 GBで十分で、超深度や非常に大きなエクスポートには16 GB以上が適しています。
help-hardware-checking = ハードウェアのチェック
help-hardware-diagnostics = ヘルプ → 「診断…」では、開発者が行うのと同じテストをお使いのマシンで実行します。「セルフテストを実行」は、数学とレンダリングを既知の正しい結果と照合し、約15秒かかります。「UIテストを実行」は、インターフェースとライブビューを複数のズーム深度で巡回し、スクリーンショットを撮ります。「GPU演算チェックを実行」は、マシンで使えるすべてのグラフィックスバックエンドでシェーダーのdouble-float演算を検証し、数秒で、ウィンドウを開かずに終わります。各テストは別のプロセスとして実行されるので、グラフィックスドライバーを不安定にするテストがあっても、落ちるのはテストだけでセッションは無事です。
help-hardware-gpu-check = GPU演算チェックには合格も不合格もありません。お使いのGPUとドライバーがどのように計算するかを記録するもので、一部のハードウェア（これまでに試したNVIDIAのカードすべて）では、シェーダーコンパイラーが拡張精度の演算の一部を削除していることを報告します。これは私たちがベンダーをまたいで調べようとしている本物の知見であり、お使いのマシンの故障ではありません。結果がどうであれ、レポートに添付していただくと本当に助かります。
help-hardware-report = 何かおかしいと感じたら、セルフテストを実行して「問題の報告に添付…」を使ってください：レポートにお使いのハードウェアで機械的に検証された結果が含まれ、説明だけよりもはるかに役立ちます。1つ知っておくべき注意点があります：参照画像は特定の1枚のグラフィックスカードで作られており、GPUが異なれば正当にわずかに異なるピクセルを計算します。そのため、ほかのハードウェアでは画像の比較をより広い許容範囲で行い、そこでの小さな差は故障ではなく想定どおりのものです。
help-hardware-notes = 注意事項
help-hardware-note-watchdog = 高性能なGPUなら、極端な設定でもウォッチドッグの作業予算が働かずに済みます。
help-hardware-note-binary = ビルドは単一のバイナリで自己完結しており、ランタイムのインストールは不要です。
help-hardware-note-benchmark = 組み込みのベンチマーク（ツール -> ベンチマーク）を実行して、マシンの性能を測ってください。

## Help ▸ Acknowledgments & citations
## Names of people, programs, papers, books and websites stay as written, and so do the
## titles in quotation marks (the works' own titles); the prose around them translates.

help-ack-title = 謝辞と引用
# The dedication, in italics at the top of the section.
help-ack-dedication = Stone Soup Group（1988年からFractintを支えてきたボランティアたち）に捧げます。
help-ack-dedication-text = 彼らが惜しみなく共有したコードと、開かれた共同のフラクタル探索の精神こそが、私たちの多くがこの芸術に夢中になった理由です。Fractadyneはその伝統の上に作られています。
help-ack-intro = Fractadyneは、フラクタルのコミュニティによる数十年の成果の上に成り立っています。土台としている技法と先行研究（下記の出典と照合済み）：
help-ack-deep-zoom = ディープズームのアルゴリズム
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-martin-title = 摂動と級数近似 — { $who }
help-ack-martin = 参照軌道と低精度の差分による手法（δz → 2Z·δz + δz² + δc）と、最初のほうの反復を飛ばすn次の級数。SuperFractalThingと、その解説「SuperFractalThing Maths」（sft_maths.pdf、2013年）で紹介されました。
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bla-title = 二変数線形近似（BLA）とリベース — { $who }
help-ack-bla = 線形写像を合成して一続きの反復を飛ばす手法と、グリッチを避けるために単一の参照軌道へリベースする手法。fractalforums.orgで開発されました（2021年）。
# A link's label. $who: the linked page's author.
help-ack-bla-link = BLAの解説 — { $who }
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-glitch-title = グリッチ検出の基準 — { $who }
help-ack-glitch = ピクセルに別の参照軌道が必要かどうかを判定する |Z + z| << |Z| のテスト。グリッチのない摂動の基礎です（fractalforums）。
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-diffabs-title = 非解析的（Burning Ship）な摂動 — { $who }
help-ack-diffabs = 絶対値に基づく族（Burning Ship / Celtic / Buffalo）で摂動を機能させる、絶対値の「diffabs」による場合分け（fractalforums）。
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mathr-title = 参照実装と照合 — { $who }
help-ack-mathr = Fraktaler-3とKalles Fraktaler 2+（元はKarl Runmo作）。その解説はこのエンジンの指針となり、そのEXR出力とFractadyneのレンダラーを照合しています。
# A heading: credits for the coloring methods.
help-ack-coloring = 着色
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-harkonen-title = スムーズな反復回数とストライプ平均 — { $who }
help-ack-harkonen = 連続的な（正規化された）脱出時間の着色とストライプ平均の手法。「On Smooth Fractal Coloring Techniques」（修士論文、2007年）によるもので、この論文は三角不等式 / 分岐平均の着色も分析しています。
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mitchell-title = 三角不等式平均 — { $who }
help-ack-mitchell = 各 z² + c が三角不等式の上下限の中のどこに来るかを平均し、炎のようななめらかな外側の質感を生みます。
# A link's label: the person's tutorial pages. $who: the person's name.
help-ack-mitchell-link = { $who } — チュートリアル
help-ack-distance = 距離推定の着色と照明には、古典的なdz/dcによる境界までの距離の手法を使っています（Peitgen & Saupe「The Science of Fractal Images」、1988年を参照）。
help-ack-lsystems-life = Lシステムとライフゲーム
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-lindenmayer-title = Lシステム — { $who }
help-ack-lindenmayer = 並列の文字列書き換え。植物や藻類の成長をモデル化するために1968年に導入されました（「Mathematical models for cellular interactions in development」、Journal of Theoretical Biology）。
help-ack-abop = この本（1990年）のタートル解釈と、確率的・文脈依存・パラメトリックなシステムの表記に、Lシステムの読み込み機能が従っています。ABOPと記された組み込みのシステムは、この本の図から入力したものです。
# A link's label: the book can be read free on that website. $site: the website's name.
help-ack-abop-link = 本書は{ $site }で無料で読めます
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bourke-title = Lシステムのコレクション — { $who }
help-ack-bourke = 組み込みのシステムの多くは彼のLシステムのページに由来し、その中にはChris WallaceによるSierpinski曲線の規則やDietmar Saupeの茂みも含まれます。そのまま開けるFractintの.lファイルは、Stone Soup Groupの形式です。
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-conway-title = ライフゲーム — { $who }
help-ack-conway = セル・オートマトンB3/S23（1970年）。Scientific AmericanのMartin Gardnerのコラム「Mathematical Games」（1970年10月号）を通じて世界に広まりました。
# A credit's heading: the website, then who runs it. $site: the website's name.
help-ack-lifewiki-title = { $site } — ConwayLife.comのコミュニティ
help-ack-lifewiki = アプリが読み込むルールの表記（B/S、von Neumann、Generations、非トータリスティックなルールのためのAlan Henselの文字表記、MAP）、Gollyと共通のRLEパターン形式、そして組み込みのルールとパターン（その発見者も記録されています）についての参考資料です。
help-ack-foundations = 基盤とツール
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mandelbrot-title = Mandelbrot集合 — { $who }
help-ack-mandelbrot = すべての中心にある対象。
help-ack-built-with = Rust、egui/eframe（emilk）、wgpu（gfx-rs）で作られています。任意精度はastro-float、ダブルワードの浮動小数点数はtwofloat、ほかにPNG/OpenEXR、rfd、serdeを使っています。作者とメンテナーの皆さんに感謝します。
help-ack-errors = 帰属表示の誤りはすべて作者の責任です。訂正はプロジェクトのissueトラッカーで歓迎します。

## Help ▸ Third-party licenses

help-licenses-title = サードパーティのライセンス
# $licenses: Fractadyne's license expression (MIT OR Apache-2.0), as written. $url: the web address of a crate's source. THIRD-PARTY-NOTICES.md is a file name, `option-ext` a crate's name, MPL-2.0 a license's.
help-licenses-intro = Fractadyneは、下記のオープンソースのクレートと同梱のフォントを含む、静的リンクされたバイナリとして配布されています。ライセンスの全文（ダウンロードに含まれるTHIRD-PARTY-NOTICES.mdにもあります）を以下に示します。Fractadyne自身のコードは{ $licenses }のデュアルライセンスです。`option-ext`はMPL-2.0で、その未改変のソースは{ $url }にあります。
# A button, after a copy icon: copies every notice to the clipboard.
help-licenses-copy-all = すべての通知をコピー

## Help ▸ About

help-about-tagline = Rustとwgpuで作られた、ネイティブのフラクタル探索ソフトウェア。
# The arithmetic deep zoom has used. $ran: the libraries that have run so far, as names (or a note that none has yet); $built: those this build contains, with versions.
help-about-arithmetic = ディープズームの演算：{ $ran }（このビルドに含まれるもの：{ $built }）。
help-about-files = ファイルとデータ
# Followed by the folder's path.
help-about-files-text = セッション（現在のビュー、着色、環境設定）、ブックマークとそのサムネイル、保存したグラデーション、保存した数式は、このユーザーごとのフォルダーに保存され、次回の起動時に読み込まれます：
# Tooltip of the copy button beside the folder's path.
help-about-copy-path = パスをコピー
# $command: a command typed in a terminal (it stays English).
help-about-session = セッションファイルにはバージョンがあります。完全には読み込めない、より新しいFractadyneで書かれたものであれば、アプリは可能な範囲で読み込み、警告を表示します。完全に最初からやり直すには、ファイル ▸ アプリケーションの状態をリセット…（先に確認があります）を使うか、ターミナルで`{ $command }`を実行してください。
help-about-license = ライセンス
# $licenses: the license expression (MIT OR Apache-2.0), which stays as written.
help-about-license-text = { $licenses }：どちらかのライセンスを選んで使用できます。
# A link to the source code, followed by ↗.
help-about-source = GitHubのソースコード

## The live render panel ("blinkenlights": View ▸ Live render panel, key B)
## A front panel drawn over the view like a piece of 1970s test equipment. Its legends are engraved
## in capitals: write them in capitals here (the panel shows them exactly as written, it does not
## uppercase them). Numbers, units and readouts on its tubes and dials are not translated.

# Tooltip of the × button in the panel's top-right corner. The code adds the key after it: (B).
live-close-hover = ライブレンダリングパネルを閉じる
# The legend engraved along the top plate, left of the maker's badge.
live-plate = ライブレンダリング · フロントパネル

## Live render panel: instrument titles
## Engraved above each instrument, and repeated as the title of its help card.

# Above the depth readout, whose tubes show the magnification as a power of two ("2^" stays).
live-title-depth = 深度 2^
live-title-iterations = 反復回数
# Above the frame-time readout (milliseconds).
live-title-frame-ms = フレーム ms
# Above the date-and-time readout.
live-title-clock = 日付 · 時刻
# The walk dial: a radar screen showing a slow refresh rendered pass after pass, band after band.
live-title-walk = ウォーク · 帯
# A meter of how far the picture on screen lags the view.
live-title-age = 画像の古さ
# What part of the picture this frame's GPU pass covered.
live-title-tiles = タイル
# A scope of the GPU's work per frame, in milliseconds.
live-title-scope = GPUパス · ms
# The depth dial: depth, the reference orbits being built, and the arithmetic in use.
live-title-dial = 深度 · 参照 · 演算
# Iteration steps per second.
live-title-steps = ステップ / 秒
# The two GPUs (the window's and a second, worker GPU).
live-title-engines = 2基のエンジン
live-title-memory = メモリ
live-title-cores = コア · スレッド
live-title-pixels = ピクセルストリーム
# Help card titles only: the three GPU timer lamps (T0, T1, T2 stay as they are) and the row of lamps.
live-title-timers = GPUタイマー T0–T2
live-title-bank = ランプ列

## Live render panel: the light bank's lamps
## Engraved under each lamp, and the title of its help card. A cell is narrow (about eight capital
## letters): a longer label breaks at a space onto two lines, and is drawn smaller if it still does
## not fit.

live-lamp-plan = 計画
live-lamp-dispatch = 送出
live-lamp-live = ライブ
live-lamp-held = 保持
# The last picture reprojected (stretched) to fit the moving view.
live-lamp-reproj = 再投影
live-lamp-reading = 読取
# A pass of a pinned walk.
live-lamp-pin-pass = ピンパス
live-lamp-split = 分割
live-lamp-adopt = 採用
live-lamp-abandon = 破棄
# "REF": a reference orbit. Built for the view as it is (LIVE), ahead of it (AHEAD), or for a tour's next hold (HOLD).
live-lamp-ref-live = 参照 現在
live-lamp-ref-ahead = 参照 先行
live-lamp-ref-hold = 参照 停留
# The series approximation.
live-lamp-series = 級数
# The second (worker) GPU.
live-lamp-worker = ワーカー
live-lamp-tile = タイル
# Supersampling (anti-aliasing); usually kept as SSAA.
live-lamp-ssaa = SSAA
live-lamp-palette = パレット
# The arithmetic in use (direct, df32, floatexp).
live-lamp-mode = モード

## Live render panel: the kinds of GPU pass
## In the GPU scope's legend (MOVING, PIN, WORKER, SETTLE, SSAA). SETTLE and SSAA also name a still
## view's walk on the walk dial.

# A moving frame's pass.
live-kind-moving = 移動中
# A pass of a pinned walk.
live-kind-pin = ピン
# The worker GPU's passes.
live-kind-worker = ワーカー
# A still view's settle tile or walk pass.
live-kind-settle = 仕上げ
# A supersampling sample.
live-kind-ssaa = SSAA

## Live render panel: legends and readouts on the instruments

# On the walk dial during a split refresh. $done: passes done; $passes: passes in all.
live-walk-split = 分割 { $done }/{ $passes }
# On the picture-age meter's face, under its scale (0 to 2 octaves).
live-age-scale = オクターブ遅れ
# The depth dial's band for direct (plain floating-point) arithmetic. The other bands, DF32 and FLOATEXP, are names and stay.
live-dial-direct = 直接
# Under the two GPU gauges: GPU 0 draws the window; GPU 1 is the worker, or there is none.
live-engines-gpu-window = GPU 0 · ウィンドウ
live-engines-gpu-worker = GPU 1 · ワーカー
live-engines-no-worker = ワーカーなし
# The share of displayed frames each GPU made. $window and $worker: percentages, 0–100.
live-engines-on-screen = 表示比 { $window } : { $worker }
# The same readout before any frame was shown.
live-engines-on-screen-none = 表示比 —
# Under the CPU core bars. $count: the machine's cores.
live-cores-count = { $count }コア
# Under the threads scope. $count: the app's busy threads.
live-cores-threads = スレッド { $count }

## Live render panel: the memory tanks' legend
## What the GPU memory holds, by kind. Very narrow: about ten letters.

# G-buffers (the picture's working textures).
live-memory-gbuf = Gバッファ
# The held picture and the supersampling accumulator.
live-memory-hold = 保持+SSAA
# The reference orbit and its BLA (bilinear approximation) table.
live-memory-orbit = 軌道+BLA
# The worker GPU's copy of the reference.
live-memory-worker = ワーカー参照

## Live render panel: the roles on the threads scope
## Tiny labels on a small scope: at most four letters each.

# The reference builders.
live-cores-role-ref = 参照
# The scoring pool.
live-cores-role-pool = 評価
# The thread driving the worker GPU.
live-cores-role-worker = ワーカ
# Probes: short side renders.
live-cores-role-probe = 探査
# Image and tour exports.
live-cores-role-export = 出力

## Live render panel: the tiles instrument's captions
## The line under the picture's plane, then the tile size (left) and the tiles a frame (right).

# A walk's pass. $lo and $hi: the iteration band it rendered (numbers).
live-tiles-pin-band = ピンパス · 帯 { $lo }–{ $hi }
live-tiles-settle-band = 仕上げパス · 帯 { $lo }–{ $hi }
live-tiles-moving-whole = 移動フレーム · 画像全体
live-tiles-settled-whole = 仕上げ済みフレーム · 画像全体
# $pass: this pass; $passes: the passes the refresh is split into.
live-tiles-split-pass = 分割更新 · パス { $pass }/{ $passes }
# $grid: the grid's columns × rows, e.g. 12×8.
live-tiles-grid-done = 仕上げグリッド { $grid } · 完了
# $grid: the grid's columns × rows; $done and $total: tiles done, and in all.
live-tiles-grid-progress = 仕上げグリッド { $grid } · { $done }/{ $total } 完了
# The same, while a tile too costly for one frame goes out in strips.
live-tiles-grid-strips = 仕上げグリッド { $grid } · { $done }/{ $total } 完了 · 短冊分割
# $samples: supersampling samples folded so far; $lo and $hi: the iteration band the next one is walking.
live-tiles-ssaa-band = SSAA · { $samples } 完了 · 帯 { $lo }–{ $hi }
# $sample: the supersampling sample.
live-tiles-ssaa-whole = SSAA サンプル { $sample } · フレーム全体
# No GPU pass this frame, and the panel has not seen a frame yet.
live-tiles-idle-starting = 起動中
# No GPU pass this frame: the last complete picture is held on screen.
live-tiles-idle-held = 保持
# No GPU pass this frame: the last picture is stretched to fit the view.
live-tiles-idle-reprojected = 再投影
# No GPU pass this frame: the picture on screen is current.
live-tiles-idle-live = ライブ
# No GPU pass this frame, and nothing to show.
live-tiles-idle-none = 待機
# The tile size: $res, a whole picture's resolution (2400×1300).
live-tiles-tile-res = タイル { $res }
# The tile size: $px, a tile's side in pixels.
live-tiles-tile-px = タイル { $px } px
live-tiles-no-pass = パスなし
# Tiles a frame. $count: how many (0 when none).
live-tiles-rate-count = { $count }/フレーム
live-tiles-rate-one = 1タイル/フレーム
# $share: the part of one tile sent each frame, e.g. 0.33.
live-tiles-rate-part = { $share }タイル/フレーム
live-tiles-rate-sample = 1タイル/サンプル
# In the plane's corner during a walk: the share of pixels still iterating. $percent: a percentage, e.g. 54 or 1.5.
live-tiles-running = 反復中 { $percent }%

## Live render panel: help cards
## Shown when the pointer rests on an instrument or a lamp. Words in capitals (SETTLE, SSAA, ABANDON,
## HELD, REPROJ, ADOPT, REF AHEAD, ON SCREEN) are the panel's own legends: use the same words as
## their messages above.

# The heading of a card's list of what healthy looks like.
live-card-healthy = 正常
# The heading of a card's list of what signals trouble.
live-card-trouble = 異常

live-card-walk-what = 1フレームでは重すぎる深い更新は、ピン留めしてウォークします：パスを重ねてレンダリングし、パスごとにピクセルが1オクターブ先まで反復できるようにします（256、512、1,024 … 要求値まで）。外周はその回数を対数目盛りで示し、上端から始まります。ビームはウォークの進行、各円弧は1回のパスで、20 msの目標に対するGPU時間で色分けされます。白い輝点は最後のパスの計測値を待っているところで、閃光は新しい画像が画面に表示された瞬間です。
live-card-walk-note = 1回のパスでは重すぎる静止ビューも同じようにウォークし、「仕上げ」と表示されます。スーパーサンプリングの各サンプルも再びウォークし、「SSAA」と表示されます：これらのパスはそれぞれ最大400 msかかってもよく、円弧はそれを基準に色分けされます。分割更新では代わりにパスごとに1つの輪を描きます：すべての帯を、タイルの一部ずつ。
live-card-walk-good-sweep = 数フレームで一周し、閃光で終わる掃引。
live-card-walk-good-arcs = 円弧の大半が緑と琥珀色。
live-card-walk-good-still = 深い静止ビューでは：サンプルごとにゆっくりした掃引で、円弧は緑から琥珀色。
live-card-walk-bad-red = 赤い円弧：予算を超えたパスで、カクつきとして感じられます。
live-card-walk-bad-restart = 途中で止まってやり直す掃引：ウォークが破棄され、画面上の画像は古いままです。
live-card-walk-bad-hollow = 中空の円弧：タイミングが返ってこなかったパスで、次のパスは手探りで大きさが決められます。

live-card-age-what = 画面上の画像がビューからどれだけ遅れているかを、ズームのオクターブ数（1オクターブは2×）で示します。更新のレンダリング中は古い画像が引き伸ばされて表示され、古くなっていきます。更新が届くと、古さは下がります。下の帯は直近数秒間の推移です。
live-card-age-good-sawtooth = 約0.3から0.8の間ののこぎり波で、針が0.5の目盛りの周りを振れる。
live-card-age-good-still = 止まると0に近くなる。
live-card-age-bad-red = 針が1.0を超えた赤い範囲にとどまる：画像は2×以上古く、ズーム中にぼやけたりブロック状に見えたりします。
live-card-age-bad-ramp = 下がらない上り坂：更新が完了していません（「破棄」を確認してください）。

live-card-tiles-what = このフレームのGPUパスが画像をどう覆っているかを示します。移動中のフレームとウォークのパスは画像全体を覆い、キャプションにはウォークのパスの反復帯が示されます。ウォークのパスは、まだ脱出していないピクセルだけを反復します：平面はその割合まで塗られ（対数目盛り、下限は0.01%）、隅にその値が表示されるので、ウォークが深く進むにつれて減っていきます。1フレームには少し重すぎる更新は、最大8回のパスに分割され、各パスは16 pxのタイルを交互に組み合わせたものです。1回のパスでは重すぎる静止ビューの仕上げは、中心から外へ1フレームに1枚ずつ進むタイルのグリッドになり、それでも重すぎるタイルは短冊に分けて送られます。下に表示されるのは、タイルのサイズとフレームあたりのタイル数です。
live-card-tiles-good-whole = ズーム中は画像全体のパス。分割更新は数パス。
live-card-tiles-good-grid = 止まったとき：数十枚のタイルのグリッドが1～2秒で埋まり、続いてSSAA。
live-card-tiles-bad-small = 数百枚の小さなタイルのグリッド：各パスの予算が小さく、鮮明になるまで長くかかります。
live-card-tiles-bad-strips = 多くのタイルが短冊に分けて送られる：タイルが、グリッドの大きさを決めたときの想定より遅く動いています。
live-card-tiles-bad-splits = 分割が8パスに張り付いている：更新がぎりぎり収まっている状態で、少し深くなると遅いウォークに戻ります。
live-card-tiles-bad-dark = 移動中に長い間暗い：パスがまったくありません。

live-card-scope-what = 1フレームにつき1列で、最新が右端です：そのフレームで実行したGPUの処理を種類ごとに積み上げ（移動フレームは緑、ピンパスはコストに応じて緑から赤、仕上げタイルや静止ビューのウォークのパスはクリーム色、スーパーサンプリングのサンプルはラベンダー色、ワーカーGPUのフレームとサンプルは青）、ミリ秒で示します。破線は16.7 ms（60 fps）、33、50の位置です。中空の印は、タイミングが返ってこなかったパスです。
live-card-scope-good-under = 移動中は列が16.7の線より下。
live-card-scope-good-pin = ピンパスは緑から琥珀色。
live-card-scope-good-samples = 静止ビューのスーパーサンプリングのサンプルは長くかかることがある：何も動いていないためです。
live-card-scope-bad-over = 33や50 msを超える列：目に見えるカクつき。
live-card-scope-bad-seconds = 秒単位のパス：GPUのウォッチドッグがデバイスをリセットするおそれがあります。
live-card-scope-bad-hollow = 中空の印が多い：計測値なしで処理の大きさが決められています。

live-card-timers-what = GPUの3つのタイムスタンプスロットです。パスの計時が始まるとスロットが点灯し、2～3フレーム後に計測値が返ってくると暗くなります。T0は毎フレームのパスを、T1とT2は追加のピンパスを計時します。
live-card-timers-good = 処理があるときはT0が点滅し、ウォーク中はT1とT2も点滅する。
live-card-timers-bad-stuck = ランプが点灯したまま：計測値が返ってこず、見積もりが古くなっていきます。
live-card-timers-bad-dark = GPUが忙しいのにすべて暗い：何も計測されていません。

live-card-dial-what = 赤い指針はビューの深度、つまり倍率を2のべき乗で示したものです。帯はそこでGPUが使う演算です：直接、df32（1つの数に2つのfloat）、そして2^93以降のfloatexp（指数部の広いfloat）。点は構築中の参照軌道です：緑はビュー用、琥珀色はズームが向かう先のビューより前方用、青はツアーの次の停留用です。輪は導入を、赤い×はビューに追い越されて破棄された構築を示します。
live-card-dial-good-ahead = 琥珀色の点が指針の先に現れ、指針が届く前に輪になる。
live-card-dial-good-crosses = 赤い×が少ない。
live-card-dial-bad-passing = 指針が最後に導入された参照軌道を通り過ぎる：フレームが参照軌道を待ち、フリーズのように感じられます。
live-card-dial-bad-crosses = 赤い×が繰り返し現れる：ズームが構築を追い越しており、CPUが無駄になっています。
live-card-dial-bad-crossing = 2^93をまたぐところは、既知の遅い箇所です：最初のfloatexpのフレームはコストが高くなります。

live-card-bank-what = 各ランプは対応する出来事が起きると点灯し、約150 msかけて消えていくので、忙しいレンダラーではちらつき、何もしていないときは暗くなります。左から右へ：フレーム、更新、参照軌道、そしてワーカー、仕上げ、パレット、演算。
live-card-bank-note = ランプにポインターを合わせると、その意味が表示されます。

live-card-depth-what = 倍率を2のべき乗で示したものです：2^100は約1.3 × 10^30です。
live-card-depth-note = 2^93を超えると、どのステップもコストが高くなり（floatexp）、参照軌道の構築にも時間がかかります。

live-card-iter-what = 反復の要求値：各ピクセルが集合の内側とみなされるまでに反復できる回数です。深度とともに増えます。
live-card-iter-good = ダイブするにつれて着実に増える。
live-card-iter-bad-low = 深いのに低いまま：螺旋やミニブロットが平坦な内部としてレンダリングされます。
live-card-iter-bad-high = 非常に高い：更新のたびにコストが増え、ウォークも長くかかります。

live-card-ms-what = 表示されるフレームの間隔です。
live-card-ms-good = ディスプレイの間隔：60 Hzのディスプレイなら16.7。
live-card-ms-bad-stutter = 33を超える：目に見えるカクつき。
live-card-ms-bad-spikes = 50を超えるスパイク：引っかかり。通常はコストの高いパスが1回あります（GPUパスを参照）。

live-card-clock-what = ローカル時刻の実時間の時計で、10分の1秒まで表示します。
live-card-clock-note = パネルのスクリーンショットに撮影時刻が刻まれるので、問題の画像をアプリのログと照らし合わせられます。

live-card-tach-what = 1秒あたりの反復ステップ数です。白：要求されたステップ（ピクセル × 反復回数）。琥珀色：GPUが実際に実行したステップ。その差はBLA（双線形近似）が多数の反復を一度に飛ばした分で、×Nはその倍率です。積算計はセッションの合計です。
live-card-tach-good = 深いところで差が大きい：BLAが役目を果たしている状態で、×10から×1,000。
live-card-tach-bad-together = 深いところで針がそろう：ここではBLAが飛ばせず（ミニブロットの内部付近でよくあります）、更新が遅くなります。
live-card-tach-bad-red = 白い針が赤い範囲：GPUが1フレームでこなせる以上を要求しています。

live-card-engines-what = 各GPUの忙しさを、そのフレームを画面へ運ぶ点とともに示します：直近1秒間のそのGPUのパスのGPU時間で、ワーカーの移動フレームとスーパーサンプリングのサンプルも含みます。「表示比」は、表示されたフレームのうち各GPUが作った割合です。GPUが1つの場合、ワーカー側は待機したままです。
live-card-engines-good = ダイブ中は両方とも忙しく、ワーカーがフレームの一部を供給している。
live-card-engines-bad-pinned = ウィンドウのGPUであるGPU 0が100%近くに張り付いている：そこがボトルネックで、フレームがカクつきます。
live-card-engines-bad-idle = ワーカーが有効なのに待機している：フレームを届けていません。

live-card-memory-what = RAM：アプリのワーキングセットで、赤い線はそのピークです。VRAM：アプリがGPU上に保持しているもので、割り当てたサイズから種類ごと（Gバッファ、保持とSSAAのテクスチャ、軌道とBLAのバッファ、ワーカーの参照軌道のコピー）に算出し、カードの容量と比べて示します。
live-card-memory-good = ズーム中は平坦で、参照軌道の出入りに合わせて上下する。
live-card-memory-bad-leak = RAMが増え続けて下がらない：メモリリークです。
live-card-memory-bad-vram = VRAMがカードの容量に近い：割り当てが失敗したり、デバイスロストが起きたりすることがあります。

live-card-cores-what = コアごとのCPU使用率（緑から琥珀色、赤へ）と、忙しいスレッドがそれぞれの役割の周りを回るスコープです：参照軌道の構築、評価プール、ワーカー、サイドレンダリング。
live-card-cores-good = ズーム中は1～2コアが忙しく、参照点を選ぶときに全コアで短いバーストが起きる。
live-card-cores-bad-every = すべてのコアが数秒間忙しい：超深度での参照軌道の構築です。
live-card-cores-bad-own = アプリ自身のコアが赤に張り付いている：ウィンドウ自体がカクつきます。

live-card-pixels-what = 1秒あたりに計算されたピクセル数（Mpx/s）です。暗い：ウォーク中でまだ反復中。明るい：完了。
live-card-pixels-good = 移動中は数十Mpx/s。
live-card-pixels-bad = 画像が古いのにゼロ付近：GPUが何か（参照軌道か計測値）を待っています。

## Live render panel: the lamps' help cards

live-card-lamp-plan-what = フレームが計画されました：解像度、反復の要求値、予算を選び、更新するかどうかを決めます。
live-card-lamp-plan-good = ビューや設定が変わるたびにちらつく。
live-card-lamp-plan-bad = ズーム中に暗い：UIスレッドが止まっています。

live-card-lamp-dispatch-what = このフレームでGPUに処理が送られました。
live-card-lamp-dispatch-good = 何かが変化している間は点灯し、待機中は暗い。
live-card-lamp-dispatch-bad = 何も変化していないのに忙しい：アプリが仕上げを終えられていません。

live-card-lamp-live-what = 画面上の画像は、このフレーム自身のレンダリング結果です。
live-card-lamp-live-note = 深いダイブ中にはめったに点灯しませんが、これは正常です：「保持」と「再投影」を参照してください。
live-card-lamp-live-good = フレームが安価な浅い深度で点灯する。

# "Prefer detail" is a setting's name.
live-card-lamp-held-what = 次の画像がその下で組み立てられている間、最後に完成した画像が画面に表示されています：ディープズームの更新、または「ズーム中は詳細を優先」がオンのときの、静止ビューの次のタイルやスーパーサンプリングのサンプル。
live-card-lamp-held-good-dive = ディープズーム中にまとまって点灯し、「採用」で途切れる。
live-card-lamp-held-good-still = 静止ビューが鮮明になっていく間は点灯したままで、SSAAが点滅する。
live-card-lamp-held-bad = ズーム中に「採用」なしで数秒間点灯：更新が完了していません。

live-card-lamp-reproj-what = 新しい画像ができるまで、最後の画像を動いているビューに合わせて引き伸ばしたもの。
live-card-lamp-reproj-good = 移動中に短く点灯。
live-card-lamp-reproj-bad = 長く点灯：引き伸ばすほど画像はぼやけていきます。

live-card-lamp-reading-what = GPUから計測値が返ってきました：パスのタイミング、脱出値の範囲、または内容。
live-card-lamp-reading-good = レンダリング中は一定のちらつき。
live-card-lamp-reading-bad = レンダリング中に長く暗い：タイミングが返ってこず、予算は推測で決められています。

live-card-lamp-pin-pass-what = ピン留めされたウォークのパス：画像を反復帯1つ分先までレンダリングしました。
live-card-lamp-pin-pass-good = 一連のパスが「採用」で終わる。
live-card-lamp-pin-pass-bad = 一連のパスが「破棄」で終わる。

live-card-lamp-split-what = 分割更新のパス：タイルの一部を、完全な反復回数でレンダリングしました。
live-card-lamp-split-good = 短い連続が「採用」で終わる。
live-card-lamp-split-bad = 上限の8回の連続：更新がぎりぎり収まっています。

live-card-lamp-adopt-what = 完了した更新が画面上の画像を置き換えました。
live-card-lamp-adopt-good = ズーム中に1秒に数回。
# "Picture age" names the instrument (its title, PICTURE AGE).
live-card-lamp-adopt-bad = ズーム中にめったに点灯しない：画像が古くなっていきます（「画像の古さ」を参照）。

live-card-lamp-abandon-what = 進行中の更新が破棄されました：ビューが大きく動きすぎたか、古くなりすぎたか、より新しい更新に置き換えられました。ビューに追い越された参照軌道の構築では、薄く点灯します。
live-card-lamp-abandon-good = まれ、またはジャンプ直後に1回。
live-card-lamp-abandon-bad = 頻繁：処理が無駄になり、画像は古いままです。

live-card-lamp-ref-live-what = 現在のビューのために参照軌道を構築しています：参照軌道がない、持っていたものから外れた、またはより長いものが必要です。
live-card-lamp-ref-live-good = 短い点滅。
live-card-lamp-ref-live-bad-long = 長く続く：深いビューはそれを待っています。
live-card-lamp-ref-live-bad-lookahead = ダイブ中に頻繁に点灯し、「参照 先行」が暗い：先読みが追いついていません。

live-card-lamp-ref-ahead-what = ズームが向かう先、ビューより深い位置で、先読みの参照軌道を構築しています。
live-card-lamp-ref-ahead-good = ダイブ中に規則的に点滅。
live-card-lamp-ref-ahead-bad = 深いダイブ中に暗い：ビューが参照軌道を追い越してしまいます。

live-card-lamp-ref-hold-what = ツアーの停留の先読み：次の停留キーフレームの参照軌道を、そこへ向かってなめらかに進む間に構築するので、再生が待たされることはありません。
live-card-lamp-ref-hold-note = ツアーの再生中以外は暗い。
live-card-lamp-ref-hold-good = ツアーの各停留の前に1回点滅。
live-card-lamp-ref-hold-bad = 停留の間ずっと点灯：先読みが遅れ、ツアーがそれを待っています。

live-card-lamp-series-what = 新しい参照軌道のための級数近似で、すべてのピクセルが最初の反復を飛ばせるようにします。
live-card-lamp-series-good = 短い。
live-card-lamp-series-bad = 深いところで長い：参照軌道の導入が遅れます。

live-card-lamp-worker-what = 2つ目のGPUがフレームを届けました。
live-card-lamp-worker-good = ワーカーGPUが有効なときは一定の点滅。
live-card-lamp-worker-bad = ワーカーが有効なのに暗い：フレームを届けていません。

live-card-lamp-tile-what = 仕上げタイル、またはその短冊がGPUに送られました。
live-card-lamp-tile-good = 止まったあとにバーストし、その後は静か。
live-card-lamp-tile-bad = 止まらない：何かがビューや設定を変え続けているため、グリッドが再開を繰り返しています。

live-card-lamp-ssaa-what = 仕上がった画像に、スーパーサンプリングのサンプルが追加されました。
live-card-lamp-ssaa-good = 仕上げのあとに一連の点滅があり、その後は暗い。
live-card-lamp-ssaa-bad = 何度も最初からやり直す：仕上げが妨げられ続けています。

live-card-lamp-palette-what = 色の正規化が新しい脱出値の範囲を取り込み、色がズームに追従しました。
live-card-lamp-palette-good = ズーム中にちらつく。
live-card-lamp-palette-bad = ズーム中に暗い：パレットの範囲が固定されたままで、色が平坦になりコントラストが下がります。

live-card-lamp-mode-what = 演算が切り替わりました：直接、df32、またはfloatexp。
live-card-lamp-mode-good = 2^93のような境界で1回点滅。
live-card-lamp-mode-bad = 行ったり来たり点滅する：ビューが境界をはさんで揺れています。

## The Render client window (File ▸ Render client…): lending this machine's GPU to a render farm

farm-client-title = レンダークライアント
farm-client-intro = レンダーファームのためにフレームをレンダリングします。コントローラー（別のマシンのツール ▸ ファームでレンダリング…）に、そのアドレスとファームキーが表示されます。このマシンはポートを開きません：コントローラーに接続します。
# A field label, and the label of the status row that names the controller.
farm-client-controller = コントローラー
farm-client-find = 検索
farm-client-find-hover = このネットワーク上のコントローラーを探します（コントローラーはファームが待ち受け中の間に応答します）
farm-client-key = ファームキー
# A checkbox beside the masked farm key: show it in clear.
farm-client-show-key = 表示
# A field label (the name this machine goes by), and the label of the status row about this machine.
farm-client-machine = このマシン
farm-client-machine-hover = コントローラーがこのマシンに表示する名前です。
farm-client-share = 共有ドライブ
# Greyed hint inside the empty field; the path is an example.
farm-client-share-hint = 省略可。例：\\fileserver\share
farm-client-share-hover = コントローラーが使う共有ドライブへの、このマシンからのパスです（共有モード）：フレームは接続経由で送る代わりにそこへ書き込まれます。空欄にすると、すべてのフレームを送信します。
farm-client-gpu = グラフィックスカード
# The label of the row with the idle option.
farm-client-when = 実行条件
# A checkbox followed by a number of minutes ("Only when idle for [5] min").
farm-client-when-idle = アイドル状態が続いたときのみ：
farm-client-when-idle-hover = このマシンのキーボードやマウスにこの時間だれも触れていないとき、または画面がロックされている間だけ、処理を引き受けます。だれかが使い始めると、進行中のフレームはすぐに停止してファームに戻されます。
# A collapsible section: the largest job this machine accepts.
farm-client-limits = 制限
farm-client-limits-note = これを超えるジョブは、縮小されることなく拒否されます。
farm-client-limit-size = 最大フレームサイズ
farm-client-limit-ss = 最大スーパーサンプリング
farm-client-limit-iter = 最大反復回数

## Render client: the graphics-card choice

farm-client-gpu-system = システムの選択
farm-client-gpu-every = すべてのグラフィックスカード
# $count: this machine's graphics cards.
farm-client-gpu-every-n = すべてのグラフィックスカード（{ $count }）
# $count: this machine's graphics cards.
farm-client-gpu-every-each = すべてのグラフィックスカード（{ $count }）：それぞれ1セッション
# A card the window cannot name. $number: its number (or numbers) as --list-adapters gives it.
farm-client-gpu-adapter = アダプター { $number }
farm-client-gpu-finding = このマシンのグラフィックスカードを探しています…
# "NAME · GPU 1" is how such a machine is named in the controller's list: keep it as it is.
farm-client-gpu-hover = ファームのフレームをレンダリングするグラフィックスカードです。すべてのカードを選ぶと、各カードが個別のマシンとしてファームに参加し（「NAME · GPU 1」、「NAME · GPU 2」）、「一時停止」と「切断」はそのすべてに作用します。

## Render client: looking for controllers on the network (the Find button)

farm-client-finding = このネットワーク上のコントローラーを探しています…
# $port: the UDP port number.
farm-client-found-none = 応答したコントローラーはありませんでした。コントローラーはファームが待ち受け中の間に応答します。そのファイアウォールでUDP { $port }の受信を許可する必要があります。
farm-client-found-pick = このネットワーク上にあります。クリックして使用します：
# $identity: the controller's identity fingerprint.
farm-client-found-hover = ID { $identity }：このコントローラーを使用します（ファームキーは引き続き必要です）
# After a warning sign. $version and $git: this app's version and build.
farm-client-found-other-build = このビルド（{ $version } { $git }）とは異なるビルドです：このマシンは拒否されます。

## Render client: what is wrong with what was typed, or with starting the client

farm-client-bad-address = コントローラーのアドレスを、そのウィンドウに表示されているとおりHOST:PORTの形式で入力してください（例：192.168.1.20:46733）。
# $why: what is wrong with the key (in English).
farm-client-bad-key = ファームキー：{ $why }
# $why: what is wrong with the name (in English).
farm-client-bad-name = 名前：{ $why }
farm-client-no-config = ファームキーを保存する設定フォルダーがありません。
# $why: the system's reason.
farm-client-start-failed = レンダークライアントを開始できませんでした：{ $why }

## Render client: the state, in the status heading

farm-client-phase-connecting = 接続中…
farm-client-phase-checking = セルフチェックを実行中…
farm-client-phase-idle = 接続済み：処理を待機中
farm-client-phase-rendering = レンダリング中
farm-client-phase-in-use = 待機中：だれかがこのマシンを使用中
farm-client-phase-paused = 一時停止中
farm-client-phase-retrying = コントローラーに接続できません：再試行中
# $seconds: the wait before the next try.
farm-client-phase-retrying-in = コントローラーに接続できません：{ $seconds }秒後に再試行
farm-client-phase-disconnected = 切断済み
farm-client-phase-ended = 終了

## Render client: the line under the status heading

# $address: the controller's address (HOST:PORT).
farm-client-detail-connecting = { $address }に接続中…
# $controller: the controller's name.
farm-client-detail-checking = 「{ $controller }」に接続しました：セルフチェックを実行中
# $controller: the controller's name.
farm-client-detail-approval = 「{ $controller }」に接続しました：そのユーザーがこのマシンを承認するのを待っています
# $controller: the controller's name.
farm-client-detail-idle = 「{ $controller }」に接続しました：待機中
# $why: the check that failed and why (in English).
farm-client-detail-check-failed = セルフチェックに失敗しました：{ $why }
# $why: the controller's reason (in English).
farm-client-detail-refused = コントローラーに拒否されました：{ $why }
# $why: the controller's reason (in English).
farm-client-detail-closed = コントローラーが接続を閉じました：{ $why }
# $why: the controller's reason (in English).
farm-client-detail-ended = コントローラーが接続を終了しました：{ $why }
# $job: the job's name.
farm-client-detail-bundle = ジョブ「{ $job }」：バンドルを受信中
# $job: the job's name.
farm-client-detail-job-waiting = ジョブ「{ $job }」：レンダリングするフレームを待機中
# $why: the limit the job is over (in English).
farm-client-detail-job-refused = このジョブを拒否しました：{ $why }
farm-client-detail-job-over = ジョブは終了しました
# $first and $last: frame numbers.
farm-client-detail-rendering = フレーム{ $first }～{ $last }をレンダリング中
# $time: how long nobody must use the machine ("90 s", "5 min").
farm-client-detail-in-use = 待機中：だれかがこのマシンを使用中です。{ $time }のあいだだれも使わなければ、処理を再開します
farm-client-detail-paused = 一時停止中：コントローラーに通知済み
farm-client-detail-pausing = 一時停止の準備中：進行中のフレームを完了させています
farm-client-detail-cancelled = 進行中のフレームをキャンセルし、一時停止しました
# A machine with several graphics cards. $rendering: the cards rendering now; $total: all its cards (two or more).
farm-client-rendering-on-cards = { $total }枚中{ $rendering }枚のグラフィックスカードでレンダリング中

## Render client: the status rows

# The value of the Controller row. $name: the controller's name; $address: HOST:PORT; $identity: its identity fingerprint.
farm-client-row-controller-value = 「{ $name }」 · { $address } · ID { $identity }
# The value of the This machine row. $name: this machine's name; $identity: its identity fingerprint.
farm-client-row-machine-value = 「{ $name }」 · ID { $identity }
farm-client-row-gpu = GPU
# One graphics card's row. $slot: the card's number.
farm-client-row-gpu-n = GPU { $slot }
farm-client-row-job = ジョブ
# The frames handed to this machine.
farm-client-row-run = 割り当て
farm-client-row-sent = 送信済み
# The Run row, in parts joined with " · ": the run, the frame in progress, its time. $first and $last: frame numbers.
farm-client-run-frames = フレーム { $first }–{ $last }
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-client-run-frame = フレーム { $frame }（{ $count }中{ $index }番目）
# $time: the time spent on it so far ("2.1 s").
farm-client-run-frame-time = このフレームに{ $time }
# One graphics card's run. $frame: the frame in progress; $first and $last: the run's frames.
farm-client-session-frame = フレーム { $frame }（{ $first }–{ $last }中）
farm-client-session-no-card = グラフィックスカードはまだ不明
# The Sent row. $count: frames sent; $time: the mean time per frame ("2.1 s").
farm-client-sent = { $count }フレーム · 1フレームあたり{ $time }
# The Sent row of a machine with several cards. $count: frames sent; $split: each card's share ("812 + 422"); $time: the mean time per frame.
farm-client-sent-split = { $count }フレーム（{ $split }） · 1フレームあたり{ $time }
# After a warning sign. $note: how the GPUs differ, and both GPUs (in English).
farm-client-gpu-note = このマシン：{ $note }。そのフレームはコントローラーのフレームとわずかに異なり、停留中のショットでその差が見えることがあります。
# A collapsible section: the self-check's results.
farm-client-self-check = セルフチェック
farm-client-thumb-hover = このマシンが最後に送ったフレーム
farm-client-resume = 再開
farm-client-resume-hover = 再び処理を引き受けます
farm-client-pause = 一時停止
farm-client-pause-hover = 進行中のフレームを完了してから、それ以上処理を引き受けません。コントローラーはこのマシンのフレームをほかのマシンに回します。
farm-client-cancel-frame = フレームをキャンセル
farm-client-cancel-frame-hover = 進行中のフレームを今すぐ停止して一時停止します。コントローラーはそのフレームを別の場所でレンダリングします。
farm-client-live-paused = このマシンがファームのフレームをレンダリングしている間、ライブビューは一時停止しています。
farm-client-close = 閉じる
farm-client-close-active-hover = ウィンドウを閉じます。このマシンはファームに残ります（ファイルメニューから再び開けます）
farm-client-close-hover = ウィンドウを閉じる
farm-client-disconnect = 切断
farm-client-disconnect-hover = ファームから離脱します：進行中のフレームは返却されます
farm-client-connect = 接続
farm-client-connect-hover = コントローラーに接続して、そのためにフレームをレンダリングします

## The Render on farm window (Tools ▸ Render on farm…): the controller of a render farm

farm-ctl-title = ファームでレンダリング
farm-ctl-out = 出力フォルダー
farm-ctl-browse = 参照…
farm-ctl-out-note = フレームと、farm/内のジョブの状態。同じジョブを再び実行すると再開します。
farm-ctl-size = サイズ
farm-ctl-size-custom = カスタム…
# The size list's choice when the size is typed in. $width and $height: pixels.
farm-ctl-size-custom-value = カスタム：{ $width }×{ $height }
farm-ctl-fps = フレーム/秒
farm-ctl-ss = スーパーサンプリング
farm-ctl-port = 待ち受けポート
farm-ctl-port-hover = クライアントはこのポートに接続します。ローカルネットワークに対して、ファイアウォールでこのポートを許可してください。
farm-ctl-share = 共有ドライブ
# Greyed hint inside the empty field.
farm-ctl-share-hint = 省略可。このマシンからのパス
farm-ctl-share-hover = 共有モード：同じドライブを持つクライアントは、フレームを接続経由で送る代わりにそこへ書き込み、このマシンがそこで1枚ずつ確認します。ドライブを持たないクライアントは、フレームを送信します。
# A label followed by a number of clients and two checkboxes.
farm-ctl-start-with = クライアント数
# The unit after the number in the box ("Start with [3] clients"). $count: that number.
farm-ctl-start-clients = 台で開始
farm-ctl-start-clients-hover = この台数のマシンが接続してセルフチェックに合格すると、ジョブが始まります。ほかのマシンは後から参加できます。
farm-ctl-local = このマシンでもレンダリング
farm-ctl-local-hover = このマシンはクライアント1台として数えます。
farm-ctl-sharing = 参照軌道を共有
farm-ctl-sharing-hover = 深いフレームの参照軌道は、構築に数分から数時間かかることがあります。共有すると、1回だけ構築してほかのマシンに渡します。共有された参照軌道は別の、同じく有効な参照軌道です：超深度では、自分で参照軌道を構築したフレームとわずかな数のピクセルが異なることがあります。
farm-ctl-key = ファームキー
# A checkbox beside the shortened farm key: show all of it.
farm-ctl-show-key = 表示
farm-ctl-copy-key = キーをコピー
farm-ctl-copy-key-hover = 各クライアントのレンダークライアントウィンドウに貼り付けてください
# In place of the farm key, when there is none.
farm-ctl-no-key = まだありません
farm-ctl-make-key = キーを作成
farm-ctl-make-key-hover = クライアントの参加に必要です。「レンダリング」を押したときにも作成されます
# What to do on the other machines. $address: this controller's HOST:PORT.
farm-ctl-steps = 各クライアントで：ファイル ▸ レンダークライアント…、コントローラー{ $address }、そしてファームキー。
# The same when this machine's address is not known. $port: the port number.
farm-ctl-steps-port = 各クライアントで：ファイル ▸ レンダークライアント…、このマシンのアドレスとポート{ $port }、そしてファームキー。
# The size of the job. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
farm-ctl-summary = { $count }フレーム · ツアー{ $seconds }秒 · { $width }×{ $height } ss{ $ss }
farm-ctl-strip-hover = ジョブのすべてのフレームを左から右へ：完了、レンダリング中、未処理、断念。
farm-ctl-no-tour = ツアーが読み込まれていません。
# $folder: the folder; $why: the system's reason.
farm-ctl-out-error = 出力フォルダー{ $folder }：{ $why }
# $why: the system's reason.
farm-ctl-start-failed = ファームを開始できませんでした：{ $why }
# $code: the controller process's exit code, as the code gives it.
farm-ctl-ended-exit = コントローラーが終了しました（終了コード { $code }）。

## Render on farm: the state, in the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-phase-waiting = レンダークライアントを待機中（{ $needed }台中{ $connected }台が接続済み）
farm-ctl-phase-preparing = ジョブを準備中
farm-ctl-phase-rendering = レンダリング中
farm-ctl-phase-paused = 一時停止中
farm-ctl-phase-complete = 完了
farm-ctl-phase-stopped = 停止
farm-ctl-phase-ended = 終了

## Render on farm: the words after the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-detail-waiting = レンダークライアントを待機中：{ $needed }台中{ $connected }台が接続済み
farm-ctl-detail-preparing = 正規化の基準点を、すべてのクライアントのためにこのマシンで1回測定しています
farm-ctl-detail-no-space = 出力フォルダーの空き容量を待っています
farm-ctl-detail-paused = 一時停止中：新しい処理は割り当てられません
farm-ctl-detail-not-rendered = レンダリングされていないフレームを残して終了しました。同じフォルダーで再び実行すると再開します
farm-ctl-detail-stopped = 停止しました。同じフォルダーで再び実行すると再開します
# $code: the controller process's exit code.
farm-ctl-detail-ended = 終了しました（終了コード { $code }）
farm-ctl-detail-finishing = 終了処理中

## Render on farm: progress

# On the progress bar. $done: frames done; $total: frames in the job.
farm-ctl-progress = { $done } / { $total }フレーム
# Added to the progress bar. $rate: frames per second, with decimals.
farm-ctl-progress-rate = { $rate }フレーム/秒
# Added to the progress bar. $time: the time still to go ("38m 10s").
farm-ctl-progress-eta = 残り約{ $time }
# Facts under the progress bar, joined with " · ". $received and $sent: kilobytes per second.
farm-ctl-io = 受信 { $received } KB/s · 送信 { $sent } KB/s
# $space: free space on the output drive ("412 GB").
farm-ctl-free = 空き { $space }
# $time: the time since the job started ("38m 10s").
farm-ctl-elapsed = 経過 { $time }
# $count: frames given up on.
farm-ctl-failed = 未レンダリング { $count }
# Reference orbits. $built: built for this job; $cached: found in the cache; $reused: reused between frames; $shared: passed between machines.
farm-ctl-refs = 参照軌道：構築 { $built }、キャッシュから { $cached }、再利用 { $reused } · 共有 { $shared }
# After a warning sign.
farm-ctl-storage-low = 出力フォルダーがほぼいっぱいです：空きができるまで、新しい処理は割り当てられません。
# After a warning sign.
farm-ctl-gpus-differ = このファームのGPUは同一ではありません。それぞれのフレームはわずかに異なり、停留中のショットでその差が見えることがあります。
# After a warning sign. $count: the GPU classes in the farm.
farm-ctl-gpus-differ-classes = このファームのGPUは同一ではありません（GPUクラス{ $count }種：異なるGPU、ドライバー、またはプローブレンダリング）。それぞれのフレームはわずかに異なり、停留中のショットでその差が見えることがあります。
# $gpu: this machine's GPU, API and driver.
farm-ctl-this-gpu = このマシン：{ $gpu }
# A machine whose GPU differs. $name: the machine; $note: how its GPU differs (in English); $adapter: its GPU.
farm-ctl-gpu-row = { $name }：{ $note } — { $adapter }
# The same with its driver. $driver: the driver's name and version.
farm-ctl-gpu-row-driver = { $name }：{ $note } — { $adapter }、ドライバー { $driver }

## Render on farm: the table of machines

# Column headers (short: the table is narrow).
farm-ctl-col-machine = マシン
farm-ctl-col-state = 状態
farm-ctl-col-gpu = GPU
farm-ctl-col-frames = フレーム
farm-ctl-col-per-frame = 1枚あたり
# How the machine delivers its frames (a speed, or "shared drive").
farm-ctl-col-link = 転送
# When the controller last heard from the machine.
farm-ctl-col-heard = 最終応答
# In the machine's tooltip. $count: its frames that failed verification.
farm-ctl-strikes = 検証に失敗したフレーム：{ $count }枚
# In the Link column: it delivers its frames through the shared drive.
farm-ctl-link-share = 共有ドライブ
# In the Heard column. $seconds: how long ago.
farm-ctl-heard-ago = { $seconds }秒前
farm-ctl-readmit = 再参加を許可
farm-ctl-readmit-hover = 再接続したときに、このマシンが再び参加できるようにします
farm-ctl-remove = 除外
farm-ctl-remove-hover = このマシンをジョブから外します。そのフレームはほかのマシンに回されます
# The GPU column's tooltip.
farm-ctl-probe-same = プローブレンダリングは、このマシンのものと同一です。
# $count: the pixels that differ.
farm-ctl-probe-differs = プローブレンダリングがこのマシンのものと{ $count }ピクセル異なります：フレームはこのマシンのものとわずかに異なります。
farm-ctl-probe-unknown = プローブレンダリングはまだ比較されていません。
# $driver: the driver's name and version.
farm-ctl-gpu-driver = ドライバー { $driver }
# After a warning sign. $note: how its GPU differs (in English: "a different GPU").
farm-ctl-gpu-note = { $note }（このマシンとの比較）

## Render on farm: a machine's state, in the State column

# $count: its frames that failed verification.
farm-ctl-state-removed-bad = 除外：不良フレーム{ $count }枚
farm-ctl-state-removed = 手動で除外
farm-ctl-state-removing = 除外中：診断情報を収集しています
farm-ctl-state-gone = 切断
farm-ctl-state-leaving = 離脱中
farm-ctl-state-checking = 確認中（セルフチェック）
farm-ctl-state-paused = ユーザーが一時停止
farm-ctl-state-parked = 保留：2回クラッシュ。再接続すると再参加します
farm-ctl-state-refused = このジョブを拒否（独自の制限）
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-ctl-state-rendering = レンダリング中 { $frame }（{ $index }/{ $count }）
farm-ctl-state-starting = 割り当てを開始中
farm-ctl-state-idle = 待機中
farm-ctl-state-admitted = 参加許可済み：ジョブを待機中

## Render on farm: the buttons

# A collapsible section: the controller's messages.
farm-ctl-events = イベント
farm-ctl-resume = 再開
farm-ctl-pause = 一時停止
farm-ctl-pause-hover = 新しい処理を割り当てません。進行中のフレームは完了します
farm-ctl-live-paused = このマシンがファームのためにレンダリングしている間、ライブビューは一時停止しています。
farm-ctl-close = 閉じる
farm-ctl-close-active-hover = ウィンドウを閉じます。ジョブは実行を続けます（ツールメニューから再び開けます）
farm-ctl-close-hover = ウィンドウを閉じる
farm-ctl-copy-command = コマンドをコピー
farm-ctl-copy-command-hover = 同等のコマンドラインをコピーします
farm-ctl-stop = 停止
# The Stop button once a stop has been asked for and the job has not ended after 10 seconds.
farm-ctl-stop-now = 今すぐ停止
farm-ctl-stop-hover = これまでのフレームでジョブを終了します。同じジョブを再び実行すると再開します
farm-ctl-render = レンダリング
farm-ctl-render-hover = クライアントの待ち受けを開始し、それらでツアーをレンダリングします

## The Render tour window (Tools ▸ Render tour…)

tour-render-title = ツアーをレンダリング
tour-render-out = 出力フォルダー
tour-render-browse = 参照…
tour-render-out-hover = フレームを書き出すフォルダーのフルパスです。
tour-render-prefix = フレームの接頭辞
# <prefix> stands for the prefix typed in the field.
tour-render-prefix-hover = フレームは<接頭辞>_00000.pngとして書き出されます
tour-render-size = サイズ
tour-render-size-custom = カスタム…
# The size list's choice when the size is typed in. $width and $height: pixels.
tour-render-size-custom-value = カスタム：{ $width }×{ $height }
tour-render-fps = フレーム/秒
tour-render-fps-hover = 1 fps未満も正当な設定です：長いダイブを0.25 fpsでサンプリングすれば、何千ものフレームをレンダリングせずに、深い回帰テストの停留を確認できます。
tour-render-ss = スーパーサンプリング
tour-render-chapter = チャプター
tour-render-whole = ツアー全体
tour-render-chapter-hover = 1つのチャプターをレンダリングしても、フレーム番号はツアー全体の通し番号のままなので、そのフレームをそのまま全体のシーケンスに戻せます。
tour-render-mp4 = mp4を作成
tour-render-mp4-hover = 完了後にffmpegを実行します（PATHに含まれている必要があります）。PNGのフレームは残ります。
tour-render-overwrite = 既存のフレームを上書き
tour-render-overwrite-hover = オフの場合、レンダリングは停止して確認を求めますが、ここからはそれができません。
tour-render-resume = 中断したレンダリングを再開
tour-render-resume-hover = 出力フォルダーにすでにあるフレームを残し、足りないものだけをレンダリングします。最新のフレームを最初に確認し、不完全なら破棄します（レンダリングが止まったときに書き込んでいたフレームです）。正常なフレームが見つかるまでさかのぼります。異なるサイズのフレームが入っているフォルダーは、このシーケンスに混ぜずに拒否します。
# $count: this machine's graphics cards.
tour-render-all-gpus = すべてのグラフィックスカードを使用（{ $count }）
tour-render-all-gpus-hover = このコンピューター上のレンダーファームを通じて、このマシンのすべてのグラフィックスカードで同時にレンダリングします（--gpus all）。フレーム、その名前、再開、mp4は1枚のカードの場合と同じで、ツアー全体がファーム独自の順序でレンダリングされます。異なるモデルのカードでは、いくつかのピクセルがわずかに異なって描かれます：停留中のショットとディゾルブは1枚のカードで処理されます。
tour-render-order = レンダリング順
tour-render-sequential = 順番どおり
tour-render-sequential-hover = フレーム0、1、2…の順（既定）。
tour-render-progressive = プログレッシブ
tour-render-progressive-hover = まずツアー全体をプレビューし、それから完全な細部まで仕上げます：キーフレームを最初にレンダリングし、次に時間的に最も大きな隙間を二分していき、すべてのフレームがそろうまで続けます。どちらの順でもフレームは正しい番号の位置に置かれるので、再開とmp4はそのまま使えます。途中で止めて粗いパラパラ漫画を目で確かめ、どちらの順でも「再開」で再スタートできます。
# The cost of the render. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
tour-render-summary = { $count }フレーム · ツアー{ $seconds }秒 · { $width }×{ $height } ss{ $ss }
# The disk space the frames will take (an estimate). $size: a size ("5.3 GB").
tour-render-disk = フレーム ≈{ $size }
# $size: the frames' estimated size; $free: the free space on that drive ("6.0 GB").
tour-render-disk-free = フレーム ≈{ $size } · 空き { $free }
# After a warning sign. $disk: the line above ("≈51.0 GB of frames · 6.0 GB free").
tour-render-disk-short = { $disk }：シーケンス全体を入れる空きがありません
tour-render-separate = スクリプトをディスクから読み込み、別のプロセスでレンダリングするので、保存していない編集は含まれません。また、そこでGPUデバイスが失われても、このセッションが巻き込まれることはありません。
# On the progress bar. $done: frames done; $total: frames to render.
tour-render-progress = { $done } / { $total }フレーム
# Before the render's first progress line.
tour-render-starting = 開始中…
tour-render-live-paused = ライブビューは一時停止中です。レンダリングがGPUを独占するので早く終わり、自身のフレームのコストも正確に測れます。
tour-render-cancel = キャンセル
tour-render-cancel-hover = レンダリングせずに閉じる
tour-render-copy-command = コマンドをコピー
tour-render-copy-command-hover = 同等のコマンドラインをクリップボードにコピーします
tour-render-stop = レンダリングを停止
tour-render-render = レンダリング
tour-render-render-hover = フレームシーケンスのレンダリングを開始します
# $folder: the output folder.
tour-render-finished = レンダリング完了 → { $folder }
# $why: the render's own reason (in English).
tour-render-failed = レンダリングに失敗しました：{ $why }
# $last: the render's last progress line (in English), or nothing.
tour-render-failed-or-stopped = レンダリングが失敗したか、停止されました。{ $last }
# $why: the system's reason.
tour-render-no-exe = 実行ファイルが見つかりません：{ $why }
# $why: the system's reason.
tour-render-start-failed = レンダリングを開始できませんでした：{ $why }

## The Custom formula dialog (Fractal ▸ Custom formula…)
## The formula language itself stays as it is in every language: function names (sin, cosh, conj…),
## operators, variables (z, c, p1), and the formulas. Keep them as written inside these messages.

# The window's title.
formula-dialog-title = カスタム数式
# A small line at the top of the window, above the formula.
formula-dialog-intro = 1ステップ分の式（Fractint形式）
# The two ways to show and edit the formula (top right of the window).
formula-dialog-textbook = 教科書
formula-dialog-textbook-hover = 数式を教科書のように組版して表示し、そのまま編集します
formula-dialog-text = テキスト
formula-dialog-text-hover = 数式をテキストとして編集します
# The tooltip of the "LaTeX" button (the button's own text stays LaTeX).
formula-dialog-latex-hover = 数式を、ここで組版されているとおりにLaTeXとしてコピーします
# The drop-down list of example formulas (the collection).
formula-dialog-examples = 例
# Help under the formula in Textbook mode. The characters / ^ ( are what the user types: keep them.
formula-dialog-help-textbook = テキストと同じように入力します：/ は直前の項を分数に、^ は指数に、( は括弧になります。矢印キーで数式内を移動し、Tabで次の空欄へ移動します。赤い行は読み込めない行です：クリックするとテキストとして編集できます。
# Help under the formula in Text mode. "; starts a comment": the ; is what the user types.
formula-dialog-help-text = 入力するか、キーパッドを使います。キーパッドには数式言語が知っているすべての名前があります。名前は入力中に補完されます（Tab）。文は改行またはカンマで区切ります。; 以降はコメントです。
# The syntax check, when the formula reads.
formula-dialog-reads = 正しく読み込めます。
# The syntax check in Textbook mode, when the formula has an empty box to fill in.
formula-dialog-empty-box = 空欄を埋めてください。
# The syntax check in Textbook mode. $line: the line number; $message: what is wrong (from the formula parser, in English).
formula-dialog-error-line = { $line }行目：{ $message }
# One function written with another notation's name, and the language's name for it: "ln to log".
# $from: the name as written (ln); $to: the formula language's name (log).
formula-respell-pair = { $from } → { $to }
# A button that rewrites them. $changes: one or more formula-respell-pair, joined with ", ".
formula-respell = { $changes }に変更
formula-respell-hover = これらの関数を、数式言語での名前で書き直します
# Shown when Apply finds a parameter field that is not a number. $name: the parameter (p1 … p5).
formula-param-not-numbers = { $name }が2つの数値の組になっていません
# The Apply button (also in the Formula library window).
formula-apply = 適用
formula-dialog-apply-hover = 数式をコンパイルして表示します
# Shown beside Apply while the formula is being compiled.
formula-dialog-compiling = GPU用に数式をコンパイル中…
# The greyed hint in an empty name box (also in the Formula library window).
formula-name-hint = 名前（省略可）
# The save button: "Update" when the library already has a formula of that name.
formula-dialog-update-in-library = ライブラリ内を更新
formula-dialog-save-to-library = ライブラリに保存
formula-dialog-save-hover = この数式とそのパラメーターを数式ライブラリに保存します
formula-dialog-save-disabled-hover = 保存できるのは、正しく読み込める数式だけです
# Opens the Formula library window.
formula-dialog-library = ライブラリ…
formula-dialog-library-hover = 保存した数式：適用、編集、インポート、エクスポート

## The Custom formula dialog's keypad
## The keys' captions are formula tokens and stay as they are; these are their tooltips.

# The keypad's third tab (the other two are "123" and "f(z)"). A short word.
formula-keypad-tab-names = 名前
# The tabs' tooltips.
formula-keypad-tab-basic-hint = 数値、演算子、よく使う名前
formula-keypad-tab-functions-hint = 関数
formula-keypad-tab-names-hint = すべての変数、パラメーター、定数と、文の構文
# The key z.
formula-key-z = 反復値
# The key c.
formula-key-c = ピクセル（JuliaモードではJulia定数）
# The key p1 on the first tab.
formula-key-p1-below = パラメーター1：下の欄で設定
# The keys p1 … p5. $n: the parameter's number.
formula-key-param = パラメーター{ $n }
formula-key-multiply = 乗算
formula-key-divide = 除算
# The key that types ^2.
formula-key-square-after = 2乗（値のあとに：z^2）
# The key that types ^.
formula-key-power = べき乗：整数、実数、複素数の指数
formula-key-square-root = 平方根
# The key that types |…|: in this language it is the squared modulus, as in Fractint.
formula-key-squared-modulus = 絶対値の「2乗」|z|²：Fractintと同じ
formula-key-add = 加算
formula-key-subtract = 減算（または符号反転）
# The key that types =.
formula-key-assign = 代入：z = …、または一時変数 t = …
# The key that starts a new line (first tab).
formula-key-next-statement-comma = 次の文（カンマでも可）
# The key "del".
formula-key-backspace = カーソルの前の文字（または選択範囲）を削除
# The key e.
formula-key-euler = ネイピア数
# The key that types (0.5, 0.5).
formula-key-complex-constant = 複素数の定数：(実部, 虚部)
# The key that types a comma.
formula-key-comma = 文、または複素数の定数の各部を区切ります
formula-key-left = カーソルを左へ
formula-key-right = カーソルを右へ
# The function keys (the f(z) tab), one per function.
formula-key-sin = 正弦
formula-key-cos = 余弦
formula-key-tan = 正接
formula-key-cotan = 余接
formula-key-exp = 指数関数 e^z
formula-key-sinh = 双曲線正弦
formula-key-cosh = 双曲線余弦
formula-key-tanh = 双曲線正接
formula-key-cotanh = 双曲線余接
formula-key-log = 自然対数 ln（主枝）
formula-key-sqr = 2乗：sqr(z) = z²
formula-key-sqrt = 平方根（主値）
formula-key-recip = 逆数 1/z
formula-key-conj = 複素共役
formula-key-flip = 実部と虚部を入れ替え
formula-key-abs = 両方の部分の絶対値：|x| + i|y|
formula-key-cabs = 絶対値 |z|（実数）
formula-key-real = 実部
formula-key-imag = 虚部（実数として）
# "fn" is Fractint's name for a function slot: keep it.
formula-key-ident = 恒等関数（Fractintのfnのプレースホルダー）
# The key "pixel".
formula-key-pixel = cと同じ（Fractintでの名前）
# The key "z =".
formula-key-set-z = zを設定する文を開始
# The key "t =".
formula-key-temporary = 一時変数：新しい名前なら何でも可（t、w、a2、…）
# The key "; …".
formula-key-comment = 行末までのコメント
# The key that starts a new line (names tab).
formula-key-next-statement = 次の文

## The formula's completion list (names offered as you type)

# Beside a name the formula itself assigns.
formula-completion-variable = この数式が設定する変数
# Beside a function typed with another notation's name.
# $what: what the function is, in English (from the formula parser: "The natural logarithm");
# $other: the other notation's name (ln); $ours: the formula language's name (log).
formula-completion-other-spelling = { $what }：{ $other }はここでは{ $ours }と書きます
# Under a list cut short. $count: the names not shown.
formula-completion-more = … ほか{ $count }件
# The last line of the list. Tab, Enter and Esc are key names.
formula-completion-keys = TabまたはEnterで補完、Escで閉じる

## The Custom formula dialog's Textbook mode

# The tooltip of a line that does not read. $why: what is wrong (from the formula parser, in English, or formula-textbook-fractint-sections).
formula-textbook-unread-hover = 読み込めません：{ $why }。クリックするとテキストとして編集できます。
# Why a line with Fractint's sections is shown as text in Textbook mode.
formula-textbook-fractint-sections = Fractintのセクション（初期化の':'、比較、a = b = …）はテキストとして編集します

## The Formula library window (Fractal ▸ Formula library…)

# The window's title.
formula-library-title = 数式ライブラリ
# The window's two tabs. $count: the formulas in each.
formula-library-mine = マイ数式（{ $count }）
formula-library-mine-hover = 保存またはインポートした数式
formula-library-collection = コレクション（{ $count }）
formula-library-collection-hover = Fractadyneに付属する数式。それぞれに開始ビューがあります
# A small marker before the name of the formula the view shows now. A short word.
formula-library-showing = 表示中
# The tooltip of Apply on a formula of the collection.
formula-library-apply-shipped-hover = 開始ビューで表示します
# A row's Edit button (after a pencil icon).
formula-library-edit = 編集
formula-library-edit-hover = 適用せずに、カスタム数式ダイアログで開きます
# A collection row's button that copies the formula into "My formulas".
formula-library-copy-to-mine = マイ数式にコピー
formula-library-copy-to-mine-hover = 保持したり変更したりできるよう、自分の数式に追加します
formula-library-close = 閉じる
# Saves the formula the view shows, under the name typed beside it.
formula-library-add-current = 現在の数式を追加
formula-library-add-current-hover = ビューに表示中の数式を保存します
# "Fractal > Custom formula…" is the menu path: write it as the menus are named.
formula-library-add-current-disabled-hover = 先にカスタム数式を適用してください（フラクタル > カスタム数式…）
formula-library-import = インポート…
formula-library-import-hover = 数式ファイル内の数式を追加します。ここにあるものは何も置き換えられません
formula-library-export-all = すべてエクスポート…
formula-library-export-all-hover = ここにあるすべての数式を1つのファイルに書き出します。共有や保存に使えます
# The filter box: its label, and the greyed hint inside it.
formula-library-filter = フィルター
formula-library-filter-hint = 名前またはテキストで絞り込み
# Shown while the user has no saved formulas.
formula-library-empty = 保存した数式はまだありません。カスタム数式ダイアログから保存するか、ビューに表示中の数式を追加するか、ファイル（このアプリの形式、またはFractintの.frm）をインポートしてください。
# The tooltip of Apply on one of the user's formulas: with a view saved, and without one.
formula-library-apply-view-hover = この数式を、一緒に保存されたビューで表示します
formula-library-apply-hover = この数式を表示します
# The tooltip of a row's save icon.
formula-library-export-one-hover = この数式をファイルにエクスポート
# A row asks before deleting: the question, then its two buttons. "Delete" is also the delete icon's tooltip.
formula-library-delete-ask = 削除しますか？
formula-library-delete = 削除
formula-library-keep = 残す
# Under a list cut short. $count: the formulas not listed.
formula-library-more = …ほか{ $count }件：フィルターに入力して探してください。
formula-library-no-match = フィルターに一致するものはありません。
# Under a saved formula this version cannot read. $why: what is wrong (from the formula parser, in English).
formula-library-unread = このバージョンでは読み込めません：{ $why }

## Formula library notices

# $why: the system's reason, in English.
formula-library-save-failed = 数式ライブラリを保存できませんでした：{ $why }
formula-library-nothing-to-save = 保存するものがありません：数式が空です。
# $name: the formula's name in the library.
formula-library-updated = 数式ライブラリの「{ $name }」を更新しました。
formula-library-saved = 「{ $name }」を数式ライブラリに保存しました。
# $name: the formula's name; $why: what is wrong.
formula-library-apply-failed = 「{ $name }」を適用できません：{ $why }
# Copying a formula of the collection into "My formulas". $name: its name; $as_named: the name it was given (the same, or "Name (2)" when the name was taken).
formula-library-copy-exists = 「{ $name }」はすでにマイ数式にあります。
formula-library-copied = 「{ $name }」を「{ $as_named }」としてマイ数式にコピーしました。
# $name: the formula's name.
formula-library-deleted = 「{ $name }」を数式ライブラリから削除しました。
# Why the library could not be saved (follows formula-library-save-failed).
formula-library-no-config-dir = 設定フォルダーがありません
# At startup, when the library file cannot be read. $why: the reason, in English; $path: a file's full path.
formula-library-unreadable-kept = 数式ライブラリを読み込めませんでした（{ $why }）。{ $path }として保管し、新しい空のライブラリを開始しました。
formula-library-unreadable-stuck = 数式ライブラリを読み込めず（{ $why }）、退避することもできませんでした：数式を保存するとこのファイルが置き換えられるので、その前に{ $path }を安全な場所にコピーしてください。

## Formula library: Import and Export
## A formula file's contents stay in English; this is what the windows and notices around them say.

# File-type names in the open and save dialogs (the extensions are added by the system).
formula-filter-formula-files = 数式ファイル
formula-filter-fractadyne = Fractadyneの数式
formula-filter-fractint = Fractintの数式
formula-filter-all-files = すべてのファイル
# Why a file cannot be imported (they follow formula-import-failed).
formula-import-too-large = 数式ファイルにしては大きすぎます
formula-import-not-text = テキストファイルではありません
formula-import-no-formulas = 数式が含まれていません
# $why: the TOML reader's message, in English.
formula-file-not-formulas = 数式ファイルではありません（{ $why }）
# $format: what the file says it is ("fractadyne-gradients").
formula-file-other-format = 「{ $format }」ファイルで、数式ファイルではありません
# "[[formula]]" is the file's own syntax: keep it.
formula-file-no-formulas = 数式が含まれていません（数式ファイルでは[[formula]]テーブルとして列挙します）
# $file: the file's name; $why: one of the reasons above, or the system's (in English).
formula-import-failed = 「{ $file }」をインポートできませんでした：{ $why }。
# After an import that added nothing. $file: the file's name.
formula-import-nothing-new = 「{ $file }」に新しいものはありません。
# After an import. $count: the formulas added; $file: the file's name.
formula-import-added = 「{ $file }」から{ $count }個の数式をインポートしました。
# The same two, followed by what else happened. $notes: one or more of the formula-import-note-… messages below, joined with "; ".
formula-import-nothing-new-notes = 「{ $file }」に新しいものはありません：{ $notes }。
formula-import-added-notes = 「{ $file }」から{ $count }個の数式をインポートしました：{ $notes }。
# Notes (clauses, no final full stop). $count: the formulas already in the library, skipped.
formula-import-note-duplicates = { $count }個の数式はすでにライブラリにありました
# One formula was renamed because its name was taken. $from: its name in the file; $to: the name it was given ("Name (2)").
formula-import-note-renamed-one = 「{ $from }」は使われていたため、「{ $to }」にしました
# $count: the formulas renamed because their names were taken (2 or more).
formula-import-note-renamed = 名前が使われていた{ $count }個の名前を変更しました
# $count: the entries skipped as empty or too long.
formula-import-note-skipped = { $count }個のエントリは空または長すぎるため、飛ばしました
# A second sentence after importing a Fractint .frm file. $count: the entries that do not read; $why: the commonest reason (from the formula parser, in English).
formula-import-frm-unread = { $count }個のエントリはこのバージョンでは読み込めません（最も多い理由：{ $why }）。
# After exporting one formula. $name: the formula's name; $file: the file's name.
formula-export-one = 「{ $name }」を「{ $file }」にエクスポートしました。
# After exporting several. $count: how many; $file: the file's name.
formula-export-many = { $count }個の数式を「{ $file }」にエクスポートしました。
# $file: the file's name; $why: the system's reason, in English.
formula-export-failed = 「{ $file }」に書き込めませんでした：{ $why }

## The control panel (right-hand side): header and section headings

# The panel's title, beside the button that hides it.
panel-title = コントロール
panel-hide-hover = コントロールパネルを隠す
# The button that brings the hidden panel back (top right of the view).
panel-show-hover = コントロールパネルを表示
# Section headings.
panel-section-navigate = ナビゲート
panel-section-coloring = 着色
panel-section-quality = 品質
panel-section-effects = エフェクト
panel-section-overlays = オーバーレイ
panel-section-advanced = 詳細設定
# The last section: reference notes about the current fractal. $name: the fractal's name (Mandelbrot, Burning Ship…).
panel-section-about = { $name }について
panel-section-performance = パフォーマンス

## The control panel: Navigate
## Labels before a control sit in a column about 104 px wide (roughly 15 characters): a longer label pushes its control to the right.

panel-zoom-speed = ズーム速度
panel-zoom-speed-hover = Spaceを長押ししたときの連続ズームの速さです（1× ≈ 1.5秒で2×）。
panel-snap-zoom = ズームを特徴点に吸着
panel-snap-zoom-hover = マウスホイールやSpaceでズームインしているとき、カーソルの近くにあるミニブロット（または実軸）へズームを引き寄せ、それがカーソルの下に滑り込んでズームがそこに着地するようにします。特徴点が近いほど、引き寄せる力は強くなります。Mandelbrotの単一ビューのみ。
panel-click-zoom = クリックでズーム
panel-click-zoom-hover = オンにすると、ビュー内で左クリックしたとき、クリックした点を中心に下の倍率でダイブします（右クリックで戻ります）。ドラッグは引き続きパンで、Shift+ドラッグ / 右ドラッグは引き続きボックスズームです。Backspaceでクリックを元に戻せます。デュアルビューではどちらのパネルでも機能し、これがオンの間はJulia cのピン留めがCtrl+クリックに移ります。Shiftを押したままにすると、クリックした場合に移動する先のビューを表示する拡大鏡がカーソルの下に現れ、押したままクリックするとその点に移動します。Shift+ドラッグは引き続きボックスズームです。
# The click-to-zoom factor, followed by buttons 2×, 4×, 10×…
panel-click-zoom-factor = 倍率
panel-click-zoom-snap = 最寄りの中心に合わせる
panel-click-zoom-snap-hover = ジャンプしたあと、最寄りのミニブロットの核を計算して正確にそこへ合わせるので、特徴点の近くではなく特徴点そのものに着地します。ズームは変わりません（それはクリックで決まっています）。Mandelbrotのみ。計算はバックグラウンドで行われます：ビューはすぐにジャンプし、答えが届くと自動的に修正されます。極端な深度では数秒かかることがあります。

## The control panel: Coloring
## Labels before a control sit in a column about 104 px wide (roughly 15 characters).

panel-method = 着色方法
panel-method-hover = 脱出データを色に割り当てる方法です。ストライプ / 三角不等式 / 軌道トラップ / 分解は軌道の構造を表し、距離推定は境界への近さで陰影を付けます。
panel-stripe-density = ストライプ密度
panel-stripe-tail = 末尾のみ（ディープズーム）
panel-stripe-tail-hover = ストライプ項を、軌道全体ではなく各軌道の最後のN回の反復（指数窓）で平均します。極端な深度では、どの軌道も最後の数百回以外は参照軌道に寄り添うため、軌道全体の平均はビュー全体で1色になってしまいます。末尾だけを使うとコントラストが保たれます。窓より短い軌道には影響しません。--renderでは--stripe-tail Nも使えます。
panel-tail-length = 末尾の長さ
# The unit after the tail length's number: short for "iterations" (the slider reads "64 it").
panel-tail-length-unit = 回
panel-tail-length-hover = 窓の長さ（反復回数）です。短い窓（8～32）はどの深度でも古典的な太い帯を生みます。長い窓では、長いカオス的な軌道のストライプ項が平均されて打ち消し合うため、再び平坦な色に近づきます。
panel-trap-shape = トラップの形
panel-palette = パレット
# The two colours of the Binary palette, each beside its colour swatch.
panel-in-set = 集合内
panel-out-of-set = 集合外
# The two colours of the Duotone palette, each beside its colour swatch.
panel-shadow = シャドウ
panel-highlight = ハイライト
panel-cycle = サイクル
panel-offset = オフセット
panel-normalize-hover = 極端な深度では脱出回数が数十万にわたり、固定のサイクルでは隣り合うピクセルの間でパレットが何千回も折り返すため、正しい密な模様が斑点状のノイズに見えてしまいます。これは、ビューで実際にエイリアシングが起きているときに限り、パレットをビューで測定した脱出値の範囲に割り当て直します（そのとき「サイクル」は、その範囲にわたってパレットを何回繰り返すかを決めます）。なめらかなビューはそのままにします。割り当て直すと、意味もなく画像が変わってしまうからです。それでも割り当て直すには、下の「常に範囲に合わせる」をオンにします。着色方法が「スムーズ反復」のときのみ。--normalizeと同じです。
# A checkbox indented under "Normalize deep colors" (also in the Color menu).
panel-fit-range = 常に範囲に合わせる
panel-fit-range-hover =
    ビューでエイリアシングが起きていなくても、パレットをビューで測定した脱出値の範囲に割り当て直します。上のチェックボックスは、密な模様が斑点のように見えてしまう場合にだけ割り当て直します。

    ノイズを直すためではなく、画面上にあるものの範囲にパレットを広げたいときに使います。測定値が必要なので、ビューが仕上がるまでの一瞬は何もせず、脱出したピクセルがないビューではまったく何もしません。
# A checkbox indented under "Always fit range" (also in the Color menu).
panel-log-scale = 対数カラースケール
panel-log-scale-hover =
    パレットを脱出値に比例させず、その「対数」に従って広げます。深いところでは脱出回数が大きい側に偏るため、線形の割り当てではパレットの大部分が境界付近の薄い層に費やされ、ほかのすべてが平坦になります。

    正規化された割り当てに適用されます。これは「深い色を正規化」（エクスポートでは--normalize）でオンになるものです。範囲が大きく偏っている場合は、これがオンかどうかに関係なく、対数の割り当てが使われます。
# The same checkbox's tooltip while it is greyed out (normalization is off).
panel-log-scale-off-hover =
    正規化が必要です：これは測定した脱出値の範囲にパレットを「どのように」割り当てるかを選ぶものなので、「深い色を正規化」がオフ（かつ--normalizeのエクスポートでもない）のときは変える割り当てがなく、1ピクセルも変わりません。

    使うには「深い色を正規化」をオンにしてください。
# The palette animation: its kind, then its speed.
panel-animate = アニメーション
panel-anim-speed = 速度
panel-anim-speed-hover = 循環の速さ：1秒あたりの色オフセットのサイクル数、または（ランダム）1秒あたりのグラデーションの変化の回数。
panel-shuffle-gradient = グラデーションをシャッフル

## The control panel: Quality

panel-auto-iter = ズームに合わせて反復回数を自動調整
# The iteration-count label while auto-scaling is on (the count is then a base the zoom raises).
panel-iterations-base = 反復回数（基準）
panel-iterations = 反復回数
panel-iterations-hover = 基準となる反復回数です。自動調整がオンのときは、実効的な回数がズーム深度とともに増えます。移動中はプレビューの反復回数を応答性のために低く（50,000に）抑え、ビューが仕上がると完全な回数まで鮮明にします。そのため、深い境界は動いている間はなめらかに見え、止まると解像します。
# A warning under the iteration count. $count: the iterations the settled preview renders.
panel-res-limited = この深度では、応答性を保つため、仕上げのプレビューは{ $count }回の反復を低い解像度でレンダリングします。完全な解像度の細部はエクスポートで得られます。
panel-anti-alias = アンチエイリアス
# Anti-aliasing off (the other choices are 2×, 3×, 4×, 8×).
panel-anti-alias-off = オフ
panel-anti-alias-hover = 静止画のためのスーパーサンプリングです（ビューが仕上がると適用されます）。高くするほど外側の細かい「ちり」が抑えられますが、レンダリング時間が増えます。

## The control panel: Effects

panel-relief-lighting = 3Dレリーフ照明
panel-relief-lighting-hover = 距離推定の傾きを使って表面に陰影を付け、浮き彫りに光を当てたような見た目にします。（正則な族：Mandelbrot / Multibrot）
panel-light-angle = 光の角度
panel-relief = レリーフ
panel-relief-hover = 低い = シャープなレリーフ、高い = 柔らかく平坦。
panel-rotate-light = 光を回転
panel-rotate-light-hover = 時間とともに光の方向を回転させます（「速度」スライダーを使います）。
panel-distance-glow = 距離グロー
panel-distance-glow-hover = 距離推定による明るい等高線の帯で、境界の近くで密になり光る糸のようになります。（正則な族）
panel-glow = グロー
panel-band-width = 帯の幅
panel-band-width-hover = 距離の等高線の間隔（帯あたりのオクターブ数）。
panel-animate-glow = グローをアニメーション
panel-animate-glow-hover = 時間とともにグローの帯を流します（「速度」スライダーを使います）。

## The control panel: Overlays

panel-orbits-hover = カーソルの下の点の反復経路を描きます。
panel-orbit-anim = アニメーション（走る点）
panel-orbit-anim-hover = 色が循環する点を、軌道に沿って走らせます。
panel-orbit-speed = 軌道の速度
panel-minimap-hover = 「現在地」マーカーとズーム深度を添えた、ホームビューの小さな全体図です。クリックすると、その領域へジャンプします。

## The control panel: Advanced
## Labels before a control sit in a column about 104 px wide; most of these are already wider in English.

# Small headings of the section's two groups.
panel-accelerators = 高速化
panel-performance-tuning = パフォーマンス調整
panel-bla = BLAによる高速化（ディープズーム）
panel-bla-hover = 双線形近似：極端な深度で軌道全体にわたって反復を飛ばします（floatexpのMandelbrot、1e28×以上）。GPUでのレンダリングが約5倍速く、出力は同一です（セルフテストで検証済み）。既定ではオンです。比較したいときや、アーティファクトが出たときはオフにしてください。
panel-series-approx = 級数近似
panel-series-approx-hover = 多項式から摂動を始めて、BLAが働かない場合（df32の深度、Multibrot、BLAがオフ）に最初のほうの反復を飛ばします。出力は同一です。比較するにはオフにしてください。
panel-glitch-correct = グリッチ補正（エクスポート）
panel-glitch-correct-hover = エクスポートする画像のための、複数の参照軌道によるグリッチ補正です：摂動のグリッチを検出し、そのピクセルをきれいになるまで追加の参照軌道に対して再レンダリングします。既定ではオンです。約32 MPまでのエクスポートで使われます（補助データを使わない着色の場合）。それより大きな画像とライブビューは通常の方式を使います。MandelbrotやMultibrotのビュー（Juliaモード以外）では使われません：厳密な計算と照合したところ、そこでは何も修正せず、エクスポート時間が最大5倍かかったためです。
panel-dive-limit = 自動ズームのダイブ上限
panel-dive-limit-hover = 自動ズーム（Aキー）が止まる深度です。約1e271×まではなめらかに滑るように進み、それより深いところでは、超深度に素早く到達するため、カクカクした段階的なダイブに切り替わります。
# Followed by two choices, Speed and Quality.
panel-autopilot-priority = 自動ズームの優先
panel-priority-speed-hover = 設定した速度でズームします。細部のあるフレームができしだい画面が更新され、更新が途切れないような解像度で表示されます（ズームがレンダリングを追い越すと「最小移動時解像度」を下回ることもあります）。更新の合間は、最後のフレームがズームに追従して、だんだんぼやけます。
panel-priority-quality-hover = 更新はすべて完全に解像したフル解像度のフレームで、画面上のフレームが次のフレームの到着前に大きく拡大されすぎないよう、必要に応じてズームが遅くなります。どの瞬間も鮮明な、ゆっくりとしたダイブです。
# Followed by two choices, Detail and Misiurewicz point.
panel-autopilot-target = 自動ズームの目標
panel-target-detail-hover = ビュー内のフラクタルの境界で最も豊かな構造に向かってダイブし、進みながら目標を選び直します。
panel-target-misiurewicz-hover = 最寄りのMisiurewicz点（構造が永遠に繰り返す螺旋の中心）を見つけ、そこへ正確にダイブします。ダイブはまず、点をダイブ上限の深度まで計算する間は静止します。通常の深度では一瞬ですが、1e1000×を超えると長くなります。Mandelbrot集合のみ。
panel-work-budget = ライブレンダリング予算
panel-work-budget-hover = ディープズームでのライブビューの、細部と速度のバランスです。高くするとより完全な解像度でレンダリングされます（より鮮明で、拡大したような「ぼやけた」見た目が減ります）が、フレームレートが下がり、非常に高い値では最も重いフレームで一時的にGPUが止まるおそれがあります。エクスポートはこの設定に関係なく、常に完全な解像度です。
panel-min-motion-res = 最小移動時解像度
panel-min-motion-res-hover = ディープズームで連続ズームやパンをしている間に、ライブビューが下げてよい最低の解像度です。高くすると速いダイブがブロック状/ピクセル状になるのを防げます（より鮮明）が、フレームレートが下がります。100%では、深いダイブの細部の更新を完全なコストでレンダリングしなければならず、ディープズームが目に見えてカクついたり段階的になったりすることがあります。深いダイブがぎくしゃくする場合は、これを下げてください。既定値は30%で、仕上がった画像やエクスポートには影響しません。
panel-finish-sound = レンダリング完了時に音を鳴らす
panel-finish-sound-hover = エクスポートやツアーのレンダリングが完了すると、FRACTINTの完了メロディーを鳴らします：上がっていく3つの音（1047、1109、1175 Hz、各100 ms）で、DOS版の原作に記録されていたとおりです。帯域制限された矩形波で合成し、ハイパスフィルターをかけて、それを鳴らしていた小さなPCスピーカーのような音にしています。
panel-prefer-detail = ズーム中は詳細を優先
panel-prefer-detail-hover = ズームやパンの間、毎フレーム品質を下げて再レンダリングする代わりに、最後に完全に描かれたフレームを、動きに追従するよう拡大縮小・追跡しながら表示し続けます。動きが止まると、ビューは完全にレンダリングされます。オフ：動きは低い解像度でライブにレンダリングされます（よりなめらかで、より粗い）。浅いビュー（directモード）は常にライブでレンダリングされます：どちらにしても毎フレーム安価で鮮明だからです。

## The control panel: About (the current fractal)

# A link to the fractal's reference page, followed by an arrow.
panel-about-reference = 参考資料

## The control panel: Advanced ▸ Second graphics card

# A label before a drop-down (wider than the panel's label column already).
gpu2-label = 2枚目のグラフィックスカード
gpu2-off = オフ
# Another device on the same graphics card as the window (the default).
gpu2-same-card = 同じカード
# A card the list does not name. $number: the card's number.
gpu2-card-n = カード{ $number }
gpu2-finding = このマシンのグラフィックスカードを探しています…
gpu2-none = このマシンにはほかのグラフィックスカードがありません。
gpu2-hover =
    別のグラフィックスデバイスが、深いビューが動いている間のフレームと、止まったあとに鮮明にするための追加サンプルの一部をレンダリングします。そのため、画像がズームに追いつき、最終的な品質に早く到達します。ウィンドウを描画しているカードは、引き続きすべてを描画します。

    別のカードを使うと、そのカードの分の速度が加わります。異なる2つのカードのモデルでは、いくつかのピクセルがわずかに異なって描かれます。2枚目のカードが失敗しても、ビューは1枚で処理を続けます。

    「同じカード」（既定）は、このカード上に2つ目のデバイスを開きます。速度は増えませんが、画面のフレームを待たずにレンダリングします。カードを共有するので、カードをリセットするような障害が起きるとアプリが終了します（ウィンドウ自身のレンダリングでの障害と同じです）。次回の起動時に、オフにするかどうかを尋ねます。

## The view (canvas)

# A notice after Ctrl+click pins the Julia c at a deep zoom.
canvas-julia-pin-quantized = Julia cをピン留めしました。ただし倍精度で保持されるため、この深度ではパネル全体が1つの値に丸められ、ピンはビューの中心に置かれます。狙いを付けるにはズームアウトしてください。
# A small caption in the top-left corner of the view while the orbit overlay is fitted to the view (its "Normalize (fit to view)" option).
canvas-orbit-normalized = 軌道（正規化）
# The minimap's tooltip.
canvas-minimap-hover = 全体図 / 現在地。ドラッグで現在のズームのままパン、スクロールでズームイン/アウト、クリックでその領域へジャンプします（ホームのズーム）。

## The timestamp overlay (View ▸ Show timestamp)

# Under the large elapsed-time clock. $frame: the frame number (as the log writes it, without grouping).
overlay-timestamp-frame = フレーム { $frame }

## File ▸ Settings ▸ Reference cache…

cache-title = 参照軌道キャッシュ
cache-intro = 極端な深度では、ビューで時間がかかるのは参照軌道です：任意精度の演算で最大1時間かかります。Fractadyneは完成した軌道をここに保存しておくので、訪れたことのある位置に戻ったり、その近くにズームしたりするときは、代わりに数秒で済みます。軌道1つは約4 MBです。
cache-off = この実行ではオフです（タスクの呼び出し、または--no-orbit-cache）。次に通常どおり起動するまで、何も読み書きされません。
# Small headings of the window's parts.
cache-location = 場所
cache-usage = 使用量
cache-limit = 上限
cache-no-config = （使用できる設定フォルダーがありません）
cache-open-folder = フォルダーを開く
# The text on the usage bar. $used and $limit: sizes such as "1.2 GB"; $count: the orbits kept.
cache-usage-bar = { $used } / { $limit }：軌道{ $count }個
cache-limit-hover = キャッシュがこれを超えると、構築コストが「最も安い」軌道から削除されます。1時間かかった深い軌道は、すぐに作れる軌道がいくつあっても、それらがどれほど最近使われていても、それより長く残ります。
# Beside the limit. $count: about how many orbits of the live view's length fit in it.
cache-limit-estimate = ≈ ライブの上限の長さの軌道{ $count }個分
cache-kept = 保存済み（価値の高い順）
# Column headers of the list of kept orbits.
cache-col-iterations = 反復回数
cache-col-precision = 精度
cache-col-size = サイズ
cache-col-last-used = 最終使用
# The precision column. $bits: the precision in bits.
cache-row-bits = { $bits }ビット
# Under the list. $count: the orbits not listed.
cache-rows-more = … ほか{ $count }個
# When an orbit was last used (the "last used" column).
cache-ago-now = たった今
# $count: minutes.
cache-ago-minutes = { $count }分前
# $count: hours.
cache-ago-hours = { $count }時間前
# $count: days.
cache-ago-days = { $count }日前
# Asked before clearing. $count: the orbits; $size: their total size, such as "1.2 GB".
cache-confirm = 軌道{ $count }個（{ $size }）を削除しますか？失うのは再構築にかかる時間だけで、データは失われません。
cache-delete = 削除
cache-cancel = キャンセル
cache-clear = キャッシュを消去…
cache-clear-hover = 保存されているすべての軌道を削除します。事前に確認があります。
cache-close = 閉じる
# $count: the orbits removed.
cache-cleared = 参照軌道キャッシュを消去しました：軌道{ $count }個を削除しました。
# $why: the system's error message.
cache-clear-failed = 参照軌道キャッシュを消去できませんでした：{ $why }

## Help ▸ Diagnostics…

diag-title = 診断
diag-intro = テストを実行して、Fractadyneがお使いのハードウェアで正しく動作するかを確認します。結果はこのマシンに保存され、問題の報告に添付できます。勝手にどこかへ送信されることはありません。
# $status: which arbitrary-precision library is in use, as the program reports it (in English).
diag-arithmetic = ディープズームの演算：{ $status }
# The three tests' buttons, each followed by a sentence saying what it does.
diag-run-selftest = セルフテストを実行
diag-run-uitest = UIテストを実行
diag-run-gputest = GPU演算チェックを実行
diag-blurb-selftest = お使いのGPUでの数学とレンダリングを、既知の正しい結果と照合します。約15秒かかります。
diag-blurb-uitest = インターフェースとライブビューを巡回し、複数のズーム深度でスクリーンショットを撮ります。1～2分かかり、実行中はウィンドウが開きます。
diag-blurb-gputest = マシンで使えるすべてのグラフィックスバックエンドで、シェーダーのdouble-float演算を既知の正しい値と照合します。数秒で終わり、ウィンドウは開きません。失敗が報告された場合、それは私たちがぜひ見たい本物の知見です。レポートに添付してください。
# While a test runs. $count: the checks finished so far.
diag-checks-done = { $count }件のチェックが完了
# A finished test's result. $test: the test's name (Self-test, UI test, GPU arithmetic check).
diag-verdict-captured = { $test }：結果を記録しました
diag-verdict-passed = { $test }：合格
diag-verdict-problems = { $test }：問題が報告されました
# Under the GPU arithmetic check's result.
diag-informational-note = このテストには合格も不合格もありません：お使いのGPUがどのように計算するかを記録するもので、報告された失敗はお使いのマシンの故障ではなく本物の知見です。いずれにしても、レポートに添付していただくのが最も役立つ使い方です。
# Under a test's result when it reported problems.
diag-problems-note = 参照GPU以外のハードウェアでは、ある程度の差は想定どおりです（特に画像の比較）。これを問題の報告に添付していただくのが最も役立つ使い方です。
diag-open-results = 結果を開く
diag-attach = 問題の報告に添付…
diag-attach-hover = この結果を含めた報告ダイアログを開きます。issueには説明だけでなく、機械的に検証されたテストが添付されます
diag-console = 診断情報をコンソールに出力
diag-console-hover = 今後、[fd-…]の行をstderrに表示します。引数なしで起動した場合は既定でオフです。これによって「記録」される内容は何も変わりません（ログファイルには常にすべての行が書き込まれます）。ターミナルを見ているのでなければオフのままにしてください。このセッションの間だけ有効です。起動時の出力も捉えるには、--consoleフラグかFRACTADYNE_CONSOLE=1を使ってください。
diag-close = 閉じる
# Errors starting a test. $why: the system's error message; $dir: a folder.
diag-error-no-exe = 実行ファイルが見つかりません：{ $why }
diag-error-create-dir = { $dir }を作成できません：{ $why }
diag-error-start = テストを開始できませんでした：{ $why }

## Navigate ▸ Misiurewicz explorer…

misi-title = Misiurewiczエクスプローラー
misi-mandelbrot-only = Misiurewicz点の閲覧はMandelbrot専用です。使うにはフラクタルを切り替えてください。
# While the gallery is being built. $seconds: the time so far, such as "3".
misi-solving = 小さな型の族を計算中（{ $seconds }秒）…
# Above the gallery. $total: all points; $curated: the hand-picked ones; $found: the ones the sweep found.
misi-summary = { $total }個の点：厳選したもの{ $curated }個、小さな型についてシードを用いたニュートン法の探索で見つかったもの{ $found }個（上半平面。粗いグリッドで大半の引き込み領域が見つかりますが、すべてとは保証できません）。
# The name of a point the sweep found. $kind: its type, such as "(3,1)"; $point: its coordinate, such as "-0.1011+0.9563i".
misi-entry-name = { $point }の{ $kind }
# Under each thumbnail. $kind: the point's type, such as "(3,1)"; $octaves: the zoom period in octaves, such as "3.17"; $twist: the twist per repeat in degrees, such as "+12.5". "oct" is short for octaves.
misi-entry-readout = { $kind } · 繰り返し{ $octaves }オクターブ · { $twist }°
# Before the depth field.
misi-zoom-to = ズーム先
misi-cancel = キャンセル
misi-solve-jump = 計算してジャンプ
misi-solve-jump-hover = この正確な点を、保存された座標を初期値として、指定した深度に必要な精度までニュートン法で求め、そこへ直接ジャンプします。深い指定ほど時間がかかります：計算は任意精度です。
misi-select-hint = 点を選び、深度を指定してジャンプします。各サムネイルの下の乗数の行は、そこへダイブしたときの見え方を示します：ビューが繰り返すまでのオクターブ数と、繰り返しごとのねじれの大きさです。
misi-cancelled = 計算をキャンセルしました。
# $text: what was typed in the depth field.
misi-bad-zoom = 「{ $text }」を倍率として読み取れません（1e50、3.2e120、…のように書いてください）。
misi-no-kp = このエントリには、計算に使える検出済みの(k,p)がありません。
misi-parse-failed = 保存された座標を解析できませんでした。
# A notice after the jump. $name: the point's name; $zoom: the magnification, such as "1e50"; $seconds: how long the solve took, such as "1.2".
misi-zoomed = { $name }にズームしました：{ $zoom }×（{ $seconds }秒）
# Added after the notice above when the iteration count is too low. $have: the fixed iteration count; $typical: what the depth typically needs.
misi-iter-note = 反復回数は{ $have }に固定されています。この深度では通常、約{ $typical }が必要です。ビューが平坦にレンダリングされる場合は、反復回数を上げてください（または反復回数の自動調整を有効にしてください）。
# $name: the point's name; $why: why the solve failed (a technical word).
misi-solve-missed = 深い計算は{ $name }に到達しませんでした：{ $why }。保存された点はそれ自体の深度では正確なままです。より浅い目標を試してください。

## Life (Game of Life): the side panel

# Shown when the graphics card cannot run the Life view.
life-no-compute = このグラフィックスアダプターではライフゲームを実行できません（コンピュートシェーダーがありません）。
# Row label: the pattern list (pattern names stay in English).
life-pattern = パターン
life-open = 開く…
life-open-hover = RLE、.cells、またはLife 1.05/1.06のパターンファイルを開きます。
life-pattern-text = パターンテキスト…
life-pattern-text-hover = パターンを貼り付けます（LifeWikiが示すRLE形式、またはプレーンテキスト）。
life-save = 保存…
life-save-hover = 現在の盤面をRLEとして保存します。
# Row label: the rule list and the typed rule (B3/S23 notation).
life-rule = ルール
# Shown in the rule list when the rule in use is none of the library's.
life-rule-custom = カスタム
life-apply = 適用
# Playback buttons (a pause or play symbol is shown before them).
life-pause = 一時停止
life-play = 再生
life-play-hover = 盤面を実行します（Enter）。
life-step = ステップ
life-step-hover = 1世代進めます（N）。
# Tooltip of the button beside Step, which reads "+16", "+1,024"…: that many generations.
life-stride-hover = この世代数だけ進めます（Shift+N）。
life-reset = リセット
life-reset-hover = 開いたときのパターンに戻します（第0世代）。
life-speed = 速度
# After the speed slider's number: generations a second. Short.
life-speed-unit = 世代/秒
# Row label: how many generations the "+n" button advances.
life-stride = ストライド
# The panel's numbers.
life-generation = 世代
life-population = 個体数
life-tiles = タイル
# $count: the tiles stored; $capacity: how many the GPU pool holds.
life-tiles-of = { $count } / { $capacity }
life-tiles-hover = 保存されている64×64セルのタイル（各4 KiB）と、GPUプールの容量。
# Row label: what a left-drag in the view does (Pan or Draw).
life-mouse = マウス
life-pan = パン
life-pan-hover = ドラッグしてビューを移動します。
life-draw = 描画
# The Draw tool's tooltip; the toolbar's shows it after "Draw (D):", so it starts lowercase.
life-draw-hint = セルをクリックして反転、ドラッグしてペイント：死んだセルから始めたストロークは描画し、生きたセルから始めたストロークは消去します。実行中の盤面は、描いている間は一時停止し、離すと再開します。
life-undo = 元に戻す
life-undo-hover = 直前の編集、塗りつぶし、クリア、ルール変更の前の盤面に戻します（描画中はCtrl+Z）。
life-grid = グリッド線
life-grid-hover = 1セルが8ピクセル以上になるまでズームインしたときのセルの境界線。
# A random fill: "Random" labels the density field, "seed" the seed field after it.
life-random = ランダム
life-density-hover = 塗りつぶしで生きたセルにする割合。
life-seed = シード
life-fill-view = ビューを塗りつぶす
# $cells: the largest side of the filled area, in cells (2,048).
life-fill-view-hover = 表示領域（中心から最大{ $cells } × { $cells }セル）をランダムに塗りつぶします。
life-clear = クリア
life-clear-hover = すべてのセルを消します（元に戻すで復元できます）。

## Life: notices in the panel, and file dialogs

# $path: the file written.
life-saved = { $path }を保存しました
# $error: why (from the system, in English).
life-save-failed = 保存できませんでした：{ $error }
# The universe stopped running. $error: why (in English).
life-stopped = 停止しました：{ $error }
# A pattern file named a rule that does not read. $error: why (in English); $rule: the rule kept, in B3/S23 notation.
life-rule-not-understood = パターンのルールを理解できませんでした（{ $error }）。{ $rule }のままにしました
# $path: the file; $error: why (in English).
life-open-failed = { $path }を開けませんでした：{ $error }
# File-dialog filter names.
life-filter-patterns = ライフゲームのパターン
life-filter-rle = ランレングス符号化パターン

## Life: the toolbar (its buttons are icons; these are their tooltips)

life-toolbar-pause-hover = 盤面を一時停止（Enter）
life-toolbar-play-hover = 盤面を実行（Enter）
# $stride: the generations a Shift+click steps.
life-toolbar-step-hover = 1世代進める（N）。Shift+クリックまたはShift+Nで{ $stride }世代進める
life-toolbar-reset-hover = 開いたときのパターンに戻す（第0世代）
# $hint: the Draw tool's tooltip (life-draw-hint).
life-toolbar-draw-hover = 描画（D）：{ $hint }

## Life: the Pattern text window

life-text-title = パターンテキスト
life-text-intro = パターンを貼り付けてください：RLE（LifeWikiやGollyが示す形式）、プレーンテキストの.cells、またはLife 1.05 / 1.06。
life-text-load = 読み込む
life-text-current = 現在の盤面をRLEで
life-text-current-hover = 最後に読み込んだ盤面でボックスを埋めます。

## L-systems: the side panel

# Row label: the system list (system names stay in English).
lsys-system = システム
lsys-open = 開く…
lsys-open-hover = Fractintの.lファイルまたは.lsysのシステムを開きます。
lsys-edit = 編集…
lsys-edit-hover = システムのテキスト：公理、生成規則、角度。編集して適用します。
lsys-save = 保存…
lsys-save-hover = システムを.lsysテキストファイルとして保存します。
# Tooltip of the "SVG…" button.
lsys-svg-hover = ビューをSVG図面としてエクスポートします：線と図形をベクターとして、表示中の色で。
lsys-follow = 次数をズームに連動
lsys-follow-hover = ズームインすると次数が上がるので、細部が尽きることはありません。
lsys-follow-fixed-hover = このシステムは次数ごとに一定倍で成長しないため、固定の次数で描画されます。
lsys-order = 次数
lsys-angle = 角度
lsys-sweep-hover = 角度を行って戻るように掃引します：回転につれて曲線が変形します
# A small button that puts back the system's own angle after the slider moved it.
lsys-system-angle = システムの角度
lsys-seed = シード
lsys-seed-hover = ランダムな選択が従う値です：同じシードなら同じ植物になります。
lsys-seed-new = 新規
lsys-seed-new-hover = 別の植物：新しいシード。
lsys-line-width = 線の幅
# Row label of a percentage slider: how much of the curve is drawn.
lsys-draw-on = 描画範囲
lsys-draw-on-hover = 曲線のどこまでを、タートルが描く順に描画するか
lsys-draw-on-play-hover = 曲線が自らを描いていく様子
lsys-colour-by = 色分けの基準
# The colourings, in the "Colour by" list.
lsys-colour-position = 曲線に沿った位置
lsys-colour-depth = 枝の深さ
lsys-colour-heading = 向き
# C, < and > are the system's commands that set the colour index: keep them.
lsys-colour-index = カラーインデックス（C、<、>）
lsys-colour-plain = 単色
# The panel's numbers.
lsys-growth = 成長
lsys-growth-built = 次数に応じて語として構築
lsys-growth-built-hover = パラメトリックまたは文脈依存：モジュールの書き換えがその数値や隣接するモジュールによって決まるため、語を1世代ずつ構築し（最大200万モジュール）、倍精度で描画します。次数はズームに連動しません。
# How much the picture grows from one order to the next. $factor: the factor (1.4142).
lsys-growth-factor = 次数ごとに×{ $factor }
lsys-growth-factor-period-2 = 次数ごとに×{ $factor }（次数は2ずつ増加）
lsys-growth-none = なし（固定の次数）
lsys-drawn = 描画
# $drawn: what was drawn (lsys-drawn-segments, lsys-drawn-shapes or lsys-drawn-segments-shapes); $order: the order.
lsys-drawn-at-order = 次数{ $order }で{ $drawn }
# The same, when the walk stopped at its segment budget.
lsys-drawn-at-order-budget = 次数{ $order }で{ $drawn }（予算上限）
# $segments: the line segments drawn.
lsys-drawn-segments = 線分{ $segments }本
# $shapes: the filled shapes drawn.
lsys-drawn-shapes = 塗りつぶし図形{ $shapes }個
# Both counts. $segments: line segments; $shapes: filled shapes.
lsys-drawn-segments-shapes = 線分{ $segments }本、塗りつぶし図形{ $shapes }個
# $ms: the walk's time in milliseconds (12.3).
lsys-drawn-hover = { $ms } msで巡回。ビュー外の部分木は飛ばし、1ピクセル未満のものは1本の線分として描画します
# A word too long to build at the order asked for. $wanted: the order asked for; $built: the order drawn.
lsys-drawn-short = 次数{ $wanted }の語は200万モジュールを超えるため、次数{ $built }で描画しました。
# $order: the deepest order the app draws.
lsys-deepest-order = 描画する最も深い次数（{ $order }）です：これ以上ズームしても細部は増えません。

## L-systems: notices in the panel and the editor, and file dialogs

# $path: the file; $error: why (in English).
lsys-open-failed = { $path }を開けませんでした：{ $error }
# Why a file was not opened (shown after "Could not open …:").
lsys-file-too-large = 4 MiBを超えています
# Why a Fractint .l file was not opened: it holds no entries. `Name { … }` is the file's syntax: keep it.
lsys-no-entries = エントリがありません（.lファイルには`Name {"{"} … {"}"}`形式のエントリが入ります）
# $path: the file written.
lsys-saved = { $path }を保存しました
# $error: why (from the system, in English).
lsys-save-failed = 保存できませんでした：{ $error }
# After File ▸ Export SVG. $path: the file written.
lsys-svg-exported = { $path }をエクスポートしました
# $error: why (from the system, in English).
lsys-svg-failed = エクスポートできませんでした：{ $error }
# File-dialog filter names: opening (any system file) and saving (one system).
lsys-filter-systems = Lシステム
lsys-filter-system = Lシステム

## L-systems: the toolbar

lsys-toolbar-lower-hover = 次数を下げる（固定されます）
lsys-toolbar-raise-hover = 次数を上げる（固定されます）
# Between the − and + buttons, in a fixed-width font. $order: the order, padded to 3 characters so the label keeps its width.
lsys-toolbar-order = 次数 { $order }
# A toggle: the order follows the zoom. Short.
lsys-toolbar-auto = 自動
lsys-toolbar-auto-hover = 次数をズームに連動させます
lsys-toolbar-edit-hover = システムのテキストを編集

## L-systems: the editor window, and a .l file's entries

lsys-editor-title = Lシステム
# The editor's help. The quoted examples ('X = word', 'seed n', 'define R 1.456'…), the commands and "The Algorithmic Beauty of Plants" (a book's title) are the system's syntax and names: keep them as they are.
lsys-editor-help = 1行に1つのキーを書きます：angle（度。/nなら円のn等分）、axiom、生成規則'X = word'、そして必要に応じてheading、draw / move / variables、colour、order。ランダムに選ばれる選択肢を持つ記号は、選択肢ごとに重み付きで1行ずつ書きます：'X (0.3) = word'。'seed n'で植物を選びます。語の中では、{"{"} {"}"}がタートルの経路を塗りつぶし、. が頂点を加えます。パラメトリックおよび文脈依存の生成規則（The Algorithmic Beauty of Plants）：'A(s) : s > 1 = F(s)[+A(s/2)]'、'b < a > c = b'、さらに'define R 1.456'と'ignore +-F'。F(l)はlだけ進み、+(a)はa度回転します。Fractintの.lのエントリはそのまま貼り付けられます。
lsys-editor-apply = 適用
lsys-editor-revert = 元に戻す
lsys-editor-revert-hover = 表示中のシステムのテキスト。
# The window listing a .l file's entries to pick from. $file: the file's name.
lsys-entries-title = { $file }：エントリ
# Tooltip of an entry that does not parse (its error is shown beside it, in English).
lsys-entry-unreadable-hover = このエントリは読み込めません。

## Export: opening a view or location (File ▸ Open view or location…)

# File-dialog filter names.
export-filter-view = Fractadyneのビューまたは位置
export-filter-image = 画像（PNG / EXR）
export-filter-location = 位置（.fdn / .kfr）
# $zoom: the magnification (1.2e148), followed by the magnification sign ×.
export-loaded-location = 位置を読み込みました（{ $zoom }×）
# $zoom: the magnification; $note: what was noteworthy in the file (in English).
export-loaded-location-note = { $zoom }×の位置を読み込みました：{ $note }
# $path: the file.
export-not-location = { $path }はFractadyneの位置ではありません。
export-file-too-large = ファイルが大きすぎます（.fdnの位置ではない可能性があります）。
# $path: the file; $error: why (from the system, in English).
export-read-failed = { $path }を読み込めませんでした：{ $error }
# A Kalles Fraktaler .kfr file did not import. $error: why (in English).
export-import-failed = インポートできませんでした：{ $error }
# $path: an image whose embedded text is not a view.
export-not-view = { $path }にはFractadyneのメタデータがありますが、ビューではありません。
# $path: the image the view was read from.
export-loaded-view = { $path }からビューを読み込みました
# $path: the image; $note: what was noteworthy in it (in English).
export-loaded-view-note = { $path }からビューを読み込みました：{ $note }
export-no-metadata = そのファイルには、Fractadyneのビューのメタデータが埋め込まれていません。

## Export: notices after an export or a snapshot
## "File → Export image…" in them names the menu items (menu-file, menu-file-export).

# The export window's status while the image renders.
export-rendering = レンダリング中…
export-canceled = エクスポートをキャンセルしました。
# $error: why (in English).
export-failed = エクスポートに失敗しました：{ $error }
# $size: the image's size in pixels (1920×1080); $path: the file written.
export-saved = { $size }を保存しました → { $path }
# An L-system export whose walk stopped at its budget. $size: pixels; $segments: the budget (4,000,000); $path: the file.
export-saved-incomplete = { $size }を保存しました（不完全：{ $segments }本の線分の予算を超えました。次数を下げてください） → { $path }
# A two-panel export written as two files. $path: the first of them.
export-saved-two = 2つのファイルを保存しました → { $path }
# $size: pixels; $path: the file written.
export-saved-corrected = { $size }を保存しました（グリッチ補正済み） → { $path }
# $path: the first of the two files.
export-saved-two-corrected = 2つのファイルを保存しました（グリッチ補正済み） → { $path }
# Added after a finished export's notice. $duration: how long it took (8.3s, 1m 04.0s).
export-elapsed = （所要時間 { $duration }）
export-preparing-deep = 深いエクスポートを準備中：参照軌道を構築しています（しばらくかかることがあります）…
# $size: pixels (5120×4035); $ss: the supersampling factor; $iterations: the iteration count.
export-large = 大きなエクスポート（{ $size }、{ $ss }×スーパーサンプリング、{ $iterations }回の反復）：バックグラウンドでレンダリングしています。このサイズのエクスポートではグリッチ補正は省略されます…
export-life-unavailable = ライフゲームのビューの画像エクスポートはまだ使用できません。スクリーンショットを使うか、ファイル ▸ ライフゲームのパターンを保存を使ってください。
export-already-running = エクスポートはすでに実行中です。ファイル → 画像をエクスポート…で進行状況を確認できます。
# $size: the image's size in pixels.
export-snapshot-background = スナップショット：{ $size }をバックグラウンドでレンダリングしています。ファイル → 画像をエクスポート…で進行状況の確認やキャンセルができます。
export-no-gpu = GPUを使用できません
# A screen-capture snapshot. $size: pixels; $path: the file written.
export-snapshot-saved = 画面のスナップショット{ $size }を保存しました → { $path }
# $error: why (in English).
export-snapshot-failed = スナップショットに失敗しました：{ $error }

## The colour picker (the popup of a colour swatch)

# The third row's label: the colour as hexadecimal bytes. Short (about 6 characters).
picker-row-hex = 16進
picker-hex-hover = クリックして16進値を入力
picker-cancel = キャンセル
picker-cancel-hover = このピッカーを開いたときの色に戻します
picker-ok = OK
picker-ok-hover = この色を使用
# Starts the screen eyedropper (an icon is shown before it).
picker-pick = スポイト
picker-pick-hover = 画面上のどこからでも色を取得します
# Tooltip of the disabled Pick button (in the picker and the gradient editor) on Linux and macOS.
picker-eyedropper-unsupported = 画面上のほかの場所から色を取得するには、プラットフォームの画面キャプチャAPIが必要です。それがあるのはWindows版だけです。LinuxではX11への依存が必要になり、Waylandでは独自の許可を求めるポータルが必要です。

## The fractal families: what each one is (the info panel's "About" section and the pickers' hover text).
## The formulas themselves (z -> z^2 + c) are written in code and stay as they are.

fractal-mandelbrot-about = 代表的な脱出時間フラクタルで、0の軌道が有界にとどまるcの集合です。その境界は無限に入り組んでいます。
fractal-multibrot3-about = Multibrot集合：Mandelbrotの構成を高いべきで行ったものです。べきdで(d-1)回の回転対称性になります。
fractal-multibrot4-about = べき4のMultibrot：3回対称で、幅の広いバルブ。
fractal-multibrot5-about = べき5のMultibrot：4回対称。
fractal-tricorn-about = Tricorn（Mandelbar）：各ステップでzの共役を取ります。この反正則な写像から、三つの角を持つ形が生まれます。
fractal-burning-ship-about = 2乗する前にzの各部分の絶対値を取ります。その結果は燃える船に似ています。
fractal-celtic-about = Burning Shipの仲間で、z^2の実部だけの絶対値を取り、ケルト結び目やハートのような模様を生みます。
fractal-buffalo-about = z^2の両方の成分の絶対値を取る、絶対値の変種です。
fractal-phoenix-about = Phoenixは1つ前の反復値も使い、炎のような糸状の模様を生みます。JuliaモードでJulia形も試してみてください。
fractal-newton-about = z^3 = 1に対するニュートン法の根を求める反復で、各点がどれだけ速く収束するかで色を付けます。（脱出ではなく）収束のフラクタルです。
fractal-multibrot6-about = べき6のMultibrot：5回対称。
fractal-multibrot7-about = べき7のMultibrot：6回対称で、バルブは細くなって円に近づきます。
fractal-multibrot8-about = べき8のMultibrot：7回対称。
fractal-burning-ship3-about = べき3のBurning Ship：絶対値を取ってから3乗します。
fractal-burning-ship4-about = べき4のBurning Ship。
fractal-burning-ship5-about = べき5のBurning Ship。
fractal-tricorn3-about = べき3のTricorn（Mandelbar）：4回対称。
fractal-tricorn4-about = べき4のTricorn：5回対称。
fractal-tricorn5-about = べき5のTricorn：6回対称。
fractal-celtic3-about = べき3のCelticの折り返し：z^3の実部の絶対値。
fractal-celtic4-about = べき4のCelticの折り返し。
fractal-celtic5-about = べき5のCelticの折り返し。
fractal-buffalo3-about = べき3のBuffalo：z^3の両方の部分の絶対値。
fractal-buffalo4-about = べき4のBuffalo。
fractal-buffalo5-about = べき5のBuffalo。
fractal-custom-about = Fractint形式の式で書かれた数式です（フラクタル ▸ カスタム数式…）。ほとんどの数式は組み込みの族と同様に摂動によるディープズームができます。できるかどうかは数式ダイアログに表示されます。
fractal-life-about = 無限の平面上のConwayのライフゲームとその仲間：Generationsルール、非トータリスティック（Hensel）ルール、パターンファイル。再生、ステップ実行、描画ができます。
fractal-lsystem-about = Lindenmayerシステム：書き換え文法からタートルが描く曲線、空間充填曲線、島、植物。次数がズームに連動するので、細部が尽きることはありません。Fractintの.lファイルをそのまま開けます。

# Shown where the other families show their formula (z -> z^2 + c).
fractal-custom-formula = z -> 自分で書く数式
# Shown where the other families show their formula. "B3/S23" is the rule's notation: keep it.
fractal-life-formula = B3/S23：生きた隣接セルが3つで誕生、2つか3つで生存
# Shown where the other families show their formula. "F = F+F--F+F" is the grammar's notation: keep it.
fractal-lsystem-formula = F = F+F--F+F：各記号を書き換え、タートルが描画

# The last paragraph of the picker's hover text for a family that renders only on the direct path.
fractal-no-deep-zoom = directレンダリングのみ：この族にはディープズームがありません。

## Names of settings, shown in the Settings menu and the welcome window.

theme-dark = ダーク
theme-light = ライト
# The update track that follows only stable releases.
update-track-stable = 安定版
# The update track that also offers pre-releases (betas).
update-track-beta = ベータ版（プレリリース）

## The status bar (its hover texts; the readouts follow in their own section)

status-period-hover = ジャンプしたミニブロットの周期です（Mを押すか、位置へ移動 ▸ 特徴点へ移動）。ビューがその特徴点にある間は表示され、離れると消えます。

## The playback transport: the player over the view while a tour plays.

# $close: the close button's icon, as drawn on that button.
play-finished-hover = 終了：スクラブで戻るか、{ $close }で閉じます
# $kind: "benchmark" or "script" (the variants hold the wording); $name: the tour's name.
play-playing-hover =
    { $kind ->
        [benchmark] ベンチマーク「{ $name }」を再生中
       *[other] スクリプト「{ $name }」を再生中
    }
play-restart = 最初からやり直す
play-back = 10秒戻る
play-again = 最初からもう一度再生
play-resume = 再開
play-pause = 一時停止
play-stop = 停止して最初に巻き戻す
play-forward = 10秒進む
play-speed-hover = 再生速度（クリックで0.5× / 1× / 2× / 4×を切り替え）
play-render = このスクリプトをフレームシーケンスにレンダリング…
play-render-none = 組み込みのベンチマークには、レンダリングするスクリプトファイルがありません
play-open = 別のスクリプトを再生…
play-loop = ツアーが最後まで来たら繰り返す
play-close = プレーヤーを閉じて、自分のビュー設定に戻す
# A notice beside the scrub bar while the renderer holds the tour's clock. Keep it short.
play-waiting = 細部を待機中
# "[playback] pace" is a setting in the script file: keep it as written.
play-waiting-hover = レンダラーがこのビューを解像している間（参照軌道の構築 / 反復予算の上昇）、ツアーの時計は一時停止しています。再生は自動的に再開します。スクリプトの[playback] paceを参照してください。
play-scrub = ドラッグしてツアー内をスクラブ
# $behind: how far the script clock is behind real time, as minutes:seconds (1:05).
play-scrub-behind =
    ドラッグしてツアー内をスクラブ
    スクリプトの時計は実時間より{ $behind }遅れています（レンダラーが追いつけるよう、ペーサーがツアーを遅くしました）。上の目盛りは、実時間どおりに再生した場合の位置を示します。

## The autopilot's notices (toasts). Where a notice has two lines, the first is the verdict and the second what to do about it.

autopilot-on-detail = オートパイロットをオンにしました：細部に向かってダイブ中（何か入力すると停止）
autopilot-off = オートパイロットをオフにしました
autopilot-misi-needs-mandelbrot =
    オートパイロット：Misiurewicz点の目標にはMandelbrot集合が必要です
    代わりに細部に向かってダイブします
autopilot-on-misi =
    オートパイロットをオンにしました：最寄りのMisiurewicz点を探しています…
    何か入力すると停止します
# $seconds: how long the search has run, in whole seconds.
autopilot-misi-finding =
    オートパイロット：最寄りのMisiurewicz点を探しています…
    { $seconds }秒 · 何か入力すると停止します
autopilot-misi-not-found =
    オートパイロット：このビューの近くにMisiurewicz点が見つかりません
    検出機能がこの深度まで届かない可能性があります。目標を「細部」にしてみてください
# ($preperiod,$period) names the point, as mathematicians write it: keep the notation. $widths: a power of two (2^$widths screen widths).
autopilot-misi-off-screen =
    オートパイロット：最寄りのMisiurewicz点({ $preperiod },{ $period })は画面外にあります
    画面の幅の約2^{ $widths }倍離れています。螺旋の中心に近づいてください
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-not-solved =
    オートパイロット：このビューの近くのMisiurewicz点({ $preperiod },{ $period })を計算できませんでした
    目標を「細部」にしてみてください
autopilot-misi-refused = オートパイロット：Misiurewicz点の計算が拒否されました（不正な組）
autopilot-misi-stopped = オートパイロット：Misiurewiczソルバーが予期せず停止しました
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-diving =
    オートパイロット：Misiurewicz ({ $preperiod },{ $period })へダイブ中
    何か入力すると停止します
# ($preperiod,$period) names the point: keep the notation. $octaves: how often the structure repeats (2.37); $twist: its turn in degrees, signed (+12.5).
autopilot-misi-diving-repeat =
    オートパイロット：Misiurewicz ({ $preperiod },{ $period })へダイブ中
    構造は{ $octaves }オクターブごとに繰り返し、ねじれ{ $twist }° · 何か入力すると停止します
# $limit: the magnification, as written (1e300).
autopilot-dive-limit = オートパイロット：ダイブ上限に達しました（約{ $limit }×）
# $have: the fixed iteration count; $typical: what this depth typically needs.
autopilot-dead-end-short =
    オートパイロットを停止しました：狙う対象がもうありません。反復回数が{ $have }に固定されていますが、この深度では通常、約{ $typical }が必要です
    反復回数の自動調整を有効にするか、回数を上げてください
# $limit: the fixed iteration count.
autopilot-dead-end-cap =
    オートパイロットを停止しました：狙う対象がもうありません。ビュー内のすべてが固定の反復上限{ $limit }に達しています
    反復回数の自動調整を有効にするか、回数を上げてください
autopilot-dead-end = オートパイロット：前方に細部がありません（停止）

## Tours (camera scripts)

# The name of the file type in the file dialog's filter list.
script-file-filter = Fractadyneスクリプト（TOML）
# $name: the script's name.
script-finished = スクリプトが終了しました：「{ $name }」
# The title of the notice that follows (the file's path and the error, in English).
script-load-failed = スクリプトを読み込めませんでした

## Installing a language file

# The name of the file type in the file dialog's filter list.
language-file-filter = Fractadyneの言語ファイル

## The notice when the graphics driver's shader cache stops keeping shaders. The text is shown in a fixed-width font with its line breaks as written: break the lines where they read well.

driver-cache-title = グラフィックスドライバーがシェーダーを保持していません
# $count: how many shaders compiled again; $pipeline: the slowest one's name (a technical id: keep it); $seconds: its time in seconds (7.5).
driver-cache-body =
    このセッションで、グラフィックスドライバーがFractadyneの
    シェーダーのうち{ $count }個を再びコンパイルしました。どれも
    以前にコンパイル済みだったものです（最も遅いもの：
    { $pipeline }、{ $seconds }秒）。ドライバーのシェーダー
    キャッシュがコンパイル済みのシェーダーを保持しておらず、
    多くの場合、その原因はキャッシュがいっぱいなことです。
    これが解消されるまで、エクスポート、深いビュー、Fractadyneの
    起動のたびに、コンパイルに数秒かかることがあります。
# $folder: the cache folder's path; $size: its size in MB (1024). "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia-folder =
    解決するには：Fractadyneを閉じてから、次のフォルダーの
    中身を削除してください。
      { $folder }   （{ $size } MB）
    ドライバーがキャッシュを再構築します。再びいっぱいに
    ならないようにするには、NVIDIA コントロール パネル >
    3D 設定の管理 > シェーダー キャッシュ サイズ を
    大きくしてください。
# "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia =
    解決するには：NVIDIA コントロール パネル > 3D 設定の管理 >
    シェーダー キャッシュ サイズ を大きくするか、ドライバーの
    シェーダーキャッシュを消去してください。
# "AMD Software > Graphics > Advanced > Reset Shader Cache" names AMD's own settings: use the names AMD's software shows in this language.
driver-cache-fix-amd = 解決するには：AMD Software > グラフィックス > 詳細設定 > シェーダーキャッシュのリセット。
driver-cache-fix-other = 解決するには：グラフィックスドライバーのシェーダーキャッシュを消去してください。

## Why a language file could not be installed: each ends the sentence "{ $file } is not a usable language file: …".

language-install-not-utf8 = UTF-8のテキストではありません
language-install-too-large = 大きすぎます
# "fractadyne-<language>.ftl" is the file name to use: keep it as written.
language-install-no-tag = どの言語のファイルかが示されていません（fractadyne-<language>.ftlという名前にしてください）
language-install-english = 英語は組み込まれています
language-install-empty = このバージョンのメッセージを1つも翻訳していません
language-install-no-folder = 設定フォルダーがありません

## The notice after a session that did not end cleanly, when its iteration setting was set aside.

# $base: the iteration count automatic iterations start from.
quarantine-setting-auto = 基準値{ $base }からの自動の反復回数
# $count: the fixed iteration count.
quarantine-setting-fixed = { $count }回の反復
# $end: "device" when the session lost its graphics device, "unclean" otherwise (the variants hold the wording). $depth: the magnification, as written (1.2e148). $setting and $restored: an iteration setting (quarantine-setting-auto or -fixed).
quarantine-notice =
    { $end ->
        [device] 前回のセッションは、{ $setting }に設定された状態で、{ $depth }×でグラフィックスデバイスを失いました。同じ負荷にそのまま戻ってしまわないよう、このセッションは代わりに{ $restored }で開始しました。
       *[other] 前回のセッションは、{ $setting }に設定された状態で、{ $depth }×で正常に終了しませんでした。同じ負荷にそのまま戻ってしまわないよう、このセッションは代わりに{ $restored }で開始しました。
    }

## The second graphics card's status line, under its setting in the control panel.

gpu2-state-off = オフ：このウィンドウのグラフィックスカードがすべてをレンダリングします。
gpu2-state-starting = 開始中…
# $name: the card's name (and its graphics API).
gpu2-state-running = 使用中：{ $name }
# $why: the error, as the graphics driver reports it.
gpu2-state-failed = 開始できませんでした：{ $why }
# $name: the card's name.
gpu2-state-lost = 停止：{ $name }が失われました。ここからは1枚のカードで処理します。再試行するには、もう一度選んでください。
# $state: one of the lines above. "--worker-gpu" is a command-line option: keep it.
gpu2-state-pinned = { $state }（--worker-gpuで設定）

## The custom-formula dialog: how deep the formula renders (one line under the formula).

formula-depth-deep = 摂動によるディープズームです。組み込みのフラクタルと同じく、1e28x以降は拡張範囲を使います（すべてのステップを反復：カスタム数式にはまだ級数近似もBLAもありません）。
# $why: what the formula has that keeps it off the deep-zoom path (one of the formula-depth-why messages below).
formula-depth-direct = directレンダリング（{ $why }にはまだディープズームの形式がありません）：cの単精度での1ステップが1ピクセルにまたがるまで鮮明です。約1e4xから1e5xまでで、原点から離れるほど浅くなります。
formula-depth-why-init = 初期化セクション
formula-depth-why-bailout = 独自の脱出判定
formula-depth-why-variable = ステップ間で保持される変数
formula-depth-why-previous = 1つ前の反復値
formula-depth-why-comparison = 比較またはifブロック
# floor, ceil, trunc and round are functions of the formula language: keep them.
formula-depth-why-rounding = 丸め（floor、ceil、trunc、round）
# "maxit" is a word of the formula language: keep it.
formula-depth-why-maxit = maxit
formula-depth-why-power = 指数が変化するべき
formula-depth-why-perturbed = すでに摂動化されたプログラム

## Image and frame sizes: the presets described in words (the others are standard names: HD 720p, WUXGA, DCI 4K).

# $size: the pixel size (2048×2048).
size-preset-square = 正方形 — { $size }
# $ratio: the aspect ratio (9:16); $size: the pixel size.
size-preset-portrait = 縦長 { $ratio } — { $size }
# $ratio: the aspect ratio (32:9); $size: the pixel size.
size-preset-super-ultrawide = スーパーウルトラワイド { $ratio } — { $size }

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
fractal-family-custom = カスタム
# Conway's Game of Life.
fractal-family-life = ライフゲーム
fractal-family-lsystem = Lシステム
# A family at a power: "Multibrot 3", "Burning Ship 4". $family: a name above; $power: the power.
fractal-with-power = { $family } { $power }

## Names of choices: colouring methods, orbit traps, palette animation, snapshots, angle units.

color-method-smooth = スムーズ反復
color-method-stripe = ストライプ平均
color-method-triangle = 三角不等式
color-method-trap = 軌道トラップ
color-method-distance = 距離推定
color-method-decomposition = 分解
# The shape an orbit trap catches the orbit with.
trap-point = 点
trap-cross = 十字
trap-circle = 円
# Palette animation: none.
palette-anim-off = オフ
palette-anim-forward = 順方向
palette-anim-reverse = 逆方向
palette-anim-pingpong = 往復
# Palette animation: a new random gradient every so often.
palette-anim-random = ランダムなグラデーション
# What the Snapshot button does.
snapshot-mode-ask = 毎回確認
snapshot-mode-screen = 画面キャプチャ
snapshot-mode-render = フルレンダリング
# The unit of an angle typed into Go to location's polar fields.
angle-degrees = 度
angle-radians = ラジアン
angle-turns = 回転

## Report an issue: the choices. (The report itself is sent in English.)

issue-kind-crash = アプリケーションのクラッシュ / フリーズ
issue-kind-rendering = 誤ったレンダリング
issue-kind-performance = パフォーマンスの問題
issue-kind-ui = UI / 使いやすさの問題
issue-kind-feature = 機能の要望
issue-kind-other = その他
severity-low = 低
severity-medium = 中
severity-high = 高
severity-blocking = 作業不能
repro-always = 常に
repro-sometimes = ときどき
repro-once = 1回だけ発生
repro-cannot = 再現できない

## The benchmark's depths and the diagnostics' tests.

# $zoom: the dive's final magnification (1e32).
bench-depth-standard = 標準（{ $zoom }×）
# $zoom: the dive's final magnification (1e48).
bench-depth-ultra = 超深度（{ $zoom }×）
diag-test-selftest = セルフテスト
diag-test-uitest = UIテスト
diag-test-gputest = GPU演算チェック

## The status bar's readouts: each word is followed by a value in a slot of fixed width. Keep the words short (one word each).

status-center = 中心
status-cursor = カーソル
status-zoom = ズーム
# Short for iterations.
status-iter = 反復
# The period of the minibrot jumped to.
status-period = 周期
# A Game of Life view's scale (followed by "1 px = 4 cells" or "8 px / cell").
status-scale = 縮尺
# A Game of Life view's generation. Short.
status-generation = 世代
# A Game of Life view's population. Short.
status-population = 個体
# An L-system's order (how many times its rules were applied).
status-order = 次数
# How many segments and shapes an L-system drew.
status-drawn = 描画
# $value: how many cells one pixel covers (4, 9.99e300). "px" is the unit symbol: keep it.
life-scale-cells = 1 px = { $value }セル
# $value: how many pixels one cell covers (8). "px" is the unit symbol: keep it.
life-scale-pixels = { $value } px / セル

## The status bar's limit diagnostic: a short label (after a warning sign), and what it means when hovered.

# The view is as deep as this location can be rendered. Short.
status-limit-depth = 深度限界
# Every pixel ran out of iterations. Short.
status-limit-exhausted = 反復切れ
# Pixels are capped at the iteration budget. Short.
status-limit-capped = 反復上限
# $pixels: how many pixels wide one step of the coordinates is; $sharp_to: the magnification the view is sharp to (written out, 1,234,567).
status-limit-direct-detail = このビューはGPU上で単精度で直接レンダリングされており、ここでは座標の1ステップが{ $pixels }ピクセル幅になります：隣り合うピクセルが同じ点を計算するため、画像がブロック状に崩れます。この位置では約{ $sharp_to }×まで鮮明です。この数式にはまだディープズーム（摂動）の処理経路がないため、これ以上ズームしても細部は増えません。
# $samples: the GPU buffer's capacity in orbit samples; $iterations: the iterations pixels are limited to.
status-limit-depth-detail = 参照軌道が脱出しないままGPUバッファの上限（{ $samples }サンプル）に達したため、ここでのピクセルは約{ $iterations }回の反復に制限され、このビューではライブでもエクスポートでも、より細かい細部は計算できません。これがこの種の位置での現在の深度の限界です。
# $budget: the largest iteration budget this view can be given.
status-limit-exhausted-detail = このビューに与えられる最大の予算（{ $budget }）でも、すべてのピクセルが反復を使い切りました。ビューがここで集合の内側にある（深いところでは普通のことです）か、アプリが現在提供できる以上の反復回数が必要かのどちらかです。どちらの場合も画像は同じなので、予算は安価なものに戻されました。
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations" is the setting's name in the Rendering panel.
status-limit-capped-max = { $budget }の予算でピクセルの{ $percent }%が反復を使い切り、脱出するピクセルもちょうどその上限で終わっています：ビューの解像が不足しています。「反復回数」はすでに最大なので、このビューにはアプリが現在提供できる以上のものが必要です。
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations" and "Rendering" are the names of a setting and a panel section.
status-limit-capped-raise = { $budget }の予算でピクセルの{ $percent }%が反復を使い切り、脱出するピクセルもちょうどその上限で終わっています：ビューの解像が不足しています。解像するには、「反復回数」（コントロール ▸ 品質）を上げてください。
