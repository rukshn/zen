<script lang="ts">
  import * as Breadcrumb from "$lib/components/ui/breadcrumb/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import { Separator } from "$lib/components/ui/separator/index.js";
  import SidebarLeft from "$lib/components/sidebar-left.svelte";
  import SidebarRight from "$lib/components/sidebar-right.svelte";
  import Database from "@tauri-apps/plugin-sql";
  import CalendarFilledIcon from "@iconify-svelte/tabler/calendar-filled";
  import { invoke } from "@tauri-apps/api/core";
  import { checkCalendarConnection } from "@/funcs";

  const DB_PATH = "sqlite:settings.db";
  const KEY = "deepseek_api_key";
  const GOOGLE_CLIENT_ID_KEY = "google_client_id";
  const GOOGLE_CLIENT_SECRET_KEY = "google_client_secret";

  let apiKey = $state("");
  let googleClientId = $state("");
  let googleClientSecret = $state("");
  let loading = $state(true);
  let saving = $state(false);
  let saved = $state(false);
  let authMessage = $state("");
  let authError = $state("");
  let isAuthing = $state(false);
  let googleCalendarConnected = $state("Connect Google Calendar");

  const init = async () => {
    const db = await Database.load(DB_PATH);
    await db.execute(
      "CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
    );
    const rows = await db.select<{ key: string; value: string }[]>(
      "SELECT key, value FROM settings WHERE key IN ($1, $2, $3)",
      [KEY, GOOGLE_CLIENT_ID_KEY, GOOGLE_CLIENT_SECRET_KEY],
    );
    for (const row of rows) {
      if (row.key === KEY) apiKey = row.value;
      if (row.key === GOOGLE_CLIENT_ID_KEY) googleClientId = row.value;
      if (row.key === GOOGLE_CLIENT_SECRET_KEY) googleClientSecret = row.value;
    }

    await db.close();
    loading = false;
    await connectCalendar();
  };

  const connectCalendar = async () => {
    const connection = await checkCalendarConnection();
    if (connection) {
      googleCalendarConnected = "Calendar - Connected";
    } else {
    }
  };

  const save = async () => {
    saving = true;
    saved = false;
    const db = await Database.load(DB_PATH);
    await db.execute(
      "INSERT INTO settings (key, value) VALUES ($1, $2) ON CONFLICT(key) DO UPDATE SET value = $2",
      [KEY, apiKey],
    );
    await db.execute(
      "INSERT INTO settings (key, value) VALUES ($1, $2) ON CONFLICT(key) DO UPDATE SET value = $2",
      [GOOGLE_CLIENT_ID_KEY, googleClientId],
    );
    await db.execute(
      "INSERT INTO settings (key, value) VALUES ($1, $2) ON CONFLICT(key) DO UPDATE SET value = $2",
      [GOOGLE_CLIENT_SECRET_KEY, googleClientSecret],
    );
    await db.close();
    saving = false;
    saved = true;
  };

  const authenticateGoogleCalendar = async () => {
    isAuthing = true;
    authError = "";
    authMessage = "";
    console.log("authenticating");
    try {
      await invoke("mcp_server_auth", {
        server: "google-calendar",
        clientId: googleClientId,
        clientSecret: googleClientSecret,
      });
      authMessage = "Auth flow started — check the browser and the terminal.";
    } catch (e) {
      console.error(e);
      authError = `Auth failed: ${e}`;
    } finally {
      isAuthing = false;
    }
  };
  init();
</script>

<Sidebar.Provider>
  <SidebarLeft />
  <Sidebar.Inset>
    <header
      class="sticky top-0 flex h-14 shrink-0 items-center gap-2 bg-background"
    >
      <div class="flex flex-1 items-center gap-2 px-3">
        <Sidebar.Trigger />
        <Separator
          orientation="vertical"
          class="me-2 data-[orientation=vertical]:h-4"
        />
        <Breadcrumb.Root>
          <Breadcrumb.List>
            <Breadcrumb.Item>
              <Breadcrumb.Page class="line-clamp-1">Settings</Breadcrumb.Page>
            </Breadcrumb.Item>
          </Breadcrumb.List>
        </Breadcrumb.Root>
      </div>
    </header>
    <div class="flex flex-1 flex-col gap-4 p-4">
      <div class="mx-auto w-full max-w-3xl rounded-xl border bg-background p-6">
        <h1 class="text-lg font-semibold">API Keys</h1>
        <p class="mt-1 text-sm text-muted-foreground">
          Your API key is stored locally in the app database.
        </p>
        {#if loading}
          <p class="mt-4 text-sm text-muted-foreground">Loading…</p>
        {:else}
          <form
            class="mt-4 flex flex-col gap-4"
            onsubmit={(e) => {
              e.preventDefault();
              save();
            }}
          >
            <div class="flex flex-col gap-2">
              <label for="deepseek-api-key" class="text-sm font-medium">
                DeepSeek API Key
              </label>
              <Input
                id="deepseek-api-key"
                type="password"
                placeholder="sk-…"
                bind:value={apiKey}
                autocomplete="off"
              />
            </div>
            <div class="flex flex-col gap-2">
              <label for="google-client-id" class="text-sm font-medium">
                Google Client ID
              </label>
              <Input
                id="google-client-id"
                placeholder="XXXXX.apps.googleusercontent.com"
                bind:value={googleClientId}
                autocomplete="off"
              />
            </div>
            <div class="flex flex-col gap-2">
              <label for="google-client-secret" class="text-sm font-medium">
                Google Client Secret
              </label>
              <Input
                id="google-client-secret"
                type="password"
                placeholder="GOCSPX-…"
                bind:value={googleClientSecret}
                autocomplete="off"
              />
              <p class="text-xs text-muted-foreground">
                Your OAuth Desktop app credentials from Google Cloud Console.
                Used by the Calendar MCP server.
              </p>
            </div>
            <div class="flex items-center gap-3">
              <Button type="submit" disabled={saving}>
                {saving ? "Saving…" : "Save"}
              </Button>
              {#if saved}
                <span class="text-sm text-muted-foreground">Saved</span>
              {/if}
            </div>
          </form>
        {/if}
      </div>
    </div>

    <div class="flex flex-1 flex-col p-4">
      <div class="mx-auto w-full max-w-3xl rounded-xl border bg-background p-6">
        <h1 class="text-lg font-semibold">Connect Services</h1>

        <p class="mt-1 text-sm text-muted-foreground">
          Connect Different Services
        </p>

        <div class="mt-4">
          <Button
            onclick={authenticateGoogleCalendar}
            disabled={isAuthing}
            class="bg-blue-700"
            ><CalendarFilledIcon />{googleCalendarConnected}</Button
          >
          {#if authMessage}<p class="mt-2 text-sm text-muted-foreground">
              {authMessage}
            </p>{/if}
          {#if authError}<p class="mt-2 text-sm text-red-600">
              {authError}
            </p>{/if}
        </div>
      </div>
    </div>
  </Sidebar.Inset>
  <SidebarRight />
</Sidebar.Provider>
