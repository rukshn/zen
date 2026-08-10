<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Textarea } from "$lib/components/ui/textarea";
  import { checkCalendarConnection, getAPIKey } from "@/funcs";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";
  import * as Avatar from "$lib/components/ui/avatar";
  import { profile } from "$lib/store/profile.svelte";
  import {
    type ChatMessage,
    type ToolDefinition,
  } from "$lib/store/message.svelte";
  import { messageStore } from "$lib/store/message.svelte";
  import ThreeDotsLoadingIcon from "@iconify-svelte/eos-icons/three-dots-loading";

  let pendingAction = $state(false);
  let userMessage = $state("");
  let message = $state("");
  let hasMessage = $state(false);
  let isStreaming = $state(false);

  type OpenAiTool = {
    type: "function";
    function: {
      name: string;
      description: string;
      parameters: Record<string, any>;
    };
  };

  type ToolCall = {
    id: string;
    type: "function";
    function: { name: string; arguments: string };
  };

  const toOpenAiTools = (defs: ToolDefinition[]): OpenAiTool[] =>
    defs.map((d) => ({
      type: "function",
      function: {
        name: d.name,
        description: d.description ?? "",
        parameters: d.input_schema,
      },
    }));

  let messages = messageStore.messages;
  let unlisteners: UnlistenFn[] = [];
  let activeRequestId: string | null = null;
  let apiKey = "";
  let apiTools: OpenAiTool[] = [];
  let toolLookup = new Map<string, string>();
  let toolRunCount = 0;
  const MAX_TOOL_ROUNDS = 6;

  type ToolCallAcc = { id: string; name: string; arguments: string };
  let streamAcc: {
    assistantIndex: number | null;
    toolCalls: Map<number, ToolCallAcc>;
  } = {
    assistantIndex: null,
    toolCalls: new Map(),
  };
  const resetStreamAcc = () => ({
    assistantIndex: null,
    toolCalls: new Map<number, ToolCallAcc>(),
  });

  onMount(async () => {
    unlisteners = await Promise.all([
      listen<{ requestId: string; chunk: any }>("llm:delta", (event) => {
        const { requestId, chunk } = event.payload;
        if (requestId !== activeRequestId) return;

        const delta = chunk?.choices?.[0]?.delta;
        const text: string | undefined = delta?.content;
        if (text) {
          if (streamAcc.assistantIndex === null) {
            messages.push({ role: "assistant", content: "" });
            streamAcc.assistantIndex = messages.length - 1;
          }
          const ai = streamAcc.assistantIndex;
          const m = messages[ai] as ChatMessage;
          m.content = (m.content ?? "") + text;
        }

        const toolCalls = delta?.tool_calls;
        if (Array.isArray(toolCalls)) {
          for (const tc of toolCalls) {
            const idx = tc.index ?? 0;
            const cur = streamAcc.toolCalls.get(idx) ?? {
              id: "",
              name: "",
              arguments: "",
            };
            if (tc.id) cur.id = tc.id;
            if (tc.function?.name) cur.name = tc.function.name;
            if (tc.function?.arguments) cur.arguments += tc.function.arguments;
            streamAcc.toolCalls.set(idx, cur);
          }
        }
      }),

      listen<{ requestId: string }>("llm:done", (event) => {
        if (event.payload.requestId !== activeRequestId) return;
        activeRequestId = null;
        isStreaming = false;
        finalizeTurn();
      }),

      listen<{ requestId: string; error: string }>("llm:error", (event) => {
        if (event.payload.requestId !== activeRequestId) return;
        activeRequestId = null;
        isStreaming = false;
        hasMessage = true;
        message = event.payload.error;
      }),
    ]);
  });

  onDestroy(() => {
    for (const un of unlisteners) un();
  });

  const runStream = async () => {
    activeRequestId = crypto.randomUUID();
    isStreaming = true;
    const apiMessages = messages.map(({ tools: _tools, ...msg }) => msg);

    try {
      await invoke("llm_stream_chat", {
        requestId: activeRequestId,
        payload: {
          model: "deepseek-v4-flash",
          messages: apiMessages,
          tools: apiTools,
          tool_choice: "auto",
        },
        apiKey,
      });
    } catch (e) {
      activeRequestId = null;
      isStreaming = false;
      hasMessage = true;
      message = String(e);
    }
  };

  const finalizeTurn = async () => {
    const acc = streamAcc;
    streamAcc = resetStreamAcc();
    if (acc.toolCalls.size === 0) return;

    const calls: ToolCallAcc[] = [...acc.toolCalls.values()];
    console.log("[llm] tool calls received:", calls);

    const toolCallsMsg: ToolCall[] = calls.map((c) => ({
      id: c.id || `call_${c.name}`,
      type: "function",
      function: { name: c.name, arguments: c.arguments || "{}" },
    }));

    const ai = acc.assistantIndex;
    if (ai !== null) {
      messages[ai].content =
        `Calling ${calls.map((c) => c.name).join(", ")}...`;
      messages[ai].tool_calls = toolCallsMsg;
    }

    const results = await Promise.all(
      calls.map(async (c) => {
        const server = toolLookup.get(c.name);
        try {
          if (!server)
            throw new Error(`no server registered for tool "${c.name}"`);
          const args = c.arguments ? JSON.parse(c.arguments) : {};
          const res = await invoke<{ text: string; is_error: boolean }>(
            "mcp_call_tool",
            {
              server,
              tool: c.name,
              arguments: args,
            },
          );
          if (res.is_error) return `Tool error: ${res.text}`;
          return res.text || `(tool "${c.name}" returned no text)`;
        } catch (e) {
          return `Failed to call "${c.name}": ${String(e)}`;
        }
      }),
    );

    results.forEach((content, i) => {
      messages.push({
        role: "tool",
        tool_call_id: toolCallsMsg[i].id,
        content,
      });
    });

    if (toolRunCount >= MAX_TOOL_ROUNDS) {
      hasMessage = true;
      message = `Stopped after ${MAX_TOOL_ROUNDS} tool rounds.`;
      return;
    }
    toolRunCount += 1;
    await runStream();
  };

  const sendRequest = async () => {
    hasMessage = false;
    if (!userMessage.trim()) return;

    if (!(await checkCalendarConnection())) {
      hasMessage = true;
      message = "Error connecting to Google calendar";
      return;
    }

    try {
      const toolDefs: ToolDefinition[] = await invoke("mcp_tool_defs");
      toolLookup = new Map(toolDefs.map((d) => [d.name, d.server]));
      apiTools = toOpenAiTools(toolDefs);
      apiKey = (await getAPIKey()) ?? "";
    } catch (e) {
      hasMessage = true;
      message = String(e);
      return;
    }
    if (!apiKey) {
      hasMessage = true;
      message = "No LLM api key defined";
      return;
    }

    messages.push({ role: "user", content: userMessage });
    userMessage = "";
    toolRunCount = 0;
    streamAcc = resetStreamAcc();
    await runStream();
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
  <Button onclick={sendRequest} disabled={isStreaming}>
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