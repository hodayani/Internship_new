import { db } from "./firebase";
import {
  collection,
  query,
  where,
  getDocs,
  updateDoc,
} from "firebase/firestore";

export async function hasCompletedChallenge(
  userId: string,
  challengeId: string
): Promise<boolean> {
  const q = query(
    collection(db, "Users and challenges"),
    where("user_id", "==", userId),
    where("challenge_id", "==", challengeId)
  );

  const snap = await getDocs(q);

  if (snap.empty) return false;

  const docRef = snap.docs[0].ref;
  const data = snap.docs[0].data();

  if (
    data.Status === "Done" &&
    data.Received_the_reward === "no"
  ) {
    // עדכון בפיירבייס
    await updateDoc(docRef, {
      Received_the_reward: "yes",
    });

    return true;
  }

  return false;
}