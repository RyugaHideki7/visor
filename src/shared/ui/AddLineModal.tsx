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
        className="absolute inset-0 bg-black/60 backdrop-blur-sm"
        onClick={handleClose}
      />
      
      {/* Modal */}
      <div className="relative w-full max-w-2xl mx-4 bg-[#202020] border border-[#333] rounded-xl shadow-2xl">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-[#333]">
          <h2 className="text-lg font-semibold text-white">
            {initialData?.id ? "Modifier la ligne" : "Ajouter une ligne de production"}
          </h2>
          <button
            onClick={handleClose}
            className="p-2 rounded-lg hover:bg-[#333] transition-colors cursor-pointer"
          >
            <FontAwesomeIcon icon={faXmark} className="text-gray-400" />
          </button>
        </div>

        {/* Body */}
        <div className="px-6 py-4 flex flex-col gap-4">
          <div>
            <label className="block text-sm text-gray-400 mb-2">Nom de la ligne</label>
            <input
              type="text"
              placeholder="ex: Ligne A"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              className="w-full px-3 py-2 bg-[#191919] border border-[#333] rounded-lg text-white focus:border-blue-500 focus:outline-none"
            />
          </div>

          <div>
            <label className="block text-sm text-gray-400 mb-2">Chemin de surveillance</label>
            <div className="relative">
              <input
                type="text"
                placeholder="C:/Production/LigneA"
                value={formData.path}
                onChange={(e) => setFormData({ ...formData, path: e.target.value })}
                className="w-full px-3 py-2 pr-10 bg-[#191919] border border-[#333] rounded-lg text-white focus:border-blue-500 focus:outline-none"
              />
              <button className="absolute right-2 top-1/2 -translate-y-1/2 p-1 hover:bg-[#333] rounded cursor-pointer">
                <FontAwesomeIcon icon={faFolderOpen} className="text-gray-400" />
              </button>
            </div>
          </div>

          <div className="flex gap-4">
            <div className="flex-1">
              <label className="block text-sm text-gray-400 mb-2">Préfixe des fichiers</label>
              <input
                type="text"
                placeholder="STOCK"
                value={formData.prefix}
                onChange={(e) => setFormData({ ...formData, prefix: e.target.value })}
                className="w-full px-3 py-2 bg-[#191919] border border-[#333] rounded-lg text-white focus:border-blue-500 focus:outline-none"
              />
            </div>
            <div className="flex-1">
              <label className="block text-sm text-gray-400 mb-2">Dossier d'archivage</label>
              <input
                type="text"
                placeholder="C:/Archive/LigneA"
                value={formData.archived_path ?? ""}
                onChange={(e) => setFormData({ ...formData, archived_path: e.target.value })}
                className="w-full px-3 py-2 bg-[#191919] border border-[#333] rounded-lg text-white focus:border-blue-500 focus:outline-none"
              />
            </div>
          </div>

          <div className="flex gap-4">
            <div className="flex-1">
              <label className="block text-sm text-gray-400 mb-2">Intervalle marche (min)</label>
              <input
                type="number"
                value={formData.interval_check}
                onChange={(e) => setFormData({ ...formData, interval_check: parseInt(e.target.value) || 0 })}
                className="w-full px-3 py-2 bg-[#191919] border border-[#333] rounded-lg text-white focus:border-blue-500 focus:outline-none"
              />
            </div>
            <div className="flex-1">
              <label className="block text-sm text-gray-400 mb-2">Intervalle arrêt (min)</label>
              <input
                type="number"
                value={formData.interval_alert}
                onChange={(e) => setFormData({ ...formData, interval_alert: parseInt(e.target.value) || 0 })}
                className="w-full px-3 py-2 bg-[#191919] border border-[#333] rounded-lg text-white focus:border-blue-500 focus:outline-none"
              />
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="flex justify-end gap-3 px-6 py-4 border-t border-[#333]">
          <button
            onClick={handleClose}
            className="px-4 py-2 bg-[#333] text-white rounded-lg hover:bg-[#444] transition-colors cursor-pointer"
          >
            Annuler
          </button>
          <button
            onClick={handleSave}
            disabled={isSaving}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors cursor-pointer disabled:opacity-50"
          >
            {isSaving ? "..." : "Sauvegarder"}
          </button>
        </div>
      </div>
    </div>
  );
}
