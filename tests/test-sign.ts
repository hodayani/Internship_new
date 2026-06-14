import "dotenv/config";

import { signClaim } from "../backend/signClaim";
import nacl from "tweetnacl";
import bs58 from "bs58";

console.log("PRIVATE KEY:", process.env.BACKEND_PRIVATE_KEY);
console.log("PUBLIC KEY:", process.env.BACKEND_PUBLIC_KEY);

const user = "test-wallet";

const { payload, signature, backendPubkey } = signClaim(user);

const valid = nacl.sign.detached.verify(
  Buffer.from(JSON.stringify(payload)),
  bs58.decode(signature),
  bs58.decode(backendPubkey)
);

console.log("VALID:", valid);