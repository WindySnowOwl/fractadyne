### Fractadyne user interface: Indonesian (Bahasa Indonesia).
### A draft: machine translation (Claude, by Anthropic) of the English file, 2026-10-23, not yet
### reviewed by a native speaker. Corrections are welcome as pull requests (see i18n/README.md).

## Language data: read by Fractadyne, not shown as messages.

# This language's own name, shown in the language menu.
language-name = Bahasa Indonesia
# The same name in English, shown beside it in the menu.
language-english-name = Indonesian
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
language-more = Bahasa lainnya…
# The Languages window's title.
language-window-title = Bahasa
# The Languages window's introduction.
language-window-intro = Antarmuka Fractadyne dalam bahasa lain. Satu bahasa adalah satu berkas: berkas itu disertakan dalam rilis, atau dapat Anda pasang di sini. Koordinat, berkas, dan baris perintah tetap dalam bahasa Inggris di semua bahasa.
# Column headers of the installed-language list.
language-col-language = Bahasa
language-col-translated = Diterjemahkan
language-col-from = Sumber
# Where a language comes from.
language-source-builtin = bawaan
language-source-package = rilis ini
language-source-user = dipasang
language-source-dev = folder kerja
# $percent: how much of the interface the file translates, 0–100.
language-coverage = { $percent }%
# $count: messages in the file that failed the checks and show in English instead.
language-dropped = { $count } pesan perlu diperiksa
# Buttons of the Languages window.
language-install = Pasang berkas bahasa…
language-install-hover = Pilih berkas bahasa Fractadyne (.ftl). Berkas itu diperiksa, lalu disalin ke folder bahasa dalam pengaturan Anda, tempat berkas itu tetap tersimpan meskipun ada pembaruan.
language-open-folder = Buka folder bahasa
language-reload = Muat ulang
language-reload-hover = Baca ulang berkas bahasa (setelah Anda menyunting atau menambahkan berkas).
language-use = Gunakan
# Notices after installing a file. $name: the language's own name; $file: the file's name.
language-installed = { $name } dipasang dari { $file }.
# $file: the file's name; $why: what was wrong with it.
language-install-failed = { $file } bukan berkas bahasa yang dapat digunakan: { $why }
# Shown when a language file cannot be loaded and English is used instead. $name: the language.
language-load-failed = { $name } tidak dapat dimuat, jadi Fractadyne ditampilkan dalam bahasa Inggris: { $why }
# A language whose writing system needs a font this computer does not have (Japanese or Chinese without a system font). $name: the language, in its own name.
language-font-missing = Tidak ditemukan fon yang dapat menampilkan { $name } di komputer ini, jadi bahasa itu tidak dapat ditampilkan. Pasang fon { $name } (misalnya paket bahasa milik sistem itu sendiri), lalu pilih lagi.
# Shown after a draft translation's name in the language menu and the Languages window. Short.
language-draft = draf
language-draft-hover = Draf: terjemahan ini belum ditinjau oleh penutur asli. Koreksi dipersilakan, sebagai pull request ke berkas bahasa tersebut.
# The heading over the decimal-mark choice, in the language menu and the Languages window.
language-decimal-mark = Tanda desimal
language-decimal-mark-hover = Cara angka ditulis: waktu, ukuran, dan jumlah. Koordinat dan perbesaran selalu memakai titik, sebagaimana berkas dan program lain membacanya.
# $sample: a number written with this language's own punctuation (1,234.5).
language-decimal-language = Sesuai penulisan bahasa ini ({ $sample })
# $sample: a number written with a decimal point (1,234.5).
language-decimal-point = Titik ({ $sample })
# $sample: a number written with a decimal comma (1.234,5).
language-decimal-comma = Koma ({ $sample })

## Asked once when the system's language changes

# The window's title.
language-changed-title = Bahasa
# $new: the system's new language, in its own name.
language-changed-question = Bahasa sistem Anda sekarang { $new }. Gunakan bahasa itu di Fractadyne?
# Buttons. $name: a language's own name.
language-changed-use = Gunakan { $name }
language-changed-keep = Pertahankan { $name }

## The welcome window (first run)

welcome-title = Selamat datang di Fractadyne
welcome-intro = Penjelajah zoom dalam untuk himpunan Mandelbrot dan kerabatnya. Beberapa kontrol untuk memulai:
# The language row, at the top of the window.
welcome-language = Bahasa
# Controls: the key or gesture on the left, what it does on the right.
welcome-key-drag = seret
welcome-key-drag-does = geser tampilan
welcome-key-scroll = gulir
welcome-key-scroll-does = perbesar / perkecil ke arah kursor
welcome-key-space = tahan Space
welcome-key-space-does = zoom halus berkelanjutan (Shift+Space untuk memperkecil)
# Shown after the click-zoom toolbar icon.
welcome-key-click = lalu klik
welcome-key-click-does = menyelam ke titik yang diklik (bilah alat)
welcome-key-m-does = lompat ke minibrot terdekat pada skalanya sendiri
welcome-landmarks = Lompat ke lokasi terkenal untuk memulai:
welcome-theme = Tema
welcome-watermark = Tampilkan tanda “Fd”
welcome-watermark-hover = Tanda merek kecil di kanan bawah tampilan. Jika dimatikan, tanda itu dihapus dari tampilan langsung dan dari gambar yang diekspor.
welcome-updates = Pembaruan
welcome-check-updates = Periksa pembaruan
welcome-check-updates-hover = Menanyakan rilis terbaru ke GitHub saat aplikasi dimulai. Tidak ada yang diunggah.
welcome-files = Berkas
welcome-no-config = (tidak ada direktori konfigurasi)
welcome-open = Buka
welcome-rendering-on = Merender di
welcome-open-help = Buka bantuan lengkap  (F1)
# $icon: the Settings icon.
welcome-more-in-settings = Lainnya di Berkas → { $icon } Pengaturan
welcome-get-started = Mulai

## Menus: shared

# A percentage in a menu (UI scale). $value: the number.
common-percent = { $value }%
# A state shown after a menu item's name, in brackets.
menu-state-connected = terhubung
menu-state-rendering = merender
menu-state-running = berjalan

## Navigate ▸ Bookmarks (also the toolbar's star)

menu-bookmarks = Markah
menu-bookmarks-add = Tambahkan tampilan saat ini
menu-bookmarks-manage = Kelola…

## The fractal picker (Fractal menu and the toolbar's dropdown)

menu-family-powers = Pangkat lebih tinggi
menu-family-automata = Automata
menu-family-lsystems = Sistem-L
menu-fractal-custom = Rumus kustom…
menu-fractal-library = Pustaka rumus…
menu-fractal-library-hover = Rumus kustom yang tersimpan: terapkan, sunting, impor, dan ekspor
# The "Show" group: which picture of the formula is drawn. $name: the formula's name (Mandelbrot, Burning Ship…).
menu-show = Tampilkan
menu-show-set = Himpunan { $name }
menu-show-set-hover = Bidang parameter — himpunan { $name } itu sendiri.

    Setiap titiknya adalah himpunan Julia yang berbeda; ini adalah peta dari semuanya.
menu-show-julia = Himpunan Julia
menu-show-julia-hover = Satu titik dari himpunan { $name }, digambar sebagai gambar tersendiri — bentuk yang Anda dapatkan dengan menahan titik itu tetap.

    Pilih titiknya dengan mengarahkan kursor ke peta dalam “Keduanya, tertaut”; satu klik di sana menyematkannya.
menu-show-julia-none = { $name } tidak memiliki parameter bebas, jadi tidak memiliki himpunan Julia.
menu-show-both = Keduanya, tertaut
menu-show-both-hover = Himpunan { $name } di kiri, dan di kanan himpunan Julia dari titik mana pun yang berada di bawah kursor Anda.

    Gerakkan kursor melintasi peta dan perhatikan Julia berubah — ini cara tercepat untuk melihat hubungan keduanya.
menu-show-both-none = { $name } tidak memiliki himpunan Julia untuk dipasangkan.

## File menu

menu-file = Berkas
menu-file-open-view = Buka tampilan atau lokasi…
menu-file-gallery = Galeri…
menu-file-export = Ekspor gambar…
menu-file-snapshot = Cuplikan
menu-file-open-life = Buka pola Life…
menu-file-open-life-hover = Berkas RLE, .cells, atau Life 1.05/1.06, ditampilkan sebagai semesta Life.
menu-file-life-text = Teks pola Life…
menu-file-life-text-hover = Tempelkan pola sebagai teks (RLE seperti yang diberikan LifeWiki).
menu-file-save-life = Simpan pola Life…
menu-file-save-life-hover = Semesta sebagaimana adanya sekarang, sebagai RLE.
menu-file-open-lsystem = Buka sistem-L…
menu-file-open-lsystem-hover = Berkas Fractint .l (pilih salah satu entrinya) atau sistem .lsys.
menu-file-save-lsystem = Simpan sistem-L…
menu-file-save-lsystem-hover = Teks sistem ini, sebagai berkas .lsys.
menu-file-export-svg = Ekspor SVG…
menu-file-export-svg-hover = Tampilan sebagai gambar SVG: garis dan bentuknya sebagai vektor, dengan warnanya.
menu-file-render-client = Klien render…
menu-file-render-client-hover = Pinjamkan GPU mesin ini ke render farm: sambungkan ke pengendali (Alat ▸ Render di farm… pada mesin lain) dan render bingkai untuknya.
menu-file-settings = Pengaturan
menu-file-reset = Atur ulang status aplikasi…
menu-file-reset-hover = Hapus semua status tersimpan (sesi, markah, gambar mini) dan mulai dari awal. Meminta konfirmasi terlebih dahulu.
menu-file-quit = Keluar

## File ▸ Settings

menu-settings-fps = Batas laju bingkai
menu-settings-fps-uncapped = Tanpa batas
# $fps: frames per second.
menu-settings-fps-n = { $fps } FPS
menu-settings-ui-scale = Skala UI (ukuran fon)
menu-settings-theme = Tema
menu-settings-updates = Pembaruan
menu-settings-updates-stable-hover = Rilis stabil terbaru
menu-settings-updates-beta-hover = Build terbaru, termasuk pra-rilis
menu-settings-updates-on-launch = Periksa pembaruan saat dijalankan
menu-settings-updates-on-launch-hover = Jika tidak, periksa secara manual melalui Bantuan → Periksa pembaruan.
# Followed by its shortcut, (Ctrl+S).
menu-settings-snapshot = Cuplikan
menu-settings-snapshot-ask-hover = Tawarkan pilihan setiap kali ditekan
menu-settings-snapshot-screen-hover = Simpan tampilan persis seperti di layar (instan, PNG)
menu-settings-snapshot-render-hover = Render penuh dengan pengaturan Ekspor, di latar belakang (bisa memakan waktu lama pada tampilan dalam)
menu-settings-refcache = Cache referensi…
menu-settings-refcache-hover = Orbit referensi zoom dalam yang disimpan di disk, sehingga lokasi yang Anda kunjungi lagi selesai dirender dalam hitungan detik. Lokasinya, ukurannya, batasnya, dan cara mengosongkannya.

## Fractal menu

menu-fractal = Fraktal

## View menu

menu-view = Tampilan
menu-view-zoom-home = Perkecil ke tampilan penuh
menu-view-zoom-home-hover = Perkecil ke tampilan awal penuh (beranimasi)
menu-view-reset = Atur ulang ke tampilan bawaan
menu-view-reset-hover = Atur ulang ke tampilan bawaan fraktal (instan)
# Followed by its shortcuts.
menu-view-undo = Urungkan tampilan
menu-view-redo = Ulangi tampilan
menu-view-control-panel = Panel kontrol
menu-view-control-panel-hover = Tampilkan/sembunyikan panel kontrol di sisi kanan.
menu-view-minimap = Ikhtisar peta mini
menu-view-minimap-hover = Tampilkan ikhtisar kecil tampilan awal dengan penanda “Anda di sini” dan kedalaman zoom. Klik ikhtisar itu untuk melompat ke suatu wilayah.
menu-view-perf-panel = Panel kinerja
menu-view-timestamp = Tampilkan stempel waktu
menu-view-timestamp-hover = Gambar jam waktu berlalu yang besar di atas tampilan. Jam itu menunjukkan “+12.345s” yang sama dengan cap waktu log di setiap baris, sehingga rekaman layar dari masalah yang hanya muncul saat bergerak dapat dicocokkan dengan log, bingkai demi bingkai. Nonaktif secara bawaan.
menu-view-zoom-target = Tampilkan target zoom otomatis
menu-view-zoom-target-hover = Selama zoom otomatis berjalan, beri garis tepi pada wilayah tampilan yang sedang diperbesar — bagian yang akan memenuhi layar setelah beberapa perbesaran lagi — dengan garis dari sudut-sudutnya ke sudut-sudut layar, serta sebuah cincin pada titik yang sedang dituju. Nonaktif secara bawaan.
menu-view-live-panel = Panel render langsung
menu-view-live-panel-hover = Panel depan di atas tampilan yang memperlihatkan perender langsung saat bekerja: apa yang dikirim setiap bingkai ke GPU, seberapa lama umur gambar di layar, penelusuran melalui pita iterasi, pembangun referensi, dan mesin di baliknya. B mengganti ukurannya secara bergiliran, dan tombol × di sudutnya menutupnya; seret panel ke sudut mana pun; arahkan kursor ke sebuah instrumen untuk melihat artinya serta seperti apa keadaan sehat dan bermasalah.
# The live render panel's sizes.
menu-view-live-off = Mati
menu-view-live-off-hover = Tanpa panel.
menu-view-live-strip = Lajur
menu-view-live-strip-hover = Tampilan angka nixie dan deret lampu.
menu-view-live-panel-size = Panel
menu-view-live-panel-size-hover = Instrumen: dial penelusuran, umur gambar, petak, lintasan GPU, dial kedalaman, lampu.
menu-view-live-full = Penuh
menu-view-live-full-hover = Panel beserta ruang mesin: langkah, GPU, memori, inti, piksel.
menu-view-live-opacity = Opasitas
menu-view-watermark = Tampilkan tanda air “Fd”
menu-view-watermark-hover = Gambar tanda merek “Fd” yang samar di sudut kanan bawah tampilan langsung dan gambar yang diekspor. Aktif secara bawaan.

## Color menu

menu-color = Warna
menu-color-method = Metode
menu-color-palette = Palet
menu-color-custom = Kustom
menu-color-duotone = Dua nada
menu-color-binary = Biner (himpunan)
menu-color-binary-hover = Dua warna datar: di dalam himpunan vs di luar himpunan, tanpa gradien.
menu-color-edit-gradient = Sunting gradien…
menu-color-normalize = Normalkan warna dalam
menu-color-normalize-hover = Petakan ulang palet ke rentang lolos terukur dari tampilan ketika medan yang padat akan tampak seperti bintik-bintik. Tampilan yang tidak mengalami aliasing dibiarkan apa adanya — centang “Selalu sesuaikan rentang” untuk memetakan ulang dalam keadaan apa pun. Hanya untuk metode Iterasi halus.
menu-color-more = Siklus, pergeseran, animasi, dan efek: panel Kontrol ▸ Pewarnaan

## Tools menu

menu-tools = Alat
# Followed by its shortcut, (M).
menu-tools-find-minibrot = Temukan minibrot + zoom ke sana
menu-tools-find-minibrot-hover = Dengan metode Newton, tepatkan pusat tampilan ke pusat persis minibrot di dekatnya, laporkan periodenya, lalu zoom ke skala minibrot itu sendiri — sering kali banyak orde besaran dalam satu langkah. Sudah lebih dalam darinya? Hanya pusatnya yang berpindah.
menu-tools-solver = Pemecah Newton / Misiurewicz…
menu-tools-solver-hover = Cari pusat persis minibrot (periode + ukuran) atau titik Misiurewicz yang terdekat dengan tampilan — metode Newton dengan presisi penuh, beserta pengali tolak λ. Membuka dialog lokasi, tempat pemecah-pemecah itu berada.
# Followed by its shortcut, (A).
menu-tools-autopilot = Zoom otomatis (autopilot)
menu-tools-autopilot-stop = Hentikan autopilot
menu-tools-autopilot-hover = Zoom dalam berkelanjutan tanpa campur tangan: menyelam ke wilayah yang paling kaya detail, sambil terus mengoreksi arah. Masukan navigasi apa pun akan menghentikannya.
menu-tools-priority = Prioritas:
menu-tools-priority-speed = Kecepatan
menu-tools-priority-speed-hover = Zoom pada kecepatan yang diatur; segarkan dengan bingkai pertama yang memiliki detail.
menu-tools-priority-quality = Kualitas
menu-tools-priority-quality-hover = Hanya bingkai yang terselesaikan penuh; zoom menunggu bingkai itu.
menu-tools-target = Target:
menu-tools-target-detail = Detail
menu-tools-target-detail-hover = Tepi yang paling kaya dalam tampilan, dipilih ulang seiring penyelaman berlangsung.
menu-tools-target-misiurewicz = Titik Misiurewicz
menu-tools-target-misiurewicz-hover = Titik Misiurewicz terdekat, dihitung sekali lalu diselami secara persis (hanya Mandelbrot).
menu-tools-orbits = Lapisan orbit
menu-tools-orbits-hover = Gambar jalur iterasi titik di bawah kursor. Opsi animasi: panel Kontrol ▸ Lapisan.
menu-tools-orbits-normalize = Normalkan (sesuaikan dengan tampilan)
menu-tools-orbits-normalize-hover = Sesuaikan orbit dengan seluruh tampilan agar tetap terbingkai baik pada zoom berapa pun (alih-alih dipetakan melalui viewport, yang membuatnya melayang keluar layar saat tampilan dalam).
menu-tools-play-tour = Putar tur…
menu-tools-play-tour-hover = Pilih skrip tur untuk diputar. Tombol ▶ di bilah alat memutar ulang tur terakhir dengan sekali klik.
menu-tools-tour-from-view = Tur dari tampilan saat ini…
menu-tools-tour-from-view-hover = Buat tur yang melakukan zoom dari tampilan penuh hingga tampilan saat ini — tambahkan keterangan dan atur durasi penyelaman. Disimpan sebagai skrip tur (.toml) yang dapat Anda sunting, putar, atau render.
menu-tools-render-tour = Render tur…
menu-tools-render-tour-hover = Render tur yang dimuat menjadi urutan bingkai PNG (dan, jika mau, mp4) dalam proses terpisah.
menu-tools-close-player = Tutup pemutar tur
menu-tools-close-player-hover = Hentikan tur, tutup pemutarnya, dan pulihkan pengaturan iterasi dan pewarnaan Anda sendiri
menu-tools-render-farm = Render di farm…
menu-tools-render-farm-hover = Render tur yang dimuat di beberapa mesin: mesin ini mengendalikan pekerjaan, mesin lain bergabung melalui Berkas ▸ Klien render….
menu-tools-benchmark = Tolok ukur…
menu-tools-benchmark-hover = Ukur kecepatan render — dengan pengaturan saat ini, atau dengan proses terstandar (resolusi + pengaturan tetap) yang dapat dibandingkan antarmesin. Burn-in mengulanginya untuk memeriksa stabilitas / thermal throttling.
menu-tools-last-benchmark = Tampilkan tolok ukur terakhir

## Navigate menu

menu-navigate = Navigasi
menu-navigate-goto = Pergi ke lokasi…
menu-navigate-goto-hover = Masukkan pusat/zoom, lompat ke titik terkenal, atau pecahkan dengan metode Newton fitur Misiurewicz / minibrot di dekat tampilan.
menu-navigate-misiurewicz = Penjelajah Misiurewicz…
menu-navigate-misiurewicz-hover = Lihat rupa jenis-jenis Misiurewicz yang kecil — gambar mini dengan periode zoom dan puntiran setiap titik — dan pecahkan salah satunya dengan metode Newton hingga kedalaman sembarang. (Mandelbrot.)
menu-navigate-exact = Titik eksak (ekspresi)
menu-navigate-exact-note = Akar bulatan-bulatan kardioid utama, diberikan sebagai ekspresi eksak (cos/sin dari pecahan 2π). Koordinatnya disimpan sebagai ekspresi, sehingga zoom ke dalam tetap tepat pada titik itu.
menu-navigate-random = Lokasi acak
menu-navigate-random-hover = Lompat ke titik batas acak yang kaya detail
menu-navigate-import-kfr = Impor .kfr…
menu-navigate-import-kfr-hover = Muat berkas lokasi Kalles Fraktaler
menu-navigate-share = Bagikan lokasi…
menu-navigate-share-hover = Salin / tempel / simpan / muat lokasi mandiri (.fdn): fraktal, pusat dengan presisi penuh, zoom, pewarnaan.

## Help menu

menu-help = Bantuan
# Followed by its shortcut, (F1).
menu-help-reference = Bantuan & referensi…
menu-help-welcome = Selamat datang / mulai cepat…
menu-help-welcome-hover = Ikhtisar saat pertama kali dijalankan: kontrol-kontrol penting dan beberapa lokasi terkenal.
menu-help-diagnostics = Diagnostik…
menu-help-diagnostics-hover = Periksa apakah Fractadyne bekerja dengan benar di perangkat keras Anda, dan lampirkan hasilnya ke laporan masalah
menu-help-report = Laporkan masalah…
menu-help-report-hover = Ajukan issue GitHub (atau email) dengan laporan crash, log, dan lokasi terlampir
menu-help-accelerated = Zoom dalam lebih cepat…
menu-help-accelerated-hover = Build opsional yang menghitung orbit referensi zoom dalam 2,5–6,4x lebih cepat. Gambarnya identik; pengaturan Anda ikut terbawa.
menu-help-check-updates = Periksa pembaruan
# $track: "stable" or "beta", the update track in use (the variants hold the wording).
menu-help-check-updates-hover =
    { $track ->
        [beta] Periksa build Beta (pra-rilis) yang lebih baru di GitHub
       *[other] Periksa build Stabil yang lebih baru di GitHub
    }
# $version: the new version; $channel: "beta" for a pre-release, "stable" otherwise (the variants hold the wording).
menu-help-update-available =
    { $channel ->
        [beta] { $version } (beta) tersedia — unduh
       *[other] { $version } (stabil) tersedia — unduh
    }
menu-help-up-to-date = Sudah versi terbaru
menu-help-tagline = Penjelajah fraktal native
menu-help-license = Lisensi
# $year and $holder: the copyright year and holder, as the code gives them.
menu-help-copyright = © { $year } { $holder }. Berlisensi ganda; gunakan di bawah salah satu lisensi sesuai pilihan Anda.
menu-help-source = Kode sumber

## The toolbar (its buttons are icons; these are their tooltips)

menu-toolbar-julia = Julia
menu-toolbar-julia-hover = Tampilkan himpunan Julia dari rumus ini, memenuhi jendela
menu-toolbar-dual-hover = Keduanya, tertaut: himpunan di kiri, Julia dari titik kursor di kanan
menu-toolbar-open-hover = Buka tampilan atau lokasi (PNG / EXR, .fdn, .kfr)
menu-toolbar-gallery-hover = Galeri
menu-toolbar-export-hover = Ekspor gambar…
menu-toolbar-share-hover = Bagikan lokasi — salin, tempel, simpan, atau muat .fdn mandiri (fraktal, pusat dengan presisi penuh, zoom, pewarnaan)
menu-toolbar-zoom-in-hover = Perbesar
menu-toolbar-zoom-out-hover = Perkecil
# $factor: the click-zoom factor (4, 10…).
menu-toolbar-click-zoom-hover = Klik untuk zoom ({ $factor }×): klik kiri menyelam ke titik itu, klik kanan mundur (seret tetap menggeser). Faktornya di Kontrol ▸ Navigasi.
# Added to the click-to-zoom tooltip in the dual view.
menu-toolbar-click-zoom-dual = Kunci Julia c dengan Ctrl+klik selama ini aktif.
menu-toolbar-autopilot-running-hover = Zoom otomatis sedang berjalan — klik untuk menghentikan
menu-toolbar-autopilot-hover = Zoom otomatis: menyelam ke arah detail (A)
menu-toolbar-reset-hover = Atur ulang tampilan (instan)
menu-toolbar-home-hover = Perkecil ke tampilan awal (beranimasi)
menu-toolbar-bookmarks-hover = Markah: simpan tampilan ini, atau lompat ke tampilan yang tersimpan
# $name: the palette's name.
menu-toolbar-next-palette-hover = Palet berikutnya ({ $name })
# $file: the last tour's file name.
menu-toolbar-play-last-hover = Putar tur — { $file } (klik) · memutar ulang tur terakhir; pilih yang lain melalui Alat ▸ Putar tur…
menu-toolbar-play-hover = Putar tur… (Alat ▸ Putar tur)
menu-toolbar-speed-hover = Kecepatan pemutaran
menu-toolbar-fullscreen-hover = Alihkan layar penuh

## Dialogs: words shared by many windows

# A button that closes a window without doing what it offers.
dlg-cancel = Batal
# A button that dismisses a window (nothing in it is pending).
dlg-close = Tutup
# A button that copies the window's text to the clipboard (after a copy icon).
dlg-copy = Salin
# A button that jumps to a location.
dlg-go = Pergi
# A button that fills the fields from the view on screen.
dlg-use-current = Gunakan saat ini
# A notice after a file could not be written. $error: the system's reason, as it gives it.
dlg-save-failed = Gagal menyimpan: { $error }
# A notice when no graphics device is available to render with.
dlg-gpu-not-available = GPU tidak tersedia
# The name of the plain-text file type in a save dialog (the extension .txt is added by the code).
dlg-filter-text = Teks

## Asked after a session that did not shut down cleanly

dlg-crash-title = Fractadyne tidak ditutup dengan benar
dlg-crash-intro = Sesi sebelumnya berakhir secara tak terduga. Sebuah laporan telah disimpan di mesin ini — apakah Anda ingin mengirimnya?
dlg-crash-review = Tidak ada yang dikirim sebelum Anda meninjaunya: layar berikutnya menunjukkan persis apa yang akan disertakan, dan informasi sistem dapat dikecualikan dengan satu kotak centang.
dlg-crash-report = Laporkan…
dlg-crash-report-hover = Membuka Laporkan masalah, dengan laporan crash dan lokasi yang mengalami crash terlampir.
dlg-crash-not-now = Nanti saja
dlg-crash-dont-ask = Jangan tanya lagi setelah crash
dlg-crash-dont-ask-hover = Laporan tetap disimpan secara lokal, dan Bantuan ▸ Laporkan masalah selalu dapat digunakan.

## Asked after a session that ended with a second graphics device in use

dlg-second-device-title = Matikan perangkat grafis kedua?
# $device: the graphics device's name and details, as the system gives them.
dlg-second-device-intro = Sesi sebelumnya berakhir secara tak terduga saat perangkat grafis kedua sedang digunakan ({ $device }). Perangkat itu mungkin penyebabnya.
dlg-second-device-note = Jika dimatikan, kartu yang menggambar jendela akan merender semuanya, seperti sebelumnya. Anda dapat menyalakannya kembali di Lanjutan ▸ Kartu grafis kedua.
dlg-second-device-off = Matikan
dlg-second-device-keep = Biarkan menyala

## Shown when the previous session's iteration setting was set aside

dlg-quarantine-title = Pengaturan iterasi dikesampingkan
dlg-quarantine-change = Anda dapat mengubahnya kapan saja di Kontrol ▸ Kualitas.
dlg-quarantine-keep = Pertahankan bawaan
# A button. $setting: the iteration setting the previous session used (such as "20,000 iterations").
dlg-quarantine-restore = Pulihkan { $setting }

## Navigate ▸ Go to location

dlg-goto-title = Pergi ke lokasi
# Before two choices: "x, y" (cartesian) and the polar form below.
dlg-goto-enter-as = Masukkan sebagai:
# The polar way of entering a point: an offset plus a radius r at an angle θ. Keep the symbols.
dlg-goto-polar = pergeseran + r ∠ θ
# The tooltip of the "?" button beside the entry choice.
dlg-goto-help-hover = Apa saja yang diterima kolom: pecahan, pangkat, sqrt/sin/cos…, pi, dan konstanta lain — membuka Bantuan → Ekspresi koordinat
# Labels over the entry fields. x0, y0, r, θ, Re and Im are the fields' symbols: keep them.
dlg-goto-polar-offset = Pergeseran pusat  (x0, y0 — biarkan 0 untuk titik polar murni)
dlg-goto-polar-radius = Jari-jari r, sudut θ
dlg-goto-re = Pusat — real (Re)
dlg-goto-im = Pusat — imajiner (Im)
dlg-goto-zoom = Zoom (perbesaran)
# A warning under the zoom field, after a ⚠ sign. $have: the iteration count set; $typical: the count this depth typically needs.
dlg-goto-iter-warning = Iterasi ditetapkan pada { $have }; kedalaman ini biasanya memerlukan ~{ $typical }. Tampilan akan dirender datar sampai nilainya dinaikkan — atau aktifkan iterasi otomatis (Kontrol ▸ Kualitas), yang menyesuaikan dengan sendirinya.
dlg-goto-copy-hover = Salin lokasi ini ke papan klip
dlg-goto-cancel-hover = Tutup tanpa pergi ke mana pun
# The expressions in brackets are examples of what can be typed: keep them as they are.
dlg-goto-paste-note = Tempelkan pusat/zoom dari orang lain, atau Salin untuk berbagi. Koordinat menerima ekspresi: pecahan eksak (-3/4), fungsi dan konstanta (-0.5 + 0.25*cos(pi/4), sqrt, root(x,n), ^; sudut dalam radian), dan kolom real menerima nilai kompleks utuh ((37+16i)/100), yang mengisi kedua kolom.
# A section that opens and closes.
dlg-goto-feature = Pergi ke fitur (Mandelbrot)
dlg-goto-feature-poi = Titik terkenal:
# The dropdown's text before a point is picked.
dlg-goto-feature-poi-pick = Lompat ke titik menarik…
dlg-goto-feature-solve = Atau cari fitur di dekat tampilan saat ini:
# Two choices: what to solve for. Misiurewicz is a mathematician's name (a Misiurewicz point).
dlg-goto-feature-misiurewicz = Misiurewicz
dlg-goto-feature-minibrot = Minibrot terdekat
# Labels of two small number fields; k and p are their symbols.
dlg-goto-feature-preperiod = Praperiode k
dlg-goto-feature-period = Periode p
# The faint text in an empty field: left empty, the value is found automatically.
dlg-goto-feature-auto-hint = otomatis
# A small button that empties both fields, so they are found automatically again.
dlg-goto-feature-auto = Otomatis
dlg-goto-feature-auto-hover = Kosongkan kedua kotak agar pencarian berikutnya mendeteksi praperiode dan periode dari orbit di pusat tampilan.
dlg-goto-feature-kp-hover = Biarkan keduanya kosong untuk mendeteksinya dari orbit di pusat tampilan; nilai yang ditemukan akan diisikan di sini. Tidak ada orang yang melihat sebuah spiral lalu tahu praperiodenya, dan keharusan memberikannya itulah yang membuat pemecah ini tidak dapat dipakai dalam praktik.
# While a solve runs, followed by its elapsed time and the note below.
dlg-goto-feature-solving = Menghitung…
dlg-goto-feature-solving-note = (presisi sembarang; tampilan dalam memakan waktu lebih lama)
dlg-goto-feature-cancel-hover = Berhenti menunggu. Perhitungannya sendiri tetap berjalan hingga selesai di latar belakang dan jawabannya dibuang — tidak ada yang tertinggal dalam keadaan setengah jadi.
dlg-goto-feature-find = Cari di dekat tampilan
dlg-goto-feature-find-hover =
    Pecahkan dengan metode Newton dari pusat saat ini ke fitur yang persis (presisi sembarang). Navigasikan ke dekat fitur itu terlebih dahulu.

    Atur kolom Zoom di atas ke kedalaman yang Anda INGINKAN sebelum menghitung: titiknya dihitung dengan presisi yang dibutuhkan kedalaman itu, dan tampilan langsung melompat ke sana. Itu satu kali perhitungan, alih-alih rangkaian panjang langkah zoom manual.
dlg-goto-solve-cancelled = Perhitungan dibatalkan.

## Snapshot (the camera button): its tooltip, and the choice on the first press

# The camera button's tooltip, one per Snapshot setting. Followed by its shortcut, (Ctrl+S).
dlg-snapshot-hover-ask = Cuplikan — simpan tampilan seperti yang terlihat, atau render penuh; penekanan pertama akan bertanya
dlg-snapshot-hover-screen = Cuplikan — simpan tampilan persis seperti yang terlihat, ke folder ekspor terakhir
dlg-snapshot-hover-render = Cuplikan — render penuh dengan pengaturan Ekspor, ke folder ekspor terakhir
# The window's title.
dlg-snapshot-title = Cuplikan
dlg-snapshot-question = Apa yang harus disimpan oleh Cuplikan (tombol kamera, Ctrl+S)?
# The first choice, and a short tag beside it.
dlg-snapshot-screen = Tangkapan layar
dlg-snapshot-screen-tag = instan
dlg-snapshot-screen-note = Menyimpan tampilan persis seperti yang terlihat di layar, pada resolusi layar, sebagai PNG di folder ekspor terakhir. Apa yang Anda lihat itulah yang Anda dapatkan.
# The second choice, and a short tag beside it.
dlg-snapshot-render = Render penuh
dlg-snapshot-render-tag = mungkin memakan waktu lama
# $size: the image size, as 1920×1080; $ss: the supersampling factor (1–4); $format: PNG or EXR.
dlg-snapshot-render-note = Merender dengan pengaturan Ekspor — { $size }, supersampling { $ss }×, { $format }. Kualitas terbaik, tetapi tampilan dalam dapat memakan waktu beberapa menit hingga berjam-jam. Proses ini berjalan di latar belakang; Berkas → Ekspor gambar… menampilkan kemajuannya dan dapat membatalkannya.
dlg-snapshot-remember = Ingat pilihan saya dan jangan tanya lagi
dlg-snapshot-change = Anda dapat mengubahnya kapan saja di Berkas → Pengaturan → Cuplikan.

## Share location (.fdn)

dlg-share-title = Bagikan lokasi
dlg-share-intro = Lokasi mandiri (fraktal, pusat dengan presisi penuh, zoom, pewarnaan). Salin untuk membagikannya, atau tempel/muat sebuah lokasi lalu klik Terapkan.
dlg-share-thumb = Sertakan gambar mini dalam berkas yang disimpan
dlg-share-thumb-hover = Simpan .fdn menyematkan gambar kecil tampilan ini agar berkasnya dapat dikenali di galeri. Menambah sekitar 55 KB, dan bukan bagian dari teks di atas.
dlg-share-copy-hover = Salin teks ke papan klip
# Buttons (the first after a save icon).
dlg-share-save = Simpan .fdn…
dlg-share-load = Muat .fdn…
dlg-share-apply = Terapkan
dlg-share-apply-hover = Lompat ke lokasi di dalam kotak
dlg-share-copied = Disalin ke papan klip.

## Help ▸ Report an issue (the report itself stays in English: it is read by the maintainer)

dlg-report-title = Laporkan masalah
dlg-report-intro = Tempat terbaik untuk melapor adalah issue GitHub — terbuka untuk umum dan dapat dicari, dan Anda akan melihat kapan masalahnya diperbaiki. Email juga bisa. Tinjau dulu semua yang ada di bawah: tidak ada yang keluar dari mesin Anda sampai Anda bertindak.
# Labels before three dropdowns.
dlg-report-type = Jenis:
dlg-report-severity = Tingkat keparahan:
dlg-report-repro = Dapat direproduksi:
dlg-report-describe = Jelaskan apa yang terjadi:
# The faint text in the empty description box.
dlg-report-describe-hint = Apa yang terjadi, langkah-langkah untuk mereproduksinya, dan apa yang Anda harapkan sebagai gantinya.
# Before the checkboxes of what the report carries.
dlg-report-include = Sertakan:
dlg-report-sysinfo = Info sistem (versi, OS, CPU, GPU, VRAM)
dlg-report-location = Lokasi saat ini (.fdn)
dlg-report-log = Log terbaru
dlg-report-frames = Catatan bingkai (satu menit terakhir, dan 40 bingkai terakhir)
dlg-report-crash = Laporan crash terbaru
dlg-report-crash-none = Laporan crash terbaru (tidak ditemukan)
dlg-report-crash-view = Lokasi yang mengalami crash (.fdn)
dlg-report-crash-view-none = Lokasi yang mengalami crash (.fdn) (tidak ditemukan)
dlg-report-crash-view-hover = Tampilan persis tempat crash atau kehilangan perangkat grafis terjadi — satu-satunya hal yang memungkinkan pengelola mereproduksinya. Direkam secara otomatis.
dlg-report-test = Hasil uji diagnostik
dlg-report-test-none = Hasil uji diagnostik (jalankan dari Bantuan → Diagnostik…)
dlg-report-test-hover = Hasil lulus/gagal yang divalidasi mesin dari perangkat keras Anda sendiri — jauh lebih berguna bagi pengelola daripada deskripsi saja
# A section that opens and closes. $size: the report's size, as "12 KB".
dlg-report-preview = Pratinjau apa yang akan dikirim  ({ $size })
dlg-report-github = Buka issue GitHub
dlg-report-github-hover = Buka halaman issue baru dengan judul yang sudah terisi; laporan disalin ke papan klip Anda — tempelkan ke isi issue (Ctrl+V).
# Buttons (after a copy and a save icon).
dlg-report-copy = Salin laporan
dlg-report-copy-hover = Salin semuanya ke papan klip
dlg-report-save = Simpan laporan…
dlg-report-save-hover = Simpan sebagai .txt untuk dilampirkan
dlg-report-by-email = Atau melalui email:
dlg-report-gmail = Tulis di Gmail
# $address: the email address reports go to.
dlg-report-gmail-hover = Buka jendela penulisan Gmail ke { $address } dengan subjek yang sudah terisi; tempelkan laporan yang disalin
dlg-report-email = Aplikasi email…
dlg-report-email-hover = Buka aplikasi email bawaan Anda (mailto) — memerlukan penangan email yang sudah dikonfigurasi
dlg-report-tip = Tips: laporan disalin ke papan klip Anda pada setiap pilihan ini — tempelkan ke isi issue atau pesan (Ctrl+V). Jika bisa, lampirkan juga tangkapan layar atau contoh gambar (seret ke dalamnya); itu sangat membantu untuk masalah render dan UI.
# Notices in the window after a button is pressed.
dlg-report-opened-github = GitHub dibuka — laporan ada di papan klip; tempelkan ke issue.
dlg-report-copied = Laporan disalin ke papan klip.
# $path: where the report was saved.
dlg-report-saved = Disimpan ke { $path }
dlg-report-opened-email = Aplikasi email Anda dibuka — laporan ada di papan klip; tempelkan ke pesan.
dlg-report-opened-gmail = Jendela penulisan Gmail dibuka — tempelkan laporan yang disalin ke isinya (Ctrl+V).

## A location whose checksum does not match

dlg-checksum-title = Lokasi ini mungkin rusak
# After a ⚠ sign.
dlg-checksum-mismatch = Checksum-nya tidak cocok dengan data di sampingnya.
dlg-checksum-why = Itu biasanya berarti teksnya terpotong atau berubah dalam perjalanan ke sini — salinan sebagian dari jendela obrolan atau kiriman forum adalah penyebab yang umum.
# A label over where the location came from (a file path, or the clipboard).
dlg-checksum-source = Sumber
# The two checksums, side by side: the one written in the file, and the one computed from its data.
dlg-checksum-in-file = di berkas
dlg-checksum-from-data = dari data
dlg-checksum-load-note = Anda tetap dapat memuatnya — koordinatnya mungkin masih baik, dan tidak ada yang ditimpa. Apa pun yang tidak terbaca akan dilaporkan setelah dimuat.
dlg-checksum-load = Tetap muat
dlg-checksum-load-hover = Lompat ke sana meskipun tidak cocok
# Notices after loading it. $zoom: the magnification, as 1.2e148.
dlg-checksum-loaded = Dimuat @ { $zoom }× meskipun checksum tidak cocok
# $zoom: the magnification, as 1.2e148; $note: what the loader found wrong with the location.
dlg-checksum-loaded-note = Dimuat @ { $zoom }× — { $note }
dlg-checksum-not-loaded = Tidak dimuat.

## File ▸ Reset application state

dlg-reset-title = Atur ulang status aplikasi
# After a ⚠ sign, followed by a list.
dlg-reset-warning = Ini menghapus secara permanen semua data Fractadyne yang tersimpan:
# The list's items, each after a bullet.
dlg-reset-session = sesi yang tersimpan (tampilan saat ini, pewarnaan, preferensi)
dlg-reset-bookmarks = semua markah beserta gambar mininya
dlg-reset-refcache = cache orbit referensi (dibangun ulang saat Anda mengunjungi kembali lokasi)
# A label over the folder that is deleted.
dlg-reset-location = Lokasi
dlg-reset-note = Ini tidak dapat dibatalkan. Sesi saat ini tidak akan disimpan ulang saat keluar, sehingga nilai bawaan dimuat pada peluncuran berikutnya.
dlg-reset-confirm = Atur ulang semuanya
dlg-reset-done = Status aplikasi telah diatur ulang — nilai bawaan akan dimuat pada peluncuran berikutnya.
# $error: the system's reason, as it gives it.
dlg-reset-failed = Gagal mengatur ulang: { $error }

## Tools ▸ Tour from current view

dlg-tour-title = Tur dari tampilan saat ini
dlg-tour-intro = Membuat tur yang melakukan zoom dari tampilan penuh hingga posisi Anda sekarang.
# A label before the fractal and magnification the tour ends at.
dlg-tour-destination = Tujuan
dlg-tour-notation = Catatan (keterangan opsional yang ditampilkan selama penyelaman)
# The faint example text in the empty caption box.
dlg-tour-notation-hint = mis. Menyelam ke Misiurewicz tiga lengan…
dlg-tour-duration = Durasi zoom
# $rate: how fast the dive zooms, in powers of ten a second (as 2.5).
dlg-tour-rate = ≈ { $rate } dekade/s
# A small button that puts the duration back to its default.
dlg-tour-reset = Atur ulang
dlg-tour-reset-hover = Bawaan berdasarkan kedalaman
dlg-tour-duration-note = Nilai bawaan diskalakan sesuai kedalaman agar penyelaman tetap nyaman ditonton. Penyelaman yang lebih cepat dapat memunculkan kilatan bingkai satu warna pada zoom ekstrem.
# A button, after a save icon.
dlg-tour-save = Simpan tur…
# The file type's name in the save dialog.
dlg-tour-filter = Skrip Fractadyne (TOML)
# A notice. $file: the saved file's name. --render-tour is a command-line option: keep it.
dlg-tour-saved = Tur disimpan → { $file }. Putar melalui Alat → Putar tur, atau render dengan --render-tour.

## Navigate ▸ Bookmarks ▸ Manage…

dlg-bookmarks-title = Markah
# The faint text in the empty name box.
dlg-bookmarks-name-hint = nama (opsional)
# A button, after a bookmark icon.
dlg-bookmarks-add = Tambahkan tampilan saat ini
dlg-bookmarks-empty = Belum ada markah. Tambahkan dari tampilan saat ini.
# The tooltip of a bookmark's delete (bin) button.
dlg-bookmarks-delete-hover = Hapus

## Tools ▸ Benchmark…

dlg-bench-title = Tolok ukur
dlg-bench-mode = Mode
dlg-bench-current = Pengaturan saat ini
dlg-bench-current-hover = Putar tur zoom dalam yang tetap di tampilan langsung dengan resolusi dan pengaturan Anda saat ini.
dlg-bench-standard = Terstandar
dlg-bench-standard-hover = Tetapkan resolusi + semua pengaturan render agar skornya dapat dibandingkan antarmesin.
dlg-bench-resolution = Resolusi
dlg-bench-depth = Kedalaman
# Standard and Ultra are the two depths in the dropdown this describes.
dlg-bench-depth-hover = Kedua penyelaman masuk ke jalur floatexp/BLA yang dalam; Ultra (1e48×) menghabiskan waktu jauh lebih lama di sana daripada Standar (1e32×), dan karenanya juga butuh waktu lebih lama untuk dijalankan.
dlg-bench-burnin = Burn-in (ulang)
dlg-bench-burnin-hover = Jalankan tolok ukur berulang kali untuk mengungkap stabilitas dan thermal throttling.
# The label of a slider: how many times the benchmark runs.
dlg-bench-passes = putaran
# $size: the image size, as 1920×1080; $ss: the supersampling factor; $depth: the dive's final magnification, as 1e32.
dlg-bench-std-summary = Merender di luar layar pada { $size }, SS { $ss }×, Mandelbrot/halus, penyelaman 60 bingkai ke { $depth }×.
dlg-bench-run = Jalankan

## The standardized benchmark's progress window

dlg-bench-progress-title = Menjalankan tolok ukur…
# $resolution: the benchmark's resolution, as the dropdown names it.
dlg-bench-progress-mode = Terstandar · { $resolution }
# $pass: the pass running now; $total: how many passes there are.
dlg-bench-progress-pass = Putaran burn-in { $pass }/{ $total }
dlg-bench-progress-dive = Merender penyelaman tetap…
# On the progress bar. $done: frames rendered; $total: frames in the dive.
dlg-bench-progress-frame = bingkai { $done }/{ $total }
# $fps: frames per second, as 41.3.
dlg-bench-progress-last = putaran terakhir: { $fps } fps

## The benchmark's results

dlg-bench-results-title = Hasil tolok ukur
# Buttons, each after an icon.
dlg-bench-results-run-again = Jalankan lagi…
dlg-bench-results-save = Simpan…
dlg-bench-results-none = Belum ada tolok ukur yang dijalankan.
dlg-bench-results-saved = Tolok ukur disimpan.

## The accelerated build's notice when its arithmetic libraries are missing

dlg-backend-title = Build terakselerasi — menggunakan aritmetika bawaan
dlg-backend-dont-show = Jangan tampilkan lagi

## File ▸ Gallery…

dlg-gallery-title = Galeri
# A button that picks the folder to show.
dlg-gallery-folder = Folder…
# A button, after a refresh icon.
dlg-gallery-refresh = Segarkan
dlg-gallery-empty = Tidak ada gambar Fractadyne di folder ini.
# An image's title when it does not name its fractal.
dlg-gallery-untitled = Gambar Fractadyne
# $zoom: the image's magnification, as 1.2e148; $saved: when it was saved, as the file gives it.
dlg-gallery-zoom = zoom { $zoom }   { $saved }
# The notes saved with the image, in quotation marks. Use your language's quotation marks.
dlg-gallery-notes = “{ $notes }”
dlg-gallery-open = Buka tampilan ini
dlg-gallery-loaded = Tampilan dimuat dari galeri.

## File ▸ Export image…

dlg-export-title = Ekspor gambar
# A dropdown of image sizes.
dlg-export-size = Ukuran
# The size dropdown's text for a size set by hand. $size: the size, as 1920×1080.
dlg-export-size-custom-current = Kustom — { $size }
# The size dropdown's first choice.
dlg-export-size-custom = Kustom…
dlg-export-size-custom-hover = Atur sendiri lebarnya; tinggi mengikuti Rasio di bawah.
dlg-export-width = Lebar (px)
# After the width field. $height: the height in pixels, which follows from the aspect.
dlg-export-height-from-aspect = × { $height } (dari Rasio)
dlg-export-ss = Supersampling
dlg-export-aspect = Rasio
# The aspect choice that keeps the window's own shape.
dlg-export-aspect-window = Sesuai jendela
# Before the choices PNG and OpenEXR.
dlg-export-format = Format:
dlg-export-hud = HUD lokasi
dlg-export-hud-hover = Cetak permanen panel tingkat zoom + koordinat di kiri atas gambar yang diekspor (menyesuaikan skala keluaran; menampilkan zoom + pusat tampilan peta/Mandelbrot).
dlg-export-glitch = Koreksi glitch
dlg-export-glitch-hover = Koreksi glitch perturbasi dengan banyak referensi. Otomatis dilewati untuk ekspor tampilan tunggal yang sangat dalam (floatexp) agar pembangunan referensi + render berjalan di luar thread utama dan aplikasi tetap responsif. Tidak pernah digunakan untuk tampilan Mandelbrot atau Multibrot (di luar mode Julia): saat diperiksa terhadap perhitungan eksak, koreksi ini tidak memperbaiki apa pun di sana dan memakan hingga 5x waktu ekspor.
dlg-export-deep-note = Ekspor dalam: referensi dibangun di luar thread utama (UI tetap aktif); koreksi glitch dilewati pada kedalaman ini.
# A dropdown, shown in the dual (map + Julia) view, and its three choices.
dlg-export-dual = Tata letak ganda
dlg-export-dual-side = Berdampingan
dlg-export-dual-separate = Berkas terpisah
dlg-export-dual-map = Hanya peta
dlg-export-notes = Catatan:
# The faint text in the empty notes box.
dlg-export-notes-hint = disimpan bersama gambar (≤120 karakter)
# $width and $height: the image size in pixels; $left: how many characters the notes can still take.
dlg-export-output = Keluaran: { $width } × { $height } px   (sisa { $left } karakter)
# When the export's shape differs from the window's. $percent: how much more, as 25.
dlg-export-extra-width = Menampilkan { $percent }% lebih lebar daripada jendela (tidak ada yang dipotong)
# The same, when the export is taller than the window. $percent: how much more, as 25.
dlg-export-extra-height = Menampilkan { $percent }% lebih tinggi daripada jendela (tidak ada yang dipotong)
dlg-export-tiles-note = Dirender dalam petak (tanpa batas ukuran) pada thread latar belakang. Berkas itu menyematkan tampilan + catatan agar dapat dibuka kembali melalui Berkas ▸ Buka tampilan.
# A label before the button that picks the folder exports go to.
dlg-export-folder = Folder:
dlg-export-choose = Pilih…
dlg-export-choose-hover = Pilih direktori tujuan untuk ekspor
dlg-export-open-after = Buka setelah selesai
dlg-export-open-after-hover = Serahkan gambar yang sudah selesai ke penampil sistem begitu berkasnya ditulis.
# $count: the graphics cards in this machine (2 or more).
dlg-export-all-gpus = Gunakan semua kartu grafis ({ $count })
dlg-export-all-gpus-hover = Bagi gambar per petak ke semua kartu grafis di mesin ini, masing-masing dengan salinan referensi dalamnya sendiri. Kartu yang identik menghasilkan gambar yang persis sama dengan yang dirender satu kartu saja; model yang berbeda menggambar beberapa piksel sedikit berbeda, petak demi petak (terukur: tidak ada sambungan yang terlihat). Jika sebuah kartu gagal, petaknya dialihkan ke kartu lain. Gambar yang tidak dapat dibagi (beberapa metode pewarnaan dan rumus) menggunakan kartu jendela ini.
# The export's progress.
dlg-export-preparing = Menyiapkan (membangun referensi)…
dlg-export-saving = Menyimpan…
dlg-export-rendering = Merender…
# $time: the time since the export started, as 8.3s or 1m 04.0s.
dlg-export-elapsed = Berlalu: { $time }
dlg-export-cancel-hover = Tutup tanpa mengekspor
# A button, after a save icon.
dlg-export-save-as = Simpan sebagai…
dlg-export-save-as-hover = Pilih nama dan lokasi berkas
dlg-export-export = Ekspor
dlg-export-export-hover = Render dan simpan ke folder di atas (nama otomatis)

## Notices worded at startup (main.rs)

# A notice shown once after starting, when the saved session comes from a newer version of the app. $saved and $handled: version numbers of the session file's format (1, 2…).
app-state-newer = Sesi ini disimpan oleh Fractadyne yang lebih baru (state v{ $saved }; build ini menangani v{ $handled }). Beberapa pengaturan mungkin tidak berlaku, dan saat disimpan, sesi ini akan ditulis ulang dalam format build ini.
# The title of the notice that says the accelerated build's arithmetic libraries are found again (its text comes from elsewhere).
app-notice-mpfr-restored-title = Aritmetika terakselerasi dipulihkan
# After Fractadyne restarted itself because the graphics driver reset the GPU.
app-toast-gpu-recovered = Pulih dari reset perangkat grafis — dibuka kembali di tampilan awal, karena lokasi sebelumnya terlalu berat untuk dirender.

## Short notices (toasts)

# When the autopilot reached a view that could not be rendered and put the last good view back.
app-toast-autopilot-view-restored = Autopilot berhenti: tampilan menjadi tidak dapat digunakan dan dikembalikan (dicatat di log)
# $error: the system's reason, as it gives it (often in English).
app-toast-gradients-save-failed = Tidak dapat menyimpan gradien: { $error }
# $error: the reason, as the program gives it (often in English).
app-toast-gradients-serialize-failed = Tidak dapat menserialisasi gradien: { $error }
# $name: the name the user gave the gradient.
app-toast-gradient-saved = Gradien “{ $name }” disimpan.
# $error: the reason, as the program gives it (often in English).
app-toast-bookmarks-serialize-failed = Tidak dapat menserialisasi markah: { $error }
# $error: the system's reason, as it gives it (often in English).
app-toast-bookmarks-save-failed = Tidak dapat menyimpan markah: { $error }
# Jumping to a named point given by exact expressions. $name: the point's name (in English).
app-toast-coordinates-failed = { $name }: koordinat tidak dapat dievaluasi
# $zoom: the magnification, already formatted (1,234.5 or 1.2e148).
app-toast-random-location = Lokasi acak · { $zoom }×
# Importing a Kalles Fraktaler .kfr location. $error: the reason (in English).
app-toast-import-failed = Impor gagal: { $error }
# $zoom: the magnification, already formatted.
app-toast-location-loaded = Lokasi dimuat @ { $zoom }×

## Go to location: why a jump could not be made (one line per field that failed)

# "Re" is the dialog's real-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-re = Re: { $error }
# "Im" is the dialog's imaginary-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-im = Im: { $error }
# The imaginary field was given a complex value. $error: the evaluator's reason (in English).
app-goto-error-im-real = Im: { $error } — ketik bagian imajiner sebagai bilangan real, tanpa i
# "Zoom" is the dialog's magnification field.
app-goto-error-zoom-empty = Zoom: masukkan perbesaran, mis. 1e50 atau 250000
# $zoom: what was typed, in quotes.
app-goto-error-zoom = Zoom: tidak dapat membaca { $zoom } — masukkan perbesaran positif seperti 1e50 atau 250000
app-goto-error-invalid = Masukan tidak valid — periksa koordinat dan zoom.

## Finding a minibrot or Misiurewicz point (the M key and the Go to dialog)

app-feature-needs-holomorphic = Pencari minibrot memerlukan keluarga holomorfik (Mandelbrot / Multibrot).
app-feature-not-in-julia = Pencari minibrot mencari di himpunan Mandelbrot — matikan mode Julia untuk menggunakannya.
app-feature-busy = Sedang mencari fitur — satu per satu.
app-feature-searching = Mencari minibrot terdekat…
app-feature-mandelbrot-only = Pencarian fitur hanya untuk Mandelbrot.
app-feature-no-pair = Tidak ditemukan titik Misiurewicz dalam anggaran pencarian — titik itu mungkin lebih dalam daripada jangkauan pencari pada tampilan ini, atau pusatnya mungkin tidak berada di dekat spiral atau titik cabang.
# The name of what was found, used as $feature in the messages below. $period: the minibrot's period (a whole number).
app-feature-minibrot = minibrot periode { $period }
# $preperiod and $period: the point's two whole numbers (k, p). $octaves: how often the view repeats when zooming in (2 decimals); $twist: the turn per repeat in degrees.
app-feature-misiurewicz-lambda = Misiurewicz ({ $preperiod },{ $period }) — berulang setiap { $octaves } oktaf, puntiran { $twist }°
# $preperiod and $period: the point's two whole numbers (k, p).
app-feature-misiurewicz = Misiurewicz ({ $preperiod },{ $period })
# Added after the messages below when the fixed iteration count is too low for the new depth. $have: the iterations set now; $typical: what the depth typically needs.
app-feature-iter-note = ⚠ Iterasi ditetapkan pada { $have }; kedalaman ini biasanya memerlukan ~{ $typical } — naikkan (atau aktifkan iterasi otomatis) jika tampilan dirender datar.
# $feature: what was found (see above). $zoom and $asked: magnifications, already formatted; $octaves: the solver's depth limit.
app-feature-zoomed-capped = Zoom ke { $feature } — { $zoom }×. Perhitungan dibatasi hingga { $octaves } oktaf, jadi berhenti di sana alih-alih di { $asked }×.
# $feature: what was found. $zoom: the magnification, already formatted. $seconds: how long the search took (1 decimal).
app-feature-zoomed = Zoom ke { $feature } — { $zoom }× ({ $seconds }s)
# $feature: what was found. $seconds: how long the search took (1 decimal).
app-feature-snapped = Ditepatkan ke pusat { $feature } ({ $seconds }s)
app-feature-no-minibrot = Tidak ditemukan pusat minibrot di dekat tampilan — zoom lebih dekat ke salah satunya.
# $feature: the point asked for, e.g. Misiurewicz (3,1). $widths: how many view-widths away, already formatted. $octaves: how far to zoom out.
app-feature-just-outside = Ditemukan titik { $feature } tepat di luar tampilan ini — sejauh { $widths } lebar tampilan. Perkecil sekitar { $octaves } oktaf untuk membawanya ke layar, atau atur praperiode yang lebih besar untuk mencari fitur pada skala ini.
# $feature: the point asked for. $widths: how many view-widths away, already formatted (may be 2^187). $octaves: how far to zoom out.
app-feature-far = Ditemukan titik { $feature }, tetapi jaraknya { $widths } lebar tampilan — fitur itu jauh lebih KASAR daripada tampilan ini. PERKECIL sekitar { $octaves } oktaf untuk mencapainya, atau coba praperiode yang lebih besar untuk mencari fitur pada skala ini.
# $feature: the point asked for. $residual: a small number in scientific notation.
app-feature-not-preperiodic = { $feature } tidak cocok dengan orbit di sini (residu { $residual }) — kosongkan kedua kotak untuk mendeteksi pasangannya dari tampilan.
# $feature: the point asked for. "k/p" are the dialog's two fields.
app-feature-not-converged = Tidak ada titik { $feature } yang konvergen di dekat tampilan — navigasikan lebih dekat, atau coba k/p yang berbeda.
app-feature-bad-request = Praperiode dan periode keduanya harus positif.

## Share location dialog and location files

app-location-empty = Tidak ada yang dapat dimuat (atau teksnya terlalu besar).
app-location-not-fractadyne = Bukan lokasi Fractadyne.
# $zoom: the magnification, already formatted. $note: what in the text was ignored or repaired (in English).
app-share-loaded-note = Dimuat @ { $zoom }× — { $note }.
# $error: the system's reason (often in English).
app-share-save-failed = Gagal menyimpan: { $error }
app-share-saved = Disimpan.
app-share-saved-rendering-thumb = Disimpan. Merender gambar mini…
app-share-saved-no-thumb = Disimpan — tanpa gambar mini (tidak ada yang dapat dirender untuk tampilan ini).
app-share-saved-with-thumb = Disimpan, dengan gambar mini.
# $error: the system's reason (often in English).
app-share-saved-thumb-failed = Disimpan, tetapi gambar mini tidak dapat ditambahkan: { $error }
app-share-loaded-into-box = Dimuat ke dalam kotak — tinjau, lalu Terapkan.
# Reading a file failed (a location or a palette). $error: the system's reason (often in English).
app-read-failed = Gagal membaca: { $error }
app-share-too-large = Berkas terlalu besar (bukan lokasi .fdn?).

## File-dialog filter names (the extensions are added by the code)

app-filter-kfr = Lokasi Kalles Fraktaler
app-filter-fdn = Lokasi Fractadyne
app-filter-map = Palet Fractint / KF
app-filter-ugr = Gradien Ultra Fractal
app-filter-ase = Adobe Swatch Exchange
app-filter-ggr = Gradien GIMP
app-filter-all = Semua berkas

## Importing and saving palette files (shown under the gradient editor)

# The reason a file was refused before reading it.
app-gradient-map-too-large = terlalu besar untuk menjadi palet .map
app-gradient-ugr-too-large = terlalu besar untuk menjadi berkas palet .ugr
app-gradient-ase-too-large = terlalu besar untuk menjadi daftar swatch .ase
app-gradient-ggr-too-large = terlalu besar untuk menjadi gradien .ggr
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported-vga = { $count } entri diimpor dari { $file } - tabel VGA 6-bit (putih bernilai 252, bukan 255), dipertahankan apa adanya agar cocok dengan keluaran Fractint sendiri.
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported = { $count } entri diimpor dari { $file }, sebagai pita tegas.
# $error: the reader's reason (in English).
app-gradient-map-invalid = Bukan palet .map: { $error }
# $name: the gradient's name in the file; $file: the file's name.
app-gradient-ugr-imported = “{ $name }” diimpor dari { $file }.
# $file: the file's name; $count: how many gradients it holds.
app-gradient-ugr-pick = { $file } berisi { $count } gradien — pilih salah satu di bawah.
# $error: the reader's reason (in English).
app-gradient-ugr-invalid = Bukan berkas palet .ugr: { $error }
# $used: the swatches kept; $total: the swatches in the file.
app-gradient-ase-imported-some = { $used } dari { $total } swatch diimpor, dengan jarak merata.
# $count: the swatches in the file.
app-gradient-ase-imported = { $count } swatch diimpor, dengan jarak merata dan dibaurkan.
# $error: the reader's reason (in English).
app-gradient-ase-invalid = Bukan daftar swatch .ase: { $error }
app-gradient-nothing-to-save = Belum ada gradien kustom untuk disimpan.
# $file: the saved file's name.
app-gradient-ggr-saved = “{ $file }” disimpan — titik tengah, kurva paduan, dan ruang warna semuanya utuh.
# $file: the saved file's name; $count: the segments whose curve could not be kept.
app-gradient-ggr-saved-lossy = “{ $file }” disimpan, tetapi { $count } segmen menggunakan kurva Bézier yang tidak dapat direpresentasikan oleh format .ggr. Segmen terkait ditulis dengan interpolasi linear; salinan di pustaka Anda tetap mempertahankan kurvanya.
# $path: the file; $error: the system's reason (often in English).
app-gradient-write-failed = Tidak dapat menulis { $path }: { $error }
# $name: the gradient's name; $count: its segments.
app-gradient-ggr-imported = “{ $name }” diimpor - { $count } segmen, beserta titik tengah, kurva paduan, dan ruang warnanya.
# $error: the reader's reason (in English).
app-gradient-ggr-invalid = Bukan gradien .ggr: { $error }

## The eyedropper's banner across the top of the screen while picking a colour

# $hex: the colour under the pointer, as #rrggbb.
app-eyedropper-banner-color = Memilih warna  ·  { $hex }  ·  klik untuk mengambilnya, Esc untuk membatalkan
app-eyedropper-banner = Memilih warna  ·  klik untuk mengambilnya, Esc untuk membatalkan

## The gradient editor window

app-gradient-title = Penyunting gradien
# Before the two view buttons, Bar and Ring.
app-gradient-view = Tampilan
app-gradient-view-bar = Batang
app-gradient-view-bar-hover = Lajur lurus — paling baik untuk menempatkan titik warna secara tepat
app-gradient-view-ring = Cincin
app-gradient-view-ring-hover = Cincin, karena palet itu BERSIKLUS: pertemuan di bagian atas adalah sambungan yang dilintasi perender pada setiap sapuan, dan gradien yang tampak baik pada batang bisa memiliki tepi tajam di sana.
app-gradient-add-lane-hover = Klik di mana saja pada garis ini untuk menambahkan titik warna di sana. Tombol + menambahkan satu di celah terlebar.
app-gradient-strip-hover = Seret penanda untuk memindahkan titik warnanya · seret lajur itu sendiri untuk MEMUTAR seluruh gradien, yang membungkus di sambungan · klik ganda untuk menambahkan titik warna, klik kanan penanda untuk menghapusnya · tombol panah menggeser pilihan sedikit (Shift untuk langkah halus).
app-gradient-remove-selected-hover = Hapus titik warna yang dipilih
app-gradient-ring-hover = Seret cincin itu sendiri untuk MEMUTAR seluruh gradien — apa yang melewati sambungan akan muncul kembali dari sisi lain · seret penanda untuk memindahkan titik warnanya · klik jalur luar untuk menambahkan titik warna di sana, atau ⊕ di tengah untuk menambahkan satu di celah terlebar · ⊖ menghapus titik warna yang dipilih. Tanda berongga di bagian atas adalah sambungan, tempat palet bersiklus membungkus; tanda itu tetap di tempatnya karena tanda itulah pertemuannya, dan warna-warnanya berputar di bawahnya.
app-gradient-empty = Belum ada gradien kustom — “Salin preset…”, sebuah impor, atau gradien tersimpan di bawah akan memulainya, lalu titik-titik warna dan kurva per segmennya muncul di sini.
# $count: the entries of an imported palette, more than the editor shows stop by stop.
app-gradient-too-many = { $count } entri yang diimpor - terlalu banyak untuk disunting satu per satu. Gunakan pratinjau di atas, atau Salin preset untuk memulai dari awal.
# The selected segment's heading. $index: its number; $total: how many there are; $left and $right: where it starts and ends, 0.000–1.000.
app-gradient-segment-heading = Segmen { $index } dari { $total }  ·  { $left } → { $right }
# A label before the list of blend curves.
app-gradient-curve = Kurva
app-gradient-curve-hover = Seberapa cepat warna bergerak melintasi segmen ini. “Lengkung” dibentuk oleh titik tengah — pada titik tengah 0.50 kurvanya persis linear.
# The blend curves (GIMP's kinds), in a narrow list: keep them short.
app-gradient-blend-linear = Linear
app-gradient-blend-curved = Lengkung
app-gradient-blend-sine = Sinus
app-gradient-blend-sphere-increasing = Bola ↑
app-gradient-blend-sphere-decreasing = Bola ↓
app-gradient-blend-bezier = Bézier
# Hover on Bézier in the list of blend curves.
app-gradient-bezier-exact-hover = Dua titik kendali yang dapat Anda seret ke dua arah. Kurva ini sudah kubik, jadi mengonversinya tidak mengubah apa pun.
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-visible-hover = Dua titik kendali yang dapat Anda seret ke dua arah. Konversi MENDEKATI kurva saat ini — yang ini dengan selisih hingga { $error }, yang jelas terlihat.
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-barely-hover = Dua titik kendali yang dapat Anda seret ke dua arah. Konversi MENDEKATI kurva saat ini — yang ini dengan selisih hingga { $error }, yang nyaris tidak terlihat.
# A label before the list of colour spaces (RGB, HSV).
app-gradient-space = Ruang
# A label before the Bézier presets beside it.
app-gradient-ease = Easing
app-gradient-ease-hover = Preset menuliskan kedua titik kendali; seret titik-titik itu pada kurva untuk bentuk lainnya. Bézier tidak memiliki titik tengah — pegangannya menentukan di mana kurva mencapai setengah jalan.
# Bézier presets, as in CSS easing: short buttons.
app-gradient-ease-linear = Linear
app-gradient-ease-in = Masuk
app-gradient-ease-out = Keluar
app-gradient-ease-in-out = Masuk-keluar
app-gradient-midpoint = Titik tengah
app-gradient-midpoint-hover = Tempat paduan mencapai setengah jalan, sebagai pecahan dari segmen ini. 0.50 berarti tepat di tengah.
# A button that puts the midpoint back in the middle of the segment.
app-gradient-centre = Tengahkan
app-gradient-centre-hover = Kembali ke 0.50
app-gradient-no-hue = ⚠ salah satu ujung segmen ini tidak memiliki rona
app-gradient-no-hue-hover = Salah satu ujungnya hitam, putih, atau abu-abu, sehingga sapuan dimulai dari merah dan menempuh seluruh roda warna. Itu sah dan kadang memang diinginkan — tetapi itulah sebabnya bagian tengah bisa berupa warna yang tidak dimiliki kedua ujungnya. Lajur di bawah kurva menunjukkan efeknya.
app-gradient-segment-start-hover = Warna di AWAL segmen ini
app-gradient-segment-end-hover = Warna di AKHIR segmen ini
app-gradient-bytes-hover = Merah, hijau, dan biru sebagai 0–255
app-gradient-floats-hover = Warna yang sama sebagai 0–1, yaitu bentuk yang disimpan perender
app-gradient-canvas-bezier-hover = Kurva paduan untuk segmen yang dipilih. Seret salah satu titik kendali — pada x DAN y — untuk mengubah bentuknya. Garis samar adalah 0 dan 1; kurva boleh melewati keduanya, dan yang dibatasi adalah warnanya, bukan kurvanya.
app-gradient-canvas-hover = Kurva paduan untuk segmen yang dipilih: seberapa cepat warna bergerak dari titik warna kiri ke titik warna kanan. Seret cincin untuk memindahkan titik tengah, atau ubah kurva ke Bézier untuk mendapatkan titik kendali yang bergerak ke dua arah.
# The selected stop's heading. $index: its number; $total: how many there are.
app-gradient-stop-heading = Titik warna { $index } dari { $total }
app-gradient-stop-color-hover = Warna pada titik warna ini — klik contoh warna untuk memilih, atau ketik nilai heks di sampingnya
app-gradient-stop-bytes-hover = Merah, hijau, dan biru sebagai 0–255 — byte yang digunakan untuk menulis warna ini
# The eyedropper button.
app-gradient-pick = Ambil
app-gradient-pick-hover = Pipet — ambil warna dari mana saja di layar. Arahkan penunjuk ke warna yang Anda inginkan lalu klik; Esc membatalkan.
# Written before the stop's position (0.000–1.000) in its number field: a short abbreviation of "position".
app-gradient-stop-pos = pos
app-gradient-nudge-hover = Geser sedikit titik warna ini (tahan Shift untuk langkah lebih halus)
app-gradient-remove = Hapus
app-gradient-remove-hover = Hapus titik warna ini (atau klik ⊖ di bawahnya)
app-gradient-add-stop = Tambah titik warna
app-gradient-add-stop-hover = Belah segmen terlebar (atau klik garis di atas gradien)
# A menu of the built-in palettes, to start from a copy of one.
app-gradient-copy-preset = Salin preset…
app-gradient-paste = Tempel…
app-gradient-paste-hover = Impor palet dari warna heks atau tripel RGB 0–255
app-gradient-save-ggr = Simpan .ggr…
app-gradient-save-ggr-hover = Tulis gradien ini ke berkas GIMP .ggr — dapat dibaca oleh GIMP, Krita, dan Inkscape, serta oleh penyunting ini
# A menu of the palette file formats (.map, .ugr, .ggr, .ase).
app-gradient-import = Impor
app-gradient-import-map-hover = Muat berkas palet .map Fractint / Kalles Fraktaler (baris R G B, satu per entri)
app-gradient-import-ugr-hover = Muat berkas gradien .ugr Ultra Fractal (sebuah .ugr biasanya berisi banyak gradien — Anda memilih salah satunya)
app-gradient-import-ggr-hover = Muat gradien GIMP .ggr, dengan titik tengah, kurva paduan, dan ruang warnanya tetap utuh
app-gradient-import-ase-hover = Muat daftar swatch Adobe (.ase). Swatch tidak memiliki posisi, jadi semuanya ditempatkan dengan jarak merata dan dibaurkan.
# The gradient's name, for the library of saved gradients.
app-gradient-name = Nama
# Shown greyed in the empty name field.
app-gradient-name-hint = gradien saya
# The save button, when a gradient of that name is already saved.
app-gradient-update = Perbarui
app-gradient-save = Simpan
app-gradient-save-hover = Simpan gradien ini ke pustaka Anda, dengan kurva dan ruang warnanya tetap utuh
app-gradient-save-needs-name-hover = Beri nama gradien terlebih dahulu
# A menu of the saved gradients. $count: how many there are.
app-gradient-saved-menu = Tersimpan ({ $count })
app-gradient-nothing-saved = Belum ada yang disimpan
app-gradient-delete-saved-hover = Hapus gradien tersimpan ini
# $count: the gradient's segments.
app-gradient-rich = Gradien kaya: { $count } segmen memuat titik tengah, kurva paduan, atau ruang warna — lebih dari yang dapat ditampung daftar titik warna biasa.
app-gradient-convert = Konversi ke titik warna yang dapat disunting
app-gradient-convert-hover = Ganti dengan daftar titik warna biasa. Ini MEMBUANG titik tengah, kurva paduan, dan sapuan rona yang tidak dapat ditampung daftar titik warna.
# Above the list of gradients in an Ultra Fractal file. $count: how many; $file: the file's name.
app-gradient-ugr-list = { $count } gradien di { $file }
app-gradient-close-list = Tutup daftar
app-gradient-seamless = Mulus (akhir sama dengan awal)
app-gradient-seamless-hover = Palet diputar bersiklus, sehingga posisi 1 dan posisi 0 bersebelahan di layar. Jika ini aktif, warna titik warna terakhir dibuat sama dengan yang pertama — tampilan cincin menunjukkan pertemuannya di bagian atas.
app-gradient-ends-differ = ⚠ ujung berbeda
app-gradient-ends-differ-hover = Warna di akhir gradien tidak sama dengan warna di awal, sehingga palet bersiklus menunjukkan tepi tajam di tempat palet itu membungkus.
app-gradient-hard-bands = Pita tegas (tanpa pembauran antarentri)
app-gradient-hard-bands-hover = Semantik Fractint / .map: setiap entri adalah warna datar. Jika dimatikan, entri-entri dibaurkan menjadi gradien yang halus.
app-gradient-paste-intro = Tempelkan warna heks (#ff8800) atau tripel 0–255 (255 136 0), dipisahkan dengan koma, spasi, atau baris baru:
# Applies the pasted colours.
app-gradient-apply = Terapkan
# Empties the paste box.
app-gradient-clear = Kosongkan
# $used: the colours kept; $total: the colours pasted; $limit: the most that are kept.
app-gradient-paste-imported-some = { $used } dari { $total } warna diimpor, diambil secara merata dari daftar Anda (batasnya { $limit }).
# $count: the colours pasted.
app-gradient-paste-imported = { $count } warna diimpor.
# $error: why the pasted text could not be read (in English).
app-gradient-paste-unreadable = Tidak dapat membacanya: { $error }
# A hint at the bottom of the editor. $count: the palette's entries.
app-gradient-bands-hint = { $count } pita · diambil sesuai urutan berkas; posisi diabaikan selama pita aktif.
# A hint at the bottom of the editor, in the ring view. $count: the gradient's stops.
app-gradient-ring-hint = { $count } titik warna · klik jalur luar untuk menambahkan satu di sana · ⊕ menambahkan di celah terlebar, ⊖ menghapus titik warna yang dipilih · seret penanda untuk memindahkannya · tanda di bagian atas adalah sambungan
# A hint at the bottom of the editor, in the bar view. $count: the gradient's stops.
app-gradient-bar-hint = { $count } titik warna · klik garis di atas gradien untuk menambahkan satu di sana · seret penanda untuk memindahkannya · ⊖ atau klik kanan untuk menghapus
app-gradient-cancel = Batal
app-gradient-cancel-hover = Buang semua perubahan sejak jendela ini dibuka, lalu tutup jendela
app-gradient-ok = OK
app-gradient-ok-hover = Simpan perubahan ini dan tutup penyunting

## Shared

# The button that closes the Help window and the "Faster deep zoom" window.
app-close = Tutup

## The "Faster deep zoom" window (Help menu): the optional accelerated build

app-accel-title = Zoom dalam lebih cepat
app-accel-active-heading = Anda menjalankan build terakselerasi
app-accel-active-body = Orbit referensi zoom dalam sedang dihitung dengan MPFR/GMP, yang 2,5–6,4x lebih cepat daripada build standar pada langkah itu - jeda sebelum tampilan dalam mulai terselesaikan.
# $status: which arithmetic library is in use and its version (in English).
app-accel-in-use = Aritmetika yang digunakan: { $status }
app-accel-fallback-heading = Build terakselerasi — menggunakan aritmetika bawaan
app-accel-fallback-body = Ini adalah build terakselerasi (MPFR), tetapi pustaka matematikanya tidak dapat ditemukan, sehingga build ini menggunakan aritmetika bawaan (astro-float). Gambarnya identik; hanya jeda sebelum tampilan dalam terselesaikan yang lebih lambat.
# Followed by the folder the program runs from, on its own line.
app-accel-fallback-dlls = Untuk menggunakan jalur yang lebih cepat, letakkan libmpfr-6.dll dan libgmp-10.dll (serta libgcc_s_seh-1.dll, libwinpthread-1.dll) di samping fractadyne.exe, lalu mulai ulang:
app-accel-warning-hidden = Peringatan saat memulai tentang hal ini disembunyikan.
app-accel-show-again = Tampilkan lagi
app-accel-offer-heading = Build opsional 2,5–6,4x lebih cepat pada kedalaman
app-accel-offer-body = Zoom dalam menghabiskan banyak waktunya untuk menghitung orbit referensi di CPU - jeda sebelum tampilan dalam mulai terselesaikan. Build opsional melakukannya dengan pustaka MPFR/GMP, yang 2,5 hingga 6,4 kali lebih cepat, dan makin dalam Anda pergi, makin besar keunggulannya. Semua hal lain sama.
app-accel-offer-identical = Gambarnya IDENTIK BYTE DEMI BYTE - matematika yang sama dihitung oleh pustaka yang lebih cepat, diperiksa pada setiap rumus fraktal dan seluruh korpus perbandingan zoom dalam.
app-accel-offer-shared = Pengaturan, sesi tersimpan, dan lokasi Anda digunakan bersama oleh keduanya, sehingga Anda dapat beralih dengan bebas dan tidak ada yang perlu diimpor.
# Shown on Windows.
app-accel-separate-windows = Build ini diunduh terpisah karena pustaka yang digunakannya tidak dapat dibangun dengan kompiler yang digunakan build Windows standar, dan pustaka itu memiliki lisensi yang berbeda (GNU LGPL v3) dari lisensi Fractadyne sendiri.
# Shown on other systems.
app-accel-separate = Build ini diunduh terpisah karena pustaka yang digunakannya memiliki lisensi yang berbeda (GNU LGPL v3) dari lisensi Fractadyne sendiri.
app-accel-download = Unduh untuk versi ini
app-accel-all-releases = Semua rilis
app-accel-all-releases-hover = Jika tautan langsung menghasilkan 404, versi ini belum memiliki build terakselerasi - pilih yang terdekat di sini.
# Shown on Windows.
app-accel-install-windows = Ekstrak, lalu jalankan fractadyne.exe dari folder itu, dengan berkas .dll tetap di sampingnya.
# Shown on Linux.
app-accel-install-linux = Ekstrak, lalu jalankan ./fractadyne dari folder itu. Build ini menggunakan GMP dan MPFR sistem Anda (libgmp10 / libmpfr6), yang hampir selalu sudah ada di setiap desktop - lihat README-ACCELERATED.txt dalam paket.
# Shown on other systems.
app-accel-install-other = Belum ada paket terakselerasi yang dibangun untuk platform ini; paket dapat dibangun dari kode sumber dengan --features fractadyne-core/rug.

## The Help window's frame (its sections are under help-)

app-help-title = Bantuan Fractadyne

## Checking for updates

app-toast-up-to-date = Anda sudah menggunakan versi terbaru.
# $error: why the check failed (in English).
app-toast-update-failed = Pemeriksaan pembaruan gagal: { $error }
# The window that offers a newer version.
app-update-title = Pembaruan tersedia
# $version: the new version (0.3.0-beta.20). $channel: the token "stable" or "beta", in English; a translation can choose its words by it ({ $channel -> [beta] … *[other] … }).
app-update-version =
    { $channel ->
        [beta] Fractadyne { $version } (beta) sudah tersedia
       *[other] Fractadyne { $version } (stabil) sudah tersedia
    }
# $current: the version running now. $track: the update track the user follows (its name in the language in use).
app-update-running = Anda menjalankan { $current }  ·  { $track }
app-update-download-intro = Unduh rilis terbaru dari GitHub:
app-update-download = Unduh dari GitHub
app-update-later = Ingatkan saya nanti

## The second graphics card (shown in its window, after "In use:" or "Could not start:")

# The second device is the same card as the window's. $name: the card's name.
app-worker-same-card = kartu yang sama, { $name }
# Why the second device could not start.
app-worker-thread-stopped = thread perangkat berhenti

## Export status (shown in the Export image window)

app-export-gpu-unavailable = GPU tidak tersedia
app-export-reference-aborted = Ekspor gagal: pembangunan referensi dibatalkan.

## The performance panel (View ▸ Performance panel)
## Its row labels sit in a monospace column about 14 characters wide: keep them short (a longer label pushes its value to the right).

app-perf-fps = FPS
app-perf-fps-hover = Bingkai yang ditampilkan per detik, dari interval antarbingkai yang dihaluskan. Selama tampilan sedang menuju stabil, baris ini melaporkan kemajuan penyempurnaan — tampilan yang sedang menuju stabil tidak berjalan pada laju yang tersirat oleh angka di sini.
# What the FPS row shows while a view is settling. $done and $total: tiles; $eta: the time left, as m:ss.
app-perf-refining-tiles = menyempurnakan { $done }/{ $total } (~{ $eta })
app-perf-refining-arming = menyempurnakan (bersiap)
# $percent: how much is done, with one decimal.
app-perf-refining-percent = menyempurnakan { $percent }%
app-perf-building-reference = membangun referensi
app-perf-idle = diam
app-perf-frame = bingkai
app-perf-frame-hover = Waktu jam dinding antarbingkai yang dihaluskan — bukan jumlah dari baris-baris di bawahnya. Waktu ini mencakup penantian untuk menampilkan, sehingga pada tampilan diam yang sudah stabil, nilainya mencerminkan irama penggambaran ulang, bukan pekerjaan apa pun.
app-perf-cpu = cpu
app-perf-cpu-hover = Waktu thread utama di dalam satu pembaruan: penyiapan parameter dan pengelolaan cache referensi. Inilah baris yang melonjak saat referensi dibangun ulang, dan lonjakan di sini adalah patah-patah yang Anda rasakan.
app-perf-gpu-idle = gpu/diam
app-perf-gpu-idle-hover = Waktu bingkai dikurangi waktu CPU thread utama. Itu adalah pekerjaan GPU saat tampilan sedang dirender, tetapi pada tampilan yang sudah stabil sebagian besar berupa penantian diam — namanya menyebut keduanya karena panel tidak dapat membedakannya hanya dari pengurangan ini.
app-perf-timing = pewaktuan
# How frame cost is measured, the timing row's value. TIMESTAMP_QUERY is a name.
app-perf-timing-starved = jam dinding (stempel waktu tak tiba)
app-perf-timing-gpu = stempel waktu GPU
app-perf-timing-no-query = jam dinding (tanpa TIMESTAMP_QUERY)
app-perf-timing-hover = Cara biaya bingkai diukur. Stempel waktu GPU adalah sumber yang presisi; cadangan jam dinding aktif ketika pewaktuan berhenti datang, dan anggaran yang ditakar dengan jam dinding berulang kali salah menakar bingkai dalam — anggap warna merah di sini sebagai “batas aman saat ini hanyalah perkiraan”.
app-perf-mode = mode
app-perf-mode-hover = Jalur aritmetika yang digunakan untuk merender bingkai terakhir: langsung (tanpa orbit referensi), perturbasi df32, atau floatexp - jalur rentang diperluas yang digunakan setelah sekitar 1e28x, dan satu-satunya jalur tempat BLA berjalan.
app-perf-eff-iter = iter efektif
app-perf-eff-iter-hover = Iterasi yang benar-benar digunakan bingkai terakhir, setelah penskalaan kedalaman dan batas apa pun yang diterapkan saat bergerak - bukan pengaturan dasar di Kualitas, yang hanya merupakan titik awal.
app-perf-precision = presisi
app-perf-precision-hover = Jumlah bit mantisa yang digunakan untuk menghitung orbit referensi. Jumlahnya bertambah seiring kedalaman; 64 bit adalah batas bawah untuk tampilan dangkal, bukan batas yang sedang Anda capai.
app-perf-orbit-len = panjang orbit
# $cap: the most orbit samples this GPU holds, already formatted, or the text of app-perf-uncapped.
app-perf-orbit-len-hover = Sampel orbit referensi. GPU ini menampung paling banyak { $cap } — mendekatinya berarti mendekati dinding kedalaman praktis: setelah melewatinya, tampilan langsung tidak dapat menyelesaikan lokasi yang lebih dalam yang memerlukan orbit lebih panjang.
app-perf-uncapped = (tanpa batas)
app-perf-sa-skip = lompatan SA
app-perf-sa-skip-hover = Iterasi yang dilompati aproksimasi deret sebelum pekerjaan per piksel dimulai. Lebih tinggi berarti lebih cepat dan tidak mengubah apa pun pada gambar; baris ini tidak muncul jika tidak ada yang dilompati.
app-perf-aa = aa
app-perf-aa-hover = Supersampling: setiap piksel dirender dari sejumlah sampel ini per sumbu, sehingga 2x memerlukan empat kali lipat pekerjaan.
app-perf-dual = ganda
app-perf-dual-hover = Apakah tampilan terbagi Mandelbrot/Julia sedang berjalan. Jika nilainya true, setiap biaya di panel ini mencakup KEDUA panel.
app-perf-zoom = zoom
app-perf-zoom-hover = Perbesaran relatif terhadap tampilan penuh. Mode aritmetika di atas berubah pada sekitar 1e4x dan lagi pada sekitar 1e28x.
app-perf-iterate = iterasi
# $target and $lethal: times in milliseconds, already formatted.
app-perf-iterate-hover = Waktu GPU terukur dari dispatch iterasi terakhir. Pengendali menargetkan { $target } ms; pembacaan yang terus-menerus mendekati { $lethal } ms adalah pita tempat kelas perangkat keras ini pernah kehilangan perangkat.
app-perf-steps = langkah/s
app-perf-steps-hover = Langkah iterasi per detik di GPU, dalam miliar. Tampilan yang didominasi interior berjalan lebih lambat per langkah daripada yang tersirat dari jumlahnya saja, jadi ini adalah pembacaan throughput, bukan skor.
app-perf-budget = anggaran
# After the budget's value while it is still converging; the hover below quotes it.
app-perf-budget-settling = (menuju stabil)
app-perf-budget-hover = Anggaran langkah per dispatch yang adaptif, pengaman pengendali terhadap bingkai yang cukup lama hingga driver mereset GPU. “menuju stabil” berarti anggaran masih konvergen dari pembacaan yang jarang dan belum menjadi batas atas yang dapat dipercaya.
app-perf-rss = rss
# $mb and $peak: memory in megabytes, already formatted.
app-perf-rss-value = { $mb } MB (puncak { $peak })
app-perf-rss-hover = Memori residen proses, dan nilai tertinggi yang dicapai pada sesi ini. Pembangunan referensi dalam pernah memuncak melewati 2 GB di lapangan tanpa ada hal lain di layar yang menunjukkannya; berwarna kuning setelah melewati tiga perempat RAM sistem.
app-perf-gpu-est = est. gpu
app-perf-gpu-est-hover = Perkiraan memori yang menetap di GPU: orbit referensi, pohon BLA, tekstur iterasi, ping-pong status potongan saat penyempurnaan berjalan, dan snapshot gerbang penyajian. Disusun dari ukuran alokasi yang diketahui; angka sebenarnya dari driver tidak dapat dibaca secara portabel.
app-perf-julia-re = julia c.re
app-perf-julia-re-hover = Bagian real dari parameter c yang sedang dirender panel Julia - titik di bawah kursor, kecuali jika dikunci.
app-perf-julia-im = julia c.im
app-perf-julia-im-hover = Bagian imajiner dari parameter c yang sedang dirender panel Julia.
app-perf-julia-c = julia c
# The julia c row's value when the parameter is held fixed.
app-perf-pinned = dikunci
app-perf-pinned-hover = Parameter ditahan pada titik tetap, sehingga menggerakkan kursor tidak lagi mengubah panel Julia.
app-perf-c-panel = c/panel
app-perf-c-panel-hover = Lebar ruang c yang dicakup seluruh panel Mandelbrot. Jika nilai ini jauh di bawah satu piksel Julia, mengarahkan kursor mengubah c tetapi Julia tampak tidak berubah — wajar pada zoom dalam, bukan macet.
app-perf-ref-recompute = hitung ref
app-perf-ref-recompute-hover = Durasi penghitungan ulang orbit referensi yang terakhir. Inilah jeda sebelum tampilan dalam mulai terselesaikan, dan langkah yang dipercepat oleh build terakselerasi opsional.
app-perf-recompute-rate = hitung/s
app-perf-recompute-rate-hover = Orbit referensi yang DIPASANG per detik - yang hasilnya benar-benar digunakan. Bandingkan dengan baris di bawahnya: selisih keduanya adalah pekerjaan yang dibuang.
app-perf-builds-rate = bangun ref/s
app-perf-builds-rate-hover = Pembangunan referensi yang DIMULAI per detik, termasuk yang hasilnya dibuang - slot lookahead yang tidak pernah dicapai penyelaman, atau perpanjangan yang ditolak. Semua itu memakan CPU penuh dan tidak muncul di tempat lain: sebuah putaran prefetch pernah menjalankan sekitar 400 pembangunan per detik selama enam menit sementara hitung/s menunjukkan 2. Angka satu digit adalah normal; selisih besar antara kedua baris ini berarti ada pekerjaan yang dibuang.
app-perf-recompute-total = total hitung
app-perf-recompute-total-hover = Penghitungan ulang referensi sejak aplikasi diluncurkan. Berguna sebagai laju selama interval yang diketahui; angka yang terus naik sementara tidak ada yang bergerak adalah sinyal yang patut ditelusuri.

## Help: the contents list (Help & reference…, F1)

# Each section's name in the contents list (about 165 px wide: keep them short), and its heading.
help-section-overview = Ikhtisar
help-section-navigation = Navigasi
help-section-expressions = Ekspresi koordinat
help-section-options = Pewarnaan & opsi
help-section-fractals = Fraktal
help-section-how-it-works = Cara kerjanya
help-section-command-line = Baris perintah
help-section-shortcuts = Pintasan
help-section-hardware = Perangkat keras disarankan
# Credits to the prior work Fractadyne builds on.
help-section-acknowledgments = Ucapan terima kasih
# The third-party license notices.
help-section-licenses = Lisensi
help-section-about = Tentang

## Help ▸ Overview

help-overview-intro = Fractadyne adalah penjelajah fraktal native yang dibuat untuk zoom yang sangat dalam dan untuk kecepatan. Program ini menggambar fraktal “waktu lolos” (escape-time) — gambar yang tercipta dengan mengulangi satu rumus sederhana di setiap piksel.
help-overview-what-is = Apa itu fraktal waktu lolos?
help-overview-what-is-text = Untuk setiap piksel, program menjalankan rumus seperti z → z² + c berulang-ulang, dimulai dari nol. Jika nilai yang berjalan tetap kecil selamanya, piksel itu termasuk dalam himpunan dan digambar gelap. Jika nilainya akhirnya tumbuh tanpa batas (“lolos”), piksel itu berada di luar himpunan, dan warnanya mencatat berapa langkah yang dibutuhkan. Batas yang detailnya tak terhingga antara “tetap” dan “lolos” itulah fraktalnya.
help-overview-can-do = Apa yang dapat Anda lakukan
help-overview-can-pan = Geser dan zoom praktis tanpa batas (posisi tetap tepat pada kedalaman berapa pun).
help-overview-can-families = Beralih di antara 25 keluarga fraktal bawaan (sepuluh, dan lima di antaranya pada pangkat lebih tinggi), rumus Anda sendiri, Life, dan sistem-L, serta lihat keluarga waktu lolos mana pun kecuali Newton sebagai himpunan Julia.
help-overview-can-recolor = Warnai ulang dengan gradien preset atau kustom dan beberapa metode pewarnaan.
help-overview-can-lighting = Tambahkan pencahayaan relief 3D dan kontur batas yang berpendar.
help-overview-can-minibrots = Tepatkan ke minibrot, markahi tempat, dan ekspor gambar beresolusi tinggi.
help-overview-can-autopilot = Biarkan autopilot zoom otomatis menyelam ke arah detail dengan sendirinya.
help-overview-can-share = Bagikan tempat mana pun sebagai potongan teks “.fdn” yang kecil atau sebagai berkas.
help-overview-can-tours = Jalankan tur berskrip dan tolok ukur perangkat keras.
help-overview-first-steps = Langkah pertama
# "Seahorse Valley" is a place in the Navigate menu: use its name as that menu shows it. A and F1 are keys.
help-overview-first-steps-text = Buka menu Navigasi dan pilih “Seahorse Valley”, lalu putar roda mouse untuk memperbesar. Seret untuk menggeser. Atau cukup tekan A dan biarkan autopilot menyelam untuk Anda. Tekan F1 kapan saja untuk kembali ke bantuan ini.

## Help ▸ Coordinate expressions
## The expressions, numbers and function names here are typed as shown in every language:
## keep them exactly as written. Menu items in quotation marks are the menus' own labels.

help-expr-intro = Di mana pun Fractadyne menerima koordinat, Anda dapat mengetikkan ekspresi sebagai ganti desimal: kolom Re / Im di Navigasi → “Pergi ke lokasi…” (dan setiap kolom mode polarnya), baris center_re / center_im pada lokasi .fdn, re / im pada skrip tur, dan --center di baris perintah. Ekspresi dievaluasi dengan digit sebanyak yang dibutuhkan zoom, dan dialog Pergi ke lokasi menyimpannya bersama tampilan — sehingga titik yang Anda masukkan sebagai 1/3, atau sebagai -0.5 + 0.25*cos(pi/4), tetap tepat pada titik itu sedalam apa pun Anda melakukan zoom, sedangkan desimal akan bergeser begitu digitnya habis.
help-expr-examples = Contoh
help-expr-ex-third = Sepertiga yang eksak. Makin banyak digit muncul saat Anda memperbesar.
help-expr-ex-neck = Leher Seahorse Valley — akar bulatan periode 2.
help-expr-ex-circle = Sebuah titik pada lingkaran berjari-jari 1/4 di sekitar -0.5 (mode polar menyusun ini untuk Anda).
help-expr-ex-bulb = Re dari akar bulatan periode 3; Im-nya adalah sin(2*pi/3)/2 - sin(4*pi/3)/4. Navigasi → “Titik eksak (ekspresi)” sudah menyediakan titik-titik ini.
help-expr-ex-complex = Nilai kompleks utuh yang diketik di kolom Re mengisi Re dan Im sekaligus.
help-expr-ex-combine = Fungsi dan konstanta dapat dikombinasikan dengan bebas.
help-expr-ex-powers = Pangkat: 2^-20 adalah 1/1048576. -2^2 adalah -4 — tanda minus diterapkan setelah pangkat.
help-expr-ex-root = -2, akar pangkat tiga yang real. Akar genap dari bilangan negatif ditolak; sqrt(-1) menghasilkan i.
help-expr-ex-scientific = Notasi ilmiah juga berlaku di dalam ekspresi.
help-expr-numbers = Bilangan dan operator
# The decimal point in an expression is '.' in every language: keep that rule as it is.
help-expr-numbers-decimals = Desimal dengan eksponen opsional: 0.25, -1.5e-30, .5. Hanya digit — tanpa spasi atau koma pemisah ribuan di dalam bilangan, dan titik desimal selalu '.'.
help-expr-numbers-imaginary = Tambahkan akhiran i untuk bilangan imajiner (0.25i), atau gunakan i saja: 3 + 4*i.
# Starts with the operators themselves, two spaces apart.
help-expr-numbers-operators = +  -  *  /  dan ^ (pangkat; dievaluasi dari kanan ke kiri, jadi 2^3^2 = 2^9). Tanda kurung mengelompokkan. Tidak ada perkalian implisit: tulis 2*pi, bukan 2pi.
help-expr-numbers-case = Nama tidak membedakan huruf besar-kecil: PI, Pi, dan pi adalah konstanta yang sama.
help-expr-constants = Konstanta
help-expr-functions = Fungsi
help-expr-functions-angles = Sudut dinyatakan dalam radian. Untuk derajat, kalikan dengan pi/180 — cos(45*pi/180) — atau alihkan dialog Pergi ke lokasi ke mode polarnya dan pilih satuan derajat.
help-expr-refused = Apa yang ditolak, dan mengapa
help-expr-refused-fractional = Bilangan negatif berpangkat pecahan, seperti (-2)^(1/2): hasilnya kompleks dan ada beberapa akar yang dapat dipilih. Gunakan root(x, n) atau cbrt(x) untuk akar real, atau sqrt(x) untuk akar kompleks utama.
help-expr-refused-complex = Argumen kompleks untuk fungsi bernilai real (sin(i), ln(1+i)). Hanya sqrt dan abs yang menerima argumen kompleks; operator aritmetika dan pangkat bilangan bulat berlaku pada nilai kompleks.
help-expr-refused-domain = Galat domain: ln atau log dari bilangan yang tidak positif, asin atau acos di luar -1…1, 0 berpangkat negatif, pembagian dengan nol.
help-expr-refused-bounds = Argumen sin, cos, tan, exp, atau ^ di luar ±2^32, dan penyarangan lebih dalam dari 32 tingkat: koordinat yang ditempel adalah masukan yang tidak tepercaya, dan hal-hal itu akan memakan waktu tanpa batas.
help-expr-refused-grammar = Tidak ada variabel, fungsi buatan pengguna, perbandingan, atau kondisional. Tata bahasanya sengaja dibuat kecil agar lokasi yang disimpan memiliki arti yang sama di setiap versi.
help-expr-no-parse = Jika tidak dapat diurai
help-expr-no-parse-text = Pesannya menyebutkan kolomnya, apa yang ditemukan dan pada karakter ke berapa, serta — untuk kekeliruan yang umum — apa yang harus diketik sebagai gantinya: tanda minus tipografis (−) yang ditempel dari dokumen, × atau ÷, koma desimal, * yang hilang antara bilangan dan nama, tanda kurung yang tidak ditutup, atau fungsi yang ditulis tanpa tanda kurungnya. Tidak ada yang dibaca setengah-setengah: jika ada bagian teks yang tidak dapat dipahami, seluruh koordinat ditolak, bukan dipendekkan diam-diam.

## Help ▸ Navigation
## Menu paths (Navigate → "Go to location…") and quoted items name the interface's own
## menus and controls: use their translations. Key names (Ctrl, Shift, Space, Esc) stay.

# A heading: the mouse controls (also in Shortcuts).
help-mouse = Mouse
# The left column of a controls table: a gesture or key. Keep key names (Shift, Space, Ctrl) as they are.
help-key-left-drag = Seret kiri
help-nav-left-drag = Geser tampilan.
help-key-mouse-wheel = Roda mouse
help-nav-wheel = Perbesar/perkecil ke arah kursor; putar lebih cepat untuk zoom lebih jauh per takik. Dengan “Tepatkan zoom ke fitur” aktif, memperbesar di dekat minibrot akan menarik zoom ke minibrot itu.
help-key-shift-drag = Shift+seret
help-nav-shift-drag = Zoom kotak — seret sebuah persegi panjang, dan zoom akan membuatnya memenuhi tampilan.
# $icon: the click-to-zoom toolbar button's symbol.
help-key-click-zoom = Klik untuk zoom ({ $icon })
help-nav-click-zoom = Aktifkan tombol target di bilah alat (tampilan tunggal): klik kiri kemudian menyelam ke titik itu dengan faktor yang diatur, klik kanan mundur. Seret tetap menggeser; pilih faktornya (2–100×) di bawah Navigasi pada panel kanan. Backspace mengurungkan satu klik.
help-nav-continuous = Zoom berkelanjutan & tampilan awal
help-key-hold-space = Tahan Space
help-nav-hold-space = Perbesar dengan halus, berpusat pada kursor.
help-key-hold-shift-space = Tahan Shift+Space
help-nav-hold-shift-space = Perkecil dengan halus.
help-nav-continuous-text = Tombol tampilan awal menganimasikan penerbangan kembali yang lembut ke tampilan penuh. “Kecepatan zoom” di panel kanan mengatur laju zoom berkelanjutan.
help-nav-autopilot = Autopilot
help-nav-autopilot-text = Tekan A (atau Alat → “Zoom otomatis (autopilot)”) untuk penyelaman tanpa campur tangan: autopilot memilih sebuah titik di tepi fraktal tempat detailnya paling padat, dengan mengutamakan bagian tengah tampilan, lalu meluncur ke dalamnya sambil perlahan membawanya ke tengah layar. Autopilot mempertahankan target itu selama masih menarik dan berbelok dengan halus ke struktur yang lebih kaya bila muncul. Masukan navigasi apa pun — atau Esc — menghentikannya. “Prioritas zoom otomatis” di panel kanan (atau di menu Alat) memilih apa yang diutamakan: Kecepatan melakukan zoom pada laju yang diatur dan menyegarkan layar begitu bingkai yang memiliki detail siap; Kualitas hanya menampilkan bingkai beresolusi penuh yang terselesaikan sepenuhnya dan memperlambat zoom bila perlu agar gambar tidak pernah diperbesar terlalu jauh sebelum bingkai berikutnya tiba. Bagaimanapun juga, layar tidak pernah menampilkan bingkai kosong atau satu warna: penyegaran yang belum menyelesaikan detail apa pun tetap tersembunyi di balik bingkai sebelumnya. “Target zoom otomatis” memilih apa yang dituju: Detail (tepi paling kaya dalam tampilan, dipilih ulang sambil berjalan) atau Titik Misiurewicz — autopilot menemukan titik Misiurewicz terdekat, menghitungnya hingga kedalaman batas penyelaman sambil diam di tempat, lalu menyelam tepat ke dalamnya; struktur di sekitar titik semacam itu berulang setiap beberapa oktaf selamanya, sehingga penyelaman itu tidak pernah menemui jalan buntu (hanya himpunan Mandelbrot).
help-nav-history = Riwayat & gerakan presisi
help-nav-undo = Urungkan tampilan sebelumnya.
help-nav-redo = Ulangi.
help-nav-goto = Navigasi → “Pergi ke lokasi…” memungkinkan Anda membaca, mengetik, menempel, atau menyalin pusat dan zoom yang persis (presisi penuh, kedalaman berapa pun). Kolom pusat menerima lebih dari sekadar bilangan biasa: pecahan seperti “1/3” dan ekspresi yang dibangun dari pi, tau, e, sqrt, dan sin/cos/tan — jadi “-0.5 + 0.25*cos(pi/4)” adalah koordinat yang sah. Ekspresi disimpan bersama tampilan dan dievaluasi ulang dengan presisi yang dibutuhkan setiap kedalaman, sehingga titik yang dimasukkan dengan cara itu tetap persis sejauh apa pun Anda memperbesar (desimal yang dibulatkan lama-kelamaan tidak akan begitu). Mode polar (pergeseran + r ∠ θ, dalam derajat / radian / putaran) menyusun salah satu ekspresi ini untuk Anda. Daftar lengkap fungsi dan konstanta, beserta contohnya, ada di bagian “Ekspresi koordinat” pada bantuan ini (tombol ? di dialog membukanya). Navigasi → Markah menyimpan dan memanggil kembali lokasi.
help-nav-exact-points = Navigasi → “Titik eksak (ekspresi)” melompat ke titik-titik yang didefinisikan secara eksak, bukan dengan desimal yang dibulatkan — akar bulatan-bulatan kardioid utama, tempat setiap tunas periode q menempel, ditulis sebagai ekspresi cos/sin. Karena ekspresinya ikut bersama tampilan, Anda dapat memperbesar salah satunya tanpa pernah bergeser dari titik itu.
help-nav-share = Navigasi → “Bagikan lokasi…” merekam seluruh tampilan — fraktal, pusat dengan presisi penuh, zoom, dan pewarnaan — sebagai potongan “.fdn” yang ringkas yang dapat Anda Salin, tempel kembali lalu Terapkan, atau Simpan/Muat sebagai berkas, untuk mereproduksi suatu tempat secara persis (atau mengirimkannya kepada seseorang).
help-nav-open = Berkas → “Buka tampilan atau lokasi…” membuka salah satu dari ini secara langsung: PNG/EXR yang diekspor (tampilan yang tersemat di dalamnya dipulihkan), berkas lokasi “.fdn”, atau “.kfr” Kalles Fraktaler.
help-nav-finding = Menemukan detail
help-nav-minibrots = Minibrot adalah salinan kecil dari seluruh himpunan yang tersembunyi di sepanjang batas. Letakkan salah satunya kira-kira di tengah lalu tekan M (atau Alat → “Temukan minibrot + zoom ke sana”) untuk menepatkan dengan metode Newton tepat ke pusatnya, membaca periodenya, dan langsung zoom ke skalanya sendiri — sering kali banyak orde besaran dalam satu langkah. Jika Anda sudah lebih dalam daripada minibrot itu, hanya pusatnya yang berpindah.
help-nav-minimap = Tampilan → “Ikhtisar peta mini” menampilkan peta kecil seluruh himpunan dengan penanda “Anda di sini” dan kedalaman zoom saat ini; klik peta itu untuk melompat ke suatu wilayah.

## Help ▸ Coloring & options
## Names in quotation marks are the controls' own labels in the right-hand panel.

help-options-intro = Semua ini ada di panel kanan (dan tetap tersimpan di antara sesi).
help-options-palette = Palet
help-options-palette-text = Palet memilih gradien preset, gradien Kustom Anda, atau mode dua warna. “Sunting gradien…” membuka penyunting tempat setiap titik warna memiliki warna dan posisi (0–1); tambahkan titik warna, salin preset sebagai titik awal, atau impor berkas palet.
help-options-paste = “Tempel…” di penyunting gradien mengimpor palet yang Anda temukan di tempat lain. Fitur ini menerima warna heks dengan atau tanpa # di depan (#ff8800, ff8800, atau bentuk pendek #f80) dan tripel RGB 0–255 biasa seperti “255 136 0” — format yang digunakan berkas .map Fractint dan Kalles Fraktaler — dipisahkan dengan koma, spasi, atau baris baru. Hingga 256 warna disimpan; daftar yang lebih panjang diambil sampelnya secara merata hingga 256, dengan mempertahankan yang pertama dan terakhir agar bentuk gradien tetap terjaga.
help-options-import-map = “Impor ▸ .map…” di sebelahnya memuat berkas palet .map Fractint atau Kalles Fraktaler. Berkas semacam itu adalah tabel pencarian yang diindeks berdasarkan jumlah iterasi tanpa pembauran antarentri, dan undakan warna tegas yang dihasilkannya itulah yang membentuk tampilan klasik Fractint — maka begitulah berkas itu diimpor, dengan kotak centang “Pita tegas” untuk membaurkan warna-warna yang sama menjadi gradien yang halus. Berkas Fractint sendiri menyimpan nilai warna 6-bit yang dikalikan empat, sehingga putihnya bernilai 252, bukan 255; Fractadyne memberitahukan hal itu saat mengimpornya dan membiarkan nilainya persis seperti yang tertulis, karena gambar yang disimpan Fractint sendiri juga membawa nilai 252 yang sama.
help-options-duotone = Dua nada memetakan nilai ke gradasi dua warna yang halus (Bayangan → Sorotan). Biner (himpunan) adalah tampilan dua warna datar — satu warna padat untuk titik di dalam himpunan dan warna lain untuk di luarnya, tanpa gradien — cara paling jelas untuk melihat bentuk himpunan.
help-options-cycle = Siklus mengatur berapa kali gradien berulang di sepanjang rentang iterasi (pita lebih rapat atau lebih renggang). Pergeseran memutar seluruh gradien.
help-options-normalize = “Normalkan warna dalam” menyesuaikan palet dengan rentang nilai lolos yang benar-benar ada di tampilan. Tampilan dalam mencakup ratusan ribu iterasi, dan tanpa ini siklus yang tetap akan membungkus gradien ribuan kali di antara piksel yang bersebelahan, mengubah detail yang benar menjadi bintik-bintik.
help-options-log-scale = “Skala warna log” menyebarkan palet berdasarkan logaritma nilai lolos, bukan secara linear. Jumlah iterasi lolos menumpuk ke arah ujung atas makin dalam Anda pergi, sehingga penyesuaian linear menghabiskan sebagian besar gradien pada lapisan tipis yang menempel di batas dan membiarkan sisanya hampir datar; skala log membukanya, dan menjaga warna lebih stabil sepanjang video zoom seiring rentangnya bertambah.
help-options-animate = Animasi memutar warna seiring waktu — Maju, Mundur, atau Bolak-balik menggeser offset; Acak terus-menerus menyintesis gradien harmonis yang berubah bentuk dengan halus. Kecepatan mengatur jumlah siklus (atau perubahan bentuk) per detik; “Acak gradien” menghasilkan gradien baru dalam mode Acak.
help-options-method = Metode pewarnaan — bagaimana data menjadi warna
# The coloring methods, by the names the Coloring panel gives them, each with what it does.
help-method-smooth = Iterasi halus
help-method-smooth-text = Pita kontinu klasik berdasarkan waktu lolos.
help-method-stripe = Rata-rata garis
help-method-stripe-text = Pita mengalir dari sudut orbit (penggeser Kepadatan garis).
help-method-triangle = Ketaksamaan segitiga
help-method-triangle-text = Tekstur halus dari tempat setiap langkah mendarat di antara batas-batas.
help-method-trap = Perangkap orbit
help-method-trap-text = Jarak orbit ke suatu bentuk (titik/silang/lingkaran); juga mewarnai interior.
help-method-distance = Estimasi jarak
help-method-distance-text = Memberi bayangan berdasarkan kedekatan dengan batas.
help-method-decomposition = Dekomposisi
help-method-decomposition-text = Sel-sel dari sudut lolos terakhir.
help-options-relief = Pencahayaan relief 3D
help-options-relief-text = Memberi bayangan pada permukaan berdasarkan kemiringan batas (turunannya) untuk tampilan timbul yang tersinari. Sudut cahaya mengatur arahnya; Relief mengatur kekuatannya (lebih rendah = lebih tajam); “Putar cahaya” menganimasikannya. Hanya keluarga holomorfik (Mandelbrot / Multibrot).
help-options-glow = Pendaran jarak
help-options-glow-text = Pita kontur terang yang memadat menjadi filamen berpendar di dekat batas. Pendaran adalah takaran paduannya, Lebar pita adalah jaraknya, dan “Animasikan pendaran” mengalirkannya.
help-options-quality = Kualitas & iterasi
help-options-quality-text = Iterasi adalah jumlah langkah maksimum sebelum sebuah piksel dianggap berada di dalam himpunan; “Skalakan otomatis” menaikkannya secara otomatis saat Anda melakukan zoom (detail yang lebih dalam membutuhkan lebih banyak). Anti-alias melakukan supersampling pada gambar diam (2×–8×) begitu tampilan stabil, meredam “debu” halus di bagian luar.
help-options-other = Lainnya
help-options-other-text = Kecepatan zoom mengatur laju zoom berkelanjutan; batas FPS membatasi laju bingkai; “Keduanya, tertaut” menampilkan himpunan Mandelbrot dan himpunan Julia-nya berdampingan (kursor mengatur parameter Julia secara langsung).

## Help ▸ Fractals
## Fractal family names (Mandelbrot, Multibrot, Tricorn, Burning Ship, Celtic, Buffalo,
## Phoenix, Newton) are names: keep them as written, as the formulas. Formula syntax
## (z = z^3 - p1*z + c, sqr, |z|, if / elseif / endif, p1-p5, maxit) is typed as shown.

help-fractals-intro = Setiap keluarga mengiterasi sebuah rumus dengan z dimulai dari 0 dan c ditentukan oleh piksel (waktu lolos), kecuali disebutkan lain. z = x + iy adalah bilangan kompleks.
help-fractals-mandelbrot = z → z² + c. Yang asli. Himpunan ini adalah semua c yang orbitnya tetap terbatas; himpunan ini terhubung, dan batasnya begitu berkerut sehingga berdimensi Hausdorff 2.
help-fractals-multibrot = z → z^d + c untuk pangkat d = 3, 4, 5. Pangkat yang lebih tinggi menambah cuping: himpunannya memiliki simetri putar lipat (d−1) (Multibrot 3 lipat-2, 4 lipat-3, 5 lipat-4).
help-fractals-tricorn = z → conj(z)² + c, dengan conj(x + iy) = x − iy. Konjugasi membuatnya anti-holomorfik, menghasilkan simetri lipat-3 dan “cakar” melengkung yang khas.
help-fractals-burning-ship = z → (|x| + i|y|)² + c — ambil nilai mutlak sebelum dikuadratkan (real = x²−y²+cx, imag = 2|xy|+cy). Non-analitik; zoom dalam memperlihatkan struktur mirip kapal (secara tradisional dilihat terbalik).
help-fractals-celtic = Seperti Mandelbrot tetapi dengan nilai mutlak bagian real: real = |x²−y²| + cx, imag = 2xy + cy. Menghasilkan motif berbentuk hati dan perisai.
help-fractals-buffalo = Nilai mutlak kedua bagian z²: real = |x²−y²| + cx, imag = |2xy| + cy — persilangan antara Celtic dan Burning Ship.
# A heading: the families at powers above 2 (the Fractal menu's "Higher powers").
help-fractals-powers = Pangkat lebih tinggi
# ("Burning Ship  3 4 5") is how the Fractal menu shows such a row: keep it as it is.
help-fractals-powers-text = Multibrot 6, 7, dan 8 (z^d + c), serta Burning Ship, Tricorn, Celtic, dan Buffalo pada pangkat 3, 4, dan 5: lipatan yang sama pada pangkat yang lebih tinggi. Burning Ship d mengambil nilai mutlak bagian-bagian z lalu memangkatkannya; Tricorn d mengonjugasikan terlebih dahulu; Celtic d dan Buffalo d melipat bagian real (dan, untuk Buffalo, bagian imajiner) dari z^d. Menu Fraktal dan daftar tarik-turun mencantumkannya satu keluarga per baris (“Burning Ship  3 4 5”). Masing-masing mendukung zoom dalam seperti saudaranya yang berpangkat 2, dan Multibrot 6, 7, dan 8 juga mendukung aproksimasi deret dan pencari minibrot (M), seperti Multibrot 3–5. Multibrot 8 lolos pada radius 128, bukan 256, karena pada radius itu langkah berikutnya akan keluar dari jangkauan presisi tunggal.
help-fractals-phoenix = z → z² + c + p·z(n-1), dengan z(n-1) adalah iterasi sebelumnya dan p adalah konstanta (p = -0.5). Suku memori ini menghasilkan filamen mirip nyala api.
help-fractals-newton = Metode Newton untuk akar-akar z³ − 1 = 0: z → z − (z³−1)/(3z²). Alih-alih waktu lolos, piksel diwarnai berdasarkan akar pangkat tiga dari satu mana yang dituju iterasinya (cekungan tarikan) dan seberapa cepat; batas-batas cekungan yang kusut itulah fraktalnya.
help-fractals-julia = Mode Julia
help-fractals-julia-text = Untuk setiap keluarga kecuali Newton, Anda dapat beralih ke himpunan Julia: alih-alih memulai z dari 0 dengan c = piksel, Anda menetapkan c (sebuah parameter) dan membiarkan z dimulai dari piksel. Himpunan Julia adalah batas antara titik awal yang tetap terbatas dan yang lolos, untuk c yang tetap itu. Dalam “Keduanya, tertaut”, menggerakkan kursor di atas panel Mandelbrot mengatur c secara langsung.
help-fractals-deep = Dukungan zoom dalam
help-fractals-deep-text = Mandelbrot, Multibrot 3/4/5, dan Tricorn mendukung zoom dalam perturbasi pada kedalaman ekstrem (floatexp). Burning Ship, Celtic, dan Buffalo non-analitik (mengambil nilai mutlak), sehingga menggunakan perturbasi yang memperhatikan tanda; kini perturbasi ini juga berjalan pada rentang floatexp, dengan zoom dalam jauh melewati batas df32 lama ~1e28× (bintik-bintik langka di dekat lipatan abs masih menunggu koreksi glitch multireferensi). Phoenix menggunakan perturbasi dua suku dan juga mendukung zoom dalam pada rentang floatexp (tanpa SA/BLA, jadi lebih berat daripada Mandelbrot). Pangkat yang lebih tinggi mendukung zoom dalam dengan cara yang sama, perturbasi setiap pangkat mengikuti rantai pengkuadratannya sendiri; Multibrot 3–8 juga mendukung aproksimasi deret dan BLA, seperti Mandelbrot, dan Tricorn, Burning Ship, Celtic, dan Buffalo pada setiap pangkat mendukung BLA (tanpa deret), yang melompati interior tampilan dalam alih-alih mengiterasinya langkah demi langkah. Hanya Newton yang menggunakan jalur langsung, tajam hingga ~1e6×; rumus kustom mendukung zoom dalam kecuali bagian Rumus kustom di bawah menyatakan lain.
help-fractals-custom = Rumus kustom
help-fractals-custom-text = Fraktal ▸ Rumus kustom… menerima langkah sebuah rumus dalam ekspresi gaya Fractint: z = z^3 - p1*z + c, atau beberapa pernyataan (t = sqr(z), lalu z = t*t + c). z adalah iterasi, c adalah piksel (konstanta Julia dalam mode Julia), p1-p5 adalah parameter; |z| adalah modulus kuadrat, seperti di Fractint, dan fungsi-fungsi yang biasa tersedia (sqr, abs, conj, real, imag, cabs, flip, recip, ident, exp, log, sqrt, sin, cos, tan, cotan, sinh, cosh, tanh, cotanh, asin, acos, atan, asinh, acosh, atanh, cosxx, serta floor, ceil, trunc, dan round, yang membulatkan setiap bagian). log adalah logaritma natural, seperti di Fractint dan dalam analisis kompleks: yang oleh banyak buku teks ditulis ln (tidak ada logaritma basis 10). Rumus yang ditulis dengan ln, cot, coth, Re, atau Im tidak dapat dibaca; pemeriksaan sintaksis menyebutkan fungsi yang harus digunakan, sebuah tombol menulis ulang rumusnya, dan mengetik ln akan menawarkan log. Setiap langkah mendukung zoom dalam dengan perturbasi, dengan aritmetika rentang diperluas yang sama setelah 1e28x seperti fraktal bawaan, kecuali langkah yang memiliki pangkat dengan eksponen yang bervariasi terhadap z atau c (seperti z^c): langkah itu dirender pada jalur langsung, tajam hingga satu langkah presisi tunggal dari c mencakup satu piksel, sekitar 1e4x hingga 1e5x, dan lebih dangkal jika jauh dari titik asal. Dialog rumus menyebutkan mana yang berlaku. log, sqrt, dan pangkat bukan bilangan bulat mengambil nilai utamanya, yang melompat melintasi sumbu real negatif. Perturbasi mengikuti lompatan itu dengan tepat, tetapi piksel yang orbitnya menyeberang di tempat orbit referensi tidak menyeberang akan dilanjutkan dengan presisi kira-kira tunggal sejak saat itu, sehingga tampilan dalam yang orbitnya mengangkangi potongan akan menunjukkan derau di sana (terukur: bersih pada 1e5x di sumbu real untuk sqrt(z^4 + c), berderau untuk rumus log bahkan pada 1e4x). Setiap langkah diiterasi penuh (tanpa aproksimasi deret), sehingga tampilan dalam dengan jumlah iterasi tinggi dirender lebih lambat daripada fraktal bawaan. Di dekat batasnya, rumus yang memuat fungsi mengorbit secara kacau, dan orbit panjang di sana hanya diikuti secara longgar dalam presisi tunggal: pada 2.000 iterasi, sepertiga piksel sin(z) + c berbeda dari orbit eksaknya, pada 60 hampir tidak ada. Tampilan, sesi, dan gambar yang diekspor menyimpan teks rumusnya.
help-fractals-custom-sections = Bagian-bagian Fractint juga berfungsi. Pernyataan sebelum ':' adalah bagian init, dijalankan sekali per piksel; perbandingan sebagai pernyataan terakhir adalah uji bailout rumus itu sendiri, yang terus mengiterasi selama perbandingan itu berlaku (|z| <= 100 && |z - 1| > 0.000001 menghentikan orbit yang lolos atau menetap di 1); if (...), elseif (...), else, dan endif memilih pernyataan; dan variabel yang dibaca sebelum ditetapkan dalam suatu langkah memperoleh nilainya dari langkah sebelumnya (dari bagian init, atau 0). Perbandingan membandingkan bagian real dan menghasilkan 1 atau 0, && dan || menggabungkannya, dan penugasan juga merupakan nilai (a = b = pixel). pixel, p1-p5, pi, dan e boleh ditugasi nilai, dan mempertahankan nilainya sendiri sampai ditugasi; maxit adalah jumlah iterasi dan ismand bernilai 1 (bentuk Mandelbrot dari rumus itu). Rumus yang memuat salah satu dari ini, atau memuat pembulatan, dirender pada jalur langsung, tanpa zoom dalam. Pustaka rumus > Impor juga membaca berkas .frm Fractint. Setiap entri yang dapat dibaca menjadi sebuah rumus, dan barisnya menyebutkan apa yang diubah oleh pembacaan itu: fn1-fn4 mengambil nilai bawaan Fractint (sin, sqr, sinh, cosh); variabel c milik Fractint diganti namanya menjadi c_, karena di sini c adalah piksel; nama yang tidak ditetapkan oleh apa pun bernilai 0, seperti di Fractint; spasi di dalam nama dibuang (end if menjadi endif), seperti yang dilakukan Fractint. Pesan impor menghitung entri yang tidak dapat dibaca dan memberikan alasan yang paling umum: variabel layar dan tampilan (whitesq, scrnpix, scrnmax, center, magxmag, rotskew), lastsqr, dan bilangan acak tidak didukung. Rumus-rumus klasik Fractint dalam Koleksi (Magnet, Nova, Spider, Manowar, dan lainnya) ditulis dengan bagian-bagian ini.
help-fractals-custom-editor = Di kolom rumus, tanda kurung diberi warna per tingkat sarang; pasangan di sebelah kursor disorot, dan tanda kurung tanpa pasangan ditampilkan merah (tanda kurung di dalam komentar tidak dihitung, karena pengurai mengabaikannya). Nama dilengkapi saat Anda mengetik: setelah dua huruf, sebuah daftar menawarkan fungsi, nama, dan variabel Anda sendiri yang diawali huruf-huruf itu. Atas dan Bawah memilih, Tab atau Enter mengambil satu (fungsi disertai tanda kurung bukanya), Esc menutup daftar, dan Enter setelah nama yang diketik lengkap tetap memulai baris baru.
help-fractals-custom-textbook = Buku teks (sakelar di atas rumus) menampilkan rumus dengan tata letak seperti di buku teks, dan menyuntingnya langsung di tempat. Ketik seperti pada teks: / membuat pecahan dari suku sebelumnya, ^ membuat eksponen (+, -, atau = yang diketik di akhir eksponen akan keluar darinya), ( membuat tanda kurung, atau pemanggilan setelah nama fungsi, | membuat garis tegak, dan koma di dalam tanda kurung membuat konstanta kompleks. Tombol panah bergerak di sepanjang rumus, Atas dan Bawah berpindah antara pembilang dan penyebutnya, Tab menuju kotak kosong berikutnya; Backspace di awal pecahan atau sepasang tanda kurung memilihnya, dan tekanan kedua menghapusnya sambil mempertahankan isinya. Salin, tempel, dan urungkan berfungsi seperti di kolom teks, nama dilengkapi seperti di sana, dan papan tombol mengetik ke dalam rumus yang tertata (□² mengkuadratkan apa yang ada sebelum tanda sisip, tombol fungsi membungkus pilihan). Setiap suntingan ditulis kembali ke teks, sehingga kedua tampilan selalu memuat rumus yang sama; baris yang belum Anda sunting mempertahankan spasi dan komentarnya. Notasi tidak pernah mengubah apa yang dihitung: |z| adalah modulus kuadrat dan ditampilkan sebagai |z|², cabs(z) ditampilkan sebagai |z|, dan exp(z) tetap exp(z). Baris yang tidak dapat dibaca ditampilkan merah; klik untuk menyuntingnya sebagai teks. LaTeX menyalin rumus sebagai sumber LaTeX, sesuai tata letaknya, dan selama Buku teks aktif, pustaka rumus juga menampilkan rumus-rumusnya dalam bentuk tertata.
help-fractals-custom-numbers = Bilangan dalam rumus (0.1, 1/3, pi, parameter) dibaca sebagai nilai presisi ganda, sekitar 16 digit, dan orbit referensi dalam beriterasi dengan nilai-nilai persis itu. Karena itu, dua rumus yang sama di atas kertas dapat berbeda jauh di dalam: 0.1*z mengalikan dengan nilai double yang terdekat dengan 0.1, sedangkan z/10 membagi dengan 10 yang eksak, dan kedua gambar itu berpisah setelah sekitar 1e15x (terukur: 5% piksel pada 1e15x, sepertiga pada 1e17x, semuanya pada 1e20x). Masing-masing gambar benar untuk rumusnya sendiri; tuliskan rumus yang memang ingin Anda bagikan.
help-fractals-custom-library = Simpan ke pustaka di dialog menyimpan rumus beserta parameternya dengan sebuah nama (menyimpan dengan nama yang sudah ada di pustaka akan memperbarui entri itu). Fraktal ▸ Pustaka rumus… mencantumkannya untuk diterapkan, disunting, diekspor, atau dihapus; Impor menambahkan rumus-rumus dalam berkas rumus tanpa mengganti satu pun: rumus yang sudah ada di pustaka dilewati, dan rumus baru yang namanya sudah terpakai menjadi “nama (2)”. Pustaka itu adalah formulas.toml di folder yang disebutkan di bawah Berkas & data (Tentang), dan berkas itu sendiri adalah berkas rumus. Bingkai kunci tur membawa rumus sebagai formula = "..." dengan formula_params = [[re, im], ...], dan Alat > Tur dari tampilan saat ini menuliskannya untuk tampilan kustom.
help-fractals-custom-collection = Koleksi berisi 45 rumus disertakan bersama Fractadyne, ditulis khusus untuknya: pangkat yang lebih tinggi dan varian terlipat dari keluarga Burning Ship dan Celtic, paduan dan hibrida dua langkah, himpunan Mandelbrot yang dilihat melalui pemetaan lain (terbalik, log-polar, melalui sinus), rumus transendental dan rasional, serta himpunan Julia. Rumus-rumus itu adalah Contoh di dialog dan tab Koleksi di pustaka rumus, tempat Salin ke milik saya menambahkan satu rumus ke rumus-rumus Anda. Masing-masing memiliki tampilan pembuka, dan Terapkan membawa Anda ke sana: rumus yang dibuka pada tampilan awal sering kali bukan tempat gambarnya berada. Rumus yang Anda simpan saat sedang ditampilkan juga menyimpan tampilan tempat Anda berada dengan cara yang sama, dan Terapkan membawa Anda kembali ke sana.

## Help ▸ How it works

help-how-escape = Waktu lolos & warna halus
help-how-escape-text = Setiap piksel mengiterasi rumus sampai besarnya melampaui radius bailout atau mencapai batas iterasi. Jumlah langkah mentah saja menghasilkan pita yang tegas; menambahkan bagian pecahan yang diturunkan dari besaran akhir menghasilkan warna yang kontinu tanpa pita.
help-how-precision = Posisi dengan presisi sembarang
help-how-precision-text = Bilangan 64-bit biasa kehabisan digit di sekitar zoom 1e15×. Fractadyne menyimpan pusat tampilan dalam presisi sembarang, dengan jumlah digit yang bertambah saat Anda melakukan zoom, sehingga lokasinya tidak pernah menurun ketepatannya. Skala piksel juga disimpan dengan eksponen yang diperluas, sehingga tidak tertahan pada batas ~1e308× milik 64-bit — kedalaman hanya dibatasi oleh kesabaran, bukan oleh rentang bilangan tetap apa pun.
help-how-perturbation = Perturbasi
help-how-perturbation-text = Mengiterasi setiap piksel dalam presisi tinggi akan terlalu lambat. Sebagai gantinya, satu piksel referensi diiterasi dalam presisi tinggi di CPU (“orbit referensi”), dan setiap piksel lainnya dihitung di GPU sebagai selisih kecil δ darinya dalam presisi rendah yang cepat: δz → 2·Z·δz + δz² + δc.
help-how-floatexp = Kedalaman ekstrem (floatexp)
help-how-floatexp-text = Setelah sekitar 1e28×, bahkan selisih kecil itu mengalami underflow di luar rentang 32-bit, sehingga disimpan sebagai mantisa ditambah eksponen bilangan bulat terpisah (“floatexp”), yang menyingkirkan dinding kedalaman 32-bit. Mesin beralih secara otomatis: matematika langsung saat dangkal, perturbasi saat dalam, dan floatexp saat paling dalam. Kedalaman kemudian dibatasi oleh presisi koordinat dan anggaran iterasi, bukan oleh batas tetap — diperiksa silang terhadap Fraktaler-3 pada korpus referensi 39 lokasi, yang terdalam pada 5.63e18003× (38 baris non-ekstrem adalah gerbang rutin, hingga 6.13e1105×), dengan rangkaian uji konsistensi presisi internal hingga 1e1000000×; tur bawaan mencapai ~1.3e420×.
help-how-reference = Pemilihan referensi & rebasing
help-how-reference-text = Referensi dipilih (dinilai dalam presisi tinggi) agar orbitnya tetap berada di dalam tampilan selama mungkin. Ketika selisihnya tumbuh terlalu besar, selisih itu di-“rebase” kembali ke referensi agar tetap akurat.
help-how-series = Aproksimasi deret
help-how-series-text = Pada zoom dalam, iterasi-iterasi awal hampir tidak menggeser δ dari referensi, sehingga dapat dihitung sekaligus: δz didekati dengan polinomial pendek dalam δc (orde 3), yang koefisiennya diiterasi di sepanjang referensi. Perender melompat maju ke iterasi terakhir di mana polinomial itu masih akurat untuk seluruh tampilan, lalu beriterasi seperti biasa — menghemat langkah-langkah yang dilompati tanpa mengubah gambar (aktifkan/nonaktifkan di Kontrol ▸ Lanjutan). Mandelbrot dan Multibrot 3–8, rentang terdalam; tidak dengan pewarnaan rata-rata garis / ketaksamaan segitiga / perangkap orbit / dekomposisi (yang memerlukan setiap iterasi). Di mana BLA aktif, BLA sudah mencakup lompatan awal ini.
help-how-distance = Estimasi jarak & pencahayaan
help-how-distance-text = Melacak turunan dz/dc menghasilkan jarak setiap piksel ke batas. Itulah yang menggerakkan pencahayaan relief 3D, pendaran jarak, dan metode pewarnaan “Estimasi jarak” — semuanya valid pada kedalaman zoom berapa pun.
help-how-safety = Anti-aliasing & keamanan
help-how-safety-text = Gambar diam di-supersampling (2–8× per sumbu) begitu tampilan stabil. Anggaran kerja menjaga satu penggambaran GPU tetap dalam batas watchdog driver dengan menurunkan resolusi (tidak pernah jumlah iterasi) pada pengaturan ekstrem, sehingga tampilan dalam tetap detail alih-alih menjadi kosong.

## Help ▸ Command line (the flags themselves stay English)

# The section's introduction. The flags listed below it are the command line itself, English in every language.
help-cli-intro = Fractadyne dapat berjalan tanpa antarmuka (headless) untuk render, ekspor film/tur, otomasi, pemeriksaan gambar acuan (golden image), dan tolok ukur. Tanpa flag, Fractadyne meluncurkan GUI interaktif.

## Help ▸ Shortcuts

# What each mouse gesture and key does (the gestures are the Navigation section's).
help-shortcuts-pan = Geser
# A gesture: turning the mouse wheel.
help-key-wheel = Roda
help-shortcuts-wheel = Zoom di posisi kursor
help-shortcuts-box-zoom = Zoom kotak (seret persegi panjang untuk zoom ke sana)
# $icon: the click-to-zoom toolbar button's symbol. "tool on": while that button is on.
help-key-click-tool = Klik (alat { $icon } aktif)
help-shortcuts-click = Menyelam ke titik itu; klik kanan untuk mundur
# A heading: the keyboard shortcuts.
help-shortcuts-keyboard = Papan ketik
help-shortcuts-space = Perbesar / perkecil berkelanjutan
help-shortcuts-undo = Urungkan tampilan
help-shortcuts-redo = Ulangi tampilan
help-shortcuts-minibrot = Temukan minibrot + zoom ke sana (Newton-Raphson)
help-shortcuts-autopilot = Autopilot zoom otomatis (menyelam ke arah detail; masukan apa pun menghentikannya)
# off / strip / panel / full: the panel's sizes, as View ▸ Live render panel names them.
help-shortcuts-live-panel = Panel render langsung: mati / lajur / panel / penuh (arahkan kursor ke sebuah instrumen untuk melihat artinya; tombol × menutupnya)
help-shortcuts-esc = Hentikan autopilot / tur yang sedang diputar, atau keluar dari layar penuh
help-shortcuts-snapshot = Cuplikan ke folder ekspor terakhir — tangkapan layar tampilan seperti yang terlihat, atau render penuh dengan pengaturan ekspor; penekanan pertama menanyakan yang mana, dan Berkas → Pengaturan → Cuplikan mengubahnya.
help-shortcuts-help = Buka bantuan ini
# A heading: the shortcuts of the Game of Life, with the menu path to it.
help-shortcuts-life = Life (Fraktal → Automata → Life)
help-shortcuts-life-run = Jalankan / jeda semesta
help-shortcuts-life-step = Maju satu generasi / maju sebanyak lompatan (diatur di panel Life)
help-shortcuts-life-draw = Alat gambar aktif / nonaktif (pensil di bilah alat)
# A gesture. "Draw on": while the Life draw tool is on.
help-key-click-drag-draw = Klik / seret (Gambar aktif)
help-shortcuts-life-paint = Balik sebuah sel / lukis: goresan yang dimulai pada sel mati akan menggambar, goresan yang dimulai pada sel hidup akan menghapus. Semesta yang sedang berjalan dijeda selama Anda menggambar dan berlanjut saat Anda melepaskannya.
# $key: a keyboard shortcut (Ctrl+Z). "Draw on": while the Life draw tool is on.
help-key-when-drawing = { $key } (Gambar aktif)
help-shortcuts-life-undo = Urungkan suntingan, isian, pengosongan, atau perubahan aturan terakhir

## Help ▸ Recommended hardware

help-hardware-intro = Fractadyne membagi pekerjaannya antara GPU (iterasi per piksel + pewarnaan) dan CPU (orbit referensi presisi sembarang yang menjadi dasar zoom dalam). Keduanya penting, tetapi untuk hal yang berbeda.
help-hardware-gpu = GPU (paling penting)
help-hardware-gpu-text = Setiap piksel diiterasi dan diwarnai di GPU, sehingga kecepatan GPU menentukan laju bingkai dan waktu ekspor. GPU diskret dengan throughput 32-bit (presisi tunggal) yang kuat memberikan zoom dalam yang paling mulus. GPU terintegrasi berfungsi dengan baik — hanya lebih lambat, dan lebih cepat mencapai anggaran kerja per bingkai (pada titik itu perender memangkas resolusi, bukan detail). Memerlukan API grafis modern: Vulkan, Direct3D 12, Metal, atau OpenGL melalui wgpu.
# The left column of a requirements table.
help-hardware-minimum = Minimum
help-hardware-gpu-minimum = GPU apa pun dengan Vulkan / DX12 / Metal / GL 3.3.
help-hardware-recommended = Disarankan
help-hardware-gpu-recommended = GPU diskret (sekitar 5 tahun terakhir); VRAM >= 2 GB.
help-hardware-big-exports = Untuk ekspor besar
help-hardware-gpu-big-exports = VRAM >= 4 GB (dirender per petak, jadi kinerjanya menurun secara bertahap).
# A heading: the processor.
help-hardware-cpu = CPU
help-hardware-cpu-text = Orbit referensi presisi tinggi (dan tabel BLA) dihitung di CPU dan sebagian besar berjalan dalam satu thread, sehingga kinerja per inti yang cepat paling berpengaruh — itulah yang menentukan seberapa cepat tampilan dalam terselesaikan setelah Anda bergerak. Inti tambahan terutama membantu pengodean ekspor di latar belakang. Diperlukan CPU 64-bit.
help-hardware-cpu-recommended = CPU 64-bit modern dengan kecepatan inti tunggal yang baik.
help-hardware-memory = Memori
help-hardware-memory-text = Referensi dalam bertambah besar seiring kedalaman (banyak iterasi dalam presisi tinggi); ekspor yang sangat besar menyimpan petak di RAM. 8 GB cukup nyaman untuk penjelajahan sehari-hari; 16 GB+ cocok untuk kedalaman ekstrem dan ekspor yang sangat besar.
help-hardware-checking = Memeriksa perangkat keras Anda
help-hardware-diagnostics = Bantuan → “Diagnostik…” menjalankan uji yang sama dengan yang dijalankan pengembang, di mesin Anda. “Jalankan uji mandiri” memeriksa matematika dan render terhadap hasil yang diketahui benar dalam sekitar lima belas detik; “Jalankan uji UI” menelusuri antarmuka dan tampilan langsung pada beberapa kedalaman zoom, sambil mengambil tangkapan layar; “Jalankan pemeriksaan aritmetika GPU” memverifikasi aritmetika double-float shader pada setiap backend grafis yang tersedia di mesin Anda, dalam beberapa detik dan tanpa jendela. Masing-masing berjalan sebagai proses terpisah, sehingga uji yang mengganggu driver grafis hanya menjatuhkan uji itu sendiri, bukan sesi Anda.
help-hardware-gpu-check = Pemeriksaan aritmetika GPU tidak memiliki hasil lulus atau gagal — pemeriksaan ini mencatat cara GPU dan driver Anda menghitung, dan pada sebagian perangkat keras (setiap kartu NVIDIA yang dicoba sejauh ini) pemeriksaan ini melaporkan bahwa kompiler shader membuang sebagian aritmetika presisi diperluas. Itu adalah temuan nyata yang sedang kami coba petakan di berbagai vendor, bukan kerusakan pada mesin Anda, jadi apa pun hasilnya, melampirkannya ke laporan benar-benar membantu.
help-hardware-report = Jika ada yang tampak salah, jalankan uji mandiri dan gunakan “Lampirkan ke laporan masalah…”: laporan itu kemudian membawa hasil yang divalidasi mesin dari perangkat keras Anda sendiri, yang jauh lebih berguna daripada deskripsi saja. Satu catatan yang perlu diketahui — gambar referensi dibuat pada satu kartu grafis tertentu, dan GPU yang berbeda secara sah menghitung piksel yang sedikit berbeda, sehingga perbandingan gambar diperiksa dengan toleransi yang lebih longgar pada perangkat keras lain, dan perbedaan kecil di sana memang wajar, bukan kesalahan.
help-hardware-notes = Catatan
help-hardware-note-watchdog = GPU yang mumpuni mencegah anggaran kerja watchdog turun tangan pada pengaturan ekstrem.
help-hardware-note-binary = Build ini berupa satu biner yang mandiri — tidak perlu memasang runtime.
help-hardware-note-benchmark = Jalankan tolok ukur bawaan (Alat ▸ Tolok ukur) untuk mengukur kemampuan mesin Anda.

## Help ▸ Acknowledgments & citations
## Names of people, programs, papers, books and websites stay as written, and so do the
## titles in quotation marks (the works' own titles); the prose around them translates.

help-ack-title = Ucapan terima kasih & sitasi
# The dedication, in italics at the top of the section.
help-ack-dedication = Dipersembahkan untuk Stone Soup Group — para sukarelawan di balik Fractint (sejak 1988).
help-ack-dedication-text = Kode yang mereka bagikan secara bebas dan semangat penjelajahan fraktal yang terbuka dan kolaboratif adalah alasan begitu banyak dari kita jatuh cinta pada seni ini. Fractadyne dibangun dalam tradisi itu.
help-ack-intro = Fractadyne berdiri di atas karya komunitas fraktal selama puluhan tahun. Teknik dan karya terdahulu yang menjadi dasarnya — diverifikasi terhadap sumber-sumber di bawah:
help-ack-deep-zoom = Algoritme zoom dalam
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-martin-title = Perturbasi & aproksimasi deret — { $who }
help-ack-martin = Metode orbit referensi + delta presisi rendah (δz → 2Z·δz + δz² + δc) dan deret orde n yang melompati iterasi awal, diperkenalkan dalam SuperFractalThing dan catatannya “SuperFractalThing Maths” (sft_maths.pdf, 2013).
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bla-title = Aproksimasi linear bivariat (BLA) & rebasing — { $who }
help-ack-bla = Melompati rangkaian iterasi melalui pemetaan linear yang digabungkan, dan rebasing ke satu referensi untuk menghindari glitch; dikembangkan di fractalforums.org (2021).
# A link's label. $who: the linked page's author.
help-ack-bla-link = Penjelasan BLA — { $who }
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-glitch-title = Kriteria deteksi glitch — { $who }
help-ack-glitch = Uji |Z + z| << |Z| untuk menentukan kapan sebuah piksel memerlukan referensi lain — dasar perturbasi bebas glitch (fractalforums).
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-diffabs-title = Perturbasi non-analitik (Burning Ship) — { $who }
help-ack-diffabs = Analisis kasus nilai mutlak “diffabs” yang membuat perturbasi bekerja untuk keluarga berbasis abs (Burning Ship / Celtic / Buffalo), di fractalforums.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mathr-title = Implementasi rujukan & pemeriksaan silang — { $who }
help-ack-mathr = Fraktaler-3 dan Kalles Fraktaler 2+ (aslinya oleh Karl Runmo) — yang tulisannya memandu mesin ini, dan yang keluaran EXR-nya digunakan Fractadyne untuk memeriksa silang perendernya.
# A heading: credits for the coloring methods.
help-ack-coloring = Pewarnaan
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-harkonen-title = Jumlah iterasi halus & Rata-rata garis — { $who }
help-ack-harkonen = Pewarnaan waktu lolos yang kontinu (dinormalisasi) dan metode Rata-rata garis (Stripe Average), dari “On Smooth Fractal Coloring Techniques” (tesis M.Sc., 2007), yang juga menganalisis pewarnaan Ketaksamaan segitiga / rata-rata percabangan.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mitchell-title = Rata-rata ketaksamaan segitiga — { $who }
help-ack-mitchell = Merata-ratakan tempat setiap z² + c mendarat di dalam batas-batas ketaksamaan segitiga, untuk tekstur luar yang halus mirip nyala api.
# A link's label: the person's tutorial pages. $who: the person's name.
help-ack-mitchell-link = { $who } — tutorial
help-ack-distance = Pewarnaan/pencahayaan estimasi jarak menggunakan metode jarak batas dz/dc yang klasik (lihat Peitgen & Saupe, “The Science of Fractal Images”, 1988).
help-ack-lsystems-life = Sistem-L & Game of Life
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-lindenmayer-title = Sistem-L — { $who }
help-ack-lindenmayer = Penulisan ulang string secara paralel, diperkenalkan pada 1968 untuk memodelkan cara tumbuhan dan alga tumbuh (“Mathematical models for cellular interactions in development”, Journal of Theoretical Biology).
help-ack-abop = Buku (1990) yang interpretasi kura-kura dan notasinya untuk sistem stokastik, peka konteks, dan parametrik diikuti oleh pembaca sistem-L; sistem bawaan yang bertanda ABOP dimasukkan dari gambar-gambar di buku itu.
# A link's label: the book can be read free on that website. $site: the website's name.
help-ack-abop-link = Bukunya, gratis dari { $site }
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bourke-title = Koleksi sistem-L — { $who }
help-ack-bourke = Banyak sistem bawaan berasal dari halaman sistem-L miliknya, di antaranya aturan Chris Wallace untuk kurva Sierpinski dan semak Dietmar Saupe. Berkas .l Fractint, yang dapat dibuka langsung, menggunakan format Stone Soup Group.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-conway-title = Game of Life — { $who }
help-ack-conway = Automaton seluler B3/S23 (1970), yang dikenal dunia melalui kolom “Mathematical Games” karya Martin Gardner di Scientific American (Oktober 1970).
# A credit's heading: the website, then who runs it. $site: the website's name.
help-ack-lifewiki-title = { $site } — komunitas ConwayLife.com
help-ack-lifewiki = Rujukan untuk notasi aturan yang dibaca aplikasi (B/S, von Neumann, Generations, huruf-huruf Alan Hensel untuk aturan non-totalistik, dan MAP), untuk format pola RLE yang digunakannya bersama Golly, serta untuk aturan dan pola bawaan, yang penemunya dicatat di sana.
help-ack-foundations = Fondasi & perkakas
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mandelbrot-title = Himpunan Mandelbrot — { $who }
help-ack-mandelbrot = Objek yang menjadi inti dari semuanya.
help-ack-built-with = Dibangun dengan Rust, egui/eframe (emilk), dan wgpu (gfx-rs); presisi sembarang melalui astro-float, float kata ganda melalui twofloat; PNG/OpenEXR, rfd, serde. Terima kasih kepada para penulis dan pengelolanya.
help-ack-errors = Kesalahan atribusi apa pun adalah tanggung jawab penulis; koreksi dipersilakan di pelacak issue proyek.

## Help ▸ Third-party licenses

help-licenses-title = Lisensi pihak ketiga
# $licenses: Fractadyne's license expression (MIT OR Apache-2.0), as written. $url: the web address of a crate's source. THIRD-PARTY-NOTICES.md is a file name, `option-ext` a crate's name, MPL-2.0 a license's.
help-licenses-intro = Fractadyne didistribusikan sebagai biner bertaut statis yang mencakup crate sumber terbuka dan fon bawaan di bawah ini. Teks lisensi lengkapnya (juga ada di THIRD-PARTY-NOTICES.md, yang disertakan dalam unduhan) tercantum berikut ini. Kode Fractadyne sendiri berlisensi ganda { $licenses }. `option-ext` berlisensi MPL-2.0 — kode sumbernya yang tidak diubah ada di { $url }.
# A button, after a copy icon: copies every notice to the clipboard.
help-licenses-copy-all = Salin semua pemberitahuan

## Help ▸ About

help-about-tagline = Penjelajah fraktal native yang dibangun dengan Rust dan wgpu.
# The arithmetic deep zoom has used. $ran: the libraries that have run so far, as names (or a note that none has yet); $built: those this build contains, with versions.
help-about-arithmetic = Aritmetika zoom dalam: { $ran } (build ini berisi: { $built }).
help-about-files = Berkas & data
# Followed by the folder's path.
help-about-files-text = Sesi Anda (tampilan saat ini, pewarnaan, dan preferensi), markah beserta gambar mininya, gradien tersimpan, dan rumus tersimpan disimpan di folder per pengguna ini, dan dimuat ulang pada peluncuran berikutnya:
# Tooltip of the copy button beside the folder's path.
help-about-copy-path = Salin jalur
# $command: a command typed in a terminal (it stays English).
help-about-session = Berkas sesi memiliki versi; jika berkas itu ditulis oleh Fractadyne yang lebih baru daripada yang dapat membacanya sepenuhnya, aplikasi memuatnya sebisa mungkin dan memperingatkan Anda. Untuk memulai benar-benar dari awal, gunakan Berkas ▸ Atur ulang status aplikasi… (meminta konfirmasi terlebih dahulu), atau jalankan `{ $command }` dari terminal.
help-about-license = Lisensi
# $licenses: the license expression (MIT OR Apache-2.0), which stays as written.
help-about-license-text = { $licenses } — gunakan di bawah salah satu lisensi, sesuai pilihan Anda.
# A link to the source code, followed by ↗.
help-about-source = Kode sumber di GitHub

## The live render panel ("blinkenlights": View ▸ Live render panel, key B)
## A front panel drawn over the view like a piece of 1970s test equipment. Its legends are engraved
## in capitals: write them in capitals here (the panel shows them exactly as written, it does not
## uppercase them). Numbers, units and readouts on its tubes and dials are not translated.

# Tooltip of the × button in the panel's top-right corner. The code adds the key after it: (B).
live-close-hover = Tutup panel render langsung
# The legend engraved along the top plate, left of the maker's badge.
live-plate = RENDER LANGSUNG · PANEL DEPAN

## Live render panel: instrument titles
## Engraved above each instrument, and repeated as the title of its help card.

# Above the depth readout, whose tubes show the magnification as a power of two ("2^" stays).
live-title-depth = KEDALAMAN 2^
live-title-iterations = ITERASI
# Above the frame-time readout (milliseconds).
live-title-frame-ms = MS BINGKAI
# Above the date-and-time readout.
live-title-clock = TANGGAL · WAKTU
# The walk dial: a radar screen showing a slow refresh rendered pass after pass, band after band.
live-title-walk = PENELUSURAN · PITA
# A meter of how far the picture on screen lags the view.
live-title-age = UMUR GAMBAR
# What part of the picture this frame's GPU pass covered.
live-title-tiles = PETAK
# A scope of the GPU's work per frame, in milliseconds.
live-title-scope = LINTASAN GPU · MS
# The depth dial: depth, the reference orbits being built, and the arithmetic in use.
live-title-dial = KEDALAMAN · REFERENSI · ARITMETIKA
# Iteration steps per second.
live-title-steps = LANGKAH / S
# The two GPUs (the window's and a second, worker GPU).
live-title-engines = DUA MESIN
live-title-memory = MEMORI
live-title-cores = INTI · THREAD
live-title-pixels = ALIRAN PIKSEL
# Help card titles only: the three GPU timer lamps (T0, T1, T2 stay as they are) and the row of lamps.
live-title-timers = PEWAKTU GPU T0–T2
live-title-bank = DERET LAMPU

## Live render panel: the light bank's lamps
## Engraved under each lamp, and the title of its help card. A cell is narrow (about eight capital
## letters): a longer label breaks at a space onto two lines, and is drawn smaller if it still does
## not fit.

live-lamp-plan = RENCANA
live-lamp-dispatch = KIRIM
live-lamp-live = KINI
live-lamp-held = DITAHAN
# The last picture reprojected (stretched) to fit the moving view.
live-lamp-reproj = REPROY
live-lamp-reading = BACAAN
# A pass of a pinned walk.
live-lamp-pin-pass = LINTASAN KUNCI
live-lamp-split = TERBAGI
live-lamp-adopt = ADOPSI
live-lamp-abandon = BATAL
# "REF": a reference orbit. Built for the view as it is (LIVE), ahead of it (AHEAD), or for a tour's next hold (HOLD).
live-lamp-ref-live = REF KINI
live-lamp-ref-ahead = REF DEPAN
live-lamp-ref-hold = REF TAHAN
# The series approximation.
live-lamp-series = DERET
# The second (worker) GPU.
live-lamp-worker = PEKERJA
live-lamp-tile = PETAK
# Supersampling (anti-aliasing); usually kept as SSAA.
live-lamp-ssaa = SSAA
live-lamp-palette = PALET
# The arithmetic in use (direct, df32, floatexp).
live-lamp-mode = MODE

## Live render panel: the kinds of GPU pass
## In the GPU scope's legend (MOVING, PIN, WORKER, SETTLE, SSAA). SETTLE and SSAA also name a still
## view's walk on the walk dial.

# A moving frame's pass.
live-kind-moving = BERGERAK
# A pass of a pinned walk.
live-kind-pin = KUNCI
# The worker GPU's passes.
live-kind-worker = PEKERJA
# A still view's settle tile or walk pass.
live-kind-settle = STABIL
# A supersampling sample.
live-kind-ssaa = SSAA

## Live render panel: legends and readouts on the instruments

# On the walk dial during a split refresh. $done: passes done; $passes: passes in all.
live-walk-split = TERBAGI { $done }/{ $passes }
# On the picture-age meter's face, under its scale (0 to 2 octaves).
live-age-scale = OKTAF TERTINGGAL
# The depth dial's band for direct (plain floating-point) arithmetic. The other bands, DF32 and FLOATEXP, are names and stay.
live-dial-direct = LANGSUNG
# Under the two GPU gauges: GPU 0 draws the window; GPU 1 is the worker, or there is none.
live-engines-gpu-window = GPU 0 · JENDELA
live-engines-gpu-worker = GPU 1 · PEKERJA
live-engines-no-worker = TANPA PEKERJA
# The share of displayed frames each GPU made. $window and $worker: percentages, 0–100.
live-engines-on-screen = DI LAYAR { $window } : { $worker }
# The same readout before any frame was shown.
live-engines-on-screen-none = DI LAYAR —
# Under the CPU core bars. $count: the machine's cores.
live-cores-count = { $count } INTI
# Under the threads scope. $count: the app's busy threads.
live-cores-threads = THREAD { $count }

## Live render panel: the memory tanks' legend
## What the GPU memory holds, by kind. Very narrow: about ten letters.

# G-buffers (the picture's working textures).
live-memory-gbuf = G-BUF
# The held picture and the supersampling accumulator.
live-memory-hold = TAHAN+SSAA
# The reference orbit and its BLA (bilinear approximation) table.
live-memory-orbit = ORBIT+BLA
# The worker GPU's copy of the reference.
live-memory-worker = REF PKRJ

## Live render panel: the roles on the threads scope
## Tiny labels on a small scope: at most four letters each.

# The reference builders.
live-cores-role-ref = REF
# The scoring pool.
live-cores-role-pool = SKOR
# The thread driving the worker GPU.
live-cores-role-worker = PKRJ
# Probes: short side renders.
live-cores-role-probe = UJI
# Image and tour exports.
live-cores-role-export = EKS

## Live render panel: the tiles instrument's captions
## The line under the picture's plane, then the tile size (left) and the tiles a frame (right).

# A walk's pass. $lo and $hi: the iteration band it rendered (numbers).
live-tiles-pin-band = LINTASAN KUNCI · PITA { $lo }–{ $hi }
live-tiles-settle-band = LINTASAN STABIL · PITA { $lo }–{ $hi }
live-tiles-moving-whole = BINGKAI BERGERAK · GAMBAR UTUH
live-tiles-settled-whole = BINGKAI STABIL · GAMBAR UTUH
# $pass: this pass; $passes: the passes the refresh is split into.
live-tiles-split-pass = PENYEGARAN TERBAGI · LINTASAN { $pass } DARI { $passes }
# $grid: the grid's columns × rows, e.g. 12×8.
live-tiles-grid-done = KISI STABIL { $grid } · SELESAI
# $grid: the grid's columns × rows; $done and $total: tiles done, and in all.
live-tiles-grid-progress = KISI STABIL { $grid } · { $done } DARI { $total } SELESAI
# The same, while a tile too costly for one frame goes out in strips.
live-tiles-grid-strips = KISI STABIL { $grid } · { $done } DARI { $total } SELESAI · PER LAJUR
# $samples: supersampling samples folded so far; $lo and $hi: the iteration band the next one is walking.
live-tiles-ssaa-band = SSAA · { $samples } SELESAI · PITA { $lo }–{ $hi }
# $sample: the supersampling sample.
live-tiles-ssaa-whole = SAMPEL SSAA { $sample } · BINGKAI UTUH
# No GPU pass this frame, and the panel has not seen a frame yet.
live-tiles-idle-starting = MEMULAI
# No GPU pass this frame: the last complete picture is held on screen.
live-tiles-idle-held = DITAHAN
# No GPU pass this frame: the last picture is stretched to fit the view.
live-tiles-idle-reprojected = DIREPROYEKSI
# No GPU pass this frame: the picture on screen is current.
live-tiles-idle-live = KINI
# No GPU pass this frame, and nothing to show.
live-tiles-idle-none = DIAM
# The tile size: $res, a whole picture's resolution (2400×1300).
live-tiles-tile-res = PETAK { $res }
# The tile size: $px, a tile's side in pixels.
live-tiles-tile-px = PETAK { $px } PX
live-tiles-no-pass = TANPA LINTASAN
# Tiles a frame. $count: how many (0 when none).
live-tiles-rate-count = { $count }/BINGKAI
live-tiles-rate-one = 1 PETAK/BINGKAI
# $share: the part of one tile sent each frame, e.g. 0.33.
live-tiles-rate-part = { $share } PETAK/BINGKAI
live-tiles-rate-sample = 1 PETAK/SAMPEL
# In the plane's corner during a walk: the share of pixels still iterating. $percent: a percentage, e.g. 54 or 1.5.
live-tiles-running = BERJALAN { $percent }%

## Live render panel: help cards
## Shown when the pointer rests on an instrument or a lamp. Words in capitals (SETTLE, SSAA, ABANDON,
## HELD, REPROJ, ADOPT, REF AHEAD, ON SCREEN) are the panel's own legends: use the same words as
## their messages above.

# The heading of a card's list of what healthy looks like.
live-card-healthy = SEHAT
# The heading of a card's list of what signals trouble.
live-card-trouble = BERMASALAH

live-card-walk-what = Penyegaran dalam yang terlalu mahal untuk satu bingkai akan dikunci dan ditelusuri: dirender lintasan demi lintasan, masing-masing membiarkan piksel beriterasi satu oktaf lebih jauh (256, 512, 1.024 … hingga jumlah yang diminta). Tepian dial adalah jumlah itu pada skala log, dimulai dari atas; sinarnya adalah kemajuan penelusuran; setiap busur adalah satu lintasan, diwarnai menurut waktu GPU-nya terhadap target 20 ms. Titik putih menunggu bacaan lintasan terakhir; kilatan adalah gambar baru yang masuk ke layar.
live-card-walk-note = Tampilan diam yang terlalu mahal untuk satu lintasan ditelusuri dengan cara yang sama, ditandai STABIL, dan setiap sampel supersampling ditelusuri lagi, ditandai SSAA: lintasannya masing-masing boleh memakan hingga 400 ms, dan busurnya diwarnai terhadap angka itu. Penyegaran terbagi justru menggambar satu cincin per lintasan: setiap pita, sebagian dari petak.
live-card-walk-good-sweep = Sapuan yang berputar dalam beberapa bingkai dan berakhir dengan kilatan.
live-card-walk-good-arcs = Busur sebagian besar hijau dan kuning.
live-card-walk-good-still = Pada tampilan diam yang dalam: sapuan yang lebih lambat per sampel, busurnya hijau hingga kuning.
live-card-walk-bad-red = Busur merah: lintasan melebihi anggaran, terasa sebagai patah-patah.
live-card-walk-bad-restart = Sapuan yang berhenti di tengah jalan lalu mulai lagi: penelusuran dibatalkan, dan gambar di layar tetap lama.
live-card-walk-bad-hollow = Busur berongga: lintasan yang pewaktuannya tidak pernah kembali, sehingga lintasan berikutnya ditakar tanpa data.

live-card-age-what = Seberapa jauh gambar di layar tertinggal dari tampilan, dalam oktaf zoom (satu oktaf adalah 2×). Selama penyegaran dirender, gambar lama ditampilkan teregang dan menua; saat penyegaran tiba, umurnya turun. Lajur di bawahnya adalah beberapa detik terakhir.
live-card-age-good-sawtooth = Gigi gergaji antara sekitar 0,3 dan 0,8, dengan jarum berayun di sekitar tanda 0,5.
live-card-age-good-still = Mendekati 0 begitu Anda berhenti.
live-card-age-bad-red = Jarum terus berada di atas 1,0, di zona merah: gambar tertinggal lebih dari 2× dan tampak lembek atau kotak-kotak saat zoom.
live-card-age-bad-ramp = Tanjakan yang tidak pernah turun: penyegaran tidak selesai (perhatikan BATAL).

live-card-tiles-what = Bagaimana lintasan GPU bingkai ini mencakup gambar. Bingkai bergerak dan lintasan penelusuran mencakup gambar secara utuh, dan keterangannya menyebutkan pita iterasi lintasan penelusuran. Lintasan penelusuran hanya mengiterasi piksel yang belum lolos: bidangnya terisi sesuai porsi itu (skala log, 0,01% di dasar) dan angkanya terbaca di sudut, sehingga bidang itu menyusut seiring penelusuran makin dalam. Penyegaran yang sedikit terlalu mahal untuk satu bingkai dibagi menjadi hingga 8 lintasan, masing-masing berupa kumpulan petak 16 px yang saling berselang-seling. Tampilan stabil yang terlalu mahal untuk satu lintasan menjadi kisi petak, satu per bingkai dari tengah ke luar, dan petak yang masih terlalu mahal dikirim per lajur. Terbaca di bawah: ukuran petak dan petak per bingkai. Batang di bawah bidang adalah rentang iterasi yang dicakup lintasan, dalam skala log: penelusuran menyalakan pitanya, dengan bagian yang sudah selesai di bawahnya. Bingkai utuh digambar sebagai papan catur yang kotaknya membesar saat dirender di bawah resolusi layar (pecahannya terbaca di sudut), dan selama supersampling sebuah piksel yang diperbesar menunjukkan tempat setiap sampel mendarat.
live-card-tiles-good-whole = Lintasan utuh selama Anda melakukan zoom; penyegaran terbagi yang terdiri dari beberapa lintasan.
live-card-tiles-good-grid = Saat Anda berhenti: kisi berisi puluhan petak yang terisi dalam satu atau dua detik, lalu SSAA.
live-card-tiles-bad-small = Kisi berisi ratusan petak kecil: anggaran setiap lintasan kecil, dan penajaman akan memakan waktu lama.
live-card-tiles-bad-strips = Banyak petak dikirim per lajur: petak berjalan lebih lambat daripada perkiraan saat kisi ditakar.
live-card-tiles-bad-splits = Pembagian tertahan di 8 lintasan: penyegaran nyaris tidak muat, dan sedikit lebih dalam lagi penyegaran kembali ke penelusuran yang lebih lambat.
live-card-tiles-bad-dark = Gelap dalam waktu lama saat bergerak: tidak ada lintasan sama sekali.

live-card-scope-what = Satu kolom per bingkai, yang terbaru di kanan: pekerjaan GPU yang dijalankannya, ditumpuk menurut jenis (bingkai bergerak hijau, lintasan kunci hijau hingga merah menurut biaya, petak stabil atau lintasan penelusuran tampilan diam krem, sampel supersampling lavender, bingkai dan sampel GPU pekerja biru), dalam milidetik. Garis putus-putus pada 16,7 ms (60 fps), 33, dan 50. Tanda berongga adalah lintasan yang pewaktuannya tidak pernah kembali.
live-card-scope-good-under = Kolom di bawah garis 16,7 selama Anda bergerak.
live-card-scope-good-pin = Lintasan kunci hijau hingga kuning.
live-card-scope-good-samples = Sampel supersampling tampilan diam boleh berjalan lebih lama: tidak ada yang bergerak.
live-card-scope-bad-over = Kolom melewati 33 atau 50 ms: patah-patah yang terlihat.
live-card-scope-bad-seconds = Lintasan berdurasi beberapa detik: watchdog GPU dapat mereset perangkat.
live-card-scope-bad-hollow = Banyak tanda berongga: pekerjaan ditakar tanpa pengukuran.

live-card-timers-what = Tiga slot stempel waktu GPU. Sebuah slot menyala saat sebuah lintasan diwaktu dan meredup saat bacaannya kembali, dua atau tiga bingkai kemudian. T0 mewaktu lintasan setiap bingkai; T1 dan T2 mewaktu lintasan kunci tambahan.
live-card-timers-good = T0 berkedip setiap kali ada pekerjaan; T1 dan T2 berkedip selama penelusuran.
live-card-timers-bad-stuck = Lampu yang terus menyala: bacaannya tidak kembali, dan takaran menjadi basi.
live-card-timers-bad-dark = Semua gelap saat GPU sibuk: tidak ada yang diukur.

live-card-dial-what = Jarum merah adalah kedalaman tampilan: perbesarannya sebagai pangkat dua. Pita-pitanya adalah aritmetika yang digunakan GPU di sana: langsung, df32 (dua float per bilangan), dan floatexp (float dengan eksponen lebar) setelah 2^93. Titik-titiknya adalah orbit referensi yang sedang dibangun: hijau untuk tampilan, kuning di depannya ke arah tujuan zoom, biru untuk penahanan tur berikutnya. Cincin adalah pemasangan; silang merah adalah pembangunan yang dibuang karena tampilan menyusulnya.
live-card-dial-good-ahead = Titik kuning yang mendarat di depan jarum dan berubah menjadi cincin sebelum jarum mencapainya.
live-card-dial-good-crosses = Sedikit silang merah.
live-card-dial-bad-passing = Jarum melewati referensi terpasang terakhirnya: bingkai menunggu referensi, terasa sebagai macet.
live-card-dial-bad-crosses = Silang merah berulang: zoom melaju lebih cepat daripada pembangun, dan CPU terbuang.
live-card-dial-bad-crossing = Penyeberangan 2^93 adalah titik lambat yang sudah diketahui: bingkai floatexp pertama lebih mahal.

live-card-bank-what = Setiap lampu menyala saat peristiwanya terjadi dan memudar dalam sekitar 150 ms, sehingga perender yang sibuk berkedip-kedip dan yang menganggur menjadi gelap. Dari kiri ke kanan: bingkai, penyegaran, referensi, lalu pekerja, penstabilan, palet, dan aritmetika.
live-card-bank-note = Arahkan kursor ke sebuah lampu untuk melihat artinya.

live-card-depth-what = Perbesaran sebagai pangkat dua: 2^100 kira-kira 1,3 × 10^30.
live-card-depth-note = Setelah 2^93 setiap langkah lebih mahal (floatexp), dan referensi membutuhkan waktu lebih lama untuk dibangun.

live-card-iter-what = Jumlah iterasi yang diminta: seberapa jauh setiap piksel boleh beriterasi sebelum dianggap berada di dalam himpunan. Jumlahnya naik seiring kedalaman.
live-card-iter-good = Naik dengan stabil saat Anda menyelam.
live-card-iter-bad-low = Tertahan rendah saat dalam: spiral dan minibrot dirender sebagai interior datar.
live-card-iter-bad-high = Sangat tinggi: setiap penyegaran lebih mahal, dan penelusuran memakan waktu lebih lama.

live-card-ms-what = Waktu antara bingkai yang ditampilkan.
live-card-ms-good = Interval layar Anda: 16,7 pada layar 60 Hz.
live-card-ms-bad-stutter = Di atas 33: patah-patah yang terlihat.
live-card-ms-bad-spikes = Lonjakan di atas 50: tersendat, biasanya karena satu lintasan yang mahal (lihat Lintasan GPU).

live-card-clock-what = Jam dinding dalam waktu lokal, hingga sepersepuluh detik.
live-card-clock-note = Jam ini menandai setiap tangkapan layar panel dengan waktu pengambilannya, sehingga gambar suatu masalah dapat dicocokkan dengan log aplikasi.

live-card-tach-what = Langkah iterasi per detik. Putih: langkah yang diminta (piksel × iterasi). Kuning: langkah yang benar-benar dieksekusi GPU. Selisihnya adalah BLA (aproksimasi bilinear) yang melompati banyak iterasi sekaligus; ×N adalah pengali itu. Odometer menjumlahkan seluruh sesi.
live-card-tach-good = Selisih yang lebar pada kedalaman: BLA menjalankan tugasnya, ×10 hingga ×1.000.
live-card-tach-bad-together = Jarum berimpit pada kedalaman: BLA tidak dapat melompat di sini (sering kali di dekat interior minibrot), sehingga penyegaran lambat.
live-card-tach-bad-red = Jarum putih di zona merah: meminta lebih banyak daripada yang dapat dilakukan GPU dalam satu bingkai.

live-card-engines-what = Seberapa sibuk setiap GPU, dengan titik-titik yang membawa bingkainya ke layar: waktu GPU lintasannya selama satu detik terakhir, termasuk bingkai gerak dan sampel supersampling milik pekerja. DI LAYAR adalah porsi bingkai yang ditampilkan yang dibuat oleh masing-masing GPU. Dengan satu GPU, sisi pekerja tetap diam.
live-card-engines-good = Keduanya sibuk selama penyelaman, dengan pekerja menyumbang sebagian bingkai.
live-card-engines-bad-pinned = GPU 0, milik jendela, tertahan mendekati 100%: GPU itulah hambatannya, dan bingkai tersendat.
live-card-engines-bad-idle = Pekerja diaktifkan tetapi diam: pekerja tidak mengirimkan bingkai.

live-card-memory-what = RAM: working set aplikasi, garis merah adalah puncaknya. VRAM: apa yang ditahan aplikasi di GPU, dihitung dari ukuran yang dialokasikannya, menurut jenis (G-buffer, tekstur tahan dan SSAA, buffer orbit dan BLA, salinan referensi milik pekerja), dibandingkan dengan kapasitas kartu.
live-card-memory-good = Datar saat zoom; naik dan turun kembali bertahap seiring referensi datang dan pergi.
live-card-memory-bad-leak = RAM terus naik dan tidak pernah turun: kebocoran memori.
live-card-memory-bad-vram = VRAM mendekati kapasitas kartu: alokasi dapat gagal, atau perangkat dapat hilang.

live-card-cores-what = Penggunaan CPU per inti, hijau ke kuning ke merah, dan sebuah skop tempat setiap thread yang sibuk mengorbit perannya: pembangun referensi, kumpulan penilai, pekerja, render sampingan.
live-card-cores-good = Satu atau dua inti sibuk saat zoom; ledakan singkat di semua inti saat referensi dipilih.
live-card-cores-bad-every = Semua inti sibuk selama beberapa detik: pembangunan referensi pada kedalaman ekstrem.
live-card-cores-bad-own = Inti milik aplikasi tertahan di zona merah: jendelanya sendiri tersendat.

live-card-pixels-what = Piksel yang dihitung per detik, dalam Mpx/s. Redup: masih beriterasi dalam penelusuran. Terang: selesai.
live-card-pixels-good = Puluhan Mpx/s saat bergerak.
live-card-pixels-bad = Mendekati nol saat gambar basi: GPU sedang menunggu sesuatu, referensi atau bacaan.

## Live render panel: the lamps' help cards

live-card-lamp-plan-what = Sebuah bingkai direncanakan: resolusi, jumlah iterasi yang diminta, dan anggaran dipilih, serta apakah perlu menyegarkan.
live-card-lamp-plan-good = Berkedip setiap kali tampilan atau pengaturan berubah.
live-card-lamp-plan-bad = Gelap saat Anda melakukan zoom: thread UI macet.

live-card-lamp-dispatch-what = Pekerjaan dikirim ke GPU pada bingkai ini.
live-card-lamp-dispatch-good = Menyala selama ada yang berubah; gelap saat diam.
live-card-lamp-dispatch-bad = Sibuk padahal tidak ada yang berubah: aplikasi tidak kunjung stabil.

live-card-lamp-live-what = Gambar di layar adalah hasil render bingkai ini sendiri.
live-card-lamp-live-note = Jarang menyala selama penyelaman dalam, dan itu normal: lihat DITAHAN dan REPROY.
live-card-lamp-live-good = Menyala pada kedalaman dangkal, tempat sebuah bingkai murah.

# "Prefer detail" is a setting's name.
live-card-lamp-held-what = Gambar lengkap terakhir ada di layar sementara gambar berikutnya disusun di bawahnya: penyegaran zoom dalam, atau, dengan Utamakan detail, petak atau sampel supersampling berikutnya dari tampilan diam.
live-card-lamp-held-good-dive = Menyala dalam rentang-rentang waktu selama zoom dalam, diselingi ADOPSI.
live-card-lamp-held-good-still = Menyala terus saat tampilan diam menajam, dengan SSAA berkedip.
live-card-lamp-held-bad = Menyala selama beberapa detik saat zoom tanpa ADOPSI: penyegaran tidak selesai.

live-card-lamp-reproj-what = Gambar terakhir, diregangkan agar sesuai dengan tampilan yang bergerak sampai gambar baru siap.
live-card-lamp-reproj-good = Ledakan singkat saat Anda bergerak.
live-card-lamp-reproj-bad = Menyala lama: gambar menjadi makin lembut makin jauh ia diregangkan.

live-card-lamp-reading-what = Sebuah bacaan kembali dari GPU: pewaktuan lintasan, rentang lolos, atau isi.
live-card-lamp-reading-good = Kedipan yang stabil selama merender.
live-card-lamp-reading-bad = Gelap dalam waktu lama selama merender: pewaktuan tidak kembali, dan anggaran hanya menebak.

live-card-lamp-pin-pass-what = Sebuah lintasan dari penelusuran yang dikunci: gambar dirender satu pita iterasi lebih jauh.
live-card-lamp-pin-pass-good = Rangkaian lintasan yang berakhir dengan ADOPSI.
live-card-lamp-pin-pass-bad = Rangkaian yang berakhir dengan BATAL.

live-card-lamp-split-what = Sebuah lintasan dari penyegaran terbagi: sebagian dari petak, dirender dengan jumlah iterasi penuh.
live-card-lamp-split-good = Rangkaian pendek yang berakhir dengan ADOPSI.
live-card-lamp-split-bad = Rangkaian 8, jumlah maksimum yang diizinkan: penyegaran nyaris tidak muat.

live-card-lamp-adopt-what = Penyegaran yang selesai menggantikan gambar di layar.
live-card-lamp-adopt-good = Beberapa kali per detik saat zoom.
# "Picture age" names the instrument (its title, PICTURE AGE).
live-card-lamp-adopt-bad = Jarang saat zoom: gambar menua (lihat Umur gambar).

live-card-lamp-abandon-what = Penyegaran yang sedang berlangsung dibuang: tampilan bergerak terlalu jauh, penyegaran itu menjadi terlalu tua, atau penyegaran yang lebih baru menggantikannya. Pembangunan referensi yang disusul tampilan menyalakannya dengan redup.
live-card-lamp-abandon-good = Jarang, atau sekali tepat setelah lompatan.
live-card-lamp-abandon-bad = Sering: pekerjaan terbuang, dan gambar tetap lama.

live-card-lamp-ref-live-what = Orbit referensi sedang dibangun untuk tampilan sebagaimana adanya: tampilan belum memilikinya, telah meninggalkan yang dimilikinya, atau memerlukan yang lebih panjang.
live-card-lamp-ref-live-good = Kedipan singkat.
live-card-lamp-ref-live-bad-long = Rentang waktu yang panjang: tampilan dalam menunggunya.
live-card-lamp-ref-live-bad-lookahead = Sering menyala selama penyelaman sementara REF DEPAN gelap: lookahead tidak mampu mengimbangi.

live-card-lamp-ref-ahead-what = Referensi lookahead sedang dibangun lebih dalam daripada tampilan, ke arah tujuan zoom.
live-card-lamp-ref-ahead-good = Kedipan teratur selama penyelaman.
live-card-lamp-ref-ahead-bad = Gelap selama penyelaman dalam: tampilan akan mendahului referensinya.

live-card-lamp-ref-hold-what = Prefetch penahanan tur: referensi untuk bingkai kunci tertahan berikutnya, dibangun selama luncuran ke arahnya sehingga pemutaran tidak pernah menunggu.
live-card-lamp-ref-hold-note = Gelap di luar pemutaran tur.
live-card-lamp-ref-hold-good = Satu kedipan menjelang setiap penahanan dalam tur.
live-card-lamp-ref-hold-bad = Menyala selama seluruh penahanan: prefetch terlambat, dan tur menunggunya.

live-card-lamp-series-what = Aproksimasi deret untuk referensi baru, yang memungkinkan setiap piksel melompati iterasi pertamanya.
live-card-lamp-series-good = Singkat.
live-card-lamp-series-bad = Lama pada kedalaman: menunda pemasangan referensi.

live-card-lamp-worker-what = GPU kedua mengirimkan sebuah bingkai.
live-card-lamp-worker-good = Kedipan stabil saat GPU pekerja diaktifkan.
live-card-lamp-worker-bad = Gelap padahal pekerja diaktifkan: pekerja tidak mengirimkan apa pun.

live-card-lamp-tile-what = Sebuah petak stabil, atau satu lajurnya, dikirim ke GPU.
live-card-lamp-tile-good = Ledakan singkat setelah Anda berhenti, lalu tenang.
live-card-lamp-tile-bad = Tidak pernah berhenti: kisi terus dimulai ulang, karena ada sesuatu yang terus mengubah tampilan atau pengaturan.

live-card-lamp-ssaa-what = Sebuah sampel supersampling ditambahkan ke gambar yang sudah stabil.
live-card-lamp-ssaa-good = Serangkaian kedipan setelah tampilan stabil, lalu gelap.
live-card-lamp-ssaa-bad = Dimulai ulang berkali-kali: penstabilan terus terganggu.

live-card-lamp-palette-what = Normalisasi warna menerima rentang lolos yang baru, sehingga warna mengikuti zoom.
live-card-lamp-palette-good = Berkedip saat zoom.
live-card-lamp-palette-bad = Gelap saat zoom: rentang palet membeku, dan warna menjadi datar dan kontrasnya rendah.

live-card-lamp-mode-what = Aritmetika berubah: langsung, df32, atau floatexp.
live-card-lamp-mode-good = Satu kedipan pada sebuah batas seperti 2^93.
live-card-lamp-mode-bad = Berkedip bolak-balik: tampilan terombang-ambing di sekitar sebuah batas.

## The Render client window (File ▸ Render client…): lending this machine's GPU to a render farm

farm-client-title = Klien render
farm-client-intro = Render bingkai untuk sebuah render farm. Pengendali — Alat ▸ Render di farm… pada mesin lain — menampilkan alamat dan kunci farm-nya. Mesin ini tidak membuka port: mesin inilah yang menyambung ke pengendali.
# A field label, and the label of the status row that names the controller.
farm-client-controller = Pengendali
farm-client-find = Cari
farm-client-find-hover = Cari pengendali di jaringan ini (pengendali menjawab selama farm-nya sedang mendengarkan)
farm-client-key = Kunci farm
# A checkbox beside the masked farm key: show it in clear.
farm-client-show-key = Tampilkan
# A field label (the name this machine goes by), and the label of the status row about this machine.
farm-client-machine = Mesin ini
farm-client-machine-hover = Nama yang ditampilkan pengendali untuk mesin ini.
farm-client-share = Drive bersama
# Greyed hint inside the empty field; the path is an example.
farm-client-share-hint = opsional — mis. \\fileserver\share
farm-client-share-hover = Jalur mesin ini ke drive bersama yang digunakan pengendali (mode berbagi): bingkai ditulis ke sana alih-alih dikirim melalui koneksi. Biarkan kosong untuk mengirim setiap bingkai.
farm-client-gpu = Kartu grafis
# The label of the row with the idle option.
farm-client-when = Kapan
# A checkbox followed by a number of minutes ("Only when idle for [5] min").
farm-client-when-idle = Hanya saat menganggur selama
farm-client-when-idle-hover = Ambil pekerjaan hanya setelah tidak ada yang menyentuh papan ketik atau mouse mesin ini selama waktu tersebut, atau saat layarnya terkunci. Saat seseorang mulai menggunakannya, bingkai yang sedang dikerjakan langsung berhenti dan dikembalikan ke farm.
# A collapsible section: the largest job this machine accepts.
farm-client-limits = Batas
farm-client-limits-note = Pekerjaan yang meminta lebih dari batas ini akan ditolak, tidak pernah dikurangi.
farm-client-limit-size = Bingkai terbesar
farm-client-limit-ss = Supersampling terbanyak
farm-client-limit-iter = Iterasi terbanyak

## Render client: the graphics-card choice

farm-client-gpu-system = Pilihan sistem
farm-client-gpu-every = Semua kartu grafis
# $count: this machine's graphics cards.
farm-client-gpu-every-n = Semua kartu grafis ({ $count })
# $count: this machine's graphics cards.
farm-client-gpu-every-each = Semua kartu grafis ({ $count }) — satu sesi per kartu
# A card the window cannot name. $number: its number (or numbers) as --list-adapters gives it.
farm-client-gpu-adapter = Adapter { $number }
farm-client-gpu-finding = Mencari kartu grafis mesin ini…
# "NAME · GPU 1" is how such a machine is named in the controller's list: keep it as it is.
farm-client-gpu-hover = Kartu grafis mana yang merender bingkai farm. Dengan semua kartu, setiap kartu bergabung ke farm sebagai mesin tersendiri — “NAME · GPU 1”, “NAME · GPU 2” — dan Jeda serta Putuskan berlaku untuk semuanya.

## Render client: looking for controllers on the network (the Find button)

farm-client-finding = Mencari pengendali di jaringan ini…
# $port: the UDP port number.
farm-client-found-none = Tidak ada pengendali yang menjawab. Pengendali menjawab selama farm-nya sedang mendengarkan; firewall-nya harus mengizinkan UDP { $port } masuk.
farm-client-found-pick = Di jaringan ini — klik salah satu untuk menggunakannya:
# $identity: the controller's identity fingerprint.
farm-client-found-hover = Identitas { $identity } — gunakan pengendali ini (kunci farm-nya tetap diperlukan)
# After a warning sign. $version and $git: this app's version and build.
farm-client-found-other-build = Build yang berbeda dari build ini ({ $version } { $git }): pengendali itu akan menolak mesin ini.

## Render client: what is wrong with what was typed, or with starting the client

farm-client-bad-address = Masukkan alamat pengendali sebagai HOST:PORT, seperti yang ditampilkan jendelanya (misalnya 192.168.1.20:46733).
# $why: what is wrong with the key (in English).
farm-client-bad-key = Kunci farm: { $why }
# $why: what is wrong with the name (in English).
farm-client-bad-name = Nama: { $why }
farm-client-no-config = Tidak ada folder konfigurasi untuk menyimpan kunci farm.
# $why: the system's reason.
farm-client-start-failed = Tidak dapat memulai klien render: { $why }

## Render client: the state, in the status heading

farm-client-phase-connecting = Menyambungkan…
farm-client-phase-checking = Menjalankan pemeriksaan mandiri…
farm-client-phase-idle = Tersambung — menunggu pekerjaan
farm-client-phase-rendering = Merender
farm-client-phase-in-use = Menunggu — seseorang sedang menggunakan mesin ini
farm-client-phase-paused = Dijeda
farm-client-phase-retrying = Pengendali tidak dapat dijangkau — mencoba lagi
# $seconds: the wait before the next try.
farm-client-phase-retrying-in = Pengendali tidak dapat dijangkau — mencoba lagi dalam { $seconds } s
farm-client-phase-disconnected = Terputus
farm-client-phase-ended = Berakhir

## Render client: the line under the status heading

# $address: the controller's address (HOST:PORT).
farm-client-detail-connecting = Menyambungkan ke { $address }…
# $controller: the controller's name.
farm-client-detail-checking = Tersambung ke “{ $controller }” — menjalankan pemeriksaan mandiri
# $controller: the controller's name.
farm-client-detail-approval = Tersambung ke “{ $controller }” — menunggu penggunanya menyetujui mesin ini
# $controller: the controller's name.
farm-client-detail-idle = Tersambung ke “{ $controller }” — menganggur
# $why: the check that failed and why (in English).
farm-client-detail-check-failed = Pemeriksaan mandiri gagal — { $why }
# $why: the controller's reason (in English).
farm-client-detail-refused = Ditolak oleh pengendali: { $why }
# $why: the controller's reason (in English).
farm-client-detail-closed = Pengendali menutup koneksi: { $why }
# $why: the controller's reason (in English).
farm-client-detail-ended = Pengendali mengakhiri koneksi: { $why }
# $job: the job's name.
farm-client-detail-bundle = Pekerjaan “{ $job }” — menerima paketnya
# $job: the job's name.
farm-client-detail-job-waiting = Pekerjaan “{ $job }” — menunggu bingkai untuk dirender
# $why: the limit the job is over (in English).
farm-client-detail-job-refused = Pekerjaan ini ditolak: { $why }
farm-client-detail-job-over = Pekerjaan sudah selesai
# $first and $last: frame numbers.
farm-client-detail-rendering = Merender bingkai { $first }–{ $last }
# $time: how long nobody must use the machine ("90 s", "5 min").
farm-client-detail-in-use = Menunggu — seseorang sedang menggunakan mesin ini; pekerjaan berlanjut setelah tidak ada yang menggunakannya selama { $time }
farm-client-detail-paused = Dijeda — pengendali sudah diberi tahu
farm-client-detail-pausing = Menjeda — menyelesaikan bingkai yang sedang dikerjakan
farm-client-detail-cancelled = Bingkai yang sedang dikerjakan dibatalkan; dijeda
# A machine with several graphics cards. $rendering: the cards rendering now; $total: all its cards (two or more).
farm-client-rendering-on-cards = Merender di { $rendering } dari { $total } kartu grafis

## Render client: the status rows

# The value of the Controller row. $name: the controller's name; $address: HOST:PORT; $identity: its identity fingerprint.
farm-client-row-controller-value = “{ $name }” · { $address } · identitas { $identity }
# The value of the This machine row. $name: this machine's name; $identity: its identity fingerprint.
farm-client-row-machine-value = “{ $name }” · identitas { $identity }
farm-client-row-gpu = GPU
# One graphics card's row. $slot: the card's number.
farm-client-row-gpu-n = GPU { $slot }
farm-client-row-job = Pekerjaan
# The frames handed to this machine.
farm-client-row-run = Rangkaian
farm-client-row-sent = Terkirim
# The Run row, in parts joined with " · ": the run, the frame in progress, its time. $first and $last: frame numbers.
farm-client-run-frames = bingkai { $first }–{ $last }
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-client-run-frame = bingkai { $frame } ({ $index } dari { $count })
# $time: the time spent on it so far ("2.1 s").
farm-client-run-frame-time = { $time } untuk bingkai ini
# One graphics card's run. $frame: the frame in progress; $first and $last: the run's frames.
farm-client-session-frame = bingkai { $frame } dari { $first }–{ $last }
farm-client-session-no-card = kartu grafis belum diketahui
# The Sent row. $count: frames sent; $time: the mean time per frame ("2.1 s").
farm-client-sent = { $count } bingkai · { $time } per bingkai
# The Sent row of a machine with several cards. $count: frames sent; $split: each card's share ("812 + 422"); $time: the mean time per frame.
farm-client-sent-split = { $count } bingkai ({ $split }) · { $time } per bingkai
# After a warning sign. $note: how the GPUs differ, and both GPUs (in English).
farm-client-gpu-note = Mesin ini memiliki { $note }. Bingkainya akan sedikit berbeda dari bingkai pengendali, dan hal itu dapat terlihat pada adegan yang ditahan.
# A collapsible section: the self-check's results.
farm-client-self-check = Pemeriksaan mandiri
farm-client-thumb-hover = Bingkai terakhir yang dikirim mesin ini
farm-client-resume = Lanjutkan
farm-client-resume-hover = Ambil pekerjaan lagi
farm-client-pause = Jeda
farm-client-pause-hover = Selesaikan bingkai yang sedang dikerjakan, lalu jangan ambil pekerjaan lagi. Pengendali menyerahkan bingkai mesin ini ke mesin lain.
farm-client-cancel-frame = Batalkan bingkai
farm-client-cancel-frame-hover = Hentikan bingkai yang sedang dikerjakan sekarang dan jeda; pengendali merendernya di tempat lain.
farm-client-live-paused = Tampilan langsung dijeda selama mesin ini merender bingkai farm.
farm-client-close = Tutup
farm-client-close-active-hover = Tutup jendela; mesin ini tetap berada di farm (buka lagi dari menu Berkas)
farm-client-close-hover = Tutup jendela
farm-client-disconnect = Putuskan
farm-client-disconnect-hover = Keluar dari farm: bingkai yang sedang dikerjakan dikembalikan
farm-client-connect = Sambungkan
farm-client-connect-hover = Sambungkan ke pengendali dan render bingkai untuknya

## The Render on farm window (Tools ▸ Render on farm…): the controller of a render farm

farm-ctl-title = Render di farm
farm-ctl-out = Folder keluaran
farm-ctl-browse = Telusuri…
farm-ctl-out-note = Bingkai, dan status pekerjaan di farm/: jalankan pekerjaan yang sama lagi untuk melanjutkannya.
farm-ctl-size = Ukuran
farm-ctl-size-custom = Kustom…
# The size list's choice when the size is typed in. $width and $height: pixels.
farm-ctl-size-custom-value = Kustom — { $width }×{ $height }
farm-ctl-fps = Bingkai / detik
farm-ctl-ss = Supersampling
farm-ctl-port = Dengarkan di port
farm-ctl-port-hover = Klien menyambung ke port ini. Izinkan port ini melewati firewall untuk jaringan lokal Anda.
farm-ctl-share = Drive bersama
# Greyed hint inside the empty field.
farm-ctl-share-hint = opsional — jalur mesin ini ke drive itu
farm-ctl-share-hover = Mode berbagi: klien yang memiliki drive yang sama menulis bingkainya ke drive itu alih-alih mengirimkannya melalui koneksi, dan mesin ini memeriksa setiap bingkai di sana. Klien yang tidak memilikinya mengirimkan bingkainya.
# A label followed by a number of clients and two checkboxes.
farm-ctl-start-with = Mulai dengan
# The unit after the number in the box ("Start with [3] clients"). $count: that number.
farm-ctl-start-clients = klien
farm-ctl-start-clients-hover = Pekerjaan dimulai setelah sejumlah mesin ini tersambung dan lulus pemeriksaan mandirinya; mesin lain dapat bergabung kemudian.
farm-ctl-local = Render juga di mesin ini
farm-ctl-local-hover = Mesin ini dihitung sebagai satu klien.
farm-ctl-sharing = Bagikan referensi
farm-ctl-sharing-hover = Orbit referensi bingkai dalam dapat memakan waktu beberapa menit hingga berjam-jam untuk dibangun. Jika dibagikan, orbit itu dibangun sekali lalu diteruskan ke mesin lain. Referensi yang dibagikan adalah referensi yang berbeda tetapi sama sahnya: pada kedalaman ekstrem, segelintir piksel dapat berbeda dari bingkai yang membangun referensinya sendiri.
farm-ctl-key = Kunci farm
# A checkbox beside the shortened farm key: show all of it.
farm-ctl-show-key = Tampilkan
farm-ctl-copy-key = Salin kunci
farm-ctl-copy-key-hover = Tempelkan ke jendela Klien render pada setiap klien
# In place of the farm key, when there is none.
farm-ctl-no-key = belum ada
farm-ctl-make-key = Buat kunci
farm-ctl-make-key-hover = Klien memerlukannya untuk bergabung; kunci juga dibuat saat Anda menekan Render
# What to do on the other machines. $address: this controller's HOST:PORT.
farm-ctl-steps = Pada setiap klien: Berkas ▸ Klien render…, pengendali { $address }, dan kunci farm.
# The same when this machine's address is not known. $port: the port number.
farm-ctl-steps-port = Pada setiap klien: Berkas ▸ Klien render…, alamat mesin ini dan port { $port }, dan kunci farm.
# The size of the job. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
farm-ctl-summary = { $count } bingkai · tur { $seconds }s · { $width }×{ $height } ss{ $ss }
farm-ctl-strip-hover = Setiap bingkai pekerjaan, dari kiri ke kanan: selesai, sedang dirender, belum dikerjakan, ditinggalkan.
farm-ctl-no-tour = Tidak ada tur yang dimuat.
# $folder: the folder; $why: the system's reason.
farm-ctl-out-error = Folder keluaran { $folder }: { $why }
# $why: the system's reason.
farm-ctl-start-failed = Tidak dapat memulai farm: { $why }
# $code: the controller process's exit code, as the code gives it.
farm-ctl-ended-exit = Pengendali berakhir (kode keluar { $code }).

## Render on farm: the state, in the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-phase-waiting = Menunggu klien render ({ $connected } dari { $needed } tersambung)
farm-ctl-phase-preparing = Menyiapkan pekerjaan
farm-ctl-phase-rendering = Merender
farm-ctl-phase-paused = Dijeda
farm-ctl-phase-complete = Selesai
farm-ctl-phase-stopped = Dihentikan
farm-ctl-phase-ended = Berakhir

## Render on farm: the words after the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-detail-waiting = Menunggu klien render: { $connected } dari { $needed } tersambung
farm-ctl-detail-preparing = Mengukur jangkar normalisasi di mesin ini, sekali untuk semua klien
farm-ctl-detail-no-space = Menunggu ruang kosong di folder keluaran
farm-ctl-detail-paused = Dijeda: tidak ada pekerjaan baru yang dibagikan
farm-ctl-detail-not-rendered = Selesai dengan sejumlah bingkai tidak dirender — jalankan lagi pada folder yang sama untuk melanjutkan
farm-ctl-detail-stopped = Dihentikan — jalankan lagi pada folder yang sama untuk melanjutkan
# $code: the controller process's exit code.
farm-ctl-detail-ended = Berakhir (kode keluar { $code })
farm-ctl-detail-finishing = Menyelesaikan

## Render on farm: progress

# On the progress bar. $done: frames done; $total: frames in the job.
farm-ctl-progress = { $done } / { $total } bingkai
# Added to the progress bar. $rate: frames per second, with decimals.
farm-ctl-progress-rate = { $rate } bingkai/s
# Added to the progress bar. $time: the time still to go ("38m 10s").
farm-ctl-progress-eta = sekitar { $time } lagi
# Facts under the progress bar, joined with " · ". $received and $sent: kilobytes per second.
farm-ctl-io = masuk { $received } KB/s · keluar { $sent } KB/s
# $space: free space on the output drive ("412 GB").
farm-ctl-free = { $space } tersedia
# $time: the time since the job started ("38m 10s").
farm-ctl-elapsed = { $time } berlalu
# $count: frames given up on.
farm-ctl-failed = { $count } tidak dirender
# Reference orbits. $built: built for this job; $cached: found in the cache; $reused: reused between frames; $shared: passed between machines.
farm-ctl-refs = referensi: { $built } dibangun, { $cached } dari cache, { $reused } digunakan ulang · { $shared } dibagikan
# After a warning sign.
farm-ctl-storage-low = Folder keluaran hampir penuh: tidak ada pekerjaan baru yang dibagikan sampai ada ruang.
# After a warning sign.
farm-ctl-gpus-differ = GPU di farm ini berbeda-beda. Bingkainya sedikit berbeda, dan hal itu dapat terlihat pada adegan yang ditahan.
# After a warning sign. $count: the GPU classes in the farm.
farm-ctl-gpus-differ-classes = GPU di farm ini berbeda-beda ({ $count } kelas GPU: GPU, driver, atau render uji yang berbeda). Bingkainya sedikit berbeda, dan hal itu dapat terlihat pada adegan yang ditahan.
# $gpu: this machine's GPU, API and driver.
farm-ctl-this-gpu = Mesin ini: { $gpu }
# A machine whose GPU differs. $name: the machine; $note: how its GPU differs (in English); $adapter: its GPU.
farm-ctl-gpu-row = { $name }: { $note } — { $adapter }
# The same with its driver. $driver: the driver's name and version.
farm-ctl-gpu-row-driver = { $name }: { $note } — { $adapter }, driver { $driver }

## Render on farm: the table of machines

# Column headers (short: the table is narrow).
farm-ctl-col-machine = Mesin
farm-ctl-col-state = Status
farm-ctl-col-gpu = GPU
farm-ctl-col-frames = Bingkai
farm-ctl-col-per-frame = Per bingkai
# How the machine delivers its frames (a speed, or "shared drive").
farm-ctl-col-link = Koneksi
# When the controller last heard from the machine.
farm-ctl-col-heard = Kontak
# In the machine's tooltip. $count: its frames that failed verification.
farm-ctl-strikes = { $count } bingkai gagal verifikasi
# In the Link column: it delivers its frames through the shared drive.
farm-ctl-link-share = drive bersama
# In the Heard column. $seconds: how long ago.
farm-ctl-heard-ago = { $seconds } s lalu
farm-ctl-readmit = Terima lagi
farm-ctl-readmit-hover = Izinkan mesin ini bergabung lagi saat tersambung kembali
farm-ctl-remove = Keluarkan
farm-ctl-remove-hover = Keluarkan mesin ini dari pekerjaan; bingkainya dialihkan ke mesin lain
# The GPU column's tooltip.
farm-ctl-probe-same = Render ujinya identik dengan render uji mesin ini.
# $count: the pixels that differ.
farm-ctl-probe-differs = Render ujinya berbeda dari render uji mesin ini pada { $count } piksel: bingkainya akan sedikit berbeda dari bingkai mesin ini.
farm-ctl-probe-unknown = Render ujinya belum dibandingkan.
# $driver: the driver's name and version.
farm-ctl-gpu-driver = driver { $driver }
# After a warning sign. $note: how its GPU differs (in English: "a different GPU").
farm-ctl-gpu-note = { $note } dari GPU mesin ini

## Render on farm: a machine's state, in the State column

# $count: its frames that failed verification.
farm-ctl-state-removed-bad = dikeluarkan: { $count } bingkai buruk
farm-ctl-state-removed = dikeluarkan oleh Anda
farm-ctl-state-removing = mengeluarkan — mengumpulkan diagnostiknya
farm-ctl-state-gone = hilang
farm-ctl-state-leaving = keluar
farm-ctl-state-checking = memeriksa (pemeriksaan mandiri)
farm-ctl-state-paused = dijeda oleh penggunanya
farm-ctl-state-parked = diparkir: crash dua kali — bergabung lagi saat tersambung kembali
farm-ctl-state-refused = menolak pekerjaan ini (batasnya sendiri)
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-ctl-state-rendering = merender { $frame } ({ $index }/{ $count })
farm-ctl-state-starting = memulai rangkaian
farm-ctl-state-idle = menganggur
farm-ctl-state-admitted = diterima — menunggu pekerjaan

## Render on farm: the buttons

# A collapsible section: the controller's messages.
farm-ctl-events = Peristiwa
farm-ctl-resume = Lanjutkan
farm-ctl-pause = Jeda
farm-ctl-pause-hover = Jangan bagikan pekerjaan baru; bingkai yang sedang dikerjakan tetap diselesaikan
farm-ctl-live-paused = Tampilan langsung dijeda selama mesin ini merender untuk farm.
farm-ctl-close = Tutup
farm-ctl-close-active-hover = Tutup jendela; pekerjaan tetap berjalan (buka lagi dari menu Alat)
farm-ctl-close-hover = Tutup jendela
farm-ctl-copy-command = Salin perintah
farm-ctl-copy-command-hover = Salin baris perintah yang setara
farm-ctl-stop = Hentikan
# The Stop button once a stop has been asked for and the job has not ended after 10 seconds.
farm-ctl-stop-now = Hentikan sekarang
farm-ctl-stop-hover = Akhiri pekerjaan dengan bingkai yang sudah ada; menjalankan pekerjaan yang sama lagi akan melanjutkannya
farm-ctl-render = Render
farm-ctl-render-hover = Mulai mendengarkan klien dan render tur di klien-klien itu

## The Render tour window (Tools ▸ Render tour…)

tour-render-title = Render tur
tour-render-out = Folder keluaran
tour-render-browse = Telusuri…
tour-render-out-hover = Jalur lengkap tempat bingkai ditulis.
tour-render-prefix = Awalan bingkai
# <prefix> stands for the prefix typed in the field.
tour-render-prefix-hover = Bingkai ditulis sebagai <awalan>_00000.png
tour-render-size = Ukuran
tour-render-size-custom = Kustom…
# The size list's choice when the size is typed in. $width and $height: pixels.
tour-render-size-custom-value = Kustom — { $width }×{ $height }
tour-render-fps = Bingkai / detik
tour-render-fps-hover = Di bawah 1 fps itu sah: mengambil sampel penyelaman panjang pada 0,25 fps adalah cara memeriksa penahanan regresi dalam tanpa merender ribuan bingkai.
tour-render-ss = Supersampling
tour-render-chapter = Bab
tour-render-whole = Seluruh tur
tour-render-chapter-hover = Merender satu bab tetap mempertahankan penomoran bingkai GLOBAL, sehingga bingkainya dapat langsung dimasukkan kembali ke urutan lengkap.
tour-render-mp4 = Rakit mp4
tour-render-mp4-hover = Menjalankan ffmpeg setelahnya (harus ada di PATH). Bingkai PNG tetap disimpan.
tour-render-overwrite = Timpa bingkai yang ada
tour-render-overwrite-hover = Nonaktif: render berhenti untuk bertanya, dan itu tidak dapat dilakukan dari sini.
tour-render-resume = Lanjutkan render yang terputus
tour-render-resume-hover = Pertahankan bingkai yang sudah ada di folder keluaran dan render hanya yang belum ada. Bingkai terbaru diperiksa terlebih dahulu dan dibuang jika tidak lengkap (bingkai tempat render terhenti adalah bingkai yang sedang ditulisnya), mundur terus sampai ditemukan bingkai yang baik; folder yang berisi bingkai dengan ukuran berbeda ditolak alih-alih dicampur ke dalam urutan ini.
# $count: this machine's graphics cards.
tour-render-all-gpus = Gunakan semua kartu grafis ({ $count })
tour-render-all-gpus-hover = Render di semua kartu grafis mesin ini sekaligus, melalui render farm di komputer ini (--gpus all). Bingkai, namanya, Lanjutkan, dan mp4 sama seperti untuk satu kartu; seluruh tur dirender, dalam urutan farm itu sendiri. Model kartu yang berbeda menggambar beberapa piksel sedikit berbeda: adegan yang ditahan dan transisi larut tetap di satu kartu.
tour-render-order = Urutan render
tour-render-sequential = Berurutan
tour-render-sequential-hover = Bingkai 0, 1, 2, … secara berurutan (bawaan).
tour-render-progressive = Progresif
tour-render-progressive-hover = Pratinjau seluruh tur terlebih dahulu, lalu sempurnakan hingga detail penuh: bingkai kunci dirender lebih dulu, lalu celah waktu terbesar dibelah dua sampai setiap bingkai ada. Bingkai tetap mendarat pada indeks yang benar dengan cara mana pun, sehingga Lanjutkan dan mp4 bekerja tanpa perubahan — Anda dapat berhenti lebih awal, melihat sekilas flip-book kasarnya, lalu memulai lagi dengan Lanjutkan dalam urutan mana pun.
# The cost of the render. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
tour-render-summary = { $count } bingkai · tur { $seconds }s · { $width }×{ $height } ss{ $ss }
# The disk space the frames will take (an estimate). $size: a size ("5.3 GB").
tour-render-disk = ≈{ $size } bingkai
# $size: the frames' estimated size; $free: the free space on that drive ("6.0 GB").
tour-render-disk-free = ≈{ $size } bingkai · { $free } tersedia
# After a warning sign. $disk: the line above ("≈51.0 GB of frames · 6.0 GB free").
tour-render-disk-short = { $disk } — ruang tidak cukup untuk seluruh urutan
tour-render-separate = Merender skrip DARI DISK dalam proses terpisah, sehingga suntingan yang belum disimpan tidak ikut — dan perangkat GPU yang hilang di sana tidak dapat ikut menjatuhkan sesi ini.
# On the progress bar. $done: frames done; $total: frames to render.
tour-render-progress = { $done } / { $total } bingkai
# Before the render's first progress line.
tour-render-starting = memulai…
tour-render-live-paused = Tampilan langsung dijeda — render memakai GPU sendirian, sehingga lebih cepat selesai dan mengukur biaya bingkainya sendiri dengan akurat.
tour-render-cancel = Batal
tour-render-cancel-hover = Tutup tanpa merender
tour-render-copy-command = Salin perintah
tour-render-copy-command-hover = Salin baris perintah yang setara ke papan klip
tour-render-stop = Hentikan render
tour-render-render = Render
tour-render-render-hover = Mulai merender urutan bingkai
# $folder: the output folder.
tour-render-finished = Render selesai → { $folder }
# $why: the render's own reason (in English).
tour-render-failed = Render gagal: { $why }
# $last: the render's last progress line (in English), or nothing.
tour-render-failed-or-stopped = Render gagal atau dihentikan. { $last }
# $why: the system's reason.
tour-render-no-exe = Tidak dapat menemukan berkas program: { $why }
# $why: the system's reason.
tour-render-start-failed = Tidak dapat memulai render: { $why }

## The Custom formula dialog (Fractal ▸ Custom formula…)
## The formula language itself stays as it is in every language: function names (sin, cosh, conj…),
## operators, variables (z, c, p1), and the formulas. Keep them as written inside these messages.

# The window's title.
formula-dialog-title = Rumus kustom
# A small line at the top of the window, above the formula.
formula-dialog-intro = Langkahnya, dalam ekspresi gaya Fractint
# The two ways to show and edit the formula (top right of the window).
formula-dialog-textbook = Buku teks
formula-dialog-textbook-hover = Tampilkan rumus dengan tata letak seperti di buku teks, dan sunting dalam bentuk itu
formula-dialog-text = Teks
formula-dialog-text-hover = Sunting rumus sebagai teks
# The tooltip of the "LaTeX" button (the button's own text stays LaTeX).
formula-dialog-latex-hover = Salin rumus sebagai LaTeX, sesuai tata letaknya di sini
# The drop-down list of example formulas (the collection).
formula-dialog-examples = Contoh
# Help under the formula in Textbook mode. The characters / ^ ( are what the user types: keep them.
formula-dialog-help-textbook = Ketik seperti pada teks: / membuat pecahan dari suku sebelumnya, ^ membuat eksponen, ( membuat tanda kurung. Tombol panah bergerak di sepanjang rumus; Tab menuju kotak kosong berikutnya. Baris berwarna merah tidak dapat dibaca: klik untuk menyuntingnya sebagai teks.
# Help under the formula in Text mode. "; starts a comment": the ; is what the user types.
formula-dialog-help-text = Ketik, atau gunakan papan tombol — papan itu memuat setiap nama yang dikenal bahasa rumus. Nama dilengkapi saat Anda mengetik (Tab). Pernyataan dipisahkan dengan baris baru atau koma; ; memulai komentar.
# The syntax check, when the formula reads.
formula-dialog-reads = Terbaca dengan benar.
# The syntax check in Textbook mode, when the formula has an empty box to fill in.
formula-dialog-empty-box = Isi kotak yang kosong.
# The syntax check in Textbook mode. $line: the line number; $message: what is wrong (from the formula parser, in English).
formula-dialog-error-line = Baris { $line }: { $message }
# One function written with another notation's name, and the language's name for it: "ln to log".
# $from: the name as written (ln); $to: the formula language's name (log).
formula-respell-pair = { $from } menjadi { $to }
# A button that rewrites them. $changes: one or more formula-respell-pair, joined with ", ".
formula-respell = Ubah { $changes }
formula-respell-hover = Tulis fungsi-fungsi ini sesuai nama dalam bahasa rumus
# Shown when Apply finds a parameter field that is not a number. $name: the parameter (p1 … p5).
formula-param-not-numbers = { $name } bukan pasangan bilangan
# The Apply button (also in the Formula library window).
formula-apply = Terapkan
formula-dialog-apply-hover = Kompilasi rumus dan tampilkan
# Shown beside Apply while the formula is being compiled.
formula-dialog-compiling = Mengompilasi rumus untuk GPU…
# The greyed hint in an empty name box (also in the Formula library window).
formula-name-hint = nama (opsional)
# The save button: "Update" when the library already has a formula of that name.
formula-dialog-update-in-library = Perbarui di pustaka
formula-dialog-save-to-library = Simpan ke pustaka
formula-dialog-save-hover = Simpan rumus ini beserta parameternya di pustaka rumus
formula-dialog-save-disabled-hover = Hanya rumus yang terbaca dengan benar yang dapat disimpan
# Opens the Formula library window.
formula-dialog-library = Pustaka…
formula-dialog-library-hover = Rumus yang tersimpan: terapkan, sunting, impor, dan ekspor

## The Custom formula dialog's keypad
## The keys' captions are formula tokens and stay as they are; these are their tooltips.

# The keypad's third tab (the other two are "123" and "f(z)"). A short word.
formula-keypad-tab-names = nama
# The tabs' tooltips.
formula-keypad-tab-basic-hint = Bilangan, operator, dan nama yang paling sering dipakai
formula-keypad-tab-functions-hint = Fungsi
formula-keypad-tab-names-hint = Setiap variabel, parameter, dan konstanta, serta sintaksis pernyataan
# The key z.
formula-key-z = Nilai iterasi
# The key c.
formula-key-c = Piksel (konstanta Julia dalam mode Julia)
# The key p1 on the first tab.
formula-key-p1-below = Parameter 1 — diatur di kolom di bawah
# The keys p1 … p5. $n: the parameter's number.
formula-key-param = Parameter { $n }
formula-key-multiply = Kali
formula-key-divide = Bagi
# The key that types ^2.
formula-key-square-after = Kuadrat (setelah sebuah nilai: z^2)
# The key that types ^.
formula-key-power = Pangkat: eksponen bilangan bulat, real, atau kompleks
formula-key-square-root = Akar kuadrat
# The key that types |…|: in this language it is the squared modulus, as in Fractint.
formula-key-squared-modulus = Modulus KUADRAT |z|² — seperti di Fractint
formula-key-add = Tambah
formula-key-subtract = Kurang (atau negasi)
# The key that types =.
formula-key-assign = Penugasan: z = …, atau variabel sementara t = …
# The key that starts a new line (first tab).
formula-key-next-statement-comma = Pernyataan berikutnya (koma juga bisa)
# The key "del".
formula-key-backspace = Hapus karakter sebelum kursor (atau pilihan)
# The key e.
formula-key-euler = Bilangan Euler
# The key that types (0.5, 0.5).
formula-key-complex-constant = Konstanta kompleks: (real, imajiner)
# The key that types a comma.
formula-key-comma = Memisahkan pernyataan, atau bagian-bagian konstanta kompleks
formula-key-left = Kursor ke kiri
formula-key-right = Kursor ke kanan
# The function keys (the f(z) tab), one per function.
formula-key-sin = Sinus
formula-key-cos = Kosinus
formula-key-tan = Tangen
formula-key-cotan = Kotangen
formula-key-exp = Eksponensial e^z
formula-key-sinh = Sinus hiperbolik
formula-key-cosh = Kosinus hiperbolik
formula-key-tanh = Tangen hiperbolik
formula-key-cotanh = Kotangen hiperbolik
formula-key-log = Logaritma natural, ln (cabang utama)
formula-key-sqr = Kuadrat: sqr(z) = z²
formula-key-sqrt = Akar kuadrat (utama)
formula-key-recip = Kebalikan 1/z
formula-key-conj = Konjugat kompleks
formula-key-flip = Tukar bagian real dan imajiner
formula-key-abs = Nilai mutlak KEDUA bagian: |x| + i|y|
formula-key-cabs = Modulus |z| (bilangan real)
formula-key-real = Bagian real
formula-key-imag = Bagian imajiner, sebagai bilangan real
# "fn" is Fractint's name for a function slot: keep it.
formula-key-ident = Identitas (placeholder fn milik Fractint)
# The key "pixel".
formula-key-pixel = Sama dengan c (nama dari Fractint)
# The key "z =".
formula-key-set-z = Mulai pernyataan yang menetapkan z
# The key "t =".
formula-key-temporary = Variabel sementara: nama baru apa pun bisa (t, w, a2, …)
# The key "; …".
formula-key-comment = Komentar, sampai akhir baris
# The key that starts a new line (names tab).
formula-key-next-statement = Pernyataan berikutnya

## The formula's completion list (names offered as you type)

# Beside a name the formula itself assigns.
formula-completion-variable = Variabel yang ditetapkan rumus ini
# Beside a function typed with another notation's name.
# $what: what the function is, in English (from the formula parser: "The natural logarithm");
# $other: the other notation's name (ln); $ours: the formula language's name (log).
formula-completion-other-spelling = { $what } — { $other } ditulis { $ours } di sini
# Under a list cut short. $count: the names not shown.
formula-completion-more = … { $count } lainnya
# The last line of the list. Tab, Enter and Esc are key names.
formula-completion-keys = Tab atau Enter untuk melengkapi, Esc untuk menutup

## The Custom formula dialog's Textbook mode

# The tooltip of a line that does not read. $why: what is wrong (from the formula parser, in English, or formula-textbook-fractint-sections).
formula-textbook-unread-hover = Tidak dapat dibaca: { $why }. Klik untuk menyuntingnya sebagai teks.
# Why a line with Fractint's sections is shown as text in Textbook mode.
formula-textbook-fractint-sections = Bagian-bagian Fractint (':' pada init, perbandingan, a = b = …) disunting sebagai teks

## The Formula library window (Fractal ▸ Formula library…)

# The window's title.
formula-library-title = Pustaka rumus
# The window's two tabs. $count: the formulas in each.
formula-library-mine = Rumus saya ({ $count })
formula-library-mine-hover = Rumus yang Anda simpan atau impor
formula-library-collection = Koleksi ({ $count })
formula-library-collection-hover = Rumus yang disertakan bersama Fractadyne, masing-masing dengan tampilan pembuka
# A small marker before the name of the formula the view shows now. A short word.
formula-library-showing = tampil
# The tooltip of Apply on a formula of the collection.
formula-library-apply-shipped-hover = Tampilkan, pada tampilan pembukanya
# A row's Edit button (after a pencil icon).
formula-library-edit = Sunting
formula-library-edit-hover = Buka di dialog Rumus kustom, tanpa menerapkannya
# A collection row's button that copies the formula into "My formulas".
formula-library-copy-to-mine = Salin ke milik saya
formula-library-copy-to-mine-hover = Tambahkan ke rumus Anda, untuk disimpan atau diubah
formula-library-close = Tutup
# Saves the formula the view shows, under the name typed beside it.
formula-library-add-current = Tambahkan rumus saat ini
formula-library-add-current-hover = Simpan rumus yang sedang ditampilkan
# "Fractal > Custom formula…" is the menu path: write it as the menus are named.
formula-library-add-current-disabled-hover = Terapkan rumus kustom terlebih dahulu (Fraktal > Rumus kustom…)
formula-library-import = Impor…
formula-library-import-hover = Tambahkan rumus-rumus dalam sebuah berkas rumus; tidak ada yang diganti di sini
formula-library-export-all = Ekspor semua…
formula-library-export-all-hover = Tulis semua rumus di sini ke satu berkas, untuk dibagikan atau disimpan
# The filter box: its label, and the greyed hint inside it.
formula-library-filter = Filter
formula-library-filter-hint = filter berdasarkan nama atau teks
# Shown while the user has no saved formulas.
formula-library-empty = Belum ada rumus tersimpan. Simpan dari dialog Rumus kustom, tambahkan rumus yang sedang ditampilkan, atau impor sebuah berkas (berkas kami, atau .frm Fractint).
# The tooltip of Apply on one of the user's formulas: with a view saved, and without one.
formula-library-apply-view-hover = Tampilkan rumus ini, pada tampilan yang disimpan bersamanya
formula-library-apply-hover = Tampilkan rumus ini
# The tooltip of a row's save icon.
formula-library-export-one-hover = Ekspor rumus ini ke sebuah berkas
# A row asks before deleting: the question, then its two buttons. "Delete" is also the delete icon's tooltip.
formula-library-delete-ask = Hapus?
formula-library-delete = Hapus
formula-library-keep = Pertahankan
# Under a list cut short. $count: the formulas not listed.
formula-library-more = …dan { $count } lainnya: ketik di filter untuk menemukannya.
formula-library-no-match = Tidak ada yang cocok dengan filter.
# Under a saved formula this version cannot read. $why: what is wrong (from the formula parser, in English).
formula-library-unread = Tidak dapat dibaca di versi ini: { $why }

## Formula library notices

# $why: the system's reason, in English.
formula-library-save-failed = Tidak dapat menyimpan pustaka rumus: { $why }
formula-library-nothing-to-save = Tidak ada yang disimpan: rumusnya kosong.
# $name: the formula's name in the library.
formula-library-updated = “{ $name }” diperbarui di pustaka rumus.
formula-library-saved = “{ $name }” disimpan ke pustaka rumus.
# $name: the formula's name; $why: what is wrong.
formula-library-apply-failed = “{ $name }” tidak dapat diterapkan: { $why }
# Copying a formula of the collection into "My formulas". $name: its name; $as_named: the name it was given (the same, or "Name (2)" when the name was taken).
formula-library-copy-exists = “{ $name }” sudah ada di rumus Anda.
formula-library-copied = “{ $name }” disalin ke rumus Anda sebagai “{ $as_named }”.
# $name: the formula's name.
formula-library-deleted = “{ $name }” dihapus dari pustaka rumus.
# Why the library could not be saved (follows formula-library-save-failed).
formula-library-no-config-dir = tidak ada direktori konfigurasi
# At startup, when the library file cannot be read. $why: the reason, in English; $path: a file's full path.
formula-library-unreadable-kept = Pustaka rumus tidak dapat dibaca ({ $why }). Berkasnya disimpan sebagai { $path } dan pustaka baru yang kosong dimulai.
formula-library-unreadable-stuck = Pustaka rumus tidak dapat dibaca ({ $why }), dan juga tidak dapat dipindahkan: salin { $path } ke tempat yang aman sebelum menyimpan rumus, karena penyimpanan akan menggantinya.

## Formula library: Import and Export
## A formula file's contents stay in English; this is what the windows and notices around them say.

# File-type names in the open and save dialogs (the extensions are added by the system).
formula-filter-formula-files = Berkas rumus
formula-filter-fractadyne = Rumus Fractadyne
formula-filter-fractint = Rumus Fractint
formula-filter-all-files = Semua berkas
# Why a file cannot be imported (they follow formula-import-failed).
formula-import-too-large = terlalu besar untuk menjadi berkas rumus
formula-import-not-text = bukan berkas teks
formula-import-no-formulas = tidak berisi rumus
# $why: the TOML reader's message, in English.
formula-file-not-formulas = bukan berkas rumus ({ $why })
# $format: what the file says it is ("fractadyne-gradients").
formula-file-other-format = berkas “{ $format }”, bukan berkas rumus
# "[[formula]]" is the file's own syntax: keep it.
formula-file-no-formulas = tidak berisi rumus (berkas rumus mencantumkannya sebagai tabel [[formula]])
# $file: the file's name; $why: one of the reasons above, or the system's (in English).
formula-import-failed = Tidak dapat mengimpor “{ $file }”: { $why }.
# After an import that added nothing. $file: the file's name.
formula-import-nothing-new = Tidak ada yang baru di “{ $file }”.
# After an import. $count: the formulas added; $file: the file's name.
formula-import-added = { $count } rumus diimpor dari “{ $file }”.
# The same two, followed by what else happened. $notes: one or more of the formula-import-note-… messages below, joined with "; ".
formula-import-nothing-new-notes = Tidak ada yang baru di “{ $file }”: { $notes }.
formula-import-added-notes = { $count } rumus diimpor dari “{ $file }”: { $notes }.
# Notes (clauses, no final full stop). $count: the formulas already in the library, skipped.
formula-import-note-duplicates = { $count } rumus sudah ada di pustaka
# One formula was renamed because its name was taken. $from: its name in the file; $to: the name it was given ("Name (2)").
formula-import-note-renamed-one = “{ $from }” sudah terpakai, jadi namanya menjadi “{ $to }”
# $count: the formulas renamed because their names were taken (2 or more).
formula-import-note-renamed = { $count } diganti namanya karena namanya sudah terpakai
# $count: the entries skipped as empty or too long.
formula-import-note-skipped = { $count } entri kosong atau terlalu panjang, dilewati
# A second sentence after importing a Fractint .frm file. $count: the entries that do not read; $why: the commonest reason (from the formula parser, in English).
formula-import-frm-unread = { $count } entri tidak dapat dibaca di versi ini (paling sering: { $why }).
# After exporting one formula. $name: the formula's name; $file: the file's name.
formula-export-one = “{ $name }” diekspor ke “{ $file }”.
# After exporting several. $count: how many; $file: the file's name.
formula-export-many = { $count } rumus diekspor ke “{ $file }”.
# $file: the file's name; $why: the system's reason, in English.
formula-export-failed = Tidak dapat menulis “{ $file }”: { $why }

## The control panel (right-hand side): header and section headings

# The panel's title, beside the button that hides it.
panel-title = Kontrol
panel-hide-hover = Sembunyikan panel kontrol
# The button that brings the hidden panel back (top right of the view).
panel-show-hover = Tampilkan panel kontrol
# Section headings.
panel-section-navigate = Navigasi
panel-section-coloring = Pewarnaan
panel-section-quality = Kualitas
panel-section-effects = Efek
panel-section-overlays = Lapisan
panel-section-advanced = Lanjutan
# The last section: reference notes about the current fractal. $name: the fractal's name (Mandelbrot, Burning Ship…).
panel-section-about = Tentang { $name }
panel-section-performance = Kinerja

## The control panel: Navigate
## Labels before a control sit in a column about 104 px wide (roughly 15 characters): a longer label pushes its control to the right.

panel-zoom-speed = Kecepatan zoom
panel-zoom-speed-hover = Kecepatan zoom berkelanjutan dengan menahan Space (1× ≈ 2× per 1,5 s).
panel-snap-zoom = Tepatkan zoom ke fitur
panel-snap-zoom-hover = Saat memperbesar dengan roda mouse atau Space, tarik zoom ke arah minibrot (atau sumbu real) di dekat kursor, sehingga fitur itu bergeser ke bawah kursor dan zoom mendarat di atasnya. Makin dekat fiturnya, makin kuat tarikannya. Hanya Mandelbrot, tampilan tunggal.
panel-click-zoom = Klik untuk zoom
panel-click-zoom-hover = Jika aktif, klik kiri di tampilan akan menyelam ke dalam dengan faktor di bawah (klik kanan mundur), berpusat ulang pada titik yang diklik. Seret tetap menggeser; Shift+seret / seret kanan tetap melakukan zoom kotak. Backspace mengurungkan satu klik. Dalam tampilan ganda, fitur ini bekerja di kedua panel, dan penguncian Julia c berpindah ke Ctrl+klik selama fitur ini aktif. Tahan Shift untuk memunculkan kaca pembesar di bawah kursor yang menunjukkan tampilan yang akan Anda capai dengan klik itu, dan klik sambil menahannya untuk mengambil titik itu; Shift+SERET tetap melakukan zoom kotak.
# The click-to-zoom factor, followed by buttons 2×, 4×, 10×…
panel-click-zoom-factor = Faktor
panel-click-zoom-snap = Tepatkan ke pusat terdekat
panel-click-zoom-snap-hover = Setelah lompatan, hitung nukleus minibrot terdekat dan tepatkan tampilan persis di atasnya, sehingga Anda mendarat pada fitur itu, bukan sekadar di dekatnya. Zoom tidak diubah — klik itulah yang memilihnya. Hanya Mandelbrot. Perhitungan berjalan di latar belakang: tampilan langsung melompat dan mengoreksi dirinya sendiri saat jawabannya tiba, yang pada kedalaman ekstrem dapat memakan beberapa detik.

## The control panel: Coloring
## Labels before a control sit in a column about 104 px wide (roughly 15 characters).

panel-method = Metode
panel-method-hover = Cara data lolos dipetakan ke warna. Rata-rata garis / ketaksamaan segitiga / perangkap orbit / dekomposisi memperlihatkan struktur orbit; estimasi jarak memberi bayangan berdasarkan kedekatan dengan batas.
panel-stripe-density = Kepadatan garis
panel-stripe-tail = Hanya ekor (zoom dalam)
panel-stripe-tail-hover = Rata-ratakan suku garis selama N iterasi terakhir setiap orbit (jendela eksponensial), bukan seluruh orbit. Pada kedalaman ekstrem, setiap orbit membayangi referensi untuk semua kecuali beberapa ratus iterasi terakhirnya, sehingga rata-rata seluruh orbit menghasilkan satu warna untuk seluruh tampilan; ekor mempertahankan kontrasnya. Orbit yang lebih pendek dari jendela tidak terpengaruh. Juga --stripe-tail N untuk --render.
panel-tail-length = Panjang ekor
# The unit after the tail length's number: short for "iterations" (the slider reads "64 it").
panel-tail-length-unit = it
panel-tail-length-hover = Panjang jendela dalam iterasi. Jendela pendek (8–32) memberikan pita tebal klasik pada kedalaman berapa pun; jendela yang lebih panjang kembali merata-rata menjadi warna datar, karena suku garis dari orbit kacau yang panjang saling meniadakan.
panel-trap-shape = Bentuk perangkap
panel-palette = Palet
# The two colours of the Binary palette, each beside its colour swatch.
panel-in-set = Di dalam
panel-out-of-set = Di luar
# The two colours of the Duotone palette, each beside its colour swatch.
panel-shadow = Bayangan
panel-highlight = Sorotan
panel-cycle = Siklus
panel-offset = Pergeseran
panel-normalize-hover = Pada kedalaman ekstrem, jumlah iterasi lolos mencakup ratusan ribu, dan Siklus yang tetap membungkus palet ribuan kali di antara piksel yang bersebelahan — medan padat yang benar tampak seperti derau bintik-bintik. Ini memetakan ulang palet ke rentang lolos terukur dari tampilan (Siklus kemudian mengatur berapa sapuan palet yang mencakupnya) KETIKA tampilan benar-benar mengalami aliasing — tampilan yang halus dibiarkan, karena memetakannya ulang akan mengubah gambar tanpa alasan. Centang “Selalu sesuaikan rentang” di bawah untuk memetakan ulang dalam keadaan apa pun. Hanya metode Iterasi halus; sama dengan --normalize.
# A checkbox indented under "Normalize deep colors" (also in the Color menu).
panel-fit-range = Selalu sesuaikan rentang
panel-fit-range-hover =
    Petakan ulang palet ke rentang lolos terukur dari tampilan bahkan ketika tampilan tidak mengalami aliasing — kotak centang di atas hanya memetakan ulang ketika medan yang padat akan tampak seperti bintik-bintik.

    Gunakan ini jika Anda ingin palet mencakup apa yang ada di layar, bukan untuk mengatasi derau. Fitur ini tetap memerlukan pengukuran, jadi tidak melakukan apa pun selama sesaat sebelum tampilan stabil, dan sama sekali tidak berpengaruh pada tampilan tanpa piksel yang lolos.
# A checkbox indented under "Always fit range" (also in the Color menu).
panel-log-scale = Skala warna log
panel-log-scale-hover =
    Sebarkan palet berdasarkan LOGARITMA nilai lolos, bukan secara linear. Jumlah iterasi lolos menumpuk ke arah ujung atas pada kedalaman, sehingga pemetaan linear menghabiskan sebagian besar palet pada lapisan tipis di dekat batas dan meratakan semua yang lain.

    Berlaku untuk pemetaan yang dinormalisasi — yang diaktifkan oleh “Normalkan warna dalam” (atau --normalize pada ekspor). Rentang yang sangat miring tetap memakai pemetaan log, dicentang atau tidak.
# The same checkbox's tooltip while it is greyed out (normalization is off).
panel-log-scale-off-hover =
    Memerlukan normalisasi: ini memilih BAGAIMANA palet dipetakan ke rentang lolos terukur, jadi jika “Normalkan warna dalam” mati (dan tidak ada ekspor --normalize), tidak ada pemetaan yang dapat diubahnya dan tidak satu piksel pun akan berubah.

    Aktifkan “Normalkan warna dalam” untuk menggunakannya.
# The palette animation: its kind, then its speed.
panel-animate = Animasi
panel-anim-speed = Kecepatan
panel-anim-speed-hover = Kecepatan siklus: siklus pergeseran warna/detik, atau (Acak) perubahan gradien/detik.
panel-shuffle-gradient = Acak gradien

## The control panel: Quality

panel-auto-iter = Skalakan iterasi otomatis sesuai zoom
# The iteration-count label while auto-scaling is on (the count is then a base the zoom raises).
panel-iterations-base = Iterasi (dasar)
panel-iterations = Iterasi
panel-iterations-hover = Jumlah iterasi dasar. Dengan Skalakan otomatis aktif, jumlah efektif naik seiring kedalaman zoom. Saat Anda bergerak, pratinjau membatasi iterasi pada nilai rendah (50.000) agar tetap responsif, lalu menajam ke jumlah penuh saat tampilan stabil — sehingga tepi dalam tampak halus selama bergerak dan menjadi jelas saat Anda berhenti.
# A warning under the iteration count. $count: the iterations the settled preview renders.
panel-res-limited = Pada kedalaman ini, pratinjau yang stabil merender { $count } iterasi pada resolusi yang dikurangi agar tetap responsif. Ekspor untuk mendapatkan detail resolusi penuh.
panel-anti-alias = Anti-alias
# Anti-aliasing off (the other choices are 2×, 3×, 4×, 8×).
panel-anti-alias-off = Mati
panel-anti-alias-hover = Supersampling untuk gambar diam (diterapkan saat tampilan stabil). Nilai lebih tinggi meredam “debu” halus di bagian luar dengan konsekuensi waktu render.

## The control panel: Effects

panel-relief-lighting = Pencahayaan relief 3D
panel-relief-lighting-hover = Beri bayangan pada permukaan menggunakan kemiringan estimasi jarak — tampilan timbul yang tersinari. (Keluarga holomorfik: Mandelbrot / Multibrot.)
panel-light-angle = Sudut cahaya
panel-relief = Relief
panel-relief-hover = Lebih rendah = relief lebih tajam; lebih tinggi = lebih lembut/datar.
panel-rotate-light = Putar cahaya
panel-rotate-light-hover = Putar arah cahaya seiring waktu (menggunakan penggeser Kecepatan).
panel-distance-glow = Pendaran jarak
panel-distance-glow-hover = Pita kontur estimasi jarak yang terang, yang memadat menjadi filamen berpendar di dekat batas. (Keluarga holomorfik.)
panel-glow = Pendaran
panel-band-width = Lebar pita
panel-band-width-hover = Jarak antarkontur jarak (oktaf per pita).
panel-animate-glow = Animasikan pendaran
panel-animate-glow-hover = Alirkan pita pendaran seiring waktu (menggunakan penggeser Kecepatan).

## The control panel: Overlays

panel-orbits-hover = Gambar jalur iterasi titik di bawah kursor.
panel-orbit-anim = Animasi (titik melaju)
panel-orbit-anim-hover = Luncurkan titik berganti warna yang melaju di sepanjang orbit.
panel-orbit-speed = Kecepatan orbit
panel-minimap-hover = Ikhtisar kecil tampilan awal dengan penanda “Anda di sini” dan kedalaman zoom. Klik untuk melompat ke suatu wilayah.

## The control panel: Advanced
## Labels before a control sit in a column about 104 px wide; most of these are already wider in English.

# Small headings of the section's two groups.
panel-accelerators = Akselerator
panel-performance-tuning = Penyetelan kinerja
panel-bla = Akselerasi BLA (zoom dalam)
panel-bla-hover = Aproksimasi bilinear: lompati iterasi di sepanjang orbit pada kedalaman ekstrem (Mandelbrot floatexp, ≥1e28×) — render GPU ~5× lebih cepat, keluaran identik (diverifikasi oleh uji mandiri). Aktif secara bawaan; matikan untuk membandingkan atau jika Anda menemui artefak.
panel-series-approx = Aproksimasi deret
panel-series-approx-hover = Mulai perturbasi dari sebuah polinomial untuk melompati iterasi awal, di tempat BLA tidak aktif (kedalaman df32, Multibrot, BLA mati). Keluaran identik; matikan untuk membandingkan.
panel-glitch-correct = Koreksi glitch (ekspor)
panel-glitch-correct-hover = Koreksi glitch multireferensi untuk gambar yang diekspor: mendeteksi glitch perturbasi dan merender ulang piksel-piksel itu terhadap referensi tambahan sampai bersih. Aktif secara bawaan. Ekspor hingga ~32 MP (pewarnaan non-aux); gambar yang lebih besar dan tampilan langsung menggunakan jalur biasa. Tidak pernah digunakan untuk tampilan Mandelbrot atau Multibrot (di luar mode Julia): saat diperiksa terhadap perhitungan eksak, koreksi ini tidak memperbaiki apa pun di sana dan memakan hingga 5x waktu ekspor.
panel-dive-limit = Batas selam zoom otomatis
panel-dive-limit-hover = Kedalaman tempat zoom otomatis (tombol A) berhenti. Hingga ~1e271× zoom meluncur dengan mulus; lebih dalam lagi, zoom beralih ke penyelaman bertahap yang tersendat untuk mencapai kedalaman ekstrem dengan cepat.
# Followed by two choices, Speed and Quality.
panel-autopilot-priority = Prioritas zoom otomatis
panel-priority-speed-hover = Zoom pada kecepatan yang diatur. Layar disegarkan begitu bingkai yang memiliki detail siap, pada resolusi berapa pun yang membuat penyegaran terus datang (di bawah batas bawah Resolusi gerak min jika zoom melampaui render); di antara penyegaran, bingkai terakhir mengikuti zoom dan menjadi lembut.
panel-priority-quality-hover = Setiap penyegaran adalah bingkai beresolusi penuh yang terselesaikan sepenuhnya, dan zoom melambat bila perlu agar bingkai di layar tidak pernah diperbesar terlalu jauh sebelum bingkai berikutnya tiba. Penyelaman yang lebih lambat tetapi tajam di setiap saat.
# Followed by two choices, Detail and Misiurewicz point.
panel-autopilot-target = Target zoom otomatis
panel-target-detail-hover = Menyelam ke struktur paling kaya di tepi fraktal dalam tampilan, sambil memilih ulang targetnya seiring penyelaman berlangsung.
panel-target-misiurewicz-hover = Temukan titik Misiurewicz terdekat (pusat spiral yang strukturnya berulang selamanya) dan menyelam tepat ke dalamnya. Penyelaman pertama-tama diam di tempat sementara titik itu dihitung hingga kedalaman batas selam — sesaat pada kedalaman biasa, lebih lama setelah 1e1000×. Hanya himpunan Mandelbrot.
panel-work-budget = Anggaran render langsung
panel-work-budget-hover = Detail vs. kecepatan untuk tampilan langsung pada zoom dalam. Nilai lebih tinggi merender pada resolusi yang lebih penuh (lebih tajam — mengurangi kesan “lembek” hasil upscale) tetapi menurunkan laju bingkai, dan nilai yang sangat tinggi berisiko membuat GPU tersendat sesaat pada bingkai terberat. Ekspor selalu beresolusi penuh, berapa pun nilai ini.
panel-min-motion-res = Resolusi gerak min
panel-min-motion-res-hover = Resolusi terendah yang boleh digunakan tampilan langsung saat terus melakukan zoom atau menggeser pada zoom dalam. Nilai lebih tinggi mencegah penyelaman cepat menjadi kotak-kotak/pecah piksel (lebih tajam) dengan mengorbankan laju bingkai — pada 100%, penyegaran detail penyelaman dalam harus dirender dengan biaya penuh, yang dapat membuat zoom dalam terlihat tersendat/patah-patah. Jika penyelaman dalam terasa tersentak-sentak, turunkan nilai ini. Bawaan 30%; tidak memengaruhi gambar yang stabil atau ekspor.
panel-finish-sound = Bunyi saat render selesai
panel-finish-sound-hover = Putar nada selesai milik FRACTINT saat ekspor atau render tur selesai: tiga nada naik (1047, 1109, 1175 Hz masing-masing selama 100 ms), persis seperti yang dikodekan versi DOS aslinya — disintesis sebagai gelombang kotak dengan pita terbatas dan dilewatkan melalui filter high-pass agar terdengar seperti kerucut pengeras suara PC kecil yang dulu memainkannya.
panel-prefer-detail = Utamakan detail saat zoom
panel-prefer-detail-hover = Saat melakukan zoom atau menggeser, tetap tampilkan bingkai terakhir yang sepenuhnya detail — diskalakan dan dilacak mengikuti gerakan — alih-alih merender ulang dengan kualitas yang dikurangi setiap bingkai. Tampilan dirender penuh saat gerakan berhenti sejenak. Nonaktif: gerakan dirender langsung pada resolusi yang dikurangi (lebih mulus, lebih kasar). Tampilan dangkal (mode langsung) selalu dirender langsung — tampilan itu murah dan tajam di setiap bingkai bagaimanapun juga.

## The control panel: About (the current fractal)

# A link to the fractal's reference page, followed by an arrow.
panel-about-reference = Referensi

## The control panel: Advanced ▸ Second graphics card

# A label before a drop-down (wider than the panel's label column already).
gpu2-label = Kartu grafis kedua
gpu2-off = Mati
# Another device on the same graphics card as the window (the default).
gpu2-same-card = Kartu yang sama
# A card the list does not name. $number: the card's number.
gpu2-card-n = Kartu { $number }
gpu2-finding = Mencari kartu grafis mesin ini…
gpu2-none = Mesin ini tidak memiliki kartu grafis lain.
gpu2-hover =
    Perangkat grafis lain merender bingkai tampilan dalam saat bergerak, dan sebagian sampel tambahan yang menajamkannya setelah berhenti, sehingga gambar dapat mengimbangi zoom dan mencapai kualitas akhirnya lebih cepat. Kartu yang menggambar jendela tetap menggambar semuanya.

    Kartu lain menambahkan kecepatannya sendiri. Dua model kartu yang berbeda menggambar beberapa piksel sedikit berbeda; jika kartu kedua gagal, tampilan berlanjut dengan satu kartu.

    “Kartu yang sama” (bawaan) membuka perangkat kedua pada kartu ini. Ini tidak menambah kecepatan, tetapi merender tanpa menunggu bingkai layar. Perangkat ini berbagi kartu: kegagalan yang mereset kartu akan mengakhiri aplikasi, sama seperti kegagalan render jendela itu sendiri, dan peluncuran berikutnya akan menawarkan untuk mematikannya.

## The view (canvas)

# A notice after Ctrl+click pins the Julia c at a deep zoom.
canvas-julia-pin-quantized = Julia c dikunci — tetapi nilainya disimpan dalam presisi ganda, sehingga pada kedalaman ini seluruh panel dibulatkan menjadi satu nilai dan kuncinya jatuh di pusat tampilan. Perkecil untuk membidiknya.
# A small caption in the top-left corner of the view while the orbit overlay is fitted to the view (its "Normalize (fit to view)" option).
canvas-orbit-normalized = orbit (dinormalkan)
# The minimap's tooltip.
canvas-minimap-hover = Ikhtisar / Anda di sini. Seret untuk menggeser pada zoom saat ini, gulir untuk memperbesar dan memperkecil, klik untuk melompat ke wilayah itu (zoom awal).

## The timestamp overlay (View ▸ Show timestamp)

# Under the large elapsed-time clock. $frame: the frame number (as the log writes it, without grouping).
overlay-timestamp-frame = bingkai { $frame }

## File ▸ Settings ▸ Reference cache…

cache-title = Cache referensi
cache-intro = Pada kedalaman ekstrem, bagian yang lambat dari sebuah tampilan adalah orbit referensinya — hingga satu jam aritmetika presisi sembarang. Fractadyne menyimpan orbit yang sudah selesai di sini, sehingga kembali ke lokasi yang pernah Anda kunjungi, atau melakukan zoom di dekatnya, hanya memakan beberapa detik. Satu orbit berukuran sekitar 4 MB.
cache-off = Nonaktif untuk sesi ini (pemanggilan tugas, atau --no-orbit-cache). Tidak ada yang dibaca atau ditulis sampai peluncuran biasa berikutnya.
# Small headings of the window's parts.
cache-location = Lokasi
cache-usage = Penggunaan
cache-limit = Batas
cache-no-config = (tidak ada direktori konfigurasi)
cache-open-folder = Buka folder
# The text on the usage bar. $used and $limit: sizes such as "1.2 GB"; $count: the orbits kept.
cache-usage-bar = { $used } dari { $limit } — { $count } orbit
cache-limit-hover = Saat cache melampaui batas ini, orbit yang PALING MURAH untuk dibangun dihapus lebih dulu — orbit dalam yang butuh satu jam akan bertahan lebih lama daripada orbit cepat sebanyak apa pun, betapapun baru orbit-orbit itu digunakan.
# Beside the limit. $count: about how many orbits of the live view's length fit in it.
cache-limit-estimate = ≈ { $count } orbit pada batas tampilan langsung
cache-kept = Disimpan (yang paling berharga lebih dulu)
# Column headers of the list of kept orbits.
cache-col-iterations = iterasi
cache-col-precision = presisi
cache-col-size = ukuran
cache-col-last-used = terakhir dipakai
# The precision column. $bits: the precision in bits.
cache-row-bits = { $bits } bit
# Under the list. $count: the orbits not listed.
cache-rows-more = … dan { $count } lainnya
# When an orbit was last used (the "last used" column).
cache-ago-now = baru saja
# $count: minutes.
cache-ago-minutes = { $count } mnt lalu
# $count: hours.
cache-ago-hours = { $count } jam lalu
# $count: days.
cache-ago-days = { $count } hr lalu
# Asked before clearing. $count: the orbits; $size: their total size, such as "1.2 GB".
cache-confirm = Hapus { $count } orbit ({ $size })? Membangunnya kembali memakan waktu, bukan data.
cache-delete = Hapus
cache-cancel = Batal
cache-clear = Kosongkan cache…
cache-clear-hover = Menghapus setiap orbit yang tersimpan. Meminta konfirmasi terlebih dahulu.
cache-close = Tutup
# $count: the orbits removed.
cache-cleared = Cache referensi dikosongkan — { $count } orbit dihapus.
# $why: the system's error message.
cache-clear-failed = Tidak dapat mengosongkan cache referensi: { $why }

## Help ▸ Diagnostics…

diag-title = Diagnostik
diag-intro = Jalankan uji untuk memeriksa apakah Fractadyne bekerja dengan benar di perangkat keras Anda. Hasilnya disimpan di mesin Anda dan dapat dilampirkan ke laporan masalah — tidak ada yang dikirim ke mana pun dengan sendirinya.
# $status: which arbitrary-precision library is in use, as the program reports it (in English).
diag-arithmetic = Aritmetika zoom dalam: { $status }
# The three tests' buttons, each followed by a sentence saying what it does.
diag-run-selftest = Jalankan uji mandiri
diag-run-uitest = Jalankan uji UI
diag-run-gputest = Jalankan pemeriksaan aritmetika GPU
diag-blurb-selftest = Memeriksa matematika dan render di GPU Anda terhadap hasil yang diketahui benar. Memakan waktu sekitar 15 detik.
diag-blurb-uitest = Menelusuri antarmuka dan tampilan langsung, sambil mengambil tangkapan layar pada beberapa kedalaman zoom. Memakan waktu satu atau dua menit dan membuka jendela selama berjalan.
diag-blurb-gputest = Memeriksa aritmetika double-float shader terhadap nilai yang diketahui benar pada setiap backend grafis yang tersedia di mesin Anda. Memakan waktu beberapa detik, tidak membuka jendela. Jika pemeriksaan ini melaporkan kegagalan, itu adalah temuan nyata yang ingin kami lihat — harap lampirkan ke laporan.
# While a test runs. $count: the checks finished so far.
diag-checks-done = { $count } pemeriksaan selesai
# A finished test's result. $test: the test's name (Self-test, UI test, GPU arithmetic check).
diag-verdict-captured = { $test } — hasil direkam
diag-verdict-passed = { $test } — lulus
diag-verdict-problems = { $test } — melaporkan masalah
# Under the GPU arithmetic check's result.
diag-informational-note = Uji ini tidak memiliki hasil lulus atau gagal: uji ini mencatat cara GPU Anda menghitung, dan kegagalan yang dilaporkan adalah temuan nyata, bukan kerusakan pada mesin Anda. Bagaimanapun juga, melampirkannya ke laporan adalah hal paling berguna yang dapat Anda lakukan dengan hasil ini.
# Under a test's result when it reported problems.
diag-problems-note = Sejumlah perbedaan memang wajar pada perangkat keras selain GPU referensi — terutama perbandingan gambar. Melampirkan ini ke laporan masalah adalah hal paling berguna yang dapat Anda lakukan dengan hasil ini.
diag-open-results = Buka hasil
diag-attach = Lampirkan ke laporan masalah…
diag-attach-hover = Membuka dialog laporan dengan hasil ini disertakan, sehingga issue membawa uji yang divalidasi mesin, bukan sekadar deskripsi
diag-console = Cetak diagnostik ke konsol
diag-console-hover = Tampilkan baris [fd-…] di stderr mulai sekarang. Nonaktif secara bawaan saat Fractadyne diluncurkan tanpa argumen. Ini tidak mengubah apa pun tentang apa yang DIREKAM — berkas log selalu mendapatkan setiap baris — jadi biarkan nonaktif kecuali Anda sedang mengamati terminal. Hanya berlaku untuk sesi ini; gunakan flag --console atau FRACTADYNE_CONSOLE=1 untuk juga menangkap proses awal.
diag-close = Tutup
# Errors starting a test. $why: the system's error message; $dir: a folder.
diag-error-no-exe = Tidak dapat menemukan berkas program: { $why }
diag-error-create-dir = Tidak dapat membuat { $dir }: { $why }
diag-error-start = Tidak dapat memulai uji: { $why }

## Navigate ▸ Misiurewicz explorer…

misi-title = Penjelajah Misiurewicz
misi-mandelbrot-only = Penjelajahan Misiurewicz hanya untuk Mandelbrot — ganti fraktalnya untuk menggunakannya.
# While the gallery is being built. $seconds: the time so far, such as "3".
misi-solving = Menghitung keluarga-keluarga jenis kecil ({ $seconds }s)…
# Above the gallery. $total: all points; $curated: the hand-picked ones; $found: the ones the sweep found.
misi-summary = { $total } titik — { $curated } pilihan kurasi, { $found } ditemukan oleh sapuan Newton berbenih atas jenis-jenis kecil (setengah bidang atas; kisi kasar menemukan sebagian besar cekungan, tetapi tidak terbukti semuanya).
# The name of a point the sweep found. $kind: its type, such as "(3,1)"; $point: its coordinate, such as "-0.1011+0.9563i".
misi-entry-name = { $kind } di { $point }
# Under each thumbnail. $kind: the point's type, such as "(3,1)"; $octaves: the zoom period in octaves, such as "3.17"; $twist: the twist per repeat in degrees, such as "+12.5". "oct" is short for octaves.
misi-entry-readout = { $kind } · { $octaves } okt/ulang · { $twist }°
# Before the depth field.
misi-zoom-to = Zoom ke
misi-cancel = Batal
misi-solve-jump = Hitung & lompat
misi-solve-jump-hover = Pecahkan titik persis ini dengan metode Newton hingga presisi yang dibutuhkan kedalaman yang diminta (dengan koordinat tersimpannya sebagai benih), lalu langsung melompat ke sana. Permintaan yang dalam memakan waktu lebih lama — perhitungannya menggunakan presisi sembarang.
misi-select-hint = Pilih sebuah titik, tentukan kedalaman, lalu lompat — baris pengali di bawah setiap gambar mini menunjukkan seperti apa penyelaman di sana: berapa oktaf sebelum tampilan berulang, dan seberapa jauh tampilan berputar setiap pengulangan.
misi-cancelled = Perhitungan dibatalkan.
# $text: what was typed in the depth field.
misi-bad-zoom = Tidak dapat membaca “{ $text }” sebagai perbesaran (tulis 1e50, 3.2e120, …).
misi-no-kp = Entri ini tidak memiliki (k,p) terdeteksi untuk dihitung.
misi-parse-failed = Koordinat tersimpan gagal diurai.
# A notice after the jump. $name: the point's name; $zoom: the magnification, such as "1e50"; $seconds: how long the solve took, such as "1.2".
misi-zoomed = Zoom ke { $name } — { $zoom }× ({ $seconds }s)
# Added after the notice above when the iteration count is too low. $have: the fixed iteration count; $typical: what the depth typically needs.
misi-iter-note = Iterasi ditetapkan pada { $have }; kedalaman ini biasanya memerlukan ~{ $typical } — naikkan (atau aktifkan iterasi otomatis) jika tampilan dirender datar.
# $name: the point's name; $why: why the solve failed (a technical word).
misi-solve-missed = Perhitungan dalam tidak mendarat di { $name }: { $why }. Titik yang tersimpan tetap persis pada kedalamannya sendiri — coba target yang lebih dangkal.

## Life (Game of Life): the side panel

# Shown when the graphics card cannot run the Life view.
life-no-compute = Adaptor grafis ini tidak dapat menjalankan Life (tidak ada compute shader).
# Row label: the pattern list (pattern names stay in English).
life-pattern = Pola
life-open = Buka…
life-open-hover = Buka berkas pola RLE, .cells, atau Life 1.05/1.06.
life-pattern-text = Teks pola…
life-pattern-text-hover = Tempelkan pola (RLE seperti yang diberikan LifeWiki, atau teks biasa).
life-save = Simpan…
life-save-hover = Simpan semesta sebagaimana adanya sekarang, sebagai RLE.
# Row label: the rule list and the typed rule (B3/S23 notation).
life-rule = Aturan
# Shown in the rule list when the rule in use is none of the library's.
life-rule-custom = kustom
life-apply = Terapkan
# Playback buttons (a pause or play symbol is shown before them).
life-pause = Jeda
life-play = Putar
life-play-hover = Jalankan semesta (Enter).
life-step = Langkah
life-step-hover = Maju satu generasi (N).
# Tooltip of the button beside Step, which reads "+16", "+1,024"…: that many generations.
life-stride-hover = Maju sebanyak generasi ini (Shift+N).
life-reset = Atur ulang
life-reset-hover = Kembali ke pola seperti saat dibuka (generasi 0).
life-speed = Kecepatan
# After the speed slider's number: generations a second. Short.
life-speed-unit = gen/s
# Row label: how many generations the "+n" button advances.
life-stride = Lompatan
# The panel's numbers.
life-generation = Generasi
life-population = Populasi
life-tiles = Petak
# $count: the tiles stored; $capacity: how many the GPU pool holds.
life-tiles-of = { $count } dari { $capacity }
life-tiles-hover = Petak 64×64 sel yang tersimpan (masing-masing 4 KiB) dan kapasitas kumpulan GPU.
# Row label: what a left-drag in the view does (Pan or Draw).
life-mouse = Mouse
life-pan = Geser
life-pan-hover = Seret untuk memindahkan tampilan.
life-draw = Gambar
# The Draw tool's tooltip; the toolbar's shows it after "Draw (D):", so it starts lowercase.
life-draw-hint = klik sebuah sel untuk membaliknya; seret untuk melukis — goresan yang dimulai pada sel mati akan menggambar, goresan yang dimulai pada sel hidup akan menghapus. Semesta yang sedang berjalan dijeda selama Anda menggambar dan berlanjut saat Anda melepaskannya.
life-undo = Urungkan
life-undo-hover = Kembali ke semesta sebelum suntingan, isian, pengosongan, atau perubahan aturan terakhir (Ctrl+Z saat menggambar).
life-grid = Garis kisi
life-grid-hover = Batas sel saat diperbesar hingga 8 piksel per sel atau lebih.
# A random fill: "Random" labels the density field, "seed" the seed field after it.
life-random = Acak
life-density-hover = Porsi sel yang dihidupkan oleh pengisian.
life-seed = benih
life-fill-view = Isi tampilan
# $cells: the largest side of the filled area, in cells (2,048).
life-fill-view-hover = Isi area yang terlihat (hingga { $cells } × { $cells } sel di sekitar pusatnya) secara acak.
life-clear = Kosongkan
life-clear-hover = Matikan setiap sel (Urungkan mengembalikannya).

## Life: notices in the panel, and file dialogs

# $path: the file written.
life-saved = { $path } disimpan
# $error: why (from the system, in English).
life-save-failed = Gagal menyimpan: { $error }
# The universe stopped running. $error: why (in English).
life-stopped = Berhenti: { $error }
# A pattern file named a rule that does not read. $error: why (in English); $rule: the rule kept, in B3/S23 notation.
life-rule-not-understood = Aturan pola tidak dipahami ({ $error }); { $rule } dipertahankan
# $path: the file; $error: why (in English).
life-open-failed = Tidak dapat membuka { $path }: { $error }
# File-dialog filter names.
life-filter-patterns = Pola Life
life-filter-rle = Pola run-length encoded

## Life: the toolbar (its buttons are icons; these are their tooltips)

life-toolbar-pause-hover = Jeda semesta (Enter)
life-toolbar-play-hover = Jalankan semesta (Enter)
# $stride: the generations a Shift+click steps.
life-toolbar-step-hover = Maju satu generasi (N); Shift+klik atau Shift+N maju { $stride }
life-toolbar-reset-hover = Kembali ke pola seperti saat dibuka (generasi 0)
# $hint: the Draw tool's tooltip (life-draw-hint).
life-toolbar-draw-hover = Gambar (D): { $hint }

## Life: the Pattern text window

life-text-title = Teks pola
life-text-intro = Tempelkan pola: RLE (seperti yang diberikan LifeWiki dan Golly), teks biasa .cells, atau Life 1.05 / 1.06.
life-text-load = Muat
life-text-current = Saat ini sebagai RLE
life-text-current-hover = Isi kotak dengan semesta seperti saat terakhir dimuat.

## L-systems: the side panel

# Row label: the system list (system names stay in English).
lsys-system = Sistem
lsys-open = Buka…
lsys-open-hover = Buka berkas Fractint .l atau sistem .lsys.
lsys-edit = Sunting…
lsys-edit-hover = Teks sistem: aksioma, produksi, sudut — sunting lalu terapkan.
lsys-save = Simpan…
lsys-save-hover = Simpan sistem sebagai berkas teks .lsys.
# Tooltip of the "SVG…" button.
lsys-svg-hover = Ekspor tampilan sebagai gambar SVG: garis dan bentuknya sebagai vektor, dengan warnanya.
lsys-follow = Orde mengikuti zoom
lsys-follow-hover = Orde naik saat Anda memperbesar, sehingga detailnya tidak pernah habis.
lsys-follow-fixed-hover = Sistem ini tidak tumbuh dengan suatu faktor di setiap orde, sehingga digambar pada orde tetap.
lsys-order = Orde
lsys-angle = Sudut
lsys-sweep-hover = Sapukan sudutnya, pergi dan kembali: kurva berubah bentuk sambil berputar
# A small button that puts back the system's own angle after the slider moved it.
lsys-system-angle = Sudut sistem
lsys-seed = Benih
lsys-seed-hover = Yang diikuti pilihan acak: benih yang sama, tumbuhan yang sama.
lsys-seed-new = Baru
lsys-seed-new-hover = Tumbuhan lain: benih baru.
lsys-line-width = Lebar garis
# Row label of a percentage slider: how much of the curve is drawn.
lsys-draw-on = Penggambaran
lsys-draw-on-hover = Seberapa banyak kurva yang digambar, menurut urutan kura-kura menggambarnya
lsys-draw-on-play-hover = Kurva menggambar dirinya sendiri
lsys-colour-by = Warnai menurut
# The colourings, in the "Colour by" list.
lsys-colour-position = posisi di sepanjang kurva
lsys-colour-depth = kedalaman cabang
lsys-colour-heading = arah hadap
# C, < and > are the system's commands that set the colour index: keep them.
lsys-colour-index = indeks warna (C, <, >)
lsys-colour-plain = satu warna
# The panel's numbers.
lsys-growth = Pertumbuhan
lsys-growth-built = dibangun sebagai kata, pada ordenya
lsys-growth-built-hover = Parametrik atau peka konteks: penulisan ulang sebuah modul bergantung pada bilangannya atau tetangganya, sehingga kata dibangun satu generasi demi satu generasi (hingga dua juta modul) dan digambar dalam presisi ganda — orde tidak mengikuti zoom.
# How much the picture grows from one order to the next. $factor: the factor (1.4142).
lsys-growth-factor = ×{ $factor } per orde
lsys-growth-factor-period-2 = ×{ $factor } per orde (orde naik per 2)
lsys-growth-none = tidak ada (orde tetap)
lsys-drawn = Tergambar
# $drawn: what was drawn (lsys-drawn-segments, lsys-drawn-shapes or lsys-drawn-segments-shapes); $order: the order.
lsys-drawn-at-order = { $drawn } pada orde { $order }
# The same, when the walk stopped at its segment budget.
lsys-drawn-at-order-budget = { $drawn } pada orde { $order } (anggaran)
# $segments: the line segments drawn.
lsys-drawn-segments = { $segments } segmen
# $shapes: the filled shapes drawn.
lsys-drawn-shapes = { $shapes } bentuk terisi
# Both counts. $segments: line segments; $shapes: filled shapes.
lsys-drawn-segments-shapes = { $segments } segmen, { $shapes } bentuk terisi
# $ms: the walk's time in milliseconds (12.3).
lsys-drawn-hover = ditelusuri dalam { $ms } ms; subpohon di luar tampilan dilewati, dan yang lebih kecil dari satu piksel digambar sebagai satu segmen
# A word too long to build at the order asked for. $wanted: the order asked for; $built: the order drawn.
lsys-drawn-short = Kata orde { $wanted } melebihi dua juta modul: digambar pada orde { $built }.
# $order: the deepest order the app draws.
lsys-deepest-order = Pada orde terdalam yang digambar ({ $order }): zoom lebih jauh tidak menambah detail.

## L-systems: notices in the panel and the editor, and file dialogs

# $path: the file; $error: why (in English).
lsys-open-failed = Tidak dapat membuka { $path }: { $error }
# Why a file was not opened (shown after "Could not open …:").
lsys-file-too-large = lebih besar dari 4 MiB
# Why a Fractint .l file was not opened: it holds no entries. `Name { … }` is the file's syntax: keep it.
lsys-no-entries = tidak ada entri (berkas .l berisi entri `Name {"{"} … {"}"}`)
# $path: the file written.
lsys-saved = { $path } disimpan
# $error: why (from the system, in English).
lsys-save-failed = Gagal menyimpan: { $error }
# After File ▸ Export SVG. $path: the file written.
lsys-svg-exported = { $path } diekspor
# $error: why (from the system, in English).
lsys-svg-failed = Ekspor gagal: { $error }
# File-dialog filter names: opening (any system file) and saving (one system).
lsys-filter-systems = Sistem-L
lsys-filter-system = Sistem-L

## L-systems: the toolbar

lsys-toolbar-lower-hover = Turunkan orde (sekaligus menetapkannya)
lsys-toolbar-raise-hover = Naikkan orde (sekaligus menetapkannya)
# Between the − and + buttons, in a fixed-width font. $order: the order, padded to 3 characters so the label keeps its width.
lsys-toolbar-order = orde { $order }
# A toggle: the order follows the zoom. Short.
lsys-toolbar-auto = otomatis
lsys-toolbar-auto-hover = Orde mengikuti zoom
lsys-toolbar-edit-hover = Sunting teks sistem

## L-systems: the editor window, and a .l file's entries

lsys-editor-title = Sistem-L
# The editor's help. The quoted examples ('X = word', 'seed n', 'define R 1.456'…), the commands and "The Algorithmic Beauty of Plants" (a book's title) are the system's syntax and names: keep them as they are.
lsys-editor-help = Satu kunci per baris: angle (derajat, atau /n untuk pembagian lingkaran), axiom, produksi 'X = word', dan secara opsional heading, draw / move / variables, colour, order. Simbol dengan alternatif yang dipilih secara acak memiliki satu baris untuk tiap alternatif, beserta bobotnya: 'X (0.3) = word'; 'seed n' memilih tumbuhannya. Di dalam kata, {"{"} {"}"} mengisi jalur kura-kura dan . menambahkan sebuah verteks. Produksi parametrik dan peka konteks (The Algorithmic Beauty of Plants): 'A(s) : s > 1 = F(s)[+A(s/2)]', 'b < a > c = b', dengan 'define R 1.456' dan 'ignore +-F'; F(l) melangkah sejauh l, +(a) berbelok a derajat. Entri .l Fractint dapat ditempel apa adanya.
lsys-editor-apply = Terapkan
lsys-editor-revert = Kembalikan
lsys-editor-revert-hover = Teks sistem yang ditampilkan.
# The window listing a .l file's entries to pick from. $file: the file's name.
lsys-entries-title = { $file } — entri
# Tooltip of an entry that does not parse (its error is shown beside it, in English).
lsys-entry-unreadable-hover = Entri ini tidak dapat dibaca.

## Export: opening a view or location (File ▸ Open view or location…)

# File-dialog filter names.
export-filter-view = Tampilan atau lokasi Fractadyne
export-filter-image = Gambar (PNG / EXR)
export-filter-location = Lokasi (.fdn / .kfr)
# $zoom: the magnification (1.2e148), followed by the magnification sign ×.
export-loaded-location = Lokasi dimuat @ { $zoom }×
# $zoom: the magnification; $note: what was noteworthy in the file (in English).
export-loaded-location-note = Dimuat @ { $zoom }× — { $note }
# $path: the file.
export-not-location = { $path } bukan lokasi Fractadyne.
export-file-too-large = Berkas terlalu besar (bukan lokasi .fdn?).
# $path: the file; $error: why (from the system, in English).
export-read-failed = Tidak dapat membaca { $path }: { $error }
# A Kalles Fraktaler .kfr file did not import. $error: why (in English).
export-import-failed = Impor gagal: { $error }
# $path: an image whose embedded text is not a view.
export-not-view = { $path } memiliki metadata Fractadyne, tetapi bukan tampilan.
# $path: the image the view was read from.
export-loaded-view = Tampilan dimuat dari { $path }
# $path: the image; $note: what was noteworthy in it (in English).
export-loaded-view-note = Tampilan dimuat dari { $path } — { $note }
export-no-metadata = Berkas itu tidak memiliki metadata tampilan Fractadyne yang tersemat.

## Export: notices after an export or a snapshot
## "File → Export image…" in them names the menu items (menu-file, menu-file-export).

# The export window's status while the image renders.
export-rendering = Merender…
export-canceled = Ekspor dibatalkan.
# $error: why (in English).
export-failed = Ekspor gagal: { $error }
# $size: the image's size in pixels (1920×1080); $path: the file written.
export-saved = { $size } disimpan → { $path }
# An L-system export whose walk stopped at its budget. $size: pixels; $segments: the budget (4,000,000); $path: the file.
export-saved-incomplete = { $size } disimpan (tidak lengkap: melewati anggaran { $segments } segmen — turunkan ordenya) → { $path }
# A two-panel export written as two files. $path: the first of them.
export-saved-two = 2 berkas disimpan → { $path }
# $size: pixels; $path: the file written.
export-saved-corrected = { $size } disimpan (glitch dikoreksi) → { $path }
# $path: the first of the two files.
export-saved-two-corrected = 2 berkas disimpan (glitch dikoreksi) → { $path }
# Added after a finished export's notice. $duration: how long it took (8.3s, 1m 04.0s).
export-elapsed = (dalam { $duration })
export-preparing-deep = Menyiapkan ekspor dalam — membangun referensi (ini bisa memakan waktu cukup lama)…
# $size: pixels (5120×4035); $ss: the supersampling factor; $iterations: the iteration count.
export-large = Ekspor besar ({ $size }, supersampling { $ss }×, { $iterations } iterasi) — dirender di latar belakang; koreksi glitch dilewati untuk ekspor sebesar ini…
export-life-unavailable = Ekspor gambar dari tampilan Life belum tersedia — gunakan tangkapan layar, atau Berkas ▸ Simpan pola Life.
export-already-running = Sebuah ekspor sedang berjalan — Berkas → Ekspor gambar… menampilkan kemajuannya.
# $size: the image's size in pixels.
export-snapshot-background = Cuplikan: merender { $size } di latar belakang — Berkas → Ekspor gambar… menampilkan kemajuannya dan dapat membatalkannya.
export-no-gpu = GPU tidak tersedia
# A screen-capture snapshot. $size: pixels; $path: the file written.
export-snapshot-saved = Cuplikan layar { $size } disimpan → { $path }
# $error: why (in English).
export-snapshot-failed = Cuplikan gagal: { $error }

## The colour picker (the popup of a colour swatch)

# The third row's label: the colour as hexadecimal bytes. Short (about 6 characters).
picker-row-hex = Heks
picker-hex-hover = Klik untuk mengetik nilai heks
picker-cancel = Batal
picker-cancel-hover = Kembalikan warna saat pemilih ini dibuka
picker-ok = OK
picker-ok-hover = Simpan warna ini
# Starts the screen eyedropper (an icon is shown before it).
picker-pick = Ambil
picker-pick-hover = Ambil warna dari mana saja di layar
# Tooltip of the disabled Pick button (in the picker and the gradient editor) on Linux and macOS.
picker-eyedropper-unsupported = Mengambil warna dari bagian lain layar memerlukan API tangkapan layar milik platform. Hanya build Windows yang memilikinya; di Linux hal ini memerlukan dependensi X11, dan Wayland memerlukan portal dengan permintaan izinnya sendiri.

## The fractal families: what each one is (the info panel's "About" section and the pickers' hover text).
## The formulas themselves (z -> z^2 + c) are written in code and stay as they are.

fractal-mandelbrot-about = Fraktal waktu lolos yang kanonis: himpunan c yang orbit 0-nya tetap terbatas. Batasnya memiliki kerumitan yang tak terhingga.
fractal-multibrot3-about = Himpunan Multibrot - konstruksi Mandelbrot pada pangkat yang lebih tinggi. Pangkat d menghasilkan simetri putar lipat (d-1).
fractal-multibrot4-about = Multibrot pada pangkat 4: simetri lipat tiga, bulatan lebar.
fractal-multibrot5-about = Multibrot pada pangkat 5: simetri lipat empat.
fractal-tricorn-about = Tricorn (Mandelbar): mengonjugasikan z di setiap langkah. Pemetaan anti-holomorfik ini menghasilkan bentuk bersudut tiga.
fractal-burning-ship-about = Nilai mutlak bagian-bagian z diambil sebelum dikuadratkan; hasilnya menyerupai kapal yang terbakar.
fractal-celtic-about = Kerabat Burning Ship yang mengambil nilai mutlak hanya dari bagian real z^2, menghasilkan motif simpul Kelt / hati.
fractal-buffalo-about = Varian abs yang mengambil nilai mutlak kedua komponen z^2.
fractal-phoenix-about = Phoenix juga menggunakan iterasi sebelumnya, menghasilkan filamen mirip nyala api. Coba bentuk Julia-nya melalui mode Julia.
fractal-newton-about = Iterasi pencarian akar Newton untuk z^3 = 1, diwarnai berdasarkan seberapa cepat setiap titik konvergen. Fraktal konvergensi (bukan lolos).
fractal-multibrot6-about = Multibrot pada pangkat 6: simetri lipat lima.
fractal-multibrot7-about = Multibrot pada pangkat 7: simetri lipat enam, bulatannya menipis mendekati lingkaran.
fractal-multibrot8-about = Multibrot pada pangkat 8: simetri lipat tujuh.
fractal-burning-ship3-about = Burning Ship pada pangkat 3: nilai mutlak diambil, lalu dipangkatkan tiga.
fractal-burning-ship4-about = Burning Ship pada pangkat 4.
fractal-burning-ship5-about = Burning Ship pada pangkat 5.
fractal-tricorn3-about = Tricorn (Mandelbar) pada pangkat 3: simetri lipat empat.
fractal-tricorn4-about = Tricorn pada pangkat 4: simetri lipat lima.
fractal-tricorn5-about = Tricorn pada pangkat 5: simetri lipat enam.
fractal-celtic3-about = Lipatan Celtic pada pangkat 3: nilai mutlak bagian real z^3.
fractal-celtic4-about = Lipatan Celtic pada pangkat 4.
fractal-celtic5-about = Lipatan Celtic pada pangkat 5.
fractal-buffalo3-about = Buffalo pada pangkat 3: nilai mutlak kedua bagian z^3.
fractal-buffalo4-about = Buffalo pada pangkat 4.
fractal-buffalo5-about = Buffalo pada pangkat 5.
fractal-custom-about = Rumus yang ditulis dalam ekspresi gaya Fractint (Fraktal ▸ Rumus kustom…). Sebagian besar mendukung zoom dalam dengan perturbasi seperti keluarga bawaan; dialog rumus menyebutkan apakah rumus Anda mendukungnya.
fractal-life-about = Game of Life karya Conway dan kerabatnya pada bidang tak terbatas: aturan Generations, aturan non-totalistik (Hensel), berkas pola. Putar, maju selangkah, gambar.
fractal-lsystem-about = Sistem Lindenmayer: kurva, kurva pengisi ruang, pulau, dan tumbuhan yang digambar oleh kura-kura dari tata bahasa penulisan ulang. Orde mengikuti zoom, sehingga detailnya tidak pernah habis. Berkas .l Fractint dapat dibuka langsung.

# Shown where the other families show their formula (z -> z^2 + c).
fractal-custom-formula = z -> rumus yang Anda tulis
# Shown where the other families show their formula. "B3/S23" is the rule's notation: keep it.
fractal-life-formula = B3/S23: lahir dengan 3 tetangga hidup, bertahan dengan 2 atau 3
# Shown where the other families show their formula. "F = F+F--F+F" is the grammar's notation: keep it.
fractal-lsystem-formula = F = F+F--F+F: setiap simbol ditulis ulang, lalu digambar oleh kura-kura

# The last paragraph of the picker's hover text for a family that renders only on the direct path.
fractal-no-deep-zoom = Hanya render langsung - keluarga ini tidak memiliki zoom dalam.

## Names of settings, shown in the Settings menu and the welcome window.

theme-dark = Gelap
theme-light = Terang
# The update track that follows only stable releases.
update-track-stable = Stabil
# The update track that also offers pre-releases (betas).
update-track-beta = Beta (pra-rilis)

## The status bar (its hover texts; the readouts follow in their own section)

status-period-hover = Periode minibrot yang Anda tuju (tekan M, atau Pergi ke lokasi ▸ Pergi ke fitur). Nilai ini tetap ada selama tampilan berada pada fitur itu dan hilang saat Anda menjauh.

## The playback transport: the player over the view while a tour plays.

# $close: the close button's icon, as drawn on that button.
play-finished-hover = Selesai — geser kembali ke dalam tur, atau { $close } untuk menutup
# $kind: "benchmark" or "script" (the variants hold the wording); $name: the tour's name.
play-playing-hover =
    { $kind ->
        [benchmark] Memutar tolok ukur “{ $name }”
       *[other] Memutar skrip “{ $name }”
    }
play-restart = Mulai ulang dari awal
play-back = Mundur 10 detik
play-again = Putar lagi dari awal
play-resume = Lanjutkan
play-pause = Jeda
play-stop = Hentikan dan kembali ke awal
play-forward = Maju 10 detik
play-speed-hover = Kecepatan pemutaran (klik untuk berganti 0,5× / 1× / 2× / 4×)
play-render = Render skrip ini menjadi urutan bingkai…
play-render-none = Tolok ukur bawaan tidak memiliki berkas skrip untuk dirender
play-open = Putar skrip lain…
play-loop = Ulangi tur saat mencapai akhir
play-close = Tutup pemutar dan pulihkan pengaturan tampilan Anda sendiri
# A notice beside the scrub bar while the renderer holds the tour's clock. Keep it short.
play-waiting = menunggu detail
# "[playback] pace" is a setting in the script file: keep it as written.
play-waiting-hover = Jam tur dijeda sementara perender menyelesaikan tampilan ini (pembangunan referensi / anggaran iterasi sedang naik). Pemutaran berlanjut dengan sendirinya; lihat [playback] pace di skrip.
play-scrub = Seret untuk menggulir di sepanjang tur
# $behind: how far the script clock is behind real time, as minutes:seconds (1:05).
play-scrub-behind =
    Seret untuk menggulir di sepanjang tur
    Jam skrip tertinggal { $behind } dari waktu nyata (pengatur tempo memperlambat tur agar perender dapat mengimbangi). Tanda atas menunjukkan posisi pemutaran menurut jam dinding.

## The autopilot's notices (toasts). Where a notice has two lines, the first is the verdict and the second what to do about it.

autopilot-on-detail = Autopilot aktif — menyelam ke arah detail (masukan apa pun menghentikannya)
autopilot-off = Autopilot nonaktif
autopilot-misi-needs-mandelbrot =
    Autopilot: target Misiurewicz memerlukan himpunan Mandelbrot
    Menyelam ke arah detail sebagai gantinya
autopilot-on-misi =
    Autopilot aktif — mencari titik Misiurewicz terdekat…
    Masukan apa pun menghentikannya
# $seconds: how long the search has run, in whole seconds.
autopilot-misi-finding =
    Autopilot: mencari titik Misiurewicz terdekat…
    { $seconds } s · masukan apa pun menghentikannya
autopilot-misi-not-found =
    Autopilot: tidak ditemukan titik Misiurewicz di dekat tampilan ini
    Pencari mungkin tidak menjangkau kedalaman ini — coba target Detail
# ($preperiod,$period) names the point, as mathematicians write it: keep the notation. $widths: a power of two (2^$widths screen widths).
autopilot-misi-off-screen =
    Autopilot: titik Misiurewicz terdekat ({ $preperiod },{ $period }) berada di luar layar
    Jaraknya ~2^{ $widths } lebar layar — dekati pusat spiral
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-not-solved =
    Autopilot: titik Misiurewicz ({ $preperiod },{ $period }) di dekat tampilan ini tidak berhasil dihitung
    Coba target Detail
autopilot-misi-refused = Autopilot: perhitungan Misiurewicz ditolak (pasangan tidak valid)
autopilot-misi-stopped = Autopilot: pemecah Misiurewicz berhenti secara tak terduga
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-diving =
    Autopilot: menyelam ke Misiurewicz ({ $preperiod },{ $period })
    Masukan apa pun menghentikannya
# ($preperiod,$period) names the point: keep the notation. $octaves: how often the structure repeats (2.37); $twist: its turn in degrees, signed (+12.5).
autopilot-misi-diving-repeat =
    Autopilot: menyelam ke Misiurewicz ({ $preperiod },{ $period })
    Struktur berulang setiap { $octaves } oktaf, puntiran { $twist }° · masukan apa pun menghentikannya
# $limit: the magnification, as written (1e300).
autopilot-dive-limit = Autopilot: batas selam tercapai (~{ $limit }×)
# $have: the fixed iteration count; $typical: what this depth typically needs.
autopilot-dead-end-short =
    Autopilot berhenti: tidak ada lagi yang dapat dituju — iterasi ditetapkan pada { $have } dan kedalaman ini biasanya memerlukan ~{ $typical }
    Aktifkan iterasi otomatis atau naikkan jumlahnya
# $limit: the fixed iteration count.
autopilot-dead-end-cap =
    Autopilot berhenti: tidak ada lagi yang dapat dituju — semua yang ada di tampilan mencapai batas iterasi tetap { $limit }
    Aktifkan iterasi otomatis atau naikkan jumlahnya
autopilot-dead-end = Autopilot: tidak ada detail di depan (berhenti)

## Tours (camera scripts)

# The name of the file type in the file dialog's filter list.
script-file-filter = Skrip Fractadyne (TOML)
# $name: the script's name.
script-finished = Skrip selesai — “{ $name }”
# The title of the notice that follows (the file's path and the error, in English).
script-load-failed = Tidak dapat memuat skrip

## Installing a language file

# The name of the file type in the file dialog's filter list.
language-file-filter = Berkas bahasa Fractadyne

## The notice when the graphics driver's shader cache stops keeping shaders. The text is shown in a fixed-width font with its line breaks as written: break the lines where they read well.

driver-cache-title = Driver grafis tidak menyimpan shader
# $count: how many shaders compiled again; $pipeline: the slowest one's name (a technical id: keep it); $seconds: its time in seconds (7.5).
driver-cache-body =
    Driver grafis Anda mengompilasi ulang { $count } shader Fractadyne pada
    sesi ini, padahal sebelumnya setiap shader itu sudah pernah dikompilasi
    (yang paling lambat: { $pipeline }, { $seconds } s). Cache shader-nya
    tidak menyimpan shader yang sudah dikompilasi, paling sering karena
    cache itu penuh. Selama hal itu belum berubah, ekspor, tampilan dalam,
    dan peluncuran Fractadyne masing-masing dapat menghabiskan beberapa
    detik untuk kompilasi.
# $folder: the cache folder's path; $size: its size in MB (1024). "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia-folder =
    Untuk memperbaikinya: tutup Fractadyne, lalu hapus isi folder
      { $folder }   ({ $size } MB)
    Driver akan membangun ulang cache itu. Agar tidak penuh lagi, naikkan
    NVIDIA Control Panel > Manage 3D settings > Shader Cache Size.
# "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia =
    Untuk memperbaikinya: naikkan NVIDIA Control Panel > Manage 3D settings >
    Shader Cache Size, atau kosongkan cache shader milik driver.
# "AMD Software > Graphics > Advanced > Reset Shader Cache" names AMD's own settings: use the names AMD's software shows in this language.
driver-cache-fix-amd = Untuk memperbaikinya: AMD Software > Graphics > Advanced > Reset Shader Cache.
driver-cache-fix-other = Untuk memperbaikinya: kosongkan cache shader milik driver grafis.

## Why a language file could not be installed: each ends the sentence "{ $file } is not a usable language file: …".

language-install-not-utf8 = bukan teks UTF-8
language-install-too-large = terlalu besar
# "fractadyne-<language>.ftl" is the file name to use: keep it as written.
language-install-no-tag = berkas itu tidak menyebutkan bahasanya (beri nama fractadyne-<language>.ftl)
language-install-english = bahasa Inggris sudah bawaan
language-install-empty = berkas itu tidak menerjemahkan satu pun pesan versi ini
language-install-no-folder = tidak ada folder pengaturan

## The notice after a session that did not end cleanly, when its iteration setting was set aside.

# $base: the iteration count automatic iterations start from.
quarantine-setting-auto = iterasi otomatis dari dasar { $base }
# $count: the fixed iteration count.
quarantine-setting-fixed = { $count } iterasi
# $end: "device" when the session lost its graphics device, "unclean" otherwise (the variants hold the wording). $depth: the magnification, as written (1.2e148). $setting and $restored: an iteration setting (quarantine-setting-auto or -fixed).
quarantine-notice =
    { $end ->
        [device] Sesi sebelumnya kehilangan perangkat grafisnya pada { $depth }× saat diatur ke { $setting }. Sesi ini dimulai dengan { $restored } sebagai gantinya, agar tidak langsung kembali ke beban yang sama.
       *[other] Sesi sebelumnya tidak berakhir dengan benar pada { $depth }× saat diatur ke { $setting }. Sesi ini dimulai dengan { $restored } sebagai gantinya, agar tidak langsung kembali ke beban yang sama.
    }

## The second graphics card's status line, under its setting in the control panel.

gpu2-state-off = Nonaktif: kartu grafis jendela ini merender semuanya.
gpu2-state-starting = Memulai…
# $name: the card's name (and its graphics API).
gpu2-state-running = Digunakan: { $name }
# $why: the error, as the graphics driver reports it.
gpu2-state-failed = Tidak dapat dimulai: { $why }
# $name: the card's name.
gpu2-state-lost = Berhenti: { $name } hilang. Mulai sekarang satu kartu saja; pilih lagi untuk mencoba ulang.
# $state: one of the lines above. "--worker-gpu" is a command-line option: keep it.
gpu2-state-pinned = { $state } (diatur oleh --worker-gpu)

## The custom-formula dialog: how deep the formula renders (one line under the formula).

formula-depth-deep = Zoom dalam dengan perturbasi, rentang diperluas setelah 1e28x seperti pada fraktal bawaan (setiap langkah diiterasi: belum ada aproksimasi deret atau BLA untuk rumus kustom).
# $why: what the formula has that keeps it off the deep-zoom path (one of the formula-depth-why messages below).
formula-depth-direct = Render langsung ({ $why } belum memiliki bentuk zoom dalam): tajam hingga satu langkah presisi tunggal dari c mencakup satu piksel — sekitar 1e4x hingga 1e5x, dan lebih dangkal jika jauh dari titik asal.
formula-depth-why-init = bagian init
formula-depth-why-bailout = bailout tersendiri
formula-depth-why-variable = variabel yang dipertahankan dari langkah ke langkah
formula-depth-why-previous = iterasi sebelumnya
formula-depth-why-comparison = perbandingan atau blok if
# floor, ceil, trunc and round are functions of the formula language: keep them.
formula-depth-why-rounding = pembulatan (floor, ceil, trunc, round)
# "maxit" is a word of the formula language: keep it.
formula-depth-why-maxit = maxit
formula-depth-why-power = pangkat yang eksponennya bervariasi
formula-depth-why-perturbed = program yang sudah diperturbasi

## Image and frame sizes: the presets described in words (the others are standard names: HD 720p, WUXGA, DCI 4K).

# $size: the pixel size (2048×2048).
size-preset-square = Persegi — { $size }
# $ratio: the aspect ratio (9:16); $size: the pixel size.
size-preset-portrait = Potret { $ratio } — { $size }
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
fractal-family-custom = Kustom
# Conway's Game of Life.
fractal-family-life = Life
fractal-family-lsystem = Sistem-L
# A family at a power: "Multibrot 3", "Burning Ship 4". $family: a name above; $power: the power.
fractal-with-power = { $family } { $power }

## Names of choices: colouring methods, orbit traps, palette animation, snapshots, angle units.

color-method-smooth = Iterasi halus
color-method-stripe = Rata-rata garis
color-method-triangle = Ketaksamaan segitiga
color-method-trap = Perangkap orbit
color-method-distance = Estimasi jarak
color-method-decomposition = Dekomposisi
# The shape an orbit trap catches the orbit with.
trap-point = Titik
trap-cross = Silang
trap-circle = Lingkaran
# Palette animation: none.
palette-anim-off = Mati
palette-anim-forward = Maju
palette-anim-reverse = Mundur
palette-anim-pingpong = Bolak-balik
# Palette animation: a new random gradient every so often.
palette-anim-random = Gradien acak
# What the Snapshot button does.
snapshot-mode-ask = Tanya setiap kali
snapshot-mode-screen = Tangkapan layar
snapshot-mode-render = Render penuh
# The unit of an angle typed into Go to location's polar fields.
angle-degrees = derajat
angle-radians = radian
angle-turns = putaran

## Report an issue: the choices. (The report itself is sent in English.)

issue-kind-crash = Aplikasi crash / macet
issue-kind-rendering = Render tidak benar
issue-kind-performance = Masalah kinerja
issue-kind-ui = Masalah UI / kegunaan
issue-kind-feature = Permintaan fitur
issue-kind-other = Lainnya
severity-low = Rendah
severity-medium = Sedang
severity-high = Tinggi
severity-blocking = Memblokir
repro-always = Selalu
repro-sometimes = Kadang-kadang
repro-once = Terjadi sekali
repro-cannot = Tidak dapat direproduksi

## The benchmark's depths and the diagnostics' tests.

# $zoom: the dive's final magnification (1e32).
bench-depth-standard = Standar ({ $zoom }×)
# $zoom: the dive's final magnification (1e48).
bench-depth-ultra = Ultra dalam ({ $zoom }×)
diag-test-selftest = Uji mandiri
diag-test-uitest = Uji UI
diag-test-gputest = Pemeriksaan aritmetika GPU

## The status bar's readouts: each word is followed by a value in a slot of fixed width. Keep the words short (one word each).

status-center = pusat
status-cursor = kursor
status-zoom = zoom
# Short for iterations.
status-iter = iter
# The period of the minibrot jumped to.
status-period = periode
# A Game of Life view's scale (followed by "1 px = 4 cells" or "8 px / cell").
status-scale = skala
# A Game of Life view's generation. Short.
status-generation = gen
# A Game of Life view's population. Short.
status-population = pop
# An L-system's order (how many times its rules were applied).
status-order = orde
# How many segments and shapes an L-system drew.
status-drawn = tergambar
# $value: how many cells one pixel covers (4, 9.99e300). "px" is the unit symbol: keep it.
life-scale-cells = 1 px = { $value } sel
# $value: how many pixels one cell covers (8). "px" is the unit symbol: keep it.
life-scale-pixels = { $value } px / sel

## The status bar's limit diagnostic: a short label (after a warning sign), and what it means when hovered.

# The view is as deep as this location can be rendered. Short.
status-limit-depth = batas kedalaman
# Every pixel ran out of iterations. Short.
status-limit-exhausted = iter habis
# Pixels are capped at the iteration budget. Short.
status-limit-capped = iter terbatas
# $pixels: how many pixels wide one step of the coordinates is; $sharp_to: the magnification the view is sharp to (written out, 1,234,567).
status-limit-direct-detail = Tampilan ini dirender langsung di GPU dalam presisi tunggal, dan di sini satu langkah koordinat selebar { $pixels } piksel: piksel-piksel yang bersebelahan menghitung titik yang sama, sehingga gambar pecah menjadi kotak-kotak. Di lokasi ini gambar tajam hingga sekitar { $sharp_to }×. Rumus ini belum memiliki jalur zoom dalam (perturbasi), jadi zoom lebih jauh tidak menampilkan detail tambahan.
# $samples: the GPU buffer's capacity in orbit samples; $iterations: the iterations pixels are limited to.
status-limit-depth-detail = Orbit referensi mencapai batas buffer GPU ({ $samples } sampel) tanpa lolos, sehingga piksel di sini dibatasi hingga ~{ $iterations } iterasi dan detail yang lebih halus tidak dapat dihitung pada tampilan ini — baik di tampilan langsung maupun saat diekspor. Ini adalah batas kedalaman saat ini untuk keluarga lokasi ini.
# $budget: the largest iteration budget this view can be given.
status-limit-exhausted-detail = Setiap piksel tetap kehabisan iterasi pada anggaran terbesar yang dapat diberikan untuk tampilan ini ({ $budget }). Entah tampilan ini memang berada di dalam himpunan di sini (wajar pada kedalaman), atau tampilan ini memerlukan lebih banyak iterasi daripada yang saat ini dapat disediakan aplikasi — gambarnya sama saja dalam kedua kasus, jadi anggarannya dikembalikan ke nilai yang murah.
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations" is the setting's name in the control panel's Quality section.
status-limit-capped-max = { $percent }% piksel kehabisan iterasi pada anggaran { $budget }, dan piksel yang lolos selesai tepat di batas atas itu — tampilan kurang terselesaikan. Iterasi sudah pada nilai maksimum, jadi tampilan ini memerlukan lebih dari yang saat ini dapat diberikan aplikasi.
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations", "Controls" and "Quality" are the names of a setting, the control panel and its section.
status-limit-capped-raise = { $percent }% piksel kehabisan iterasi pada anggaran { $budget }, dan piksel yang lolos selesai tepat di batas atas itu — tampilan kurang terselesaikan. Naikkan Iterasi (Kontrol ▸ Kualitas) untuk menyelesaikannya.
