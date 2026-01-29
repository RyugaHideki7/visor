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
          <h1 className="text-2xl font-bold text-white">Mapping</h1>
          <p className="text-gray-400 text-sm">
            Associez les colonnes du fichier et/ou des paramètres de la ligne aux champs SQL.
          </p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={handleAddRow}
            className="flex items-center gap-2 px-4 py-2 bg-[#333] text-white rounded-lg hover:bg-[#444] transition-colors cursor-pointer"
          >
            <FontAwesomeIcon icon={faPlus} />
            Ajouter
          </button>
          <button
            onClick={handleSave}
            disabled={isSaving}
            className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors cursor-pointer disabled:opacity-50"
          >
            {isSaving ? "..." : "Sauvegarder"}
          </button>
        </div>
      </div>

      <div className="max-w-md">
        <label className="block text-sm text-gray-400 mb-2">Ligne</label>
        <select
          value={selectedLineId}
          onChange={(e) => setSelectedLineId(e.target.value)}
          className="w-full px-3 py-2 bg-[#191919] border border-[#333] rounded-lg text-white focus:border-blue-500 focus:outline-none cursor-pointer"
        >
          <option value="">Sélectionner une ligne</option>
          {lines.map((l) => (
            <option key={l.id} value={String(l.id)}>
              {l.name}
            </option>
          ))}
        </select>
      </div>

      <div className="bg-[#202020] rounded-lg border border-[#333] overflow-hidden">
        <table className="w-full">
          <thead>
            <tr className="border-b border-[#333] text-left text-sm text-gray-400">
              <th className="px-4 py-3 font-medium">CHAMP SQL</th>
              <th className="px-4 py-3 font-medium">COLONNE</th>
              <th className="px-4 py-3 font-medium">PARAMÈTRE</th>
              <th className="px-4 py-3 font-medium">TRANSFORMATION</th>
              <th className="px-4 py-3 font-medium">DESCRIPTION</th>
              <th className="px-4 py-3 font-medium">ACTIONS</th>
            </tr>
          </thead>
          <tbody>
            {rows.length === 0 ? (
              <tr>
                <td colSpan={6} className="px-4 py-8 text-center text-gray-500">
                  Aucun mapping pour cette ligne.
                </td>
              </tr>
            ) : (
              rows.map((r, idx) => (
                <tr key={`${r.id ?? "new"}-${idx}`} className="border-b border-[#333] last:border-b-0">
                  <td className="px-2 py-2">
                    <input
                      type="text"
                      value={r.sql_field}
                      onChange={(e) => handleUpdateRow(idx, { sql_field: e.target.value })}
                      className="w-full px-2 py-1 bg-[#191919] border border-[#333] rounded text-white text-sm focus:border-blue-500 focus:outline-none"
                      placeholder="ex: YSCC_0"
                    />
                  </td>
                  <td className="px-2 py-2">
                    <input
                      type="text"
                      value={r.file_column ?? ""}
                      onChange={(e) => handleUpdateRow(idx, { file_column: e.target.value })}
                      className="w-full px-2 py-1 bg-[#191919] border border-[#333] rounded text-white text-sm focus:border-blue-500 focus:outline-none"
                      placeholder="ex: 0"
                    />
                  </td>
                  <td className="px-2 py-2">
                    <input
                      type="text"
                      value={r.parameter ?? ""}
                      onChange={(e) => handleUpdateRow(idx, { parameter: e.target.value })}
                      className="w-full px-2 py-1 bg-[#191919] border border-[#333] rounded text-white text-sm focus:border-blue-500 focus:outline-none"
                      placeholder="ex: site"
                    />
                  </td>
                  <td className="px-2 py-2">
                    <input
                      type="text"
                      value={r.transformation ?? ""}
                      onChange={(e) => handleUpdateRow(idx, { transformation: e.target.value })}
                      className="w-full px-2 py-1 bg-[#191919] border border-[#333] rounded text-white text-sm focus:border-blue-500 focus:outline-none"
                      placeholder="ex: date"
                    />
                  </td>
                  <td className="px-2 py-2">
                    <input
                      type="text"
                      value={r.description ?? ""}
                      onChange={(e) => handleUpdateRow(idx, { description: e.target.value })}
                      className="w-full px-2 py-1 bg-[#191919] border border-[#333] rounded text-white text-sm focus:border-blue-500 focus:outline-none"
                      placeholder="Description"
                    />
                  </td>
                  <td className="px-2 py-2">
                    <button
                      onClick={() => handleDeleteRow(idx)}
                      className="p-2 rounded-lg hover:bg-red-500/20 transition-colors cursor-pointer"
                      title="Supprimer"
                    >
                      <FontAwesomeIcon icon={faTrash} className="text-gray-400 hover:text-red-400" />
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
