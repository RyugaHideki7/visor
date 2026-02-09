"use client";

import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faSave, faRotateRight, faSync, faExclamationTriangle } from "@fortawesome/free-solid-svg-icons";
import { Button, Tabs, Tab, Card, CardBody, CardHeader, Divider, Textarea, Switch, Input, Progress } from "@heroui/react";

import { useSchedulerStore } from "@/stores/schedulerStore";

type SyncResult = {
  total: number;
  inserted: number;
  updated: number;
  errors: number;
  details: string[];
};

type SyncProgress = {
  current: number;
  total: number;
  status: string;
};

export default function AteisProduitPage() {
  const [isRunning, setIsRunning] = useState(false);
  const [result, setResult] = useState<SyncResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<"sync" | "sql">("sync");
  const [sqlQuery, setSqlQuery] = useState<string>("");
  const [isSavingQuery, setIsSavingQuery] = useState(false);
  const [saveStatus, setSaveStatus] = useState<string | null>(null);
  const [progress, setProgress] = useState<SyncProgress | null>(null);

  const { tasks, startTask, stopTask, setTaskInterval, loadIntervals, syncStatus } = useSchedulerStore();
  const taskState = tasks["ATEIS_PRODUIT_SYNC"];

  useEffect(() => {
    loadIntervals();
    syncStatus();
  }, [loadIntervals, syncStatus]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    
    async function setupListener() {
        unlisten = await listen<SyncProgress>("ateis-produit-sync-progress", (event) => {
            setProgress(event.payload);
            if (event.payload.status === "Completed") {
                setTimeout(() => setProgress(null), 3000);
            }
        });
        
        const unlistenResult = await listen<SyncResult>("ateis-produit-sync-result", (event) => {
            setResult(event.payload);
        });
        
        // Combine unlisten functions
        const oldUnlisten = unlisten;
        unlisten = () => {
            oldUnlisten();
            unlistenResult();
        };
    }
    setupListener();
    
    return () => {
        if (unlisten) unlisten();
    };
  }, []);

  useEffect(() => {
    const loadQuery = async () => {
      try {
        const q = await invoke<string>("get_sql_query", { formatName: "ATEIS_PRODUIT" });
        setSqlQuery(q);
      } catch (e) {
        console.error("Failed to load SQL query", e);
      }
    };
    loadQuery();
  }, []);

  const handleAutoToggle = async (enabled: boolean) => {
    if (enabled) {
        await startTask("ATEIS_PRODUIT_SYNC", taskState.intervalMinutes);
    } else {
        await stopTask("ATEIS_PRODUIT_SYNC");
    }
  };

  const handleIntervalChange = async (v: number) => {
    const newInterval = Math.max(1, v || 1);
    setTaskInterval("ATEIS_PRODUIT_SYNC", newInterval);
    if (taskState.running) {
        await stopTask("ATEIS_PRODUIT_SYNC");
        await startTask("ATEIS_PRODUIT_SYNC", newInterval);
    }
  };

  const handleSync = async () => {
    setIsRunning(true);
    setError(null);
    setResult(null);
    setProgress(null);

    try {
      const res = await invoke<SyncResult>("sync_ateis_produit");
      setResult(res);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsRunning(false);
      setProgress(null);
    }
  };

  const handleSaveQuery = async () => {
    setIsSavingQuery(true);
    setError(null);
    setSaveStatus(null);
    try {
      await invoke("save_sql_query", { formatName: "ATEIS_PRODUIT", queryTemplate: sqlQuery });
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
      await invoke("reset_sql_query", { formatName: "ATEIS_PRODUIT" });
      const q = await invoke<string>("get_sql_query", { formatName: "ATEIS_PRODUIT" });
      setSqlQuery(q);
      setSaveStatus("Requête réinitialisée");
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
          Data exchange · Ateis · Produit
        </h1>
        <p className="text-sm" style={{ color: "var(--text-secondary)" }}>
          Synchronisation des articles SQL Server vers HFSQL (Article).
        </p>
      </div>

      <Tabs
        selectedKey={activeTab}
        onSelectionChange={(key) => setActiveTab(key as "sync" | "sql")}
        variant="underlined"
        classNames={{
          tabList: "gap-6 w-full relative rounded-none p-0 border-b border-[var(--border-default)] flex justify-center",
          cursor: "w-full bg-[var(--accent-primary)]",
          tab: "max-w-fit px-0 h-12 text-[var(--text-secondary)]",
          tabContent: "group-data-[selected=true]:text-[var(--accent-primary)]",
        }}
      >
        <Tab key="sync" title="Synchronisation" />
        <Tab key="sql" title="Requête SQL" />
      </Tabs>

      {activeTab === "sync" && (
        <Card className="bg-[var(--bg-secondary)] border border-[var(--border-default)]">
          <CardHeader className="flex flex-col gap-1">
            <h2 className="font-semibold" style={{ color: "var(--text-primary)" }}>
              Synchronisation
            </h2>
            <p className="text-xs" style={{ color: "var(--text-tertiary)" }}>
              Lancez la mise à jour des articles dans HFSQL.
            </p>
          </CardHeader>
          <Divider />
          <CardBody className="flex flex-col gap-6">
            <div className="border border-[var(--border-default)] rounded-lg p-4 bg-[var(--bg-tertiary)] flex flex-col gap-2">
              <div className="flex flex-wrap items-center gap-4 justify-between">
                <div className="flex items-center gap-3">
                  <Switch isSelected={taskState?.running || false} onValueChange={handleAutoToggle}>
                    Auto-synchro
                  </Switch>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-sm" style={{ color: "var(--text-secondary)" }}>
                    Intervalle (min)
                  </span>
                  <Input
                    type="number"
                    aria-label="Intervalle (minutes)"
                    value={taskState?.intervalMinutes?.toString() || "60"}
                    onValueChange={(v) => handleIntervalChange(Number(v))}
                    className="w-32"
                    min={1}
                  />
                </div>
              </div>
            </div>

            <Button
              color="primary"
              startContent={!isRunning ? <FontAwesomeIcon icon={faSync} /> : null}
              onPress={handleSync}
              isDisabled={isRunning || (taskState?.running || false)}
              isLoading={isRunning}
              className="bg-[var(--button-primary-bg)] text-white"
            >
              {isRunning ? "Synchronisation..." : "Lancer la synchronisation"}
            </Button>
            
            {progress && (
                <div className="flex flex-col gap-2">
                    <div className="flex justify-between text-xs text-[var(--text-secondary)]">
                        <span>{progress.status}</span>
                        <span>{Math.round((progress.current / progress.total) * 100)}%</span>
                    </div>
                    <Progress 
                        size="sm"
                        value={(progress.current / progress.total) * 100} 
                        color="primary"
                        aria-label="Progression synchronisation"
                    />
                </div>
            )}

            {result && (
               <div className="flex flex-col gap-2">
                  <div className="grid grid-cols-4 gap-2">
                     <div className="p-3 rounded-lg bg-[var(--bg-tertiary)] text-center border border-[var(--border-default)]">
                        <div className="text-xs text-[var(--text-secondary)]">Total</div>
                        <div className="text-xl font-bold text-[var(--text-primary)]">{result.total}</div>
                     </div>
                     <div className="p-3 rounded-lg bg-[var(--color-success-bg)] text-center border border-[var(--color-success)]/20">
                        <div className="text-xs text-[var(--color-success)]">Insérés</div>
                        <div className="text-xl font-bold text-[var(--color-success)]">{result.inserted}</div>
                     </div>
                     <div className="p-3 rounded-lg bg-[var(--color-info-bg)] text-center border border-[var(--color-info)]/20">
                        <div className="text-xs text-[var(--color-info)]">Mis à jour</div>
                        <div className="text-xl font-bold text-[var(--color-info)]">{result.updated}</div>
                     </div>
                     <div className="p-3 rounded-lg bg-[var(--color-error-bg)] text-center border border-[var(--color-error)]/20">
                        <div className="text-xs text-[var(--color-error)]">Erreurs</div>
                        <div className="text-xl font-bold text-[var(--color-error)]">{result.errors}</div>
                     </div>
                  </div>
                  
                  {result.details.length > 0 && (
                      <div className="mt-2 p-3 text-xs font-mono bg-[var(--bg-tertiary)] rounded-lg text-[var(--color-error)] max-h-40 overflow-auto border border-[var(--border-default)]">
                          {result.details.map((d, i) => <div key={i}>{d}</div>)}
                      </div>
                  )}
               </div>
            )}

            {error && (
              <div className="px-4 py-3 rounded-lg text-sm bg-[var(--color-error-bg)] text-[var(--color-error)] flex items-center gap-2">
                 <FontAwesomeIcon icon={faExclamationTriangle} />
                 {error}
              </div>
            )}
          </CardBody>
        </Card>
      )}



      {activeTab === "sql" && (
        <Card className="bg-[var(--bg-secondary)] border border-[var(--border-default)]">
          <CardHeader className="flex justify-between items-start gap-4">
            <div className="flex flex-col gap-1">
              <h2 className="font-semibold" style={{ color: "var(--text-primary)" }}>
                Requête SQL (Ateis · Produit)
              </h2>
            </div>
            <div className="flex gap-2">
              <Button
                variant="bordered"
                startContent={<FontAwesomeIcon icon={faRotateRight} />}
                onPress={handleResetQuery}
                isDisabled={isSavingQuery}
                className="border-[var(--border-default)] text-[var(--text-primary)]"
              >
                Réinitialiser
              </Button>
              <Button
                color="primary"
                startContent={<FontAwesomeIcon icon={faSave} />}
                onPress={handleSaveQuery}
                isLoading={isSavingQuery}
                className="bg-[var(--button-primary-bg)] text-white"
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
              <div className="px-4 py-2 rounded-lg text-sm bg-[var(--bg-tertiary)] text-[var(--text-primary)]">
                {saveStatus}
              </div>
            )}
            {error && (
               <div className="px-4 py-2 rounded-lg text-sm bg-[var(--color-error-bg)] text-[var(--color-error)]">
                 {error}
               </div>
            )}
          </CardBody>
        </Card>
      )}
    </div>
  );
}
