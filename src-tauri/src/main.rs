#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
mod git;

use duct::cmd;
use git::{GitApp, GitBranchInfo, GitChangeLinesParams, GitConfig};
use std::path::Path;
use tauri::{Manager};

fn main() {
    // 解决路径问题，No such file or directory
    fix_path_env::fix().unwrap();
    // 内容
    let context = tauri::generate_context!();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            is_git,
            get_config,
            get_branche_info,
            get_developer,
            calc_change_lines,
            get_branch_create_date,
        ])
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }
            Ok(())
        })
        .run(context)
        .expect("启动iCalcLines失败");
}

// git 配置
#[tauri::command]
fn get_config(path: String) -> GitConfig {
    let git = GitApp::new(path);
    git.get_config()
}

// 获取分支信息
#[tauri::command]
fn get_branche_info(path: String) -> GitBranchInfo {
    let git = GitApp::new(path);
    git.get_branch_info()
}

// 判断是否是 git 项目
#[tauri::command]
fn is_git(path: String) -> bool {
    let mut url = path.to_string();
    url.push_str("/.git");
    let is_folder = Path::new(&url).exists();
    is_folder
}

// 获取开发者
#[tauri::command]
fn get_developer(path: String) -> Vec<String> {
    let target = cmd!("git", "log")
        .dir(path)
        .pipe(cmd!("grep", "Author:"))
        .pipe(cmd!("sort"))
        .pipe(cmd!("uniq"))
        .read()
        .unwrap();
    let val = target.split("Author:").collect::<Vec<&str>>();
    let mut items: Vec<String> = Vec::new();
    for item in val.iter() {
        if item.is_empty() {
            continue;
        }
        items.push(item.to_string().replace(" ", ""));
    }
    items
}
// 获取创建时间
#[tauri::command]
fn get_branch_create_date(path: String, branch: String) -> String {
    let target = cmd!("git", "reflog")
        .dir(path)
        .before_spawn(move |cmd| {
            cmd.arg("show");
            cmd.arg("--date=iso");
            cmd.arg(&branch);
            Ok(())
        })
        .read()
        .unwrap();
    let val = target.split("\n").collect::<Vec<&str>>();
    let mut date_str: String = "".to_string();
    for item in val.iter() {
        if item.contains("branch: Created") {
            date_str = item.to_string();
            break;
        }
    }
    date_str
}

// 计算某个人的提交行数
#[tauri::command]
fn calc_change_lines(path: String, params: GitChangeLinesParams) -> String {
    let GitChangeLinesParams {
        branch,
        // compare,
        start,
        end,
        excludes,
        author,
    } = params;
    let mut target = cmd!("git", "log").dir(path).before_spawn(move |cmd| {
        // 分支处理
        cmd.arg(&branch);
        // 默认参数
        cmd.arg("--pretty=tformat:");
        cmd.arg("--numstat");
        // 开发者处理
        if !author.is_empty() {
            let mut author_arg = "--author=".to_string();
            author_arg += &author;
            cmd.arg(author_arg);
        }
        // 处理开始&结束时间
        if !start.is_empty() {
            let mut start_val = "--since=".to_string();
            start_val += &start.to_string();
            cmd.arg(start_val);
        }
        if !end.is_empty() {
            let mut end_val = "--until=".to_string();
            end_val += &end.to_string();
            cmd.arg(end_val);
        }
        Ok(())
    });
    for item in excludes {
        if !item.is_empty() {
            target = target.pipe(cmd!("grep", "-v").before_spawn(move |cmd| {
                cmd.arg(&item);
                Ok(())
            }));
        }
    }

    let str = target.pipe(cmd!("awk", "{ add += $1 ; subs += $2 ; loc += $1 + $2 } END { printf \"add_lines:%s,subtract_lines:%s,total_lines:%s\",add,subs,loc }"))
        .read()
        .unwrap();
    str
}
