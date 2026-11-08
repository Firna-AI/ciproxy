const fs = require("node:fs");

const { parse } = require("../../ts/node_modules/yaml");

const documents = Object.fromEntries(
  process.argv
    .slice(2)
    .map((path) => [path, parse(fs.readFileSync(path, "utf8"))]),
);
process.stdout.write(JSON.stringify(documents));
