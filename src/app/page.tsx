"use client";

import { useEffect, useState } from "react";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faIndustry, faClock, faExclamationTriangle } from "@fortawesome/free-solid-svg-icons";
import { invoke } from "@tauri-apps/api/core";

interface LineStatus {
    id: number;
    name: string;
    active: boolean;
    pending_files: number;
    last_processed?: string;
    total_processed: number;
    status: "MARCHE" | "ALERTE" | "ARRET" | "ERREUR";
}

const statusStyles = {
    MARCHE: { bg: "var(--status-running-bg)", color: "var(--status-running)", bar: "var(--status-running)" },
    ALERTE: { bg: "var(--status-alert-bg)", color: "var(--status-alert)", bar: "var(--status-alert)" },
    ARRET: { bg: "var(--status-stopped-bg)", color: "var(--status-stopped)", bar: "var(--status-stopped)" },
    ERREUR: { bg: "var(--status-error-bg)", color: "var(--status-error)", bar: "var(--status-error)" },
};

export default function Dashboard() {
    const [lineStatuses, setLineStatuses] = useState<LineStatus[]>([]);

    const fetchStatus = async () => {
        try {
            const snapshot = await invoke<LineStatus[]>("get_dashboard_snapshot");
            setLineStatuses(snapshot);
        } catch (error) {
            console.error("Dashboard fetch error:", error);
        }
    };

    useEffect(() => {
        fetchStatus();
        const interval = setInterval(fetchStatus, 5000);
        return () => clearInterval(interval);
    }, []);

    const getProgressValue = (status: string) => {
        if (status === "MARCHE") return 100;
        if (status === "ALERTE") return 50;
        return 20;
    };

    return (
        <div className="p-8 flex flex-col gap-8">
            <div>
                <h1 className="text-2xl font-bold" style={{ color: "var(--text-primary)" }}>Tableau de bord</h1>
                <p className="text-sm" style={{ color: "var(--text-secondary)" }}>Statut en temps réel des lignes de production</p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {lineStatuses.map((line) => {
                    const styles = statusStyles[line.status] || statusStyles.ARRET;
                    return (
                        <div 
                            key={line.id} 
                            style={{ 
                                background: "var(--bg-secondary)", 
                                borderRadius: 12, 
                                border: "1px solid var(--border-default)",
                                overflow: "hidden" 
                            }}
                        >
                            <div className="flex justify-between items-start p-5">
                                <div className="flex gap-3 items-center">
                                    <div 
                                        className="p-2 rounded-lg"
                                        style={{ background: styles.bg, color: styles.color }}
                                    >
                                        <FontAwesomeIcon icon={faIndustry} className="h-5 w-5" />
                                    </div>
                                    <div>
                                        <p className="text-md font-bold" style={{ color: "var(--text-primary)" }}>{line.name}</p>
                                        <p className="text-xs" style={{ color: "var(--text-tertiary)" }}>ID: {line.id}</p>
                                    </div>
                                </div>
                                <span 
                                    className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium"
                                    style={{ background: styles.bg, color: styles.color }}
                                >
                                    {line.status}
                                </span>
                            </div>
                            
                            <div className="mx-5 h-px" style={{ background: "var(--border-default)" }} />
                            
                            <div className="p-5 flex flex-col gap-4">
                                <div className="flex flex-col gap-2">
                                    <div className="flex justify-between items-center text-xs">
                                        <span style={{ color: "var(--text-tertiary)" }}>Dernière activité</span>
                                        <span style={{ color: "var(--text-secondary)" }}>
                                            {line.last_processed || "Jamais"}
                                        </span>
                                    </div>
                                    <div 
                                        className="w-full h-2 rounded-full overflow-hidden"
                                        style={{ background: "var(--bg-tertiary)" }}
                                    >
                                        <div 
                                            className="h-full transition-all duration-300"
                                            style={{ width: `${getProgressValue(line.status)}%`, background: styles.bar }}
                                        />
                                    </div>
                                </div>

                                <div className="grid grid-cols-2 gap-4">
                                    <div 
                                        className="p-3 rounded-lg flex flex-col items-center"
                                        style={{ background: "var(--bg-tertiary)" }}
                                    >
                                        <p className="text-[10px] uppercase font-bold" style={{ color: "var(--text-tertiary)" }}>Traités</p>
                                        <p className="text-lg font-bold" style={{ color: "var(--color-success)" }}>{line.total_processed}</p>
                                    </div>
                                    <div 
                                        className="p-3 rounded-lg flex flex-col items-center"
                                        style={{ background: "var(--bg-tertiary)" }}
                                    >
                                        <p className="text-[10px] uppercase font-bold" style={{ color: "var(--text-tertiary)" }}>En attente</p>
                                        <p className="text-lg font-bold" style={{ color: "var(--color-warning)" }}>{line.pending_files}</p>
                                    </div>
                                </div>

                                <div className="flex gap-2 items-center text-[10px]" style={{ color: "var(--text-tertiary)" }}>
                                    <FontAwesomeIcon icon={faClock} />
                                    <span>Actualisé toutes les 5 secondes</span>
                                </div>
                            </div>
                        </div>
                    );
                })}
            </div>

            {lineStatuses.length === 0 && (
                <div className="flex flex-col items-center justify-center py-20 gap-4 opacity-60">
                    <FontAwesomeIcon icon={faExclamationTriangle} className="h-12 w-12" style={{ color: "var(--color-warning)" }} />
                    <p className="text-lg font-medium" style={{ color: "var(--text-primary)" }}>Aucune ligne active trouvée.</p>
                    <p className="text-sm" style={{ color: "var(--text-secondary)" }}>Configurez vos lignes de production dans la section dédiée.</p>
                </div>
            )}
        </div>
    );
}
