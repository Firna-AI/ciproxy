import { defineUseCase } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { relatedMvp } from "../../components/related-docs.ts";

export const newMachineGetsImage = defineUseCase({
  title: "A new machine gets its job image",
  description:
    "On the busy afternoon of 18 November, Cal Moore adds a fourth AX42, hetzner-ax42-4. The machine connects at once, and then it gets its first job image. Every page that shows it says that it takes no job until the image is ready, and the other machines and Boat run the jobs meanwhile.",
  relatedDocs: relatedMvp("agent", "jobRouting", "metering"),
  steps: [
    {
      screenPath: paths.addMachine,
      title: "Copy the install command",
      description:
        "Cal opens Add a machine and runs the one command on hetzner-ax42-4. The page waits for the machine.",
    },
    {
      screenPath: paths.addMachinePreparing,
      title: "The machine connects",
      description:
        "The agent reports 16 threads, 64 GB, KVM, and 6 slots. The machine gets its first job image, and jobs start on it when it is ready.",
    },
    {
      screenPath: paths.machinesPreparing,
      title: "Machines shows Preparing",
      description:
        "The row of hetzner-ax42-4 says Preparing and Getting the job image. Slots in use counts only the slots of the other three machines.",
    },
    {
      screenPath: paths.machinePreparing,
      title: "The machine page says why",
      description:
        "The notice says that jobs start on hetzner-ax42-4 when it is ready, and that the other machines and Boat run them until then.",
    },
  ],
});
