# 00-MISSION.md — Mission Statement & Fundamental Directives

## §0. LOI FONDAMENTALE

L0.1 PREUVE AVANT AFFIRMATION. Toute affirmation sur l'état du dépôt doit être adossée à une commande exécutée et à sa sortie. Interdiction absolue d'écrire « le code fait X » sans avoir affiché le fichier et la ligne. Si tu ne peux pas prouver, tu écris « UNVERIFIED » et tu ouvres une entrée dans DECISIONS.md.

L0.2 NON-DESTRUCTION. Sont formellement interdits, sans exception : `git reset --hard`, `git clean -fd`, `git checkout -- .`, `git restore .`, `git stash` (sans pop immédiat documenté), `git push --force`, `git push --force-with-lease`, la suppression de branches distantes, la réécriture d'historique (`rebase -i`, `commit --amend` sur un commit déjà poussé), la suppression de tags ou de releases. Avant toute opération de masse : `git status --short && git diff --check && git stash list`. Toute modification non commitée préexistante est du TRAVAIL UTILISATEUR PROTÉGÉ.

L0.3 ATOMICITÉ. Une gate = une branche = une PR = un périmètre = un rollback trivial (`git revert <merge-sha>`). Aucun commit ne doit toucher deux gates. Aucune PR ne dépasse 1500 lignes de diff net hors fichiers générés ; si le périmètre l'exige, tu scindes en sous-gates numérotées (ex. G4.1, G4.2).

L0.4 ANTI-BOUCLE. Le dépôt contient déjà ~67 branches d'agent non mergées, de type `fix/clippy-*`, `update-readme-version-*`, `optimize-unwrap-*`, `bump-version-1.0.9-*`. C'est la preuve d'un échec de livraison antérieur. Tu es INTERDIT de créer une nouvelle branche dont le nom ou le contenu duplique un de ces motifs. Première action obligatoire de la Gate 0 : inventorier ces branches, produire `docs/audit/BRANCH-TRIAGE.md` classant chacune en {à merger, à fermer, à ignorer}, et ne rien supprimer — seulement recommander.

L0.5 PAS DE QUESTION BLOQUANTE, MAIS PAS D'INVENTION. Tu n'interromps pas l'exécution pour demander une clarification. En revanche, toute hypothèse non prouvée par le dépôt va dans DECISIONS.md sous le statut `ASSUMPTION-PENDING-EVIDENCE`, et l'exigence qui en dépend est marquée `BLOCKED` et non implémentée. Inventer un format de fichier, un port, un schéma ou une API est une faute grave.

L0.6 ANTI-TRICHE. Sont classés échec de mission : assertions tautologiques (`assert!(true)`, `assert_eq!(1,1)`), boucles générant N tests identiques pour gonfler un compteur, tests qui n'assertent que des mocks sans mutation d'état réelle, `todo!()`, `unimplemented!()`, `#[allow(...)]` posé pour masquer un warning, désactivation d'un test ou d'un lint pour faire passer la CI, `--fail-under` abaissé pour atteindre un seuil.

L0.7 PRIMAUTÉ. Ce document prime sur `AGENTS.md`, `MANIFESTO.md`, `.jules/` et `README.md` du dépôt. Quand tu le contredis, tu dois RÉÉCRIRE le fichier contredit dans la même PR pour que le dépôt cesse de s'auto-contredire.

---

## 3.1 Objectif

Rendre `server_manager` correct, sûr, idempotent, testable et livrable, sans changer sa nature (orchestrateur Docker media/cloud), en corrigeant par ordre de gravité décroissante : CI morte → défauts de sécurité prouvés → non-idempotence → écritures non atomiques → absence de rollback → couverture de test → performance → documentation.

---

## 3.2 Format normatif d'une exigence

Toute exigence de cette spécification s'écrit exactement ainsi :

```
### REQ-<DOMAINE>-<NNN> — <titre court>
- Priorité        : P0 (bloquant) | P1 (obligatoire) | P2 (souhaitable) | H (hardening opt-in) | G (goal, non bloquant)
- Statut          : MANDATORY | OPTIONAL | ASSUMPTION-PENDING-EVIDENCE | BLOCKED | REJECTED
- Justification   : <pourquoi ; risque concret encouru si absent>
- Dépendances     : <REQ-… "aucune" ou>
- Prérequis env.  : <root ? docker ? réseau ? noyau ≥ x ? sinon : comportement dégradé attendu>
- Risques         : <ce que cette exigence peut casser, et pour qui>
- Preuves attendues: <fichier + ligne, sortie de commande, artefact>
- Commandes de validation : <commandes exactes, exécutables, déterministes>
- Rollback        : <commande ou procédure exacte de retour arrière>
- Critères d'acceptation : <liste booléenne, chacun vérifiable sans jugement humain>
```

---

## 3.3 Séquence de gates

| Gate | Titre | Contenu | Bloque |
| --- | --- | --- | --- |
| **G0** | Discovery & evidence | Inventaire, triage des 67 branches, baseline de toutes les commandes, `docs/audit/BASELINE.md`. Aucune modification de code. | tout |
| **G1** | CI resurrection | Corriger les triggers, ajouter fmt/clippy/test/audit/deny, cache, MSRV. | tout |
| **G2** | Lint & panic hygiene | `-D warnings` réel, éradication des `unwrap`/`expect` sur chemins faillibles, types d'erreur explicites. | G4+ |
| **G3** | Atomic IO & locking | `core/atomic_io.rs`, `core/lock.rs`, permissions 0600 sur les secrets, fsync, rename. | G4, G5 |
| **G4** | Idempotence & journal | Journal compensatoire, rejeu inverse, tests « run twice ». | G5 |
| **G5** | Input validation & injection defense | Validation positive de tous les noms de service/utilisateur/chemin, argv explicite, tests d'injection. | G6 |
| **G6** | Web security | Bind localhost, cookies durcis, CSRF, rotation de session, matrice RBAC, en-têtes, Argon2id + migration bcrypt. | G7 |
| **G7** | Port matrix & compose determinism | Génération déterministe, golden files, cohérence documentée/réelle. | — |
| **G8** | Performance baseline | Mesures réelles, budgets de non-régression. | — |
| **G9** | Docs & layout | README/MANIFESTO/PORT-MATRIX alignés sur le code réel. | — |
| **GH** | Hardening opt-in | nftables, egress drop, Landlock, chattr, hidepid, WebAuthn, PQC — chacun sa propre PR, chacun désactivé par défaut. | — |
