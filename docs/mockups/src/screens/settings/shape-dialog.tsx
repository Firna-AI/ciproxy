// Edit a label and its shape, or add a label whose shape fits no own machine.

import { Switch } from "@firna/ui/switch";

import { LinkAction, noop } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { TextField } from "../../components/fields.tsx";
import { paths } from "../../components/paths.ts";
import { Dialog } from "./settings-frame.tsx";

function Toggle({
  label,
  caption,
  value,
  disabled,
}: {
  label: string;
  caption: string;
  value: boolean;
  disabled?: boolean;
}) {
  return (
    <div className="toggle-row">
      <span className="stack-tight">
        <span className="strong">{label}</span>
        <span className="caption">{caption}</span>
      </span>
      <Switch
        value={value}
        onValueChange={disabled ? undefined : noop}
        disabled={disabled}
        accessibilityLabel={label}
      />
    </div>
  );
}

export function ShapeDialog({ tooLarge }: { tooLarge: boolean }) {
  return (
    <Dialog
      title={tooLarge ? "Add a label" : "Edit ciproxy-8vcpu"}
      footer={
        <>
          <LinkAction to={paths.settings} tone="ghost">
            Cancel
          </LinkAction>
          <LinkAction to={paths.settings} tone="primary">
            {tooLarge ? "Add label" : "Save label"}
          </LinkAction>
        </>
      }
    >
      <TextField
        label="Label"
        value={tooLarge ? "ciproxy-32vcpu" : "ciproxy-8vcpu"}
      />
      <div className="field-row">
        <TextField label="vCPU" value={tooLarge ? "32" : "8"} />
        <TextField label="Memory, GB" value={tooLarge ? "128" : "32"} />
        <TextField label="Disk, GB" value={tooLarge ? "200" : "120"} />
      </div>
      {tooLarge ? (
        <Notice tone="warning" title="No machine has 32 vCPU free for jobs">
          Jobs with this label run on Boat. Your machines offer 12 vCPU each.
        </Notice>
      ) : (
        <p className="caption">
          Each of your machines runs 1 job of this shape at a time.
        </p>
      )}
      <Toggle
        label="Your machines"
        caption="Jobs start here first"
        value={!tooLarge}
        disabled={tooLarge}
      />
      <Toggle
        label="Boat"
        caption={
          tooLarge ? "Every job with this label" : "When your machines are busy"
        }
        value
      />
    </Dialog>
  );
}
