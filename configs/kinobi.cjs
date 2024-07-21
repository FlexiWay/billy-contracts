const path = require("path");
const k = require("@metaplex-foundation/kinobi");

// Paths.
const clientDir = path.join(__dirname, "..", "clients");
const idlDir = path.join(__dirname, "..", "idls");

// Instanciate Kinobi.
const kinobi = k.createFromIdls([path.join(idlDir, "bonding_curve.json")]);

kinobi.update(
  new k.updateProgramsVisitor({
    lmaofunBonding: { name: "lmaofunBondingCurve", prefix: "lbc" },
  })
);

// Update accounts.
kinobi.update(
  new k.updateAccountsVisitor({
    testState: {
      seeds: [k.constantPdaSeedNodeFromString("test-state")],
    },
    // tokenManager: {
    //   seeds: [
    //     k.constantPdaSeedNodeFro3mString("token-manager")
    //   ],
    // },
    // poolManager: {
    //   seeds: [
    //     k.constantPdaSeedNodeFromString("pool-manager")
    //   ],
    // },
    // gatekeeper: {
    //   seeds: [
    //     k.constantPdaSeedNodeFromString("gatekeeper"),
    //     k.variablePdaSeedNode(
    //       "wallet",
    //       k.publicKeyTypeNode(),
    //       "The address of the gate_keeper wallet"
    //     ),
    //   ],
    // },
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

// Render Rust.
const crateDir = path.join(clientDir, "rust");
const rustDir = path.join(clientDir, "rust", "src", "generated");
kinobi.accept(
  new k.renderRustVisitor(rustDir, {
    formatCode: true,
    crateFolder: crateDir,
  })
);
