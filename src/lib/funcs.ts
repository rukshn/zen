import { invoke } from "@tauri-apps/api/core";
import Database from "@tauri-apps/plugin-sql";

const DB_PATH = "sqlite:settings.db";

export const checkCalendarConnection = async () => {
  try {
    const db = await Database.load(DB_PATH);
    const rows = await db.select<{ key: string; value: string }[]>(
      "SELECT key, value FROM settings WHERE key IN ($1, $2, $3)",
      ["google_client_id", "google_client_secret"],
    );

    let googleClientId: string | undefined = undefined;
    let googleClientSecret: string | undefined = undefined;

    for (const row of rows) {
      if (row.key === "google_client_id") googleClientId = row.value;
      if (row.key === "google_client_secret") googleClientSecret = row.value;
    }

    if (!googleClientId || !googleClientSecret) return;

    const connection: { name: string; description: string }[] = await invoke(
      "mcp_connect",
      {
        server: "google-calendar",
        clientId: googleClientId,
        clientSecret: googleClientSecret,
      },
    );
    if (connection.length > 0) {
      return true;
    }
  } catch (e) {
    console.log(e);
    return false;
  }
  return false;
};

export const getAPIKey = async () => {
  const db = await Database.load(DB_PATH);
  const rows = await db.select<{ key: string; value: string }[]>(
    "SELECT key, value FROM settings WHERE key IN ($1)",
    ["deepseek_api_key"],
  );

  if (rows) {
    const apiKey = rows[0].value;
    return apiKey;
  }
  return undefined;
};
