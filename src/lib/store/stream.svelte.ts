import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  messageStore,
  type ChatMessage,
  type ToolDefinition,
} from "$lib/store/message.svelte";
import { checkCalendarConnection, getAPIKey } from "@/funcs";
import { getDb } from "./dbstore";
import { workspaceStore } from "./bases.svelte";

type OpenAiTool = {
  type: "function";
  function: {
    name: string;
    description: string;
    parameters: Record<string, any>;
  };
};

type SearchResult = {
  name: string;
  server: string;
  description: string;
  input_schema: Record<string, any>;
  score: number;
};

type ToolCallAcc = { id: string; name: string; arguments: string };

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

export const streamState = $state({
  isStreaming: false,
  hasMessage: false,
  message: "",
});

let initialized = false;
let activeRequestId: string | null = null;
let apiKey = "";
let apiEndpoint = "";
let modelName = "";
let apiTools: OpenAiTool[] = [];
let toolLookup = new Map<string, string>();
let toolRunCount = 0;
const MAX_TOOL_ROUNDS = 6;

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

export const initStreaming = async () => {
  if (initialized) return;
  initialized = true;

  let unlisteners: UnlistenFn[] = [];

  unlisteners = await Promise.all([
    listen<{ requestId: string; chunk: any }>("llm:delta", (event) => {
      const { requestId, chunk } = event.payload;
      if (requestId !== activeRequestId) return;

      const delta = chunk?.choices?.[0]?.delta;
      const text: string | undefined = delta?.content;
      if (text) {
        if (streamAcc.assistantIndex === null) {
          messageStore.messages.push({ role: "assistant", content: "" });
          streamAcc.assistantIndex = messageStore.messages.length - 1;
        }
        const ai = streamAcc.assistantIndex;
        const m = messageStore.messages[ai] as ChatMessage;
        m.content = (m.content ?? "") + text;
      }

      const toolCalls = delta?.tool_calls;
      if (Array.isArray(toolCalls)) {
        if (streamAcc.assistantIndex === null) {
          messageStore.messages.push({ role: "assistant", content: null });
          streamAcc.assistantIndex = messageStore.messages.length - 1;
        }
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
      finalizeTurn();
    }),

    listen<{ requestId: string; error: string }>("llm:error", (event) => {
      if (event.payload.requestId !== activeRequestId) return;
      activeRequestId = null;
      streamState.isStreaming = false;
      streamState.hasMessage = true;
      streamState.message = event.payload.error;
    }),
  ]);
};

const runStream = async () => {
  activeRequestId = crypto.randomUUID();
  streamState.isStreaming = true;
  const apiMessages = messageStore.messages.map(
    ({ tools: _tools, ...msg }) => msg,
  );

  try {
    await invoke("llm_stream_chat", {
      requestId: activeRequestId,
      payload: {
        model: modelName || "gpt-5.6-luna",
        messages: apiMessages,
        tools: apiTools,
        tool_choice: "auto",
      },
      apiKey,
      apiEndpoint,
    });
  } catch (e) {
    activeRequestId = null;
    streamState.isStreaming = false;
    streamState.hasMessage = true;
    streamState.message = String(e);
  }
};

const finalizeTurn = async () => {
  const acc = streamAcc;
  streamAcc = resetStreamAcc();
  if (acc.toolCalls.size === 0) {
    streamState.isStreaming = false;
    messageStore.activeConversation = crypto.randomUUID();
    try {
      const db = await getDb();
   
      let activeConversation = undefined;
      if (!messageStore.activeConversation) {
        activeConversation = crypto.randomUUID()
        messageStore.activeConversation = activeConversation
      } else {
        activeConversation = messageStore.activeConversation
      }

      const addConversation = db.execute(
        `INSERT INTO conversations (messages, uuid, base) VALUES ($1, $2, $3) ON CONFLICT (uuid) DO UPDATE SET messages = $1`,
        [
          JSON.stringify(messageStore.messages),
          activeConversation,    
          workspaceStore.activeTeam?.id,
        ],
      );
    } catch (e) {
      console.log(e);
    }

    return;
  }

  const calls: ToolCallAcc[] = [...acc.toolCalls.values()];
  console.log("[llm] tool calls received:", calls);
  try {
    const getOriginalToolName = (prefixedName: string): string => {
      const idx = prefixedName.indexOf("_");
      return idx >= 0 ? prefixedName.slice(idx + 1) : prefixedName;
    };

    const toolCallsMsg: ToolCall[] = calls.map((c) => ({
      id: c.id || `call_${c.name}`,
      type: "function",
      function: { name: c.name, arguments: c.arguments || "{}" },
    }));

    const ai = acc.assistantIndex;
    if (ai !== null) {
      messageStore.messages[ai].content =
        `Calling ${calls.map((c) => c.name).join(", ")}...`;
      messageStore.messages[ai].tool_calls = toolCallsMsg;
    }

    const results = await Promise.all(
      calls.map(async (c) => {
        const server = toolLookup.get(c.name);
        try {
          if (!server)
            throw new Error(`no server registered for tool "${c.name}"`);
          const args = c.arguments ? JSON.parse(c.arguments) : {};
          const originalToolName = getOriginalToolName(c.name);
          const res = await invoke<{ text: string; is_error: boolean }>(
            "mcp_call_tool",
            {
              server,
              tool: originalToolName,
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
      messageStore.messages.push({
        role: "tool",
        tool_call_id: toolCallsMsg[i].id,
        content,
      });
    });
  } catch (e) {
    streamState.isStreaming = false;
    streamState.hasMessage = true;
    streamState.message = `Tool round failed: ${String(e)}`;
    return;
  }

  if (toolRunCount >= MAX_TOOL_ROUNDS) {
    streamState.isStreaming = false;
    streamState.hasMessage = true;
    streamState.message = `Stopped after ${MAX_TOOL_ROUNDS} tool rounds.`;
    return;
  }
  toolRunCount += 1;
  await runStream();
};

export const sendRequest = async (userMessage: string) => {
  streamState.hasMessage = false;
  if (!userMessage.trim()) return;

  if (!(await checkCalendarConnection())) {
    streamState.hasMessage = true;
    streamState.message = "Error connecting to Google calendar";
    return;
  }

  try {
    const searchResults: SearchResult[] = await invoke("search_tools", {
      query: userMessage,
      limit: 10,
    });

    let toolDefs: ToolDefinition[];
    let imap_look_accounts: SearchResult[];

    if (searchResults.length > 0) {
      toolDefs = searchResults.map((r) => ({
        name: r.name,
        server: r.server,
        description: r.description,
        input_schema: r.input_schema,
      }));

      const hasImapServer = toolDefs.some((t) => t.server === "imap-mail");
      const hasImapListAccounts = toolDefs.some(
        (t) => t.name === "imap_list_accounts",
      );

      if (hasImapServer && !hasImapListAccounts) {
        imap_look_accounts = await invoke("search_tools", {
          query: "list_imap_accounts",
          limit: 1,
        });
        let map_look_accounts: ToolDefinition = {
          name: imap_look_accounts[0].name,
          server: imap_look_accounts[0].server,
          description: imap_look_accounts[0].description,
          input_schema: imap_look_accounts[0].input_schema,
        };
        console.log(toolDefs);
        console.log(imap_look_accounts);
        console.log(map_look_accounts);
        toolDefs.push(map_look_accounts);
      }
    } else {
      const allTools: ToolDefinition[] = await invoke("mcp_tool_defs");
      toolDefs = allTools;
    }

    toolLookup = new Map(toolDefs.map((d) => [d.name, d.server]));
    apiTools = toOpenAiTools(toolDefs);
    const apiEndpointAndKey = await getAPIKey();
    if (apiEndpointAndKey) {
      apiKey = apiEndpointAndKey.apiKey ?? "";
      apiEndpoint = apiEndpointAndKey.apiEndpoint ?? "";
      modelName = apiEndpointAndKey.modelName ?? "";
    }
  } catch (e) {
    streamState.hasMessage = true;
    streamState.message = String(e);
    return;
  }
  if (!apiKey) {
    streamState.hasMessage = true;
    streamState.message = "No LLM api key defined";
    return;
  }

  messageStore.messages.push({ role: "user", content: userMessage });
  toolRunCount = 0;
  streamAcc = resetStreamAcc();
  await runStream();
};
