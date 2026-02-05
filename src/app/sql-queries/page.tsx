"use client";

import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faSave, faRotateRight } from "@fortawesome/free-solid-svg-icons";
import {
  Button,
  Tabs,
  Tab,
  Textarea,
  Card,
  CardBody,
  CardHeader,
  Divider,
} from "@heroui/react";

interface SqlQuery {
  id: number;
  format_name: string;
  query_template: string;
}

export default function SqlQueriesPage() {
  const [queries, setQueries] = useState<SqlQuery[]>([]);
  const [selectedFormat, setSelectedFormat] = useState<string>("ATEIS");
  const [currentQuery, setCurrentQuery] = useState<string>("");
  const [isSaving, setIsSaving] = useState(false);
  const [saveStatus, setSaveStatus] = useState<string | null>(null);

  const fetchQueries = async () => {
    try {
      const result = await invoke<SqlQuery[]>("get_sql_queries");
      setQueries(result);
      
      const selected = result.find(q => q.format_name === selectedFormat);
      if (selected) {
        setCurrentQuery(selected.query_template);
      }
    } catch (error) {
      console.error("Failed to fetch SQL queries:", error);
    }
  };

  useEffect(() => {
    fetchQueries();
  }, []);

  useEffect(() => {
    const selected = queries.find(q => q.format_name === selectedFormat);
    if (selected) {
      setCurrentQuery(selected.query_template);
    }
  }, [selectedFormat, queries]);

  const handleSave = async () => {
    setIsSaving(true);
    setSaveStatus(null);
    try {
      await invoke("save_sql_query", {
        formatName: selectedFormat,
        queryTemplate: currentQuery,
      });
      setSaveStatus("Requête sauvegardée avec succès");
      setTimeout(() => setSaveStatus(null), 3000);
      fetchQueries();
    } catch (error) {
      console.error("Failed to save SQL query:", error);
      setSaveStatus("Erreur lors de la sauvegarde");
    } finally {
      setIsSaving(false);
    }
  };

  const handleReset = async () => {
    try {
      await invoke("reset_sql_query", { formatName: selectedFormat });
      await fetchQueries();
      setSaveStatus("Requête réinitialisée par défaut");
      setTimeout(() => setSaveStatus(null), 3000);
    } catch (error) {
      console.error("Failed to reset query:", error);
      setSaveStatus("Erreur lors de la réinitialisation");
    }
  };

  return (
    <div className="p-8 flex flex-col gap-6 h-full">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-2xl font-bold" style={{ color: "var(--text-primary)" }}>
            Requêtes SQL
          </h1>
          <p className="text-sm" style={{ color: "var(--text-secondary)" }}>
            Configurez les requêtes INSERT pour chaque format de fichier
          </p>
        </div>
        <div className="flex gap-2">
          <Button
            variant="bordered"
            startContent={<FontAwesomeIcon icon={faRotateRight} />}
            onPress={handleReset}
            className="border-[var(--border-default)] text-[var(--text-primary)]"
          >
            Réinitialiser
          </Button>
          <Button
            color="primary"
            startContent={<FontAwesomeIcon icon={faSave} />}
            onPress={handleSave}
            isLoading={isSaving}
            className="bg-[var(--button-primary-bg)] text-white hover:bg-[var(--button-primary-hover)]"
          >
            Sauvegarder
          </Button>
        </div>
      </div>

      {saveStatus && (
        <div
          className={`px-4 py-2 rounded-lg text-sm ${
            saveStatus.includes("Erreur")
              ? "bg-[var(--color-error-bg)] text-[var(--color-error)]"
              : "bg-[var(--color-success-bg)] text-[var(--color-success)]"
          }`}
        >
          {saveStatus}
        </div>
      )}

      <Card className="flex-1 bg-[var(--bg-secondary)] border border-[var(--border-default)]">
        <CardHeader className="flex flex-col gap-2 pb-0">
          <Tabs
            selectedKey={selectedFormat}
            onSelectionChange={(key) => setSelectedFormat(key as string)}
            variant="underlined"
            classNames={{
              tabList: "gap-6 w-full relative rounded-none p-0 border-b border-[var(--border-default)]",
              cursor: "w-full bg-[var(--accent-primary)]",
              tab: "max-w-fit px-0 h-12 text-[var(--text-secondary)]",
              tabContent: "group-data-[selected=true]:text-[var(--accent-primary)]",
            }}
          >
            <Tab key="ATEIS" title="Format ATEIS" />
            <Tab key="LOGITRON" title="Format LOGITRON" />
          </Tabs>
        </CardHeader>
        <Divider />
        <CardBody className="flex-1 p-4">
          <div className="flex flex-col gap-4 h-full">
            <div className="text-sm" style={{ color: "var(--text-tertiary)" }}>
              <p>
                Utilisez les paramètres <code className="px-1 py-0.5 rounded bg-[var(--bg-tertiary)]">@P1</code>, <code className="px-1 py-0.5 rounded bg-[var(--bg-tertiary)]">@P2</code>, etc. pour les valeurs mappées.
              </p>
              <p className="mt-1">
                L&apos;ordre des paramètres correspond à l&apos;ordre des mappings définis pour chaque ligne.
              </p>
            </div>
            <Textarea
              value={currentQuery}
              onValueChange={setCurrentQuery}
              placeholder="INSERT INTO table (col1, col2, ...) VALUES (@P1, @P2, ...)"
              minRows={15}
              maxRows={30}
              variant="bordered"
              classNames={{
                base: "flex-1",
                inputWrapper: "bg-[var(--bg-tertiary)] border-[var(--border-default)] hover:border-[var(--accent-primary)] focus-within:border-[var(--accent-primary)] h-full",
                input: "text-[var(--text-primary)] font-mono text-sm",
              }}
            />
          </div>
        </CardBody>
      </Card>
    </div>
  );
}
