use std::{env, path::{self, Path}, u32};
use anyhow::Ok;
use std::fs;
use std::path::PathBuf;

use clap::{builder::Str, Parser, Subcommand};

const CPP_TEMPLATE: &str = r#"#include <iostream>
using namespace std;

int main()
{

}
"#;

#[derive(Parser, Debug)]
#[command(
    author = "Zrzr",
    version = "0.1.0",
    about = "",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    New {
        contest_name: String,
    },
    Test {
        problem_char: String,
    },
    Copy {
        problem_char: String
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { contest_name } => {
            println!("新しいコンテストセット {} を作成します", contest_name);
            create_contest_directory(contest_name)?;
        }
        Commands::Test { problem_char } => {
            let current_path = env::current_dir()?;
            println!("現在のディレクトリ: {}", current_path.display());
            let contest_id = extract_contest_id_from_path(&current_path)?;
            println!("コンテストID: {}", contest_id);
            // run_tests(&contest_id, problem_char)?;
        }
        Commands::Copy { problem_char } => {
            let current_path = env::current_dir()?;
            println!("現在のディレクトリ: {}", current_path.display());
            let contest_id = extract_contest_id_from_path(&current_path)?;
            println!("コンテストID: {}", contest_id);
            // copy_problem_template(&contest_id, problem_char)?;
        }
    }
    return Ok(());
}

fn format_contest_id(input_id: &String) -> anyhow::Result<String> {
    /* 
    // ユーザーの入力値を正しい形式に変換する
    // [abc or arc or agc or ahc] + [3桁の数字]
    // args:
    //     input_id: ユーザーが入力したコンテストID
    // returns:
    //     正しい形式のコンテストID
    */
    if input_id.len() == 6 && (input_id.starts_with("abc") || input_id.starts_with("arc") || input_id.starts_with("agc") || input_id.starts_with("ahc")) {
        return Ok(input_id.to_string());
    }

    if input_id.len() < 4 || input_id.len() > 6 {
        return Err(anyhow::anyhow!("コンテストIDは4〜6文字でなければなりません"));
    }

    let (prefix, num_str) = if input_id.starts_with("abc") {
        ("abc", &input_id[3..])
    } else if input_id.starts_with("arc") {
        ("arc", &input_id[3..])
    } else if input_id.starts_with("agc") {
        ("agc", &input_id[3..])
    } else if input_id.starts_with("ahc") {
        ("ahc", &input_id[3..])
    } else {
        return Err(anyhow::anyhow!("コンテストIDはabc, arc, agc, ahcのいずれかで始まる必要があります"));
    };

    let contest_num = num_str.parse::<u32>().map_err(|_| {
        anyhow::anyhow!("コンテスト番号は数字でなければなりません")
    })?;

    return Ok(format!("{}{:03}", prefix, contest_num));
}

fn extract_contest_id_from_path(path: &PathBuf) -> anyhow::Result<String> {
    /*
    現在のディレクトリ名が正しいパスであることを確認する
    args:
        path: 現在のディレクトリのパス
    */
    let dir_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("不正なディレクトリ名である"))?;

    let is_valid_contest_id = dir_name.len() == 6 && (dir_name.starts_with("abc") || dir_name.starts_with("arc") || dir_name.starts_with("agc") || dir_name.starts_with("ahc")) && dir_name[3..].parse::<u32>().is_ok();

    if is_valid_contest_id {
        Ok(dir_name.to_string())
    } else {
        Err(anyhow::anyhow!("現在のディレクトリ名が正しいコンテストIDではありません"))
    }
}

fn create_contest_directory(contest_name: String) -> anyhow::Result<()> {
    let contest_dir = PathBuf::from(contest_name);

    if contest_dir.exists() {
        return Err(anyhow::anyhow!("{} はすでに存在します", contest_dir.display()));
    } else {
        println!("{} を作成します", contest_dir.display());
        fs::create_dir_all(&contest_dir)?;
    }

    let problems = ['a', 'b', 'c', 'd', 'e', 'f', 'g'];

    for problem in problems {
        let file_name = format!("{}.cpp", problem);
        let file_path = contest_dir.join(file_name);

        if file_path.exists() {
            println!("{} はすでに存在します", file_path.display());
            continue;
        }

        println!("{} を作成します", file_path.display());
        let mut file = fs::File::create(&file_path)?;
        use std::io::Write;
        file.write_all(CPP_TEMPLATE.as_bytes())?;        
    }

    let test_dir = contest_dir.join("test");

    if test_dir.exists() {
        return Err(anyhow::anyhow!("{} はすでに存在します", test_dir.display()));
    } else {
        println!("{} を作成します", test_dir.display());
        fs::create_dir_all(&test_dir)?;
    }

    return Ok(());
}
