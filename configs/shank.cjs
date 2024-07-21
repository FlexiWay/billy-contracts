const path = require("path");
const { generateIdl } = require("@metaplex-foundation/shank-js");

const idlDir = path.join(__dirname, "..", "idls");
const binaryInstallDir = path.join(__dirname, "..", ".crates");
const programDir = path.join(__dirname, "..", "programs");

generateIdl({
  generator: "anchor",
  programName: "bonding_curve",
  programId: "E52KjA58odp3taqmaCuBFdDya3s4TA1ho4tSXoW2igxb",
  idlDir,
  binaryInstallDir,
  programDir: path.join(programDir, "lmaofun-bonding-curve"),
  rustbin: { locked: true },
});
