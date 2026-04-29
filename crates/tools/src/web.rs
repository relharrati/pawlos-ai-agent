use anyhow::Result;

pub struct WebTool;

impl WebTool {
    pub fn search_definition() -> serde_json::Value {
        serde_json::json!({
            "name": "web_search",
            "description": "Search the web using DuckDuckGo Lite API and return top results.",
            "parameters": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query" },
                    "num_results": { "type": "integer", "description": "Max results (default 5)" }
                },
                "required": ["query"]
            }
        })
    }

    pub fn fetch_definition() -> serde_json::Value {
        serde_json::json!({
            "name": "web_fetch",
            "description": "Fetch the content of a URL and return it as plain text.",
            "parameters": {
                "type": "object",
                "properties": {
                    "url": { "type": "string" }
                },
                "required": ["url"]
            }
        })
    }

    pub async fn search(args: &serde_json::Value) -> Result<String> {
        let query = args["query"].as_str().unwrap_or("").to_string();
        let num   = args["num_results"].as_u64().unwrap_or(5) as usize;

        let encoded = urlencoding::encode(&query);
        let url = format!("https://html.duckduckgo.com/html/?q={encoded}");
        let client = reqwest::Client::builder()
            .user_agent("pawlos-agent/0.1")
            .build()?;
        let body = client.get(&url).send().await?.text().await?;

        // Extract result snippets from the HTML (simple pattern)
        let re = regex::Regex::new("class=\"result__snippet\"").unwrap();
        let mut results = Vec::new();
        for cap in re.find_iter(&body).take(num) {
            let snippet = cap.as_str().trim().to_string();
            if !snippet.is_empty() { results.push(snippet); }
        }

        if results.is_empty() {
            Ok(format!("No results found for: {query}"))
        } else {
            Ok(results.join("\n---\n"))
        }
    }

    pub async fn fetch(args: &serde_json::Value) -> Result<String> {
        let url = args["url"].as_str().unwrap_or("").to_string();
        let client = reqwest::Client::builder()
            .user_agent("pawlos-agent/0.1")
            .build()?;
        let body = client.get(&url).send().await?.text().await?;
        // Strip HTML tags naively
        let re = regex::Regex::new(r"<[^>]+>")?;
        let text = re.replace_all(&body, "").to_string();
        // Collapse whitespace
        let ws = regex::Regex::new(r"\s{3,}")?;
        let clean = ws.replace_all(&text, "\n\n").to_string();
        Ok(clean.chars().take(8000).collect())
    }
}
