# Cylae: The State-of-the-Art Media Server Orchestrator 🚀

**Cylae** is a next-generation infrastructure-as-code tool written in Rust. It automatically detects your hardware and compiles a tailored `docker-compose.yml` stack for your media server needs.

---

## 🇺🇸 English Guide

### 👶 New Users
Welcome! Cylae makes setting up a media server incredibly easy.

1.  **Download & Run**
    ```bash
    # Assuming you have the binary
    sudo ./cylae install
    ```
    *That's it!* Cylae will:
    *   ✅ Detect your RAM and CPU.
    *   ✅ Install Docker (if missing).
    *   ✅ Configure your firewall.
    *   ✅ Start Plex, Sonarr, Radarr, etc.

2.  **Access Your Services**
    *   **Plex:** `http://<your-ip>:32400`
    *   **Sonarr:** `http://<your-ip>:8989`
    *   **Radarr:** `http://<your-ip>:7878`
    *   **qBittorrent:** `http://<your-ip>:8080` (Default: admin/adminadmin)

### 🤓 Advanced Users
For power users who want control and understanding.

*   **Idempotency:** You can run `cylae install` as many times as you want. It will only apply necessary changes.
*   **Security:** Database passwords are automatically generated and stored in `/opt/cylae/secrets.yaml`.
*   **Hardware Profiles:**
    *   **Low (<4GB RAM):** Disables .NET diagnostics, optimizes GC, uses disk for transcoding.
    *   **High (>16GB RAM):** Enables RAM transcoding (`/dev/shm`), maximizes buffer pools.
*   **GPU Passthrough:** automatically detects Nvidia drivers (`nvidia-smi`) or Intel QuickSync (`/dev/dri`) and injects the devices into the Plex container.
*   **Commands:**
    *   `cylae status`: View detected hardware and docker status.
    *   `cylae generate`: Only generate the `docker-compose.yml` without running it.

---

## 🇫🇷 Guide Français

### 👶 Nouveaux Utilisateurs
Bienvenue ! Cylae rend l'installation d'un serveur multimédia incroyablement simple.

1.  **Télécharger et Exécuter**
    ```bash
    sudo ./cylae install
    ```
    *C'est tout !* Cylae va :
    *   ✅ Détecter votre RAM et CPU.
    *   ✅ Installer Docker (si absent).
    *   ✅ Configurer votre pare-feu.
    *   ✅ Démarrer Plex, Sonarr, Radarr, etc.

2.  **Accéder à vos Services**
    *   **Plex :** `http://<votre-ip>:32400`
    *   **Sonarr :** `http://<votre-ip>:8989`
    *   **Radarr :** `http://<votre-ip>:7878`

### 🤓 Utilisateurs Avancés
Pour les experts qui veulent comprendre et contrôler.

*   **Idempotence :** Vous pouvez exécuter `cylae install` autant de fois que vous le souhaitez.
*   **Profils Matériels :**
    *   **Faible (<4Go RAM) :** Désactive les diagnostics .NET, optimise le GC, utilise le disque pour le transcodage.
    *   **Élevé (>16Go RAM) :** Active le transcodage en RAM (`/dev/shm`), maximise les pools de mémoire tampon.
*   **Accélération GPU :** Détecte automatiquement les pilotes Nvidia ou Intel QuickSync et injecte les périphériques dans le conteneur Plex.
*   **Commandes :**
    *   `cylae status` : Voir le matériel détecté et l'état de Docker.
    *   `cylae generate` : Générer uniquement le fichier `docker-compose.yml`.
