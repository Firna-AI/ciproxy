import assert from "node:assert/strict";
import test from "node:test";

import { createBuildGate } from "../src/state/build-gate.ts";
import {
  appLinkHref,
  createNavigator,
  installNavigationGate,
  type ClickEvent,
  type GateDocument,
  type GateWindow,
  type LinkElement,
} from "../src/state/navigation.ts";

function environment() {
  const actions: string[] = [];
  let click: ((event: ClickEvent & { target: unknown }) => void) | null = null;
  let history: ((event: { stopImmediatePropagation(): void }) => void) | null =
    null;
  const win: GateWindow = {
    location: {
      origin: "https://dashboard.example",
      assign: (href) => {
        actions.push(`document:${href}`);
      },
      replace: (href) => {
        actions.push(`replace-document:${href}`);
      },
      reload: () => {
        actions.push("reload");
      },
    },
    addEventListener: (_type, listener) => {
      history = listener;
    },
    removeEventListener: () => {
      history = null;
    },
  };
  const doc: GateDocument = {
    addEventListener: (_type, listener) => {
      click = listener;
    },
    removeEventListener: () => {
      click = null;
    },
  };
  const event: ClickEvent & { target: unknown } = {
    button: 0,
    metaKey: false,
    ctrlKey: false,
    shiftKey: false,
    altKey: false,
    defaultPrevented: false,
    target: null,
    preventDefault: () => {
      actions.push("prevent-default");
    },
    stopPropagation: () => {
      actions.push("stop-click");
    },
  };
  return {
    actions,
    win,
    doc,
    event,
    click: () => {
      assert.notEqual(click, null);
      click?.(event);
    },
    history: () => {
      assert.notEqual(history, null);
      history?.({
        stopImmediatePropagation: () => {
          actions.push("stop-history");
        },
      });
    },
  };
}

test("a build change preserves the page until programmatic navigation replaces the document", () => {
  for (const replace of [false, true]) {
    const gate = createBuildGate();
    const env = environment();
    const navigator = createNavigator(gate, env.win.location, {
      push: (href) => {
        env.actions.push(`router:${href}`);
      },
      replace: (href) => {
        env.actions.push(`replace-router:${href}`);
      },
    });
    gate.observe(null);
    gate.observe(" ");
    gate.observe("a");
    navigator.go("/jobs?workspace=one", { replace });
    gate.observe("b");
    gate.observe("a");
    assert.equal(gate.reloading(), false);
    assert.deepEqual(env.actions, [
      replace
        ? "replace-router:/jobs?workspace=one"
        : "router:/jobs?workspace=one",
    ]);
    navigator.go("/machines?workspace=one", { replace });
    assert.equal(gate.reloading(), true);
    assert.equal(
      env.actions[1],
      replace
        ? "replace-document:/machines?workspace=one"
        : "document:/machines?workspace=one",
    );
  }
  const nextDocument = createBuildGate();
  nextDocument.observe("b");
  assert.equal(nextDocument.changed(), false);
});

test("plain app links and history reload before the router sees a changed build", () => {
  const link: LinkElement = {
    href: "https://dashboard.example/machines?workspace=one#ready",
    target: "",
    hasAttribute: () => false,
  };
  for (const kind of ["click", "history"] as const) {
    const gate = createBuildGate();
    const env = environment();
    const remove = installNavigationGate(gate, env.win, env.doc, () => link);
    env[kind]();
    assert.deepEqual(env.actions, []);
    gate.observe("a");
    gate.observe("b");
    env[kind]();
    assert.deepEqual(
      env.actions,
      kind === "click"
        ? [
            "prevent-default",
            "stop-click",
            "document:/machines?workspace=one#ready",
          ]
        : ["stop-history", "reload"],
    );
    assert.equal(gate.reloading(), true);
    remove();
  }
});

test("the app leaves modified, download, external, and auth links to the browser", () => {
  const env = environment();
  const link: LinkElement = {
    href: "/jobs?workspace=one",
    target: "",
    hasAttribute: () => false,
  };
  assert.equal(
    appLinkHref(link, env.event, env.win.location.origin),
    "/jobs?workspace=one",
  );
  assert.equal(
    appLinkHref(link, { ...env.event, ctrlKey: true }, env.win.location.origin),
    null,
  );
  assert.equal(
    appLinkHref(
      { ...link, hasAttribute: () => true },
      env.event,
      env.win.location.origin,
    ),
    null,
  );
  assert.equal(
    appLinkHref(
      { ...link, target: "_blank" },
      env.event,
      env.win.location.origin,
    ),
    null,
  );
  assert.equal(
    appLinkHref(
      { ...link, href: "https://github.com/apps/ciproxy" },
      env.event,
      env.win.location.origin,
    ),
    null,
  );
  assert.equal(
    appLinkHref(
      { ...link, href: "/api/auth/github" },
      env.event,
      env.win.location.origin,
    ),
    null,
  );
});
