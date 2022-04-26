# lopxy

[![crates.io](https://img.shields.io/crates/v/lopxy.svg)](https://crates.io/crates/lopxy)
[![Documentation](https://docs.rs/lopxy/badge.svg)](https://docs.rs/lopxy)
[![MIT/Apache-2 licensed](https://img.shields.io/crates/l/lopxy.svg)](./LICENSE)

lopxy是一个小型的本地代理服务器，用于对本地应用请求的网络资源路径进行替换，和监控请求失败网络资源的网络路径。主要应用场景是当某些应用依赖的一些网络资源失效时，可以通过lopxy来替换网络资源的路径为另一个有效的网络路径或者替换为本地文件。


## Install

```shell
cargo install lopxy
```

## Command

```shell
# 启动服务
lopxy start

# 关闭服务
lopxy stop

# 暂停proxy
lopxy disable

# 重新启用proxy
lopxy enable

# 添加代理条目(note: 可以使用短名，-r,-p,-c)，content-type默认为application/octet-stream
lopxy add --resource-url <原始资源路径> --proxy-resource-url <替换路径> --content-type <content-type>

# 删除条目
lopxy remove --resource-url <资源路径>

# 详细命令说明使用help子命令查看
lopxy help

# 查看子命令帮助
lopxy add --help
```

添加条目例子：

```shell
lopxy add -r http://www.resource.com/file -p file:///e:/文本文件.txt -c "text/plain"
```

## 访问Web管理界面

lopxy提供一个Web管理界面来添加proxy条目、查看异常请求日志和管理proxy服务，端口号可以在启动lopxy时通过参数指定，默认端口号是8283。

```shell
http://127.0.0.1:8283
```

## 系统代理设置方法

- [windows]由lopxy自动设置
- [mac]lopxy会通过调用mac的networksetup命令来设置和获取代理配置，但是未测试可用，最好手动配置
- [linux]由于linux发行版众多，暂时未整理出对所有发行版通用的设置方法，所以需要手动配置

## 支持操作系统

目前只支持windows，lopxy有几个方法未对mac和linux实现，在这两个环境下启动lopxy会直接panic。

## 目前支持协议

- http[**√**]
- https[**×**]
- ftp[**×**]
- sock[**×**]

## 使用注意

- 目前只支持http协议，对于https协议lopxy除了直接代理转发之外啥也不做
- 不支持chunked
- 对于http的长连接（Keep-Alive）会话，除了第一个请求，后序的请求目前不接管
- 最好只用于接管小文件，对于大文件还未测试过
- 添加proxy条目时，如果uri带中文等象形文字，需要是UTF-8编码，最好使用urlencoding来编码uri
- 添加proxy条目时，content-type只对本地文件有效，默认为application/octet-stream
- 对于本地文件使用file://作为协议的scheme前缀

## License

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)