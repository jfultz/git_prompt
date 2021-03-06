extern crate git2;
// TODO: Investigate ansi_term crate.  https://crates.io/crates/ansi_term

use std::path::Path;
// use std::path::PathBuf;
use std::fmt::Write;

pub struct GitPromptRepo {
    lg2_repo: Option<git2::Repository>,
    // origin_path: PathBuf,
    // root_path: PathBuf,
    repo_state: git2::RepositoryState,
    is_unborn: bool,
    has_head: bool,
    has_checkout: bool,
}

impl GitPromptRepo {
    pub fn new(path_spec: &Path) -> GitPromptRepo {
        let lg2_repo = git2::Repository::discover(path_spec).ok();
        // let mut root_path = PathBuf::new();
        let mut repo_state = git2::RepositoryState::Clean;
        let mut is_unborn = false;
        let mut has_head = false;
        let mut has_checkout = false;

        if let Some(ref repo) = lg2_repo {
            // root_path = repo.path().to_path_buf();
            repo_state = repo.state();
            if let Err(e) = repo.head() {
                is_unborn = e.code() == git2::ErrorCode::UnbornBranch;
            } else {
                has_head = true;
            }
            has_checkout = !repo.is_bare();
        }

        GitPromptRepo {
            lg2_repo,
            // origin_path: path_spec.to_path_buf(),
            // root_path,
            repo_state,
            is_unborn,
            has_head,
            has_checkout,
        }
    }

    // Assumes has_head is true
    fn head_name(&self) -> String {
        self.lg2_repo
            .as_ref()
            .unwrap()
            .head()
            .unwrap()
            .name()
            .unwrap_or("")
            .trim_start_matches("refs/heads/")
            .to_string()
    }

    fn head_to_branch(&self) -> Option<git2::Branch> {
        if self.has_head {
            let head_name = self.head_name();
            return self
                .lg2_repo
                .as_ref()
                .unwrap()
                .find_branch(head_name.as_str(), git2::BranchType::Local)
                .ok();
        }
        None
    }

    fn head_branch_has_upstream(&self) -> bool {
        if let Some(branch) = self.head_to_branch() {
            return branch.upstream().is_ok();
        }
        false
    }

    // fn is_remote(&self) -> bool {
    //     match &self.ref_result {
    //         &Ok(ref r) => r.is_remote(),
    //         &Err(_) => false
    //     }
    // }

    fn build_ref_name_for_commit(&self, commit: &git2::Commit, ref_string: &mut String) {
        let oid = commit.id();
        let mut candidate_branch_names = Vec::new();
        let branches_it = self
            .lg2_repo
            .as_ref()
            .unwrap()
            .branches(Some(git2::BranchType::Remote))
            .unwrap()
            .filter_map(|branch_result| branch_result.ok());
        for branch in branches_it {
            let branch_ref = &branch.0.get();
            let peeled_obj_res = branch_ref.peel(git2::ObjectType::Commit);
            if peeled_obj_res.is_ok()
                && peeled_obj_res.unwrap().id() == oid
                && branch_ref.name().is_some()
            {
                candidate_branch_names.push(branch_ref.name().unwrap().to_string());
            }
        }
        ref_string.clear();
        if candidate_branch_names.is_empty() {
            let mut oid_string = String::new();
            write!(oid_string, "{}", oid).unwrap();
            oid_string.truncate(8);
            write!(ref_string, "{}", oid_string).unwrap();
        } else {
            write!(
                ref_string,
                "{}",
                find_best_branch_name(&candidate_branch_names)
            )
            .unwrap();
        }
    }

    pub fn ref_name_head(&self) -> String {
        let mut ref_string = String::new();
        // TODO: should check more repo_state possibilities
        if self.is_unborn {
            ref_string = "[Unborn]".to_string();
        } else if self.repo_state == git2::RepositoryState::Rebase
            || self.repo_state == git2::RepositoryState::RebaseInteractive
            || self.repo_state == git2::RepositoryState::RebaseMerge
        {
            // TODO: Should look at .git/rebase-apply/head-name rather than
            // compute rebasing_name as we do here
            if let Ok(rebasing_ref) = self
                .lg2_repo
                .as_ref()
                .unwrap()
                .find_reference("rebase-apply/orig-head")
            {
                let onto_ref = self
                    .lg2_repo
                    .as_ref()
                    .unwrap()
                    .find_reference("rebase-apply/onto")
                    .unwrap();
                let mut rebasing_name = String::new();
                let mut onto_name = String::new();
                self.build_ref_name_for_commit(
                    rebasing_ref
                        .peel(git2::ObjectType::Commit)
                        .unwrap()
                        .as_commit()
                        .unwrap(),
                    &mut rebasing_name,
                );
                self.build_ref_name_for_commit(
                    onto_ref
                        .peel(git2::ObjectType::Commit)
                        .unwrap()
                        .as_commit()
                        .unwrap(),
                    &mut onto_name,
                );
                write!(
                    ref_string,
                    "\x1b[1;35m...rebasing\x1b[0m {} \x1b[1;35monto\x1b[0m {}",
                    rebasing_name, onto_name
                )
                .unwrap();
            } else if self.lg2_repo.as_ref().unwrap().is_worktree() {
                write!(ref_string, "worktree rebase").unwrap();
            } else {
                write!(ref_string, "unhandled rebase case").unwrap();
            }
        } else if self.repo_state == git2::RepositoryState::Revert
            || self.repo_state == git2::RepositoryState::RevertSequence
        {
            write!(ref_string, "Reverting").unwrap();
        } else if self.repo_state == git2::RepositoryState::CherryPick
            || self.repo_state == git2::RepositoryState::CherryPickSequence
        {
            write!(ref_string, "Cherry-picking").unwrap();
        } else if self.repo_state == git2::RepositoryState::ApplyMailbox
            || self.repo_state == git2::RepositoryState::ApplyMailboxOrRebase
        {
            write!(ref_string, "Applying").unwrap();
        } else if self.repo_state == git2::RepositoryState::Merge {
            write!(ref_string, "Merging").unwrap();
        } else if self.repo_state == git2::RepositoryState::Bisect {
            write!(ref_string, "Bisecting").unwrap();
        } else if self.has_head {
            ref_string = self.head_name();
        }
        if ref_string == "HEAD" {
            let head_object = self
                .lg2_repo
                .as_ref()
                .unwrap()
                .head()
                .unwrap()
                .resolve()
                .unwrap()
                .peel(git2::ObjectType::Commit)
                .unwrap();
            let head_commit = head_object.as_commit().unwrap();
            self.build_ref_name_for_commit(head_commit, &mut ref_string);
        }
        ref_string.to_string()
    }

    pub fn upstream_name(&self) -> String {
        if self.head_branch_has_upstream() {
            let branch = self.head_to_branch().unwrap();
            let upstream_branch = branch.upstream().unwrap();
            if let Ok(Some(name)) = upstream_branch.name() {
                return name.to_string();
            }
        }
        String::new()
    }

    pub fn ahead_behind(&self) -> String {
        let mut result = String::new();
        if self.head_branch_has_upstream() {
            let head_branch = self.head_to_branch().unwrap();
            let upstream_branch = head_branch.upstream().unwrap();
            let head_branch_oid = head_branch.get().resolve().unwrap().target().unwrap();
            let upstream_branch_oid = upstream_branch.get().resolve().unwrap().target().unwrap();
            let repo = self.lg2_repo.as_ref().unwrap();
            let (ahead, behind) = repo
                .graph_ahead_behind(head_branch_oid, upstream_branch_oid)
                .unwrap();
            if behind > 0 {
                result += "↓·";
                result += &*behind.to_string();
            }
            if ahead > 0 {
                result += "↑·";
                result += &*ahead.to_string();
            }
        }
        result
    }

    pub fn status(&self) -> String {
        let mut result = String::new();
        if self.has_checkout {
            let mut opts = git2::StatusOptions::new();
            if let Ok(statuses) = self.lg2_repo.as_ref().unwrap().statuses(Some(
                opts.update_index(true)
                    .include_untracked(true)
                    .recurse_untracked_dirs(true),
            )) {
                result += &status_bit_to_string(&statuses, git2::Status::INDEX_MODIFIED, "∂");
                result += &status_bit_to_string(&statuses, git2::Status::INDEX_NEW, "…");
                result += &status_bit_to_string(&statuses, git2::Status::INDEX_DELETED, "✖");
                result += &status_bit_to_string(&statuses, git2::Status::CONFLICTED, "\x1b[31;1m≠");
                result += &status_bit_to_string(&statuses, git2::Status::WT_MODIFIED, "\x1b[34m∂");
                result += &status_bit_to_string(&statuses, git2::Status::WT_NEW, "\x1b[34m…");
                result += &status_bit_to_string(&statuses, git2::Status::WT_DELETED, "\x1b[34m✖");
            }
            if result.is_empty() {
                result = String::from("\x1b[36m√\x1b[m");
            }
        }
        result
    }
}

fn status_bit_to_string(statuses: &git2::Statuses, flag: git2::Status, prefix: &str) -> String {
    let count = statuses
        .iter()
        .filter(|s| s.status().contains(flag))
        .count();
    let mut result = String::new();
    if count > 0 {
        write!(result, "{}{}\x1b[m", prefix, count).unwrap();
    }
    result
}

fn abbreviated_remote_branch_name(full_name: &String) -> Option<String> {
    let mut full_name_it = full_name.split('/').peekable();
    if full_name_it.next() == Some("refs")
        && full_name_it.next() == Some("remotes")
        && full_name_it.peek().is_some()
    {
        let mut abbreviated_name = full_name_it.next().unwrap().to_string();
        if full_name_it.peek().unwrap_or(&"") == &"HEAD" {
            return None;
        }
        for s in full_name_it {
            abbreviated_name.push('/');
            abbreviated_name.push_str(s);
        }
        if !abbreviated_name.is_empty() {
            return Some(abbreviated_name);
        }
    }
    None
}

fn find_best_branch_name(branch_names: &[String]) -> String {
    // Transmute branch names into canonical shortened forms.
    // Also, remove all variants terminating in HEAD
    let working_branch_names: Vec<String> = branch_names
        .iter()
        .filter_map(abbreviated_remote_branch_name)
        .filter(|b| b != "HEAD")
        .collect();

    // Determine the minimum namespace depth of the names and
    // filter the list to include only those at the minimum depth.
    // The theory here is that we want to produce the "simplest" name,
    // and minimizing the depth should be done before, e.g., minimizing
    // the charcount
    let slash_counts: Vec<usize> = working_branch_names
        .iter()
        .map(|s| s.chars().filter(|c| c == &'/').count())
        .collect();
    let min_slashes = slash_counts.iter().min().unwrap();
    let branch_and_count_iter = working_branch_names.into_iter().zip(slash_counts.iter());
    let sifted_branch_names: Vec<String> = branch_and_count_iter
        .filter_map(|t| if t.1 == min_slashes { Some(t.0) } else { None })
        .collect();

    // Second tier preferences
    for candidate in &sifted_branch_names {
        if candidate == "origin/master" {
            return candidate.clone();
        }
    }
    for candidate in &sifted_branch_names {
        if candidate.starts_with("origin/release/") {
            return candidate.clone();
        }
    }
    sifted_branch_names[0].clone()
}
