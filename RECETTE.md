# Recette de Casier (fili-file) avant la v1

Ce qui ne s'automatise pas: souris, clavier réel, périphériques, autres applications. Le reste est gardé par
`tests/fili-file/run.sh` (27 scénarios sans affichage, voir AGENTS.md) et `cargo test`: navigation, sélections, onglets,
copier/couper/coller, conflits de nom, renommage, corbeille, archives, dialogues. Ne pas les refaire ici.

**Préparation** (une fois):

1. `cd apps/fili-file && CARGO_TARGET_DIR=$HOME/.cache/filigrane-apps cargo build --release`, puis `./bootstrap.sh --apps`
   (déploie sans sudo).
2. `tests/fili-file/recette-fixture.sh` crée `/tmp/casier-recette` (noms à espaces, accents, emoji, nom très long,
   5000 fichiers, liens cassés, archives dont `protegee.zip` de mot de passe `secret`, image, PDF, dossier en lecture
   seule, deux `rapport.txt` différents dans `conflit/source` et `conflit/cible`). Ouvrir Casier dessus:
   `fili-file /tmp/casier-recette` (ou Mod+E puis Ctrl+L).
3. Cocher au fur et à mesure. Pour chaque échec: la ligne, ce qui s'est passé, ce qui était attendu (le dire à l'assistant,
   qui corrige puis relance `tests/fili-file/run.sh`).

Raccourcis relevés dans `qml/Main.qml`: Ctrl+C/X/V, Ctrl+D (dupliquer), F2, Suppr ou Ctrl+Retour arrière (corbeille),
Ctrl+Z, Ctrl+A, Ctrl+Maj+I (inverser), Ctrl+Maj+M (motif), Ctrl+F ou `/` (recherche), Ctrl+L (chemin), Ctrl+H (cachés),
Ctrl+1/2 (liste/icônes), Ctrl+Maj+P (aperçu), Ctrl+I (infos), Ctrl+Maj+N (nouveau dossier), Ctrl+T/W/Tab/Maj+Tab
(onglets), Alt+←/→/↑, Ctrl+[ / Ctrl+], F5 ou Ctrl+R, Ctrl+Alt+T (terminal ici), Ctrl+Maj+C (chemin), Ctrl+Alt+C (nom),
Ctrl+K (serveur), Ctrl+Maj+R (à proximité), Ctrl+Maj+Suppr (vider la corbeille), Ctrl+Q.

## 1. Clavier (clavier français)

- [ ] **`/`** ouvre la recherche sur un clavier AZERTY (Maj+: ou la touche dédiée selon la disposition), sans taper de « / » dans le champ.
- [ ] Flèches haut/bas déplacent la ligne courante; Maj+flèches étendent la sélection; Ctrl+A sélectionne tout.
- [ ] Retour/Entrée ouvre le dossier ou le fichier; Alt+↑ remonte.
- [ ] Taper des lettres dans la liste saute au premier nom qui commence ainsi; une pause remet la saisie à zéro; avec les noms accentués et les majuscules.
- [ ] **Espace** n'a pas d'aperçu rapide (c'est Ctrl+Maj+P): à décider avant la v1 si l'absence gêne.
- [ ] Suppr et Ctrl+Retour arrière mettent à la corbeille; Ctrl+Z défait.
- [ ] Les raccourcis ne font **rien** pendant la saisie d'un nom, du chemin, de la recherche, et derrière un dialogue ouvert (Compresser, mot de passe, À propos, Serveur, Infos).
- [ ] Échap ferme dialogue, menu, renommage, recherche, champ du chemin, dans cet ordre, sans fermer la fenêtre.
- [ ] Le contour de focus est visible et suit le clavier.

## 2. Souris

- [ ] Clic sélectionne; Ctrl+clic ajoute ou retire; Maj+clic étend; clic dans le vide désélectionne.
- [ ] Double-clic ouvre un dossier, un fichier (application par défaut via xdg-open), une archive (extraction ici).
- [ ] **Lasso**: glisser depuis le vide sélectionne la zone traversée, en liste et en icônes; Ctrl ajoute.
- [ ] Clic droit sur un élément, sur plusieurs éléments, sur le vide, sur un élément de la corbeille: entrées adaptées (Ouvrir avec…, Renommer, Dupliquer, Compresser/Extraire, Copier le chemin, Informations…).
- [ ] Clic droit sur l'en-tête: choix des colonnes; clic sur une colonne trie, un deuxième clic inverse; la largeur des colonnes n'est pas réglable (à décider si l'absence gêne).
- [ ] Défilement à la molette et au pavé tactile: inertie naturelle, pas de saut; défilement rapide du dossier `gros` (5000 fichiers) fluide.
- [ ] Survol: pas de clignotement; curseur correct sur les bords et les boutons.

## 3. Renommage

- [ ] F2 renomme sur place: le nom sans extension est sélectionné; Entrée valide; Échap annule; clic ailleurs valide.
- [ ] Renommer vers un nom existant: message d'erreur lisible, rien n'est écrasé.
- [ ] Noms difficiles: `été à la plage.txt`, `emoji 🎉.txt`, `l'apostrophe.txt`, nom de 200+ caractères (troncature au milieu, nom complet en infobulle ou dans les infos).
- [ ] **Renommage en lot** (sélectionner plusieurs, F2): aperçu en direct pour Remplacer, Numéroter, Casse; **Appliquer** renomme tout; un nom invalide ou en collision est signalé et bloque; Ctrl+Z défait le lot en une fois.

## 4. Glisser-déposer

- [ ] Glisser un fichier vers un dossier de la liste: déplace; avec Ctrl: copie; le curseur et la surbrillance de la cible l'annoncent.
- [ ] Glisser plusieurs éléments sélectionnés.
- [ ] Glisser vers un lieu de la barre latérale (Documents, corbeille).
- [ ] Glisser vers un autre onglet (survol de l'onglet) et réorganiser les onglets en les glissant.
- [ ] Glisser d'une autre application (navigateur: image ou fichier téléchargé; terminal) vers Casier.
- [ ] Glisser de Casier vers le navigateur (zone d'envoi de fichier d'un site) et vers une autre application.
- [ ] Échap pendant le glissement annule; lâcher dans le dossier d'origine ne fait rien.
- [ ] Conflit de nom au dépôt: `conflit/source/rapport.txt` glissé dans `conflit/cible`: le dialogue propose Remplacer / Garder les deux / Ignorer / Annuler, **chacun cliqué à la souris** et validé à Entrée (= Garder les deux), Échap (= Annuler); le contenu final correspond au choix.

## 5. Menu de la fili-bar et menu contextuel

- [ ] Pour chaque entrée du menu de Casier dans la fili-bar (Fichier, Édition, Présentation, Aller): clic = la commande se fait; raccourci affiché identique à celui du clavier; entrées grisées quand la commande est impossible (rien de sélectionné, corbeille).
- [ ] « À propos de Casier »: fiche avec icône, « Aussi connu sous le nom de fili-file », version (doit afficher 1.0.0 après la sortie de recette).
- [ ] Menu de la barre et fenêtre restent d'accord après avoir changé d'onglet, ouvert la corbeille, affiché des résultats.

## 6. Dossier, chemin et recherche

- [ ] Ctrl+L: saisie d'un chemin, `~`, `..`, chemin relatif; les propositions façon fzf apparaissent et Tab/flèches les choisissent; un mot sans « / » saute à un dossier déjà visité.
- [ ] Fil d'Ariane cliquable (chaque segment); un chemin très long se replie sans casser la barre.
- [ ] Recherche (Ctrl+F): filtre en direct; Échap la vide; le filtre se vide en changeant de dossier.
- [ ] Dossier vide (`vide`): état vide lisible. Dossier sans droit de lecture: message « permission refusée » clair, pas de fenêtre vide muette.
- [ ] `lecture-seule`: Nouveau dossier, coller, renommer, déposer donnent une erreur lisible; rien n'est créé.
- [ ] `liens`: lien valide s'ouvre sur sa cible; lien cassé s'affiche (icône distincte) et ne plante pas; lien vers un dossier s'ouvre; Infos montre la cible.
- [ ] Fichiers cachés (Ctrl+H) apparaissent et disparaissent; l'état est gardé à la réouverture.

## 7. Aperçu, miniatures, infos

- [ ] Ctrl+Maj+P: aperçu d'image (`image.png`), de PDF (`document.pdf`), de texte (`notes.txt`), de dossier; panneau à largeur fixe; vide quand rien n'est sélectionné.
- [ ] Miniatures en mode icônes (`medias`): image et PDF; première ouverture un peu plus lente, la deuxième immédiate (cache `~/.cache/filigrane/thumbs`).
- [ ] Infos (Ctrl+I): taille de dossier calculée en arrière-plan (barre ou valeur qui se met à jour), SHA-256 d'un fichier, droits en cases à cocher (modifier puis annuler), plusieurs éléments.
- [ ] Un dossier de 5000 fichiers s'ouvre sans geler la fenêtre; le tri par chaque colonne reste fluide.

## 8. Archives

- [ ] Compresser (dialogue): zip, 7z, tar.gz/xz/zst; niveau; sans fichiers cachés; **avec mot de passe** (zip, 7z): l'archive demande ce mot de passe à l'ouverture ailleurs.
- [ ] Extraire `normale.zip` / `normale.7z`: résultat à côté, nom libre si collision.
- [ ] Double-clic sur `protegee.zip`: demande de mot de passe; un mauvais mot de passe est signalé et redemandé; `secret` extrait.
- [ ] Annuler une compression ou une extraction en cours (Échap ou liste des tâches): rien de partiel ne reste.

## 9. Corbeille et volumes

- [ ] Corbeille du dossier personnel: mettre, ouvrir « Corbeille », remettre en place, supprimer définitivement, vider (confirmation, annulable).
- [ ] **Corbeille d'un autre volume** (clé USB ou second disque): un fichier supprimé là va dans `.Trash-UID` du volume et se retrouve dans la Corbeille de Casier.
- [ ] **Clé USB réelle**: branchement = apparition dans la barre latérale sans relancer; clic la monte (ou montage automatique); le contenu s'affiche; bouton d'éjection la démonte et la ligne disparaît; débranchement brutal ne plante pas.
- [ ] Copie de gros fichiers vers la clé: progression, annulation, file d'attente de plusieurs copies, liste des tâches.
- [ ] **Téléphone MTP**: branché, apparaît; montage demande l'adresse à gio si besoin; parcours, copie dans les deux sens; éjecté, disparaît jusqu'au rebranchement.
- [ ] **Serveur** (Ctrl+K): adresse `smb://…` ou `sftp://…`, identifiants, erreur d'adresse lisible.
- [ ] « À proximité » (Ctrl+Maj+R) et Partager: liste des appareils, envoi d'un fichier, réception.

## 10. Intégration au système

- [ ] **Mod+E** ouvre Casier; un lien « Afficher dans le dossier » d'un navigateur (Chrome: téléchargements) ouvre Casier avec le fichier sélectionné (service FileManager1).
- [ ] **Sélecteur de fichiers du portail dans Chrome**: « Importer un fichier » (un fichier, plusieurs), « Enregistrer sous… » (nom proposé, écrasement demandé), Annuler; la fenêtre est flottante, au bon dossier de départ, filtre de types respecté.
- [ ] `xdg-open /tmp/casier-recette` et `inode/directory` ouvrent Casier (pas Thunar).
- [ ] « Ouvrir avec… »: liste des applications, l'application préférée en premier; choisir lance le fichier.
- [ ] Ouvrir un terminal ici (Ctrl+Alt+T) dans le bon dossier; copier le chemin/le nom (Ctrl+Maj+C, Ctrl+Alt+C) puis coller ailleurs.
- [ ] Copier dans Casier puis coller dans un autre gestionnaire (ou l'inverse): les fichiers suivent; couper n'efface qu'après le collage.
- [ ] Fenêtres multiples: deux fenêtres Casier indépendantes; l'assistant (Mod+Y) voit le dossier et la sélection de la fenêtre active.
- [ ] Thème clair et sombre, taille de police du système, écran haute densité: pas de texte coupé, contrastes lisibles, icônes nettes.
- [ ] Fermer la fenêtre pendant une copie: avertit ou termine proprement, sans fichier à moitié copié.

## 11. Mesures (à relever sur la machine, `budget`)

| Mesure | Valeur | Objectif |
|---|---|---|
| Ouverture (fenêtre visible) | 240 ms (3 essais identiques, 2026-11-02) | < 350 ms |
| Mémoire privée fenêtre ouverte sur un dossier | 43 Mo (PSS 84 Mo, RSS 155 Mo) | < 40 Mo |
| Après ouverture de `gros` (5000 fichiers) et de l'aperçu | à relever | pas de croissance continue |
| Après 20 ouvertures/fermetures de dossiers | à relever | revient à ±5 Mo |

La mémoire dépasse l'objectif de 3 Mo (V0: ≈ 30 Mo). Répartition mesurée le 2026-11-02: 24 Mo de mémoire anonyme et 10 Mo de tas (Qt Quick, scène graphique, pilote Mesa/LLVM), 1 Mo de tas JavaScript QML, le reste en bibliothèques; le code Rust de l'application n'en est pas la cause visible. Décision à prendre: accepter 43 Mo (3 Mo de plus que l'objectif pour onglets, aperçu, grille, dialogues) ou chercher (désactiver des couches Qt Quick inutiles, `QSG_RHI_BACKEND`, éviter les `MultiEffect`).

## 12. Sortie de recette

Quand tout est coché (ou les écarts acceptés, notés ici):

- [ ] Écarts connus acceptés: …
- [ ] `tests/fili-shell/run.sh` vert.
- [ ] `version = "1.0.0"` dans `Cargo.toml`, affichée dans la fiche « À propos ».
- [ ] Retrait de Thunar (paquets `thunar`, `thunar-volman`, `thunar-archive-plugin` de `packages/desktop.txt`, blocs `.thunar` des CSS GTK3, bloc `accels.scm` de `bootstrap.sh`, chaîne i18n `Thunar: Space = quick look`, `references/resources.md`), puis `budget diff`.
