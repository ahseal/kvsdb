# 基于 tokio 实现的简单的键值数据库

## 运行

```bash
# 服务端
cargo b --bin kvs-server

# 客户端
cargo b --bin kvs-cli set hello world
```

## 支持命令

### 客户端

```
$ kvs-cli -h
A simple key-value database

Usage: kvs-cli.exe [OPTIONS] <COMMAND>

Commands:
  get   Get the value of key
  set   Set key to the string value
  del   Remove a given key
  ping  TPing server status
  help  Print this message or the help of the given subcommand(s)

Options:
      --host <HOST>  Hostname to set [default: 127.0.0.1]
  -p, --port <PORT>  Port to set [default: 3000]
  -l, --log          Enable log
  -h, --help         Print help
  -V, --version      Print version
```

### 服务端

```
$ kvs-server -h
A simple key-value database

Usage: kvs-server.exe [OPTIONS]

Options:
  -p, --port <PORT>  Port to set [default: 3000]
  -l, --log          Enable log
  -h, --help         Print help
  -V, --version      Print version
```
