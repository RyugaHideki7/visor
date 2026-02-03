"use client";

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faDatabase, faServer, faUser, faLock, faCheck, faSpinner, faArrowRotateRight, faCloudArrowDown } from "@fortawesome/free-solid-svg-icons";
import { check } from "@tauri-apps/plugin-updater";

interface SqlServerConfig {
  id: number;
  server: string | null;
  database: string | null;
  username: string | null;
  password: string | null;
  enabled: boolean;
}

interface ConnectionTestResult {
  success: boolean;
  error: string | null;
}

type UpdateStatus =
  | { state: "idle" }
  | { state: "checking" }
  | { state: "available"; version: string }
  | { state: "downloading"; version: string; progress?: number }
  | { state: "ready"; version: string }
  | { state: "uptodate" }
  | { state: "error"; message: string };

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
  const [isTesting, setIsTesting] = useState(false);
  const [testResult, setTestResult] = useState<ConnectionTestResult | null>(null);
  const [updateStatus, setUpdateStatus] = useState<UpdateStatus>({ state: "idle" });
  const [appVersion, setAppVersion] = useState<string>("");

  useEffect(() => {
    loadConfig();
    loadVersion();
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

  const loadVersion = async () => {
    try {
      const v = await getVersion();
      setAppVersion(v);
    } catch (error) {
      console.error("Failed to load app version:", error);
    }
  };

  const handleCheckUpdate = async () => {
    setUpdateStatus({ state: "checking" });
    try {
      const result = await check();
      if (!result) {
        setUpdateStatus({ state: "uptodate" });
        return;
      }
      if (result.available) {
        const version = result.version ?? "";
        setUpdateStatus({ state: "available", version });

        await result.downloadAndInstall((event) => {
          if (event.event === "Progress") {
            setUpdateStatus({ state: "downloading", version });
          }
        });

        // If download succeeded, finalize and restart
        await result.install();
        setUpdateStatus({ state: "ready", version });

        // Tauri will relaunch on install(); in case it doesn't, you can prompt restart
      } else {
        setUpdateStatus({ state: "uptodate" });
      }
    } catch (err) {
      setUpdateStatus({ state: "error", message: String(err) });
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

  const handleTestConnection = async () => {
    setIsTesting(true);
    setTestResult(null);
    try {
      const res = await invoke<ConnectionTestResult>("test_sql_server_connection");
      setTestResult(res);
    } catch (error) {
      console.error("Failed to test SQL Server connection:", error);
      setTestResult({ success: false, error: String(error) });
    } finally {
      setIsTesting(false);
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

      {/* Mises à jour */}
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
              style={{ background: "var(--color-warning-bg)", color: "var(--color-warning)" }}
            >
              <FontAwesomeIcon icon={faArrowRotateRight} className="h-5 w-5" />
            </div>
            <div>
              <h2 className="font-semibold" style={{ color: "var(--text-primary)" }}>Mises à jour</h2>
              <p className="text-xs" style={{ color: "var(--text-tertiary)" }}>
                Vérifier et installer automatiquement les nouvelles versions
              </p>
            </div>
          </div>
          <button
            onClick={handleCheckUpdate}
            disabled={updateStatus.state === "checking" || updateStatus.state === "downloading" || updateStatus.state === "uptodate"}
            className="px-4 py-2 rounded-lg text-sm font-medium flex items-center gap-2"
            style={{
              background: "var(--button-secondary-bg)",
              color: "var(--text-primary)",
              border: "1px solid var(--border-default)",
              opacity: updateStatus.state === "checking" || updateStatus.state === "downloading" ? 0.6 : 1,
            }}
          >
            {updateStatus.state === "checking" || updateStatus.state === "downloading" ? (
              <FontAwesomeIcon icon={faSpinner} className="h-4 w-4 animate-spin" />
            ) : (
              <FontAwesomeIcon icon={faCloudArrowDown} className="h-4 w-4" />
            )}
            {updateStatus.state === "checking" && "Recherche..."}
            {updateStatus.state === "downloading" && "Téléchargement..."}
            {updateStatus.state === "available" && "Installer la mise à jour"}
            {updateStatus.state === "ready" && "Redémarrage..."}
            {updateStatus.state === "idle" && "Rechercher une mise à jour"}
            {updateStatus.state === "uptodate" && "À jour"}
            {updateStatus.state === "error" && "Réessayer"}
          </button>
        </div>

        <div className="p-6 flex flex-col gap-3 text-sm" style={{ color: "var(--text-secondary)" }}>
          {updateStatus.state === "available" && (
            <div style={{ color: "var(--color-info)", background: "var(--color-info-bg)", padding: "8px 12px", borderRadius: 8 }}>
              Nouvelle version disponible: {updateStatus.version}
            </div>
          )}
          {updateStatus.state === "downloading" && (
            <div style={{ color: "var(--color-warning)" }}>
              Téléchargement...
            </div>
          )}
          {updateStatus.state === "ready" && (
            <div style={{ color: "var(--color-success)" }}>
              Mise à jour téléchargée. L'application va redémarrer pour appliquer la mise à jour.
            </div>
          )}
          {updateStatus.state === "uptodate" && (
            <div style={{ color: "var(--color-success)" }}>
              Vous êtes déjà à jour.
            </div>
          )}
          {updateStatus.state === "error" && (
            <div style={{ color: "var(--color-error)" }}>
              Échec de mise à jour: {updateStatus.message}
            </div>
          )}
          {updateStatus.state === "idle" && (
            <div>
              Les mises à jour seront proposées automatiquement si une nouvelle version est disponible.
            </div>
          )}
        </div>
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
          {testResult && (
            <div
              className="px-4 py-2 rounded-lg text-sm"
              style={{
                background: testResult.success ? "var(--color-success-bg)" : "var(--color-error-bg)",
                color: testResult.success ? "var(--color-success)" : "var(--color-error)",
              }}
            >
              {testResult.success
                ? "✅ Connexion à la base de données réussie"
                : `❌ Échec de connexion: ${testResult.error ?? "Erreur inconnue"}`}
            </div>
          )}

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
              onClick={handleTestConnection}
              disabled={isSaving || isTesting}
              className="flex items-center gap-2 px-4 py-2 rounded-lg transition-colors cursor-pointer disabled:opacity-50"
              style={{ background: "var(--bg-tertiary)", color: "var(--text-primary)", border: "1px solid var(--border-default)" }}
              onMouseEnter={(e) => !isTesting && (e.currentTarget.style.borderColor = "var(--accent-primary)")}
              onMouseLeave={(e) => (e.currentTarget.style.borderColor = "var(--border-default)")}
            >
              {isTesting ? (
                <FontAwesomeIcon icon={faSpinner} className="h-4 w-4 animate-spin" />
              ) : (
                <FontAwesomeIcon icon={faDatabase} className="h-4 w-4" />
              )}
              Tester connexion
            </button>
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
          Version {appVersion || "..."}
        </p>
      </div>
    </div>
  );
}
