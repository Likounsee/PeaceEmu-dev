# PeaceEmu

Ce dépôt est la **version de développement (Dev)** de PeaceEmu. Il évolue indépendamment du dépôt Stable : ses mises à jour, ses versions et ses tests ne modifient pas la version Stable.

PeaceEmu est un projet d'émulation multi-consoles en Rust. Il repose sur une application de bureau commune et des modules de console distribués séparément.

[Site officiel](https://likounsee.github.io/PeaceEmu/)

## État du projet

Le projet est en développement. L'application hôte fournit l'interface, la bibliothèque de jeux, la gestion des sessions, des préférences et l'installation des modules. Les modules de console évoluent séparément : la présence d'un module installé ne signifie pas à elle seule que tous les jeux ou toutes les fonctions de la console sont compatibles.

## Structure

- `PeaceEmu/crates/core` — contrats, runtime, sessions, modules, contenus et sauvegardes.
- `PeaceEmu/crates/app` — contrôleur applicatif partagé.
- `PeaceEmu/crates/desktop` — interface native, bibliothèque, paramètres et gestion des modules.
- `PeaceEmu/crates/windows-*` — intégrations et garanties propres à Windows.
- `.github/workflows` — intégration continue et vérifications multiplateformes.

## Développement

Prérequis : Rust stable avec `rustfmt` et `clippy`. Depuis la racine du dépôt :

```sh
cd PeaceEmu
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
```

L'application de bureau est un crate séparé du workspace principal :

```sh
cargo run --manifest-path crates/desktop/Cargo.toml
cargo test --manifest-path crates/desktop/Cargo.toml --all-targets --locked
cargo clippy --manifest-path crates/desktop/Cargo.toml --all-targets --locked -- -D warnings
```

La CI vérifie les crates Rust et l'application de bureau sur Linux, Windows et macOS, ainsi que le démarrage de l'application sur Linux. Consultez l'onglet **Actions** pour connaître le résultat du commit concerné.

## Règles du dépôt

- Les développements se font sur la branche `PeaceEmu`.
- Les dépôts Dev et Stable restent séparés.
- Les modules de console restent indépendants de l'interface et du runtime hôte.
- Les fichiers de jeux, firmwares et BIOS sont des contenus non fiables. Ne distribuez aucun contenu protégé sans autorisation.
- Vérifiez les licences, la sécurité et les conditions de redistribution avant d'ajouter une dépendance.
