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
                <h1 className="text-2xl font-bold" style={{ color: "var(--text-primary)" }}>Lignes de production</h1>
                <button
                    onClick={() => setIsModalOpen(true)}
                    className="flex items-center gap-2 px-4 py-2 rounded-lg transition-colors cursor-pointer"
                    style={{ background: "var(--button-primary-bg)", color: "white" }}
                    onMouseEnter={(e) => e.currentTarget.style.background = "var(--button-primary-hover)"}
                    onMouseLeave={(e) => e.currentTarget.style.background = "var(--button-primary-bg)"}
                >
                    <FontAwesomeIcon icon={faPlus} />
                    Ajouter une ligne
                </button>
            </div>

            <div style={{ background: "var(--bg-secondary)", borderRadius: 8, border: "1px solid var(--border-default)", overflow: "hidden" }}>
                <table className="w-full">
                    <thead>
                        <tr style={{ borderBottom: "1px solid var(--border-default)", textAlign: "left", fontSize: 14, color: "var(--text-tertiary)" }}>
                            <th style={{ padding: "12px 16px", fontWeight: 500 }}>NOM</th>
                            <th style={{ padding: "12px 16px", fontWeight: 500 }}>CHEMIN</th>
                            <th style={{ padding: "12px 16px", fontWeight: 500 }}>PRÉFIXE</th>
                            <th style={{ padding: "12px 16px", fontWeight: 500 }}>STATUT</th>
                            <th style={{ padding: "12px 16px", fontWeight: 500 }}>ACTIONS</th>
                        </tr>
                    </thead>
                    <tbody>
                        {lines.length === 0 ? (
                            <tr>
                                <td colSpan={5} style={{ padding: "32px 16px", textAlign: "center", color: "var(--text-tertiary)" }}>
                                    Aucune ligne configurée.
                                </td>
                            </tr>
                        ) : (
                            lines.map((line) => (
                                <tr 
                                    key={line.id} 
                                    style={{ borderBottom: "1px solid var(--border-default)" }}
                                    onMouseEnter={(e) => e.currentTarget.style.background = "var(--bg-hover)"}
                                    onMouseLeave={(e) => e.currentTarget.style.background = "transparent"}
                                >
                                    <td style={{ padding: "12px 16px", fontWeight: 500, color: "var(--text-primary)" }}>{line.name}</td>
                                    <td style={{ padding: "12px 16px", fontSize: 12, color: "var(--text-tertiary)" }}>{line.path}</td>
                                    <td style={{ padding: "12px 16px", color: "var(--text-secondary)" }}>{line.prefix}</td>
                                    <td style={{ padding: "12px 16px" }}>
                                        <span 
                                            className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium"
                                            style={{
                                                background: line.active ? "var(--status-running-bg)" : "var(--status-stopped-bg)",
                                                color: line.active ? "var(--status-running)" : "var(--status-stopped)"
                                            }}
                                        >
                                            {line.active ? "Active" : "Inactif"}
                                        </span>
                                    </td>
                                    <td style={{ padding: "12px 16px" }}>
                                        <div className="flex gap-2">
                                            <button
                                                onClick={() => handleToggleActive(line)}
                                                className="p-2 rounded-lg transition-colors cursor-pointer"
                                                style={{ color: "var(--text-tertiary)" }}
                                                onMouseEnter={(e) => {
                                                    e.currentTarget.style.background = "var(--bg-active)";
                                                    e.currentTarget.style.color = "var(--text-primary)";
                                                }}
                                                onMouseLeave={(e) => {
                                                    e.currentTarget.style.background = "transparent";
                                                    e.currentTarget.style.color = "var(--text-tertiary)";
                                                }}
                                                title={line.active ? "Mettre en pause" : "Démarrer"}
                                            >
                                                <FontAwesomeIcon icon={line.active ? faPause : faPlay} />
                                            </button>
                                            <button
                                                onClick={() => line.id && handleDelete(line.id)}
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
