# VoltTS Standard Package Design (working notes)

Tieredで広げすぎを防ぎつつ、現場で即使える“厚めの標準”を目指すメモ。Bunの統合DXとGo/Rustの基礎力を意識しつつ、TSの過積載を避ける。

## レイヤー方針
- **Tier S (v0.1で確実に入れる)**: core.*, collections, strings/bytes, time, io/fs/path, os/process/env, encoding.json/base64, log, testing, net.http (最小クライアント)
- **Tier A (v0.1〜v0.2で順次)**: net.http サーバ強化、crypto、cli、bench/doc生成
- **Tier B (拡張)**: 便利系は薄く後から

## 推奨パッケージの粒度
- **core**: option/none, result, compare, iter
- **collections**: Array/Map/Set + 連続メモリ系（Vec相当）の芽
- **strings / bytes**: split/join/trim/replace、utf8、buffer操作
- **math / random**: 用途別 random.fast / random.crypto
- **time**: Instant/Duration/DateTime/sleep
- **io / fs / path**: read/write/mkdir/walk、path.join/normalize
- **os / process / env**: platform, exec, args, env get/set
- **net**: tcp/udp（薄く）、http client/server（ミドルウェアは最小）
- **encoding**: json, base64, hex, url
- **crypto**: hash/hmac/random（安全乱数の正規ルート）
- **concurrency**: mutex/rwlock/atomic/channel
- **log**: info/warn/error（文字列＆構造化を最小サポート）
- **cli**: flag/prompt/color
- **testing**: describe/test/it/beforeEach/afterEach/expect + bench/watch

## 運用ルール
1. **APIは少なく拡張点は太く** — 標準で1つの正解ルートを用意（例: JSONはencoding.json、ログはlog）。
2. **薄く長持ちさせる** — TSの型魔術を持ち込まず、構造的・消せる型だけを基本に。
3. **Bun型統合DX + Rust/Go型の土台** — テスト/ビルド/パッケージ体験をCLIに統合しつつ、標準は日常必須セットを先に固める。

## main の扱い（v0.1メモ）
- `main` は戻り値省略OK。コードジェネレータは `return` が無ければ `0` を返すCコードを吐く。
- サンプルも戻り値なしで統一。

## 非同期構文の扱い（v0.1 試験実装）
- `async fn` と `await <call>` を構文として受け付ける。現状は同期的にそのまま実行するが、コードの見た目を先に非同期スタイルに寄せておく。
- 標準ランタイム呼び出し（`log.*`, `time.now`, `time.sleep`, `fs.readFile|writeFile`）には `await` を付けてもよい。
- C ランタイムは非同期ではないため、`await` の挙動は「逐次実行」のシンタックスシュガーとして実装している。

この方針で「小さい言語 + 強い標準 + 統合CLI」の三点セットを目指す。

## v0.1 プロトタイプで入れた最小実装（Rust CLI + C runtime）
- **log**: `log.info|warn|error("text")` を埋め込み C ランタイムで出力（`[info] ...` など）。
- **time**: `time.now()` でエポック ms を出力、`time.sleep(ms)` で簡易 sleep。
- **fs**: `fs.writeFile(path, text)` で親ディレクトリも含めて作成し、`fs.readFile(path)` で全量読み込んで stdout に吐くスモール実装を追加。
- 既存の `print` と合わせて、標準パッケージの“日常セット”を呼べるようにした（構文はまだ限定）。
- 標準パッケージは `import { fs, log, time } from "std"` のようにパッケージ import で呼び出し、`./helper.vts` などの相対 import も TS 風に解決する（ビルド時に依存ファイルを再帰的に読み込み）。
 - `tests/stdlib_showcase.vts` + `tests/helpers/helper.vts` に標準パッケージと相対 import をまとめて叩くデモを追加し、Rust 統合テストで動作を検証。
 - `tests/fs_sample.vts` で fs ランタイムの write/read を Rust 統合テストとして確認し、外部ランナー不要の形に整理。
- C 側には `vts_log_*`, `vts_time_now_ms`, `vts_sleep_ms`, `vts_fs_read_file`, `vts_fs_write_file` を同梱しており、追加のリンク依存を持たない（`cc` 一発でビルド）。
- Rust 統合テスト（`tests/cli_std_runtime.rs`）で fs ランタイムや標準 import を含む CLI 挙動をまとめて検証する。
