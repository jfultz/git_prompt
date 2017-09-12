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
}


impl GitPromptRepo {
    pub fn new(path_spec: &Path) -> GitPromptRepo {
        let lg2_repo = git2::Repository::discover(path_spec).ok();
        let mut root_path = PathBuf::new();
        let mut is_unborn = false;
        let mut has_head = false;

        if let Some(ref repo) = lg2_repo {
            root_path = repo.path().to_path_buf();
            if let Err(e) = repo.head() {
                is_unborn = e.code() == git2::ErrorCode::UnbornBranch;
            } else {
                has_head = true;
            }
        }

        GitPromptRepo {
            lg2_repo: lg2_repo,
            origin_path: path_spec.to_path_buf(),
            root_path: root_path,
            is_unborn: is_unborn,
            has_head: has_head,
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

    pub fn ref_name(&self) -> String {
        let mut ref_string = "".to_string();
        if self.is_unborn {
            ref_string = "[Unborn]".to_string();
        } else if self.has_head {
            ref_string = self.head_name();
        }
        if ref_string == "HEAD" {
            let head_reference = self.lg2_repo
                .as_ref()
                .unwrap()
                .head()
                .unwrap()
                .resolve()
                .unwrap();
            let head_oid = head_reference.target().unwrap();
            ref_string.clear();
            write!(ref_string, "{}", head_oid).unwrap();
            ref_string.truncate(8);
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
}
