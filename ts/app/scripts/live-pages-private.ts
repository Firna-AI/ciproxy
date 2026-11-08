import { constants } from "node:fs";
import { open, readFile, realpath } from "node:fs/promises";
import { dirname, isAbsolute, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const repository = fileURLToPath(new URL("../../../", import.meta.url));

async function outsideRepository(path: string): Promise<string> {
  const difference = relative(await realpath(repository), path);
  if (
    difference === "" ||
    (!isAbsolute(difference) &&
      difference !== ".." &&
      !difference.startsWith(`..${sep}`))
  ) {
    throw new Error("A private file must be outside the repository.");
  }
  return path;
}

export async function privateInputPath(path: string): Promise<string> {
  return outsideRepository(await realpath(resolve(path)));
}

export async function privateOutputPath(path: string): Promise<string> {
  const absolute = resolve(path);
  const parent = await realpath(dirname(absolute));
  return outsideRepository(
    resolve(parent, relative(dirname(absolute), absolute)),
  );
}

export async function readPrivateJson(path: string): Promise<unknown> {
  return JSON.parse(await readFile(await privateInputPath(path), "utf8"));
}

export async function writePrivateJson(
  path: string,
  value: unknown,
  create: boolean,
): Promise<void> {
  const target = await privateOutputPath(path);
  const flags =
    constants.O_WRONLY |
    constants.O_CREAT |
    constants.O_NOFOLLOW |
    (create ? constants.O_EXCL : constants.O_TRUNC);
  const file = await open(target, flags, 0o600);
  try {
    await file.chmod(0o600);
    await file.writeFile(JSON.stringify(value, null, 2) + "\n", "utf8");
  } finally {
    await file.close();
  }
}
