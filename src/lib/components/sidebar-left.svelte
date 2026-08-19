<script lang="ts">
  import AudioWaveformIcon from "@lucide/svelte/icons/audio-waveform";
  import BlocksIcon from "@lucide/svelte/icons/blocks";
  import CalendarIcon from "@lucide/svelte/icons/calendar";
  import HouseIcon from "@lucide/svelte/icons/house";
  import InboxIcon from "@lucide/svelte/icons/inbox";
  import MessageCircleQuestionIcon from "@lucide/svelte/icons/message-circle-question";
  import SearchIcon from "@lucide/svelte/icons/search";
  import Settings2Icon from "@lucide/svelte/icons/settings-2";
  import SparklesIcon from "@lucide/svelte/icons/sparkles";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import NavMain from "./nav-main.svelte";
  import NavSecondary from "./nav-secondary.svelte";
  import NavWorkspaces from "./nav-workspaces.svelte";
  import TeamSwitcher from "./team-switcher.svelte";
  import { onMount, type ComponentProps } from "svelte";
  import { getDb } from "@/store/dbstore";
  import { workspaceStore, type Base } from "@/store/bases.svelte";
  import { type Team, type Conversation } from "@/store/bases.svelte";
  import { messageStore } from "@/store/message.svelte";

  let teams: Team[] = $derived(
    workspaceStore.workspaces.map((team) => ({
      name: team.name,
      uuid: team.uuid,
      base_path: team.base_path,
      logo: AudioWaveformIcon,
      created_at: team.created_at,
      id: team.id,
    })),
  );

  let sidebarChats = $derived(
    messageStore.conversations.map((message) => ({
      firstMessage: JSON.parse(message.messages)[0].content as string,
      uuid: message.uuid
    })),
  );

  onMount(async () => {
    try {
      const db = await getDb();
      const getTeams: Base[] = await db.select("SELECT * FROM bases");

      workspaceStore.workspaces = getTeams;

      const activeProject = workspaceStore.activeTeam?.id;

      const getConversations: Conversation[] = await db.select(
        "SELECT * FROM conversations WHERE base = $1",
        [activeProject],
      );
      messageStore.conversations = getConversations;
    } catch (e) {
      console.error("error connecting to database", e);
    }
  });

  // This is sample data.
  const data = {
    navMain: [
      {
        title: "Search",
        url: "#",
        icon: SearchIcon,
      },
      {
        title: "Ask AI",
        url: "#",
        icon: SparklesIcon,
      },
      {
        title: "Home",
        url: "/",
        icon: HouseIcon,
        isActive: true,
      },
      {
        title: "Inbox",
        url: "#",
        icon: InboxIcon,
        badge: "10",
      },
    ],
    navSecondary: [
      {
        title: "Calendar",
        url: "#",
        icon: CalendarIcon,
      },
      {
        title: "Settings",
        url: "/settings",
        icon: Settings2Icon,
      },
      {
        title: "Templates",
        url: "#",
        icon: BlocksIcon,
      },
      {
        title: "Trash",
        url: "#",
        icon: Trash2Icon,
      },
      {
        title: "Help",
        url: "#",
        icon: MessageCircleQuestionIcon,
      },
    ],
  };

  let {
    ref = $bindable(null),
    ...restProps
  }: ComponentProps<typeof Sidebar.Root> = $props();
</script>

<Sidebar.Root bind:ref class="border-e-0" {...restProps}>
  <Sidebar.Header>
    <TeamSwitcher {teams} />
    <NavMain items={data.navMain} />
  </Sidebar.Header>
  <Sidebar.Content>
    <NavWorkspaces {sidebarChats} />
    <NavSecondary items={data.navSecondary} class="mt-auto" />
  </Sidebar.Content>
  <Sidebar.Rail />
</Sidebar.Root>
