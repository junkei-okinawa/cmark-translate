# cmark-translate

Translate CommonMark/Markdown file using DeepL API.

forked: https://github.com/hanabu/cmark-translate

# Internet Computer developer portal の DeepL 自動翻訳
DeepL で [portal](https://github.com/dfinity/portal) の markdown を日本語へ自動翻訳します。
原文と翻訳が並んで出力されます（翻訳の質が悪い場合に、原文を読めるように）。

# 前提条件
## 前提条件１
```
$ rustc --version
rustc 1.67.0 (fc594f156 2023-01-24)
```
バージョンは、いくつでも動くかもしれない。少なくとも、上記バージョンで動作する。

## 前提条件２
[portal](https://github.com/dfinity/portal) の README に従って、npm start してブラウザで http://localhost:3000/ を確認できる状態だとベスト。最低限 git clone git@github.com:dfinity/portal.git は必要。
```
$ git clone git@github.com:dfinity/portal.git
$ cd portal/
$ git submodule update --init
$ npm install
$ npm start
```

## 前提条件３
DeepL API の APIキーを発行済み。

# 事前準備
1. git clone

カレントディレクトリは任意。
```bash
git clone https://github.com/junkei-okinawa/cmark-translate.git
cd cmark-translate
```

2. DeepL API の設定

`deepl.sample.toml`のファイル名を`deepl.toml`に変更して`"your_DeepL_api_key"`部分に前提条件３で発行したDeepLのAPIキーを入力

API KEY の末尾が`:fx`で終わる文字列の場合はfreeプランと判断し、エンドポイントを切り替えています。

```toml
api_key = "your_DeepL_api_key"
# 例)
api_key = "123ab456-7c89-0dfg-12hi-jk345l67l890:fx"
```
3. 辞書設定

辞書機能を使う場合は`deepl.toml`に`[glossaries.internet_computer]`セクションを追加して`"key" = "value"`の形式で使用したい文字列を追記してください。

'deepl.sample.toml'に記載済みの内容を参考にしてください。

note: "internet_computer"以外のglossary名で辞書登録も可能ですが、現状、翻訳実行で使用できるのは"internet_computer"で登録した辞書のみです。今後、汎用的に使えるように改修を検討します。

```toml
# 例）
[glossaries.internet_computer]
"Internet Computer" = "Internet Computer"
"internet computer" = "internet computer"
"Internet Computer Protocol" = "Internet Computer Protocol"
"internet computer protocol" = "internet computer protocol"
"query call" = "クエリーコール"
"update call" = "アップデートコール"
```

追記が完了したら、以下のコマンドを実行して、辞書をDeepL API に登録します。

```bash
# --name: 登録対象の辞書名。toml内に記載がない場合はエラーになって登録できません
# -f, --from: 翻訳元の言語,選択可能な言語は Appendix 参照。
# -t, --to: 翻訳後の言語, 選択可能な言語はfromと同じ。
# input: 読み込み対象のtomlファイルのパスを指定。

cargo run -- glossary register --name "internet_computer" -f en -t ja ./deepl.toml
# -> Total 19 entries are registered as ID = ****************
```

note: 辞書を変更(`deepl.toml`を修正)した場合は、以下のコマンドで辞書の削除と再登録が必要です。

```bash
# 削除対象の辞書のIDを確認
cargo run -- glossary list
# -> DeeplGlossary { glossary_id: "1234567-9876-6789-09ji-cdpiauhesoaiu", name: "internet_computer", ready: true, source_lang: "en", target_lang: "ja", creation_time: "2023-08-17T05:57:06.339196Z", entry_count: 19 }

# 辞書の削除
# 上記の出力の削除したい辞書名の glossary_id の値を使って以下のコマンドを実行
cargo run -- glossary delete "1234567-9876-6789-09ji-cdpiauhesoaiu"
# -> `target/debug/cmark-translate glossary delete 1234567-9876-6789-09ji-cdpiauhesoaiu`

# 辞書の再登録
cargo run -- glossary register --name "internet_computer" -f en -t ja ./deepl.toml
# -> Total 19 entries are registered as ID = ****************
```

# 翻訳の実行
以下のコマンドを実行して翻訳を開始してください。


```bash
# --formality: formal（敬称） or informal（親称）。　指定しない場合は default が使用される
# -f, --from: 翻訳元の言語,選択可能な言語は Appendix 参照。
# -t, --to: 翻訳後の言語, 選択可能な言語はfromと同じ。
# -m, --max-depth: input がフォルダの場合、処理するフォルダ階層の深さを指定。
#                  デフォルトは usize::Max なので、フォルダ階層の考え方では無制限と同様の認識で良い。
# input: ファイル、またはフォルダのパスを指定。フォルダを指定した場合は .md の拡張子ファイルのみを翻訳対象とします。
# output: ファイル、またはフォルダのパスを指定。
#         inputがファイルの場合はファイルを、フォルダの場合はフォルダを指定しないとエラーで停止するします。
#         存在しないフォルダを指定した場合は、フォルダを生成して出力ファイルを格納します。
#         フォルダを指定した場合はinputのファイル群と同じファイル名で出力します。

cargo run --  translate --formality formal -f en -t ja ./portal/docs/concepts ./target/portal
# -> 翻訳したファイル１つに対して、以下のような1行分のレコードが出力されます。
# 　　　　　　Use glossary 1234567-9876-6789-09ji-cdpiauhesoaiu
#    Use glossary 1234567-9876-6789-09ji-cdpiauhesoaiu
#    Use glossary 1234567-9876-6789-09ji-cdpiauhesoaiu
```
note: [DeepLのアカウントページ](https://www.deepl.com/ja/account/usage)で「翻訳可能な残り文字数」が残っていることを確認してください。残り文字数が足りなくなるとエラーでAPIが使用できなくなります。

# Appendix
## 対応可能言語
```
"de"
"es"
"en"
"fr"
"it"
"ja"
"nl"
"pt-br"
"pt-br"
"ru"
```