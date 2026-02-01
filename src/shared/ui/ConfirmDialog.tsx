"use client";

import { useState, useCallback } from "react";
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
  variant?: "default" | "danger";
  isLoading?: boolean;
  onConfirm: () => void;
  onClose: () => void;
}

export const ConfirmDialog: React.FC<ConfirmDialogProps> = ({
  isOpen,
  title,
  message,
  confirmText = "Confirmer",
  cancelText = "Annuler",
  variant = "default",
  isLoading = false,
  onConfirm,
  onClose,
}) => {
  if (!isOpen) return null;

  const isDanger = variant === "danger";

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      backdrop="blur"
      placement="center"
      size="md"
      classNames={{
        base: "bg-(--bg-secondary) border border-(--border-default)",
        header: "border-b border-(--border-default)",
        footer: "border-t border-(--border-default)",
        closeButton: "hover:bg-(--bg-hover) active:bg-(--bg-active)",
      }}
    >
      <ModalContent>
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
            onPress={onClose}
            className="text-(--text-primary) hover:bg-(--bg-hover)"
            isDisabled={isLoading}
          >
            {cancelText}
          </Button>
          <Button
            color={isDanger ? "danger" : "primary"}
            onPress={onConfirm}
            isLoading={isLoading}
            className={
              isDanger
                ? "bg-(--color-error) text-white"
                : "bg-(--button-primary-bg) text-white hover:bg-(--button-primary-hover)"
            }
          >
            {confirmText}
          </Button>
        </ModalFooter>
      </ModalContent>
    </Modal>
  );
};

type ConfirmDialogOptions = {
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  variant?: "default" | "danger";
};

export const useConfirmDialog = () => {
  const [isOpen, setIsOpen] = useState(false);
  const [options, setOptions] = useState<ConfirmDialogOptions | null>(null);
  const [resolver, setResolver] = useState<((v: boolean) => void) | null>(null);

  const open = useCallback((opts: ConfirmDialogOptions) => {
    setOptions(opts);
    setIsOpen(true);
    return new Promise<boolean>((resolve) => {
      setResolver(() => resolve);
    });
  }, []);

  const close = useCallback(() => {
    setIsOpen(false);
    setOptions(null);
    setResolver(null);
  }, []);

  const handleConfirm = useCallback(() => {
    if (resolver) resolver(true);
    close();
  }, [resolver, close]);

  const handleCancel = useCallback(() => {
    if (resolver) resolver(false);
    close();
  }, [resolver, close]);

  const props = {
    isOpen,
    title: options?.title ?? "",
    message: options?.message ?? "",
    confirmText: options?.confirmText,
    cancelText: options?.cancelText,
    variant: options?.variant,
    onConfirm: handleConfirm,
    onClose: handleCancel,
  } as ConfirmDialogProps;

  return { open, props };
};
