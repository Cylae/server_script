# Triage des branches — 13 septembre 2026

Base inspectée : `a6adc51a3d10fbcdddb5f4b8045b670947e82285`.
Sources : `git branch -r` et API GitHub `pulls?state=open&per_page=100`.
20 PR ouvertes et 20 branches de travail distantes à cette date. Les « 67 branches » du texte normatif sont un historique, pas l’inventaire actuel.

Ce triage exprime des recommandations issues du sujet et des écarts avec les contrats ; il ne certifie ni le diff complet ni les performances revendiquées de chaque PR. Aucune ancienne branche n’a été supprimée ou fusionnée dans cette mission.

| PR | Branche | Classement | Motif |
|---|---|---|---|
| #416 | `jules-5992953717060295136-253661ac` | à ignorer | Chevauchements sur les chemins/écritures ; une validation lexicale ne prouve pas le confinement des symlinks. À réexaminer après l’audit. |
| #415 | `jules-12468028964604981280-c35a1273` | à ignorer | Chevauchements sur les chemins/écritures ; une validation lexicale ne prouve pas le confinement des symlinks. À réexaminer après l’audit. |
| #414 | `fix/security-and-user-sorting-1430597682708282895` | à ignorer | Chevauchements sur les chemins/écritures ; une validation lexicale ne prouve pas le confinement des symlinks. À réexaminer après l’audit. |
| #413 | `jules-14022616465358029411-9f94621f` | à ignorer | Changement potentiellement utile, à revoir/tester dans son périmètre ; aucune fusion automatique de cette PR. |
| #412 | `perf/cache-hardware-detection-17336695570065704145` | à ignorer | Mémoriser RAM/disques définitivement rend la détection obsolète ; benchmark revendiqué non reproduit. |
| #411 | `fix/useradd-absolute-path-13098466155646312693` | à ignorer | Changement potentiellement utile, à revoir/tester dans son périmètre ; aucune fusion automatique de cette PR. |
| #410 | `perf/optimize-doctor-port-conflicts-13098466155646316055` | à ignorer | Changement potentiellement utile, à revoir/tester dans son périmètre ; aucune fusion automatique de cette PR. |
| #409 | `perf/telemetry-poller-throttling-9685045745597693274` | à ignorer | Changement potentiellement utile, à revoir/tester dans son périmètre ; aucune fusion automatique de cette PR. |
| #408 | `fix/integrate-validate-safe-path-10011583263562090100` | à ignorer | Chevauchements sur les chemins/écritures ; une validation lexicale ne prouve pas le confinement des symlinks. À réexaminer après l’audit. |
| #407 | `fix-command-injection-run-cli-toggle-2775550130301488844` | à ignorer | Changement potentiellement utile, à revoir/tester dans son périmètre ; aucune fusion automatique de cette PR. |
| #406 | `fix/atomic-io-insecure-tempfile-10188298294900936929` | à ignorer | Chevauchements sur les chemins/écritures ; une validation lexicale ne prouve pas le confinement des symlinks. À réexaminer après l’audit. |
| #405 | `perf/users-list-alloc-optimization-16929115810620791053` | à ignorer | Changement potentiellement utile, à revoir/tester dans son périmètre ; aucune fusion automatique de cette PR. |
| #404 | `fix/remove-unused-validate-domain-2958005753390998787` | à fermer | Retire des API publiques d’une bibliothèque sous prétexte de code mort ; absence de justification fonctionnelle. |
| #403 | `code-health/remove-unused-validate-port-str-7597355792170492956` | à fermer | Retire des API publiques d’une bibliothèque sous prétexte de code mort ; absence de justification fonctionnelle. |
| #402 | `fix/journal-recovery-test-13226370120388620467` | à ignorer | Changement potentiellement utile, à revoir/tester dans son périmètre ; aucune fusion automatique de cette PR. |
| #401 | `remove-unused-is-public-portmapping-9151569566518202775` | à fermer | Retire des API publiques d’une bibliothèque sous prétexte de code mort ; absence de justification fonctionnelle. |
| #400 | `test-check-root-privilege-1541694811706861378` | à ignorer | Changement potentiellement utile, à revoir/tester dans son périmètre ; aucune fusion automatique de cette PR. |
| #399 | `fix/remove-unused-generate-op-id-1337842746933228318` | à fermer | Retire des API publiques d’une bibliothèque sous prétexte de code mort ; absence de justification fonctionnelle. |
| #398 | `test/port-mapping-error-handling-16855835431767212044` | à ignorer | Changement potentiellement utile, à revoir/tester dans son périmètre ; aucune fusion automatique de cette PR. |
| #397 | `jules-16231249243070530309-c9278a62` | à ignorer | Priorité secondaire ; à revalider après les corrections fonctionnelles. |

Branches restantes : `origin/main` (référence, à ignorer), `origin/HEAD` (alias).
