"use client";

import { useEffect, useState } from "react";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faIndustry, faClock, faExclamationTriangle } from "@fortawesome/free-solid-svg-icons";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardHeader, CardBody, Divider, Chip, Progress } from "@heroui/react";

interface LineStatus {
    id: number;
    name: string;
    active: boolean;
    pending_files: number;
    last_processed?: string;
    total_processed: number;
    status: "MARCHE" | "ALERTE" | "ARRET" | "ERREUR";
}

const statusColorMap: Record<string, "success" | "warning" | "default" | "danger"> = {
    MARCHE: "success",
    ALERTE: "warning",
    ARRET: "default",
    ERREUR: "danger",
};

const statusClassMap: Record<string, string> = {
    MARCHE: "status-chip status-chip-green",
    ALERTE: "status-chip status-chip-orange",
    ARRET: "status-chip status-chip-red",
    ERREUR: "status-chip status-chip-red",
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
        const onFocus = () => fetchStatus();
        const onVisibility = () => {
            if (document.visibilityState === "visible") {
                fetchStatus();
            }
        };

        window.addEventListener("focus", onFocus);
        document.addEventListener("visibilitychange", onVisibility);

        return () => {
            clearInterval(interval);
            window.removeEventListener("focus", onFocus);
            document.removeEventListener("visibilitychange", onVisibility);
        };
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

            <div className="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-4 auto-rows-fr">
                {lineStatuses.map((line) => {
                    const statusColor = statusColorMap[line.status] || "default";
                    
                    return (
                        <Card 
                            key={line.id} 
                            className="bg-(--bg-secondary) border border-(--border-default) shadow-sm h-full"
                            radius="lg"
                        >
                            <CardHeader className="flex justify-between items-start p-5">
                                <div className="flex gap-3 items-center">
                                    <div 
                                        className="p-2 rounded-lg"
                                        style={{ 
                                            background: `var(--status-${line.status.toLowerCase()}-bg)`, 
                                            color: `var(--status-${line.status.toLowerCase()})` 
                                        }}
                                    >
                                        <FontAwesomeIcon icon={faIndustry} className="h-5 w-5" />
                                    </div>
                                    <div>
                                        <p className="text-md font-bold text-(--text-primary)">{line.name}</p>
                                        <p className="text-xs text-(--text-tertiary)">ID: {line.id}</p>
                                    </div>
                                </div>
                                <Chip 
                                    color={statusColor} 
                                    variant="flat" 
                                    size="sm"
                                    className={`font-medium ${statusClassMap[line.status] ?? ""}`}
                                >
                                    {line.status}
                                </Chip>
                            </CardHeader>
                            
                            <Divider className="opacity-50" />
                            
                            <CardBody className="p-5 flex flex-col gap-4">
                                <div className="flex flex-col gap-2">
                                    <div className="flex justify-between items-center text-xs">
                                        <span className="text-(--text-tertiary)">Dernière activité</span>
                                        <span className="text-(--text-secondary)">
                                            {line.last_processed || "Jamais"}
                                        </span>
                                    </div>
                                    <Progress 
                                        aria-label="Status progress"
                                        size="sm"
                                        value={getProgressValue(line.status)}
                                        color={statusColor}
                                        className="max-w-md"
                                        classNames={{
                                            track: "bg-[var(--bg-tertiary)]",
                                        }}
                                    />
                                </div>

                                <div className="grid grid-cols-2 gap-4">
                                    <div className="p-3 rounded-lg flex flex-col items-center bg-(--bg-tertiary)">
                                        <p className="text-[10px] uppercase font-bold text-(--text-tertiary)">Traités</p>
                                        <p className="text-lg font-bold text-(--color-success)">{line.total_processed}</p>
                                    </div>
                                    <div className="p-3 rounded-lg flex flex-col items-center bg-(--bg-tertiary)">
                                        <p className="text-[10px] uppercase font-bold text-(--text-tertiary)">En attente</p>
                                        <p className="text-lg font-bold text-(--color-warning)">{line.pending_files}</p>
                                    </div>
                                </div>

                                <div className="flex gap-2 items-center text-[10px] text-(--text-tertiary)">
                                    <FontAwesomeIcon icon={faClock} />
                                    <span>Actualisé toutes les 5 secondes</span>
                                </div>
                            </CardBody>
                        </Card>
                    );
                })}
            </div>

            {lineStatuses.length === 0 && (
                <div className="flex flex-col items-center justify-center py-20 gap-4 opacity-60">
                    <FontAwesomeIcon icon={faExclamationTriangle} className="h-12 w-12" style={{ color: "var(--color-warning)" }} />
                    <p className="text-lg font-medium text-(--text-primary)">Aucune ligne active trouvée.</p>
                    <p className="text-sm text-(--text-secondary)">Configurez vos lignes de production dans la section dédiée.</p>
                </div>
            )}
        </div>
    );
}
