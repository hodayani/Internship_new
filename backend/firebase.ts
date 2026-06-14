import { initializeApp } from "firebase/app";
import { getFirestore } from "firebase/firestore";

const firebaseConfig = {
  apiKey: "AIzaSyA1loorLXecGf38PdC2ans8Ziv1lM7ab3c",
  authDomain: "users-and-challenges.firebaseapp.com",
  projectId: "users-and-challenges",
  storageBucket: "users-and-challenges.firebasestorage.app",
  messagingSenderId: "557197560946",
  appId: "1:557197560946:web:243fd17d24ca41596b06eb",
  measurementId: "G-20HRSYFNSG"
};

const app = initializeApp(firebaseConfig);

export const db = getFirestore(app);