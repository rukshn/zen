<script lang="ts">
  import * as Breadcrumb from "$lib/components/ui/breadcrumb/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import { Separator } from "$lib/components/ui/separator/index.js";
  import SidebarLeft from "$lib/components/sidebar-left.svelte";
  import { onMount } from "svelte";
  import * as Field from "$lib/components/ui/field";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getDb } from "@/store/dbstore";
  import { workspaceStore, type Base } from "@/store/bases.svelte";

  const save = async () => {
    try {
      const db = await getDb();
      const createBase = await db.execute(
        "INSERT INTO bases (name, uuid, created_at, base_path) VALUES($1, $2, CURRENT_TIMESTAMP, $3);",
        [name, crypto.randomUUID(), workspace],
      );

      if (createBase.lastInsertId) {
        console.log("created", name);
        message = `Workspace ${name} created, select it from left sidebar`;
        name = "";
        workspace = "";
        const getWorkspaces: Base[] = await db.select("SELECT * FROM bases")
        workspaceStore.workspaces = getWorkspaces
      } else {
        error = true;
        message = "Error creating workspace: " + createBase;
      }
    } catch (e) {
      error = true;
      message = "Error creating workspace: " + e;
      console.log("error creating base", e);
    }
  };

  
  onMount(async () => {
    const db = await getDb();
  });
  
  let message = $state("");
  let error = $state(false);
  let loading = $state(false);
  let name = $state("");
  let workspace = $state("");

  const pickFolder = async () => {
    const picked = await open({ directory: true });
    if (typeof picked === "string") workspace = picked;
  };

  const deleteWorkspace = async (uuid: string) => {
    const db = await getDb();
    const deleteWp = await db.execute("DELETE FROM bases WHERE uuid = $1", [
      uuid,
    ]);
    if (deleteWp.rowsAffected === 1) {
      const getWorkspaces: Base[]= await db.select("SELECT * FROM bases");
      workspaceStore.workspaces = getWorkspaces
    } else {
      console.log("failed to delete workspace");
    }
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
              <Breadcrumb.Page class="line-clamp-1">New Base</Breadcrumb.Page>
            </Breadcrumb.Item>
          </Breadcrumb.List>
        </Breadcrumb.Root>
      </div>
    </header>
    <div class="flex flex-1 flex-col gap-4 p-4">
      <div class="mx-auto w-full max-w-3xl rounded-xl border bg-background p-6">
        <h1 class="text-lg font-semibold">API Keys</h1>

        <form
          class="mt-4 flex flex-col gap-4"
          onsubmit={(e) => {
            e.preventDefault();
            save();
          }}
        >
          <Field.Group>
            <Field.Set>
              <Field.Legend>👋 Create your new workspace</Field.Legend>
              <Field.Description
                >Your conversations are contained in a workspace (work,
                personal..)</Field.Description
              >
            </Field.Set>

            {#if loading}
              <Field.Group>
                <p class="mt-4 text-sm text-muted-foreground">Loading…</p>
              </Field.Group>
            {:else}
              <Field.Group>
                <Field.Label for="name">Workspace Name</Field.Label>
                <Input
                  required
                  id="name"
                  autocomplete="off"
                  placeholder="Workspace Name"
                  bind:value={name}
                />
              </Field.Group>
              <Field.Group>
                <Field.Label for="path">Workspace Path</Field.Label>
                <div class="flex gap-2">
                  <Input
                    bind:value={workspace}
                    required
                    id="path"
                    autocomplete="off"
                    placeholder="/path/to/workspace"
                  />
                  <Button type="button" variant="outline" onclick={pickFolder}>
                    Browse…
                  </Button>
                </div>
              </Field.Group>
            {/if}
            <Field.Field orientation="horizontal">
              <Button type="submit">Create</Button>
              {#if message}
                <Field.Label class={error ? "text-red-700" : "text-green-700"}
                  >{message}</Field.Label
                >
              {/if}
            </Field.Field>
          </Field.Group>
        </form>
      </div>

      <div class="mx-auto w-full max-w-3xl rounded-xl border bg-background p-6">
        <h1 class="text-lg font-semibold">Current Workspaces</h1>

        {#each workspaceStore.workspaces as wp}
          <div class="grid grid-cols-2 my-3">
            <div>{wp.name}</div>
            <div class="flex justify-end">
              <Button
                variant="destructive"
                onclick={() => deleteWorkspace(wp.uuid)}>Delete</Button
              >
            </div>
          </div>
        {/each}
      </div>
    </div>
  </Sidebar.Inset>
</Sidebar.Provider>
