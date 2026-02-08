import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faDatabase, faUser, faLock, faCheck, faSpinner, faServer, faFolderOpen } from "@fortawesome/free-solid-svg-icons";

interface HfsqlConfig {
  id: number;
  dsn: string | null;
  username: string | null;
  password: string | null;
  log_path: string | null;
}

interface ConnectionTestResult {
  success: boolean;
  error: string | null;
}

export default function HfsqlConfigTab() {
  const [config, setConfig] = useState<HfsqlConfig>({
    id: 1,
    dsn: "HFSQL",
    username: "Admin",
    password: "",
    log_path: "C:\\Users\\anis.bennia\\Desktop\\T\\BLOG",
  });
  const [isSaving, setIsSaving] = useState(false);
  const [saveSuccess, setSaveSuccess] = useState(false);
  const [isTesting, setIsTesting] = useState(false);
  const [testResult, setTestResult] = useState<ConnectionTestResult | null>(null);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      const data = await invoke<HfsqlConfig>("get_hfsql_config");
      setConfig({
        ...data,
        dsn: data.dsn ?? "HFSQL",
        username: data.username ?? "Admin",
        password: data.password ?? "",
        log_path: data.log_path ?? "C:\\Users\\anis.bennia\\Desktop\\T\\BLOG",
      });
    } catch (error) {
      console.error("Failed to load HFSQL config:", error);
    }
  };

  const handleSave = async () => {
    setIsSaving(true);
    setSaveSuccess(false);
    try {
      await invoke("save_hfsql_config", {
        dsn: config.dsn || "",
        username: config.username || "",
        password: config.password || "",
        logPath: config.log_path || "C:\\Users\\anis.bennia\\Desktop\\T\\BLOG",
      });
      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 2000);
    } catch (error) {
      console.error("Failed to save HFSQL config:", error);
    } finally {
      setIsSaving(false);
    }
  };

  const handleTestConnection = async () => {
    setIsTesting(true);
    setTestResult(null);
    try {
      const res = await invoke<ConnectionTestResult>("test_hfsql_connection");
      setTestResult(res);
    } catch (error) {
      console.error("Failed to test HFSQL connection:", error);
      setTestResult({ success: false, error: String(error) });
    } finally {
      setIsTesting(false);
    }
  };
  
  const handleSelectLogDir = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: config.log_path || undefined,
      });
      if (selected && typeof selected === 'string') {
        setConfig({ ...config, log_path: selected });
      }
    } catch (err) {
      console.error("Failed to open directory dialog:", err);
    }
  };

  const inputStyle = {
    background: "var(--bg-tertiary)",
    border: "1px solid var(--border-default)",
    color: "var(--text-primary)",
  };

  return (
    <div className="flex flex-col gap-6">
      <div 
        style={{ 
          background: "var(--bg-secondary)", 
          borderRadius: 12, 
          border: "1px solid var(--border-default)",
          overflow: "hidden" 
        }}
      >
        <div className="px-6 py-4 border-b border-[var(--border-default)]">
          <div className="flex items-center gap-3">
            <div 
              className="p-2 rounded-lg"
              style={{ background: "var(--color-info-bg)", color: "var(--color-info)" }}
            >
              <FontAwesomeIcon icon={faDatabase} className="h-5 w-5" />
            </div>
            <div>
              <h2 className="font-semibold" style={{ color: "var(--text-primary)" }}>Connexion HFSQL (ODBC)</h2>
              <p className="text-xs" style={{ color: "var(--text-tertiary)" }}>
                Configuration de la connexion ODBC vers HFSQL pour la synchronisation
              </p>
            </div>
          </div>
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
                ? "✅ Connexion ODBC réussie"
                : `❌ Échec de connexion: ${testResult.error ?? "Erreur inconnue"}`}
            </div>
          )}

          <div className="flex gap-4">
            <div className="flex-1">
              <label className="flex items-center gap-2 text-sm mb-2" style={{ color: "var(--text-secondary)" }}>
                <FontAwesomeIcon icon={faServer} className="h-3 w-3" />
                Source de données ODBC (DSN)
              </label>
              <input
                type="text"
                placeholder="ex: HFSQL"
                value={config.dsn ?? ""}
                onChange={(e) => setConfig({ ...config, dsn: e.target.value })}
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
                placeholder="ex: Admin"
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
          
          <div className="flex gap-4">
            <div className="flex-1">
              <label className="flex items-center gap-2 text-sm mb-2" style={{ color: "var(--text-secondary)" }}>
                <FontAwesomeIcon icon={faFolderOpen} className="h-3 w-3" />
                Dossier de logs
              </label>
              <div className="flex gap-2">
                  <input
                    type="text"
                    placeholder="C:\..."
                    value={config.log_path ?? ""}
                    onChange={(e) => setConfig({ ...config, log_path: e.target.value })}
                    className="w-full px-3 py-2 rounded-lg focus:outline-none"
                    style={inputStyle}
                    readOnly
                  />
                  <button
                    onClick={handleSelectLogDir}
                    className="px-3 py-2 rounded-lg transition-colors cursor-pointer"
                    style={{ background: "var(--bg-tertiary)", color: "var(--text-primary)", border: "1px solid var(--border-default)" }}
                  >
                    <FontAwesomeIcon icon={faFolderOpen} />
                  </button>
              </div>
            </div>
          </div>

          <div className="flex justify-end gap-3 pt-2">
            <button
              onClick={handleTestConnection}
              disabled={isSaving || isTesting}
              className="flex items-center gap-2 px-4 py-2 rounded-lg transition-colors cursor-pointer disabled:opacity-50"
              style={{ background: "var(--bg-tertiary)", color: "var(--text-primary)", border: "1px solid var(--border-default)" }}
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
    </div>
  );
}
