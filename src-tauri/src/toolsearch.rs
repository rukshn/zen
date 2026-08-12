//! BM25-based tool search for dynamic tool retrieval, exposed as a
//! Tauri command. Add `bm25 = "2"` (or similar) to Cargo.toml, or use
//! the hand-rolled version below if you'd rather avoid the dependency.
//!
//! Cargo.toml:
//!   [dependencies]
//!   serde = { version = "1", features = ["derive"] }
//!   serde_json = "1"

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

// ---------------------------------------------------------------------
// 1. Tool catalog — load this once at startup (from a JSON file, embedded
//    const, or DB) and keep it in Tauri's managed state.
// ---------------------------------------------------------------------
#[derive(Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    /// Extra searchable text not shown to the model (aliases, tags)
    pub keywords: Vec<String>,
}

#[derive(Deserialize)]
struct McpToolDef {
    pub server: String,
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}

pub struct ToolCatalog {
    pub tools: Vec<ToolDef>,
}

impl ToolCatalog {
    pub fn load(app: &AppHandle) -> Result<Self, String> {
        let app_data_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("failed to resolve app data dir: {e}"))?;
        let path = app_data_dir.join("mcp-tool-defs.json");
        if !path.exists() {
            return Ok(ToolCatalog { tools: Vec::new() });
        }
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let mcp_defs: Vec<McpToolDef> = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        
        let tools = mcp_defs
            .into_iter()
            .map(|mcp| ToolDef {
                name: format!("{}_{}", mcp.server, mcp.name),
                description: mcp.description.unwrap_or_default(),
                input_schema: mcp.input_schema,
                keywords: vec![mcp.server, mcp.name],
            })
            .collect();
        
        Ok(ToolCatalog { tools })
    }
}

// ---------------------------------------------------------------------
// 2. Minimal BM25 implementation (no external crate needed)
// ---------------------------------------------------------------------
pub struct Bm25Index {
    // one document per tool: tokenized (description + keywords)
    pub docs: Vec<Vec<String>>,
    pub doc_len: Vec<usize>,
    pub avg_doc_len: f64,
    // term -> number of docs containing it
    pub doc_freq: HashMap<String, usize>,
    pub n_docs: usize,
    pub k1: f64,
    pub b: f64,
}

fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

impl Bm25Index {
    pub fn build(catalog: &ToolCatalog) -> Self {
        let docs: Vec<Vec<String>> = catalog
            .tools
            .iter()
            .map(|t| {
                let mut text = t.description.clone();
                text.push(' ');
                text.push_str(&t.keywords.join(" "));
                tokenize(&text)
            })
            .collect();

        let doc_len: Vec<usize> = docs.iter().map(|d| d.len()).collect();
        let avg_doc_len = doc_len.iter().sum::<usize>() as f64 / doc_len.len().max(1) as f64;

        let mut doc_freq: HashMap<String, usize> = HashMap::new();
        for doc in &docs {
            let unique: std::collections::HashSet<&String> = doc.iter().collect();
            for term in unique {
                *doc_freq.entry(term.clone()).or_insert(0) += 1;
            }
        }

        Bm25Index {
            n_docs: docs.len(),
            docs,
            doc_len,
            avg_doc_len,
            doc_freq,
            k1: 1.5,
            b: 0.75,
        }
    }

    fn idf(&self, term: &str) -> f64 {
        let n = self.n_docs as f64;
        let df = *self.doc_freq.get(term).unwrap_or(&0) as f64;
        // BM25 idf with +1 smoothing to keep it non-negative
        ((n - df + 0.5) / (df + 0.5) + 1.0).ln()
    }

    /// Returns (doc_index, score) sorted descending by score
    pub fn search(&self, query: &str, limit: usize) -> Vec<(usize, f64)> {
        let query_terms = tokenize(query);
        let mut scores: Vec<(usize, f64)> = (0..self.n_docs)
            .map(|i| {
                let doc = &self.docs[i];
                let dl = self.doc_len[i] as f64;
                let score: f64 = query_terms
                    .iter()
                    .map(|term| {
                        let tf = doc.iter().filter(|t| *t == term).count() as f64;
                        if tf == 0.0 {
                            return 0.0;
                        }
                        let idf = self.idf(term);
                        let numer = tf * (self.k1 + 1.0);
                        let denom = tf + self.k1 * (1.0 - self.b + self.b * dl / self.avg_doc_len);
                        idf * numer / denom
                    })
                    .sum();
                (i, score)
            })
            .filter(|(_, s)| *s > 0.0)
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scores.truncate(limit);
        scores
    }
}

// ---------------------------------------------------------------------
// 3. Tauri command wiring
// ---------------------------------------------------------------------
use tauri::State;

pub struct AppState {
    pub catalog: ToolCatalog,
    pub index: Bm25Index,
}

impl AppState {
    pub fn load(app: &AppHandle) -> Result<Self, String> {
        let catalog = ToolCatalog::load(app)?;
        let index = Bm25Index::build(&catalog);
        Ok(AppState { catalog, index })
    }
    
    pub fn refresh(&mut self, app: &AppHandle) -> Result<(), String> {
        self.catalog = ToolCatalog::load(app)?;
        self.index = Bm25Index::build(&self.catalog);
        Ok(())
    }
}

#[derive(Serialize)]
pub struct SearchResult {
    pub name: String,
    pub server: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub score: f64,
}

#[tauri::command]
pub fn search_tools(query: String, limit: Option<usize>, state: State<Mutex<Option<AppState>>>) -> Vec<SearchResult> {
    let limit = limit.unwrap_or(10);
    let guard = state.lock().unwrap();
    if let Some(app_state) = guard.as_ref() {
        app_state
            .index
            .search(&query, limit)
            .into_iter()
            .map(|(idx, score)| {
                let t = &app_state.catalog.tools[idx];
                let server = t.name.split('_').next().unwrap_or("").to_string();
                SearchResult {
                    name: t.name.clone(),
                    server,
                    description: t.description.clone(),
                    input_schema: t.input_schema.clone(),
                    score,
                }
            })
            .collect()
    } else {
        Vec::new()
    }
}

