import type { NetworkInfo } from "../lib/types";

interface Props {
  networkInfo: NetworkInfo | null;
}

export function AppHeader({ networkInfo }: Props) {
  return (
    <header className="flex shrink-0 flex-wrap items-center justify-between gap-4 border-b border-border px-4 py-3 sm:px-8 sm:py-4">
      <div className="flex items-center gap-4">
        <img
          src="/bo2ripperlogo.png"
          alt=""
          className="h-16 w-16 object-contain invert sm:h-24 sm:w-24 dark:invert-0"
        />
        <div>
          <h1 className="text-lg font-bold tracking-tight">Black Ops 2 Emblem Swapper</h1>
          <p className="text-sm text-muted">
            If you can see an emblem in Black Ops II, you can make it yours
          </p>
        </div>
      </div>

      <div className="rounded-xl border border-border bg-surface px-5 py-3">
        <div className="text-[10px] font-bold uppercase tracking-wider text-muted">
          PS5 proxy setting
        </div>
        {!networkInfo ? (
          <div className="text-sm text-muted">detecting…</div>
        ) : networkInfo.lan_ip ? (
          <div className="font-mono text-base font-bold tracking-tight tabular-nums">
            {networkInfo.lan_ip}
            <span className="mx-1 text-muted">:</span>
            {networkInfo.proxy_port}
          </div>
        ) : (
          <div className="text-sm text-muted">couldn&apos;t detect, see docs/INSTALL.md</div>
        )}
      </div>
    </header>
  );
}
