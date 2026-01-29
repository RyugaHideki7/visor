"use client";

import { useState, useEffect } from "react";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faFolderOpen, faXmark } from "@fortawesome/free-solid-svg-icons";

export interface LineFormData {
  id?: number;
  name: string;
  path: string;
  prefix: string;
  interval_check: number;
  interval_alert: number;
  archived_path?: string;
  active: boolean;
  site?: string;
  unite?: string;
  flag_dec?: string;
  code_ligne?: string;
  log_path?: string;
  file_format?: string;
}

interface AddLineModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (line: LineFormData) => Promise<void>;
  initialData?: LineFormData;
}

const defaultLine: LineFormData = {
  name: "",
  path: "",
  prefix: "STOCK",
  interval_check: 60,
  interval_alert: 120,
  active: true,
  site: "",
  unite: "",
  flag_dec: "",
  code_ligne: "",
  log_path: "",
  file_format: "ATEIS",
};

export function AddLineModal({ isOpen, onClose, onSave, initialData }: AddLineModalProps) {
  const [formData, setFormData] = useState<LineFormData>(initialData ?? defaultLine);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    if (isOpen) {
      setFormData(initialData ?? defaultLine);
    }
  }, [isOpen, initialData]);

  const handleSave = async () => {
    setIsSaving(true);
    try {
      await onSave(formData);
      setFormData(defaultLine);
      onClose();
    } catch (error) {
      console.error("Failed to save line:", error);
    } finally {
      setIsSaving(false);
    }
  };

  const handleClose = () => {
    setFormData(defaultLine);
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Backdrop */}
      <div 
        className="absolute inset-0 backdrop-blur-sm"
        style={{ background: "rgba(0,0,0,0.5)" }}
        onClick={handleClose}
      />
      
      {/* Modal */}
      <div 
        className="relative w-full max-w-2xl mx-4 rounded-xl"
        style={{ background: "var(--bg-secondary)", border: "1px solid var(--border-default)", boxShadow: "var(--shadow-lg)" }}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4" style={{ borderBottom: "1px solid var(--border-default)" }}>
          <h2 className="text-lg font-semibold" style={{ color: "var(--text-primary)" }}>
            {initialData?.id ? "Modifier la ligne" : "Ajouter une ligne de production"}
          </h2>
          <button
            onClick={handleClose}
            className="p-2 rounded-lg transition-colors cursor-pointer"
            style={{ color: "var(--text-tertiary)" }}
            onMouseEnter={(e) => e.currentTarget.style.background = "var(--bg-hover)"}
            onMouseLeave={(e) => e.currentTarget.style.background = "transparent"}
          >
            <FontAwesomeIcon icon={faXmark} />
          </button>
        </div>

        {/* Body */}
        <div className="px-6 py-4 flex flex-col gap-4">
          <div>
            <label className="block text-sm mb-2" style={{ color: "var(--text-secondary)" }}>Nom de la ligne</label>
            <input
              type="text"
              placeholder="ex: Ligne A"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              className="w-full px-3 py-2 rounded-lg focus:outline-none"
              style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-default)", color: "var(--text-primary)" }}
              onFocus={(e) => e.currentTarget.style.borderColor = "var(--accent-primary)"}
              onBlur={(e) => e.currentTarget.style.borderColor = "var(--border-default)"}
            />
          </div>

          <div>
            <label className="block text-sm mb-2" style={{ color: "var(--text-secondary)" }}>Chemin de surveillance</label>
            <div className="relative">
              <input
                type="text"
                placeholder="C:/Production/LigneA"
                value={formData.path}
                onChange={(e) => setFormData({ ...formData, path: e.target.value })}
                className="w-full px-3 py-2 pr-10 rounded-lg focus:outline-none"
                style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-default)", color: "var(--text-primary)" }}
                onFocus={(e) => e.currentTarget.style.borderColor = "var(--accent-primary)"}
                onBlur={(e) => e.currentTarget.style.borderColor = "var(--border-default)"}
              />
              <button 
                className="absolute right-2 top-1/2 -translate-y-1/2 p-1 rounded cursor-pointer"
                style={{ color: "var(--text-tertiary)" }}
              >
                <FontAwesomeIcon icon={faFolderOpen} />
              </button>
            </div>
          </div>

          <div className="flex gap-4">
            <div className="flex-1">
              <label className="block text-sm mb-2" style={{ color: "var(--text-secondary)" }}>Préfixe des fichiers</label>
              <input
                type="text"
                placeholder="STOCK"
                value={formData.prefix}
                onChange={(e) => setFormData({ ...formData, prefix: e.target.value })}
                className="w-full px-3 py-2 rounded-lg focus:outline-none"
                style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-default)", color: "var(--text-primary)" }}
                onFocus={(e) => e.currentTarget.style.borderColor = "var(--accent-primary)"}
                onBlur={(e) => e.currentTarget.style.borderColor = "var(--border-default)"}
              />
            </div>
            <div className="flex-1">
              <label className="block text-sm mb-2" style={{ color: "var(--text-secondary)" }}>Dossier d'archivage</label>
              <input
                type="text"
                placeholder="C:/Archive/LigneA"
                value={formData.archived_path ?? ""}
                onChange={(e) => setFormData({ ...formData, archived_path: e.target.value })}
                className="w-full px-3 py-2 rounded-lg focus:outline-none"
                style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-default)", color: "var(--text-primary)" }}
                onFocus={(e) => e.currentTarget.style.borderColor = "var(--accent-primary)"}
                onBlur={(e) => e.currentTarget.style.borderColor = "var(--border-default)"}
              />
            </div>
          </div>

          <div className="flex gap-4">
            <div className="flex-1">
              <label className="block text-sm mb-2" style={{ color: "var(--text-secondary)" }}>Intervalle marche (min)</label>
              <input
                type="number"
                value={formData.interval_check}
                onChange={(e) => setFormData({ ...formData, interval_check: parseInt(e.target.value) || 0 })}
                className="w-full px-3 py-2 rounded-lg focus:outline-none"
                style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-default)", color: "var(--text-primary)" }}
                onFocus={(e) => e.currentTarget.style.borderColor = "var(--accent-primary)"}
                onBlur={(e) => e.currentTarget.style.borderColor = "var(--border-default)"}
              />
            </div>
            <div className="flex-1">
              <label className="block text-sm mb-2" style={{ color: "var(--text-secondary)" }}>Intervalle arrêt (min)</label>
              <input
                type="number"
                value={formData.interval_alert}
                onChange={(e) => setFormData({ ...formData, interval_alert: parseInt(e.target.value) || 0 })}
                className="w-full px-3 py-2 rounded-lg focus:outline-none"
                style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-default)", color: "var(--text-primary)" }}
                onFocus={(e) => e.currentTarget.style.borderColor = "var(--accent-primary)"}
                onBlur={(e) => e.currentTarget.style.borderColor = "var(--border-default)"}
              />
            </div>
          </div>

          {/* Separator */}
          <div className="my-2" style={{ borderTop: "1px solid var(--border-default)" }} />
          <p className="text-xs font-medium uppercase" style={{ color: "var(--text-tertiary)" }}>Paramètres SQL</p>

          <div className="flex gap-4">
            <div className="flex-1">
              <label className="block text-sm mb-2" style={{ color: "var(--text-secondary)" }}>Site</label>
              <input
                type="text"
                placeholder="ex: SITE01"
                value={formData.site ?? ""}
                onChange={(e) => setFormData({ ...formData, site: e.target.value })}
                className="w-full px-3 py-2 rounded-lg focus:outline-none"
                style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-default)", color: "var(--text-primary)" }}
                onFocus={(e) => e.currentTarget.style.borderColor = "var(--accent-primary)"}
                onBlur={(e) => e.currentTarget.style.borderColor = "var(--border-default)"}
              />
            </div>
            <div className="flex-1">
              <label className="block text-sm mb-2" style={{ color: "var(--text-secondary)" }}>Unité</label>
              <input
                type="text"
                placeholder="ex: UNITE01"
                value={formData.unite ?? ""}
                onChange={(e) => setFormData({ ...formData, unite: e.target.value })}
                className="w-full px-3 py-2 rounded-lg focus:outline-none"
                style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-default)", color: "var(--text-primary)" }}
                onFocus={(e) => e.currentTarget.style.borderColor = "var(--accent-primary)"}
                onBlur={(e) => e.currentTarget.style.borderColor = "var(--border-default)"}
              />
            </div>
          </div>

          <div className="flex gap-4">
            <div className="flex-1">
              <label className="block text-sm mb-2" style={{ color: "var(--text-secondary)" }}>Flag Déc</label>
              <input
                type="text"
                placeholder="ex: 1"
                value={formData.flag_dec ?? ""}
                onChange={(e) => setFormData({ ...formData, flag_dec: e.target.value })}
                className="w-full px-3 py-2 rounded-lg focus:outline-none"
                style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-default)", color: "var(--text-primary)" }}
                onFocus={(e) => e.currentTarget.style.borderColor = "var(--accent-primary)"}
                onBlur={(e) => e.currentTarget.style.borderColor = "var(--border-default)"}
              />
            </div>
            <div className="flex-1">
              <label className="block text-sm mb-2" style={{ color: "var(--text-secondary)" }}>Code Ligne</label>
              <input
                type="text"
                placeholder="ex: L01"
                value={formData.code_ligne ?? ""}
                onChange={(e) => setFormData({ ...formData, code_ligne: e.target.value })}
                className="w-full px-3 py-2 rounded-lg focus:outline-none"
                style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-default)", color: "var(--text-primary)" }}
                onFocus={(e) => e.currentTarget.style.borderColor = "var(--accent-primary)"}
                onBlur={(e) => e.currentTarget.style.borderColor = "var(--border-default)"}
              />
            </div>
          </div>

          <div className="flex gap-4">
            <div className="flex-1">
              <label className="block text-sm mb-2" style={{ color: "var(--text-secondary)" }}>Dossier logs</label>
              <input
                type="text"
                placeholder="C:/Logs/LigneA"
                value={formData.log_path ?? ""}
                onChange={(e) => setFormData({ ...formData, log_path: e.target.value })}
                className="w-full px-3 py-2 rounded-lg focus:outline-none"
                style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-default)", color: "var(--text-primary)" }}
                onFocus={(e) => e.currentTarget.style.borderColor = "var(--accent-primary)"}
                onBlur={(e) => e.currentTarget.style.borderColor = "var(--border-default)"}
              />
            </div>
            <div className="flex-1">
              <label className="block text-sm mb-2" style={{ color: "var(--text-secondary)" }}>Format fichier</label>
              <select
                value={formData.file_format ?? "ATEIS"}
                onChange={(e) => setFormData({ ...formData, file_format: e.target.value })}
                className="w-full px-3 py-2 rounded-lg focus:outline-none cursor-pointer"
                style={{ background: "var(--bg-tertiary)", border: "1px solid var(--border-default)", color: "var(--text-primary)" }}
              >
                <option value="ATEIS">ATEIS</option>
                <option value="LOGITRON">LOGITRON</option>
              </select>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-3 px-6 py-4" style={{ borderTop: "1px solid var(--border-default)" }}>
          <button
            onClick={handleClose}
            className="px-4 py-2 rounded-lg transition-colors cursor-pointer"
            style={{ background: "var(--button-secondary-bg)", color: "var(--text-primary)" }}
            onMouseEnter={(e) => e.currentTarget.style.background = "var(--button-secondary-hover)"}
            onMouseLeave={(e) => e.currentTarget.style.background = "var(--button-secondary-bg)"}
          >
            Annuler
          </button>
          <button
            onClick={handleSave}
            disabled={isSaving}
            className="px-4 py-2 rounded-lg transition-colors cursor-pointer disabled:opacity-50"
            style={{ background: "var(--button-primary-bg)", color: "white" }}
            onMouseEnter={(e) => !isSaving && (e.currentTarget.style.background = "var(--button-primary-hover)")}
            onMouseLeave={(e) => e.currentTarget.style.background = "var(--button-primary-bg)"}
          >
            {isSaving ? "..." : "Sauvegarder"}
          </button>
        </div>
      </div>
    </div>
  );
}
