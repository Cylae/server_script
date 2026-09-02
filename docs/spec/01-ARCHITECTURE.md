# 01-ARCHITECTURE.md — System Architecture & Modular Layout

## 4.1 Invariants d'architecture

1. **Frontière de confiance explicite**: `CLI args | fichiers YAML | requêtes HTTP | variables d'env` sont non fiables ; `server_manager/src/core/*` est le seul module habilité à muter le système ; aucun module de `src/interface/` ni `src/services/` n'appelle `Command::new` directement.
2. **Abstractions par traits**: Toute opération privilégiée passe par un trait (`SystemOps`, `DockerOps`, `FirewallBackend`, `UserOps`) pour être pleinement testable sans privilèges root, sans daemon Docker actif et sans connectivité réseau.
3. **Absence d'état global mutable**: Aucun état global mutable ne doit exister ; aucun effet de bord ne doit se produire lors de l'initialisation ou de l'import d'un module.
4. **Catalogue de services comme source unique de vérité**: Le catalogue de services est une structure de données typée, pas du code impératif dupliqué : un port ou paramètre n'est déclaré qu'une seule fois et toute documentation/matrice en est directement dérivée.

---

## 4.2 Contrat établi à préserver

Le contrat de l'application est documenté et figé dans `docs/audit/CONTRACT.md`:
- Surface CLI (`install`, `apply`, `update`, `clean`, `fix`, `interactive`, `status`, `web`, `user add|delete|list|passwd`).
- Codes de sortie conformes aux conventions sysexits.h.
- Emplacements de persistance et schémas YAML (`config.yaml`, `secrets.yaml`, `users.yaml`, `docker-compose.yml`).
- Catalogue des 28 services intégrés et leur matrice de ports associés.
- Seuils de configuration matériel (profils LOW, STANDARD, HIGH).
- Web administration UI sur le port 8099.

---

## 4.3 Structure des modules (dans `server_manager/src/core/`)

- `hardware.rs` — Détection télémétrique du matériel (CPU, RAM, Disques, GPU).
- `system.rs` — Gestion des paquets système et services (fail2ban, apt).
- `docker.rs` — Abstraction du daemon et des conteneurs Docker.
- `compose.rs` — Génération et orchestration des piles Docker Compose.
- `config.rs` — Structure et chargement de la configuration système.
- `secrets.rs` — Gestion sécurisée des secrets et mots de passe.
- `users.rs` — Gestion des utilisateurs, rôles (Admin/Observer/Operator/Auditor) et quotas.
- `firewall.rs` — Automations UFW et backend optionnel nftables.
- `atomic_io.rs` — [Nouveau] Écritures atomiques (tmpfile + fsync + rename + permissions 0600/0644).
- `lock.rs` — [Nouveau] Verrouillage consultatif inter-processus (`flock`).
- `journal.rs` — [Nouveau] Journal d'opérations compensatoires et moteur de rollback.
- `sandbox.rs` — [Nouveau] Isolement Landlock best-effort.
- `validate.rs` — [Nouveau] Validateurs stricts pour entrées, nommages et chemins.
