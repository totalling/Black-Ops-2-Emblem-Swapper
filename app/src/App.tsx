import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { confirm } from "@tauri-apps/plugin-dialog";
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";
import { api } from "./lib/api";
import type { EmblemCapturedPayload, EmblemInfo, NetworkInfo, PublicMode, Selection } from "./lib/types";
import { AppHeader } from "./components/AppHeader";
import { ModeSwitcher } from "./components/ModeSwitcher";
import { EmblemGrid } from "./components/EmblemGrid";
import { AppFooter } from "./components/AppFooter";
import { ErrorToast } from "./components/ui/ErrorToast";

const CAPTURE_NOTIFY_DELAY_MS = 2500;

async function sendCaptureNotification(count: number) {
  let granted = await isPermissionGranted();
  if (!granted) {
    granted = (await requestPermission()) === "granted";
  }
  if (!granted) return;
  sendNotification(
    count === 1
      ? { title: "Emblem captured", body: "Saved to your library." }
      : { title: `${count} emblems captured`, body: "Saved to your library." },
  );
}

function errorMessage(e: unknown): string {
  if (typeof e === "string") return e;
  if (e instanceof Error) return e.message;
  return "An unexpected error occurred.";
}

function App() {
  const [mode, setMode] = useState<PublicMode>("off");
  const [selected, setSelected] = useState<Selection | null>(null);
  const [emblems, setEmblems] = useState<EmblemInfo[]>([]);
  const [networkInfo, setNetworkInfo] = useState<NetworkInfo | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [selectMode, setSelectMode] = useState(false);
  const [checkedIds, setCheckedIds] = useState<Set<string>>(new Set());

  const runAction = useCallback(async (action: () => Promise<void>) => {
    try {
      await action();
    } catch (e) {
      setError(errorMessage(e));
    }
  }, []);

  const refreshStatus = useCallback(async () => {
    const status = await api.getStatus();
    setMode(status.mode);
    setSelected(status.selected);
  }, []);

  const refreshEmblems = useCallback(async () => {
    setEmblems(await api.listEmblems());
  }, []);

  useEffect(() => {
    runAction(async () => {
      setNetworkInfo(await api.getNetworkInfo());
    });
    runAction(refreshStatus);
    runAction(refreshEmblems);
  }, [runAction, refreshStatus, refreshEmblems]);

  const pendingCaptureCount = useRef(0);
  const captureNotifyTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    const unlisten = listen<EmblemCapturedPayload>("emblem-captured", () => {
      runAction(refreshEmblems);

      pendingCaptureCount.current += 1;
      if (captureNotifyTimer.current) clearTimeout(captureNotifyTimer.current);
      captureNotifyTimer.current = setTimeout(() => {
        sendCaptureNotification(pendingCaptureCount.current);
        pendingCaptureCount.current = 0;
        captureNotifyTimer.current = null;
      }, CAPTURE_NOTIFY_DELAY_MS);
    });
    return () => {
      unlisten.then((fn) => fn());
      if (captureNotifyTimer.current) clearTimeout(captureNotifyTimer.current);
    };
  }, [runAction, refreshEmblems]);

  const handleModeChange = (next: PublicMode) =>
    runAction(async () => {
      await api.setMode(next);
      await refreshStatus();
    });

  const handleSelect = (group: string, slot: number) =>
    runAction(async () => {
      const isAlreadySelected = selected?.group === group && selected?.slot === slot;
      if (isAlreadySelected) {
        await api.clearSelection();
      } else {
        await api.selectEmblem(group, slot);
      }
      await refreshStatus();
    });

  const handleLabelChange = (group: string, slot: number, label: string) =>
    runAction(async () => {
      await api.setEmblemLabel(group, slot, label);
      setEmblems((prev) =>
        prev.map((e) => (e.group === group && e.slot === slot ? { ...e, label } : e)),
      );
    });

  const handleRefresh = () => runAction(refreshEmblems);

  const handleExport = (emblem: EmblemInfo) =>
    runAction(async () => {
      await api.exportEmblem(emblem);
    });

  const handleImport = () =>
    runAction(async () => {
      const imported = await api.importEmblem();
      if (imported) {
        await refreshEmblems();
      }
    });

  const handleDelete = (emblem: EmblemInfo) =>
    runAction(async () => {
      const ok = await confirm(
        `Delete "${emblem.label || `Emblem from ${emblem.captured_at}`}"? This can't be undone.`,
        { title: "Delete emblem", kind: "warning" },
      );
      if (!ok) return;
      await api.deleteEmblem(emblem.group, emblem.slot);
      setCheckedIds((prev) => {
        const next = new Set(prev);
        next.delete(emblem.id);
        return next;
      });
      await Promise.all([refreshEmblems(), refreshStatus()]);
    });

  const handleToggleSelectMode = () => {
    setSelectMode((prev) => !prev);
    setCheckedIds(new Set());
  };

  const handleToggleCheck = (id: string) =>
    setCheckedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });

  const handleToggleSelectAll = () =>
    setCheckedIds((prev) =>
      prev.size === emblems.length ? new Set() : new Set(emblems.map((e) => e.id)),
    );

  const handleBackup = () =>
    runAction(async () => {
      await api.backupLibrary();
    });

  const handleRestore = () =>
    runAction(async () => {
      const count = await api.restoreLibrary();
      if (count !== null) {
        await refreshEmblems();
      }
    });

  const handleBulkDelete = () =>
    runAction(async () => {
      const items = emblems
        .filter((e) => checkedIds.has(e.id))
        .map((e) => ({ group: e.group, slot: e.slot }));
      if (items.length === 0) return;
      const ok = await confirm(
        `Delete ${items.length} emblem${items.length === 1 ? "" : "s"}? This can't be undone.`,
        { title: "Delete emblems", kind: "warning" },
      );
      if (!ok) return;
      await api.deleteEmblems(items);
      setCheckedIds(new Set());
      setSelectMode(false);
      await Promise.all([refreshEmblems(), refreshStatus()]);
    });

  return (
    <div className="safe-area mx-auto flex min-h-screen max-w-[1600px] flex-col lg:h-screen lg:overflow-hidden">
      <AppHeader networkInfo={networkInfo} />

      <main className="flex flex-col gap-4 p-4 sm:p-6 lg:min-h-0 lg:flex-1">
        <div className="shrink-0">
          <ModeSwitcher mode={mode} onChange={handleModeChange} />
        </div>
        <div className="shrink-0 border-t border-border" />
        <EmblemGrid
          emblems={emblems}
          selected={selected}
          onSelect={handleSelect}
          onLabelChange={handleLabelChange}
          onExport={handleExport}
          onDelete={handleDelete}
          onImport={handleImport}
          onRefresh={handleRefresh}
          selectMode={selectMode}
          checkedIds={checkedIds}
          onToggleSelectMode={handleToggleSelectMode}
          onToggleCheck={handleToggleCheck}
          onToggleSelectAll={handleToggleSelectAll}
          onBulkDelete={handleBulkDelete}
          onBackup={handleBackup}
          onRestore={handleRestore}
        />
      </main>

      <AppFooter />

      {error && <ErrorToast message={error} onDismiss={() => setError(null)} />}
    </div>
  );
}

export default App;
