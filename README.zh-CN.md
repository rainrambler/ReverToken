# ReverToken

[English](README.md) | [简体中文](README.zh-CN.md)

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust 1.85+](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)
[![Platforms](https://img.shields.io/badge/platforms-Linux%20%7C%20Windows%20%7C%20macOS-lightgrey.svg)](#支持的格式)

ReverToken 是一个跨平台的命令行工具，用于查看可执行文件和共享库的**导出表**。
它可以递归扫描文件或目录，从 **PE（EXE/DLL）**、**ELF（可执行文件/`.so`）** 和
**Mach-O（可执行文件/dylib）** 中提取导出符号，还原 C++ 名称，并支持按关键字搜索。

## 目录

- [为什么选择 ReverToken？](#为什么选择-revertoken)
- [功能](#功能)
- [支持的格式](#支持的格式)
- [安装](#安装)
- [快速开始](#快速开始)
- [命令行参数](#命令行参数)
- [搜索规则](#搜索规则)
- [输出](#输出)
- [JSON 输出](#json-输出)
- [实现方式](#实现方式)
- [限制](#限制)
- [开发](#开发)
- [贡献](#贡献)
- [安全](#安全)
- [许可证](#许可证)

## 为什么选择 ReverToken？

逆向工程师和开发者经常需要快速回答这些问题：

- 这个 DLL 或 `.so` 导出了哪些函数？
- `?add@demo@@YAHHH@Z` 或 `_ZN4demo3addEii` 对应的可读名称是什么？
- 哪些导出符号包含 `crypto`、`SSL` 或 `CreateFile`？

ReverToken 用一条命令回答这些问题，并提供稳定的 JSON 输出，方便脚本处理。
它不是反汇编器或反编译器，只专注于导出表元数据。

## 功能

- **文件或目录输入**：目录会递归扫描；按文件头识别格式，不依赖扩展名。
- **导出表提取**
  - PE：解析 Export Directory，支持序号、RVA/VA、转发导出，以及根据所在节区
    判断函数/数据。
  - ELF：解析 `.dynsym` 中已定义、已导出且可见性为 default/protected 的符号。
  - Mach-O：解析 export trie，支持 thin 和 universal（fat）二进制。
- **C++ 名称还原**
  - Itanium ABI（Linux 上的 GCC/Clang、MinGW、macOS）：
    `_ZN4demo3addEii` → `demo::add(int, int)`
  - MSVC（Windows）：`?add@demo@@YAHHH@Z` → `int __cdecl demo::add(int, int)`
  - C 符号和无法识别的名称保持原样；原始名称始终保留。
- **关键字搜索**：同时匹配原始名和还原名；默认忽略大小写，可重复使用 `-s`。
- **终端表格和 JSON**：既适合人工查看，也适合脚本处理。
- **终端安全**：符号名中的控制字符会被转义。
- **轻量可移植**：纯 Rust 实现，运行时不需要系统工具链。

## 支持的格式

| 格式 | 变体 | 导出表来源 | 说明 |
| --- | --- | --- | --- |
| PE | PE32、PE32+（`.exe`、`.dll`） | Export Directory | 有名字的导出、序号、转发导出 |
| ELF | ELF32、ELF64（`ET_EXEC`、`ET_DYN`） | `.dynsym` | 仅动态导出符号 |
| Mach-O | 32/64 位，thin 和 fat | export trie | 可执行文件、dylib、bundle |

## 安装

### 预编译二进制

打标签发布时，GitHub Actions 会为 Linux（x86-64）、Windows（x86-64）和 macOS
（Apple Silicon 与 Intel）生成压缩包。请从仓库的 **Releases** 页面下载对应平台
的压缩包，并把 `revertoken`（Windows 上为 `revertoken.exe`）加入 `PATH`。

Linux 压缩包面向 glibc 2.35（Ubuntu 22.04）及以上环境。macOS 压缩包未签名、
未公证，首次运行时可能需要在**系统设置 → 隐私与安全性**中允许运行。

### 从源码构建

需要 Rust **1.85 或更高版本**（项目使用 edition 2024）。

在仓库检出目录中执行：

```sh
cargo install --path .
```

也可以只构建不安装：

```sh
cargo build --release
./target/release/revertoken --help
```

## 快速开始

```sh
revertoken <PATH> [OPTIONS]
```

扫描目录，并同时在原始名和还原名中搜索 `add`：

```sh
revertoken ./tests/fixtures -s add
```

```text
tests/fixtures/demo.dll
  PE32+ | X86_64 | LE | DLL | 2 exports, 1 match
  ADDRESS             ORD  TYPE  NAME                             MANGLED
  ------------------  ---  ----  -------------------------------  ------------------
  0x0000000180001000    1  FUNC  int __cdecl demo::add(int, int)  ?add@demo@@YAHHH@Z

tests/fixtures/libdemo.so
  ELF64 | x86_64 | LE | shared library | 2 exports, 1 match
  ADDRESS             ORD  TYPE  NAME                 MANGLED
  ------------------  ---  ----  -------------------  ---------------
  0x00000000000012e0    -  FUNC  demo::add(int, int)  _ZN4demo3addEii

tests/fixtures/libdemo.dylib
  Mach-O 64 | arm64 | LE | dylib | 2 exports, 1 match
  ADDRESS             ORD  TYPE  NAME                MANGLED
  ------------------  ---  ----  ------------------  --------------
  0x0000000000000320    -  SYM   demo_add(int, int)  __Z8demo_addii

Scanned 3 binaries, 6 exports; 3 matched.
```

更多示例：

```sh
# Windows
revertoken C:\Windows\System32\kernel32.dll -s CreateFile

# Linux
revertoken /usr/lib/libssl.so -s SSL_CTX

# 多个关键字必须全部命中（AND）
revertoken ./plugins -s crypto -s tls --match-all

# 机器可读输出
revertoken ./bin --json > exports.json
```

## 命令行参数

| 参数 | 说明 |
| --- | --- |
| `<PATH>` | 要扫描的文件或目录 |
| `-s, --search <TEXT>` | 搜索关键字，可重复；匹配原始名和还原名 |
| `--match-all` | 多个 `--search` 时要求全部命中（默认任一命中） |
| `-c, --case-sensitive` | 区分大小写 |
| `--no-demangle` | 不还原 C++ 名称，只显示原始符号 |
| `--json` | 输出 JSON 而不是表格 |
| `-L, --follow-links` | 递归时跟随符号链接 |
| `-v, --verbose` | 显示无法解析的文件和目录遍历警告 |
| `-h, --help` | 显示帮助 |
| `-V, --version` | 显示版本 |

退出码：

- `0` — 扫描完成，包括没有符号命中的情况。
- `1` — I/O 错误、不支持的文件或二进制格式损坏。
- `2` — 命令行参数错误（由 clap 处理）。

## 搜索规则

- 关键字同时匹配**原始符号名**和**还原后的名称**。
- 使用子串匹配，默认**忽略大小写**；可用 `--case-sensitive` 改变。
- 多个 `--search` 默认是 **OR**；加 `--match-all` 后要求全部命中。
- 搜索不改变退出码：没有命中时仍然返回 `0`。

## 输出

每个表格包含：

| 列 | 含义 |
| --- | --- |
| `ADDRESS` | PE：ImageBase + RVA；ELF：`st_value`；Mach-O：导出地址。转发导出显示为 `-` |
| `ORD` | PE 导出序号；其他格式为 `-` |
| `TYPE` | `FUNC`、`DATA`；类型未知时为 `SYM` |
| `NAME` | 优先显示还原后的名称，否则显示原始名 |
| `MANGLED` | 原始符号名，仅在它与 `NAME` 不同时显示 |
| `NOTE` | 转发导出的目标 |

## JSON 输出

```sh
revertoken tests/fixtures/demo.dll --json -s demo
```

```json
{
  "scanned_binaries": 1,
  "total_exports": 2,
  "matched_exports": 1,
  "files": [
    {
      "path": "tests/fixtures/demo.dll",
      "format": "pe",
      "arch": "X86_64",
      "bits": 64,
      "endianness": "little",
      "kind": "DLL",
      "export_count": 2,
      "exports": [
        {
          "name": "?add@demo@@YAHHH@Z",
          "demangled": "int __cdecl demo::add(int, int)",
          "kind": "function",
          "address": 6442455072,
          "rva": 4128,
          "ordinal": 1
        }
      ]
    }
  ]
}
```

`exports` 是过滤后的结果，`export_count` 是该文件的导出总数。启用搜索时，所有
被扫描的二进制仍会出现在 `files` 中，即使其 `exports` 为空。

## 实现方式

1. 输入是目录时，使用 [`walkdir`](https://crates.io/crates/walkdir) 递归展开。
2. 读取文件头 16 字节，跳过非 PE、ELF、Mach-O 的文件。
3. 使用 [`goblin`](https://crates.io/crates/goblin) 解析容器和导出表。
4. 按需使用 [`cpp_demangle`](https://crates.io/crates/cpp_demangle)（Itanium ABI）
   或 [`msvc-demangler`](https://crates.io/crates/msvc-demangler)（MSVC）还原名称。
5. 应用过滤并输出结果。解析和还原过程不会修改输入文件。

## 限制

- 当前聚焦**导出表**，不扫描导入表。
- PE 只包含**有名字的导出**；仅有序号的匿名导出不会显示。
- ELF 只读取 `.dynsym`；静态符号表 `.symtab` 不读取，`GLIBC_2.2.5` 等符号版本
  不单独展示。
- Mach-O 的 C 符号保留平台要求的 `_` 前缀。
- 目录扫描会跳过目标文件、静态库和未知格式；显式传入这类文件会报错。
- 不还原 Rust v0 名称。
- 这不是反汇编器，不会从机器码恢复函数签名或分析函数体。

## 开发

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

测试套件包含位于 [`tests/fixtures`](tests/fixtures) 的小型 PE、ELF、Mach-O
样例，生成方式见 [`tests/fixtures/README.md`](tests/fixtures/README.md)。

CI 会在 Linux、Windows 和 macOS 上运行格式检查、Clippy 和测试，并验证最低支持的
Rust 版本。

## 贡献

欢迎提交 Issue 和 Pull Request。开始之前请阅读
[`CONTRIBUTING.md`](CONTRIBUTING.md) 和
[`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md)。

- 可复现的 Bug 和功能建议请使用 GitHub Issues。
- 安全问题请按 [`SECURITY.md`](SECURITY.md) 私下报告。
- 保持改动聚焦，并为新行为添加测试。
- 用户可见的改动请更新 [`CHANGELOG.md`](CHANGELOG.md)。

## 安全

ReverToken 会解析不受信任的二进制文件。请不要在公开 Issue 中附加敏感或专有二进制。
报告流程见 [`SECURITY.md`](SECURITY.md)。

## 许可证

本项目使用 [MIT License](LICENSE) 开源。

## 致谢

ReverToken 受益于 Rust 生态，特别是
[`goblin`](https://crates.io/crates/goblin)、
[`cpp_demangle`](https://crates.io/crates/cpp_demangle)、
[`msvc-demangler`](https://crates.io/crates/msvc-demangler)、
[`clap`](https://crates.io/crates/clap) 和
[`walkdir`](https://crates.io/crates/walkdir)。
