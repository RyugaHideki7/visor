"use client";

import { useEffect, useMemo, useState } from "react";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faExclamationTriangle } from "@fortawesome/free-solid-svg-icons";
import { invoke } from "@tauri-apps/api/core";
import { Table, TableHeader, TableColumn, TableBody, TableRow, TableCell, Chip, Pagination, Tabs, Tab } from "@heroui/react";

interface LineStatus {
    id: number;
    name: string;
    active: boolean;
    pending_files: number;
    last_processed?: string;
    total_processed: number;
    status: "MARCHE" | "ALERTE" | "ARRET" | "ERREUR";
    site?: string | null;
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
    const [page, setPage] = useState(1);
    const rowsPerPage = 10;
    const [siteFilter, setSiteFilter] = useState<string>("ALL");

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

    const sites = useMemo(() => {
        const unique = Array.from(
            new Set(
                lineStatuses
                    .map((l) => l.site?.trim())
                    .filter((s): s is string => Boolean(s && s.length > 0))
            )
        );
        unique.sort((a, b) => a.localeCompare(b));
        return ["ALL", ...unique];
    }, [lineStatuses]);

    const filtered = useMemo(() => {
        if (siteFilter === "ALL") return lineStatuses;
        return lineStatuses.filter((l) => (l.site || "").trim() === siteFilter);
    }, [lineStatuses, siteFilter]);

    useEffect(() => {
        setPage(1);
    }, [siteFilter]);

    const totalPages = Math.max(1, Math.ceil(filtered.length / rowsPerPage));
    const paginatedItems = useMemo(() => {
        const start = (page - 1) * rowsPerPage;
        return filtered.slice(start, start + rowsPerPage);
    }, [filtered, page]);

    return (
        <div className="p-8 flex flex-col gap-8">
            <div>
                <h1 className="text-2xl font-bold" style={{ color: "var(--text-primary)" }}>Tableau de bord</h1>
                <p className="text-sm" style={{ color: "var(--text-secondary)" }}>Statut en temps réel des lignes de production</p>
            </div>

            <Tabs
                aria-label="Filtrer par site"
                selectedKey={siteFilter}
                onSelectionChange={(k) => setSiteFilter(String(k))}
                variant="underlined"
                color="primary"
            >
                {sites.map((site) => (
                    <Tab key={site} title={site === "ALL" ? "Tous les sites" : site} />
                ))}
            </Tabs>

            <div className="bg-(--bg-secondary) border border-(--border-default) rounded-xl p-4">
                <Table
                    aria-label="Statut des lignes de production"
                    removeWrapper
                    bottomContent={
                        <div className="flex justify-center">
                            <Pagination
                                page={page}
                                total={totalPages}
                                isCompact
                                showControls
                                size="sm"
                                onChange={setPage}
                            />
                        </div>
                    }
                    classNames={{
                        th: "text-(--text-secondary) text-center",
                        td: "text-(--text-primary) text-center",
                    }}
                >
                    <TableHeader>
                        <TableColumn>Ligne</TableColumn>
                        <TableColumn>ID</TableColumn>
                        <TableColumn>Statut</TableColumn>
                        <TableColumn>Dernière activité</TableColumn>
                        <TableColumn>Traités</TableColumn>
                        <TableColumn>En attente</TableColumn>
                    </TableHeader>
                    <TableBody emptyContent="Aucune ligne trouvée" items={paginatedItems}>
                        {(line) => {
                            const statusColor = statusColorMap[line.status] || "default";
                            return (
                                <TableRow key={line.id}>
                                    <TableCell className="font-semibold text-(--text-primary)">{line.name}</TableCell>
                                    <TableCell className="text-(--text-tertiary)">{line.id}</TableCell>
                                    <TableCell className="text-center">
                                        <Chip 
                                            color={statusColor} 
                                            variant="flat" 
                                            size="sm"
                                            className={`font-medium ${statusClassMap[line.status] ?? ""}`}
                                        >
                                            {line.status}
                                        </Chip>
                                    </TableCell>
                                    <TableCell>{line.last_processed || "Jamais"}</TableCell>
                                    <TableCell className="text-(--color-success)">{line.total_processed}</TableCell>
                                    <TableCell className="text-(--color-warning)">{line.pending_files}</TableCell>
                                </TableRow>
                            );
                        }}
                    </TableBody>
                </Table>
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
