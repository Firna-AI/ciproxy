import { defineUseCase } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { related } from "../../components/related-docs.ts";

export const adminChangesBoatKey = defineUseCase({
  title: "Change the Boat key, then disconnect Boat",
  description:
    "Cal Moore replaces the Boat key of mokly-ai, and later disconnects Boat. Each connection asks for the spend limit, and running Boat jobs always finish.",
  relatedDocs: related("overflow", "security"),
  steps: [
    {
      screenPath: paths.overflow,
      title: "The connected account",
      description:
        "Settings under Overflow names the Boat account mokly, with Change key and Disconnect.",
    },
    {
      screenPath: paths.overflowChangeKey,
      title: "Change key",
      description:
        "The new connection asks for the new key and the limit, which starts with the saved $200. Running Boat jobs finish with the current key.",
    },
    {
      screenPath: paths.overflowDisconnect,
      title: "Disconnect asks first",
      description:
        "ciproxy starts no new jobs on Boat. The jobs that run on Boat finish, and ciproxy then deletes the key.",
    },
    {
      screenPath: paths.overflowNotConnected,
      title: "Boat is not connected",
      description:
        "Jobs run on the own machines only, and Connect Boat asks for a key and a limit again.",
    },
  ],
});
