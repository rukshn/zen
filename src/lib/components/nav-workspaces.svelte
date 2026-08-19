<script lang="ts">
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import EllipsisIcon from "@lucide/svelte/icons/ellipsis";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import * as Collapsible from "$lib/components/ui/collapsible/index.js";
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import AbstractInstanceOutlinedIcon from "@iconify-svelte/eos-icons/abstract-instance-outlined";
  import { getDb } from "@/store/dbstore";
  import type { Conversation } from "@/store/bases.svelte";
  import { messageStore } from "@/store/message.svelte";
	
  let {
    sidebarChats,
  }: { sidebarChats: { firstMessage: string; uuid: string }[] } = $props();

  const swtichConversation = async(uuid: string) => {
	const db = await getDb()
	const conversation: Conversation[] = await db.select("SELECT * from conversations WHERE uuid = $1" , [uuid])

	if (conversation) {
		messageStore.activeConversation = uuid
		messageStore.messages = JSON.parse(conversation[0].messages)
	}
  }
</script>

<Sidebar.Group>
  <Sidebar.GroupLabel>Converations</Sidebar.GroupLabel>
  <Sidebar.GroupContent>
    <Sidebar.Menu>
      {#each sidebarChats as chat (chat.uuid)}
        <Collapsible.Root>
          <Sidebar.MenuItem>
            <Sidebar.MenuButton onclick={() => swtichConversation(chat.uuid)}>
              {#snippet child({ props })}
                <a href="##" {...props}>
                  <span><AbstractInstanceOutlinedIcon /></span>
                  <span>{chat.firstMessage}</span>
                </a>
              {/snippet}
            </Sidebar.MenuButton>
            <Sidebar.MenuAction showOnHover>
              <PlusIcon />
            </Sidebar.MenuAction>
          </Sidebar.MenuItem>
        </Collapsible.Root>
      {/each}
      <Sidebar.MenuItem>
        <Sidebar.MenuButton class="text-sidebar-foreground/70">
          <EllipsisIcon />
          <span>More</span>
        </Sidebar.MenuButton>
      </Sidebar.MenuItem>
    </Sidebar.Menu>
  </Sidebar.GroupContent>
</Sidebar.Group>
