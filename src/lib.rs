extern crate git2;

use std::path::Path;
use std::path::PathBuf;
use std::fmt::Write;

pub struct GitPromptRepo {
    lg2_repo: Option<git2::Repository>,
    origin_path: PathBuf,
    root_path: PathBuf,
    is_unborn: bool,
    has_head: bool,
    has_checkout: bool,
}


impl GitPromptRepo {
    pub fn new(path_spec: &Path) -> GitPromptRepo {
        let lg2_repo = git2::Repository::discover(path_spec).ok();
        let mut root_path = PathBuf::new();
        let mut is_unborn = false;
        let mut has_head = false;
        let mut has_checkout = false;

        if let Some(ref repo) = lg2_repo {
            root_path = repo.path().to_path_buf();
            if let Err(e) = repo.head() {
                is_unborn = e.code() == git2::ErrorCode::UnbornBranch;
            } else {
                has_head = true;
            }
            has_checkout = !repo.is_bare();
        }

        GitPromptRepo {
            lg2_repo: lg2_repo,
            origin_path: path_spec.to_path_buf(),
            root_path: root_path,
            is_unborn: is_unborn,
            has_head: has_head,
            has_checkout: has_checkout,
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
            .trim_left_matches("refs/heads/")
            .to_string()
    }

    fn head_to_branch(&self) -> Option<git2::Branch> {
        if self.has_head {
            let head_name = self.head_name();
            return self.lg2_repo
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
        let branches_it = self.lg2_repo
            .as_ref()
            .unwrap()
            .branches(Some(git2::BranchType::Remote))
            .unwrap()
            .filter_map(|branch_result| branch_result.ok());
        for branch in branches_it {
            let ref branch_ref = branch.0.get();
            let peeled_obj_res = branch_ref.peel(git2::ObjectType::Commit);
            if peeled_obj_res.is_ok() && peeled_obj_res.unwrap().id() == oid
                && branch_ref.name().is_some()
            {
                candidate_branch_names.push(branch_ref.name().unwrap().to_string());
            }
        }
        ref_string.clear();
        if candidate_branch_names.is_empty() {
            let mut oid_string = "".to_string();
            write!(oid_string, "{}", oid).unwrap();
            oid_string.truncate(8);
            write!(ref_string, "{}", oid_string).unwrap();
        } else {
            write!(
                ref_string,
                "{}",
                find_best_branch_name(candidate_branch_names)
            ).unwrap();
        }
    }

    pub fn ref_name_head(&self) -> String {
        let mut ref_string = "".to_string();
        if self.is_unborn {
            ref_string = "[Unborn]".to_string();
        } else if self.has_head {
            ref_string = self.head_name();
        }
        if ref_string == "HEAD" {
            let head_object = self.lg2_repo
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
        "".to_string()
    }

    pub fn ahead_behind(&self) -> String {
        let mut result = String::from("");
        if self.head_branch_has_upstream() {
            let head_branch = self.head_to_branch().unwrap();
            let upstream_branch = head_branch.upstream().unwrap();
            let head_branch_oid = head_branch.get().resolve().unwrap().target().unwrap();
            let upstream_branch_oid = upstream_branch.get().resolve().unwrap().target().unwrap();
            let repo = self.lg2_repo.as_ref().unwrap();
            let (ahead, behind) = repo.graph_ahead_behind(head_branch_oid, upstream_branch_oid)
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
        let mut result = String::from("");
        if self.has_checkout {
            let mut opts = git2::StatusOptions::new();
            if let Ok(statuses) = self.lg2_repo.as_ref().unwrap().statuses(Some(
                opts.update_index(true)
                    .include_untracked(true)
                    .recurse_untracked_dirs(true),
            )) {
                result += &status_bit_to_string(&statuses, git2::STATUS_INDEX_MODIFIED, "∂");
                result += &status_bit_to_string(&statuses, git2::STATUS_INDEX_NEW, "…");
                result += &status_bit_to_string(&statuses, git2::STATUS_INDEX_DELETED, "✖");
                result +=
                    &status_bit_to_string(&statuses, git2::STATUS_CONFLICTED, "\x1b[31;1m≠");
                result += &status_bit_to_string(&statuses, git2::STATUS_WT_MODIFIED, "\x1b[34m∂");
                result += &status_bit_to_string(&statuses, git2::STATUS_WT_NEW, "\x1b[34m…");
                result += &status_bit_to_string(&statuses, git2::STATUS_WT_DELETED, "\x1b[34m✖");
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
    let mut result = "".to_string();
    if count > 0 {
        write!(result, "{}{}\x1b[m", prefix, count).unwrap();
    }
    result
}

fn abbreviated_remote_branch_name(full_name: &String) -> Option<String> {
    let mut full_name_it = full_name.split('/').peekable();
    if full_name_it.next() == Some("refs") && full_name_it.next() == Some("remotes")
        && full_name_it.peek().is_some()
    {
        let mut abbreviated_name = full_name_it.next().unwrap().to_string();
        if full_name_it.peek().unwrap_or(&"").to_string() == "HEAD".to_string() {
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

fn find_best_branch_name(branch_names: Vec<String>) -> String {
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
    sifted_branch_names[0].clone()
}
