use git2::{Repository, Oid, Signature};
use anyhow::Result;
use std::path::Path;

pub struct GitContext {
    repo: Repository,
}

impl GitContext {
    /// Opens an existing repository at the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = Repository::open(path)?;
        Ok(Self { repo })
    }

    /// Checks if the repo has uncommitted changes
    pub fn has_uncommitted_changes(&self) -> Result<bool> {
        let statuses = self.repo.statuses(None)?;
        Ok(!statuses.is_empty())
    }

    /// Creates a new branch from HEAD and checks it out
    pub fn create_and_checkout_branch(&self, branch_name: &str) -> Result<()> {
        let head = self.repo.head()?.peel_to_commit()?;

        let branch = self.repo.branch(branch_name, &head, false)?;

        let obj = branch.get().peel(git2::ObjectType::Commit)?;
        self.repo.checkout_tree(&obj, None)?;
        self.repo.set_head(branch.get().name().unwrap())?;

        Ok(())
    }

    /// Adds all changes and commits them with the given message
    pub fn commit_all(&self, message: &str) -> Result<Oid> {
        let mut index = self.repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        let oid = index.write_tree()?;
        let signature = Signature::now("Raider Agent", "agent@raider.local")?;
        let tree = self.repo.find_tree(oid)?;
        let head = self.repo.head()?.peel_to_commit()?;

        let commit_oid = self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&head],
        )?;

        Ok(commit_oid)
    }
}
