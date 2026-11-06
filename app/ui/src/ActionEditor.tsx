import type { ActionSpec, AnimSpec, FourShow, SixShow } from "./bindings";
import { Choice, Field, NumberField } from "./form";

type Led = Extract<ActionSpec, { output: "led" }>;
type Lcd = Extract<ActionSpec, { output: "lcd" }>;
type Four = Extract<ActionSpec, { output: "four_digit" }>;
type Six = Extract<ActionSpec, { output: "six_digit" }>;

export const OUTPUTS = ["led", "lcd", "four_digit", "six_digit"] as const;

/** A fresh action of the given kind, with values that are valid on their own. */
export function defaultAction(output: ActionSpec["output"]): ActionSpec {
  switch (output) {
    case "led":
      return { output, start: 0, end: 1, color: "FFFFFF", anim: "solid", period_ms: null };
    case "lcd":
      return { output, row: 1, text: "{system}" };
    case "four_digit":
      return { output, show: "time", at: null, seconds: null, pause: false };
    case "six_digit":
      return { output, show: "reset", value: null, period_ms: null };
  }
}

const ANIMS: readonly AnimSpec[] = ["solid", "blink", "pulse"];
const FOUR: readonly FourShow[] = ["time", "startwatch", "timer"];
const SIX: readonly SixShow[] = ["set", "reset", "count_up", "count_down"];

function LedEditor({ a, onChange }: { a: Led; onChange: (a: ActionSpec) => void }) {
  const anim = a.anim ?? "solid";
  return (
    <>
      <NumberField label="First LED" value={a.start} onChange={(v) => onChange({ ...a, start: v ?? 0 })} />
      <NumberField label="End LED (exclusive, max 46)" value={a.end} onChange={(v) => onChange({ ...a, end: v ?? 0 })} />
      <Field label="Colour (RRGGBB)">
        <span className="row">
          <input
            type="text"
            className="mono"
            maxLength={6}
            value={a.color}
            onChange={(e) => onChange({ ...a, color: e.target.value })}
          />
          <input
            type="color"
            aria-label="Pick colour"
            // The picker works in #rrggbb; fall back to black while the text is not a colour.
            value={/^[0-9a-fA-F]{6}$/.test(a.color) ? `#${a.color}` : "#000000"}
            onChange={(e) => onChange({ ...a, color: e.target.value.slice(1).toUpperCase() })}
          />
        </span>
      </Field>
      <Choice
        label="Animation"
        value={anim}
        options={ANIMS}
        // A solid LED takes no period, so drop it when switching to solid.
        onChange={(v) => onChange({ ...a, anim: v, period_ms: v === "solid" ? null : a.period_ms })}
      />
      {anim !== "solid" && (
        <NumberField
          label="Period ms (empty = default)"
          optional
          value={a.period_ms}
          onChange={(v) => onChange({ ...a, period_ms: v })}
        />
      )}
    </>
  );
}

function LcdEditor(props: { a: Lcd; fields: string[]; onChange: (a: ActionSpec) => void }) {
  const { a, fields, onChange } = props;
  return (
    <>
      <Choice
        label="Row"
        value={a.row === 2 ? "2" : "1"}
        options={["1", "2"] as const}
        onChange={(v) => onChange({ ...a, row: Number(v) })}
      />
      <Field label="Text (16 characters at most once filled in)">
        <input type="text" value={a.text} onChange={(e) => onChange({ ...a, text: e.target.value })} />
      </Field>
      <div className="chips" aria-label="Insert a field">
        {fields.map((f) => (
          <button key={f} type="button" onClick={() => onChange({ ...a, text: `${a.text}{${f}}` })}>
            {`{${f}}`}
          </button>
        ))}
      </div>
    </>
  );
}

function FourEditor({ a, onChange }: { a: Four; onChange: (a: ActionSpec) => void }) {
  return (
    <>
      <Choice
        label="Mode"
        value={a.show}
        options={FOUR}
        // The modes take different arguments, so start clean to avoid a conflicting mix.
        onChange={(show) => onChange({ output: "four_digit", show, at: null, seconds: null, pause: false })}
      />
      {a.show === "time" ? (
        <Field label="Time HH:MM (empty = resume)">
          <input
            type="text"
            className="mono"
            value={a.at ?? ""}
            placeholder="12:34"
            onChange={(e) => onChange({ ...a, at: e.target.value === "" ? null : e.target.value })}
          />
        </Field>
      ) : (
        <>
          <NumberField
            label="Start at seconds (empty = resume)"
            optional
            value={a.seconds}
            onChange={(v) => onChange({ ...a, seconds: v, pause: v === null ? a.pause : false })}
          />
          <label className="check">
            <input
              type="checkbox"
              checked={a.pause ?? false}
              disabled={a.seconds !== null}
              onChange={(e) => onChange({ ...a, pause: e.target.checked })}
            />
            Pause
          </label>
        </>
      )}
    </>
  );
}

function SixEditor({ a, onChange }: { a: Six; onChange: (a: ActionSpec) => void }) {
  const counts = a.show === "count_up" || a.show === "count_down";
  return (
    <>
      <Choice
        label="Mode"
        value={a.show}
        options={SIX}
        onChange={(show) =>
          onChange({ output: "six_digit", show, value: show === "reset" ? null : (a.value ?? 0), period_ms: null })
        }
      />
      {a.show !== "reset" && (
        <NumberField label="Value (0-999999)" value={a.value} onChange={(v) => onChange({ ...a, value: v ?? 0 })} />
      )}
      {counts && (
        <NumberField
          label="Step ms (empty = default)"
          optional
          value={a.period_ms}
          onChange={(v) => onChange({ ...a, period_ms: v })}
        />
      )}
    </>
  );
}

/** Editor for one action; which fields show depends on the output type. */
export function ActionEditor(props: {
  action: ActionSpec;
  fields: string[];
  onChange: (a: ActionSpec) => void;
}) {
  const { action, fields, onChange } = props;
  return (
    <>
      <Choice
        label="Output"
        value={action.output}
        options={OUTPUTS}
        onChange={(o) => o !== action.output && onChange(defaultAction(o))}
      />
      {action.output === "led" && <LedEditor a={action} onChange={onChange} />}
      {action.output === "lcd" && <LcdEditor a={action} fields={fields} onChange={onChange} />}
      {action.output === "four_digit" && <FourEditor a={action} onChange={onChange} />}
      {action.output === "six_digit" && <SixEditor a={action} onChange={onChange} />}
    </>
  );
}
