import { useState } from "react";
import { SetupGuideModal } from "./SetupGuideModal";

export function AppFooter() {
  const [showGuide, setShowGuide] = useState(false);

  return (
    <>
      <footer className="flex shrink-0 items-center justify-end border-t border-border px-8 py-2">
        <button
          onClick={() => setShowGuide(true)}
          className="rounded-lg border border-border px-3 py-1.5 text-sm font-bold transition-colors hover:border-fg hover:bg-surface"
        >
          Setup &amp; Troubleshooting
        </button>
      </footer>

      {showGuide && <SetupGuideModal onClose={() => setShowGuide(false)} />}
    </>
  );
}
