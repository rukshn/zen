<script lang="ts">
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import { goto } from "$app/navigation";
  import type { Component } from "svelte";

  type Team = {
    name: string;
    logo: Component;
    uuid: string;
    base_path: string;
  };

  let {
    teams,
  }: {
    teams: Team[];
  } = $props();

  let selected = $state<Team | undefined>(undefined);
  let activeTeam = $derived.by(() => selected ?? teams[0]);

  const newBasePage = () => {
    return goto("/new");
  };
</script>

<Sidebar.Menu>
  <Sidebar.MenuItem>
    <DropdownMenu.Root>
      <DropdownMenu.Trigger>
        {#snippet child({ props })}
          <Sidebar.MenuButton {...props} class="w-fit px-1.5">
            {#if activeTeam}
              <div
                class="flex aspect-square size-5 items-center justify-center rounded-md bg-sidebar-primary text-sidebar-primary-foreground"
              >
                <activeTeam.logo class="size-3" />
              </div>
              <span class="truncate font-medium">{activeTeam.name}</span>
            {:else}
              <div
                class="flex aspect-square size-5 items-center justify-center rounded-md bg-sidebar-primary text-sidebar-primary-foreground"
              >
                <PlusIcon class="size-3" />
              </div>
              <span class="truncate font-medium">Select Workspace</span>
            {/if}
            <ChevronDownIcon class="opacity-50" />
          </Sidebar.MenuButton>
        {/snippet}
      </DropdownMenu.Trigger>
      <DropdownMenu.Content
        class="w-64 rounded-lg"
        align="start"
        side="bottom"
        sideOffset={4}
      >
        <DropdownMenu.Label class="text-xs text-muted-foreground"
          >Teams</DropdownMenu.Label
        >
        {#if teams.length > 0}
          {#each teams as team, index (team.uuid)}
            <DropdownMenu.Item
              onSelect={() => (selected = team)}
              class="gap-2 p-2"
            >
              <div
                class="flex size-6 items-center justify-center rounded-xs border"
              >
                <team.logo class="size-4 shrink-0" />
              </div>
              {team.name}
              <DropdownMenu.Shortcut>⌘{index + 1}</DropdownMenu.Shortcut>
            </DropdownMenu.Item>
          {/each}
        {:else}
          <DropdownMenu.Item class="gap-2 p-2">
            <span class="text-xs text-mauve-500">No workspace available</span>
          </DropdownMenu.Item>
        {/if}
        <DropdownMenu.Separator />
        <DropdownMenu.Item onSelect={() => newBasePage()} class="gap-2 p-2">
          <div
            class="flex size-6 items-center justify-center rounded-md border bg-background"
          >
            <PlusIcon class="size-4" />
          </div>
          <div class="font-medium text-muted-foreground">Add workspace</div>
        </DropdownMenu.Item>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  </Sidebar.MenuItem>
</Sidebar.Menu>
