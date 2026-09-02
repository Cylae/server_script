# DECISIONS.md — Architecture Decision Records & Consolidation Matrix

## 1. MATRICE DE CONSOLIDATION DES DIVERGENCES

### 1.1 Divergence de PÉRIMÈTRE

| Axe | Valeurs trouvées dans les sources | DÉCISION | MOTIF |
|---|---|---|---|
| Nature du produit | (a) « orchestrateur de production kernel-grade Debian » gérant nginx/php/postgres/wireguard via `services.d/*.sh` ; (b) « orchestrateur media/cloud Docker » avec 28 services et dashboard port 8099 | **(b) fait foi.** Le dépôt réel est (b) : crate `server_manager`, `src/core/{hardware,docker,compose,firewall,users,secrets}.rs`, `src/services/{infra,media,arr,download,apps}.rs`. | (a) décrit un logiciel qui n'existe pas. Imposer (a) revient à commander une réécriture totale non demandée, sous couvert d'« audit ». La cible (a) est reclassée en `VISION-OPTIONAL` (Annexe Z), hors périmètre obligatoire. |
| Arborescence cible | `src/bin/server-manager.rs`, `src/core/{atomicio,cgroups,fanotify,lock,sandbox,wal}.rs`, `services.d/*.sh`, `ui/public/**`, `tests/{unit,integration,fuzz,chaos,concurrency}` | **Conserver la topologie réelle** `server_manager/src/{core,services,interface}`. Les modules nouveaux (`wal.rs`, `atomic_io.rs`, `lock.rs`) s'ajoutent DANS `src/core/`. | Un déplacement de racine casse le CI, les chemins de test, les benches Criterion et toute PR en cours, pour zéro gain fonctionnel. Refactor de layout = gate séparée G9, optionnelle. |
| Langage `services.d` | Recettes Bash + shellcheck + shfmt + BATS | **REJETÉ comme obligatoire.** `AGENTS.md` du dépôt stipule « Rust First: We do not use Python or Bash scripts for logic. » | Contradiction directe avec l'instruction locale. Introduire Bash = régression de sûreté de type et de testabilité. Si un jour un hook shell existe, alors shellcheck devient obligatoire (règle conditionnelle, pas préalable). |

### 1.2 Divergence sur le NOMBRE DE TESTS et la COUVERTURE

| Source | Assertions totales | Idempotence | Fuzzing | Concurrence | Chaos | Couverture branches | Score mutation |
|---|---|---|---|---|---|---|---|
| gemini-code-…-2.md | 500 | 100 | 150 | 100 | 150 | non spécifié | non spécifié |
| server_script_prompt-4.txt | 500 | 100 | 150 | 100 | 150 | non spécifié | non spécifié |
| prompt_2.txt | 600 | 100 | 200 | 150 | 150 | > 90 % | > 85 % |
| prompt_8-5.txt | 600 | 100 | 200 | 150 | 150 | > 90 % | > 85 % |
| prompt_3-10.txt | 700 | 150 | 250 | 150 | 150 | > 90 % | > 90 % |
| prompt_4-9.txt | 1000 | 200 | 300 | 200 | 200 | > 95 % | > 90 % |
| prompt_5-8 / 6-7.txt | 1000 | 250 | 350 | 200 | 200 | > 95 % | > 90 % |
| prompt_7-6.txt | 1200 | 300 | 400 | 200 | 200 | > 98 % | > 95 % |

**DÉCISION.** Le nombre absolu d'assertions est **abandonné comme critère de livraison**.
Motif : c'est une métrique de vanité, directement incitative à la triche (§L0.6), et aucune des sources ne le justifie par une analyse de risque. Il est remplacé par :

- **Critère T1 (obligatoire)** : couverture de branches ≥ **75 %** sur `src/core/` et `src/services/`, mesurée par `cargo llvm-cov --branch --fail-under-lines 75`.
- **Critère T2 (obligatoire)** : **couverture par contrat**, pas par volume. Chaque comportement listé au §4.2 possède ≥ 1 test nommé `contract_<domaine>_<comportement>`.
- **Critère T3 (obligatoire)** : **zéro régression**. Chaque défaut corrigé ajoute un test `regression_<id-finding>`.
- **Critère T4 (optionnel, durcissement)** : `cargo-mutants` score ≥ 60 % sur `src/core/`.

### 1.3 Divergence LANDLOCK

| Source | Version exigée |
|---|---|
| gemini-code-2, prompt_2, prompt_8-5, server_script_prompt-4 | « Landlock LSM » sans version |
| prompt_3-10, prompt_5-8, prompt_6-7, prompt_7-6 | **ABI v4** |
| prompt_4-9 | **ABI v5** |

**DÉCISION.** Aucune version n'est codée en dur. Landlock est *feature-detected* au runtime. Mode BestEffort par défaut, WARN si absent.

### 1.4 Divergence CRYPTOGRAPHIE

| Élément | Valeurs sources | DÉCISION |
| --- | --- | --- |
| Hash de mot de passe | Argon2id vs bcrypt | Argon2id obligatoire pour NOUVEAUX hachages + migration transparente au login depuis bcrypt. |
| Paramètres Argon2id | m=65536, t=3, p=4 | Retenu, borné par le profil mémoire (m=19456 sur profil LOW). |
| TLS | TLS 1.3 exclusif vs TLS 1.2 | TLS 1.3 imposé en serveur d'écoute ; TLS 1.2 accepté pour clients sortants (APT, registries). |
| Asymétrique | Ed25519 vs RSA vs NIST P-256 | Ed25519/X25519 en préférence ; RSA ≥ 3072 et NIST P-256 (requis par WebAuthn) acceptés. |
| Post-quantique | ML-KEM / Kyber768 | Reclassé OPTIONNEL / HARDENING jusqu'à stabilisation dans rustls. |
| Chiffrement de sauvegarde | ChaCha20-Poly1305 | Retenu avec nonce unique par chunk, KDF Argon2id/HKDF, et entête d'archive versionné. |

### 1.5 Divergence PRIVILÈGES / ISOLATION

| Axe | Sources | DÉCISION |
| --- | --- | --- |
| Séparation de privilèges | Broker non privilégié + daemon root | Scission par étapes : Whitelist d'opérations nommées typées en Rust (G6.1). |
| Socket | Bind localhost vs unix socket | Bind par défaut sur `127.0.0.1:8099` (breaking change), flag `--bind` pour exposition. |
| `chattr +i` / `hidepid` / `noexec /tmp` | Diverses | Reclassés `HARDENING-OPT-IN`. |

### 1.6 Divergence RÉSEAU

| Axe | Sources | DÉCISION |
| --- | --- | --- |
| Pare-feu | UFW vs nftables | UFW reste le backend par défaut. Backend nftables optionnel derrière un trait `FirewallBackend`. |
| Egress `policy drop` | Diverses | Reclassé `HARDENING-OPT-IN` avec `--dry-run` et rollback automatique (`at now + 5 min`). |
| Fail2ban | Supprimer vs conserver | Conservé jusqu'à preuve d'équivalence nftables active. |

### 1.7 Divergence APT / STOCKAGE / PAQUETS

| Axe | DÉCISION |
| --- | --- |
| APT | deb822, keyrings 0644 root:root, verrous d'attente, flags noninteractifs. |
| Stockage persistant | Persistance YAML conservée ; toutes écritures atomiques (tmpfile + fsync + rename) et verrouillées par `flock`. |

### 1.8 Divergence WEBUI

| Axe | DÉCISION |
| --- | --- |
| Framework | Vanilla JS/CSS embedded, budget < 40 KB gzip mesuré en CI. |
| WCAG | WCAG AA obligatoire (contraste 4.5:1), AAA visé sur texte de corps. |
| Auth & RBAC | Session durcie + Argon2id + RBAC (Admin, Operator, Auditor, Observer alias) + WebAuthn en 2FA opt-in. |

### 1.9 Divergence CI/CD & PERFORMANCES

| Axe | DÉCISION |
| --- | --- |
| Workflow GitHub | Corriger triggers `main` + `pull_request`, matrix stable/MSRV, cargo fmt, clippy, test, audit, deny. |
| Budgets perf | Mesurer les valeurs réelles sur l'existant en G8 et figer +10% en budget de non-régression. |

---

## 2. ARCHITECTURE DECISION RECORDS (ADRs)

### ADR-001: Retention of Docker Media/Cloud Scope
- **Contexte**: Divergence entre les spécifications décrivant un orchestrateur systemd/kernel Debian et le code réel du dépôt.
- **Décision**: Retenir le périmètre réel Docker media/cloud stack de `server_manager`.
- **Conséquences**: La réécriture complète vers systemd/services.d est écartée du périmètre obligatoire et reclassée en Annexe Z.

### ADR-002: Abandoning Raw Assertion Counts in Favor of Branch Coverage Floor
- **Contexte**: Exigence de 500 à 1200 assertions arbitraires.
- **Décision**: Remplacer le nombre d'assertions par un plancher de couverture de branches à 75% (`cargo llvm-cov`) et des tests basés sur les contrats (`contract_*`).
- **Conséquences**: Suppression de la métrique de vanité au profit d'une vérification de couverture mesurable.

### ADR-003: Graceful Degradation for Landlock LSM
- **Contexte**: Exigence d'ABI Landlock fixe (v4/v5).
- **Décision**: Négociation au runtime de la plus haute ABI supportée par le noyau, avec repli en mode unsandboxed + WARN log.
- **Conséquences**: Garantit la compatibilité du binaire sur les noyaux Linux antérieurs sans crash.

### ADR-004: Transparent Migration from Bcrypt to Argon2id
- **Contexte**: Le dépôt utilise `bcrypt`, les spécifications exigent `Argon2id`.
- **Décision**: Employer Argon2id pour tous les nouveaux comptes et mots de passe modifiés, et re-hacher de manière transparente les comptes bcrypt existants au login.
- **Conséquences**: Conservation de l'accès pour les utilisateurs existants sans perte de données.

### ADR-005: NIST P-256 Curve Support
- **Contexte**: Interdiction théorique des courbes NIST vs exigence de support WebAuthn / FIDO2.
- **Décision**: Autoriser NIST P-256 spécifiquement pour la compatibilité WebAuthn (COSE ES256).
- **Conséquences**: Évite une contradiction fatale empêchant l'implémentation de WebAuthn.

### ADR-006: Post-Quantum Cryptography (PQC) Deferred
- **Contexte**: Exigence ML-KEM / Kyber768 non encore stabilisée dans l'écosystème Rust standard.
- **Décision**: Classer la PQC en Hardening Optionnel.
- **Conséquences**: Évite de bloquer la compilation sur des dépendances expérimentales.

### ADR-007: UFW Primary Backend with Pluggable Nftables
- **Contexte**: Code basé sur UFW vs demande de nftables natif.
- **Décision**: Conserver UFW par défaut, introduire un trait `FirewallBackend` et une implémentation nftables optionnelle.
- **Conséquences**: Sécurité et stabilité préservées pour les déploiements existants.

### ADR-008: Non-Default Egress Drop Policy
- **Contexte**: Application d'un Egress `policy drop` global.
- **Décision**: Conserver la politique Egress ouverte par défaut et passer Egress Drop en OPT-IN avec `--dry-run` et auto-rollback.
- **Conséquences**: Évite de couper les accès réseau critiques (DNS, Docker registries, SSH).

### ADR-009: Atomic YAML Persistence over Embedded Database
- **Contexte**: Exigence SQLite WAL / Sled vs persistance YAML existante.
- **Décision**: Conserver YAML pour garder des fichiers lisibles et auditables, mais sécuriser toutes les écritures via tmpfile + fsync + rename + advisory flock.
- **Conséquences**: Évite une migration de format complexe tout en résolvant les risques de corruption de données.

### ADR-010: Claiming SLSA Level 3 Provenance Attestation
- **Contexte**: Revendication de SLSA Level 4.
- **Décision**: Revendiquer SLSA Level 3 via la génération de SBOM CycloneDX et `actions/attest-build-provenance` en CI.
- **Conséquences**: Alignement honnête sur les capacités effectives de la chaîne de build GitHub Actions.

### ADR-011: Measured Empirical Performance Budgets
- **Contexte**: Seuils arbitraires postulés (15 MB RSS / 5 ms startup).
- **Décision**: Mesurer les performances réelles au cours de la Gate 8 et fixer un budget de non-régression à +10%.
- **Conséquences**: Budgets réalistes basés sur des métriques réelles.

### ADR-012: Accessibility Standards (WCAG AA Required, AAA Targeted)
- **Contexte**: Exigence WCAG AAA sur l'ensemble de l'interface.
- **Décision**: Rendre WCAG AA obligatoire (contraste 4.5:1) et viser AAA sur les textes de corps.
- **Conséquences**: Conformité mesurable via `axe-core` sans blocage sur les états désactivés.

### ADR-013: Localhost Binding Default for Web UI
- **Contexte**: Web UI écoutait sur `0.0.0.0:8099`.
- **Décision**: Par défaut, bind sur `127.0.0.1:8099`, exposition externe via `--bind 0.0.0.0`.
- **Conséquences**: Sécurité renforcée par défaut (breaking change documenté).

### ADR-014: Opt-In File Systems and Kernel Hardening
- **Contexte**: Invocation systématique de `chattr +i` et `hidepid=invisible`.
- **Décision**: Classer ces mesures en `HARDENING-OPT-IN`.
- **Conséquences**: Prévention du blocage des mises à jour APT et des outils de supervision.

### ADR-015: 3-Attempt Gate Budget
- **Contexte**: Directive « itérer sans limite de temps ».
- **Décision**: Limiter les tentatives de correction par gate à 3 maximum avant ouverture en draft et documentation de l'état d'échec.
- **Conséquences**: Évite les boucles infinies de commits d'agent non livrés.
