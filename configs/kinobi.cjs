const path = require("path");
const k = require("@metaplex-foundation/kinobi");
const fs = require("fs");
// Paths.
const clientDir = path.join(__dirname, "..", "clients");
const idlDir = path.join(__dirname, "..", "idls");

// Instanciate Kinobi.
const kinobi = k.createFromIdls([
  path.join(idlDir, "lmaofun_bonding_curve.json"),
]);

kinobi.update(
  new k.updateProgramsVisitor({
    lmaofunBonding: { name: "lmaofunBondingCurve", prefix: "lbc" },
  })
);

// Update accounts.
kinobi.update(
  new k.updateAccountsVisitor({
    global: {
      seeds: [k.constantPdaSeedNodeFromString("global")],
    },
    eventAuthority: {
      seeds: [k.constantPdaSeedNodeFromString("__event_authority")],
    },
  })
);

// Render JavaScript.
const jsDir = path.join(clientDir, "js", "src", "generated");
const prettier = require(path.join(clientDir, "js", ".prettierrc.json"));
kinobi.accept(
  new k.renderJavaScriptVisitor(jsDir, {
    prettierOptions: prettier,
    exportAccounts: true,
  })
);

// cp idls dir in clients/js/src/idls
const idlsTargetDir = path.join(clientDir, "js", "src", "idls");
fs.cpSync(idlDir, idlsTargetDir, { recursive: true });
// cp target/types in clients/js/src/idls
fs.cpSync(path.join(__dirname, "..", "target", "types"), idlsTargetDir, { recursive: true });

// Render Rust.
const crateDir = path.join(clientDir, "rust");
const rustDir = path.join(clientDir, "rust", "src", "generated");
kinobi.accept(
  new k.renderRustVisitor(rustDir, {
    formatCode: true,
    crateFolder: crateDir,
  })
);
