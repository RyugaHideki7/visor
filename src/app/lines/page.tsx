"use client";

import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faPlus, faTrash, faPlay, faPause } from "@fortawesome/free-solid-svg-icons";
import { AddLineModal, type LineFormData } from "@/shared/ui/AddLineModal";

interface Line {
    id?: number;
    name: string;
    path: string;
    prefix: string;
    interval_check: number;
    interval_alert: number;
    archived_path?: string;
    active: boolean;
}

export default function LinesPage() {
    const [lines, setLines] = useState<Line[]>([]);
    const [isModalOpen, setIsModalOpen] = useState(false);

    const fetchLines = async () => {
        try {
            const result = await invoke<Line[]>("get_lines");
            setLines(result);
        } catch (error) {
            console.error("Failed to fetch lines:", error);
        }
    };

    useEffect(() => {
        fetchLines();
    }, []);

    const handleSave = async (line: LineFormData) => {
        await invoke("save_line", { line });
        fetchLines();
    };

    const handleDelete = async (id: number) => {
        try {
            await invoke("delete_line", { id });
            fetchLines();
        } catch (error) {
            console.error("Failed to delete line:", error);
        }
    };

    const handleToggleActive = async (line: Line) => {
        try {
            await invoke("toggle_line_active", { id: line.id, active: !line.active });
            fetchLines();
        } catch (error) {
            console.error("Failed to toggle line:", error);
        }
    };

    return (
        <div className="p-8 flex flex-col gap-6">
            <div className="flex justify-between items-center">
                <h1 className="text-2xl font-bold text-white">Lignes de production</h1>
                <button
                    onClick={() => setIsModalOpen(true)}
                    className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors cursor-pointer"
                >
                    <FontAwesomeIcon icon={faPlus} />
                    Ajouter une ligne
                </button>
            </div>

            <div className="bg-[#202020] rounded-lg border border-[#333] overflow-hidden">
                <table className="w-full">
                    <thead>
                        <tr className="border-b border-[#333] text-left text-sm text-gray-400">
                            <th className="px-4 py-3 font-medium">NOM</th>
                            <th className="px-4 py-3 font-medium">CHEMIN</th>
                            <th className="px-4 py-3 font-medium">PRÉFIXE</th>
                            <th className="px-4 py-3 font-medium">STATUT</th>
                            <th className="px-4 py-3 font-medium">ACTIONS</th>
                        </tr>
                    </thead>
                    <tbody>
                        {lines.length === 0 ? (
                            <tr>
                                <td colSpan={5} className="px-4 py-8 text-center text-gray-500">
                                    Aucune ligne configurée.
                                </td>
                            </tr>
                        ) : (
                            lines.map((line) => (
                                <tr key={line.id} className="border-b border-[#333] last:border-b-0 hover:bg-[#252525]">
                                    <td className="px-4 py-3 font-medium text-white">{line.name}</td>
                                    <td className="px-4 py-3 text-xs text-gray-400">{line.path}</td>
                                    <td className="px-4 py-3 text-gray-300">{line.prefix}</td>
                                    <td className="px-4 py-3">
                                        <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${
                                            line.active 
                                                ? "bg-green-500/20 text-green-400" 
                                                : "bg-gray-500/20 text-gray-400"
                                        }`}>
                                            {line.active ? "Active" : "Inactif"}
                                        </span>
                                    </td>
                                    <td className="px-4 py-3">
                                        <div className="flex gap-2">
                                            <button
                                                onClick={() => handleToggleActive(line)}
                                                className="p-2 rounded-lg hover:bg-[#333] transition-colors cursor-pointer"
                                                title={line.active ? "Mettre en pause" : "Démarrer"}
                                            >
                                                <FontAwesomeIcon 
                                                    icon={line.active ? faPause : faPlay} 
                                                    className="text-gray-400 hover:text-white"
                                                />
                                            </button>
                                            <button
                                                onClick={() => line.id && handleDelete(line.id)}
                                                className="p-2 rounded-lg hover:bg-red-500/20 transition-colors cursor-pointer"
                                                title="Supprimer"
                                            >
                                                <FontAwesomeIcon 
                                                    icon={faTrash} 
                                                    className="text-gray-400 hover:text-red-400"
                                                />
                                            </button>
                                        </div>
                                    </td>
                                </tr>
                            ))
                        )}
                    </tbody>
                </table>
            </div>

            <AddLineModal
                isOpen={isModalOpen}
                onClose={() => setIsModalOpen(false)}
                onSave={handleSave}
            />
        </div>
    );
}
