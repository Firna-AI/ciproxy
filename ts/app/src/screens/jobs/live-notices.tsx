// The notices of the live queue, from the model. A page of the app opens
// through the router, a link out of the app opens a new tab, and a caller who
// may not act reads who can.

import { Link } from "expo-router";

import { ActionLink } from "../../components/actions.tsx";
import { ExternalIcon } from "../../components/icons.tsx";
import { Notice } from "../../components/notice.tsx";
import { RoleNote } from "../../components/role-note.tsx";
import type { LiveNotice, NoticeAction } from "../../view/jobs-notices.ts";

function Action({ action }: { action: NoticeAction }) {
  switch (action.kind) {
    case "go":
      return (
        <Link href={action.href} asChild>
          <ActionLink label={action.label} href={action.href} size="sm" />
        </Link>
      );
    case "leave":
      return (
        <ActionLink
          label={action.label}
          href={action.href}
          size="sm"
          icon={ExternalIcon}
          newTab
        />
      );
    case "ask":
      return <RoleNote>{action.text}</RoleNote>;
  }
}

export function LiveNotices({ notices }: { notices: readonly LiveNotice[] }) {
  return (
    <>
      {notices.map((notice) => (
        <Notice
          key={notice.key}
          tone={notice.tone}
          title={notice.title}
          actions={notice.action ? <Action action={notice.action} /> : null}
        >
          {notice.body}
        </Notice>
      ))}
    </>
  );
}
