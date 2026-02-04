import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Loader2, Server, CheckCircle, AlertCircle } from "lucide-react";
import "./App.css";

function App() {
  const [status, setStatus] = useState<"checking" | "starting" | "ready" | "error">("checking");
  const [message, setMessage] = useState("Vérification du moteur Docker...");

  useEffect(() => {
    initSystem();
  }, []);

  async function initSystem() {
    try {
      // 1. Initialisation Base de Données (Shared / CollabTools)
      setStatus("starting");
      setMessage("Connexion au moteur PostgreSQL...");
      await invoke("init_db");

      // 2. Démarrage du Serveur Wiki.js (Node.js)
      setStatus("starting");
      setMessage("Démarrage du serveur Wiki (Node.js)...");
      await invoke("start_wiki_server");

      // 3. Wait for Wiki to be responsive (Polling strict)
      setStatus("checking");
      setMessage("Attente du serveur Web...");

      let attempts = 0;
      const maxAttempts = 60;

      while (attempts < maxAttempts) {
        attempts++;
        if (attempts % 5 === 0) setMessage(`Démarrage en cours... (${attempts}s)`);

        try {
          // On demande à Rust si le serveur répond 200 OK
          const isHealthy = await invoke("check_health");

          if (isHealthy) {
            setMessage("Serveur prêt ! Lancement...");
            await new Promise(r => setTimeout(r, 1000));
            break;
          } else {
            throw new Error("Not ready");
          }

        } catch (err) {
          await new Promise(r => setTimeout(r, 1000));
        }
      }

      if (attempts >= maxAttempts) {
        throw new Error("Délai dépassé. Wiki.js ne répond pas.");
      }

      setStatus("ready");
      window.location.href = "http://localhost:3000";

    } catch (e) {
      console.error(e);
      setStatus("error");
      setMessage(String(e));
    }
  }



  if (status === ("ready" as string)) {
    return (
      <div style={{ width: "100vw", height: "100vh", backgroundColor: "#0f172a", display: "flex", alignItems: "center", justifyContent: "center", color: "white" }}>
        Chargement de l'interface...
      </div>
    );
  }

  return (
    <div className="container" style={{
      display: "flex",
      flexDirection: "column",
      alignItems: "center",
      justifyContent: "center",
      height: "100vh",
      backgroundColor: "#0f172a",
      color: "#e2e8f0",
      fontFamily: "sans-serif"
    }}>
      <div style={{ textAlign: "center", maxWidth: "400px" }}>

        <div style={{ marginBottom: "2rem", display: "flex", justifyContent: "center" }}>
          {status === "error" ? (
            <AlertCircle size={64} color="#ef4444" />
          ) : (
            <Server size={64} color="#3b82f6" />
          )}
        </div>

        <h1 style={{ fontSize: "1.5rem", marginBottom: "1rem" }}>WikiTools Launcher</h1>

        <div style={{ display: "flex", alignItems: "center", justifyContent: "center", gap: "10px", marginBottom: "0.5rem" }}>
          {status === "starting" || status === "checking" ? (
            <Loader2 className="spin" size={24} />
          ) : status === "ready" ? (
            <CheckCircle size={24} color="#22c55e" />
          ) : null}
          <span style={{ fontSize: "1.1rem" }}>{message}</span>
        </div>

        <p style={{ color: "#64748b", marginTop: "2rem", fontSize: "0.9rem" }}>
          Propulsé par Wiki.js & PostgreSQL
        </p>
      </div>

      <style>{`
        .spin { animation: spin 2s linear infinite; }
        @keyframes spin { 0% { transform: rotate(0deg); } 100% { transform: rotate(360deg); } }
      `}</style>
    </div>
  );
}

export default App;
