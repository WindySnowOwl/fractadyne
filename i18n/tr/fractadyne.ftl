### Fractadyne user interface: Turkish (Türkçe).
### A draft: machine translation (Claude, by Anthropic) of the English file, 2026-10-23, not yet
### reviewed by a native speaker. Corrections are welcome as pull requests (see i18n/README.md).

## Language data: read by Fractadyne, not shown as messages.

# This language's own name, shown in the language menu.
language-name = Türkçe
# The same name in English, shown beside it in the menu.
language-english-name = Turkish
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
language-more = Diğer diller…
# The Languages window's title.
language-window-title = Diller
# The Languages window's introduction.
language-window-intro = Fractadyne arayüzü başka bir dilde. Her dil tek bir dosyadır: bir sürümle birlikte gelir ya da buradan yükleyebilirsiniz. Koordinatlar, dosyalar ve komut satırı her dilde İngilizce kalır.
# Column headers of the installed-language list.
language-col-language = Dil
language-col-translated = Çevrilen
language-col-from = Kaynak
# Where a language comes from.
language-source-builtin = yerleşik
language-source-package = bu sürüm
language-source-user = yüklenen
language-source-dev = çalışma klasörü
# $percent: how much of the interface the file translates, 0–100.
language-coverage = %{ $percent }
# $count: messages in the file that failed the checks and show in English instead.
language-dropped =
    { $count ->
        [one] bir iletinin denetlenmesi gerekiyor
       *[other] { $count } iletinin denetlenmesi gerekiyor
    }
# Buttons of the Languages window.
language-install = Dil dosyası yükle…
language-install-hover = Bir Fractadyne dil dosyası (.ftl) seçin. Dosya denetlenir, ardından ayarlarınızın diller klasörüne kopyalanır; güncellemelerden sonra da orada kalır.
language-open-folder = Diller klasörünü aç
language-reload = Yeniden yükle
language-reload-hover = Dil dosyalarını yeniden okur (birini düzenledikten ya da ekledikten sonra).
language-use = Kullan
# Notices after installing a file. $name: the language's own name; $file: the file's name.
language-installed = { $name } yüklendi. Kaynak dosya: { $file }.
# $file: the file's name; $why: what was wrong with it.
language-install-failed = { $file } kullanılabilir bir dil dosyası değil: { $why }
# Shown when a language file cannot be loaded and English is used instead. $name: the language.
language-load-failed = { $name } yüklenemedi, bu yüzden Fractadyne İngilizce gösteriliyor: { $why }
# A language whose writing system needs a font this computer does not have (Japanese or Chinese without a system font). $name: the language, in its own name.
language-font-missing = Bu bilgisayarda { $name } dilini çizebilecek bir yazı tipi bulunamadı, bu yüzden bu dil gösterilemiyor. Bir { $name } yazı tipi yükleyin (örneğin sistemin kendi dil paketini) ve dili yeniden seçin.
# Shown after a draft translation's name in the language menu and the Languages window. Short.
language-draft = taslak
language-draft-hover = Taslak: bu çeviri henüz anadili bu dil olan biri tarafından gözden geçirilmedi. Düzeltmeler, dilin dosyasına bir pull request olarak memnuniyetle karşılanır.
# The heading over the decimal-mark choice, in the language menu and the Languages window.
language-decimal-mark = Ondalık işareti
language-decimal-mark-hover = Sayıların nasıl yazılacağı: süreler, boyutlar ve sayımlar. Koordinatlar ve büyütmeler, dosyaların ve diğer programların okuduğu biçimde, her zaman nokta kullanır.
# $sample: a number written with this language's own punctuation (1,234.5).
language-decimal-language = Dilin yazdığı gibi ({ $sample })
# $sample: a number written with a decimal point (1,234.5).
language-decimal-point = Nokta ({ $sample })
# $sample: a number written with a decimal comma (1.234,5).
language-decimal-comma = Virgül ({ $sample })

## Asked once when the system's language changes

# The window's title.
language-changed-title = Dil
# $new: the system's new language, in its own name.
language-changed-question = Sistem diliniz { $new } olarak değişti. Fractadyne de bu dili kullansın mı?
# Buttons. $name: a language's own name.
language-changed-use = { $name } dilini kullan
language-changed-keep = { $name } dilinde kal

## The welcome window (first run)

welcome-title = Fractadyne’a hoş geldiniz
welcome-intro = Mandelbrot kümesi ve akrabaları için bir derin yakınlaştırma gezgini. Başlamanız için birkaç kontrol:
# The language row, at the top of the window.
welcome-language = Dil
# Controls: the key or gesture on the left, what it does on the right.
welcome-key-drag = sürükle
welcome-key-drag-does = görünümü kaydırır
welcome-key-scroll = tekerlek
welcome-key-scroll-does = imlece doğru yakınlaştırır / uzaklaştırır
welcome-key-space = Space tuşunu basılı tut
welcome-key-space-does = akıcı, sürekli yakınlaştırma (Shift+Space ile uzaklaştırma)
# Shown after the click-zoom toolbar icon.
welcome-key-click = sonra tıkla
welcome-key-click-does = tıklanan noktaya dalar (araç çubuğu)
welcome-key-m-does = en yakın minibrota, kendi ölçeğinde atlar
welcome-landmarks = Başlamak için ünlü bir konuma atlayın:
welcome-theme = Tema
welcome-watermark = “Fd” işaretini göster
welcome-watermark-hover = Görünümün sağ alt köşesindeki küçük marka işareti. Kapalıyken hem canlı görünümden hem de dışa aktarılan görüntülerden kaldırılır.
welcome-updates = Güncellemeler
welcome-check-updates = Güncellemeleri denetle
welcome-check-updates-hover = Uygulama başlarken GitHub’a en son sürümü sorar. Hiçbir şey gönderilmez.
welcome-files = Dosyalar
welcome-no-config = (yapılandırma klasörü yok)
welcome-open = Aç
welcome-rendering-on = İşleme aygıtı
welcome-open-help = Tüm yardımı aç  (F1)
# $icon: the Settings icon.
welcome-more-in-settings = Daha fazlası: Dosya → { $icon } Ayarlar
welcome-get-started = Başla

## Menus: shared

# A percentage in a menu (UI scale). $value: the number.
common-percent = %{ $value }
# A state shown after a menu item's name, in brackets.
menu-state-connected = bağlı
menu-state-rendering = işleniyor
menu-state-running = çalışıyor

## Navigate ▸ Bookmarks (also the toolbar's star)

menu-bookmarks = Yer imleri
menu-bookmarks-add = Geçerli görünümü ekle
menu-bookmarks-manage = Yönet…

## The fractal picker (Fractal menu and the toolbar's dropdown)

menu-family-powers = Yüksek kuvvetler
menu-family-automata = Otomatlar
menu-family-lsystems = L-sistemleri
menu-fractal-custom = Özel formül…
menu-fractal-library = Formül kitaplığı…
menu-fractal-library-hover = Kaydedilmiş özel formüller: uygulayın, düzenleyin, içe ve dışa aktarın
# The "Show" group: which picture of the formula is drawn. $name: the formula's name (Mandelbrot, Burning Ship…).
menu-show = Göster
menu-show-set = { $name } kümesi
menu-show-set-hover = Parametre düzlemi — { $name } kümesinin kendisi.

    Her noktası ayrı bir Julia kümesidir; bu, hepsinin haritasıdır.
menu-show-julia = Julia kümesi
menu-show-julia-hover = { $name } kümesinin tek bir noktası, kendi resmi olarak çizilmiş hâli — o noktayı sabit tuttuğunuzda elde ettiğiniz şekil.

    Noktayı “İkisi, bağlantılı” modunda haritanın üzerine gelerek seçin; orada tıklamak onu sabitler.
menu-show-julia-none = { $name } için serbest parametre yok, bu yüzden Julia kümesi de yok.
menu-show-both = İkisi, bağlantılı
menu-show-both-hover = Solda { $name } kümesi, sağda ise imlecinizin üzerinde durduğu noktanın Julia kümesi.

    İmleci harita üzerinde gezdirin ve Julia kümesinin değişimini izleyin — ikisinin ilişkisini görmenin en hızlı yolu budur.
menu-show-both-none = { $name } için eşleştirilecek bir Julia kümesi yok.

## File menu

menu-file = Dosya
menu-file-open-view = Görünüm veya konum aç…
menu-file-gallery = Galeri…
menu-file-export = Görüntüyü dışa aktar…
menu-file-snapshot = Anlık görüntü
menu-file-open-life = Life deseni aç…
menu-file-open-life-hover = Bir RLE, .cells ya da Life 1.05/1.06 dosyası; bir Life evreni olarak gösterilir.
menu-file-life-text = Life deseni metni…
menu-file-life-text-hover = Bir deseni metin olarak yapıştırın (LifeWiki’nin verdiği biçimde RLE).
menu-file-save-life = Life desenini kaydet…
menu-file-save-life-hover = Evrenin şu anki hâli, RLE olarak.
menu-file-open-lsystem = L-sistemi aç…
menu-file-open-lsystem-hover = Bir Fractint .l dosyası (içinden seçilecek girdileriyle) ya da bir .lsys sistemi.
menu-file-save-lsystem = L-sistemini kaydet…
menu-file-save-lsystem-hover = Sistemin metni, bir .lsys dosyası olarak.
menu-file-export-svg = SVG olarak dışa aktar…
menu-file-export-svg-hover = Görünüm bir SVG çizimi olarak: çizgileri ve şekilleri vektör olarak, kendi renkleriyle.
menu-file-render-client = İşleme istemcisi…
menu-file-render-client-hover = Bu makinenin GPU’sunu bir işleme çiftliğine ödünç verin: bir denetleyiciye bağlanın (başka bir makinede Araçlar ▸ Çiftlikte işle…) ve onun için kareler işleyin.
menu-file-settings = Ayarlar
menu-file-reset = Uygulama durumunu sıfırla…
menu-file-reset-hover = Kaydedilmiş tüm durumu (oturum, yer imleri, küçük resimler) siler ve sıfırdan başlar. Önce onay ister.
menu-file-quit = Çık

## File ▸ Settings

menu-settings-fps = Kare hızı sınırı
menu-settings-fps-uncapped = Sınırsız
# $fps: frames per second.
menu-settings-fps-n = { $fps } FPS
menu-settings-ui-scale = Arayüz ölçeği (yazı boyutu)
menu-settings-theme = Tema
menu-settings-updates = Güncellemeler
menu-settings-updates-stable-hover = En son kararlı sürüm
menu-settings-updates-beta-hover = Ön sürümler dahil en son derleme
menu-settings-updates-on-launch = Açılışta güncellemeleri denetle
menu-settings-updates-on-launch-hover = Aksi hâlde Yardım → Güncellemeleri denetle ile elle denetleyin.
# Followed by its shortcut, (Ctrl+S).
menu-settings-snapshot = Anlık görüntü
menu-settings-snapshot-ask-hover = Her basışta seçimi sor
menu-settings-snapshot-screen-hover = Görünümü tam olarak ekranda göründüğü gibi kaydet (anında, PNG)
menu-settings-snapshot-render-hover = Dışa aktarma ayarlarıyla, arka planda tam bir işleme (derin görünümlerde uzun sürebilir)
menu-settings-refcache = Referans önbelleği…
menu-settings-refcache-hover = Diskte tutulan derin yakınlaştırma referans yörüngeleri; böylece geri döndüğünüz bir konum saniyeler içinde işlenir. Nerede olduğu, ne kadar büyük olduğu, sınırı ve onu temizlemenin bir yolu.

## Fractal menu

menu-fractal = Fraktal

## View menu

menu-view = Görünüm
menu-view-zoom-home = Tam görünüme uzaklaş
menu-view-zoom-home-hover = Tam başlangıç görünümüne uzaklaş (animasyonlu)
menu-view-reset = Varsayılan görünüme sıfırla
menu-view-reset-hover = Fraktalın varsayılan görünümüne sıfırla (anında)
# Followed by its shortcuts.
menu-view-undo = Görünümü geri al
menu-view-redo = Görünümü yinele
menu-view-control-panel = Denetim paneli
menu-view-control-panel-hover = Sağdaki denetim panelini göster/gizle.
menu-view-minimap = Mini harita
menu-view-minimap-hover = “Buradasınız” işaretiyle ve yakınlaştırma derinliğiyle küçük bir başlangıç görünümü özeti gösterir. Bir bölgeye atlamak için üzerine tıklayın.
menu-view-perf-panel = Performans paneli
menu-view-timestamp = Zaman damgasını göster
menu-view-timestamp-hover = Görünümün üzerine büyük bir geçen süre saati çizer. Günlüğün her satırın başına yazdığı “+12.345s” değerinin aynısını gösterir; böylece yalnızca hareket sırasında ortaya çıkan bir sorunun ekran kaydı, günlükle kare kare eşleştirilebilir. Varsayılan olarak kapalı.
menu-view-zoom-target = Otomatik yakınlaştırma hedefini göster
menu-view-zoom-target-hover = Otomatik yakınlaştırma çalışırken görünümün yakınlaştırılan bölgesinin — birkaç büyütme daha sonra ekranı dolduracak kısmın — ana hatlarını, köşelerinden ekranın köşelerine uzanan çizgilerle birlikte çizer ve yöneldiği noktaya bir halka koyar. Varsayılan olarak kapalı.
menu-view-live-panel = Canlı işleme paneli
menu-view-live-panel-hover = Görünümün üzerinde, canlı işleyiciyi çalışırken gösteren bir ön panel: her karenin neyi gönderdiği, ekrandaki resmin ne kadar eski olduğu, yineleme bantları boyunca tarama, referans oluşturucular ve alttaki makine. B tuşu boyutları sırayla değiştirir, köşesindeki × onu kapatır; paneli herhangi bir köşeye sürükleyebilirsiniz; bir göstergenin ne anlama geldiğini ve sağlıklı ya da sorunlu durumun nasıl göründüğünü görmek için üzerine gelin.
# The live render panel's sizes.
menu-view-live-off = Kapalı
menu-view-live-off-hover = Panel yok.
menu-view-live-strip = Şerit
menu-view-live-strip-hover = Nixie göstergeleri ve lamba sırası.
menu-view-live-panel-size = Panel
menu-view-live-panel-size-hover = Göstergeler: tarama kadranı, resim yaşı, karolar, GPU geçişleri, derinlik kadranı, lambalar.
menu-view-live-full = Tam
menu-view-live-full-hover = Panel ve makine dairesi: adımlar, GPU’lar, bellek, çekirdekler, pikseller.
menu-view-live-opacity = Opaklık
menu-view-watermark = “Fd” filigranını göster
menu-view-watermark-hover = Canlı görünümün ve dışa aktarılan görüntülerin sağ alt köşesine göze batmayan bir “Fd” marka işareti çizer. Varsayılan olarak açık.

## Color menu

menu-color = Renk
menu-color-method = Yöntem
menu-color-palette = Palet
menu-color-custom = Özel
menu-color-duotone = İki ton
menu-color-binary = İkili (küme)
menu-color-binary-hover = Düz iki renk: küme içi ve küme dışı, gradyan yok.
menu-color-edit-gradient = Gradyanı düzenle…
menu-color-normalize = Derin renkleri normalleştir
menu-color-normalize-hover = Yoğun bir alan aksi hâlde benekli gürültü gibi görünecekse paleti görünümün ölçülen kaçış aralığına yeniden eşler. Örtüşme (aliasing) olmayan bir görünüme dokunulmaz — her durumda yeniden eşlemek için “Her zaman aralığa sığdır” seçeneğini işaretleyin. Yalnızca yumuşak yineleme yönteminde.
menu-color-more = Döngü, kaydırma, animasyon ve efektler: Denetimler paneli ▸ Renklendirme

## Tools menu

menu-tools = Araçlar
# Followed by its shortcut, (M).
menu-tools-find-minibrot = Minibrot bul + yakınlaştır
menu-tools-find-minibrot-hover = Görünüm merkezini Newton yöntemiyle yakındaki minibrotun tam merkezine oturtur, periyodunu bildirir ve minibrotun kendi ölçeğine yakınlaştırır — çoğu zaman tek adımda birçok büyüklük mertebesi. Zaten ondan daha derinde misiniz? O zaman yalnızca merkez değişir.
menu-tools-solver = Newton / Misiurewicz çözücü…
menu-tools-solver-hover = Görünüme en yakın minibrotun tam merkezini (periyot + boyut) ya da en yakın Misiurewicz noktasını çözer — tam hassasiyette Newton yöntemiyle, itici çarpan λ ile birlikte. Çözücüleri barındıran konum penceresini açar.
# Followed by its shortcut, (A).
menu-tools-autopilot = Otomatik yakınlaştırma (otopilot)
menu-tools-autopilot-stop = Otopilotu durdur
menu-tools-autopilot-hover = Elinizi sürmeden sürekli derin yakınlaştırma: ayrıntının en zengin olduğu bölgeye doğru dalar, ilerledikçe yönünü yeniden belirler. Herhangi bir gezinme girdisi onu durdurur.
menu-tools-priority = Öncelik:
menu-tools-priority-speed = Hız
menu-tools-priority-speed-hover = Ayarlanan hızda yakınlaştırır; ekranı ayrıntı içeren ilk kareyle yeniler.
menu-tools-priority-quality = Kalite
menu-tools-priority-quality-hover = Yalnızca tamamen çözümlenmiş kareler; yakınlaştırma onları bekler.
menu-tools-target = Hedef:
menu-tools-target-detail = Ayrıntı
menu-tools-target-detail-hover = Görünümdeki en zengin kenar; dalış ilerledikçe yeniden seçilir.
menu-tools-target-misiurewicz = Misiurewicz noktası
menu-tools-target-misiurewicz-hover = En yakın Misiurewicz noktası; bir kez çözülür ve tam içine dalınır (yalnızca Mandelbrot).
menu-tools-orbits = Yörünge kaplaması
menu-tools-orbits-hover = İmlecin altındaki noktanın yineleme yolunu çizer. Animasyon seçenekleri: Denetimler paneli ▸ Kaplamalar.
menu-tools-orbits-normalize = Normalleştir (görünüme sığdır)
menu-tools-orbits-normalize-hover = Yörüngeyi tüm görünüme sığdırır; böylece her yakınlaştırmada iyi çerçevelenmiş kalır (görüntü alanı üzerinden eşlemek yerine; o eşlemede yörünge derinde ekranın dışına uçar).
menu-tools-play-tour = Tur oynat…
menu-tools-play-tour-hover = Oynatılacak bir tur betiği seçin. Araç çubuğundaki ▶ sonuncusunu tek tıklamayla yeniden oynatır.
menu-tools-tour-from-view = Geçerli görünümden tur…
menu-tools-tour-from-view-hover = Tam görünümden geçerli görünüme kadar yakınlaştıran bir tur oluşturun — bir altyazı ekleyin ve dalış süresini ayarlayın. Düzenleyebileceğiniz, oynatabileceğiniz ya da işleyebileceğiniz bir tur betiği (.toml) olarak kaydedilir.
menu-tools-render-tour = Tur işle…
menu-tools-render-tour-hover = Yüklü turu ayrı bir süreçte bir PNG kare dizisine (ve isteğe bağlı olarak bir mp4 dosyasına) işler.
menu-tools-close-player = Tur oynatıcısını kapat
menu-tools-close-player-hover = Turu durdurur, oynatıcısını kapatır ve kendi yineleme ve renklendirme ayarlarınızı geri yükler
menu-tools-render-farm = Çiftlikte işle…
menu-tools-render-farm-hover = Yüklü turu birkaç makinede işler: bu makine işi yönetir, diğerleri Dosya ▸ İşleme istemcisi… ile katılır.
menu-tools-benchmark = Kıyaslama…
menu-tools-benchmark-hover = İşleme hızını ölçer — geçerli ayarlarla ya da makineler arasında karşılaştırılabilir standart bir çalıştırmayla (sabit çözünürlük + ayarlar). Dayanıklılık testi, kararlılığı / ısıl kısmayı denetlemek için onu tekrarlar.
menu-tools-last-benchmark = Son kıyaslamayı göster

## Navigate menu

menu-navigate = Gezinme
menu-navigate-goto = Konuma git…
menu-navigate-goto-hover = Bir merkez/yakınlaştırma girin, iyi bilinen bir noktaya atlayın ya da görünümün yakınındaki bir Misiurewicz / minibrot özelliğini Newton yöntemiyle çözün.
menu-navigate-misiurewicz = Misiurewicz gezgini…
menu-navigate-misiurewicz-hover = Küçük Misiurewicz türlerinin neye benzediğine göz atın — her noktanın yakınlaştırma periyodu ve burulmasıyla küçük resimler — ve herhangi birini Newton yöntemiyle istediğiniz derinliğe kadar çözün. (Mandelbrot.)
menu-navigate-exact = Tam noktalar (ifadeler)
menu-navigate-exact-note = Ana kardioidin tomurcuklarının kökleri, tam ifadeler olarak verilmiştir (2π’nin bir kesrinin cos/sin değeri). Koordinat ifade olarak tutulur; böylece yakınlaştırdıkça tam olarak noktanın üzerinde kalınır.
menu-navigate-random = Rastgele konum
menu-navigate-random-hover = Ayrıntı açısından zengin, rastgele bir sınır noktasına atla
menu-navigate-import-kfr = .kfr içe aktar…
menu-navigate-import-kfr-hover = Bir Kalles Fraktaler konum dosyası yükle
menu-navigate-share = Konumu paylaş…
menu-navigate-share-hover = Kendi içinde eksiksiz bir konumu (.fdn) kopyalayın / yapıştırın / kaydedin / yükleyin: fraktal, tam hassasiyetli merkez, yakınlaştırma, renklendirme.

## Help menu

menu-help = Yardım
# Followed by its shortcut, (F1).
menu-help-reference = Yardım ve başvuru…
menu-help-welcome = Hoş geldiniz / hızlı başlangıç…
menu-help-welcome-hover = İlk çalıştırmadaki genel bakış: temel kontroller ve birkaç ünlü konum.
menu-help-diagnostics = Tanılama…
menu-help-diagnostics-hover = Fractadyne’ın donanımınızda doğru çalıştığını denetleyin ve sonucu bir sorun raporuna ekleyin
menu-help-report = Sorun bildir…
menu-help-report-hover = Kilitlenme raporu, günlük ve konum eklenmiş bir GitHub sorunu (ya da e-posta) gönderin
menu-help-accelerated = Daha hızlı derin yakınlaştırma…
menu-help-accelerated-hover = Derin yakınlaştırma referans yörüngelerini 2,5-6,4 kat daha hızlı hesaplayan isteğe bağlı bir derleme. Görüntüler aynıdır; ayarlarınız aktarılır.
menu-help-check-updates = Güncellemeleri denetle
# $track: "stable" or "beta", the update track in use (the variants hold the wording).
menu-help-check-updates-hover =
    { $track ->
        [beta] GitHub’da daha yeni bir Beta (ön sürüm) derlemesi olup olmadığını denetle
       *[other] GitHub’da daha yeni bir Kararlı derleme olup olmadığını denetle
    }
# $version: the new version; $channel: "beta" for a pre-release, "stable" otherwise (the variants hold the wording).
menu-help-update-available =
    { $channel ->
        [beta] { $version } (beta) mevcut — indir
       *[other] { $version } (kararlı) mevcut — indir
    }
menu-help-up-to-date = Güncel
menu-help-tagline = Yerel fraktal gezgini
menu-help-license = Lisans
# $year and $holder: the copyright year and holder, as the code gives them.
menu-help-copyright = © { $year } { $holder }. Çift lisanslıdır; dilediğiniz lisansın koşullarıyla kullanabilirsiniz.
menu-help-source = Kaynak kod

## The toolbar (its buttons are icons; these are their tooltips)

menu-toolbar-julia = Julia
menu-toolbar-julia-hover = Bu formülün Julia kümesini pencereyi dolduracak şekilde göster
menu-toolbar-dual-hover = İkisi, bağlantılı: solda küme, sağda imlecin bulunduğu noktanın Julia kümesi
menu-toolbar-open-hover = Bir görünüm ya da konum aç (PNG / EXR, .fdn, .kfr)
menu-toolbar-gallery-hover = Galeri
menu-toolbar-export-hover = Görüntüyü dışa aktar…
menu-toolbar-share-hover = Konumu paylaş — kendi içinde eksiksiz bir .fdn dosyasını kopyala, yapıştır, kaydet ya da yükle (fraktal, tam hassasiyetli merkez, yakınlaştırma, renklendirme)
menu-toolbar-zoom-in-hover = Yakınlaştır
menu-toolbar-zoom-out-hover = Uzaklaştır
# $factor: the click-zoom factor (4, 10…).
menu-toolbar-click-zoom-hover = Tıklayarak yakınlaştırma ({ $factor }×): sol tık noktaya dalar, sağ tık geri çıkar (sürükleme yine kaydırır). Çarpan: Denetimler ▸ Gezinme.
# Added to the click-to-zoom tooltip in the dual view.
menu-toolbar-click-zoom-dual = Bu açıkken Julia c’yi Ctrl+tık ile sabitleyin.
menu-toolbar-autopilot-running-hover = Otomatik yakınlaştırma çalışıyor — durdurmak için tıklayın
menu-toolbar-autopilot-hover = Otomatik yakınlaştırma: ayrıntıya doğru dal (A)
menu-toolbar-reset-hover = Görünümü sıfırla (anında)
menu-toolbar-home-hover = Başlangıç görünümüne uzaklaş (animasyonlu)
menu-toolbar-bookmarks-hover = Yer imleri: bu görünümü kaydedin ya da kayıtlı bir görünüme atlayın
# $name: the palette's name.
menu-toolbar-next-palette-hover = Sonraki palet ({ $name })
# $file: the last tour's file name.
menu-toolbar-play-last-hover = Tur oynat — { $file } (tıklayın) · son turu yeniden oynatır; başka birini Araçlar ▸ Tur oynat… ile seçin
menu-toolbar-play-hover = Bir tur oynat… (Araçlar ▸ Tur oynat)
menu-toolbar-speed-hover = Oynatma hızı
menu-toolbar-fullscreen-hover = Tam ekranı aç/kapat

## Dialogs: words shared by many windows

# A button that closes a window without doing what it offers.
dlg-cancel = İptal
# A button that dismisses a window (nothing in it is pending).
dlg-close = Kapat
# A button that copies the window's text to the clipboard (after a copy icon).
dlg-copy = Kopyala
# A button that jumps to a location.
dlg-go = Git
# A button that fills the fields from the view on screen.
dlg-use-current = Geçerliyi kullan
# A notice after a file could not be written. $error: the system's reason, as it gives it.
dlg-save-failed = Kaydedilemedi: { $error }
# A notice when no graphics device is available to render with.
dlg-gpu-not-available = GPU kullanılamıyor
# The name of the plain-text file type in a save dialog (the extension .txt is added by the code).
dlg-filter-text = Metin

## Asked after a session that did not shut down cleanly

dlg-crash-title = Fractadyne düzgün kapanmadı
dlg-crash-intro = Önceki oturum beklenmedik şekilde sona erdi. Bu makineye bir rapor kaydedildi — göndermek ister misiniz?
dlg-crash-review = Siz gözden geçirmeden hiçbir şey gönderilmez: sonraki ekran neyin ekleneceğini tam olarak gösterir ve sistem bilgilerini hariç tutmak için tek bir onay kutusu yeterlidir.
dlg-crash-report = Bildir…
dlg-crash-report-hover = Sorun bildir penceresini, kilitlenme raporu ve kilitlenmenin yaşandığı konum eklenmiş olarak açar.
dlg-crash-not-now = Şimdi değil
dlg-crash-dont-ask = Kilitlenmeden sonra bir daha sorma
dlg-crash-dont-ask-hover = Raporlar yine de yerel olarak kaydedilir ve Yardım ▸ Sorun bildir her zaman çalışır.

## Asked after a session that ended with a second graphics device in use

dlg-second-device-title = İkinci grafik aygıtı kapatılsın mı?
# $device: the graphics device's name and details, as the system gives them.
dlg-second-device-intro = Önceki oturum, ikinci bir grafik aygıtı kullanımdayken beklenmedik şekilde sona erdi ({ $device }). Nedeni bu olabilir.
dlg-second-device-note = Kapalıyken, eskiden olduğu gibi her şeyi pencereyi çizen kart işler. Gelişmiş ▸ İkinci grafik kartı bölümünden yeniden açabilirsiniz.
dlg-second-device-off = Kapat
dlg-second-device-keep = Açık kalsın

## Shown when the previous session's iteration setting was set aside

dlg-quarantine-title = Yineleme ayarı bir kenara alındı
dlg-quarantine-change = Bunu istediğiniz zaman Denetimler ▸ Kalite bölümünden değiştirebilirsiniz.
dlg-quarantine-keep = Varsayılanı koru
# A button. $setting: the iteration setting the previous session used (such as "20,000 iterations").
dlg-quarantine-restore = Geri yükle: { $setting }

## Navigate ▸ Go to location

dlg-goto-title = Konuma git
# Before two choices: "x, y" (cartesian) and the polar form below.
dlg-goto-enter-as = Giriş biçimi:
# The polar way of entering a point: an offset plus a radius r at an angle θ. Keep the symbols.
dlg-goto-polar = öteleme + r ∠ θ
# The tooltip of the "?" button beside the entry choice.
dlg-goto-help-hover = Alanların kabul ettikleri: kesirler, kuvvetler, sqrt/sin/cos…, pi ve diğer sabitler — Yardım → Koordinat ifadeleri bölümünü açar
# Labels over the entry fields. x0, y0, r, θ, Re and Im are the fields' symbols: keep them.
dlg-goto-polar-offset = Öteleme merkezi  (x0, y0 — saf kutupsal bir nokta için 0 bırakın)
dlg-goto-polar-radius = Yarıçap r, açı θ
dlg-goto-re = Merkez — gerçel (Re)
dlg-goto-im = Merkez — sanal (Im)
dlg-goto-zoom = Yakınlaştırma (büyütme)
# A warning under the zoom field, after a ⚠ sign. $have: the iteration count set; $typical: the count this depth typically needs.
dlg-goto-iter-warning = Yineleme sayısı { $have } değerine sabitlenmiş; bu derinlik genellikle ~{ $typical } gerektirir. Bu değer yükseltilene kadar görünüm düz işlenir — ya da kendi kendine uyum sağlayan otomatik yinelemeyi açın (Denetimler ▸ Kalite).
dlg-goto-copy-hover = Bu konumu panoya kopyala
dlg-goto-cancel-hover = Hiçbir yere gitmeden kapat
# The expressions in brackets are examples of what can be typed: keep them as they are.
dlg-goto-paste-note = Başka birinden bir merkez/yakınlaştırma yapıştırın ya da paylaşmak için kopyalayın. Koordinatlar ifade kabul eder: tam kesirler (-3/4), fonksiyonlar ve sabitler (-0.5 + 0.25*cos(pi/4), sqrt, root(x,n), ^; açılar radyan cinsinden); gerçel alan ise ikisini birden dolduran bütün bir karmaşık değeri de kabul eder ((37+16i)/100).
# A section that opens and closes.
dlg-goto-feature = Özelliğe git (Mandelbrot)
dlg-goto-feature-poi = İyi bilinen noktalar:
# The dropdown's text before a point is picked.
dlg-goto-feature-poi-pick = İlginç bir noktaya atla…
dlg-goto-feature-solve = Ya da geçerli görünümün yakınındaki bir özelliği çözün:
# Two choices: what to solve for. Misiurewicz is a mathematician's name (a Misiurewicz point).
dlg-goto-feature-misiurewicz = Misiurewicz
dlg-goto-feature-minibrot = En yakın minibrot
# Labels of two small number fields; k and p are their symbols.
dlg-goto-feature-preperiod = Ön periyot k
dlg-goto-feature-period = Periyot p
# The faint text in an empty field: left empty, the value is found automatically.
dlg-goto-feature-auto-hint = otomatik
# A small button that empties both fields, so they are found automatically again.
dlg-goto-feature-auto = Otomatik
dlg-goto-feature-auto-hover = İki kutuyu da boşaltır; böylece sonraki arama ön periyodu ve periyodu görünüm merkezindeki yörüngeden algılar.
dlg-goto-feature-kp-hover = Görünüm merkezindeki yörüngeden algılanmaları için ikisini de boş bırakın; bulunan değerler buraya doldurulur. Bir spirale bakan hiç kimse onun ön periyodunu bilmez; bir değer girmek zorunda olmak, bu çözücüyü pratikte kullanılamaz kılan şeydi.
# While a solve runs, followed by its elapsed time and the note below.
dlg-goto-feature-solving = Çözülüyor…
dlg-goto-feature-solving-note = (keyfi hassasiyet; derin görünümler daha uzun sürer)
dlg-goto-feature-cancel-hover = Beklemeyi bırak. Çözüm arka planda sonuna kadar çalışır ve yanıtı atılır — yarım kalmış hiçbir şey bırakılmaz.
dlg-goto-feature-find = Görünümün yakınında bul
dlg-goto-feature-find-hover =
    Geçerli merkezden tam özelliğe Newton yöntemiyle çöz (keyfi hassasiyet). Önce özelliğin yakınına gidin.

    Çözmeden önce yukarıdaki Yakınlaştırma alanını İSTEDİĞİNİZ derinliğe ayarlayın: nokta o derinliğin gerektirdiği hassasiyetle hesaplanır ve görünüm doğrudan oraya atlar. Uzun bir elle yakınlaştırma adımları dizisi yerine tek bir çözüm.
dlg-goto-solve-cancelled = Çözüm iptal edildi.

## Snapshot (the camera button): its tooltip, and the choice on the first press

# The camera button's tooltip, one per Snapshot setting. Followed by its shortcut, (Ctrl+S).
dlg-snapshot-hover-ask = Anlık görüntü — görünümü gösterildiği gibi ya da tam işleme olarak kaydet; ilk basışta sorar
dlg-snapshot-hover-screen = Anlık görüntü — görünümü tam olarak gösterildiği gibi son dışa aktarma klasörüne kaydet
dlg-snapshot-hover-render = Anlık görüntü — Dışa aktarma ayarlarıyla tam işleme, son dışa aktarma klasörüne
# The window's title.
dlg-snapshot-title = Anlık görüntü
dlg-snapshot-question = Anlık görüntü (kamera düğmesi, Ctrl+S) neyi kaydetsin?
# The first choice, and a short tag beside it.
dlg-snapshot-screen = Ekran görüntüsü
dlg-snapshot-screen-tag = anında
dlg-snapshot-screen-note = Görünümü tam olarak ekranda göründüğü gibi, ekran çözünürlüğünde, son dışa aktarma klasörüne PNG olarak kaydeder. Ne görüyorsanız onu alırsınız.
# The second choice, and a short tag beside it.
dlg-snapshot-render = Tam işleme
dlg-snapshot-render-tag = uzun sürebilir
# $size: the image size, as 1920×1080; $ss: the supersampling factor (1–4); $format: PNG or EXR.
dlg-snapshot-render-note = Dışa aktarma ayarlarıyla işler — { $size }, { $ss }× süper örnekleme, { $format }. En iyi kalite, ancak derin bir görünüm dakikalar ile saatler arasında sürebilir. Arka planda çalışır; Dosya → Görüntüyü dışa aktar… ilerlemeyi gösterir ve işlemi iptal edebilir.
dlg-snapshot-remember = Seçimimi hatırla ve bir daha sorma
dlg-snapshot-change = Bunu istediğiniz zaman Dosya → Ayarlar → Anlık görüntü bölümünden değiştirebilirsiniz.

## Share location (.fdn)

dlg-share-title = Konumu paylaş
dlg-share-intro = Kendi içinde eksiksiz bir konum (fraktal, tam hassasiyetli merkez, yakınlaştırma, renklendirme). Paylaşmak için kopyalayın ya da bir konum yapıştırın/yükleyin ve Uygula düğmesine basın.
dlg-share-thumb = Kaydedilen dosyaya küçük resim ekle
dlg-share-thumb-hover = .fdn kaydedilirken bu görünümün küçük bir resmi dosyaya gömülür; böylece dosya galeride tanınabilir. Yaklaşık 55 KB ekler ve yukarıdaki metnin parçası değildir.
dlg-share-copy-hover = Metni panoya kopyala
# Buttons (the first after a save icon).
dlg-share-save = .fdn kaydet…
dlg-share-load = .fdn yükle…
dlg-share-apply = Uygula
dlg-share-apply-hover = Kutudaki konuma atla
dlg-share-copied = Panoya kopyalandı.

## Help ▸ Report an issue (the report itself stays in English: it is read by the maintainer)

dlg-report-title = Sorun bildir
dlg-report-intro = Bildirmek için en iyi yer bir GitHub sorunudur — herkese açıktır, aranabilir ve ne zaman düzeltildiğini görürsünüz. E-posta da olur. Önce aşağıdakilerin tümünü gözden geçirin: siz bir işlem yapmadan hiçbir şey makinenizden çıkmaz.
# Labels before three dropdowns.
dlg-report-type = Tür:
dlg-report-severity = Önem derecesi:
dlg-report-repro = Yeniden üretilebilir:
dlg-report-describe = Ne olduğunu anlatın:
# The faint text in the empty description box.
dlg-report-describe-hint = Ne oldu, yeniden üretmek için hangi adımlar gerekiyor ve bunun yerine ne bekliyordunuz.
# Before the checkboxes of what the report carries.
dlg-report-include = Ekle:
dlg-report-sysinfo = Sistem bilgisi (sürüm, işletim sistemi, CPU, GPU, VRAM)
dlg-report-location = Geçerli konum (.fdn)
dlg-report-log = Son günlük
dlg-report-frames = Kare kaydı (son dakika ve son 40 kare)
dlg-report-crash = Son kilitlenme raporu
dlg-report-crash-none = Son kilitlenme raporu (bulunamadı)
dlg-report-crash-view = Kilitlenmenin yaşandığı konum (.fdn)
dlg-report-crash-view-none = Kilitlenmenin yaşandığı konum (.fdn) (bulunamadı)
dlg-report-crash-view-hover = Bir kilitlenmenin ya da grafik aygıtı kaybının yaşandığı görünümün tam kendisi — geliştiricinin sorunu yeniden üretebilmesini sağlayan tek şey. Otomatik olarak kaydedilir.
dlg-report-test = Tanılama testi sonucu
dlg-report-test-none = Tanılama testi sonucu (Yardım → Tanılama… ile bir test çalıştırın)
dlg-report-test-hover = Kendi donanımınızdan, makine tarafından doğrulanmış bir geçti/kaldı sonucu — geliştirici için tek başına bir açıklamadan çok daha yararlı
# A section that opens and closes. $size: the report's size, as "12 KB".
dlg-report-preview = Gönderilecekleri önizle  ({ $size })
dlg-report-github = GitHub sorunu aç
dlg-report-github-hover = Başlığı doldurulmuş bir yeni sorun sayfası açar; rapor panonuza kopyalanır — onu sorunun gövdesine yapıştırın (Ctrl+V).
# Buttons (after a copy and a save icon).
dlg-report-copy = Raporu kopyala
dlg-report-copy-hover = Her şeyi panoya kopyala
dlg-report-save = Raporu kaydet…
dlg-report-save-hover = Eklemek üzere .txt olarak kaydet
dlg-report-by-email = Ya da e-postayla:
dlg-report-gmail = Gmail’de oluştur
# $address: the email address reports go to.
dlg-report-gmail-hover = { $address } adresine, konusu doldurulmuş bir Gmail ileti penceresi açar; kopyalanan raporu yapıştırın
dlg-report-email = E-posta uygulaması…
dlg-report-email-hover = Varsayılan e-posta uygulamanızı açar (mailto) — yapılandırılmış bir e-posta işleyicisi gerekir
dlg-report-tip = İpucu: bunlardan herhangi birinde rapor panonuza kopyalanır — sorunun ya da iletinin gövdesine yapıştırın (Ctrl+V). Mümkünse bir ekran görüntüsü ya da örnek görüntü de ekleyin (sürükleyip bırakın); işleme ve arayüz sorunlarında çok yardımcı olur.
# Notices in the window after a button is pressed.
dlg-report-opened-github = GitHub açıldı — rapor panoda; onu soruna yapıştırın.
dlg-report-copied = Rapor panoya kopyalandı.
# $path: where the report was saved.
dlg-report-saved = Kaydedildi: { $path }
dlg-report-opened-email = E-posta uygulamanız açıldı — rapor panoda; iletiye yapıştırın.
dlg-report-opened-gmail = Gmail ileti penceresi açıldı — kopyalanan raporu gövdeye yapıştırın (Ctrl+V).

## A location whose checksum does not match

dlg-checksum-title = Bu konum bozulmuş olabilir
# After a ⚠ sign.
dlg-checksum-mismatch = Sağlama toplamı, yanındaki verilerle eşleşmiyor.
dlg-checksum-why = Bu genellikle metnin buraya gelirken kısaldığı ya da değiştiği anlamına gelir — bir sohbet penceresinden ya da forum gönderisinden yapılan eksik bir kopyalama yaygın bir nedendir.
# A label over where the location came from (a file path, or the clipboard).
dlg-checksum-source = Kaynak
# The two checksums, side by side: the one written in the file, and the one computed from its data.
dlg-checksum-in-file = dosyada
dlg-checksum-from-data = verilerden
dlg-checksum-load-note = Yine de yükleyebilirsiniz — koordinatlar hâlâ doğru olabilir ve hiçbir şeyin üzerine yazılmaz. Okunamayan her şey yüklemeden sonra bildirilir.
dlg-checksum-load = Yine de yükle
dlg-checksum-load-hover = Uyuşmazlığa rağmen konuma atla
# Notices after loading it. $zoom: the magnification, as 1.2e148.
dlg-checksum-loaded = Yüklendi @ { $zoom }×, sağlama toplamı uyuşmazlığına rağmen
# $zoom: the magnification, as 1.2e148; $note: what the loader found wrong with the location.
dlg-checksum-loaded-note = Yüklendi @ { $zoom }× — { $note }
dlg-checksum-not-loaded = Yüklenmedi.

## File ▸ Reset application state

dlg-reset-title = Uygulama durumunu sıfırla
# After a ⚠ sign, followed by a list.
dlg-reset-warning = Bu işlem kaydedilmiş tüm Fractadyne verilerini kalıcı olarak siler:
# The list's items, each after a bullet.
dlg-reset-session = kaydedilmiş oturum (geçerli görünüm, renklendirme, tercihler)
dlg-reset-bookmarks = tüm yer imleri ve küçük resimleri
dlg-reset-refcache = referans yörünge önbelleği (konumları yeniden ziyaret ettikçe yeniden oluşturulur)
# A label over the folder that is deleted.
dlg-reset-location = Klasör
dlg-reset-note = Bu işlem geri alınamaz. Geçerli oturum çıkışta yeniden kaydedilmez; böylece bir sonraki açılışta varsayılanlar yüklenir.
dlg-reset-confirm = Her şeyi sıfırla
dlg-reset-done = Uygulama durumu sıfırlandı — bir sonraki açılışta varsayılanlar yüklenecek.
# $error: the system's reason, as it gives it.
dlg-reset-failed = Sıfırlama başarısız: { $error }

## Tools ▸ Tour from current view

dlg-tour-title = Geçerli görünümden tur
dlg-tour-intro = Tam görünümden şu an bulunduğunuz yere kadar yakınlaştıran bir tur oluşturur.
# A label before the fractal and magnification the tour ends at.
dlg-tour-destination = Hedef
dlg-tour-notation = Not (dalış sırasında gösterilen isteğe bağlı altyazı)
# The faint example text in the empty caption box.
dlg-tour-notation-hint = örn. Misiurewicz üç kollu spiraline dalış…
dlg-tour-duration = Yakınlaştırma süresi
# $rate: how fast the dive zooms, in powers of ten a second (as 2.5).
dlg-tour-rate = ≈ { $rate } dekad/s
# A small button that puts the duration back to its default.
dlg-tour-reset = Sıfırla
dlg-tour-reset-hover = Derinliğe göre varsayılan
dlg-tour-duration-note = Varsayılan süre, dalış izlenebilir kalsın diye derinlikle birlikte ölçeklenir. Daha hızlı dalışlar aşırı yakınlaştırmada tek renkli kareler gösterebilir.
# A button, after a save icon.
dlg-tour-save = Turu kaydet…
# The file type's name in the save dialog.
dlg-tour-filter = Fractadyne betiği (TOML)
# A notice. $file: the saved file's name. --render-tour is a command-line option: keep it.
dlg-tour-saved = Tur kaydedildi → { $file }. Araçlar → Tur oynat ile oynatın ya da --render-tour ile işleyin.

## Navigate ▸ Bookmarks ▸ Manage…

dlg-bookmarks-title = Yer imleri
# The faint text in the empty name box.
dlg-bookmarks-name-hint = ad (isteğe bağlı)
# A button, after a bookmark icon.
dlg-bookmarks-add = Geçerli görünümü ekle
dlg-bookmarks-empty = Henüz yer imi yok. Geçerli görünümden bir tane ekleyin.
# The tooltip of a bookmark's delete (bin) button.
dlg-bookmarks-delete-hover = Sil

## Tools ▸ Benchmark…

dlg-bench-title = Kıyaslama
dlg-bench-mode = Mod
dlg-bench-current = Geçerli ayarlar
dlg-bench-current-hover = Sabit derin yakınlaştırma turunu, geçerli çözünürlüğünüz ve ayarlarınızla canlı görünümde oynatır.
dlg-bench-standard = Standart
dlg-bench-standard-hover = Çözünürlüğü ve tüm işleme ayarlarını sabitler; böylece puan makineler arasında karşılaştırılabilir olur.
dlg-bench-resolution = Çözünürlük
dlg-bench-depth = Derinlik
# Standard and Ultra are the two depths in the dropdown this describes.
dlg-bench-depth-hover = Her iki dalış da derin floatexp/BLA yoluna geçer; Ultra (1e48×) orada Standart’tan (1e32×) çok daha uzun süre kalır ve çalışması da buna göre daha uzun sürer.
dlg-bench-burnin = Dayanıklılık testi (tekrarla)
dlg-bench-burnin-hover = Kararlılığı ve ısıl kısmayı ortaya çıkarmak için kıyaslamayı tekrar tekrar çalıştırır.
# The label of a slider: how many times the benchmark runs.
dlg-bench-passes = geçiş
# $size: the image size, as 1920×1080; $ss: the supersampling factor; $depth: the dive's final magnification, as 1e32.
dlg-bench-std-summary = Ekran dışında { $size }, { $ss }× SS, Mandelbrot/yumuşak ile işler; { $depth }× düzeyine 60 karelik dalış.
dlg-bench-run = Çalıştır

## The standardized benchmark's progress window

dlg-bench-progress-title = Kıyaslama çalışıyor…
# $resolution: the benchmark's resolution, as the dropdown names it.
dlg-bench-progress-mode = Standart · { $resolution }
# $pass: the pass running now; $total: how many passes there are.
dlg-bench-progress-pass = Dayanıklılık geçişi { $pass }/{ $total }
dlg-bench-progress-dive = Sabit dalış işleniyor…
# On the progress bar. $done: frames rendered; $total: frames in the dive.
dlg-bench-progress-frame = kare { $done }/{ $total }
# $fps: frames per second, as 41.3.
dlg-bench-progress-last = son geçiş: { $fps } fps

## The benchmark's results

dlg-bench-results-title = Kıyaslama sonuçları
# Buttons, each after an icon.
dlg-bench-results-run-again = Yeniden çalıştır…
dlg-bench-results-save = Kaydet…
dlg-bench-results-none = Henüz hiç kıyaslama çalıştırılmadı.
dlg-bench-results-saved = Kıyaslama kaydedildi.

## The accelerated build's notice when its arithmetic libraries are missing

dlg-backend-title = Hızlandırılmış derleme — yerleşik aritmetik kullanılıyor
dlg-backend-dont-show = Bunu bir daha gösterme

## File ▸ Gallery…

dlg-gallery-title = Galeri
# A button that picks the folder to show.
dlg-gallery-folder = Klasör…
# A button, after a refresh icon.
dlg-gallery-refresh = Yenile
dlg-gallery-empty = Bu klasörde Fractadyne görüntüsü yok.
# An image's title when it does not name its fractal.
dlg-gallery-untitled = Fractadyne görüntüsü
# $zoom: the image's magnification, as 1.2e148; $saved: when it was saved, as the file gives it.
dlg-gallery-zoom = yakınlaştırma { $zoom }   { $saved }
# The notes saved with the image, in quotation marks. Use your language's quotation marks.
dlg-gallery-notes = “{ $notes }”
dlg-gallery-open = Bu görünümü aç
dlg-gallery-loaded = Görünüm galeriden yüklendi.

## File ▸ Export image…

dlg-export-title = Görüntüyü dışa aktar
# A dropdown of image sizes.
dlg-export-size = Boyut
# The size dropdown's text for a size set by hand. $size: the size, as 1920×1080.
dlg-export-size-custom-current = Özel — { $size }
# The size dropdown's first choice.
dlg-export-size-custom = Özel…
dlg-export-size-custom-hover = Genişliği kendiniz ayarlayın; yükseklik aşağıdaki En-boy oranına göre belirlenir.
dlg-export-width = Genişlik (px)
# After the width field. $height: the height in pixels, which follows from the aspect.
dlg-export-height-from-aspect = × { $height } (En-boy oranından)
dlg-export-ss = Süper örnekleme
dlg-export-aspect = En-boy oranı
# The aspect choice that keeps the window's own shape.
dlg-export-aspect-window = Pencereye uy
# Before the choices PNG and OpenEXR.
dlg-export-format = Biçim:
dlg-export-hud = Konum göstergesi
dlg-export-hud-hover = Dışa aktarılan görüntünün sol üst köşesine kalıcı olarak bir yakınlaştırma düzeyi + koordinat paneli ekler (çıktıyla birlikte ölçeklenir; harita/Mandelbrot görünümünün yakınlaştırmasını + merkezini gösterir).
dlg-export-glitch = Bozulma düzeltme
dlg-export-glitch-hover = Pertürbasyon bozulmalarının çoklu referansla düzeltilmesi. Çok derin (floatexp) tek görünümlü dışa aktarmalarda otomatik olarak atlanır; böylece referans oluşturma + işleme ayrı bir iş parçacığında çalışır ve uygulama yanıt vermeye devam eder. Mandelbrot ya da Multibrot görünümlerinde (Julia modu dışında) hiç kullanılmaz: kesin bir hesaplamayla karşılaştırıldığında orada hiçbir şeyi düzeltmedi ve dışa aktarma süresinin 5 katına kadar zaman aldı.
dlg-export-deep-note = Derin dışa aktarma: referans ayrı bir iş parçacığında oluşturulur (arayüz canlı kalır); bu derinlikte bozulma düzeltme atlanır.
# A dropdown, shown in the dual (map + Julia) view, and its three choices.
dlg-export-dual = İkili düzen
dlg-export-dual-side = Yan yana
dlg-export-dual-separate = Ayrı dosyalar
dlg-export-dual-map = Yalnızca harita
dlg-export-notes = Notlar:
# The faint text in the empty notes box.
dlg-export-notes-hint = görüntüyle birlikte kaydedilir (≤120 karakter)
# $width and $height: the image size in pixels; $left: how many characters the notes can still take.
dlg-export-output = Çıktı: { $width } × { $height } px   ({ $left } karakter kaldı)
# When the export's shape differs from the window's. $percent: how much more, as 25.
dlg-export-extra-width = Pencereden %{ $percent } daha fazla genişlik gösterir (hiçbir şey kırpılmaz)
# The same, when the export is taller than the window. $percent: how much more, as 25.
dlg-export-extra-height = Pencereden %{ $percent } daha fazla yükseklik gösterir (hiçbir şey kırpılmaz)
dlg-export-tiles-note = Arka plandaki bir iş parçacığında karolar hâlinde işlenir (boyut sınırı yok). Dosya, görünümü + notları içerir; böylece Dosya ▸ Görünüm veya konum aç ile yeniden açılabilir.
# A label before the button that picks the folder exports go to.
dlg-export-folder = Klasör:
dlg-export-choose = Seç…
dlg-export-choose-hover = Dışa aktarmaların gideceği hedef klasörü seçin
dlg-export-open-after = Bitince aç
dlg-export-open-after-hover = Bitmiş görüntüyü, yazılır yazılmaz sistemin görüntüleyicisine verir.
# $count: the graphics cards in this machine (2 or more).
dlg-export-all-gpus = Tüm grafik kartlarını kullan ({ $count })
dlg-export-all-gpus-hover = Görüntüyü karo karo bu makinenin tüm grafik kartlarına böler; her kart derin referansın kendi kopyasını kullanır. Aynı model kartlar, tek bir kartın tek başına işlediği görüntünün tıpatıp aynısını verir; farklı modeller birkaç pikseli karo karo biraz farklı çizer (ölçüldü: görünür bir dikiş yok). Bir kart başarısız olursa onun karoları diğerlerine gider. Bölmenin üstlenemediği görüntüler (bazı renklendirme yöntemleri ve formüller) bu pencerenin kartını kullanır.
# The export's progress.
dlg-export-preparing = Hazırlanıyor (referans oluşturuluyor)…
dlg-export-saving = Kaydediliyor…
dlg-export-rendering = İşleniyor…
# $time: the time since the export started, as 8.3s or 1m 04.0s.
dlg-export-elapsed = Geçen süre: { $time }
dlg-export-cancel-hover = Dışa aktarmadan kapat
# A button, after a save icon.
dlg-export-save-as = Farklı kaydet…
dlg-export-save-as-hover = Dosya adını ve konumunu seçin
dlg-export-export = Dışa aktar
dlg-export-export-hover = İşle ve yukarıdaki klasöre kaydet (ad otomatik verilir)

## Notices worded at startup (main.rs)

# A notice shown once after starting, when the saved session comes from a newer version of the app. $saved and $handled: version numbers of the session file's format (1, 2…).
app-state-newer = Bu oturum daha yeni bir Fractadyne sürümüyle kaydedilmiş (durum v{ $saved }; bu derleme v{ $handled } sürümünü işleyebiliyor). Bazı ayarlar uygulanmayabilir ve kaydetme, dosyayı bu derlemenin biçiminde yeniden yazar.
# The title of the notice that says the accelerated build's arithmetic libraries are found again (its text comes from elsewhere).
app-notice-mpfr-restored-title = Hızlandırılmış aritmetik yeniden etkin
# After Fractadyne restarted itself because the graphics driver reset the GPU.
app-toast-gpu-recovered = Grafik aygıtı sıfırlamasından kurtarıldı — önceki konum işlenemeyecek kadar ağır olduğundan başlangıç görünümünde yeniden açıldı.

## Short notices (toasts)

# When the autopilot reached a view that could not be rendered and put the last good view back.
app-toast-autopilot-view-restored = Otopilot durdu: görünüm kullanılamaz hâle geldi ve geri alındı (günlüğe kaydedildi)
# $error: the system's reason, as it gives it (often in English).
app-toast-gradients-save-failed = Gradyanlar kaydedilemedi: { $error }
# $error: the reason, as the program gives it (often in English).
app-toast-gradients-serialize-failed = Gradyanlar serileştirilemedi: { $error }
# $name: the name the user gave the gradient.
app-toast-gradient-saved = Gradyan kaydedildi: “{ $name }”.
# $error: the reason, as the program gives it (often in English).
app-toast-bookmarks-serialize-failed = Yer imleri serileştirilemedi: { $error }
# $error: the system's reason, as it gives it (often in English).
app-toast-bookmarks-save-failed = Yer imleri kaydedilemedi: { $error }
# Jumping to a named point given by exact expressions. $name: the point's name (in English).
app-toast-coordinates-failed = { $name }: koordinatlar hesaplanamadı
# $zoom: the magnification, already formatted (1,234.5 or 1.2e148).
app-toast-random-location = Rastgele konum · { $zoom }×
# Importing a Kalles Fraktaler .kfr location. $error: the reason (in English).
app-toast-import-failed = İçe aktarma başarısız: { $error }
# $zoom: the magnification, already formatted.
app-toast-location-loaded = Konum yüklendi @ { $zoom }×

## Go to location: why a jump could not be made (one line per field that failed)

# "Re" is the dialog's real-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-re = Re: { $error }
# "Im" is the dialog's imaginary-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-im = Im: { $error }
# The imaginary field was given a complex value. $error: the evaluator's reason (in English).
app-goto-error-im-real = Im: { $error } — sanal kısmı i olmadan, gerçel bir sayı olarak yazın
# "Zoom" is the dialog's magnification field.
app-goto-error-zoom-empty = Yakınlaştırma: bir büyütme girin, örn. 1e50 ya da 250000
# $zoom: what was typed, in quotes.
app-goto-error-zoom = Yakınlaştırma: { $zoom } okunamıyor — 1e50 ya da 250000 gibi pozitif bir büyütme girin
app-goto-error-invalid = Geçersiz giriş — koordinatları ve yakınlaştırmayı denetleyin.

## Finding a minibrot or Misiurewicz point (the M key and the Go to dialog)

app-feature-needs-holomorphic = Minibrot bulucu holomorf bir aile gerektirir (Mandelbrot / Multibrot).
app-feature-not-in-julia = Minibrot bulucu Mandelbrot kümesinde arar — kullanmak için Julia modunu kapatın.
app-feature-busy = Zaten bir özellik aranıyor — her seferinde yalnızca bir tane.
app-feature-searching = En yakın minibrot aranıyor…
app-feature-mandelbrot-only = Özellik bulma yalnızca Mandelbrot içindir.
app-feature-no-pair = Arama bütçesi içinde Misiurewicz noktası bulunamadı — nokta, bu görünümde bulucunun ulaşabildiğinden daha derinde olabilir ya da merkez bir spirale veya dallanma noktasına yakın olmayabilir.
# The name of what was found, used as $feature in the messages below. $period: the minibrot's period (a whole number).
app-feature-minibrot = { $period } periyotlu minibrot
# $preperiod and $period: the point's two whole numbers (k, p). $octaves: how often the view repeats when zooming in (2 decimals); $twist: the turn per repeat in degrees.
app-feature-misiurewicz-lambda = Misiurewicz ({ $preperiod },{ $period }) — her { $octaves } oktavda bir tekrarlanır, burulma { $twist }°
# $preperiod and $period: the point's two whole numbers (k, p).
app-feature-misiurewicz = Misiurewicz ({ $preperiod },{ $period })
# Added after the messages below when the fixed iteration count is too low for the new depth. $have: the iterations set now; $typical: what the depth typically needs.
app-feature-iter-note = ⚠ Yineleme sayısı { $have } değerine sabitlenmiş; bu derinlik genellikle ~{ $typical } gerektirir — görünüm düz işlenirse yükseltin (ya da otomatik yinelemeyi açın).
# $feature: what was found (see above). $zoom and $asked: magnifications, already formatted; $octaves: the solver's depth limit.
app-feature-zoomed-capped = Yakınlaştırıldı: { $feature } — { $zoom }×. Çözüm { $octaves } oktavla sınırlı olduğundan { $asked }× yerine orada durdu.
# $feature: what was found. $zoom: the magnification, already formatted. $seconds: how long the search took (1 decimal).
app-feature-zoomed = Yakınlaştırıldı: { $feature } — { $zoom }× ({ $seconds } s)
# $feature: what was found. $seconds: how long the search took (1 decimal).
app-feature-snapped = Merkeze oturtuldu: { $feature } ({ $seconds } s)
app-feature-no-minibrot = Görünümün yakınında minibrot merkezi bulunamadı — birine daha çok yaklaşın.
# $feature: the point asked for, e.g. Misiurewicz (3,1). $widths: how many view-widths away, already formatted. $octaves: how far to zoom out.
app-feature-just-outside = Bu görünümün hemen dışında bir nokta bulundu: { $feature } — { $widths } görünüm genişliği uzakta. Ekrana getirmek için yaklaşık { $octaves } oktav uzaklaştırın ya da bu ölçekteki bir özellik için daha büyük bir ön periyot ayarlayın.
# $feature: the point asked for. $widths: how many view-widths away, already formatted (may be 2^187). $octaves: how far to zoom out.
app-feature-far = Bir nokta bulundu: { $feature }; ancak { $widths } görünüm genişliği uzakta — bu özellik bu görünümden çok daha KABA ölçekte. Ona ulaşmak için yaklaşık { $octaves } oktav UZAKLAŞTIRIN ya da bu ölçekteki bir özellik için daha büyük bir ön periyot deneyin.
# $feature: the point asked for. $residual: a small number in scientific notation.
app-feature-not-preperiodic = { $feature } buradaki yörüngeye uymuyor (artık { $residual }) — çifti görünümden algılamak için iki kutuyu da boşaltın.
# $feature: the point asked for. "k/p" are the dialog's two fields.
app-feature-not-converged = Görünümün yakınında yakınsayan bir nokta yok: { $feature } — daha yakına gidin ya da farklı k/p deneyin.
app-feature-bad-request = Ön periyot ve periyot pozitif olmalıdır.

## Share location dialog and location files

app-location-empty = Yüklenecek bir şey yok (ya da metin çok büyük).
app-location-not-fractadyne = Bir Fractadyne konumu değil.
# $zoom: the magnification, already formatted. $note: what in the text was ignored or repaired (in English).
app-share-loaded-note = Yüklendi @ { $zoom }× — { $note }.
# $error: the system's reason (often in English).
app-share-save-failed = Kaydedilemedi: { $error }
app-share-saved = Kaydedildi.
app-share-saved-rendering-thumb = Kaydedildi. Küçük resim işleniyor…
app-share-saved-no-thumb = Kaydedildi — küçük resim olmadan (bu görünüm için işlenemedi).
app-share-saved-with-thumb = Kaydedildi, küçük resimle birlikte.
# $error: the system's reason (often in English).
app-share-saved-thumb-failed = Kaydedildi, ancak küçük resim eklenemedi: { $error }
app-share-loaded-into-box = Kutuya yüklendi — gözden geçirin, ardından Uygula düğmesine basın.
# Reading a file failed (a location or a palette). $error: the system's reason (often in English).
app-read-failed = Okunamadı: { $error }
app-share-too-large = Dosya çok büyük (bir .fdn konumu değil mi?).

## File-dialog filter names (the extensions are added by the code)

app-filter-kfr = Kalles Fraktaler konumu
app-filter-fdn = Fractadyne konumu
app-filter-map = Fractint / KF paleti
app-filter-ugr = Ultra Fractal gradyanı
app-filter-ase = Adobe Swatch Exchange
app-filter-ggr = GIMP gradyanı
app-filter-all = Tüm dosyalar

## Importing and saving palette files (shown under the gradient editor)

# The reason a file was refused before reading it.
app-gradient-map-too-large = bir .map paleti olamayacak kadar büyük
app-gradient-ugr-too-large = bir .ugr palet dosyası olamayacak kadar büyük
app-gradient-ase-too-large = bir .ase renk örneği listesi olamayacak kadar büyük
app-gradient-ggr-too-large = bir .ggr gradyanı olamayacak kadar büyük
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported-vga = { $file } dosyasından { $count } girdi içe aktarıldı - 6 bitlik bir VGA tablosu (beyaz 255 değil 252); Fractint’in kendi çıktısıyla eşleşsin diye yazıldığı gibi korundu.
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported = { $file } dosyasından { $count } girdi keskin bantlar olarak içe aktarıldı.
# $error: the reader's reason (in English).
app-gradient-map-invalid = Bir .map paleti değil: { $error }
# $name: the gradient's name in the file; $file: the file's name.
app-gradient-ugr-imported = { $file } dosyasından içe aktarıldı: “{ $name }”.
# $file: the file's name; $count: how many gradients it holds.
app-gradient-ugr-pick = { $file } dosyasında { $count } gradyan var — aşağıdan birini seçin.
# $error: the reader's reason (in English).
app-gradient-ugr-invalid = Bir .ugr palet dosyası değil: { $error }
# $used: the swatches kept; $total: the swatches in the file.
app-gradient-ase-imported-some = { $total } renk örneğinden { $used } tanesi eşit aralıklarla içe aktarıldı.
# $count: the swatches in the file.
app-gradient-ase-imported = { $count } renk örneği eşit aralıklarla yerleştirilip harmanlanarak içe aktarıldı.
# $error: the reader's reason (in English).
app-gradient-ase-invalid = Bir .ase renk örneği listesi değil: { $error }
app-gradient-nothing-to-save = Henüz kaydedilecek özel gradyan yok.
# $file: the saved file's name.
app-gradient-ggr-saved = Kaydedildi: “{ $file }” — orta noktalar, karışım eğrileri ve renk uzaylarının tümü korundu.
# $file: the saved file's name; $count: the segments whose curve could not be kept.
app-gradient-ggr-saved-lossy =
    { $count ->
        [one] “{ $file }” kaydedildi, ancak { $count } segment .ggr biçiminin ifade edemediği bir Bézier eğrisi kullanıyor. Segment doğrusal bir karışım olarak kaydedildi; kitaplıktaki kopya eğriyi koruyor.
       *[other] “{ $file }” kaydedildi, ancak { $count } segment .ggr biçiminin ifade edemediği Bézier eğrileri kullanıyor. Segmentler doğrusal karışımlar olarak kaydedildi; kitaplıktaki kopya eğrileri koruyor.
    }
# $path: the file; $error: the system's reason (often in English).
app-gradient-write-failed = { $path } yazılamadı: { $error }
# $name: the gradient's name; $count: its segments.
app-gradient-ggr-imported = İçe aktarıldı: “{ $name }” - { $count } segment, orta noktaları, karışım eğrileri ve renk uzaylarıyla birlikte.
# $error: the reader's reason (in English).
app-gradient-ggr-invalid = Bir .ggr gradyanı değil: { $error }

## The eyedropper's banner across the top of the screen while picking a colour

# $hex: the colour under the pointer, as #rrggbb.
app-eyedropper-banner-color = Renk seçiliyor  ·  { $hex }  ·  almak için tıklayın, iptal için Esc
app-eyedropper-banner = Renk seçiliyor  ·  almak için tıklayın, iptal için Esc

## The gradient editor window

app-gradient-title = Gradyan düzenleyici
# Before the two view buttons, Bar and Ring.
app-gradient-view = Görünüm
app-gradient-view-bar = Çubuk
app-gradient-view-bar-hover = Düz bir şerit — durakları hassas biçimde yerleştirmek için en iyisi
app-gradient-view-ring = Halka
app-gradient-view-ring-hover = Bir halka, çünkü palet DÖNGÜSELDİR: tepedeki birleşim yeri, işleyicinin her döngüde üzerinden geçtiği dikiştir ve çubukta iyi görünen bir gradyanın orada sert bir kenarı olabilir.
app-gradient-add-lane-hover = Oraya bir durak eklemek için bu çizginin herhangi bir yerine tıklayın. + düğmesi en geniş boşluğa bir durak ekler.
app-gradient-strip-hover = Durağını taşımak için bir işareti sürükleyin · tüm gradyanı DÖNDÜRMEK için şeridin kendisini sürükleyin, dikişte başa sarar · durak eklemek için çift tıklayın, kaldırmak için bir işarete sağ tıklayın · ok tuşları seçimi kaydırır (ince ayar için Shift).
app-gradient-remove-selected-hover = Seçili durağı kaldır
app-gradient-ring-hover = Tüm gradyanı DÖNDÜRMEK için halkanın kendisini sürükleyin — dikişi geçen kısım diğer taraftan geri gelir · durağını taşımak için bir işareti sürükleyin · oraya durak eklemek için dış halkaya, en geniş boşluğa bir durak eklemek için ortadaki ⊕ simgesine tıklayın · ⊖ seçili durağı kaldırır. Tepedeki içi boş işaret dikiştir, döngüsel bir paletin başa sardığı yer; birleşim yerinin TA KENDİSİ olduğu için yerinde kalır ve renkler onun altında döner.
app-gradient-empty = Henüz özel gradyan yok — “Hazır ayarı kopyala…”, bir içe aktarma ya da aşağıdaki kayıtlı bir gradyan bir tane başlatır; durakları ve segment başına eğrileri burada görünür.
# $count: the entries of an imported palette, more than the editor shows stop by stop.
app-gradient-too-many = { $count } içe aktarılmış girdi - durak durak düzenlemek için çok fazla. Yukarıdaki önizlemeyi kullanın ya da baştan başlamak için Hazır ayarı kopyala menüsünü kullanın.
# The selected segment's heading. $index: its number; $total: how many there are; $left and $right: where it starts and ends, 0.000–1.000.
app-gradient-segment-heading = Segment { $index } / { $total }  ·  { $left } → { $right }
# A label before the list of blend curves.
app-gradient-curve = Eğri
app-gradient-curve-hover = Rengin bu segment boyunca ne kadar hızlı ilerlediği. “Kavisli” orta noktaya göre şekillenir — orta nokta 0.50 iken tam olarak doğrusaldır.
# The blend curves (GIMP's kinds), in a narrow list: keep them short.
app-gradient-blend-linear = Doğrusal
app-gradient-blend-curved = Kavisli
app-gradient-blend-sine = Sinüs
app-gradient-blend-sphere-increasing = Küre ↑
app-gradient-blend-sphere-decreasing = Küre ↓
app-gradient-blend-bezier = Bézier
# Hover on Bézier in the list of blend curves.
app-gradient-bezier-exact-hover = Her iki yönde de sürükleyebileceğiniz iki kontrol noktası. Bu eğri zaten kübik olduğundan dönüştürmek hiçbir şeyi değiştirmez.
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-visible-hover = Her iki yönde de sürükleyebileceğiniz iki kontrol noktası. Dönüştürme mevcut eğriye YAKLAŞIK bir eğri üretir — bu eğri için sapma { $error } içinde kalır ve bu açıkça görülür.
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-barely-hover = Her iki yönde de sürükleyebileceğiniz iki kontrol noktası. Dönüştürme mevcut eğriye YAKLAŞIK bir eğri üretir — bu eğri için sapma { $error } içinde kalır ve bu güçlükle fark edilir.
# A label before the list of colour spaces (RGB, HSV).
app-gradient-space = Renk uzayı
# A label before the Bézier presets beside it.
app-gradient-ease = Yumuşatma
app-gradient-ease-hover = Hazır ayarlar iki kontrol noktasını yazar; başka her şey için onları eğri üzerinde sürükleyin. Bir Bézier eğrisinin orta noktası yoktur — yarıya nerede ulaştığını tutamakları belirler.
# Bézier presets, as in CSS easing: short buttons.
app-gradient-ease-linear = Doğrusal
app-gradient-ease-in = Giriş
app-gradient-ease-out = Çıkış
app-gradient-ease-in-out = Giriş-çıkış
app-gradient-midpoint = Orta nokta
app-gradient-midpoint-hover = Karışımın yarıya ulaştığı yer, bu segmentin bir kesri olarak. 0.50 ortadır.
# A button that puts the midpoint back in the middle of the segment.
app-gradient-centre = Ortala
app-gradient-centre-hover = 0.50 değerine dön
app-gradient-no-hue = ⚠ bu segmentin bir ucunun tonu yok
app-gradient-no-hue-hover = Bir uç siyah, beyaz ya da gri; bu yüzden geçiş kırmızıdan başlar ve tüm renk çemberini dolaşır. Bu geçerlidir ve bazen istenir — ama ortanın iki uçta da bulunmayan bir renk olabilmesinin nedeni budur. Eğrinin altındaki şerit ne yaptığını gösterir.
app-gradient-segment-start-hover = Bu segmentin BAŞINDAKİ renk
app-gradient-segment-end-hover = Bu segmentin SONUNDAKİ renk
app-gradient-bytes-hover = 0–255 olarak kırmızı, yeşil ve mavi
app-gradient-floats-hover = Aynı renk 0–1 olarak; işleyicinin sakladığı biçim budur
app-gradient-canvas-bezier-hover = Seçili segmentin karışım eğrisi. Yeniden şekillendirmek için kontrol noktalarından birini — x VE y yönünde — sürükleyin. Soluk çizgiler 0 ve 1’dir; eğri bunların dışına çıkabilir, sınırlanan eğri değil renktir.
app-gradient-canvas-hover = Seçili segmentin karışım eğrisi: rengin soldaki duraktan sağdakine ne kadar hızlı ilerlediği. Orta noktayı taşımak için halkayı sürükleyin ya da her iki yönde hareket eden bir kontrol noktası için eğriyi Bézier’e çevirin.
# The selected stop's heading. $index: its number; $total: how many there are.
app-gradient-stop-heading = Durak { $index } / { $total }
app-gradient-stop-color-hover = Bu duraktaki renk — seçmek için renk örneğine tıklayın ya da yanına onaltılık bir değer yazın
app-gradient-stop-bytes-hover = 0–255 olarak kırmızı, yeşil ve mavi — bu rengin yazıldığı baytlar
# The eyedropper button.
app-gradient-pick = Seç
app-gradient-pick-hover = Damlalık — ekranın herhangi bir yerinden renk alın. İşaretçiyi istediğiniz renge götürüp tıklayın; Esc iptal eder.
# Written before the stop's position (0.000–1.000) in its number field: a short abbreviation of "position".
app-gradient-stop-pos = kon
app-gradient-nudge-hover = Bu durağı kaydır (daha ince adım için Shift’i basılı tutun)
app-gradient-remove = Kaldır
app-gradient-remove-hover = Bu durağı kaldır (ya da altındaki ⊖ simgesine tıklayın)
app-gradient-add-stop = Durak ekle
app-gradient-add-stop-hover = En geniş segmenti böl (ya da gradyanın üstündeki çizgiye tıklayın)
# A menu of the built-in palettes, to start from a copy of one.
app-gradient-copy-preset = Hazır ayarı kopyala…
app-gradient-paste = Yapıştır…
app-gradient-paste-hover = Onaltılık renklerden ya da 0–255 RGB üçlülerinden bir palet içe aktar
app-gradient-save-ggr = .ggr kaydet…
app-gradient-save-ggr-hover = Bu gradyanı bir GIMP .ggr dosyasına yaz — GIMP, Krita ve Inkscape ile bu düzenleyici tarafından okunabilir
# A menu of the palette file formats (.map, .ugr, .ggr, .ase).
app-gradient-import = İçe aktar
app-gradient-import-map-hover = Bir Fractint / Kalles Fraktaler .map palet dosyası yükle (her girdi için bir satır R G B)
app-gradient-import-ugr-hover = Bir Ultra Fractal .ugr gradyan dosyası yükle (bir .ugr genellikle birçok gradyan içerir — birini siz seçersiniz)
app-gradient-import-ggr-hover = Bir GIMP .ggr gradyanını orta noktaları, karışım eğrileri ve renk uzayları eksiksiz olarak yükle
app-gradient-import-ase-hover = Bir Adobe renk örneği listesi (.ase) yükle. Renk örneklerinin konumu olmadığından eşit aralıklı ve harmanlanmış olarak gelirler.
# The gradient's name, for the library of saved gradients.
app-gradient-name = Ad
# Shown greyed in the empty name field.
app-gradient-name-hint = gradyanım
# The save button, when a gradient of that name is already saved.
app-gradient-update = Güncelle
app-gradient-save = Kaydet
app-gradient-save-hover = Bu gradyanı eğrileri ve renk uzaylarıyla eksiksiz olarak kitaplığınıza kaydet
app-gradient-save-needs-name-hover = Önce gradyana bir ad verin
# A menu of the saved gradients. $count: how many there are.
app-gradient-saved-menu = Kayıtlı ({ $count })
app-gradient-nothing-saved = Henüz kayıtlı bir şey yok
app-gradient-delete-saved-hover = Bu kayıtlı gradyanı sil
# $count: the gradient's segments.
app-gradient-rich = Zengin gradyan: { $count } segment orta nokta, karışım eğrisi ya da renk uzayı taşıyor — düz bir durak listesinin tutabileceğinden fazlası.
app-gradient-convert = Düzenlenebilir duraklara dönüştür
app-gradient-convert-hover = Onu düz bir durak listesiyle değiştirir. Bu işlem, bir durak listesinin tutamayacağı orta noktaları, karışım eğrilerini ve ton geçişlerini ATAR.
# Above the list of gradients in an Ultra Fractal file. $count: how many; $file: the file's name.
app-gradient-ugr-list = { $file } dosyasında { $count } gradyan
app-gradient-close-list = Listeyi kapat
app-gradient-seamless = Kesintisiz (son, başlangıçla eşleşir)
app-gradient-seamless-hover = Palet döngüseldir, bu yüzden 1 konumu ile 0 konumu ekranda yan yanadır. Bu açıkken son durağın rengi ilkine eşit tutulur — halka görünümü birleşim yerini tepede gösterir.
app-gradient-ends-differ = ⚠ uçlar farklı
app-gradient-ends-differ-hover = Gradyanın sonundaki renk başındaki renkle aynı değil; bu yüzden döngüsel bir palet başa sardığı yerde sert bir kenar gösterir.
app-gradient-hard-bands = Keskin bantlar (girdiler arasında karışım yok)
app-gradient-hard-bands-hover = Fractint / .map anlamı: her girdi düz bir renktir. Kapalıyken girdiler yumuşak bir gradyan olarak harmanlanır.
app-gradient-paste-intro = Onaltılık renkleri (#ff8800) ya da 0–255 üçlülerini (255 136 0) virgül, boşluk ya da yeni satırla ayırarak yapıştırın:
# Applies the pasted colours.
app-gradient-apply = Uygula
# Empties the paste box.
app-gradient-clear = Temizle
# $used: the colours kept; $total: the colours pasted; $limit: the most that are kept.
app-gradient-paste-imported-some = { $total } renkten { $used } tanesi, listeniz boyunca eşit aralıklarla örneklenerek içe aktarıldı (sınır: { $limit }).
# $count: the colours pasted.
app-gradient-paste-imported = { $count } renk içe aktarıldı.
# $error: why the pasted text could not be read (in English).
app-gradient-paste-unreadable = Okunamadı: { $error }
# A hint at the bottom of the editor. $count: the palette's entries.
app-gradient-bands-hint = { $count } bant · dosyadaki sırayla alınır; bantlar açıkken konumlar yok sayılır.
# A hint at the bottom of the editor, in the ring view. $count: the gradient's stops.
app-gradient-ring-hint = { $count } durak · oraya durak eklemek için dış halkaya tıklayın · ⊕ en geniş boşluğa ekler, ⊖ seçili durağı kaldırır · taşımak için bir işareti sürükleyin · tepedeki işaret dikiştir
# A hint at the bottom of the editor, in the bar view. $count: the gradient's stops.
app-gradient-bar-hint = { $count } durak · oraya durak eklemek için gradyanın üstündeki çizgiye tıklayın · taşımak için bir işareti sürükleyin · ⊖ ya da sağ tık kaldırır
app-gradient-cancel = İptal
app-gradient-cancel-hover = Bu pencere açıldığından beri yapılan her değişikliği at ve pencereyi kapat
app-gradient-ok = Tamam
app-gradient-ok-hover = Bu değişiklikleri koru ve düzenleyiciyi kapat

## Shared

# The button that closes the Help window and the "Faster deep zoom" window.
app-close = Kapat

## The "Faster deep zoom" window (Help menu): the optional accelerated build

app-accel-title = Daha hızlı derin yakınlaştırma
app-accel-active-heading = Hızlandırılmış derlemeyi çalıştırıyorsunuz
app-accel-active-body = Derin yakınlaştırma referans yörüngeleri MPFR/GMP ile hesaplanıyor; bu, o adımda standart derlemeden 2,5-6,4 kat daha hızlıdır - derin bir görünüm çözümlenmeye başlamadan önceki duraklama.
# $status: which arithmetic library is in use and its version (in English).
app-accel-in-use = Kullanılan aritmetik: { $status }
app-accel-fallback-heading = Hızlandırılmış derleme — yerleşik aritmetik kullanılıyor
app-accel-fallback-body = Bu, hızlandırılmış (MPFR) derleme, ancak matematik kitaplıkları bulunamadığı için yerleşik aritmetiği (astro-float) kullanıyor. Görüntüler aynıdır; yalnızca derin bir görünüm çözümlenmeden önceki duraklama daha yavaştır.
# Followed by the folder the program runs from, on its own line.
app-accel-fallback-dlls = Daha hızlı yolu kullanmak için libmpfr-6.dll ve libgmp-10.dll dosyalarını (ve libgcc_s_seh-1.dll, libwinpthread-1.dll dosyalarını) fractadyne.exe dosyasının yanına koyun, ardından yeniden başlatın:
app-accel-warning-hidden = Bununla ilgili başlangıç uyarısı gizlendi.
app-accel-show-again = Yeniden göster
app-accel-offer-heading = İsteğe bağlı bir derleme, derinde 2,5-6,4 kat daha hızlı
app-accel-offer-body = Derin yakınlaştırma, zamanının büyük kısmını CPU’da referans yörüngeleri hesaplayarak geçirir - derin bir görünüm çözümlenmeye başlamadan önceki duraklama. İsteğe bağlı bir derleme bunu MPFR/GMP kitaplıklarıyla yapar; bu 2,5 ila 6,4 kat daha hızlıdır ve ne kadar derine inerseniz fark o kadar artar. Geri kalan her şey aynıdır.
app-accel-offer-identical = Görüntüler BAYT BAYT AYNIDIR - daha hızlı bir kitaplıkla hesaplanan aynı matematiktir; her fraktal formülünde ve tüm derin yakınlaştırma karşılaştırma derleminde denetlenmiştir.
app-accel-offer-shared = Ayarlarınız, kaydedilmiş oturumunuz ve konumlarınız ikisi arasında paylaşılır; böylece serbestçe geçiş yapabilirsiniz ve hiçbir şeyi içe aktarmanız gerekmez.
# Shown on Windows.
app-accel-separate-windows = Ayrı bir indirmedir, çünkü kullandığı kitaplıklar standart Windows derlemesinin kullandığı derleyiciyle derlenemez ve Fractadyne’ın kendi lisansından farklı bir lisans (GNU LGPL v3) taşırlar.
# Shown on other systems.
app-accel-separate = Ayrı bir indirmedir, çünkü kullandığı kitaplıklar Fractadyne’ın kendi lisansından farklı bir lisans (GNU LGPL v3) taşır.
app-accel-download = Bu sürüm için indir
app-accel-all-releases = Tüm sürümler
app-accel-all-releases-hover = Doğrudan bağlantı 404 verirse bu sürümün henüz hızlandırılmış derlemesi yoktur - buradan en yakınını seçin.
# Shown on Windows.
app-accel-install-windows = Arşivi açın ve fractadyne.exe dosyasını o klasörden çalıştırın; .dll dosyalarını yanında tutun.
# Shown on Linux.
app-accel-install-linux = Arşivi açın ve ./fractadyne komutunu o klasörden çalıştırın. Sisteminizin GMP ve MPFR kitaplıklarını (libgmp10 / libmpfr6) kullanır; bunlar neredeyse her masaüstünde zaten bulunur - paketteki README-ACCELERATED.txt dosyasına bakın.
# Shown on other systems.
app-accel-install-other = Bu platform için henüz hızlandırılmış paket derlenmiyor; --features fractadyne-core/rug ile kaynak koddan derlenebilir.

## The Help window's frame (its sections are under help-)

app-help-title = Fractadyne Yardımı

## Checking for updates

app-toast-up-to-date = En son sürümü kullanıyorsunuz.
# $error: why the check failed (in English).
app-toast-update-failed = Güncelleme denetimi başarısız: { $error }
# The window that offers a newer version.
app-update-title = Güncelleme mevcut
# $version: the new version (0.3.0-beta.20). $channel: the token "stable" or "beta", in English; a translation can choose its words by it ({ $channel -> [beta] … *[other] … }).
app-update-version =
    { $channel ->
        [beta] Fractadyne { $version } (beta) mevcut
       *[other] Fractadyne { $version } (kararlı) mevcut
    }
# $current: the version running now. $track: the update track the user follows (its name in the language in use).
app-update-running = Çalışan sürüm: { $current }  ·  { $track }
app-update-download-intro = En son sürümü GitHub’dan indirin:
app-update-download = GitHub’dan indir
app-update-later = Daha sonra hatırlat

## The second graphics card (shown in its window, after "In use:" or "Could not start:")

# The second device is the same card as the window's. $name: the card's name.
app-worker-same-card = aynı kart, { $name }
# Why the second device could not start.
app-worker-thread-stopped = aygıt iş parçacığı durdu

## Export status (shown in the Export image window)

app-export-gpu-unavailable = GPU kullanılamıyor
app-export-reference-aborted = Dışa aktarma başarısız: referans oluşturma yarıda kesildi.

## The performance panel (View ▸ Performance panel)
## Its row labels sit in a monospace column about 14 characters wide: keep them short (a longer label pushes its value to the right).

app-perf-fps = FPS
app-perf-fps-hover = Kareler arasındaki yumuşatılmış aralıktan hesaplanan, saniyede sunulan kare sayısı. Bir görünüm durulurken bunun yerine iyileştirme ilerlemesini bildirir - durulmakta olan bir görünüm, buradaki bir sayının düşündüreceği hızda çalışmıyordur.
# What the FPS row shows while a view is settling. $done and $total: tiles; $eta: the time left, as m:ss.
app-perf-refining-tiles = iyileştiriliyor { $done }/{ $total } (~{ $eta })
app-perf-refining-arming = iyileştiriliyor (hazırlanıyor)
# $percent: how much is done, with one decimal.
app-perf-refining-percent = iyileştiriliyor %{ $percent }
app-perf-building-reference = referans oluşturuluyor
app-perf-idle = boşta
app-perf-frame = kare
app-perf-frame-hover = Kareler arasındaki yumuşatılmış gerçek süre - altındaki satırların toplamı değildir. Sunum için beklemeyi de içerir; bu yüzden durulmuş, boşta bir görünümde herhangi bir işi değil yeniden çizim temposunu yansıtır.
app-perf-cpu = cpu
app-perf-cpu-hover = Bir güncelleme içindeki ana iş parçacığı süresi: parametre hazırlığı ve referans önbelleği yönetimi. Bir referans yeniden oluşturulduğunda sıçrayan satır budur ve buradaki bir sıçrama, hissettiğiniz takılmadır.
app-perf-gpu-idle = gpu/boşta
app-perf-gpu-idle-hover = Kare süresi eksi ana iş parçacığı CPU süresi. Görünüm işlenirken bu GPU işidir, ancak durulmuş bir görünümde çoğunlukla boşta beklemedir - ad ikisini birden söyler, çünkü panel yalnızca bu çıkarmayla ikisini ayırt edemez.
app-perf-timing = zamanlama
# How frame cost is measured, the timing row's value. TIMESTAMP_QUERY is a name.
app-perf-timing-starved = gerçek saat (zaman damgası gelmiyor)
app-perf-timing-gpu = GPU zaman damgaları
app-perf-timing-no-query = gerçek saat (TIMESTAMP_QUERY yok)
app-perf-timing-hover = Kare maliyetinin nasıl ölçüldüğü. GPU zaman damgaları kesin kaynaktır; zamanlamalar gelmez olduğunda gerçek saat yedeği devreye girer ve gerçek saate göre fiyatlanan bir bütçe derin kareleri defalarca yanlış boyutlandırmıştır — buradaki kırmızıyı “güvenlik payları şu an birer tahmin” diye okuyun.
app-perf-mode = mod
app-perf-mode-hover = Son karenin işlendiği aritmetik yol: doğrudan (referans yörüngesiz), df32 pertürbasyon ya da floatexp - yaklaşık 1e28x ötesinde kullanılan genişletilmiş aralıklı yol ve BLA’nın çalıştığı tek yol.
app-perf-eff-iter = etkin yin.
app-perf-eff-iter-hover = Son karenin, derinlik ölçeklemesinden ve hareket sırasında uygulanan olası bir sınırdan sonra fiilen kullandığı yineleme sayısı - yalnızca başlangıç noktası olan Kalite’deki temel ayar değil.
app-perf-precision = hassasiyet
app-perf-precision-hover = Referans yörüngenin hesaplandığı mantis bit sayısı. Derinlikle artar; 64 bit sığ derinliklerin tabanıdır, ulaştığınız bir sınır değil.
app-perf-orbit-len = yörünge boyu
# $cap: the most orbit samples this GPU holds, already formatted, or the text of app-perf-uncapped.
app-perf-orbit-len-hover = Referans yörünge örnekleri. Bu GPU en fazla { $cap } tutar — buna yaklaşmak pratikteki derinlik duvarıdır: onu geçince canlı görünüm, daha uzun yörünge gerektiren daha derin konumları çözümleyemez.
app-perf-uncapped = (sınırsız)
app-perf-sa-skip = SA atlama
app-perf-sa-skip-hover = Piksel başına iş başlamadan önce seri yaklaşımının atladığı yineleme sayısı. Yüksek olması daha hızlıdır ve görüntüde hiçbir şeyi değiştirmez; hiç atlamadığında satır gösterilmez.
app-perf-aa = aa
app-perf-aa-hover = Süper örnekleme: her piksel eksen başına bu kadar örnekten işlenir; yani 2x dört kat iş demektir.
app-perf-dual = ikili
app-perf-dual-hover = Bölünmüş Mandelbrot/Julia görünümünün çalışıp çalışmadığı. Açıkken bu paneldeki her maliyet İKİ bölmeyi de kapsar.
app-perf-zoom = yakınlaştırma
app-perf-zoom-hover = Tam görünüme göre büyütme. Yukarıdaki aritmetik modu yaklaşık 1e4x düzeyinde ve yeniden yaklaşık 1e28x düzeyinde değişir.
app-perf-iterate = yineleme
# $target and $lethal: times in milliseconds, already formatted.
app-perf-iterate-hover = Son yineleme gönderiminin ölçülen GPU süresi. Denetleyici { $target } ms hedefler; { $lethal } ms civarında süregelen okumalar, bu donanım sınıfının aygıtı kaybettiği banttır.
app-perf-steps = adım/s
app-perf-steps-hover = GPU’da saniyedeki yineleme adımı, milyar cinsinden. İç bölgenin ağır bastığı görünümler, sayının tek başına düşündürdüğünden adım başına daha yavaş çalışır; bu yüzden bu bir puan değil, bir verim okumasıdır.
app-perf-budget = bütçe
# After the budget's value while it is still converging; the hover below quotes it.
app-perf-budget-settling = (duruluyor)
app-perf-budget-hover = Gönderim başına uyarlanan adım bütçesi; denetleyicinin, sürücünün GPU’yu sıfırlayacağı kadar uzun bir kareye karşı koruması. “duruluyor”, bütçenin seyrek okumalardan hâlâ yakınsadığı ve henüz güvenilir bir tavan olmadığı anlamına gelir.
app-perf-rss = rss
# $mb and $peak: memory in megabytes, already formatted.
app-perf-rss-value = { $mb } MB (en yüksek { $peak })
app-perf-rss-hover = Sürecin yerleşik belleği ve bu oturumda ulaştığı en yüksek değer. Derin referans oluşturmalar sahada, ekrandaki başka hiçbir şey bunu belirtmeden 2 GB’ın üstüne çıkmıştır; sistem RAM’inin dörtte üçünü geçince kehribar rengine döner.
app-perf-gpu-est = gpu tahm.
app-perf-gpu-est-hover = GPU’da yerleşik belleğin tahmini: referans yörüngeleri, BLA ağaçları, yineleme dokuları, bir iyileştirme sürerken parça durumunun ping-pong arabellekleri ve sunum kapısının anlık görüntüsü. Bilinen ayırma boyutlarından derlenir; sürücünün gerçek rakamı taşınabilir biçimde okunamaz.
app-perf-julia-re = julia c.re
app-perf-julia-re-hover = Julia bölmesinin işlediği c parametresinin gerçel kısmı - sabitlenmemişse imlecin üzerinde durduğu nokta.
app-perf-julia-im = julia c.im
app-perf-julia-im-hover = Julia bölmesinin işlediği c parametresinin sanal kısmı.
app-perf-julia-c = julia c
# The julia c row's value when the parameter is held fixed.
app-perf-pinned = sabitlendi
app-perf-pinned-hover = Parametre sabit bir noktada tutuluyor; bu yüzden imleci hareket ettirmek artık Julia bölmesini değiştirmez.
app-perf-c-panel = c/panel
app-perf-c-panel-hover = Tüm Mandelbrot panelinin kapsadığı c uzayı genişliği. Bu, bir Julia pikselinin çok altındaysa üzerine gelmek c’yi değiştirir ama Julia değişmemiş görünür — derin yakınlaştırmada beklenen bir durumdur, donma değildir.
app-perf-ref-recompute = ref hesaplama
app-perf-ref-recompute-hover = En son referans yörünge yeniden hesaplamasının süresi. Derin bir görünüm çözümlenmeye başlamadan önceki duraklama budur ve isteğe bağlı hızlandırılmış derlemenin hızlandırdığı adım da budur.
app-perf-recompute-rate = hesaplama/s
app-perf-recompute-rate-hover = Saniyede KURULAN referans yörüngeleri - sonucu fiilen kullanılanlar. Alttaki satırla karşılaştırın: aradaki fark, atılan iştir.
app-perf-builds-rate = ref oluşt./s
app-perf-builds-rate-hover = Saniyede BAŞLATILAN referans oluşturmaları, sonucu atılanlar da dahil - dalışın hiç ulaşmadığı bir ileri bakış yuvası ya da reddedilen bir uzatma. Bunlar tam CPU harcar ve başka hiçbir yerde görünmez: bir keresinde bir ön getirme döngüsü, hesaplama/s 2 gösterirken altı dakika boyunca saniyede yaklaşık 400 oluşturma çalıştırdı. Tek haneli değerler normaldir; bu iki satır arasındaki büyük bir fark, işin boşa atıldığı anlamına gelir.
app-perf-recompute-total = hesaplama top.
app-perf-recompute-total-hover = Açılıştan bu yana yapılan referans yeniden hesaplamaları. Bilinen bir aralıktaki oran olarak yararlıdır; hiçbir şey hareket etmezken tırmanan bir sayı, peşine düşmeye değer bir işarettir.

## Help: the contents list (Help & reference…, F1)

# Each section's name in the contents list (about 165 px wide: keep them short), and its heading.
help-section-overview = Genel bakış
help-section-navigation = Gezinme
help-section-expressions = Koordinat ifadeleri
help-section-options = Renklendirme ve seçenekler
help-section-fractals = Fraktallar
help-section-how-it-works = Nasıl çalışır
help-section-command-line = Komut satırı
help-section-shortcuts = Kısayollar
help-section-hardware = Önerilen donanım
# Credits to the prior work Fractadyne builds on.
help-section-acknowledgments = Teşekkürler
# The third-party license notices.
help-section-licenses = Lisanslar
help-section-about = Hakkında

## Help ▸ Overview

help-overview-intro = Fractadyne, ultra derin yakınlaştırma ve hız için geliştirilmiş yerel bir fraktal gezginidir. “Kaçış zamanlı” fraktalları çizer — her pikselde tek bir basit formülün tekrarlanmasıyla oluşturulan görüntüler.
help-overview-what-is = Kaçış zamanlı fraktal nedir?
help-overview-what-is-text = Program her piksel için z → z² + c gibi bir formülü sıfırdan başlayarak tekrar tekrar çalıştırır. Değer sonsuza dek küçük kalırsa piksel kümeye aittir ve koyu çizilir. Sonunda sınırsızca büyürse (“kaçarsa”) piksel kümenin dışındadır ve rengi bunun kaç adım sürdüğünü kaydeder. “Kalır” ile “kaçar” arasındaki sonsuz ayrıntılı sınır, fraktalın ta kendisidir.
help-overview-can-do = Neler yapabilirsiniz
help-overview-can-pan = Neredeyse sınırsız kaydırın ve yakınlaştırın (konum her derinlikte kesindir).
help-overview-can-families = 25 yerleşik fraktal ailesi (on aile ve bunlardan beşinin yüksek kuvvetleri), kendi formülleriniz, Life ve L-sistemleri arasında geçiş yapın; Newton dışındaki her kaçış zamanlı aileyi bir Julia kümesi olarak görüntüleyin.
help-overview-can-recolor = Hazır ya da özel gradyanlar ve çeşitli renklendirme yöntemleriyle yeniden renklendirin.
help-overview-can-lighting = 3B kabartma aydınlatması ve parlayan sınır konturları ekleyin.
help-overview-can-minibrots = Minibrotlara oturun, yerleri yer imlerine ekleyin ve yüksek çözünürlüklü görüntüler dışa aktarın.
help-overview-can-autopilot = Otomatik yakınlaştırma otopilotunun kendi başına ayrıntıya doğru dalmasına izin verin.
help-overview-can-share = Herhangi bir yeri küçük bir “.fdn” metin parçası ya da dosyası olarak paylaşın.
help-overview-can-tours = Betikli turlar ve bir donanım kıyaslaması çalıştırın.
help-overview-first-steps = İlk adımlar
# "Seahorse Valley" is a place in the Navigate menu: use its name as that menu shows it. A and F1 are keys.
help-overview-first-steps-text = Gezinme menüsünü açıp “Seahorse Valley” konumunu seçin, ardından yakınlaştırmak için fare tekerleğini çevirin. Kaydırmak için sürükleyin. Ya da yalnızca A tuşuna basın ve otopilotun sizin için dalmasına izin verin. Bu yardıma dönmek için istediğiniz zaman F1 tuşuna basın.

## Help ▸ Coordinate expressions
## The expressions, numbers and function names here are typed as shown in every language:
## keep them exactly as written. Menu items in quotation marks are the menus' own labels.

help-expr-intro = Fractadyne’ın koordinat aldığı her yerde ondalık sayı yerine bir ifade yazabilirsiniz: Gezinme → “Konuma git…” penceresindeki Re / Im alanları (ve kutupsal modunun her alanı), bir .fdn konumunun center_re / center_im satırları, bir tur betiğinin re / im değerleri ve komut satırındaki --center. İfade, yakınlaştırmanın gerektirdiği kadar basamakla hesaplanır ve Konuma git penceresi onu görünümle birlikte tutar — böylece 1/3 ya da -0.5 + 0.25*cos(pi/4) olarak girdiğiniz bir nokta, ne kadar derine yakınlaştırırsanız yakınlaştırın tam o noktada kalır; ondalık bir sayı ise basamakları tükenince kayardı.
help-expr-examples = Örnekler
help-expr-ex-third = Tam üçte bir. Yakınlaştırdıkça daha fazla basamak görünür.
help-expr-ex-neck = Seahorse Valley’nin boynu — 2 periyotlu tomurcuğun kökü.
help-expr-ex-circle = -0.5 çevresindeki 1/4 yarıçaplı çember üzerinde bir nokta (kutupsal mod bunu sizin için oluşturur).
help-expr-ex-bulb = 3 periyotlu tomurcuk kökünün Re değeri; Im değeri: sin(2*pi/3)/2 - sin(4*pi/3)/4. Gezinme → “Tam noktalar (ifadeler)” bunları hazır içerir.
help-expr-ex-complex = Re alanına yazılan bütün bir karmaşık değer hem Re hem Im alanını doldurur.
help-expr-ex-combine = Fonksiyonlar ve sabitler serbestçe birleştirilebilir.
help-expr-ex-powers = Kuvvetler: 2^-20, 1/1048576 demektir. -2^2 ise -4’tür — eksi işareti kuvvetten sonra uygulanır.
help-expr-ex-root = -2, gerçel küp kök. Negatif bir sayının çift dereceli kökleri reddedilir; sqrt(-1) i verir.
help-expr-ex-scientific = Bilimsel gösterim ifadelerin içinde de çalışır.
help-expr-numbers = Sayılar ve işleçler
# The decimal point in an expression is '.' in every language: keep that rule as it is.
help-expr-numbers-decimals = İsteğe bağlı üslü ondalık sayılar: 0.25, -1.5e-30, .5. Yalnızca rakamlar — bir sayının içinde boşluk ya da basamak gruplama virgülü olamaz ve ondalık ayırıcı her zaman “.” işaretidir.
help-expr-numbers-imaginary = Sanal bir sayı için i son ekini ekleyin (0.25i) ya da i’yi tek başına kullanın: 3 + 4*i.
# Starts with the operators themselves, two spaces apart.
help-expr-numbers-operators = +  -  *  /  ve ^ (kuvvet; sağdan sola hesaplanır, yani 2^3^2 = 2^9). Parantezler gruplar. Örtük çarpma yoktur: 2pi değil, 2*pi yazın.
help-expr-numbers-case = Adlarda büyük/küçük harf ayrımı yoktur: PI, Pi ve pi aynı sabittir.
help-expr-constants = Sabitler
help-expr-functions = Fonksiyonlar
help-expr-functions-angles = Açılar radyan cinsindendir. Derece için pi/180 ile çarpın — cos(45*pi/180) — ya da Konuma git penceresini kutupsal moda geçirip derece birimini seçin.
help-expr-refused = Neler reddedilir ve neden
help-expr-refused-fractional = Negatif bir sayının kesirli kuvveti, örneğin (-2)^(1/2): sonuç karmaşıktır ve seçilecek birkaç kök vardır. Gerçel bir kök için root(x, n) ya da cbrt(x), asıl karmaşık kök için sqrt(x) kullanın.
help-expr-refused-complex = Gerçel değerli bir fonksiyona karmaşık bağımsız değişken (sin(i), ln(1+i)). Yalnızca sqrt ve abs karmaşık bağımsız değişken kabul eder; aritmetik işleçler ve tam sayı kuvvetler karmaşık değerlerle çalışır.
help-expr-refused-domain = Tanım kümesi hataları: pozitif olmayan bir sayının ln ya da log değeri, -1…1 dışında asin ya da acos, 0’ın negatif kuvveti, sıfıra bölme.
help-expr-refused-bounds = sin, cos, tan, exp ya da ^ için ±2^32’yi aşan bağımsız değişkenler ve 32 düzeyden derin iç içe yerleştirme: yapıştırılan bir koordinat güvenilmeyen bir girdidir ve bunlar sınırsız zaman alabilir.
help-expr-refused-grammar = Değişken, kullanıcı tanımlı fonksiyon, karşılaştırma ya da koşul yoktur. Dilbilgisi bilerek küçük tutulur; böylece kaydedilmiş bir konum her sürümde aynı anlama gelir.
help-expr-no-parse = Ayrıştırılamazsa
help-expr-no-parse-text = İleti alanın adını verir, ne bulunduğunu ve hangi karakterde olduğunu söyler ve — sık yapılan hatalarda — bunun yerine ne yazılacağını belirtir: bir belgeden yapıştırılmış tipografik eksi işareti (−), × ya da ÷, ondalık virgül, bir sayı ile bir ad arasında eksik *, kapanmamış bir parantez ya da parantezleri olmadan yazılmış bir fonksiyon. Hiçbir şey asla yarım okunmaz: metnin herhangi bir kısmı anlaşılamazsa, sessizce kısaltılmak yerine koordinatın tamamı reddedilir.

## Help ▸ Navigation
## Menu paths (Navigate → "Go to location…") and quoted items name the interface's own
## menus and controls: use their translations. Key names (Ctrl, Shift, Space, Esc) stay.

# A heading: the mouse controls (also in Shortcuts).
help-mouse = Fare
# The left column of a controls table: a gesture or key. Keep key names (Shift, Space, Ctrl) as they are.
help-key-left-drag = Sol sürükleme
help-nav-left-drag = Görünümü kaydırır.
help-key-mouse-wheel = Fare tekerleği
help-nav-wheel = İmlece doğru yakınlaştırır/uzaklaştırır; çentik başına daha fazla yakınlaştırmak için daha hızlı çevirin. “Yakınlaştırmayı özelliklere çek” açıkken bir minibrotun yakınında yakınlaştırmak, yakınlaştırmayı onun üzerine çeker.
help-key-shift-drag = Shift+sürükleme
help-nav-shift-drag = Kutuyla yakınlaştırma — bir dikdörtgen çizin, görünümü dolduracak şekilde yakınlaştırır.
# $icon: the click-to-zoom toolbar button's symbol.
help-key-click-zoom = Tıklayarak yakınlaştırma ({ $icon })
help-nav-click-zoom = Araç çubuğundaki hedef düğmesini açın (tek görünüm): bundan sonra sol tık noktaya belirlenen çarpanla dalar, sağ tık geri çıkar. Sürükleme yine kaydırır; çarpanı (2–100×) sağ paneldeki Gezinme bölümünden seçin. Backspace bir tıklamayı geri alır.
help-nav-continuous = Sürekli yakınlaştırma ve başlangıç
help-key-hold-space = Space basılı
help-nav-hold-space = İmleç sabit noktada kalarak akıcı biçimde yakınlaştırır.
help-key-hold-shift-space = Shift+Space basılı
help-nav-hold-shift-space = Akıcı biçimde uzaklaştırır.
help-nav-continuous-text = Başlangıca uzaklaş düğmesi, tam görünüme yumuşak bir geri uçuş canlandırır. Sağ paneldeki “Yakınlaştırma hızı” sürekli yakınlaştırmanın hızını ayarlar.
help-nav-autopilot = Otopilot
help-nav-autopilot-text = Elinizi sürmeden bir dalış için A tuşuna basın (ya da Araçlar → “Otomatik yakınlaştırma (otopilot)”): fraktalın kenarında ayrıntının en yoğun olduğu, tercihen görünümün ortasındaki bir noktayı seçer ve onu ekranın merkezine doğru yumuşakça kaydırırken içine süzülür. İlginç kaldığı sürece o hedefi korur ve daha zengin bir yapı belirdiğinde yumuşakça ona yönelir. Herhangi bir gezinme girdisi — ya da Esc — onu durdurur. Sağ paneldeki (ya da Araçlar altındaki) “Otomatik yakınlaştırma önceliği” neyin önce geleceğini seçer: Hız, ayarlanan hızda yakınlaştırır ve ayrıntı içeren bir kare hazır olur olmaz ekranı yeniler; Kalite yalnızca tamamen çözümlenmiş, tam çözünürlüklü kareler gösterir ve resim bir sonraki gelmeden çok fazla büyümesin diye yakınlaştırmayı gerektiği kadar yavaşlatır. Her iki durumda da ekran hiçbir zaman boş ya da tek renkli bir kare göstermez: henüz hiç ayrıntı çözümlememiş bir yenileme, bir öncekinin arkasında gizli kalır. “Otomatik yakınlaştırma hedefi” neye nişan alınacağını seçer: Ayrıntı (görünümdeki en zengin kenar, ilerledikçe yeniden seçilir) ya da Misiurewicz noktası — en yakın Misiurewicz noktasını bulur, yerinde durarak onu dalış sınırının derinliğine kadar çözer, ardından tam içine dalar; böyle bir noktanın çevresindeki yapı sonsuza dek birkaç oktavda bir tekrarlanır, bu yüzden o dalış hiçbir zaman çıkmaza girmez (yalnızca Mandelbrot kümesi).
help-nav-history = Geçmiş ve hassas hareketler
help-nav-undo = Önceki görünüme geri döner.
help-nav-redo = Yineler.
help-nav-goto = Gezinme → “Konuma git…”, tam merkezi ve yakınlaştırmayı (tam hassasiyet, her derinlik) okumanızı, yazmanızı, yapıştırmanızı ya da kopyalamanızı sağlar. Merkez alanları düz sayılardan fazlasını kabul eder: “1/3” gibi kesirler ve pi, tau, e, sqrt ve sin/cos/tan ile kurulan ifadeler — yani “-0.5 + 0.25*cos(pi/4)” geçerli bir koordinattır. İfade görünümle birlikte tutulur ve her derinliğin gerektirdiği hassasiyetle yeniden hesaplanır; böylece bu şekilde girilen bir nokta ne kadar yakınlaştırırsanız yakınlaştırın kesin kalır (yuvarlanmış bir ondalık sayı ise bir noktada kalmazdı). Kutupsal bir mod (öteleme + r ∠ θ, derece / radyan / devir cinsinden) bu ifadelerden birini sizin için oluşturur. Fonksiyonların ve sabitlerin örnekli tam listesi bu yardımın “Koordinat ifadeleri” bölümündedir (penceredeki ? düğmesi onu açar). Gezinme → Yer imleri konumları kaydeder ve geri çağırır.
help-nav-exact-points = Gezinme → “Tam noktalar (ifadeler)”, yuvarlanmış bir ondalık sayıyla değil tam olarak tanımlanmış noktalara atlar — ana kardioidin tomurcuklarının kökleri, yani her q periyotlu tomurcuğun bağlandığı yerler, cos/sin ifadeleri olarak yazılmış. İfade görünümle birlikte taşındığı için bunlardan birine, noktadan hiç kaymadan yakınlaştırabilirsiniz.
help-nav-share = Gezinme → “Konumu paylaş…” tüm görünümü — fraktal, tam hassasiyetli merkez, yakınlaştırma ve renklendirme — kopyalayabileceğiniz, geri yapıştırıp uygulayabileceğiniz ya da dosya olarak kaydedip yükleyebileceğiniz küçük bir “.fdn” parçası olarak yakalar; böylece bir yeri tam olarak yeniden üretebilir (ya da birine gönderebilirsiniz).
help-nav-open = Dosya → “Görünüm veya konum aç…” bunların herhangi birini doğrudan açar: dışa aktarılmış bir PNG/EXR (gömülü görünümü geri yüklenir), bir “.fdn” konum dosyası ya da bir Kalles Fraktaler “.kfr” dosyası.
help-nav-finding = Ayrıntı bulma
help-nav-minibrots = Minibrotlar, sınır boyunca gizlenmiş, tüm kümenin küçük kopyalarıdır. Birini kabaca ortalayın ve M tuşuna basın (ya da Araçlar → “Minibrot bul + yakınlaştır”): Newton yöntemiyle tam merkezine oturur, periyodunu okur ve doğrudan kendi ölçeğine yakınlaştırır — çoğu zaman tek adımda birçok büyüklük mertebesi. Zaten minibrottan daha derindeyseniz yalnızca merkez değişir.
help-nav-minimap = Görünüm → “Mini harita”, tüm kümenin “buradasınız” işaretli ve geçerli yakınlaştırma derinliğini gösteren küçük bir haritasını gösterir; bir bölgeye atlamak için haritaya tıklayın.

## Help ▸ Coloring & options
## Names in quotation marks are the controls' own labels in the right-hand panel.

help-options-intro = Bunların hepsi sağdaki panelde bulunur (ve oturumlar arasında korunur).
help-options-palette = Palet
help-options-palette-text = Palet; hazır bir gradyanı, Özel gradyanınızı ya da iki renkli bir modu seçer. “Gradyanı düzenle…”, her renk durağının bir rengi ve bir konumu (0–1) olduğu bir düzenleyici açar; durak ekleyin, başlangıç için bir hazır ayarı kopyalayın ya da bir palet dosyası içe aktarın.
help-options-paste = Gradyan düzenleyicideki “Yapıştır…”, başka bir yerde bulduğunuz bir paleti içe aktarır. Başında # olan ya da olmayan onaltılık renkleri (#ff8800, ff8800 ya da kısa #f80) ve “255 136 0” gibi düz 0–255 RGB üçlülerini — Fractint ve Kalles Fraktaler .map dosyalarının kullandığı biçim — virgül, boşluk ya da yeni satırla ayrılmış olarak kabul eder. En fazla 256 renk tutulur; daha uzun bir liste eşit aralıklarla 256 renge örneklenir ve gradyanın şekli korunsun diye ilk ve son renk tutulur.
help-options-import-map = Yanındaki “İçe aktar ▸ .map…” bir Fractint ya da Kalles Fraktaler .map palet dosyası yükler. Bunlar, girdiler arasında karışım olmadan yineleme sayısıyla dizinlenen arama tablolarıdır ve bunun ürettiği sert renk basamakları klasik Fractint görünümünün ta kendisidir — bu yüzden öyle içe aktarılırlar; aynı renkleri bunun yerine yumuşak bir gradyana harmanlamak için bir “Keskin bantlar” onay kutusu vardır. Fractint’in kendi dosyaları dörtle çarpılmış 6 bitlik renk değerleri saklar, bu yüzden beyazları 255 değil 252’dir; Fractadyne böyle bir dosyayı içe aktarırken bunu belirtir ve değerleri tam olarak yazıldığı gibi bırakır, çünkü Fractint’in kendi kaydettiği görüntüler de aynı 252 değerini taşır.
help-options-duotone = İki ton, değeri yumuşak, iki renkli bir geçişe eşler (Gölge → Vurgu). İkili (küme) düz, iki renkli bir görünümdür — kümenin içindeki noktalar için tek bir düz renk, dışındakiler için başka bir renk, gradyan yok — kümenin şeklini görmenin en net yolu.
help-options-cycle = Döngü, gradyanın yineleme aralığı boyunca kaç kez tekrarlanacağını ayarlar (daha sık ya da daha seyrek bantlar). Kaydırma tüm gradyanı döndürür.
help-options-normalize = “Derin renkleri normalleştir” paleti görünümde gerçekten bulunan kaçış değerleri aralığına sığdırır. Derin görünümler yüz binlerce yinelemeye yayılır; bu olmadan sabit bir döngü gradyanı komşu pikseller arasında binlerce kez başa sarar ve doğru ayrıntıyı beneklere dönüştürür.
help-options-log-scale = “Logaritmik renk ölçeği” paleti doğrusal yerine kaçış değerinin logaritmasına göre yayar. Derine indikçe kaçış sayıları üst uçta yığılır, bu yüzden doğrusal bir sığdırma gradyanın çoğunu sınıra sarılan ince bir kabuğa harcar ve geri kalanı neredeyse düz bırakır; logaritmik ölçek bunu açar ve aralık büyüdükçe bir yakınlaştırma videosu boyunca renkleri daha kararlı tutar.
help-options-animate = Animasyon renkleri zaman içinde döndürür — İleri, Geri ya da Git-gel kaydırmayı değiştirir; Rastgele sürekli olarak yumuşakça biçim değiştiren, uyumlu gradyanlar üretir. Hız saniyedeki döngüleri (ya da dönüşümleri) denetler; “Gradyanı karıştır” Rastgele modda yeni bir gradyan çeker.
help-options-method = Renklendirme yöntemi — verinin renge nasıl dönüştüğü
# The coloring methods, by the names the Coloring panel gives them, each with what it does.
help-method-smooth = Yumuşak yineleme
help-method-smooth-text = Kaçış zamanına göre klasik sürekli bantlar.
help-method-stripe = Şerit ortalaması
help-method-stripe-text = Yörüngenin açısından akan bantlar (Şerit yoğunluğu kaydırıcısı).
help-method-triangle = Üçgen eşitsizliği
help-method-triangle-text = Her adımın sınırlar arasında nereye düştüğünden ince doku.
help-method-trap = Yörünge tuzağı
help-method-trap-text = Yörüngenin bir şekle (nokta/çarpı/çember) uzaklığı; iç bölgeyi de renklendirir.
help-method-distance = Uzaklık tahmini
help-method-distance-text = Sınıra yakınlığa göre gölgelendirir.
help-method-decomposition = Ayrışım
help-method-decomposition-text = Son kaçış açısından hücreler.
help-options-relief = 3B kabartma aydınlatması
help-options-relief-text = Kabartmalı, ışıklı bir görünüm için yüzeyi sınırın eğiminden (türevden) gölgelendirir. Işık açısı yönü, Kabartma gücü ayarlar (düşük = daha keskin); “Işığı döndür” onu canlandırır. Yalnızca holomorf aileler (Mandelbrot / Multibrot).
help-options-glow = Uzaklık parıltısı
help-options-glow-text = Sınırın yakınında yoğunlaşarak parlayan ipliklere dönüşen parlak kontur bantları. Parıltı karışım miktarını, Bant genişliği aralığı belirler; “Parıltıyı canlandır” ise onları akıtır.
help-options-quality = Kalite ve yinelemeler
help-options-quality-text = Yineleme, bir piksel kümenin içinde sayılmadan önceki en fazla adım sayısıdır; “Otomatik ölçekle” yakınlaştırdıkça onu otomatik olarak yükseltir (daha derin ayrıntı daha fazlasını gerektirir). Kenar yumuşatma, görünüm durulunca durağan görüntüleri süper örnekler (2×–8×) ve dıştaki ince “tozu” bastırır.
help-options-other = Diğer
help-options-other-text = Yakınlaştırma hızı sürekli yakınlaştırmanın hızını ayarlar; Kare hızı sınırı kare hızını sınırlar; “İkisi, bağlantılı” bir Mandelbrot kümesini ve onun Julia kümesini yan yana gösterir (imleç Julia parametresini canlı olarak ayarlar).

## Help ▸ Fractals
## Fractal family names (Mandelbrot, Multibrot, Tricorn, Burning Ship, Celtic, Buffalo,
## Phoenix, Newton) are names: keep them as written, as the formulas. Formula syntax
## (z = z^3 - p1*z + c, sqr, |z|, if / elseif / endif, p1-p5, maxit) is typed as shown.

help-fractals-intro = Aksi belirtilmedikçe her aile bir formülü, z 0’dan başlayıp c piksel tarafından belirlenecek şekilde yineler (kaçış zamanı). z = x + iy bir karmaşık sayıdır.
help-fractals-mandelbrot = z → z² + c. Orijinali. Küme, yörüngesi sınırlı kalan her c’den oluşur; bağlantılıdır ve sınırı o kadar kıvrımlıdır ki Hausdorff boyutu 2’dir.
help-fractals-multibrot = d = 3, 4, 5 kuvveti için z → z^d + c. Daha yüksek kuvvetler loblar ekler: kümenin (d−1) katlı dönme simetrisi vardır (Multibrot 3 için 2 katlı, 4 için 3 katlı, 5 için 4 katlı).
help-fractals-tricorn = z → conj(z)² + c; burada conj(x + iy) = x − iy. Eşlenik alma onu anti-holomorf yapar; 3 katlı simetri ve karakteristik kıvrık “pençeler” verir.
help-fractals-burning-ship = z → (|x| + i|y|)² + c — kare almadan önce mutlak değerleri alın (gerçel = x²−y²+cx, sanal = 2|xy|+cy). Analitik değildir; derin yakınlaştırmalar gemiye benzer yapılar ortaya çıkarır (geleneksel olarak baş aşağı görüntülenir).
help-fractals-celtic = Mandelbrot gibi, ancak gerçel kısmın mutlak değeriyle: gerçel = |x²−y²| + cx, sanal = 2xy + cy. Kalp ve kalkan biçimli motifler üretir.
help-fractals-buffalo = z² ifadesinin her iki kısmının mutlak değeri: gerçel = |x²−y²| + cx, sanal = |2xy| + cy — Celtic ile Burning Ship arasında bir melez.
# A heading: the families at powers above 2 (the Fractal menu's "Higher powers").
help-fractals-powers = Yüksek kuvvetler
# ("Burning Ship  3 4 5") is how the Fractal menu shows such a row: keep it as it is.
help-fractals-powers-text = Multibrot 6, 7 ve 8 (z^d + c) ile 3, 4 ve 5 kuvvetlerinde Burning Ship, Tricorn, Celtic ve Buffalo: daha yüksek bir kuvvette aynı katlamalar. Burning Ship d, z’nin kısımlarının mutlak değerlerini alır, ardından kuvveti uygular; Tricorn d önce eşleniğini alır; Celtic d ve Buffalo d, z^d ifadesinin gerçel kısmını (Buffalo’da sanal kısmını da) katlar. Fraktal menüsü ve açılır liste bunları her satıra bir aile gelecek şekilde listeler (“Burning Ship  3 4 5”). Her biri kuvvet-2 kardeşi gibi derin yakınlaştırılır; Multibrot 6, 7 ve 8 de Multibrot 3–5 gibi seri yaklaşımını ve minibrot bulucuyu (M) kullanır. Multibrot 8, bir sonraki adımı tek hassasiyet aralığının dışına çıkacağı için 256 yerine 128 yarıçapında kaçar.
help-fractals-phoenix = z → z² + c + p·z(n-1); burada z(n-1) bir önceki yineleme değeri, p ise bir sabittir (p = -0.5). Bellek terimi aleve benzer iplikler üretir.
help-fractals-newton = z³ − 1 = 0 denkleminin kökleri için Newton yöntemi: z → z − (z³−1)/(3z²). Pikseller kaçış zamanına göre değil, yinelemenin birimin üç küp kökünden hangisine yakınsadığına (çekim havzaları) ve ne kadar hızlı yakınsadığına göre renklendirilir; birbirine dolanmış havza sınırları fraktalın kendisidir.
help-fractals-julia = Julia modu
help-fractals-julia-text = Newton dışındaki her aile için bir Julia kümesine geçebilirsiniz: z’yi 0’dan ve c’yi piksel olarak başlatmak yerine c’yi (bir parametre) sabitler ve z’nin pikselden başlamasını sağlarsınız. Julia kümesi, o sabit c için sınırlı kalan başlangıç noktaları ile kaçanlar arasındaki sınırdır. “İkisi, bağlantılı” modunda imleci Mandelbrot panelinin üzerinde gezdirmek c’yi canlı olarak ayarlar.
help-fractals-deep = Derin yakınlaştırma desteği
help-fractals-deep-text = Mandelbrot, Multibrot 3/4/5 ve Tricorn aşırı derinlikte (floatexp) pertürbasyonlu derin yakınlaştırmayı destekler. Burning Ship, Celtic ve Buffalo analitik değildir (mutlak değer alırlar), bu yüzden işarete duyarlı bir pertürbasyon kullanırlar; bu artık floatexp aralığında da çalışır ve eski ~1e28× df32 sınırının çok ötesine derin yakınlaştırır (mutlak değer katlamalarının yakınındaki seyrek benekler çoklu referanslı bozulma düzeltmesini bekliyor). Phoenix iki terimli bir pertürbasyon kullanır ve o da floatexp aralığında derin yakınlaştırır (SA/BLA olmadan, bu yüzden Mandelbrot’tan daha ağırdır). Yüksek kuvvetler de aynı şekilde derin yakınlaştırır, her kuvvetin pertürbasyonu kendi kare alma zincirini izler; Multibrot 3–8, Mandelbrot gibi seri yaklaşımı ve BLA da kullanır; Tricorn, Burning Ship, Celtic ve Buffalo ise her kuvvette BLA kullanır (seri yok); BLA, derin bir görünümün iç bölgesini adım adım yinelemek yerine taşır. Yalnızca Newton doğrudan yolu kullanır ve ~1e6× düzeyine kadar keskindir; özel formüller, aşağıdaki Özel formüller bölümü aksini söylemedikçe derin yakınlaştırır.
help-fractals-custom = Özel formüller
help-fractals-custom-text = Fraktal ▸ Özel formül…, bir formülün adımını Fractint tarzı ifadelerle alır: z = z^3 - p1*z + c ya da birkaç deyim (t = sqr(z), ardından z = t*t + c). z yineleme değeri, c piksel (Julia modunda Julia sabiti), p1-p5 ise parametrelerdir; |z|, Fractint’teki gibi modülün karesidir ve alışılmış fonksiyonlar kullanılabilir (sqr, abs, conj, real, imag, cabs, flip, recip, ident, exp, log, sqrt, sin, cos, tan, cotan, sinh, cosh, tanh, cotanh, asin, acos, atan, asinh, acosh, atanh, cosxx ile her kısmı ayrı ayrı yuvarlayan floor, ceil, trunc ve round). log, Fractint’te ve karmaşık analizde olduğu gibi doğal logaritmadır: birçok ders kitabının ln diye yazdığı şey (10 tabanlı logaritma yoktur). ln, cot, coth, Re ya da Im ile yazılmış bir formül okunmaz; sözdizimi denetimi kullanılacak fonksiyonu söyler, bir düğme formülü yeniden yazar ve ln yazınca log önerilir. Her adım, yerleşik fraktallarla aynı şekilde 1e28x ötesinde genişletilmiş aralıklı aritmetikle pertürbasyon yoluyla derin yakınlaştırılır; tek istisna, üssü z ya da c ile değişen bir kuvvet içeren formüldür (z^c gibi): o, doğrudan yolda işlenir ve c’nin tek hassasiyetli bir adımı bir pikseli kaplayana kadar, yani yaklaşık 1e4x ile 1e5x arasına kadar keskindir; orijinden uzakta daha az derindir. Formül penceresi hangisinin geçerli olduğunu söyler. log, sqrt ve tam sayı olmayan kuvvetler esas değerlerini alır; bu değerler negatif gerçel eksen boyunca sıçrar. Pertürbasyon sıçramayı tam olarak izler, ancak yörüngesi referansınkinin geçmediği bir yerden geçen bir piksel o andan itibaren yaklaşık tek hassasiyetle sürdürülür; bu yüzden yörüngeleri bir kesiğin iki yanına düşen derin görünümler orada gürültü gösterir (ölçüldü: sqrt(z^4 + c) için gerçel eksende 1e5x düzeyinde temiz, log formüllerinde 1e4x düzeyinde bile gürültülü). Her adım tam olarak yinelenir (seri yaklaşımı yok), bu yüzden yüksek yineleme sayılı derin bir görünüm, yerleşik bir fraktalınkinden daha yavaş işlenir. Fonksiyon içeren bir formül sınırının yakınında kaotik yörüngeler izler ve oradaki uzun bir yörünge tek hassasiyette ancak gevşek biçimde takip edilebilir: 2.000 yinelemede sin(z) + c formülünün piksellerinin üçte biri kesin yörüngeninkinden farklıdır, 60 yinelemede neredeyse hiçbiri. Görünümler, oturumlar ve dışa aktarılan görüntüler formülün metnini korur.
help-fractals-custom-sections = Fractint’in bölümleri de çalışır. Bir “:” işaretinden önceki deyimler, piksel başına bir kez çalışan başlangıç bölümüdür; son deyim olarak bir karşılaştırma, formülün kendi kaçış testidir ve doğru olduğu sürece yineleme sürer (|z| <= 100 && |z - 1| > 0.000001, kaçan ya da 1’e yerleşen bir yörüngeyi durdurur); if (...), elseif (...), else ve endif deyimleri seçer; bir adımda atanmadan önce okunan bir değişken değerini bir önceki adımdan alır (başlangıç bölümünden ya da 0). Karşılaştırmalar gerçel kısımları karşılaştırır ve 1 ya da 0 verir, && ve || bunları birleştirir; bir atama da bir değerdir (a = b = pixel). pixel, p1-p5, pi ve e atanabilir ve atanana kadar kendi değerlerini korur; maxit yineleme sayısıdır, ismand ise 1’dir (formülün Mandelbrot biçimi). Bunlardan herhangi birini ya da yuvarlamayı içeren bir formül, derin yakınlaştırma olmadan doğrudan yolda işlenir. Formül kitaplığı > İçe aktar, Fractint .frm dosyalarını da okur. Okunabilen her girdi bir formül olur ve satırı okumanın neyi değiştirdiğini söyler: fn1-fn4 Fractint’in varsayılanlarını alır (sin, sqr, sinh, cosh); burada c piksel olduğundan Fractint’in c değişkeninin adı c_ olarak değiştirilir; hiçbir şeyin atamadığı bir ad, Fractint’te olduğu gibi 0’dır; bir adın içindeki boşluklar, Fractint’in yaptığı gibi atılır (end if, endif olur). İçe aktarma iletisi okunamayan girdileri sayar ve en yaygın nedeni verir: ekran ve görünüm değişkenleri (whitesq, scrnpix, scrnmax, center, magxmag, rotskew), lastsqr ve rastgele sayılar desteklenmez. Koleksiyon’daki Fractint klasikleri (Magnet, Nova, Spider, Manowar ve diğerleri) bu bölümlerle yazılmıştır.
help-fractals-custom-editor = Formül alanında parantezler iç içe geçme düzeyine göre renk alır; imlecin yanındaki çift vurgulanır, eşi olmayan bir parantez kırmızı gösterilir (ayrıştırıcı yorumları yok saydığından bir yorumdakiler sayılmaz). Adlar siz yazarken tamamlanır: iki harften sonra bir liste, öyle başlayan fonksiyonları, adları ve kendi değişkenlerinizi önerir. Yukarı ve Aşağı seçer, Tab ya da Enter birini alır (bir fonksiyon açılış paranteziyle gelir), Esc listeyi kapatır ve tam olarak yazılmış bir addan sonra Enter yine yeni bir satır başlatır.
help-fractals-custom-textbook = Ders kitabı (formülün üstündeki anahtar) formülü bir ders kitabının dizdiği gibi dizilmiş olarak gösterir ve yerinde düzenler. Metindeki gibi yazın: / önündeki terimden bir kesir, ^ bir üs (bir üssün sonunda yazılan +, - ya da = üsten çıkar), ( parantez ya da bir fonksiyon adından sonra bir çağrı, | dikey çizgiler, parantez içindeki bir virgül ise karmaşık bir sabit oluşturur. Oklar formül içinde dolaşır, Yukarı ve Aşağı bir pay ile paydası arasında, Tab bir sonraki boş kutuya gider; bir kesrin ya da bir parantez çiftinin başında Backspace onu seçer, ikinci basış onu kaldırır ve içindekini korur. Kopyalama, yapıştırma ve geri alma bir metin alanındaki gibi çalışır, adlar orada olduğu gibi tamamlanır ve tuş takımı dizilmiş formüle yazar (□² imlecin önündekinin karesini alır, bir fonksiyon tuşu seçimi sarar). Her düzenleme metne geri yazılır, böylece iki görünüm her zaman aynı formülü tutar; düzenlemediğiniz bir satır boşluklarını ve yorumunu korur. Gösterim, hesaplanan şeyi asla değiştirmez: |z| modülün karesidir ve |z|² olarak görünür, cabs(z) |z| olarak görünür, exp(z) ise exp(z) olarak kalır. Okunamayan bir satır kırmızı görünür; metin olarak düzenlemek için üzerine tıklayın. LaTeX düğmesi formülü dizildiği hâliyle LaTeX kaynağı olarak kopyalar; Ders kitabı açıkken formül kitaplığı da formüllerini dizilmiş olarak gösterir.
help-fractals-custom-numbers = Bir formüldeki sayılar (0.1, 1/3, pi, parametreler) yaklaşık 16 basamaklı çift hassasiyetli değerler olarak okunur ve derin referans yörünge tam olarak bu değerlerle yinelenir. Bu yüzden kâğıt üzerinde eşit olan iki formül derinlerde farklılaşabilir: 0.1*z, 0.1’e en yakın double değeriyle çarpar, z/10 ise tam 10’a böler ve iki resim yaklaşık 1e15x ötesinde ayrışır (ölçüldü: 1e15x düzeyinde piksellerin %5’i, 1e17x düzeyinde üçte biri, 1e20x düzeyinde tamamı). Her iki resim de kendi formülü için doğrudur; paylaşmak istediğiniz formülü yazın.
help-fractals-custom-library = Penceredeki Kitaplığa kaydet, bir formülü ve parametrelerini bir ad altında saklar (kitaplıkta bulunan bir adla kaydetmek o girdiyi günceller). Fraktal ▸ Formül kitaplığı… bunları uygulamak, düzenlemek, dışa aktarmak ya da silmek için listeler; İçe aktar, bir formül dosyasındaki formülleri hiçbirinin yerine geçmeden ekler: kitaplıkta zaten bulunan atlanır, adı alınmış yeni bir formül ise “ad (2)” olur. Kitaplık, Dosyalar ve veriler (Hakkında) altında adı verilen klasördeki formulas.toml dosyasıdır ve kendisi de bir formül dosyasıdır. Bir tur anahtar karesi bir formülü formula_params = [[re, im], ...] ile birlikte formula = "..." olarak taşır ve Araçlar > Geçerli görünümden tur, özel bir görünüm için bunları yazar.
help-fractals-custom-collection = Fractadyne ile birlikte, onun için yazılmış 45 formüllük bir koleksiyon gelir: yüksek kuvvetler ve Burning Ship ile Celtic ailelerinin katlanmış çeşitleri, karışımlar ve iki adımlı melezler, başka bir eşlemeden görülen Mandelbrot kümesi (ters çevrilmiş, log-kutupsal, sinüs üzerinden), aşkın ve rasyonel formüller ve Julia kümeleri. Bunlar penceredeki Örnekler ile formül kitaplığının Koleksiyon sekmesidir; orada Benimkilere kopyala birini formüllerinize ekler. Her birinin bir başlangıç görünümü vardır ve Uygula oraya gider: başlangıç görünümünde açılan bir formülün resmi çoğu zaman orada değildir. Gösterilirken kaydettiğiniz bir formül de bulunduğunuz görünümü aynı şekilde korur ve Uygula ona geri döner.

## Help ▸ How it works

help-how-escape = Kaçış zamanı ve yumuşak renk
help-how-escape-text = Her piksel, büyüklüğü bir kaçış yarıçapını aşana ya da yineleme sınırına ulaşana kadar formülü yineler. Ham adım sayısı tek başına sert bantlar oluşturur; son büyüklükten türetilen kesirli bir kısım eklemek sürekli, bantsız renk verir.
help-how-precision = Keyfi hassasiyetli konum
help-how-precision-text = Sıradan 64 bitlik sayıların basamakları 1e15× yakınlaştırma civarında tükenir. Fractadyne görünüm merkezini keyfi hassasiyetle tutar ve basamak sayısı yakınlaştırdıkça artar; böylece konum hiçbir zaman bozulmaz. Piksel ölçeği de genişletilmiş bir üsle tutulur, bu yüzden 64 bitin ~1e308× sınırında da takılmaz — derinliği herhangi bir sabit sayı aralığı değil, yalnızca sabrınız sınırlar.
help-how-perturbation = Pertürbasyon
help-how-perturbation-text = Her pikseli yüksek hassasiyetle yinelemek çok fazla yavaş olurdu. Bunun yerine tek bir referans piksel CPU’da yüksek hassasiyetle yinelenir (“referans yörünge”) ve diğer her piksel GPU’da, ondan küçük bir δ farkı olarak hızlı, düşük hassasiyetle hesaplanır: δz → 2·Z·δz + δz² + δc.
help-how-floatexp = Aşırı derinlik (floatexp)
help-how-floatexp-text = Yaklaşık 1e28× ötesinde o küçük fark bile 32 bitlik aralıkta alttan taşar, bu yüzden bir mantis artı ayrı bir tam sayı üs olarak saklanır (“floatexp”) ve 32 bitlik derinlik duvarı kalkar. Motor otomatik olarak geçiş yapar: sığken doğrudan matematik, derinken pertürbasyon, en derinde floatexp. Derinliği bundan sonra sabit bir sınır değil, koordinat hassasiyeti ve yineleme bütçesi sınırlar — 39 konumluk bir referans derleminde Fraktaler-3 ile çapraz denetlenmiştir; en derini 5.63e18003× (aşırı olmayan 38 satır olağan denetimdir, 6.13e1105× düzeyine kadar); ayrıca 1e1000000× düzeyine kadar bir iç hassasiyet tutarlılığı test takımı vardır; birlikte gelen tur ~1.3e420× düzeyine ulaşır.
help-how-reference = Referans seçimi ve yeniden tabanlama
help-how-reference-text = Referans, yörüngesi mümkün olduğunca uzun süre görünümün içinde kalacak şekilde seçilir (yüksek hassasiyetle puanlanır). Fark çok büyüdüğünde, doğru kalmak için referansa geri “yeniden tabanlanır”.
help-how-series = Seri yaklaşımı
help-how-series-text = Derin yakınlaştırmada ilk yinelemeler δ farkını referanstan neredeyse hiç uzaklaştırmaz, bu yüzden hepsi bir kerede hesaplanabilir: δz, katsayıları referans boyunca yinelenen, δc cinsinden kısa bir polinomla (3. dereceden) yaklaşık olarak hesaplanır. İşleyici, bu polinomun tüm görünüm için hâlâ doğru olduğu son yinelemeye kadar ileri atlar, ardından normal şekilde yineler — atlanan adımları, görüntüyü değiştirmeden kazandırır (Denetimler ▸ Gelişmiş bölümünden açılıp kapatılır). Mandelbrot ve Multibrot 3–8, en derin aralık; şerit / üçgen eşitsizliği / yörünge tuzağı / ayrışım renklendirmesiyle kullanılmaz (bunlar her yinelemeye ihtiyaç duyar). BLA’nın etkin olduğu yerde BLA bu erken atlamayı zaten kapsar.
help-how-distance = Uzaklık tahmini ve aydınlatma
help-how-distance-text = dz/dc türevini izlemek, her pikselin sınıra olan uzaklığını verir. Bu; 3B kabartma aydınlatmasını, uzaklık parıltısını ve “uzaklık” renklendirme yöntemini besler — hepsi her yakınlaştırma derinliğinde geçerlidir.
help-how-safety = Kenar yumuşatma ve güvenlik
help-how-safety-text = Durağan görüntüler, görünüm durulunca süper örneklenir (eksen başına 2–8×). Bir iş bütçesi, aşırı ayarlarda çözünürlüğü (asla yineleme sayısını değil) düşürerek tek bir GPU çiziminin sürücünün watchdog sınırı içinde kalmasını sağlar; böylece derin görünümler boşalmak yerine ayrıntılı kalır.

## Help ▸ Command line (the flags themselves stay English)

# The section's introduction. The flags listed below it are the command line itself, English in every language.
help-cli-intro = Fractadyne; işleme, film/tur dışa aktarma, otomasyon, referans görüntü denetimleri ve kıyaslama için arayüzsüz çalışabilir. Hiçbir seçenek verilmezse etkileşimli grafik arayüzü başlatır.

## Help ▸ Shortcuts

# What each mouse gesture and key does (the gestures are the Navigation section's).
help-shortcuts-pan = Kaydır
# A gesture: turning the mouse wheel.
help-key-wheel = Tekerlek
help-shortcuts-wheel = İmleçte yakınlaştır
help-shortcuts-box-zoom = Kutuyla yakınlaştırma (yakınlaştırmak için bir dikdörtgen çizin)
# $icon: the click-to-zoom toolbar button's symbol. "tool on": while that button is on.
help-key-click-tool = Tıklama ({ $icon } aracı açık)
help-shortcuts-click = Noktaya dal; sağ tık geri çıkar
# A heading: the keyboard shortcuts.
help-shortcuts-keyboard = Klavye
help-shortcuts-space = Sürekli yakınlaştırma / uzaklaştırma
help-shortcuts-undo = Görünümü geri al
help-shortcuts-redo = Görünümü yinele
help-shortcuts-minibrot = Minibrot bul + yakınlaştır (Newton-Raphson)
help-shortcuts-autopilot = Otomatik yakınlaştırma otopilotu (ayrıntıya doğru dalar; herhangi bir girdi durdurur)
# off / strip / panel / full: the panel's sizes, as View ▸ Live render panel names them.
help-shortcuts-live-panel = Canlı işleme paneli: kapalı / şerit / panel / tam (anlamını görmek için bir göstergenin üzerine gelin; köşesindeki × paneli kapatır)
help-shortcuts-esc = Otopilotu / oynatılan turu durdur ya da tam ekrandan çık
help-shortcuts-snapshot = Son dışa aktarma klasörüne anlık görüntü — görünümün gösterildiği gibi bir ekran görüntüsü ya da dışa aktarma ayarlarıyla tam bir işleme; ilk basışta hangisi olacağı sorulur, Dosya → Ayarlar → Anlık görüntü bunu değiştirir.
help-shortcuts-help = Bu yardımı aç
# A heading: the shortcuts of the Game of Life, with the menu path to it.
help-shortcuts-life = Life (Fraktal → Otomatlar → Life)
help-shortcuts-life-run = Evreni çalıştır / duraklat
help-shortcuts-life-step = Bir nesil ilerlet / adım aralığı kadar ilerlet (Life panelinde ayarlanır)
help-shortcuts-life-draw = Çizim aracını aç / kapat (araç çubuğundaki kalem)
# A gesture. "Draw on": while the Life draw tool is on.
help-key-click-drag-draw = Tıkla / sürükle (Çizim açık)
help-shortcuts-life-paint = Bir hücreyi çevir / boya: ölü bir hücrede başlayan bir çizgi çizer, canlı bir hücrede başlayan siler. Çalışan bir evren siz çizerken duraklar, bıraktığınızda devam eder.
# $key: a keyboard shortcut (Ctrl+Z). "Draw on": while the Life draw tool is on.
help-key-when-drawing = { $key } (Çizim açık)
help-shortcuts-life-undo = Son düzenlemeyi, doldurmayı, temizlemeyi ya da kural değişikliğini geri al

## Help ▸ Recommended hardware

help-hardware-intro = Fractadyne işini GPU (piksel başına yineleme + renklendirme) ile CPU (derin yakınlaştırmanın üzerine kurulduğu keyfi hassasiyetli referans yörünge) arasında böler. İkisi de önemlidir, ama farklı şeyler için.
help-hardware-gpu = GPU (en önemlisi)
help-hardware-gpu-text = Her piksel GPU’da yinelenir ve renklendirilir, bu yüzden kare hızını ve dışa aktarma süresini GPU hızı belirler. Güçlü 32 bit (tek hassasiyetli) işlem gücüne sahip ayrık bir GPU en akıcı derin yakınlaştırmayı sağlar. Tümleşik GPU’lar da sorunsuz çalışır — yalnızca daha yavaştır ve kare başına iş bütçesine daha çabuk ulaşırlar (bu noktada işleyici ayrıntıyı değil çözünürlüğü kırpar). Modern bir grafik API’si gerektirir: Vulkan, Direct3D 12, Metal ya da wgpu üzerinden OpenGL.
# The left column of a requirements table.
help-hardware-minimum = En düşük
help-hardware-gpu-minimum = Vulkan / DX12 / Metal / GL 3.3 destekli herhangi bir GPU.
help-hardware-recommended = Önerilen
help-hardware-gpu-recommended = Ayrık bir GPU (son ~5 yıl); >= 2 GB VRAM.
help-hardware-big-exports = Büyük dışa aktarmalar için
help-hardware-gpu-big-exports = >= 4 GB VRAM (karolar hâlinde işlendiği için performans kademeli olarak düşer).
# A heading: the processor.
help-hardware-cpu = CPU
help-hardware-cpu-text = Yüksek hassasiyetli referans yörünge (ve BLA tabloları) CPU’da hesaplanır ve büyük ölçüde tek iş parçacıklıdır, bu yüzden en çok çekirdek başına performans önemlidir — hareket ettikten sonra derin bir görünümün ne kadar çabuk çözümleneceğini o belirler. Ek çekirdekler çoğunlukla arka plandaki dışa aktarma kodlamasına yardımcı olur. 64 bitlik bir CPU gerekir.
help-hardware-cpu-recommended = İyi tek çekirdek hızına sahip modern, 64 bitlik bir CPU.
help-hardware-memory = Bellek
help-hardware-memory-text = Derin referanslar derinlikle büyür (yüksek hassasiyette çok sayıda yineleme); çok büyük dışa aktarmalar karoları RAM’de tutar. Günlük keşif için 8 GB rahattır; aşırı derinlik ve çok büyük dışa aktarmalar için 16 GB ve üstü uygundur.
help-hardware-checking = Donanımınızı denetleme
help-hardware-diagnostics = Yardım → “Tanılama…” geliştiricilerin çalıştırdığı testlerin aynısını sizin makinenizde çalıştırır. “Öz sınamayı çalıştır” matematiği ve işlemeyi yaklaşık on beş saniyede, doğruluğu bilinen sonuçlarla karşılaştırarak denetler; “Arayüz testini çalıştır” arayüzü ve canlı görünümü birkaç yakınlaştırma derinliğinde dolaşarak ekran görüntüleri yakalar; “GPU aritmetik denetimini çalıştır” gölgelendiricinin çift-float aritmetiğini makinenizin sunduğu her grafik arka ucunda, birkaç saniyede ve pencere açmadan doğrular. Her biri ayrı bir süreç olarak çalışır; böylece grafik sürücüsünü bozan bir test oturumunuzu değil, yalnızca kendisini çökertir.
help-hardware-gpu-check = GPU aritmetik denetiminin geçti ya da kaldı sonucu yoktur — sizin GPU’nuzun ve sürücünüzün nasıl hesap yaptığını kaydeder ve bazı donanımlarda (şimdiye kadar denenen her NVIDIA kartında) gölgelendirici derleyicisinin genişletilmiş hassasiyetli aritmetiğin bir kısmını attığını bildirir. Bu, üreticiler genelinde haritalamaya çalıştığımız gerçek bir bulgudur, makinenizdeki bir arıza değil; bu yüzden ne derse desin, sonucu bir rapora eklemek gerçekten yardımcı olur.
help-hardware-report = Bir şey yanlış görünüyorsa öz sınamayı çalıştırın ve “Sorun raporuna ekle…” seçeneğini kullanın: rapor böylece kendi donanımınızdan, makine tarafından doğrulanmış bir sonuç taşır; bu, tek başına bir açıklamadan çok daha yararlıdır. Bilinmesinde yarar olan bir uyarı — referans görüntüler belirli bir grafik kartında üretildi ve farklı GPU’lar meşru olarak biraz farklı pikseller hesaplar; bu yüzden başka donanımlarda görüntü karşılaştırmaları daha geniş bir toleransla denetlenir ve oradaki küçük farklar arıza değil, beklenen bir durumdur.
help-hardware-notes = Notlar
help-hardware-note-watchdog = Yetenekli bir GPU, aşırı ayarlarda watchdog iş bütçesinin devreye girmesini önler.
help-hardware-note-binary = Derleme tek bir çalıştırılabilir dosyadır ve kendi içinde eksiksizdir — çalışma zamanı kurulumu gerekmez.
help-hardware-note-benchmark = Makinenizi ölçmek için yerleşik kıyaslamayı çalıştırın (Araçlar ▸ Kıyaslama).

## Help ▸ Acknowledgments & citations
## Names of people, programs, papers, books and websites stay as written, and so do the
## titles in quotation marks (the works' own titles); the prose around them translates.

help-ack-title = Teşekkürler ve kaynaklar
# The dedication, in italics at the top of the section.
help-ack-dedication = Stone Soup Group’a ithaf edilmiştir — Fractint’in arkasındaki gönüllüler (1988’den beri).
help-ack-dedication-text = Özgürce paylaştıkları kod ve açık, iş birliğine dayalı fraktal keşfi ruhları, birçoğumuzun bu sanata âşık olmasının nedenidir. Fractadyne bu gelenek içinde geliştirildi.
help-ack-intro = Fractadyne, fraktal topluluğunun onlarca yıllık çalışmasının üzerinde durur. Üzerine kurulduğu teknikler ve önceki çalışmalar — aşağıdaki kaynaklarla doğrulanmıştır:
help-ack-deep-zoom = Derin yakınlaştırma algoritmaları
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-martin-title = Pertürbasyon ve seri yaklaşımı — { $who }
help-ack-martin = Referans yörünge + düşük hassasiyetli fark yöntemi (δz → 2Z·δz + δz² + δc) ve ilk yinelemeleri atlayan n. dereceden seri; SuperFractalThing’de ve onun “SuperFractalThing Maths” notunda (sft_maths.pdf, 2013) tanıtıldı.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bla-title = İki Değişkenli Doğrusal Yaklaşım (BLA) ve yeniden tabanlama — { $who }
help-ack-bla = Birleştirilmiş doğrusal eşlemelerle yineleme dizilerini atlama ve bozulmalardan kaçınmak için tek bir referansa yeniden tabanlama; fractalforums.org’da geliştirildi (2021).
# A link's label. $who: the linked page's author.
help-ack-bla-link = BLA’nın açıklaması — { $who }
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-glitch-title = Bozulma algılama ölçütü — { $who }
help-ack-glitch = Bir pikselin ne zaman farklı bir referansa ihtiyaç duyduğunu belirleyen |Z + z| << |Z| testi — bozulmasız pertürbasyonun temeli (fractalforums).
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-diffabs-title = Analitik olmayan (Burning Ship) pertürbasyon — { $who }
help-ack-diffabs = Pertürbasyonun mutlak değere dayalı ailelerde (Burning Ship / Celtic / Buffalo) çalışmasını sağlayan mutlak değer “diffabs” durum analizi; fractalforums’ta.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mathr-title = Referans uygulamalar ve çapraz denetimler — { $who }
help-ack-mathr = Fraktaler-3 ve Kalles Fraktaler 2+ (ilk olarak Karl Runmo tarafından yazıldı) — yazıları bu motora yol gösterdi; Fractadyne işleyicisini onların EXR çıktısıyla çapraz denetler.
# A heading: credits for the coloring methods.
help-ack-coloring = Renklendirme
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-harkonen-title = Yumuşak yineleme sayısı ve Şerit Ortalaması — { $who }
help-ack-harkonen = Sürekli (normalleştirilmiş) kaçış zamanı renklendirmesi ve Şerit Ortalaması yöntemi; Üçgen Eşitsizliği / dallanma ortalaması renklendirmelerini de inceleyen “On Smooth Fractal Coloring Techniques” (yüksek lisans tezi, 2007) çalışmasından.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mitchell-title = Üçgen Eşitsizliği Ortalaması — { $who }
help-ack-mitchell = Yumuşak, aleve benzer dış doku için her z² + c değerinin üçgen eşitsizliği sınırları içinde nereye düştüğünün ortalamasını alma.
# A link's label: the person's tutorial pages. $who: the person's name.
help-ack-mitchell-link = { $who } — öğreticiler
help-ack-distance = Uzaklık tahmini renklendirmesi/aydınlatması, klasik dz/dc sınır uzaklığı yöntemini kullanır (bkz. Peitgen & Saupe, “The Science of Fractal Images”, 1988).
help-ack-lsystems-life = L-sistemleri ve Hayat Oyunu
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-lindenmayer-title = L-sistemleri — { $who }
help-ack-lindenmayer = Bitkilerin ve alglerin nasıl büyüdüğünü modellemek için 1968’de tanıtılan paralel dizge yeniden yazımı (“Mathematical models for cellular interactions in development”, Journal of Theoretical Biology).
help-ack-abop = L-sistemi okuyucusunun izlediği kaplumbağa yorumunu ve stokastik, bağlama duyarlı ve parametrik sistemler için gösterimi sunan kitap (1990); ABOP olarak işaretlenen yerleşik sistemler onun şekillerinden girilmiştir.
# A link's label: the book can be read free on that website. $site: the website's name.
help-ack-abop-link = Kitap, ücretsiz: { $site }
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bourke-title = L-sistemi koleksiyonu — { $who }
help-ack-bourke = Yerleşik sistemlerin çoğu onun L-sistemi sayfalarından gelir; aralarında Chris Wallace’ın Sierpinski eğrisi kuralları ve Dietmar Saupe’nin çalısı vardır. Doğrudan açılan Fractint .l dosyaları Stone Soup Group’un biçimindedir.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-conway-title = Hayat Oyunu — { $who }
help-ack-conway = Dünyaya Martin Gardner’ın Scientific American’daki “Mathematical Games” köşesiyle (Ekim 1970) ulaşan B3/S23 hücresel otomatı (1970).
# A credit's heading: the website, then who runs it. $site: the website's name.
help-ack-lifewiki-title = { $site } — ConwayLife.com topluluğu
help-ack-lifewiki = Uygulamanın okuduğu kural gösterimleri (B/S, von Neumann, Generations, totalistik olmayan kurallar için Alan Hensel’in harfleri ve MAP), Golly ile paylaştığı RLE desen biçimi ve keşfedenlerini kaydettiği yerleşik kurallar ve desenler için başvuru kaynağı.
help-ack-foundations = Temeller ve araçlar
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mandelbrot-title = Mandelbrot kümesi — { $who }
help-ack-mandelbrot = Her şeyin kalbindeki nesne.
help-ack-built-with = Rust, egui/eframe (emilk) ve wgpu (gfx-rs) ile geliştirildi; keyfi hassasiyet astro-float, çift sözcüklü kayan noktalı sayılar twofloat ile; PNG/OpenEXR, rfd, serde. Yazarlarına ve bakımcılarına teşekkürler.
help-ack-errors = Atıflardaki her türlü hata yazara aittir; düzeltmeler projenin sorun izleyicisinde memnuniyetle karşılanır.

## Help ▸ Third-party licenses

help-licenses-title = Üçüncü taraf lisansları
# $licenses: Fractadyne's license expression (MIT OR Apache-2.0), as written. $url: the web address of a crate's source. THIRD-PARTY-NOTICES.md is a file name, `option-ext` a crate's name, MPL-2.0 a license's.
help-licenses-intro = Fractadyne, aşağıdaki açık kaynak crate’leri ve birlikte gelen yazı tiplerini içeren, statik olarak bağlanmış bir çalıştırılabilir dosya olarak dağıtılır. Tam lisans metinleri (indirmeyle birlikte gelen THIRD-PARTY-NOTICES.md dosyasında da bulunur) aşağıdadır. Fractadyne’ın kendi kodu { $licenses } olarak çift lisanslıdır. `option-ext` MPL-2.0 lisanslıdır — değiştirilmemiş kaynak kodu şu adrestedir: { $url }.
# A button, after a copy icon: copies every notice to the clipboard.
help-licenses-copy-all = Tüm bildirimleri kopyala

## Help ▸ About

help-about-tagline = Rust ve wgpu ile geliştirilmiş yerel bir fraktal gezgini.
# The arithmetic deep zoom has used. $ran: the libraries that have run so far, as names (or a note that none has yet); $built: those this build contains, with versions.
help-about-arithmetic = Derin yakınlaştırma aritmetiği: { $ran } (bu derlemenin içerdikleri: { $built }).
help-about-files = Dosyalar ve veriler
# Followed by the folder's path.
help-about-files-text = Oturumunuz (geçerli görünüm, renklendirme ve tercihler), yer imleri ve küçük resimleri, kaydedilmiş gradyanlar ve kaydedilmiş formüller bu kullanıcıya özel klasörde saklanır ve bir sonraki açılışta yeniden yüklenir:
# Tooltip of the copy button beside the folder's path.
help-about-copy-path = Yolu kopyala
# $command: a command typed in a terminal (it stays English).
help-about-session = Oturum dosyası sürümlüdür; onu tam okuyabilecek sürümden daha yeni bir Fractadyne tarafından yazıldıysa uygulama onu elinden geldiğince yükler ve sizi uyarır. Tamamen sıfırdan başlamak için Dosya ▸ Uygulama durumunu sıfırla… seçeneğini kullanın (önce sorar) ya da bir terminalde `{ $command }` komutunu çalıştırın.
help-about-license = Lisans
# $licenses: the license expression (MIT OR Apache-2.0), which stays as written.
help-about-license-text = { $licenses } — dilediğiniz lisansın koşullarıyla kullanabilirsiniz.
# A link to the source code, followed by ↗.
help-about-source = GitHub’da kaynak kod

## The live render panel ("blinkenlights": View ▸ Live render panel, key B)
## A front panel drawn over the view like a piece of 1970s test equipment. Its legends are engraved
## in capitals: write them in capitals here (the panel shows them exactly as written, it does not
## uppercase them). Numbers, units and readouts on its tubes and dials are not translated.

# Tooltip of the × button in the panel's top-right corner. The code adds the key after it: (B).
live-close-hover = Canlı işleme panelini kapat
# The legend engraved along the top plate, left of the maker's badge.
live-plate = CANLI İŞLEME · ÖN PANEL

## Live render panel: instrument titles
## Engraved above each instrument, and repeated as the title of its help card.

# Above the depth readout, whose tubes show the magnification as a power of two ("2^" stays).
live-title-depth = DERİNLİK 2^
live-title-iterations = YİNELEME
# Above the frame-time readout (milliseconds).
live-title-frame-ms = KARE MS
# Above the date-and-time readout.
live-title-clock = TARİH · SAAT
# The walk dial: a radar screen showing a slow refresh rendered pass after pass, band after band.
live-title-walk = TARAMA · BANTLAR
# A meter of how far the picture on screen lags the view.
live-title-age = RESİM YAŞI
# What part of the picture this frame's GPU pass covered.
live-title-tiles = KAROLAR
# A scope of the GPU's work per frame, in milliseconds.
live-title-scope = GPU GEÇİŞLERİ · MS
# The depth dial: depth, the reference orbits being built, and the arithmetic in use.
live-title-dial = DERİNLİK · REFERANSLAR · ARİTMETİK
# Iteration steps per second.
live-title-steps = ADIM / S
# The two GPUs (the window's and a second, worker GPU).
live-title-engines = İKİ MOTOR
live-title-memory = BELLEK
live-title-cores = ÇEKİRDEKLER · İŞ PARÇACIKLARI
live-title-pixels = PİKSEL AKIŞI
# Help card titles only: the three GPU timer lamps (T0, T1, T2 stay as they are) and the row of lamps.
live-title-timers = GPU ZAMANLAYICILARI T0–T2
live-title-bank = LAMBA SIRASI

## Live render panel: the light bank's lamps
## Engraved under each lamp, and the title of its help card. A cell is narrow (about eight capital
## letters): a longer label breaks at a space onto two lines, and is drawn smaller if it still does
## not fit.

live-lamp-plan = PLAN
live-lamp-dispatch = GÖNDERİM
live-lamp-live = CANLI
live-lamp-held = TUTULAN
# The last picture reprojected (stretched) to fit the moving view.
live-lamp-reproj = YENİDEN İZDÜŞÜM
live-lamp-reading = OKUMA
# A pass of a pinned walk.
live-lamp-pin-pass = SABİT GEÇİŞ
live-lamp-split = BÖLME
live-lamp-adopt = KABUL
live-lamp-abandon = TERK
# "REF": a reference orbit. Built for the view as it is (LIVE), ahead of it (AHEAD), or for a tour's next hold (HOLD).
live-lamp-ref-live = REF CANLI
live-lamp-ref-ahead = REF İLERİ
live-lamp-ref-hold = REF DURAK
# The series approximation.
live-lamp-series = SERİ
# The second (worker) GPU.
live-lamp-worker = YARDIMCI
live-lamp-tile = KARO
# Supersampling (anti-aliasing); usually kept as SSAA.
live-lamp-ssaa = SSAA
live-lamp-palette = PALET
# The arithmetic in use (direct, df32, floatexp).
live-lamp-mode = MOD

## Live render panel: the kinds of GPU pass
## In the GPU scope's legend (MOVING, PIN, WORKER, SETTLE, SSAA). SETTLE and SSAA also name a still
## view's walk on the walk dial.

# A moving frame's pass.
live-kind-moving = HAREKET
# A pass of a pinned walk.
live-kind-pin = SABİT
# The worker GPU's passes.
live-kind-worker = YARDIMCI
# A still view's settle tile or walk pass.
live-kind-settle = DURULMA
# A supersampling sample.
live-kind-ssaa = SSAA

## Live render panel: legends and readouts on the instruments

# On the walk dial during a split refresh. $done: passes done; $passes: passes in all.
live-walk-split = BÖLME { $done }/{ $passes }
# On the picture-age meter's face, under its scale (0 to 2 octaves).
live-age-scale = OKTAV GERİDE
# The depth dial's band for direct (plain floating-point) arithmetic. The other bands, DF32 and FLOATEXP, are names and stay.
live-dial-direct = DOĞRUDAN
# Under the two GPU gauges: GPU 0 draws the window; GPU 1 is the worker, or there is none.
live-engines-gpu-window = GPU 0 · PENCERE
live-engines-gpu-worker = GPU 1 · YARDIMCI
live-engines-no-worker = YARDIMCI YOK
# The share of displayed frames each GPU made. $window and $worker: percentages, 0–100.
live-engines-on-screen = EKRANDA { $window } : { $worker }
# The same readout before any frame was shown.
live-engines-on-screen-none = EKRANDA —
# Under the CPU core bars. $count: the machine's cores.
live-cores-count = { $count } ÇEKİRDEK
# Under the threads scope. $count: the app's busy threads.
live-cores-threads = İŞ PARÇACIĞI { $count }

## Live render panel: the memory tanks' legend
## What the GPU memory holds, by kind. Very narrow: about ten letters.

# G-buffers (the picture's working textures).
live-memory-gbuf = G-BUF
# The held picture and the supersampling accumulator.
live-memory-hold = RESİM+SSAA
# The reference orbit and its BLA (bilinear approximation) table.
live-memory-orbit = YÖRÜNGE+BLA
# The worker GPU's copy of the reference.
live-memory-worker = YARD. REF

## Live render panel: the roles on the threads scope
## Tiny labels on a small scope: at most four letters each.

# The reference builders.
live-cores-role-ref = REF
# The scoring pool.
live-cores-role-pool = PUAN
# The thread driving the worker GPU.
live-cores-role-worker = YRD
# Probes: short side renders.
live-cores-role-probe = SOND
# Image and tour exports.
live-cores-role-export = DIŞA

## Live render panel: the tiles instrument's captions
## The line under the picture's plane, then the tile size (left) and the tiles a frame (right).

# A walk's pass. $lo and $hi: the iteration band it rendered (numbers).
live-tiles-pin-band = SABİT GEÇİŞ · BANT { $lo }–{ $hi }
live-tiles-settle-band = DURULMA GEÇİŞİ · BANT { $lo }–{ $hi }
live-tiles-moving-whole = HAREKETLİ KARE · TÜM RESİM
live-tiles-settled-whole = DURULMUŞ KARE · TÜM RESİM
# $pass: this pass; $passes: the passes the refresh is split into.
live-tiles-split-pass = BÖLÜNMÜŞ YENİLEME · GEÇİŞ { $pass } / { $passes }
# $grid: the grid's columns × rows, e.g. 12×8.
live-tiles-grid-done = DURULMA IZGARASI { $grid } · BİTTİ
# $grid: the grid's columns × rows; $done and $total: tiles done, and in all.
live-tiles-grid-progress = DURULMA IZGARASI { $grid } · { $done } / { $total } BİTTİ
# The same, while a tile too costly for one frame goes out in strips.
live-tiles-grid-strips = DURULMA IZGARASI { $grid } · { $done } / { $total } BİTTİ · ŞERİTLER HÂLİNDE
# $samples: supersampling samples folded so far; $lo and $hi: the iteration band the next one is walking.
live-tiles-ssaa-band = SSAA · { $samples } BİTTİ · BANT { $lo }–{ $hi }
# $sample: the supersampling sample.
live-tiles-ssaa-whole = SSAA ÖRNEĞİ { $sample } · TÜM KARE
# No GPU pass this frame, and the panel has not seen a frame yet.
live-tiles-idle-starting = BAŞLIYOR
# No GPU pass this frame: the last complete picture is held on screen.
live-tiles-idle-held = TUTULAN
# No GPU pass this frame: the last picture is stretched to fit the view.
live-tiles-idle-reprojected = YENİDEN İZDÜŞÜM
# No GPU pass this frame: the picture on screen is current.
live-tiles-idle-live = CANLI
# No GPU pass this frame, and nothing to show.
live-tiles-idle-none = BOŞTA
# The tile size: $res, a whole picture's resolution (2400×1300).
live-tiles-tile-res = KARO { $res }
# The tile size: $px, a tile's side in pixels.
live-tiles-tile-px = KARO { $px } PX
live-tiles-no-pass = GEÇİŞ YOK
# Tiles a frame. $count: how many (0 when none).
live-tiles-rate-count = { $count }/KARE
live-tiles-rate-one = 1 KARO/KARE
# $share: the part of one tile sent each frame, e.g. 0.33.
live-tiles-rate-part = { $share } KARO/KARE
live-tiles-rate-sample = 1 KARO/ÖRNEK
# In the plane's corner during a walk: the share of pixels still iterating. $percent: a percentage, e.g. 54 or 1.5.
live-tiles-running = ÇALIŞIYOR %{ $percent }

## Live render panel: help cards
## Shown when the pointer rests on an instrument or a lamp. Words in capitals (SETTLE, SSAA, ABANDON,
## HELD, REPROJ, ADOPT, REF AHEAD, ON SCREEN) are the panel's own legends: use the same words as
## their messages above.

# The heading of a card's list of what healthy looks like.
live-card-healthy = SAĞLIKLI
# The heading of a card's list of what signals trouble.
live-card-trouble = SORUNLU

live-card-walk-what = Tek bir kare için fazla maliyetli derin bir yenileme sabitlenir ve taranır: geçiş üstüne geçiş işlenir, her geçiş piksellerin bir oktav daha ileri yinelemesine izin verir (256, 512, 1.024 … istenen sayıya kadar). Kenar çemberi bu sayıyı tepeden başlayarak logaritmik ölçekte gösterir; ışın taramanın ilerlemesidir; her yay bir geçiştir ve GPU süresine göre, 20 ms hedefine karşı renklendirilir. Beyaz nokta son geçişin okumasını bekler; parlama, yeni resmin ekrana gelişidir.
live-card-walk-note = Tek bir geçiş için fazla maliyetli durağan bir görünüm de aynı şekilde taranır ve DURULMA olarak işaretlenir; her süper örnekleme örneği de yeniden taranır ve SSAA olarak işaretlenir: bunların geçişlerinin her biri 400 ms’ye kadar sürebilir ve yaylar buna göre renklendirilir. Bölünmüş bir yenileme ise bunun yerine her geçiş için bir halka çizer: her bant, karoların bir payı.
live-card-walk-good-sweep = Bir avuç karede tam tur atıp bir parlamayla biten bir süpürme.
live-card-walk-good-arcs = Yaylar çoğunlukla yeşil ve kehribar.
live-card-walk-good-still = Derin, durağan bir görünümde: örnek başına daha yavaş bir süpürme, yayları yeşilden kehribara.
live-card-walk-bad-red = Kırmızı yaylar: bütçeyi aşan geçişler, takılma olarak hissedilir.
live-card-walk-bad-restart = Yarı yolda durup yeniden başlayan bir süpürme: tarama terk edildi ve ekrandaki resim eski kalıyor.
live-card-walk-bad-hollow = İçi boş yaylar: zamanlaması hiç geri gelmeyen geçişler, bu yüzden sonraki geçiş körlemesine boyutlandırılır.

live-card-age-what = Ekrandaki resmin görünümün ne kadar gerisinde kaldığı, yakınlaştırma oktavı cinsinden (bir oktav 2× demektir). Bir yenileme işlenirken eski resim gerilmiş olarak gösterilir ve yaşlanır; yenileme geldiğinde yaş düşer. Alttaki şerit son birkaç saniyeyi gösterir.
live-card-age-good-sawtooth = Yaklaşık 0,3 ile 0,8 arasında bir testere dişi; ibre 0,5 işaretinin çevresinde salınır.
live-card-age-good-still = Durduğunuzda 0’a yakın.
live-card-age-bad-red = İbrenin 1,0 ötesinde, kırmızıda kalması: resim 2 kattan fazla eskimiştir ve yakınlaştırırken yumuşak ya da bloklu görünür.
live-card-age-bad-ramp = Hiç düşmeyen bir rampa: yenilemeler bitmiyor (TERK lambasını izleyin).

live-card-tiles-what = Bu karenin GPU geçişinin resmi nasıl kapladığı. Hareketli kareler ve tarama geçişleri resmin tamamını kaplar; altyazı bir tarama geçişinin yineleme bandını adlandırır. Bir taramanın geçişi yalnızca henüz kaçmamış pikselleri yineler: düzlem o paya kadar dolar (logaritmik ölçek, tabanda %0,01) ve payı köşede gösterir; böylece tarama derinleştikçe boşalır. Tek bir kare için biraz fazla maliyetli bir yenileme en fazla 8 geçişe bölünür; her biri 16 px’lik karolardan oluşan iç içe geçmiş bir kümedir. Tek bir geçiş için fazla maliyetli durulmuş bir görünüm, merkezden dışa doğru her karede bir tane olmak üzere bir karo ızgarasına dönüşür ve hâlâ fazla maliyetli bir karo şeritler hâlinde gönderilir. Altta okunanlar: karo boyutu ve kare başına karo.
live-card-tiles-good-whole = Yakınlaştırırken tam geçişler; birkaç geçişlik bölünmüş yenilemeler.
live-card-tiles-good-grid = Durduğunuzda: bir iki saniyede dolan onlarca karoluk bir ızgara, ardından SSAA.
live-card-tiles-bad-small = Yüzlerce küçük karodan oluşan bir ızgara: her geçişin bütçesi küçüktür ve keskinleşme uzun sürecektir.
live-card-tiles-bad-strips = Şerit olarak gönderilen çok sayıda karo: karolar, ızgaranın boyutlandırıldığından daha yavaş çalıştı.
live-card-tiles-bad-splits = 8 geçişte takılı kalan bölmeler: yenileme ancak sığıyor ve biraz daha derinde daha yavaş taramaya geri düşer.
live-card-tiles-bad-dark = Hareket ederken uzun süre karanlık: hiç geçiş yok.

live-card-scope-what = Her kare için bir sütun, en yenisi sağda: çalıştırdığı GPU işi, türe göre yığılmış (hareketli kare yeşil, sabit geçiş maliyetine göre yeşilden kırmızıya, durulma karosu ya da durağan görünümün tarama geçişi krem, süper örnekleme örneği lavanta, yardımcı GPU’nun kareleri ve örnekleri mavi), milisaniye cinsinden. Kesikli çizgiler 16,7 ms (60 fps), 33 ve 50’de. İçi boş bir işaret, zamanlaması hiç geri gelmeyen bir geçiştir.
live-card-scope-good-under = Hareket ederken 16,7 çizgisinin altında kalan sütunlar.
live-card-scope-good-pin = Sabit geçişler yeşilden kehribara.
live-card-scope-good-samples = Durağan bir görünümün süper örnekleme örnekleri daha uzun sürebilir: hiçbir şey hareket etmiyor.
live-card-scope-bad-over = 33 ya da 50 ms’yi aşan sütunlar: görünür takılma.
live-card-scope-bad-seconds = Saniyeler süren geçişler: GPU’nun watchdog’u aygıtı sıfırlayabilir.
live-card-scope-bad-hollow = Çok sayıda içi boş işaret: iş, ölçüm olmadan boyutlandırılıyor.

live-card-timers-what = Üç GPU zaman damgası yuvası. Bir geçiş zamanlandığında yuva yanar, okuması iki üç kare sonra geri geldiğinde söner. T0 her karenin geçişini zamanlar; T1 ve T2 ek sabit geçişleri zamanlar.
live-card-timers-good = İş olduğunda T0 yanıp söner; taramalar sırasında T1 ve T2 yanıp söner.
live-card-timers-bad-stuck = Yanık kalan bir lamba: okuması geri gelmiyor ve fiyatlar eskiyor.
live-card-timers-bad-dark = GPU meşgulken hepsi karanlık: hiçbir şey ölçülmüyor.

live-card-dial-what = Kırmızı ibre görünümün derinliğidir: ikinin kuvveti olarak büyütmesi. Bantlar GPU’nun orada kullandığı aritmetiktir: doğrudan, df32 (sayı başına iki float) ve 2^93 ötesinde floatexp (geniş üslü bir float). Noktalar oluşturulmakta olan referans yörüngeleridir: görünüm için yeşil, yakınlaştırmanın gittiği yönde görünümün ilerisi için kehribar, bir turun sonraki durağı için mavi. Bir halka bir kurulumdur; kırmızı bir çarpı, görünüm onu geçtiği için bırakılan bir oluşturmadır.
live-card-dial-good-ahead = İbrenin önüne düşen ve ibre onlara ulaşmadan halkalanan kehribar noktalar.
live-card-dial-good-crosses = Az sayıda kırmızı çarpı.
live-card-dial-bad-passing = İbrenin son kurulu referansını geçmesi: kareler bir referans bekler, donma olarak hissedilir.
live-card-dial-bad-crosses = Tekrarlanan kırmızı çarpılar: yakınlaştırma oluşturuculardan hızlı gidiyor ve CPU boşa harcanıyor.
live-card-dial-bad-crossing = 2^93 geçişi bilinen yavaş bir noktadır: ilk floatexp kareleri daha maliyetlidir.

live-card-bank-what = Her lamba kendi olayı gerçekleştiğinde yanar ve yaklaşık 150 ms’de söner; bu yüzden meşgul bir işleyici titreşir, boştaki ise karanlıkta kalır. Soldan sağa: kare, yenileme, referanslar, ardından yardımcı, durulma, palet ve aritmetik.
live-card-bank-note = Ne anlama geldiğini görmek için bir lambanın üzerine gelin.

live-card-depth-what = İkinin kuvveti olarak büyütme: 2^100 yaklaşık 1,3 × 10^30 eder.
live-card-depth-note = 2^93 ötesinde her adım daha maliyetlidir (floatexp) ve referansların oluşturulması daha uzun sürer.

live-card-iter-what = İstenen yineleme: her pikselin kümenin içinde sayılmadan önce ne kadar yineleyebileceği. Derinlikle birlikte artar.
live-card-iter-good = Daldıkça istikrarlı biçimde artar.
live-card-iter-bad-low = Derindeyken düşükte takılı: spiraller ve minibrotlar düz iç bölge olarak işlenir.
live-card-iter-bad-high = Çok yüksek: her yenileme daha maliyetlidir ve taramalar daha uzun sürer.

live-card-ms-what = Görüntülenen kareler arasındaki süre.
live-card-ms-good = Ekranınızın aralığı: 60 Hz bir ekranda 16,7.
live-card-ms-bad-stutter = 33’ün üstü: görünür takılma.
live-card-ms-bad-spikes = 50’nin üstünde sıçramalar: aksamalar, genellikle tek bir maliyetli geçiş (bkz. GPU GEÇİŞLERİ).

live-card-clock-what = Yerel saatle duvar saati, saniyenin onda birine kadar.
live-card-clock-note = Panelin her ekran görüntüsüne çekildiği anı damgalar; böylece bir sorunun resmi uygulamanın günlüğüyle eşleştirilebilir.

live-card-tach-what = Saniyedeki yineleme adımı. Beyaz: istenen adımlar (piksel × yineleme). Kehribar: GPU’nun fiilen yürüttüğü adımlar. Aradaki fark, BLA’nın (çift doğrusal yaklaşım) birçok yinelemeyi bir kerede atlamasıdır; ×N bu çarpandır. Kilometre sayacı oturumun toplamını gösterir.
live-card-tach-good = Derinde geniş bir fark: BLA işini yapıyor, ×10 ile ×1.000 arası.
live-card-tach-bad-together = Derinde ibreler bir arada: BLA burada atlama yapamıyor (çoğu zaman bir minibrotun iç bölgesinin yakınında), bu yüzden yenilemeler yavaş.
live-card-tach-bad-red = Beyaz ibre kırmızıda: GPU’nun bir karede yapabileceğinden fazlası isteniyor.

live-card-engines-what = Her GPU’nun ne kadar meşgul olduğu, kareleri ekrana taşıyan noktalarla: geçişlerinin son bir saniyedeki GPU süresi, yardımcının hareket kareleri ve süper örnekleme örnekleri dahil. EKRANDA, görüntülenen karelerden her birinin ürettiği paydır. Tek GPU ile yardımcı tarafı boşta kalır.
live-card-engines-good = Bir dalış sırasında ikisi de meşgul, yardımcı karelerin bir kısmını sağlıyor.
live-card-engines-bad-pinned = Pencerenin GPU’su olan GPU 0 %100’e yakın takılı: darboğaz odur ve kareler takılır.
live-card-engines-bad-idle = Etkin ama boşta bir yardımcı: kare teslim etmiyor.

live-card-memory-what = RAM: uygulamanın çalışma kümesi, kırmızı çizgi en yüksek değeri. VRAM: uygulamanın GPU’da tuttukları, ayırdığı boyutlardan hesaplanarak, türe göre (G-arabellekleri, tutma ve SSAA dokuları, yörünge ve BLA arabellekleri, yardımcının referans kopyası), kartın kapasitesine karşı.
live-card-memory-good = Yakınlaştırırken düz; referanslar gelip gittikçe basamak basamak yükselir ve geri iner.
live-card-memory-bad-leak = Tırmanıp hiç inmeyen RAM: bir sızıntı.
live-card-memory-bad-vram = Kartın boyutuna yakın VRAM: ayırmalar başarısız olabilir ya da aygıt kaybedilebilir.

live-card-cores-what = Çekirdek başına CPU kullanımı, yeşilden kehribara ve kırmızıya; ve her meşgul iş parçacığının kendi rolünün çevresinde döndüğü bir skop: referans oluşturucular, puanlama havuzu, yardımcı, yan işlemeler.
live-card-cores-good = Yakınlaştırırken bir iki çekirdek meşgul; bir referans seçildiğinde tüm çekirdeklerde kısa bir patlama.
live-card-cores-bad-every = Saniyelerce tüm çekirdekler meşgul: aşırı derinlikte referans oluşturma.
live-card-cores-bad-own = Uygulamanın kendi çekirdeği kırmızıda takılı: pencerenin kendisi takılır.

live-card-pixels-what = Saniyede hesaplanan piksel, Mpx/s cinsinden. Soluk: bir taramada hâlâ yineleniyor. Parlak: bitti.
live-card-pixels-good = Hareket ederken onlarca Mpx/s.
live-card-pixels-bad = Resim eskiyken sıfıra yakın: GPU bir şey bekliyor, bir referans ya da bir okuma.

## Live render panel: the lamps' help cards

live-card-lamp-plan-what = Bir kare planlandı: çözünürlük, istenen yineleme ve bütçe seçildi ve yenileme yapılıp yapılmayacağına karar verildi.
live-card-lamp-plan-good = Görünüm ya da ayarlar her değiştiğinde titreşir.
live-card-lamp-plan-bad = Yakınlaştırırken karanlık: arayüz iş parçacığı takılmış.

live-card-lamp-dispatch-what = Bu karede GPU’ya iş gönderildi.
live-card-lamp-dispatch-good = Herhangi bir şey değişirken yanar; boştayken karanlık.
live-card-lamp-dispatch-bad = Hiçbir şey değişmezken meşgul: uygulama durulmuyor.

live-card-lamp-live-what = Ekrandaki resim bu karenin kendi işlemesidir.
live-card-lamp-live-note = Derin bir dalış sırasında nadiren yanar ve bu normaldir: bkz. TUTULAN ve YENİDEN İZDÜŞÜM.
live-card-lamp-live-good = Bir karenin ucuz olduğu sığ derinliklerde yanar.

# "Prefer detail" is a setting's name.
live-card-lamp-held-what = Bir sonraki resim altında oluşturulurken son tam resim ekranda kalır: derin bir yakınlaştırmanın yenilemesi ya da “Yakınlaştırırken ayrıntıyı tercih et” açıkken durağan bir görünümün sonraki karosu veya süper örnekleme örneği.
live-card-lamp-held-good-dive = Derin yakınlaştırmalar sırasında aralıklarla yanar, KABUL ile kesilir.
live-card-lamp-held-good-still = Durağan bir görünüm keskinleşirken sürekli yanar, SSAA yanıp söner.
live-card-lamp-held-bad = Yakınlaştırırken KABUL olmadan saniyelerce yanar: yenilemeler bitmiyor.

live-card-lamp-reproj-what = Son resim, yenisi hazır olana kadar hareket eden görünüme sığacak şekilde gerilir.
live-card-lamp-reproj-good = Hareket ederken kısa patlamalar.
live-card-lamp-reproj-bad = Uzun süre yanar: resim ne kadar çok gerilirse o kadar yumuşar.

live-card-lamp-reading-what = GPU’dan bir okuma geldi: bir geçişin zamanlaması, kaçış aralığı ya da içeriği.
live-card-lamp-reading-good = İşleme sırasında düzenli bir titreşim.
live-card-lamp-reading-bad = İşleme sırasında uzun süre karanlık: zamanlamalar gelmiyor ve bütçeler tahmin yürütüyor.

live-card-lamp-pin-pass-what = Sabitlenmiş bir taramanın bir geçişi: resim bir yineleme bandı daha ileri işlendi.
live-card-lamp-pin-pass-good = KABUL ile biten geçiş dizileri.
live-card-lamp-pin-pass-bad = TERK ile biten diziler.

live-card-lamp-split-what = Bölünmüş bir yenilemenin bir geçişi: karoların bir payı, tam yineleme sayısında işlendi.
live-card-lamp-split-good = KABUL ile biten kısa diziler.
live-card-lamp-split-bad = İzin verilen en fazla sayı olan 8’lik diziler: yenileme ancak sığıyor.

live-card-lamp-adopt-what = Bitmiş bir yenileme ekrandaki resmin yerini aldı.
live-card-lamp-adopt-good = Yakınlaştırırken saniyede birkaç kez.
# "Picture age" names the instrument (its title, PICTURE AGE).
live-card-lamp-adopt-bad = Yakınlaştırırken seyrek: resim yaşlanır (bkz. Resim yaşı).

live-card-lamp-abandon-what = Sürmekte olan bir yenileme atıldı: görünüm çok uzağa gitti, çok eskidi ya da daha yenisi onun yerini aldı. Görünümün geride bıraktığı bir referans oluşturma da onu soluk yakar.
live-card-lamp-abandon-good = Seyrek ya da bir atlamanın hemen ardından bir kez.
live-card-lamp-abandon-bad = Sık: iş boşa gider ve resim eski kalır.

live-card-lamp-ref-live-what = Görünümün şu anki hâli için bir referans yörünge oluşturuluyor: hiç yok, sahip olduğu referanstan ayrıldı ya da daha uzun bir referansa ihtiyaç duyuyor.
live-card-lamp-ref-live-good = Kısa yanıp sönmeler.
live-card-lamp-ref-live-bad-long = Uzun süreler: derin görünümler onu bekler.
live-card-lamp-ref-live-bad-lookahead = REF İLERİ karanlıkken bir dalış sırasında sık yanar: ileri bakış yetişemiyor.

live-card-lamp-ref-ahead-what = Görünümden daha derinde, yakınlaştırmanın gittiği yerde bir ileri bakış referansı oluşturuluyor.
live-card-lamp-ref-ahead-good = Bir dalış sırasında düzenli yanıp sönmeler.
live-card-lamp-ref-ahead-bad = Derin bir dalış sırasında karanlık: görünüm referanslarını geride bırakacak.

live-card-lamp-ref-hold-what = Bir turun durak ön getirmesi: bir sonraki durak anahtar karesinin referansı, oynatma hiç beklemesin diye ona doğru süzülürken oluşturulur.
live-card-lamp-ref-hold-note = Tur oynatma dışında karanlık.
live-card-lamp-ref-hold-good = Bir turdaki her duraktan önce bir yanıp sönme.
live-card-lamp-ref-hold-bad = Bütün bir durak boyunca yanar: ön getirme gecikmiş ve tur onu bekliyor.

live-card-lamp-series-what = Yeni bir referans için seri yaklaşımı; her pikselin ilk yinelemelerini atlamasını sağlar.
live-card-lamp-series-good = Kısa.
live-card-lamp-series-bad = Derinde uzun: referansın kurulmasını geciktirir.

live-card-lamp-worker-what = İkinci GPU bir kare teslim etti.
live-card-lamp-worker-good = Yardımcı GPU etkinken düzenli yanıp sönmeler.
live-card-lamp-worker-bad = Yardımcı etkinken karanlık: teslim etmiyor.

live-card-lamp-tile-what = Bir durulma karosu ya da bir karonun bir şeridi GPU’ya gönderildi.
live-card-lamp-tile-good = Durduktan sonra bir patlama, ardından sessizlik.
live-card-lamp-tile-bad = Hiç durmuyor: bir şey görünümü ya da ayarları değiştirmeye devam ettiği için ızgara sürekli yeniden başlıyor.

live-card-lamp-ssaa-what = Durulmuş resme bir süper örnekleme örneği eklendi.
live-card-lamp-ssaa-good = Bir durulmanın ardından bir dizi yanıp sönme, sonra karanlık.
live-card-lamp-ssaa-bad = Tekrar tekrar yeniden başlıyor: durulma sürekli bozuluyor.

live-card-lamp-palette-what = Renk normalleştirmesi yeni bir kaçış aralığı aldı; böylece renkler yakınlaştırmayı izler.
live-card-lamp-palette-good = Yakınlaştırırken titreşir.
live-card-lamp-palette-bad = Yakınlaştırırken karanlık: palet aralığı donmuş, renkler düzleşiyor ve kontrastı düşüyor.

live-card-lamp-mode-what = Aritmetik değişti: doğrudan, df32 ya da floatexp.
live-card-lamp-mode-good = 2^93 gibi bir sınırda tek bir yanıp sönme.
live-card-lamp-mode-bad = İleri geri yanıp sönme: görünüm bir sınırın iki yanında gidip geliyor.

## The Render client window (File ▸ Render client…): lending this machine's GPU to a render farm

farm-client-title = İşleme istemcisi
farm-client-intro = Bir işleme çiftliği için kareler işleyin. Denetleyici — başka bir makinede Araçlar ▸ Çiftlikte işle… — adresini ve çiftlik anahtarını gösterir. Bu makine hiçbir bağlantı noktası açmaz: denetleyiciye kendisi bağlanır.
# A field label, and the label of the status row that names the controller.
farm-client-controller = Denetleyici
farm-client-find = Bul
farm-client-find-hover = Bu ağdaki denetleyicileri ara (bir denetleyici, çiftliği dinlerken yanıt verir)
farm-client-key = Çiftlik anahtarı
# A checkbox beside the masked farm key: show it in clear.
farm-client-show-key = Göster
# A field label (the name this machine goes by), and the label of the status row about this machine.
farm-client-machine = Bu makine
farm-client-machine-hover = Denetleyicinin bu makine için gösterdiği ad.
farm-client-share = Paylaşılan sürücü
# Greyed hint inside the empty field; the path is an example.
farm-client-share-hint = isteğe bağlı — örn. \\fileserver\share
farm-client-share-hover = Bu makinenin, denetleyicinin kullandığı paylaşılan sürücüye giden yolu (paylaşım modu): kareler bağlantı üzerinden gönderilmek yerine oraya yazılır. Her kareyi göndermek için boş bırakın.
farm-client-gpu = Grafik kartı
# The label of the row with the idle option.
farm-client-when = Ne zaman
# A checkbox followed by a number of minutes ("Only when idle for [5] min").
farm-client-when-idle = Yalnızca şu süre boşta kalınca:
farm-client-when-idle-hover = Yalnızca bu makinenin klavyesine ya da faresine bu kadar süre kimse dokunmadığında ya da ekranı kilitliyken iş al. Biri makineyi kullanmaya başladığında sürmekte olan kare hemen durur ve çiftliğe geri döner.
# A collapsible section: the largest job this machine accepts.
farm-client-limits = Sınırlar
farm-client-limits-note = Bunlardan fazlasını isteyen bir iş reddedilir, asla küçültülmez.
farm-client-limit-size = En büyük kare
farm-client-limit-ss = En fazla süper örnekleme
farm-client-limit-iter = En fazla yineleme

## Render client: the graphics-card choice

farm-client-gpu-system = Sistemin seçimi
farm-client-gpu-every = Tüm grafik kartları
# $count: this machine's graphics cards.
farm-client-gpu-every-n = Tüm grafik kartları ({ $count })
# $count: this machine's graphics cards.
farm-client-gpu-every-each = Tüm grafik kartları ({ $count }) — her biri için bir oturum
# A card the window cannot name. $number: its number (or numbers) as --list-adapters gives it.
farm-client-gpu-adapter = Bağdaştırıcı { $number }
farm-client-gpu-finding = Bu makinenin grafik kartları aranıyor…
# "NAME · GPU 1" is how such a machine is named in the controller's list: keep it as it is.
farm-client-gpu-hover = Çiftliğin karelerini hangi grafik kartının işleyeceği. Tüm kartlar seçiliyse her biri çiftliğe ayrı bir makine olarak katılır — “NAME · GPU 1”, “NAME · GPU 2” — ve Duraklat ile Bağlantıyı kes hepsine birden uygulanır.

## Render client: looking for controllers on the network (the Find button)

farm-client-finding = Bu ağdaki denetleyiciler aranıyor…
# $port: the UDP port number.
farm-client-found-none = Hiçbir denetleyici yanıt vermedi. Bir denetleyici, çiftliği dinlerken yanıt verir; güvenlik duvarı UDP { $port } trafiğine izin vermelidir.
farm-client-found-pick = Bu ağda — kullanmak için birine tıklayın:
# $identity: the controller's identity fingerprint.
farm-client-found-hover = Kimlik { $identity } — bu denetleyiciyi kullan (çiftlik anahtarı yine de gerekir)
# After a warning sign. $version and $git: this app's version and build.
farm-client-found-other-build = Bundan farklı bir derleme ({ $version } { $git }): bu makineyi reddeder.

## Render client: what is wrong with what was typed, or with starting the client

farm-client-bad-address = Denetleyicinin adresini, penceresinde gösterildiği gibi HOST:PORT olarak girin (örneğin 192.168.1.20:46733).
# $why: what is wrong with the key (in English).
farm-client-bad-key = Çiftlik anahtarı: { $why }
# $why: what is wrong with the name (in English).
farm-client-bad-name = Ad: { $why }
farm-client-no-config = Çiftlik anahtarının saklanacağı bir yapılandırma klasörü yok.
# $why: the system's reason.
farm-client-start-failed = İşleme istemcisi başlatılamadı: { $why }

## Render client: the state, in the status heading

farm-client-phase-connecting = Bağlanılıyor…
farm-client-phase-checking = Öz denetim çalışıyor…
farm-client-phase-idle = Bağlandı — iş bekleniyor
farm-client-phase-rendering = İşleniyor
farm-client-phase-in-use = Bekliyor — bu makineyi biri kullanıyor
farm-client-phase-paused = Duraklatıldı
farm-client-phase-retrying = Denetleyiciye ulaşılamıyor — yeniden deneniyor
# $seconds: the wait before the next try.
farm-client-phase-retrying-in = Denetleyiciye ulaşılamıyor — { $seconds } s sonra yeniden denenecek
farm-client-phase-disconnected = Bağlantı kesildi
farm-client-phase-ended = Sona erdi

## Render client: the line under the status heading

# $address: the controller's address (HOST:PORT).
farm-client-detail-connecting = Bağlanılıyor: { $address }…
# $controller: the controller's name.
farm-client-detail-checking = “{ $controller }” denetleyicisine bağlandı — öz denetim çalışıyor
# $controller: the controller's name.
farm-client-detail-approval = “{ $controller }” denetleyicisine bağlandı — kullanıcısının bu makineyi onaylaması bekleniyor
# $controller: the controller's name.
farm-client-detail-idle = “{ $controller }” denetleyicisine bağlandı — boşta
# $why: the check that failed and why (in English).
farm-client-detail-check-failed = Öz denetim başarısız oldu — { $why }
# $why: the controller's reason (in English).
farm-client-detail-refused = Denetleyici tarafından reddedildi: { $why }
# $why: the controller's reason (in English).
farm-client-detail-closed = Denetleyici bağlantıyı kapattı: { $why }
# $why: the controller's reason (in English).
farm-client-detail-ended = Denetleyici bağlantıyı sonlandırdı: { $why }
# $job: the job's name.
farm-client-detail-bundle = “{ $job }” işi — paketi alınıyor
# $job: the job's name.
farm-client-detail-job-waiting = “{ $job }” işi — işlenecek kareler bekleniyor
# $why: the limit the job is over (in English).
farm-client-detail-job-refused = Bu iş reddedildi: { $why }
farm-client-detail-job-over = İş bitti
# $first and $last: frame numbers.
farm-client-detail-rendering = Kareler işleniyor: { $first }–{ $last }
# $time: how long nobody must use the machine ("90 s", "5 min").
farm-client-detail-in-use = Bekliyor — bu makineyi biri kullanıyor; kimse { $time } boyunca kullanmadığında iş yeniden başlar
farm-client-detail-paused = Duraklatıldı — denetleyiciye bildirildi
farm-client-detail-pausing = Duraklatılıyor — sürmekte olan kare bitiriliyor
farm-client-detail-cancelled = Sürmekte olan kare iptal edildi; duraklatıldı
# A machine with several graphics cards. $rendering: the cards rendering now; $total: all its cards (two or more).
farm-client-rendering-on-cards = { $total } grafik kartından { $rendering } tanesinde işleniyor

## Render client: the status rows

# The value of the Controller row. $name: the controller's name; $address: HOST:PORT; $identity: its identity fingerprint.
farm-client-row-controller-value = “{ $name }” · { $address } · kimlik { $identity }
# The value of the This machine row. $name: this machine's name; $identity: its identity fingerprint.
farm-client-row-machine-value = “{ $name }” · kimlik { $identity }
farm-client-row-gpu = GPU
# One graphics card's row. $slot: the card's number.
farm-client-row-gpu-n = GPU { $slot }
farm-client-row-job = İş
# The frames handed to this machine.
farm-client-row-run = Dizi
farm-client-row-sent = Gönderilen
# The Run row, in parts joined with " · ": the run, the frame in progress, its time. $first and $last: frame numbers.
farm-client-run-frames = kareler { $first }–{ $last }
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-client-run-frame = kare { $frame } ({ $index } / { $count })
# $time: the time spent on it so far ("2.1 s").
farm-client-run-frame-time = bu karede { $time }
# One graphics card's run. $frame: the frame in progress; $first and $last: the run's frames.
farm-client-session-frame = kare { $frame } / { $first }–{ $last }
farm-client-session-no-card = grafik kartı henüz bilinmiyor
# The Sent row. $count: frames sent; $time: the mean time per frame ("2.1 s").
farm-client-sent =
    { $count ->
        [one] { $count } kare · kare başına { $time }
       *[other] { $count } kare · kare başına { $time }
    }
# The Sent row of a machine with several cards. $count: frames sent; $split: each card's share ("812 + 422"); $time: the mean time per frame.
farm-client-sent-split =
    { $count ->
        [one] { $count } kare ({ $split }) · kare başına { $time }
       *[other] { $count } kare ({ $split }) · kare başına { $time }
    }
# After a warning sign. $note: how the GPUs differ, and both GPUs (in English).
farm-client-gpu-note = Bu makinede { $note } var. Kareleri denetleyicininkilerden biraz farklı olacaktır; bu, durağan bir çekimde görülebilir.
# A collapsible section: the self-check's results.
farm-client-self-check = Öz denetim
farm-client-thumb-hover = Bu makinenin gönderdiği son kare
farm-client-resume = Sürdür
farm-client-resume-hover = Yeniden iş al
farm-client-pause = Duraklat
farm-client-pause-hover = Sürmekte olan kareyi bitir, ardından başka iş alma. Denetleyici bu makinenin karelerini diğerlerine verir.
farm-client-cancel-frame = Kareyi iptal et
farm-client-cancel-frame-hover = Sürmekte olan kareyi şimdi durdur ve duraklat; denetleyici onu başka bir makinede işler.
farm-client-live-paused = Bu makine çiftlik kareleri işlerken canlı görünüm duraklatıldı.
farm-client-close = Kapat
farm-client-close-active-hover = Pencereyi kapat; bu makine çiftlikte kalır (Dosya menüsünden yeniden açın)
farm-client-close-hover = Pencereyi kapat
farm-client-disconnect = Bağlantıyı kes
farm-client-disconnect-hover = Çiftlikten ayrıl: sürmekte olan kare geri verilir
farm-client-connect = Bağlan
farm-client-connect-hover = Denetleyiciye bağlan ve onun için kareler işle

## The Render on farm window (Tools ▸ Render on farm…): the controller of a render farm

farm-ctl-title = Çiftlikte işle
farm-ctl-out = Çıktı klasörü
farm-ctl-browse = Gözat…
farm-ctl-out-note = Kareler ve işin durumu farm/ içinde: sürdürmek için aynı işi yeniden çalıştırın.
farm-ctl-size = Boyut
farm-ctl-size-custom = Özel…
# The size list's choice when the size is typed in. $width and $height: pixels.
farm-ctl-size-custom-value = Özel — { $width }×{ $height }
farm-ctl-fps = Kare / saniye
farm-ctl-ss = Süper örnekleme
farm-ctl-port = Dinlenecek bağlantı noktası
farm-ctl-port-hover = İstemciler bu bağlantı noktasına bağlanır. Yerel ağınız için güvenlik duvarında buna izin verin.
farm-ctl-share = Paylaşılan sürücü
# Greyed hint inside the empty field.
farm-ctl-share-hint = isteğe bağlı — bu makinenin ona giden yolu
farm-ctl-share-hover = Paylaşım modu: aynı sürücüye sahip istemciler karelerini bağlantı üzerinden göndermek yerine oraya yazar ve bu makine her birini orada denetler. Sürücüye sahip olmayan istemciler karelerini gönderir.
# A label followed by a number of clients and two checkboxes.
farm-ctl-start-with = Başlamak için
# The unit after the number in the box ("Start with [3] clients"). $count: that number.
farm-ctl-start-clients =
    { $count ->
        [one] istemci
       *[other] istemci
    }
farm-ctl-start-clients-hover = İş, bu sayıda makine bağlanıp öz denetimini geçtiğinde başlar; diğerleri sonradan katılabilir.
farm-ctl-local = Bu makinede de işle
farm-ctl-local-hover = Bu makine bir istemci sayılır.
farm-ctl-sharing = Referansları paylaş
farm-ctl-sharing-hover = Derin bir karenin referans yörüngesinin oluşturulması dakikalar ile saatler arasında sürebilir. Paylaşıldığında bir kez oluşturulur ve diğer makinelere aktarılır. Paylaşılan referans farklı ama eşit derecede geçerli bir referanstır: aşırı derinlikte birkaç piksel, kendi referansını oluşturan bir kareden farklı olabilir.
farm-ctl-key = Çiftlik anahtarı
# A checkbox beside the shortened farm key: show all of it.
farm-ctl-show-key = Göster
farm-ctl-copy-key = Anahtarı kopyala
farm-ctl-copy-key-hover = Her istemcinin İşleme istemcisi penceresine yapıştırın
# In place of the farm key, when there is none.
farm-ctl-no-key = henüz yok
farm-ctl-make-key = Anahtar oluştur
farm-ctl-make-key-hover = İstemcilerin katılmak için buna ihtiyacı vardır; İşle düğmesine bastığınızda da oluşturulur
# What to do on the other machines. $address: this controller's HOST:PORT.
farm-ctl-steps = Her istemcide: Dosya ▸ İşleme istemcisi…, denetleyici { $address } ve çiftlik anahtarı.
# The same when this machine's address is not known. $port: the port number.
farm-ctl-steps-port = Her istemcide: Dosya ▸ İşleme istemcisi…, bu makinenin adresi ve { $port } bağlantı noktası ve çiftlik anahtarı.
# The size of the job. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
farm-ctl-summary =
    { $count ->
        [one] { $count } kare · { $seconds } s’lik tur · { $width }×{ $height } ss{ $ss }
       *[other] { $count } kare · { $seconds } s’lik tur · { $width }×{ $height } ss{ $ss }
    }
farm-ctl-strip-hover = İşin her karesi, soldan sağa: bitti, işleniyor, yapılacak, vazgeçildi.
farm-ctl-no-tour = Yüklü tur yok.
# $folder: the folder; $why: the system's reason.
farm-ctl-out-error = Çıktı klasörü { $folder }: { $why }
# $why: the system's reason.
farm-ctl-start-failed = Çiftlik başlatılamadı: { $why }
# $code: the controller process's exit code, as the code gives it.
farm-ctl-ended-exit = Denetleyici sona erdi (çıkış { $code }).

## Render on farm: the state, in the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-phase-waiting = İşleme istemcileri bekleniyor ({ $needed } istemciden { $connected } tanesi bağlı)
farm-ctl-phase-preparing = İş hazırlanıyor
farm-ctl-phase-rendering = İşleniyor
farm-ctl-phase-paused = Duraklatıldı
farm-ctl-phase-complete = Tamamlandı
farm-ctl-phase-stopped = Durduruldu
farm-ctl-phase-ended = Sona erdi

## Render on farm: the words after the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-detail-waiting = İşleme istemcileri bekleniyor: { $needed } istemciden { $connected } tanesi bağlı
farm-ctl-detail-preparing = Normalleştirme çapaları bu makinede, tüm istemciler için bir kez ölçülüyor
farm-ctl-detail-no-space = Çıktı klasöründe yer açılması bekleniyor
farm-ctl-detail-paused = Duraklatıldı: yeni iş dağıtılmıyor
farm-ctl-detail-not-rendered = İşlenmemiş karelerle bitti — sürdürmek için aynı klasörde yeniden çalıştırın
farm-ctl-detail-stopped = Durduruldu — sürdürmek için aynı klasörde yeniden çalıştırın
# $code: the controller process's exit code.
farm-ctl-detail-ended = Sona erdi (çıkış { $code })
farm-ctl-detail-finishing = Bitiriliyor

## Render on farm: progress

# On the progress bar. $done: frames done; $total: frames in the job.
farm-ctl-progress =
    { $total ->
        [one] { $done } / { $total } kare
       *[other] { $done } / { $total } kare
    }
# Added to the progress bar. $rate: frames per second, with decimals.
farm-ctl-progress-rate = { $rate } kare/s
# Added to the progress bar. $time: the time still to go ("38m 10s").
farm-ctl-progress-eta = yaklaşık { $time } kaldı
# Facts under the progress bar, joined with " · ". $received and $sent: kilobytes per second.
farm-ctl-io = gelen { $received } KB/s · giden { $sent } KB/s
# $space: free space on the output drive ("412 GB").
farm-ctl-free = { $space } boş
# $time: the time since the job started ("38m 10s").
farm-ctl-elapsed = { $time } geçti
# $count: frames given up on.
farm-ctl-failed = { $count } kare işlenmedi
# Reference orbits. $built: built for this job; $cached: found in the cache; $reused: reused between frames; $shared: passed between machines.
farm-ctl-refs = referanslar: { $built } oluşturuldu, { $cached } önbellekten, { $reused } yeniden kullanıldı · { $shared } paylaşıldı
# After a warning sign.
farm-ctl-storage-low = Çıktı klasörü neredeyse dolu: yer açılana kadar yeni iş dağıtılmaz.
# After a warning sign.
farm-ctl-gpus-differ = Bu çiftlikteki GPU’lar farklı. Kareleri biraz farklıdır; bu, durağan bir çekimde görülebilir.
# After a warning sign. $count: the GPU classes in the farm.
farm-ctl-gpus-differ-classes = Bu çiftlikteki GPU’lar farklı ({ $count } GPU sınıfı: farklı GPU’lar, sürücüler ya da sonda işlemeleri). Kareleri biraz farklıdır; bu, durağan bir çekimde görülebilir.
# $gpu: this machine's GPU, API and driver.
farm-ctl-this-gpu = Bu makine: { $gpu }
# A machine whose GPU differs. $name: the machine; $note: how its GPU differs (in English); $adapter: its GPU.
farm-ctl-gpu-row = { $name }: { $note } — { $adapter }
# The same with its driver. $driver: the driver's name and version.
farm-ctl-gpu-row-driver = { $name }: { $note } — { $adapter }, sürücü { $driver }

## Render on farm: the table of machines

# Column headers (short: the table is narrow).
farm-ctl-col-machine = Makine
farm-ctl-col-state = Durum
farm-ctl-col-gpu = GPU
farm-ctl-col-frames = Kare
farm-ctl-col-per-frame = Kare başına
# How the machine delivers its frames (a speed, or "shared drive").
farm-ctl-col-link = Bağlantı
# When the controller last heard from the machine.
farm-ctl-col-heard = Son temas
# In the machine's tooltip. $count: its frames that failed verification.
farm-ctl-strikes =
    { $count ->
        [one] { $count } kare doğrulamada başarısız oldu
       *[other] { $count } kare doğrulamada başarısız oldu
    }
# In the Link column: it delivers its frames through the shared drive.
farm-ctl-link-share = paylaşılan sürücü
# In the Heard column. $seconds: how long ago.
farm-ctl-heard-ago = { $seconds } s önce
farm-ctl-readmit = Yeniden kabul et
farm-ctl-readmit-hover = Bu makine yeniden bağlandığında tekrar katılmasına izin ver
farm-ctl-remove = Çıkar
farm-ctl-remove-hover = Bu makineyi işten çıkar; kareleri diğerlerine gider
# The GPU column's tooltip.
farm-ctl-probe-same = Sonda işlemesi bu makineninkiyle aynı.
# $count: the pixels that differ.
farm-ctl-probe-differs =
    { $count ->
        [one] Sonda işlemesi bu makineninkinden { $count } pikselde farklı: kareleri bu makineninkilerden biraz farklı olacak.
       *[other] Sonda işlemesi bu makineninkinden { $count } pikselde farklı: kareleri bu makineninkilerden biraz farklı olacak.
    }
farm-ctl-probe-unknown = Sonda işlemesi henüz karşılaştırılmadı.
# $driver: the driver's name and version.
farm-ctl-gpu-driver = sürücü { $driver }
# After a warning sign. $note: how its GPU differs (in English: "a different GPU").
farm-ctl-gpu-note = { $note } (bu makineninkine göre)

## Render on farm: a machine's state, in the State column

# $count: its frames that failed verification.
farm-ctl-state-removed-bad =
    { $count ->
        [one] çıkarıldı: { $count } hatalı kare
       *[other] çıkarıldı: { $count } hatalı kare
    }
farm-ctl-state-removed = sizin tarafınızdan çıkarıldı
farm-ctl-state-removing = çıkarılıyor — tanılama bilgileri toplanıyor
farm-ctl-state-gone = ayrıldı
farm-ctl-state-leaving = ayrılıyor
farm-ctl-state-checking = denetleniyor (öz denetim)
farm-ctl-state-paused = kullanıcısı tarafından duraklatıldı
farm-ctl-state-parked = bekletiliyor: iki kez çöktü — yeniden bağlandığında tekrar katılır
farm-ctl-state-refused = bu işi reddetti (kendi sınırları)
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-ctl-state-rendering = kare { $frame } işleniyor ({ $index }/{ $count })
farm-ctl-state-starting = bir dizi başlatılıyor
farm-ctl-state-idle = boşta
farm-ctl-state-admitted = kabul edildi — iş bekleniyor

## Render on farm: the buttons

# A collapsible section: the controller's messages.
farm-ctl-events = Olaylar
farm-ctl-resume = Sürdür
farm-ctl-pause = Duraklat
farm-ctl-pause-hover = Yeni iş dağıtma; sürmekte olan kareler biter
farm-ctl-live-paused = Bu makine çiftlik için işlerken canlı görünüm duraklatıldı.
farm-ctl-close = Kapat
farm-ctl-close-active-hover = Pencereyi kapat; iş çalışmaya devam eder (Araçlar menüsünden yeniden açın)
farm-ctl-close-hover = Pencereyi kapat
farm-ctl-copy-command = Komutu kopyala
farm-ctl-copy-command-hover = Eşdeğer komut satırını kopyala
farm-ctl-stop = Durdur
# The Stop button once a stop has been asked for and the job has not ended after 10 seconds.
farm-ctl-stop-now = Hemen durdur
farm-ctl-stop-hover = İşi elindeki karelerle bitir; aynı işi yeniden çalıştırmak onu sürdürür
farm-ctl-render = İşle
farm-ctl-render-hover = İstemcileri dinlemeye başla ve turu onlarda işle

## The Render tour window (Tools ▸ Render tour…)

tour-render-title = Tur işle
tour-render-out = Çıktı klasörü
tour-render-browse = Gözat…
tour-render-out-hover = Karelerin yazılacağı tam yol.
tour-render-prefix = Kare öneki
# <prefix> stands for the prefix typed in the field.
tour-render-prefix-hover = Kareler <önek>_00000.png olarak yazılır
tour-render-size = Boyut
tour-render-size-custom = Özel…
# The size list's choice when the size is typed in. $width and $height: pixels.
tour-render-size-custom-value = Özel — { $width }×{ $height }
tour-render-fps = Kare / saniye
tour-render-fps-hover = 1 fps’nin altı geçerlidir: uzun bir dalışı 0,25 fps ile örneklemek, binlerce kare işlemeden derin regresyon duraklarını incelemenin yoludur.
tour-render-ss = Süper örnekleme
tour-render-chapter = Bölüm
tour-render-whole = Tüm tur
tour-render-chapter-hover = Tek bir bölümü işlemek GENEL kare numaralandırmasını korur; böylece kareleri doğrudan tam bir dizinin içine geri yerleşir.
tour-render-mp4 = mp4 oluştur
tour-render-mp4-hover = Ardından ffmpeg çalıştırılır (PATH üzerinde olmalıdır). PNG kareleri korunur.
tour-render-overwrite = Mevcut karelerin üzerine yaz
tour-render-overwrite-hover = Kapalıyken işleme sormak için durur; bunu buradan yapamaz.
tour-render-resume = Yarıda kalmış bir işlemeyi sürdür
tour-render-resume-hover = Çıktı klasöründeki mevcut kareleri korur ve yalnızca eksik olanları işler. En yeni kare önce denetlenir ve eksikse atılır (bir işlemenin çöktüğü kare, o sırada yazdığı karedir); iyi bir kare bulunana kadar geriye gidilir. Farklı boyutta kareler içeren bir klasör bu diziye karıştırılmak yerine reddedilir.
# $count: this machine's graphics cards.
tour-render-all-gpus = Tüm grafik kartlarını kullan ({ $count })
tour-render-all-gpus-hover = Bu bilgisayardaki bir işleme çiftliği aracılığıyla (--gpus all) bu makinenin tüm grafik kartlarında aynı anda işler. Kareler, adları, Sürdürme ve mp4 tek kartla olduğu gibidir; tüm tur, çiftliğin kendi sırasıyla işlenir. Farklı kart modelleri birkaç pikseli biraz farklı çizer: durağan bir çekim ve bir çapraz geçiş tek kartta kalır.
tour-render-order = İşleme sırası
tour-render-sequential = Sıralı
tour-render-sequential-hover = Kare 0, 1, 2, … sırayla (varsayılan).
tour-render-progressive = Aşamalı
tour-render-progressive-hover = Önce tüm turu önizle, ardından tam ayrıntıya iyileştir: önce anahtar kareler işlenir, ardından her kare var olana kadar en büyük zaman boşlukları ikiye bölünür. Kareler her iki durumda da doğru dizinlerine yerleşir, bu yüzden Sürdürme ve mp4 değişmeden çalışır — erken durdurabilir, kaba çevirmeli kitaba göz gezdirebilir ve her iki sırada da Sürdürme ile yeniden başlatabilirsiniz.
# The cost of the render. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
tour-render-summary =
    { $count ->
        [one] { $count } kare · { $seconds } s’lik tur · { $width }×{ $height } ss{ $ss }
       *[other] { $count } kare · { $seconds } s’lik tur · { $width }×{ $height } ss{ $ss }
    }
# The disk space the frames will take (an estimate). $size: a size ("5.3 GB").
tour-render-disk = ≈{ $size } kare verisi
# $size: the frames' estimated size; $free: the free space on that drive ("6.0 GB").
tour-render-disk-free = ≈{ $size } kare verisi · { $free } boş
# After a warning sign. $disk: the line above ("≈51.0 GB of frames · 6.0 GB free").
tour-render-disk-short = { $disk } — dizinin tamamı için yeterli yer yok
tour-render-separate = Betiği ayrı bir süreçte DİSKTEN işler; bu yüzden kaydedilmemiş düzenlemeler dahil edilmez — ve orada kaybedilen bir GPU aygıtı bu oturumu da beraberinde götüremez.
# On the progress bar. $done: frames done; $total: frames to render.
tour-render-progress =
    { $total ->
        [one] { $done } / { $total } kare
       *[other] { $done } / { $total } kare
    }
# Before the render's first progress line.
tour-render-starting = başlıyor…
tour-render-live-paused = Canlı görünüm duraklatıldı — işleme GPU’yu tek başına kullanır, böylece daha çabuk biter ve kendi kare maliyetlerini doğru ölçer.
tour-render-cancel = İptal
tour-render-cancel-hover = İşlemeden kapat
tour-render-copy-command = Komutu kopyala
tour-render-copy-command-hover = Eşdeğer komut satırını panoya kopyala
tour-render-stop = İşlemeyi durdur
tour-render-render = İşle
tour-render-render-hover = Kare dizisini işlemeye başla
# $folder: the output folder.
tour-render-finished = İşleme bitti → { $folder }
# $why: the render's own reason (in English).
tour-render-failed = İşleme başarısız: { $why }
# $last: the render's last progress line (in English), or nothing.
tour-render-failed-or-stopped = İşleme başarısız oldu ya da durduruldu. { $last }
# $why: the system's reason.
tour-render-no-exe = Çalıştırılabilir dosya bulunamıyor: { $why }
# $why: the system's reason.
tour-render-start-failed = İşleme başlatılamadı: { $why }

## The Custom formula dialog (Fractal ▸ Custom formula…)
## The formula language itself stays as it is in every language: function names (sin, cosh, conj…),
## operators, variables (z, c, p1), and the formulas. Keep them as written inside these messages.

# The window's title.
formula-dialog-title = Özel formül
# A small line at the top of the window, above the formula.
formula-dialog-intro = Adım, Fractint tarzı ifadelerle
# The two ways to show and edit the formula (top right of the window).
formula-dialog-textbook = Ders kitabı
formula-dialog-textbook-hover = Formülü bir ders kitabının dizeceği gibi dizilmiş olarak göster ve öyle düzenle
formula-dialog-text = Metin
formula-dialog-text-hover = Formülü metin olarak düzenle
# The tooltip of the "LaTeX" button (the button's own text stays LaTeX).
formula-dialog-latex-hover = Formülü burada dizildiği gibi LaTeX olarak kopyala
# The drop-down list of example formulas (the collection).
formula-dialog-examples = Örnekler
# Help under the formula in Textbook mode. The characters / ^ ( are what the user types: keep them.
formula-dialog-help-textbook = Metindeki gibi yazın: / önündeki terimden bir kesir, ^ bir üs, ( parantez oluşturur. Oklar formül içinde dolaşır; Tab bir sonraki boş kutuya gider. Kırmızı bir satır okunamıyor demektir: metin olarak düzenlemek için üzerine tıklayın.
# Help under the formula in Text mode. "; starts a comment": the ; is what the user types.
formula-dialog-help-text = Yazın ya da tuş takımını kullanın — formül dilinin bildiği her adı içerir. Adlar siz yazarken tamamlanır (Tab). Deyimler yeni bir satırla ya da virgülle ayrılır; ; bir yorum başlatır.
# The syntax check, when the formula reads.
formula-dialog-reads = Doğru okunuyor.
# The syntax check in Textbook mode, when the formula has an empty box to fill in.
formula-dialog-empty-box = Boş kutuyu doldurun.
# The syntax check in Textbook mode. $line: the line number; $message: what is wrong (from the formula parser, in English).
formula-dialog-error-line = Satır { $line }: { $message }
# One function written with another notation's name, and the language's name for it: "ln to log".
# $from: the name as written (ln); $to: the formula language's name (log).
formula-respell-pair = { $from } → { $to }
# A button that rewrites them. $changes: one or more formula-respell-pair, joined with ", ".
formula-respell = Değiştir: { $changes }
formula-respell-hover = Bu fonksiyonları formül dilinin adlandırdığı gibi yaz
# Shown when Apply finds a parameter field that is not a number. $name: the parameter (p1 … p5).
formula-param-not-numbers = { $name } bir sayı çifti değil
# The Apply button (also in the Formula library window).
formula-apply = Uygula
formula-dialog-apply-hover = Formülü derle ve göster
# Shown beside Apply while the formula is being compiled.
formula-dialog-compiling = Formül GPU için derleniyor…
# The greyed hint in an empty name box (also in the Formula library window).
formula-name-hint = ad (isteğe bağlı)
# The save button: "Update" when the library already has a formula of that name.
formula-dialog-update-in-library = Kitaplıkta güncelle
formula-dialog-save-to-library = Kitaplığa kaydet
formula-dialog-save-hover = Bu formülü ve parametrelerini formül kitaplığında sakla
formula-dialog-save-disabled-hover = Yalnızca doğru okunan bir formül kaydedilebilir
# Opens the Formula library window.
formula-dialog-library = Kitaplık…
formula-dialog-library-hover = Kaydedilmiş formüller: uygulayın, düzenleyin, içe ve dışa aktarın

## The Custom formula dialog's keypad
## The keys' captions are formula tokens and stay as they are; these are their tooltips.

# The keypad's third tab (the other two are "123" and "f(z)"). A short word.
formula-keypad-tab-names = adlar
# The tabs' tooltips.
formula-keypad-tab-basic-hint = Sayılar, işleçler ve en çok kullanılan adlar
formula-keypad-tab-functions-hint = Fonksiyonlar
formula-keypad-tab-names-hint = Her değişken, parametre ve sabit ile deyim sözdizimi
# The key z.
formula-key-z = Yineleme değeri
# The key c.
formula-key-c = Piksel (Julia modunda Julia sabiti)
# The key p1 on the first tab.
formula-key-p1-below = Parametre 1 — aşağıdaki alanlarda ayarlanır
# The keys p1 … p5. $n: the parameter's number.
formula-key-param = Parametre { $n }
formula-key-multiply = Çarp
formula-key-divide = Böl
# The key that types ^2.
formula-key-square-after = Kare (bir değerden sonra: z^2)
# The key that types ^.
formula-key-power = Kuvvet: tam sayı, gerçel ya da karmaşık üs
formula-key-square-root = Karekök
# The key that types |…|: in this language it is the squared modulus, as in Fractint.
formula-key-squared-modulus = Modülün KARESİ |z|² — Fractint’teki gibi
formula-key-add = Topla
formula-key-subtract = Çıkar (ya da işaretini değiştir)
# The key that types =.
formula-key-assign = Ata: z = … ya da geçici bir t = …
# The key that starts a new line (first tab).
formula-key-next-statement-comma = Sonraki deyim (virgül de olur)
# The key "del".
formula-key-backspace = İmleçten önceki karakteri (ya da seçimi) sil
# The key e.
formula-key-euler = Euler sayısı
# The key that types (0.5, 0.5).
formula-key-complex-constant = Karmaşık bir sabit: (gerçel, sanal)
# The key that types a comma.
formula-key-comma = Deyimleri ya da karmaşık bir sabitin kısımlarını ayırır
formula-key-left = İmleç sola
formula-key-right = İmleç sağa
# The function keys (the f(z) tab), one per function.
formula-key-sin = Sinüs
formula-key-cos = Kosinüs
formula-key-tan = Tanjant
formula-key-cotan = Kotanjant
formula-key-exp = Üstel e^z
formula-key-sinh = Hiperbolik sinüs
formula-key-cosh = Hiperbolik kosinüs
formula-key-tanh = Hiperbolik tanjant
formula-key-cotanh = Hiperbolik kotanjant
formula-key-log = Doğal logaritma, ln (esas dal)
formula-key-sqr = Kare: sqr(z) = z²
formula-key-sqrt = Karekök (esas)
formula-key-recip = Çarpmaya göre ters 1/z
formula-key-conj = Karmaşık eşlenik
formula-key-flip = Gerçel ve sanal kısımların yerini değiştir
formula-key-abs = HER İKİ kısmın mutlak değeri: |x| + i|y|
formula-key-cabs = Modül |z| (bir gerçel sayı)
formula-key-real = Gerçel kısım
formula-key-imag = Sanal kısım, gerçel bir sayı olarak
# "fn" is Fractint's name for a function slot: keep it.
formula-key-ident = Özdeşlik (Fractint’in fn yer tutucusu)
# The key "pixel".
formula-key-pixel = c ile aynı (Fractint’teki adı)
# The key "z =".
formula-key-set-z = z değerini ayarlayan bir deyim başlat
# The key "t =".
formula-key-temporary = Geçici: herhangi bir yeni ad olur (t, w, a2, …)
# The key "; …".
formula-key-comment = Satır sonuna kadar bir yorum
# The key that starts a new line (names tab).
formula-key-next-statement = Sonraki deyim

## The formula's completion list (names offered as you type)

# Beside a name the formula itself assigns.
formula-completion-variable = Bu formülün ayarladığı bir değişken
# Beside a function typed with another notation's name.
# $what: what the function is, in English (from the formula parser: "The natural logarithm");
# $other: the other notation's name (ln); $ours: the formula language's name (log).
formula-completion-other-spelling = { $what } — { $other } burada { $ours } olarak yazılır
# Under a list cut short. $count: the names not shown.
formula-completion-more = … { $count } tane daha
# The last line of the list. Tab, Enter and Esc are key names.
formula-completion-keys = Tamamlamak için Tab ya da Enter, kapatmak için Esc

## The Custom formula dialog's Textbook mode

# The tooltip of a line that does not read. $why: what is wrong (from the formula parser, in English, or formula-textbook-fractint-sections).
formula-textbook-unread-hover = Okunamıyor: { $why }. Metin olarak düzenlemek için tıklayın.
# Why a line with Fractint's sections is shown as text in Textbook mode.
formula-textbook-fractint-sections = Fractint’in bölümleri (başlangıç bölümünün “:” işareti, karşılaştırmalar, a = b = …) metin olarak düzenlenir

## The Formula library window (Fractal ▸ Formula library…)

# The window's title.
formula-library-title = Formül kitaplığı
# The window's two tabs. $count: the formulas in each.
formula-library-mine = Formüllerim ({ $count })
formula-library-mine-hover = Kaydettiğiniz ya da içe aktardığınız formüller
formula-library-collection = Koleksiyon ({ $count })
formula-library-collection-hover = Fractadyne ile birlikte gelen, her biri başlangıç için bir görünüme sahip formüller
# A small marker before the name of the formula the view shows now. A short word.
formula-library-showing = gösteriliyor
# The tooltip of Apply on a formula of the collection.
formula-library-apply-shipped-hover = Başlangıç görünümünde göster
# A row's Edit button (after a pencil icon).
formula-library-edit = Düzenle
formula-library-edit-hover = Uygulamadan Özel formül penceresinde aç
# A collection row's button that copies the formula into "My formulas".
formula-library-copy-to-mine = Benimkilere kopyala
formula-library-copy-to-mine-hover = Saklamak ya da değiştirmek için formüllerinize ekleyin
formula-library-close = Kapat
# Saves the formula the view shows, under the name typed beside it.
formula-library-add-current = Geçerli formülü ekle
formula-library-add-current-hover = Görünümün gösterdiği formülü kaydet
# "Fractal > Custom formula…" is the menu path: write it as the menus are named.
formula-library-add-current-disabled-hover = Önce özel bir formül uygulayın (Fraktal > Özel formül…)
formula-library-import = İçe aktar…
formula-library-import-hover = Bir formül dosyasındaki formülleri ekle; buradaki hiçbir şeyin yerine geçilmez
formula-library-export-all = Tümünü dışa aktar…
formula-library-export-all-hover = Buradaki her formülü, paylaşmak ya da saklamak için tek bir dosyaya yaz
# The filter box: its label, and the greyed hint inside it.
formula-library-filter = Süz
formula-library-filter-hint = ada ya da metne göre süz
# Shown while the user has no saved formulas.
formula-library-empty = Henüz kayıtlı formül yok. Özel formül penceresinden birini kaydedin, görünümün gösterdiğini ekleyin ya da bir dosya içe aktarın (bizimkini ya da Fractint’in .frm dosyasını).
# The tooltip of Apply on one of the user's formulas: with a view saved, and without one.
formula-library-apply-view-hover = Bu formülü, onunla birlikte kaydedilen görünümde göster
formula-library-apply-hover = Bu formülü göster
# The tooltip of a row's save icon.
formula-library-export-one-hover = Bu formülü bir dosyaya dışa aktar
# A row asks before deleting: the question, then its two buttons. "Delete" is also the delete icon's tooltip.
formula-library-delete-ask = Silinsin mi?
formula-library-delete = Sil
formula-library-keep = Koru
# Under a list cut short. $count: the formulas not listed.
formula-library-more = …ve { $count } tane daha: birini bulmak için süzgece yazın.
formula-library-no-match = Süzgeçle eşleşen yok.
# Under a saved formula this version cannot read. $why: what is wrong (from the formula parser, in English).
formula-library-unread = Bu sürümde okunamıyor: { $why }

## Formula library notices

# $why: the system's reason, in English.
formula-library-save-failed = Formül kitaplığı kaydedilemedi: { $why }
formula-library-nothing-to-save = Kaydedilecek bir şey yok: formül boş.
# $name: the formula's name in the library.
formula-library-updated = Formül kitaplığında güncellendi: “{ $name }”.
formula-library-saved = Formül kitaplığına kaydedildi: “{ $name }”.
# $name: the formula's name; $why: what is wrong.
formula-library-apply-failed = “{ $name }” uygulanamıyor: { $why }
# Copying a formula of the collection into "My formulas". $name: its name; $as_named: the name it was given (the same, or "Name (2)" when the name was taken).
formula-library-copy-exists = “{ $name }” zaten formülleriniz arasında.
formula-library-copied = “{ $name }”, formüllerinize “{ $as_named }” adıyla kopyalandı.
# $name: the formula's name.
formula-library-deleted = Formül kitaplığından silindi: “{ $name }”.
# Why the library could not be saved (follows formula-library-save-failed).
formula-library-no-config-dir = yapılandırma klasörü yok
# At startup, when the library file cannot be read. $why: the reason, in English; $path: a file's full path.
formula-library-unreadable-kept = Formül kitaplığı okunamadı ({ $why }). Şu adla saklandı: { $path }; yeni, boş bir kitaplık başlatıldı.
formula-library-unreadable-stuck = Formül kitaplığı okunamadı ({ $why }) ve kenara da taşınamadı: bir formül kaydetmeden önce şu dosyayı güvenli bir yere kopyalayın, çünkü kaydetme onun yerine geçer: { $path }

## Formula library: Import and Export
## A formula file's contents stay in English; this is what the windows and notices around them say.

# File-type names in the open and save dialogs (the extensions are added by the system).
formula-filter-formula-files = Formül dosyaları
formula-filter-fractadyne = Fractadyne formülleri
formula-filter-fractint = Fractint formülleri
formula-filter-all-files = Tüm dosyalar
# Why a file cannot be imported (they follow formula-import-failed).
formula-import-too-large = bir formül dosyası olamayacak kadar büyük
formula-import-not-text = bir metin dosyası değil
formula-import-no-formulas = içinde formül yok
# $why: the TOML reader's message, in English.
formula-file-not-formulas = bir formül dosyası değil ({ $why })
# $format: what the file says it is ("fractadyne-gradients").
formula-file-other-format = bir “{ $format }” dosyası, formül dosyası değil
# "[[formula]]" is the file's own syntax: keep it.
formula-file-no-formulas = içinde formül yok (bir formül dosyası onları [[formula]] tabloları olarak listeler)
# $file: the file's name; $why: one of the reasons above, or the system's (in English).
formula-import-failed = “{ $file }” içe aktarılamadı: { $why }.
# After an import that added nothing. $file: the file's name.
formula-import-nothing-new = “{ $file }” içinde yeni bir şey yok.
# After an import. $count: the formulas added; $file: the file's name.
formula-import-added =
    { $count ->
        [one] “{ $file }” dosyasından { $count } formül içe aktarıldı.
       *[other] “{ $file }” dosyasından { $count } formül içe aktarıldı.
    }
# The same two, followed by what else happened. $notes: one or more of the formula-import-note-… messages below, joined with "; ".
formula-import-nothing-new-notes = “{ $file }” içinde yeni bir şey yok: { $notes }.
formula-import-added-notes =
    { $count ->
        [one] “{ $file }” dosyasından { $count } formül içe aktarıldı: { $notes }.
       *[other] “{ $file }” dosyasından { $count } formül içe aktarıldı: { $notes }.
    }
# Notes (clauses, no final full stop). $count: the formulas already in the library, skipped.
formula-import-note-duplicates =
    { $count ->
        [one] { $count } formül zaten kitaplıktaydı
       *[other] { $count } formül zaten kitaplıktaydı
    }
# One formula was renamed because its name was taken. $from: its name in the file; $to: the name it was given ("Name (2)").
formula-import-note-renamed-one = “{ $from }” adı alınmıştı, bu yüzden adı “{ $to }” oldu
# $count: the formulas renamed because their names were taken (2 or more).
formula-import-note-renamed = adı alınmış olan { $count } formülün adı değiştirildi
# $count: the entries skipped as empty or too long.
formula-import-note-skipped =
    { $count ->
        [one] { $count } girdi boş ya da çok uzundu, atlandı
       *[other] { $count } girdi boş ya da çok uzundu, atlandı
    }
# A second sentence after importing a Fractint .frm file. $count: the entries that do not read; $why: the commonest reason (from the formula parser, in English).
formula-import-frm-unread =
    { $count ->
        [one] { $count } girdi bu sürümde okunamıyor (en sık neden: { $why }).
       *[other] { $count } girdi bu sürümde okunamıyor (en sık neden: { $why }).
    }
# After exporting one formula. $name: the formula's name; $file: the file's name.
formula-export-one = “{ $name }”, “{ $file }” dosyasına dışa aktarıldı.
# After exporting several. $count: how many; $file: the file's name.
formula-export-many =
    { $count ->
        [one] { $count } formül “{ $file }” dosyasına dışa aktarıldı.
       *[other] { $count } formül “{ $file }” dosyasına dışa aktarıldı.
    }
# $file: the file's name; $why: the system's reason, in English.
formula-export-failed = “{ $file }” yazılamadı: { $why }

## The control panel (right-hand side): header and section headings

# The panel's title, beside the button that hides it.
panel-title = Denetimler
panel-hide-hover = Denetim panelini gizle
# The button that brings the hidden panel back (top right of the view).
panel-show-hover = Denetim panelini göster
# Section headings.
panel-section-navigate = Gezinme
panel-section-coloring = Renklendirme
panel-section-quality = Kalite
panel-section-effects = Efektler
panel-section-overlays = Kaplamalar
panel-section-advanced = Gelişmiş
# The last section: reference notes about the current fractal. $name: the fractal's name (Mandelbrot, Burning Ship…).
panel-section-about = { $name } hakkında
panel-section-performance = Performans

## The control panel: Navigate
## Labels before a control sit in a column about 104 px wide (roughly 15 characters): a longer label pushes its control to the right.

panel-zoom-speed = Yakınlaştırma hızı
panel-zoom-speed-hover = Space basılı tutularak yapılan sürekli yakınlaştırmanın hızı (1× ≈ her 1,5 s’de 2×).
panel-snap-zoom = Yakınlaştırmayı özelliklere çek
panel-snap-zoom-hover = Fare tekerleği ya da Space ile yakınlaştırırken yakınlaştırmayı imlecin yakınındaki bir minibrota (ya da gerçel eksene) doğru çeker; böylece özellik imlecin altına kayar ve yakınlaştırma onun üzerine iner. Özellik ne kadar yakınsa çekim o kadar güçlüdür. Yalnızca Mandelbrot, tek görünüm.
panel-click-zoom = Tıklayarak yakınlaştır
panel-click-zoom-hover = Açıkken görünümdeki bir sol tık, tıklanan noktayı merkeze alarak aşağıdaki çarpanla içeri dalar (sağ tık geri çıkar). Sürükleme yine kaydırır; Shift+sürükleme / sağ sürükleme yine kutuyla yakınlaştırır. Backspace bir tıklamayı geri alır. İkili görünümde her iki panelde de çalışır ve bu açıkken Julia c’yi sabitleme Ctrl+tık’a geçer. Tıklamanın sizi götüreceği görünümü gösteren bir büyüteci imlecin altında açmak için Shift’i basılı tutun; o noktayı almak için Shift basılıyken tıklayın; Shift+SÜRÜKLEME yine kutuyla yakınlaştırır.
# The click-to-zoom factor, followed by buttons 2×, 4×, 10×…
panel-click-zoom-factor = Çarpan
panel-click-zoom-snap = En yakın merkeze oturt
panel-click-zoom-snap-hover = Atlamadan sonra en yakın minibrot çekirdeğini çözer ve tam olarak onun üzerine oturur; böylece özelliğin yakınına değil, üzerine inersiniz. Yakınlaştırma değiştirilmez — onu tıklama seçti. Yalnızca Mandelbrot. Çözüm arka planda çalışır: görünüm hemen atlar ve yanıt geldiğinde kendini düzeltir; bu, aşırı derinlikte birkaç saniye sürebilir.

## The control panel: Coloring
## Labels before a control sit in a column about 104 px wide (roughly 15 characters).

panel-method = Yöntem
panel-method-hover = Kaçış verisinin renge nasıl eşlendiği. Şerit / üçgen eşitsizliği / yörünge tuzağı / ayrışım yörünge yapısını ortaya çıkarır; uzaklık, sınıra yakınlığa göre gölgelendirir.
panel-stripe-density = Şerit yoğunluğu
panel-stripe-tail = Yalnızca kuyruk (derin yakınlaştırma)
panel-stripe-tail-hover = Şerit terimini tüm yörünge yerine her yörüngenin son N yinelemesi üzerinden (üstel bir pencereyle) ortalar. Aşırı derinlikte her yörünge, son birkaç yüz yinelemesi dışında referansı izler; bu yüzden tüm yörüngenin ortalaması bütün görünüm için tek bir renk olur; kuyruk ise kontrastı korur. Pencereden kısa yörüngeler etkilenmez. --render için --stripe-tail N de vardır.
panel-tail-length = Kuyruk uzunluğu
# The unit after the tail length's number: short for "iterations" (the slider reads "64 it").
panel-tail-length-unit = yin.
panel-tail-length-hover = Yineleme cinsinden pencere uzunluğu. Kısa pencereler (8–32) her derinlikte klasik kalın bantları verir; daha uzun olanlar yeniden düz bir renge doğru ortalanır, çünkü uzun, kaotik bir yörüngenin şerit terimi ortalamada birbirini götürür.
panel-trap-shape = Tuzak şekli
panel-palette = Palet
# The two colours of the Binary palette, each beside its colour swatch.
panel-in-set = Küme içi
panel-out-of-set = Küme dışı
# The two colours of the Duotone palette, each beside its colour swatch.
panel-shadow = Gölge
panel-highlight = Vurgu
panel-cycle = Döngü
panel-offset = Kaydırma
panel-normalize-hover = Aşırı derinlikte kaçış sayıları yüz binlerce değere yayılır ve sabit bir Döngü paleti komşu pikseller arasında binlerce kez başa sarar — doğru, yoğun bir alan benekli gürültü gibi görünür. Bu seçenek, görünüm gerçekten örtüştüğü (aliasing) ZAMAN paleti görünümün ölçülen kaçış aralığına yeniden eşler (Döngü bu durumda aralığı kaç palet geçişinin kaplayacağını belirler) — yumuşak bir görünüme dokunulmaz, çünkü onu yeniden eşlemek resmi boş yere değiştirirdi. Her durumda yeniden eşlemek için aşağıdaki “Her zaman aralığa sığdır” kutusunu işaretleyin. Yalnızca yumuşak yineleme yöntemi; --normalize ile aynıdır.
# A checkbox indented under "Normalize deep colors" (also in the Color menu).
panel-fit-range = Her zaman aralığa sığdır
panel-fit-range-hover =
    Görünüm örtüşmediğinde bile paleti görünümün ölçülen kaçış aralığına yeniden eşler — yukarıdaki kutu yalnızca yoğun bir alan aksi hâlde benek gibi görünecekse yeniden eşler.

    Paletin gürültüyü gidermek yerine ekranda olanı kapsamasını istediğinizde bunu kullanın. Yine de bir ölçüm gerektirir; bu yüzden görünüm durulmadan önceki bir iki anda hiçbir şey yapmaz, kaçan pikseli olmayan bir görünümde ise hiç etkisi olmaz.
# A checkbox indented under "Always fit range" (also in the Color menu).
panel-log-scale = Logaritmik renk ölçeği
panel-log-scale-hover =
    Paleti doğrusal yerine kaçış değerinin LOGARİTMASINA göre yayar. Derinde kaçış sayıları üst uçta yığılır, bu yüzden doğrusal bir eşleme paletin çoğunu sınırın yakınındaki ince bir kabuğa harcar ve geri kalan her şeyi düzleştirir.

    Normalleştirilmiş eşlemeye uygulanır — bunu “Derin renkleri normalleştir” (ya da bir dışa aktarmada --normalize) açar. Çok çarpık bir aralık, bu kutu işaretli olsun ya da olmasın zaten logaritmik eşlemeyi kullanır.
# The same checkbox's tooltip while it is greyed out (normalization is off).
panel-log-scale-off-hover =
    Normalleştirme gerektirir: bu seçenek paletin ölçülen kaçış aralığına NASIL eşleneceğini belirler; bu yüzden “Derin renkleri normalleştir” kapalıyken (ve --normalize ile dışa aktarma yokken) değiştireceği bir eşleme yoktur ve tek bir pikseli bile değiştirmez.

    Kullanmak için “Derin renkleri normalleştir” seçeneğini açın.
# The palette animation: its kind, then its speed.
panel-animate = Animasyon
panel-anim-speed = Hız
panel-anim-speed-hover = Döngü hızı: saniyede renk kaydırma döngüsü ya da (Rastgele) saniyede gradyan değişimi.
panel-shuffle-gradient = Gradyanı karıştır

## The control panel: Quality

panel-auto-iter = Yinelemeleri yakınlaştırmayla otomatik ölçekle
# The iteration-count label while auto-scaling is on (the count is then a base the zoom raises).
panel-iterations-base = Yineleme (temel)
panel-iterations = Yineleme
panel-iterations-hover = Temel yineleme sayısı. Otomatik ölçekleme açıkken etkin sayı yakınlaştırma derinliğiyle artar. Hareket ederken önizleme, akıcılık için yinelemeleri düşük bir değerle (50.000) sınırlar, ardından görünüm durulunca tam sayıya keskinleşir — böylece derin kenarlar hareket sırasında yumuşak görünür ve durduğunuzda netleşir.
# A warning under the iteration count. $count: the iterations the settled preview renders.
panel-res-limited = Bu derinlikte durulmuş önizleme, akıcı kalmak için { $count } yinelemeyi düşük çözünürlükte işler. Tam çözünürlüklü ayrıntı için dışa aktarın.
panel-anti-alias = Kenar yumuşatma
# Anti-aliasing off (the other choices are 2×, 3×, 4×, 8×).
panel-anti-alias-off = Kapalı
panel-anti-alias-hover = Durağan görüntüler için süper örnekleme (görünüm durulunca uygulanır). Daha yüksek değer, işleme süresi pahasına dıştaki ince “tozu” bastırır.

## The control panel: Effects

panel-relief-lighting = 3B kabartma aydınlatması
panel-relief-lighting-hover = Yüzeyi uzaklık tahmininin eğimiyle gölgelendirir — kabartmalı, ışıklı bir görünüm. (Holomorf aileler: Mandelbrot / Multibrot.)
panel-light-angle = Işık açısı
panel-relief = Kabartma
panel-relief-hover = Düşük = daha keskin kabartma; yüksek = daha yumuşak/düz.
panel-rotate-light = Işığı döndür
panel-rotate-light-hover = Işık yönünü zamanla döndürür (Hız kaydırıcısını kullanır).
panel-distance-glow = Uzaklık parıltısı
panel-distance-glow-hover = Sınırın yakınında yoğunlaşarak parlayan ipliklere dönüşen parlak uzaklık tahmini kontur bantları. (Holomorf aileler.)
panel-glow = Parıltı
panel-band-width = Bant genişliği
panel-band-width-hover = Uzaklık konturlarının aralığı (bant başına oktav).
panel-animate-glow = Parıltıyı canlandır
panel-animate-glow-hover = Parıltı bantlarını zamanla akıtır (Hız kaydırıcısını kullanır).

## The control panel: Overlays

panel-orbits-hover = İmlecin altındaki noktanın yineleme yolunu çizer.
panel-orbit-anim = Canlandır (yarışan nokta)
panel-orbit-anim-hover = Yörünge boyunca renk değiştiren bir noktayı yarıştırır.
panel-orbit-speed = Yörünge hızı
panel-minimap-hover = “Buradasınız” işaretiyle ve yakınlaştırma derinliğiyle küçük bir başlangıç görünümü özeti. Bir bölgeye atlamak için tıklayın.

## The control panel: Advanced
## Labels before a control sit in a column about 104 px wide; most of these are already wider in English.

# Small headings of the section's two groups.
panel-accelerators = Hızlandırıcılar
panel-performance-tuning = Performans ayarı
panel-bla = BLA hızlandırma (derin yakınlaştırma)
panel-bla-hover = Çift doğrusal yaklaşım: aşırı derinlikte yörünge boyunca yinelemeleri atlar (floatexp Mandelbrot, ≥1e28×) — ~5× daha hızlı GPU işlemesi, aynı çıktı (öz sınama ile doğrulanmıştır). Varsayılan olarak açık; karşılaştırmak için ya da bir görüntü kusuruyla karşılaşırsanız kapatın.
panel-series-approx = Seri yaklaşımı
panel-series-approx-hover = BLA’nın etkin olmadığı yerlerde (df32 derinlikleri, Multibrot, BLA kapalı) ilk yinelemeleri atlamak için pertürbasyonu bir polinomdan başlatır. Çıktı aynıdır; karşılaştırmak için kapatın.
panel-glitch-correct = Bozulma düzeltme (dışa aktarma)
panel-glitch-correct-hover = Dışa aktarılan görüntüler için çoklu referanslı bozulma düzeltme: pertürbasyon bozulmalarını algılar ve bu pikselleri temiz olana kadar ek referanslara karşı yeniden işler. Varsayılan olarak açık. ~32 MP’ye kadar dışa aktarmalar (aux olmayan renklendirme); daha büyük görüntüler ve canlı görünüm düz yolu kullanır. Mandelbrot ya da Multibrot görünümlerinde (Julia modu dışında) hiç kullanılmaz: kesin bir hesaplamayla karşılaştırıldığında orada hiçbir şeyi düzeltmedi ve dışa aktarma süresinin 5 katına kadar zaman aldı.
panel-dive-limit = Otomatik yakınlaştırma dalış sınırı
panel-dive-limit-hover = Otomatik yakınlaştırmanın (A tuşu) durduğu derinlik. ~1e271× düzeyine kadar akıcı biçimde süzülür; daha derinde, aşırı derinliğe hızla ulaşmak için kesik kesik, basamaklı bir dalışa geçer.
# Followed by two choices, Speed and Quality.
panel-autopilot-priority = Otomatik yakınlaştırma önceliği
panel-priority-speed-hover = Ayarlanan hızda yakınlaştırır. Ayrıntı içeren bir kare hazır olur olmaz ekran yenilenir; yenilemelerin gelmesini sağlayan çözünürlük hangisiyse o kullanılır (yakınlaştırma işlemeyi geride bırakırsa En düşük hareket çözünürlüğü tabanının altına da inilir); yenilemeler arasında son kare yakınlaştırmayı izler ve yumuşar.
panel-priority-quality-hover = Her yenileme tamamen çözümlenmiş, tam çözünürlüklü bir karedir ve yakınlaştırma gerektiği kadar yavaşlar; böylece ekrandaki kare, bir sonraki gelmeden asla çok fazla büyümez. Her an keskin olan, daha yavaş bir dalış.
# Followed by two choices, Detail and Misiurewicz point.
panel-autopilot-target = Otomatik yakınlaştırma hedefi
panel-target-detail-hover = Görünümde, fraktalın kenarındaki en zengin yapıya doğru dalar; dalış ilerledikçe hedefi yeniden seçer.
panel-target-misiurewicz-hover = En yakın Misiurewicz noktasını (yapısı sonsuza dek tekrarlanan bir spiral merkezi) bulur ve tam içine dalar. Dalış, nokta dalış sınırının derinliğine kadar çözülürken önce yerinde bekler — sıradan derinliklerde bir an, 1e1000× ötesinde daha uzun. Yalnızca Mandelbrot kümesi.
panel-work-budget = Canlı işleme bütçesi
panel-work-budget-hover = Derin yakınlaştırmada canlı görünüm için ayrıntıya karşı hız. Daha yüksek değer daha tam çözünürlükte işler (daha keskin — “yumuşak”, büyütülmüş görünüm azalır), ancak kare hızını düşürür ve çok yüksek değerler en ağır karelerde kısa bir GPU takılması riski taşır. Dışa aktarmalar bundan bağımsız olarak her zaman tam çözünürlüktedir.
panel-min-motion-res = En düşük hareket çözünürlüğü
panel-min-motion-res-hover = Derin yakınlaştırmada sürekli yakınlaştırırken ya da kaydırırken canlı görünümün inebileceği en düşük çözünürlük. Daha yüksek değer hızlı bir dalışın bloklu/pikselli olmasını önler (daha keskin), ancak kare hızı pahasına — %100’de derin dalışın ayrıntı yenilemeleri tam maliyetle işlenmek zorundadır, bu da derin yakınlaştırmanın gözle görülür biçimde basamaklı/takılarak ilerlemesine yol açabilir. Derin bir dalış sarsıntılı geliyorsa bunu düşürün. Varsayılan %30; durulmuş görüntüyü ve dışa aktarmaları etkilemez.
panel-finish-sound = İşleme bitince ses çal
panel-finish-sound-hover = Bir dışa aktarma ya da tur işlemesi bittiğinde FRACTINT’in tamamlanma ezgisini çalar: üç yükselen nota (her biri 100 ms boyunca 1047, 1109, 1175 Hz), tam olarak DOS orijinalinin kodladığı gibi — bant sınırlı bir kare dalga olarak sentezlenir ve onu çalan küçük PC hoparlörü gibi duyulsun diye yüksek geçiren bir filtreden geçirilir.
panel-prefer-detail = Yakınlaştırırken ayrıntıyı tercih et
panel-prefer-detail-hover = Yakınlaştırırken ya da kaydırırken, her karede düşük kalitede yeniden işlemek yerine son tam ayrıntılı kareyi — hareketi izleyecek şekilde ölçeklenip kaydırılarak — göstermeye devam eder. Hareket durduğunda görünüm tam olarak işlenir. Kapalı: hareket düşük çözünürlükte canlı işlenir (daha akıcı, daha kaba). Sığ görünümler (doğrudan mod) her zaman canlı işlenir — her iki durumda da ucuzdurlar ve her karede keskindirler.

## The control panel: About (the current fractal)

# A link to the fractal's reference page, followed by an arrow.
panel-about-reference = Başvuru

## The control panel: Advanced ▸ Second graphics card

# A label before a drop-down (wider than the panel's label column already).
gpu2-label = İkinci grafik kartı
gpu2-off = Kapalı
# Another device on the same graphics card as the window (the default).
gpu2-same-card = Aynı kart
# A card the list does not name. $number: the card's number.
gpu2-card-n = Kart { $number }
gpu2-finding = Bu makinenin grafik kartları aranıyor…
gpu2-none = Bu makinede başka grafik kartı yok.
gpu2-hover =
    Başka bir grafik aygıtı, derin bir görünüm hareket ederken karelerini ve durduktan sonra onu keskinleştiren ek örneklerin bir kısmını işler; böylece resim yakınlaştırmaya yetişir ve son kalitesine daha çabuk ulaşır. Pencereyi çizen kart yine her şeyi çizer.

    Başka bir kart kendi hızını ekler. İki farklı kart modeli birkaç pikseli biraz farklı çizer; ikinci kart başarısız olursa görünüm tek kartla devam eder.

    “Aynı kart” (varsayılan) bu kartta ikinci bir aygıt açar. Hız eklemez ama ekranın karelerini beklemeden işler. Kartı paylaşır: kartı sıfırlayan bir arıza, pencerenin kendi işlemesindeki bir arızanın yaptığı gibi uygulamayı sonlandırır ve bir sonraki başlangıçta onu kapatmak önerilir.

## The view (canvas)

# A notice after Ctrl+click pins the Julia c at a deep zoom.
canvas-julia-pin-quantized = Julia c sabitlendi — ancak çift hassasiyette tutulduğundan bu derinlikte tüm panel tek bir değere yuvarlanır ve sabitleme görünüm merkezine düşer. Nişan almak için uzaklaştırın.
# A small caption in the top-left corner of the view while the orbit overlay is fitted to the view (its "Normalize (fit to view)" option).
canvas-orbit-normalized = yörünge (normalleştirilmiş)
# The minimap's tooltip.
canvas-minimap-hover = Genel bakış / buradasınız. Geçerli yakınlaştırmada kaydırmak için sürükleyin, yakınlaştırıp uzaklaştırmak için tekerleği çevirin, o bölgeye atlamak için tıklayın (başlangıç yakınlaştırması).

## The timestamp overlay (View ▸ Show timestamp)

# Under the large elapsed-time clock. $frame: the frame number (as the log writes it, without grouping).
overlay-timestamp-frame = kare { $frame }

## File ▸ Settings ▸ Reference cache…

cache-title = Referans önbelleği
cache-intro = Aşırı derinlikte bir görünümün yavaş kısmı referans yörüngesidir — bir saate kadar süren keyfi hassasiyetli aritmetik. Fractadyne bitmiş yörüngeleri burada tutar; böylece ziyaret ettiğiniz bir konuma dönmek ya da yakınında bir yere yakınlaştırmak bunun yerine saniyeler sürer. Bir yörünge yaklaşık 4 MB’tır.
cache-off = Bu çalıştırma için kapalı (bir görev çağrısı ya da --no-orbit-cache). Bir sonraki olağan açılışa kadar hiçbir şey okunmaz ya da yazılmaz.
# Small headings of the window's parts.
cache-location = Konum
cache-usage = Kullanım
cache-limit = Sınır
cache-no-config = (kullanılabilir yapılandırma klasörü yok)
cache-open-folder = Klasörü aç
# The text on the usage bar. $used and $limit: sizes such as "1.2 GB"; $count: the orbits kept.
cache-usage-bar =
    { $count ->
        [one] { $used } / { $limit } — bir yörünge
       *[other] { $used } / { $limit } — { $count } yörünge
    }
cache-limit-hover = Önbellek bunu aştığında önce oluşturması EN UCUZ olan yörüngeler kaldırılır — oluşturması bir saat süren derin bir yörünge, ne kadar yakın zamanda kullanılmış olurlarsa olsunlar, hızlı oluşturulan yörüngelerin hepsinden uzun yaşar.
# Beside the limit. $count: about how many orbits of the live view's length fit in it.
cache-limit-estimate =
    { $count ->
        [one] ≈ canlı sınırda bir yörünge
       *[other] ≈ canlı sınırda { $count } yörünge
    }
cache-kept = Saklananlar (en değerli önce)
# Column headers of the list of kept orbits.
cache-col-iterations = yineleme
cache-col-precision = hassasiyet
cache-col-size = boyut
cache-col-last-used = son kullanım
# The precision column. $bits: the precision in bits.
cache-row-bits = { $bits } bit
# Under the list. $count: the orbits not listed.
cache-rows-more = … ve { $count } tane daha
# When an orbit was last used (the "last used" column).
cache-ago-now = az önce
# $count: minutes.
cache-ago-minutes = { $count } dk önce
# $count: hours.
cache-ago-hours = { $count } sa önce
# $count: days.
cache-ago-days = { $count } g önce
# Asked before clearing. $count: the orbits; $size: their total size, such as "1.2 GB".
cache-confirm =
    { $count ->
        [one] Bir yörünge silinsin mi ({ $size })? Yeniden oluşturmanın bedeli veri değil, zamandır.
       *[other] { $count } yörünge silinsin mi ({ $size })? Yeniden oluşturmanın bedeli veri değil, zamandır.
    }
cache-delete = Sil
cache-cancel = İptal
cache-clear = Önbelleği temizle…
cache-clear-hover = Saklanan her yörüngeyi siler. Önce sorar.
cache-close = Kapat
# $count: the orbits removed.
cache-cleared =
    { $count ->
        [one] Referans önbelleği temizlendi — bir yörünge kaldırıldı.
       *[other] Referans önbelleği temizlendi — { $count } yörünge kaldırıldı.
    }
# $why: the system's error message.
cache-clear-failed = Referans önbelleği temizlenemedi: { $why }

## Help ▸ Diagnostics…

diag-title = Tanılama
diag-intro = Fractadyne’ın donanımınızda doğru çalıştığını denetlemek için bir test çalıştırın. Sonuçlar makinenize kaydedilir ve bir sorun raporuna eklenebilir — kendiliğinden hiçbir yere hiçbir şey gönderilmez.
# $status: which arbitrary-precision library is in use, as the program reports it (in English).
diag-arithmetic = Derin yakınlaştırma aritmetiği: { $status }
# The three tests' buttons, each followed by a sentence saying what it does.
diag-run-selftest = Öz sınamayı çalıştır
diag-run-uitest = Arayüz testini çalıştır
diag-run-gputest = GPU aritmetik denetimini çalıştır
diag-blurb-selftest = GPU’nuzdaki matematiği ve işlemeyi doğruluğu bilinen sonuçlarla karşılaştırarak denetler. Yaklaşık 15 saniye sürer.
diag-blurb-uitest = Arayüzü ve canlı görünümü dolaşır, birkaç yakınlaştırma derinliğinde ekran görüntüleri yakalar. Bir iki dakika sürer ve çalışırken pencereler açar.
diag-blurb-gputest = Gölgelendiricinin çift-float aritmetiğini makinenizin sunduğu her grafik arka ucunda, doğruluğu bilinen değerlerle karşılaştırarak denetler. Birkaç saniye sürer, pencere açmaz. Bir başarısızlık bildirirse bu, görmek istediğimiz gerçek bir bulgudur — lütfen bir rapora ekleyin.
# While a test runs. $count: the checks finished so far.
diag-checks-done =
    { $count ->
        [one] bir denetim bitti
       *[other] { $count } denetim bitti
    }
# A finished test's result. $test: the test's name (Self-test, UI test, GPU arithmetic check).
diag-verdict-captured = { $test } — sonuç kaydedildi
diag-verdict-passed = { $test } — geçti
diag-verdict-problems = { $test } — sorun bildirdi
# Under the GPU arithmetic check's result.
diag-informational-note = Bunun geçti ya da kaldı sonucu yoktur: GPU’nuzun nasıl hesap yaptığını kaydeder ve bildirilen bir başarısızlık, makinenizdeki bir arıza değil gerçek bir bulgudur. Her durumda, onu bir rapora eklemek onunla yapabileceğiniz en yararlı şeydir.
# Under a test's result when it reported problems.
diag-problems-note = Referans GPU dışındaki donanımlarda bazı farklar beklenir — özellikle görüntü karşılaştırmalarında. Bunu bir sorun raporuna eklemek, onunla yapabileceğiniz en yararlı şeydir.
diag-open-results = Sonuçları aç
diag-attach = Sorun raporuna ekle…
diag-attach-hover = Rapor penceresini bu sonuç eklenmiş olarak açar; böylece sorun kaydı yalnızca bir açıklama değil, makine tarafından doğrulanmış bir test taşır
diag-console = Tanılamayı konsola yazdır
diag-console-hover = Bundan sonra [fd-…] satırlarını stderr’de gösterir. Fractadyne bağımsız değişken olmadan başlatıldığında varsayılan olarak kapalıdır. Bu, KAYDEDİLENLERDE hiçbir şeyi değiştirmez — günlük dosyası her zaman her satırı alır — bu yüzden bir terminali izlemiyorsanız kapalı bırakın. Yalnızca bu oturum için geçerlidir; başlangıcı da yakalamak için --console seçeneğini ya da FRACTADYNE_CONSOLE=1 kullanın.
diag-close = Kapat
# Errors starting a test. $why: the system's error message; $dir: a folder.
diag-error-no-exe = Çalıştırılabilir dosya bulunamıyor: { $why }
diag-error-create-dir = { $dir } oluşturulamıyor: { $why }
diag-error-start = Test başlatılamadı: { $why }

## Navigate ▸ Misiurewicz explorer…

misi-title = Misiurewicz gezgini
misi-mandelbrot-only = Misiurewicz noktalarına göz atma yalnızca Mandelbrot içindir — kullanmak için fraktalı değiştirin.
# While the gallery is being built. $seconds: the time so far, such as "3".
misi-solving = Küçük tür aileleri çözülüyor ({ $seconds } s)…
# Above the gallery. $total: all points; $curated: the hand-picked ones; $found: the ones the sweep found.
misi-summary = { $total } nokta — { $curated } tanesi seçilmiş, { $found } tanesi küçük türlerin tohumlu bir Newton taramasıyla bulunmuş (üst yarı düzlem; kaba bir ızgara havzaların çoğunu bulur, hepsini bulduğu kanıtlanamaz).
# The name of a point the sweep found. $kind: its type, such as "(3,1)"; $point: its coordinate, such as "-0.1011+0.9563i".
misi-entry-name = { $kind }, { $point } noktasında
# Under each thumbnail. $kind: the point's type, such as "(3,1)"; $octaves: the zoom period in octaves, such as "3.17"; $twist: the twist per repeat in degrees, such as "+12.5". "oct" is short for octaves.
misi-entry-readout = { $kind } · { $octaves } okt/tekrar · { $twist }°
# Before the depth field.
misi-zoom-to = Yakınlaştır:
misi-cancel = İptal
misi-solve-jump = Çöz ve atla
misi-solve-jump-hover = Bu tam noktayı, istenen derinliğin gerektirdiği hassasiyete kadar Newton yöntemiyle çözer (saklanan koordinatını başlangıç değeri olarak kullanarak), ardından doğrudan oraya atlar. Derin istekler daha uzun sürer — çözüm keyfi hassasiyetlidir.
misi-select-hint = Bir nokta seçin, bir derinlik belirtin ve atlayın — her küçük resmin altındaki çarpan satırı, oraya bir dalışın nasıl görüneceğini söyler: görünüm tekrarlanmadan önce kaç oktav geçtiğini ve her tekrarda ne kadar büküldüğünü.
misi-cancelled = Çözüm iptal edildi.
# $text: what was typed in the depth field.
misi-bad-zoom = “{ $text }” bir büyütme olarak okunamıyor (1e50, 3.2e120, … biçiminde yazın).
misi-no-kp = Bu girdinin çözüm için algılanmış bir (k,p) değeri yok.
misi-parse-failed = Saklanan koordinat ayrıştırılamadı.
# A notice after the jump. $name: the point's name; $zoom: the magnification, such as "1e50"; $seconds: how long the solve took, such as "1.2".
misi-zoomed = Yakınlaştırıldı: { $name } — { $zoom }× ({ $seconds } s)
# Added after the notice above when the iteration count is too low. $have: the fixed iteration count; $typical: what the depth typically needs.
misi-iter-note = Yineleme sayısı { $have } değerine sabitlenmiş; bu derinlik genellikle ~{ $typical } gerektirir — görünüm düz işlenirse yükseltin (ya da otomatik yinelemeyi açın).
# $name: the point's name; $why: why the solve failed (a technical word).
misi-solve-missed = Derin çözüm { $name } noktasına ulaşmadı: { $why }. Saklanan nokta kendi derinliğinde hâlâ kesindir — daha sığ bir hedef deneyin.

## Life (Game of Life): the side panel

# Shown when the graphics card cannot run the Life view.
life-no-compute = Bu grafik bağdaştırıcısı Life görünümünü çalıştıramıyor (hesaplama gölgelendiricisi yok).
# Row label: the pattern list (pattern names stay in English).
life-pattern = Desen
life-open = Aç…
life-open-hover = Bir RLE, .cells ya da Life 1.05/1.06 desen dosyası aç.
life-pattern-text = Desen metni…
life-pattern-text-hover = Bir desen yapıştırın (LifeWiki’nin verdiği biçimde RLE ya da düz metin).
life-save = Kaydet…
life-save-hover = Evreni şu anki hâliyle RLE olarak kaydet.
# Row label: the rule list and the typed rule (B3/S23 notation).
life-rule = Kural
# Shown in the rule list when the rule in use is none of the library's.
life-rule-custom = özel
life-apply = Uygula
# Playback buttons (a pause or play symbol is shown before them).
life-pause = Duraklat
life-play = Oynat
life-play-hover = Evreni çalıştır (Enter).
life-step = Adım
life-step-hover = Bir nesil ilerlet (N).
# Tooltip of the button beside Step, which reads "+16", "+1,024"…: that many generations.
life-stride-hover = Bu kadar nesil ilerlet (Shift+N).
life-reset = Sıfırla
life-reset-hover = Açıldığı hâliyle desene dön (nesil 0).
life-speed = Hız
# After the speed slider's number: generations a second. Short.
life-speed-unit = nesil/s
# Row label: how many generations the "+n" button advances.
life-stride = Adım aralığı
# The panel's numbers.
life-generation = Nesil
life-population = Nüfus
life-tiles = Karolar
# $count: the tiles stored; $capacity: how many the GPU pool holds.
life-tiles-of = { $count } / { $capacity }
life-tiles-hover = Saklanan 64×64 hücrelik karolar (her biri 4 KiB) ve GPU havuzunun kapasitesi.
# Row label: what a left-drag in the view does (Pan or Draw).
life-mouse = Fare
life-pan = Kaydır
life-pan-hover = Görünümü taşımak için sürükleyin.
life-draw = Çiz
# The Draw tool's tooltip; the toolbar's shows it after "Draw (D):", so it starts lowercase.
life-draw-hint = çevirmek için bir hücreye tıklayın; boyamak için sürükleyin — ölü bir hücrede başlayan bir çizgi çizer, canlı bir hücrede başlayan siler. Çalışan bir evren siz çizerken duraklar, bıraktığınızda devam eder.
life-undo = Geri al
life-undo-hover = Son düzenlemeden, doldurmadan, temizlemeden ya da kural değişikliğinden önceki evrene dön (çizerken Ctrl+Z).
life-grid = Izgara çizgileri
life-grid-hover = Hücre başına 8 piksel ya da daha fazla yakınlaştırıldığında hücre kenarlıkları.
# A random fill: "Random" labels the density field, "seed" the seed field after it.
life-random = Rastgele
life-density-hover = Bir doldurmanın canlı yaptığı hücrelerin oranı.
life-seed = tohum
life-fill-view = Görünümü doldur
# $cells: the largest side of the filled area, in cells (2,048).
life-fill-view-hover = Görünür alanı (merkezinin çevresinde en fazla { $cells } × { $cells } hücre) rastgele doldur.
life-clear = Temizle
life-clear-hover = Her hücreyi öldür (Geri al onları geri getirir).

## Life: notices in the panel, and file dialogs

# $path: the file written.
life-saved = Kaydedildi: { $path }
# $error: why (from the system, in English).
life-save-failed = Kaydedilemedi: { $error }
# The universe stopped running. $error: why (in English).
life-stopped = Durdu: { $error }
# A pattern file named a rule that does not read. $error: why (in English); $rule: the rule kept, in B3/S23 notation.
life-rule-not-understood = Desenin kuralı anlaşılamadı ({ $error }); { $rule } korundu
# $path: the file; $error: why (in English).
life-open-failed = { $path } açılamadı: { $error }
# File-dialog filter names.
life-filter-patterns = Life desenleri
life-filter-rle = Ardışık uzunluk kodlamalı desen

## Life: the toolbar (its buttons are icons; these are their tooltips)

life-toolbar-pause-hover = Evreni duraklat (Enter)
life-toolbar-play-hover = Evreni çalıştır (Enter)
# $stride: the generations a Shift+click steps.
life-toolbar-step-hover = Bir nesil ilerlet (N); Shift+tık ya da Shift+N { $stride } nesil ilerletir
life-toolbar-reset-hover = Açıldığı hâliyle desene dön (nesil 0)
# $hint: the Draw tool's tooltip (life-draw-hint).
life-toolbar-draw-hover = Çiz (D): { $hint }

## Life: the Pattern text window

life-text-title = Desen metni
life-text-intro = Bir desen yapıştırın: RLE (LifeWiki ve Golly’nin verdiği biçimde), düz metin .cells ya da Life 1.05 / 1.06.
life-text-load = Yükle
life-text-current = Geçerliyi RLE olarak
life-text-current-hover = Kutuyu, evrenin son yüklendiği hâliyle doldur.

## L-systems: the side panel

# Row label: the system list (system names stay in English).
lsys-system = Sistem
lsys-open = Aç…
lsys-open-hover = Bir Fractint .l dosyası ya da bir .lsys sistemi aç.
lsys-edit = Düzenle…
lsys-edit-hover = Sistemin metni: aksiyom, üretim kuralları, açı — düzenleyin ve uygulayın.
lsys-save = Kaydet…
lsys-save-hover = Sistemi bir .lsys metin dosyası olarak kaydet.
# Tooltip of the "SVG…" button.
lsys-svg-hover = Görünümü bir SVG çizimi olarak dışa aktar: çizgileri ve şekilleri vektör olarak, kendi renkleriyle.
lsys-follow = Mertebe yakınlaştırmayı izler
lsys-follow-hover = Yakınlaştırdıkça mertebe yükselir, böylece ayrıntı hiç tükenmez.
lsys-follow-fixed-hover = Bu sistem her mertebede bir çarpan kadar büyümez, bu yüzden sabit bir mertebede çizilir.
lsys-order = Mertebe
lsys-angle = Açı
lsys-sweep-hover = Açıyı bir gidip bir gelerek tara: eğri dönerken biçim değiştirir
# A small button that puts back the system's own angle after the slider moved it.
lsys-system-angle = Sistemin açısı
lsys-seed = Tohum
lsys-seed-hover = Rastgele seçimlerin izlediği değer: aynı tohum, aynı bitki.
lsys-seed-new = Yeni
lsys-seed-new-hover = Başka bir bitki: yeni bir tohum.
lsys-line-width = Çizgi kalınlığı
# Row label of a percentage slider: how much of the curve is drawn.
lsys-draw-on = Çizilen kısım
lsys-draw-on-hover = Eğrinin, kaplumbağanın çizdiği sırayla ne kadarının çizildiği
lsys-draw-on-play-hover = Eğrinin kendini çizmesi
lsys-colour-by = Renklendirme ölçütü
# The colourings, in the "Colour by" list.
lsys-colour-position = eğri boyunca konum
lsys-colour-depth = dal derinliği
lsys-colour-heading = yön
# C, < and > are the system's commands that set the colour index: keep them.
lsys-colour-index = renk dizini (C, <, >)
lsys-colour-plain = tek renk
# The panel's numbers.
lsys-growth = Büyüme
lsys-growth-built = sözcük olarak, kendi mertebesinde oluşturuldu
lsys-growth-built-hover = Parametrik ya da bağlama duyarlı: bir modülün yeniden yazımı sayılarına ya da komşularına bağlıdır; bu yüzden sözcük her seferinde bir nesil olmak üzere (en fazla iki milyon modül) oluşturulur ve çift hassasiyetle çizilir — mertebe yakınlaştırmayı izlemez.
# How much the picture grows from one order to the next. $factor: the factor (1.4142).
lsys-growth-factor = mertebe başına ×{ $factor }
lsys-growth-factor-period-2 = mertebe başına ×{ $factor } (mertebeler 2’şer artar)
lsys-growth-none = yok (sabit mertebe)
lsys-drawn = Çizilen
# $drawn: what was drawn (lsys-drawn-segments, lsys-drawn-shapes or lsys-drawn-segments-shapes); $order: the order.
lsys-drawn-at-order = { $drawn } (mertebe { $order })
# The same, when the walk stopped at its segment budget.
lsys-drawn-at-order-budget = { $drawn } (mertebe { $order }, bütçe)
# $segments: the line segments drawn.
lsys-drawn-segments =
    { $segments ->
        [one] { $segments } segment
       *[other] { $segments } segment
    }
# $shapes: the filled shapes drawn.
lsys-drawn-shapes =
    { $shapes ->
        [one] { $shapes } dolu şekil
       *[other] { $shapes } dolu şekil
    }
# Both counts. $segments: line segments; $shapes: filled shapes.
lsys-drawn-segments-shapes =
    { $segments ->
        [one] { $segments } segment
       *[other] { $segments } segment
    }, { $shapes ->
        [one] { $shapes } dolu şekil
       *[other] { $shapes } dolu şekil
    }
# $ms: the walk's time in milliseconds (12.3).
lsys-drawn-hover = { $ms } ms’de dolaşıldı; görünüm dışındaki alt ağaçlar atlanır, bir pikselin altında kalanlar tek bir segment olarak çizilir
# A word too long to build at the order asked for. $wanted: the order asked for; $built: the order drawn.
lsys-drawn-short = Mertebe { $wanted } sözcüğü iki milyon modülü aşıyor: mertebe { $built } çizildi.
# $order: the deepest order the app draws.
lsys-deepest-order = Çizilen en derin mertebede ({ $order }): daha fazla yakınlaştırmak ayrıntı eklemez.

## L-systems: notices in the panel and the editor, and file dialogs

# $path: the file; $error: why (in English).
lsys-open-failed = { $path } açılamadı: { $error }
# Why a file was not opened (shown after "Could not open …:").
lsys-file-too-large = 4 MiB’tan büyük
# Why a Fractint .l file was not opened: it holds no entries. `Name { … }` is the file's syntax: keep it.
lsys-no-entries = girdi yok (bir .l dosyası `Name {"{"} … {"}"}` girdileri içerir)
# $path: the file written.
lsys-saved = Kaydedildi: { $path }
# $error: why (from the system, in English).
lsys-save-failed = Kaydedilemedi: { $error }
# After File ▸ Export SVG. $path: the file written.
lsys-svg-exported = Dışa aktarıldı: { $path }
# $error: why (from the system, in English).
lsys-svg-failed = Dışa aktarma başarısız: { $error }
# File-dialog filter names: opening (any system file) and saving (one system).
lsys-filter-systems = L-sistemleri
lsys-filter-system = L-sistemi

## L-systems: the toolbar

lsys-toolbar-lower-hover = Mertebeyi düşür (sabitler)
lsys-toolbar-raise-hover = Mertebeyi yükselt (sabitler)
# Between the − and + buttons, in a fixed-width font. $order: the order, padded to 3 characters so the label keeps its width.
lsys-toolbar-order = mertebe { $order }
# A toggle: the order follows the zoom. Short.
lsys-toolbar-auto = oto
lsys-toolbar-auto-hover = Mertebe yakınlaştırmayı izler
lsys-toolbar-edit-hover = Sistemin metnini düzenle

## L-systems: the editor window, and a .l file's entries

lsys-editor-title = L-sistemi
# The editor's help. The quoted examples ('X = word', 'seed n', 'define R 1.456'…), the commands and "The Algorithmic Beauty of Plants" (a book's title) are the system's syntax and names: keep them as they are.
lsys-editor-help = Her satıra bir anahtar: angle (derece ya da çemberin bir bölümü için /n), axiom, 'X = word' biçiminde üretim kuralları ve isteğe bağlı olarak heading, draw / move / variables, colour, order. Rastgele seçilen alternatifleri olan bir sembolün, ağırlığıyla birlikte her alternatif için bir satırı vardır: 'X (0.3) = word'; 'seed n' bitkiyi seçer. Bir sözcükte {"{"} {"}"} kaplumbağanın yolunu doldurur ve . bir köşe ekler. Parametrik ve bağlama duyarlı üretim kuralları (The Algorithmic Beauty of Plants): 'A(s) : s > 1 = F(s)[+A(s/2)]', 'b < a > c = b', 'define R 1.456' ve 'ignore +-F' ile; F(l) l kadar ilerler, +(a) a derece döner. Bir Fractint .l girdisi olduğu gibi yapıştırılabilir.
lsys-editor-apply = Uygula
lsys-editor-revert = Geri döndür
lsys-editor-revert-hover = Gösterilen sistemin metni.
# The window listing a .l file's entries to pick from. $file: the file's name.
lsys-entries-title = { $file } — girdiler
# Tooltip of an entry that does not parse (its error is shown beside it, in English).
lsys-entry-unreadable-hover = Bu girdi okunamıyor.

## Export: opening a view or location (File ▸ Open view or location…)

# File-dialog filter names.
export-filter-view = Fractadyne görünümü ya da konumu
export-filter-image = Görüntü (PNG / EXR)
export-filter-location = Konum (.fdn / .kfr)
# $zoom: the magnification (1.2e148), followed by the magnification sign ×.
export-loaded-location = Konum yüklendi @ { $zoom }×
# $zoom: the magnification; $note: what was noteworthy in the file (in English).
export-loaded-location-note = Yüklendi @ { $zoom }× — { $note }
# $path: the file.
export-not-location = { $path } bir Fractadyne konumu değil.
export-file-too-large = Dosya çok büyük (bir .fdn konumu değil mi?).
# $path: the file; $error: why (from the system, in English).
export-read-failed = { $path } okunamadı: { $error }
# A Kalles Fraktaler .kfr file did not import. $error: why (in English).
export-import-failed = İçe aktarma başarısız: { $error }
# $path: an image whose embedded text is not a view.
export-not-view = { $path } Fractadyne meta verisi içeriyor, ancak bir görünüm içermiyor.
# $path: the image the view was read from.
export-loaded-view = Görünüm yüklendi: { $path }
# $path: the image; $note: what was noteworthy in it (in English).
export-loaded-view-note = Görünüm yüklendi: { $path } — { $note }
export-no-metadata = Bu dosyada gömülü Fractadyne görünüm meta verisi yok.

## Export: notices after an export or a snapshot
## "File → Export image…" in them names the menu items (menu-file, menu-file-export).

# The export window's status while the image renders.
export-rendering = İşleniyor…
export-canceled = Dışa aktarma iptal edildi.
# $error: why (in English).
export-failed = Dışa aktarma başarısız: { $error }
# $size: the image's size in pixels (1920×1080); $path: the file written.
export-saved = Kaydedildi: { $size } → { $path }
# An L-system export whose walk stopped at its budget. $size: pixels; $segments: the budget (4,000,000); $path: the file.
export-saved-incomplete = Kaydedildi: { $size } (eksik: { $segments } segmentlik bütçe aşıldı — mertebeyi düşürün) → { $path }
# A two-panel export written as two files. $path: the first of them.
export-saved-two = 2 dosya kaydedildi → { $path }
# $size: pixels; $path: the file written.
export-saved-corrected = Kaydedildi: { $size } (bozulmalar düzeltildi) → { $path }
# $path: the first of the two files.
export-saved-two-corrected = 2 dosya kaydedildi (bozulmalar düzeltildi) → { $path }
# Added after a finished export's notice. $duration: how long it took (8.3s, 1m 04.0s).
export-elapsed = ({ $duration } sürdü)
export-preparing-deep = Derin dışa aktarma hazırlanıyor — referans oluşturuluyor (bu biraz sürebilir)…
# $size: pixels (5120×4035); $ss: the supersampling factor; $iterations: the iteration count.
export-large = Büyük dışa aktarma ({ $size }, { $ss }× süper örnekleme, { $iterations } yineleme) — arka planda işleniyor; bu boyuttaki dışa aktarmalarda bozulma düzeltme atlanır…
export-life-unavailable = Bir Life görünümünü görüntü olarak dışa aktarmak henüz mümkün değil — bir ekran görüntüsü alın ya da Dosya ▸ Life desenini kaydet seçeneğini kullanın.
export-already-running = Zaten bir dışa aktarma çalışıyor — Dosya → Görüntüyü dışa aktar… ilerlemesini gösterir.
# $size: the image's size in pixels.
export-snapshot-background = Anlık görüntü: { $size } arka planda işleniyor — Dosya → Görüntüyü dışa aktar… ilerlemeyi gösterir ve işlemi iptal edebilir.
export-no-gpu = GPU kullanılamıyor
# A screen-capture snapshot. $size: pixels; $path: the file written.
export-snapshot-saved = Ekran anlık görüntüsü kaydedildi: { $size } → { $path }
# $error: why (in English).
export-snapshot-failed = Anlık görüntü başarısız: { $error }

## The colour picker (the popup of a colour swatch)

# The third row's label: the colour as hexadecimal bytes. Short (about 6 characters).
picker-row-hex = Hex
picker-hex-hover = Onaltılık bir değer yazmak için tıklayın
picker-cancel = İptal
picker-cancel-hover = Bu seçicinin açıldığı rengi geri koy
picker-ok = Tamam
picker-ok-hover = Bu rengi koru
# Starts the screen eyedropper (an icon is shown before it).
picker-pick = Seç
picker-pick-hover = Ekranın herhangi bir yerinden bir renk al
# Tooltip of the disabled Pick button (in the picker and the gradient editor) on Linux and macOS.
picker-eyedropper-unsupported = Ekranın başka bir yerinden renk seçmek, platformun bir ekran yakalama API’sini gerektirir. Yalnızca Windows derlemesinde bu vardır; Linux’ta bunun için bir X11 bağımlılığı gerekir, Wayland ise kendi izin istemine sahip bir portal ister.

## The fractal families: what each one is (the info panel's "About" section and the pickers' hover text).
## The formulas themselves (z -> z^2 + c) are written in code and stay as they are.

fractal-mandelbrot-about = Kanonik kaçış zamanlı fraktal: 0’ın yörüngesinin sınırlı kaldığı c değerlerinin kümesi. Sınırı sonsuz derecede karmaşıktır.
fractal-multibrot3-about = Bir Multibrot kümesi - daha yüksek bir kuvvette Mandelbrot yapısı. d kuvveti (d-1) katlı dönme simetrisi verir.
fractal-multibrot4-about = 4. kuvvette Multibrot: üç katlı simetri, geniş tomurcuklar.
fractal-multibrot5-about = 5. kuvvette Multibrot: dört katlı simetri.
fractal-tricorn-about = Tricorn (Mandelbar): her adımda z’nin eşleniğini alır. Bu anti-holomorf eşleme üç köşeli bir şekil verir.
fractal-burning-ship-about = Kare almadan önce z’nin kısımlarının mutlak değerleri alınır; sonuç alevler içindeki bir gemiye benzer.
fractal-celtic-about = Yalnızca z^2’nin gerçel kısmının mutlak değerini alan, Burning Ship’in bir akrabası; Kelt düğümü / kalp motifleri üretir.
fractal-buffalo-about = z^2’nin her iki bileşeninin mutlak değerini alan bir mutlak değer çeşidi.
fractal-phoenix-about = Phoenix bir önceki yineleme değerini de kullanır ve aleve benzer iplikler verir. Julia biçimini Julia modu ile deneyin.
fractal-newton-about = z^3 = 1 için Newton’un kök bulma yinelemesi; her noktanın ne kadar hızlı yakınsadığına göre renklendirilir. Bir yakınsama (kaçış değil) fraktalı.
fractal-multibrot6-about = 6. kuvvette Multibrot: beş katlı simetri.
fractal-multibrot7-about = 7. kuvvette Multibrot: altı katlı simetri, tomurcuklar bir çembere doğru incelir.
fractal-multibrot8-about = 8. kuvvette Multibrot: yedi katlı simetri.
fractal-burning-ship3-about = 3. kuvvette Burning Ship: mutlak değerler alınır, ardından küpü alınır.
fractal-burning-ship4-about = 4. kuvvette Burning Ship.
fractal-burning-ship5-about = 5. kuvvette Burning Ship.
fractal-tricorn3-about = 3. kuvvette Tricorn (Mandelbar): dört katlı simetri.
fractal-tricorn4-about = 4. kuvvette Tricorn: beş katlı simetri.
fractal-tricorn5-about = 5. kuvvette Tricorn: altı katlı simetri.
fractal-celtic3-about = 3. kuvvette Celtic katlaması: z^3’ün gerçel kısmının mutlak değeri.
fractal-celtic4-about = 4. kuvvette Celtic katlaması.
fractal-celtic5-about = 5. kuvvette Celtic katlaması.
fractal-buffalo3-about = 3. kuvvette Buffalo: z^3’ün her iki kısmının mutlak değerleri.
fractal-buffalo4-about = 4. kuvvette Buffalo.
fractal-buffalo5-about = 5. kuvvette Buffalo.
fractal-custom-about = Fractint tarzı ifadelerle yazılmış bir formül (Fraktal ▸ Özel formül…). Çoğu, yerleşik aileler gibi pertürbasyonla derin yakınlaştırılır; sizinkinin öyle olup olmadığını formül penceresi söyler.
fractal-life-about = Conway’in Hayat Oyunu ve akrabaları, sınırsız bir düzlemde: Generations kuralları, totalistik olmayan (Hensel) kurallar, desen dosyaları. Oynatın, adım adım ilerletin, çizin.
fractal-lsystem-about = Lindenmayer sistemleri: bir yeniden yazma dilbilgisinden bir kaplumbağanın çizdiği eğriler, alan dolduran eğriler, adalar ve bitkiler. Mertebe yakınlaştırmayı izler, böylece ayrıntı hiç tükenmez. Fractint .l dosyaları doğrudan açılır.

# Shown where the other families show their formula (z -> z^2 + c).
fractal-custom-formula = z -> sizin yazdığınız formül
# Shown where the other families show their formula. "B3/S23" is the rule's notation: keep it.
fractal-life-formula = B3/S23: 3 canlı komşuyla doğar, 2 ya da 3 komşuyla hayatta kalır
# Shown where the other families show their formula. "F = F+F--F+F" is the grammar's notation: keep it.
fractal-lsystem-formula = F = F+F--F+F: her sembol yeniden yazılır, ardından bir kaplumbağa tarafından çizilir

# The last paragraph of the picker's hover text for a family that renders only on the direct path.
fractal-no-deep-zoom = Yalnızca doğrudan işleme - bu ailede derin yakınlaştırma yok.

## Names of settings, shown in the Settings menu and the welcome window.

theme-dark = Koyu
theme-light = Açık
# The update track that follows only stable releases.
update-track-stable = Kararlı
# The update track that also offers pre-releases (betas).
update-track-beta = Beta (ön sürümler)

## The status bar (its hover texts; the readouts follow in their own section)

status-period-hover = Atladığınız minibrotun periyodu (M tuşuna basın ya da Konuma git ▸ Özelliğe git). Görünüm özelliğin üzerinde kaldığı sürece görünür, başka yere gittiğinizde silinir.

## The playback transport: the player over the view while a tour plays.

# $close: the close button's icon, as drawn on that button.
play-finished-hover = Bitti — geri sarmak için sürükleyin ya da kapatmak için { $close }
# $kind: "benchmark" or "script" (the variants hold the wording); $name: the tour's name.
play-playing-hover =
    { $kind ->
        [benchmark] “{ $name }” kıyaslaması oynatılıyor
       *[other] “{ $name }” betiği oynatılıyor
    }
play-restart = Baştan yeniden başlat
play-back = 10 saniye geri
play-again = Baştan yeniden oynat
play-resume = Sürdür
play-pause = Duraklat
play-stop = Durdur ve başa sar
play-forward = 10 saniye ileri
play-speed-hover = Oynatma hızı (0.5× / 1× / 2× / 4× arasında geçmek için tıklayın)
play-render = Bu betiği bir kare dizisine işle…
play-render-none = Yerleşik kıyaslamanın işlenecek bir betik dosyası yok
play-open = Başka bir betik oynat…
play-loop = Tur sona erdiğinde tekrarla
play-close = Oynatıcıyı kapat ve kendi görünüm ayarlarınızı geri yükle
# A notice beside the scrub bar while the renderer holds the tour's clock. Keep it short.
play-waiting = ayrıntı bekleniyor
# "[playback] pace" is a setting in the script file: keep it as written.
play-waiting-hover = İşleyici bu görünümü çözümlerken tur saati duraklatılır (referans oluşturma / yineleme bütçesinin tırmanması). Oynatma kendiliğinden devam eder; betikteki [playback] pace ayarına bakın.
play-scrub = Turda ileri geri gitmek için sürükleyin
# $behind: how far the script clock is behind real time, as minutes:seconds (1:05).
play-scrub-behind =
    Turda ileri geri gitmek için sürükleyin
    Betik saati gerçek zamanın { $behind } gerisinde (hız ayarlayıcı, işleyici yetişebilsin diye turu yavaşlattı). Üstteki çentik, gerçek saatle oynatmanın nerede olacağını gösterir.

## The autopilot's notices (toasts). Where a notice has two lines, the first is the verdict and the second what to do about it.

autopilot-on-detail = Otopilot açık — ayrıntıya doğru dalıyor (herhangi bir girdi durdurur)
autopilot-off = Otopilot kapalı
autopilot-misi-needs-mandelbrot =
    Otopilot: Misiurewicz hedefi Mandelbrot kümesini gerektirir
    Bunun yerine ayrıntıya doğru dalınıyor
autopilot-on-misi =
    Otopilot açık — en yakın Misiurewicz noktası aranıyor…
    Herhangi bir girdi durdurur
# $seconds: how long the search has run, in whole seconds.
autopilot-misi-finding =
    Otopilot: en yakın Misiurewicz noktası aranıyor…
    { $seconds } s · herhangi bir girdi durdurur
autopilot-misi-not-found =
    Otopilot: bu görünümün yakınında Misiurewicz noktası bulunamadı
    Bulucu bu derinliğe ulaşamıyor olabilir — Ayrıntı hedefini deneyin
# ($preperiod,$period) names the point, as mathematicians write it: keep the notation. $widths: a power of two (2^$widths screen widths).
autopilot-misi-off-screen =
    Otopilot: en yakın Misiurewicz noktası ({ $preperiod },{ $period }) ekranın dışında
    ~2^{ $widths } ekran genişliği uzakta — bir spiral merkezine daha çok yaklaşın
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-not-solved =
    Otopilot: bu görünümün yakınındaki Misiurewicz noktası ({ $preperiod },{ $period }) çözülemedi
    Ayrıntı hedefini deneyin
autopilot-misi-refused = Otopilot: Misiurewicz çözümü reddedildi (geçersiz çift)
autopilot-misi-stopped = Otopilot: Misiurewicz çözücüsü beklenmedik şekilde durdu
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-diving =
    Otopilot: Misiurewicz ({ $preperiod },{ $period }) noktasına dalıyor
    Herhangi bir girdi durdurur
# ($preperiod,$period) names the point: keep the notation. $octaves: how often the structure repeats (2.37); $twist: its turn in degrees, signed (+12.5).
autopilot-misi-diving-repeat =
    Otopilot: Misiurewicz ({ $preperiod },{ $period }) noktasına dalıyor
    Yapı her { $octaves } oktavda bir tekrarlanır, burulma { $twist }° · herhangi bir girdi durdurur
# $limit: the magnification, as written (1e300).
autopilot-dive-limit = Otopilot: dalış sınırına ulaşıldı (~{ $limit }×)
# $have: the fixed iteration count; $typical: what this depth typically needs.
autopilot-dead-end-short =
    Otopilot durdu: nişan alınacak bir şey kalmadı — yineleme sayısı { $have } değerine sabitlenmiş ve bu derinlik genellikle ~{ $typical } gerektirir
    Otomatik yinelemeyi açın ya da sayıyı yükseltin
# $limit: the fixed iteration count.
autopilot-dead-end-cap =
    Otopilot durdu: nişan alınacak bir şey kalmadı — görünümdeki her şey { $limit } değerindeki sabit yineleme sınırına kadar çalışıyor
    Otomatik yinelemeyi açın ya da sayıyı yükseltin
autopilot-dead-end = Otopilot: ileride ayrıntı yok (durdu)

## Tours (camera scripts)

# The name of the file type in the file dialog's filter list.
script-file-filter = Fractadyne betiği (TOML)
# $name: the script's name.
script-finished = Betik bitti — “{ $name }”
# The title of the notice that follows (the file's path and the error, in English).
script-load-failed = Betik yüklenemedi

## Installing a language file

# The name of the file type in the file dialog's filter list.
language-file-filter = Fractadyne dil dosyası

## The notice when the graphics driver's shader cache stops keeping shaders. The text is shown in a fixed-width font with its line breaks as written: break the lines where they read well.

driver-cache-title = Grafik sürücüsü gölgelendiricileri saklamıyor
# $count: how many shaders compiled again; $pipeline: the slowest one's name (a technical id: keep it); $seconds: its time in seconds (7.5).
driver-cache-body =
    Grafik sürücünüz bu oturumda Fractadyne gölgelendiricilerinden { $count }
    tanesini, her birini daha önce derlemiş olmasına rağmen yeniden derledi
    (en yavaşı: { $pipeline }, { $seconds } s). Sürücünün gölgelendirici
    önbelleği derlenmiş gölgelendiricileri saklamıyor; bunun en sık nedeni
    önbelleğin dolu olmasıdır. Bu değişene kadar dışa aktarmalar, derin
    görünümler ve Fractadyne’ın başlatılması derlemeye saniyeler harcayabilir.
# $folder: the cache folder's path; $size: its size in MB (1024). "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia-folder =
    Düzeltmek için: Fractadyne’ı kapatın, ardından şu klasörün içeriğini silin:
      { $folder }   ({ $size } MB)
    Sürücü önbelleği yeniden oluşturur. Yeniden dolmasını önlemek için
    NVIDIA Denetim Masası > 3D ayarlarını yönet > Gölgelendirici Önbellek
    Boyutu değerini yükseltin.
# "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia =
    Düzeltmek için: NVIDIA Denetim Masası > 3D ayarlarını yönet >
    Gölgelendirici Önbellek Boyutu değerini yükseltin ya da sürücünün
    gölgelendirici önbelleğini temizleyin.
# "AMD Software > Graphics > Advanced > Reset Shader Cache" names AMD's own settings: use the names AMD's software shows in this language.
driver-cache-fix-amd = Düzeltmek için: AMD Software > Grafik > Gelişmiş > Gölgelendirici Önbelleğini Sıfırla.
driver-cache-fix-other = Düzeltmek için: grafik sürücüsünün gölgelendirici önbelleğini temizleyin.

## Why a language file could not be installed: each ends the sentence "{ $file } is not a usable language file: …".

language-install-not-utf8 = UTF-8 metin değil
language-install-too-large = çok büyük
# "fractadyne-<language>.ftl" is the file name to use: keep it as written.
language-install-no-tag = hangi dil olduğunu belirtmiyor (dosyaya fractadyne-<language>.ftl adını verin)
language-install-english = İngilizce zaten yerleşik
language-install-empty = bu sürümün hiçbir iletisini çevirmiyor
language-install-no-folder = ayarlar klasörü yok

## The notice after a session that did not end cleanly, when its iteration setting was set aside.

# $base: the iteration count automatic iterations start from.
quarantine-setting-auto = { $base } temel değerinden başlayan otomatik yineleme
# $count: the fixed iteration count.
quarantine-setting-fixed =
    { $count ->
        [one] { $count } yineleme
       *[other] { $count } yineleme
    }
# $end: "device" when the session lost its graphics device, "unclean" otherwise (the variants hold the wording). $depth: the magnification, as written (1.2e148). $setting and $restored: an iteration setting (quarantine-setting-auto or -fixed).
quarantine-notice =
    { $end ->
        [device] Önceki oturum, { $setting } ayarındayken { $depth }× derinlikte grafik aygıtını kaybetti. Aynı yüke doğrudan geri dönmesin diye bu oturum bunun yerine şu ayarla başladı: { $restored }.
       *[other] Önceki oturum, { $setting } ayarındayken { $depth }× derinlikte düzgün sona ermedi. Aynı yüke doğrudan geri dönmesin diye bu oturum bunun yerine şu ayarla başladı: { $restored }.
    }

## The second graphics card's status line, under its setting in the control panel.

gpu2-state-off = Kapalı: her şeyi bu pencerenin grafik kartı işler.
gpu2-state-starting = Başlatılıyor…
# $name: the card's name (and its graphics API).
gpu2-state-running = Kullanımda: { $name }
# $why: the error, as the graphics driver reports it.
gpu2-state-failed = Başlatılamadı: { $why }
# $name: the card's name.
gpu2-state-lost = Durdu: { $name } kaybedildi. Bundan sonra tek kart; yeniden denemek için onu yeniden seçin.
# $state: one of the lines above. "--worker-gpu" is a command-line option: keep it.
gpu2-state-pinned = { $state } (--worker-gpu ile ayarlandı)

## The custom-formula dialog: how deep the formula renders (one line under the formula).

formula-depth-deep = Pertürbasyonla derin yakınlaştırma; yerleşik fraktallarda olduğu gibi 1e28x ötesinde genişletilmiş aralık (her adım yinelenir: özel formüller için henüz seri yaklaşımı ya da BLA yok).
# $why: what the formula has that keeps it off the deep-zoom path (one of the formula-depth-why messages below).
formula-depth-direct = Doğrudan işleme ({ $why } için henüz derin yakınlaştırma biçimi yok): c’nin tek hassasiyetli bir adımı bir pikseli kaplayana kadar keskin — yaklaşık 1e4x ile 1e5x arası, orijinden uzakta daha az derin.
formula-depth-why-init = bir başlangıç bölümü
formula-depth-why-bailout = kendine ait bir kaçış testi
formula-depth-why-variable = adımdan adıma korunan bir değişken
formula-depth-why-previous = bir önceki yineleme değeri
formula-depth-why-comparison = bir karşılaştırma ya da bir if bloğu
# floor, ceil, trunc and round are functions of the formula language: keep them.
formula-depth-why-rounding = yuvarlama (floor, ceil, trunc, round)
# "maxit" is a word of the formula language: keep it.
formula-depth-why-maxit = maxit
formula-depth-why-power = üssü değişen bir kuvvet
formula-depth-why-perturbed = zaten pertürbe edilmiş bir program

## Image and frame sizes: the presets described in words (the others are standard names: HD 720p, WUXGA, DCI 4K).

# $size: the pixel size (2048×2048).
size-preset-square = Kare — { $size }
# $ratio: the aspect ratio (9:16); $size: the pixel size.
size-preset-portrait = Dikey { $ratio } — { $size }
# $ratio: the aspect ratio (32:9); $size: the pixel size.
size-preset-super-ultrawide = Süper ultra geniş { $ratio } — { $size }

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
fractal-family-custom = Özel
# Conway's Game of Life.
fractal-family-life = Life
fractal-family-lsystem = L-sistemi
# A family at a power: "Multibrot 3", "Burning Ship 4". $family: a name above; $power: the power.
fractal-with-power = { $family } { $power }

## Names of choices: colouring methods, orbit traps, palette animation, snapshots, angle units.

color-method-smooth = Yumuşak yineleme
color-method-stripe = Şerit ortalaması
color-method-triangle = Üçgen eşitsizliği
color-method-trap = Yörünge tuzağı
color-method-distance = Uzaklık tahmini
color-method-decomposition = Ayrışım
# The shape an orbit trap catches the orbit with.
trap-point = Nokta
trap-cross = Çarpı
trap-circle = Çember
# Palette animation: none.
palette-anim-off = Kapalı
palette-anim-forward = İleri
palette-anim-reverse = Geri
palette-anim-pingpong = Git-gel
# Palette animation: a new random gradient every so often.
palette-anim-random = Rastgele gradyanlar
# What the Snapshot button does.
snapshot-mode-ask = Her seferinde sor
snapshot-mode-screen = Ekran görüntüsü
snapshot-mode-render = Tam işleme
# The unit of an angle typed into Go to location's polar fields.
angle-degrees = derece
angle-radians = radyan
angle-turns = devir

## Report an issue: the choices. (The report itself is sent in English.)

issue-kind-crash = Uygulama çökmesi / donması
issue-kind-rendering = Yanlış işleme
issue-kind-performance = Performans sorunu
issue-kind-ui = Arayüz / kullanılabilirlik sorunu
issue-kind-feature = Özellik isteği
issue-kind-other = Diğer
severity-low = Düşük
severity-medium = Orta
severity-high = Yüksek
severity-blocking = Engelleyici
repro-always = Her zaman
repro-sometimes = Bazen
repro-once = Bir kez oldu
repro-cannot = Yeniden üretilemiyor

## The benchmark's depths and the diagnostics' tests.

# $zoom: the dive's final magnification (1e32).
bench-depth-standard = Standart ({ $zoom }×)
# $zoom: the dive's final magnification (1e48).
bench-depth-ultra = Ultra derin ({ $zoom }×)
diag-test-selftest = Öz sınama
diag-test-uitest = Arayüz testi
diag-test-gputest = GPU aritmetik denetimi

## The status bar's readouts: each word is followed by a value in a slot of fixed width. Keep the words short (one word each).

status-center = merkez
status-cursor = imleç
status-zoom = büyütme
# Short for iterations.
status-iter = yin
# The period of the minibrot jumped to.
status-period = periyot
# A Game of Life view's scale (followed by "1 px = 4 cells" or "8 px / cell").
status-scale = ölçek
# A Game of Life view's generation. Short.
status-generation = nesil
# A Game of Life view's population. Short.
status-population = nüfus
# An L-system's order (how many times its rules were applied).
status-order = mertebe
# How many segments and shapes an L-system drew.
status-drawn = çizilen
# $value: how many cells one pixel covers (4, 9.99e300). "px" is the unit symbol: keep it.
life-scale-cells = 1 px = { $value } hücre
# $value: how many pixels one cell covers (8). "px" is the unit symbol: keep it.
life-scale-pixels = { $value } px / hücre

## The status bar's limit diagnostic: a short label (after a warning sign), and what it means when hovered.

# The view is as deep as this location can be rendered. Short.
status-limit-depth = derinlik sınırı
# Every pixel ran out of iterations. Short.
status-limit-exhausted = yineleme tükendi
# Pixels are capped at the iteration budget. Short.
status-limit-capped = yineleme tavanda
# $pixels: how many pixels wide one step of the coordinates is; $sharp_to: the magnification the view is sharp to (written out, 1,234,567).
status-limit-direct-detail = Bu görünüm GPU’da tek hassasiyetle doğrudan işlenir; burada koordinatların bir adımı { $pixels } piksel genişliğindedir: komşu pikseller aynı noktayı hesaplar, bu yüzden görüntü bloklara bölünür. Bu konumda yaklaşık { $sharp_to }× düzeyine kadar keskindir. Bu formülün henüz bir derin yakınlaştırma (pertürbasyon) yolu yok, bu yüzden daha fazla yakınlaştırmak daha fazla ayrıntı göstermez.
# $samples: the GPU buffer's capacity in orbit samples; $iterations: the iterations pixels are limited to.
status-limit-depth-detail = Referans yörünge kaçmadan GPU arabelleği sınırına ({ $samples } örnek) ulaştı; bu yüzden buradaki pikseller ~{ $iterations } yinelemeyle sınırlıdır ve bu görünümde daha ince ayrıntı hesaplanamaz — ne canlı ne de dışa aktarılmış. Bu, bu konum ailesi için geçerli derinlik sınırıdır.
# $budget: the largest iteration budget this view can be given.
status-limit-exhausted-detail = Bu görünüme verilebilecek en büyük bütçede ({ $budget }) bile her pikselin yinelemesi tükendi. Ya görünüm burada kümenin içindedir (derinde normal) ya da uygulamanın şu an sağlayabileceğinden fazla yineleme gerektirir — görüntü her iki durumda da aynıdır, bu yüzden bütçe ucuz bir değere geri döndürüldü.
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations" is the setting's name in the control panel's Quality section.
status-limit-capped-max = Piksellerin %{ $percent } kadarının yinelemesi { $budget } bütçesinde tükendi ve kaçanlar tam o tavanda bitiyor — görünüm yetersiz çözümlenmiş. Yineleme zaten en yüksek değerde, bu yüzden bu görünüm uygulamanın şu an verebileceğinden fazlasını gerektiriyor.
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations", "Controls" and "Quality" are the names of a setting, the control panel and its section.
status-limit-capped-raise = Piksellerin %{ $percent } kadarının yinelemesi { $budget } bütçesinde tükendi ve kaçanlar tam o tavanda bitiyor — görünüm yetersiz çözümlenmiş. Çözümlemek için Yineleme değerini yükseltin (Denetimler ▸ Kalite).
