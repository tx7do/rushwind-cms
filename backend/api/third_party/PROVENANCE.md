# api/third_party 来源清单

全部文件由 `api/vendor-third-party.sh` 从本地 buf 模块缓存复制，版本与
`go-wind-admin/backend/api/buf.lock` 锁定的 commit 一一对应。**不要手改**，重新执行脚本即可。

| 文件 | 模块 | 锁定 commit（buf.lock） | 用途 |
|---|---|---|---|
| `google/api/annotations.proto`、`google/api/http.proto` | buf.build/googleapis/googleapis | c17df5b2beca46928cc87d5656bd5343 | `google.api.http` 路由注解声明（admin-gen 生成器解析） |
| `google/api/field_behavior.proto` | 同上 | 同上 | 字段行为注解声明 |
| `google/api/httpbody.proto` | 同上 | 同上 | `google.api.HttpBody`（文件下载流） |
| `errors/errors.proto` | buf.build/kratos/apis | c2de25f14fa445a79a054214f31d17a8 | `(errors.code)` reason→HTTP 状态注解声明 |
| `pagination/v1/pagination.proto` | buf.build/tx7do/pagination | 7e34dd27013f4c67bb09025c73a73980 | go-crud `PagingRequest` 消息（列请求信封） |
| `validate/validate.proto` | buf.build/envoyproxy/protoc-gen-validate | daf171c6cdb54629b5f51e345a79e4dd | PGV 校验注解声明 |
| `gnostic/openapi/v3/annotations.proto`、`gnostic/openapi/v3/openapiv3.proto` | buf.build/gnostic/gnostic | 087bc8072ce44e339f213209e4d57bf0 | OpenAPI 扩展注解（免鉴权端点标注） |
| `redact/v1/redact.proto` | buf.build/go-wind/redact | d1f98995227e44d6b839e578d9fa1466 | 静态脱敏注解声明 |

第三方 proto 版权归各自上游（Apache-2.0）；vendor 副本仅用于编译期 descriptor 解析。
