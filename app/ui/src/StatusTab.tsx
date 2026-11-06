import { useEffect, useRef, useState } from "react";
import { commands, type Connection, type Game, type Port } from "./bindings";

function describe(c: Connection): string {
  switch (c.kind) {
    case "Stopped":
      return "Stopped";
    case "Connecting":
      return "Connecting...";
    case "Connected":
      return "Connected";
    case "Reconnecting":
      return `Reconnecting (attempt ${c.attempt + 1})`;
  }
}

function ConnectionPanel({ connection }: { connection: Connection }) {
  const [ports, setPorts] = useState<Port[]>([]);
  const [port, setPort] = useState("");
  const [error, setError] = useState<string | null>(null);
  const running = connection.kind !== "Stopped";

  // Keep the choice if it is still there; otherwise take the first port.
  const refresh = async (preferred: string) => {
    const r = await commands.listPorts();
    if (r.status === "error") return setError(r.error);
    setPorts(r.data);
    setPort(r.data.some((x) => x.path === preferred) ? preferred : (r.data[0]?.path ?? ""));
  };

  useEffect(() => {
    // Start from the port used last time.
    void commands.getSettings().then((s) => refresh(s.status === "ok" ? (s.data.port ?? "") : ""));
  }, []);

  const toggle = async () => {
    const r = running ? await commands.stop() : await commands.start(port);
    setError(r.status === "error" ? r.error : null);
  };

  return (
    <section>
      <h2>Board</h2>
      <div className="row">
        <select
          aria-label="Serial port"
          value={port}
          disabled={running}
          onChange={(e) => setPort(e.target.value)}
        >
          {ports.length === 0 && <option value="">No ports found</option>}
          {ports.map((p) => (
            <option key={p.path} value={p.path}>
              {p.path} - {p.description}
            </option>
          ))}
        </select>
        <button onClick={() => void refresh(port)} disabled={running}>
          Refresh
        </button>
        <button className="primary" onClick={() => void toggle()} disabled={!running && port === ""}>
          {running ? "Stop" : "Start"}
        </button>
        <span className={`status ${connection.kind.toLowerCase()}`} role="status">
          {describe(connection)}
        </span>
      </div>
      {error !== null && (
        <p className="error" role="alert">
          {error}
        </p>
      )}
    </section>
  );
}

const show = (v: string | number | null): string => (v === null ? "-" : String(v));

function GamePanel({ game }: { game: Game }) {
  const fuel =
    game.fuel_main === null
      ? null
      : `${game.fuel_main.toFixed(1)}${game.fuel_capacity === null ? "" : ` / ${game.fuel_capacity.toFixed(1)}`} t`;
  const rows: [string, string][] = [
    ["Commander", show(game.commander)],
    ["Ship", show(game.ship)],
    ["System", show(game.system)],
    ["Body", show(game.body)],
    ["Station", show(game.station)],
    ["Fuel", show(fuel)],
    ["Cargo", show(game.cargo_count)],
  ];
  return (
    <section>
      <h2>Game {game.in_game ? "" : "(not in game)"}</h2>
      <dl>
        {rows.map(([name, value]) => (
          <div key={name}>
            <dt>{name}</dt>
            <dd>{value}</dd>
          </div>
        ))}
      </dl>
      <ul className="flags">
        {game.flags.map((f) => (
          <li key={f}>{f}</li>
        ))}
      </ul>
    </section>
  );
}

function LogPanel({ lines }: { lines: string[] }) {
  const end = useRef<HTMLDivElement>(null);
  // Follow the newest line, as a terminal does.
  // Braces matter: an expression body would hand scrollIntoView's return value to React as a cleanup.
  useEffect(() => {
    end.current?.scrollIntoView({ block: "end" });
  }, [lines]);
  return (
    <section className="log">
      <h2>Commands</h2>
      <pre>
        {lines.map((l, i) => (
          <div key={i}>{l}</div>
        ))}
        <div ref={end} />
      </pre>
    </section>
  );
}

export function StatusTab(props: { connection: Connection; game: Game; log: string[] }) {
  return (
    <div className="stack">
      <ConnectionPanel connection={props.connection} />
      <GamePanel game={props.game} />
      <LogPanel lines={props.log} />
    </div>
  );
}
