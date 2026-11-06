import { useState } from "react";
import { ManualTab } from "./ManualTab";
import { RulesTab } from "./RulesTab";
import { SettingsTab } from "./SettingsTab";
import { StatusTab } from "./StatusTab";
import { useSnapshot } from "./useSnapshot";

const TABS = ["Status", "Rules", "Manual", "Settings"] as const;
type Tab = (typeof TABS)[number];

export function App() {
  // The snapshot lives here so the log and game state keep updating while another tab is open.
  const { connection, game, log } = useSnapshot();
  const [tab, setTab] = useState<Tab>("Status");
  return (
    <main>
      <div role="tablist" aria-label="Sections" className="tabs">
        {TABS.map((t) => (
          <button
            key={t}
            role="tab"
            id={`tab-${t}`}
            aria-selected={tab === t}
            aria-controls="panel"
            onClick={() => setTab(t)}
          >
            {t}
          </button>
        ))}
      </div>
      <div role="tabpanel" id="panel" aria-labelledby={`tab-${tab}`} className="panel">
        {tab === "Status" && <StatusTab connection={connection} game={game} log={log} />}
        {tab === "Rules" && <RulesTab />}
        {tab === "Manual" && <ManualTab connection={connection} />}
        {tab === "Settings" && <SettingsTab />}
      </div>
    </main>
  );
}
