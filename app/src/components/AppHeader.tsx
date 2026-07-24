import type { NetworkInfo } from "../lib/types";

interface Props {
  networkInfo: NetworkInfo | null;
}

export function AppHeader({ networkInfo }: Props) {
  const setupValue = networkInfo
    ? networkInfo.lan_ip
      ? `${networkInfo.lan_ip} : ${networkInfo.proxy_port}`
      : "couldn't detect, see docs/INSTALL.md"
    : "detecting…";

  return (
    <header className="flex shrink-0 flex-wrap items-center justify-between gap-4 border-b border-border px-8 py-4">
      <div className="flex items-center gap-4">
        <img
          src="/bo2ripperlogo.png"
          alt=""
          className="h-24 w-24 object-contain invert dark:invert-0"
        />
        <div>
          <h1 className="text-lg font-bold tracking-tight">Black Ops 2 Emblem Swapper</h1>
          <p className="text-sm text-muted">
            If you can see an emblem in Black Ops II, you can make it yours
          </p>
        </div>
      </div>

      <div className="rounded-xl bg-surface px-4 py-2.5 text-right">
        <div className="text-[10px] font-bold uppercase tracking-wider text-muted">
          PS5 proxy setting
        </div>
        <div className="font-bold tabular-nums">{setupValue}</div>
      </div>
    </header>
  );
}
