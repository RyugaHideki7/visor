"use client";

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { 
  faFilter, 
  faTrash, 
  faRefresh, 
  faCircleInfo, 
  faTriangleExclamation, 
  faCircleXmark,
  faCircleCheck
} from "@fortawesome/free-solid-svg-icons";

interface LogEntry {
  id: number;
  line_id: number | null;
  level: string;
  source: string | null;
  message: string;
  details: string | null;
  created_at: string;
}

interface Line {
  id: number;
  name: string;
}

const levelConfig: Record<string, { icon: typeof faCircleInfo; color: string; bg: string }> = {
  INFO: { icon: faCircleInfo, color: "var(--color-info)", bg: "var(--color-info-bg)" },
  WARNING: { icon: faTriangleExclamation, color: "var(--color-warning)", bg: "var(--color-warning-bg)" },
  ERROR: { icon: faCircleXmark, color: "var(--color-error)", bg: "var(--color-error-bg)" },
  SUCCESS: { icon: faCircleCheck, color: "var(--color-success)", bg: "var(--color-success-bg)" },
};

export default function Journaux() {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [lines, setLines] = useState<Line[]>([]);
  const [selectedLineId, setSelectedLineId] = useState<string>("");
  const [selectedLevel, setSelectedLevel] = useState<string>("");
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    loadLines();
    loadLogs();
  }, []);

  useEffect(() => {
    loadLogs();
  }, [selectedLineId, selectedLevel]);

  const loadLines = async () => {
    try {
      const data = await invoke<Line[]>("get_lines");
      setLines(data);
    } catch (error) {
      console.error("Failed to load lines:", error);
    }
  };

  const loadLogs = async () => {
    setIsLoading(true);
    try {
      const params: { lineId?: number; level?: string; limit?: number } = { limit: 500 };
      if (selectedLineId) params.lineId = parseInt(selectedLineId);
      if (selectedLevel) params.level = selectedLevel;
      
      const data = await invoke<LogEntry[]>("get_logs", params);
      setLogs(data);
    } catch (error) {
      console.error("Failed to load logs:", error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleClearLogs = async () => {
    if (!confirm("Êtes-vous sûr de vouloir supprimer tous les journaux ?")) return;
    
    try {
      const params: { lineId?: number } = {};
      if (selectedLineId) params.lineId = parseInt(selectedLineId);
      await invoke("clear_logs", params);
      loadLogs();
    } catch (error) {
      console.error("Failed to clear logs:", error);
    }
  };

  const getLineName = (lineId: number | null) => {
    if (!lineId) return "Système";
    const line = lines.find(l => l.id === lineId);
    return line?.name ?? `Ligne ${lineId}`;
  };

  const formatDate = (dateStr: string) => {
    try {
      const date = new Date(dateStr);
      return date.toLocaleString("fr-FR", {
        day: "2-digit",
        month: "2-digit",
        year: "numeric",
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
      });
    } catch {
      return dateStr;
    }
  };

  return (
    <div className="p-8 flex flex-col gap-6 h-full">
      <div className="flex justify-between items-start gap-6">
        <div>
          <h1 className="text-2xl font-bold" style={{ color: "var(--text-primary)" }}>Journaux</h1>
          <p className="text-sm" style={{ color: "var(--text-secondary)" }}>
            Historique des événements et opérations
          </p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={loadLogs}
            className="flex items-center gap-2 px-3 py-2 rounded-lg transition-colors cursor-pointer"
            style={{ background: "var(--button-secondary-bg)", color: "var(--text-primary)" }}
            onMouseEnter={(e) => e.currentTarget.style.background = "var(--button-secondary-hover)"}
            onMouseLeave={(e) => e.currentTarget.style.background = "var(--button-secondary-bg)"}
          >
            <FontAwesomeIcon icon={faRefresh} className={isLoading ? "animate-spin" : ""} />
          </button>
          <button
            onClick={handleClearLogs}
            className="flex items-center gap-2 px-3 py-2 rounded-lg transition-colors cursor-pointer"
            style={{ color: "var(--color-error)" }}
            onMouseEnter={(e) => e.currentTarget.style.background = "var(--color-error-bg)"}
            onMouseLeave={(e) => e.currentTarget.style.background = "transparent"}
          >
            <FontAwesomeIcon icon={faTrash} />
          </button>
        </div>
      </div>

      {/* Filters */}
      <div className="flex gap-4 items-center">
        <FontAwesomeIcon icon={faFilter} style={{ color: "var(--text-tertiary)" }} />
        <select
          value={selectedLineId}
          onChange={(e) => setSelectedLineId(e.target.value)}
          className="px-3 py-2 rounded-lg focus:outline-none cursor-pointer"
          style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-default)", color: "var(--text-primary)" }}
        >
          <option value="">Toutes les lignes</option>
          {lines.map((l) => (
            <option key={l.id} value={String(l.id)}>{l.name}</option>
          ))}
        </select>
        <select
          value={selectedLevel}
          onChange={(e) => setSelectedLevel(e.target.value)}
          className="px-3 py-2 rounded-lg focus:outline-none cursor-pointer"
          style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-default)", color: "var(--text-primary)" }}
        >
          <option value="">Tous les niveaux</option>
          <option value="INFO">Info</option>
          <option value="WARNING">Avertissement</option>
          <option value="ERROR">Erreur</option>
          <option value="SUCCESS">Succès</option>
        </select>
        <span className="text-sm" style={{ color: "var(--text-tertiary)" }}>
          {logs.length} entrée{logs.length !== 1 ? "s" : ""}
        </span>
      </div>

      {/* Logs Table */}
      <div 
        className="flex-1 overflow-auto"
        style={{ 
          background: "var(--bg-secondary)", 
          borderRadius: 8, 
          border: "1px solid var(--border-default)" 
        }}
      >
        <table className="w-full">
          <thead className="sticky top-0" style={{ background: "var(--bg-secondary)" }}>
            <tr style={{ borderBottom: "1px solid var(--border-default)", textAlign: "left", fontSize: 12, color: "var(--text-tertiary)" }}>
              <th style={{ padding: "12px 16px", fontWeight: 500, width: 100 }}>NIVEAU</th>
              <th style={{ padding: "12px 16px", fontWeight: 500, width: 150 }}>DATE</th>
              <th style={{ padding: "12px 16px", fontWeight: 500, width: 120 }}>LIGNE</th>
              <th style={{ padding: "12px 16px", fontWeight: 500, width: 100 }}>SOURCE</th>
              <th style={{ padding: "12px 16px", fontWeight: 500 }}>MESSAGE</th>
            </tr>
          </thead>
          <tbody>
            {logs.length === 0 ? (
              <tr>
                <td colSpan={5} style={{ padding: "48px 16px", textAlign: "center", color: "var(--text-tertiary)" }}>
                  Aucun journal trouvé.
                </td>
              </tr>
            ) : (
              logs.map((log) => {
                const config = levelConfig[log.level] ?? levelConfig.INFO;
                return (
                  <tr 
                    key={log.id} 
                    style={{ borderBottom: "1px solid var(--border-default)" }}
                    onMouseEnter={(e) => e.currentTarget.style.background = "var(--bg-hover)"}
                    onMouseLeave={(e) => e.currentTarget.style.background = "transparent"}
                  >
                    <td style={{ padding: "10px 16px" }}>
                      <span 
                        className="inline-flex items-center gap-1.5 px-2 py-1 rounded text-xs font-medium"
                        style={{ background: config.bg, color: config.color }}
                      >
                        <FontAwesomeIcon icon={config.icon} className="h-3 w-3" />
                        {log.level}
                      </span>
                    </td>
                    <td style={{ padding: "10px 16px", fontSize: 12, color: "var(--text-tertiary)" }}>
                      {formatDate(log.created_at)}
                    </td>
                    <td style={{ padding: "10px 16px", fontSize: 13, color: "var(--text-secondary)" }}>
                      {getLineName(log.line_id)}
                    </td>
                    <td style={{ padding: "10px 16px", fontSize: 12, color: "var(--text-tertiary)" }}>
                      {log.source ?? "-"}
                    </td>
                    <td style={{ padding: "10px 16px", fontSize: 13, color: "var(--text-primary)" }}>
                      {log.message}
                      {log.details && (
                        <span 
                          className="block text-xs mt-1" 
                          style={{ color: "var(--text-tertiary)" }}
                        >
                          {log.details}
                        </span>
                      )}
                    </td>
                  </tr>
                );
              })
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
