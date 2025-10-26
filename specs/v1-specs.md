# 高性能抽奖系统 v1 规格说明

## 项目概述
构建一个高性能实时抽奖系统demo，支持高并发场景，采用Go后端 + Vue前端 + Docker-compose部署。

## 第一期目标 (specs分支)
专注于业务正确性验证，不追求极致性能，确保功能完整可用。

## 核心功能规格

### 1. 抽奖场景
- **实时抽奖**：用户登录后可立即参与转盘抽奖
- **转盘界面**：传统转盘样式，支持动画效果
- **即时结果**：抽中后即时显示结果并更新库存

### 2. 用户系统
- **用户登录**：支持用户注册和登录
- **用户身份**：每个用户有唯一标识，用于防重复抽奖
- **抽奖记录**：记录用户参与历史

### 3. 管理后台
- **奖项设置**：可配置多个奖项（一等奖、二等奖等）
- **奖品管理**：设置每个奖项的奖品数量、启用/禁用状态
- **概率配置**：设置各奖项的中奖概率
- **库存管理**：实时查看奖品剩余数量，库存预警
- **活动管理**：设置活动时间、状态管理
- **全局中奖历史**：分页展示所有用户中奖记录

### 4. 业务规则
- **库存控制**：奖品抽完后不再中该奖项，严格防止超卖
- **概率计算**：按配置概率进行随机抽取
- **公平性**：确保随机算法公平，不可预测
- **防刷机制**：
  - 单用户抽奖频率限制（如1分钟1次）
  - 活动时间限制（开始/结束时间）
  - 用户总抽奖次数限制（可选）
- **活动状态**：支持未开始/进行中/已结束/暂停状态
- **奖品状态**：支持启用/禁用状态

## 技术架构

### 后端 (Go)
- **框架**：Gin（已确定）
- **数据库**：MySQL 8.0（主数据库）
- **缓存**：Redis（库存缓存和会话管理）
- **认证**：JWT Token
- **压测支持**：内置性能监控指标

### 前端 (Vue)
- **框架**：Vue 3 + TypeScript
- **UI库**：Element Plus 或 Ant Design Vue
- **动画**：转盘动画效果
- **管理后台**：前后端分离的管理界面

### 部署
- **容器化**：Docker + Docker-compose
- **服务拆分**：前端、后端、数据库、Redis分离
- **环境配置**：开发/测试/生产环境配置

## 性能目标 (第一期)
- **并发用户**：100-500并发
- **响应时间**：< 500ms
- **准确率**：100%业务正确性
- **可用性**：99.9%

## 压测方案设计

### 压测目标
- **验证并发能力**：找出系统的实际并发上限
- **验证业务正确性**：高并发下无超卖、无重复中奖
- **性能基线**：建立v1版本性能基准数据

### 压测场景
1. **注册压测**：批量注册1000万测试账号
2. **登录压测**：测试登录接口并发能力
3. **抽奖压测**：模拟真实抽奖场景
4. **混合压测**：登录+抽奖混合场景

### 压测工具
- **Vegeta**：Go语言编写的HTTP压测工具
- **自定义脚本**：批量注册和用户登录脚本
- **数据验证**：压测后验证库存数据一致性

### 压测指标
- **QPS**：每秒查询数
- **响应时间**：P50、P95、P99延迟
- **错误率**：业务错误和系统错误
- **并发数**：同时在线用户数
- **资源使用**：CPU、内存、数据库连接数

### 压测步骤
1. **环境准备**：Docker-compose启动完整环境
2. **数据准备**：批量注册测试用户，配置奖品数据
3. **基准测试**：单机单接口压测，建立基线
4. **业务测试**：完整抽奖流程压测
5. **数据验证**：检查库存一致性、中奖记录完整性
6. **报告输出**：生成压测报告和性能分析

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

## 演进计划
- **specs分支**：规格设计和基础框架
- **v1分支**：基础功能实现，业务正确性
- **v2分支**：性能优化，高并发支持
- **v3分支**：完整功能，生产就绪

### 前端交互规范
- **抽奖动画**：转盘旋转3-5秒，期间禁用抽奖按钮
- **结果展示**：中奖结果展示2-3秒后自动关闭
- **错误处理**：库存不足、活动结束等需要友好提示
- **实时更新**：全局中奖历史直接查询，无缓存优化

## 压测工具设计

### 批量注册脚本
```bash
# 批量注册1000万用户
./scripts/batch-register.sh --count=10000000 --concurrent=100
```

### 压测执行脚本
```bash
# 登录压测
echo "POST http://localhost:8080/api/auth/login" | vegeta attack -duration=30s -rate=1000 | vegeta report

# 抽奖压测（需要携带token）
echo "POST http://localhost:8080/api/lottery/draw" | vegeta attack -duration=60s -rate=500 | vegeta report
```

### 数据验证脚本
```bash
# 验证库存一致性
./scripts/validate-inventory.sh

# 验证无重复中奖
./scripts/validate-duplicate-wins.sh
```

## 待讨论事项
1. 具体的概率算法实现方式（权重法 vs 区间法）
2. 前端转盘的动画实现方案（CSS动画 vs Canvas）
3. 管理后台免登录实现
4. 错误码和异常处理规范
5. MySQL8的表引擎选择（InnoDB）和索引设计
6. 是否需要连接池监控和慢查询日志
7. 活动状态管理（未开始/进行中/已结束/暂停）