# eh2telegraph

中文|[英文](README.md)

自动从 EH/EX/NH 下载图片集并上传至 Telegraph 的 Bot。

本代码只保证在 MacOS（部分功能）和 Linux 上可以正确运行。

## 部署指引
1. 安装 Docker 和 docker-compose。
2. 创建新文件夹 `ehbot`。
2. 复制项目中的 `config_example.yaml` 至 `ehbot` 并重命名为 `config.yaml`，之后修改配置细节（请参考下一节）。
3. 复制 `docker-compose.yml` 至 `ehbot`。
4. 开启与关闭：
    1. 开启：在该路径中运行 `docker-compose up -d`。
    2. 关闭：在该路径中运行 `docker-compose down`。
    3. 查看日志：在该路径中运行 `docker-compose logs`。
    4. 更新镜像：在该路径中运行 `docker-compose pull`。

## 配置指引
1. 基础配置：
    1. Bot Token：Telegram 内找 @BotFather 申请。
    2. Admin（可空）：你的 Telegram ID，随便找个相关 Bot 就可以拿到（也可以通过本 Bot `/id` 拿到）。
    3. Telegraph：使用浏览器通过[这个链接](https://api.telegra.ph/createAccount?short_name=test_account&author_name=test_author)创建 Telegraph Token 并填写。你也可以修改作者名字和 URL。
2. 代理配置：
    1. 部署本仓库中的 `worker/web_proxy.js` 至 CloudFlare Workers，并配置 `KEY` 环境变量为一段随机字符串（该 KEY 目的是防止对代理的未授权请求）。
    2. 填写 URL 和 Key 到配置中。
    3. 该代理用于请求一些有频率限制的服务，请勿滥用。
3. IPv6 配置：
    1. 可以填写一个 IPv6 段，如果你并没有拥有一个较大的（指比 `/64` 大）IPv6 段，请留空。
    2. 填写的话需要开启 `net.ipv6.ip_nonlocal_bind` 内核参数（参考后续章节说明）。
    3. 配置 IPv6 可以一定程度上缓解针对单 IP 的限流。
4. 配置部分 Collector 的 Cookie：
    1. 目前只有 exhentai 需要。
5. KV 配置：
    1. 本项目内置使用了一个缓存服务，可以避免对一个图片集的重复同步。
    2. 请参考 [cloudflare-kv-proxy](https://github.com/ihciah/cloudflare-kv-proxy) 进行部署，并填写至配置文件。
    3. 如果不想使用远程缓存，也可以使用纯内存缓存（重启后会失效），需要自行改代码并重新编译。

## 开发指引
### 环境
需要 Rust 最新的 Nightly 版本。推荐使用 VSCode 或 Clion 开发。

中国大陆推荐使用 [RsProxy](https://rsproxy.cn/) 作为 crates.io 镜像与工具链安装源。

### 版本发布
打 `v` 开头的 Tag 即可触发 Docker 构建。你可以直接在 git 中打 tag 之后 push 上去；但更方便的是在 github 中发布 release，并填写 `v` 开头的命名。

## 技术细节
虽然本项目就是一个简单的爬虫，但是还是有一些注意事项需要说明一下。

### Github Action 构建
Github Action 可以用于自动构建 Docker 镜像，本项目支持自动构建 `x86_64` 平台的版本。

但事实上也可以构建 `arm64` 的版本，由于其机制上使用了 qemu 在 x86_64 上模拟了 arm 环境，所以速度极其缓慢（单次构建需要 1h 以上），于是没有开启。

### IPv6 幽灵客户端（口胡的名字）
某些网站有针对 IP 的访问频率限制，使用多个 IP 即可缓解该限制。实践上最常用的办法是代理池，但代理池往往极不稳定，并需要维护，可能还有一定成本。

观察本项目的目标网站，很多使用了 Cloudflare，而 Cloudflare 支持 IPv6 且限流粒度是 `/64`。如果我们为本机绑定一个更大的 IPv6 段并从中随机选择 IP 作为客户端出口地址，则可以稳定地进行更高频率的请求。

由于网卡只会绑定单个 IPv6 地址，所以我们需要开启 `net.ipv6.ip_nonlocal_bind`。

配置 IPv6 后，对于可以走 IPv6 的目标站点，本项目会使用 IPv6 段中的随机 IP 请求。

配置（对网卡的配置可以写在 `if-up` 中便于持久化）：
1. `sudo ip add add local 2001:x:x::/48 dev lo`
2. `sudo ip route add local 2001:x:x::/48 dev your-interface`
3. 在 Sysctl 中配置 `net.ipv6.ip_nonlocal_bind=1`。该步骤因发行版而异（比如常见的 `/etc/sysctl.conf` 在 Arch Linux 中不存在）。

去哪搞 IPv6？he.net 提供了相关免费服务，当然自己购买一个 IPv6 IP 段也并不昂贵。

你可以通过 `curl --interface 2001:***** ifconfig.co` 测试配置是否正确。

### 强制 IPv6
前一小节提到的网站虽然用了 Cloudflare，但是事实上并没有真正启用 IPv6。当你直接使用 curl 指定 ipv6 请求时会发现，它根本就没有 AAAA 记录。但是由于 CF 的基础设施是 Anycast 的，所以如果目标网站不在代码中明确地拒绝 IPv6 访客，它们还是可以通过 IPv6 访问的。

1. telegra.ph: 无 AAAA 记录，但是强制解析到 Telegram 的入口 IP 可以访问，但证书是 `*.telegram.org` 的。

    ~~本项目写了一个校验指定域名证书有效性的 TLS 验证器，用于在保证安全性的情况下允许其证书配置错误。~~

    但是 Telegraph 以极快的速度修掉了该问题，所以该 TLS 校验器目前处于禁用状态。
2. EH/NH: 强制 IPv6 可用。
3. EX: 未使用 CF 且无 IPv6 服务。

### 代理
本项目使用 Cloudflare Workers 作为部分 API 代理，在 IPv6 不可用时缓解限流问题。参考 `src/http_proxy.rs` 和 `worker/web_proxy.js`。

### 缓存
为了尽可能少地重复拉取，本项目使用了内存缓存与远程持久化缓存。远程持久化缓存使用 Cloudflare Worker 配合 Cloudflare KV 搭建。项目主代码参考 [cloudflare-kv-proxy](https://github.com/ihciah/cloudflare-kv-proxy)。

由于同步图片集需要一定时间，为了避免重复同步，本项目使用了 [singleflight-async](https://github.com/ihciah/singleflight-async) 减少这类浪费。

## 贡献指引
欢迎你对本项目贡献代码！无论大小我们都欢迎！
