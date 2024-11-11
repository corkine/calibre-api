# Calibre Web Api

Calibre Web 的一个基于 Rust 实现的服务端 API，用于提供 Calibre Web 的数据接口，包括图书信息、封面、搜索以及图书下载功能。

## Run

- 需要在同级目录下包含 app.db，其 Calibre Web 的 user 数据库执行 Basic 验证，带缓存。
- 需要在父级目录下包含 calibre-web 文件夹，其中包含 metadata.db 以及图书文件夹，API 数据由此数据库提供。

```bash
cargo run --release
# or use container
docker build -t calibre-api:latest .
docker run -it --rm -p 8080:8080 -v ../app.db:/calibre-web/app.db -v ../books:/calibre-data localhost/calibre-api:0.1.4
```

## License

MIT
