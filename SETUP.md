# Mirae — Installation d'une machine de développement

Guide pour préparer un PC Windows 11 à développer Mirae, de zéro.

Compter environ **45 minutes**, dont beaucoup d'attente pendant les téléchargements.

---

## 0. Avant de commencer : le point important

Mirae est une application desktop Windows. La compiler produit en permanence des
exécutables non signés (scripts de build, macros procédurales, binaires de test),
que Windows doit pouvoir lancer.

**Smart App Control l'en empêche.** C'est une protection de Windows 11 qui refuse
d'exécuter tout binaire non signé et sans réputation. Elle n'accepte aucune
exclusion : ni par dossier, ni par fichier, ni par processus.

C'est pour cette raison qu'une machine dédiée au développement est préférable à un
PC personnel : on désactive Smart App Control uniquement sur celle-là.

Toutes les autres protections restent actives : Microsoft Defender, SmartScreen,
la protection en temps réel, l'intégrité mémoire, le pare-feu.

---

## 1. Vérifier l'état de Smart App Control

Ouvre PowerShell (pas besoin d'administrateur) et colle :

```powershell
Get-ItemProperty "HKLM:\SYSTEM\CurrentControlSet\Control\CI\Policy" |
    Select-Object VerifiedAndReputablePolicyState
```

| Valeur | Signification | À faire |
|---:|---|---|
| `0` | désactivé | rien, passe à l'étape 3 |
| `1` | **actif, en mode appliqué** | étape 2 |
| `2` | mode évaluation | étape 2 par précaution |
| clé absente | non pris en charge | rien, passe à l'étape 3 |

## 2. Désactiver Smart App Control

**Uniquement sur la machine de développement.**

D'abord, vérifie que Windows est à jour, puis relève ta version :

```powershell
Get-ItemProperty "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion" |
    Select-Object DisplayVersion, CurrentBuild, UBR
```

À partir des builds `26100.7701` et `26200.7701`, Smart App Control peut être
**réactivé** ensuite sans réinstaller Windows. En dessous, la désactivation est
définitive : il faudrait réinitialiser Windows pour le rallumer.

Ensuite :

```text
Sécurité Windows
  → Contrôle des applications et du navigateur
    → Paramètres de Smart App Control
      → Désactivé
```

Redémarre, puis contrôle que la valeur est bien passée à `0` avec la commande de
l'étape 1.

> Ne modifie jamais cette clé de registre à la main. Microsoft précise que cela
> peut laisser le système dans un état incohérent. Passe par l'interface.

---

## 3. Installer les outils

### Git

<https://git-scm.com/download/win> — installeur par défaut.

### Visual Studio Build Tools

Nécessaire : Rust utilise l'éditeur de liens MSVC sous Windows.

<https://visualstudio.microsoft.com/visual-cpp-build-tools/>

Dans l'installeur, coche **« Développement Desktop en C++ »**. C'est plusieurs
gigaoctets ; c'est normal.

### Rust

<https://rustup.rs> — lance `rustup-init.exe`, installation par défaut.

La bonne version de Rust s'installera toute seule au premier build : le dépôt
contient un `rust-toolchain.toml` qui l'impose.

### Node.js

Installe **nvm for Windows** : <https://github.com/coreybutler/nvm-windows/releases>

Puis, dans un PowerShell **administrateur** :

```powershell
nvm install 24.18.1
nvm use 24.18.1
```

La version exacte est imposée par le fichier `.node-version` du dépôt.

### pnpm

pnpm s'active tout seul via corepack, qui est fourni avec Node :

```powershell
corepack enable
```

La version exacte vient du champ `packageManager` du `package.json`.

### WebView2

Déjà présent sur Windows 11 à jour. Rien à faire.

---

## 4. Récupérer le projet

Choisis un chemin **court et sans espaces** ni caractères accentués. Les chemins
longs posent encore des problèmes à certains outils Windows.

```powershell
mkdir C:\dev
cd C:\dev
git clone https://github.com/luminescencedev/usemirae.git
cd usemirae
```

---

## 5. Vérifier la chaîne d'outils

```powershell
cargo xtask bootstrap
```

Cette commande compare Rust, Node et pnpm aux versions imposées par le dépôt. En
cas d'écart, elle affiche la version attendue, la version trouvée, et la commande
exacte qui corrige le problème.

Résultat attendu :

```text
Mirae toolchain check
  ok   node   expected 24.18.1    found 24.18.1
  ok   pnpm   expected 11.17.0    found 11.17.0
  ok   rustc  expected 1.97.1     found 1.97.1
  ok   cargo  expected 1.97.1     found 1.97.1

Toolchain matches DEPENDENCY_VERSIONS.md.
```

---

## 6. Installer les dépendances et tout valider

```powershell
pnpm install --frozen-lockfile
cargo xtask check
```

`cargo xtask check` enchaîne : politiques du dépôt, contrôle des contrats générés,
formatage, lint, tests Rust et TypeScript, validation de la documentation.

Le premier passage compile tout : compte **5 à 15 minutes**. Les suivants sont
courts.

Résultat attendu : la commande se termine sans erreur.

---

## 7. Lancer l'application

### Le moteur seul

```powershell
cargo run --package mirae-engine
```

Il affiche son état de démarrage en JSON, puis s'arrête.

### Le shell qui supervise le moteur

```powershell
cargo run --package mirae-shell
```

Le shell lance le moteur, effectue le handshake authentifié, puis l'arrête
proprement :

```text
handshake accepted: protocol=1.0 session=... max_frame=1048576 launches=1
```

### L'interface de contrôle

```powershell
pnpm --filter @mirae/control-ui dev
```

Ouvre l'adresse affichée dans un navigateur.

> Il n'y a pas encore de fenêtre native : c'est le ticket MIR-0016. L'interface
> affichera « Engine unavailable », ce qui est le comportement attendu tant que le
> pont entre la fenêtre et le moteur n'existe pas.

---

## 8. En cas de problème

### « could not execute process ... (never executed) »

### « Une stratégie de contrôle d'application a bloqué ce fichier (os error 4551) »

### « can't find crate for `time_macros` »

Ces trois messages ont la même cause : Smart App Control bloque les binaires
compilés localement. Reprends les étapes 1 et 2.

C'est particulièrement trompeur pour le troisième : l'erreur désigne une
dépendance, alors que le vrai problème est que Windows a empêché l'exécution de la
macro procédurale.

### Les tests échouent alors que la compilation réussit

Même cause. La compilation peut aboutir pendant que l'exécution des binaires de
test est refusée.

### Popup « Une partie de cette application a été bloquée »

Ne l'ignore pas, contrairement à ce qu'on pourrait croire : c'est le symptôme du
même blocage.

### Les builds sont très lents

Ajoute une exclusion Microsoft Defender pour le dossier `target` du dépôt. C'est
une optimisation, sans rapport avec Smart App Control, et cela réduit la
surveillance antivirus sur ce dossier : à toi de juger.

---

## 9. Résumé des commandes utiles

```powershell
cargo xtask help              # toutes les commandes disponibles
cargo xtask bootstrap         # vérifie la chaîne d'outils
cargo xtask check             # validation complète, avant chaque commit
cargo xtask fmt               # formate Rust et TypeScript
cargo xtask test              # tous les tests
cargo xtask docs --check      # valide la documentation
```

Documents de référence à la racine du dépôt :

- `README.md` — vue d'ensemble et démarrage
- `CLAUDE.md` — règles d'ingénierie
- `DEPENDENCY_VERSIONS.md` — versions exactes, faisant autorité
- `BOOTSTRAP_TICKETS.md` — état d'avancement des tickets
- `docs/SUMMARY.md` — toute la documentation d'architecture
