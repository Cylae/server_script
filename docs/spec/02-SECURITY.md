# 02-SECURITY.md — Security Specifications & Hardening Guidelines

### REQ-SEC-001 — Direct argv Execution Without Shell Interpolation
- Priorité        : P0
- Statut          : MANDATORY
- Justification   : Injection de commandes système si un paramètre utilisateur est passé à un shell intermediate (`sh -c` / `bash -c`).
- Dépendances     : aucune
- Prérequis env.  : aucun
- Risques         : aucun
- Preuves attendues: `rg -n 'sh -c|bash -c|format!\(.*Command' server_manager/src/` retourne zéro occurrence non sécurisée.
- Commandes de validation : `cargo test --test security_tests`
- Rollback        : `git revert`
- Critères d'acceptation :
  - All process invocations use explicit `std::process::Command::new("binary").args([...])`.

### REQ-SEC-002 — Strict Input Validation & Canonicalization
- Priorité        : P0
- Statut          : MANDATORY
- Justification   : Traversée de répertoire (path traversal) et injection de noms de services/utilisateurs invalides.
- Dépendances     : aucune
- Prérequis env.  : aucun
- Risques         : aucun
- Preuves attendues: `server_manager/src/core/validate.rs` contient les regex et fonctions de validation canonique.
- Commandes de validation : `cargo test test_input_validation`
- Rollback        : `git revert`
- Critères d'acceptation :
  - Service names match `^[a-z0-9][a-z0-9_-]{0,62}$`.
  - Usernames conform to Debian `NAME_REGEX`.
  - Path inputs are canonicalized and confined within allowed base directories.

### REQ-SEC-003 — Secret File Protection & Atomic Persistence
- Priorité        : P0
- Statut          : MANDATORY
- Justification   : Une fuite de droits sur `secrets.yaml` expose les mots de passe administrateur et tokens système.
- Dépendances     : REQ-SEC-002
- Prérequis env.  : Système de fichiers POSIX
- Risques         : Aucun si droits 0600 root:root
- Preuves attendues: Inspection des permissions via `stat -c "%a %U %G" /opt/server_manager/secrets.yaml`
- Commandes de validation : `cargo test test_secrets_file_permissions`
- Rollback        : `chmod 0600 /opt/server_manager/secrets.yaml`
- Critères d'acceptation :
  - File permissions are explicitly set to 0600 on creation/update.
  - Existing valid secrets files are never overwritten or regenerated.

### REQ-SEC-004 — Secret Redaction in Logs, Argv, and HTTP Responses
- Priorité        : P0
- Statut          : MANDATORY
- Justification   : Fuite d'identifiants dans le journal système ou les endpoints API.
- Dépendances     : aucune
- Prérequis env.  : aucun
- Risques         : aucun
- Preuves attendues: Capture stdout/stderr durant les opérations d'installation/administration.
- Commandes de validation : `cargo test test_secret_redaction`
- Rollback        : `git revert`
- Critères d'acceptation :
  - Zero plaintext secrets in stdout, stderr, logs, or HTTP response bodies.

### REQ-SEC-005 — Argon2id Password Hashing with Transparent Bcrypt Migration
- Priorité        : P1
- Statut          : MANDATORY
- Justification   : Renforcement des hachages de mots de passe contre les attaques par GPU/ASIC.
- Dépendances     : REQ-SEC-003
- Prérequis env.  : Détection RAM (profils LOW vs HIGH)
- Risques         : Incompatibilité de login si la migration n'est pas transparente.
- Preuves attendues: `server_manager/src/core/users.rs` supporte Argon2id et migre les hashs `$2b$`.
- Commandes de validation : `cargo test test_argon2id_migration`
- Rollback        : `git revert`
- Critères d'acceptation :
  - Argon2id used for all new password hashes (m=65536, t=3, p=4; reduced on LOW memory profile).
  - Legacy bcrypt hashes automatically upgraded to Argon2id upon successful login.

### REQ-SEC-006 — Web UI Session Security & CSRF Defense
- Priorité        : P1
- Statut          : MANDATORY
- Justification   : Vol de session (session hijacking) et fixation de session.
- Dépendances     : aucune
- Prérequis env.  : Web UI actif sur port 8099
- Risques         : aucun
- Preuves attendues: Intercepteurs/middleware dans `server_manager/src/interface/web.rs`.
- Commandes de validation : `cargo test test_session_security`
- Rollback        : `git revert`
- Critères d'acceptation :
  - Session ID regenerated on login.
  - Cookies configured with `HttpOnly`, `SameSite=Strict`, and `Secure` (when TLS enabled).
  - CSRF protection enabled for state-changing requests.

### REQ-SEC-007 — Localhost Default Binding
- Priorité        : P1
- Statut          : MANDATORY
- Justification   : Prévention de l'exposition non désirée de l'interface d'administration web.
- Dépendances     : aucune
- Prérequis env.  : aucun
- Risques         : Changement de comportement (breaking) : nécessite `--bind 0.0.0.0` pour exposition externe directe.
- Preuves attendues: Valeur par défaut dans `server_manager/src/interface/cli.rs`.
- Commandes de validation : `cargo test test_web_bind_default`
- Rollback        : Executer avec `--bind 0.0.0.0`
- Critères d'acceptation :
  - Web UI defaults to `127.0.0.1:8099`.

### REQ-SEC-008 — Exhaustive RBAC Matrix
- Priorité        : P1
- Statut          : MANDATORY
- Justification   : Contournement de privilèges par des comptes non-administrateurs.
- Dépendances     : REQ-SEC-006
- Prérequis env.  : aucun
- Risques         : aucun
- Preuves attendues: Tests de matrice de rôles (Admin, Operator, Auditor/Observer).
- Commandes de validation : `cargo test test_rbac_matrix`
- Rollback        : `git revert`
- Critères d'acceptation :
  - All web routes enforce explicit role access checks.

### REQ-SEC-009 — HTTP Security Response Headers
- Priorité        : P1
- Statut          : MANDATORY
- Justification   : Attaques XSS, clickjacking, mime-sniffing.
- Dépendances     : aucune
- Prérequis env.  : aucun
- Risques         : aucun
- Preuves attendues: Middleware axum dans `server_manager/src/interface/web.rs`.
- Commandes de validation : `cargo test test_security_headers`
- Rollback        : `git revert`
- Critères d'acceptation :
  - `Content-Security-Policy`, `X-Content-Type-Options`, `Referrer-Policy`, and `X-Frame-Options` headers injected on all HTTP responses.

### REQ-SEC-010 — Docker Socket Least-Privilege Confinement
- Priorité        : P1
- Statut          : MANDATORY
- Justification   : Monter le socket Docker équivaut à accorder les privilèges root hôte.
- Dépendances     : aucune
- Prérequis env.  : Docker daemon
- Risques         : aucun
- Preuves attendues: Inspections des définitions de services dans `server_manager/src/services/`.
- Commandes de validation : `cargo test test_docker_socket_mounting`
- Rollback        : `git revert`
- Critères d'acceptation :
  - Docker socket (`/var/run/docker.sock`) is not mounted in container stacks unless explicitly documented and required, and mounted read-only where possible.

### REQ-SEC-011 — Container Image & GitHub Action Pinning
- Priorité        : P1
- Statut          : MANDATORY
- Justification   : Attaques de la chaîne d'approvisionnement (supply chain attacks).
- Dépendances     : aucune
- Prérequis env.  : aucun
- Risques         : aucun
- Preuves attendues: `server_manager/src/services/*.rs` et `.github/workflows/`.
- Commandes de validation : `cargo test test_pinned_images`
- Rollback        : `git revert`
- Critères d'acceptation :
  - Container images pinned by tag.
  - GitHub Actions pinned by commit SHA.
