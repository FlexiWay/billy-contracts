const path = require("path");
const { generateIdl } = require("@metaplex-foundation/shank-js");

const idlDir = path.join(__dirname, "..", "idls");
const binaryInstallDir = path.join(__dirname, "..", ".crates");
const programDir = path.join(__dirname, "..", "programs");

generateIdl({
  generator: "anchor",
  programName: "billy_bonding_curve",
  programId: "71odFTZ59cG8yyBtEZrnJdBYaepzri2A12hEc16vK6WP",
  idlDir,
  binaryInstallDir,
  programDir: path.join(programDir, "billy-bonding-curve"),
  rustbin: { locked: true },
});
