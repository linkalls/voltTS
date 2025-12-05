# VoltTS

VoltTS は、TS に似た読みやすさと Go/V のシンプルさ、Bun の DX を目指す言語の実験リポジトリです。まずは C 出力によるネイティブ体験を最優先で固めています。Rust 製の簡易フロントエンドで `.vts` を C に変換し、`cc` でネイティブ実行する流れを最小実装しています。標準ランタイムも同梱し、`log`/`time`/`fs` といった日常 API を `.vts` から直接呼び出せます。`async fn`/`await` も最低限サポートし、現状は同期的に直列実行されますが「非同期構文で書ける」体験を先に用意しています。標準パッケージは `import { fs, log, time } from "std"` のようにパッケージ import で呼び出し、`./foo.vts` のような相対 import も TS 風に解決します（ビルド時に依存ファイルを再帰的に読み込む）。

## ツールチェーン
- Rust 製 CLI `voltts` を公式ツールの土台として実装中。`cargo run -- <command>` で動作確認できます。
- Bun は DX の参考例として扱いますが、リポジトリ自体は Bun 依存ではありません。検証は Rust 側の統合テスト（`cargo test`）で完結しており、Bun ベースのスモークテストは不要になりました。
- `voltts build/run` は `.vts` を C に変換して `dist/app.c` / `dist/app` を生成するプロトタイプです。
- `voltts fmt` は対応している構文（`import { ... } from "..."`、`async fn` / `fn` / `await` / `print` / `return`、`log.*`、`time.*`、`fs.readFile|writeFile`、シンプルな関数呼び出し）をパースし、正規化したスタイルで書き戻します。
- `voltts lint` は構文チェックを通すだけの簡易診断です。
- 埋め込みの標準ランタイム（log/time/fs）を C 生成時に同梱し、`log.info|warn|error`、`time.now`/`time.sleep`、`fs.readFile|writeFile` が `.vts` から呼べます。`await` を付けても同期実行されるため、コードの見た目だけ先に非同期対応しています。
- `voltts test` は v0.1 仕様に沿ったファイル検出まで実装済みです（実行は今後対応）。
- C 出力を Rust 出力に置き換える案は検討中ですが、生成された Rust コードを最終的に `rustc`/`cargo` でビルドする必要があるため
  「コンパイラをビルドすればそれだけで完結」という状態にはならず、Rust ツールチェーンの配布やクロスコンパイルの重さが残ります。

## ドキュメント
- [v0.1 Specification Snapshot](docs/v0.1-spec.md)
- [Tooling Notes](docs/tooling.md)
- [Standard Package Design Notes](docs/standard-packages.md)
