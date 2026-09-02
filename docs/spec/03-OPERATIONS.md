# 03-OPERATIONS.md — Operational Standards, Recovery & Doctor

### REQ-OPS-001 — Operational Idempotency
- Priorité        : P0
- Statut          : MANDATORY
- Justification   : Les réexécutions d'installation ou d'application ne doivent jamais corrompre l'état ni dupliquer des configurations.
- Dépendances     : aucune
- Prérequis env.  : aucun
- Risques         : aucun
- Preuves attendues: Execution double des commandes CLI en environnement de test.
- Commandes de validation : `cargo test test_idempotency`
- Rollback        : `git revert`
- Critères d'acceptation :
  - Re-executing `install` or `apply` results in zero redundant configuration changes, zero duplicate file lines, and return code 0.

### REQ-OPS-002 — Compensatory Operational Logging & Replay Recovery
- Priorité        : P0
- Statut          : MANDATORY
- Justification   : Prévention des états partiellement appliqués et corrompus lors d'une interruption brutale (panne de courant, SIGKILL).
- Dépendances     : REQ-OPS-001
- Prérequis env.  : `/var/lib/server_manager/` accessible en écriture
- Risques         : aucun
- Preuves attendues: `server_manager/src/core/journal.rs`
- Commandes de validation : `cargo test test_journal_recovery`
- Rollback        : `git revert`
- Critères d'acceptation :
  - Multi-step mutating operations record inverse compensatory actions in `/var/lib/server_manager/journal.jsonl` BEFORE executing.
  - Interrupted operations are automatically rolled back or completed upon next startup.

### REQ-OPS-003 — Standardized Exit Codes (sysexits.h)
- Priorité        : P1
- Statut          : MANDATORY
- Justification   : Intégration cohérente avec les superviseurs système (systemd, scripts d'orchestration).
- Dépendances     : aucune
- Prérequis env.  : aucun
- Risques         : aucun
- Preuves attendues: `server_manager/src/interface/cli.rs`
- Commandes de validation : `cargo test test_sysexits_codes`
- Rollback        : `git revert`
- Critères d'acceptation :
  - Application exits with standard codes: 0 (OK), 64 (USAGE), 65 (DATAERR), 66 (NOINPUT), 70 (SOFTWARE), 71 (OSERR), 73 (CANTCREAT), 75 (TEMPFAIL), 78 (CONFIGERR).

### REQ-OPS-004 — Non-Destructive Host Guarantee
- Priorité        : P0
- Statut          : MANDATORY
- Justification   : Interdiction absolue de purger ou détruire des ressources hôte non gérées par `server_manager`.
- Dépendances     : aucune
- Prérequis env.  : aucun
- Risques         : aucun
- Preuves attendues: Code source de `server_manager/src/core/`
- Commandes de validation : `cargo test test_non_destructive_guarantee`
- Rollback        : `git revert`
- Critères d'acceptation :
  - No indiscriminate global prune operations (`docker system prune -a --volumes` on non-managed resources, `ufw reset`, `userdel` on host users, `rm -rf /opt/*`).

### REQ-OPS-005 — Diagnostic Subcommand (`server_manager doctor`)
- Priorité        : P1
- Statut          : MANDATORY
- Justification   : Inspection rapide de la santé du système, du noyau, des dépendances et de la sécurité.
- Dépendances     : aucune
- Prérequis env.  : aucun
- Risques         : aucun
- Preuves attendues: Commande `server_manager doctor`
- Commandes de validation : `cargo test test_doctor_command`
- Rollback        : `git revert`
- Critères d'acceptation :
  - Reports status as `{OK, WARN, FAIL, SKIPPED+reason}`.
  - Checks Kernel version, Landlock availability and ABI version, cgroups v2, Docker daemon status, UFW/nftables state, port conflicts against listening sockets, disk usage, and NTP drift.
