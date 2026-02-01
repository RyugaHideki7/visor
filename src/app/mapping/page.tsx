"use client";

import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faPlus, faTrash } from "@fortawesome/free-solid-svg-icons";
import { Select, SelectItem, Button, Input } from "@heroui/react";

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

  const handleSelectLine = (keys: any) => {
    const first = Array.from(keys)[0] as string | undefined;
    setSelectedLineId(first ?? "");
  };

  const lineOptions = useMemo(() => [
    { key: "", name: "Sélectionner une ligne" },
    ...lines.map((l) => ({ key: String(l.id), name: l.name })),
  ], [lines]);

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
          <Button
            onPress={handleAddRow}
            variant="bordered"
            startContent={<FontAwesomeIcon icon={faPlus} />}
            className="border-[var(--border-default)] text-[var(--text-primary)] hover:bg-[var(--bg-hover)]"
          >
            Ajouter
          </Button>
          <Button
            onPress={handleSave}
            isLoading={isSaving}
            color="primary"
            className="bg-[var(--button-primary-bg)] text-white hover:bg-[var(--button-primary-hover)]"
          >
            Sauvegarder
          </Button>
        </div>
      </div>

      <div className="max-w-md">
        <Select
          items={lineOptions}
          selectedKeys={new Set([selectedLineId])}
          onSelectionChange={handleSelectLine}
          placeholder="Sélectionner une ligne"
          variant="bordered"
          className="w-full"
          classNames={{
            label: "text-[var(--text-secondary)]",
            trigger: "bg-[var(--bg-tertiary)] border-[var(--border-default)] hover:border-[var(--accent-primary)] data-[open=true]:border-[var(--accent-primary)]",
            value: "text-[var(--text-primary)]",
            popoverContent: "bg-[var(--bg-secondary)] border border-[var(--border-default)]",
          }}
        >
          {(item) => (
            <SelectItem key={item.key} textValue={item.name} className="text-[var(--text-primary)] hover:bg-[var(--bg-hover)]">
              {item.name}
            </SelectItem>
          )}
        </Select>
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
                    <Input
                      type="text"
                      value={r.sql_field}
                      onValueChange={(val) => handleUpdateRow(idx, { sql_field: val })}
                      placeholder="ex: YSSCC_0"
                      size="sm"
                      variant="bordered"
                      classNames={{
                        inputWrapper: "bg-[var(--bg-tertiary)] border-[var(--border-default)] hover:border-[var(--accent-primary)] focus-within:border-[var(--accent-primary)]",
                        input: "text-[var(--text-primary)] placeholder:text-[var(--text-placeholder)] text-sm",
                      }}
                    />
                  </td>
                  <td style={{ padding: "8px" }}>
                    <Input
                      type="text"
                      value={r.file_column ?? ""}
                      onValueChange={(val) => handleUpdateRow(idx, { file_column: val })}
                      placeholder="ex: 0"
                      size="sm"
                      variant="bordered"
                      classNames={{
                        inputWrapper: "bg-[var(--bg-tertiary)] border-[var(--border-default)] hover:border-[var(--accent-primary)] focus-within:border-[var(--accent-primary)]",
                        input: "text-[var(--text-primary)] placeholder:text-[var(--text-placeholder)] text-sm",
                      }}
                    />
                  </td>
                  <td style={{ padding: "8px" }}>
                    <Input
                      type="text"
                      value={r.parameter ?? ""}
                      onValueChange={(val) => handleUpdateRow(idx, { parameter: val })}
                      placeholder="ex: site"
                      size="sm"
                      variant="bordered"
                      classNames={{
                        inputWrapper: "bg-[var(--bg-tertiary)] border-[var(--border-default)] hover:border-[var(--accent-primary)] focus-within:border-[var(--accent-primary)]",
                        input: "text-[var(--text-primary)] placeholder:text-[var(--text-placeholder)] text-sm",
                      }}
                    />
                  </td>
                  <td style={{ padding: "8px" }}>
                    <Input
                      type="text"
                      value={r.transformation ?? ""}
                      onValueChange={(val) => handleUpdateRow(idx, { transformation: val })}
                      placeholder="ex: date"
                      size="sm"
                      variant="bordered"
                      classNames={{
                        inputWrapper: "bg-[var(--bg-tertiary)] border-[var(--border-default)] hover:border-[var(--accent-primary)] focus-within:border-[var(--accent-primary)]",
                        input: "text-[var(--text-primary)] placeholder:text-[var(--text-placeholder)] text-sm",
                      }}
                    />
                  </td>
                  <td style={{ padding: "8px" }}>
                    <Input
                      type="text"
                      value={r.description ?? ""}
                      onValueChange={(val) => handleUpdateRow(idx, { description: val })}
                      placeholder="Description"
                      size="sm"
                      variant="bordered"
                      classNames={{
                        inputWrapper: "bg-[var(--bg-tertiary)] border-[var(--border-default)] hover:border-[var(--accent-primary)] focus-within:border-[var(--accent-primary)]",
                        input: "text-[var(--text-primary)] placeholder:text-[var(--text-placeholder)] text-sm",
                      }}
                    />
                  </td>
                  <td style={{ padding: "8px" }}>
                    <Button
                      isIconOnly
                      size="sm"
                      variant="light"
                      onPress={() => handleDeleteRow(idx)}
                      className="text-[var(--text-tertiary)] hover:bg-[var(--color-error-bg)] hover:text-[var(--color-error)]"
                      title="Supprimer"
                    >
                      <FontAwesomeIcon icon={faTrash} />
                    </Button>
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
