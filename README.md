### AtCoder Tool Kit

#### 使い方
以下の1 ~ 3の使い方があります。

1. `atk new abcxxx` (xxx はコンテストの数字) でカレントディレクトリ内に /abcxxxを作成し，その中にa.cpp ~ g.cppを作成します。
各ファイルにはテンプレートが書き込まれています。(src/main.rs内のコードで変更可能)
abc, arc, agc を選択可能です。
(ahcも選択可能だが、ヒューリスティック用の構成になってないので注意)
2. `atk test x` (x はa ~ g) で選択されたファイルのテストを実行します。
/abcxxx 等のディレクトリで実行されることを想定しています。
(例えば、カレントディレクトリが \~/atcoderで実行した場合エラーを吐きます、\~/atcoder/abcxxxで実行してください)
3. `atk copy x` (x はa ~ g) で選択されたファイルをクリップボードにコピーします。

#### 導入方法
Rust環境とxclipが必要です。
Rustは[公式サイト](https://www.rust-lang.org/tools/install)からインストールできます。
xclipは`sudo apt update && sudo apt install xclip`でインストールできます。

`git clone https://github.com/zerozero-0-0/AtCoder-Tool-Kit.git`で当プロジェクトをローカル環境にクローンしてください。
`src/main.rs` を上から見ていくと、定数CPP_TERMPLATEがあるので、必要に応じて変更してください。
(r#" ~ "# の ~ の部分を編集してください)

`cargo install --path .` 
`atk --version` で　`atk 0.1.0` のように表示されれば成功です。
