# 05-DELIVERY.md — Delivery Workflow, PR Rules & Final Audit Report

### REQ-DEL-001 — One Gate Per Branch Per PR
- Priorité        : P0
- Statut          : MANDATORY
- Justification   : Assurer la traçabilité, l'isolation des risques et des rollbacks atomiques par `git revert`.
- Dépendances     : aucune
- Prérequis env.  : git
- Risques         : aucun
- Preuves attendues: Branches nommées `gate/<NN>-<slug>` et PRs associées.
- Commandes de validation : `git branch -a`
- Rollback        : `git revert <merge-sha>`
- Critères d'acceptation :
  - Each gate is developed on a dedicated branch and delivered in a single PR.
  - Net diff does not exceed 1500 lines per PR (excluding generated files).

### REQ-DEL-002 — Structured Pull Request Template
- Priorité        : P0
- Statut          : MANDATORY
- Justification   : Documentation obligatoire des exigences couvertes, des preuves de test, des risques et de la procédure de rollback.
- Dépendances     : REQ-DEL-001
- Prérequis env.  : aucun
- Risques         : aucun
- Preuves attendues: Description de chaque PR alignée sur le modèle structuré.
- Commandes de validation : Inspection visuelle des PRs
- Rollback        : N/A
- Critères d'acceptation :
  - PR descriptions contain: Gate & Scope, Covered Requirements (REQ-...), Evidence (commands + exit codes + excerpts), Risks, Exact Rollback Procedure, Visible Behavior Changes, Out-of-Scope Items.

### REQ-DEL-003 — Conventional Commits & Atomic History
- Priorité        : P1
- Statut          : MANDATORY
- Justification   : Historique lisible, bisectable et intégration automatisée des changelogs.
- Dépendances     : aucune
- Prérequis env.  : git
- Risques         : aucun
- Preuves attendues: Output de `git log --oneline`
- Commandes de validation : `git log`
- Rollback        : N/A
- Critères d'acceptation :
  - Commits strictly follow Conventional Commits format (`feat:`, `fix:`, `docs:`, `test:`, `refactor:`, `chore:`).
  - Every commit in history builds and passes tests independently.

### REQ-DEL-004 — Final Audit Outcome Report Structure
- Priorité        : P0
- Statut          : MANDATORY
- Justification   : Fournir un rapport final d'audit transparent et factuel à l'issue de l'exécution de toutes les gates.
- Dépendances     : REQ-DEL-001
- Prérequis env.  : aucun
- Risques         : aucun
- Preuves attendues: Rapport final produit après la dernière gate.
- Commandes de validation : N/A
- Rollback        : N/A
- Critères d'acceptation :
  - Final report contains sections: Audit outcome, Repository scope inspected, Findings fixed, Reviewed but not changed, Architectural changes, Technology evaluation, Files changed, Validation results, Remaining limitations, Confidence assessment.
