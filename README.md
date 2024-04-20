# eh2telegraph

[中文](README-zh.md)|英文

Bot that automatically downloads image sets from EH/EX/NH and uploads them to Telegraph.

This code is only guaranteed to work correctly on MacOS (partial functionality) and Linux.

## Deployment Guidelines
1. Install Docker and docker-compose.
2. Create a new folder `ehbot`.
2. Copy `config_example.yaml` from the project to `ehbot` and rename it to `config.yaml`, then change the configuration details (see the next section).
3. Copy `docker-compose.yml` to `ehbot`.
4. Start and Shutdown.
    1. Start: Run `docker-compose up -d` in this folder.
    2. Shutdown: Run `docker-compose down` in this folder.
    3. View logs: Run `docker-compose logs` in this folder.
    4. Update the image: Run `docker-compose pull` in this folder.

## Configuration Guidelines
1. Basic Configuration
    Bot Token: Find @BotFather in Telegram to apply.
    2. Admin (can be empty): your Telegram ID, you can get it from any relevant Bot (you can also get it from this Bot `/id`).
    3. Telegraph: Use your browser to create a Telegraph Token via [this link](https://api.telegra.ph/createAccount?short_name=test_account&author_name=test_author) and fill in. You can also change the author name and URL.
2. Proxy Configuration
    1. Deploy `worker/web_proxy.js` of this repository to Cloudflare Workers and configure the `KEY` environment variable to be a random string (the purpose of the `KEY` is to prevent unauthorized requests to the proxy).
    2. Fill in the URL and Key into the yaml.
    3. The proxy is used to request some services with frequency limitation, so do not abuse it.
3. IPv6 configuration
    1. You can specify an IPv6 segment, if you do not have a larger (meaning larger than `/64`) IPv6 segment, please leave it blank.
    2. Configure IPv6 to somewhat alleviate the flow restriction for single IP.
4. Configure cookies for some Collectors.
    1. Currently, only exhentai is required.
5. KV configuration
    1. This project uses a built-in caching service to avoid repeated synchronization of an image set.
    2. Please refer to [cloudflare-kv-proxy](https://github.com/ihciah/cloudflare-kv-proxy) for deployment and fill in the yaml file.
    3. If you don't want to use remote caching, you can also use pure memory caching (it will be invalid after reboot). If you want to do so, you need to modify the code and recompile it by yourself.

## Development Guidelines
### Environment
Requires the latest Nightly version of Rust. Recommended to use VSCode or Clion for development.

[RsProxy](https://rsproxy.cn/) is recommended as the crates.io source and toolchain installation source for users in China Mainland.

### Version Release
A Docker build can be triggered by typing a Tag starting with `v`. You can type the tag directly in git and push it up; however, it is easier to publish the release in github and fill in the `v` prefix.

## Technical Details
Although this project is a simple crawler, there are still some considerations that need to be explained.

### Github Action Builds
Github Action can be used to automatically build Docker images, and this project supports automatic builds for the `x86_64` platform.

However, it can also build `arm64` versions, but it is not enabled because it uses qemu to emulate the arm environment on x86_64, so it is extremely slow (more than 1h for a single build).

### IPv6 Ghost Client (it's not a well-known name, just made up by myself)
Some sites have IP-specific access frequency limits, which can be mitigated by using multiple IPs. The most common approach in practice is proxy pooling, but proxy pools are often extremely unstable and require maintenance and possibly some cost.

Observe the target sites of this project, many use Cloudflare, and Cloudflare supports IPv6 and the granularity of flow limitation is `/64`. If we bind a larger IPv6 segment for the local machine and randomly select IPs from it as client exit addresses, we can make more frequent requests steadily.

Since the NIC will only bind a single IPv6 address, we need to enable `net.ipv6.ip_nonlocal_bind`.

After configuring IPv6, for target sites that can use IPv6, this project will use random IP requests from the IPv6 segment.

Configuration (configuration for the NIC can be written in `if-up` for persistence).
1. `sudo ip add add local 2001:x:x::/48 dev lo`
2. `sudo ip route add local 2001:x:x::/48 dev your-interface`
3. Configure `net.ipv6.ip_nonlocal_bind=1` in Sysctl. This step varies by distribution (for example, the common `/etc/sysctl.conf` does not exist in Arch Linux).

Where to get IPv6? he.net offers a free service for this, but of course it is not expensive to buy an IPv6 IP segment yourself.

You can test the configuration with `curl --interface 2001:***** ifconfig.co` to see if it is correct.

### Forcing IPv6
The site mentioned in the previous subsection uses Cloudflare, but in fact does not really enable IPv6. when you specify the ipv6 request directly using curl, you will find that it has no AAAA records at all. But because the CF infrastructure is Anycast, so if the target site does not explicitly deny IPv6 visitors in the code, they can still be accessed through IPv6.

1. telegra.ph: No AAAA records, but force resolves to Telegram's entry IP for access, but the certificate is `*.telegram.org`.

    ~~This project writes a TLS validator that checks the validity of a given domain's certificate, to allow for misconfiguration of its certificate while maintaining security.~~

    However, Telegraph fixed the problem very quickly, so the TLS verifier is currently disabled.
2. EH/NH: Forced IPv6 availability.
3. EX: CF is not used and no IPv6 service is available.

### Proxy
This project uses Cloudflare Workers as a partial API proxy to alleviate the flow limitation problem when IPv6 is not available. See `src/http_proxy.rs` and `worker/web_proxy.js`.

### Caching
To minimize duplicate pulls, this project uses in-memory caching and remote persistent caching. Remote persistent cache using Cloudflare Worker with Cloudflare KV to build. The main project code reference is [cloudflare-kv-proxy](https://github.com/ihciah/cloudflare-kv-proxy).

Since it takes some time to synchronize image sets, to avoid repeated synchronization, this project uses [singleflight-async](https://github.com/ihciah/singleflight-async) to reduce this kind of waste.

## Contribute Guidelines
You are welcome to contribute code to this project(no matter how small the commit is)!
