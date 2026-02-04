"use client";

import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faSave, faRotateRight, faDownload, faFolderOpen } from "@fortawesome/free-solid-svg-icons";
import { Button, Tabs, Tab, Card, CardBody, CardHeader, Divider, Input, Textarea } from "@heroui/react";
import { save } from "@tauri-apps/plugin-dialog";

type ExportDatResult = {
  output_path: string;
  rows: number;
};

export default function LogitronProduitPage() {
  const [outputPath, setOutputPath] = useState<string>("");
  const [isRunning, setIsRunning] = useState(false);
  const [result, setResult] = useState<ExportDatResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<"export" | "sql">("export");
  const [sqlQuery, setSqlQuery] = useState<string>("");
  const [isSavingQuery, setIsSavingQuery] = useState(false);
  const [saveStatus, setSaveStatus] = useState<string | null>(null);
  const [exportStatus, setExportStatus] = useState<string | null>(null);

  useEffect(() => {
    const loadQuery = async () => {
      try {
        const q = await invoke<string>("get_sql_query", { formatName: "LOGITRON_PRODUIT" });
        setSqlQuery(q);
      } catch (e) {
        console.error("Failed to load SQL query", e);
      }
    };
    loadQuery();
  }, []);

  const handleExport = async () => {
    setIsRunning(true);
    setError(null);
    setResult(null);
    setExportStatus(null);

    const pathToUse = outputPath.trim().length > 0 ? outputPath : "produit.dat";

    try {
      const res = await invoke<ExportDatResult>("export_logitron_produit_dat", {
        outputPath: pathToUse,
      });
      setResult(res);
      setExportStatus(`Export OK: ${res.rows} lignes → ${res.output_path}`);
    } catch (e) {
      setError(String(e));
      setExportStatus(null);
    } finally {
      setIsRunning(false);
    }
  };

  const handleSaveQuery = async () => {
    setIsSavingQuery(true);
    setError(null);
    setSaveStatus(null);
    try {
      await invoke("save_sql_query", { formatName: "LOGITRON_PRODUIT", queryTemplate: sqlQuery });
      setSaveStatus("Requête sauvegardée");
    } catch (e) {
      setError(String(e));
      setSaveStatus("Erreur de sauvegarde");
    } finally {
      setIsSavingQuery(false);
    }
  };

  const handleResetQuery = async () => {
    setIsSavingQuery(true);
    setError(null);
    setSaveStatus(null);
    try {
      await invoke("reset_sql_query", { formatName: "LOGITRON_PRODUIT" });
      const q = await invoke<string>("get_sql_query", { formatName: "LOGITRON_PRODUIT" });
      setSqlQuery(q);
      setSaveStatus("Requête réinitialisée (défaut)");
    } catch (e) {
      setError(String(e));
      setSaveStatus("Erreur de réinitialisation");
    } finally {
      setIsSavingQuery(false);
    }
  };

  return (
    <div className="p-8 flex flex-col gap-6">
      <div>
        <h1 className="text-2xl font-bold" style={{ color: "var(--text-primary)" }}>
          Data exchange · Logitron · Produit
        </h1>
        <p className="text-sm" style={{ color: "var(--text-secondary)" }}>
          Génère le fichier fixed-width <code>produit.dat</code> depuis SQL Server.
        </p>
      </div>

      <Tabs
        selectedKey={activeTab}
        onSelectionChange={(key) => setActiveTab(key as "export" | "sql")}
        variant="underlined"
        classNames={{
          tabList: "gap-6 w-full relative rounded-none p-0 border-b border-[var(--border-default)] flex justify-center",
          cursor: "w-full bg-[var(--accent-primary)]",
          tab: "max-w-fit px-0 h-12 text-[var(--text-secondary)]",
          tabContent: "group-data-[selected=true]:text-[var(--accent-primary)]",
        }}
      >
        <Tab key="export" title="Export" />
        <Tab key="sql" title="Requête SQL" />
      </Tabs>

      {activeTab === "export" && (
        <Card className="bg-(--bg-secondary) border border-(--border-default)">
          <CardHeader className="flex flex-col gap-1">
            <h2 className="font-semibold" style={{ color: "var(--text-primary)" }}>
              Export
            </h2>
            <p className="text-xs" style={{ color: "var(--text-tertiary)" }}>
              Indiquez le chemin de sortie, puis cliquez sur Exporter.
            </p>
          </CardHeader>
          <Divider />
          <CardBody className="flex flex-col gap-4">
            <Input
              label="Chemin de sortie"
              labelPlacement="outside"
              value={outputPath}
              onValueChange={setOutputPath}
              placeholder="Ex: ...\\produit.dat"
              endContent={
                <Button
                  isIconOnly
                  variant="light"
                  size="sm"
                  onPress={async () => {
                    const selected = await save({
                      defaultPath: (outputPath && outputPath.trim()) || "produit.dat",
                      filters: [{ name: "DAT", extensions: ["dat"] }],
                    });
                    if (selected) {
                      setOutputPath(selected as string);
                    }
                  }}
                  className="text-(--text-secondary)"
                >
                  <FontAwesomeIcon icon={faFolderOpen} />
                </Button>
              }
              classNames={{
                inputWrapper: "bg-[var(--bg-tertiary)] border-[var(--border-default)]",
                input: "text-[var(--text-primary)]",
                label: "text-[var(--text-secondary)]",
              }}
            />

            <Button
              color="primary"
              startContent={<FontAwesomeIcon icon={faDownload} />}
              onPress={handleExport}
              isDisabled={isRunning || !outputPath.trim()}
              isLoading={isRunning}
              className="bg-(--button-primary-bg) text-white"
            >
              {isRunning ? "Export en cours..." : "Exporter"}
            </Button>

            {exportStatus && (
              <div className="px-4 py-2 rounded-lg text-sm bg-(--color-success-bg) text-(--color-success)">
                {exportStatus}
              </div>
            )}

            {error && (
              <div className="px-4 py-2 rounded-lg text-sm bg-(--color-error-bg) text-(--color-error)">
                Erreur: {error}
              </div>
            )}
          </CardBody>
        </Card>
      )}

      {activeTab === "sql" && (
        <Card className="bg-(--bg-secondary) border border-(--border-default)">
          <CardHeader className="flex justify-between items-start gap-4">
            <div className="flex flex-col gap-1">
              <h2 className="font-semibold" style={{ color: "var(--text-primary)" }}>
                Requête SQL (Logitron · Produit)
              </h2>
              <p className="text-xs" style={{ color: "var(--text-tertiary)" }}>
                Modifiez puis sauvegardez. L’export utilisera cette requête.
              </p>
            </div>
            <div className="flex gap-2">
              <Button
                variant="bordered"
                startContent={<FontAwesomeIcon icon={faRotateRight} />}
                onPress={handleResetQuery}
                isDisabled={isSavingQuery}
                className="border-(--border-default) text-(--text-primary)"
              >
                Réinitialiser
              </Button>
              <Button
                color="primary"
                startContent={<FontAwesomeIcon icon={faSave} />}
                onPress={handleSaveQuery}
                isLoading={isSavingQuery}
                className="bg-(--button-primary-bg) text-white"
              >
                Sauvegarder
              </Button>
            </div>
          </CardHeader>
          <Divider />
          <CardBody className="flex flex-col gap-4">
            <Textarea
              value={sqlQuery}
              onValueChange={setSqlQuery}
              minRows={15}
              maxRows={30}
              variant="bordered"
              classNames={{
                inputWrapper: "bg-[var(--bg-tertiary)] border-[var(--border-default)] hover:border-[var(--accent-primary)] focus-within:border-[var(--accent-primary)]",
                input: "text-[var(--text-primary)] font-mono text-sm",
              }}
            />

            {saveStatus && (
              <div
                className={`px-4 py-2 rounded-lg text-sm ${
                  saveStatus.toLowerCase().includes("erreur")
                    ? "bg-(--color-error-bg) text-(--color-error)"
                    : "bg-(--color-success-bg) text-(--color-success)"
                }`}
              >
                {saveStatus}
              </div>
            )}

            {error && (
              <div className="px-4 py-2 rounded-lg text-sm bg-(--color-error-bg) text-(--color-error)">
                Erreur: {error}
              </div>
            )}
          </CardBody>
        </Card>
      )}
    </div>
  );
}
