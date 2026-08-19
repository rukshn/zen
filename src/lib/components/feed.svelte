<script lang="ts">
  import { messageStore, type ChatMessage } from "$lib/store/message.svelte";
  import * as Avatar from "$lib/components/ui/avatar";
  import { loadProfile, profile } from "@/store/profile.svelte";
  import { onMount, tick } from "svelte";
  import { marked } from "marked";
  import { openUrl } from "@tauri-apps/plugin-opener";

  onMount(async () => {
    await loadProfile();
  });

  onMount(() => {
    const w = window as Window & {
      openExternalLink?: (url: string) => void;
    };
    w.openExternalLink = (url: string) => {
      openUrl(url);
    };
    return () => {
      delete w.openExternalLink;
    };
  });

  // const visibleMessages =
  //  $derived(
  //   messageStore.messages.filter(
  //     (m) => (m.role === "user" || m.role === "assistant"),
  //   ),
  // );

  marked.use({
    renderer: {
      link({ href, title, tokens }) {
        const text = this.parser.parseInline(tokens);
        const titleAttr = title ? `title="${title}"` : "";

        // Check if it's an external web link
        if (href.startsWith("http://") || href.startsWith("https://")) {
          return `<a href="${href}" ${titleAttr} onclick="event.preventDefault(); window.openExternalLink('${href}')">${text}</a>`;
        }

        // Return normal link for internal/relative anchors
        return `<a href="${href}" ${titleAttr}>${text}</a>`;
      },
    },
  });

  const visibleMessages = $derived(
    messageStore.messages.filter(
      (m) => m.role === "user" || (m.role === "assistant" && !m.tool_calls),
    ),
  );

  const md2html = (md: string) => {
    return marked(md);
  };

  let { viewport }: { viewport: HTMLElement | null } = $props();

  $effect.pre(() => {
    const last = visibleMessages.at(-1);
    if (!last || !viewport) return;
    last.content;
    last.tool_calls;

    const { scrollTop, scrollHeight, clientHeight } = viewport;
    const nearBottom = scrollTop + clientHeight >= scrollHeight - 120;
    if (nearBottom) {
      tick().then(() => {
        viewport.scrollTo({ top: viewport.scrollHeight, behavior: "smooth" });
      });
    }
  });
</script>

<div class="py-3 px-4 space-y-6 max-w-3xl mx-auto mb-8">
  {#each visibleMessages as msg}
    {#if msg.role === "user"}
      <div class="flex gap-2 justify-end items-baseline">
        <div
          class="prose prose-sm grow rounded-l-lg rounded-tr-lg rounded-br-sm px-4 py-4 bg-violet-100 font-bold"
        >
          {@html md2html(msg.content ?? "")}
        </div>

        <Avatar.Root>
          <Avatar.Image src={profile.avatar}></Avatar.Image>
          <Avatar.Fallback>R</Avatar.Fallback>
        </Avatar.Root>
      </div>
    {/if}

    {#if msg.role === "assistant"}
      <div class="flex gap-2 items-baseline">
        <Avatar.Root>
          <Avatar.Image src="/2151065166.jpg"></Avatar.Image>
          <Avatar.Fallback>A</Avatar.Fallback>
        </Avatar.Root>
        <div
          class="grow prose prose-sm rounded-r-lg rounded-tl-lg rounded-bl-sm p-4 {msg.tool_calls
            ? ' bg-mauve-100'
            : 'bg-amber-50'}"
        >
          {#if msg.tool_calls}
            <p class="text-gray-600 font-mono text-xs">
              Calling tool - {JSON.stringify(msg)}
            </p>
          {:else}
            {@html md2html(msg.content ?? "")}
          {/if}
        </div>
      </div>
    {/if}
  {/each}
</div>
