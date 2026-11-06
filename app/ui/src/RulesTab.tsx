import { useEffect, useState } from "react";
import { commands, type RuleOptions, type RuleProblem, type RuleSpec } from "./bindings";
import { defaultAction } from "./ActionEditor";
import { RuleCard } from "./RuleCard";

const newRule = (): RuleSpec => ({
  trigger: { state: "in_game" },
  action: defaultAction("led"),
  priority: 0,
});

/** Immutable list helpers: the editor state is always replaced, never changed in place. */
const replaceAt = <T,>(xs: T[], i: number, x: T): T[] => xs.map((old, j) => (j === i ? x : old));
const removeAt = <T,>(xs: T[], i: number): T[] => xs.filter((_, j) => j !== i);
function swap<T>(xs: T[], i: number, j: number): T[] {
  const [a, b] = [xs[i], xs[j]];
  return a === undefined || b === undefined ? xs : xs.map((x, k) => (k === i ? b : k === j ? a : x));
}

const problemText = (p: RuleProblem) => p.message;

export function RulesTab() {
  const [options, setOptions] = useState<RuleOptions | null>(null);
  const [rules, setRules] = useState<RuleSpec[] | null>(null);
  const [saved, setSaved] = useState("");
  const [problem, setProblem] = useState<RuleProblem | null>(null);
  const [toml, setToml] = useState("");
  const [note, setNote] = useState<string | null>(null);
  const [confirmReset, setConfirmReset] = useState(false);

  useEffect(() => {
    void commands.ruleOptions().then(setOptions);
    void commands.getRules().then((r) => {
      if (r.status === "ok") {
        setRules(r.data);
        setSaved(JSON.stringify(r.data));
      } else setProblem(r.error);
    });
  }, []);

  // Check every edit with the backend, so the errors are the ones saving would give.
  useEffect(() => {
    if (rules === null) return;
    // Replies can arrive out of order, so only the newest request may set the result.
    let current = true;
    void commands.validateRules(rules).then((r) => {
      if (!current) return;
      if (r.status === "ok") {
        setToml(r.data);
        setProblem(null);
      } else setProblem(r.error);
    });
    return () => {
      current = false;
    };
  }, [rules]);

  if (rules === null || options === null) {
    return <section>{problem !== null && <p className="error">{problemText(problem)}</p>}</section>;
  }

  const edit = (next: RuleSpec[]) => {
    setRules(next);
    setNote(null);
    setConfirmReset(false);
  };
  const dirty = JSON.stringify(rules) !== saved;

  const save = async () => {
    const r = await commands.saveRules(rules);
    if (r.status === "error") return setProblem(r.error);
    setSaved(JSON.stringify(rules));
    setNote("Saved. A running board uses the new rules now.");
  };

  const reset = async () => {
    const r = await commands.resetToDefaults();
    setConfirmReset(false);
    if (r.status === "error") return setProblem(r.error);
    setRules(r.data);
    setSaved(JSON.stringify(r.data));
    setNote("Back to the built-in rules.");
  };

  return (
    <div className="stack-auto">
      <section>
        <h2>Rules</h2>
        <p className="muted">
          Rules that share an LED or LCD row compete: the higher priority wins, and the rule listed first wins a tie.
        </p>
        <ol className="rules">
          {rules.map((rule, i) => (
            <RuleCard
              key={i}
              rule={rule}
              index={i}
              count={rules.length}
              options={options}
              problem={problem?.rule === i + 1 ? problem : null}
              onChange={(r) => edit(replaceAt(rules, i, r))}
              onMove={(by) => edit(swap(rules, i, i + by))}
              onRemove={() => edit(removeAt(rules, i))}
            />
          ))}
        </ol>
        <div className="row">
          <button onClick={() => edit([...rules, newRule()])}>Add rule</button>
          <button className="primary" onClick={() => void save()} disabled={problem !== null || !dirty}>
            Save
          </button>
          {confirmReset ? (
            <>
              <button onClick={() => void reset()}>Confirm: discard my rules</button>
              <button onClick={() => setConfirmReset(false)}>Cancel</button>
            </>
          ) : (
            <button onClick={() => setConfirmReset(true)}>Reset to defaults</button>
          )}
          {dirty && <span className="muted">Unsaved changes</span>}
        </div>
        {problem !== null && (
          <p className="error" role="alert">
            {problemText(problem)}
          </p>
        )}
        {note !== null && <p className="muted" role="status">{note}</p>}
      </section>
      <section>
        <details>
          <summary>Advanced: raw TOML (read-only)</summary>
          <pre className="toml">{problem === null ? toml : "Fix the error above to see the TOML."}</pre>
        </details>
      </section>
    </div>
  );
}
