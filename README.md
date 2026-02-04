# WikiTools (Native Edition)

**WikiTools** est un lanceur d'application pour Wiki.js, conçu pour fonctionner sans Docker, en utilisant le moteur de base de données partagé de l'écosystème CollabTools. Il permet une intégration native avec Windows pour ouvrir les fichiers locaux directement.

<p align="center">
  <img src="./assets/logo-wikijs.png" width="80" style="vertical-align: middle;" />
  &nbsp;<span style="font-size: 40px; font-weight: bold; vertical-align: middle;">+</span>&nbsp;
  <img src="./assets/logo-tauri.png" width="80" style="vertical-align: middle;" />
  &nbsp;<span style="font-size: 40px; font-weight: bold; vertical-align: middle;">=</span>&nbsp;
  <img src="./assets/logo-wikitools.png" width="100" style="vertical-align: middle;" />
</p>

## 🚀 Pré-requis

Pour fonctionner, WikiTools a besoin que le moteur de base de données **CollabTools** soit installé sur la machine.
*   Il utilise la configuration située dans `%APPDATA%\com.collabtools.core\postgresql`.
*   Il partage le même service PostgreSQL (Port `18246` par défaut).

## 📦 Installation

1.  Décompressez l'archive (ou placez le dossier `WikiTools FullNative`).
2.  Assurez-vous que les sous-dossiers suivants sont présents à côté de `WikiTools.exe` :
    *   `wiki/` (Le serveur Wiki.js Node.js)
    *   `postgresql/` (Les binaires PostgreSQL si besoin d'autonomie)

## 🛠️ Premier Démarrage & Configuration

1.  Lancez **`WikiTools.exe`**.
2.  L'application va :
    *   Détecter la configuration CollabTools.
    *   Démarrer le moteur PostgreSQL s'il est éteint.
    *   Créer automatiquement la base de données `wiki` si elle n'existe pas.
    *   Lancer le serveur Wiki.js.
3.  Sur l'écran d'installation de Wiki.js :
    *   **Administrator Email** : Votre email.
    *   **Password** : Votre mot de passe.
    *   **Site URL** : `http://localhost:3000` (Très important !).
4.  Cliquez sur **Install**. Le Wiki va redémarrer et vous serez redirigé vers la page de connexion.

## 🔗 Activation de l'Ouverture de Fichiers Natifs

Pour que WikiTools puisse ouvrir vos fichiers Word, Excel, PDF directement sur votre PC (au lieu de les télécharger), vous devez injecter un petit script dans l'administration de Wiki.js.

1.  Connectez-vous en **Admin** sur votre Wiki.
2.  Allez dans **Administration** (roue dentée) > **Code Injection**.
3.  Dans la case **Head**, collez le code suivant :

```html
<script>
document.addEventListener('click', function(e) {
    // 1. Vérifier si c'est un lien <a>
    var target = e.target.closest('a');
    if (!target) return;

    var href = target.getAttribute('href');
    if (!href) return;

    // 2. Liste des extensions à ouvrir nativement
    var extensions = ['.pdf', '.docx', '.xlsx', '.pptx', '.txt', '.csv', '.rtf', '.msg', '.eml'];
    var ext = href.substring(href.lastIndexOf('.')).toLowerCase();

    if (extensions.includes(ext)) {
        e.preventDefault(); // Bloquer le téléchargement navigateur
        console.log("[WikiTools] Interception du fichier :", href);

        // 3. Appeler WikiTools (Rust) directement
        if (window.__TAURI__ && window.__TAURI__.core) {
            window.__TAURI__.core.invoke('download_and_open', { url: href })
                .then(() => console.log("[WikiTools] Ouverture demandée avec succès"))
                .catch(err => alert("Erreur WikiTools : " + err));
        } else {
            // Fallback pour navigateur classique (Chrome/Edge)
            window.location.href = href;
        }
    }
});
</script>
```

4.  Cliquez sur **Apply** (en haut à droite).
5.  C'est fini ! Vos fichiers s'ouvriront désormais avec vos applications Windows par défaut.

## 🏗️ Architecture Technique

*   **Frontend :** React (pour le Launcher uniquement)
*   **Backend :** Rust (Tauri)
*   **Serveur Wiki :** Wiki.js (Node.js Process)
*   **Base de Données :** PostgreSQL 15+ (Géré via `postgres_manager` partagé avec GroundWorks).
