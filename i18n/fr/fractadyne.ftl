### Fractadyne user interface: French (Français).
### A draft: machine translation (Claude, by Anthropic) of the English file, 2026-10-23, not yet
### reviewed by a native speaker. Corrections are welcome as pull requests (see i18n/README.md).

## Language data: read by Fractadyne, not shown as messages.

# This language's own name, shown in the language menu.
language-name = Français
# The same name in English, shown beside it in the menu.
language-english-name = French
# The writing system the app must be able to draw (an ISO 15924 code: Latn, Cyrl, Jpan, Hans, Khmr).
language-script = Latn
# Punctuation for counts, durations and sizes. Never used for coordinates or magnifications.
# One character each. A space as the group separator is written as a quoted literal, so it shows:
# { "\u00A0" } (no-break space) or { "\u202F" } (narrow no-break space).
number-decimal = ,
number-group = { "\u202F" }
# Whether a native speaker has reviewed the translation: draft (the language menu says so) or
# reviewed. The English file is the source.
language-status = draft

## The language menu (top bar) and the Languages window

# The language menu's last item: opens the Languages window.
language-more = Plus de langues…
# The Languages window's title.
language-window-title = Langues
# The Languages window's introduction.
language-window-intro = L’interface de Fractadyne dans une autre langue. Une langue tient en un seul fichier : il est fourni avec une version, ou vous l’installez ici. Les coordonnées, les fichiers et la ligne de commande restent en anglais dans toutes les langues.
# Column headers of the installed-language list.
language-col-language = Langue
language-col-translated = Traduit
language-col-from = Provenance
# Where a language comes from.
language-source-builtin = intégrée
language-source-package = cette version
language-source-user = installée
language-source-dev = dossier de travail
# $percent: how much of the interface the file translates, 0–100.
language-coverage = { $percent } %
# $count: messages in the file that failed the checks and show in English instead.
language-dropped =
    { $count ->
        [one] { $count } message à vérifier
       *[other] { $count } messages à vérifier
    }
# Buttons of the Languages window.
language-install = Installer un fichier de langue…
language-install-hover = Choisir un fichier de langue Fractadyne (.ftl). Il est vérifié, puis copié dans le dossier des langues de vos paramètres, où il est conservé d’une mise à jour à l’autre.
language-open-folder = Ouvrir le dossier des langues
language-reload = Recharger
language-reload-hover = Relire les fichiers de langue (après en avoir modifié ou ajouté un).
language-use = Utiliser
# Notices after installing a file. $name: the language's own name; $file: the file's name.
language-installed = Langue { $name } installée depuis { $file }.
# $file: the file's name; $why: what was wrong with it.
language-install-failed = { $file } n’est pas un fichier de langue utilisable : { $why }
# Shown when a language file cannot be loaded and English is used instead. $name: the language.
language-load-failed = La langue { $name } n’a pas pu être chargée, Fractadyne est donc en anglais : { $why }
# A language whose writing system needs a font this computer does not have (Japanese or Chinese without a system font). $name: the language, in its own name.
language-font-missing = Aucune police capable d’afficher { $name } n’a été trouvée sur cet ordinateur : cette langue ne peut donc pas s’afficher. Installez une police pour { $name } (par exemple le module de langue du système), puis choisissez de nouveau cette langue.
# Shown after a draft translation's name in the language menu and the Languages window. Short.
language-draft = brouillon
language-draft-hover = Un brouillon : cette traduction n’a pas encore été relue par un locuteur natif. Les corrections sont les bienvenues, sous forme de pull request sur le fichier de la langue.
# The heading over the decimal-mark choice, in the language menu and the Languages window.
language-decimal-mark = Séparateur décimal
language-decimal-mark-hover = La façon d’écrire les nombres : durées, tailles et décomptes. Les coordonnées et les grossissements utilisent toujours un point, comme les lisent les fichiers et les autres programmes.
# $sample: a number written with this language's own punctuation (1,234.5).
language-decimal-language = Comme l’écrit la langue ({ $sample })
# $sample: a number written with a decimal point (1,234.5).
language-decimal-point = Point ({ $sample })
# $sample: a number written with a decimal comma (1.234,5).
language-decimal-comma = Virgule ({ $sample })

## Asked once when the system's language changes

# The window's title.
language-changed-title = Langue
# $new: the system's new language, in its own name.
language-changed-question = La langue de votre système est désormais { $new }. L’utiliser dans Fractadyne ?
# Buttons. $name: a language's own name.
language-changed-use = Utiliser { $name }
language-changed-keep = Garder { $name }

## The welcome window (first run)

welcome-title = Bienvenue dans Fractadyne
welcome-intro = Un explorateur à zoom profond pour l’ensemble de Mandelbrot et ses cousins. Quelques commandes pour bien démarrer :
# The language row, at the top of the window.
welcome-language = Langue
# Controls: the key or gesture on the left, what it does on the right.
welcome-key-drag = glisser
welcome-key-drag-does = déplacer la vue
welcome-key-scroll = molette
welcome-key-scroll-does = zoom avant / arrière vers le curseur
welcome-key-space = maintenir Space
welcome-key-space-does = zoom continu et fluide (Shift+Space en arrière)
# Shown after the click-zoom toolbar icon.
welcome-key-click = puis cliquer
welcome-key-click-does = plonger vers le point cliqué (barre d’outils)
welcome-key-m-does = sauter au minibrot le plus proche, à sa propre échelle
welcome-landmarks = Pour commencer, aller à un repère :
welcome-theme = Thème
welcome-watermark = Afficher la marque « Fd »
welcome-watermark-hover = La petite marque en bas à droite de la vue. Désactivée, elle disparaît de la vue en direct et des images exportées.
welcome-updates = Mises à jour
welcome-check-updates = Rechercher des mises à jour
welcome-check-updates-hover = Demande à GitHub la dernière version au démarrage de l’application. Rien n’est envoyé.
welcome-files = Fichiers
welcome-no-config = (aucun dossier de configuration)
welcome-open = Ouvrir
welcome-rendering-on = Rendu sur
welcome-open-help = Ouvrir l’aide complète  (F1)
# $icon: the Settings icon.
welcome-more-in-settings = Plus d’options dans Fichier → { $icon } Paramètres
welcome-get-started = Commencer

## Menus: shared

# A percentage in a menu (UI scale). $value: the number.
common-percent = { $value } %
# A state shown after a menu item's name, in brackets.
menu-state-connected = connecté
menu-state-rendering = rendu en cours
menu-state-running = en cours

## Navigate ▸ Bookmarks (also the toolbar's star)

menu-bookmarks = Signets
menu-bookmarks-add = Ajouter la vue actuelle
menu-bookmarks-manage = Gérer…

## The fractal picker (Fractal menu and the toolbar's dropdown)

menu-family-powers = Puissances supérieures
menu-family-automata = Automates
menu-family-lsystems = L-systèmes
menu-fractal-custom = Formule personnalisée…
menu-fractal-library = Bibliothèque de formules…
menu-fractal-library-hover = Formules personnalisées enregistrées : les appliquer, les modifier, les importer et les exporter
# The "Show" group: which picture of the formula is drawn. $name: the formula's name (Mandelbrot, Burning Ship…).
menu-show = Afficher
menu-show-set = Ensemble de { $name }
menu-show-set-hover = Le plan des paramètres — l’ensemble de { $name } lui-même.

    Chacun de ses points correspond à un ensemble de Julia différent ; c’est la carte de tous ces ensembles.
menu-show-julia = Ensemble de Julia
menu-show-julia-hover = Un point de l’ensemble de { $name }, dessiné comme une image à part entière — la forme obtenue en gardant ce point fixe.

    Choisissez le point en survolant la carte en mode « Les deux, liés ». Un clic sur la carte le fixe.
menu-show-julia-none = { $name } n’a pas de paramètre libre, donc pas d’ensemble de Julia.
menu-show-both = Les deux, liés
menu-show-both-hover = L’ensemble de { $name } à gauche et, à droite, l’ensemble de Julia du point que survole votre curseur.

    Déplacez le curseur sur la carte et regardez l’ensemble de Julia changer — c’est le moyen le plus rapide de voir le lien entre les deux.
menu-show-both-none = { $name } n’a pas d’ensemble de Julia à associer.

## File menu

menu-file = Fichier
menu-file-open-view = Ouvrir une vue ou un emplacement…
menu-file-gallery = Galerie…
menu-file-export = Exporter l’image…
menu-file-snapshot = Instantané
menu-file-open-life = Ouvrir un motif Life…
menu-file-open-life-hover = Un fichier RLE, .cells ou Life 1.05/1.06, affiché comme un univers Life.
menu-file-life-text = Texte d’un motif Life…
menu-file-life-text-hover = Coller un motif sous forme de texte (du RLE, tel que le donne LifeWiki).
menu-file-save-life = Enregistrer le motif Life…
menu-file-save-life-hover = L’univers dans son état actuel, au format RLE.
menu-file-open-lsystem = Ouvrir un L-système…
menu-file-open-lsystem-hover = Un fichier .l de Fractint (avec ses entrées au choix) ou un système .lsys.
menu-file-save-lsystem = Enregistrer le L-système…
menu-file-save-lsystem-hover = Le texte du système, sous forme de fichier .lsys.
menu-file-export-svg = Exporter en SVG…
menu-file-export-svg-hover = La vue sous forme de dessin SVG : ses lignes et ses formes en vectoriel, dans ses couleurs.
menu-file-render-client = Client de rendu…
menu-file-render-client-hover = Prêter le GPU de cette machine à une ferme de rendu : se connecter à un contrôleur (Outils ▸ Rendre sur la ferme… sur une autre machine) et calculer des images pour lui.
menu-file-settings = Paramètres
menu-file-reset = Réinitialiser l’état de l’application…
menu-file-reset-hover = Supprimer tout l’état enregistré (session, signets, miniatures) et repartir de zéro. Une confirmation est d’abord demandée.
menu-file-quit = Quitter

## File ▸ Settings

menu-settings-fps = Limite d’images par seconde
menu-settings-fps-uncapped = Illimitée
# $fps: frames per second.
menu-settings-fps-n = { $fps } FPS
menu-settings-ui-scale = Échelle de l’interface (taille du texte)
menu-settings-theme = Thème
menu-settings-updates = Mises à jour
menu-settings-updates-stable-hover = Dernière version stable
menu-settings-updates-beta-hover = Dernière version, préversions comprises
menu-settings-updates-on-launch = Rechercher des mises à jour au démarrage
menu-settings-updates-on-launch-hover = Sinon, vérifiez manuellement via Aide → Rechercher des mises à jour.
# Followed by its shortcut, (Ctrl+S).
menu-settings-snapshot = Instantané
menu-settings-snapshot-ask-hover = Proposer le choix à chaque appui
menu-settings-snapshot-screen-hover = Enregistrer la vue exactement telle qu’elle apparaît à l’écran (immédiat, PNG)
menu-settings-snapshot-render-hover = Un rendu complet avec les paramètres d’exportation, en arrière-plan (peut être long sur les vues profondes)
menu-settings-refcache = Cache des références…
menu-settings-refcache-hover = Les orbites de référence du zoom profond, conservées sur le disque pour qu’un emplacement où vous revenez se rende en quelques secondes. Son emplacement, sa taille, sa limite, et un moyen de le vider.

## Fractal menu

menu-fractal = Fractale

## View menu

menu-view = Affichage
menu-view-zoom-home = Dézoomer jusqu’à la vue complète
menu-view-zoom-home-hover = Dézoomer jusqu’à la vue d’accueil complète (animé)
menu-view-reset = Revenir à la vue par défaut
menu-view-reset-hover = Revenir à la vue par défaut de la fractale (immédiat)
# Followed by its shortcuts.
menu-view-undo = Vue précédente
menu-view-redo = Vue suivante
menu-view-control-panel = Panneau de contrôle
menu-view-control-panel-hover = Afficher ou masquer le panneau de contrôle de droite.
menu-view-minimap = Mini-carte
menu-view-minimap-hover = Afficher un petit aperçu de la vue d’accueil, avec un repère « vous êtes ici » et la profondeur de zoom. Cliquez dessus pour sauter vers une région.
menu-view-perf-panel = Panneau de performances
menu-view-timestamp = Afficher l’horodatage
menu-view-timestamp-hover = Dessiner une grande horloge du temps écoulé par-dessus la vue. Elle affiche le même « +12.345s » que celui dont le journal horodate chaque ligne : un enregistrement d’écran d’un problème qui ne se manifeste qu’en mouvement peut ainsi être aligné sur le journal, image par image. Désactivé par défaut.
menu-view-zoom-target = Afficher la cible du zoom automatique
menu-view-zoom-target-hover = Pendant le zoom automatique, entourer la région de la vue vers laquelle il zoome — la partie qui remplira l’écran après quelques grossissements de plus — avec des lignes reliant ses coins à ceux de l’écran, et un anneau sur le point vers lequel il se dirige. Désactivé par défaut.
menu-view-live-panel = Panneau de rendu en direct
menu-view-live-panel-hover = Un panneau frontal par-dessus la vue, qui montre le moteur de rendu en direct au travail : ce que chaque image a envoyé, l’âge de l’image affichée, la progression à travers les bandes d’itérations, les constructeurs de références et la machine sous-jacente. B le fait passer d’une taille à l’autre, et le × dans son coin le ferme ; faites-le glisser dans n’importe quel coin ; survolez un instrument pour savoir ce qu’il signifie et à quoi ressemblent un fonctionnement sain et un fonctionnement perturbé.
# The live render panel's sizes.
menu-view-live-off = Désactivé
menu-view-live-off-hover = Aucun panneau.
menu-view-live-strip = Bandeau
menu-view-live-strip-hover = Les afficheurs nixie et la rangée de voyants.
menu-view-live-panel-size = Panneau
menu-view-live-panel-size-hover = Les instruments : cadran de progression, âge de l’image, tuiles, passes GPU, cadran de profondeur, voyants.
menu-view-live-full = Complet
menu-view-live-full-hover = Le panneau et la salle des machines : pas, GPU, mémoire, cœurs, pixels.
menu-view-live-opacity = Opacité
menu-view-watermark = Afficher le filigrane « Fd »
menu-view-watermark-hover = Dessiner une discrète marque « Fd » dans le coin inférieur droit de la vue en direct et des images exportées. Activé par défaut.

## Color menu

menu-color = Couleur
menu-color-method = Méthode
menu-color-palette = Palette
menu-color-custom = Personnalisée
menu-color-duotone = Bichromie
menu-color-binary = Binaire (ensemble)
menu-color-binary-hover = Deux couleurs unies : dans l’ensemble ou hors de l’ensemble, sans dégradé.
menu-color-edit-gradient = Modifier le dégradé…
menu-color-normalize = Normaliser les couleurs profondes
menu-color-normalize-hover = Recaler la palette sur la plage d’échappement mesurée de la vue lorsqu’un champ dense apparaîtrait sinon comme des mouchetures. Une vue sans crénelage est laissée telle quelle — cochez « Toujours ajuster la plage » pour recaler dans tous les cas. Méthode « Itération lissée » uniquement.
menu-color-more = Cycle, décalage, animation et effets : panneau de contrôle ▸ Coloration

## Tools menu

menu-tools = Outils
# Followed by its shortcut, (M).
menu-tools-find-minibrot = Trouver un minibrot et zoomer dessus
menu-tools-find-minibrot-hover = Caler par la méthode de Newton le centre de la vue sur le centre exact du minibrot voisin, indiquer sa période et zoomer à l’échelle propre du minibrot — souvent de nombreux ordres de grandeur d’un seul coup. Déjà plus profond que lui ? Seul le centre se déplace.
menu-tools-solver = Solveur Newton / Misiurewicz…
menu-tools-solver-hover = Calculer le centre exact du minibrot (période + taille) ou le point de Misiurewicz le plus proche de la vue — méthode de Newton en pleine précision, avec le multiplicateur répulsif λ. Ouvre la fenêtre des emplacements, qui contient les solveurs.
# Followed by its shortcut, (A).
menu-tools-autopilot = Zoom automatique (pilote automatique)
menu-tools-autopilot-stop = Arrêter le pilote automatique
menu-tools-autopilot-hover = Zoom profond continu, sans les mains : plonge vers la région la plus riche en détails, en corrigeant sa trajectoire en chemin. Toute action de navigation l’arrête.
menu-tools-priority = Priorité :
menu-tools-priority-speed = Vitesse
menu-tools-priority-speed-hover = Zoomer à la vitesse réglée ; rafraîchir dès la première image qui a du détail.
menu-tools-priority-quality = Qualité
menu-tools-priority-quality-hover = Uniquement des images entièrement résolues ; le zoom les attend.
menu-tools-target = Cible :
menu-tools-target-detail = Détail
menu-tools-target-detail-hover = Le bord le plus riche de la vue, choisi de nouveau au fil de la plongée.
menu-tools-target-misiurewicz = Point de Misiurewicz
menu-tools-target-misiurewicz-hover = Le point de Misiurewicz le plus proche, calculé une fois puis atteint exactement par la plongée (Mandelbrot uniquement).
menu-tools-orbits = Superposition de l’orbite
menu-tools-orbits-hover = Dessiner le chemin d’itération du point sous le curseur. Options d’animation : panneau de contrôle ▸ Superpositions.
menu-tools-orbits-normalize = Normaliser (ajuster à la vue)
menu-tools-orbits-normalize-hover = Ajuster l’orbite à toute la vue pour qu’elle reste bien cadrée à n’importe quel zoom (au lieu de la projeter à travers la fenêtre d’affichage, d’où elle s’échappe hors de l’écran en profondeur).
menu-tools-play-tour = Lire un parcours…
menu-tools-play-tour-hover = Choisir un script de parcours à lire. Le ▶ de la barre d’outils relit le dernier d’un seul clic.
menu-tools-tour-from-view = Parcours depuis la vue actuelle…
menu-tools-tour-from-view-hover = Créer un parcours qui zoome de la vue complète jusqu’à la vue actuelle — ajoutez une légende et réglez la durée de la plongée. Enregistré comme script de parcours (.toml) que vous pouvez modifier, lire ou rendre.
menu-tools-render-tour = Rendre un parcours…
menu-tools-render-tour-hover = Rendre le parcours chargé en une séquence d’images PNG (et, en option, une vidéo mp4) dans un processus séparé.
menu-tools-close-player = Fermer le lecteur de parcours
menu-tools-close-player-hover = Arrêter le parcours, fermer son lecteur et rétablir vos propres réglages d’itérations et de coloration
menu-tools-render-farm = Rendre sur la ferme…
menu-tools-render-farm-hover = Rendre le parcours chargé sur plusieurs machines : celle-ci pilote la tâche, les autres la rejoignent avec Fichier ▸ Client de rendu….
menu-tools-benchmark = Banc d’essai…
menu-tools-benchmark-hover = Mesurer la vitesse de rendu — avec les réglages actuels, ou par une exécution standardisée (résolution et réglages fixes) comparable d’une machine à l’autre. Le test d’endurance la répète pour vérifier la stabilité / la limitation thermique.
menu-tools-last-benchmark = Afficher le dernier banc d’essai

## Navigate menu

menu-navigate = Navigation
menu-navigate-goto = Aller à l’emplacement…
menu-navigate-goto-hover = Saisir un centre et un zoom, sauter vers un point connu, ou calculer par Newton un point de Misiurewicz ou un minibrot près de la vue.
menu-navigate-misiurewicz = Explorateur de Misiurewicz…
menu-navigate-misiurewicz-hover = Voir à quoi ressemblent les petits types de Misiurewicz — des miniatures avec la période de zoom et la torsion de chaque point — et calculer par Newton n’importe lequel d’entre eux jusqu’à une profondeur arbitraire. (Mandelbrot.)
menu-navigate-exact = Points exacts (expressions)
menu-navigate-exact-note = Racines des bulbes de la cardioïde principale, données sous forme d’expressions exactes (cos/sin d’une fraction de 2π). La coordonnée est conservée sous forme d’expression : le zoom reste donc exactement sur le point.
menu-navigate-random = Emplacement aléatoire
menu-navigate-random-hover = Sauter vers un point aléatoire de la frontière, riche en détails
menu-navigate-import-kfr = Importer un .kfr…
menu-navigate-import-kfr-hover = Charger un fichier d’emplacement Kalles Fraktaler
menu-navigate-share = Partager l’emplacement…
menu-navigate-share-hover = Copier / coller / enregistrer / charger un emplacement autonome (.fdn) : fractale, centre en pleine précision, zoom, coloration.

## Help menu

menu-help = Aide
# Followed by its shortcut, (F1).
menu-help-reference = Aide et référence…
menu-help-welcome = Bienvenue / prise en main…
menu-help-welcome-hover = L’aperçu du premier lancement : les commandes essentielles et quelques repères.
menu-help-diagnostics = Diagnostics…
menu-help-diagnostics-hover = Vérifier que Fractadyne fonctionne correctement sur votre matériel, et joindre le résultat à un signalement de problème
menu-help-report = Signaler un problème…
menu-help-report-hover = Ouvrir un ticket GitHub (ou envoyer un e-mail) avec un rapport de plantage, le journal et l’emplacement en pièces jointes
menu-help-accelerated = Zoom profond plus rapide…
menu-help-accelerated-hover = Une version optionnelle qui calcule les orbites de référence du zoom profond 2,5 à 6,4 fois plus vite. Images identiques ; vos réglages sont conservés.
menu-help-check-updates = Rechercher des mises à jour
# $track: the update track (stable or beta).
# $track: "stable" or "beta", the update track in use (the variants hold the wording).
menu-help-check-updates-hover =
    { $track ->
        [beta] Chercher sur GitHub une version bêta (préversions) plus récente
       *[other] Chercher sur GitHub une version stable plus récente
    }
# $version: the new version; $channel: "beta" for a pre-release, "stable" otherwise (the variants hold the wording).
menu-help-update-available =
    { $channel ->
        [beta] { $version } (bêta) disponible — télécharger
       *[other] { $version } (stable) disponible — télécharger
    }
menu-help-up-to-date = À jour
menu-help-tagline = Explorateur de fractales natif
menu-help-license = Licence
# $year and $holder: the copyright year and holder, as the code gives them.
menu-help-copyright = © { $year } { $holder }. Double licence ; utilisation sous l’une ou l’autre licence, à votre choix.
menu-help-source = Code source

## The toolbar (its buttons are icons; these are their tooltips)

menu-toolbar-julia = Julia
menu-toolbar-julia-hover = Afficher l’ensemble de Julia de cette formule, en remplissant la fenêtre
menu-toolbar-dual-hover = Les deux, liés : l’ensemble à gauche, l’ensemble de Julia du point sous le curseur à droite
menu-toolbar-open-hover = Ouvrir une vue ou un emplacement (PNG / EXR, .fdn, .kfr)
menu-toolbar-gallery-hover = Galerie
menu-toolbar-export-hover = Exporter l’image…
menu-toolbar-share-hover = Partager l’emplacement — copier, coller, enregistrer ou charger un .fdn autonome (fractale, centre en pleine précision, zoom, coloration)
menu-toolbar-zoom-in-hover = Zoom avant
menu-toolbar-zoom-out-hover = Zoom arrière
# $factor: the click-zoom factor (4, 10…).
menu-toolbar-click-zoom-hover = Cliquer pour zoomer ({ $factor }×) : le clic gauche plonge vers le point, le clic droit recule (glisser déplace toujours la vue). Facteur dans Contrôles ▸ Navigation.
# Added to the click-to-zoom tooltip in the dual view.
menu-toolbar-click-zoom-dual = Épinglez le Julia c avec Ctrl+clic tant que ce mode est actif.
menu-toolbar-autopilot-running-hover = Le zoom automatique est en cours — cliquer pour l’arrêter
menu-toolbar-autopilot-hover = Zoom automatique : plonger vers le détail (A)
menu-toolbar-reset-hover = Réinitialiser la vue (immédiat)
menu-toolbar-home-hover = Dézoomer jusqu’à la vue d’accueil (animé)
menu-toolbar-bookmarks-hover = Signets : enregistrer cette vue, ou sauter vers une vue enregistrée
# $name: the palette's name.
menu-toolbar-next-palette-hover = Palette suivante ({ $name })
# $file: the last tour's file name.
menu-toolbar-play-last-hover = Lire le parcours — { $file } (clic) · relit le dernier parcours ; en choisir un autre via Outils ▸ Lire un parcours…
menu-toolbar-play-hover = Lire un parcours… (Outils ▸ Lire un parcours)
menu-toolbar-speed-hover = Vitesse de lecture
menu-toolbar-fullscreen-hover = Basculer en plein écran

## Dialogs: words shared by many windows

# A button that closes a window without doing what it offers.
dlg-cancel = Annuler
# A button that dismisses a window (nothing in it is pending).
dlg-close = Fermer
# A button that copies the window's text to the clipboard (after a copy icon).
dlg-copy = Copier
# A button that jumps to a location.
dlg-go = Aller
# A button that fills the fields from the view on screen.
dlg-use-current = Utiliser la vue actuelle
# A notice after a file could not be written. $error: the system's reason, as it gives it.
dlg-save-failed = Échec de l’enregistrement : { $error }
# A notice when no graphics device is available to render with.
dlg-gpu-not-available = GPU non disponible
# The name of the plain-text file type in a save dialog (the extension .txt is added by the code).
dlg-filter-text = Texte

## Asked after a session that did not shut down cleanly

dlg-crash-title = Fractadyne ne s’est pas fermé correctement
dlg-crash-intro = La session précédente s’est terminée de façon inattendue. Un rapport a été enregistré sur cette machine — voulez-vous l’envoyer ?
dlg-crash-review = Rien n’est envoyé avant que vous l’ayez relu : l’écran suivant montre exactement ce qui serait inclus, et une seule case à cocher suffit pour exclure les informations système.
dlg-crash-report = Le signaler…
dlg-crash-report-hover = Ouvre « Signaler un problème », avec en pièces jointes le rapport de plantage et l’emplacement où le plantage s’est produit.
dlg-crash-not-now = Pas maintenant
dlg-crash-dont-ask = Ne plus demander après un plantage
dlg-crash-dont-ask-hover = Les rapports sont toujours enregistrés localement, et Aide ▸ Signaler un problème fonctionne en permanence.

## Asked after a session that ended with a second graphics device in use

dlg-second-device-title = Désactiver le second périphérique graphique ?
# $device: the graphics device's name and details, as the system gives them.
dlg-second-device-intro = La session précédente s’est terminée de façon inattendue alors qu’un second périphérique graphique était utilisé ({ $device }). Il en est peut-être la cause.
dlg-second-device-note = S’il est désactivé, la carte qui dessine la fenêtre effectue tout le rendu, comme avant. Vous pouvez le réactiver dans Avancé ▸ Seconde carte graphique.
dlg-second-device-off = Le désactiver
dlg-second-device-keep = Le garder activé

## Shown when the previous session's iteration setting was set aside

dlg-quarantine-title = Réglage d’itérations mis de côté
dlg-quarantine-change = Vous pouvez le modifier à tout moment dans Panneau de contrôle ▸ Qualité.
dlg-quarantine-keep = Garder la valeur par défaut
# A button. $setting: the iteration setting the previous session used (such as "20,000 iterations").
dlg-quarantine-restore = Rétablir { $setting }

## Navigate ▸ Go to location

dlg-goto-title = Aller à l’emplacement
# Before two choices: "x, y" (cartesian) and the polar form below.
dlg-goto-enter-as = Saisir en :
# The polar way of entering a point: an offset plus a radius r at an angle θ. Keep the symbols.
dlg-goto-polar = décalage + r ∠ θ
# The tooltip of the "?" button beside the entry choice.
dlg-goto-help-hover = Ce que les champs acceptent : fractions, puissances, sqrt/sin/cos…, pi et d’autres constantes — ouvre Aide → Expressions de coordonnées
# Labels over the entry fields. x0, y0, r, θ, Re and Im are the fields' symbols: keep them.
dlg-goto-polar-offset = Décalage du centre  (x0, y0 — laisser 0 pour un point purement polaire)
dlg-goto-polar-radius = Rayon r, angle θ
dlg-goto-re = Centre — partie réelle (Re)
dlg-goto-im = Centre — partie imaginaire (Im)
dlg-goto-zoom = Zoom (grossissement)
# A warning under the zoom field, after a ⚠ sign. $have: the iteration count set; $typical: the count this depth typically needs.
dlg-goto-iter-warning = Le réglage Itérations est fixé à { $have } ; cette profondeur en demande généralement ~{ $typical }. La vue restera plate tant qu’il n’est pas augmenté — ou activez les itérations automatiques (Contrôles ▸ Qualité), qui s’adaptent d’elles-mêmes.
dlg-goto-copy-hover = Copier cet emplacement dans le presse-papiers
dlg-goto-cancel-hover = Fermer sans aller nulle part
# The expressions in brackets are examples of what can be typed: keep them as they are.
dlg-goto-paste-note = Collez un centre et un zoom venant de quelqu’un d’autre, ou copiez-les pour les partager. Les coordonnées acceptent des expressions : fractions exactes (-3/4), fonctions et constantes (-0.5 + 0.25*cos(pi/4), sqrt, root(x,n), ^ ; angles en radians), et le champ réel accepte une valeur complexe entière ((37+16i)/100), qui remplit les deux champs.
# A section that opens and closes.
dlg-goto-feature = Aller à une structure (Mandelbrot)
dlg-goto-feature-poi = Points connus :
# The dropdown's text before a point is picked.
dlg-goto-feature-poi-pick = Sauter vers un point d’intérêt…
dlg-goto-feature-solve = Ou calculer une structure près de la vue actuelle :
# Two choices: what to solve for. Misiurewicz is a mathematician's name (a Misiurewicz point).
dlg-goto-feature-misiurewicz = Misiurewicz
dlg-goto-feature-minibrot = Minibrot le plus proche
# Labels of two small number fields; k and p are their symbols.
dlg-goto-feature-preperiod = Prépériode k
dlg-goto-feature-period = Période p
# The faint text in an empty field: left empty, the value is found automatically.
dlg-goto-feature-auto-hint = auto
# A small button that empties both fields, so they are found automatically again.
dlg-goto-feature-auto = Auto
dlg-goto-feature-auto-hover = Vider les deux champs pour que la prochaine recherche détecte la prépériode et la période à partir de l’orbite au centre de la vue.
dlg-goto-feature-kp-hover = Laissez les deux vides pour les détecter à partir de l’orbite au centre de la vue ; les valeurs trouvées s’inscrivent ici. Personne ne connaît la prépériode d’une spirale en la regardant, et devoir en fournir une rendait ce solveur inutilisable en pratique.
# While a solve runs, followed by its elapsed time and the note below.
dlg-goto-feature-solving = Calcul en cours…
dlg-goto-feature-solving-note = (précision arbitraire ; les vues profondes prennent plus de temps)
dlg-goto-feature-cancel-hover = Ne plus attendre. Le calcul lui-même se poursuit jusqu’au bout en arrière-plan et sa réponse est ignorée — rien ne reste à moitié fait.
dlg-goto-feature-find = Chercher près de la vue
dlg-goto-feature-find-hover =
    Calculer par Newton, depuis le centre actuel, la structure exacte (précision arbitraire). Approchez-vous d’abord de la structure.

    Réglez le champ Zoom ci-dessus sur la profondeur VOULUE avant le calcul : le point est calculé avec la précision qu’exige cette profondeur, et la vue y saute directement. Un seul calcul, au lieu d’une longue suite de zooms manuels.
dlg-goto-solve-cancelled = Calcul annulé.

## Snapshot (the camera button): its tooltip, and the choice on the first press

# The camera button's tooltip, one per Snapshot setting. Followed by its shortcut, (Ctrl+S).
dlg-snapshot-hover-ask = Instantané — enregistrer la vue telle qu’affichée, ou un rendu complet ; le premier appui pose la question
dlg-snapshot-hover-screen = Instantané — enregistrer la vue exactement telle qu’affichée, dans le dernier dossier d’exportation
dlg-snapshot-hover-render = Instantané — rendu complet avec les paramètres d’exportation, dans le dernier dossier d’exportation
# The window's title.
dlg-snapshot-title = Instantané
dlg-snapshot-question = Que doit enregistrer Instantané (le bouton appareil photo, Ctrl+S) ?
# The first choice, and a short tag beside it.
dlg-snapshot-screen = Capture d’écran
dlg-snapshot-screen-tag = immédiat
dlg-snapshot-screen-note = Enregistre la vue exactement telle qu’elle apparaît à l’écran, à la résolution de l’écran, en PNG dans le dernier dossier d’exportation. Ce que vous voyez est ce que vous obtenez.
# The second choice, and a short tag beside it.
dlg-snapshot-render = Rendu complet
dlg-snapshot-render-tag = peut être long
# $size: the image size, as 1920×1080; $ss: the supersampling factor (1–4); $format: PNG or EXR.
dlg-snapshot-render-note = Effectue le rendu avec les paramètres d’exportation — { $size }, suréchantillonnage { $ss }×, { $format }. Meilleure qualité, mais une vue profonde peut prendre de quelques minutes à plusieurs heures. Il s’exécute en arrière-plan ; Fichier → Exporter l’image… affiche la progression et permet de l’annuler.
dlg-snapshot-remember = Mémoriser mon choix et ne plus demander
dlg-snapshot-change = Vous pouvez le modifier à tout moment dans Fichier → Paramètres → Instantané.

## Share location (.fdn)

dlg-share-title = Partager l’emplacement
dlg-share-intro = Un emplacement autonome (fractale, centre en pleine précision, zoom, coloration). Copiez-le pour le partager, ou collez/chargez-en un et cliquez sur Appliquer.
dlg-share-thumb = Inclure une miniature dans le fichier enregistré
dlg-share-thumb-hover = « Enregistrer le .fdn » intègre une petite image de cette vue pour que le fichier soit reconnaissable dans la galerie. Ajoute environ 55 KB, et ne fait pas partie du texte ci-dessus.
dlg-share-copy-hover = Copier le texte dans le presse-papiers
# Buttons (the first after a save icon).
dlg-share-save = Enregistrer le .fdn…
dlg-share-load = Charger un .fdn…
dlg-share-apply = Appliquer
dlg-share-apply-hover = Sauter vers l’emplacement du champ
dlg-share-copied = Copié dans le presse-papiers.

## Help ▸ Report an issue (the report itself stays in English: it is read by the maintainer)

dlg-report-title = Signaler un problème
dlg-report-intro = Le meilleur endroit pour signaler un problème est un ticket GitHub — il est public et consultable, et vous verrez quand il sera corrigé. L’e-mail fonctionne aussi. Relisez d’abord tout ce qui suit : rien ne quitte votre machine tant que vous n’agissez pas.
# Labels before three dropdowns.
dlg-report-type = Type :
dlg-report-severity = Gravité :
dlg-report-repro = Reproductible :
dlg-report-describe = Décrivez ce qui s’est passé :
# The faint text in the empty description box.
dlg-report-describe-hint = Ce qui s’est passé, les étapes pour le reproduire, et ce que vous attendiez à la place.
# Before the checkboxes of what the report carries.
dlg-report-include = Inclure :
dlg-report-sysinfo = Informations système (version, OS, CPU, GPU, VRAM)
dlg-report-location = Emplacement actuel (.fdn)
dlg-report-log = Journal récent
dlg-report-frames = Relevé des images (la dernière minute et les 40 dernières images)
dlg-report-crash = Dernier rapport de plantage
dlg-report-crash-none = Dernier rapport de plantage (aucun trouvé)
dlg-report-crash-view = Emplacement du plantage (.fdn)
dlg-report-crash-view-none = Emplacement du plantage (.fdn) (aucun trouvé)
dlg-report-crash-view-hover = La vue exacte où s’est produit un plantage ou une perte du périphérique graphique — la seule chose qui permette à un mainteneur de le reproduire. Enregistrée automatiquement.
dlg-report-test = Résultat d’un test de diagnostic
dlg-report-test-none = Résultat d’un test de diagnostic (lancez-en un depuis Aide → Diagnostics…)
dlg-report-test-hover = Un verdict réussi/échoué validé automatiquement sur votre propre matériel — bien plus utile à un mainteneur qu’une simple description
# A section that opens and closes. $size: the report's size, as "12 KB".
dlg-report-preview = Aperçu de ce qui sera envoyé  ({ $size })
dlg-report-github = Ouvrir un ticket GitHub
dlg-report-github-hover = Ouvrir une page de nouveau ticket avec le titre rempli ; le rapport est copié dans votre presse-papiers — collez-le dans le corps du ticket (Ctrl+V).
# Buttons (after a copy and a save icon).
dlg-report-copy = Copier le rapport
dlg-report-copy-hover = Tout copier dans le presse-papiers
dlg-report-save = Enregistrer le rapport…
dlg-report-save-hover = Enregistrer en .txt pour le joindre
dlg-report-by-email = Ou par e-mail :
dlg-report-gmail = Rédiger dans Gmail
# $address: the email address reports go to.
dlg-report-gmail-hover = Ouvrir une fenêtre de rédaction Gmail adressée à { $address } avec l’objet rempli ; collez le rapport copié
dlg-report-email = Application de messagerie…
dlg-report-email-hover = Ouvrir votre application de messagerie par défaut (mailto) — nécessite un gestionnaire d’e-mail configuré
dlg-report-tip = Astuce : avec chacun de ces boutons, le rapport est copié dans votre presse-papiers — collez-le dans le ticket ou le corps du message (Ctrl+V). Si possible, joignez aussi une capture d’écran ou une image d’exemple (faites-la glisser) ; c’est très utile pour les problèmes de rendu et d’interface.
# Notices in the window after a button is pressed.
dlg-report-opened-github = GitHub ouvert — le rapport est dans le presse-papiers ; collez-le dans le ticket.
dlg-report-copied = Rapport copié dans le presse-papiers.
# $path: where the report was saved.
dlg-report-saved = Enregistré dans { $path }
dlg-report-opened-email = Application de messagerie ouverte — le rapport est dans le presse-papiers ; collez-le.
dlg-report-opened-gmail = Rédaction Gmail ouverte — collez le rapport copié dans le corps du message (Ctrl+V).

## A location whose checksum does not match

dlg-checksum-title = Cet emplacement est peut-être endommagé
# After a ⚠ sign.
dlg-checksum-mismatch = Sa somme de contrôle ne correspond pas aux données qui l’accompagnent.
dlg-checksum-why = Cela signifie généralement que le texte a été tronqué ou modifié en chemin — une copie partielle depuis une fenêtre de discussion ou un message de forum en est la cause habituelle.
# A label over where the location came from (a file path, or the clipboard).
dlg-checksum-source = Source
# The two checksums, side by side: the one written in the file, and the one computed from its data.
dlg-checksum-in-file = dans le fichier
dlg-checksum-from-data = d’après les données
dlg-checksum-load-note = Vous pouvez le charger quand même — les coordonnées peuvent être correctes malgré tout, et rien n’est écrasé. Tout ce qui est illisible est signalé après le chargement.
dlg-checksum-load = Charger quand même
dlg-checksum-load-hover = Y sauter malgré la différence
# Notices after loading it. $zoom: the magnification, as 1.2e148.
dlg-checksum-loaded = Chargé @ { $zoom }× malgré la somme de contrôle discordante
# $zoom: the magnification, as 1.2e148; $note: what the loader found wrong with the location.
dlg-checksum-loaded-note = Chargé @ { $zoom }× — { $note }
dlg-checksum-not-loaded = Non chargé.

## File ▸ Reset application state

dlg-reset-title = Réinitialiser l’état de l’application
# After a ⚠ sign, followed by a list.
dlg-reset-warning = Ceci supprime définitivement toutes les données enregistrées de Fractadyne :
# The list's items, each after a bullet.
dlg-reset-session = la session enregistrée (vue actuelle, coloration, préférences)
dlg-reset-bookmarks = tous les signets et leurs miniatures
dlg-reset-refcache = le cache des orbites de référence (reconstruit à mesure que vous revisitez des emplacements)
# A label over the folder that is deleted.
dlg-reset-location = Dossier
dlg-reset-note = Cette action est irréversible. La session actuelle ne sera pas réenregistrée à la fermeture : les valeurs par défaut seront chargées au prochain lancement.
dlg-reset-confirm = Tout réinitialiser
dlg-reset-done = État de l’application réinitialisé — les valeurs par défaut seront chargées au prochain lancement.
# $error: the system's reason, as it gives it.
dlg-reset-failed = Échec de la réinitialisation : { $error }

## Tools ▸ Tour from current view

dlg-tour-title = Parcours depuis la vue actuelle
dlg-tour-intro = Crée un parcours qui zoome de la vue complète jusqu’à l’endroit où vous êtes.
# A label before the fractal and magnification the tour ends at.
dlg-tour-destination = Destination
dlg-tour-notation = Annotation (légende facultative affichée pendant la plongée)
# The faint example text in the empty caption box.
dlg-tour-notation-hint = p. ex. Plongée vers le Misiurewicz à trois branches…
dlg-tour-duration = Durée du zoom
# $rate: how fast the dive zooms, in powers of ten a second (as 2.5).
dlg-tour-rate = ≈ { $rate } décades/s
# A small button that puts the duration back to its default.
dlg-tour-reset = Rétablir
dlg-tour-reset-hover = Valeur par défaut selon la profondeur
dlg-tour-duration-note = La valeur par défaut augmente avec la profondeur pour que la plongée reste regardable. Des plongées plus rapides peuvent faire clignoter des images d’une seule couleur à un zoom extrême.
# A button, after a save icon.
dlg-tour-save = Enregistrer le parcours…
# The file type's name in the save dialog.
dlg-tour-filter = Script Fractadyne (TOML)
# A notice. $file: the saved file's name. --render-tour is a command-line option: keep it.
dlg-tour-saved = Parcours enregistré → { $file }. Lisez-le via Outils → Lire un parcours, ou rendez-le avec --render-tour.

## Navigate ▸ Bookmarks ▸ Manage…

dlg-bookmarks-title = Signets
# The faint text in the empty name box.
dlg-bookmarks-name-hint = nom (facultatif)
# A button, after a bookmark icon.
dlg-bookmarks-add = Ajouter la vue actuelle
dlg-bookmarks-empty = Aucun signet pour l’instant. Ajoutez-en un depuis la vue actuelle.
# The tooltip of a bookmark's delete (bin) button.
dlg-bookmarks-delete-hover = Supprimer

## Tools ▸ Benchmark…

dlg-bench-title = Banc d’essai
dlg-bench-mode = Mode
dlg-bench-current = Réglages actuels
dlg-bench-current-hover = Lire le parcours fixe de zoom profond dans la vue en direct, avec votre résolution et vos réglages actuels.
dlg-bench-standard = Standardisé
dlg-bench-standard-hover = Fixer la résolution et tous les réglages de rendu pour que le score soit comparable d’une machine à l’autre.
dlg-bench-resolution = Résolution
dlg-bench-depth = Profondeur
# Standard and Ultra are the two depths in the dropdown this describes.
dlg-bench-depth-hover = Les deux plongées entrent dans le chemin profond floatexp/BLA ; Ultra (1e48×) y passe bien plus de temps que Standard (1e32×), et met d’autant plus de temps à s’exécuter.
dlg-bench-burnin = Test d’endurance (répéter)
dlg-bench-burnin-hover = Exécuter le banc d’essai à répétition pour révéler la stabilité et la limitation thermique.
# The label of a slider: how many times the benchmark runs.
dlg-bench-passes = passes
# $size: the image size, as 1920×1080; $ss: the supersampling factor; $depth: the dive's final magnification, as 1e32.
dlg-bench-std-summary = Rendu hors écran en { $size }, SS { $ss }×, Mandelbrot/lissé, plongée de 60 images jusqu’à { $depth }×.
dlg-bench-run = Lancer

## The standardized benchmark's progress window

dlg-bench-progress-title = Banc d’essai en cours…
# $resolution: the benchmark's resolution, as the dropdown names it.
dlg-bench-progress-mode = Standardisé · { $resolution }
# $pass: the pass running now; $total: how many passes there are.
dlg-bench-progress-pass = Passe d’endurance { $pass }/{ $total }
dlg-bench-progress-dive = Rendu de la plongée fixe…
# On the progress bar. $done: frames rendered; $total: frames in the dive.
dlg-bench-progress-frame = image { $done }/{ $total }
# $fps: frames per second, as 41.3.
dlg-bench-progress-last = dernière passe : { $fps } fps

## The benchmark's results

dlg-bench-results-title = Résultats du banc d’essai
# Buttons, each after an icon.
dlg-bench-results-run-again = Relancer…
dlg-bench-results-save = Enregistrer…
dlg-bench-results-none = Aucun banc d’essai n’a encore été lancé.
dlg-bench-results-saved = Résultats du banc d’essai enregistrés.

## The accelerated build's notice when its arithmetic libraries are missing

dlg-backend-title = Version accélérée — arithmétique intégrée utilisée
dlg-backend-dont-show = Ne plus afficher

## File ▸ Gallery…

dlg-gallery-title = Galerie
# A button that picks the folder to show.
dlg-gallery-folder = Dossier…
# A button, after a refresh icon.
dlg-gallery-refresh = Actualiser
dlg-gallery-empty = Aucune image Fractadyne dans ce dossier.
# An image's title when it does not name its fractal.
dlg-gallery-untitled = Image Fractadyne
# $zoom: the image's magnification, as 1.2e148; $saved: when it was saved, as the file gives it.
dlg-gallery-zoom = zoom { $zoom }   { $saved }
# The notes saved with the image, in quotation marks. Use your language's quotation marks.
dlg-gallery-notes = « { $notes } »
dlg-gallery-open = Ouvrir cette vue
dlg-gallery-loaded = Vue chargée depuis la galerie.

## File ▸ Export image…

dlg-export-title = Exporter l’image
# A dropdown of image sizes.
dlg-export-size = Taille
# The size dropdown's text for a size set by hand. $size: the size, as 1920×1080.
dlg-export-size-custom-current = Personnalisée — { $size }
# The size dropdown's first choice.
dlg-export-size-custom = Personnalisée…
dlg-export-size-custom-hover = Réglez vous-même la largeur ; la hauteur suit les Proportions ci-dessous.
dlg-export-width = Largeur (px)
# After the width field. $height: the height in pixels, which follows from the aspect.
dlg-export-height-from-aspect = × { $height } (d’après les proportions)
dlg-export-ss = Suréchantillonnage
dlg-export-aspect = Proportions
# The aspect choice that keeps the window's own shape.
dlg-export-aspect-window = Comme la fenêtre
# Before the choices PNG and OpenEXR.
dlg-export-format = Format :
dlg-export-hud = Incrustation de l’emplacement
dlg-export-hud-hover = Incruster un panneau niveau de zoom + coordonnées en haut à gauche de l’image exportée (à l’échelle de la sortie ; affiche le zoom et le centre de la vue carte/Mandelbrot).
dlg-export-glitch = Correction des glitches
dlg-export-glitch-hover = Correction multi-références des glitches de perturbation. Automatiquement ignorée pour les exportations très profondes (floatexp) à vue unique, afin que la construction de la référence et le rendu s’exécutent hors du fil principal et que l’application reste réactive. Jamais utilisée pour les vues Mandelbrot ou Multibrot (hors mode Julia) : vérifiée par rapport à un calcul exact, elle n’y corrigeait rien et multipliait jusqu’à 5 fois la durée d’exportation.
dlg-export-deep-note = Exportation profonde : la référence se construit hors du fil principal (l’interface reste réactive) ; la correction des glitches est ignorée à cette profondeur.
# A dropdown, shown in the dual (map + Julia) view, and its three choices.
dlg-export-dual = Disposition double
dlg-export-dual-side = Côte à côte
dlg-export-dual-separate = Fichiers séparés
dlg-export-dual-map = Carte seule
dlg-export-notes = Notes :
# The faint text in the empty notes box.
dlg-export-notes-hint = enregistrées avec l’image (≤120 caractères)
# $width and $height: the image size in pixels; $left: how many characters the notes can still take.
dlg-export-output = Sortie : { $width } × { $height } px   ({ $left } caractères restants)
# When the export's shape differs from the window's. $percent: how much more, as 25.
dlg-export-extra-width = Montre { $percent } % de largeur en plus que la fenêtre (rien n’est rogné)
# The same, when the export is taller than the window. $percent: how much more, as 25.
dlg-export-extra-height = Montre { $percent } % de hauteur en plus que la fenêtre (rien n’est rogné)
dlg-export-tiles-note = Rendu par tuiles (sans limite de taille) dans un fil d’arrière-plan. Le fichier intègre la vue et les notes, pour pouvoir être rouvert via Fichier ▸ Ouvrir une vue ou un emplacement.
# A label before the button that picks the folder exports go to.
dlg-export-folder = Dossier :
dlg-export-choose = Choisir…
dlg-export-choose-hover = Choisir le dossier de destination des exportations
dlg-export-open-after = Ouvrir une fois terminé
dlg-export-open-after-hover = Confier l’image terminée à la visionneuse du système dès qu’elle est écrite.
# $count: the graphics cards in this machine (2 or more).
dlg-export-all-gpus = Utiliser toutes les cartes graphiques ({ $count })
dlg-export-all-gpus-hover = Répartir l’image, tuile par tuile, entre toutes les cartes graphiques de cette machine, chacune avec sa propre copie de la référence profonde. Des cartes identiques donnent exactement l’image qu’une seule produirait ; des modèles différents dessinent quelques pixels légèrement différemment, tuile par tuile (mesuré : aucun raccord visible). Si une carte échoue, ses tuiles vont aux autres. Les images que la répartition ne peut pas prendre en charge (certaines méthodes de coloration et formules) utilisent la carte de cette fenêtre.
# The export's progress.
dlg-export-preparing = Préparation (construction de la référence)…
dlg-export-saving = Enregistrement…
dlg-export-rendering = Rendu en cours…
# $time: the time since the export started, as 8.3s or 1m 04.0s.
dlg-export-elapsed = Écoulé : { $time }
dlg-export-cancel-hover = Fermer sans exporter
# A button, after a save icon.
dlg-export-save-as = Enregistrer sous…
dlg-export-save-as-hover = Choisir le nom et l’emplacement du fichier
dlg-export-export = Exporter
dlg-export-export-hover = Effectuer le rendu et enregistrer dans le dossier ci-dessus (nom automatique)

## Notices worded at startup (main.rs)

# A notice shown once after starting, when the saved session comes from a newer version of the app. $saved and $handled: version numbers of the session file's format (1, 2…).
app-state-newer = Cette session a été enregistrée par une version plus récente de Fractadyne (état v{ $saved } ; cette version gère v{ $handled }). Certains réglages peuvent ne pas s’appliquer, et l’enregistrement la réécrira au format de cette version.
# The title of the notice that says the accelerated build's arithmetic libraries are found again (its text comes from elsewhere).
app-notice-mpfr-restored-title = Arithmétique accélérée rétablie
# After Fractadyne restarted itself because the graphics driver reset the GPU.
app-toast-gpu-recovered = Rétabli après une réinitialisation du périphérique graphique — rouvert sur la vue d’accueil, car l’emplacement précédent était trop exigeant à rendre.

## Short notices (toasts)

# When the autopilot reached a view that could not be rendered and put the last good view back.
app-toast-autopilot-view-restored = Pilote automatique arrêté : la vue est devenue inutilisable et a été rétablie (consigné dans le journal)
# $error: the system's reason, as it gives it (often in English).
app-toast-gradients-save-failed = Impossible d’enregistrer les dégradés : { $error }
# $error: the reason, as the program gives it (often in English).
app-toast-gradients-serialize-failed = Impossible de sérialiser les dégradés : { $error }
# $name: the name the user gave the gradient.
app-toast-gradient-saved = Dégradé « { $name } » enregistré.
# $error: the reason, as the program gives it (often in English).
app-toast-bookmarks-serialize-failed = Impossible de sérialiser les signets : { $error }
# $error: the system's reason, as it gives it (often in English).
app-toast-bookmarks-save-failed = Impossible d’enregistrer les signets : { $error }
# Jumping to a named point given by exact expressions. $name: the point's name (in English).
app-toast-coordinates-failed = { $name } : impossible d’évaluer les coordonnées
# $zoom: the magnification, already formatted (1,234.5 or 1.2e148).
app-toast-random-location = Emplacement aléatoire · { $zoom }×
# Importing a Kalles Fraktaler .kfr location. $error: the reason (in English).
app-toast-import-failed = Échec de l’importation : { $error }
# $zoom: the magnification, already formatted.
app-toast-location-loaded = Emplacement chargé @ { $zoom }×

## Go to location: why a jump could not be made (one line per field that failed)

# "Re" is the dialog's real-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-re = Re : { $error }
# "Im" is the dialog's imaginary-part field. $error: the evaluator's reason, with the character position (in English).
app-goto-error-im = Im : { $error }
# The imaginary field was given a complex value. $error: the evaluator's reason (in English).
app-goto-error-im-real = Im : { $error } — saisissez la partie imaginaire comme un nombre réel, sans le i
# "Zoom" is the dialog's magnification field.
app-goto-error-zoom-empty = Zoom : saisissez un grossissement, p. ex. 1e50 ou 250000
# $zoom: what was typed, in quotes.
app-goto-error-zoom = Zoom : impossible de lire { $zoom } — saisissez un grossissement positif, comme 1e50 ou 250000
app-goto-error-invalid = Saisie invalide — vérifiez les coordonnées et le zoom.

## Finding a minibrot or Misiurewicz point (the M key and the Go to dialog)

app-feature-needs-holomorphic = La recherche de minibrot nécessite une famille holomorphe (Mandelbrot / Multibrot).
app-feature-not-in-julia = La recherche de minibrot explore l’ensemble de Mandelbrot — désactivez le mode Julia pour l’utiliser.
app-feature-busy = Une recherche de structure est déjà en cours — une à la fois.
app-feature-searching = Recherche du minibrot le plus proche…
app-feature-mandelbrot-only = La recherche de structures ne fonctionne que pour Mandelbrot.
app-feature-no-pair = Aucun point de Misiurewicz trouvé dans le budget de recherche — le point est peut-être plus profond que ce que la recherche atteint depuis cette vue, ou le centre n’est peut-être pas près d’une spirale ou d’un point de branchement.
# The name of what was found, used as $feature in the messages below. $period: the minibrot's period (a whole number).
app-feature-minibrot = minibrot de période { $period }
# $preperiod and $period: the point's two whole numbers (k, p). $octaves: how often the view repeats when zooming in (2 decimals); $twist: the turn per repeat in degrees.
app-feature-misiurewicz-lambda = Misiurewicz ({ $preperiod },{ $period }) — se répète toutes les { $octaves } octaves, torsion { $twist }°
# $preperiod and $period: the point's two whole numbers (k, p).
app-feature-misiurewicz = Misiurewicz ({ $preperiod },{ $period })
# Added after the messages below when the fixed iteration count is too low for the new depth. $have: the iterations set now; $typical: what the depth typically needs.
app-feature-iter-note = ⚠ Le réglage Itérations est fixé à { $have } ; cette profondeur en demande généralement ~{ $typical } — augmentez-le (ou activez les itérations automatiques) si la vue s’affiche plate.
# $feature: what was found (see above). $zoom and $asked: magnifications, already formatted; $octaves: the solver's depth limit.
app-feature-zoomed-capped = Zoom sur le { $feature } — { $zoom }×. Le calcul est plafonné à { $octaves } octaves : il s’est donc arrêté là plutôt qu’à { $asked }×.
# $feature: what was found. $zoom: the magnification, already formatted. $seconds: how long the search took (1 decimal).
app-feature-zoomed = Zoom sur le { $feature } — { $zoom }× ({ $seconds } s)
# $feature: what was found. $seconds: how long the search took (1 decimal).
app-feature-snapped = Calé sur le centre du { $feature } ({ $seconds } s)
app-feature-no-minibrot = Aucun centre de minibrot trouvé près de la vue — zoomez plus près de l’un d’eux.
# $feature: the point asked for, e.g. Misiurewicz (3,1). $widths: how many view-widths away, already formatted. $octaves: how far to zoom out.
app-feature-just-outside = Un point { $feature } a été trouvé juste en dehors de cette vue — à { $widths } largeurs de vue. Dézoomez d’environ { $octaves } octaves pour l’amener à l’écran, ou choisissez une prépériode plus grande pour une structure à cette échelle.
# $feature: the point asked for. $widths: how many view-widths away, already formatted (may be 2^187). $octaves: how far to zoom out.
app-feature-far = Un point { $feature } a été trouvé, mais il est à { $widths } largeurs de vue — cette structure est bien plus GROSSIÈRE que cette vue. DÉZOOMEZ d’environ { $octaves } octaves pour l’atteindre, ou essayez une prépériode plus grande pour une structure à cette échelle.
# $feature: the point asked for. $residual: a small number in scientific notation.
app-feature-not-preperiodic = { $feature } ne correspond pas à l’orbite ici (résidu { $residual }) — videz les deux champs pour détecter la paire à partir de la vue.
# $feature: the point asked for. "k/p" are the dialog's two fields.
app-feature-not-converged = Aucun point { $feature } n’a convergé près de la vue — rapprochez-vous, ou essayez d’autres k/p.
app-feature-bad-request = La prépériode et la période doivent toutes deux être positives.

## Share location dialog and location files

app-location-empty = Rien à charger (ou texte trop long).
app-location-not-fractadyne = Ce n’est pas un emplacement Fractadyne.
# $zoom: the magnification, already formatted. $note: what in the text was ignored or repaired (in English).
app-share-loaded-note = Chargé @ { $zoom }× — { $note }.
# $error: the system's reason (often in English).
app-share-save-failed = Échec de l’enregistrement : { $error }
app-share-saved = Enregistré.
app-share-saved-rendering-thumb = Enregistré. Rendu de la miniature…
app-share-saved-no-thumb = Enregistré — sans miniature (aucune n’a pu être rendue pour cette vue).
app-share-saved-with-thumb = Enregistré, avec une miniature.
# $error: the system's reason (often in English).
app-share-saved-thumb-failed = Enregistré, mais la miniature n’a pas pu être ajoutée : { $error }
app-share-loaded-into-box = Chargé dans le champ — vérifiez, puis cliquez sur Appliquer.
# Reading a file failed (a location or a palette). $error: the system's reason (often in English).
app-read-failed = Échec de la lecture : { $error }
app-share-too-large = Fichier trop volumineux (ce n’est pas un emplacement .fdn ?).

## File-dialog filter names (the extensions are added by the code)

app-filter-kfr = Emplacement Kalles Fraktaler
app-filter-fdn = Emplacement Fractadyne
app-filter-map = Palette Fractint / KF
app-filter-ugr = Dégradé Ultra Fractal
app-filter-ase = Nuancier Adobe Swatch Exchange
app-filter-ggr = Dégradé GIMP
app-filter-all = Tous les fichiers

## Importing and saving palette files (shown under the gradient editor)

# The reason a file was refused before reading it.
app-gradient-map-too-large = bien trop volumineux pour être une palette .map
app-gradient-ugr-too-large = bien trop volumineux pour être un fichier de palette .ugr
app-gradient-ase-too-large = bien trop volumineux pour être une liste de nuances .ase
app-gradient-ggr-too-large = bien trop volumineux pour être un dégradé .ggr
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported-vga = { $count } entrées importées de { $file } — une table VGA 6 bits (le blanc vaut 252, pas 255), conservée telle quelle pour correspondre à la sortie de Fractint lui-même.
# $count: the palette's entries; $file: the file's name.
app-gradient-map-imported = { $count } entrées importées de { $file }, en bandes franches.
# $error: the reader's reason (in English).
app-gradient-map-invalid = Ce n’est pas une palette .map : { $error }
# $name: the gradient's name in the file; $file: the file's name.
app-gradient-ugr-imported = « { $name } » importé de { $file }.
# $file: the file's name; $count: how many gradients it holds.
app-gradient-ugr-pick = { $file } contient { $count } dégradés — choisissez-en un ci-dessous.
# $error: the reader's reason (in English).
app-gradient-ugr-invalid = Ce n’est pas un fichier de palette .ugr : { $error }
# $used: the swatches kept; $total: the swatches in the file.
app-gradient-ase-imported-some = { $used } nuances importées sur { $total }, réparties uniformément.
# $count: the swatches in the file.
app-gradient-ase-imported = { $count } nuances importées, réparties uniformément et fondues.
# $error: the reader's reason (in English).
app-gradient-ase-invalid = Ce n’est pas une liste de nuances .ase : { $error }
app-gradient-nothing-to-save = Aucun dégradé personnalisé à enregistrer pour l’instant.
# $file: the saved file's name.
app-gradient-ggr-saved = « { $file } » enregistré — points médians, courbes de fondu et espaces de couleur, tout est intact.
# $file: the saved file's name; $count: the segments whose curve could not be kept.
app-gradient-ggr-saved-lossy =
    { $count ->
        [one] « { $file } » a été enregistré, mais { $count } segment utilise une courbe de Bézier que le format .ggr ne peut pas représenter. Le segment a été enregistré en interpolation linéaire ; la copie dans votre bibliothèque conserve la courbe.
       *[other] « { $file } » a été enregistré, mais { $count } segments utilisent des courbes de Bézier que le format .ggr ne peut pas représenter. Ils ont été enregistrés en interpolation linéaire ; la copie dans votre bibliothèque conserve les courbes.
    }
# $path: the file; $error: the system's reason (often in English).
app-gradient-write-failed = Impossible d’écrire { $path } : { $error }
# $name: the gradient's name; $count: its segments.
app-gradient-ggr-imported = « { $name } » importé — { $count } segments, avec leurs points médians, courbes de fondu et espaces de couleur.
# $error: the reader's reason (in English).
app-gradient-ggr-invalid = Ce n’est pas un dégradé .ggr : { $error }

## The eyedropper's banner across the top of the screen while picking a colour

# $hex: the colour under the pointer, as #rrggbb.
app-eyedropper-banner-color = Choix d’une couleur  ·  { $hex }  ·  cliquez pour la prendre, Esc pour annuler
app-eyedropper-banner = Choix d’une couleur  ·  cliquez pour la prendre, Esc pour annuler

## The gradient editor window

app-gradient-title = Éditeur de dégradé
# Before the two view buttons, Bar and Ring.
app-gradient-view = Vue
app-gradient-view-bar = Barre
app-gradient-view-bar-hover = Une bande droite — idéale pour placer les points de couleur avec précision
app-gradient-view-ring = Anneau
app-gradient-view-ring-hover = Un anneau, car la palette est CYCLIQUE : la jonction en haut est la couture que le moteur de rendu franchit à chaque balayage, et un dégradé qui semble correct sur une barre peut y présenter une arête franche.
app-gradient-add-lane-hover = Cliquez n’importe où sur cette ligne pour y ajouter un point de couleur. Le + en ajoute un dans le plus grand intervalle.
app-gradient-strip-hover = Faites glisser un repère pour déplacer son point de couleur · faites glisser la bande elle-même pour FAIRE TOURNER tout le dégradé, avec bouclage à la couture · double-cliquez pour ajouter un point de couleur, clic droit sur un repère pour le supprimer · les flèches du clavier déplacent légèrement la sélection (Shift pour un réglage fin).
app-gradient-remove-selected-hover = Supprimer le point de couleur sélectionné
app-gradient-ring-hover = Faites glisser l’anneau lui-même pour FAIRE TOURNER tout le dégradé — ce qui passe la couture revient de l’autre côté · faites glisser un repère pour déplacer son point de couleur · cliquez sur la piste extérieure pour y ajouter un point de couleur, ou sur ⊕ au milieu pour en ajouter un dans le plus grand intervalle · ⊖ supprime le point de couleur sélectionné. La marque creuse en haut est la couture, où une palette cyclique boucle ; elle reste fixe parce qu’elle EST la jonction, et les couleurs tournent sous elle.
app-gradient-empty = Pas encore de dégradé personnalisé — « Copier un préréglage… », une importation ou un dégradé enregistré ci-dessous en crée un, et ses points de couleur et ses courbes par segment apparaissent ici.
# $count: the entries of an imported palette, more than the editor shows stop by stop.
app-gradient-too-many = { $count } entrées importées — trop pour les modifier point par point. Utilisez l’aperçu ci-dessus, ou Copier un préréglage pour repartir de zéro.
# The selected segment's heading. $index: its number; $total: how many there are; $left and $right: where it starts and ends, 0.000–1.000.
app-gradient-segment-heading = Segment { $index } sur { $total }  ·  { $left } → { $right }
# A label before the list of blend curves.
app-gradient-curve = Courbe
app-gradient-curve-hover = La vitesse à laquelle la couleur parcourt ce segment. « Incurvée » est façonnée par le point médian — avec un point médian à 0.50, elle est exactement linéaire.
# The blend curves (GIMP's kinds), in a narrow list: keep them short.
app-gradient-blend-linear = Linéaire
app-gradient-blend-curved = Incurvée
app-gradient-blend-sine = Sinus
app-gradient-blend-sphere-increasing = Sphère ↑
app-gradient-blend-sphere-decreasing = Sphère ↓
app-gradient-blend-bezier = Bézier
# Hover on Bézier in the list of blend curves.
app-gradient-bezier-exact-hover = Deux points de contrôle que vous pouvez faire glisser dans les deux directions. Cette courbe est déjà cubique : la convertir ne change rien.
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-visible-hover = Deux points de contrôle que vous pouvez faire glisser dans les deux directions. La conversion APPROCHE la courbe actuelle — celle-ci à { $error } près, ce qui est nettement visible.
# Hover on Bézier in the list of blend curves. $error: how far the fit is off (2 decimals).
app-gradient-bezier-barely-hover = Deux points de contrôle que vous pouvez faire glisser dans les deux directions. La conversion APPROCHE la courbe actuelle — celle-ci à { $error } près, ce qui est à peine visible.
# A label before the list of colour spaces (RGB, HSV).
app-gradient-space = Espace
# A label before the Bézier presets beside it.
app-gradient-ease = Transition
app-gradient-ease-hover = Les préréglages écrivent les deux points de contrôle ; faites-les glisser sur la courbe pour tout le reste. Une Bézier n’a pas de point médian — ses poignées indiquent où elle atteint la moitié.
# Bézier presets, as in CSS easing: short buttons.
app-gradient-ease-linear = Linéaire
app-gradient-ease-in = Entrée
app-gradient-ease-out = Sortie
app-gradient-ease-in-out = Entrée-sortie
app-gradient-midpoint = Point médian
app-gradient-midpoint-hover = L’endroit où le fondu atteint la moitié, en fraction de ce segment. 0.50 est au centre.
# A button that puts the midpoint back in the middle of the segment.
app-gradient-centre = Centrer
app-gradient-centre-hover = Revenir à 0.50
app-gradient-no-hue = ⚠ une extrémité de ce segment n’a pas de teinte
app-gradient-no-hue-hover = Une extrémité est noire, blanche ou grise : le balayage part donc du rouge et parcourt toute la roue chromatique. C’est permis et parfois voulu — mais c’est pourquoi le milieu peut avoir une couleur qu’aucune des deux extrémités ne contient. La bande sous la courbe montre ce que cela donne.
app-gradient-segment-start-hover = Couleur au DÉBUT de ce segment
app-gradient-segment-end-hover = Couleur à la FIN de ce segment
app-gradient-bytes-hover = Rouge, vert et bleu de 0 à 255
app-gradient-floats-hover = La même couleur de 0 à 1, telle que la stocke le moteur de rendu
app-gradient-canvas-bezier-hover = La courbe de fondu du segment sélectionné. Faites glisser l’un ou l’autre point de contrôle — en x ET en y — pour la remodeler. Les lignes pâles sont 0 et 1 ; une courbe peut passer au-delà, et c’est la couleur qui est bornée, pas la courbe.
app-gradient-canvas-hover = La courbe de fondu du segment sélectionné : la vitesse à laquelle la couleur va du point de couleur de gauche à celui de droite. Faites glisser l’anneau pour déplacer le point médian, ou passez la courbe en Bézier pour obtenir un point de contrôle qui se déplace dans les deux directions.
# The selected stop's heading. $index: its number; $total: how many there are.
app-gradient-stop-heading = Point de couleur { $index } sur { $total }
app-gradient-stop-color-hover = La couleur de ce point — cliquez sur la pastille pour la choisir, ou saisissez une valeur hexadécimale à côté
app-gradient-stop-bytes-hover = Rouge, vert et bleu de 0 à 255 — les octets sous lesquels cette couleur est écrite
# The eyedropper button.
app-gradient-pick = Prélever
app-gradient-pick-hover = Pipette — prélever une couleur n’importe où à l’écran. Placez le pointeur sur la couleur voulue et cliquez ; Esc annule.
# Written before the stop's position (0.000–1.000) in its number field: a short abbreviation of "position".
app-gradient-stop-pos = pos
app-gradient-nudge-hover = Déplacer légèrement ce point (maintenez Shift pour un pas plus fin)
app-gradient-remove = Supprimer
app-gradient-remove-hover = Supprimer ce point de couleur (ou cliquer sur le ⊖ en dessous)
app-gradient-add-stop = Ajouter un point
app-gradient-add-stop-hover = Diviser le plus grand segment (ou cliquer sur la ligne au-dessus du dégradé)
# A menu of the built-in palettes, to start from a copy of one.
app-gradient-copy-preset = Copier un préréglage…
app-gradient-paste = Coller…
app-gradient-paste-hover = Importer une palette à partir de couleurs hexadécimales ou de triplets RVB 0–255
app-gradient-save-ggr = Enregistrer en .ggr…
app-gradient-save-ggr-hover = Écrire ce dégradé dans un fichier .ggr de GIMP — lisible par GIMP, Krita et Inkscape, ainsi que par cet éditeur
# A menu of the palette file formats (.map, .ugr, .ggr, .ase).
app-gradient-import = Importer
app-gradient-import-map-hover = Charger un fichier de palette .map Fractint / Kalles Fraktaler (lignes R G B, une par entrée)
app-gradient-import-ugr-hover = Charger un fichier de dégradés .ugr Ultra Fractal (un .ugr contient généralement de nombreux dégradés — vous en choisissez un)
app-gradient-import-ggr-hover = Charger un dégradé .ggr de GIMP, avec ses points médians, courbes de fondu et espaces de couleur intacts
app-gradient-import-ase-hover = Charger une liste de nuances Adobe (.ase). Les nuances n’ont pas de position : elles arrivent réparties uniformément et fondues.
# The gradient's name, for the library of saved gradients.
app-gradient-name = Nom
# Shown greyed in the empty name field.
app-gradient-name-hint = mon dégradé
# The save button, when a gradient of that name is already saved.
app-gradient-update = Mettre à jour
app-gradient-save = Enregistrer
app-gradient-save-hover = Enregistrer ce dégradé dans votre bibliothèque, avec ses courbes et ses espaces de couleur intacts
app-gradient-save-needs-name-hover = Donnez d’abord un nom au dégradé
# A menu of the saved gradients. $count: how many there are.
app-gradient-saved-menu = Enregistrés ({ $count })
app-gradient-nothing-saved = Rien d’enregistré pour l’instant
app-gradient-delete-saved-hover = Supprimer ce dégradé enregistré
# $count: the gradient's segments.
app-gradient-rich = Dégradé enrichi : { $count } segments portent des points médians, des courbes de fondu ou des espaces de couleur — plus qu’une simple liste de points de couleur ne peut en contenir.
app-gradient-convert = Convertir en points de couleur modifiables
app-gradient-convert-hover = Le remplacer par une simple liste de points de couleur. Ceci SUPPRIME les points médians, courbes de fondu et balayages de teinte qu’une liste de points ne peut pas contenir.
# Above the list of gradients in an Ultra Fractal file. $count: how many; $file: the file's name.
app-gradient-ugr-list = { $count } dégradés dans { $file }
app-gradient-close-list = Fermer la liste
app-gradient-seamless = Sans couture (la fin rejoint le début)
app-gradient-seamless-hover = Une palette est cyclique : les positions 1 et 0 sont donc voisines à l’écran. Avec cette option, la couleur du dernier point est maintenue égale à celle du premier — la vue en anneau montre la jonction en haut.
app-gradient-ends-differ = ⚠ extrémités différentes
app-gradient-ends-differ-hover = La couleur à la fin du dégradé n’est pas celle du début : une palette cyclique montre donc une arête franche à l’endroit où elle boucle.
app-gradient-hard-bands = Bandes franches (sans fondu entre les entrées)
app-gradient-hard-bands-hover = Sémantique Fractint / .map : chaque entrée est une couleur unie. Désactivé, elles se fondent en un dégradé lisse.
app-gradient-paste-intro = Collez des couleurs hexadécimales (#ff8800) ou des triplets 0–255 (255 136 0), séparés par des virgules, des espaces ou des retours à la ligne :
# Applies the pasted colours.
app-gradient-apply = Appliquer
# Empties the paste box.
app-gradient-clear = Effacer
# $used: the colours kept; $total: the colours pasted; $limit: the most that are kept.
app-gradient-paste-imported-some = { $used } couleurs importées sur { $total }, échantillonnées uniformément dans votre liste (la limite est de { $limit }).
# $count: the colours pasted.
app-gradient-paste-imported = { $count } couleurs importées.
# $error: why the pasted text could not be read (in English).
app-gradient-paste-unreadable = Impossible de lire ceci : { $error }
# A hint at the bottom of the editor. $count: the palette's entries.
app-gradient-bands-hint = { $count } bandes · prises dans l’ordre du fichier ; les positions sont ignorées tant que les bandes sont activées.
# A hint at the bottom of the editor, in the ring view. $count: the gradient's stops.
app-gradient-ring-hint = { $count } points · cliquez sur la piste extérieure pour en ajouter un à cet endroit · ⊕ ajoute dans le plus grand intervalle, ⊖ supprime le point sélectionné · faites glisser un repère pour le déplacer · la marque en haut est la couture
# A hint at the bottom of the editor, in the bar view. $count: the gradient's stops.
app-gradient-bar-hint = { $count } points · cliquez sur la ligne au-dessus du dégradé pour en ajouter un à cet endroit · faites glisser un repère pour le déplacer · ⊖ ou clic droit pour supprimer
app-gradient-cancel = Annuler
app-gradient-cancel-hover = Abandonner toutes les modifications faites depuis l’ouverture de cette fenêtre, et la fermer
app-gradient-ok = OK
app-gradient-ok-hover = Conserver ces modifications et fermer l’éditeur

## Shared

# The button that closes the Help window and the "Faster deep zoom" window.
app-close = Fermer

## The "Faster deep zoom" window (Help menu): the optional accelerated build

app-accel-title = Zoom profond plus rapide
app-accel-active-heading = Vous utilisez la version accélérée
app-accel-active-body = Les orbites de référence du zoom profond sont calculées avec MPFR/GMP, 2,5 à 6,4 fois plus vite qu’avec la version standard pour cette étape — la pause avant qu’une vue profonde commence à se résoudre.
# $status: which arithmetic library is in use and its version (in English).
app-accel-in-use = Arithmétique utilisée : { $status }
app-accel-fallback-heading = Version accélérée — arithmétique intégrée utilisée
app-accel-fallback-body = Ceci est la version accélérée (MPFR), mais ses bibliothèques mathématiques sont introuvables : elle utilise donc l’arithmétique intégrée (astro-float). Les images sont identiques ; seule la pause avant qu’une vue profonde se résolve est plus longue.
# Followed by the folder the program runs from, on its own line.
app-accel-fallback-dlls = Pour utiliser le chemin rapide, placez libmpfr-6.dll et libgmp-10.dll (ainsi que libgcc_s_seh-1.dll et libwinpthread-1.dll) à côté de fractadyne.exe, puis redémarrez :
app-accel-warning-hidden = L’avertissement de démarrage à ce sujet est masqué.
app-accel-show-again = L’afficher de nouveau
app-accel-offer-heading = Une version optionnelle est 2,5 à 6,4 fois plus rapide en profondeur
app-accel-offer-body = Le zoom profond passe une grande partie de son temps à calculer des orbites de référence sur le CPU — la pause avant qu’une vue profonde commence à se résoudre. Une version optionnelle le fait plutôt avec les bibliothèques MPFR/GMP, ce qui est 2,5 à 6,4 fois plus rapide, et d’autant plus que vous allez profond. Tout le reste est identique.
app-accel-offer-identical = Les images sont IDENTIQUES À L’OCTET PRÈS — ce sont les mêmes mathématiques calculées par une bibliothèque plus rapide, vérifiées sur toutes les formules fractales et sur l’ensemble du corpus de comparaison du zoom profond.
app-accel-offer-shared = Vos réglages, votre session enregistrée et vos emplacements sont partagés entre les deux : vous pouvez passer librement de l’une à l’autre, sans rien importer.
# Shown on Windows.
app-accel-separate-windows = C’est un téléchargement séparé, car les bibliothèques qu’elle utilise ne peuvent pas être compilées avec le compilateur qu’utilise la version Windows standard, et elles sont sous une licence (GNU LGPL v3) différente de celle de Fractadyne.
# Shown on other systems.
app-accel-separate = C’est un téléchargement séparé, car les bibliothèques qu’elle utilise sont sous une licence (GNU LGPL v3) différente de celle de Fractadyne.
app-accel-download = Télécharger pour cette version
app-accel-all-releases = Toutes les versions
app-accel-all-releases-hover = Si le lien direct renvoie une erreur 404, cette version n’a pas encore de version accélérée — choisissez ici la plus proche.
# Shown on Windows.
app-accel-install-windows = Décompressez-la et lancez fractadyne.exe depuis ce dossier, en gardant les fichiers .dll à côté.
# Shown on Linux.
app-accel-install-linux = Décompressez-la et lancez ./fractadyne depuis ce dossier. Elle utilise les GMP et MPFR de votre système (libgmp10 / libmpfr6), que presque tous les postes de travail ont déjà — voir README-ACCELERATED.txt dans le paquet.
# Shown on other systems.
app-accel-install-other = Aucun paquet accéléré n’est encore produit pour cette plateforme ; il peut être compilé à partir des sources avec --features fractadyne-core/rug.

## The Help window's frame (its sections are under help-)

app-help-title = Aide de Fractadyne

## Checking for updates

app-toast-up-to-date = Vous avez la dernière version.
# $error: why the check failed (in English).
app-toast-update-failed = Échec de la recherche de mises à jour : { $error }
# The window that offers a newer version.
app-update-title = Mise à jour disponible
# $version: the new version (0.3.0-beta.20). $channel: the token "stable" or "beta", in English; a translation can choose its words by it ({ $channel -> [beta] … *[other] … }).
app-update-version =
    { $channel ->
        [beta] Fractadyne { $version } (bêta) est disponible
       *[other] Fractadyne { $version } (stable) est disponible
    }
# $current: the version running now. $track: the update track the user follows (its name in the language in use).
app-update-running = Vous utilisez { $current }  ·  { $track }
app-update-download-intro = Téléchargez la dernière version depuis GitHub :
app-update-download = Télécharger depuis GitHub
app-update-later = Me le rappeler plus tard

## The second graphics card (shown in its window, after "In use:" or "Could not start:")

# The second device is the same card as the window's. $name: the card's name.
app-worker-same-card = même carte, { $name }
# Why the second device could not start.
app-worker-thread-stopped = le fil d’exécution du périphérique s’est arrêté

## Export status (shown in the Export image window)

app-export-gpu-unavailable = GPU non disponible
app-export-reference-aborted = Échec de l’exportation : construction de la référence interrompue.

## The performance panel (View ▸ Performance panel)
## Its row labels sit in a monospace column about 14 characters wide: keep them short (a longer label pushes its value to the right).

app-perf-fps = FPS
app-perf-fps-hover = Images présentées par seconde, d’après l’intervalle lissé entre les images. Pendant qu’une vue se stabilise, cette ligne indique plutôt la progression de l’affinage — une vue en cours de stabilisation ne tourne pas au rythme qu’un nombre ici laisserait supposer.
# What the FPS row shows while a view is settling. $done and $total: tiles; $eta: the time left, as m:ss.
app-perf-refining-tiles = affinage { $done }/{ $total } (~{ $eta })
app-perf-refining-arming = affinage (armement)
# $percent: how much is done, with one decimal.
app-perf-refining-percent = affinage { $percent } %
app-perf-building-reference = construction de la référence
app-perf-idle = inactif
app-perf-frame = image
app-perf-frame-hover = Temps réel lissé entre les images — pas la somme des lignes en dessous. Il inclut l’attente de présentation : sur une vue stabilisée et inactive, il reflète donc la cadence de rafraîchissement plutôt qu’un travail.
app-perf-cpu = cpu
app-perf-cpu-hover = Temps du fil principal dans une mise à jour : préparation des paramètres et gestion du cache des références. C’est la ligne qui grimpe en flèche quand une référence se reconstruit, et un pic ici est la saccade que vous ressentez.
app-perf-gpu-idle = gpu/inactif
app-perf-gpu-idle-hover = Temps d’image moins le temps CPU du fil principal. C’est du travail GPU pendant le rendu de la vue, mais sur une vue stabilisée c’est surtout de l’attente — le nom dit les deux, car le panneau ne peut pas les distinguer par cette seule soustraction.
app-perf-timing = mesure
# How frame cost is measured, the timing row's value. TIMESTAMP_QUERY is a name.
app-perf-timing-starved = temps réel (horodatages absents)
app-perf-timing-gpu = horodatages GPU
app-perf-timing-no-query = temps réel (pas de TIMESTAMP_QUERY)
app-perf-timing-hover = La manière dont le coût d’une image est mesuré. Les horodatages GPU sont la source précise ; le repli sur le temps réel s’enclenche quand les mesures cessent d’arriver, et un budget évalué au temps réel a plusieurs fois mal dimensionné des images profondes — lisez le rouge ici comme « les marges de sécurité sont pour l’instant des estimations ».
app-perf-mode = mode
app-perf-mode-hover = Le chemin arithmétique de la dernière image : direct (sans orbite de référence), perturbation df32, ou floatexp — le chemin à plage étendue utilisé au-delà d’environ 1e28x, et le seul sur lequel BLA fonctionne.
app-perf-eff-iter = iter eff.
app-perf-eff-iter-hover = Les itérations réellement utilisées par la dernière image, après la mise à l’échelle selon la profondeur et tout plafond appliqué pendant le mouvement — pas le réglage de base dans Qualité, qui n’est que le point de départ.
app-perf-precision = précision
app-perf-precision-hover = Le nombre de bits de mantisse avec lequel l’orbite de référence est calculée. Il augmente avec la profondeur ; 64 bits est le plancher en faible profondeur, pas une limite que vous atteignez.
app-perf-orbit-len = long. orbite
# $cap: the most orbit samples this GPU holds, already formatted, or the text of app-perf-uncapped.
app-perf-orbit-len-hover = Échantillons de l’orbite de référence. Ce GPU en contient au plus { $cap } — s’en approcher, c’est atteindre le mur de profondeur pratique : au-delà, la vue en direct ne peut pas résoudre les emplacements plus profonds qui demandent une orbite plus longue.
app-perf-uncapped = (sans plafond)
app-perf-sa-skip = saut SA
app-perf-sa-skip-hover = Les itérations que l’approximation par séries a sautées avant le début du travail par pixel. Plus c’est élevé, plus c’est rapide, sans rien changer à l’image ; la ligne est absente quand elle n’en a sauté aucune.
app-perf-aa = aa
app-perf-aa-hover = Suréchantillonnage : chaque pixel est rendu à partir de ce nombre d’échantillons par axe ; 2x coûte donc quatre fois le travail.
app-perf-dual = double
app-perf-dual-hover = Indique si la vue partagée Mandelbrot/Julia est active. Si oui, chaque coût de ce panneau couvre LES DEUX volets.
app-perf-zoom = zoom
app-perf-zoom-hover = Le grossissement par rapport à la vue complète. Le mode arithmétique ci-dessus change vers 1e4x, puis de nouveau vers 1e28x.
app-perf-iterate = itération
# $target and $lethal: times in milliseconds, already formatted.
app-perf-iterate-hover = Temps GPU mesuré du dernier envoi d’itération. Le régulateur vise { $target } ms ; des mesures soutenues proches de { $lethal } ms correspondent à la plage où cette classe de matériel a perdu le périphérique.
app-perf-steps = pas/s
app-perf-steps-hover = Pas d’itération par seconde sur le GPU, en milliards. Les vues riches en intérieur sont plus lentes par pas que le simple décompte ne le laisse penser : c’est donc une mesure de débit plutôt qu’un score.
app-perf-budget = budget
# After the budget's value while it is still converging; the hover below quotes it.
app-perf-budget-settling = (convergence)
app-perf-budget-hover = Budget adaptatif de pas par envoi, la protection du régulateur contre une image assez longue pour que le pilote réinitialise le GPU. « convergence » signifie qu’il converge encore à partir de mesures éparses et n’est pas encore un plafond fiable.
app-perf-rss = rss
# $mb and $peak: memory in megabytes, already formatted.
app-perf-rss-value = { $mb } MB (pic { $peak })
app-perf-rss-hover = Mémoire résidente du processus, et le maximum atteint au cours de cette session. Des constructions de références profondes ont dépassé 2 GB en conditions réelles sans que rien d’autre à l’écran ne le signale ; ambre au-delà des trois quarts de la RAM du système.
app-perf-gpu-est = gpu estim.
app-perf-gpu-est-hover = Mémoire résidente estimée du GPU : orbites de référence, arbres BLA, textures d’itérations, le ping-pong de l’état des blocs pendant un affinage, et l’instantané du verrou de présentation. Assemblée à partir des tailles d’allocation connues ; le chiffre réel du pilote n’est pas lisible de façon portable.
app-perf-julia-re = julia c.re
app-perf-julia-re-hover = Partie réelle du paramètre c que rend le volet Julia — le point sous le curseur, sauf s’il est épinglé.
app-perf-julia-im = julia c.im
app-perf-julia-im-hover = Partie imaginaire du paramètre c que rend le volet Julia.
app-perf-julia-c = julia c
# The julia c row's value when the parameter is held fixed.
app-perf-pinned = épinglé
app-perf-pinned-hover = Le paramètre est maintenu sur un point fixe : déplacer le curseur ne change donc plus le volet Julia.
app-perf-c-panel = c/volet
app-perf-c-panel-hover = Largeur de l’espace des c couverte par tout le volet Mandelbrot. Quand elle est bien inférieure à un pixel Julia, le survol change c mais l’ensemble de Julia semble inchangé — c’est normal en zoom profond, pas un blocage.
app-perf-ref-recompute = recalcul réf.
app-perf-ref-recompute-hover = Durée du plus récent recalcul d’orbite de référence. C’est la pause avant qu’une vue profonde commence à se résoudre, et l’étape qu’accélère la version accélérée optionnelle.
app-perf-recompute-rate = recalculs/s
app-perf-recompute-rate-hover = Orbites de référence INSTALLÉES par seconde — celles dont le résultat a réellement servi. Comparez avec la ligne du dessous : la différence est le travail jeté.
app-perf-builds-rate = constr. réf./s
app-perf-builds-rate-hover = Constructions de références LANCÉES par seconde, y compris celles dont le résultat est jeté — un créneau d’anticipation que la plongée n’atteint jamais, ou une extension refusée. Elles coûtent le CPU à plein et n’apparaissent nulle part ailleurs : une boucle de préchargement a un jour lancé environ 400 constructions par seconde pendant six minutes alors que recalculs/s affichait 2. Un seul chiffre est normal ; un grand écart entre ces deux lignes signifie que du travail est jeté.
app-perf-recompute-total = recalculs tot.
app-perf-recompute-total-hover = Recalculs de référence depuis le lancement. Utile comme débit sur un intervalle connu ; un nombre qui grimpe alors que rien ne bouge est le signal qui mérite d’être suivi.

## Help: the contents list (Help & reference…, F1)

# Each section's name in the contents list (about 165 px wide: keep them short), and its heading.
help-section-overview = Vue d’ensemble
help-section-navigation = Navigation
help-section-expressions = Expressions de coordonnées
help-section-options = Coloration et options
help-section-fractals = Fractales
help-section-how-it-works = Fonctionnement
help-section-command-line = Ligne de commande
help-section-shortcuts = Raccourcis
help-section-hardware = Matériel recommandé
# Credits to the prior work Fractadyne builds on.
help-section-acknowledgments = Remerciements
# The third-party license notices.
help-section-licenses = Licences
help-section-about = À propos

## Help ▸ Overview

help-overview-intro = Fractadyne est un explorateur de fractales natif conçu pour le zoom ultra-profond et la vitesse. Il dessine des fractales « à temps d’échappement » — des images créées en répétant une formule simple en chaque pixel.
help-overview-what-is = Qu’est-ce qu’une fractale à temps d’échappement ?
help-overview-what-is-text = Pour chaque pixel, le programme applique encore et encore une formule comme z → z² + c, en partant de zéro. Si la valeur courante reste petite à jamais, le pixel appartient à l’ensemble et il est dessiné en sombre. Si elle finit par croître sans limite (« s’échappe »), le pixel est hors de l’ensemble, et sa couleur indique combien d’étapes cela a pris. La frontière infiniment détaillée entre « reste » et « s’échappe » est la fractale.
help-overview-can-do = Ce que vous pouvez faire
help-overview-can-pan = Déplacer la vue et zoomer pratiquement sans limite (la position est exacte à toute profondeur).
help-overview-can-families = Passer d’une famille à l’autre parmi 25 familles de fractales intégrées (dix, dont cinq aussi à des puissances supérieures), vos propres formules, Life et les L-systèmes, et afficher n’importe quelle famille à temps d’échappement, sauf Newton, sous forme d’ensemble de Julia.
help-overview-can-recolor = Recolorer avec des dégradés prédéfinis ou personnalisés et plusieurs méthodes de coloration.
help-overview-can-lighting = Ajouter un éclairage en relief 3D et des contours lumineux sur la frontière.
help-overview-can-minibrots = Se caler sur des minibrots, mettre des endroits en signet et exporter des images haute résolution.
help-overview-can-autopilot = Laisser le pilote automatique du zoom plonger seul vers le détail.
help-overview-can-share = Partager n’importe quel endroit sous forme d’un petit extrait de texte ou d’un fichier « .fdn ».
help-overview-can-tours = Lancer des parcours scriptés et un banc d’essai du matériel.
help-overview-first-steps = Premiers pas
# "Seahorse Valley" is a place in the Navigate menu: use its name as that menu shows it. A and F1 are keys.
help-overview-first-steps-text = Ouvrez le menu Navigation et choisissez « Seahorse Valley », puis tournez la molette de la souris pour zoomer. Faites glisser pour déplacer la vue. Ou appuyez simplement sur A et laissez le pilote automatique plonger pour vous. Appuyez sur F1 à tout moment pour revenir à cette aide.

## Help ▸ Coordinate expressions
## The expressions, numbers and function names here are typed as shown in every language:
## keep them exactly as written. Menu items in quotation marks are the menus' own labels.

help-expr-intro = Partout où Fractadyne attend une coordonnée, vous pouvez saisir une expression au lieu d’un nombre décimal : les champs Re / Im de Navigation → « Aller à l’emplacement… » (et chaque champ de son mode polaire), les lignes center_re / center_im d’un emplacement .fdn, les re / im d’un script de parcours, et --center sur la ligne de commande. L’expression est évaluée avec autant de chiffres que le zoom l’exige, et la fenêtre « Aller à l’emplacement » la conserve avec la vue — ainsi, un point saisi comme 1/3, ou comme -0.5 + 0.25*cos(pi/4), reste exactement sur ce point quelle que soit la profondeur du zoom, là où un nombre décimal dériverait une fois ses chiffres épuisés.
help-expr-examples = Exemples
help-expr-ex-third = Un tiers exact. Davantage de chiffres apparaissent à mesure que vous zoomez.
help-expr-ex-neck = Le col de Seahorse Valley (la vallée des hippocampes) — la racine du bulbe de période 2.
help-expr-ex-circle = Un point du cercle de rayon 1/4 centré en -0.5 (le mode polaire compose cela pour vous).
help-expr-ex-bulb = Re de la racine du bulbe de période 3 ; son Im est sin(2*pi/3)/2 - sin(4*pi/3)/4. Navigation → « Points exacts (expressions) » les contient déjà.
help-expr-ex-complex = Une valeur complexe entière saisie dans le champ Re remplit à la fois Re et Im.
help-expr-ex-combine = Fonctions et constantes se combinent librement.
help-expr-ex-powers = Puissances : 2^-20 vaut 1/1048576. -2^2 vaut -4 — le signe moins s’applique après la puissance.
help-expr-ex-root = -2, la racine cubique réelle. Les racines paires d’un nombre négatif sont refusées ; sqrt(-1) donne i.
help-expr-ex-scientific = La notation scientifique fonctionne aussi dans les expressions.
help-expr-numbers = Nombres et opérateurs
# The decimal point in an expression is '.' in every language: keep that rule as it is.
help-expr-numbers-decimals = Décimaux avec un exposant facultatif : 0.25, -1.5e-30, .5. Uniquement des chiffres — ni espaces ni virgules de groupement dans un nombre, et le séparateur décimal est toujours le point « . ».
help-expr-numbers-imaginary = Ajoutez un suffixe i pour un nombre imaginaire (0.25i), ou utilisez i seul : 3 + 4*i.
# Starts with the operators themselves, two spaces apart.
help-expr-numbers-operators = +  -  *  /  et ^ (puissance ; évaluée de droite à gauche, donc 2^3^2 = 2^9). Les parenthèses regroupent. Il n’y a pas de multiplication implicite : écrivez 2*pi, pas 2pi.
help-expr-numbers-case = Les noms ne tiennent pas compte de la casse : PI, Pi et pi sont la même constante.
help-expr-constants = Constantes
help-expr-functions = Fonctions
help-expr-functions-angles = Les angles sont en radians. Pour des degrés, multipliez par pi/180 — cos(45*pi/180) — ou passez la fenêtre « Aller à l’emplacement » en mode polaire et choisissez l’unité degrés.
help-expr-refused = Ce qui est refusé, et pourquoi
help-expr-refused-fractional = Un nombre négatif élevé à une puissance fractionnaire, comme (-2)^(1/2) : le résultat est complexe et il y a plusieurs racines possibles. Utilisez root(x, n) ou cbrt(x) pour une racine réelle, ou sqrt(x) pour la racine complexe principale.
help-expr-refused-complex = Un argument complexe passé à une fonction à valeurs réelles (sin(i), ln(1+i)). Seules sqrt et abs acceptent un argument complexe ; les opérateurs arithmétiques et les puissances entières fonctionnent sur les valeurs complexes.
help-expr-refused-domain = Erreurs de domaine : ln ou log d’un nombre qui n’est pas positif, asin ou acos hors de -1…1, 0 élevé à une puissance négative, division par zéro.
help-expr-refused-bounds = Des arguments de sin, cos, tan, exp ou ^ au-delà de ±2^32, et une imbrication de plus de 32 niveaux : une coordonnée collée est une entrée non fiable, et ces cas prendraient un temps illimité.
help-expr-refused-grammar = Pas de variables, de fonctions définies par l’utilisateur, de comparaisons ni de conditions. La grammaire est volontairement réduite pour qu’un emplacement enregistré signifie la même chose dans toutes les versions.
help-expr-no-parse = Si l’expression n’est pas comprise
help-expr-no-parse-text = Le message nomme le champ, indique ce qui a été trouvé et à quel caractère, et — pour les erreurs habituelles — ce qu’il faut saisir à la place : un signe moins typographique (−) collé depuis un document, × ou ÷, une virgule décimale, un * manquant entre un nombre et un nom, une parenthèse non fermée, ou une fonction écrite sans ses parenthèses. Rien n’est jamais lu à moitié : si une partie du texte ne peut pas être comprise, toute la coordonnée est refusée plutôt que raccourcie en silence.

## Help ▸ Navigation
## Menu paths (Navigate → "Go to location…") and quoted items name the interface's own
## menus and controls: use their translations. Key names (Ctrl, Shift, Space, Esc) stay.

# A heading: the mouse controls (also in Shortcuts).
help-mouse = Souris
# The left column of a controls table: a gesture or key. Keep key names (Shift, Space, Ctrl) as they are.
help-key-left-drag = Glisser (bouton gauche)
help-nav-left-drag = Déplacer la vue.
help-key-mouse-wheel = Molette de la souris
help-nav-wheel = Zoom avant/arrière vers le curseur ; tournez-la plus vite pour zoomer davantage par cran. Avec « Zoom aimanté aux structures » activé, zoomer près d’un minibrot attire le zoom sur celui-ci.
help-key-shift-drag = Shift+glisser
help-nav-shift-drag = Zoom sur un cadre — tracez un rectangle et la vue zoome pour qu’il la remplisse.
# $icon: the click-to-zoom toolbar button's symbol.
help-key-click-zoom = Cliquer pour zoomer ({ $icon })
help-nav-click-zoom = Activez le bouton cible de la barre d’outils (vue simple) : un clic gauche plonge alors vers le point d’un facteur donné, un clic droit recule. Glisser déplace toujours la vue ; choisissez le facteur (2–100×) sous Navigation dans le panneau de droite. Backspace annule un clic.
help-nav-continuous = Zoom continu et retour à l’accueil
help-key-hold-space = Maintenir Space
help-nav-hold-space = Zoom avant fluide, ancré au curseur.
help-key-hold-shift-space = Maintenir Shift+Space
help-nav-hold-shift-space = Zoom arrière fluide.
help-nav-continuous-text = Le bouton de retour à l’accueil anime un doux retour vers la vue complète. « Vitesse de zoom » dans le panneau de droite règle la vitesse du zoom continu.
help-nav-autopilot = Pilote automatique
help-nav-autopilot-text = Appuyez sur A (ou Outils → « Zoom automatique (pilote automatique) ») pour une plongée sans les mains : il choisit un point du bord de la fractale où le détail est le plus dense, de préférence au milieu de la vue, et glisse vers lui tout en l’amenant doucement au centre de l’écran. Il garde cette cible tant qu’elle reste intéressante et tourne en douceur vers une structure plus riche quand il en apparaît une. Toute action de navigation — ou Esc — l’arrête. « Priorité du zoom automatique » dans le panneau de droite (ou dans Outils) choisit ce qu’il fait passer en premier : Vitesse zoome à la vitesse réglée et rafraîchit l’écran dès qu’une image avec du détail est prête ; Qualité n’affiche que des images entièrement résolues en pleine résolution et ralentit le zoom autant que nécessaire pour que l’image ne soit jamais très agrandie avant l’arrivée de la suivante. Dans les deux cas, l’écran n’affiche jamais d’image vide ou d’une seule couleur : un rafraîchissement qui n’a encore résolu aucun détail reste caché derrière le précédent. « Cible du zoom automatique » choisit ce qu’il vise : Détail (le bord le plus riche de la vue, choisi de nouveau en chemin) ou Point de Misiurewicz — il trouve le point de Misiurewicz le plus proche, le calcule jusqu’à la profondeur de la limite de plongée en restant immobile, puis plonge exactement vers lui ; la structure autour d’un tel point se répète toutes les quelques octaves, à l’infini, si bien que cette plongée n’aboutit jamais à une impasse (ensemble de Mandelbrot uniquement).
help-nav-history = Historique et déplacements précis
help-nav-undo = Revenir à la vue précédente.
help-nav-redo = Rétablir.
help-nav-goto = Navigation → « Aller à l’emplacement… » vous permet de lire, saisir, coller ou copier le centre et le zoom exacts (pleine précision, toute profondeur). Les champs du centre acceptent plus que de simples nombres : des fractions comme « 1/3 » et des expressions construites avec pi, tau, e, sqrt et sin/cos/tan — ainsi, « -0.5 + 0.25*cos(pi/4) » est une coordonnée valide. Une expression est conservée avec la vue et réévaluée à la précision qu’exige chaque profondeur : un point saisi ainsi reste exact quelle que soit la profondeur du zoom (ce que ne ferait pas, à terme, un nombre décimal arrondi). Un mode polaire (décalage + r ∠ θ, en degrés / radians / tours) compose l’une de ces expressions pour vous. La liste complète des fonctions et constantes, avec des exemples, se trouve dans la section « Expressions de coordonnées » de cette aide (le bouton « ? » de la fenêtre l’ouvre). Navigation → Signets enregistre et rappelle des emplacements.
help-nav-exact-points = Navigation → « Points exacts (expressions) » saute vers des points définis exactement plutôt que par un nombre décimal arrondi — les racines des bulbes de la cardioïde principale, là où se rattache chaque bourgeon de période q, écrites sous forme d’expressions cos/sin. Comme l’expression accompagne la vue, vous pouvez zoomer sur l’un d’eux sans jamais vous écarter du point.
help-nav-share = Navigation → « Partager l’emplacement… » capture toute la vue — fractale, centre en pleine précision, zoom et coloration — sous forme d’un extrait « .fdn » compact que vous pouvez copier, recoller puis appliquer, ou enregistrer/charger comme fichier, pour reproduire exactement un endroit (ou l’envoyer à quelqu’un).
help-nav-open = Fichier → « Ouvrir une vue ou un emplacement… » ouvre directement chacun de ces éléments : un PNG/EXR exporté (sa vue intégrée est restaurée), un fichier d’emplacement « .fdn », ou un « .kfr » de Kalles Fraktaler.
help-nav-finding = Trouver du détail
help-nav-minibrots = Les minibrots sont de minuscules copies de l’ensemble entier, cachées le long de la frontière. Centrez-en un approximativement et appuyez sur M (ou Outils → « Trouver un minibrot et zoomer dessus ») pour vous caler par Newton exactement sur son centre, lire sa période et zoomer directement à sa propre échelle — souvent de nombreux ordres de grandeur d’un seul coup. Si vous êtes déjà plus profond que le minibrot, seul le centre se déplace.
help-nav-minimap = Affichage → « Mini-carte » affiche une petite carte de l’ensemble entier avec un repère « vous êtes ici » et la profondeur de zoom actuelle ; cliquez sur la carte pour sauter vers une région.

## Help ▸ Coloring & options
## Names in quotation marks are the controls' own labels in the right-hand panel.

help-options-intro = Tout cela se trouve dans le panneau de droite (et est conservé d’une session à l’autre).
help-options-palette = Palette
help-options-palette-text = Palette choisit un dégradé prédéfini, votre dégradé personnalisé, ou un mode à deux couleurs. « Modifier le dégradé… » ouvre un éditeur où chaque point de couleur a une couleur et une position (0–1) ; ajoutez des points, copiez un préréglage comme point de départ, ou importez un fichier de palette.
help-options-paste = « Coller… » dans l’éditeur de dégradé importe une palette trouvée ailleurs. Il accepte les couleurs hexadécimales avec ou sans # initial (#ff8800, ff8800, ou la forme courte #f80) et les simples triplets RVB 0–255 comme « 255 136 0 » — le format des fichiers .map de Fractint et de Kalles Fraktaler — séparés par des virgules, des espaces ou des retours à la ligne. Jusqu’à 256 couleurs sont conservées ; une liste plus longue est échantillonnée uniformément jusqu’à 256, en gardant la première et la dernière pour que la forme du dégradé soit préservée.
help-options-import-map = « Importer ▸ .map… », juste à côté, charge un fichier de palette .map de Fractint ou de Kalles Fraktaler. Ce sont des tables de correspondance indexées par le nombre d’itérations, sans fondu entre les entrées, et les marches de couleur franches qui en résultent font tout le style Fractint classique — c’est donc ainsi qu’elles sont importées, avec une case « Bandes franches » qui permet de fondre plutôt ces mêmes couleurs en un dégradé lisse. Les fichiers de Fractint lui-même stockent des valeurs de couleur sur 6 bits multipliées par quatre : leur blanc vaut donc 252 et non 255 ; Fractadyne le signale quand il en importe un et laisse les valeurs exactement telles qu’elles sont écrites, car les images enregistrées par Fractint lui-même portent le même 252.
help-options-duotone = Bichromie projette la valeur sur une rampe lisse à deux couleurs (Ombre → Lumière). Binaire (ensemble) est une vue unie à deux couleurs — une couleur pleine pour les points dans l’ensemble et une autre pour l’extérieur, sans dégradé — le moyen le plus clair de voir la forme de l’ensemble.
help-options-cycle = Cycle règle combien de fois le dégradé se répète sur la plage d’itérations (bandes plus serrées ou plus larges). Décalage fait tourner tout le dégradé.
help-options-normalize = « Normaliser les couleurs profondes » ajuste la palette à la plage des valeurs d’échappement réellement présentes dans la vue. Les vues profondes couvrent des centaines de milliers d’itérations et, sans cela, un cycle fixe fait boucler le dégradé des milliers de fois entre pixels voisins, transformant un détail correct en mouchetures.
help-options-log-scale = « Échelle de couleur logarithmique » répartit la palette selon le logarithme de la valeur d’échappement plutôt que linéairement. Les nombres d’échappement se concentrent vers le haut à mesure que vous descendez : un ajustement linéaire consacre alors l’essentiel du dégradé à une mince couche collée à la frontière et laisse le reste presque uni ; l’échelle logarithmique l’étale, et garde des couleurs plus stables au fil d’une vidéo de zoom quand la plage s’élargit.
help-options-animate = Animer fait défiler les couleurs dans le temps — Avant, Arrière ou Aller-retour déplacent le décalage ; Aléatoire synthétise en continu des dégradés harmonieux qui se transforment en douceur. Vitesse règle les cycles (ou transformations) par seconde ; « Mélanger le dégradé » en tire un nouveau en mode Aléatoire.
help-options-method = Méthode de coloration — comment les données deviennent des couleurs
# The coloring methods, by the names the Coloring panel gives them, each with what it does.
help-method-smooth = Itération lissée
help-method-smooth-text = Bandes continues classiques selon le temps d’échappement.
help-method-stripe = Moyenne de rayures
help-method-stripe-text = Bandes fluides issues de l’angle de l’orbite (curseur « Densité des rayures »).
help-method-triangle = Inégalité triangulaire
help-method-triangle-text = Texture fine selon l’endroit où chaque pas tombe entre des bornes.
help-method-trap = Piège orbital
help-method-trap-text = Distance de l’orbite à une forme (point/croix/cercle) ; colore aussi l’intérieur.
help-method-distance = Estimation de distance
help-method-distance-text = Ombre selon la proximité de la frontière.
help-method-decomposition = Décomposition
help-method-decomposition-text = Cellules issues de l’angle d’échappement final.
help-options-relief = Éclairage en relief 3D
help-options-relief-text = Ombre la surface d’après la pente de la frontière (la dérivée) pour un aspect embossé et éclairé. « Angle de la lumière » règle la direction ; Relief règle l’intensité (plus bas = plus net) ; « Faire tourner la lumière » l’anime. Familles holomorphes uniquement (Mandelbrot / Multibrot).
help-options-glow = Lueur de distance
help-options-glow-text = Des bandes de contour lumineuses qui se densifient en filaments brillants près de la frontière. Lueur est la quantité de fondu, « Largeur de bande » l’espacement, et « Animer la lueur » les fait s’écouler.
help-options-quality = Qualité et itérations
help-options-quality-text = Itérations est le nombre maximal de pas avant qu’un pixel soit considéré comme dans l’ensemble ; « Adapter les itérations au zoom » l’augmente automatiquement à mesure que vous zoomez (un détail plus profond en demande davantage). Anticrénelage suréchantillonne les images fixes (2×–8×) une fois la vue stabilisée, atténuant la fine « poussière » extérieure.
help-options-other = Autres
help-options-other-text = Vitesse de zoom règle la vitesse du zoom continu ; la limite de FPS plafonne la fréquence d’images ; « Les deux, liés » montre un ensemble de Mandelbrot et son ensemble de Julia côte à côte (le curseur règle le paramètre de Julia en direct).

## Help ▸ Fractals
## Fractal family names (Mandelbrot, Multibrot, Tricorn, Burning Ship, Celtic, Buffalo,
## Phoenix, Newton) are names: keep them as written, as the formulas. Formula syntax
## (z = z^3 - p1*z + c, sqr, |z|, if / elseif / endif, p1-p5, maxit) is typed as shown.

help-fractals-intro = Chaque famille itère une formule avec z partant de 0 et c fixé par le pixel (temps d’échappement), sauf mention contraire. z = x + iy est un nombre complexe.
help-fractals-mandelbrot = z → z² + c. L’original. L’ensemble est formé de tous les c dont l’orbite reste bornée ; il est connexe, et sa frontière est si plissée qu’elle a une dimension de Hausdorff égale à 2.
help-fractals-multibrot = z → z^d + c pour une puissance d = 3, 4, 5. Les puissances plus élevées ajoutent des lobes : l’ensemble a une symétrie de rotation d’ordre (d−1) (Multibrot 3 est d’ordre 2, 4 d’ordre 3, 5 d’ordre 4).
help-fractals-tricorn = z → conj(z)² + c, où conj(x + iy) = x − iy. La conjugaison le rend antiholomorphe, ce qui lui donne une symétrie d’ordre 3 et des « griffes » courbes caractéristiques.
help-fractals-burning-ship = z → (|x| + i|y|)² + c — on prend les valeurs absolues avant d’élever au carré (réel = x²−y²+cx, imag. = 2|xy|+cy). Non analytique ; les zooms profonds révèlent des structures en forme de navire (traditionnellement vues à l’envers).
help-fractals-celtic = Comme Mandelbrot, mais avec la valeur absolue de la partie réelle : réel = |x²−y²| + cx, imag. = 2xy + cy. Produit des motifs en forme de cœur et de bouclier.
help-fractals-buffalo = Valeur absolue des deux parties de z² : réel = |x²−y²| + cx, imag. = |2xy| + cy — un croisement entre Celtic et Burning Ship.
# A heading: the families at powers above 2 (the Fractal menu's "Higher powers").
help-fractals-powers = Puissances supérieures
# ("Burning Ship  3 4 5") is how the Fractal menu shows such a row: keep it as it is.
help-fractals-powers-text = Multibrot 6, 7 et 8 (z^d + c), ainsi que Burning Ship, Tricorn, Celtic et Buffalo aux puissances 3, 4 et 5 : les mêmes repliements à une puissance plus élevée. Burning Ship d prend les valeurs absolues des parties de z puis la puissance ; Tricorn d conjugue d’abord ; Celtic d et Buffalo d replient la partie réelle (et, pour Buffalo, la partie imaginaire) de z^d. Le menu Fractale et la liste déroulante les présentent à raison d’une famille par ligne (« Burning Ship  3 4 5 »). Chacune zoome en profondeur comme sa sœur de puissance 2, et Multibrot 6, 7 et 8 bénéficient aussi de l’approximation par séries et de la recherche de minibrots (M), comme Multibrot 3–5. Multibrot 8 s’échappe au rayon 128 au lieu de 256, là où son pas suivant sortirait de la plage de la simple précision.
help-fractals-phoenix = z → z² + c + p·z(n-1), où z(n-1) est l’itéré précédent et p une constante (p = -0.5). Le terme de mémoire produit des filaments en forme de flammes.
help-fractals-newton = La méthode de Newton pour les racines de z³ − 1 = 0 : z → z − (z³−1)/(3z²). Plutôt que selon le temps d’échappement, les pixels sont colorés selon celle des trois racines cubiques de l’unité vers laquelle l’itération converge (les bassins d’attraction) et selon la vitesse de convergence ; les frontières enchevêtrées des bassins forment la fractale.
help-fractals-julia = Mode Julia
help-fractals-julia-text = Pour toutes les familles sauf Newton, vous pouvez passer à un ensemble de Julia : au lieu de faire partir z de 0 avec c = pixel, vous fixez c (un paramètre) et faites partir z du pixel. L’ensemble de Julia est la frontière entre les points de départ qui restent bornés et ceux qui s’échappent, pour ce c fixé. En mode « Les deux, liés », déplacer le curseur sur le volet Mandelbrot règle c en direct.
help-fractals-deep = Prise en charge du zoom profond
help-fractals-deep-text = Mandelbrot, Multibrot 3/4/5 et Tricorn prennent en charge le zoom profond par perturbation à profondeur extrême (floatexp). Burning Ship, Celtic et Buffalo sont non analytiques (ils prennent des valeurs absolues) : ils utilisent donc une perturbation qui tient compte des signes ; elle fonctionne désormais aussi en plage floatexp, et zoome bien au-delà de l’ancienne limite df32 d’environ 1e28× (de rares mouchetures près des replis de la valeur absolue attendent la correction des glitches multi-références). Phoenix utilise une perturbation à deux termes et zoome aussi en profondeur en plage floatexp (sans SA/BLA, donc plus lourd que Mandelbrot). Les puissances supérieures zooment en profondeur de la même façon, la perturbation de chaque puissance suivant sa propre chaîne d’élévations au carré ; Multibrot 3–8 bénéficient aussi de l’approximation par séries et de BLA, comme Mandelbrot, et Tricorn, Burning Ship, Celtic et Buffalo, à toutes les puissances, bénéficient de BLA (sans séries), qui franchit l’intérieur d’une vue profonde au lieu de l’itérer pas à pas. Seul Newton utilise le chemin direct, net jusqu’à ~1e6× ; les formules personnalisées zooment en profondeur, sauf indication contraire dans la section Formules personnalisées ci-dessous.
help-fractals-custom = Formules personnalisées
help-fractals-custom-text = Fractale ▸ Formule personnalisée… prend le pas d’une formule sous forme d’expressions de style Fractint : z = z^3 - p1*z + c, ou plusieurs instructions (t = sqr(z), puis z = t*t + c). z est l’itéré, c le pixel (la constante de Julia en mode Julia), p1-p5 des paramètres ; |z| est le module au carré, comme dans Fractint, et les fonctions usuelles sont disponibles (sqr, abs, conj, real, imag, cabs, flip, recip, ident, exp, log, sqrt, sin, cos, tan, cotan, sinh, cosh, tanh, cotanh, asin, acos, atan, asinh, acosh, atanh, cosxx, ainsi que floor, ceil, trunc et round, qui arrondissent chaque partie). log est le logarithme naturel, comme dans Fractint et en analyse complexe : ce que beaucoup de manuels notent ln (il n’y a pas de logarithme en base 10). Une formule écrite avec ln, cot, coth, Re ou Im n’est pas lue ; la vérification de syntaxe nomme la fonction à utiliser, un bouton la réécrit, et taper ln propose log. Chaque pas zoome en profondeur par perturbation, avec la même arithmétique à plage étendue au-delà de 1e28x que les fractales intégrées, sauf une formule avec une puissance dont l’exposant varie avec z ou c (comme z^c) : elle est rendue sur le chemin direct, nette jusqu’à ce qu’un pas de c en simple précision couvre un pixel, soit environ 1e4x à 1e5x, et moins profond loin de l’origine. La fenêtre de formule indique ce qui s’applique. log, sqrt et les puissances non entières prennent leurs valeurs principales, qui sautent de part et d’autre de l’axe réel négatif. La perturbation suit exactement ce saut, mais un pixel dont l’orbite franchit la coupure là où celle de la référence ne la franchit pas est ensuite poursuivi à peu près en simple précision, si bien que les vues profondes dont les orbites chevauchent une coupure y montrent du bruit (mesuré : net à 1e5x sur l’axe réel pour sqrt(z^4 + c), bruité pour les formules en log même à 1e4x). Chaque pas est itéré en entier (sans approximation par séries) : une vue profonde avec un nombre d’itérations élevé se rend donc plus lentement que celle d’une fractale intégrée. Près de sa frontière, une formule avec des fonctions a des orbites chaotiques, et une longue orbite y est suivie seulement approximativement en simple précision : à 2 000 itérations, un tiers des pixels de sin(z) + c diffèrent de ceux de l’orbite exacte, à 60 presque aucun. Les vues, les sessions et les images exportées conservent le texte de la formule.
help-fractals-custom-sections = Les sections de Fractint fonctionnent aussi. Les instructions avant un « : » forment la section d’initialisation, exécutée une fois par pixel ; une comparaison en dernière instruction est le propre test d’échappement de la formule, l’itération continuant tant qu’elle est vraie (|z| <= 100 && |z - 1| > 0.000001 arrête une orbite qui s’échappe ou se stabilise sur 1) ; if (...), elseif (...), else et endif choisissent des instructions ; et une variable lue avant d’être affectée dans un pas a sa valeur du pas précédent (celle de la section d’initialisation, ou 0). Les comparaisons comparent les parties réelles et donnent 1 ou 0, && et || les combinent, et une affectation est aussi une valeur (a = b = pixel). pixel, p1-p5, pi et e peuvent être affectés, et gardent leur propre valeur jusqu’à ce qu’ils le soient ; maxit est le nombre d’itérations et ismand vaut 1 (la forme Mandelbrot de la formule). Une formule qui utilise l’un de ces éléments, ou un arrondi, est rendue sur le chemin direct, sans zoom profond. Bibliothèque de formules > Importer lit aussi les fichiers .frm de Fractint. Chaque entrée lisible devient une formule, et sa ligne indique ce que la lecture a changé : fn1-fn4 prennent les valeurs par défaut de Fractint (sin, sqr, sinh, cosh) ; la variable c de Fractint est renommée c_, car c est ici le pixel ; un nom que rien n’affecte vaut 0, comme dans Fractint ; les blancs dans un nom sont supprimés (end if devient endif), comme Fractint les supprime. Le message de l’importation compte les entrées illisibles et donne la raison la plus fréquente : les variables d’écran et de vue (whitesq, scrnpix, scrnmax, center, magxmag, rotskew), lastsqr et les nombres aléatoires ne sont pas pris en charge. Les classiques Fractint de la Collection (Magnet, Nova, Spider, Manowar et d’autres) sont écrits avec ces sections.
help-fractals-custom-editor = Dans le champ de formule, les parenthèses prennent une couleur par niveau d’imbrication ; la paire voisine du curseur est mise en évidence, et une parenthèse sans partenaire est affichée en rouge (celles d’un commentaire ne comptent pas, car l’analyseur les ignore). Les noms se complètent pendant la saisie : après deux lettres, une liste propose les fonctions, les noms et vos propres variables qui commencent ainsi. Haut et Bas choisissent, Tab ou Enter en prend un (une fonction arrive avec sa parenthèse ouvrante), Esc ferme la liste, et Enter après un nom saisi en entier commence toujours une nouvelle ligne.
help-fractals-custom-textbook = « Mise en forme » (l’interrupteur au-dessus de la formule) l’affiche composée comme dans un manuel, et permet de la modifier sur place. Tapez comme dans le texte : / fait une fraction du terme qui le précède, ^ un exposant (+, - ou = tapé à la fin d’un exposant en fait sortir), ( des parenthèses, ou un appel après le nom d’une fonction, | des barres, et une virgule entre parenthèses une constante complexe. Les flèches parcourent la formule, Haut et Bas passent d’un numérateur à son dénominateur, Tab va à la case vide suivante ; Backspace au début d’une fraction ou d’une paire de parenthèses la sélectionne, et un second appui la supprime en gardant ce qu’elle contenait. Copier, coller et annuler fonctionnent comme dans un champ de texte, les noms s’y complètent de la même façon, et le pavé tape dans la formule composée (□² élève au carré ce qui précède le curseur, une touche de fonction enveloppe la sélection). Chaque modification est réécrite dans le texte, si bien que les deux vues contiennent toujours la même formule ; une ligne que vous n’avez pas modifiée garde son espacement et son commentaire. La notation ne change jamais ce qui est calculé : |z| est le module au carré et s’affiche |z|², cabs(z) s’affiche |z|, et exp(z) reste exp(z). Une ligne illisible s’affiche en rouge ; cliquez dessus pour la modifier comme texte. LaTeX copie la formule sous forme de source LaTeX, telle qu’elle est composée, et tant que « Mise en forme » est activé, la bibliothèque de formules affiche aussi ses formules composées.
help-fractals-custom-numbers = Les nombres d’une formule (0.1, 1/3, pi, les paramètres) sont lus comme des valeurs en double précision, environ 16 chiffres, et l’orbite de référence profonde itère avec exactement ces valeurs. Deux formules égales sur le papier peuvent donc différer en profondeur : 0.1*z multiplie par le double le plus proche de 0.1, tandis que z/10 divise par un 10 exact, et les deux images divergent au-delà d’environ 1e15x (mesuré : 5 % des pixels à 1e15x, un tiers à 1e17x, tous à 1e20x). Chaque image est correcte pour sa propre formule ; écrivez celle que vous voulez partager.
help-fractals-custom-library = Le bouton « Enregistrer dans la bibliothèque » de la fenêtre conserve une formule et ses paramètres sous un nom (enregistrer sous un nom que la bibliothèque contient déjà met à jour cette entrée). Fractale ▸ Bibliothèque de formules… les liste pour les appliquer, les modifier, les exporter ou les supprimer ; Importer ajoute les formules d’un fichier de formules sans en remplacer aucune : une formule que la bibliothèque a déjà est ignorée, et une nouvelle dont le nom est pris devient « nom (2) ». La bibliothèque est formulas.toml dans le dossier indiqué sous Fichiers et données (À propos), lui-même un fichier de formules. Une image clé de parcours porte une formule sous la forme formula = "..." avec formula_params = [[re, im], ...], et Outils > Parcours depuis la vue actuelle les écrit pour une vue personnalisée.
help-fractals-custom-collection = Une collection de 45 formules est fournie avec Fractadyne, écrites pour lui : puissances supérieures et variantes repliées des familles Burning Ship et Celtic, mélanges et hybrides à deux étapes, l’ensemble de Mandelbrot vu à travers une autre transformation (inversé, log-polaire, à travers le sinus), formules transcendantes et rationnelles, et ensembles de Julia. Ce sont les Exemples de la fenêtre et l’onglet Collection de la bibliothèque de formules, où « Copier dans les miennes » en ajoute une à vos formules. Chacune a une vue de départ, et Appliquer s’y rend : une formule ouverte sur la vue d’accueil n’est souvent pas là où se trouve son image. Une formule que vous enregistrez pendant qu’elle est affichée conserve de la même façon la vue où vous êtes, et Appliquer y revient.

## Help ▸ How it works

help-how-escape = Temps d’échappement et couleur lissée
help-how-escape-text = Chaque pixel itère la formule jusqu’à ce que son module dépasse un rayon d’échappement ou qu’il atteigne le plafond d’itérations. Le simple nombre de pas produit des bandes franches ; ajouter une partie fractionnaire tirée du module final donne une couleur continue, sans bandes.
help-how-precision = Position en précision arbitraire
help-how-precision-text = Les nombres 64 bits ordinaires manquent de chiffres vers un zoom de 1e15×. Fractadyne conserve le centre de la vue en précision arbitraire, le nombre de chiffres augmentant à mesure que vous zoomez, si bien que l’emplacement ne se dégrade jamais. L’échelle des pixels est de même conservée avec un exposant étendu : elle ne bute donc pas non plus sur la limite d’environ 1e308× des nombres 64 bits — la profondeur n’est bornée que par la patience, pas par une plage numérique fixe.
help-how-perturbation = Perturbation
help-how-perturbation-text = Itérer chaque pixel en haute précision serait bien trop lent. À la place, un pixel de référence est itéré en haute précision sur le CPU (l’« orbite de référence »), et chaque autre pixel est calculé sur le GPU comme une infime différence δ par rapport à lui, en basse précision rapide : δz → 2·Z·δz + δz² + δc.
help-how-floatexp = Profondeur extrême (floatexp)
help-how-floatexp-text = Au-delà d’environ 1e28×, même cette infime différence sort par le bas de la plage 32 bits : elle est donc stockée sous forme d’une mantisse plus un exposant entier séparé (« floatexp »), ce qui lève le mur de profondeur des 32 bits. Le moteur bascule automatiquement : calcul direct en faible profondeur, perturbation en profondeur, et floatexp au plus profond. La profondeur est alors bornée par la précision des coordonnées et le budget d’itérations plutôt que par une limite fixe — vérifié par comparaison avec Fraktaler-3 sur un corpus de référence de 39 emplacements, le plus profond à 5.63e18003× (les 38 lignes non extrêmes forment la vérification de routine, jusqu’à 6.13e1105×), avec une batterie interne d’autocohérence de la précision jusqu’à 1e1000000× ; le parcours fourni atteint ~1.3e420×.
help-how-reference = Choix de la référence et rebasage
help-how-reference-text = La référence est choisie (évaluée en haute précision) pour que son orbite reste dans la vue le plus longtemps possible. Quand la différence devient trop grande, elle est « rebasée » sur la référence pour rester exacte.
help-how-series = Approximation par séries
help-how-series-text = En zoom profond, les premières itérations éloignent à peine δ de la référence : elles peuvent donc être calculées d’un seul coup. δz est approché par un court polynôme en δc (d’ordre 3), dont les coefficients sont itérés le long de la référence. Le moteur de rendu saute directement à la dernière itération où ce polynôme reste exact pour toute la vue, puis itère normalement — économisant les pas sautés sans rien changer à l’image (activable dans Contrôles ▸ Avancé). Mandelbrot et Multibrot 3–8, plage la plus profonde ; pas avec les colorations rayures / inégalité triangulaire / piège orbital / décomposition (elles ont besoin de chaque itération). Là où la BLA est active, elle englobe déjà ce saut initial.
help-how-distance = Estimation de distance et éclairage
help-how-distance-text = Suivre la dérivée dz/dc donne la distance de chaque pixel à la frontière. C’est ce qui alimente l’éclairage en relief 3D, la lueur de distance et la méthode de coloration « distance » — tous valables à n’importe quelle profondeur de zoom.
help-how-safety = Anticrénelage et sécurité
help-how-safety-text = Les images fixes sont suréchantillonnées (2–8× par axe) une fois la vue stabilisée. Un budget de travail maintient chaque tracé GPU sous la limite du chien de garde du pilote en réduisant la résolution (jamais le nombre d’itérations) aux réglages extrêmes, pour que les vues profondes restent détaillées au lieu de devenir vides.

## Help ▸ Command line (the flags themselves stay English)

# The section's introduction. The flags listed below it are the command line itself, English in every language.
help-cli-intro = Fractadyne fonctionne sans interface pour le rendu, l’exportation de films et de parcours, l’automatisation, les vérifications par images de référence et les bancs d’essai. Sans option, il lance l’interface graphique interactive.

## Help ▸ Shortcuts

# What each mouse gesture and key does (the gestures are the Navigation section's).
help-shortcuts-pan = Déplacer la vue
# A gesture: turning the mouse wheel.
help-key-wheel = Molette
help-shortcuts-wheel = Zoom au curseur
help-shortcuts-box-zoom = Zoom sur un cadre (tracer un rectangle pour zoomer dessus)
# $icon: the click-to-zoom toolbar button's symbol. "tool on": while that button is on.
help-key-click-tool = Clic (outil { $icon } actif)
help-shortcuts-click = Plonger vers le point ; le clic droit recule
# A heading: the keyboard shortcuts.
help-shortcuts-keyboard = Clavier
help-shortcuts-space = Zoom continu avant / arrière
help-shortcuts-undo = Vue précédente
help-shortcuts-redo = Vue suivante
help-shortcuts-minibrot = Trouver un minibrot et zoomer dessus (Newton-Raphson)
help-shortcuts-autopilot = Pilote automatique du zoom (plonger vers le détail ; toute action l’arrête)
# off / strip / panel / full: the panel's sizes, as View ▸ Live render panel names them.
help-shortcuts-live-panel = Panneau de rendu en direct : désactivé / bandeau / panneau / complet (survolez un instrument pour savoir ce qu’il signifie ; son × le ferme)
help-shortcuts-esc = Arrêter le pilote automatique / un parcours en lecture, ou quitter le plein écran
help-shortcuts-snapshot = Instantané dans le dernier dossier d’exportation — une capture d’écran de la vue telle qu’affichée, ou un rendu complet avec les paramètres d’exportation ; le premier appui demande lequel, et Fichier → Paramètres → Instantané permet de changer.
help-shortcuts-help = Ouvrir cette aide
# A heading: the shortcuts of the Game of Life, with the menu path to it.
help-shortcuts-life = Life (Fractale → Automates → Life)
help-shortcuts-life-run = Lancer / mettre en pause l’univers
help-shortcuts-life-step = Avancer d’une génération / avancer du pas réglé (dans le panneau Life)
help-shortcuts-life-draw = Outil de dessin activé / désactivé (le crayon de la barre d’outils)
# A gesture. "Draw on": while the Life draw tool is on.
help-key-click-drag-draw = Clic / glisser (dessin actif)
help-shortcuts-life-paint = Inverser une cellule / peindre : un trait qui commence sur une cellule morte dessine, un trait qui commence sur une cellule vivante efface. Un univers en marche se met en pause pendant que vous dessinez et reprend quand vous relâchez.
# $key: a keyboard shortcut (Ctrl+Z). "Draw on": while the Life draw tool is on.
help-key-when-drawing = { $key } (dessin actif)
help-shortcuts-life-undo = Annuler la dernière modification, le dernier remplissage, le dernier effacement ou le dernier changement de règle

## Help ▸ Recommended hardware

help-hardware-intro = Fractadyne répartit son travail entre le GPU (itération + coloration par pixel) et le CPU (l’orbite de référence en précision arbitraire sur laquelle repose le zoom profond). Les deux comptent, mais pour des choses différentes.
help-hardware-gpu = GPU (le plus important)
help-hardware-gpu-text = Chaque pixel est itéré et coloré sur le GPU : la vitesse du GPU détermine donc la fréquence d’images et la durée d’exportation. Un GPU dédié au bon débit en 32 bits (simple précision) donne le zoom profond le plus fluide. Les GPU intégrés fonctionnent bien — simplement plus lentement, et ils atteignent plus tôt le budget de travail par image (le moteur de rendu réduit alors la résolution, pas le détail). Nécessite une API graphique moderne : Vulkan, Direct3D 12, Metal, ou OpenGL via wgpu.
# The left column of a requirements table.
help-hardware-minimum = Minimum
help-hardware-gpu-minimum = N’importe quel GPU compatible Vulkan / DX12 / Metal / GL 3.3.
help-hardware-recommended = Recommandé
help-hardware-gpu-recommended = Un GPU dédié (des ~5 dernières années) ; >= 2 GB de VRAM.
help-hardware-big-exports = Pour les grandes exportations
help-hardware-gpu-big-exports = >= 4 GB de VRAM (rendu par tuiles, donc dégradation progressive).
# A heading: the processor.
help-hardware-cpu = CPU
help-hardware-cpu-text = L’orbite de référence en haute précision (et les tables BLA) sont calculées sur le CPU, en grande partie sur un seul fil : la performance par cœur compte donc le plus — c’est elle qui détermine la vitesse à laquelle une vue profonde se résout après un déplacement. Des cœurs supplémentaires aident surtout l’encodage des exportations en arrière-plan. Un CPU 64 bits est requis.
help-hardware-cpu-recommended = Un CPU 64 bits moderne avec une bonne vitesse par cœur.
help-hardware-memory = Mémoire
help-hardware-memory-text = Les références profondes grossissent avec la profondeur (de nombreuses itérations en haute précision) ; les très grandes exportations gardent des tuiles en RAM. 8 GB suffisent confortablement pour l’exploration courante ; 16 GB ou plus conviennent à la profondeur extrême et aux très grandes exportations.
help-hardware-checking = Vérifier votre matériel
help-hardware-diagnostics = Aide → « Diagnostics… » exécute sur votre machine les mêmes tests que les développeurs. « Lancer l’autotest » vérifie les mathématiques et le rendu par rapport à des résultats connus en une quinzaine de secondes ; « Lancer le test d’interface » parcourt l’interface et la vue en direct à plusieurs profondeurs de zoom, en prenant des captures d’écran ; « Lancer la vérification arithmétique GPU » vérifie l’arithmétique double-float du shader sur chaque backend graphique que propose votre machine, en quelques secondes et sans fenêtre. Chacun s’exécute dans un processus séparé : un test qui perturbe le pilote graphique fait tomber le test plutôt que votre session.
help-hardware-gpu-check = La vérification arithmétique GPU n’a ni réussite ni échec — elle enregistre la façon dont votre GPU et votre pilote calculent, et sur certains matériels (toutes les cartes NVIDIA essayées jusqu’ici) elle signale que le compilateur de shaders supprime une partie de l’arithmétique en précision étendue. C’est une véritable découverte que nous essayons de cartographier chez tous les fabricants, pas un défaut de votre machine : quel que soit son résultat, la joindre à un signalement aide vraiment.
help-hardware-report = Si quelque chose semble incorrect, lancez l’autotest et utilisez « Joindre à un signalement de problème… » : le signalement contient alors un résultat validé automatiquement sur votre propre matériel, bien plus utile qu’une simple description. Une réserve à connaître — les images de référence ont été produites sur une carte graphique particulière, et des GPU différents calculent légitimement des pixels légèrement différents : les comparaisons d’images sont donc vérifiées avec une tolérance plus large sur d’autres matériels, et de petites différences y sont attendues plutôt que des défauts.
help-hardware-notes = Remarques
help-hardware-note-watchdog = Un GPU performant évite que le budget de travail du chien de garde n’intervienne aux réglages extrêmes.
help-hardware-note-binary = Le programme tient en un seul binaire autonome — aucune installation d’environnement d’exécution n’est nécessaire.
help-hardware-note-benchmark = Lancez le banc d’essai intégré (Outils ▸ Banc d’essai) pour évaluer votre machine.

## Help ▸ Acknowledgments & citations
## Names of people, programs, papers, books and websites stay as written, and so do the
## titles in quotation marks (the works' own titles); the prose around them translates.

help-ack-title = Remerciements et citations
# The dedication, in italics at the top of the section.
help-ack-dedication = Dédié au Stone Soup Group — les bénévoles derrière Fractint (depuis 1988).
help-ack-dedication-text = Leur code librement partagé et leur esprit d’exploration fractale ouverte et collaborative sont la raison pour laquelle tant d’entre nous sont tombés amoureux de cet art. Fractadyne s’inscrit dans cette tradition.
help-ack-intro = Fractadyne repose sur des décennies de travail de la communauté fractale. Les techniques et travaux antérieurs sur lesquels il s’appuie — vérifiés d’après les sources ci-dessous :
help-ack-deep-zoom = Algorithmes de zoom profond
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-martin-title = Perturbation et approximation par séries — { $who }
help-ack-martin = La méthode orbite de référence + delta en basse précision (δz → 2Z·δz + δz² + δc) et la série d’ordre n qui saute les premières itérations, introduites dans SuperFractalThing et sa note « SuperFractalThing Maths » (sft_maths.pdf, 2013).
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bla-title = Approximation linéaire bivariée (BLA) et rebasage — { $who }
help-ack-bla = Sauter des séries d’itérations grâce à des applications linéaires fusionnées, et rebaser sur une référence unique pour éviter les glitches ; développé sur fractalforums.org (2021).
# A link's label. $who: the linked page's author.
help-ack-bla-link = La BLA expliquée — { $who }
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-glitch-title = Critère de détection des glitches — { $who }
help-ack-glitch = Le test |Z + z| << |Z| qui détermine quand un pixel a besoin d’une autre référence — la base de la perturbation sans glitches (fractalforums).
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-diffabs-title = Perturbation non analytique (Burning Ship) — { $who }
help-ack-diffabs = L’analyse par cas de la valeur absolue, « diffabs », qui fait fonctionner la perturbation pour les familles fondées sur la valeur absolue (Burning Ship / Celtic / Buffalo), sur fractalforums.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mathr-title = Implémentations de référence et vérifications croisées — { $who }
help-ack-mathr = Fraktaler-3 et Kalles Fraktaler 2+ (à l’origine de Karl Runmo) — dont les écrits ont guidé ce moteur, et dont la sortie EXR sert à Fractadyne pour vérifier son moteur de rendu.
# A heading: credits for the coloring methods.
help-ack-coloring = Coloration
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-harkonen-title = Nombre d’itérations lissé et moyenne de rayures (Stripe Average) — { $who }
help-ack-harkonen = La coloration continue (normalisée) par temps d’échappement et la méthode Stripe Average, tirées de « On Smooth Fractal Coloring Techniques » (mémoire de M.Sc., 2007), qui analyse aussi les colorations Triangle Inequality / moyenne de branchement.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mitchell-title = Moyenne d’inégalité triangulaire (Triangle Inequality Average) — { $who }
help-ack-mitchell = Moyenne des positions où chaque z² + c tombe dans les bornes de l’inégalité triangulaire, pour une texture extérieure lisse, en forme de flammes.
# A link's label: the person's tutorial pages. $who: the person's name.
help-ack-mitchell-link = { $who } — tutoriels
help-ack-distance = La coloration et l’éclairage par estimation de distance utilisent la méthode classique de distance à la frontière par dz/dc (voir Peitgen et Saupe, « The Science of Fractal Images », 1988).
help-ack-lsystems-life = L-systèmes et Jeu de la vie
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-lindenmayer-title = L-systèmes — { $who }
help-ack-lindenmayer = La réécriture parallèle de chaînes, introduite en 1968 pour modéliser la croissance des plantes et des algues (« Mathematical models for cellular interactions in development », Journal of Theoretical Biology).
help-ack-abop = Le livre (1990) dont le lecteur de L-systèmes suit l’interprétation par la tortue et la notation des systèmes stochastiques, contextuels et paramétriques ; les systèmes intégrés marqués ABOP sont saisis d’après ses figures.
# A link's label: the book can be read free on that website. $site: the website's name.
help-ack-abop-link = Le livre, gratuit sur { $site }
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-bourke-title = Collection de L-systèmes — { $who }
help-ack-bourke = Beaucoup des systèmes intégrés viennent de ses pages sur les L-systèmes, parmi lesquels les règles de Chris Wallace pour la courbe de Sierpinski et le buisson de Dietmar Saupe. Les fichiers .l de Fractint, qui s’ouvrent directement, sont au format du Stone Soup Group.
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-conway-title = Le Jeu de la vie — { $who }
help-ack-conway = L’automate cellulaire B3/S23 (1970), qui a atteint le grand public grâce à la rubrique « Mathematical Games » de Martin Gardner dans Scientific American (octobre 1970).
# A credit's heading: the website, then who runs it. $site: the website's name.
help-ack-lifewiki-title = { $site } — la communauté ConwayLife.com
help-ack-lifewiki = La référence pour les notations de règles que l’application lit (B/S, von Neumann, Generations, les lettres d’Alan Hensel pour les règles non totalistes, et MAP), pour le format de motifs RLE qu’elle partage avec Golly, et pour les règles et motifs intégrés, dont elle recense les découvreurs.
help-ack-foundations = Fondations et outils
# A credit's heading: what was contributed, then by whom. $who: the person's name, as written.
help-ack-mandelbrot-title = L’ensemble de Mandelbrot — { $who }
help-ack-mandelbrot = L’objet au cœur de tout cela.
help-ack-built-with = Construit avec Rust, egui/eframe (emilk) et wgpu (gfx-rs) ; précision arbitraire grâce à astro-float, flottants double-mot grâce à twofloat ; PNG/OpenEXR, rfd, serde. Merci à leurs auteurs et mainteneurs.
help-ack-errors = Toute erreur d’attribution est le fait de l’auteur ; les corrections sont les bienvenues sur le gestionnaire de tickets du projet.

## Help ▸ Third-party licenses

help-licenses-title = Licences tierces
# $licenses: Fractadyne's license expression (MIT OR Apache-2.0), as written. $url: the web address of a crate's source. THIRD-PARTY-NOTICES.md is a file name, `option-ext` a crate's name, MPL-2.0 a license's.
help-licenses-intro = Fractadyne est distribué sous forme de binaire lié statiquement qui inclut les crates open source et les polices fournies ci-dessous. Les textes complets des licences (également dans THIRD-PARTY-NOTICES.md, inclus dans le téléchargement) suivent. Le code propre de Fractadyne est sous double licence { $licenses }. `option-ext` est sous MPL-2.0 — sa source non modifiée se trouve à { $url }.
# A button, after a copy icon: copies every notice to the clipboard.
help-licenses-copy-all = Copier toutes les mentions

## Help ▸ About

help-about-tagline = Un explorateur de fractales natif écrit en Rust avec wgpu.
# The arithmetic deep zoom has used. $ran: the libraries that have run so far, as names (or a note that none has yet); $built: those this build contains, with versions.
help-about-arithmetic = Arithmétique du zoom profond : { $ran } (cette version contient : { $built }).
help-about-files = Fichiers et données
# Followed by the folder's path.
help-about-files-text = Votre session (vue actuelle, coloration et préférences), vos signets et leurs miniatures, vos dégradés et vos formules enregistrés sont stockés dans ce dossier propre à l’utilisateur, et rechargés au prochain lancement :
# Tooltip of the copy button beside the folder's path.
help-about-copy-path = Copier le chemin
# $command: a command typed in a terminal (it stays English).
help-about-session = Le fichier de session est versionné ; s’il a été écrit par une version de Fractadyne plus récente que celle qui peut le lire entièrement, l’application le charge au mieux et vous avertit. Pour repartir complètement de zéro, utilisez Fichier ▸ Réinitialiser l’état de l’application… (une confirmation est demandée), ou lancez `{ $command }` depuis un terminal.
help-about-license = Licence
# $licenses: the license expression (MIT OR Apache-2.0), which stays as written.
help-about-license-text = { $licenses } — utilisation sous l’une ou l’autre licence, à votre choix.
# A link to the source code, followed by ↗.
help-about-source = Code source sur GitHub

## The live render panel ("blinkenlights": View ▸ Live render panel, key B)
## A front panel drawn over the view like a piece of 1970s test equipment. Its legends are engraved
## in capitals: write them in capitals here (the panel shows them exactly as written, it does not
## uppercase them). Numbers, units and readouts on its tubes and dials are not translated.

# Tooltip of the × button in the panel's top-right corner. The code adds the key after it: (B).
live-close-hover = Fermer le panneau de rendu en direct
# The legend engraved along the top plate, left of the maker's badge.
live-plate = RENDU EN DIRECT · PANNEAU FRONTAL

## Live render panel: instrument titles
## Engraved above each instrument, and repeated as the title of its help card.

# Above the depth readout, whose tubes show the magnification as a power of two ("2^" stays).
live-title-depth = PROFONDEUR 2^
live-title-iterations = ITÉRATIONS
# Above the frame-time readout (milliseconds).
live-title-frame-ms = IMAGE MS
# Above the date-and-time readout.
live-title-clock = DATE · HEURE
# The walk dial: a radar screen showing a slow refresh rendered pass after pass, band after band.
live-title-walk = PROGRESSION · BANDES
# A meter of how far the picture on screen lags the view.
live-title-age = ÂGE DE L’IMAGE
# What part of the picture this frame's GPU pass covered.
live-title-tiles = TUILES
# A scope of the GPU's work per frame, in milliseconds.
live-title-scope = PASSES GPU · MS
# The depth dial: depth, the reference orbits being built, and the arithmetic in use.
live-title-dial = PROFONDEUR · RÉFÉRENCES · ARITHMÉTIQUE
# Iteration steps per second.
live-title-steps = PAS / S
# The two GPUs (the window's and a second, worker GPU).
live-title-engines = DEUX MOTEURS
live-title-memory = MÉMOIRE
live-title-cores = CŒURS · THREADS
live-title-pixels = FLUX DE PIXELS
# Help card titles only: the three GPU timer lamps (T0, T1, T2 stay as they are) and the row of lamps.
live-title-timers = CHRONOMÈTRES GPU T0–T2
live-title-bank = RANGÉE DE VOYANTS

## Live render panel: the light bank's lamps
## Engraved under each lamp, and the title of its help card. A cell is narrow (about eight capital
## letters): a longer label breaks at a space onto two lines, and is drawn smaller if it still does
## not fit.

live-lamp-plan = PLAN
live-lamp-dispatch = ENVOI
live-lamp-live = ACTUEL
live-lamp-held = MAINTENU
# The last picture reprojected (stretched) to fit the moving view.
live-lamp-reproj = REPROJ
live-lamp-reading = LECTURE
# A pass of a pinned walk.
live-lamp-pin-pass = PASSE ÉPINGLÉE
live-lamp-split = SCINDÉ
live-lamp-adopt = ADOPTION
live-lamp-abandon = ABANDON
# "REF": a reference orbit. Built for the view as it is (LIVE), ahead of it (AHEAD), or for a tour's next hold (HOLD).
live-lamp-ref-live = RÉF ACTUELLE
live-lamp-ref-ahead = RÉF AVANCE
live-lamp-ref-hold = RÉF PAUSE
# The series approximation.
live-lamp-series = SÉRIES
# The second (worker) GPU.
live-lamp-worker = RENFORT
live-lamp-tile = TUILE
# Supersampling (anti-aliasing); usually kept as SSAA.
live-lamp-ssaa = SSAA
live-lamp-palette = PALETTE
# The arithmetic in use (direct, df32, floatexp).
live-lamp-mode = MODE

## Live render panel: the kinds of GPU pass
## In the GPU scope's legend (MOVING, PIN, WORKER, SETTLE, SSAA). SETTLE and SSAA also name a still
## view's walk on the walk dial.

# A moving frame's pass.
live-kind-moving = MOUVEMENT
# A pass of a pinned walk.
live-kind-pin = ÉPINGLE
# The worker GPU's passes.
live-kind-worker = RENFORT
# A still view's settle tile or walk pass.
live-kind-settle = FINITION
# A supersampling sample.
live-kind-ssaa = SSAA

## Live render panel: legends and readouts on the instruments

# On the walk dial during a split refresh. $done: passes done; $passes: passes in all.
live-walk-split = SCINDÉ { $done }/{ $passes }
# On the picture-age meter's face, under its scale (0 to 2 octaves).
live-age-scale = OCTAVES DE RETARD
# The depth dial's band for direct (plain floating-point) arithmetic. The other bands, DF32 and FLOATEXP, are names and stay.
live-dial-direct = DIRECT
# Under the two GPU gauges: GPU 0 draws the window; GPU 1 is the worker, or there is none.
live-engines-gpu-window = GPU 0 · FENÊTRE
live-engines-gpu-worker = GPU 1 · RENFORT
live-engines-no-worker = SANS RENFORT
# The share of displayed frames each GPU made. $window and $worker: percentages, 0–100.
live-engines-on-screen = À L’ÉCRAN { $window } : { $worker }
# The same readout before any frame was shown.
live-engines-on-screen-none = À L’ÉCRAN —
# Under the CPU core bars. $count: the machine's cores.
live-cores-count = { $count } CŒURS
# Under the threads scope. $count: the app's busy threads.
live-cores-threads = THREADS { $count }

## Live render panel: the memory tanks' legend
## What the GPU memory holds, by kind. Very narrow: about ten letters.

# G-buffers (the picture's working textures).
live-memory-gbuf = G-BUF
# The held picture and the supersampling accumulator.
live-memory-hold = IMAGE+SSAA
# The reference orbit and its BLA (bilinear approximation) table.
live-memory-orbit = ORBITE+BLA
# The worker GPU's copy of the reference.
live-memory-worker = RÉF RENFORT

## Live render panel: the roles on the threads scope
## Tiny labels on a small scope: at most four letters each.

# The reference builders.
live-cores-role-ref = RÉF
# The scoring pool.
live-cores-role-pool = POOL
# The thread driving the worker GPU.
live-cores-role-worker = RENF
# Probes: short side renders.
live-cores-role-probe = SOND
# Image and tour exports.
live-cores-role-export = EXP

## Live render panel: the tiles instrument's captions
## The line under the picture's plane, then the tile size (left) and the tiles a frame (right).

# A walk's pass. $lo and $hi: the iteration band it rendered (numbers).
live-tiles-pin-band = PASSE ÉPINGLÉE · BANDE { $lo }–{ $hi }
live-tiles-settle-band = PASSE DE FINITION · BANDE { $lo }–{ $hi }
live-tiles-moving-whole = IMAGE EN MOUVEMENT · TOUTE L’IMAGE
live-tiles-settled-whole = IMAGE STABILISÉE · TOUTE L’IMAGE
# $pass: this pass; $passes: the passes the refresh is split into.
live-tiles-split-pass = RAFRAÎCHISSEMENT SCINDÉ · PASSE { $pass } SUR { $passes }
# $grid: the grid's columns × rows, e.g. 12×8.
live-tiles-grid-done = GRILLE DE FINITION { $grid } · TERMINÉE
# $grid: the grid's columns × rows; $done and $total: tiles done, and in all.
live-tiles-grid-progress = GRILLE DE FINITION { $grid } · { $done } SUR { $total } FAITES
# The same, while a tile too costly for one frame goes out in strips.
live-tiles-grid-strips = GRILLE DE FINITION { $grid } · { $done } SUR { $total } FAITES · EN TRANCHES
# $samples: supersampling samples folded so far; $lo and $hi: the iteration band the next one is walking.
live-tiles-ssaa-band = SSAA · { $samples } FAITS · BANDE { $lo }–{ $hi }
# $sample: the supersampling sample.
live-tiles-ssaa-whole = ÉCHANTILLON SSAA { $sample } · IMAGE ENTIÈRE
# No GPU pass this frame, and the panel has not seen a frame yet.
live-tiles-idle-starting = DÉMARRAGE
# No GPU pass this frame: the last complete picture is held on screen.
live-tiles-idle-held = MAINTENU
# No GPU pass this frame: the last picture is stretched to fit the view.
live-tiles-idle-reprojected = REPROJETÉ
# No GPU pass this frame: the picture on screen is current.
live-tiles-idle-live = ACTUEL
# No GPU pass this frame, and nothing to show.
live-tiles-idle-none = INACTIF
# The tile size: $res, a whole picture's resolution (2400×1300).
live-tiles-tile-res = TUILE { $res }
# The tile size: $px, a tile's side in pixels.
live-tiles-tile-px = TUILE { $px } PX
live-tiles-no-pass = AUCUNE PASSE
# Tiles a frame. $count: how many (0 when none).
live-tiles-rate-count = { $count }/IMAGE
live-tiles-rate-one = 1 TUILE/IMAGE
# $share: the part of one tile sent each frame, e.g. 0.33.
live-tiles-rate-part = { $share } TUILE/IMAGE
live-tiles-rate-sample = 1 TUILE/ÉCHANTILLON
# In the plane's corner during a walk: the share of pixels still iterating. $percent: a percentage, e.g. 54 or 1.5.
live-tiles-running = EN COURS { $percent } %

## Live render panel: help cards
## Shown when the pointer rests on an instrument or a lamp. Words in capitals (SETTLE, SSAA, ABANDON,
## HELD, REPROJ, ADOPT, REF AHEAD, ON SCREEN) are the panel's own legends: use the same words as
## their messages above.

# The heading of a card's list of what healthy looks like.
live-card-healthy = SAIN
# The heading of a card's list of what signals trouble.
live-card-trouble = PROBLÈME

live-card-walk-what = Un rafraîchissement profond trop coûteux pour une seule image est épinglé et rendu en progression : passe après passe, chacune laissant les pixels itérer une octave plus loin (256, 512, 1 024 … jusqu’au nombre demandé). Le pourtour représente ce nombre sur une échelle logarithmique, en partant du haut ; le faisceau est l’avancement de la progression ; chaque arc est une passe, colorée selon son temps GPU par rapport à la cible de 20 ms. Le point blanc attend la mesure de la dernière passe ; l’éclair est la nouvelle image qui s’affiche.
live-card-walk-note = Une vue fixe trop coûteuse pour une seule passe progresse de la même façon, marquée FINITION, et chaque échantillon de suréchantillonnage progresse à son tour, marqué SSAA : leurs passes peuvent prendre chacune jusqu’à 400 ms, et les arcs sont colorés par rapport à cette durée. Un rafraîchissement scindé dessine plutôt un anneau par passe : chaque bande, une part des tuiles.
live-card-walk-good-sweep = Un balayage qui fait le tour en une poignée d’images et se termine par un éclair.
live-card-walk-good-arcs = Des arcs surtout verts et ambre.
live-card-walk-good-still = Sur une vue fixe profonde : un balayage plus lent par échantillon, aux arcs verts à ambre.
live-card-walk-bad-red = Des arcs rouges : des passes hors budget, ressenties comme des saccades.
live-card-walk-bad-restart = Un balayage qui s’arrête en chemin et recommence : la progression a été abandonnée, et l’image à l’écran reste ancienne.
live-card-walk-bad-hollow = Des arcs creux : des passes dont la mesure n’est jamais revenue, si bien que la passe suivante est dimensionnée à l’aveugle.

live-card-age-what = Le retard de l’image à l’écran sur la vue, en octaves de zoom (une octave vaut 2×). Pendant le rendu d’un rafraîchissement, l’ancienne image est affichée étirée et vieillit ; quand le rafraîchissement arrive, l’âge chute. La bande du bas montre les dernières secondes.
live-card-age-good-sawtooth = Une dent de scie entre environ 0.3 et 0.8, l’aiguille oscillant autour de la marque 0.5.
live-card-age-good-still = Proche de 0 une fois que vous vous arrêtez.
live-card-age-bad-red = L’aiguille qui reste au-delà de 1.0, dans le rouge : l’image a plus de 2× de retard et paraît floue ou pixelisée pendant le zoom.
live-card-age-bad-ramp = Une rampe qui ne redescend jamais : les rafraîchissements ne se terminent pas (surveillez ABANDON).

live-card-tiles-what = La façon dont la passe GPU de cette image couvre l’image. Les images en mouvement et les passes de progression la couvrent entièrement, et la légende indique la bande d’itérations d’une passe de progression. Une passe de progression n’itère que les pixels qui ne se sont pas encore échappés : le plan se remplit à hauteur de cette part (échelle logarithmique, 0,01 % au plancher) et l’affiche dans le coin, si bien qu’il se vide à mesure que la progression va plus loin. Un rafraîchissement un peu trop coûteux pour une seule image est scindé en 8 passes au plus, chacune étant un ensemble entrelacé de tuiles de 16 px. Une vue stabilisée trop coûteuse pour une seule passe devient une grille de tuiles, une par image, du centre vers l’extérieur, et une tuile encore trop coûteuse part en tranches. Affichés en dessous : la taille des tuiles et le nombre de tuiles par image.
live-card-tiles-good-whole = Des passes entières pendant le zoom ; des rafraîchissements scindés en quelques passes.
live-card-tiles-good-grid = Quand vous vous arrêtez : une grille de quelques dizaines de tuiles remplie en une ou deux secondes, puis SSAA.
live-card-tiles-bad-small = Une grille de centaines de petites tuiles : le budget de chaque passe est faible, et la netteté mettra longtemps à venir.
live-card-tiles-bad-strips = Beaucoup de tuiles qui partent en tranches : les tuiles ont été plus lentes que prévu au dimensionnement de la grille.
live-card-tiles-bad-splits = Des rafraîchissements bloqués à 8 passes : le rafraîchissement tient tout juste, et un peu plus profond il retombe sur la progression, plus lente.
live-card-tiles-bad-dark = Sombre pendant de longues périodes en mouvement : aucune passe du tout.

live-card-scope-what = Une colonne par image, la plus récente à droite : le travail GPU qu’elle a exécuté, empilé par type (image en mouvement en vert, passe épinglée du vert au rouge selon son coût, tuile de finition ou passe de progression d’une vue fixe en crème, échantillon de suréchantillonnage en lavande, images et échantillons du GPU de renfort en bleu), en millisecondes. Lignes pointillées à 16.7 ms (60 fps), 33 et 50. Une marque creuse est une passe dont la mesure n’est jamais revenue.
live-card-scope-good-under = Des colonnes sous la ligne 16.7 pendant le mouvement.
live-card-scope-good-pin = Des passes épinglées du vert à l’ambre.
live-card-scope-good-samples = Les échantillons de suréchantillonnage d’une vue fixe peuvent durer plus longtemps : rien ne bouge.
live-card-scope-bad-over = Des colonnes au-delà de 33 ou 50 ms : saccades visibles.
live-card-scope-bad-seconds = Des passes de plusieurs secondes : le chien de garde du GPU risque de réinitialiser le périphérique.
live-card-scope-bad-hollow = Beaucoup de marques creuses : le travail est dimensionné sans mesures.

live-card-timers-what = Les trois emplacements d’horodatage du GPU. Un emplacement s’allume quand une passe est chronométrée et s’éteint quand sa mesure revient, deux ou trois images plus tard. T0 chronomètre la passe de chaque image ; T1 et T2 chronomètrent les passes épinglées supplémentaires.
live-card-timers-good = T0 qui clignote dès qu’il y a du travail ; T1 et T2 qui clignotent pendant les progressions.
live-card-timers-bad-stuck = Un voyant bloqué allumé : sa mesure ne revient pas, et les estimations de coût vieillissent.
live-card-timers-bad-dark = Tout éteint alors que le GPU est occupé : rien n’est mesuré.

live-card-dial-what = L’aiguille rouge est la profondeur de la vue : son grossissement en puissance de deux. Les bandes sont l’arithmétique qu’y utilise le GPU : direct, df32 (deux flottants par nombre) et floatexp (un flottant avec un large exposant) au-delà de 2^93. Les points sont des orbites de référence en construction : vert pour la vue, ambre en avant d’elle, là où se dirige le zoom, bleu pour la prochaine pause d’un parcours. Un anneau est une installation ; une croix rouge est une construction abandonnée parce que la vue l’a dépassée.
live-card-dial-good-ahead = Des points ambre qui apparaissent devant l’aiguille et s’entourent d’un anneau avant qu’elle ne les atteigne.
live-card-dial-good-crosses = Peu de croix rouges.
live-card-dial-bad-passing = L’aiguille qui dépasse sa dernière référence installée : les images en attendent une, ce qui se ressent comme un gel.
live-card-dial-bad-crosses = Des croix rouges répétées : le zoom va plus vite que les constructeurs, et du CPU est gaspillé.
live-card-dial-bad-crossing = Le passage de 2^93 est un point lent connu : les premières images floatexp coûtent plus cher.

live-card-bank-what = Chaque voyant s’allume quand son événement se produit et s’estompe en environ 150 ms : un moteur de rendu occupé scintille et un moteur inactif s’éteint. De gauche à droite : l’image, le rafraîchissement, les références, puis le renfort, la finition, la palette et l’arithmétique.
live-card-bank-note = Survolez un voyant pour savoir ce qu’il signifie.

live-card-depth-what = Le grossissement en puissance de deux : 2^100 vaut environ 1,3 × 10^30.
live-card-depth-note = Au-delà de 2^93, chaque pas coûte plus cher (floatexp), et les références sont plus longues à construire.

live-card-iter-what = Le nombre d’itérations demandé : jusqu’où chaque pixel peut itérer avant d’être compté comme intérieur à l’ensemble. Il augmente avec la profondeur.
live-card-iter-good = Une montée régulière pendant la plongée.
live-card-iter-bad-low = Bloqué bas alors que la vue est profonde : spirales et minibrots s’affichent comme un intérieur uni.
live-card-iter-bad-high = Très élevé : chaque rafraîchissement coûte plus cher, et les progressions prennent plus de temps.

live-card-ms-what = Le temps entre deux images affichées.
live-card-ms-good = L’intervalle de votre écran : 16.7 sur un écran à 60 Hz.
live-card-ms-bad-stutter = Au-dessus de 33 : saccades visibles.
live-card-ms-bad-spikes = Des pics au-dessus de 50 : des à-coups, généralement une passe coûteuse (voir PASSES GPU).

live-card-clock-what = L’horloge en heure locale, au dixième de seconde.
live-card-clock-note = Elle date toute capture d’écran du panneau du moment où elle a été prise : une image d’un problème peut ainsi être rapprochée du journal de l’application.

live-card-tach-what = Pas d’itération par seconde. Blanc : les pas demandés (pixels × itérations). Ambre : les pas réellement exécutés par le GPU. L’écart, c’est BLA (approximation bilinéaire) qui saute de nombreuses itérations d’un coup ; ×N est ce multiplicateur. Le compteur totalise la session.
live-card-tach-good = Un large écart en profondeur : BLA fait son travail, de ×10 à ×1 000.
live-card-tach-bad-together = Aiguilles confondues en profondeur : BLA ne peut rien sauter ici (souvent près de l’intérieur d’un minibrot), donc les rafraîchissements sont lents.
live-card-tach-bad-red = L’aiguille blanche dans le rouge : la demande dépasse ce que le GPU peut faire en une image.

live-card-engines-what = L’occupation de chaque GPU, avec des points qui portent ses images vers l’écran : le temps GPU de ses passes sur la dernière seconde, images en mouvement et échantillons de suréchantillonnage du renfort compris. À L’ÉCRAN est la part des images affichées produite par chacun. Avec un seul GPU, le côté renfort reste inactif.
live-card-engines-good = Les deux occupés pendant une plongée, le renfort fournissant une part des images.
live-card-engines-bad-pinned = GPU 0, celui de la fenêtre, bloqué près de 100 % : c’est le goulot d’étranglement, et les images saccadent.
live-card-engines-bad-idle = Un renfort activé mais inactif : il ne fournit pas d’images.

live-card-memory-what = RAM : l’ensemble de travail de l’application, la ligne rouge marquant son pic. VRAM : ce que l’application occupe sur le GPU, calculé d’après les tailles qu’elle alloue, par type (G-buffers, textures de maintien et SSAA, tampons d’orbite et BLA, copie de la référence du renfort), par rapport à la carte.
live-card-memory-good = Stable pendant le zoom ; monte et redescend par paliers à mesure que les références vont et viennent.
live-card-memory-bad-leak = La RAM qui monte sans jamais redescendre : une fuite.
live-card-memory-bad-vram = La VRAM proche de la taille de la carte : des allocations peuvent échouer, ou le périphérique peut être perdu.

live-card-cores-what = L’utilisation CPU par cœur, du vert à l’ambre puis au rouge, et un oscilloscope où chaque thread occupé orbite autour de son rôle : les constructeurs de références, le pool d’évaluation, le renfort, les rendus annexes.
live-card-cores-good = Un ou deux cœurs occupés pendant le zoom ; une courte rafale sur tous les cœurs quand une référence est choisie.
live-card-cores-bad-every = Tous les cœurs occupés pendant des secondes : construction de référence à profondeur extrême.
live-card-cores-bad-own = Le cœur de l’application elle-même bloqué dans le rouge : la fenêtre elle-même saccade.

live-card-pixels-what = Pixels calculés par seconde, en Mpx/s. Pâle : encore en cours d’itération dans une progression. Vif : terminé.
live-card-pixels-good = Des dizaines de Mpx/s en mouvement.
live-card-pixels-bad = Presque zéro alors que l’image est périmée : le GPU attend quelque chose, une référence ou une mesure.

## Live render panel: the lamps' help cards

live-card-lamp-plan-what = Une image a été planifiée : résolution, itérations demandées et budget choisis, et décision de rafraîchir ou non.
live-card-lamp-plan-good = Scintille chaque fois que la vue ou les réglages changent.
live-card-lamp-plan-bad = Éteint pendant que vous zoomez : le fil de l’interface est bloqué.

live-card-lamp-dispatch-what = Du travail a été envoyé au GPU pour cette image.
live-card-lamp-dispatch-good = Allumé dès que quelque chose change ; éteint au repos.
live-card-lamp-dispatch-bad = Occupé alors que rien ne change : l’application ne se stabilise pas.

live-card-lamp-live-what = L’image à l’écran est le rendu propre de l’image en cours.
live-card-lamp-live-note = Rarement allumé pendant une plongée profonde, ce qui est normal : voir MAINTENU et REPROJ.
live-card-lamp-live-good = Allumé aux faibles profondeurs, où une image coûte peu.

# "Prefer detail" is a setting's name.
live-card-lamp-held-what = La dernière image complète est à l’écran pendant que la suivante se compose en dessous : le rafraîchissement d’un zoom profond ou, avec « Privilégier le détail pendant le zoom », la prochaine tuile ou le prochain échantillon de suréchantillonnage d’une vue fixe.
live-card-lamp-held-good-dive = Allumé par périodes pendant les zooms profonds, interrompu par ADOPTION.
live-card-lamp-held-good-still = Allumé en continu pendant qu’une vue fixe s’affine, avec SSAA qui clignote.
live-card-lamp-held-bad = Allumé pendant des secondes en zoomant, sans ADOPTION : les rafraîchissements ne se terminent pas.

live-card-lamp-reproj-what = La dernière image, étirée pour s’ajuster à la vue en mouvement jusqu’à ce qu’une nouvelle soit prête.
live-card-lamp-reproj-good = De courtes rafales pendant le mouvement.
live-card-lamp-reproj-bad = Allumé longtemps : l’image devient d’autant plus floue qu’elle est étirée.

live-card-lamp-reading-what = Une mesure est revenue du GPU : le temps d’une passe, une plage d’échappement ou un contenu.
live-card-lamp-reading-good = Un scintillement régulier pendant le rendu.
live-card-lamp-reading-bad = Éteint longtemps pendant le rendu : les temps ne reviennent pas, et les budgets sont estimés au jugé.

live-card-lamp-pin-pass-what = Une passe d’une progression épinglée : l’image rendue une bande d’itérations plus loin.
live-card-lamp-pin-pass-good = Des séries de passes qui se terminent par ADOPTION.
live-card-lamp-pin-pass-bad = Des séries qui se terminent par ABANDON.

live-card-lamp-split-what = Une passe d’un rafraîchissement scindé : une part des tuiles, rendue avec le nombre complet d’itérations.
live-card-lamp-split-good = De courtes séries qui se terminent par ADOPTION.
live-card-lamp-split-bad = Des séries de 8, le maximum autorisé : le rafraîchissement tient tout juste.

live-card-lamp-adopt-what = Un rafraîchissement terminé a remplacé l’image à l’écran.
live-card-lamp-adopt-good = Plusieurs fois par seconde pendant le zoom.
# "Picture age" names the instrument (its title, PICTURE AGE).
live-card-lamp-adopt-bad = Rare pendant le zoom : l’image vieillit (voir Âge de l’image).

live-card-lamp-abandon-what = Un rafraîchissement en cours a été jeté : la vue s’est trop déplacée, il est devenu trop ancien, ou un plus récent l’a remplacé. Une construction de référence dépassée par la vue l’allume faiblement.
live-card-lamp-abandon-good = Rare, ou une fois juste après un saut.
live-card-lamp-abandon-bad = Fréquent : du travail gaspillé, et l’image reste ancienne.

live-card-lamp-ref-live-what = Une orbite de référence est en construction pour la vue telle qu’elle est : elle n’en a pas, a quitté celle qu’elle avait, ou en demande une plus longue.
live-card-lamp-ref-live-good = De brefs clignotements.
live-card-lamp-ref-live-bad-long = De longues périodes : les vues profondes l’attendent.
live-card-lamp-ref-live-bad-lookahead = Souvent allumé pendant une plongée alors que RÉF AVANCE est éteint : l’anticipation ne suit pas.

live-card-lamp-ref-ahead-what = Une référence d’anticipation est en construction plus profond que la vue, là où se dirige le zoom.
live-card-lamp-ref-ahead-good = Des clignotements réguliers pendant une plongée.
live-card-lamp-ref-ahead-bad = Éteint pendant une plongée profonde : la vue va dépasser ses références.

live-card-lamp-ref-hold-what = Le préchargement de pause d’un parcours : la référence de la prochaine image clé avec pause, construite pendant le glissement vers elle pour que la lecture n’attende jamais.
live-card-lamp-ref-hold-note = Éteint en dehors de la lecture d’un parcours.
live-card-lamp-ref-hold-good = Un clignotement avant chaque pause d’un parcours.
live-card-lamp-ref-hold-bad = Allumé pendant toute une pause : le préchargement est en retard, et le parcours l’attend.

live-card-lamp-series-what = L’approximation par séries d’une nouvelle référence, qui permet à chaque pixel de sauter ses premières itérations.
live-card-lamp-series-good = Bref.
live-card-lamp-series-bad = Long en profondeur : retarde l’installation de la référence.

live-card-lamp-worker-what = Le second GPU a fourni une image.
live-card-lamp-worker-good = Des clignotements réguliers quand un GPU de renfort est activé.
live-card-lamp-worker-bad = Éteint alors qu’un renfort est activé : il ne fournit rien.

live-card-lamp-tile-what = Une tuile de finition, ou une tranche de tuile, est partie au GPU.
live-card-lamp-tile-good = Une rafale après l’arrêt, puis le calme.
live-card-lamp-tile-bad = Ne s’arrête jamais : la grille ne cesse de recommencer, car quelque chose change sans cesse la vue ou les réglages.

live-card-lamp-ssaa-what = Un échantillon de suréchantillonnage a été ajouté à l’image stabilisée.
live-card-lamp-ssaa-good = Une série de clignotements après une finition, puis éteint.
live-card-lamp-ssaa-bad = Recommence encore et encore : la finition est sans cesse perturbée.

live-card-lamp-palette-what = La normalisation des couleurs a pris en compte une nouvelle plage d’échappement : les couleurs suivent donc le zoom.
live-card-lamp-palette-good = Scintille pendant le zoom.
live-card-lamp-palette-bad = Éteint pendant le zoom : la plage de la palette est figée, et les couleurs deviennent plates et peu contrastées.

live-card-lamp-mode-what = L’arithmétique a changé : direct, df32 ou floatexp.
live-card-lamp-mode-good = Un clignotement à une frontière comme 2^93.
live-card-lamp-mode-bad = Des clignotements d’avant en arrière : la vue oscille autour d’une frontière.

## The Render client window (File ▸ Render client…): lending this machine's GPU to a render farm

farm-client-title = Client de rendu
farm-client-intro = Calculer des images pour une ferme de rendu. Le contrôleur — Outils ▸ Rendre sur la ferme… sur une autre machine — affiche son adresse et sa clé de ferme. Cette machine n’ouvre aucun port : c’est elle qui se connecte au contrôleur.
# A field label, and the label of the status row that names the controller.
farm-client-controller = Contrôleur
farm-client-find = Chercher
farm-client-find-hover = Chercher des contrôleurs sur ce réseau (un contrôleur répond tant que sa ferme est à l’écoute)
farm-client-key = Clé de ferme
# A checkbox beside the masked farm key: show it in clear.
farm-client-show-key = Afficher
# A field label (the name this machine goes by), and the label of the status row about this machine.
farm-client-machine = Cette machine
farm-client-machine-hover = Le nom que le contrôleur affiche pour cette machine.
farm-client-share = Lecteur partagé
# Greyed hint inside the empty field; the path is an example.
farm-client-share-hint = facultatif — p. ex. \\fileserver\share
farm-client-share-hover = Le chemin, depuis cette machine, du lecteur partagé qu’utilise le contrôleur (mode partage) : les images y sont écrites au lieu d’être envoyées par la connexion. Laissez vide pour envoyer chaque image.
farm-client-gpu = Carte graphique
# The label of the row with the idle option.
farm-client-when = Quand
# A checkbox followed by a number of minutes ("Only when idle for [5] min").
farm-client-when-idle = Seulement si inactive depuis
farm-client-when-idle-hover = Ne prendre du travail que lorsque personne n’a touché le clavier ou la souris de cette machine depuis ce délai, ou tant que son écran est verrouillé. Dès que quelqu’un se met à l’utiliser, l’image en cours s’arrête aussitôt et retourne à la ferme.
# A collapsible section: the largest job this machine accepts.
farm-client-limits = Limites
farm-client-limits-note = Une tâche qui demande plus que ces valeurs est refusée, jamais réduite.
farm-client-limit-size = Image la plus grande
farm-client-limit-ss = Suréchantillonnage maximal
farm-client-limit-iter = Itérations maximales

## Render client: the graphics-card choice

farm-client-gpu-system = Le choix du système
farm-client-gpu-every = Toutes les cartes graphiques
# $count: this machine's graphics cards.
farm-client-gpu-every-n = Toutes les cartes graphiques ({ $count })
# $count: this machine's graphics cards.
farm-client-gpu-every-each = Toutes les cartes graphiques ({ $count }) — une session chacune
# A card the window cannot name. $number: its number (or numbers) as --list-adapters gives it.
farm-client-gpu-adapter = Adaptateur { $number }
farm-client-gpu-finding = Recherche des cartes graphiques de cette machine…
# "NAME · GPU 1" is how such a machine is named in the controller's list: keep it as it is.
farm-client-gpu-hover = La carte graphique qui calcule les images de la ferme. Avec toutes les cartes, chacune rejoint la ferme comme une machine à part entière — « NAME · GPU 1 », « NAME · GPU 2 » — et « Mettre en pause » et « Déconnecter » agissent sur toutes.

## Render client: looking for controllers on the network (the Find button)

farm-client-finding = Recherche de contrôleurs sur ce réseau…
# $port: the UDP port number.
farm-client-found-none = Aucun contrôleur n’a répondu. Un contrôleur répond tant que sa ferme est à l’écoute ; son pare-feu doit laisser entrer UDP { $port }.
farm-client-found-pick = Sur ce réseau — cliquez sur l’un d’eux pour l’utiliser :
# $identity: the controller's identity fingerprint.
farm-client-found-hover = Identité { $identity } — utiliser ce contrôleur (sa clé de ferme reste nécessaire)
# After a warning sign. $version and $git: this app's version and build.
farm-client-found-other-build = Une version différente de celle-ci ({ $version } { $git }) : elle refuserait cette machine.

## Render client: what is wrong with what was typed, or with starting the client

farm-client-bad-address = Saisissez l’adresse du contrôleur sous la forme HÔTE:PORT, telle que sa fenêtre l’affiche (par exemple 192.168.1.20:46733).
# $why: what is wrong with the key (in English).
farm-client-bad-key = La clé de ferme : { $why }
# $why: what is wrong with the name (in English).
farm-client-bad-name = Le nom : { $why }
farm-client-no-config = Il n’y a pas de dossier de configuration où conserver la clé de ferme.
# $why: the system's reason.
farm-client-start-failed = Impossible de démarrer le client de rendu : { $why }

## Render client: the state, in the status heading

farm-client-phase-connecting = Connexion…
farm-client-phase-checking = Autovérification en cours…
farm-client-phase-idle = Connecté — en attente de travail
farm-client-phase-rendering = Rendu en cours
farm-client-phase-in-use = En attente — quelqu’un utilise cette machine
farm-client-phase-paused = En pause
farm-client-phase-retrying = Contrôleur injoignable — nouvelle tentative
# $seconds: the wait before the next try.
farm-client-phase-retrying-in = Contrôleur injoignable — nouvelle tentative dans { $seconds } s
farm-client-phase-disconnected = Déconnecté
farm-client-phase-ended = Terminé

## Render client: the line under the status heading

# $address: the controller's address (HOST:PORT).
farm-client-detail-connecting = Connexion à { $address }…
# $controller: the controller's name.
farm-client-detail-checking = Connecté à « { $controller } » — autovérification en cours
# $controller: the controller's name.
farm-client-detail-approval = Connecté à « { $controller } » — en attente de l’approbation de cette machine par son utilisateur
# $controller: the controller's name.
farm-client-detail-idle = Connecté à « { $controller } » — inactif
# $why: the check that failed and why (in English).
farm-client-detail-check-failed = L’autovérification a échoué — { $why }
# $why: the controller's reason (in English).
farm-client-detail-refused = Refusé par le contrôleur : { $why }
# $why: the controller's reason (in English).
farm-client-detail-closed = Le contrôleur a fermé la connexion : { $why }
# $why: the controller's reason (in English).
farm-client-detail-ended = Le contrôleur a mis fin à la connexion : { $why }
# $job: the job's name.
farm-client-detail-bundle = Tâche « { $job } » — réception de son paquet
# $job: the job's name.
farm-client-detail-job-waiting = Tâche « { $job } » — en attente d’images à calculer
# $why: the limit the job is over (in English).
farm-client-detail-job-refused = Tâche refusée : { $why }
farm-client-detail-job-over = La tâche est terminée
# $first and $last: frame numbers.
farm-client-detail-rendering = Rendu des images { $first }–{ $last }
# $time: how long nobody must use the machine ("90 s", "5 min").
farm-client-detail-in-use = En attente — quelqu’un utilise cette machine ; le travail reprendra quand personne ne l’aura utilisée depuis { $time }
farm-client-detail-paused = En pause — le contrôleur a été prévenu
farm-client-detail-pausing = Mise en pause — fin de l’image en cours
farm-client-detail-cancelled = Image en cours annulée ; en pause
# A machine with several graphics cards. $rendering: the cards rendering now; $total: all its cards (two or more).
farm-client-rendering-on-cards = Rendu sur { $rendering } cartes graphiques sur { $total }

## Render client: the status rows

# The value of the Controller row. $name: the controller's name; $address: HOST:PORT; $identity: its identity fingerprint.
farm-client-row-controller-value = « { $name } » · { $address } · identité { $identity }
# The value of the This machine row. $name: this machine's name; $identity: its identity fingerprint.
farm-client-row-machine-value = « { $name } » · identité { $identity }
farm-client-row-gpu = GPU
# One graphics card's row. $slot: the card's number.
farm-client-row-gpu-n = GPU { $slot }
farm-client-row-job = Tâche
# The frames handed to this machine.
farm-client-row-run = Lot
farm-client-row-sent = Envoyées
# The Run row, in parts joined with " · ": the run, the frame in progress, its time. $first and $last: frame numbers.
farm-client-run-frames = images { $first }–{ $last }
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-client-run-frame = image { $frame } ({ $index } sur { $count })
# $time: the time spent on it so far ("2.1 s").
farm-client-run-frame-time = { $time } sur cette image
# One graphics card's run. $frame: the frame in progress; $first and $last: the run's frames.
farm-client-session-frame = image { $frame } sur { $first }–{ $last }
farm-client-session-no-card = carte graphique pas encore connue
# The Sent row. $count: frames sent; $time: the mean time per frame ("2.1 s").
farm-client-sent =
    { $count ->
        [one] { $count } image · { $time } par image
       *[other] { $count } images · { $time } par image
    }
# The Sent row of a machine with several cards. $count: frames sent; $split: each card's share ("812 + 422"); $time: the mean time per frame.
farm-client-sent-split =
    { $count ->
        [one] { $count } image ({ $split }) · { $time } par image
       *[other] { $count } images ({ $split }) · { $time } par image
    }
# After a warning sign. $note: how the GPUs differ, and both GPUs (in English).
farm-client-gpu-note = Cette machine a { $note }. Ses images différeront légèrement de celles du contrôleur, ce qui peut se voir dans un plan fixe.
# A collapsible section: the self-check's results.
farm-client-self-check = Autovérification
farm-client-thumb-hover = La dernière image envoyée par cette machine
farm-client-resume = Reprendre
farm-client-resume-hover = Reprendre du travail
farm-client-pause = Mettre en pause
farm-client-pause-hover = Terminer l’image en cours, puis ne plus prendre de travail. Le contrôleur confie les images de cette machine aux autres.
farm-client-cancel-frame = Annuler l’image
farm-client-cancel-frame-hover = Arrêter immédiatement l’image en cours et se mettre en pause ; le contrôleur la fait calculer ailleurs.
farm-client-live-paused = Vue en direct en pause pendant que cette machine calcule des images pour la ferme.
farm-client-close = Fermer
farm-client-close-active-hover = Fermer la fenêtre ; cette machine reste dans la ferme (rouvrez la fenêtre depuis le menu Fichier)
farm-client-close-hover = Fermer la fenêtre
farm-client-disconnect = Déconnecter
farm-client-disconnect-hover = Quitter la ferme : l’image en cours est rendue au contrôleur
farm-client-connect = Connecter
farm-client-connect-hover = Se connecter au contrôleur et calculer des images pour lui

## The Render on farm window (Tools ▸ Render on farm…): the controller of a render farm

farm-ctl-title = Rendu sur la ferme
farm-ctl-out = Dossier de sortie
farm-ctl-browse = Parcourir…
farm-ctl-out-note = Les images, et l’état de la tâche dans farm/ : relancez la même tâche pour la reprendre.
farm-ctl-size = Taille
farm-ctl-size-custom = Personnalisée…
# The size list's choice when the size is typed in. $width and $height: pixels.
farm-ctl-size-custom-value = Personnalisée — { $width }×{ $height }
farm-ctl-fps = Images / seconde
farm-ctl-ss = Suréchantillonnage
farm-ctl-port = Écouter sur le port
farm-ctl-port-hover = Les clients se connectent à ce port. Autorisez-le dans le pare-feu pour votre réseau local.
farm-ctl-share = Lecteur partagé
# Greyed hint inside the empty field.
farm-ctl-share-hint = facultatif — son chemin depuis cette machine
farm-ctl-share-hover = Mode partage : les clients qui ont le même lecteur y écrivent leurs images au lieu de les envoyer par la connexion, et cette machine les y vérifie toutes. Les clients qui ne l’ont pas envoient les leurs.
# A label followed by a number of clients and two checkboxes.
farm-ctl-start-with = Démarrer avec
# The unit after the number in the box ("Start with [3] clients"). $count: that number.
farm-ctl-start-clients =
    { $count ->
        [one] client
       *[other] clients
    }
farm-ctl-start-clients-hover = La tâche démarre dès que ce nombre de machines se sont connectées et ont réussi leur autovérification ; d’autres peuvent la rejoindre plus tard.
farm-ctl-local = Calculer aussi sur cette machine
farm-ctl-local-hover = Cette machine compte comme un client.
farm-ctl-sharing = Partager les références
farm-ctl-sharing-hover = L’orbite de référence d’une image profonde peut prendre de quelques minutes à plusieurs heures à construire. Partagée, elle est construite une seule fois et transmise aux autres machines. Une référence partagée est une référence différente, tout aussi valable : à profondeur extrême, une poignée de pixels peuvent différer d’une image qui a construit la sienne.
farm-ctl-key = Clé de ferme
# A checkbox beside the shortened farm key: show all of it.
farm-ctl-show-key = Afficher
farm-ctl-copy-key = Copier la clé
farm-ctl-copy-key-hover = Collez-la dans la fenêtre Client de rendu de chaque client
# In place of the farm key, when there is none.
farm-ctl-no-key = aucune pour l’instant
farm-ctl-make-key = Créer une clé
farm-ctl-make-key-hover = Les clients en ont besoin pour rejoindre la ferme ; elle est aussi créée quand vous appuyez sur « Lancer le rendu »
# What to do on the other machines. $address: this controller's HOST:PORT.
farm-ctl-steps = Sur chaque client : Fichier ▸ Client de rendu…, contrôleur { $address }, et la clé de ferme.
# The same when this machine's address is not known. $port: the port number.
farm-ctl-steps-port = Sur chaque client : Fichier ▸ Client de rendu…, l’adresse de cette machine et le port { $port }, et la clé de ferme.
# The size of the job. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
farm-ctl-summary =
    { $count ->
        [one] { $count } image · { $seconds } s de parcours · { $width }×{ $height } ss{ $ss }
       *[other] { $count } images · { $seconds } s de parcours · { $width }×{ $height } ss{ $ss }
    }
farm-ctl-strip-hover = Chaque image de la tâche, de gauche à droite : terminée, en cours de rendu, encore à faire, abandonnée.
farm-ctl-no-tour = Aucun parcours n’est chargé.
# $folder: the folder; $why: the system's reason.
farm-ctl-out-error = Le dossier de sortie { $folder } : { $why }
# $why: the system's reason.
farm-ctl-start-failed = Impossible de démarrer la ferme : { $why }
# $code: the controller process's exit code, as the code gives it.
farm-ctl-ended-exit = Le contrôleur s’est arrêté (code de sortie { $code }).

## Render on farm: the state, in the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-phase-waiting = En attente des clients de rendu ({ $connected } sur { $needed } connectés)
farm-ctl-phase-preparing = Préparation de la tâche
farm-ctl-phase-rendering = Rendu en cours
farm-ctl-phase-paused = En pause
farm-ctl-phase-complete = Achevé
farm-ctl-phase-stopped = Arrêté
farm-ctl-phase-ended = Terminé

## Render on farm: the words after the status heading

# $connected: clients connected so far; $needed: clients the job waits for.
farm-ctl-detail-waiting = En attente des clients de rendu : { $connected } sur { $needed } connectés
farm-ctl-detail-preparing = Mesure des ancres de normalisation sur cette machine, une fois pour tous les clients
farm-ctl-detail-no-space = En attente d’espace dans le dossier de sortie
farm-ctl-detail-paused = En pause : aucun nouveau travail n’est distribué
farm-ctl-detail-not-rendered = Terminé avec des images non rendues — relancez sur le même dossier pour reprendre
farm-ctl-detail-stopped = Arrêté — relancez sur le même dossier pour reprendre
# $code: the controller process's exit code.
farm-ctl-detail-ended = Terminé (code de sortie { $code })
farm-ctl-detail-finishing = Finalisation

## Render on farm: progress

# On the progress bar. $done: frames done; $total: frames in the job.
farm-ctl-progress =
    { $total ->
        [one] { $done } / { $total } image
       *[other] { $done } / { $total } images
    }
# Added to the progress bar. $rate: frames per second, with decimals.
farm-ctl-progress-rate = { $rate } images/s
# Added to the progress bar. $time: the time still to go ("38m 10s").
farm-ctl-progress-eta = encore environ { $time }
# Facts under the progress bar, joined with " · ". $received and $sent: kilobytes per second.
farm-ctl-io = entrée { $received } KB/s · sortie { $sent } KB/s
# $space: free space on the output drive ("412 GB").
farm-ctl-free = { $space } libres
# $time: the time since the job started ("38m 10s").
farm-ctl-elapsed = écoulé : { $time }
# $count: frames given up on.
farm-ctl-failed = non rendues : { $count }
# Reference orbits. $built: built for this job; $cached: found in the cache; $reused: reused between frames; $shared: passed between machines.
farm-ctl-refs = références : { $built } construites, { $cached } depuis le cache, { $reused } réutilisées · { $shared } partagées
# After a warning sign.
farm-ctl-storage-low = Le dossier de sortie est presque plein : aucun nouveau travail n’est distribué tant qu’il n’y a pas de place.
# After a warning sign.
farm-ctl-gpus-differ = Les GPU diffèrent dans cette ferme. Leurs images diffèrent légèrement, ce qui peut se voir dans un plan fixe.
# After a warning sign. $count: the GPU classes in the farm.
farm-ctl-gpus-differ-classes = Les GPU diffèrent dans cette ferme ({ $count } classes de GPU : GPU, pilotes ou rendus de sondage différents). Leurs images diffèrent légèrement, ce qui peut se voir dans un plan fixe.
# $gpu: this machine's GPU, API and driver.
farm-ctl-this-gpu = Cette machine : { $gpu }
# A machine whose GPU differs. $name: the machine; $note: how its GPU differs (in English); $adapter: its GPU.
farm-ctl-gpu-row = { $name } : { $note } — { $adapter }
# The same with its driver. $driver: the driver's name and version.
farm-ctl-gpu-row-driver = { $name } : { $note } — { $adapter }, pilote { $driver }

## Render on farm: the table of machines

# Column headers (short: the table is narrow).
farm-ctl-col-machine = Machine
farm-ctl-col-state = État
farm-ctl-col-gpu = GPU
farm-ctl-col-frames = Images
farm-ctl-col-per-frame = Par image
# How the machine delivers its frames (a speed, or "shared drive").
farm-ctl-col-link = Liaison
# When the controller last heard from the machine.
farm-ctl-col-heard = Contact
# In the machine's tooltip. $count: its frames that failed verification.
farm-ctl-strikes =
    { $count ->
        [one] { $count } image a échoué à la vérification
       *[other] { $count } images ont échoué à la vérification
    }
# In the Link column: it delivers its frames through the shared drive.
farm-ctl-link-share = lecteur partagé
# In the Heard column. $seconds: how long ago.
farm-ctl-heard-ago = il y a { $seconds } s
farm-ctl-readmit = Réadmettre
farm-ctl-readmit-hover = Laisser cette machine rejoindre de nouveau la ferme quand elle se reconnecte
farm-ctl-remove = Retirer
farm-ctl-remove-hover = Retirer cette machine de la tâche ; ses images vont aux autres
# The GPU column's tooltip.
farm-ctl-probe-same = Son rendu de sondage est identique à celui de cette machine.
# $count: the pixels that differ.
farm-ctl-probe-differs =
    { $count ->
        [one] Son rendu de sondage diffère de celui de cette machine sur { $count } pixel : ses images différeront légèrement de celles de cette machine.
       *[other] Son rendu de sondage diffère de celui de cette machine sur { $count } pixels : ses images différeront légèrement de celles de cette machine.
    }
farm-ctl-probe-unknown = Son rendu de sondage n’a pas été comparé.
# $driver: the driver's name and version.
farm-ctl-gpu-driver = pilote { $driver }
# After a warning sign. $note: how its GPU differs (in English: "a different GPU").
farm-ctl-gpu-note = { $note } (par rapport à celui de cette machine)

## Render on farm: a machine's state, in the State column

# $count: its frames that failed verification.
farm-ctl-state-removed-bad =
    { $count ->
        [one] retirée : { $count } image défectueuse
       *[other] retirée : { $count } images défectueuses
    }
farm-ctl-state-removed = retirée par vous
farm-ctl-state-removing = retrait en cours — collecte de ses diagnostics
farm-ctl-state-gone = partie
farm-ctl-state-leaving = en train de partir
farm-ctl-state-checking = vérification (autovérification)
farm-ctl-state-paused = mise en pause par son utilisateur
farm-ctl-state-parked = mise à l’écart : deux plantages — revient quand elle se reconnecte
farm-ctl-state-refused = a refusé cette tâche (ses propres limites)
# $frame: the frame number; $index: its place in the run; $count: the frames in the run.
farm-ctl-state-rendering = rendu de { $frame } ({ $index }/{ $count })
farm-ctl-state-starting = démarrage d’un lot
farm-ctl-state-idle = inactive
farm-ctl-state-admitted = admise — en attente de la tâche

## Render on farm: the buttons

# A collapsible section: the controller's messages.
farm-ctl-events = Événements
farm-ctl-resume = Reprendre
farm-ctl-pause = Mettre en pause
farm-ctl-pause-hover = Ne plus distribuer de nouveau travail ; les images en cours se terminent
farm-ctl-live-paused = Vue en direct en pause pendant que cette machine calcule pour la ferme.
farm-ctl-close = Fermer
farm-ctl-close-active-hover = Fermer la fenêtre ; la tâche continue (rouvrez la fenêtre depuis le menu Outils)
farm-ctl-close-hover = Fermer la fenêtre
farm-ctl-copy-command = Copier la commande
farm-ctl-copy-command-hover = Copier la ligne de commande équivalente
farm-ctl-stop = Arrêter
# The Stop button once a stop has been asked for and the job has not ended after 10 seconds.
farm-ctl-stop-now = Arrêter maintenant
farm-ctl-stop-hover = Terminer la tâche avec les images qu’elle a ; relancer la même tâche la reprend
farm-ctl-render = Lancer le rendu
farm-ctl-render-hover = Commencer à écouter les clients et rendre le parcours sur eux

## The Render tour window (Tools ▸ Render tour…)

tour-render-title = Rendu d’un parcours
tour-render-out = Dossier de sortie
tour-render-browse = Parcourir…
tour-render-out-hover = Chemin complet où les images sont écrites.
tour-render-prefix = Préfixe des images
# <prefix> stands for the prefix typed in the field.
tour-render-prefix-hover = Les images sont écrites sous la forme <préfixe>_00000.png
tour-render-size = Taille
tour-render-size-custom = Personnalisée…
# The size list's choice when the size is typed in. $width and $height: pixels.
tour-render-size-custom-value = Personnalisée — { $width }×{ $height }
tour-render-fps = Images / seconde
tour-render-fps-hover = Moins de 1 fps est légitime : échantillonner une longue plongée à 0,25 fps est la façon d’inspecter les pauses de régression en profondeur sans rendre des milliers d’images.
tour-render-ss = Suréchantillonnage
tour-render-chapter = Chapitre
tour-render-whole = Parcours entier
tour-render-chapter-hover = Rendre un seul chapitre conserve la numérotation GLOBALE des images : ses images s’insèrent donc directement dans une séquence complète.
tour-render-mp4 = Assembler un mp4
tour-render-mp4-hover = Lance ffmpeg ensuite (il doit être dans le PATH). Les images PNG sont conservées.
tour-render-overwrite = Écraser les images existantes
tour-render-overwrite-hover = Désactivé : le rendu s’arrête pour demander, ce qu’il ne peut pas faire depuis ici.
tour-render-resume = Reprendre un rendu interrompu
tour-render-resume-hover = Conserver les images déjà présentes dans le dossier de sortie et ne rendre que celles qui manquent. L’image la plus récente est vérifiée d’abord et supprimée si elle est incomplète (l’image sur laquelle un rendu s’interrompt est celle qu’il écrivait), en remontant jusqu’à en trouver une bonne ; un dossier contenant des images d’une autre taille est refusé plutôt que mélangé à cette séquence.
# $count: this machine's graphics cards.
tour-render-all-gpus = Utiliser toutes les cartes graphiques ({ $count })
tour-render-all-gpus-hover = Rendre sur toutes les cartes graphiques de cette machine à la fois, via une ferme de rendu sur cet ordinateur (--gpus all). Les images, leurs noms, la reprise et le mp4 sont les mêmes qu’avec une seule carte ; tout le parcours est rendu, dans l’ordre propre à la ferme. Des modèles de cartes différents dessinent quelques pixels légèrement différemment : un plan fixe et un fondu enchaîné restent sur une seule carte.
tour-render-order = Ordre de rendu
tour-render-sequential = Séquentiel
tour-render-sequential-hover = Image 0, 1, 2, … dans l’ordre (par défaut).
tour-render-progressive = Progressif
tour-render-progressive-hover = Prévisualiser d’abord tout le parcours, puis affiner jusqu’au plein détail : les images clés sont rendues en premier, puis les plus grands écarts temporels sont coupés en deux jusqu’à ce que chaque image existe. Les images arrivent à leurs bons indices dans les deux cas : la reprise et le mp4 fonctionnent donc sans changement — vous pouvez vous arrêter tôt, jeter un œil au folioscope grossier, puis relancer avec la reprise, dans l’un ou l’autre ordre.
# The cost of the render. $count: frames; $seconds: the tour's length in seconds; $width × $height: the frame size in pixels; $ss: the supersampling factor.
tour-render-summary =
    { $count ->
        [one] { $count } image · { $seconds } s de parcours · { $width }×{ $height } ss{ $ss }
       *[other] { $count } images · { $seconds } s de parcours · { $width }×{ $height } ss{ $ss }
    }
# The disk space the frames will take (an estimate). $size: a size ("5.3 GB").
tour-render-disk = ≈{ $size } d’images
# $size: the frames' estimated size; $free: the free space on that drive ("6.0 GB").
tour-render-disk-free = ≈{ $size } d’images · { $free } libres
# After a warning sign. $disk: the line above ("≈51.0 GB of frames · 6.0 GB free").
tour-render-disk-short = { $disk } — pas assez de place pour toute la séquence
tour-render-separate = Rend le script DEPUIS LE DISQUE dans un processus séparé : les modifications non enregistrées ne sont donc pas incluses — et une perte du périphérique GPU là-bas ne peut pas emporter cette session avec elle.
# On the progress bar. $done: frames done; $total: frames to render.
tour-render-progress =
    { $total ->
        [one] { $done } / { $total } image
       *[other] { $done } / { $total } images
    }
# Before the render's first progress line.
tour-render-starting = démarrage…
tour-render-live-paused = Vue en direct en pause — le rendu a le GPU pour lui seul : il se termine donc plus tôt et mesure précisément le coût de ses propres images.
tour-render-cancel = Annuler
tour-render-cancel-hover = Fermer sans rendre
tour-render-copy-command = Copier la commande
tour-render-copy-command-hover = Copier la ligne de commande équivalente dans le presse-papiers
tour-render-stop = Arrêter le rendu
tour-render-render = Lancer le rendu
tour-render-render-hover = Commencer le rendu de la séquence d’images
# $folder: the output folder.
tour-render-finished = Rendu terminé → { $folder }
# $why: the render's own reason (in English).
tour-render-failed = Échec du rendu : { $why }
# $last: the render's last progress line (in English), or nothing.
tour-render-failed-or-stopped = Le rendu a échoué ou a été arrêté. { $last }
# $why: the system's reason.
tour-render-no-exe = Impossible de trouver l’exécutable : { $why }
# $why: the system's reason.
tour-render-start-failed = Impossible de démarrer le rendu : { $why }

## The Custom formula dialog (Fractal ▸ Custom formula…)
## The formula language itself stays as it is in every language: function names (sin, cosh, conj…),
## operators, variables (z, c, p1), and the formulas. Keep them as written inside these messages.

# The window's title.
formula-dialog-title = Formule personnalisée
# A small line at the top of the window, above the formula.
formula-dialog-intro = Le pas, en expressions de style Fractint
# The two ways to show and edit the formula (top right of the window).
formula-dialog-textbook = Mise en forme
formula-dialog-textbook-hover = Afficher la formule composée, comme dans un manuel, et la modifier sous cette forme
formula-dialog-text = Texte
formula-dialog-text-hover = Modifier la formule sous forme de texte
# The tooltip of the "LaTeX" button (the button's own text stays LaTeX).
formula-dialog-latex-hover = Copier la formule en LaTeX, telle qu’elle est composée ici
# The drop-down list of example formulas (the collection).
formula-dialog-examples = Exemples
# Help under the formula in Textbook mode. The characters / ^ ( are what the user types: keep them.
formula-dialog-help-textbook = Tapez comme dans le texte : / fait une fraction du terme qui le précède, ^ un exposant, ( des parenthèses. Les flèches parcourent la formule ; Tab va à la case vide suivante. Une ligne en rouge est illisible : cliquez dessus pour la modifier comme texte.
# Help under the formula in Text mode. "; starts a comment": the ; is what the user types.
formula-dialog-help-text = Tapez, ou utilisez le pavé — il contient tous les noms que connaît le langage de formules. Les noms se complètent pendant la saisie (Tab). Les instructions sont séparées par un retour à la ligne ou une virgule ; « ; » commence un commentaire.
# The syntax check, when the formula reads.
formula-dialog-reads = Lecture correcte.
# The syntax check in Textbook mode, when the formula has an empty box to fill in.
formula-dialog-empty-box = Remplissez la case vide.
# The syntax check in Textbook mode. $line: the line number; $message: what is wrong (from the formula parser, in English).
formula-dialog-error-line = Ligne { $line } : { $message }
# One function written with another notation's name, and the language's name for it: "ln to log".
# $from: the name as written (ln); $to: the formula language's name (log).
formula-respell-pair = { $from } en { $to }
# A button that rewrites them. $changes: one or more formula-respell-pair, joined with ", ".
formula-respell = Changer { $changes }
formula-respell-hover = Écrire ces fonctions comme les nomme le langage de formules
# Shown when Apply finds a parameter field that is not a number. $name: the parameter (p1 … p5).
formula-param-not-numbers = { $name } n’est pas une paire de nombres
# The Apply button (also in the Formula library window).
formula-apply = Appliquer
formula-dialog-apply-hover = Compiler la formule et l’afficher
# Shown beside Apply while the formula is being compiled.
formula-dialog-compiling = Compilation de la formule pour le GPU…
# The greyed hint in an empty name box (also in the Formula library window).
formula-name-hint = nom (facultatif)
# The save button: "Update" when the library already has a formula of that name.
formula-dialog-update-in-library = Mettre à jour dans la bibliothèque
formula-dialog-save-to-library = Enregistrer dans la bibliothèque
formula-dialog-save-hover = Conserver cette formule et ses paramètres dans la bibliothèque de formules
formula-dialog-save-disabled-hover = Seule une formule lue correctement peut être enregistrée
# Opens the Formula library window.
formula-dialog-library = Bibliothèque…
formula-dialog-library-hover = Les formules enregistrées : les appliquer, les modifier, les importer et les exporter

## The Custom formula dialog's keypad
## The keys' captions are formula tokens and stay as they are; these are their tooltips.

# The keypad's third tab (the other two are "123" and "f(z)"). A short word.
formula-keypad-tab-names = noms
# The tabs' tooltips.
formula-keypad-tab-basic-hint = Nombres, opérateurs et noms les plus utilisés
formula-keypad-tab-functions-hint = Fonctions
formula-keypad-tab-names-hint = Toutes les variables, tous les paramètres et constantes, et la syntaxe des instructions
# The key z.
formula-key-z = L’itéré
# The key c.
formula-key-c = Le pixel (la constante de Julia en mode Julia)
# The key p1 on the first tab.
formula-key-p1-below = Paramètre 1 — réglé dans les champs ci-dessous
# The keys p1 … p5. $n: the parameter's number.
formula-key-param = Paramètre { $n }
formula-key-multiply = Multiplier
formula-key-divide = Diviser
# The key that types ^2.
formula-key-square-after = Carré (après une valeur : z^2)
# The key that types ^.
formula-key-power = Puissance : un exposant entier, réel ou complexe
formula-key-square-root = Racine carrée
# The key that types |…|: in this language it is the squared modulus, as in Fractint.
formula-key-squared-modulus = Module AU CARRÉ |z|² — comme dans Fractint
formula-key-add = Additionner
formula-key-subtract = Soustraire (ou prendre l’opposé)
# The key that types =.
formula-key-assign = Affecter : z = …, ou une variable temporaire t = …
# The key that starts a new line (first tab).
formula-key-next-statement-comma = Instruction suivante (une virgule fonctionne aussi)
# The key "del".
formula-key-backspace = Supprimer le caractère avant le curseur (ou la sélection)
# The key e.
formula-key-euler = Le nombre d’Euler
# The key that types (0.5, 0.5).
formula-key-complex-constant = Une constante complexe : (réel, imaginaire)
# The key that types a comma.
formula-key-comma = Sépare les instructions, ou les parties d’une constante complexe
formula-key-left = Curseur à gauche
formula-key-right = Curseur à droite
# The function keys (the f(z) tab), one per function.
formula-key-sin = Sinus
formula-key-cos = Cosinus
formula-key-tan = Tangente
formula-key-cotan = Cotangente
formula-key-exp = Exponentielle e^z
formula-key-sinh = Sinus hyperbolique
formula-key-cosh = Cosinus hyperbolique
formula-key-tanh = Tangente hyperbolique
formula-key-cotanh = Cotangente hyperbolique
formula-key-log = Logarithme naturel, ln (branche principale)
formula-key-sqr = Carré : sqr(z) = z²
formula-key-sqrt = Racine carrée (principale)
formula-key-recip = Inverse 1/z
formula-key-conj = Conjugué complexe
formula-key-flip = Échanger les parties réelle et imaginaire
formula-key-abs = Valeur absolue des DEUX parties : |x| + i|y|
formula-key-cabs = Module |z| (un nombre réel)
formula-key-real = Partie réelle
formula-key-imag = Partie imaginaire, en tant que nombre réel
# "fn" is Fractint's name for a function slot: keep it.
formula-key-ident = Identité (l’emplacement fn de Fractint)
# The key "pixel".
formula-key-pixel = Identique à c (le nom de Fractint)
# The key "z =".
formula-key-set-z = Commencer une instruction qui affecte z
# The key "t =".
formula-key-temporary = Une variable temporaire : n’importe quel nouveau nom convient (t, w, a2, …)
# The key "; …".
formula-key-comment = Un commentaire, jusqu’à la fin de la ligne
# The key that starts a new line (names tab).
formula-key-next-statement = Instruction suivante

## The formula's completion list (names offered as you type)

# Beside a name the formula itself assigns.
formula-completion-variable = Une variable que cette formule affecte
# Beside a function typed with another notation's name.
# $what: what the function is, in English (from the formula parser: "The natural logarithm");
# $other: the other notation's name (ln); $ours: the formula language's name (log).
formula-completion-other-spelling = { $what } — { $other } s’écrit { $ours } ici
# Under a list cut short. $count: the names not shown.
formula-completion-more = … { $count } de plus
# The last line of the list. Tab, Enter and Esc are key names.
formula-completion-keys = Tab ou Enter pour compléter, Esc pour fermer

## The Custom formula dialog's Textbook mode

# The tooltip of a line that does not read. $why: what is wrong (from the formula parser, in English, or formula-textbook-fractint-sections).
formula-textbook-unread-hover = Illisible : { $why }. Cliquez pour la modifier comme texte.
# Why a line with Fractint's sections is shown as text in Textbook mode.
formula-textbook-fractint-sections = Les sections de Fractint (le « : » d’une initialisation, les comparaisons, a = b = …) se modifient comme texte

## The Formula library window (Fractal ▸ Formula library…)

# The window's title.
formula-library-title = Bibliothèque de formules
# The window's two tabs. $count: the formulas in each.
formula-library-mine = Mes formules ({ $count })
formula-library-mine-hover = Les formules que vous avez enregistrées ou importées
formula-library-collection = Collection ({ $count })
formula-library-collection-hover = Formules fournies avec Fractadyne, chacune avec une vue de départ
# A small marker before the name of the formula the view shows now. A short word.
formula-library-showing = affichée
# The tooltip of Apply on a formula of the collection.
formula-library-apply-shipped-hover = L’afficher, à sa vue de départ
# A row's Edit button (after a pencil icon).
formula-library-edit = Modifier
formula-library-edit-hover = L’ouvrir dans la fenêtre Formule personnalisée, sans l’appliquer
# A collection row's button that copies the formula into "My formulas".
formula-library-copy-to-mine = Copier dans les miennes
formula-library-copy-to-mine-hover = L’ajouter à vos formules, pour la garder ou la modifier
formula-library-close = Fermer
# Saves the formula the view shows, under the name typed beside it.
formula-library-add-current = Ajouter la formule actuelle
formula-library-add-current-hover = Enregistrer la formule qu’affiche la vue
# "Fractal > Custom formula…" is the menu path: write it as the menus are named.
formula-library-add-current-disabled-hover = Appliquez d’abord une formule personnalisée (Fractale > Formule personnalisée…)
formula-library-import = Importer…
formula-library-import-hover = Ajouter les formules d’un fichier de formules ; rien ici n’est remplacé
formula-library-export-all = Tout exporter…
formula-library-export-all-hover = Écrire toutes les formules d’ici dans un seul fichier, pour les partager ou les garder
# The filter box: its label, and the greyed hint inside it.
formula-library-filter = Filtre
formula-library-filter-hint = filtrer par nom ou par texte
# Shown while the user has no saved formulas.
formula-library-empty = Aucune formule enregistrée pour l’instant. Enregistrez-en une depuis la fenêtre Formule personnalisée, ajoutez celle qu’affiche la vue, ou importez un fichier (le nôtre, ou un .frm de Fractint).
# The tooltip of Apply on one of the user's formulas: with a view saved, and without one.
formula-library-apply-view-hover = Afficher cette formule, à la vue enregistrée avec elle
formula-library-apply-hover = Afficher cette formule
# The tooltip of a row's save icon.
formula-library-export-one-hover = Exporter cette formule dans un fichier
# A row asks before deleting: the question, then its two buttons. "Delete" is also the delete icon's tooltip.
formula-library-delete-ask = La supprimer ?
formula-library-delete = Supprimer
formula-library-keep = Garder
# Under a list cut short. $count: the formulas not listed.
formula-library-more = … et { $count } de plus : tapez dans le filtre pour en trouver une.
formula-library-no-match = Aucune ne correspond au filtre.
# Under a saved formula this version cannot read. $why: what is wrong (from the formula parser, in English).
formula-library-unread = Illisible dans cette version : { $why }

## Formula library notices

# $why: the system's reason, in English.
formula-library-save-failed = Impossible d’enregistrer la bibliothèque de formules : { $why }
formula-library-nothing-to-save = Rien à enregistrer : la formule est vide.
# $name: the formula's name in the library.
formula-library-updated = « { $name } » mise à jour dans la bibliothèque de formules.
formula-library-saved = « { $name } » enregistrée dans la bibliothèque de formules.
# $name: the formula's name; $why: what is wrong.
formula-library-apply-failed = « { $name } » ne peut pas être appliquée : { $why }
# Copying a formula of the collection into "My formulas". $name: its name; $as_named: the name it was given (the same, or "Name (2)" when the name was taken).
formula-library-copy-exists = « { $name } » est déjà dans vos formules.
formula-library-copied = « { $name } » copiée dans vos formules sous le nom « { $as_named } ».
# $name: the formula's name.
formula-library-deleted = « { $name } » supprimée de la bibliothèque de formules.
# Why the library could not be saved (follows formula-library-save-failed).
formula-library-no-config-dir = aucun dossier de configuration
# At startup, when the library file cannot be read. $why: the reason, in English; $path: a file's full path.
formula-library-unreadable-kept = La bibliothèque de formules n’a pas pu être lue ({ $why }). Elle a été conservée sous { $path } et une nouvelle bibliothèque vide a été créée.
formula-library-unreadable-stuck = La bibliothèque de formules n’a pas pu être lue ({ $why }), ni mise de côté : copiez { $path } en lieu sûr avant d’enregistrer une formule, ce qui la remplacerait.

## Formula library: Import and Export
## A formula file's contents stay in English; this is what the windows and notices around them say.

# File-type names in the open and save dialogs (the extensions are added by the system).
formula-filter-formula-files = Fichiers de formules
formula-filter-fractadyne = Formules Fractadyne
formula-filter-fractint = Formules Fractint
formula-filter-all-files = Tous les fichiers
# Why a file cannot be imported (they follow formula-import-failed).
formula-import-too-large = bien trop volumineux pour être un fichier de formules
formula-import-not-text = ce n’est pas un fichier texte
formula-import-no-formulas = il ne contient aucune formule
# $why: the TOML reader's message, in English.
formula-file-not-formulas = ce n’est pas un fichier de formules ({ $why })
# $format: what the file says it is ("fractadyne-gradients").
formula-file-other-format = un fichier « { $format } », pas un fichier de formules
# "[[formula]]" is the file's own syntax: keep it.
formula-file-no-formulas = il ne contient aucune formule (un fichier de formules les liste sous forme de tables [[formula]])
# $file: the file's name; $why: one of the reasons above, or the system's (in English).
formula-import-failed = Impossible d’importer « { $file } » : { $why }.
# After an import that added nothing. $file: the file's name.
formula-import-nothing-new = Rien de nouveau dans « { $file } ».
# After an import. $count: the formulas added; $file: the file's name.
formula-import-added =
    { $count ->
        [one] { $count } formule importée de « { $file } ».
       *[other] { $count } formules importées de « { $file } ».
    }
# The same two, followed by what else happened. $notes: one or more of the formula-import-note-… messages below, joined with "; ".
formula-import-nothing-new-notes = Rien de nouveau dans « { $file } » : { $notes }.
formula-import-added-notes =
    { $count ->
        [one] { $count } formule importée de « { $file } » : { $notes }.
       *[other] { $count } formules importées de « { $file } » : { $notes }.
    }
# Notes (clauses, no final full stop). $count: the formulas already in the library, skipped.
formula-import-note-duplicates =
    { $count ->
        [one] { $count } formule était déjà dans la bibliothèque
       *[other] { $count } formules étaient déjà dans la bibliothèque
    }
# One formula was renamed because its name was taken. $from: its name in the file; $to: the name it was given ("Name (2)").
formula-import-note-renamed-one = « { $from } » était pris, elle s’appelle donc « { $to } »
# $count: the formulas renamed because their names were taken (2 or more).
formula-import-note-renamed = { $count } renommées car leur nom était pris
# $count: the entries skipped as empty or too long.
formula-import-note-skipped =
    { $count ->
        [one] { $count } entrée vide ou trop longue, ignorée
       *[other] { $count } entrées vides ou trop longues, ignorées
    }
# A second sentence after importing a Fractint .frm file. $count: the entries that do not read; $why: the commonest reason (from the formula parser, in English).
formula-import-frm-unread =
    { $count ->
        [one] { $count } entrée est illisible dans cette version (le plus souvent : { $why }).
       *[other] { $count } entrées sont illisibles dans cette version (le plus souvent : { $why }).
    }
# After exporting one formula. $name: the formula's name; $file: the file's name.
formula-export-one = « { $name } » exportée vers « { $file } ».
# After exporting several. $count: how many; $file: the file's name.
formula-export-many =
    { $count ->
        [one] { $count } formule exportée vers « { $file } ».
       *[other] { $count } formules exportées vers « { $file } ».
    }
# $file: the file's name; $why: the system's reason, in English.
formula-export-failed = Impossible d’écrire « { $file } » : { $why }

## The control panel (right-hand side): header and section headings

# The panel's title, beside the button that hides it.
panel-title = Contrôles
panel-hide-hover = Masquer le panneau de contrôle
# The button that brings the hidden panel back (top right of the view).
panel-show-hover = Afficher le panneau de contrôle
# Section headings.
panel-section-navigate = Navigation
panel-section-coloring = Coloration
panel-section-quality = Qualité
panel-section-effects = Effets
panel-section-overlays = Superpositions
panel-section-advanced = Avancé
# The last section: reference notes about the current fractal. $name: the fractal's name (Mandelbrot, Burning Ship…).
panel-section-about = À propos de { $name }
panel-section-performance = Performances

## The control panel: Navigate
## Labels before a control sit in a column about 104 px wide (roughly 15 characters): a longer label pushes its control to the right.

panel-zoom-speed = Vitesse de zoom
panel-zoom-speed-hover = Vitesse du zoom continu avec Space maintenu (1× ≈ 2× par 1,5 s).
panel-snap-zoom = Zoom aimanté aux structures
panel-snap-zoom-hover = Pendant un zoom avant à la molette ou avec Space, attirer le zoom vers un minibrot (ou l’axe réel) proche du curseur, pour qu’il glisse sous le curseur et que le zoom aboutisse dessus. Plus la structure est proche, plus l’attraction est forte. Mandelbrot uniquement, vue simple.
panel-click-zoom = Cliquer pour zoomer
panel-click-zoom-hover = Activé, un clic gauche dans la vue plonge du facteur ci-dessous (le clic droit recule), en recentrant sur le point cliqué. Glisser déplace toujours la vue ; Shift+glisser / glisser avec le bouton droit font toujours un zoom sur cadre. Backspace annule un clic. En vue double, cela fonctionne sur l’un ou l’autre volet, et l’épinglage du Julia c passe à Ctrl+clic tant que cette option est active. Maintenez Shift pour faire apparaître sous le curseur une loupe montrant la vue où le clic vous mènerait, et cliquez en la maintenant pour prendre ce point ; Shift+GLISSER fait toujours un zoom sur cadre.
# The click-to-zoom factor, followed by buttons 2×, 4×, 10×…
panel-click-zoom-factor = Facteur
panel-click-zoom-snap = Caler sur le centre le plus proche
panel-click-zoom-snap-hover = Après le saut, calculer le noyau de minibrot le plus proche et se caler exactement dessus, pour arriver sur la structure plutôt qu’à côté. Le zoom n’est pas modifié — c’est le clic qui l’a choisi. Mandelbrot uniquement. Le calcul s’exécute en arrière-plan : la vue saute aussitôt et se corrige quand la réponse arrive, ce qui peut prendre quelques secondes à profondeur extrême.

## The control panel: Coloring
## Labels before a control sit in a column about 104 px wide (roughly 15 characters).

panel-method = Méthode
panel-method-hover = La façon dont les données d’échappement deviennent des couleurs. Rayures / inégalité triangulaire / piège orbital / décomposition révèlent la structure des orbites ; distance ombre selon la proximité de la frontière.
panel-stripe-density = Densité des rayures
panel-stripe-tail = Queue seule (zoom profond)
panel-stripe-tail-hover = Faire la moyenne du terme de rayures sur les N dernières itérations de chaque orbite (une fenêtre exponentielle) au lieu de l’orbite entière. À profondeur extrême, chaque orbite suit la référence de près sauf pendant ses quelques centaines de dernières itérations : la moyenne sur l’orbite entière donne donc une seule couleur pour toute la vue ; la queue conserve le contraste. Les orbites plus courtes que la fenêtre ne sont pas affectées. Aussi --stripe-tail N pour --render.
panel-tail-length = Longueur de queue
# The unit after the tail length's number: short for "iterations" (the slider reads "64 it").
panel-tail-length-unit = it
panel-tail-length-hover = Longueur de la fenêtre en itérations. Les fenêtres courtes (8–32) donnent les bandes marquées classiques à toute profondeur ; les plus longues tendent de nouveau vers une couleur uniforme, car le terme de rayures d’une longue orbite chaotique se moyenne.
panel-trap-shape = Forme du piège
panel-palette = Palette
# The two colours of the Binary palette, each beside its colour swatch.
panel-in-set = Dans l’ensemble
panel-out-of-set = Hors ensemble
# The two colours of the Duotone palette, each beside its colour swatch.
panel-shadow = Ombre
panel-highlight = Lumière
panel-cycle = Cycle
panel-offset = Décalage
panel-normalize-hover = À profondeur extrême, les nombres d’échappement couvrent des centaines de milliers d’itérations et un Cycle fixe fait boucler la palette des milliers de fois entre pixels voisins — un champ dense correct apparaît comme un bruit moucheté. Ceci recale la palette sur la plage d’échappement mesurée de la vue (Cycle règle alors combien de balayages de la palette la couvrent) LORSQUE la vue présente réellement du crénelage — une vue lisse est laissée telle quelle, car la recaler changerait l’image pour rien. Cochez « Toujours ajuster la plage » ci-dessous pour recaler dans tous les cas. Méthode « Itération lissée » uniquement ; correspond à --normalize.
# A checkbox indented under "Normalize deep colors" (also in the Color menu).
panel-fit-range = Toujours ajuster la plage
panel-fit-range-hover =
    Recaler la palette sur la plage d’échappement mesurée de la vue même quand la vue ne présente pas de crénelage — la case du dessus ne recale que lorsqu’un champ dense apparaîtrait sinon comme des mouchetures.

    Utilisez-la quand vous voulez que la palette couvre ce qui est à l’écran plutôt que de corriger du bruit. Elle a tout de même besoin d’une mesure : elle ne fait donc rien pendant le court instant qui précède la stabilisation de la vue, et rien du tout sur une vue sans pixels échappés.
# A checkbox indented under "Always fit range" (also in the Color menu).
panel-log-scale = Échelle de couleur logarithmique
panel-log-scale-hover =
    Répartir la palette selon le LOGARITHME de la valeur d’échappement plutôt que linéairement. En profondeur, les nombres d’échappement se concentrent vers le haut : une correspondance linéaire consacre alors l’essentiel de la palette à une mince couche près de la frontière et aplatit tout le reste.

    S’applique à la correspondance normalisée — celle qu’active « Normaliser les couleurs profondes » (ou --normalize pour une exportation). Une plage très asymétrique prend de toute façon la correspondance logarithmique, que cette case soit cochée ou non.
# The same checkbox's tooltip while it is greyed out (normalization is off).
panel-log-scale-off-hover =
    Nécessite la normalisation : ceci choisit COMMENT la palette est projetée sur la plage d’échappement mesurée ; avec « Normaliser les couleurs profondes » désactivé (et sans exportation --normalize), il n’y a aucune correspondance à modifier et cela ne changerait pas un seul pixel.

    Activez « Normaliser les couleurs profondes » pour l’utiliser.
# The palette animation: its kind, then its speed.
panel-animate = Animer
panel-anim-speed = Vitesse
panel-anim-speed-hover = Vitesse du cycle : cycles de décalage des couleurs par seconde, ou (Aléatoire) changements de dégradé par seconde.
panel-shuffle-gradient = Mélanger le dégradé

## The control panel: Quality

panel-auto-iter = Adapter les itérations au zoom
# The iteration-count label while auto-scaling is on (the count is then a base the zoom raises).
panel-iterations-base = Itérations (base)
panel-iterations = Itérations
panel-iterations-hover = Nombre d’itérations de base. Avec « Adapter les itérations au zoom » activé, le nombre effectif augmente avec la profondeur de zoom. Pendant le mouvement, l’aperçu plafonne les itérations à une valeur basse (50 000) pour rester réactif, puis s’affine jusqu’au nombre complet quand la vue se stabilise — ainsi, les bords profonds paraissent lisses pendant le mouvement et se résolvent quand vous vous arrêtez.
# A warning under the iteration count. $count: the iterations the settled preview renders.
panel-res-limited = À cette profondeur, l’aperçu stabilisé rend { $count } itérations en résolution réduite pour rester réactif. Exportez pour obtenir le détail en pleine résolution.
panel-anti-alias = Anticrénelage
# Anti-aliasing off (the other choices are 2×, 3×, 4×, 8×).
panel-anti-alias-off = Désactivé
panel-anti-alias-hover = Suréchantillonnage des images fixes (appliqué quand la vue se stabilise). Une valeur plus élevée atténue la fine « poussière » extérieure au prix du temps de rendu.

## The control panel: Effects

panel-relief-lighting = Éclairage en relief 3D
panel-relief-lighting-hover = Ombrer la surface d’après la pente de l’estimation de distance — un aspect embossé et éclairé. (Familles holomorphes : Mandelbrot / Multibrot.)
panel-light-angle = Angle de la lumière
panel-relief = Relief
panel-relief-hover = Plus bas = relief plus net ; plus haut = plus doux/plus plat.
panel-rotate-light = Faire tourner la lumière
panel-rotate-light-hover = Faire tourner la direction de la lumière dans le temps (utilise le curseur Vitesse).
panel-distance-glow = Lueur de distance
panel-distance-glow-hover = Bandes de contour lumineuses de l’estimation de distance qui se densifient en filaments brillants près de la frontière. (Familles holomorphes.)
panel-glow = Lueur
panel-band-width = Largeur de bande
panel-band-width-hover = Espacement des contours de distance (octaves par bande).
panel-animate-glow = Animer la lueur
panel-animate-glow-hover = Faire s’écouler les bandes de lueur dans le temps (utilise le curseur Vitesse).

## The control panel: Overlays

panel-orbits-hover = Dessiner le chemin d’itération du point sous le curseur.
panel-orbit-anim = Animer (point qui court)
panel-orbit-anim-hover = Envoyer un point aux couleurs changeantes courir le long de l’orbite.
panel-orbit-speed = Vitesse de l’orbite
panel-minimap-hover = Un petit aperçu de la vue d’accueil avec un repère « vous êtes ici » et la profondeur de zoom. Cliquez dessus pour sauter vers une région.

## The control panel: Advanced
## Labels before a control sit in a column about 104 px wide; most of these are already wider in English.

# Small headings of the section's two groups.
panel-accelerators = Accélérateurs
panel-performance-tuning = Réglage des performances
panel-bla = Accélération BLA (zoom profond)
panel-bla-hover = Approximation bilinéaire : sauter des itérations tout au long de l’orbite à profondeur extrême (Mandelbrot floatexp, ≥1e28×) — rendu GPU ~5× plus rapide, résultat identique (vérifié par l’autotest). Activée par défaut ; désactivez-la pour comparer ou si vous rencontrez un artefact.
panel-series-approx = Approximation par séries
panel-series-approx-hover = Amorcer la perturbation à partir d’un polynôme pour sauter les premières itérations, là où BLA n’est pas active (profondeurs df32, Multibrot, BLA désactivée). Résultat identique ; désactivez-la pour comparer.
panel-glitch-correct = Correction des glitches (exportation)
panel-glitch-correct-hover = Correction des glitches multi-références pour les images exportées : détecte les glitches de perturbation et rend de nouveau ces pixels avec des références supplémentaires jusqu’à ce qu’ils soient propres. Activée par défaut. Exportations jusqu’à ~32 MP (coloration non auxiliaire) ; les images plus grandes et la vue en direct utilisent le chemin simple. Jamais utilisée pour les vues Mandelbrot ou Multibrot (hors mode Julia) : vérifiée par rapport à un calcul exact, elle n’y corrigeait rien et multipliait jusqu’à 5 fois la durée d’exportation.
panel-dive-limit = Limite de plongée (zoom auto)
panel-dive-limit-hover = Profondeur où le zoom automatique (touche A) s’arrête. Jusqu’à ~1e271×, il glisse en douceur ; plus profond, il passe à une plongée saccadée par paliers pour atteindre rapidement la profondeur extrême.
# Followed by two choices, Speed and Quality.
panel-autopilot-priority = Priorité du zoom automatique
panel-priority-speed-hover = Zoomer à la vitesse réglée. L’écran se rafraîchit dès qu’une image avec du détail est prête, à la résolution qui permet de maintenir les rafraîchissements (sous le plancher « Résolution min. en mouvement » si le zoom va plus vite que le rendu) ; entre les rafraîchissements, la dernière image suit le zoom et s’adoucit.
panel-priority-quality-hover = Chaque rafraîchissement est une image entièrement résolue en pleine résolution, et le zoom ralentit autant que nécessaire pour que l’image à l’écran ne soit jamais très agrandie avant l’arrivée de la suivante. Une plongée plus lente, nette à chaque instant.
# Followed by two choices, Detail and Misiurewicz point.
panel-autopilot-target = Cible du zoom automatique
panel-target-detail-hover = Plonger vers la structure la plus riche du bord de la fractale dans la vue, en choisissant de nouveau la cible au fil de la plongée.
panel-target-misiurewicz-hover = Trouver le point de Misiurewicz le plus proche (un centre de spirale dont la structure se répète à l’infini) et plonger exactement vers lui. La plongée reste d’abord immobile pendant que le point est calculé jusqu’à la profondeur de la limite de plongée — un instant aux profondeurs ordinaires, plus longtemps au-delà de 1e1000×. Ensemble de Mandelbrot uniquement.
panel-work-budget = Budget du rendu en direct
panel-work-budget-hover = Détail ou vitesse pour la vue en direct en zoom profond. Plus haut rend à une résolution plus complète (plus net — moins l’aspect « doux » agrandi) mais réduit la fréquence d’images, et des valeurs très élevées risquent un bref blocage du GPU sur les images les plus lourdes. Les exportations sont toujours en pleine résolution, quel que soit ce réglage.
panel-min-motion-res = Résolution min. en mouvement
panel-min-motion-res-hover = La résolution la plus basse à laquelle la vue en direct peut descendre pendant un zoom ou un déplacement continu en zoom profond. Plus haut empêche une plongée rapide de devenir pixelisée (plus net), au prix de la fréquence d’images — à 100 %, les rafraîchissements de détail en plongée profonde doivent être rendus à plein coût, ce qui peut rendre le zoom profond visiblement saccadé. Si une plongée profonde semble hachée, baissez cette valeur. 30 % par défaut ; n’affecte ni l’image stabilisée ni les exportations.
panel-finish-sound = Son à la fin d’un rendu
panel-finish-sound-hover = Jouer la mélodie de fin de FRACTINT quand une exportation ou un rendu de parcours se termine : trois notes ascendantes (1047, 1109, 1175 Hz pendant 100 ms chacune), exactement telles que l’original DOS les codait — synthétisées en signal carré à bande limitée et filtrées passe-haut pour sonner comme le petit haut-parleur de PC qui la jouait.
panel-prefer-detail = Privilégier le détail pendant le zoom
panel-prefer-detail-hover = Pendant un zoom ou un déplacement, continuer d’afficher la dernière image entièrement détaillée — mise à l’échelle et suivie pour accompagner le mouvement — au lieu d’effectuer à chaque image un nouveau rendu en qualité réduite. La vue est rendue entièrement quand le mouvement s’interrompt. Désactivé : le mouvement est rendu en direct en résolution réduite (plus fluide, plus grossier). Les vues peu profondes (mode direct) sont toujours rendues en direct — elles sont peu coûteuses et nettes à chaque image dans tous les cas.

## The control panel: About (the current fractal)

# A link to the fractal's reference page, followed by an arrow.
panel-about-reference = Référence

## The control panel: Advanced ▸ Second graphics card

# A label before a drop-down (wider than the panel's label column already).
gpu2-label = Seconde carte graphique
gpu2-off = Désactivée
# Another device on the same graphics card as the window (the default).
gpu2-same-card = Même carte
# A card the list does not name. $number: the card's number.
gpu2-card-n = Carte { $number }
gpu2-finding = Recherche des cartes graphiques de cette machine…
gpu2-none = Cette machine n’a pas d’autre carte graphique.
gpu2-hover =
    Un autre périphérique graphique rend des images d’une vue profonde pendant qu’elle bouge, ainsi qu’une partie des échantillons supplémentaires qui l’affinent après son arrêt : l’image suit ainsi le zoom et atteint plus tôt sa qualité finale. La carte qui dessine la fenêtre continue de tout dessiner.

    Une autre carte ajoute sa propre vitesse. Deux modèles de cartes différents dessinent quelques pixels légèrement différemment ; si la seconde carte échoue, la vue continue avec une seule.

    « Même carte » (par défaut) ouvre un second périphérique sur cette carte. Il n’ajoute pas de vitesse mais rend sans attendre les images de l’écran. Il partage la carte : une défaillance qui réinitialise la carte met fin à l’application, comme le ferait une défaillance du rendu de la fenêtre elle-même, et le démarrage suivant propose de le désactiver.

## The view (canvas)

# A notice after Ctrl+click pins the Julia c at a deep zoom.
canvas-julia-pin-quantized = Julia c épinglé — mais il est conservé en double précision : à cette profondeur, tout le volet s’arrondit à une seule valeur et l’épingle tombe au centre de la vue. Dézoomez pour viser.
# A small caption in the top-left corner of the view while the orbit overlay is fitted to the view (its "Normalize (fit to view)" option).
canvas-orbit-normalized = orbite (normalisée)
# The minimap's tooltip.
canvas-minimap-hover = Aperçu / vous êtes ici. Faites glisser pour déplacer la vue au zoom actuel, utilisez la molette pour zoomer et dézoomer, cliquez pour sauter vers cette région (zoom d’accueil).

## The timestamp overlay (View ▸ Show timestamp)

# Under the large elapsed-time clock. $frame: the frame number (as the log writes it, without grouping).
overlay-timestamp-frame = image { $frame }

## File ▸ Settings ▸ Reference cache…

cache-title = Cache des références
cache-intro = À profondeur extrême, la partie lente d’une vue est son orbite de référence — jusqu’à une heure d’arithmétique en précision arbitraire. Fractadyne conserve ici les orbites terminées : revenir à un emplacement déjà visité, ou zoomer quelque part à proximité, ne prend alors que quelques secondes. Une orbite pèse environ 4 MB.
cache-off = Désactivé pour cette exécution (une invocation de tâche, ou --no-orbit-cache). Rien n’est lu ni écrit jusqu’au prochain lancement ordinaire.
# Small headings of the window's parts.
cache-location = Emplacement
cache-usage = Utilisation
cache-limit = Limite
cache-no-config = (aucun dossier de configuration disponible)
cache-open-folder = Ouvrir le dossier
# The text on the usage bar. $used and $limit: sizes such as "1.2 GB"; $count: the orbits kept.
cache-usage-bar =
    { $count ->
        [one] { $used } sur { $limit } — { $count } orbite
       *[other] { $used } sur { $limit } — { $count } orbites
    }
cache-limit-hover = Quand le cache dépasse cette taille, les orbites les MOINS COÛTEUSES à construire sont supprimées en premier — une orbite profonde qui a pris une heure survit à n’importe quel nombre d’orbites rapides, aussi récemment qu’elles aient servi.
# Beside the limit. $count: about how many orbits of the live view's length fit in it.
cache-limit-estimate =
    { $count ->
        [one] ≈ { $count } orbite au plafond de la vue en direct
       *[other] ≈ { $count } orbites au plafond de la vue en direct
    }
cache-kept = Conservées (les plus précieuses d’abord)
# Column headers of the list of kept orbits.
cache-col-iterations = itérations
cache-col-precision = précision
cache-col-size = taille
cache-col-last-used = dernière utilisation
# The precision column. $bits: the precision in bits.
cache-row-bits = { $bits } bits
# Under the list. $count: the orbits not listed.
cache-rows-more = … et { $count } de plus
# When an orbit was last used (the "last used" column).
cache-ago-now = à l’instant
# $count: minutes.
cache-ago-minutes = il y a { $count } min
# $count: hours.
cache-ago-hours = il y a { $count } h
# $count: days.
cache-ago-days = il y a { $count } j
# Asked before clearing. $count: the orbits; $size: their total size, such as "1.2 GB".
cache-confirm =
    { $count ->
        [one] Supprimer { $count } orbite ({ $size }) ? La reconstruire coûte du temps, pas des données.
       *[other] Supprimer { $count } orbites ({ $size }) ? Les reconstruire coûte du temps, pas des données.
    }
cache-delete = Supprimer
cache-cancel = Annuler
cache-clear = Vider le cache…
cache-clear-hover = Supprime toutes les orbites stockées. Une confirmation est demandée.
cache-close = Fermer
# $count: the orbits removed.
cache-cleared =
    { $count ->
        [one] Cache des références vidé — { $count } orbite supprimée.
       *[other] Cache des références vidé — { $count } orbites supprimées.
    }
# $why: the system's error message.
cache-clear-failed = Impossible de vider le cache des références : { $why }

## Help ▸ Diagnostics…

diag-title = Diagnostics
diag-intro = Lancez un test pour vérifier que Fractadyne fonctionne correctement sur votre matériel. Les résultats sont enregistrés sur votre machine et peuvent être joints à un signalement de problème — rien n’est envoyé nulle part de lui-même.
# $status: which arbitrary-precision library is in use, as the program reports it (in English).
diag-arithmetic = Arithmétique du zoom profond : { $status }
# The three tests' buttons, each followed by a sentence saying what it does.
diag-run-selftest = Lancer l’autotest
diag-run-uitest = Lancer le test d’interface
diag-run-gputest = Lancer la vérification arithmétique GPU
diag-blurb-selftest = Vérifie les mathématiques et le rendu sur votre GPU par rapport à des résultats connus. Dure environ 15 secondes.
diag-blurb-uitest = Parcourt l’interface et la vue en direct, en prenant des captures d’écran à plusieurs profondeurs de zoom. Dure une minute ou deux et ouvre des fenêtres pendant son exécution.
diag-blurb-gputest = Vérifie l’arithmétique double-float du shader par rapport à des valeurs connues, sur chaque backend graphique que propose votre machine. Dure quelques secondes, n’ouvre aucune fenêtre. Si elle signale un échec, c’est une véritable découverte que nous aimerions voir — merci de la joindre à un signalement.
# While a test runs. $count: the checks finished so far.
diag-checks-done =
    { $count ->
        [one] { $count } vérification effectuée
       *[other] { $count } vérifications effectuées
    }
# A finished test's result. $test: the test's name (Self-test, UI test, GPU arithmetic check).
diag-verdict-captured = { $test } — résultat enregistré
diag-verdict-passed = { $test } — réussite
diag-verdict-problems = { $test } — problèmes signalés
# Under the GPU arithmetic check's result.
diag-informational-note = Celle-ci n’a ni réussite ni échec : elle enregistre la façon dont votre GPU calcule, et un échec signalé est une vraie découverte plutôt qu’un défaut de votre machine. Dans tous les cas, la joindre à un signalement est la chose la plus utile que vous puissiez en faire.
# Under a test's result when it reported problems.
diag-problems-note = Certaines différences sont attendues sur un matériel autre que le GPU de référence — les comparaisons d’images en particulier. Joindre ceci à un signalement de problème est la chose la plus utile que vous puissiez en faire.
diag-open-results = Ouvrir les résultats
diag-attach = Joindre à un signalement de problème…
diag-attach-hover = Ouvre la fenêtre de signalement avec ce résultat inclus : le ticket contient ainsi un test validé automatiquement plutôt qu’une simple description
diag-console = Afficher les diagnostics dans la console
diag-console-hover = Afficher désormais les lignes [fd-…] sur stderr. Désactivé par défaut quand Fractadyne est lancé sans arguments. Cela ne change rien à ce qui est ENREGISTRÉ — le fichier journal reçoit toujours chaque ligne — alors laissez cette option désactivée, sauf si vous surveillez un terminal. Vaut pour cette session seulement ; utilisez l’option --console ou FRACTADYNE_CONSOLE=1 pour capter aussi le démarrage.
diag-close = Fermer
# Errors starting a test. $why: the system's error message; $dir: a folder.
diag-error-no-exe = Impossible de trouver l’exécutable : { $why }
diag-error-create-dir = Impossible de créer { $dir } : { $why }
diag-error-start = Impossible de démarrer le test : { $why }

## Navigate ▸ Misiurewicz explorer…

misi-title = Explorateur de Misiurewicz
misi-mandelbrot-only = L’exploration des points de Misiurewicz ne fonctionne que pour Mandelbrot — changez de fractale pour l’utiliser.
# While the gallery is being built. $seconds: the time so far, such as "3".
misi-solving = Calcul des familles de petits types ({ $seconds } s)…
# Above the gallery. $total: all points; $curated: the hand-picked ones; $found: the ones the sweep found.
misi-summary = { $total } points — { $curated } choisis à la main, { $found } trouvés par un balayage de Newton amorcé des petits types (demi-plan supérieur ; une grille grossière trouve la plupart des bassins, sans garantie de les trouver tous).
# The name of a point the sweep found. $kind: its type, such as "(3,1)"; $point: its coordinate, such as "-0.1011+0.9563i".
misi-entry-name = { $kind } en { $point }
# Under each thumbnail. $kind: the point's type, such as "(3,1)"; $octaves: the zoom period in octaves, such as "3.17"; $twist: the twist per repeat in degrees, such as "+12.5". "oct" is short for octaves.
misi-entry-readout = { $kind } · { $octaves } oct/répétition · { $twist }°
# Before the depth field.
misi-zoom-to = Zoomer jusqu’à
misi-cancel = Annuler
misi-solve-jump = Calculer et sauter
misi-solve-jump-hover = Calculer par Newton ce point exact avec la précision qu’exige la profondeur demandée (en partant de sa coordonnée stockée), puis y sauter directement. Les demandes profondes prennent plus de temps — le calcul est en précision arbitraire.
misi-select-hint = Choisissez un point, indiquez une profondeur et sautez — la ligne du multiplicateur sous chaque miniature montre à quoi ressemblera une plongée à cet endroit : combien d’octaves avant que la vue se répète, et de combien elle tourne à chaque répétition.
misi-cancelled = Calcul annulé.
# $text: what was typed in the depth field.
misi-bad-zoom = Impossible de lire « { $text } » comme un grossissement (écrivez 1e50, 3.2e120, …).
misi-no-kp = Cette entrée n’a pas de (k,p) détecté avec lequel calculer.
misi-parse-failed = La coordonnée stockée n’a pas pu être analysée.
# A notice after the jump. $name: the point's name; $zoom: the magnification, such as "1e50"; $seconds: how long the solve took, such as "1.2".
misi-zoomed = Zoom sur { $name } — { $zoom }× ({ $seconds } s)
# Added after the notice above when the iteration count is too low. $have: the fixed iteration count; $typical: what the depth typically needs.
misi-iter-note = Le réglage Itérations est fixé à { $have } ; cette profondeur en demande généralement ~{ $typical } — augmentez-le (ou activez les itérations automatiques) si la vue s’affiche plate.
# $name: the point's name; $why: why the solve failed (a technical word).
misi-solve-missed = Le calcul profond n’a pas abouti sur { $name } : { $why }. Le point stocké reste exact à sa propre profondeur — essayez une cible moins profonde.

## Life (Game of Life): the side panel

# Shown when the graphics card cannot run the Life view.
life-no-compute = Cet adaptateur graphique ne peut pas exécuter Life (pas de compute shaders).
# Row label: the pattern list (pattern names stay in English).
life-pattern = Motif
life-open = Ouvrir…
life-open-hover = Ouvrir un fichier de motif RLE, .cells ou Life 1.05/1.06.
life-pattern-text = Texte du motif…
life-pattern-text-hover = Coller un motif (du RLE tel que le donne LifeWiki, ou du texte brut).
life-save = Enregistrer…
life-save-hover = Enregistrer l’univers dans son état actuel, au format RLE.
# Row label: the rule list and the typed rule (B3/S23 notation).
life-rule = Règle
# Shown in the rule list when the rule in use is none of the library's.
life-rule-custom = personnalisée
life-apply = Appliquer
# Playback buttons (a pause or play symbol is shown before them).
life-pause = Pause
life-play = Lancer
life-play-hover = Faire évoluer l’univers (Enter).
life-step = Avancer
life-step-hover = Avancer d’une génération (N).
# Tooltip of the button beside Step, which reads "+16", "+1,024"…: that many generations.
life-stride-hover = Avancer de ce nombre de générations (Shift+N).
life-reset = Réinitialiser
life-reset-hover = Revenir au motif tel qu’ouvert (génération 0).
life-speed = Vitesse
# After the speed slider's number: generations a second. Short.
life-speed-unit = gén/s
# Row label: how many generations the "+n" button advances.
life-stride = Pas
# The panel's numbers.
life-generation = Génération
life-population = Population
life-tiles = Tuiles
# $count: the tiles stored; $capacity: how many the GPU pool holds.
life-tiles-of = { $count } sur { $capacity }
life-tiles-hover = Tuiles de 64×64 cellules stockées (4 KiB chacune) et capacité du pool du GPU.
# Row label: what a left-drag in the view does (Pan or Draw).
life-mouse = Souris
life-pan = Déplacer
life-pan-hover = Faites glisser pour déplacer la vue.
life-draw = Dessiner
# The Draw tool's tooltip; the toolbar's shows it after "Draw (D):", so it starts lowercase.
life-draw-hint = cliquez sur une cellule pour l’inverser ; faites glisser pour peindre — un trait qui commence sur une cellule morte dessine, un trait qui commence sur une cellule vivante efface. Un univers en marche se met en pause pendant que vous dessinez et reprend quand vous relâchez.
life-undo = Annuler
life-undo-hover = Revenir à l’univers d’avant la dernière modification, le dernier remplissage, le dernier effacement ou le dernier changement de règle (Ctrl+Z pendant le dessin).
life-grid = Quadrillage
life-grid-hover = Bordures des cellules quand le zoom atteint 8 pixels par cellule ou plus.
# A random fill: "Random" labels the density field, "seed" the seed field after it.
life-random = Aléatoire
life-density-hover = La proportion de cellules qu’un remplissage rend vivantes.
life-seed = graine
life-fill-view = Remplir la vue
# $cells: the largest side of the filled area, in cells (2,048).
life-fill-view-hover = Remplir aléatoirement la zone visible (jusqu’à { $cells } × { $cells } cellules autour de son centre).
life-clear = Effacer
life-clear-hover = Tuer toutes les cellules (Annuler les ramène).

## Life: notices in the panel, and file dialogs

# $path: the file written.
life-saved = Enregistré : { $path }
# $error: why (from the system, in English).
life-save-failed = Échec de l’enregistrement : { $error }
# The universe stopped running. $error: why (in English).
life-stopped = Arrêté : { $error }
# A pattern file named a rule that does not read. $error: why (in English); $rule: the rule kept, in B3/S23 notation.
life-rule-not-understood = La règle du motif n’a pas été comprise ({ $error }) ; règle { $rule } conservée
# $path: the file; $error: why (in English).
life-open-failed = Impossible d’ouvrir { $path } : { $error }
# File-dialog filter names.
life-filter-patterns = Motifs Life
life-filter-rle = Motif codé par plages (RLE)

## Life: the toolbar (its buttons are icons; these are their tooltips)

life-toolbar-pause-hover = Mettre l’univers en pause (Enter)
life-toolbar-play-hover = Faire évoluer l’univers (Enter)
# $stride: the generations a Shift+click steps.
life-toolbar-step-hover = Avancer d’une génération (N) ; Shift+clic ou Shift+N avance de { $stride }
life-toolbar-reset-hover = Revenir au motif tel qu’ouvert (génération 0)
# $hint: the Draw tool's tooltip (life-draw-hint).
life-toolbar-draw-hover = Dessiner (D) : { $hint }

## Life: the Pattern text window

life-text-title = Texte du motif
life-text-intro = Collez un motif : RLE (tel que le donnent LifeWiki et Golly), texte brut .cells, ou Life 1.05 / 1.06.
life-text-load = Charger
life-text-current = Actuel en RLE
life-text-current-hover = Remplir le champ avec l’univers tel qu’il a été chargé en dernier.

## L-systems: the side panel

# Row label: the system list (system names stay in English).
lsys-system = Système
lsys-open = Ouvrir…
lsys-open-hover = Ouvrir un fichier .l de Fractint ou un système .lsys.
lsys-edit = Modifier…
lsys-edit-hover = Le texte du système : axiome, productions, angle — modifiez-le et appliquez.
lsys-save = Enregistrer…
lsys-save-hover = Enregistrer le système sous forme de fichier texte .lsys.
# Tooltip of the "SVG…" button.
lsys-svg-hover = Exporter la vue sous forme de dessin SVG : ses lignes et ses formes en vectoriel, dans ses couleurs.
lsys-follow = L’ordre suit le zoom
lsys-follow-hover = L’ordre augmente à mesure que vous zoomez, si bien que le détail ne s’épuise jamais.
lsys-follow-fixed-hover = Ce système ne grandit pas d’un facteur à chaque ordre : il est donc dessiné à un ordre fixe.
lsys-order = Ordre
lsys-angle = Angle
lsys-sweep-hover = Balayer l’angle, aller et retour : la courbe se transforme en tournant
# A small button that puts back the system's own angle after the slider moved it.
lsys-system-angle = Angle du système
lsys-seed = Graine
lsys-seed-hover = Ce que suivent les choix aléatoires : même graine, même plante.
lsys-seed-new = Nouvelle
lsys-seed-new-hover = Une autre plante : une nouvelle graine.
lsys-line-width = Épaisseur de trait
# Row label of a percentage slider: how much of the curve is drawn.
lsys-draw-on = Tracé
lsys-draw-on-hover = La part de la courbe qui est dessinée, dans l’ordre où la tortue la dessine
lsys-draw-on-play-hover = La courbe qui se dessine elle-même
lsys-colour-by = Couleur selon
# The colourings, in the "Colour by" list.
lsys-colour-position = la position le long de la courbe
lsys-colour-depth = la profondeur de branche
lsys-colour-heading = le cap
# C, < and > are the system's commands that set the colour index: keep them.
lsys-colour-index = l’indice de couleur (C, <, >)
lsys-colour-plain = une seule couleur
# The panel's numbers.
lsys-growth = Croissance
lsys-growth-built = construit comme un mot, à son ordre
lsys-growth-built-hover = Paramétrique ou contextuel : la réécriture d’un module dépend de ses nombres ou de ses voisins, si bien que le mot est construit une génération à la fois (jusqu’à deux millions de modules) et dessiné en double précision — l’ordre ne suit pas le zoom.
# How much the picture grows from one order to the next. $factor: the factor (1.4142).
lsys-growth-factor = ×{ $factor } par ordre
lsys-growth-factor-period-2 = ×{ $factor } par ordre (les ordres avancent de 2 en 2)
lsys-growth-none = aucune (ordre fixe)
lsys-drawn = Dessiné
# $drawn: what was drawn (lsys-drawn-segments, lsys-drawn-shapes or lsys-drawn-segments-shapes); $order: the order.
lsys-drawn-at-order = { $drawn } à l’ordre { $order }
# The same, when the walk stopped at its segment budget.
lsys-drawn-at-order-budget = { $drawn } à l’ordre { $order } (budget)
# $segments: the line segments drawn.
lsys-drawn-segments =
    { $segments ->
        [one] { $segments } segment
       *[other] { $segments } segments
    }
# $shapes: the filled shapes drawn.
lsys-drawn-shapes =
    { $shapes ->
        [one] { $shapes } forme pleine
       *[other] { $shapes } formes pleines
    }
# Both counts. $segments: line segments; $shapes: filled shapes.
lsys-drawn-segments-shapes =
    { $segments ->
        [one] { $segments } segment
       *[other] { $segments } segments
    }, { $shapes ->
        [one] { $shapes } forme pleine
       *[other] { $shapes } formes pleines
    }
# $ms: the walk's time in milliseconds (12.3).
lsys-drawn-hover = parcouru en { $ms } ms ; les sous-arbres hors de la vue sont ignorés, ceux de moins d’un pixel sont dessinés comme un seul segment
# A word too long to build at the order asked for. $wanted: the order asked for; $built: the order drawn.
lsys-drawn-short = Le mot de l’ordre { $wanted } dépasse deux millions de modules : dessiné à l’ordre { $built }.
# $order: the deepest order the app draws.
lsys-deepest-order = À l’ordre le plus profond dessiné ({ $order }) : zoomer davantage n’ajoute aucun détail.

## L-systems: notices in the panel and the editor, and file dialogs

# $path: the file; $error: why (in English).
lsys-open-failed = Impossible d’ouvrir { $path } : { $error }
# Why a file was not opened (shown after "Could not open …:").
lsys-file-too-large = plus de 4 MiB
# Why a Fractint .l file was not opened: it holds no entries. `Name { … }` is the file's syntax: keep it.
lsys-no-entries = aucune entrée (un fichier .l contient des entrées `Name {"{"} … {"}"}`)
# $path: the file written.
lsys-saved = Enregistré : { $path }
# $error: why (from the system, in English).
lsys-save-failed = Échec de l’enregistrement : { $error }
# After File ▸ Export SVG. $path: the file written.
lsys-svg-exported = Exporté : { $path }
# $error: why (from the system, in English).
lsys-svg-failed = Échec de l’exportation : { $error }
# File-dialog filter names: opening (any system file) and saving (one system).
lsys-filter-systems = L-systèmes
lsys-filter-system = L-système

## L-systems: the toolbar

lsys-toolbar-lower-hover = Baisser l’ordre (le fixe)
lsys-toolbar-raise-hover = Augmenter l’ordre (le fixe)
# Between the − and + buttons, in a fixed-width font. $order: the order, padded to 3 characters so the label keeps its width.
lsys-toolbar-order = ordre { $order }
# A toggle: the order follows the zoom. Short.
lsys-toolbar-auto = auto
lsys-toolbar-auto-hover = L’ordre suit le zoom
lsys-toolbar-edit-hover = Modifier le texte du système

## L-systems: the editor window, and a .l file's entries

lsys-editor-title = L-système
# The editor's help. The quoted examples ('X = word', 'seed n', 'define R 1.456'…), the commands and "The Algorithmic Beauty of Plants" (a book's title) are the system's syntax and names: keep them as they are.
lsys-editor-help = Une clé par ligne : angle (en degrés, ou /n pour une division du cercle), axiom, les productions 'X = word', et, en option, heading, draw / move / variables, colour, order. Un symbole dont les alternatives sont choisies au hasard a une ligne pour chacune, avec son poids : 'X (0.3) = word' ; 'seed n' choisit la plante. Dans un mot, {"{"} {"}"} remplit le chemin de la tortue et . ajoute un sommet. Productions paramétriques et contextuelles (The Algorithmic Beauty of Plants) : 'A(s) : s > 1 = F(s)[+A(s/2)]', 'b < a > c = b', avec 'define R 1.456' et 'ignore +-F' ; F(l) avance de l, +(a) tourne de a degrés. Une entrée .l de Fractint peut être collée telle quelle.
lsys-editor-apply = Appliquer
lsys-editor-revert = Rétablir
lsys-editor-revert-hover = Le texte du système affiché.
# The window listing a .l file's entries to pick from. $file: the file's name.
lsys-entries-title = { $file } — entrées
# Tooltip of an entry that does not parse (its error is shown beside it, in English).
lsys-entry-unreadable-hover = Cette entrée est illisible.

## Export: opening a view or location (File ▸ Open view or location…)

# File-dialog filter names.
export-filter-view = Vue ou emplacement Fractadyne
export-filter-image = Image (PNG / EXR)
export-filter-location = Emplacement (.fdn / .kfr)
# $zoom: the magnification (1.2e148), followed by the magnification sign ×.
export-loaded-location = Emplacement chargé @ { $zoom }×
# $zoom: the magnification; $note: what was noteworthy in the file (in English).
export-loaded-location-note = Chargé @ { $zoom }× — { $note }
# $path: the file.
export-not-location = { $path } n’est pas un emplacement Fractadyne.
export-file-too-large = Fichier trop volumineux (ce n’est pas un emplacement .fdn ?).
# $path: the file; $error: why (from the system, in English).
export-read-failed = Impossible de lire { $path } : { $error }
# A Kalles Fraktaler .kfr file did not import. $error: why (in English).
export-import-failed = Échec de l’importation : { $error }
# $path: an image whose embedded text is not a view.
export-not-view = { $path } contient des métadonnées Fractadyne, mais pas de vue.
# $path: the image the view was read from.
export-loaded-view = Vue chargée depuis { $path }
# $path: the image; $note: what was noteworthy in it (in English).
export-loaded-view-note = Vue chargée depuis { $path } — { $note }
export-no-metadata = Ce fichier ne contient pas de métadonnées de vue Fractadyne intégrées.

## Export: notices after an export or a snapshot
## "File → Export image…" in them names the menu items (menu-file, menu-file-export).

# The export window's status while the image renders.
export-rendering = Rendu en cours…
export-canceled = Exportation annulée.
# $error: why (in English).
export-failed = Échec de l’exportation : { $error }
# $size: the image's size in pixels (1920×1080); $path: the file written.
export-saved = Image { $size } enregistrée → { $path }
# An L-system export whose walk stopped at its budget. $size: pixels; $segments: the budget (4,000,000); $path: the file.
export-saved-incomplete = Image { $size } enregistrée (incomplète : au-delà du budget de { $segments } segments — baissez l’ordre) → { $path }
# A two-panel export written as two files. $path: the first of them.
export-saved-two = 2 fichiers enregistrés → { $path }
# $size: pixels; $path: the file written.
export-saved-corrected = Image { $size } enregistrée (glitches corrigés) → { $path }
# $path: the first of the two files.
export-saved-two-corrected = 2 fichiers enregistrés (glitches corrigés) → { $path }
# Added after a finished export's notice. $duration: how long it took (8.3s, 1m 04.0s).
export-elapsed = (en { $duration })
export-preparing-deep = Préparation de l’exportation profonde — construction de la référence (cela peut prendre un moment)…
# $size: pixels (5120×4035); $ss: the supersampling factor; $iterations: the iteration count.
export-large = Grande exportation ({ $size }, suréchantillonnage { $ss }×, { $iterations } itérations) — rendu en arrière-plan ; la correction des glitches est ignorée pour les exportations de cette taille…
export-life-unavailable = L’exportation d’image d’une vue Life n’est pas encore disponible — utilisez une capture d’écran, ou Fichier ▸ Enregistrer le motif Life.
export-already-running = Une exportation est déjà en cours — Fichier → Exporter l’image… affiche sa progression.
# $size: the image's size in pixels.
export-snapshot-background = Instantané : rendu de { $size } en arrière-plan — Fichier → Exporter l’image… affiche la progression et permet de l’annuler.
export-no-gpu = GPU non disponible
# A screen-capture snapshot. $size: pixels; $path: the file written.
export-snapshot-saved = Capture d’écran { $size } enregistrée → { $path }
# $error: why (in English).
export-snapshot-failed = Échec de l’instantané : { $error }

## The colour picker (the popup of a colour swatch)

# The third row's label: the colour as hexadecimal bytes. Short (about 6 characters).
picker-row-hex = Hex
picker-hex-hover = Cliquez pour saisir une valeur hexadécimale
picker-cancel = Annuler
picker-cancel-hover = Rétablir la couleur avec laquelle ce sélecteur s’est ouvert
picker-ok = OK
picker-ok-hover = Garder cette couleur
# Starts the screen eyedropper (an icon is shown before it).
picker-pick = Prélever
picker-pick-hover = Prélever une couleur n’importe où à l’écran
# Tooltip of the disabled Pick button (in the picker and the gradient editor) on Linux and macOS.
picker-eyedropper-unsupported = Prélever une couleur ailleurs à l’écran nécessite une API de capture d’écran de la plateforme. Seule la version Windows en dispose ; sous Linux, il faudrait une dépendance X11, et Wayland exige un portail avec sa propre demande d’autorisation.

## The fractal families: what each one is (the info panel's "About" section and the pickers' hover text).
## The formulas themselves (z -> z^2 + c) are written in code and stay as they are.

fractal-mandelbrot-about = La fractale à temps d’échappement canonique : l’ensemble des c pour lesquels l’orbite de 0 reste bornée. Sa frontière est d’une complexité infinie.
fractal-multibrot3-about = Un ensemble Multibrot — la construction de Mandelbrot à une puissance plus élevée. La puissance d donne une symétrie de rotation d’ordre (d-1).
fractal-multibrot4-about = Multibrot à la puissance 4 : symétrie d’ordre 3, larges bulbes.
fractal-multibrot5-about = Multibrot à la puissance 5 : symétrie d’ordre 4.
fractal-tricorn-about = Le Tricorn (Mandelbar) : conjugue z à chaque pas. Cette application antiholomorphe produit une forme à trois coins.
fractal-burning-ship-about = Les valeurs absolues des parties de z sont prises avant l’élévation au carré ; le résultat ressemble à un navire en flammes.
fractal-celtic-about = Un parent du Burning Ship qui ne prend la valeur absolue que de la partie réelle de z^2, produisant des motifs d’entrelacs celtiques / de cœurs.
fractal-buffalo-about = Une variante à valeur absolue qui prend les valeurs absolues des deux composantes de z^2.
fractal-phoenix-about = Le Phoenix utilise aussi l’itéré précédent, ce qui donne des filaments en forme de flammes. Essayez sa forme Julia via le mode Julia.
fractal-newton-about = L’itération de Newton pour trouver les racines de z^3 = 1, colorée selon la vitesse de convergence de chaque point. Une fractale de convergence (et non d’échappement).
fractal-multibrot6-about = Multibrot à la puissance 6 : symétrie d’ordre 5.
fractal-multibrot7-about = Multibrot à la puissance 7 : symétrie d’ordre 6, les bulbes s’amincissant vers un cercle.
fractal-multibrot8-about = Multibrot à la puissance 8 : symétrie d’ordre 7.
fractal-burning-ship3-about = Le Burning Ship à la puissance 3 : on prend les valeurs absolues, puis on élève au cube.
fractal-burning-ship4-about = Le Burning Ship à la puissance 4.
fractal-burning-ship5-about = Le Burning Ship à la puissance 5.
fractal-tricorn3-about = Le Tricorn (Mandelbar) à la puissance 3 : symétrie d’ordre 4.
fractal-tricorn4-about = Le Tricorn à la puissance 4 : symétrie d’ordre 5.
fractal-tricorn5-about = Le Tricorn à la puissance 5 : symétrie d’ordre 6.
fractal-celtic3-about = Le repli Celtic à la puissance 3 : la valeur absolue de la partie réelle de z^3.
fractal-celtic4-about = Le repli Celtic à la puissance 4.
fractal-celtic5-about = Le repli Celtic à la puissance 5.
fractal-buffalo3-about = Le Buffalo à la puissance 3 : les valeurs absolues des deux parties de z^3.
fractal-buffalo4-about = Le Buffalo à la puissance 4.
fractal-buffalo5-about = Le Buffalo à la puissance 5.
fractal-custom-about = Une formule écrite en expressions de style Fractint (Fractale ▸ Formule personnalisée…). La plupart permettent le zoom profond par perturbation, comme les familles intégrées ; la fenêtre de formule indique si c’est le cas de la vôtre.
fractal-life-about = Le Jeu de la vie de Conway et ses parents sur un plan illimité : règles Generations, règles non totalistes (Hensel), fichiers de motifs. Lancer, avancer pas à pas, dessiner.
fractal-lsystem-about = Systèmes de Lindenmayer : courbes, courbes qui remplissent l’espace, îles et plantes dessinées par une tortue à partir d’une grammaire de réécriture. L’ordre suit le zoom, si bien que le détail ne s’épuise jamais. Les fichiers .l de Fractint s’ouvrent directement.

# Shown where the other families show their formula (z -> z^2 + c).
fractal-custom-formula = z -> la formule que vous écrivez
# Shown where the other families show their formula. "B3/S23" is the rule's notation: keep it.
fractal-life-formula = B3/S23 : naît avec 3 voisines vivantes, survit avec 2 ou 3
# Shown where the other families show their formula. "F = F+F--F+F" is the grammar's notation: keep it.
fractal-lsystem-formula = F = F+F--F+F : chaque symbole est réécrit, puis dessiné par une tortue

# The last paragraph of the picker's hover text for a family that renders only on the direct path.
fractal-no-deep-zoom = Rendu direct uniquement — cette famille n’a pas de zoom profond.

## Names of settings, shown in the Settings menu and the welcome window.

theme-dark = Sombre
theme-light = Clair
# The update track that follows only stable releases.
update-track-stable = Stable
# The update track that also offers pre-releases (betas).
update-track-beta = Bêta (préversions)

## The status bar (its hover texts; the readouts follow in their own section)

status-period-hover = Période du minibrot vers lequel vous avez sauté (appuyez sur M, ou Aller à l’emplacement ▸ Aller à une structure). Elle reste affichée tant que la vue est sur la structure et s’efface quand vous vous en éloignez.

## The playback transport: the player over the view while a tour plays.

# $close: the close button's icon, as drawn on that button.
play-finished-hover = Terminé — revenez en arrière avec le curseur, ou { $close } pour fermer
# $kind: "benchmark" or "script" (the variants hold the wording); $name: the tour's name.
play-playing-hover =
    { $kind ->
        [benchmark] Lecture du banc d’essai « { $name } »
       *[other] Lecture du script « { $name } »
    }
play-restart = Recommencer depuis le début
play-back = Reculer de 10 secondes
play-again = Relire depuis le début
play-resume = Reprendre
play-pause = Pause
play-stop = Arrêter et revenir au début
play-forward = Avancer de 10 secondes
play-speed-hover = Vitesse de lecture (cliquer pour alterner 0.5× / 1× / 2× / 4×)
play-render = Rendre ce script en séquence d’images…
play-render-none = Le banc d’essai intégré n’a pas de fichier de script à rendre
play-open = Lire un autre script…
play-loop = Répéter le parcours quand il arrive à la fin
play-close = Fermer le lecteur et rétablir vos propres réglages de vue
# A notice beside the scrub bar while the renderer holds the tour's clock. Keep it short.
play-waiting = attente du détail
# "[playback] pace" is a setting in the script file: keep it as written.
play-waiting-hover = L’horloge du parcours est en pause pendant que le moteur de rendu résout cette vue (construction de la référence / budget d’itérations en hausse). La lecture reprend d’elle-même ; voir [playback] pace dans le script.
play-scrub = Faites glisser pour naviguer dans le parcours
# $behind: how far the script clock is behind real time, as minutes:seconds (1:05).
play-scrub-behind =
    Faites glisser pour naviguer dans le parcours
    L’horloge du script a { $behind } de retard sur le temps réel (le régulateur d’allure a ralenti le parcours pour laisser le moteur de rendu suivre). La graduation du haut marque où en serait une lecture en temps réel.

## The autopilot's notices (toasts). Where a notice has two lines, the first is the verdict and the second what to do about it.

autopilot-on-detail = Pilote automatique activé — plongée vers le détail (toute action l’arrête)
autopilot-off = Pilote automatique désactivé
autopilot-misi-needs-mandelbrot =
    Pilote automatique : une cible Misiurewicz nécessite l’ensemble de Mandelbrot
    Plongée vers le détail à la place
autopilot-on-misi =
    Pilote automatique activé — recherche du point de Misiurewicz le plus proche…
    Toute action l’arrête
# $seconds: how long the search has run, in whole seconds.
autopilot-misi-finding =
    Pilote automatique : recherche du point de Misiurewicz le plus proche…
    { $seconds } s · toute action l’arrête
autopilot-misi-not-found =
    Pilote automatique : aucun point de Misiurewicz trouvé près de cette vue
    La recherche n’atteint peut-être pas cette profondeur — essayez la cible Détail
# ($preperiod,$period) names the point, as mathematicians write it: keep the notation. $widths: a power of two (2^$widths screen widths).
autopilot-misi-off-screen =
    Pilote automatique : le point de Misiurewicz le plus proche ({ $preperiod },{ $period }) est hors de l’écran
    À ~2^{ $widths } largeurs d’écran — rapprochez-vous d’un centre de spirale
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-not-solved =
    Pilote automatique : le point de Misiurewicz ({ $preperiod },{ $period }) près de cette vue n’a pas pu être calculé
    Essayez la cible Détail
autopilot-misi-refused = Pilote automatique : calcul de Misiurewicz refusé (paire invalide)
autopilot-misi-stopped = Pilote automatique : le solveur de Misiurewicz s’est arrêté de façon inattendue
# ($preperiod,$period) names the point: keep the notation.
autopilot-misi-diving =
    Pilote automatique : plongée vers Misiurewicz ({ $preperiod },{ $period })
    Toute action l’arrête
# ($preperiod,$period) names the point: keep the notation. $octaves: how often the structure repeats (2.37); $twist: its turn in degrees, signed (+12.5).
autopilot-misi-diving-repeat =
    Pilote automatique : plongée vers Misiurewicz ({ $preperiod },{ $period })
    La structure se répète toutes les { $octaves } octaves, torsion { $twist }° · toute action l’arrête
# $limit: the magnification, as written (1e300).
autopilot-dive-limit = Pilote automatique : limite de plongée atteinte (~{ $limit }×)
# $have: the fixed iteration count; $typical: what this depth typically needs.
autopilot-dead-end-short =
    Pilote automatique arrêté : plus rien à viser — les itérations sont fixées à { $have } et cette profondeur en demande généralement ~{ $typical }
    Activez les itérations automatiques ou augmentez le nombre
# $limit: the fixed iteration count.
autopilot-dead-end-cap =
    Pilote automatique arrêté : plus rien à viser — tout ce qui est dans la vue atteint la limite fixe de { $limit } itérations
    Activez les itérations automatiques ou augmentez le nombre
autopilot-dead-end = Pilote automatique : aucun détail devant (arrêté)

## Tours (camera scripts)

# The name of the file type in the file dialog's filter list.
script-file-filter = Script Fractadyne (TOML)
# $name: the script's name.
script-finished = Script terminé — « { $name } »
# The title of the notice that follows (the file's path and the error, in English).
script-load-failed = Impossible de charger le script

## Installing a language file

# The name of the file type in the file dialog's filter list.
language-file-filter = Langue Fractadyne

## The notice when the graphics driver's shader cache stops keeping shaders. The text is shown in a fixed-width font with its line breaks as written: break the lines where they read well.

driver-cache-title = Le pilote graphique ne conserve pas les shaders
# $count: how many shaders compiled again; $pipeline: the slowest one's name (a technical id: keep it); $seconds: its time in seconds (7.5).
driver-cache-body =
    Votre pilote graphique a recompilé { $count } shaders de Fractadyne pendant
    cette session, alors qu’il les avait déjà tous compilés (le plus lent :
    { $pipeline }, { $seconds } s). Son cache ne garde pas les shaders compilés, le plus
    souvent parce qu’il est plein. Tant que cela dure, les exportations, les vues
    profondes et le démarrage de Fractadyne peuvent passer des secondes à compiler.
# $folder: the cache folder's path; $size: its size in MB (1024). "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia-folder =
    Pour corriger : fermez Fractadyne, puis supprimez le contenu de
      { $folder }   ({ $size } MB)
    Le pilote reconstruit le cache. Pour qu’il ne se remplisse plus, augmentez
    Panneau de configuration NVIDIA > Gérer les paramètres 3D > Taille du cache des shaders.
# "NVIDIA Control Panel > Manage 3D settings > Shader Cache Size" names NVIDIA's own settings: use the names NVIDIA's software shows in this language.
driver-cache-fix-nvidia =
    Pour corriger : augmentez Panneau de configuration NVIDIA > Gérer les paramètres 3D >
    Taille du cache des shaders, ou videz le cache de shaders du pilote.
# "AMD Software > Graphics > Advanced > Reset Shader Cache" names AMD's own settings: use the names AMD's software shows in this language.
driver-cache-fix-amd = Pour corriger : AMD Software > Graphiques > Avancé > Réinitialiser le cache des shaders.
driver-cache-fix-other = Pour corriger : videz le cache de shaders du pilote graphique.

## Why a language file could not be installed: each ends the sentence "{ $file } is not a usable language file: …".

language-install-not-utf8 = ce n’est pas du texte UTF-8
language-install-too-large = trop volumineux
# "fractadyne-<language>.ftl" is the file name to use: keep it as written.
language-install-no-tag = il n’indique pas de quelle langue il s’agit (nommez-le fractadyne-<language>.ftl)
language-install-english = l’anglais est intégré
language-install-empty = il ne traduit aucun message de cette version
language-install-no-folder = il n’y a pas de dossier de paramètres

## The notice after a session that did not end cleanly, when its iteration setting was set aside.

# $base: the iteration count automatic iterations start from.
quarantine-setting-auto = itérations automatiques à partir d’une base de { $base }
# $count: the fixed iteration count.
quarantine-setting-fixed =
    { $count ->
        [one] { $count } itération
       *[other] { $count } itérations
    }
# $end: "device" when the session lost its graphics device, "unclean" otherwise (the variants hold the wording). $depth: the magnification, as written (1.2e148). $setting and $restored: an iteration setting (quarantine-setting-auto or -fixed).
quarantine-notice =
    { $end ->
        [device] La session précédente a perdu son périphérique graphique à { $depth }× alors qu’elle était réglée sur { $setting }. Cette session a démarré avec { $restored } à la place, si bien qu’elle ne peut pas retomber directement dans la même charge.
       *[other] La session précédente ne s’est pas terminée correctement à { $depth }× alors qu’elle était réglée sur { $setting }. Cette session a démarré avec { $restored } à la place, si bien qu’elle ne peut pas retomber directement dans la même charge.
    }

## The second graphics card's status line, under its setting in the control panel.

gpu2-state-off = Désactivée : la carte graphique de cette fenêtre effectue tout le rendu.
gpu2-state-starting = Démarrage…
# $name: the card's name (and its graphics API).
gpu2-state-running = Utilisée : { $name }
# $why: the error, as the graphics driver reports it.
gpu2-state-failed = Démarrage impossible : { $why }
# $name: the card's name.
gpu2-state-lost = Arrêtée : { $name } a été perdue. Une seule carte désormais ; choisissez-la de nouveau pour réessayer.
# $state: one of the lines above. "--worker-gpu" is a command-line option: keep it.
gpu2-state-pinned = { $state } (défini par --worker-gpu)

## The custom-formula dialog: how deep the formula renders (one line under the formula).

formula-depth-deep = Zoom profond par perturbation, plage étendue au-delà de 1e28x comme pour les fractales intégrées (chaque pas est itéré : pas encore d’approximation par séries ni de BLA pour les formules personnalisées).
# $why: what the formula has that keeps it off the deep-zoom path (one of the formula-depth-why messages below).
formula-depth-direct = Rendu direct ({ $why } n’a pas encore de forme pour le zoom profond) : net jusqu’à ce qu’un pas de c en simple précision couvre un pixel — environ 1e4x à 1e5x, et moins profond loin de l’origine.
formula-depth-why-init = une section d’initialisation
formula-depth-why-bailout = un test d’échappement propre
formula-depth-why-variable = une variable conservée d’un pas à l’autre
formula-depth-why-previous = l’itéré précédent
formula-depth-why-comparison = une comparaison ou un bloc if
# floor, ceil, trunc and round are functions of the formula language: keep them.
formula-depth-why-rounding = un arrondi (floor, ceil, trunc, round)
# "maxit" is a word of the formula language: keep it.
formula-depth-why-maxit = maxit
formula-depth-why-power = une puissance dont l’exposant varie
formula-depth-why-perturbed = un programme déjà perturbé

## Image and frame sizes: the presets described in words (the others are standard names: HD 720p, WUXGA, DCI 4K).

# $size: the pixel size (2048×2048).
size-preset-square = Carré — { $size }
# $ratio: the aspect ratio (9:16); $size: the pixel size.
size-preset-portrait = Portrait { $ratio } — { $size }
# $ratio: the aspect ratio (32:9); $size: the pixel size.
size-preset-super-ultrawide = Super ultra-large { $ratio } — { $size }

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
fractal-family-custom = Personnalisée
# Conway's Game of Life.
fractal-family-life = Life
fractal-family-lsystem = L-système
# A family at a power: "Multibrot 3", "Burning Ship 4". $family: a name above; $power: the power.
fractal-with-power = { $family } { $power }

## Names of choices: colouring methods, orbit traps, palette animation, snapshots, angle units.

color-method-smooth = Itération lissée
color-method-stripe = Moyenne de rayures
color-method-triangle = Inégalité triangulaire
color-method-trap = Piège orbital
color-method-distance = Estimation de distance
color-method-decomposition = Décomposition
# The shape an orbit trap catches the orbit with.
trap-point = Point
trap-cross = Croix
trap-circle = Cercle
# Palette animation: none.
palette-anim-off = Désactivée
palette-anim-forward = Avant
palette-anim-reverse = Arrière
palette-anim-pingpong = Aller-retour
# Palette animation: a new random gradient every so often.
palette-anim-random = Dégradés aléatoires
# What the Snapshot button does.
snapshot-mode-ask = Demander à chaque fois
snapshot-mode-screen = Capture d’écran
snapshot-mode-render = Rendu complet
# The unit of an angle typed into Go to location's polar fields.
angle-degrees = degrés
angle-radians = radians
angle-turns = tours

## Report an issue: the choices. (The report itself is sent in English.)

issue-kind-crash = Plantage / blocage de l’application
issue-kind-rendering = Rendu incorrect
issue-kind-performance = Problème de performances
issue-kind-ui = Problème d’interface / d’ergonomie
issue-kind-feature = Demande de fonctionnalité
issue-kind-other = Autre
severity-low = Faible
severity-medium = Moyenne
severity-high = Élevée
severity-blocking = Bloquante
repro-always = Toujours
repro-sometimes = Parfois
repro-once = Arrivé une fois
repro-cannot = Impossible à reproduire

## The benchmark's depths and the diagnostics' tests.

# $zoom: the dive's final magnification (1e32).
bench-depth-standard = Standard ({ $zoom }×)
# $zoom: the dive's final magnification (1e48).
bench-depth-ultra = Ultra profond ({ $zoom }×)
diag-test-selftest = Autotest
diag-test-uitest = Test d’interface
diag-test-gputest = Vérification arithmétique GPU

## The status bar's readouts: each word is followed by a value in a slot of fixed width. Keep the words short (one word each).

status-center = centre
status-cursor = curseur
status-zoom = zoom
# Short for iterations.
status-iter = iter
# The period of the minibrot jumped to.
status-period = période
# A Game of Life view's scale (followed by "1 px = 4 cells" or "8 px / cell").
status-scale = échelle
# A Game of Life view's generation. Short.
status-generation = gén
# A Game of Life view's population. Short.
status-population = pop
# An L-system's order (how many times its rules were applied).
status-order = ordre
# How many segments and shapes an L-system drew.
status-drawn = dessiné
# $value: how many cells one pixel covers (4, 9.99e300). "px" is the unit symbol: keep it.
life-scale-cells = 1 px = { $value } cellules
# $value: how many pixels one cell covers (8). "px" is the unit symbol: keep it.
life-scale-pixels = { $value } px / cellule

## The status bar's limit diagnostic: a short label (after a warning sign), and what it means when hovered.

# The view is as deep as this location can be rendered. Short.
status-limit-depth = limite de profondeur
# Every pixel ran out of iterations. Short.
status-limit-exhausted = iter épuisées
# Pixels are capped at the iteration budget. Short.
status-limit-capped = iter plafonnées
# $pixels: how many pixels wide one step of the coordinates is; $sharp_to: the magnification the view is sharp to (written out, 1,234,567).
status-limit-direct-detail = Cette vue est rendue directement sur le GPU en simple précision, où un pas des coordonnées fait ici { $pixels } pixels de large : des pixels voisins calculent le même point, si bien que l’image se décompose en blocs. À cet emplacement, elle est nette jusqu’à environ { $sharp_to }×. Cette formule n’a pas encore de chemin de zoom profond (perturbation) : zoomer davantage n’apporte donc plus de détail.
# $samples: the GPU buffer's capacity in orbit samples; $iterations: the iterations pixels are limited to.
status-limit-depth-detail = L’orbite de référence a atteint la capacité du tampon GPU ({ $samples } échantillons) sans s’échapper : les pixels sont donc limités ici à ~{ $iterations } itérations, et un détail plus fin ne peut pas être calculé dans cette vue — ni en direct ni à l’exportation. C’est la limite de profondeur actuelle pour cette famille d’emplacements.
# $budget: the largest iteration budget this view can be given.
status-limit-exhausted-detail = Tous les pixels ont épuisé leurs itérations au plus grand budget que cette vue peut recevoir ({ $budget }). Soit la vue est ici dans l’ensemble (normal en profondeur), soit elle demande plus d’itérations que l’application ne peut actuellement en fournir — l’image est la même dans les deux cas, donc le budget a été ramené à une valeur peu coûteuse.
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations" is the setting's name in the Rendering panel.
status-limit-capped-max = { $percent } % des pixels ont épuisé leurs itérations au budget de { $budget }, et ceux qui s’échappent se terminent juste à ce plafond — la vue est sous-résolue. Itérations est déjà au maximum : cette vue demande donc plus que ce que l’application peut actuellement fournir.
# $percent: the share of pixels, a whole number; $budget: the iteration budget. "Iterations" and "Rendering" are the names of a setting and a panel section.
status-limit-capped-raise = { $percent } % des pixels ont épuisé leurs itérations au budget de { $budget }, et ceux qui s’échappent se terminent juste à ce plafond — la vue est sous-résolue. Augmentez Itérations (Contrôles ▸ Qualité) pour la résoudre.
