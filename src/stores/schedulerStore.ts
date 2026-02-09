import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

interface TaskState {
  running: boolean;
  intervalMinutes: number;
  param?: string;
}

interface SchedulerStore {
  tasks: Record<string, TaskState>;
  syncStatus: () => Promise<void>;
  startTask: (taskType: string, intervalMinutes: number, param?: string) => Promise<void>;
  stopTask: (taskType: string) => Promise<void>;
  setTaskInterval: (taskType: string, intervalMinutes: number) => void;
  setTaskParam: (taskType: string, param: string) => void;
  loadIntervals: () => void;
}

const DEFAULT_INTERVAL = 60; // 1 hour default

export const useSchedulerStore = create<SchedulerStore>((set) => ({
  tasks: {
    "ATEIS_PRODUIT_SYNC": { running: false, intervalMinutes: DEFAULT_INTERVAL },
    "ATEIS_OF_SYNC": { running: false, intervalMinutes: DEFAULT_INTERVAL },
    "LOGITRON_PRODUIT": { running: false, intervalMinutes: DEFAULT_INTERVAL, param: "" },
    "LOGITRON_OF": { running: false, intervalMinutes: DEFAULT_INTERVAL, param: "" },
    "ATEIS_EXPORT": { running: false, intervalMinutes: DEFAULT_INTERVAL },
  },

  syncStatus: async () => {
    const taskTypes = ["ATEIS_PRODUIT_SYNC", "ATEIS_OF_SYNC", "LOGITRON_PRODUIT", "LOGITRON_OF", "ATEIS_EXPORT"];
    const updates: Record<string, Partial<TaskState>> = {};

    for (const type of taskTypes) {
      try {
        const running = await invoke<boolean>("get_scheduler_status", { taskType: type });
        updates[type] = { running };
      } catch (e) {
        console.error(`Failed to get status for ${type}`, e);
      }
    }

    set((state) => {
        const newTasks = { ...state.tasks };
        for (const [type, update] of Object.entries(updates)) {
            if (newTasks[type]) {
                newTasks[type] = { ...newTasks[type], ...update };
            }
        }
        return { tasks: newTasks };
    });
  },

  startTask: async (taskType: string, intervalMinutes: number, param?: string) => {
    try {
      await invoke("start_scheduler", { taskType, intervalMinutes, param: param || null });
      set((state) => ({
        tasks: {
          ...state.tasks,
          [taskType]: { ...state.tasks[taskType], running: true, intervalMinutes, param }
        }
      }));
    } catch (e) {
      console.error(`Failed to start ${taskType}`, e);
      throw e;
    }
  },

  stopTask: async (taskType: string) => {
    try {
      await invoke("stop_scheduler", { taskType });
      set((state) => ({
        tasks: {
          ...state.tasks,
          [taskType]: { ...state.tasks[taskType], running: false }
        }
      }));
    } catch (e) {
      console.error(`Failed to stop ${taskType}`, e);
      throw e;
    }
  },

  setTaskInterval: (taskType: string, intervalMinutes: number) => {
    set((state) => ({
        tasks: {
            ...state.tasks,
            [taskType]: { ...state.tasks[taskType], intervalMinutes }
        }
    }));
    // Persist
    localStorage.setItem(`${taskType}_interval_minutes`, intervalMinutes.toString());
  },

  setTaskParam: (taskType: string, param: string) => {
    set((state) => ({
        tasks: {
            ...state.tasks,
            [taskType]: { ...state.tasks[taskType], param }
        }
    }));
    // Persist
    localStorage.setItem(`${taskType}_param`, param);
  },

  loadIntervals: () => {
    const taskTypes = ["ATEIS_PRODUIT_SYNC", "ATEIS_OF_SYNC", "LOGITRON_PRODUIT", "LOGITRON_OF", "ATEIS_EXPORT"];
    const updates: Record<string, Partial<TaskState>> = {};

    for (const type of taskTypes) {
        // Load Interval
        const savedInterval = localStorage.getItem(`${type}_interval_minutes`);
        if (savedInterval) {
            const val = Number(savedInterval);
            if (Number.isFinite(val) && val > 0) {
                updates[type] = { ...updates[type], intervalMinutes: val };
            }
        }

        // Load Param (Output Path) - including legacy migration
        let savedParam = localStorage.getItem(`${type}_param`);
        
        // Legacy Migration for Logitron
        if (!savedParam) {
            if (type === "LOGITRON_PRODUIT") {
                savedParam = localStorage.getItem("logitron_produit_output_path");
                if (savedParam) localStorage.setItem(`${type}_param`, savedParam); // Migrate
            } else if (type === "LOGITRON_OF") {
                savedParam = localStorage.getItem("logitron_of_output_path");
                if (savedParam) localStorage.setItem(`${type}_param`, savedParam); // Migrate
            }
        }

        if (savedParam !== null) {
            updates[type] = { ...updates[type], param: savedParam };
        }
    }

    set((state) => {
        const newTasks = { ...state.tasks };
        for (const [type, update] of Object.entries(updates)) {
            if (newTasks[type]) {
                newTasks[type] = { ...newTasks[type], ...update };
            }
        }
        return { tasks: newTasks };
    });
  }
}));
