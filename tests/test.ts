import "dotenv/config";
import { hasCompletedChallenge } from "../backend/checkChallenge";
import { signClaim } from "../backend/signClaim";

async function run() {
  const userId = "user_01";
  const challengeId = "challenge_01";

  try {
    const completed = await hasCompletedChallenge(userId, challengeId);

    if (!completed) {
      console.log("REJECTED");
      return;
    }

    const result = signClaim(userId);
    console.log("APPROVED:");
    console.log(result);
  } catch (err: any) {
    console.log("ERROR:");
    console.log(err.message);
  }
}

run()
  .then(() => {
    process.exit(0);
  })
  .catch((err) => {
    console.error(err);
    process.exit(1);
  });