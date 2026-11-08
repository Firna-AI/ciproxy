// A path that the dashboard does not have.

import { Link } from "expo-router";

import { ActionLink } from "../components/actions.tsx";
import { AuthPage } from "../components/auth-page.tsx";
import { pages } from "../routing/pages.ts";

export default function NotFound() {
  return (
    <AuthPage
      model={{
        title: "Page not found",
        lead: "The dashboard has no page at this address.",
        notice: null,
        list: null,
        actions: [],
        footer: "",
      }}
    >
      <Link href={pages.home} asChild>
        <ActionLink
          label="Open the dashboard"
          href={pages.home}
          tone="primary"
          size="lg"
          block
        />
      </Link>
    </AuthPage>
  );
}
