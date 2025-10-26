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
- **奖品管理**：设置每个奖项的奖品数量
- **概率配置**：设置各奖项的中奖概率
- **库存管理**：实时查看奖品剩余数量

### 4. 业务规则
- **库存控制**：奖品抽完后不再中该奖项
- **概率计算**：按配置概率进行随机抽取
- **公平性**：确保随机算法公平，不可预测
- **防刷机制**：基本的时间间隔限制

## 技术架构

### 后端 (Go)
- **框架**：Gin 或 Echo
- **数据库**：MySQL + Redis
- **缓存**：Redis用于库存缓存和并发控制
- **认证**：JWT Token

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

## 数据模型设计

### 用户表 (users)
```sql
id, username, password, email, created_at, updated_at
```

### 奖项表 (prizes)
```sql
id, name, description, total_count, remaining_count, probability, created_at
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

### 管理后台
- GET /admin/api/prizes - 获取奖品列表
- POST /admin/api/prizes - 创建奖品
- PUT /admin/api/prizes/{id} - 更新奖品
- DELETE /admin/api/prizes/{id} - 删除奖品
- GET /admin/api/statistics - 获取统计数据

## 演进计划
- **specs分支**：规格设计和基础框架
- **v1分支**：基础功能实现，业务正确性
- **v2分支**：性能优化，高并发支持
- **v3分支**：完整功能，生产就绪

## 待讨论事项
1. 具体的概率算法实现方式
2. 并发控制策略（乐观锁/悲观锁）
3. 前端转盘的动画实现方案
4. 管理后台的权限控制
5. 日志和监控方案