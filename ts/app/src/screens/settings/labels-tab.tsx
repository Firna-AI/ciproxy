// The Labels tab: the labels of the workspace with their sizes and where
// their jobs run, Add label, and Edit for each label. A workflow picks a
// label in runs-on, and each label starts a fresh VM of its size.

import { useState, type ReactNode } from "react";
import { StyleSheet, Text } from "react-native";

import { ActionButton } from "../../components/actions.tsx";
import { Panel } from "../../components/feature-parts.tsx";
import { PlusIcon } from "../../components/icons.tsx";
import { text } from "../../components/text.ts";
import { useViewport } from "../../components/viewport.ts";
import { useServices } from "../../services/context.tsx";
import { useValue } from "../../services/hooks.ts";
import { settingsServices } from "../../state/settings-services.ts";
import { labelRows } from "../../view/settings-labels.ts";
import type { WorkspaceContext } from "../workspace-page.tsx";
import { LabelDialog, type LabelTarget } from "./label-dialog.tsx";
import { LabelList } from "./label-list.tsx";
import { SettingsRead } from "./settings-read.tsx";

type Opened = { kind: "add" } | { kind: "edit"; labelId: string };

function LabelsPanel({
  children,
  onAdd,
}: {
  children: ReactNode;
  onAdd?: () => void;
}) {
  return (
    <Panel
      title="Labels"
      flush
      action={
        onAdd ? (
          <ActionButton
            label="Add label"
            size="sm"
            icon={PlusIcon}
            onPress={onAdd}
          />
        ) : undefined
      }
    >
      {children}
    </Panel>
  );
}

export function LabelsTab({ context }: { context: WorkspaceContext }) {
  const store = settingsServices(useServices()).settings(context.workspace.id);
  const snapshot = useValue(store.state);
  const phone = useViewport() === "mobile";
  const [opened, setOpened] = useState<Opened | null>(null);
  return (
    <>
      <Text style={[text.small, styles.intro]}>
        A workflow picks a label in <Text style={text.mono}>runs-on</Text>. Each
        label starts a fresh VM of its size, on your machines first and on Boat
        when they are busy.
      </Text>
      <SettingsRead
        snapshot={snapshot}
        subject="settings"
        onRetry={() => void store.load()}
        loading={
          <LabelsPanel>
            <LabelList
              rows={[]}
              phone={phone}
              loading
              onEdit={() => undefined}
            />
          </LabelsPanel>
        }
      >
        {(settings) => {
          const edited =
            opened?.kind === "edit"
              ? settings.labels.find((label) => label.id === opened.labelId)
              : undefined;
          const target: LabelTarget | null =
            opened?.kind === "add"
              ? { kind: "add" }
              : edited
                ? { kind: "edit", label: edited }
                : null;
          return (
            <>
              <LabelsPanel onAdd={() => setOpened({ kind: "add" })}>
                <LabelList
                  rows={labelRows(settings.labels)}
                  phone={phone}
                  onEdit={(labelId) => setOpened({ kind: "edit", labelId })}
                />
              </LabelsPanel>
              <Text style={text.caption}>
                A slot on your machines is 2 vCPU and 8 GB. A larger size takes
                several slots. Each running job also holds disk for the image
                and its free disk.
              </Text>
              {target ? (
                <LabelDialog
                  key={opened?.kind === "edit" ? opened.labelId : "add"}
                  store={store}
                  target={target}
                  labels={settings.labels}
                  onClose={() => setOpened(null)}
                />
              ) : null}
            </>
          );
        }}
      </SettingsRead>
    </>
  );
}

const styles = StyleSheet.create({ intro: { maxWidth: 640 } });
