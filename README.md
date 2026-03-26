<div align="center">
  <img src="icon.png" width="128" alt="karukan" />
  <h1>Karukan</h1>
  <p>Linux向け日本語入力システム — ニューラルかな漢字変換エンジン + fcitx5</p>

  [![CI (engine)](https://github.com/togatoga/karukan/actions/workflows/karukan-engine-ci.yml/badge.svg)](https://github.com/togatoga/karukan/actions/workflows/karukan-engine-ci.yml)
  [![CI (im)](https://github.com/togatoga/karukan/actions/workflows/karukan-im-ci.yml/badge.svg)](https://github.com/togatoga/karukan/actions/workflows/karukan-im-ci.yml)
  [![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
</div>

<div align="center">
  <img src="images/demo.gif" width="800" alt="karukan demo" />
</div>

## プロジェクト構成

| クレート | 説明 |
|---------|------|
| [karukan-im](karukan-im/) | karukan-engineを利用したfcitx5向け日本語入力システム |
| [karukan-engine](karukan-engine/) | コアライブラリ — ローマ字→ひらがな変換 + llama.cppによるニューラルかな漢字変換 |
| [karukan-cli](karukan-cli/) | CLIツール・サーバー — 辞書ビルド、Sudachi辞書生成、辞書ビューア、AJIMEE-Bench、HTTPサーバー |

## 特徴

- **ニューラルかな漢字変換**: GPT-2ベースのモデルをllama.cppで推論し、高度な日本語変換
- **コンテキスト対応**: 周辺テキストを考慮した日本語変換
- **変換学習**: ユーザーが選択した変換結果を記憶し、次回以降の変換で優先表示。予測変換（前方一致）にも対応し、入力途中でも学習済みの候補を提示
- **システム辞書**: [SudachiDict](https://github.com/WorksApplications/SudachiDict)の辞書データからシステム辞書を構築

> **Note:** 初回起動時にHugging Faceからモデルをダウンロードするため、初回の変換開始までに時間がかかります。2回目以降はダウンロード済みのモデルが使用されます。

## インストール

インストール方法は [karukan-im の README](karukan-im/README.md#install) を参照してください。

## ライセンス

MIT OR Apache-2.0 のデュアルライセンスで提供しています。

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)

---

## フォーク変更点

本フォークでは、私的な使い勝手を向上させるため以下の変更を加えています。

### 機能追加・修正

#### 1. 末尾「n」の自動変換
Space変換時に末尾の `n` を自動的に「ん」に変換します。`nn` と入力する必要がなくなりました。

- 対象: `karukan-engine/src/romaji/converter.rs`

#### 2. Shift英字入力後のモード自動復帰
Shift+英字でアルファベットを入力した後、確定してEmpty状態に戻ると自動的にひらがなモードに復帰します。

- 対象: `karukan-im/src/core/engine/input.rs`

#### 3. カーソル位置による部分変換
カーソルを移動してからSpaceを押すと、カーソルより前のテキストのみを変換し、残りはComposing状態に戻ります。

- 対象: `karukan-im/src/core/engine/conversion.rs`, `mod.rs`

#### 4. 記号の全角変換
ひらがなモード時に `#`, `(`, `)`, `@`, `<`, `>` 等の記号が全角に変換されます。

- 対象: `karukan-engine/src/romaji/rules.rs`

### 設定オプション

`~/.config/karukan-im/config.toml` に以下の設定を追加しています。

```toml
[conversion]
# 入力中の自動変換候補表示を無効化（false = Spaceキー変換時のみ表示）
auto_suggest = false

# 変換候補ウインドウを表示するまでのSpace押下回数（0 = 常に表示）
candidate_window_threshold = 3

# 補助テキスト（推論時間・辞書ソース等）の表示（false = 常に非表示）
show_aux_text = false
```

| 設定項目 | デフォルト | 説明 |
|---|---|---|
| `auto_suggest` | `true` | 入力中に変換候補を自動表示する |
| `candidate_window_threshold` | `3` | 候補ウインドウを表示するまでのSpace押下回数。0で常に表示 |
| `show_aux_text` | `true` | 推論時間・辞書ソース等の補助テキストを表示する |

### 辞書の拡張

jawiki（Wikipedia固有名詞）のシステム辞書統合や、顔文字・絵文字辞書の導入手順については [辞書セットアップガイド](docs/dictionary-setup.md) を参照してください。
