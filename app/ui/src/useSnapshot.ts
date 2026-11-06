import { useEffect, useState } from "react";
import { commands, events, type Snapshot } from "./bindings";

/** Keep in step with LOG_LIMIT in src-tauri/src/dto.rs. */
const LOG_LIMIT = 200;

const empty: Snapshot = {
  connection: { kind: "Stopped" },
  game: {
    in_game: false,
    commander: null,
    ship: null,
    system: null,
    body: null,
    station: null,
    fuel_main: null,
    fuel_capacity: null,
    cargo_count: null,
    flags: [],
  },
  log: [],
};

/** The backend's snapshot, kept fresh by its events. */
export function useSnapshot(): Snapshot {
  const [snapshot, setSnapshot] = useState(empty);

  useEffect(() => {
    // Subscribe before fetching, so nothing that happens in between is lost.
    const subscriptions = [
      events.connection.listen((e) =>
        setSnapshot((s) => ({ ...s, connection: e.payload })),
      ),
      events.game.listen((e) => setSnapshot((s) => ({ ...s, game: e.payload }))),
      events.logLine.listen((e) =>
        setSnapshot((s) => ({ ...s, log: [...s.log, e.payload].slice(-LOG_LIMIT) })),
      ),
    ];
    void commands.getSnapshot().then(setSnapshot);
    return () => {
      // Unlisten functions arrive asynchronously, so wait for them before calling.
      void Promise.all(subscriptions).then((offs) => offs.forEach((off) => off()));
    };
  }, []);

  return snapshot;
}
