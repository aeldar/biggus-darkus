import type { RuleOptions, RuleProblem, RuleSpec, Trigger } from "./bindings";
import { ActionEditor } from "./ActionEditor";
import { Choice, NumberField } from "./form";

const KINDS = ["state", "event"] as const;

function TriggerEditor(props: {
  trigger: Trigger;
  options: RuleOptions;
  onChange: (t: Trigger) => void;
}) {
  const { trigger, options, onChange } = props;
  // The generated type marks the other key as `never`, so test the value, not the key.
  const kind = trigger.state !== undefined ? "state" : "event";
  return (
    <>
      <Choice
        label="Trigger"
        value={kind}
        options={KINDS}
        onChange={(k) => {
          if (k === kind) return;
          const [state, event] = [options.conditions[0], options.events[0]];
          if (k === "state" && state !== undefined) onChange({ state });
          if (k === "event" && event !== undefined) onChange({ event });
        }}
      />
      {trigger.state !== undefined ? (
        <Choice
          label="While"
          value={trigger.state}
          options={options.conditions}
          onChange={(state) => onChange({ state })}
        />
      ) : (
        <Choice
          label="On event"
          value={trigger.event}
          options={options.events}
          onChange={(event) => onChange({ event })}
        />
      )}
    </>
  );
}

/** One rule: trigger, priority, action, and the backend's complaint about it if any. */
export function RuleCard(props: {
  rule: RuleSpec;
  index: number;
  count: number;
  options: RuleOptions;
  problem: RuleProblem | null;
  onChange: (r: RuleSpec) => void;
  onMove: (by: -1 | 1) => void;
  onRemove: () => void;
}) {
  const { rule, index, count, options, problem, onChange, onMove, onRemove } = props;
  const n = index + 1;
  return (
    <li className={`rule ${problem === null ? "" : "invalid"}`} aria-label={`Rule ${n}`}>
      <div className="rule-head">
        <strong>Rule {n}</strong>
        <span className="row">
          <button onClick={() => onMove(-1)} disabled={index === 0} aria-label={`Move rule ${n} up`}>
            Up
          </button>
          <button onClick={() => onMove(1)} disabled={index === count - 1} aria-label={`Move rule ${n} down`}>
            Down
          </button>
          <button onClick={onRemove} aria-label={`Remove rule ${n}`}>
            Remove
          </button>
        </span>
      </div>
      <div className="fields">
        <TriggerEditor trigger={rule.trigger} options={options} onChange={(trigger) => onChange({ ...rule, trigger })} />
        <NumberField label="Priority" signed value={rule.priority ?? 0} onChange={(v) => onChange({ ...rule, priority: v ?? 0 })} />
        <ActionEditor action={rule.action} fields={options.fields} onChange={(action) => onChange({ ...rule, action })} />
      </div>
      {problem !== null && (
        <p className="error" role="alert">
          {problem.message}
        </p>
      )}
    </li>
  );
}

