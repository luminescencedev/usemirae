Mirae — Installer et migrer le développement vers un PC Windows dédié

Statut : guide opérationnelProjet : luminescencedev/usemiraeObjectif : conserver le PC principal/de jeu intact et déplacer tout le développement natif Windows de Mirae sur une seconde machine.

1. Ce que cette installation va faire

La machine dédiée servira à :

compiler le moteur et le shell Rust ;

exécuter les scripts de build et macros procédurales Cargo ;

lancer les tests Rust et TypeScript ;

développer la fenêtre native wry + tao ;

tester WebView2 et les futures API Windows ;

utiliser Claude Code dans le dépôt ;

produire plus tard les builds Windows de Mirae.

Le PC principal reste inchangé :

Smart App Control peut y rester actif ;

aucun Build Tools, Rust ou Node n’a besoin d’y être installé pour Mirae ;

aucune exclusion de sécurité n’est nécessaire ;

il peut seulement servir à consulter GitHub ou, plus tard, à piloter le PC dédié à distance.

2. Pourquoi une machine dédiée est nécessaire

Mirae compile continuellement des fichiers locaux non signés :

build-script-build.exe ;

DLL de macros procédurales ;

exécutables de tests ;

exécutables de doctests ;

mirae-engine.exe ;

mirae-shell.exe.

Smart App Control peut bloquer ces fichiers avant leur première exécution.

Les erreurs suivantes peuvent toutes provenir du même blocage Windows :

could not execute process ... (never executed)

Une stratégie de contrôle d'application a bloqué ce fichier.
(os error 4551)

can't find crate for `time_macros`

La dernière erreur est trompeuse : la dépendance peut être présente et correctement compilée, mais Windows peut bloquer son chargement ou l’exécution d’un artefact intermédiaire.

Microsoft indique que Smart App Control n’est pas recommandé sur une machine de développement, car tout mode autre que désactivé peut affecter négativement les outils de développement.

Références officielles :

Smart App Control FAQ :https://support.microsoft.com/windows/smart-app-control-frequently-asked-questions

Vérifier la stratégie avec citool.exe :https://learn.microsoft.com/windows/apps/develop/smart-app-control/test-your-app-with-smart-app-control

Configuration Rust sous Windows :https://learn.microsoft.com/windows/dev-environment/rust/setup

Partie A — Sauvegarder le travail depuis l’ancien PC

Cette partie se fait sur le PC actuel avant de passer à la nouvelle machine.

3. Ouvrir le dépôt actuel

cd "CHEMIN\VERS\usemirae"
git status

Ne supprime rien avant d’avoir poussé les changements.

4. Créer une branche de transfert pour MIR-0016

Si tu n’es pas déjà sur une branche MIR-0016 :

git switch -c feat/MIR-0016-native-shell

Si elle existe déjà :

git switch feat/MIR-0016-native-shell

5. Vérifier ce qui doit être transféré

Les changements utiles peuvent notamment concerner :

Cargo.toml
Cargo.lock
apps/desktop-shell/Cargo.toml
apps/desktop-shell/src/main.rs
DEPENDENCY_VERSIONS.md
BOOTSTRAP_TICKETS.md
SETUP.md
docs/adr/ADR-0068-system-webview-desktop-shell.md

Les versions retenues pour MIR-0016 doivent être :

wry  = 0.55.1
tao  = 0.35.3

Vérifie les modifications :

git status --short
git diff

6. Ne pas transférer les fichiers propres à l’ancien PC

Ne commit pas et ne copie pas :

target/
node_modules/
.env
.env.*
*.log
mir0016-build.log
mir0016-tree.txt

Ne transfère pas non plus une configuration Cargo machine-locale qui contiendrait par exemple :

[build]
target-dir = "C:\\mirae-target"

Le fichier canonique du dépôt .cargo/config.toml ne doit contenir que les réglages réellement partagés par tous les développeurs, notamment l’alias xtask.

Vérifie où se trouvent les configurations Cargo éventuellement héritées :

Get-ChildItem -Path .. -Filter config.toml -Recurse -Force -ErrorAction SilentlyContinue |
    Where-Object { $_.FullName -match "\\.cargo\\" }

7. Committer la branche de transfert

Ajoute uniquement les fichiers utiles :

git add Cargo.toml Cargo.lock
git add apps/desktop-shell
git add DEPENDENCY_VERSIONS.md BOOTSTRAP_TICKETS.md
git add SETUP.md

Ajoute les autres fichiers uniquement s’ils font réellement partie de MIR-0016.

Contrôle le commit avant de le créer :

git diff --cached

Puis :

git commit -m "wip(MIR-0016): prepare native window dependencies"
git push -u origin feat/MIR-0016-native-shell

Un commit WIP sur une branche dédiée est préférable à une copie manuelle du dossier : Git conserve exactement les fichiers nécessaires et exclut les artefacts compilés.

Si aucun changement MIR-0016 n’est encore prêt, pousse au minimum ce guide sur une branche dédiée.

8. Vérifier que la branche est bien distante

git status
git branch -vv
git log -1 --oneline

La branche doit afficher un suivi vers :

origin/feat/MIR-0016-native-shell

À partir de ce moment, le second PC peut reprendre le travail sans copier le dossier local.

Partie B — Préparer le nouveau PC Windows

9. Préparation recommandée

Prévois :

Windows 11 64 bits à jour ;

un compte administrateur local ;

au moins 40 à 60 Go d’espace libre ;

une connexion Internet stable ;

idéalement un SSD ;

l’accès à ton compte GitHub ;

l’accès à ton compte Claude.

Crée un point de restauration Windows avant de modifier les paramètres de sécurité :

Menu Démarrer
→ Rechercher « Créer un point de restauration »
→ Protection du système
→ Créer

Nom recommandé :

Avant environnement Mirae

10. Mettre Windows à jour

Paramètres
→ Windows Update
→ Rechercher des mises à jour

Installe toutes les mises à jour disponibles, puis redémarre.

Vérifie la version :

Get-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion" |
    Select-Object DisplayVersion, CurrentBuild, UBR

La documentation actuelle de Microsoft indique que les versions récentes de Windows peuvent réactiver Smart App Control depuis l’application Sécurité Windows. Cette possibilité dépend toutefois de l’état de mise à jour de la machine : mets Windows complètement à jour avant de changer le réglage.

11. Vérifier précisément Smart App Control

Méthode visuelle

Sécurité Windows
→ Contrôle des applications et du navigateur
→ Paramètres de Smart App Control

Vérification de la stratégie réellement appliquée

Ouvre PowerShell et lance :

citool.exe -lp

Cherche :

Friendly Name: VerifiedAndReputableDesktop
Is Currently Enforced: true

Cette combinaison signifie que Smart App Control est appliqué.

Le mode évaluation apparaît généralement comme :

Friendly Name: VerifiedAndReputableDesktopEvaluation
Is Currently Enforced: true

Si tu vois une autre stratégie appliquée, surtout sur un PC professionnel ou géré par une organisation, ne la désactive pas : il peut s’agir d’une politique App Control/WDAC administrée.

12. Désactiver Smart App Control uniquement sur le PC dédié

À faire uniquement si la machine est personnelle et réservée au développement.

Sécurité Windows
→ Contrôle des applications et du navigateur
→ Paramètres de Smart App Control
→ Désactivé

Redémarre ensuite la machine.

Ne modifie pas directement les clés de registre de Smart App Control.

Cette opération ne désactive pas :

Microsoft Defender Antivirus ;

la protection en temps réel ;

SmartScreen ;

le pare-feu Windows ;

l’intégrité mémoire ;

Windows Update.

Ne crée pas d’exclusion Defender pour le dossier du projet par défaut. Le but est uniquement de retirer le blocage App Control incompatible avec les exécutables générés localement, pas de réduire les autres protections.

Après redémarrage :

citool.exe -lp

Vérifie que VerifiedAndReputableDesktop n’apparaît plus comme stratégie actuellement appliquée.

Partie C — Installer les outils de développement

13. Installer Git for Windows

Téléchargement :

https://git-scm.com/download/win

Les options par défaut conviennent.

Ferme puis rouvre PowerShell après l’installation.

Vérifie :

git --version

Configure ton identité Git :

git config --global user.name "Arthur Garnier"
git config --global user.email "TON_EMAIL_GITHUB"
git config --global core.autocrlf true
git config --global core.longpaths true

N’utilise pas l’adresse EFREI si elle n’est pas associée à ton compte GitHub. Utilise l’adresse que tu veux voir apparaître dans les commits ou ton adresse GitHub privée noreply.

14. Installer les Microsoft C++ Build Tools

Rust avec la cible MSVC a besoin de l’éditeur de liens et du SDK Windows.

Téléchargement officiel :

https://visualstudio.microsoft.com/downloads/

Dans Tools for Visual Studio, installe Build Tools for Visual Studio.

Dans l’installateur, sélectionne :

Développement Desktop en C++

Vérifie dans les détails d’installation que sont inclus :

MSVC Build Tools x64/x86 ;

Windows 11 SDK ;

MSBuild ;

C++ core desktop features.

Il n’est pas nécessaire d’installer l’IDE Visual Studio complet si tu utilises un autre éditeur.

Redémarre Windows après l’installation si l’installateur le demande.

15. Installer Rust avec rustup

Méthode officielle :

https://rustup.rs

Ou avec WinGet :

winget install --id Rustlang.Rustup -e

Ferme puis rouvre PowerShell.

Vérifie :

rustup --version
rustc --version
cargo --version

Le dépôt contient rust-toolchain.toml et installera automatiquement :

Rust 1.97.1
clippy
rustfmt
rust-src

Il n’est pas nécessaire de forcer manuellement cette version avant d’avoir cloné le dépôt.

16. Installer NVM for Windows et Node.js

Le dépôt impose exactement :

Node.js 24.18.1

Installe NVM for Windows depuis :

https://github.com/coreybutler/nvm-windows/releases

Sur une machine propre, n’installe pas Node séparément avant NVM.

Après installation, ouvre PowerShell en administrateur :

nvm install 24.18.1 64
nvm use 24.18.1

Ferme puis rouvre le terminal.

Vérifie :

node --version
npm --version
nvm current

Résultat attendu pour Node :

v24.18.1

Si node --version affiche une autre version :

where.exe node
nvm debug

Un ancien Node installé séparément peut prendre la priorité dans le PATH.

17. Activer pnpm avec Corepack

Le dépôt impose exactement :

pnpm 11.17.0

Dans PowerShell :

corepack enable
corepack prepare pnpm@11.17.0 --activate

Vérifie :

pnpm --version

Résultat attendu :

11.17.0

N’installe pas pnpm globalement avec npm install -g pnpm, sauf si Corepack est réellement indisponible et qu’un ticket de tooling le documente.

18. Installer Claude Code

Claude Code fonctionne nativement sous Windows avec Git for Windows.

Installation :

npm install -g @anthropic-ai/claude-code

Vérification :

claude --version
claude doctor

Lance une première fois :

claude

Suis le parcours de connexion avec ton compte Claude.

Si Claude Code ne trouve pas Git Bash :

$env:CLAUDE_CODE_GIT_BASH_PATH = "C:\Program Files\Git\bin\bash.exe"
[Environment]::SetEnvironmentVariable(
    "CLAUDE_CODE_GIT_BASH_PATH",
    "C:\Program Files\Git\bin\bash.exe",
    "User"
)

Documentation officielle :

https://docs.anthropic.com/en/docs/claude-code/getting-started

Partie D — Récupérer Mirae sur le nouveau PC

19. Cloner dans un chemin court

Utilise un chemin sans espace ni caractère accentué :

New-Item -ItemType Directory -Path C:\dev -Force
Set-Location C:\dev

git clone https://github.com/luminescencedev/usemirae.git
Set-Location C:\dev\usemirae

Ne copie pas l’ancien dossier target/.

Ne copie pas node_modules/.

Ne copie pas le dossier .git/ à la main : le clone Git l’a déjà créé correctement.

20. Récupérer la branche MIR-0016

git fetch --all --prune
git switch feat/MIR-0016-native-shell

Si aucune branche distante MIR-0016 n’a été créée :

git switch -c feat/MIR-0016-native-shell

Mais dans ce cas, les changements non commités de l’ancien PC ne seront pas présents. Reviens à la Partie A et pousse-les d’abord.

Vérifie :

git status
git log -3 --oneline

21. Vérifier les fichiers de version

Get-Content .node-version
Get-Content rust-toolchain.toml
Select-String -Path package.json -Pattern '"packageManager"|"node"|"pnpm"'

Résultats attendus :

Node.js 24.18.1
pnpm 11.17.0
Rust 1.97.1

Ces valeurs sont définies dans le dépôt, pas dans ce guide.

Partie E — Installer et valider le projet

22. Vérifier la chaîne d’outils

Depuis C:\dev\usemirae :

cargo xtask bootstrap

Résultat attendu :

Mirae toolchain check
  ok   node   expected 24.18.1    found 24.18.1
  ok   pnpm   expected 11.17.0    found 11.17.0
  ok   rustc  expected 1.97.1     found 1.97.1
  ok   cargo  expected 1.97.1     found 1.97.1

Toolchain matches DEPENDENCY_VERSIONS.md.

Si la commande télécharge Rust 1.97.1 au premier lancement, c’est normal.

23. Installer les dépendances JavaScript

pnpm install --frozen-lockfile

Le script preinstall relance également la vérification du toolchain.

N’utilise pas :

npm install
yarn
bun install
pnpm update --latest

Le fichier pnpm-lock.yaml doit rester inchangé après une installation propre.

24. Lancer la validation complète

cargo xtask check

Cette commande contrôle notamment :

versions des outils ;

politique du dépôt ;

secrets et chemins machine-locaux ;

dépendances et directions d’architecture ;

contrats générés ;

formatage ;

Clippy ;

ESLint ;

tests Rust ;

tests TypeScript ;

documentation.

Le premier lancement peut être long parce que toute la workspace Rust est compilée.

25. Test décisif contre l’ancien blocage Windows

Lance explicitement :

cargo test --workspace

Puis :

cargo build --package mirae-shell

Ces commandes doivent pouvoir :

exécuter les scripts de build ;

charger les macros procédurales ;

lancer les binaires de tests ;

produire le shell.

Aucune erreur ne doit contenir :

never executed
os error 4551
Une stratégie de contrôle d'application a bloqué ce fichier
can't find crate for time_macros

Si ces erreurs réapparaissent, consulte la section Dépannage avant de modifier les versions Cargo.

Partie F — Lancer Mirae

26. Lancer le moteur seul

cargo run --package mirae-engine

Le comportement exact dépend du mode de lancement. Le moteur peut publier son état de démarrage puis s’arrêter s’il n’est pas supervisé.

27. Lancer le shell avec sa fenêtre

Depuis MIR-0016, le shell ouvre une vraie fenêtre et il faut donc lui indiquer où se trouve l’interface compilée. Construis-la d’abord :

pnpm --filter @mirae/control-ui build

Puis lance le shell :

$env:MIRAE_UI_PATH = "C:\dev\usemirae\apps\control-ui\dist"
cargo run --package mirae-shell

Le shell doit :

localiser mirae-engine.exe ;

lancer le moteur ;

effectuer le handshake authentifié ;

afficher une confirmation ;

ouvrir la fenêtre de contrôle et y afficher l’interface ;

arrêter proprement le moteur à la fermeture de la fenêtre.

Exemple :

control UI served from C:\dev\usemirae\apps\control-ui\dist
handshake accepted: protocol=1.0 session=... max_frame=1048576 launches=1

Sans MIRAE_UI_PATH, le shell cherche un dossier ui à côté de l’exécutable, ce qui correspond à la disposition d’un build packagé. S’il ne trouve ni l’un ni l’autre, il le dit et s’arrête : c’est une panne d’interface, pas une panne de moteur, et le message le précise.

F5 recharge la fenêtre. Les ressources sont relues sur le disque à chaque requête, donc un nouveau build apparaît sans relancer le shell.

28. Lancer l’interface React séparément

pnpm --filter @mirae/control-ui dev

Ouvre l’adresse locale indiquée par Vite. Ce mode reste utile pour le développement de l’interface avec rechargement à chaud ; la fenêtre du shell, elle, sert toujours des ressources packagées locales (501 invariant 2).

Tant que le pont shell ↔ interface n’est pas terminé, l’UI affiche que le moteur est indisponible, dans les deux modes. Ce comportement est volontaire : l’interface ne doit jamais simuler un état moteur qu’elle ne peut pas observer.

Partie G — Reprendre MIR-0016 avec Claude Code

29. Vérifications avant Claude

git status
cargo xtask check

Le dépôt doit être propre ou ne contenir que les modifications volontairement liées à MIR-0016.

30. Lancer Claude Code depuis le dépôt

Set-Location C:\dev\usemirae
claude

Claude doit lire, dans cet ordre :

CLAUDE.md
DEPENDENCY_VERSIONS.md
BOOTSTRAP_TICKETS.md
SETUP.md
docs/05-platform/501-desktop-shell.md
docs/adr/ADR-0037-native-shell-replaceable-web-control-ui.md
docs/adr/ADR-0068-system-webview-desktop-shell.md
apps/desktop-shell/src/main.rs
apps/desktop-shell/Cargo.toml
Cargo.toml

31. Prompt prêt à donner à Claude

Historique : MIR-0016 a été livré sur la machine dédiée avec ce prompt. Il est conservé comme modèle pour un ticket suivant, pas comme travail restant.

Continue MIR-0016 on the current branch.

First read CLAUDE.md, DEPENDENCY_VERSIONS.md, BOOTSTRAP_TICKETS.md, SETUP.md,
docs/05-platform/501-desktop-shell.md, ADR-0037, and ADR-0068.

The development machine has been moved specifically because Smart App Control
blocked locally compiled Rust executables on the previous PC. Do not change Cargo
versions or dependency features to work around that old machine-level issue.

Preserve the approved exact pair:
- wry 0.55.1
- tao 0.35.3

Keep the existing engine process, supervisor, authenticated handshake, and Mirae
IPC. Do not introduce Electron or Tauri.

Implement the smallest complete MIR-0016 vertical slice:
- create the Tao event loop and native window;
- create the Wry system webview;
- load locally packaged UI resources through the approved custom protocol;
- deny arbitrary top-level navigation;
- keep the bridge narrow and typed;
- keep the engine supervised while the event loop is alive;
- stop the engine cooperatively when the window closes;
- add the required tests and diagnostics;
- update the dependency review and ticket tracker;
- run cargo xtask check.

Do not merge. Stop after one reviewable MIR-0016 result and report every command
run, its result, and any remaining gap.

Partie H — Dépannage

32. cargo xtask bootstrap indique une mauvaise version de Node

nvm use 24.18.1
node --version
where.exe node
nvm debug

Si plusieurs installations Node existent, désinstalle l’installation MSI indépendante et laisse NVM gérer le symlink.

33. pnpm est introuvable

corepack enable
corepack prepare pnpm@11.17.0 --activate
pnpm --version

Ferme et rouvre PowerShell si nécessaire.

34. link.exe est introuvable

Rouvre l’installateur Visual Studio Build Tools et vérifie :

Développement Desktop en C++
MSVC x64/x86
Windows 11 SDK

Puis redémarre le terminal.

35. Erreur os error 4551 ou never executed

Vérifie la stratégie :

citool.exe -lp

Puis examine les derniers blocages :

Get-WinEvent -FilterHashtable @{
    LogName = "Microsoft-Windows-CodeIntegrity/Operational"
    Id      = 3077
} -MaxEvents 20 |
    Select-Object TimeCreated, Id, Message |
    Format-List

Si VerifiedAndReputableDesktop est toujours appliqué, Smart App Control n’est pas désactivé ou le redémarrage n’a pas finalisé le changement.

Si le journal montre une autre politique, le PC est peut-être soumis à une stratégie WDAC/App Control différente. Ne modifie pas Cargo pour contourner une politique Windows.

36. Erreur can't find crate for time_macros

Commence par rechercher un événement Code Integrity 3077 au même moment.

Ne rétrograde pas immédiatement time, cookie ou wry.

Sur l’ancien PC, cette erreur était un effet secondaire du blocage d’exécution Windows.

Si aucun événement de blocage n’existe sur la nouvelle machine :

cargo clean
Remove-Item Cargo.lock -Force
cargo generate-lockfile
cargo build --package mirae-shell -vv

Ne régénère le lockfile que sur la branche MIR-0016 et contrôle le diff avant commit.

37. WebView2 est absent

Windows 11 à jour et les outils Visual Studio utilisent normalement WebView2.

Si Wry signale explicitement l’absence du runtime, installe l’Evergreen Runtime officiel :

https://developer.microsoft.com/microsoft-edge/webview2/

Ne télécharge pas WebView2 depuis un site tiers.

38. Le clone ne contient pas les changements MIR-0016

git branch -a
git fetch --all --prune
git switch feat/MIR-0016-native-shell

Si la branche distante n’existe pas, les changements étaient uniquement locaux sur l’ancien PC. Retourne à la Partie A et pousse-les.

39. Le dépôt contient soudainement beaucoup de fichiers modifiés

Vérifie :

git status --short

Causes fréquentes :

node_modules ou target mal ignoré ;

changement global de fins de ligne ;

configuration Cargo locale commitée ;

fichier généré modifié à la main ;

commande de mise à jour de dépendances lancée sans ticket.

N’exécute pas git add . avant d’avoir compris chaque groupe de fichiers.

Partie I — Checklist finale

Ancien PC

Branche feat/MIR-0016-native-shell créée.

Pins wry 0.55.1 et tao 0.35.3 sauvegardés.

Cargo.lock sauvegardé si modifié.

Aucun target/ ou node_modules/ commité.

Aucune configuration Cargo machine-locale commitée.

Branche poussée vers GitHub.

Nouveau PC

Windows complètement mis à jour.

Point de restauration créé.

Smart App Control vérifié avec citool.exe -lp.

Smart App Control désactivé uniquement sur le PC dédié.

Defender, SmartScreen, pare-feu et intégrité mémoire toujours actifs.

Git installé.

Build Tools avec « Développement Desktop en C++ » installé.

Rustup installé.

Node 24.18.1 actif.

pnpm 11.17.0 actif.

Claude Code installé et authentifié.

Repo cloné dans C:\dev\usemirae.

Branche MIR-0016 récupérée.

cargo xtask bootstrap passe.

pnpm install --frozen-lockfile passe.

cargo xtask check passe.

cargo test --workspace exécute réellement les tests.

cargo build --package mirae-shell passe.

Aucun événement Code Integrity 3077 lié aux artefacts Mirae.

Claude peut reprendre MIR-0016 avec le prompt fourni.

Résumé minimal des commandes sur le nouveau PC

# Après installation de Git, Build Tools, rustup et NVM
nvm install 24.18.1 64
nvm use 24.18.1

corepack enable
corepack prepare pnpm@11.17.0 --activate

npm install -g @anthropic-ai/claude-code

New-Item -ItemType Directory -Path C:\dev -Force
Set-Location C:\dev

git clone https://github.com/luminescencedev/usemirae.git
Set-Location C:\dev\usemirae

git fetch --all --prune
git switch feat/MIR-0016-native-shell

cargo xtask bootstrap
pnpm install --frozen-lockfile
cargo xtask check

cargo test --workspace
cargo build --package mirae-shell

claude