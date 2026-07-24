interface Props {
  onClose: () => void;
}

const FIREWALL_CMD =
  'New-NetFirewallRule -DisplayName "Black Ops 2 Emblem Swapper" -Direction Inbound -Protocol TCP -LocalPort 8080 -Action Allow -Profile Private';

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="border-t border-border pt-5 first:border-t-0 first:pt-0">
      <h4 className="font-bold tracking-tight">{title}</h4>
      <div className="mt-2 space-y-2 text-sm text-muted">{children}</div>
    </section>
  );
}

function SubHeading({ children }: { children: React.ReactNode }) {
  return <h5 className="pt-1 text-sm font-bold text-fg">{children}</h5>;
}

export function SetupGuideModal({ onClose }: Props) {
  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-8"
      onClick={onClose}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        className="max-h-[85vh] w-full max-w-2xl overflow-y-auto rounded-2xl border border-border bg-bg p-6"
      >
        <div className="mb-5 flex items-center justify-between">
          <h3 className="text-lg font-bold tracking-tight">PS5 Setup &amp; Instructions</h3>
          <button
            onClick={onClose}
            className="rounded-lg border border-border px-2 py-1 text-sm hover:border-fg hover:bg-surface"
          >
            Close
          </button>
        </div>

        <div className="space-y-5">
          <Section title="Setup">
            <p className="border-l-2 border-fg/40 pl-3">
              <strong className="text-fg">Note:</strong> Your PS5 and PC must be connected to the
              exact same network (same Wi-Fi, Ethernet, or PC mobile hotspot).
            </p>
            <ol className="list-decimal space-y-1 pl-5">
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
            <ol className="list-decimal space-y-1 pl-5">
              <li>
                Turn on <strong className="text-fg">Capture Mode</strong> in the app.
              </li>
              <li>
                On your PS5, view the profile or channel of the player whose emblem you want. The
                app saves it automatically.
              </li>
              <li>In the app, select the captured emblem.</li>
              <li>
                Turn on <strong className="text-fg">Show Mode</strong>.
              </li>
              <li>Open your PS5 emblem editor. The selected emblem will load in place of your own.</li>
              <li>Save the emblem.</li>
            </ol>
          </Section>

          <Section title="Game Limitations">
            <ul className="list-disc space-y-1 pl-5">
              <li>
                <strong className="text-fg">Session Caching:</strong> Black Ops II only fetches
                emblem data once per launch. To apply a different emblem after opening the editor,
                restart the game or switch between Multiplayer and Zombies to clear the cache.
              </li>
              <li>
                <strong className="text-fg">Locked Content:</strong> You cannot save emblems that
                contain shapes or layers you haven&apos;t unlocked on your account. The game
                enforces this natively.
              </li>
            </ul>
          </Section>

          <Section title="Troubleshooting">
            <SubHeading>PS5 says &quot;No Internet Connection&quot;</SubHeading>
            <ul className="list-disc space-y-1 pl-5">
              <li>
                <strong className="text-fg">Wrong Network:</strong> Confirm both devices share the
                same IP range. If using a PC hotspot, select the hotspot&apos;s IP address in the
                app, not your router&apos;s IP.
              </li>
              <li>
                <strong className="text-fg">Firewall:</strong> Windows Firewall may be blocking
                incoming traffic on port <code className="text-fg">8080</code>. Run this in{" "}
                <strong className="text-fg">PowerShell (Admin)</strong> to fix it:
                <pre className="mt-2 overflow-x-auto rounded-lg border border-border bg-surface p-3 text-xs text-fg">
                  {FIREWALL_CMD}
                </pre>
              </li>
              <li>
                <strong className="text-fg">DNS Blockers:</strong> Ad-blocking DNS services can
                interfere with PlayStation network checks. Switch your PS5 DNS to{" "}
                <code className="text-fg">1.1.1.1</code> or <code className="text-fg">8.8.8.8</code>.
              </li>
            </ul>

            <SubHeading>Incorrect IP Address Detected</SubHeading>
            <p>
              If you use VPNs or virtual network adapters, the app might select the wrong local
              IP. Find your correct local IP manually:
            </p>
            <ul className="list-disc space-y-1 pl-5">
              <li>
                <strong className="text-fg">Windows:</strong> Run <code className="text-fg">ipconfig</code>{" "}
                in Command Prompt.
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
            <ul className="list-disc space-y-1 pl-5">
              <li>
                <strong className="text-fg">Capture Mode:</strong> Intercepts and downloads emblem
                assets from the server to your PC.
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
  );
}
