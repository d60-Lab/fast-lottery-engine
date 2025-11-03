今天开始做开源项目。
一方面因为自己做的事情我感觉没啥挑战，锻炼下技术能力，基于场景深挖下。
另一方面是近些年的项目经历实在过于单薄，感觉拿不出手。除了全栈，多面手，就没亮点了。

考虑到我近些年的经验主要是 web开发。
所以我预计做 4 个项目，慢慢做。会写下自己的思考过程，以及实现过程中遇到的坑，还有一些反思。

因为是验证想法和达到目的，所以主要是vibe coding.

我有github copliot, claude code, droid, iflow cli.
主要编程工具是 zed, vscode.
考虑到主要是验证自己的想法，所以就白嫖iflow cli，也行，国内的模型。vscode review代码。最终就是vscode+ iflow cli extention.



| 项目	| 关键主题	| 技术类型 |
|-|-|-|
| 1. 高并发抽奖系统	| 缓存 + 异步 + 压测	| 极限性能架构 |
| 2. AI内容平台	| 调度 + 自动化 + AI能力 | 	智能系统架构 |
| 3. WebSocket系统	| 实时通信 + 分布式消息 | 	实时体验架构 |
| 4. 🛒 电商核心系统	| 数据一致性 + 事务保真|	企业级业务架构|

day1: 建仓库：[高并发抽奖系统](https://github.com/d60-Lab/fast-lottery-engine)
技术选型：
我个人会的技术栈是php/python/golang/rust
rust一直没用起来,所以我打算玩下rust后端, vue 做前端。
基于我之前阅读过极客课程，《如何做秒杀系统》。印象中的思想就是 层层过滤。
抽奖也是一样的， 你能不能抽中其实是个概率问题。
那么从性能优化的角度思考，我可以把概率计算放到每个环节。
<!--more-->

为了防止AI幻觉，今天我就是和ai讨论，我的要求是：
1. 多分支体现技术演进。specs 分支 就记录讨论结果下的规范。
2. 然后xx1, xxx2 一直演进。xx1就关注业务定义和技术实现，只关注业务的正确性。也可以作为一个基础衡量指标，看不优化的场景下，并发量支持多少。


业务定义：
1. 抽奖需要登录 （为了测试方便，直接脚本批量注册 千万账号）
2. 管理后台配置抽奖活动，多少个奖品，不同的奖品数据多少，中奖概率是多少。（简单起见，免登录）
3. 需要展示中奖历史，就是xxx中奖了，全局的。也就是告诉新用户，确实有人中奖。 （都会实现，谁中了奖需要马上出现，当然我觉得加 1 秒的缓存也是可以的）
4. 不能有超卖，就是奖品只设置了 4 个，结果有人中奖肯定是不行的。
5. 加个简单的防刷机制，防止机器人预登录，批量调接口

ps: iflow cli画excalidraw流程图不行。之前用claude code cli, 一次出图。这次用了kimi/qwen-code-plus/glm-4.6 都不行。 一个图错误，一个打不开。
![image.png](http://blog.go2live.cn/static/upload/202510/26t9fcslra82.png)
最后用deepseek3.2搞定了。不管怎么样，🉐拥抱AI, 用好了确实提升效率。
![image.png](http://blog.go2live.cn/static/upload/202510/268s358b83np.png)

数据库模型设计和api设计还行，不过api设计有缺陷，之前没有 活动管理接口。

## 数据模型设计

### 用户表 (users)
```sql
id, username, password, email, last_lottery_at, created_at, updated_at
```

### 活动表 (activities)
```sql
id, name, description, start_time, end_time, status, created_at, updated_at
```

### 奖项表 (prizes)
```sql
id, activity_id, name, description, total_count, remaining_count, probability, is_enabled, created_at, updated_at
```

### 抽奖记录表 (lottery_records)
```sql
id, user_id, prize_id, prize_name, created_at
```

## API接口规划

### 用户相关
- POST /api/auth/login - 用户登录
- POST /api/auth/register - 用户注册
- GET /api/user/profile - 获取用户信息
- GET /api/user/lottery-history - 获取用户抽奖历史

### 抽奖相关
- POST /api/lottery/draw - 执行抽奖
- GET /api/lottery/prizes - 获取可抽奖品列表
- GET /api/lottery/result/{id} - 获取抽奖结果
- GET /api/lottery/global-history - 获取全局中奖历史（分页）

### 管理后台
- GET /admin/api/prizes - 获取奖品列表
- POST /admin/api/prizes - 创建奖品
- PUT /admin/api/prizes/{id} - 更新奖品
- DELETE /admin/api/prizes/{id} - 删除奖品
- GET /admin/api/statistics - 获取统计数据
- GET /admin/api/activities - 获取活动列表
- POST /admin/api/activities - 创建活动
- PUT /admin/api/activities/{id} - 更新活动状态
