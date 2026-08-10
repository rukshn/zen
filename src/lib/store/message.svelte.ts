 export type ChatMessage = {
    role: "user" | "assistant" | "tool";
    content: string | null;
    tools?: ToolDefinition[];
    tool_calls?: ToolCall[];
    tool_call_id?: string;
  };

   export type ToolCall = {
    id: string;
    type: "function";
    function: { name: string; arguments: string };
  };

  export   type ToolDefinition = {
    server: string;
    name: string;
    description?: string | null;
    input_schema: Record<string, any>;
  };


    export type OpenAiTool = {
    type: "function";
    function: {
      name: string;
      description: string;
      parameters: Record<string, any>;
    };
  };

export const messageStore = $state<{messages: ChatMessage[]}>({messages: []})

