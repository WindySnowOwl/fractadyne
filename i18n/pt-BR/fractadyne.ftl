### Fractadyne user interface: Portuguese, Brazil (Português (Brasil)).
### A draft: machine translation (Claude, by Anthropic) of the English file, 2026-10-23, not yet
### reviewed by a native speaker. Corrections are welcome as pull requests (see i18n/README.md).

## Language data: read by Fractadyne, not shown as messages.

# This language's own name, shown in the language menu.
language-name = Português (Brasil)
# The same name in English, shown beside it in the menu.
language-english-name = Portuguese (Brazil)
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
language-more = Mais idiomas…
# The Languages window's title.
language-window-title = Idiomas
# The Languages window's introduction.
language-window-intro = A interface do Fractadyne em outro idioma. Cada idioma é um arquivo: ele vem com uma versão, ou você pode instalá-lo aqui. Coordenadas, arquivos e a linha de comando continuam em inglês em todos os idiomas.
# Column headers of the installed-language list.
language-col-language = Idioma
language-col-translated = Traduzido
language-col-from = Origem
# Where a language comes from.
language-source-builtin = integrado
language-source-package = esta versão
language-source-user = instalado
language-source-dev = pasta de trabalho
# $percent: how much of the interface the file translates, 0–100.
language-coverage = { $percent }%
# $count: messages in the file that failed the checks and show in English instead.
language-dropped =
    { $count ->
        [one] { $count } mensagem precisa ser verificada
       *[other] { $count } mensagens precisam ser verificadas
    }
# Buttons of the Languages window.
language-install = Instalar arquivo de idioma…
language-install-hover = Escolha um arquivo de idioma do Fractadyne (.ftl). Ele é verificado e depois copiado para a pasta de idiomas das suas configurações, onde permanece entre atualizações.
language-open-folder = Abrir pasta de idiomas
language-reload = Recarregar
language-reload-hover = Ler os arquivos de idioma novamente (depois de editar ou adicionar um).
language-use = Usar
# Notices after installing a file. $name: the language's own name; $file: the file's name.
language-installed = { $name } instalado a partir de { $file }.
# $file: the file's name; $why: what was wrong with it.
language-install-failed = { $file } não é um arquivo de idioma utilizável: { $why }
# Shown when a language file cannot be loaded and English is used instead. $name: the language.
language-load-failed = Não foi possível carregar { $name }, então o Fractadyne está em inglês: { $why }
# A language whose writing system needs a font this computer does not have (Japanese or Chinese without a system font). $name: the language, in its own name.
language-font-missing = Nenhuma fonte capaz de desenhar { $name } foi encontrada neste computador, então o idioma não pode ser exibido. Instale uma fonte para { $name } (por exemplo, o pacote de idioma do próprio sistema) e escolha-o novamente.
# Shown after a draft translation's name in the language menu and the Languages window. Short.
language-draft = rascunho
language-draft-hover = Um rascunho: esta tradução ainda não foi revisada por um falante nativo. Correções são bem-vindas, como um pull request para o arquivo do idioma.
# The heading over the decimal-mark choice, in the language menu and the Languages window.
language-decimal-mark = Separador decimal
language-decimal-mark-hover = Como os números são escritos: tempos, tamanhos e contagens. Coordenadas e ampliações sempre usam ponto, como os arquivos e outros programas as leem.
# $sample: a number written with this language's own punctuation (1,234.5).
language-decimal-language = Como o idioma escreve ({ $sample })
# $sample: a number written with a decimal point (1,234.5).
language-decimal-point = Ponto ({ $sample })
# $sample: a number written with a decimal comma (1.234,5).
language-decimal-comma = Vírgula ({ $sample })

## Asked once when the system's language changes

# The window's title.
language-changed-title = Idioma
# $new: the system's new language, in its own name.
language-changed-question = O idioma do seu sistema agora é { $new }. Usá-lo no Fractadyne?
# Buttons. $name: a language's own name.
language-changed-use = Usar { $name }
language-changed-keep = Manter { $name }

## The welcome window (first run)

welcome-title = Bem-vindo ao Fractadyne
welcome-intro = Um explorador de zoom profundo para o conjunto de Mandelbrot e seus parentes. Alguns controles para você começar:
# The language row, at the top of the window.
welcome-language = Idioma
# Controls: the key or gesture on the left, what it does on the right.
welcome-key-drag = arrastar
welcome-key-drag-does = mover a vista
welcome-key-scroll = rolar
welcome-key-scroll-does = aproximar / afastar o zoom em direção ao cursor
welcome-key-space = segurar Space
welcome-key-space-does = zoom contínuo e suave (Shift+Space afasta)
# Shown after the click-zoom toolbar icon.
welcome-key-click = e depois clicar
welcome-key-click-does = mergulhar no ponto clicado (barra de ferramentas)
welcome-key-m-does = saltar para o minibrot mais próximo, na escala dele
welcome-landmarks = Salte para um marco para começar:
welcome-theme = Tema
welcome-watermark = Mostrar a marca “Fd”
welcome-watermark-hover = A pequena marca no canto inferior direito da vista. Desligada, ela sai da vista ao vivo e das imagens exportadas.
welcome-updates = Atualizações
welcome-check-updates = Verificar atualizações
welcome-check-updates-hover = Consulta o GitHub pela versão mais recente quando o aplicativo inicia. Nada é enviado.
welcome-files = Arquivos
welcome-no-config = (sem pasta de configuração)
welcome-open = Abrir
welcome-rendering-on = Renderizando em
welcome-open-help = Abrir a ajuda completa  (F1)
# $icon: the Settings icon.
welcome-more-in-settings = Mais em Arquivo → { $icon } Configurações
welcome-get-started = Começar

## Menus: shared

# A percentage in a menu (UI scale). $value: the number.
common-percent = { $value }%
# A state shown after a menu item's name, in brackets.
menu-state-connected = conectado
menu-state-rendering = renderizando
menu-state-running = em execução

## Navigate ▸ Bookmarks (also the toolbar's star)

menu-bookmarks = Favoritos
menu-bookmarks-add = Adicionar a vista atual
menu-bookmarks-manage = Gerenciar…

## The fractal picker (Fractal menu and the toolbar's dropdown)

menu-family-powers = Potências maiores
menu-family-automata = Autômatos
menu-family-lsystems = Sistemas L
menu-fractal-custom = Fórmula personalizada…
menu-fractal-library = Biblioteca de fórmulas…
menu-fractal-library-hover = Fórmulas personalizadas salvas: aplique, edite, importe e exporte-as
# The "Show" group: which picture of the formula is drawn. $name: the formula's name (Mandelbrot, Burning Ship…).
menu-show = Mostrar
menu-show-set = Conjunto de { $name }
menu-show-set-hover = O plano de parâmetros — o próprio conjunto de { $name }.

    Cada ponto dele é um conjunto de Julia diferente; este é o mapa de todos eles.
menu-show-julia = Conjunto de Julia
menu-show-julia-hover = Um ponto do conjunto de { $name }, desenhado como uma imagem própria — a forma que você obtém mantendo esse ponto fixo.

    Escolha o ponto passando o cursor sobre o mapa em “Ambos, vinculados”; um clique ali o fixa.
menu-show-julia-none = { $name } não tem parâmetro livre, então não tem conjunto de Julia.
menu-show-both = Ambos, vinculados
menu-show-both-hover = O conjunto de { $name } à esquerda e, à direita, o conjunto de Julia do ponto que estiver sob o cursor.

    Mova o cursor pelo mapa e veja a Julia mudar — esta é a maneira mais rápida de ver como os dois se relacionam.
menu-show-both-none = { $name } não tem um conjunto de Julia para formar o par.

## File menu

menu-file = Arquivo
menu-file-open-view = Abrir vista ou local…
menu-file-gallery = Galeria…
menu-file-export = Exportar imagem…
menu-file-snapshot = Captura
menu-file-open-life = Abrir padrão do Life…
menu-file-open-life-hover = Um arquivo RLE, .cells ou Life 1.05/1.06, mostrado como um universo do Life.
menu-file-life-text = Texto de padrão do Life…
menu-file-life-text-hover = Cole um padrão como texto (RLE, como o LifeWiki o fornece).
menu-file-save-life = Salvar padrão do Life…
menu-file-save-life-hover = O universo como está agora, em RLE.
menu-file-open-lsystem = Abrir sistema L…
menu-file-open-lsystem-hover = Um arquivo .l do Fractint (com suas entradas para escolher) ou um sistema .lsys.
menu-file-save-lsystem = Salvar sistema L…
menu-file-save-lsystem-hover = O texto do sistema, como um arquivo .lsys.
menu-file-export-svg = Exportar SVG…
menu-file-export-svg-hover = A vista como um desenho SVG: suas linhas e formas como vetores, nas suas cores.
menu-file-render-client = Cliente de renderização…
menu-file-render-client-hover = Empreste a GPU desta máquina a uma fazenda de renderização: conecte-se a um controlador (Ferramentas ▸ Renderizar na fazenda… em outra máquina) e renderize quadros para ele.
menu-file-settings = Configurações
menu-file-reset = Redefinir estado do aplicativo…
menu-file-reset-hover = Excluir todo o estado salvo (sessão, favoritos, miniaturas) e começar do zero. Pede confirmação antes.
menu-file-quit = Sair

## File ▸ Settings

menu-settings-fps = Limite de quadros por segundo
menu-settings-fps-uncapped = Sem limite
# $fps: frames per second.
menu-settings-fps-n = { $fps } FPS
menu-settings-ui-scale = Escala da interface (tamanho da fonte)
menu-settings-theme = Tema
menu-settings-updates = Atualizações
menu-settings-updates-stable-hover = Última versão estável
menu-settings-updates-beta-hover = Última build, incluindo pré-lançamentos
menu-settings-updates-on-launch = Verificar atualizações ao iniciar
menu-settings-updates-on-launch-hover = Caso contrário, verifique manualmente em Ajuda → Verificar atualizações.
# Followed by its shortcut, (Ctrl+S).
menu-settings-snapshot = Captura
menu-settings-snapshot-ask-hover = Oferecer a escolha a cada vez
menu-settings-snapshot-screen-hover = Salvar a vista exatamente como aparece na tela (instantâneo, PNG)
menu-settings-snapshot-render-hover = Uma renderização completa com as configurações de exportação, em segundo plano (pode demorar muito em vistas profundas)
menu-settings-refcache = Cache de referências…
menu-settings-refcache-hover = Órbitas de referência de zoom profundo guardadas em disco, para que um local ao qual você volta seja renderizado em segundos. Onde fica, qual é o tamanho, o limite e uma forma de limpá-lo.

## Fractal menu

menu-fractal = Fractal

## View menu

menu-view = Exibir
menu-view-zoom-home = Afastar até a vista completa
menu-view-zoom-home-hover = Afastar o zoom até a vista inicial completa (animado)
menu-view-reset = Redefinir para a vista padrão
menu-view-reset-hover = Redefinir para a vista padrão do fractal (instantâneo)
# Followed by its shortcuts.
menu-view-undo = Desfazer vista
menu-view-redo = Refazer vista
menu-view-control-panel = Painel de controles
menu-view-control-panel-hover = Mostrar/ocultar o painel de controles à direita.
menu-view-minimap = Minimapa
menu-view-minimap-hover = Mostrar uma pequena visão geral da vista inicial, com um marcador “você está aqui” e a profundidade do zoom. Clique nela para saltar para uma região.
menu-view-perf-panel = Painel de desempenho
menu-view-timestamp = Mostrar carimbo de tempo
menu-view-timestamp-hover = Desenhar um grande relógio de tempo decorrido sobre a vista. Ele mostra o mesmo “+12.345s” com que o log marca cada linha, para que a gravação de tela de um problema que só aparece em movimento possa ser alinhada com o log quadro a quadro. Desligado por padrão.
menu-view-zoom-target = Mostrar o alvo do zoom automático
menu-view-zoom-target-hover = Enquanto o zoom automático estiver rodando, contornar a região da vista na qual ele está entrando — a parte que vai preencher a tela depois de mais algumas ampliações — com linhas dos cantos dela até os cantos da tela, e um anel no ponto para o qual ele está se dirigindo. Desligado por padrão.
menu-view-live-panel = Painel de renderização ao vivo
menu-view-live-panel-hover = Um painel frontal sobre a vista que mostra o renderizador ao vivo trabalhando: o que cada quadro despachou, a idade da imagem na tela, o percurso pelas faixas de iteração, os construtores de referência e a máquina por baixo. B alterna entre os tamanhos, e o × no canto o fecha; arraste-o para qualquer canto; passe o cursor sobre um instrumento para ver o que ele significa e como ele fica quando tudo vai bem e quando há problemas.
# The live render panel's sizes.
menu-view-live-off = Desligado
menu-view-live-off-hover = Sem painel.
menu-view-live-strip = Faixa
menu-view-live-strip-hover = Os mostradores nixie e o painel de lâmpadas.
menu-view-live-panel-size = Painel
menu-view-live-panel-size-hover = Os instrumentos: mostrador do percurso, idade da imagem, blocos, passadas da GPU, mostrador de profundidade, lâmpadas.
menu-view-live-full = Completo
menu-view-live-full-hover = O painel e a casa de máquinas: passos, GPUs, memória, núcleos, pixels.
menu-view-live-opacity = Opacidade
menu-view-watermark = Mostrar a marca d’água “Fd”
menu-view-watermark-hover = Desenhar uma discreta marca “Fd” no canto inferior direito da vista ao vivo e das imagens exportadas. Ligado por padrão.

## Color menu

menu-color = Cor
menu-color-method = Método
menu-color-palette = Paleta
menu-color-custom = Personalizada
menu-color-duotone = Duotom
menu-color-binary = Binária (conjunto)
menu-color-binary-hover = Duas cores chapadas: dentro vs. fora do conjunto, sem gradiente.
menu-color-edit-gradient = Editar gradiente…
menu-color-normalize = Normalizar cores profundas
menu-color-normalize-hover = Remapear a paleta para a faixa de escape medida na vista quando um campo denso pareceria granulação. Uma vista sem aliasing fica como está — marque “Sempre ajustar a faixa” para remapear mesmo assim. Somente com o método Iteração suave.
menu-color-more = Ciclo, deslocamento, animação e efeitos: painel Controles ▸ Coloração

## Tools menu

menu-tools = Ferramentas
# Followed by its shortcut, (M).
menu-tools-find-minibrot = Encontrar minibrot + dar zoom nele
menu-tools-find-minibrot-hover = Encaixar por Newton o centro da vista no centro exato do minibrot próximo, informar seu período e dar zoom até a escala do próprio minibrot — muitas vezes várias ordens de grandeza num único passo. Já está mais fundo que ele? Só o centro se move.
menu-tools-solver = Solucionador de Newton / Misiurewicz…
menu-tools-solver-hover = Calcular o centro exato do minibrot (período + tamanho) ou o ponto de Misiurewicz mais próximo da vista — método de Newton com precisão total, com o multiplicador repulsor λ. Abre a janela de local, que abriga os solucionadores.
# Followed by its shortcut, (A).
menu-tools-autopilot = Zoom automático (piloto automático)
menu-tools-autopilot-stop = Parar o piloto automático
menu-tools-autopilot-hover = Zoom profundo contínuo sem as mãos: mergulha em direção à região mais rica em detalhes, corrigindo o rumo pelo caminho. Qualquer comando de navegação o interrompe.
menu-tools-priority = Prioridade:
menu-tools-priority-speed = Velocidade
menu-tools-priority-speed-hover = Zoom na velocidade definida; atualizar com o primeiro quadro que tiver detalhes.
menu-tools-priority-quality = Qualidade
menu-tools-priority-quality-hover = Somente quadros totalmente resolvidos; o zoom espera por eles.
menu-tools-target = Alvo:
menu-tools-target-detail = Detalhe
menu-tools-target-detail-hover = A borda mais rica da vista, escolhida de novo à medida que o mergulho avança.
menu-tools-target-misiurewicz = Ponto de Misiurewicz
menu-tools-target-misiurewicz-hover = O ponto de Misiurewicz mais próximo, calculado uma vez e com mergulho exato nele (somente Mandelbrot).
menu-tools-orbits = Sobreposição de órbita
menu-tools-orbits-hover = Desenhar o caminho de iteração do ponto sob o cursor. Opções de animação: painel Controles ▸ Sobreposições.
menu-tools-orbits-normalize = Normalizar (ajustar à vista)
menu-tools-orbits-normalize-hover = Ajustar a órbita à vista inteira para que ela fique bem enquadrada em qualquer zoom (em vez de mapeada pela janela de visualização, onde ela sai da tela quando o zoom é profundo).
menu-tools-play-tour = Reproduzir tour…
menu-tools-play-tour-hover = Escolha um script de tour para reproduzir. O ▶ da barra de ferramentas repete o último com um único clique.
menu-tools-tour-from-view = Tour a partir da vista atual…
menu-tools-tour-from-view-hover = Criar um tour que dá zoom da vista completa até a vista atual — adicione uma legenda e defina a duração do mergulho. Salvo como um script de tour (.toml) que você pode editar, reproduzir ou renderizar.
menu-tools-render-tour = Renderizar tour…
menu-tools-render-tour-hover = Renderizar o tour carregado como uma sequência de quadros PNG (e, opcionalmente, um mp4) em um processo separado.
menu-tools-close-player = Fechar o reprodutor de tour
menu-tools-close-player-hover = Parar o tour, fechar o reprodutor e restaurar as suas próprias configurações de iteração e coloração
menu-tools-render-farm = Renderizar na fazenda…
menu-tools-render-farm-hover = Renderizar o tour carregado em várias máquinas: esta controla o trabalho, as outras se juntam com Arquivo ▸ Cliente de renderização….
menu-tools-benchmark = Benchmark…
menu-tools-benchmark-hover = Medir a velocidade de renderização — com as configurações atuais, ou uma execução padronizada (resolução + configurações fixas) comparável entre máquinas. O burn-in a repete para verificar a estabilidade / o throttling térmico.
menu-tools-last-benchmark = Mostrar o último benchmark

## Navigate menu

menu-navigate = Navegar
menu-navigate-goto = Ir para local…
menu-navigate-goto-hover = Inserir um centro/zoom, saltar para um ponto conhecido, ou resolver por Newton uma estrutura de Misiurewicz / minibrot perto da vista.
menu-navigate-misiurewicz = Explorador de Misiurewicz…
menu-navigate-misiurewicz-hover = Ver como são os pequenos tipos de Misiurewicz — miniaturas com o período de zoom e a torção de cada ponto — e resolver qualquer um deles por Newton até uma profundidade arbitrária. (Mandelbrot.)
menu-navigate-exact = Pontos exatos (expressões)
menu-navigate-exact-note = Raízes dos bulbos da cardioide principal, dadas como expressões exatas (cos/sin de uma fração de 2π). A coordenada é mantida como expressão, então o zoom permanece exatamente sobre o ponto.
menu-navigate-random = Local aleatório
menu-navigate-random-hover = Saltar para um ponto aleatório da borda, rico em detalhes
menu-navigate-import-kfr = Importar .kfr…
menu-navigate-import-kfr-hover = Carregar um arquivo de local do Kalles Fraktaler
menu-navigate-share = Compartilhar local…
menu-navigate-share-hover = Copiar / colar / salvar / carregar um local autocontido (.fdn): fractal, centro com precisão total, zoom, coloração.

## Help menu

menu-help = Ajuda
# Followed by its shortcut, (F1).
menu-help-reference = Ajuda e referência…
menu-help-welcome = Boas-vindas / início rápido…
menu-help-welcome-hover = A visão geral da primeira execução: os controles essenciais e alguns marcos.
menu-help-diagnostics = Diagnóstico…
menu-help-diagnostics-hover = Verificar se o Fractadyne funciona corretamente no seu hardware e anexar o resultado a um relato de problema
menu-help-report = Relatar um problema…
menu-help-report-hover = Abrir uma issue no GitHub (ou enviar um e-mail) com relatório de falha, log e local anexados
menu-help-accelerated = Zoom profundo mais rápido…
menu-help-accelerated-hover = Uma build opcional que calcula as órbitas de referência de zoom profundo de 2,5 a 6,4x mais rápido. Imagens idênticas; suas configurações são mantidas.
menu-help-check-updates = Verificar atualizações
# $track: "stable" or "beta", the update track in use (the variants hold the wording).
menu-help-check-updates-hover =
    { $track ->
        [beta] Procurar no GitHub uma build Beta (pré-lançamentos) mais recente
       *[other] Procurar no GitHub uma build Estável mais recente
    }
# $version: the new version; $channel: "beta" for a pre-release, "stable" otherwise (the variants hold the wording).
menu-help-update-available =
    { $channel ->
        [beta] { $version } (beta) disponível — baixar
       *[other] { $version } (estável) disponível — baixar
    }
menu-help-up-to-date = Atualizado
menu-help-tagline = Explorador de fractais nativo
menu-help-license = Licença
# $year and $holder: the copyright year and holder, as the code gives them.
menu-help-copyright = © { $year } { $holder }. Licença dupla; use-o sob qualquer uma das duas licenças, à sua escolha.
menu-help-source = Código-fonte

## The toolbar (its buttons are icons; these are their tooltips)

menu-toolbar-julia = Julia
menu-toolbar-julia-hover = Mostrar o conjunto de Julia desta fórmula, preenchendo a janela
menu-toolbar-dual-hover = Ambos, vinculados: o conjunto à esquerda, a Julia do ponto do cursor à direita
menu-toolbar-open-hover = Abrir uma vista ou local (PNG / EXR, .fdn, .kfr)
menu-toolbar-gallery-hover = Galeria
menu-toolbar-export-hover = Exportar imagem…
menu-toolbar-share-hover = Compartilhar local — copiar, colar, salvar ou carregar um .fdn autocontido (fractal, centro com precisão total, zoom, coloração)
menu-toolbar-zoom-in-hover = Aproximar
menu-toolbar-zoom-out-hover = Afastar
# $factor: the click-zoom factor (4, 10…).
menu-toolbar-click-zoom-hover = Clicar para dar zoom ({ $factor }×): o clique esquerdo mergulha no ponto, o direito recua (arrastar ainda move a vista). Fator em Controles ▸ Navegar.
# Added to the click-to-zoom tooltip in the dual view.
menu-toolbar-click-zoom-dual = Com isto ligado, fixe o c da Julia com Ctrl+clique.
menu-toolbar-autopilot-running-hover = O zoom automático está rodando — clique para parar
menu-toolbar-autopilot-hover = Zoom automático: mergulhar em direção aos detalhes (A)
menu-toolbar-reset-hover = Redefinir a vista (instantâneo)
menu-toolbar-home-hover = Afastar até a vista inicial (animado)
menu-toolbar-bookmarks-hover = Favoritos: salvar esta vista ou saltar para uma vista salva
# $name: the palette's name.
menu-toolbar-next-palette-hover = Próxima paleta ({ $name })
# $file: the last tour's file name.
menu-toolbar-play-last-hover = Reproduzir tour — { $file } (clique) · repete o último tour; escolha outro em Ferramentas ▸ Reproduzir tour…
menu-toolbar-play-hover = Reproduzir um tour… (Ferramentas ▸ Reproduzir tour)
menu-toolbar-speed-hover = Velocidade de reprodução
menu-toolbar-fullscreen-hover = Alternar tela cheia

## Dialogs: words shared by many windows

# A button that closes a window without doing what it offers.
dlg-cancel = Cancelar
# A button that dismisses a window (nothing in it is pending).
dlg-close = Fechar
# A button that copies the window's text to the clipboard (after a copy icon).
dlg-copy = Copiar
# A button that jumps to a location.
dlg-go = Ir
# A button that fills the fields from the view on screen.
dlg-use-current = Usar a atual
# A notice after a file could not be written. $error: the system's reason, as it gives it.
dlg-save-failed = Falha ao salvar: { $error }
# A notice when no graphics device is available to render with.
dlg-gpu-not-available = GPU indisponível
# The name of the plain-text file type in a save dialog (the extension .txt is added by the code).
dlg-filter-text = Texto

## Asked after a session that did not shut down cleanly

dlg-crash-title = O Fractadyne não foi encerrado corretamente
dlg-crash-intro = A sessão anterior terminou inesperadamente. Um relatório foi salvo nesta máquina — você gostaria de enviá-lo?
dlg-crash-review = Nada é enviado antes de você revisar: a próxima tela mostra exatamente o que seria incluído, e as informações do sistema podem ser excluídas com uma caixa de seleção.
dlg-crash-report = Relatar…
dlg-crash-report-hover = Abre Relatar um problema, com o relatório de falha e o local que travou anexados.
dlg-crash-not-now = Agora não
dlg-crash-dont-ask = Não perguntar de novo após uma falha
dlg-crash-dont-ask-hover = Os relatórios continuam sendo salvos localmente, e Ajuda ▸ Relatar um problema sempre funciona.

## Asked after a session that ended with a second graphics device in use

dlg-second-device-title = Desligar o segundo dispositivo gráfico?
# $device: the graphics device's name and details, as the system gives them.
dlg-second-device-intro = A sessão anterior terminou inesperadamente enquanto um segundo dispositivo gráfico estava em uso ({ $device }). Ele pode ser a causa.
dlg-second-device-note = Com ele desligado, a placa que desenha a janela renderiza tudo, como antes. Você pode ligá-lo de novo em Avançado ▸ Segunda placa de vídeo.
dlg-second-device-off = Desligar
dlg-second-device-keep = Manter ligado

## Shown when the previous session's iteration setting was set aside

dlg-quarantine-title = Configuração de iterações deixada de lado
dlg-quarantine-change = Você pode alterá-la a qualquer momento em Controles ▸ Qualidade.
dlg-quarantine-keep = Manter o padrão
# A button. $setting: the iteration setting the previous session used (such as "20,000 iterations").
dlg-quarantine-restore = Restaurar { $setting }

## Navigate ▸ Go to location

dlg-goto-title = Ir para local
# Before two choices: "x, y" (cartesian) and the polar form below.
dlg-goto-enter-as = Inserir como:
# The polar way of entering a point: an offset plus a radius r at an angle θ. Keep the symbols.
dlg-goto-polar = deslocamento + r ∠ θ
# The tooltip of the "?" button beside the entry choice.
dlg-goto-help-hover = O que os campos aceitam: frações, potências, sqrt/sin/cos…, pi e outras constantes — abre Ajuda → Expressões de coordenadas
# Labels over the entry fields. x0, y0, r, θ, Re and Im are the fields' symbols: keep them.
dlg-goto-polar-offset = Centro do deslocamento  (x0, y0 — deixe 0 para um ponto puramente polar)
dlg-goto-polar-radius = Raio r, ângulo θ
dlg-goto-re = Centro — real (Re)
dlg-goto-im = Centro — imaginário (Im)
dlg-goto-zoom = Zoom (ampliação)
# A warning under the zoom field, after a ⚠ sign. $have: the iteration count set; $typical: the count this depth typically needs.
dlg-goto-iter-warning = As iterações estão fixas em { $have }; esta profundidade normalmente precisa de ~{ $typical }. A vista será renderizada chapada até que esse valor seja aumentado — ou ative as iterações automáticas (Controles ▸ Qualidade), que se ajustam sozinhas.
dlg-goto-copy-hover = Copiar este local para a área de transferência
dlg-goto-cancel-hover = Fechar sem ir a lugar nenhum
# The expressions in brackets are examples of what can be typed: keep them as they are.
dlg-goto-paste-note = Cole um centro/zoom de outra pessoa, ou use Copiar para compartilhar. As coordenadas aceitam expressões: frações exatas (-3/4), funções e constantes (-0.5 + 0.25*cos(pi/4), sqrt, root(x,n), ^; ângulos em radianos), e o campo real aceita um valor complexo inteiro ((37+16i)/100), que preenche os dois.
# A section that opens and closes.
dlg-goto-feature = Ir para uma estrutura (Mandelbrot)
dlg-goto-feature-poi = Pontos conhecidos:
# The dropdown's text before a point is picked.
dlg-goto-feature-poi-pick = Saltar para um ponto de interesse…
dlg-goto-feature-solve = Ou resolva uma estrutura perto da vista atual:
# Two choices: what to solve for. Misiurewicz is a mathematician's name (a Misiurewicz point).
dlg-goto-feature-misiurewicz = Misiurewicz
dlg-goto-feature-minibrot = Minibrot mais próximo
# Labels of two small number fields; k and p are their symbols.
dlg-goto-feature-preperiod = Pré-período k
dlg-goto-feature-period = Período p
# The faint text in an empty field: left empty, the value is found automatically.
dlg-goto-feature-auto-hint = auto
# A small button that empties both fields, so they are found automatically again.
dlg-goto-feature-auto = Auto
dlg-goto-feature-auto-hover = Limpar as duas caixas para que a próxima busca detecte o pré-período e o período a partir da órbita no centro da vista.
dlg-goto-feature-kp-hover = Deixe os dois em branco para detectá-los a partir da órbita no centro da vista; os valores encontrados são preenchidos aqui. Ninguém que olha uma espiral sabe o pré-período dela, e ter de fornecer um era o que tornava este solucionador inutilizável na prática.
# While a solve runs, followed by its elapsed time and the note below.
dlg-goto-feature-solving = Resolvendo…
dlg-goto-feature-solving-note = (precisão arbitrária; vistas profundas demoram mais)
dlg-goto-feature-cancel-hover = Parar de esperar. O cálculo em si continua até o fim em segundo plano e a resposta é descartada — nada fica pela metade.
dlg-goto-feature-find = Encontrar perto da vista
dlg-goto-feature-find-hover =
    Resolver por Newton a partir do centro atual até a estrutura exata (precisão arbitrária). Navegue para perto da estrutura primeiro.

    Defina no campo Zoom acima a profundidade que você QUER antes de resolver: o ponto é calculado com a precisão que essa profundidade exige, e a vista salta direto para lá. É um único cálculo em vez de uma longa sequência de passos manuais de zoom.
dlg-goto-solve-cancelled = Cálculo cancelado.

## Snapshot (the camera button): its tooltip, and the choice on the first press

# The camera button's tooltip, one per Snapshot setting. Followed by its shortcut, (Ctrl+S).
dlg-snapshot-hover-ask = Captura — salvar a vista como é mostrada, ou uma renderização completa; o primeiro clique pergunta
dlg-snapshot-hover-screen = Captura — salvar a vista exatamente como é mostrada, na última pasta de exportação
dlg-snapshot-hover-render = Captura — renderização completa com as configurações de exportação, na última pasta de exportação
# The window's title.
dlg-snapshot-title = Captura
dlg-snapshot-question = O que a Captura (o botão da câmera, Ctrl+S) deve salvar?
# The first choice, and a short tag beside it.
dlg-snapshot-screen = Captura de tela
dlg-snapshot-screen-tag = instantâneo
dlg-snapshot-screen-note = Salva a vista exatamente como aparece na tela, na resolução da tela, como PNG na última pasta de exportação. O que você vê é o que você obtém.
# The second choice, and a short tag beside it.
dlg-snapshot-render = Renderização completa
dlg-snapshot-render-tag = pode demorar muito
# $size: the image size, as 1920×1080; $ss: the supersampling factor (1–4); $format: PNG or EXR.
dlg-snapshot-render-note = Renderiza com as configurações de exportação — { $size }, superamostragem { $ss }×, { $format }. Melhor qualidade, mas uma vista profunda pode levar de minutos a horas. Roda em segundo plano; Arquivo → Exportar imagem… mostra o progresso e permite cancelá-la.
dlg-snapshot-remember = Lembrar minha escolha e não perguntar de novo
dlg-snapshot-change = Você pode mudar isso a qualquer momento em Arquivo → Configurações → Captura.

## Share location (.fdn)

dlg-share-title = Compartilhar local
dlg-share-intro = Um local autocontido (fractal, centro com precisão total, zoom, coloração). Copie-o para compartilhar, ou cole/carregue um e clique em Aplicar.
dlg-share-thumb = Incluir uma miniatura no arquivo salvo
dlg-share-thumb-hover = Salvar .fdn incorpora uma pequena imagem desta vista para que o arquivo possa ser reconhecido na galeria. Acrescenta cerca de 55 KB e não faz parte do texto acima.
dlg-share-copy-hover = Copiar o texto para a área de transferência
# Buttons (the first after a save icon).
dlg-share-save = Salvar .fdn…
dlg-share-load = Carregar .fdn…
dlg-share-apply = Aplicar
dlg-share-apply-hover = Saltar para o local na caixa
dlg-share-copied = Copiado para a área de transferência.

## Help ▸ Report an issue (the report itself stays in English: it is read by the maintainer)

dlg-report-title = Relatar um problema
dlg-report-intro = O melhor lugar para relatar é uma issue no GitHub — é pública e pesquisável, e você verá quando o problema for corrigido. E-mail também funciona. Revise tudo abaixo primeiro: nada sai da sua máquina até que você aja.
# Labels before three dropdowns.
dlg-report-type = Tipo:
dlg-report-severity = Gravidade:
dlg-report-repro = Reproduzível:
dlg-report-describe = Descreva o que aconteceu:
# The faint text in the empty description box.
dlg-report-describe-hint = O que aconteceu, os passos para reproduzir e o que você esperava em vez disso.
# Before the checkboxes of what the report carries.
dlg-report-include = Incluir:
dlg-report-sysinfo = Informações do sistema (versão, SO, CPU, GPU, VRAM)
dlg-report-location = Local atual (.fdn)
dlg-report-log = Log recente
dlg-report-frames = Registro de quadros (o último minuto e os últimos 40 quadros)
dlg-report-crash = Relatório de falha mais recente
dlg-report-crash-none = Relatório de falha mais recente (nenhum encontrado)
dlg-report-crash-view = Local que travou (.fdn)
dlg-report-crash-view-none = Local que travou (.fdn) (nenhum encontrado)
dlg-report-crash-view-hover = A vista exata em que ocorreu uma falha ou uma perda do dispositivo gráfico — a única coisa que permite ao mantenedor reproduzi-la. Registrada automaticamente.
dlg-report-test = Resultado do teste de diagnóstico
dlg-report-test-none = Resultado do teste de diagnóstico (execute um em Ajuda → Diagnóstico…)
dlg-report-test-hover = Um aprovado/reprovado validado pela máquina, vindo do seu próprio hardware — muito mais útil para um mantenedor do que apenas uma descrição
# A section that opens and closes. $size: the report's size, as "12 KB".
dlg-report-preview = Pré-visualizar o que será enviado  ({ $size })
dlg-report-github = Abrir uma issue no GitHub
dlg-report-github-hover = Abrir uma página de nova issue com o título preenchido; o relatório é copiado para a área de transferência — cole-o no corpo da issue (Ctrl+V).
# Buttons (after a copy and a save icon).
dlg-report-copy = Copiar relatório
dlg-report-copy-hover = Copiar tudo para a área de transferência
dlg-report-save = Salvar relatório…
dlg-report-save-hover = Salvar como .txt para anexar
dlg-report-by-email = Ou por e-mail:
dlg-report-gmail = Escrever no Gmail
# $address: the email address reports go to.
dlg-report-gmail-hover = Abrir uma janela de nova mensagem do Gmail para { $address } com o assunto preenchido; cole o relatório copiado
dlg-report-email = Aplicativo de e-mail…
dlg-report-email-hover = Abrir o seu aplicativo de e-mail padrão (mailto) — requer um gerenciador de e-mail configurado
dlg-report-tip = Dica: com qualquer um destes, o relatório é copiado para a área de transferência — cole-o no corpo da issue ou da mensagem (Ctrl+V). Se puder, anexe também uma captura de tela ou uma imagem de exemplo (arraste-a para lá); isso ajuda muito em problemas de renderização e de interface.
# Notices in the window after a button is pressed.
dlg-report-opened-github = GitHub aberto — o relatório está na área de transferência; cole-o na issue.
dlg-report-copied = Relatório copiado para a área de transferência.
# $path: where the report was saved.
dlg-report-saved = Salvo em { $path }
dlg-report-opened-email = Seu aplicativo de e-mail foi aberto — o relatório está na área de transferência; cole-o lá.
dlg-report-opened-gmail = Nova mensagem do Gmail aberta — cole o relatório copiado no corpo (Ctrl+V).

## A location whose checksum does not match

dlg-checksum-title = Este local pode estar danificado
# After a ⚠ sign.
dlg-checksum-mismatch = A soma de verificação dele não corresponde aos dados ao lado.
dlg-checksum-why = Isso geralmente significa que o texto foi cortado ou alterado no caminho até aqui — uma cópia parcial tirada de uma janela de chat ou de uma postagem de fórum é a causa comum.
# A label over where the location came from (a file path, or the clipboard).
dlg-checksum-source = Origem
# The two checksums, side by side: the one written in the file, and the one computed from its data.
dlg-checksum-in-file = no arquivo
dlg-checksum-from-data = pelos dados
dlg-checksum-load-note = Você pode carregá-lo mesmo assim — as coordenadas ainda podem estar corretas, e nada é sobrescrito. Qualquer coisa ilegível é informada após o carregamento.
dlg-checksum-load = Carregar mesmo assim
dlg-checksum-load-hover = Saltar para ele apesar da divergência
# Notices after loading it. $zoom: the magnification, as 1.2e148.
dlg-checksum-loaded = Carregado @ { $zoom }× apesar da divergência na soma de verificação
# $zoom: the magnification, as 1.2e148; $note: what the loader found wrong with the location.
dlg-checksum-loaded-note = Carregado @ { $zoom }× — { $note }
dlg-checksum-not-loaded = Não carregado.

## File ▸ Reset application state

dlg-reset-title = Redefinir estado do aplicativo
# After a ⚠ sign, followed by a list.
dlg-reset-warning = Isto exclui permanentemente todos os dados salvos do Fractadyne:
# The list's items, each after a bullet.
dlg-reset-session = a sessão salva (vista atual, coloração, preferências)
dlg-reset-bookmarks = todos os favoritos e suas miniaturas
dlg-reset-refcache = o cache de órbitas de referência (reconstruído conforme você revisita os locais)
# A label over the folder that is deleted.
dlg-reset-location = Localização
dlg-reset-note = Isto não pode ser desfeito. A sessão atual não será salva de novo ao sair, então os padrões serão carregados na próxima inicialização.
dlg-reset-confirm = Redefinir tudo
dlg-reset-done = Estado do aplicativo redefinido — os padrões serão carregados na próxima inicialização.
# $error: the system's reason, as it gives it.
dlg-reset-failed = Falha ao redefinir: { $error }

## Tools ▸ Tour from current view

dlg-tour-title = Tour a partir da vista atual
dlg-tour-intro = Cria um tour que dá zoom da vista completa até onde você está agora.
# A label before the fractal and magnification the tour ends at.
dlg-tour-destination = Destino
dlg-tour-notation = Anotação (legenda opcional mostrada durante o mergulho)
# The faint example text in the empty caption box.
dlg-tour-notation-hint = ex.: Mergulhando no Misiurewicz de três braços…
dlg-tour-duration = Duração do zoom
# $rate: how fast the dive zooms, in powers of ten a second (as 2.5).
dlg-tour-rate = ≈ { $rate } décadas/s
# A small button that puts the duration back to its default.
dlg-tour-reset = Redefinir
dlg-tour-reset-hover = Padrão baseado na profundidade
dlg-tour-duration-note = O padrão aumenta com a profundidade para que o mergulho continue assistível. Mergulhos mais rápidos podem piscar quadros de uma só cor em zoom extremo.
# A button, after a save icon.
dlg-tour-save = Salvar tour…
# The file type's name in the save dialog.
dlg-tour-filter = Script do Fractadyne (TOML)
# A notice. $file: the saved file's name. --render-tour is a command-line option: keep it.
dlg-tour-saved = Tour salvo → { $file }. Reproduza-o em Ferramentas → Reproduzir tour, ou renderize-o com --render-tour.

## Navigate ▸ Bookmarks ▸ Manage…

dlg-bookmarks-title = Favoritos
# The faint text in the empty name box.
dlg-bookmarks-name-hint = nome (opcional)
# A button, after a bookmark icon.
dlg-bookmarks-add = Adicionar a vista atual
dlg-bookmarks-empty = Nenhum favorito ainda. Adicione um a partir da vista atual.
# The tooltip of a bookmark's delete (bin) button.
dlg-bookmarks-delete-hover = Excluir

## Tools ▸ Benchmark…

dlg-bench-title = Benchmark
dlg-bench-mode = Modo
dlg-bench-current = Configurações atuais
dlg-bench-current-hover = Reproduzir o tour fixo de zoom profundo na vista ao vivo, usando a sua resolução e as suas configurações atuais.
dlg-bench-standard = Padronizado
dlg-bench-standard-hover = Fixar a resolução + todas as configurações de renderização para que a pontuação seja comparável entre máquinas.
dlg-bench-resolution = Resolução
dlg-bench-depth = Profundidade
# Standard and Ultra are the two depths in the dropdown this describes.
dlg-bench-depth-hover = Os dois mergulhos entram no caminho profundo floatexp/BLA; o Ultraprofundo (1e48×) passa muito mais tempo ali que o Padrão (1e32×), e por isso demora proporcionalmente mais para rodar.
dlg-bench-burnin = Burn-in (repetir)
dlg-bench-burnin-hover = Executar o benchmark repetidamente para revelar a estabilidade e o throttling térmico.
# The label of a slider: how many times the benchmark runs.
dlg-bench-passes = rodadas
# $size: the image size, as 1920×1080; $ss: the supersampling factor; $depth: the dive's final magnification, as 1e32.
dlg-bench-std-summary = Renderiza fora da tela em { $size }, { $ss }× SS, Mandelbrot/suave, mergulho de 60 quadros até { $depth }×.
dlg-bench-run = Executar

## The standardized benchmark's progress window

dlg-bench-progress-title = Executando o benchmark…
# $resolution: the benchmark's resolution, as the dropdown names it.
dlg-bench-progress-mode = Padronizado · { $resolution }
# $pass: the pass running now; $total: how many passes there are.
dlg-bench-progress-pass = Rodada de burn-in { $pass }/{ $total }
dlg-bench-progress-dive = Renderizando o mergulho fixo…
# On the progress bar. $done: frames rendered; $total: frames in the dive.
dlg-bench-progress-frame = quadro { $done }/{ $total }
# $fps: frames per second, as 41.3.
dlg-bench-progress-last = última rodada: { $fps } fps

## The benchmark's results

dlg-bench-results-title = Resultados do benchmark
# Buttons, each after an icon.
dlg-bench-results-run-again = Executar de novo…
dlg-bench-results-save = Salvar…
dlg-bench-results-none = Nenhum benchmark foi executado ainda.
dlg-bench-results-saved = Benchmark salvo.

## The accelerated build's notice when its arithmetic libraries are missing

dlg-backend-title = Build acelerada — usando a aritmética integrada
dlg-backend-dont-show = Não mostrar isto de novo

## File ▸ Gallery…

dlg-gallery-title = Galeria
# A button that picks the folder to show.
dlg-gallery-folder = Pasta…
# A button, after a refresh icon.
dlg-gallery-refresh = Atualizar
dlg-gallery-empty = Nenhuma imagem do Fractadyne nesta pasta.
# An image's title when it does not name its fractal.
dlg-gallery-untitled = Imagem do Fractadyne
# $zoom: the image's magnification, as 1.2e148; $saved: when it was saved, as the file gives it.
dlg-gallery-zoom = zoom { $zoom }   { $saved }
# The notes saved with the image, in quotation marks. Use your language's quotation marks.
dlg-gallery-notes = “{ $notes }”
dlg-gallery-open = Abrir esta vista
dlg-gallery-loaded = Vista carregada da galeria.

## File ▸ Export image…

dlg-export-title = Exportar imagem
# A dropdown of image sizes.
dlg-export-size = Tamanho
# The size dropdown's text for a size set by hand. $size: the size, as 1920×1080.
dlg-export-size-custom-current = Personalizado — { $size }
# The size dropdown's first choice.
dlg-export-size-custom = Personalizado…
dlg-export-size-custom-hover = Defina a largura você mesmo; a altura segue a Proporção abaixo.
dlg-export-width = Largura (px)
# After the width field. $height: the height in pixels, which follows from the aspect.
dlg-export-height-from-aspect = × { $height } (pela Proporção)
dlg-export-ss = Superamostragem
dlg-export-aspect = Proporção
# The aspect choice that keeps the window's own shape.
dlg-export-aspect-window = Igual à janela
# Before the choices PNG and OpenEXR.
dlg-export-format = Formato:
dlg-export-hud = HUD de local
dlg-export-hud-hover = Gravar um painel com nível de zoom + coordenadas no canto superior esquerdo da imagem exportada (acompanha a escala da saída; mostra o zoom + o centro da vista do mapa/Mandelbrot).
dlg-export-glitch = Correção de glitches
dlg-export-glitch-hover = Correção de glitches de perturbação com várias referências. Pulada automaticamente em exportações de vista única muito profundas (floatexp), para que a construção da referência + a renderização rodem fora da thread principal e o aplicativo continue respondendo. Nunca usada em vistas de Mandelbrot ou Multibrot (fora do modo Julia): comparada com um cálculo exato, ela não corrigiu nada ali e custou até 5x o tempo de exportação.
dlg-export-deep-note = Exportação profunda: a referência é construída fora da thread principal (a interface continua ativa); a correção de glitches é pulada nesta profundidade.
# A dropdown, shown in the dual (map + Julia) view, and its three choices.
dlg-export-dual = Layout duplo
dlg-export-dual-side = Lado a lado
dlg-export-dual-separate = Arquivos separados
dlg-export-dual-map = Só o mapa
dlg-export-notes = Notas:
# The faint text in the empty notes box.
dlg-export-notes-hint = salvas com a imagem (≤120 caracteres)
# $width and $height: the image size in pixels; $left: how many characters the notes can still take.
dlg-export-output = Saída: { $width } × { $height } px   (restam { $left } caracteres)
# When the export's shape differs from the window's. $percent: how much more, as 25.
dlg-export-extra-width = Mostra { $percent }% a mais de largura que a janela (nada é cortado)
# The same, when the export is taller than the window. $percent: how much more, as 25.
dlg-export-extra-height = Mostra { $percent }% a mais de altura que a janela (nada é cortado)
dlg-export-tiles-note = Renderizada em blocos (sem limite de tamanho) numa thread em segundo plano. O arquivo incorpora a vista + as notas, para que possa ser reaberto em Arquivo ▸ Abrir vista ou local….
# A label before the button that picks the folder exports go to.
dlg-export-folder = Pasta:
dlg-export-choose = Escolher…
dlg-export-choose-hover = Escolher a pasta de destino das exportações
dlg-export-open-after = Abrir ao terminar
dlg-export-open-after-hover = Entregar a imagem pronta ao visualizador do sistema assim que ela for gravada.
# $count: the graphics cards in this machine (2 or more).
dlg-export-all-gpus = Usar todas as placas de vídeo ({ $count })
dlg-export-all-gpus-hover = Dividir a imagem por blocos entre todas as placas de vídeo desta máquina, cada uma com sua própria cópia da referência profunda. Placas idênticas dão exatamente a imagem que uma delas renderiza sozinha; modelos diferentes desenham alguns pixels de forma ligeiramente diferente, bloco a bloco (medido: nenhuma emenda visível). Se uma placa falhar, os blocos dela vão para as outras. As imagens que a divisão não comporta (alguns métodos de coloração e fórmulas) usam a placa desta janela.
# The export's progress.
dlg-export-preparing = Preparando (construindo a referência)…
dlg-export-saving = Salvando…
dlg-export-rendering = Renderizando…
# $time: the time since the export started, as 8.3s or 1m 04.0s.
dlg-export-elapsed = Decorrido: { $time }
dlg-export-cancel-hover = Fechar sem exportar
# A button, after a save icon.
dlg-export-save-as = Salvar como…
dlg-export-save-as-hover = Escolher o nome e o local do arquivo
dlg-export-export = Exportar
dlg-export-export-hover = Renderizar e salvar na pasta acima (nome automático)

## Notices worded at startup (main.rs)

# A notice shown once after starting, when the saved session comes from a newer version of the app. $saved and $handled: version numbers of the session file's format (1, 2…).
app-state-newer = Esta sessão foi salva por um Fractadyne mais recente (estado v{ $saved }; esta build lida com v{ $handled }). Algumas configurações podem não se aplicar, e salvar vai regravá-la no formato desta build.
# The title of the notice that says the accelerated build's arithmetic libraries are found again (its text comes from elsewhere).
app-notice-mpfr-restored-title = Aritmética acelerada restaurada
# After Fractadyne restarted itself because the graphics driver reset the GPU.
app-toast-gpu-recovered = Recuperado de uma redefinição do dispositivo gráfico — reaberto na vista inicial, pois o local anterior era exigente demais para renderizar.

## Short notices (toasts)

# When the autopilot reached a view that could not be rendered and put the last good view back.
app-toast-autopilot-view-restored = Piloto automático parado: a vista ficou inutilizável e foi restaurada (registrado no log)
# $error: the system's reason, as it gives it (often in English).
app-toast-gradients-save-failed = Não foi possível salvar os gradientes: { $error }
# $error: the reason, as the program gives it (often in English).
app-toast-gradients-serialize-failed = Não foi possível serializar os gradientes: { $error }
# $name: the name the user gave the gradient.
app-toast-gradient-saved = Gradiente “{ $name }” salvo.
# $error: the reason, as the program gives it (often in English).
app-toast-bookmarks-serialize-failed = Não foi possível serializar os favoritos: { $error }
# $error: the system's reason, as it gives it (often in English).
app-toast-bookmarks-save-failed = Não foi possível salvar os favoritos: { $error }
# Jumping to a named point given by exact expressions. $name: the point's name (in English).
app-toast-coordinates-failed = { $name }: não foi possível avaliar as coordenadas
# $zoom: the magnification, already formatted (1,234.5 or 1.2e148).
app-toast-random-location = Local aleatório · { $zoom }×
# Importing a Kalles Fraktaler .kfr location. $error: the reason (in English).
app-toast-import-failed = Falha na importação: { $error }
# $zoom: the magnification, already formatted.
app-toast-location-loaded = Local carregado @ { $zoom }×

## Go to location: why a jump could not be made (one line per field that failed)

# "Re" is the dialog's real-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-re = Re: { $error }
# "Im" is the dialog's imaginary-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-im = Im: { $error }
# The imaginary field was given a complex value. $error: the evaluator's reason (in English).
app-goto-error-im-real = Im: { $error } — digite a parte imaginária como um número real, sem o i
# "Zoom" is the dialog's magnification field.
app-goto-error-zoom-empty = Zoom: insira uma ampliação, por exemplo 1e50 ou 250000
# $zoom: what was typed, in quotes.
app-goto-error-zoom = Zoom: não foi possível ler { $zoom } — insira uma ampliação positiva, como 1e50 ou 250000
app-goto-error-invalid = Entrada inválida — verifique as coordenadas e o zoom.

## Finding a minibrot or Misiurewicz point (the M key and the Go to dialog)

app-feature-needs-holomorphic = O localizador de minibrots precisa de uma família holomorfa (Mandelbrot / Multibrot).
app-feature-not-in-julia = O localizador de minibrots busca no conjunto de Mandelbrot — desligue o modo Julia para usá-lo.
app-feature-busy = Já há uma busca de estrutura em andamento — uma de cada vez.
app-feature-searching = Procurando o minibrot mais próximo…
app-feature-mandelbrot-only = A busca de estruturas é exclusiva do Mandelbrot.
app-feature-no-pair = Nenhum ponto de Misiurewicz encontrado dentro do orçamento de busca — o ponto pode estar mais fundo do que o localizador alcança nesta vista, ou o centro pode não estar perto de uma espiral ou de um ponto de ramificação.
# The name of what was found, used as $feature in the messages below. $period: the minibrot's period (a whole number).
app-feature-minibrot = minibrot de período { $period }
# $preperiod and $period: the point's two whole numbers (k, p). $octaves: how often the view repeats when zooming in (2 decimals); $twist: the turn per repeat in degrees.
app-feature-misiurewicz-lambda = Misiurewicz ({ $preperiod },{ $period }) — repete-se a cada { $octaves } oitavas, torção de { $twist }°
# $preperiod and $period: the point's two whole numbers (k, p).
app-feature-misiurewicz = Misiurewicz ({ $preperiod },{ $period })
# Added after the messages below when the fixed iteration count is too low for the new depth. $have: the iterations set now; $typical: what the depth typically needs.
app-feature-iter-note = ⚠ As iterações estão fixas em { $have }; esta profundidade normalmente precisa de ~{ $typical } — aumente-as (ou ative as iterações automáticas) se a vista for renderizada chapada.
# $feature: what was found (see above). $zoom and $asked: magnifications, already formatted; $octaves: the solver's depth limit.
app-feature-zoomed-capped = Zoom feito até o { $feature } — { $zoom }×. O cálculo é limitado a { $octaves } oitavas, então ele parou ali em vez de em { $asked }×.
# $feature: what was found. $zoom: the magnification, already formatted. $seconds: how long the search took (1 decimal).
app-feature-zoomed = Zoom feito até o { $feature } — { $zoom }× ({ $seconds }s)
# $feature: what was found. $seconds: how long the search took (1 decimal).
app-feature-snapped = Encaixado no centro do { $feature } ({ $seconds }s)
app-feature-no-minibrot = Nenhum centro de minibrot encontrado perto da vista — aproxime-se mais de um.
# $feature: the point asked for, e.g. Misiurewicz (3,1). $widths: how many view-widths away, already formatted. $octaves: how far to zoom out.
app-feature-just-outside = Encontrado um ponto de { $feature } logo fora desta vista — a { $widths } larguras de vista de distância. Afaste o zoom cerca de { $octaves } oitavas para trazê-lo à tela, ou defina um pré-período maior para uma estrutura nesta escala.
# $feature: the point asked for. $widths: how many view-widths away, already formatted (may be 2^187). $octaves: how far to zoom out.
app-feature-far = Encontrado um ponto de { $feature }, mas ele está a { $widths } larguras de vista de distância — essa estrutura está numa escala muito MAIOR que esta vista. AFASTE o zoom cerca de { $octaves } oitavas para alcançá-la, ou tente um pré-período maior para uma estrutura nesta escala.
# $feature: the point asked for. $residual: a small number in scientific notation.
app-feature-not-preperiodic = { $feature } não se ajusta à órbita aqui (resíduo { $residual }) — limpe as duas caixas para detectar o par a partir da vista.
# $feature: the point asked for. "k/p" are the dialog's two fields.
app-feature-not-converged = Nenhum ponto de { $feature } convergiu perto da vista — navegue para mais perto, ou tente outros k/p.
app-feature-bad-request = O pré-período e o período devem ser ambos positivos.

## Share location dialog and location files

app-location-empty = Nada para carregar (ou texto grande demais).
app-location-not-fractadyne = Não é um local do Fractadyne.
# $zoom: the magnification, already formatted. $note: what in the text was ignored or repaired (in English).
app-share-loaded-note = Carregado @ { $zoom }× — { $note }.
# $error: the system's reason (often in English).
app-share-save-failed = Falha ao salvar: { $error }
app-share-saved = Salvo.
app-share-saved-rendering-thumb = Salvo. Renderizando a miniatura…
app-share-saved-no-thumb = Salvo — sem miniatura (não foi possível renderizar uma para esta vista).
app-share-saved-with-thumb = Salvo, com miniatura.
# $error: the system's reason (often in English).
app-share-saved-thumb-failed = Salvo, mas não foi possível adicionar a miniatura: { $error }
app-share-loaded-into-box = Carregado na caixa — revise e depois clique em Aplicar.
# Reading a file failed (a location or a palette). $error: the system's reason (often in English).
app-read-failed = Falha na leitura: { $error }
app-share-too-large = Arquivo grande demais (não é um local .fdn?).

## File-dialog filter names (the extensions are added by the code)

app-filter-kfr = Local do Kalles Fraktaler
app-filter-fdn = Local do Fractadyne
app-filter-map = Paleta do Fractint / KF
app-filter-ugr = Gradiente do Ultra Fractal
app-filter-ase = Adobe Swatch Exchange
app-filter-ggr = Gradiente do GIMP
app-filter-all = Todos os arquivos

## Importing and saving palette files (shown under the gradient editor)

# The reason a file was refused before reading it.
app-gradient-map-too-large = grande demais para ser uma paleta .map
app-gradient-ugr-too-large = grande demais para ser um arquivo de paleta .ugr
app-gradient-ase-too-large = grande demais para ser uma lista de amostras .ase
app-gradient-ggr-too-large = grande demais para ser um gradiente .ggr
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported-vga = { $count } entradas importadas de { $file } - uma tabela VGA de 6 bits (o branco é 252, não 255), mantida como está escrita para corresponder à saída do próprio Fractint.
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported = { $count } entradas importadas de { $file }, como faixas rígidas.
# $error: the reader's reason (in English).
app-gradient-map-invalid = Não é uma paleta .map: { $error }
# $name: the gradient's name in the file; $file: the file's name.
app-gradient-ugr-imported = “{ $name }” importado de { $file }.
# $file: the file's name; $count: how many gradients it holds.
app-gradient-ugr-pick = { $file } contém { $count } gradientes — escolha um abaixo.
# $error: the reader's reason (in English).
app-gradient-ugr-invalid = Não é um arquivo de paleta .ugr: { $error }
# $used: the swatches kept; $total: the swatches in the file.
app-gradient-ase-imported-some = { $used } de { $total } amostras importadas, espaçadas por igual.
# $count: the swatches in the file.
app-gradient-ase-imported = { $count } amostras importadas, espaçadas por igual e mescladas.
# $error: the reader's reason (in English).
app-gradient-ase-invalid = Não é uma lista de amostras .ase: { $error }
app-gradient-nothing-to-save = Ainda não há gradiente personalizado para salvar.
# $file: the saved file's name.
app-gradient-ggr-saved = “{ $file }” salvo — pontos médios, curvas de mistura e espaços de cor, tudo intacto.
# $file: the saved file's name; $count: the segments whose curve could not be kept.
app-gradient-ggr-saved-lossy = “{ $file }” salvo — mas { $count } segmento(s) usam uma curva de Bézier, que o formato .ggr não consegue expressar. Eles foram gravados como misturas retas; a cópia na sua biblioteca mantém as curvas.
# $path: the file; $error: the system's reason (often in English).
app-gradient-write-failed = Não foi possível gravar { $path }: { $error }
# $name: the gradient's name; $count: its segments.
app-gradient-ggr-imported = “{ $name }” importado - { $count } segmentos, com seus pontos médios, curvas de mistura e espaços de cor.
# $error: the reader's reason (in English).
app-gradient-ggr-invalid = Não é um gradiente .ggr: { $error }

## The eyedropper's banner across the top of the screen while picking a colour

# $hex: the colour under the pointer, as #rrggbb.
app-eyedropper-banner-color = Escolhendo uma cor  ·  { $hex }  ·  clique para pegá-la, Esc para cancelar
app-eyedropper-banner = Escolhendo uma cor  ·  clique para pegá-la, Esc para cancelar

## The gradient editor window

app-gradient-title = Editor de gradiente
# Before the two view buttons, Bar and Ring.
app-gradient-view = Exibição
app-gradient-view-bar = Barra
app-gradient-view-bar-hover = Uma faixa reta — melhor para posicionar as paradas com precisão
app-gradient-view-ring = Anel
app-gradient-view-ring-hover = Um anel, porque a paleta é CÍCLICA: a junção no topo é a emenda que o renderizador cruza a cada volta, e um gradiente que parece bom numa barra pode ter uma borda dura ali.
app-gradient-add-lane-hover = Clique em qualquer ponto desta linha para adicionar uma parada ali. O + adiciona uma no maior intervalo.
app-gradient-strip-hover = Arraste um marcador para mover sua parada · arraste a própria faixa para GIRAR o gradiente inteiro, dando a volta na emenda · clique duas vezes para adicionar uma parada, clique com o botão direito num marcador para removê-lo · as setas do teclado deslocam a seleção (Shift para um passo fino).
app-gradient-remove-selected-hover = Remover a parada selecionada
app-gradient-ring-hover = Arraste o próprio anel para GIRAR o gradiente inteiro — o que passa pela emenda volta pelo outro lado · arraste um marcador para mover sua parada · clique na trilha externa para adicionar uma parada ali, ou em ⊕ no meio para adicionar uma no maior intervalo · ⊖ remove a parada selecionada. A marca vazada no topo é a emenda, onde uma paleta cíclica dá a volta; ela fica parada porque ELA É a junção, e as cores giram sob ela.
app-gradient-empty = Nenhum gradiente personalizado ainda — “Copiar predefinição…”, uma importação ou um gradiente salvo abaixo inicia um, e suas paradas e curvas por segmento aparecem aqui.
# $count: the entries of an imported palette, more than the editor shows stop by stop.
app-gradient-too-many = { $count } entradas importadas - demais para editar parada por parada. Use a pré-visualização acima, ou Copiar predefinição para recomeçar.
# The selected segment's heading. $index: its number; $total: how many there are; $left and $right: where it starts and ends, 0.000–1.000.
app-gradient-segment-heading = Segmento { $index } de { $total }  ·  { $left } → { $right }
# A label before the list of blend curves.
app-gradient-curve = Curva
app-gradient-curve-hover = A rapidez com que a cor percorre este segmento. “Curvada” é moldada pelo ponto médio — com o ponto médio em 0.50, ela é exatamente linear.
# The blend curves (GIMP's kinds), in a narrow list: keep them short.
app-gradient-blend-linear = Linear
app-gradient-blend-curved = Curvada
app-gradient-blend-sine = Seno
app-gradient-blend-sphere-increasing = Esfera ↑
app-gradient-blend-sphere-decreasing = Esfera ↓
app-gradient-blend-bezier = Bézier
# Hover on Bézier in the list of blend curves.
app-gradient-bezier-exact-hover = Dois pontos de controle que você pode arrastar nas duas direções. Esta curva já é cúbica, então convertê-la não muda nada.
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-visible-hover = Dois pontos de controle que você pode arrastar nas duas direções. Converter APROXIMA a curva atual — esta, com erro de até { $error }, o que é claramente visível.
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-barely-hover = Dois pontos de controle que você pode arrastar nas duas direções. Converter APROXIMA a curva atual — esta, com erro de até { $error }, o que é quase imperceptível.
# A label before the list of colour spaces (RGB, HSV).
app-gradient-space = Espaço
# A label before the Bézier presets beside it.
app-gradient-ease = Suavização
app-gradient-ease-hover = As predefinições gravam os dois pontos de controle; arraste-os na curva para qualquer outra coisa. Uma Bézier não tem ponto médio — suas alças dizem onde ela chega à metade.
# Bézier presets, as in CSS easing: short buttons.
app-gradient-ease-linear = Linear
app-gradient-ease-in = Entrada
app-gradient-ease-out = Saída
app-gradient-ease-in-out = Entrada-saída
app-gradient-midpoint = Ponto médio
app-gradient-midpoint-hover = Onde a mistura chega à metade, como fração deste segmento. 0.50 é o centro.
# A button that puts the midpoint back in the middle of the segment.
app-gradient-centre = Centralizar
app-gradient-centre-hover = Voltar a 0.50
app-gradient-no-hue = ⚠ uma ponta deste segmento não tem matiz
app-gradient-no-hue-hover = Uma ponta é preta, branca ou cinza, então a varredura começa no vermelho e percorre a roda de cores inteira. Isso é válido e às vezes desejado — mas é por isso que o meio pode ter uma cor que nenhuma das pontas contém. A faixa sob a curva mostra o que ela faz.
app-gradient-segment-start-hover = Cor no INÍCIO deste segmento
app-gradient-segment-end-hover = Cor no FIM deste segmento
app-gradient-bytes-hover = Vermelho, verde e azul como 0–255
app-gradient-floats-hover = A mesma cor como 0–1, que é o que o renderizador armazena
app-gradient-canvas-bezier-hover = A curva de mistura do segmento selecionado. Arraste qualquer um dos pontos de controle — em x E em y — para remodelá-la. As linhas claras são 0 e 1; uma curva pode passar fora delas, e é a cor que é limitada, não a curva.
app-gradient-canvas-hover = A curva de mistura do segmento selecionado: a rapidez com que a cor vai da parada da esquerda até a da direita. Arraste o anel para mover o ponto médio, ou mude a curva para Bézier para ter um ponto de controle que se move nas duas direções.
# The selected stop's heading. $index: its number; $total: how many there are.
app-gradient-stop-heading = Parada { $index } de { $total }
app-gradient-stop-color-hover = A cor nesta parada — clique na amostra para escolher, ou digite um valor hexadecimal ao lado
app-gradient-stop-bytes-hover = Vermelho, verde e azul como 0–255 — os bytes com que esta cor é gravada
# The eyedropper button.
app-gradient-pick = Escolher
app-gradient-pick-hover = Conta-gotas — pegar uma cor de qualquer lugar da tela. Mova o ponteiro até a cor desejada e clique; Esc cancela.
# Written before the stop's position (0.000–1.000) in its number field: a short abbreviation of "position".
app-gradient-stop-pos = pos
app-gradient-nudge-hover = Deslocar esta parada (segure Shift para um passo mais fino)
app-gradient-remove = Remover
app-gradient-remove-hover = Remover esta parada (ou clique no ⊖ abaixo dela)
app-gradient-add-stop = Adicionar parada
app-gradient-add-stop-hover = Dividir o segmento mais largo (ou clique na linha acima do gradiente)
# A menu of the built-in palettes, to start from a copy of one.
app-gradient-copy-preset = Copiar predefinição…
app-gradient-paste = Colar…
app-gradient-paste-hover = Importar uma paleta de cores hexadecimais ou de triplas RGB 0–255
app-gradient-save-ggr = Salvar .ggr…
app-gradient-save-ggr-hover = Gravar este gradiente num arquivo .ggr do GIMP — legível pelo GIMP, Krita e Inkscape, e por este editor
# A menu of the palette file formats (.map, .ugr, .ggr, .ase).
app-gradient-import = Importar
app-gradient-import-map-hover = Carregar um arquivo de paleta .map do Fractint / Kalles Fraktaler (linhas R G B, uma por entrada)
app-gradient-import-ugr-hover = Carregar um arquivo de gradiente .ugr do Ultra Fractal (um .ugr geralmente contém vários gradientes — você escolhe um)
app-gradient-import-ggr-hover = Carregar um gradiente .ggr do GIMP, com seus pontos médios, curvas de mistura e espaços de cor intactos
app-gradient-import-ase-hover = Carregar uma lista de amostras da Adobe (.ase). As amostras não têm posições, então chegam espaçadas por igual e mescladas.
# The gradient's name, for the library of saved gradients.
app-gradient-name = Nome
# Shown greyed in the empty name field.
app-gradient-name-hint = meu gradiente
# The save button, when a gradient of that name is already saved.
app-gradient-update = Atualizar
app-gradient-save = Salvar
app-gradient-save-hover = Salvar este gradiente na sua biblioteca, com suas curvas e espaços de cor intactos
app-gradient-save-needs-name-hover = Dê um nome ao gradiente primeiro
# A menu of the saved gradients. $count: how many there are.
app-gradient-saved-menu = Salvos ({ $count })
app-gradient-nothing-saved = Nada salvo ainda
app-gradient-delete-saved-hover = Excluir este gradiente salvo
# $count: the gradient's segments.
app-gradient-rich = Gradiente rico: { $count } segmentos têm pontos médios, curvas de mistura ou espaços de cor — mais do que uma simples lista de paradas comporta.
app-gradient-convert = Converter em paradas editáveis
app-gradient-convert-hover = Substituí-lo por uma simples lista de paradas. Isso DESCARTA os pontos médios, as curvas de mistura e as varreduras de matiz que uma lista de paradas não comporta.
# Above the list of gradients in an Ultra Fractal file. $count: how many; $file: the file's name.
app-gradient-ugr-list = { $count } gradientes em { $file }
app-gradient-close-list = Fechar lista
app-gradient-seamless = Sem emenda (o fim igual ao início)
app-gradient-seamless-hover = Uma paleta é cíclica, então as posições 1 e 0 ficam lado a lado na tela. Com isto ligado, a cor da última parada é mantida igual à da primeira — a vista em anel mostra a junção no topo.
app-gradient-ends-differ = ⚠ as pontas diferem
app-gradient-ends-differ-hover = A cor no fim do gradiente não é a cor do início, então uma paleta cíclica mostra uma borda dura onde dá a volta.
app-gradient-hard-bands = Faixas rígidas (sem mistura entre as entradas)
app-gradient-hard-bands-hover = Semântica do Fractint / .map: cada entrada é uma cor chapada. Desligado, elas se misturam num gradiente suave.
app-gradient-paste-intro = Cole cores hexadecimais (#ff8800) ou triplas 0–255 (255 136 0), separadas por vírgulas, espaços ou quebras de linha:
# Applies the pasted colours.
app-gradient-apply = Aplicar
# Empties the paste box.
app-gradient-clear = Limpar
# $used: the colours kept; $total: the colours pasted; $limit: the most that are kept.
app-gradient-paste-imported-some = { $used } de { $total } cores importadas, amostradas por igual ao longo da sua lista ({ $limit } é o limite).
# $count: the colours pasted.
app-gradient-paste-imported = { $count } cores importadas.
# $error: why the pasted text could not be read (in English).
app-gradient-paste-unreadable = Não foi possível ler isso: { $error }
# A hint at the bottom of the editor. $count: the palette's entries.
app-gradient-bands-hint = { $count } faixas · tomadas na ordem do arquivo; as posições são ignoradas enquanto as faixas estiverem ligadas.
# A hint at the bottom of the editor, in the ring view. $count: the gradient's stops.
app-gradient-ring-hint = { $count } paradas · clique na trilha externa para adicionar uma ali · ⊕ adiciona no maior intervalo, ⊖ remove a parada selecionada · arraste um marcador para movê-lo · a marca no topo é a emenda
# A hint at the bottom of the editor, in the bar view. $count: the gradient's stops.
app-gradient-bar-hint = { $count } paradas · clique na linha acima do gradiente para adicionar uma ali · arraste um marcador para movê-lo · ⊖ ou o clique direito remove
app-gradient-cancel = Cancelar
app-gradient-cancel-hover = Descartar todas as alterações feitas desde que esta janela foi aberta, e fechá-la
app-gradient-ok = OK
app-gradient-ok-hover = Manter estas alterações e fechar o editor

## Shared

# The button that closes the Help window and the "Faster deep zoom" window.
app-close = Fechar

## The "Faster deep zoom" window (Help menu): the optional accelerated build

app-accel-title = Zoom profundo mais rápido
app-accel-active-heading = Você está usando a build acelerada
app-accel-active-body = As órbitas de referência de zoom profundo estão sendo calculadas com MPFR/GMP, o que é de 2,5 a 6,4x mais rápido que a build padrão nessa etapa - a pausa antes de uma vista profunda começar a se resolver.
# $status: which arithmetic library is in use and its version (in English).
app-accel-in-use = Aritmética em uso: { $status }
app-accel-fallback-heading = Build acelerada — usando a aritmética integrada
app-accel-fallback-body = Esta é a build acelerada (MPFR), mas as bibliotecas matemáticas dela não foram encontradas, então ela está usando a aritmética integrada (astro-float). As imagens são idênticas; só a pausa antes de uma vista profunda se resolver é mais lenta.
# Followed by the folder the program runs from, on its own line.
app-accel-fallback-dlls = Para usar o caminho mais rápido, coloque libmpfr-6.dll e libgmp-10.dll (e libgcc_s_seh-1.dll, libwinpthread-1.dll) ao lado do fractadyne.exe e reinicie:
app-accel-warning-hidden = O aviso de inicialização sobre isso está oculto.
app-accel-show-again = Mostrar de novo
app-accel-offer-heading = Uma build opcional é de 2,5 a 6,4x mais rápida em profundidade
app-accel-offer-body = O zoom profundo passa boa parte do tempo calculando órbitas de referência na CPU - a pausa antes de uma vista profunda começar a se resolver. Uma build opcional faz isso com as bibliotecas MPFR/GMP, o que é de 2,5 a 6,4 vezes mais rápido, e mais ainda quanto mais fundo você vai. Todo o resto é igual.
app-accel-offer-identical = As imagens são IDÊNTICAS BYTE A BYTE - é a mesma matemática calculada por uma biblioteca mais rápida, verificada em todas as fórmulas de fractal e em todo o corpus de comparação de zoom profundo.
app-accel-offer-shared = Suas configurações, sessão salva e locais são compartilhados entre as duas, então você pode alternar à vontade e nada precisa ser importado.
# Shown on Windows.
app-accel-separate-windows = É um download separado porque as bibliotecas que ela usa não podem ser compiladas com o compilador que a build padrão para Windows usa, e têm uma licença diferente (GNU LGPL v3) da do próprio Fractadyne.
# Shown on other systems.
app-accel-separate = É um download separado porque as bibliotecas que ela usa têm uma licença diferente (GNU LGPL v3) da do próprio Fractadyne.
app-accel-download = Baixar para esta versão
app-accel-all-releases = Todas as versões
app-accel-all-releases-hover = Se o link direto der 404, esta versão ainda não tem build acelerada - escolha a mais próxima aqui.
# Shown on Windows.
app-accel-install-windows = Extraia o pacote e execute o fractadyne.exe a partir dessa pasta, mantendo os arquivos .dll ao lado dele.
# Shown on Linux.
app-accel-install-linux = Extraia o pacote e execute ./fractadyne a partir dessa pasta. Ela usa o GMP e o MPFR do seu sistema (libgmp10 / libmpfr6), que quase todo desktop já tem - veja README-ACCELERATED.txt no pacote.
# Shown on other systems.
app-accel-install-other = Ainda não há pacote acelerado para esta plataforma; ele pode ser compilado a partir do código-fonte com --features fractadyne-core/rug.

## The Help window's frame (its sections are under help-)

app-help-title = Ajuda do Fractadyne

## Checking for updates

app-toast-up-to-date = Você está na versão mais recente.
# $error: why the check failed (in English).
app-toast-update-failed = Falha na verificação de atualizações: { $error }
# The window that offers a newer version.
app-update-title = Atualização disponível
# $version: the new version (0.3.0-beta.20). $channel: the token "stable" or "beta", in English; a translation can choose its words by it ({ $channel -> [beta] … *[other] … }).
app-update-version =
    { $channel ->
        [beta] O Fractadyne { $version } (beta) está disponível
       *[other] O Fractadyne { $version } (estável) está disponível
    }
# $current: the version running now. $track: the update track the user follows (its name in the language in use).
app-update-running = Você está usando { $current }  ·  { $track }
app-update-download-intro = Baixe a versão mais recente do GitHub:
app-update-download = Baixar do GitHub
app-update-later = Lembrar mais tarde

## The second graphics card (shown in its window, after "In use:" or "Could not start:")

# The second device is the same card as the window's. $name: the card's name.
app-worker-same-card = mesma placa, { $name }
# Why the second device could not start.
app-worker-thread-stopped = a thread do dispositivo parou

## Export status (shown in the Export image window)

app-export-gpu-unavailable = GPU indisponível
app-export-reference-aborted = Falha na exportação: a construção da referência foi abortada.

## The performance panel (View ▸ Performance panel)
## Its row labels sit in a monospace column about 14 characters wide: keep them short (a longer label pushes its value to the right).

app-perf-fps = FPS
app-perf-fps-hover = Quadros apresentados por segundo, a partir do intervalo suavizado entre quadros. Enquanto uma vista está se estabilizando, isto mostra o progresso do refinamento - uma vista em estabilização não está rodando na taxa que um número aqui sugeriria.
# What the FPS row shows while a view is settling. $done and $total: tiles; $eta: the time left, as m:ss.
app-perf-refining-tiles = refinando { $done }/{ $total } (~{ $eta })
app-perf-refining-arming = refinando (preparando)
# $percent: how much is done, with one decimal.
app-perf-refining-percent = refinando { $percent }%
app-perf-building-reference = construindo referência
app-perf-idle = ocioso
app-perf-frame = quadro
app-perf-frame-hover = Tempo de relógio suavizado entre quadros - não é a soma das linhas abaixo. Inclui a espera para apresentar, então numa vista estabilizada e ociosa reflete a cadência de redesenho, e não algum trabalho.
app-perf-cpu = cpu
app-perf-cpu-hover = Tempo da thread principal dentro de uma atualização: preparação dos parâmetros e gerenciamento do cache de referências. É esta a linha que dá picos quando uma referência é reconstruída, e um pico aqui é o engasgo que você sente.
app-perf-gpu-idle = gpu/ocioso
app-perf-gpu-idle-hover = Tempo do quadro menos o tempo de CPU da thread principal. Isso é trabalho da GPU enquanto a vista está renderizando, mas numa vista estabilizada é principalmente espera ociosa - o nome diz as duas coisas porque o painel não consegue distingui-las só com esta subtração.
app-perf-timing = medição
# How frame cost is measured, the timing row's value. TIMESTAMP_QUERY is a name.
app-perf-timing-starved = relógio (faltam timestamps)
app-perf-timing-gpu = timestamps da GPU
app-perf-timing-no-query = relógio (sem TIMESTAMP_QUERY)
app-perf-timing-hover = Como o custo do quadro é medido. Os timestamps da GPU são a fonte precisa; o relógio de parede entra como alternativa quando as medições param de chegar, e um orçamento calculado pelo relógio de parede já dimensionou mal quadros profundos repetidas vezes — trate o vermelho aqui como “as margens de segurança são estimativas neste momento”.
app-perf-mode = modo
app-perf-mode-hover = O caminho aritmético com que o último quadro foi renderizado: direto (sem órbita de referência), perturbação df32 ou floatexp - o caminho de faixa estendida usado além de cerca de 1e28x, e o único em que o BLA roda.
app-perf-eff-iter = iter efet.
app-perf-eff-iter-hover = Iterações que o último quadro realmente usou, após o escalonamento pela profundidade e qualquer limite aplicado durante o movimento - não a configuração base em Qualidade, que é apenas o ponto de partida.
app-perf-precision = precisão
app-perf-precision-hover = Bits de mantissa com que a órbita de referência é calculada. Cresce com a profundidade; 64 bits é o piso raso, não um limite que você está atingindo.
app-perf-orbit-len = compr. órbita
# $cap: the most orbit samples this GPU holds, already formatted, or the text of app-perf-uncapped.
app-perf-orbit-len-hover = Amostras da órbita de referência. Esta GPU comporta no máximo { $cap } — chegar perto disso é o muro de profundidade prático: além dele, a vista ao vivo não consegue resolver locais mais profundos que precisem de uma órbita mais longa.
app-perf-uncapped = (sem limite)
app-perf-sa-skip = salto SA
app-perf-sa-skip-hover = Iterações que a aproximação por séries pulou antes de começar o trabalho por pixel. Mais é mais rápido e não muda nada na imagem; a linha não aparece quando nada foi pulado.
app-perf-aa = aa
app-perf-aa-hover = Superamostragem: cada pixel é renderizado a partir deste número de amostras por eixo, então 2x custa quatro vezes o trabalho.
app-perf-dual = dupla
app-perf-dual-hover = Se a vista dividida Mandelbrot/Julia está rodando. Quando verdadeiro, todo custo neste painel cobre OS DOIS painéis.
app-perf-zoom = zoom
app-perf-zoom-hover = Ampliação relativa à vista completa. O modo aritmético acima muda por volta de 1e4x e de novo por volta de 1e28x.
app-perf-iterate = iterar
# $target and $lethal: times in milliseconds, already formatted.
app-perf-iterate-hover = Tempo de GPU medido do último despacho de iteração. O controlador mira em { $target } ms; leituras sustentadas perto de { $lethal } ms são a faixa em que esta classe de hardware perdeu o dispositivo.
app-perf-steps = passos/s
app-perf-steps-hover = Passos de iteração por segundo na GPU, em bilhões. Vistas com muito interior rodam mais devagar por passo do que a contagem sozinha sugere, então isto é uma leitura de vazão e não uma pontuação.
app-perf-budget = orçamento
# After the budget's value while it is still converging; the hover below quotes it.
app-perf-budget-settling = (estabilizando)
app-perf-budget-hover = Orçamento adaptativo de passos por despacho, a proteção do controlador contra um quadro longo o bastante para o driver redefinir a GPU. “estabilizando” significa que ele ainda está convergindo a partir de leituras esparsas e ainda não é um teto confiável.
app-perf-rss = rss
# $mb and $peak: memory in megabytes, already formatted.
app-perf-rss-value = { $mb } MB (pico { $peak })
app-perf-rss-hover = Memória residente do processo, e o máximo que ela atingiu nesta sessão. Construções de referência profundas já passaram de 2 GB em uso real sem que nada mais na tela o indicasse; fica âmbar acima de três quartos da RAM do sistema.
app-perf-gpu-est = gpu est.
app-perf-gpu-est-hover = Memória estimada residente na GPU: órbitas de referência, árvores BLA, texturas de iteração, o pingue-pongue do estado dos chunks enquanto um refinamento roda, e o instantâneo da barreira de apresentação. Montada a partir de tamanhos de alocação conhecidos; o valor real do driver não pode ser lido de forma portável.
app-perf-julia-re = julia c.re
app-perf-julia-re-hover = Parte real do parâmetro c que o painel Julia está renderizando - o ponto sob o cursor, a não ser que esteja fixado.
app-perf-julia-im = julia c.im
app-perf-julia-im-hover = Parte imaginária do parâmetro c que o painel Julia está renderizando.
app-perf-julia-c = julia c
# The julia c row's value when the parameter is held fixed.
app-perf-pinned = fixado
app-perf-pinned-hover = O parâmetro está preso num ponto fixo, então mover o cursor não muda mais o painel Julia.
app-perf-c-panel = c/painel
app-perf-c-panel-hover = Largura do espaço c abrangida pelo painel Mandelbrot inteiro. Quando isto fica muito abaixo de um pixel da Julia, passar o cursor muda o c, mas a Julia parece igual — esperado em zoom profundo, não é um travamento.
app-perf-ref-recompute = recálc. ref
app-perf-ref-recompute-hover = Duração do recálculo mais recente da órbita de referência. É a pausa antes de uma vista profunda começar a se resolver, e a etapa que a build acelerada opcional torna mais rápida.
app-perf-recompute-rate = recálc./s
app-perf-recompute-rate-hover = Órbitas de referência INSTALADAS por segundo - aquelas cujo resultado foi realmente usado. Compare com a linha abaixo: a diferença entre elas é o trabalho jogado fora.
app-perf-builds-rate = constr. ref/s
app-perf-builds-rate-hover = Construções de referência INICIADAS por segundo, incluindo aquelas cujo resultado é descartado - um slot de antecipação que o mergulho nunca alcança, ou uma extensão recusada. Elas custam CPU integral e não aparecem em nenhum outro lugar: uma vez, um giro de pré-busca rodou cerca de 400 construções por segundo durante seis minutos enquanto recálc./s marcava 2. Um só dígito é normal; uma grande diferença entre estas duas linhas significa que trabalho está sendo jogado fora.
app-perf-recompute-total = recálc. total
app-perf-recompute-total-hover = Recálculos de referência desde a inicialização. Útil como taxa num intervalo conhecido; um número subindo enquanto nada se move é o sinal que vale a pena investigar.

## Help: the contents list (Help & reference…, F1)

# Each section's name in the contents list (about 165 px wide: keep them short), and its heading.
help-section-overview = Visão geral
help-section-navigation = Navegação
help-section-expressions = Expressões de coordenadas
help-section-options = Coloração e opções
help-section-fractals = Fractais
help-section-how-it-works = Como funciona
help-section-command-line = Linha de comando
help-section-shortcuts = Atalhos
help-section-hardware = Hardware recomendado
# Credits to the prior work Fractadyne builds on.
help-section-acknowledgments = Agradecimentos
# The third-party license notices.
help-section-licenses = Licenças
help-section-about = Sobre

## Help ▸ Overview

help-overview-intro = O Fractadyne é um explorador de fractais nativo, feito para zoom ultraprofundo e velocidade. Ele desenha fractais de “tempo de escape” — imagens criadas repetindo uma única fórmula simples em cada pixel.
help-overview-what-is = O que é um fractal de tempo de escape?
help-overview-what-is-text = Para cada pixel, o programa executa uma fórmula como z → z² + c repetidas vezes, começando do zero. Se o valor acumulado continuar pequeno para sempre, o pixel pertence ao conjunto e é desenhado escuro. Se ele acabar crescendo sem limite (“escapar”), o pixel está fora do conjunto, e sua cor registra quantos passos isso levou. A fronteira infinitamente detalhada entre “fica” e “escapa” é o fractal.
help-overview-can-do = O que você pode fazer
help-overview-can-pan = Mover a vista e dar zoom praticamente sem limite (a posição é exata em qualquer profundidade).
help-overview-can-families = Alternar entre 25 famílias de fractais integradas (dez, e cinco delas em potências maiores), suas próprias fórmulas, Life e sistemas L, e ver qualquer família de tempo de escape, exceto Newton, como um conjunto de Julia.
help-overview-can-recolor = Recolorir com gradientes predefinidos ou personalizados e vários métodos de coloração.
help-overview-can-lighting = Adicionar iluminação de relevo 3D e contornos brilhantes na fronteira.
help-overview-can-minibrots = Encaixar em minibrots, guardar lugares nos favoritos e exportar imagens de alta resolução.
help-overview-can-autopilot = Deixar o piloto automático do zoom mergulhar sozinho em direção aos detalhes.
help-overview-can-share = Compartilhar qualquer lugar como um pequeno trecho de texto “.fdn” ou como arquivo.
help-overview-can-tours = Executar tours roteirizados e um benchmark de hardware.
help-overview-first-steps = Primeiros passos
# "Seahorse Valley" is a place in the Navigate menu: use its name as that menu shows it. A and F1 are keys.
help-overview-first-steps-text = Abra o menu Navegar e escolha “Seahorse Valley”, depois gire a roda do mouse para aproximar. Arraste para mover a vista. Ou simplesmente pressione A e deixe o piloto automático mergulhar por você. Pressione F1 a qualquer momento para voltar a esta ajuda.

## Help ▸ Coordinate expressions
## The expressions, numbers and function names here are typed as shown in every language:
## keep them exactly as written. Menu items in quotation marks are the menus' own labels.

help-expr-intro = Onde quer que o Fractadyne receba uma coordenada, você pode digitar uma expressão em vez de um decimal: os campos Re / Im de Navegar → “Ir para local…” (e cada campo do seu modo polar), as linhas center_re / center_im de um local .fdn, os re / im de um script de tour e --center na linha de comando. A expressão é avaliada com tantos dígitos quantos o zoom precisar, e a janela Ir para local a mantém junto com a vista — então um ponto que você insere como 1/3, ou como -0.5 + 0.25*cos(pi/4), fica exatamente sobre esse ponto por mais fundo que seja o zoom, enquanto um decimal se desviaria quando seus dígitos acabassem.
help-expr-examples = Exemplos
help-expr-ex-third = Um terço exato. Mais dígitos aparecem conforme você aproxima.
help-expr-ex-neck = O pescoço do Seahorse Valley — a raiz do bulbo de período 2.
help-expr-ex-circle = Um ponto no círculo de raio 1/4 em torno de -0.5 (o modo polar compõe isso para você).
help-expr-ex-bulb = Re da raiz do bulbo de período 3; o Im dela é sin(2*pi/3)/2 - sin(4*pi/3)/4. Navegar → “Pontos exatos (expressões)” já traz estes prontos.
help-expr-ex-complex = Um valor complexo inteiro digitado no campo Re preenche tanto Re quanto Im.
help-expr-ex-combine = Funções e constantes se combinam livremente.
help-expr-ex-powers = Potências: 2^-20 é 1/1048576. -2^2 é -4 — o sinal de menos é aplicado depois da potência.
help-expr-ex-root = -2, a raiz cúbica real. Raízes pares de um número negativo são recusadas; sqrt(-1) dá i.
help-expr-ex-scientific = A notação científica também funciona dentro das expressões.
help-expr-numbers = Números e operadores
# The decimal point in an expression is '.' in every language: keep that rule as it is.
help-expr-numbers-decimals = Decimais com expoente opcional: 0.25, -1.5e-30, .5. Somente dígitos — sem espaços nem vírgulas de agrupamento dentro de um número, e o separador decimal é sempre '.'.
help-expr-numbers-imaginary = Adicione um sufixo i para um número imaginário (0.25i), ou use i sozinho: 3 + 4*i.
# Starts with the operators themselves, two spaces apart.
help-expr-numbers-operators = +  -  *  /  e ^ (potência; avaliada da direita para a esquerda, então 2^3^2 = 2^9). Parênteses agrupam. Não há multiplicação implícita: escreva 2*pi, não 2pi.
help-expr-numbers-case = Os nomes não diferenciam maiúsculas de minúsculas: PI, Pi e pi são a mesma constante.
help-expr-constants = Constantes
help-expr-functions = Funções
help-expr-functions-angles = Os ângulos são em radianos. Para graus, multiplique por pi/180 — cos(45*pi/180) — ou mude a janela Ir para local para o modo polar e escolha a unidade graus.
help-expr-refused = O que é recusado, e por quê
help-expr-refused-fractional = Um número negativo elevado a uma potência fracionária, como (-2)^(1/2): o resultado é complexo e há várias raízes para escolher. Use root(x, n) ou cbrt(x) para uma raiz real, ou sqrt(x) para a raiz complexa principal.
help-expr-refused-complex = Um argumento complexo para uma função de valor real (sin(i), ln(1+i)). Só sqrt e abs aceitam argumento complexo; os operadores aritméticos e as potências inteiras funcionam com valores complexos.
help-expr-refused-domain = Erros de domínio: ln ou log de um número que não é positivo, asin ou acos fora de -1…1, 0 elevado a uma potência negativa, divisão por zero.
help-expr-refused-bounds = Argumentos de sin, cos, tan, exp ou ^ além de ±2^32, e aninhamento com mais de 32 níveis: uma coordenada colada é uma entrada não confiável, e esses casos levariam um tempo ilimitado.
help-expr-refused-grammar = Sem variáveis, funções definidas pelo usuário, comparações ou condicionais. A gramática é mantida pequena de propósito, para que um local salvo signifique a mesma coisa em todas as versões.
help-expr-no-parse = Se a expressão não for reconhecida
help-expr-no-parse-text = A mensagem nomeia o campo, diz o que foi encontrado e em qual caractere e — para os deslizes mais comuns — o que digitar em vez disso: um sinal de menos tipográfico (−) colado de um documento, × ou ÷, uma vírgula decimal, um * faltando entre um número e um nome, um parêntese não fechado, ou uma função escrita sem os parênteses. Nada é lido pela metade: se qualquer parte do texto não puder ser entendida, a coordenada inteira é recusada em vez de ser encurtada silenciosamente.

## Help ▸ Navigation
## Menu paths (Navigate → "Go to location…") and quoted items name the interface's own
## menus and controls: use their translations. Key names (Ctrl, Shift, Space, Esc) stay.

# A heading: the mouse controls (also in Shortcuts).
help-mouse = Mouse
# The left column of a controls table: a gesture or key. Keep key names (Shift, Space, Ctrl) as they are.
help-key-left-drag = Arrastar (botão esq.)
help-nav-left-drag = Mover a vista.
help-key-mouse-wheel = Roda do mouse
help-nav-wheel = Aproximar/afastar em direção ao cursor; gire mais rápido para avançar mais o zoom a cada clique da roda. Com “Atrair o zoom às estruturas” ligado, aproximar perto de um minibrot puxa o zoom para ele.
help-key-shift-drag = Shift+arrastar
help-nav-shift-drag = Zoom por retângulo — arraste um retângulo e o zoom faz com que ele preencha a vista.
# $icon: the click-to-zoom toolbar button's symbol.
help-key-click-zoom = Clicar para dar zoom ({ $icon })
help-nav-click-zoom = Ative o botão de alvo na barra de ferramentas (vista única): um clique esquerdo então mergulha no ponto por um fator definido, e o clique direito recua. Arrastar ainda move a vista; escolha o fator (2–100×) em Navegar, no painel da direita. Backspace desfaz um clique.
help-nav-continuous = Zoom contínuo e vista inicial
help-key-hold-space = Segurar Space
help-nav-hold-space = Aproximar suavemente, ancorado no cursor.
help-key-hold-shift-space = Segurar Shift+Space
help-nav-hold-shift-space = Afastar suavemente.
help-nav-continuous-text = O botão da vista inicial anima um voo suave de volta à vista completa. “Velocidade do zoom” no painel da direita define a taxa do zoom contínuo.
help-nav-autopilot = Piloto automático
help-nav-autopilot-text = Pressione A (ou Ferramentas → “Zoom automático (piloto automático)”) para um mergulho sem as mãos: ele escolhe um ponto na borda do fractal onde os detalhes são mais densos, preferindo o meio da vista, e desliza para dentro dele enquanto o leva suavemente ao centro da tela. Ele mantém esse alvo enquanto ele continuar interessante e vira suavemente em direção a estruturas mais ricas quando elas aparecem. Qualquer comando de navegação — ou Esc — o interrompe. “Prioridade do zoom automático” no painel da direita (ou em Ferramentas) escolhe o que vem primeiro: Velocidade dá zoom na taxa definida e atualiza a tela assim que um quadro com detalhes fica pronto; Qualidade mostra apenas quadros totalmente resolvidos em resolução total e desacelera o zoom conforme necessário para que a imagem nunca seja ampliada demais antes de o próximo quadro chegar. De qualquer forma, a tela nunca mostra um quadro vazio ou de uma só cor: uma atualização que ainda não resolveu nenhum detalhe fica escondida atrás da anterior. “Alvo do zoom automático” escolhe para onde ele mira: Detalhe (a borda mais rica da vista, escolhida de novo pelo caminho) ou Ponto de Misiurewicz — ele encontra o ponto de Misiurewicz mais próximo, resolve-o até a profundidade do limite de mergulho enquanto fica parado, e então mergulha exatamente nele; a estrutura em torno de um ponto assim se repete a cada poucas oitavas para sempre, então esse mergulho nunca chega a um beco sem saída (somente no conjunto de Mandelbrot).
help-nav-history = Histórico e movimentos precisos
help-nav-undo = Desfazer a vista anterior.
help-nav-redo = Refazer.
help-nav-goto = Navegar → “Ir para local…” permite ler, digitar, colar ou copiar o centro e o zoom exatos (precisão total, qualquer profundidade). Os campos do centro aceitam mais do que números simples: frações como “1/3” e expressões montadas com pi, tau, e, sqrt e sin/cos/tan — então “-0.5 + 0.25*cos(pi/4)” é uma coordenada válida. Uma expressão é mantida com a vista e reavaliada com a precisão que cada profundidade exige, então um ponto inserido assim permanece exato por mais que você aproxime (um decimal arredondado acabaria se desviando). Um modo polar (deslocamento + r ∠ θ, em graus / radianos / voltas) compõe uma dessas expressões para você. A lista completa de funções e constantes, com exemplos, está na seção “Expressões de coordenadas” desta ajuda (o botão ? na janela a abre). Navegar → Favoritos salva e recupera locais.
help-nav-exact-points = Navegar → “Pontos exatos (expressões)” salta para pontos definidos exatamente, e não por um decimal arredondado — as raízes dos bulbos da cardioide principal, onde cada broto de período q se liga, escritas como expressões cos/sin. Como a expressão viaja com a vista, você pode dar zoom num deles sem que ele jamais se afaste do ponto.
help-nav-share = Navegar → “Compartilhar local…” captura a vista inteira — fractal, centro com precisão total, zoom e coloração — como um trecho “.fdn” compacto que você pode Copiar, colar de volta e Aplicar, ou Salvar/Carregar como arquivo, para reproduzir um lugar exatamente (ou enviá-lo a alguém).
help-nav-open = Arquivo → “Abrir vista ou local…” abre qualquer um destes diretamente: um PNG/EXR exportado (a vista incorporada nele é restaurada), um arquivo de local “.fdn” ou um “.kfr” do Kalles Fraktaler.
help-nav-finding = Encontrando detalhes
help-nav-minibrots = Minibrots são pequenas cópias do conjunto inteiro escondidas ao longo da borda. Centralize um aproximadamente e pressione M (ou Ferramentas → “Encontrar minibrot + dar zoom nele”) para encaixar por Newton exatamente no centro dele, ler seu período e dar zoom direto até a escala dele — muitas vezes várias ordens de grandeza num único passo. Se você já estiver mais fundo que o minibrot, só o centro se move.
help-nav-minimap = Exibir → “Minimapa” mostra um pequeno mapa do conjunto inteiro com um marcador “você está aqui” e a profundidade atual do zoom; clique no mapa para saltar para uma região.

## Help ▸ Coloring & options
## Names in quotation marks are the controls' own labels in the right-hand panel.

help-options-intro = Tudo isto fica no painel da direita (e persiste entre sessões).
help-options-palette = Paleta
help-options-palette-text = Paleta escolhe um gradiente predefinido, a sua paleta Personalizada ou um modo de duas cores. “Editar gradiente…” abre um editor em que cada parada de cor tem uma cor e uma posição (0–1); adicione paradas, copie uma predefinição como ponto de partida ou importe um arquivo de paleta.
help-options-paste = “Colar…” no editor de gradiente importa uma paleta que você encontrou em outro lugar. Ele aceita cores hexadecimais com ou sem # inicial (#ff8800, ff8800 ou o curto #f80) e triplas RGB 0–255 simples como “255 136 0” — o formato que os arquivos .map do Fractint e do Kalles Fraktaler usam —, separadas por vírgulas, espaços ou quebras de linha. São mantidas até 256 cores; uma lista mais longa é amostrada por igual até 256, mantendo a primeira e a última para que a forma do gradiente sobreviva.
help-options-import-map = “Importar ▸ .map…” ao lado carrega um arquivo de paleta .map do Fractint ou do Kalles Fraktaler. Esses arquivos são tabelas de consulta indexadas pela contagem de iterações, sem mistura entre as entradas, e os degraus de cor bruscos que isso produz são justamente o visual clássico do Fractint — então é assim que eles são importados, com uma caixa de seleção “Faixas rígidas” para misturar as mesmas cores num gradiente suave. Os próprios arquivos do Fractint guardam valores de cor de 6 bits multiplicados por quatro, então o branco deles é 252 em vez de 255; o Fractadyne avisa isso quando importa um e deixa os valores exatamente como estão escritos, porque as próprias imagens salvas do Fractint têm o mesmo 252.
help-options-duotone = Duotom mapeia o valor numa rampa suave de duas cores (Sombra → Realce). Binária (conjunto) é uma vista chapada de duas cores — uma cor sólida para os pontos dentro do conjunto e outra para os de fora, sem gradiente — a maneira mais clara de ver a forma do conjunto.
help-options-cycle = Ciclo define quantas vezes o gradiente se repete ao longo da faixa de iterações (faixas mais apertadas ou mais largas). Deslocamento gira o gradiente inteiro.
help-options-normalize = “Normalizar cores profundas” ajusta a paleta à faixa de valores de escape realmente presentes na vista. Vistas profundas abrangem centenas de milhares de iterações, e sem isso um ciclo fixo dá a volta no gradiente milhares de vezes entre pixels vizinhos, transformando detalhes corretos em granulação.
help-options-log-scale = “Escala de cor logarítmica” distribui a paleta pelo logaritmo do valor de escape em vez de linearmente. As contagens de escape se acumulam no extremo alto à medida que você se aprofunda, então um ajuste linear gasta a maior parte do gradiente numa casca fina colada à borda e deixa o resto quase chapado; a escala logarítmica abre isso e mantém as cores mais estáveis ao longo de um vídeo de zoom conforme a faixa cresce.
help-options-animate = Animar faz as cores circularem com o tempo — Para frente, Inverso ou Pingue-pongue movem o deslocamento; Gradientes aleatórios sintetiza continuamente gradientes harmoniosos que se transformam suavemente. Velocidade controla os ciclos (ou transformações) por segundo; “Embaralhar gradiente” sorteia um novo no modo Gradientes aleatórios.
help-options-method = Método de coloração — como os dados viram cor
# The coloring methods, by the names the Coloring panel gives them, each with what it does.
help-method-smooth = Iteração suave
help-method-smooth-text = Faixas contínuas clássicas pelo tempo de escape.
help-method-stripe = Média de listras
help-method-stripe-text = Faixas fluidas a partir do ângulo da órbita (controle Densidade de listras).
help-method-triangle = Desigualdade triangular
help-method-triangle-text = Textura fina a partir de onde cada passo cai entre os limites.
help-method-trap = Armadilha de órbita
help-method-trap-text = Distância da órbita a uma forma (ponto/cruz/círculo); colore também o interior.
help-method-distance = Estimativa de distância
help-method-distance-text = Sombreia pela proximidade da borda.
help-method-decomposition = Decomposição
help-method-decomposition-text = Células a partir do ângulo final de escape.
help-options-relief = Iluminação de relevo 3D
help-options-relief-text = Sombreia a superfície a partir da inclinação da borda (a derivada) para um visual em alto-relevo, iluminado. Ângulo da luz define a direção; Relevo define a intensidade (menor = mais nítido); “Girar a luz” a anima. Somente famílias holomorfas (Mandelbrot / Multibrot).
help-options-glow = Brilho de distância
help-options-glow-text = Faixas de contorno brilhantes que se adensam em filamentos luminosos perto da borda. Brilho é a quantidade de mistura, Largura da faixa é o espaçamento, e “Animar o brilho” as faz fluir.
help-options-quality = Qualidade e iterações
help-options-quality-text = Iterações é o número máximo de passos antes que um pixel seja tratado como dentro do conjunto; “Escalar iterações com o zoom” o aumenta automaticamente conforme você dá zoom (detalhes mais profundos precisam de mais). Anti-aliasing superamostra as imagens paradas (2×–8×) quando a vista se estabiliza, domando a fina “poeira” exterior.
help-options-other = Outros
help-options-other-text = Velocidade do zoom define a taxa do zoom contínuo; o limite de FPS limita a taxa de quadros; “Ambos, vinculados” mostra um conjunto de Mandelbrot e seu conjunto de Julia lado a lado (o cursor define o parâmetro da Julia ao vivo).

## Help ▸ Fractals
## Fractal family names (Mandelbrot, Multibrot, Tricorn, Burning Ship, Celtic, Buffalo,
## Phoenix, Newton) are names: keep them as written, as the formulas. Formula syntax
## (z = z^3 - p1*z + c, sqr, |z|, if / elseif / endif, p1-p5, maxit) is typed as shown.

help-fractals-intro = Toda família itera uma fórmula com z começando em 0 e c definido pelo pixel (tempo de escape), salvo indicação em contrário. z = x + iy é um número complexo.
help-fractals-mandelbrot = z → z² + c. O original. O conjunto é formado por todo c cuja órbita permanece limitada; ele é conexo, e sua fronteira é tão enrugada que tem dimensão de Hausdorff 2.
help-fractals-multibrot = z → z^d + c para a potência d = 3, 4, 5. Potências maiores acrescentam lóbulos: o conjunto tem simetria rotacional de ordem (d−1) (Multibrot 3 tem ordem 2, 4 tem ordem 3, 5 tem ordem 4).
help-fractals-tricorn = z → conj(z)² + c, onde conj(x + iy) = x − iy. A conjugação o torna anti-holomorfo, o que dá simetria de ordem 3 e as características “garras” curvas.
help-fractals-burning-ship = z → (|x| + i|y|)² + c — toma os valores absolutos antes de elevar ao quadrado (real = x²−y²+cx, imag = 2|xy|+cy). Não analítico; zooms profundos revelam estruturas parecidas com navios (tradicionalmente vistas de cabeça para baixo).
help-fractals-celtic = Como o Mandelbrot, mas com o valor absoluto da parte real: real = |x²−y²| + cx, imag = 2xy + cy. Produz motivos em forma de coração e de escudo.
help-fractals-buffalo = Valor absoluto das duas partes de z²: real = |x²−y²| + cx, imag = |2xy| + cy — um cruzamento entre Celtic e Burning Ship.
# A heading: the families at powers above 2 (the Fractal menu's "Higher powers").
help-fractals-powers = Potências maiores
# ("Burning Ship  3 4 5") is how the Fractal menu shows such a row: keep it as it is.
help-fractals-powers-text = Multibrot 6, 7 e 8 (z^d + c), e Burning Ship, Tricorn, Celtic e Buffalo nas potências 3, 4 e 5: as mesmas dobras numa potência maior. Burning Ship d toma os valores absolutos das partes de z e depois a potência; Tricorn d conjuga primeiro; Celtic d e Buffalo d dobram a parte real (e, no Buffalo, a parte imaginária) de z^d. O menu Fractal e a lista suspensa os mostram uma família por linha (“Burning Ship  3 4 5”). Cada um faz zoom profundo como o seu irmão de potência 2, e Multibrot 6, 7 e 8 também aceitam aproximação por séries e o localizador de minibrots (M), como Multibrot 3–5. Multibrot 8 escapa no raio 128 em vez de 256, onde o passo seguinte sairia da faixa de precisão simples.
help-fractals-phoenix = z → z² + c + p·z(n-1), onde z(n-1) é o iterado anterior e p é uma constante (p = -0.5). O termo de memória produz filamentos parecidos com chamas.
help-fractals-newton = Método de Newton para as raízes de z³ − 1 = 0: z → z − (z³−1)/(3z²). Em vez do tempo de escape, os pixels são coloridos conforme a raiz cúbica da unidade, entre as três, para a qual a iteração converge (as bacias de atração) e com que rapidez; as fronteiras emaranhadas das bacias são o fractal.
help-fractals-julia = Modo Julia
help-fractals-julia-text = Para toda família, exceto Newton, você pode mudar para um conjunto de Julia: em vez de começar z em 0 com c = pixel, você fixa c (um parâmetro) e deixa z começar no pixel. O conjunto de Julia é a fronteira entre os pontos de partida que permanecem limitados e os que escapam, para aquele c fixo. Em “Ambos, vinculados”, mover o cursor sobre o painel do Mandelbrot define c ao vivo.
help-fractals-deep = Suporte a zoom profundo
help-fractals-deep-text = Mandelbrot, Multibrot 3/4/5 e Tricorn suportam zoom profundo por perturbação em profundidade extrema (floatexp). Burning Ship, Celtic e Buffalo não são analíticos (tomam valores absolutos), então usam uma perturbação que leva o sinal em conta; ela agora também roda na faixa floatexp, com zoom profundo muito além do antigo limite df32 de ~1e28× (uma granulação rara perto das dobras de abs aguarda a correção de glitches com várias referências). Phoenix usa uma perturbação de dois termos e também faz zoom profundo na faixa floatexp (sem SA/BLA, então é mais pesado que o Mandelbrot). As potências maiores fazem zoom profundo do mesmo jeito, com a perturbação de cada potência seguindo sua própria cadeia de quadrados; Multibrot 3–8 também aceitam aproximação por séries e BLA, como o Mandelbrot, e Tricorn, Burning Ship, Celtic e Buffalo em todas as potências aceitam BLA (sem séries), que atravessa o interior de uma vista profunda em vez de iterá-lo passo a passo. Só o Newton usa o caminho direto, nítido até ~1e6×; as fórmulas personalizadas fazem zoom profundo, a menos que a seção Fórmulas personalizadas abaixo diga o contrário.
help-fractals-custom = Fórmulas personalizadas
help-fractals-custom-text = Fractal ▸ Fórmula personalizada… recebe o passo de uma fórmula em expressões no estilo do Fractint: z = z^3 - p1*z + c, ou várias instruções (t = sqr(z), depois z = t*t + c). z é o iterado, c o pixel (a constante de Julia no modo Julia), p1-p5 os parâmetros; |z| é o módulo ao quadrado, como no Fractint, e as funções usuais estão disponíveis (sqr, abs, conj, real, imag, cabs, flip, recip, ident, exp, log, sqrt, sin, cos, tan, cotan, sinh, cosh, tanh, cotanh, asin, acos, atan, asinh, acosh, atanh, cosxx, e floor, ceil, trunc e round, que arredondam cada parte). log é o logaritmo natural, como no Fractint e na análise complexa: o que muitos livros escrevem ln (não há logaritmo de base 10). Uma fórmula escrita com ln, cot, coth, Re ou Im não é lida; a verificação de sintaxe nomeia a função a usar, um botão a reescreve, e digitar ln oferece log. Todo passo faz zoom profundo por perturbação, com a mesma aritmética de faixa estendida além de 1e28x que os fractais integrados, exceto um com uma potência cujo expoente varia com z ou c (como z^c): esse é renderizado pelo caminho direto, nítido até que um passo de precisão simples de c abranja um pixel, por volta de 1e4x a 1e5x, e menos fundo longe da origem. A janela da fórmula diz qual caso se aplica. log, sqrt e as potências não inteiras tomam seus valores principais, que saltam através do semieixo real negativo. A perturbação segue o salto exatamente, mas um pixel cuja órbita cruza onde a da referência não cruza passa a ser calculado com precisão aproximadamente simples daí em diante, então vistas profundas cujas órbitas atravessam um corte mostram ruído ali (medido: limpo em 1e5x no eixo real para sqrt(z^4 + c), ruidoso para fórmulas com log já em 1e4x). Cada passo é iterado por completo (sem aproximação por séries), então uma vista profunda com contagem de iterações alta é renderizada mais devagar que a de um fractal integrado. Perto da borda, uma fórmula com funções tem órbitas caóticas, e uma órbita longa ali só é seguida de forma aproximada em precisão simples: com 2.000 iterações, um terço dos pixels de sin(z) + c difere da órbita exata; com 60, quase nenhum. Vistas, sessões e imagens exportadas guardam o texto da fórmula.
help-fractals-custom-sections = As seções do Fractint também funcionam. As instruções antes de um ':' são a seção de inicialização, executada uma vez por pixel; uma comparação como última instrução é o próprio teste de escape da fórmula, que itera enquanto ela vale (|z| <= 100 && |z - 1| > 0.000001 para uma órbita que escapa ou se estabiliza em 1); if (...), elseif (...), else e endif escolhem instruções; e uma variável lida antes de ser definida num passo tem o valor do passo anterior (da seção de inicialização, ou 0). As comparações comparam partes reais e dão 1 ou 0, && e || as combinam, e uma atribuição também é um valor (a = b = pixel). pixel, p1-p5, pi e e podem receber atribuições, mantendo o próprio valor até que isso aconteça; maxit é a contagem de iterações e ismand é 1 (a forma Mandelbrot da fórmula). Uma fórmula com qualquer um destes, ou com arredondamento, é renderizada pelo caminho direto, sem zoom profundo. Biblioteca de fórmulas > Importar… também lê arquivos .frm do Fractint. Cada entrada que é lida vira uma fórmula, e sua linha diz o que a leitura mudou: fn1-fn4 recebem os padrões do Fractint (sin, sqr, sinh, cosh); a variável c do Fractint é renomeada para c_, já que c aqui é o pixel; um nome que nada define vale 0, como no Fractint; os espaços dentro de um nome são descartados (end if vira endif), como o Fractint faz. A mensagem da importação conta as entradas que não são lidas e dá o motivo mais comum: as variáveis de tela e de vista (whitesq, scrnpix, scrnmax, center, magxmag, rotskew), lastsqr e números aleatórios não são suportados. Os clássicos do Fractint da Coleção (Magnet, Nova, Spider, Manowar e outros) são escritos com essas seções.
help-fractals-custom-editor = No campo da fórmula, os parênteses ganham uma cor por nível de aninhamento; o par junto ao cursor é destacado, e um sem parceiro aparece em vermelho (os de um comentário não contam, pois o analisador os ignora). Os nomes se completam enquanto você digita: depois de duas letras, uma lista oferece as funções, os nomes e as suas próprias variáveis que começam assim. As setas para cima e para baixo escolhem, Tab ou Enter aceita um (uma função vem com o parêntese de abertura), Esc fecha a lista, e Enter depois de um nome digitado por inteiro ainda inicia uma nova linha.
help-fractals-custom-textbook = Livro-texto (a chave acima da fórmula) a mostra composta como um livro-texto a comporia, e a edita no próprio lugar. Digite como no texto: / transforma em fração o termo antes dela, ^ cria um expoente (+, - ou = digitados no fim de um expoente saem dele), ( abre parênteses, ou uma chamada depois do nome de uma função, | barras, e uma vírgula dentro de parênteses cria uma constante complexa. As setas percorrem a fórmula, as setas para cima e para baixo passam entre um numerador e seu denominador, Tab vai para a próxima caixa vazia; Backspace no início de uma fração ou de um par de parênteses a seleciona, e um segundo toque a remove, mantendo o que ela continha. Copiar, colar e desfazer funcionam como num campo de texto, os nomes se completam como lá, e o teclado de fórmulas digita na fórmula composta (□² eleva ao quadrado o que está antes do cursor, uma tecla de função envolve a seleção). Toda edição é gravada de volta no texto, então as duas vistas sempre contêm a mesma fórmula; uma linha que você não editou mantém o espaçamento e o comentário. A notação nunca muda o que é calculado: |z| é o módulo ao quadrado e aparece como |z|², cabs(z) aparece como |z|, e exp(z) continua exp(z). Uma linha que não é lida aparece em vermelho; clique nela para editá-la como texto. LaTeX copia a fórmula como código-fonte LaTeX, do jeito que está composta, e enquanto Livro-texto estiver ligado a biblioteca de fórmulas também mostra suas fórmulas compostas.
help-fractals-custom-numbers = Os números numa fórmula (0.1, 1/3, pi, os parâmetros) são lidos como valores de precisão dupla, cerca de 16 dígitos, e a órbita de referência profunda itera exatamente com esses valores. Duas fórmulas iguais no papel podem, portanto, diferir lá no fundo: 0.1*z multiplica pelo double mais próximo de 0.1, enquanto z/10 divide por um 10 exato, e as duas imagens se separam além de cerca de 1e15x (medido: 5% dos pixels em 1e15x, um terço em 1e17x, todos em 1e20x). Cada imagem é correta para a sua própria fórmula; escreva a que você pretende compartilhar.
help-fractals-custom-library = O botão Salvar na biblioteca da janela guarda uma fórmula e seus parâmetros sob um nome (salvar com um nome que a biblioteca já tem atualiza essa entrada). Fractal ▸ Biblioteca de fórmulas… as lista para aplicar, editar, exportar ou excluir; Importar… adiciona as fórmulas de um arquivo de fórmulas sem substituir nenhuma: uma que a biblioteca já tem é pulada, e uma nova cujo nome já está em uso vira “nome (2)”. A biblioteca é o formulas.toml na pasta indicada em Arquivos e dados (Sobre), ele mesmo um arquivo de fórmulas. Um quadro-chave de tour carrega uma fórmula como formula = "..." com formula_params = [[re, im], ...], e Ferramentas > Tour a partir da vista atual os grava para uma vista personalizada.
help-fractals-custom-collection = Uma coleção de 45 fórmulas vem com o Fractadyne, escritas para ele: potências maiores e as variantes dobradas das famílias Burning Ship e Celtic, misturas e híbridos de dois passos, o conjunto de Mandelbrot visto através de outro mapa (invertido, log-polar, através do seno), fórmulas transcendentais e racionais, e conjuntos de Julia. Elas são os Exemplos da janela e a aba Coleção da biblioteca de fórmulas, onde Copiar para as minhas adiciona uma às suas fórmulas. Cada uma tem uma vista de partida, e Aplicar vai até lá: uma fórmula aberta na vista inicial muitas vezes não está onde fica a imagem dela. Uma fórmula que você salva enquanto ela está sendo exibida guarda da mesma forma a vista em que você está, e Aplicar volta a ela.

## Help ▸ How it works

help-how-escape = Tempo de escape e cor suave
help-how-escape-text = Cada pixel itera a fórmula até que sua magnitude ultrapasse um raio de escape ou atinja o limite de iterações. A contagem bruta de passos sozinha produz faixas duras; somar uma parte fracionária derivada da magnitude final dá uma cor contínua, sem faixas.
help-how-precision = Posição com precisão arbitrária
help-how-precision-text = Números comuns de 64 bits ficam sem dígitos perto de 1e15× de zoom. O Fractadyne mantém o centro da vista em precisão arbitrária, com o número de dígitos crescendo conforme você dá zoom, então o local nunca se degrada. A escala do pixel também é mantida com um expoente estendido, então ela também não trava no limite de ~1e308× dos 64 bits — a profundidade é limitada só pela paciência, não por alguma faixa numérica fixa.
help-how-perturbation = Perturbação
help-how-perturbation-text = Iterar cada pixel em alta precisão seria lento demais. Em vez disso, um pixel de referência é iterado em alta precisão na CPU (a “órbita de referência”), e todos os outros pixels são calculados na GPU como uma pequena diferença δ em relação a ele, em baixa precisão rápida: δz → 2·Z·δz + δz² + δc.
help-how-floatexp = Profundidade extrema (floatexp)
help-how-floatexp-text = Além de cerca de 1e28×, mesmo essa pequena diferença sofre underflow na faixa de 32 bits, então ela é armazenada como uma mantissa mais um expoente inteiro separado (“floatexp”), derrubando o muro de profundidade dos 32 bits. O motor muda automaticamente: matemática direta quando raso, perturbação quando fundo e floatexp quando mais fundo. A profundidade passa então a ser limitada pela precisão das coordenadas e pelo orçamento de iterações, e não por um limite fixo — verificado contra o Fraktaler-3 num corpus de referência de 39 locais, o mais fundo em 5.63e18003× (as 38 linhas não extremas são a verificação de rotina, até 6.13e1105×), com uma bateria interna de autoconsistência de precisão até 1e1000000×; o tour incluído chega a ~1.3e420×.
help-how-reference = Escolha da referência e rebase
help-how-reference-text = A referência é escolhida (pontuada em alta precisão) de modo que sua órbita permaneça dentro da vista pelo maior tempo possível. Quando a diferença cresce demais, é feito um “rebase” de volta para a referência para manter a exatidão.
help-how-series = Aproximação por séries
help-how-series-text = Em zoom profundo, as primeiras iterações mal afastam δ da referência, então podem ser calculadas de uma vez: δz é aproximado por um polinômio curto em δc (ordem 3), cujos coeficientes são iterados ao longo da referência. O renderizador salta adiante até a última iteração em que esse polinômio ainda é exato para a vista inteira e então itera normalmente — economizando os passos pulados sem nenhuma mudança na imagem (liga/desliga em Controles ▸ Avançado). Mandelbrot e Multibrot 3–8, na faixa mais profunda; não com as colorações listras / desigualdade triangular / armadilha de órbita / decomposição (essas precisam de todas as iterações). Onde o BLA está ativo, ele já engloba esse salto inicial.
help-how-distance = Estimativa de distância e iluminação
help-how-distance-text = Acompanhar a derivada dz/dc fornece a distância de cada pixel até a borda. É isso que alimenta a iluminação de relevo 3D, o brilho de distância e o método de coloração “estimativa de distância” — todos válidos em qualquer profundidade de zoom.
help-how-safety = Anti-aliasing e segurança
help-how-safety-text = As imagens paradas são superamostradas (2–8× por eixo) quando a vista se estabiliza. Um orçamento de trabalho mantém cada desenho da GPU dentro do limite do watchdog do driver reduzindo a resolução (nunca a contagem de iterações) em configurações extremas, para que as vistas profundas continuem detalhadas em vez de ficarem em branco.

## Help ▸ Command line (the flags themselves stay English)

# The section's introduction. The flags listed below it are the command line itself, English in every language.
help-cli-intro = O Fractadyne roda sem interface para renderização, exportação de filmes/tours, automação, verificações com imagens de referência e benchmarks. Sem opções, ele abre a interface gráfica interativa.

## Help ▸ Shortcuts

# What each mouse gesture and key does (the gestures are the Navigation section's).
help-shortcuts-pan = Mover a vista
# A gesture: turning the mouse wheel.
help-key-wheel = Roda
help-shortcuts-wheel = Zoom no cursor
help-shortcuts-box-zoom = Zoom por retângulo (arraste um retângulo para dar zoom nele)
# $icon: the click-to-zoom toolbar button's symbol. "tool on": while that button is on.
help-key-click-tool = Clique (ferramenta { $icon } ligada)
help-shortcuts-click = Mergulhar no ponto; o clique direito recua
# A heading: the keyboard shortcuts.
help-shortcuts-keyboard = Teclado
help-shortcuts-space = Aproximar / afastar continuamente
help-shortcuts-undo = Desfazer vista
help-shortcuts-redo = Refazer vista
help-shortcuts-minibrot = Encontrar minibrot + dar zoom nele (Newton-Raphson)
help-shortcuts-autopilot = Piloto automático do zoom (mergulha em direção aos detalhes; qualquer comando o interrompe)
# off / strip / panel / full: the panel's sizes, as View ▸ Live render panel names them.
help-shortcuts-live-panel = Painel de renderização ao vivo: desligado / faixa / painel / completo (passe o cursor sobre um instrumento para ver o que ele significa; o × do painel o fecha)
help-shortcuts-esc = Parar o piloto automático / um tour em reprodução, ou sair da tela cheia
help-shortcuts-snapshot = Captura na última pasta de exportação — uma captura de tela da vista como é mostrada, ou uma renderização completa com as configurações de exportação; o primeiro uso pergunta qual, e Arquivo → Configurações → Captura muda isso.
help-shortcuts-help = Abrir esta ajuda
# A heading: the shortcuts of the Game of Life, with the menu path to it.
help-shortcuts-life = Life (Fractal → Autômatos → Life)
help-shortcuts-life-run = Executar / pausar o universo
help-shortcuts-life-step = Avançar uma geração / avançar o salto (definido no painel do Life)
help-shortcuts-life-draw = Ferramenta de desenho ligada / desligada (o lápis na barra de ferramentas)
# A gesture. "Draw on": while the Life draw tool is on.
help-key-click-drag-draw = Clicar / arrastar (Desenho ligado)
help-shortcuts-life-paint = Inverter uma célula / pintar: um traço que começa numa célula morta desenha, um que começa numa célula viva apaga. Um universo em execução pausa enquanto você desenha e continua quando você solta.
# $key: a keyboard shortcut (Ctrl+Z). "Draw on": while the Life draw tool is on.
help-key-when-drawing = { $key } (Desenho ligado)
help-shortcuts-life-undo = Desfazer a última edição, preenchimento, limpeza ou mudança de regra

## Help ▸ Recommended hardware

help-hardware-intro = O Fractadyne divide o trabalho entre a GPU (iteração + coloração por pixel) e a CPU (a órbita de referência de precisão arbitrária sobre a qual o zoom profundo é construído). As duas importam, mas para coisas diferentes.
help-hardware-gpu = GPU (o mais importante)
help-hardware-gpu-text = Cada pixel é iterado e colorido na GPU, então a velocidade da GPU define a taxa de quadros e o tempo de exportação. Uma GPU dedicada com bom desempenho em 32 bits (precisão simples) dá o zoom profundo mais suave. GPUs integradas funcionam bem — só são mais lentas e atingem antes o orçamento de trabalho por quadro (quando então o renderizador reduz a resolução, não os detalhes). Requer uma API gráfica moderna: Vulkan, Direct3D 12, Metal ou OpenGL via wgpu.
# The left column of a requirements table.
help-hardware-minimum = Mínimo
help-hardware-gpu-minimum = Qualquer GPU com Vulkan / DX12 / Metal / GL 3.3.
help-hardware-recommended = Recomendado
help-hardware-gpu-recommended = Uma GPU dedicada (dos últimos ~5 anos); >= 2 GB de VRAM.
help-hardware-big-exports = Para exportações grandes
help-hardware-gpu-big-exports = >= 4 GB de VRAM (em blocos, então degrada de forma suave).
# A heading: the processor.
help-hardware-cpu = CPU
help-hardware-cpu-text = A órbita de referência de alta precisão (e as tabelas BLA) são calculadas na CPU e são em grande parte de thread única, então o desempenho por núcleo é o que mais importa — ele determina a rapidez com que uma vista profunda se resolve depois que você se move. Núcleos extras ajudam principalmente na codificação da exportação em segundo plano. É necessária uma CPU de 64 bits.
help-hardware-cpu-recommended = Uma CPU moderna de 64 bits com boa velocidade por núcleo.
help-hardware-memory = Memória
help-hardware-memory-text = As referências profundas crescem com a profundidade (muitas iterações em alta precisão); exportações enormes mantêm blocos na RAM. 8 GB é confortável para a exploração do dia a dia; 16 GB ou mais é adequado para profundidade extrema e exportações muito grandes.
help-hardware-checking = Verificando o seu hardware
help-hardware-diagnostics = Ajuda → “Diagnóstico…” executa na sua máquina os mesmos testes que os desenvolvedores executam. “Executar autoteste” verifica a matemática e a renderização contra resultados sabidamente corretos em cerca de quinze segundos; “Executar teste de interface” percorre a interface e a vista ao vivo em várias profundidades de zoom, capturando telas; “Executar verificação aritmética da GPU” verifica a aritmética double-float do shader em todos os backends gráficos que a sua máquina oferece, em poucos segundos e sem janela. Cada um roda como um processo separado, então um teste que perturbe o driver gráfico derruba o teste e não a sua sessão.
help-hardware-gpu-check = A verificação aritmética da GPU não tem aprovação nem reprovação — ela registra como a sua GPU e o seu driver específicos calculam, e em alguns hardwares (todas as placas NVIDIA testadas até agora) ela informa que o compilador de shaders descarta parte da aritmética de precisão estendida. Isso é uma descoberta genuína que estamos tentando mapear entre fabricantes, não um defeito da sua máquina, então, diga ela o que disser, anexá-la a um relato realmente ajuda.
help-hardware-report = Se algo parecer errado, execute o autoteste e use “Anexar a um relato de problema…”: o relato passa a levar um resultado validado pela máquina, vindo do seu próprio hardware, o que é muito mais útil do que apenas uma descrição. Uma ressalva que vale conhecer — as imagens de referência foram produzidas numa placa de vídeo específica, e GPUs diferentes calculam legitimamente pixels ligeiramente diferentes, então as comparações de imagens são verificadas com uma tolerância maior em outros hardwares, e pequenas diferenças ali são esperadas, não defeitos.
help-hardware-notes = Observações
help-hardware-note-watchdog = Uma GPU capaz evita que o orçamento de trabalho do watchdog entre em ação em configurações extremas.
help-hardware-note-binary = A build é um único executável autocontido — nenhuma instalação de runtime é necessária.
help-hardware-note-benchmark = Execute o benchmark integrado (Ferramentas ▸ Benchmark) para avaliar a sua máquina.

## Help ▸ Acknowledgments & citations
## Names of people, programs, papers, books and websites stay as written, and so do the
## titles in quotation marks (the works' own titles); the prose around them translates.

help-ack-title = Agradecimentos e citações
# The dedication, in italics at the top of the section.
help-ack-dedication = Dedicado ao Stone Soup Group — os voluntários por trás do Fractint (desde 1988).
help-ack-dedication-text = O código que eles compartilharam livremente e o seu espírito de exploração aberta e colaborativa de fractais são o motivo pelo qual tantos de nós se apaixonaram por esta arte. O Fractadyne é construído nessa tradição.
help-ack-intro = O Fractadyne se apoia em décadas de trabalho da comunidade de fractais. Técnicas e trabalhos anteriores em que ele se baseia — verificados nas fontes abaixo:
help-ack-deep-zoom = Algoritmos de zoom profundo
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-martin-title = Perturbação e aproximação por séries — { $who }
help-ack-martin = O método da órbita de referência + delta de baixa precisão (δz → 2Z·δz + δz² + δc) e a série de ordem n que pula as primeiras iterações, apresentados no SuperFractalThing e na sua nota “SuperFractalThing Maths” (sft_maths.pdf, 2013).
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bla-title = Aproximação linear bivariada (BLA) e rebase — { $who }
help-ack-bla = Saltar sequências de iterações por meio de mapas lineares combinados, e fazer rebase para uma única referência para evitar glitches; desenvolvido no fractalforums.org (2021).
# A link's label. $who: the linked page's author.
help-ack-bla-link = BLA explicado — { $who }
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-glitch-title = Critério de detecção de glitches — { $who }
help-ack-glitch = O teste |Z + z| << |Z| para saber quando um pixel precisa de outra referência — a base da perturbação sem glitches (fractalforums).
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-diffabs-title = Perturbação não analítica (Burning Ship) — { $who }
help-ack-diffabs = A análise de casos “diffabs” do valor absoluto, que faz a perturbação funcionar para as famílias baseadas em abs (Burning Ship / Celtic / Buffalo), no fractalforums.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mathr-title = Implementações de referência e verificações cruzadas — { $who }
help-ack-mathr = Fraktaler-3 e Kalles Fraktaler 2+ (originalmente de Karl Runmo) — cujos textos guiaram este motor, e com cuja saída EXR o Fractadyne confere o seu renderizador.
# A heading: credits for the coloring methods.
help-ack-coloring = Coloração
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-harkonen-title = Contagem de iterações suave e Média de listras — { $who }
help-ack-harkonen = A coloração contínua (normalizada) por tempo de escape e o método Média de listras, de “On Smooth Fractal Coloring Techniques” (dissertação de mestrado, 2007), que também analisa as colorações Desigualdade triangular / média de ramificação.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mitchell-title = Média por desigualdade triangular — { $who }
help-ack-mitchell = Fazer a média de onde cada z² + c cai dentro dos limites da desigualdade triangular, para uma textura exterior suave, parecida com chamas.
# A link's label: the person's tutorial pages. $who: the person's name.
help-ack-mitchell-link = { $who } — tutoriais
help-ack-distance = A coloração/iluminação por estimativa de distância usa o método clássico de distância à fronteira por dz/dc (veja Peitgen & Saupe, “The Science of Fractal Images”, 1988).
help-ack-lsystems-life = Sistemas L e o Jogo da Vida
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-lindenmayer-title = Sistemas L — { $who }
help-ack-lindenmayer = Reescrita paralela de cadeias, introduzida em 1968 para modelar como plantas e algas crescem (“Mathematical models for cellular interactions in development”, Journal of Theoretical Biology).
help-ack-abop = O livro (1990) cuja interpretação por tartaruga e cuja notação para sistemas estocásticos, sensíveis ao contexto e paramétricos o leitor de sistemas L segue; os sistemas integrados marcados ABOP foram transcritos das figuras dele.
# A link's label: the book can be read free on that website. $site: the website's name.
help-ack-abop-link = O livro, gratuito em { $site }
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bourke-title = Coleção de sistemas L — { $who }
help-ack-bourke = Muitos dos sistemas integrados vêm das páginas de sistemas L dele, entre eles as regras de Chris Wallace para a curva de Sierpinski e o arbusto de Dietmar Saupe. Os arquivos .l do Fractint, que abrem diretamente, estão no formato do Stone Soup Group.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-conway-title = O Jogo da Vida — { $who }
help-ack-conway = O autômato celular B3/S23 (1970), que chegou ao mundo pela coluna “Mathematical Games” de Martin Gardner na Scientific American (outubro de 1970).
# A credit's heading: the website, then who runs it. $site: the website's name.
help-ack-lifewiki-title = { $site } — a comunidade ConwayLife.com
help-ack-lifewiki = A referência para as notações de regras que o aplicativo lê (B/S, von Neumann, Generations, as letras de Alan Hensel para regras não totalísticas, e MAP), para o formato de padrões RLE que ele compartilha com o Golly, e para as regras e padrões integrados, cujos descobridores ela registra.
help-ack-foundations = Fundamentos e ferramentas
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mandelbrot-title = O conjunto de Mandelbrot — { $who }
help-ack-mandelbrot = O objeto no coração de tudo isso.
help-ack-built-with = Feito com Rust, egui/eframe (emilk) e wgpu (gfx-rs); precisão arbitrária via astro-float, floats de palavra dupla via twofloat; PNG/OpenEXR, rfd, serde. Obrigado aos seus autores e mantenedores.
help-ack-errors = Quaisquer erros de atribuição são do autor; correções são bem-vindas no rastreador de issues do projeto.

## Help ▸ Third-party licenses

help-licenses-title = Licenças de terceiros
# $licenses: Fractadyne's license expression (MIT OR Apache-2.0), as written. $url: the web address of a crate's source. THIRD-PARTY-NOTICES.md is a file name, `option-ext` a crate's name, MPL-2.0 a license's.
help-licenses-intro = O Fractadyne é distribuído como um binário com ligação estática que inclui os crates de código aberto e as fontes empacotadas abaixo. Os textos completos das licenças (também em THIRD-PARTY-NOTICES.md, incluído no download) vêm a seguir. O código do próprio Fractadyne tem licença dupla { $licenses }. `option-ext` é MPL-2.0 — o código-fonte não modificado dele está em { $url }.
# A button, after a copy icon: copies every notice to the clipboard.
help-licenses-copy-all = Copiar todos os avisos

## Help ▸ About

help-about-tagline = Um explorador de fractais nativo feito em Rust com wgpu.
# The arithmetic deep zoom has used. $ran: the libraries that have run so far, as names (or a note that none has yet); $built: those this build contains, with versions.
help-about-arithmetic = Aritmética de zoom profundo: { $ran } (esta build contém: { $built }).
help-about-files = Arquivos e dados
# Followed by the folder's path.
help-about-files-text = A sua sessão (vista atual, coloração e preferências), os favoritos e suas miniaturas, os gradientes salvos e as fórmulas salvas são guardados nesta pasta do usuário e recarregados na próxima inicialização:
# Tooltip of the copy button beside the folder's path.
help-about-copy-path = Copiar caminho
# $command: a command typed in a terminal (it stays English).
help-about-session = O arquivo de sessão tem versão; se ele foi gravado por um Fractadyne mais recente do que este consegue ler por completo, o aplicativo o carrega da melhor forma possível e avisa você. Para começar totalmente do zero, use Arquivo ▸ Redefinir estado do aplicativo… (pergunta antes), ou execute `{ $command }` num terminal.
help-about-license = Licença
# $licenses: the license expression (MIT OR Apache-2.0), which stays as written.
help-about-license-text = { $licenses } — use sob qualquer uma das licenças, à sua escolha.
# A link to the source code, followed by ↗.
help-about-source = Código-fonte no GitHub

## The live render panel ("blinkenlights": View ▸ Live render panel, key B)
## A front panel drawn over the view like a piece of 1970s test equipment. Its legends are engraved
## in capitals: write them in capitals here (the panel shows them exactly as written, it does not
## uppercase them). Numbers, units and readouts on its tubes and dials are not translated.

# Tooltip of the × button in the panel's top-right corner. The code adds the key after it: (B).
live-close-hover = Fechar o painel de renderização ao vivo
# The legend engraved along the top plate, left of the maker's badge.
live-plate = RENDERIZAÇÃO AO VIVO · PAINEL FRONTAL

## Live render panel: instrument titles
## Engraved above each instrument, and repeated as the title of its help card.

# Above the depth readout, whose tubes show the magnification as a power of two ("2^" stays).
live-title-depth = PROFUNDIDADE 2^
live-title-iterations = ITERAÇÕES
# Above the frame-time readout (milliseconds).
live-title-frame-ms = QUADRO MS
# Above the date-and-time readout.
live-title-clock = DATA · HORA
# The walk dial: a radar screen showing a slow refresh rendered pass after pass, band after band.
live-title-walk = PERCURSO · FAIXAS
# A meter of how far the picture on screen lags the view.
live-title-age = IDADE DA IMAGEM
# What part of the picture this frame's GPU pass covered.
live-title-tiles = BLOCOS
# A scope of the GPU's work per frame, in milliseconds.
live-title-scope = PASSADAS DA GPU · MS
# The depth dial: depth, the reference orbits being built, and the arithmetic in use.
live-title-dial = PROFUNDIDADE · REFERÊNCIAS · ARITMÉTICA
# Iteration steps per second.
live-title-steps = PASSOS / S
# The two GPUs (the window's and a second, worker GPU).
live-title-engines = DOIS MOTORES
live-title-memory = MEMÓRIA
live-title-cores = NÚCLEOS · THREADS
live-title-pixels = FLUXO DE PIXELS
# Help card titles only: the three GPU timer lamps (T0, T1, T2 stay as they are) and the row of lamps.
live-title-timers = TEMPORIZADORES DA GPU T0–T2
live-title-bank = PAINEL DE LÂMPADAS

## Live render panel: the light bank's lamps
## Engraved under each lamp, and the title of its help card. A cell is narrow (about eight capital
## letters): a longer label breaks at a space onto two lines, and is drawn smaller if it still does
## not fit.

live-lamp-plan = PLANO
live-lamp-dispatch = DESPACHO
live-lamp-live = AO VIVO
live-lamp-held = RETIDA
# The last picture reprojected (stretched) to fit the moving view.
live-lamp-reproj = REPROJ
live-lamp-reading = LEITURA
# A pass of a pinned walk.
live-lamp-pin-pass = PASSADA FIXA
live-lamp-split = DIVISÃO
live-lamp-adopt = ADOÇÃO
live-lamp-abandon = ABANDONO
# "REF": a reference orbit. Built for the view as it is (LIVE), ahead of it (AHEAD), or for a tour's next hold (HOLD).
live-lamp-ref-live = REF ATUAL
live-lamp-ref-ahead = REF ADIANTE
live-lamp-ref-hold = REF PAUSA
# The series approximation.
live-lamp-series = SÉRIES
# The second (worker) GPU.
live-lamp-worker = AUXILIAR
live-lamp-tile = BLOCO
# Supersampling (anti-aliasing); usually kept as SSAA.
live-lamp-ssaa = SSAA
live-lamp-palette = PALETA
# The arithmetic in use (direct, df32, floatexp).
live-lamp-mode = MODO

## Live render panel: the kinds of GPU pass
## In the GPU scope's legend (MOVING, PIN, WORKER, SETTLE, SSAA). SETTLE and SSAA also name a still
## view's walk on the walk dial.

# A moving frame's pass.
live-kind-moving = MOVIMENTO
# A pass of a pinned walk.
live-kind-pin = FIXA
# The worker GPU's passes.
live-kind-worker = AUXILIAR
# A still view's settle tile or walk pass.
live-kind-settle = REFINO
# A supersampling sample.
live-kind-ssaa = SSAA

## Live render panel: legends and readouts on the instruments

# On the walk dial during a split refresh. $done: passes done; $passes: passes in all.
live-walk-split = DIVISÃO { $done }/{ $passes }
# On the picture-age meter's face, under its scale (0 to 2 octaves).
live-age-scale = OITAVAS DE ATRASO
# The depth dial's band for direct (plain floating-point) arithmetic. The other bands, DF32 and FLOATEXP, are names and stay.
live-dial-direct = DIRETO
# Under the two GPU gauges: GPU 0 draws the window; GPU 1 is the worker, or there is none.
live-engines-gpu-window = GPU 0 · JANELA
live-engines-gpu-worker = GPU 1 · AUXILIAR
live-engines-no-worker = SEM AUXILIAR
# The share of displayed frames each GPU made. $window and $worker: percentages, 0–100.
live-engines-on-screen = NA TELA { $window } : { $worker }
# The same readout before any frame was shown.
live-engines-on-screen-none = NA TELA —
# Under the CPU core bars. $count: the machine's cores.
live-cores-count = { $count } NÚCLEOS
# Under the threads scope. $count: the app's busy threads.
live-cores-threads = THREADS { $count }

## Live render panel: the memory tanks' legend
## What the GPU memory holds, by kind. Very narrow: about ten letters.

# G-buffers (the picture's working textures).
live-memory-gbuf = G-BUF
# The held picture and the supersampling accumulator.
live-memory-hold = RETIDA+SSAA
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
live-tiles-pin-band = PASSADA FIXA · FAIXA { $lo }–{ $hi }
live-tiles-settle-band = PASSADA DE REFINO · FAIXA { $lo }–{ $hi }
live-tiles-moving-whole = QUADRO EM MOVIMENTO · IMAGEM INTEIRA
live-tiles-settled-whole = QUADRO ESTÁVEL · IMAGEM INTEIRA
# $pass: this pass; $passes: the passes the refresh is split into.
live-tiles-split-pass = ATUALIZAÇÃO DIVIDIDA · PASSADA { $pass } DE { $passes }
# $grid: the grid's columns × rows, e.g. 12×8.
live-tiles-grid-done = GRADE DE REFINO { $grid } · CONCLUÍDA
# $grid: the grid's columns × rows; $done and $total: tiles done, and in all.
live-tiles-grid-progress = GRADE DE REFINO { $grid } · { $done } DE { $total } PRONTOS
# The same, while a tile too costly for one frame goes out in strips.
live-tiles-grid-strips = GRADE DE REFINO { $grid } · { $done } DE { $total } PRONTOS · EM TIRAS
# $samples: supersampling samples folded so far; $lo and $hi: the iteration band the next one is walking.
live-tiles-ssaa-band = SSAA · { $samples } PRONTAS · FAIXA { $lo }–{ $hi }
# $sample: the supersampling sample.
live-tiles-ssaa-whole = AMOSTRA SSAA { $sample } · QUADRO INTEIRO
# No GPU pass this frame, and the panel has not seen a frame yet.
live-tiles-idle-starting = INICIANDO
# No GPU pass this frame: the last complete picture is held on screen.
live-tiles-idle-held = RETIDA
# No GPU pass this frame: the last picture is stretched to fit the view.
live-tiles-idle-reprojected = REPROJETADA
# No GPU pass this frame: the picture on screen is current.
live-tiles-idle-live = AO VIVO
# No GPU pass this frame, and nothing to show.
live-tiles-idle-none = OCIOSO
# The tile size: $res, a whole picture's resolution (2400×1300).
live-tiles-tile-res = BLOCO { $res }
# The tile size: $px, a tile's side in pixels.
live-tiles-tile-px = BLOCO { $px } PX
live-tiles-no-pass = SEM PASSADA
# Tiles a frame. $count: how many (0 when none).
live-tiles-rate-count = { $count }/QUADRO
live-tiles-rate-one = 1 BLOCO/QUADRO
# $share: the part of one tile sent each frame, e.g. 0.33.
live-tiles-rate-part = { $share } BLOCO/QUADRO
live-tiles-rate-sample = 1 BLOCO/AMOSTRA
# In the plane's corner during a walk: the share of pixels still iterating. $percent: a percentage, e.g. 54 or 1.5.
live-tiles-running = RODANDO { $percent }%

## Live render panel: help cards
## Shown when the pointer rests on an instrument or a lamp. Words in capitals (SETTLE, SSAA, ABANDON,
## HELD, REPROJ, ADOPT, REF AHEAD, ON SCREEN) are the panel's own legends: use the same words as
## their messages above.

# The heading of a card's list of what healthy looks like.
live-card-healthy = SAUDÁVEL
# The heading of a card's list of what signals trouble.
live-card-trouble = PROBLEMA

live-card-walk-what = Uma atualização profunda cara demais para um quadro é fixada e percorrida: renderizada passada após passada, cada uma deixando os pixels iterarem uma oitava a mais (256, 512, 1.024 … até o valor pedido). A borda é essa contagem em escala logarítmica, começando no topo; o feixe é o progresso do percurso; cada arco é uma passada, colorida pelo seu tempo de GPU em relação à meta de 20 ms. O ponto branco espera a leitura da última passada; o clarão é a nova imagem entrando na tela.
live-card-walk-note = Uma vista parada cara demais para uma passada é percorrida da mesma forma, marcada REFINO, e cada amostra de superamostragem percorre de novo, marcada SSAA: cada uma das passadas delas pode levar até 400 ms, e os arcos são coloridos em relação a isso. Uma atualização dividida desenha, em vez disso, um anel por passada: todas as faixas, uma parte dos blocos.
live-card-walk-good-sweep = Uma varredura que dá a volta em poucos quadros e termina num clarão.
live-card-walk-good-arcs = Arcos na maioria verdes e âmbar.
live-card-walk-good-still = Numa vista profunda parada: uma varredura mais lenta por amostra, com arcos de verde a âmbar.
live-card-walk-bad-red = Arcos vermelhos: passadas acima do orçamento, sentidas como engasgos.
live-card-walk-bad-restart = Uma varredura que para no meio do caminho e recomeça: o percurso foi abandonado, e a imagem na tela continua velha.
live-card-walk-bad-hollow = Arcos vazados: passadas cuja medição nunca voltou, então a próxima passada é dimensionada às cegas.

live-card-age-what = Quanto a imagem na tela está atrasada em relação à vista, em oitavas de zoom (uma oitava é 2×). Enquanto uma atualização renderiza, a imagem antiga é mostrada esticada e envelhece; quando a atualização chega, a idade cai. A faixa abaixo mostra os últimos segundos.
live-card-age-good-sawtooth = Um dente de serra entre cerca de 0,3 e 0,8, com o ponteiro oscilando em torno da marca de 0,5.
live-card-age-good-still = Perto de 0 assim que você para.
live-card-age-bad-red = O ponteiro morando além de 1,0, no vermelho: a imagem está mais de 2× desatualizada e parece borrada ou em blocos durante o zoom.
live-card-age-bad-ramp = Uma rampa que nunca cai: as atualizações não estão terminando (observe ABANDONO).

live-card-tiles-what = Como a passada de GPU deste quadro cobre a imagem. Quadros em movimento e passadas de percurso a cobrem inteira, e a legenda nomeia a faixa de iterações de uma passada de percurso. Uma passada de percurso itera só os pixels que ainda não escaparam: o plano se preenche até essa fração (escala logarítmica, 0,01% no piso) e a mostra no canto, então ele se esvazia conforme o percurso se aprofunda. Uma atualização um pouco cara demais para um quadro é dividida em até 8 passadas, cada uma um conjunto intercalado de blocos de 16 px. Uma vista estabilizada cara demais para uma passada vira uma grade de blocos, um por quadro, do centro para fora, e um bloco ainda caro demais sai em tiras. Mostrados abaixo: tamanho do bloco e blocos por quadro.
live-card-tiles-good-whole = Passadas inteiras enquanto você dá zoom; atualizações divididas em poucas passadas.
live-card-tiles-good-grid = Quando você para: uma grade de dezenas de blocos preenchida em um ou dois segundos, depois SSAA.
live-card-tiles-bad-small = Uma grade de centenas de blocos pequenos: o orçamento de cada passada é pequeno, e a nitidez vai demorar muito.
live-card-tiles-bad-strips = Muitos blocos saindo em tiras: os blocos rodaram mais devagar do que o previsto no dimensionamento da grade.
live-card-tiles-bad-splits = Divisões presas em 8 passadas: a atualização mal cabe, e um pouco mais fundo ela recorre ao percurso, que é mais lento.
live-card-tiles-bad-dark = Escuro por longos períodos durante o movimento: nenhuma passada.

live-card-scope-what = Uma coluna por quadro, a mais recente à direita: o trabalho de GPU que ele executou, empilhado por tipo (quadro em movimento em verde, passada fixa de verde a vermelho conforme o custo, bloco de refino ou passada de percurso de vista parada em creme, amostra de superamostragem em lavanda, quadros e amostras da GPU auxiliar em azul), em milissegundos. Linhas tracejadas em 16,7 ms (60 fps), 33 e 50. Uma marca vazada é uma passada cuja medição nunca voltou.
live-card-scope-good-under = Colunas abaixo da linha de 16,7 enquanto você se move.
live-card-scope-good-pin = Passadas fixas de verde a âmbar.
live-card-scope-good-samples = As amostras de superamostragem de uma vista parada podem demorar mais: nada está se movendo.
live-card-scope-bad-over = Colunas além de 33 ou 50 ms: engasgos visíveis.
live-card-scope-bad-seconds = Passadas de segundos: o watchdog da GPU pode redefinir o dispositivo.
live-card-scope-bad-hollow = Muitas marcas vazadas: o trabalho está sendo dimensionado sem medições.

live-card-timers-what = Os três slots de timestamp da GPU. Um slot acende quando uma passada é cronometrada e apaga quando a leitura volta, dois ou três quadros depois. T0 cronometra a passada de cada quadro; T1 e T2 cronometram passadas fixas extras.
live-card-timers-good = T0 piscando sempre que há trabalho; T1 e T2 piscando durante os percursos.
live-card-timers-bad-stuck = Uma lâmpada presa acesa: a leitura dela não está voltando, e as estimativas de custo ficam defasadas.
live-card-timers-bad-dark = Todas apagadas enquanto a GPU está ocupada: nada está sendo medido.

live-card-dial-what = O ponteiro vermelho é a profundidade da vista: a ampliação como potência de dois. As faixas são a aritmética que a GPU usa ali: direta, df32 (dois floats por número) e floatexp (um float com expoente amplo) além de 2^93. Os pontos são órbitas de referência sendo construídas: verdes para a vista, âmbar à frente dela, para onde o zoom está indo, azuis para a próxima pausa de um tour. Um anel é uma instalação; um X vermelho é uma construção descartada porque a vista a ultrapassou.
live-card-dial-good-ahead = Pontos âmbar caindo à frente do ponteiro e ganhando um anel antes que ele os alcance.
live-card-dial-good-crosses = Poucos X vermelhos.
live-card-dial-bad-passing = O ponteiro passando da última referência instalada: os quadros esperam por uma, o que é sentido como um congelamento.
live-card-dial-bad-crosses = X vermelhos repetidos: o zoom está ultrapassando os construtores, e CPU é desperdiçada.
live-card-dial-bad-crossing = A travessia de 2^93 é um ponto lento conhecido: os primeiros quadros floatexp custam mais.

live-card-bank-what = Cada lâmpada acende quando o seu evento acontece e se apaga em cerca de 150 ms, então um renderizador ocupado cintila e um ocioso fica escuro. Da esquerda para a direita: o quadro, a atualização, as referências, depois o auxiliar, o refino, a paleta e a aritmética.
live-card-bank-note = Passe o cursor sobre uma lâmpada para ver o que ela significa.

live-card-depth-what = A ampliação como potência de dois: 2^100 é cerca de 1,3 × 10^30.
live-card-depth-note = Além de 2^93, cada passo custa mais (floatexp), e as referências demoram mais para ser construídas.

live-card-iter-what = O pedido de iterações: até onde cada pixel pode iterar antes de contar como dentro do conjunto. Ele sobe com a profundidade.
live-card-iter-good = Subindo de forma constante conforme você mergulha.
live-card-iter-bad-low = Preso num valor baixo em profundidade: espirais e minibrots são renderizados como interior chapado.
live-card-iter-bad-high = Muito alto: cada atualização custa mais, e os percursos demoram mais.

live-card-ms-what = O tempo entre os quadros exibidos.
live-card-ms-good = O intervalo da sua tela: 16,7 numa tela de 60 Hz.
live-card-ms-bad-stutter = Acima de 33: engasgos visíveis.
live-card-ms-bad-spikes = Picos acima de 50: travadas, geralmente uma passada cara (veja Passadas da GPU).

live-card-clock-what = O relógio no horário local, com precisão de um décimo de segundo.
live-card-clock-note = Ele marca qualquer captura de tela do painel com o momento em que foi tirada, para que a imagem de um problema possa ser comparada com o log do aplicativo.

live-card-tach-what = Passos de iteração por segundo. Branco: os passos pedidos (pixels × iterações). Âmbar: os passos que a GPU realmente executou. A diferença é o BLA (aproximação bilinear) saltando muitas iterações de uma vez; ×N é esse multiplicador. O odômetro totaliza a sessão.
live-card-tach-good = Uma grande diferença em profundidade: o BLA fazendo o seu trabalho, de ×10 a ×1.000.
live-card-tach-bad-together = Ponteiros juntos em profundidade: o BLA não consegue saltar aqui (muitas vezes perto do interior de um minibrot), então as atualizações são lentas.
live-card-tach-bad-red = O ponteiro branco no vermelho: pedindo mais do que a GPU consegue fazer num quadro.

live-card-engines-what = Quão ocupada está cada GPU, com pontos levando os quadros dela até a tela: o tempo de GPU das suas passadas no último segundo, incluindo os quadros de movimento e as amostras de superamostragem do auxiliar. NA TELA é a fração dos quadros exibidos que cada uma produziu. Com uma só GPU, o lado do auxiliar fica ocioso.
live-card-engines-good = As duas ocupadas durante um mergulho, com o auxiliar fornecendo uma parte dos quadros.
live-card-engines-bad-pinned = A GPU 0, a da janela, cravada perto de 100%: ela é o gargalo, e os quadros engasgam.
live-card-engines-bad-idle = Um auxiliar ativado mas ocioso: ele não está entregando quadros.

live-card-memory-what = RAM: o conjunto de trabalho do aplicativo, com a linha vermelha no pico. VRAM: o que o aplicativo mantém na GPU, calculado a partir dos tamanhos que ele aloca, por tipo (G-buffers, texturas retidas e de SSAA, buffers de órbita e BLA, a cópia da referência do auxiliar), em relação à placa.
live-card-memory-good = Estável durante o zoom; sobe e desce em degraus conforme as referências vêm e vão.
live-card-memory-bad-leak = RAM subindo e nunca descendo: um vazamento.
live-card-memory-bad-vram = VRAM perto do tamanho da placa: as alocações podem falhar, ou o dispositivo pode ser perdido.

live-card-cores-what = Uso de CPU por núcleo, de verde a âmbar a vermelho, e um osciloscópio em que cada thread ocupada orbita a sua função: os construtores de referência, o pool de pontuação, o auxiliar, as renderizações paralelas.
live-card-cores-good = Um ou dois núcleos ocupados durante o zoom; uma rajada curta em todos os núcleos quando uma referência é escolhida.
live-card-cores-bad-every = Todos os núcleos ocupados por segundos: construção de referência em profundidade extrema.
live-card-cores-bad-own = O núcleo do próprio aplicativo cravado no vermelho: a própria janela engasga.

live-card-pixels-what = Pixels calculados por segundo, em Mpx/s. Fraco: ainda iterando num percurso. Forte: concluído.
live-card-pixels-good = Dezenas de Mpx/s durante o movimento.
live-card-pixels-bad = Perto de zero enquanto a imagem está desatualizada: a GPU está esperando alguma coisa, uma referência ou uma leitura.

## Live render panel: the lamps' help cards

live-card-lamp-plan-what = Um quadro foi planejado: resolução, pedido de iterações e orçamento escolhidos, e se deve haver atualização.
live-card-lamp-plan-good = Cintila sempre que a vista ou as configurações mudam.
live-card-lamp-plan-bad = Apagada enquanto você dá zoom: a thread da interface está travada.

live-card-lamp-dispatch-what = Trabalho foi para a GPU neste quadro.
live-card-lamp-dispatch-good = Acesa enquanto algo muda; apagada quando ocioso.
live-card-lamp-dispatch-bad = Ocupada sem nada mudando: o aplicativo não está se estabilizando.

live-card-lamp-live-what = A imagem na tela é a renderização do próprio quadro.
live-card-lamp-live-note = Raramente acesa durante um mergulho profundo, o que é normal: veja RETIDA e REPROJ.
live-card-lamp-live-good = Acesa em profundidades rasas, onde um quadro é barato.

# "Prefer detail" is a setting's name.
live-card-lamp-held-what = A última imagem completa está na tela enquanto a próxima é composta por baixo dela: a atualização de um zoom profundo ou, com Preferir detalhes, o próximo bloco ou amostra de superamostragem de uma vista parada.
live-card-lamp-held-good-dive = Acesa em trechos durante zooms profundos, interrompida por ADOÇÃO.
live-card-lamp-held-good-still = Acesa de forma contínua enquanto uma vista parada ganha nitidez, com SSAA piscando.
live-card-lamp-held-bad = Acesa por segundos durante o zoom sem ADOÇÃO: as atualizações não estão terminando.

live-card-lamp-reproj-what = A última imagem, esticada para caber na vista em movimento até que uma nova esteja pronta.
live-card-lamp-reproj-good = Rajadas curtas enquanto você se move.
live-card-lamp-reproj-bad = Acesa por muito tempo: a imagem fica mais borrada quanto mais é esticada.

live-card-lamp-reading-what = Uma leitura voltou da GPU: a medição de tempo, a faixa de escape ou o conteúdo de uma passada.
live-card-lamp-reading-good = Uma cintilação constante durante a renderização.
live-card-lamp-reading-bad = Apagada por muito tempo durante a renderização: as medições não estão voltando, e os orçamentos estão sendo chutados.

live-card-lamp-pin-pass-what = Uma passada de um percurso fixado: a imagem renderizada uma faixa de iterações adiante.
live-card-lamp-pin-pass-good = Sequências de passadas que terminam em ADOÇÃO.
live-card-lamp-pin-pass-bad = Sequências que terminam em ABANDONO.

live-card-lamp-split-what = Uma passada de uma atualização dividida: uma parte dos blocos, renderizada com a contagem total de iterações.
live-card-lamp-split-good = Sequências curtas que terminam em ADOÇÃO.
live-card-lamp-split-bad = Sequências de 8, o máximo permitido: a atualização mal cabe.

live-card-lamp-adopt-what = Uma atualização concluída substituiu a imagem na tela.
live-card-lamp-adopt-good = Várias vezes por segundo durante o zoom.
# "Picture age" names the instrument (its title, PICTURE AGE).
live-card-lamp-adopt-bad = Rara durante o zoom: a imagem envelhece (veja Idade da imagem).

live-card-lamp-abandon-what = Uma atualização em andamento foi descartada: a vista se moveu demais, ela ficou velha demais, ou uma mais nova a substituiu. Uma construção de referência ultrapassada pela vista a acende fracamente.
live-card-lamp-abandon-good = Rara, ou uma vez logo após um salto.
live-card-lamp-abandon-bad = Frequente: trabalho desperdiçado, e a imagem continua velha.

live-card-lamp-ref-live-what = Uma órbita de referência está sendo construída para a vista como ela está: ela não tem nenhuma, saiu da que tinha, ou precisa de uma mais longa.
live-card-lamp-ref-live-good = Piscadas breves.
live-card-lamp-ref-live-bad-long = Trechos longos: as vistas profundas esperam por ela.
live-card-lamp-ref-live-bad-lookahead = Acesa com frequência durante um mergulho enquanto REF ADIANTE está apagada: a antecipação não está acompanhando.

live-card-lamp-ref-ahead-what = Uma referência antecipada está sendo construída mais fundo que a vista, para onde o zoom está indo.
live-card-lamp-ref-ahead-good = Piscadas regulares durante um mergulho.
live-card-lamp-ref-ahead-bad = Apagada durante um mergulho profundo: a vista vai ultrapassar as suas referências.

live-card-lamp-ref-hold-what = A pré-busca da pausa de um tour: a referência do próximo quadro-chave com pausa, construída durante o deslizamento até ele para que a reprodução nunca espere.
live-card-lamp-ref-hold-note = Apagada fora da reprodução de tours.
live-card-lamp-ref-hold-good = Uma piscada antes de cada pausa de um tour.
live-card-lamp-ref-hold-bad = Acesa durante uma pausa inteira: a pré-busca está atrasada, e o tour espera por ela.

live-card-lamp-series-what = A aproximação por séries para uma nova referência, que permite que cada pixel pule as primeiras iterações.
live-card-lamp-series-good = Breve.
live-card-lamp-series-bad = Longa em profundidade: atrasa a instalação da referência.

live-card-lamp-worker-what = A segunda GPU entregou um quadro.
live-card-lamp-worker-good = Piscadas constantes quando uma GPU auxiliar está ativada.
live-card-lamp-worker-bad = Apagada com um auxiliar ativado: ele não está entregando.

live-card-lamp-tile-what = Um bloco de refino, ou uma tira dele, foi para a GPU.
live-card-lamp-tile-good = Uma rajada depois que você para, e então silêncio.
live-card-lamp-tile-bad = Nunca para: a grade fica recomeçando, porque algo continua mudando a vista ou as configurações.

live-card-lamp-ssaa-what = Uma amostra de superamostragem foi adicionada à imagem estabilizada.
live-card-lamp-ssaa-good = Uma sequência de piscadas depois de um refino, e então apagada.
live-card-lamp-ssaa-bad = Recomeçando sem parar: o refino continua sendo perturbado.

live-card-lamp-palette-what = A normalização de cor recebeu uma nova faixa de escape, então as cores acompanham o zoom.
live-card-lamp-palette-good = Cintila durante o zoom.
live-card-lamp-palette-bad = Apagada durante o zoom: a faixa da paleta está congelada, e as cores ficam chapadas e com pouco contraste.

live-card-lamp-mode-what = A aritmética mudou: direta, df32 ou floatexp.
live-card-lamp-mode-good = Uma piscada numa fronteira como 2^93.
live-card-lamp-mode-bad = Piscando para lá e para cá: a vista está oscilando em torno de uma fronteira.

## The Render client window (File ▸ Render client…): lending this machine's GPU to a render farm

farm-client-title = Cliente de renderização
farm-client-intro = Renderize quadros para uma fazenda de renderização. O controlador — Ferramentas ▸ Renderizar na fazenda… em outra máquina — mostra o endereço e a chave da fazenda. Esta máquina não abre nenhuma porta: ela se conecta ao controlador.
# A field label, and the label of the status row that names the controller.
farm-client-controller = Controlador
farm-client-find = Procurar
farm-client-find-hover = Procurar controladores nesta rede (um controlador responde enquanto a fazenda dele está escutando)
farm-client-key = Chave da fazenda
# A checkbox beside the masked farm key: show it in clear.
farm-client-show-key = Mostrar
# A field label (the name this machine goes by), and the label of the status row about this machine.
farm-client-machine = Esta máquina
farm-client-machine-hover = O nome que o controlador mostra para esta máquina.
farm-client-share = Unidade compartilhada
# Greyed hint inside the empty field; the path is an example.
farm-client-share-hint = opcional — ex.: \\fileserver\share
farm-client-share-hover = O caminho desta máquina até a unidade compartilhada que o controlador usa (modo compartilhado): os quadros são gravados ali em vez de enviados pela conexão. Deixe vazio para enviar todos os quadros.
farm-client-gpu = Placa de vídeo
# The label of the row with the idle option.
farm-client-when = Quando
# A checkbox followed by a number of minutes ("Only when idle for [5] min").
farm-client-when-idle = Só quando ociosa por
farm-client-when-idle-hover = Aceitar trabalho só depois que ninguém tocar no teclado ou no mouse desta máquina por esse tempo, ou enquanto a tela dela estiver bloqueada. Quando alguém começa a usá-la, o quadro em andamento para na hora e volta para a fazenda.
# A collapsible section: the largest job this machine accepts.
farm-client-limits = Limites
farm-client-limits-note = Um trabalho que peça mais que isto é recusado, nunca reduzido.
farm-client-limit-size = Maior quadro
farm-client-limit-ss = Superamostragem máxima
farm-client-limit-iter = Iterações máximas

## Render client: the graphics-card choice

farm-client-gpu-system = A escolha do sistema
farm-client-gpu-every = Todas as placas de vídeo
# $count: this machine's graphics cards.
farm-client-gpu-every-n = Todas as placas de vídeo ({ $count })
# $count: this machine's graphics cards.
farm-client-gpu-every-each = Todas as placas de vídeo ({ $count }) — uma sessão cada
# A card the window cannot name. $number: its number (or numbers) as --list-adapters gives it.
farm-client-gpu-adapter = Adaptador { $number }
farm-client-gpu-finding = Procurando as placas de vídeo desta máquina…
# "NAME · GPU 1" is how such a machine is named in the controller's list: keep it as it is.
farm-client-gpu-hover = Qual placa de vídeo renderiza os quadros da fazenda. Com todas as placas, cada uma entra na fazenda como uma máquina própria — “NAME · GPU 1”, “NAME · GPU 2” — e Pausar e Desconectar agem sobre todas elas.

## Render client: looking for controllers on the network (the Find button)

farm-client-finding = Procurando controladores nesta rede…
# $port: the UDP port number.
farm-client-found-none = Nenhum controlador respondeu. Um controlador responde enquanto a fazenda dele está escutando; o firewall dele precisa deixar entrar UDP { $port }.
farm-client-found-pick = Nesta rede — clique em um para usá-lo:
# $identity: the controller's identity fingerprint.
farm-client-found-hover = Identidade { $identity } — usar este controlador (a chave da fazenda ainda é necessária)
# After a warning sign. $version and $git: this app's version and build.
farm-client-found-other-build = Uma build diferente desta ({ $version } { $git }): ele recusaria esta máquina.

## Render client: what is wrong with what was typed, or with starting the client

farm-client-bad-address = Insira o endereço do controlador como HOST:PORT, como a janela dele o mostra (por exemplo 192.168.1.20:46733).
# $why: what is wrong with the key (in English).
farm-client-bad-key = A chave da fazenda: { $why }
# $why: what is wrong with the name (in English).
farm-client-bad-name = O nome: { $why }
farm-client-no-config = Não há pasta de configuração para guardar a chave da fazenda.
# $why: the system's reason.
farm-client-start-failed = Não foi possível iniciar o cliente de renderização: { $why }

## Render client: the state, in the status heading

farm-client-phase-connecting = Conectando…
farm-client-phase-checking = Executando a autoverificação…
farm-client-phase-idle = Conectado — aguardando trabalho
farm-client-phase-rendering = Renderizando
farm-client-phase-in-use = Aguardando — alguém está usando esta máquina
farm-client-phase-paused = Pausado
farm-client-phase-retrying = Controlador inacessível — tentando de novo
# $seconds: the wait before the next try.
farm-client-phase-retrying-in = Controlador inacessível — tentando de novo em { $seconds } s
farm-client-phase-disconnected = Desconectado
farm-client-phase-ended = Encerrado

## Render client: the line under the status heading

# $address: the controller's address (HOST:PORT).
farm-client-detail-connecting = Conectando a { $address }…
# $controller: the controller's name.
farm-client-detail-checking = Conectado a “{ $controller }” — executando a autoverificação
# $controller: the controller's name.
farm-client-detail-approval = Conectado a “{ $controller }” — aguardando o usuário dele aprovar esta máquina
# $controller: the controller's name.
farm-client-detail-idle = Conectado a “{ $controller }” — ocioso
# $why: the check that failed and why (in English).
farm-client-detail-check-failed = A autoverificação falhou — { $why }
# $why: the controller's reason (in English).
farm-client-detail-refused = Recusado pelo controlador: { $why }
# $why: the controller's reason (in English).
farm-client-detail-closed = O controlador fechou a conexão: { $why }
# $why: the controller's reason (in English).
farm-client-detail-ended = O controlador encerrou a conexão: { $why }
# $job: the job's name.
farm-client-detail-bundle = Trabalho “{ $job }” — recebendo o pacote dele
# $job: the job's name.
farm-client-detail-job-waiting = Trabalho “{ $job }” — aguardando quadros para renderizar
# $why: the limit the job is over (in English).
farm-client-detail-job-refused = Este trabalho foi recusado: { $why }
farm-client-detail-job-over = O trabalho terminou
# $first and $last: frame numbers.
farm-client-detail-rendering = Renderizando os quadros { $first }–{ $last }
# $time: how long nobody must use the machine ("90 s", "5 min").
farm-client-detail-in-use = Aguardando — alguém está usando esta máquina; o trabalho recomeça depois que ninguém a usar por { $time }
farm-client-detail-paused = Pausado — o controlador foi avisado
farm-client-detail-pausing = Pausando — terminando o quadro em andamento
farm-client-detail-cancelled = O quadro em andamento foi cancelado; pausado
# A machine with several graphics cards. $rendering: the cards rendering now; $total: all its cards (two or more).
farm-client-rendering-on-cards = Renderizando em { $rendering } de { $total } placas de vídeo

## Render client: the status rows

# The value of the Controller row. $name: the controller's name; $address: HOST:PORT; $identity: its identity fingerprint.
farm-client-row-controller-value = “{ $name }” · { $address } · identidade { $identity }
# The value of the This machine row. $name: this machine's name; $identity: its identity fingerprint.
farm-client-row-machine-value = “{ $name }” · identidade { $identity }
farm-client-row-gpu = GPU
# One graphics card's row. $slot: the card's number.
farm-client-row-gpu-n = GPU { $slot }
farm-client-row-job = Trabalho
# The frames handed to this machine.
farm-client-row-run = Lote
farm-client-row-sent = Enviados
# The Run row, in parts joined with " · ": the run, the frame in progress, its time. $first and $last: frame numbers.
farm-client-run-frames = quadros { $first }–{ $last }
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-client-run-frame = quadro { $frame } ({ $index } de { $count })
# $time: the time spent on it so far ("2.1 s").
farm-client-run-frame-time = { $time } neste quadro
# One graphics card's run. $frame: the frame in progress; $first and $last: the run's frames.
farm-client-session-frame = quadro { $frame } de { $first }–{ $last }
farm-client-session-no-card = placa de vídeo ainda desconhecida
# The Sent row. $count: frames sent; $time: the mean time per frame ("2.1 s").
farm-client-sent =
    { $count ->
        [one] { $count } quadro · { $time } por quadro
       *[other] { $count } quadros · { $time } por quadro
    }
# The Sent row of a machine with several cards. $count: frames sent; $split: each card's share ("812 + 422"); $time: the mean time per frame.
farm-client-sent-split =
    { $count ->
        [one] { $count } quadro ({ $split }) · { $time } por quadro
       *[other] { $count } quadros ({ $split }) · { $time } por quadro
    }
# After a warning sign. $note: how the GPUs differ, and both GPUs (in English).
farm-client-gpu-note = Esta máquina tem { $note }. Os quadros dela vão diferir ligeiramente dos do controlador, o que pode aparecer numa cena parada.
# A collapsible section: the self-check's results.
farm-client-self-check = Autoverificação
farm-client-thumb-hover = O último quadro que esta máquina enviou
farm-client-resume = Retomar
farm-client-resume-hover = Voltar a aceitar trabalho
farm-client-pause = Pausar
farm-client-pause-hover = Terminar o quadro em andamento e depois não aceitar mais trabalho. O controlador passa os quadros desta máquina para as outras.
farm-client-cancel-frame = Cancelar quadro
farm-client-cancel-frame-hover = Parar agora o quadro em andamento e pausar; o controlador o renderiza em outro lugar.
farm-client-live-paused = Vista ao vivo pausada enquanto esta máquina renderiza quadros da fazenda.
farm-client-close = Fechar
farm-client-close-active-hover = Fechar a janela; esta máquina continua na fazenda (reabra-a pelo menu Arquivo)
farm-client-close-hover = Fechar a janela
farm-client-disconnect = Desconectar
farm-client-disconnect-hover = Sair da fazenda: o quadro em andamento é devolvido
farm-client-connect = Conectar
farm-client-connect-hover = Conectar-se ao controlador e renderizar quadros para ele

## The Render on farm window (Tools ▸ Render on farm…): the controller of a render farm

farm-ctl-title = Renderizar na fazenda
farm-ctl-out = Pasta de saída
farm-ctl-browse = Procurar…
farm-ctl-out-note = Os quadros, e o estado do trabalho em farm/: execute o mesmo trabalho de novo para retomá-lo.
farm-ctl-size = Tamanho
farm-ctl-size-custom = Personalizado…
# The size list's choice when the size is typed in. $width and $height: pixels.
farm-ctl-size-custom-value = Personalizado — { $width }×{ $height }
farm-ctl-fps = Quadros / segundo
farm-ctl-ss = Superamostragem
farm-ctl-port = Escutar na porta
farm-ctl-port-hover = Os clientes se conectam a esta porta. Libere-a no firewall para a sua rede local.
farm-ctl-share = Unidade compartilhada
# Greyed hint inside the empty field.
farm-ctl-share-hint = opcional — o caminho desta máquina até ela
farm-ctl-share-hover = Modo compartilhado: os clientes que têm a mesma unidade gravam os quadros nela em vez de enviá-los pela conexão, e esta máquina confere cada um ali. Os clientes sem ela enviam os seus.
# A label followed by a number of clients and two checkboxes.
farm-ctl-start-with = Começar com
# The unit after the number in the box ("Start with [3] clients"). $count: that number.
farm-ctl-start-clients =
    { $count ->
        [one] cliente
       *[other] clientes
    }
farm-ctl-start-clients-hover = O trabalho começa assim que esse número de máquinas tiver se conectado e passado na autoverificação; outras podem entrar depois.
farm-ctl-local = Renderizar também nesta máquina
farm-ctl-local-hover = Esta máquina conta como um cliente.
farm-ctl-sharing = Compartilhar referências
farm-ctl-sharing-hover = A órbita de referência de um quadro profundo pode levar de minutos a horas para ser construída. Compartilhada, ela é construída uma vez e passada às outras máquinas. Uma referência compartilhada é uma referência diferente, igualmente válida: em profundidade extrema, alguns pixels podem diferir dos de um quadro que construiu a sua própria.
farm-ctl-key = Chave da fazenda
# A checkbox beside the shortened farm key: show all of it.
farm-ctl-show-key = Mostrar
farm-ctl-copy-key = Copiar chave
farm-ctl-copy-key-hover = Cole-a na janela Cliente de renderização de cada cliente
# In place of the farm key, when there is none.
farm-ctl-no-key = nenhuma ainda
farm-ctl-make-key = Gerar uma chave
farm-ctl-make-key-hover = Os clientes precisam dela para entrar; ela também é gerada quando você clica em Renderizar
# What to do on the other machines. $address: this controller's HOST:PORT.
farm-ctl-steps = Em cada cliente: Arquivo ▸ Cliente de renderização…, controlador { $address } e a chave da fazenda.
# The same when this machine's address is not known. $port: the port number.
farm-ctl-steps-port = Em cada cliente: Arquivo ▸ Cliente de renderização…, o endereço desta máquina e a porta { $port }, e a chave da fazenda.
# The size of the job. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
farm-ctl-summary =
    { $count ->
        [one] { $count } quadro · { $seconds }s de tour · { $width }×{ $height } ss{ $ss }
       *[other] { $count } quadros · { $seconds }s de tour · { $width }×{ $height } ss{ $ss }
    }
farm-ctl-strip-hover = Todos os quadros do trabalho, da esquerda para a direita: prontos, sendo renderizados, ainda por fazer, abandonados.
farm-ctl-no-tour = Nenhum tour carregado.
# $folder: the folder; $why: the system's reason.
farm-ctl-out-error = A pasta de saída { $folder }: { $why }
# $why: the system's reason.
farm-ctl-start-failed = Não foi possível iniciar a fazenda: { $why }
# $code: the controller process's exit code, as the code gives it.
farm-ctl-ended-exit = O controlador terminou (código de saída { $code }).

## Render on farm: the state, in the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-phase-waiting = Aguardando clientes de renderização ({ $connected } de { $needed } conectados)
farm-ctl-phase-preparing = Preparando o trabalho
farm-ctl-phase-rendering = Renderizando
farm-ctl-phase-paused = Pausado
farm-ctl-phase-complete = Concluído
farm-ctl-phase-stopped = Parado
farm-ctl-phase-ended = Encerrado

## Render on farm: the words after the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-detail-waiting = Aguardando clientes de renderização: { $connected } de { $needed } conectados
farm-ctl-detail-preparing = Medindo as âncoras de normalização nesta máquina, uma vez para todos os clientes
farm-ctl-detail-no-space = Aguardando espaço na pasta de saída
farm-ctl-detail-paused = Pausado: nenhum trabalho novo é distribuído
farm-ctl-detail-not-rendered = Terminado com quadros não renderizados — execute de novo na mesma pasta para retomar
farm-ctl-detail-stopped = Parado — execute de novo na mesma pasta para retomar
# $code: the controller process's exit code.
farm-ctl-detail-ended = Encerrado (código de saída { $code })
farm-ctl-detail-finishing = Finalizando

## Render on farm: progress

# On the progress bar. $done: frames done; $total: frames in the job.
farm-ctl-progress =
    { $total ->
        [one] { $done } / { $total } quadro
       *[other] { $done } / { $total } quadros
    }
# Added to the progress bar. $rate: frames per second, with decimals.
farm-ctl-progress-rate = { $rate } quadros/s
# Added to the progress bar. $time: the time still to go ("38m 10s").
farm-ctl-progress-eta = faltam cerca de { $time }
# Facts under the progress bar, joined with " · ". $received and $sent: kilobytes per second.
farm-ctl-io = entrada { $received } KB/s · saída { $sent } KB/s
# $space: free space on the output drive ("412 GB").
farm-ctl-free = { $space } livres
# $time: the time since the job started ("38m 10s").
farm-ctl-elapsed = { $time } decorridos
# $count: frames given up on.
farm-ctl-failed = { $count } não renderizados
# Reference orbits. $built: built for this job; $cached: found in the cache; $reused: reused between frames; $shared: passed between machines.
farm-ctl-refs = referências: { $built } construídas, { $cached } do cache, { $reused } reutilizadas · { $shared } compartilhadas
# After a warning sign.
farm-ctl-storage-low = A pasta de saída está quase cheia: nenhum trabalho novo é distribuído até que haja espaço.
# After a warning sign.
farm-ctl-gpus-differ = As GPUs desta fazenda são diferentes. Os quadros delas diferem ligeiramente, o que pode aparecer numa cena parada.
# After a warning sign. $count: the GPU classes in the farm.
farm-ctl-gpus-differ-classes = As GPUs desta fazenda são diferentes ({ $count } classes de GPU: GPUs, drivers ou renderizações de sonda diferentes). Os quadros delas diferem ligeiramente, o que pode aparecer numa cena parada.
# $gpu: this machine's GPU, API and driver.
farm-ctl-this-gpu = Esta máquina: { $gpu }
# A machine whose GPU differs. $name: the machine; $note: how its GPU differs (in English); $adapter: its GPU.
farm-ctl-gpu-row = { $name }: { $note } — { $adapter }
# The same with its driver. $driver: the driver's name and version.
farm-ctl-gpu-row-driver = { $name }: { $note } — { $adapter }, driver { $driver }

## Render on farm: the table of machines

# Column headers (short: the table is narrow).
farm-ctl-col-machine = Máquina
farm-ctl-col-state = Estado
farm-ctl-col-gpu = GPU
farm-ctl-col-frames = Quadros
farm-ctl-col-per-frame = Por quadro
# How the machine delivers its frames (a speed, or "shared drive").
farm-ctl-col-link = Conexão
# When the controller last heard from the machine.
farm-ctl-col-heard = Contato
# In the machine's tooltip. $count: its frames that failed verification.
farm-ctl-strikes =
    { $count ->
        [one] { $count } quadro falhou na verificação
       *[other] { $count } quadros falharam na verificação
    }
# In the Link column: it delivers its frames through the shared drive.
farm-ctl-link-share = unidade compartilhada
# In the Heard column. $seconds: how long ago.
farm-ctl-heard-ago = há { $seconds } s
farm-ctl-readmit = Readmitir
farm-ctl-readmit-hover = Deixar esta máquina entrar de novo quando ela se reconectar
farm-ctl-remove = Remover
farm-ctl-remove-hover = Tirar esta máquina do trabalho; os quadros dela vão para as outras
# The GPU column's tooltip.
farm-ctl-probe-same = A renderização de sonda dela é idêntica à desta máquina.
# $count: the pixels that differ.
farm-ctl-probe-differs =
    { $count ->
        [one] A renderização de sonda dela difere da desta máquina em { $count } pixel: os quadros dela vão diferir ligeiramente dos desta máquina.
       *[other] A renderização de sonda dela difere da desta máquina em { $count } pixels: os quadros dela vão diferir ligeiramente dos desta máquina.
    }
farm-ctl-probe-unknown = A renderização de sonda dela ainda não foi comparada.
# $driver: the driver's name and version.
farm-ctl-gpu-driver = driver { $driver }
# After a warning sign. $note: how its GPU differs (in English: "a different GPU").
farm-ctl-gpu-note = { $note } em relação à desta máquina

## Render on farm: a machine's state, in the State column

# $count: its frames that failed verification.
farm-ctl-state-removed-bad =
    { $count ->
        [one] removida: { $count } quadro ruim
       *[other] removida: { $count } quadros ruins
    }
farm-ctl-state-removed = removida por você
farm-ctl-state-removing = removendo — coletando os diagnósticos dela
farm-ctl-state-gone = ausente
farm-ctl-state-leaving = saindo
farm-ctl-state-checking = verificando (autoverificação)
farm-ctl-state-paused = pausada pelo usuário dela
farm-ctl-state-parked = estacionada: travou duas vezes — volta quando se reconectar
farm-ctl-state-refused = recusou este trabalho (os próprios limites)
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-ctl-state-rendering = renderizando { $frame } ({ $index }/{ $count })
farm-ctl-state-starting = iniciando um lote
farm-ctl-state-idle = ociosa
farm-ctl-state-admitted = admitida — aguardando o trabalho

## Render on farm: the buttons

# A collapsible section: the controller's messages.
farm-ctl-events = Eventos
farm-ctl-resume = Retomar
farm-ctl-pause = Pausar
farm-ctl-pause-hover = Não distribuir trabalho novo; os quadros em andamento terminam
farm-ctl-live-paused = Vista ao vivo pausada enquanto esta máquina renderiza para a fazenda.
farm-ctl-close = Fechar
farm-ctl-close-active-hover = Fechar a janela; o trabalho continua rodando (reabra-a pelo menu Ferramentas)
farm-ctl-close-hover = Fechar a janela
farm-ctl-copy-command = Copiar comando
farm-ctl-copy-command-hover = Copiar a linha de comando equivalente
farm-ctl-stop = Parar
# The Stop button once a stop has been asked for and the job has not ended after 10 seconds.
farm-ctl-stop-now = Parar agora
farm-ctl-stop-hover = Encerrar o trabalho com os quadros que ele tem; executar o mesmo trabalho de novo o retoma
farm-ctl-render = Renderizar
farm-ctl-render-hover = Começar a escutar clientes e renderizar o tour neles

## The Render tour window (Tools ▸ Render tour…)

tour-render-title = Renderizar tour
tour-render-out = Pasta de saída
tour-render-browse = Procurar…
tour-render-out-hover = Caminho completo em que os quadros são gravados.
tour-render-prefix = Prefixo dos quadros
# <prefix> stands for the prefix typed in the field.
tour-render-prefix-hover = Os quadros são gravados como <prefixo>_00000.png
tour-render-size = Tamanho
tour-render-size-custom = Personalizado…
# The size list's choice when the size is typed in. $width and $height: pixels.
tour-render-size-custom-value = Personalizado — { $width }×{ $height }
tour-render-fps = Quadros / segundo
tour-render-fps-hover = Menos de 1 fps é legítimo: amostrar um mergulho longo a 0,25 fps é como as pausas profundas de regressão são inspecionadas sem renderizar milhares de quadros.
tour-render-ss = Superamostragem
tour-render-chapter = Capítulo
tour-render-whole = Tour inteiro
tour-render-chapter-hover = Renderizar um capítulo mantém a numeração GLOBAL dos quadros, para que eles se encaixem direto de volta numa sequência completa.
tour-render-mp4 = Montar mp4
tour-render-mp4-hover = Executa o ffmpeg depois (ele precisa estar no PATH). Os quadros PNG são mantidos.
tour-render-overwrite = Sobrescrever quadros existentes
tour-render-overwrite-hover = Desligado: a renderização para e pergunta, o que ela não consegue fazer a partir daqui.
tour-render-resume = Retomar uma renderização interrompida
tour-render-resume-hover = Manter os quadros que já estão na pasta de saída e renderizar só os que faltam. O quadro mais recente é verificado primeiro e descartado se estiver incompleto (o quadro em que uma renderização morre é o que ela estava gravando), recuando até encontrar um bom; uma pasta com quadros de outro tamanho é recusada em vez de ser misturada a esta sequência.
# $count: this machine's graphics cards.
tour-render-all-gpus = Usar todas as placas de vídeo ({ $count })
tour-render-all-gpus-hover = Renderizar em todas as placas de vídeo desta máquina ao mesmo tempo, por meio de uma fazenda de renderização neste computador (--gpus all). Os quadros, seus nomes, Retomar e o mp4 são como numa só placa; o tour inteiro é renderizado, na ordem da própria fazenda. Modelos de placa diferentes desenham alguns pixels de forma ligeiramente diferente: uma cena parada e uma fusão ficam numa só placa.
tour-render-order = Ordem de renderização
tour-render-sequential = Sequencial
tour-render-sequential-hover = Quadro 0, 1, 2, … em ordem (o padrão).
tour-render-progressive = Progressiva
tour-render-progressive-hover = Pré-visualizar o tour inteiro primeiro e refinar até o detalhe total: os quadros-chave são renderizados primeiro, depois os maiores intervalos temporais são divididos ao meio até que todos os quadros existam. Os quadros caem nos índices corretos de qualquer forma, então Retomar e o mp4 funcionam sem mudança — você pode parar cedo, dar uma olhada no flip-book grosseiro e recomeçar com Retomar em qualquer uma das ordens.
# The cost of the render. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
tour-render-summary =
    { $count ->
        [one] { $count } quadro · { $seconds }s de tour · { $width }×{ $height } ss{ $ss }
       *[other] { $count } quadros · { $seconds }s de tour · { $width }×{ $height } ss{ $ss }
    }
# The disk space the frames will take (an estimate). $size: a size ("5.3 GB").
tour-render-disk = ≈{ $size } de quadros
# $size: the frames' estimated size; $free: the free space on that drive ("6.0 GB").
tour-render-disk-free = ≈{ $size } de quadros · { $free } livres
# After a warning sign. $disk: the line above ("≈51.0 GB of frames · 6.0 GB free").
tour-render-disk-short = { $disk } — não há espaço para a sequência inteira
tour-render-separate = Renderiza o script A PARTIR DO DISCO num processo separado, então edições não salvas não são incluídas — e uma perda do dispositivo de GPU lá não pode derrubar esta sessão junto.
# On the progress bar. $done: frames done; $total: frames to render.
tour-render-progress =
    { $total ->
        [one] { $done } / { $total } quadro
       *[other] { $done } / { $total } quadros
    }
# Before the render's first progress line.
tour-render-starting = iniciando…
tour-render-live-paused = Vista ao vivo pausada — a renderização tem a GPU só para si, então termina mais cedo e mede com precisão os custos dos próprios quadros.
tour-render-cancel = Cancelar
tour-render-cancel-hover = Fechar sem renderizar
tour-render-copy-command = Copiar comando
tour-render-copy-command-hover = Copiar a linha de comando equivalente para a área de transferência
tour-render-stop = Parar renderização
tour-render-render = Renderizar
tour-render-render-hover = Começar a renderizar a sequência de quadros
# $folder: the output folder.
tour-render-finished = Renderização concluída → { $folder }
# $why: the render's own reason (in English).
tour-render-failed = Falha na renderização: { $why }
# $last: the render's last progress line (in English), or nothing.
tour-render-failed-or-stopped = A renderização falhou ou foi parada. { $last }
# $why: the system's reason.
tour-render-no-exe = Não foi possível encontrar o executável: { $why }
# $why: the system's reason.
tour-render-start-failed = Não foi possível iniciar a renderização: { $why }

## The Custom formula dialog (Fractal ▸ Custom formula…)
## The formula language itself stays as it is in every language: function names (sin, cosh, conj…),
## operators, variables (z, c, p1), and the formulas. Keep them as written inside these messages.

# The window's title.
formula-dialog-title = Fórmula personalizada
# A small line at the top of the window, above the formula.
formula-dialog-intro = O passo, em expressões no estilo do Fractint
# The two ways to show and edit the formula (top right of the window).
formula-dialog-textbook = Livro-texto
formula-dialog-textbook-hover = Mostrar a fórmula composta, como um livro-texto a comporia, e editá-la assim
formula-dialog-text = Texto
formula-dialog-text-hover = Editar a fórmula como texto
# The tooltip of the "LaTeX" button (the button's own text stays LaTeX).
formula-dialog-latex-hover = Copiar a fórmula como LaTeX, do jeito que está composta aqui
# The drop-down list of example formulas (the collection).
formula-dialog-examples = Exemplos
# Help under the formula in Textbook mode. The characters / ^ ( are what the user types: keep them.
formula-dialog-help-textbook = Digite como no texto: / transforma em fração o termo antes dela, ^ cria um expoente, ( abre parênteses. As setas percorrem a fórmula; Tab vai para a próxima caixa vazia. Uma linha em vermelho não é lida: clique nela para editá-la como texto.
# Help under the formula in Text mode. "; starts a comment": the ; is what the user types.
formula-dialog-help-text = Digite, ou use o teclado de fórmulas — ele contém todos os nomes que a linguagem de fórmulas conhece. Os nomes se completam enquanto você digita (Tab). As instruções são separadas por uma quebra de linha ou uma vírgula; ; inicia um comentário.
# The syntax check, when the formula reads.
formula-dialog-reads = Lida corretamente.
# The syntax check in Textbook mode, when the formula has an empty box to fill in.
formula-dialog-empty-box = Preencha a caixa vazia.
# The syntax check in Textbook mode. $line: the line number; $message: what is wrong (from the formula parser, in English).
formula-dialog-error-line = Linha { $line }: { $message }
# One function written with another notation's name, and the language's name for it: "ln to log".
# $from: the name as written (ln); $to: the formula language's name (log).
formula-respell-pair = { $from } para { $to }
# A button that rewrites them. $changes: one or more formula-respell-pair, joined with ", ".
formula-respell = Mudar { $changes }
formula-respell-hover = Escrever estas funções como a linguagem de fórmulas as nomeia
# Shown when Apply finds a parameter field that is not a number. $name: the parameter (p1 … p5).
formula-param-not-numbers = { $name } não é um par de números
# The Apply button (also in the Formula library window).
formula-apply = Aplicar
formula-dialog-apply-hover = Compilar a fórmula e mostrá-la
# Shown beside Apply while the formula is being compiled.
formula-dialog-compiling = Compilando a fórmula para a GPU…
# The greyed hint in an empty name box (also in the Formula library window).
formula-name-hint = nome (opcional)
# The save button: "Update" when the library already has a formula of that name.
formula-dialog-update-in-library = Atualizar na biblioteca
formula-dialog-save-to-library = Salvar na biblioteca
formula-dialog-save-hover = Guardar esta fórmula e seus parâmetros na biblioteca de fórmulas
formula-dialog-save-disabled-hover = Só uma fórmula lida corretamente pode ser salva
# Opens the Formula library window.
formula-dialog-library = Biblioteca…
formula-dialog-library-hover = As fórmulas salvas: aplique, edite, importe e exporte-as

## The Custom formula dialog's keypad
## The keys' captions are formula tokens and stay as they are; these are their tooltips.

# The keypad's third tab (the other two are "123" and "f(z)"). A short word.
formula-keypad-tab-names = nomes
# The tabs' tooltips.
formula-keypad-tab-basic-hint = Números, operadores e os nomes mais usados
formula-keypad-tab-functions-hint = Funções
formula-keypad-tab-names-hint = Todas as variáveis, parâmetros e constantes, e a sintaxe das instruções
# The key z.
formula-key-z = O iterado
# The key c.
formula-key-c = O pixel (a constante de Julia no modo Julia)
# The key p1 on the first tab.
formula-key-p1-below = Parâmetro 1 — definido nos campos abaixo
# The keys p1 … p5. $n: the parameter's number.
formula-key-param = Parâmetro { $n }
formula-key-multiply = Multiplicar
formula-key-divide = Dividir
# The key that types ^2.
formula-key-square-after = Quadrado (depois de um valor: z^2)
# The key that types ^.
formula-key-power = Potência: um expoente inteiro, real ou complexo
formula-key-square-root = Raiz quadrada
# The key that types |…|: in this language it is the squared modulus, as in Fractint.
formula-key-squared-modulus = Módulo AO QUADRADO |z|² — como no Fractint
formula-key-add = Somar
formula-key-subtract = Subtrair (ou negar)
# The key that types =.
formula-key-assign = Atribuir: z = …, ou uma temporária t = …
# The key that starts a new line (first tab).
formula-key-next-statement-comma = Próxima instrução (uma vírgula também serve)
# The key "del".
formula-key-backspace = Apagar o caractere antes do cursor (ou a seleção)
# The key e.
formula-key-euler = O número de Euler
# The key that types (0.5, 0.5).
formula-key-complex-constant = Uma constante complexa: (real, imaginária)
# The key that types a comma.
formula-key-comma = Separa instruções, ou as partes de uma constante complexa
formula-key-left = Cursor para a esquerda
formula-key-right = Cursor para a direita
# The function keys (the f(z) tab), one per function.
formula-key-sin = Seno
formula-key-cos = Cosseno
formula-key-tan = Tangente
formula-key-cotan = Cotangente
formula-key-exp = Exponencial e^z
formula-key-sinh = Seno hiperbólico
formula-key-cosh = Cosseno hiperbólico
formula-key-tanh = Tangente hiperbólica
formula-key-cotanh = Cotangente hiperbólica
formula-key-log = Logaritmo natural, ln (ramo principal)
formula-key-sqr = Quadrado: sqr(z) = z²
formula-key-sqrt = Raiz quadrada (principal)
formula-key-recip = Recíproco 1/z
formula-key-conj = Conjugado complexo
formula-key-flip = Trocar as partes real e imaginária
formula-key-abs = Valor absoluto das DUAS partes: |x| + i|y|
formula-key-cabs = Módulo |z| (um número real)
formula-key-real = Parte real
formula-key-imag = Parte imaginária, como um número real
# "fn" is Fractint's name for a function slot: keep it.
formula-key-ident = Identidade (o marcador de posição fn do Fractint)
# The key "pixel".
formula-key-pixel = O mesmo que c (o nome do Fractint)
# The key "z =".
formula-key-set-z = Iniciar uma instrução que define z
# The key "t =".
formula-key-temporary = Uma temporária: qualquer nome novo serve (t, w, a2, …)
# The key "; …".
formula-key-comment = Um comentário, até o fim da linha
# The key that starts a new line (names tab).
formula-key-next-statement = Próxima instrução

## The formula's completion list (names offered as you type)

# Beside a name the formula itself assigns.
formula-completion-variable = Uma variável que esta fórmula define
# Beside a function typed with another notation's name.
# $what: what the function is, in English (from the formula parser: "The natural logarithm");
# $other: the other notation's name (ln); $ours: the formula language's name (log).
formula-completion-other-spelling = { $what } — { $other } se escreve { $ours } aqui
# Under a list cut short. $count: the names not shown.
formula-completion-more = … mais { $count }
# The last line of the list. Tab, Enter and Esc are key names.
formula-completion-keys = Tab ou Enter para completar, Esc para fechar

## The Custom formula dialog's Textbook mode

# The tooltip of a line that does not read. $why: what is wrong (from the formula parser, in English, or formula-textbook-fractint-sections).
formula-textbook-unread-hover = Não é lida: { $why }. Clique para editá-la como texto.
# Why a line with Fractint's sections is shown as text in Textbook mode.
formula-textbook-fractint-sections = As seções do Fractint (o ':' de uma inicialização, comparações, a = b = …) são editadas como texto

## The Formula library window (Fractal ▸ Formula library…)

# The window's title.
formula-library-title = Biblioteca de fórmulas
# The window's two tabs. $count: the formulas in each.
formula-library-mine = Minhas fórmulas ({ $count })
formula-library-mine-hover = As fórmulas que você salvou ou importou
formula-library-collection = Coleção ({ $count })
formula-library-collection-hover = Fórmulas que vêm com o Fractadyne, cada uma com uma vista de partida
# A small marker before the name of the formula the view shows now. A short word.
formula-library-showing = exibindo
# The tooltip of Apply on a formula of the collection.
formula-library-apply-shipped-hover = Mostrá-la, na vista de partida dela
# A row's Edit button (after a pencil icon).
formula-library-edit = Editar
formula-library-edit-hover = Abri-la na janela Fórmula personalizada, sem aplicá-la
# A collection row's button that copies the formula into "My formulas".
formula-library-copy-to-mine = Copiar para as minhas
formula-library-copy-to-mine-hover = Adicioná-la às suas fórmulas, para guardar ou alterar
formula-library-close = Fechar
# Saves the formula the view shows, under the name typed beside it.
formula-library-add-current = Adicionar a fórmula atual
formula-library-add-current-hover = Salvar a fórmula que a vista mostra
# "Fractal > Custom formula…" is the menu path: write it as the menus are named.
formula-library-add-current-disabled-hover = Aplique uma fórmula personalizada primeiro (Fractal > Fórmula personalizada…)
formula-library-import = Importar…
formula-library-import-hover = Adicionar as fórmulas de um arquivo de fórmulas; nada aqui é substituído
formula-library-export-all = Exportar todas…
formula-library-export-all-hover = Gravar todas as fórmulas daqui num único arquivo, para compartilhar ou guardar
# The filter box: its label, and the greyed hint inside it.
formula-library-filter = Filtro
formula-library-filter-hint = filtrar por nome ou texto
# Shown while the user has no saved formulas.
formula-library-empty = Nenhuma fórmula salva ainda. Salve uma na janela Fórmula personalizada, adicione a que a vista mostra, ou importe um arquivo (nosso, ou um .frm do Fractint).
# The tooltip of Apply on one of the user's formulas: with a view saved, and without one.
formula-library-apply-view-hover = Mostrar esta fórmula, na vista salva com ela
formula-library-apply-hover = Mostrar esta fórmula
# The tooltip of a row's save icon.
formula-library-export-one-hover = Exportar esta fórmula para um arquivo
# A row asks before deleting: the question, then its two buttons. "Delete" is also the delete icon's tooltip.
formula-library-delete-ask = Excluir?
formula-library-delete = Excluir
formula-library-keep = Manter
# Under a list cut short. $count: the formulas not listed.
formula-library-more = …e mais { $count }: digite no filtro para encontrar uma.
formula-library-no-match = Nenhuma corresponde ao filtro.
# Under a saved formula this version cannot read. $why: what is wrong (from the formula parser, in English).
formula-library-unread = Não é lida nesta versão: { $why }

## Formula library notices

# $why: the system's reason, in English.
formula-library-save-failed = Não foi possível salvar a biblioteca de fórmulas: { $why }
formula-library-nothing-to-save = Nada para salvar: a fórmula está vazia.
# $name: the formula's name in the library.
formula-library-updated = “{ $name }” atualizada na biblioteca de fórmulas.
formula-library-saved = “{ $name }” salva na biblioteca de fórmulas.
# $name: the formula's name; $why: what is wrong.
formula-library-apply-failed = “{ $name }” não pode ser aplicada: { $why }
# Copying a formula of the collection into "My formulas". $name: its name; $as_named: the name it was given (the same, or "Name (2)" when the name was taken).
formula-library-copy-exists = “{ $name }” já está nas suas fórmulas.
formula-library-copied = “{ $name }” copiada para as suas fórmulas como “{ $as_named }”.
# $name: the formula's name.
formula-library-deleted = “{ $name }” excluída da biblioteca de fórmulas.
# Why the library could not be saved (follows formula-library-save-failed).
formula-library-no-config-dir = sem pasta de configuração
# At startup, when the library file cannot be read. $why: the reason, in English; $path: a file's full path.
formula-library-unreadable-kept = Não foi possível ler a biblioteca de fórmulas ({ $why }). Ela foi guardada como { $path } e uma biblioteca nova, vazia, foi iniciada.
formula-library-unreadable-stuck = Não foi possível ler a biblioteca de fórmulas ({ $why }), nem tirá-la do caminho: copie { $path } para um lugar seguro antes de salvar uma fórmula, o que a substitui.

## Formula library: Import and Export
## A formula file's contents stay in English; this is what the windows and notices around them say.

# File-type names in the open and save dialogs (the extensions are added by the system).
formula-filter-formula-files = Arquivos de fórmulas
formula-filter-fractadyne = Fórmulas do Fractadyne
formula-filter-fractint = Fórmulas do Fractint
formula-filter-all-files = Todos os arquivos
# Why a file cannot be imported (they follow formula-import-failed).
formula-import-too-large = grande demais para ser um arquivo de fórmulas
formula-import-not-text = não é um arquivo de texto
formula-import-no-formulas = não há fórmulas nele
# $why: the TOML reader's message, in English.
formula-file-not-formulas = não é um arquivo de fórmulas ({ $why })
# $format: what the file says it is ("fractadyne-gradients").
formula-file-other-format = um arquivo “{ $format }”, não um arquivo de fórmulas
# "[[formula]]" is the file's own syntax: keep it.
formula-file-no-formulas = não há fórmulas nele (um arquivo de fórmulas as lista como tabelas [[formula]])
# $file: the file's name; $why: one of the reasons above, or the system's (in English).
formula-import-failed = Não foi possível importar “{ $file }”: { $why }.
# After an import that added nothing. $file: the file's name.
formula-import-nothing-new = Nada de novo em “{ $file }”.
# After an import. $count: the formulas added; $file: the file's name.
formula-import-added =
    { $count ->
        [one] { $count } fórmula importada de “{ $file }”.
       *[other] { $count } fórmulas importadas de “{ $file }”.
    }
# The same two, followed by what else happened. $notes: one or more of the formula-import-note-… messages below, joined with "; ".
formula-import-nothing-new-notes = Nada de novo em “{ $file }”: { $notes }.
formula-import-added-notes =
    { $count ->
        [one] { $count } fórmula importada de “{ $file }”: { $notes }.
       *[other] { $count } fórmulas importadas de “{ $file }”: { $notes }.
    }
# Notes (clauses, no final full stop). $count: the formulas already in the library, skipped.
formula-import-note-duplicates =
    { $count ->
        [one] { $count } fórmula já estava na biblioteca
       *[other] { $count } fórmulas já estavam na biblioteca
    }
# One formula was renamed because its name was taken. $from: its name in the file; $to: the name it was given ("Name (2)").
formula-import-note-renamed-one = “{ $from }” já existia, então ficou “{ $to }”
# $count: the formulas renamed because their names were taken (2 or more).
formula-import-note-renamed = { $count } renomeadas porque o nome já existia
# $count: the entries skipped as empty or too long.
formula-import-note-skipped =
    { $count ->
        [one] { $count } entrada estava vazia ou longa demais e foi pulada
       *[other] { $count } entradas estavam vazias ou longas demais e foram puladas
    }
# A second sentence after importing a Fractint .frm file. $count: the entries that do not read; $why: the commonest reason (from the formula parser, in English).
formula-import-frm-unread =
    { $count ->
        [one] { $count } entrada não é lida nesta versão (motivo mais comum: { $why }).
       *[other] { $count } entradas não são lidas nesta versão (motivo mais comum: { $why }).
    }
# After exporting one formula. $name: the formula's name; $file: the file's name.
formula-export-one = “{ $name }” exportada para “{ $file }”.
# After exporting several. $count: how many; $file: the file's name.
formula-export-many =
    { $count ->
        [one] { $count } fórmula exportada para “{ $file }”.
       *[other] { $count } fórmulas exportadas para “{ $file }”.
    }
# $file: the file's name; $why: the system's reason, in English.
formula-export-failed = Não foi possível gravar “{ $file }”: { $why }

## The control panel (right-hand side): header and section headings

# The panel's title, beside the button that hides it.
panel-title = Controles
panel-hide-hover = Ocultar o painel de controles
# The button that brings the hidden panel back (top right of the view).
panel-show-hover = Mostrar o painel de controles
# Section headings.
panel-section-navigate = Navegar
panel-section-coloring = Coloração
panel-section-quality = Qualidade
panel-section-effects = Efeitos
panel-section-overlays = Sobreposições
panel-section-advanced = Avançado
# The last section: reference notes about the current fractal. $name: the fractal's name (Mandelbrot, Burning Ship…).
panel-section-about = Sobre o { $name }
panel-section-performance = Desempenho

## The control panel: Navigate
## Labels before a control sit in a column about 104 px wide (roughly 15 characters): a longer label pushes its control to the right.

panel-zoom-speed = Velocidade do zoom
panel-zoom-speed-hover = Velocidade do zoom contínuo ao segurar Space (1× ≈ 2× a cada 1,5 s).
panel-snap-zoom = Atrair o zoom às estruturas
panel-snap-zoom-hover = Ao aproximar com a roda do mouse ou com Space, puxar o zoom em direção a um minibrot (ou ao eixo real) perto do cursor, para que ele deslize para baixo do cursor e o zoom pouse nele. Quanto mais perto a estrutura, mais forte a atração. Somente Mandelbrot, vista única.
panel-click-zoom = Clicar para dar zoom
panel-click-zoom-hover = Quando ligado, um clique esquerdo na vista mergulha pelo fator abaixo (o clique direito recua), recentralizando no ponto clicado. Arrastar ainda move a vista; Shift+arrastar / arrastar com o botão direito ainda dão zoom por retângulo. Backspace desfaz um clique. Na vista dupla, funciona em qualquer um dos painéis, e fixar o c da Julia passa para Ctrl+clique enquanto isto estiver ligado. Segure Shift para abrir uma lupa sob o cursor mostrando a vista à qual o clique levaria você, e clique enquanto o segura para ir a esse ponto; Shift+ARRASTAR ainda dá zoom por retângulo.
# The click-to-zoom factor, followed by buttons 2×, 4×, 10×…
panel-click-zoom-factor = Fator
panel-click-zoom-snap = Encaixar no centro mais próximo
panel-click-zoom-snap-hover = Depois do salto, calcular o núcleo de minibrot mais próximo e se encaixar exatamente nele, para que você caia sobre a estrutura em vez de perto dela. O zoom não muda — o clique o escolheu. Somente Mandelbrot. O cálculo roda em segundo plano: a vista salta na hora e se corrige quando a resposta chega, o que em profundidade extrema pode levar alguns segundos.

## The control panel: Coloring
## Labels before a control sit in a column about 104 px wide (roughly 15 characters).

panel-method = Método
panel-method-hover = Como os dados de escape viram cor. Listras / desigualdade triangular / armadilha de órbita / decomposição revelam a estrutura da órbita; distância sombreia pela proximidade da borda.
panel-stripe-density = Densidade de listras
panel-stripe-tail = Só a cauda (zoom profundo)
panel-stripe-tail-hover = Fazer a média do termo de listras nas últimas N iterações de cada órbita (uma janela exponencial) em vez da órbita inteira. Em profundidade extrema, toda órbita acompanha a referência em todas as iterações exceto nas últimas centenas, então a média da órbita inteira dá uma só cor para a vista toda; a cauda mantém o contraste. Órbitas mais curtas que a janela não são afetadas. Também --stripe-tail N para --render.
panel-tail-length = Compr. da cauda
# The unit after the tail length's number: short for "iterations" (the slider reads "64 it").
panel-tail-length-unit = it
panel-tail-length-hover = Comprimento da janela em iterações. Janelas curtas (8–32) dão as faixas fortes clássicas em qualquer profundidade; as mais longas voltam a tender a uma cor chapada, já que o termo de listras de uma órbita caótica longa se anula na média.
panel-trap-shape = Forma da armadilha
panel-palette = Paleta
# The two colours of the Binary palette, each beside its colour swatch.
panel-in-set = Dentro
panel-out-of-set = Fora
# The two colours of the Duotone palette, each beside its colour swatch.
panel-shadow = Sombra
panel-highlight = Realce
panel-cycle = Ciclo
panel-offset = Deslocamento
panel-normalize-hover = Em profundidade extrema, as contagens de escape abrangem centenas de milhares e um Ciclo fixo dá a volta na paleta milhares de vezes entre pixels vizinhos — um campo denso correto parece ruído granulado. Isto remapeia a paleta para a faixa de escape medida na vista (o Ciclo então define quantas varreduras da paleta a abrangem) QUANDO a vista realmente tem aliasing — uma vista suave fica como está, porque remapeá-la mudaria a imagem à toa. Marque “Sempre ajustar a faixa” abaixo para remapear mesmo assim. Somente com o método Iteração suave; corresponde a --normalize.
# A checkbox indented under "Normalize deep colors" (also in the Color menu).
panel-fit-range = Sempre ajustar a faixa
panel-fit-range-hover =
    Remapear a paleta para a faixa de escape medida na vista mesmo quando a vista não tem aliasing — a caixa de seleção acima só remapeia quando um campo denso pareceria granulação.

    Use isto quando você quiser que a paleta abranja o que está na tela, e não para corrigir ruído. Ainda é preciso uma medição, então isto não faz nada durante o instante ou dois antes de a vista se estabilizar, e nada mesmo numa vista sem pixels que escaparam.
# A checkbox indented under "Always fit range" (also in the Color menu).
panel-log-scale = Escala de cor logarítmica
panel-log-scale-hover =
    Distribuir a paleta pelo LOGARITMO do valor de escape em vez de linearmente. As contagens de escape se acumulam no extremo alto em profundidade, então um mapeamento linear gasta a maior parte da paleta numa casca fina perto da borda e achata todo o resto.

    Aplica-se ao mapeamento normalizado — que é o que “Normalizar cores profundas” (ou --normalize numa exportação) liga. Uma faixa muito assimétrica recebe o mapeamento logarítmico de qualquer forma, esteja isto marcado ou não.
# The same checkbox's tooltip while it is greyed out (normalization is off).
panel-log-scale-off-hover =
    Requer normalização: isto escolhe COMO a paleta é mapeada para a faixa de escape medida, então com “Normalizar cores profundas” desligado (e sem exportação com --normalize) não há mapeamento para mudar, e isto não alteraria um único pixel.

    Ligue “Normalizar cores profundas” para usá-lo.
# The palette animation: its kind, then its speed.
panel-animate = Animar
panel-anim-speed = Velocidade
panel-anim-speed-hover = Velocidade do ciclo: ciclos de deslocamento de cor por segundo, ou (Gradientes aleatórios) trocas de gradiente por segundo.
panel-shuffle-gradient = Embaralhar gradiente

## The control panel: Quality

panel-auto-iter = Escalar iterações com o zoom
# The iteration-count label while auto-scaling is on (the count is then a base the zoom raises).
panel-iterations-base = Iterações (base)
panel-iterations = Iterações
panel-iterations-hover = Contagem base de iterações. Com “Escalar iterações com o zoom” ligado, a contagem efetiva sobe com a profundidade do zoom. Enquanto você se move, a pré-visualização limita as iterações a um valor baixo (50.000) para continuar responsiva, e depois ganha nitidez com a contagem total quando a vista se estabiliza — então bordas profundas parecem suaves durante o movimento e se resolvem quando você para.
# A warning under the iteration count. $count: the iterations the settled preview renders.
panel-res-limited = Nesta profundidade, a pré-visualização estabilizada renderiza { $count } iterações em resolução reduzida para continuar responsiva. Exporte para ter detalhes em resolução total.
panel-anti-alias = Anti-aliasing
# Anti-aliasing off (the other choices are 2×, 3×, 4×, 8×).
panel-anti-alias-off = Desligado
panel-anti-alias-hover = Superamostragem para imagens paradas (aplicada quando a vista se estabiliza). Valores maiores domam a fina “poeira” exterior, ao custo de tempo de renderização.

## The control panel: Effects

panel-relief-lighting = Iluminação de relevo 3D
panel-relief-lighting-hover = Sombrear a superfície usando a inclinação da estimativa de distância — um visual em alto-relevo, iluminado. (Famílias holomorfas: Mandelbrot / Multibrot.)
panel-light-angle = Ângulo da luz
panel-relief = Relevo
panel-relief-hover = Menor = relevo mais nítido; maior = mais suave/plano.
panel-rotate-light = Girar a luz
panel-rotate-light-hover = Girar a direção da luz com o tempo (usa o controle Velocidade).
panel-distance-glow = Brilho de distância
panel-distance-glow-hover = Faixas de contorno brilhantes da estimativa de distância, que se adensam em filamentos luminosos perto da borda. (Famílias holomorfas.)
panel-glow = Brilho
panel-band-width = Largura da faixa
panel-band-width-hover = Espaçamento dos contornos de distância (oitavas por faixa).
panel-animate-glow = Animar o brilho
panel-animate-glow-hover = Fazer as faixas de brilho fluírem com o tempo (usa o controle Velocidade).

## The control panel: Overlays

panel-orbits-hover = Desenhar o caminho de iteração do ponto sob o cursor.
panel-orbit-anim = Animar (ponto correndo)
panel-orbit-anim-hover = Lançar um ponto de cor cíclica correndo ao longo da órbita.
panel-orbit-speed = Veloc. da órbita
panel-minimap-hover = Uma pequena visão geral da vista inicial com um marcador “você está aqui” e a profundidade do zoom. Clique nela para saltar para uma região.

## The control panel: Advanced
## Labels before a control sit in a column about 104 px wide; most of these are already wider in English.

# Small headings of the section's two groups.
panel-accelerators = Aceleradores
panel-performance-tuning = Ajuste de desempenho
panel-bla = Aceleração BLA (zoom profundo)
panel-bla-hover = Aproximação bilinear: pula iterações ao longo de toda a órbita em profundidade extrema (Mandelbrot floatexp, ≥1e28×) — renderização na GPU ~5× mais rápida, saída idêntica (verificada pelo autoteste). Ligada por padrão; desligue para comparar ou se encontrar algum artefato.
panel-series-approx = Aproximação por séries
panel-series-approx-hover = Iniciar a perturbação a partir de um polinômio para pular as primeiras iterações, onde o BLA não está ativo (profundidades df32, Multibrot, BLA desligado). Saída idêntica; desligue para comparar.
panel-glitch-correct = Correção de glitches (exportação)
panel-glitch-correct-hover = Correção de glitches com várias referências para imagens exportadas: detecta glitches de perturbação e renderiza esses pixels de novo com referências extras até ficarem limpos. Ligada por padrão. Exportações de até ~32 MP (coloração sem dados auxiliares); imagens maiores e a vista ao vivo usam o caminho simples. Nunca usada em vistas de Mandelbrot ou Multibrot (fora do modo Julia): comparada com um cálculo exato, ela não corrigiu nada ali e custou até 5x o tempo de exportação.
panel-dive-limit = Limite de mergulho do zoom automático
panel-dive-limit-hover = Profundidade em que o zoom automático (tecla A) para. Até ~1e271× ele desliza suavemente; mais fundo, muda para um mergulho em degraus, entrecortado, para chegar rápido à profundidade extrema.
# Followed by two choices, Speed and Quality.
panel-autopilot-priority = Prioridade do zoom automático
panel-priority-speed-hover = Dar zoom na velocidade definida. A tela é atualizada assim que um quadro com detalhes fica pronto, na resolução que mantiver as atualizações chegando (abaixo do piso de Resolução mín. em movimento se o zoom ultrapassar a renderização); entre as atualizações, o último quadro acompanha o zoom e perde nitidez.
panel-priority-quality-hover = Cada atualização é um quadro totalmente resolvido em resolução total, e o zoom desacelera conforme necessário para que o quadro na tela nunca seja ampliado demais antes de o próximo chegar. Um mergulho mais lento, nítido a todo momento.
# Followed by two choices, Detail and Misiurewicz point.
panel-autopilot-target = Alvo do zoom automático
panel-target-detail-hover = Mergulhar em direção à estrutura mais rica da borda do fractal na vista, escolhendo o alvo de novo conforme o mergulho avança.
panel-target-misiurewicz-hover = Encontrar o ponto de Misiurewicz mais próximo (o centro de uma espiral cuja estrutura se repete para sempre) e mergulhar exatamente nele. O mergulho primeiro fica parado enquanto o ponto é calculado até a profundidade do limite de mergulho — um instante em profundidades comuns, mais tempo além de 1e1000×. Somente no conjunto de Mandelbrot.
panel-work-budget = Orçamento ao vivo
panel-work-budget-hover = Detalhe vs. velocidade da vista ao vivo em zoom profundo. Valores maiores renderizam em resolução mais completa (mais nítido — menos do aspecto “suave” ampliado), mas reduzem a taxa de quadros, e valores muito altos arriscam uma breve parada da GPU nos quadros mais pesados. As exportações são sempre em resolução total, independentemente disto.
panel-min-motion-res = Resolução mín. em movimento
panel-min-motion-res-hover = A menor resolução a que a vista ao vivo pode cair durante zoom ou movimento contínuos em zoom profundo. Valores maiores impedem que um mergulho rápido fique em blocos/pixelado (mais nítido), ao custo da taxa de quadros — a 100%, as atualizações de detalhe de um mergulho profundo precisam ser renderizadas com custo total, o que pode fazer o zoom profundo avançar aos solavancos de forma visível. Se um mergulho profundo parecer engasgado, diminua isto. Padrão 30%; não afeta a imagem estabilizada nem as exportações.
panel-finish-sound = Som quando uma renderização termina
panel-finish-sound-hover = Tocar a melodia de conclusão do FRACTINT quando uma exportação ou renderização de tour termina: três notas ascendentes (1047, 1109, 1175 Hz por 100 ms cada), exatamente como o original para DOS as codificava — sintetizadas como uma onda quadrada de banda limitada e filtradas por um passa-altas para soar como o pequeno cone do alto-falante do PC que as tocava.
panel-prefer-detail = Preferir detalhes durante o zoom
panel-prefer-detail-hover = Durante o zoom ou o movimento, continuar mostrando o último quadro totalmente detalhado — escalado e acompanhando o movimento — em vez de renderizar de novo com qualidade reduzida a cada quadro. A vista é renderizada por completo quando o movimento pausa. Desligado: o movimento é renderizado ao vivo em resolução reduzida (mais suave, mais grosseiro). Vistas rasas (modo direto) sempre são renderizadas ao vivo — elas são baratas e nítidas a cada quadro de qualquer forma.

## The control panel: About (the current fractal)

# A link to the fractal's reference page, followed by an arrow.
panel-about-reference = Referência

## The control panel: Advanced ▸ Second graphics card

# A label before a drop-down (wider than the panel's label column already).
gpu2-label = Segunda placa de vídeo
gpu2-off = Desligada
# Another device on the same graphics card as the window (the default).
gpu2-same-card = Mesma placa
# A card the list does not name. $number: the card's number.
gpu2-card-n = Placa { $number }
gpu2-finding = Procurando as placas de vídeo desta máquina…
gpu2-none = Esta máquina não tem outra placa de vídeo.
gpu2-hover =
    Outro dispositivo gráfico renderiza quadros de uma vista profunda enquanto ela se move, e algumas das amostras extras que a deixam nítida depois que ela para, para que a imagem acompanhe um zoom e chegue mais cedo à qualidade final. A placa que desenha a janela continua desenhando tudo.

    Outra placa acrescenta a sua própria velocidade. Dois modelos de placa diferentes desenham alguns pixels de forma ligeiramente diferente; se a segunda placa falhar, a vista continua com uma só.

    “Mesma placa” (o padrão) abre um segundo dispositivo nesta placa. Não acrescenta velocidade, mas renderiza sem esperar pelos quadros da tela. Ele compartilha a placa: uma falha que redefine a placa encerra o aplicativo, como uma falha da renderização da própria janela, e a próxima inicialização oferece desligá-lo.

## The view (canvas)

# A notice after Ctrl+click pins the Julia c at a deep zoom.
canvas-julia-pin-quantized = c da Julia fixado — mas ele é mantido em precisão dupla, então nesta profundidade o painel inteiro se arredonda para um único valor e a fixação cai no centro da vista. Afaste o zoom para mirar.
# A small caption in the top-left corner of the view while the orbit overlay is fitted to the view (its "Normalize (fit to view)" option).
canvas-orbit-normalized = órbita (normalizada)
# The minimap's tooltip.
canvas-minimap-hover = Visão geral / você está aqui. Arraste para mover na ampliação atual, role para aproximar e afastar, clique para saltar para aquela região (zoom inicial).

## The timestamp overlay (View ▸ Show timestamp)

# Under the large elapsed-time clock. $frame: the frame number (as the log writes it, without grouping).
overlay-timestamp-frame = quadro { $frame }

## File ▸ Settings ▸ Reference cache…

cache-title = Cache de referências
cache-intro = Em profundidade extrema, a parte lenta de uma vista é a sua órbita de referência — até uma hora de aritmética de precisão arbitrária. O Fractadyne guarda aqui as órbitas prontas, para que voltar a um local que você já visitou, ou dar zoom em algum lugar perto dele, leve só segundos. Uma órbita tem cerca de 4 MB.
cache-off = Desligado nesta execução (uma invocação de tarefa, ou --no-orbit-cache). Nada é lido ou gravado até a próxima inicialização normal.
# Small headings of the window's parts.
cache-location = Localização
cache-usage = Uso
cache-limit = Limite
cache-no-config = (nenhuma pasta de configuração disponível)
cache-open-folder = Abrir pasta
# The text on the usage bar. $used and $limit: sizes such as "1.2 GB"; $count: the orbits kept.
cache-usage-bar =
    { $count ->
        [one] { $used } de { $limit } — { $count } órbita
       *[other] { $used } de { $limit } — { $count } órbitas
    }
cache-limit-hover = Quando o cache passa disto, as órbitas que foram MAIS BARATAS de construir são removidas primeiro — uma órbita profunda que levou uma hora sobrevive a qualquer número de órbitas rápidas, por mais recentemente que elas tenham sido usadas.
# Beside the limit. $count: about how many orbits of the live view's length fit in it.
cache-limit-estimate =
    { $count ->
        [one] ≈ { $count } órbita do comprimento máximo da vista ao vivo
       *[other] ≈ { $count } órbitas do comprimento máximo da vista ao vivo
    }
cache-kept = Guardadas (as mais valiosas primeiro)
# Column headers of the list of kept orbits.
cache-col-iterations = iterações
cache-col-precision = precisão
cache-col-size = tamanho
cache-col-last-used = último uso
# The precision column. $bits: the precision in bits.
cache-row-bits = { $bits } bits
# Under the list. $count: the orbits not listed.
cache-rows-more = … e mais { $count }
# When an orbit was last used (the "last used" column).
cache-ago-now = agora mesmo
# $count: minutes.
cache-ago-minutes = há { $count } min
# $count: hours.
cache-ago-hours = há { $count } h
# $count: days.
cache-ago-days = há { $count } d
# Asked before clearing. $count: the orbits; $size: their total size, such as "1.2 GB".
cache-confirm =
    { $count ->
        [one] Excluir { $count } órbita ({ $size })? Reconstruí-la custa tempo, não dados.
       *[other] Excluir { $count } órbitas ({ $size })? Reconstruí-las custa tempo, não dados.
    }
cache-delete = Excluir
cache-cancel = Cancelar
cache-clear = Limpar cache…
cache-clear-hover = Exclui todas as órbitas armazenadas. Pergunta antes.
cache-close = Fechar
# $count: the orbits removed.
cache-cleared =
    { $count ->
        [one] Cache de referências limpo — { $count } órbita removida.
       *[other] Cache de referências limpo — { $count } órbitas removidas.
    }
# $why: the system's error message.
cache-clear-failed = Não foi possível limpar o cache de referências: { $why }

## Help ▸ Diagnostics…

diag-title = Diagnóstico
diag-intro = Execute um teste para verificar se o Fractadyne funciona corretamente no seu hardware. Os resultados são salvos na sua máquina e podem ser anexados a um relato de problema — nada é enviado a lugar nenhum por conta própria.
# $status: which arbitrary-precision library is in use, as the program reports it (in English).
diag-arithmetic = Aritmética de zoom profundo: { $status }
# The three tests' buttons, each followed by a sentence saying what it does.
diag-run-selftest = Executar autoteste
diag-run-uitest = Executar teste de interface
diag-run-gputest = Executar verificação aritmética da GPU
diag-blurb-selftest = Verifica a matemática e a renderização na sua GPU contra resultados sabidamente corretos. Leva cerca de 15 segundos.
diag-blurb-uitest = Percorre a interface e a vista ao vivo, capturando telas em várias profundidades de zoom. Leva um ou dois minutos e abre janelas enquanto roda.
diag-blurb-gputest = Verifica a aritmética double-float do shader contra valores sabidamente corretos em todos os backends gráficos que a sua máquina oferece. Leva alguns segundos e não abre janela. Se ela informar uma falha, isso é uma descoberta genuína que gostaríamos de ver — por favor, anexe-a a um relato.
# While a test runs. $count: the checks finished so far.
diag-checks-done =
    { $count ->
        [one] { $count } verificação concluída
       *[other] { $count } verificações concluídas
    }
# A finished test's result. $test: the test's name (Self-test, UI test, GPU arithmetic check).
diag-verdict-captured = { $test } — resultado capturado
diag-verdict-passed = { $test } — aprovado
diag-verdict-problems = { $test } — problemas relatados
# Under the GPU arithmetic check's result.
diag-informational-note = Esta não tem aprovação nem reprovação: ela registra como a sua GPU calcula, e uma falha relatada é uma descoberta real, não um defeito da sua máquina. De qualquer forma, anexá-la a um relato é a coisa mais útil que você pode fazer com ela.
# Under a test's result when it reported problems.
diag-problems-note = Algumas diferenças são esperadas em hardware diferente da GPU de referência — comparações de imagens em particular. Anexar isto a um relato de problema é a coisa mais útil que você pode fazer com ele.
diag-open-results = Abrir resultados
diag-attach = Anexar a um relato de problema…
diag-attach-hover = Abre a janela de relato com este resultado incluído, para que a issue leve um teste validado pela máquina em vez de apenas uma descrição
diag-console = Imprimir diagnósticos no console
diag-console-hover = Mostrar as linhas [fd-…] no stderr a partir de agora. Desligado por padrão quando o Fractadyne é iniciado sem argumentos. Isto não muda nada no que é GRAVADO — o arquivo de log sempre recebe todas as linhas —, então deixe desligado a menos que você esteja olhando um terminal. Vale só para esta sessão; use a opção --console ou FRACTADYNE_CONSOLE=1 para capturar também a inicialização.
diag-close = Fechar
# Errors starting a test. $why: the system's error message; $dir: a folder.
diag-error-no-exe = Não foi possível encontrar o executável: { $why }
diag-error-create-dir = Não foi possível criar { $dir }: { $why }
diag-error-start = Não foi possível iniciar o teste: { $why }

## Navigate ▸ Misiurewicz explorer…

misi-title = Explorador de Misiurewicz
misi-mandelbrot-only = A navegação por pontos de Misiurewicz é exclusiva do Mandelbrot — troque o fractal para usá-la.
# While the gallery is being built. $seconds: the time so far, such as "3".
misi-solving = Resolvendo as famílias de tipos pequenos ({ $seconds }s)…
# Above the gallery. $total: all points; $curated: the hand-picked ones; $found: the ones the sweep found.
misi-summary = { $total } pontos — { $curated } selecionados à mão, { $found } encontrados por uma varredura de Newton com sementes sobre os tipos pequenos (semiplano superior; uma grade grosseira encontra a maioria das bacias, mas não comprovadamente todas).
# The name of a point the sweep found. $kind: its type, such as "(3,1)"; $point: its coordinate, such as "-0.1011+0.9563i".
misi-entry-name = { $kind } em { $point }
# Under each thumbnail. $kind: the point's type, such as "(3,1)"; $octaves: the zoom period in octaves, such as "3.17"; $twist: the twist per repeat in degrees, such as "+12.5". "oct" is short for octaves.
misi-entry-readout = { $kind } · { $octaves } oit/repetição · { $twist }°
# Before the depth field.
misi-zoom-to = Zoom até
misi-cancel = Cancelar
misi-solve-jump = Resolver e saltar
misi-solve-jump-hover = Resolver por Newton este ponto exato até a precisão que a profundidade pedida exige (com a coordenada armazenada como semente) e então saltar direto para lá. Pedidos profundos demoram mais — o cálculo é de precisão arbitrária.
misi-select-hint = Selecione um ponto, indique uma profundidade e salte — a linha do multiplicador sob cada miniatura mostra como será um mergulho ali: quantas oitavas até a vista se repetir, e quanto ela gira a cada repetição.
misi-cancelled = Cálculo cancelado.
# $text: what was typed in the depth field.
misi-bad-zoom = Não foi possível ler “{ $text }” como uma ampliação (escreva 1e50, 3.2e120, …).
misi-no-kp = Esta entrada não tem (k,p) detectado para resolver.
misi-parse-failed = A coordenada armazenada não pôde ser interpretada.
# A notice after the jump. $name: the point's name; $zoom: the magnification, such as "1e50"; $seconds: how long the solve took, such as "1.2".
misi-zoomed = Zoom feito até { $name } — { $zoom }× ({ $seconds }s)
# Added after the notice above when the iteration count is too low. $have: the fixed iteration count; $typical: what the depth typically needs.
misi-iter-note = As iterações estão fixas em { $have }; esta profundidade normalmente precisa de ~{ $typical } — aumente-as (ou ative as iterações automáticas) se a vista for renderizada chapada.
# $name: the point's name; $why: why the solve failed (a technical word).
misi-solve-missed = O cálculo profundo não caiu em { $name }: { $why }. O ponto armazenado continua exato na sua própria profundidade — tente um alvo mais raso.

## Life (Game of Life): the side panel

# Shown when the graphics card cannot run the Life view.
life-no-compute = Este adaptador gráfico não consegue rodar o Life (sem compute shaders).
# Row label: the pattern list (pattern names stay in English).
life-pattern = Padrão
life-open = Abrir…
life-open-hover = Abrir um arquivo de padrão RLE, .cells ou Life 1.05/1.06.
life-pattern-text = Texto do padrão…
life-pattern-text-hover = Colar um padrão (RLE como o LifeWiki o fornece, ou texto simples).
life-save = Salvar…
life-save-hover = Salvar o universo como está agora, em RLE.
# Row label: the rule list and the typed rule (B3/S23 notation).
life-rule = Regra
# Shown in the rule list when the rule in use is none of the library's.
life-rule-custom = personalizada
life-apply = Aplicar
# Playback buttons (a pause or play symbol is shown before them).
life-pause = Pausar
life-play = Executar
life-play-hover = Executar o universo (Enter).
life-step = Avançar
life-step-hover = Avançar uma geração (N).
# Tooltip of the button beside Step, which reads "+16", "+1,024"…: that many generations.
life-stride-hover = Avançar este número de gerações (Shift+N).
life-reset = Redefinir
life-reset-hover = Voltar ao padrão como foi aberto (geração 0).
life-speed = Velocidade
# After the speed slider's number: generations a second. Short.
life-speed-unit = ger/s
# Row label: how many generations the "+n" button advances.
life-stride = Salto
# The panel's numbers.
life-generation = Geração
life-population = População
life-tiles = Blocos
# $count: the tiles stored; $capacity: how many the GPU pool holds.
life-tiles-of = { $count } de { $capacity }
life-tiles-hover = Blocos armazenados de 64×64 células (4 KiB cada) e a capacidade do pool da GPU.
# Row label: what a left-drag in the view does (Pan or Draw).
life-mouse = Mouse
life-pan = Mover
life-pan-hover = Arraste para mover a vista.
life-draw = Desenhar
# The Draw tool's tooltip; the toolbar's shows it after "Draw (D):", so it starts lowercase.
life-draw-hint = clique numa célula para invertê-la; arraste para pintar — um traço que começa numa célula morta desenha, um que começa numa célula viva apaga. Um universo em execução pausa enquanto você desenha e continua quando você solta.
life-undo = Desfazer
life-undo-hover = Voltar ao universo de antes da última edição, preenchimento, limpeza ou mudança de regra (Ctrl+Z enquanto desenha).
life-grid = Linhas de grade
life-grid-hover = Bordas das células com zoom de 8 pixels por célula ou mais.
# A random fill: "Random" labels the density field, "seed" the seed field after it.
life-random = Aleatório
life-density-hover = A fração de células que um preenchimento deixa vivas.
life-seed = semente
life-fill-view = Preencher vista
# $cells: the largest side of the filled area, in cells (2,048).
life-fill-view-hover = Preencher aleatoriamente a área visível (até { $cells } × { $cells } células em torno do centro dela).
life-clear = Limpar
life-clear-hover = Matar todas as células (Desfazer as traz de volta).

## Life: notices in the panel, and file dialogs

# $path: the file written.
life-saved = Salvo: { $path }
# $error: why (from the system, in English).
life-save-failed = Falha ao salvar: { $error }
# The universe stopped running. $error: why (in English).
life-stopped = Parado: { $error }
# A pattern file named a rule that does not read. $error: why (in English); $rule: the rule kept, in B3/S23 notation.
life-rule-not-understood = A regra do padrão não foi entendida ({ $error }); mantida { $rule }
# $path: the file; $error: why (in English).
life-open-failed = Não foi possível abrir { $path }: { $error }
# File-dialog filter names.
life-filter-patterns = Padrões do Life
life-filter-rle = Padrão com codificação run-length

## Life: the toolbar (its buttons are icons; these are their tooltips)

life-toolbar-pause-hover = Pausar o universo (Enter)
life-toolbar-play-hover = Executar o universo (Enter)
# $stride: the generations a Shift+click steps.
life-toolbar-step-hover = Avançar uma geração (N); Shift+clique ou Shift+N avança { $stride }
life-toolbar-reset-hover = Voltar ao padrão como foi aberto (geração 0)
# $hint: the Draw tool's tooltip (life-draw-hint).
life-toolbar-draw-hover = Desenhar (D): { $hint }

## Life: the Pattern text window

life-text-title = Texto do padrão
life-text-intro = Cole um padrão: RLE (como o LifeWiki e o Golly o fornecem), texto simples .cells, ou Life 1.05 / 1.06.
life-text-load = Carregar
life-text-current = Atual em RLE
life-text-current-hover = Preencher a caixa com o universo como foi carregado por último.

## L-systems: the side panel

# Row label: the system list (system names stay in English).
lsys-system = Sistema
lsys-open = Abrir…
lsys-open-hover = Abrir um arquivo .l do Fractint ou um sistema .lsys.
lsys-edit = Editar…
lsys-edit-hover = O texto do sistema: axioma, produções, ângulo — edite e aplique.
lsys-save = Salvar…
lsys-save-hover = Salvar o sistema como um arquivo de texto .lsys.
# Tooltip of the "SVG…" button.
lsys-svg-hover = Exportar a vista como um desenho SVG: suas linhas e formas como vetores, nas suas cores.
lsys-follow = Ordem segue o zoom
lsys-follow-hover = A ordem sobe conforme você aproxima, então os detalhes nunca acabam.
lsys-follow-fixed-hover = Este sistema não cresce por um fator a cada ordem, então é desenhado numa ordem fixa.
lsys-order = Ordem
lsys-angle = Ângulo
lsys-sweep-hover = Varrer o ângulo, ida e volta: a curva se transformando enquanto gira
# A small button that puts back the system's own angle after the slider moved it.
lsys-system-angle = Ângulo do sistema
lsys-seed = Semente
lsys-seed-hover = O que as escolhas aleatórias seguem: a mesma semente, a mesma planta.
lsys-seed-new = Nova
lsys-seed-new-hover = Outra planta: uma nova semente.
lsys-line-width = Espessura da linha
# Row label of a percentage slider: how much of the curve is drawn.
lsys-draw-on = Traçado
lsys-draw-on-hover = Quanto da curva é desenhado, na ordem em que a tartaruga a desenha
lsys-draw-on-play-hover = A curva se desenhando sozinha
lsys-colour-by = Colorir por
# The colourings, in the "Colour by" list.
lsys-colour-position = posição ao longo da curva
lsys-colour-depth = profundidade do ramo
lsys-colour-heading = direção
# C, < and > are the system's commands that set the colour index: keep them.
lsys-colour-index = índice de cor (C, <, >)
lsys-colour-plain = uma só cor
# The panel's numbers.
lsys-growth = Crescimento
lsys-growth-built = construído como palavra, na sua ordem
lsys-growth-built-hover = Paramétrico ou sensível ao contexto: a reescrita de um módulo depende dos seus números ou dos seus vizinhos, então a palavra é construída uma geração por vez (até dois milhões de módulos) e desenhada em precisão dupla — a ordem não segue o zoom.
# How much the picture grows from one order to the next. $factor: the factor (1.4142).
lsys-growth-factor = ×{ $factor } por ordem
lsys-growth-factor-period-2 = ×{ $factor } por ordem (as ordens avançam de 2 em 2)
lsys-growth-none = nenhum (ordem fixa)
lsys-drawn = Desenhado
# $drawn: what was drawn (lsys-drawn-segments, lsys-drawn-shapes or lsys-drawn-segments-shapes); $order: the order.
lsys-drawn-at-order = { $drawn } na ordem { $order }
# The same, when the walk stopped at its segment budget.
lsys-drawn-at-order-budget = { $drawn } na ordem { $order } (orçamento)
# $segments: the line segments drawn.
lsys-drawn-segments =
    { $segments ->
        [one] { $segments } segmento
       *[other] { $segments } segmentos
    }
# $shapes: the filled shapes drawn.
lsys-drawn-shapes =
    { $shapes ->
        [one] { $shapes } forma preenchida
       *[other] { $shapes } formas preenchidas
    }
# Both counts. $segments: line segments; $shapes: filled shapes.
lsys-drawn-segments-shapes =
    { $segments ->
        [one] { $segments } segmento
       *[other] { $segments } segmentos
    }, { $shapes ->
        [one] { $shapes } forma preenchida
       *[other] { $shapes } formas preenchidas
    }
# $ms: the walk's time in milliseconds (12.3).
lsys-drawn-hover = percorrido em { $ms } ms; as subárvores fora da vista são puladas, e as menores que um pixel são desenhadas como um só segmento
# A word too long to build at the order asked for. $wanted: the order asked for; $built: the order drawn.
lsys-drawn-short = A palavra da ordem { $wanted } passa de dois milhões de módulos: desenhada na ordem { $built }.
# $order: the deepest order the app draws.
lsys-deepest-order = Na ordem mais profunda desenhada ({ $order }): mais zoom não acrescenta detalhes.

## L-systems: notices in the panel and the editor, and file dialogs

# $path: the file; $error: why (in English).
lsys-open-failed = Não foi possível abrir { $path }: { $error }
# Why a file was not opened (shown after "Could not open …:").
lsys-file-too-large = maior que 4 MiB
# Why a Fractint .l file was not opened: it holds no entries. `Name { … }` is the file's syntax: keep it.
lsys-no-entries = nenhuma entrada (um arquivo .l contém entradas `Name {"{"} … {"}"}`)
# $path: the file written.
lsys-saved = Salvo: { $path }
# $error: why (from the system, in English).
lsys-save-failed = Falha ao salvar: { $error }
# After File ▸ Export SVG. $path: the file written.
lsys-svg-exported = Exportado: { $path }
# $error: why (from the system, in English).
lsys-svg-failed = Falha na exportação: { $error }
# File-dialog filter names: opening (any system file) and saving (one system).
lsys-filter-systems = Sistemas L
lsys-filter-system = Sistema L

## L-systems: the toolbar

lsys-toolbar-lower-hover = Diminuir a ordem (fixa-a)
lsys-toolbar-raise-hover = Aumentar a ordem (fixa-a)
# Between the − and + buttons, in a fixed-width font. $order: the order, padded to 3 characters so the label keeps its width.
lsys-toolbar-order = ordem { $order }
# A toggle: the order follows the zoom. Short.
lsys-toolbar-auto = auto
lsys-toolbar-auto-hover = A ordem segue o zoom
lsys-toolbar-edit-hover = Editar o texto do sistema

## L-systems: the editor window, and a .l file's entries

lsys-editor-title = Sistema L
# The editor's help. The quoted examples ('X = word', 'seed n', 'define R 1.456'…), the commands and "The Algorithmic Beauty of Plants" (a book's title) are the system's syntax and names: keep them as they are.
lsys-editor-help = Uma chave por linha: angle (graus, ou /n para uma divisão do círculo), axiom, as produções 'X = word' e, opcionalmente, heading, draw / move / variables, colour, order. Um símbolo com alternativas escolhidas ao acaso tem uma linha para cada uma, com o seu peso: 'X (0.3) = word'; 'seed n' escolhe a planta. Numa palavra, {"{"} {"}"} preenche o caminho da tartaruga e . acrescenta um vértice. Produções paramétricas e sensíveis ao contexto (The Algorithmic Beauty of Plants): 'A(s) : s > 1 = F(s)[+A(s/2)]', 'b < a > c = b', com 'define R 1.456' e 'ignore +-F'; F(l) avança l, +(a) gira a graus. Uma entrada .l do Fractint pode ser colada como está.
lsys-editor-apply = Aplicar
lsys-editor-revert = Reverter
lsys-editor-revert-hover = O texto do sistema mostrado.
# The window listing a .l file's entries to pick from. $file: the file's name.
lsys-entries-title = { $file } — entradas
# Tooltip of an entry that does not parse (its error is shown beside it, in English).
lsys-entry-unreadable-hover = Esta entrada não é lida.

## Export: opening a view or location (File ▸ Open view or location…)

# File-dialog filter names.
export-filter-view = Vista ou local do Fractadyne
export-filter-image = Imagem (PNG / EXR)
export-filter-location = Local (.fdn / .kfr)
# $zoom: the magnification (1.2e148), followed by the magnification sign ×.
export-loaded-location = Local carregado @ { $zoom }×
# $zoom: the magnification; $note: what was noteworthy in the file (in English).
export-loaded-location-note = Carregado @ { $zoom }× — { $note }
# $path: the file.
export-not-location = { $path } não é um local do Fractadyne.
export-file-too-large = Arquivo grande demais (não é um local .fdn?).
# $path: the file; $error: why (from the system, in English).
export-read-failed = Não foi possível ler { $path }: { $error }
# A Kalles Fraktaler .kfr file did not import. $error: why (in English).
export-import-failed = Falha na importação: { $error }
# $path: an image whose embedded text is not a view.
export-not-view = { $path } tem metadados do Fractadyne, mas não uma vista.
# $path: the image the view was read from.
export-loaded-view = Vista carregada de { $path }
# $path: the image; $note: what was noteworthy in it (in English).
export-loaded-view-note = Vista carregada de { $path } — { $note }
export-no-metadata = Esse arquivo não tem metadados de vista do Fractadyne incorporados.

## Export: notices after an export or a snapshot
## "File → Export image…" in them names the menu items (menu-file, menu-file-export).

# The export window's status while the image renders.
export-rendering = Renderizando…
export-canceled = Exportação cancelada.
# $error: why (in English).
export-failed = Falha na exportação: { $error }
# $size: the image's size in pixels (1920×1080); $path: the file written.
export-saved = Salvo: { $size } → { $path }
# An L-system export whose walk stopped at its budget. $size: pixels; $segments: the budget (4,000,000); $path: the file.
export-saved-incomplete = Salvo: { $size } (incompleto: passou do orçamento de { $segments } segmentos — diminua a ordem) → { $path }
# A two-panel export written as two files. $path: the first of them.
export-saved-two = 2 arquivos salvos → { $path }
# $size: pixels; $path: the file written.
export-saved-corrected = Salvo: { $size } (glitches corrigidos) → { $path }
# $path: the first of the two files.
export-saved-two-corrected = 2 arquivos salvos (glitches corrigidos) → { $path }
# Added after a finished export's notice. $duration: how long it took (8.3s, 1m 04.0s).
export-elapsed = (em { $duration })
export-preparing-deep = Preparando a exportação profunda — construindo a referência (isso pode demorar)…
# $size: pixels (5120×4035); $ss: the supersampling factor; $iterations: the iteration count.
export-large = Exportação grande ({ $size }, superamostragem { $ss }×, { $iterations } iterações) — renderizando em segundo plano; a correção de glitches é pulada em exportações deste tamanho…
export-life-unavailable = A exportação de imagem de uma vista do Life ainda não está disponível — use uma captura de tela, ou Arquivo ▸ Salvar padrão do Life….
export-already-running = Uma exportação já está em andamento — Arquivo → Exportar imagem… mostra o progresso dela.
# $size: the image's size in pixels.
export-snapshot-background = Captura: renderizando { $size } em segundo plano — Arquivo → Exportar imagem… mostra o progresso e permite cancelá-la.
export-no-gpu = GPU indisponível
# A screen-capture snapshot. $size: pixels; $path: the file written.
export-snapshot-saved = Captura de tela salva: { $size } → { $path }
# $error: why (in English).
export-snapshot-failed = Falha na captura: { $error }

## The colour picker (the popup of a colour swatch)

# The third row's label: the colour as hexadecimal bytes. Short (about 6 characters).
picker-row-hex = Hex
picker-hex-hover = Clique para digitar um valor hexadecimal
picker-cancel = Cancelar
picker-cancel-hover = Restaurar a cor com que este seletor foi aberto
picker-ok = OK
picker-ok-hover = Manter esta cor
# Starts the screen eyedropper (an icon is shown before it).
picker-pick = Escolher
picker-pick-hover = Pegar uma cor de qualquer lugar da tela
# Tooltip of the disabled Pick button (in the picker and the gradient editor) on Linux and macOS.
picker-eyedropper-unsupported = Escolher uma cor de outro lugar da tela requer uma API de captura de tela da plataforma. Só a build para Windows tem uma; no Linux isso exigiria uma dependência do X11, e o Wayland requer um portal com o seu próprio pedido de permissão.

## The fractal families: what each one is (the info panel's "About" section and the pickers' hover text).
## The formulas themselves (z -> z^2 + c) are written in code and stay as they are.

fractal-mandelbrot-about = O fractal de tempo de escape canônico: o conjunto dos c para os quais a órbita de 0 permanece limitada. Sua fronteira é infinitamente intrincada.
fractal-multibrot3-about = Um conjunto Multibrot - a construção do Mandelbrot numa potência maior. A potência d dá simetria rotacional de ordem (d-1).
fractal-multibrot4-about = Multibrot na potência 4: simetria de ordem três, bulbos largos.
fractal-multibrot5-about = Multibrot na potência 5: simetria de ordem quatro.
fractal-tricorn-about = O Tricorn (Mandelbar): conjuga z a cada passo. Esta aplicação anti-holomorfa produz uma forma de três pontas.
fractal-burning-ship-about = Os valores absolutos das partes de z são tomados antes de elevar ao quadrado; o resultado lembra um navio em chamas.
fractal-celtic-about = Um parente do Burning Ship que toma o valor absoluto só da parte real de z^2, produzindo motivos de nó celta / coração.
fractal-buffalo-about = Uma variante com abs que toma os valores absolutos das duas componentes de z^2.
fractal-phoenix-about = O Phoenix também usa o iterado anterior, o que dá filamentos parecidos com chamas. Experimente a forma Julia dele com o modo Julia.
fractal-newton-about = A iteração de Newton para encontrar as raízes de z^3 = 1, colorida pela rapidez com que cada ponto converge. Um fractal de convergência (não de escape).
fractal-multibrot6-about = Multibrot na potência 6: simetria de ordem cinco.
fractal-multibrot7-about = Multibrot na potência 7: simetria de ordem seis, com os bulbos afinando em direção a um círculo.
fractal-multibrot8-about = Multibrot na potência 8: simetria de ordem sete.
fractal-burning-ship3-about = O Burning Ship na potência 3: os valores absolutos são tomados e depois elevados ao cubo.
fractal-burning-ship4-about = O Burning Ship na potência 4.
fractal-burning-ship5-about = O Burning Ship na potência 5.
fractal-tricorn3-about = O Tricorn (Mandelbar) na potência 3: simetria de ordem quatro.
fractal-tricorn4-about = O Tricorn na potência 4: simetria de ordem cinco.
fractal-tricorn5-about = O Tricorn na potência 5: simetria de ordem seis.
fractal-celtic3-about = A dobra Celtic na potência 3: o valor absoluto da parte real de z^3.
fractal-celtic4-about = A dobra Celtic na potência 4.
fractal-celtic5-about = A dobra Celtic na potência 5.
fractal-buffalo3-about = O Buffalo na potência 3: os valores absolutos das duas partes de z^3.
fractal-buffalo4-about = O Buffalo na potência 4.
fractal-buffalo5-about = O Buffalo na potência 5.
fractal-custom-about = Uma fórmula escrita em expressões no estilo do Fractint (Fractal ▸ Fórmula personalizada…). A maioria faz zoom profundo por perturbação, como as famílias integradas; a janela da fórmula diz se a sua faz.
fractal-life-about = O Jogo da Vida de Conway e seus parentes num plano ilimitado: regras Generations, regras não totalísticas (Hensel), arquivos de padrões. Execute, avance, desenhe.
fractal-lsystem-about = Sistemas de Lindenmayer: curvas, curvas que preenchem o espaço, ilhas e plantas desenhadas por uma tartaruga a partir de uma gramática de reescrita. A ordem segue o zoom, então os detalhes nunca acabam. Os arquivos .l do Fractint abrem diretamente.

# Shown where the other families show their formula (z -> z^2 + c).
fractal-custom-formula = z -> a fórmula que você escreve
# Shown where the other families show their formula. "B3/S23" is the rule's notation: keep it.
fractal-life-formula = B3/S23: nasce com 3 vizinhos vivos, sobrevive com 2 ou 3
# Shown where the other families show their formula. "F = F+F--F+F" is the grammar's notation: keep it.
fractal-lsystem-formula = F = F+F--F+F: cada símbolo é reescrito e depois desenhado por uma tartaruga

# The last paragraph of the picker's hover text for a family that renders only on the direct path.
fractal-no-deep-zoom = Só renderização direta - esta família não tem zoom profundo.

## Names of settings, shown in the Settings menu and the welcome window.

theme-dark = Escuro
theme-light = Claro
# The update track that follows only stable releases.
update-track-stable = Estável
# The update track that also offers pre-releases (betas).
update-track-beta = Beta (pré-lançamentos)

## The status bar (its hover texts; the readouts follow in their own section)

status-period-hover = Período do minibrot para o qual você saltou (pressione M, ou Ir para local ▸ Ir para uma estrutura). Ele fica enquanto a vista estiver sobre a estrutura e some quando você se afasta.

## The playback transport: the player over the view while a tour plays.

# $close: the close button's icon, as drawn on that button.
play-finished-hover = Terminado — arraste para voltar, ou { $close } para fechar
# $kind: "benchmark" or "script" (the variants hold the wording); $name: the tour's name.
play-playing-hover =
    { $kind ->
        [benchmark] Reproduzindo o benchmark “{ $name }”
       *[other] Reproduzindo o script “{ $name }”
    }
play-restart = Recomeçar do início
play-back = Voltar 10 segundos
play-again = Reproduzir de novo desde o início
play-resume = Retomar
play-pause = Pausar
play-stop = Parar e voltar ao início
play-forward = Avançar 10 segundos
play-speed-hover = Velocidade de reprodução (clique para alternar entre 0.5× / 1× / 2× / 4×)
play-render = Renderizar este script como uma sequência de quadros…
play-render-none = O benchmark integrado não tem arquivo de script para renderizar
play-open = Reproduzir outro script…
play-loop = Repetir o tour quando ele chegar ao fim
play-close = Fechar o reprodutor e restaurar as suas próprias configurações de vista
# A notice beside the scrub bar while the renderer holds the tour's clock. Keep it short.
play-waiting = aguardando detalhes
# "[playback] pace" is a setting in the script file: keep it as written.
play-waiting-hover = O relógio do tour está pausado enquanto o renderizador resolve esta vista (construção da referência / orçamento de iterações subindo). A reprodução continua sozinha; veja [playback] pace no script.
play-scrub = Arraste para percorrer o tour
# $behind: how far the script clock is behind real time, as minutes:seconds (1:05).
play-scrub-behind =
    Arraste para percorrer o tour
    O relógio do script está { $behind } atrás do tempo real (o regulador de ritmo desacelerou o tour para que o renderizador acompanhe). A marca superior indica onde estaria a reprodução em tempo real.

## The autopilot's notices (toasts). Where a notice has two lines, the first is the verdict and the second what to do about it.

autopilot-on-detail = Piloto automático ligado — mergulhando em direção aos detalhes (qualquer comando o interrompe)
autopilot-off = Piloto automático desligado
autopilot-misi-needs-mandelbrot =
    Piloto automático: um alvo de Misiurewicz precisa do conjunto de Mandelbrot
    Mergulhando em direção aos detalhes em vez disso
autopilot-on-misi =
    Piloto automático ligado — procurando o ponto de Misiurewicz mais próximo…
    Qualquer comando o interrompe
# $seconds: how long the search has run, in whole seconds.
autopilot-misi-finding =
    Piloto automático: procurando o ponto de Misiurewicz mais próximo…
    { $seconds } s · qualquer comando o interrompe
autopilot-misi-not-found =
    Piloto automático: nenhum ponto de Misiurewicz encontrado perto desta vista
    O localizador pode não alcançar esta profundidade — tente o alvo Detalhe
# ($preperiod,$period) names the point, as mathematicians write it: keep the notation. $widths: a power of two (2^$widths screen widths).
autopilot-misi-off-screen =
    Piloto automático: o ponto de Misiurewicz mais próximo ({ $preperiod },{ $period }) está fora da tela
    a ~2^{ $widths } larguras de tela de distância — aproxime-se do centro de uma espiral
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-not-solved =
    Piloto automático: o ponto de Misiurewicz ({ $preperiod },{ $period }) perto desta vista não foi resolvido
    Tente o alvo Detalhe
autopilot-misi-refused = Piloto automático: cálculo de Misiurewicz recusado (par inválido)
autopilot-misi-stopped = Piloto automático: o solucionador de Misiurewicz parou inesperadamente
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-diving =
    Piloto automático: mergulhando em Misiurewicz ({ $preperiod },{ $period })
    Qualquer comando o interrompe
# ($preperiod,$period) names the point: keep the notation. $octaves: how often the structure repeats (2.37); $twist: its turn in degrees, signed (+12.5).
autopilot-misi-diving-repeat =
    Piloto automático: mergulhando em Misiurewicz ({ $preperiod },{ $period })
    A estrutura se repete a cada { $octaves } oitavas, torção de { $twist }° · qualquer comando o interrompe
# $limit: the magnification, as written (1e300).
autopilot-dive-limit = Piloto automático: limite de mergulho atingido (~{ $limit }×)
# $have: the fixed iteration count; $typical: what this depth typically needs.
autopilot-dead-end-short =
    Piloto automático parado: não há mais nada para mirar — as iterações estão fixas em { $have } e esta profundidade normalmente precisa de ~{ $typical }
    Ative as iterações automáticas ou aumente a contagem
# $limit: the fixed iteration count.
autopilot-dead-end-cap =
    Piloto automático parado: não há mais nada para mirar — tudo na vista vai até o limite fixo de iterações de { $limit }
    Ative as iterações automáticas ou aumente a contagem
autopilot-dead-end = Piloto automático: nenhum detalhe à frente (parado)

## Tours (camera scripts)

# The name of the file type in the file dialog's filter list.
script-file-filter = Script do Fractadyne (TOML)
# $name: the script's name.
script-finished = Script concluído — “{ $name }”
# The title of the notice that follows (the file's path and the error, in English).
script-load-failed = Não foi possível carregar o script

## Installing a language file

# The name of the file type in the file dialog's filter list.
language-file-filter = Idioma do Fractadyne

## The notice when the graphics driver's shader cache stops keeping shaders. The text is shown in a fixed-width font with its line breaks as written: break the lines where they read well.

driver-cache-title = O driver gráfico não está guardando os shaders
# $count: how many shaders compiled again; $pipeline: the slowest one's name (a technical id: keep it); $seconds: its time in seconds (7.5).
driver-cache-body =
    O seu driver gráfico compilou de novo { $count } shaders do Fractadyne nesta
    sessão, embora já tivesse compilado cada um deles antes (o mais lento:
    { $pipeline }, { $seconds } s). O cache de shaders dele não está guardando
    os shaders compilados, na maioria das vezes porque o cache está cheio.
    Até que isso mude, exportações, vistas profundas e a inicialização do
    Fractadyne podem gastar segundos compilando.
# $folder: the cache folder's path; $size: its size in MB (1024). "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia-folder =
    Para corrigir: feche o Fractadyne e depois apague o conteúdo de
      { $folder }   ({ $size } MB)
    O driver reconstrói o cache. Para que ele não volte a encher, aumente
    Painel de controle da NVIDIA > Gerenciar as configurações em 3D >
    Tamanho do cache do sombreador.
# "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia =
    Para corrigir: aumente Painel de controle da NVIDIA > Gerenciar as
    configurações em 3D > Tamanho do cache do sombreador, ou limpe o
    cache de shaders do driver.
# "AMD Software > Graphics > Advanced > Reset Shader Cache" names AMD's own settings: use the names AMD's software shows in this language.
driver-cache-fix-amd = Para corrigir: AMD Software > Gráficos > Avançado > Redefinir cache do sombreador.
driver-cache-fix-other = Para corrigir: limpe o cache de shaders do driver gráfico.

## Why a language file could not be installed: each ends the sentence "{ $file } is not a usable language file: …".

language-install-not-utf8 = não é texto UTF-8
language-install-too-large = é grande demais
# "fractadyne-<language>.ftl" is the file name to use: keep it as written.
language-install-no-tag = ele não diz qual é o idioma (dê a ele o nome fractadyne-<language>.ftl)
language-install-english = o inglês já vem integrado
language-install-empty = ele não traduz nenhuma mensagem desta versão
language-install-no-folder = não há pasta de configurações

## The notice after a session that did not end cleanly, when its iteration setting was set aside.

# $base: the iteration count automatic iterations start from.
quarantine-setting-auto = iterações automáticas a partir de uma base de { $base }
# $count: the fixed iteration count.
quarantine-setting-fixed =
    { $count ->
        [one] { $count } iteração
       *[other] { $count } iterações
    }
# $end: "device" when the session lost its graphics device, "unclean" otherwise (the variants hold the wording). $depth: the magnification, as written (1.2e148). $setting and $restored: an iteration setting (quarantine-setting-auto or -fixed).
quarantine-notice =
    { $end ->
        [device] A sessão anterior perdeu o dispositivo gráfico em { $depth }× quando estava configurada com { $setting }. Esta sessão começou com { $restored } em vez disso, para não voltar direto à mesma carga.
       *[other] A sessão anterior não terminou corretamente em { $depth }× quando estava configurada com { $setting }. Esta sessão começou com { $restored } em vez disso, para não voltar direto à mesma carga.
    }

## The second graphics card's status line, under its setting in the control panel.

gpu2-state-off = Desligada: a placa de vídeo desta janela renderiza tudo.
gpu2-state-starting = Iniciando…
# $name: the card's name (and its graphics API).
gpu2-state-running = Em uso: { $name }
# $why: the error, as the graphics driver reports it.
gpu2-state-failed = Não foi possível iniciar: { $why }
# $name: the card's name.
gpu2-state-lost = Parada: { $name } foi perdida. Daqui em diante, uma só placa; escolha-a de novo para tentar outra vez.
# $state: one of the lines above. "--worker-gpu" is a command-line option: keep it.
gpu2-state-pinned = { $state } (definido por --worker-gpu)

## The custom-formula dialog: how deep the formula renders (one line under the formula).

formula-depth-deep = Zoom profundo por perturbação, com faixa estendida além de 1e28x como nos fractais integrados (todo passo é iterado: ainda sem aproximação por séries nem BLA para fórmulas personalizadas).
# $why: what the formula has that keeps it off the deep-zoom path (one of the formula-depth-why messages below).
formula-depth-direct = Renderização direta ({ $why } ainda não tem forma de zoom profundo): nítida até que um passo de precisão simples de c abranja um pixel — por volta de 1e4x a 1e5x, e menos fundo longe da origem.
formula-depth-why-init = uma seção de inicialização
formula-depth-why-bailout = um teste de escape próprio
formula-depth-why-variable = uma variável mantida de um passo para o outro
formula-depth-why-previous = o iterado anterior
formula-depth-why-comparison = uma comparação ou um bloco if
# floor, ceil, trunc and round are functions of the formula language: keep them.
formula-depth-why-rounding = arredondamento (floor, ceil, trunc, round)
# "maxit" is a word of the formula language: keep it.
formula-depth-why-maxit = maxit
formula-depth-why-power = uma potência cujo expoente varia
formula-depth-why-perturbed = um programa já perturbado

## Image and frame sizes: the presets described in words (the others are standard names: HD 720p, WUXGA, DCI 4K).

# $size: the pixel size (2048×2048).
size-preset-square = Quadrado — { $size }
# $ratio: the aspect ratio (9:16); $size: the pixel size.
size-preset-portrait = Retrato { $ratio } — { $size }
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
fractal-family-custom = Personalizado
# Conway's Game of Life.
fractal-family-life = Life
fractal-family-lsystem = Sistema L
# A family at a power: "Multibrot 3", "Burning Ship 4". $family: a name above; $power: the power.
fractal-with-power = { $family } { $power }

## Names of choices: colouring methods, orbit traps, palette animation, snapshots, angle units.

color-method-smooth = Iteração suave
color-method-stripe = Média de listras
color-method-triangle = Desigualdade triangular
color-method-trap = Armadilha de órbita
color-method-distance = Estimativa de distância
color-method-decomposition = Decomposição
# The shape an orbit trap catches the orbit with.
trap-point = Ponto
trap-cross = Cruz
trap-circle = Círculo
# Palette animation: none.
palette-anim-off = Desligado
palette-anim-forward = Para frente
palette-anim-reverse = Inverso
palette-anim-pingpong = Pingue-pongue
# Palette animation: a new random gradient every so often.
palette-anim-random = Gradientes aleatórios
# What the Snapshot button does.
snapshot-mode-ask = Perguntar sempre
snapshot-mode-screen = Captura de tela
snapshot-mode-render = Renderização completa
# The unit of an angle typed into Go to location's polar fields.
angle-degrees = graus
angle-radians = radianos
angle-turns = voltas

## Report an issue: the choices. (The report itself is sent in English.)

issue-kind-crash = Falha / travamento do aplicativo
issue-kind-rendering = Renderização incorreta
issue-kind-performance = Problema de desempenho
issue-kind-ui = Problema de interface / usabilidade
issue-kind-feature = Pedido de recurso
issue-kind-other = Outro
severity-low = Baixa
severity-medium = Média
severity-high = Alta
severity-blocking = Bloqueante
repro-always = Sempre
repro-sometimes = Às vezes
repro-once = Aconteceu uma vez
repro-cannot = Não consigo reproduzir

## The benchmark's depths and the diagnostics' tests.

# $zoom: the dive's final magnification (1e32).
bench-depth-standard = Padrão ({ $zoom }×)
# $zoom: the dive's final magnification (1e48).
bench-depth-ultra = Ultraprofundo ({ $zoom }×)
diag-test-selftest = Autoteste
diag-test-uitest = Teste de interface
diag-test-gputest = Verificação aritmética da GPU

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
status-generation = ger
# A Game of Life view's population. Short.
status-population = pop
# An L-system's order (how many times its rules were applied).
status-order = ordem
# How many segments and shapes an L-system drew.
status-drawn = traçado
# $value: how many cells one pixel covers (4, 9.99e300). "px" is the unit symbol: keep it.
life-scale-cells = 1 px = { $value } células
# $value: how many pixels one cell covers (8). "px" is the unit symbol: keep it.
life-scale-pixels = { $value } px / célula

## The status bar's limit diagnostic: a short label (after a warning sign), and what it means when hovered.

# The view is as deep as this location can be rendered. Short.
status-limit-depth = limite de prof.
# Every pixel ran out of iterations. Short.
status-limit-exhausted = iter esgotadas
# Pixels are capped at the iteration budget. Short.
status-limit-capped = iter no limite
# $pixels: how many pixels wide one step of the coordinates is; $sharp_to: the magnification the view is sharp to (written out, 1,234,567).
status-limit-direct-detail = Esta vista é renderizada diretamente na GPU em precisão simples, e aqui um passo das coordenadas tem { $pixels } pixels de largura: pixels vizinhos calculam o mesmo ponto, então a imagem se quebra em blocos. Neste local, ela é nítida até cerca de { $sharp_to }×. Esta fórmula ainda não tem caminho de zoom profundo (perturbação), então mais zoom não mostra mais detalhes.
# $samples: the GPU buffer's capacity in orbit samples; $iterations: the iterations pixels are limited to.
status-limit-depth-detail = A órbita de referência chegou ao limite do buffer da GPU ({ $samples } amostras) sem escapar, então os pixels aqui ficam limitados a ~{ $iterations } iterações e detalhes mais finos não podem ser calculados nesta vista — ao vivo ou exportada. Este é o limite de profundidade atual para esta família de locais.
# $budget: the largest iteration budget this view can be given.
status-limit-exhausted-detail = Todos os pixels continuaram esgotando as iterações no maior orçamento que esta vista pode receber ({ $budget }). Ou a vista está dentro do conjunto aqui (normal em profundidade), ou ela precisa de mais iterações do que o aplicativo consegue fornecer no momento — a imagem é a mesma em ambos os casos, então o orçamento voltou a um valor barato.
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations" is the setting's name in the control panel's Quality section.
status-limit-capped-max = { $percent }% dos pixels esgotaram as iterações no orçamento de { $budget }, e os que escapam estão terminando bem no teto — a vista está sub-resolvida. Iterações já está no máximo, então esta vista precisa de mais do que o aplicativo consegue dar no momento.
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations", "Controls" and "Quality" are the names of a setting, the control panel and its section.
status-limit-capped-raise = { $percent }% dos pixels esgotaram as iterações no orçamento de { $budget }, e os que escapam estão terminando bem no teto — a vista está sub-resolvida. Aumente Iterações (Controles ▸ Qualidade) para resolvê-la.
