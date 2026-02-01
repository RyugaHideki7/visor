"use client";

import { useState, useEffect, useMemo } from "react";
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
import { Select, SelectItem, Button } from "@heroui/react";
import { ConfirmDialog, useConfirmDialog } from "@/shared/ui/ConfirmDialog";

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
  const confirmDialog = useConfirmDialog();

  const lineOptions = useMemo(() => [
    { key: "", name: "Toutes les lignes" },
    ...lines.map((l) => ({ key: String(l.id), name: l.name })),
  ], [lines]);

  const levelOptions = useMemo(() => [
    { key: "", name: "Tous les niveaux" },
    { key: "INFO", name: "Info" },
    { key: "WARNING", name: "Avertissement" },
    { key: "ERROR", name: "Erreur" },
    { key: "SUCCESS", name: "Succès" },
  ], []);

  const handleSelectLine = (keys: any) => {
    const first = Array.from(keys)[0] as string | undefined;
    setSelectedLineId(first ?? "");
  };

  const handleSelectLevel = (keys: any) => {
    const first = Array.from(keys)[0] as string | undefined;
    setSelectedLevel(first ?? "");
  };

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
    const confirmed = await confirmDialog.open({
      title: "Confirmation",
      message: "Êtes-vous sûr de vouloir supprimer tous les journaux ?",
      confirmText: "Supprimer",
      cancelText: "Annuler",
      variant: "danger",
    });
    if (!confirmed) return;

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
          <Button
            onPress={loadLogs}
            isIconOnly
            variant="bordered"
            className="border-(--border-default) text-(--text-primary) hover:bg-(--bg-hover)"
          >
            <FontAwesomeIcon icon={faRefresh} className={isLoading ? "animate-spin" : ""} />
          </Button>
          <Button
            onPress={handleClearLogs}
            isLoading={isLoading}
            variant="bordered"
            startContent={<FontAwesomeIcon icon={faTrash} />}
            className="border-(--border-default) text-(--text-primary) hover:bg-(--bg-hover)"
          >
            Vider
          </Button>
        </div>
      </div>

      <ConfirmDialog {...confirmDialog.props} />

      {/* Filters */}
      <div className="flex gap-4 items-center">
        
        <Select
          items={lineOptions}
          selectedKeys={new Set([selectedLineId])}
          onSelectionChange={handleSelectLine}
          placeholder="Toutes les lignes"
          variant="bordered"
          className="min-w-[200px]"
          classNames={{
            trigger: "bg-[var(--bg-tertiary)] border-[var(--border-default)] hover:border-[var(--accent-primary)] data-[open=true]:border-[var(--accent-primary)]",
            value: "text-[var(--text-primary)]",
            popoverContent: "bg-[var(--bg-secondary)] border border-[var(--border-default)]",
          }}
        >
          {(item) => (
            <SelectItem key={item.key} textValue={item.name} className="text-(--text-primary) hover:bg-(--bg-hover)">
              {item.name}
            </SelectItem>
          )}
        </Select>
        <Select
          items={levelOptions}
          selectedKeys={new Set([selectedLevel])}
          onSelectionChange={handleSelectLevel}
          placeholder="Tous les niveaux"
          variant="bordered"
          className="min-w-[180px]"
          classNames={{
            trigger: "bg-[var(--bg-tertiary)] border-[var(--border-default)] hover:border-[var(--accent-primary)] data-[open=true]:border-[var(--accent-primary)]",
            value: "text-[var(--text-primary)]",
            popoverContent: "bg-[var(--bg-secondary)] border border-[var(--border-default)]",
          }}
        >
          {(item) => (
            <SelectItem key={item.key} textValue={item.name} className="text-(--text-primary) hover:bg-(--bg-hover)">
              {item.name}
            </SelectItem>
          )}
        </Select>
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
