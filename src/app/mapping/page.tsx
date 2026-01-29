"use client";

import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faPlus, faTrash } from "@fortawesome/free-solid-svg-icons";

type Line = {
  id?: number;
  name: string;
};

type MappingRow = {
  id?: number;
  line_id: number;
  sort_order: number;
  sql_field: string;
  file_column?: string | null;
  parameter?: string | null;
  transformation?: string | null;
  description?: string | null;
};

export default function MappingPage() {
  const [lines, setLines] = useState<Line[]>([]);
  const [selectedLineId, setSelectedLineId] = useState<string>("");
  const [rows, setRows] = useState<MappingRow[]>([]);
  const [isSaving, setIsSaving] = useState(false);

  const lineIdNumber = useMemo(() => {
    const n = Number(selectedLineId);
    return Number.isFinite(n) ? n : null;
  }, [selectedLineId]);

  const loadLines = async () => {
    const res = await invoke<any[]>("get_lines");
    const mapped: Line[] = res.map((l) => ({ id: l.id, name: l.name }));
    setLines(mapped);
    if (!selectedLineId && mapped.length > 0 && mapped[0].id != null) {
      setSelectedLineId(String(mapped[0].id));
    }
  };

  const loadMappings = async (id: number) => {
    const res = await invoke<MappingRow[]>("get_mappings", { lineId: id });
    setRows(res);
  };

  useEffect(() => {
    loadLines().catch((e) => console.error(e));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (lineIdNumber == null) return;
    loadMappings(lineIdNumber).catch((e) => console.error(e));
  }, [lineIdNumber]);

  const handleAddRow = () => {
    if (lineIdNumber == null) return;
    setRows((prev) => [
      ...prev,
      {
        id: undefined,
        line_id: lineIdNumber,
        sort_order: prev.length,
        sql_field: "",
        file_column: "",
        parameter: "",
        transformation: "",
        description: "",
      },
    ]);
  };

  const handleDeleteRow = (index: number) => {
    setRows((prev) => prev.filter((_, i) => i !== index));
  };

  const handleUpdateRow = (index: number, patch: Partial<MappingRow>) => {
    setRows((prev) => prev.map((r, i) => (i === index ? { ...r, ...patch } : r)));
  };

  const handleSave = async () => {
    if (lineIdNumber == null) return;
    setIsSaving(true);
    try {
      const normalized = rows.map((r, idx) => ({
        ...r,
        line_id: lineIdNumber,
        sort_order: idx,
      }));
      await invoke("save_mappings", { lineId: lineIdNumber, mappings: normalized });
      await loadMappings(lineIdNumber);
    } catch (e) {
      console.error(e);
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="p-8 flex flex-col gap-6">
      <div className="flex justify-between items-start gap-6">
        <div>
          <h1 className="text-2xl font-bold" style={{ color: "var(--text-primary)" }}>Mapping</h1>
          <p className="text-sm" style={{ color: "var(--text-secondary)" }}>
            Associez les colonnes du fichier et/ou des paramètres de la ligne aux champs SQL.
          </p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={handleAddRow}
            className="flex items-center gap-2 px-4 py-2 rounded-lg transition-colors cursor-pointer"
            style={{ background: "var(--button-secondary-bg)", color: "var(--text-primary)" }}
            onMouseEnter={(e) => e.currentTarget.style.background = "var(--button-secondary-hover)"}
            onMouseLeave={(e) => e.currentTarget.style.background = "var(--button-secondary-bg)"}
          >
            <FontAwesomeIcon icon={faPlus} />
            Ajouter
          </button>
          <button
            onClick={handleSave}
            disabled={isSaving}
            className="flex items-center gap-2 px-4 py-2 rounded-lg transition-colors cursor-pointer disabled:opacity-50"
            style={{ background: "var(--button-primary-bg)", color: "white" }}
            onMouseEnter={(e) => !isSaving && (e.currentTarget.style.background = "var(--button-primary-hover)")}
            onMouseLeave={(e) => e.currentTarget.style.background = "var(--button-primary-bg)"}
          >
            {isSaving ? "..." : "Sauvegarder"}
          </button>
        </div>
      </div>

      <div className="max-w-md">
        <label className="block text-sm mb-2" style={{ color: "var(--text-secondary)" }}>Ligne</label>
        <select
          value={selectedLineId}
          onChange={(e) => setSelectedLineId(e.target.value)}
          className="w-full px-3 py-2 rounded-lg focus:outline-none cursor-pointer"
          style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-default)", color: "var(--text-primary)" }}
        >
          <option value="">Sélectionner une ligne</option>
          {lines.map((l) => (
            <option key={l.id} value={String(l.id)}>
              {l.name}
            </option>
          ))}
        </select>
      </div>

      <div style={{ background: "var(--bg-secondary)", borderRadius: 8, border: "1px solid var(--border-default)", overflow: "hidden" }}>
        <table className="w-full">
          <thead>
            <tr style={{ borderBottom: "1px solid var(--border-default)", textAlign: "left", fontSize: 14, color: "var(--text-tertiary)" }}>
              <th style={{ padding: "12px 16px", fontWeight: 500 }}>CHAMP SQL</th>
              <th style={{ padding: "12px 16px", fontWeight: 500 }}>COLONNE</th>
              <th style={{ padding: "12px 16px", fontWeight: 500 }}>PARAMÈTRE</th>
              <th style={{ padding: "12px 16px", fontWeight: 500 }}>TRANSFORMATION</th>
              <th style={{ padding: "12px 16px", fontWeight: 500 }}>DESCRIPTION</th>
              <th style={{ padding: "12px 16px", fontWeight: 500 }}>ACTIONS</th>
            </tr>
          </thead>
          <tbody>
            {rows.length === 0 ? (
              <tr>
                <td colSpan={6} style={{ padding: "32px 16px", textAlign: "center", color: "var(--text-tertiary)" }}>
                  Aucun mapping pour cette ligne.
                </td>
              </tr>
            ) : (
              rows.map((r, idx) => (
                <tr key={`${r.id ?? "new"}-${idx}`} style={{ borderBottom: "1px solid var(--border-default)" }}>
                  <td style={{ padding: "8px" }}>
                    <input
                      type="text"
                      value={r.sql_field}
                      onChange={(e) => handleUpdateRow(idx, { sql_field: e.target.value })}
                      className="w-full px-2 py-1 rounded text-sm focus:outline-none"
                      style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-default)", color: "var(--text-primary)" }}
                      placeholder="ex: YSCC_0"
                    />
                  </td>
                  <td style={{ padding: "8px" }}>
                    <input
                      type="text"
                      value={r.file_column ?? ""}
                      onChange={(e) => handleUpdateRow(idx, { file_column: e.target.value })}
                      className="w-full px-2 py-1 rounded text-sm focus:outline-none"
                      style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-default)", color: "var(--text-primary)" }}
                      placeholder="ex: 0"
                    />
                  </td>
                  <td style={{ padding: "8px" }}>
                    <input
                      type="text"
                      value={r.parameter ?? ""}
                      onChange={(e) => handleUpdateRow(idx, { parameter: e.target.value })}
                      className="w-full px-2 py-1 rounded text-sm focus:outline-none"
                      style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-default)", color: "var(--text-primary)" }}
                      placeholder="ex: site"
                    />
                  </td>
                  <td style={{ padding: "8px" }}>
                    <input
                      type="text"
                      value={r.transformation ?? ""}
                      onChange={(e) => handleUpdateRow(idx, { transformation: e.target.value })}
                      className="w-full px-2 py-1 rounded text-sm focus:outline-none"
                      style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-default)", color: "var(--text-primary)" }}
                      placeholder="ex: date"
                    />
                  </td>
                  <td style={{ padding: "8px" }}>
                    <input
                      type="text"
                      value={r.description ?? ""}
                      onChange={(e) => handleUpdateRow(idx, { description: e.target.value })}
                      className="w-full px-2 py-1 rounded text-sm focus:outline-none"
                      style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-default)", color: "var(--text-primary)" }}
                      placeholder="Description"
                    />
                  </td>
                  <td style={{ padding: "8px" }}>
                    <button
                      onClick={() => handleDeleteRow(idx)}
                      className="p-2 rounded-lg transition-colors cursor-pointer"
                      style={{ color: "var(--text-tertiary)" }}
                      onMouseEnter={(e) => {
                        e.currentTarget.style.background = "var(--color-error-bg)";
                        e.currentTarget.style.color = "var(--color-error)";
                      }}
                      onMouseLeave={(e) => {
                        e.currentTarget.style.background = "transparent";
                        e.currentTarget.style.color = "var(--text-tertiary)";
                      }}
                      title="Supprimer"
                    >
                      <FontAwesomeIcon icon={faTrash} />
                    </button>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
