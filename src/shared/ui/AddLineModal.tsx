"use client";

import { useState, useEffect } from "react";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faFolderOpen } from "@fortawesome/free-solid-svg-icons";
import { 
  Modal, 
  ModalContent, 
  ModalHeader, 
  ModalBody, 
  ModalFooter, 
  Button, 
  Input, 
  Select, 
  SelectItem,
  Divider
} from "@heroui/react";

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

  return (
    <Modal 
      isOpen={isOpen} 
      onClose={handleClose}
      size="2xl"
      backdrop="blur"
      placement="center"
      scrollBehavior="inside"
      classNames={{
        base: "bg-[var(--bg-secondary)] border border-[var(--border-default)]",
        header: "border-b border-[var(--border-default)]",
        footer: "border-t border-[var(--border-default)]",
        closeButton: "hover:bg-[var(--bg-hover)] active:bg-[var(--bg-active)]",
      }}
    >
      <ModalContent>
        {() => (
          <>
            <ModalHeader className="flex flex-col gap-1">
              <h2 className="text-lg font-semibold" style={{ color: "var(--text-primary)" }}>
                {initialData?.id ? "Modifier la ligne" : "Ajouter une ligne de production"}
              </h2>
            </ModalHeader>
            <ModalBody className="py-6 flex flex-col gap-6">
              <Input
                label="Nom de la ligne"
                labelPlacement="outside"
                placeholder="ex: Ligne A"
                value={formData.name}
                onValueChange={(val) => setFormData({ ...formData, name: val })}
                variant="bordered"
                classNames={{
                  label: "text-[var(--text-secondary)]",
                  inputWrapper: "bg-[var(--bg-tertiary)] border-[var(--border-default)] hover:border-[var(--accent-primary)] focus-within:border-[var(--accent-primary)]",
                  input: "text-[var(--text-primary)] placeholder:text-[var(--text-placeholder)]",
                }}
              />

              <Input
                label="Chemin de surveillance"
                labelPlacement="outside"
                placeholder="C:/Production/LigneA"
                value={formData.path}
                onValueChange={(val) => setFormData({ ...formData, path: val })}
                variant="bordered"
                endContent={
                  <Button isIconOnly size="sm" variant="light" className="text-[var(--text-tertiary)]">
                    <FontAwesomeIcon icon={faFolderOpen} />
                  </Button>
                }
                classNames={{
                  label: "text-[var(--text-secondary)]",
                  inputWrapper: "bg-[var(--bg-tertiary)] border-[var(--border-default)] hover:border-[var(--accent-primary)] focus-within:border-[var(--accent-primary)]",
                  input: "text-[var(--text-primary)] placeholder:text-[var(--text-placeholder)]",
                }}
              />

              <div className="flex gap-4">
                <Input
                  label="Préfixe des fichiers"
                  labelPlacement="outside"
                  placeholder="STOCK"
                  value={formData.prefix}
                  onValueChange={(val) => setFormData({ ...formData, prefix: val })}
                  variant="bordered"
                  className="flex-1"
                  classNames={{
                    label: "text-[var(--text-secondary)]",
                    inputWrapper: "bg-[var(--bg-tertiary)] border-[var(--border-default)] hover:border-[var(--accent-primary)] focus-within:border-[var(--accent-primary)]",
                    input: "text-[var(--text-primary)] placeholder:text-[var(--text-placeholder)]",
                  }}
                />
                <Input
                  label="Dossier d'archivage"
                  labelPlacement="outside"
                  placeholder="C:/Archive/LigneA"
                  value={formData.archived_path ?? ""}
                  onValueChange={(val) => setFormData({ ...formData, archived_path: val })}
                  variant="bordered"
                  className="flex-1"
                  classNames={{
                    label: "text-[var(--text-secondary)]",
                    inputWrapper: "bg-[var(--bg-tertiary)] border-[var(--border-default)] hover:border-[var(--accent-primary)] focus-within:border-[var(--accent-primary)]",
                    input: "text-[var(--text-primary)] placeholder:text-[var(--text-placeholder)]",
                  }}
                />
              </div>

              <div className="flex gap-4">
                <Input
                  type="number"
                  label="Intervalle marche (min)"
                  labelPlacement="outside"
                  value={formData.interval_check.toString()}
                  onValueChange={(val) => setFormData({ ...formData, interval_check: parseInt(val) || 0 })}
                  variant="bordered"
                  className="flex-1"
                  classNames={{
                    label: "text-[var(--text-secondary)]",
                    inputWrapper: "bg-[var(--bg-tertiary)] border-[var(--border-default)] hover:border-[var(--accent-primary)] focus-within:border-[var(--accent-primary)]",
                    input: "text-[var(--text-primary)]",
                  }}
                />
                <Input
                  type="number"
                  label="Intervalle arrêt (min)"
                  labelPlacement="outside"
                  value={formData.interval_alert.toString()}
                  onValueChange={(val) => setFormData({ ...formData, interval_alert: parseInt(val) || 0 })}
                  variant="bordered"
                  className="flex-1"
                  classNames={{
                    label: "text-[var(--text-secondary)]",
                    inputWrapper: "bg-[var(--bg-tertiary)] border-[var(--border-default)] hover:border-[var(--accent-primary)] focus-within:border-[var(--accent-primary)]",
                    input: "text-[var(--text-primary)]",
                  }}
                />
              </div>

              <div className="flex flex-col gap-4">
                <div className="flex items-center gap-2">
                  <Divider className="flex-1" />
                  <p className="text-xs font-medium uppercase text-[var(--text-tertiary)]">Paramètres SQL</p>
                  <Divider className="flex-1" />
                </div>

                <div className="flex gap-4">
                  <Input
                    label="Site"
                    labelPlacement="outside"
                    placeholder="ex: SITE01"
                    value={formData.site ?? ""}
                    onValueChange={(val) => setFormData({ ...formData, site: val })}
                    variant="bordered"
                    className="flex-1"
                    classNames={{
                      label: "text-[var(--text-secondary)]",
                      inputWrapper: "bg-[var(--bg-tertiary)] border-[var(--border-default)] hover:border-[var(--accent-primary)] focus-within:border-[var(--accent-primary)]",
                      input: "text-[var(--text-primary)]",
                    }}
                  />
                  <Input
                    label="Unité"
                    labelPlacement="outside"
                    placeholder="ex: UNITE01"
                    value={formData.unite ?? ""}
                    onValueChange={(val) => setFormData({ ...formData, unite: val })}
                    variant="bordered"
                    className="flex-1"
                    classNames={{
                      label: "text-[var(--text-secondary)]",
                      inputWrapper: "bg-[var(--bg-tertiary)] border-[var(--border-default)] hover:border-[var(--accent-primary)] focus-within:border-[var(--accent-primary)]",
                      input: "text-[var(--text-primary)]",
                    }}
                  />
                </div>

                <div className="flex gap-4">
                  <Input
                    label="Flag Déc"
                    labelPlacement="outside"
                    placeholder="ex: 1"
                    value={formData.flag_dec ?? ""}
                    onValueChange={(val) => setFormData({ ...formData, flag_dec: val })}
                    variant="bordered"
                    className="flex-1"
                    classNames={{
                      label: "text-[var(--text-secondary)]",
                      inputWrapper: "bg-[var(--bg-tertiary)] border-[var(--border-default)] hover:border-[var(--accent-primary)] focus-within:border-[var(--accent-primary)]",
                      input: "text-[var(--text-primary)]",
                    }}
                  />
                  <Input
                    label="Code Ligne"
                    labelPlacement="outside"
                    placeholder="ex: L01"
                    value={formData.code_ligne ?? ""}
                    onValueChange={(val) => setFormData({ ...formData, code_ligne: val })}
                    variant="bordered"
                    className="flex-1"
                    classNames={{
                      label: "text-[var(--text-secondary)]",
                      inputWrapper: "bg-[var(--bg-tertiary)] border-[var(--border-default)] hover:border-[var(--accent-primary)] focus-within:border-[var(--accent-primary)]",
                      input: "text-[var(--text-primary)]",
                    }}
                  />
                </div>

                <div className="flex gap-4">
                  <Input
                    label="Dossier logs"
                    labelPlacement="outside"
                    placeholder="C:/Logs/LigneA"
                    value={formData.log_path ?? ""}
                    onValueChange={(val) => setFormData({ ...formData, log_path: val })}
                    variant="bordered"
                    className="flex-1"
                    classNames={{
                      label: "text-[var(--text-secondary)]",
                      inputWrapper: "bg-[var(--bg-tertiary)] border-[var(--border-default)] hover:border-[var(--accent-primary)] focus-within:border-[var(--accent-primary)]",
                      input: "text-[var(--text-primary)]",
                    }}
                  />
                  <Select
                    label="Format fichier"
                    labelPlacement="outside"
                    selectedKeys={[formData.file_format ?? "ATEIS"]}
                    onSelectionChange={(keys) => {
                      const selected = Array.from(keys)[0] as string;
                      setFormData({ ...formData, file_format: selected });
                    }}
                    variant="bordered"
                    className="flex-1"
                    classNames={{
                      label: "text-[var(--text-secondary)]",
                      trigger: "bg-[var(--bg-tertiary)] border-[var(--border-default)] hover:border-[var(--accent-primary)] data-[open=true]:border-[var(--accent-primary)]",
                      value: "text-[var(--text-primary)]",
                      popoverContent: "bg-[var(--bg-secondary)] border border-[var(--border-default)]",
                    }}
                  >
                    <SelectItem key="ATEIS" textValue="ATEIS" className="text-[var(--text-primary)] hover:bg-[var(--bg-hover)]">ATEIS</SelectItem>
                    <SelectItem key="LOGITRON" textValue="LOGITRON" className="text-[var(--text-primary)] hover:bg-[var(--bg-hover)]">LOGITRON</SelectItem>
                  </Select>
                </div>
              </div>
            </ModalBody>
            <ModalFooter>
              <Button 
                variant="light" 
                onPress={handleClose}
                className="text-[var(--text-primary)] hover:bg-[var(--bg-hover)]"
              >
                Annuler
              </Button>
              <Button 
                color="primary" 
                onPress={handleSave}
                isLoading={isSaving}
                className="bg-[var(--button-primary-bg)] text-white hover:bg-[var(--button-primary-hover)]"
              >
                Sauvegarder
              </Button>
            </ModalFooter>
          </>
        )}
      </ModalContent>
    </Modal>
  );
}
