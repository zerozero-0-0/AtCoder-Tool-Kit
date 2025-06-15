use anyhow::Ok;
use clap::{Parser, Subcommand};
use colored::Colorize;
use reqwest::Client;
use scraper::{Html, Selector};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::{
    env::{self, current_dir},
    process::Stdio,
    u32,
};

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
    New { contest_name: String },
    Test { problem_char: String },
    Copy { problem_char: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { contest_name } => {
            let contest_id = format_contest_id(&contest_name)?;
            println!("新しいコンテストセット {} を作成します", contest_id);
            create_contest_directory(contest_id)?;
        }
        Commands::Test { problem_char } => {
            let current_path = env::current_dir()?;
            let contest_id = extract_contest_id_from_path(&current_path)?;
            run_tests(contest_id, problem_char).await?;
        }
        Commands::Copy { problem_char } => {
            let current_path = env::current_dir()?;
            println!("現在のディレクトリ: {}", current_path.display());
            let contest_id = extract_contest_id_from_path(&current_path)?;
            println!("コンテストID: {}", contest_id);
            copy_problem_template(&contest_id, problem_char)?;
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
    if input_id.len() == 6
        && (input_id.starts_with("abc")
            || input_id.starts_with("arc")
            || input_id.starts_with("agc")
            || input_id.starts_with("ahc"))
    {
        return Ok(input_id.to_string());
    }

    if input_id.len() < 4 || input_id.len() > 6 {
        return Err(anyhow::anyhow!(
            "コンテストIDは4〜6文字でなければなりません"
        ));
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
        return Err(anyhow::anyhow!(
            "コンテストIDはabc, arc, agc, ahcのいずれかで始まる必要があります"
        ));
    };

    let contest_num = num_str
        .parse::<u32>()
        .map_err(|_| anyhow::anyhow!("コンテスト番号は数字でなければなりません"))?;

    return Ok(format!("{}{:03}", prefix, contest_num));
}

fn extract_contest_id_from_path(path: &PathBuf) -> anyhow::Result<String> {
    /*
    現在のディレクトリ名が正しいパスであることを確認する
    args:
        path: 現在のディレクトリのパス
    returns:
        正しい形式のコンテストID
    */
    let dir_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("不正なディレクトリ名です"))?;

    let is_valid_contest_id = dir_name.len() == 6
        && (dir_name.starts_with("abc")
            || dir_name.starts_with("arc")
            || dir_name.starts_with("agc")
            || dir_name.starts_with("ahc"))
        && dir_name[3..].parse::<u32>().is_ok();

    if is_valid_contest_id {
        Ok(dir_name.to_string())
    } else {
        Err(anyhow::anyhow!(
            "現在のディレクトリ名が正しいコンテストIDではありません"
        ))
    }
}

fn create_contest_directory(contest_name: String) -> anyhow::Result<()> {
    let contest_dir = PathBuf::from(contest_name);

    if contest_dir.exists() {
        return Err(anyhow::anyhow!(
            "{} はすでに存在します",
            contest_dir.display()
        ));
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

#[derive(Debug)]
struct TestCase {
    input: String,
    output: String,
}

async fn get_sample_cases(
    contest_id: String,
    problem_char: &String,
) -> anyhow::Result<Vec<TestCase>> {
    /*
    指定されたコンテストIDと問題文字に基づいて、AtCoderのサンプルケースを取得する
    args:
        contest_id: コンテストID (例: "abc123")
        problem_char: 問題文字 (例: "a", "b", "c" など)
    returns:
        サンプルケースのリスト
    */
    let url = format!(
        "https://atcoder.jp/contests/{}/tasks/{}_{}",
        contest_id, contest_id, problem_char
    );

    let client = Client::new();
    let res = client.get(&url).send().await?;
    if !res.status().is_success() {
        return Err(anyhow::anyhow!(
            "HTTPリクエストが失敗しました: {}",
            res.status()
        ));
    }

    let body = res.text().await?;

    let document = Html::parse_document(&body);

    let h3_selector = Selector::parse("h3").unwrap();
    let pre_selector = Selector::parse("pre").unwrap();

    let mut samples = Vec::new();
    let mut h3s = document.select(&h3_selector).peekable();
    let mut pre_iter = document.select(&pre_selector);
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    // h3タグを順に見ていき、入力例/出力例の直後のpreを取得
    while let Some(h3) = h3s.next() {
        let h3_text = h3.text().collect::<String>();
        if h3_text.contains("入力例") {
            if let Some(pre) = h3
                .next_sibling()
                .and_then(|n| scraper::ElementRef::wrap(n))
                .filter(|e| e.value().name() == "pre")
            {
                inputs.push(pre.text().collect::<Vec<_>>().join(""));
            } else if let Some(pre) = pre_iter.next() {
                // fallback: preタグの順番で取得
                inputs.push(pre.text().collect::<Vec<_>>().join(""));
            }
        } else if h3_text.contains("出力例") {
            if let Some(pre) = h3
                .next_sibling()
                .and_then(|n| scraper::ElementRef::wrap(n))
                .filter(|e| e.value().name() == "pre")
            {
                outputs.push(pre.text().collect::<Vec<_>>().join(""));
            } else if let Some(pre) = pre_iter.next() {
                outputs.push(pre.text().collect::<Vec<_>>().join(""));
            }
        }
    }
    let n = std::cmp::min(inputs.len(), outputs.len());
    for i in 0..n {
        samples.push(TestCase {
            input: inputs[i].clone(),
            output: outputs[i].clone(),
        });
    }

    Ok(samples)
}

async fn get_sample_cases_cached(
    contest_id: String,
    problem_char: &String,
) -> anyhow::Result<Vec<TestCase>> {
    /*
    キャッシュされたテストケースを取得する
    args:
        contest_id: コンテストID (例: "abc123")
        problem_char: 問題文字 (例: "a", "b", "c" など)
    returns:
        サンプルケースのリスト
    */

    // テストケース保存ディレクトリ
    let test_dir = PathBuf::from("test").join(problem_char);
    if test_dir.exists() {
        // 既存のテストケースを読み込む
        let mut cases = Vec::new();
        let mut idx = 1;
        loop {
            let in_path = test_dir.join(format!("in{}.txt", idx));
            let out_path = test_dir.join(format!("out{}.txt", idx));
            if !in_path.exists() || !out_path.exists() {
                break;
            }
            let input = fs::read_to_string(&in_path)?;
            let output = fs::read_to_string(&out_path)?;
            cases.push(TestCase { input, output });
            idx += 1;
        }
        if !cases.is_empty() {
            return Ok(cases);
        }
    }
    // なければWebから取得し保存
    let cases = get_sample_cases(contest_id, problem_char).await?;
    if !cases.is_empty() {
        fs::create_dir_all(&test_dir)?;
        for (i, case) in cases.iter().enumerate() {
            let in_path = test_dir.join(format!("in{}.txt", i + 1));
            let out_path = test_dir.join(format!("out{}.txt", i + 1));
            fs::write(in_path, &case.input)?;
            fs::write(out_path, &case.output)?;
        }
    }
    Ok(cases)
}

async fn run_tests(contest_id: String, problem_char: String) -> anyhow::Result<()> {
    /*
    指定されたコンテストIDと問題文字に基づいて、AtCoderのサンプルケースを取得し、ローカルでテストを実行する
    args:
        contest_id: コンテストID (例: "abc123")
        problem_char: 問題文字 (例: "a", "b", "c" など)
    */

    let problem_file = format!("{}.cpp", &problem_char);
    let problem_path = env::current_dir()?.join(problem_file);

    if !problem_path.exists() {
        return Err(anyhow::anyhow!("{} が存在しません", problem_path.display()));
    }

    let samples = get_sample_cases_cached(contest_id, &problem_char).await?;
    if samples.is_empty() {
        return Err(anyhow::anyhow!("サンプルケースが見つかりませんでした"));
    }

    let executable_name = format!("{}.out", problem_char);
    let executable_path = current_dir()?.join(executable_name);

    let compile_output = std::process::Command::new("g++")
        .arg(problem_path)
        .arg("-o")
        .arg(&executable_path)
        .output()
        .expect("コンパイルに失敗しました");

    if !compile_output.status.success() {
        eprintln!(
            "コンパイルエラー:\n{}",
            String::from_utf8_lossy(&compile_output.stderr)
        );
        return Err(anyhow::anyhow!("コンパイルに失敗しました"));
    }

    for (i, sample) in samples.iter().enumerate() {
        println!(" --- Running test case {} ---", i + 1);

        let mut test_output = std::process::Command::new(&executable_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        {
            let child_stdin = test_output.stdin.as_mut().unwrap();
            child_stdin.write_all(sample.input.as_bytes())?;
        } // 標準入力を閉じる

        let output = test_output.wait_with_output()?;

        if !output.status.success() {
            eprintln!("{}", "    Runtime error ".red().bold());
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            continue;
        }

        let acutual_output = String::from_utf8_lossy(&output.stdout).to_string();

        let normalized_actual = acutual_output
            .trim_end()
            .split('\n')
            .map(|s| s.trim_end())
            .collect::<Vec<_>>()
            .join("\n");
        let normalized_expected = sample
            .output
            .trim_end()
            .split('\n')
            .map(|s| s.trim_end())
            .collect::<Vec<_>>()
            .join("\n");

        if normalized_actual == normalized_expected {
            println!(
                " --- Test Case {}: {} ---",
                (i + 1).to_string().green(),
                "AC".green().bold()
            );
        } else {
            println!(
                " --- Test Case {}: {} ---",
                (i + 1).to_string().yellow(),
                "WA".red().bold()
            );
            println!("Expected:\n{}", normalized_expected);
            println!("Actual:\n{}", normalized_actual);
        }
    }

    fs::remove_file(&executable_path)?;
    return Ok(());
}

fn copy_problem_template(_contest_id: &str, problem_char: String) -> anyhow::Result<()> {
    // 問題ファイル名
    let file_name = format!("{}.cpp", problem_char);
    let file_path = PathBuf::from(&file_name);
    if !file_path.exists() {
        return Err(anyhow::anyhow!("{} が存在しません", file_path.display()));
    }
    let code = fs::read_to_string(&file_path)?;
    // xclipでクリップボードにコピー
    use std::process::Command;
    let mut child = Command::new("xclip")
        .arg("-selection")
        .arg("clipboard")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| anyhow::anyhow!("xclipの起動に失敗: {}", e))?;
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("xclipのstdin取得に失敗"))?;
        stdin.write_all(code.as_bytes())?;
    }
    let status = child.wait()?;
    if !status.success() {
        return Err(anyhow::anyhow!(
            "xclipによるクリップボードコピーに失敗しました"
        ));
    }
    println!("{} をクリップボードにコピーしました", file_name);
    Ok(())
}
