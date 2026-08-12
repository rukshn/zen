import Database from '@tauri-apps/plugin-sql';
const SETTINGS_DB_PATH = "sqlite:settings.db";


let dbInstance: Database | null = null;
let loadPromise: Promise<Database> | null = null;

/** Get the shared DB connection, loading it once if needed. */
export function getDb(): Promise<Database> {
  if (dbInstance) return Promise.resolve(dbInstance);
  if (!loadPromise) {
    loadPromise = Database.load(SETTINGS_DB_PATH).then((db) => {
      dbInstance = db;
      return db;
    });
  }
  return loadPromise;
}

/** Close the shared connection. Call this ONLY on full app teardown. */
export async function closeDb(): Promise<void> {
  if (dbInstance) {
    await dbInstance.close();
    dbInstance = null;
    loadPromise = null;
  }
}