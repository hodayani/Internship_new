import nacl from "tweetnacl";
import bs58 from "bs58";
import crypto from "crypto";

const backendPrivateKey = bs58.decode(
  process.env.BACKEND_PRIVATE_KEY!
);

export function signClaim(recipient: string) {
  const payload = {
    recipient,
    amount: 50,
    nonce: crypto.randomUUID(),
    expires_at: Math.floor(Date.now() / 1000) + 300,
    challenge_id: "test-challenge",
    timestamp: Math.floor(Date.now() / 1000),
  };

  const payloadBytes = Buffer.from(JSON.stringify(payload));

  const signature = nacl.sign.detached(
    payloadBytes,
    backendPrivateKey
  );

  return {
    payload,
    signature: bs58.encode(signature),
    backendPubkey: process.env.BACKEND_PUBLIC_KEY!,
  };
}