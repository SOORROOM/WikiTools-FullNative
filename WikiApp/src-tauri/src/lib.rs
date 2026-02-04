use std::process::Command;
use std::time::Duration;
use tauri::Manager;
use std::sync::Mutex;
use std::path::PathBuf;
use std::os::windows::process::CommandExt;

mod postgres_manager;
use postgres_manager::PostgresManager;

struct AppState {
    postgres_manager: Mutex<Option<PostgresManager>>,
    wiki_process: Mutex<Option<std::process::Child>>,
}

#[tauri::command]
async fn init_db(app_handle: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<String, String> {
    println!("🔌 Initialisation de la Base de Données...");
    
    // 1. Chercher la config CollabTools (Roaming/com.collabtools.core/postgresql/db_config.json)
    let roaming = std::env::var("APPDATA").map_err(|_| "Impossible de trouver AppData".to_string())?;
    let collab_config_path = PathBuf::from(roaming.clone())
        .join("com.collabtools.core")
        .join("postgresql")
        .join("db_config.json");
        
    let resource_dir = app_handle.path().resource_dir().map_err(|e| e.to_string())?;

    let mut pm = if collab_config_path.exists() {
        println!("✅ Configuration CollabTools trouvée à : {:?}", collab_config_path);
        PostgresManager::from_existing_config(collab_config_path, resource_dir, "wiki")?
    } else {
        println!("⚠️ Pas de CollabTools détecté. Passage en mode Autonome.");
        
        // Mode Autonome : On utilise notre propre AppData
        // AppData/Roaming/com.wikitools.app/postgresql
        let wiki_app_data = PathBuf::from(roaming).join("com.wikitools.app");
        
        // Créer le dossier s'il n'existe pas
        if !wiki_app_data.exists() {
             std::fs::create_dir_all(&wiki_app_data).map_err(|e| format!("Impossible de créer AppData: {}", e))?;
        }

        let mut standalone_pm = PostgresManager::new(wiki_app_data, resource_dir)
            .map_err(|e| format!("Echec init manager autonome: {}", e))?;
            
        // En mode autonome, on s'assure d'initialiser (initdb) si c'est la toute première fois
        standalone_pm.init_database().map_err(|e| format!("Echec initdb autonome: {}", e))?;
            
        standalone_pm
    };

    println!("🔄 Tentative de démarrage du Manager PostgreSQL...");
    // 2. Démarrer / Vérifier Postgre
    pm.start().map_err(|e| format!("Erreur start(): {}", e))?;
    println!("✅ Manager démarré (ou déjà running).");
    
    // 3. Créer la DB 'wiki' si nécessaire
    println!("🔄 Vérification de la base de données 'wiki'...");
    pm.ensure_database_exists().map_err(|e| format!("Erreur ensure_db(): {}", e))?;
    println!("✅ Base 'wiki' validée.");
    
    // Stocker le manager dans l'état
    *state.postgres_manager.lock().unwrap() = Some(pm);
    
    Ok("Base de données prête.".to_string())
}

#[tauri::command]
async fn start_wiki_server(app_handle: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<String, String> {
    println!("🚀 Démarrage du Serveur Wiki.js...");
    
    // Vérifier si déjà lancé
    let mut procs = state.wiki_process.lock().unwrap();
    if procs.is_some() {
        return Ok("Déjà lancé".to_string());
    }

    // Chemin vers le dossier 'wiki' (à côté de l'exe)
    // En prod, l'exe est à la racine, 'wiki' est à côté.
    // En dev, on est dans src-tauri... attention.
    
    // Astuce : En mode Dev (tauri dev), le current_dir est souvent src-tauri.
    // En mode Prod, c'est le dossier de l'exe.
    
    let current_dir = std::env::current_dir().map_err(|e| e.to_string())?;
    
    // On cherche 'wiki' à plusieurs endroits possibles
    let candidates = vec![
        current_dir.join("wiki"),          // Prod
        current_dir.join("../wiki"),       // Dev (depuis src-tauri ou WikiApp)
        current_dir.join("../../wiki"),    // Dev (depuis src-tauri/src ?)
    ];
    
    let wiki_dir = candidates.into_iter().find(|p| p.exists())
        .ok_or(format!("Dossier 'wiki' introuvable. Cherché dans : {:?}", current_dir))?;


    // Configurer l'environnement (notamment le port DB si besoin, mais c'est dans config.yml)
    // On lance "node server"
    let child = Command::new("node")
        .arg("server")
        .current_dir(&wiki_dir)
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .spawn()
        .map_err(|e| format!("Impossible de lancer Node.js: {}", e))?;
        
    *procs = Some(child);
    
    println!("✅ Wiki.js démarré en tâche de fond.");
    Ok("Wiki lancé".to_string())
}

#[tauri::command]
async fn check_health() -> bool {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap_or_default();
        
    match client.get("http://localhost:3000").send().await {
        Ok(res) => res.status().is_success(),
        Err(_) => false,
    }
}

#[tauri::command]
async fn download_and_open(url: String) -> Result<(), String> {
    let filename = url.split('/').last().unwrap_or("document.bin");
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(filename);

    // Gérer les URLs relatives
    let full_url = if url.starts_with("http") { 
        url 
    } else if url.starts_with("/") {
        format!("http://localhost:3000{}", url)
    } else {
        format!("http://localhost:3000/{}", url)
    };

    println!("📥 Téléchargement de : {}", full_url);

    // Utilisation ASYNC pour ne pas bloquer/paniquer le runtime
    let response = reqwest::get(&full_url).await.map_err(|e| e.to_string())?;
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    
    // Écriture synchrone acceptable ici (ou utiliser tokio::fs)
    std::fs::write(&file_path, bytes).map_err(|e| e.to_string())?;

    println!("📂 Ouverture native : {:?}", file_path);
    open::that(&file_path).map_err(|e| e.to_string())?;
    Ok(())
}

fn start_local_command_server() {
    std::thread::spawn(|| {
        // Port inhabituel pour éviter les conflits
        match std::net::TcpListener::bind("127.0.0.1:45678") {
            Ok(listener) => {
                println!("🚀 Serveur Commandes Local démarré sur :45678");
                for stream in listener.incoming() {
                    match stream {
                        Ok(mut stream) => {
                            std::thread::spawn(move || {
                                let mut buffer = [0; 2048]; // Buffer suffisant pour URL longue
                                use std::io::Read;
                                use std::io::Write;
                                
                                if let Ok(n) = stream.read(&mut buffer) {
                                    if n > 0 {
                                        let request = String::from_utf8_lossy(&buffer[..n]);
                                        // On cherche "GET /open?url="
                                        if let Some(start_idx) = request.find("GET /open?url=") {
                                            let rest = &request[start_idx + 14..];
                                            if let Some(end_idx) = rest.find(" HTTP") {
                                                let encoded_url = &rest[..end_idx];
                                                let decoded = urlencoding::decode(encoded_url).unwrap_or_default().to_string();
                                                println!("📡 Commande reçue : Open {}", decoded);
                                                
                                                // Lancer l'action via le runtime Tauri
                                                tauri::async_runtime::spawn(async move {
                                                    let _ = download_and_open(decoded).await;
                                                });
                                            }
                                        }
                                    }
                                }

                                // Réponse image PNG vide 1x1 base64 pour que le navigateur soit content (si utilisé en <img src>)
                                // Ou juste 200 OK avec CORS
                                let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: 2\r\n\r\nOK";
                                let _ = stream.write_all(response.as_bytes());
                            });
                        }
                        Err(e) => eprintln!("Erreur connexion: {}", e),
                    }
                }
            }
            Err(e) => eprintln!("❌ Impossible de démarrer le serveur de commandes : {}", e),
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|_app| {
            // Démarrer notre backend de secours
            start_local_command_server();
            Ok(())
        })

        .manage(AppState {
            postgres_manager: Mutex::new(None),
            wiki_process: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            init_db, 
            start_wiki_server, 
            check_health, 
            download_and_open
        ])
        .on_page_load(|window, _| {
            let injection_script = r#"
                // 1. Gestionnaire de CLICS pour Téléchargement Natif
                if (!window.wt_click_handler) {
                    window.wt_click_handler = true;
                    document.addEventListener('click', function(e) {
                        var target = e.target.closest('a');
                        if (!target) return;
                        var href = target.getAttribute('href');
                        if (!href) return;
                        var extensions = ['.pdf', '.docx', '.xlsx', '.pptx', '.txt', '.csv', '.rtf', '.msg', '.eml'];
                        var ext = href.substring(href.lastIndexOf('.')).toLowerCase();
                        
                        // Détection des extensions
                        if (extensions.includes(ext)) {
                            e.preventDefault();
                            console.log("WikiTools: Appel au serveur local pour", href);
                            // Appel au serveur local (Plan G)
                            fetch('http://127.0.0.1:45678/open?url=' + encodeURIComponent(href))
                                .catch(err => console.error("Echec appel serveur local:", err));
                        }
                    });
                }

                // MODAL D'AIDE A L'INSTALLATION & CONFIGURATION
                function checkAndInjectModal() {
                    // L'URL reste sur / lors du setup, donc on détecte le contenu de la page
                    // On cherche "Administrator Email" et "Site URL" qui sont spécifiques à l'install
                    const isSetupPage = document.body.innerText.includes("Administrator Email") && document.body.innerText.includes("Site URL");
                    
                    
                    if (isSetupPage && !document.getElementById("wt-helper-modal") && !window.wt_modal_dismissed) {
                        console.log("WikiTools: Page d'installation détectée (via contenu). Injection du modal.");
                        
                        const style = document.createElement('style');
                        style.id = "wt-helper-style";
                        style.innerHTML = `
                            .wt-modal-overlay { position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(15, 23, 42, 0.95); z-index: 2147483647; display: flex; justify-content: center; align-items: center; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; backdrop-filter: blur(5px); }
                            .wt-modal-content { background: white; width: 650px; max-width: 90%; border-radius: 16px; padding: 40px; box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25); text-align: left; color: #334155; animation: wt-fadein 0.3s ease-out; }
                            @keyframes wt-fadein { from { opacity: 0; transform: scale(0.95); } to { opacity: 1; transform: scale(1); } }
                            .wt-title { font-size: 28px; font-weight: 800; color: #0f172a; margin-bottom: 25px; border-bottom: 3px solid #3b82f6; padding-bottom: 15px; }
                            .wt-section { margin-bottom: 25px; }
                            .wt-info { background: #eff6ff; border-left: 5px solid #3b82f6; padding: 15px; margin: 15px 0; font-size: 16px; line-height: 1.5; border-radius: 0 8px 8px 0; }
                            .wt-warning { background: #fef2f2; border-left: 5px solid #ef4444; padding: 15px; margin: 15px 0; font-size: 15px; line-height: 1.5; border-radius: 0 8px 8px 0; }
                            .wt-code { background: #1e293b; color: #38bdf8; padding: 4px 10px; border-radius: 6px; font-family: 'Consolas', monospace; font-weight: bold; font-size: 18px; letter-spacing: 0.5px; }
                            .wt-btn { display: block; width: 100%; padding: 16px; background: linear-gradient(135deg, #2563eb 0%, #1d4ed8 100%); color: white; border: none; border-radius: 12px; font-size: 18px; font-weight: bold; cursor: pointer; transition: transform 0.1s, box-shadow 0.2s; margin-top: 30px; text-transform: uppercase; letter-spacing: 1px; box-shadow: 0 10px 15px -3px rgba(37, 99, 235, 0.3); }
                            .wt-btn:hover { transform: translateY(-2px); box-shadow: 0 15px 20px -3px rgba(37, 99, 235, 0.4); }
                        `;
                        if (!document.getElementById("wt-helper-style")) document.head.appendChild(style);

                        const modal = document.createElement('div');
                        modal.id = "wt-helper-modal";
                        modal.className = 'wt-modal-overlay';
                        modal.innerHTML = `
                            <div class="wt-modal-content">
                                <div class="wt-title">👋 Bienvenue sur WikiTools</div>
                                <div class="wt-section">
                                    <p style="font-size: 1.1em;">Ceci est l'édition <strong>Native Bureau</strong>.</p>
                                    <div class="wt-warning">
                                        ⚠️ <strong>Ce n'est PAS un site web public.</strong><br/>
                                        N'essayez pas de configurer un nom de domaine ou une IP externe.
                                    </div>
                                </div>
                                <div class="wt-section">
                                    <p style="font-weight: bold; margin-bottom: 10px; font-size: 1.1em;">🔧 CONFIGURATION OBLIGATOIRE :</p>
                                    <p>Dans le champ <strong>Site URL</strong>, COPIEZ CELA :</p>
                                    <div class="wt-info" style="text-align: center; display: flex; align-items: center; justify-content: center; gap: 10px;">
                                        <span class="wt-code">http://localhost:3000</span>
                                        <button id="wt-copy-btn" style="background: #e2e8f0; border: none; padding: 6px 12px; border-radius: 6px; cursor: pointer; font-size: 13px; font-weight: bold; color: #475569; transition: all 0.2s;">COPIER</button>
                                    </div>
                                </div>
                                <button class="wt-btn" id="wt-close-btn">J'ai compris, Installer</button>
                            </div>
                        `;
                        document.body.appendChild(modal);
                        
                        // Gestionnaire fermeture
                        document.getElementById("wt-close-btn").addEventListener('click', function() { 
                            window.wt_modal_dismissed = true;
                            modal.remove(); 
                        });

                        // Gestionnaire Copie
                        document.getElementById("wt-copy-btn").addEventListener('click', function(e) {
                            navigator.clipboard.writeText('http://localhost:3000').then(function() {
                                const btn = e.target;
                                const originalText = btn.innerText;
                                btn.innerText = '✅ COPIÉ !';
                                btn.style.background = '#dcfce7';
                                btn.style.color = '#166534';
                                setTimeout(function() {
                                    btn.innerText = originalText;
                                    btn.style.background = '#e2e8f0';
                                    btn.style.color = '#475569';
                                }, 2000);
                            });
                        });
                    }
                }
                checkAndInjectModal();
                if (!window.wt_interval) window.wt_interval = setInterval(checkAndInjectModal, 1000);
            "#;
            let _ = window.eval(injection_script);
        })

        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
