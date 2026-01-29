"use client";

import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faPlus, faTrash, faPlay, faPause } from "@fortawesome/free-solid-svg-icons";
import { AddLineModal, type LineFormData } from "@/shared/ui/AddLineModal";
import { 
  Table, 
  TableHeader, 
  TableColumn, 
  TableBody, 
  TableRow, 
  TableCell, 
  Button, 
  Tooltip,
  Chip
} from "@heroui/react";

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
                <div>
                    <h1 className="text-2xl font-bold text-[var(--text-primary)]">Lignes de production</h1>
                    <p className="text-sm text-[var(--text-secondary)]">Gérez vos lignes de surveillance et paramètres de traitement</p>
                </div>
                <Button
                    onPress={() => setIsModalOpen(true)}
                    color="primary"
                    startContent={<FontAwesomeIcon icon={faPlus} />}
                    className="bg-[var(--button-primary-bg)] text-white hover:bg-[var(--button-primary-hover)]"
                >
                    Ajouter une ligne
                </Button>
            </div>

            <Table 
                aria-label="Table des lignes de production"
                classNames={{
                    base: "border border-[var(--border-default)] rounded-xl overflow-hidden shadow-sm",
                    table: "bg-[var(--bg-secondary)]",
                    thead: "[&>tr]:first:rounded-none bg-[var(--bg-tertiary)]",
                    th: "bg-[var(--bg-tertiary)] text-[var(--text-tertiary)] font-bold text-xs uppercase py-4 px-6 border-b border-[var(--border-default)]",
                    td: "py-4 px-6 text-sm text-[var(--text-primary)] border-b border-[var(--border-default)/50]",
                    tr: "hover:bg-[var(--bg-hover)] transition-colors",
                }}
            >
                <TableHeader>
                    <TableColumn>NOM</TableColumn>
                    <TableColumn>CHEMIN</TableColumn>
                    <TableColumn>PRÉFIXE</TableColumn>
                    <TableColumn>STATUT</TableColumn>
                    <TableColumn align="center">ACTIONS</TableColumn>
                </TableHeader>
                <TableBody emptyContent={"Aucune ligne configurée."}>
                    {lines.map((line) => (
                        <TableRow key={line.id}>
                            <TableCell className="font-semibold">{line.name}</TableCell>
                            <TableCell className="text-xs text-[var(--text-tertiary)]">{line.path}</TableCell>
                            <TableCell className="text-[var(--text-secondary)]">{line.prefix}</TableCell>
                            <TableCell>
                                <Chip 
                                    color={line.active ? "success" : "default"} 
                                    variant="flat" 
                                    size="sm"
                                    className="font-medium"
                                >
                                    {line.active ? "Active" : "Inactif"}
                                </Chip>
                            </TableCell>
                            <TableCell>
                                <div className="flex justify-center gap-2">
                                    <Tooltip content={line.active ? "Mettre en pause" : "Démarrer"}>
                                        <Button
                                            isIconOnly
                                            size="sm"
                                            variant="light"
                                            onPress={() => handleToggleActive(line)}
                                            className="text-[var(--text-tertiary)] hover:bg-[var(--bg-active)] hover:text-[var(--text-primary)]"
                                        >
                                            <FontAwesomeIcon icon={line.active ? faPause : faPlay} />
                                        </Button>
                                    </Tooltip>
                                    <Tooltip content="Supprimer" color="danger">
                                        <Button
                                            isIconOnly
                                            size="sm"
                                            variant="light"
                                            onPress={() => line.id && handleDelete(line.id)}
                                            className="text-[var(--text-tertiary)] hover:bg-[var(--color-error-bg)] hover:text-[var(--color-error)]"
                                        >
                                            <FontAwesomeIcon icon={faTrash} />
                                        </Button>
                                    </Tooltip>
                                </div>
                            </TableCell>
                        </TableRow>
                    ))}
                </TableBody>
            </Table>

            <AddLineModal
                isOpen={isModalOpen}
                onClose={() => setIsModalOpen(false)}
                onSave={handleSave}
            />
        </div>
    );
}
