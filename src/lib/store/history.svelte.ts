import { getDb } from "./dbstore";
import type { ChatMessage } from "./message.svelte";

export const saveConversation = async (messages: ChatMessage[]) => {
  try {
    const db = await getDb();
    const uuid = crypto.randomUUID();
    await db.execute(
      "INSERT INTO CONVERSATIONS (uuid, messages, base) VALUES ($1, $2, $3)",
      [uuid, JSON.stringify(messages), 1],
    );
  } catch (e) {
    console.log("error saving conversation", e);
  }
};
