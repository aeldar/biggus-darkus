import { useEffect, useState } from "react";
import { commands, type Settings } from "./bindings";
import { Field } from "./form";

export function SettingsTab() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [note, setNote] = useState<{ text: string; bad: boolean } | null>(null);

  useEffect(() => {
    void commands.getSettings().then((r) => {
      if (r.status === "ok") setSettings(r.data);
      else setNote({ text: r.error, bad: true });
    });
  }, []);

  if (settings === null) return <section>{note && <p className="error">{note.text}</p>}</section>;

  const detect = async () => {
    const found = await commands.detectJournalDir();
    if (found === null) return setNote({ text: "No journal folder found. Type its path.", bad: true });
    setSettings({ ...settings, journal_dir: found });
    setNote({ text: `Detected: ${found}`, bad: false });
  };

  const save = async () => {
    const r = await commands.saveSettings(settings);
    setNote(
      r.status === "ok"
        ? { text: "Saved. Changes apply the next time you press Start.", bad: false }
        : { text: r.error, bad: true },
    );
  };

  return (
    <section>
      <h2>Settings</h2>
      <div className="form">
        <Field label="Journal folder (empty = find automatically)">
          <input
            type="text"
            value={settings.journal_dir ?? ""}
            placeholder="Auto-detect"
            onChange={(e) =>
              setSettings({ ...settings, journal_dir: e.target.value === "" ? null : e.target.value })
            }
          />
        </Field>
        <div className="row">
          <button onClick={() => void detect()}>Auto-detect</button>
          <button onClick={() => setSettings({ ...settings, journal_dir: null })}>Use automatic</button>
        </div>
        <p className="muted">Last used port: {settings.port ?? "none yet"}</p>
        <label className="check">
          <input
            type="checkbox"
            checked={settings.auto_start}
            onChange={(e) => setSettings({ ...settings, auto_start: e.target.checked })}
          />
          Start automatically when the app opens (needs a last used port)
        </label>
        <div className="row">
          <button className="primary" onClick={() => void save()}>
            Save settings
          </button>
        </div>
        {note !== null && (
          <p className={note.bad ? "error" : "muted"} role="status">
            {note.text}
          </p>
        )}
      </div>
    </section>
  );
}
