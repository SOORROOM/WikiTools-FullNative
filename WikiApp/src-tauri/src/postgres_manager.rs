use std::process::{Command, Stdio};
#[cfg(windows)]
use std::os::windows::process::CommandExt;

use std::path::PathBuf;
use std::fs;
use std::time::Duration;
use std::thread;
use rand::Rng;
use rand::distributions::Alphanumeric;
use serde::{Serialize, Deserialize};
use serde_json;

const APP_USER: &str = "app_user";

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum DatabaseMode {
    Embedded,
    Network,
}

impl Default for DatabaseMode {
    fn default() -> Self {
        DatabaseMode::Embedded
    }
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

#[derive(Serialize, Deserialize, Clone)]
struct DbConfig {
    #[serde(default)]
    mode: DatabaseMode,
    #[serde(default = "default_host")]
    host: String,
    port: u16,
    postgres_password: String,
    app_password: String,
}

pub struct PostgresManager {
    child: Option<std::process::Child>,
    data_dir: PathBuf,
    postgres_bin_dir: PathBuf,
    config_file_path: PathBuf,
    config: DbConfig,
    pub db_name: String,
}

impl PostgresManager {
    pub fn new(app_dir: PathBuf, resources_dir: PathBuf) -> Result<Self, String> {
        let postgres_root = resources_dir.join("postgresql");
        let postgres_bin_dir_raw = postgres_root.join("bin");
        let postgres_bin_dir = PathBuf::from(postgres_bin_dir_raw.to_string_lossy().replace("\\\\?\\", ""));
        let data_dir = app_dir.join("postgresql").join("data");
        let config_file_path = app_dir.join("postgresql").join("db_config.json");
        
        println!("📂 DB Paths: bin={:?}, data={:?}, config={:?}", postgres_bin_dir, data_dir, config_file_path);
        
        let is_new_config = !config_file_path.exists();
        let config = if !is_new_config {
            let config_str = fs::read_to_string(&config_file_path)
                .map_err(|e| format!("Echec lecture config: {}", e))?;
            serde_json::from_str(&config_str)
                .map_err(|e| format!("Echec parse config: {}", e))?
        } else {
            let new_config = DbConfig {
                mode: DatabaseMode::Embedded,
                host: "127.0.0.1".to_string(),
                port: rand::thread_rng().gen_range(15000..25000),
                postgres_password: Self::generate_strong_password(),
                app_password: Self::generate_strong_password(),
            };
            // Save immediately
            fs::create_dir_all(app_dir.join("postgresql")).ok();
            fs::write(&config_file_path, serde_json::to_string_pretty(&new_config).unwrap()).ok();
            new_config
        };
        
        Ok(Self {
            child: None,
            postgres_bin_dir,
            data_dir,
            config_file_path,
            config,
            db_name: "wiki".to_string(),
        })
    }

    /// Load an existing PostgreSQL configuration (e.g., from Core)
    pub fn from_existing_config(config_path: PathBuf, resources_dir: PathBuf, db_name: &str) -> Result<Self, String> {
        println!("📖 Loading existing config from: {:?}", config_path);
        
        let config_str = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;
        let config: DbConfig = serde_json::from_str(&config_str)
            .map_err(|e| format!("Failed to parse config: {}", e))?;
        
        let postgres_root = resources_dir.join("postgresql");
        let postgres_bin_dir = postgres_root.join("bin");
        
        // Use the data_dir from the config's parent directory
        let data_dir = config_path.parent()
            .ok_or("Invalid config path")?
            .join("data");
        
        // Nettoyage du chemin Windows (suppression du préfixe \\?\) pour plaire à Postgres
        let postgres_bin_dir_str = postgres_bin_dir.to_string_lossy().replace("\\\\?\\", "");
        let postgres_bin_dir = PathBuf::from(postgres_bin_dir_str);

        println!("📂 Using existing DB: bin={:?}, data={:?}, port={}", postgres_bin_dir, data_dir, config.port);
        
        Ok(Self {
            child: None,
            postgres_bin_dir,
            data_dir,
            config_file_path: config_path,
            config,
            db_name: db_name.to_string(),
        })
    }

    pub fn set_db_name(&mut self, name: &str) {
        self.db_name = name.to_string();
    }
    
    // Ajouté pour pouvoir changer le mode/host depuis l'UI
    pub fn update_config(&mut self, mode: DatabaseMode, host: String, port: u16) -> Result<(), String> {
        self.config.mode = mode;
        self.config.host = host;
        self.config.port = port;
        fs::write(&self.config_file_path, serde_json::to_string_pretty(&self.config).unwrap())
            .map_err(|e| e.to_string())
    }
    
    fn generate_strong_password() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    }
    
    pub fn is_initialized(&self) -> bool {
        if self.config.mode == DatabaseMode::Network {
             return true; // En réseau, on considère que c'est toujours "prêt" (c'est géré ailleurs)
        }
        self.data_dir.exists() && self.data_dir.join("PG_VERSION").exists()
    }
    
    pub fn init_database(&mut self) -> Result<(), String> {
        if self.config.mode == DatabaseMode::Network {
            return Ok(()); // Rien à faire en mode réseau
        }

        if self.is_initialized() {
            return Ok(());
        }
        
        println!("==> 1/6 : Initialisation du cluster PostgreSQL...");
        
        if self.data_dir.exists() {
            fs::remove_dir_all(&self.data_dir).ok();
        }
        fs::create_dir_all(&self.data_dir).map_err(|e| e.to_string())?;
        
        let pw_file_path = self.data_dir.parent().unwrap().join("pg_pw.tmp");
        fs::write(&pw_file_path, &self.config.postgres_password).map_err(|e| e.to_string())?;

        let output = Command::new(self.postgres_bin_dir.join("initdb.exe"))
            .arg("-D").arg(&self.data_dir)
            .arg("-U").arg("postgres")
            .arg("--encoding=UTF8")
            .arg("--locale=C")         
            .arg("--auth=scram-sha-256")
            .arg("--pwfile").arg(&pw_file_path)
            .arg("--no-sync")
            .output()
            .map_err(|e| e.to_string())?;
        
        let _ = fs::remove_file(&pw_file_path);
        
        if !output.status.success() {
            return Err(format!("Initdb erreur: {}", String::from_utf8_lossy(&output.stderr)));
        }

        fs::write(&self.config_file_path, serde_json::to_string_pretty(&self.config).unwrap()).ok();
        
        println!("==> 2/6 : Configuration du serveur...");
        self.configure_secure_postgres()?;
        
        println!("==> 3/6 : Démarrage temporaire...");
        self.start_internal()?;
        
        println!("==> 4/6 : Création de la base et attribution des droits...");
        self.ensure_database_exists()?;
        
        println!("==> 5/6 : Sécurisation des fichiers Windows...");
        self.secure_file_permissions()?;
        
        println!("==> 6/6 : Finalisation...");
        self.stop()?;
        
        println!("✅ Initialisation sécurisée terminée !");
        Ok(())
    }
    
    fn configure_secure_postgres(&self) -> Result<(), String> {
        let config_path = self.data_dir.join("postgresql.conf");
        let config = format!(
            "port = {}\nlisten_addresses = '127.0.0.1'\nmax_connections = 50\nshared_buffers = 128MB\npassword_encryption = scram-sha-256\ndynamic_shared_memory_type = windows\n",
            self.config.port
        );
        fs::write(&config_path, config).map_err(|e| e.to_string())?;
        
        let hba_path = self.data_dir.join("pg_hba.conf");
        let hba = format!(
            "host all postgres 127.0.0.1/32 scram-sha-256\nhost all {0} 127.0.0.1/32 scram-sha-256\n# Bloquer le reste\nhost all all 127.0.0.1/32 reject\n",
            APP_USER
        );
        fs::write(&hba_path, hba).map_err(|e| e.to_string())?;
        Ok(())
    }
    
    fn start_internal(&self) -> Result<(), String> {
        let mut cmd = Command::new(self.postgres_bin_dir.join("postgres.exe"));
        cmd.arg("-D").arg(&self.data_dir)
            .arg("-p").arg(self.config.port.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
            
        #[cfg(windows)]
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        cmd.spawn()
            .map_err(|e| e.to_string())?;
        
        self.wait_for_ready_internal(60)
    }

    fn wait_for_ready_internal(&self, attempts: i32) -> Result<(), String> {
        let isready = self.postgres_bin_dir.join("pg_isready.exe");
        for _ in 0..attempts {
            let mut cmd = Command::new(&isready);
            cmd.arg("-h").arg("127.0.0.1")
                .arg("-p").arg(self.config.port.to_string());;
                
            #[cfg(windows)]
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

            let status = cmd.status();
            if let Ok(s) = status {
                if s.success() { return Ok(()); }
            }
            thread::sleep(Duration::from_millis(500));
        }
        Err("Le serveur n'a pas démarré à temps".into())
    }
    
    pub fn start(&mut self) -> Result<(), String> {
        if self.config.mode == DatabaseMode::Network {
            println!("📡 Mode Réseau actif: Connexion à {}:{}", self.config.host, self.config.port);
            return Ok(());
        }

        if self.child.is_some() { return Ok(()); }

        // Check if already running (e.g. orphan process from previous session)
        if self.wait_for_ready_internal(2).is_ok() {
            println!("✅ PostgreSQL est déjà en cours d'exécution sur le port {}", self.config.port);
            return Ok(());
        }

        println!("🚀 Démarrage de PostgreSQL sur le port {}...", self.config.port);
        let postgres_exe = self.postgres_bin_dir.join("postgres.exe");
        if !postgres_exe.exists() {
            return Err(format!("❌ Exécutable PostgreSQL introuvable à : {:?}", postgres_exe));
        }

        let mut cmd = Command::new(&postgres_exe);
        cmd.arg("-D").arg(&self.data_dir)
            .arg("-p").arg(self.config.port.to_string());
            //.stdout(Stdio::null())
            //.stderr(std::fs::File::create("postgres_startup.log").unwrap());
            
        #[cfg(windows)]
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        let process = cmd.spawn()
            .map_err(|e| format!("Echec du spawn postgres: {}", e))?;
        
        self.child = Some(process);
        self.wait_for_ready_internal(60)?;
        println!("✅ Base de données prête !");
        Ok(())
    }
    
    pub fn stop(&mut self) -> Result<(), String> {
        if self.config.mode == DatabaseMode::Network {
            return Ok(());
        }

        let mut cmd = Command::new(self.postgres_bin_dir.join("pg_ctl.exe"));
        cmd.arg("stop").arg("-D").arg(&self.data_dir).arg("-m").arg("fast");
        
        #[cfg(windows)]
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        let _ = cmd.output();
        self.child = None;
        Ok(())
    }
    
    pub fn ensure_database_exists(&self) -> Result<(), String> {
        if self.config.mode == DatabaseMode::Network {
             // En mode réseau, on suppose que la DB existe déjà (créée par l'installateur serveur)
             // Mais on pourrait vérifier la connexion ici
             return Ok(());
        }

        let mut cmd = Command::new(self.postgres_bin_dir.join("createdb.exe"));
        cmd.arg("-h").arg("127.0.0.1").arg("-p").arg(self.config.port.to_string())
            .arg("-U").arg("postgres")
            .arg(&self.db_name)
            .env("PGPASSWORD", &self.config.postgres_password);
            
        #[cfg(windows)]
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        let _ = cmd.output();
        
        // Always try to authorize the user on the current db
        self.authorize_user_on_db(&self.db_name)?;
        Ok(())
    }

    pub fn authorize_user_on_db(&self, db_name: &str) -> Result<(), String> {
         if self.config.mode == DatabaseMode::Network { return Ok(()); }

        // 1. Ensure user exists (connect to 'postgres')
        let user_sql = format!(
            "DO $$ BEGIN IF NOT EXISTS (SELECT FROM pg_catalog.pg_user WHERE usename = '{0}') THEN CREATE USER {0} WITH PASSWORD '{1}'; END IF; END $$;",
            APP_USER, self.config.app_password
        );
        let mut cmd = Command::new(self.postgres_bin_dir.join("psql.exe"));
        cmd.arg("-h").arg("127.0.0.1").arg("-p").arg(self.config.port.to_string())
            .arg("-U").arg("postgres").arg("-d").arg("postgres")
            .arg("-c").arg(&user_sql)
            .env("PGPASSWORD", &self.config.postgres_password);
            
        #[cfg(windows)]
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        let _ = cmd.output();

        // 2. Grant rights on the specific database (connect to that database)
        let db_sql = format!(
            "ALTER DATABASE {0} OWNER TO {1}; \
             GRANT ALL ON SCHEMA public TO {1};",
            db_name, APP_USER
        );
        
        let mut cmd = Command::new(self.postgres_bin_dir.join("psql.exe"));
        cmd.arg("-h").arg("127.0.0.1").arg("-p").arg(self.config.port.to_string())
            .arg("-U").arg("postgres").arg("-d").arg(db_name)
            .arg("-c").arg(&db_sql)
            .env("PGPASSWORD", &self.config.postgres_password);
            
        #[cfg(windows)]
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        let output = cmd.output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            println!("Erreur d'attribution des droits sur {}: {}", db_name, String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }
    
    fn secure_file_permissions(&self) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            let data_path = self.data_dir.to_str().unwrap();
            // SYSTEM et Administrateurs
            let _ = Command::new("icacls").arg(data_path).arg("/grant").arg("SYSTEM:(OI)(CI)F").output();
            let _ = Command::new("icacls").arg(data_path).arg("/grant").arg("*S-1-5-32-544:(OI)(CI)F").output();
            
            // On s'assure que l'utilisateur local actuel a aussi l'accès total pour le développement
            let username = std::env::var("USERNAME").unwrap_or_default();
            if !username.is_empty() {
                let _ = Command::new("icacls").arg(data_path).arg("/grant").arg(format!("{}:(OI)(CI)F", username)).output();
            }
        }
        Ok(())
    }
    
    pub fn get_connection_string(&self) -> String {
        format!("postgres://postgres:{}@{}:{}/{}?sslmode=disable", 
            self.config.postgres_password,
            self.config.host,
            self.config.port,
            self.db_name
        )
    }
}

// On ne veut PAS arrêter la DB quand le manager est droppé.
// Elle doit rester active pour les autres applis ou pour le Wiki lui-même.
/*
impl Drop for PostgresManager {
    fn drop(&mut self) { let _ = self.stop(); }
}
*/
