# 地图切片代理服务

一个基于 Rust 开发的地图服务代理平台，用于管理地图调用 Key，避免应用在调用时泄露密钥。

## 目录

- [功能](#功能)
- [技术栈](#技术栈)
- [编译与构建](#编译与构建)
- [运行方式](#运行方式)
- [常见问题](#常见问题)

## 功能

- 统一管理多个地图服务的 API Key
- 代理地图瓦片请求，隐藏真实的密钥信息
- 支持缓存，减少重复请求对上游服务的压力
- 基于 Actix-web 的高性能 HTTP 服务

## 技术栈

- **语言**: Rust
- **Web 框架**: Actix-web
- **构建**: Cargo

## 配置
```yaml
port: 9090 # 应用的地址
cache_root: /home/tile-data  # 配置存储的路径  
token_name: 'key' # 请求的参数名称
token_value:
  - 'access token'
base_url: 'http://192.168.8.54:{port}'  # 你自己的服务地址，主要用于替换WMTS 配置文件
source_base_url: 'http://<host>:<port>'  # gis 服务的地址前缀 
services:
  <service_name>: '<source url>'  # 与 source_base_url 拼接组成上游的ArcGIS服务地址,调用时使用
  #调用地址  {source_base_url}/arcgis/rest/services/{service_name}/MapServer/tile/{z}/{x}/{y}
  # 配置地址 {source_base_url}/arcgis/rest/services/{service_name}/MapServer?f=pjson
tianditu_service:
  - 'vec_c'
  - 'vec_w'
  - 'cva_c'
  - 'cva_w'
  - 'img_c'
  - 'img_w'
  - 'cia_c'
  - 'cia_w'
  - 'ter_c'
  - 'ter_w'
  - 'cta_c'
  - 'cta_w'
  - 'swdx'
tianditu_token:
  - '<天地图Key>'
# 天地图调用地址：  {source_base_url}/arcgis/rest/services/tianditu/{tianditu_service}/wmts/tile/{z}/{x}/{y}
```
## 运行
```shell
cargo run
```

## 编译与构建

### 1. 安装依赖

```shell
sudo dnf install epel-release
```

### 2. 安装并配置 musl（用于静态编译）

```shell
# 下载 musl 源码（以 v1.2.6 为例）
wget https://musl.libc.org/releases/musl-1.2.6.tar.gz
tar -xzf musl-1.2.6.tar.gz
cd musl-1.2.6

# 配置安装路径并编译
./configure --prefix=/usr/local/musl
make -j$(nproc)
sudo make install
```

### 3. 配置交叉编译工具链

```shell
# 创建 musl-ar 软链接
sudo ln -s /usr/bin/ar /usr/local/musl/bin/musl-ar

# 设置环境变量
export CC_x86_64_unknown_linux_musl=/usr/local/musl/bin/musl-gcc
export AR_x86_64_unknown_linux_musl=/usr/local/musl/bin/musl-ar
```

### 4. 添加编译目标

```shell
rustup target add x86_64-unknown-linux-musl
```

### 5. 构建发布版本

```shell
cargo build --release --target x86_64-unknown-linux-musl
```

## 运行方式

### 以 systemd 服务运行

1. 创建服务文件 `/etc/systemd/system/map-proxy-cache.service`：

```ini
[Unit]
Description=Map Proxy Service By Rust
After=network.target

[Service]
Type=simple
ExecStart=/opt/map-proxy-service/map-proxy-actix-web
WorkingDirectory=/opt/map-proxy-service
User=root
Group=root
Restart=always
RestartSec=5
Environment="RUST_LOG=info"

[Install]
WantedBy=multi-user.target
```

2. 重新加载 systemd 配置：

```shell
sudo systemctl daemon-reload
```

3. 设置开机自启并启动服务：

```shell
sudo systemctl enable map-proxy-cache.service
sudo systemctl start map-proxy-cache.service
```

## 常见问题

1. `error occurred in cc-rs: failed to find tool "/usr/local/musl/bin/musl-ar"`

```shell
# 检查系统 ar 的位置
which ar

# 创建软链接到 musl 目录
sudo ln -s /usr/bin/ar /usr/local/musl/bin/musl-ar

# 验证
ls -l /usr/local/musl/bin/musl-ar

# 设置环境变量后重新构建
export CC_x86_64_unknown_linux_musl=/usr/local/musl/bin/musl-gcc
export AR_x86_64_unknown_linux_musl=/usr/local/musl/bin/musl-ar
cargo build --release --target x86_64-unknown-linux-musl
```