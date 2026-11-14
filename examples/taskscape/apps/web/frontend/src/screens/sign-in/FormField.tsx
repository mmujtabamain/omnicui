// One labelled field of the sign-in form: a visible label, the well with the text field, and the
// server's message for this field when it refused one.
import * as React from "react";
import { Input } from "@taskscape/ui/input";
import { Text } from "@taskscape/ui/text";

export function FormField({
  label,
  ariaLabel,
  part,
  errorPart,
  value,
  onChange,
  onSubmit,
  error,
  masked = false,
  spellCheck = true,
}: {
  label: string;
  ariaLabel: string;
  part: string;
  errorPart: string;
  value: string;
  onChange: (value: string) => void;
  onSubmit: () => void;
  error: string;
  /** Hides the typed characters. The field stays a text box: the Input widget has no password type. */
  masked?: boolean;
  spellCheck?: boolean;
}) {
  return (
    <div className="flex flex-col gap-space-3">
      <Text as="span" size="sm" weight="medium" tone="secondary">
        {label}
      </Text>
      <div className="flex h-[2.5rem] items-center rounded-(--radius-field) border border-edge-2 bg-surface-1 px-space-4 outline-none focus-within:ring-2 focus-within:ring-focus-1">
        <Input
          data-part={part}
          mode="bare"
          size="md"
          ariaLabel={ariaLabel}
          value={value}
          onValueChange={onChange}
          onSubmit={() => onSubmit()}
          spellCheck={spellCheck}
          className={masked ? "focus:ring-0 [-webkit-text-security:disc]" : "focus:ring-0"}
        />
      </div>
      {error !== "" && (
        <Text as="p" role="status" data-part={errorPart} size="sm" tone="danger">
          {error}
        </Text>
      )}
    </div>
  );
}
