import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { CreatorPayer } from "../target/types/creator_payer";

describe("creator-payer", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.CreatorPayer as Program<CreatorPayer>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
