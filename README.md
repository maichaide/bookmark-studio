# Bookmark Studio

跨平台、离线运行的浏览器书签整理桌面应用，使用 Rust + egui 开发。

## 功能

- 导入 Firefox、Chrome、Edge 导出的 Netscape Bookmark HTML（`.html` / `.htm`）。
- 解析书签标题、URL、文件夹层级、`ADD_DATE`、`LAST_MODIFIED`、`ICON`、`TAGS`。
- 搜索标题、URL、标签和文件夹路径。
- 按标题、标签、URL 排序；标题和标签支持中文拼音排序。
- 高亮完全重复的 URL。
- 自定义重复匹配：忽略 fragment、query、末尾斜杠、协议、www 前缀、主机名大小写。
- 批量删除选中书签。
- 清理重复项，每个重复组保留一条；若该组有选中项则优先保留选中项。
- 撤销最近一次删除或批量标签编辑。
- 批量替换选中书签的标签。
- 导出浏览器兼容的 Netscape Bookmark HTML，并保留文件夹层级及常见元数据。
- 简体中文 / English 界面切换。
- 所有处理在本机完成，不上传书签。

## 环境要求

- Rust stable（建议通过 https://rustup.rs/ 安装）。
- Linux 桌面环境需要 eframe/winit 对应的系统开发库。Ubuntu/Debian 可先安装：

```bash
sudo apt update
sudo apt install build-essential pkg-config libx11-dev libxrandr-dev \
  libxi-dev libxcursor-dev libxinerama-dev libxkbcommon-dev \
  libwayland-dev libxcb1-dev libgl1-mesa-dev
```

## 运行与测试

```bash
cargo run
cargo test
```

## Release 编译

```bash
cargo build --release
```

输出位于 `target/release/`，Windows 下为 `bookmark-studio.exe`。

## 跨平台编译

建议在对应操作系统的原生 runner 上编译，避免 Apple SDK、Windows linker 和 Linux 系统库差异。源码不需要修改。

```bash
# Windows x64（在 Windows 上运行）
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc

# macOS Apple Silicon（在 macOS 上运行）
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# macOS Intel（在 macOS 上运行）
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin

# Linux x64（在 Linux 上运行）
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu

# Linux ARM64（在配置好 ARM64 linker/sysroot 的 Linux 环境中运行）
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
```

跨操作系统交叉编译还需要目标平台 linker / SDK；优先使用 GitHub Actions 对应平台 runner。

## GitHub Actions

`.github/workflows/build.yml` 提供 Windows、macOS、Linux 构建矩阵，可在 GitHub Actions 中手动触发或推送 `v*` 标签触发。

## 重复检测说明

- 默认按 URL 原文严格比较，不会自动去掉参数、片段、协议差异或末尾斜杠。
- 勾选自定义规则后才会按规则规范化 URL。
- 删除重复项时，每组保留一条。若某组内勾选了书签，则优先保留该条；否则保留解析顺序中的第一条。
- 当前撤销栈保存最近一次可撤销操作。

## 已知边界

- Netscape Bookmark HTML 不同浏览器及版本存在细微差异；本项目针对标准导出格式解析。建议使用自己浏览器的导出文件做验收。
- 某些浏览器专有元数据可能不在 Netscape HTML 中，因此无法保证往返导出后保留所有厂商私有信息。
- 多义汉字拼音排序采用拼音库默认读音，并非上下文词典级分词排序。

## License

MIT

### Chinese font support

The application bundles Noto Sans CJK SC and registers it with egui at startup.
Chinese UI labels and Chinese bookmark titles therefore do not depend on fonts
installed on the host operating system. The font's SIL Open Font License is in
`assets/OFL.txt`.
