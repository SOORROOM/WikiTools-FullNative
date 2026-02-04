# Documentation Technique & Guide d'Installation - WikiTools

## 1. À propos de la solution
**WikiTools** est une solution packagée "clé en main" permettant de déployer un Wiki d'entreprise (basé sur le moteur Wiki.js) sans complexité technique pour l'utilisateur final.

Elle combine :
1.  **Fiabilité** : Un conteneur Docker **PostgreSQL** (Base de données) et **Wiki.js** (Serveur Web).
2.  **Simplicité** : Une application "Lanceur" (`WikiTools.exe`) qui gère tout le cycle de vie :
    *   Vérification et démarrage automatique de Docker Desktop.
    *   Montage des conteneurs via Docker Compose.
    *   **Health Check intelligent :** Attend que l'application soit réellement disponible (Code 200 via API) avant d'afficher l'interface, évitant les erreurs de connexion au démarrage.
3.  **Intégration Windows** : Une fonctionnalité unique permettant d'ouvrir les fichiers bureautiques (Word, Excel, PDF) directement dans leurs applications natives depuis le navigateur.

---

## 2. Architecture et Prérequis
La solution est conçue pour tourner sur une machine Windows (Poste de travail ou Serveur).

### Prérequis Système
*   **OS :** Windows 10/11 ou Windows Server 2019/2022.
*   **Moteur :** Docker Desktop doit être installé.
*   **Virtualisation :** Si installé sur une Machine Virtuelle (VM), l'option **"Nested Virtualization"** (Virtualisation Imbriquée) doit être activée sur l'hyperviseur hôte.

### Architecture des Données
Les données ne sont **pas** enfermées dans le système.
Elles sont accessibles physiquement dans le dossier du projet :
*   `./data/postgres` : Base de données brute.
*   *Note : Ce dossier est à inclure dans vos sauvegardes quotidiennes (Veeam, etc.).*

---

## 3. Guide d'Installation (Pas à Pas)

### Étape 1 : Déploiement
1.  Copier le dossier `WikiTools` complet sur le disque du serveur (ex: `C:\Apps\WikiTools`).
2.  S'assurer que Docker Desktop est installé.

### Étape 2 : Premier Lancement
1.  Lancer l'exécutable **`WikiTools.exe`**.
2.  L'application va automatiquement :
    *   Vérifier la présence de Docker.
    *   Démarrer le moteur Docker s'il est éteint.
    *   Monter les conteneurs (Base de données + Wiki).
3.  Une fenêtre s'ouvrira sur l'assistant d'installation Wiki.js.
4.  Compléter l'installation :
    *   **Admin Email/Password** : Définir vos accès.
    *   **Site URL** : Laisser impérativement `http://localhost:3000`.

### Étape 3 : Activation de l'Ouverture Native (Script Listener)
Pour que les utilisateurs puissent ouvrir les fichiers Word/Excel directement sans les télécharger, une configuration unique est requise.

1.  Ouvrir le fichier `SCRIPT_INJECTION.txt` présent dans le dossier.
2.  Copier l'intégralité du contenu.
3.  Dans le Wiki, aller dans **Administration** (Roue dentée) > **Code Injection**.
4.  Coller le script dans la zone **"Head HTML"**.
5.  Cliquer sur **Apply** (en haut à droite).

*Le Wiki est maintenant capable de communiquer avec l'application Windows pour ordonner l'ouverture des fichiers.*

---

## 4. Maintenance & Réseau

### Accès Multi-utilisateurs
Si cette installation est faite sur un serveur, les autres utilisateurs peuvent accéder au Wiki via un navigateur classique (Chrome/Edge) à l'adresse :
> `http://IP-DU-SERVEUR:3000`

*Note : L'ouverture native des fichiers ne fonctionne que pour l'utilisateur utilisant l'application `WikiTools.exe`. Les utilisateurs web classiques auront un comportement de téléchargement standard.*

### Sauvegarde
Pour sauvegarder l'intégralité du Wiki (Logiciel + Données + Configuration), il suffit de copier/sauvegarder le dossier `WikiTools` entier.
