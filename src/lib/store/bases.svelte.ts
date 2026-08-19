import { getDb } from "./dbstore";
import { type Component } from "svelte";

export type Base = {
  name: string;
  uuid: string;
  id: number;
  base_path: string;
  created_at: Date;
};

export type Team = {
  name: string;
  logo: Component;
  uuid: string;
  created_at: Date;
  base_path: string;
  id: number;
};

export type Conversation = {
  messages: string, 
  id: number, 
  uuid: string, 
  created_at: Date
}

const getWorkspaces = async () => {
  const db = await getDb();
  const getWorkspaces: Base[] = await db.select("SELECT * FROM bases");
  return getWorkspaces;
};

export const workspaceStore = $state<{
  workspaces: Base[];
  activeTeam: Base | null;
}>({ workspaces: [], activeTeam: null });
