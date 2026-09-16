//! What one review needs to talk to an agent: the templates, the project
//! rules, canon for the quoted siblings and glossary, and the directory
//! the prompt files go in.

use super::files::{self, prompt_path, write_prompt};
use super::prompt::{self, PromptContext};
use super::{assistant, parse_hints, Assist, AssistError, Assistant, Hint};
use crate::glossary::Glossary;
use crate::model::Canon;
use crate::review::pair::requirement_text;
use crate::review::Pairing;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub struct Session {
    pub root: PathBuf,
    pub canon: Canon,
    /// The project's glossary; empty when it has no `definitions`
    /// capability, and then the prompts have no glossary section.
    pub glossary: Glossary,
    pub rules: Vec<String>,
    pub assist: Option<Assist>,
    /// Where prompt files are written: the state directory of the change,
    /// or a temporary directory under `--no-state`.
    pub prompt_dir: PathBuf,
    /// The search path the agent binary is looked up in.
    pub search_path: Option<OsString>,
}

impl Session {
    /// Read everything from the working tree once, at the start of a run.
    /// Canon, the configuration and the glossary all degrade to empty: a
    /// prompt is never worth failing a review over, and a configuration
    /// the review itself could not read has already been reported.
    pub fn open(root: &Path, prompt_dir: PathBuf) -> Session {
        let canon = crate::source::load_canon(root).unwrap_or_default();
        let config = crate::citations::read_config(root).ok().flatten();
        let capability = config
            .as_ref()
            .map(|c| c.definitions.capability.clone())
            .unwrap_or_else(|| crate::glossary::DEFAULT_CAPABILITY.to_string());
        let glossary = Glossary::build(&canon, &[], &capability);
        Session {
            root: root.to_path_buf(),
            canon,
            glossary,
            rules: files::spec_rules(root),
            assist: config.and_then(|c| c.assist),
            prompt_dir,
            search_path: std::env::var_os("PATH"),
        }
    }

    /// The glossary the review built, which also holds the terms the
    /// change under review adds.
    pub fn with_glossary(mut self, glossary: Glossary) -> Session {
        self.glossary = glossary;
        self
    }

    pub fn assistant(&self) -> Result<Box<dyn Assistant>, AssistError> {
        assistant(self.assist.as_ref(), self.search_path.as_ref())
    }

    /// Whether an agent could run at all, so a key press can say why not
    /// before anything is written.
    pub fn check(&self) -> Result<(), AssistError> {
        self.assistant().map(|_| ())
    }

    fn context(&self, p: &Pairing, batch: bool) -> PromptContext {
        let after = p
            .after
            .as_ref()
            .or(p.before.as_ref())
            .map(requirement_text)
            .unwrap_or_default();
        PromptContext {
            rules: self.rules.clone(),
            siblings: prompt::sibling_texts(p, &self.canon),
            glossary: prompt::glossary_terms(&self.glossary, &after),
            schema: batch.then(|| files::load_template(&self.root, "hints.md")),
        }
    }

    pub fn pairing_text(&self, p: &Pairing, batch: bool) -> String {
        let template = files::load_template(&self.root, "pairing.md");
        prompt::pairing_prompt(&template, p, &self.context(p, batch))
    }

    pub fn change_text(
        &self,
        change: &str,
        proposal: Option<&str>,
        pairings: &[&Pairing],
        batch: bool,
    ) -> String {
        let template = files::load_template(&self.root, "change.md");
        let schema = batch.then(|| files::load_template(&self.root, "hints.md"));
        let with_context: Vec<(&Pairing, PromptContext)> = pairings
            .iter()
            .map(|p| {
                let mut context = self.context(p, false);
                context.rules.clear();
                (*p, context)
            })
            .collect();
        prompt::change_prompt(
            &template,
            change,
            proposal,
            &self.rules,
            &with_context,
            schema.as_deref(),
        )
    }

    /// The prompt file for one pairing, written where the adapter can read
    /// it and the reviewer can look at it afterwards.
    pub fn write_pairing_prompt(&self, p: &Pairing, batch: bool) -> Result<PathBuf, AssistError> {
        let path = prompt_path(&self.prompt_dir, &p.change, &p.key());
        write_prompt(&path, &self.pairing_text(p, batch))?;
        Ok(path)
    }

    pub fn write_change_prompt(
        &self,
        change: &str,
        proposal: Option<&str>,
        pairings: &[&Pairing],
        batch: bool,
    ) -> Result<PathBuf, AssistError> {
        let path = prompt_path(&self.prompt_dir, change, "change");
        write_prompt(&path, &self.change_text(change, proposal, pairings, batch))?;
        Ok(path)
    }

    /// One agent call on one pairing: write the prompt, run it, parse the
    /// reply.
    pub fn review_pairing(
        &self,
        agent: &dyn Assistant,
        p: &Pairing,
    ) -> Result<Vec<Hint>, AssistError> {
        let path = self.write_pairing_prompt(p, true)?;
        Ok(parse_hints(&agent.review(&path)?))
    }
}
