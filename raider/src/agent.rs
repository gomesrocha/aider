use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use regex::Regex;
use std::fs;

use crate::llm::LlmProvider;
use crate::fs_ops::FileOps;

pub struct AgentEngine {
    provider: Box<dyn LlmProvider>,
    workspace: PathBuf,
}

#[derive(Debug, PartialEq)]
pub struct ReplaceBlock {
    pub filepath: String,
    pub search: String,
    pub replace: String,
}

impl AgentEngine {
    pub fn new(provider: Box<dyn LlmProvider>, workspace: impl AsRef<Path>) -> Self {
        Self {
            provider,
            workspace: workspace.as_ref().to_path_buf(),
        }
    }

    pub fn build_system_prompt() -> String {
        r#"You are an expert AI software engineer.
You are helping the user edit their code.
Your output must strictly consist of SEARCH/REPLACE blocks.
Do not output anything else except these blocks.

The SEARCH/REPLACE block format is:
```filepath
<<<<<<< SEARCH
[Exact lines to search for]
=======
[Lines to replace with]
>>>>>>> REPLACE
```

- `filepath` must be the exact path of the file relative to the project root.
- The `SEARCH` section must contain the *exact* lines from the file that need to be replaced, including indentation.
- The `REPLACE` section contains the new lines.

To create a NEW file, simply leave the SEARCH section empty:
```src/new_file.rs
<<<<<<< SEARCH
=======
fn main() {
    println!("I am a new file!");
}
>>>>>>> REPLACE
```

Example of editing an existing file:
```src/main.rs
<<<<<<< SEARCH
fn main() {
    println!("Hello");
}
=======
fn main() {
    println!("Hello world!");
}
>>>>>>> REPLACE
```
"#.to_string()
    }

    pub fn build_user_prompt(&self, prompt: &str, files: &[PathBuf]) -> Result<String> {
        let mut user_prompt = String::new();
        user_prompt.push_str("Here are the current files:\n\n");

        for file in files {
            // convert absolute paths to relative if possible, otherwise use as is
            let rel_path = file.strip_prefix(&self.workspace).unwrap_or(file);
            let content = FileOps::read_file(file)?;
            user_prompt.push_str(&format!("---\nFile: {}\n\n{}\n\n", rel_path.display(), content));
        }

        user_prompt.push_str("Task:\n");
        user_prompt.push_str(prompt);

        Ok(user_prompt)
    }

    pub fn parse_blocks(response: &str) -> Vec<ReplaceBlock> {
        let mut blocks = Vec::new();
        // Regex to capture blocks:
        // Group 1: filepath
        // Group 2: search block
        // Group 3: replace block
        let re = Regex::new(r"(?ms)^```([^\n]+)\n<<<<<<< SEARCH\n(.*?)\n=======\n(.*?)\n>>>>>>> REPLACE\n```").unwrap();

        for cap in re.captures_iter(response) {
            let filepath = cap[1].trim().to_string();
            let search = cap[2].to_string();
            let replace = cap[3].to_string();

            blocks.push(ReplaceBlock {
                filepath,
                search,
                replace,
            });
        }
        blocks
    }

    pub fn apply_blocks(&self, blocks: &[ReplaceBlock]) -> Result<()> {
        for block in blocks {
            let full_path = self.workspace.join(&block.filepath);

            // Handle new file creation
            if !full_path.exists() {
                if let Some(parent) = full_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                if block.search.trim().is_empty() {
                    fs::write(&full_path, &block.replace)
                        .with_context(|| format!("Failed to write new file {:?}", full_path))?;
                    tracing::info!("Created new file: {}", block.filepath);
                    continue;
                } else {
                    anyhow::bail!("File not found but SEARCH block is not empty: {:?}", full_path);
                }
            }

            let mut content = fs::read_to_string(&full_path)
                .with_context(|| format!("Failed to read {:?}", full_path))?;

            if content.contains(&block.search) {
                content = content.replacen(&block.search, &block.replace, 1);
                fs::write(&full_path, content)
                    .with_context(|| format!("Failed to write {:?}", full_path))?;
                tracing::info!("Updated file: {}", block.filepath);
            } else {
                tracing::warn!("Could not find SEARCH block in {}", block.filepath);
            }
        }
        Ok(())
    }

    pub async fn run_task(&self, task_prompt: &str, target_files: &[PathBuf]) -> Result<()> {
        let system_prompt = Self::build_system_prompt();
        let user_prompt = self.build_user_prompt(task_prompt, target_files)?;

        tracing::info!("Sending request to LLM...");
        let response = self.provider.generate(&system_prompt, &user_prompt).await?;
        tracing::debug!("LLM response:\n{}", response);

        let blocks = Self::parse_blocks(&response);
        if blocks.is_empty() {
            tracing::info!("No SEARCH/REPLACE blocks found in the LLM response.");
            return Ok(());
        }

        tracing::info!("Applying {} replace blocks...", blocks.len());
        self.apply_blocks(&blocks)?;

        Ok(())
    }
}
