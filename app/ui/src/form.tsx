import type { ReactNode } from "react";

/** A label tied to its control, so screen readers announce it. */
export function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="field">
      <span>{label}</span>
      {children}
    </label>
  );
}

/**
 * A whole-number input. Empty means `null` when optional, 0 when not.
 * Negative values are clamped to 0 unless `signed`, because the backend takes unsigned numbers.
 */
export function NumberField(props: {
  label: string;
  value: number | null;
  onChange: (value: number | null) => void;
  optional?: boolean;
  signed?: boolean;
}) {
  const { label, value, onChange, optional = false, signed = false } = props;
  return (
    <Field label={label}>
      <input
        type="number"
        step={1}
        min={signed ? undefined : 0}
        value={value ?? ""}
        onChange={(e) => {
          const n = e.target.valueAsNumber;
          if (Number.isNaN(n)) return onChange(optional ? null : 0);
          const whole = Math.trunc(n);
          onChange(signed ? whole : Math.max(0, whole));
        }}
      />
    </Field>
  );
}

/** A dropdown over a fixed list of string values. */
export function Choice<T extends string>(props: {
  label: string;
  value: T;
  options: readonly T[];
  onChange: (value: T) => void;
}) {
  const { label, value, options, onChange } = props;
  return (
    <Field label={label}>
      <select value={value} onChange={(e) => onChange(e.target.value as T)}>
        {options.map((o) => (
          <option key={o} value={o}>
            {o.replaceAll("_", " ")}
          </option>
        ))}
      </select>
    </Field>
  );
}
