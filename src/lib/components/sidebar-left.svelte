<script lang="ts">
  import AudioWaveformIcon from "@lucide/svelte/icons/audio-waveform";
  import BlocksIcon from "@lucide/svelte/icons/blocks";
  import CalendarIcon from "@lucide/svelte/icons/calendar";
  import CommandIcon from "@lucide/svelte/icons/command";
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

  let teams = $derived( 
    workspaceStore.workspaces.map((team) => ({
      name: team.name,
      uuid: team.uuid,
      base_path: team.base_path,
      logo: AudioWaveformIcon
    })
    )
  )
  onMount(async () => {
    try {
      const db = await getDb();
      const getTeams:Base[] = await db.select("SELECT * FROM bases");

      workspaceStore.workspaces = getTeams

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
    workspaces: [
      {
        name: "Personal Life Management",
        emoji: "🏠",
        pages: [
          {
            name: "Daily Journal & Reflection",
            url: "#",
            emoji: "📔",
          },
          {
            name: "Health & Wellness Tracker",
            url: "#",
            emoji: "🍏",
          },
          {
            name: "Personal Growth & Learning Goals",
            url: "#",
            emoji: "🌟",
          },
        ],
      },
      {
        name: "Professional Development",
        emoji: "💼",
        pages: [
          {
            name: "Career Objectives & Milestones",
            url: "#",
            emoji: "🎯",
          },
          {
            name: "Skill Acquisition & Training Log",
            url: "#",
            emoji: "🧠",
          },
          {
            name: "Networking Contacts & Events",
            url: "#",
            emoji: "🤝",
          },
        ],
      },
      {
        name: "Creative Projects",
        emoji: "🎨",
        pages: [
          {
            name: "Writing Ideas & Story Outlines",
            url: "#",
            emoji: "✍️",
          },
          {
            name: "Art & Design Portfolio",
            url: "#",
            emoji: "🖼️",
          },
          {
            name: "Music Composition & Practice Log",
            url: "#",
            emoji: "🎵",
          },
        ],
      },
      {
        name: "Home Management",
        emoji: "🏡",
        pages: [
          {
            name: "Household Budget & Expense Tracking",
            url: "#",
            emoji: "💰",
          },
          {
            name: "Home Maintenance Schedule & Tasks",
            url: "#",
            emoji: "🔧",
          },
          {
            name: "Family Calendar & Event Planning",
            url: "#",
            emoji: "📅",
          },
        ],
      },
      {
        name: "Travel & Adventure",
        emoji: "🧳",
        pages: [
          {
            name: "Trip Planning & Itineraries",
            url: "#",
            emoji: "🗺️",
          },
          {
            name: "Travel Bucket List & Inspiration",
            url: "#",
            emoji: "🌎",
          },
          {
            name: "Travel Journal & Photo Gallery",
            url: "#",
            emoji: "📸",
          },
        ],
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
    <TeamSwitcher teams={teams} />
    <NavMain items={data.navMain} />
  </Sidebar.Header>
  <Sidebar.Content>
    <NavWorkspaces workspaces={data.workspaces} />
    <NavSecondary items={data.navSecondary} class="mt-auto" />
  </Sidebar.Content>
  <Sidebar.Rail />
</Sidebar.Root>
