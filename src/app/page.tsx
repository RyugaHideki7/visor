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

const statusColors = {
    MARCHE: { bg: "bg-green-500/20", text: "text-green-400", bar: "bg-green-500" },
    ALERTE: { bg: "bg-yellow-500/20", text: "text-yellow-400", bar: "bg-yellow-500" },
    ARRET: { bg: "bg-gray-500/20", text: "text-gray-400", bar: "bg-gray-500" },
    ERREUR: { bg: "bg-red-500/20", text: "text-red-400", bar: "bg-red-500" },
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
                <h1 className="text-2xl font-bold text-white">Tableau de bord</h1>
                <p className="text-gray-400 text-sm">Statut en temps réel des lignes de production</p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {lineStatuses.map((line) => {
                    const colors = statusColors[line.status] || statusColors.ARRET;
                    return (
                        <div key={line.id} className="bg-[#202020] rounded-xl border border-[#333] overflow-hidden">
                            <div className="flex justify-between items-start p-5">
                                <div className="flex gap-3 items-center">
                                    <div className={`p-2 rounded-lg ${colors.bg} ${colors.text}`}>
                                        <FontAwesomeIcon icon={faIndustry} className="h-5 w-5" />
                                    </div>
                                    <div>
                                        <p className="text-md font-bold text-white">{line.name}</p>
                                        <p className="text-xs text-gray-500">ID: {line.id}</p>
                                    </div>
                                </div>
                                <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${colors.bg} ${colors.text}`}>
                                    {line.status}
                                </span>
                            </div>
                            
                            <div className="mx-5 h-px bg-[#333]" />
                            
                            <div className="p-5 flex flex-col gap-4">
                                <div className="flex flex-col gap-2">
                                    <div className="flex justify-between items-center text-xs">
                                        <span className="text-gray-500">Dernière activité</span>
                                        <span className="text-gray-300">
                                            {line.last_processed || "Jamais"}
                                        </span>
                                    </div>
                                    <div className="w-full h-2 bg-[#333] rounded-full overflow-hidden">
                                        <div 
                                            className={`h-full ${colors.bar} transition-all duration-300`}
                                            style={{ width: `${getProgressValue(line.status)}%` }}
                                        />
                                    </div>
                                </div>

                                <div className="grid grid-cols-2 gap-4">
                                    <div className="bg-black/30 p-3 rounded-lg flex flex-col items-center">
                                        <p className="text-[10px] text-gray-500 uppercase font-bold">Traités</p>
                                        <p className="text-lg font-bold text-green-400">{line.total_processed}</p>
                                    </div>
                                    <div className="bg-black/30 p-3 rounded-lg flex flex-col items-center">
                                        <p className="text-[10px] text-gray-500 uppercase font-bold">En attente</p>
                                        <p className="text-lg font-bold text-yellow-400">{line.pending_files}</p>
                                    </div>
                                </div>

                                <div className="flex gap-2 items-center text-[10px] text-gray-500">
                                    <FontAwesomeIcon icon={faClock} />
                                    <span>Actualisé toutes les 5 secondes</span>
                                </div>
                            </div>
                        </div>
                    );
                })}
            </div>

            {lineStatuses.length === 0 && (
                <div className="flex flex-col items-center justify-center py-20 gap-4 opacity-50">
                    <FontAwesomeIcon icon={faExclamationTriangle} className="h-12 w-12 text-yellow-500" />
                    <p className="text-lg font-medium text-white">Aucune ligne active trouvée.</p>
                    <p className="text-sm text-gray-400">Configurez vos lignes de production dans la section dédiée.</p>
                </div>
            )}
        </div>
    );
}
