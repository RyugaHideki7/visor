"use client";

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faDatabase, faServer, faUser, faLock, faCheck, faSpinner } from "@fortawesome/free-solid-svg-icons";

interface SqlServerConfig {
  id: number;
  server: string | null;
  database: string | null;
  username: string | null;
  password: string | null;
  enabled: boolean;
}

export default function Settings() {
  const [config, setConfig] = useState<SqlServerConfig>({
    id: 1,
    server: "",
    database: "",
    username: "",
    password: "",
    enabled: false,
  });
  const [isSaving, setIsSaving] = useState(false);
  const [saveSuccess, setSaveSuccess] = useState(false);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      const data = await invoke<SqlServerConfig>("get_sql_server_config");
      setConfig({
        ...data,
        server: data.server ?? "",
        database: data.database ?? "",
        username: data.username ?? "",
        password: data.password ?? "",
      });
    } catch (error) {
      console.error("Failed to load SQL Server config:", error);
    }
  };

  const handleSave = async () => {
    setIsSaving(true);
    setSaveSuccess(false);
    try {
      await invoke("save_sql_server_config", {
        server: config.server || "",
        database: config.database || "",
        username: config.username || "",
        password: config.password || "",
        enabled: config.enabled,
      });
      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 2000);
    } catch (error) {
      console.error("Failed to save SQL Server config:", error);
    } finally {
      setIsSaving(false);
    }
  };

  const inputStyle = {
    background: "var(--bg-tertiary)",
    border: "1px solid var(--border-default)",
    color: "var(--text-primary)",
  };

  return (
    <div className="p-8 flex flex-col gap-8">
      <div>
        <h1 className="text-2xl font-bold" style={{ color: "var(--text-primary)" }}>Paramètres</h1>
        <p className="text-sm" style={{ color: "var(--text-secondary)" }}>
          Configuration de l'application et connexion SQL Server
        </p>
      </div>

      {/* SQL Server Configuration */}
      <div 
        style={{ 
          background: "var(--bg-secondary)", 
          borderRadius: 12, 
          border: "1px solid var(--border-default)",
          overflow: "hidden" 
        }}
      >
        <div className="px-6 py-4 flex items-center justify-between" style={{ borderBottom: "1px solid var(--border-default)" }}>
          <div className="flex items-center gap-3">
            <div 
              className="p-2 rounded-lg"
              style={{ background: "var(--color-info-bg)", color: "var(--color-info)" }}
            >
              <FontAwesomeIcon icon={faDatabase} className="h-5 w-5" />
            </div>
            <div>
              <h2 className="font-semibold" style={{ color: "var(--text-primary)" }}>Connexion SQL Server</h2>
              <p className="text-xs" style={{ color: "var(--text-tertiary)" }}>
                Paramètres de connexion à la base de données
              </p>
            </div>
          </div>
          <label className="flex items-center gap-2 cursor-pointer">
            <span className="text-sm" style={{ color: "var(--text-secondary)" }}>
              {config.enabled ? "Activé" : "Désactivé"}
            </span>
            <div 
              className="relative w-11 h-6 rounded-full transition-colors cursor-pointer"
              style={{ background: config.enabled ? "var(--color-success)" : "var(--bg-tertiary)" }}
              onClick={() => setConfig({ ...config, enabled: !config.enabled })}
            >
              <div 
                className="absolute top-1 w-4 h-4 rounded-full bg-white transition-all"
                style={{ left: config.enabled ? 24 : 4 }}
              />
            </div>
          </label>
        </div>

        <div className="p-6 flex flex-col gap-4">
          <div className="flex gap-4">
            <div className="flex-1">
              <label className="flex items-center gap-2 text-sm mb-2" style={{ color: "var(--text-secondary)" }}>
                <FontAwesomeIcon icon={faServer} className="h-3 w-3" />
                Serveur
              </label>
              <input
                type="text"
                placeholder="ex: localhost\SQLEXPRESS"
                value={config.server ?? ""}
                onChange={(e) => setConfig({ ...config, server: e.target.value })}
                className="w-full px-3 py-2 rounded-lg focus:outline-none"
                style={inputStyle}
                onFocus={(e) => e.currentTarget.style.borderColor = "var(--accent-primary)"}
                onBlur={(e) => e.currentTarget.style.borderColor = "var(--border-default)"}
              />
            </div>
            <div className="flex-1">
              <label className="flex items-center gap-2 text-sm mb-2" style={{ color: "var(--text-secondary)" }}>
                <FontAwesomeIcon icon={faDatabase} className="h-3 w-3" />
                Base de données
              </label>
              <input
                type="text"
                placeholder="ex: Production"
                value={config.database ?? ""}
                onChange={(e) => setConfig({ ...config, database: e.target.value })}
                className="w-full px-3 py-2 rounded-lg focus:outline-none"
                style={inputStyle}
                onFocus={(e) => e.currentTarget.style.borderColor = "var(--accent-primary)"}
                onBlur={(e) => e.currentTarget.style.borderColor = "var(--border-default)"}
              />
            </div>
          </div>

          <div className="flex gap-4">
            <div className="flex-1">
              <label className="flex items-center gap-2 text-sm mb-2" style={{ color: "var(--text-secondary)" }}>
                <FontAwesomeIcon icon={faUser} className="h-3 w-3" />
                Utilisateur
              </label>
              <input
                type="text"
                placeholder="ex: sa"
                value={config.username ?? ""}
                onChange={(e) => setConfig({ ...config, username: e.target.value })}
                className="w-full px-3 py-2 rounded-lg focus:outline-none"
                style={inputStyle}
                onFocus={(e) => e.currentTarget.style.borderColor = "var(--accent-primary)"}
                onBlur={(e) => e.currentTarget.style.borderColor = "var(--border-default)"}
              />
            </div>
            <div className="flex-1">
              <label className="flex items-center gap-2 text-sm mb-2" style={{ color: "var(--text-secondary)" }}>
                <FontAwesomeIcon icon={faLock} className="h-3 w-3" />
                Mot de passe
              </label>
              <input
                type="password"
                placeholder="••••••••"
                value={config.password ?? ""}
                onChange={(e) => setConfig({ ...config, password: e.target.value })}
                className="w-full px-3 py-2 rounded-lg focus:outline-none"
                style={inputStyle}
                onFocus={(e) => e.currentTarget.style.borderColor = "var(--accent-primary)"}
                onBlur={(e) => e.currentTarget.style.borderColor = "var(--border-default)"}
              />
            </div>
          </div>

          <div className="flex justify-end gap-3 pt-2">
            <button
              onClick={handleSave}
              disabled={isSaving}
              className="flex items-center gap-2 px-4 py-2 rounded-lg transition-colors cursor-pointer disabled:opacity-50"
              style={{ background: "var(--button-primary-bg)", color: "white" }}
              onMouseEnter={(e) => !isSaving && (e.currentTarget.style.background = "var(--button-primary-hover)")}
              onMouseLeave={(e) => e.currentTarget.style.background = "var(--button-primary-bg)"}
            >
              {isSaving ? (
                <FontAwesomeIcon icon={faSpinner} className="h-4 w-4 animate-spin" />
              ) : saveSuccess ? (
                <FontAwesomeIcon icon={faCheck} className="h-4 w-4" />
              ) : null}
              {saveSuccess ? "Enregistré" : "Enregistrer"}
            </button>
          </div>
        </div>
      </div>

      {/* App Info */}
      <div 
        className="p-6 rounded-xl"
        style={{ background: "var(--bg-secondary)", border: "1px solid var(--border-default)" }}
      >
        <h3 className="font-semibold mb-2" style={{ color: "var(--text-primary)" }}>À propos</h3>
        <p className="text-sm" style={{ color: "var(--text-tertiary)" }}>
          Visor - Surveillance des lignes de production<br />
          Version 1.0.0
        </p>
      </div>
    </div>
  );
}
