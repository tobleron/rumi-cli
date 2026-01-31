# Rust Quick Reference for Rumi-CLI

## Async (Tokio)
- **Main:** `#[tokio::main] async fn main() { ... }`.
- **Await:** `.await?` for handling results in async contexts.

## JSON (Serde)
- **Derive:** `#[derive(Serialize, Deserialize)]`.
- **Naming:** `#[serde(rename_all = "camelCase")]`.

## Error Handling
- Use `Result<T, Box<dyn std::error::Error>>` for generic CLI errors.
- Use `anyhow` if available for easier context.

## Reqwest
- **Client:** `let client = reqwest::Client::new();`.
- **JSON Post:** `client.post(url).json(&body).send().await?`.
