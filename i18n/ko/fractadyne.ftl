### Fractadyne user interface: Korean (한국어).
### A draft: machine translation (Claude, by Anthropic) of the English file, 2026-10-23, not yet
### reviewed by a native speaker. Corrections are welcome as pull requests (see i18n/README.md).

## Language data: read by Fractadyne, not shown as messages.

# This language's own name, shown in the language menu.
language-name = 한국어
# The same name in English, shown beside it in the menu.
language-english-name = Korean
# The writing system the app must be able to draw (an ISO 15924 code: Latn, Cyrl, Jpan, Hans, Khmr).
language-script = Kore
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
language-more = 다른 언어…
# The Languages window's title.
language-window-title = 언어
# The Languages window's introduction.
language-window-intro = 다른 언어로 보는 Fractadyne 인터페이스입니다. 언어 하나는 파일 하나이며, 릴리스에 포함되어 있거나 여기에서 설치합니다. 좌표, 파일, 명령줄은 어느 언어에서나 영어로 유지됩니다.
# Column headers of the installed-language list.
language-col-language = 언어
language-col-translated = 번역됨
language-col-from = 출처
# Where a language comes from.
language-source-builtin = 내장
language-source-package = 이 릴리스
language-source-user = 설치됨
language-source-dev = 작업 폴더
# $percent: how much of the interface the file translates, 0–100.
language-coverage = { $percent }%
# $count: messages in the file that failed the checks and show in English instead.
language-dropped = 확인이 필요한 메시지 { $count }개
# Buttons of the Languages window.
language-install = 언어 파일 설치…
language-install-hover = Fractadyne 언어 파일(.ftl)을 선택합니다. 파일을 검사한 뒤 설정의 언어 폴더로 복사하며, 업데이트 후에도 그곳에 유지됩니다.
language-open-folder = 언어 폴더 열기
language-reload = 다시 읽기
language-reload-hover = 언어 파일을 다시 읽습니다(파일을 편집하거나 추가한 뒤에).
language-use = 사용
# Notices after installing a file. $name: the language's own name; $file: the file's name.
language-installed = { $file }에서 { $name }을(를) 설치했습니다.
# $file: the file's name; $why: what was wrong with it.
language-install-failed = { $file }은(는) 사용할 수 있는 언어 파일이 아닙니다: { $why }
# Shown when a language file cannot be loaded and English is used instead. $name: the language.
language-load-failed = { $name }을(를) 불러올 수 없어 Fractadyne을 영어로 표시합니다: { $why }
# A language whose writing system needs a font this computer does not have (Japanese or Chinese without a system font). $name: the language, in its own name.
language-font-missing = 이 컴퓨터에서 { $name }을(를) 표시할 수 있는 글꼴을 찾지 못해 표시할 수 없습니다. { $name } 글꼴(예: 시스템의 언어 팩)을 설치한 뒤 다시 선택하십시오.
# Shown after a draft translation's name in the language menu and the Languages window. Short.
language-draft = 초안
language-draft-hover = 초안: 이 번역은 아직 원어민의 검토를 받지 않았습니다. 수정 사항은 해당 언어 파일에 대한 풀 리퀘스트로 환영합니다.
# The heading over the decimal-mark choice, in the language menu and the Languages window.
language-decimal-mark = 소수점 기호
language-decimal-mark-hover = 숫자를 쓰는 방식입니다(시간, 크기, 개수). 좌표와 배율은 파일과 다른 프로그램이 읽을 수 있도록 항상 점을 사용합니다.
# $sample: a number written with this language's own punctuation (1,234.5).
language-decimal-language = 언어의 표기 방식({ $sample })
# $sample: a number written with a decimal point (1,234.5).
language-decimal-point = 점({ $sample })
# $sample: a number written with a decimal comma (1.234,5).
language-decimal-comma = 쉼표({ $sample })

## Asked once when the system's language changes

# The window's title.
language-changed-title = 언어
# $new: the system's new language, in its own name.
language-changed-question = 시스템 언어가 { $new }(으)로 바뀌었습니다. Fractadyne에서도 사용하시겠습니까?
# Buttons. $name: a language's own name.
language-changed-use = { $name } 사용
language-changed-keep = { $name } 유지

## The welcome window (first run)

welcome-title = Fractadyne에 오신 것을 환영합니다
welcome-intro = Mandelbrot 집합과 그 친척들을 위한 딥 줌 탐색기입니다. 시작하는 데 필요한 몇 가지 조작법:
# The language row, at the top of the window.
welcome-language = 언어
# Controls: the key or gesture on the left, what it does on the right.
welcome-key-drag = 드래그
welcome-key-drag-does = 뷰 이동
welcome-key-scroll = 스크롤
welcome-key-scroll-does = 커서 쪽으로 확대 / 축소
welcome-key-space = Space 누르고 있기
welcome-key-space-does = 부드러운 연속 줌(Shift+Space는 축소)
# Shown after the click-zoom toolbar icon.
welcome-key-click = 켠 뒤 클릭
welcome-key-click-does = 클릭한 지점으로 다이브(도구 모음)
welcome-key-m-does = 가장 가까운 미니브로트로, 그 크기에 맞춰 이동
welcome-landmarks = 랜드마크로 이동해 시작하기:
welcome-theme = 테마
welcome-watermark = “Fd” 마크 표시
welcome-watermark-hover = 뷰 오른쪽 아래의 작은 브랜드 마크입니다. 끄면 라이브 뷰와 내보낸 이미지에서 사라집니다.
welcome-updates = 업데이트
welcome-check-updates = 업데이트 확인
welcome-check-updates-hover = 앱이 시작될 때 GitHub에 최신 릴리스를 묻습니다. 아무것도 업로드하지 않습니다.
welcome-files = 파일
welcome-no-config = (설정 디렉터리 없음)
welcome-open = 열기
welcome-rendering-on = 렌더링 장치
welcome-open-help = 전체 도움말 열기  (F1)
# $icon: the Settings icon.
welcome-more-in-settings = 파일 → { $icon } 설정에서 더 보기
welcome-get-started = 시작하기

## Menus: shared

# A percentage in a menu (UI scale). $value: the number.
common-percent = { $value }%
# A state shown after a menu item's name, in brackets.
menu-state-connected = 연결됨
menu-state-rendering = 렌더링 중
menu-state-running = 실행 중

## Navigate ▸ Bookmarks (also the toolbar's star)

menu-bookmarks = 북마크
menu-bookmarks-add = 현재 뷰 추가
menu-bookmarks-manage = 관리…

## The fractal picker (Fractal menu and the toolbar's dropdown)

menu-family-powers = 고차 거듭제곱
menu-family-automata = 오토마타
menu-family-lsystems = L-시스템
menu-fractal-custom = 사용자 정의 수식…
menu-fractal-library = 수식 라이브러리…
menu-fractal-library-hover = 저장된 사용자 정의 수식: 적용, 편집, 가져오기, 내보내기
# The "Show" group: which picture of the formula is drawn. $name: the formula's name (Mandelbrot, Burning Ship…).
menu-show = 표시
menu-show-set = { $name } 집합
menu-show-set-hover = 매개변수 평면 — { $name } 집합 그 자체입니다.

    이 평면의 모든 점은 각각 서로 다른 Julia 집합이며, 이것은 그 모두를 담은 지도입니다.
menu-show-julia = Julia 집합
menu-show-julia-hover = { $name } 집합의 한 점을 그 자체의 그림으로 그린 것입니다 — 그 점을 고정했을 때 얻는 모양입니다.

    “둘 다, 연결”에서 지도 위에 커서를 올려 점을 고릅니다. 그곳을 클릭하면 고정됩니다.
menu-show-julia-none = { $name }에는 자유 매개변수가 없으므로 Julia 집합이 없습니다.
menu-show-both = 둘 다, 연결
menu-show-both-hover = 왼쪽에는 { $name } 집합을, 오른쪽에는 커서가 가리키는 점의 Julia 집합을 표시합니다.

    커서를 지도 위로 움직이며 Julia가 바뀌는 모습을 보십시오 — 둘의 관계를 가장 빨리 알 수 있는 방법입니다.
menu-show-both-none = { $name }에는 짝지을 Julia 집합이 없습니다.

## File menu

menu-file = 파일
menu-file-open-view = 뷰 또는 위치 열기…
menu-file-gallery = 갤러리…
menu-file-export = 이미지 내보내기…
menu-file-snapshot = 스냅샷
menu-file-open-life = Life 패턴 열기…
menu-file-open-life-hover = RLE, .cells 또는 Life 1.05/1.06 파일을 Life 세계로 표시합니다.
menu-file-life-text = Life 패턴 텍스트…
menu-file-life-text-hover = 패턴을 텍스트로 붙여 넣습니다(LifeWiki가 제공하는 RLE 형식).
menu-file-save-life = Life 패턴 저장…
menu-file-save-life-hover = 지금 상태의 세계를 RLE로 저장합니다.
menu-file-open-lsystem = L-시스템 열기…
menu-file-open-lsystem-hover = Fractint .l 파일(그 안의 항목 중에서 선택) 또는 .lsys 시스템입니다.
menu-file-save-lsystem = L-시스템 저장…
menu-file-save-lsystem-hover = 시스템의 텍스트를 .lsys 파일로 저장합니다.
menu-file-export-svg = SVG 내보내기…
menu-file-export-svg-hover = 뷰를 SVG 도면으로 내보냅니다: 선과 도형을 벡터로, 뷰의 색상 그대로.
menu-file-render-client = 렌더 클라이언트…
menu-file-render-client-hover = 이 컴퓨터의 GPU를 렌더 팜에 빌려줍니다: 컨트롤러(다른 컴퓨터의 도구 ▸ 팜에서 렌더링…)에 연결해 그 컨트롤러를 위해 프레임을 렌더링합니다.
menu-file-settings = 설정
menu-file-reset = 애플리케이션 상태 초기화…
menu-file-reset-hover = 저장된 모든 상태(세션, 북마크, 썸네일)를 삭제하고 새로 시작합니다. 먼저 확인을 요청합니다.
menu-file-quit = 종료

## File ▸ Settings

menu-settings-fps = 프레임 속도 제한
menu-settings-fps-uncapped = 제한 없음
# $fps: frames per second.
menu-settings-fps-n = { $fps } FPS
menu-settings-ui-scale = UI 크기(글꼴 크기)
menu-settings-theme = 테마
menu-settings-updates = 업데이트
menu-settings-updates-stable-hover = 최신 안정 릴리스
menu-settings-updates-beta-hover = 사전 릴리스를 포함한 최신 빌드
menu-settings-updates-on-launch = 시작할 때 업데이트 확인
menu-settings-updates-on-launch-hover = 끄면 도움말 → 업데이트 확인에서 직접 확인합니다.
# Followed by its shortcut, (Ctrl+S).
menu-settings-snapshot = 스냅샷
menu-settings-snapshot-ask-hover = 누를 때마다 선택지 표시
menu-settings-snapshot-screen-hover = 화면에 보이는 그대로 뷰 저장(즉시, PNG)
menu-settings-snapshot-render-hover = 내보내기 설정으로 백그라운드에서 전체 렌더링(깊은 뷰에서는 오래 걸릴 수 있음)
menu-settings-refcache = 기준 궤도 캐시…
menu-settings-refcache-hover = 디스크에 보관한 딥 줌 기준 궤도로, 다시 찾은 위치를 몇 초 만에 렌더링합니다. 저장 위치, 크기, 한도, 그리고 비우는 방법.

## Fractal menu

menu-fractal = 프랙털

## View menu

menu-view = 보기
menu-view-zoom-home = 전체 뷰로 축소
menu-view-zoom-home-hover = 전체 홈 뷰로 축소(애니메이션)
menu-view-reset = 기본 뷰로 초기화
menu-view-reset-hover = 프랙털의 기본 뷰로 초기화(즉시)
# Followed by its shortcuts.
menu-view-undo = 뷰 실행 취소
menu-view-redo = 뷰 다시 실행
menu-view-control-panel = 컨트롤 패널
menu-view-control-panel-hover = 오른쪽 컨트롤 패널을 표시하거나 숨깁니다.
menu-view-minimap = 미니맵 개요
menu-view-minimap-hover = “현재 위치” 표시와 줌 깊이가 나타나는 작은 홈 뷰 개요를 표시합니다. 클릭하면 해당 영역으로 이동합니다.
menu-view-perf-panel = 성능 패널
menu-view-timestamp = 타임스탬프 표시
menu-view-timestamp-hover = 뷰 위에 큰 경과 시간 시계를 그립니다. 로그가 모든 줄에 찍는 것과 같은 “+12.345s”를 표시하므로, 움직일 때만 드러나는 문제를 녹화한 화면을 로그와 프레임 단위로 맞춰 볼 수 있습니다. 기본값은 꺼짐입니다.
menu-view-zoom-target = 자동 줌 목표 표시
menu-view-zoom-target-hover = 자동 줌이 실행되는 동안, 줌이 향하는 뷰 영역(몇 번 더 확대하면 화면을 채울 부분)의 윤곽을 그리고, 그 모서리에서 화면 모서리까지 선을 긋고, 향하고 있는 점에 고리를 표시합니다. 기본값은 꺼짐입니다.
menu-view-live-panel = 라이브 렌더 패널
menu-view-live-panel-hover = 라이브 렌더러가 일하는 모습을 보여 주는, 뷰 위의 전면 패널입니다: 각 프레임이 디스패치한 작업, 화면의 그림이 얼마나 오래되었는지, 반복 구간을 따라가는 순회, 기준 궤도 계산, 그리고 그 아래의 하드웨어. B 키로 크기를 차례로 바꾸고, 모서리의 ×로 닫습니다. 아무 모서리로나 끌어다 놓을 수 있으며, 계기 위에 포인터를 올리면 그 의미와 정상일 때와 문제가 있을 때의 모습을 알려 줍니다.
# The live render panel's sizes.
menu-view-live-off = 끄기
menu-view-live-off-hover = 패널 없음.
menu-view-live-strip = 스트립
menu-view-live-strip-hover = 닉시관 표시와 램프 뱅크.
menu-view-live-panel-size = 패널
menu-view-live-panel-size-hover = 계기: 순회 다이얼, 그림 지연, 타일, GPU 패스, 깊이 다이얼, 램프.
menu-view-live-full = 전체
menu-view-live-full-hover = 패널과 기관실: 스텝, GPU, 메모리, 코어, 픽셀.
menu-view-live-opacity = 불투명도
menu-view-watermark = “Fd” 워터마크 표시
menu-view-watermark-hover = 라이브 뷰와 내보낸 이미지의 오른쪽 아래 모서리에 눈에 띄지 않는 “Fd” 브랜드 마크를 그립니다. 기본값은 켜짐입니다.

## Color menu

menu-color = 색상
menu-color-method = 방식
menu-color-palette = 팔레트
menu-color-custom = 사용자 정의
menu-color-duotone = 듀오톤
menu-color-binary = 이진(집합)
menu-color-binary-hover = 단순한 두 가지 색: 집합 안과 집합 밖, 그라데이션 없음.
menu-color-edit-gradient = 그라데이션 편집…
menu-color-normalize = 깊은 색상 정규화
menu-color-normalize-hover = 조밀한 필드가 반점처럼 보일 때, 뷰에서 측정한 탈출 범위에 맞게 팔레트를 다시 매핑합니다. 앨리어싱이 없는 뷰는 그대로 둡니다 — 상관없이 다시 매핑하려면 ‘항상 범위에 맞춤’을 선택하십시오. 부드러운 반복 방식에만 적용됩니다.
menu-color-more = 순환, 오프셋, 애니메이션, 효과: 컨트롤 패널 ▸ 채색

## Tools menu

menu-tools = 도구
# Followed by its shortcut, (M).
menu-tools-find-minibrot = 미니브로트 찾기 + 확대
menu-tools-find-minibrot-hover = 뉴턴 방법으로 뷰 중심을 근처 미니브로트의 정확한 중심에 맞추고, 그 주기를 알려 주고, 미니브로트 자체의 배율까지 확대합니다 — 한 번에 여러 자릿수를 건너뛰는 경우가 많습니다. 이미 그보다 깊이 있다면 중심만 이동합니다.
menu-tools-solver = 뉴턴 / Misiurewicz 솔버…
menu-tools-solver-hover = 정확한 미니브로트 중심(주기 + 크기) 또는 뷰에 가장 가까운 Misiurewicz 점을 구합니다 — 반발 승수 λ와 함께, 완전한 정밀도의 뉴턴 방법으로. 솔버가 들어 있는 위치 대화 상자를 엽니다.
# Followed by its shortcut, (A).
menu-tools-autopilot = 자동 줌(자동 조종)
menu-tools-autopilot-stop = 자동 조종 중지
menu-tools-autopilot-hover = 손을 쓰지 않는 연속 딥 줌: 디테일이 가장 풍부한 영역을 향해 다이브하며, 진행하면서 방향을 다시 잡습니다. 탐색 입력이 하나라도 있으면 멈춥니다.
menu-tools-priority = 우선순위:
menu-tools-priority-speed = 속도
menu-tools-priority-speed-hover = 설정한 속도로 줌하고, 디테일이 있는 첫 프레임으로 갱신합니다.
menu-tools-priority-quality = 품질
menu-tools-priority-quality-hover = 완전히 렌더링된 프레임만 표시하며, 줌이 그 프레임을 기다립니다.
menu-tools-target = 목표:
menu-tools-target-detail = 디테일
menu-tools-target-detail-hover = 뷰에서 가장 풍부한 경계로, 다이브하는 동안 다시 고릅니다.
menu-tools-target-misiurewicz = Misiurewicz 점
menu-tools-target-misiurewicz-hover = 가장 가까운 Misiurewicz 점을 한 번 구한 뒤 정확히 그 점으로 다이브합니다(Mandelbrot만).
menu-tools-orbits = 궤도 오버레이
menu-tools-orbits-hover = 커서 아래 점의 반복 경로를 그립니다. 애니메이션 옵션: 컨트롤 패널 ▸ 오버레이.
menu-tools-orbits-normalize = 정규화(뷰에 맞춤)
menu-tools-orbits-normalize-hover = 궤도를 뷰 전체에 맞춰 어떤 줌에서도 화면 안에 잘 들어오게 합니다(뷰포트를 거쳐 매핑하면 깊은 곳에서 화면 밖으로 날아가 버립니다).
menu-tools-play-tour = 투어 재생…
menu-tools-play-tour-hover = 재생할 투어 스크립트를 고릅니다. 도구 모음의 ▶는 클릭 한 번으로 마지막 투어를 다시 재생합니다.
menu-tools-tour-from-view = 현재 뷰로 투어 만들기…
menu-tools-tour-from-view-hover = 전체 뷰에서 현재 뷰까지 확대하는 투어를 만듭니다 — 캡션을 추가하고 다이브 시간을 정합니다. 편집, 재생, 렌더링할 수 있는 투어 스크립트(.toml)로 저장됩니다.
menu-tools-render-tour = 투어 렌더링…
menu-tools-render-tour-hover = 불러온 투어를 별도 프로세스에서 PNG 프레임 시퀀스(선택적으로 mp4)로 렌더링합니다.
menu-tools-close-player = 투어 플레이어 닫기
menu-tools-close-player-hover = 투어를 멈추고, 플레이어를 닫고, 사용자의 반복 및 채색 설정을 복원합니다
menu-tools-render-farm = 팜에서 렌더링…
menu-tools-render-farm-hover = 불러온 투어를 여러 컴퓨터에서 렌더링합니다: 이 컴퓨터가 작업을 관리하고, 다른 컴퓨터는 파일 ▸ 렌더 클라이언트…로 참여합니다.
menu-tools-benchmark = 벤치마크…
menu-tools-benchmark-hover = 렌더링 속도를 측정합니다 — 현재 설정으로, 또는 컴퓨터끼리 비교할 수 있는 표준 실행(고정 해상도 + 설정)으로. 번인은 이를 반복해 안정성 / 열 스로틀링을 점검합니다.
menu-tools-last-benchmark = 마지막 벤치마크 보기

## Navigate menu

menu-navigate = 탐색
menu-navigate-goto = 위치로 이동…
menu-navigate-goto-hover = 중심/줌을 입력하거나, 잘 알려진 점으로 이동하거나, 뷰 근처의 Misiurewicz / 미니브로트 특징점을 뉴턴 방법으로 구합니다.
menu-navigate-misiurewicz = Misiurewicz 탐색기…
menu-navigate-misiurewicz-hover = 작은 Misiurewicz 유형들이 어떻게 생겼는지 둘러보고 — 각 점의 줌 주기와 비틀림을 담은 썸네일 — 그중 어느 것이든 뉴턴 방법으로 임의의 깊이까지 구합니다. (Mandelbrot.)
menu-navigate-exact = 정확한 점(식)
menu-navigate-exact-note = 주 카디오이드 벌브의 뿌리점으로, 정확한 식(2π의 분수에 대한 cos/sin)으로 주어집니다. 좌표를 식 그대로 보관하므로 확대해도 정확히 그 점에 머뭅니다.
menu-navigate-random = 무작위 위치
menu-navigate-random-hover = 디테일이 풍부한 무작위 경계점으로 이동
menu-navigate-import-kfr = .kfr 가져오기…
menu-navigate-import-kfr-hover = Kalles Fraktaler 위치 파일 불러오기
menu-navigate-share = 위치 공유…
menu-navigate-share-hover = 자체 완결형 위치(.fdn) 복사 / 붙여 넣기 / 저장 / 불러오기: 프랙털, 완전 정밀도 중심, 줌, 채색.

## Help menu

menu-help = 도움말
# Followed by its shortcut, (F1).
menu-help-reference = 도움말 및 참조…
menu-help-welcome = 환영 / 빠른 시작…
menu-help-welcome-hover = 처음 실행할 때의 개요: 꼭 필요한 조작법과 몇 가지 랜드마크.
menu-help-diagnostics = 진단…
menu-help-diagnostics-hover = Fractadyne이 사용자의 하드웨어에서 올바르게 작동하는지 확인하고, 그 결과를 문제 보고에 첨부합니다
menu-help-report = 문제 보고…
menu-help-report-hover = 충돌 보고서, 로그, 위치를 첨부해 GitHub 이슈(또는 이메일)를 제출합니다
menu-help-accelerated = 더 빠른 딥 줌…
menu-help-accelerated-hover = 딥 줌 기준 궤도를 2.5-6.4배 빠르게 계산하는 선택적 빌드입니다. 이미지는 동일하며, 설정도 그대로 이어집니다.
menu-help-check-updates = 업데이트 확인
# $track: "stable" or "beta", the update track in use (the variants hold the wording).
menu-help-check-updates-hover =
    { $track ->
        [beta] GitHub에서 더 새로운 베타(사전 릴리스) 빌드를 확인합니다
       *[other] GitHub에서 더 새로운 안정 빌드를 확인합니다
    }
# $version: the new version; $channel: "beta" for a pre-release, "stable" otherwise (the variants hold the wording).
menu-help-update-available =
    { $channel ->
        [beta] { $version } (베타) 사용 가능 — 다운로드
       *[other] { $version } (안정) 사용 가능 — 다운로드
    }
menu-help-up-to-date = 최신 상태
menu-help-tagline = 네이티브 프랙털 탐색기
menu-help-license = 라이선스
# $year and $holder: the copyright year and holder, as the code gives them.
menu-help-copyright = © { $year } { $holder }. 이중 라이선스이며, 둘 중 원하는 라이선스로 사용할 수 있습니다.
menu-help-source = 소스

## The toolbar (its buttons are icons; these are their tooltips)

menu-toolbar-julia = Julia
menu-toolbar-julia-hover = 이 수식의 Julia 집합을 창 가득 표시
menu-toolbar-dual-hover = 둘 다, 연결: 왼쪽에는 집합, 오른쪽에는 커서 위치 점의 Julia
menu-toolbar-open-hover = 뷰 또는 위치 열기(PNG / EXR, .fdn, .kfr)
menu-toolbar-gallery-hover = 갤러리
menu-toolbar-export-hover = 이미지 내보내기…
menu-toolbar-share-hover = 위치 공유 — 자체 완결형 .fdn(프랙털, 완전 정밀도 중심, 줌, 채색)을 복사, 붙여 넣기, 저장 또는 불러오기
menu-toolbar-zoom-in-hover = 확대
menu-toolbar-zoom-out-hover = 축소
# $factor: the click-zoom factor (4, 10…).
menu-toolbar-click-zoom-hover = 클릭하여 줌({ $factor }×): 왼쪽 클릭은 그 점으로 다이브하고, 오른쪽 클릭은 되돌아 나옵니다(드래그는 그대로 이동). 배수는 컨트롤 ▸ 탐색에서 정합니다.
# Added to the click-to-zoom tooltip in the dual view.
menu-toolbar-click-zoom-dual = 이 기능이 켜져 있는 동안에는 Ctrl+클릭으로 Julia c를 고정합니다.
menu-toolbar-autopilot-running-hover = 자동 줌 실행 중 — 클릭하면 중지
menu-toolbar-autopilot-hover = 자동 줌: 디테일을 향해 다이브(A)
menu-toolbar-reset-hover = 뷰 초기화(즉시)
menu-toolbar-home-hover = 홈 뷰로 축소(애니메이션)
menu-toolbar-bookmarks-hover = 북마크: 이 뷰를 저장하거나 저장된 뷰로 이동
# $name: the palette's name.
menu-toolbar-next-palette-hover = 다음 팔레트({ $name })
# $file: the last tour's file name.
menu-toolbar-play-last-hover = 투어 재생 — { $file }(클릭) · 마지막 투어를 다시 재생합니다. 다른 투어는 도구 ▸ 투어 재생…에서 고릅니다
menu-toolbar-play-hover = 투어 재생…(도구 ▸ 투어 재생)
menu-toolbar-speed-hover = 재생 속도
menu-toolbar-fullscreen-hover = 전체 화면 전환

## Dialogs: words shared by many windows

# A button that closes a window without doing what it offers.
dlg-cancel = 취소
# A button that dismisses a window (nothing in it is pending).
dlg-close = 닫기
# A button that copies the window's text to the clipboard (after a copy icon).
dlg-copy = 복사
# A button that jumps to a location.
dlg-go = 이동
# A button that fills the fields from the view on screen.
dlg-use-current = 현재 값 사용
# A notice after a file could not be written. $error: the system's reason, as it gives it.
dlg-save-failed = 저장 실패: { $error }
# A notice when no graphics device is available to render with.
dlg-gpu-not-available = GPU를 사용할 수 없음
# The name of the plain-text file type in a save dialog (the extension .txt is added by the code).
dlg-filter-text = 텍스트

## Asked after a session that did not shut down cleanly

dlg-crash-title = Fractadyne이 정상적으로 종료되지 않았습니다
dlg-crash-intro = 이전 세션이 예기치 않게 종료되었습니다. 보고서가 이 컴퓨터에 저장되었습니다 — 보내시겠습니까?
dlg-crash-review = 검토하기 전에는 아무것도 보내지 않습니다: 다음 화면에서 포함될 내용을 정확히 보여 주며, 시스템 정보는 확인란 하나로 제외할 수 있습니다.
dlg-crash-report = 보고하기…
dlg-crash-report-hover = 충돌 보고서와 충돌이 일어난 위치를 첨부해 문제 보고를 엽니다.
dlg-crash-not-now = 나중에
dlg-crash-dont-ask = 충돌 후 다시 묻지 않기
dlg-crash-dont-ask-hover = 보고서는 계속 이 컴퓨터에 저장되며, 도움말 ▸ 문제 보고는 언제든 사용할 수 있습니다.

## Asked after a session that ended with a second graphics device in use

dlg-second-device-title = 두 번째 그래픽 장치를 끄시겠습니까?
# $device: the graphics device's name and details, as the system gives them.
dlg-second-device-intro = 두 번째 그래픽 장치({ $device })를 사용하는 중에 이전 세션이 예기치 않게 종료되었습니다. 그 장치가 원인일 수 있습니다.
dlg-second-device-note = 끄면 이전처럼 창을 그리는 카드가 모든 것을 렌더링합니다. 고급 ▸ 두 번째 그래픽 카드에서 다시 켤 수 있습니다.
dlg-second-device-off = 끄기
dlg-second-device-keep = 켜 두기

## Shown when the previous session's iteration setting was set aside

dlg-quarantine-title = 반복 설정 보류됨
dlg-quarantine-change = 컨트롤 ▸ 품질에서 언제든 바꿀 수 있습니다.
dlg-quarantine-keep = 기본값 유지
# A button. $setting: the iteration setting the previous session used (such as "20,000 iterations").
dlg-quarantine-restore = { $setting } 복원

## Navigate ▸ Go to location

dlg-goto-title = 위치로 이동
# Before two choices: "x, y" (cartesian) and the polar form below.
dlg-goto-enter-as = 입력 형식:
# The polar way of entering a point: an offset plus a radius r at an angle θ. Keep the symbols.
dlg-goto-polar = 오프셋 + r ∠ θ
# The tooltip of the "?" button beside the entry choice.
dlg-goto-help-hover = 입력란에 쓸 수 있는 것: 분수, 거듭제곱, sqrt/sin/cos…, pi 및 기타 상수 — 도움말 → 좌표 식을 엽니다
# Labels over the entry fields. x0, y0, r, θ, Re and Im are the fields' symbols: keep them.
dlg-goto-polar-offset = 오프셋 중심  (x0, y0 — 순수한 극좌표 점이면 0으로 둠)
dlg-goto-polar-radius = 반지름 r, 각도 θ
dlg-goto-re = 중심 — 실수부(Re)
dlg-goto-im = 중심 — 허수부(Im)
dlg-goto-zoom = 줌(배율)
# A warning under the zoom field, after a ⚠ sign. $have: the iteration count set; $typical: the count this depth typically needs.
dlg-goto-iter-warning = 반복 횟수가 { $have }회로 고정되어 있지만, 이 깊이에는 보통 약 { $typical }회가 필요합니다. 값을 올리기 전까지 뷰가 평평하게 렌더링됩니다 — 또는 스스로 맞춰지는 자동 반복 횟수(컨트롤 ▸ 품질)를 켜십시오.
dlg-goto-copy-hover = 이 위치를 클립보드에 복사
dlg-goto-cancel-hover = 이동하지 않고 닫기
# The expressions in brackets are examples of what can be typed: keep them as they are.
dlg-goto-paste-note = 다른 사람의 중심/줌을 붙여 넣거나, 복사로 공유합니다. 좌표에는 식을 쓸 수 있습니다: 정확한 분수(-3/4), 함수와 상수(-0.5 + 0.25*cos(pi/4), sqrt, root(x,n), ^; 각도는 라디안), 그리고 실수부 입력란에는 복소수 값 전체((37+16i)/100)를 넣을 수 있으며, 그러면 두 입력란이 모두 채워집니다.
# A section that opens and closes.
dlg-goto-feature = 특징점으로 이동(Mandelbrot)
dlg-goto-feature-poi = 잘 알려진 점:
# The dropdown's text before a point is picked.
dlg-goto-feature-poi-pick = 관심 지점으로 이동…
dlg-goto-feature-solve = 또는 현재 뷰 근처의 특징점 구하기:
# Two choices: what to solve for. Misiurewicz is a mathematician's name (a Misiurewicz point).
dlg-goto-feature-misiurewicz = Misiurewicz
dlg-goto-feature-minibrot = 가장 가까운 미니브로트
# Labels of two small number fields; k and p are their symbols.
dlg-goto-feature-preperiod = 전주기 k
dlg-goto-feature-period = 주기 p
# The faint text in an empty field: left empty, the value is found automatically.
dlg-goto-feature-auto-hint = 자동
# A small button that empties both fields, so they are found automatically again.
dlg-goto-feature-auto = 자동
dlg-goto-feature-auto-hover = 두 칸을 모두 비워, 다음 검색이 뷰 중심의 궤도에서 전주기와 주기를 감지하게 합니다.
dlg-goto-feature-kp-hover = 둘 다 비워 두면 뷰 중심의 궤도에서 감지하며, 찾은 값이 여기에 채워집니다. 나선을 보고 그 전주기를 아는 사람은 없으며, 그 값을 직접 넣어야 했던 것이 이 솔버를 실제로는 쓸 수 없게 만든 원인이었습니다.
# While a solve runs, followed by its elapsed time and the note below.
dlg-goto-feature-solving = 구하는 중…
dlg-goto-feature-solving-note = (임의 정밀도, 깊은 뷰는 더 오래 걸림)
dlg-goto-feature-cancel-hover = 기다리기를 멈춥니다. 계산 자체는 백그라운드에서 끝까지 실행되고 그 결과는 버려집니다 — 반쯤 끝난 상태로 남는 것은 없습니다.
dlg-goto-feature-find = 뷰 근처에서 찾기
dlg-goto-feature-find-hover =
    현재 중심에서 출발해 뉴턴 방법으로 정확한 특징점을 구합니다(임의 정밀도). 먼저 특징점 근처로 이동하십시오.

    구하기 전에 위의 줌 입력란을 원하는 깊이로 설정하십시오: 점은 그 깊이에 필요한 정밀도로 계산되며, 뷰는 곧바로 그곳으로 이동합니다. 수동 줌을 길게 반복하는 대신 한 번의 계산으로 끝납니다.
dlg-goto-solve-cancelled = 계산을 취소했습니다.

## Snapshot (the camera button): its tooltip, and the choice on the first press

# The camera button's tooltip, one per Snapshot setting. Followed by its shortcut, (Ctrl+S).
dlg-snapshot-hover-ask = 스냅샷 — 보이는 그대로의 뷰 또는 전체 렌더링을 저장합니다. 처음 누를 때 묻습니다
dlg-snapshot-hover-screen = 스냅샷 — 보이는 그대로의 뷰를 마지막 내보내기 폴더에 저장
dlg-snapshot-hover-render = 스냅샷 — 내보내기 설정으로 전체 렌더링하여 마지막 내보내기 폴더에 저장
# The window's title.
dlg-snapshot-title = 스냅샷
dlg-snapshot-question = 스냅샷(카메라 버튼, Ctrl+S)으로 무엇을 저장하시겠습니까?
# The first choice, and a short tag beside it.
dlg-snapshot-screen = 화면 캡처
dlg-snapshot-screen-tag = 즉시
dlg-snapshot-screen-note = 화면에 보이는 그대로의 뷰를 화면 해상도의 PNG로 마지막 내보내기 폴더에 저장합니다. 보이는 그대로 저장됩니다.
# The second choice, and a short tag beside it.
dlg-snapshot-render = 전체 렌더링
dlg-snapshot-render-tag = 오래 걸릴 수 있음
# $size: the image size, as 1920×1080; $ss: the supersampling factor (1–4); $format: PNG or EXR.
dlg-snapshot-render-note = 내보내기 설정으로 렌더링합니다 — { $size }, { $ss }× 슈퍼샘플링, { $format }. 최고 품질이지만 깊은 뷰는 몇 분에서 몇 시간이 걸릴 수 있습니다. 백그라운드에서 실행되며, 파일 → 이미지 내보내기…에서 진행 상황을 보고 취소할 수 있습니다.
dlg-snapshot-remember = 선택을 기억하고 다시 묻지 않기
dlg-snapshot-change = 파일 → 설정 → 스냅샷에서 언제든 바꿀 수 있습니다.

## Share location (.fdn)

dlg-share-title = 위치 공유
dlg-share-intro = 자체 완결형 위치(프랙털, 완전 정밀도 중심, 줌, 채색)입니다. 공유하려면 복사하고, 받은 위치는 붙여 넣거나 불러온 뒤 적용을 누르십시오.
dlg-share-thumb = 저장 파일에 썸네일 포함
dlg-share-thumb-hover = .fdn 저장 시 이 뷰의 작은 그림을 넣어 갤러리에서 파일을 알아볼 수 있게 합니다. 약 55 KB가 늘어나며, 위의 텍스트에는 포함되지 않습니다.
dlg-share-copy-hover = 텍스트를 클립보드에 복사
# Buttons (the first after a save icon).
dlg-share-save = .fdn 저장…
dlg-share-load = .fdn 불러오기…
dlg-share-apply = 적용
dlg-share-apply-hover = 상자에 있는 위치로 이동
dlg-share-copied = 클립보드에 복사했습니다.

## Help ▸ Report an issue (the report itself stays in English: it is read by the maintainer)

dlg-report-title = 문제 보고
dlg-report-intro = 보고하기에 가장 좋은 곳은 GitHub 이슈입니다 — 공개되어 검색할 수 있고, 언제 수정되는지 볼 수 있습니다. 이메일도 괜찮습니다. 먼저 아래 내용을 모두 검토하십시오: 직접 실행하기 전에는 아무것도 이 컴퓨터를 떠나지 않습니다.
# Labels before three dropdowns.
dlg-report-type = 유형:
dlg-report-severity = 심각도:
dlg-report-repro = 재현 가능:
dlg-report-describe = 무슨 일이 있었는지 설명:
# The faint text in the empty description box.
dlg-report-describe-hint = 무슨 일이 있었는지, 재현하는 단계, 그리고 원래 기대한 결과.
# Before the checkboxes of what the report carries.
dlg-report-include = 포함:
dlg-report-sysinfo = 시스템 정보(버전, OS, CPU, GPU, VRAM)
dlg-report-location = 현재 위치(.fdn)
dlg-report-log = 최근 로그
dlg-report-frames = 프레임 기록(마지막 1분과 마지막 40프레임)
dlg-report-crash = 최근 충돌 보고서
dlg-report-crash-none = 최근 충돌 보고서(찾지 못함)
dlg-report-crash-view = 충돌한 위치(.fdn)
dlg-report-crash-view-none = 충돌한 위치(.fdn)(찾지 못함)
dlg-report-crash-view-hover = 충돌이나 그래픽 장치 손실이 일어난 정확한 뷰입니다 — 관리자가 문제를 재현할 수 있게 해 주는 단 하나의 단서입니다. 자동으로 기록됩니다.
dlg-report-test = 진단 테스트 결과
dlg-report-test-none = 진단 테스트 결과(도움말 → 진단…에서 실행)
dlg-report-test-hover = 사용자의 하드웨어에서 기계가 검증한 통과/실패 결과입니다 — 관리자에게는 설명만 있는 것보다 훨씬 유용합니다
# A section that opens and closes. $size: the report's size, as "12 KB".
dlg-report-preview = 보낼 내용 미리 보기  ({ $size })
dlg-report-github = GitHub 이슈 열기
dlg-report-github-hover = 제목이 채워진 새 이슈 페이지를 엽니다. 보고서는 클립보드에 복사되므로 이슈 본문에 붙여 넣으십시오(Ctrl+V).
# Buttons (after a copy and a save icon).
dlg-report-copy = 보고서 복사
dlg-report-copy-hover = 모든 내용을 클립보드에 복사
dlg-report-save = 보고서 저장…
dlg-report-save-hover = 첨부할 .txt로 저장
dlg-report-by-email = 또는 이메일로:
dlg-report-gmail = Gmail에서 작성
# $address: the email address reports go to.
dlg-report-gmail-hover = 제목이 채워진 { $address } 앞 Gmail 작성 창을 엽니다. 복사한 보고서를 붙여 넣으십시오
dlg-report-email = 이메일 앱…
dlg-report-email-hover = 기본 메일 앱(mailto)을 엽니다 — 이메일 처리기가 설정되어 있어야 합니다
dlg-report-tip = 팁: 어느 버튼을 누르든 보고서가 클립보드에 복사됩니다 — 이슈나 메시지 본문에 붙여 넣으십시오(Ctrl+V). 가능하면 스크린샷이나 샘플 이미지도 첨부해 주십시오(끌어다 놓기). 렌더링과 UI 문제에 큰 도움이 됩니다.
# Notices in the window after a button is pressed.
dlg-report-opened-github = GitHub을 열었습니다 — 보고서가 클립보드에 있으니 이슈에 붙여 넣으십시오.
dlg-report-copied = 보고서를 클립보드에 복사했습니다.
# $path: where the report was saved.
dlg-report-saved = { $path }에 저장했습니다
dlg-report-opened-email = 이메일 앱을 열었습니다 — 보고서가 클립보드에 있으니 붙여 넣으십시오.
dlg-report-opened-gmail = Gmail 작성 창을 열었습니다 — 복사한 보고서를 본문에 붙여 넣으십시오(Ctrl+V).

## A location whose checksum does not match

dlg-checksum-title = 이 위치는 손상되었을 수 있습니다
# After a ⚠ sign.
dlg-checksum-mismatch = 체크섬이 함께 있는 데이터와 일치하지 않습니다.
dlg-checksum-why = 보통 텍스트가 전달되는 도중에 잘리거나 바뀌었다는 뜻입니다 — 채팅 창이나 포럼 글에서 일부만 복사한 것이 흔한 원인입니다.
# A label over where the location came from (a file path, or the clipboard).
dlg-checksum-source = 출처
# The two checksums, side by side: the one written in the file, and the one computed from its data.
dlg-checksum-in-file = 파일에 기록된 값
dlg-checksum-from-data = 데이터로 계산한 값
dlg-checksum-load-note = 그래도 불러올 수 있습니다 — 좌표는 여전히 괜찮을 수 있으며, 아무것도 덮어쓰지 않습니다. 읽을 수 없는 부분은 불러온 뒤에 알려 드립니다.
dlg-checksum-load = 그래도 불러오기
dlg-checksum-load-hover = 불일치를 무시하고 그 위치로 이동
# Notices after loading it. $zoom: the magnification, as 1.2e148.
dlg-checksum-loaded = 체크섬이 일치하지 않지만 @ { $zoom }×에서 불러왔습니다
# $zoom: the magnification, as 1.2e148; $note: what the loader found wrong with the location.
dlg-checksum-loaded-note = @ { $zoom }×에서 불러왔습니다 — { $note }
dlg-checksum-not-loaded = 불러오지 않았습니다.

## File ▸ Reset application state

dlg-reset-title = 애플리케이션 상태 초기화
# After a ⚠ sign, followed by a list.
dlg-reset-warning = 저장된 모든 Fractadyne 데이터를 영구적으로 삭제합니다:
# The list's items, each after a bullet.
dlg-reset-session = 저장된 세션(현재 뷰, 채색, 환경 설정)
dlg-reset-bookmarks = 모든 북마크와 그 썸네일
dlg-reset-refcache = 기준 궤도 캐시(위치를 다시 방문하면 다시 만들어짐)
# A label over the folder that is deleted.
dlg-reset-location = 위치
dlg-reset-note = 되돌릴 수 없습니다. 종료할 때 현재 세션을 다시 저장하지 않으므로, 다음 실행 때 기본값을 불러옵니다.
dlg-reset-confirm = 모두 초기화
dlg-reset-done = 애플리케이션 상태를 초기화했습니다 — 다음 실행 때 기본값을 불러옵니다.
# $error: the system's reason, as it gives it.
dlg-reset-failed = 초기화 실패: { $error }

## Tools ▸ Tour from current view

dlg-tour-title = 현재 뷰로 투어 만들기
dlg-tour-intro = 전체 뷰에서 지금 있는 곳까지 확대하는 투어를 만듭니다.
# A label before the fractal and magnification the tour ends at.
dlg-tour-destination = 목적지
dlg-tour-notation = 설명(다이브하는 동안 표시되는 캡션, 선택 사항)
# The faint example text in the empty caption box.
dlg-tour-notation-hint = 예: Misiurewicz 세 갈래 가지로 다이브…
dlg-tour-duration = 줌 시간
# $rate: how fast the dive zooms, in powers of ten a second (as 2.5).
dlg-tour-rate = ≈ { $rate } 자릿수/초
# A small button that puts the duration back to its default.
dlg-tour-reset = 초기화
dlg-tour-reset-hover = 깊이에 따른 기본값
dlg-tour-duration-note = 기본값은 깊이에 따라 늘어나 다이브를 계속 지켜볼 수 있게 합니다. 더 빠른 다이브에서는 극단적인 줌에서 단색 프레임이 번쩍일 수 있습니다.
# A button, after a save icon.
dlg-tour-save = 투어 저장…
# The file type's name in the save dialog.
dlg-tour-filter = Fractadyne 스크립트(TOML)
# A notice. $file: the saved file's name. --render-tour is a command-line option: keep it.
dlg-tour-saved = 투어를 저장했습니다 → { $file }. 도구 → 투어 재생으로 재생하거나, --render-tour로 렌더링하십시오.

## Navigate ▸ Bookmarks ▸ Manage…

dlg-bookmarks-title = 북마크
# The faint text in the empty name box.
dlg-bookmarks-name-hint = 이름(선택 사항)
# A button, after a bookmark icon.
dlg-bookmarks-add = 현재 뷰 추가
dlg-bookmarks-empty = 아직 북마크가 없습니다. 현재 뷰에서 추가하십시오.
# The tooltip of a bookmark's delete (bin) button.
dlg-bookmarks-delete-hover = 삭제

## Tools ▸ Benchmark…

dlg-bench-title = 벤치마크
dlg-bench-mode = 모드
dlg-bench-current = 현재 설정
dlg-bench-current-hover = 현재 해상도와 설정으로, 고정된 딥 줌 투어를 라이브 뷰에서 재생합니다.
dlg-bench-standard = 표준
dlg-bench-standard-hover = 해상도와 모든 렌더링 설정을 고정해 점수를 컴퓨터끼리 비교할 수 있게 합니다.
dlg-bench-resolution = 해상도
dlg-bench-depth = 깊이
# Standard and Ultra are the two depths in the dropdown this describes.
dlg-bench-depth-hover = 두 다이브 모두 깊은 floatexp/BLA 경로로 들어갑니다. 초심도(1e48×)는 표준(1e32×)보다 그곳에서 훨씬 오래 머물며, 그만큼 실행 시간도 깁니다.
dlg-bench-burnin = 번인(반복)
dlg-bench-burnin-hover = 벤치마크를 반복 실행해 안정성과 열 스로틀링을 드러냅니다.
# The label of a slider: how many times the benchmark runs.
dlg-bench-passes = 회 실행
# $size: the image size, as 1920×1080; $ss: the supersampling factor; $depth: the dive's final magnification, as 1e32.
dlg-bench-std-summary = 화면 밖에서 { $size }, { $ss }× SS, Mandelbrot/부드러운 반복, { $depth }×까지 60프레임 다이브로 렌더링합니다.
dlg-bench-run = 실행

## The standardized benchmark's progress window

dlg-bench-progress-title = 벤치마크 실행 중…
# $resolution: the benchmark's resolution, as the dropdown names it.
dlg-bench-progress-mode = 표준 · { $resolution }
# $pass: the pass running now; $total: how many passes there are.
dlg-bench-progress-pass = 번인 패스 { $pass }/{ $total }
dlg-bench-progress-dive = 고정된 다이브 렌더링 중…
# On the progress bar. $done: frames rendered; $total: frames in the dive.
dlg-bench-progress-frame = 프레임 { $done }/{ $total }
# $fps: frames per second, as 41.3.
dlg-bench-progress-last = 마지막 패스: { $fps } fps

## The benchmark's results

dlg-bench-results-title = 벤치마크 결과
# Buttons, each after an icon.
dlg-bench-results-run-again = 다시 실행…
dlg-bench-results-save = 저장…
dlg-bench-results-none = 아직 실행한 벤치마크가 없습니다.
dlg-bench-results-saved = 벤치마크를 저장했습니다.

## The accelerated build's notice when its arithmetic libraries are missing

dlg-backend-title = 가속 빌드 — 내장 산술 사용 중
dlg-backend-dont-show = 다시 표시하지 않기

## File ▸ Gallery…

dlg-gallery-title = 갤러리
# A button that picks the folder to show.
dlg-gallery-folder = 폴더…
# A button, after a refresh icon.
dlg-gallery-refresh = 새로 고침
dlg-gallery-empty = 이 폴더에 Fractadyne 이미지가 없습니다.
# An image's title when it does not name its fractal.
dlg-gallery-untitled = Fractadyne 이미지
# $zoom: the image's magnification, as 1.2e148; $saved: when it was saved, as the file gives it.
dlg-gallery-zoom = 줌 { $zoom }   { $saved }
# The notes saved with the image, in quotation marks. Use your language's quotation marks.
dlg-gallery-notes = “{ $notes }”
dlg-gallery-open = 이 뷰 열기
dlg-gallery-loaded = 갤러리에서 뷰를 불러왔습니다.

## File ▸ Export image…

dlg-export-title = 이미지 내보내기
# A dropdown of image sizes.
dlg-export-size = 크기
# The size dropdown's text for a size set by hand. $size: the size, as 1920×1080.
dlg-export-size-custom-current = 사용자 정의 — { $size }
# The size dropdown's first choice.
dlg-export-size-custom = 사용자 정의…
dlg-export-size-custom-hover = 너비를 직접 정합니다. 높이는 아래의 화면비를 따릅니다.
dlg-export-width = 너비(px)
# After the width field. $height: the height in pixels, which follows from the aspect.
dlg-export-height-from-aspect = × { $height }(화면비에서)
dlg-export-ss = 슈퍼샘플링
dlg-export-aspect = 화면비
# The aspect choice that keeps the window's own shape.
dlg-export-aspect-window = 창에 맞춤
# Before the choices PNG and OpenEXR.
dlg-export-format = 형식:
dlg-export-hud = 위치 HUD
dlg-export-hud-hover = 내보낸 이미지의 왼쪽 위에 줌 배율 + 좌표 패널을 새겨 넣습니다(출력 크기에 맞춰 조정되며, 지도/Mandelbrot 뷰의 줌 + 중심을 표시합니다).
dlg-export-glitch = 글리치 보정
dlg-export-glitch-hover = 섭동 글리치를 여러 기준점으로 보정합니다. 아주 깊은(floatexp) 단일 뷰 내보내기에서는 자동으로 건너뛰므로, 기준 궤도 계산 + 렌더링이 별도 스레드에서 실행되고 앱이 계속 반응합니다. Mandelbrot나 Multibrot 뷰(Julia 모드 밖)에는 사용하지 않습니다: 정확한 계산과 비교해 보니 그곳에서는 아무것도 고치지 못하면서 내보내기 시간이 최대 5배 늘었습니다.
dlg-export-deep-note = 깊은 내보내기: 기준 궤도를 별도 스레드에서 계산합니다(UI는 계속 반응). 이 깊이에서는 글리치 보정을 건너뜁니다.
# A dropdown, shown in the dual (map + Julia) view, and its three choices.
dlg-export-dual = 이중 배치
dlg-export-dual-side = 나란히
dlg-export-dual-separate = 별도 파일
dlg-export-dual-map = 지도만
dlg-export-notes = 메모:
# The faint text in the empty notes box.
dlg-export-notes-hint = 이미지와 함께 저장됨(≤120자)
# $width and $height: the image size in pixels; $left: how many characters the notes can still take.
dlg-export-output = 출력: { $width } × { $height } px   ({ $left }자 남음)
# When the export's shape differs from the window's. $percent: how much more, as 25.
dlg-export-extra-width = 창보다 너비를 { $percent }% 더 보여 줍니다(잘리는 부분 없음)
# The same, when the export is taller than the window. $percent: how much more, as 25.
dlg-export-extra-height = 창보다 높이를 { $percent }% 더 보여 줍니다(잘리는 부분 없음)
dlg-export-tiles-note = 백그라운드 스레드에서 타일 단위로 렌더링합니다(크기 제한 없음). 파일에 뷰 + 메모가 들어 있어 파일 ▸ 뷰 또는 위치 열기로 다시 열 수 있습니다.
# A label before the button that picks the folder exports go to.
dlg-export-folder = 폴더:
dlg-export-choose = 선택…
dlg-export-choose-hover = 내보낼 대상 디렉터리 선택
dlg-export-open-after = 완료되면 열기
dlg-export-open-after-hover = 완성된 이미지가 기록되는 즉시 시스템 뷰어로 넘깁니다.
# $count: the graphics cards in this machine (2 or more).
dlg-export-all-gpus = 모든 그래픽 카드 사용({ $count }개)
dlg-export-all-gpus-hover = 이 컴퓨터의 모든 그래픽 카드에 이미지를 타일 단위로 나누어 맡기며, 각 카드는 깊은 기준 궤도의 사본을 따로 가집니다. 같은 카드끼리는 카드 한 장이 혼자 렌더링한 것과 정확히 같은 이미지를 만듭니다. 모델이 다르면 타일마다 몇몇 픽셀을 약간 다르게 그립니다(측정 결과: 눈에 보이는 이음매 없음). 카드 하나가 실패하면 그 타일은 다른 카드로 넘어갑니다. 분할할 수 없는 이미지(일부 채색 방식과 수식)는 이 창의 카드를 사용합니다.
# The export's progress.
dlg-export-preparing = 준비 중(기준 궤도 계산 중)…
dlg-export-saving = 저장 중…
dlg-export-rendering = 렌더링 중…
# $time: the time since the export started, as 8.3s or 1m 04.0s.
dlg-export-elapsed = 경과: { $time }
dlg-export-cancel-hover = 내보내지 않고 닫기
# A button, after a save icon.
dlg-export-save-as = 다른 이름으로 저장…
dlg-export-save-as-hover = 파일 이름과 위치 선택
dlg-export-export = 내보내기
dlg-export-export-hover = 렌더링하여 위 폴더에 저장(이름 자동 지정)

## Notices worded at startup (main.rs)

# A notice shown once after starting, when the saved session comes from a newer version of the app. $saved and $handled: version numbers of the session file's format (1, 2…).
app-state-newer = 이 세션은 더 새로운 Fractadyne에서 저장되었습니다(상태 v{ $saved }, 이 빌드는 v{ $handled }까지 처리). 일부 설정이 적용되지 않을 수 있으며, 저장하면 이 빌드의 형식으로 다시 기록됩니다.
# The title of the notice that says the accelerated build's arithmetic libraries are found again (its text comes from elsewhere).
app-notice-mpfr-restored-title = 가속 산술 복원됨
# After Fractadyne restarted itself because the graphics driver reset the GPU.
app-toast-gpu-recovered = 그래픽 장치 재설정에서 복구했습니다 — 이전 위치는 렌더링 부담이 너무 커서 홈 뷰에서 다시 열었습니다.

## Short notices (toasts)

# When the autopilot reached a view that could not be rendered and put the last good view back.
app-toast-autopilot-view-restored = 자동 조종 중지: 뷰를 사용할 수 없게 되어 원래대로 되돌렸습니다(로그에 기록됨)
# $error: the system's reason, as it gives it (often in English).
app-toast-gradients-save-failed = 그라데이션을 저장할 수 없습니다: { $error }
# $error: the reason, as the program gives it (often in English).
app-toast-gradients-serialize-failed = 그라데이션을 직렬화할 수 없습니다: { $error }
# $name: the name the user gave the gradient.
app-toast-gradient-saved = 그라데이션 “{ $name }”을(를) 저장했습니다.
# $error: the reason, as the program gives it (often in English).
app-toast-bookmarks-serialize-failed = 북마크를 직렬화할 수 없습니다: { $error }
# $error: the system's reason, as it gives it (often in English).
app-toast-bookmarks-save-failed = 북마크를 저장할 수 없습니다: { $error }
# Jumping to a named point given by exact expressions. $name: the point's name (in English).
app-toast-coordinates-failed = { $name }: 좌표를 계산할 수 없습니다
# $zoom: the magnification, already formatted (1,234.5 or 1.2e148).
app-toast-random-location = 무작위 위치 · { $zoom }×
# Importing a Kalles Fraktaler .kfr location. $error: the reason (in English).
app-toast-import-failed = 가져오기 실패: { $error }
# $zoom: the magnification, already formatted.
app-toast-location-loaded = 위치를 불러왔습니다 @ { $zoom }×

## Go to location: why a jump could not be made (one line per field that failed)

# "Re" is the dialog's real-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-re = Re: { $error }
# "Im" is the dialog's imaginary-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-im = Im: { $error }
# The imaginary field was given a complex value. $error: the evaluator's reason (in English).
app-goto-error-im-real = Im: { $error } — 허수부는 i 없이 실수로 입력하십시오
# "Zoom" is the dialog's magnification field.
app-goto-error-zoom-empty = 줌: 배율을 입력하십시오(예: 1e50 또는 250000)
# $zoom: what was typed, in quotes.
app-goto-error-zoom = 줌: { $zoom }을(를) 읽을 수 없습니다 — 1e50이나 250000 같은 양의 배율을 입력하십시오
app-goto-error-invalid = 잘못된 입력입니다 — 좌표와 줌을 확인하십시오.

## Finding a minibrot or Misiurewicz point (the M key and the Go to dialog)

app-feature-needs-holomorphic = 미니브로트 찾기에는 정칙 계열(Mandelbrot / Multibrot)이 필요합니다.
app-feature-not-in-julia = 미니브로트 찾기는 Mandelbrot 집합에서 검색합니다 — 사용하려면 Julia 모드를 끄십시오.
app-feature-busy = 이미 특징점을 찾는 중입니다 — 한 번에 하나씩만 찾습니다.
app-feature-searching = 가장 가까운 미니브로트를 찾는 중…
app-feature-mandelbrot-only = 특징점 찾기는 Mandelbrot 전용입니다.
app-feature-no-pair = 검색 예산 안에서 Misiurewicz 점을 찾지 못했습니다 — 점이 이 뷰에서 찾기 기능이 닿는 깊이보다 깊거나, 중심이 나선이나 분기점 근처에 있지 않을 수 있습니다.
# The name of what was found, used as $feature in the messages below. $period: the minibrot's period (a whole number).
app-feature-minibrot = 주기 { $period } 미니브로트
# $preperiod and $period: the point's two whole numbers (k, p). $octaves: how often the view repeats when zooming in (2 decimals); $twist: the turn per repeat in degrees.
app-feature-misiurewicz-lambda = Misiurewicz ({ $preperiod },{ $period }) — { $octaves }옥타브마다 반복, 비틀림 { $twist }°
# $preperiod and $period: the point's two whole numbers (k, p).
app-feature-misiurewicz = Misiurewicz ({ $preperiod },{ $period })
# Added after the messages below when the fixed iteration count is too low for the new depth. $have: the iterations set now; $typical: what the depth typically needs.
app-feature-iter-note = ⚠ 반복 횟수가 { $have }회로 고정되어 있지만 이 깊이에는 보통 약 { $typical }회가 필요합니다 — 뷰가 평평하게 렌더링되면 값을 올리십시오(또는 자동 반복 횟수를 켜십시오).
# $feature: what was found (see above). $zoom and $asked: magnifications, already formatted; $octaves: the solver's depth limit.
app-feature-zoomed-capped = { $feature }(으)로 줌했습니다 — { $zoom }×. 계산은 { $octaves }옥타브로 제한되므로 { $asked }×가 아니라 그곳에서 멈췄습니다.
# $feature: what was found. $zoom: the magnification, already formatted. $seconds: how long the search took (1 decimal).
app-feature-zoomed = { $feature }(으)로 줌했습니다 — { $zoom }× ({ $seconds }s)
# $feature: what was found. $seconds: how long the search took (1 decimal).
app-feature-snapped = { $feature } 중심에 맞췄습니다({ $seconds }s)
app-feature-no-minibrot = 뷰 근처에서 미니브로트 중심을 찾지 못했습니다 — 미니브로트에 더 가까이 확대하십시오.
# $feature: the point asked for, e.g. Misiurewicz (3,1). $widths: how many view-widths away, already formatted. $octaves: how far to zoom out.
app-feature-just-outside = 이 뷰 바로 바깥에서 { $feature } 점을 찾았습니다 — 뷰 너비의 { $widths }배만큼 떨어져 있습니다. 화면에 보이게 하려면 약 { $octaves }옥타브 축소하거나, 이 배율의 특징점을 찾으려면 더 큰 전주기를 설정하십시오.
# $feature: the point asked for. $widths: how many view-widths away, already formatted (may be 2^187). $octaves: how far to zoom out.
app-feature-far = { $feature } 점을 찾았지만 뷰 너비의 { $widths }배만큼 떨어져 있습니다 — 그 특징점은 이 뷰보다 훨씬 거친 배율에 있습니다. 도달하려면 약 { $octaves }옥타브 축소하거나, 이 배율의 특징점을 찾으려면 더 큰 전주기를 시도하십시오.
# $feature: the point asked for. $residual: a small number in scientific notation.
app-feature-not-preperiodic = { $feature }은(는) 이곳의 궤도와 맞지 않습니다(잔차 { $residual }) — 두 칸을 모두 비워 뷰에서 쌍을 감지하게 하십시오.
# $feature: the point asked for. "k/p" are the dialog's two fields.
app-feature-not-converged = 뷰 근처에서 수렴한 { $feature } 점이 없습니다 — 더 가까이 이동하거나 다른 k/p를 시도하십시오.
app-feature-bad-request = 전주기와 주기는 모두 양수여야 합니다.

## Share location dialog and location files

app-location-empty = 불러올 것이 없습니다(또는 텍스트가 너무 큽니다).
app-location-not-fractadyne = Fractadyne 위치가 아닙니다.
# $zoom: the magnification, already formatted. $note: what in the text was ignored or repaired (in English).
app-share-loaded-note = @ { $zoom }×에서 불러왔습니다 — { $note }.
# $error: the system's reason (often in English).
app-share-save-failed = 저장 실패: { $error }
app-share-saved = 저장했습니다.
app-share-saved-rendering-thumb = 저장했습니다. 썸네일 렌더링 중…
app-share-saved-no-thumb = 저장했습니다 — 썸네일 없이(이 뷰는 썸네일을 렌더링할 수 없었습니다).
app-share-saved-with-thumb = 썸네일과 함께 저장했습니다.
# $error: the system's reason (often in English).
app-share-saved-thumb-failed = 저장했지만 썸네일을 추가할 수 없었습니다: { $error }
app-share-loaded-into-box = 상자에 불러왔습니다 — 검토한 뒤 적용을 누르십시오.
# Reading a file failed (a location or a palette). $error: the system's reason (often in English).
app-read-failed = 읽기 실패: { $error }
app-share-too-large = 파일이 너무 큽니다(.fdn 위치가 아닐 수 있습니다).

## File-dialog filter names (the extensions are added by the code)

app-filter-kfr = Kalles Fraktaler 위치
app-filter-fdn = Fractadyne 위치
app-filter-map = Fractint / KF 팔레트
app-filter-ugr = Ultra Fractal 그라데이션
app-filter-ase = Adobe 색상 견본 교환
app-filter-ggr = GIMP 그라데이션
app-filter-all = 모든 파일

## Importing and saving palette files (shown under the gradient editor)

# The reason a file was refused before reading it.
app-gradient-map-too-large = .map 팔레트라기에는 너무 큽니다
app-gradient-ugr-too-large = .ugr 팔레트 파일이라기에는 너무 큽니다
app-gradient-ase-too-large = .ase 견본 목록이라기에는 너무 큽니다
app-gradient-ggr-too-large = .ggr 그라데이션이라기에는 너무 큽니다
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported-vga = { $file }에서 항목 { $count }개를 가져왔습니다 - 6비트 VGA 표(흰색이 255가 아니라 252)이며, Fractint 자체의 출력과 일치하도록 기록된 그대로 두었습니다.
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported = { $file }에서 항목 { $count }개를 뚜렷한 띠로 가져왔습니다.
# $error: the reader's reason (in English).
app-gradient-map-invalid = .map 팔레트가 아닙니다: { $error }
# $name: the gradient's name in the file; $file: the file's name.
app-gradient-ugr-imported = { $file }에서 “{ $name }”을(를) 가져왔습니다.
# $file: the file's name; $count: how many gradients it holds.
app-gradient-ugr-pick = { $file }에 그라데이션이 { $count }개 있습니다 — 아래에서 하나를 고르십시오.
# $error: the reader's reason (in English).
app-gradient-ugr-invalid = .ugr 팔레트 파일이 아닙니다: { $error }
# $used: the swatches kept; $total: the swatches in the file.
app-gradient-ase-imported-some = 견본 { $total }개 중 { $used }개를 균등한 간격으로 가져왔습니다.
# $count: the swatches in the file.
app-gradient-ase-imported = 견본 { $count }개를 균등한 간격으로 배치하고 혼합해 가져왔습니다.
# $error: the reader's reason (in English).
app-gradient-ase-invalid = .ase 견본 목록이 아닙니다: { $error }
app-gradient-nothing-to-save = 아직 저장할 사용자 정의 그라데이션이 없습니다.
# $file: the saved file's name.
app-gradient-ggr-saved = “{ $file }”을(를) 저장했습니다 — 중간점, 혼합 곡선, 색 공간이 모두 그대로입니다.
# $file: the saved file's name; $count: the segments whose curve could not be kept.
app-gradient-ggr-saved-lossy = “{ $file }”을(를) 저장했습니다 — 하지만 세그먼트 { $count }개가 .ggr 형식으로 표현할 수 없는 베지어 곡선을 사용합니다. 그 세그먼트는 직선 혼합으로 기록했으며, 라이브러리의 사본에는 곡선이 그대로 남아 있습니다.
# $path: the file; $error: the system's reason (often in English).
app-gradient-write-failed = { $path }에 쓸 수 없습니다: { $error }
# $name: the gradient's name; $count: its segments.
app-gradient-ggr-imported = “{ $name }”을(를) 가져왔습니다 - 세그먼트 { $count }개, 그 중간점, 혼합 곡선, 색 공간 포함.
# $error: the reader's reason (in English).
app-gradient-ggr-invalid = .ggr 그라데이션이 아닙니다: { $error }

## The eyedropper's banner across the top of the screen while picking a colour

# $hex: the colour under the pointer, as #rrggbb.
app-eyedropper-banner-color = 색 고르는 중  ·  { $hex }  ·  클릭하면 선택, Esc는 취소
app-eyedropper-banner = 색 고르는 중  ·  클릭하면 선택, Esc는 취소

## The gradient editor window

app-gradient-title = 그라데이션 편집기
# Before the two view buttons, Bar and Ring.
app-gradient-view = 보기
app-gradient-view-bar = 막대
app-gradient-view-bar-hover = 곧은 띠 — 정지점을 정확히 놓기에 가장 좋습니다
app-gradient-view-ring = 고리
app-gradient-view-ring-hover = 팔레트가 순환되기 때문에 고리입니다: 위쪽의 이음부는 렌더러가 한 바퀴를 돌 때마다 건너는 이음매이며, 막대에서는 괜찮아 보이는 그라데이션도 그곳에 뚜렷한 경계가 생길 수 있습니다.
app-gradient-add-lane-hover = 이 선의 아무 곳이나 클릭하면 그 자리에 정지점을 추가합니다. +는 가장 넓은 간격에 하나를 추가합니다.
app-gradient-strip-hover = 표식을 드래그하면 정지점을 옮깁니다 · 띠 자체를 드래그하면 그라데이션 전체를 회전하며 이음매에서 감깁니다 · 더블클릭하면 정지점 추가, 표식을 오른쪽 클릭하면 제거 · 화살표 키로 선택 항목을 조금씩 옮깁니다(Shift는 미세 조정).
app-gradient-remove-selected-hover = 선택한 정지점 제거
app-gradient-ring-hover = 고리 자체를 드래그하면 그라데이션 전체를 회전합니다 — 이음매를 지나간 부분은 반대편으로 돌아 나옵니다 · 표식을 드래그하면 정지점을 옮깁니다 · 바깥 트랙을 클릭하면 그 자리에 정지점을 추가하고, 가운데의 ⊕는 가장 넓은 간격에 하나를 추가합니다 · ⊖는 선택한 정지점을 제거합니다. 위쪽의 속이 빈 표시는 순환 팔레트가 감기는 이음매입니다. 그것이 바로 이음부이므로 제자리에 머물고, 색이 그 아래에서 돕니다.
app-gradient-empty = 아직 사용자 정의 그라데이션이 없습니다 — “프리셋 복사…”, 가져오기, 또는 아래의 저장된 그라데이션으로 시작하면, 그 정지점과 세그먼트별 곡선이 여기에 나타납니다.
# $count: the entries of an imported palette, more than the editor shows stop by stop.
app-gradient-too-many = 가져온 항목 { $count }개 - 정지점 하나씩 편집하기에는 너무 많습니다. 위의 미리 보기를 사용하거나, 프리셋 복사로 새로 시작하십시오.
# The selected segment's heading. $index: its number; $total: how many there are; $left and $right: where it starts and ends, 0.000–1.000.
app-gradient-segment-heading = 세그먼트 { $index }/{ $total }  ·  { $left } → { $right }
# A label before the list of blend curves.
app-gradient-curve = 곡선
app-gradient-curve-hover = 이 세그먼트를 가로질러 색이 얼마나 빨리 변하는지입니다. “곡선형”은 중간점으로 모양이 정해지며, 중간점 0.50에서는 정확히 선형입니다.
# The blend curves (GIMP's kinds), in a narrow list: keep them short.
app-gradient-blend-linear = 선형
app-gradient-blend-curved = 곡선형
app-gradient-blend-sine = 사인
app-gradient-blend-sphere-increasing = 구형 ↑
app-gradient-blend-sphere-decreasing = 구형 ↓
app-gradient-blend-bezier = 베지어
# Hover on Bézier in the list of blend curves.
app-gradient-bezier-exact-hover = 양방향으로 드래그할 수 있는 두 제어점입니다. 이 곡선은 이미 3차 곡선이므로 변환해도 아무것도 바뀌지 않습니다.
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-visible-hover = 양방향으로 드래그할 수 있는 두 제어점입니다. 변환하면 현재 곡선을 근사합니다 — 이 곡선은 오차 { $error } 이내이며, 차이가 뚜렷이 보입니다.
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-barely-hover = 양방향으로 드래그할 수 있는 두 제어점입니다. 변환하면 현재 곡선을 근사합니다 — 이 곡선은 오차 { $error } 이내이며, 차이가 거의 보이지 않습니다.
# A label before the list of colour spaces (RGB, HSV).
app-gradient-space = 색 공간
# A label before the Bézier presets beside it.
app-gradient-ease = 이징
app-gradient-ease-hover = 프리셋은 두 제어점을 기록합니다. 그 밖의 모양은 곡선 위에서 제어점을 드래그하십시오. 베지어에는 중간점이 없으며, 핸들이 절반에 이르는 위치를 정합니다.
# Bézier presets, as in CSS easing: short buttons.
app-gradient-ease-linear = 선형
app-gradient-ease-in = 가속
app-gradient-ease-out = 감속
app-gradient-ease-in-out = 가감속
app-gradient-midpoint = 중간점
app-gradient-midpoint-hover = 혼합이 절반에 이르는 위치로, 이 세그먼트에 대한 비율입니다. 0.50이 가운데입니다.
# A button that puts the midpoint back in the middle of the segment.
app-gradient-centre = 가운데
app-gradient-centre-hover = 0.50으로 되돌리기
app-gradient-no-hue = ⚠ 이 세그먼트의 한쪽 끝에 색조가 없습니다
app-gradient-no-hue-hover = 한쪽 끝이 검정, 흰색 또는 회색이므로, 빨강에서 출발해 색상환 전체를 돕니다. 허용되는 동작이고 때로는 원하는 결과이지만 — 가운데에 양쪽 끝 어디에도 없는 색이 나타날 수 있는 이유가 이것입니다. 곡선 아래의 띠가 그 결과를 보여 줍니다.
app-gradient-segment-start-hover = 이 세그먼트 시작 부분의 색
app-gradient-segment-end-hover = 이 세그먼트 끝 부분의 색
app-gradient-bytes-hover = 0–255로 나타낸 빨강, 초록, 파랑
app-gradient-floats-hover = 같은 색을 0–1로 나타낸 값이며, 렌더러가 저장하는 형식입니다
app-gradient-canvas-bezier-hover = 선택한 세그먼트의 혼합 곡선입니다. 어느 제어점이든 x와 y 양쪽으로 드래그해 모양을 바꿉니다. 흐린 선은 0과 1입니다. 곡선은 그 밖으로 나갈 수 있으며, 그때는 곡선이 아니라 색이 범위 안으로 제한됩니다.
app-gradient-canvas-hover = 선택한 세그먼트의 혼합 곡선: 왼쪽 정지점에서 오른쪽 정지점까지 색이 얼마나 빨리 변하는지입니다. 고리를 드래그해 중간점을 옮기거나, 곡선을 베지어로 바꾸면 양방향으로 움직이는 제어점을 쓸 수 있습니다.
# The selected stop's heading. $index: its number; $total: how many there are.
app-gradient-stop-heading = 정지점 { $index }/{ $total }
app-gradient-stop-color-hover = 이 정지점의 색 — 견본을 클릭해 고르거나, 옆에 16진수 값을 입력하십시오
app-gradient-stop-bytes-hover = 0–255로 나타낸 빨강, 초록, 파랑 — 이 색이 기록되는 바이트입니다
# The eyedropper button.
app-gradient-pick = 스포이트
app-gradient-pick-hover = 스포이트 — 화면 어디에서나 색을 가져옵니다. 원하는 색으로 포인터를 옮기고 클릭하십시오. Esc는 취소합니다.
# Written before the stop's position (0.000–1.000) in its number field: a short abbreviation of "position".
app-gradient-stop-pos = 위치
app-gradient-nudge-hover = 이 정지점을 조금씩 이동(Shift를 누르면 더 미세하게)
app-gradient-remove = 제거
app-gradient-remove-hover = 이 정지점 제거(또는 그 아래의 ⊖ 클릭)
app-gradient-add-stop = 정지점 추가
app-gradient-add-stop-hover = 가장 넓은 세그먼트를 나눕니다(또는 그라데이션 위의 선을 클릭)
# A menu of the built-in palettes, to start from a copy of one.
app-gradient-copy-preset = 프리셋 복사…
app-gradient-paste = 붙여 넣기…
app-gradient-paste-hover = 16진수 색이나 0–255 RGB 세 값으로 팔레트 가져오기
app-gradient-save-ggr = .ggr 저장…
app-gradient-save-ggr-hover = 이 그라데이션을 GIMP .ggr 파일로 기록합니다 — GIMP, Krita, Inkscape, 그리고 이 편집기에서 읽을 수 있습니다
# A menu of the palette file formats (.map, .ugr, .ggr, .ase).
app-gradient-import = 가져오기
app-gradient-import-map-hover = Fractint / Kalles Fraktaler .map 팔레트 파일 불러오기(항목마다 R G B 한 줄)
app-gradient-import-ugr-hover = Ultra Fractal .ugr 그라데이션 파일 불러오기(.ugr에는 보통 여러 그라데이션이 들어 있으며, 그중 하나를 고릅니다)
app-gradient-import-ggr-hover = 중간점, 혼합 곡선, 색 공간을 그대로 유지하며 GIMP .ggr 그라데이션 불러오기
app-gradient-import-ase-hover = Adobe 견본 목록(.ase)을 불러옵니다. 견본에는 위치가 없으므로 균등한 간격으로 배치되고 혼합됩니다.
# The gradient's name, for the library of saved gradients.
app-gradient-name = 이름
# Shown greyed in the empty name field.
app-gradient-name-hint = 내 그라데이션
# The save button, when a gradient of that name is already saved.
app-gradient-update = 업데이트
app-gradient-save = 저장
app-gradient-save-hover = 곡선과 색 공간을 그대로 유지하며 이 그라데이션을 라이브러리에 저장합니다
app-gradient-save-needs-name-hover = 먼저 그라데이션에 이름을 붙이십시오
# A menu of the saved gradients. $count: how many there are.
app-gradient-saved-menu = 저장됨({ $count })
app-gradient-nothing-saved = 아직 저장된 것이 없습니다
app-gradient-delete-saved-hover = 저장된 이 그라데이션 삭제
# $count: the gradient's segments.
app-gradient-rich = 풍부한 그라데이션: 세그먼트 { $count }개에 중간점, 혼합 곡선 또는 색 공간이 있습니다 — 단순한 정지점 목록에 담을 수 있는 것보다 많습니다.
app-gradient-convert = 편집 가능한 정지점으로 변환
app-gradient-convert-hover = 단순한 정지점 목록으로 바꿉니다. 정지점 목록에 담을 수 없는 중간점, 혼합 곡선, 색조 회전은 버려집니다.
# Above the list of gradients in an Ultra Fractal file. $count: how many; $file: the file's name.
app-gradient-ugr-list = { $file }의 그라데이션 { $count }개
app-gradient-close-list = 목록 닫기
app-gradient-seamless = 이음매 없음(끝이 시작과 일치)
app-gradient-seamless-hover = 팔레트는 순환하므로, 화면에서는 위치 1과 위치 0이 서로 붙어 있습니다. 켜면 마지막 정지점의 색을 첫 정지점과 같게 유지합니다 — 고리 보기에서는 이음부가 위쪽에 보입니다.
app-gradient-ends-differ = ⚠ 양 끝이 다름
app-gradient-ends-differ-hover = 그라데이션 끝의 색이 시작의 색과 달라서, 순환 팔레트가 감기는 곳에 뚜렷한 경계가 보입니다.
app-gradient-hard-bands = 뚜렷한 띠(항목 사이 혼합 없음)
app-gradient-hard-bands-hover = Fractint / .map 방식: 모든 항목이 단색입니다. 끄면 부드러운 그라데이션으로 혼합합니다.
app-gradient-paste-intro = 16진수 색(#ff8800)이나 0–255 세 값(255 136 0)을 쉼표, 공백 또는 줄 바꿈으로 구분해 붙여 넣으십시오:
# Applies the pasted colours.
app-gradient-apply = 적용
# Empties the paste box.
app-gradient-clear = 지우기
# $used: the colours kept; $total: the colours pasted; $limit: the most that are kept.
app-gradient-paste-imported-some = 색 { $total }개 중 { $used }개를 목록 전체에서 균등하게 골라 가져왔습니다(최대 { $limit }개).
# $count: the colours pasted.
app-gradient-paste-imported = 색 { $count }개를 가져왔습니다.
# $error: why the pasted text could not be read (in English).
app-gradient-paste-unreadable = 읽을 수 없습니다: { $error }
# A hint at the bottom of the editor. $count: the palette's entries.
app-gradient-bands-hint = 띠 { $count }개 · 파일 순서대로 사용하며, 띠가 켜져 있는 동안 위치는 무시됩니다.
# A hint at the bottom of the editor, in the ring view. $count: the gradient's stops.
app-gradient-ring-hint = 정지점 { $count }개 · 바깥 트랙을 클릭하면 그 자리에 추가 · ⊕는 가장 넓은 간격에 추가, ⊖는 선택한 정지점 제거 · 표식을 드래그해 이동 · 위쪽 표시는 이음매
# A hint at the bottom of the editor, in the bar view. $count: the gradient's stops.
app-gradient-bar-hint = 정지점 { $count }개 · 그라데이션 위의 선을 클릭하면 그 자리에 추가 · 표식을 드래그해 이동 · ⊖ 또는 오른쪽 클릭으로 제거
app-gradient-cancel = 취소
app-gradient-cancel-hover = 이 창을 연 뒤의 모든 변경을 버리고 창을 닫습니다
app-gradient-ok = 확인
app-gradient-ok-hover = 변경 사항을 유지하고 편집기를 닫습니다

## Shared

# The button that closes the Help window and the "Faster deep zoom" window.
app-close = 닫기

## The "Faster deep zoom" window (Help menu): the optional accelerated build

app-accel-title = 더 빠른 딥 줌
app-accel-active-heading = 가속 빌드를 실행하고 있습니다
app-accel-active-body = 딥 줌 기준 궤도를 MPFR/GMP로 계산하고 있으며, 이 단계에서 표준 빌드보다 2.5-6.4배 빠릅니다 - 깊은 뷰가 그려지기 시작하기 전의 멈춤입니다.
# $status: which arithmetic library is in use and its version (in English).
app-accel-in-use = 사용 중인 산술: { $status }
app-accel-fallback-heading = 가속 빌드 — 내장 산술 사용 중
app-accel-fallback-body = 이것은 가속(MPFR) 빌드이지만 수학 라이브러리를 찾을 수 없어 내장 산술(astro-float)을 사용하고 있습니다. 이미지는 동일하며, 깊은 뷰가 그려지기 전의 멈춤만 더 길어집니다.
# Followed by the folder the program runs from, on its own line.
app-accel-fallback-dlls = 더 빠른 경로를 사용하려면 libmpfr-6.dll과 libgmp-10.dll(그리고 libgcc_s_seh-1.dll, libwinpthread-1.dll)을 fractadyne.exe 옆에 둔 뒤 다시 시작하십시오:
app-accel-warning-hidden = 이에 대한 시작 시 경고가 숨겨져 있습니다.
app-accel-show-again = 다시 표시
app-accel-offer-heading = 선택적 빌드는 깊은 곳에서 2.5-6.4배 빠릅니다
app-accel-offer-body = 딥 줌은 CPU에서 기준 궤도를 계산하는 데 많은 시간을 씁니다 - 깊은 뷰가 그려지기 시작하기 전의 멈춤입니다. 선택적 빌드는 이 계산을 MPFR/GMP 라이브러리로 수행하며, 2.5배에서 6.4배 빠르고 깊이 들어갈수록 더 빨라집니다. 그 밖의 모든 것은 같습니다.
app-accel-offer-identical = 이미지는 바이트 단위까지 동일합니다 - 같은 수학을 더 빠른 라이브러리로 계산한 것이며, 모든 프랙털 수식과 딥 줌 비교 자료 전체에서 확인했습니다.
app-accel-offer-shared = 설정, 저장된 세션, 위치는 두 빌드가 공유하므로 자유롭게 오갈 수 있고, 아무것도 가져올 필요가 없습니다.
# Shown on Windows.
app-accel-separate-windows = 별도로 다운로드하는 이유는, 사용하는 라이브러리를 표준 Windows 빌드의 컴파일러로 빌드할 수 없고, Fractadyne 자체와 다른 라이선스(GNU LGPL v3)를 따르기 때문입니다.
# Shown on other systems.
app-accel-separate = 별도로 다운로드하는 이유는, 사용하는 라이브러리가 Fractadyne 자체와 다른 라이선스(GNU LGPL v3)를 따르기 때문입니다.
app-accel-download = 이 버전용 다운로드
app-accel-all-releases = 모든 릴리스
app-accel-all-releases-hover = 직접 링크가 404를 반환하면 이 버전에는 아직 가속 빌드가 없는 것입니다 - 여기서 가장 가까운 버전을 고르십시오.
# Shown on Windows.
app-accel-install-windows = 압축을 풀고 그 폴더에서 fractadyne.exe를 실행하십시오. .dll 파일은 그 옆에 그대로 두십시오.
# Shown on Linux.
app-accel-install-linux = 압축을 풀고 그 폴더에서 ./fractadyne을 실행하십시오. 시스템의 GMP와 MPFR(libgmp10 / libmpfr6)을 사용하며, 거의 모든 데스크톱에 이미 설치되어 있습니다 - 패키지의 README-ACCELERATED.txt를 참조하십시오.
# Shown on other systems.
app-accel-install-other = 이 플랫폼용 가속 패키지는 아직 빌드되지 않았습니다. --features fractadyne-core/rug로 소스에서 빌드할 수 있습니다.

## The Help window's frame (its sections are under help-)

app-help-title = Fractadyne 도움말

## Checking for updates

app-toast-up-to-date = 최신 버전을 사용하고 있습니다.
# $error: why the check failed (in English).
app-toast-update-failed = 업데이트 확인 실패: { $error }
# The window that offers a newer version.
app-update-title = 업데이트 있음
# $version: the new version (0.3.0-beta.20). $channel: the token "stable" or "beta", in English; a translation can choose its words by it ({ $channel -> [beta] … *[other] … }).
app-update-version =
    { $channel ->
        [beta] Fractadyne { $version } (베타) 버전을 사용할 수 있습니다
       *[other] Fractadyne { $version } (안정판) 버전을 사용할 수 있습니다
    }
# $current: the version running now. $track: the update track the user follows (its name in the language in use).
app-update-running = 현재 실행 중: { $current }  ·  { $track }
app-update-download-intro = GitHub에서 최신 릴리스를 다운로드하십시오:
app-update-download = GitHub에서 다운로드
app-update-later = 나중에 알림

## The second graphics card (shown in its window, after "In use:" or "Could not start:")

# The second device is the same card as the window's. $name: the card's name.
app-worker-same-card = 같은 카드, { $name }
# Why the second device could not start.
app-worker-thread-stopped = 장치 스레드가 멈춤

## Export status (shown in the Export image window)

app-export-gpu-unavailable = GPU를 사용할 수 없음
app-export-reference-aborted = 내보내기 실패: 기준 궤도 계산이 중단되었습니다.

## The performance panel (View ▸ Performance panel)
## Its row labels sit in a monospace column about 14 characters wide: keep them short (a longer label pushes its value to the right).

app-perf-fps = FPS
app-perf-fps-hover = 프레임 사이의 평활화된 간격으로 계산한, 초당 표시된 프레임 수입니다. 뷰가 안정화되는 동안에는 대신 다듬기 진행 상황을 보여 줍니다 - 안정화 중인 뷰는 여기의 숫자가 암시하는 속도로 돌고 있지 않습니다.
# What the FPS row shows while a view is settling. $done and $total: tiles; $eta: the time left, as m:ss.
app-perf-refining-tiles = 다듬는 중 { $done }/{ $total } (~{ $eta })
app-perf-refining-arming = 다듬는 중(준비)
# $percent: how much is done, with one decimal.
app-perf-refining-percent = 다듬는 중 { $percent }%
app-perf-building-reference = 기준 궤도 계산 중
app-perf-idle = 유휴
app-perf-frame = 프레임
app-perf-frame-hover = 프레임 사이의 평활화된 실제 경과 시간입니다 - 아래 행들의 합이 아닙니다. 표시를 기다리는 시간도 포함되므로, 안정된 유휴 뷰에서는 어떤 작업이 아니라 다시 그리기 주기를 반영합니다.
app-perf-cpu = cpu
app-perf-cpu-hover = 한 번의 업데이트 안에서 쓰는 메인 스레드 시간: 매개변수 설정과 기준 궤도 캐시 관리. 기준 궤도를 다시 계산할 때 치솟는 행이며, 여기서 치솟는 것이 바로 체감되는 끊김입니다.
app-perf-gpu-idle = gpu/유휴
app-perf-gpu-idle-hover = 프레임 시간에서 메인 스레드 CPU 시간을 뺀 값입니다. 뷰가 렌더링되는 동안은 GPU 작업이지만, 안정된 뷰에서는 대부분 유휴 대기입니다 - 이 뺄셈만으로는 둘을 구별할 수 없어서 이름에 둘 다 적었습니다.
app-perf-timing = 시간 측정
# How frame cost is measured, the timing row's value. TIMESTAMP_QUERY is a name.
app-perf-timing-starved = 실제 시간(타임스탬프 부족)
app-perf-timing-gpu = GPU 타임스탬프
app-perf-timing-no-query = 실제 시간(TIMESTAMP_QUERY 없음)
app-perf-timing-hover = 프레임 비용을 측정하는 방식입니다. GPU 타임스탬프가 정밀한 출처이며, 측정값이 더 이상 도착하지 않으면 실제 시간으로 대체합니다. 실제 시간으로 매긴 예산은 깊은 프레임의 크기를 여러 번 잘못 잡았습니다 — 여기가 빨간색이면 ‘지금은 안전 여유가 추정값’이라는 뜻으로 받아들이십시오.
app-perf-mode = 모드
app-perf-mode-hover = 마지막 프레임을 렌더링한 산술 경로: direct(기준 궤도 없음), df32 섭동, 또는 floatexp - 약 1e28x를 넘으면 쓰는 확장 범위 경로로, BLA가 동작하는 유일한 경로입니다.
app-perf-eff-iter = 실효 반복
app-perf-eff-iter-hover = 깊이에 따른 조정과 움직이는 동안의 제한을 적용한 뒤, 마지막 프레임이 실제로 사용한 반복 횟수입니다 - 품질의 기본 설정이 아니며, 그것은 출발점일 뿐입니다.
app-perf-precision = 정밀도
app-perf-precision-hover = 기준 궤도를 계산하는 가수 비트 수입니다. 깊이에 따라 늘어나며, 64비트는 얕은 곳의 최솟값이지 부딪힌 한계가 아닙니다.
app-perf-orbit-len = 궤도 길이
# $cap: the most orbit samples this GPU holds, already formatted, or the text of app-perf-uncapped.
app-perf-orbit-len-hover = 기준 궤도 샘플 수입니다. 이 GPU가 담을 수 있는 최대치는 { $cap }입니다 — 여기에 가까워지는 것이 실질적인 깊이의 벽입니다: 이를 넘으면 더 긴 궤도가 필요한 더 깊은 위치를 라이브 뷰가 그려 낼 수 없습니다.
app-perf-uncapped = (제한 없음)
app-perf-sa-skip = SA 건너뜀
app-perf-sa-skip-hover = 픽셀별 작업이 시작되기 전에 급수 근사가 건너뛴 반복 횟수입니다. 클수록 빠르며 이미지는 전혀 바뀌지 않습니다. 건너뛴 것이 없으면 이 행은 표시되지 않습니다.
app-perf-aa = aa
app-perf-aa-hover = 슈퍼샘플링: 각 픽셀을 축마다 이만큼의 샘플로 렌더링하므로, 2x는 작업량이 네 배입니다.
app-perf-dual = 이중
app-perf-dual-hover = Mandelbrot/Julia 분할 뷰가 실행 중인지 여부입니다. true이면 이 패널의 모든 비용은 두 창을 모두 포함합니다.
app-perf-zoom = 줌
app-perf-zoom-hover = 전체 뷰에 대한 배율입니다. 위의 산술 모드는 약 1e4x에서, 그리고 다시 약 1e28x에서 바뀝니다.
app-perf-iterate = 반복 계산
# $target and $lethal: times in milliseconds, already formatted.
app-perf-iterate-hover = 마지막 반복 디스패치의 측정된 GPU 시간입니다. 제어기는 { $target } ms를 목표로 합니다. { $lethal } ms 근처의 값이 계속되면, 이 등급의 하드웨어가 장치를 잃었던 구간입니다.
app-perf-steps = 스텝/s
app-perf-steps-hover = GPU의 초당 반복 스텝 수(10억 단위)입니다. 내부가 많은 뷰는 숫자만 보고 예상하는 것보다 스텝당 더 느리므로, 이것은 점수가 아니라 처리량 측정값입니다.
app-perf-budget = 예산
# After the budget's value while it is still converging; the hover below quotes it.
app-perf-budget-settling = (안정화 중)
app-perf-budget-hover = 디스패치마다 적응하는 스텝 예산으로, 드라이버가 GPU를 재설정할 만큼 긴 프레임을 막는 제어기의 안전장치입니다. “안정화 중”은 아직 드문 측정값에서 수렴하는 중이며 믿을 수 있는 상한이 아니라는 뜻입니다.
app-perf-rss = rss
# $mb and $peak: memory in megabytes, already formatted.
app-perf-rss-value = { $mb } MB(최고 { $peak })
app-perf-rss-hover = 프로세스의 상주 메모리와 이번 세션의 최고치입니다. 실제 사용 환경에서 깊은 기준 궤도 계산이, 화면의 다른 어떤 표시도 없이 2 GB를 넘은 적이 있습니다. 시스템 RAM의 4분의 3을 넘으면 호박색입니다.
app-perf-gpu-est = gpu 추정
app-perf-gpu-est-hover = GPU에 상주하는 메모리 추정치: 기준 궤도, BLA 트리, 반복 텍스처, 다듬기 중의 청크 상태 핑퐁 버퍼, 표시 게이트 스냅샷. 알려진 할당 크기로 합산한 값이며, 드라이버의 실제 수치는 이식 가능한 방법으로 읽을 수 없습니다.
app-perf-julia-re = julia c.re
app-perf-julia-re-hover = Julia 창이 렌더링하는 매개변수 c의 실수부입니다 - 고정하지 않았다면 커서가 가리키는 점입니다.
app-perf-julia-im = julia c.im
app-perf-julia-im-hover = Julia 창이 렌더링하는 매개변수 c의 허수부입니다.
app-perf-julia-c = julia c
# The julia c row's value when the parameter is held fixed.
app-perf-pinned = 고정됨
app-perf-pinned-hover = 매개변수가 한 점에 고정되어 있으므로, 커서를 움직여도 Julia 창이 더 이상 바뀌지 않습니다.
app-perf-c-panel = c/패널
app-perf-c-panel-hover = Mandelbrot 패널 전체가 차지하는 c 공간의 너비입니다. 이 값이 Julia 픽셀 하나보다 훨씬 작으면, 커서를 올려 c가 바뀌어도 Julia는 그대로 보입니다 — 딥 줌에서는 예상된 동작이며 멈춘 것이 아닙니다.
app-perf-ref-recompute = 기준 재계산
app-perf-ref-recompute-hover = 가장 최근 기준 궤도 재계산에 걸린 시간입니다. 깊은 뷰가 그려지기 시작하기 전의 멈춤이며, 선택적 가속 빌드가 빠르게 하는 단계입니다.
app-perf-recompute-rate = 재계산/s
app-perf-recompute-rate-hover = 초당 설치된 기준 궤도 수 - 결과가 실제로 사용된 것입니다. 아래 행과 비교하십시오: 둘의 차이가 버려진 작업입니다.
app-perf-builds-rate = 기준 계산/s
app-perf-builds-rate-hover = 초당 시작된 기준 궤도 계산 수로, 결과가 버려진 것도 포함합니다 - 다이브가 끝내 도달하지 않는 미리 보기 슬롯이나 거부된 확장 등. 이것들은 CPU를 온전히 쓰면서도 다른 어디에도 나타나지 않습니다: 한번은 미리 가져오기가 헛돌면서 6분 동안 초당 약 400개를 계산했는데, 재계산/s는 2를 표시했습니다. 한 자릿수가 정상이며, 두 행의 차이가 크면 작업이 버려지고 있다는 뜻입니다.
app-perf-recompute-total = 재계산 합계
app-perf-recompute-total-hover = 실행 이후의 기준 궤도 재계산 횟수입니다. 정해진 시간 동안의 비율로 보면 유용합니다. 아무것도 움직이지 않는데 숫자가 오르면 추적할 가치가 있는 신호입니다.

## Help: the contents list (Help & reference…, F1)

# Each section's name in the contents list (about 165 px wide: keep them short), and its heading.
help-section-overview = 개요
help-section-navigation = 탐색
help-section-expressions = 좌표 식
help-section-options = 채색 및 옵션
help-section-fractals = 프랙털
help-section-how-it-works = 작동 원리
help-section-command-line = 명령줄
help-section-shortcuts = 단축키
help-section-hardware = 권장 하드웨어
# Credits to the prior work Fractadyne builds on.
help-section-acknowledgments = 감사의 말
# The third-party license notices.
help-section-licenses = 라이선스
help-section-about = 정보

## Help ▸ Overview

help-overview-intro = Fractadyne은 극도로 깊은 줌과 속도를 위해 만든 네이티브 프랙털 탐색기입니다. “탈출 시간” 프랙털을 그립니다 — 모든 픽셀에서 하나의 간단한 수식을 반복해 만들어지는 이미지입니다.
help-overview-what-is = 탈출 시간 프랙털이란?
help-overview-what-is-text = 프로그램은 각 픽셀마다 z → z² + c 같은 수식을 0에서 시작해 거듭 계산합니다. 계산 중인 값이 영원히 작게 머물면 그 픽셀은 집합에 속하며 어둡게 그려집니다. 결국 한없이 커지면(“탈출”하면) 그 픽셀은 집합 밖에 있으며, 그 색은 탈출하기까지 걸린 단계 수를 나타냅니다. “머무름”과 “탈출” 사이의 무한히 세밀한 경계가 바로 프랙털입니다.
help-overview-can-do = 할 수 있는 일
help-overview-can-pan = 사실상 한계 없이 이동하고 확대/축소합니다(위치는 어떤 깊이에서도 정확합니다).
help-overview-can-families = 25가지 내장 프랙털 계열(10가지, 그리고 그중 5가지의 고차 거듭제곱), 사용자 수식, Life와 L-시스템 사이를 전환하고, Newton을 제외한 모든 탈출 시간 계열을 Julia 집합으로 볼 수 있습니다.
help-overview-can-recolor = 프리셋 또는 사용자 정의 그라데이션과 여러 채색 방식으로 색을 바꿉니다.
help-overview-can-lighting = 3D 입체 조명과 빛나는 경계 등고선을 더합니다.
help-overview-can-minibrots = 미니브로트에 맞추고, 지점을 북마크하고, 고해상도 이미지를 내보냅니다.
help-overview-can-autopilot = 자동 줌 자동 조종이 스스로 디테일을 향해 다이브하게 합니다.
help-overview-can-share = 어느 지점이든 작은 “.fdn” 텍스트 조각이나 파일로 공유합니다.
help-overview-can-tours = 스크립트 투어와 하드웨어 벤치마크를 실행합니다.
help-overview-first-steps = 첫걸음
# "Seahorse Valley" is a place in the Navigate menu: use its name as that menu shows it. A and F1 are keys.
help-overview-first-steps-text = 탐색 메뉴를 열고 “Seahorse Valley”를 고른 다음, 마우스 휠을 굴려 확대하십시오. 드래그하면 이동합니다. 또는 A를 누르고 자동 조종이 대신 다이브하게 두십시오. 언제든 F1을 누르면 이 도움말로 돌아옵니다.

## Help ▸ Coordinate expressions
## The expressions, numbers and function names here are typed as shown in every language:
## keep them exactly as written. Menu items in quotation marks are the menus' own labels.

help-expr-intro = Fractadyne이 좌표를 받는 곳이라면 어디서든 소수 대신 식을 입력할 수 있습니다: 탐색 → “위치로 이동…”의 Re / Im 입력란(그리고 극좌표 모드의 각 입력란), .fdn 위치의 center_re / center_im 줄, 투어 스크립트의 re / im, 그리고 명령줄의 --center. 식은 줌에 필요한 만큼의 자릿수로 계산되며, 이동 대화 상자는 식을 뷰와 함께 보관합니다 — 그래서 1/3이나 -0.5 + 0.25*cos(pi/4)로 입력한 점은 아무리 깊이 확대해도 정확히 그 점에 머물지만, 소수는 자릿수가 바닥나면 어긋나 버립니다.
help-expr-examples = 예
help-expr-ex-third = 정확한 3분의 1. 확대할수록 자릿수가 더 나타납니다.
help-expr-ex-neck = Seahorse Valley의 목 — 주기 2 벌브의 뿌리점.
help-expr-ex-circle = -0.5를 중심으로 반지름 1/4인 원 위의 한 점(극좌표 모드가 이 식을 대신 만들어 줍니다).
help-expr-ex-bulb = 주기 3 벌브 뿌리점의 Re. 그 Im은 sin(2*pi/3)/2 - sin(4*pi/3)/4입니다. 탐색 → “정확한 점(식)”에 이런 점들이 들어 있습니다.
help-expr-ex-complex = Re 입력란에 복소수 값 전체를 입력하면 Re와 Im이 모두 채워집니다.
help-expr-ex-combine = 함수와 상수를 자유롭게 조합할 수 있습니다.
help-expr-ex-powers = 거듭제곱: 2^-20은 1/1048576입니다. -2^2는 -4입니다 — 마이너스는 거듭제곱 다음에 적용됩니다.
help-expr-ex-root = -2, 실수 세제곱근. 음수의 짝수 제곱근은 거부되며, sqrt(-1)은 i가 됩니다.
help-expr-ex-scientific = 과학적 표기법도 식 안에서 쓸 수 있습니다.
help-expr-numbers = 숫자와 연산자
# The decimal point in an expression is '.' in every language: keep that rule as it is.
help-expr-numbers-decimals = 지수를 붙일 수 있는 소수: 0.25, -1.5e-30, .5. 숫자만 씁니다 — 숫자 안에 공백이나 자릿수 구분 쉼표를 넣을 수 없으며, 소수점은 항상 '.'입니다.
help-expr-numbers-imaginary = 허수는 i 접미사를 붙이거나(0.25i), i만 따로 씁니다: 3 + 4*i.
# Starts with the operators themselves, two spaces apart.
help-expr-numbers-operators = +  -  *  /  그리고 ^(거듭제곱, 오른쪽에서 왼쪽으로 계산하므로 2^3^2 = 2^9). 괄호로 묶습니다. 암시적 곱셈은 없습니다: 2pi가 아니라 2*pi로 쓰십시오.
help-expr-numbers-case = 이름은 대소문자를 구분하지 않습니다: PI, Pi, pi는 같은 상수입니다.
help-expr-constants = 상수
help-expr-functions = 함수
help-expr-functions-angles = 각도는 라디안입니다. 도 단위로 쓰려면 pi/180을 곱하거나 — cos(45*pi/180) — 이동 대화 상자를 극좌표 모드로 바꾸고 도 단위를 고르십시오.
help-expr-refused = 거부되는 것과 그 이유
help-expr-refused-fractional = (-2)^(1/2)처럼 음수의 분수 거듭제곱: 결과가 복소수이며 고를 수 있는 근이 여럿입니다. 실수 근에는 root(x, n)이나 cbrt(x)를, 주 복소수 근에는 sqrt(x)를 쓰십시오.
help-expr-refused-complex = 실숫값 함수에 대한 복소수 인수(sin(i), ln(1+i)). 복소수 인수를 받는 것은 sqrt와 abs뿐이며, 산술 연산자와 정수 거듭제곱은 복소수 값에도 동작합니다.
help-expr-refused-domain = 정의역 오류: 양수가 아닌 수의 ln이나 log, -1…1 밖의 asin이나 acos, 0의 음수 거듭제곱, 0으로 나누기.
help-expr-refused-bounds = ±2^32를 넘는 sin, cos, tan, exp 또는 ^의 인수, 그리고 32단계보다 깊은 중첩: 붙여 넣은 좌표는 신뢰할 수 없는 입력이며, 이런 것들은 시간이 끝없이 걸릴 수 있습니다.
help-expr-refused-grammar = 변수, 사용자 정의 함수, 비교, 조건문은 없습니다. 저장된 위치가 모든 버전에서 같은 의미를 갖도록 문법을 일부러 작게 유지합니다.
help-expr-no-parse = 해석되지 않을 때
help-expr-no-parse-text = 메시지는 입력란을 지목하고, 무엇이 몇 번째 문자에서 발견되었는지 알려 주며 — 흔한 실수라면 — 대신 무엇을 입력해야 하는지도 알려 줍니다: 문서에서 붙여 넣은 인쇄용 마이너스 기호(−), × 또는 ÷, 소수점 쉼표, 숫자와 이름 사이에 빠진 *, 닫히지 않은 괄호, 괄호 없이 쓴 함수. 절반만 읽는 일은 없습니다: 텍스트의 어느 부분이라도 이해할 수 없으면, 조용히 잘라 내는 대신 좌표 전체를 거부합니다.

## Help ▸ Navigation
## Menu paths (Navigate → "Go to location…") and quoted items name the interface's own
## menus and controls: use their translations. Key names (Ctrl, Shift, Space, Esc) stay.

# A heading: the mouse controls (also in Shortcuts).
help-mouse = 마우스
# The left column of a controls table: a gesture or key. Keep key names (Shift, Space, Ctrl) as they are.
help-key-left-drag = 왼쪽 드래그
help-nav-left-drag = 뷰를 이동합니다.
help-key-mouse-wheel = 마우스 휠
help-nav-wheel = 커서 쪽으로 확대/축소합니다. 빨리 돌릴수록 한 칸에 더 많이 줌합니다. 특징점에 줌 맞추기를 켜면, 미니브로트 근처에서 확대할 때 줌이 미니브로트 쪽으로 끌려갑니다.
help-key-shift-drag = Shift+드래그
help-nav-shift-drag = 영역 줌 — 사각형을 드래그하면 그 영역이 뷰를 채우도록 줌합니다.
# $icon: the click-to-zoom toolbar button's symbol.
help-key-click-zoom = 클릭하여 줌({ $icon })
help-nav-click-zoom = 도구 모음의 과녁 버튼을 켭니다(단일 뷰): 그러면 왼쪽 클릭은 설정된 배수만큼 그 점으로 다이브하고, 오른쪽 클릭은 되돌아 나옵니다. 드래그는 그대로 이동입니다. 배수(2–100×)는 오른쪽 패널의 탐색에서 고릅니다. Backspace는 클릭 한 번을 되돌립니다.
help-nav-continuous = 연속 줌과 홈
help-key-hold-space = Space 누르고 있기
help-nav-hold-space = 커서를 기준으로 부드럽게 확대합니다.
help-key-hold-shift-space = Shift+Space 누르고 있기
help-nav-hold-shift-space = 부드럽게 축소합니다.
help-nav-continuous-text = 홈으로 축소 버튼은 전체 뷰로 부드럽게 날아 돌아가는 애니메이션을 보여 줍니다. 오른쪽 패널의 “줌 속도”는 연속 줌의 속도를 정합니다.
help-nav-autopilot = 자동 조종
help-nav-autopilot-text = A를 누르면(또는 도구 → “자동 줌(자동 조종)”) 손을 쓰지 않는 다이브가 시작됩니다: 프랙털 경계에서 디테일이 가장 조밀한 점을 뷰 가운데를 우선해 고르고, 그 점을 화면 중앙으로 서서히 옮기면서 미끄러지듯 들어갑니다. 목표가 흥미로운 동안에는 그 목표를 유지하고, 더 풍부한 구조가 나타나면 부드럽게 그쪽으로 방향을 틉니다. 탐색 입력이 하나라도 있거나 Esc를 누르면 멈춥니다. 오른쪽 패널(또는 도구 아래)의 “자동 줌 우선순위”는 무엇을 우선할지 정합니다: 속도는 설정한 속도로 줌하며 디테일이 있는 프레임이 준비되는 즉시 화면을 갱신하고, 품질은 완전히 렌더링된 최대 해상도 프레임만 보여 주며 다음 프레임이 도착하기 전에 그림이 지나치게 확대되지 않도록 필요한 만큼 줌을 늦춥니다. 어느 쪽이든 화면에 빈 프레임이나 단색 프레임이 나타나지 않습니다: 아직 디테일이 그려지지 않은 갱신은 마지막 프레임 뒤에 숨겨 둡니다. “자동 줌 목표”는 무엇을 겨냥할지 정합니다: 디테일(뷰에서 가장 풍부한 경계로, 진행하면서 다시 고름) 또는 Misiurewicz 점 — 가장 가까운 Misiurewicz 점을 찾아 제자리에 멈춘 채 다이브 한도의 깊이까지 구한 다음, 정확히 그 점으로 다이브합니다. 그런 점 주변의 구조는 몇 옥타브마다 영원히 반복되므로, 이 다이브는 결코 막다른 곳에 이르지 않습니다(Mandelbrot 집합만).
help-nav-history = 기록과 정밀한 이동
help-nav-undo = 이전 뷰로 되돌립니다.
help-nav-redo = 다시 실행합니다.
help-nav-goto = 탐색 → “위치로 이동…”에서 정확한 중심과 줌을 읽고, 입력하고, 붙여 넣고, 복사할 수 있습니다(완전한 정밀도, 어떤 깊이든). 중심 입력란에는 일반 숫자 이상을 쓸 수 있습니다: “1/3” 같은 분수와 pi, tau, e, sqrt, sin/cos/tan으로 만든 식 — 그래서 “-0.5 + 0.25*cos(pi/4)”도 올바른 좌표입니다. 식은 뷰와 함께 보관되어 깊이마다 필요한 정밀도로 다시 계산되므로, 그렇게 입력한 점은 아무리 깊이 확대해도 정확하게 유지됩니다(반올림한 소수는 결국 그렇지 못합니다). 극좌표 모드(오프셋 + r ∠ θ, 도 / 라디안 / 회전 단위)가 이런 식을 대신 만들어 줍니다. 함수와 상수의 전체 목록과 예는 이 도움말의 “좌표 식” 절에 있습니다(대화 상자의 ? 버튼으로 열 수 있습니다). 탐색 → 북마크는 위치를 저장하고 불러옵니다.
help-nav-exact-points = 탐색 → “정확한 점(식)”은 반올림한 소수가 아니라 정확하게 정의된 점으로 이동합니다 — 주 카디오이드 벌브의 뿌리점, 즉 주기 q인 싹이 붙는 곳을 cos/sin 식으로 쓴 것입니다. 식이 뷰와 함께 다니므로, 그 점으로 확대해도 점에서 벗어나지 않습니다.
help-nav-share = 탐색 → “위치 공유…”는 뷰 전체 — 프랙털, 완전 정밀도 중심, 줌, 채색 — 를 간결한 “.fdn” 조각으로 담아, 복사하거나, 다시 붙여 넣어 적용하거나, 파일로 저장/불러오기 해서 지점을 정확히 재현할 수 있게 합니다(또는 다른 사람에게 보낼 수 있습니다).
help-nav-open = 파일 → “뷰 또는 위치 열기…”는 다음을 바로 엽니다: 내보낸 PNG/EXR(포함된 뷰를 복원), “.fdn” 위치 파일, 또는 Kalles Fraktaler “.kfr”.
help-nav-finding = 디테일 찾기
help-nav-minibrots = 미니브로트는 경계를 따라 숨어 있는 집합 전체의 작은 복제본입니다. 하나를 대략 가운데에 두고 M을 누르면(또는 도구 → “미니브로트 찾기 + 확대”) 뉴턴 방법으로 정확히 그 중심에 맞추고, 주기를 읽고, 그 자체의 배율까지 곧바로 줌합니다 — 한 번에 여러 자릿수를 건너뛰는 경우가 많습니다. 이미 미니브로트보다 깊이 있다면 중심만 이동합니다.
help-nav-minimap = 보기 → “미니맵 개요”는 “현재 위치” 표시와 현재 줌 깊이가 담긴 집합 전체의 작은 지도를 보여 줍니다. 지도를 클릭하면 해당 영역으로 이동합니다.

## Help ▸ Coloring & options
## Names in quotation marks are the controls' own labels in the right-hand panel.

help-options-intro = 이 모든 설정은 오른쪽 패널에 있습니다(세션이 바뀌어도 유지됩니다).
help-options-palette = 팔레트
help-options-palette-text = 팔레트는 프리셋 그라데이션, 사용자 정의 그라데이션, 또는 두 가지 색 모드를 고릅니다. “그라데이션 편집…”은 각 색상 정지점에 색과 위치(0–1)가 있는 편집기를 엽니다. 정지점을 추가하거나, 프리셋을 복사해 출발점으로 삼거나, 팔레트 파일을 가져올 수 있습니다.
help-options-paste = 그라데이션 편집기의 “붙여 넣기…”는 다른 곳에서 찾은 팔레트를 가져옵니다. 앞에 #이 있든 없든 16진수 색(#ff8800, ff8800, 또는 짧은 #f80)과 “255 136 0” 같은 일반 0–255 RGB 세 값 — Fractint와 Kalles Fraktaler .map 파일이 쓰는 형식 — 을 쉼표, 공백 또는 줄 바꿈으로 구분해 받습니다. 최대 256색을 보관하며, 더 긴 목록은 처음과 마지막 색을 유지한 채 256개로 균등하게 추려 그라데이션의 모양이 유지되게 합니다.
help-options-import-map = 그 옆의 “가져오기 ▸ .map…”은 Fractint나 Kalles Fraktaler .map 팔레트 파일을 불러옵니다. 이 파일은 항목 사이의 혼합 없이 반복 횟수로 색인하는 조회 표이며, 그로 인해 생기는 뚜렷한 색 단계가 바로 고전적인 Fractint 느낌을 만듭니다 — 그래서 그렇게 가져오며, “뚜렷한 띠” 확인란으로 같은 색들을 대신 부드러운 그라데이션으로 혼합할 수 있습니다. Fractint 자체 파일은 6비트 색 값에 4를 곱해 저장하므로 흰색이 255가 아니라 252입니다. Fractadyne은 그런 파일을 가져올 때 그렇다고 알려 주고 값을 기록된 그대로 두는데, Fractint 자체가 저장한 이미지에도 같은 252가 들어 있기 때문입니다.
help-options-duotone = 듀오톤은 값을 부드러운 두 가지 색 경사(그림자 → 하이라이트)에 매핑합니다. 이진(집합)은 단순한 두 가지 색 보기입니다 — 집합 안의 점은 한 단색, 밖의 점은 다른 단색이며 그라데이션이 없습니다 — 집합의 모양을 가장 분명하게 보는 방법입니다.
help-options-cycle = 순환은 반복 범위에 걸쳐 그라데이션이 몇 번 되풀이되는지 정합니다(촘촘하거나 느슨한 띠). 오프셋은 그라데이션 전체를 회전합니다.
help-options-normalize = “깊은 색상 정규화”는 뷰에 실제로 있는 탈출 값의 범위에 팔레트를 맞춥니다. 깊은 뷰는 수십만 회의 반복에 걸쳐 있으며, 이것이 없으면 고정된 순환이 이웃한 픽셀 사이에서 그라데이션을 수천 번 감아, 올바른 디테일을 반점으로 바꿔 버립니다.
help-options-log-scale = “로그 색상 척도”는 선형 대신 탈출 값의 로그로 팔레트를 펼칩니다. 깊이 들어갈수록 탈출 횟수가 높은 쪽으로 몰리므로, 선형 맞춤은 그라데이션의 대부분을 경계에 붙은 얇은 껍질에 쓰고 나머지는 거의 평평하게 남깁니다. 로그 척도는 이를 펼쳐 주며, 범위가 커지는 줌 영상에서도 색을 더 안정적으로 유지합니다.
help-options-animate = 애니메이션은 시간에 따라 색을 순환합니다 — 정방향, 역방향, 핑퐁은 오프셋을 옮기고, 무작위 그라데이션은 부드럽게 변하는 조화로운 그라데이션을 계속 합성합니다. 속도는 초당 순환(또는 변형) 횟수를 정하며, “그라데이션 섞기”는 무작위 모드에서 새 그라데이션을 만듭니다.
help-options-method = 채색 방식 — 데이터가 색이 되는 방법
# The coloring methods, by the names the Coloring panel gives them, each with what it does.
help-method-smooth = 부드러운 반복
help-method-smooth-text = 탈출 시간에 따른 고전적인 연속 띠.
help-method-stripe = 스트라이프 평균
help-method-stripe-text = 궤도의 각도에서 나오는 흐르는 띠(스트라이프 밀도 슬라이더).
help-method-triangle = 삼각 부등식
help-method-triangle-text = 각 단계가 경계값 사이 어디에 놓이는지에서 나오는 섬세한 질감.
help-method-trap = 궤도 트랩
help-method-trap-text = 궤도와 도형(점/십자/원) 사이의 거리. 내부도 채색합니다.
help-method-distance = 거리 추정
help-method-distance-text = 경계에 얼마나 가까운지에 따라 음영을 줍니다.
help-method-decomposition = 분해
help-method-decomposition-text = 마지막 탈출 각도로 나뉜 칸.
help-options-relief = 3D 입체 조명
help-options-relief-text = 경계의 기울기(도함수)로 표면에 음영을 주어, 돋을새김에 빛을 비춘 듯한 모습을 만듭니다. 조명 각도는 방향을, 입체감은 강도를 정하며(낮을수록 날카로움), “조명 회전”은 이를 움직입니다. 정칙 계열(Mandelbrot / Multibrot)만 해당됩니다.
help-options-glow = 거리 글로우
help-options-glow-text = 경계 근처에서 빛나는 가는 실처럼 조밀해지는 밝은 등고선 띠입니다. 글로우는 혼합 정도, 띠 너비는 간격을 정하며, “글로우 애니메이션”은 띠를 흐르게 합니다.
help-options-quality = 품질과 반복
help-options-quality-text = 반복 횟수는 픽셀을 집합 안으로 판정하기 전까지의 최대 단계 수입니다. “줌에 따라 반복 횟수 자동 조정”은 확대할수록 이를 자동으로 올립니다(더 깊은 디테일에는 더 많이 필요합니다). 안티앨리어싱은 뷰가 안정되면 정지 이미지를 슈퍼샘플링해(2×–8×) 섬세한 외부 “먼지”를 줄입니다.
help-options-other = 기타
help-options-other-text = 줌 속도는 연속 줌 속도를 정하고, 프레임 속도 제한은 프레임 속도를 제한하며, “둘 다, 연결”은 Mandelbrot 집합과 그 Julia 집합을 나란히 보여 줍니다(커서가 Julia 매개변수를 실시간으로 정합니다).

## Help ▸ Fractals
## Fractal family names (Mandelbrot, Multibrot, Tricorn, Burning Ship, Celtic, Buffalo,
## Phoenix, Newton) are names: keep them as written, as the formulas. Formula syntax
## (z = z^3 - p1*z + c, sqr, |z|, if / elseif / endif, p1-p5, maxit) is typed as shown.

help-fractals-intro = 따로 적지 않는 한, 모든 계열은 z를 0에서 시작하고 c를 픽셀로 정해 수식을 반복합니다(탈출 시간). z = x + iy는 복소수입니다.
help-fractals-mandelbrot = z → z² + c. 원조입니다. 집합은 궤도가 유계로 머무는 모든 c이며, 연결되어 있고, 그 경계는 너무 구불구불해서 하우스도르프 차원이 2입니다.
help-fractals-multibrot = 거듭제곱 d = 3, 4, 5에 대해 z → z^d + c. 거듭제곱이 높을수록 돌출부가 늘어납니다: 집합은 (d−1)겹 회전 대칭을 가집니다(Multibrot 3은 2겹, 4는 3겹, 5는 4겹).
help-fractals-tricorn = z → conj(z)² + c, 여기서 conj(x + iy) = x − iy. 켤레 연산 때문에 반정칙이 되어 3겹 대칭과 특유의 굽은 “발톱”이 생깁니다.
help-fractals-burning-ship = z → (|x| + i|y|)² + c — 제곱하기 전에 절댓값을 취합니다(실수부 = x²−y²+cx, 허수부 = 2|xy|+cy). 해석적이지 않으며, 깊이 확대하면 배 모양의 구조가 드러납니다(전통적으로 위아래를 뒤집어 봅니다).
help-fractals-celtic = Mandelbrot과 같지만 실수부에 절댓값을 취합니다: 실수부 = |x²−y²| + cx, 허수부 = 2xy + cy. 하트와 방패 모양의 무늬가 생깁니다.
help-fractals-buffalo = z²의 두 부분 모두에 절댓값: 실수부 = |x²−y²| + cx, 허수부 = |2xy| + cy — Celtic과 Burning Ship의 중간입니다.
# A heading: the families at powers above 2 (the Fractal menu's "Higher powers").
help-fractals-powers = 고차 거듭제곱
# ("Burning Ship  3 4 5") is how the Fractal menu shows such a row: keep it as it is.
help-fractals-powers-text = Multibrot 6, 7, 8(z^d + c), 그리고 거듭제곱 3, 4, 5의 Burning Ship, Tricorn, Celtic, Buffalo: 같은 접기를 더 높은 거듭제곱으로 한 것입니다. Burning Ship d는 z의 두 부분에 절댓값을 취한 뒤 거듭제곱하고, Tricorn d는 먼저 켤레를 취하며, Celtic d와 Buffalo d는 z^d의 실수부(Buffalo는 허수부도)를 접습니다. 프랙털 메뉴와 드롭다운은 한 행에 한 계열씩 나열합니다(“Burning Ship  3 4 5”). 각각 거듭제곱 2인 형제처럼 딥 줌하며, Multibrot 6, 7, 8도 Multibrot 3–5처럼 급수 근사와 미니브로트 찾기(M)를 지원합니다. Multibrot 8은 256 대신 반지름 128에서 탈출하는데, 그 다음 단계가 단정밀도 범위를 벗어나기 때문입니다.
help-fractals-phoenix = z → z² + c + p·z(n-1), 여기서 z(n-1)은 이전 반복값이고 p는 상수(p = -0.5)입니다. 이 기억 항이 불꽃 같은 가는 실을 만듭니다.
help-fractals-newton = z³ − 1 = 0의 근을 찾는 뉴턴 방법: z → z − (z³−1)/(3z²). 탈출 시간 대신, 반복이 1의 세 세제곱근 중 어느 것으로 수렴하는지(끌림 영역)와 얼마나 빨리 수렴하는지로 픽셀을 칠합니다. 얽힌 끌림 영역의 경계가 프랙털입니다.
help-fractals-julia = Julia 모드
help-fractals-julia-text = Newton을 제외한 모든 계열은 Julia 집합으로 전환할 수 있습니다: z를 0에서 시작하고 c = 픽셀로 두는 대신, c(매개변수)를 고정하고 z를 픽셀에서 시작합니다. Julia 집합은 그 고정된 c에 대해 유계로 머무는 시작점과 탈출하는 시작점 사이의 경계입니다. “둘 다, 연결”에서는 Mandelbrot 패널 위로 커서를 움직이면 c가 실시간으로 정해집니다.
help-fractals-deep = 딥 줌 지원
help-fractals-deep-text = Mandelbrot, Multibrot 3/4/5, Tricorn은 극한 깊이(floatexp) 섭동 딥 줌을 지원합니다. Burning Ship, Celtic, Buffalo는 해석적이지 않으므로(절댓값을 취하므로) 부호를 고려하는 섭동을 사용합니다. 이것도 이제 floatexp 범위에서 실행되어, 예전의 ~1e28× df32 한계를 훨씬 넘어 딥 줌합니다(절댓값 접힘 근처의 드문 반점은 다중 기준점 글리치 보정을 기다리고 있습니다). Phoenix는 두 항 섭동을 사용하며 역시 floatexp 범위에서 딥 줌합니다(SA/BLA 없이, 그래서 Mandelbrot보다 무겁습니다). 고차 거듭제곱도 같은 방식으로 딥 줌하며, 각 거듭제곱의 섭동은 자신만의 제곱 연쇄를 따릅니다. Multibrot 3–8은 Mandelbrot처럼 급수 근사와 BLA도 사용하고, 모든 거듭제곱의 Tricorn, Burning Ship, Celtic, Buffalo는 BLA(급수 없음)를 사용하는데, BLA는 깊은 뷰의 내부를 한 단계씩 반복하는 대신 건너뛰어 처리합니다. 직접 경로를 쓰는 것은 Newton뿐이며 ~1e6×까지 선명합니다. 사용자 정의 수식은 아래의 사용자 정의 수식 절에서 따로 말하지 않는 한 딥 줌합니다.
help-fractals-custom = 사용자 정의 수식
help-fractals-custom-text = 프랙털 ▸ 사용자 정의 수식…은 수식의 한 단계를 Fractint 방식의 식으로 받습니다: z = z^3 - p1*z + c, 또는 여러 문장(t = sqr(z), 그다음 z = t*t + c). z는 반복값, c는 픽셀(Julia 모드에서는 Julia 상수), p1-p5는 매개변수입니다. |z|는 Fractint에서처럼 절댓값의 제곱이며, 일반적인 함수를 쓸 수 있습니다(sqr, abs, conj, real, imag, cabs, flip, recip, ident, exp, log, sqrt, sin, cos, tan, cotan, sinh, cosh, tanh, cotanh, asin, acos, atan, asinh, acosh, atanh, cosxx, 그리고 각 부분을 반올림하는 floor, ceil, trunc, round). log는 Fractint와 복소해석학에서처럼 자연로그입니다: 많은 교과서가 ln으로 쓰는 것입니다(상용로그는 없습니다). ln, cot, coth, Re 또는 Im으로 쓴 수식은 해석되지 않습니다. 구문 검사가 써야 할 함수를 알려 주고, 버튼 하나가 고쳐 쓰며, ln을 입력하면 log를 제안합니다. 모든 단계는 섭동으로 딥 줌하며, 1e28x를 넘으면 내장 프랙털과 같은 확장 범위 산술을 사용합니다. 단, 지수가 z나 c에 따라 바뀌는 거듭제곱(예: z^c)이 있는 수식은 예외입니다: 그런 수식은 직접 경로에서 렌더링되며, c의 단정밀도 한 단계가 한 픽셀에 걸칠 때까지, 즉 약 1e4x에서 1e5x까지 선명하고, 원점에서 멀어질수록 덜 깊습니다. 수식 대화 상자가 어느 쪽인지 알려 줍니다. log, sqrt, 정수가 아닌 거듭제곱은 주값을 취하며, 주값은 음의 실수축을 가로지르며 불연속적으로 뜁니다. 섭동은 이 도약을 정확히 따르지만, 기준 궤도가 건너지 않는 곳에서 궤도가 건너는 픽셀은 그때부터 약 단정밀도로 계산되므로, 궤도가 분지 절단에 걸치는 깊은 뷰에는 그곳에 노이즈가 보입니다(측정: sqrt(z^4 + c)는 실수축에서 1e5x에서도 깨끗하고, log 수식은 1e4x에서도 노이즈가 있음). 모든 단계를 빠짐없이 반복하므로(급수 근사 없음), 반복 횟수가 높은 깊은 뷰는 내장 프랙털보다 느리게 렌더링됩니다. 경계 근처에서 함수가 들어간 수식의 궤도는 혼돈적이며, 그곳의 긴 궤도는 단정밀도로 느슨하게만 따라갈 수 있습니다: 2,000회 반복에서는 sin(z) + c 픽셀의 3분의 1이 정확한 궤도의 결과와 다르고, 60회에서는 거의 없습니다. 뷰, 세션, 내보낸 이미지는 수식의 텍스트를 보관합니다.
help-fractals-custom-sections = Fractint의 섹션도 쓸 수 있습니다. ':' 앞의 문장은 초기화 섹션으로, 픽셀마다 한 번 실행됩니다. 마지막 문장이 비교식이면 수식 자체의 탈출 조건 검사가 되어, 그 조건이 참인 동안 반복합니다(|z| <= 100 && |z - 1| > 0.000001은 탈출하거나 1에 정착하는 궤도를 멈춥니다). if (...), elseif (...), else, endif로 문장을 고르며, 한 단계에서 값을 정하기 전에 읽은 변수는 이전 단계의 값을 가집니다(초기화 섹션의 값 또는 0). 비교식은 실수부를 비교해 1 또는 0을 내고, &&와 ||로 조합하며, 대입도 값이 됩니다(a = b = pixel). pixel, p1-p5, pi, e에는 대입할 수 있으며, 대입하기 전까지는 원래 값을 유지합니다. maxit는 반복 횟수이고 ismand는 1입니다(수식의 Mandelbrot 형태). 이런 것들이나 반올림이 들어간 수식은 딥 줌 없이 직접 경로에서 렌더링됩니다. 수식 라이브러리 > 가져오기는 Fractint .frm 파일도 읽습니다. 읽히는 항목은 각각 수식이 되고, 그 행에는 읽으면서 바뀐 내용이 표시됩니다: fn1-fn4는 Fractint의 기본값(sin, sqr, sinh, cosh)을 취합니다. 여기서는 c가 픽셀이므로 Fractint의 변수 c는 c_로 이름이 바뀝니다. Fractint에서처럼 아무것도 값을 정하지 않는 이름은 0입니다. Fractint처럼 이름 안의 공백은 제거됩니다(end if는 endif). 가져오기 메시지는 읽히지 않는 항목 수를 세고 가장 흔한 이유를 알려 줍니다: 화면과 뷰 변수(whitesq, scrnpix, scrnmax, center, magxmag, rotskew), lastsqr, 난수는 지원하지 않습니다. 컬렉션의 Fractint 고전 수식(Magnet, Nova, Spider, Manowar 등)은 이런 섹션으로 작성되어 있습니다.
help-fractals-custom-editor = 수식 입력란에서 괄호는 중첩 수준마다 색이 다르며, 커서 옆의 괄호 쌍은 강조되고, 짝이 없는 괄호는 빨간색으로 표시됩니다(주석 안의 괄호는 파서가 무시하므로 세지 않습니다). 이름은 입력하는 대로 완성됩니다: 두 글자를 입력하면 그렇게 시작하는 함수, 이름, 사용자 변수 목록이 나타납니다. Up과 Down으로 고르고, Tab이나 Enter로 선택하며(함수는 여는 괄호와 함께 입력됨), Esc는 목록을 닫고, 이름을 끝까지 입력한 뒤 누르는 Enter는 여전히 새 줄을 시작합니다.
help-fractals-custom-textbook = 교과서(수식 위의 전환 스위치)는 수식을 교과서처럼 조판해 보여 주고, 그 자리에서 편집합니다. 텍스트에서처럼 입력하십시오: /는 앞의 항을 분수로 만들고, ^는 지수를 만들며(지수 끝에서 +, - 또는 =를 입력하면 지수에서 빠져나옴), (는 괄호, 또는 함수 이름 뒤에서는 호출, |는 절댓값 막대, 괄호 안의 쉼표는 복소수 상수를 만듭니다. 화살표 키로 수식 안을 이동하고, Up과 Down으로 분자와 분모 사이를, Tab으로 다음 빈 칸으로 이동합니다. 분수나 괄호 쌍의 시작에서 Backspace를 누르면 그것을 선택하고, 한 번 더 누르면 그것을 지우고 안의 내용은 남깁니다. 복사, 붙여 넣기, 실행 취소는 텍스트 입력란에서처럼 동작하고, 이름도 텍스트에서처럼 완성되며, 키패드는 조판된 수식에 입력합니다(□²는 캐럿 앞의 것을 제곱하고, 함수 키는 선택 영역을 감쌉니다). 모든 편집은 텍스트에 다시 기록되므로 두 보기는 항상 같은 수식을 담습니다. 편집하지 않은 줄은 간격과 주석을 그대로 유지합니다. 표기법은 계산 내용을 결코 바꾸지 않습니다: |z|는 절댓값의 제곱이며 |z|²로 표시되고, cabs(z)는 |z|로 표시되며, exp(z)는 exp(z) 그대로입니다. 해석되지 않는 줄은 빨간색으로 표시되며, 클릭하면 텍스트로 편집할 수 있습니다. LaTeX는 수식을 조판된 모양 그대로 LaTeX 소스로 복사하며, 교과서가 켜져 있는 동안에는 수식 라이브러리도 수식을 조판해 보여 줍니다.
help-fractals-custom-numbers = 수식 안의 숫자(0.1, 1/3, pi, 매개변수)는 약 16자리의 배정밀도 값으로 읽히며, 깊은 기준 궤도는 정확히 그 값으로 반복합니다. 그래서 종이 위에서 같은 두 수식도 깊은 곳에서는 다를 수 있습니다: 0.1*z는 0.1에 가장 가까운 double을 곱하지만 z/10은 정확한 10으로 나누므로, 두 그림은 약 1e15x를 넘으면 갈라집니다(측정: 1e15x에서 픽셀의 5%, 1e17x에서 3분의 1, 1e20x에서 전부). 어느 그림이든 그 자신의 수식으로서는 올바릅니다. 공유하려는 쪽을 쓰십시오.
help-fractals-custom-library = 대화 상자의 라이브러리에 저장은 수식과 그 매개변수를 이름을 붙여 보관합니다(라이브러리에 이미 있는 이름으로 저장하면 그 항목을 업데이트합니다). 프랙털 ▸ 수식 라이브러리…는 이를 나열해 적용, 편집, 내보내기, 삭제할 수 있게 합니다. 가져오기는 수식 파일의 수식을 아무것도 바꾸지 않고 추가합니다: 라이브러리에 이미 있는 수식은 건너뛰고, 이름이 이미 쓰인 새 수식은 “이름 (2)”가 됩니다. 라이브러리는 파일 및 데이터(정보) 아래에 표시된 폴더의 formulas.toml이며, 그 자체가 수식 파일입니다. 투어 키프레임은 수식을 formula = "..."와 formula_params = [[re, im], ...]로 담으며, 도구 > 현재 뷰로 투어 만들기가 사용자 정의 뷰에 대해 이를 기록합니다.
help-fractals-custom-collection = Fractadyne에는 이 프로그램을 위해 작성한 수식 45개의 컬렉션이 함께 제공됩니다: 고차 거듭제곱과 Burning Ship 및 Celtic 계열의 접힌 변형, 혼합과 두 단계 혼성, 다른 사상을 거쳐 본 Mandelbrot 집합(반전, 로그 극좌표, 사인을 통해), 초월 수식과 유리 수식, 그리고 Julia 집합. 이것들은 대화 상자의 예제이자 수식 라이브러리의 컬렉션 탭이며, 그곳의 내 수식으로 복사는 수식 하나를 사용자의 수식에 추가합니다. 각 수식에는 시작 뷰가 있으며, 적용하면 그곳으로 갑니다: 홈 뷰에서 연 수식은 그림이 있는 곳이 아닌 경우가 많습니다. 수식이 표시되는 동안 저장한 수식도 같은 방식으로 지금의 뷰를 보관하며, 적용하면 그 뷰로 돌아갑니다.

## Help ▸ How it works

help-how-escape = 탈출 시간과 부드러운 색
help-how-escape-text = 각 픽셀은 크기가 탈출 반경을 넘거나 반복 상한에 이를 때까지 수식을 반복합니다. 날것의 단계 수만으로는 뚜렷한 띠가 생기며, 마지막 크기에서 얻은 소수 부분을 더하면 띠 없는 연속적인 색이 됩니다.
help-how-precision = 임의 정밀도 위치
help-how-precision-text = 일반적인 64비트 숫자는 약 1e15× 줌에서 자릿수가 바닥납니다. Fractadyne은 뷰 중심을 임의 정밀도로 보관하며, 확대할수록 자릿수가 늘어나므로 위치가 결코 흐트러지지 않습니다. 픽셀 크기도 확장된 지수로 보관하므로 64비트의 ~1e308× 한계에서 멈추지도 않습니다 — 깊이를 제한하는 것은 고정된 숫자 범위가 아니라 인내심뿐입니다.
help-how-perturbation = 섭동
help-how-perturbation-text = 모든 픽셀을 높은 정밀도로 반복하면 너무 느립니다. 대신 기준 픽셀 하나를 CPU에서 높은 정밀도로 반복하고(“기준 궤도”), 나머지 모든 픽셀은 GPU에서 그 기준과의 아주 작은 차이 δ로서 빠른 낮은 정밀도로 계산합니다: δz → 2·Z·δz + δz² + δc.
help-how-floatexp = 극한 깊이(floatexp)
help-how-floatexp-text = 약 1e28×를 넘으면 그 작은 차이조차 32비트 범위 아래로 언더플로하므로, 가수와 별도의 정수 지수로 저장합니다(“floatexp”). 이로써 32비트 깊이의 벽이 사라집니다. 엔진은 자동으로 전환합니다: 얕을 때는 직접 계산, 깊을 때는 섭동, 가장 깊을 때는 floatexp. 이때 깊이를 제한하는 것은 고정된 한계가 아니라 좌표 정밀도와 반복 예산입니다 — 39개 위치로 된 기준 자료에서 Fraktaler-3과 교차 검증했으며, 가장 깊은 곳은 5.63e18003×입니다(극단적이지 않은 38개 행이 일상적인 관문으로, 6.13e1105×까지). 1e1000000×까지의 내부 정밀도 자기 일관성 검사 묶음도 있으며, 함께 제공되는 투어는 ~1.3e420×에 이릅니다.
help-how-reference = 기준점 선택과 리베이스
help-how-reference-text = 기준점은 그 궤도가 가능한 한 오래 뷰 안에 머물도록 (높은 정밀도로 점수를 매겨) 고릅니다. 차이가 너무 커지면 정확성을 유지하기 위해 다시 기준 궤도로 “리베이스”합니다.
help-how-series = 급수 근사
help-how-series-text = 딥 줌에서는 초기 반복이 δ를 기준점에서 거의 움직이지 않으므로, 한꺼번에 계산할 수 있습니다: δz를 δc에 대한 짧은 다항식(3차)으로 근사하며, 그 계수는 기준 궤도를 따라 반복합니다. 렌더러는 그 다항식이 뷰 전체에서 여전히 정확한 마지막 반복까지 건너뛴 다음 평소대로 반복합니다 — 건너뛴 단계만큼 절약하면서 이미지는 바뀌지 않습니다(컨트롤 ▸ 고급에서 켜고 끔). Mandelbrot과 Multibrot 3–8, 가장 깊은 범위에서 동작하며, 스트라이프 / 삼각 부등식 / 궤도 트랩 / 분해 채색과는 함께 쓰지 않습니다(이 방식들은 모든 반복이 필요합니다). BLA가 동작하는 곳에서는 BLA가 이미 이 초기 건너뛰기를 포함합니다.
help-how-distance = 거리 추정과 조명
help-how-distance-text = 도함수 dz/dc를 추적하면 각 픽셀에서 경계까지의 거리를 얻습니다. 이것이 3D 입체 조명, 거리 글로우, “거리 추정” 채색 방식을 가능하게 하며 — 모두 어떤 줌 깊이에서도 유효합니다.
help-how-safety = 안티앨리어싱과 안전장치
help-how-safety-text = 정지 이미지는 뷰가 안정되면 슈퍼샘플링합니다(축마다 2–8×). 작업 예산은 극단적인 설정에서 해상도를 줄여(반복 횟수는 결코 줄이지 않음) 한 번의 GPU 그리기를 드라이버의 워치독 한도 안에 유지하므로, 깊은 뷰가 빈 화면이 되는 대신 디테일을 유지합니다.

## Help ▸ Command line (the flags themselves stay English)

# The section's introduction. The flags listed below it are the command line itself, English in every language.
help-cli-intro = Fractadyne은 렌더링, 동영상/투어 내보내기, 자동화, 골든 이미지 검사, 벤치마크를 위해 화면 없이 실행할 수 있습니다. 플래그 없이 실행하면 대화형 GUI가 시작됩니다.

## Help ▸ Shortcuts

# What each mouse gesture and key does (the gestures are the Navigation section's).
help-shortcuts-pan = 이동
# A gesture: turning the mouse wheel.
help-key-wheel = 휠
help-shortcuts-wheel = 커서 위치에서 줌
help-shortcuts-box-zoom = 영역 줌(사각형을 드래그해 그곳으로 줌)
# $icon: the click-to-zoom toolbar button's symbol. "tool on": while that button is on.
help-key-click-tool = 클릭({ $icon } 도구 켬)
help-shortcuts-click = 그 점으로 다이브. 오른쪽 클릭은 되돌아 나옴
# A heading: the keyboard shortcuts.
help-shortcuts-keyboard = 키보드
help-shortcuts-space = 연속 확대 / 축소
help-shortcuts-undo = 뷰 실행 취소
help-shortcuts-redo = 뷰 다시 실행
help-shortcuts-minibrot = 미니브로트 찾기 + 확대(뉴턴-랩슨)
help-shortcuts-autopilot = 자동 줌 자동 조종(디테일을 향해 다이브, 아무 입력이나 하면 멈춤)
# off / strip / panel / full: the panel's sizes, as View ▸ Live render panel names them.
help-shortcuts-live-panel = 라이브 렌더 패널: 끄기 / 스트립 / 패널 / 전체(계기 위에 포인터를 올리면 그 의미를 보여 주며, ×로 닫음)
help-shortcuts-esc = 자동 조종 / 재생 중인 투어 중지, 또는 전체 화면 끝내기
help-shortcuts-snapshot = 마지막 내보내기 폴더에 스냅샷 — 보이는 그대로의 뷰 화면 캡처, 또는 내보내기 설정의 전체 렌더링. 처음 누를 때 어느 쪽인지 묻고, 파일 → 설정 → 스냅샷에서 바꿀 수 있습니다.
help-shortcuts-help = 이 도움말 열기
# A heading: the shortcuts of the Game of Life, with the menu path to it.
help-shortcuts-life = Life(프랙털 → 오토마타 → Life)
help-shortcuts-life-run = 세계 실행 / 일시 정지
help-shortcuts-life-step = 한 세대 진행 / 보폭만큼 진행(Life 패널에서 설정)
help-shortcuts-life-draw = 그리기 도구 켜기 / 끄기(도구 모음의 연필)
# A gesture. "Draw on": while the Life draw tool is on.
help-key-click-drag-draw = 클릭 / 드래그(그리기 켬)
help-shortcuts-life-paint = 셀 뒤집기 / 칠하기: 죽은 셀에서 시작한 획은 그리고, 산 셀에서 시작한 획은 지웁니다. 실행 중인 세계는 그리는 동안 멈추고, 손을 떼면 다시 실행됩니다.
# $key: a keyboard shortcut (Ctrl+Z). "Draw on": while the Life draw tool is on.
help-key-when-drawing = { $key }(그리기 켬)
help-shortcuts-life-undo = 마지막 편집, 채우기, 지우기 또는 규칙 변경 실행 취소

## Help ▸ Recommended hardware

help-hardware-intro = Fractadyne은 작업을 GPU(픽셀별 반복 + 채색)와 CPU(딥 줌의 바탕이 되는 임의 정밀도 기준 궤도)에 나눕니다. 둘 다 중요하지만, 중요한 부분이 다릅니다.
help-hardware-gpu = GPU(가장 중요)
help-hardware-gpu-text = 모든 픽셀을 GPU에서 반복하고 채색하므로, GPU 속도가 프레임 속도와 내보내기 시간을 정합니다. 32비트(단정밀도) 처리량이 높은 외장 GPU가 가장 부드러운 딥 줌을 제공합니다. 내장 GPU도 잘 동작합니다 — 다만 더 느리고, 프레임당 작업 예산에 더 빨리 도달합니다(그때 렌더러는 디테일이 아니라 해상도를 줄입니다). 최신 그래픽 API가 필요합니다: wgpu를 통한 Vulkan, Direct3D 12, Metal 또는 OpenGL.
# The left column of a requirements table.
help-hardware-minimum = 최소
help-hardware-gpu-minimum = Vulkan / DX12 / Metal / GL 3.3을 지원하는 모든 GPU.
help-hardware-recommended = 권장
help-hardware-gpu-recommended = 외장 GPU(최근 약 5년 이내), VRAM >= 2 GB.
help-hardware-big-exports = 큰 내보내기용
help-hardware-gpu-big-exports = VRAM >= 4 GB(타일 방식이라 부족해도 성능이 완만하게 낮아집니다).
# A heading: the processor.
help-hardware-cpu = CPU
help-hardware-cpu-text = 고정밀 기준 궤도(와 BLA 표)는 CPU에서 계산되며 대부분 단일 스레드이므로, 코어당 성능이 가장 중요합니다 — 이동한 뒤 깊은 뷰가 얼마나 빨리 그려지는지를 좌우합니다. 추가 코어는 주로 백그라운드 내보내기 인코딩에 도움이 됩니다. 64비트 CPU가 필요합니다.
help-hardware-cpu-recommended = 단일 코어 속도가 좋은 최신 64비트 CPU.
help-hardware-memory = 메모리
help-hardware-memory-text = 깊은 기준 궤도는 깊이에 따라 커지며(높은 정밀도의 많은 반복), 거대한 내보내기는 타일을 RAM에 보관합니다. 일상적인 탐색에는 8 GB면 넉넉하고, 극한 깊이와 아주 큰 내보내기에는 16 GB 이상이 알맞습니다.
help-hardware-checking = 하드웨어 점검
help-hardware-diagnostics = 도움말 → “진단…”은 개발자가 하는 것과 같은 테스트를 사용자의 컴퓨터에서 실행합니다. “자체 테스트 실행”은 약 15초 동안 수학과 렌더링을 알려진 정답과 비교해 확인합니다. “UI 테스트 실행”은 인터페이스와 라이브 뷰를 여러 줌 깊이에서 둘러보며 스크린샷을 찍습니다. “GPU 산술 검사 실행”은 컴퓨터가 제공하는 모든 그래픽 백엔드에서 셰이더의 double-float 산술을 몇 초 만에, 창 없이 검증합니다. 각 테스트는 별도 프로세스로 실행되므로, 그래픽 드라이버를 흔드는 테스트가 있어도 세션이 아니라 그 테스트만 종료됩니다.
help-hardware-gpu-check = GPU 산술 검사에는 통과나 실패가 없습니다 — 사용자의 GPU와 드라이버가 어떻게 계산하는지 기록하며, 일부 하드웨어(지금까지 시험한 모든 NVIDIA 카드)에서는 셰이더 컴파일러가 확장 정밀도 산술의 일부를 버린다고 보고합니다. 이것은 여러 제조사에 걸쳐 파악하려는 실제 발견이지 사용자 컴퓨터의 결함이 아니므로, 결과가 무엇이든 보고서에 첨부하면 정말로 도움이 됩니다.
help-hardware-report = 무언가 이상해 보이면 자체 테스트를 실행하고 “문제 보고에 첨부…”를 사용하십시오: 그러면 보고서에 사용자 하드웨어에서 기계가 검증한 결과가 담기며, 이는 설명만 있는 것보다 훨씬 유용합니다. 알아 둘 만한 주의점이 하나 있습니다 — 기준 이미지는 특정 그래픽 카드 한 장에서 만들어졌고, GPU가 다르면 픽셀을 정당하게 약간 다르게 계산하므로, 다른 하드웨어에서는 이미지 비교를 더 넓은 허용 오차로 확인하며 그곳의 작은 차이는 결함이 아니라 예상된 것입니다.
help-hardware-notes = 참고
help-hardware-note-watchdog = 성능 좋은 GPU라면 극단적인 설정에서 워치독 작업 예산이 개입하는 일을 피할 수 있습니다.
help-hardware-note-binary = 빌드는 단일 실행 파일이며 자체 완결형입니다 — 런타임을 설치할 필요가 없습니다.
help-hardware-note-benchmark = 내장 벤치마크(도구 ▸ 벤치마크)를 실행해 컴퓨터 성능을 가늠해 보십시오.

## Help ▸ Acknowledgments & citations
## Names of people, programs, papers, books and websites stay as written, and so do the
## titles in quotation marks (the works' own titles); the prose around them translates.

help-ack-title = 감사의 말과 인용
# The dedication, in italics at the top of the section.
help-ack-dedication = Stone Soup Group — Fractint를 만든 자원봉사자들(1988년부터) — 에게 바칩니다.
help-ack-dedication-text = 그들이 아낌없이 공유한 코드와, 열린 협업으로 프랙털을 탐구하는 정신 덕분에 수많은 사람이 이 예술과 사랑에 빠졌습니다. Fractadyne은 그 전통 위에 만들어졌습니다.
help-ack-intro = Fractadyne은 프랙털 커뮤니티가 수십 년간 쌓아 온 작업 위에 서 있습니다. 바탕이 된 기법과 선행 연구 — 아래 출처와 대조해 확인했습니다:
help-ack-deep-zoom = 딥 줌 알고리즘
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-martin-title = 섭동과 급수 근사 — { $who }
help-ack-martin = 기준 궤도 + 저정밀도 델타 방법(δz → 2Z·δz + δz² + δc)과 초기 반복을 건너뛰는 n차 급수로, SuperFractalThing과 그 노트 “SuperFractalThing Maths”(sft_maths.pdf, 2013)에서 소개되었습니다.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bla-title = 이변수 선형 근사(BLA)와 리베이스 — { $who }
help-ack-bla = 병합된 선형 사상으로 연속된 반복을 건너뛰고, 글리치를 피하기 위해 단일 기준 궤도로 리베이스합니다. fractalforums.org에서 개발되었습니다(2021).
# A link's label. $who: the linked page's author.
help-ack-bla-link = BLA 설명 — { $who }
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-glitch-title = 글리치 감지 기준 — { $who }
help-ack-glitch = 픽셀에 다른 기준 궤도가 필요한 때를 판정하는 |Z + z| << |Z| 검사 — 글리치 없는 섭동의 기초입니다(fractalforums).
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-diffabs-title = 비해석적(Burning Ship) 섭동 — { $who }
help-ack-diffabs = 절댓값 기반 계열(Burning Ship / Celtic / Buffalo)에서 섭동이 동작하게 하는 절댓값 “diffabs” 경우 분석으로, fractalforums에서 발표되었습니다.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mathr-title = 참조 구현과 교차 검증 — { $who }
help-ack-mathr = Fraktaler-3과 Kalles Fraktaler 2+(원저자 Karl Runmo) — 그 해설이 이 엔진의 길잡이가 되었고, Fractadyne은 그 EXR 출력과 자신의 렌더러를 교차 검증합니다.
# A heading: credits for the coloring methods.
help-ack-coloring = 채색
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-harkonen-title = 부드러운 반복 횟수와 스트라이프 평균 — { $who }
help-ack-harkonen = 연속(정규화된) 탈출 시간 채색과 스트라이프 평균 방법으로, “On Smooth Fractal Coloring Techniques”(석사 논문, 2007)에서 나왔으며, 이 논문은 삼각 부등식 / 분기 평균 채색도 분석합니다.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mitchell-title = 삼각 부등식 평균 — { $who }
help-ack-mitchell = 각 z² + c가 삼각 부등식 경계 안에서 어디에 놓이는지를 평균해, 부드럽고 불꽃 같은 외부 질감을 만듭니다.
# A link's label: the person's tutorial pages. $who: the person's name.
help-ack-mitchell-link = { $who } — 튜토리얼
help-ack-distance = 거리 추정 채색/조명은 고전적인 dz/dc 경계 거리 방법을 사용합니다(Peitgen & Saupe, “The Science of Fractal Images”, 1988 참조).
help-ack-lsystems-life = L-시스템과 생명 게임
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-lindenmayer-title = L-시스템 — { $who }
help-ack-lindenmayer = 식물과 조류가 자라는 방식을 모형화하기 위해 1968년에 도입된 병렬 문자열 재작성(“Mathematical models for cellular interactions in development”, Journal of Theoretical Biology).
help-ack-abop = L-시스템 판독기가 따르는, 확률적·문맥 의존적·매개변수적 시스템의 거북이 해석과 표기법을 담은 책(1990)입니다. ABOP 표시가 있는 내장 시스템은 이 책의 그림에서 옮겨 입력한 것입니다.
# A link's label: the book can be read free on that website. $site: the website's name.
help-ack-abop-link = 이 책, { $site }에서 무료로
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bourke-title = L-시스템 컬렉션 — { $who }
help-ack-bourke = 내장 시스템 다수가 그의 L-시스템 페이지에서 왔으며, 그중에는 Chris Wallace의 Sierpinski 곡선 규칙과 Dietmar Saupe의 덤불이 있습니다. 바로 열 수 있는 Fractint의 .l 파일은 Stone Soup Group의 형식입니다.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-conway-title = 생명 게임 — { $who }
help-ack-conway = 세포 자동자 B3/S23(1970)으로, Scientific American의 Martin Gardner 칼럼 “Mathematical Games”(1970년 10월)를 통해 세상에 알려졌습니다.
# A credit's heading: the website, then who runs it. $site: the website's name.
help-ack-lifewiki-title = { $site } — ConwayLife.com 커뮤니티
help-ack-lifewiki = 앱이 읽는 규칙 표기법(B/S, von Neumann, Generations, 비총합 규칙을 위한 Alan Hensel의 문자, MAP), Golly와 공유하는 RLE 패턴 형식, 그리고 내장 규칙과 패턴의 참고 자료이며, 그 발견자들도 기록하고 있습니다.
help-ack-foundations = 기반과 도구
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mandelbrot-title = Mandelbrot 집합 — { $who }
help-ack-mandelbrot = 이 모든 것의 중심에 있는 대상.
help-ack-built-with = Rust, egui/eframe(emilk), wgpu(gfx-rs)로 만들었습니다. 임의 정밀도는 astro-float, 이중 워드 부동소수점은 twofloat, 그리고 PNG/OpenEXR, rfd, serde를 사용합니다. 그 저자와 관리자 여러분께 감사드립니다.
help-ack-errors = 출처 표기의 오류는 저자의 책임이며, 수정 사항은 프로젝트의 이슈 트래커에서 환영합니다.

## Help ▸ Third-party licenses

help-licenses-title = 서드 파티 라이선스
# $licenses: Fractadyne's license expression (MIT OR Apache-2.0), as written. $url: the web address of a crate's source. THIRD-PARTY-NOTICES.md is a file name, `option-ext` a crate's name, MPL-2.0 a license's.
help-licenses-intro = Fractadyne은 아래의 오픈 소스 크레이트와 번들 글꼴을 포함하는 정적 링크 실행 파일로 배포됩니다. 전체 라이선스 텍스트(다운로드에 포함된 THIRD-PARTY-NOTICES.md에도 있음)가 이어집니다. Fractadyne 자체 코드는 { $licenses } 이중 라이선스입니다. `option-ext`는 MPL-2.0이며, 수정하지 않은 소스는 { $url }에 있습니다.
# A button, after a copy icon: copies every notice to the clipboard.
help-licenses-copy-all = 모든 고지 복사

## Help ▸ About

help-about-tagline = Rust와 wgpu로 만든 네이티브 프랙털 탐색기.
# The arithmetic deep zoom has used. $ran: the libraries that have run so far, as names (or a note that none has yet); $built: those this build contains, with versions.
help-about-arithmetic = 딥 줌 산술: { $ran }(이 빌드에 포함: { $built }).
help-about-files = 파일 및 데이터
# Followed by the folder's path.
help-about-files-text = 세션(현재 뷰, 채색, 환경 설정), 북마크와 그 썸네일, 저장된 그라데이션과 저장된 수식은 이 사용자별 폴더에 저장되며, 다음 실행 때 다시 불러옵니다:
# Tooltip of the copy button beside the folder's path.
help-about-copy-path = 경로 복사
# $command: a command typed in a terminal (it stays English).
help-about-session = 세션 파일에는 버전이 있습니다. 이 앱이 완전히 읽을 수 있는 것보다 새로운 Fractadyne이 기록한 파일이면, 앱은 가능한 만큼 불러온 뒤 경고합니다. 완전히 새로 시작하려면 파일 ▸ 애플리케이션 상태 초기화…(먼저 확인함)를 사용하거나, 터미널에서 `{ $command }`를 실행하십시오.
help-about-license = 라이선스
# $licenses: the license expression (MIT OR Apache-2.0), which stays as written.
help-about-license-text = { $licenses } — 둘 중 원하는 라이선스로 사용할 수 있습니다.
# A link to the source code, followed by ↗.
help-about-source = GitHub의 소스

## The live render panel ("blinkenlights": View ▸ Live render panel, key B)
## A front panel drawn over the view like a piece of 1970s test equipment. Its legends are engraved
## in capitals: write them in capitals here (the panel shows them exactly as written, it does not
## uppercase them). Numbers, units and readouts on its tubes and dials are not translated.

# Tooltip of the × button in the panel's top-right corner. The code adds the key after it: (B).
live-close-hover = 라이브 렌더 패널 닫기
# The legend engraved along the top plate, left of the maker's badge.
live-plate = 라이브 렌더 · 전면 패널

## Live render panel: instrument titles
## Engraved above each instrument, and repeated as the title of its help card.

# Above the depth readout, whose tubes show the magnification as a power of two ("2^" stays).
live-title-depth = 깊이 2^
live-title-iterations = 반복
# Above the frame-time readout (milliseconds).
live-title-frame-ms = 프레임 MS
# Above the date-and-time readout.
live-title-clock = 날짜 · 시각
# The walk dial: a radar screen showing a slow refresh rendered pass after pass, band after band.
live-title-walk = 순회 · 구간
# A meter of how far the picture on screen lags the view.
live-title-age = 그림 지연
# What part of the picture this frame's GPU pass covered.
live-title-tiles = 타일
# A scope of the GPU's work per frame, in milliseconds.
live-title-scope = GPU 패스 · MS
# The depth dial: depth, the reference orbits being built, and the arithmetic in use.
live-title-dial = 깊이 · 기준 궤도 · 산술
# Iteration steps per second.
live-title-steps = 스텝 / S
# The two GPUs (the window's and a second, worker GPU).
live-title-engines = 두 엔진
live-title-memory = 메모리
live-title-cores = 코어 · 스레드
live-title-pixels = 픽셀 스트림
# Help card titles only: the three GPU timer lamps (T0, T1, T2 stay as they are) and the row of lamps.
live-title-timers = GPU 타이머 T0–T2
live-title-bank = 램프 뱅크

## Live render panel: the light bank's lamps
## Engraved under each lamp, and the title of its help card. A cell is narrow (about eight capital
## letters): a longer label breaks at a space onto two lines, and is drawn smaller if it still does
## not fit.

live-lamp-plan = 계획
live-lamp-dispatch = 디스패치
live-lamp-live = 라이브
live-lamp-held = 유지
# The last picture reprojected (stretched) to fit the moving view.
live-lamp-reproj = 재투영
live-lamp-reading = 판독
# A pass of a pinned walk.
live-lamp-pin-pass = 고정 패스
live-lamp-split = 분할
live-lamp-adopt = 채택
live-lamp-abandon = 폐기
# "REF": a reference orbit. Built for the view as it is (LIVE), ahead of it (AHEAD), or for a tour's next hold (HOLD).
live-lamp-ref-live = 기준 라이브
live-lamp-ref-ahead = 기준 선행
live-lamp-ref-hold = 기준 정지
# The series approximation.
live-lamp-series = 급수
# The second (worker) GPU.
live-lamp-worker = 워커
live-lamp-tile = 타일
# Supersampling (anti-aliasing); usually kept as SSAA.
live-lamp-ssaa = SSAA
live-lamp-palette = 팔레트
# The arithmetic in use (direct, df32, floatexp).
live-lamp-mode = 모드

## Live render panel: the kinds of GPU pass
## In the GPU scope's legend (MOVING, PIN, WORKER, SETTLE, SSAA). SETTLE and SSAA also name a still
## view's walk on the walk dial.

# A moving frame's pass.
live-kind-moving = 이동
# A pass of a pinned walk.
live-kind-pin = 고정
# The worker GPU's passes.
live-kind-worker = 워커
# A still view's settle tile or walk pass.
live-kind-settle = 안정화
# A supersampling sample.
live-kind-ssaa = SSAA

## Live render panel: legends and readouts on the instruments

# On the walk dial during a split refresh. $done: passes done; $passes: passes in all.
live-walk-split = 분할 { $done }/{ $passes }
# On the picture-age meter's face, under its scale (0 to 2 octaves).
live-age-scale = 뒤처진 옥타브
# The depth dial's band for direct (plain floating-point) arithmetic. The other bands, DF32 and FLOATEXP, are names and stay.
live-dial-direct = 직접
# Under the two GPU gauges: GPU 0 draws the window; GPU 1 is the worker, or there is none.
live-engines-gpu-window = GPU 0 · 창
live-engines-gpu-worker = GPU 1 · 워커
live-engines-no-worker = 워커 없음
# The share of displayed frames each GPU made. $window and $worker: percentages, 0–100.
live-engines-on-screen = 화면 표시 { $window } : { $worker }
# The same readout before any frame was shown.
live-engines-on-screen-none = 화면 표시 —
# Under the CPU core bars. $count: the machine's cores.
live-cores-count = 코어 { $count }개
# Under the threads scope. $count: the app's busy threads.
live-cores-threads = 스레드 { $count }

## Live render panel: the memory tanks' legend
## What the GPU memory holds, by kind. Very narrow: about ten letters.

# G-buffers (the picture's working textures).
live-memory-gbuf = G-BUF
# The held picture and the supersampling accumulator.
live-memory-hold = 유지+SSAA
# The reference orbit and its BLA (bilinear approximation) table.
live-memory-orbit = 궤도+BLA
# The worker GPU's copy of the reference.
live-memory-worker = 워커 기준

## Live render panel: the roles on the threads scope
## Tiny labels on a small scope: at most four letters each.

# The reference builders.
live-cores-role-ref = 기준
# The scoring pool.
live-cores-role-pool = 풀
# The thread driving the worker GPU.
live-cores-role-worker = 워커
# Probes: short side renders.
live-cores-role-probe = 탐침
# Image and tour exports.
live-cores-role-export = 출력

## Live render panel: the tiles instrument's captions
## The line under the picture's plane, then the tile size (left) and the tiles a frame (right).

# A walk's pass. $lo and $hi: the iteration band it rendered (numbers).
live-tiles-pin-band = 고정 패스 · 구간 { $lo }–{ $hi }
live-tiles-settle-band = 안정화 패스 · 구간 { $lo }–{ $hi }
live-tiles-moving-whole = 이동 프레임 · 전체 그림
live-tiles-settled-whole = 안정된 프레임 · 전체 그림
# $pass: this pass; $passes: the passes the refresh is split into.
live-tiles-split-pass = 분할 갱신 · 패스 { $pass }/{ $passes }
# $grid: the grid's columns × rows, e.g. 12×8.
live-tiles-grid-done = 안정화 격자 { $grid } · 완료
# $grid: the grid's columns × rows; $done and $total: tiles done, and in all.
live-tiles-grid-progress = 안정화 격자 { $grid } · { $done }/{ $total } 완료
# The same, while a tile too costly for one frame goes out in strips.
live-tiles-grid-strips = 안정화 격자 { $grid } · { $done }/{ $total } 완료 · 스트립 단위
# $samples: supersampling samples folded so far; $lo and $hi: the iteration band the next one is walking.
live-tiles-ssaa-band = SSAA · { $samples } 완료 · 구간 { $lo }–{ $hi }
# $sample: the supersampling sample.
live-tiles-ssaa-whole = SSAA 샘플 { $sample } · 전체 프레임
# No GPU pass this frame, and the panel has not seen a frame yet.
live-tiles-idle-starting = 시작 중
# No GPU pass this frame: the last complete picture is held on screen.
live-tiles-idle-held = 유지
# No GPU pass this frame: the last picture is stretched to fit the view.
live-tiles-idle-reprojected = 재투영됨
# No GPU pass this frame: the picture on screen is current.
live-tiles-idle-live = 라이브
# No GPU pass this frame, and nothing to show.
live-tiles-idle-none = 유휴
# The tile size: $res, a whole picture's resolution (2400×1300).
live-tiles-tile-res = 타일 { $res }
# The tile size: $px, a tile's side in pixels.
live-tiles-tile-px = 타일 { $px } PX
live-tiles-no-pass = 패스 없음
# Tiles a frame. $count: how many (0 when none).
live-tiles-rate-count = { $count }/프레임
live-tiles-rate-one = 1 타일/프레임
# $share: the part of one tile sent each frame, e.g. 0.33.
live-tiles-rate-part = { $share } 타일/프레임
live-tiles-rate-sample = 1 타일/샘플
# In the plane's corner during a walk: the share of pixels still iterating. $percent: a percentage, e.g. 54 or 1.5.
live-tiles-running = 반복 중 { $percent }%

## Live render panel: help cards
## Shown when the pointer rests on an instrument or a lamp. Words in capitals (SETTLE, SSAA, ABANDON,
## HELD, REPROJ, ADOPT, REF AHEAD, ON SCREEN) are the panel's own legends: use the same words as
## their messages above.

# The heading of a card's list of what healthy looks like.
live-card-healthy = 정상
# The heading of a card's list of what signals trouble.
live-card-trouble = 문제

live-card-walk-what = 한 프레임에 하기에는 너무 무거운 깊은 갱신은 고정되어 순회됩니다: 패스를 거듭해 렌더링하며, 패스마다 픽셀이 한 옥타브 더 반복하게 합니다(256, 512, 1,024 … 요청한 값까지). 테두리는 그 횟수를 로그 척도로 나타내며 맨 위에서 시작합니다. 빔은 순회의 진행 상황이고, 각 호는 패스 하나로, 20 ms 목표에 대한 GPU 시간에 따라 색이 칠해집니다. 흰 점은 마지막 패스의 측정값을 기다리는 중이며, 번쩍임은 새 그림이 화면에 올라가는 순간입니다.
live-card-walk-note = 한 패스로 하기에는 너무 무거운 정지 뷰도 같은 방식으로 순회하며 안정화로 표시되고, 각 슈퍼샘플링 샘플도 다시 순회하며 SSAA로 표시됩니다: 이들의 패스는 각각 최대 400 ms까지 걸릴 수 있으며, 호의 색도 그 기준으로 칠해집니다. 분할 갱신은 대신 패스마다 고리 하나를 그립니다: 모든 구간, 타일의 일부.
live-card-walk-good-sweep = 몇 프레임 만에 한 바퀴를 돌고 번쩍임으로 끝나는 스윕.
live-card-walk-good-arcs = 호가 대부분 초록색과 호박색.
live-card-walk-good-still = 깊은 정지 뷰에서: 샘플마다 더 느린 스윕, 호는 초록색에서 호박색.
live-card-walk-bad-red = 빨간 호: 예산을 넘은 패스로, 끊김으로 느껴집니다.
live-card-walk-bad-restart = 도중에 멈췄다가 다시 시작하는 스윕: 순회가 폐기되었으며, 화면의 그림은 오래된 채로 남습니다.
live-card-walk-bad-hollow = 속이 빈 호: 시간 측정값이 돌아오지 않은 패스로, 다음 패스의 크기를 짐작으로 정하게 됩니다.

live-card-age-what = 화면의 그림이 뷰보다 얼마나 뒤처졌는지를 줌 옥타브로 나타냅니다(1옥타브는 2×). 갱신이 렌더링되는 동안에는 이전 그림이 늘어난 채로 표시되며 오래되어 가고, 갱신이 도착하면 지연이 떨어집니다. 아래의 띠는 최근 몇 초입니다.
live-card-age-good-sawtooth = 약 0.3에서 0.8 사이의 톱니 모양으로, 바늘이 0.5 눈금 주위를 오갑니다.
live-card-age-good-still = 멈추면 0에 가깝습니다.
live-card-age-bad-red = 바늘이 1.0을 넘어 빨간 영역에 머묾: 그림이 2× 넘게 뒤처져, 줌하는 동안 흐리거나 블록처럼 깨져 보입니다.
live-card-age-bad-ramp = 떨어지지 않는 경사: 갱신이 끝나지 않고 있습니다(폐기를 지켜보십시오).

live-card-tiles-what = 이 프레임의 GPU 패스가 그림을 어떻게 덮는지 보여 줍니다. 이동 프레임과 순회 패스는 그림 전체를 덮으며, 캡션은 순회 패스의 반복 구간을 표시합니다. 순회 패스는 아직 탈출하지 않은 픽셀만 반복합니다: 평면은 그 비율만큼 채워지고(로그 척도, 최저 0.01%) 모서리에 그 값을 표시하므로, 순회가 깊어질수록 줄어듭니다. 한 프레임에 하기에 조금 무거운 갱신은 최대 8개의 패스로 나뉘며, 각 패스는 16-px 타일들을 엇갈려 묶은 것입니다. 한 패스로 하기에는 너무 무거운 안정된 뷰는 타일 격자가 되어, 가운데부터 바깥쪽으로 프레임마다 타일 하나씩 처리되며, 그래도 너무 무거운 타일은 스트립 단위로 나갑니다. 아래 표시: 타일 크기와 프레임당 타일 수.
live-card-tiles-good-whole = 줌하는 동안은 전체 패스, 분할 갱신은 패스 몇 개.
live-card-tiles-good-grid = 멈추면: 수십 개의 타일로 된 격자가 1~2초 안에 채워지고, 그다음 SSAA.
live-card-tiles-bad-small = 수백 개의 작은 타일로 된 격자: 패스마다 예산이 작아서 선명해지는 데 오래 걸립니다.
live-card-tiles-bad-strips = 스트립으로 나가는 타일이 많음: 타일이 격자를 정할 때 예상한 것보다 느리게 실행되었습니다.
live-card-tiles-bad-splits = 분할이 8 패스에 머묾: 갱신이 간신히 들어맞으며, 조금만 더 깊어지면 더 느린 순회로 돌아갑니다.
live-card-tiles-bad-dark = 움직이는 동안 오래 어두움: 패스가 전혀 없습니다.

live-card-scope-what = 프레임마다 한 열이며, 가장 최근 것이 오른쪽입니다: 그 프레임이 실행한 GPU 작업을 종류별로 쌓은 것입니다(이동 프레임은 초록색, 고정 패스는 비용에 따라 초록색에서 빨간색, 안정화 타일이나 정지 뷰의 순회 패스는 크림색, 슈퍼샘플링 샘플은 라벤더색, 워커 GPU의 프레임과 샘플은 파란색), 단위는 밀리초입니다. 점선은 16.7 ms(60 fps), 33, 50에 있습니다. 속이 빈 표시는 시간 측정값이 돌아오지 않은 패스입니다.
live-card-scope-good-under = 움직이는 동안 열이 16.7 선 아래.
live-card-scope-good-pin = 고정 패스는 초록색에서 호박색.
live-card-scope-good-samples = 정지 뷰의 슈퍼샘플링 샘플은 더 오래 걸릴 수 있습니다: 움직이는 것이 없기 때문입니다.
live-card-scope-bad-over = 33이나 50 ms를 넘는 열: 눈에 보이는 끊김.
live-card-scope-bad-seconds = 몇 초씩 걸리는 패스: GPU의 워치독이 장치를 재설정할 수 있습니다.
live-card-scope-bad-hollow = 속이 빈 표시가 많음: 측정 없이 작업 크기를 정하고 있습니다.

live-card-timers-what = 세 개의 GPU 타임스탬프 슬롯입니다. 패스의 시간을 잴 때 슬롯이 켜지고, 2~3프레임 뒤 측정값이 돌아오면 어두워집니다. T0은 모든 프레임의 패스를 재고, T1과 T2는 추가 고정 패스를 잽니다.
live-card-timers-good = 작업이 있을 때마다 T0이 깜박이고, 순회 중에는 T1과 T2가 깜박임.
live-card-timers-bad-stuck = 켜진 채 멈춘 램프: 측정값이 돌아오지 않아 비용 추정이 낡아 갑니다.
live-card-timers-bad-dark = GPU가 바쁜데 모두 어두움: 아무것도 측정되지 않고 있습니다.

live-card-dial-what = 빨간 바늘은 뷰의 깊이, 즉 2의 거듭제곱으로 나타낸 배율입니다. 구간은 그곳에서 GPU가 쓰는 산술입니다: 직접, df32(숫자 하나에 float 둘), 그리고 2^93을 넘으면 floatexp(지수 범위가 넓은 float). 점은 계산 중인 기준 궤도입니다: 초록색은 현재 뷰용, 호박색은 줌이 향하는 앞쪽용, 파란색은 투어의 다음 정지 구간용입니다. 고리는 설치를, 빨간 X는 뷰가 앞질러 버려진 계산을 뜻합니다.
live-card-dial-good-ahead = 호박색 점이 바늘보다 앞에 놓이고, 바늘이 도달하기 전에 고리가 생김.
live-card-dial-good-crosses = 빨간 X가 적음.
live-card-dial-bad-passing = 바늘이 마지막으로 설치된 기준 궤도를 지나침: 프레임이 기준 궤도를 기다리게 되어 멈춘 것처럼 느껴집니다.
live-card-dial-bad-crosses = 빨간 X가 반복됨: 줌이 기준 궤도 계산을 앞지르고 있으며, CPU가 낭비됩니다.
live-card-dial-bad-crossing = 2^93을 넘는 곳은 알려진 느린 지점입니다: 처음 몇 floatexp 프레임은 비용이 더 듭니다.

live-card-bank-what = 각 램프는 해당 사건이 일어나면 켜지고 약 150 ms에 걸쳐 흐려지므로, 바쁜 렌더러는 깜박이고 한가한 렌더러는 어두워집니다. 왼쪽부터: 프레임, 갱신, 기준 궤도, 그다음 워커, 안정화, 팔레트, 산술.
live-card-bank-note = 램프 위에 포인터를 올리면 그 의미를 보여 줍니다.

live-card-depth-what = 2의 거듭제곱으로 나타낸 배율: 2^100은 약 1.3 × 10^30입니다.
live-card-depth-note = 2^93을 넘으면 모든 단계의 비용이 커지고(floatexp), 기준 궤도를 계산하는 데 더 오래 걸립니다.

live-card-iter-what = 반복 요청: 각 픽셀이 집합 안으로 판정되기 전까지 반복할 수 있는 한도입니다. 깊이에 따라 올라갑니다.
live-card-iter-good = 다이브하면서 꾸준히 올라감.
live-card-iter-bad-low = 깊은데도 낮게 머묾: 나선과 미니브로트가 평평한 내부로 렌더링됩니다.
live-card-iter-bad-high = 매우 높음: 모든 갱신의 비용이 커지고, 순회가 더 오래 걸립니다.

live-card-ms-what = 표시된 프레임 사이의 시간입니다.
live-card-ms-good = 디스플레이의 간격: 60 Hz 디스플레이에서 16.7.
live-card-ms-bad-stutter = 33 초과: 눈에 보이는 끊김.
live-card-ms-bad-spikes = 50을 넘는 순간적 급등: 버벅임이며, 보통 무거운 패스 하나 때문입니다(GPU 패스 참조).

live-card-clock-what = 현지 시각의 실제 시계로, 0.1초 단위까지 표시합니다.
live-card-clock-note = 패널의 스크린샷마다 찍힌 시각이 남으므로, 문제를 찍은 그림을 앱의 로그와 맞춰 볼 수 있습니다.

live-card-tach-what = 초당 반복 스텝 수입니다. 흰색: 요청한 스텝(픽셀 × 반복). 호박색: GPU가 실제로 실행한 스텝. 그 차이는 한 번에 많은 반복을 건너뛰는 BLA(쌍선형 근사)이며, ×N이 그 배수입니다. 주행계는 세션 전체를 합산합니다.
live-card-tach-good = 깊은 곳에서 큰 차이: BLA가 제 역할을 하는 중으로, ×10에서 ×1,000.
live-card-tach-bad-together = 깊은 곳에서 바늘이 붙어 있음: 여기서는 BLA가 건너뛸 수 없어(흔히 미니브로트 내부 근처) 갱신이 느립니다.
live-card-tach-bad-red = 흰 바늘이 빨간 영역에: 한 프레임에 GPU가 할 수 있는 것보다 많이 요청하고 있습니다.

live-card-engines-what = 각 GPU가 얼마나 바쁜지 보여 주며, 점들이 그 프레임을 화면으로 나릅니다: 지난 1초 동안 그 GPU의 패스가 쓴 GPU 시간이며, 워커의 이동 프레임과 슈퍼샘플링 샘플도 포함됩니다. 화면 표시는 표시된 프레임 중 각 GPU가 만든 비율입니다. GPU가 하나뿐이면 워커 쪽은 유휴 상태로 남습니다.
live-card-engines-good = 다이브하는 동안 둘 다 바쁘며, 워커가 프레임의 일부를 공급함.
live-card-engines-bad-pinned = 창을 그리는 GPU 0이 100% 가까이에 붙어 있음: 병목이며 프레임이 끊깁니다.
live-card-engines-bad-idle = 워커가 켜져 있지만 유휴 상태: 프레임을 공급하지 않고 있습니다.

live-card-memory-what = RAM: 앱의 작업 집합이며, 빨간 선은 최고치입니다. VRAM: 앱이 GPU에 보관하는 양으로, 할당하는 크기로 계산해 종류별로(G-버퍼, 유지 및 SSAA 텍스처, 궤도 및 BLA 버퍼, 워커의 기준 궤도 사본) 카드 용량과 비교해 표시합니다.
live-card-memory-good = 줌하는 동안 평평함. 기준 궤도가 생기고 사라짐에 따라 올라갔다 내려옴.
live-card-memory-bad-leak = RAM이 올라가기만 하고 내려오지 않음: 누수.
live-card-memory-bad-vram = VRAM이 카드 용량에 가까움: 할당이 실패하거나 장치가 손실될 수 있습니다.

live-card-cores-what = 코어별 CPU 사용량(초록색에서 호박색, 빨간색까지)과, 바쁜 스레드가 각자의 역할 주위를 도는 스코프입니다: 기준 궤도 계산, 점수 매기기 풀, 워커, 보조 렌더링.
live-card-cores-good = 줌하는 동안 코어 한두 개가 바쁨. 기준 궤도를 고를 때 모든 코어에 걸친 짧은 폭주.
live-card-cores-bad-every = 모든 코어가 몇 초씩 바쁨: 극한 깊이에서의 기준 궤도 계산.
live-card-cores-bad-own = 앱 자체의 코어가 빨간 영역에 붙어 있음: 창 자체가 끊깁니다.

live-card-pixels-what = 초당 계산한 픽셀 수(Mpx/s)입니다. 흐림: 순회 중 아직 반복 중. 밝음: 완료.
live-card-pixels-good = 움직이는 동안 수십 Mpx/s.
live-card-pixels-bad = 그림이 오래되었는데 0에 가까움: GPU가 무언가(기준 궤도나 측정값)를 기다리고 있습니다.

## Live render panel: the lamps' help cards

live-card-lamp-plan-what = 프레임이 계획되었습니다: 해상도, 반복 요청, 예산을 정하고 갱신할지 결정했습니다.
live-card-lamp-plan-good = 뷰나 설정이 바뀔 때마다 깜박임.
live-card-lamp-plan-bad = 줌하는 동안 어두움: UI 스레드가 멈췄습니다.

live-card-lamp-dispatch-what = 이 프레임에 작업이 GPU로 보내졌습니다.
live-card-lamp-dispatch-good = 무언가 바뀌는 동안 켜지고, 유휴 상태에서는 어두움.
live-card-lamp-dispatch-bad = 아무것도 바뀌지 않는데 바쁨: 앱이 안정되지 않고 있습니다.

live-card-lamp-live-what = 화면의 그림이 이 프레임 자체의 렌더링입니다.
live-card-lamp-live-note = 깊은 다이브 중에는 거의 켜지지 않으며, 이는 정상입니다: 유지와 재투영을 보십시오.
live-card-lamp-live-good = 프레임 비용이 싼 얕은 깊이에서 켜짐.

# "Prefer detail" is a setting's name.
live-card-lamp-held-what = 다음 그림이 그 아래에서 만들어지는 동안 마지막 완성 그림이 화면에 있습니다: 딥 줌의 갱신이거나, 줌하는 동안 디테일 우선이 켜져 있을 때 정지 뷰의 다음 타일이나 슈퍼샘플링 샘플입니다.
live-card-lamp-held-good-dive = 딥 줌 중에 한동안씩 켜지며, 채택으로 끊김.
live-card-lamp-held-good-still = 정지 뷰가 선명해지는 동안 계속 켜져 있고, SSAA가 깜박임.
live-card-lamp-held-bad = 줌하는 동안 채택 없이 몇 초씩 켜져 있음: 갱신이 끝나지 않고 있습니다.

live-card-lamp-reproj-what = 새 그림이 준비될 때까지, 움직이는 뷰에 맞게 늘인 마지막 그림입니다.
live-card-lamp-reproj-good = 움직이는 동안 짧게 반짝임.
live-card-lamp-reproj-bad = 오래 켜져 있음: 그림은 늘어날수록 흐려집니다.

live-card-lamp-reading-what = GPU에서 측정값이 돌아왔습니다: 패스의 시간, 탈출 범위 또는 내용.
live-card-lamp-reading-good = 렌더링하는 동안 꾸준히 깜박임.
live-card-lamp-reading-bad = 렌더링하는 동안 오래 어두움: 시간 측정값이 돌아오지 않아 예산이 짐작으로 정해지고 있습니다.

live-card-lamp-pin-pass-what = 고정된 순회의 패스: 그림을 반복 구간 하나만큼 더 렌더링했습니다.
live-card-lamp-pin-pass-good = 채택으로 끝나는 연속된 패스.
live-card-lamp-pin-pass-bad = 폐기로 끝나는 연속된 패스.

live-card-lamp-split-what = 분할 갱신의 패스: 타일의 일부를 전체 반복 횟수로 렌더링했습니다.
live-card-lamp-split-good = 채택으로 끝나는 짧은 연속.
live-card-lamp-split-bad = 허용되는 최대인 8개의 연속: 갱신이 간신히 들어맞습니다.

live-card-lamp-adopt-what = 완성된 갱신이 화면의 그림을 대체했습니다.
live-card-lamp-adopt-good = 줌하는 동안 초당 여러 번.
# "Picture age" names the instrument (its title, PICTURE AGE).
live-card-lamp-adopt-bad = 줌하는 동안 드묾: 그림이 오래되어 갑니다(그림 지연 참조).

live-card-lamp-abandon-what = 진행 중인 갱신이 버려졌습니다: 뷰가 너무 멀리 움직였거나, 너무 오래되었거나, 더 새로운 갱신이 대체했습니다. 뷰가 앞지른 기준 궤도 계산은 이 램프를 희미하게 켭니다.
live-card-lamp-abandon-good = 드묾, 또는 이동 직후 한 번.
live-card-lamp-abandon-bad = 잦음: 작업이 낭비되고 그림이 오래된 채로 남습니다.

live-card-lamp-ref-live-what = 현재 상태의 뷰를 위한 기준 궤도를 계산하는 중입니다: 기준 궤도가 없거나, 가지고 있던 것을 벗어났거나, 더 긴 것이 필요합니다.
live-card-lamp-ref-live-good = 짧은 깜박임.
live-card-lamp-ref-live-bad-long = 오래 켜짐: 깊은 뷰가 이것을 기다립니다.
live-card-lamp-ref-live-bad-lookahead = 다이브 중에 자주 켜지는데 기준 선행이 어두움: 미리 계산이 따라가지 못하고 있습니다.

live-card-lamp-ref-ahead-what = 줌이 향하는, 뷰보다 깊은 곳의 미리 계산 기준 궤도를 계산하는 중입니다.
live-card-lamp-ref-ahead-good = 다이브 중의 규칙적인 깜박임.
live-card-lamp-ref-ahead-bad = 깊은 다이브 중에 어두움: 뷰가 기준 궤도를 앞지르게 됩니다.

live-card-lamp-ref-hold-what = 투어의 정지 구간 미리 가져오기: 다음 정지 키프레임의 기준 궤도를 그쪽으로 미끄러져 가는 동안 계산해 두어, 재생이 기다리는 일이 없게 합니다.
live-card-lamp-ref-hold-note = 투어 재생 외에는 어두움.
live-card-lamp-ref-hold-good = 투어의 각 정지 구간 앞에서 한 번 깜박임.
live-card-lamp-ref-hold-bad = 정지 구간 내내 켜져 있음: 미리 가져오기가 늦어 투어가 기다립니다.

live-card-lamp-series-what = 새 기준 궤도의 급수 근사로, 모든 픽셀이 처음 반복들을 건너뛸 수 있게 합니다.
live-card-lamp-series-good = 짧음.
live-card-lamp-series-bad = 깊은 곳에서 오래 걸림: 기준 궤도의 설치를 늦춥니다.

live-card-lamp-worker-what = 두 번째 GPU가 프레임을 전달했습니다.
live-card-lamp-worker-good = 워커 GPU가 켜져 있을 때 꾸준히 깜박임.
live-card-lamp-worker-bad = 워커가 켜져 있는데 어두움: 프레임을 전달하지 않고 있습니다.

live-card-lamp-tile-what = 안정화 타일 또는 그 스트립 하나가 GPU로 보내졌습니다.
live-card-lamp-tile-good = 멈춘 뒤 한 차례 연속으로 켜지고, 그다음 조용함.
live-card-lamp-tile-bad = 멈추지 않음: 무언가가 계속 뷰나 설정을 바꾸고 있어 격자가 계속 다시 시작합니다.

live-card-lamp-ssaa-what = 슈퍼샘플링 샘플이 안정된 그림에 더해졌습니다.
live-card-lamp-ssaa-good = 안정화 후 연속된 깜박임, 그다음 어두움.
live-card-lamp-ssaa-bad = 계속 다시 시작함: 안정화가 자꾸 방해받고 있습니다.

live-card-lamp-palette-what = 색상 정규화가 새 탈출 범위를 반영해, 색이 줌을 따라갑니다.
live-card-lamp-palette-good = 줌하는 동안 깜박임.
live-card-lamp-palette-bad = 줌하는 동안 어두움: 팔레트 범위가 고정되어 색이 평평해지고 대비가 낮아집니다.

live-card-lamp-mode-what = 산술이 바뀌었습니다: 직접, df32 또는 floatexp.
live-card-lamp-mode-good = 2^93 같은 경계에서 한 번 깜박임.
live-card-lamp-mode-bad = 앞뒤로 깜박임: 뷰가 경계를 사이에 두고 오가고 있습니다.

## The Render client window (File ▸ Render client…): lending this machine's GPU to a render farm

farm-client-title = 렌더 클라이언트
farm-client-intro = 렌더 팜을 위해 프레임을 렌더링합니다. 컨트롤러 — 다른 컴퓨터의 도구 ▸ 팜에서 렌더링… — 가 자신의 주소와 팜 키를 보여 줍니다. 이 컴퓨터는 포트를 열지 않으며, 컨트롤러에 연결합니다.
# A field label, and the label of the status row that names the controller.
farm-client-controller = 컨트롤러
farm-client-find = 찾기
farm-client-find-hover = 이 네트워크에서 컨트롤러를 찾습니다(팜이 수신 대기 중인 동안 컨트롤러가 응답합니다)
farm-client-key = 팜 키
# A checkbox beside the masked farm key: show it in clear.
farm-client-show-key = 표시
# A field label (the name this machine goes by), and the label of the status row about this machine.
farm-client-machine = 이 컴퓨터
farm-client-machine-hover = 컨트롤러가 이 컴퓨터에 대해 표시하는 이름입니다.
farm-client-share = 공유 드라이브
# Greyed hint inside the empty field; the path is an example.
farm-client-share-hint = 선택 사항 — 예: \\fileserver\share
farm-client-share-hover = 컨트롤러가 사용하는 공유 드라이브에 대한 이 컴퓨터의 경로입니다(공유 모드): 프레임을 연결로 보내는 대신 그곳에 기록합니다. 비워 두면 모든 프레임을 보냅니다.
farm-client-gpu = 그래픽 카드
# The label of the row with the idle option.
farm-client-when = 조건
# A checkbox followed by a number of minutes ("Only when idle for [5] min").
farm-client-when-idle = 다음 시간 이상 유휴일 때만
farm-client-when-idle-hover = 아무도 이 컴퓨터의 키보드나 마우스를 이만큼 건드리지 않았거나 화면이 잠겨 있을 때만 작업을 받습니다. 누군가 사용하기 시작하면 진행 중인 프레임은 즉시 멈추고 팜으로 돌아갑니다.
# A collapsible section: the largest job this machine accepts.
farm-client-limits = 한도
farm-client-limits-note = 이보다 많은 것을 요구하는 작업은 줄여서 받지 않고 거부합니다.
farm-client-limit-size = 최대 프레임
farm-client-limit-ss = 최대 슈퍼샘플링
farm-client-limit-iter = 최대 반복 횟수

## Render client: the graphics-card choice

farm-client-gpu-system = 시스템의 선택
farm-client-gpu-every = 모든 그래픽 카드
# $count: this machine's graphics cards.
farm-client-gpu-every-n = 모든 그래픽 카드({ $count }개)
# $count: this machine's graphics cards.
farm-client-gpu-every-each = 모든 그래픽 카드({ $count }개) — 카드마다 세션 하나
# A card the window cannot name. $number: its number (or numbers) as --list-adapters gives it.
farm-client-gpu-adapter = 어댑터 { $number }
farm-client-gpu-finding = 이 컴퓨터의 그래픽 카드를 찾는 중…
# "NAME · GPU 1" is how such a machine is named in the controller's list: keep it as it is.
farm-client-gpu-hover = 팜의 프레임을 렌더링할 그래픽 카드입니다. 모든 카드를 고르면 각 카드가 별도의 컴퓨터로 팜에 참여하며 — “NAME · GPU 1”, “NAME · GPU 2” — 일시 정지와 연결 끊기는 그 모두에 적용됩니다.

## Render client: looking for controllers on the network (the Find button)

farm-client-finding = 이 네트워크에서 컨트롤러를 찾는 중…
# $port: the UDP port number.
farm-client-found-none = 응답한 컨트롤러가 없습니다. 컨트롤러는 팜이 수신 대기 중인 동안 응답하며, 방화벽이 UDP { $port }를 허용해야 합니다.
farm-client-found-pick = 이 네트워크에 있음 — 사용할 컨트롤러를 클릭하십시오:
# $identity: the controller's identity fingerprint.
farm-client-found-hover = 식별자 { $identity } — 이 컨트롤러 사용(팜 키는 여전히 필요합니다)
# After a warning sign. $version and $git: this app's version and build.
farm-client-found-other-build = 이것과 다른 빌드입니다({ $version } { $git }): 이 컴퓨터를 거부할 것입니다.

## Render client: what is wrong with what was typed, or with starting the client

farm-client-bad-address = 컨트롤러의 주소를 그 창에 표시된 대로 HOST:PORT 형식으로 입력하십시오(예: 192.168.1.20:46733).
# $why: what is wrong with the key (in English).
farm-client-bad-key = 팜 키: { $why }
# $why: what is wrong with the name (in English).
farm-client-bad-name = 이름: { $why }
farm-client-no-config = 팜 키를 보관할 설정 폴더가 없습니다.
# $why: the system's reason.
farm-client-start-failed = 렌더 클라이언트를 시작할 수 없습니다: { $why }

## Render client: the state, in the status heading

farm-client-phase-connecting = 연결 중…
farm-client-phase-checking = 자체 점검 실행 중…
farm-client-phase-idle = 연결됨 — 작업 대기 중
farm-client-phase-rendering = 렌더링 중
farm-client-phase-in-use = 대기 중 — 누군가 이 컴퓨터를 사용하고 있습니다
farm-client-phase-paused = 일시 정지됨
farm-client-phase-retrying = 컨트롤러에 연결할 수 없음 — 다시 시도 중
# $seconds: the wait before the next try.
farm-client-phase-retrying-in = 컨트롤러에 연결할 수 없음 — { $seconds } s 후 다시 시도
farm-client-phase-disconnected = 연결 끊김
farm-client-phase-ended = 종료됨

## Render client: the line under the status heading

# $address: the controller's address (HOST:PORT).
farm-client-detail-connecting = { $address }에 연결 중…
# $controller: the controller's name.
farm-client-detail-checking = “{ $controller }”에 연결됨 — 자체 점검 실행 중
# $controller: the controller's name.
farm-client-detail-approval = “{ $controller }”에 연결됨 — 그쪽 사용자가 이 컴퓨터를 승인하기를 기다리는 중
# $controller: the controller's name.
farm-client-detail-idle = “{ $controller }”에 연결됨 — 유휴
# $why: the check that failed and why (in English).
farm-client-detail-check-failed = 자체 점검 실패 — { $why }
# $why: the controller's reason (in English).
farm-client-detail-refused = 컨트롤러가 거부함: { $why }
# $why: the controller's reason (in English).
farm-client-detail-closed = 컨트롤러가 연결을 닫았습니다: { $why }
# $why: the controller's reason (in English).
farm-client-detail-ended = 컨트롤러가 연결을 종료했습니다: { $why }
# $job: the job's name.
farm-client-detail-bundle = 작업 “{ $job }” — 번들 받는 중
# $job: the job's name.
farm-client-detail-job-waiting = 작업 “{ $job }” — 렌더링할 프레임을 기다리는 중
# $why: the limit the job is over (in English).
farm-client-detail-job-refused = 이 작업을 거부함: { $why }
farm-client-detail-job-over = 작업이 끝났습니다
# $first and $last: frame numbers.
farm-client-detail-rendering = 프레임 { $first }–{ $last } 렌더링 중
# $time: how long nobody must use the machine ("90 s", "5 min").
farm-client-detail-in-use = 대기 중 — 누군가 이 컴퓨터를 사용하고 있습니다. { $time } 동안 아무도 사용하지 않으면 작업을 재개합니다
farm-client-detail-paused = 일시 정지됨 — 컨트롤러에 알렸습니다
farm-client-detail-pausing = 일시 정지하는 중 — 진행 중인 프레임을 마무리하고 있습니다
farm-client-detail-cancelled = 진행 중인 프레임을 취소하고 일시 정지했습니다
# A machine with several graphics cards. $rendering: the cards rendering now; $total: all its cards (two or more).
farm-client-rendering-on-cards = 그래픽 카드 { $total }개 중 { $rendering }개에서 렌더링 중

## Render client: the status rows

# The value of the Controller row. $name: the controller's name; $address: HOST:PORT; $identity: its identity fingerprint.
farm-client-row-controller-value = “{ $name }” · { $address } · 식별자 { $identity }
# The value of the This machine row. $name: this machine's name; $identity: its identity fingerprint.
farm-client-row-machine-value = “{ $name }” · 식별자 { $identity }
farm-client-row-gpu = GPU
# One graphics card's row. $slot: the card's number.
farm-client-row-gpu-n = GPU { $slot }
farm-client-row-job = 작업
# The frames handed to this machine.
farm-client-row-run = 할당
farm-client-row-sent = 보냄
# The Run row, in parts joined with " · ": the run, the frame in progress, its time. $first and $last: frame numbers.
farm-client-run-frames = 프레임 { $first }–{ $last }
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-client-run-frame = 프레임 { $frame }({ $count }개 중 { $index }번째)
# $time: the time spent on it so far ("2.1 s").
farm-client-run-frame-time = 이 프레임에 { $time }
# One graphics card's run. $frame: the frame in progress; $first and $last: the run's frames.
farm-client-session-frame = 프레임 { $frame }({ $first }–{ $last } 중)
farm-client-session-no-card = 그래픽 카드를 아직 알 수 없음
# The Sent row. $count: frames sent; $time: the mean time per frame ("2.1 s").
farm-client-sent = 프레임 { $count }개 · 프레임당 { $time }
# The Sent row of a machine with several cards. $count: frames sent; $split: each card's share ("812 + 422"); $time: the mean time per frame.
farm-client-sent-split = 프레임 { $count }개({ $split }) · 프레임당 { $time }
# After a warning sign. $note: how the GPUs differ, and both GPUs (in English).
farm-client-gpu-note = 이 컴퓨터에는 { $note }이(가) 있습니다. 이 컴퓨터의 프레임은 컨트롤러의 프레임과 약간 다르며, 정지 장면에서 드러날 수 있습니다.
# A collapsible section: the self-check's results.
farm-client-self-check = 자체 점검
farm-client-thumb-hover = 이 컴퓨터가 마지막으로 보낸 프레임
farm-client-resume = 재개
farm-client-resume-hover = 다시 작업 받기
farm-client-pause = 일시 정지
farm-client-pause-hover = 진행 중인 프레임을 마친 뒤 더 이상 작업을 받지 않습니다. 컨트롤러가 이 컴퓨터의 프레임을 다른 컴퓨터에 넘깁니다.
farm-client-cancel-frame = 프레임 취소
farm-client-cancel-frame-hover = 진행 중인 프레임을 지금 멈추고 일시 정지합니다. 컨트롤러가 그 프레임을 다른 곳에서 렌더링합니다.
farm-client-live-paused = 이 컴퓨터가 팜 프레임을 렌더링하는 동안 라이브 뷰를 일시 정지했습니다.
farm-client-close = 닫기
farm-client-close-active-hover = 창을 닫습니다. 이 컴퓨터는 팜에 남습니다(파일 메뉴에서 다시 열 수 있습니다)
farm-client-close-hover = 창 닫기
farm-client-disconnect = 연결 끊기
farm-client-disconnect-hover = 팜에서 나갑니다: 진행 중인 프레임은 돌려줍니다
farm-client-connect = 연결
farm-client-connect-hover = 컨트롤러에 연결해 그 컨트롤러를 위해 프레임을 렌더링합니다

## The Render on farm window (Tools ▸ Render on farm…): the controller of a render farm

farm-ctl-title = 팜에서 렌더링
farm-ctl-out = 출력 폴더
farm-ctl-browse = 찾아보기…
farm-ctl-out-note = 프레임, 그리고 farm/ 안의 작업 상태: 같은 작업을 다시 실행하면 이어서 진행합니다.
farm-ctl-size = 크기
farm-ctl-size-custom = 사용자 정의…
# The size list's choice when the size is typed in. $width and $height: pixels.
farm-ctl-size-custom-value = 사용자 정의 — { $width }×{ $height }
farm-ctl-fps = 초당 프레임 수
farm-ctl-ss = 슈퍼샘플링
farm-ctl-port = 수신 포트
farm-ctl-port-hover = 클라이언트가 이 포트로 연결합니다. 로컬 네트워크에 대해 방화벽에서 허용하십시오.
farm-ctl-share = 공유 드라이브
# Greyed hint inside the empty field.
farm-ctl-share-hint = 선택 사항 — 이 컴퓨터에서의 경로
farm-ctl-share-hover = 공유 모드: 같은 드라이브가 있는 클라이언트는 프레임을 연결로 보내는 대신 그곳에 기록하며, 이 컴퓨터가 그곳에서 모든 프레임을 확인합니다. 드라이브가 없는 클라이언트는 프레임을 보냅니다.
# A label followed by a number of clients and two checkboxes.
farm-ctl-start-with = 시작 조건
# The unit after the number in the box ("Start with [3] clients"). $count: that number.
farm-ctl-start-clients = 클라이언트
farm-ctl-start-clients-hover = 이만큼의 컴퓨터가 연결되어 자체 점검을 통과하면 작업이 시작됩니다. 다른 컴퓨터는 나중에 참여할 수 있습니다.
farm-ctl-local = 이 컴퓨터에서도 렌더링
farm-ctl-local-hover = 이 컴퓨터도 클라이언트 하나로 셉니다.
farm-ctl-sharing = 기준 궤도 공유
farm-ctl-sharing-hover = 깊은 프레임의 기준 궤도는 계산에 몇 분에서 몇 시간이 걸릴 수 있습니다. 공유하면 한 번만 계산해 다른 컴퓨터에 전달합니다. 공유된 기준 궤도는 다르지만 똑같이 유효한 기준 궤도입니다: 극한 깊이에서는 자체 기준 궤도를 계산한 프레임과 몇몇 픽셀이 다를 수 있습니다.
farm-ctl-key = 팜 키
# A checkbox beside the shortened farm key: show all of it.
farm-ctl-show-key = 표시
farm-ctl-copy-key = 키 복사
farm-ctl-copy-key-hover = 각 클라이언트의 렌더 클라이언트 창에 붙여 넣으십시오
# In place of the farm key, when there is none.
farm-ctl-no-key = 아직 없음
farm-ctl-make-key = 키 만들기
farm-ctl-make-key-hover = 클라이언트가 참여하려면 필요합니다. 렌더링을 누를 때도 만들어집니다
# What to do on the other machines. $address: this controller's HOST:PORT.
farm-ctl-steps = 각 클라이언트에서: 파일 ▸ 렌더 클라이언트…, 컨트롤러 { $address }, 그리고 팜 키.
# The same when this machine's address is not known. $port: the port number.
farm-ctl-steps-port = 각 클라이언트에서: 파일 ▸ 렌더 클라이언트…, 이 컴퓨터의 주소와 포트 { $port }, 그리고 팜 키.
# The size of the job. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
farm-ctl-summary = 프레임 { $count }개 · 투어 { $seconds }s · { $width }×{ $height } ss{ $ss }
farm-ctl-strip-hover = 작업의 모든 프레임, 왼쪽에서 오른쪽으로: 완료, 렌더링 중, 남음, 포기함.
farm-ctl-no-tour = 불러온 투어가 없습니다.
# $folder: the folder; $why: the system's reason.
farm-ctl-out-error = 출력 폴더 { $folder }: { $why }
# $why: the system's reason.
farm-ctl-start-failed = 팜을 시작할 수 없습니다: { $why }
# $code: the controller process's exit code, as the code gives it.
farm-ctl-ended-exit = 컨트롤러가 종료되었습니다(종료 코드 { $code }).

## Render on farm: the state, in the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-phase-waiting = 렌더 클라이언트 대기 중({ $needed }개 중 { $connected }개 연결됨)
farm-ctl-phase-preparing = 작업 준비 중
farm-ctl-phase-rendering = 렌더링 중
farm-ctl-phase-paused = 일시 정지됨
farm-ctl-phase-complete = 완료
farm-ctl-phase-stopped = 중지됨
farm-ctl-phase-ended = 종료됨

## Render on farm: the words after the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-detail-waiting = 렌더 클라이언트 대기 중: { $needed }개 중 { $connected }개 연결됨
farm-ctl-detail-preparing = 모든 클라이언트를 위해 이 컴퓨터에서 정규화 앵커를 한 번 측정하는 중
farm-ctl-detail-no-space = 출력 폴더에 공간이 생기기를 기다리는 중
farm-ctl-detail-paused = 일시 정지됨: 새 작업을 나눠 주지 않습니다
farm-ctl-detail-not-rendered = 렌더링되지 않은 프레임이 있는 채로 끝났습니다 — 같은 폴더로 다시 실행하면 이어서 진행합니다
farm-ctl-detail-stopped = 중지됨 — 같은 폴더로 다시 실행하면 이어서 진행합니다
# $code: the controller process's exit code.
farm-ctl-detail-ended = 종료됨(종료 코드 { $code })
farm-ctl-detail-finishing = 마무리 중

## Render on farm: progress

# On the progress bar. $done: frames done; $total: frames in the job.
farm-ctl-progress = { $done } / { $total } 프레임
# Added to the progress bar. $rate: frames per second, with decimals.
farm-ctl-progress-rate = { $rate } 프레임/s
# Added to the progress bar. $time: the time still to go ("38m 10s").
farm-ctl-progress-eta = 약 { $time } 남음
# Facts under the progress bar, joined with " · ". $received and $sent: kilobytes per second.
farm-ctl-io = 수신 { $received } KB/s · 송신 { $sent } KB/s
# $space: free space on the output drive ("412 GB").
farm-ctl-free = { $space } 사용 가능
# $time: the time since the job started ("38m 10s").
farm-ctl-elapsed = { $time } 경과
# $count: frames given up on.
farm-ctl-failed = { $count }개 렌더링 안 됨
# Reference orbits. $built: built for this job; $cached: found in the cache; $reused: reused between frames; $shared: passed between machines.
farm-ctl-refs = 기준 궤도: { $built }개 계산, { $cached }개 캐시에서, { $reused }개 재사용 · { $shared }개 공유
# After a warning sign.
farm-ctl-storage-low = 출력 폴더가 거의 가득 찼습니다: 공간이 생길 때까지 새 작업을 나눠 주지 않습니다.
# After a warning sign.
farm-ctl-gpus-differ = 이 팜의 GPU가 서로 다릅니다. 그 프레임들은 약간 다르며, 정지 장면에서 드러날 수 있습니다.
# After a warning sign. $count: the GPU classes in the farm.
farm-ctl-gpus-differ-classes = 이 팜의 GPU가 서로 다릅니다(GPU 등급 { $count }개: 다른 GPU, 드라이버 또는 시험 렌더링). 그 프레임들은 약간 다르며, 정지 장면에서 드러날 수 있습니다.
# $gpu: this machine's GPU, API and driver.
farm-ctl-this-gpu = 이 컴퓨터: { $gpu }
# A machine whose GPU differs. $name: the machine; $note: how its GPU differs (in English); $adapter: its GPU.
farm-ctl-gpu-row = { $name }: { $note } — { $adapter }
# The same with its driver. $driver: the driver's name and version.
farm-ctl-gpu-row-driver = { $name }: { $note } — { $adapter }, 드라이버 { $driver }

## Render on farm: the table of machines

# Column headers (short: the table is narrow).
farm-ctl-col-machine = 컴퓨터
farm-ctl-col-state = 상태
farm-ctl-col-gpu = GPU
farm-ctl-col-frames = 프레임
farm-ctl-col-per-frame = 프레임당
# How the machine delivers its frames (a speed, or "shared drive").
farm-ctl-col-link = 전송
# When the controller last heard from the machine.
farm-ctl-col-heard = 응답
# In the machine's tooltip. $count: its frames that failed verification.
farm-ctl-strikes = 프레임 { $count }개가 검증에 실패함
# In the Link column: it delivers its frames through the shared drive.
farm-ctl-link-share = 공유 드라이브
# In the Heard column. $seconds: how long ago.
farm-ctl-heard-ago = { $seconds } s 전
farm-ctl-readmit = 다시 허용
farm-ctl-readmit-hover = 이 컴퓨터가 다시 연결되면 다시 참여하게 합니다
farm-ctl-remove = 제거
farm-ctl-remove-hover = 이 컴퓨터를 작업에서 뺍니다. 그 프레임은 다른 컴퓨터로 갑니다
# The GPU column's tooltip.
farm-ctl-probe-same = 시험 렌더링이 이 컴퓨터의 것과 동일합니다.
# $count: the pixels that differ.
farm-ctl-probe-differs = 시험 렌더링이 이 컴퓨터의 것과 { $count }픽셀 다릅니다: 그 프레임은 이 컴퓨터의 프레임과 약간 다를 것입니다.
farm-ctl-probe-unknown = 시험 렌더링을 아직 비교하지 않았습니다.
# $driver: the driver's name and version.
farm-ctl-gpu-driver = 드라이버 { $driver }
# After a warning sign. $note: how its GPU differs (in English: "a different GPU").
farm-ctl-gpu-note = 이 컴퓨터와 비교해: { $note }

## Render on farm: a machine's state, in the State column

# $count: its frames that failed verification.
farm-ctl-state-removed-bad = 제거됨: 잘못된 프레임 { $count }개
farm-ctl-state-removed = 사용자가 제거함
farm-ctl-state-removing = 제거 중 — 진단 정보 수집 중
farm-ctl-state-gone = 사라짐
farm-ctl-state-leaving = 떠나는 중
farm-ctl-state-checking = 점검 중(자체 점검)
farm-ctl-state-paused = 그쪽 사용자가 일시 정지함
farm-ctl-state-parked = 보류: 두 번 충돌함 — 다시 연결하면 재참여
farm-ctl-state-refused = 이 작업을 거부함(자체 한도)
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-ctl-state-rendering = { $frame } 렌더링 중({ $index }/{ $count })
farm-ctl-state-starting = 할당분 시작 중
farm-ctl-state-idle = 유휴
farm-ctl-state-admitted = 허용됨 — 작업 대기 중

## Render on farm: the buttons

# A collapsible section: the controller's messages.
farm-ctl-events = 이벤트
farm-ctl-resume = 재개
farm-ctl-pause = 일시 정지
farm-ctl-pause-hover = 새 작업을 나눠 주지 않습니다. 진행 중인 프레임은 끝까지 진행됩니다
farm-ctl-live-paused = 이 컴퓨터가 팜을 위해 렌더링하는 동안 라이브 뷰를 일시 정지했습니다.
farm-ctl-close = 닫기
farm-ctl-close-active-hover = 창을 닫습니다. 작업은 계속 실행됩니다(도구 메뉴에서 다시 열 수 있습니다)
farm-ctl-close-hover = 창 닫기
farm-ctl-copy-command = 명령 복사
farm-ctl-copy-command-hover = 같은 동작을 하는 명령줄 복사
farm-ctl-stop = 중지
# The Stop button once a stop has been asked for and the job has not ended after 10 seconds.
farm-ctl-stop-now = 지금 중지
farm-ctl-stop-hover = 지금까지의 프레임으로 작업을 끝냅니다. 같은 작업을 다시 실행하면 이어서 진행합니다
farm-ctl-render = 렌더링
farm-ctl-render-hover = 클라이언트 수신 대기를 시작하고 그 클라이언트들에서 투어를 렌더링합니다

## The Render tour window (Tools ▸ Render tour…)

tour-render-title = 투어 렌더링
tour-render-out = 출력 폴더
tour-render-browse = 찾아보기…
tour-render-out-hover = 프레임을 기록할 전체 경로입니다.
tour-render-prefix = 프레임 접두사
# <prefix> stands for the prefix typed in the field.
tour-render-prefix-hover = 프레임은 <접두사>_00000.png로 기록됩니다
tour-render-size = 크기
tour-render-size-custom = 사용자 정의…
# The size list's choice when the size is typed in. $width and $height: pixels.
tour-render-size-custom-value = 사용자 정의 — { $width }×{ $height }
tour-render-fps = 초당 프레임 수
tour-render-fps-hover = 1 fps 미만도 올바른 값입니다: 긴 다이브를 0.25 fps로 표본화하는 것이, 프레임 수천 개를 렌더링하지 않고 깊은 회귀 검사용 정지 구간을 살펴보는 방법입니다.
tour-render-ss = 슈퍼샘플링
tour-render-chapter = 챕터
tour-render-whole = 투어 전체
tour-render-chapter-hover = 챕터 하나를 렌더링해도 전역 프레임 번호를 유지하므로, 그 프레임을 전체 시퀀스에 그대로 다시 넣을 수 있습니다.
tour-render-mp4 = mp4 조립
tour-render-mp4-hover = 끝난 뒤 ffmpeg를 실행합니다(PATH에 있어야 함). PNG 프레임은 유지됩니다.
tour-render-overwrite = 기존 프레임 덮어쓰기
tour-render-overwrite-hover = 끄면: 렌더링이 멈추고 물어보는데, 여기서는 물어볼 수 없습니다.
tour-render-resume = 중단된 렌더링 이어서 하기
tour-render-resume-hover = 출력 폴더에 이미 있는 프레임은 유지하고 빠진 프레임만 렌더링합니다. 가장 최근 프레임을 먼저 확인해 불완전하면 버리고(렌더링이 죽을 때 기록하던 프레임이 그것이므로), 온전한 프레임이 나올 때까지 거슬러 올라갑니다. 다른 크기의 프레임이 든 폴더는 이 시퀀스에 섞지 않고 거부합니다.
# $count: this machine's graphics cards.
tour-render-all-gpus = 모든 그래픽 카드 사용({ $count }개)
tour-render-all-gpus-hover = 이 컴퓨터 안의 렌더 팜을 통해(--gpus all) 이 컴퓨터의 모든 그래픽 카드에서 동시에 렌더링합니다. 프레임, 그 이름, 이어서 하기, mp4는 카드 하나일 때와 같으며, 투어 전체를 팜 자체의 순서로 렌더링합니다. 카드 모델이 다르면 몇몇 픽셀을 약간 다르게 그립니다: 정지 장면과 디졸브는 한 카드에 머뭅니다.
tour-render-order = 렌더링 순서
tour-render-sequential = 순차
tour-render-sequential-hover = 프레임 0, 1, 2, … 순서대로(기본값).
tour-render-progressive = 점진
tour-render-progressive-hover = 먼저 투어 전체를 미리 본 뒤 최대 디테일로 다듬습니다: 키프레임을 먼저 렌더링하고, 그다음 시간상 가장 큰 간격을 모든 프레임이 생길 때까지 반으로 나눕니다. 어느 쪽이든 프레임은 올바른 번호에 놓이므로 이어서 하기와 mp4는 그대로 동작합니다 — 일찍 멈추고 거친 플립북을 눈으로 확인한 뒤, 어느 순서로든 이어서 하기로 다시 시작할 수 있습니다.
# The cost of the render. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
tour-render-summary = 프레임 { $count }개 · 투어 { $seconds }s · { $width }×{ $height } ss{ $ss }
# The disk space the frames will take (an estimate). $size: a size ("5.3 GB").
tour-render-disk = 프레임 ≈{ $size }
# $size: the frames' estimated size; $free: the free space on that drive ("6.0 GB").
tour-render-disk-free = 프레임 ≈{ $size } · { $free } 사용 가능
# After a warning sign. $disk: the line above ("≈51.0 GB of frames · 6.0 GB free").
tour-render-disk-short = { $disk } — 전체 시퀀스를 담을 공간이 부족합니다
tour-render-separate = 스크립트를 디스크에서 읽어 별도 프로세스로 렌더링하므로, 저장하지 않은 편집은 포함되지 않습니다 — 그리고 그곳에서 GPU 장치가 손실되어도 이 세션까지 함께 종료되지 않습니다.
# On the progress bar. $done: frames done; $total: frames to render.
tour-render-progress = { $done } / { $total } 프레임
# Before the render's first progress line.
tour-render-starting = 시작 중…
tour-render-live-paused = 라이브 뷰 일시 정지됨 — 렌더링이 GPU를 독차지하므로 더 빨리 끝나고 자체 프레임 비용을 정확히 측정합니다.
tour-render-cancel = 취소
tour-render-cancel-hover = 렌더링하지 않고 닫기
tour-render-copy-command = 명령 복사
tour-render-copy-command-hover = 같은 동작을 하는 명령줄을 클립보드에 복사
tour-render-stop = 렌더링 중지
tour-render-render = 렌더링
tour-render-render-hover = 프레임 시퀀스 렌더링 시작
# $folder: the output folder.
tour-render-finished = 렌더링 완료 → { $folder }
# $why: the render's own reason (in English).
tour-render-failed = 렌더링 실패: { $why }
# $last: the render's last progress line (in English), or nothing.
tour-render-failed-or-stopped = 렌더링이 실패했거나 중지되었습니다. { $last }
# $why: the system's reason.
tour-render-no-exe = 실행 파일을 찾을 수 없습니다: { $why }
# $why: the system's reason.
tour-render-start-failed = 렌더링을 시작할 수 없습니다: { $why }

## The Custom formula dialog (Fractal ▸ Custom formula…)
## The formula language itself stays as it is in every language: function names (sin, cosh, conj…),
## operators, variables (z, c, p1), and the formulas. Keep them as written inside these messages.

# The window's title.
formula-dialog-title = 사용자 정의 수식
# A small line at the top of the window, above the formula.
formula-dialog-intro = Fractint 방식의 식으로 쓴 한 단계
# The two ways to show and edit the formula (top right of the window).
formula-dialog-textbook = 교과서
formula-dialog-textbook-hover = 수식을 교과서처럼 조판해 보여 주고, 그 모양 그대로 편집합니다
formula-dialog-text = 텍스트
formula-dialog-text-hover = 수식을 텍스트로 편집
# The tooltip of the "LaTeX" button (the button's own text stays LaTeX).
formula-dialog-latex-hover = 여기에 조판된 모양 그대로 수식을 LaTeX로 복사
# The drop-down list of example formulas (the collection).
formula-dialog-examples = 예제
# Help under the formula in Textbook mode. The characters / ^ ( are what the user types: keep them.
formula-dialog-help-textbook = 텍스트에서처럼 입력하십시오: /는 앞의 항을 분수로, ^는 지수를, (는 괄호를 만듭니다. 화살표 키로 수식 안을 이동하고, Tab은 다음 빈 칸으로 갑니다. 빨간색 줄은 해석되지 않는 줄이며, 클릭하면 텍스트로 편집할 수 있습니다.
# Help under the formula in Text mode. "; starts a comment": the ; is what the user types.
formula-dialog-help-text = 입력하거나 키패드를 사용하십시오 — 키패드에는 수식 언어가 아는 모든 이름이 있습니다. 이름은 입력하는 대로 완성됩니다(Tab). 문장은 줄 바꿈이나 쉼표로 구분하며, ;는 주석을 시작합니다.
# The syntax check, when the formula reads.
formula-dialog-reads = 올바르게 해석됩니다.
# The syntax check in Textbook mode, when the formula has an empty box to fill in.
formula-dialog-empty-box = 빈 칸을 채우십시오.
# The syntax check in Textbook mode. $line: the line number; $message: what is wrong (from the formula parser, in English).
formula-dialog-error-line = { $line }행: { $message }
# One function written with another notation's name, and the language's name for it: "ln to log".
# $from: the name as written (ln); $to: the formula language's name (log).
formula-respell-pair = { $from } → { $to }
# A button that rewrites them. $changes: one or more formula-respell-pair, joined with ", ".
formula-respell = 변경: { $changes }
formula-respell-hover = 이 함수들을 수식 언어의 이름으로 바꿔 씁니다
# Shown when Apply finds a parameter field that is not a number. $name: the parameter (p1 … p5).
formula-param-not-numbers = { $name }은(는) 숫자 쌍이 아닙니다
# The Apply button (also in the Formula library window).
formula-apply = 적용
formula-dialog-apply-hover = 수식을 컴파일해 표시합니다
# Shown beside Apply while the formula is being compiled.
formula-dialog-compiling = GPU용으로 수식을 컴파일하는 중…
# The greyed hint in an empty name box (also in the Formula library window).
formula-name-hint = 이름(선택 사항)
# The save button: "Update" when the library already has a formula of that name.
formula-dialog-update-in-library = 라이브러리에서 업데이트
formula-dialog-save-to-library = 라이브러리에 저장
formula-dialog-save-hover = 이 수식과 그 매개변수를 수식 라이브러리에 보관합니다
formula-dialog-save-disabled-hover = 올바르게 해석되는 수식만 저장할 수 있습니다
# Opens the Formula library window.
formula-dialog-library = 라이브러리…
formula-dialog-library-hover = 저장된 수식: 적용, 편집, 가져오기, 내보내기

## The Custom formula dialog's keypad
## The keys' captions are formula tokens and stay as they are; these are their tooltips.

# The keypad's third tab (the other two are "123" and "f(z)"). A short word.
formula-keypad-tab-names = 이름
# The tabs' tooltips.
formula-keypad-tab-basic-hint = 숫자, 연산자, 가장 많이 쓰는 이름
formula-keypad-tab-functions-hint = 함수
formula-keypad-tab-names-hint = 모든 변수, 매개변수, 상수와 문장 구문
# The key z.
formula-key-z = 반복값
# The key c.
formula-key-c = 픽셀(Julia 모드에서는 Julia 상수)
# The key p1 on the first tab.
formula-key-p1-below = 매개변수 1 — 아래 입력란에서 설정
# The keys p1 … p5. $n: the parameter's number.
formula-key-param = 매개변수 { $n }
formula-key-multiply = 곱하기
formula-key-divide = 나누기
# The key that types ^2.
formula-key-square-after = 제곱(값 다음에: z^2)
# The key that types ^.
formula-key-power = 거듭제곱: 정수, 실수 또는 복소수 지수
formula-key-square-root = 제곱근
# The key that types |…|: in this language it is the squared modulus, as in Fractint.
formula-key-squared-modulus = 절댓값의 제곱 |z|² — Fractint에서처럼
formula-key-add = 더하기
formula-key-subtract = 빼기(또는 부호 반전)
# The key that types =.
formula-key-assign = 대입: z = …, 또는 임시 변수 t = …
# The key that starts a new line (first tab).
formula-key-next-statement-comma = 다음 문장(쉼표도 됨)
# The key "del".
formula-key-backspace = 커서 앞의 문자(또는 선택 영역) 삭제
# The key e.
formula-key-euler = 오일러 수
# The key that types (0.5, 0.5).
formula-key-complex-constant = 복소수 상수: (실수부, 허수부)
# The key that types a comma.
formula-key-comma = 문장, 또는 복소수 상수의 두 부분을 구분합니다
formula-key-left = 커서 왼쪽으로
formula-key-right = 커서 오른쪽으로
# The function keys (the f(z) tab), one per function.
formula-key-sin = 사인
formula-key-cos = 코사인
formula-key-tan = 탄젠트
formula-key-cotan = 코탄젠트
formula-key-exp = 지수 함수 e^z
formula-key-sinh = 쌍곡 사인
formula-key-cosh = 쌍곡 코사인
formula-key-tanh = 쌍곡 탄젠트
formula-key-cotanh = 쌍곡 코탄젠트
formula-key-log = 자연로그, ln(주가지)
formula-key-sqr = 제곱: sqr(z) = z²
formula-key-sqrt = 제곱근(주값)
formula-key-recip = 역수 1/z
formula-key-conj = 켤레 복소수
formula-key-flip = 실수부와 허수부 바꾸기
formula-key-abs = 두 부분 모두의 절댓값: |x| + i|y|
formula-key-cabs = 절댓값 |z|(실수)
formula-key-real = 실수부
formula-key-imag = 허수부(실수로)
# "fn" is Fractint's name for a function slot: keep it.
formula-key-ident = 항등 함수(Fractint의 fn 자리 표시자)
# The key "pixel".
formula-key-pixel = c와 같음(Fractint의 이름)
# The key "z =".
formula-key-set-z = z를 정하는 문장 시작
# The key "t =".
formula-key-temporary = 임시 변수: 새 이름이면 무엇이든 됩니다(t, w, a2, …)
# The key "; …".
formula-key-comment = 줄 끝까지 이어지는 주석
# The key that starts a new line (names tab).
formula-key-next-statement = 다음 문장

## The formula's completion list (names offered as you type)

# Beside a name the formula itself assigns.
formula-completion-variable = 이 수식이 정하는 변수
# Beside a function typed with another notation's name.
# $what: what the function is, in English (from the formula parser: "The natural logarithm");
# $other: the other notation's name (ln); $ours: the formula language's name (log).
formula-completion-other-spelling = { $what } — { $other }은(는) 여기서 { $ours }(으)로 씁니다
# Under a list cut short. $count: the names not shown.
formula-completion-more = … { $count }개 더
# The last line of the list. Tab, Enter and Esc are key names.
formula-completion-keys = Tab 또는 Enter로 완성, Esc로 닫기

## The Custom formula dialog's Textbook mode

# The tooltip of a line that does not read. $why: what is wrong (from the formula parser, in English, or formula-textbook-fractint-sections).
formula-textbook-unread-hover = 해석되지 않음: { $why }. 클릭하면 텍스트로 편집합니다.
# Why a line with Fractint's sections is shown as text in Textbook mode.
formula-textbook-fractint-sections = Fractint의 섹션(초기화의 ':', 비교식, a = b = …)은 텍스트로 편집합니다

## The Formula library window (Fractal ▸ Formula library…)

# The window's title.
formula-library-title = 수식 라이브러리
# The window's two tabs. $count: the formulas in each.
formula-library-mine = 내 수식({ $count })
formula-library-mine-hover = 사용자가 저장하거나 가져온 수식
formula-library-collection = 컬렉션({ $count })
formula-library-collection-hover = Fractadyne과 함께 제공되는 수식으로, 각각 시작할 뷰가 있습니다
# A small marker before the name of the formula the view shows now. A short word.
formula-library-showing = 표시 중
# The tooltip of Apply on a formula of the collection.
formula-library-apply-shipped-hover = 시작 뷰에서 표시합니다
# A row's Edit button (after a pencil icon).
formula-library-edit = 편집
formula-library-edit-hover = 적용하지 않고 사용자 정의 수식 대화 상자에서 엽니다
# A collection row's button that copies the formula into "My formulas".
formula-library-copy-to-mine = 내 수식으로 복사
formula-library-copy-to-mine-hover = 사용자의 수식에 추가해 보관하거나 수정합니다
formula-library-close = 닫기
# Saves the formula the view shows, under the name typed beside it.
formula-library-add-current = 현재 수식 추가
formula-library-add-current-hover = 뷰에 표시된 수식을 저장합니다
# "Fractal > Custom formula…" is the menu path: write it as the menus are named.
formula-library-add-current-disabled-hover = 먼저 사용자 정의 수식을 적용하십시오(프랙털 > 사용자 정의 수식…)
formula-library-import = 가져오기…
formula-library-import-hover = 수식 파일의 수식을 추가합니다. 여기 있는 것은 아무것도 바뀌지 않습니다
formula-library-export-all = 모두 내보내기…
formula-library-export-all-hover = 여기 있는 모든 수식을 파일 하나에 기록해 공유하거나 보관합니다
# The filter box: its label, and the greyed hint inside it.
formula-library-filter = 필터
formula-library-filter-hint = 이름이나 텍스트로 필터
# Shown while the user has no saved formulas.
formula-library-empty = 아직 저장된 수식이 없습니다. 사용자 정의 수식 대화 상자에서 저장하거나, 뷰에 표시된 수식을 추가하거나, 파일(이 프로그램의 파일 또는 Fractint의 .frm)을 가져오십시오.
# The tooltip of Apply on one of the user's formulas: with a view saved, and without one.
formula-library-apply-view-hover = 함께 저장된 뷰에서 이 수식을 표시합니다
formula-library-apply-hover = 이 수식 표시
# The tooltip of a row's save icon.
formula-library-export-one-hover = 이 수식을 파일로 내보내기
# A row asks before deleting: the question, then its two buttons. "Delete" is also the delete icon's tooltip.
formula-library-delete-ask = 삭제하시겠습니까?
formula-library-delete = 삭제
formula-library-keep = 유지
# Under a list cut short. $count: the formulas not listed.
formula-library-more = …그리고 { $count }개 더: 필터에 입력해 찾으십시오.
formula-library-no-match = 필터와 일치하는 것이 없습니다.
# Under a saved formula this version cannot read. $why: what is wrong (from the formula parser, in English).
formula-library-unread = 이 버전에서는 해석되지 않음: { $why }

## Formula library notices

# $why: the system's reason, in English.
formula-library-save-failed = 수식 라이브러리를 저장할 수 없습니다: { $why }
formula-library-nothing-to-save = 저장할 것이 없습니다: 수식이 비어 있습니다.
# $name: the formula's name in the library.
formula-library-updated = 수식 라이브러리의 “{ $name }”을(를) 업데이트했습니다.
formula-library-saved = “{ $name }”을(를) 수식 라이브러리에 저장했습니다.
# $name: the formula's name; $why: what is wrong.
formula-library-apply-failed = “{ $name }”을(를) 적용할 수 없습니다: { $why }
# Copying a formula of the collection into "My formulas". $name: its name; $as_named: the name it was given (the same, or "Name (2)" when the name was taken).
formula-library-copy-exists = “{ $name }”은(는) 이미 사용자의 수식에 있습니다.
formula-library-copied = “{ $name }”을(를) 사용자의 수식에 “{ $as_named }”(으)로 복사했습니다.
# $name: the formula's name.
formula-library-deleted = 수식 라이브러리에서 “{ $name }”을(를) 삭제했습니다.
# Why the library could not be saved (follows formula-library-save-failed).
formula-library-no-config-dir = 설정 디렉터리 없음
# At startup, when the library file cannot be read. $why: the reason, in English; $path: a file's full path.
formula-library-unreadable-kept = 수식 라이브러리를 읽을 수 없습니다({ $why }). 기존 파일은 { $path }(으)로 보관하고 새 빈 라이브러리를 시작했습니다.
formula-library-unreadable-stuck = 수식 라이브러리를 읽을 수 없고({ $why }), 다른 곳으로 옮길 수도 없습니다: 수식을 저장하면 이 파일이 대체되므로, 그 전에 { $path }을(를) 안전한 곳에 복사하십시오.

## Formula library: Import and Export
## A formula file's contents stay in English; this is what the windows and notices around them say.

# File-type names in the open and save dialogs (the extensions are added by the system).
formula-filter-formula-files = 수식 파일
formula-filter-fractadyne = Fractadyne 수식
formula-filter-fractint = Fractint 수식
formula-filter-all-files = 모든 파일
# Why a file cannot be imported (they follow formula-import-failed).
formula-import-too-large = 수식 파일이라기에는 너무 큼
formula-import-not-text = 텍스트 파일이 아님
formula-import-no-formulas = 수식이 없음
# $why: the TOML reader's message, in English.
formula-file-not-formulas = 수식 파일이 아님({ $why })
# $format: what the file says it is ("fractadyne-gradients").
formula-file-other-format = 수식 파일이 아니라 “{ $format }” 파일임
# "[[formula]]" is the file's own syntax: keep it.
formula-file-no-formulas = 수식이 없음(수식 파일은 수식을 [[formula]] 표로 나열합니다)
# $file: the file's name; $why: one of the reasons above, or the system's (in English).
formula-import-failed = “{ $file }”을(를) 가져올 수 없습니다: { $why }.
# After an import that added nothing. $file: the file's name.
formula-import-nothing-new = “{ $file }”에 새로운 것이 없습니다.
# After an import. $count: the formulas added; $file: the file's name.
formula-import-added = “{ $file }”에서 수식 { $count }개를 가져왔습니다.
# The same two, followed by what else happened. $notes: one or more of the formula-import-note-… messages below, joined with "; ".
formula-import-nothing-new-notes = “{ $file }”에 새로운 것이 없습니다: { $notes }.
formula-import-added-notes = “{ $file }”에서 수식 { $count }개를 가져왔습니다: { $notes }.
# Notes (clauses, no final full stop). $count: the formulas already in the library, skipped.
formula-import-note-duplicates = 수식 { $count }개가 이미 라이브러리에 있었음
# One formula was renamed because its name was taken. $from: its name in the file; $to: the name it was given ("Name (2)").
formula-import-note-renamed-one = “{ $from }”은(는) 이미 쓰인 이름이라 “{ $to }”(으)로 바뀜
# $count: the formulas renamed because their names were taken (2 or more).
formula-import-note-renamed = 이름이 이미 쓰여 { $count }개의 이름을 바꿈
# $count: the entries skipped as empty or too long.
formula-import-note-skipped = 항목 { $count }개가 비어 있거나 너무 길어 건너뜀
# A second sentence after importing a Fractint .frm file. $count: the entries that do not read; $why: the commonest reason (from the formula parser, in English).
formula-import-frm-unread = 항목 { $count }개는 이 버전에서 해석되지 않습니다(가장 흔한 이유: { $why }).
# After exporting one formula. $name: the formula's name; $file: the file's name.
formula-export-one = “{ $name }”을(를) “{ $file }”(으)로 내보냈습니다.
# After exporting several. $count: how many; $file: the file's name.
formula-export-many = 수식 { $count }개를 “{ $file }”(으)로 내보냈습니다.
# $file: the file's name; $why: the system's reason, in English.
formula-export-failed = “{ $file }”에 쓸 수 없습니다: { $why }

## The control panel (right-hand side): header and section headings

# The panel's title, beside the button that hides it.
panel-title = 컨트롤
panel-hide-hover = 컨트롤 패널 숨기기
# The button that brings the hidden panel back (top right of the view).
panel-show-hover = 컨트롤 패널 표시
# Section headings.
panel-section-navigate = 탐색
panel-section-coloring = 채색
panel-section-quality = 품질
panel-section-effects = 효과
panel-section-overlays = 오버레이
panel-section-advanced = 고급
# The last section: reference notes about the current fractal. $name: the fractal's name (Mandelbrot, Burning Ship…).
panel-section-about = { $name } 정보
panel-section-performance = 성능

## The control panel: Navigate
## Labels before a control sit in a column about 104 px wide (roughly 15 characters): a longer label pushes its control to the right.

panel-zoom-speed = 줌 속도
panel-zoom-speed-hover = Space를 누르고 있을 때의 연속 줌 속도(1× ≈ 1.5 s마다 2×).
panel-snap-zoom = 특징점에 줌 맞추기
panel-snap-zoom-hover = 마우스 휠이나 Space로 확대하는 동안, 커서 가까이의 미니브로트(또는 실수축) 쪽으로 줌을 끌어당겨, 그것이 커서 아래로 미끄러져 들어오고 줌이 그 위에 내려앉게 합니다. 특징점이 가까울수록 더 강하게 끌어당깁니다. Mandelbrot만, 단일 뷰.
panel-click-zoom = 클릭하여 줌
panel-click-zoom-hover = 켜면 뷰에서 왼쪽 클릭할 때 아래의 배수만큼 다이브하며(오른쪽 클릭은 되돌아 나옴), 클릭한 점이 중심이 됩니다. 드래그는 그대로 이동이고, Shift+드래그 / 오른쪽 드래그는 그대로 영역 줌입니다. Backspace는 클릭 한 번을 되돌립니다. 이중 뷰에서는 어느 패널에서나 동작하며, 이 기능이 켜져 있는 동안 Julia c 고정은 Ctrl+클릭으로 옮겨집니다. Shift를 누르고 있으면 클릭했을 때 도착할 뷰를 보여 주는 돋보기가 커서 아래에 나타나며, 누른 채로 클릭하면 그 점을 택합니다. Shift+드래그는 그대로 영역 줌입니다.
# The click-to-zoom factor, followed by buttons 2×, 4×, 10×…
panel-click-zoom-factor = 배수
panel-click-zoom-snap = 가장 가까운 중심에 맞추기
panel-click-zoom-snap-hover = 이동한 뒤 가장 가까운 미니브로트 핵을 구해 정확히 그 위에 내려앉으므로, 특징점 근처가 아니라 특징점 위에 도착합니다. 줌은 바뀌지 않습니다 — 그것은 클릭이 정했습니다. Mandelbrot만. 계산은 백그라운드에서 실행됩니다: 뷰는 즉시 이동하고 답이 도착하면 스스로 바로잡으며, 극한 깊이에서는 몇 초가 걸릴 수 있습니다.

## The control panel: Coloring
## Labels before a control sit in a column about 104 px wide (roughly 15 characters).

panel-method = 방식
panel-method-hover = 탈출 데이터를 색으로 바꾸는 방법입니다. 스트라이프 / 삼각 부등식 / 궤도 트랩 / 분해는 궤도 구조를 드러내고, 거리 추정은 경계와의 가까움에 따라 음영을 줍니다.
panel-stripe-density = 스트라이프 밀도
panel-stripe-tail = 꼬리만(딥 줌)
panel-stripe-tail-hover = 스트라이프 항을 궤도 전체가 아니라 각 궤도의 마지막 N회 반복(지수 창)에 대해 평균합니다. 극한 깊이에서는 모든 궤도가 마지막 수백 회를 제외하면 기준 궤도를 그림자처럼 따라가므로, 궤도 전체의 평균은 뷰 전체에서 한 색이 됩니다. 꼬리는 대비를 유지합니다. 창보다 짧은 궤도는 영향을 받지 않습니다. --render에서는 --stripe-tail N으로도 쓸 수 있습니다.
panel-tail-length = 꼬리 길이
# The unit after the tail length's number: short for "iterations" (the slider reads "64 it").
panel-tail-length-unit = 회
panel-tail-length-hover = 반복 횟수로 나타낸 창 길이입니다. 짧은 창(8–32)은 어느 깊이에서나 고전적인 굵은 띠를 만들고, 긴 창은 다시 평평한 색으로 평균되는데, 길고 혼돈적인 궤도의 스트라이프 항은 평균하면 상쇄되기 때문입니다.
panel-trap-shape = 트랩 모양
panel-palette = 팔레트
# The two colours of the Binary palette, each beside its colour swatch.
panel-in-set = 집합 안
panel-out-of-set = 집합 밖
# The two colours of the Duotone palette, each beside its colour swatch.
panel-shadow = 그림자
panel-highlight = 하이라이트
panel-cycle = 순환
panel-offset = 오프셋
panel-normalize-hover = 극한 깊이에서는 탈출 횟수가 수십만에 걸쳐 있어서, 고정된 순환은 이웃한 픽셀 사이에서 팔레트를 수천 번 감습니다 — 올바르고 조밀한 필드가 반점 노이즈처럼 보입니다. 이 옵션은 뷰가 실제로 앨리어싱될 때에만 팔레트를 뷰에서 측정한 탈출 범위에 다시 매핑합니다(그러면 순환은 그 범위에 팔레트가 몇 번 펼쳐질지를 정합니다) — 부드러운 뷰는 그대로 두는데, 다시 매핑하면 아무 이유 없이 그림이 바뀌기 때문입니다. 상관없이 다시 매핑하려면 아래의 ‘항상 범위에 맞춤’을 선택하십시오. 부드러운 반복 방식에만 해당하며, --normalize와 같습니다.
# A checkbox indented under "Normalize deep colors" (also in the Color menu).
panel-fit-range = 항상 범위에 맞춤
panel-fit-range-hover =
    뷰가 앨리어싱되지 않을 때에도 팔레트를 뷰에서 측정한 탈출 범위에 다시 매핑합니다 — 위의 확인란은 조밀한 필드가 반점처럼 보일 때만 다시 매핑합니다.

    노이즈를 고치기보다 화면에 있는 것에 팔레트를 펼치고 싶을 때 사용하십시오. 측정값이 필요하므로, 뷰가 안정되기 전 잠깐 동안은 아무 효과가 없고, 탈출한 픽셀이 없는 뷰에서는 전혀 효과가 없습니다.
# A checkbox indented under "Always fit range" (also in the Color menu).
panel-log-scale = 로그 색상 척도
panel-log-scale-hover =
    탈출 값을 선형이 아니라 로그로 팔레트에 펼칩니다. 깊은 곳에서는 탈출 횟수가 높은 쪽으로 몰리므로, 선형 매핑은 팔레트의 대부분을 경계 근처의 얇은 껍질에 쓰고 나머지를 모두 평평하게 만듭니다.

    정규화된 매핑에 적용됩니다 — ‘깊은 색상 정규화’(또는 내보내기의 --normalize)가 켜는 매핑입니다. 범위가 심하게 치우치면 이 확인란과 상관없이 로그 매핑을 사용합니다.
# The same checkbox's tooltip while it is greyed out (normalization is off).
panel-log-scale-off-hover =
    정규화가 필요합니다: 이 옵션은 측정된 탈출 범위에 팔레트를 어떻게 매핑할지 정하므로, ‘깊은 색상 정규화’가 꺼져 있으면(그리고 --normalize 내보내기가 아니면) 바꿀 매핑이 없어 픽셀 하나도 바뀌지 않습니다.

    사용하려면 ‘깊은 색상 정규화’를 켜십시오.
# The palette animation: its kind, then its speed.
panel-animate = 애니메이션
panel-anim-speed = 속도
panel-anim-speed-hover = 순환 속도: 초당 색 오프셋 순환 횟수, 또는 (무작위) 초당 그라데이션 변경 횟수.
panel-shuffle-gradient = 그라데이션 섞기

## The control panel: Quality

panel-auto-iter = 줌에 따라 반복 횟수 자동 조정
# The iteration-count label while auto-scaling is on (the count is then a base the zoom raises).
panel-iterations-base = 반복 횟수(기본)
panel-iterations = 반복 횟수
panel-iterations-hover = 기본 반복 횟수입니다. 자동 조정을 켜면 실제 횟수는 줌 깊이에 따라 올라갑니다. 움직이는 동안 미리 보기는 반응성을 위해 반복 횟수를 낮게(50,000) 제한했다가, 뷰가 안정되면 전체 횟수로 선명해집니다 — 그래서 깊은 경계가 움직이는 동안에는 부드럽게 보이고 멈추면 또렷해집니다.
# A warning under the iteration count. $count: the iterations the settled preview renders.
panel-res-limited = 이 깊이에서는 반응성을 유지하기 위해 안정된 미리 보기를 낮은 해상도에서 { $count }회 반복으로 렌더링합니다. 최대 해상도의 디테일은 내보내기를 사용하십시오.
panel-anti-alias = 안티앨리어싱
# Anti-aliasing off (the other choices are 2×, 3×, 4×, 8×).
panel-anti-alias-off = 끄기
panel-anti-alias-hover = 정지 이미지용 슈퍼샘플링입니다(뷰가 안정될 때 적용). 높을수록 섬세한 외부 ‘먼지’를 줄이지만 렌더링 시간이 늘어납니다.

## The control panel: Effects

panel-relief-lighting = 3D 입체 조명
panel-relief-lighting-hover = 거리 추정의 기울기로 표면에 음영을 줍니다 — 돋을새김에 빛을 비춘 모습. (정칙 계열: Mandelbrot / Multibrot.)
panel-light-angle = 조명 각도
panel-relief = 입체감
panel-relief-hover = 낮을수록 날카로운 입체감, 높을수록 부드럽고 평평함.
panel-rotate-light = 조명 회전
panel-rotate-light-hover = 시간에 따라 조명 방향을 돌립니다(속도 슬라이더 사용).
panel-distance-glow = 거리 글로우
panel-distance-glow-hover = 경계 근처에서 빛나는 가는 실처럼 조밀해지는 밝은 거리 추정 등고선 띠. (정칙 계열.)
panel-glow = 글로우
panel-band-width = 띠 너비
panel-band-width-hover = 거리 등고선의 간격(띠당 옥타브).
panel-animate-glow = 글로우 애니메이션
panel-animate-glow-hover = 시간에 따라 글로우 띠를 흐르게 합니다(속도 슬라이더 사용).

## The control panel: Overlays

panel-orbits-hover = 커서 아래 점의 반복 경로를 그립니다.
panel-orbit-anim = 애니메이션(달리는 점)
panel-orbit-anim-hover = 색이 순환하는 점을 궤도를 따라 달려가게 합니다.
panel-orbit-speed = 궤도 속도
panel-minimap-hover = “현재 위치” 표시와 줌 깊이가 나타나는 작은 홈 뷰 개요입니다. 클릭하면 해당 영역으로 이동합니다.

## The control panel: Advanced
## Labels before a control sit in a column about 104 px wide; most of these are already wider in English.

# Small headings of the section's two groups.
panel-accelerators = 가속
panel-performance-tuning = 성능 조정
panel-bla = BLA 가속(딥 줌)
panel-bla-hover = 쌍선형 근사: 극한 깊이에서 궤도 전체에 걸쳐 반복을 건너뜁니다(floatexp Mandelbrot, ≥1e28×) — GPU 렌더링이 약 5배 빨라지며, 출력은 동일합니다(자체 테스트로 검증). 기본값은 켜짐입니다. 비교하거나 결함을 발견하면 끄십시오.
panel-series-approx = 급수 근사
panel-series-approx-hover = 다항식으로 섭동의 초깃값을 정해, BLA가 동작하지 않는 곳(df32 깊이, Multibrot, BLA 꺼짐)에서 초기 반복을 건너뜁니다. 출력은 동일하며, 비교하려면 끄십시오.
panel-glitch-correct = 글리치 보정(내보내기)
panel-glitch-correct-hover = 내보낸 이미지에 대한 다중 기준점 글리치 보정: 섭동 글리치를 감지하고 그 픽셀을 깨끗해질 때까지 추가 기준점으로 다시 렌더링합니다. 기본값은 켜짐입니다. 약 32 MP까지의 내보내기에 적용되며(보조 채색이 아닌 경우), 더 큰 이미지와 라이브 뷰는 일반 경로를 사용합니다. Mandelbrot나 Multibrot 뷰(Julia 모드 밖)에는 사용하지 않습니다: 정확한 계산과 비교해 보니 그곳에서는 아무것도 고치지 못하면서 내보내기 시간이 최대 5배 늘었습니다.
panel-dive-limit = 자동 줌 다이브 한도
panel-dive-limit-hover = 자동 줌(A 키)이 멈추는 깊이입니다. 약 1e271×까지는 부드럽게 미끄러지듯 들어가고, 그보다 깊으면 극한 깊이에 빨리 닿도록 끊기는 단계식 다이브로 바뀝니다.
# Followed by two choices, Speed and Quality.
panel-autopilot-priority = 자동 줌 우선순위
panel-priority-speed-hover = 설정한 속도로 줌합니다. 디테일이 있는 프레임이 준비되는 즉시 화면을 갱신하며, 갱신이 계속 이어지도록 어떤 해상도든 사용합니다(줌이 렌더링을 앞지르면 최소 이동 해상도 아래로도). 갱신 사이에는 마지막 프레임이 줌을 따라가며 흐려집니다.
panel-priority-quality-hover = 모든 갱신이 완전히 렌더링된 최대 해상도 프레임이며, 다음 프레임이 도착하기 전에 화면의 프레임이 지나치게 확대되지 않도록 필요한 만큼 줌이 느려집니다. 더 느리지만 매 순간 선명한 다이브입니다.
# Followed by two choices, Detail and Misiurewicz point.
panel-autopilot-target = 자동 줌 목표
panel-target-detail-hover = 뷰에 있는 프랙털 경계에서 가장 풍부한 구조를 향해 다이브하며, 다이브하는 동안 목표를 다시 고릅니다.
panel-target-misiurewicz-hover = 가장 가까운 Misiurewicz 점(구조가 영원히 반복되는 나선의 중심)을 찾아 정확히 그 점으로 다이브합니다. 다이브는 먼저 그 점을 다이브 한도의 깊이까지 구하는 동안 멈춰 있습니다 — 보통 깊이에서는 잠깐이고, 1e1000×를 넘으면 더 깁니다. Mandelbrot 집합만.
panel-work-budget = 라이브 렌더 예산
panel-work-budget-hover = 딥 줌에서 라이브 뷰의 디테일과 속도 사이의 균형입니다. 높을수록 더 높은 해상도로 렌더링하지만(더 선명하고, 업스케일된 “흐린” 느낌이 덜함) 프레임 속도가 낮아지며, 아주 높은 값은 가장 무거운 프레임에서 잠깐 GPU가 멈출 위험이 있습니다. 내보내기는 이 설정과 상관없이 항상 최대 해상도입니다.
panel-min-motion-res = 최소 이동 해상도
panel-min-motion-res-hover = 딥 줌에서 연속으로 줌하거나 이동하는 동안 라이브 뷰가 떨어질 수 있는 최저 해상도입니다. 높을수록 빠른 다이브가 블록처럼 깨지는 것을 막지만(더 선명) 프레임 속도가 낮아집니다 — 100%에서는 딥 다이브의 디테일 갱신을 전체 비용으로 렌더링해야 하므로, 딥 줌이 눈에 띄게 뚝뚝 끊길 수 있습니다. 딥 다이브가 덜컹거리면 이 값을 낮추십시오. 기본값 30%. 안정된 이미지나 내보내기에는 영향을 주지 않습니다.
panel-finish-sound = 렌더링이 끝나면 소리 재생
panel-finish-sound-hover = 내보내기나 투어 렌더링이 끝나면 FRACTINT의 완료 음악을 재생합니다: 올라가는 세 음(1047, 1109, 1175 Hz, 각 100 ms)으로, DOS 원본에 인코딩된 그대로이며 — 대역 제한 사각파로 합성한 뒤 고역 통과 필터를 거쳐, 그것을 연주하던 작은 PC 스피커 콘 같은 소리를 냅니다.
panel-prefer-detail = 줌하는 동안 디테일 우선
panel-prefer-detail-hover = 줌하거나 이동하는 동안, 매 프레임 낮은 품질로 다시 렌더링하는 대신 마지막으로 완전히 렌더링된 프레임을 — 움직임에 맞춰 크기를 바꾸고 따라가게 하며 — 계속 보여 줍니다. 움직임이 멈추면 뷰를 완전히 렌더링합니다. 끄면: 움직이는 동안 낮은 해상도로 실시간 렌더링합니다(더 부드럽지만 더 거침). 얕은 뷰(직접 모드)는 항상 실시간으로 렌더링합니다 — 어느 쪽이든 매 프레임 저렴하고 선명합니다.

## The control panel: About (the current fractal)

# A link to the fractal's reference page, followed by an arrow.
panel-about-reference = 참고 자료

## The control panel: Advanced ▸ Second graphics card

# A label before a drop-down (wider than the panel's label column already).
gpu2-label = 두 번째 그래픽 카드
gpu2-off = 끄기
# Another device on the same graphics card as the window (the default).
gpu2-same-card = 같은 카드
# A card the list does not name. $number: the card's number.
gpu2-card-n = 카드 { $number }
gpu2-finding = 이 컴퓨터의 그래픽 카드를 찾는 중…
gpu2-none = 이 컴퓨터에는 다른 그래픽 카드가 없습니다.
gpu2-hover =
    다른 그래픽 장치가, 깊은 뷰가 움직이는 동안의 프레임과 멈춘 뒤 뷰를 선명하게 하는 추가 샘플 일부를 렌더링하므로, 그림이 줌을 따라가고 최종 품질에 더 빨리 도달합니다. 창을 그리는 카드는 여전히 모든 것을 그립니다.

    다른 카드는 자신의 속도를 더해 줍니다. 서로 다른 두 카드 모델은 몇몇 픽셀을 약간 다르게 그리며, 두 번째 카드가 실패하면 뷰는 카드 하나로 계속됩니다.

    “같은 카드”(기본값)는 이 카드에 두 번째 장치를 엽니다. 속도는 더해지지 않지만 화면의 프레임을 기다리지 않고 렌더링합니다. 카드를 공유하므로, 카드를 재설정하는 실패는 창 자체의 렌더링 실패와 마찬가지로 앱을 종료시키며, 다음 시작 때 이 기능을 끌지 묻습니다.

## The view (canvas)

# A notice after Ctrl+click pins the Julia c at a deep zoom.
canvas-julia-pin-quantized = Julia c를 고정했습니다 — 하지만 배정밀도로 보관되므로, 이 깊이에서는 패널 전체가 하나의 값으로 반올림되어 고정점이 뷰 중심에 놓입니다. 조준하려면 축소하십시오.
# A small caption in the top-left corner of the view while the orbit overlay is fitted to the view (its "Normalize (fit to view)" option).
canvas-orbit-normalized = 궤도(정규화됨)
# The minimap's tooltip.
canvas-minimap-hover = 개요 / 현재 위치. 드래그하면 현재 줌에서 이동하고, 스크롤하면 확대/축소하며, 클릭하면 그 영역으로 이동합니다(홈 줌).

## The timestamp overlay (View ▸ Show timestamp)

# Under the large elapsed-time clock. $frame: the frame number (as the log writes it, without grouping).
overlay-timestamp-frame = 프레임 { $frame }

## File ▸ Settings ▸ Reference cache…

cache-title = 기준 궤도 캐시
cache-intro = 극한 깊이에서 뷰의 느린 부분은 기준 궤도입니다 — 최대 한 시간에 이르는 임의 정밀도 산술입니다. Fractadyne은 완성된 궤도를 여기에 보관하므로, 방문했던 위치로 돌아가거나 그 근처로 줌할 때 대신 몇 초면 됩니다. 궤도 하나는 약 4 MB입니다.
cache-off = 이번 실행에서는 꺼져 있습니다(작업 호출 또는 --no-orbit-cache). 다음 일반 실행 전까지 아무것도 읽거나 쓰지 않습니다.
# Small headings of the window's parts.
cache-location = 위치
cache-usage = 사용량
cache-limit = 한도
cache-no-config = (사용할 수 있는 설정 디렉터리 없음)
cache-open-folder = 폴더 열기
# The text on the usage bar. $used and $limit: sizes such as "1.2 GB"; $count: the orbits kept.
cache-usage-bar = { $limit } 중 { $used } — 궤도 { $count }개
cache-limit-hover = 캐시가 이 크기를 넘으면 계산 비용이 가장 적게 든 궤도부터 제거합니다 — 한 시간 걸린 깊은 궤도는, 빨리 만든 궤도가 아무리 많고 아무리 최근에 쓰였더라도 그것들보다 오래 남습니다.
# Beside the limit. $count: about how many orbits of the live view's length fit in it.
cache-limit-estimate = ≈ 라이브 상한 길이의 궤도 { $count }개
cache-kept = 보관된 항목(가치가 큰 순)
# Column headers of the list of kept orbits.
cache-col-iterations = 반복 횟수
cache-col-precision = 정밀도
cache-col-size = 크기
cache-col-last-used = 마지막 사용
# The precision column. $bits: the precision in bits.
cache-row-bits = { $bits }비트
# Under the list. $count: the orbits not listed.
cache-rows-more = … 그리고 { $count }개 더
# When an orbit was last used (the "last used" column).
cache-ago-now = 방금
# $count: minutes.
cache-ago-minutes = { $count }분 전
# $count: hours.
cache-ago-hours = { $count }시간 전
# $count: days.
cache-ago-days = { $count }일 전
# Asked before clearing. $count: the orbits; $size: their total size, such as "1.2 GB".
cache-confirm = 궤도 { $count }개({ $size })를 삭제하시겠습니까? 다시 만드는 데는 데이터가 아니라 시간이 듭니다.
cache-delete = 삭제
cache-cancel = 취소
cache-clear = 캐시 비우기…
cache-clear-hover = 저장된 모든 궤도를 삭제합니다. 먼저 확인합니다.
cache-close = 닫기
# $count: the orbits removed.
cache-cleared = 기준 궤도 캐시를 비웠습니다 — 궤도 { $count }개를 제거했습니다.
# $why: the system's error message.
cache-clear-failed = 기준 궤도 캐시를 비울 수 없습니다: { $why }

## Help ▸ Diagnostics…

diag-title = 진단
diag-intro = 테스트를 실행해 Fractadyne이 사용자의 하드웨어에서 올바르게 작동하는지 확인합니다. 결과는 이 컴퓨터에 저장되며 문제 보고에 첨부할 수 있습니다 — 저절로 어디로 보내지는 것은 없습니다.
# $status: which arbitrary-precision library is in use, as the program reports it (in English).
diag-arithmetic = 딥 줌 산술: { $status }
# The three tests' buttons, each followed by a sentence saying what it does.
diag-run-selftest = 자체 테스트 실행
diag-run-uitest = UI 테스트 실행
diag-run-gputest = GPU 산술 검사 실행
diag-blurb-selftest = GPU에서 수학과 렌더링을 알려진 정답과 비교해 확인합니다. 약 15초 걸립니다.
diag-blurb-uitest = 인터페이스와 라이브 뷰를 둘러보며 여러 줌 깊이에서 스크린샷을 찍습니다. 1~2분 걸리며, 실행되는 동안 창이 열립니다.
diag-blurb-gputest = 컴퓨터가 제공하는 모든 그래픽 백엔드에서 셰이더의 double-float 산술을 알려진 정답과 비교해 확인합니다. 몇 초 걸리며, 창을 열지 않습니다. 실패를 보고하면 그것은 저희가 꼭 보고 싶은 실제 발견입니다 — 보고서에 첨부해 주십시오.
# While a test runs. $count: the checks finished so far.
diag-checks-done = 검사 { $count }개 완료
# A finished test's result. $test: the test's name (Self-test, UI test, GPU arithmetic check).
diag-verdict-captured = { $test } — 결과 기록됨
diag-verdict-passed = { $test } — 통과
diag-verdict-problems = { $test } — 문제 보고됨
# Under the GPU arithmetic check's result.
diag-informational-note = 이 검사에는 통과나 실패가 없습니다: 사용자의 GPU가 어떻게 계산하는지 기록하며, 보고된 실패는 사용자 컴퓨터의 결함이 아니라 실제 발견입니다. 어느 쪽이든, 보고서에 첨부하는 것이 이 결과로 할 수 있는 가장 유용한 일입니다.
# Under a test's result when it reported problems.
diag-problems-note = 기준 GPU가 아닌 하드웨어에서는 어느 정도 차이가 예상됩니다 — 특히 이미지 비교에서. 문제 보고에 첨부하는 것이 이 결과로 할 수 있는 가장 유용한 일입니다.
diag-open-results = 결과 열기
diag-attach = 문제 보고에 첨부…
diag-attach-hover = 이 결과를 포함해 보고 대화 상자를 열므로, 이슈에 설명만이 아니라 기계가 검증한 테스트가 담깁니다
diag-console = 진단 정보를 콘솔에 출력
diag-console-hover = 지금부터 [fd-…] 줄을 stderr에 표시합니다. Fractadyne을 인수 없이 실행하면 기본값은 꺼짐입니다. 기록되는 내용은 전혀 바뀌지 않으므로 — 로그 파일에는 항상 모든 줄이 기록됩니다 — 터미널을 지켜보는 경우가 아니면 꺼 두십시오. 이번 세션에만 유지됩니다. 시작 과정도 잡으려면 --console 플래그나 FRACTADYNE_CONSOLE=1을 사용하십시오.
diag-close = 닫기
# Errors starting a test. $why: the system's error message; $dir: a folder.
diag-error-no-exe = 실행 파일을 찾을 수 없습니다: { $why }
diag-error-create-dir = { $dir }을(를) 만들 수 없습니다: { $why }
diag-error-start = 테스트를 시작할 수 없습니다: { $why }

## Navigate ▸ Misiurewicz explorer…

misi-title = Misiurewicz 탐색기
misi-mandelbrot-only = Misiurewicz 둘러보기는 Mandelbrot 전용입니다 — 사용하려면 프랙털을 바꾸십시오.
# While the gallery is being built. $seconds: the time so far, such as "3".
misi-solving = 작은 유형 계열을 구하는 중({ $seconds }s)…
# Above the gallery. $total: all points; $curated: the hand-picked ones; $found: the ones the sweep found.
misi-summary = 점 { $total }개 — 선별한 것 { $curated }개, 작은 유형에 대한 시드 기반 뉴턴 탐색으로 찾은 것 { $found }개(위쪽 반평면, 거친 격자는 대부분의 끌림 영역을 찾지만 전부를 찾는다고 증명할 수는 없음).
# The name of a point the sweep found. $kind: its type, such as "(3,1)"; $point: its coordinate, such as "-0.1011+0.9563i".
misi-entry-name = { $point }의 { $kind }
# Under each thumbnail. $kind: the point's type, such as "(3,1)"; $octaves: the zoom period in octaves, such as "3.17"; $twist: the twist per repeat in degrees, such as "+12.5". "oct" is short for octaves.
misi-entry-readout = { $kind } · { $octaves } 옥타브/반복 · { $twist }°
# Before the depth field.
misi-zoom-to = 목표 배율
misi-cancel = 취소
misi-solve-jump = 구하고 이동
misi-solve-jump-hover = 저장된 좌표를 시드로 삼아, 요청한 깊이에 필요한 정밀도로 이 정확한 점을 뉴턴 방법으로 구한 뒤 곧바로 그곳으로 이동합니다. 깊이 요청할수록 오래 걸립니다 — 계산이 임의 정밀도이기 때문입니다.
misi-select-hint = 점을 고르고, 깊이를 정하고, 이동하십시오 — 각 썸네일 아래의 승수 줄은 그곳으로 다이브할 때의 모습입니다: 뷰가 반복되기까지 몇 옥타브인지, 반복마다 얼마나 비틀리는지.
misi-cancelled = 계산을 취소했습니다.
# $text: what was typed in the depth field.
misi-bad-zoom = “{ $text }”을(를) 배율로 읽을 수 없습니다(1e50, 3.2e120, … 처럼 쓰십시오).
misi-no-kp = 이 항목에는 계산에 쓸 감지된 (k,p)가 없습니다.
misi-parse-failed = 저장된 좌표를 해석하지 못했습니다.
# A notice after the jump. $name: the point's name; $zoom: the magnification, such as "1e50"; $seconds: how long the solve took, such as "1.2".
misi-zoomed = { $name }(으)로 줌했습니다 — { $zoom }× ({ $seconds }s)
# Added after the notice above when the iteration count is too low. $have: the fixed iteration count; $typical: what the depth typically needs.
misi-iter-note = 반복 횟수가 { $have }회로 고정되어 있지만 이 깊이에는 보통 약 { $typical }회가 필요합니다 — 뷰가 평평하게 렌더링되면 값을 올리십시오(또는 자동 반복 횟수를 켜십시오).
# $name: the point's name; $why: why the solve failed (a technical word).
misi-solve-missed = 깊은 계산이 { $name }에 도달하지 못했습니다: { $why }. 저장된 점은 자신의 깊이에서는 여전히 정확합니다 — 더 얕은 목표를 시도하십시오.

## Life (Game of Life): the side panel

# Shown when the graphics card cannot run the Life view.
life-no-compute = 이 그래픽 어댑터에서는 Life를 실행할 수 없습니다(컴퓨트 셰이더 없음).
# Row label: the pattern list (pattern names stay in English).
life-pattern = 패턴
life-open = 열기…
life-open-hover = RLE, .cells 또는 Life 1.05/1.06 패턴 파일을 엽니다.
life-pattern-text = 패턴 텍스트…
life-pattern-text-hover = 패턴을 붙여 넣습니다(LifeWiki가 제공하는 RLE 형식 또는 일반 텍스트).
life-save = 저장…
life-save-hover = 지금 상태의 세계를 RLE로 저장합니다.
# Row label: the rule list and the typed rule (B3/S23 notation).
life-rule = 규칙
# Shown in the rule list when the rule in use is none of the library's.
life-rule-custom = 사용자 정의
life-apply = 적용
# Playback buttons (a pause or play symbol is shown before them).
life-pause = 일시 정지
life-play = 재생
life-play-hover = 세계를 실행합니다(Enter).
life-step = 단계
life-step-hover = 한 세대 진행합니다(N).
# Tooltip of the button beside Step, which reads "+16", "+1,024"…: that many generations.
life-stride-hover = 이만큼의 세대를 진행합니다(Shift+N).
life-reset = 초기화
life-reset-hover = 처음 연 패턴으로 되돌립니다(0세대).
life-speed = 속도
# After the speed slider's number: generations a second. Short.
life-speed-unit = 세대/s
# Row label: how many generations the "+n" button advances.
life-stride = 보폭
# The panel's numbers.
life-generation = 세대
life-population = 개체 수
life-tiles = 타일
# $count: the tiles stored; $capacity: how many the GPU pool holds.
life-tiles-of = { $capacity }개 중 { $count }개
life-tiles-hover = 저장된 64×64셀 타일(각 4 KiB)과 GPU 풀의 용량.
# Row label: what a left-drag in the view does (Pan or Draw).
life-mouse = 마우스
life-pan = 이동
life-pan-hover = 드래그해 뷰를 움직입니다.
life-draw = 그리기
# The Draw tool's tooltip; the toolbar's shows it after "Draw (D):", so it starts lowercase.
life-draw-hint = 셀을 클릭하면 뒤집히고, 드래그하면 칠합니다 — 죽은 셀에서 시작한 획은 그리고, 산 셀에서 시작한 획은 지웁니다. 실행 중인 세계는 그리는 동안 멈추고, 손을 떼면 다시 실행됩니다.
life-undo = 실행 취소
life-undo-hover = 마지막 편집, 채우기, 지우기 또는 규칙 변경 이전의 세계로 되돌립니다(그리는 중에는 Ctrl+Z).
life-grid = 격자선
life-grid-hover = 셀당 8픽셀 이상으로 확대했을 때 셀 경계를 표시합니다.
# A random fill: "Random" labels the density field, "seed" the seed field after it.
life-random = 무작위
life-density-hover = 채우기가 살아 있게 만드는 셀의 비율입니다.
life-seed = 시드
life-fill-view = 뷰 채우기
# $cells: the largest side of the filled area, in cells (2,048).
life-fill-view-hover = 보이는 영역(중심 주위로 최대 { $cells } × { $cells } 셀)을 무작위로 채웁니다.
life-clear = 지우기
life-clear-hover = 모든 셀을 죽입니다(실행 취소로 되돌릴 수 있음).

## Life: notices in the panel, and file dialogs

# $path: the file written.
life-saved = 저장했습니다: { $path }
# $error: why (from the system, in English).
life-save-failed = 저장 실패: { $error }
# The universe stopped running. $error: why (in English).
life-stopped = 멈춤: { $error }
# A pattern file named a rule that does not read. $error: why (in English); $rule: the rule kept, in B3/S23 notation.
life-rule-not-understood = 패턴의 규칙을 이해하지 못했습니다({ $error }). { $rule }을(를) 유지했습니다
# $path: the file; $error: why (in English).
life-open-failed = { $path }을(를) 열 수 없습니다: { $error }
# File-dialog filter names.
life-filter-patterns = Life 패턴
life-filter-rle = 런 렝스 인코딩 패턴

## Life: the toolbar (its buttons are icons; these are their tooltips)

life-toolbar-pause-hover = 세계 일시 정지(Enter)
life-toolbar-play-hover = 세계 실행(Enter)
# $stride: the generations a Shift+click steps.
life-toolbar-step-hover = 한 세대 진행(N). Shift+클릭 또는 Shift+N은 { $stride }세대 진행
life-toolbar-reset-hover = 처음 연 패턴으로 되돌리기(0세대)
# $hint: the Draw tool's tooltip (life-draw-hint).
life-toolbar-draw-hover = 그리기(D): { $hint }

## Life: the Pattern text window

life-text-title = 패턴 텍스트
life-text-intro = 패턴을 붙여 넣으십시오: RLE(LifeWiki와 Golly가 제공하는 형식), 일반 텍스트 .cells, 또는 Life 1.05 / 1.06.
life-text-load = 불러오기
life-text-current = 현재 상태를 RLE로
life-text-current-hover = 마지막으로 불러온 세계로 상자를 채웁니다.

## L-systems: the side panel

# Row label: the system list (system names stay in English).
lsys-system = 시스템
lsys-open = 열기…
lsys-open-hover = Fractint .l 파일이나 .lsys 시스템을 엽니다.
lsys-edit = 편집…
lsys-edit-hover = 시스템의 텍스트: 공리, 생성 규칙, 각도 — 편집하고 적용합니다.
lsys-save = 저장…
lsys-save-hover = 시스템을 .lsys 텍스트 파일로 저장합니다.
# Tooltip of the "SVG…" button.
lsys-svg-hover = 뷰를 SVG 도면으로 내보냅니다: 선과 도형을 벡터로, 뷰의 색상 그대로.
lsys-follow = 차수가 줌을 따름
lsys-follow-hover = 확대할수록 차수가 올라가므로 디테일이 바닥나지 않습니다.
lsys-follow-fixed-hover = 이 시스템은 차수마다 일정한 배율로 커지지 않으므로, 고정된 차수로 그립니다.
lsys-order = 차수
lsys-angle = 각도
lsys-sweep-hover = 각도를 갔다가 돌아오며 쓸어 갑니다: 회전하면서 변형되는 곡선
# A small button that puts back the system's own angle after the slider moved it.
lsys-system-angle = 시스템 각도
lsys-seed = 시드
lsys-seed-hover = 무작위 선택이 따르는 값: 같은 시드면 같은 식물.
lsys-seed-new = 새로
lsys-seed-new-hover = 다른 식물: 새 시드.
lsys-line-width = 선 너비
# Row label of a percentage slider: how much of the curve is drawn.
lsys-draw-on = 그리기 진행
lsys-draw-on-hover = 거북이가 그리는 순서대로, 곡선을 얼마나 그릴지
lsys-draw-on-play-hover = 스스로 그려지는 곡선
lsys-colour-by = 색 기준
# The colourings, in the "Colour by" list.
lsys-colour-position = 곡선 위의 위치
lsys-colour-depth = 가지 깊이
lsys-colour-heading = 방향
# C, < and > are the system's commands that set the colour index: keep them.
lsys-colour-index = 색 인덱스(C, <, >)
lsys-colour-plain = 한 가지 색
# The panel's numbers.
lsys-growth = 성장
lsys-growth-built = 해당 차수에서 단어로 생성
lsys-growth-built-hover = 매개변수적 또는 문맥 의존적: 모듈의 재작성이 그 숫자나 이웃에 따라 달라지므로, 단어를 한 세대씩 생성하고(최대 200만 모듈) 배정밀도로 그립니다 — 차수는 줌을 따르지 않습니다.
# How much the picture grows from one order to the next. $factor: the factor (1.4142).
lsys-growth-factor = 차수당 ×{ $factor }
lsys-growth-factor-period-2 = 차수당 ×{ $factor }(차수는 2씩 증가)
lsys-growth-none = 없음(고정 차수)
lsys-drawn = 그린 양
# $drawn: what was drawn (lsys-drawn-segments, lsys-drawn-shapes or lsys-drawn-segments-shapes); $order: the order.
lsys-drawn-at-order = 차수 { $order }에서 { $drawn }
# The same, when the walk stopped at its segment budget.
lsys-drawn-at-order-budget = 차수 { $order }에서 { $drawn }(예산)
# $segments: the line segments drawn.
lsys-drawn-segments = 선분 { $segments }개
# $shapes: the filled shapes drawn.
lsys-drawn-shapes = 채운 도형 { $shapes }개
# Both counts. $segments: line segments; $shapes: filled shapes.
lsys-drawn-segments-shapes = 선분 { $segments }개, 채운 도형 { $shapes }개
# $ms: the walk's time in milliseconds (12.3).
lsys-drawn-hover = { $ms } ms 동안 순회했습니다. 뷰 밖의 하위 트리는 건너뛰고, 한 픽셀보다 작은 하위 트리는 선분 하나로 그립니다
# A word too long to build at the order asked for. $wanted: the order asked for; $built: the order drawn.
lsys-drawn-short = 차수 { $wanted }의 단어가 200만 모듈을 넘습니다: 차수 { $built }에서 그렸습니다.
# $order: the deepest order the app draws.
lsys-deepest-order = 그릴 수 있는 가장 깊은 차수({ $order })입니다: 더 확대해도 디테일이 늘지 않습니다.

## L-systems: notices in the panel and the editor, and file dialogs

# $path: the file; $error: why (in English).
lsys-open-failed = { $path }을(를) 열 수 없습니다: { $error }
# Why a file was not opened (shown after "Could not open …:").
lsys-file-too-large = 4 MiB보다 큼
# Why a Fractint .l file was not opened: it holds no entries. `Name { … }` is the file's syntax: keep it.
lsys-no-entries = 항목 없음(.l 파일에는 `Name {"{"} … {"}"}` 항목이 들어 있습니다)
# $path: the file written.
lsys-saved = 저장했습니다: { $path }
# $error: why (from the system, in English).
lsys-save-failed = 저장 실패: { $error }
# After File ▸ Export SVG. $path: the file written.
lsys-svg-exported = 내보냈습니다: { $path }
# $error: why (from the system, in English).
lsys-svg-failed = 내보내기 실패: { $error }
# File-dialog filter names: opening (any system file) and saving (one system).
lsys-filter-systems = L-시스템
lsys-filter-system = L-시스템

## L-systems: the toolbar

lsys-toolbar-lower-hover = 차수 낮추기(고정됨)
lsys-toolbar-raise-hover = 차수 높이기(고정됨)
# Between the − and + buttons, in a fixed-width font. $order: the order, padded to 3 characters so the label keeps its width.
lsys-toolbar-order = 차수 { $order }
# A toggle: the order follows the zoom. Short.
lsys-toolbar-auto = 자동
lsys-toolbar-auto-hover = 차수가 줌을 따름
lsys-toolbar-edit-hover = 시스템의 텍스트 편집

## L-systems: the editor window, and a .l file's entries

lsys-editor-title = L-시스템
# The editor's help. The quoted examples ('X = word', 'seed n', 'define R 1.456'…), the commands and "The Algorithmic Beauty of Plants" (a book's title) are the system's syntax and names: keep them as they are.
lsys-editor-help = 한 줄에 키 하나: angle(도, 또는 원을 나눈 /n), axiom, 생성 규칙 'X = word', 그리고 선택적으로 heading, draw / move / variables, colour, order. 무작위로 고르는 대안이 있는 기호는 대안마다 한 줄씩, 가중치와 함께 씁니다: 'X (0.3) = word'. 'seed n'은 식물을 고릅니다. 단어 안에서 {"{"} {"}"}는 거북이의 경로를 채우고, .는 꼭짓점을 추가합니다. 매개변수적 및 문맥 의존적 생성 규칙(The Algorithmic Beauty of Plants): 'A(s) : s > 1 = F(s)[+A(s/2)]', 'b < a > c = b', 그리고 'define R 1.456'과 'ignore +-F'. F(l)은 l만큼 나아가고, +(a)는 a도 회전합니다. Fractint .l 항목은 그대로 붙여 넣을 수 있습니다.
lsys-editor-apply = 적용
lsys-editor-revert = 되돌리기
lsys-editor-revert-hover = 표시된 시스템의 텍스트로 되돌립니다.
# The window listing a .l file's entries to pick from. $file: the file's name.
lsys-entries-title = { $file } — 항목
# Tooltip of an entry that does not parse (its error is shown beside it, in English).
lsys-entry-unreadable-hover = 이 항목은 해석되지 않습니다.

## Export: opening a view or location (File ▸ Open view or location…)

# File-dialog filter names.
export-filter-view = Fractadyne 뷰 또는 위치
export-filter-image = 이미지(PNG / EXR)
export-filter-location = 위치(.fdn / .kfr)
# $zoom: the magnification (1.2e148), followed by the magnification sign ×.
export-loaded-location = 위치를 불러왔습니다 @ { $zoom }×
# $zoom: the magnification; $note: what was noteworthy in the file (in English).
export-loaded-location-note = @ { $zoom }×에서 불러왔습니다 — { $note }
# $path: the file.
export-not-location = { $path }은(는) Fractadyne 위치가 아닙니다.
export-file-too-large = 파일이 너무 큽니다(.fdn 위치가 아닐 수 있습니다).
# $path: the file; $error: why (from the system, in English).
export-read-failed = { $path }을(를) 읽을 수 없습니다: { $error }
# A Kalles Fraktaler .kfr file did not import. $error: why (in English).
export-import-failed = 가져오기 실패: { $error }
# $path: an image whose embedded text is not a view.
export-not-view = { $path }에는 Fractadyne 메타데이터가 있지만 뷰는 아닙니다.
# $path: the image the view was read from.
export-loaded-view = { $path }에서 뷰를 불러왔습니다
# $path: the image; $note: what was noteworthy in it (in English).
export-loaded-view-note = { $path }에서 뷰를 불러왔습니다 — { $note }
export-no-metadata = 이 파일에는 포함된 Fractadyne 뷰 메타데이터가 없습니다.

## Export: notices after an export or a snapshot
## "File → Export image…" in them names the menu items (menu-file, menu-file-export).

# The export window's status while the image renders.
export-rendering = 렌더링 중…
export-canceled = 내보내기를 취소했습니다.
# $error: why (in English).
export-failed = 내보내기 실패: { $error }
# $size: the image's size in pixels (1920×1080); $path: the file written.
export-saved = { $size } 저장 완료 → { $path }
# An L-system export whose walk stopped at its budget. $size: pixels; $segments: the budget (4,000,000); $path: the file.
export-saved-incomplete = { $size } 저장 완료(불완전: 선분 { $segments }개 예산 초과 — 차수를 낮추십시오) → { $path }
# A two-panel export written as two files. $path: the first of them.
export-saved-two = 파일 2개 저장 완료 → { $path }
# $size: pixels; $path: the file written.
export-saved-corrected = { $size } 저장 완료(글리치 보정됨) → { $path }
# $path: the first of the two files.
export-saved-two-corrected = 파일 2개 저장 완료(글리치 보정됨) → { $path }
# Added after a finished export's notice. $duration: how long it took (8.3s, 1m 04.0s).
export-elapsed = ({ $duration } 소요)
export-preparing-deep = 깊은 내보내기 준비 중 — 기준 궤도 계산 중(시간이 걸릴 수 있습니다)…
# $size: pixels (5120×4035); $ss: the supersampling factor; $iterations: the iteration count.
export-large = 큰 내보내기({ $size }, { $ss }× 슈퍼샘플링, 반복 { $iterations }회) — 백그라운드에서 렌더링 중입니다. 이 크기의 내보내기에서는 글리치 보정을 건너뜁니다…
export-life-unavailable = Life 뷰의 이미지 내보내기는 아직 지원하지 않습니다 — 스크린샷이나 파일 ▸ Life 패턴 저장을 사용하십시오.
export-already-running = 이미 내보내기가 실행 중입니다 — 파일 → 이미지 내보내기…에서 진행 상황을 볼 수 있습니다.
# $size: the image's size in pixels.
export-snapshot-background = 스냅샷: 백그라운드에서 { $size }을(를) 렌더링 중 — 파일 → 이미지 내보내기…에서 진행 상황을 보고 취소할 수 있습니다.
export-no-gpu = GPU를 사용할 수 없음
# A screen-capture snapshot. $size: pixels; $path: the file written.
export-snapshot-saved = 화면 스냅샷 { $size } 저장 완료 → { $path }
# $error: why (in English).
export-snapshot-failed = 스냅샷 실패: { $error }

## The colour picker (the popup of a colour swatch)

# The third row's label: the colour as hexadecimal bytes. Short (about 6 characters).
picker-row-hex = 16진
picker-hex-hover = 클릭하여 16진수 값 입력
picker-cancel = 취소
picker-cancel-hover = 이 선택기를 열 때의 색으로 되돌리기
picker-ok = 확인
picker-ok-hover = 이 색 유지
# Starts the screen eyedropper (an icon is shown before it).
picker-pick = 스포이트
picker-pick-hover = 화면 어디에서나 색 가져오기
# Tooltip of the disabled Pick button (in the picker and the gradient editor) on Linux and macOS.
picker-eyedropper-unsupported = 화면의 다른 곳에서 색을 가져오려면 플랫폼의 화면 캡처 API가 필요합니다. Windows 빌드에만 있으며, Linux에서는 X11 의존성이 필요하고, Wayland는 자체 권한 확인 창이 있는 포털을 요구합니다.

## The fractal families: what each one is (the info panel's "About" section and the pickers' hover text).
## The formulas themselves (z -> z^2 + c) are written in code and stay as they are.

fractal-mandelbrot-about = 대표적인 탈출 시간 프랙털: 0의 궤도가 유계로 머무는 c의 집합입니다. 그 경계는 무한히 복잡합니다.
fractal-multibrot3-about = Multibrot 집합 - Mandelbrot 구성을 더 높은 거듭제곱으로 한 것입니다. 거듭제곱 d는 (d-1)겹 회전 대칭을 줍니다.
fractal-multibrot4-about = 거듭제곱 4의 Multibrot: 3겹 대칭, 넓은 벌브.
fractal-multibrot5-about = 거듭제곱 5의 Multibrot: 4겹 대칭.
fractal-tricorn-about = Tricorn(Mandelbar): 단계마다 z의 켤레를 취합니다. 이 반정칙 사상은 세 모서리 모양을 만듭니다.
fractal-burning-ship-about = 제곱하기 전에 z의 두 부분에 절댓값을 취합니다. 그 결과는 불타는 배를 닮았습니다.
fractal-celtic-about = z^2의 실수부에만 절댓값을 취하는 Burning Ship의 친척으로, 켈트 매듭 / 하트 무늬를 만듭니다.
fractal-buffalo-about = z^2의 두 성분 모두에 절댓값을 취하는 절댓값 변형입니다.
fractal-phoenix-about = Phoenix는 이전 반복값도 사용해 불꽃 같은 가는 실을 만듭니다. Julia 모드로 Julia 형태를 시험해 보십시오.
fractal-newton-about = z^3 = 1에 대한 뉴턴의 근 찾기 반복으로, 각 점이 얼마나 빨리 수렴하는지에 따라 색을 칠합니다. 탈출이 아니라 수렴 프랙털입니다.
fractal-multibrot6-about = 거듭제곱 6의 Multibrot: 5겹 대칭.
fractal-multibrot7-about = 거듭제곱 7의 Multibrot: 6겹 대칭, 벌브가 가늘어지며 원에 가까워집니다.
fractal-multibrot8-about = 거듭제곱 8의 Multibrot: 7겹 대칭.
fractal-burning-ship3-about = 거듭제곱 3의 Burning Ship: 절댓값을 취한 뒤 세제곱합니다.
fractal-burning-ship4-about = 거듭제곱 4의 Burning Ship.
fractal-burning-ship5-about = 거듭제곱 5의 Burning Ship.
fractal-tricorn3-about = 거듭제곱 3의 Tricorn(Mandelbar): 4겹 대칭.
fractal-tricorn4-about = 거듭제곱 4의 Tricorn: 5겹 대칭.
fractal-tricorn5-about = 거듭제곱 5의 Tricorn: 6겹 대칭.
fractal-celtic3-about = 거듭제곱 3의 Celtic 접기: z^3의 실수부의 절댓값.
fractal-celtic4-about = 거듭제곱 4의 Celtic 접기.
fractal-celtic5-about = 거듭제곱 5의 Celtic 접기.
fractal-buffalo3-about = 거듭제곱 3의 Buffalo: z^3의 두 부분 모두의 절댓값.
fractal-buffalo4-about = 거듭제곱 4의 Buffalo.
fractal-buffalo5-about = 거듭제곱 5의 Buffalo.
fractal-custom-about = Fractint 방식의 식으로 쓴 수식입니다(프랙털 ▸ 사용자 정의 수식…). 대부분 내장 계열처럼 섭동으로 딥 줌하며, 사용자의 수식이 그런지는 수식 대화 상자가 알려 줍니다.
fractal-life-about = 무한한 평면 위의 Conway의 생명 게임과 그 친척들: Generations 규칙, 비총합(Hensel) 규칙, 패턴 파일. 재생, 단계 진행, 그리기.
fractal-lsystem-about = Lindenmayer 시스템: 재작성 문법으로부터 거북이가 그리는 곡선, 공간 채움 곡선, 섬, 식물. 차수가 줌을 따르므로 디테일이 바닥나지 않습니다. Fractint .l 파일을 바로 열 수 있습니다.

# Shown where the other families show their formula (z -> z^2 + c).
fractal-custom-formula = z -> 사용자가 쓴 수식
# Shown where the other families show their formula. "B3/S23" is the rule's notation: keep it.
fractal-life-formula = B3/S23: 살아 있는 이웃이 3이면 태어나고, 2나 3이면 살아남음
# Shown where the other families show their formula. "F = F+F--F+F" is the grammar's notation: keep it.
fractal-lsystem-formula = F = F+F--F+F: 각 기호를 재작성한 뒤 거북이가 그림

# The last paragraph of the picker's hover text for a family that renders only on the direct path.
fractal-no-deep-zoom = 직접 렌더링만 지원 - 이 계열에는 딥 줌이 없습니다.

## Names of settings, shown in the Settings menu and the welcome window.

theme-dark = 어둡게
theme-light = 밝게
# The update track that follows only stable releases.
update-track-stable = 안정판
# The update track that also offers pre-releases (betas).
update-track-beta = 베타(사전 릴리스)

## The status bar (its hover texts; the readouts follow in their own section)

status-period-hover = 이동한 미니브로트의 주기입니다(M을 누르거나 위치로 이동 ▸ 특징점으로 이동). 뷰가 그 특징점에 있는 동안 유지되고, 벗어나면 사라집니다.

## The playback transport: the player over the view while a tour plays.

# $close: the close button's icon, as drawn on that button.
play-finished-hover = 끝남 — 스크러브해 되돌아가거나, { $close }로 닫으십시오
# $kind: "benchmark" or "script" (the variants hold the wording); $name: the tour's name.
play-playing-hover =
    { $kind ->
        [benchmark] 벤치마크 “{ $name }” 재생 중
       *[other] 스크립트 “{ $name }” 재생 중
    }
play-restart = 처음부터 다시 시작
play-back = 10초 뒤로
play-again = 처음부터 다시 재생
play-resume = 재개
play-pause = 일시 정지
play-stop = 멈추고 처음으로 되감기
play-forward = 10초 앞으로
play-speed-hover = 재생 속도(클릭하면 0.5× / 1× / 2× / 4× 순환)
play-render = 이 스크립트를 프레임 시퀀스로 렌더링…
play-render-none = 내장 벤치마크에는 렌더링할 스크립트 파일이 없습니다
play-open = 다른 스크립트 재생…
play-loop = 끝에 도달하면 투어 반복
play-close = 플레이어를 닫고 사용자의 뷰 설정 복원
# A notice beside the scrub bar while the renderer holds the tour's clock. Keep it short.
play-waiting = 디테일 대기 중
# "[playback] pace" is a setting in the script file: keep it as written.
play-waiting-hover = 렌더러가 이 뷰를 그려 내는 동안 투어 시계가 멈춰 있습니다(기준 궤도 계산 / 반복 예산 상승). 재생은 저절로 재개됩니다. 스크립트의 [playback] pace를 참조하십시오.
play-scrub = 드래그하여 투어 탐색
# $behind: how far the script clock is behind real time, as minutes:seconds (1:05).
play-scrub-behind =
    드래그하여 투어 탐색
    스크립트 시계가 실제 시간보다 { $behind } 뒤처져 있습니다(렌더러가 따라올 수 있도록 속도 조절기가 투어를 늦췄습니다). 위쪽 눈금은 실제 시계 기준으로 재생되었을 위치를 표시합니다.

## The autopilot's notices (toasts). Where a notice has two lines, the first is the verdict and the second what to do about it.

autopilot-on-detail = 자동 조종 켜짐 — 디테일을 향해 다이브(아무 입력이나 하면 멈춤)
autopilot-off = 자동 조종 꺼짐
autopilot-misi-needs-mandelbrot =
    자동 조종: Misiurewicz 목표에는 Mandelbrot 집합이 필요합니다
    대신 디테일을 향해 다이브합니다
autopilot-on-misi =
    자동 조종 켜짐 — 가장 가까운 Misiurewicz 점을 찾는 중…
    아무 입력이나 하면 멈춤
# $seconds: how long the search has run, in whole seconds.
autopilot-misi-finding =
    자동 조종: 가장 가까운 Misiurewicz 점을 찾는 중…
    { $seconds } s · 아무 입력이나 하면 멈춤
autopilot-misi-not-found =
    자동 조종: 이 뷰 근처에서 Misiurewicz 점을 찾지 못했습니다
    찾기 기능이 이 깊이에 닿지 못할 수 있습니다 — 디테일 목표를 시도하십시오
# ($preperiod,$period) names the point, as mathematicians write it: keep the notation. $widths: a power of two (2^$widths screen widths).
autopilot-misi-off-screen =
    자동 조종: 가장 가까운 Misiurewicz 점 ({ $preperiod },{ $period })이(가) 화면 밖에 있습니다
    화면 너비의 ~2^{ $widths }배 거리 — 나선 중심에 더 가까이 이동하십시오
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-not-solved =
    자동 조종: 이 뷰 근처의 Misiurewicz 점 ({ $preperiod },{ $period })을(를) 구하지 못했습니다
    디테일 목표를 시도하십시오
autopilot-misi-refused = 자동 조종: Misiurewicz 계산이 거부되었습니다(잘못된 쌍)
autopilot-misi-stopped = 자동 조종: Misiurewicz 솔버가 예기치 않게 멈췄습니다
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-diving =
    자동 조종: Misiurewicz ({ $preperiod },{ $period })(으)로 다이브 중
    아무 입력이나 하면 멈춤
# ($preperiod,$period) names the point: keep the notation. $octaves: how often the structure repeats (2.37); $twist: its turn in degrees, signed (+12.5).
autopilot-misi-diving-repeat =
    자동 조종: Misiurewicz ({ $preperiod },{ $period })(으)로 다이브 중
    구조가 { $octaves }옥타브마다 반복, 비틀림 { $twist }° · 아무 입력이나 하면 멈춤
# $limit: the magnification, as written (1e300).
autopilot-dive-limit = 자동 조종: 다이브 한도에 도달했습니다(~{ $limit }×)
# $have: the fixed iteration count; $typical: what this depth typically needs.
autopilot-dead-end-short =
    자동 조종 중지: 더 겨냥할 곳이 없습니다 — 반복 횟수가 { $have }회로 고정되어 있지만 이 깊이에는 보통 약 { $typical }회가 필요합니다
    자동 반복 횟수를 켜거나 횟수를 올리십시오
# $limit: the fixed iteration count.
autopilot-dead-end-cap =
    자동 조종 중지: 더 겨냥할 곳이 없습니다 — 뷰의 모든 것이 고정된 반복 한도 { $limit }회까지 실행됩니다
    자동 반복 횟수를 켜거나 횟수를 올리십시오
autopilot-dead-end = 자동 조종: 앞에 디테일이 없습니다(중지됨)

## Tours (camera scripts)

# The name of the file type in the file dialog's filter list.
script-file-filter = Fractadyne 스크립트(TOML)
# $name: the script's name.
script-finished = 스크립트 완료 — “{ $name }”
# The title of the notice that follows (the file's path and the error, in English).
script-load-failed = 스크립트를 불러올 수 없습니다

## Installing a language file

# The name of the file type in the file dialog's filter list.
language-file-filter = Fractadyne 언어

## The notice when the graphics driver's shader cache stops keeping shaders. The text is shown in a fixed-width font with its line breaks as written: break the lines where they read well.

driver-cache-title = 그래픽 드라이버가 셰이더를 보관하지 않음
# $count: how many shaders compiled again; $pipeline: the slowest one's name (a technical id: keep it); $seconds: its time in seconds (7.5).
driver-cache-body =
    그래픽 드라이버가 이번 세션에서 Fractadyne의 셰이더
    { $count }개를, 앞서 이미 각각 컴파일했는데도 다시 컴파일했습니다
    (가장 느린 것: { $pipeline }, { $seconds } s). 드라이버의 셰이더
    캐시가 컴파일된 셰이더를 보관하지 못하고 있으며, 대개 캐시가
    가득 찼기 때문입니다. 이 상태가 바뀔 때까지 내보내기, 깊은 뷰,
    Fractadyne 시작에 각각 몇 초씩 컴파일 시간이 들 수 있습니다.
# $folder: the cache folder's path; $size: its size in MB (1024). "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia-folder =
    해결 방법: Fractadyne을 닫은 뒤 다음 폴더의 내용을 삭제하십시오.
      { $folder }   ({ $size } MB)
    드라이버가 캐시를 다시 만듭니다. 다시 가득 차지 않게 하려면
    NVIDIA 제어판 > 3D 설정 관리 > 셰이더 캐시 크기를 늘리십시오.
# "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia =
    해결 방법: NVIDIA 제어판 > 3D 설정 관리 > 셰이더 캐시 크기를
    늘리거나, 드라이버의 셰이더 캐시를 비우십시오.
# "AMD Software > Graphics > Advanced > Reset Shader Cache" names AMD's own settings: use the names AMD's software shows in this language.
driver-cache-fix-amd = 해결 방법: AMD Software > 그래픽 > 고급 > 셰이더 캐시 재설정.
driver-cache-fix-other = 해결 방법: 그래픽 드라이버의 셰이더 캐시를 비우십시오.

## Why a language file could not be installed: each ends the sentence "{ $file } is not a usable language file: …".

language-install-not-utf8 = UTF-8 텍스트가 아님
language-install-too-large = 너무 큼
# "fractadyne-<language>.ftl" is the file name to use: keep it as written.
language-install-no-tag = 어떤 언어인지 나타나 있지 않음(이름을 fractadyne-<language>.ftl로 지정하십시오)
language-install-english = 영어는 내장되어 있음
language-install-empty = 이 버전의 메시지를 하나도 번역하지 않음
language-install-no-folder = 설정 폴더가 없음

## The notice after a session that did not end cleanly, when its iteration setting was set aside.

# $base: the iteration count automatic iterations start from.
quarantine-setting-auto = 기본값 { $base }에서 시작하는 자동 반복 횟수
# $count: the fixed iteration count.
quarantine-setting-fixed = 반복 { $count }회
# $end: "device" when the session lost its graphics device, "unclean" otherwise (the variants hold the wording). $depth: the magnification, as written (1.2e148). $setting and $restored: an iteration setting (quarantine-setting-auto or -fixed).
quarantine-notice =
    { $end ->
        [device] 이전 세션이 { $setting }(으)로 설정된 상태에서, { $depth }×에서 그래픽 장치를 잃었습니다. 같은 부하로 곧바로 다시 들어가지 않도록, 이번 세션은 대신 { $restored }(으)로 시작했습니다.
       *[other] 이전 세션이 { $setting }(으)로 설정된 상태에서, { $depth }×에서 정상적으로 종료되지 않았습니다. 같은 부하로 곧바로 다시 들어가지 않도록, 이번 세션은 대신 { $restored }(으)로 시작했습니다.
    }

## The second graphics card's status line, under its setting in the control panel.

gpu2-state-off = 끄기: 이 창의 그래픽 카드가 모든 것을 렌더링합니다.
gpu2-state-starting = 시작 중…
# $name: the card's name (and its graphics API).
gpu2-state-running = 사용 중: { $name }
# $why: the error, as the graphics driver reports it.
gpu2-state-failed = 시작할 수 없음: { $why }
# $name: the card's name.
gpu2-state-lost = 중지됨: { $name }을(를) 잃었습니다. 이제부터 카드 하나로 동작합니다. 다시 시도하려면 다시 선택하십시오.
# $state: one of the lines above. "--worker-gpu" is a command-line option: keep it.
gpu2-state-pinned = { $state }(--worker-gpu로 설정됨)

## The custom-formula dialog: how deep the formula renders (one line under the formula).

formula-depth-deep = 섭동에 의한 딥 줌, 내장 프랙털처럼 1e28x를 넘으면 확장 범위(모든 단계를 반복: 사용자 정의 수식에는 아직 급수 근사나 BLA가 없음).
# $why: what the formula has that keeps it off the deep-zoom path (one of the formula-depth-why messages below).
formula-depth-direct = 직접 렌더링({ $why }에는 아직 딥 줌 형태가 없음): c의 단정밀도 한 단계가 한 픽셀에 걸칠 때까지 선명합니다 — 약 1e4x에서 1e5x까지이며, 원점에서 멀어질수록 덜 깊습니다.
formula-depth-why-init = 초기화 섹션
formula-depth-why-bailout = 자체 탈출 조건
formula-depth-why-variable = 단계에서 단계로 유지되는 변수
formula-depth-why-previous = 이전 반복값
formula-depth-why-comparison = 비교식이나 if 블록
# floor, ceil, trunc and round are functions of the formula language: keep them.
formula-depth-why-rounding = 반올림(floor, ceil, trunc, round)
# "maxit" is a word of the formula language: keep it.
formula-depth-why-maxit = maxit
formula-depth-why-power = 지수가 변하는 거듭제곱
formula-depth-why-perturbed = 이미 섭동된 프로그램

## Image and frame sizes: the presets described in words (the others are standard names: HD 720p, WUXGA, DCI 4K).

# $size: the pixel size (2048×2048).
size-preset-square = 정사각형 — { $size }
# $ratio: the aspect ratio (9:16); $size: the pixel size.
size-preset-portrait = 세로 { $ratio } — { $size }
# $ratio: the aspect ratio (32:9); $size: the pixel size.
size-preset-super-ultrawide = 슈퍼 울트라와이드 { $ratio } — { $size }

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
fractal-family-custom = 사용자 정의
# Conway's Game of Life.
fractal-family-life = Life
fractal-family-lsystem = L-시스템
# A family at a power: "Multibrot 3", "Burning Ship 4". $family: a name above; $power: the power.
fractal-with-power = { $family } { $power }

## Names of choices: colouring methods, orbit traps, palette animation, snapshots, angle units.

color-method-smooth = 부드러운 반복
color-method-stripe = 스트라이프 평균
color-method-triangle = 삼각 부등식
color-method-trap = 궤도 트랩
color-method-distance = 거리 추정
color-method-decomposition = 분해
# The shape an orbit trap catches the orbit with.
trap-point = 점
trap-cross = 십자
trap-circle = 원
# Palette animation: none.
palette-anim-off = 끄기
palette-anim-forward = 정방향
palette-anim-reverse = 역방향
palette-anim-pingpong = 핑퐁
# Palette animation: a new random gradient every so often.
palette-anim-random = 무작위 그라데이션
# What the Snapshot button does.
snapshot-mode-ask = 매번 묻기
snapshot-mode-screen = 화면 캡처
snapshot-mode-render = 전체 렌더링
# The unit of an angle typed into Go to location's polar fields.
angle-degrees = 도
angle-radians = 라디안
angle-turns = 회전

## Report an issue: the choices. (The report itself is sent in English.)

issue-kind-crash = 애플리케이션 충돌 / 멈춤
issue-kind-rendering = 잘못된 렌더링
issue-kind-performance = 성능 문제
issue-kind-ui = UI / 사용성 문제
issue-kind-feature = 기능 요청
issue-kind-other = 기타
severity-low = 낮음
severity-medium = 보통
severity-high = 높음
severity-blocking = 차단
repro-always = 항상
repro-sometimes = 가끔
repro-once = 한 번 발생
repro-cannot = 재현할 수 없음

## The benchmark's depths and the diagnostics' tests.

# $zoom: the dive's final magnification (1e32).
bench-depth-standard = 표준({ $zoom }×)
# $zoom: the dive's final magnification (1e48).
bench-depth-ultra = 초심도({ $zoom }×)
diag-test-selftest = 자체 테스트
diag-test-uitest = UI 테스트
diag-test-gputest = GPU 산술 검사

## The status bar's readouts: each word is followed by a value in a slot of fixed width. Keep the words short (one word each).

status-center = 중심
status-cursor = 커서
status-zoom = 줌
# Short for iterations.
status-iter = 반복
# The period of the minibrot jumped to.
status-period = 주기
# A Game of Life view's scale (followed by "1 px = 4 cells" or "8 px / cell").
status-scale = 축척
# A Game of Life view's generation. Short.
status-generation = 세대
# A Game of Life view's population. Short.
status-population = 개체
# An L-system's order (how many times its rules were applied).
status-order = 차수
# How many segments and shapes an L-system drew.
status-drawn = 그림
# $value: how many cells one pixel covers (4, 9.99e300). "px" is the unit symbol: keep it.
life-scale-cells = 1 px = 셀 { $value }개
# $value: how many pixels one cell covers (8). "px" is the unit symbol: keep it.
life-scale-pixels = { $value } px / 셀

## The status bar's limit diagnostic: a short label (after a warning sign), and what it means when hovered.

# The view is as deep as this location can be rendered. Short.
status-limit-depth = 깊이 한계
# Every pixel ran out of iterations. Short.
status-limit-exhausted = 반복 소진
# Pixels are capped at the iteration budget. Short.
status-limit-capped = 반복 상한
# $pixels: how many pixels wide one step of the coordinates is; $sharp_to: the magnification the view is sharp to (written out, 1,234,567).
status-limit-direct-detail = 이 뷰는 GPU에서 단정밀도로 직접 렌더링되며, 여기서는 좌표의 한 단계가 { $pixels }픽셀 너비입니다: 이웃한 픽셀들이 같은 점을 계산하므로 이미지가 블록으로 깨집니다. 이 위치에서는 약 { $sharp_to }×까지 선명합니다. 이 수식에는 아직 딥 줌(섭동) 경로가 없으므로, 더 확대해도 디테일이 더 보이지 않습니다.
# $samples: the GPU buffer's capacity in orbit samples; $iterations: the iterations pixels are limited to.
status-limit-depth-detail = 기준 궤도가 탈출하지 않은 채 GPU 버퍼 상한(샘플 { $samples }개)에 도달했으므로, 이곳의 픽셀은 약 { $iterations }회 반복으로 제한되어 이 뷰에서는 더 세밀한 디테일을 계산할 수 없습니다 — 라이브든 내보내기든. 이것이 이 위치 계열의 현재 깊이 한계입니다.
# $budget: the largest iteration budget this view can be given.
status-limit-exhausted-detail = 이 뷰에 줄 수 있는 가장 큰 예산({ $budget })에서도 모든 픽셀이 반복을 다 써 버렸습니다. 이곳이 집합 내부이거나(깊은 곳에서는 정상), 앱이 현재 제공할 수 있는 것보다 많은 반복이 필요한 것입니다 — 어느 쪽이든 이미지는 같으므로, 예산을 저렴한 값으로 되돌렸습니다.
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations" is the setting's name in the control panel's Quality section.
status-limit-capped-max = 픽셀의 { $percent }%가 { $budget } 예산에서 반복을 다 써 버렸고, 탈출하는 픽셀도 바로 그 상한에서 끝나고 있습니다 — 뷰가 충분히 그려지지 않았습니다. 반복 횟수가 이미 최대이므로, 이 뷰에는 앱이 현재 제공할 수 있는 것보다 많은 반복이 필요합니다.
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations", "Controls" and "Quality" are the names of a setting, the control panel and its section.
status-limit-capped-raise = 픽셀의 { $percent }%가 { $budget } 예산에서 반복을 다 써 버렸고, 탈출하는 픽셀도 바로 그 상한에서 끝나고 있습니다 — 뷰가 충분히 그려지지 않았습니다. 반복 횟수(컨트롤 ▸ 품질)를 올려 해결하십시오.
