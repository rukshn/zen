<script lang="ts">
  import * as Breadcrumb from "$lib/components/ui/breadcrumb/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import { Separator } from "$lib/components/ui/separator/index.js";
  import SidebarLeft from "$lib/components/sidebar-left.svelte";
  import SidebarRight from "$lib/components/sidebar-right.svelte";
  import CalendarFilledIcon from "@iconify-svelte/tabler/calendar-filled";
  import { invoke } from "@tauri-apps/api/core";
  import { checkCalendarConnection, refreshConnections } from "@/funcs";
  import * as Avatar from "$lib/components/ui/avatar";
  import { profile, loadProfile } from "@/store/profile.svelte";
  import { onMount } from "svelte";
  import EmailIcon from "@iconify-svelte/mage/email";
  import RefreshIcon from "@iconify-svelte/mage/refresh";
  import { checkImapConnection, openImapSetupWizard } from "$lib/funcs";
  import { getDb } from "@/store/dbstore";
  import Label from "@/components/ui/label/label.svelte";

  const KEY = "deepseek_api_key";
  const GOOGLE_CLIENT_ID_KEY = "google_client_id";
  const GOOGLE_CLIENT_SECRET_KEY = "google_client_secret";
  const AVATAR_KEY = "user_avatar";
  const API_ENDPOINT = "api_endpoint";
  const MODEL_NAME = "model_name";

  let connectedAccounts = $state(0);
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
  let avatarError = $state("");
  let apiEndpoint = $state("");
  let modelName = $state("");

  const readAsDataUrl = (file: File) => {
    const reader = new FileReader();
    reader.onload = async () => {
      const dataUrl = String(reader.result);
      await uploadAvatar(dataUrl); // → DB upsert + store update
    };
    reader.onerror = () => {
      avatarError = "Could not read the file.";
    };
    reader.readAsDataURL(file);
  };

  const onPickAvatar = (e: Event) => {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];

    input.value = ""; // allow re-selecting the same file later

    if (!file) return;
    if (!file.type.startsWith("image/")) {
      avatarError = "Please choose an image file.";
      return;
    }
    if (file.size > 2 * 1024 * 1024) {
      avatarError = "Image too large (max 2MB).";
      return;
    }

    avatarError = "";
    readAsDataUrl(file); // pass the File object to the reader
  };

  const uploadAvatar = async (dataUrl: string) => {
    try {
      const db = await getDb();
      await db.execute(
        "INSERT INTO settings (key, value) VALUES ($1, $2) ON CONFLICT(key) DO UPDATE SET value = $2",
        [AVATAR_KEY, dataUrl],
      );
      profile.avatar = dataUrl;
    } catch (e) {
      console.log("error occured in setting avatar");
    }
  };

  onMount(async () => {
    const db = await getDb();
    await db.execute(
      "CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
    );
    const rows = await db.select<{ key: string; value: string }[]>(
      "SELECT key, value FROM settings WHERE key IN ($1, $2, $3, $4, $5)",
      [
        KEY,
        GOOGLE_CLIENT_ID_KEY,
        GOOGLE_CLIENT_SECRET_KEY,
        API_ENDPOINT,
        MODEL_NAME,
      ],
    );

    console.log(rows)
    for (const row of rows) {
      if (row.key === KEY) apiKey = row.value;
      if (row.key === GOOGLE_CLIENT_ID_KEY) googleClientId = row.value;
      if (row.key === GOOGLE_CLIENT_SECRET_KEY) googleClientSecret = row.value;
      if (row.key === API_ENDPOINT) apiEndpoint = row.value;
      if (row.key === MODEL_NAME) modelName = row.value;
    }

    loading = false;
    await connectCalendar();
    const imapAccounts = await checkImapConnection();
    connectedAccounts = imapAccounts.length;
    await loadProfile();
  });

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
    const db = await getDb();
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
    await db.execute(
      "INSERT INTO settings (key, value) VALUES ($1, $2) on CONFLICT(key) DO UPDATE SET value = $2",
      [API_ENDPOINT, apiEndpoint],
    );
    await db.execute(
      "INSERT INTO settings (key, value) VALUES ($1, $2) ON CONFLICT(key) DO UPDATE SET value = $2",
      [MODEL_NAME, modelName],
    );
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

  const updateMCPTools = async () => {
    const updateTools = await invoke("mcp_tool_defs");
    console.log(updateTools);
  };

  const updateConnections = async () => {
    const imapAccounts = await refreshConnections();
    connectedAccounts = imapAccounts.length;
    await invoke("mcp_tool_defs");
  };
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
              <label for="api_endpoint" class="text-sm font-medium"
                >API Endpoint</label
              >
              <Input
                id="api_endpoint"
                type="url"
                placeholder="https://...."
                bind:value={apiEndpoint}
                autocomplete="off"
              />
              <label for="deepseek-api-key" class="text-sm font-medium">
                API Key
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
              <Label for="model_name">Model Name</Label>
              <Input
                id="model_name"
                bind:value={modelName}
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

        <div class="mt-4 flex gap-1.5">
          <Button
            onclick={authenticateGoogleCalendar}
            disabled={isAuthing}
            class="bg-blue-700"
          >
            <CalendarFilledIcon />{googleCalendarConnected}
          </Button>

          <Button onclick={openImapSetupWizard}>
            <EmailIcon />
            {connectedAccounts === 0
              ? `Connect Email`
              : `Connected Emails - ${connectedAccounts}`}
          </Button>
          <Button variant="secondary" onclick={updateConnections}>
            <RefreshIcon />
          </Button>
          {#if authMessage}<p class="mt-2 text-sm text-muted-foreground">
              {authMessage}
            </p>{/if}
          {#if authError}<p class="mt-2 text-sm text-red-600">
              {authError}
            </p>{/if}
        </div>
      </div>
    </div>

    <div class="flex flex-1 flex-col p-4">
      <div class="mx-auto w-full max-w-3xl rounded-xl border bg-background p-6">
        <h1 class="text-lg font-semibold">Your Aatar</h1>
        <div class="flex gap-2">
          <Input
            type="file"
            id="picture"
            onchange={onPickAvatar}
            accept="image/*"
          />
          <Avatar.Root>
            <Avatar.Image src={profile.avatar}></Avatar.Image>
            <Avatar.Fallback>R</Avatar.Fallback>
          </Avatar.Root>
        </div>
      </div>
    </div>

    <div class="flex flex-1 flex-col p-4">
      <div class="mx-auto w-full max-w-3xl rounded-xl border bg-background p-6">
        <h1 class="text-lg font-semibold">Actions</h1>
        <Button variant="secondary" onclick={updateMCPTools}
          >Update MCP tools</Button
        >
      </div>
    </div>
  </Sidebar.Inset>
  <SidebarRight />
</Sidebar.Provider>
