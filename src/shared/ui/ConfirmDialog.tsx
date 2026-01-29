"use client";

import {
  Modal,
  ModalContent,
  ModalHeader,
  ModalBody,
  ModalFooter,
  Button,
} from "@heroui/react";

interface ConfirmDialogProps {
  isOpen: boolean;
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  isDanger?: boolean;
  isLoading?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmDialog({
  isOpen,
  title,
  message,
  confirmText = "Confirmer",
  cancelText = "Annuler",
  isDanger = false,
  isLoading = false,
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  return (
    <Modal
      isOpen={isOpen}
      onClose={onCancel}
      backdrop="blur"
      placement="center"
      size="md"
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
                {title}
              </h2>
            </ModalHeader>
            <ModalBody>
              <p className="text-sm" style={{ color: "var(--text-secondary)" }}>
                {message}
              </p>
            </ModalBody>
            <ModalFooter>
              <Button
                variant="light"
                onPress={onCancel}
                isDisabled={isLoading}
                className="text-[var(--text-primary)] hover:bg-[var(--bg-hover)]"
              >
                {cancelText}
              </Button>
              <Button
                color={isDanger ? "danger" : "primary"}
                onPress={onConfirm}
                isLoading={isLoading}
                className={
                  isDanger
                    ? "bg-[var(--color-error)] text-white"
                    : "bg-[var(--button-primary-bg)] text-white hover:bg-[var(--button-primary-hover)]"
                }
              >
                {confirmText}
              </Button>
            </ModalFooter>
          </>
        )}
      </ModalContent>
    </Modal>
  );
}
