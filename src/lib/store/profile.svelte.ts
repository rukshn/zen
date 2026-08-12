import { getDb } from "./dbstore";

export const profile = $state({avatar: ""})

const AVATAR_KEY = "user_avatar"

export const loadProfile = async () => {
  try {
    const db = await getDb()
    await db.execute(
      "CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
    );
    const rows = await db.select<{ key: string; value: string }[]>(
      "SELECT key, value FROM settings WHERE key IN ($1)",
      [AVATAR_KEY],
    );
    if (rows[0]) profile.avatar = rows[0].value;
  } catch (e) {
    console.error("loadProfile failed:", e);
  }
};
