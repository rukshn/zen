<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Textarea } from "$lib/components/ui/textarea";
  import * as Avatar from "$lib/components/ui/avatar";
  import { profile } from "$lib/store/profile.svelte";
  import { streamState, sendRequest } from "$lib/store/stream.svelte";
  import ThreeDotsLoadingIcon from "@iconify-svelte/eos-icons/three-dots-loading";

  let pendingAction = $state(false);
  let userMessage = $state("");

  const isStreaming = $derived(streamState.isStreaming);
  const hasMessage = $derived(streamState.hasMessage);
  const message = $derived(streamState.message);

  const onSend = async () => {
    await sendRequest(userMessage);
    userMessage = "";
  };
</script>

<div class="flex">
  <Avatar.Root>
    <Avatar.Image class="bg-center" src={profile.avatar}></Avatar.Image>
    <Avatar.Fallback>R</Avatar.Fallback>
  </Avatar.Root>
  <Textarea
    placeholder="What would you like to do today?"
    autofocus
    class="resize-none focus-visible:border-0 focus-visible:ring-0 border-none outline-0 shadow-none"
    bind:value={userMessage}
  />
</div>
<div class="flex mt-2 justify-end">
  <Button onclick={onSend} disabled={isStreaming}>
    {#if isStreaming}
      <ThreeDotsLoadingIcon height="1em" />
    {/if}
    {isStreaming ? "Thinking..." : "Ask"}
  </Button>
</div>

{#if pendingAction}
  <div class="space-y-2">
    <p class="text-xs">Approve or reject the action</p>
    <Button size="xs" variant="default">Approve</Button>
    <Button size="xs" variant="secondary">Reject</Button>
  </div>
{/if}

{#if hasMessage}
  <div class="space-y-2">
    <p class="text-xs">{message}</p>
  </div>
{/if}
