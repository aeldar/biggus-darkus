import { useEffect, useState } from "react";
import { commands, type CommandHelp, type Connection } from "./bindings";

type Entry = { line: string; outcome: string; bad: boolean };

/** The parse result of what is typed: the line as it will be sent, or the reason it cannot be. */
function useCheck(text: string): { line: string | null; error: string | null } {
  const [result, setResult] = useState<{ line: string | null; error: string | null }>({
    line: null,
    error: null,
  });
  useEffect(() => {
    // Replies can arrive out of order, so only the newest request may set the result.
    let current = true;
    void commands.checkCommand(text).then((r) => {
      if (current) setResult(r.status === "ok" ? { line: r.data, error: null } : { line: null, error: r.error });
    });
    return () => {
      current = false;
    };
  }, [text]);
  return result;
}

function Hints({ help, onPick }: { help: CommandHelp[]; onPick: (example: string) => void }) {
  return (
    <section>
      <h2>Commands</h2>
      <ul className="help">
        {help.map((h) => (
          <li key={h.syntax}>
            <button onClick={() => onPick(h.example)} title={h.description}>
              {h.title}
            </button>
            <code>{h.syntax}</code>
            <span className="muted">e.g. {h.example}</span>
          </li>
        ))}
      </ul>
    </section>
  );
}

export function ManualTab({ connection }: { connection: Connection }) {
  const [text, setText] = useState("");
  const [help, setHelp] = useState<CommandHelp[]>([]);
  const [history, setHistory] = useState<Entry[]>([]);
  const { line, error } = useCheck(text);
  const connected = connection.kind === "Connected";

  useEffect(() => {
    void commands.commandHelp().then(setHelp);
  }, []);

  const send = async () => {
    const r = await commands.sendCommand(text);
    const entry: Entry =
      r.status === "ok"
        ? { line: line ?? text, outcome: r.data, bad: r.data !== "OK" }
        : { line: line ?? text, outcome: r.error, bad: true };
    setHistory((h) => [...h, entry].slice(-50));
  };

  return (
    <div className="stack-auto">
      <section>
        <h2>Send a command</h2>
        {!connected && <p className="muted">Start the board on the Status tab to send commands.</p>}
        <form
          className="row"
          onSubmit={(e) => {
            e.preventDefault();
            void send();
          }}
        >
          <input
            type="text"
            aria-label="Protocol command"
            aria-invalid={text !== "" && error !== null}
            className="grow mono"
            value={text}
            placeholder="LCD ROW 1 Hello"
            spellCheck={false}
            onChange={(e) => setText(e.target.value)}
          />
          <button className="primary" type="submit" disabled={!connected || line === null}>
            Send
          </button>
        </form>
        <p className={text !== "" && error !== null ? "error" : "muted"} aria-live="polite">
          {text === "" ? "Type a command, or pick one below." : (error ?? `Will send: ${line}`)}
        </p>
        <ul className="history" aria-label="Replies">
          {history.map((h, i) => (
            <li key={i} className={h.bad ? "bad" : ""}>
              <code>{h.line}</code> {"->"} {h.outcome}
            </li>
          ))}
        </ul>
      </section>
      <Hints help={help} onPick={setText} />
    </div>
  );
}
