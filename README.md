# WikiTools (Native Edition)

**WikiTools** est une solution "clé en main" pour déployer un Wiki personnel ou d'équipe (basé sur **Wiki.js**) sur un poste Windows, sans aucune compétence technique requise.

Il transforme Wiki.js en une véritable application de bureau, capable d'ouvrir vos documents locaux (PDF, Word, Excel...) directement avec vos applications installées, ce qu'un navigateur web classique ne peut pas faire.

<p align="center">
  <img src="./assets/logo-wikijs.png" width="80" style="vertical-align: middle;" />
  &nbsp;<span style="font-size: 40px; font-weight: bold; vertical-align: middle;">+</span>&nbsp;
  <img src="./assets/logo-tauri.png" width="80" style="vertical-align: middle;" />
  &nbsp;<span style="font-size: 40px; font-weight: bold; vertical-align: middle;">=</span>&nbsp;
  <img src="./assets/logo-wikitools.png" width="100" style="vertical-align: middle;" />
</p>

## 🌟 Points Forts

*   **100% Autonome** : Embarque son propre moteur de base de données. Rien à installer à part WikiTools.
*   **Intégration Windows** : Vos fichiers bureautiques s'ouvrent instantanément (pas de téléchargement dans le dossier "Téléchargements").
*   **Compatible CollabTools** : Si vous utilisez la suite **CollabTools** *(Produit à venir)*, WikiTools détectera automatiquement le moteur partagé pour économiser les ressources de votre PC.

## 📦 Installation

1.  Téléchargez et lancez l'installateur `WikiTools_Setup.exe`.
2.  Laissez-vous guider.
3.  Une fois installé, lancez **WikiTools** depuis votre bureau.

## 🛠️ Premier Démarrage

Lors du tout premier lancement, une fenêtre d'aide apparaîtra pour vous guider dans la configuration initiale de Wiki.js.

⚠️ **Point Critique :**
Sur la page d'installation, à la ligne **Site URL**, vous devez impérativement entrer :
> `http://localhost:3000`

C'est la condition sine qua non pour que l'ouverture des fichiers locaux fonctionne.

## 🏗️ Architecture Technique

WikiTools est conçu pour être léger et performant. Contrairement aux installations classiques de Wiki.js qui nécessitent Docker ou un serveur dédié, WikiTools utilise :

*   **Frontend :** React (Launcher)
*   **Backend :** Rust (Tauri) pour la performance et l'intégration système.
*   **Serveur Wiki :** Processus Node.js natif (embarqué).
*   **Base de Données :** PostgreSQL 15+ (Mode Hybride : Autonome ou Partagé si CollabTools est présent).

---
*WikiTools - L'outil de documentation simple et puissant.*
