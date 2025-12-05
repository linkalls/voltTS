# Tooling Notes

VoltTS の開発で参照するツール/実装メモです。Bun の体験をインスピレーションとして扱いますが、リポジトリの実装は Rust 製 CLI (`voltts`) を中心に進めます。Bun は参考例にとどめ、テストは Rust 側で完結させます。

## Bun をどう使うか
- Bun はあくまで DX の参考例。`bun test` の体験を観察しつつ、VoltTS 自体は Rust 製 CLI が中心。テストは `cargo test` で完結する。
- Node.js ベースのコマンドは不要。Bun を触る場合も、必要に応じてローカルで個別に導入する。
- 現時点で Bun 依存のスモークテストは廃止済み。標準挙動は Rust 統合テストで担保する。

## Rust 製 CLI (`voltts`)
- サブコマンド: `init`, `run`, `test`, `fmt`, `lint`, `build`（C 出力 + ネイティブビルドまで実装）。
- 役割: v0.1 の C 出力パイプラインに向けた公式ツールの足場。
- 実行例: `cargo run -- init` / `cargo run -- build src/main.vts` / `cargo run -- run src/main.vts`。
- 依存: `cc`（clang/gcc 想定）で `dist/app.c` を `dist/app` にコンパイルする。

## 現状わかっていること・メモ
- CLI は Rust で提供し、テストも Rust 側で完結させる。Bun ランナーは参考情報としてのみ扱う。
- プロジェクト初期化時に `src/main.vts` と空の `tests/` を生成するサンプルを用意。
- `build` は `.vts` をパース→C 生成→`cc` で `dist/app` にビルドする最小実装。対応構文は `import { ... } from "..."`、`async fn` / `fn` / `await` / `print` /`return`（整数）に加え、標準ランタイム呼び出しとして `log.info|warn|error`、`time.now`、`time.sleep`、`fs.readFile`/`fs.writeFile`、引数なしの関数呼び出しをサポート。`main` の戻り値は省略可能で、省略時は C 側で `return 0;` を自動挿入する。`await` は現状シンタックスシュガーとして逐次実行される。`import` は TS 風に解決し、`./foo.vts` のような相対 import を再帰的に読み込んでコード生成する。
- `fmt` / `lint` は上記構文のパースを通すことで最低限の整形・診断を行う。対応していない構文はエラーを返す。
- `test` は v0.1 の検出パターンで `*.test.vts`, `*.spec.vts`, `*_test.vts` を列挙するところまで対応（実行は未実装）。
- Rust 側に統合テスト（`tests/cli_std_runtime.rs`）を持ち、CLI 挙動と標準ランタイムをまとめて検証する。標準 import + 相対 import を合わせて叩く `tests/stdlib_showcase.vts` も Rust テストから実行する。

### C 出力と Rust 出力の比較メモ
- **Rust 出力にすると？** `.vts` から生成された Rust コードを最終的に `rustc`/`cargo` でビルドする必要があるため、「コンパイラをビルドして配布すればユーザー環境はそれだけで済む」という状態にはならない。Rust ツールチェーン（標準ライブラリ、ターゲットごとの sysroot など）の配布が別途必要で、クロスコンパイル設定も増える。
- **C 出力の利点**: `cc` による単純なネイティブビルドで完結し、最小限の依存で済む。`libc` 相当があれば多くのプラットフォームでそのまま動かせる。
- **Rust 出力を選ぶ場合の対応**: 生成 Rust のビルドを `voltts` がラップする実装は可能だが、Rustup/クロスツールチェーンを同梱する必要があるため、現状は C 出力をデフォルトとし、Rust バックエンドは将来のオプションとして検討する。

## 実装メモ（2025-02 現在）
- `src/` を責務別に分割（`cli.rs`、`parser.rs`、`formatter.rs`、`codegen.rs`、`diagnostics.rs`、`ast.rs`）。エントリポイントは `main.rs` で Clap のハンドラを呼び分けるだけの薄い構造に整理。
- パーサは `VoltError` + `SourceLocation` を返し、`line/col` 情報付きでエラーメッセージを出す。CLI では `anyhow` にブリッジしてそのまま表示。
- 標準ランタイム（log/time/fs）の C 埋め込み部分は `codegen.rs` でまとめて生成。`log.info/log.warn/log.error`、`time.now/time.sleep`、`fs.readFile/fs.writeFile` を最小構成でサポート。
