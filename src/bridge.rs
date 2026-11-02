// Objets exposés au QML (module Fili.Files): FolderModel, la liste du dossier affiché (navigation, historique, tri,
// ouverture, mise à jour en direct par inotify, opérations: copier/couper/coller en arrière-plan, corbeille,
// renommage, annulation); Fili, l'environnement (thème suivi en direct, traductions, lieux de la barre latérale).
#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++Qt" {
        include!(<QtCore/QAbstractListModel>);
        #[qobject]
        type QAbstractListModel;
    }

    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
    }

    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        #[qproperty(QString, path, READ, NOTIFY = changed)]
        #[qproperty(bool, show_hidden, cxx_name = "showHidden", READ, NOTIFY = changed)]
        #[qproperty(i32, sort_key, cxx_name = "sortKey", READ, NOTIFY = changed)]
        #[qproperty(bool, sort_desc, cxx_name = "sortDesc", READ, NOTIFY = changed)]
        #[qproperty(bool, can_back, cxx_name = "canBack", READ, NOTIFY = changed)]
        #[qproperty(bool, can_forward, cxx_name = "canForward", READ, NOTIFY = changed)]
        #[qproperty(QString, status, READ, NOTIFY = changed)]
        #[qproperty(QString, error, READ, NOTIFY = changed)]
        #[qproperty(QString, filter, READ, NOTIFY = changed)]
        // Titre des résultats affichés à la place du contenu du dossier (vide: dossier normal)
        #[qproperty(QString, results, READ, NOTIFY = changed)]
        // Dossier à montrer dans le fil d'Ariane et le titre: le dossier commun des résultats, sinon le dossier affiché
        #[qproperty(QString, shown, READ, NOTIFY = changed)]
        #[qproperty(bool, busy, READ, NOTIFY = changed)]
        // Avancement de l'opération en cours (0 à 1), -1 tant que le total n'est pas connu
        #[qproperty(f64, progress, READ, NOTIFY = changed)]
        #[qproperty(QString, job, READ, NOTIFY = changed)]
        #[qproperty(bool, in_trash, cxx_name = "inTrash", READ, NOTIFY = changed)]
        #[qproperty(bool, can_paste, cxx_name = "canPaste", READ, NOTIFY = changed)]
        #[qproperty(bool, can_undo, cxx_name = "canUndo", READ, NOTIFY = changed)]
        type FolderModel = super::FolderModelRust;

        #[qsignal]
        fn changed(self: Pin<&mut FolderModel>);
        // Le contenu du dossier a changé sur le disque (ou après une opération): la vue remet sa sélection en place
        #[qsignal]
        fn reloaded(self: Pin<&mut FolderModel>);
        // Un nom existe déjà dans la destination: la vue demande quoi faire (resolveConflict); names = un nom par ligne
        #[qsignal]
        fn conflict(self: Pin<&mut FolderModel>, names: &QString, dest: &QString);

        // L'index récursif du dossier est prêt: la recherche couvre tous les niveaux
        #[qsignal]
        #[cxx_name = "indexReady"]
        fn index_ready(self: Pin<&mut FolderModel>);
        // Miniature de ce fichier prête (génération en arrière-plan)
        #[qsignal]
        #[cxx_name = "thumbReady"]
        fn thumb_ready(self: Pin<&mut FolderModel>, path: &QString);

        // Les déplacements rendent la ligne à sélectionner ensuite (-1: rien n'a changé)
        #[qinvokable]
        fn go(self: Pin<&mut FolderModel>, path: &QString) -> i32;
        // Affiche une liste de fichiers (chemins séparés par des retours à la ligne) à la place du dossier, sous un titre
        #[qinvokable]
        #[cxx_name = "showResults"]
        fn show_results(self: Pin<&mut FolderModel>, title: &QString, paths: &QString);
        // Revient au contenu du dossier
        #[qinvokable]
        #[cxx_name = "closeResults"]
        fn close_results(self: Pin<&mut FolderModel>);
        #[qinvokable]
        fn activate(self: Pin<&mut FolderModel>, row: i32) -> i32;
        #[qinvokable]
        fn up(self: Pin<&mut FolderModel>) -> i32;
        #[qinvokable]
        fn back(self: Pin<&mut FolderModel>) -> i32;
        #[qinvokable]
        fn forward(self: Pin<&mut FolderModel>) -> i32;
        #[qinvokable]
        fn refresh(self: Pin<&mut FolderModel>, row: i32) -> i32;
        #[qinvokable]
        #[cxx_name = "sortBy"]
        fn sort_by(self: Pin<&mut FolderModel>, key: i32, row: i32) -> i32;
        #[qinvokable]
        #[cxx_name = "toggleHidden"]
        fn toggle_hidden(self: Pin<&mut FolderModel>, row: i32) -> i32;
        // Ne garder que les noms qui contiennent ce texte (vide: tout)
        #[qinvokable]
        #[cxx_name = "setPickFilter"]
        fn set_pick_filter(self: Pin<&mut FolderModel>, index: i32);
        #[qinvokable]
        #[cxx_name = "setFilter"]
        fn set_filter(self: Pin<&mut FolderModel>, text: &QString);
        #[qinvokable]
        fn quicklook(self: &FolderModel, path: &QString);
        #[qinvokable]
        fn find(self: &FolderModel, prefix: &QString) -> i32;
        #[qinvokable]
        #[cxx_name = "pathAt"]
        fn path_at(self: &FolderModel, row: i32) -> QString;
        // Ligne portant ce nom (-1: absent)
        #[qinvokable]
        #[cxx_name = "rowOf"]
        fn find_name(self: &FolderModel, name: &QString) -> i32;
        #[qinvokable]
        fn has(self: &FolderModel, path: &QString) -> bool;
        // Efface le message d'erreur (une commande de ctl lit ensuite si elle a échoué)
        #[qinvokable]
        #[cxx_name = "clearError"]
        fn clear_error(self: Pin<&mut FolderModel>);
        // Chemins (un par ligne) des lignes affichées dont le nom correspond au motif (* et ?), ou qui ont le même type
        // que les chemins donnés
        #[qinvokable]
        fn matching(self: &FolderModel, pattern: &QString) -> QString;
        #[qinvokable]
        #[cxx_name = "sameKind"]
        fn same_kind(self: &FolderModel, paths: &QString) -> QString;
        // Description de la sélection pour la barre d'état (un élément: nom et taille ou contenu)
        #[qinvokable]
        fn describe(self: &FolderModel, paths: &QString) -> QString;
        #[qinvokable]
        #[cxx_name = "pathIsDir"]
        fn path_is_dir(self: &FolderModel, path: &QString) -> bool;
        // Opérations: les chemins sont séparés par des retours à la ligne; les rendus sont la ligne à sélectionner
        // ensuite (-1: aucune). Les erreurs vont dans la barre d'état (error).
        #[qinvokable]
        fn uris(self: &FolderModel, paths: &QString) -> QString;
        #[qinvokable]
        fn copy(self: Pin<&mut FolderModel>, paths: &QString);
        #[qinvokable]
        fn cut(self: Pin<&mut FolderModel>, paths: &QString);
        #[qinvokable]
        fn paste(self: Pin<&mut FolderModel>);
        // Relit le presse-papiers système (des fichiers y sont-ils?) pour canPaste
        #[qinvokable]
        #[cxx_name = "refreshPaste"]
        fn refresh_paste(self: Pin<&mut FolderModel>);
        // Réponse à `conflict`: "replace" (l'ancien va à la corbeille), "both" (suffixe « copie »), "skip", "cancel"
        #[qinvokable]
        #[cxx_name = "resolveConflict"]
        fn resolve_conflict(self: Pin<&mut FolderModel>, choice: &QString);
        #[qinvokable]
        #[cxx_name = "dropItems"]
        fn drop_items(self: Pin<&mut FolderModel>, urls: &QString, dest: &QString, copy: bool);
        #[qinvokable]
        fn trash(self: Pin<&mut FolderModel>, paths: &QString, row: i32) -> i32;
        #[qinvokable]
        #[cxx_name = "putBack"]
        fn put_back(self: Pin<&mut FolderModel>, paths: &QString, row: i32) -> i32;
        #[qinvokable]
        #[cxx_name = "emptyTrash"]
        fn empty_trash(self: Pin<&mut FolderModel>);
        #[qinvokable]
        fn rename(self: Pin<&mut FolderModel>, path: &QString, name: &QString) -> i32;
        #[qinvokable]
        #[cxx_name = "newFolder"]
        fn new_folder(self: Pin<&mut FolderModel>) -> i32;
        #[qinvokable]
        fn undo(self: Pin<&mut FolderModel>);
        // Applications pour ouvrir ce fichier, en JSON: [{id, name}] (la préférée d'abord)
        #[qinvokable]
        #[cxx_name = "openWithList"]
        fn open_with_list(self: &FolderModel, path: &QString) -> QString;
        #[qinvokable]
        #[cxx_name = "openWith"]
        fn open_with(self: &FolderModel, id: &QString, path: &QString);
        // Tous les chemins sont des archives que extract sait ouvrir
        #[qinvokable]
        #[cxx_name = "allArchives"]
        fn all_archives(self: &FolderModel, paths: &QString) -> bool;
        #[qinvokable]
        fn extract(self: Pin<&mut FolderModel>, paths: &QString, password: &QString);
        // Une archive demande un mot de passe (ou le mot de passe donné est faux): la vue le demande, puis rappelle extract avec ces archives
        #[qsignal]
        #[cxx_name = "passwordNeeded"]
        fn password_needed(self: Pin<&mut FolderModel>, paths: &QString, wrong: bool);
        // Nom proposé par le panneau de compression (sans extension)
        #[qinvokable]
        #[cxx_name = "archiveName"]
        fn archive_name(self: &FolderModel, paths: &QString) -> QString;
        // Compresse avec les choix du panneau: nom (sans extension), format, niveau (fast|normal|best), sans fichiers cachés
        #[qinvokable]
        fn compress(self: Pin<&mut FolderModel>, paths: &QString, name: &QString, format: &QString, level: &QString, skip_hidden: bool, password: &QString);
        #[qinvokable]
        fn cancel(self: Pin<&mut FolderModel>);
        // Renommage en lot (modes replace, number, case; voir ops::bulk_name). L'aperçu rend du JSON
        // [{from, to, ok, same}] dans l'ordre d'affichage (ok = nom valide et libre); l'application renomme ce qui change
        #[qinvokable]
        #[cxx_name = "bulkPreview"]
        fn bulk_preview(self: &FolderModel, paths: &QString, mode: &QString, a: &QString, b: &QString, start: i32) -> QString;
        #[qinvokable]
        #[cxx_name = "bulkApply"]
        fn bulk_apply(self: Pin<&mut FolderModel>, paths: &QString, mode: &QString, a: &QString, b: &QString, start: i32) -> bool;
        // Tâches en cours et en attente (JSON [{label, progress}], progress -2 = en attente) et annulation de l'une d'elles
        #[qinvokable]
        fn tasks(self: &FolderModel) -> QString;
        #[qinvokable]
        #[cxx_name = "cancelTask"]
        fn cancel_task(self: Pin<&mut FolderModel>, index: i32);
        // Copie dans le dossier même (nom « copie »)
        #[qinvokable]
        fn duplicate(self: Pin<&mut FolderModel>, paths: &QString);
        // Terminal (foot) dans ce dossier, sinon dans le dossier affiché
        #[qinvokable]
        fn terminal(self: &FolderModel, path: &QString);
        // Infos d'un ou plusieurs éléments, en JSON: [[libellé, valeur]]; la taille des dossiers suit par infoSize
        #[qinvokable]
        fn info(self: &FolderModel, paths: &QString) -> QString;
        #[qinvokable]
        #[cxx_name = "startInfoSize"]
        fn start_info_size(self: Pin<&mut FolderModel>, paths: &QString);
        #[qsignal]
        #[cxx_name = "infoSize"]
        fn info_size(self: Pin<&mut FolderModel>, text: &QString);
        // Empreinte SHA-256 d'un fichier, calculée à part (sha256sum), rendue par infoHash
        #[qinvokable]
        #[cxx_name = "startInfoHash"]
        fn start_info_hash(self: Pin<&mut FolderModel>, path: &QString);
        #[qsignal]
        #[cxx_name = "infoHash"]
        fn info_hash(self: Pin<&mut FolderModel>, text: &QString);
        // Droits (rwx, 0..511) d'un fichier ou dossier qui nous appartient, -1 sinon (lien, autre propriétaire)
        #[qinvokable]
        #[cxx_name = "permMode"]
        fn perm_mode(self: &FolderModel, path: &QString) -> i32;
        // Change les droits (annulable); rend le texte « rwxr-xr-x (755) », vide en cas d'échec
        #[qinvokable]
        #[cxx_name = "setPerm"]
        fn set_perm(self: Pin<&mut FolderModel>, path: &QString, mode: i32) -> QString;
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginInsertRows"]
        unsafe fn begin_insert_rows(self: Pin<&mut FolderModel>, parent: &QModelIndex, first: i32, last: i32);
        #[inherit]
        #[cxx_name = "endInsertRows"]
        unsafe fn end_insert_rows(self: Pin<&mut FolderModel>);
        #[inherit]
        #[cxx_name = "beginRemoveRows"]
        unsafe fn begin_remove_rows(self: Pin<&mut FolderModel>, parent: &QModelIndex, first: i32, last: i32);
        #[inherit]
        #[cxx_name = "endRemoveRows"]
        unsafe fn end_remove_rows(self: Pin<&mut FolderModel>);
    }

    unsafe extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginResetModel"]
        unsafe fn begin_reset_model(self: Pin<&mut FolderModel>);
        #[inherit]
        #[cxx_name = "endResetModel"]
        unsafe fn end_reset_model(self: Pin<&mut FolderModel>);
    }

    extern "RustQt" {
        #[cxx_override]
        fn data(self: &FolderModel, index: &QModelIndex, role: i32) -> QVariant;
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &FolderModel) -> QHash_i32_QByteArray;
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &FolderModel, parent: &QModelIndex) -> i32;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(bool, light, READ, NOTIFY = theme_changed)]
        #[qproperty(QString, accent, READ, NOTIFY = theme_changed)]
        #[qproperty(bool, key_nav, cxx_name = "keyNav")]
        type Fili = super::FiliRust;

        #[qsignal]
        #[cxx_name = "themeChanged"]
        fn theme_changed(self: Pin<&mut Fili>);

        // Les volumes ou les montages ont changé: la barre latérale relit ses lieux
        #[qsignal]
        #[cxx_name = "placesChanged"]
        fn places_changed(self: Pin<&mut Fili>);
        // Commande de l'assistant (fili-file ctl): numéro (à rendre à reply), verbe, argument, corps
        #[qsignal]
        fn command(self: Pin<&mut Fili>, id: i32, verb: &QString, arg: &QString, body: &QString);
        // Volume monté à la demande de mount_volume (chemin) ou erreur (texte)
        #[qsignal]
        fn mounted(self: Pin<&mut Fili>, path: &QString);
        #[qsignal]
        #[cxx_name = "volumeError"]
        fn volume_error(self: Pin<&mut Fili>, message: &QString);
        // Mode sélecteur de fichiers (portail FileChooser): "" hors de ce mode, sinon "open" ou "save"
        #[qinvokable]
        #[cxx_name = "pickKind"]
        fn pick_kind(self: &Fili) -> QString;
        #[qinvokable]
        #[cxx_name = "pickTitle"]
        fn pick_title(self: &Fili) -> QString;
        #[qinvokable]
        #[cxx_name = "pickName"]
        fn pick_name(self: &Fili) -> QString;
        #[qinvokable]
        #[cxx_name = "pickAccept"]
        fn pick_accept(self: &Fili) -> QString;
        #[qinvokable]
        #[cxx_name = "pickMultiple"]
        fn pick_multiple(self: &Fili) -> bool;
        #[qinvokable]
        #[cxx_name = "pickDirectory"]
        fn pick_directory(self: &Fili) -> bool;
        // Noms des filtres, en JSON: ["Images", "Tous les fichiers"]
        #[qinvokable]
        #[cxx_name = "pickFilters"]
        fn pick_filters(self: &Fili) -> QString;
        #[qinvokable]
        #[cxx_name = "pickFilter"]
        fn pick_filter(self: &Fili) -> i32;
        // Écrit la sélection (un chemin par ligne) et quitte; pickCancel quitte sans rien écrire
        #[qinvokable]
        #[cxx_name = "pickFinish"]
        fn pick_finish(self: &Fili, paths: &QString, filter: i32);
        // Colonnes affichées de la liste (clés séparées par des virgules), gardées dans l'état de l'utilisateur
        #[qinvokable]
        fn columns(self: &Fili) -> QString;
        #[qinvokable]
        #[cxx_name = "setColumns"]
        fn set_columns(self: &Fili, keys: &QString);
        // Vue de la liste de fichiers: "list" ou "grid", gardée dans l'état de l'utilisateur
        #[qinvokable]
        #[cxx_name = "viewMode"]
        fn view_mode(self: &Fili) -> QString;
        #[qinvokable]
        #[cxx_name = "setViewMode"]
        fn set_view_mode(self: &Fili, mode: &QString);
        // Petit état d'interface gardé entre deux lancements (largeurs des colonnes « col.size », du panneau d'aperçu
        // « preview »): valeur en texte, vide si absente
        #[qinvokable]
        #[cxx_name = "uiState"]
        fn ui_state(self: &Fili, key: &QString) -> QString;
        #[qinvokable]
        #[cxx_name = "setUiState"]
        fn set_ui_state(self: &Fili, key: &QString, value: &QString);
        #[qinvokable]
        #[cxx_name = "previewShown"]
        fn preview_shown(self: &Fili) -> bool;
        #[qinvokable]
        #[cxx_name = "setPreviewShown"]
        fn set_preview_shown(self: &Fili, on: bool);
        #[qinvokable]
        #[cxx_name = "pathExists"]
        fn path_exists(self: &Fili, path: &QString) -> bool;
        // Aperçu d'un fichier: première ligne = genre (image, pdf, text, folder, none), reste = texte ou chemin de l'image
        #[qinvokable]
        #[cxx_name = "previewInfo"]
        fn preview_info(self: &Fili, path: &QString) -> QString;
        #[qinvokable]
        #[cxx_name = "pickCancel"]
        fn pick_cancel(self: &Fili);
        // Favoris de l'utilisateur (signets GTK) et presse-papiers (wl-copy)
        #[qinvokable]
        #[cxx_name = "addFavorite"]
        fn add_favorite(self: Pin<&mut Fili>, path: &QString);
        #[qinvokable]
        #[cxx_name = "removeFavorite"]
        fn remove_favorite(self: Pin<&mut Fili>, path: &QString);
        // La corbeille contient quelque chose (icône pleine ou vide de la barre latérale)
        #[qinvokable]
        #[cxx_name = "trashFull"]
        fn trash_full(self: &Fili) -> bool;
        #[qinvokable]
        #[cxx_name = "isFavorite"]
        fn is_favorite(self: &Fili, path: &QString) -> bool;
        #[qinvokable]
        #[cxx_name = "copyText"]
        fn copy_text(self: &Fili, text: &QString);
        // Propositions de chemins pour ce que l'utilisateur tape dans le champ du chemin (JSON: [{name, path, dir}])
        #[qinvokable]
        #[cxx_name = "pathSuggest"]
        fn path_suggest(self: &Fili, text: &QString) -> QString;
        #[qinvokable]
        #[cxx_name = "mountVolume"]
        fn mount_volume(self: Pin<&mut Fili>, dev: &QString);
        #[qinvokable]
        #[cxx_name = "unmountVolume"]
        fn unmount_volume(self: Pin<&mut Fili>, dev: &QString);
        // Partage (fili-share): protocoles et pairs en JSON par shareProtocolsReady / sharePeersReady (fil à part, la
        // recherche LocalSend dure 2 s); shareSend envoie les fichiers (un chemin par ligne) en tâche détachée
        #[qinvokable]
        #[cxx_name = "shareProtocols"]
        fn share_protocols(self: Pin<&mut Fili>);
        #[qsignal]
        #[cxx_name = "shareProtocolsReady"]
        fn share_protocols_ready(self: Pin<&mut Fili>, json: &QString);
        #[qinvokable]
        #[cxx_name = "sharePeers"]
        fn share_peers(self: Pin<&mut Fili>, proto: &QString, tag: &QString);
        #[qsignal]
        #[cxx_name = "sharePeersReady"]
        fn share_peers_ready(self: Pin<&mut Fili>, proto: &QString, tag: &QString, json: &QString);
        #[qinvokable]
        #[cxx_name = "shareSend"]
        fn share_send(self: &Fili, proto: &QString, id: &QString, name: &QString, paths: &QString);
        // Choisir les fichiers à envoyer à cet appareil (fili-share send ouvre le sélecteur de Casier)
        #[qinvokable]
        #[cxx_name = "sharePick"]
        fn share_pick(self: &Fili, proto: &QString, id: &QString, name: &QString);
        // Réception (fili-share receive-info / received): état des protocoles et fichiers reçus en JSON par
        // receiveInfoReady(info, received); receiveSet active ou coupe un protocole puis renvoie l'état
        #[qinvokable]
        #[cxx_name = "receiveInfo"]
        fn receive_info(self: Pin<&mut Fili>);
        #[qsignal]
        #[cxx_name = "receiveInfoReady"]
        fn receive_info_ready(self: Pin<&mut Fili>, info: &QString, received: &QString);
        #[qinvokable]
        #[cxx_name = "receiveSet"]
        fn receive_set(self: Pin<&mut Fili>, proto: &QString, on: bool);
        // Lecteur réseau (gvfs): adresse smb://, sftp://, davs://, ftp://, nfs://; réponse par mounted / volumeError
        #[cxx_name = "connectServer"]
        fn connect_server(self: Pin<&mut Fili>, uri: &QString, user: &QString, password: &QString);
        #[qinvokable]
        fn tr(self: &Fili, text: &QString) -> QString;
        // Icône de dossier à la couleur d'accentuation (autres icônes inchangées)
        #[qinvokable]
        fn tint(self: &Fili, url: &QString, accent: &QString, light: bool) -> QString;
        // Fiche « À propos de Casier »: lignes [[libellé, valeur]] en JSON, et dossier du thème par défaut (clair ou sombre)
        #[qinvokable]
        #[cxx_name = "aboutRows"]
        fn about_rows(self: &Fili) -> QString;
        #[qinvokable]
        #[cxx_name = "aboutFolder"]
        fn about_folder(self: &Fili, light: bool) -> QString;
        #[qinvokable]
        #[cxx_name = "aboutLogo"]
        fn about_logo(self: &Fili, light: bool) -> QString;
        // Lieux de la barre latérale, en JSON: [{section, name, path, icon}]
        #[qinvokable]
        fn places(self: &Fili) -> QString;
        // Premier argument de la ligne de commande (dossier, fichier ou URL file://), sinon le dossier personnel
        #[qinvokable]
        #[cxx_name = "startPath"]
        fn start_path(self: &Fili) -> QString;
        #[qinvokable]
        fn home(self: &Fili) -> QString;
        // Lancé avec --info (FileManager1.ShowItemProperties): la fenêtre d'informations s'ouvre au démarrage
        #[qinvokable]
        #[cxx_name = "startInfo"]
        fn start_info(self: &Fili) -> bool;
        // Contexte pour l'assistant (fili-context): dossier affiché et sélection (chemins séparés par des retours à la ligne);
        // un dossier vide retire le contexte (fermeture)
        // Réponse (une ligne JSON) à la commande `id` de ctl
        #[qinvokable]
        fn reply(self: &Fili, id: i32, text: &QString);
        #[qinvokable]
        fn pid(self: &Fili) -> i32;
        #[qinvokable]
        #[cxx_name = "publishContext"]
        fn publish_context(self: &Fili, location: &QString, selection: &QString, results: &QString);
    }

    impl cxx_qt::Initialize for Fili {}
    impl cxx_qt::Threading for Fili {}
    impl cxx_qt::Initialize for FolderModel {}
    impl cxx_qt::Threading for FolderModel {}
}

use crate::{apps, fs, i18n, icons, ops, pick, visits, volumes, watch::Watcher};
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QByteArray, QHash, QHashPair_i32_QByteArray, QModelIndex, QString, QVariant};
use qobject::*;
use std::{
    collections::HashSet,
    env,
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Instant, SystemTime, UNIX_EPOCH},
};

const ROLE_NAME: i32 = 256;
const ROLE_PATH: i32 = 257;
const ROLE_DIR: i32 = 258;
const ROLE_SIZE: i32 = 259;
const ROLE_DATE: i32 = 260;
const ROLE_ICON: i32 = 261;
const ROLE_HIDDEN: i32 = 262;
const ROLE_THUMB: i32 = 263;
const ROLE_THUMB_READY: i32 = 264;
const ROLE_IMAGE: i32 = 265;
const ROLE_KIND: i32 = 266;
const ROLE_CREATED: i32 = 267;
const ROLE_PERMS: i32 = 268;
const ROLE_OWNER: i32 = 269;

fn home() -> String {
    env::var("HOME").unwrap_or_else(|_| "/".into())
}

// Dossiers personnels XDG (user-dirs.dirs, à défaut ~/Desktop, ~/Documents...) -> nom d'icône freedesktop
fn xdg_dirs() -> Vec<(String, &'static str)> {
    let home = home();
    let config = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{home}/.config"));
    let text = std::fs::read_to_string(format!("{config}/user-dirs.dirs")).unwrap_or_default();
    let dirs = [
        ("XDG_DESKTOP_DIR", "Desktop", "user-desktop"),
        ("XDG_DOCUMENTS_DIR", "Documents", "folder-documents"),
        ("XDG_DOWNLOAD_DIR", "Downloads", "folder-download"),
        ("XDG_MUSIC_DIR", "Music", "folder-music"),
        ("XDG_PICTURES_DIR", "Pictures", "folder-pictures"),
        ("XDG_VIDEOS_DIR", "Videos", "folder-videos"),
    ];
    let mut out = Vec::new();
    for (key, fallback, icon) in dirs {
        let v = text.lines().find_map(|l| l.strip_prefix(key)?.strip_prefix('=')).map(|v| v.trim().trim_matches('"'));
        let p = v.map_or_else(|| format!("{home}/{fallback}"), |v| v.replace("$HOME", &home));
        if p != home && Path::new(&p).is_dir() {
            out.push((p, icon));
        }
    }
    out
}

// Favoris par défaut, dans l'ordre du Finder: Applications, Bureau, Documents, Téléchargements, dossier personnel,
// puis Images, Musique, Vidéos. (chemin, nom d'icône)
fn default_favorites() -> Vec<(String, &'static str)> {
    let home = home();
    let xdg = xdg_dirs();
    let by_icon = |icon: &str| xdg.iter().find(|(_, i)| *i == icon).cloned();
    let mut out = Vec::new();
    let apps = format!("{home}/Applications");
    if Path::new(&apps).is_dir() {
        out.push((apps, "folder-applications"));
    }
    for icon in ["user-desktop", "folder-documents", "folder-download"] {
        out.extend(by_icon(icon));
    }
    out.push((home.clone(), "user-home"));
    for icon in ["folder-pictures", "folder-music", "folder-videos"] {
        out.extend(by_icon(icon));
    }
    out
}

fn bookmarks_file() -> PathBuf {
    let config = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{}/.config", home()));
    PathBuf::from(config).join("gtk-3.0/bookmarks")
}

// Favoris de l'utilisateur: signets GTK (partagés avec le sélecteur de fichiers), « file:///chemin libellé »
fn bookmarks() -> Vec<(String, String)> {
    let text = std::fs::read_to_string(bookmarks_file()).unwrap_or_default();
    text.lines()
        .filter_map(|line| {
            let (uri, label) = line.split_once(' ').unwrap_or((line, ""));
            Some((percent_decode(uri.strip_prefix("file://")?), label.to_string()))
        })
        .collect()
}

fn write_bookmarks(list: &[(String, String)]) -> std::io::Result<()> {
    let file = bookmarks_file();
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut text = String::new();
    for (p, label) in list {
        text.push_str(&format!("file://{}{}{}\n", ops::encode(p), if label.is_empty() { "" } else { " " }, label));
    }
    std::fs::write(file, text)
}

use ops::decode as percent_decode;

// Chemin absolu nettoyé (~, file://, ., ..), sans suivre les liens: le chemin affiché reste celui parcouru
fn normalize(p: &str, base: &str) -> PathBuf {
    let p = p.trim();
    let p = match p.strip_prefix("file://") {
        Some(rest) => percent_decode(rest),
        None => p.to_string(),
    };
    let p = if p == "~" {
        home()
    } else if let Some(rest) = p.strip_prefix("~/") {
        format!("{}/{rest}", home())
    } else {
        p
    };
    let full = if p.starts_with('/') { PathBuf::from(p) } else { Path::new(base).join(p) };
    let mut out = PathBuf::from("/");
    for c in full.components() {
        match c {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::Normal(n) => out.push(n),
            _ => {}
        }
    }
    out
}

fn json(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

// Lance un programme détaché (ne meurt pas avec fili-file), ramassé par un fil pour ne pas laisser de zombie
fn spawn(program: &str, arg: &str, dir: &Path) {
    spawn_detached(Command::new(program).arg(arg).current_dir(dir));
}

// Lance une commande détachée (propre groupe de processus, flux fermés) et la fait ramasser par un fil. Un échec de
// lancement (programme absent) est écrit sur la sortie d'erreur de fili-file plutôt que perdu.
fn spawn_detached(cmd: &mut Command) {
    let child = cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).process_group(0).spawn();
    match child {
        Ok(mut child) => {
            std::thread::spawn(move || child.wait());
        }
        Err(e) => eprintln!("fili-file: cannot run {}: {e}", cmd.get_program().to_string_lossy()),
    }
}

// Miniatures déjà vues sur le disque: data() est appelée à chaque affichage d'une ligne, un stat par appel serait perdu
static THUMBS_SEEN: std::sync::Mutex<Option<HashSet<PathBuf>>> = std::sync::Mutex::new(None);

fn thumb_known(t: &Path) -> bool {
    let mut g = THUMBS_SEEN.lock().unwrap_or_else(|e| e.into_inner());
    let seen = g.get_or_insert_with(HashSet::new);
    if seen.contains(t) {
        return true;
    }
    let ok = t.exists();
    if ok {
        seen.insert(t.to_path_buf());
    }
    ok
}

fn thumb_exists(e: &fs::Entry) -> bool {
    thumb_known(&thumb_path(e))
}

// Miniature d'un PDF: première page par pdftoppm, gardée dans ~/.cache/filigrane/thumbs (nom = chemin, date, taille)
fn thumb_path(e: &fs::Entry) -> PathBuf {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    (&e.path, e.mtime, e.size).hash(&mut h);
    let cache = env::var("XDG_CACHE_HOME").unwrap_or_else(|_| format!("{}/.cache", home()));
    PathBuf::from(cache).join(format!("filigrane/thumbs/{:016x}.png", h.finish()))
}

// Aperçu du panneau latéral: "genre\ncontenu". Texte: 6 Kio au plus, seulement de l'UTF-8 sans octet nul.
// PDF: première page en 480 px par pdftoppm, gardée dans le cache des miniatures.
fn preview(path: &str) -> String {
    use std::io::Read;
    let p = Path::new(path);
    let Ok(meta) = std::fs::metadata(p) else { return "none\n".into() };
    if meta.is_dir() {
        let Ok(rd) = std::fs::read_dir(p) else { return "none\n".into() };
        let mut names: Vec<(bool, String)> = rd
            .flatten()
            .map(|e| (e.path().is_dir(), e.file_name().to_string_lossy().into_owned()))
            .filter(|(_, n)| !n.starts_with('.'))
            .collect();
        let total = names.len();
        names.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.to_lowercase().cmp(&b.1.to_lowercase())));
        let mut out = String::from("folder\n");
        for (d, n) in names.iter().take(30) {
            out.push_str(n);
            if *d {
                out.push('/');
            }
            out.push('\n');
        }
        if total > 30 {
            out.push_str(&format!("… {}\n", total - 30));
        }
        return out;
    }
    let name = path.rsplit('/').next().unwrap_or("");
    if icons::is_image_name(name) {
        return if meta.len() < 20 << 20 { format!("image\nfile://{path}") } else { "none\n".into() };
    }
    if name.to_lowercase().ends_with(".pdf") {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        (path, meta.len(), meta.modified().ok(), "preview").hash(&mut h);
        let cache = env::var("XDG_CACHE_HOME").unwrap_or_else(|_| format!("{}/.cache", home()));
        let stem = PathBuf::from(cache).join(format!("filigrane/thumbs/{:016x}", h.finish()));
        let png = stem.with_extension("png");
        if !png.exists() {
            if let Some(dir) = png.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let _ = Command::new("pdftoppm")
                .args(["-png", "-f", "1", "-l", "1", "-scale-to", "480", "-singlefile", path])
                .arg(&stem)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        return if png.exists() { format!("image\nfile://{}", png.display()) } else { "none\n".into() };
    }
    let mut buf = Vec::new();
    if std::fs::File::open(p).and_then(|f| f.take(6144).read_to_end(&mut buf)).is_err() || buf.contains(&0) {
        return "none\n".into();
    }
    let text = match String::from_utf8(buf) {
        Ok(t) => t,
        Err(e) => {
            // Coupure au milieu d'un caractère à la fin: on garde le début valide
            let n = e.utf8_error().valid_up_to();
            if e.utf8_error().error_len().is_some() || n + 4 < e.as_bytes().len() {
                return "none\n".into();
            }
            String::from_utf8_lossy(&e.as_bytes()[..n]).into_owned()
        }
    };
    format!("text\n{text}")
}

static THUMBING: AtomicBool = AtomicBool::new(false);
// Relecture du dossier par inotify en cours / à refaire (voir live_reload)
static RELOADING: AtomicBool = AtomicBool::new(false);
static RELOAD_AGAIN: AtomicBool = AtomicBool::new(false);
// Numéro de l'index en cours de construction; tout changement l'annule
static INDEX_GEN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

// Nom sans extension d'archive reconnue (None: ce n'est pas une archive)
fn archive_stem(name: &str) -> Option<String> {
    let lower = name.to_lowercase();
    for ext in [".tar.gz", ".tar.xz", ".tar.bz2", ".tar.zst", ".tgz", ".tar", ".zip", ".7z", ".rar", ".iso"] {
        if lower.ends_with(ext) && name.len() > ext.len() {
            return Some(name[..name.len() - ext.len()].to_string());
        }
    }
    None
}

// Sortie de `fili-share ARGS` (JSON); liste vide si le script manque ou échoue
fn share_run(args: &[&str]) -> String {
    match Command::new(bin("fili-share")).args(args).stdin(Stdio::null()).stderr(Stdio::null()).output() {
        Ok(o) if !o.stdout.is_empty() => String::from_utf8_lossy(&o.stdout).into_owned(),
        _ => "[]".into(),
    }
}

fn bin(name: &str) -> String {
    let p = format!("{}/.local/bin/{name}", home());
    if Path::new(&p).exists() { p } else { name.to_string() }
}

#[derive(Default)]
pub struct FolderModelRust {
    path: QString,
    show_hidden: bool,
    sort_key: i32,
    sort_desc: bool,
    can_back: bool,
    can_forward: bool,
    status: QString,
    error: QString,
    filter: QString,
    results: QString,
    shown: QString,
    // Filtre du sélecteur de fichiers (index dans pick::Pick::filters, -1: aucun)
    pick_filter: i32,
    busy: bool,
    progress: f64,
    job: QString,
    in_trash: bool,
    can_paste: bool,
    can_undo: bool,
    dir: String,
    all: Vec<fs::Entry>,
    // Tout le contenu du dossier à tous les niveaux (recherche floue), construit à la première saisie
    index: Option<Vec<fs::Entry>>,
    indexing: bool,
    rows: Vec<fs::Entry>,
    history_back: Vec<String>,
    history_forward: Vec<String>,
    now: i64,
    clipboard: Vec<String>,
    clip_cut: bool,
    undo: Vec<Undo>,
    cancel: Arc<AtomicBool>,
    watcher: Option<Watcher>,
    // Copie ou déplacement en attente de la réponse à `conflict`
    pending: Option<(Vec<String>, String, Mode)>,
    // Copies et déplacements en attente de la fin de l'opération en cours (file d'attente, dans l'ordre)
    queue: Vec<(Vec<String>, String, Mode, Policy)>,
}

// Ce que Ctrl+Z défait (le plus récent en dernier, 20 au plus)
enum Undo {
    // (ancien chemin, nouveau chemin)
    Rename(PathBuf, PathBuf),
    Trash(Vec<ops::Trashed>),
    // Éléments créés (copies, nouveau dossier, éléments remis en place): défaire = les mettre à la corbeille
    Created(Vec<PathBuf>),
    // (origine, destination)
    Moved(Vec<(PathBuf, PathBuf)>),
    // (chemin, anciens droits)
    Mode(PathBuf, u32),
    // Renommage en lot: (ancien chemin, nouveau chemin)
    Renames(Vec<(PathBuf, PathBuf)>),
}

// Que faire d'un nom déjà pris dans la destination
#[derive(Clone, Copy, PartialEq)]
enum Policy {
    Both,
    Replace,
    Skip,
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Copy,
    Move,
    // Déplacer sur le même volume, copier sinon (glisser-déposer)
    Auto,
}

struct ArchiveOpts {
    name: String,
    format: String,
    level: String,
    skip_hidden: bool,
}

fn paths_of(list: &QString) -> Vec<String> {
    list.to_string().split('\n').filter(|p| !p.is_empty()).map(String::from).collect()
}

// Presse-papiers Wayland (wl-clipboard). Les lectures sont courtes: wl-paste rend la main dès que la sélection est lue.
fn clip_set(paths: &[String]) {
    if paths.is_empty() {
        return;
    }
    let text = paths.iter().map(|p| format!("file://{}", ops::encode(p))).collect::<Vec<_>>().join("\r\n");
    let child = Command::new("wl-copy").args(["--type", "text/uri-list"]).stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
    if let Ok(mut child) = child {
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            let _ = stdin.write_all(text.as_bytes());
        }
        std::thread::spawn(move || child.wait());
    }
}

fn clip_clear() {
    let _ = Command::new("wl-copy").arg("--clear").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();
}

fn clip_read(mime: &str) -> Option<String> {
    let out = Command::new("wl-paste").args(["--no-newline", "--type", mime]).stdin(Stdio::null()).stderr(Stdio::null()).output().ok()?;
    (out.status.success() && !out.stdout.is_empty()).then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

fn clip_has_files() -> bool {
    let out = Command::new("wl-paste").arg("--list-types").stdin(Stdio::null()).stderr(Stdio::null()).output();
    out.is_ok_and(|o| {
        let t = String::from_utf8_lossy(&o.stdout);
        t.lines().any(|l| l == "text/uri-list" || l == "x-special/gnome-copied-files")
    })
}

// Fichiers du presse-papiers et « couper » si l'application source le dit (GNOME, Thunar: x-special/gnome-copied-files)
fn clip_get() -> Option<(Vec<String>, Option<bool>)> {
    let (text, gnome) = match clip_read("x-special/gnome-copied-files") {
        Some(t) => (t, true),
        None => (clip_read("text/uri-list")?, false),
    };
    let parsed = parse_clip(&text, gnome);
    (!parsed.0.is_empty()).then_some(parsed)
}

// gnome: première ligne « copy » ou « cut », puis une URI par ligne; sinon une URI par ligne (# = commentaire)
fn parse_clip(text: &str, gnome: bool) -> (Vec<String>, Option<bool>) {
    let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#'));
    let mut cut = None;
    let mut rest: Vec<&str> = lines.by_ref().collect();
    if gnome && let Some(&first) = rest.first() && !first.contains("://") {
        cut = Some(first == "cut");
        rest.remove(0);
    }
    let items = rest.iter().filter(|l| l.starts_with("file://")).map(|l| normalize(l, "/").to_string_lossy().into_owned()).collect();
    (items, cut)
}

fn job_label(items: &[String], mode: Mode) -> String {
    let what = if items.len() == 1 { base(&items[0]) } else { fs::count(items.len()) };
    if mode == Mode::Copy { i18n::trf("Copying %1", &[&what]) } else { i18n::trf("Moving %1", &[&what]) }
}

// Chemins proposés pour un texte tapé: le dossier est la partie avant le dernier « / » (~ et .. compris), le reste est
// cherché façon fzf dans les noms de ce dossier (les fichiers cachés seulement si le reste commence par un point).
// Rend (nom, chemin complet, dossier?): meilleurs scores d'abord, dossiers avant fichiers à score égal.
fn suggest(text: &str, limit: usize) -> Vec<(String, String, bool)> {
    let t = text.trim_start();
    // Sans « / »: dossiers déjà visités qui correspondent (comme zoxide), puis le contenu du dossier personnel
    let mut jumps: Vec<(String, String, bool)> = Vec::new();
    if !t.contains('/') && !t.is_empty() && !t.starts_with('~') {
        let mut scored: Vec<(i64, String)> = visits::ranked().into_iter().filter_map(|v| fs::fuzzy(&v.path, t).map(|s| (s, v.path))).collect();
        scored.sort_by(|a, b| b.0.cmp(&a.0));
        jumps = scored.into_iter().take(limit).map(|(_, p)| (base(&p), p, true)).collect();
    }
    let (dir, needle) = match t.rfind('/') {
        Some(i) => (&t[..=i], &t[i + 1..]),
        None => ("~/", t),
    };
    let dir = normalize(dir, "/");
    let Ok(rd) = std::fs::read_dir(&dir) else { return Vec::new() };
    let mut found: Vec<(i64, bool, String, String)> = Vec::new();
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') && !needle.starts_with('.') {
            continue;
        }
        let score = if needle.is_empty() { Some(0) } else { fs::fuzzy(&name, needle) };
        let Some(score) = score else { continue };
        let path = dir.join(&name);
        found.push((score, path.is_dir(), name, path.to_string_lossy().into_owned()));
    }
    found.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)).then_with(|| a.2.to_lowercase().cmp(&b.2.to_lowercase())));
    jumps.extend(found.into_iter().take(limit).map(|(_, d, n, p)| (n, p, d)).filter(|(_, p, _)| !jumps.iter().any(|(_, q, _)| q == p)).collect::<Vec<_>>());
    jumps.truncate(limit);
    jumps
}

fn base(p: &str) -> String {
    Path::new(p).file_name().map_or(p.to_string(), |n| n.to_string_lossy().into_owned())
}

impl FolderModel {
    fn row(&self, row: i32) -> Option<&fs::Entry> {
        usize::try_from(row).ok().and_then(|r| self.rows.get(r))
    }

    fn row_of(&self, name: &str) -> i32 {
        self.rows.iter().position(|e| e.name == name).map_or(0, |i| i as i32)
    }

    // Liste courante filtrée (cachés) et triée, sans relire le disque
    fn filtered(&self) -> Vec<fs::Entry> {
        let (hidden, key, desc) = (self.show_hidden, fs::Key::from(self.sort_key), self.sort_desc);
        let needle = self.filter.to_string();
        if !needle.trim().is_empty() {
            // Recherche floue sur le chemin relatif, dans tout le sous-arbre (à défaut d'index, le dossier seul);
            // les résultats affichés de l'assistant restent la seule source dans leur mode. Meilleur score d'abord.
            let source = match (&self.index, self.results.is_empty()) {
                (Some(i), true) => i,
                _ => &self.all,
            };
            let mut scored: Vec<(i64, &fs::Entry)> = source
                .iter()
                .filter(|e| hidden || !e.name.split('/').any(|c| c.starts_with('.')))
                .filter(|e| self.picked(e))
                .filter_map(|e| fs::fuzzy(&e.name, &needle).map(|s| (s, e)))
                .collect();
            scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| fs::natural(&a.1.name, &b.1.name)));
            // Métadonnées lues seulement pour les meilleurs résultats (l'index n'en porte pas)
            return scored
                .into_iter()
                .take(500)
                .map(|(_, e)| match fs::entry(Path::new(&e.path)) {
                    Some(n) => fs::Entry { name: e.name.clone(), ..n },
                    None => e.clone(),
                })
                .collect();
        }
        let mut rows: Vec<fs::Entry> = self.all.iter().filter(|e| (hidden || !e.hidden()) && self.picked(e)).cloned().collect();
        fs::sort(&mut rows, key, desc);
        rows
    }

    // Sélecteur de fichiers: seuls les dossiers en mode « choisir un dossier », sinon les fichiers du filtre choisi
    fn picked(&self, e: &fs::Entry) -> bool {
        let Some(p) = pick::get() else { return true };
        if p.directory {
            return e.is_dir;
        }
        match usize::try_from(self.pick_filter).ok().and_then(|i| p.filters.get(i)) {
            Some(f) => e.is_dir || f.matches(&e.name, icons::mime_type(&e.name).as_deref()),
            None => true,
        }
    }

    pub fn set_pick_filter(mut self: Pin<&mut Self>, index: i32) {
        self.as_mut().rust_mut().pick_filter = index;
        self.as_mut().rebuild();
        self.as_mut().changed();
    }

    fn set_status(mut self: Pin<&mut Self>) {
        let n = self.rows.len();
        let free = fs::free(Path::new(&self.dir)).map(fs::size);
        let searching = !self.filter.to_string().trim().is_empty() || !self.results.is_empty();
        let status = match free {
            _ if searching => i18n::trf("%1 found", &[&fs::count(n)]),
            Some(f) => i18n::trf("%1, %2 available", &[&fs::count(n), &f]),
            None => fs::count(n),
        };
        let mut r = self.as_mut().rust_mut();
        r.status = QString::from(&status);
        r.now = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64);
    }

    // Remplace la liste (nouveau dossier, tri, cachés): la vue repart de zéro
    fn rebuild(mut self: Pin<&mut Self>) {
        let rows = self.filtered();
        unsafe { self.as_mut().begin_reset_model() };
        self.as_mut().rust_mut().rows = rows;
        self.as_mut().set_status();
        unsafe { self.as_mut().end_reset_model() };
    }

    // Même dossier relu: seules les lignes ajoutées, retirées ou modifiées bougent, la vue garde sa position
    fn apply(mut self: Pin<&mut Self>) {
        type Key<'a> = (&'a str, u64, i64, bool);
        fn key(e: &fs::Entry) -> Key<'_> {
            (&e.path, e.size, e.mtime, e.is_dir)
        }
        let new = self.filtered();
        let new_keys: HashSet<Key> = new.iter().map(key).collect();
        let changes = {
            let old_keys: HashSet<Key> = self.rows.iter().map(key).collect();
            self.rows.iter().filter(|e| !new_keys.contains(&key(e))).count() + new.iter().filter(|e| !old_keys.contains(&key(e))).count()
        };
        if changes > 200 {
            unsafe { self.as_mut().begin_reset_model() };
            self.as_mut().rust_mut().rows = new;
            unsafe { self.as_mut().end_reset_model() };
        } else {
            let none = QModelIndex::default();
            for i in (0..self.rows.len()).rev() {
                if !new_keys.contains(&key(&self.rows[i])) {
                    unsafe { self.as_mut().begin_remove_rows(&none, i as i32, i as i32) };
                    self.as_mut().rust_mut().rows.remove(i);
                    unsafe { self.as_mut().end_remove_rows() };
                }
            }
            // Les lignes restantes sont dans l'ordre de new: insérer les manquantes à leur place
            for (i, e) in new.iter().enumerate() {
                if self.rows.get(i).is_none_or(|o| key(o) != key(e)) {
                    unsafe { self.as_mut().begin_insert_rows(&none, i as i32, i as i32) };
                    self.as_mut().rust_mut().rows.insert(i, e.clone());
                    unsafe { self.as_mut().end_insert_rows() };
                }
            }
        }
        self.as_mut().set_status();
        self.as_mut().changed();
        self.as_mut().queue_thumbs();
    }

    // Génère en arrière-plan les miniatures PDF manquantes du dossier (40 au plus, une série à la fois)
    fn queue_thumbs(self: Pin<&mut Self>) {
        let todo: Vec<(String, PathBuf)> = self
            .rows
            .iter()
            .filter(|e| icons::is_pdf(e))
            .map(|e| (e.path.clone(), thumb_path(e)))
            .filter(|(_, t)| !thumb_known(t))
            .take(40)
            .collect();
        if todo.is_empty() || THUMBING.swap(true, Ordering::SeqCst) {
            return;
        }
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            for (src, out) in todo {
                let Some(dir) = out.parent() else { continue };
                let _ = std::fs::create_dir_all(dir);
                let stem = out.with_extension("");
                let ok = Command::new("pdftoppm")
                    .args(["-png", "-f", "1", "-l", "1", "-scale-to", "96", "-singlefile", &src])
                    .arg(&stem)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .is_ok_and(|s| s.success());
                if ok {
                    let _ = qt.queue(move |mut m| m.as_mut().thumb_ready(&QString::from(&src)));
                }
            }
            THUMBING.store(false, Ordering::SeqCst);
        });
    }

    // Construit en arrière-plan l'index de tous les niveaux sous le dossier affiché (300 000 éléments au plus); la
    // recherche s'étend à mesure que les paquets arrivent
    fn start_index(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().index = None;
        self.as_mut().rust_mut().indexing = false;
        if self.dir.is_empty() || !self.results.is_empty() {
            return;
        }
        let serial = INDEX_GEN.fetch_add(1, Ordering::SeqCst) + 1;
        let (dir, hidden) = (PathBuf::from(&self.dir), self.show_hidden);
        {
            let mut r = self.as_mut().rust_mut();
            r.index = Some(Vec::new());
            r.indexing = true;
        }
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            fs::walk(&dir, hidden, 300_000, |batch| {
                if INDEX_GEN.load(Ordering::SeqCst) != serial {
                    return false;
                }
                let _ = qt.queue(move |mut m| {
                    if INDEX_GEN.load(Ordering::SeqCst) == serial {
                        if let Some(i) = m.as_mut().rust_mut().index.as_mut() {
                            i.extend(batch);
                        }
                        m.as_mut().rebuild();
                        m.as_mut().changed();
                    }
                });
                true
            });
            let _ = qt.queue(move |mut m| {
                if INDEX_GEN.load(Ordering::SeqCst) == serial {
                    m.as_mut().rust_mut().indexing = false;
                    m.as_mut().index_ready();
                }
            });
        });
    }

    // Abandonne l'index (filtre vidé, autre dossier): le calcul en cours s'arrête
    fn drop_index(mut self: Pin<&mut Self>) {
        INDEX_GEN.fetch_add(1, Ordering::SeqCst);
        let mut r = self.as_mut().rust_mut();
        r.index = None;
        r.indexing = false;
    }

    // Relit le dossier affiché sur place; false si on ne peut plus le lire
    fn reload_rows(mut self: Pin<&mut Self>) -> bool {
        // Résultats affichés: on ne relit pas le dossier, on relit chaque résultat et on retire ceux qui ont disparu
        if !self.results.is_empty() {
            let all: Vec<fs::Entry> = self
                .all
                .iter()
                .filter_map(|e| fs::entry(Path::new(&e.path)).map(|n| fs::Entry { name: e.name.clone(), ..n }))
                .collect();
            self.as_mut().rust_mut().all = all;
            self.as_mut().apply();
            return true;
        }
        match fs::list(Path::new(&self.dir)) {
            Ok(all) => {
                self.as_mut().rust_mut().all = all;
                if self.filter.is_empty() {
                    self.as_mut().drop_index();
                } else {
                    self.as_mut().start_index();
                }
                self.as_mut().apply();
                true
            }
            Err(_) => false,
        }
    }

    // Changement vu par inotify: relecture du dossier sur un fil (un gros dossier ne gèle pas la fenêtre), ou, si le
    // dossier a disparu, le plus proche dossier qui existe. Une seule relecture à la fois: un changement qui arrive
    // pendant ce temps en demande une autre dès la fin de la première.
    fn live_reload(mut self: Pin<&mut Self>) {
        if self.dir.is_empty() {
            return;
        }
        // Résultats affichés (assistant): relecture des seuls résultats, sur place
        if !self.results.is_empty() {
            self.as_mut().reload_rows();
            self.as_mut().reloaded();
            return;
        }
        if RELOADING.swap(true, Ordering::SeqCst) {
            RELOAD_AGAIN.store(true, Ordering::SeqCst);
            return;
        }
        let (dir, qt) = (self.dir.clone(), self.qt_thread());
        std::thread::spawn(move || {
            let listing = fs::list(Path::new(&dir));
            let _ = qt.queue(move |mut m| {
                RELOADING.store(false, Ordering::SeqCst);
                if m.dir == dir {
                    match listing {
                        Ok(all) => {
                            m.as_mut().rust_mut().all = all;
                            if m.filter.is_empty() {
                                m.as_mut().drop_index();
                            } else {
                                m.as_mut().start_index();
                            }
                            m.as_mut().apply();
                        }
                        Err(_) => {
                            let mut p = PathBuf::from(&m.dir);
                            while !p.is_dir() && p.pop() {}
                            m.as_mut().navigate(p, None);
                        }
                    }
                    m.as_mut().reloaded();
                }
                if RELOAD_AGAIN.swap(false, Ordering::SeqCst) {
                    m.live_reload();
                }
            });
        });
    }

    // Charge dir; select: nom à sélectionner. Échec: le dossier affiché ne change pas, l'erreur va dans la barre d'état
    fn load(mut self: Pin<&mut Self>, dir: &Path, select: Option<&str>) -> Result<i32, ()> {
        match fs::list(dir) {
            Ok(all) => {
                let dir = dir.to_string_lossy().into_owned();
                let mut r = self.as_mut().rust_mut();
                r.all = all;
                r.path = QString::from(&dir);
                r.in_trash = Path::new(&dir) == ops::trash_files();
                if let Some(w) = &r.watcher {
                    w.watch(Path::new(&dir));
                }
                r.shown = QString::from(&dir);
                r.dir = dir;
                r.error = QString::default();
                r.filter = QString::default();
                r.results = QString::default();
                self.as_mut().drop_index();
                self.as_mut().rebuild();
                self.as_mut().queue_thumbs();
                let row = select.map_or(0, |n| self.row_of(n));
                self.as_mut().update_history_flags();
                Ok(row)
            }
            Err(e) => {
                let name = dir.file_name().map_or("/".into(), |n| n.to_string_lossy().into_owned());
                let msg = i18n::trf("Cannot open “%1”: %2", &[&name, &fs::error(&e)]);
                self.as_mut().rust_mut().error = QString::from(&msg);
                self.as_mut().changed();
                Err(())
            }
        }
    }

    fn update_history_flags(mut self: Pin<&mut Self>) {
        let mut r = self.as_mut().rust_mut();
        r.can_back = !r.history_back.is_empty();
        r.can_forward = !r.history_forward.is_empty();
        self.as_mut().changed();
    }

    // Va dans target en gardant l'historique; un fichier ouvre son dossier et le sélectionne
    fn navigate(mut self: Pin<&mut Self>, target: PathBuf, select: Option<String>) -> i32 {
        let (dir, select) = if !target.is_dir() && target.parent().is_some() && target.exists() {
            let name = target.file_name().map(|n| n.to_string_lossy().into_owned());
            (target.parent().unwrap_or(Path::new("/")).to_path_buf(), name)
        } else {
            (target, select)
        };
        if dir.to_string_lossy() == self.dir {
            self.as_mut().close_results();
            return select.map_or(-1, |n| self.row_of(&n));
        }
        let previous = self.dir.clone();
        match self.as_mut().load(&dir, select.as_deref()) {
            Ok(row) => {
                if dir != ops::trash_files() {
                    visits::record(&dir.to_string_lossy());
                }
                if !previous.is_empty() {
                    let mut r = self.as_mut().rust_mut();
                    r.history_back.push(previous);
                    r.history_forward.clear();
                    self.as_mut().update_history_flags();
                }
                row
            }
            Err(()) => -1,
        }
    }

    pub fn show_results(mut self: Pin<&mut Self>, title: &QString, paths: &QString) {
        if self.dir.is_empty() {
            return;
        }
        let found: Vec<fs::Entry> = paths_of(paths).iter().take(2000).filter_map(|p| fs::entry(Path::new(p))).collect();
        // Nom = chemin relatif au plus proche dossier commun (deux résultats peuvent porter le même nom)
        let mut common: Vec<&str> = Vec::new();
        for (i, e) in found.iter().enumerate() {
            let parts: Vec<&str> = e.path.split('/').collect();
            let dirs = &parts[..parts.len() - 1];
            if i == 0 {
                common = dirs.to_vec();
            } else {
                let n = common.iter().zip(dirs).take_while(|(a, b)| a == b).count();
                common.truncate(n);
            }
        }
        let shown = if common.len() > 1 { common.join("/") } else { "/".into() };
        let prefix = format!("{}/", shown.trim_end_matches('/'));
        let all: Vec<fs::Entry> = found
            .iter()
            .map(|e| fs::Entry { name: e.path.strip_prefix(&prefix).unwrap_or(&e.path).to_string(), ..e.clone() })
            .collect();
        let mut r = self.as_mut().rust_mut();
        r.all = all;
        r.filter = QString::default();
        r.results = if title.is_empty() { QString::from("…") } else { title.clone() };
        r.shown = QString::from(&shown);
        self.as_mut().drop_index();
        self.as_mut().rebuild();
        self.as_mut().changed();
    }

    pub fn close_results(mut self: Pin<&mut Self>) {
        if !self.results.is_empty() {
            let dir = QString::from(&self.dir);
            let mut r = self.as_mut().rust_mut();
            r.results = QString::default();
            r.shown = dir;
            self.as_mut().reload_rows();
            self.as_mut().changed();
        }
    }

    pub fn go(mut self: Pin<&mut Self>, path: &QString) -> i32 {
        let first = self.dir.is_empty();
        let base = if first { home() } else { self.dir.clone() };
        let target = normalize(&path.to_string(), &base);
        let row = self.as_mut().navigate(target, None);
        // Premier dossier introuvable: le dossier personnel, l'erreur reste affichée
        if row < 0 && first {
            let error = self.error.clone();
            let row = self.as_mut().navigate(PathBuf::from(home()), None);
            self.as_mut().rust_mut().error = error;
            self.as_mut().changed();
            return row;
        }
        row
    }

    pub fn activate(self: Pin<&mut Self>, row: i32) -> i32 {
        let Some(e) = self.row(row).cloned() else { return -1 };
        if e.is_dir {
            return self.navigate(PathBuf::from(&e.path), None);
        }
        // Une archive s'extrait ici (fili-archive), avec la demande de mot de passe de Casier si elle est protégée
        if archive_stem(&e.name).is_some() && !self.in_trash {
            self.start_archive(vec![e.path.clone()], None, String::new());
            return -1;
        }
        spawn("xdg-open", &e.path, Path::new(&self.dir));
        -1
    }

    // Dossier parent, le dossier quitté sélectionné (Finder)
    pub fn up(self: Pin<&mut Self>) -> i32 {
        let here = PathBuf::from(&self.dir);
        let Some(parent) = here.parent() else { return -1 };
        let name = here.file_name().map(|n| n.to_string_lossy().into_owned());
        self.navigate(parent.to_path_buf(), name)
    }

    // Retour et avance: le dossier quitté est sélectionné s'il est dans celui où l'on arrive
    fn jump(mut self: Pin<&mut Self>, forward: bool) -> i32 {
        let target = {
            let mut r = self.as_mut().rust_mut();
            let stack = if forward { &mut r.history_forward } else { &mut r.history_back };
            match stack.pop() {
                Some(t) => t,
                None => return -1,
            }
        };
        let here = self.dir.clone();
        let select = Path::new(&here)
            .strip_prefix(&target)
            .ok()
            .and_then(|rel| rel.components().next())
            .map(|c| c.as_os_str().to_string_lossy().into_owned());
        match self.as_mut().load(Path::new(&target), select.as_deref()) {
            Ok(row) => {
                let mut r = self.as_mut().rust_mut();
                if forward { r.history_back.push(here) } else { r.history_forward.push(here) }
                self.as_mut().update_history_flags();
                row
            }
            Err(()) => {
                // Dossier disparu: retiré de l'historique
                self.as_mut().update_history_flags();
                -1
            }
        }
    }

    pub fn back(self: Pin<&mut Self>) -> i32 {
        self.jump(false)
    }

    pub fn forward(self: Pin<&mut Self>) -> i32 {
        self.jump(true)
    }

    fn selected_name(&self, row: i32) -> Option<String> {
        self.row(row).map(|e| e.name.clone())
    }

    pub fn refresh(mut self: Pin<&mut Self>, row: i32) -> i32 {
        let name = self.selected_name(row);
        let dir = PathBuf::from(&self.dir);
        self.as_mut().load(&dir, name.as_deref()).unwrap_or(-1)
    }

    // Même colonne: ordre inversé; autre colonne: croissant
    pub fn sort_by(mut self: Pin<&mut Self>, key: i32, row: i32) -> i32 {
        let name = self.selected_name(row);
        let mut r = self.as_mut().rust_mut();
        if r.sort_key == key {
            r.sort_desc = !r.sort_desc;
        } else {
            r.sort_key = key;
            r.sort_desc = false;
        }
        self.as_mut().rebuild();
        self.as_mut().changed();
        name.map_or(0, |n| self.row_of(&n))
    }

    pub fn toggle_hidden(mut self: Pin<&mut Self>, row: i32) -> i32 {
        let name = self.selected_name(row);
        let mut r = self.as_mut().rust_mut();
        r.show_hidden = !r.show_hidden;
        if !self.filter.is_empty() {
            self.as_mut().start_index();
        }
        self.as_mut().rebuild();
        self.as_mut().changed();
        name.map_or(0, |n| self.row_of(&n))
    }

    pub fn set_filter(mut self: Pin<&mut Self>, text: &QString) {
        self.as_mut().rust_mut().filter = text.clone();
        if text.to_string().trim().is_empty() {
            self.as_mut().drop_index();
        } else if self.index.is_none() {
            self.as_mut().start_index();
        }
        self.as_mut().rebuild();
        self.as_mut().changed();
    }

    // Aperçu rapide (~/.local/bin/quicklook): relancé sur le même fichier, il se referme
    pub fn quicklook(&self, path: &QString) {
        spawn(&bin("quicklook"), &path.to_string(), Path::new(&self.dir));
    }

    // Saisie au clavier: première ligne dont le nom commence par prefix (sans casse ni accents)
    pub fn find(&self, prefix: &QString) -> i32 {
        let p = prefix.to_string();
        self.rows.iter().position(|e| fs::starts_with(&e.name, &p)).map_or(-1, |i| i as i32)
    }

    pub fn path_at(&self, row: i32) -> QString {
        QString::from(self.row(row).map_or("", |e| e.path.as_str()))
    }

    pub fn find_name(&self, name: &QString) -> i32 {
        let n = name.to_string();
        self.rows.iter().position(|e| e.name == n).map_or(-1, |i| i as i32)
    }

    pub fn clear_error(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().error = QString::default();
        self.as_mut().changed();
    }

    pub fn has(&self, path: &QString) -> bool {
        let p = path.to_string();
        self.rows.iter().any(|e| e.path == p)
    }

    pub fn matching(&self, pattern: &QString) -> QString {
        let p = pattern.to_string();
        QString::from(&self.rows.iter().filter(|e| fs::glob_match(&e.name, &p)).map(|e| e.path.as_str()).collect::<Vec<_>>().join("\n"))
    }

    pub fn same_kind(&self, paths: &QString) -> QString {
        let list = paths_of(paths);
        let kinds: Vec<&str> = self.rows.iter().filter(|e| list.contains(&e.path)).map(|e| e.kind.as_str()).collect();
        QString::from(&self.rows.iter().filter(|e| kinds.contains(&e.kind.as_str())).map(|e| e.path.as_str()).collect::<Vec<_>>().join("\n"))
    }

    pub fn describe(&self, paths: &QString) -> QString {
        let list = paths_of(paths);
        let entries: Vec<&fs::Entry> = list.iter().filter_map(|p| self.rows.iter().find(|e| e.path == *p)).collect();
        let text = match entries.as_slice() {
            [] => return QString::default(),
            [e] => {
                let what = if e.is_dir {
                    let n = std::fs::read_dir(&e.path).map_or(0, |d| d.count());
                    fs::count(n)
                } else {
                    fs::size(e.size)
                };
                i18n::trf("%1 — %2", &[&e.name, &what])
            }
            many => {
                let total: u64 = many.iter().filter(|e| !e.is_dir).map(|e| e.size).sum();
                let head = i18n::trf("%1 of %2 selected", &[&many.len(), &self.rows.len()]);
                if total > 0 { i18n::trf("%1, %2", &[&head, &fs::size(total)]) } else { head }
            }
        };
        QString::from(&text)
    }

    pub fn path_is_dir(&self, path: &QString) -> bool {
        let p = path.to_string();
        self.rows.iter().any(|e| e.path == p && e.is_dir)
    }

    pub fn uris(&self, paths: &QString) -> QString {
        let list: Vec<String> = paths_of(paths).iter().map(|p| format!("file://{}", ops::encode(p))).collect();
        QString::from(&list.join("\r\n"))
    }

    fn fail(mut self: Pin<&mut Self>, msg: String) {
        self.as_mut().rust_mut().error = QString::from(&msg);
        self.as_mut().changed();
    }

    fn push_undo(mut self: Pin<&mut Self>, u: Undo) {
        let mut r = self.as_mut().rust_mut();
        r.undo.push(u);
        if r.undo.len() > 20 {
            r.undo.remove(0);
        }
        r.can_undo = true;
    }

    // Après une opération synchrone: relire le dossier; rend la ligne portant name, sinon la plus proche de near
    fn settle(mut self: Pin<&mut Self>, name: Option<&str>, near: i32) -> i32 {
        self.as_mut().reload_rows();
        self.as_mut().changed();
        match name.map(|n| self.find_name(&QString::from(n))) {
            Some(r) if r >= 0 => r,
            _ if self.rows.is_empty() => -1,
            _ => near.clamp(0, self.rows.len() as i32 - 1),
        }
    }

    // Copier et couper posent aussi les fichiers dans le presse-papiers Wayland (text/uri-list): les autres
    // applications peuvent les coller. Le « couper » n'existe que chez nous, tant que le presse-papiers n'a pas changé.
    pub fn copy(mut self: Pin<&mut Self>, paths: &QString) {
        let list = paths_of(paths);
        clip_set(&list);
        let mut r = self.as_mut().rust_mut();
        r.can_paste = !list.is_empty();
        r.clipboard = list;
        r.clip_cut = false;
        self.as_mut().changed();
    }

    pub fn cut(mut self: Pin<&mut Self>, paths: &QString) {
        if self.in_trash {
            return self.fail(i18n::tr("Items in the Trash cannot be cut").into());
        }
        self.as_mut().copy(paths);
        self.as_mut().rust_mut().clip_cut = true;
    }

    pub fn refresh_paste(mut self: Pin<&mut Self>) {
        let has = clip_has_files() || !self.clipboard.is_empty();
        if self.can_paste != has {
            self.as_mut().rust_mut().can_paste = has;
            self.as_mut().changed();
        }
    }

    pub fn paste(mut self: Pin<&mut Self>) {
        if !self.results.is_empty() {
            return self.fail(i18n::tr("Close the results first").into());
        }
        if self.in_trash {
            return self.fail(i18n::tr("Nothing can be pasted into the Trash").into());
        }
        // Le presse-papiers système d'abord (copié ici ou dans une autre application), sinon la copie interne
        let (items, cut) = match clip_get() {
            Some((items, cut)) => {
                let ours = items == self.clipboard;
                (items, cut.unwrap_or(ours && self.clip_cut))
            }
            None => (self.clipboard.clone(), self.clip_cut),
        };
        let dir = self.dir.clone();
        let mode = if cut { Mode::Move } else { Mode::Copy };
        if cut {
            let mut r = self.as_mut().rust_mut();
            r.clipboard.clear();
            r.can_paste = false;
            clip_clear();
        }
        self.start_job(items, dir, mode);
    }

    // Glisser-déposer: copie si demandé, sinon déplacement sur le même volume et copie sur un autre. Sur la
    // corbeille: mise à la corbeille. Depuis la corbeille: copie.
    pub fn drop_items(mut self: Pin<&mut Self>, urls: &QString, dest: &QString, copy: bool) {
        let items: Vec<String> =
            paths_of(&QString::from(&urls.to_string().replace('\r', ""))).iter().map(|u| normalize(u, "/").to_string_lossy().into_owned()).collect();
        let dest = normalize(&dest.to_string(), "/");
        if dest == ops::trash_files() {
            let list = QString::from(&items.join("\n"));
            self.as_mut().trash(&list, 0);
            return;
        }
        if !dest.is_dir() {
            return;
        }
        let mode = if copy || self.in_trash { Mode::Copy } else { Mode::Auto };
        self.start_job(items, dest.to_string_lossy().into_owned(), mode);
    }

    // Copie ou déplacement en arrière-plan: avancement dans la barre d'état, annulation (Échap) qui retire la copie
    // en cours; ce qui est déjà fini reste (et Ctrl+Z le défait).
    // Un nom déjà pris ailleurs que dans le dossier source: on demande (remplacer, garder les deux, ignorer).
    fn start_job(mut self: Pin<&mut Self>, items: Vec<String>, dest: String, mode: Mode) {
        if items.is_empty() {
            return;
        }
        let clashes: Vec<String> = items
            .iter()
            .filter(|i| {
                let src = Path::new(i);
                src.parent() != Some(Path::new(&dest)) && src.file_name().is_some_and(|n| ops::exists(&Path::new(&dest).join(n)))
            })
            .map(|i| base(i))
            .collect();
        if clashes.is_empty() {
            return self.run_job(items, dest, mode, Policy::Both);
        }
        self.as_mut().rust_mut().pending = Some((items, dest.clone(), mode));
        self.conflict(&QString::from(&clashes.join("\n")), &QString::from(&dest));
    }

    pub fn resolve_conflict(mut self: Pin<&mut Self>, choice: &QString) {
        let Some((items, dest, mode)) = self.as_mut().rust_mut().pending.take() else { return };
        match choice.to_string().as_str() {
            "replace" => self.run_job(items, dest, mode, Policy::Replace),
            "both" => self.run_job(items, dest, mode, Policy::Both),
            "skip" => self.run_job(items, dest, mode, Policy::Skip),
            _ => {}
        }
    }

    fn run_job(mut self: Pin<&mut Self>, items: Vec<String>, dest: String, mode: Mode, policy: Policy) {
        if items.is_empty() {
            return;
        }
        if self.busy {
            self.as_mut().rust_mut().queue.push((items, dest, mode, policy));
            self.as_mut().changed();
            return;
        }
        let label = job_label(&items, mode);
        let cancel = Arc::new(AtomicBool::new(false));
        {
            let mut r = self.as_mut().rust_mut();
            r.busy = true;
            r.progress = -1.0;
            r.job = QString::from(&label);
            r.error = QString::default();
            r.cancel = cancel.clone();
        }
        self.as_mut().changed();
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let dest = PathBuf::from(dest);
            let total = items.iter().map(|p| ops::tree_size(Path::new(p))).sum::<u64>().max(1);
            let (mut done, mut last) = (0u64, Instant::now());
            let (mut moved, mut created, mut replaced, mut error) = (Vec::new(), Vec::new(), Vec::new(), None);
            for item in &items {
                if cancel.load(Ordering::Relaxed) {
                    break;
                }
                let src = Path::new(item);
                let parent = src.parent().unwrap_or(Path::new("/"));
                let as_move = match mode {
                    Mode::Copy => false,
                    Mode::Move => true,
                    Mode::Auto => ops::same_volume(parent, &dest),
                };
                if as_move && parent == dest {
                    continue;
                }
                if dest.starts_with(src) {
                    error = Some(i18n::trf("Cannot put “%1” inside itself", &[&base(item)]));
                    break;
                }
                let mut progress = |n: u64| {
                    done += n;
                    if last.elapsed().as_millis() > 100 {
                        last = Instant::now();
                        let f = done as f64 / total as f64;
                        let _ = qt.queue(move |mut m| {
                            m.as_mut().rust_mut().progress = f;
                            m.as_mut().changed();
                        });
                    }
                    !cancel.load(Ordering::Relaxed)
                };
                // Nom déjà pris: ignorer, ou mettre l'ancien à la corbeille (Ctrl+Z le rend), sinon un nom libre
                let target = dest.join(src.file_name().unwrap_or_default());
                let clash = target != src && ops::exists(&target);
                if clash && policy == Policy::Skip {
                    continue;
                }
                if clash && policy == Policy::Replace {
                    match ops::trash(&target) {
                        Ok(t) => replaced.push(t),
                        Err(e) => {
                            error = Some(i18n::trf("Cannot replace “%1”: %2", &[&base(item), &fs::error(&e)]));
                            break;
                        }
                    }
                }
                let r = match (as_move, clash && policy == Policy::Replace) {
                    (true, true) => ops::move_to(src, target, &mut progress),
                    (false, true) => ops::copy_to(src, target, &mut progress),
                    (true, false) => ops::move_into(src, &dest, &mut progress),
                    (false, false) => ops::copy_into(src, &dest, &mut progress),
                };
                match r {
                    Ok(dst) if as_move => moved.push((src.to_path_buf(), dst)),
                    Ok(dst) => created.push(dst),
                    Err(e) if ops::is_cancel(&e) => break,
                    Err(e) => {
                        let (name, why) = (base(item), fs::error(&e));
                        error = Some(if as_move {
                            i18n::trf("Cannot move “%1”: %2", &[&name, &why])
                        } else {
                            i18n::trf("Cannot copy “%1”: %2", &[&name, &why])
                        });
                        break;
                    }
                }
            }
            let _ = qt.queue(move |mut m| {
                {
                    let mut r = m.as_mut().rust_mut();
                    r.busy = false;
                    r.progress = 0.0;
                    r.job = QString::default();
                    r.error = QString::from(&error.unwrap_or_default());
                }
                // Les remplacés d'abord: Ctrl+Z défait la copie, un second Ctrl+Z rend l'ancien
                if !replaced.is_empty() {
                    m.as_mut().push_undo(Undo::Trash(replaced));
                }
                if !moved.is_empty() {
                    m.as_mut().push_undo(Undo::Moved(moved));
                }
                if !created.is_empty() {
                    m.as_mut().push_undo(Undo::Created(created));
                }
                m.as_mut().reload_rows();
                m.as_mut().changed();
                m.as_mut().reloaded();
                m.as_mut().next_job();
            });
        });
    }

    // La file d'attente avance: la copie ou le déplacement suivant démarre dès que le précédent est fini
    fn next_job(mut self: Pin<&mut Self>) {
        if self.busy || self.queue.is_empty() {
            return;
        }
        let (items, dest, mode, policy) = self.as_mut().rust_mut().queue.remove(0);
        self.run_job(items, dest, mode, policy);
    }

    // Tâches pour la fenêtre des tâches, en JSON: la tâche en cours d'abord, puis la file [{label, progress}]
    // Nouveaux noms du lot (dans l'ordre d'affichage), avec leur validité: nom valide, pas de doublon dans le lot, pas
    // d'élément existant hors du lot qui porte déjà ce nom
    fn bulk_plan(&self, paths: &QString, mode: &str, a: &str, b: &str, start: i32) -> Vec<(PathBuf, String, bool)> {
        let mut list = paths_of(paths);
        let order = |p: &String| self.rows.iter().position(|e| &e.path == p).unwrap_or(usize::MAX);
        list.sort_by_key(order);
        let planned: Vec<(PathBuf, String)> = list
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let path = PathBuf::from(p);
                let name = base(p);
                let new = ops::bulk_name(&name, path.is_dir(), mode, a, b, i, start as i64);
                (path, new)
            })
            .collect();
        planned
            .iter()
            .map(|(path, new)| {
                let target = path.with_file_name(new);
                let same_count = planned.iter().filter(|(q, n)| q.with_file_name(n) == target).count();
                let taken_outside = target != *path && ops::exists(&target) && !planned.iter().any(|(q, _)| *q == target);
                (path.clone(), new.clone(), ops::valid_name(new) && same_count == 1 && !taken_outside)
            })
            .collect()
    }

    pub fn bulk_preview(&self, paths: &QString, mode: &QString, a: &QString, b: &QString, start: i32) -> QString {
        let plan = self.bulk_plan(paths, &mode.to_string(), &a.to_string(), &b.to_string(), start);
        let body: Vec<String> = plan
            .iter()
            .map(|(p, new, ok)| format!("{{\"from\":{},\"to\":{},\"ok\":{ok},\"same\":{}}}", json(&base(&p.to_string_lossy())), json(new), base(&p.to_string_lossy()) == *new))
            .collect();
        QString::from(&format!("[{}]", body.join(",")))
    }

    pub fn bulk_apply(mut self: Pin<&mut Self>, paths: &QString, mode: &QString, a: &QString, b: &QString, start: i32) -> bool {
        let plan = self.bulk_plan(paths, &mode.to_string(), &a.to_string(), &b.to_string(), start);
        if plan.iter().any(|(_, _, ok)| !ok) {
            self.as_mut().fail(i18n::tr("Some new names are not valid or already exist").into());
            return false;
        }
        let pairs: Vec<(PathBuf, PathBuf)> = plan.into_iter().filter(|(p, n, _)| base(&p.to_string_lossy()) != *n).map(|(p, n, _)| (p.clone(), p.with_file_name(n))).collect();
        if pairs.is_empty() {
            return true;
        }
        match ops::rename_many(&pairs) {
            Ok(()) => {
                self.as_mut().push_undo(Undo::Renames(pairs));
                self.as_mut().settle(None, 0);
                self.as_mut().reloaded();
                true
            }
            Err(e) => {
                self.as_mut().fail(i18n::trf("Cannot rename: %1", &[&fs::error(&e)]));
                false
            }
        }
    }

    pub fn tasks(&self) -> QString {
        let mut list = Vec::new();
        if self.busy {
            list.push(format!("{{\"label\":{},\"progress\":{}}}", json(&self.job.to_string()), self.progress));
        }
        for (items, _, mode, _) in &self.queue {
            list.push(format!("{{\"label\":{},\"progress\":-2}}", json(&job_label(items, *mode))));
        }
        QString::from(&format!("[{}]", list.join(",")))
    }

    // Annule la tâche i: la première arrête l'opération en cours, les autres sortent de la file
    pub fn cancel_task(mut self: Pin<&mut Self>, index: i32) {
        if self.busy && index == 0 {
            self.cancel.store(true, Ordering::Relaxed);
        } else {
            let i = index - i32::from(self.busy);
            if i >= 0 && (i as usize) < self.queue.len() {
                self.as_mut().rust_mut().queue.remove(i as usize);
            }
        }
        self.as_mut().changed();
    }

    pub fn duplicate(self: Pin<&mut Self>, paths: &QString) {
        if self.in_trash {
            return;
        }
        let dir = self.dir.clone();
        self.run_job(paths_of(paths), dir, Mode::Copy, Policy::Both);
    }

    pub fn terminal(&self, path: &QString) {
        let p = path.to_string();
        let dir = if !p.is_empty() && Path::new(&p).is_dir() { p } else { self.dir.clone() };
        let child = Command::new("foot").current_dir(&dir).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).process_group(0).spawn();
        if let Ok(mut child) = child {
            std::thread::spawn(move || child.wait());
        }
    }

    pub fn info(&self, paths: &QString) -> QString {
        use std::os::unix::fs::MetadataExt;
        let list = paths_of(paths);
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64);
        let mut rows: Vec<(String, String)> = Vec::new();
        match list.as_slice() {
            [] => {}
            [one] => {
                let p = Path::new(one);
                let Ok(m) = std::fs::symlink_metadata(p) else { return QString::from("[]") };
                let target = std::fs::metadata(p).unwrap_or_else(|_| m.clone());
                let is_dir = target.is_dir();
                rows.push((i18n::tr("Name").into(), base(one)));
                rows.push((i18n::tr("Kind").into(), if m.is_symlink() { i18n::tr("Alias").into() } else { icons::kind(&base(one), is_dir) }));
                rows.push((i18n::tr("Size").into(), if is_dir { "…".into() } else { fs::size(m.len()) }));
                if m.is_symlink()
                    && let Ok(t) = std::fs::read_link(p)
                {
                    rows.push((i18n::tr("Points to").into(), t.to_string_lossy().into_owned()));
                }
                rows.push((i18n::tr("Where").into(), p.parent().map_or("/".into(), |d| d.to_string_lossy().into_owned())));
                if let Ok(t) = m.created() {
                    rows.push((i18n::tr("Created").into(), fs::date(t.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64), now)));
                }
                rows.push((i18n::tr("Modified").into(), fs::date(m.mtime(), now)));
                rows.push((i18n::tr("Permissions").into(), format!("{} ({:o})", fs::permissions(m.mode()), m.mode() & 0o777)));
                rows.push((i18n::tr("Owner").into(), fs::owner(m.uid())));
                if target.is_file() {
                    rows.push(("SHA-256".into(), "…".into()));
                }
            }
            many => {
                rows.push((i18n::tr("Name").into(), fs::count(many.len())));
                rows.push((i18n::tr("Size").into(), "…".into()));
                rows.push((i18n::tr("Where").into(), Path::new(&many[0]).parent().map_or("/".into(), |d| d.to_string_lossy().into_owned())));
            }
        }
        let body: Vec<String> = rows.iter().map(|(k, v)| format!("[{},{}]", json(k), json(v))).collect();
        QString::from(&format!("[{}]", body.join(",")))
    }

    pub fn start_info_hash(self: Pin<&mut Self>, path: &QString) {
        let path = path.to_string();
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let out = Command::new("sha256sum").arg("--").arg(&path).stdin(Stdio::null()).stderr(Stdio::null()).output();
            let text = out
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| String::from_utf8_lossy(&o.stdout).split_whitespace().next().map(String::from))
                .unwrap_or_else(|| "—".into());
            let _ = qt.queue(move |m| m.info_hash(&QString::from(&text)));
        });
    }

    pub fn perm_mode(&self, path: &QString) -> i32 {
        use std::os::unix::fs::MetadataExt;
        match std::fs::symlink_metadata(path.to_string()) {
            Ok(m) if !m.is_symlink() && m.uid() == unsafe { libc::geteuid() } => (m.mode() & 0o777) as i32,
            _ => -1,
        }
    }

    pub fn set_perm(mut self: Pin<&mut Self>, path: &QString, mode: i32) -> QString {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        let p = PathBuf::from(path.to_string());
        let result = std::fs::symlink_metadata(&p).and_then(|m| {
            let old = m.mode() & 0o7777;
            let new = (old & !0o777) | (mode as u32 & 0o777);
            std::fs::set_permissions(&p, std::fs::Permissions::from_mode(new)).map(|()| (old, new))
        });
        match result {
            Ok((old, new)) => {
                self.as_mut().push_undo(Undo::Mode(p, old));
                QString::from(&format!("{} ({:o})", fs::permissions(new), new & 0o777))
            }
            Err(e) => {
                self.as_mut().fail(i18n::trf("Cannot change permissions: %1", &[&fs::error(&e)]));
                QString::default()
            }
        }
    }

    // Taille totale (dossiers compris) calculée à part, rendue par le signal infoSize
    pub fn start_info_size(self: Pin<&mut Self>, paths: &QString) {
        let list = paths_of(paths);
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let total: u64 = list.iter().map(|p| ops::tree_size(Path::new(p))).sum();
            let text = if total < 1024 { fs::size(total) } else { format!("{} ({})", fs::size(total), i18n::trf("%1 B", &[&total])) };
            let _ = qt.queue(move |m| m.info_size(&QString::from(&text)));
        });
    }

    pub fn open_with_list(&self, path: &QString) -> QString {
        let path = path.to_string();
        let list = icons::mime_type(&base(&path)).map(|m| apps::for_mime(&m)).unwrap_or_default();
        let body: Vec<String> = list.iter().map(|a| format!("{{\"id\":{},\"name\":{}}}", json(&a.id), json(&a.name))).collect();
        QString::from(&format!("[{}]", body.join(",")))
    }

    pub fn open_with(&self, id: &QString, path: &QString) {
        let (id, path) = (id.to_string(), path.to_string());
        let Some(app) = apps::desktop(&id) else { return };
        let argv = apps::command(&app, &path);
        let Some((program, args)) = argv.split_first() else { return };
        spawn_detached(Command::new(program).args(args).current_dir(&self.dir));
    }

    pub fn all_archives(&self, paths: &QString) -> bool {
        let list = paths_of(paths);
        !list.is_empty() && list.iter().all(|p| archive_stem(&base(p)).is_some())
    }

    pub fn extract(self: Pin<&mut Self>, paths: &QString, password: &QString) {
        self.start_archive(paths_of(paths), None, password.to_string());
    }

    pub fn archive_name(&self, paths: &QString) -> QString {
        let items = paths_of(paths);
        QString::from(&match items.as_slice() {
            [one] => {
                let n = base(one);
                match n.rsplit_once('.') {
                    Some((s, _)) if !s.is_empty() && !Path::new(one).is_dir() => s.to_string(),
                    _ => n,
                }
            }
            _ => i18n::tr("Archive").to_string(),
        })
    }

    pub fn compress(self: Pin<&mut Self>, paths: &QString, name: &QString, format: &QString, level: &QString, skip_hidden: bool, password: &QString) {
        let format = match format.to_string().as_str() {
            f @ ("zip" | "7z" | "tar.gz" | "tar.xz" | "tar.zst") => f.to_string(),
            _ => "zip".into(),
        };
        let level = match level.to_string().as_str() {
            l @ ("fast" | "best") => l.to_string(),
            _ => "normal".into(),
        };
        // Pas de séparateur de chemin dans le nom
        let name = name.to_string().replace('/', "-").trim().to_string();
        let opts = ArchiveOpts { name, format, level, skip_hidden };
        // Seuls zip et 7z chiffrent
        let password = if matches!(opts.format.as_str(), "zip" | "7z") { password.to_string() } else { String::new() };
        self.start_archive(paths_of(paths), Some(opts), password);
    }

    // Compression ou extraction en arrière-plan par fili-archive (-q: pas de transaction ni de notification), annulable;
    // le résultat se défait (corbeille). Un mot de passe passe par l'entrée standard du programme, jamais en argument.
    fn start_archive(mut self: Pin<&mut Self>, items: Vec<String>, opts: Option<ArchiveOpts>, password: String) {
        let compress = opts.is_some();
        if items.is_empty() {
            return;
        }
        if self.busy {
            return self.fail(i18n::tr("Another operation is in progress").into());
        }
        if self.in_trash {
            return self.fail(i18n::tr("Nothing can be created in the Trash").into());
        }
        let what = if items.len() == 1 { base(&items[0]) } else { fs::count(items.len()) };
        let label = if compress { i18n::trf("Compressing %1", &[&what]) } else { i18n::trf("Extracting %1", &[&what]) };
        let cancel = Arc::new(AtomicBool::new(false));
        {
            let mut r = self.as_mut().rust_mut();
            r.busy = true;
            r.progress = -1.0;
            r.job = QString::from(&label);
            r.error = QString::default();
            r.cancel = cancel.clone();
        }
        self.as_mut().changed();
        let dir = PathBuf::from(&self.dir);
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let msg = |name: &str, why: &str| {
                if compress {
                    i18n::trf("Cannot compress “%1”: %2", &[&name, &why])
                } else {
                    i18n::trf("Cannot extract “%1”: %2", &[&name, &why])
                }
            };
            let (mut created, mut error): (Vec<PathBuf>, Option<String>) = (Vec::new(), None);
            // Archives à rouvrir avec un mot de passe: (celles qui restent, mauvais mot de passe)
            let mut ask: Option<(Vec<String>, bool)> = None;
            // Jobs: (arguments de fili-archive, nom pour les erreurs, archives restantes à partir de celle-ci)
            let mut jobs: Vec<(Vec<String>, String, Vec<String>)> = Vec::new();
            let base_args = |sub: &str| -> Vec<String> {
                let mut a: Vec<String> = vec![sub.into(), "-q".into()];
                if !password.is_empty() {
                    a.push("--password-stdin".into());
                }
                a
            };
            if let Some(o) = &opts {
                let name = if o.name.is_empty() || o.name.starts_with('.') { i18n::tr("Archive").to_string() } else { o.name.clone() };
                let out = ops::unique(&dir, &format!("{name}.{}", o.format), "", false);
                let mut args = base_args("pack");
                // Pas de -f: avec -o, fili-archive déduit le format de l'extension (avec -f il ajouterait la sienne: x.zip.zip)
                args.extend(["-l", &o.level].map(String::from));
                if o.skip_hidden {
                    args.push("--no-hidden".into());
                }
                args.extend(["-o".into(), out.to_string_lossy().into_owned(), "--".into()]);
                args.extend(items.iter().cloned());
                jobs.push((args, name, Vec::new()));
            } else {
                for (i, item) in items.iter().enumerate() {
                    let name = base(item);
                    if archive_stem(&name).is_none() {
                        continue;
                    }
                    let mut args = base_args("unpack");
                    args.extend(["-o".into(), dir.to_string_lossy().into_owned(), "--".into(), item.clone()]);
                    jobs.push((args, name, items[i..].to_vec()));
                }
            }
            for (args, name, remaining) in jobs {
                let child = Command::new("fili-archive")
                    .args(&args)
                    .current_dir(&dir)
                    .stdin(if password.is_empty() { Stdio::null() } else { Stdio::piped() })
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn();
                let mut child = match child {
                    Ok(c) => c,
                    Err(e) => {
                        error = Some(msg(&name, &fs::error(&e)));
                        break;
                    }
                };
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = std::io::Write::write_all(&mut stdin, format!("{password}\n").as_bytes());
                }
                let ok = loop {
                    if cancel.load(Ordering::Relaxed) {
                        let _ = child.kill();
                        let _ = child.wait();
                        // le programme tué laisse son fichier ou dossier de travail caché (« .nom.PID.part… »)
                        let marker = format!(".{}.part", child.id());
                        for e in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
                            let n = e.file_name().to_string_lossy().into_owned();
                            if n.starts_with('.') && n.contains(&marker) {
                                let _ = ops::remove_tree(&e.path());
                            }
                        }
                        break None;
                    }
                    match child.try_wait() {
                        Ok(Some(status)) => break Some(status.success()),
                        Ok(None) => std::thread::sleep(std::time::Duration::from_millis(100)),
                        Err(_) => break Some(false),
                    }
                };
                let (mut out, mut err) = (String::new(), String::new());
                if let Some(mut o) = child.stdout.take() {
                    let _ = std::io::Read::read_to_string(&mut o, &mut out);
                }
                if let Some(mut e) = child.stderr.take() {
                    let _ = std::io::Read::read_to_string(&mut e, &mut err);
                }
                match ok {
                    Some(true) => created.extend(out.lines().filter(|l| !l.is_empty()).map(PathBuf::from)),
                    Some(false) => {
                        // « fili-archive: cannot extract “x”: la raison »
                        let why = err.lines().last().map(|l| l.rsplit(": ").next().unwrap_or("").trim().to_string());
                        let why = why.filter(|w| !w.is_empty()).unwrap_or_else(|| i18n::tr("the tool reported an error").to_string());
                        if !compress && (why.contains("protected by a password") || why == "wrong password") {
                            ask = Some((remaining, why == "wrong password"));
                        } else {
                            error = Some(msg(&name, &why));
                        }
                        break;
                    }
                    None => break,
                }
            }
            let _ = qt.queue(move |mut m| {
                {
                    let mut r = m.as_mut().rust_mut();
                    r.busy = false;
                    r.progress = 0.0;
                    r.job = QString::default();
                    r.error = QString::from(&error.unwrap_or_default());
                }
                if !created.is_empty() {
                    m.as_mut().push_undo(Undo::Created(created));
                }
                m.as_mut().reload_rows();
                m.as_mut().changed();
                m.as_mut().reloaded();
                m.as_mut().next_job();
                if let Some((rest, wrong)) = ask {
                    m.as_mut().password_needed(&QString::from(&rest.join("\n")), wrong);
                }
            });
        });
    }

    pub fn cancel(self: Pin<&mut Self>) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    pub fn trash(mut self: Pin<&mut Self>, paths: &QString, row: i32) -> i32 {
        if self.in_trash {
            self.as_mut().fail(i18n::tr("These items are already in the Trash").into());
            return -1;
        }
        self.as_mut().rust_mut().error = QString::default();
        let (mut done, mut error) = (Vec::new(), None);
        for p in paths_of(paths) {
            match ops::trash(Path::new(&p)) {
                Ok(t) => done.push(t),
                Err(e) => {
                    error = Some(i18n::trf("Cannot move “%1” to the Trash: %2", &[&base(&p), &fs::error(&e)]));
                    break;
                }
            }
        }
        if !done.is_empty() {
            self.as_mut().push_undo(Undo::Trash(done));
        }
        let row = self.as_mut().settle(None, row);
        if let Some(msg) = error {
            self.as_mut().fail(msg);
        }
        row
    }

    pub fn put_back(mut self: Pin<&mut Self>, paths: &QString, row: i32) -> i32 {
        self.as_mut().rust_mut().error = QString::default();
        let (mut done, mut error) = (Vec::new(), None);
        for p in paths_of(paths) {
            match ops::restore(Path::new(&p)) {
                Ok(dst) => done.push(dst),
                Err(e) => {
                    error = Some(i18n::trf("Cannot put back “%1”: %2", &[&base(&p), &fs::error(&e)]));
                    break;
                }
            }
        }
        if !done.is_empty() {
            self.as_mut().push_undo(Undo::Created(done));
        }
        let row = self.as_mut().settle(None, row);
        if let Some(msg) = error {
            self.as_mut().fail(msg);
        }
        row
    }

    pub fn empty_trash(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().error = QString::default();
        let result = ops::empty_trash();
        {
            // Plus rien à défaire dans la corbeille
            let mut r = self.as_mut().rust_mut();
            r.undo.retain(|u| !matches!(u, Undo::Trash(_)));
            r.can_undo = !r.undo.is_empty();
        }
        self.as_mut().settle(None, 0);
        if let Err(e) = result {
            self.as_mut().fail(i18n::trf("Cannot empty the Trash: %1", &[&fs::error(&e)]));
        }
    }

    pub fn rename(mut self: Pin<&mut Self>, path: &QString, name: &QString) -> i32 {
        let (path, name) = (path.to_string(), name.to_string());
        self.as_mut().rust_mut().error = QString::default();
        if self.in_trash {
            self.as_mut().fail(i18n::tr("Items in the Trash cannot be renamed").into());
            return -1;
        }
        match ops::rename(Path::new(&path), &name) {
            Ok(dst) => {
                if dst != Path::new(&path) {
                    self.as_mut().push_undo(Undo::Rename(PathBuf::from(&path), dst.clone()));
                }
                if !self.results.is_empty() {
                    // Le résultat suit son nouveau nom (le dossier d'origine du chemin ne change pas)
                    let dst = dst.to_string_lossy().into_owned();
                    for e in self.as_mut().rust_mut().all.iter_mut().filter(|e| e.path == path) {
                        e.name = format!("{}{}", e.name.strip_suffix(&base(&path)).unwrap_or(""), name);
                        e.path = dst.clone();
                    }
                    self.as_mut().reload_rows();
                    self.as_mut().changed();
                    return self.rows.iter().position(|e| e.path == dst).map_or(-1, |i| i as i32);
                }
                self.as_mut().settle(Some(&name), 0)
            }
            Err(e) => {
                let msg = match e.kind() {
                    std::io::ErrorKind::AlreadyExists => i18n::trf("“%1” already exists", &[&name]),
                    std::io::ErrorKind::InvalidInput => i18n::tr("This name is not valid").into(),
                    _ => i18n::trf("Cannot rename “%1”: %2", &[&base(&path), &fs::error(&e)]),
                };
                self.as_mut().fail(msg);
                -1
            }
        }
    }

    pub fn new_folder(mut self: Pin<&mut Self>) -> i32 {
        self.as_mut().rust_mut().error = QString::default();
        if !self.results.is_empty() {
            self.as_mut().fail(i18n::tr("Close the results first").into());
            return -1;
        }
        if self.in_trash {
            self.as_mut().fail(i18n::tr("Nothing can be created in the Trash").into());
            return -1;
        }
        match ops::new_folder(Path::new(&self.dir)) {
            Ok(p) => {
                self.as_mut().push_undo(Undo::Created(vec![p.clone()]));
                let name = base(&p.to_string_lossy());
                self.as_mut().settle(Some(&name), 0)
            }
            Err(e) => {
                let dir = base(&self.dir);
                self.as_mut().fail(i18n::trf("Cannot create a folder in “%1”: %2", &[&dir, &fs::error(&e)]));
                -1
            }
        }
    }

    pub fn undo(mut self: Pin<&mut Self>) {
        let Some(u) = self.as_mut().rust_mut().undo.pop() else { return };
        self.as_mut().rust_mut().can_undo = !self.undo.is_empty();
        self.as_mut().rust_mut().error = QString::default();
        let result = match u {
            Undo::Rename(old, new) => {
                if ops::exists(&old) { Err(std::io::Error::from(std::io::ErrorKind::AlreadyExists)) } else { std::fs::rename(&new, &old) }
            }
            Undo::Trash(items) => items.iter().rev().try_for_each(|t| ops::restore(&t.trashed).map(|_| ())),
            Undo::Created(paths) => paths.iter().try_for_each(|p| ops::trash(p).map(|_| ())),
            Undo::Moved(pairs) => pairs.iter().rev().try_for_each(|(src, dst)| {
                ops::move_into(dst, src.parent().unwrap_or(Path::new("/")), &mut |_| true).map(|_| ())
            }),
            Undo::Renames(pairs) => {
                let back: Vec<(PathBuf, PathBuf)> = pairs.into_iter().map(|(old, new)| (new, old)).collect();
                ops::rename_many(&back)
            }
            Undo::Mode(p, mode) => {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&p, std::fs::Permissions::from_mode(mode))
            }
        };
        self.as_mut().settle(None, 0);
        self.as_mut().reloaded();
        if let Err(e) = result {
            self.as_mut().fail(i18n::trf("Cannot undo: %1", &[&fs::error(&e)]));
        }
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let Some(e) = self.row(index.row()) else { return QVariant::default() };
        match role {
            ROLE_NAME => QVariant::from(&QString::from(&e.name)),
            ROLE_PATH => QVariant::from(&QString::from(&e.path)),
            ROLE_DIR => QVariant::from(&e.is_dir),
            ROLE_SIZE => QVariant::from(&QString::from(&if e.is_dir { String::from("—") } else { fs::size(e.size) })),
            ROLE_DATE => QVariant::from(&QString::from(&fs::date(e.mtime, self.now))),
            ROLE_ICON => QVariant::from(&QString::from(&if e.is_dir {
                icons::folder()
            } else {
                icons::file(e)
            })),
            ROLE_HIDDEN => QVariant::from(&e.hidden()),
            ROLE_THUMB => QVariant::from(&QString::from(&if icons::is_pdf(e) { format!("file://{}", thumb_path(e).display()) } else { String::new() })),
            ROLE_THUMB_READY => QVariant::from(&(icons::is_pdf(e) && thumb_exists(e))),
            ROLE_IMAGE => QVariant::from(&icons::is_image(e)),
            ROLE_KIND => QVariant::from(&QString::from(&e.kind)),
            ROLE_CREATED => QVariant::from(&QString::from(&fs::date(e.created, self.now))),
            ROLE_PERMS => QVariant::from(&QString::from(&fs::permissions(e.mode))),
            ROLE_OWNER => QVariant::from(&QString::from(&fs::owner(e.uid))),
            _ => QVariant::default(),
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut roles = QHash::<QHashPair_i32_QByteArray>::default();
        for (role, name) in [
            (ROLE_NAME, "name"),
            (ROLE_PATH, "path"),
            (ROLE_DIR, "isDir"),
            (ROLE_SIZE, "size"),
            (ROLE_DATE, "modified"),
            (ROLE_ICON, "icon"),
            (ROLE_HIDDEN, "hidden"),
            (ROLE_THUMB, "thumb"),
            (ROLE_THUMB_READY, "thumbReady"),
            (ROLE_IMAGE, "isImage"),
            (ROLE_KIND, "kind"),
            (ROLE_CREATED, "created"),
            (ROLE_PERMS, "perms"),
            (ROLE_OWNER, "owner"),
        ] {
            roles.insert(role, QByteArray::from(name));
        }
        roles
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.rows.len() as i32
    }
}

impl cxx_qt::Initialize for FolderModel {
    fn initialize(mut self: Pin<&mut Self>) {
        let qt = self.qt_thread();
        let cfg = crate::config::load();
        {
            let mut m = self.as_mut();
            let r = m.as_mut().rust_mut().get_mut();
            r.show_hidden = cfg.show_hidden;
            r.sort_key = cfg.sort;
            r.sort_desc = cfg.sort_desc;
        }
        self.as_mut().rust_mut().pick_filter = -1;
        self.as_mut().rust_mut().watcher = Watcher::new(move || {
            let _ = qt.queue(|m| m.live_reload());
        });
    }
}

#[derive(Default)]
pub struct FiliRust {
    light: bool,
    accent: QString,
    key_nav: bool,
}

fn view_file() -> PathBuf {
    let state = env::var("XDG_STATE_HOME").unwrap_or_else(|_| format!("{}/.local/state", home()));
    PathBuf::from(state).join("filigrane/fili-file-view")
}

fn ui_file() -> PathBuf {
    let state = env::var("XDG_STATE_HOME").unwrap_or_else(|_| format!("{}/.local/state", home()));
    PathBuf::from(state).join("filigrane/fili-file-ui")
}

// Fichier d'état « clé=valeur » (une par ligne): lecture d'une clé
fn ui_get(text: &str, key: &str) -> String {
    text.lines().find_map(|l| l.split_once('=').filter(|(k, _)| *k == key).map(|(_, v)| v.trim().to_string())).unwrap_or_default()
}

// Le même texte avec la clé posée (ou retirée si la valeur est vide); clés et valeurs sans saut de ligne ni « = » dans la clé
fn ui_set(text: &str, key: &str, value: &str) -> String {
    let clean = |s: &str| s.replace(['\n', '\r'], " ");
    let (key, value) = (clean(key).replace('=', "-"), clean(value));
    let mut lines: Vec<String> = text.lines().filter(|l| l.split_once('=').is_none_or(|(k, _)| k != key)).map(String::from).collect();
    if !value.trim().is_empty() {
        lines.push(format!("{key}={}", value.trim()));
    }
    lines.sort();
    if lines.is_empty() { String::new() } else { lines.join("\n") + "\n" }
}

fn columns_file() -> PathBuf {
    let state = env::var("XDG_STATE_HOME").unwrap_or_else(|_| format!("{}/.local/state", home()));
    PathBuf::from(state).join("filigrane/fili-file-columns")
}

// État de fili-theme (« light » ou « dark »), comme FiliTheme.qml du fili-shell
fn theme_file() -> PathBuf {
    let state = env::var("XDG_STATE_HOME").unwrap_or_else(|_| format!("{}/.local/state", home()));
    PathBuf::from(state).join("filigrane/theme")
}

fn read_light() -> bool {
    std::fs::read_to_string(theme_file()).is_ok_and(|t| t.trim() == "light")
}

// Couleur d'accentuation (nom), « default » par défaut
fn read_accent() -> String {
    std::fs::read_to_string(theme_file().with_file_name("accent")).map(|t| t.trim().to_string()).unwrap_or_else(|_| "default".into())
}

impl cxx_qt::Initialize for Fili {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().light = read_light();
        self.as_mut().rust_mut().accent = QString::from(&read_accent());
        // Commandes de l'assistant (pas dans le sélecteur de fichiers)
        if pick::get().is_none() {
            let cmd = self.qt_thread();
            crate::context::serve(move |id, verb, arg, body| {
                let _ = cmd.queue(move |mut f| {
                    f.as_mut().command(id, &QString::from(&verb), &QString::from(&arg), &QString::from(&body));
                });
            });
        }
        // Montages: /proc/self/mountinfo signale chaque changement par POLLPRI
        let mounts = self.qt_thread();
        std::thread::spawn(move || {
            use std::{io::{Read, Seek}, os::fd::AsRawFd};
            let Ok(mut file) = std::fs::File::open("/proc/self/mountinfo") else { return };
            let mut pfd = libc::pollfd { fd: file.as_raw_fd(), events: libc::POLLPRI, revents: 0 };
            // Le changement est acquitté par une lecture complète du même descripteur
            let mut table = String::new();
            let mut ack = |file: &mut std::fs::File| {
                table.clear();
                let _ = file.rewind();
                let _ = file.read_to_string(&mut table);
            };
            ack(&mut file);
            loop {
                if unsafe { libc::poll(&mut pfd, 1, -1) } < 0 {
                    return;
                }
                // Laisser les montages en rafale finir, puis relire la table (le changement est acquitté par la lecture)
                std::thread::sleep(std::time::Duration::from_millis(300));
                ack(&mut file);
                if mounts.queue(|mut f| f.as_mut().places_changed()).is_err() {
                    return;
                }
            }
        });
        // Branchement et débranchement d'appareils (téléphone MTP...): udev réécrit sa base /run/udev/data; inotify, sans réveil au repos
        let devices = self.qt_thread();
        std::thread::spawn(move || unsafe {
            let fd = libc::inotify_init1(libc::IN_CLOEXEC);
            if fd < 0 {
                return;
            }
            let dir = std::ffi::CString::new("/run/udev/data").unwrap();
            let mask = libc::IN_CREATE | libc::IN_MOVED_TO | libc::IN_DELETE | libc::IN_MOVED_FROM;
            if libc::inotify_add_watch(fd, dir.as_ptr(), mask) < 0 {
                return;
            }
            let mut buf = [0u8; 4096];
            // Seuls comptent les appareils USB (c189:…) et les disques (b…): ni batterie ni clavier ne rafraîchissent la barre latérale
            let relevant = |buf: &[u8], n: usize| {
                let (mut i, mut any) = (0, false);
                while i + 16 <= n {
                    let len = u32::from_ne_bytes(buf[i + 12..i + 16].try_into().unwrap()) as usize;
                    let name = &buf[i + 16..(i + 16 + len).min(n)];
                    any |= name.starts_with(b"c189:") || name.starts_with(b"b");
                    i += 16 + len;
                }
                any
            };
            loop {
                let n = libc::read(fd, buf.as_mut_ptr().cast(), buf.len());
                if n <= 0 {
                    return;
                }
                let mut hit = relevant(&buf, n as usize);
                // Rafale d'événements (un branchement en écrit plusieurs): attendre le calme, puis relire les emplacements
                let mut pfd = libc::pollfd { fd, events: libc::POLLIN, revents: 0 };
                while libc::poll(&mut pfd, 1, 600) > 0 {
                    let n = libc::read(fd, buf.as_mut_ptr().cast(), buf.len());
                    if n > 0 {
                        hit |= relevant(&buf, n as usize);
                    }
                }
                if hit && devices.queue(|mut f| f.as_mut().places_changed()).is_err() {
                    return;
                }
            }
        });
        // Suivi du dossier d'état (inotify, fil bloqué sur read: aucun réveil tant que rien ne change)
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let file = theme_file();
            let Some(dir) = file.parent() else { return };
            let _ = std::fs::create_dir_all(dir);
            let Ok(c) = std::ffi::CString::new(dir.as_os_str().as_encoded_bytes()) else { return };
            let fd = unsafe { libc::inotify_init1(libc::IN_CLOEXEC) };
            if fd < 0
                || unsafe { libc::inotify_add_watch(fd, c.as_ptr(), libc::IN_CLOSE_WRITE | libc::IN_MOVED_TO | libc::IN_CREATE) } < 0
            {
                return;
            }
            let mut buf = [0u8; 4096];
            loop {
                if unsafe { libc::read(fd, buf.as_mut_ptr().cast(), buf.len()) } <= 0 {
                    return;
                }
                let light = read_light();
                let accent = read_accent();
                let sent = qt.queue(move |mut f| {
                    if f.light != light || f.accent.to_string() != accent {
                        f.as_mut().rust_mut().light = light;
                        f.as_mut().rust_mut().accent = QString::from(&accent);
                        f.as_mut().theme_changed();
                    }
                });
                if sent.is_err() {
                    return;
                }
            }
        });
    }
}

impl Fili {
    // Monte ou démonte en arrière-plan (udisksctl): le QML est prévenu par mounted / volumeError, puis placesChanged
    pub fn mount_volume(self: Pin<&mut Self>, dev: &QString) {
        let dev = dev.to_string();
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let r = volumes::mount(&dev);
            let _ = qt.queue(move |mut f| match r {
                Ok(path) => {
                    f.as_mut().places_changed();
                    f.as_mut().mounted(&QString::from(&path));
                }
                Err(e) => {
                    let msg = i18n::trf("Cannot mount “%1”: %2", &[&dev, &e]);
                    f.as_mut().volume_error(&QString::from(&msg));
                }
            });
        });
    }

    pub fn share_protocols(self: Pin<&mut Self>) {
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let json = share_run(&["protocols"]);
            let _ = qt.queue(move |mut f| f.as_mut().share_protocols_ready(&QString::from(&json)));
        });
    }

    // `tag` identifie le demandeur (Partager…, À proximité et son tour de scan): la réponse le porte
    pub fn share_peers(self: Pin<&mut Self>, proto: &QString, tag: &QString) {
        let (proto, tag) = (proto.to_string(), tag.to_string());
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let json = share_run(&["peers", &proto]);
            let _ = qt.queue(move |mut f| {
                f.as_mut().share_peers_ready(&QString::from(&proto), &QString::from(&tag), &QString::from(&json))
            });
        });
    }

    pub fn share_pick(&self, proto: &QString, id: &QString, name: &QString) {
        spawn_detached(Command::new(bin("fili-share")).args(["send", &proto.to_string(), &id.to_string(), &name.to_string()]));
    }

    pub fn receive_info(self: Pin<&mut Self>) {
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let info = share_run(&["receive-info"]);
            let received = share_run(&["received", "15"]);
            let _ = qt.queue(move |mut f| f.as_mut().receive_info_ready(&QString::from(&info), &QString::from(&received)));
        });
    }

    pub fn receive_set(self: Pin<&mut Self>, proto: &QString, on: bool) {
        let proto = proto.to_string();
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let _ = Command::new(bin("fili-share"))
                .args(["receive", &proto, if on { "on" } else { "off" }])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            let info = share_run(&["receive-info"]);
            let received = share_run(&["received", "15"]);
            let _ = qt.queue(move |mut f| f.as_mut().receive_info_ready(&QString::from(&info), &QString::from(&received)));
        });
    }

    pub fn share_send(&self, proto: &QString, id: &QString, name: &QString, paths: &QString) {
        let mut args = vec!["send-files".to_string(), proto.to_string(), id.to_string(), name.to_string()];
        args.extend(paths_of(paths));
        spawn_detached(Command::new(bin("fili-share")).args(args));
    }

    pub fn connect_server(self: Pin<&mut Self>, uri: &QString, user: &QString, password: &QString) {
        let (uri, user, password) = (uri.to_string(), user.to_string(), password.to_string());
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let r = volumes::connect(&uri, &user, &password);
            let _ = qt.queue(move |mut f| match r {
                Ok(path) => {
                    f.as_mut().places_changed();
                    f.as_mut().mounted(&QString::from(&path));
                }
                Err(e) => {
                    let msg = i18n::trf("Cannot connect to “%1”: %2", &[&uri, &e]);
                    f.as_mut().volume_error(&QString::from(&msg));
                }
            });
        });
    }

    pub fn unmount_volume(self: Pin<&mut Self>, dev: &QString) {
        let dev = dev.to_string();
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let r = volumes::unmount(&dev);
            let _ = qt.queue(move |mut f| match r {
                Ok(()) => f.as_mut().places_changed(),
                Err(e) => {
                    let msg = i18n::trf("Cannot eject “%1”: %2", &[&dev, &e]);
                    f.as_mut().volume_error(&QString::from(&msg));
                }
            });
        });
    }

    pub fn pick_kind(&self) -> QString {
        QString::from(pick::get().map_or("", |p| p.kind.as_str()))
    }

    pub fn pick_title(&self) -> QString {
        QString::from(pick::get().map_or("", |p| p.title.as_str()))
    }

    pub fn pick_name(&self) -> QString {
        QString::from(pick::get().map_or("", |p| p.name.as_str()))
    }

    pub fn pick_accept(&self) -> QString {
        QString::from(pick::get().map_or("", |p| p.accept.as_str()))
    }

    pub fn pick_multiple(&self) -> bool {
        pick::get().is_some_and(|p| p.multiple)
    }

    pub fn pick_directory(&self) -> bool {
        pick::get().is_some_and(|p| p.directory)
    }

    pub fn pick_filters(&self) -> QString {
        let names: Vec<String> = pick::get().map(|p| p.filters.iter().map(|f| json(&f.name)).collect()).unwrap_or_default();
        QString::from(&format!("[{}]", names.join(",")))
    }

    // Le sélecteur s'ouvre sur « tous les fichiers » (-1), jamais sur le filtre demandé par l'application: il cacherait des fichiers
    pub fn pick_filter(&self) -> i32 {
        -1
    }

    pub fn pick_finish(&self, paths: &QString, filter: i32) {
        if let Some(p) = pick::get() {
            let _ = std::fs::write(&p.out, format!("# filter {filter}\n{}\n", paths.to_string()));
        }
        std::process::exit(0);
    }

    pub fn columns(&self) -> QString {
        let text = std::fs::read_to_string(columns_file()).unwrap_or_default();
        let default = crate::config::load().columns.unwrap_or_else(|| "size,modified".into());
        QString::from(if text.trim().is_empty() { default.as_str() } else { text.trim() })
    }

    pub fn set_columns(&self, keys: &QString) {
        let file = columns_file();
        if let Some(dir) = file.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(file, keys.to_string());
    }

    pub fn ui_state(&self, key: &QString) -> QString {
        QString::from(&ui_get(&std::fs::read_to_string(ui_file()).unwrap_or_default(), &key.to_string()))
    }

    pub fn set_ui_state(&self, key: &QString, value: &QString) {
        let file = ui_file();
        let text = ui_set(&std::fs::read_to_string(&file).unwrap_or_default(), &key.to_string(), &value.to_string());
        if let Some(dir) = file.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(file, text);
    }

    pub fn view_mode(&self) -> QString {
        let text = std::fs::read_to_string(view_file()).unwrap_or_default();
        let text = if text.trim().is_empty() { crate::config::load().view.unwrap_or_default() } else { text };
        QString::from(if text.trim() == "grid" { "grid" } else { "list" })
    }

    pub fn set_view_mode(&self, mode: &QString) {
        let file = view_file();
        if let Some(dir) = file.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(file, mode.to_string());
    }

    pub fn path_exists(&self, path: &QString) -> bool {
        ops::exists(Path::new(&path.to_string()))
    }

    pub fn preview_shown(&self) -> bool {
        view_file().with_file_name("fili-file-preview").exists()
    }

    pub fn set_preview_shown(&self, on: bool) {
        let f = view_file().with_file_name("fili-file-preview");
        if on {
            if let Some(dir) = f.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let _ = std::fs::write(f, "1");
        } else {
            let _ = std::fs::remove_file(f);
        }
    }

    pub fn preview_info(&self, path: &QString) -> QString {
        QString::from(&preview(&path.to_string()))
    }

    pub fn pick_cancel(&self) {
        std::process::exit(1);
    }

    pub fn add_favorite(mut self: Pin<&mut Self>, path: &QString) {
        let p = normalize(&path.to_string(), "/").to_string_lossy().into_owned();
        let mut list = bookmarks();
        if !Path::new(&p).is_dir() || default_favorites().iter().any(|(d, _)| *d == p) || list.iter().any(|(b, _)| *b == p) {
            return;
        }
        list.push((p, String::new()));
        if write_bookmarks(&list).is_ok() {
            self.as_mut().places_changed();
        }
    }

    pub fn remove_favorite(mut self: Pin<&mut Self>, path: &QString) {
        let p = path.to_string();
        let list: Vec<(String, String)> = bookmarks().into_iter().filter(|(b, _)| *b != p).collect();
        if write_bookmarks(&list).is_ok() {
            self.as_mut().places_changed();
        }
    }

    pub fn trash_full(&self) -> bool {
        !ops::trash_is_empty()
    }

    pub fn is_favorite(&self, path: &QString) -> bool {
        let p = path.to_string();
        default_favorites().iter().any(|(d, _)| *d == p) || bookmarks().iter().any(|(b, _)| *b == p)
    }

    pub fn path_suggest(&self, text: &QString) -> QString {
        let list: Vec<String> = suggest(&text.to_string(), 40)
            .into_iter()
            .map(|(name, path, dir)| format!("{{\"name\":{},\"path\":{},\"dir\":{dir}}}", json(&name), json(&path)))
            .collect();
        QString::from(&format!("[{}]", list.join(",")))
    }

    // Presse-papiers Wayland par wl-copy (qui reste en arrière-plan tant que le texte est la sélection)
    pub fn copy_text(&self, text: &QString) {
        let text = text.to_string();
        let child = Command::new("wl-copy").arg("--").arg(text).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
        if let Ok(mut child) = child {
            std::thread::spawn(move || child.wait());
        }
    }

    pub fn tint(&self, url: &QString, accent: &QString, light: bool) -> QString {
        QString::from(icons::tint(&url.to_string(), &accent.to_string(), light))
    }

    pub fn about_rows(&self) -> QString {
        let rows = crate::about::rows();
        QString::from(format!(
            "[{}]",
            rows.iter().map(|(k, v)| format!("[{k:?},{v:?}]")).collect::<Vec<_>>().join(",")
        ))
    }

    pub fn about_folder(&self, light: bool) -> QString {
        QString::from(icons::tint(&icons::folder(), "default", light))
    }

    pub fn about_logo(&self, light: bool) -> QString {
        QString::from(crate::about::logo(light))
    }

    pub fn tr(&self, text: &QString) -> QString {
        QString::from(i18n::tr(&text.to_string()))
    }

    pub fn places(&self) -> QString {
        let home = home();
        let (places, devices) = (i18n::tr("Favorites"), i18n::tr("Locations"));
        let name = |p: &str| Path::new(p).file_name().map_or(p.to_string(), |n| n.to_string_lossy().into_owned());
        // (section, nom, chemin, nom d'icône, volume)
        let mut items: Vec<(&str, String, String, &str, String)> = Vec::new();
        let mut seen: Vec<String> = Vec::new();
        for (p, icon) in default_favorites() {
            let label = if p == home { name(&home) } else { name(&p) };
            items.push((places, label, p.clone(), icon, String::new()));
            seen.push(p);
        }
        // Favoris ajoutés par l'utilisateur (retirables)
        let mut removable: Vec<String> = Vec::new();
        for (p, label) in bookmarks() {
            if seen.contains(&p) || !Path::new(&p).is_dir() {
                continue;
            }
            let label = if label.is_empty() { name(&p) } else { label };
            let icon = if label.to_lowercase().contains("icloud") { "folder-cloud" } else { "folder" };
            items.push((places, label, p.clone(), icon, String::new()));
            removable.push(p.clone());
            seen.push(p);
        }
        // Dossiers souvent ouverts récemment (hors favoris, dossier personnel et racine), 5 au plus
        let frequent = i18n::tr("Frequent");
        for v in visits::ranked().into_iter().filter(|v| v.path != home && v.path != "/" && !seen.contains(&v.path)).take(5) {
            items.push((frequent, name(&v.path), v.path.clone(), "folder", String::new()));
        }
        // Partage de proximité (vue À proximité de la fenêtre, pas un dossier)
        items.push((devices, i18n::tr("Nearby").to_string(), "nearby:".into(), "network-wireless", String::new()));
        items.push((devices, i18n::tr("Computer").to_string(), "/".into(), "computer", String::new()));
        // Volumes amovibles: non montés, un clic les monte (path vide, dev renseigné)
        for v in volumes::list() {
            items.push((devices, v.name, v.mount.unwrap_or_default(), "drive-removable-media", v.dev));
        }
        // Téléphones et appareils photo MTP branchés: un clic les monte (gvfs)
        for v in volumes::mtp_devices() {
            items.push((devices, v.name, String::new(), "phone", v.dev));
        }
        // Serveurs déjà utilisés, pas connectés: un clic les reconnecte, ✕ les oublie
        for v in volumes::saved_servers() {
            items.push((devices, v.name, String::new(), "folder-remote", v.dev));
        }
        // Lecteurs réseau connectés (gvfs, NFS, CIFS, sshfs): éjectables comme les volumes
        for v in volumes::network() {
            items.push((devices, v.name, v.mount.unwrap_or_default(), "folder-remote", v.dev));
        }
        let _ = ops::ensure_trash(&ops::home_trash());
        let trash = if ops::trash_is_empty() { "user-trash" } else { "user-trash-full" };
        items.push((devices, i18n::tr("Trash").to_string(), ops::trash_files().to_string_lossy().into_owned(), trash, String::new()));
        let body: Vec<String> = items
            .iter()
            .map(|(s, n, p, i, dev)| {
                format!(
                    "{{\"section\":{},\"name\":{},\"path\":{},\"dev\":{},\"removable\":{},\"mono\":{},\"full\":{}}}",
                    json(s),
                    json(n),
                    json(p),
                    json(dev),
                    removable.contains(p),
                    json(&icons::symbolic(i)),
                    *i == "user-trash-full"
                )
            })
            .collect();
        QString::from(&format!("[{}]", body.join(",")))
    }

    pub fn start_path(&self) -> QString {
        let args: Vec<String> = env::args().skip(1).collect();
        let arg = pick::positional(&args).or_else(|| pick::get().map(|p| p.folder.clone()).filter(|f| !f.is_empty()));
        let start = crate::config::load().start.unwrap_or_else(home);
        let p = normalize(&arg.unwrap_or(start), &env::current_dir().map_or(home(), |d| d.to_string_lossy().into_owned()));
        QString::from(&p.to_string_lossy().into_owned())
    }

    pub fn home(&self) -> QString {
        QString::from(&home())
    }

    pub fn publish_context(&self, location: &QString, selection: &QString, results: &QString) {
        let loc = location.to_string();
        if loc.is_empty() {
            crate::context::clear();
        } else {
            crate::context::publish("fili-file", &loc, &paths_of(selection), &results.to_string());
        }
    }

    pub fn pid(&self) -> i32 {
        std::process::id() as i32
    }

    pub fn reply(&self, id: i32, text: &QString) {
        crate::context::reply(id, &text.to_string());
    }

    pub fn start_info(&self) -> bool {
        env::args().any(|a| a == "--info")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_state_file() {
        let t = ui_set("", "col.size", "120");
        assert_eq!(t, "col.size=120\n");
        let t = ui_set(&t, "preview", "340");
        assert_eq!(ui_get(&t, "col.size"), "120");
        assert_eq!(ui_get(&t, "preview"), "340");
        assert_eq!(ui_get(&t, "col.kind"), "");
        // Remplacer ne duplique pas; vide retire; un « = » ou un saut de ligne ne casse pas le format
        let t = ui_set(&t, "col.size", "150");
        assert_eq!(t, "col.size=150\npreview=340\n");
        assert_eq!(ui_set(&t, "col.size", ""), "preview=340\n");
        let t = ui_set(&t, "a=b", "x\ny");
        assert_eq!((ui_get(&t, "a-b"), t.lines().count()), ("x y".to_string(), 3));
        assert_eq!(ui_set("preview=1\n", "preview", ""), "");
    }

    #[test]
    fn parses_clipboard() {
        assert_eq!(parse_clip("cut\nfile:///tmp/a%20b\nfile:///tmp/c", true), (vec!["/tmp/a b".into(), "/tmp/c".into()], Some(true)));
        assert_eq!(parse_clip("copy\nfile:///tmp/a", true), (vec!["/tmp/a".into()], Some(false)));
        assert_eq!(parse_clip("# x\r\nfile:///tmp/a\r\nhttp://x/y\r\n", false), (vec!["/tmp/a".into()], None));
    }

    #[test]
    fn suggests_paths_like_fzf() {
        let d = std::env::temp_dir().join(format!("fili-file-suggest-{}", std::process::id()));
        std::fs::create_dir_all(d.join("Documents")).unwrap();
        std::fs::create_dir_all(d.join("Downloads")).unwrap();
        std::fs::write(d.join("notes.txt"), "x").unwrap();
        std::fs::write(d.join(".secret"), "x").unwrap();
        let base = d.to_string_lossy().into_owned();
        let names = |t: &str| suggest(t, 10).into_iter().map(|(n, _, dir)| format!("{n}{}", if dir { "/" } else { "" })).collect::<Vec<_>>();
        assert_eq!(names(&format!("{base}/")), vec!["Documents/", "Downloads/", "notes.txt"]);
        assert_eq!(names(&format!("{base}/dow")), vec!["Downloads/"]);
        assert_eq!(names(&format!("{base}/dcs")), vec!["Documents/"]);
        // Un point: les fichiers cachés entrent, et le nom qui commence par le point passe devant
        assert_eq!(names(&format!("{base}/."))[0], ".secret");
        assert!(!names(&format!("{base}/n")).contains(&".secret".to_string()));
        assert!(names(&format!("{base}/zzz")).is_empty());
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn normalizes_paths() {
        let h = home();
        assert_eq!(normalize("~/a/../b/./c", "/"), PathBuf::from(format!("{h}/b/c")));
        assert_eq!(normalize("file:///tmp/a%20b", "/"), PathBuf::from("/tmp/a b"));
        assert_eq!(normalize("../x", "/usr/share"), PathBuf::from("/usr/x"));
        assert_eq!(normalize("/..", "/"), PathBuf::from("/"));
    }

    #[test]
    fn archive_names() {
        assert_eq!(archive_stem("photos.TAR.GZ").as_deref(), Some("photos"));
        assert_eq!(archive_stem("a.b.zip").as_deref(), Some("a.b"));
        assert_eq!(archive_stem(".zip"), None);
        assert_eq!(archive_stem("notes.txt"), None);
    }

    #[test]
    fn decodes_percent() {
        assert_eq!(percent_decode("caf%C3%A9%2"), "café%2");
    }
}
