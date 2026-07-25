import { useState } from "react";
import { api } from "../lib/api";
import type { DiagnosticCheck, DiagnosticStatus } from "../lib/types";
import { IconClose, IconPulse, IconWrench } from "./icons";

interface Props {
  onClose: () => void;
}

const FIREWALL_CMD =
  'New-NetFirewallRule -DisplayName "Black Ops 2 Emblem Swapper" -Direction Inbound -Protocol TCP -LocalPort 8080 -Action Allow -Profile Private';

function IconTile({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-surface">
      {children}
    </div>
  );
}

function Section({
  title,
  icon,
  children,
}: {
  title: string;
  icon?: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <section className="rounded-xl border border-border/70 bg-surface/40 p-5">
      <div className="flex items-center gap-3">
        {icon && <IconTile>{icon}</IconTile>}
        <h4 className="font-bold tracking-tight">{title}</h4>
      </div>
      <div className="mt-3 space-y-2.5 text-sm leading-relaxed text-muted">{children}</div>
    </section>
  );
}

function SubHeading({ first, children }: { first?: boolean; children: React.ReactNode }) {
  return (
    <h5
      className={`text-xs font-bold uppercase tracking-wider text-fg/70 ${
        first ? "" : "mt-4 border-t border-border/60 pt-4"
      }`}
    >
      {children}
    </h5>
  );
}

const STATUS_LABEL: Record<DiagnosticStatus, string> = {
  ok: "OK",
  warn: "Warn",
  fail: "Fail",
  unknown: "?",
};

const STATUS_STYLE: Record<DiagnosticStatus, string> = {
  ok: "border-l-success text-success",
  warn: "border-l-amber-500 text-amber-500",
  fail: "border-l-danger text-danger",
  unknown: "border-l-muted text-muted",
};

function DiagnosticsPanel() {
  const [running, setRunning] = useState(false);
  const [results, setResults] = useState<DiagnosticCheck[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleRun = async () => {
    setRunning(true);
    setError(null);
    try {
      setResults(await api.runConnectionDiagnostics());
    } catch (e) {
      setError(String(e));
    } finally {
      setRunning(false);
    }
  };

  return (
    <Section title="Quick Diagnostics" icon={<IconPulse size={16} />}>
      <p>
        Runs a few automated checks (proxy status, network type, firewall rule) covering the most
        common reasons the PS5 can&apos;t connect.
      </p>
      <button
        onClick={handleRun}
        disabled={running}
        className={`flex items-center gap-2 rounded-lg px-4 py-2 text-sm font-bold transition-all active:scale-[0.97] ${
          running ? "cursor-not-allowed bg-fg/50 text-bg" : "bg-fg text-bg hover:opacity-85"
        }`}
      >
        <span className={running ? "animate-pulse" : undefined}>
          <IconPulse size={14} />
        </span>
        {running ? "Running…" : "Run Diagnostics"}
      </button>

      {error && (
        <p className="rounded-lg border border-danger/40 bg-danger/10 p-3 text-sm text-danger">
          {error}
        </p>
      )}

      {results && (
        <ul className="animate-fade-in space-y-2 pt-1">
          {results.map((r) => (
            <li
              key={r.name}
              className={`flex items-start justify-between gap-3 rounded-lg border border-l-4 border-border/60 bg-bg p-3 ${STATUS_STYLE[r.status]}`}
            >
              <div className="min-w-0">
                <p className="text-sm font-bold text-fg">{r.name}</p>
                <p className="mt-0.5 text-sm text-muted">{r.detail}</p>
              </div>
              <span className="shrink-0 text-xs font-bold uppercase tracking-wider">
                {STATUS_LABEL[r.status]}
              </span>
            </li>
          ))}
        </ul>
      )}
    </Section>
  );
}

export function SetupGuideModal({ onClose }: Props) {
  return (
    <div
      className="animate-overlay-in fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-8 backdrop-blur-sm"
      onClick={onClose}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        className="animate-modal-in flex max-h-[85vh] w-full max-w-2xl flex-col overflow-hidden rounded-2xl border border-border bg-bg shadow-2xl"
      >
        <div className="flex shrink-0 items-start justify-between border-b border-border px-6 py-5">
          <div>
            <h3 className="text-lg font-bold tracking-tight">PS5 Setup &amp; Instructions</h3>
            <p className="mt-0.5 text-sm text-muted">
              Everything you need to get capturing and showing emblems
            </p>
          </div>
          <button
            onClick={onClose}
            className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg border border-border text-muted transition-all hover:border-fg hover:bg-surface hover:text-fg active:scale-[0.96]"
          >
            <IconClose size={14} />
          </button>
        </div>

        <div className="scroll-thin flex-1 overflow-y-auto px-6 py-6">
          <div className="space-y-4">
            <DiagnosticsPanel />

            <Section title="Setup">
              <p>
                Your PS5 and PC must be connected to the exact same network (same Wi-Fi,
                Ethernet, or PC mobile hotspot).
              </p>
              <ol className="list-decimal space-y-1.5 pl-5">
                <li>
                  On PS5: go to <strong className="text-fg">Settings</strong> →{" "}
                  <strong className="text-fg">Network</strong> →{" "}
                  <strong className="text-fg">Settings</strong> →{" "}
                  <strong className="text-fg">Set Up Internet Connection</strong>.
                </li>
                <li>
                  Select your active connection, then select{" "}
                  <strong className="text-fg">Advanced Settings</strong>.
                </li>
                <li>
                  Set <strong className="text-fg">Proxy Server</strong> to{" "}
                  <strong className="text-fg">Use</strong>.
                </li>
                <li>
                  Enter the <strong className="text-fg">IP Address</strong> and{" "}
                  <strong className="text-fg">Port</strong> shown in the app (e.g.,{" "}
                  <code className="text-fg">192.168.1.42</code> and{" "}
                  <code className="text-fg">8080</code>).
                </li>
                <li>Save settings and test the connection.</li>
              </ol>
            </Section>

            <Section title="How to Copy Emblems">
              <ol className="list-decimal space-y-1.5 pl-5">
                <li>
                  Turn on <strong className="text-fg">Capture Mode</strong> in the app.
                </li>
                <li>
                  On your PS5, view the profile or channel of the player whose emblem you want.
                  The app saves it automatically.
                </li>
                <li>In the app, select the captured emblem.</li>
                <li>
                  Turn on <strong className="text-fg">Show Mode</strong>.
                </li>
                <li>
                  Open your PS5 emblem editor. The selected emblem will load in place of your own.
                </li>
                <li>Save the emblem.</li>
              </ol>
            </Section>

            <Section title="Game Limitations">
              <ul className="list-disc space-y-1.5 pl-5">
                <li>
                  <strong className="text-fg">Session Caching:</strong> Black Ops II only fetches
                  emblem data once per launch. To apply a different emblem after opening the
                  editor, restart the game or switch between Multiplayer and Zombies to clear the
                  cache.
                </li>
                <li>
                  <strong className="text-fg">Locked Content:</strong> You cannot save emblems
                  that contain shapes or layers you haven&apos;t unlocked on your account. The
                  game enforces this natively.
                </li>
              </ul>
            </Section>

            <Section title="Troubleshooting" icon={<IconWrench size={16} />}>
              <SubHeading first>PS5 says &quot;No Internet Connection&quot;</SubHeading>
              <ul className="list-disc space-y-1.5 pl-5">
                <li>
                  <strong className="text-fg">Wrong Network:</strong> Confirm both devices share
                  the same IP range. If using a PC hotspot, select the hotspot&apos;s IP address
                  in the app, not your router&apos;s IP.
                </li>
                <li>
                  <strong className="text-fg">Firewall:</strong> Windows Firewall may be blocking
                  incoming traffic on port <code className="text-fg">8080</code>. Run this in{" "}
                  <strong className="text-fg">PowerShell (Admin)</strong> to fix it:
                  <pre className="mt-2 overflow-x-auto rounded-lg border border-border bg-bg p-3 text-xs text-fg">
                    {FIREWALL_CMD}
                  </pre>
                </li>
                <li>
                  <strong className="text-fg">DNS Blockers:</strong> Ad-blocking DNS services can
                  interfere with PlayStation network checks. Switch your PS5 DNS to{" "}
                  <code className="text-fg">1.1.1.1</code> or{" "}
                  <code className="text-fg">8.8.8.8</code>.
                </li>
              </ul>

              <SubHeading>Incorrect IP Address Detected</SubHeading>
              <p>
                If you use VPNs or virtual network adapters, the app might select the wrong local
                IP. Find your correct local IP manually:
              </p>
              <ul className="list-disc space-y-1.5 pl-5">
                <li>
                  <strong className="text-fg">Windows:</strong> Run{" "}
                  <code className="text-fg">ipconfig</code> in Command Prompt.
                </li>
                <li>
                  <strong className="text-fg">macOS / Linux:</strong> Run{" "}
                  <code className="text-fg">ifconfig</code> or <code className="text-fg">ip addr</code>{" "}
                  in Terminal.
                </li>
              </ul>
            </Section>

            <Section title="Technical Overview">
              <p>
                Black Ops II fetches emblem data via plain HTTP from Demonware servers. This tool
                acts as a local proxy targeting only those specific emblem endpoints:
              </p>
              <ul className="list-disc space-y-1.5 pl-5">
                <li>
                  <strong className="text-fg">Capture Mode:</strong> Intercepts and downloads
                  emblem assets from the server to your PC.
                </li>
                <li>
                  <strong className="text-fg">Show Mode:</strong> Intercepts outgoing requests and
                  feeds your selected local emblem back to the console.
                </li>
              </ul>
              <p>
                All HTTPS traffic, PSN authentication, and matchmaking data pass through
                un-decrypted and untouched.
              </p>
            </Section>
          </div>
        </div>
      </div>
    </div>
  );
}
