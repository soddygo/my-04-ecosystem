# 说明
## Jaeger
本地运行jaeger服务

```shell
docker run --rm --name jaeger \
  -p 4317:4317 \
  -p 4318:4318 -p 16686:16686 \
  jaegertracing/all-in-one:1.59
```