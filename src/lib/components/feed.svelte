<script lang="ts">
  import { messageStore } from "$lib/store/message.svelte";
  import * as Avatar from "$lib/components/ui/avatar";
  import { loadProfile, profile } from "@/store/profile.svelte";
  import { onMount } from "svelte";
  import { marked } from "marked";

  onMount(async () => {
    await loadProfile();
  });

  const visibleMessages = $derived(
    messageStore.messages.filter(
      (m) => m.role === "user" || (m.role === "assistant" && !m.tool_calls),
    ),
  );

  const md2html = (md: string) => {
    return marked(md);
  };
</script>

<div class="py-3 px-4 space-y-6">
  {#each visibleMessages as msg}
    {#if msg.role === "user"}
      <div class="flex gap-2 justify-end">
        <div
          class="prose prose-sm grow rounded-l-lg rounded-tr-lg rounded-br-sm px-4 py-4 bg-blue-50"
        >
          {@html md2html(msg.content ?? "")}
        </div>

        <div class="h-full items-baseline">
          <Avatar.Root>
            <Avatar.Image src={profile.avatar}></Avatar.Image>
            <Avatar.Fallback>R</Avatar.Fallback>
          </Avatar.Root>
        </div>
      </div>
    {/if}

    {#if msg.role === "assistant"}
      <div class="flex gap-2">
        <Avatar.Root>
          <Avatar.Image src="/2151065166.jpg"></Avatar.Image>
          <Avatar.Fallback>A</Avatar.Fallback>
        </Avatar.Root>
        <div
          class="grow prose prose-sm rounded-r-lg rounded-tl-lg rounded-bl-sm p-4 bg-amber-50"
        >
          {@html md2html(msg.content ?? "")}
        </div>
      </div>
    {/if}
  {/each}
</div>
