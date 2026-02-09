"use client";



import { useEffect, useState } from "react";

import { invoke } from "@tauri-apps/api/core";

import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";

import { faPlus, faTrash, faPlay, faPause } from "@fortawesome/free-solid-svg-icons";

import { AddLineModal, type LineFormData } from "@/shared/ui/AddLineModal";

import { ConfirmDialog } from "@/shared/ui/ConfirmDialog";

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
    const [editingLine, setEditingLine] = useState<Line | null>(null);

    const [deleteTarget, setDeleteTarget] = useState<Line | null>(null);

    const [isDeleting, setIsDeleting] = useState(false);



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

        await invoke<number>("save_line", { line });

        setEditingLine(null);
        fetchLines();

    };



    const handleDeleteConfirmed = async () => {

        if (!deleteTarget?.id) return;

        setIsDeleting(true);

        try {

            await invoke("delete_line", { id: deleteTarget.id });

            setDeleteTarget(null);

            fetchLines();

        } catch (error) {

            console.error("Failed to delete line:", error);

        } finally {

            setIsDeleting(false);

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
                    <h1 className="text-2xl font-bold text-(--text-primary)">Lignes de production</h1>
                    <p className="text-sm text-(--text-secondary)">Gérez vos lignes de surveillance et paramètres de traitement</p>
                </div>
                <Button
                    onPress={() => {
                        setEditingLine(null);
                        setIsModalOpen(true);
                    }}
                    color="primary"
                    startContent={<FontAwesomeIcon icon={faPlus} />}
                    className="bg-(--button-primary-bg) text-white hover:bg-(--button-primary-hover)"
                >
                    Ajouter une ligne
                </Button>
            </div>

            <Table 
                    aria-label="Table des lignes de production"
                    removeWrapper
                    classNames={{
                        base: "border border-(--border-default) rounded-xl shadow-sm bg-(--bg-secondary)",
                        table: "rounded-xl",
                        thead: "bg-(--bg-tertiary) [&>tr]:first:rounded-t-xl after:content-none after:hidden after:h-0 after:w-0",
                        th: "bg-(--bg-tertiary) text-(--text-tertiary) font-bold text-xs uppercase py-4 px-6 first:rounded-tl-xl last:rounded-tr-xl border-b border-(--border-default)",
                        td: "py-4 px-6 text-sm text-(--text-primary)",
                        tr: "hover:bg-(--bg-hover) transition-colors border-b border-(--border-default)/50 last:border-b-0",
                        tbody: "[&>tr:last-child>td]:first:rounded-bl-xl [&>tr:last-child>td]:last:rounded-br-xl",
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
                    {lines.map((line: Line) => (
                        <TableRow key={line.id}>
                            <TableCell className="font-semibold">{line.name}</TableCell>
                            <TableCell className="text-xs text-(--text-tertiary)">{line.path}</TableCell>
                            <TableCell className="text-(--text-secondary)">{line.prefix}</TableCell>
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
                                    <Tooltip content="Modifier">
                                        <Button
                                            isIconOnly
                                            size="sm"
                                            variant="light"
                                            onPress={() => {
                                                setEditingLine(line);
                                                setIsModalOpen(true);
                                            }}
                                            className="text-(--text-tertiary) hover:bg-(--bg-hover) hover:text-(--text-primary)"
                                        >
                                            ✎
                                        </Button>
                                    </Tooltip>
                                    <Tooltip content={line.active ? "Mettre en pause" : "Démarrer"}>
                                        <Button
                                            isIconOnly
                                            size="sm"
                                            variant="light"
                                            onPress={() => handleToggleActive(line)}
                                            className="text-(--text-tertiary) hover:bg-(--bg-active) hover:text-(--text-primary)"
                                        >
                                            <FontAwesomeIcon icon={line.active ? faPause : faPlay} />
                                        </Button>
                                    </Tooltip>
                                    <Tooltip content="Supprimer" color="danger">
                                        <Button
                                            isIconOnly
                                            size="sm"
                                            variant="light"
                                            onPress={() => setDeleteTarget(line as Line)}
                                            className="text-(--text-tertiary) hover:bg-(--color-error-bg) hover:text-(--color-error)"
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
                initialData={editingLine ?? undefined}
            />

            <ConfirmDialog
                isOpen={deleteTarget !== null}
                title="Supprimer la ligne"
                message={
                    deleteTarget
                        ? `Voulez-vous vraiment supprimer la ligne "${deleteTarget.name}" ? Cette action est irréversible.`
                        : "Voulez-vous vraiment supprimer cette ligne ?"
                }
                confirmText="Supprimer"
                cancelText="Annuler"
                variant="danger"
                isLoading={isDeleting}
                onClose={() => (isDeleting ? null : setDeleteTarget(null))}
                onConfirm={handleDeleteConfirmed}
            />
        </div>
    );
}
