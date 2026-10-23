# 官网维护

线上地址：<https://alvinfunborn.github.io/screen-buoy/>

官网是 `website/` 下的独立静态页面，不依赖桌面应用的 Node / Rust 构建。页面使用本地字体和资源，不调用 GitHub API，也没有分析统计脚本。下载链接使用 GitHub Releases 的 `latest/download/` 地址；发版时保持文件名不变，官网就会指向最新正式版本。

## 本地预览

```sh
python3 -m http.server 4173 --directory website --bind 127.0.0.1
```

打开 <http://127.0.0.1:4173/>。`index.html` 包含正文、安装指引和下载入口，`style.css` 定义响应式布局，`site.js` 仅控制网页 Hint 示意。演示只在自身区域处理按键，不注册页面全局快捷键。

## 自动发布

仓库 Pages 的构建来源设为 **GitHub Actions**。合并或推送 `website/**` 或 `.github/workflows/pages.yml` 的修改到 `main`，会运行 `Deploy website`；也可以手动触发此工作流。它只上传 `website/`，不会发布源码、开发文档或应用构建目录。

## 素材

`assets/icon.png` 来自 `src-tauri/icons/icon.png`。桌面截图、滚动和拖拽视频来自仓库已有 `docs/hints.png`、`docs/scroll.gif`、`docs/drag.gif`，网页中分别标注为真实运行和交互示意。GIF 转为 H.264 MP4，视频设为按需加载，保留原生播放控件；静态海报和截图使用压缩 JPEG。

更换域名时，同时更新 canonical、Open Graph URL / 图片、`robots.txt` 和 `sitemap.xml`。
