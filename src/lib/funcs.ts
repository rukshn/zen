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

export const connectImapMail = async () => {
  try {
    const tools = await invoke<unknown[]>("mcp_connect", {
      server: "imap-mail",
      clientId: "",
      clientSecret: "",
    });
    return tools.length > 0;
  } catch (e) {
    console.log(e);
    return false;
  }
};

export const openImapSetupWizard = async (): Promise<void> => {
  try {
    await invoke("imap_open_setup_wizard");
  } catch (e) {
    console.log(e);
    throw e;
  }
};

type ImapToolResult = {
  is_error: boolean;
  text: string;
  structured: unknown;
};

const imapAccountName = (account: unknown): string => {
  if (typeof account === "string") return account;
  if (!account || typeof account !== "object") return "";
  const a = account as Record<string, unknown>;
  return String(a.name ?? a.email ?? a.user ?? a.accountName ?? a.id ?? "");
};

const parseImapAccounts = (res: ImapToolResult): string[] => {
  const structured = res.structured as unknown;
  const arr = Array.isArray(structured)
    ? structured
    : ((structured as { accounts?: unknown[] } | null)?.accounts ?? null);

  if (Array.isArray(arr)) {
    return arr.map(imapAccountName).filter(Boolean);
  }

  try {
    const parsed: unknown = JSON.parse(res.text);
    const parsedArr = Array.isArray(parsed)
      ? parsed
      : ((parsed as { accounts?: unknown[] } | null)?.accounts ?? null);
    if (Array.isArray(parsedArr)) {
      return parsedArr.map(imapAccountName).filter(Boolean);
    }
  } catch {
    // not JSON, fall through to plain text
  }

  return res.text ? [res.text] : [];
};

export const checkImapConnection = async (): Promise<string[]> => {
  try {
    await invoke("mcp_connect", {
      server: "imap-mail",
      clientId: "",
      clientSecret: "",
    });
    const accounts = await listImapAccounts();
    if (accounts.length > 0) return accounts;

    await invoke("mcp_disconnect", { server: "imap-mail" }).catch(() => {});
    await invoke("mcp_connect", {
      server: "imap-mail",
      clientId: "",
      clientSecret: "",
    });
    return listImapAccounts();
  } catch (e) {
    console.log(e);
    return [];
  }
};

export const disconnectImap = async (): Promise<void> => {
  try {
    await invoke("mcp_disconnect", { server: "imap-mail" });
  } catch (e) {
    console.log(e);
    throw e;
  }
};

export const refreshConnections = async () => {
  await invoke("mcp_disconnect", { server: "imap-mail" }).catch(() => {});
  await invoke("mcp_connect", {
    server: "imap-mail",
    clientId: "",
    clientSecret: "",
  });

  return listImapAccounts();
};

const listImapAccounts = async (): Promise<string[]> => {
  const res = await invoke<ImapToolResult>("mcp_call_tool", {
    server: "imap-mail",
    tool: "imap_list_accounts",
    arguments: {},
  });
  if (res.is_error) return [];

  const parse = parseImapAccounts(res);
  return parse;
};
