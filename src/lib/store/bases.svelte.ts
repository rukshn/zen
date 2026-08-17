import { getDb } from "./dbstore"

export type Base = {
    name: string 
    created_at: Date
    uuid: string
    id: number
    base_path: string
}

const getWorkspaces = async() => {
    const db = await getDb();
    const getWorkspaces: Base[] = await db.select("SELECT * FROM bases")
    return getWorkspaces
}

export const workspaceStore = $state<{workspaces: Base[]}>({workspaces: []})