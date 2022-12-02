use duct::cmd;
use git2::{BranchType, Repository};
use serde::{Deserialize, Serialize};

pub struct GitApp {
    pub repo: Repository,
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct GitConfig {
    pub user_name: String,
    pub user_email: String,
    pub remote_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct GitBranchItem {
    pub name: String,
    pub is_local: bool,
}

#[derive(Serialize, Deserialize)]
pub struct GitBranchInfo {
    pub current: String,
    pub local_list: Vec<GitBranchItem>,
    pub remote_list: Vec<GitBranchItem>,
}

#[derive(Serialize, Deserialize)]
pub struct GitCheckParams {
    // 开发者
    pub developer: String,
    // 分支
    pub branch: String,
    pub remote_list: Vec<GitBranchItem>,
}

#[derive(Serialize, Deserialize)]
pub struct GitChangeLines {
    pub add_lines: i32,
    pub subtract_lines: i32,
    pub total_lines: i32,
}

#[derive(Serialize, Deserialize)]
pub struct GitChangeLinesParams {
    pub branch: String,
    // pub compare: String,
    pub start: String,
    pub end: String,
    pub excludes: Vec<String>,
    pub author: String,
}

impl GitApp {
    pub fn new(path: String) -> GitApp {
        let target = path.to_string();
        let repo = match Repository::open(path) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to open: {}", e),
        };
        GitApp { repo, path: target }
    }
    // 获取配置信息
    pub fn get_config(&self) -> GitConfig {
        let config = self.repo.config().unwrap();
        let name = config.get_string("user.name").unwrap();
        let email = config.get_string("user.email").unwrap();
        let remote = self.repo.find_remote("origin").unwrap();
        let mut url: String = "".to_string();
        match remote.url() {
            Some(s) => url = s.to_string(),
            None => println!("remote 匹配错误"),
        }
        GitConfig {
            user_name: name.to_string(),
            user_email: email.to_string(),
            remote_url: url,
        }
    }
    // 获取分支信息，本地分支、远程分支、当前分支
    pub fn get_branch_info(&self) -> GitBranchInfo {
        let local_list = self.get_branches_by_type(BranchType::Local);
        let remote_list = self.get_branches_by_type(BranchType::Remote);
        let current = cmd!("git", "branch")
            .dir(&self.path)
            .before_spawn(|cmd| {
                cmd.arg("--show-current");
                Ok(())
            })
            .read()
            .unwrap();
        GitBranchInfo {
            current,
            local_list,
            remote_list,
        }
    }

    pub fn get_branches_by_type(&self, branch_type: BranchType) -> Vec<GitBranchItem> {
        let result = self.repo.branches(Some(branch_type)).unwrap();
        let mut items: Vec<GitBranchItem> = Vec::new();
        result.into_iter().for_each(|item| {
            let (branch, branch_type) = item.unwrap();
            items.push(GitBranchItem {
                name: branch.name().unwrap().unwrap().to_string(),
                is_local: branch_type.eq(&BranchType::Local),
            });
        });
        items
    }
}
