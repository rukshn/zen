<script lang="ts">
  import * as Breadcrumb from "$lib/components/ui/breadcrumb/index.js";
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import { Separator } from "$lib/components/ui/separator/index.js";
  import SidebarLeft from "$lib/components/sidebar-left.svelte";
  import Feed from "$lib/components/feed.svelte";
  import SidebarRight from "$lib/components/sidebar-right.svelte";
  import Chat from "@/components/chat.svelte";
  import { loadProfile } from "$lib/store/profile.svelte";
  import { getDb } from "@/store/dbstore";
  import { ScrollArea } from "$lib/components/ui/scroll-area";

  const KEY = "deepseek_api_key";

  let apiKey = $state("");
  let loading = $state(true);

  const onMount = async () => {
    await loadProfile();
    const db = await getDb();
    const rows = await db.select<{ key: string; value: string }[]>(
      "SELECT key, value FROM settings WHERE key IN ($1)",
      [KEY],
    );

    if (rows[0]) {
      apiKey = rows[0].value;
    }

    loading = false;
  };

  const sendRequest = () => {};
  onMount();
</script>

<Sidebar.Provider>
  <SidebarLeft />
  <Sidebar.Inset>
    <div class="flex h-screen flex-col overflow-hidden">
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
                <Breadcrumb.Page class="line-clamp-1">
                  Agent Calls
                </Breadcrumb.Page>
              </Breadcrumb.Item>
            </Breadcrumb.List>
          </Breadcrumb.Root>
        </div>
      </header>

      <div class="flex min-h-0 flex-1 flex-col gap-4 p-4">
        <div
          class="mx-auto w-full max-w-3xl shrink-0 rounded-xl bg-muted/50 px-4 py-3"
        >
          <Chat />
        </div>

        <ScrollArea class="min-h-0 w-full flex-1">
          <Feed />
        </ScrollArea>
      </div>
    </div>
  </Sidebar.Inset>
  <SidebarRight />
</Sidebar.Provider>
