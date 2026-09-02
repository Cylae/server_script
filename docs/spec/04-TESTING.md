# 04-TESTING.md — Testing Strategy, Criteria & Verification

### REQ-TST-001 — Test Criteria T1: Branch Coverage Floor
- Priorité        : P1
- Statut          : MANDATORY
- Justification   : Garantir qu'au moins 75 % des branches logiques dans `src/core/` et `src/services/` sont vérifiées par automated tests.
- Dépendances     : aucune
- Prérequis env.  : `cargo-llvm-cov`
- Risques         : aucun
- Preuves attendues: Rapport `cargo llvm-cov --branch`
- Commandes de validation : `cargo llvm-cov --branch --fail-under-lines 75`
- Rollback        : `git revert`
- Critères d'acceptation :
  - Branch coverage on `src/core/` and `src/services/` is ≥ 75%.

### REQ-TST-002 — Test Criteria T2: Contract-Based Test Suite
- Priorité        : P0
- Statut          : MANDATORY
- Justification   : Chaque comportement établi doit posséder au moins un test explicite.
- Dépendances     : aucune
- Prérequis env.  : aucun
- Risques         : aucun
- Preuves attendues: Tests nommés `contract_<domaine>_<comportement>` dans `server_manager/tests/`.
- Commandes de validation : `cargo test contract_`
- Rollback        : `git revert`
- Critères d'acceptation :
  - Comprehensive suite of `contract_*` tests verifying all core requirements.

### REQ-TST-003 — Test Criteria T3: Zero Regression Defect Tests
- Priorité        : P0
- Statut          : MANDATORY
- Justification   : Tout défaut corrigé ajoute un test de non-régression dédié.
- Dépendances     : aucune
- Prérequis env.  : aucun
- Risques         : aucun
- Preuves attendues: Tests nommés `regression_<id_finding>`
- Commandes de validation : `cargo test regression_`
- Rollback        : `git revert`
- Critères d'acceptation :
  - Every fixed issue includes a corresponding `regression_*` test.

### REQ-TST-004 — Test Criteria T4: Mutation Testing (Opt-In / CI Goal)
- Priorité        : G
- Statut          : OPTIONAL
- Justification   : Évaluation de la qualité réelle des assertions par mutation de code.
- Dépendances     : aucune
- Prérequis env.  : `cargo-mutants`
- Risques         : aucun
- Preuves attendues: Output de `cargo mutants`
- Commandes de validation : `cargo mutants --dir server_manager`
- Rollback        : N/A
- Critères d'acceptation :
  - Mutation score target ≥ 60% on `src/core/` (non-blocking in CI).

### REQ-TST-005 — Unified Verification Script (`./verify.sh`)
- Priorité        : P0
- Statut          : MANDATORY
- Justification   : `./verify.sh` doit être l'exact reflet du job CI sur runner propre, sans divergence.
- Dépendances     : aucune
- Prérequis env.  : bash, rustup, cargo
- Risques         : aucun
- Preuves attendues: Script `./verify.sh` à la racine du dépôt.
- Commandes de validation : `./verify.sh`
- Rollback        : `git revert`
- Critères d'acceptation :
  - `./verify.sh` runs `cargo fmt --check`, `cargo clippy -D warnings`, and `cargo test` targeting `server_manager/Cargo.toml`.
  - Exits with return code 0 when all checks pass.
